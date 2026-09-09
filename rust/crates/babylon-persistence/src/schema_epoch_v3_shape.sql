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
        (namespace.nspname = 'babylon_ref' AND relation.relname IN (
            'h3_cell', 'h3_reference_cohort', 'h3_reference_membership'
        ))
        OR (namespace.nspname = 'babylon_state' AND relation.relname = 'schema_migration')
    )
      AND relation.relkind = 'r'
      AND relation.relpersistence = 'p'
    ORDER BY namespace.nspname, relation.relname
    LIMIT 5
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
           attribute.attgenerated, attribute.attacl,
           default_row.oid IS NOT NULL AS has_default
    FROM owned_relations AS relation
    JOIN pg_catalog.pg_attribute AS attribute ON attribute.attrelid = relation.oid
    LEFT JOIN pg_catalog.pg_attrdef AS default_row
      ON default_row.adrelid = attribute.attrelid
     AND default_row.adnum = attribute.attnum
    WHERE attribute.attnum > 0 AND NOT attribute.attisdropped
    ORDER BY relation.nspname, relation.relname, attribute.attnum
    LIMIT 26
),
expected_columns(nspname, relname, attnum, attname, atttypid) AS (
    VALUES
        ('babylon_ref'::pg_catalog.text, 'h3_cell'::pg_catalog.text, 1::pg_catalog.int2,
         'cell_id'::pg_catalog.name, 'pg_catalog.int8'::pg_catalog.regtype),
        ('babylon_ref', 'h3_cell', 2, 'resolution', 'pg_catalog.int2'::pg_catalog.regtype),
        ('babylon_ref', 'h3_cell', 3, 'immediate_parent', 'pg_catalog.int8'::pg_catalog.regtype),
        ('babylon_ref', 'h3_cell', 4, 'ancestor_r4', 'pg_catalog.int8'::pg_catalog.regtype),
        ('babylon_ref', 'h3_cell', 5, 'ancestor_r5', 'pg_catalog.int8'::pg_catalog.regtype),
        ('babylon_ref', 'h3_cell', 6, 'ancestor_r6', 'pg_catalog.int8'::pg_catalog.regtype),
        ('babylon_ref', 'h3_cell', 7, 'ancestor_r7', 'pg_catalog.int8'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_cohort', 1, 'ref_digest',
         'pg_catalog.bytea'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_cohort', 2, 'format_version',
         'pg_catalog.int2'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_cohort', 3, 'artifact_name',
         'pg_catalog.text'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_cohort', 4, 'artifact_manifest_version',
         'pg_catalog.text'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_cohort', 5, 'artifact_digest',
         'pg_catalog.bytea'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_cohort', 6, 'source_digest',
         'pg_catalog.bytea'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_cohort', 7, 'source_r5_digest',
         'pg_catalog.bytea'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_cohort', 8, 'source_r7_digest',
         'pg_catalog.bytea'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_cohort', 9, 'closure_digest',
         'pg_catalog.bytea'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_cohort', 10, 'membership_digest',
         'pg_catalog.bytea'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_cohort', 11, 'direct_cell_count',
         'pg_catalog.int8'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_cohort', 12, 'derived_ancestor_count',
         'pg_catalog.int8'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_cohort', 13, 'closure_cell_count',
         'pg_catalog.int8'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_membership', 1, 'ref_digest',
         'pg_catalog.bytea'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_membership', 2, 'cell_id',
         'pg_catalog.int8'::pg_catalog.regtype),
        ('babylon_ref', 'h3_reference_membership', 3, 'origin',
         'pg_catalog.int2'::pg_catalog.regtype),
        ('babylon_state', 'schema_migration', 1, 'version',
         'pg_catalog.int8'::pg_catalog.regtype),
        ('babylon_state', 'schema_migration', 2, 'checksum',
         'pg_catalog.bytea'::pg_catalog.regtype)
),
relation_constraints AS MATERIALIZED (
    SELECT relation.relname, constraint_row.conname::pg_catalog.text AS conname,
           constraint_row.contype, constraint_row.convalidated,
           constraint_row.condeferrable, constraint_row.condeferred,
           constraint_row.conkey, constraint_row.confrelid, constraint_row.confkey,
           constraint_row.confupdtype, constraint_row.confdeltype,
           constraint_row.confmatchtype,
           pg_catalog.pg_get_constraintdef(constraint_row.oid, true) AS definition
    FROM owned_relations AS relation
    JOIN pg_catalog.pg_constraint AS constraint_row
      ON constraint_row.conrelid = relation.oid
    ORDER BY relation.relname, constraint_row.conname
    LIMIT 39
),
expected_reference_constraints(
    relname, conname, contype, conkey, condeferrable, condeferred, definition
) AS (
    VALUES
        ('h3_reference_cohort'::pg_catalog.text,
         'h3_reference_cohort_artifact_digest_length'::pg_catalog.text,
         'c'::pg_catalog."char", ARRAY[5]::pg_catalog.int2[], false, false,
         'CHECK (octet_length(artifact_digest) = 32)'::pg_catalog.text),
        ('h3_reference_cohort', 'h3_reference_cohort_artifact_identity',
         'u'::pg_catalog."char", ARRAY[2, 5]::pg_catalog.int2[], false, false,
         'UNIQUE (format_version, artifact_digest)'),
        ('h3_reference_cohort', 'h3_reference_cohort_artifact_manifest_version_length',
         'c'::pg_catalog."char", ARRAY[4]::pg_catalog.int2[], false, false,
         'CHECK (octet_length(artifact_manifest_version) >= 1 AND '
         'octet_length(artifact_manifest_version) <= 64)'),
        ('h3_reference_cohort', 'h3_reference_cohort_artifact_name_length',
         'c'::pg_catalog."char", ARRAY[3]::pg_catalog.int2[], false, false,
         'CHECK (octet_length(artifact_name) >= 1 AND octet_length(artifact_name) <= 255)'),
        ('h3_reference_cohort', 'h3_reference_cohort_closure_count_matches',
         'c'::pg_catalog."char", ARRAY[13, 11, 12]::pg_catalog.int2[], false, false,
         'CHECK (closure_cell_count >= 1 AND closure_cell_count <= 1048576 AND '
         'closure_cell_count = (direct_cell_count + derived_ancestor_count))'),
        ('h3_reference_cohort', 'h3_reference_cohort_closure_digest_length',
         'c'::pg_catalog."char", ARRAY[9]::pg_catalog.int2[], false, false,
         'CHECK (octet_length(closure_digest) = 32)'),
        ('h3_reference_cohort', 'h3_reference_cohort_derived_count_nonnegative',
         'c'::pg_catalog."char", ARRAY[12]::pg_catalog.int2[], false, false,
         'CHECK (derived_ancestor_count >= 0 AND derived_ancestor_count <= 1048576)'),
        ('h3_reference_cohort', 'h3_reference_cohort_direct_count_positive',
         'c'::pg_catalog."char", ARRAY[11]::pg_catalog.int2[], false, false,
         'CHECK (direct_cell_count >= 1 AND direct_cell_count <= 65536)'),
        ('h3_reference_cohort', 'h3_reference_cohort_format_v1',
         'c'::pg_catalog."char", ARRAY[2]::pg_catalog.int2[], false, false,
         'CHECK (format_version = 1)'),
        ('h3_reference_cohort', 'h3_reference_cohort_membership_digest_length',
         'c'::pg_catalog."char", ARRAY[10]::pg_catalog.int2[], false, false,
         'CHECK (octet_length(membership_digest) = 32)'),
        ('h3_reference_cohort', 'h3_reference_cohort_pkey',
         'p'::pg_catalog."char", ARRAY[1]::pg_catalog.int2[], false, false,
         'PRIMARY KEY (ref_digest)'),
        ('h3_reference_cohort', 'h3_reference_cohort_ref_digest_length',
         'c'::pg_catalog."char", ARRAY[1]::pg_catalog.int2[], false, false,
         'CHECK (octet_length(ref_digest) = 32)'),
        ('h3_reference_cohort', 'h3_reference_cohort_source_digest_length',
         'c'::pg_catalog."char", ARRAY[6]::pg_catalog.int2[], false, false,
         'CHECK (octet_length(source_digest) = 32)'),
        ('h3_reference_cohort', 'h3_reference_cohort_source_r5_digest_length',
         'c'::pg_catalog."char", ARRAY[7]::pg_catalog.int2[], false, false,
         'CHECK (octet_length(source_r5_digest) = 32)'),
        ('h3_reference_cohort', 'h3_reference_cohort_source_r7_digest_length',
         'c'::pg_catalog."char", ARRAY[8]::pg_catalog.int2[], false, false,
         'CHECK (octet_length(source_r7_digest) = 32)'),
        ('h3_reference_membership', 'h3_reference_membership_cell_fkey',
         'f'::pg_catalog."char", ARRAY[2]::pg_catalog.int2[], true, true,
         'FOREIGN KEY (cell_id) REFERENCES babylon_ref.h3_cell(cell_id) '
         'DEFERRABLE INITIALLY DEFERRED'),
        ('h3_reference_membership', 'h3_reference_membership_cell_positive',
         'c'::pg_catalog."char", ARRAY[2]::pg_catalog.int2[], false, false,
         'CHECK (cell_id > 0)'),
        ('h3_reference_membership', 'h3_reference_membership_cohort_fkey',
         'f'::pg_catalog."char", ARRAY[1]::pg_catalog.int2[], true, true,
         'FOREIGN KEY (ref_digest) REFERENCES babylon_ref.h3_reference_cohort(ref_digest) '
         'DEFERRABLE INITIALLY DEFERRED'),
        ('h3_reference_membership', 'h3_reference_membership_origin_closed',
         'c'::pg_catalog."char", ARRAY[3]::pg_catalog.int2[], false, false,
         'CHECK (origin = ANY (ARRAY[1, 2]))'),
        ('h3_reference_membership', 'h3_reference_membership_pkey',
         'p'::pg_catalog."char", ARRAY[1, 2]::pg_catalog.int2[], false, false,
         'PRIMARY KEY (ref_digest, cell_id)'),
        ('h3_reference_membership', 'h3_reference_membership_ref_digest_length',
         'c'::pg_catalog."char", ARRAY[1]::pg_catalog.int2[], false, false,
         'CHECK (octet_length(ref_digest) = 32)')
),
relation_indexes AS MATERIALIZED (
    SELECT parent.relname AS parent_name, index_relation.relname AS index_name,
           index_row.indisprimary, index_row.indisunique, index_row.indisvalid,
           index_row.indisready, index_row.indpred, index_row.indexprs,
           index_row.indnkeyatts, index_row.indnatts,
           pg_catalog.pg_get_indexdef(index_relation.oid, 1, true) AS key_one,
           CASE WHEN index_row.indnkeyatts >= 2
                THEN pg_catalog.pg_get_indexdef(index_relation.oid, 2, true)
                ELSE NULL END AS key_two
    FROM owned_relations AS parent
    JOIN pg_catalog.pg_index AS index_row ON index_row.indrelid = parent.oid
    JOIN pg_catalog.pg_class AS index_relation ON index_relation.oid = index_row.indexrelid
    ORDER BY parent.relname, index_relation.relname
    LIMIT 7
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
    LIMIT 11
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
    AND (SELECT pg_catalog.count(*) = 4 FROM owned_relations)
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
    AND (SELECT pg_catalog.count(*) = 25 FROM relation_columns)
    AND (SELECT pg_catalog.count(*) = 25 FROM expected_columns)
    AND NOT EXISTS (
        SELECT 1
        FROM relation_columns AS actual
        LEFT JOIN expected_columns AS expected
          ON expected.nspname = actual.nspname
         AND expected.relname = actual.relname
         AND expected.attnum = actual.attnum
         AND expected.attname = actual.attname
         AND expected.atttypid = actual.atttypid
        WHERE expected.attnum IS NULL
           OR actual.attnotnull IS DISTINCT FROM CASE
               WHEN actual.nspname = 'babylon_ref'
                AND actual.relname = 'h3_cell'
                AND actual.attnum BETWEEN 3 AND 7 THEN false
               ELSE true
           END
           OR actual.attidentity <> ''
           OR actual.attgenerated <> ''
           OR actual.has_default
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1
        FROM expected_columns AS expected
        LEFT JOIN relation_columns AS actual
          ON actual.nspname = expected.nspname
         AND actual.relname = expected.relname
         AND actual.attnum = expected.attnum
         AND actual.attname = expected.attname
         AND actual.atttypid = expected.atttypid
        WHERE actual.attnum IS NULL
        LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 38 FROM relation_constraints)
    AND (SELECT pg_catalog.count(*) = 3 FROM relation_constraints
         WHERE relname = 'schema_migration')
    AND EXISTS (
        SELECT 1 FROM relation_constraints
        WHERE relname = 'schema_migration' AND contype = 'p'
          AND definition = 'PRIMARY KEY (version)'
    )
    AND EXISTS (
        SELECT 1 FROM relation_constraints
        WHERE relname = 'schema_migration' AND contype = 'c'
          AND definition = 'CHECK (version > 0)'
    )
    AND EXISTS (
        SELECT 1 FROM relation_constraints
        WHERE relname = 'schema_migration' AND contype = 'c'
          AND definition = 'CHECK (octet_length(checksum) = 32)'
    )
    AND (SELECT pg_catalog.count(*) = 14 FROM relation_constraints
         WHERE relname = 'h3_cell')
    AND (SELECT pg_catalog.count(*) = 8 FROM relation_constraints
         WHERE relname = 'h3_cell' AND contype = 'c')
    AND (SELECT pg_catalog.count(*) = 5 FROM relation_constraints
         WHERE relname = 'h3_cell' AND contype = 'f')
    AND (SELECT pg_catalog.count(*) = 1 FROM relation_constraints
         WHERE relname = 'h3_cell' AND contype = 'p')
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
    ]::pg_catalog.text[] FROM relation_constraints WHERE relname = 'h3_cell')
    AND (SELECT pg_catalog.count(*) = 15 FROM relation_constraints
         WHERE relname = 'h3_reference_cohort')
    AND (SELECT pg_catalog.count(*) = 13 FROM relation_constraints
         WHERE relname = 'h3_reference_cohort' AND contype = 'c')
    AND (SELECT pg_catalog.count(*) = 1 FROM relation_constraints
         WHERE relname = 'h3_reference_cohort' AND contype = 'p')
    AND (SELECT pg_catalog.count(*) = 1 FROM relation_constraints
         WHERE relname = 'h3_reference_cohort' AND contype = 'u')
    AND (SELECT pg_catalog.array_agg(conname ORDER BY conname) = ARRAY[
        'h3_reference_cohort_artifact_digest_length',
        'h3_reference_cohort_artifact_identity',
        'h3_reference_cohort_artifact_manifest_version_length',
        'h3_reference_cohort_artifact_name_length',
        'h3_reference_cohort_closure_count_matches',
        'h3_reference_cohort_closure_digest_length',
        'h3_reference_cohort_derived_count_nonnegative',
        'h3_reference_cohort_direct_count_positive',
        'h3_reference_cohort_format_v1',
        'h3_reference_cohort_membership_digest_length',
        'h3_reference_cohort_pkey',
        'h3_reference_cohort_ref_digest_length',
        'h3_reference_cohort_source_digest_length',
        'h3_reference_cohort_source_r5_digest_length',
        'h3_reference_cohort_source_r7_digest_length'
    ]::pg_catalog.text[] FROM relation_constraints
        WHERE relname = 'h3_reference_cohort')
    AND (SELECT pg_catalog.count(*) = 6 FROM relation_constraints
         WHERE relname = 'h3_reference_membership')
    AND (SELECT pg_catalog.count(*) = 3 FROM relation_constraints
         WHERE relname = 'h3_reference_membership' AND contype = 'c')
    AND (SELECT pg_catalog.count(*) = 2 FROM relation_constraints
         WHERE relname = 'h3_reference_membership' AND contype = 'f')
    AND (SELECT pg_catalog.count(*) = 1 FROM relation_constraints
         WHERE relname = 'h3_reference_membership' AND contype = 'p')
    AND (SELECT pg_catalog.array_agg(conname ORDER BY conname) = ARRAY[
        'h3_reference_membership_cell_fkey',
        'h3_reference_membership_cell_positive',
        'h3_reference_membership_cohort_fkey',
        'h3_reference_membership_origin_closed',
        'h3_reference_membership_pkey',
        'h3_reference_membership_ref_digest_length'
    ]::pg_catalog.text[] FROM relation_constraints
        WHERE relname = 'h3_reference_membership')
    AND (SELECT pg_catalog.count(*) = 21 FROM expected_reference_constraints)
    AND NOT EXISTS (
        SELECT 1
        FROM relation_constraints AS actual
        LEFT JOIN expected_reference_constraints AS expected
          ON expected.relname = actual.relname
         AND expected.conname = actual.conname
        WHERE actual.relname IN (
            'h3_reference_cohort', 'h3_reference_membership'
        )
          AND (
              expected.conname IS NULL
              OR actual.contype IS DISTINCT FROM expected.contype
              OR actual.conkey IS DISTINCT FROM expected.conkey
              OR actual.condeferrable IS DISTINCT FROM expected.condeferrable
              OR actual.condeferred IS DISTINCT FROM expected.condeferred
              OR actual.definition IS DISTINCT FROM expected.definition
          )
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1
        FROM expected_reference_constraints AS expected
        LEFT JOIN relation_constraints AS actual
          ON actual.relname = expected.relname
         AND actual.conname = expected.conname
        WHERE actual.conname IS NULL
        LIMIT 1
    )
    AND (SELECT pg_catalog.bool_and(convalidated) FROM relation_constraints)
    AND NOT EXISTS (
        SELECT 1 FROM relation_constraints
        WHERE (contype = 'f' AND NOT (condeferrable AND condeferred))
           OR (contype <> 'f' AND (condeferrable OR condeferred))
        LIMIT 1
    )
    AND NOT EXISTS (
        SELECT 1 FROM relation_constraints
        WHERE relname = 'h3_reference_membership'
          AND contype = 'f'
          AND (
              (conname = 'h3_reference_membership_cohort_fkey'
               AND confrelid <> (
                   SELECT oid FROM owned_relations
                   WHERE relname = 'h3_reference_cohort'
               ))
              OR (conname = 'h3_reference_membership_cell_fkey'
                  AND confrelid <> (
                      SELECT oid FROM owned_relations WHERE relname = 'h3_cell'
                  ))
              OR confkey IS DISTINCT FROM ARRAY[1]::pg_catalog.int2[]
              OR confupdtype <> 'a'
              OR confdeltype <> 'a'
              OR confmatchtype <> 's'
          )
        LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 6 FROM relation_indexes)
    AND NOT EXISTS (
        SELECT 1 FROM relation_indexes
        WHERE indpred IS NOT NULL OR indexprs IS NOT NULL
           OR NOT indisvalid OR NOT indisready
           OR indnatts <> indnkeyatts
           OR NOT (
               (parent_name = 'schema_migration'
                AND index_name = 'schema_migration_pkey'
                AND indisprimary AND indisunique AND indnkeyatts = 1
                AND key_one = 'version' AND key_two IS NULL)
               OR (parent_name = 'h3_cell'
                   AND index_name = 'h3_cell_pkey'
                   AND indisprimary AND indisunique AND indnkeyatts = 1
                   AND key_one = 'cell_id' AND key_two IS NULL)
               OR (parent_name = 'h3_reference_cohort'
                   AND index_name = 'h3_reference_cohort_pkey'
                   AND indisprimary AND indisunique AND indnkeyatts = 1
                   AND key_one = 'ref_digest' AND key_two IS NULL)
               OR (parent_name = 'h3_reference_cohort'
                   AND index_name = 'h3_reference_cohort_artifact_identity'
                   AND NOT indisprimary AND indisunique AND indnkeyatts = 2
                   AND key_one = 'format_version' AND key_two = 'artifact_digest')
               OR (parent_name = 'h3_reference_membership'
                   AND index_name = 'h3_reference_membership_pkey'
                   AND indisprimary AND indisunique AND indnkeyatts = 2
                   AND key_one = 'ref_digest' AND key_two = 'cell_id')
               OR (parent_name = 'h3_reference_membership'
                   AND index_name = 'h3_reference_membership_cell_id_idx'
                   AND NOT indisprimary AND NOT indisunique AND indnkeyatts = 2
                   AND key_one = 'cell_id' AND key_two = 'ref_digest')
           )
        LIMIT 1
    )
    AND (SELECT pg_catalog.count(*) = 10 FROM owned_classes)
    AND NOT EXISTS (
        SELECT 1 FROM owned_classes
        WHERE NOT (
            (nspname = 'babylon_ref' AND relname = 'h3_cell' AND relkind = 'r')
            OR (nspname = 'babylon_ref' AND relname = 'h3_cell_pkey' AND relkind = 'i')
            OR (nspname = 'babylon_ref' AND relname = 'h3_reference_cohort'
                AND relkind = 'r')
            OR (nspname = 'babylon_ref' AND relname = 'h3_reference_cohort_pkey'
                AND relkind = 'i')
            OR (nspname = 'babylon_ref'
                AND relname = 'h3_reference_cohort_artifact_identity' AND relkind = 'i')
            OR (nspname = 'babylon_ref' AND relname = 'h3_reference_membership'
                AND relkind = 'r')
            OR (nspname = 'babylon_ref'
                AND relname = 'h3_reference_membership_cell_id_idx' AND relkind = 'i')
            OR (nspname = 'babylon_ref' AND relname = 'h3_reference_membership_pkey'
                AND relkind = 'i')
            OR (nspname = 'babylon_state' AND relname = 'schema_migration'
                AND relkind = 'r')
            OR (nspname = 'babylon_state' AND relname = 'schema_migration_pkey'
                AND relkind = 'i')
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
