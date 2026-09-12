//! Pure current strike-proposal eligibility and independent-participation boundary.

use std::collections::BTreeSet;

use babylon_kernel::content_digest::sha256_of;

use crate::actor::ActorOrganizationId;
use crate::{
    practice_proposal_key, resolved_practice_batch_digest, InputAuthorityId, PracticeId,
    PracticeInputAuthorityLedger, PracticeIntent, PracticeProposalKey, PracticeTargetIdentity,
    PracticeTargetTag, ProposalNonce, ResolvedPracticeBatch, ResolvedPracticeBatchError,
    TaggedPracticeTarget, MAX_RESOLVED_PRACTICE_BATCH_ITEMS,
};

const SCHEMA_VERSION: u16 = 2;

/// Canonical domain for the frozen current strike-proposal law.
pub const STRIKE_PROPOSAL_CONTRACT_DOMAIN_BYTES: &[u8] = b"babylon.strike-proposal-contract.v2";
/// Canonical domain for one validated current labor-process register.
pub const STRIKE_LABOR_PROCESS_REGISTER_DOMAIN_BYTES: &[u8] =
    b"babylon.strike-labor-process-register.v2";
/// Canonical domain for one admitted current strike proposal.
pub const ADMITTED_STRIKE_PROPOSAL_DOMAIN_BYTES: &[u8] = b"babylon.admitted-strike-proposal.v2";
/// SHA-256 of the exact language-neutral current strike-proposal schema bytes.
pub const STRIKE_PROPOSAL_SOURCE_SHA256: [u8; 32] = [
    0x33, 0x63, 0x82, 0x53, 0x59, 0x41, 0xff, 0x06, 0xcf, 0x47, 0x99, 0xca, 0x0a, 0x5c, 0x53, 0xf1,
    0x03, 0x49, 0x30, 0x79, 0x22, 0xf2, 0xd6, 0xa5, 0x29, 0xac, 0x91, 0x0a, 0xbe, 0x32, 0x8c, 0x4e,
];
/// Designed validation and serialization ceiling, not a worker or organization quota.
pub const MAX_STRIKE_AFFECTED_COHORTS: usize = 65_536;
/// Designed validation and serialization ceiling, not a worker or organization quota.
pub const MAX_STRIKE_ORGANIZATION_RELATIONS: usize = 65_536;

/// Exact strike-proposal contract failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum StrikeProposalError {
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

/// Unknown current strike-proposal error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownStrikeProposalErrorCode(pub u16);

impl TryFrom<u16> for StrikeProposalError {
    type Error = UnknownStrikeProposalErrorCode;

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
            _ => Err(UnknownStrikeProposalErrorCode(value)),
        }
    }
}

impl From<StrikeProposalError> for u16 {
    fn from(value: StrikeProposalError) -> Self {
        value as Self
    }
}

/// Lossless authoritative-batch or strike-specific refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveStrikeProposalError {
    Batch(ResolvedPracticeBatchError),
    Strike(StrikeProposalError),
}

impl From<ResolvedPracticeBatchError> for ResolveStrikeProposalError {
    fn from(value: ResolvedPracticeBatchError) -> Self {
        Self::Batch(value)
    }
}

impl From<StrikeProposalError> for ResolveStrikeProposalError {
    fn from(value: StrikeProposalError) -> Self {
        Self::Strike(value)
    }
}

/// Stable identity of one worker cohort affected by a labor process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct StrikeWorkerCohortIdentity([u8; 32]);

impl StrikeWorkerCohortIdentity {
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
pub struct StrikeAffectedWorkerCohort {
    pub labor_process_id: PracticeTargetIdentity,
    pub worker_cohort_id: StrikeWorkerCohortIdentity,
    pub labor_relation_digest: [u8; 32],
}

/// Attributed organization membership intersecting one affected worker cohort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrikeWorkerOrganizationRelation {
    pub labor_process_id: PracticeTargetIdentity,
    pub worker_cohort_id: StrikeWorkerCohortIdentity,
    pub organization_id: ActorOrganizationId,
    pub membership_attribution_digest: [u8; 32],
}

/// Validated current-tick labor-process evidence consumed by strike admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrikeLaborProcessRegister {
    pub schema_version: u16,
    pub resolve_tick: u64,
    pub content_digest: [u8; 32],
    pub affected_cohorts: Vec<StrikeAffectedWorkerCohort>,
    pub organization_relations: Vec<StrikeWorkerOrganizationRelation>,
}

/// Governed material-connection derivation law.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StrikeMaterialConnectionLaw {
    AffectedCohortAttributedMembershipIntersection = 1,
}

/// Governed participation boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StrikeParticipationLaw {
    IndependentPendingRows = 1,
}

/// Frozen current strike-proposal law.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrikeProposalContract {
    pub schema_version: u16,
    pub material_connection_law: StrikeMaterialConnectionLaw,
    pub participation_law: StrikeParticipationLaw,
    pub max_affected_cohorts: u32,
    pub max_organization_relations: u32,
}

impl StrikeProposalContract {
    #[must_use]
    pub const fn materially_connected_workers() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            material_connection_law:
                StrikeMaterialConnectionLaw::AffectedCohortAttributedMembershipIntersection,
            participation_law: StrikeParticipationLaw::IndependentPendingRows,
            max_affected_cohorts: 65_536,
            max_organization_relations: 65_536,
        }
    }
}

/// Affected-worker state produced by proposal admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StrikeParticipationState {
    PendingIndependentResolution = 1,
}

/// One affected cohort awaiting its own governed participation resolver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrikeParticipationRow {
    worker_cohort_id: StrikeWorkerCohortIdentity,
    state: StrikeParticipationState,
}

impl StrikeParticipationRow {
    #[must_use]
    pub const fn worker_cohort_id(&self) -> StrikeWorkerCohortIdentity {
        self.worker_cohort_id
    }

    #[must_use]
    pub const fn state(&self) -> StrikeParticipationState {
        self.state
    }
}

/// Admitted proposal identity with no participation or withholding decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedStrikeProposal {
    proposal_key: PracticeProposalKey,
    resolved_practice_batch_digest: [u8; 32],
    labor_process_register_digest: [u8; 32],
    participation_rows: Vec<StrikeParticipationRow>,
}

impl AdmittedStrikeProposal {
    #[must_use]
    pub const fn proposal_key(&self) -> PracticeProposalKey {
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
    pub fn participation_rows(&self) -> &[StrikeParticipationRow] {
        &self.participation_rows
    }
}

fn validate_contract(value: &StrikeProposalContract) -> Result<(), StrikeProposalError> {
    if value.schema_version != SCHEMA_VERSION {
        return Err(StrikeProposalError::StrikeSchemaVersion);
    }
    if value != &StrikeProposalContract::materially_connected_workers() {
        return Err(StrikeProposalError::StrikeContractValue);
    }
    Ok(())
}

fn affected_key(
    value: &StrikeAffectedWorkerCohort,
) -> (PracticeTargetIdentity, StrikeWorkerCohortIdentity) {
    (value.labor_process_id, value.worker_cohort_id)
}

fn relation_key(
    value: &StrikeWorkerOrganizationRelation,
) -> (
    PracticeTargetIdentity,
    StrikeWorkerCohortIdentity,
    ActorOrganizationId,
) {
    (
        value.labor_process_id,
        value.worker_cohort_id,
        value.organization_id,
    )
}

fn validate_affected_cohorts(
    rows: &[StrikeAffectedWorkerCohort],
) -> Result<BTreeSet<(PracticeTargetIdentity, StrikeWorkerCohortIdentity)>, StrikeProposalError> {
    if rows.len() > MAX_STRIKE_AFFECTED_COHORTS {
        return Err(StrikeProposalError::StrikeAffectedCohortLimit);
    }
    let mut keys = BTreeSet::new();
    let mut previous = None;
    for row in rows.iter().take(MAX_STRIKE_AFFECTED_COHORTS + 1) {
        let key = affected_key(row);
        if previous == Some(key) {
            return Err(StrikeProposalError::StrikeAffectedCohortDuplicate);
        }
        if previous.is_some_and(|prior| key < prior) {
            return Err(StrikeProposalError::StrikeAffectedCohortOrder);
        }
        keys.insert(key);
        previous = Some(key);
    }
    Ok(keys)
}

fn validate_organization_relations(
    rows: &[StrikeWorkerOrganizationRelation],
    affected_keys: &BTreeSet<(PracticeTargetIdentity, StrikeWorkerCohortIdentity)>,
) -> Result<(), StrikeProposalError> {
    if rows.len() > MAX_STRIKE_ORGANIZATION_RELATIONS {
        return Err(StrikeProposalError::StrikeOrganizationRelationLimit);
    }
    let mut previous = None;
    for row in rows.iter().take(MAX_STRIKE_ORGANIZATION_RELATIONS + 1) {
        let key = relation_key(row);
        if previous == Some(key) {
            return Err(StrikeProposalError::StrikeOrganizationRelationDuplicate);
        }
        if previous.is_some_and(|prior| key < prior) {
            return Err(StrikeProposalError::StrikeOrganizationRelationOrder);
        }
        if !affected_keys.contains(&(row.labor_process_id, row.worker_cohort_id)) {
            return Err(StrikeProposalError::StrikeRelationCohortMissing);
        }
        previous = Some(key);
    }
    Ok(())
}

/// Validate a current labor-process register.
///
/// # Errors
/// Returns the first exact schema, bound, order, duplicate, or reference refusal.
pub fn validate_strike_labor_process_register(
    value: &StrikeLaborProcessRegister,
) -> Result<(), StrikeProposalError> {
    if value.schema_version != SCHEMA_VERSION {
        return Err(StrikeProposalError::StrikeSchemaVersion);
    }
    let affected_keys = validate_affected_cohorts(&value.affected_cohorts)?;
    validate_organization_relations(&value.organization_relations, &affected_keys)
}

/// Encode the frozen current strike-proposal law.
///
/// # Errors
/// Returns an exact schema or governed-value refusal.
pub fn encode_strike_proposal_contract(
    value: &StrikeProposalContract,
) -> Result<Vec<u8>, StrikeProposalError> {
    validate_contract(value)?;
    let mut output = Vec::with_capacity(STRIKE_PROPOSAL_CONTRACT_DOMAIN_BYTES.len() + 13);
    output.extend_from_slice(STRIKE_PROPOSAL_CONTRACT_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&value.schema_version.to_be_bytes());
    output.push(value.material_connection_law as u8);
    output.push(value.participation_law as u8);
    output.extend_from_slice(&value.max_affected_cohorts.to_be_bytes());
    output.extend_from_slice(&value.max_organization_relations.to_be_bytes());
    Ok(output)
}

/// Hash the validated current strike-proposal law.
///
/// # Errors
/// Returns the exact contract refusal without publishing a digest.
pub fn strike_proposal_contract_digest(
    value: &StrikeProposalContract,
) -> Result<[u8; 32], StrikeProposalError> {
    Ok(sha256_of(&encode_strike_proposal_contract(value)?))
}

struct StrikeCursor<'a> {
    payload: &'a [u8],
    index: usize,
}

impl<'a> StrikeCursor<'a> {
    const fn new(payload: &'a [u8]) -> Self {
        Self { payload, index: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], StrikeProposalError> {
        let end = self
            .index
            .checked_add(count)
            .ok_or(StrikeProposalError::StrikeTruncated)?;
        let value = self
            .payload
            .get(self.index..end)
            .ok_or(StrikeProposalError::StrikeTruncated)?;
        self.index = end;
        Ok(value)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], StrikeProposalError> {
        self.take(N)?
            .try_into()
            .map_err(|_| StrikeProposalError::StrikeTruncated)
    }

    fn u8(&mut self) -> Result<u8, StrikeProposalError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, StrikeProposalError> {
        Ok(u16::from_be_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, StrikeProposalError> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, StrikeProposalError> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    fn finish(self) -> Result<(), StrikeProposalError> {
        if self.index == self.payload.len() {
            Ok(())
        } else {
            Err(StrikeProposalError::StrikeTrailingBytes)
        }
    }
}

fn validate_domain(
    cursor: &mut StrikeCursor<'_>,
    domain: &[u8],
) -> Result<(), StrikeProposalError> {
    if cursor.take(domain.len())? != domain || cursor.take(1)? != [0] {
        return Err(StrikeProposalError::StrikeDomain);
    }
    Ok(())
}

/// Decode the exact frozen current strike-proposal law.
///
/// # Errors
/// Returns the first exact wire, enum, schema, or governed-value refusal.
pub fn decode_strike_proposal_contract(
    payload: &[u8],
) -> Result<StrikeProposalContract, StrikeProposalError> {
    let mut cursor = StrikeCursor::new(payload);
    validate_domain(&mut cursor, STRIKE_PROPOSAL_CONTRACT_DOMAIN_BYTES)?;
    let schema_version = cursor.u16()?;
    let material_connection_law = match cursor.u8()? {
        1 => StrikeMaterialConnectionLaw::AffectedCohortAttributedMembershipIntersection,
        _ => return Err(StrikeProposalError::StrikeEnumCode),
    };
    let participation_law = match cursor.u8()? {
        1 => StrikeParticipationLaw::IndependentPendingRows,
        _ => return Err(StrikeProposalError::StrikeEnumCode),
    };
    let value = StrikeProposalContract {
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

/// Encode one validated current labor-process register.
///
/// # Errors
/// Returns the first exact schema, bound, order, duplicate, or reference refusal.
pub fn encode_strike_labor_process_register(
    value: &StrikeLaborProcessRegister,
) -> Result<Vec<u8>, StrikeProposalError> {
    validate_strike_labor_process_register(value)?;
    let mut output = Vec::new();
    output.extend_from_slice(STRIKE_LABOR_PROCESS_REGISTER_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&value.schema_version.to_be_bytes());
    output.extend_from_slice(&value.resolve_tick.to_be_bytes());
    output.extend_from_slice(&value.content_digest);
    let affected_count = u32::try_from(value.affected_cohorts.len())
        .map_err(|_| StrikeProposalError::StrikeAffectedCohortLimit)?;
    output.extend_from_slice(&affected_count.to_be_bytes());
    for row in value
        .affected_cohorts
        .iter()
        .take(MAX_STRIKE_AFFECTED_COHORTS + 1)
    {
        output.extend_from_slice(&row.labor_process_id.as_bytes());
        output.extend_from_slice(&row.worker_cohort_id.as_bytes());
        output.extend_from_slice(&row.labor_relation_digest);
    }
    let relation_count = u32::try_from(value.organization_relations.len())
        .map_err(|_| StrikeProposalError::StrikeOrganizationRelationLimit)?;
    output.extend_from_slice(&relation_count.to_be_bytes());
    for row in value
        .organization_relations
        .iter()
        .take(MAX_STRIKE_ORGANIZATION_RELATIONS + 1)
    {
        output.extend_from_slice(&row.labor_process_id.as_bytes());
        output.extend_from_slice(&row.worker_cohort_id.as_bytes());
        output.extend_from_slice(&row.organization_id.to_bytes());
        output.extend_from_slice(&row.membership_attribution_digest);
    }
    Ok(output)
}

/// Hash one validated current labor-process register.
///
/// # Errors
/// Returns the exact register refusal without publishing a digest.
pub fn strike_labor_process_register_digest(
    value: &StrikeLaborProcessRegister,
) -> Result<[u8; 32], StrikeProposalError> {
    Ok(sha256_of(&encode_strike_labor_process_register(value)?))
}

fn decode_affected_cohorts(
    cursor: &mut StrikeCursor<'_>,
) -> Result<Vec<StrikeAffectedWorkerCohort>, StrikeProposalError> {
    let count = usize::try_from(cursor.u32()?)
        .map_err(|_| StrikeProposalError::StrikeAffectedCohortLimit)?;
    if count > MAX_STRIKE_AFFECTED_COHORTS {
        return Err(StrikeProposalError::StrikeAffectedCohortLimit);
    }
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_STRIKE_AFFECTED_COHORTS {
        if index == count {
            break;
        }
        rows.push(StrikeAffectedWorkerCohort {
            labor_process_id: PracticeTargetIdentity::from_bytes(cursor.array()?),
            worker_cohort_id: StrikeWorkerCohortIdentity::from_bytes(cursor.array()?),
            labor_relation_digest: cursor.array()?,
        });
    }
    Ok(rows)
}

fn decode_organization_relations(
    cursor: &mut StrikeCursor<'_>,
) -> Result<Vec<StrikeWorkerOrganizationRelation>, StrikeProposalError> {
    let count = usize::try_from(cursor.u32()?)
        .map_err(|_| StrikeProposalError::StrikeOrganizationRelationLimit)?;
    if count > MAX_STRIKE_ORGANIZATION_RELATIONS {
        return Err(StrikeProposalError::StrikeOrganizationRelationLimit);
    }
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_STRIKE_ORGANIZATION_RELATIONS {
        if index == count {
            break;
        }
        rows.push(StrikeWorkerOrganizationRelation {
            labor_process_id: PracticeTargetIdentity::from_bytes(cursor.array()?),
            worker_cohort_id: StrikeWorkerCohortIdentity::from_bytes(cursor.array()?),
            organization_id: ActorOrganizationId::from_bytes(cursor.array()?),
            membership_attribution_digest: cursor.array()?,
        });
    }
    Ok(rows)
}

/// Decode one validated current labor-process register.
///
/// # Errors
/// Returns the first exact wire, schema, bound, order, duplicate, or reference refusal.
pub fn decode_strike_labor_process_register(
    payload: &[u8],
) -> Result<StrikeLaborProcessRegister, StrikeProposalError> {
    let mut cursor = StrikeCursor::new(payload);
    validate_domain(&mut cursor, STRIKE_LABOR_PROCESS_REGISTER_DOMAIN_BYTES)?;
    let value = StrikeLaborProcessRegister {
        schema_version: cursor.u16()?,
        resolve_tick: cursor.u64()?,
        content_digest: cursor.array()?,
        affected_cohorts: decode_affected_cohorts(&mut cursor)?,
        organization_relations: decode_organization_relations(&mut cursor)?,
    };
    cursor.finish()?;
    validate_strike_labor_process_register(&value)?;
    Ok(value)
}

fn participation_rows(
    target: PracticeTargetIdentity,
    register: &StrikeLaborProcessRegister,
) -> Vec<StrikeParticipationRow> {
    register
        .affected_cohorts
        .iter()
        .take(MAX_STRIKE_AFFECTED_COHORTS + 1)
        .filter(|row| row.labor_process_id == target)
        .map(|row| StrikeParticipationRow {
            worker_cohort_id: row.worker_cohort_id,
            state: StrikeParticipationState::PendingIndependentResolution,
        })
        .collect()
}

fn organization_is_connected(
    target: PracticeTargetIdentity,
    organization_id: ActorOrganizationId,
    register: &StrikeLaborProcessRegister,
) -> bool {
    register
        .organization_relations
        .iter()
        .take(MAX_STRIKE_ORGANIZATION_RELATIONS + 1)
        .any(|row| row.labor_process_id == target && row.organization_id == organization_id)
}

fn validate_strike_target_kind(
    practice_id: PracticeId,
    target_tag: PracticeTargetTag,
) -> Result<(), StrikeProposalError> {
    if practice_id != PracticeId::Strike || target_tag != PracticeTargetTag::LaborProcess {
        return Err(StrikeProposalError::StrikePracticeMismatch);
    }
    Ok(())
}

fn admit_strike_intent(
    contract: &StrikeProposalContract,
    intent: &PracticeIntent,
    register: &StrikeLaborProcessRegister,
    resolved_practice_batch_digest: [u8; 32],
) -> Result<AdmittedStrikeProposal, ResolveStrikeProposalError> {
    validate_contract(contract)?;
    validate_strike_labor_process_register(register)?;
    validate_strike_target_kind(intent.practice_id, intent.target.tag)?;
    if intent.resolve_tick != register.resolve_tick {
        return Err(StrikeProposalError::StrikeResolveTickMismatch.into());
    }
    if intent.quoted_content_digest != register.content_digest {
        return Err(StrikeProposalError::StrikeContentDigestMismatch.into());
    }
    let rows = participation_rows(intent.target.identity, register);
    if rows.is_empty() {
        return Err(StrikeProposalError::StrikeTargetNoAffectedCohort.into());
    }
    if !organization_is_connected(intent.target.identity, intent.actor_org_id, register) {
        return Err(StrikeProposalError::StrikeOrganizationNotConnected.into());
    }
    Ok(AdmittedStrikeProposal {
        proposal_key: practice_proposal_key(intent),
        resolved_practice_batch_digest,
        labor_process_register_digest: strike_labor_process_register_digest(register)?,
        participation_rows: rows,
    })
}

fn accepted_intent(
    batch: &ResolvedPracticeBatch,
    proposal_key: PracticeProposalKey,
) -> Result<&PracticeIntent, StrikeProposalError> {
    batch
        .items
        .iter()
        .take(MAX_RESOLVED_PRACTICE_BATCH_ITEMS + 1)
        .find(|item| practice_proposal_key(&item.intent) == proposal_key)
        .map(|item| &item.intent)
        .ok_or(StrikeProposalError::StrikeProposalNotAccepted)
}

/// Admit one accepted, inhabited, materially connected strike proposal.
///
/// # Errors
/// Returns the first exact batch, authority, intent, contract, register, identity,
/// or eligibility refusal.
pub fn admit_strike_proposal(
    contract: &StrikeProposalContract,
    batch: &ResolvedPracticeBatch,
    authority_ledger: &PracticeInputAuthorityLedger,
    proposal_key: PracticeProposalKey,
    register: &StrikeLaborProcessRegister,
) -> Result<AdmittedStrikeProposal, ResolveStrikeProposalError> {
    let batch_digest = resolved_practice_batch_digest(batch, authority_ledger)?;
    let intent = accepted_intent(batch, proposal_key)?;
    admit_strike_intent(contract, intent, register, batch_digest)
}

fn append_proposal_key(output: &mut Vec<u8>, value: PracticeProposalKey) {
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
) -> Result<PracticeProposalKey, StrikeProposalError> {
    let resolve_tick = cursor.u64()?;
    let input_authority_id = InputAuthorityId::from_bytes(cursor.array()?);
    let actor_org_id = ActorOrganizationId::from_bytes(cursor.array()?);
    let practice_id =
        PracticeId::try_from(cursor.u8()?).map_err(|_| StrikeProposalError::StrikeEnumCode)?;
    let tag = PracticeTargetTag::try_from(cursor.u8()?)
        .map_err(|_| StrikeProposalError::StrikeEnumCode)?;
    validate_strike_target_kind(practice_id, tag)?;
    let identity = PracticeTargetIdentity::from_bytes(cursor.array()?);
    let proposal_nonce = ProposalNonce::from_bytes(cursor.array()?);
    Ok(PracticeProposalKey {
        resolve_tick,
        input_authority_id,
        actor_org_id,
        practice_id,
        target: TaggedPracticeTarget { tag, identity },
        proposal_nonce,
    })
}

fn validate_admission(value: &AdmittedStrikeProposal) -> Result<(), StrikeProposalError> {
    if value.participation_rows.len() > MAX_STRIKE_AFFECTED_COHORTS {
        return Err(StrikeProposalError::StrikeAffectedCohortLimit);
    }
    let mut previous = None;
    for row in value
        .participation_rows
        .iter()
        .take(MAX_STRIKE_AFFECTED_COHORTS + 1)
    {
        if previous == Some(row.worker_cohort_id) {
            return Err(StrikeProposalError::StrikeAffectedCohortDuplicate);
        }
        if previous.is_some_and(|prior| row.worker_cohort_id < prior) {
            return Err(StrikeProposalError::StrikeAffectedCohortOrder);
        }
        previous = Some(row.worker_cohort_id);
    }
    Ok(())
}

/// Encode one admitted proposal without a participation or withholding result.
///
/// # Errors
/// Returns the first exact contract, register, bound, or canonical-order refusal.
pub fn encode_admitted_strike_proposal(
    contract: &StrikeProposalContract,
    value: &AdmittedStrikeProposal,
) -> Result<Vec<u8>, StrikeProposalError> {
    validate_contract(contract)?;
    validate_admission(value)?;
    let mut output = Vec::new();
    output.extend_from_slice(ADMITTED_STRIKE_PROPOSAL_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&SCHEMA_VERSION.to_be_bytes());
    output.extend_from_slice(&strike_proposal_contract_digest(contract)?);
    output.extend_from_slice(&value.resolved_practice_batch_digest);
    output.extend_from_slice(&value.labor_process_register_digest);
    append_proposal_key(&mut output, value.proposal_key);
    let count = u32::try_from(value.participation_rows.len())
        .map_err(|_| StrikeProposalError::StrikeAffectedCohortLimit)?;
    output.extend_from_slice(&count.to_be_bytes());
    for row in value
        .participation_rows
        .iter()
        .take(MAX_STRIKE_AFFECTED_COHORTS + 1)
    {
        output.extend_from_slice(&row.worker_cohort_id.as_bytes());
        output.push(row.state as u8);
    }
    Ok(output)
}

#[derive(Debug, PartialEq, Eq)]
struct AdmissionIdentity {
    proposal_key: PracticeProposalKey,
    batch_digest: [u8; 32],
    register_digest: [u8; 32],
    rows: Vec<(StrikeWorkerCohortIdentity, StrikeParticipationState)>,
}

fn admission_identity(value: &AdmittedStrikeProposal) -> AdmissionIdentity {
    AdmissionIdentity {
        proposal_key: value.proposal_key,
        batch_digest: value.resolved_practice_batch_digest,
        register_digest: value.labor_process_register_digest,
        rows: value
            .participation_rows
            .iter()
            .take(MAX_STRIKE_AFFECTED_COHORTS + 1)
            .map(|row| (row.worker_cohort_id, row.state))
            .collect(),
    }
}

fn decode_admission_rows(
    cursor: &mut StrikeCursor<'_>,
) -> Result<Vec<(StrikeWorkerCohortIdentity, StrikeParticipationState)>, StrikeProposalError> {
    let count = usize::try_from(cursor.u32()?)
        .map_err(|_| StrikeProposalError::StrikeAffectedCohortLimit)?;
    if count > MAX_STRIKE_AFFECTED_COHORTS {
        return Err(StrikeProposalError::StrikeAffectedCohortLimit);
    }
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_STRIKE_AFFECTED_COHORTS {
        if index == count {
            break;
        }
        let cohort = StrikeWorkerCohortIdentity::from_bytes(cursor.array()?);
        let state = match cursor.u8()? {
            1 => StrikeParticipationState::PendingIndependentResolution,
            _ => return Err(StrikeProposalError::StrikeEnumCode),
        };
        rows.push((cohort, state));
    }
    Ok(rows)
}

/// Decode one admission and replay its governed eligibility inputs.
///
/// # Errors
/// Returns the first exact wire, contract, register, intent, eligibility, or identity refusal.
pub fn decode_admitted_strike_proposal(
    payload: &[u8],
    contract: &StrikeProposalContract,
    batch: &ResolvedPracticeBatch,
    authority_ledger: &PracticeInputAuthorityLedger,
    proposal_key: PracticeProposalKey,
    register: &StrikeLaborProcessRegister,
) -> Result<AdmittedStrikeProposal, ResolveStrikeProposalError> {
    validate_contract(contract)?;
    let mut cursor = StrikeCursor::new(payload);
    validate_domain(&mut cursor, ADMITTED_STRIKE_PROPOSAL_DOMAIN_BYTES)?;
    if cursor.u16()? != SCHEMA_VERSION {
        return Err(StrikeProposalError::StrikeSchemaVersion.into());
    }
    if cursor.array::<32>()? != strike_proposal_contract_digest(contract)? {
        return Err(StrikeProposalError::StrikeAdmissionContractDigest.into());
    }
    let batch_digest = cursor.array()?;
    if batch_digest != resolved_practice_batch_digest(batch, authority_ledger)? {
        return Err(StrikeProposalError::StrikeAdmissionBatchDigest.into());
    }
    let register_digest = cursor.array()?;
    if register_digest != strike_labor_process_register_digest(register)? {
        return Err(StrikeProposalError::StrikeAdmissionRegisterDigest.into());
    }
    let actual = AdmissionIdentity {
        proposal_key: decode_proposal_key(&mut cursor)?,
        batch_digest,
        register_digest,
        rows: decode_admission_rows(&mut cursor)?,
    };
    cursor.finish()?;
    let expected =
        admit_strike_proposal(contract, batch, authority_ledger, proposal_key, register)?;
    if actual != admission_identity(&expected) {
        return Err(StrikeProposalError::StrikeAdmissionMismatch.into());
    }
    Ok(expected)
}

/// Hash one admitted current strike proposal.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn admitted_strike_proposal_digest(
    contract: &StrikeProposalContract,
    value: &AdmittedStrikeProposal,
) -> Result<[u8; 32], StrikeProposalError> {
    Ok(sha256_of(&encode_admitted_strike_proposal(
        contract, value,
    )?))
}
