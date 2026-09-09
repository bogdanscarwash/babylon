//! Static guards for the exact epoch-7 canonical H3 reader shape verifier.

const SHAPE_SQL: &str = include_str!("../src/schema_epoch_v7_shape.sql");

const CANONICAL_READER_VIEWS: [&str; 10] = [
    "v_county_value_aggregate",
    "v_hex_aid",
    "v_hex_economic",
    "v_hex_heat",
    "v_hex_intel",
    "v_hex_mobilize",
    "v_hex_state_asof",
    "v_national_value_aggregate",
    "v_state_value_aggregate",
    "view_runtime_trace_emission",
];

#[test]
fn v7_shape_is_bounded_and_closes_the_exact_reader_view_set() {
    assert!(SHAPE_SQL.ends_with('\n'));
    assert!(SHAPE_SQL.len() <= 65_536);

    for view in CANONICAL_READER_VIEWS {
        assert!(SHAPE_SQL.contains(view), "v7 shape omits view {view}");
    }
    for required in [
        "expected_views",
        "present_views",
        "pg_catalog.pg_class",
        "pg_catalog.pg_namespace",
        "relation.relkind = 'v'",
        "pg_catalog.count(*) IN (0, 10)",
    ] {
        assert!(SHAPE_SQL.contains(required), "v7 shape omits {required:?}");
    }
    assert!(!SHAPE_SQL.contains("v_global_phi_balance"));
}

#[test]
fn v7_shape_requires_bigint_cell_identity_and_no_legacy_reader_dependency() {
    for required in [
        "expected_cell_id_columns",
        "cell_id",
        "h3_index",
        "pg_catalog.int8",
        "pg_catalog.count(*) = 6",
        "pg_catalog.pg_attribute",
        "pg_catalog.pg_rewrite",
        "pg_catalog.pg_depend",
        "referenced_columns",
    ] {
        assert!(SHAPE_SQL.contains(required), "v7 shape omits {required:?}");
    }
    for relation in [
        "dynamic_hex_state",
        "hex_activity",
        "hex_cell",
        "hex_latest",
        "hex_map",
        "hex_r8_linear_features_reference",
        "hex_r8_reference",
        "hex_spatial_map",
        "hex_state",
        "hex_substrate",
        "hex_terrain_state",
        "immutable_reference_lodes_od_matrix",
        "infrastructure_link_state",
        "org_snapshot",
        "tick_event",
    ] {
        assert!(
            SHAPE_SQL.contains(relation),
            "v7 shape omits predecessor relation {relation}"
        );
    }
}
