// Rust schema implementation paired with contracts/practice_contract_v1.yaml.

pub const PRACTICE_INPUT_AUTHORITY_V1_DOMAIN_BYTES: &[u8] = b"babylon.practice-input-authority.v1";
pub const PRACTICE_INTENT_V1_DOMAIN_BYTES: &[u8] = b"babylon.practice-intent.v1";
pub const ORGANIZATION_BUDGET_DELTA_V1_DOMAIN_BYTES: &[u8] =
    b"babylon.organization-budget-delta.v1";
pub const PRACTICE_WIRE_DOMAIN_TERMINATOR_BYTES: &[u8] = b"\x00";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum PracticeContractError {
    PracticeDomain = 1,
    PracticeSchemaVersion = 2,
    PracticeEnumCode = 3,
    PracticeLength = 5,
    PracticeTruncated = 6,
    PracticeTrailingBytes = 7,
    PracticeBoolean = 9,
    PracticeParameter = 10,
    PracticeParameterLimit = 11,
    PracticeParameterLength = 12,
    PracticeEvidenceLimit = 13,
    PracticeEvidenceOrder = 14,
    PracticeEvidenceDuplicate = 15,
    PracticeTickOverflow = 16,
    PracticeTickMismatch = 17,
    PracticeAuthorityRegistryLimit = 18,
    PracticeAuthorityRegistryOrder = 19,
    PracticeAuthorityRegistryDuplicate = 20,
    PracticeAuthorityUnregistered = 21,
    PracticeActorMismatch = 22,
    PracticeAuthorityContentMismatch = 23,
    PracticeQuoteContentMismatch = 24,
    PracticeQuoteCostMismatch = 25,
    PracticeBatchLimit = 26,
    PracticeDuplicateActor = 27,
    PracticeBudgetNonfinite = 28,
    PracticeBudgetNegative = 29,
    PracticeBudgetFractional = 30,
    PracticeBudgetRange = 31,
    PracticeBudgetRoundtrip = 32,
    PracticeBudgetInsufficient = 33,
    PracticeBudgetArithmetic = 34,
    PracticeFootprintLimit = 35,
    PracticeFootprintOrder = 36,
    PracticeFootprintDuplicate = 37,
    PracticeFootprintSource = 38,
    PracticeFootprintStrengthNonfinite = 39,
    PracticeFootprintStrengthNonpositive = 40,
    PracticeTopologyOrganizationLimit = 41,
    PracticeTopologyOrganizationOrder = 42,
    PracticeTopologyOrganizationDuplicate = 43,
    PracticeTopologyBudgetMissing = 44,
    PracticeTopologyEdgeOrder = 45,
    PracticeTopologyEdgeDuplicate = 46,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownPracticeContractErrorCode(pub u16);

impl TryFrom<u16> for PracticeContractError {
    type Error = UnknownPracticeContractErrorCode;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::PracticeDomain),
            2 => Ok(Self::PracticeSchemaVersion),
            3 => Ok(Self::PracticeEnumCode),
            5 => Ok(Self::PracticeLength),
            6 => Ok(Self::PracticeTruncated),
            7 => Ok(Self::PracticeTrailingBytes),
            9 => Ok(Self::PracticeBoolean),
            10 => Ok(Self::PracticeParameter),
            11 => Ok(Self::PracticeParameterLimit),
            12 => Ok(Self::PracticeParameterLength),
            13 => Ok(Self::PracticeEvidenceLimit),
            14 => Ok(Self::PracticeEvidenceOrder),
            15 => Ok(Self::PracticeEvidenceDuplicate),
            16 => Ok(Self::PracticeTickOverflow),
            17 => Ok(Self::PracticeTickMismatch),
            18 => Ok(Self::PracticeAuthorityRegistryLimit),
            19 => Ok(Self::PracticeAuthorityRegistryOrder),
            20 => Ok(Self::PracticeAuthorityRegistryDuplicate),
            21 => Ok(Self::PracticeAuthorityUnregistered),
            22 => Ok(Self::PracticeActorMismatch),
            23 => Ok(Self::PracticeAuthorityContentMismatch),
            24 => Ok(Self::PracticeQuoteContentMismatch),
            25 => Ok(Self::PracticeQuoteCostMismatch),
            26 => Ok(Self::PracticeBatchLimit),
            27 => Ok(Self::PracticeDuplicateActor),
            28 => Ok(Self::PracticeBudgetNonfinite),
            29 => Ok(Self::PracticeBudgetNegative),
            30 => Ok(Self::PracticeBudgetFractional),
            31 => Ok(Self::PracticeBudgetRange),
            32 => Ok(Self::PracticeBudgetRoundtrip),
            33 => Ok(Self::PracticeBudgetInsufficient),
            34 => Ok(Self::PracticeBudgetArithmetic),
            35 => Ok(Self::PracticeFootprintLimit),
            36 => Ok(Self::PracticeFootprintOrder),
            37 => Ok(Self::PracticeFootprintDuplicate),
            38 => Ok(Self::PracticeFootprintSource),
            39 => Ok(Self::PracticeFootprintStrengthNonfinite),
            40 => Ok(Self::PracticeFootprintStrengthNonpositive),
            41 => Ok(Self::PracticeTopologyOrganizationLimit),
            42 => Ok(Self::PracticeTopologyOrganizationOrder),
            43 => Ok(Self::PracticeTopologyOrganizationDuplicate),
            44 => Ok(Self::PracticeTopologyBudgetMissing),
            45 => Ok(Self::PracticeTopologyEdgeOrder),
            46 => Ok(Self::PracticeTopologyEdgeDuplicate),
            _ => Err(UnknownPracticeContractErrorCode(value)),
        }
    }
}

impl From<PracticeContractError> for u16 {
    fn from(value: PracticeContractError) -> Self {
        value as Self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PracticeIdV1 {
    Organize = 1,
    Agitate = 2,
    MutualAid = 3,
}

impl TryFrom<u8> for PracticeIdV1 {
    type Error = PracticeContractError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Organize),
            2 => Ok(Self::Agitate),
            3 => Ok(Self::MutualAid),
            _ => Err(PracticeContractError::PracticeEnumCode),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum VerbStemV1 {
    Mobilize = 1,
    Aid = 2,
}

impl TryFrom<u8> for VerbStemV1 {
    type Error = PracticeContractError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Mobilize),
            2 => Ok(Self::Aid),
            _ => Err(PracticeContractError::PracticeEnumCode),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum VerbModeV1 {
    Canvass = 1,
    Agitate = 2,
}

impl TryFrom<u8> for VerbModeV1 {
    type Error = PracticeContractError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Canvass),
            2 => Ok(Self::Agitate),
            _ => Err(PracticeContractError::PracticeEnumCode),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PracticeAuthorityKindV1 {
    PlayerSeat = 1,
    DeterministicPolicy = 2,
}

impl TryFrom<u8> for PracticeAuthorityKindV1 {
    type Error = PracticeContractError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::PlayerSeat),
            2 => Ok(Self::DeterministicPolicy),
            _ => Err(PracticeContractError::PracticeEnumCode),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PracticeTargetDomainV1 {
    SocialClass = 1,
}

impl TryFrom<u8> for PracticeTargetDomainV1 {
    type Error = PracticeContractError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::SocialClass),
            _ => Err(PracticeContractError::PracticeEnumCode),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum PracticeRejectionCodeV1 {
    PracticeUnwired = 1,
    PracticeStaleContent = 2,
    PracticeCostMismatch = 3,
    PracticeAuthorityUnregistered = 4,
    PracticeActorMismatch = 5,
    PracticeDuplicateActor = 6,
    PracticeBatchLimit = 7,
    PracticeTickMismatch = 8,
    PracticeBudgetInsufficient = 9,
    PracticeTargetIneligible = 10,
    PracticePendingDuplicate = 11,
}

impl TryFrom<u16> for PracticeRejectionCodeV1 {
    type Error = PracticeContractError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::PracticeUnwired),
            2 => Ok(Self::PracticeStaleContent),
            3 => Ok(Self::PracticeCostMismatch),
            4 => Ok(Self::PracticeAuthorityUnregistered),
            5 => Ok(Self::PracticeActorMismatch),
            6 => Ok(Self::PracticeDuplicateActor),
            7 => Ok(Self::PracticeBatchLimit),
            8 => Ok(Self::PracticeTickMismatch),
            9 => Ok(Self::PracticeBudgetInsufficient),
            10 => Ok(Self::PracticeTargetIneligible),
            11 => Ok(Self::PracticePendingDuplicate),
            _ => Err(PracticeContractError::PracticeEnumCode),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PracticeActivationBlockerV1 {
    Gate3CommittedEnvelope = 1,
    Gate5PendingInput = 2,
    Per30OrdersInventory = 3,
    Per31FreightRealization = 4,
}

impl TryFrom<u8> for PracticeActivationBlockerV1 {
    type Error = PracticeContractError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Gate3CommittedEnvelope),
            2 => Ok(Self::Gate5PendingInput),
            3 => Ok(Self::Per30OrdersInventory),
            4 => Ok(Self::Per31FreightRealization),
            _ => Err(PracticeContractError::PracticeEnumCode),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MachineVerbV1 {
    pub stem: VerbStemV1,
    pub mode: Option<VerbModeV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeInputAuthorityV1 {
    pub schema_version: u16,
    pub authority_kind: PracticeAuthorityKindV1,
    pub actor_org_id: u64,
    pub producer_content_digest: [u8; 32],
}

pub const PRACTICE_INPUT_AUTHORITY_V1_FIELD_ORDER: [&str; 4] = [
    "schema_version",
    "authority_kind",
    "actor_org_id",
    "producer_content_digest",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeParameterV1 {
    pub key_u8: u8,
    pub value_kind_u8: u8,
    pub value_length_u16: u16,
    pub value_bytes: Vec<u8>,
}

pub const PRACTICE_PARAMETER_V1_FIELD_ORDER: [&str; 4] =
    ["key_u8", "value_kind_u8", "value_length_u16", "value_bytes"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeIntentV1 {
    pub schema_version: u16,
    pub submit_after_tick: u64,
    pub resolve_tick: u64,
    pub actor_org_id: u64,
    pub practice_id: PracticeIdV1,
    pub target_domain: PracticeTargetDomainV1,
    pub target_node_id: u64,
    pub quoted_content_digest: [u8; 32],
    pub quoted_action_budget_cost: u32,
    pub parameters: Vec<PracticeParameterV1>,
    pub evidence_digests: Vec<[u8; 32]>,
}

pub const PRACTICE_INTENT_V1_FIELD_ORDER: [&str; 11] = [
    "schema_version",
    "submit_after_tick",
    "resolve_tick",
    "actor_org_id",
    "practice_id",
    "target_domain",
    "target_node_id",
    "quoted_content_digest",
    "quoted_action_budget_cost",
    "parameters",
    "evidence_digests",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyAuthorityPairV1 {
    pub producer_content_digest: [u8; 32],
    pub actor_org_id: u64,
}

pub const POLICY_AUTHORITY_PAIR_V1_FIELD_ORDER: [&str; 2] =
    ["producer_content_digest", "actor_org_id"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeAuthorityContextV1 {
    pub player_org_id: u64,
    pub player_gateway_content_digest: [u8; 32],
    pub policy_authorities: Vec<PolicyAuthorityPairV1>,
}

pub const PRACTICE_AUTHORITY_CONTEXT_V1_FIELD_ORDER: [&str; 3] = [
    "player_org_id",
    "player_gateway_content_digest",
    "policy_authorities",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeQuoteContextV1 {
    pub last_committed_tick: u64,
    pub content_digest: [u8; 32],
    pub budget_terms: PracticeBudgetTermsV1,
}

pub const PRACTICE_QUOTE_CONTEXT_V1_FIELD_ORDER: [&str; 3] =
    ["last_committed_tick", "content_digest", "budget_terms"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolidarityFootprintEdgeV1 {
    pub source_org_node_id_u64: u64,
    pub target_domain_u8: PracticeTargetDomainV1,
    pub target_class_node_id_u64: u64,
    pub strength_f64_bits_u64: u64,
}

pub const SOLIDARITY_FOOTPRINT_EDGE_V1_FIELD_ORDER: [&str; 4] = [
    "source_org_node_id_u64",
    "target_domain_u8",
    "target_class_node_id_u64",
    "strength_f64_bits_u64",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationPracticeTopologyEdgeV1 {
    pub target_domain: PracticeTargetDomainV1,
    pub target_class_node_id_u64: u64,
}

pub const ORGANIZATION_PRACTICE_TOPOLOGY_EDGE_V1_FIELD_ORDER: [&str; 2] =
    ["target_domain", "target_class_node_id_u64"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationPracticeTopologyRowV1 {
    pub node_id_u64: u64,
    pub active_bool: bool,
    pub action_budget_storage_f64_bits_u64: Option<u64>,
    pub edges: Vec<OrganizationPracticeTopologyEdgeV1>,
}

pub const ORGANIZATION_PRACTICE_TOPOLOGY_ROW_V1_FIELD_ORDER: [&str; 4] = [
    "node_id_u64",
    "active_bool",
    "action_budget_storage_f64_bits_u64",
    "edges",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationPracticeTopologyV1 {
    pub organizations: Vec<OrganizationPracticeTopologyRowV1>,
}

pub const ORGANIZATION_PRACTICE_TOPOLOGY_V1_FIELD_ORDER: [&str; 1] = ["organizations"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationBudgetDeltaV1 {
    pub schema_version: u16,
    pub tick: u64,
    pub actor_node_id: u64,
    pub pre_action_world_hash: [u8; 32],
    pub budget_before: u32,
    pub governed_cost: u32,
    pub footprint_count: u32,
    pub raw_credit: u32,
    pub credited_credit: u32,
    pub ceiling_bound: bool,
    pub budget_after: u32,
}

pub const ORGANIZATION_BUDGET_DELTA_V1_FIELD_ORDER: [&str; 11] = [
    "schema_version",
    "tick",
    "actor_node_id",
    "pre_action_world_hash",
    "budget_before",
    "governed_cost",
    "footprint_count",
    "raw_credit",
    "credited_credit",
    "ceiling_bound",
    "budget_after",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeSubmissionRejectionV1 {
    pub schema_version: u16,
    pub submitted_bytes_digest: [u8; 32],
    pub reason_code: PracticeRejectionCodeV1,
    pub last_committed_tick: u64,
    pub content_digest: [u8; 32],
}

pub const PRACTICE_SUBMISSION_REJECTION_V1_FIELD_ORDER: [&str; 5] = [
    "schema_version",
    "submitted_bytes_digest",
    "reason_code",
    "last_committed_tick",
    "content_digest",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeBudgetTermsV1 {
    pub initial: u32,
    pub weekly_credit_cap: u32,
    pub storage_ceiling: u32,
    pub organize_cost: u32,
    pub agitate_cost: u32,
    pub mutual_aid_cost: u32,
}

pub const PRACTICE_BUDGET_TERMS_V1_FIELD_ORDER: [&str; 6] = [
    "initial",
    "weekly_credit_cap",
    "storage_ceiling",
    "organize_cost",
    "agitate_cost",
    "mutual_aid_cost",
];

pub const MAX_PARAMETERS: usize = 16;
pub const MAX_PARAMETER_VALUE_BYTES: usize = 256;
pub const MAX_PARAMETER_BYTES: usize = 256;
pub const MAX_EVIDENCE_DIGESTS: usize = 64;
pub const MAX_INTENT_CANONICAL_BYTES: usize = 16384;
pub const MAX_POLICY_AUTHORITY_PAIRS: usize = 4096;
pub const MAX_INTENTS_PER_RESOLVE_TICK: usize = 4096;
pub const MAX_ORGANIZATIONS: usize = 4096;
pub const MAX_ORG_SOLIDARITY_EDGES_PER_ORG: usize = 256;
pub const MAX_JSONL_SOURCE_BYTES: usize = 2097152;
pub const MAX_JSONL_CASES: usize = 512;
pub const MAX_JSONL_LINE_BYTES: usize = 65536;
pub const MAX_JSONL_CASE_ID_BYTES: usize = 128;
pub const MAX_JSON_DEPTH: usize = 32;

#[must_use]
pub const fn practice_machine_verb(practice: PracticeIdV1) -> MachineVerbV1 {
    match practice {
        PracticeIdV1::Organize => MachineVerbV1 {
            stem: VerbStemV1::Mobilize,
            mode: Some(VerbModeV1::Canvass),
        },
        PracticeIdV1::Agitate => MachineVerbV1 {
            stem: VerbStemV1::Mobilize,
            mode: Some(VerbModeV1::Agitate),
        },
        PracticeIdV1::MutualAid => MachineVerbV1 {
            stem: VerbStemV1::Aid,
            mode: None,
        },
    }
}

/// Checks the shape-only intent collections against their contract bounds.
///
/// # Errors
///
/// Returns the assigned parameter or evidence bound error.
pub fn validate_intent_collection_bounds(
    value: &PracticeIntentV1,
) -> Result<(), PracticeContractError> {
    if value.parameters.len() > MAX_PARAMETERS {
        return Err(PracticeContractError::PracticeParameterLimit);
    }
    if value.evidence_digests.len() > MAX_EVIDENCE_DIGESTS {
        return Err(PracticeContractError::PracticeEvidenceLimit);
    }
    Ok(())
}

/// Checks the shape-only authority registry against its contract bound.
///
/// # Errors
///
/// Returns the assigned authority registry bound error.
pub fn validate_authority_context_collection_bounds(
    value: &PracticeAuthorityContextV1,
) -> Result<(), PracticeContractError> {
    if value.policy_authorities.len() > MAX_POLICY_AUTHORITY_PAIRS {
        return Err(PracticeContractError::PracticeAuthorityRegistryLimit);
    }
    Ok(())
}

/// Checks shape-only topology collections against their contract bounds.
///
/// # Errors
///
/// Returns the assigned organization or footprint bound error.
pub fn validate_topology_collection_bounds(
    value: &OrganizationPracticeTopologyV1,
) -> Result<(), PracticeContractError> {
    if value.organizations.len() > MAX_ORGANIZATIONS {
        return Err(PracticeContractError::PracticeTopologyOrganizationLimit);
    }
    for row in value.organizations.iter().take(MAX_ORGANIZATIONS + 1) {
        if row.edges.len() > MAX_ORG_SOLIDARITY_EDGES_PER_ORG {
            return Err(PracticeContractError::PracticeFootprintLimit);
        }
    }
    Ok(())
}
