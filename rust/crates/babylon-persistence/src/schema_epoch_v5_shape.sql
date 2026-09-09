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
        ('babylon_ref'::pg_catalog.text, 'county_h3_land_area'::pg_catalog.text, 6, 8, 2),
        ('babylon_ref', 'county_identity', 7, 10, 2),
        ('babylon_ref', 'county_place_h3_land_area', 9, 13, 2),
        ('babylon_ref', 'h3_cell', 7, 14, 1),
        ('babylon_ref', 'h3_land_fraction', 6, 8, 1),
        ('babylon_ref', 'h3_population_count', 5, 7, 1),
        ('babylon_ref', 'h3_reference_cohort', 13, 15, 2),
        ('babylon_ref', 'h3_reference_membership', 3, 6, 3),
        ('babylon_ref', 'h3_workplace_count', 5, 7, 1),
        ('babylon_ref', 'place_identity', 13, 15, 1),
        ('babylon_ref', 'reference_product', 8, 10, 1),
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
    LIMIT 23
),
relation_columns AS MATERIALIZED (
    SELECT relation.oid, relation.nspname, relation.relname, relation.relowner,
           attribute.attnum, attribute.attidentity, attribute.attgenerated,
           attribute.attacl, default_row.oid IS NOT NULL AS has_default
    FROM owned_relations AS relation
    JOIN pg_catalog.pg_attribute AS attribute ON attribute.attrelid = relation.oid
    LEFT JOIN pg_catalog.pg_attrdef AS default_row
      ON default_row.adrelid = attribute.attrelid
     AND default_row.adnum = attribute.attnum
    WHERE attribute.attnum > 0 AND NOT attribute.attisdropped
    ORDER BY relation.nspname, relation.relname, attribute.attnum
    LIMIT 138
),
relation_constraints AS MATERIALIZED (
    SELECT relation.nspname, relation.relname, constraint_row.contype,
           constraint_row.convalidated, constraint_row.condeferrable,
           constraint_row.condeferred, constraint_row.confupdtype,
           constraint_row.confdeltype, constraint_row.confmatchtype
    FROM owned_relations AS relation
    JOIN pg_catalog.pg_constraint AS constraint_row
      ON constraint_row.conrelid = relation.oid
    ORDER BY relation.nspname, relation.relname, constraint_row.conname
    LIMIT 180
),
relation_indexes AS MATERIALIZED (
    SELECT relation.nspname, relation.relname,
           index_row.indisvalid, index_row.indisready, index_row.indpred,
           index_row.indexprs, index_row.indnatts, index_row.indnkeyatts
    FROM owned_relations AS relation
    JOIN pg_catalog.pg_index AS index_row ON index_row.indrelid = relation.oid
    ORDER BY relation.nspname, relation.relname, index_row.indexrelid
    LIMIT 29
),
owned_classes AS MATERIALIZED (
    SELECT relation.oid, namespace.nspname, relation.relname, relation.relkind,
           relation.relowner, relation.relacl
    FROM pg_catalog.pg_class AS relation
    JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace
    WHERE namespace.nspname IN ('babylon_ref', 'babylon_state')
    ORDER BY namespace.nspname, relation.relname, relation.relkind
    LIMIT 51
),
allowed_types AS MATERIALIZED (
    SELECT type_row.oid, type_row.typarray
    FROM owned_relations AS relation
    JOIN pg_catalog.pg_type AS type_row ON type_row.oid = relation.reltype
),
intel_role AS MATERIALIZED (
    SELECT role_row.oid
    FROM pg_catalog.pg_roles AS role_row
    WHERE role_row.rolname = 'babylon_intel'
    LIMIT 2
)
SELECT
    (SELECT pg_catalog.count(*) = 3 FROM owned_schemas)
    AND (SELECT pg_catalog.bool_and(schema_row.nspowner = owner_row.owner_oid)
         FROM owned_schemas AS schema_row CROSS JOIN database_owner AS owner_row)
    AND NOT EXISTS (
        SELECT 1
        FROM owned_schemas AS schema_row
        CROSS JOIN LATERAL pg_catalog.aclexplode(
            coalesce(schema_row.nspacl, pg_catalog.acldefault('n', schema_row.nspowner))
        ) AS acl
        WHERE acl.grantee = 0
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM intel_role AS intel CROSS JOIN owned_schemas AS schema_row
        WHERE pg_catalog.has_schema_privilege(intel.oid, schema_row.oid, 'USAGE')
           OR pg_catalog.has_schema_privilege(intel.oid, schema_row.oid, 'CREATE')
        LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 22 FROM expected_relations)
    AND (SELECT pg_catalog.count(*) = 22 FROM owned_relations)
    AND NOT EXISTS (
        SELECT 1 FROM owned_relations AS actual
        FULL JOIN expected_relations AS expected
          ON expected.nspname = actual.nspname AND expected.relname = actual.relname
        WHERE expected.relname IS NULL OR actual.relname IS NULL
        LIMIT 1
    )
    AND (SELECT pg_catalog.bool_and(relation.relowner = owner_row.owner_oid)
         FROM owned_relations AS relation CROSS JOIN database_owner AS owner_row)
    AND NOT EXISTS (
        SELECT 1
        FROM owned_relations AS relation
        CROSS JOIN LATERAL pg_catalog.aclexplode(
            coalesce(relation.relacl, pg_catalog.acldefault('r', relation.relowner))
        ) AS acl
        WHERE acl.grantee = 0
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM intel_role AS intel CROSS JOIN owned_relations AS relation
        WHERE pg_catalog.has_table_privilege(intel.oid, relation.oid, 'SELECT')
           OR pg_catalog.has_table_privilege(intel.oid, relation.oid, 'INSERT')
           OR pg_catalog.has_table_privilege(intel.oid, relation.oid, 'UPDATE')
           OR pg_catalog.has_table_privilege(intel.oid, relation.oid, 'DELETE')
           OR pg_catalog.has_table_privilege(intel.oid, relation.oid, 'TRUNCATE')
           OR pg_catalog.has_table_privilege(intel.oid, relation.oid, 'REFERENCES')
           OR pg_catalog.has_table_privilege(intel.oid, relation.oid, 'TRIGGER')
        LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 137 FROM relation_columns)
    AND NOT EXISTS (
        SELECT 1 FROM relation_columns
        WHERE attidentity <> '' OR attgenerated <> '' OR has_default
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM relation_columns AS column_row
        CROSS JOIN LATERAL pg_catalog.aclexplode(column_row.attacl) AS acl
        WHERE acl.grantee = 0
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1
        FROM intel_role AS intel
        CROSS JOIN relation_columns AS column_row
        WHERE pg_catalog.has_column_privilege(
                  intel.oid, column_row.oid, column_row.attnum, 'SELECT'
              )
           OR pg_catalog.has_column_privilege(
                  intel.oid, column_row.oid, column_row.attnum, 'INSERT'
              )
           OR pg_catalog.has_column_privilege(
                  intel.oid, column_row.oid, column_row.attnum, 'UPDATE'
              )
           OR pg_catalog.has_column_privilege(
                  intel.oid, column_row.oid, column_row.attnum, 'REFERENCES'
              )
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM expected_relations AS expected
        LEFT JOIN LATERAL (
            SELECT pg_catalog.count(*)::pg_catalog.int4 AS actual_count
            FROM relation_columns AS column_row
            WHERE column_row.nspname = expected.nspname
              AND column_row.relname = expected.relname
        ) AS actual ON true
        WHERE actual.actual_count <> expected.column_count
        LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 179 FROM relation_constraints)
    AND (SELECT pg_catalog.bool_and(convalidated) FROM relation_constraints)
    AND NOT EXISTS (
        SELECT 1 FROM relation_constraints
        WHERE (contype = 'f' AND NOT (condeferrable AND condeferred))
           OR (contype <> 'f' AND (condeferrable OR condeferred))
           OR (contype = 'f' AND (
               confupdtype <> 'a' OR confdeltype <> 'a' OR confmatchtype <> 's'
           ))
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
        WHERE actual.actual_count <> expected.constraint_count
        LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 28 FROM relation_indexes)
    AND NOT EXISTS (
        SELECT 1 FROM relation_indexes
        WHERE NOT indisvalid OR NOT indisready OR indpred IS NOT NULL OR indexprs IS NOT NULL
           OR indnatts <> indnkeyatts
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM expected_relations AS expected
        LEFT JOIN LATERAL (
            SELECT pg_catalog.count(*)::pg_catalog.int4 AS actual_count
            FROM relation_indexes AS index_row
            WHERE index_row.nspname = expected.nspname
              AND index_row.relname = expected.relname
        ) AS actual ON true
        WHERE actual.actual_count <> expected.index_count
        LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 50 FROM owned_classes)
    AND NOT EXISTS (
        SELECT 1 FROM owned_classes AS class_row
        CROSS JOIN database_owner AS owner_row
        WHERE class_row.relkind NOT IN ('r', 'i')
           OR class_row.relowner <> owner_row.owner_oid
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM pg_catalog.pg_proc AS routine
        JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = routine.pronamespace
        WHERE namespace.nspname IN ('babylon_ref', 'babylon_state')
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM pg_catalog.pg_type AS type_row
        JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = type_row.typnamespace
        LEFT JOIN allowed_types AS allowed
          ON type_row.oid = allowed.oid OR type_row.oid = allowed.typarray
        WHERE namespace.nspname IN ('babylon_ref', 'babylon_state')
          AND allowed.oid IS NULL
        LIMIT 1
    ) AS epoch_shape_matches;
