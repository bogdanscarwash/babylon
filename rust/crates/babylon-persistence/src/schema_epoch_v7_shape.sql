WITH expected_relations(relation_name) AS (
    VALUES
        ('dynamic_hex_state'::pg_catalog.text),
        ('hex_activity'),
        ('hex_cell'),
        ('hex_latest'),
        ('hex_map'),
        ('hex_r8_linear_features_reference'),
        ('hex_r8_reference'),
        ('hex_spatial_map'),
        ('hex_state'),
        ('hex_substrate'),
        ('hex_terrain_state'),
        ('immutable_reference_lodes_od_matrix'),
        ('infrastructure_link_state'),
        ('org_snapshot'),
        ('tick_event')
),
named_relations AS MATERIALIZED (
    SELECT relation.oid, relation.relname, relation.relkind
    FROM expected_relations AS expected
    JOIN pg_catalog.pg_class AS relation
      ON relation.oid = pg_catalog.to_regclass(
          pg_catalog.format('public.%I', expected.relation_name)
      )
    JOIN pg_catalog.pg_namespace AS namespace
      ON namespace.oid = relation.relnamespace
     AND namespace.nspname = 'public'
),
present_relations AS MATERIALIZED (
    SELECT relation.oid, relation.relname
    FROM named_relations AS relation
    WHERE relation.relkind IN ('r', 'p')
),
expected_views(view_name) AS (
    VALUES
        ('v_county_value_aggregate'::pg_catalog.text),
        ('v_hex_aid'),
        ('v_hex_economic'),
        ('v_hex_heat'),
        ('v_hex_intel'),
        ('v_hex_mobilize'),
        ('v_hex_state_asof'),
        ('v_national_value_aggregate'),
        ('v_state_value_aggregate'),
        ('view_runtime_trace_emission')
),
named_views AS MATERIALIZED (
    SELECT relation.oid, relation.relname, relation.relkind
    FROM expected_views AS expected
    JOIN pg_catalog.pg_class AS relation
      ON relation.oid = pg_catalog.to_regclass(
          pg_catalog.format('public.%I', expected.view_name)
      )
    JOIN pg_catalog.pg_namespace AS namespace
      ON namespace.oid = relation.relnamespace
     AND namespace.nspname = 'public'
),
present_views AS MATERIALIZED (
    SELECT relation.oid, relation.relname
    FROM named_views AS relation
    WHERE relation.relkind = 'v'
),
expected_cell_id_columns(view_name) AS (
    VALUES
        ('v_hex_aid'::pg_catalog.text),
        ('v_hex_economic'),
        ('v_hex_heat'),
        ('v_hex_intel'),
        ('v_hex_mobilize'),
        ('v_hex_state_asof')
),
canonical_identity_outputs AS MATERIALIZED (
    SELECT view_row.relname, attribute.attname, attribute.atttypid
    FROM expected_cell_id_columns AS expected
    JOIN present_views AS view_row ON view_row.relname = expected.view_name
    JOIN pg_catalog.pg_attribute AS attribute
      ON attribute.attrelid = view_row.oid
     AND attribute.attname = 'cell_id'
     AND attribute.attnum > 0
     AND NOT attribute.attisdropped
),
legacy_identity_columns(relation_name, column_name) AS (
    VALUES
        ('dynamic_hex_state'::pg_catalog.text, 'h3_index'::pg_catalog.text),
        ('hex_activity', 'h3_index'),
        ('hex_cell', 'h3_index'),
        ('hex_cell', 'res5_parent'),
        ('hex_cell', 'res6_parent'),
        ('hex_latest', 'h3_index'),
        ('hex_map', 'h3_index'),
        ('hex_r8_linear_features_reference', 'h3_index'),
        ('hex_r8_reference', 'h3_index'),
        ('hex_r8_reference', 'parent_h3'),
        ('hex_spatial_map', 'h3_index'),
        ('hex_state', 'h3_index'),
        ('hex_substrate', 'h3_index'),
        ('hex_substrate', 'r7_parent'),
        ('hex_terrain_state', 'h3_index'),
        ('immutable_reference_lodes_od_matrix', 'home_hex'),
        ('immutable_reference_lodes_od_matrix', 'workplace_dest'),
        ('infrastructure_link_state', 'source_h3'),
        ('infrastructure_link_state', 'target_h3'),
        ('org_snapshot', 'home_hex'),
        ('tick_event', 'h3_index')
),
legacy_identity_dependencies AS MATERIALIZED (
    SELECT DISTINCT view_row.relname, source_relation.relname, source_column.attname
    FROM present_views AS view_row
    JOIN pg_catalog.pg_rewrite AS rewrite_row
      ON rewrite_row.ev_class = view_row.oid
    JOIN pg_catalog.pg_depend AS dependency
      ON dependency.classid = 'pg_catalog.pg_rewrite'::pg_catalog.regclass
     AND dependency.objid = rewrite_row.oid
     AND dependency.refclassid = 'pg_catalog.pg_class'::pg_catalog.regclass
     AND dependency.refobjsubid > 0
    JOIN pg_catalog.pg_class AS source_relation
      ON source_relation.oid = dependency.refobjid
    JOIN pg_catalog.pg_namespace AS source_namespace
      ON source_namespace.oid = source_relation.relnamespace
     AND source_namespace.nspname = 'public'
    JOIN pg_catalog.pg_attribute AS source_column
      ON source_column.attrelid = source_relation.oid
     AND source_column.attnum = dependency.refobjsubid
     AND NOT source_column.attisdropped
    JOIN legacy_identity_columns AS legacy
      ON legacy.relation_name = source_relation.relname
     AND legacy.column_name = source_column.attname
),
expected_referenced_columns(view_name, relation_name, column_name) AS (
    VALUES
        ('v_hex_aid'::pg_catalog.text, 'hex_latest'::pg_catalog.text, 'cell_id'::pg_catalog.text),
        ('v_hex_economic', 'hex_latest', 'cell_id'),
        ('v_hex_heat', 'hex_latest', 'cell_id'),
        ('v_hex_intel', 'hex_latest', 'cell_id'),
        ('v_hex_mobilize', 'hex_latest', 'cell_id'),
        ('v_hex_state_asof', 'dynamic_hex_state', 'cell_id'),
        ('v_hex_state_asof', 'hex_spatial_map', 'cell_id'),
        ('v_county_value_aggregate', 'dynamic_hex_state', 'cell_id'),
        ('v_county_value_aggregate', 'hex_spatial_map', 'cell_id'),
        ('v_state_value_aggregate', 'dynamic_hex_state', 'cell_id'),
        ('v_state_value_aggregate', 'hex_spatial_map', 'cell_id'),
        ('v_national_value_aggregate', 'dynamic_hex_state', 'cell_id'),
        ('view_runtime_trace_emission', 'dynamic_hex_state', 'cell_id'),
        ('view_runtime_trace_emission', 'hex_spatial_map', 'cell_id')
),
referenced_columns AS MATERIALIZED (
    SELECT DISTINCT view_row.relname AS view_name,
           source_relation.relname AS relation_name,
           source_column.attname AS column_name
    FROM present_views AS view_row
    JOIN pg_catalog.pg_rewrite AS rewrite_row
      ON rewrite_row.ev_class = view_row.oid
    JOIN pg_catalog.pg_depend AS dependency
      ON dependency.classid = 'pg_catalog.pg_rewrite'::pg_catalog.regclass
     AND dependency.objid = rewrite_row.oid
     AND dependency.refclassid = 'pg_catalog.pg_class'::pg_catalog.regclass
     AND dependency.refobjsubid > 0
    JOIN pg_catalog.pg_class AS source_relation
      ON source_relation.oid = dependency.refobjid
    JOIN pg_catalog.pg_namespace AS source_namespace
      ON source_namespace.oid = source_relation.relnamespace
     AND source_namespace.nspname = 'public'
    JOIN pg_catalog.pg_attribute AS source_column
      ON source_column.attrelid = source_relation.oid
     AND source_column.attnum = dependency.refobjsubid
     AND source_column.attname = 'cell_id'
     AND NOT source_column.attisdropped
)
SELECT
    (SELECT pg_catalog.count(*) = 15 FROM expected_relations)
    AND (SELECT pg_catalog.count(*) = 10 FROM expected_views)
    AND (SELECT pg_catalog.count(*) = 6 FROM expected_cell_id_columns)
    AND (SELECT pg_catalog.count(*) = 14 FROM expected_referenced_columns)
    AND (SELECT pg_catalog.count(*) IN (0, 15) FROM named_relations)
    AND (
        (SELECT pg_catalog.count(*) FROM named_relations)
        = (SELECT pg_catalog.count(*) FROM present_relations)
    )
    AND (SELECT pg_catalog.count(*) IN (0, 10) FROM named_views)
    AND (
        (SELECT pg_catalog.count(*) FROM named_views)
        = (SELECT pg_catalog.count(*) FROM present_views)
    )
    AND (SELECT pg_catalog.count(*) IN (0, 10) FROM present_views)
    AND (SELECT pg_catalog.count(*) IN (0, 15) FROM present_relations)
    AND (
        (
            (SELECT pg_catalog.count(*) = 0 FROM present_relations)
            AND (SELECT pg_catalog.count(*) = 0 FROM present_views)
            AND (SELECT pg_catalog.count(*) = 0 FROM canonical_identity_outputs)
            AND (SELECT pg_catalog.count(*) = 0 FROM legacy_identity_dependencies)
            AND (SELECT pg_catalog.count(*) = 0 FROM referenced_columns)
        )
        OR
        (
            (SELECT pg_catalog.count(*) = 15 FROM present_relations)
            AND (SELECT pg_catalog.count(*) = 10 FROM present_views)
            AND (SELECT pg_catalog.count(*) = 6 FROM canonical_identity_outputs)
            AND NOT EXISTS (
                SELECT 1 FROM canonical_identity_outputs
                WHERE atttypid <> 'pg_catalog.int8'::pg_catalog.regtype
            )
            AND NOT EXISTS (
                SELECT 1
                FROM expected_cell_id_columns AS expected
                JOIN present_views AS view_row ON view_row.relname = expected.view_name
                JOIN pg_catalog.pg_attribute AS attribute
                  ON attribute.attrelid = view_row.oid
                 AND attribute.attname = 'h3_index'
                 AND attribute.attnum > 0
                 AND NOT attribute.attisdropped
            )
            AND (SELECT pg_catalog.count(*) = 0 FROM legacy_identity_dependencies)
            AND NOT EXISTS (
                SELECT 1
                FROM expected_referenced_columns AS expected
                FULL JOIN referenced_columns AS actual
                  ON actual.view_name = expected.view_name
                 AND actual.relation_name = expected.relation_name
                 AND actual.column_name = expected.column_name
                WHERE expected.view_name IS NULL OR actual.view_name IS NULL
            )
        )
    );
