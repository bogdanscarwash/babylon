//! Static contract for additive Migration 0004 committed-tick storage.

const MIGRATION_SQL: &str = include_str!("../migrations/0004_committed_tick_storage.sql");

#[test]
fn migration_four_is_schema_qualified_additive_storage_only() {
    assert!(MIGRATION_SQL.ends_with('\n'));
    assert!(MIGRATION_SQL.len() <= 65_536);

    for relation in [
        "babylon_state.campaign",
        "babylon_state.tick_commit",
        "babylon_state.tick_graph_row",
        "babylon_state.tick_state_row",
        "babylon_state.tick_event_row",
        "babylon_state.tick_subsystem_row",
        "babylon_state.tick_conservation_row",
        "babylon_state.tick_boundary_flow_row",
        "babylon_state.tick_checkpoint_row",
        "babylon_state.tick_archive_dirty_receipt_row",
    ] {
        assert!(
            MIGRATION_SQL.contains(&format!("CREATE TABLE {relation}")),
            "Migration 0004 omits {relation}"
        );
        assert!(
            MIGRATION_SQL.contains(&format!("REVOKE ALL ON TABLE {relation} FROM PUBLIC")),
            "Migration 0004 leaves PUBLIC authority on {relation}"
        );
    }

    let uppercase = MIGRATION_SQL.to_ascii_uppercase();
    for prohibited in [
        "BEGIN;",
        "COMMIT;",
        "ROLLBACK;",
        "START TRANSACTION",
        "INSERT",
        "UPDATE",
        "DELETE",
        "DROP ",
        "ALTER ",
        "TRUNCATE",
        "CREATE VIEW",
        "CREATE EXTENSION",
        "BABYLON_INTEL",
        "GRANT",
        "PUBLIC.TICK_COMMIT",
    ] {
        assert!(
            !uppercase.contains(prohibited),
            "Migration 0004 contains prohibited surface {prohibited}"
        );
    }
}

#[test]
fn every_family_row_is_marker_last_compatible_and_byte_exact() {
    assert_eq!(
        MIGRATION_SQL
            .matches("row_ordinal INTEGER NOT NULL")
            .count(),
        8
    );
    assert_eq!(MIGRATION_SQL.matches("row_key BYTEA NOT NULL").count(), 8);
    assert_eq!(
        MIGRATION_SQL.matches("row_payload BYTEA NOT NULL").count(),
        8
    );
    assert_eq!(
        MIGRATION_SQL
            .matches("REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)")
            .count(),
        8
    );
    assert_eq!(
        MIGRATION_SQL
            .matches("DEFERRABLE INITIALLY DEFERRED")
            .count(),
        10
    );
    assert_eq!(MIGRATION_SQL.matches("row_pkey PRIMARY KEY").count(), 8);
    assert_eq!(
        MIGRATION_SQL.matches("resolve_tick, row_ordinal").count(),
        8
    );
    assert_eq!(
        MIGRATION_SQL
            .matches("row_ordinal BETWEEN 0 AND 1048575")
            .count(),
        8
    );
    assert!(!MIGRATION_SQL.contains("PRIMARY KEY (campaign_id, resolve_tick, row_key)"));
    assert_eq!(
        MIGRATION_SQL
            .matches("pg_catalog.octet_length(row_key) BETWEEN 1 AND 67108856")
            .count(),
        8
    );
    assert_eq!(
        MIGRATION_SQL
            .matches("pg_catalog.octet_length(row_payload) BETWEEN 0 AND 67108855")
            .count(),
        8
    );
}
