//! Closed, database-free semantic codec shared by the live Rust persistence
//! writer, restart reconstruction, and exact contract vectors.

use crate::committed_tick_envelope::{
    CommittedTickEnvelopeErrorV2, CommittedTickEnvelopeV2, CommittedTickRowFamiliesV2,
    CommittedTickRowFamilyV2, CommittedTickRowV2,
};
use crate::identity::CampaignId;
use crate::tick_commit_claim::TickCommitClaimV1;
use babylon_bsl::identity_codec::{
    canonical_f64_bits, encode_stable_bsl_value_v1, IdentityCodecError, StableBslValueV1,
};
use babylon_graph::stable_element::StableElementKeyV1;
use babylon_kernel::{sha256_of, tick_content_hash::TickContentHashV1, H3CellId};
use std::collections::TryReserveError;

use uuid::Uuid;

const LAYOUT_V1: u32 = 1;
const ROW_KEY_DOMAIN: &[u8] = b"babylon.committed-tick-row-key.v1\0";
const ROW_PAYLOAD_DOMAIN: &[u8] = b"babylon.committed-tick-row-payload.v1\0";
const FOUNDATION_CONTENT_DOMAIN: &[u8] = b"babylon.campaign-foundation-content.v1\0";
const FOUNDATION_CONTENT_DOMAIN_V2: &[u8] = b"babylon.campaign-foundation-content.v2\0";
const MAX_FOUNDATION_SOURCE_V2: usize = 1_048_576;
const CHECKPOINT_DOMAIN: &[u8] = b"babylon.full-checkpoint-manifest.v1\0";
const EMPTY_PROOF_DOMAIN: &[u8] = b"babylon.semantic-empty-proof.v1\0";
const MAX_UTF8_BYTES: usize = 65_535;
const MAX_BYTES: usize = 67_108_864;
const MAX_ITEMS: usize = 1_048_576;

/// One enum-ordered branch in a canonical choice-receipt semantic row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChoiceReceiptSemanticBranchV1 {
    pub(crate) outcome_member: String,
    pub(crate) mass_nanounits: u64,
    pub(crate) ticket_start: u128,
    pub(crate) ticket_end_exclusive: u128,
    pub(crate) ticket_count: u128,
}

/// Exact aggregate input to the V2 envelope's `ChoiceReceipt` family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChoiceReceiptSemanticRowV1 {
    pub(crate) encounter_ordinal: u32,
    pub(crate) rule_id: String,
    pub(crate) sample: String,
    pub(crate) slot: u32,
    pub(crate) outcome_enum: String,
    pub(crate) stable_carrier: StableElementKeyV1,
    pub(crate) active_elements: Vec<StableElementKeyV1>,
    pub(crate) branches: Vec<ChoiceReceiptSemanticBranchV1>,
    pub(crate) draw_ticket: u64,
    pub(crate) selected_outcome: String,
    pub(crate) allocation_digest: [u8; 32],
    pub(crate) instance_digest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SemanticRefusalCodeV1 {
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

impl SemanticRefusalCodeV1 {
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
pub(crate) enum SemanticCodecErrorV1 {
    Refusal(SemanticRefusalCodeV1),
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

impl From<IdentityCodecError> for SemanticCodecErrorV1 {
    fn from(value: IdentityCodecError) -> Self {
        match value {
            IdentityCodecError::NonFiniteValue => {
                Self::Refusal(SemanticRefusalCodeV1::NonfiniteF64)
            }
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

type ReserveSemanticBytesV1 = fn(&mut Vec<u8>, usize) -> Result<(), ()>;

fn reserve_semantic_bytes_v1(bytes: &mut Vec<u8>, additional: usize) -> Result<(), ()> {
    bytes
        .try_reserve_exact(additional)
        .map_err(|_: TryReserveError| ())
}

pub(crate) struct SemanticWriterV1 {
    field: &'static str,
    maximum: usize,
    bytes: Vec<u8>,
    reserve: ReserveSemanticBytesV1,
}

impl SemanticWriterV1 {
    pub(crate) fn new(field: &'static str, maximum: usize) -> Self {
        Self::with_reserver(field, maximum, reserve_semantic_bytes_v1)
    }

    fn with_reserver(field: &'static str, maximum: usize, reserve: ReserveSemanticBytesV1) -> Self {
        Self {
            field,
            maximum,
            bytes: Vec::new(),
            reserve,
        }
    }

    pub(crate) fn write_byte(&mut self, value: u8) -> Result<(), SemanticCodecErrorV1> {
        self.write_all(&[value])
    }

    pub(crate) fn write_all(&mut self, value: &[u8]) -> Result<(), SemanticCodecErrorV1> {
        let target =
            checked_semantic_capacity(self.bytes.len(), value.len(), self.field, self.maximum)?;
        (self.reserve)(&mut self.bytes, value.len()).map_err(|()| {
            SemanticCodecErrorV1::Allocation {
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
) -> Result<usize, SemanticCodecErrorV1> {
    let actual = current
        .checked_add(additional)
        .ok_or(SemanticCodecErrorV1::CapacityOverflow { field })?;
    if actual > maximum {
        return Err(SemanticCodecErrorV1::ByteLimit {
            field,
            actual,
            maximum,
        });
    }
    Ok(actual)
}

fn encode_fixed(value: &[u8]) -> Result<Vec<u8>, SemanticCodecErrorV1> {
    let mut output = SemanticWriterV1::new("fixed semantic scalar", value.len());
    output.write_all(value)?;
    Ok(output.finish())
}

pub(crate) fn encode_bool(value: bool) -> Result<Vec<u8>, SemanticCodecErrorV1> {
    encode_fixed(&[u8::from(value)])
}
pub(crate) fn encode_u64(value: u64) -> Result<Vec<u8>, SemanticCodecErrorV1> {
    encode_fixed(&value.to_be_bytes())
}
pub(crate) fn encode_i64(value: i64) -> Result<Vec<u8>, SemanticCodecErrorV1> {
    encode_fixed(&value.to_be_bytes())
}
pub(crate) fn encode_i128(value: i128) -> Result<Vec<u8>, SemanticCodecErrorV1> {
    encode_fixed(&value.to_be_bytes())
}

pub(crate) fn encode_f64(value: f64) -> Result<Vec<u8>, SemanticCodecErrorV1> {
    encode_fixed(&canonical_f64_bits(value)?.to_be_bytes())
}

pub(crate) fn encode_h3(raw: i128) -> Result<Vec<u8>, SemanticCodecErrorV1> {
    let raw = i64::try_from(raw)
        .map_err(|_| SemanticCodecErrorV1::Refusal(SemanticRefusalCodeV1::InvalidH3CellId))?;
    let cell = H3CellId::try_from(raw)
        .map_err(|_| SemanticCodecErrorV1::Refusal(SemanticRefusalCodeV1::InvalidH3CellId))?;
    encode_fixed(&cell.to_be_bytes())
}

pub(crate) fn encode_optional_utf8(value: Option<&str>) -> Result<Vec<u8>, SemanticCodecErrorV1> {
    let mut output = SemanticWriterV1::new("optional UTF-8", MAX_BYTES);
    match value {
        None => output.write_byte(0)?,
        Some(value) => {
            output.write_byte(1)?;
            append_utf8(&mut output, value)?;
        }
    }
    Ok(output.finish())
}

pub(crate) fn encode_stable_bsl(value: &StableBslValueV1) -> Result<Vec<u8>, SemanticCodecErrorV1> {
    let mut output = Vec::new();
    encode_stable_bsl_value_v1(value, &mut output)?;
    let mut checked = SemanticWriterV1::new("stable BSL value", MAX_BYTES);
    checked.write_all(&output)?;
    Ok(checked.finish())
}

pub(crate) fn encode_stable_graph_node(
    local_name: &str,
    node_type: &str,
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    compose_row(
        CommittedTickRowFamilyV2::Graph,
        1,
        |key| append_utf8(key, local_name),
        |payload| append_utf8(payload, node_type),
    )
}
pub(crate) fn encode_stable_graph_node_f64(
    local_name: &str,
    qname: &str,
    value: f64,
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    compose_row(
        CommittedTickRowFamilyV2::Graph,
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
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    compose_row(
        CommittedTickRowFamilyV2::Graph,
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
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    compose_row(
        CommittedTickRowFamilyV2::Graph,
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
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    compose_row(
        CommittedTickRowFamilyV2::Graph,
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
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    compose_row(
        CommittedTickRowFamilyV2::Graph,
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
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    compose_row(
        CommittedTickRowFamilyV2::Graph,
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
    value: &StableBslValueV1,
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    compose_row(
        CommittedTickRowFamilyV2::State,
        1,
        |key| append_utf8(key, register_name),
        |payload| {
            payload.write_all(&encode_stable_bsl(value)?)?;
            Ok(())
        },
    )
}
pub(crate) fn encode_territory_state(
    territory_id: &StableElementKeyV1,
    ordered_fields: &[(&str, &StableBslValueV1)],
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    compose_row(
        CommittedTickRowFamilyV2::State,
        2,
        |key| append_stable_key(key, territory_id),
        |payload| append_named_stable(payload, ordered_fields),
    )
}
pub(crate) fn encode_dynamic_hex_state(
    cell_id: u64,
    values: &[f64; 9],
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    compose_row(
        CommittedTickRowFamilyV2::State,
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
    organization_id: &StableElementKeyV1,
    organization_kind: &StableBslValueV1,
    ordered_territory_ids: &[StableElementKeyV1],
    ordered_fields: &[(&str, &StableBslValueV1)],
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    compose_row(
        CommittedTickRowFamilyV2::State,
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
    ordered_fields: &[(&str, &StableBslValueV1)],
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    compose_row(
        CommittedTickRowFamilyV2::Event,
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
pub(crate) fn encode_historical_successful_event_v1_vector(
    ordinal: u32,
    event_type: &str,
    ordered_fields: &[(&str, &StableBslValueV1)],
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    compose_row(
        CommittedTickRowFamilyV2::Event,
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
    receipt: &ChoiceReceiptSemanticRowV1,
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    compose_row(
        CommittedTickRowFamilyV2::ChoiceReceipt,
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
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    validate_closed_checkpoint_tag(section_tag)?;
    validate_completeness_tag(completeness_tag)?;
    compose_row(
        CommittedTickRowFamilyV2::Checkpoint,
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
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    let key = row_prefix(
        ROW_KEY_DOMAIN,
        CommittedTickRowFamilyV2::ArchiveDirtyReceipt,
        1,
    )?;
    let mut payload = SemanticWriterV1::new("archive dirty receipt payload", 32);
    payload.write_all(tick_content_hash)?;
    CommittedTickRowV2::compose(key.finish(), payload.finish())
        .map_err(|_| SemanticCodecErrorV1::Invalid("archive dirty receipt row"))
}

pub(crate) fn encode_foundation_content(
    scenario_source: &str,
    prelude_source: Option<&str>,
    rule_source: &str,
    defines: &[u8],
    reference_manifest: &[u8],
) -> Result<Vec<u8>, SemanticCodecErrorV1> {
    let mut output = SemanticWriterV1::new("foundation content", MAX_BYTES);
    output.write_all(FOUNDATION_CONTENT_DOMAIN)?;
    output.write_all(&LAYOUT_V1.to_be_bytes())?;
    output.write_byte(1)?;
    append_utf8(&mut output, scenario_source)?;
    output.write_byte(2)?;
    output.write_all(&encode_optional_utf8(prelude_source)?)?;
    output.write_byte(3)?;
    append_utf8(&mut output, rule_source)?;
    output.write_byte(4)?;
    append_bytes(&mut output, defines)?;
    output.write_byte(5)?;
    append_bytes(&mut output, reference_manifest)?;
    Ok(output.finish())
}

/// Explicit content successor. V1's source encoder and bounds remain frozen.
pub(crate) fn encode_foundation_content_v2(
    scenario_source: &str,
    prelude_source: Option<&str>,
    rule_source: &str,
    defines: &[u8],
    reference_manifest: &[u8],
) -> Result<Vec<u8>, SemanticCodecErrorV1> {
    foundation_content_v2_length(
        scenario_source.len(),
        prelude_source.map(str::len),
        rule_source.len(),
        defines.len(),
        reference_manifest.len(),
    )?;
    let mut output = SemanticWriterV1::new("foundation content V2", MAX_BYTES);
    output.write_all(FOUNDATION_CONTENT_DOMAIN_V2)?;
    output.write_all(&2_u32.to_be_bytes())?;
    output.write_byte(1)?;
    append_foundation_source_v2(&mut output, scenario_source)?;
    output.write_byte(2)?;
    match prelude_source {
        None => output.write_byte(0)?,
        Some(source) => {
            output.write_byte(1)?;
            append_foundation_source_v2(&mut output, source)?;
        }
    }
    output.write_byte(3)?;
    append_foundation_source_v2(&mut output, rule_source)?;
    output.write_byte(4)?;
    append_bytes(&mut output, defines)?;
    output.write_byte(5)?;
    append_bytes(&mut output, reference_manifest)?;
    Ok(output.finish())
}

fn foundation_content_v2_length(
    scenario: usize,
    prelude: Option<usize>,
    rules: usize,
    defines: usize,
    reference: usize,
) -> Result<usize, SemanticCodecErrorV1> {
    if [Some(scenario), prelude, Some(rules)]
        .into_iter()
        .flatten()
        .any(|length| length > MAX_FOUNDATION_SOURCE_V2)
        || defines > MAX_BYTES
        || reference > MAX_BYTES
    {
        return Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::FieldByteBound,
        ));
    }
    // Domain, version, five tags, four required length words and one option tag.
    let mut length = FOUNDATION_CONTENT_DOMAIN_V2.len() + 4 + 5 + 4 * 4 + 1;
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
            .ok_or(SemanticCodecErrorV1::CapacityOverflow {
                field: "foundation content V2",
            })?;
    }
    if length > MAX_BYTES {
        return Err(SemanticCodecErrorV1::ByteLimit {
            field: "foundation content V2",
            actual: length,
            maximum: MAX_BYTES,
        });
    }
    Ok(length)
}

fn append_foundation_source_v2(
    output: &mut SemanticWriterV1,
    source: &str,
) -> Result<(), SemanticCodecErrorV1> {
    if source.len() > MAX_FOUNDATION_SOURCE_V2 || source.as_bytes().contains(&0) {
        return Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::FieldByteBound,
        ));
    }
    output.write_all(&checked_u32(source.len(), "V2 source byte length")?.to_be_bytes())?;
    output.write_all(source.as_bytes())
}

#[cfg(test)]
mod foundation_content_v2_bounds_tests {
    use super::*;

    #[test]
    fn complete_bundle_includes_framing_and_enforces_exact_aggregate_bound() {
        let overhead = foundation_content_v2_length(0, None, 0, 0, 0).unwrap();
        assert_eq!(
            foundation_content_v2_length(0, None, 0, MAX_BYTES - overhead, 0),
            Ok(MAX_BYTES)
        );
        assert!(
            matches!(foundation_content_v2_length(0, None, 0, MAX_BYTES - overhead, 1),
            Err(SemanticCodecErrorV1::ByteLimit { actual, maximum: MAX_BYTES, .. }) if actual == MAX_BYTES + 1)
        );
        assert_eq!(
            foundation_content_v2_length(0, Some(0), 0, 0, 0),
            Ok(overhead + 4)
        );
        let encoded =
            encode_foundation_content_v2("scenario", Some("prelude"), "", b"defines", b"ref")
                .unwrap();
        assert_eq!(
            encoded.len(),
            foundation_content_v2_length(8, Some(7), 0, 7, 3).unwrap()
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
) -> Result<Vec<u8>, SemanticCodecErrorV1> {
    let mut output = SemanticWriterV1::new("campaign foundation", MAX_BYTES);
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
) -> Result<Vec<u8>, SemanticCodecErrorV1> {
    validate_resolve_tick(resolve_tick)?;
    validate_full_sections(sections)?;
    let mut output = SemanticWriterV1::new("full checkpoint", MAX_BYTES);
    output.write_all(CHECKPOINT_DOMAIN)?;
    output.write_all(&LAYOUT_V1.to_be_bytes())?;
    output.write_byte(1)?;
    output.write_all(campaign_id.canonical_bytes())?;
    output.write_all(&resolve_tick.to_be_bytes())?;
    let section_count =
        u16::try_from(sections.len()).map_err(|_| SemanticCodecErrorV1::IntegerConversion {
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
) -> Result<Vec<u8>, SemanticCodecErrorV1> {
    validate_producer_tag(producer_tag)?;
    if source_count != 0 {
        return Err(SemanticCodecErrorV1::Invalid("nonempty source proof"));
    }
    let mut output = SemanticWriterV1::new("semantic empty proof", MAX_BYTES);
    output.write_all(EMPTY_PROOF_DOMAIN)?;
    output.write_all(&LAYOUT_V1.to_be_bytes())?;
    output.write_byte(producer_tag)?;
    output.write_all(&source_count.to_be_bytes())?;
    output.write_all(&source_digest)?;
    Ok(output.finish())
}

pub(crate) fn validate_duplicate_row_keys(key_ids: &[String]) -> Result<(), SemanticCodecErrorV1> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(key_ids.len())
        .map_err(|_: TryReserveError| SemanticCodecErrorV1::Allocation {
            field: "duplicate row fixture",
            requested: key_ids.len(),
        })?;
    for key in key_ids {
        let mut key_writer = SemanticWriterV1::new("duplicate row fixture key", MAX_BYTES);
        key_writer.write_all(key.as_bytes())?;
        let payload_writer = SemanticWriterV1::new("duplicate row fixture payload", MAX_BYTES);
        rows.push(
            CommittedTickRowV2::compose(key_writer.finish(), payload_writer.finish())
                .map_err(|_| SemanticCodecErrorV1::Invalid("duplicate row fixture"))?,
        );
    }
    let input = CommittedTickRowFamiliesV2 {
        graph: Vec::new(),
        state: Vec::new(),
        event: rows,
        choice_receipt: Vec::new(),
        checkpoint: Vec::new(),
        archive_dirty_receipt: encode_archive_dirty_receipt(&[0; 32])?,
    };
    let claim = TickCommitClaimV1::compose(
        CampaignId::from_uuid(Uuid::nil()),
        1,
        TickContentHashV1::from_bytes([0; 32]),
    );
    match CommittedTickEnvelopeV2::compose(claim, input) {
        Err(CommittedTickEnvelopeErrorV2::DuplicateRowKey { .. }) => Err(
            SemanticCodecErrorV1::Refusal(SemanticRefusalCodeV1::DuplicateRowKey),
        ),
        Ok(_) => Ok(()),
        Err(_) => Err(SemanticCodecErrorV1::Invalid("duplicate row rule")),
    }
}

pub(crate) fn validate_producer_tag(tag: u8) -> Result<(), SemanticCodecErrorV1> {
    if matches!(tag, 1 | 16 | 24 | 32 | 96 | 112) {
        Ok(())
    } else {
        Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::UnknownProducerTag,
        ))
    }
}

pub(crate) fn validate_resolve_tick(tick: u64) -> Result<(), SemanticCodecErrorV1> {
    if tick == 0 {
        Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::SyntheticTickZero,
        ))
    } else if tick > i64::MAX as u64 {
        Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::ResolveTickSqlRange,
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn validate_empty_family(
    family: &str,
    proof_producer: Option<&str>,
) -> Result<(), SemanticCodecErrorV1> {
    let expected = match family {
        "event" => "successful_event_batch_v2",
        "choice_receipt" => "choice_receipt_batch_v1",
        _ => return Err(SemanticCodecErrorV1::Invalid("empty family")),
    };
    match proof_producer {
        None => Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::MissingEmptyProof,
        )),
        Some(actual) if actual != expected => Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::ForeignEmptyProof,
        )),
        Some(_) => Ok(()),
    }
}

pub(crate) fn validate_restart_root(
    completeness: &str,
    section_tags: &[u8],
) -> Result<(), SemanticCodecErrorV1> {
    if completeness == "delta" {
        return Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::DeltaCheckpointNotRestartRoot,
        ));
    }
    if completeness != "full" || section_tags != [1, 2, 3, 4, 5, 6, 7, 8, 9] {
        return Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::IncompleteFullCheckpoint,
        ));
    }
    Ok(())
}

pub(crate) fn validate_foundation_artifact(
    present: bool,
    expected: Option<[u8; 32]>,
    actual: Option<[u8; 32]>,
) -> Result<(), SemanticCodecErrorV1> {
    if !present {
        return Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::MissingFoundationArtifact,
        ));
    }
    if expected != actual {
        return Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::FoundationArtifactDigestMismatch,
        ));
    }
    Ok(())
}

pub(crate) fn refuse_runtime_graph_handle() -> SemanticCodecErrorV1 {
    SemanticCodecErrorV1::Refusal(SemanticRefusalCodeV1::RuntimeGraphHandle)
}

pub(crate) fn refuse_unknown_closed_tag() -> SemanticCodecErrorV1 {
    SemanticCodecErrorV1::Refusal(SemanticRefusalCodeV1::UnknownClosedTag)
}

pub(crate) fn refuse_opaque_payload() -> SemanticCodecErrorV1 {
    SemanticCodecErrorV1::Refusal(SemanticRefusalCodeV1::OpaqueSemanticPayload)
}

pub(crate) fn validate_utf8_length(length: usize) -> Result<(), SemanticCodecErrorV1> {
    if length > MAX_UTF8_BYTES {
        Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::FieldByteBound,
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
    family: CommittedTickRowFamilyV2,
    tag: u8,
) -> Result<SemanticWriterV1, SemanticCodecErrorV1> {
    let mut output = SemanticWriterV1::new("semantic row", MAX_BYTES);
    output.write_all(domain)?;
    output.write_all(&LAYOUT_V1.to_be_bytes())?;
    output.write_byte(family.tag())?;
    output.write_byte(producer_tag(family))?;
    output.write_byte(tag)?;
    Ok(output)
}

const fn producer_tag(family: CommittedTickRowFamilyV2) -> u8 {
    match family {
        CommittedTickRowFamilyV2::Graph => 1,
        CommittedTickRowFamilyV2::State => 16,
        CommittedTickRowFamilyV2::Event => 32,
        CommittedTickRowFamilyV2::ChoiceReceipt => 24,
        CommittedTickRowFamilyV2::Checkpoint => 96,
        CommittedTickRowFamilyV2::ArchiveDirtyReceipt => 112,
    }
}

fn append_utf8(output: &mut SemanticWriterV1, value: &str) -> Result<(), SemanticCodecErrorV1> {
    validate_utf8_length(value.len())?;
    if value.as_bytes().contains(&0) {
        return Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::FieldByteBound,
        ));
    }
    output.write_all(&checked_u32(value.len(), "UTF-8 byte length")?.to_be_bytes())?;
    output.write_all(value.as_bytes())?;
    Ok(())
}

fn append_bytes(output: &mut SemanticWriterV1, value: &[u8]) -> Result<(), SemanticCodecErrorV1> {
    if value.len() > MAX_BYTES {
        return Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::FieldByteBound,
        ));
    }
    output.write_all(&checked_u32(value.len(), "byte length")?.to_be_bytes())?;
    output.write_all(value)?;
    Ok(())
}

fn append_f64(output: &mut SemanticWriterV1, value: f64) -> Result<(), SemanticCodecErrorV1> {
    output.write_all(&canonical_f64_bits(value)?.to_be_bytes())?;
    Ok(())
}

fn append_h3(output: &mut SemanticWriterV1, value: u64) -> Result<(), SemanticCodecErrorV1> {
    output.write_all(&encode_h3(i128::from(value))?)?;
    Ok(())
}

fn append_stable_key(
    output: &mut SemanticWriterV1,
    value: &StableElementKeyV1,
) -> Result<(), SemanticCodecErrorV1> {
    let bytes = value
        .canonical_bytes()
        .map_err(|_| SemanticCodecErrorV1::Invalid("stable element key"))?;
    output.write_all(&bytes)
}

fn append_ordered_stable_keys(
    output: &mut SemanticWriterV1,
    values: &[StableElementKeyV1],
) -> Result<(), SemanticCodecErrorV1> {
    if values.len() > MAX_ITEMS {
        return Err(SemanticCodecErrorV1::Invalid("ordered stable key rows"));
    }
    let encoded = values
        .iter()
        .map(StableElementKeyV1::canonical_bytes)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| SemanticCodecErrorV1::Invalid("stable element key"))?;
    if encoded.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::NoncanonicalFieldOrder,
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
    output: &mut SemanticWriterV1,
    values: &[StableElementKeyV1],
) -> Result<(), SemanticCodecErrorV1> {
    if values.len() > MAX_ITEMS {
        return Err(SemanticCodecErrorV1::Invalid("stable key sequence rows"));
    }
    output.write_all(&checked_u32(values.len(), "stable key sequence row count")?.to_be_bytes())?;
    for value in values {
        let bytes = value
            .canonical_bytes()
            .map_err(|_| SemanticCodecErrorV1::Invalid("stable element key"))?;
        output.write_all(&checked_u32(bytes.len(), "stable key byte length")?.to_be_bytes())?;
        output.write_all(&bytes)?;
    }
    Ok(())
}

fn append_ordered_utf8(
    output: &mut SemanticWriterV1,
    values: &[String],
) -> Result<(), SemanticCodecErrorV1> {
    if values.len() > MAX_ITEMS {
        return Err(SemanticCodecErrorV1::Invalid("ordered UTF-8 rows"));
    }
    output.write_all(&checked_u32(values.len(), "ordered UTF-8 row count")?.to_be_bytes())?;
    for value in values {
        append_utf8(output, value)?;
    }
    Ok(())
}

fn append_named_stable(
    output: &mut SemanticWriterV1,
    values: &[(&str, &StableBslValueV1)],
) -> Result<(), SemanticCodecErrorV1> {
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
) -> Result<(), SemanticCodecErrorV1> {
    let mut prior: Option<&str> = None;
    let mut count = 0_usize;
    for value in values {
        count += 1;
        if count > MAX_ITEMS || prior.is_some_and(|prior| prior.as_bytes() >= value.as_bytes()) {
            return Err(SemanticCodecErrorV1::Refusal(
                SemanticRefusalCodeV1::NoncanonicalFieldOrder,
            ));
        }
        prior = Some(value);
    }
    Ok(())
}

fn validate_closed_checkpoint_tag(tag: u8) -> Result<(), SemanticCodecErrorV1> {
    if (1..=9).contains(&tag) {
        Ok(())
    } else {
        Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::UnknownClosedTag,
        ))
    }
}

fn validate_completeness_tag(tag: u8) -> Result<(), SemanticCodecErrorV1> {
    if matches!(tag, 1 | 2) {
        Ok(())
    } else {
        Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::UnknownClosedTag,
        ))
    }
}

fn validate_full_sections(sections: &[(u8, u32, [u8; 32])]) -> Result<(), SemanticCodecErrorV1> {
    if sections.len() != 9 || sections.iter().map(|section| section.0).ne(1_u8..=9) {
        return Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::IncompleteFullCheckpoint,
        ));
    }
    Ok(())
}

fn checked_u32(value: usize, field: &'static str) -> Result<u32, SemanticCodecErrorV1> {
    u32::try_from(value).map_err(|_| SemanticCodecErrorV1::IntegerConversion { field, value })
}

fn compose_row(
    family: CommittedTickRowFamilyV2,
    tag: u8,
    encode_key: impl FnOnce(&mut SemanticWriterV1) -> Result<(), SemanticCodecErrorV1>,
    encode_payload: impl FnOnce(&mut SemanticWriterV1) -> Result<(), SemanticCodecErrorV1>,
) -> Result<CommittedTickRowV2, SemanticCodecErrorV1> {
    let mut key = row_prefix(ROW_KEY_DOMAIN, family, tag)?;
    let mut payload = row_prefix(ROW_PAYLOAD_DOMAIN, family, tag)?;
    encode_key(&mut key)?;
    encode_payload(&mut payload)?;
    CommittedTickRowV2::compose(key.finish(), payload.finish())
        .map_err(|_| SemanticCodecErrorV1::Invalid("row"))
}

#[cfg(test)]
mod allocation_tests {
    use super::{checked_semantic_capacity, SemanticCodecErrorV1, SemanticWriterV1};

    fn refuse_reservation(_bytes: &mut Vec<u8>, _additional: usize) -> Result<(), ()> {
        Err(())
    }

    #[test]
    fn writer_accepts_the_exact_byte_ceiling_and_refuses_plus_one() {
        let mut writer = SemanticWriterV1::new("writer boundary", 3);
        writer.write_all(&[1, 2, 3]).expect("exact byte ceiling");
        assert_eq!(writer.as_bytes(), [1, 2, 3]);
        assert_eq!(
            writer.write_byte(4),
            Err(SemanticCodecErrorV1::ByteLimit {
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
            Err(SemanticCodecErrorV1::CapacityOverflow {
                field: "writer overflow",
            })
        );
    }

    #[test]
    fn injected_reserve_failure_exposes_no_partial_bytes() {
        let mut writer =
            SemanticWriterV1::with_reserver("writer allocation", 8, refuse_reservation);
        assert_eq!(
            writer.write_all(&[1, 2, 3]),
            Err(SemanticCodecErrorV1::Allocation {
                field: "writer allocation",
                requested: 3,
            })
        );
        assert!(writer.as_bytes().is_empty());
    }
}
