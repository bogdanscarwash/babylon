//! Static guards for the exact epoch-4 committed-tick storage shape verifier.

const SHAPE_SQL: &str = include_str!("../src/schema_epoch_v4_shape.sql");

#[test]
fn v4_shape_verifier_is_bounded_and_checks_the_closed_authority_surface() {
    assert!(SHAPE_SQL.ends_with('\n'));
    assert!(SHAPE_SQL.len() <= 65_536);

    for required in [
        "babylon_ref",
        "babylon_state",
        "babylon_meta",
        "campaign",
        "tick_commit",
        "tick_graph_row",
        "tick_state_row",
        "tick_event_row",
        "tick_subsystem_row",
        "tick_conservation_row",
        "tick_boundary_flow_row",
        "tick_checkpoint_row",
        "tick_archive_dirty_receipt_row",
        "campaign_replay_session_ascii_graphic",
        "tick_commit_envelope_layout_v1",
        "tick_archive_dirty_receipt_row_campaign_tick_fkey",
        "tick_archive_dirty_receipt_row_ordinal_range",
        "row_ordinal",
        "pg_catalog.has_table_privilege",
        "pg_catalog.has_column_privilege",
        "pg_catalog.aclexplode",
        "expected_columns",
        "expected_constraints",
        "expected_indexes",
        "expected_classes",
    ] {
        assert!(
            SHAPE_SQL.contains(required),
            "v4 shape verifier omits governed contract {required:?}"
        );
    }
}
