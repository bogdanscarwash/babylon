//! Static guards for the exact epoch-2 shape verifier.

const SHAPE_SQL: &str = include_str!("../src/schema_epoch_v2_shape.sql");

#[test]
fn v2_shape_verifier_is_bounded_and_checks_the_closed_authority_surface() {
    assert!(SHAPE_SQL.ends_with('\n'));
    assert!(SHAPE_SQL.len() <= 65_536);

    for required in [
        "babylon_ref",
        "babylon_state",
        "babylon_meta",
        "h3_cell",
        "schema_migration",
        "h3_cell_resolution_matches_id",
        "h3_cell_immediate_parent_matches",
        "h3_cell_ancestor_r7_matches",
        "h3_cell_immediate_parent_fkey",
        "pg_catalog.has_table_privilege",
        "'MAINTAIN'",
        "pg_catalog.has_column_privilege",
        "pg_catalog.aclexplode(column_row.attacl)",
        "LIMIT 15",
        "LIMIT 10",
        "LIMIT 5",
    ] {
        assert!(
            SHAPE_SQL.contains(required),
            "v2 shape verifier omits governed contract {required:?}"
        );
    }

    let lowercase = SHAPE_SQL.to_ascii_lowercase();
    for prohibited in [
        "h3index",
        "h3_is_valid_cell",
        "h3_cell_to_parent",
        "create extension",
        "gist",
        "spgist",
    ] {
        assert!(
            !lowercase.contains(prohibited),
            "v2 shape verifier depends on prohibited H3 surface {prohibited:?}"
        );
    }
}
