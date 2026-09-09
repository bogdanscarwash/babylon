//! Exact static contract for the first Rust-owned `PostgreSQL` schema epoch.

use std::path::Path;

const MIGRATION_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/migrations/0001_owned_schema_epoch.sql"
);

const EXPECTED_SQL: &str = "\
CREATE SCHEMA IF NOT EXISTS babylon_ref AUTHORIZATION CURRENT_USER;
REVOKE ALL ON SCHEMA babylon_ref FROM PUBLIC;

CREATE SCHEMA IF NOT EXISTS babylon_state AUTHORIZATION CURRENT_USER;
REVOKE ALL ON SCHEMA babylon_state FROM PUBLIC;

CREATE SCHEMA IF NOT EXISTS babylon_meta AUTHORIZATION CURRENT_USER;
REVOKE ALL ON SCHEMA babylon_meta FROM PUBLIC;

CREATE TABLE babylon_state.schema_migration (
    version BIGINT PRIMARY KEY CHECK (version > 0),
    checksum BYTEA NOT NULL CHECK (pg_catalog.octet_length(checksum) = 32)
);
REVOKE ALL ON TABLE babylon_state.schema_migration FROM PUBLIC;
";

fn migration_sql() -> String {
    std::fs::read_to_string(Path::new(MIGRATION_PATH)).unwrap_or_else(|error| {
        panic!("failed to read {MIGRATION_PATH}: {error}");
    })
}

#[test]
fn owned_schema_epoch_sql_matches_adr225() {
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
        "APPLIED_AT",
        "STATUS",
        "DESCRIPTION",
        "GRANT",
        "PUBLIC.",
        "TICK_COMMIT",
        "H3",
        "CAMPAIGN",
        "OUTBOX",
        "WRITER",
        "BABYLON_INTEL",
        "POSTGRES",
    ] {
        assert!(
            !uppercase.contains(prohibited),
            "migration contains prohibited surface {prohibited}"
        );
    }
}
