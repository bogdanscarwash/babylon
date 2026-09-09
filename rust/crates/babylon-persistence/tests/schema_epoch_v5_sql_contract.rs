//! Static contract for additive Migration 0005 spatial reference products.

const MIGRATION_SQL: &str = include_str!("../migrations/0005_spatial_reference_products.sql");

#[test]
fn migration_five_is_schema_qualified_additive_reference_storage_only() {
    assert!(MIGRATION_SQL.ends_with('\n'));
    assert!(MIGRATION_SQL.len() <= 65_536);

    for relation in [
        "babylon_ref.reference_product",
        "babylon_ref.county_identity",
        "babylon_ref.place_identity",
        "babylon_ref.h3_land_fraction",
        "babylon_ref.h3_population_count",
        "babylon_ref.h3_workplace_count",
        "babylon_ref.county_h3_land_area",
        "babylon_ref.county_place_h3_land_area",
    ] {
        assert!(
            MIGRATION_SQL.contains(&format!("CREATE TABLE {relation}")),
            "Migration 0005 omits {relation}"
        );
        assert!(
            MIGRATION_SQL.contains(&format!("REVOKE ALL ON TABLE {relation} FROM PUBLIC")),
            "Migration 0005 leaves PUBLIC authority on {relation}"
        );
    }

    for required in [
        "REFERENCES babylon_ref.h3_reference_cohort(ref_digest)",
        "REFERENCES babylon_ref.h3_reference_membership(ref_digest, cell_id, origin)",
        "census_county_h3_land_overlap_mi_2023",
        "census_county_place_h3_land_overlap_mi_2023",
        "land_fraction_ppm",
        "population_count",
        "workplace_count",
        "land_area_m2",
        "place_land_area_m2",
        "cell_mi_land_area_m2",
        "place_land_area_share_ppb",
        "COLLATE \"C\"",
    ] {
        assert!(
            MIGRATION_SQL.contains(required),
            "Migration 0005 omits governed contract {required:?}"
        );
    }
    assert_eq!(
        MIGRATION_SQL
            .matches("DEFERRABLE INITIALLY DEFERRED")
            .count(),
        17,
        "every epoch-5 foreign key must share the deferred exact-bundle law"
    );

    let uppercase = MIGRATION_SQL.to_ascii_uppercase();
    for prohibited in [
        "BEGIN;",
        "COMMIT;",
        "ROLLBACK;",
        "START TRANSACTION",
        "INSERT ",
        "UPDATE ",
        "DELETE ",
        "DROP ",
        "ALTER ",
        "TRUNCATE",
        "CREATE VIEW",
        "CREATE EXTENSION",
        "CREATE DOMAIN",
        "GRANT ",
        " WEIGHT",
    ] {
        assert!(
            !uppercase.contains(prohibited),
            "Migration 0005 contains prohibited surface {prohibited}"
        );
    }
}

#[test]
fn migration_five_keeps_unlike_measures_in_distinct_relations() {
    for declaration in [
        "land_fraction_ppm INTEGER NOT NULL,",
        "population_count BIGINT NOT NULL,",
        "workplace_count BIGINT NOT NULL,",
        "land_area_m2 BIGINT NOT NULL,",
        "place_land_area_m2 BIGINT NOT NULL,",
    ] {
        assert_eq!(
            MIGRATION_SQL
                .lines()
                .filter(|line| line.trim() == declaration)
                .count(),
            1,
            "unexpected declaration count for {declaration}"
        );
    }
    assert!(!MIGRATION_SQL.to_ascii_lowercase().contains("weight"));
}

#[test]
fn every_cell_product_requires_direct_membership_in_its_named_cohort() {
    assert!(MIGRATION_SQL.contains(concat!(
        "CREATE UNIQUE INDEX h3_reference_membership_origin_key\n",
        "    ON babylon_ref.h3_reference_membership (ref_digest, cell_id, origin);"
    )));
    assert_eq!(
        MIGRATION_SQL
            .lines()
            .filter(|line| line.trim() == "membership_origin SMALLINT NOT NULL,")
            .count(),
        5,
        "every H3-bearing product relation must store the governed origin discriminator"
    );
    assert_eq!(
        MIGRATION_SQL
            .matches("FOREIGN KEY (ref_digest, cell_id, membership_origin)")
            .count(),
        5,
        "every H3-bearing product relation must bind cohort, cell, and origin together"
    );
    assert_eq!(
        MIGRATION_SQL
            .matches("REFERENCES babylon_ref.h3_reference_membership(ref_digest, cell_id, origin)",)
            .count(),
        5,
        "every H3-bearing product relation must reference exact cohort membership"
    );
    assert_eq!(
        MIGRATION_SQL
            .matches("CHECK (membership_origin = 1)")
            .count(),
        5,
        "product cells must be direct artifact members, never derived ancestors"
    );
    assert!(!MIGRATION_SQL.contains("REFERENCES babylon_ref.h3_cell(cell_id)"));
}
