//! Exact static contract for the epoch-2 canonical H3 identity relation.

use std::path::Path;

const MIGRATION_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/migrations/0002_h3_cell.sql");

const EXPECTED_SQL: &str = "\
CREATE TABLE babylon_ref.h3_cell (
    cell_id BIGINT NOT NULL,
    resolution SMALLINT NOT NULL,
    immediate_parent BIGINT,
    ancestor_r4 BIGINT,
    ancestor_r5 BIGINT,
    ancestor_r6 BIGINT,
    ancestor_r7 BIGINT,
    CONSTRAINT h3_cell_pkey PRIMARY KEY (cell_id),
    CONSTRAINT h3_cell_id_positive CHECK (cell_id > 0),
    CONSTRAINT h3_cell_resolution_range CHECK (resolution BETWEEN 0 AND 15),
    CONSTRAINT h3_cell_resolution_matches_id CHECK (
        resolution = ((cell_id >> 52) & 15)::SMALLINT
    ),
    CONSTRAINT h3_cell_immediate_parent_matches CHECK (
        CASE
            WHEN resolution = 0 THEN immediate_parent IS NULL
            WHEN resolution BETWEEN 1 AND 15 THEN
                immediate_parent IS NOT NULL
                AND immediate_parent = (
                    (cell_id & ~(15::BIGINT << 52))
                    | ((resolution - 1)::BIGINT << 52)
                    | ((1::BIGINT << (3 * (16 - resolution))) - 1)
                )
            ELSE FALSE
        END
    ),
    CONSTRAINT h3_cell_ancestor_r4_matches CHECK (
        CASE
            WHEN resolution BETWEEN 0 AND 3 THEN ancestor_r4 IS NULL
            WHEN resolution BETWEEN 4 AND 15 THEN
                ancestor_r4 IS NOT NULL
                AND ancestor_r4 = (
                    (cell_id & ~(15::BIGINT << 52))
                    | (4::BIGINT << 52)
                    | ((1::BIGINT << 33) - 1)
                )
            ELSE FALSE
        END
    ),
    CONSTRAINT h3_cell_ancestor_r5_matches CHECK (
        CASE
            WHEN resolution BETWEEN 0 AND 4 THEN ancestor_r5 IS NULL
            WHEN resolution BETWEEN 5 AND 15 THEN
                ancestor_r5 IS NOT NULL
                AND ancestor_r5 = (
                    (cell_id & ~(15::BIGINT << 52))
                    | (5::BIGINT << 52)
                    | ((1::BIGINT << 30) - 1)
                )
            ELSE FALSE
        END
    ),
    CONSTRAINT h3_cell_ancestor_r6_matches CHECK (
        CASE
            WHEN resolution BETWEEN 0 AND 5 THEN ancestor_r6 IS NULL
            WHEN resolution BETWEEN 6 AND 15 THEN
                ancestor_r6 IS NOT NULL
                AND ancestor_r6 = (
                    (cell_id & ~(15::BIGINT << 52))
                    | (6::BIGINT << 52)
                    | ((1::BIGINT << 27) - 1)
                )
            ELSE FALSE
        END
    ),
    CONSTRAINT h3_cell_ancestor_r7_matches CHECK (
        CASE
            WHEN resolution BETWEEN 0 AND 6 THEN ancestor_r7 IS NULL
            WHEN resolution BETWEEN 7 AND 15 THEN
                ancestor_r7 IS NOT NULL
                AND ancestor_r7 = (
                    (cell_id & ~(15::BIGINT << 52))
                    | (7::BIGINT << 52)
                    | ((1::BIGINT << 24) - 1)
                )
            ELSE FALSE
        END
    ),
    CONSTRAINT h3_cell_immediate_parent_fkey FOREIGN KEY (immediate_parent)
        REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_cell_ancestor_r4_fkey FOREIGN KEY (ancestor_r4)
        REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_cell_ancestor_r5_fkey FOREIGN KEY (ancestor_r5)
        REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_cell_ancestor_r6_fkey FOREIGN KEY (ancestor_r6)
        REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_cell_ancestor_r7_fkey FOREIGN KEY (ancestor_r7)
        REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED
);
REVOKE ALL ON TABLE babylon_ref.h3_cell FROM PUBLIC;
";

fn migration_sql() -> String {
    std::fs::read_to_string(Path::new(MIGRATION_PATH)).unwrap_or_else(|error| {
        panic!("failed to read {MIGRATION_PATH}: {error}");
    })
}

#[test]
fn h3_cell_epoch_sql_is_exact_and_additive_only() {
    let sql = migration_sql();
    assert_eq!(sql, EXPECTED_SQL);
    assert_eq!(sql.as_bytes().last(), Some(&b'\n'));

    let uppercase = sql.to_ascii_uppercase();
    for prohibited in [
        "BEGIN",
        "COMMIT",
        "ROLLBACK",
        "START TRANSACTION",
        "INSERT",
        "UPDATE",
        "DELETE",
        "CREATE EXTENSION",
        "H3INDEX",
        "CREATE INDEX",
        "CREATE VIEW",
        "BRIDGE",
        "CAMPAIGN",
        "OUTBOX",
        "WRITER",
        "BABYLON_INTEL",
        "GRANT",
    ] {
        assert!(
            !uppercase.contains(prohibited),
            "epoch-2 migration contains prohibited surface {prohibited}"
        );
    }
}

#[test]
fn parent_construction_rewrites_resolution_and_trailing_digits() {
    let sql = migration_sql();
    assert!(sql.contains("& ~(15::BIGINT << 52)"));
    assert!(sql.contains("| ((resolution - 1)::BIGINT << 52)"));
    assert!(sql.contains("| ((1::BIGINT << (3 * (16 - resolution))) - 1)"));
    assert!(!sql.to_ascii_lowercase().contains("descendant"));
    assert!(!sql.to_ascii_lowercase().contains("between cell_id"));
}
