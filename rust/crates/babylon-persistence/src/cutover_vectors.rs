//! Strict bounded JSONL adapter for the language-neutral persistence vectors.

use std::collections::{BTreeMap, BTreeSet};

use babylon_bsl::identity_codec::StableBslValueV1;
use babylon_graph::stable_element::StableElementKeyV1;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::semantic_codec::{self, SemanticCodecErrorV1, SemanticRefusalCodeV1};

const MAX_VECTOR_BYTES: usize = 262_144;
const MAX_VECTOR_ROWS: usize = 128;
const MAX_VECTOR_LINE_BYTES: usize = 65_536;
const GOVERNED_VECTOR_ROWS: usize = 56;
const GOVERNED_VECTOR_SHA256: [u8; 32] = [
    0xeb, 0x7e, 0x50, 0xf8, 0x87, 0xe3, 0x9a, 0x30, 0xd4, 0x8e, 0x08, 0x5b, 0x2d, 0x9b, 0x00, 0x1b,
    0xb3, 0xab, 0xd8, 0x23, 0x08, 0x9d, 0x7b, 0xd6, 0xdf, 0x7c, 0x7a, 0x06, 0x6e, 0x68, 0xff, 0x94,
];
const AUTHORITY_LEDGER_DOMAIN: &[u8] = b"babylon.persistence-authority-ledger-row.v1\0";
const AUTHORITY_LEDGER_LAYOUT: u32 = 1;
const PREPARED_AUTHORITY_ROW_SHA256: [u8; 32] = [
    0x7d, 0x9d, 0x13, 0x78, 0x2b, 0x60, 0x34, 0x86, 0xb3, 0xc0, 0x3f, 0x1a, 0x90, 0xd7, 0x3a, 0x7d,
    0xa0, 0x52, 0x43, 0xd7, 0x58, 0x18, 0xce, 0x32, 0xfc, 0x00, 0xbf, 0x56, 0x8a, 0x23, 0x24, 0x40,
];

/// One bounded cutover-vector verification failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustPersistenceVectorErrorV1 {
    /// Corpus or row bytes exceeded a governed bound.
    Bound {
        field: &'static str,
        actual: usize,
        maximum: usize,
    },
    /// JSON was malformed or carried an open shape.
    Shape { field: &'static str },
    /// One semantic operation did not produce its declared exact result.
    Semantic { id: Box<str>, field: &'static str },
}

/// One independently executed vector row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustPersistenceVectorOutcomeV1 {
    id: Box<str>,
    kind: Box<str>,
    row_codec: Option<Box<str>>,
}

impl RustPersistenceVectorOutcomeV1 {
    /// Borrow the untrusted diagnostic row identifier after successful execution.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Borrow the closed vector kind after successful execution.
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }
}

/// Aggregate execution counts for the exact governed corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustPersistenceVectorReportV1 {
    row_count: usize,
    kind_counts: BTreeMap<Box<str>, usize>,
    valid_row_codec_counts: BTreeMap<Box<str>, usize>,
}

impl RustPersistenceVectorReportV1 {
    /// Return the executed row count.
    #[must_use]
    pub const fn row_count(&self) -> usize {
        self.row_count
    }

    /// Return the exact count for one closed vector kind.
    #[must_use]
    pub fn kind_count(&self, kind: &str) -> usize {
        self.kind_counts.get(kind).copied().unwrap_or_default()
    }

    /// Return executions of one valid-row codec.
    #[must_use]
    pub fn valid_row_codec_count(&self, codec: &str) -> usize {
        self.valid_row_codec_counts
            .get(codec)
            .copied()
            .unwrap_or_default()
    }
}

/// Execute one bounded language-neutral vector independently of corpus identity.
///
/// # Errors
/// Returns a bound, closed-shape, typed semantic, exact-byte, or digest mismatch.
pub fn verify_rust_persistence_cutover_vector_row_v1(
    row: &[u8],
) -> Result<RustPersistenceVectorOutcomeV1, RustPersistenceVectorErrorV1> {
    if row.len() > MAX_VECTOR_LINE_BYTES {
        return Err(RustPersistenceVectorErrorV1::Bound {
            field: "vector row bytes",
            actual: row.len(),
            maximum: MAX_VECTOR_LINE_BYTES,
        });
    }
    let value: Value =
        serde_json::from_slice(row).map_err(|_| RustPersistenceVectorErrorV1::Shape {
            field: "vector JSON",
        })?;
    execute_row(&value)
}

/// Execute the exact governed 56-row vector corpus.
///
/// # Errors
/// Returns before a report if corpus identity, bounds, shape, or any owned
/// typed semantic rule differs.
pub fn verify_rust_persistence_cutover_vectors_v1(
    vectors: &[u8],
) -> Result<RustPersistenceVectorReportV1, RustPersistenceVectorErrorV1> {
    if vectors.len() > MAX_VECTOR_BYTES {
        return Err(RustPersistenceVectorErrorV1::Bound {
            field: "vector corpus bytes",
            actual: vectors.len(),
            maximum: MAX_VECTOR_BYTES,
        });
    }
    if semantic_codec::digest(vectors) != GOVERNED_VECTOR_SHA256 {
        return Err(RustPersistenceVectorErrorV1::Shape {
            field: "vector corpus digest",
        });
    }
    let mut lines = vectors.split(|byte| *byte == b'\n').collect::<Vec<_>>();
    if lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    if lines.len() != GOVERNED_VECTOR_ROWS
        || lines.len() > MAX_VECTOR_ROWS
        || lines.iter().any(|line| line.is_empty())
    {
        return Err(RustPersistenceVectorErrorV1::Shape {
            field: "vector corpus rows",
        });
    }
    let mut ids = BTreeSet::new();
    let mut kind_counts = BTreeMap::new();
    let mut valid_row_codec_counts = BTreeMap::new();
    for line in &lines {
        let outcome = verify_rust_persistence_cutover_vector_row_v1(line)?;
        if !ids.insert(outcome.id.clone()) {
            return Err(RustPersistenceVectorErrorV1::Shape {
                field: "vector row id",
            });
        }
        *kind_counts.entry(outcome.kind.clone()).or_insert(0) += 1;
        if let Some(codec) = outcome.row_codec {
            *valid_row_codec_counts.entry(codec).or_insert(0) += 1;
        }
    }
    Ok(RustPersistenceVectorReportV1 {
        row_count: lines.len(),
        kind_counts,
        valid_row_codec_counts,
    })
}

fn execute_row(
    value: &Value,
) -> Result<RustPersistenceVectorOutcomeV1, RustPersistenceVectorErrorV1> {
    let object = as_object(value, "vector row")?;
    let id = string(object, "id")?;
    if id.is_empty() {
        return Err(RustPersistenceVectorErrorV1::Shape {
            field: "vector row id",
        });
    }
    let kind = string(object, "kind")?;
    let row_codec = match kind {
        "valid_scalar" => {
            exact_keys(object, &["id", "kind", "codec", "input", "expected_hex"])?;
            execute_valid_scalar(id, object)?;
            None
        }
        "valid_row" => {
            exact_keys(
                object,
                &[
                    "id",
                    "kind",
                    "codec",
                    "data",
                    "expected_key_hex",
                    "expected_payload_hex",
                    "expected_hex",
                    "expected_sha256",
                ],
            )?;
            Some(execute_valid_row(id, object)?.into())
        }
        "valid_foundation" => {
            exact_keys(
                object,
                &[
                    "id",
                    "kind",
                    "data",
                    "expected_content_bundle_hex",
                    "expected_hex",
                    "expected_sha256",
                ],
            )?;
            execute_foundation(id, object)?;
            None
        }
        "valid_checkpoint" => {
            exact_keys(
                object,
                &["id", "kind", "data", "expected_hex", "expected_sha256"],
            )?;
            execute_checkpoint(id, object)?;
            None
        }
        "valid_empty_family" => {
            exact_keys(
                object,
                &["id", "kind", "data", "expected_hex", "expected_sha256"],
            )?;
            execute_empty_proof(id, object)?;
            None
        }
        "valid_authority_ledger" => {
            exact_keys(
                object,
                &["id", "kind", "data", "expected_hex", "expected_sha256"],
            )?;
            execute_authority_ledger(id, object)?;
            None
        }
        "refusal" => {
            execute_refusal(id, object)?;
            None
        }
        _ => return semantic(id, "vector kind"),
    };
    Ok(RustPersistenceVectorOutcomeV1 {
        id: id.into(),
        kind: kind.into(),
        row_codec,
    })
}

fn execute_valid_scalar(
    id: &str,
    object: &Map<String, Value>,
) -> Result<(), RustPersistenceVectorErrorV1> {
    let codec = string(object, "codec")?;
    let input = field(object, "input")?;
    let encoded = match codec {
        "bool_u8" => semantic_result(
            id,
            semantic_codec::encode_bool(boolean_value(input, "scalar bool")?),
        )?,
        "u64_be" => semantic_result(
            id,
            semantic_codec::encode_u64(decimal_u64(input, "scalar u64")?),
        )?,
        "i64_be" => semantic_result(
            id,
            semantic_codec::encode_i64(decimal_i64(input, "scalar i64")?),
        )?,
        "i128_be" => semantic_result(
            id,
            semantic_codec::encode_i128(decimal_i128(input, "scalar i128")?),
        )?,
        "f64_be_canonical" => semantic_result(id, semantic_codec::encode_f64(vector_f64(input)?))?,
        "h3_cell_id_i64_be" => semantic_result(
            id,
            semantic_codec::encode_h3(decimal_i128(input, "scalar H3")?),
        )?,
        "optional_bounded_utf8" => semantic_result(
            id,
            semantic_codec::encode_optional_utf8(optional_string(input, "optional UTF-8")?),
        )?,
        "stable_bsl_value_v1" => semantic_result(
            id,
            semantic_codec::encode_stable_bsl(&parse_stable_bsl(input)?),
        )?,
        _ => return semantic(id, "scalar codec"),
    };
    compare_hex(
        id,
        &encoded,
        string(object, "expected_hex")?,
        "scalar bytes",
    )
}

#[allow(clippy::too_many_lines)]
fn execute_valid_row<'a>(
    id: &str,
    object: &'a Map<String, Value>,
) -> Result<&'a str, RustPersistenceVectorErrorV1> {
    let codec = string(object, "codec")?;
    let data = as_object(field(object, "data")?, "valid row data")?;
    let row = match codec {
        "stable_graph_node_v1" => {
            exact_keys(data, &["local_name", "node_type"])?;
            semantic_result(
                id,
                semantic_codec::encode_stable_graph_node(
                    string(data, "local_name")?,
                    string(data, "node_type")?,
                ),
            )?
        }
        "stable_graph_node_f64_v1" => {
            exact_keys(data, &["local_name", "qname", "value"])?;
            semantic_result(
                id,
                semantic_codec::encode_stable_graph_node_f64(
                    string(data, "local_name")?,
                    string(data, "qname")?,
                    vector_f64(field(data, "value")?)?,
                ),
            )?
        }
        "stable_graph_edge_v1" => {
            exact_keys(data, &["edge_type", "source", "target", "strength"])?;
            semantic_result(
                id,
                semantic_codec::encode_stable_graph_edge(
                    string(data, "edge_type")?,
                    string(data, "source")?,
                    string(data, "target")?,
                    vector_f64(field(data, "strength")?)?,
                ),
            )?
        }
        "stable_graph_hyperedge_v1" => {
            exact_keys(data, &["local_name", "hyperedge_type", "ordered_members"])?;
            let members = string_array(field(data, "ordered_members")?, "ordered members")?;
            semantic_result(
                id,
                semantic_codec::encode_stable_graph_hyperedge(
                    string(data, "local_name")?,
                    string(data, "hyperedge_type")?,
                    &members,
                ),
            )?
        }
        "stable_graph_edge_f64_v1" => {
            exact_keys(data, &["edge_type", "source", "target", "qname", "value"])?;
            semantic_result(
                id,
                semantic_codec::encode_stable_graph_edge_f64(
                    string(data, "edge_type")?,
                    string(data, "source")?,
                    string(data, "target")?,
                    string(data, "qname")?,
                    vector_f64(field(data, "value")?)?,
                ),
            )?
        }
        "stable_graph_node_currency_v1" => {
            exact_keys(data, &["local_name", "qname", "micro_units"])?;
            semantic_result(
                id,
                semantic_codec::encode_stable_graph_node_currency(
                    string(data, "local_name")?,
                    string(data, "qname")?,
                    decimal_i128(field(data, "micro_units")?, "micro units")?,
                ),
            )?
        }
        "stable_graph_hyperedge_f64_v1" => {
            exact_keys(data, &["local_name", "qname", "value"])?;
            semantic_result(
                id,
                semantic_codec::encode_stable_graph_hyperedge_f64(
                    string(data, "local_name")?,
                    string(data, "qname")?,
                    vector_f64(field(data, "value")?)?,
                ),
            )?
        }
        "world_register_v1" => {
            exact_keys(data, &["register_name", "value"])?;
            let value = parse_stable_bsl(field(data, "value")?)?;
            semantic_result(
                id,
                semantic_codec::encode_world_register(string(data, "register_name")?, &value),
            )?
        }
        "territory_state_v1" => {
            exact_keys(data, &["territory_id", "ordered_fields"])?;
            let territory_id = parse_stable_key(field(data, "territory_id")?)?;
            let values = parse_named_stable(field(data, "ordered_fields")?)?;
            let refs = stable_refs(&values);
            semantic_result(
                id,
                semantic_codec::encode_territory_state(&territory_id, &refs),
            )?
        }
        "dynamic_hex_state_v1" => execute_dynamic_hex(id, data)?,
        "organization_state_v1" => execute_organization(id, data)?,
        "successful_event_v1" => {
            exact_keys(data, &["ordinal", "event_type", "ordered_fields"])?;
            let values = parse_named_stable(field(data, "ordered_fields")?)?;
            let refs = stable_refs(&values);
            semantic_result(
                id,
                semantic_codec::encode_historical_successful_event_v1_vector(
                    integer_u32(field(data, "ordinal")?, "event ordinal")?,
                    string(data, "event_type")?,
                    &refs,
                ),
            )?
        }
        "checkpoint_v1" => {
            exact_keys(
                data,
                &[
                    "section_tag",
                    "ordinal",
                    "completeness_tag",
                    "exact_section_hex",
                ],
            )?;
            let bytes = hex_bytes(string(data, "exact_section_hex")?, "checkpoint section")?;
            semantic_result(
                id,
                semantic_codec::encode_checkpoint_row(
                    integer_u8(field(data, "section_tag")?, "section tag")?,
                    integer_u32(field(data, "ordinal")?, "checkpoint ordinal")?,
                    integer_u8(field(data, "completeness_tag")?, "completeness tag")?,
                    &bytes,
                ),
            )?
        }
        "archive_dirty_receipt_v1" => {
            exact_keys(data, &["tick_content_hash"])?;
            let digest = digest32(string(data, "tick_content_hash")?)?;
            semantic_result(id, semantic_codec::encode_archive_dirty_receipt(&digest))?
        }
        _ => return semantic(id, "row codec"),
    };
    let key = row.key();
    let payload = row.payload();
    compare_hex(id, key, string(object, "expected_key_hex")?, "row key")?;
    compare_hex(
        id,
        payload,
        string(object, "expected_payload_hex")?,
        "row payload",
    )?;
    let mut composite = Vec::with_capacity(key.len() + payload.len());
    composite.extend_from_slice(key);
    composite.extend_from_slice(payload);
    compare_hex(
        id,
        &composite,
        string(object, "expected_hex")?,
        "row composite",
    )?;
    compare_digest(id, &composite, string(object, "expected_sha256")?)?;
    Ok(codec)
}

fn execute_dynamic_hex(
    id: &str,
    data: &Map<String, Value>,
) -> Result<crate::committed_tick_envelope::CommittedTickRowV2, RustPersistenceVectorErrorV1> {
    exact_keys(
        data,
        &[
            "cell_id",
            "c",
            "v",
            "s",
            "k",
            "biocapacity_stock",
            "energy_stock",
            "raw_material_stock",
            "internet_access_pct",
            "surveillance_coupling",
        ],
    )?;
    let values = [
        "c",
        "v",
        "s",
        "k",
        "biocapacity_stock",
        "energy_stock",
        "raw_material_stock",
        "internet_access_pct",
        "surveillance_coupling",
    ]
    .map(|field_name| field(data, field_name).and_then(vector_f64))
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;
    let values: [f64; 9] = values
        .try_into()
        .map_err(|_| RustPersistenceVectorErrorV1::Shape {
            field: "dynamic hex values",
        })?;
    semantic_result(
        id,
        semantic_codec::encode_dynamic_hex_state(
            decimal_u64(field(data, "cell_id")?, "cell id")?,
            &values,
        ),
    )
}

fn execute_organization(
    id: &str,
    data: &Map<String, Value>,
) -> Result<crate::committed_tick_envelope::CommittedTickRowV2, RustPersistenceVectorErrorV1> {
    exact_keys(
        data,
        &[
            "organization_id",
            "organization_kind",
            "ordered_territory_ids",
            "ordered_fields",
        ],
    )?;
    let organization_id = parse_stable_key(field(data, "organization_id")?)?;
    let organization_kind = parse_stable_bsl(field(data, "organization_kind")?)?;
    let ordered_territory_ids = array(
        field(data, "ordered_territory_ids")?,
        "ordered territory identities",
    )?
    .iter()
    .map(parse_stable_key)
    .collect::<Result<Vec<_>, _>>()?;
    let values = parse_named_stable(field(data, "ordered_fields")?)?;
    let refs = stable_refs(&values);
    semantic_result(
        id,
        semantic_codec::encode_organization_state(
            &organization_id,
            &organization_kind,
            &ordered_territory_ids,
            &refs,
        ),
    )
}

fn execute_foundation(
    id: &str,
    object: &Map<String, Value>,
) -> Result<(), RustPersistenceVectorErrorV1> {
    let data = as_object(field(object, "data")?, "foundation data")?;
    exact_keys(
        data,
        &[
            "stable_graph_hex",
            "world_registers_hex",
            "resolver_manifest_hex",
            "prepared_environment_hex",
            "replay_session_identity",
            "rng_seed",
            "content_digest",
            "reference_digest",
            "content_bundle",
        ],
    )?;
    let digest_pair = as_object(field(data, "content_digest")?, "content digest")?;
    exact_keys(digest_pair, &["defines_hash", "rules_hash"])?;
    let bundle = as_object(field(data, "content_bundle")?, "content bundle")?;
    exact_keys(
        bundle,
        &[
            "layout",
            "scenario_source_bytes",
            "prelude_source_bytes",
            "rule_source_bytes",
            "defines_hex",
            "reference_bundle_manifest_hex",
        ],
    )?;
    if integer_u32(field(bundle, "layout")?, "content bundle layout")? != 1 {
        return semantic(id, "content bundle layout");
    }
    let defines = hex_bytes(string(bundle, "defines_hex")?, "defines")?;
    let manifest = hex_bytes(
        string(bundle, "reference_bundle_manifest_hex")?,
        "reference manifest",
    )?;
    let content = semantic_result(
        id,
        semantic_codec::encode_foundation_content(
            string(bundle, "scenario_source_bytes")?,
            optional_string(field(bundle, "prelude_source_bytes")?, "prelude source")?,
            string(bundle, "rule_source_bytes")?,
            &defines,
            &manifest,
        ),
    )?;
    compare_hex(
        id,
        &content,
        string(object, "expected_content_bundle_hex")?,
        "foundation content",
    )?;
    let stable_graph = hex_bytes(string(data, "stable_graph_hex")?, "stable graph")?;
    let world = hex_bytes(string(data, "world_registers_hex")?, "world registers")?;
    let resolver = hex_bytes(string(data, "resolver_manifest_hex")?, "resolver manifest")?;
    let prepared = hex_bytes(
        string(data, "prepared_environment_hex")?,
        "prepared environment",
    )?;
    let defines_hash = digest32(string(digest_pair, "defines_hash")?)?;
    let rules_hash = digest32(string(digest_pair, "rules_hash")?)?;
    let reference = digest32(string(data, "reference_digest")?)?;
    let encoded = semantic_result(
        id,
        semantic_codec::encode_foundation(
            &stable_graph,
            &world,
            &resolver,
            &prepared,
            string(data, "replay_session_identity")?,
            decimal_i64(field(data, "rng_seed")?, "rng seed")?,
            &defines_hash,
            &rules_hash,
            &reference,
            &content,
        ),
    )?;
    compare_hex(
        id,
        &encoded,
        string(object, "expected_hex")?,
        "foundation bytes",
    )?;
    compare_digest(id, &encoded, string(object, "expected_sha256")?)
}

fn execute_checkpoint(
    id: &str,
    object: &Map<String, Value>,
) -> Result<(), RustPersistenceVectorErrorV1> {
    let data = as_object(field(object, "data")?, "checkpoint data")?;
    exact_keys(
        data,
        &[
            "layout",
            "completeness",
            "campaign_id",
            "resolve_tick",
            "sections",
        ],
    )?;
    if integer_u32(field(data, "layout")?, "checkpoint layout")? != 1
        || string(data, "completeness")? != "full"
    {
        return semantic(id, "checkpoint header");
    }
    let uuid = Uuid::parse_str(string(data, "campaign_id")?).map_err(|_| {
        RustPersistenceVectorErrorV1::Shape {
            field: "campaign id",
        }
    })?;
    let sections = array(field(data, "sections")?, "checkpoint sections")?
        .iter()
        .map(|section| {
            let section = as_object(section, "checkpoint section")?;
            exact_keys(section, &["tag", "row_count", "sha256"])?;
            Ok((
                integer_u8(field(section, "tag")?, "section tag")?,
                integer_u32(field(section, "row_count")?, "section row count")?,
                digest32(string(section, "sha256")?)?,
            ))
        })
        .collect::<Result<Vec<_>, RustPersistenceVectorErrorV1>>()?;
    let campaign = crate::identity::CampaignId::from_uuid(uuid);
    let encoded = semantic_result(
        id,
        semantic_codec::encode_full_checkpoint(
            campaign,
            decimal_u64(field(data, "resolve_tick")?, "resolve tick")?,
            &sections,
        ),
    )?;
    compare_hex(
        id,
        &encoded,
        string(object, "expected_hex")?,
        "checkpoint bytes",
    )?;
    compare_digest(id, &encoded, string(object, "expected_sha256")?)
}

fn execute_empty_proof(
    id: &str,
    object: &Map<String, Value>,
) -> Result<(), RustPersistenceVectorErrorV1> {
    let data = as_object(field(object, "data")?, "empty proof data")?;
    exact_keys(
        data,
        &[
            "family",
            "producer",
            "producer_tag",
            "source_count",
            "source_empty_digest",
        ],
    )?;
    if string(data, "family")? != "event"
        || string(data, "producer")? != "successful_event_batch_v1"
    {
        return semantic(id, "empty proof owner");
    }
    let encoded = semantic_result(
        id,
        semantic_codec::encode_empty_proof(
            integer_u8(field(data, "producer_tag")?, "producer tag")?,
            integer_u32(field(data, "source_count")?, "source count")?,
            digest32(string(data, "source_empty_digest")?)?,
        ),
    )?;
    compare_hex(
        id,
        &encoded,
        string(object, "expected_hex")?,
        "empty proof bytes",
    )?;
    compare_digest(id, &encoded, string(object, "expected_sha256")?)
}

fn execute_authority_ledger(
    id: &str,
    object: &Map<String, Value>,
) -> Result<(), RustPersistenceVectorErrorV1> {
    let data = as_object(field(object, "data")?, "authority ledger data")?;
    exact_keys(
        data,
        &[
            "ordinal",
            "state",
            "state_tag",
            "schema_epoch",
            "contract_sha256",
            "reader_contract_sha256",
            "predecessor_sha256",
        ],
    )?;
    let ordinal = integer_u16(field(data, "ordinal")?, "authority ledger ordinal")?;
    let state = string(data, "state")?;
    let state_tag = integer_u8(field(data, "state_tag")?, "authority ledger state tag")?;
    let schema_epoch = integer_u16(
        field(data, "schema_epoch")?,
        "authority ledger schema epoch",
    )?;
    let predecessor = match field(data, "predecessor_sha256")? {
        Value::Null => None,
        value => Some(digest32(value.as_str().ok_or(
            RustPersistenceVectorErrorV1::Shape {
                field: "authority ledger predecessor",
            },
        )?)?),
    };
    match state {
        "prepared" if (ordinal, state_tag, schema_epoch, predecessor) == (1, 1, 8, None) => {}
        "rust_active"
            if (ordinal, state_tag, schema_epoch, predecessor)
                == (2, 2, 9, Some(PREPARED_AUTHORITY_ROW_SHA256)) => {}
        _ => return semantic(id, "authority ledger state identity"),
    }

    let contract_sha256 = digest32(string(data, "contract_sha256")?)?;
    let reader_contract_sha256 = digest32(string(data, "reader_contract_sha256")?)?;
    let mut encoded = Vec::with_capacity(
        AUTHORITY_LEDGER_DOMAIN.len() + 4 + 2 + 1 + 2 + 32 + 32 + 1 + predecessor.map_or(0, |_| 32),
    );
    encoded.extend_from_slice(AUTHORITY_LEDGER_DOMAIN);
    encoded.extend_from_slice(&AUTHORITY_LEDGER_LAYOUT.to_be_bytes());
    encoded.extend_from_slice(&ordinal.to_be_bytes());
    encoded.push(state_tag);
    encoded.extend_from_slice(&schema_epoch.to_be_bytes());
    encoded.extend_from_slice(&contract_sha256);
    encoded.extend_from_slice(&reader_contract_sha256);
    match predecessor {
        None => encoded.push(0),
        Some(digest) => {
            encoded.push(1);
            encoded.extend_from_slice(&digest);
        }
    }
    if state == "prepared" && semantic_codec::digest(&encoded) != PREPARED_AUTHORITY_ROW_SHA256 {
        return semantic(id, "prepared authority row SHA-256");
    }
    compare_hex(
        id,
        &encoded,
        string(object, "expected_hex")?,
        "authority ledger bytes",
    )?;
    compare_digest(id, &encoded, string(object, "expected_sha256")?)
}

#[allow(clippy::too_many_lines)]
fn execute_refusal(
    id: &str,
    object: &Map<String, Value>,
) -> Result<(), RustPersistenceVectorErrorV1> {
    let operation = string(object, "operation")?;
    let input = field(object, "input")?;
    let actual = match operation {
        "encode_scalar" => {
            exact_keys(
                object,
                &["id", "kind", "operation", "codec", "input", "expected_code"],
            )?;
            match string(object, "codec")? {
                "f64_be_canonical" => {
                    capture_refusal(semantic_codec::encode_f64(vector_f64(input)?))
                }
                "h3_cell_id_i64_be" => capture_refusal(semantic_codec::encode_h3(decimal_i128(
                    input,
                    "refusal H3",
                )?)),
                "stable_bsl_value_v1" => {
                    let value = as_object(input, "refusal stable BSL")?;
                    if value.contains_key("tag_u8") {
                        exact_keys(value, &["tag_u8"])?;
                        let tag = integer_u8(field(value, "tag_u8")?, "unknown stable tag")?;
                        if (1..=9).contains(&tag) {
                            return semantic(id, "known stable BSL tag");
                        }
                        capture_refusal::<Vec<u8>>(Err(semantic_codec::refuse_unknown_closed_tag()))
                    } else {
                        exact_keys(value, &["tag", "runtime_handle"])?;
                        if string(value, "tag")? != "node_ref" {
                            return semantic(id, "runtime graph handle tag");
                        }
                        let _handle =
                            integer_u64(field(value, "runtime_handle")?, "runtime handle")?;
                        capture_refusal::<Vec<u8>>(Err(
                            semantic_codec::refuse_runtime_graph_handle(),
                        ))
                    }
                }
                "bounded_utf8" => {
                    let value = as_object(input, "bounded UTF-8 refusal")?;
                    exact_keys(value, &["byte_length"])?;
                    capture_refusal(semantic_codec::validate_utf8_length(integer_usize(
                        field(value, "byte_length")?,
                        "UTF-8 byte length",
                    )?))
                }
                _ => return semantic(id, "refusal scalar codec"),
            }
        }
        "encode_row" => {
            exact_keys(
                object,
                &[
                    "id",
                    "kind",
                    "operation",
                    "producer",
                    "input",
                    "expected_code",
                ],
            )?;
            if string(object, "producer")? != "successful_event_batch_v1" {
                return semantic(id, "refusal row producer");
            }
            let value = as_object(input, "row refusal input")?;
            exact_keys(value, &["ordered_fields"])?;
            let fields = parse_named_stable(field(value, "ordered_fields")?)?;
            let refs = stable_refs(&fields);
            capture_refusal(
                semantic_codec::encode_historical_successful_event_v1_vector(0, "vector", &refs),
            )
        }
        "compose_family" => {
            if object.contains_key("producer") {
                exact_keys(
                    object,
                    &[
                        "id",
                        "kind",
                        "operation",
                        "producer",
                        "input",
                        "expected_code",
                    ],
                )?;
                if string(object, "producer")? != "successful_event_batch_v1" {
                    return semantic(id, "duplicate row producer");
                }
                let value = as_object(input, "duplicate row input")?;
                exact_keys(value, &["key_ids"])?;
                let keys = string_array(field(value, "key_ids")?, "row key ids")?;
                capture_refusal(semantic_codec::validate_duplicate_row_keys(&keys))
            } else {
                exact_keys(
                    object,
                    &["id", "kind", "operation", "input", "expected_code"],
                )?;
                let value = as_object(input, "empty family refusal")?;
                if value.contains_key("empty_proof") {
                    exact_keys(value, &["family", "rows", "empty_proof"])?;
                    ensure_empty_array(field(value, "rows")?, "empty rows")?;
                    if !field(value, "empty_proof")?.is_null() {
                        return semantic(id, "empty proof absence");
                    }
                    capture_refusal(semantic_codec::validate_empty_family(
                        string(value, "family")?,
                        None,
                    ))
                } else {
                    exact_keys(value, &["family", "rows", "proof_producer"])?;
                    ensure_empty_array(field(value, "rows")?, "empty rows")?;
                    capture_refusal(semantic_codec::validate_empty_family(
                        string(value, "family")?,
                        Some(string(value, "proof_producer")?),
                    ))
                }
            }
        }
        "decode_row" => {
            exact_keys(
                object,
                &["id", "kind", "operation", "input", "expected_code"],
            )?;
            let value = as_object(input, "decode row refusal")?;
            exact_keys(value, &["producer_tag_u8"])?;
            capture_refusal(semantic_codec::validate_producer_tag(integer_u8(
                field(value, "producer_tag_u8")?,
                "producer tag",
            )?))
        }
        "prepare_committed_tick" => {
            exact_keys(
                object,
                &["id", "kind", "operation", "input", "expected_code"],
            )?;
            let value = as_object(input, "prepare refusal")?;
            if value.contains_key("resolve_tick") {
                exact_keys(value, &["resolve_tick"])?;
                capture_refusal(semantic_codec::validate_resolve_tick(decimal_u64(
                    field(value, "resolve_tick")?,
                    "resolve tick",
                )?))
            } else {
                exact_keys(value, &["family", "row_payload"])?;
                if string(value, "family")? != "state" {
                    return semantic(id, "opaque payload family");
                }
                let payload = as_object(field(value, "row_payload")?, "opaque payload")?;
                exact_keys(payload, &["json"])?;
                let json = as_object(field(payload, "json")?, "opaque JSON")?;
                exact_keys(json, &["guessed"])?;
                if !boolean_value(field(json, "guessed")?, "guessed payload")? {
                    return semantic(id, "opaque payload witness");
                }
                capture_refusal::<Vec<u8>>(Err(semantic_codec::refuse_opaque_payload()))
            }
        }
        "select_restart_root" => {
            exact_keys(
                object,
                &["id", "kind", "operation", "input", "expected_code"],
            )?;
            let value = as_object(input, "restart refusal")?;
            exact_keys(value, &["completeness", "section_tags"])?;
            let tags = array(field(value, "section_tags")?, "section tags")?
                .iter()
                .map(|tag| integer_u8(tag, "section tag"))
                .collect::<Result<Vec<_>, _>>()?;
            capture_refusal(semantic_codec::validate_restart_root(
                string(value, "completeness")?,
                &tags,
            ))
        }
        "resolve_foundation" => {
            exact_keys(
                object,
                &["id", "kind", "operation", "input", "expected_code"],
            )?;
            let value = as_object(input, "foundation refusal")?;
            if value.contains_key("artifact_present") {
                exact_keys(value, &["logical_name", "artifact_present"])?;
                let _logical_name = string(value, "logical_name")?;
                capture_refusal(semantic_codec::validate_foundation_artifact(
                    boolean_value(field(value, "artifact_present")?, "artifact present")?,
                    None,
                    None,
                ))
            } else {
                exact_keys(value, &["expected_sha256", "actual_sha256"])?;
                capture_refusal(semantic_codec::validate_foundation_artifact(
                    true,
                    Some(digest32(string(value, "expected_sha256")?)?),
                    Some(digest32(string(value, "actual_sha256")?)?),
                ))
            }
        }
        _ => return semantic(id, "refusal operation"),
    }?;
    if actual.as_str() != string(object, "expected_code")? {
        return semantic(id, "refusal code");
    }
    Ok(())
}

fn capture_refusal<T>(
    result: Result<T, SemanticCodecErrorV1>,
) -> Result<SemanticRefusalCodeV1, RustPersistenceVectorErrorV1> {
    match result {
        Err(SemanticCodecErrorV1::Refusal(code)) => Ok(code),
        Err(_) => Err(RustPersistenceVectorErrorV1::Shape {
            field: "semantic refusal",
        }),
        Ok(value) => {
            drop(value);
            Err(RustPersistenceVectorErrorV1::Shape {
                field: "semantic refusal",
            })
        }
    }
}

fn parse_stable_bsl(value: &Value) -> Result<StableBslValueV1, RustPersistenceVectorErrorV1> {
    let object = as_object(value, "stable BSL input")?;
    let tag = string(object, "tag")?;
    match tag {
        "int_i64" => {
            exact_keys(object, &["tag", "value"])?;
            Ok(StableBslValueV1::Int(decimal_i64(
                field(object, "value")?,
                "BSL int",
            )?))
        }
        "currency_i128" => {
            exact_keys(object, &["tag", "micro_units"])?;
            Ok(StableBslValueV1::CurrencyMicroUnits(decimal_i128(
                field(object, "micro_units")?,
                "BSL currency",
            )?))
        }
        "real_f64_bits" => {
            exact_keys(object, &["tag", "value"])?;
            Ok(StableBslValueV1::RealBits(
                vector_f64(field(object, "value")?)?.to_bits(),
            ))
        }
        "ratio_f64_bits_with_optional_bounds" => {
            exact_keys(object, &["tag", "value", "floor", "cap"])?;
            Ok(StableBslValueV1::RatioBits {
                value: vector_f64(field(object, "value")?)?.to_bits(),
                floor: optional_f64(field(object, "floor")?)?.map(f64::to_bits),
                cap: optional_f64(field(object, "cap")?)?.map(f64::to_bits),
            })
        }
        "bool" => {
            exact_keys(object, &["tag", "value"])?;
            Ok(StableBslValueV1::Bool(boolean_value(
                field(object, "value")?,
                "BSL bool",
            )?))
        }
        "enum_type_and_member" => {
            exact_keys(object, &["tag", "enum_type", "member"])?;
            Ok(StableBslValueV1::Enum {
                enum_type: string(object, "enum_type")?.to_owned(),
                member: string(object, "member")?.to_owned(),
            })
        }
        "stable_node_key" => {
            exact_keys(object, &["tag", "scenario", "local_name"])?;
            Ok(StableBslValueV1::Node(StableElementKeyV1::Node {
                scenario: string(object, "scenario")?.to_owned(),
                local_name: string(object, "local_name")?.to_owned(),
            }))
        }
        "stable_hyperedge_key" => {
            exact_keys(object, &["tag", "scenario", "local_name"])?;
            Ok(StableBslValueV1::Hyperedge(StableElementKeyV1::Hyperedge {
                scenario: string(object, "scenario")?.to_owned(),
                local_name: string(object, "local_name")?.to_owned(),
            }))
        }
        "stable_edge_key" => {
            exact_keys(
                object,
                &[
                    "tag",
                    "scenario",
                    "edge_type",
                    "source_local_name",
                    "target_local_name",
                ],
            )?;
            Ok(StableBslValueV1::Edge(StableElementKeyV1::Edge {
                scenario: string(object, "scenario")?.to_owned(),
                edge_type: string(object, "edge_type")?.to_owned(),
                source_local_name: string(object, "source_local_name")?.to_owned(),
                target_local_name: string(object, "target_local_name")?.to_owned(),
            }))
        }
        _ => Err(RustPersistenceVectorErrorV1::Shape {
            field: "stable BSL tag",
        }),
    }
}

fn parse_stable_key(value: &Value) -> Result<StableElementKeyV1, RustPersistenceVectorErrorV1> {
    match parse_stable_bsl(value)? {
        StableBslValueV1::Node(key)
        | StableBslValueV1::Hyperedge(key)
        | StableBslValueV1::Edge(key) => Ok(key),
        _ => Err(RustPersistenceVectorErrorV1::Shape {
            field: "stable element key",
        }),
    }
}

fn parse_named_stable(
    value: &Value,
) -> Result<Vec<(String, StableBslValueV1)>, RustPersistenceVectorErrorV1> {
    array(value, "named stable fields")?
        .iter()
        .map(|item| {
            let item = as_object(item, "named stable field")?;
            exact_keys(item, &["name", "value"])?;
            Ok((
                string(item, "name")?.to_owned(),
                parse_stable_bsl(field(item, "value")?)?,
            ))
        })
        .collect()
}

fn stable_refs(values: &[(String, StableBslValueV1)]) -> Vec<(&str, &StableBslValueV1)> {
    values
        .iter()
        .map(|(name, value)| (name.as_str(), value))
        .collect()
}

fn as_object<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<&'a Map<String, Value>, RustPersistenceVectorErrorV1> {
    value
        .as_object()
        .ok_or(RustPersistenceVectorErrorV1::Shape { field })
}

fn array<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<&'a [Value], RustPersistenceVectorErrorV1> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or(RustPersistenceVectorErrorV1::Shape { field })
}

fn ensure_empty_array(
    value: &Value,
    field_name: &'static str,
) -> Result<(), RustPersistenceVectorErrorV1> {
    if array(value, field_name)?.is_empty() {
        Ok(())
    } else {
        Err(RustPersistenceVectorErrorV1::Shape { field: field_name })
    }
}

fn exact_keys(
    object: &Map<String, Value>,
    expected: &[&str],
) -> Result<(), RustPersistenceVectorErrorV1> {
    if object.len() == expected.len() && expected.iter().all(|key| object.contains_key(*key)) {
        Ok(())
    } else {
        Err(RustPersistenceVectorErrorV1::Shape {
            field: "exact object keys",
        })
    }
}

fn field<'a>(
    object: &'a Map<String, Value>,
    name: &'static str,
) -> Result<&'a Value, RustPersistenceVectorErrorV1> {
    object
        .get(name)
        .ok_or(RustPersistenceVectorErrorV1::Shape { field: name })
}

fn string<'a>(
    object: &'a Map<String, Value>,
    name: &'static str,
) -> Result<&'a str, RustPersistenceVectorErrorV1> {
    field(object, name)?
        .as_str()
        .ok_or(RustPersistenceVectorErrorV1::Shape { field: name })
}

fn optional_string<'a>(
    value: &'a Value,
    field_name: &'static str,
) -> Result<Option<&'a str>, RustPersistenceVectorErrorV1> {
    if value.is_null() {
        Ok(None)
    } else {
        value
            .as_str()
            .map(Some)
            .ok_or(RustPersistenceVectorErrorV1::Shape { field: field_name })
    }
}

fn boolean_value(
    value: &Value,
    field_name: &'static str,
) -> Result<bool, RustPersistenceVectorErrorV1> {
    value
        .as_bool()
        .ok_or(RustPersistenceVectorErrorV1::Shape { field: field_name })
}

fn integer_u64(
    value: &Value,
    field_name: &'static str,
) -> Result<u64, RustPersistenceVectorErrorV1> {
    value
        .as_u64()
        .ok_or(RustPersistenceVectorErrorV1::Shape { field: field_name })
}

fn integer_usize(
    value: &Value,
    field_name: &'static str,
) -> Result<usize, RustPersistenceVectorErrorV1> {
    usize::try_from(integer_u64(value, field_name)?)
        .map_err(|_| RustPersistenceVectorErrorV1::Shape { field: field_name })
}

fn integer_u8(value: &Value, field_name: &'static str) -> Result<u8, RustPersistenceVectorErrorV1> {
    u8::try_from(integer_u64(value, field_name)?)
        .map_err(|_| RustPersistenceVectorErrorV1::Shape { field: field_name })
}

fn integer_u16(
    value: &Value,
    field_name: &'static str,
) -> Result<u16, RustPersistenceVectorErrorV1> {
    u16::try_from(integer_u64(value, field_name)?)
        .map_err(|_| RustPersistenceVectorErrorV1::Shape { field: field_name })
}

fn integer_u32(
    value: &Value,
    field_name: &'static str,
) -> Result<u32, RustPersistenceVectorErrorV1> {
    u32::try_from(integer_u64(value, field_name)?)
        .map_err(|_| RustPersistenceVectorErrorV1::Shape { field: field_name })
}

fn decimal_u64(
    value: &Value,
    field_name: &'static str,
) -> Result<u64, RustPersistenceVectorErrorV1> {
    if let Some(value) = value.as_u64() {
        return Ok(value);
    }
    value
        .as_str()
        .and_then(|value| value.parse().ok())
        .ok_or(RustPersistenceVectorErrorV1::Shape { field: field_name })
}

fn decimal_i64(
    value: &Value,
    field_name: &'static str,
) -> Result<i64, RustPersistenceVectorErrorV1> {
    if let Some(value) = value.as_i64() {
        return Ok(value);
    }
    value
        .as_str()
        .and_then(|value| value.parse().ok())
        .ok_or(RustPersistenceVectorErrorV1::Shape { field: field_name })
}

fn decimal_i128(
    value: &Value,
    field_name: &'static str,
) -> Result<i128, RustPersistenceVectorErrorV1> {
    if let Some(value) = value.as_i64() {
        return Ok(i128::from(value));
    }
    if let Some(value) = value.as_u64() {
        return Ok(i128::from(value));
    }
    value
        .as_str()
        .and_then(|value| value.parse().ok())
        .ok_or(RustPersistenceVectorErrorV1::Shape { field: field_name })
}

fn vector_f64(value: &Value) -> Result<f64, RustPersistenceVectorErrorV1> {
    let value = value.as_str().ok_or(RustPersistenceVectorErrorV1::Shape {
        field: "binary64 value",
    })?;
    match value {
        "negative_zero" => Ok(-0.0),
        "nan" => Ok(f64::NAN),
        "positive_infinity" => Ok(f64::INFINITY),
        "negative_infinity" => Ok(f64::NEG_INFINITY),
        value => value
            .parse()
            .map_err(|_| RustPersistenceVectorErrorV1::Shape {
                field: "binary64 value",
            }),
    }
}

fn optional_f64(value: &Value) -> Result<Option<f64>, RustPersistenceVectorErrorV1> {
    if value.is_null() {
        Ok(None)
    } else {
        vector_f64(value).map(Some)
    }
}

fn string_array(
    value: &Value,
    field_name: &'static str,
) -> Result<Vec<String>, RustPersistenceVectorErrorV1> {
    array(value, field_name)?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or(RustPersistenceVectorErrorV1::Shape { field: field_name })
        })
        .collect()
}

fn hex_bytes(
    value: &str,
    field_name: &'static str,
) -> Result<Vec<u8>, RustPersistenceVectorErrorV1> {
    if !value.len().is_multiple_of(2)
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
    {
        return Err(RustPersistenceVectorErrorV1::Shape { field: field_name });
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = hex_nibble(pair[0])
                .ok_or(RustPersistenceVectorErrorV1::Shape { field: field_name })?;
            let low = hex_nibble(pair[1])
                .ok_or(RustPersistenceVectorErrorV1::Shape { field: field_name })?;
            Ok(high << 4 | low)
        })
        .collect()
}

fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}

fn digest32(value: &str) -> Result<[u8; 32], RustPersistenceVectorErrorV1> {
    hex_bytes(value, "SHA-256")?
        .try_into()
        .map_err(|_| RustPersistenceVectorErrorV1::Shape { field: "SHA-256" })
}

fn compare_hex(
    id: &str,
    actual: &[u8],
    expected: &str,
    field_name: &'static str,
) -> Result<(), RustPersistenceVectorErrorV1> {
    if actual == hex_bytes(expected, field_name)? {
        Ok(())
    } else {
        semantic(id, field_name)
    }
}

fn compare_digest(
    id: &str,
    bytes: &[u8],
    expected: &str,
) -> Result<(), RustPersistenceVectorErrorV1> {
    if semantic_codec::digest(bytes) == digest32(expected)? {
        Ok(())
    } else {
        semantic(id, "SHA-256")
    }
}

fn semantic_result<T>(
    id: &str,
    result: Result<T, SemanticCodecErrorV1>,
) -> Result<T, RustPersistenceVectorErrorV1> {
    result.map_err(|error| match error {
        SemanticCodecErrorV1::Refusal(_) => RustPersistenceVectorErrorV1::Semantic {
            id: id.into(),
            field: "unexpected semantic refusal",
        },
        SemanticCodecErrorV1::Invalid(field_name) => RustPersistenceVectorErrorV1::Semantic {
            id: id.into(),
            field: field_name,
        },
        SemanticCodecErrorV1::CapacityOverflow { field }
        | SemanticCodecErrorV1::IntegerConversion { field, .. }
        | SemanticCodecErrorV1::ByteLimit { field, .. }
        | SemanticCodecErrorV1::Allocation { field, .. } => {
            RustPersistenceVectorErrorV1::Semantic {
                id: id.into(),
                field,
            }
        }
    })
}

fn semantic<T>(id: &str, field_name: &'static str) -> Result<T, RustPersistenceVectorErrorV1> {
    Err(RustPersistenceVectorErrorV1::Semantic {
        id: id.into(),
        field: field_name,
    })
}
