//! Closed, database-free semantic codec shared by the live Rust persistence
//! writer, restart reconstruction, and exact contract vectors.

use crate::committed_tick_envelope::{
    compose_row_families, CommittedTickEnvelopeError, CommittedTickRow, CommittedTickRowFamilies,
    CommittedTickRowFamily,
};
use crate::identity::CampaignId;
use babylon_bsl::identity_codec::{
    canonical_f64_bits, encode_stable_bsl_value, IdentityCodecError, StableBslValue,
};
use babylon_graph::stable_element::StableElementKey;
use babylon_kernel::{content_digest::sha256_of, H3CellId};
use std::collections::TryReserveError;

const LAYOUT: u32 = 1;
const ROW_KEY_DOMAIN: &[u8] = b"babylon.committed-tick-row-key.v1\0";
const ROW_PAYLOAD_DOMAIN: &[u8] = b"babylon.committed-tick-row-payload.v1\0";
const FOUNDATION_CONTENT_DOMAIN: &[u8] = b"babylon.campaign-foundation-content.v2\0";
const MAX_FOUNDATION_SOURCE: usize = 1_048_576;
const CHECKPOINT_DOMAIN: &[u8] = b"babylon.full-checkpoint-manifest.v1\0";
const EMPTY_PROOF_DOMAIN: &[u8] = b"babylon.semantic-empty-proof.v1\0";
const MAX_UTF8_BYTES: usize = 65_535;
const MAX_BYTES: usize = 67_108_864;
const MAX_ITEMS: usize = 1_048_576;

/// One enum-ordered branch in a canonical choice-receipt semantic row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChoiceReceiptSemanticBranch {
    pub(crate) outcome_member: String,
    pub(crate) mass_nanounits: u64,
    pub(crate) ticket_start: u128,
    pub(crate) ticket_end_exclusive: u128,
    pub(crate) ticket_count: u128,
}

/// Exact aggregate input to the V2 envelope's `ChoiceReceipt` family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChoiceReceiptSemanticRow {
    pub(crate) encounter_ordinal: u32,
    pub(crate) rule_id: String,
    pub(crate) sample: String,
    pub(crate) slot: u32,
    pub(crate) outcome_enum: String,
    pub(crate) stable_carrier: StableElementKey,
    pub(crate) active_elements: Vec<StableElementKey>,
    pub(crate) branches: Vec<ChoiceReceiptSemanticBranch>,
    pub(crate) draw_ticket: u64,
    pub(crate) selected_outcome: String,
    pub(crate) allocation_digest: [u8; 32],
    pub(crate) instance_digest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SemanticRefusalCode {
    NonfiniteF64,
    InvalidH3CellId,
    UnknownClosedTag,
    RuntimeGraphHandle,
    NoncanonicalFieldOrder,
    DuplicateRowKey,
    UnknownProducerTag,
    SyntheticTickZero,
    ResolveTickSqlRange,
    MissingEmptyProof,
    ForeignEmptyProof,
    IncompleteFullCheckpoint,
    DeltaCheckpointNotRestartRoot,
    MissingFoundationArtifact,
    FoundationArtifactDigestMismatch,
    FieldByteBound,
    OpaqueSemanticPayload,
}

impl SemanticRefusalCode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::NonfiniteF64 => "nonfinite_f64",
            Self::InvalidH3CellId => "invalid_h3_cell_id",
            Self::UnknownClosedTag => "unknown_closed_tag",
            Self::RuntimeGraphHandle => "runtime_graph_handle",
            Self::NoncanonicalFieldOrder => "noncanonical_field_order",
            Self::DuplicateRowKey => "duplicate_row_key",
            Self::UnknownProducerTag => "unknown_producer_tag",
            Self::SyntheticTickZero => "synthetic_tick_zero",
            Self::ResolveTickSqlRange => "resolve_tick_sql_range",
            Self::MissingEmptyProof => "missing_empty_proof",
            Self::ForeignEmptyProof => "foreign_empty_proof",
            Self::IncompleteFullCheckpoint => "incomplete_full_checkpoint",
            Self::DeltaCheckpointNotRestartRoot => "delta_checkpoint_not_restart_root",
            Self::MissingFoundationArtifact => "missing_foundation_artifact",
            Self::FoundationArtifactDigestMismatch => "foundation_artifact_digest_mismatch",
            Self::FieldByteBound => "field_byte_bound",
            Self::OpaqueSemanticPayload => "opaque_semantic_payload",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SemanticCodecError {
    Refusal(SemanticRefusalCode),
    Invalid(&'static str),
    CapacityOverflow {
        field: &'static str,
    },
    IntegerConversion {
        field: &'static str,
        value: usize,
    },
    ByteLimit {
        field: &'static str,
        actual: usize,
        maximum: usize,
    },
    Allocation {
        field: &'static str,
        requested: usize,
    },
}

impl From<IdentityCodecError> for SemanticCodecError {
    fn from(value: IdentityCodecError) -> Self {
        match value {
            IdentityCodecError::NonFiniteValue => Self::Refusal(SemanticRefusalCode::NonfiniteF64),
            IdentityCodecError::CapacityOverflow { field } => Self::CapacityOverflow { field },
            IdentityCodecError::IntegerConversion { field, value } => {
                Self::IntegerConversion { field, value }
            }
            IdentityCodecError::ByteLimit {
                field,
                actual,
                maximum,
            } => Self::ByteLimit {
                field,
                actual,
                maximum,
            },
            IdentityCodecError::Allocation { field, requested } => {
                Self::Allocation { field, requested }
            }
            _ => Self::Invalid("stable BSL value"),
        }
    }
}

type ReserveSemanticBytes = fn(&mut Vec<u8>, usize) -> Result<(), ()>;

fn reserve_semantic_bytes(bytes: &mut Vec<u8>, additional: usize) -> Result<(), ()> {
    bytes
        .try_reserve_exact(additional)
        .map_err(|_: TryReserveError| ())
}

pub(crate) struct SemanticWriter {
    field: &'static str,
    maximum: usize,
    bytes: Vec<u8>,
    reserve: ReserveSemanticBytes,
}

impl SemanticWriter {
    pub(crate) fn new(field: &'static str, maximum: usize) -> Self {
        Self::with_reserver(field, maximum, reserve_semantic_bytes)
    }

    fn with_reserver(field: &'static str, maximum: usize, reserve: ReserveSemanticBytes) -> Self {
        Self {
            field,
            maximum,
            bytes: Vec::new(),
            reserve,
        }
    }

    pub(crate) fn write_byte(&mut self, value: u8) -> Result<(), SemanticCodecError> {
        self.write_all(&[value])
    }

    pub(crate) fn write_all(&mut self, value: &[u8]) -> Result<(), SemanticCodecError> {
        let target =
            checked_semantic_capacity(self.bytes.len(), value.len(), self.field, self.maximum)?;
        (self.reserve)(&mut self.bytes, value.len()).map_err(|()| {
            SemanticCodecError::Allocation {
                field: self.field,
                requested: target,
            }
        })?;
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

pub(crate) fn checked_semantic_capacity(
    current: usize,
    additional: usize,
    field: &'static str,
    maximum: usize,
) -> Result<usize, SemanticCodecError> {
    let actual = current
        .checked_add(additional)
        .ok_or(SemanticCodecError::CapacityOverflow { field })?;
    if actual > maximum {
        return Err(SemanticCodecError::ByteLimit {
            field,
            actual,
            maximum,
        });
    }
    Ok(actual)
}

fn encode_fixed(value: &[u8]) -> Result<Vec<u8>, SemanticCodecError> {
    let mut output = SemanticWriter::new("fixed semantic scalar", value.len());
    output.write_all(value)?;
    Ok(output.finish())
}

pub(crate) fn encode_bool(value: bool) -> Result<Vec<u8>, SemanticCodecError> {
    encode_fixed(&[u8::from(value)])
}
pub(crate) fn encode_u64(value: u64) -> Result<Vec<u8>, SemanticCodecError> {
    encode_fixed(&value.to_be_bytes())
}
pub(crate) fn encode_i64(value: i64) -> Result<Vec<u8>, SemanticCodecError> {
    encode_fixed(&value.to_be_bytes())
}
pub(crate) fn encode_i128(value: i128) -> Result<Vec<u8>, SemanticCodecError> {
    encode_fixed(&value.to_be_bytes())
}

pub(crate) fn encode_f64(value: f64) -> Result<Vec<u8>, SemanticCodecError> {
    encode_fixed(&canonical_f64_bits(value)?.to_be_bytes())
}

pub(crate) fn encode_h3(raw: i128) -> Result<Vec<u8>, SemanticCodecError> {
    let raw = i64::try_from(raw)
        .map_err(|_| SemanticCodecError::Refusal(SemanticRefusalCode::InvalidH3CellId))?;
    let cell = H3CellId::try_from(raw)
        .map_err(|_| SemanticCodecError::Refusal(SemanticRefusalCode::InvalidH3CellId))?;
    encode_fixed(&cell.to_be_bytes())
}

pub(crate) fn encode_optional_utf8(value: Option<&str>) -> Result<Vec<u8>, SemanticCodecError> {
    let mut output = SemanticWriter::new("optional UTF-8", MAX_BYTES);
    match value {
        None => output.write_byte(0)?,
        Some(value) => {
            output.write_byte(1)?;
            append_utf8(&mut output, value)?;
        }
    }
    Ok(output.finish())
}

pub(crate) fn encode_stable_bsl(value: &StableBslValue) -> Result<Vec<u8>, SemanticCodecError> {
    let mut output = Vec::new();
    encode_stable_bsl_value(value, &mut output)?;
    let mut checked = SemanticWriter::new("stable BSL value", MAX_BYTES);
    checked.write_all(&output)?;
    Ok(checked.finish())
}

pub(crate) fn encode_stable_graph_node(
    local_name: &str,
    node_type: &str,
) -> Result<CommittedTickRow, SemanticCodecError> {
    compose_row(
        CommittedTickRowFamily::Graph,
        1,
        |key| append_utf8(key, local_name),
        |payload| append_utf8(payload, node_type),
    )
}
pub(crate) fn encode_stable_graph_node_f64(
    local_name: &str,
    qname: &str,
    value: f64,
) -> Result<CommittedTickRow, SemanticCodecError> {
    compose_row(
        CommittedTickRowFamily::Graph,
        2,
        |key| {
            append_utf8(key, local_name)?;
            append_utf8(key, qname)
        },
        |payload| append_f64(payload, value),
    )
}
pub(crate) fn encode_stable_graph_edge(
    edge_type: &str,
    source: &str,
    target: &str,
    strength: f64,
) -> Result<CommittedTickRow, SemanticCodecError> {
    compose_row(
        CommittedTickRowFamily::Graph,
        3,
        |key| {
            append_utf8(key, edge_type)?;
            append_utf8(key, source)?;
            append_utf8(key, target)
        },
        |payload| append_f64(payload, strength),
    )
}
pub(crate) fn encode_stable_graph_hyperedge(
    local_name: &str,
    hyperedge_type: &str,
    ordered_members: &[String],
) -> Result<CommittedTickRow, SemanticCodecError> {
    compose_row(
        CommittedTickRowFamily::Graph,
        4,
        |key| append_utf8(key, local_name),
        |payload| {
            append_utf8(payload, hyperedge_type)?;
            append_ordered_utf8(payload, ordered_members)
        },
    )
}
pub(crate) fn encode_stable_graph_edge_f64(
    edge_type: &str,
    source: &str,
    target: &str,
    qname: &str,
    value: f64,
) -> Result<CommittedTickRow, SemanticCodecError> {
    compose_row(
        CommittedTickRowFamily::Graph,
        5,
        |key| {
            append_utf8(key, edge_type)?;
            append_utf8(key, source)?;
            append_utf8(key, target)?;
            append_utf8(key, qname)
        },
        |payload| append_f64(payload, value),
    )
}
pub(crate) fn encode_stable_graph_node_currency(
    local_name: &str,
    qname: &str,
    micro_units: i128,
) -> Result<CommittedTickRow, SemanticCodecError> {
    compose_row(
        CommittedTickRowFamily::Graph,
        6,
        |key| {
            append_utf8(key, local_name)?;
            append_utf8(key, qname)
        },
        |payload| {
            payload.write_all(&micro_units.to_be_bytes())?;
            Ok(())
        },
    )
}
pub(crate) fn encode_stable_graph_hyperedge_f64(
    local_name: &str,
    qname: &str,
    value: f64,
) -> Result<CommittedTickRow, SemanticCodecError> {
    compose_row(
        CommittedTickRowFamily::Graph,
        7,
        |key| {
            append_utf8(key, local_name)?;
            append_utf8(key, qname)
        },
        |payload| append_f64(payload, value),
    )
}
pub(crate) fn encode_world_register(
    register_name: &str,
    value: &StableBslValue,
) -> Result<CommittedTickRow, SemanticCodecError> {
    compose_row(
        CommittedTickRowFamily::State,
        1,
        |key| append_utf8(key, register_name),
        |payload| {
            payload.write_all(&encode_stable_bsl(value)?)?;
            Ok(())
        },
    )
}
pub(crate) fn encode_territory_state(
    territory_id: &StableElementKey,
    ordered_fields: &[(&str, &StableBslValue)],
) -> Result<CommittedTickRow, SemanticCodecError> {
    compose_row(
        CommittedTickRowFamily::State,
        2,
        |key| append_stable_key(key, territory_id),
        |payload| append_named_stable(payload, ordered_fields),
    )
}
pub(crate) fn encode_dynamic_hex_state(
    cell_id: u64,
    values: &[f64; 9],
) -> Result<CommittedTickRow, SemanticCodecError> {
    compose_row(
        CommittedTickRowFamily::State,
        3,
        |key| append_h3(key, cell_id),
        |payload| {
            for value in values {
                append_f64(payload, *value)?;
            }
            Ok(())
        },
    )
}
pub(crate) fn encode_organization_state(
    organization_id: &StableElementKey,
    organization_kind: &StableBslValue,
    ordered_territory_ids: &[StableElementKey],
    ordered_fields: &[(&str, &StableBslValue)],
) -> Result<CommittedTickRow, SemanticCodecError> {
    compose_row(
        CommittedTickRowFamily::State,
        8,
        |key| append_stable_key(key, organization_id),
        |payload| {
            payload.write_all(&encode_stable_bsl(organization_kind)?)?;
            append_ordered_stable_keys(payload, ordered_territory_ids)?;
            append_named_stable(payload, ordered_fields)
        },
    )
}
pub(crate) fn encode_successful_event(
    ordinal: u32,
    emitting_rule: &str,
    choice_receipt_ordinal: Option<u32>,
    event_type: &str,
    ordered_fields: &[(&str, &StableBslValue)],
) -> Result<CommittedTickRow, SemanticCodecError> {
    compose_row(
        CommittedTickRowFamily::Event,
        1,
        |key| {
            key.write_all(&ordinal.to_be_bytes())?;
            Ok(())
        },
        |payload| {
            append_utf8(payload, emitting_rule)?;
            match choice_receipt_ordinal {
                None => payload.write_byte(0)?,
                Some(ordinal) => {
                    payload.write_byte(1)?;
                    payload.write_all(&ordinal.to_be_bytes())?;
                }
            }
            append_utf8(payload, event_type)?;
            append_named_stable(payload, ordered_fields)
        },
    )
}

/// Reconstruct the frozen PER-281 event-row bytes for its offline contract
/// vectors. The live writer and restart path call only
/// [`encode_successful_event`]; they have no V1 event-row read/write path.
pub(crate) fn encode_historical_successful_event_vector(
    ordinal: u32,
    event_type: &str,
    ordered_fields: &[(&str, &StableBslValue)],
) -> Result<CommittedTickRow, SemanticCodecError> {
    compose_row(
        CommittedTickRowFamily::Event,
        1,
        |key| {
            key.write_all(&ordinal.to_be_bytes())?;
            Ok(())
        },
        |payload| {
            append_utf8(payload, event_type)?;
            append_named_stable(payload, ordered_fields)
        },
    )
}

pub(crate) fn encode_choice_receipt(
    receipt: &ChoiceReceiptSemanticRow,
) -> Result<CommittedTickRow, SemanticCodecError> {
    compose_row(
        CommittedTickRowFamily::ChoiceReceipt,
        1,
        |key| key.write_all(&receipt.encounter_ordinal.to_be_bytes()),
        |payload| {
            append_utf8(payload, &receipt.rule_id)?;
            append_utf8(payload, &receipt.sample)?;
            payload.write_all(&receipt.slot.to_be_bytes())?;
            append_utf8(payload, &receipt.outcome_enum)?;
            append_stable_key(payload, &receipt.stable_carrier)?;
            append_stable_key_sequence(payload, &receipt.active_elements)?;
            payload.write_all(
                &checked_u32(receipt.branches.len(), "choice receipt branch count")?.to_be_bytes(),
            )?;
            for branch in &receipt.branches {
                append_utf8(payload, &branch.outcome_member)?;
                payload.write_all(&branch.mass_nanounits.to_be_bytes())?;
                payload.write_all(&branch.ticket_start.to_be_bytes())?;
                payload.write_all(&branch.ticket_end_exclusive.to_be_bytes())?;
                payload.write_all(&branch.ticket_count.to_be_bytes())?;
            }
            payload.write_all(&receipt.draw_ticket.to_be_bytes())?;
            append_utf8(payload, &receipt.selected_outcome)?;
            payload.write_all(&receipt.allocation_digest)?;
            payload.write_all(&receipt.instance_digest)
        },
    )
}
pub(crate) fn encode_checkpoint_row(
    section_tag: u8,
    ordinal: u32,
    completeness_tag: u8,
    exact_section_bytes: &[u8],
) -> Result<CommittedTickRow, SemanticCodecError> {
    validate_closed_checkpoint_tag(section_tag)?;
    validate_completeness_tag(completeness_tag)?;
    compose_row(
        CommittedTickRowFamily::Checkpoint,
        1,
        |key| {
            key.write_byte(section_tag)?;
            key.write_all(&ordinal.to_be_bytes())?;
            Ok(())
        },
        |payload| {
            payload.write_byte(completeness_tag)?;
            append_bytes(payload, exact_section_bytes)
        },
    )
}
pub(crate) fn encode_archive_dirty_receipt(
    tick_content_hash: &[u8; 32],
) -> Result<CommittedTickRow, SemanticCodecError> {
    let key = row_prefix(
        ROW_KEY_DOMAIN,
        CommittedTickRowFamily::ArchiveDirtyReceipt,
        1,
    )?;
    let mut payload = SemanticWriter::new("archive dirty receipt payload", 32);
    payload.write_all(tick_content_hash)?;
    CommittedTickRow::compose(key.finish(), payload.finish())
        .map_err(|_| SemanticCodecError::Invalid("archive dirty receipt row"))
}

/// Current content encoding with bounded source fields.
pub(crate) fn encode_foundation_content(
    scenario_source: &str,
    prelude_source: Option<&str>,
    rule_source: &str,
    defines: &[u8],
    reference_manifest: &[u8],
) -> Result<Vec<u8>, SemanticCodecError> {
    foundation_content_length(
        scenario_source.len(),
        prelude_source.map(str::len),
        rule_source.len(),
        defines.len(),
        reference_manifest.len(),
    )?;
    let mut output = SemanticWriter::new("foundation content V2", MAX_BYTES);
    output.write_all(FOUNDATION_CONTENT_DOMAIN)?;
    output.write_all(&2_u32.to_be_bytes())?;
    output.write_byte(1)?;
    append_foundation_source(&mut output, scenario_source)?;
    output.write_byte(2)?;
    match prelude_source {
        None => output.write_byte(0)?,
        Some(source) => {
            output.write_byte(1)?;
            append_foundation_source(&mut output, source)?;
        }
    }
    output.write_byte(3)?;
    append_foundation_source(&mut output, rule_source)?;
    output.write_byte(4)?;
    append_bytes(&mut output, defines)?;
    output.write_byte(5)?;
    append_bytes(&mut output, reference_manifest)?;
    Ok(output.finish())
}

fn foundation_content_length(
    scenario: usize,
    prelude: Option<usize>,
    rules: usize,
    defines: usize,
    reference: usize,
) -> Result<usize, SemanticCodecError> {
    if [Some(scenario), prelude, Some(rules)]
        .into_iter()
        .flatten()
        .any(|length| length > MAX_FOUNDATION_SOURCE)
        || defines > MAX_BYTES
        || reference > MAX_BYTES
    {
        return Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::FieldByteBound,
        ));
    }
    // Domain, version, five tags, four required length words and one option tag.
    let mut length = FOUNDATION_CONTENT_DOMAIN.len() + 4 + 5 + 4 * 4 + 1;
    for field in [
        scenario,
        prelude.unwrap_or(0),
        rules,
        defines,
        reference,
        if prelude.is_some() { 4 } else { 0 },
    ] {
        length = length
            .checked_add(field)
            .ok_or(SemanticCodecError::CapacityOverflow {
                field: "foundation content V2",
            })?;
    }
    if length > MAX_BYTES {
        return Err(SemanticCodecError::ByteLimit {
            field: "foundation content V2",
            actual: length,
            maximum: MAX_BYTES,
        });
    }
    Ok(length)
}

fn append_foundation_source(
    output: &mut SemanticWriter,
    source: &str,
) -> Result<(), SemanticCodecError> {
    if source.len() > MAX_FOUNDATION_SOURCE || source.as_bytes().contains(&0) {
        return Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::FieldByteBound,
        ));
    }
    output.write_all(&checked_u32(source.len(), "V2 source byte length")?.to_be_bytes())?;
    output.write_all(source.as_bytes())
}

#[cfg(test)]
mod foundation_content_bounds_tests {
    use super::*;

    #[test]
    fn complete_bundle_includes_framing_and_enforces_exact_aggregate_bound() {
        let overhead = foundation_content_length(0, None, 0, 0, 0).unwrap();
        assert_eq!(
            foundation_content_length(0, None, 0, MAX_BYTES - overhead, 0),
            Ok(MAX_BYTES)
        );
        assert!(
            matches!(foundation_content_length(0, None, 0, MAX_BYTES - overhead, 1),
            Err(SemanticCodecError::ByteLimit { actual, maximum: MAX_BYTES, .. }) if actual == MAX_BYTES + 1)
        );
        assert_eq!(
            foundation_content_length(0, Some(0), 0, 0, 0),
            Ok(overhead + 4)
        );
        let encoded =
            encode_foundation_content("scenario", Some("prelude"), "", b"defines", b"ref").unwrap();
        assert_eq!(
            encoded.len(),
            foundation_content_length(8, Some(7), 0, 7, 3).unwrap()
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_foundation(
    stable_graph: &[u8],
    world_registers: &[u8],
    resolver_manifest: &[u8],
    prepared_environment: &[u8],
    replay_session_identity: &str,
    rng_seed: i64,
    defines_hash: &[u8; 32],
    rules_hash: &[u8; 32],
    reference_digest: &[u8; 32],
    content: &[u8],
) -> Result<Vec<u8>, SemanticCodecError> {
    let mut output = SemanticWriter::new("campaign foundation", MAX_BYTES);
    append_bytes(&mut output, stable_graph)?;
    append_bytes(&mut output, world_registers)?;
    append_bytes(&mut output, resolver_manifest)?;
    append_bytes(&mut output, prepared_environment)?;
    append_utf8(&mut output, replay_session_identity)?;
    output.write_all(&rng_seed.to_be_bytes())?;
    output.write_all(defines_hash)?;
    output.write_all(rules_hash)?;
    output.write_all(reference_digest)?;
    output.write_all(content)?;
    Ok(output.finish())
}

pub(crate) fn encode_full_checkpoint(
    campaign_id: CampaignId,
    resolve_tick: u64,
    sections: &[(u8, u32, [u8; 32])],
) -> Result<Vec<u8>, SemanticCodecError> {
    validate_resolve_tick(resolve_tick)?;
    validate_full_sections(sections)?;
    let mut output = SemanticWriter::new("full checkpoint", MAX_BYTES);
    output.write_all(CHECKPOINT_DOMAIN)?;
    output.write_all(&LAYOUT.to_be_bytes())?;
    output.write_byte(1)?;
    output.write_all(campaign_id.canonical_bytes())?;
    output.write_all(&resolve_tick.to_be_bytes())?;
    let section_count =
        u16::try_from(sections.len()).map_err(|_| SemanticCodecError::IntegerConversion {
            field: "checkpoint section count",
            value: sections.len(),
        })?;
    output.write_all(&section_count.to_be_bytes())?;
    for (tag, row_count, sha256) in sections {
        output.write_byte(*tag)?;
        output.write_all(&row_count.to_be_bytes())?;
        output.write_all(sha256)?;
    }
    Ok(output.finish())
}

pub(crate) fn encode_empty_proof(
    producer_tag: u8,
    source_count: u32,
    source_digest: [u8; 32],
) -> Result<Vec<u8>, SemanticCodecError> {
    validate_producer_tag(producer_tag)?;
    if source_count != 0 {
        return Err(SemanticCodecError::Invalid("nonempty source proof"));
    }
    let mut output = SemanticWriter::new("semantic empty proof", MAX_BYTES);
    output.write_all(EMPTY_PROOF_DOMAIN)?;
    output.write_all(&LAYOUT.to_be_bytes())?;
    output.write_byte(producer_tag)?;
    output.write_all(&source_count.to_be_bytes())?;
    output.write_all(&source_digest)?;
    Ok(output.finish())
}

pub(crate) fn validate_duplicate_row_keys(key_ids: &[String]) -> Result<(), SemanticCodecError> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(key_ids.len())
        .map_err(|_: TryReserveError| SemanticCodecError::Allocation {
            field: "duplicate row fixture",
            requested: key_ids.len(),
        })?;
    for key in key_ids {
        let mut key_writer = SemanticWriter::new("duplicate row fixture key", MAX_BYTES);
        key_writer.write_all(key.as_bytes())?;
        let payload_writer = SemanticWriter::new("duplicate row fixture payload", MAX_BYTES);
        rows.push(
            CommittedTickRow::compose(key_writer.finish(), payload_writer.finish())
                .map_err(|_| SemanticCodecError::Invalid("duplicate row fixture"))?,
        );
    }
    let input = CommittedTickRowFamilies {
        graph: Vec::new(),
        state: Vec::new(),
        event: rows,
        choice_receipt: Vec::new(),
        checkpoint: Vec::new(),
        archive_dirty_receipt: encode_archive_dirty_receipt(&[0; 32])?,
    };
    match compose_row_families(input) {
        Err(CommittedTickEnvelopeError::DuplicateRowKey { .. }) => Err(
            SemanticCodecError::Refusal(SemanticRefusalCode::DuplicateRowKey),
        ),
        Ok(_) => Ok(()),
        Err(_) => Err(SemanticCodecError::Invalid("duplicate row rule")),
    }
}

pub(crate) fn validate_producer_tag(tag: u8) -> Result<(), SemanticCodecError> {
    if matches!(tag, 1 | 16 | 24 | 32 | 96 | 112) {
        Ok(())
    } else {
        Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::UnknownProducerTag,
        ))
    }
}

pub(crate) fn validate_resolve_tick(tick: u64) -> Result<(), SemanticCodecError> {
    if tick == 0 {
        Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::SyntheticTickZero,
        ))
    } else if tick > i64::MAX as u64 {
        Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::ResolveTickSqlRange,
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn validate_empty_family(
    family: &str,
    proof_producer: Option<&str>,
) -> Result<(), SemanticCodecError> {
    let expected = match family {
        "event" => "successful_event_batch_v2",
        "choice_receipt" => "choice_receipt_batch_v1",
        _ => return Err(SemanticCodecError::Invalid("empty family")),
    };
    match proof_producer {
        None => Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::MissingEmptyProof,
        )),
        Some(actual) if actual != expected => Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::ForeignEmptyProof,
        )),
        Some(_) => Ok(()),
    }
}

pub(crate) fn validate_restart_root(
    completeness: &str,
    section_tags: &[u8],
) -> Result<(), SemanticCodecError> {
    if completeness == "delta" {
        return Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::DeltaCheckpointNotRestartRoot,
        ));
    }
    if completeness != "full" || section_tags != [1, 2, 3, 4, 5, 6, 7, 8, 9] {
        return Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::IncompleteFullCheckpoint,
        ));
    }
    Ok(())
}

pub(crate) fn validate_foundation_artifact(
    present: bool,
    expected: Option<[u8; 32]>,
    actual: Option<[u8; 32]>,
) -> Result<(), SemanticCodecError> {
    if !present {
        return Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::MissingFoundationArtifact,
        ));
    }
    if expected != actual {
        return Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::FoundationArtifactDigestMismatch,
        ));
    }
    Ok(())
}

pub(crate) fn refuse_runtime_graph_handle() -> SemanticCodecError {
    SemanticCodecError::Refusal(SemanticRefusalCode::RuntimeGraphHandle)
}

pub(crate) fn refuse_unknown_closed_tag() -> SemanticCodecError {
    SemanticCodecError::Refusal(SemanticRefusalCode::UnknownClosedTag)
}

pub(crate) fn refuse_opaque_payload() -> SemanticCodecError {
    SemanticCodecError::Refusal(SemanticRefusalCode::OpaqueSemanticPayload)
}

pub(crate) fn validate_utf8_length(length: usize) -> Result<(), SemanticCodecError> {
    if length > MAX_UTF8_BYTES {
        Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::FieldByteBound,
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn digest(bytes: &[u8]) -> [u8; 32] {
    sha256_of(bytes)
}

fn row_prefix(
    domain: &[u8],
    family: CommittedTickRowFamily,
    tag: u8,
) -> Result<SemanticWriter, SemanticCodecError> {
    let mut output = SemanticWriter::new("semantic row", MAX_BYTES);
    output.write_all(domain)?;
    output.write_all(&LAYOUT.to_be_bytes())?;
    output.write_byte(family.tag())?;
    output.write_byte(producer_tag(family))?;
    output.write_byte(tag)?;
    Ok(output)
}

const fn producer_tag(family: CommittedTickRowFamily) -> u8 {
    match family {
        CommittedTickRowFamily::Graph => 1,
        CommittedTickRowFamily::State => 16,
        CommittedTickRowFamily::Event => 32,
        CommittedTickRowFamily::ChoiceReceipt => 24,
        CommittedTickRowFamily::Checkpoint => 96,
        CommittedTickRowFamily::ArchiveDirtyReceipt => 112,
    }
}

fn append_utf8(output: &mut SemanticWriter, value: &str) -> Result<(), SemanticCodecError> {
    validate_utf8_length(value.len())?;
    if value.as_bytes().contains(&0) {
        return Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::FieldByteBound,
        ));
    }
    output.write_all(&checked_u32(value.len(), "UTF-8 byte length")?.to_be_bytes())?;
    output.write_all(value.as_bytes())?;
    Ok(())
}

fn append_bytes(output: &mut SemanticWriter, value: &[u8]) -> Result<(), SemanticCodecError> {
    if value.len() > MAX_BYTES {
        return Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::FieldByteBound,
        ));
    }
    output.write_all(&checked_u32(value.len(), "byte length")?.to_be_bytes())?;
    output.write_all(value)?;
    Ok(())
}

fn append_f64(output: &mut SemanticWriter, value: f64) -> Result<(), SemanticCodecError> {
    output.write_all(&canonical_f64_bits(value)?.to_be_bytes())?;
    Ok(())
}

fn append_h3(output: &mut SemanticWriter, value: u64) -> Result<(), SemanticCodecError> {
    output.write_all(&encode_h3(i128::from(value))?)?;
    Ok(())
}

fn append_stable_key(
    output: &mut SemanticWriter,
    value: &StableElementKey,
) -> Result<(), SemanticCodecError> {
    let bytes = value
        .canonical_bytes()
        .map_err(|_| SemanticCodecError::Invalid("stable element key"))?;
    output.write_all(&bytes)
}

fn append_ordered_stable_keys(
    output: &mut SemanticWriter,
    values: &[StableElementKey],
) -> Result<(), SemanticCodecError> {
    if values.len() > MAX_ITEMS {
        return Err(SemanticCodecError::Invalid("ordered stable key rows"));
    }
    let encoded = values
        .iter()
        .map(StableElementKey::canonical_bytes)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| SemanticCodecError::Invalid("stable element key"))?;
    if encoded.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::NoncanonicalFieldOrder,
        ));
    }
    output.write_all(&checked_u32(encoded.len(), "ordered stable key row count")?.to_be_bytes())?;
    for bytes in encoded {
        output.write_all(&checked_u32(bytes.len(), "stable key byte length")?.to_be_bytes())?;
        output.write_all(&bytes)?;
    }
    Ok(())
}

fn append_stable_key_sequence(
    output: &mut SemanticWriter,
    values: &[StableElementKey],
) -> Result<(), SemanticCodecError> {
    if values.len() > MAX_ITEMS {
        return Err(SemanticCodecError::Invalid("stable key sequence rows"));
    }
    output.write_all(&checked_u32(values.len(), "stable key sequence row count")?.to_be_bytes())?;
    for value in values {
        let bytes = value
            .canonical_bytes()
            .map_err(|_| SemanticCodecError::Invalid("stable element key"))?;
        output.write_all(&checked_u32(bytes.len(), "stable key byte length")?.to_be_bytes())?;
        output.write_all(&bytes)?;
    }
    Ok(())
}

fn append_ordered_utf8(
    output: &mut SemanticWriter,
    values: &[String],
) -> Result<(), SemanticCodecError> {
    if values.len() > MAX_ITEMS {
        return Err(SemanticCodecError::Invalid("ordered UTF-8 rows"));
    }
    output.write_all(&checked_u32(values.len(), "ordered UTF-8 row count")?.to_be_bytes())?;
    for value in values {
        append_utf8(output, value)?;
    }
    Ok(())
}

fn append_named_stable(
    output: &mut SemanticWriter,
    values: &[(&str, &StableBslValue)],
) -> Result<(), SemanticCodecError> {
    validate_name_order(values.iter().map(|(name, _)| *name))?;
    output.write_all(&checked_u32(values.len(), "named stable row count")?.to_be_bytes())?;
    for (name, value) in values {
        append_utf8(output, name)?;
        output.write_all(&encode_stable_bsl(value)?)?;
    }
    Ok(())
}

fn validate_name_order<'a>(
    values: impl Iterator<Item = &'a str>,
) -> Result<(), SemanticCodecError> {
    let mut prior: Option<&str> = None;
    let mut count = 0_usize;
    for value in values {
        count += 1;
        if count > MAX_ITEMS || prior.is_some_and(|prior| prior.as_bytes() >= value.as_bytes()) {
            return Err(SemanticCodecError::Refusal(
                SemanticRefusalCode::NoncanonicalFieldOrder,
            ));
        }
        prior = Some(value);
    }
    Ok(())
}

fn validate_closed_checkpoint_tag(tag: u8) -> Result<(), SemanticCodecError> {
    if (1..=9).contains(&tag) {
        Ok(())
    } else {
        Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::UnknownClosedTag,
        ))
    }
}

fn validate_completeness_tag(tag: u8) -> Result<(), SemanticCodecError> {
    if matches!(tag, 1 | 2) {
        Ok(())
    } else {
        Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::UnknownClosedTag,
        ))
    }
}

fn validate_full_sections(sections: &[(u8, u32, [u8; 32])]) -> Result<(), SemanticCodecError> {
    if sections.len() != 9 || sections.iter().map(|section| section.0).ne(1_u8..=9) {
        return Err(SemanticCodecError::Refusal(
            SemanticRefusalCode::IncompleteFullCheckpoint,
        ));
    }
    Ok(())
}

fn checked_u32(value: usize, field: &'static str) -> Result<u32, SemanticCodecError> {
    u32::try_from(value).map_err(|_| SemanticCodecError::IntegerConversion { field, value })
}

fn compose_row(
    family: CommittedTickRowFamily,
    tag: u8,
    encode_key: impl FnOnce(&mut SemanticWriter) -> Result<(), SemanticCodecError>,
    encode_payload: impl FnOnce(&mut SemanticWriter) -> Result<(), SemanticCodecError>,
) -> Result<CommittedTickRow, SemanticCodecError> {
    let mut key = row_prefix(ROW_KEY_DOMAIN, family, tag)?;
    let mut payload = row_prefix(ROW_PAYLOAD_DOMAIN, family, tag)?;
    encode_key(&mut key)?;
    encode_payload(&mut payload)?;
    CommittedTickRow::compose(key.finish(), payload.finish())
        .map_err(|_| SemanticCodecError::Invalid("row"))
}

#[cfg(test)]
mod allocation_tests {
    use super::{checked_semantic_capacity, SemanticCodecError, SemanticWriter};

    fn refuse_reservation(_bytes: &mut Vec<u8>, _additional: usize) -> Result<(), ()> {
        Err(())
    }

    #[test]
    fn writer_accepts_the_exact_byte_ceiling_and_refuses_plus_one() {
        let mut writer = SemanticWriter::new("writer boundary", 3);
        writer.write_all(&[1, 2, 3]).expect("exact byte ceiling");
        assert_eq!(writer.as_bytes(), [1, 2, 3]);
        assert_eq!(
            writer.write_byte(4),
            Err(SemanticCodecError::ByteLimit {
                field: "writer boundary",
                actual: 4,
                maximum: 3,
            })
        );
        assert_eq!(writer.as_bytes(), [1, 2, 3]);
    }

    #[test]
    fn writer_reports_arithmetic_overflow_before_allocation() {
        assert_eq!(
            checked_semantic_capacity(usize::MAX, 1, "writer overflow", usize::MAX),
            Err(SemanticCodecError::CapacityOverflow {
                field: "writer overflow",
            })
        );
    }

    #[test]
    fn injected_reserve_failure_exposes_no_partial_bytes() {
        let mut writer = SemanticWriter::with_reserver("writer allocation", 8, refuse_reservation);
        assert_eq!(
            writer.write_all(&[1, 2, 3]),
            Err(SemanticCodecError::Allocation {
                field: "writer allocation",
                requested: 3,
            })
        );
        assert!(writer.as_bytes().is_empty());
    }
}
