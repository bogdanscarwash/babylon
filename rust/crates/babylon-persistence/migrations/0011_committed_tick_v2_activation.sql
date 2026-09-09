-- One-way activation for the V2 committed-tick boundary. The caller inserts
-- the V2-active authority row after this SQL body, making that row the final
-- DML before the transaction commits. The caller uses SERIALIZABLE and this
-- top-level ACCESS EXCLUSIVE lock is deliberately the first snapshot-relevant
-- operation, so the following inventory observes any writer that committed
-- while the lock was pending and the retained locks prevent subsequent drift.

LOCK TABLE
    babylon_state.campaign,
    babylon_state.campaign_foundation,
    babylon_state.tick_commit,
    babylon_state.tick_action_batch_v1,
    babylon_state.graph_node_v1,
    babylon_state.graph_node_f64_v1,
    babylon_state.graph_edge_v1,
    babylon_state.graph_hyperedge_v1,
    babylon_state.graph_hyperedge_member_v1,
    babylon_state.graph_edge_f64_v1,
    babylon_state.graph_node_currency_v1,
    babylon_state.graph_hyperedge_f64_v1,
    babylon_state.world_register_v1,
    babylon_state.territory_state_v1,
    babylon_state.territory_state_field_v1,
    babylon_state.hex_state_delta_v1,
    babylon_state.organization_state_v1,
    babylon_state.organization_territory_v1,
    babylon_state.organization_state_field_v1,
    babylon_state.tick_event_v1,
    babylon_state.tick_event_field_v1,
    babylon_state.checkpoint_manifest,
    babylon_state.checkpoint_section_v1,
    babylon_state.archive_dirty_receipt_v1
IN ACCESS EXCLUSIVE MODE;

DO $committed_tick_v2_inventory_preflight$
BEGIN
    IF EXISTS (
        WITH live(relation_name, observed_row_count) AS (
            SELECT 'babylon_state.campaign', pg_catalog.count(*) FROM babylon_state.campaign
            UNION ALL
            SELECT 'babylon_state.campaign_foundation', pg_catalog.count(*) FROM babylon_state.campaign_foundation
            UNION ALL
            SELECT 'babylon_state.tick_commit', pg_catalog.count(*) FROM babylon_state.tick_commit
            UNION ALL
            SELECT 'babylon_state.tick_action_batch_v1', pg_catalog.count(*) FROM babylon_state.tick_action_batch_v1
            UNION ALL
            SELECT 'babylon_state.graph_node_v1', pg_catalog.count(*) FROM babylon_state.graph_node_v1
            UNION ALL
            SELECT 'babylon_state.graph_node_f64_v1', pg_catalog.count(*) FROM babylon_state.graph_node_f64_v1
            UNION ALL
            SELECT 'babylon_state.graph_edge_v1', pg_catalog.count(*) FROM babylon_state.graph_edge_v1
            UNION ALL
            SELECT 'babylon_state.graph_hyperedge_v1', pg_catalog.count(*) FROM babylon_state.graph_hyperedge_v1
            UNION ALL
            SELECT 'babylon_state.graph_hyperedge_member_v1', pg_catalog.count(*) FROM babylon_state.graph_hyperedge_member_v1
            UNION ALL
            SELECT 'babylon_state.graph_edge_f64_v1', pg_catalog.count(*) FROM babylon_state.graph_edge_f64_v1
            UNION ALL
            SELECT 'babylon_state.graph_node_currency_v1', pg_catalog.count(*) FROM babylon_state.graph_node_currency_v1
            UNION ALL
            SELECT 'babylon_state.graph_hyperedge_f64_v1', pg_catalog.count(*) FROM babylon_state.graph_hyperedge_f64_v1
            UNION ALL
            SELECT 'babylon_state.world_register_v1', pg_catalog.count(*) FROM babylon_state.world_register_v1
            UNION ALL
            SELECT 'babylon_state.territory_state_v1', pg_catalog.count(*) FROM babylon_state.territory_state_v1
            UNION ALL
            SELECT 'babylon_state.territory_state_field_v1', pg_catalog.count(*) FROM babylon_state.territory_state_field_v1
            UNION ALL
            SELECT 'babylon_state.hex_state_delta_v1', pg_catalog.count(*) FROM babylon_state.hex_state_delta_v1
            UNION ALL
            SELECT 'babylon_state.organization_state_v1', pg_catalog.count(*) FROM babylon_state.organization_state_v1
            UNION ALL
            SELECT 'babylon_state.organization_territory_v1', pg_catalog.count(*) FROM babylon_state.organization_territory_v1
            UNION ALL
            SELECT 'babylon_state.organization_state_field_v1', pg_catalog.count(*) FROM babylon_state.organization_state_field_v1
            UNION ALL
            SELECT 'babylon_state.tick_event_v1', pg_catalog.count(*) FROM babylon_state.tick_event_v1
            UNION ALL
            SELECT 'babylon_state.tick_event_field_v1', pg_catalog.count(*) FROM babylon_state.tick_event_field_v1
            UNION ALL
            SELECT 'babylon_state.checkpoint_manifest', pg_catalog.count(*) FROM babylon_state.checkpoint_manifest
            UNION ALL
            SELECT 'babylon_state.checkpoint_section_v1', pg_catalog.count(*) FROM babylon_state.checkpoint_section_v1
            UNION ALL
            SELECT 'babylon_state.archive_dirty_receipt_v1', pg_catalog.count(*) FROM babylon_state.archive_dirty_receipt_v1
        )
        SELECT 1
          FROM live
          FULL OUTER JOIN babylon_meta.committed_tick_v2_incompatible_inventory AS inventory
            ON inventory.relation_name = live.relation_name
         WHERE live.relation_name IS NULL
            OR inventory.relation_name IS NULL
            OR live.observed_row_count <> 0
            OR inventory.observed_row_count <> 0
            OR inventory.disposition_tag <> 1
            OR live.observed_row_count <> inventory.observed_row_count
    ) THEN
        RAISE EXCEPTION USING
            ERRCODE = 'P0001',
            MESSAGE = 'committed_tick_v2_activation_refused_incompatible_inventory';
    END IF;
END
$committed_tick_v2_inventory_preflight$;

-- These writer tables are proven empty above. Remove the child first so the
-- one-way activation cannot leave a second event layout callable after V2 is
-- authoritative.
DROP TABLE babylon_state.tick_event_field_v1;
DROP TABLE babylon_state.tick_event_v1;

ALTER TABLE babylon_state.tick_commit
    DROP CONSTRAINT tick_commit_envelope_layout_v1;
ALTER TABLE babylon_state.tick_commit
    ADD CONSTRAINT tick_commit_envelope_layout_v2 CHECK (envelope_layout_version = 2);
