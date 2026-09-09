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
owned_relations AS MATERIALIZED (
    SELECT relation.oid, namespace.nspname, relation.relname, relation.relowner,
           relation.relacl, relation.reltype
    FROM pg_catalog.pg_class AS relation
    JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace
    WHERE (
        (namespace.nspname = 'babylon_ref' AND relation.relname = 'h3_cell')
        OR (namespace.nspname = 'babylon_state' AND relation.relname = 'schema_migration')
    )
      AND relation.relkind = 'r'
      AND relation.relpersistence = 'p'
    ORDER BY namespace.nspname, relation.relname
    LIMIT 3
),
ledger AS MATERIALIZED (
    SELECT relation.oid
    FROM owned_relations AS relation
    WHERE relation.nspname = 'babylon_state' AND relation.relname = 'schema_migration'
    LIMIT 2
),
h3_cell AS MATERIALIZED (
    SELECT relation.oid
    FROM owned_relations AS relation
    WHERE relation.nspname = 'babylon_ref' AND relation.relname = 'h3_cell'
    LIMIT 2
),
intel_role AS MATERIALIZED (
    SELECT role_row.oid
    FROM pg_catalog.pg_roles AS role_row
    WHERE role_row.rolname = 'babylon_intel'
    LIMIT 2
),
relation_columns AS MATERIALIZED (
    SELECT relation.relname, attribute.attnum, attribute.attname, attribute.atttypid,
           attribute.attnotnull, attribute.attidentity, attribute.attgenerated,
           attribute.attacl, default_row.oid IS NOT NULL AS has_default
    FROM owned_relations AS relation
    JOIN pg_catalog.pg_attribute AS attribute ON attribute.attrelid = relation.oid
    LEFT JOIN pg_catalog.pg_attrdef AS default_row
      ON default_row.adrelid = attribute.attrelid
     AND default_row.adnum = attribute.attnum
    WHERE attribute.attnum > 0 AND NOT attribute.attisdropped
    ORDER BY relation.relname, attribute.attnum
    LIMIT 10
),
ledger_constraints AS MATERIALIZED (
    SELECT constraint_row.contype,
           pg_catalog.pg_get_constraintdef(constraint_row.oid, true) AS definition
    FROM ledger
    JOIN pg_catalog.pg_constraint AS constraint_row
      ON constraint_row.conrelid = ledger.oid
    ORDER BY constraint_row.contype, constraint_row.conname
    LIMIT 4
),
h3_constraints AS MATERIALIZED (
    SELECT constraint_row.conname::pg_catalog.text AS conname,
           constraint_row.contype, constraint_row.convalidated,
           constraint_row.condeferrable, constraint_row.condeferred,
           constraint_row.confrelid
    FROM h3_cell
    JOIN pg_catalog.pg_constraint AS constraint_row
      ON constraint_row.conrelid = h3_cell.oid
    ORDER BY constraint_row.conname
    LIMIT 15
),
relation_indexes AS MATERIALIZED (
    SELECT parent.relname AS parent_name, index_relation.relname AS index_name,
           index_row.indisprimary, index_row.indisunique, index_row.indisvalid,
           index_row.indisready, index_row.indpred, index_row.indexprs
    FROM owned_relations AS parent
    JOIN pg_catalog.pg_index AS index_row ON index_row.indrelid = parent.oid
    JOIN pg_catalog.pg_class AS index_relation ON index_relation.oid = index_row.indexrelid
    ORDER BY parent.relname, index_relation.relname
    LIMIT 3
),
allowed_types AS MATERIALIZED (
    SELECT type_row.oid, type_row.typarray
    FROM owned_relations AS relation
    JOIN pg_catalog.pg_type AS type_row ON type_row.oid = relation.reltype
),
owned_classes AS MATERIALIZED (
    SELECT namespace.nspname, relation.relname, relation.relkind
    FROM pg_catalog.pg_class AS relation
    JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace
    WHERE namespace.nspname IN ('babylon_ref', 'babylon_state')
    ORDER BY namespace.nspname, relation.relname
    LIMIT 5
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
        SELECT 1
        FROM intel_role AS intel
        CROSS JOIN owned_schemas AS schema_row
        WHERE pg_catalog.has_schema_privilege(intel.oid, schema_row.oid, 'USAGE')
           OR pg_catalog.has_schema_privilege(intel.oid, schema_row.oid, 'CREATE')
        LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 2 FROM owned_relations)
    AND (SELECT pg_catalog.count(*) = 1 FROM ledger)
    AND (SELECT pg_catalog.count(*) = 1 FROM h3_cell)
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
        SELECT 1
        FROM intel_role AS intel
        CROSS JOIN owned_relations AS relation
        WHERE pg_catalog.has_table_privilege(intel.oid, relation.oid, 'SELECT')
           OR pg_catalog.has_table_privilege(intel.oid, relation.oid, 'INSERT')
           OR pg_catalog.has_table_privilege(intel.oid, relation.oid, 'UPDATE')
           OR pg_catalog.has_table_privilege(intel.oid, relation.oid, 'DELETE')
           OR pg_catalog.has_table_privilege(intel.oid, relation.oid, 'TRUNCATE')
           OR pg_catalog.has_table_privilege(intel.oid, relation.oid, 'REFERENCES')
           OR pg_catalog.has_table_privilege(intel.oid, relation.oid, 'TRIGGER')
           OR pg_catalog.has_table_privilege(intel.oid, relation.oid, 'MAINTAIN')
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1
        FROM relation_columns AS column_row
        CROSS JOIN LATERAL pg_catalog.aclexplode(column_row.attacl) AS acl
        WHERE acl.grantee = 0
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1
        FROM intel_role AS intel
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
    AND (SELECT pg_catalog.count(*) = 9 FROM relation_columns)
    AND EXISTS (
        SELECT 1 FROM relation_columns
        WHERE relname = 'schema_migration' AND attnum = 1 AND attname = 'version'
          AND atttypid = 'pg_catalog.int8'::pg_catalog.regtype
          AND attnotnull AND attidentity = '' AND attgenerated = '' AND NOT has_default
    )
    AND EXISTS (
        SELECT 1 FROM relation_columns
        WHERE relname = 'schema_migration' AND attnum = 2 AND attname = 'checksum'
          AND atttypid = 'pg_catalog.bytea'::pg_catalog.regtype
          AND attnotnull AND attidentity = '' AND attgenerated = '' AND NOT has_default
    )
    AND EXISTS (
        SELECT 1 FROM relation_columns
        WHERE relname = 'h3_cell' AND attnum = 1 AND attname = 'cell_id'
          AND atttypid = 'pg_catalog.int8'::pg_catalog.regtype
          AND attnotnull AND attidentity = '' AND attgenerated = '' AND NOT has_default
    )
    AND EXISTS (
        SELECT 1 FROM relation_columns
        WHERE relname = 'h3_cell' AND attnum = 2 AND attname = 'resolution'
          AND atttypid = 'pg_catalog.int2'::pg_catalog.regtype
          AND attnotnull AND attidentity = '' AND attgenerated = '' AND NOT has_default
    )
    AND NOT EXISTS (
        SELECT 1 FROM relation_columns
        WHERE relname = 'h3_cell' AND attnum BETWEEN 3 AND 7
          AND NOT (
              attname = (ARRAY[
                  'immediate_parent', 'ancestor_r4', 'ancestor_r5',
                  'ancestor_r6', 'ancestor_r7'
              ]::pg_catalog.text[])[attnum - 2]
              AND atttypid = 'pg_catalog.int8'::pg_catalog.regtype
              AND NOT attnotnull AND attidentity = '' AND attgenerated = '' AND NOT has_default
          )
        LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 3 FROM ledger_constraints)
    AND EXISTS (
        SELECT 1 FROM ledger_constraints
        WHERE contype = 'p' AND definition = 'PRIMARY KEY (version)'
    )
    AND EXISTS (
        SELECT 1 FROM ledger_constraints
        WHERE contype = 'c' AND definition = 'CHECK (version > 0)'
    )
    AND EXISTS (
        SELECT 1 FROM ledger_constraints
        WHERE contype = 'c' AND definition = 'CHECK (octet_length(checksum) = 32)'
    )
    AND (SELECT pg_catalog.count(*) = 14 FROM h3_constraints)
    AND (SELECT pg_catalog.count(*) = 8 FROM h3_constraints WHERE contype = 'c')
    AND (SELECT pg_catalog.count(*) = 5 FROM h3_constraints WHERE contype = 'f')
    AND (SELECT pg_catalog.count(*) = 1 FROM h3_constraints WHERE contype = 'p')
    AND (SELECT pg_catalog.bool_and(convalidated) FROM h3_constraints)
    AND NOT EXISTS (
        SELECT 1 FROM h3_constraints
        WHERE (contype = 'f' AND NOT (
                   condeferrable AND condeferred
                   AND confrelid = (SELECT oid FROM h3_cell)
               ))
           OR (contype <> 'f' AND (condeferrable OR condeferred))
        LIMIT 1
    )
    AND (SELECT pg_catalog.array_agg(conname ORDER BY conname) = ARRAY[
        'h3_cell_ancestor_r4_fkey',
        'h3_cell_ancestor_r4_matches',
        'h3_cell_ancestor_r5_fkey',
        'h3_cell_ancestor_r5_matches',
        'h3_cell_ancestor_r6_fkey',
        'h3_cell_ancestor_r6_matches',
        'h3_cell_ancestor_r7_fkey',
        'h3_cell_ancestor_r7_matches',
        'h3_cell_id_positive',
        'h3_cell_immediate_parent_fkey',
        'h3_cell_immediate_parent_matches',
        'h3_cell_pkey',
        'h3_cell_resolution_matches_id',
        'h3_cell_resolution_range'
    ]::pg_catalog.text[] FROM h3_constraints)
    AND (SELECT pg_catalog.count(*) = 2 FROM relation_indexes)
    AND NOT EXISTS (
        SELECT 1 FROM relation_indexes
        WHERE NOT (
            index_name = parent_name || '_pkey'
            AND indisprimary AND indisunique AND indisvalid AND indisready
            AND indpred IS NULL AND indexprs IS NULL
        )
        LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 4 FROM owned_classes)
    AND NOT EXISTS (
        SELECT 1 FROM owned_classes
        WHERE NOT (
            (nspname = 'babylon_ref' AND relname = 'h3_cell' AND relkind = 'r')
            OR (nspname = 'babylon_ref' AND relname = 'h3_cell_pkey' AND relkind = 'i')
            OR (
                nspname = 'babylon_state'
                AND relname = 'schema_migration'
                AND relkind = 'r'
            )
            OR (
                nspname = 'babylon_state'
                AND relname = 'schema_migration_pkey'
                AND relkind = 'i'
            )
        )
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1
        FROM pg_catalog.pg_proc AS routine
        JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = routine.pronamespace
        WHERE namespace.nspname IN ('babylon_ref', 'babylon_state')
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1
        FROM pg_catalog.pg_type AS type_row
        JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = type_row.typnamespace
        LEFT JOIN allowed_types AS allowed
          ON type_row.oid = allowed.oid OR type_row.oid = allowed.typarray
        WHERE namespace.nspname IN ('babylon_ref', 'babylon_state')
          AND allowed.oid IS NULL
        LIMIT 1
    ) AS epoch_shape_matches;
