use babylon_rtd::*;
use serde::de::DeserializeOwned;
use std::collections::BTreeSet;

const RTD_SCHEMA: &[u8] =
    include_bytes!("../../../../contracts/relational_territory_dossier_v1.yaml");

const EXPECTED_ERRORS: [&str; 20] = [
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

const EXPECTED_METRICS: [&str; 18] = [
    "production/qcew-leaf-employment",
    "production/qcew-leaf-establishments",
    "production/qcew-leaf-total-wages-usd",
    "production/qcew-leaf-average-annual-pay-usd",
    "production/qcew-county-employment",
    "production/qcew-county-establishments",
    "production/qcew-county-total-wages-usd",
    "circulation/lodes-county-commuter-total-jobs",
    "reproduction/census-housing-households",
    "reproduction/census-median-rent-usd",
    "reproduction/census-rent-burden-households",
    "reproduction/h3-population-persons",
    "production/h3-workplace-jobs",
    "carceral/facility-count",
    "ecology/h3-land-fraction",
    "rootedness/presence",
    "rootedness/solidarity",
    "rootedness/membership",
];

const EXPECTED_UNITS: [&str; 18] = [
    "jobs",
    "establishments",
    "usd-current",
    "usd-current",
    "jobs",
    "establishments",
    "usd-current",
    "jobs",
    "households",
    "usd-current",
    "households",
    "persons",
    "jobs",
    "facilities",
    "fraction",
    "typed-relation",
    "typed-relation",
    "typed-relation",
];

const EXPECTED_SCALES: [&str; 18] = [
    "county-naics6-ownership-year",
    "county-naics6-ownership-year",
    "county-naics6-ownership-year",
    "county-naics6-ownership-year",
    "county-ownership-year",
    "county-ownership-year",
    "county-ownership-year",
    "home-county-work-county-year",
    "county-source-tenure-time-race",
    "county-source-time-race",
    "county-source-burden-time-race",
    "h3-r7-vintage",
    "h3-r7-vintage",
    "county-coercive-type-source",
    "h3-r7-vintage",
    "actor-node-verified-tick",
    "actor-node-verified-tick",
    "actor-node-verified-tick",
];

const EXPECTED_COORDINATES: [&str; 18] = [
    "county,naics6,ownership",
    "county,naics6,ownership",
    "county,naics6,ownership",
    "county,naics6,ownership",
    "county,ownership",
    "county,ownership",
    "county,ownership",
    "home-county,work-county",
    "county,source,tenure,race",
    "county,source,race",
    "county,source,burden,race",
    "h3-cell",
    "h3-cell",
    "county,coercive-type,source",
    "h3-cell",
    "actor,node",
    "actor,node",
    "actor,node",
];

const EXPECTED_PRODUCERS: [&str; 18] = [
    "fact_qcew_annual",
    "fact_qcew_annual",
    "fact_qcew_annual",
    "fact_qcew_annual",
    "fact_qcew_county_rollup",
    "fact_qcew_county_rollup",
    "fact_qcew_county_rollup",
    "fact_lodes_commuter_flow",
    "fact_census_housing",
    "fact_census_rent",
    "fact_census_rent_burden",
    "h3_res7_population",
    "h3_res7_workplace",
    "fact_coercive_infrastructure",
    "h3_res7_land_mask",
    "typed-graph-relations-at-verified-tick",
    "typed-graph-relations-at-verified-tick",
    "typed-graph-relations-at-verified-tick",
];

const EXPECTED_DIGESTS: [Option<&str>; 18] = [
    Some("ca3825a3d60831479313632073b7fc9a941d57dcf9b8940181c4713b6d442248"),
    Some("ca3825a3d60831479313632073b7fc9a941d57dcf9b8940181c4713b6d442248"),
    Some("ca3825a3d60831479313632073b7fc9a941d57dcf9b8940181c4713b6d442248"),
    Some("ca3825a3d60831479313632073b7fc9a941d57dcf9b8940181c4713b6d442248"),
    Some("34c2bbb935f79b3c8076a97092b004b14cca120e8272b93c35b3ac9dc2721d13"),
    Some("34c2bbb935f79b3c8076a97092b004b14cca120e8272b93c35b3ac9dc2721d13"),
    Some("34c2bbb935f79b3c8076a97092b004b14cca120e8272b93c35b3ac9dc2721d13"),
    Some("d3745f8def09cd8c7a38e1870e6ec2c1853e210b777d8e8358cfce36665bd64d"),
    Some("09ff2d9666b3f5ef267b65cbc77c14e99384f0157b6a4c898ac37df2e67ca59f"),
    Some("4c8cc134ec490ca75961d83485fc97c6bf240b32128e9d0517e00e62d578a99e"),
    Some("8a42a51c17bf3ebee09f0b0b5145d5c8253c7e3446eec8c75714f9951b20df12"),
    Some("b096a5891284f0ca55bedae9d1a9092eb8ea9e9e32d32b6ace430a9833b53afc"),
    Some("ea2ce1508f4fe51f1e879b9f4a1daf579c4b00349388b12a85f884a8f49eabb6"),
    Some("33e6558d2b438e7aea672021f0e15f743f1ea331ab82407c0805a428b29cf808"),
    Some("4e6caba297f0111a9ec93d948a83543bb9f7179361fe5dd318bb8a98a5be5194"),
    None,
    None,
    None,
];

const EXPECTED_REPRESENTATIONS: [MetricRepresentationV1; 18] = [
    MetricRepresentationV1::Facet,
    MetricRepresentationV1::Facet,
    MetricRepresentationV1::Facet,
    MetricRepresentationV1::Facet,
    MetricRepresentationV1::Facet,
    MetricRepresentationV1::Facet,
    MetricRepresentationV1::Facet,
    MetricRepresentationV1::ReferenceFlow,
    MetricRepresentationV1::Facet,
    MetricRepresentationV1::Facet,
    MetricRepresentationV1::Facet,
    MetricRepresentationV1::Facet,
    MetricRepresentationV1::Facet,
    MetricRepresentationV1::Facet,
    MetricRepresentationV1::Facet,
    MetricRepresentationV1::Dyad,
    MetricRepresentationV1::Dyad,
    MetricRepresentationV1::Dyad,
];

const EXPECTED_VALUE_KINDS: [Option<ValueKindV1>; 18] = [
    Some(ValueKindV1::Uint64Bits),
    Some(ValueKindV1::Uint64Bits),
    Some(ValueKindV1::Float64Bits),
    Some(ValueKindV1::Float64Bits),
    Some(ValueKindV1::Uint64Bits),
    Some(ValueKindV1::Uint64Bits),
    Some(ValueKindV1::Float64Bits),
    Some(ValueKindV1::Uint64Bits),
    Some(ValueKindV1::Uint64Bits),
    Some(ValueKindV1::Float64Bits),
    Some(ValueKindV1::Uint64Bits),
    Some(ValueKindV1::Uint64Bits),
    Some(ValueKindV1::Uint64Bits),
    Some(ValueKindV1::Uint64Bits),
    Some(ValueKindV1::Float64Bits),
    None,
    None,
    None,
];

const EXPECTED_AGGREGATIONS: [AggregationRuleV1; 18] = [
    AggregationRuleV1::None,
    AggregationRuleV1::None,
    AggregationRuleV1::None,
    AggregationRuleV1::None,
    AggregationRuleV1::PublishedRollup,
    AggregationRuleV1::PublishedRollup,
    AggregationRuleV1::PublishedRollup,
    AggregationRuleV1::LoadTimeSum,
    AggregationRuleV1::None,
    AggregationRuleV1::None,
    AggregationRuleV1::None,
    AggregationRuleV1::BlockInternalPointAssignment,
    AggregationRuleV1::BlockCoordinateAssignment,
    AggregationRuleV1::None,
    AggregationRuleV1::EqualAreaWaterIntersection,
    AggregationRuleV1::TypedRelationProjection,
    AggregationRuleV1::TypedRelationProjection,
    AggregationRuleV1::TypedRelationProjection,
];

const EXPECTED_BINDINGS: [(&str, &str, RelationPayloadModeV1); 6] = [
    (
        "REFERENCE_FLOW",
        "COMMUTER_JOBS",
        RelationPayloadModeV1::SingleMetricFacet,
    ),
    (
        "REFERENCE_FLOW",
        "BORDER_SYNTHESIS",
        RelationPayloadModeV1::Empty,
    ),
    ("DYAD", "PRESENCE", RelationPayloadModeV1::ImplicitRelation),
    (
        "DYAD",
        "MEMBERSHIP",
        RelationPayloadModeV1::ImplicitRelation,
    ),
    (
        "DYAD",
        "SOLIDARITY",
        RelationPayloadModeV1::ImplicitRelation,
    ),
    ("DYAD", "COMMAND", RelationPayloadModeV1::Empty),
];

const EXPECTED_BINDING_METRICS: [Option<&str>; 6] = [
    Some("circulation/lodes-county-commuter-total-jobs"),
    None,
    Some("rootedness/presence"),
    Some("rootedness/membership"),
    Some("rootedness/solidarity"),
    None,
];

fn rejects_unknown<T: DeserializeOwned>() {
    assert!(serde_json::from_str::<T>(r#""NOT_A_CONTRACT_VALUE""#).is_err());
}

fn requires_field<T: DeserializeOwned>(source: &str, fragment: &str) {
    assert!(serde_json::from_str::<T>(source).is_ok());
    let missing = source.replace(fragment, "");
    assert_ne!(missing, source);
    assert!(serde_json::from_str::<T>(&missing).is_err());
}

#[allow(clippy::needless_range_loop)] // Const-generic arrays fix every group bound.
fn assert_identity_group<const N: usize>(
    start: usize,
    category: &str,
    domain: &str,
    expected: [(&str, &str, &str); N],
) {
    for index in 0..N {
        let row = RTD_V1_IDENTITY_REGISTRY[start + index];
        let (symbolic_name, authority, local_id) = expected[index];
        assert_eq!(row.category, category);
        assert_eq!(row.symbolic_name, symbolic_name);
        assert_eq!(row.identity.domain, domain);
        assert_eq!(row.identity.authority, authority);
        assert_eq!(row.identity.local_id, local_id);
    }
}

fn coordinate_signature(row: &RtdMetricRegistryRowV1) -> String {
    let mut output = String::new();
    for index in 0..32 {
        if index == row.coordinates.len() {
            return output;
        }
        if index > 0 {
            output.push(',');
        }
        output.push_str(row.coordinates[index].local_id);
    }
    output
}

#[test]
fn schema_limits_and_error_registry_are_exact() {
    assert_eq!(RTD_V1_SCHEMA_ID, "babylon.relational-territory-dossier");
    assert_eq!(RTD_MAX_FOCUS, 64);
    assert_eq!(RTD_MAX_REFERENCE_DIGESTS, 4_096);
    assert_eq!(RTD_MAX_COLLECTION_ITEMS, 65_535);
    assert_eq!(RTD_MAX_COORDINATES, 32);
    assert_eq!(RTD_MAX_HYPEREDGE_MEMBERS, 1_024);
    assert_eq!(RTD_MAX_PAYLOAD_FACETS, 256);
    assert_eq!(RTD_MAX_DECISION_SURFACE_REFS, 256);
    assert_eq!(RTD_MAX_PROVENANCE_REFS, 8_192);
    assert_eq!(RTD_MAX_CANONICAL_BYTES, 67_108_864);
    assert_eq!(RTD_V1_ERROR_REGISTRY, EXPECTED_ERRORS);
}

#[test]
#[allow(clippy::needless_range_loop)] // The identity registry has a fixed 69-row bound.
fn identity_registry_has_exact_unique_rows() {
    assert_eq!(RTD_V1_IDENTITY_REGISTRY.len(), 69);
    let mut identities = BTreeSet::new();
    for index in 0..69 {
        let row = RTD_V1_IDENTITY_REGISTRY[index];
        assert!(identities.insert((
            row.identity.domain,
            row.identity.authority,
            row.identity.local_id,
        )));
    }
}

#[test]
fn metric_identities_are_exact() {
    assert_identity_group(
        0,
        "metrics",
        "metric",
        EXPECTED_METRICS.map(|local_id| (local_id, "babylon.rtd.v1", local_id)),
    );
}

#[test]
fn unit_identities_are_exact() {
    assert_identity_group(
        18,
        "units",
        "unit",
        [
            ("JOBS", "babylon.rtd.v1", "jobs"),
            ("ESTABLISHMENTS", "babylon.rtd.v1", "establishments"),
            ("USD_CURRENT", "babylon.rtd.v1", "usd-current"),
            ("HOUSEHOLDS", "babylon.rtd.v1", "households"),
            ("PERSONS", "babylon.rtd.v1", "persons"),
            ("FACILITIES", "babylon.rtd.v1", "facilities"),
            ("FRACTION", "babylon.rtd.v1", "fraction"),
            ("TYPED_RELATION", "babylon.rtd.v1", "typed-relation"),
        ],
    );
}

#[test]
fn coordinate_identities_are_exact() {
    assert_identity_group(
        26,
        "coordinates",
        "dimension",
        [
            ("county", "babylon.rtd.v1", "county"),
            ("naics6", "babylon.rtd.v1", "naics6"),
            ("ownership", "babylon.rtd.v1", "ownership"),
            ("home_county", "babylon.rtd.v1", "home-county"),
            ("work_county", "babylon.rtd.v1", "work-county"),
            ("source", "babylon.rtd.v1", "source"),
            ("tenure", "babylon.rtd.v1", "tenure"),
            ("race", "babylon.rtd.v1", "race"),
            ("burden", "babylon.rtd.v1", "burden"),
            ("h3_cell", "babylon.rtd.v1", "h3-cell"),
            ("coercive_type", "babylon.rtd.v1", "coercive-type"),
            ("actor", "babylon.rtd.v1", "actor"),
            ("node", "babylon.rtd.v1", "node"),
        ],
    );
}

#[test]
fn native_scale_identities_are_exact() {
    assert_identity_group(
        39,
        "native_scales",
        "native-scale",
        [
            (
                "COUNTY_NAICS6_OWNERSHIP_YEAR",
                "babylon.rtd.v1",
                "county-naics6-ownership-year",
            ),
            (
                "COUNTY_OWNERSHIP_YEAR",
                "babylon.rtd.v1",
                "county-ownership-year",
            ),
            (
                "HOME_COUNTY_WORK_COUNTY_YEAR",
                "babylon.rtd.v1",
                "home-county-work-county-year",
            ),
            (
                "COUNTY_SOURCE_TENURE_TIME_RACE",
                "babylon.rtd.v1",
                "county-source-tenure-time-race",
            ),
            (
                "COUNTY_SOURCE_TIME_RACE",
                "babylon.rtd.v1",
                "county-source-time-race",
            ),
            (
                "COUNTY_SOURCE_BURDEN_TIME_RACE",
                "babylon.rtd.v1",
                "county-source-burden-time-race",
            ),
            ("H3_R7_VINTAGE", "babylon.rtd.v1", "h3-r7-vintage"),
            (
                "COUNTY_COERCIVE_TYPE_SOURCE",
                "babylon.rtd.v1",
                "county-coercive-type-source",
            ),
            (
                "ACTOR_NODE_VERIFIED_TICK",
                "babylon.rtd.v1",
                "actor-node-verified-tick",
            ),
        ],
    );
}

#[test]
fn producer_identities_are_exact() {
    assert_identity_group(
        48,
        "producers",
        "producer",
        [
            ("fact_qcew_annual", "babylon.data.v7", "fact_qcew_annual"),
            (
                "fact_qcew_county_rollup",
                "babylon.data.v7",
                "fact_qcew_county_rollup",
            ),
            (
                "fact_lodes_commuter_flow",
                "babylon.data.v7",
                "fact_lodes_commuter_flow",
            ),
            (
                "fact_census_housing",
                "babylon.data.v7",
                "fact_census_housing",
            ),
            ("fact_census_rent", "babylon.data.v7", "fact_census_rent"),
            (
                "fact_census_rent_burden",
                "babylon.data.v7",
                "fact_census_rent_burden",
            ),
            (
                "h3_res7_population",
                "babylon.data.v7",
                "h3_res7_population",
            ),
            ("h3_res7_workplace", "babylon.data.v7", "h3_res7_workplace"),
            (
                "fact_coercive_infrastructure",
                "babylon.data.v7",
                "fact_coercive_infrastructure",
            ),
            ("h3_res7_land_mask", "babylon.data.v7", "h3_res7_land_mask"),
            (
                "committed typed graph",
                "babylon.engine",
                "typed-graph-relations-at-verified-tick",
            ),
        ],
    );
}

#[test]
fn reference_identities_are_exact() {
    assert_identity_group(
        59,
        "references",
        "reference-artifact",
        [
            ("fact_qcew_annual", "babylon.data.v7", "fact_qcew_annual"),
            (
                "fact_qcew_county_rollup",
                "babylon.data.v7",
                "fact_qcew_county_rollup",
            ),
            (
                "fact_lodes_commuter_flow",
                "babylon.data.v7",
                "fact_lodes_commuter_flow",
            ),
            (
                "fact_census_housing",
                "babylon.data.v7",
                "fact_census_housing",
            ),
            ("fact_census_rent", "babylon.data.v7", "fact_census_rent"),
            (
                "fact_census_rent_burden",
                "babylon.data.v7",
                "fact_census_rent_burden",
            ),
            (
                "h3_res7_population",
                "babylon.data.v7",
                "h3_res7_population",
            ),
            ("h3_res7_workplace", "babylon.data.v7", "h3_res7_workplace"),
            (
                "fact_coercive_infrastructure",
                "babylon.data.v7",
                "fact_coercive_infrastructure",
            ),
            ("h3_res7_land_mask", "babylon.data.v7", "h3_res7_land_mask"),
        ],
    );
}

fn expected_evidence_classes(index: usize) -> &'static [EvidenceClassV1] {
    if index < 7 {
        &[EvidenceClassV1::Observed, EvidenceClassV1::Derived]
    } else if matches!(index, 8 | 9 | 10 | 13) {
        &[EvidenceClassV1::Observed]
    } else {
        &[EvidenceClassV1::Derived]
    }
}

fn expected_producer_authority(index: usize) -> &'static str {
    if index < 15 {
        "babylon.data.v7"
    } else {
        "babylon.engine"
    }
}

fn assert_reference_artifact(index: usize, row: &RtdMetricRegistryRowV1) {
    if index < 15 {
        let reference = row
            .reference_artifact
            .expect("first 15 metrics require a reference");
        assert_eq!(reference.domain, "reference-artifact");
        assert_eq!(reference.authority, "babylon.data.v7");
        assert_eq!(reference.local_id, EXPECTED_PRODUCERS[index]);
    } else {
        assert_eq!(row.reference_artifact, None);
    }
}

fn assert_metric_registry_row(index: usize) {
    let row = RTD_V1_METRIC_REGISTRY[index];
    assert_eq!(
        row.metric,
        TypedIdentityLiteralV1 {
            domain: "metric",
            authority: "babylon.rtd.v1",
            local_id: EXPECTED_METRICS[index],
        }
    );
    assert_eq!(row.representation, EXPECTED_REPRESENTATIONS[index]);
    assert_eq!(row.unit.domain, "unit");
    assert_eq!(row.unit.authority, "babylon.rtd.v1");
    assert_eq!(row.unit.local_id, EXPECTED_UNITS[index]);
    assert_eq!(row.value_kind, EXPECTED_VALUE_KINDS[index]);
    assert_eq!(row.native_scale.domain, "native-scale");
    assert_eq!(row.native_scale.authority, "babylon.rtd.v1");
    assert_eq!(row.native_scale.local_id, EXPECTED_SCALES[index]);
    assert_eq!(coordinate_signature(&row), EXPECTED_COORDINATES[index]);
    assert_eq!(row.evidence_classes, expected_evidence_classes(index));
    assert_eq!(row.aggregation_rule, EXPECTED_AGGREGATIONS[index]);
    assert_eq!(row.producer.domain, "producer");
    assert_eq!(row.producer.authority, expected_producer_authority(index));
    assert_eq!(row.producer.local_id, EXPECTED_PRODUCERS[index]);
    assert_eq!(row.reference_digest, EXPECTED_DIGESTS[index]);
    assert_reference_artifact(index, &row);
}

#[test]
#[allow(clippy::needless_range_loop)] // The metric registry has a fixed 18-row bound.
fn metric_registry_rows_are_exact() {
    assert_eq!(RTD_V1_METRIC_REGISTRY.len(), 18);
    for index in 0..18 {
        assert_metric_registry_row(index);
    }
}

#[test]
#[allow(clippy::needless_range_loop)] // The relation registry has a fixed six-row bound.
fn relation_binding_registry_rows_are_exact() {
    for index in 0..6 {
        let row = RTD_V1_RELATION_BINDING_REGISTRY[index];
        assert_eq!(
            (row.record_family, row.kind, row.payload_mode),
            EXPECTED_BINDINGS[index]
        );
        assert_eq!(
            row.metric.map(|metric| metric.local_id),
            EXPECTED_BINDING_METRICS[index]
        );
        if let Some(metric) = row.metric {
            assert_eq!(metric.domain, "metric");
            assert_eq!(metric.authority, "babylon.rtd.v1");
        }
    }
}

#[test]
fn schema_metadata_mutations_do_not_match_the_registry_contract() {
    let expected = RTD_V1_METRIC_REGISTRY[4];
    let mut aggregation_mutation = expected;
    aggregation_mutation.aggregation_rule = AggregationRuleV1::None;
    assert_ne!(aggregation_mutation, expected);
    let mut producer_mutation = expected;
    producer_mutation.producer = TypedIdentityLiteralV1 {
        domain: "producer",
        authority: "test",
        local_id: "wrong",
    };
    assert_ne!(producer_mutation, expected);
}

#[test]
#[allow(clippy::needless_range_loop)] // The error registry is a fixed 20-row contract.
fn rust_error_display_matches_the_schema_registry_exactly() {
    let errors = [
        RtdError::Json,
        RtdError::JsonDepth,
        RtdError::SchemaVersion,
        RtdError::UnknownField,
        RtdError::Enum,
        RtdError::Identity,
        RtdError::Digest,
        RtdError::NonNfc,
        RtdError::LimitExceeded,
        RtdError::DuplicateKey,
        RtdError::DanglingReference,
        RtdError::StatusValue,
        RtdError::NativeGrain,
        RtdError::UnsupportedDownscale,
        RtdError::H3BeforePer21,
        RtdError::MsaEvidence,
        RtdError::CanadaControl,
        RtdError::ForbiddenReduction,
        RtdError::VectorLimit,
        RtdError::CanonicalSize,
    ];
    for index in 0..20 {
        assert_eq!(errors[index].to_string(), RTD_V1_ERROR_REGISTRY[index]);
    }
}

#[test]
fn every_schema_enum_rejects_an_unknown_discriminant() {
    rejects_unknown::<AudienceV1>();
    rejects_unknown::<DurabilityV1>();
    rejects_unknown::<EvidenceClassV1>();
    rejects_unknown::<StatusV1>();
    rejects_unknown::<ValueKindV1>();
    rejects_unknown::<CoverageV1>();
    rejects_unknown::<MembershipKindV1>();
    rejects_unknown::<FacetFamilyV1>();
    rejects_unknown::<DyadKindV1>();
    rejects_unknown::<HyperedgeKindV1>();
    rejects_unknown::<FlowKindV1>();
    rejects_unknown::<RelationPayloadModeV1>();
    rejects_unknown::<GapReasonV1>();
    rejects_unknown::<MetricRepresentationV1>();
    rejects_unknown::<AggregationRuleV1>();
    rejects_unknown::<RtdCollectionKindV1>();
}

#[test]
fn schema_records_deny_unknown_fields_and_require_nullable_keys() {
    let explicit_null = r#"{
        "reference_id":{"domain":"reference","authority":"test","local_id":"r"},
        "sha256_hex":"0000000000000000000000000000000000000000000000000000000000000000",
        "artifact_schema_id_or_null":null,
        "vintage":"2023",
        "evidence_class":"Observed"
    }"#;
    requires_field::<ReferenceDigestV1>(
        explicit_null,
        "\n        \"artifact_schema_id_or_null\":null,",
    );
    let unknown = r#"{"domain":"entity","authority":"test","local_id":"x","extra":1}"#;
    assert!(serde_json::from_str::<TypedIdentityV1>(unknown).is_err());

    let membership = r#"{"membership_id":{"domain":"membership","authority":"test","local_id":"m"},"member_ref":{"domain":"county","authority":"census","local_id":"1"},"scale_ref":{"domain":"state","authority":"census","local_id":"2"},"membership_kind":"ADMINISTRATIVE","status":"PRESENT","weight_status":"ABSENT","weight_bits_or_null":null,"coverage":"COMPLETE","evidence_class":"Observed","provenance_refs":[]}"#;
    requires_field::<ScaleMembershipV1>(membership, r#""weight_bits_or_null":null,"#);
    let facet = r#"{"facet_id":{"domain":"facet","authority":"test","local_id":"f"},"family":"PRODUCTION_CIRCULATION","subject_ref":{"domain":"county","authority":"census","local_id":"1"},"metric_id":{"domain":"metric","authority":"test","local_id":"m"},"unit_id":{"domain":"unit","authority":"test","local_id":"u"},"native_scale":{"domain":"native-scale","authority":"test","local_id":"n"},"coordinates":[],"vintage":"2023","status":"UNKNOWN","value_kind":"UINT64_BITS","value_bits_or_null":null,"coverage":"UNKNOWN","evidence_class":"Observed","provenance_refs":[]}"#;
    requires_field::<FacetV1>(facet, r#""value_bits_or_null":null,"#);
    let gap = r#"{"gap_id":{"domain":"gap","authority":"test","local_id":"g"},"requested_metric_or_relation":{"domain":"metric","authority":"test","local_id":"m"},"status":"UNKNOWN","reason_code":"MISSING_GOVERNED_PRODUCER","required_producer_or_null":null,"provenance_refs":[]}"#;
    requires_field::<GapV1>(gap, r#""required_producer_or_null":null,"#);
    let provenance = format!(
        r#"{{"provenance_id":{{"domain":"provenance","authority":"test","local_id":"p"}},"artifact_digest":"{}","locator":"","vintage":"2023","evidence_class":"Observed","transformation_digest_or_null":null}}"#,
        "0".repeat(64)
    );
    requires_field::<ProvenanceV1>(&provenance, r#","transformation_digest_or_null":null"#);

    let draft = minimal_draft_json();
    requires_field::<RtdDossierDraftV1>(&draft, r#""fog_policy_digest":null,"#);
    requires_field::<RtdDossierDraftV1>(&draft, r#""knowledge_context_digest":null,"#);
    requires_field::<RtdDossierDraftV1>(&draft, r#""actor":null,"#);
}

fn minimal_draft_json() -> String {
    let zero = "0".repeat(64);
    format!(
        r#"{{"schema":"babylon.relational-territory-dossier","schema_version":1,"projection_version":1,"audience":"ADMIN_MATERIAL","durability":"IN_MEMORY","verified_tick":0,"graph_state_hash":"{zero}","nominal_world_hash":"{zero}","reference_digests":[],"definitions_digest":"{zero}","template_digest":"{zero}","fog_policy_digest":null,"knowledge_context_digest":null,"actor":null,"focus":[],"scale_memberships":[],"facets":[],"dyads":[],"hyperedges":[],"flows":[],"gaps":[],"provenance":[],"decision_surface":{{"question_id":{{"domain":"question","authority":"test","local_id":"q"}},"signal_refs":[],"action_refs":[],"receipt_refs":[],"archive_subject_refs":[]}}}}"#
    )
}

#[test]
fn language_neutral_schema_bytes_are_bound_to_rust() {
    assert_eq!(
        babylon_kernel::sha256_of(RTD_SCHEMA),
        RTD_CONTRACT_SOURCE_SHA256
    );
}
