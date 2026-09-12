// Rust schema implementation paired with contracts/relational_territory_dossier_v1.yaml.

use serde::Deserialize;

fn deserialize_required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Audience {
    #[serde(rename = "ADMIN_MATERIAL")]
    AdminMaterial,
    #[serde(rename = "PLAYER_KNOWLEDGE")]
    PlayerKnowledge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Durability {
    #[serde(rename = "IN_MEMORY")]
    InMemory,
    #[serde(rename = "COMMITTED")]
    Committed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum EvidenceClass {
    #[serde(rename = "Observed")]
    Observed,
    #[serde(rename = "Derived")]
    Derived,
    #[serde(rename = "Calibrated")]
    Calibrated,
    #[serde(rename = "Designed")]
    Designed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Status {
    #[serde(rename = "PRESENT")]
    Present,
    #[serde(rename = "ABSENT")]
    Absent,
    #[serde(rename = "UNKNOWN")]
    Unknown,
    #[serde(rename = "NOT_COMPUTED")]
    NotComputed,
    #[serde(rename = "REDACTED")]
    Redacted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum ValueKind {
    #[serde(rename = "UINT64_BITS")]
    Uint64Bits,
    #[serde(rename = "FLOAT64_BITS")]
    Float64Bits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Coverage {
    #[serde(rename = "COMPLETE")]
    Complete,
    #[serde(rename = "PARTIAL")]
    Partial,
    #[serde(rename = "NOT_APPLICABLE")]
    NotApplicable,
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum MembershipKind {
    #[serde(rename = "ADMINISTRATIVE")]
    Administrative,
    #[serde(rename = "NATIONAL")]
    National,
    #[serde(rename = "COMMUTING_ZONE")]
    CommutingZone,
    #[serde(rename = "METROPOLITAN")]
    Metropolitan,
    #[serde(rename = "WEIGHTED_OVERLAP")]
    WeightedOverlap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum FacetFamily {
    #[serde(rename = "COMMAND_ADMINISTRATION")]
    CommandAdministration,
    #[serde(rename = "PRODUCTION_CIRCULATION")]
    ProductionCirculation,
    #[serde(rename = "REPRODUCTION_SETTLEMENT_ACCESS")]
    ReproductionSettlementAccess,
    #[serde(rename = "EXTRACTION_ABANDONMENT_CARCERAL")]
    ExtractionAbandonmentCarceral,
    #[serde(rename = "ECOLOGY_CARE")]
    EcologyCare,
    #[serde(rename = "ORGANIZATION_ROOTEDNESS")]
    OrganizationRootedness,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum DyadKind {
    #[serde(rename = "PRESENCE")]
    Presence,
    #[serde(rename = "MEMBERSHIP")]
    Membership,
    #[serde(rename = "SOLIDARITY")]
    Solidarity,
    #[serde(rename = "COMMAND")]
    Command,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum HyperedgeKind {
    #[serde(rename = "PUBLIC_RELATION")]
    PublicRelation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum FlowKind {
    #[serde(rename = "COMMUTER_JOBS")]
    CommuterJobs,
    #[serde(rename = "BORDER_SYNTHESIS")]
    BorderSynthesis,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum RelationPayloadMode {
    #[serde(rename = "EMPTY")]
    Empty,
    #[serde(rename = "SINGLE_METRIC_FACET")]
    SingleMetricFacet,
    #[serde(rename = "IMPLICIT_RELATION")]
    ImplicitRelation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum GapReason {
    #[serde(rename = "MISSING_GOVERNED_OMB_DELINEATION")]
    MissingGovernedOmbDelineation,
    #[serde(rename = "IDENTITY_CONTRACT_PENDING")]
    IdentityContractPending,
    #[serde(rename = "MISSING_GOVERNED_PRODUCER")]
    MissingGovernedProducer,
    #[serde(rename = "REFERENCE_COVERAGE_UNAVAILABLE")]
    ReferenceCoverageUnavailable,
    #[serde(rename = "PLAYER_BOUNDARY_UNAVAILABLE")]
    PlayerBoundaryUnavailable,
    #[serde(rename = "PROVENANCE_COORDINATE_CONFLICT")]
    ProvenanceCoordinateConflict,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum MetricRepresentation {
    #[serde(rename = "FACET")]
    Facet,
    #[serde(rename = "REFERENCE_FLOW")]
    ReferenceFlow,
    #[serde(rename = "DYAD")]
    Dyad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum AggregationRule {
    #[serde(rename = "NONE")]
    None,
    #[serde(rename = "PUBLISHED_ROLLUP")]
    PublishedRollup,
    #[serde(rename = "LOAD_TIME_SUM")]
    LoadTimeSum,
    #[serde(rename = "BLOCK_INTERNAL_POINT_ASSIGNMENT")]
    BlockInternalPointAssignment,
    #[serde(rename = "BLOCK_COORDINATE_ASSIGNMENT")]
    BlockCoordinateAssignment,
    #[serde(rename = "EQUAL_AREA_WATER_INTERSECTION")]
    EqualAreaWaterIntersection,
    #[serde(rename = "TYPED_RELATION_PROJECTION")]
    TypedRelationProjection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum RtdCollectionKind {
    #[serde(rename = "FOCUS")]
    Focus,
    #[serde(rename = "REFERENCE_DIGESTS")]
    ReferenceDigests,
    #[serde(rename = "SCALE_MEMBERSHIPS")]
    ScaleMemberships,
    #[serde(rename = "FACETS")]
    Facets,
    #[serde(rename = "DYADS")]
    Dyads,
    #[serde(rename = "HYPEREDGES")]
    Hyperedges,
    #[serde(rename = "FLOWS")]
    Flows,
    #[serde(rename = "GAPS")]
    Gaps,
    #[serde(rename = "PROVENANCE")]
    Provenance,
    #[serde(rename = "COORDINATES")]
    Coordinates,
    #[serde(rename = "MEMBER_REFS")]
    MemberRefs,
    #[serde(rename = "PAYLOAD_FACETS")]
    PayloadFacets,
    #[serde(rename = "DISPLAY_REFS")]
    DisplayRefs,
    #[serde(rename = "PROVENANCE_REFS")]
    ProvenanceRefs,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedIdentity {
    pub domain: String,
    pub authority: String,
    pub local_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceDigest {
    pub reference_id: TypedIdentity,
    pub sha256_hex: String,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub artifact_schema_id_or_null: Option<TypedIdentity>,
    pub vintage: String,
    pub evidence_class: EvidenceClass,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DimensionCoordinate {
    pub dimension_ref: TypedIdentity,
    pub member_ref: TypedIdentity,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaleMembership {
    pub membership_id: TypedIdentity,
    pub member_ref: TypedIdentity,
    pub scale_ref: TypedIdentity,
    pub membership_kind: MembershipKind,
    pub status: Status,
    pub weight_status: Status,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub weight_bits_or_null: Option<String>,
    pub coverage: Coverage,
    pub evidence_class: EvidenceClass,
    pub provenance_refs: Vec<TypedIdentity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facet {
    pub facet_id: TypedIdentity,
    pub family: FacetFamily,
    pub subject_ref: TypedIdentity,
    pub metric_id: TypedIdentity,
    pub unit_id: TypedIdentity,
    pub native_scale: TypedIdentity,
    pub coordinates: Vec<DimensionCoordinate>,
    pub vintage: String,
    pub status: Status,
    pub value_kind: ValueKind,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub value_bits_or_null: Option<String>,
    pub coverage: Coverage,
    pub evidence_class: EvidenceClass,
    pub provenance_refs: Vec<TypedIdentity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dyad {
    pub relation_id: TypedIdentity,
    pub relation_kind: DyadKind,
    pub from_ref: TypedIdentity,
    pub to_ref: TypedIdentity,
    pub native_scale: TypedIdentity,
    pub status: Status,
    pub coverage: Coverage,
    pub payload_facets: Vec<TypedIdentity>,
    pub evidence_class: EvidenceClass,
    pub provenance_refs: Vec<TypedIdentity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hyperedge {
    pub hyperedge_id: TypedIdentity,
    pub hyperedge_kind: HyperedgeKind,
    pub member_refs: Vec<TypedIdentity>,
    pub native_scale: TypedIdentity,
    pub status: Status,
    pub coverage: Coverage,
    pub payload_facets: Vec<TypedIdentity>,
    pub evidence_class: EvidenceClass,
    pub provenance_refs: Vec<TypedIdentity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceFlow {
    pub flow_id: TypedIdentity,
    pub flow_kind: FlowKind,
    pub origin_ref: TypedIdentity,
    pub destination_ref: TypedIdentity,
    pub payload_facets: Vec<TypedIdentity>,
    pub native_scale: TypedIdentity,
    pub status: Status,
    pub coverage: Coverage,
    pub evidence_class: EvidenceClass,
    pub provenance_refs: Vec<TypedIdentity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gap {
    pub gap_id: TypedIdentity,
    pub requested_metric_or_relation: TypedIdentity,
    pub status: Status,
    pub reason_code: GapReason,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub required_producer_or_null: Option<String>,
    pub provenance_refs: Vec<TypedIdentity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub provenance_id: TypedIdentity,
    pub artifact_digest: String,
    pub locator: String,
    pub vintage: String,
    pub evidence_class: EvidenceClass,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub transformation_digest_or_null: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionSurface {
    pub question_id: TypedIdentity,
    pub signal_refs: Vec<TypedIdentity>,
    pub action_refs: Vec<TypedIdentity>,
    pub receipt_refs: Vec<TypedIdentity>,
    pub archive_subject_refs: Vec<TypedIdentity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RtdDossierDraft {
    pub schema: String,
    pub schema_version: u16,
    pub projection_version: u16,
    pub audience: Audience,
    pub durability: Durability,
    pub verified_tick: u64,
    pub graph_state_hash: String,
    pub nominal_world_hash: String,
    pub reference_digests: Vec<ReferenceDigest>,
    pub definitions_digest: String,
    pub template_digest: String,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub fog_policy_digest: Option<String>,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub knowledge_context_digest: Option<String>,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub actor: Option<TypedIdentity>,
    pub focus: Vec<TypedIdentity>,
    pub scale_memberships: Vec<ScaleMembership>,
    pub facets: Vec<Facet>,
    pub dyads: Vec<Dyad>,
    pub hyperedges: Vec<Hyperedge>,
    pub flows: Vec<ReferenceFlow>,
    pub gaps: Vec<Gap>,
    pub provenance: Vec<Provenance>,
    pub decision_surface: DecisionSurface,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalTerritoryDossier {
    pub schema: String,
    pub schema_version: u16,
    pub projection_version: u16,
    pub audience: Audience,
    pub durability: Durability,
    pub verified_tick: u64,
    pub graph_state_hash: String,
    pub nominal_world_hash: String,
    pub reference_digests: Vec<ReferenceDigest>,
    pub definitions_digest: String,
    pub template_digest: String,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub fog_policy_digest: Option<String>,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub knowledge_context_digest: Option<String>,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub actor: Option<TypedIdentity>,
    pub focus: Vec<TypedIdentity>,
    pub scale_memberships: Vec<ScaleMembership>,
    pub facets: Vec<Facet>,
    pub dyads: Vec<Dyad>,
    pub hyperedges: Vec<Hyperedge>,
    pub flows: Vec<ReferenceFlow>,
    pub gaps: Vec<Gap>,
    pub provenance: Vec<Provenance>,
    pub decision_surface: DecisionSurface,
    pub projection_hash: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedIdentityLiteral {
    pub domain: &'static str,
    pub authority: &'static str,
    pub local_id: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RtdIdentityRegistryRow {
    pub category: &'static str,
    pub symbolic_name: &'static str,
    pub identity: TypedIdentityLiteral,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RtdMetricRegistryRow {
    pub metric: TypedIdentityLiteral,
    pub representation: MetricRepresentation,
    pub unit: TypedIdentityLiteral,
    pub value_kind: Option<ValueKind>,
    pub native_scale: TypedIdentityLiteral,
    pub coordinates: &'static [TypedIdentityLiteral],
    pub evidence_classes: &'static [EvidenceClass],
    pub aggregation_rule: AggregationRule,
    pub producer: TypedIdentityLiteral,
    pub reference_artifact: Option<TypedIdentityLiteral>,
    pub reference_digest: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RtdRelationBindingRegistryRow {
    pub record_family: &'static str,
    pub kind: &'static str,
    pub metric: Option<TypedIdentityLiteral>,
    pub payload_mode: RelationPayloadMode,
}

pub const RTD_SCHEMA_ID: &str = "babylon.relational-territory-dossier";
pub const RTD_MAX_COLLECTION_ITEMS: u64 = 65535;
pub const RTD_MAX_FOCUS: u64 = 64;
pub const RTD_MAX_REFERENCE_DIGESTS: u64 = 4096;
pub const RTD_MAX_SCALE_MEMBERSHIPS: u64 = 65535;
pub const RTD_MAX_FACETS: u64 = 65535;
pub const RTD_MAX_DYADS: u64 = 65535;
pub const RTD_MAX_HYPEREDGES: u64 = 65535;
pub const RTD_MAX_FLOWS: u64 = 65535;
pub const RTD_MAX_GAPS: u64 = 65535;
pub const RTD_MAX_PROVENANCE: u64 = 65535;
pub const RTD_MAX_COORDINATES: u64 = 32;
pub const RTD_MAX_HYPEREDGE_MEMBERS: u64 = 1024;
pub const RTD_MAX_PAYLOAD_FACETS: u64 = 256;
pub const RTD_MAX_DECISION_SURFACE_REFS: u64 = 256;
pub const RTD_MAX_PROVENANCE_REFS: u64 = 8192;
pub const RTD_MAX_IDENTITY_COMPONENT_BYTES: u64 = 256;
pub const RTD_MAX_VINTAGE_BYTES: u64 = 256;
pub const RTD_MAX_PROVENANCE_LOCATOR_BYTES: u64 = 1024;
pub const RTD_MAX_REQUIRED_PRODUCER_BYTES: u64 = 64;
pub const RTD_MAX_CANONICAL_BYTES: u64 = 67108864;

pub const RTD_LIMITS: &[(&str, u64)] = &[
    ("max_collection_items", RTD_MAX_COLLECTION_ITEMS),
    ("max_focus", RTD_MAX_FOCUS),
    ("max_reference_digests", RTD_MAX_REFERENCE_DIGESTS),
    ("max_scale_memberships", RTD_MAX_SCALE_MEMBERSHIPS),
    ("max_facets", RTD_MAX_FACETS),
    ("max_dyads", RTD_MAX_DYADS),
    ("max_hyperedges", RTD_MAX_HYPEREDGES),
    ("max_flows", RTD_MAX_FLOWS),
    ("max_gaps", RTD_MAX_GAPS),
    ("max_provenance", RTD_MAX_PROVENANCE),
    ("max_coordinates", RTD_MAX_COORDINATES),
    ("max_hyperedge_members", RTD_MAX_HYPEREDGE_MEMBERS),
    ("max_payload_facets", RTD_MAX_PAYLOAD_FACETS),
    ("max_decision_surface_refs", RTD_MAX_DECISION_SURFACE_REFS),
    ("max_provenance_refs", RTD_MAX_PROVENANCE_REFS),
    (
        "max_identity_component_bytes",
        RTD_MAX_IDENTITY_COMPONENT_BYTES,
    ),
    ("max_vintage_bytes", RTD_MAX_VINTAGE_BYTES),
    (
        "max_provenance_locator_bytes",
        RTD_MAX_PROVENANCE_LOCATOR_BYTES,
    ),
    (
        "max_required_producer_bytes",
        RTD_MAX_REQUIRED_PRODUCER_BYTES,
    ),
    ("max_canonical_bytes", RTD_MAX_CANONICAL_BYTES),
];

pub const RTD_ERROR_REGISTRY: &[&str] = &[
    "RTD_JSON",
    "RTD_JSON_DEPTH",
    "RTD_SCHEMA_VERSION",
    "RTD_UNKNOWN_FIELD",
    "RTD_ENUM",
    "RTD_IDENTITY",
    "RTD_DIGEST",
    "RTD_NON_NFC",
    "RTD_LIMIT_EXCEEDED",
    "RTD_DUPLICATE_KEY",
    "RTD_DANGLING_REF",
    "RTD_STATUS_VALUE",
    "RTD_NATIVE_GRAIN",
    "RTD_UNSUPPORTED_DOWNSCALE",
    "RTD_H3_BEFORE_PER21",
    "RTD_MSA_EVIDENCE",
    "RTD_CANADA_CONTROL",
    "RTD_FORBIDDEN_REDUCTION",
    "RTD_VECTOR_LIMIT",
    "RTD_CANONICAL_SIZE",
];

pub const RTD_IDENTITY_REGISTRY: &[RtdIdentityRegistryRow] = &[
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "production/qcew-leaf-employment",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-employment",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "production/qcew-leaf-establishments",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-establishments",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "production/qcew-leaf-total-wages-usd",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-total-wages-usd",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "production/qcew-leaf-average-annual-pay-usd",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-average-annual-pay-usd",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "production/qcew-county-employment",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-county-employment",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "production/qcew-county-establishments",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-county-establishments",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "production/qcew-county-total-wages-usd",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-county-total-wages-usd",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "circulation/lodes-county-commuter-total-jobs",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "circulation/lodes-county-commuter-total-jobs",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "reproduction/census-housing-households",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/census-housing-households",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "reproduction/census-median-rent-usd",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/census-median-rent-usd",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "reproduction/census-rent-burden-households",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/census-rent-burden-households",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "reproduction/h3-population-persons",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/h3-population-persons",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "production/h3-workplace-jobs",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/h3-workplace-jobs",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "carceral/facility-count",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "carceral/facility-count",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "ecology/h3-land-fraction",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "ecology/h3-land-fraction",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "rootedness/presence",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/presence",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "rootedness/solidarity",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/solidarity",
        },
    },
    RtdIdentityRegistryRow {
        category: "metrics",
        symbolic_name: "rootedness/membership",
        identity: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/membership",
        },
    },
    RtdIdentityRegistryRow {
        category: "units",
        symbolic_name: "JOBS",
        identity: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "jobs",
        },
    },
    RtdIdentityRegistryRow {
        category: "units",
        symbolic_name: "ESTABLISHMENTS",
        identity: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "establishments",
        },
    },
    RtdIdentityRegistryRow {
        category: "units",
        symbolic_name: "USD_CURRENT",
        identity: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "usd-current",
        },
    },
    RtdIdentityRegistryRow {
        category: "units",
        symbolic_name: "HOUSEHOLDS",
        identity: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "households",
        },
    },
    RtdIdentityRegistryRow {
        category: "units",
        symbolic_name: "PERSONS",
        identity: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "persons",
        },
    },
    RtdIdentityRegistryRow {
        category: "units",
        symbolic_name: "FACILITIES",
        identity: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "facilities",
        },
    },
    RtdIdentityRegistryRow {
        category: "units",
        symbolic_name: "FRACTION",
        identity: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "fraction",
        },
    },
    RtdIdentityRegistryRow {
        category: "units",
        symbolic_name: "TYPED_RELATION",
        identity: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "typed-relation",
        },
    },
    RtdIdentityRegistryRow {
        category: "coordinates",
        symbolic_name: "county",
        identity: TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "county",
        },
    },
    RtdIdentityRegistryRow {
        category: "coordinates",
        symbolic_name: "naics6",
        identity: TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "naics6",
        },
    },
    RtdIdentityRegistryRow {
        category: "coordinates",
        symbolic_name: "ownership",
        identity: TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "ownership",
        },
    },
    RtdIdentityRegistryRow {
        category: "coordinates",
        symbolic_name: "home_county",
        identity: TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "home-county",
        },
    },
    RtdIdentityRegistryRow {
        category: "coordinates",
        symbolic_name: "work_county",
        identity: TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "work-county",
        },
    },
    RtdIdentityRegistryRow {
        category: "coordinates",
        symbolic_name: "source",
        identity: TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "source",
        },
    },
    RtdIdentityRegistryRow {
        category: "coordinates",
        symbolic_name: "tenure",
        identity: TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "tenure",
        },
    },
    RtdIdentityRegistryRow {
        category: "coordinates",
        symbolic_name: "race",
        identity: TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "race",
        },
    },
    RtdIdentityRegistryRow {
        category: "coordinates",
        symbolic_name: "burden",
        identity: TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "burden",
        },
    },
    RtdIdentityRegistryRow {
        category: "coordinates",
        symbolic_name: "h3_cell",
        identity: TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "h3-cell",
        },
    },
    RtdIdentityRegistryRow {
        category: "coordinates",
        symbolic_name: "coercive_type",
        identity: TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "coercive-type",
        },
    },
    RtdIdentityRegistryRow {
        category: "coordinates",
        symbolic_name: "actor",
        identity: TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "actor",
        },
    },
    RtdIdentityRegistryRow {
        category: "coordinates",
        symbolic_name: "node",
        identity: TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "node",
        },
    },
    RtdIdentityRegistryRow {
        category: "native_scales",
        symbolic_name: "COUNTY_NAICS6_OWNERSHIP_YEAR",
        identity: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-naics6-ownership-year",
        },
    },
    RtdIdentityRegistryRow {
        category: "native_scales",
        symbolic_name: "COUNTY_OWNERSHIP_YEAR",
        identity: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-ownership-year",
        },
    },
    RtdIdentityRegistryRow {
        category: "native_scales",
        symbolic_name: "HOME_COUNTY_WORK_COUNTY_YEAR",
        identity: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "home-county-work-county-year",
        },
    },
    RtdIdentityRegistryRow {
        category: "native_scales",
        symbolic_name: "COUNTY_SOURCE_TENURE_TIME_RACE",
        identity: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-source-tenure-time-race",
        },
    },
    RtdIdentityRegistryRow {
        category: "native_scales",
        symbolic_name: "COUNTY_SOURCE_TIME_RACE",
        identity: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-source-time-race",
        },
    },
    RtdIdentityRegistryRow {
        category: "native_scales",
        symbolic_name: "COUNTY_SOURCE_BURDEN_TIME_RACE",
        identity: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-source-burden-time-race",
        },
    },
    RtdIdentityRegistryRow {
        category: "native_scales",
        symbolic_name: "H3_R7_VINTAGE",
        identity: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "h3-r7-vintage",
        },
    },
    RtdIdentityRegistryRow {
        category: "native_scales",
        symbolic_name: "COUNTY_COERCIVE_TYPE_SOURCE",
        identity: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-coercive-type-source",
        },
    },
    RtdIdentityRegistryRow {
        category: "native_scales",
        symbolic_name: "ACTOR_NODE_VERIFIED_TICK",
        identity: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "actor-node-verified-tick",
        },
    },
    RtdIdentityRegistryRow {
        category: "producers",
        symbolic_name: "fact_qcew_annual",
        identity: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        },
    },
    RtdIdentityRegistryRow {
        category: "producers",
        symbolic_name: "fact_qcew_county_rollup",
        identity: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        },
    },
    RtdIdentityRegistryRow {
        category: "producers",
        symbolic_name: "fact_lodes_commuter_flow",
        identity: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_lodes_commuter_flow",
        },
    },
    RtdIdentityRegistryRow {
        category: "producers",
        symbolic_name: "fact_census_housing",
        identity: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_census_housing",
        },
    },
    RtdIdentityRegistryRow {
        category: "producers",
        symbolic_name: "fact_census_rent",
        identity: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent",
        },
    },
    RtdIdentityRegistryRow {
        category: "producers",
        symbolic_name: "fact_census_rent_burden",
        identity: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent_burden",
        },
    },
    RtdIdentityRegistryRow {
        category: "producers",
        symbolic_name: "h3_res7_population",
        identity: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "h3_res7_population",
        },
    },
    RtdIdentityRegistryRow {
        category: "producers",
        symbolic_name: "h3_res7_workplace",
        identity: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "h3_res7_workplace",
        },
    },
    RtdIdentityRegistryRow {
        category: "producers",
        symbolic_name: "fact_coercive_infrastructure",
        identity: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_coercive_infrastructure",
        },
    },
    RtdIdentityRegistryRow {
        category: "producers",
        symbolic_name: "h3_res7_land_mask",
        identity: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "h3_res7_land_mask",
        },
    },
    RtdIdentityRegistryRow {
        category: "producers",
        symbolic_name: "committed typed graph",
        identity: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.engine",
            local_id: "typed-graph-relations-at-verified-tick",
        },
    },
    RtdIdentityRegistryRow {
        category: "references",
        symbolic_name: "fact_qcew_annual",
        identity: TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        },
    },
    RtdIdentityRegistryRow {
        category: "references",
        symbolic_name: "fact_qcew_county_rollup",
        identity: TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        },
    },
    RtdIdentityRegistryRow {
        category: "references",
        symbolic_name: "fact_lodes_commuter_flow",
        identity: TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_lodes_commuter_flow",
        },
    },
    RtdIdentityRegistryRow {
        category: "references",
        symbolic_name: "fact_census_housing",
        identity: TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_census_housing",
        },
    },
    RtdIdentityRegistryRow {
        category: "references",
        symbolic_name: "fact_census_rent",
        identity: TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent",
        },
    },
    RtdIdentityRegistryRow {
        category: "references",
        symbolic_name: "fact_census_rent_burden",
        identity: TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent_burden",
        },
    },
    RtdIdentityRegistryRow {
        category: "references",
        symbolic_name: "h3_res7_population",
        identity: TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "h3_res7_population",
        },
    },
    RtdIdentityRegistryRow {
        category: "references",
        symbolic_name: "h3_res7_workplace",
        identity: TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "h3_res7_workplace",
        },
    },
    RtdIdentityRegistryRow {
        category: "references",
        symbolic_name: "fact_coercive_infrastructure",
        identity: TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_coercive_infrastructure",
        },
    },
    RtdIdentityRegistryRow {
        category: "references",
        symbolic_name: "h3_res7_land_mask",
        identity: TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "h3_res7_land_mask",
        },
    },
];

pub const RTD_METRIC_REGISTRY: &[RtdMetricRegistryRow] = &[
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-employment",
        },
        representation: MetricRepresentation::Facet,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "jobs",
        },
        value_kind: Some(ValueKind::Uint64Bits),
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-naics6-ownership-year",
        },
        coordinates: &[
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "naics6",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "ownership",
            },
        ],
        evidence_classes: &[EvidenceClass::Observed, EvidenceClass::Derived],
        aggregation_rule: AggregationRule::None,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        },
        reference_artifact: Some(TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        }),
        reference_digest: Some("ca3825a3d60831479313632073b7fc9a941d57dcf9b8940181c4713b6d442248"),
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-establishments",
        },
        representation: MetricRepresentation::Facet,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "establishments",
        },
        value_kind: Some(ValueKind::Uint64Bits),
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-naics6-ownership-year",
        },
        coordinates: &[
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "naics6",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "ownership",
            },
        ],
        evidence_classes: &[EvidenceClass::Observed, EvidenceClass::Derived],
        aggregation_rule: AggregationRule::None,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        },
        reference_artifact: Some(TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        }),
        reference_digest: Some("ca3825a3d60831479313632073b7fc9a941d57dcf9b8940181c4713b6d442248"),
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-total-wages-usd",
        },
        representation: MetricRepresentation::Facet,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "usd-current",
        },
        value_kind: Some(ValueKind::Float64Bits),
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-naics6-ownership-year",
        },
        coordinates: &[
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "naics6",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "ownership",
            },
        ],
        evidence_classes: &[EvidenceClass::Observed, EvidenceClass::Derived],
        aggregation_rule: AggregationRule::None,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        },
        reference_artifact: Some(TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        }),
        reference_digest: Some("ca3825a3d60831479313632073b7fc9a941d57dcf9b8940181c4713b6d442248"),
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-average-annual-pay-usd",
        },
        representation: MetricRepresentation::Facet,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "usd-current",
        },
        value_kind: Some(ValueKind::Float64Bits),
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-naics6-ownership-year",
        },
        coordinates: &[
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "naics6",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "ownership",
            },
        ],
        evidence_classes: &[EvidenceClass::Observed, EvidenceClass::Derived],
        aggregation_rule: AggregationRule::None,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        },
        reference_artifact: Some(TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        }),
        reference_digest: Some("ca3825a3d60831479313632073b7fc9a941d57dcf9b8940181c4713b6d442248"),
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-county-employment",
        },
        representation: MetricRepresentation::Facet,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "jobs",
        },
        value_kind: Some(ValueKind::Uint64Bits),
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-ownership-year",
        },
        coordinates: &[
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "ownership",
            },
        ],
        evidence_classes: &[EvidenceClass::Observed, EvidenceClass::Derived],
        aggregation_rule: AggregationRule::PublishedRollup,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        },
        reference_artifact: Some(TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        }),
        reference_digest: Some("34c2bbb935f79b3c8076a97092b004b14cca120e8272b93c35b3ac9dc2721d13"),
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-county-establishments",
        },
        representation: MetricRepresentation::Facet,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "establishments",
        },
        value_kind: Some(ValueKind::Uint64Bits),
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-ownership-year",
        },
        coordinates: &[
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "ownership",
            },
        ],
        evidence_classes: &[EvidenceClass::Observed, EvidenceClass::Derived],
        aggregation_rule: AggregationRule::PublishedRollup,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        },
        reference_artifact: Some(TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        }),
        reference_digest: Some("34c2bbb935f79b3c8076a97092b004b14cca120e8272b93c35b3ac9dc2721d13"),
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-county-total-wages-usd",
        },
        representation: MetricRepresentation::Facet,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "usd-current",
        },
        value_kind: Some(ValueKind::Float64Bits),
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-ownership-year",
        },
        coordinates: &[
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "ownership",
            },
        ],
        evidence_classes: &[EvidenceClass::Observed, EvidenceClass::Derived],
        aggregation_rule: AggregationRule::PublishedRollup,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        },
        reference_artifact: Some(TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        }),
        reference_digest: Some("34c2bbb935f79b3c8076a97092b004b14cca120e8272b93c35b3ac9dc2721d13"),
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "circulation/lodes-county-commuter-total-jobs",
        },
        representation: MetricRepresentation::ReferenceFlow,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "jobs",
        },
        value_kind: Some(ValueKind::Uint64Bits),
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "home-county-work-county-year",
        },
        coordinates: &[
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "home-county",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "work-county",
            },
        ],
        evidence_classes: &[EvidenceClass::Derived],
        aggregation_rule: AggregationRule::LoadTimeSum,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_lodes_commuter_flow",
        },
        reference_artifact: Some(TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_lodes_commuter_flow",
        }),
        reference_digest: Some("d3745f8def09cd8c7a38e1870e6ec2c1853e210b777d8e8358cfce36665bd64d"),
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/census-housing-households",
        },
        representation: MetricRepresentation::Facet,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "households",
        },
        value_kind: Some(ValueKind::Uint64Bits),
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-source-tenure-time-race",
        },
        coordinates: &[
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "source",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "tenure",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "race",
            },
        ],
        evidence_classes: &[EvidenceClass::Observed],
        aggregation_rule: AggregationRule::None,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_census_housing",
        },
        reference_artifact: Some(TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_census_housing",
        }),
        reference_digest: Some("09ff2d9666b3f5ef267b65cbc77c14e99384f0157b6a4c898ac37df2e67ca59f"),
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/census-median-rent-usd",
        },
        representation: MetricRepresentation::Facet,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "usd-current",
        },
        value_kind: Some(ValueKind::Float64Bits),
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-source-time-race",
        },
        coordinates: &[
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "source",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "race",
            },
        ],
        evidence_classes: &[EvidenceClass::Observed],
        aggregation_rule: AggregationRule::None,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent",
        },
        reference_artifact: Some(TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent",
        }),
        reference_digest: Some("4c8cc134ec490ca75961d83485fc97c6bf240b32128e9d0517e00e62d578a99e"),
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/census-rent-burden-households",
        },
        representation: MetricRepresentation::Facet,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "households",
        },
        value_kind: Some(ValueKind::Uint64Bits),
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-source-burden-time-race",
        },
        coordinates: &[
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "source",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "burden",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "race",
            },
        ],
        evidence_classes: &[EvidenceClass::Observed],
        aggregation_rule: AggregationRule::None,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent_burden",
        },
        reference_artifact: Some(TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent_burden",
        }),
        reference_digest: Some("8a42a51c17bf3ebee09f0b0b5145d5c8253c7e3446eec8c75714f9951b20df12"),
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/h3-population-persons",
        },
        representation: MetricRepresentation::Facet,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "persons",
        },
        value_kind: Some(ValueKind::Uint64Bits),
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "h3-r7-vintage",
        },
        coordinates: &[TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "h3-cell",
        }],
        evidence_classes: &[EvidenceClass::Derived],
        aggregation_rule: AggregationRule::BlockInternalPointAssignment,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "h3_res7_population",
        },
        reference_artifact: Some(TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "h3_res7_population",
        }),
        reference_digest: Some("b096a5891284f0ca55bedae9d1a9092eb8ea9e9e32d32b6ace430a9833b53afc"),
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/h3-workplace-jobs",
        },
        representation: MetricRepresentation::Facet,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "jobs",
        },
        value_kind: Some(ValueKind::Uint64Bits),
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "h3-r7-vintage",
        },
        coordinates: &[TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "h3-cell",
        }],
        evidence_classes: &[EvidenceClass::Derived],
        aggregation_rule: AggregationRule::BlockCoordinateAssignment,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "h3_res7_workplace",
        },
        reference_artifact: Some(TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "h3_res7_workplace",
        }),
        reference_digest: Some("ea2ce1508f4fe51f1e879b9f4a1daf579c4b00349388b12a85f884a8f49eabb6"),
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "carceral/facility-count",
        },
        representation: MetricRepresentation::Facet,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "facilities",
        },
        value_kind: Some(ValueKind::Uint64Bits),
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-coercive-type-source",
        },
        coordinates: &[
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "coercive-type",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "source",
            },
        ],
        evidence_classes: &[EvidenceClass::Observed],
        aggregation_rule: AggregationRule::None,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_coercive_infrastructure",
        },
        reference_artifact: Some(TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_coercive_infrastructure",
        }),
        reference_digest: Some("33e6558d2b438e7aea672021f0e15f743f1ea331ab82407c0805a428b29cf808"),
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "ecology/h3-land-fraction",
        },
        representation: MetricRepresentation::Facet,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "fraction",
        },
        value_kind: Some(ValueKind::Float64Bits),
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "h3-r7-vintage",
        },
        coordinates: &[TypedIdentityLiteral {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "h3-cell",
        }],
        evidence_classes: &[EvidenceClass::Derived],
        aggregation_rule: AggregationRule::EqualAreaWaterIntersection,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "h3_res7_land_mask",
        },
        reference_artifact: Some(TypedIdentityLiteral {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "h3_res7_land_mask",
        }),
        reference_digest: Some("4e6caba297f0111a9ec93d948a83543bb9f7179361fe5dd318bb8a98a5be5194"),
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/presence",
        },
        representation: MetricRepresentation::Dyad,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "typed-relation",
        },
        value_kind: None,
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "actor-node-verified-tick",
        },
        coordinates: &[
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "actor",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "node",
            },
        ],
        evidence_classes: &[EvidenceClass::Derived],
        aggregation_rule: AggregationRule::TypedRelationProjection,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.engine",
            local_id: "typed-graph-relations-at-verified-tick",
        },
        reference_artifact: None,
        reference_digest: None,
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/solidarity",
        },
        representation: MetricRepresentation::Dyad,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "typed-relation",
        },
        value_kind: None,
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "actor-node-verified-tick",
        },
        coordinates: &[
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "actor",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "node",
            },
        ],
        evidence_classes: &[EvidenceClass::Derived],
        aggregation_rule: AggregationRule::TypedRelationProjection,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.engine",
            local_id: "typed-graph-relations-at-verified-tick",
        },
        reference_artifact: None,
        reference_digest: None,
    },
    RtdMetricRegistryRow {
        metric: TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/membership",
        },
        representation: MetricRepresentation::Dyad,
        unit: TypedIdentityLiteral {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "typed-relation",
        },
        value_kind: None,
        native_scale: TypedIdentityLiteral {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "actor-node-verified-tick",
        },
        coordinates: &[
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "actor",
            },
            TypedIdentityLiteral {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "node",
            },
        ],
        evidence_classes: &[EvidenceClass::Derived],
        aggregation_rule: AggregationRule::TypedRelationProjection,
        producer: TypedIdentityLiteral {
            domain: "producer",
            authority: "babylon.engine",
            local_id: "typed-graph-relations-at-verified-tick",
        },
        reference_artifact: None,
        reference_digest: None,
    },
];

pub const RTD_RELATION_BINDING_REGISTRY: &[RtdRelationBindingRegistryRow] = &[
    RtdRelationBindingRegistryRow {
        record_family: "REFERENCE_FLOW",
        kind: "COMMUTER_JOBS",
        metric: Some(TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "circulation/lodes-county-commuter-total-jobs",
        }),
        payload_mode: RelationPayloadMode::SingleMetricFacet,
    },
    RtdRelationBindingRegistryRow {
        record_family: "REFERENCE_FLOW",
        kind: "BORDER_SYNTHESIS",
        metric: None,
        payload_mode: RelationPayloadMode::Empty,
    },
    RtdRelationBindingRegistryRow {
        record_family: "DYAD",
        kind: "PRESENCE",
        metric: Some(TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/presence",
        }),
        payload_mode: RelationPayloadMode::ImplicitRelation,
    },
    RtdRelationBindingRegistryRow {
        record_family: "DYAD",
        kind: "MEMBERSHIP",
        metric: Some(TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/membership",
        }),
        payload_mode: RelationPayloadMode::ImplicitRelation,
    },
    RtdRelationBindingRegistryRow {
        record_family: "DYAD",
        kind: "SOLIDARITY",
        metric: Some(TypedIdentityLiteral {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/solidarity",
        }),
        payload_mode: RelationPayloadMode::ImplicitRelation,
    },
    RtdRelationBindingRegistryRow {
        record_family: "DYAD",
        kind: "COMMAND",
        metric: None,
        payload_mode: RelationPayloadMode::Empty,
    },
];
