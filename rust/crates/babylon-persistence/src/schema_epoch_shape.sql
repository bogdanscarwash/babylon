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
ledger AS MATERIALIZED (
    SELECT relation.oid, relation.relowner, relation.relacl, relation.reltype
    FROM pg_catalog.pg_class AS relation
    JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace
    WHERE namespace.nspname = 'babylon_state'
      AND relation.relname = 'schema_migration'
      AND relation.relkind = 'r'
      AND relation.relpersistence = 'p'
    LIMIT 2
),
intel_role AS MATERIALIZED (
    SELECT role_row.oid
    FROM pg_catalog.pg_roles AS role_row
    WHERE role_row.rolname = 'babylon_intel'
    LIMIT 2
),
ledger_columns AS MATERIALIZED (
    SELECT attribute.attnum, attribute.attname, attribute.atttypid, attribute.attnotnull,
           attribute.attidentity, attribute.attgenerated,
           default_row.oid IS NOT NULL AS has_default
    FROM ledger
    JOIN pg_catalog.pg_attribute AS attribute ON attribute.attrelid = ledger.oid
    LEFT JOIN pg_catalog.pg_attrdef AS default_row
      ON default_row.adrelid = attribute.attrelid
     AND default_row.adnum = attribute.attnum
    WHERE attribute.attnum > 0 AND NOT attribute.attisdropped
    ORDER BY attribute.attnum
    LIMIT 3
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
ledger_types AS MATERIALIZED (
    SELECT type_row.oid, type_row.typarray
    FROM ledger
    JOIN pg_catalog.pg_type AS type_row ON type_row.oid = ledger.reltype
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
    AND (SELECT pg_catalog.count(*) = 1 FROM ledger)
    AND (SELECT pg_catalog.bool_and(ledger.relowner = owner_row.owner_oid)
         FROM ledger CROSS JOIN database_owner AS owner_row)
    AND NOT EXISTS (
        SELECT 1
        FROM ledger
        CROSS JOIN LATERAL pg_catalog.aclexplode(
            coalesce(ledger.relacl, pg_catalog.acldefault('r', ledger.relowner))
        ) AS acl
        WHERE acl.grantee = 0
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1
        FROM intel_role AS intel
        CROSS JOIN ledger
        WHERE pg_catalog.has_table_privilege(intel.oid, ledger.oid, 'SELECT')
           OR pg_catalog.has_table_privilege(intel.oid, ledger.oid, 'INSERT')
           OR pg_catalog.has_table_privilege(intel.oid, ledger.oid, 'UPDATE')
           OR pg_catalog.has_table_privilege(intel.oid, ledger.oid, 'DELETE')
           OR pg_catalog.has_table_privilege(intel.oid, ledger.oid, 'TRUNCATE')
           OR pg_catalog.has_table_privilege(intel.oid, ledger.oid, 'REFERENCES')
           OR pg_catalog.has_table_privilege(intel.oid, ledger.oid, 'TRIGGER')
        LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 2 FROM ledger_columns)
    AND EXISTS (
        SELECT 1 FROM ledger_columns
        WHERE attnum = 1 AND attname = 'version'
          AND atttypid = 'pg_catalog.int8'::pg_catalog.regtype
          AND attnotnull AND attidentity = '' AND attgenerated = '' AND NOT has_default
    )
    AND EXISTS (
        SELECT 1 FROM ledger_columns
        WHERE attnum = 2 AND attname = 'checksum'
          AND atttypid = 'pg_catalog.bytea'::pg_catalog.regtype
          AND attnotnull AND attidentity = '' AND attgenerated = '' AND NOT has_default
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
    AND NOT EXISTS (
        SELECT 1
        FROM pg_catalog.pg_class AS relation
        JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace
        WHERE namespace.nspname IN ('babylon_ref', 'babylon_state')
          AND NOT (
              namespace.nspname = 'babylon_state'
              AND relation.relname IN ('schema_migration', 'schema_migration_pkey')
              AND relation.relkind IN ('r', 'i')
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
        LEFT JOIN ledger_types AS allowed
          ON type_row.oid = allowed.oid OR type_row.oid = allowed.typarray
        WHERE namespace.nspname IN ('babylon_ref', 'babylon_state')
          AND allowed.oid IS NULL
        LIMIT 1
    ) AS epoch_shape_matches;
