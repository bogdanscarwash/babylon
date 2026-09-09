//! Pure V2 strike-proposal eligibility and independent-participation boundary.

use std::collections::BTreeSet;

use babylon_kernel::sha256_of;

use crate::actor_v2::ActorOrganizationIdV2;
use crate::{
    practice_proposal_key_v2, resolved_practice_batch_v2_digest, InputAuthorityIdV2, PracticeIdV2,
    PracticeInputAuthorityLedgerV2, PracticeIntentV2, PracticeProposalKeyV2,
    PracticeTargetIdentityV2, PracticeTargetTagV2, ProposalNonceV2, ResolvedPracticeBatchV2,
    ResolvedPracticeBatchV2Error, TaggedPracticeTargetV2, MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2,
};

const SCHEMA_VERSION: u16 = 2;

/// Canonical domain for the frozen V2 strike-proposal law.
pub const STRIKE_PROPOSAL_CONTRACT_V2_DOMAIN_BYTES: &[u8] = b"babylon.strike-proposal-contract.v2";
/// Canonical domain for one validated V2 labor-process register.
pub const STRIKE_LABOR_PROCESS_REGISTER_V2_DOMAIN_BYTES: &[u8] =
    b"babylon.strike-labor-process-register.v2";
/// Canonical domain for one admitted V2 strike proposal.
pub const ADMITTED_STRIKE_PROPOSAL_V2_DOMAIN_BYTES: &[u8] = b"babylon.admitted-strike-proposal.v2";
/// SHA-256 of the exact language-neutral V2 strike-proposal schema bytes.
pub const STRIKE_PROPOSAL_V2_SOURCE_SHA256: [u8; 32] = [
    0xfe, 0x85, 0x53, 0x58, 0xc6, 0x49, 0xae, 0x82, 0xce, 0x1e, 0xf1, 0xbf, 0x7e, 0x04, 0x91, 0x5d,
    0x64, 0xcd, 0x59, 0xb9, 0x0b, 0x41, 0xa7, 0xc4, 0xb7, 0xab, 0xdb, 0x91, 0x3b, 0x90, 0xd3, 0x07,
];
/// Designed validation and serialization ceiling, not a worker or organization quota.
pub const MAX_STRIKE_AFFECTED_COHORTS_V2: usize = 65_536;
/// Designed validation and serialization ceiling, not a worker or organization quota.
pub const MAX_STRIKE_ORGANIZATION_RELATIONS_V2: usize = 65_536;

/// Exact strike-proposal contract failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum StrikeProposalV2Error {
    StrikeDomain = 1,
    StrikeSchemaVersion = 2,
    StrikeEnumCode = 3,
    StrikeTruncated = 4,
    StrikeTrailingBytes = 5,
    StrikeContractValue = 6,
    StrikePracticeMismatch = 7,
    StrikeResolveTickMismatch = 8,
    StrikeContentDigestMismatch = 9,
    StrikeAffectedCohortLimit = 10,
    StrikeAffectedCohortOrder = 11,
    StrikeAffectedCohortDuplicate = 12,
    StrikeOrganizationRelationLimit = 13,
    StrikeOrganizationRelationOrder = 14,
    StrikeOrganizationRelationDuplicate = 15,
    StrikeRelationCohortMissing = 16,
    StrikeTargetNoAffectedCohort = 17,
    StrikeOrganizationNotConnected = 18,
    StrikeProposalNotAccepted = 19,
    StrikeAdmissionContractDigest = 20,
    StrikeAdmissionBatchDigest = 21,
    StrikeAdmissionRegisterDigest = 22,
    StrikeAdmissionMismatch = 23,
}

/// Unknown V2 strike-proposal error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownStrikeProposalV2ErrorCode(pub u16);

impl TryFrom<u16> for StrikeProposalV2Error {
    type Error = UnknownStrikeProposalV2ErrorCode;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::StrikeDomain),
            2 => Ok(Self::StrikeSchemaVersion),
            3 => Ok(Self::StrikeEnumCode),
            4 => Ok(Self::StrikeTruncated),
            5 => Ok(Self::StrikeTrailingBytes),
            6 => Ok(Self::StrikeContractValue),
            7 => Ok(Self::StrikePracticeMismatch),
            8 => Ok(Self::StrikeResolveTickMismatch),
            9 => Ok(Self::StrikeContentDigestMismatch),
            10 => Ok(Self::StrikeAffectedCohortLimit),
            11 => Ok(Self::StrikeAffectedCohortOrder),
            12 => Ok(Self::StrikeAffectedCohortDuplicate),
            13 => Ok(Self::StrikeOrganizationRelationLimit),
            14 => Ok(Self::StrikeOrganizationRelationOrder),
            15 => Ok(Self::StrikeOrganizationRelationDuplicate),
            16 => Ok(Self::StrikeRelationCohortMissing),
            17 => Ok(Self::StrikeTargetNoAffectedCohort),
            18 => Ok(Self::StrikeOrganizationNotConnected),
            19 => Ok(Self::StrikeProposalNotAccepted),
            20 => Ok(Self::StrikeAdmissionContractDigest),
            21 => Ok(Self::StrikeAdmissionBatchDigest),
            22 => Ok(Self::StrikeAdmissionRegisterDigest),
            23 => Ok(Self::StrikeAdmissionMismatch),
            _ => Err(UnknownStrikeProposalV2ErrorCode(value)),
        }
    }
}

impl From<StrikeProposalV2Error> for u16 {
    fn from(value: StrikeProposalV2Error) -> Self {
        value as Self
    }
}

/// Lossless authoritative-batch or strike-specific refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveStrikeProposalV2Error {
    Batch(ResolvedPracticeBatchV2Error),
    Strike(StrikeProposalV2Error),
}

impl From<ResolvedPracticeBatchV2Error> for ResolveStrikeProposalV2Error {
    fn from(value: ResolvedPracticeBatchV2Error) -> Self {
        Self::Batch(value)
    }
}

impl From<StrikeProposalV2Error> for ResolveStrikeProposalV2Error {
    fn from(value: StrikeProposalV2Error) -> Self {
        Self::Strike(value)
    }
}

/// Stable identity of one worker cohort affected by a labor process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct StrikeWorkerCohortIdentityV2([u8; 32]);

impl StrikeWorkerCohortIdentityV2 {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(self) -> [u8; 32] {
        self.0
    }
}

/// One labor relation that makes a worker cohort affected by a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrikeAffectedWorkerCohortV2 {
    pub labor_process_id: PracticeTargetIdentityV2,
    pub worker_cohort_id: StrikeWorkerCohortIdentityV2,
    pub labor_relation_digest: [u8; 32],
}

/// Attributed organization membership intersecting one affected worker cohort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrikeWorkerOrganizationRelationV2 {
    pub labor_process_id: PracticeTargetIdentityV2,
    pub worker_cohort_id: StrikeWorkerCohortIdentityV2,
    pub organization_id: ActorOrganizationIdV2,
    pub membership_attribution_digest: [u8; 32],
}

/// Validated current-tick labor-process evidence consumed by strike admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrikeLaborProcessRegisterV2 {
    pub schema_version: u16,
    pub resolve_tick: u64,
    pub content_digest: [u8; 32],
    pub affected_cohorts: Vec<StrikeAffectedWorkerCohortV2>,
    pub organization_relations: Vec<StrikeWorkerOrganizationRelationV2>,
}

/// Governed material-connection derivation law.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StrikeMaterialConnectionLawV2 {
    AffectedCohortAttributedMembershipIntersection = 1,
}

/// Governed participation boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StrikeParticipationLawV2 {
    IndependentPendingRows = 1,
}

/// Frozen V2 strike-proposal law.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrikeProposalContractV2 {
    pub schema_version: u16,
    pub material_connection_law: StrikeMaterialConnectionLawV2,
    pub participation_law: StrikeParticipationLawV2,
    pub max_affected_cohorts: u32,
    pub max_organization_relations: u32,
}

impl StrikeProposalContractV2 {
    #[must_use]
    pub const fn materially_connected_workers() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            material_connection_law:
                StrikeMaterialConnectionLawV2::AffectedCohortAttributedMembershipIntersection,
            participation_law: StrikeParticipationLawV2::IndependentPendingRows,
            max_affected_cohorts: 65_536,
            max_organization_relations: 65_536,
        }
    }
}

/// Affected-worker state produced by proposal admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StrikeParticipationStateV2 {
    PendingIndependentResolution = 1,
}

/// One affected cohort awaiting its own governed participation resolver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrikeParticipationRowV2 {
    worker_cohort_id: StrikeWorkerCohortIdentityV2,
    state: StrikeParticipationStateV2,
}

impl StrikeParticipationRowV2 {
    #[must_use]
    pub const fn worker_cohort_id(&self) -> StrikeWorkerCohortIdentityV2 {
        self.worker_cohort_id
    }

    #[must_use]
    pub const fn state(&self) -> StrikeParticipationStateV2 {
        self.state
    }
}

/// Admitted proposal identity with no participation or withholding decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedStrikeProposalV2 {
    proposal_key: PracticeProposalKeyV2,
    resolved_practice_batch_digest: [u8; 32],
    labor_process_register_digest: [u8; 32],
    participation_rows: Vec<StrikeParticipationRowV2>,
}

impl AdmittedStrikeProposalV2 {
    #[must_use]
    pub const fn proposal_key(&self) -> PracticeProposalKeyV2 {
        self.proposal_key
    }

    #[must_use]
    pub const fn resolved_practice_batch_digest(&self) -> [u8; 32] {
        self.resolved_practice_batch_digest
    }

    #[must_use]
    pub const fn labor_process_register_digest(&self) -> [u8; 32] {
        self.labor_process_register_digest
    }

    #[must_use]
    pub fn participation_rows(&self) -> &[StrikeParticipationRowV2] {
        &self.participation_rows
    }
}

fn validate_contract(value: &StrikeProposalContractV2) -> Result<(), StrikeProposalV2Error> {
    if value.schema_version != SCHEMA_VERSION {
        return Err(StrikeProposalV2Error::StrikeSchemaVersion);
    }
    if value != &StrikeProposalContractV2::materially_connected_workers() {
        return Err(StrikeProposalV2Error::StrikeContractValue);
    }
    Ok(())
}

fn affected_key(
    value: &StrikeAffectedWorkerCohortV2,
) -> (PracticeTargetIdentityV2, StrikeWorkerCohortIdentityV2) {
    (value.labor_process_id, value.worker_cohort_id)
}

fn relation_key(
    value: &StrikeWorkerOrganizationRelationV2,
) -> (
    PracticeTargetIdentityV2,
    StrikeWorkerCohortIdentityV2,
    ActorOrganizationIdV2,
) {
    (
        value.labor_process_id,
        value.worker_cohort_id,
        value.organization_id,
    )
}

fn validate_affected_cohorts(
    rows: &[StrikeAffectedWorkerCohortV2],
) -> Result<BTreeSet<(PracticeTargetIdentityV2, StrikeWorkerCohortIdentityV2)>, StrikeProposalV2Error>
{
    if rows.len() > MAX_STRIKE_AFFECTED_COHORTS_V2 {
        return Err(StrikeProposalV2Error::StrikeAffectedCohortLimit);
    }
    let mut keys = BTreeSet::new();
    let mut previous = None;
    for row in rows.iter().take(MAX_STRIKE_AFFECTED_COHORTS_V2 + 1) {
        let key = affected_key(row);
        if previous == Some(key) {
            return Err(StrikeProposalV2Error::StrikeAffectedCohortDuplicate);
        }
        if previous.is_some_and(|prior| key < prior) {
            return Err(StrikeProposalV2Error::StrikeAffectedCohortOrder);
        }
        keys.insert(key);
        previous = Some(key);
    }
    Ok(keys)
}

fn validate_organization_relations(
    rows: &[StrikeWorkerOrganizationRelationV2],
    affected_keys: &BTreeSet<(PracticeTargetIdentityV2, StrikeWorkerCohortIdentityV2)>,
) -> Result<(), StrikeProposalV2Error> {
    if rows.len() > MAX_STRIKE_ORGANIZATION_RELATIONS_V2 {
        return Err(StrikeProposalV2Error::StrikeOrganizationRelationLimit);
    }
    let mut previous = None;
    for row in rows.iter().take(MAX_STRIKE_ORGANIZATION_RELATIONS_V2 + 1) {
        let key = relation_key(row);
        if previous == Some(key) {
            return Err(StrikeProposalV2Error::StrikeOrganizationRelationDuplicate);
        }
        if previous.is_some_and(|prior| key < prior) {
            return Err(StrikeProposalV2Error::StrikeOrganizationRelationOrder);
        }
        if !affected_keys.contains(&(row.labor_process_id, row.worker_cohort_id)) {
            return Err(StrikeProposalV2Error::StrikeRelationCohortMissing);
        }
        previous = Some(key);
    }
    Ok(())
}

/// Validate a V2 labor-process register.
///
/// # Errors
/// Returns the first exact schema, bound, order, duplicate, or reference refusal.
pub fn validate_strike_labor_process_register_v2(
    value: &StrikeLaborProcessRegisterV2,
) -> Result<(), StrikeProposalV2Error> {
    if value.schema_version != SCHEMA_VERSION {
        return Err(StrikeProposalV2Error::StrikeSchemaVersion);
    }
    let affected_keys = validate_affected_cohorts(&value.affected_cohorts)?;
    validate_organization_relations(&value.organization_relations, &affected_keys)
}

/// Encode the frozen V2 strike-proposal law.
///
/// # Errors
/// Returns an exact schema or governed-value refusal.
pub fn encode_strike_proposal_contract_v2(
    value: &StrikeProposalContractV2,
) -> Result<Vec<u8>, StrikeProposalV2Error> {
    validate_contract(value)?;
    let mut output = Vec::with_capacity(STRIKE_PROPOSAL_CONTRACT_V2_DOMAIN_BYTES.len() + 13);
    output.extend_from_slice(STRIKE_PROPOSAL_CONTRACT_V2_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&value.schema_version.to_be_bytes());
    output.push(value.material_connection_law as u8);
    output.push(value.participation_law as u8);
    output.extend_from_slice(&value.max_affected_cohorts.to_be_bytes());
    output.extend_from_slice(&value.max_organization_relations.to_be_bytes());
    Ok(output)
}

/// Hash the validated V2 strike-proposal law.
///
/// # Errors
/// Returns the exact contract refusal without publishing a digest.
pub fn strike_proposal_contract_v2_digest(
    value: &StrikeProposalContractV2,
) -> Result<[u8; 32], StrikeProposalV2Error> {
    Ok(sha256_of(&encode_strike_proposal_contract_v2(value)?))
}

struct StrikeCursor<'a> {
    payload: &'a [u8],
    index: usize,
}

impl<'a> StrikeCursor<'a> {
    const fn new(payload: &'a [u8]) -> Self {
        Self { payload, index: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], StrikeProposalV2Error> {
        let end = self
            .index
            .checked_add(count)
            .ok_or(StrikeProposalV2Error::StrikeTruncated)?;
        let value = self
            .payload
            .get(self.index..end)
            .ok_or(StrikeProposalV2Error::StrikeTruncated)?;
        self.index = end;
        Ok(value)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], StrikeProposalV2Error> {
        self.take(N)?
            .try_into()
            .map_err(|_| StrikeProposalV2Error::StrikeTruncated)
    }

    fn u8(&mut self) -> Result<u8, StrikeProposalV2Error> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, StrikeProposalV2Error> {
        Ok(u16::from_be_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, StrikeProposalV2Error> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, StrikeProposalV2Error> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    fn finish(self) -> Result<(), StrikeProposalV2Error> {
        if self.index == self.payload.len() {
            Ok(())
        } else {
            Err(StrikeProposalV2Error::StrikeTrailingBytes)
        }
    }
}

fn validate_domain(
    cursor: &mut StrikeCursor<'_>,
    domain: &[u8],
) -> Result<(), StrikeProposalV2Error> {
    if cursor.take(domain.len())? != domain || cursor.take(1)? != [0] {
        return Err(StrikeProposalV2Error::StrikeDomain);
    }
    Ok(())
}

/// Decode the exact frozen V2 strike-proposal law.
///
/// # Errors
/// Returns the first exact wire, enum, schema, or governed-value refusal.
pub fn decode_strike_proposal_contract_v2(
    payload: &[u8],
) -> Result<StrikeProposalContractV2, StrikeProposalV2Error> {
    let mut cursor = StrikeCursor::new(payload);
    validate_domain(&mut cursor, STRIKE_PROPOSAL_CONTRACT_V2_DOMAIN_BYTES)?;
    let schema_version = cursor.u16()?;
    let material_connection_law = match cursor.u8()? {
        1 => StrikeMaterialConnectionLawV2::AffectedCohortAttributedMembershipIntersection,
        _ => return Err(StrikeProposalV2Error::StrikeEnumCode),
    };
    let participation_law = match cursor.u8()? {
        1 => StrikeParticipationLawV2::IndependentPendingRows,
        _ => return Err(StrikeProposalV2Error::StrikeEnumCode),
    };
    let value = StrikeProposalContractV2 {
        schema_version,
        material_connection_law,
        participation_law,
        max_affected_cohorts: cursor.u32()?,
        max_organization_relations: cursor.u32()?,
    };
    cursor.finish()?;
    validate_contract(&value)?;
    Ok(value)
}

/// Encode one validated V2 labor-process register.
///
/// # Errors
/// Returns the first exact schema, bound, order, duplicate, or reference refusal.
pub fn encode_strike_labor_process_register_v2(
    value: &StrikeLaborProcessRegisterV2,
) -> Result<Vec<u8>, StrikeProposalV2Error> {
    validate_strike_labor_process_register_v2(value)?;
    let mut output = Vec::new();
    output.extend_from_slice(STRIKE_LABOR_PROCESS_REGISTER_V2_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&value.schema_version.to_be_bytes());
    output.extend_from_slice(&value.resolve_tick.to_be_bytes());
    output.extend_from_slice(&value.content_digest);
    let affected_count = u32::try_from(value.affected_cohorts.len())
        .map_err(|_| StrikeProposalV2Error::StrikeAffectedCohortLimit)?;
    output.extend_from_slice(&affected_count.to_be_bytes());
    for row in value
        .affected_cohorts
        .iter()
        .take(MAX_STRIKE_AFFECTED_COHORTS_V2 + 1)
    {
        output.extend_from_slice(&row.labor_process_id.as_bytes());
        output.extend_from_slice(&row.worker_cohort_id.as_bytes());
        output.extend_from_slice(&row.labor_relation_digest);
    }
    let relation_count = u32::try_from(value.organization_relations.len())
        .map_err(|_| StrikeProposalV2Error::StrikeOrganizationRelationLimit)?;
    output.extend_from_slice(&relation_count.to_be_bytes());
    for row in value
        .organization_relations
        .iter()
        .take(MAX_STRIKE_ORGANIZATION_RELATIONS_V2 + 1)
    {
        output.extend_from_slice(&row.labor_process_id.as_bytes());
        output.extend_from_slice(&row.worker_cohort_id.as_bytes());
        output.extend_from_slice(&row.organization_id.to_bytes());
        output.extend_from_slice(&row.membership_attribution_digest);
    }
    Ok(output)
}

/// Hash one validated V2 labor-process register.
///
/// # Errors
/// Returns the exact register refusal without publishing a digest.
pub fn strike_labor_process_register_v2_digest(
    value: &StrikeLaborProcessRegisterV2,
) -> Result<[u8; 32], StrikeProposalV2Error> {
    Ok(sha256_of(&encode_strike_labor_process_register_v2(value)?))
}

fn decode_affected_cohorts(
    cursor: &mut StrikeCursor<'_>,
) -> Result<Vec<StrikeAffectedWorkerCohortV2>, StrikeProposalV2Error> {
    let count = usize::try_from(cursor.u32()?)
        .map_err(|_| StrikeProposalV2Error::StrikeAffectedCohortLimit)?;
    if count > MAX_STRIKE_AFFECTED_COHORTS_V2 {
        return Err(StrikeProposalV2Error::StrikeAffectedCohortLimit);
    }
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_STRIKE_AFFECTED_COHORTS_V2 {
        if index == count {
            break;
        }
        rows.push(StrikeAffectedWorkerCohortV2 {
            labor_process_id: PracticeTargetIdentityV2::from_bytes(cursor.array()?),
            worker_cohort_id: StrikeWorkerCohortIdentityV2::from_bytes(cursor.array()?),
            labor_relation_digest: cursor.array()?,
        });
    }
    Ok(rows)
}

fn decode_organization_relations(
    cursor: &mut StrikeCursor<'_>,
) -> Result<Vec<StrikeWorkerOrganizationRelationV2>, StrikeProposalV2Error> {
    let count = usize::try_from(cursor.u32()?)
        .map_err(|_| StrikeProposalV2Error::StrikeOrganizationRelationLimit)?;
    if count > MAX_STRIKE_ORGANIZATION_RELATIONS_V2 {
        return Err(StrikeProposalV2Error::StrikeOrganizationRelationLimit);
    }
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_STRIKE_ORGANIZATION_RELATIONS_V2 {
        if index == count {
            break;
        }
        rows.push(StrikeWorkerOrganizationRelationV2 {
            labor_process_id: PracticeTargetIdentityV2::from_bytes(cursor.array()?),
            worker_cohort_id: StrikeWorkerCohortIdentityV2::from_bytes(cursor.array()?),
            organization_id: ActorOrganizationIdV2::from_bytes(cursor.array()?),
            membership_attribution_digest: cursor.array()?,
        });
    }
    Ok(rows)
}

/// Decode one validated V2 labor-process register.
///
/// # Errors
/// Returns the first exact wire, schema, bound, order, duplicate, or reference refusal.
pub fn decode_strike_labor_process_register_v2(
    payload: &[u8],
) -> Result<StrikeLaborProcessRegisterV2, StrikeProposalV2Error> {
    let mut cursor = StrikeCursor::new(payload);
    validate_domain(&mut cursor, STRIKE_LABOR_PROCESS_REGISTER_V2_DOMAIN_BYTES)?;
    let value = StrikeLaborProcessRegisterV2 {
        schema_version: cursor.u16()?,
        resolve_tick: cursor.u64()?,
        content_digest: cursor.array()?,
        affected_cohorts: decode_affected_cohorts(&mut cursor)?,
        organization_relations: decode_organization_relations(&mut cursor)?,
    };
    cursor.finish()?;
    validate_strike_labor_process_register_v2(&value)?;
    Ok(value)
}

fn participation_rows(
    target: PracticeTargetIdentityV2,
    register: &StrikeLaborProcessRegisterV2,
) -> Vec<StrikeParticipationRowV2> {
    register
        .affected_cohorts
        .iter()
        .take(MAX_STRIKE_AFFECTED_COHORTS_V2 + 1)
        .filter(|row| row.labor_process_id == target)
        .map(|row| StrikeParticipationRowV2 {
            worker_cohort_id: row.worker_cohort_id,
            state: StrikeParticipationStateV2::PendingIndependentResolution,
        })
        .collect()
}

fn organization_is_connected(
    target: PracticeTargetIdentityV2,
    organization_id: ActorOrganizationIdV2,
    register: &StrikeLaborProcessRegisterV2,
) -> bool {
    register
        .organization_relations
        .iter()
        .take(MAX_STRIKE_ORGANIZATION_RELATIONS_V2 + 1)
        .any(|row| row.labor_process_id == target && row.organization_id == organization_id)
}

fn validate_strike_target_kind(
    practice_id: PracticeIdV2,
    target_tag: PracticeTargetTagV2,
) -> Result<(), StrikeProposalV2Error> {
    if practice_id != PracticeIdV2::Strike || target_tag != PracticeTargetTagV2::LaborProcess {
        return Err(StrikeProposalV2Error::StrikePracticeMismatch);
    }
    Ok(())
}

fn admit_strike_intent_v2(
    contract: &StrikeProposalContractV2,
    intent: &PracticeIntentV2,
    register: &StrikeLaborProcessRegisterV2,
    resolved_practice_batch_digest: [u8; 32],
) -> Result<AdmittedStrikeProposalV2, ResolveStrikeProposalV2Error> {
    validate_contract(contract)?;
    validate_strike_labor_process_register_v2(register)?;
    validate_strike_target_kind(intent.practice_id, intent.target.tag)?;
    if intent.resolve_tick != register.resolve_tick {
        return Err(StrikeProposalV2Error::StrikeResolveTickMismatch.into());
    }
    if intent.quoted_content_digest != register.content_digest {
        return Err(StrikeProposalV2Error::StrikeContentDigestMismatch.into());
    }
    let rows = participation_rows(intent.target.identity, register);
    if rows.is_empty() {
        return Err(StrikeProposalV2Error::StrikeTargetNoAffectedCohort.into());
    }
    if !organization_is_connected(intent.target.identity, intent.actor_org_id, register) {
        return Err(StrikeProposalV2Error::StrikeOrganizationNotConnected.into());
    }
    Ok(AdmittedStrikeProposalV2 {
        proposal_key: practice_proposal_key_v2(intent),
        resolved_practice_batch_digest,
        labor_process_register_digest: strike_labor_process_register_v2_digest(register)?,
        participation_rows: rows,
    })
}

fn accepted_intent(
    batch: &ResolvedPracticeBatchV2,
    proposal_key: PracticeProposalKeyV2,
) -> Result<&PracticeIntentV2, StrikeProposalV2Error> {
    batch
        .items
        .iter()
        .take(MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2 + 1)
        .find(|item| practice_proposal_key_v2(&item.intent) == proposal_key)
        .map(|item| &item.intent)
        .ok_or(StrikeProposalV2Error::StrikeProposalNotAccepted)
}

/// Admit one accepted, inhabited, materially connected strike proposal.
///
/// # Errors
/// Returns the first exact batch, authority, intent, contract, register, identity,
/// or eligibility refusal.
pub fn admit_strike_proposal_v2(
    contract: &StrikeProposalContractV2,
    batch: &ResolvedPracticeBatchV2,
    authority_ledger: &PracticeInputAuthorityLedgerV2,
    proposal_key: PracticeProposalKeyV2,
    register: &StrikeLaborProcessRegisterV2,
) -> Result<AdmittedStrikeProposalV2, ResolveStrikeProposalV2Error> {
    let batch_digest = resolved_practice_batch_v2_digest(batch, authority_ledger)?;
    let intent = accepted_intent(batch, proposal_key)?;
    admit_strike_intent_v2(contract, intent, register, batch_digest)
}

fn append_proposal_key(output: &mut Vec<u8>, value: PracticeProposalKeyV2) {
    output.extend_from_slice(&value.resolve_tick.to_be_bytes());
    output.extend_from_slice(&value.input_authority_id.as_bytes());
    output.extend_from_slice(&value.actor_org_id.to_bytes());
    output.push(value.practice_id as u8);
    output.push(value.target.tag as u8);
    output.extend_from_slice(&value.target.identity.as_bytes());
    output.extend_from_slice(&value.proposal_nonce.as_bytes());
}

fn decode_proposal_key(
    cursor: &mut StrikeCursor<'_>,
) -> Result<PracticeProposalKeyV2, StrikeProposalV2Error> {
    let resolve_tick = cursor.u64()?;
    let input_authority_id = InputAuthorityIdV2::from_bytes(cursor.array()?);
    let actor_org_id = ActorOrganizationIdV2::from_bytes(cursor.array()?);
    let practice_id =
        PracticeIdV2::try_from(cursor.u8()?).map_err(|_| StrikeProposalV2Error::StrikeEnumCode)?;
    let tag = PracticeTargetTagV2::try_from(cursor.u8()?)
        .map_err(|_| StrikeProposalV2Error::StrikeEnumCode)?;
    validate_strike_target_kind(practice_id, tag)?;
    let identity = PracticeTargetIdentityV2::from_bytes(cursor.array()?);
    let proposal_nonce = ProposalNonceV2::from_bytes(cursor.array()?);
    Ok(PracticeProposalKeyV2 {
        resolve_tick,
        input_authority_id,
        actor_org_id,
        practice_id,
        target: TaggedPracticeTargetV2 { tag, identity },
        proposal_nonce,
    })
}

fn validate_admission(value: &AdmittedStrikeProposalV2) -> Result<(), StrikeProposalV2Error> {
    if value.participation_rows.len() > MAX_STRIKE_AFFECTED_COHORTS_V2 {
        return Err(StrikeProposalV2Error::StrikeAffectedCohortLimit);
    }
    let mut previous = None;
    for row in value
        .participation_rows
        .iter()
        .take(MAX_STRIKE_AFFECTED_COHORTS_V2 + 1)
    {
        if previous == Some(row.worker_cohort_id) {
            return Err(StrikeProposalV2Error::StrikeAffectedCohortDuplicate);
        }
        if previous.is_some_and(|prior| row.worker_cohort_id < prior) {
            return Err(StrikeProposalV2Error::StrikeAffectedCohortOrder);
        }
        previous = Some(row.worker_cohort_id);
    }
    Ok(())
}

/// Encode one admitted proposal without a participation or withholding result.
///
/// # Errors
/// Returns the first exact contract, register, bound, or canonical-order refusal.
pub fn encode_admitted_strike_proposal_v2(
    contract: &StrikeProposalContractV2,
    value: &AdmittedStrikeProposalV2,
) -> Result<Vec<u8>, StrikeProposalV2Error> {
    validate_contract(contract)?;
    validate_admission(value)?;
    let mut output = Vec::new();
    output.extend_from_slice(ADMITTED_STRIKE_PROPOSAL_V2_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&SCHEMA_VERSION.to_be_bytes());
    output.extend_from_slice(&strike_proposal_contract_v2_digest(contract)?);
    output.extend_from_slice(&value.resolved_practice_batch_digest);
    output.extend_from_slice(&value.labor_process_register_digest);
    append_proposal_key(&mut output, value.proposal_key);
    let count = u32::try_from(value.participation_rows.len())
        .map_err(|_| StrikeProposalV2Error::StrikeAffectedCohortLimit)?;
    output.extend_from_slice(&count.to_be_bytes());
    for row in value
        .participation_rows
        .iter()
        .take(MAX_STRIKE_AFFECTED_COHORTS_V2 + 1)
    {
        output.extend_from_slice(&row.worker_cohort_id.as_bytes());
        output.push(row.state as u8);
    }
    Ok(output)
}

#[derive(Debug, PartialEq, Eq)]
struct AdmissionIdentityV2 {
    proposal_key: PracticeProposalKeyV2,
    batch_digest: [u8; 32],
    register_digest: [u8; 32],
    rows: Vec<(StrikeWorkerCohortIdentityV2, StrikeParticipationStateV2)>,
}

fn admission_identity(value: &AdmittedStrikeProposalV2) -> AdmissionIdentityV2 {
    AdmissionIdentityV2 {
        proposal_key: value.proposal_key,
        batch_digest: value.resolved_practice_batch_digest,
        register_digest: value.labor_process_register_digest,
        rows: value
            .participation_rows
            .iter()
            .take(MAX_STRIKE_AFFECTED_COHORTS_V2 + 1)
            .map(|row| (row.worker_cohort_id, row.state))
            .collect(),
    }
}

fn decode_admission_rows(
    cursor: &mut StrikeCursor<'_>,
) -> Result<Vec<(StrikeWorkerCohortIdentityV2, StrikeParticipationStateV2)>, StrikeProposalV2Error>
{
    let count = usize::try_from(cursor.u32()?)
        .map_err(|_| StrikeProposalV2Error::StrikeAffectedCohortLimit)?;
    if count > MAX_STRIKE_AFFECTED_COHORTS_V2 {
        return Err(StrikeProposalV2Error::StrikeAffectedCohortLimit);
    }
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_STRIKE_AFFECTED_COHORTS_V2 {
        if index == count {
            break;
        }
        let cohort = StrikeWorkerCohortIdentityV2::from_bytes(cursor.array()?);
        let state = match cursor.u8()? {
            1 => StrikeParticipationStateV2::PendingIndependentResolution,
            _ => return Err(StrikeProposalV2Error::StrikeEnumCode),
        };
        rows.push((cohort, state));
    }
    Ok(rows)
}

/// Decode one admission and replay its governed eligibility inputs.
///
/// # Errors
/// Returns the first exact wire, contract, register, intent, eligibility, or identity refusal.
pub fn decode_admitted_strike_proposal_v2(
    payload: &[u8],
    contract: &StrikeProposalContractV2,
    batch: &ResolvedPracticeBatchV2,
    authority_ledger: &PracticeInputAuthorityLedgerV2,
    proposal_key: PracticeProposalKeyV2,
    register: &StrikeLaborProcessRegisterV2,
) -> Result<AdmittedStrikeProposalV2, ResolveStrikeProposalV2Error> {
    validate_contract(contract)?;
    let mut cursor = StrikeCursor::new(payload);
    validate_domain(&mut cursor, ADMITTED_STRIKE_PROPOSAL_V2_DOMAIN_BYTES)?;
    if cursor.u16()? != SCHEMA_VERSION {
        return Err(StrikeProposalV2Error::StrikeSchemaVersion.into());
    }
    if cursor.array::<32>()? != strike_proposal_contract_v2_digest(contract)? {
        return Err(StrikeProposalV2Error::StrikeAdmissionContractDigest.into());
    }
    let batch_digest = cursor.array()?;
    if batch_digest != resolved_practice_batch_v2_digest(batch, authority_ledger)? {
        return Err(StrikeProposalV2Error::StrikeAdmissionBatchDigest.into());
    }
    let register_digest = cursor.array()?;
    if register_digest != strike_labor_process_register_v2_digest(register)? {
        return Err(StrikeProposalV2Error::StrikeAdmissionRegisterDigest.into());
    }
    let actual = AdmissionIdentityV2 {
        proposal_key: decode_proposal_key(&mut cursor)?,
        batch_digest,
        register_digest,
        rows: decode_admission_rows(&mut cursor)?,
    };
    cursor.finish()?;
    let expected =
        admit_strike_proposal_v2(contract, batch, authority_ledger, proposal_key, register)?;
    if actual != admission_identity(&expected) {
        return Err(StrikeProposalV2Error::StrikeAdmissionMismatch.into());
    }
    Ok(expected)
}

/// Hash one admitted V2 strike proposal.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn admitted_strike_proposal_v2_digest(
    contract: &StrikeProposalContractV2,
    value: &AdmittedStrikeProposalV2,
) -> Result<[u8; 32], StrikeProposalV2Error> {
    Ok(sha256_of(&encode_admitted_strike_proposal_v2(
        contract, value,
    )?))
}
