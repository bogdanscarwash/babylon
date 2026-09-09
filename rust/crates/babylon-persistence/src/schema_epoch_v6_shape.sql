WITH expected(relation_name, column_name, token) AS (
    VALUES
        ('dynamic_hex_state'::pg_catalog.text, 'cell_id'::pg_catalog.text, 'dyn_cell'::pg_catalog.text),
        ('hex_activity', 'cell_id', 'activity_cell'),
        ('hex_cell', 'cell_id', 'hex_cell'),
        ('hex_cell', 'ancestor_r5', 'hex_r5'),
        ('hex_cell', 'ancestor_r6', 'hex_r6'),
        ('hex_latest', 'cell_id', 'latest_cell'),
        ('hex_map', 'cell_id', 'map_cell'),
        ('hex_r8_linear_features_reference', 'cell_id', 'r8_feature_cell'),
        ('hex_r8_reference', 'cell_id', 'r8_cell'),
        ('hex_r8_reference', 'parent_cell_id', 'r8_parent'),
        ('hex_spatial_map', 'cell_id', 'spatial_cell'),
        ('hex_state', 'cell_id', 'state_cell'),
        ('hex_substrate', 'cell_id', 'substrate_cell'),
        ('hex_substrate', 'ancestor_r7', 'substrate_r7'),
        ('hex_terrain_state', 'cell_id', 'terrain_cell'),
        ('immutable_reference_lodes_od_matrix', 'home_cell_id', 'lodes_home'),
        ('immutable_reference_lodes_od_matrix', 'workplace_cell_id', 'lodes_work'),
        ('infrastructure_link_state', 'source_cell_id', 'infra_source'),
        ('infrastructure_link_state', 'target_cell_id', 'infra_target'),
        ('org_snapshot', 'home_cell_id', 'org_home'),
        ('tick_event', 'cell_id', 'event_cell')
),
expected_relations AS (
    SELECT DISTINCT relation_name
    FROM expected
),
present_relations AS (
    SELECT relation.oid, relation.relname
    FROM expected_relations AS expected_relation
    JOIN pg_catalog.pg_class AS relation
      ON relation.oid = pg_catalog.to_regclass(
          pg_catalog.format('public.%I', expected_relation.relation_name)
      )
    JOIN pg_catalog.pg_namespace AS namespace
      ON namespace.oid = relation.relnamespace
     AND namespace.nspname = 'public'
),
shadow_columns AS (
    SELECT expected.relation_name, expected.column_name, expected.token,
           relation.oid AS relation_oid, attribute.attnum,
           attribute.atttypid, attribute.attnotnull, attribute.atthasdef,
           attribute.attidentity, attribute.attgenerated, attribute.attacl
    FROM expected
    JOIN present_relations AS relation
      ON relation.relname = expected.relation_name
    JOIN pg_catalog.pg_attribute AS attribute
      ON attribute.attrelid = relation.oid
     AND attribute.attname = expected.column_name
     AND attribute.attnum > 0
     AND NOT attribute.attisdropped
),
positive_checks AS (
    SELECT shadow.relation_name, shadow.column_name
    FROM shadow_columns AS shadow
    JOIN pg_catalog.pg_constraint AS constraint_row
      ON constraint_row.conrelid = shadow.relation_oid
     AND constraint_row.conname = 'ck_h3s_' || shadow.token || '_pos'
     AND constraint_row.contype = 'c'
     AND constraint_row.convalidated
     AND NOT constraint_row.connoinherit
),
cell_foreign_keys AS (
    SELECT shadow.relation_name, shadow.column_name
    FROM shadow_columns AS shadow
    JOIN pg_catalog.pg_constraint AS constraint_row
      ON constraint_row.conrelid = shadow.relation_oid
     AND constraint_row.conname = 'fk_h3s_' || shadow.token
     AND constraint_row.contype = 'f'
     AND constraint_row.confrelid = 'babylon_ref.h3_cell'::pg_catalog.regclass
     AND constraint_row.conkey = ARRAY[shadow.attnum]::pg_catalog.int2[]
     AND constraint_row.confkey = ARRAY[
         (
             SELECT attribute.attnum
             FROM pg_catalog.pg_attribute AS attribute
             WHERE attribute.attrelid = 'babylon_ref.h3_cell'::pg_catalog.regclass
               AND attribute.attname = 'cell_id'
               AND attribute.attnum > 0
               AND NOT attribute.attisdropped
         )
     ]::pg_catalog.int2[]
     AND constraint_row.convalidated
     AND constraint_row.condeferrable
     AND constraint_row.condeferred
     AND constraint_row.confmatchtype = 's'
     AND constraint_row.confupdtype = 'a'
     AND constraint_row.confdeltype = 'a'
),
shadow_indexes AS (
    SELECT shadow.relation_name, shadow.column_name
    FROM shadow_columns AS shadow
    JOIN pg_catalog.pg_class AS index_class
      ON index_class.oid = pg_catalog.to_regclass(
          pg_catalog.format('public.%I', 'ix_h3s_' || shadow.token)
      )
    JOIN pg_catalog.pg_index AS index_row
      ON index_row.indexrelid = index_class.oid
     AND index_row.indrelid = shadow.relation_oid
     AND index_row.indisvalid
     AND index_row.indisready
     AND NOT index_row.indisunique
     AND NOT index_row.indisprimary
     AND index_row.indnkeyatts = 1
     AND index_row.indnatts = 1
     AND index_row.indkey[0] = shadow.attnum
     AND index_row.indexprs IS NULL
     AND index_row.indpred IS NOT NULL
),
user_triggers AS (
    SELECT trigger_row.oid
    FROM present_relations AS relation
    JOIN pg_catalog.pg_trigger AS trigger_row
      ON trigger_row.tgrelid = relation.oid
     AND NOT trigger_row.tgisinternal
)
SELECT
    (SELECT pg_catalog.count(*) = 21 FROM expected)
    AND (SELECT pg_catalog.count(*) IN (0, 15) FROM present_relations)
    AND (
        (
            (SELECT pg_catalog.count(*) = 0 FROM present_relations)
            AND (SELECT pg_catalog.count(*) = 0 FROM shadow_columns)
            AND (SELECT pg_catalog.count(*) = 0 FROM positive_checks)
            AND (SELECT pg_catalog.count(*) = 0 FROM cell_foreign_keys)
            AND (SELECT pg_catalog.count(*) = 0 FROM shadow_indexes)
        )
        OR
        (
            (SELECT pg_catalog.count(*) = 15 FROM present_relations)
            AND (SELECT pg_catalog.count(*) = 21 FROM shadow_columns)
            AND NOT EXISTS (
                SELECT 1
                FROM shadow_columns
                WHERE atttypid <> 'pg_catalog.int8'::pg_catalog.regtype
                   OR attnotnull
                   OR atthasdef
                   OR attidentity <> ''
                   OR attgenerated <> ''
                   OR attacl IS NOT NULL
            )
            AND (SELECT pg_catalog.count(*) = 21 FROM positive_checks)
            AND (SELECT pg_catalog.count(*) = 21 FROM cell_foreign_keys)
            AND (SELECT pg_catalog.count(*) = 21 FROM shadow_indexes)
        )
    )
    AND NOT EXISTS (SELECT 1 FROM user_triggers);
