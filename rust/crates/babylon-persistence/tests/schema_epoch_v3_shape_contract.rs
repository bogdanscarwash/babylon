//! Static guards for the exact epoch-3 shape verifier.

const SHAPE_SQL: &str = include_str!("../src/schema_epoch_v3_shape.sql");

#[test]
fn v3_shape_verifier_is_bounded_and_checks_the_closed_authority_surface() {
    assert!(SHAPE_SQL.ends_with('\n'));
    assert!(SHAPE_SQL.len() <= 65_536);

    for required in [
        "babylon_ref",
        "babylon_state",
        "babylon_meta",
        "h3_cell",
        "h3_reference_cohort",
        "h3_reference_membership",
        "schema_migration",
        "h3_reference_cohort_membership_digest_length",
        "h3_reference_membership_origin_closed",
        "h3_reference_membership_cohort_fkey",
        "h3_reference_membership_cell_fkey",
        "h3_reference_membership_cell_id_idx",
        "artifact_manifest_version",
        "h3_reference_cohort_artifact_manifest_version_length",
        "constraint_row.conkey",
        "constraint_row.confkey",
        "constraint_row.confupdtype",
        "constraint_row.confdeltype",
        "constraint_row.confmatchtype",
        "expected_reference_constraints",
        "index_row.indnatts",
        "pg_catalog.has_table_privilege",
        "'MAINTAIN'",
        "pg_catalog.has_column_privilege",
        "pg_catalog.aclexplode(column_row.attacl)",
        "LIMIT 39",
        "LIMIT 26",
        "LIMIT 7",
        "LIMIT 11",
    ] {
        assert!(
            SHAPE_SQL.contains(required),
            "v3 shape verifier omits governed contract {required:?}"
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
            "v3 shape verifier depends on prohibited H3 surface {prohibited:?}"
        );
    }
}
