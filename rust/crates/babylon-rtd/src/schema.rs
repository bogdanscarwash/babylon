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
pub enum AudienceV1 {
    #[serde(rename = "ADMIN_MATERIAL")]
    AdminMaterial,
    #[serde(rename = "PLAYER_KNOWLEDGE")]
    PlayerKnowledge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum DurabilityV1 {
    #[serde(rename = "IN_MEMORY")]
    InMemory,
    #[serde(rename = "COMMITTED")]
    Committed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum EvidenceClassV1 {
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
pub enum StatusV1 {
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
pub enum ValueKindV1 {
    #[serde(rename = "UINT64_BITS")]
    Uint64Bits,
    #[serde(rename = "FLOAT64_BITS")]
    Float64Bits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum CoverageV1 {
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
pub enum MembershipKindV1 {
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
pub enum FacetFamilyV1 {
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
pub enum DyadKindV1 {
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
pub enum HyperedgeKindV1 {
    #[serde(rename = "PUBLIC_RELATION")]
    PublicRelation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum FlowKindV1 {
    #[serde(rename = "COMMUTER_JOBS")]
    CommuterJobs,
    #[serde(rename = "BORDER_SYNTHESIS")]
    BorderSynthesis,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum RelationPayloadModeV1 {
    #[serde(rename = "EMPTY")]
    Empty,
    #[serde(rename = "SINGLE_METRIC_FACET")]
    SingleMetricFacet,
    #[serde(rename = "IMPLICIT_RELATION")]
    ImplicitRelation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum GapReasonV1 {
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
pub enum MetricRepresentationV1 {
    #[serde(rename = "FACET")]
    Facet,
    #[serde(rename = "REFERENCE_FLOW")]
    ReferenceFlow,
    #[serde(rename = "DYAD")]
    Dyad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum AggregationRuleV1 {
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
pub enum RtdCollectionKindV1 {
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
pub struct TypedIdentityV1 {
    pub domain: String,
    pub authority: String,
    pub local_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceDigestV1 {
    pub reference_id: TypedIdentityV1,
    pub sha256_hex: String,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub artifact_schema_id_or_null: Option<TypedIdentityV1>,
    pub vintage: String,
    pub evidence_class: EvidenceClassV1,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DimensionCoordinateV1 {
    pub dimension_ref: TypedIdentityV1,
    pub member_ref: TypedIdentityV1,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaleMembershipV1 {
    pub membership_id: TypedIdentityV1,
    pub member_ref: TypedIdentityV1,
    pub scale_ref: TypedIdentityV1,
    pub membership_kind: MembershipKindV1,
    pub status: StatusV1,
    pub weight_status: StatusV1,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub weight_bits_or_null: Option<String>,
    pub coverage: CoverageV1,
    pub evidence_class: EvidenceClassV1,
    pub provenance_refs: Vec<TypedIdentityV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacetV1 {
    pub facet_id: TypedIdentityV1,
    pub family: FacetFamilyV1,
    pub subject_ref: TypedIdentityV1,
    pub metric_id: TypedIdentityV1,
    pub unit_id: TypedIdentityV1,
    pub native_scale: TypedIdentityV1,
    pub coordinates: Vec<DimensionCoordinateV1>,
    pub vintage: String,
    pub status: StatusV1,
    pub value_kind: ValueKindV1,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub value_bits_or_null: Option<String>,
    pub coverage: CoverageV1,
    pub evidence_class: EvidenceClassV1,
    pub provenance_refs: Vec<TypedIdentityV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DyadV1 {
    pub relation_id: TypedIdentityV1,
    pub relation_kind: DyadKindV1,
    pub from_ref: TypedIdentityV1,
    pub to_ref: TypedIdentityV1,
    pub native_scale: TypedIdentityV1,
    pub status: StatusV1,
    pub coverage: CoverageV1,
    pub payload_facets: Vec<TypedIdentityV1>,
    pub evidence_class: EvidenceClassV1,
    pub provenance_refs: Vec<TypedIdentityV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HyperedgeV1 {
    pub hyperedge_id: TypedIdentityV1,
    pub hyperedge_kind: HyperedgeKindV1,
    pub member_refs: Vec<TypedIdentityV1>,
    pub native_scale: TypedIdentityV1,
    pub status: StatusV1,
    pub coverage: CoverageV1,
    pub payload_facets: Vec<TypedIdentityV1>,
    pub evidence_class: EvidenceClassV1,
    pub provenance_refs: Vec<TypedIdentityV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceFlowV1 {
    pub flow_id: TypedIdentityV1,
    pub flow_kind: FlowKindV1,
    pub origin_ref: TypedIdentityV1,
    pub destination_ref: TypedIdentityV1,
    pub payload_facets: Vec<TypedIdentityV1>,
    pub native_scale: TypedIdentityV1,
    pub status: StatusV1,
    pub coverage: CoverageV1,
    pub evidence_class: EvidenceClassV1,
    pub provenance_refs: Vec<TypedIdentityV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GapV1 {
    pub gap_id: TypedIdentityV1,
    pub requested_metric_or_relation: TypedIdentityV1,
    pub status: StatusV1,
    pub reason_code: GapReasonV1,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub required_producer_or_null: Option<String>,
    pub provenance_refs: Vec<TypedIdentityV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceV1 {
    pub provenance_id: TypedIdentityV1,
    pub artifact_digest: String,
    pub locator: String,
    pub vintage: String,
    pub evidence_class: EvidenceClassV1,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub transformation_digest_or_null: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionSurfaceV1 {
    pub question_id: TypedIdentityV1,
    pub signal_refs: Vec<TypedIdentityV1>,
    pub action_refs: Vec<TypedIdentityV1>,
    pub receipt_refs: Vec<TypedIdentityV1>,
    pub archive_subject_refs: Vec<TypedIdentityV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RtdDossierDraftV1 {
    pub schema: String,
    pub schema_version: u16,
    pub projection_version: u16,
    pub audience: AudienceV1,
    pub durability: DurabilityV1,
    pub verified_tick: u64,
    pub graph_state_hash: String,
    pub nominal_world_hash: String,
    pub reference_digests: Vec<ReferenceDigestV1>,
    pub definitions_digest: String,
    pub template_digest: String,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub fog_policy_digest: Option<String>,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub knowledge_context_digest: Option<String>,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub actor: Option<TypedIdentityV1>,
    pub focus: Vec<TypedIdentityV1>,
    pub scale_memberships: Vec<ScaleMembershipV1>,
    pub facets: Vec<FacetV1>,
    pub dyads: Vec<DyadV1>,
    pub hyperedges: Vec<HyperedgeV1>,
    pub flows: Vec<ReferenceFlowV1>,
    pub gaps: Vec<GapV1>,
    pub provenance: Vec<ProvenanceV1>,
    pub decision_surface: DecisionSurfaceV1,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalTerritoryDossierV1 {
    pub schema: String,
    pub schema_version: u16,
    pub projection_version: u16,
    pub audience: AudienceV1,
    pub durability: DurabilityV1,
    pub verified_tick: u64,
    pub graph_state_hash: String,
    pub nominal_world_hash: String,
    pub reference_digests: Vec<ReferenceDigestV1>,
    pub definitions_digest: String,
    pub template_digest: String,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub fog_policy_digest: Option<String>,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub knowledge_context_digest: Option<String>,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub actor: Option<TypedIdentityV1>,
    pub focus: Vec<TypedIdentityV1>,
    pub scale_memberships: Vec<ScaleMembershipV1>,
    pub facets: Vec<FacetV1>,
    pub dyads: Vec<DyadV1>,
    pub hyperedges: Vec<HyperedgeV1>,
    pub flows: Vec<ReferenceFlowV1>,
    pub gaps: Vec<GapV1>,
    pub provenance: Vec<ProvenanceV1>,
    pub decision_surface: DecisionSurfaceV1,
    pub projection_hash: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedIdentityLiteralV1 {
    pub domain: &'static str,
    pub authority: &'static str,
    pub local_id: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RtdIdentityRegistryRowV1 {
    pub category: &'static str,
    pub symbolic_name: &'static str,
    pub identity: TypedIdentityLiteralV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RtdMetricRegistryRowV1 {
    pub metric: TypedIdentityLiteralV1,
    pub representation: MetricRepresentationV1,
    pub unit: TypedIdentityLiteralV1,
    pub value_kind: Option<ValueKindV1>,
    pub native_scale: TypedIdentityLiteralV1,
    pub coordinates: &'static [TypedIdentityLiteralV1],
    pub evidence_classes: &'static [EvidenceClassV1],
    pub aggregation_rule: AggregationRuleV1,
    pub producer: TypedIdentityLiteralV1,
    pub reference_artifact: Option<TypedIdentityLiteralV1>,
    pub reference_digest: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RtdRelationBindingRegistryRowV1 {
    pub record_family: &'static str,
    pub kind: &'static str,
    pub metric: Option<TypedIdentityLiteralV1>,
    pub payload_mode: RelationPayloadModeV1,
}

pub const RTD_V1_SCHEMA_ID: &str = "babylon.relational-territory-dossier";
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

pub const RTD_V1_LIMITS: &[(&str, u64)] = &[
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

pub const RTD_V1_ERROR_REGISTRY: &[&str] = &[
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

pub const RTD_V1_IDENTITY_REGISTRY: &[RtdIdentityRegistryRowV1] = &[
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "production/qcew-leaf-employment",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-employment",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "production/qcew-leaf-establishments",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-establishments",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "production/qcew-leaf-total-wages-usd",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-total-wages-usd",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "production/qcew-leaf-average-annual-pay-usd",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-average-annual-pay-usd",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "production/qcew-county-employment",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-county-employment",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "production/qcew-county-establishments",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-county-establishments",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "production/qcew-county-total-wages-usd",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-county-total-wages-usd",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "circulation/lodes-county-commuter-total-jobs",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "circulation/lodes-county-commuter-total-jobs",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "reproduction/census-housing-households",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/census-housing-households",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "reproduction/census-median-rent-usd",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/census-median-rent-usd",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "reproduction/census-rent-burden-households",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/census-rent-burden-households",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "reproduction/h3-population-persons",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/h3-population-persons",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "production/h3-workplace-jobs",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/h3-workplace-jobs",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "carceral/facility-count",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "carceral/facility-count",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "ecology/h3-land-fraction",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "ecology/h3-land-fraction",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "rootedness/presence",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/presence",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "rootedness/solidarity",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/solidarity",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "metrics",
        symbolic_name: "rootedness/membership",
        identity: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/membership",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "units",
        symbolic_name: "JOBS",
        identity: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "jobs",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "units",
        symbolic_name: "ESTABLISHMENTS",
        identity: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "establishments",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "units",
        symbolic_name: "USD_CURRENT",
        identity: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "usd-current",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "units",
        symbolic_name: "HOUSEHOLDS",
        identity: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "households",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "units",
        symbolic_name: "PERSONS",
        identity: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "persons",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "units",
        symbolic_name: "FACILITIES",
        identity: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "facilities",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "units",
        symbolic_name: "FRACTION",
        identity: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "fraction",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "units",
        symbolic_name: "TYPED_RELATION",
        identity: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "typed-relation",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "coordinates",
        symbolic_name: "county",
        identity: TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "county",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "coordinates",
        symbolic_name: "naics6",
        identity: TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "naics6",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "coordinates",
        symbolic_name: "ownership",
        identity: TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "ownership",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "coordinates",
        symbolic_name: "home_county",
        identity: TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "home-county",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "coordinates",
        symbolic_name: "work_county",
        identity: TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "work-county",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "coordinates",
        symbolic_name: "source",
        identity: TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "source",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "coordinates",
        symbolic_name: "tenure",
        identity: TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "tenure",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "coordinates",
        symbolic_name: "race",
        identity: TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "race",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "coordinates",
        symbolic_name: "burden",
        identity: TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "burden",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "coordinates",
        symbolic_name: "h3_cell",
        identity: TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "h3-cell",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "coordinates",
        symbolic_name: "coercive_type",
        identity: TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "coercive-type",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "coordinates",
        symbolic_name: "actor",
        identity: TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "actor",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "coordinates",
        symbolic_name: "node",
        identity: TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "node",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "native_scales",
        symbolic_name: "COUNTY_NAICS6_OWNERSHIP_YEAR",
        identity: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-naics6-ownership-year",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "native_scales",
        symbolic_name: "COUNTY_OWNERSHIP_YEAR",
        identity: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-ownership-year",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "native_scales",
        symbolic_name: "HOME_COUNTY_WORK_COUNTY_YEAR",
        identity: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "home-county-work-county-year",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "native_scales",
        symbolic_name: "COUNTY_SOURCE_TENURE_TIME_RACE",
        identity: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-source-tenure-time-race",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "native_scales",
        symbolic_name: "COUNTY_SOURCE_TIME_RACE",
        identity: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-source-time-race",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "native_scales",
        symbolic_name: "COUNTY_SOURCE_BURDEN_TIME_RACE",
        identity: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-source-burden-time-race",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "native_scales",
        symbolic_name: "H3_R7_VINTAGE",
        identity: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "h3-r7-vintage",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "native_scales",
        symbolic_name: "COUNTY_COERCIVE_TYPE_SOURCE",
        identity: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-coercive-type-source",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "native_scales",
        symbolic_name: "ACTOR_NODE_VERIFIED_TICK",
        identity: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "actor-node-verified-tick",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "producers",
        symbolic_name: "fact_qcew_annual",
        identity: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "producers",
        symbolic_name: "fact_qcew_county_rollup",
        identity: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "producers",
        symbolic_name: "fact_lodes_commuter_flow",
        identity: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_lodes_commuter_flow",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "producers",
        symbolic_name: "fact_census_housing",
        identity: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_census_housing",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "producers",
        symbolic_name: "fact_census_rent",
        identity: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "producers",
        symbolic_name: "fact_census_rent_burden",
        identity: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent_burden",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "producers",
        symbolic_name: "h3_res7_population",
        identity: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "h3_res7_population",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "producers",
        symbolic_name: "h3_res7_workplace",
        identity: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "h3_res7_workplace",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "producers",
        symbolic_name: "fact_coercive_infrastructure",
        identity: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_coercive_infrastructure",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "producers",
        symbolic_name: "h3_res7_land_mask",
        identity: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "h3_res7_land_mask",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "producers",
        symbolic_name: "committed typed graph",
        identity: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.engine",
            local_id: "typed-graph-relations-at-verified-tick",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "references",
        symbolic_name: "fact_qcew_annual",
        identity: TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "references",
        symbolic_name: "fact_qcew_county_rollup",
        identity: TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "references",
        symbolic_name: "fact_lodes_commuter_flow",
        identity: TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_lodes_commuter_flow",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "references",
        symbolic_name: "fact_census_housing",
        identity: TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_census_housing",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "references",
        symbolic_name: "fact_census_rent",
        identity: TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "references",
        symbolic_name: "fact_census_rent_burden",
        identity: TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent_burden",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "references",
        symbolic_name: "h3_res7_population",
        identity: TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "h3_res7_population",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "references",
        symbolic_name: "h3_res7_workplace",
        identity: TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "h3_res7_workplace",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "references",
        symbolic_name: "fact_coercive_infrastructure",
        identity: TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_coercive_infrastructure",
        },
    },
    RtdIdentityRegistryRowV1 {
        category: "references",
        symbolic_name: "h3_res7_land_mask",
        identity: TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "h3_res7_land_mask",
        },
    },
];

pub const RTD_V1_METRIC_REGISTRY: &[RtdMetricRegistryRowV1] = &[
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-employment",
        },
        representation: MetricRepresentationV1::Facet,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "jobs",
        },
        value_kind: Some(ValueKindV1::Uint64Bits),
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-naics6-ownership-year",
        },
        coordinates: &[
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "naics6",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "ownership",
            },
        ],
        evidence_classes: &[EvidenceClassV1::Observed, EvidenceClassV1::Derived],
        aggregation_rule: AggregationRuleV1::None,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        },
        reference_artifact: Some(TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        }),
        reference_digest: Some("ca3825a3d60831479313632073b7fc9a941d57dcf9b8940181c4713b6d442248"),
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-establishments",
        },
        representation: MetricRepresentationV1::Facet,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "establishments",
        },
        value_kind: Some(ValueKindV1::Uint64Bits),
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-naics6-ownership-year",
        },
        coordinates: &[
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "naics6",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "ownership",
            },
        ],
        evidence_classes: &[EvidenceClassV1::Observed, EvidenceClassV1::Derived],
        aggregation_rule: AggregationRuleV1::None,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        },
        reference_artifact: Some(TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        }),
        reference_digest: Some("ca3825a3d60831479313632073b7fc9a941d57dcf9b8940181c4713b6d442248"),
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-total-wages-usd",
        },
        representation: MetricRepresentationV1::Facet,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "usd-current",
        },
        value_kind: Some(ValueKindV1::Float64Bits),
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-naics6-ownership-year",
        },
        coordinates: &[
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "naics6",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "ownership",
            },
        ],
        evidence_classes: &[EvidenceClassV1::Observed, EvidenceClassV1::Derived],
        aggregation_rule: AggregationRuleV1::None,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        },
        reference_artifact: Some(TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        }),
        reference_digest: Some("ca3825a3d60831479313632073b7fc9a941d57dcf9b8940181c4713b6d442248"),
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-leaf-average-annual-pay-usd",
        },
        representation: MetricRepresentationV1::Facet,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "usd-current",
        },
        value_kind: Some(ValueKindV1::Float64Bits),
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-naics6-ownership-year",
        },
        coordinates: &[
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "naics6",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "ownership",
            },
        ],
        evidence_classes: &[EvidenceClassV1::Observed, EvidenceClassV1::Derived],
        aggregation_rule: AggregationRuleV1::None,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        },
        reference_artifact: Some(TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_annual",
        }),
        reference_digest: Some("ca3825a3d60831479313632073b7fc9a941d57dcf9b8940181c4713b6d442248"),
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-county-employment",
        },
        representation: MetricRepresentationV1::Facet,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "jobs",
        },
        value_kind: Some(ValueKindV1::Uint64Bits),
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-ownership-year",
        },
        coordinates: &[
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "ownership",
            },
        ],
        evidence_classes: &[EvidenceClassV1::Observed, EvidenceClassV1::Derived],
        aggregation_rule: AggregationRuleV1::PublishedRollup,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        },
        reference_artifact: Some(TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        }),
        reference_digest: Some("34c2bbb935f79b3c8076a97092b004b14cca120e8272b93c35b3ac9dc2721d13"),
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-county-establishments",
        },
        representation: MetricRepresentationV1::Facet,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "establishments",
        },
        value_kind: Some(ValueKindV1::Uint64Bits),
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-ownership-year",
        },
        coordinates: &[
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "ownership",
            },
        ],
        evidence_classes: &[EvidenceClassV1::Observed, EvidenceClassV1::Derived],
        aggregation_rule: AggregationRuleV1::PublishedRollup,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        },
        reference_artifact: Some(TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        }),
        reference_digest: Some("34c2bbb935f79b3c8076a97092b004b14cca120e8272b93c35b3ac9dc2721d13"),
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/qcew-county-total-wages-usd",
        },
        representation: MetricRepresentationV1::Facet,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "usd-current",
        },
        value_kind: Some(ValueKindV1::Float64Bits),
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-ownership-year",
        },
        coordinates: &[
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "ownership",
            },
        ],
        evidence_classes: &[EvidenceClassV1::Observed, EvidenceClassV1::Derived],
        aggregation_rule: AggregationRuleV1::PublishedRollup,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        },
        reference_artifact: Some(TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_qcew_county_rollup",
        }),
        reference_digest: Some("34c2bbb935f79b3c8076a97092b004b14cca120e8272b93c35b3ac9dc2721d13"),
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "circulation/lodes-county-commuter-total-jobs",
        },
        representation: MetricRepresentationV1::ReferenceFlow,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "jobs",
        },
        value_kind: Some(ValueKindV1::Uint64Bits),
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "home-county-work-county-year",
        },
        coordinates: &[
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "home-county",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "work-county",
            },
        ],
        evidence_classes: &[EvidenceClassV1::Derived],
        aggregation_rule: AggregationRuleV1::LoadTimeSum,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_lodes_commuter_flow",
        },
        reference_artifact: Some(TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_lodes_commuter_flow",
        }),
        reference_digest: Some("d3745f8def09cd8c7a38e1870e6ec2c1853e210b777d8e8358cfce36665bd64d"),
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/census-housing-households",
        },
        representation: MetricRepresentationV1::Facet,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "households",
        },
        value_kind: Some(ValueKindV1::Uint64Bits),
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-source-tenure-time-race",
        },
        coordinates: &[
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "source",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "tenure",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "race",
            },
        ],
        evidence_classes: &[EvidenceClassV1::Observed],
        aggregation_rule: AggregationRuleV1::None,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_census_housing",
        },
        reference_artifact: Some(TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_census_housing",
        }),
        reference_digest: Some("09ff2d9666b3f5ef267b65cbc77c14e99384f0157b6a4c898ac37df2e67ca59f"),
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/census-median-rent-usd",
        },
        representation: MetricRepresentationV1::Facet,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "usd-current",
        },
        value_kind: Some(ValueKindV1::Float64Bits),
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-source-time-race",
        },
        coordinates: &[
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "source",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "race",
            },
        ],
        evidence_classes: &[EvidenceClassV1::Observed],
        aggregation_rule: AggregationRuleV1::None,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent",
        },
        reference_artifact: Some(TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent",
        }),
        reference_digest: Some("4c8cc134ec490ca75961d83485fc97c6bf240b32128e9d0517e00e62d578a99e"),
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/census-rent-burden-households",
        },
        representation: MetricRepresentationV1::Facet,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "households",
        },
        value_kind: Some(ValueKindV1::Uint64Bits),
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-source-burden-time-race",
        },
        coordinates: &[
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "source",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "burden",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "race",
            },
        ],
        evidence_classes: &[EvidenceClassV1::Observed],
        aggregation_rule: AggregationRuleV1::None,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent_burden",
        },
        reference_artifact: Some(TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_census_rent_burden",
        }),
        reference_digest: Some("8a42a51c17bf3ebee09f0b0b5145d5c8253c7e3446eec8c75714f9951b20df12"),
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "reproduction/h3-population-persons",
        },
        representation: MetricRepresentationV1::Facet,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "persons",
        },
        value_kind: Some(ValueKindV1::Uint64Bits),
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "h3-r7-vintage",
        },
        coordinates: &[TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "h3-cell",
        }],
        evidence_classes: &[EvidenceClassV1::Derived],
        aggregation_rule: AggregationRuleV1::BlockInternalPointAssignment,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "h3_res7_population",
        },
        reference_artifact: Some(TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "h3_res7_population",
        }),
        reference_digest: Some("b096a5891284f0ca55bedae9d1a9092eb8ea9e9e32d32b6ace430a9833b53afc"),
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "production/h3-workplace-jobs",
        },
        representation: MetricRepresentationV1::Facet,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "jobs",
        },
        value_kind: Some(ValueKindV1::Uint64Bits),
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "h3-r7-vintage",
        },
        coordinates: &[TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "h3-cell",
        }],
        evidence_classes: &[EvidenceClassV1::Derived],
        aggregation_rule: AggregationRuleV1::BlockCoordinateAssignment,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "h3_res7_workplace",
        },
        reference_artifact: Some(TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "h3_res7_workplace",
        }),
        reference_digest: Some("ea2ce1508f4fe51f1e879b9f4a1daf579c4b00349388b12a85f884a8f49eabb6"),
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "carceral/facility-count",
        },
        representation: MetricRepresentationV1::Facet,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "facilities",
        },
        value_kind: Some(ValueKindV1::Uint64Bits),
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "county-coercive-type-source",
        },
        coordinates: &[
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "county",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "coercive-type",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "source",
            },
        ],
        evidence_classes: &[EvidenceClassV1::Observed],
        aggregation_rule: AggregationRuleV1::None,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "fact_coercive_infrastructure",
        },
        reference_artifact: Some(TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "fact_coercive_infrastructure",
        }),
        reference_digest: Some("33e6558d2b438e7aea672021f0e15f743f1ea331ab82407c0805a428b29cf808"),
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "ecology/h3-land-fraction",
        },
        representation: MetricRepresentationV1::Facet,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "fraction",
        },
        value_kind: Some(ValueKindV1::Float64Bits),
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "h3-r7-vintage",
        },
        coordinates: &[TypedIdentityLiteralV1 {
            domain: "dimension",
            authority: "babylon.rtd.v1",
            local_id: "h3-cell",
        }],
        evidence_classes: &[EvidenceClassV1::Derived],
        aggregation_rule: AggregationRuleV1::EqualAreaWaterIntersection,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.data.v7",
            local_id: "h3_res7_land_mask",
        },
        reference_artifact: Some(TypedIdentityLiteralV1 {
            domain: "reference-artifact",
            authority: "babylon.data.v7",
            local_id: "h3_res7_land_mask",
        }),
        reference_digest: Some("4e6caba297f0111a9ec93d948a83543bb9f7179361fe5dd318bb8a98a5be5194"),
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/presence",
        },
        representation: MetricRepresentationV1::Dyad,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "typed-relation",
        },
        value_kind: None,
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "actor-node-verified-tick",
        },
        coordinates: &[
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "actor",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "node",
            },
        ],
        evidence_classes: &[EvidenceClassV1::Derived],
        aggregation_rule: AggregationRuleV1::TypedRelationProjection,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.engine",
            local_id: "typed-graph-relations-at-verified-tick",
        },
        reference_artifact: None,
        reference_digest: None,
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/solidarity",
        },
        representation: MetricRepresentationV1::Dyad,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "typed-relation",
        },
        value_kind: None,
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "actor-node-verified-tick",
        },
        coordinates: &[
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "actor",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "node",
            },
        ],
        evidence_classes: &[EvidenceClassV1::Derived],
        aggregation_rule: AggregationRuleV1::TypedRelationProjection,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.engine",
            local_id: "typed-graph-relations-at-verified-tick",
        },
        reference_artifact: None,
        reference_digest: None,
    },
    RtdMetricRegistryRowV1 {
        metric: TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/membership",
        },
        representation: MetricRepresentationV1::Dyad,
        unit: TypedIdentityLiteralV1 {
            domain: "unit",
            authority: "babylon.rtd.v1",
            local_id: "typed-relation",
        },
        value_kind: None,
        native_scale: TypedIdentityLiteralV1 {
            domain: "native-scale",
            authority: "babylon.rtd.v1",
            local_id: "actor-node-verified-tick",
        },
        coordinates: &[
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "actor",
            },
            TypedIdentityLiteralV1 {
                domain: "dimension",
                authority: "babylon.rtd.v1",
                local_id: "node",
            },
        ],
        evidence_classes: &[EvidenceClassV1::Derived],
        aggregation_rule: AggregationRuleV1::TypedRelationProjection,
        producer: TypedIdentityLiteralV1 {
            domain: "producer",
            authority: "babylon.engine",
            local_id: "typed-graph-relations-at-verified-tick",
        },
        reference_artifact: None,
        reference_digest: None,
    },
];

pub const RTD_V1_RELATION_BINDING_REGISTRY: &[RtdRelationBindingRegistryRowV1] = &[
    RtdRelationBindingRegistryRowV1 {
        record_family: "REFERENCE_FLOW",
        kind: "COMMUTER_JOBS",
        metric: Some(TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "circulation/lodes-county-commuter-total-jobs",
        }),
        payload_mode: RelationPayloadModeV1::SingleMetricFacet,
    },
    RtdRelationBindingRegistryRowV1 {
        record_family: "REFERENCE_FLOW",
        kind: "BORDER_SYNTHESIS",
        metric: None,
        payload_mode: RelationPayloadModeV1::Empty,
    },
    RtdRelationBindingRegistryRowV1 {
        record_family: "DYAD",
        kind: "PRESENCE",
        metric: Some(TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/presence",
        }),
        payload_mode: RelationPayloadModeV1::ImplicitRelation,
    },
    RtdRelationBindingRegistryRowV1 {
        record_family: "DYAD",
        kind: "MEMBERSHIP",
        metric: Some(TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/membership",
        }),
        payload_mode: RelationPayloadModeV1::ImplicitRelation,
    },
    RtdRelationBindingRegistryRowV1 {
        record_family: "DYAD",
        kind: "SOLIDARITY",
        metric: Some(TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: "rootedness/solidarity",
        }),
        payload_mode: RelationPayloadModeV1::ImplicitRelation,
    },
    RtdRelationBindingRegistryRowV1 {
        record_family: "DYAD",
        kind: "COMMAND",
        metric: None,
        payload_mode: RelationPayloadModeV1::Empty,
    },
];
