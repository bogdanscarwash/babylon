//! Pure V2 practice-resource derivation and conservation-first allocation.

use std::collections::{BTreeMap, BTreeSet};

use babylon_kernel::sha256_of;

use crate::actor_v2::ActorOrganizationIdV2;
use crate::intent_v2::target_is_valid;
use crate::{
    practice_proposal_key_v2, InputAuthorityIdV2, PracticeIdV2, PracticeIntentV2,
    PracticeProposalKeyV2, PracticeTargetIdentityV2, PracticeTargetTagV2, ProposalNonceV2,
    TaggedPracticeTargetV2,
};

const SCHEMA_VERSION: u16 = 2;

/// Canonical domain for the frozen V2 allocation law.
pub const PRACTICE_RESOURCE_ALLOCATION_CONTRACT_V2_DOMAIN_BYTES: &[u8] =
    b"babylon.practice-resource-allocation-contract.v2";
/// Canonical domain for one engine-derived V2 request.
pub const PRACTICE_RESOURCE_REQUEST_V2_DOMAIN_BYTES: &[u8] =
    b"babylon.practice-resource-request.v2";
/// Exact byte length of one engine-derived V2 request.
pub const PRACTICE_RESOURCE_REQUEST_V2_CANONICAL_BYTES: usize =
    PRACTICE_RESOURCE_REQUEST_V2_DOMAIN_BYTES.len() + 1 + 2 + 82 + 9 + 32 + 32 + 8;
/// Canonical domain for one sealed V2 capacity row.
pub const PRACTICE_RESOURCE_CAPACITY_V2_DOMAIN_BYTES: &[u8] =
    b"babylon.practice-resource-capacity.v2";
/// Exact byte length of one sealed V2 capacity row.
pub const PRACTICE_RESOURCE_CAPACITY_V2_CANONICAL_BYTES: usize =
    PRACTICE_RESOURCE_CAPACITY_V2_DOMAIN_BYTES.len() + 1 + 2 + 9 + 32 + 32 + 1 + 8;
/// Canonical domain for one V2 allocation outcome.
pub const PRACTICE_RESOURCE_ALLOCATION_OUTCOME_V2_DOMAIN_BYTES: &[u8] =
    b"babylon.practice-resource-allocation-outcome.v2";
/// SHA-256 of the exact language-neutral V2 resource-allocation schema bytes.
pub const PRACTICE_RESOURCE_ALLOCATION_V2_SOURCE_SHA256: [u8; 32] = [
    0x35, 0xfd, 0x1c, 0x26, 0xca, 0x31, 0x3b, 0x4c, 0xbc, 0xfd, 0xf6, 0xda, 0x6e, 0xcb, 0x53, 0xd1,
    0x9c, 0xdb, 0x0b, 0xc9, 0x20, 0xdc, 0x67, 0xb5, 0xe8, 0xc2, 0x02, 0xd9, 0xcf, 0x75, 0x96, 0x8a,
];

/// Designed validation and fuel ceiling, not an actor capacity or political quota.
pub const MAX_PRACTICE_RESOURCE_REQUESTS_V2: usize = 65_536;
/// Designed per-intent serialization ceiling, not a material or political quota.
pub const MAX_PRACTICE_RESOURCE_REQUESTS_PER_INTENT_V2: usize = 16;
/// Designed validation and fuel ceiling on distinct capacity rows.
pub const MAX_PRACTICE_RESOURCE_CAPACITIES_V2: usize = 65_536;

/// Exact V2 resource-contract failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PracticeResourceV2Error {
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
pub struct UnknownPracticeResourceV2ErrorCode(pub u16);

impl TryFrom<u16> for PracticeResourceV2Error {
    type Error = UnknownPracticeResourceV2ErrorCode;

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
            _ => Err(UnknownPracticeResourceV2ErrorCode(value)),
        }
    }
}

impl From<PracticeResourceV2Error> for u16 {
    fn from(value: PracticeResourceV2Error) -> Self {
        value as Self
    }
}

/// Stable resource-class or resource-instance identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PracticeResourceIdV2([u8; 32]);

impl PracticeResourceIdV2 {
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
pub struct PracticeUnitIdV2([u8; 32]);

impl PracticeUnitIdV2 {
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
pub enum PracticeResourceOwnerV2 {
    Shared,
    ActorOrganization(ActorOrganizationIdV2),
}

/// Content-owned locator law used to derive a request owner from an intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PracticeResourceLocatorV2 {
    Shared,
    ActorOrganization,
}

/// Scarcity law governed by the capacity row, never selected by a proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PracticeResourceAllocationModeV2 {
    DivisibleProRata = 1,
    ExclusiveAllOrNone = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PracticeResourceRequestDerivationLawV2 {
    SealedContent = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PracticeResourceDivisibleLawV2 {
    ProportionalFloor = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PracticeResourceExclusiveTieLawV2 {
    ContestedUnallocated = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PracticeResourceResidualLawV2 {
    RetainedAvailable = 1,
}

/// One sealed-content material requirement for a practice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PracticeResourceRequirementV2 {
    pub practice_id: PracticeIdV2,
    pub locator: PracticeResourceLocatorV2,
    pub resource_id: PracticeResourceIdV2,
    pub unit_id: PracticeUnitIdV2,
    pub quantity: u64,
}

/// One request derived from an accepted intent and sealed content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeResourceRequestV2 {
    proposal_key: PracticeProposalKeyV2,
    owner: PracticeResourceOwnerV2,
    resource_id: PracticeResourceIdV2,
    unit_id: PracticeUnitIdV2,
    requested: u64,
}

impl PracticeResourceRequestV2 {
    #[must_use]
    pub const fn proposal_key(&self) -> PracticeProposalKeyV2 {
        self.proposal_key
    }

    #[must_use]
    pub const fn owner(&self) -> PracticeResourceOwnerV2 {
        self.owner
    }

    #[must_use]
    pub const fn resource_id(&self) -> PracticeResourceIdV2 {
        self.resource_id
    }

    #[must_use]
    pub const fn unit_id(&self) -> PracticeUnitIdV2 {
        self.unit_id
    }

    #[must_use]
    pub const fn requested(&self) -> u64 {
        self.requested
    }
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

fn append_owner(output: &mut Vec<u8>, value: PracticeResourceOwnerV2) {
    match value {
        PracticeResourceOwnerV2::Shared => {
            output.push(1);
            output.extend_from_slice(&[0_u8; 8]);
        }
        PracticeResourceOwnerV2::ActorOrganization(actor_org_id) => {
            output.push(2);
            output.extend_from_slice(&actor_org_id.to_bytes());
        }
    }
}

fn validate_request(value: &PracticeResourceRequestV2) -> Result<(), PracticeResourceV2Error> {
    if value.requested == 0 {
        return Err(PracticeResourceV2Error::ResourceRequestZero);
    }
    if let PracticeResourceOwnerV2::ActorOrganization(owner) = value.owner {
        if owner != value.proposal_key.actor_org_id {
            return Err(PracticeResourceV2Error::ResourceOwnerMismatch);
        }
    }
    Ok(())
}

/// Encode one engine-derived V2 request in fixed big-endian order.
///
/// # Errors
/// Returns an exact quantity or owner refusal.
pub fn encode_practice_resource_request_v2(
    value: &PracticeResourceRequestV2,
) -> Result<Vec<u8>, PracticeResourceV2Error> {
    validate_request(value)?;
    let mut output = Vec::with_capacity(PRACTICE_RESOURCE_REQUEST_V2_CANONICAL_BYTES);
    output.extend_from_slice(PRACTICE_RESOURCE_REQUEST_V2_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&SCHEMA_VERSION.to_be_bytes());
    append_proposal_key(&mut output, value.proposal_key);
    append_owner(&mut output, value.owner);
    output.extend_from_slice(&value.resource_id.as_bytes());
    output.extend_from_slice(&value.unit_id.as_bytes());
    output.extend_from_slice(&value.requested.to_be_bytes());
    Ok(output)
}

/// Hash one validated engine-derived V2 request.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn practice_resource_request_v2_digest(
    value: &PracticeResourceRequestV2,
) -> Result<[u8; 32], PracticeResourceV2Error> {
    Ok(sha256_of(&encode_practice_resource_request_v2(value)?))
}

/// One true available-capacity row from the sealed material snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeResourceCapacityV2 {
    pub owner: PracticeResourceOwnerV2,
    pub resource_id: PracticeResourceIdV2,
    pub unit_id: PracticeUnitIdV2,
    pub mode: PracticeResourceAllocationModeV2,
    pub available: u64,
}

/// Encode one sealed V2 capacity row in fixed big-endian order.
///
/// # Errors
/// This closed typed row has no fallible field after construction.
pub fn encode_practice_resource_capacity_v2(
    value: &PracticeResourceCapacityV2,
) -> Result<Vec<u8>, PracticeResourceV2Error> {
    let mut output = Vec::with_capacity(PRACTICE_RESOURCE_CAPACITY_V2_CANONICAL_BYTES);
    output.extend_from_slice(PRACTICE_RESOURCE_CAPACITY_V2_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&SCHEMA_VERSION.to_be_bytes());
    append_owner(&mut output, value.owner);
    output.extend_from_slice(&value.resource_id.as_bytes());
    output.extend_from_slice(&value.unit_id.as_bytes());
    output.push(value.mode as u8);
    output.extend_from_slice(&value.available.to_be_bytes());
    Ok(output)
}

/// Decode one complete sealed V2 capacity row.
///
/// # Errors
/// Returns the first exact domain, schema, enum, wire, or owner refusal.
pub fn decode_practice_resource_capacity_v2(
    payload: &[u8],
) -> Result<PracticeResourceCapacityV2, PracticeResourceV2Error> {
    let mut cursor = ContractCursor::new(payload);
    if cursor.take(PRACTICE_RESOURCE_CAPACITY_V2_DOMAIN_BYTES.len())?
        != PRACTICE_RESOURCE_CAPACITY_V2_DOMAIN_BYTES
        || cursor.take(1)? != [0]
    {
        return Err(PracticeResourceV2Error::ResourceDomain);
    }
    if cursor.u16()? != SCHEMA_VERSION {
        return Err(PracticeResourceV2Error::ResourceSchemaVersion);
    }
    let owner = decode_owner(&mut cursor)?;
    let resource_id = PracticeResourceIdV2::from_bytes(cursor.array()?);
    let unit_id = PracticeUnitIdV2::from_bytes(cursor.array()?);
    let mode = match cursor.u8()? {
        1 => PracticeResourceAllocationModeV2::DivisibleProRata,
        2 => PracticeResourceAllocationModeV2::ExclusiveAllOrNone,
        _ => return Err(PracticeResourceV2Error::ResourceEnumCode),
    };
    let available = cursor.u64()?;
    cursor.finish()?;
    Ok(PracticeResourceCapacityV2 {
        owner,
        resource_id,
        unit_id,
        mode,
        available,
    })
}

/// Hash one sealed V2 capacity row.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn practice_resource_capacity_v2_digest(
    value: &PracticeResourceCapacityV2,
) -> Result<[u8; 32], PracticeResourceV2Error> {
    Ok(sha256_of(&encode_practice_resource_capacity_v2(value)?))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeResourceAllocationV2 {
    request: PracticeResourceRequestV2,
    allocated: u64,
}

impl PracticeResourceAllocationV2 {
    #[must_use]
    pub const fn request(&self) -> &PracticeResourceRequestV2 {
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
pub struct PracticeResourceBalanceV2 {
    capacity: PracticeResourceCapacityV2,
    allocated: u64,
    unallocated: u64,
}

impl PracticeResourceBalanceV2 {
    #[must_use]
    pub const fn capacity(&self) -> &PracticeResourceCapacityV2 {
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
pub struct PracticeResourceAllocationOutcomeV2 {
    allocations: Vec<PracticeResourceAllocationV2>,
    balances: Vec<PracticeResourceBalanceV2>,
}

impl PracticeResourceAllocationOutcomeV2 {
    #[must_use]
    pub fn allocations(&self) -> &[PracticeResourceAllocationV2] {
        &self.allocations
    }

    #[must_use]
    pub fn balances(&self) -> &[PracticeResourceBalanceV2] {
        &self.balances
    }
}

fn validate_outcome(
    value: &PracticeResourceAllocationOutcomeV2,
) -> Result<(), PracticeResourceV2Error> {
    if value.allocations.len() > MAX_PRACTICE_RESOURCE_REQUESTS_V2
        || value.balances.len() > MAX_PRACTICE_RESOURCE_CAPACITIES_V2
    {
        return Err(PracticeResourceV2Error::ResourceOutcomeLimit);
    }
    for allocation in value
        .allocations
        .iter()
        .take(MAX_PRACTICE_RESOURCE_REQUESTS_V2 + 1)
    {
        if allocation.allocated > allocation.request.requested {
            return Err(PracticeResourceV2Error::ResourceOutcomeConservation);
        }
    }
    for balance in value
        .balances
        .iter()
        .take(MAX_PRACTICE_RESOURCE_CAPACITIES_V2 + 1)
    {
        let total = balance
            .allocated
            .checked_add(balance.unallocated)
            .ok_or(PracticeResourceV2Error::ResourceArithmetic)?;
        if total != balance.capacity.available {
            return Err(PracticeResourceV2Error::ResourceOutcomeConservation);
        }
    }
    Ok(())
}

/// Encode one allocator-produced V2 outcome with request and capacity identities.
///
/// # Errors
/// Returns the first exact contract, bound, identity, or conservation refusal.
pub fn encode_practice_resource_allocation_outcome_v2(
    contract: &PracticeResourceAllocationContractV2,
    value: &PracticeResourceAllocationOutcomeV2,
) -> Result<Vec<u8>, PracticeResourceV2Error> {
    validate_contract(contract)?;
    validate_outcome(value)?;
    let allocation_bytes = value
        .allocations
        .len()
        .checked_mul(40)
        .ok_or(PracticeResourceV2Error::ResourceArithmetic)?;
    let balance_bytes = value
        .balances
        .len()
        .checked_mul(48)
        .ok_or(PracticeResourceV2Error::ResourceArithmetic)?;
    let mut output = Vec::with_capacity(
        PRACTICE_RESOURCE_ALLOCATION_OUTCOME_V2_DOMAIN_BYTES.len()
            + 1
            + 2
            + 32
            + 4
            + allocation_bytes
            + 4
            + balance_bytes,
    );
    output.extend_from_slice(PRACTICE_RESOURCE_ALLOCATION_OUTCOME_V2_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&SCHEMA_VERSION.to_be_bytes());
    output.extend_from_slice(&practice_resource_allocation_contract_v2_digest(contract)?);
    let allocation_count = u32::try_from(value.allocations.len())
        .map_err(|_| PracticeResourceV2Error::ResourceOutcomeLimit)?;
    output.extend_from_slice(&allocation_count.to_be_bytes());
    for allocation in value
        .allocations
        .iter()
        .take(MAX_PRACTICE_RESOURCE_REQUESTS_V2 + 1)
    {
        output.extend_from_slice(&practice_resource_request_v2_digest(&allocation.request)?);
        output.extend_from_slice(&allocation.allocated.to_be_bytes());
    }
    let balance_count = u32::try_from(value.balances.len())
        .map_err(|_| PracticeResourceV2Error::ResourceOutcomeLimit)?;
    output.extend_from_slice(&balance_count.to_be_bytes());
    for balance in value
        .balances
        .iter()
        .take(MAX_PRACTICE_RESOURCE_CAPACITIES_V2 + 1)
    {
        output.extend_from_slice(&practice_resource_capacity_v2_digest(&balance.capacity)?);
        output.extend_from_slice(&balance.allocated.to_be_bytes());
        output.extend_from_slice(&balance.unallocated.to_be_bytes());
    }
    Ok(output)
}

#[derive(Debug, PartialEq, Eq)]
struct PracticeResourceOutcomeIdentityV2 {
    allocations: Vec<([u8; 32], u64)>,
    balances: Vec<([u8; 32], u64, u64)>,
}

fn outcome_identity(
    value: &PracticeResourceAllocationOutcomeV2,
) -> Result<PracticeResourceOutcomeIdentityV2, PracticeResourceV2Error> {
    let mut allocations = Vec::with_capacity(value.allocations.len());
    for allocation in value
        .allocations
        .iter()
        .take(MAX_PRACTICE_RESOURCE_REQUESTS_V2 + 1)
    {
        allocations.push((
            practice_resource_request_v2_digest(&allocation.request)?,
            allocation.allocated,
        ));
    }
    let mut balances = Vec::with_capacity(value.balances.len());
    for balance in value
        .balances
        .iter()
        .take(MAX_PRACTICE_RESOURCE_CAPACITIES_V2 + 1)
    {
        balances.push((
            practice_resource_capacity_v2_digest(&balance.capacity)?,
            balance.allocated,
            balance.unallocated,
        ));
    }
    Ok(PracticeResourceOutcomeIdentityV2 {
        allocations,
        balances,
    })
}

fn decode_outcome_identity(
    cursor: &mut ContractCursor<'_>,
) -> Result<PracticeResourceOutcomeIdentityV2, PracticeResourceV2Error> {
    let allocation_count = usize::try_from(cursor.u32()?)
        .map_err(|_| PracticeResourceV2Error::ResourceOutcomeLimit)?;
    if allocation_count > MAX_PRACTICE_RESOURCE_REQUESTS_V2 {
        return Err(PracticeResourceV2Error::ResourceOutcomeLimit);
    }
    let mut allocations = Vec::with_capacity(allocation_count);
    for index in 0..=MAX_PRACTICE_RESOURCE_REQUESTS_V2 {
        if index == allocation_count {
            break;
        }
        allocations.push((cursor.array()?, cursor.u64()?));
    }
    let balance_count = usize::try_from(cursor.u32()?)
        .map_err(|_| PracticeResourceV2Error::ResourceOutcomeLimit)?;
    if balance_count > MAX_PRACTICE_RESOURCE_CAPACITIES_V2 {
        return Err(PracticeResourceV2Error::ResourceOutcomeLimit);
    }
    let mut balances = Vec::with_capacity(balance_count);
    for index in 0..=MAX_PRACTICE_RESOURCE_CAPACITIES_V2 {
        if index == balance_count {
            break;
        }
        balances.push((cursor.array()?, cursor.u64()?, cursor.u64()?));
    }
    Ok(PracticeResourceOutcomeIdentityV2 {
        allocations,
        balances,
    })
}

/// Decode one outcome and replay its governed allocator inputs.
///
/// # Errors
/// Returns the first exact wire, contract, input, allocation, or identity refusal.
pub fn decode_practice_resource_allocation_outcome_v2(
    payload: &[u8],
    contract: &PracticeResourceAllocationContractV2,
    requests: &[PracticeResourceRequestV2],
    capacities: &[PracticeResourceCapacityV2],
) -> Result<PracticeResourceAllocationOutcomeV2, PracticeResourceV2Error> {
    validate_contract(contract)?;
    let mut cursor = ContractCursor::new(payload);
    if cursor.take(PRACTICE_RESOURCE_ALLOCATION_OUTCOME_V2_DOMAIN_BYTES.len())?
        != PRACTICE_RESOURCE_ALLOCATION_OUTCOME_V2_DOMAIN_BYTES
        || cursor.take(1)? != [0]
    {
        return Err(PracticeResourceV2Error::ResourceDomain);
    }
    if cursor.u16()? != SCHEMA_VERSION {
        return Err(PracticeResourceV2Error::ResourceSchemaVersion);
    }
    if cursor.array::<32>()? != practice_resource_allocation_contract_v2_digest(contract)? {
        return Err(PracticeResourceV2Error::ResourceOutcomeContractDigest);
    }
    let actual = decode_outcome_identity(&mut cursor)?;
    cursor.finish()?;
    let expected = allocate_practice_resources_v2(contract, requests, capacities)?;
    validate_outcome(&expected)?;
    if actual != outcome_identity(&expected)? {
        return Err(PracticeResourceV2Error::ResourceOutcomeMismatch);
    }
    Ok(expected)
}

/// Hash one validated V2 allocation outcome.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn practice_resource_allocation_outcome_v2_digest(
    contract: &PracticeResourceAllocationContractV2,
    value: &PracticeResourceAllocationOutcomeV2,
) -> Result<[u8; 32], PracticeResourceV2Error> {
    Ok(sha256_of(&encode_practice_resource_allocation_outcome_v2(
        contract, value,
    )?))
}

/// Frozen law identity quoted by every `PracticeIntentV2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PracticeResourceAllocationContractV2 {
    pub schema_version: u16,
    pub quantity_width_bits: u8,
    pub request_derivation_law: PracticeResourceRequestDerivationLawV2,
    pub divisible_law: PracticeResourceDivisibleLawV2,
    pub exclusive_tie_law: PracticeResourceExclusiveTieLawV2,
    pub residual_law: PracticeResourceResidualLawV2,
    pub max_requests_per_intent: u16,
    pub max_requests_total: u32,
    pub max_capacities_total: u32,
}

impl PracticeResourceAllocationContractV2 {
    #[must_use]
    pub const fn conservation_first() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            quantity_width_bits: 64,
            request_derivation_law: PracticeResourceRequestDerivationLawV2::SealedContent,
            divisible_law: PracticeResourceDivisibleLawV2::ProportionalFloor,
            exclusive_tie_law: PracticeResourceExclusiveTieLawV2::ContestedUnallocated,
            residual_law: PracticeResourceResidualLawV2::RetainedAvailable,
            max_requests_per_intent: 16,
            max_requests_total: 65_536,
            max_capacities_total: 65_536,
        }
    }
}

fn validate_contract(
    value: &PracticeResourceAllocationContractV2,
) -> Result<(), PracticeResourceV2Error> {
    if value.schema_version != SCHEMA_VERSION {
        return Err(PracticeResourceV2Error::ResourceSchemaVersion);
    }
    if value != &PracticeResourceAllocationContractV2::conservation_first() {
        return Err(PracticeResourceV2Error::ResourceContractValue);
    }
    Ok(())
}

/// Encode the complete frozen V2 allocation law.
///
/// # Errors
/// Returns an exact schema or governed-value refusal.
pub fn encode_practice_resource_allocation_contract_v2(
    value: &PracticeResourceAllocationContractV2,
) -> Result<Vec<u8>, PracticeResourceV2Error> {
    validate_contract(value)?;
    let mut output =
        Vec::with_capacity(PRACTICE_RESOURCE_ALLOCATION_CONTRACT_V2_DOMAIN_BYTES.len() + 18);
    output.extend_from_slice(PRACTICE_RESOURCE_ALLOCATION_CONTRACT_V2_DOMAIN_BYTES);
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

    fn take(&mut self, count: usize) -> Result<&'a [u8], PracticeResourceV2Error> {
        let end = self
            .index
            .checked_add(count)
            .ok_or(PracticeResourceV2Error::ResourceTruncated)?;
        let value = self
            .payload
            .get(self.index..end)
            .ok_or(PracticeResourceV2Error::ResourceTruncated)?;
        self.index = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, PracticeResourceV2Error> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, PracticeResourceV2Error> {
        Ok(u16::from_be_bytes(
            self.take(2)?
                .try_into()
                .map_err(|_| PracticeResourceV2Error::ResourceTruncated)?,
        ))
    }

    fn u32(&mut self) -> Result<u32, PracticeResourceV2Error> {
        Ok(u32::from_be_bytes(
            self.take(4)?
                .try_into()
                .map_err(|_| PracticeResourceV2Error::ResourceTruncated)?,
        ))
    }

    fn u64(&mut self) -> Result<u64, PracticeResourceV2Error> {
        Ok(u64::from_be_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| PracticeResourceV2Error::ResourceTruncated)?,
        ))
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], PracticeResourceV2Error> {
        self.take(N)?
            .try_into()
            .map_err(|_| PracticeResourceV2Error::ResourceTruncated)
    }

    fn finish(&self) -> Result<(), PracticeResourceV2Error> {
        if self.index == self.payload.len() {
            Ok(())
        } else {
            Err(PracticeResourceV2Error::ResourceTrailingBytes)
        }
    }
}

fn decode_owner(
    cursor: &mut ContractCursor<'_>,
) -> Result<PracticeResourceOwnerV2, PracticeResourceV2Error> {
    let tag = cursor.u8()?;
    let actor_org_id: [u8; 8] = cursor.array()?;
    match tag {
        1 if actor_org_id == [0_u8; 8] => Ok(PracticeResourceOwnerV2::Shared),
        1 => Err(PracticeResourceV2Error::ResourceOwnerMismatch),
        2 => Ok(PracticeResourceOwnerV2::ActorOrganization(
            ActorOrganizationIdV2::from_bytes(actor_org_id),
        )),
        _ => Err(PracticeResourceV2Error::ResourceEnumCode),
    }
}

fn decode_proposal_key(
    cursor: &mut ContractCursor<'_>,
) -> Result<PracticeProposalKeyV2, PracticeResourceV2Error> {
    let resolve_tick = cursor.u64()?;
    let input_authority_id = InputAuthorityIdV2::from_bytes(cursor.array()?);
    let actor_org_id = ActorOrganizationIdV2::from_bytes(cursor.array()?);
    let practice_id = PracticeIdV2::try_from(cursor.u8()?)
        .map_err(|_| PracticeResourceV2Error::ResourceEnumCode)?;
    let target = TaggedPracticeTargetV2 {
        tag: PracticeTargetTagV2::try_from(cursor.u8()?)
            .map_err(|_| PracticeResourceV2Error::ResourceEnumCode)?,
        identity: PracticeTargetIdentityV2::from_bytes(cursor.array()?),
    };
    if !target_is_valid(practice_id, target.tag) {
        return Err(PracticeResourceV2Error::ResourceEnumCode);
    }
    Ok(PracticeProposalKeyV2 {
        resolve_tick,
        input_authority_id,
        actor_org_id,
        practice_id,
        target,
        proposal_nonce: ProposalNonceV2::from_bytes(cursor.array()?),
    })
}

/// Decode one complete engine-derived V2 request.
///
/// # Errors
/// Returns the first exact domain, schema, enum, wire, quantity, or owner refusal.
pub fn decode_practice_resource_request_v2(
    payload: &[u8],
) -> Result<PracticeResourceRequestV2, PracticeResourceV2Error> {
    let mut cursor = ContractCursor::new(payload);
    if cursor.take(PRACTICE_RESOURCE_REQUEST_V2_DOMAIN_BYTES.len())?
        != PRACTICE_RESOURCE_REQUEST_V2_DOMAIN_BYTES
        || cursor.take(1)? != [0]
    {
        return Err(PracticeResourceV2Error::ResourceDomain);
    }
    if cursor.u16()? != SCHEMA_VERSION {
        return Err(PracticeResourceV2Error::ResourceSchemaVersion);
    }
    let value = PracticeResourceRequestV2 {
        proposal_key: decode_proposal_key(&mut cursor)?,
        owner: decode_owner(&mut cursor)?,
        resource_id: PracticeResourceIdV2::from_bytes(cursor.array()?),
        unit_id: PracticeUnitIdV2::from_bytes(cursor.array()?),
        requested: cursor.u64()?,
    };
    cursor.finish()?;
    validate_request(&value)?;
    Ok(value)
}

fn law(value: u8) -> Result<(), PracticeResourceV2Error> {
    if value == 1 {
        Ok(())
    } else {
        Err(PracticeResourceV2Error::ResourceEnumCode)
    }
}

/// Decode the complete frozen V2 allocation law.
///
/// # Errors
/// Returns the first exact domain, wire, enum, schema, or governed-value refusal.
pub fn decode_practice_resource_allocation_contract_v2(
    payload: &[u8],
) -> Result<PracticeResourceAllocationContractV2, PracticeResourceV2Error> {
    let mut cursor = ContractCursor::new(payload);
    if cursor.take(PRACTICE_RESOURCE_ALLOCATION_CONTRACT_V2_DOMAIN_BYTES.len())?
        != PRACTICE_RESOURCE_ALLOCATION_CONTRACT_V2_DOMAIN_BYTES
        || cursor.take(1)? != [0]
    {
        return Err(PracticeResourceV2Error::ResourceDomain);
    }
    let value = PracticeResourceAllocationContractV2 {
        schema_version: cursor.u16()?,
        quantity_width_bits: cursor.u8()?,
        request_derivation_law: {
            law(cursor.u8()?)?;
            PracticeResourceRequestDerivationLawV2::SealedContent
        },
        divisible_law: {
            law(cursor.u8()?)?;
            PracticeResourceDivisibleLawV2::ProportionalFloor
        },
        exclusive_tie_law: {
            law(cursor.u8()?)?;
            PracticeResourceExclusiveTieLawV2::ContestedUnallocated
        },
        residual_law: {
            law(cursor.u8()?)?;
            PracticeResourceResidualLawV2::RetainedAvailable
        },
        max_requests_per_intent: cursor.u16()?,
        max_requests_total: cursor.u32()?,
        max_capacities_total: cursor.u32()?,
    };
    cursor.finish()?;
    validate_contract(&value)?;
    Ok(value)
}

/// Hash the complete validated V2 allocation law.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn practice_resource_allocation_contract_v2_digest(
    value: &PracticeResourceAllocationContractV2,
) -> Result<[u8; 32], PracticeResourceV2Error> {
    Ok(sha256_of(&encode_practice_resource_allocation_contract_v2(
        value,
    )?))
}

type CapacityKey = (
    PracticeResourceOwnerV2,
    PracticeResourceIdV2,
    PracticeUnitIdV2,
);

fn capacity_key(value: &PracticeResourceCapacityV2) -> CapacityKey {
    (value.owner, value.resource_id, value.unit_id)
}

fn request_key(value: &PracticeResourceRequestV2) -> CapacityKey {
    (value.owner, value.resource_id, value.unit_id)
}

fn canonical_request_key(
    value: &PracticeResourceRequestV2,
) -> (CapacityKey, PracticeProposalKeyV2) {
    (request_key(value), value.proposal_key)
}

fn canonical_requests(
    requests: &[PracticeResourceRequestV2],
) -> Result<Vec<PracticeResourceRequestV2>, PracticeResourceV2Error> {
    let mut output = requests.to_vec();
    output.sort_unstable_by_key(canonical_request_key);
    for pair in output.windows(2).take(MAX_PRACTICE_RESOURCE_REQUESTS_V2) {
        if canonical_request_key(&pair[0]) == canonical_request_key(&pair[1]) {
            return Err(PracticeResourceV2Error::ResourceRequestDuplicate);
        }
    }
    Ok(output)
}

fn validate_requests_per_intent(
    requests: &[PracticeResourceRequestV2],
) -> Result<(), PracticeResourceV2Error> {
    let mut counts: BTreeMap<PracticeProposalKeyV2, usize> = BTreeMap::new();
    for request in requests.iter().take(MAX_PRACTICE_RESOURCE_REQUESTS_V2 + 1) {
        let count = counts.entry(request.proposal_key).or_insert(0);
        *count = count
            .checked_add(1)
            .ok_or(PracticeResourceV2Error::ResourceArithmetic)?;
        if *count > MAX_PRACTICE_RESOURCE_REQUESTS_PER_INTENT_V2 {
            return Err(PracticeResourceV2Error::ResourceRequestsPerIntentLimit);
        }
    }
    Ok(())
}

/// Derive one material request from an accepted intent and sealed content.
///
/// # Errors
/// Returns an exact practice mismatch or zero-quantity refusal.
pub fn derive_practice_resource_request_v2(
    contract: &PracticeResourceAllocationContractV2,
    intent: &PracticeIntentV2,
    requirement: &PracticeResourceRequirementV2,
) -> Result<PracticeResourceRequestV2, PracticeResourceV2Error> {
    validate_contract(contract)?;
    if intent.quoted_resource_contract_digest
        != practice_resource_allocation_contract_v2_digest(contract)?
    {
        return Err(PracticeResourceV2Error::ResourceContractDigestMismatch);
    }
    if intent.practice_id != requirement.practice_id {
        return Err(PracticeResourceV2Error::ResourceRequirementPracticeMismatch);
    }
    if requirement.quantity == 0 {
        return Err(PracticeResourceV2Error::ResourceRequestZero);
    }
    let owner = match requirement.locator {
        PracticeResourceLocatorV2::Shared => PracticeResourceOwnerV2::Shared,
        PracticeResourceLocatorV2::ActorOrganization => {
            PracticeResourceOwnerV2::ActorOrganization(intent.actor_org_id)
        }
    };
    Ok(PracticeResourceRequestV2 {
        proposal_key: practice_proposal_key_v2(intent),
        owner,
        resource_id: requirement.resource_id,
        unit_id: requirement.unit_id,
        requested: requirement.quantity,
    })
}

fn capacity_index(
    capacities: &[PracticeResourceCapacityV2],
) -> Result<BTreeMap<CapacityKey, &PracticeResourceCapacityV2>, PracticeResourceV2Error> {
    let mut output = BTreeMap::new();
    for capacity in capacities
        .iter()
        .take(MAX_PRACTICE_RESOURCE_CAPACITIES_V2 + 1)
    {
        if output.insert(capacity_key(capacity), capacity).is_some() {
            return Err(PracticeResourceV2Error::ResourceCapacityDuplicate);
        }
    }
    Ok(output)
}

fn request_groups(
    requests: &[PracticeResourceRequestV2],
    capacities: &BTreeMap<CapacityKey, &PracticeResourceCapacityV2>,
) -> Result<BTreeMap<CapacityKey, Vec<usize>>, PracticeResourceV2Error> {
    let mut output: BTreeMap<CapacityKey, Vec<usize>> = BTreeMap::new();
    for (index, request) in requests
        .iter()
        .take(MAX_PRACTICE_RESOURCE_REQUESTS_V2 + 1)
        .enumerate()
    {
        let key = request_key(request);
        if !capacities.contains_key(&key) {
            return Err(PracticeResourceV2Error::ResourceCapacityMissing);
        }
        output.entry(key).or_default().push(index);
    }
    Ok(output)
}

fn validate_authority_conflicts(
    groups: &BTreeMap<CapacityKey, Vec<usize>>,
    capacities: &BTreeMap<CapacityKey, &PracticeResourceCapacityV2>,
    requests: &[PracticeResourceRequestV2],
) -> Result<(), PracticeResourceV2Error> {
    for (key, indices) in groups.iter().take(MAX_PRACTICE_RESOURCE_CAPACITIES_V2 + 1) {
        let capacity = capacities[key];
        if capacity.mode != PracticeResourceAllocationModeV2::ExclusiveAllOrNone {
            continue;
        }
        let mut authority_claims: BTreeMap<_, (u128, BTreeSet<PracticeProposalKeyV2>)> =
            BTreeMap::new();
        for index in indices.iter().take(MAX_PRACTICE_RESOURCE_REQUESTS_V2 + 1) {
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
                .ok_or(PracticeResourceV2Error::ResourceArithmetic)?;
            claim.1.insert(proposal);
        }
        for (quantity, proposals) in authority_claims
            .values()
            .take(MAX_PRACTICE_RESOURCE_REQUESTS_V2 + 1)
        {
            if proposals.len() > 1 && *quantity > u128::from(capacity.available) {
                return Err(PracticeResourceV2Error::ResourceAuthorityConflict);
            }
        }
    }
    Ok(())
}

fn allocate_group(
    capacity: &PracticeResourceCapacityV2,
    indices: &[usize],
    requests: &[PracticeResourceRequestV2],
    allocations: &mut [PracticeResourceAllocationV2],
) -> Result<u64, PracticeResourceV2Error> {
    let mut total_requested = 0_u128;
    for index in indices.iter().take(MAX_PRACTICE_RESOURCE_REQUESTS_V2 + 1) {
        total_requested = total_requested
            .checked_add(u128::from(requests[*index].requested))
            .ok_or(PracticeResourceV2Error::ResourceArithmetic)?;
    }
    let available = u128::from(capacity.available);
    let mut total_allocated = 0_u64;
    for index in indices.iter().take(MAX_PRACTICE_RESOURCE_REQUESTS_V2 + 1) {
        let requested = requests[*index].requested;
        let allocated = if available >= total_requested {
            requested
        } else {
            match capacity.mode {
                PracticeResourceAllocationModeV2::DivisibleProRata => {
                    let product = available * u128::from(requested);
                    u64::try_from(product / total_requested)
                        .map_err(|_| PracticeResourceV2Error::ResourceArithmetic)?
                }
                PracticeResourceAllocationModeV2::ExclusiveAllOrNone => 0,
            }
        };
        allocations[*index].allocated = allocated;
        total_allocated = total_allocated
            .checked_add(allocated)
            .ok_or(PracticeResourceV2Error::ResourceArithmetic)?;
    }
    Ok(total_allocated)
}

/// Allocate exact sealed requests without using canonical order as priority.
///
/// # Errors
/// Returns the first exact schema, bound, capacity, or arithmetic failure.
pub fn allocate_practice_resources_v2(
    contract: &PracticeResourceAllocationContractV2,
    requests: &[PracticeResourceRequestV2],
    capacities: &[PracticeResourceCapacityV2],
) -> Result<PracticeResourceAllocationOutcomeV2, PracticeResourceV2Error> {
    validate_contract(contract)?;
    if requests.len() > MAX_PRACTICE_RESOURCE_REQUESTS_V2 {
        return Err(PracticeResourceV2Error::ResourceRequestLimit);
    }
    if capacities.len() > MAX_PRACTICE_RESOURCE_CAPACITIES_V2 {
        return Err(PracticeResourceV2Error::ResourceCapacityLimit);
    }
    let canonical = canonical_requests(requests)?;
    validate_requests_per_intent(&canonical)?;
    let capacity_by_key = capacity_index(capacities)?;
    let groups = request_groups(&canonical, &capacity_by_key)?;
    validate_authority_conflicts(&groups, &capacity_by_key, &canonical)?;
    let mut allocations: Vec<PracticeResourceAllocationV2> = canonical
        .iter()
        .take(MAX_PRACTICE_RESOURCE_REQUESTS_V2 + 1)
        .cloned()
        .map(|request| PracticeResourceAllocationV2 {
            request,
            allocated: 0,
        })
        .collect();
    let mut balances = Vec::with_capacity(capacity_by_key.len());
    for (key, capacity) in capacity_by_key
        .iter()
        .take(MAX_PRACTICE_RESOURCE_CAPACITIES_V2 + 1)
    {
        let indices = groups.get(key).map_or(&[][..], Vec::as_slice);
        let allocated = allocate_group(capacity, indices, &canonical, &mut allocations)?;
        let unallocated = capacity
            .available
            .checked_sub(allocated)
            .ok_or(PracticeResourceV2Error::ResourceArithmetic)?;
        balances.push(PracticeResourceBalanceV2 {
            capacity: (*capacity).clone(),
            allocated,
            unallocated,
        });
    }
    Ok(PracticeResourceAllocationOutcomeV2 {
        allocations,
        balances,
    })
}
