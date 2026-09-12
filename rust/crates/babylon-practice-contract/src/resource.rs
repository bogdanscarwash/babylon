//! Pure current practice-resource derivation and conservation-first allocation.

use std::collections::{BTreeMap, BTreeSet};

use babylon_kernel::content_digest::sha256_of;

use crate::actor::ActorOrganizationId;
use crate::intent::target_is_valid;
use crate::{
    practice_proposal_key, InputAuthorityId, PracticeId, PracticeIntent, PracticeProposalKey,
    PracticeTargetIdentity, PracticeTargetTag, ProposalNonce, TaggedPracticeTarget,
};

const SCHEMA_VERSION: u16 = 2;

/// Canonical domain for the frozen current allocation law.
pub const PRACTICE_RESOURCE_ALLOCATION_CONTRACT_DOMAIN_BYTES: &[u8] =
    b"babylon.practice-resource-allocation-contract.v2";
/// Canonical domain for one engine-derived current request.
pub const PRACTICE_RESOURCE_REQUEST_DOMAIN_BYTES: &[u8] = b"babylon.practice-resource-request.v2";
/// Exact byte length of one engine-derived current request.
pub const PRACTICE_RESOURCE_REQUEST_CANONICAL_BYTES: usize =
    PRACTICE_RESOURCE_REQUEST_DOMAIN_BYTES.len() + 1 + 2 + 82 + 9 + 32 + 32 + 8;
/// Canonical domain for one sealed current capacity row.
pub const PRACTICE_RESOURCE_CAPACITY_DOMAIN_BYTES: &[u8] = b"babylon.practice-resource-capacity.v2";
/// Exact byte length of one sealed current capacity row.
pub const PRACTICE_RESOURCE_CAPACITY_CANONICAL_BYTES: usize =
    PRACTICE_RESOURCE_CAPACITY_DOMAIN_BYTES.len() + 1 + 2 + 9 + 32 + 32 + 1 + 8;
/// Canonical domain for one current allocation outcome.
pub const PRACTICE_RESOURCE_ALLOCATION_OUTCOME_DOMAIN_BYTES: &[u8] =
    b"babylon.practice-resource-allocation-outcome.v2";
/// SHA-256 of the exact language-neutral current resource-allocation schema bytes.
pub const PRACTICE_RESOURCE_ALLOCATION_SOURCE_SHA256: [u8; 32] = [
    0x81, 0x98, 0x57, 0x8a, 0xa3, 0xf3, 0xed, 0xef, 0x72, 0xb5, 0x95, 0xb3, 0xd2, 0x08, 0xf6, 0xda,
    0x96, 0x50, 0x9f, 0x35, 0xb7, 0x3a, 0x41, 0xc8, 0xcc, 0x6f, 0x3c, 0x87, 0x04, 0xf9, 0xd5, 0xf8,
];

/// Designed validation and fuel ceiling, not an actor capacity or political quota.
pub const MAX_PRACTICE_RESOURCE_REQUESTS: usize = 65_536;
/// Designed per-intent serialization ceiling, not a material or political quota.
pub const MAX_PRACTICE_RESOURCE_REQUESTS_PER_INTENT: usize = 16;
/// Designed validation and fuel ceiling on distinct capacity rows.
pub const MAX_PRACTICE_RESOURCE_CAPACITIES: usize = 65_536;

/// Exact current resource-contract failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PracticeResourceError {
    ResourceDomain = 1,
    ResourceSchemaVersion = 2,
    ResourceEnumCode = 3,
    ResourceTruncated = 4,
    ResourceTrailingBytes = 5,
    ResourceContractValue = 6,
    ResourceContractDigestMismatch = 7,
    ResourceRequirementPracticeMismatch = 8,
    ResourceRequestZero = 9,
    ResourceRequestLimit = 10,
    ResourceRequestsPerIntentLimit = 11,
    ResourceRequestDuplicate = 12,
    ResourceOwnerMismatch = 13,
    ResourceCapacityLimit = 14,
    ResourceCapacityDuplicate = 15,
    ResourceCapacityMissing = 16,
    ResourceAuthorityConflict = 17,
    ResourceArithmetic = 18,
    ResourceOutcomeLimit = 19,
    ResourceOutcomeConservation = 20,
    ResourceOutcomeContractDigest = 21,
    ResourceOutcomeMismatch = 22,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownPracticeResourceErrorCode(pub u16);

impl TryFrom<u16> for PracticeResourceError {
    type Error = UnknownPracticeResourceErrorCode;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::ResourceDomain),
            2 => Ok(Self::ResourceSchemaVersion),
            3 => Ok(Self::ResourceEnumCode),
            4 => Ok(Self::ResourceTruncated),
            5 => Ok(Self::ResourceTrailingBytes),
            6 => Ok(Self::ResourceContractValue),
            7 => Ok(Self::ResourceContractDigestMismatch),
            8 => Ok(Self::ResourceRequirementPracticeMismatch),
            9 => Ok(Self::ResourceRequestZero),
            10 => Ok(Self::ResourceRequestLimit),
            11 => Ok(Self::ResourceRequestsPerIntentLimit),
            12 => Ok(Self::ResourceRequestDuplicate),
            13 => Ok(Self::ResourceOwnerMismatch),
            14 => Ok(Self::ResourceCapacityLimit),
            15 => Ok(Self::ResourceCapacityDuplicate),
            16 => Ok(Self::ResourceCapacityMissing),
            17 => Ok(Self::ResourceAuthorityConflict),
            18 => Ok(Self::ResourceArithmetic),
            19 => Ok(Self::ResourceOutcomeLimit),
            20 => Ok(Self::ResourceOutcomeConservation),
            21 => Ok(Self::ResourceOutcomeContractDigest),
            22 => Ok(Self::ResourceOutcomeMismatch),
            _ => Err(UnknownPracticeResourceErrorCode(value)),
        }
    }
}

impl From<PracticeResourceError> for u16 {
    fn from(value: PracticeResourceError) -> Self {
        value as Self
    }
}

/// Stable resource-class or resource-instance identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PracticeResourceId([u8; 32]);

impl PracticeResourceId {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(self) -> [u8; 32] {
        self.0
    }
}

/// Stable exact-quantity unit identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PracticeUnitId([u8; 32]);

impl PracticeUnitId {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(self) -> [u8; 32] {
        self.0
    }
}

/// Material owner of one resource capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PracticeResourceOwner {
    Shared,
    ActorOrganization(ActorOrganizationId),
}

/// Content-owned locator law used to derive a request owner from an intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PracticeResourceLocator {
    Shared,
    ActorOrganization,
}

/// Scarcity law governed by the capacity row, never selected by a proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PracticeResourceAllocationMode {
    DivisibleProRata = 1,
    ExclusiveAllOrNone = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PracticeResourceRequestDerivationLaw {
    SealedContent = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PracticeResourceDivisibleLaw {
    ProportionalFloor = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PracticeResourceExclusiveTieLaw {
    ContestedUnallocated = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PracticeResourceResidualLaw {
    RetainedAvailable = 1,
}

/// One sealed-content material requirement for a practice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PracticeResourceRequirement {
    pub practice_id: PracticeId,
    pub locator: PracticeResourceLocator,
    pub resource_id: PracticeResourceId,
    pub unit_id: PracticeUnitId,
    pub quantity: u64,
}

/// One request derived from an accepted intent and sealed content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeResourceRequest {
    proposal_key: PracticeProposalKey,
    owner: PracticeResourceOwner,
    resource_id: PracticeResourceId,
    unit_id: PracticeUnitId,
    requested: u64,
}

impl PracticeResourceRequest {
    #[must_use]
    pub const fn proposal_key(&self) -> PracticeProposalKey {
        self.proposal_key
    }

    #[must_use]
    pub const fn owner(&self) -> PracticeResourceOwner {
        self.owner
    }

    #[must_use]
    pub const fn resource_id(&self) -> PracticeResourceId {
        self.resource_id
    }

    #[must_use]
    pub const fn unit_id(&self) -> PracticeUnitId {
        self.unit_id
    }

    #[must_use]
    pub const fn requested(&self) -> u64 {
        self.requested
    }
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

fn append_owner(output: &mut Vec<u8>, value: PracticeResourceOwner) {
    match value {
        PracticeResourceOwner::Shared => {
            output.push(1);
            output.extend_from_slice(&[0_u8; 8]);
        }
        PracticeResourceOwner::ActorOrganization(actor_org_id) => {
            output.push(2);
            output.extend_from_slice(&actor_org_id.to_bytes());
        }
    }
}

fn validate_request(value: &PracticeResourceRequest) -> Result<(), PracticeResourceError> {
    if value.requested == 0 {
        return Err(PracticeResourceError::ResourceRequestZero);
    }
    if let PracticeResourceOwner::ActorOrganization(owner) = value.owner {
        if owner != value.proposal_key.actor_org_id {
            return Err(PracticeResourceError::ResourceOwnerMismatch);
        }
    }
    Ok(())
}

/// Encode one engine-derived current request in fixed big-endian order.
///
/// # Errors
/// Returns an exact quantity or owner refusal.
pub fn encode_practice_resource_request(
    value: &PracticeResourceRequest,
) -> Result<Vec<u8>, PracticeResourceError> {
    validate_request(value)?;
    let mut output = Vec::with_capacity(PRACTICE_RESOURCE_REQUEST_CANONICAL_BYTES);
    output.extend_from_slice(PRACTICE_RESOURCE_REQUEST_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&SCHEMA_VERSION.to_be_bytes());
    append_proposal_key(&mut output, value.proposal_key);
    append_owner(&mut output, value.owner);
    output.extend_from_slice(&value.resource_id.as_bytes());
    output.extend_from_slice(&value.unit_id.as_bytes());
    output.extend_from_slice(&value.requested.to_be_bytes());
    Ok(output)
}

/// Hash one validated engine-derived current request.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn practice_resource_request_digest(
    value: &PracticeResourceRequest,
) -> Result<[u8; 32], PracticeResourceError> {
    Ok(sha256_of(&encode_practice_resource_request(value)?))
}

/// One true available-capacity row from the sealed material snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeResourceCapacity {
    pub owner: PracticeResourceOwner,
    pub resource_id: PracticeResourceId,
    pub unit_id: PracticeUnitId,
    pub mode: PracticeResourceAllocationMode,
    pub available: u64,
}

/// Encode one sealed current capacity row in fixed big-endian order.
///
/// # Errors
/// This closed typed row has no fallible field after construction.
pub fn encode_practice_resource_capacity(
    value: &PracticeResourceCapacity,
) -> Result<Vec<u8>, PracticeResourceError> {
    let mut output = Vec::with_capacity(PRACTICE_RESOURCE_CAPACITY_CANONICAL_BYTES);
    output.extend_from_slice(PRACTICE_RESOURCE_CAPACITY_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&SCHEMA_VERSION.to_be_bytes());
    append_owner(&mut output, value.owner);
    output.extend_from_slice(&value.resource_id.as_bytes());
    output.extend_from_slice(&value.unit_id.as_bytes());
    output.push(value.mode as u8);
    output.extend_from_slice(&value.available.to_be_bytes());
    Ok(output)
}

/// Decode one complete sealed current capacity row.
///
/// # Errors
/// Returns the first exact domain, schema, enum, wire, or owner refusal.
pub fn decode_practice_resource_capacity(
    payload: &[u8],
) -> Result<PracticeResourceCapacity, PracticeResourceError> {
    let mut cursor = ContractCursor::new(payload);
    if cursor.take(PRACTICE_RESOURCE_CAPACITY_DOMAIN_BYTES.len())?
        != PRACTICE_RESOURCE_CAPACITY_DOMAIN_BYTES
        || cursor.take(1)? != [0]
    {
        return Err(PracticeResourceError::ResourceDomain);
    }
    if cursor.u16()? != SCHEMA_VERSION {
        return Err(PracticeResourceError::ResourceSchemaVersion);
    }
    let owner = decode_owner(&mut cursor)?;
    let resource_id = PracticeResourceId::from_bytes(cursor.array()?);
    let unit_id = PracticeUnitId::from_bytes(cursor.array()?);
    let mode = match cursor.u8()? {
        1 => PracticeResourceAllocationMode::DivisibleProRata,
        2 => PracticeResourceAllocationMode::ExclusiveAllOrNone,
        _ => return Err(PracticeResourceError::ResourceEnumCode),
    };
    let available = cursor.u64()?;
    cursor.finish()?;
    Ok(PracticeResourceCapacity {
        owner,
        resource_id,
        unit_id,
        mode,
        available,
    })
}

/// Hash one sealed current capacity row.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn practice_resource_capacity_digest(
    value: &PracticeResourceCapacity,
) -> Result<[u8; 32], PracticeResourceError> {
    Ok(sha256_of(&encode_practice_resource_capacity(value)?))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeResourceAllocation {
    request: PracticeResourceRequest,
    allocated: u64,
}

impl PracticeResourceAllocation {
    #[must_use]
    pub const fn request(&self) -> &PracticeResourceRequest {
        &self.request
    }

    #[must_use]
    pub const fn requested(&self) -> u64 {
        self.request.requested
    }

    #[must_use]
    pub const fn allocated(&self) -> u64 {
        self.allocated
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeResourceBalance {
    capacity: PracticeResourceCapacity,
    allocated: u64,
    unallocated: u64,
}

impl PracticeResourceBalance {
    #[must_use]
    pub const fn capacity(&self) -> &PracticeResourceCapacity {
        &self.capacity
    }

    #[must_use]
    pub const fn allocated(&self) -> u64 {
        self.allocated
    }

    #[must_use]
    pub const fn unallocated(&self) -> u64 {
        self.unallocated
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeResourceAllocationOutcome {
    allocations: Vec<PracticeResourceAllocation>,
    balances: Vec<PracticeResourceBalance>,
}

impl PracticeResourceAllocationOutcome {
    #[must_use]
    pub fn allocations(&self) -> &[PracticeResourceAllocation] {
        &self.allocations
    }

    #[must_use]
    pub fn balances(&self) -> &[PracticeResourceBalance] {
        &self.balances
    }
}

fn validate_outcome(
    value: &PracticeResourceAllocationOutcome,
) -> Result<(), PracticeResourceError> {
    if value.allocations.len() > MAX_PRACTICE_RESOURCE_REQUESTS
        || value.balances.len() > MAX_PRACTICE_RESOURCE_CAPACITIES
    {
        return Err(PracticeResourceError::ResourceOutcomeLimit);
    }
    for allocation in value
        .allocations
        .iter()
        .take(MAX_PRACTICE_RESOURCE_REQUESTS + 1)
    {
        if allocation.allocated > allocation.request.requested {
            return Err(PracticeResourceError::ResourceOutcomeConservation);
        }
    }
    for balance in value
        .balances
        .iter()
        .take(MAX_PRACTICE_RESOURCE_CAPACITIES + 1)
    {
        let total = balance
            .allocated
            .checked_add(balance.unallocated)
            .ok_or(PracticeResourceError::ResourceArithmetic)?;
        if total != balance.capacity.available {
            return Err(PracticeResourceError::ResourceOutcomeConservation);
        }
    }
    Ok(())
}

/// Encode one allocator-produced current outcome with request and capacity identities.
///
/// # Errors
/// Returns the first exact contract, bound, identity, or conservation refusal.
pub fn encode_practice_resource_allocation_outcome(
    contract: &PracticeResourceAllocationContract,
    value: &PracticeResourceAllocationOutcome,
) -> Result<Vec<u8>, PracticeResourceError> {
    validate_contract(contract)?;
    validate_outcome(value)?;
    let allocation_bytes = value
        .allocations
        .len()
        .checked_mul(40)
        .ok_or(PracticeResourceError::ResourceArithmetic)?;
    let balance_bytes = value
        .balances
        .len()
        .checked_mul(48)
        .ok_or(PracticeResourceError::ResourceArithmetic)?;
    let mut output = Vec::with_capacity(
        PRACTICE_RESOURCE_ALLOCATION_OUTCOME_DOMAIN_BYTES.len()
            + 1
            + 2
            + 32
            + 4
            + allocation_bytes
            + 4
            + balance_bytes,
    );
    output.extend_from_slice(PRACTICE_RESOURCE_ALLOCATION_OUTCOME_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&SCHEMA_VERSION.to_be_bytes());
    output.extend_from_slice(&practice_resource_allocation_contract_digest(contract)?);
    let allocation_count = u32::try_from(value.allocations.len())
        .map_err(|_| PracticeResourceError::ResourceOutcomeLimit)?;
    output.extend_from_slice(&allocation_count.to_be_bytes());
    for allocation in value
        .allocations
        .iter()
        .take(MAX_PRACTICE_RESOURCE_REQUESTS + 1)
    {
        output.extend_from_slice(&practice_resource_request_digest(&allocation.request)?);
        output.extend_from_slice(&allocation.allocated.to_be_bytes());
    }
    let balance_count = u32::try_from(value.balances.len())
        .map_err(|_| PracticeResourceError::ResourceOutcomeLimit)?;
    output.extend_from_slice(&balance_count.to_be_bytes());
    for balance in value
        .balances
        .iter()
        .take(MAX_PRACTICE_RESOURCE_CAPACITIES + 1)
    {
        output.extend_from_slice(&practice_resource_capacity_digest(&balance.capacity)?);
        output.extend_from_slice(&balance.allocated.to_be_bytes());
        output.extend_from_slice(&balance.unallocated.to_be_bytes());
    }
    Ok(output)
}

#[derive(Debug, PartialEq, Eq)]
struct PracticeResourceOutcomeIdentity {
    allocations: Vec<([u8; 32], u64)>,
    balances: Vec<([u8; 32], u64, u64)>,
}

fn outcome_identity(
    value: &PracticeResourceAllocationOutcome,
) -> Result<PracticeResourceOutcomeIdentity, PracticeResourceError> {
    let mut allocations = Vec::with_capacity(value.allocations.len());
    for allocation in value
        .allocations
        .iter()
        .take(MAX_PRACTICE_RESOURCE_REQUESTS + 1)
    {
        allocations.push((
            practice_resource_request_digest(&allocation.request)?,
            allocation.allocated,
        ));
    }
    let mut balances = Vec::with_capacity(value.balances.len());
    for balance in value
        .balances
        .iter()
        .take(MAX_PRACTICE_RESOURCE_CAPACITIES + 1)
    {
        balances.push((
            practice_resource_capacity_digest(&balance.capacity)?,
            balance.allocated,
            balance.unallocated,
        ));
    }
    Ok(PracticeResourceOutcomeIdentity {
        allocations,
        balances,
    })
}

fn decode_outcome_identity(
    cursor: &mut ContractCursor<'_>,
) -> Result<PracticeResourceOutcomeIdentity, PracticeResourceError> {
    let allocation_count =
        usize::try_from(cursor.u32()?).map_err(|_| PracticeResourceError::ResourceOutcomeLimit)?;
    if allocation_count > MAX_PRACTICE_RESOURCE_REQUESTS {
        return Err(PracticeResourceError::ResourceOutcomeLimit);
    }
    let mut allocations = Vec::with_capacity(allocation_count);
    for index in 0..=MAX_PRACTICE_RESOURCE_REQUESTS {
        if index == allocation_count {
            break;
        }
        allocations.push((cursor.array()?, cursor.u64()?));
    }
    let balance_count =
        usize::try_from(cursor.u32()?).map_err(|_| PracticeResourceError::ResourceOutcomeLimit)?;
    if balance_count > MAX_PRACTICE_RESOURCE_CAPACITIES {
        return Err(PracticeResourceError::ResourceOutcomeLimit);
    }
    let mut balances = Vec::with_capacity(balance_count);
    for index in 0..=MAX_PRACTICE_RESOURCE_CAPACITIES {
        if index == balance_count {
            break;
        }
        balances.push((cursor.array()?, cursor.u64()?, cursor.u64()?));
    }
    Ok(PracticeResourceOutcomeIdentity {
        allocations,
        balances,
    })
}

/// Decode one outcome and replay its governed allocator inputs.
///
/// # Errors
/// Returns the first exact wire, contract, input, allocation, or identity refusal.
pub fn decode_practice_resource_allocation_outcome(
    payload: &[u8],
    contract: &PracticeResourceAllocationContract,
    requests: &[PracticeResourceRequest],
    capacities: &[PracticeResourceCapacity],
) -> Result<PracticeResourceAllocationOutcome, PracticeResourceError> {
    validate_contract(contract)?;
    let mut cursor = ContractCursor::new(payload);
    if cursor.take(PRACTICE_RESOURCE_ALLOCATION_OUTCOME_DOMAIN_BYTES.len())?
        != PRACTICE_RESOURCE_ALLOCATION_OUTCOME_DOMAIN_BYTES
        || cursor.take(1)? != [0]
    {
        return Err(PracticeResourceError::ResourceDomain);
    }
    if cursor.u16()? != SCHEMA_VERSION {
        return Err(PracticeResourceError::ResourceSchemaVersion);
    }
    if cursor.array::<32>()? != practice_resource_allocation_contract_digest(contract)? {
        return Err(PracticeResourceError::ResourceOutcomeContractDigest);
    }
    let actual = decode_outcome_identity(&mut cursor)?;
    cursor.finish()?;
    let expected = allocate_practice_resources(contract, requests, capacities)?;
    validate_outcome(&expected)?;
    if actual != outcome_identity(&expected)? {
        return Err(PracticeResourceError::ResourceOutcomeMismatch);
    }
    Ok(expected)
}

/// Hash one validated current allocation outcome.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn practice_resource_allocation_outcome_digest(
    contract: &PracticeResourceAllocationContract,
    value: &PracticeResourceAllocationOutcome,
) -> Result<[u8; 32], PracticeResourceError> {
    Ok(sha256_of(&encode_practice_resource_allocation_outcome(
        contract, value,
    )?))
}

/// Frozen law identity quoted by every `PracticeIntent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PracticeResourceAllocationContract {
    pub schema_version: u16,
    pub quantity_width_bits: u8,
    pub request_derivation_law: PracticeResourceRequestDerivationLaw,
    pub divisible_law: PracticeResourceDivisibleLaw,
    pub exclusive_tie_law: PracticeResourceExclusiveTieLaw,
    pub residual_law: PracticeResourceResidualLaw,
    pub max_requests_per_intent: u16,
    pub max_requests_total: u32,
    pub max_capacities_total: u32,
}

impl PracticeResourceAllocationContract {
    #[must_use]
    pub const fn conservation_first() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            quantity_width_bits: 64,
            request_derivation_law: PracticeResourceRequestDerivationLaw::SealedContent,
            divisible_law: PracticeResourceDivisibleLaw::ProportionalFloor,
            exclusive_tie_law: PracticeResourceExclusiveTieLaw::ContestedUnallocated,
            residual_law: PracticeResourceResidualLaw::RetainedAvailable,
            max_requests_per_intent: 16,
            max_requests_total: 65_536,
            max_capacities_total: 65_536,
        }
    }
}

fn validate_contract(
    value: &PracticeResourceAllocationContract,
) -> Result<(), PracticeResourceError> {
    if value.schema_version != SCHEMA_VERSION {
        return Err(PracticeResourceError::ResourceSchemaVersion);
    }
    if value != &PracticeResourceAllocationContract::conservation_first() {
        return Err(PracticeResourceError::ResourceContractValue);
    }
    Ok(())
}

/// Encode the complete frozen current allocation law.
///
/// # Errors
/// Returns an exact schema or governed-value refusal.
pub fn encode_practice_resource_allocation_contract(
    value: &PracticeResourceAllocationContract,
) -> Result<Vec<u8>, PracticeResourceError> {
    validate_contract(value)?;
    let mut output =
        Vec::with_capacity(PRACTICE_RESOURCE_ALLOCATION_CONTRACT_DOMAIN_BYTES.len() + 18);
    output.extend_from_slice(PRACTICE_RESOURCE_ALLOCATION_CONTRACT_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&value.schema_version.to_be_bytes());
    output.push(value.quantity_width_bits);
    output.push(value.request_derivation_law as u8);
    output.push(value.divisible_law as u8);
    output.push(value.exclusive_tie_law as u8);
    output.push(value.residual_law as u8);
    output.extend_from_slice(&value.max_requests_per_intent.to_be_bytes());
    output.extend_from_slice(&value.max_requests_total.to_be_bytes());
    output.extend_from_slice(&value.max_capacities_total.to_be_bytes());
    Ok(output)
}

struct ContractCursor<'a> {
    payload: &'a [u8],
    index: usize,
}

impl<'a> ContractCursor<'a> {
    const fn new(payload: &'a [u8]) -> Self {
        Self { payload, index: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], PracticeResourceError> {
        let end = self
            .index
            .checked_add(count)
            .ok_or(PracticeResourceError::ResourceTruncated)?;
        let value = self
            .payload
            .get(self.index..end)
            .ok_or(PracticeResourceError::ResourceTruncated)?;
        self.index = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, PracticeResourceError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, PracticeResourceError> {
        Ok(u16::from_be_bytes(
            self.take(2)?
                .try_into()
                .map_err(|_| PracticeResourceError::ResourceTruncated)?,
        ))
    }

    fn u32(&mut self) -> Result<u32, PracticeResourceError> {
        Ok(u32::from_be_bytes(
            self.take(4)?
                .try_into()
                .map_err(|_| PracticeResourceError::ResourceTruncated)?,
        ))
    }

    fn u64(&mut self) -> Result<u64, PracticeResourceError> {
        Ok(u64::from_be_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| PracticeResourceError::ResourceTruncated)?,
        ))
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], PracticeResourceError> {
        self.take(N)?
            .try_into()
            .map_err(|_| PracticeResourceError::ResourceTruncated)
    }

    fn finish(&self) -> Result<(), PracticeResourceError> {
        if self.index == self.payload.len() {
            Ok(())
        } else {
            Err(PracticeResourceError::ResourceTrailingBytes)
        }
    }
}

fn decode_owner(
    cursor: &mut ContractCursor<'_>,
) -> Result<PracticeResourceOwner, PracticeResourceError> {
    let tag = cursor.u8()?;
    let actor_org_id: [u8; 8] = cursor.array()?;
    match tag {
        1 if actor_org_id == [0_u8; 8] => Ok(PracticeResourceOwner::Shared),
        1 => Err(PracticeResourceError::ResourceOwnerMismatch),
        2 => Ok(PracticeResourceOwner::ActorOrganization(
            ActorOrganizationId::from_bytes(actor_org_id),
        )),
        _ => Err(PracticeResourceError::ResourceEnumCode),
    }
}

fn decode_proposal_key(
    cursor: &mut ContractCursor<'_>,
) -> Result<PracticeProposalKey, PracticeResourceError> {
    let resolve_tick = cursor.u64()?;
    let input_authority_id = InputAuthorityId::from_bytes(cursor.array()?);
    let actor_org_id = ActorOrganizationId::from_bytes(cursor.array()?);
    let practice_id =
        PracticeId::try_from(cursor.u8()?).map_err(|_| PracticeResourceError::ResourceEnumCode)?;
    let target = TaggedPracticeTarget {
        tag: PracticeTargetTag::try_from(cursor.u8()?)
            .map_err(|_| PracticeResourceError::ResourceEnumCode)?,
        identity: PracticeTargetIdentity::from_bytes(cursor.array()?),
    };
    if !target_is_valid(practice_id, target.tag) {
        return Err(PracticeResourceError::ResourceEnumCode);
    }
    Ok(PracticeProposalKey {
        resolve_tick,
        input_authority_id,
        actor_org_id,
        practice_id,
        target,
        proposal_nonce: ProposalNonce::from_bytes(cursor.array()?),
    })
}

/// Decode one complete engine-derived current request.
///
/// # Errors
/// Returns the first exact domain, schema, enum, wire, quantity, or owner refusal.
pub fn decode_practice_resource_request(
    payload: &[u8],
) -> Result<PracticeResourceRequest, PracticeResourceError> {
    let mut cursor = ContractCursor::new(payload);
    if cursor.take(PRACTICE_RESOURCE_REQUEST_DOMAIN_BYTES.len())?
        != PRACTICE_RESOURCE_REQUEST_DOMAIN_BYTES
        || cursor.take(1)? != [0]
    {
        return Err(PracticeResourceError::ResourceDomain);
    }
    if cursor.u16()? != SCHEMA_VERSION {
        return Err(PracticeResourceError::ResourceSchemaVersion);
    }
    let value = PracticeResourceRequest {
        proposal_key: decode_proposal_key(&mut cursor)?,
        owner: decode_owner(&mut cursor)?,
        resource_id: PracticeResourceId::from_bytes(cursor.array()?),
        unit_id: PracticeUnitId::from_bytes(cursor.array()?),
        requested: cursor.u64()?,
    };
    cursor.finish()?;
    validate_request(&value)?;
    Ok(value)
}

fn law(value: u8) -> Result<(), PracticeResourceError> {
    if value == 1 {
        Ok(())
    } else {
        Err(PracticeResourceError::ResourceEnumCode)
    }
}

/// Decode the complete frozen current allocation law.
///
/// # Errors
/// Returns the first exact domain, wire, enum, schema, or governed-value refusal.
pub fn decode_practice_resource_allocation_contract(
    payload: &[u8],
) -> Result<PracticeResourceAllocationContract, PracticeResourceError> {
    let mut cursor = ContractCursor::new(payload);
    if cursor.take(PRACTICE_RESOURCE_ALLOCATION_CONTRACT_DOMAIN_BYTES.len())?
        != PRACTICE_RESOURCE_ALLOCATION_CONTRACT_DOMAIN_BYTES
        || cursor.take(1)? != [0]
    {
        return Err(PracticeResourceError::ResourceDomain);
    }
    let value = PracticeResourceAllocationContract {
        schema_version: cursor.u16()?,
        quantity_width_bits: cursor.u8()?,
        request_derivation_law: {
            law(cursor.u8()?)?;
            PracticeResourceRequestDerivationLaw::SealedContent
        },
        divisible_law: {
            law(cursor.u8()?)?;
            PracticeResourceDivisibleLaw::ProportionalFloor
        },
        exclusive_tie_law: {
            law(cursor.u8()?)?;
            PracticeResourceExclusiveTieLaw::ContestedUnallocated
        },
        residual_law: {
            law(cursor.u8()?)?;
            PracticeResourceResidualLaw::RetainedAvailable
        },
        max_requests_per_intent: cursor.u16()?,
        max_requests_total: cursor.u32()?,
        max_capacities_total: cursor.u32()?,
    };
    cursor.finish()?;
    validate_contract(&value)?;
    Ok(value)
}

/// Hash the complete validated current allocation law.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn practice_resource_allocation_contract_digest(
    value: &PracticeResourceAllocationContract,
) -> Result<[u8; 32], PracticeResourceError> {
    Ok(sha256_of(&encode_practice_resource_allocation_contract(
        value,
    )?))
}

type CapacityKey = (PracticeResourceOwner, PracticeResourceId, PracticeUnitId);

fn capacity_key(value: &PracticeResourceCapacity) -> CapacityKey {
    (value.owner, value.resource_id, value.unit_id)
}

fn request_key(value: &PracticeResourceRequest) -> CapacityKey {
    (value.owner, value.resource_id, value.unit_id)
}

fn canonical_request_key(value: &PracticeResourceRequest) -> (CapacityKey, PracticeProposalKey) {
    (request_key(value), value.proposal_key)
}

fn canonical_requests(
    requests: &[PracticeResourceRequest],
) -> Result<Vec<PracticeResourceRequest>, PracticeResourceError> {
    let mut output = requests.to_vec();
    output.sort_unstable_by_key(canonical_request_key);
    for pair in output.windows(2).take(MAX_PRACTICE_RESOURCE_REQUESTS) {
        if canonical_request_key(&pair[0]) == canonical_request_key(&pair[1]) {
            return Err(PracticeResourceError::ResourceRequestDuplicate);
        }
    }
    Ok(output)
}

fn validate_requests_per_intent(
    requests: &[PracticeResourceRequest],
) -> Result<(), PracticeResourceError> {
    let mut counts: BTreeMap<PracticeProposalKey, usize> = BTreeMap::new();
    for request in requests.iter().take(MAX_PRACTICE_RESOURCE_REQUESTS + 1) {
        let count = counts.entry(request.proposal_key).or_insert(0);
        *count = count
            .checked_add(1)
            .ok_or(PracticeResourceError::ResourceArithmetic)?;
        if *count > MAX_PRACTICE_RESOURCE_REQUESTS_PER_INTENT {
            return Err(PracticeResourceError::ResourceRequestsPerIntentLimit);
        }
    }
    Ok(())
}

/// Derive one material request from an accepted intent and sealed content.
///
/// # Errors
/// Returns an exact practice mismatch or zero-quantity refusal.
pub fn derive_practice_resource_request(
    contract: &PracticeResourceAllocationContract,
    intent: &PracticeIntent,
    requirement: &PracticeResourceRequirement,
) -> Result<PracticeResourceRequest, PracticeResourceError> {
    validate_contract(contract)?;
    if intent.quoted_resource_contract_digest
        != practice_resource_allocation_contract_digest(contract)?
    {
        return Err(PracticeResourceError::ResourceContractDigestMismatch);
    }
    if intent.practice_id != requirement.practice_id {
        return Err(PracticeResourceError::ResourceRequirementPracticeMismatch);
    }
    if requirement.quantity == 0 {
        return Err(PracticeResourceError::ResourceRequestZero);
    }
    let owner = match requirement.locator {
        PracticeResourceLocator::Shared => PracticeResourceOwner::Shared,
        PracticeResourceLocator::ActorOrganization => {
            PracticeResourceOwner::ActorOrganization(intent.actor_org_id)
        }
    };
    Ok(PracticeResourceRequest {
        proposal_key: practice_proposal_key(intent),
        owner,
        resource_id: requirement.resource_id,
        unit_id: requirement.unit_id,
        requested: requirement.quantity,
    })
}

fn capacity_index(
    capacities: &[PracticeResourceCapacity],
) -> Result<BTreeMap<CapacityKey, &PracticeResourceCapacity>, PracticeResourceError> {
    let mut output = BTreeMap::new();
    for capacity in capacities.iter().take(MAX_PRACTICE_RESOURCE_CAPACITIES + 1) {
        if output.insert(capacity_key(capacity), capacity).is_some() {
            return Err(PracticeResourceError::ResourceCapacityDuplicate);
        }
    }
    Ok(output)
}

fn request_groups(
    requests: &[PracticeResourceRequest],
    capacities: &BTreeMap<CapacityKey, &PracticeResourceCapacity>,
) -> Result<BTreeMap<CapacityKey, Vec<usize>>, PracticeResourceError> {
    let mut output: BTreeMap<CapacityKey, Vec<usize>> = BTreeMap::new();
    for (index, request) in requests
        .iter()
        .take(MAX_PRACTICE_RESOURCE_REQUESTS + 1)
        .enumerate()
    {
        let key = request_key(request);
        if !capacities.contains_key(&key) {
            return Err(PracticeResourceError::ResourceCapacityMissing);
        }
        output.entry(key).or_default().push(index);
    }
    Ok(output)
}

fn validate_authority_conflicts(
    groups: &BTreeMap<CapacityKey, Vec<usize>>,
    capacities: &BTreeMap<CapacityKey, &PracticeResourceCapacity>,
    requests: &[PracticeResourceRequest],
) -> Result<(), PracticeResourceError> {
    for (key, indices) in groups.iter().take(MAX_PRACTICE_RESOURCE_CAPACITIES + 1) {
        let capacity = capacities[key];
        if capacity.mode != PracticeResourceAllocationMode::ExclusiveAllOrNone {
            continue;
        }
        let mut authority_claims: BTreeMap<_, (u128, BTreeSet<PracticeProposalKey>)> =
            BTreeMap::new();
        for index in indices.iter().take(MAX_PRACTICE_RESOURCE_REQUESTS + 1) {
            let request = &requests[*index];
            let proposal = request.proposal_key;
            let group_key = (
                proposal.resolve_tick,
                proposal.input_authority_id,
                proposal.actor_org_id,
            );
            let claim = authority_claims.entry(group_key).or_default();
            claim.0 = claim
                .0
                .checked_add(u128::from(request.requested))
                .ok_or(PracticeResourceError::ResourceArithmetic)?;
            claim.1.insert(proposal);
        }
        for (quantity, proposals) in authority_claims
            .values()
            .take(MAX_PRACTICE_RESOURCE_REQUESTS + 1)
        {
            if proposals.len() > 1 && *quantity > u128::from(capacity.available) {
                return Err(PracticeResourceError::ResourceAuthorityConflict);
            }
        }
    }
    Ok(())
}

fn allocate_group(
    capacity: &PracticeResourceCapacity,
    indices: &[usize],
    requests: &[PracticeResourceRequest],
    allocations: &mut [PracticeResourceAllocation],
) -> Result<u64, PracticeResourceError> {
    let mut total_requested = 0_u128;
    for index in indices.iter().take(MAX_PRACTICE_RESOURCE_REQUESTS + 1) {
        total_requested = total_requested
            .checked_add(u128::from(requests[*index].requested))
            .ok_or(PracticeResourceError::ResourceArithmetic)?;
    }
    let available = u128::from(capacity.available);
    let mut total_allocated = 0_u64;
    for index in indices.iter().take(MAX_PRACTICE_RESOURCE_REQUESTS + 1) {
        let requested = requests[*index].requested;
        let allocated = if available >= total_requested {
            requested
        } else {
            match capacity.mode {
                PracticeResourceAllocationMode::DivisibleProRata => {
                    let product = available * u128::from(requested);
                    u64::try_from(product / total_requested)
                        .map_err(|_| PracticeResourceError::ResourceArithmetic)?
                }
                PracticeResourceAllocationMode::ExclusiveAllOrNone => 0,
            }
        };
        allocations[*index].allocated = allocated;
        total_allocated = total_allocated
            .checked_add(allocated)
            .ok_or(PracticeResourceError::ResourceArithmetic)?;
    }
    Ok(total_allocated)
}

/// Allocate exact sealed requests without using canonical order as priority.
///
/// # Errors
/// Returns the first exact schema, bound, capacity, or arithmetic failure.
pub fn allocate_practice_resources(
    contract: &PracticeResourceAllocationContract,
    requests: &[PracticeResourceRequest],
    capacities: &[PracticeResourceCapacity],
) -> Result<PracticeResourceAllocationOutcome, PracticeResourceError> {
    validate_contract(contract)?;
    if requests.len() > MAX_PRACTICE_RESOURCE_REQUESTS {
        return Err(PracticeResourceError::ResourceRequestLimit);
    }
    if capacities.len() > MAX_PRACTICE_RESOURCE_CAPACITIES {
        return Err(PracticeResourceError::ResourceCapacityLimit);
    }
    let canonical = canonical_requests(requests)?;
    validate_requests_per_intent(&canonical)?;
    let capacity_by_key = capacity_index(capacities)?;
    let groups = request_groups(&canonical, &capacity_by_key)?;
    validate_authority_conflicts(&groups, &capacity_by_key, &canonical)?;
    let mut allocations: Vec<PracticeResourceAllocation> = canonical
        .iter()
        .take(MAX_PRACTICE_RESOURCE_REQUESTS + 1)
        .cloned()
        .map(|request| PracticeResourceAllocation {
            request,
            allocated: 0,
        })
        .collect();
    let mut balances = Vec::with_capacity(capacity_by_key.len());
    for (key, capacity) in capacity_by_key
        .iter()
        .take(MAX_PRACTICE_RESOURCE_CAPACITIES + 1)
    {
        let indices = groups.get(key).map_or(&[][..], Vec::as_slice);
        let allocated = allocate_group(capacity, indices, &canonical, &mut allocations)?;
        let unallocated = capacity
            .available
            .checked_sub(allocated)
            .ok_or(PracticeResourceError::ResourceArithmetic)?;
        balances.push(PracticeResourceBalance {
            capacity: (*capacity).clone(),
            allocated,
            unallocated,
        });
    }
    Ok(PracticeResourceAllocationOutcome {
        allocations,
        balances,
    })
}
