use std::collections::{BTreeSet, HashMap};

use babylon_bsl::evaluator::Value as BslValue;
use babylon_bsl::identity_codec::{
    encode_stable_bsl_value_v1, encode_value_v1, project_stable_value_v1, StableBslValueV1,
};
use babylon_bsl::query::EdgeKey;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::stable_element::{StableElementKeyV1, StableElementResolverV1};
use babylon_graph::stable_state::encode_stable_graph_state_v1;
use babylon_graph::substrate::{GraphSubstrate, HyperedgeId, NodeId};
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1, RngDomainV2};
use babylon_kernel::tick_content_hash::{
    OrderedPracticeActionBatchDigestV1, PreparedEnvironmentDigestV1, RefDigestV1,
    StableWorldDigestV1, TickContentPartsV1, TickContentPreimageV1, TickPayloadDigestV1,
};
use babylon_kernel::{
    seed_for, seed_for_v2, sha256_of, ContentDigest, Currency, H3CellId, H3CellIdError, KernelRng,
    Ratio, SessionId,
};
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use serde::Deserialize;
use serde_json::Value;

#[path = "support/tick_content_hash_v1_vectors.rs"]
mod contract_support;

const VECTORS: &str = include_str!("../../../../contracts/tick_content_hash_v1_vectors.jsonl");
const MAX_ROWS: usize = 256;
const MAX_LINE_BYTES: usize = 262_144;
const REQUIRED_FAMILIES: [&str; 20] = [
    "action_id",
    "bsl_discriminant",
    "carrier_segment",
    "mutation",
    "ordered_action_batch",
    "prepared_environment",
    "refusal",
    "register_manifest",
    "register_set",
    "replay_seed",
    "replay_session",
    "resolver_manifest",
    "rng_v1",
    "rng_v2",
    "stable_element",
    "stable_carrier_key",
    "stable_graph",
    "stable_world",
    "tick_content_hash",
    "tick_payload",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorRow {
    id: String,
    kind: String,
    data: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TagRefusal {
    Unknown,
    Truncated,
    Trailing,
    Order,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FutureMaterialCellIdentity {
    cell_id: H3CellId,
}

#[test]
fn kernel_h3_identity_surface_is_available_to_tick_material_carriers() {
    let raw = 0x0852_8347_3fff_ffff_u64;
    let from_raw: Result<H3CellId, H3CellIdError> = H3CellId::try_from(raw);
    let cell = from_raw.expect("checked raw H3 identity");
    let sql = i64::try_from(cell).expect("fixture H3 identity fits PostgreSQL BIGINT");
    let from_sql: Result<H3CellId, H3CellIdError> = H3CellId::try_from(sql);
    let from_text: Result<H3CellId, H3CellIdError> = cell.to_string().parse();
    let ancestor: Result<H3CellId, H3CellIdError> = cell.ancestor_at(4);
    let carrier = FutureMaterialCellIdentity { cell_id: cell };

    assert_eq!(from_sql, Ok(carrier.cell_id));
    assert_eq!(from_text, Ok(carrier.cell_id));
    assert_eq!(ancestor.expect("coarser H3 ancestor").resolution(), 4);
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], TagRefusal> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or(TagRefusal::Truncated)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(TagRefusal::Truncated)?;
        self.offset = end;
        Ok(value)
    }

    fn tag(&mut self, expected: u8, allowed: &[u8]) -> Result<(), TagRefusal> {
        let actual = *self.take(1)?.first().ok_or(TagRefusal::Truncated)?;
        if !allowed.contains(&actual) {
            return Err(TagRefusal::Unknown);
        }
        if actual != expected {
            return Err(TagRefusal::Order);
        }
        Ok(())
    }

    const fn finish(self) -> Result<(), TagRefusal> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(TagRefusal::Trailing)
        }
    }
}

fn parse_vector_rows(input: &str) -> Result<Vec<VectorRow>, &'static str> {
    let input = input.strip_suffix('\n').unwrap_or(input);
    let mut rows = Vec::with_capacity(MAX_ROWS);
    for (index, line) in input.split('\n').take(MAX_ROWS + 1).enumerate() {
        if index == MAX_ROWS {
            return Err("too_many_rows");
        }
        let content = line.strip_suffix('\r').unwrap_or(line);
        if content.is_empty() || content.len() > MAX_LINE_BYTES {
            return Err("invalid_line_length");
        }
        rows.push(serde_json::from_str(content).map_err(|_| "invalid_json")?);
    }
    Ok(rows)
}

fn rows() -> Vec<VectorRow> {
    parse_vector_rows(VECTORS).expect("closed bounded vector rows")
}

fn valid_vector_line(length: usize) -> String {
    let prefix = r#"{"id":""#;
    let suffix = r#"","kind":"x","data":{}}"#;
    assert!(length >= prefix.len() + suffix.len());
    assert!(length <= MAX_LINE_BYTES + 1);
    format!(
        "{prefix}{}{suffix}",
        "x".repeat(length - prefix.len() - suffix.len())
    )
}

#[test]
fn vector_loader_executes_governed_maximum_and_plus_one_inputs() {
    let row = r#"{"id":"x","kind":"x","data":{}}"#;
    let maximum_rows = std::iter::repeat_n(row, MAX_ROWS)
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        parse_vector_rows(&maximum_rows)
            .expect("governed row maximum")
            .len(),
        MAX_ROWS
    );
    let oversized_rows = std::iter::repeat_n(row, MAX_ROWS + 1)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(matches!(
        parse_vector_rows(&oversized_rows),
        Err("too_many_rows")
    ));
    assert_eq!(
        parse_vector_rows(&valid_vector_line(MAX_LINE_BYTES))
            .expect("governed line maximum")
            .len(),
        1
    );
    assert!(matches!(
        parse_vector_rows(&valid_vector_line(MAX_LINE_BYTES + 1)),
        Err("invalid_line_length")
    ));
}

fn row<'a>(rows: &'a [VectorRow], kind: &str) -> &'a VectorRow {
    rows.iter()
        .take(MAX_ROWS)
        .find(|row| row.kind == kind)
        .expect("required vector family")
}

fn row_by_id<'a>(rows: &'a [VectorRow], id: &str) -> &'a VectorRow {
    rows.iter()
        .take(MAX_ROWS)
        .find(|row| row.id == id)
        .expect("required vector id")
}

fn linked_digest(rows: &[VectorRow], data: &Value, name: &str) -> [u8; 32] {
    let digest_name = format!("{name}_digest_hex");
    if let Some(value) = data[&digest_name].as_str() {
        return hex32(value);
    }
    let id_name = format!("{name}_id");
    let id = text(data, &id_name);
    hex32(text(&row_by_id(rows, id).data, "digest_hex"))
}

fn text<'a>(data: &'a Value, name: &str) -> &'a str {
    data[name].as_str().expect("text field")
}

fn integer(data: &Value, name: &str) -> i64 {
    data[name].as_i64().expect("signed integer field")
}

fn unsigned(data: &Value, name: &str) -> u64 {
    data[name].as_u64().expect("unsigned integer field")
}

fn hex_bytes(value: &str) -> Vec<u8> {
    assert!(value.len() <= MAX_LINE_BYTES && value.len().is_multiple_of(2));
    let mut output = Vec::with_capacity(value.len() / 2);
    for chunk in value.as_bytes().chunks_exact(2).take(MAX_LINE_BYTES / 2) {
        let text = std::str::from_utf8(chunk).expect("ASCII hex");
        output.push(u8::from_str_radix(text, 16).expect("lowercase hex byte"));
    }
    output
}

fn hex32(value: &str) -> [u8; 32] {
    hex_bytes(value).try_into().expect("32-byte digest")
}

fn verify_fixed_tags(
    bytes: &[u8],
    expected_tags: &[u8],
    payload_bytes: usize,
) -> Result<(), TagRefusal> {
    let mut cursor = Cursor::new(bytes);
    for expected in expected_tags.iter().take(10) {
        cursor.tag(*expected, expected_tags)?;
        cursor.take(payload_bytes)?;
    }
    cursor.finish()
}

fn verify_tick_content_tags(bytes: &[u8]) -> Result<(), TagRefusal> {
    let mut cursor = Cursor::new(bytes);
    cursor.take(b"babylon.tick-content\0".len() + 4)?;
    cursor.tag(1, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])?;
    cursor.take(4)?;
    let session_length = u16::from_be_bytes(cursor.take(2)?.try_into().expect("u16"));
    cursor.take(usize::from(session_length))?;
    for (tag, payload) in [(2, 8), (3, 16), (4, 68)] {
        cursor.tag(tag, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])?;
        cursor.take(payload)?;
    }
    for tag in 5..=10 {
        cursor.tag(tag, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])?;
        cursor.take(36)?;
    }
    cursor.finish()
}

#[test]
fn shared_corpus_is_bounded_complete_and_digest_exact() {
    let rows = rows();
    let received: BTreeSet<&str> = rows
        .iter()
        .take(MAX_ROWS)
        .map(|row| row.kind.as_str())
        .collect();
    let required: BTreeSet<&str> = REQUIRED_FAMILIES.into_iter().collect();
    assert_eq!(received, required);
    for row in rows.iter().take(MAX_ROWS) {
        let Some(canonical_hex) = row.data["canonical_hex"].as_str() else {
            continue;
        };
        let canonical = hex_bytes(canonical_hex);
        if let Some(digest_hex) = row.data["digest_hex"].as_str() {
            assert_eq!(sha256_of(&canonical), hex32(digest_hex), "{}", row.id);
        }
    }
}

#[test]
fn every_canonical_row_is_reconstructed_from_semantic_inputs() {
    contract_support::verify_every_canonical_row(&rows());
}

#[test]
fn every_outer_input_and_nested_layout_mutation_moves_identity() {
    contract_support::verify_every_outer_mutation(&rows());
}

#[test]
fn every_shared_refusal_is_executed() {
    contract_support::verify_every_refusal(&rows());
}

fn corpus_graph(
    shift_handles: bool,
) -> (
    MemoryGraph,
    HashMap<NodeId, String>,
    HashMap<HyperedgeId, String>,
) {
    let mut graph = MemoryGraph::new();
    if shift_handles {
        let discarded = graph.add_node("DISCARDED").unwrap();
        graph.remove_node(discarded).unwrap();
    }
    let workers = graph.add_node("SOCIAL_CLASS").unwrap();
    let capital = graph.add_node("SOCIAL_CLASS").unwrap();
    graph.add_edge("OWNS", capital, workers, 0.75).unwrap();
    let coalition = graph
        .add_hyperedge("COMMUNITY", &[workers, capital])
        .unwrap();
    graph
        .update_node(workers, "social-class/active", -0.0)
        .unwrap();
    graph
        .update_node(capital, "social-class/active", 1.5)
        .unwrap();
    graph
        .update_node_currency(
            workers,
            "economy/wealth",
            Currency::from_micro_units(-1_234_567_890_123),
        )
        .unwrap();
    graph
        .update_edge("OWNS", capital, workers, "relation/tension", -2.25)
        .unwrap();
    graph
        .update_hyperedge_attribute(coalition, "organization/cohesion", 0.125)
        .unwrap();
    (
        graph,
        HashMap::from([
            (workers, "workers".to_owned()),
            (capital, "capital".to_owned()),
        ]),
        HashMap::from([(coalition, "coalition-one".to_owned())]),
    )
}

#[test]
fn production_stable_graph_matches_cross_allocation_vectors() {
    let vectors = rows();
    for shift_handles in [false, true] {
        let (mut graph, node_names, hyperedge_names) = corpus_graph(shift_handles);
        let resolver = StableElementResolverV1::seal(
            &graph,
            "demo/cross-allocation",
            &node_names,
            &hyperedge_names,
        )
        .unwrap();
        let prior = encode_stable_graph_state_v1(&graph, &resolver).unwrap();
        assert_eq!(
            prior.canonical_bytes(),
            hex_bytes(text(
                &row_by_id(&vectors, "cross-allocation-stable-graph").data,
                "canonical_hex",
            ))
        );
        let workers = *node_names
            .iter()
            .find(|(_, name)| name.as_str() == "workers")
            .expect("workers name")
            .0;
        graph
            .update_node(workers, "social-class/active", 2.5)
            .unwrap();
        let result = encode_stable_graph_state_v1(&graph, &resolver).unwrap();
        assert_eq!(
            result.canonical_bytes(),
            hex_bytes(text(
                &row_by_id(&vectors, "result-stable-graph").data,
                "canonical_hex",
            ))
        );
    }
}

#[test]
fn stable_bsl_projection_reuses_the_governed_stable_element_bytes() {
    let vectors = rows();
    let (graph, node_names, hyperedge_names) = corpus_graph(false);
    let resolver = StableElementResolverV1::seal(
        &graph,
        "demo/cross-allocation",
        &node_names,
        &hyperedge_names,
    )
    .expect("sealed cross-allocation resolver");
    let workers = *node_names
        .iter()
        .find(|(_, name)| name.as_str() == "workers")
        .expect("workers")
        .0;
    let capital = *node_names
        .iter()
        .find(|(_, name)| name.as_str() == "capital")
        .expect("capital")
        .0;
    let coalition = *hyperedge_names
        .iter()
        .find(|(_, name)| name.as_str() == "coalition-one")
        .expect("coalition")
        .0;
    let cases = [
        ("bsl-value-int", BslValue::Int(-9)),
        (
            "bsl-value-currency",
            BslValue::Currency(Currency::from_micro_units(12_345_678_901_234_567_890)),
        ),
        ("bsl-value-real", BslValue::Real(-0.0)),
        (
            "bsl-value-ratio",
            BslValue::Ratio {
                value: Ratio::new(0.5).expect("governed ratio value"),
                floor: None,
                cap: Some(Ratio::new(1.0).expect("governed ratio cap")),
            },
        ),
        ("bsl-value-bool", BslValue::Bool(false)),
        (
            "bsl-value-enum",
            BslValue::Enum {
                enum_type: "Status".to_owned(),
                member: "ALPHA".to_owned(),
            },
        ),
        ("bsl-value-node-ref", BslValue::NodeRef(workers)),
        ("bsl-value-hyperedge-ref", BslValue::HyperedgeRef(coalition)),
        (
            "bsl-value-edge-ref",
            BslValue::EdgeRef(EdgeKey {
                source: capital,
                target: workers,
                edge_type: "OWNS".to_owned(),
            }),
        ),
    ];

    for (tick_id, runtime_value) in cases {
        let projected = project_stable_value_v1(&runtime_value, &resolver)
            .expect("runtime reference projects through the sealed resolver");
        let mut projected_bytes = Vec::new();
        encode_stable_bsl_value_v1(&projected, &mut projected_bytes).expect("stable value encodes");
        let mut runtime_bytes = Vec::new();
        encode_value_v1(&runtime_value, &resolver, &mut runtime_bytes)
            .expect("runtime value encodes");
        let tick_bytes = hex_bytes(text(&row_by_id(&vectors, tick_id).data, "canonical_hex"));

        assert_eq!(projected_bytes, runtime_bytes);
        assert_eq!(projected_bytes, tick_bytes);
    }

    let mut other_scenario_bytes = Vec::new();
    encode_stable_bsl_value_v1(
        &StableBslValueV1::Node(StableElementKeyV1::Node {
            scenario: "demo/other-scenario".to_owned(),
            local_name: "workers".to_owned(),
        }),
        &mut other_scenario_bytes,
    )
    .expect("the distinct scenario has its own stable identity");
    assert_ne!(
        other_scenario_bytes,
        hex_bytes(text(
            &row_by_id(&vectors, "bsl-value-node-ref").data,
            "canonical_hex",
        ))
    );
}

#[test]
fn production_rng_matches_both_language_neutral_vectors() {
    let vectors = rows();
    let v1 = &row(&vectors, "rng_v1").data;
    let session = SessionId::new(text(v1, "session")).expect("current session");
    assert_eq!(
        seed_for(
            &session,
            unsigned(v1, "tick"),
            text(v1, "domain"),
            text(v1, "carrier")
        ),
        hex32(text(v1, "stream_seed_hex"))
    );
    let mut rng = KernelRng::for_carrier(
        &session,
        unsigned(v1, "tick"),
        text(v1, "domain"),
        text(v1, "carrier"),
    );
    for expected in v1["first_four_u64"]
        .as_array()
        .expect("V1 draws")
        .iter()
        .take(4)
    {
        assert_eq!(rng.next_u64(), expected.as_u64().expect("u64 draw"));
    }

    let v2 = &row(&vectors, "rng_v2").data;
    let replay = ReplaySessionIdV1::try_from(text(v2, "session")).expect("replay session");
    let seed = ReplaySeed::new(integer(v2, "seed"));
    let domain = RngDomainV2::try_from(text(v2, "domain")).expect("RNG domain");
    let carrier_row = row_by_id(&vectors, text(v2, "carrier_id"));
    assert_eq!(carrier_row.kind, "stable_carrier_key");
    assert_eq!(
        text(&carrier_row.data, "resolver_id"),
        "cross-allocation-resolver-manifest"
    );
    assert_eq!(
        text(&carrier_row.data, "stable_graph_id"),
        "cross-allocation-stable-graph"
    );
    let (graph, node_names, hyperedge_names) = corpus_graph(false);
    let resolver = StableElementResolverV1::seal(
        &graph,
        "demo/cross-allocation",
        &node_names,
        &hyperedge_names,
    )
    .expect("sealed graph resolver");
    let workers = *node_names
        .iter()
        .find(|(_, name)| name.as_str() == "workers")
        .expect("workers")
        .0;
    let capital = *node_names
        .iter()
        .find(|(_, name)| name.as_str() == "capital")
        .expect("capital")
        .0;
    let coalition = *hyperedge_names
        .iter()
        .find(|(_, name)| name.as_str() == "coalition-one")
        .expect("coalition")
        .0;
    let subject = resolver.node_key(workers).expect("stable subject").clone();
    let active = [
        resolver
            .edge_key("OWNS", capital, workers)
            .expect("stable edge"),
        resolver
            .hyperedge_key(coalition)
            .expect("stable hyperedge")
            .clone(),
    ];
    let carrier = resolver
        .carrier_key(&subject, &active, integer(&carrier_row.data, "draw_slot"))
        .expect("graph-owned stable carrier");
    assert_eq!(
        carrier.validated_bytes(),
        hex_bytes(text(&carrier_row.data, "canonical_hex"))
    );
    assert_eq!(
        seed_for_v2(
            &replay,
            seed,
            unsigned(v2, "tick"),
            &domain,
            carrier.validated_bytes(),
        )
        .expect("V2 key"),
        hex32(text(v2, "stream_seed_hex"))
    );
    let mut rng = KernelRng::for_carrier_v2(
        &replay,
        seed,
        unsigned(v2, "tick"),
        &domain,
        carrier.validated_bytes(),
    )
    .expect("V2 stream");
    for expected in v2["first_nine_u64"]
        .as_array()
        .expect("V2 draws")
        .iter()
        .take(9)
    {
        assert_eq!(rng.next_u64(), expected.as_u64().expect("u64 draw"));
    }
    let mut fresh = KernelRng::for_carrier_v2(
        &replay,
        seed,
        unsigned(v2, "tick"),
        &domain,
        carrier.validated_bytes(),
    )
    .expect("fresh V2 stream");
    assert_eq!(fresh.next_f64().to_bits(), unsigned(v2, "fresh_f64_bits"));
}

#[test]
fn production_primitives_and_outer_preimage_match_shared_corpus() {
    let vectors = rows();
    let replay_row = &row(&vectors, "replay_session").data;
    let replay = ReplaySessionIdV1::try_from(text(replay_row, "session")).expect("replay session");
    assert_eq!(
        replay.canonical_bytes().expect("session bytes"),
        hex_bytes(text(replay_row, "canonical_hex"))
    );
    let element_row = &row(&vectors, "stable_element").data;
    let element = StableElementKeyV1::Node {
        scenario: text(element_row, "scenario").to_owned(),
        local_name: text(element_row, "local_name").to_owned(),
    };
    assert_eq!(
        element.canonical_bytes().expect("stable element"),
        hex_bytes(text(element_row, "canonical_hex"))
    );
    let carrier_row = &row(&vectors, "carrier_segment").data;
    assert_eq!(
        element
            .carrier_segment()
            .expect("carrier")
            .as_str()
            .as_bytes(),
        hex_bytes(text(carrier_row, "canonical_hex"))
    );
    let action_row = &row(&vectors, "ordered_action_batch").data;
    let action_session =
        ReplaySessionIdV1::try_from(text(action_row, "session")).expect("action session");
    let actions =
        OrderedPracticeActionBatchV1::empty(action_session, unsigned(action_row, "resolve_tick"))
            .expect("empty actions");
    assert_eq!(
        actions.canonical_bytes(),
        hex_bytes(text(action_row, "canonical_hex"))
    );
    assert_eq!(
        actions.digest().as_bytes(),
        &hex32(text(action_row, "digest_hex"))
    );
    verify_outer_vector(&vectors);
}

fn verify_outer_vector(vectors: &[VectorRow]) {
    let data = &row(vectors, "tick_content_hash").data;
    let session = ReplaySessionIdV1::try_from(text(data, "session")).expect("outer session");
    let action_row = row_by_id(vectors, text(data, "actions_id"));
    assert_eq!(action_row.kind, "ordered_action_batch");
    assert!(action_row.data["items"]
        .as_array()
        .expect("runtime action items")
        .is_empty());
    assert_eq!(text(&action_row.data, "session"), text(data, "session"));
    assert_eq!(
        unsigned(&action_row.data, "resolve_tick"),
        unsigned(data, "resolve_tick")
    );
    let action_batch =
        OrderedPracticeActionBatchV1::empty(session.clone(), unsigned(data, "resolve_tick"))
            .expect("exact empty runtime action batch");
    assert_eq!(
        action_batch.digest().as_bytes(),
        &linked_digest(vectors, data, "actions")
    );
    let content = ContentDigest {
        defines_hash: hex32(text(data, "defines_digest_hex")),
        rules_hash: hex32(text(data, "rules_digest_hex")),
    };
    let parts = TickContentPartsV1 {
        session: &session,
        resolve_tick: unsigned(data, "resolve_tick"),
        seed: ReplaySeed::new(integer(data, "seed")),
        content: &content,
        reference: RefDigestV1::from_bytes(hex32(text(data, "reference_digest_hex"))),
        prepared: PreparedEnvironmentDigestV1::from_bytes(linked_digest(vectors, data, "prepared")),
        prior_world: StableWorldDigestV1::from_bytes(linked_digest(vectors, data, "prior_world")),
        actions: OrderedPracticeActionBatchDigestV1::from_bytes(linked_digest(
            vectors, data, "actions",
        )),
        result_world: StableWorldDigestV1::from_bytes(linked_digest(vectors, data, "result_world")),
        payload: TickPayloadDigestV1::from_bytes(linked_digest(vectors, data, "payload")),
    };
    let preimage = TickContentPreimageV1::compose(&parts).expect("outer preimage");
    assert_eq!(preimage.as_bytes(), hex_bytes(text(data, "canonical_hex")));
    assert_eq!(
        preimage.digest().as_bytes(),
        &hex32(text(data, "digest_hex"))
    );
    assert_eq!(verify_tick_content_tags(preimage.as_bytes()), Ok(()));
}

#[test]
fn bounded_tag_parser_has_exact_closed_refusals() {
    let valid = hex_bytes("01000000010200000001");
    assert_eq!(verify_fixed_tags(&valid, &[1, 2], 4), Ok(()));
    let mut unknown = valid.clone();
    unknown[0] = 0xff;
    assert_eq!(
        verify_fixed_tags(&unknown, &[1, 2], 4),
        Err(TagRefusal::Unknown)
    );
    assert_eq!(
        verify_fixed_tags(&valid[..9], &[1, 2], 4),
        Err(TagRefusal::Truncated)
    );
    let mut trailing = valid.clone();
    trailing.push(0);
    assert_eq!(
        verify_fixed_tags(&trailing, &[1, 2], 4),
        Err(TagRefusal::Trailing)
    );
    let mut order = valid;
    order[0] = 2;
    order[5] = 1;
    assert_eq!(
        verify_fixed_tags(&order, &[1, 2], 4),
        Err(TagRefusal::Order)
    );
}
