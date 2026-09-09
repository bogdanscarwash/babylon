//! Production-operation execution for the semantic maximum/+1 recipes.

use std::collections::HashMap;

use babylon_bsl::evaluator::Value as BslValue;
use babylon_bsl::exemptions::IntensiveAggregationExemption;
use babylon_bsl::fuel::IntrinsicCosts;
use babylon_bsl::identity_codec::IdentityCodecError;
use babylon_bsl::identity_sections::{
    encode_prepared_bsl_sections_v1, encode_tick_payload_sections_v1,
};
use babylon_bsl::typecheck::TypeEnv;
use babylon_bsl::types::EnumRegistry;
use babylon_bsl::vocabulary::{ClosedVocabulary, EnumKind};
use babylon_graph::memory::MemoryGraph;
use babylon_graph::stable_element::{
    StableElementKeyV1, StableElementResolverV1, StableIdentityError,
};
use babylon_graph::stable_state::encode_stable_graph_state_v1;
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::{GraphSubstrate, HyperedgeId, NodeId};
use babylon_kernel::replay::{ReplayIdentityError, ReplaySessionIdV1, RngDomainV2};
use babylon_kernel::Currency;
use babylon_practice_contract::actor_v2::ActorOrganizationIdV2;
use babylon_practice_contract::ordered_action_v1::{
    OrderedPracticeActionBatchV1, OrderedPracticeActionError,
};
use babylon_practice_contract::{
    input_authority_ledger_v2_digest, CampaignIdV2, InputAuthorityIdV2, PracticeAuthorityKindV2,
    PracticeBatchV2Error, PracticeIdV2, PracticeInputAuthorityLedgerV2, PracticeInputAuthorityV2,
    PracticeIntentV2, PracticeTargetIdentityV2, PracticeTargetTagV2, ProposalNonceV2,
    ResolvedPracticeBatchItemV2, ResolvedPracticeBatchV2, ResolvedPracticeBatchV2Error,
    TaggedPracticeTargetV2,
};
use serde_json::Value;

pub(super) fn execute(name: &str, recipe: &Value) -> Result<(), &'static str> {
    match name {
        "replay_session_bytes"
        | "rng_domain_bytes"
        | "rng_domain_segments"
        | "symbol_bytes"
        | "qname_bytes"
        | "qname_segments"
        | "structural_type_bytes"
        | "intrinsic_identity_bytes"
        | "enum_type_bytes"
        | "enum_member_bytes"
        | "governance_utf8_bytes" => execute_text(name, recipe),
        "stable_carrier_active_elements" | "stable_carrier_bytes" => {
            execute_carrier(recipe).map_err(carrier_error)
        }
        "resolver_rows"
        | "resolver_edges"
        | "resolver_hyperedge_members"
        | "resolver_fact_units"
        | "resolver_manifest_bytes" => execute_resolver(recipe).map_err(graph_error),
        "stable_graph_elements"
        | "stable_graph_attribute_rows"
        | "stable_graph_hyperedge_members"
        | "stable_graph_fact_units"
        | "stable_graph_bytes" => execute_stable_graph(recipe).map_err(graph_error),
        "ordered_action_items" | "ordered_action_batch_bytes" => {
            execute_ordered_actions(recipe).map_err(action_error)
        }
        "prepared_rows"
        | "prepared_small_rows"
        | "prepared_enum_members"
        | "prepared_vocabulary_members"
        | "prepared_aggregate_rows"
        | "prepared_combined_bytes" => execute_prepared(recipe).map_err(bsl_error),
        "tick_rule_outcomes" | "tick_rows" | "tick_aggregate_rows" | "tick_combined_bytes" => {
            execute_tick(recipe).map_err(bsl_error)
        }
        _ => panic!("unknown production bound operation {name}"),
    }
}

fn repeated_segments(recipe: &Value) -> String {
    counts(recipe, "segment_bytes")
        .iter()
        .take(5)
        .map(|length| "a".repeat(*length))
        .collect::<Vec<_>>()
        .join("/")
}

fn execute_text(name: &str, recipe: &Value) -> Result<(), &'static str> {
    match name {
        "replay_session_bytes" => {
            ReplaySessionIdV1::try_from("s".repeat(count(recipe, "session_bytes")).as_str())
                .and_then(|session| session.canonical_bytes().map(|_| ()))
                .map_err(|error| replay_text_error(name, error))
        }
        "rng_domain_bytes" | "rng_domain_segments" => {
            RngDomainV2::try_from(repeated_segments(recipe).as_str())
                .map(|_| ())
                .map_err(|error| replay_text_error(name, error))
        }
        "symbol_bytes" => execute_symbol("a".repeat(count(recipe, "symbol_bytes")))
            .map_err(|error| bsl_text_error(name, error)),
        "qname_bytes" | "qname_segments" => {
            execute_qname(repeated_segments(recipe)).map_err(|error| bsl_text_error(name, error))
        }
        "structural_type_bytes" => execute_structural_type("A".repeat(count(recipe, "type_bytes")))
            .map_err(|error| graph_text_error(name, error)),
        "intrinsic_identity_bytes" => {
            execute_intrinsic("a".repeat(count(recipe, "identity_bytes")))
                .map_err(|error| bsl_text_error(name, error))
        }
        "enum_type_bytes" => execute_enum_type(format!(
            "A{}",
            "a".repeat(count(recipe, "type_bytes").saturating_sub(1))
        ))
        .map_err(|error| bsl_text_error(name, error)),
        "enum_member_bytes" => execute_enum_member("A".repeat(count(recipe, "member_bytes")))
            .map_err(|error| bsl_text_error(name, error)),
        "governance_utf8_bytes" => execute_governance("a".repeat(count(recipe, "utf8_bytes")))
            .map_err(|error| bsl_text_error(name, error)),
        _ => panic!("non-text production operation {name}"),
    }
}

fn execute_symbol(symbol: String) -> Result<(), IdentityCodecError> {
    let events = vec![("EventType/A".to_owned(), vec![(symbol, BslValue::Int(0))])];
    encode_tick_payload_sections_v1(&[], &events, &[], &empty_resolver()).map(|_| ())
}

fn execute_qname(qname: String) -> Result<(), IdentityCodecError> {
    let constants = HashMap::from([(qname, BslValue::Int(0))]);
    encode_prepared(
        &empty_types(),
        &IntrinsicCosts::default(),
        &constants,
        &EnumRegistry::default(),
        None,
    )
}

fn execute_structural_type(structural_type: String) -> Result<(), StableIdentityError> {
    StableElementKeyV1::Edge {
        scenario: "s".to_owned(),
        edge_type: structural_type,
        source_local_name: "a".to_owned(),
        target_local_name: "b".to_owned(),
    }
    .canonical_bytes()
    .map(|_| ())
}

fn execute_intrinsic(identity: String) -> Result<(), IdentityCodecError> {
    let intrinsics = IntrinsicCosts::new(HashMap::from([(identity, 1)]));
    encode_prepared(
        &empty_types(),
        &intrinsics,
        &HashMap::new(),
        &EnumRegistry::default(),
        None,
    )
}

fn execute_enum_type(enum_type: String) -> Result<(), IdentityCodecError> {
    execute_enum(enum_type, "A".to_owned())
}

fn execute_enum_member(member: String) -> Result<(), IdentityCodecError> {
    execute_enum("A".to_owned(), member)
}

fn execute_enum(enum_type: String, member: String) -> Result<(), IdentityCodecError> {
    let mut enums = EnumRegistry::default();
    enums
        .declare(&enum_type, &[member])
        .expect("one unique bounded enum member");
    encode_prepared(
        &empty_types(),
        &IntrinsicCosts::default(),
        &HashMap::new(),
        &enums,
        None,
    )
}

fn execute_governance(governance: String) -> Result<(), IdentityCodecError> {
    let governance = Box::leak(governance.into_boxed_str());
    let exemptions = Box::leak(
        vec![IntensiveAggregationExemption {
            field_name: "a",
            reason: governance,
            owner: "a",
            date: "a",
        }]
        .into_boxed_slice(),
    );
    let types = TypeEnv {
        fields: HashMap::new(),
        exemptions,
    };
    encode_prepared(
        &types,
        &IntrinsicCosts::default(),
        &HashMap::new(),
        &EnumRegistry::default(),
        None,
    )
}

fn replay_text_error(name: &str, error: ReplayIdentityError) -> &'static str {
    match (name, error) {
        ("replay_session_bytes", ReplayIdentityError::LengthOutOfBounds { .. }) => {
            "string_too_long"
        }
        ("rng_domain_bytes", ReplayIdentityError::LengthOutOfBounds { .. }) => "rng_domain_length",
        ("rng_domain_segments", ReplayIdentityError::InvalidRngDomainQname { .. }) => {
            "rng_domain_segments"
        }
        (_, other) => panic!("unexpected replay text bound error: {other:?}"),
    }
}

fn graph_text_error(name: &str, error: StableIdentityError) -> &'static str {
    match (name, error) {
        ("structural_type_bytes", StableIdentityError::InvalidString { .. }) => {
            "invalid_structural_type"
        }
        (_, other) => panic!("unexpected graph text bound error: {other:?}"),
    }
}

fn bsl_text_error(name: &str, error: IdentityCodecError) -> &'static str {
    let IdentityCodecError::InvalidString { .. } = error else {
        panic!("unexpected BSL text bound error: {error:?}");
    };
    match name {
        "symbol_bytes" => "invalid_symbol",
        "qname_bytes" | "qname_segments" => "invalid_qname",
        "intrinsic_identity_bytes" => "invalid_intrinsic_identity",
        "enum_type_bytes" => "invalid_enum_type",
        "enum_member_bytes" => "invalid_enum_member",
        "governance_utf8_bytes" => "governance_string_too_long",
        _ => panic!("non-BSL text bound {name}"),
    }
}

fn count(recipe: &Value, field: &str) -> usize {
    usize::try_from(recipe.get(field).and_then(Value::as_u64).unwrap_or(0))
        .expect("bounded recipe count")
}

fn signed(recipe: &Value, field: &str) -> i64 {
    recipe[field].as_i64().expect("bounded signed recipe value")
}

fn text<'a>(recipe: &'a Value, field: &str) -> &'a str {
    recipe[field].as_str().expect("bound recipe text")
}

fn counts(recipe: &Value, field: &str) -> Vec<usize> {
    recipe[field]
        .as_array()
        .expect("bound recipe counts")
        .iter()
        .take(32)
        .map(|value| usize::try_from(value.as_u64().expect("bound recipe count")).unwrap())
        .collect()
}

fn patterned_symbol(pattern: &str, index: usize) -> String {
    match pattern {
        "n{index:05x}" => format!("n{index:05x}"),
        "n{index:04x}" => format!("n{index:04x}"),
        "n{index}" => format!("n{index}"),
        "e{index:05x}" => format!("e{index:05x}"),
        "h{index}" => format!("h{index}"),
        "a{index}" => format!("a{index}"),
        "c{index}" => format!("c{index}"),
        "f{index}" => format!("f{index}"),
        "i{index}" => format!("i{index}"),
        "M{index}" => format!("M{index}"),
        "EVENT_{index}" => format!("EVENT_{index}"),
        "NODE_{index}" => format!("NODE_{index}"),
        "r{index}" => format!("r{index}"),
        other => panic!("unknown bound recipe pattern {other}"),
    }
}

fn fixed_symbol(prefix: char, index: usize, length: usize) -> String {
    assert!((2..=64).contains(&length));
    let suffix = format!("{index:x}");
    assert!(suffix.len() < length);
    let mut output = prefix.to_string();
    output.push_str(&"g".repeat(length - suffix.len() - 1));
    output.push_str(&suffix);
    output
}

fn fixed_qname(prefix: char, index: usize, length: usize) -> String {
    if length <= 64 {
        return fixed_symbol(prefix, index, length);
    }
    assert!((66..=128).contains(&length));
    format!(
        "{}/{}",
        "a".repeat(64),
        fixed_symbol(prefix, index, length - 65)
    )
}

fn add_nodes(
    graph: &mut MemoryGraph,
    count: usize,
    node_type: &str,
    name_pattern: &str,
) -> HashMap<NodeId, String> {
    assert!(count <= 65_537);
    let mut names = HashMap::with_capacity(count);
    for index in (0..=65_536).take(count) {
        let node = graph.add_node(node_type).expect("bounded synthetic node");
        names.insert(node, patterned_symbol(name_pattern, index));
    }
    names
}

fn execute_resolver(recipe: &Value) -> Result<(), StableIdentityError> {
    let fixture = text(recipe, "fixture");
    match fixture {
        "resolver_nodes" => resolver_nodes(recipe),
        "resolver_edges" => resolver_edges(recipe),
        "resolver_single_hyperedge" => resolver_single_hyperedge(recipe),
        "resolver_fact_units" => resolver_fact_units(recipe),
        "resolver_manifest_nodes" => resolver_manifest_nodes(recipe),
        _ => panic!("unknown resolver bound fixture {fixture}"),
    }
}

fn resolver_nodes(recipe: &Value) -> Result<(), StableIdentityError> {
    let mut graph = MemoryGraph::new();
    let names = add_nodes(
        &mut graph,
        count(recipe, "node_rows"),
        text(recipe, "node_type"),
        text(recipe, "node_name_pattern"),
    );
    StableElementResolverV1::seal(&graph, "s", &names, &HashMap::new()).map(|_| ())
}

fn resolver_edges(recipe: &Value) -> Result<(), StableIdentityError> {
    let mut graph = MemoryGraph::new();
    let names = add_nodes(
        &mut graph,
        count(recipe, "node_rows"),
        "n",
        text(recipe, "node_name_pattern"),
    );
    let nodes: Vec<_> = names.keys().copied().take(2).collect();
    let edge_count = count(recipe, "edge_rows");
    assert!(edge_count <= 65_537);
    for index in (0..=65_536).take(edge_count) {
        graph
            .add_edge(
                &patterned_symbol(text(recipe, "edge_type_pattern"), index),
                nodes[0],
                nodes[1],
                1.0,
            )
            .expect("bounded synthetic edge");
    }
    StableElementResolverV1::seal(&graph, "s", &names, &HashMap::new()).map(|_| ())
}

fn resolver_single_hyperedge(recipe: &Value) -> Result<(), StableIdentityError> {
    let mut graph = MemoryGraph::new();
    let node_count = count(recipe, "node_rows");
    let names = add_nodes(
        &mut graph,
        node_count,
        "n",
        text(recipe, "node_name_pattern"),
    );
    let members: Vec<_> = (0..=65_534)
        .take(node_count)
        .map(|index| NodeId(index as u64))
        .collect();
    let hyperedge = graph
        .add_hyperedge(text(recipe, "hyperedge_type"), &members)
        .expect("bounded synthetic hyperedge");
    let hyperedge_names = HashMap::from([(hyperedge, text(recipe, "hyperedge_name").to_owned())]);
    StableElementResolverV1::seal(&graph, "s", &names, &hyperedge_names).map(|_| ())
}

fn resolver_fact_units(recipe: &Value) -> Result<(), StableIdentityError> {
    let mut graph = MemoryGraph::new();
    let node_count = count(recipe, "node_rows");
    let names = add_nodes(
        &mut graph,
        node_count,
        "n",
        text(recipe, "node_name_pattern"),
    );
    let mut hyperedge_names = HashMap::new();
    for (index, member_count) in counts(recipe, "hyperedge_member_rows")
        .into_iter()
        .take(32)
        .enumerate()
    {
        let members: Vec<_> = (0..65_534)
            .take(member_count)
            .map(|member| NodeId(member as u64))
            .collect();
        let hyperedge = graph
            .add_hyperedge("h", &members)
            .expect("bounded fact-unit hyperedge");
        hyperedge_names.insert(
            hyperedge,
            patterned_symbol(text(recipe, "hyperedge_name_pattern"), index),
        );
    }
    StableElementResolverV1::seal(&graph, "s", &names, &hyperedge_names).map(|_| ())
}

fn resolver_manifest_nodes(recipe: &Value) -> Result<(), StableIdentityError> {
    let full_count = count(recipe, "full_node_rows");
    assert!(full_count <= 65_535);
    let mut graph = MemoryGraph::new();
    let mut names = HashMap::with_capacity(full_count + 1);
    for index in (0..65_535).take(full_count) {
        let node_type = "T".repeat(count(recipe, "full_node_type_bytes"));
        let node = graph.add_node(&node_type).expect("manifest full node");
        names.insert(
            node,
            fixed_symbol('n', index, count(recipe, "full_node_name_bytes")),
        );
    }
    let final_type = "T".repeat(count(recipe, "final_node_type_bytes"));
    let final_node = graph.add_node(&final_type).expect("manifest final node");
    names.insert(
        final_node,
        fixed_symbol('z', full_count, count(recipe, "final_node_name_bytes")),
    );
    StableElementResolverV1::seal(&graph, text(recipe, "scenario"), &names, &HashMap::new())
        .map(|_| ())
}

fn execute_carrier(recipe: &Value) -> Result<(), StableIdentityError> {
    match text(recipe, "fixture") {
        "sealed_carrier_active_stack" => carrier_active_stack(recipe),
        "sealed_carrier_byte_boundary" => carrier_byte_boundary(recipe),
        fixture => panic!("unknown carrier bound fixture {fixture}"),
    }
}

fn carrier_active_stack(recipe: &Value) -> Result<(), StableIdentityError> {
    let mut graph = MemoryGraph::new();
    let node = graph.add_node("n").expect("carrier node");
    let names = HashMap::from([(node, "n".to_owned())]);
    let resolver = StableElementResolverV1::seal(&graph, "s", &names, &HashMap::new())?;
    let key = resolver.node_key(node)?.clone();
    let active = vec![key.clone(); count(recipe, "active_element_count")];
    resolver.carrier_key(&key, &active, 0).map(|_| ())
}

fn carrier_byte_boundary(recipe: &Value) -> Result<(), StableIdentityError> {
    let scenario = format!("{}/{}", "a".repeat(64), "b".repeat(63));
    assert_eq!(scenario.len(), count(recipe, "scenario_bytes"));
    let mut graph = MemoryGraph::new();
    let subject = graph.add_node("n").expect("carrier subject");
    let source = graph.add_node("n").expect("carrier source");
    let target = graph.add_node("n").expect("carrier target");
    let edge_type = "E".repeat(count(recipe, "active_edge_type_bytes"));
    graph
        .add_edge(&edge_type, source, target, 1.0)
        .expect("carrier edge");
    let names = HashMap::from([
        (
            subject,
            fixed_symbol('s', 0, count(recipe, "subject_local_name_bytes")),
        ),
        (
            source,
            fixed_symbol('a', 0, count(recipe, "active_endpoint_name_bytes")),
        ),
        (
            target,
            fixed_symbol('b', 0, count(recipe, "active_endpoint_name_bytes")),
        ),
    ]);
    let resolver = StableElementResolverV1::seal(&graph, &scenario, &names, &HashMap::new())?;
    let subject_key = resolver.node_key(subject)?.clone();
    let edge_key = resolver.edge_key(&edge_type, source, target)?;
    let active = vec![edge_key; count(recipe, "active_element_count")];
    resolver
        .carrier_key(&subject_key, &active, signed(recipe, "draw_slot"))
        .map(|_| ())
}

#[derive(Debug, Clone)]
enum AttributeRecipe {
    None,
    Indexed {
        count: usize,
        pattern: String,
    },
    ByteBoundary {
        full_count: usize,
        full_length: usize,
        final_length: usize,
    },
}

#[derive(Debug, Clone)]
struct GeneratedState {
    node_count: usize,
    node_type: String,
    node_f64: AttributeRecipe,
    node_currency: AttributeRecipe,
    hyperedge_members: Option<usize>,
}

impl GeneratedState {
    fn nodes(&self) -> Vec<(NodeId, String)> {
        (0..=65_536)
            .take(self.node_count)
            .map(|index| (NodeId(index as u64), self.node_type.clone()))
            .collect()
    }

    fn f64_rows(&self, recipe: AttributeRecipe) -> Vec<(NodeId, String, f64)> {
        match recipe {
            AttributeRecipe::None => Vec::new(),
            AttributeRecipe::Indexed { count, pattern } => (0..=524_288)
                .take(count)
                .map(|index| (NodeId(0), patterned_symbol(&pattern, index), 0.0))
                .collect(),
            AttributeRecipe::ByteBoundary {
                full_count,
                full_length,
                final_length,
            } => {
                let mut rows = Vec::with_capacity(full_count + 1);
                for index in (0..524_288).take(full_count) {
                    rows.push((NodeId(0), fixed_qname('q', index, full_length), 0.0));
                }
                rows.push((NodeId(0), fixed_qname('z', full_count, final_length), 0.0));
                rows
            }
        }
    }
}

impl CanonicalState for GeneratedState {
    fn all_nodes(&self) -> Vec<(NodeId, String)> {
        self.nodes()
    }

    fn all_attributes(&self) -> Vec<(NodeId, String, f64)> {
        self.f64_rows(self.node_f64.clone())
    }

    fn all_edges(&self) -> Vec<(String, NodeId, NodeId, f64)> {
        Vec::new()
    }

    fn all_hyperedges(&self) -> Vec<(HyperedgeId, String, Vec<NodeId>)> {
        let Some(count) = self.hyperedge_members else {
            return Vec::new();
        };
        let members = (0..=65_534)
            .take(count)
            .map(|index| NodeId(index as u64))
            .collect();
        vec![(HyperedgeId(0), "h".to_owned(), members)]
    }

    fn all_edge_attributes(&self) -> Vec<(String, NodeId, NodeId, String, f64)> {
        Vec::new()
    }

    fn all_currency_attributes(&self) -> Vec<(NodeId, String, Currency)> {
        match &self.node_currency {
            AttributeRecipe::None | AttributeRecipe::ByteBoundary { .. } => Vec::new(),
            AttributeRecipe::Indexed { count, pattern } => (0..=524_288)
                .take(*count)
                .map(|index| {
                    (
                        NodeId(0),
                        patterned_symbol(pattern, index),
                        Currency::from_micro_units(0),
                    )
                })
                .collect(),
        }
    }

    fn all_hyperedge_attributes(&self) -> Vec<(HyperedgeId, String, f64)> {
        Vec::new()
    }
}

fn one_node_resolver(name: &str) -> StableElementResolverV1 {
    let mut graph = MemoryGraph::new();
    let node = graph.add_node("n").expect("stable graph node");
    StableElementResolverV1::seal(
        &graph,
        "s",
        &HashMap::from([(node, name.to_owned())]),
        &HashMap::new(),
    )
    .expect("one-node stable resolver")
}

fn execute_stable_graph(recipe: &Value) -> Result<(), StableIdentityError> {
    match text(recipe, "fixture") {
        "stable_graph_nodes" => stable_graph_nodes(recipe),
        "stable_graph_node_f64" => stable_graph_attributes(recipe),
        "stable_graph_single_hyperedge" => stable_graph_hyperedge(recipe),
        "stable_graph_fact_units" => stable_graph_facts(recipe),
        "stable_graph_byte_boundary" => stable_graph_bytes(recipe),
        fixture => panic!("unknown stable graph bound fixture {fixture}"),
    }
}

fn stable_graph_nodes(recipe: &Value) -> Result<(), StableIdentityError> {
    let mut graph = MemoryGraph::new();
    let names = add_nodes(
        &mut graph,
        count(recipe, "sealed_node_rows"),
        text(recipe, "node_type"),
        text(recipe, "node_name_pattern"),
    );
    let resolver = StableElementResolverV1::seal(&graph, "s", &names, &HashMap::new())?;
    let state = GeneratedState {
        node_count: count(recipe, "state_node_rows"),
        node_type: "n".to_owned(),
        node_f64: AttributeRecipe::None,
        node_currency: AttributeRecipe::None,
        hyperedge_members: None,
    };
    encode_stable_graph_state_v1(&state, &resolver).map(|_| ())
}

fn stable_graph_attributes(recipe: &Value) -> Result<(), StableIdentityError> {
    let resolver = one_node_resolver("n");
    let state = GeneratedState {
        node_count: 1,
        node_type: "n".to_owned(),
        node_f64: AttributeRecipe::Indexed {
            count: count(recipe, "node_f64_rows"),
            pattern: text(recipe, "qname_pattern").to_owned(),
        },
        node_currency: AttributeRecipe::None,
        hyperedge_members: None,
    };
    encode_stable_graph_state_v1(&state, &resolver).map(|_| ())
}

fn stable_graph_hyperedge(recipe: &Value) -> Result<(), StableIdentityError> {
    let sealed = count(recipe, "sealed_member_rows");
    let mut graph = MemoryGraph::new();
    let names = add_nodes(&mut graph, sealed, "n", text(recipe, "node_name_pattern"));
    let members: Vec<_> = (0..65_534)
        .take(sealed)
        .map(|index| NodeId(index as u64))
        .collect();
    let hyperedge = graph
        .add_hyperedge("h", &members)
        .expect("stable hyperedge");
    let resolver = StableElementResolverV1::seal(
        &graph,
        "s",
        &names,
        &HashMap::from([(hyperedge, "h".to_owned())]),
    )?;
    let state = GeneratedState {
        node_count: sealed,
        node_type: "n".to_owned(),
        node_f64: AttributeRecipe::None,
        node_currency: AttributeRecipe::None,
        hyperedge_members: Some(count(recipe, "state_member_rows")),
    };
    encode_stable_graph_state_v1(&state, &resolver).map(|_| ())
}

fn stable_graph_facts(recipe: &Value) -> Result<(), StableIdentityError> {
    let resolver = one_node_resolver("n");
    let state = GeneratedState {
        node_count: 1,
        node_type: "n".to_owned(),
        node_f64: AttributeRecipe::Indexed {
            count: count(recipe, "node_f64_rows"),
            pattern: text(recipe, "f64_qname_pattern").to_owned(),
        },
        node_currency: AttributeRecipe::Indexed {
            count: count(recipe, "node_currency_rows"),
            pattern: text(recipe, "currency_qname_pattern").to_owned(),
        },
        hyperedge_members: None,
    };
    encode_stable_graph_state_v1(&state, &resolver).map(|_| ())
}

fn stable_graph_bytes(recipe: &Value) -> Result<(), StableIdentityError> {
    let resolver = one_node_resolver(text(recipe, "node_name"));
    let state = GeneratedState {
        node_count: 1,
        node_type: text(recipe, "node_type").to_owned(),
        node_f64: AttributeRecipe::ByteBoundary {
            full_count: count(recipe, "full_node_f64_rows"),
            full_length: count(recipe, "full_qname_bytes"),
            final_length: count(recipe, "final_qname_bytes"),
        },
        node_currency: AttributeRecipe::None,
        hyperedge_members: None,
    };
    encode_stable_graph_state_v1(&state, &resolver).map(|_| ())
}

fn execute_prepared(recipe: &Value) -> Result<(), IdentityCodecError> {
    match text(recipe, "fixture") {
        "prepared_constants" => prepared_constants(recipe),
        "prepared_intrinsics" => prepared_intrinsics(recipe),
        "prepared_enum_members" => prepared_enum_members(recipe),
        "prepared_vocabulary_members" => prepared_vocabulary_members(recipe),
        "prepared_vocabulary_aggregate" => prepared_vocabulary_aggregate(recipe),
        "prepared_exemption_byte_boundary" => prepared_exemption_bytes(recipe),
        fixture => panic!("unknown prepared bound fixture {fixture}"),
    }
}

fn empty_types() -> TypeEnv {
    TypeEnv {
        fields: HashMap::new(),
        exemptions: &[],
    }
}

fn encode_prepared(
    types: &TypeEnv,
    intrinsics: &IntrinsicCosts,
    constants: &HashMap<String, BslValue>,
    enums: &EnumRegistry,
    vocabulary: Option<&ClosedVocabulary>,
) -> Result<(), IdentityCodecError> {
    encode_prepared_bsl_sections_v1(types, intrinsics, constants, enums, vocabulary).map(|_| ())
}

fn prepared_constants(recipe: &Value) -> Result<(), IdentityCodecError> {
    let row_count = count(recipe, "constant_rows");
    assert!(row_count <= 65_537);
    assert_eq!(signed(recipe, "constant_value"), 0);
    let constants = (0..=65_536)
        .take(row_count)
        .map(|index| {
            (
                patterned_symbol(text(recipe, "constant_name_pattern"), index),
                BslValue::Int(0),
            )
        })
        .collect();
    encode_prepared(
        &empty_types(),
        &IntrinsicCosts::default(),
        &constants,
        &EnumRegistry::default(),
        None,
    )
}

fn prepared_intrinsics(recipe: &Value) -> Result<(), IdentityCodecError> {
    let row_count = count(recipe, "intrinsic_rows");
    assert!(row_count <= 65);
    let intrinsic_cost =
        u64::try_from(count(recipe, "intrinsic_cost")).expect("bounded intrinsic cost fits u64");
    assert_eq!(intrinsic_cost, 1);
    let costs = (0..=64)
        .take(row_count)
        .map(|index| {
            (
                patterned_symbol(text(recipe, "intrinsic_name_pattern"), index),
                intrinsic_cost,
            )
        })
        .collect();
    encode_prepared(
        &empty_types(),
        &IntrinsicCosts::new(costs),
        &HashMap::new(),
        &EnumRegistry::default(),
        None,
    )
}

fn prepared_enum_members(recipe: &Value) -> Result<(), IdentityCodecError> {
    let row_count = count(recipe, "enum_member_rows");
    assert!(row_count <= 4_097);
    let members: Vec<_> = (0..=4_096)
        .take(row_count)
        .map(|index| patterned_symbol(text(recipe, "enum_member_pattern"), index))
        .collect();
    let mut enums = EnumRegistry::default();
    enums
        .declare(text(recipe, "enum_type"), &members)
        .expect("unique bounded enum members");
    encode_prepared(
        &empty_types(),
        &IntrinsicCosts::default(),
        &HashMap::new(),
        &enums,
        None,
    )
}

fn vocabulary_members(pattern: &str, count: usize) -> Vec<String> {
    assert!(count <= 524_289);
    (0..=524_288)
        .take(count)
        .map(|index| patterned_symbol(pattern, index))
        .collect()
}

fn prepared_vocabulary_members(recipe: &Value) -> Result<(), IdentityCodecError> {
    assert_eq!(text(recipe, "vocabulary_kind"), "EventType");
    let vocabulary = ClosedVocabulary::new([(
        EnumKind::EventType,
        vocabulary_members(
            text(recipe, "vocabulary_member_pattern"),
            count(recipe, "vocabulary_member_rows"),
        ),
    )])
    .expect("unique event vocabulary");
    encode_prepared(
        &empty_types(),
        &IntrinsicCosts::default(),
        &HashMap::new(),
        &EnumRegistry::default(),
        Some(&vocabulary),
    )
}

fn prepared_vocabulary_aggregate(recipe: &Value) -> Result<(), IdentityCodecError> {
    let vocabulary = ClosedVocabulary::new([
        (
            EnumKind::NodeType,
            vocabulary_members(
                text(recipe, "node_member_pattern"),
                count(recipe, "node_type_members"),
            ),
        ),
        (
            EnumKind::EventType,
            vocabulary_members(
                text(recipe, "event_member_pattern"),
                count(recipe, "event_type_members"),
            ),
        ),
    ])
    .expect("disjoint aggregate vocabulary");
    encode_prepared(
        &empty_types(),
        &IntrinsicCosts::default(),
        &HashMap::new(),
        &EnumRegistry::default(),
        Some(&vocabulary),
    )
}

fn leaked_text(length: usize) -> &'static str {
    Box::leak("x".repeat(length).into_boxed_str())
}

fn prepared_exemption_bytes(recipe: &Value) -> Result<(), IdentityCodecError> {
    let full = leaked_text(count(recipe, "full_governance_field_bytes"));
    let fields = ["a", "b", "c", "d", "e"];
    let mut rows = Vec::with_capacity(6);
    for field in fields
        .into_iter()
        .take(count(recipe, "full_exemption_rows"))
    {
        rows.push(IntensiveAggregationExemption {
            field_name: field,
            reason: full,
            owner: full,
            date: full,
        });
    }
    rows.push(IntensiveAggregationExemption {
        field_name: "f",
        reason: leaked_text(count(recipe, "final_reason_bytes")),
        owner: leaked_text(count(recipe, "final_owner_bytes")),
        date: leaked_text(count(recipe, "final_date_bytes")),
    });
    let exemptions = Box::leak(rows.into_boxed_slice());
    let types = TypeEnv {
        fields: HashMap::new(),
        exemptions,
    };
    encode_prepared(
        &types,
        &IntrinsicCosts::default(),
        &HashMap::new(),
        &EnumRegistry::default(),
        None,
    )
}

fn empty_resolver() -> StableElementResolverV1 {
    StableElementResolverV1::seal(&MemoryGraph::new(), "s", &HashMap::new(), &HashMap::new())
        .expect("empty stable resolver")
}

fn execute_tick(recipe: &Value) -> Result<(), IdentityCodecError> {
    let fixture = text(recipe, "fixture");
    let resolver = empty_resolver();
    let (outcomes, events) = match fixture {
        "tick_rule_outcomes" => (
            tick_outcomes(
                count(recipe, "rule_outcome_rows"),
                text(recipe, "rule_name_pattern"),
            ),
            Vec::new(),
        ),
        "tick_events" => (
            Vec::new(),
            tick_events(count(recipe, "event_rows"), text(recipe, "event_name")),
        ),
        "tick_payload_aggregate" => (Vec::new(), tick_aggregate_event(recipe)),
        "tick_payload_byte_boundary" => (Vec::new(), tick_byte_event(recipe)),
        _ => panic!("unknown tick bound fixture {fixture}"),
    };
    encode_tick_payload_sections_v1(&outcomes, &events, &[], &resolver).map(|_| ())
}

fn tick_outcomes(count: usize, pattern: &str) -> Vec<(String, usize)> {
    assert!(count <= 65_537);
    (0..=65_536)
        .take(count)
        .map(|index| (patterned_symbol(pattern, index), 0))
        .collect()
}

fn tick_events(count: usize, event_name: &str) -> Vec<(String, Vec<(String, BslValue)>)> {
    assert!(count <= 1_048_577);
    (0..=1_048_576)
        .take(count)
        .map(|_| (event_name.to_owned(), Vec::new()))
        .collect()
}

fn tick_aggregate_event(recipe: &Value) -> Vec<(String, Vec<(String, BslValue)>)> {
    let count = count(recipe, "payload_rows");
    assert!(count <= 1_048_576);
    assert_eq!(signed(recipe, "payload_value"), 0);
    let payload = (0..1_048_576)
        .take(count)
        .map(|_| (text(recipe, "payload_label").to_owned(), BslValue::Int(0)))
        .collect();
    vec![(text(recipe, "event_name").to_owned(), payload)]
}

fn tick_byte_event(recipe: &Value) -> Vec<(String, Vec<(String, BslValue)>)> {
    let full_count = count(recipe, "full_payload_rows");
    assert!(full_count <= 871_543);
    assert_eq!(signed(recipe, "payload_value"), 0);
    let mut payload = Vec::with_capacity(full_count + 1);
    for _index in (0..871_543).take(full_count) {
        payload.push((
            "a".repeat(count(recipe, "full_payload_label_bytes")),
            BslValue::Int(0),
        ));
    }
    payload.push((
        "b".repeat(count(recipe, "final_payload_label_bytes")),
        BslValue::Int(0),
    ));
    vec![(text(recipe, "event_name").to_owned(), payload)]
}

fn execute_ordered_actions(recipe: &Value) -> Result<(), OrderedPracticeActionError> {
    let ledger = action_ledger();
    let item_count = count(recipe, "item_count");
    assert!(item_count <= 4_097);
    let evidence_count = count(recipe, "evidence_digests_per_intent");
    let items = (0..=4_096)
        .take(item_count)
        .map(|index| action_item(index, evidence_count))
        .collect();
    let source = ResolvedPracticeBatchV2 {
        schema_version: 2,
        campaign_id: CampaignIdV2::from_bytes([0x10; 16]),
        resolve_tick: 11,
        authority_ledger_digest: input_authority_ledger_v2_digest(&ledger)
            .expect("bounded action ledger digest"),
        resource_allocation_contract_digest: [0x40; 32],
        content_digest: [0x30; 32],
        items,
    };
    let session = ReplaySessionIdV1::try_from("s".repeat(count(recipe, "session_bytes")).as_str())
        .expect("bounded replay session");
    OrderedPracticeActionBatchV1::project(session, &source, &ledger).map(|_| ())
}

fn action_authority() -> PracticeInputAuthorityV2 {
    PracticeInputAuthorityV2 {
        schema_version: 2,
        campaign_id: CampaignIdV2::from_bytes([0x10; 16]),
        authority_kind: PracticeAuthorityKindV2::PlayerSeat,
        input_authority_id: InputAuthorityIdV2::from_bytes([0x20; 16]),
        actor_org_id: ActorOrganizationIdV2::from_bytes(7_u64.to_be_bytes()),
        effective_from_tick: 10,
        effective_through_tick_exclusive: 20,
        decision_content_digest: [0x30; 32],
    }
}

fn action_ledger() -> PracticeInputAuthorityLedgerV2 {
    PracticeInputAuthorityLedgerV2 {
        schema_version: 2,
        rows: vec![action_authority()],
    }
}

fn action_item(index: usize, evidence_count: usize) -> ResolvedPracticeBatchItemV2 {
    let mut nonce = [0_u8; 16];
    nonce[8..].copy_from_slice(&(index as u64).to_be_bytes());
    let evidence_digests = (0_u8..64)
        .take(evidence_count)
        .map(|evidence| {
            let mut digest = [0_u8; 32];
            digest[31] = evidence;
            digest
        })
        .collect();
    ResolvedPracticeBatchItemV2 {
        authority: action_authority(),
        intent: PracticeIntentV2 {
            schema_version: 2,
            submit_after_tick: 10,
            resolve_tick: 11,
            input_authority_id: InputAuthorityIdV2::from_bytes([0x20; 16]),
            actor_org_id: ActorOrganizationIdV2::from_bytes(7_u64.to_be_bytes()),
            practice_id: PracticeIdV2::Strike,
            target: TaggedPracticeTargetV2 {
                tag: PracticeTargetTagV2::LaborProcess,
                identity: PracticeTargetIdentityV2::from_bytes([0x50; 32]),
            },
            proposal_nonce: ProposalNonceV2::from_bytes(nonce),
            quoted_content_digest: [0x30; 32],
            quoted_resource_contract_digest: [0x40; 32],
            parameters: Vec::new(),
            evidence_digests,
        },
    }
}

fn carrier_error(error: StableIdentityError) -> &'static str {
    match error {
        StableIdentityError::ActiveElementLimit { .. } => "active_element_limit",
        StableIdentityError::ByteLimit { .. } => "byte_limit",
        other => panic!("unexpected carrier bound error: {other:?}"),
    }
}

fn graph_error(error: StableIdentityError) -> &'static str {
    match error {
        StableIdentityError::ResolverRowLimit { .. } => "row_limit",
        StableIdentityError::EdgeLimit { .. } => "edge_limit",
        StableIdentityError::StateSectionLimit { section, .. }
            if section.contains("hyperedge members") =>
        {
            "hyperedge_members"
        }
        StableIdentityError::StateSectionLimit { .. } => "row_limit",
        StableIdentityError::FactUnitLimit { .. } => "aggregate_row_limit",
        StableIdentityError::ByteLimit { .. } => "byte_limit",
        other => panic!("unexpected graph bound error: {other:?}"),
    }
}

fn bsl_error(error: IdentityCodecError) -> &'static str {
    match error {
        IdentityCodecError::RowLimit { .. } => "row_limit",
        IdentityCodecError::AggregateRowLimit { .. } => "aggregate_row_limit",
        IdentityCodecError::ByteLimit { .. } => "byte_limit",
        other => panic!("unexpected BSL bound error: {other:?}"),
    }
}

fn action_error(error: OrderedPracticeActionError) -> &'static str {
    match error {
        OrderedPracticeActionError::Source(ResolvedPracticeBatchV2Error::Batch(
            PracticeBatchV2Error::BatchItemLimit,
        )) => "row_limit",
        OrderedPracticeActionError::BatchLength { .. } => "byte_limit",
        other => panic!("unexpected ordered-action bound error: {other:?}"),
    }
}
