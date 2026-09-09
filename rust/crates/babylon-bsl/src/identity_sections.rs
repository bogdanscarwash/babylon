//! Bounded snapshots of live BSL registries and exact tick observations.

use std::collections::HashMap;
use std::hash::BuildHasher;

use babylon_graph::stable_element::StableElementResolverV1;

use crate::causal_contract::AuditReceipt;
use crate::evaluator::Value;
use crate::fuel::IntrinsicCosts;
use crate::identity_codec::{
    canonical_event_name_v1, checked_add, checked_u32, encode_bsl_type_v1, encode_const_value_v1,
    encode_effect_signature_v1, encode_enum_kind_v1, encode_evidence_class_v1,
    encode_field_kind_v1, encode_rule_role_v1, validate_enum_member, validate_enum_type,
    validate_governance_text, validate_intrinsic_identity, validate_qname, validate_symbol,
    IdentityCodecError, IdentityWriter, IntrinsicIdentityViolation,
};
use crate::typecheck::TypeEnv;
use crate::types::{EnumDecl, EnumRegistry};
use crate::vocabulary::{ClosedVocabulary, EnumKind};

/// Maximum ordinary prepared rows in one section.
pub const MAX_PREPARED_ROWS_V1: usize = 65_536;
/// Maximum exemption or intrinsic-cost rows.
pub const MAX_PREPARED_SMALL_ROWS_V1: usize = 64;
/// Maximum members in one prepared enum declaration.
pub const MAX_PREPARED_ENUM_MEMBERS_V1: usize = 4_096;
/// Maximum members in one prepared structural-vocabulary kind.
pub const MAX_PREPARED_VOCABULARY_MEMBERS_V1: usize = 524_288;
/// Maximum aggregate prepared-environment rows.
pub const MAX_PREPARED_AGGREGATE_ROWS_V1: usize = 1_048_576;
/// Maximum aggregate tick-payload rows.
pub const MAX_TICK_AGGREGATE_ROWS_V1: usize = 1_048_576;
/// Maximum combined prepared-environment section bytes.
pub const MAX_PREPARED_COMBINED_BYTES_V1: usize = 67_108_864;
/// Maximum combined tick-payload section bytes.
pub const MAX_TICK_COMBINED_BYTES_V1: usize = 67_108_864;
/// Maximum tick rule-outcome rows.
pub const MAX_TICK_RULE_OUTCOMES_V1: usize = 65_536;
/// Maximum events or receipts in one tick payload.
pub const MAX_TICK_ROWS_V1: usize = 1_048_576;

/// Canonical BSL-owned prepared-environment section bodies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedBslSectionsV1 {
    fields_and_exemptions: Vec<u8>,
    intrinsic_costs: Vec<u8>,
    constants: Vec<u8>,
    enum_types: Vec<u8>,
    vocabulary: Vec<u8>,
    aggregate_rows: u32,
}

impl PreparedBslSectionsV1 {
    /// Borrow tag `0x04`'s body.
    #[must_use]
    pub fn fields_and_exemptions(&self) -> &[u8] {
        &self.fields_and_exemptions
    }

    /// Borrow tag `0x05`'s body.
    #[must_use]
    pub fn intrinsic_costs(&self) -> &[u8] {
        &self.intrinsic_costs
    }

    /// Borrow tag `0x06`'s body.
    #[must_use]
    pub fn constants(&self) -> &[u8] {
        &self.constants
    }

    /// Borrow tag `0x07`'s body.
    #[must_use]
    pub fn enum_types(&self) -> &[u8] {
        &self.enum_types
    }

    /// Borrow tag `0x08`'s body.
    #[must_use]
    pub fn vocabulary(&self) -> &[u8] {
        &self.vocabulary
    }

    /// Return all nested registry and member rows.
    #[must_use]
    pub const fn aggregate_rows(&self) -> u32 {
        self.aggregate_rows
    }
}

/// Canonical BSL-owned tick-payload section bodies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickPayloadSectionsV1 {
    rule_outcomes: Vec<u8>,
    events: Vec<u8>,
    receipts: Vec<u8>,
    accepted_action_outcomes: [u8; 2],
    aggregate_rows: u32,
}

impl TickPayloadSectionsV1 {
    /// Borrow tag `0x01`'s body.
    #[must_use]
    pub fn rule_outcomes(&self) -> &[u8] {
        &self.rule_outcomes
    }

    /// Borrow tag `0x02`'s body.
    #[must_use]
    pub fn events(&self) -> &[u8] {
        &self.events
    }

    /// Borrow tag `0x03`'s body.
    #[must_use]
    pub fn receipts(&self) -> &[u8] {
        &self.receipts
    }

    /// Borrow tag `0x04`'s fixed zero count.
    #[must_use]
    pub const fn accepted_action_outcomes(&self) -> &[u8; 2] {
        &self.accepted_action_outcomes
    }

    /// Return all rule, event, payload-item, and receipt rows.
    #[must_use]
    pub const fn aggregate_rows(&self) -> u32 {
        self.aggregate_rows
    }
}

/// Snapshot the live prepared BSL registries into governed section bodies.
///
/// # Errors
/// Returns the first count, aggregate, semantic, scalar, byte, arithmetic,
/// conversion, or allocation failure.
pub fn encode_prepared_bsl_sections_v1<S: BuildHasher>(
    types: &TypeEnv,
    intrinsics: &IntrinsicCosts,
    constants: &HashMap<String, Value, S>,
    enums: &EnumRegistry,
    vocabulary: Option<&ClosedVocabulary>,
) -> Result<PreparedBslSectionsV1, IdentityCodecError> {
    let intrinsic_count = intrinsics.identity_rows().count();
    validate_prepared_counts(types, intrinsic_count, constants, enums, vocabulary)?;
    let aggregate = prepared_aggregate_rows(types, intrinsic_count, constants, enums, vocabulary)?;
    validate_aggregate(aggregate, MAX_PREPARED_AGGREGATE_ROWS_V1)?;
    let fields_and_exemptions = encode_fields(types, enums)?;
    let intrinsic_costs = encode_intrinsics(intrinsics, intrinsic_count)?;
    let constants = encode_constants(constants)?;
    let enum_types = encode_enum_types(enums)?;
    let vocabulary = encode_vocabulary(vocabulary)?;
    validate_combined_bytes(
        &[
            &fields_and_exemptions,
            &intrinsic_costs,
            &constants,
            &enum_types,
            &vocabulary,
        ],
        MAX_PREPARED_COMBINED_BYTES_V1,
    )?;
    Ok(PreparedBslSectionsV1 {
        fields_and_exemptions,
        intrinsic_costs,
        constants,
        enum_types,
        vocabulary,
        aggregate_rows: checked_u32("prepared BSL aggregate rows", aggregate)?,
    })
}

fn validate_prepared_counts<S: BuildHasher>(
    types: &TypeEnv,
    intrinsic_count: usize,
    constants: &HashMap<String, Value, S>,
    enums: &EnumRegistry,
    vocabulary: Option<&ClosedVocabulary>,
) -> Result<(), IdentityCodecError> {
    validate_rows("fields", types.fields.len(), MAX_PREPARED_ROWS_V1)?;
    validate_rows(
        "exemptions",
        types.exemptions.len(),
        MAX_PREPARED_SMALL_ROWS_V1,
    )?;
    validate_rows(
        "intrinsic costs",
        intrinsic_count,
        MAX_PREPARED_SMALL_ROWS_V1,
    )?;
    validate_rows("constants", constants.len(), MAX_PREPARED_ROWS_V1)?;
    validate_rows(
        "enum types",
        enums.declarations().len(),
        MAX_PREPARED_ROWS_V1,
    )?;
    for declaration in enums.declarations().iter().take(MAX_PREPARED_ROWS_V1 + 1) {
        validate_rows(
            "enum members",
            declaration.members.len(),
            MAX_PREPARED_ENUM_MEMBERS_V1,
        )?;
    }
    if let Some(vocabulary) = vocabulary {
        for kind in enum_kinds() {
            validate_rows(
                "vocabulary members",
                vocabulary.members(kind).len(),
                MAX_PREPARED_VOCABULARY_MEMBERS_V1,
            )?;
        }
    }
    Ok(())
}

fn prepared_aggregate_rows<S: BuildHasher>(
    types: &TypeEnv,
    intrinsic_count: usize,
    constants: &HashMap<String, Value, S>,
    enums: &EnumRegistry,
    vocabulary: Option<&ClosedVocabulary>,
) -> Result<usize, IdentityCodecError> {
    let mut total = 0usize;
    for count in [
        types.fields.len(),
        types.exemptions.len(),
        intrinsic_count,
        constants.len(),
        enums.declarations().len(),
    ] {
        total = checked_add("prepared BSL aggregate rows", total, count)?;
    }
    for declaration in enums.declarations().iter().take(MAX_PREPARED_ROWS_V1 + 1) {
        total = checked_add(
            "prepared BSL aggregate rows",
            total,
            declaration.members.len(),
        )?;
    }
    if let Some(vocabulary) = vocabulary {
        total = checked_add("prepared BSL aggregate rows", total, 4)?;
        for kind in enum_kinds() {
            total = checked_add(
                "prepared BSL aggregate rows",
                total,
                vocabulary.members(kind).len(),
            )?;
        }
    }
    Ok(total)
}

fn encode_fields(types: &TypeEnv, enums: &EnumRegistry) -> Result<Vec<u8>, IdentityCodecError> {
    let mut fields: Vec<_> = types.fields.iter().collect();
    fields.sort_unstable_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    let mut output = IdentityWriter::new("prepared fields and exemptions");
    output.extend(&checked_u32("field count", fields.len())?.to_be_bytes())?;
    for (qname, declaration) in fields.iter().take(MAX_PREPARED_ROWS_V1 + 1) {
        validate_qname("field qname", qname)?;
        output.str32("field qname", qname)?;
        let mut encoded_type = Vec::new();
        encode_bsl_type_v1(&declaration.ty, enums, &mut encoded_type)?;
        output.extend(&encoded_type)?;
        output.push(encode_field_kind_v1(declaration.kind))?;
    }
    encode_exemptions(types, &mut output)?;
    Ok(output.finish())
}

fn encode_exemptions(
    types: &TypeEnv,
    output: &mut IdentityWriter,
) -> Result<(), IdentityCodecError> {
    let mut rows: Vec<_> = types.exemptions.iter().collect();
    rows.sort_unstable_by(|left, right| {
        (left.field_name, left.reason, left.owner, left.date).cmp(&(
            right.field_name,
            right.reason,
            right.owner,
            right.date,
        ))
    });
    output.extend(&checked_u32("exemption count", rows.len())?.to_be_bytes())?;
    for row in rows.iter().take(MAX_PREPARED_SMALL_ROWS_V1 + 1) {
        validate_qname("exemption field", row.field_name)?;
        output.str32("exemption field", row.field_name)?;
        validate_governance_text("exemption reason", row.reason)?;
        output.str32("exemption reason", row.reason)?;
        validate_governance_text("exemption owner", row.owner)?;
        output.str32("exemption owner", row.owner)?;
        validate_governance_text("exemption date", row.date)?;
        output.str32("exemption date", row.date)?;
    }
    Ok(())
}

fn encode_intrinsics(
    intrinsics: &IntrinsicCosts,
    count: usize,
) -> Result<Vec<u8>, IdentityCodecError> {
    let mut rows: Vec<_> = intrinsics.identity_rows().collect();
    rows.sort_unstable_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    let mut output = IdentityWriter::new("prepared intrinsic costs");
    output.extend(&checked_u32("intrinsic count", count)?.to_be_bytes())?;
    for (name, cost) in rows.iter().take(MAX_PREPARED_SMALL_ROWS_V1 + 1) {
        validate_intrinsic_identity(name).map_err(|violation| {
            IdentityCodecError::InvalidString {
                field: "intrinsic name",
                index: match violation {
                    IntrinsicIdentityViolation::Empty => 0,
                    IntrinsicIdentityViolation::TooLong { actual } => actual,
                    IntrinsicIdentityViolation::NonAscii { index }
                    | IntrinsicIdentityViolation::Delimiter { index } => index,
                },
            }
        })?;
        output.str32("intrinsic name", name)?;
        output.extend(&cost.to_be_bytes())?;
    }
    Ok(output.finish())
}

fn encode_constants<S: BuildHasher>(
    constants: &HashMap<String, Value, S>,
) -> Result<Vec<u8>, IdentityCodecError> {
    let mut rows: Vec<_> = constants.iter().collect();
    rows.sort_unstable_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    let mut output = IdentityWriter::new("prepared constants");
    output.extend(&checked_u32("constant count", rows.len())?.to_be_bytes())?;
    for (qname, value) in rows.iter().take(MAX_PREPARED_ROWS_V1 + 1) {
        validate_qname("constant qname", qname)?;
        output.str32("constant qname", qname)?;
        let mut encoded = Vec::new();
        encode_const_value_v1(value, &mut encoded)?;
        output.extend(&encoded)?;
    }
    Ok(output.finish())
}

fn encode_enum_types(enums: &EnumRegistry) -> Result<Vec<u8>, IdentityCodecError> {
    let mut rows: Vec<&EnumDecl> = enums.declarations().iter().collect();
    rows.sort_unstable_by(|left, right| left.name.as_bytes().cmp(right.name.as_bytes()));
    let mut output = IdentityWriter::new("prepared enum types");
    output.extend(&checked_u32("enum type count", rows.len())?.to_be_bytes())?;
    for declaration in rows.iter().take(MAX_PREPARED_ROWS_V1 + 1) {
        validate_enum_type("enum type name", &declaration.name)?;
        output.str32("enum type name", &declaration.name)?;
        output
            .extend(&checked_u32("enum member count", declaration.members.len())?.to_be_bytes())?;
        for member in declaration
            .members
            .iter()
            .take(MAX_PREPARED_ENUM_MEMBERS_V1 + 1)
        {
            validate_enum_member("enum member", member)?;
            output.str32("enum member", member)?;
        }
    }
    Ok(output.finish())
}

fn encode_vocabulary(vocabulary: Option<&ClosedVocabulary>) -> Result<Vec<u8>, IdentityCodecError> {
    let mut output = IdentityWriter::new("prepared vocabulary");
    let Some(vocabulary) = vocabulary else {
        output.push(0)?;
        return Ok(output.finish());
    };
    output.push(1)?;
    for kind in enum_kinds() {
        output.push(encode_enum_kind_v1(kind))?;
        if vocabulary.contains_kind(kind) {
            output.push(1)?;
            let mut members = vocabulary.members(kind).to_vec();
            members.sort_unstable_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
            output.extend(&checked_u32("vocabulary member count", members.len())?.to_be_bytes())?;
            for member in members.iter().take(MAX_PREPARED_VOCABULARY_MEMBERS_V1 + 1) {
                validate_enum_member("vocabulary member", member)?;
                output.str32("vocabulary member", member)?;
            }
        } else {
            output.push(0)?;
        }
    }
    Ok(output.finish())
}

/// Encode live tick outcomes, events, and receipts without reordering them.
///
/// # Errors
/// Returns the first count, aggregate, semantic, stable-reference, scalar,
/// byte, arithmetic, conversion, or allocation failure.
pub fn encode_tick_payload_sections_v1(
    outcomes: &[(String, usize)],
    events: &[(String, Vec<(String, Value)>)],
    receipts: &[AuditReceipt],
    resolver: &StableElementResolverV1,
) -> Result<TickPayloadSectionsV1, IdentityCodecError> {
    validate_tick_counts(outcomes, events, receipts)?;
    let aggregate = tick_aggregate_rows(outcomes, events, receipts)?;
    validate_aggregate(aggregate, MAX_TICK_AGGREGATE_ROWS_V1)?;
    let rule_outcomes = encode_outcomes(outcomes)?;
    let events = encode_events(events, resolver)?;
    let receipts = encode_receipts(receipts)?;
    validate_combined_bytes(
        &[&rule_outcomes, &events, &receipts, &[0, 0]],
        MAX_TICK_COMBINED_BYTES_V1,
    )?;
    Ok(TickPayloadSectionsV1 {
        rule_outcomes,
        events,
        receipts,
        accepted_action_outcomes: [0, 0],
        aggregate_rows: checked_u32("tick BSL aggregate rows", aggregate)?,
    })
}

fn validate_tick_counts(
    outcomes: &[(String, usize)],
    events: &[(String, Vec<(String, Value)>)],
    receipts: &[AuditReceipt],
) -> Result<(), IdentityCodecError> {
    validate_rows("rule outcomes", outcomes.len(), MAX_TICK_RULE_OUTCOMES_V1)?;
    validate_rows("events", events.len(), MAX_TICK_ROWS_V1)?;
    validate_rows("receipts", receipts.len(), MAX_TICK_ROWS_V1)?;
    for (_, payload) in events.iter().take(MAX_TICK_ROWS_V1 + 1) {
        validate_rows("event payload items", payload.len(), MAX_TICK_ROWS_V1)?;
    }
    Ok(())
}

fn tick_aggregate_rows(
    outcomes: &[(String, usize)],
    events: &[(String, Vec<(String, Value)>)],
    receipts: &[AuditReceipt],
) -> Result<usize, IdentityCodecError> {
    let mut total = checked_add("tick BSL aggregate rows", outcomes.len(), events.len())?;
    total = checked_add("tick BSL aggregate rows", total, receipts.len())?;
    for (_, payload) in events.iter().take(MAX_TICK_ROWS_V1 + 1) {
        total = checked_add("tick BSL aggregate rows", total, payload.len())?;
    }
    Ok(total)
}

fn encode_outcomes(outcomes: &[(String, usize)]) -> Result<Vec<u8>, IdentityCodecError> {
    let mut output = IdentityWriter::new("tick rule outcomes");
    output.extend(&checked_u32("rule outcome count", outcomes.len())?.to_be_bytes())?;
    for (rule, fired) in outcomes.iter().take(MAX_TICK_RULE_OUTCOMES_V1 + 1) {
        validate_qname("rule outcome qname", rule)?;
        output.str32("rule outcome qname", rule)?;
        let fired = u64::try_from(*fired)
            .map_err(|_| IdentityCodecError::FiredCountOverflow { value: *fired })?;
        output.extend(&fired.to_be_bytes())?;
    }
    Ok(output.finish())
}

fn encode_events(
    events: &[(String, Vec<(String, Value)>)],
    resolver: &StableElementResolverV1,
) -> Result<Vec<u8>, IdentityCodecError> {
    let mut output = IdentityWriter::new("tick events");
    output.extend(&checked_u32("event count", events.len())?.to_be_bytes())?;
    for (event, payload) in events.iter().take(MAX_TICK_ROWS_V1 + 1) {
        let canonical = canonical_event_name_v1(event)?;
        output.str32("event name", &canonical)?;
        output.extend(&checked_u32("event payload count", payload.len())?.to_be_bytes())?;
        for (label, value) in payload.iter().take(MAX_TICK_ROWS_V1 + 1) {
            validate_symbol("event payload label", label)?;
            output.str32("event payload label", label)?;
            let mut encoded = Vec::new();
            crate::identity_codec::encode_value_v1(value, resolver, &mut encoded)?;
            output.extend(&encoded)?;
        }
    }
    Ok(output.finish())
}

fn encode_receipts(receipts: &[AuditReceipt]) -> Result<Vec<u8>, IdentityCodecError> {
    let mut output = IdentityWriter::new("tick receipts");
    output.extend(&checked_u32("receipt count", receipts.len())?.to_be_bytes())?;
    for receipt in receipts.iter().take(MAX_TICK_ROWS_V1 + 1) {
        validate_qname("receipt rule qname", &receipt.rule_id)?;
        output.str32("receipt rule qname", &receipt.rule_id)?;
        output.push(encode_rule_role_v1(receipt.role))?;
        output.push(encode_evidence_class_v1(receipt.evidence))?;
        output.extend(&receipt.ordinal.to_be_bytes())?;
        let mut effect = Vec::new();
        encode_effect_signature_v1(&receipt.effect, &mut effect)?;
        output.extend(&effect)?;
    }
    Ok(output.finish())
}

fn validate_rows(
    section: &'static str,
    actual: usize,
    maximum: usize,
) -> Result<(), IdentityCodecError> {
    if actual <= maximum {
        Ok(())
    } else {
        Err(IdentityCodecError::RowLimit {
            section,
            actual,
            maximum,
        })
    }
}

fn validate_aggregate(actual: usize, maximum: usize) -> Result<(), IdentityCodecError> {
    if actual <= maximum {
        Ok(())
    } else {
        Err(IdentityCodecError::AggregateRowLimit { actual, maximum })
    }
}

fn validate_combined_bytes(sections: &[&[u8]], maximum: usize) -> Result<(), IdentityCodecError> {
    let mut total = 0usize;
    for section in sections.iter().take(5) {
        total = checked_add("combined BSL identity sections", total, section.len())?;
    }
    if total <= maximum {
        Ok(())
    } else {
        Err(IdentityCodecError::ByteLimit {
            field: "combined BSL identity sections",
            actual: total,
            maximum,
        })
    }
}

const fn enum_kinds() -> [EnumKind; 4] {
    [
        EnumKind::NodeType,
        EnumKind::EdgeType,
        EnumKind::HyperedgeType,
        EnumKind::EventType,
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        validate_aggregate, validate_rows, MAX_PREPARED_AGGREGATE_ROWS_V1,
        MAX_PREPARED_ENUM_MEMBERS_V1, MAX_PREPARED_ROWS_V1, MAX_PREPARED_SMALL_ROWS_V1,
        MAX_PREPARED_VOCABULARY_MEMBERS_V1, MAX_TICK_AGGREGATE_ROWS_V1, MAX_TICK_ROWS_V1,
        MAX_TICK_RULE_OUTCOMES_V1,
    };
    use crate::identity_codec::{
        validate_governance_text, IdentityCodecError, MAX_GOVERNANCE_UTF8_BYTES_V1,
    };

    #[test]
    fn every_bsl_section_ceiling_accepts_maximum_and_refuses_plus_one() {
        for (section, maximum) in [
            ("prepared", MAX_PREPARED_ROWS_V1),
            ("prepared small", MAX_PREPARED_SMALL_ROWS_V1),
            ("enum members", MAX_PREPARED_ENUM_MEMBERS_V1),
            ("vocabulary members", MAX_PREPARED_VOCABULARY_MEMBERS_V1),
            ("rule outcomes", MAX_TICK_RULE_OUTCOMES_V1),
            ("tick rows", MAX_TICK_ROWS_V1),
        ] {
            assert_eq!(validate_rows(section, maximum, maximum), Ok(()));
            assert_eq!(
                validate_rows(section, maximum + 1, maximum),
                Err(IdentityCodecError::RowLimit {
                    section,
                    actual: maximum + 1,
                    maximum,
                })
            );
        }
        for maximum in [MAX_PREPARED_AGGREGATE_ROWS_V1, MAX_TICK_AGGREGATE_ROWS_V1] {
            assert_eq!(validate_aggregate(maximum, maximum), Ok(()));
            assert_eq!(
                validate_aggregate(maximum + 1, maximum),
                Err(IdentityCodecError::AggregateRowLimit {
                    actual: maximum + 1,
                    maximum,
                })
            );
        }
    }

    #[test]
    fn governance_utf8_accepts_exact_bytes_and_refuses_plus_one() {
        let maximum = "é".repeat(MAX_GOVERNANCE_UTF8_BYTES_V1 / 2);
        assert_eq!(validate_governance_text("governance", &maximum), Ok(()));
        let oversized = format!("{maximum}x");
        assert_eq!(
            validate_governance_text("governance", &oversized),
            Err(IdentityCodecError::InvalidString {
                field: "governance",
                index: MAX_GOVERNANCE_UTF8_BYTES_V1 + 1,
            })
        );
    }
}
