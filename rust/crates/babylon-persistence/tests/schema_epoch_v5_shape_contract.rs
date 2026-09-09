//! Static bounded contract for the epoch-5 catalog oracle.

const SHAPE_SQL: &str = include_str!("../src/schema_epoch_v5_shape.sql");

#[test]
fn epoch_five_shape_is_bounded_exact_and_authority_closed() {
    assert!(SHAPE_SQL.ends_with('\n'));
    assert!(SHAPE_SQL.len() <= 65_536);
    for required in [
        "pg_catalog.count(*) = 22 FROM expected_relations",
        "pg_catalog.count(*) = 22 FROM owned_relations",
        "pg_catalog.count(*) = 137 FROM relation_columns",
        "pg_catalog.count(*) = 179 FROM relation_constraints",
        "pg_catalog.count(*) = 28 FROM relation_indexes",
        "pg_catalog.count(*) = 50 FROM owned_classes",
        "contype = 'f' AND NOT (condeferrable AND condeferred)",
        "acl.grantee = 0",
        "has_table_privilege",
        "has_column_privilege",
        "has_schema_privilege",
    ] {
        assert!(SHAPE_SQL.contains(required), "shape omits {required:?}");
    }
    for relation in [
        "reference_product",
        "county_identity",
        "place_identity",
        "h3_land_fraction",
        "h3_population_count",
        "h3_workplace_count",
        "county_h3_land_area",
        "county_place_h3_land_area",
    ] {
        assert!(SHAPE_SQL.contains(relation), "shape omits {relation}");
    }
}
