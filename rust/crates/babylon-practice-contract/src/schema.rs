//! Detached organization budget and topology contracts.

use crate::PracticeId;

pub const ORGANIZATION_BUDGET_DELTA_DOMAIN_BYTES: &[u8] = b"babylon.organization-budget-delta.v1";
pub const PRACTICE_WIRE_DOMAIN_TERMINATOR_BYTES: &[u8] = b"\x00";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum PracticeContractError {
    PracticeDomain = 1,
    PracticeSchemaVersion = 2,
    PracticeEnumCode = 3,
    PracticeTruncated = 6,
    PracticeTrailingBytes = 7,
    PracticeBoolean = 9,
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
    PracticeBudgetUnpriced = 47,
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
            6 => Ok(Self::PracticeTruncated),
            7 => Ok(Self::PracticeTrailingBytes),
            9 => Ok(Self::PracticeBoolean),
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
            47 => Ok(Self::PracticeBudgetUnpriced),
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
pub enum VerbStem {
    Mobilize = 1,
    Aid = 2,
}

impl TryFrom<u8> for VerbStem {
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
pub enum VerbMode {
    Canvass = 1,
    Agitate = 2,
}

impl TryFrom<u8> for VerbMode {
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
pub enum PracticeTargetDomain {
    SocialClass = 1,
}

impl TryFrom<u8> for PracticeTargetDomain {
    type Error = PracticeContractError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::SocialClass),
            _ => Err(PracticeContractError::PracticeEnumCode),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MachineVerb {
    pub stem: VerbStem,
    pub mode: Option<VerbMode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolidarityFootprintEdge {
    pub source_org_node_id_u64: u64,
    pub target_domain_u8: PracticeTargetDomain,
    pub target_class_node_id_u64: u64,
    pub strength_f64_bits_u64: u64,
}

pub const SOLIDARITY_FOOTPRINT_EDGE_FIELD_ORDER: [&str; 4] = [
    "source_org_node_id_u64",
    "target_domain_u8",
    "target_class_node_id_u64",
    "strength_f64_bits_u64",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationPracticeTopologyEdge {
    pub target_domain: PracticeTargetDomain,
    pub target_class_node_id_u64: u64,
}

pub const ORGANIZATION_PRACTICE_TOPOLOGY_EDGE_FIELD_ORDER: [&str; 2] =
    ["target_domain", "target_class_node_id_u64"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationPracticeTopologyRow {
    pub node_id_u64: u64,
    pub active_bool: bool,
    pub action_budget_storage_f64_bits_u64: Option<u64>,
    pub edges: Vec<OrganizationPracticeTopologyEdge>,
}

pub const ORGANIZATION_PRACTICE_TOPOLOGY_ROW_FIELD_ORDER: [&str; 4] = [
    "node_id_u64",
    "active_bool",
    "action_budget_storage_f64_bits_u64",
    "edges",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationPracticeTopology {
    pub organizations: Vec<OrganizationPracticeTopologyRow>,
}

pub const ORGANIZATION_PRACTICE_TOPOLOGY_FIELD_ORDER: [&str; 1] = ["organizations"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationBudgetDelta {
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

pub const ORGANIZATION_BUDGET_DELTA_FIELD_ORDER: [&str; 11] = [
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
pub struct PracticeBudgetTerms {
    pub initial: u32,
    pub period_credit_cap: u32,
    pub storage_ceiling: u32,
    pub organize_cost: u32,
    pub agitate_cost: u32,
    pub mutual_aid_cost: u32,
}

pub const PRACTICE_BUDGET_TERMS_FIELD_ORDER: [&str; 6] = [
    "initial",
    "period_credit_cap",
    "storage_ceiling",
    "organize_cost",
    "agitate_cost",
    "mutual_aid_cost",
];

pub const MAX_ORGANIZATIONS: usize = 4096;
pub const MAX_ORG_SOLIDARITY_EDGES_PER_ORG: usize = 256;

/// Returns the authored machine verb only for practices with a declared mapping.
#[must_use]
pub const fn practice_machine_verb(practice: PracticeId) -> Option<MachineVerb> {
    match practice {
        PracticeId::Organize => Some(MachineVerb {
            stem: VerbStem::Mobilize,
            mode: Some(VerbMode::Canvass),
        }),
        PracticeId::Agitate => Some(MachineVerb {
            stem: VerbStem::Mobilize,
            mode: Some(VerbMode::Agitate),
        }),
        PracticeId::MutualAid => Some(MachineVerb {
            stem: VerbStem::Aid,
            mode: None,
        }),
        PracticeId::Strike
        | PracticeId::Blockade
        | PracticeId::Occupation
        | PracticeId::Damage
        | PracticeId::CapitalStrike => None,
    }
}
