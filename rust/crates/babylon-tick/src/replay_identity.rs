//! Exact tick-owned replay identity objects.

use std::collections::TryReserveError;

use babylon_bsl::causal_contract::AuditReceipt;
use babylon_bsl::identity_codec::{IdentityCodecError, MAX_IDENTITY_SECTION_BYTES_V1};
use babylon_bsl::identity_sections::{
    encode_prepared_bsl_sections_v1, encode_tick_payload_sections_v1,
    MAX_PREPARED_AGGREGATE_ROWS_V1, MAX_PREPARED_ROWS_V1,
};
use babylon_bsl::rules_hash_of;
use babylon_graph::stable_element::{StableElementResolverV1, StableIdentityError};
use babylon_graph::stable_state::{StableGraphStateV1, STABLE_GRAPH_STATE_LAYOUT_VERSION_V1};
use babylon_kernel::tick_content_hash::{
    PreparedEnvironmentDigestV1, StableWorldDigestV1, TickPayloadDigestV1,
};
use babylon_kernel::{sha256_of, ContentDigest};

use crate::choice_receipt::{validate_choice_receipt_order, ChoiceReceiptV1};
use crate::committed_event::CommittedEventV2;
use crate::{phase_order, PreparedRules};

/// Prepared-environment layout version.
pub const PREPARED_ENVIRONMENT_LAYOUT_VERSION_V1: u32 = 1;

/// World-register manifest layout version.
pub const WORLD_REGISTER_MANIFEST_LAYOUT_VERSION_V1: u32 = 1;
/// World-register set layout version.
pub const WORLD_REGISTER_SET_LAYOUT_VERSION_V1: u32 = 1;
/// Completed-tick register payload layout version.
pub const COMPLETED_TICK_REGISTER_LAYOUT_VERSION_V1: u32 = 1;
/// Stable-world layout version.
pub const STABLE_WORLD_LAYOUT_VERSION_V1: u32 = 1;
/// Tick-payload layout version.
pub const TICK_PAYLOAD_LAYOUT_VERSION_V2: u32 = 2;

const WORLD_REGISTER_MANIFEST_DOMAIN: &[u8] = b"babylon.world-register-manifest\0";
const WORLD_REGISTER_SET_DOMAIN: &[u8] = b"babylon.world-register-set\0";
const STABLE_WORLD_DOMAIN: &[u8] = b"babylon.stable-world\0";
const TICK_PAYLOAD_DOMAIN_V2: &[u8] = b"babylon.tick-payload.v2\0";
const PREPARED_ENVIRONMENT_DOMAIN: &[u8] = b"babylon.prepared-environment\0";
const COMPLETED_TICK_REGISTER: &str = "world/completed-tick";

/// A checked tick-owned replay identity failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayTickIdentityError {
    /// A nested BSL codec refused its semantic input.
    Bsl(IdentityCodecError),
    /// A nested stable graph codec refused its semantic input.
    Stable(StableIdentityError),
    /// A loaded rule form could not be encoded canonically.
    CanonicalRules {
        /// Human-readable canonical-AST refusal.
        message: String,
    },
    /// Loaded forms did not match the declared rules hash.
    RulesHashMismatch {
        /// Hash supplied by the content identity.
        declared: [u8; 32],
        /// Hash independently recomputed from loaded forms.
        computed: [u8; 32],
    },
    /// The governed phase schedule could not be encoded.
    PhaseSchedule {
        /// Human-readable schedule refusal.
        message: String,
    },
    /// Completed tick was outside its non-negative domain.
    NegativeCompletedTick {
        /// Refused signed tick.
        value: i64,
    },
    /// Rule outcomes did not exactly match prepared execution order.
    RuleOutcomeOrder,
    /// Committed event provenance did not match the published sink event.
    CommittedEventSinkMismatch {
        /// Zero-based committed-event ordinal.
        ordinal: usize,
    },
    /// Choice-receipt ordinals were sparse or out of publication order.
    ChoiceReceiptOrder {
        /// First received ordinal that differed from its zero-based position.
        actual: u32,
    },
    /// The checked outcome sum exceeded `usize`.
    FiredTotalOverflow,
    /// The report's derived fired total disagreed with its outcome rows.
    FiredTotalMismatch {
        /// Sum derived from outcome rows.
        derived: usize,
        /// Reported total.
        reported: usize,
    },
    /// Canonical capacity arithmetic overflowed.
    CapacityOverflow {
        /// Stable name of the object being sized.
        field: &'static str,
    },
    /// A count could not fit its governed integer lane.
    IntegerConversion {
        /// Stable name of the converted count.
        field: &'static str,
        /// Refused value.
        value: usize,
    },
    /// A governed object exceeded its row ceiling.
    RowLimit {
        /// Stable section name.
        section: &'static str,
        /// Received row count.
        actual: usize,
        /// Governed maximum.
        maximum: usize,
    },
    /// Prepared rows exceeded their aggregate ceiling.
    AggregateRowLimit {
        /// Received aggregate rows.
        actual: usize,
        /// Governed maximum.
        maximum: usize,
    },
    /// Canonical bytes exceeded the 64 MiB ceiling.
    ByteLimit {
        /// Stable name of the encoded object.
        field: &'static str,
        /// Attempted byte length.
        actual: usize,
        /// Governed maximum.
        maximum: usize,
    },
    /// A bounded canonical allocation failed.
    Allocation {
        /// Stable name of the encoded object.
        field: &'static str,
        /// Attempted byte length.
        requested: usize,
    },
}

impl From<IdentityCodecError> for ReplayTickIdentityError {
    fn from(value: IdentityCodecError) -> Self {
        Self::Bsl(value)
    }
}

impl From<StableIdentityError> for ReplayTickIdentityError {
    fn from(value: StableIdentityError) -> Self {
        Self::Stable(value)
    }
}

/// Exact canonical identity of the loaded replay mechanics environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedEnvironmentV1 {
    canonical_bytes: Vec<u8>,
    digest: PreparedEnvironmentDigestV1,
}

impl PreparedEnvironmentV1 {
    /// Borrow the exact canonical bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Return SHA-256 of the exact canonical bytes.
    #[must_use]
    pub const fn digest(&self) -> PreparedEnvironmentDigestV1 {
        self.digest
    }
}

/// Exact canonical world-register manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldRegisterManifestV1 {
    canonical_bytes: Vec<u8>,
    digest: [u8; 32],
}

impl WorldRegisterManifestV1 {
    /// Borrow the exact canonical bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Return SHA-256 of the exact canonical bytes.
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

/// Exact canonical values of the governed world-register manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldRegisterSetV1 {
    canonical_bytes: Vec<u8>,
    digest: [u8; 32],
    completed_tick: i64,
}

impl WorldRegisterSetV1 {
    /// Borrow the exact canonical bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Return SHA-256 of the exact canonical bytes.
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    /// Return the checked completed-tick register value.
    #[must_use]
    pub const fn completed_tick(&self) -> i64 {
        self.completed_tick
    }
}

/// Exact stable graph plus world-register identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StableWorldV1 {
    canonical_bytes: Vec<u8>,
    digest: StableWorldDigestV1,
}

impl StableWorldV1 {
    /// Borrow the exact canonical bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Return SHA-256 of the exact canonical bytes.
    #[must_use]
    pub const fn digest(&self) -> StableWorldDigestV1 {
        self.digest
    }
}

/// Exact governed tick observations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickPayloadV2 {
    canonical_bytes: Vec<u8>,
    digest: TickPayloadDigestV1,
    event_section_digest: [u8; 32],
    choice_receipt_section_digest: [u8; 32],
}

impl TickPayloadV2 {
    /// Borrow the exact canonical bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Return SHA-256 of the exact canonical bytes.
    #[must_use]
    pub const fn digest(&self) -> TickPayloadDigestV1 {
        self.digest
    }

    pub(crate) const fn event_section_digest(&self) -> [u8; 32] {
        self.event_section_digest
    }

    pub(crate) const fn choice_receipt_section_digest(&self) -> [u8; 32] {
        self.choice_receipt_section_digest
    }
}

/// Compose the environment identity from successfully prepared engine state.
///
/// # Errors
/// Returns when the loaded rule forms disagree with the declared content,
/// any nested identity refuses, or a governed row/byte bound is exceeded.
pub(crate) fn encode_prepared_environment_v1(
    content: &ContentDigest,
    prepared: &PreparedRules,
    resolver: &StableElementResolverV1,
    registers: &WorldRegisterManifestV1,
) -> Result<PreparedEnvironmentV1, ReplayTickIdentityError> {
    let computed = rules_hash_of(&prepared.rule_forms).map_err(|error| {
        ReplayTickIdentityError::CanonicalRules {
            message: error.message,
        }
    })?;
    if computed != content.rules_hash {
        return Err(ReplayTickIdentityError::RulesHashMismatch {
            declared: content.rules_hash,
            computed,
        });
    }
    validate_prepared_rows(prepared)?;
    let bsl = encode_prepared_bsl_sections_v1(
        &prepared.types,
        &prepared.intrinsics,
        &prepared.consts,
        &prepared.enums,
        prepared.vocabulary.as_ref(),
    )?;
    validate_prepared_aggregate(prepared, bsl.aggregate_rows())?;
    let schedule = phase_order::phase_schedule_v1().map_err(|error| {
        ReplayTickIdentityError::PhaseSchedule {
            message: error.to_string(),
        }
    })?;
    debug_assert_eq!(sha256_of(schedule.canonical_bytes()), schedule.digest());

    let mut writer = Writer::new("prepared environment");
    writer.extend(PREPARED_ENVIRONMENT_DOMAIN)?;
    writer.u32(PREPARED_ENVIRONMENT_LAYOUT_VERSION_V1)?;
    writer.push(0x01)?;
    writer.extend(&computed)?;
    writer.push(0x02)?;
    writer.u32(phase_order::PhaseScheduleV1::layout_version())?;
    writer.extend(&schedule.digest())?;
    writer.push(0x03)?;
    writer.count32("prepared rule count", prepared.rules.len())?;
    for (rule_id, _) in prepared.rules.iter().take(MAX_PREPARED_ROWS_V1 + 1) {
        writer.str32(rule_id)?;
    }
    for (tag, section) in [
        (0x04, bsl.fields_and_exemptions()),
        (0x05, bsl.intrinsic_costs()),
        (0x06, bsl.constants()),
        (0x07, bsl.enum_types()),
        (0x08, bsl.vocabulary()),
    ] {
        writer.push(tag)?;
        writer.extend(section)?;
    }
    writer.push(0x09)?;
    writer
        .u32(babylon_graph::stable_element::STABLE_ELEMENT_RESOLVER_MANIFEST_LAYOUT_VERSION_V1)?;
    writer.extend(&resolver.manifest().digest())?;
    writer.push(0x0a)?;
    writer.u32(WORLD_REGISTER_MANIFEST_LAYOUT_VERSION_V1)?;
    writer.extend(&registers.digest())?;
    let canonical_bytes = writer.finish();
    let digest = PreparedEnvironmentDigestV1::from_bytes(sha256_of(&canonical_bytes));
    Ok(PreparedEnvironmentV1 {
        canonical_bytes,
        digest,
    })
}

fn validate_prepared_rows(prepared: &PreparedRules) -> Result<(), ReplayTickIdentityError> {
    validate_row_limit("prepared rules", prepared.rules.len(), MAX_PREPARED_ROWS_V1)?;
    let resolver_rows = prepared
        .node_content_ids
        .len()
        .checked_add(prepared.hyperedge_content_ids.len())
        .ok_or(ReplayTickIdentityError::CapacityOverflow {
            field: "stable resolver rows",
        })?;
    validate_row_limit("stable resolver rows", resolver_rows, MAX_PREPARED_ROWS_V1)
}

fn validate_prepared_aggregate(
    prepared: &PreparedRules,
    bsl_rows: u32,
) -> Result<(), ReplayTickIdentityError> {
    let resolver_rows = prepared
        .node_content_ids
        .len()
        .checked_add(prepared.hyperedge_content_ids.len())
        .ok_or(ReplayTickIdentityError::CapacityOverflow {
            field: "prepared aggregate rows",
        })?;
    let total = prepared
        .rules
        .len()
        .checked_add(bsl_rows as usize)
        .and_then(|value| value.checked_add(resolver_rows))
        .and_then(|value| value.checked_add(1))
        .ok_or(ReplayTickIdentityError::CapacityOverflow {
            field: "prepared aggregate rows",
        })?;
    if total > MAX_PREPARED_AGGREGATE_ROWS_V1 {
        return Err(ReplayTickIdentityError::AggregateRowLimit {
            actual: total,
            maximum: MAX_PREPARED_AGGREGATE_ROWS_V1,
        });
    }
    Ok(())
}

fn validate_row_limit(
    section: &'static str,
    actual: usize,
    maximum: usize,
) -> Result<(), ReplayTickIdentityError> {
    if actual > maximum {
        return Err(ReplayTickIdentityError::RowLimit {
            section,
            actual,
            maximum,
        });
    }
    Ok(())
}

/// Compose the single-entry governed world-register manifest.
///
/// # Errors
/// Returns a checked arithmetic, byte-ceiling, conversion, or allocation error.
pub fn world_register_manifest_v1() -> Result<WorldRegisterManifestV1, ReplayTickIdentityError> {
    let mut writer = Writer::new("world register manifest");
    writer.extend(WORLD_REGISTER_MANIFEST_DOMAIN)?;
    writer.u32(WORLD_REGISTER_MANIFEST_LAYOUT_VERSION_V1)?;
    writer.u32(1)?;
    writer.str32(COMPLETED_TICK_REGISTER)?;
    writer.u32(COMPLETED_TICK_REGISTER_LAYOUT_VERSION_V1)?;
    let canonical_bytes = writer.finish();
    let digest = sha256_of(&canonical_bytes);
    Ok(WorldRegisterManifestV1 {
        canonical_bytes,
        digest,
    })
}

/// Encode one non-negative completed-tick register set.
///
/// # Errors
/// Returns a completed-tick domain or checked codec error.
pub fn encode_world_register_set_v1(
    manifest: &WorldRegisterManifestV1,
    completed_tick: i64,
) -> Result<WorldRegisterSetV1, ReplayTickIdentityError> {
    if completed_tick < 0 {
        return Err(ReplayTickIdentityError::NegativeCompletedTick {
            value: completed_tick,
        });
    }
    let mut writer = Writer::new("world register set");
    writer.extend(WORLD_REGISTER_SET_DOMAIN)?;
    writer.u32(WORLD_REGISTER_SET_LAYOUT_VERSION_V1)?;
    writer.push(0x01)?;
    writer.u32(WORLD_REGISTER_MANIFEST_LAYOUT_VERSION_V1)?;
    writer.extend(&manifest.digest())?;
    writer.push(0x02)?;
    writer.u32(1)?;
    writer.str32(COMPLETED_TICK_REGISTER)?;
    writer.u32(COMPLETED_TICK_REGISTER_LAYOUT_VERSION_V1)?;
    writer.u32(8)?;
    writer.extend(&completed_tick.to_be_bytes())?;
    let canonical_bytes = writer.finish();
    let digest = sha256_of(&canonical_bytes);
    Ok(WorldRegisterSetV1 {
        canonical_bytes,
        digest,
        completed_tick,
    })
}

/// Compose one stable world from exact graph and register identities.
///
/// # Errors
/// Returns a checked codec error.
pub fn encode_stable_world_v1(
    graph: &StableGraphStateV1,
    registers: &WorldRegisterSetV1,
) -> Result<StableWorldV1, ReplayTickIdentityError> {
    let mut writer = Writer::new("stable world");
    writer.extend(STABLE_WORLD_DOMAIN)?;
    writer.u32(STABLE_WORLD_LAYOUT_VERSION_V1)?;
    writer.push(0x01)?;
    writer.u32(STABLE_GRAPH_STATE_LAYOUT_VERSION_V1)?;
    writer.extend(graph.digest().as_bytes())?;
    writer.push(0x02)?;
    writer.u32(WORLD_REGISTER_SET_LAYOUT_VERSION_V1)?;
    writer.extend(&registers.digest())?;
    let canonical_bytes = writer.finish();
    let digest = StableWorldDigestV1::from_bytes(sha256_of(&canonical_bytes));
    Ok(StableWorldV1 {
        canonical_bytes,
        digest,
    })
}

/// Compose one exact tick payload from the BSL-owned semantic sections.
///
/// # Errors
/// Returns a rule-order, fired-total, nested BSL, or checked codec error.
pub fn encode_tick_payload_v2(
    expected_rule_order: &[String],
    outcomes: &[(String, usize)],
    reported_fired: usize,
    committed_events: &[CommittedEventV2],
    choice_receipts: &[ChoiceReceiptV1],
    audit_receipts: &[AuditReceipt],
    resolver: &StableElementResolverV1,
) -> Result<TickPayloadV2, ReplayTickIdentityError> {
    encode_tick_payload_with_order(
        expected_rule_order.len(),
        expected_rule_order.iter().map(String::as_str),
        TickPayloadEvidence {
            outcomes,
            reported_fired,
            committed_events,
            choice_receipts,
            audit_receipts,
            resolver,
        },
    )
}

pub(crate) fn encode_tick_payload_for_prepared_v2(
    prepared: &PreparedRules,
    outcomes: &[(String, usize)],
    reported_fired: usize,
    committed_events: &[CommittedEventV2],
    choice_receipts: &[ChoiceReceiptV1],
    audit_receipts: &[AuditReceipt],
    resolver: &StableElementResolverV1,
) -> Result<TickPayloadV2, ReplayTickIdentityError> {
    encode_tick_payload_with_order(
        prepared.rules.len(),
        prepared.rules.iter().map(|(rule_id, _)| rule_id.as_str()),
        TickPayloadEvidence {
            outcomes,
            reported_fired,
            committed_events,
            choice_receipts,
            audit_receipts,
            resolver,
        },
    )
}

struct TickPayloadEvidence<'a> {
    outcomes: &'a [(String, usize)],
    reported_fired: usize,
    committed_events: &'a [CommittedEventV2],
    choice_receipts: &'a [ChoiceReceiptV1],
    audit_receipts: &'a [AuditReceipt],
    resolver: &'a StableElementResolverV1,
}

fn encode_tick_payload_with_order<'a>(
    expected_len: usize,
    expected_rule_order: impl Iterator<Item = &'a str>,
    evidence: TickPayloadEvidence<'_>,
) -> Result<TickPayloadV2, ReplayTickIdentityError> {
    let TickPayloadEvidence {
        outcomes,
        reported_fired,
        committed_events,
        choice_receipts,
        audit_receipts,
        resolver,
    } = evidence;
    if expected_len != outcomes.len()
        || expected_rule_order
            .zip(outcomes.iter())
            .any(|(expected, (actual, _))| expected != actual)
    {
        return Err(ReplayTickIdentityError::RuleOutcomeOrder);
    }
    let derived = outcomes.iter().try_fold(0usize, |total, (_, fired)| {
        total
            .checked_add(*fired)
            .ok_or(ReplayTickIdentityError::FiredTotalOverflow)
    })?;
    if derived != reported_fired {
        return Err(ReplayTickIdentityError::FiredTotalMismatch {
            derived,
            reported: reported_fired,
        });
    }
    validate_choice_receipt_order(choice_receipts)
        .map_err(|actual| ReplayTickIdentityError::ChoiceReceiptOrder { actual })?;
    let sink_events = committed_events
        .iter()
        .map(CommittedEventV2::sink_record)
        .collect::<Vec<_>>();
    let sections =
        encode_tick_payload_sections_v1(outcomes, &sink_events, audit_receipts, resolver)?;
    let event_section = encode_committed_event_section_v2(committed_events, sections.events())?;
    let choice_receipt_section = encode_choice_receipt_section_v1(choice_receipts)?;
    let event_section_digest = sha256_of(&event_section);
    let choice_receipt_section_digest = sha256_of(&choice_receipt_section);
    let mut writer = Writer::new("tick payload");
    writer.extend(TICK_PAYLOAD_DOMAIN_V2)?;
    writer.u32(TICK_PAYLOAD_LAYOUT_VERSION_V2)?;
    for (tag, section) in [
        (0x01, sections.rule_outcomes()),
        (0x02, event_section.as_slice()),
        (0x03, sections.receipts()),
        (0x04, sections.accepted_action_outcomes()),
        (0x05, choice_receipt_section.as_slice()),
    ] {
        writer.push(tag)?;
        writer.extend(section)?;
    }
    let canonical_bytes = writer.finish();
    let digest = TickPayloadDigestV1::from_bytes(sha256_of(&canonical_bytes));
    Ok(TickPayloadV2 {
        canonical_bytes,
        digest,
        event_section_digest,
        choice_receipt_section_digest,
    })
}

fn encode_committed_event_section_v2(
    events: &[CommittedEventV2],
    sink_section_v1: &[u8],
) -> Result<Vec<u8>, ReplayTickIdentityError> {
    let mut writer = Writer::new("committed event section");
    writer.count32("committed event count", events.len())?;
    for event in events {
        writer.str32(event.emitting_rule())?;
        match event.choice_receipt() {
            None => writer.push(0)?,
            Some(reference) => {
                writer.push(1)?;
                writer.u32(reference.encounter_ordinal())?;
            }
        }
    }
    writer.bytes32(sink_section_v1)?;
    Ok(writer.finish())
}

fn encode_choice_receipt_section_v1(
    receipts: &[ChoiceReceiptV1],
) -> Result<Vec<u8>, ReplayTickIdentityError> {
    let mut writer = Writer::new("choice receipt section");
    writer.count32("choice receipt count", receipts.len())?;
    for receipt in receipts {
        writer.u32(receipt.encounter_ordinal())?;
        writer.str32(receipt.rule_id())?;
        writer.str32(receipt.sample())?;
        writer.u32(receipt.slot())?;
        writer.str32(receipt.outcome_enum())?;
        writer.bytes32(&receipt.stable_carrier().canonical_bytes()?)?;
        writer.count32(
            "choice carrier element count",
            receipt.active_elements().len(),
        )?;
        for element in receipt.active_elements() {
            writer.bytes32(&element.canonical_bytes()?)?;
        }
        writer.count32("choice branch count", receipt.branches().len())?;
        for branch in receipt.branches() {
            writer.str32(&branch.member)?;
            writer.u64(branch.mass.nanounits())?;
            writer.u128(branch.tickets.start)?;
            writer.u128(branch.tickets.end)?;
            writer.u128(branch.tickets.count)?;
        }
        writer.u64(receipt.draw_ticket())?;
        writer.str32(receipt.selected_outcome())?;
        writer.extend(&receipt.allocation_digest())?;
        writer.extend(&receipt.instance_digest())?;
    }
    Ok(writer.finish())
}

struct Writer {
    field: &'static str,
    bytes: Vec<u8>,
}

impl Writer {
    const fn new(field: &'static str) -> Self {
        Self {
            field,
            bytes: Vec::new(),
        }
    }

    fn push(&mut self, value: u8) -> Result<(), ReplayTickIdentityError> {
        self.extend(&[value])
    }

    fn u32(&mut self, value: u32) -> Result<(), ReplayTickIdentityError> {
        self.extend(&value.to_be_bytes())
    }

    fn u64(&mut self, value: u64) -> Result<(), ReplayTickIdentityError> {
        self.extend(&value.to_be_bytes())
    }

    fn u128(&mut self, value: u128) -> Result<(), ReplayTickIdentityError> {
        self.extend(&value.to_be_bytes())
    }

    fn str32(&mut self, value: &str) -> Result<(), ReplayTickIdentityError> {
        let length =
            u32::try_from(value.len()).map_err(|_| ReplayTickIdentityError::IntegerConversion {
                field: self.field,
                value: value.len(),
            })?;
        self.u32(length)?;
        self.extend(value.as_bytes())
    }

    fn bytes32(&mut self, value: &[u8]) -> Result<(), ReplayTickIdentityError> {
        let length =
            u32::try_from(value.len()).map_err(|_| ReplayTickIdentityError::IntegerConversion {
                field: self.field,
                value: value.len(),
            })?;
        self.u32(length)?;
        self.extend(value)
    }

    fn count32(
        &mut self,
        field: &'static str,
        value: usize,
    ) -> Result<(), ReplayTickIdentityError> {
        let value = u32::try_from(value)
            .map_err(|_| ReplayTickIdentityError::IntegerConversion { field, value })?;
        self.u32(value)
    }

    fn extend(&mut self, value: &[u8]) -> Result<(), ReplayTickIdentityError> {
        let requested = self
            .bytes
            .len()
            .checked_add(value.len())
            .ok_or(ReplayTickIdentityError::CapacityOverflow { field: self.field })?;
        if requested > MAX_IDENTITY_SECTION_BYTES_V1 {
            return Err(ReplayTickIdentityError::ByteLimit {
                field: self.field,
                actual: requested,
                maximum: MAX_IDENTITY_SECTION_BYTES_V1,
            });
        }
        self.bytes
            .try_reserve_exact(value.len())
            .map_err(|_: TryReserveError| ReplayTickIdentityError::Allocation {
                field: self.field,
                requested,
            })?;
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use babylon_bsl::identity_sections::encode_prepared_bsl_sections_v1;
    use babylon_bsl::rules_hash_of;
    use babylon_graph::hypergraph_store::HypergraphStore;
    use babylon_graph::stable_element::{
        StableElementResolverV1, STABLE_ELEMENT_RESOLVER_MANIFEST_LAYOUT_VERSION_V1,
    };
    use babylon_kernel::ContentDigest;

    use super::{
        encode_prepared_environment_v1, world_register_manifest_v1, ReplayTickIdentityError,
        PREPARED_ENVIRONMENT_LAYOUT_VERSION_V1, WORLD_REGISTER_MANIFEST_LAYOUT_VERSION_V1,
    };
    use crate::{phase_order, prepare_rules};

    const SCENARIO: &str =
        include_str!("../content/scenarios/vitality-lifecycle-combined-conformance.bscn");
    const RULES: &str = concat!(
        include_str!("../content/rules/lifecycle.bsl"),
        "\n",
        include_str!("../content/rules/vitality.bsl")
    );

    fn push_str32(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&u32::try_from(value.len()).unwrap().to_be_bytes());
        output.extend_from_slice(value.as_bytes());
    }

    #[test]
    fn prepared_environment_is_exact_and_recomputes_loaded_rules_hash() {
        let mut graph = HypergraphStore::new();
        let mut prepared = prepare_rules(SCENARIO, None, RULES, &mut graph).unwrap();
        let resolver = StableElementResolverV1::seal(
            &graph,
            &prepared.scenario_scope,
            &prepared.node_content_ids,
            &prepared.hyperedge_content_ids,
        )
        .unwrap();
        let rules_hash = rules_hash_of(&prepared.rule_forms).unwrap();
        let content = ContentDigest {
            defines_hash: [0x5a; 32],
            rules_hash,
        };
        let registers = world_register_manifest_v1().unwrap();
        let environment =
            encode_prepared_environment_v1(&content, &prepared, &resolver, &registers).unwrap();

        let schedule = phase_order::phase_schedule_v1().unwrap();
        let bsl = encode_prepared_bsl_sections_v1(
            &prepared.types,
            &prepared.intrinsics,
            &prepared.consts,
            &prepared.enums,
            prepared.vocabulary.as_ref(),
        )
        .unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(b"babylon.prepared-environment\0");
        expected.extend_from_slice(&PREPARED_ENVIRONMENT_LAYOUT_VERSION_V1.to_be_bytes());
        expected.push(0x01);
        expected.extend_from_slice(&rules_hash);
        expected.push(0x02);
        expected.extend_from_slice(&phase_order::PhaseScheduleV1::layout_version().to_be_bytes());
        expected.extend_from_slice(&schedule.digest());
        expected.push(0x03);
        expected.extend_from_slice(&u32::try_from(prepared.rules.len()).unwrap().to_be_bytes());
        for (rule_id, _) in &prepared.rules {
            push_str32(&mut expected, rule_id);
        }
        for (tag, section) in [
            (0x04, bsl.fields_and_exemptions()),
            (0x05, bsl.intrinsic_costs()),
            (0x06, bsl.constants()),
            (0x07, bsl.enum_types()),
            (0x08, bsl.vocabulary()),
        ] {
            expected.push(tag);
            expected.extend_from_slice(section);
        }
        expected.push(0x09);
        expected
            .extend_from_slice(&STABLE_ELEMENT_RESOLVER_MANIFEST_LAYOUT_VERSION_V1.to_be_bytes());
        expected.extend_from_slice(&resolver.manifest().digest());
        expected.push(0x0a);
        expected.extend_from_slice(&WORLD_REGISTER_MANIFEST_LAYOUT_VERSION_V1.to_be_bytes());
        expected.extend_from_slice(&registers.digest());
        assert_eq!(environment.canonical_bytes(), expected);

        let original = environment.digest();
        prepared.rules.reverse();
        let reordered =
            encode_prepared_environment_v1(&content, &prepared, &resolver, &registers).unwrap();
        assert_ne!(original, reordered.digest());

        let mismatch = ContentDigest {
            defines_hash: content.defines_hash,
            rules_hash: [0xa5; 32],
        };
        assert!(matches!(
            encode_prepared_environment_v1(&mismatch, &prepared, &resolver, &registers),
            Err(ReplayTickIdentityError::RulesHashMismatch { .. })
        ));
    }
}
