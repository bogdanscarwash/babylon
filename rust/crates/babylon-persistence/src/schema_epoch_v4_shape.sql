WITH database_owner AS MATERIALIZED (
    SELECT database_row.datdba AS owner_oid
    FROM pg_catalog.pg_database AS database_row
    WHERE database_row.datname = pg_catalog.current_database()
),
owned_schemas AS MATERIALIZED (
    SELECT namespace.oid, namespace.nspname, namespace.nspowner, namespace.nspacl
    FROM pg_catalog.pg_namespace AS namespace
    WHERE namespace.nspname IN ('babylon_ref', 'babylon_state', 'babylon_meta')
    ORDER BY namespace.nspname
    LIMIT 4
),
expected_relations(nspname, relname, column_count, constraint_count, index_count) AS (
    VALUES
        ('babylon_ref'::pg_catalog.text, 'h3_cell'::pg_catalog.text, 7, 14, 1),
        ('babylon_ref', 'h3_reference_cohort', 13, 15, 2),
        ('babylon_ref', 'h3_reference_membership', 3, 6, 2),
        ('babylon_state', 'campaign', 8, 9, 1),
        ('babylon_state', 'schema_migration', 2, 3, 1),
        ('babylon_state', 'tick_archive_dirty_receipt_row', 5, 6, 1),
        ('babylon_state', 'tick_boundary_flow_row', 5, 6, 1),
        ('babylon_state', 'tick_checkpoint_row', 5, 6, 1),
        ('babylon_state', 'tick_commit', 5, 6, 1),
        ('babylon_state', 'tick_conservation_row', 5, 6, 1),
        ('babylon_state', 'tick_event_row', 5, 6, 1),
        ('babylon_state', 'tick_graph_row', 5, 6, 1),
        ('babylon_state', 'tick_state_row', 5, 6, 1),
        ('babylon_state', 'tick_subsystem_row', 5, 6, 1)
),
owned_relations AS MATERIALIZED (
    SELECT relation.oid, namespace.nspname, relation.relname, relation.relowner,
           relation.relacl, relation.reltype
    FROM pg_catalog.pg_class AS relation
    JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace
    WHERE namespace.nspname IN ('babylon_ref', 'babylon_state')
      AND relation.relkind = 'r'
      AND relation.relpersistence = 'p'
    ORDER BY namespace.nspname, relation.relname
    LIMIT 15
),
intel_role AS MATERIALIZED (
    SELECT role_row.oid
    FROM pg_catalog.pg_roles AS role_row
    WHERE role_row.rolname = 'babylon_intel'
    LIMIT 2
),
relation_columns AS MATERIALIZED (
    SELECT relation.nspname, relation.relname, attribute.attnum, attribute.attname,
           attribute.atttypid, attribute.attnotnull, attribute.attidentity,
           attribute.attgenerated, attribute.attcollation, attribute.attacl,
           default_row.oid IS NOT NULL AS has_default
    FROM owned_relations AS relation
    JOIN pg_catalog.pg_attribute AS attribute ON attribute.attrelid = relation.oid
    LEFT JOIN pg_catalog.pg_attrdef AS default_row
      ON default_row.adrelid = attribute.attrelid
     AND default_row.adnum = attribute.attnum
    WHERE attribute.attnum > 0 AND NOT attribute.attisdropped
    ORDER BY relation.nspname, relation.relname, attribute.attnum
    LIMIT 79
),
expected_columns(
    nspname, relname, attnum, attname, atttypid, attnotnull, attcollation
) AS (
    VALUES
        ('babylon_ref'::pg_catalog.text, 'h3_cell'::pg_catalog.text, 1::pg_catalog.int2,
         'cell_id'::pg_catalog.name, 'pg_catalog.int8'::pg_catalog.regtype, true, 0::pg_catalog.oid),
        ('babylon_ref', 'h3_cell', 2, 'resolution', 'pg_catalog.int2'::pg_catalog.regtype,
         true, 0),
        ('babylon_ref', 'h3_cell', 3, 'immediate_parent', 'pg_catalog.int8'::pg_catalog.regtype,
         false, 0),
        ('babylon_ref', 'h3_cell', 4, 'ancestor_r4', 'pg_catalog.int8'::pg_catalog.regtype,
         false, 0),
        ('babylon_ref', 'h3_cell', 5, 'ancestor_r5', 'pg_catalog.int8'::pg_catalog.regtype,
         false, 0),
        ('babylon_ref', 'h3_cell', 6, 'ancestor_r6', 'pg_catalog.int8'::pg_catalog.regtype,
         false, 0),
        ('babylon_ref', 'h3_cell', 7, 'ancestor_r7', 'pg_catalog.int8'::pg_catalog.regtype,
         false, 0),
        ('babylon_ref', 'h3_reference_cohort', 1, 'ref_digest',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_ref', 'h3_reference_cohort', 2, 'format_version',
         'pg_catalog.int2'::pg_catalog.regtype, true, 0),
        ('babylon_ref', 'h3_reference_cohort', 3, 'artifact_name',
         'pg_catalog.text'::pg_catalog.regtype, true, 'pg_catalog."default"'::pg_catalog.regcollation),
        ('babylon_ref', 'h3_reference_cohort', 4, 'artifact_manifest_version',
         'pg_catalog.text'::pg_catalog.regtype, true, 'pg_catalog."default"'::pg_catalog.regcollation),
        ('babylon_ref', 'h3_reference_cohort', 5, 'artifact_digest',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_ref', 'h3_reference_cohort', 6, 'source_digest',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_ref', 'h3_reference_cohort', 7, 'source_r5_digest',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_ref', 'h3_reference_cohort', 8, 'source_r7_digest',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_ref', 'h3_reference_cohort', 9, 'closure_digest',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_ref', 'h3_reference_cohort', 10, 'membership_digest',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_ref', 'h3_reference_cohort', 11, 'direct_cell_count',
         'pg_catalog.int8'::pg_catalog.regtype, true, 0),
        ('babylon_ref', 'h3_reference_cohort', 12, 'derived_ancestor_count',
         'pg_catalog.int8'::pg_catalog.regtype, true, 0),
        ('babylon_ref', 'h3_reference_cohort', 13, 'closure_cell_count',
         'pg_catalog.int8'::pg_catalog.regtype, true, 0),
        ('babylon_ref', 'h3_reference_membership', 1, 'ref_digest',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_ref', 'h3_reference_membership', 2, 'cell_id',
         'pg_catalog.int8'::pg_catalog.regtype, true, 0),
        ('babylon_ref', 'h3_reference_membership', 3, 'origin',
         'pg_catalog.int2'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'campaign', 1, 'campaign_id',
         'pg_catalog.uuid'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'campaign', 2, 'replay_layout_version',
         'pg_catalog.int2'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'campaign', 3, 'rng_layout_version',
         'pg_catalog.int2'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'campaign', 4, 'replay_session_id',
         'pg_catalog.text'::pg_catalog.regtype, true, 'pg_catalog."C"'::pg_catalog.regcollation),
        ('babylon_state', 'campaign', 5, 'rng_seed',
         'pg_catalog.int8'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'campaign', 6, 'defines_hash',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'campaign', 7, 'rules_hash',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'campaign', 8, 'ref_digest',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'schema_migration', 1, 'version',
         'pg_catalog.int8'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'schema_migration', 2, 'checksum',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_archive_dirty_receipt_row', 1, 'campaign_id',
         'pg_catalog.uuid'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_archive_dirty_receipt_row', 2, 'resolve_tick',
         'pg_catalog.int8'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_archive_dirty_receipt_row', 3, 'row_ordinal',
         'pg_catalog.int4'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_archive_dirty_receipt_row', 4, 'row_key',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_archive_dirty_receipt_row', 5, 'row_payload',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_boundary_flow_row', 1, 'campaign_id',
         'pg_catalog.uuid'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_boundary_flow_row', 2, 'resolve_tick',
         'pg_catalog.int8'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_boundary_flow_row', 3, 'row_ordinal',
         'pg_catalog.int4'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_boundary_flow_row', 4, 'row_key',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_boundary_flow_row', 5, 'row_payload',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_checkpoint_row', 1, 'campaign_id',
         'pg_catalog.uuid'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_checkpoint_row', 2, 'resolve_tick',
         'pg_catalog.int8'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_checkpoint_row', 3, 'row_ordinal',
         'pg_catalog.int4'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_checkpoint_row', 4, 'row_key',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_checkpoint_row', 5, 'row_payload',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_commit', 1, 'campaign_id',
         'pg_catalog.uuid'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_commit', 2, 'resolve_tick',
         'pg_catalog.int8'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_commit', 3, 'envelope_layout_version',
         'pg_catalog.int2'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_commit', 4, 'tick_content_hash',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_commit', 5, 'envelope_digest',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_conservation_row', 1, 'campaign_id',
         'pg_catalog.uuid'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_conservation_row', 2, 'resolve_tick',
         'pg_catalog.int8'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_conservation_row', 3, 'row_ordinal',
         'pg_catalog.int4'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_conservation_row', 4, 'row_key',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_conservation_row', 5, 'row_payload',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_event_row', 1, 'campaign_id',
         'pg_catalog.uuid'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_event_row', 2, 'resolve_tick',
         'pg_catalog.int8'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_event_row', 3, 'row_ordinal',
         'pg_catalog.int4'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_event_row', 4, 'row_key',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_event_row', 5, 'row_payload',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_graph_row', 1, 'campaign_id',
         'pg_catalog.uuid'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_graph_row', 2, 'resolve_tick',
         'pg_catalog.int8'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_graph_row', 3, 'row_ordinal',
         'pg_catalog.int4'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_graph_row', 4, 'row_key',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_graph_row', 5, 'row_payload',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_state_row', 1, 'campaign_id',
         'pg_catalog.uuid'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_state_row', 2, 'resolve_tick',
         'pg_catalog.int8'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_state_row', 3, 'row_ordinal',
         'pg_catalog.int4'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_state_row', 4, 'row_key',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_state_row', 5, 'row_payload',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_subsystem_row', 1, 'campaign_id',
         'pg_catalog.uuid'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_subsystem_row', 2, 'resolve_tick',
         'pg_catalog.int8'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_subsystem_row', 3, 'row_ordinal',
         'pg_catalog.int4'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_subsystem_row', 4, 'row_key',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0),
        ('babylon_state', 'tick_subsystem_row', 5, 'row_payload',
         'pg_catalog.bytea'::pg_catalog.regtype, true, 0)
),
relation_constraints AS MATERIALIZED (
    SELECT relation.nspname, relation.relname,
           constraint_row.conname::pg_catalog.text AS conname,
           constraint_row.contype, constraint_row.convalidated,
           constraint_row.condeferrable, constraint_row.condeferred,
           constraint_row.conkey, constraint_row.confrelid, constraint_row.confkey,
           constraint_row.confupdtype, constraint_row.confdeltype,
           constraint_row.confmatchtype
    FROM owned_relations AS relation
    JOIN pg_catalog.pg_constraint AS constraint_row
      ON constraint_row.conrelid = relation.oid
    ORDER BY relation.nspname, relation.relname, constraint_row.conname
    LIMIT 102
),
expected_constraints(relname, conname, contype) AS (
    VALUES
        ('campaign'::pg_catalog.text, 'campaign_defines_hash_length'::pg_catalog.text,
         'c'::pg_catalog."char"),
        ('campaign', 'campaign_pkey', 'p'::pg_catalog."char"),
        ('campaign', 'campaign_ref_digest_length', 'c'::pg_catalog."char"),
        ('campaign', 'campaign_reference_cohort_fkey', 'f'::pg_catalog."char"),
        ('campaign', 'campaign_replay_layout_v1', 'c'::pg_catalog."char"),
        ('campaign', 'campaign_replay_session_ascii_graphic', 'c'::pg_catalog."char"),
        ('campaign', 'campaign_replay_session_length', 'c'::pg_catalog."char"),
        ('campaign', 'campaign_rng_layout_v2', 'c'::pg_catalog."char"),
        ('campaign', 'campaign_rules_hash_length', 'c'::pg_catalog."char"),
        ('tick_commit', 'tick_commit_campaign_fkey', 'f'::pg_catalog."char"),
        ('tick_commit', 'tick_commit_content_hash_length', 'c'::pg_catalog."char"),
        ('tick_commit', 'tick_commit_envelope_digest_length', 'c'::pg_catalog."char"),
        ('tick_commit', 'tick_commit_envelope_layout_v1', 'c'::pg_catalog."char"),
        ('tick_commit', 'tick_commit_pkey', 'p'::pg_catalog."char"),
        ('tick_commit', 'tick_commit_resolve_tick_sql_range', 'c'::pg_catalog."char"),
        ('tick_archive_dirty_receipt_row',
         'tick_archive_dirty_receipt_row_campaign_tick_fkey', 'f'::pg_catalog."char"),
        ('tick_archive_dirty_receipt_row',
         'tick_archive_dirty_receipt_row_key_length', 'c'::pg_catalog."char"),
        ('tick_archive_dirty_receipt_row',
         'tick_archive_dirty_receipt_row_ordinal_range', 'c'::pg_catalog."char"),
        ('tick_archive_dirty_receipt_row',
         'tick_archive_dirty_receipt_row_payload_length', 'c'::pg_catalog."char"),
        ('tick_archive_dirty_receipt_row',
         'tick_archive_dirty_receipt_row_pkey', 'p'::pg_catalog."char"),
        ('tick_archive_dirty_receipt_row',
         'tick_archive_dirty_receipt_row_resolve_tick_sql_range', 'c'::pg_catalog."char"),
        ('tick_boundary_flow_row', 'tick_boundary_flow_row_campaign_tick_fkey',
         'f'::pg_catalog."char"),
        ('tick_boundary_flow_row', 'tick_boundary_flow_row_key_length',
         'c'::pg_catalog."char"),
        ('tick_boundary_flow_row', 'tick_boundary_flow_row_ordinal_range',
         'c'::pg_catalog."char"),
        ('tick_boundary_flow_row', 'tick_boundary_flow_row_payload_length',
         'c'::pg_catalog."char"),
        ('tick_boundary_flow_row', 'tick_boundary_flow_row_pkey', 'p'::pg_catalog."char"),
        ('tick_boundary_flow_row', 'tick_boundary_flow_row_resolve_tick_sql_range',
         'c'::pg_catalog."char"),
        ('tick_checkpoint_row', 'tick_checkpoint_row_campaign_tick_fkey',
         'f'::pg_catalog."char"),
        ('tick_checkpoint_row', 'tick_checkpoint_row_key_length', 'c'::pg_catalog."char"),
        ('tick_checkpoint_row', 'tick_checkpoint_row_ordinal_range',
         'c'::pg_catalog."char"),
        ('tick_checkpoint_row', 'tick_checkpoint_row_payload_length',
         'c'::pg_catalog."char"),
        ('tick_checkpoint_row', 'tick_checkpoint_row_pkey', 'p'::pg_catalog."char"),
        ('tick_checkpoint_row', 'tick_checkpoint_row_resolve_tick_sql_range',
         'c'::pg_catalog."char"),
        ('tick_conservation_row', 'tick_conservation_row_campaign_tick_fkey',
         'f'::pg_catalog."char"),
        ('tick_conservation_row', 'tick_conservation_row_key_length',
         'c'::pg_catalog."char"),
        ('tick_conservation_row', 'tick_conservation_row_ordinal_range',
         'c'::pg_catalog."char"),
        ('tick_conservation_row', 'tick_conservation_row_payload_length',
         'c'::pg_catalog."char"),
        ('tick_conservation_row', 'tick_conservation_row_pkey', 'p'::pg_catalog."char"),
        ('tick_conservation_row', 'tick_conservation_row_resolve_tick_sql_range',
         'c'::pg_catalog."char"),
        ('tick_event_row', 'tick_event_row_campaign_tick_fkey', 'f'::pg_catalog."char"),
        ('tick_event_row', 'tick_event_row_key_length', 'c'::pg_catalog."char"),
        ('tick_event_row', 'tick_event_row_ordinal_range', 'c'::pg_catalog."char"),
        ('tick_event_row', 'tick_event_row_payload_length', 'c'::pg_catalog."char"),
        ('tick_event_row', 'tick_event_row_pkey', 'p'::pg_catalog."char"),
        ('tick_event_row', 'tick_event_row_resolve_tick_sql_range', 'c'::pg_catalog."char"),
        ('tick_graph_row', 'tick_graph_row_campaign_tick_fkey', 'f'::pg_catalog."char"),
        ('tick_graph_row', 'tick_graph_row_key_length', 'c'::pg_catalog."char"),
        ('tick_graph_row', 'tick_graph_row_ordinal_range', 'c'::pg_catalog."char"),
        ('tick_graph_row', 'tick_graph_row_payload_length', 'c'::pg_catalog."char"),
        ('tick_graph_row', 'tick_graph_row_pkey', 'p'::pg_catalog."char"),
        ('tick_graph_row', 'tick_graph_row_resolve_tick_sql_range', 'c'::pg_catalog."char"),
        ('tick_state_row', 'tick_state_row_campaign_tick_fkey', 'f'::pg_catalog."char"),
        ('tick_state_row', 'tick_state_row_key_length', 'c'::pg_catalog."char"),
        ('tick_state_row', 'tick_state_row_ordinal_range', 'c'::pg_catalog."char"),
        ('tick_state_row', 'tick_state_row_payload_length', 'c'::pg_catalog."char"),
        ('tick_state_row', 'tick_state_row_pkey', 'p'::pg_catalog."char"),
        ('tick_state_row', 'tick_state_row_resolve_tick_sql_range', 'c'::pg_catalog."char"),
        ('tick_subsystem_row', 'tick_subsystem_row_campaign_tick_fkey',
         'f'::pg_catalog."char"),
        ('tick_subsystem_row', 'tick_subsystem_row_key_length', 'c'::pg_catalog."char"),
        ('tick_subsystem_row', 'tick_subsystem_row_ordinal_range',
         'c'::pg_catalog."char"),
        ('tick_subsystem_row', 'tick_subsystem_row_payload_length',
         'c'::pg_catalog."char"),
        ('tick_subsystem_row', 'tick_subsystem_row_pkey', 'p'::pg_catalog."char"),
        ('tick_subsystem_row', 'tick_subsystem_row_resolve_tick_sql_range',
         'c'::pg_catalog."char")
),
relation_indexes AS MATERIALIZED (
    SELECT parent.nspname, parent.relname AS parent_name,
           index_relation.relname AS index_name, index_row.indisprimary,
           index_row.indisunique, index_row.indisvalid, index_row.indisready,
           index_row.indpred, index_row.indexprs, index_row.indnkeyatts,
           index_row.indnatts,
           pg_catalog.pg_get_indexdef(index_relation.oid, 1, true) AS key_one,
           CASE WHEN index_row.indnkeyatts >= 2
                THEN pg_catalog.pg_get_indexdef(index_relation.oid, 2, true) END AS key_two,
           CASE WHEN index_row.indnkeyatts >= 3
                THEN pg_catalog.pg_get_indexdef(index_relation.oid, 3, true) END AS key_three
    FROM owned_relations AS parent
    JOIN pg_catalog.pg_index AS index_row ON index_row.indrelid = parent.oid
    JOIN pg_catalog.pg_class AS index_relation ON index_relation.oid = index_row.indexrelid
    ORDER BY parent.nspname, parent.relname, index_relation.relname
    LIMIT 17
),
expected_indexes(
    parent_name, index_name, indisprimary, indisunique, key_one, key_two, key_three
) AS (
    VALUES
        ('campaign'::pg_catalog.text, 'campaign_pkey'::pg_catalog.text,
         true, true, 'campaign_id'::pg_catalog.text, NULL::pg_catalog.text,
         NULL::pg_catalog.text),
        ('h3_cell', 'h3_cell_pkey', true, true, 'cell_id', NULL, NULL),
        ('h3_reference_cohort', 'h3_reference_cohort_artifact_identity',
         false, true, 'format_version', 'artifact_digest', NULL),
        ('h3_reference_cohort', 'h3_reference_cohort_pkey',
         true, true, 'ref_digest', NULL, NULL),
        ('h3_reference_membership', 'h3_reference_membership_cell_id_idx',
         false, false, 'cell_id', 'ref_digest', NULL),
        ('h3_reference_membership', 'h3_reference_membership_pkey',
         true, true, 'ref_digest', 'cell_id', NULL),
        ('schema_migration', 'schema_migration_pkey', true, true, 'version', NULL, NULL),
        ('tick_archive_dirty_receipt_row', 'tick_archive_dirty_receipt_row_pkey',
         true, true, 'campaign_id', 'resolve_tick', 'row_ordinal'),
        ('tick_boundary_flow_row', 'tick_boundary_flow_row_pkey',
         true, true, 'campaign_id', 'resolve_tick', 'row_ordinal'),
        ('tick_checkpoint_row', 'tick_checkpoint_row_pkey',
         true, true, 'campaign_id', 'resolve_tick', 'row_ordinal'),
        ('tick_commit', 'tick_commit_pkey',
         true, true, 'campaign_id', 'resolve_tick', NULL),
        ('tick_conservation_row', 'tick_conservation_row_pkey',
         true, true, 'campaign_id', 'resolve_tick', 'row_ordinal'),
        ('tick_event_row', 'tick_event_row_pkey',
         true, true, 'campaign_id', 'resolve_tick', 'row_ordinal'),
        ('tick_graph_row', 'tick_graph_row_pkey',
         true, true, 'campaign_id', 'resolve_tick', 'row_ordinal'),
        ('tick_state_row', 'tick_state_row_pkey',
         true, true, 'campaign_id', 'resolve_tick', 'row_ordinal'),
        ('tick_subsystem_row', 'tick_subsystem_row_pkey',
         true, true, 'campaign_id', 'resolve_tick', 'row_ordinal')
),
owned_classes AS MATERIALIZED (
    SELECT namespace.nspname, relation.relname, relation.relkind
    FROM pg_catalog.pg_class AS relation
    JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace
    WHERE namespace.nspname IN ('babylon_ref', 'babylon_state')
    ORDER BY namespace.nspname, relation.relname
    LIMIT 31
),
expected_classes(nspname, relname, relkind) AS (
    SELECT expected.nspname, expected.relname, 'r'::pg_catalog."char"
    FROM expected_relations AS expected
    UNION ALL
    SELECT relation.nspname, expected.index_name, 'i'::pg_catalog."char"
    FROM expected_indexes AS expected
    JOIN expected_relations AS relation ON relation.relname = expected.parent_name
),
allowed_types AS MATERIALIZED (
    SELECT type_row.oid, type_row.typarray
    FROM owned_relations AS relation
    JOIN pg_catalog.pg_type AS type_row ON type_row.oid = relation.reltype
)
SELECT
    (SELECT pg_catalog.count(*) = 3 FROM owned_schemas)
    AND (SELECT pg_catalog.bool_and(schema_row.nspowner = owner_row.owner_oid)
         FROM owned_schemas AS schema_row CROSS JOIN database_owner AS owner_row)
    AND NOT EXISTS (
        SELECT 1 FROM owned_schemas AS schema_row
        CROSS JOIN LATERAL pg_catalog.aclexplode(
            coalesce(
                schema_row.nspacl, pg_catalog.acldefault('n', schema_row.nspowner)
            )
        ) AS acl
        WHERE acl.grantee = 0 LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM intel_role AS intel CROSS JOIN owned_schemas AS schema_row
        WHERE pg_catalog.has_schema_privilege(intel.oid, schema_row.oid, 'USAGE')
           OR pg_catalog.has_schema_privilege(intel.oid, schema_row.oid, 'CREATE')
        LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 14 FROM owned_relations)
    AND NOT EXISTS (
        SELECT 1 FROM expected_relations AS expected
        FULL JOIN owned_relations AS actual
          ON actual.nspname = expected.nspname AND actual.relname = expected.relname
        WHERE expected.relname IS NULL OR actual.relname IS NULL LIMIT 1
    )
    AND (SELECT pg_catalog.bool_and(relation.relowner = owner_row.owner_oid)
         FROM owned_relations AS relation CROSS JOIN database_owner AS owner_row)
    AND NOT EXISTS (
        SELECT 1 FROM owned_relations AS relation
        CROSS JOIN LATERAL pg_catalog.aclexplode(
            coalesce(
                relation.relacl, pg_catalog.acldefault('r', relation.relowner)
            )
        ) AS acl
        WHERE acl.grantee = 0 LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM intel_role AS intel CROSS JOIN owned_relations AS relation
        WHERE pg_catalog.has_table_privilege(
            intel.oid, relation.oid,
            'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN'
        ) LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 78 FROM relation_columns)
    AND (SELECT pg_catalog.count(*) = 78 FROM expected_columns)
    AND NOT EXISTS (
        SELECT 1 FROM relation_columns AS actual
        FULL JOIN expected_columns AS expected
          ON expected.nspname = actual.nspname
         AND expected.relname = actual.relname
         AND expected.attnum = actual.attnum
        WHERE expected.attnum IS NULL OR actual.attnum IS NULL
           OR actual.attname IS DISTINCT FROM expected.attname
           OR actual.atttypid IS DISTINCT FROM expected.atttypid
           OR actual.attnotnull IS DISTINCT FROM expected.attnotnull
           OR actual.attcollation IS DISTINCT FROM expected.attcollation
           OR actual.attidentity <> '' OR actual.attgenerated <> ''
           OR actual.has_default
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM relation_columns AS column_row
        CROSS JOIN LATERAL pg_catalog.aclexplode(column_row.attacl) AS acl
        WHERE acl.grantee = 0 LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM intel_role AS intel
        CROSS JOIN owned_relations AS relation
        JOIN pg_catalog.pg_attribute AS attribute ON attribute.attrelid = relation.oid
        WHERE attribute.attnum > 0 AND NOT attribute.attisdropped
          AND (
              pg_catalog.has_column_privilege(
                  intel.oid, relation.oid, attribute.attnum, 'SELECT'
              )
              OR pg_catalog.has_column_privilege(
                  intel.oid, relation.oid, attribute.attnum, 'INSERT'
              )
              OR pg_catalog.has_column_privilege(
                  intel.oid, relation.oid, attribute.attnum, 'UPDATE'
              )
              OR pg_catalog.has_column_privilege(
                  intel.oid, relation.oid, attribute.attnum, 'REFERENCES'
              )
          )
        LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 101 FROM relation_constraints)
    AND (SELECT pg_catalog.bool_and(convalidated) FROM relation_constraints)
    AND NOT EXISTS (
        SELECT 1 FROM relation_constraints
        WHERE (contype = 'f' AND NOT (condeferrable AND condeferred))
           OR (contype <> 'f' AND (condeferrable OR condeferred))
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM expected_relations AS expected
        LEFT JOIN LATERAL (
            SELECT pg_catalog.count(*)::pg_catalog.int4 AS actual_count
            FROM relation_constraints AS constraint_row
            WHERE constraint_row.nspname = expected.nspname
              AND constraint_row.relname = expected.relname
        ) AS actual ON true
        WHERE actual.actual_count <> expected.constraint_count LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 63 FROM expected_constraints)
    AND NOT EXISTS (
        SELECT 1 FROM expected_constraints AS expected
        LEFT JOIN relation_constraints AS actual
          ON actual.relname = expected.relname AND actual.conname = expected.conname
        WHERE actual.conname IS NULL OR actual.contype IS DISTINCT FROM expected.contype
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM relation_constraints AS constraint_row
        WHERE constraint_row.contype = 'f'
          AND (
              constraint_row.confupdtype <> 'a'
              OR constraint_row.confdeltype <> 'a'
              OR constraint_row.confmatchtype <> 's'
          )
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM relation_constraints AS constraint_row
        WHERE constraint_row.contype = 'f'
          AND NOT (
              (constraint_row.relname = 'campaign'
               AND constraint_row.conname = 'campaign_reference_cohort_fkey'
               AND constraint_row.conkey = ARRAY[8]::pg_catalog.int2[]
               AND constraint_row.confrelid = 'babylon_ref.h3_reference_cohort'::pg_catalog.regclass
               AND constraint_row.confkey = ARRAY[1]::pg_catalog.int2[])
              OR (constraint_row.relname = 'tick_commit'
                  AND constraint_row.conname = 'tick_commit_campaign_fkey'
                  AND constraint_row.conkey = ARRAY[1]::pg_catalog.int2[]
                  AND constraint_row.confrelid = 'babylon_state.campaign'::pg_catalog.regclass
                  AND constraint_row.confkey = ARRAY[1]::pg_catalog.int2[])
              OR (constraint_row.relname LIKE 'tick\_%\_row' ESCAPE '\'
                  AND constraint_row.conname LIKE 'tick\_%\_row_campaign_tick_fkey' ESCAPE '\'
                  AND constraint_row.conkey = ARRAY[1, 2]::pg_catalog.int2[]
                  AND constraint_row.confrelid = 'babylon_state.tick_commit'::pg_catalog.regclass
                  AND constraint_row.confkey = ARRAY[1, 2]::pg_catalog.int2[])
              OR (constraint_row.relname = 'h3_cell'
                  AND constraint_row.confrelid = 'babylon_ref.h3_cell'::pg_catalog.regclass)
              OR (constraint_row.relname = 'h3_reference_membership'
                  AND constraint_row.conname = 'h3_reference_membership_cohort_fkey'
                  AND constraint_row.confrelid = 'babylon_ref.h3_reference_cohort'::pg_catalog.regclass
                  AND constraint_row.conkey = ARRAY[1]::pg_catalog.int2[]
                  AND constraint_row.confkey = ARRAY[1]::pg_catalog.int2[])
              OR (constraint_row.relname = 'h3_reference_membership'
                  AND constraint_row.conname = 'h3_reference_membership_cell_fkey'
                  AND constraint_row.confrelid = 'babylon_ref.h3_cell'::pg_catalog.regclass
                  AND constraint_row.conkey = ARRAY[2]::pg_catalog.int2[]
                  AND constraint_row.confkey = ARRAY[1]::pg_catalog.int2[])
          )
        LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 16 FROM relation_indexes)
    AND (SELECT pg_catalog.count(*) = 16 FROM expected_indexes)
    AND NOT EXISTS (
        SELECT 1 FROM relation_indexes AS actual
        FULL JOIN expected_indexes AS expected
          ON expected.parent_name = actual.parent_name
         AND expected.index_name = actual.index_name
        WHERE expected.index_name IS NULL OR actual.index_name IS NULL
           OR actual.indisprimary IS DISTINCT FROM expected.indisprimary
           OR actual.indisunique IS DISTINCT FROM expected.indisunique
           OR NOT actual.indisvalid OR NOT actual.indisready
           OR actual.indpred IS NOT NULL OR actual.indexprs IS NOT NULL
           OR actual.indnatts <> actual.indnkeyatts
           OR actual.indnkeyatts <> CASE
               WHEN expected.key_three IS NOT NULL THEN 3
               WHEN expected.key_two IS NOT NULL THEN 2
               ELSE 1
           END
           OR actual.key_one IS DISTINCT FROM expected.key_one
           OR actual.key_two IS DISTINCT FROM expected.key_two
           OR actual.key_three IS DISTINCT FROM expected.key_three
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM expected_relations AS expected
        LEFT JOIN LATERAL (
            SELECT pg_catalog.count(*)::pg_catalog.int4 AS actual_count
            FROM relation_indexes AS index_row
            WHERE index_row.nspname = expected.nspname
              AND index_row.parent_name = expected.relname
        ) AS actual ON true
        WHERE actual.actual_count <> expected.index_count LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 30 FROM owned_classes)
    AND (SELECT pg_catalog.count(*) = 30 FROM expected_classes)
    AND NOT EXISTS (
        SELECT 1 FROM owned_classes AS actual
        FULL JOIN expected_classes AS expected
          ON expected.nspname = actual.nspname
         AND expected.relname = actual.relname
         AND expected.relkind = actual.relkind
        WHERE expected.relname IS NULL OR actual.relname IS NULL LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM pg_catalog.pg_proc AS routine
        JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = routine.pronamespace
        WHERE namespace.nspname IN ('babylon_ref', 'babylon_state') LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM pg_catalog.pg_type AS type_row
        JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = type_row.typnamespace
        LEFT JOIN allowed_types AS allowed
          ON type_row.oid = allowed.oid OR type_row.oid = allowed.typarray
        WHERE namespace.nspname IN ('babylon_ref', 'babylon_state')
          AND allowed.oid IS NULL LIMIT 1
    ) AS epoch_shape_matches;
