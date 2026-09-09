WITH database_owner AS (
    SELECT d.datdba AS owner_oid
    FROM pg_catalog.pg_database AS d
    WHERE d.datname = pg_catalog.current_database()
),
current_database_row AS MATERIALIZED (
    SELECT d.*
    FROM pg_catalog.pg_database AS d
    WHERE d.datname = pg_catalog.current_database()
),
database_settings AS MATERIALIZED (
    SELECT settings.setrole, settings.setconfig
    FROM pg_catalog.pg_db_role_setting AS settings
    JOIN current_database_row AS database_row
      ON database_row.oid = settings.setdatabase
    ORDER BY settings.setrole
    LIMIT $1
),
candidate_database_setting_configs AS MATERIALIZED (
    SELECT settings.setrole, selected.value
    FROM database_settings AS settings
    CROSS JOIN LATERAL (
        SELECT config_item.value
        FROM pg_catalog.unnest(coalesce(settings.setconfig, ARRAY[]::pg_catalog.text[]))
            AS config_item(value)
        ORDER BY config_item.value
        LIMIT $1
    ) AS selected
),
candidate_database_acls AS MATERIALIZED (
    SELECT selected.grantor, selected.grantee, selected.privilege_type, selected.is_grantable
    FROM current_database_row AS database_row
    CROSS JOIN database_owner AS own
    CROSS JOIN LATERAL (
        SELECT acl.grantor, acl.grantee, acl.privilege_type, acl.is_grantable
        FROM pg_catalog.aclexplode(
            coalesce(database_row.datacl, pg_catalog.acldefault('d', database_row.datdba))
        ) AS acl
        ORDER BY
            CASE
                WHEN acl.grantee = 0 THEN 'PUBLIC'
                WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(acl.grantee)
            END,
            acl.privilege_type,
            CASE
                WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(acl.grantor)
            END,
            acl.is_grantable
        LIMIT $1
    ) AS selected
),
governed_parents(parent_name) AS (
    VALUES
        ('boundary_flow_register'::pg_catalog.text),
        ('conservation_audit_log'::pg_catalog.text),
        ('dynamic_consciousness_state'::pg_catalog.text),
        ('dynamic_demographics_state'::pg_catalog.text),
        ('dynamic_employment_state'::pg_catalog.text),
        ('dynamic_external_node_state'::pg_catalog.text),
        ('dynamic_hex_state'::pg_catalog.text),
        ('dynamic_relationship_state'::pg_catalog.text),
        ('tick_commit'::pg_catalog.text)
),
candidate_namespaces AS MATERIALIZED (
    SELECT n.oid, n.nspname, n.nspowner, n.nspacl
    FROM pg_catalog.pg_namespace AS n
    WHERE n.nspname <> 'information_schema'
      AND n.nspname NOT LIKE 'pg\_%' ESCAPE '\'
    ORDER BY n.nspname
    LIMIT $1
),
protected_system_namespaces AS MATERIALIZED (
    SELECT n.oid, n.nspname
    FROM pg_catalog.pg_namespace AS n
    WHERE n.nspname IN ('pg_catalog', 'information_schema')
    ORDER BY n.nspname
    LIMIT 2
),
candidate_unsupported_catalog AS MATERIALIZED (
    SELECT 'pg_am'::pg_catalog.text AS family, candidate.oid
    FROM (
        SELECT access_method.oid
        FROM pg_catalog.pg_am AS access_method
        WHERE access_method.oid >= 16384
          AND NOT EXISTS (
              SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
              WHERE extension_dependency.classid = 'pg_catalog.pg_am'::pg_catalog.regclass
                AND extension_dependency.objid = access_method.oid
                AND extension_dependency.deptype = 'e'
          )
        ORDER BY access_method.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_cast', candidate.oid
    FROM (
        SELECT cast_row.oid
        FROM pg_catalog.pg_cast AS cast_row
        JOIN pg_catalog.pg_type AS source_type ON source_type.oid = cast_row.castsource
        JOIN pg_catalog.pg_type AS target_type ON target_type.oid = cast_row.casttarget
        LEFT JOIN pg_catalog.pg_proc AS cast_function ON cast_function.oid = cast_row.castfunc
        WHERE (
            cast_row.oid >= 16384
            OR EXISTS (SELECT 1 FROM candidate_namespaces AS n WHERE n.oid = source_type.typnamespace)
            OR EXISTS (SELECT 1 FROM candidate_namespaces AS n WHERE n.oid = target_type.typnamespace)
            OR EXISTS (SELECT 1 FROM candidate_namespaces AS n WHERE n.oid = cast_function.pronamespace)
        )
          AND NOT EXISTS (
              SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
              WHERE extension_dependency.classid = 'pg_catalog.pg_cast'::pg_catalog.regclass
                AND extension_dependency.objid = cast_row.oid
                AND extension_dependency.deptype = 'e'
          )
        ORDER BY cast_row.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_class', candidate.oid
    FROM (
        SELECT relation.oid
        FROM pg_catalog.pg_class AS relation
        JOIN protected_system_namespaces AS object_namespace
          ON object_namespace.oid = relation.relnamespace
        WHERE relation.oid >= 16384
          AND relation.relpersistence <> 't'
          AND NOT EXISTS (
              SELECT 1 FROM pg_catalog.pg_depend AS dependency
              WHERE dependency.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
                AND dependency.objid = relation.oid
                AND dependency.deptype IN ('e', 'i')
          )
        ORDER BY object_namespace.nspname, relation.relname, relation.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_proc', candidate.oid
    FROM (
        SELECT routine.oid
        FROM pg_catalog.pg_proc AS routine
        JOIN protected_system_namespaces AS object_namespace
          ON object_namespace.oid = routine.pronamespace
        WHERE routine.oid >= 16384
          AND NOT EXISTS (
              SELECT 1 FROM pg_catalog.pg_depend AS dependency
              WHERE dependency.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
                AND dependency.objid = routine.oid
                AND dependency.deptype IN ('e', 'i')
          )
        ORDER BY object_namespace.nspname, routine.proname, routine.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_type', candidate.oid
    FROM (
        SELECT type_row.oid
        FROM pg_catalog.pg_type AS type_row
        JOIN protected_system_namespaces AS object_namespace
          ON object_namespace.oid = type_row.typnamespace
        WHERE type_row.oid >= 16384
          AND NOT EXISTS (
              SELECT 1 FROM pg_catalog.pg_depend AS dependency
              WHERE dependency.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
                AND dependency.objid = type_row.oid
                AND dependency.deptype IN ('e', 'i')
          )
        ORDER BY object_namespace.nspname, type_row.typname, type_row.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_collation', candidate.oid
    FROM (
        SELECT collation_row.oid
        FROM pg_catalog.pg_collation AS collation_row
        JOIN pg_catalog.pg_namespace AS catalog_namespace
          ON catalog_namespace.oid = collation_row.collnamespace
        LEFT JOIN candidate_namespaces AS n ON n.oid = collation_row.collnamespace
        LEFT JOIN protected_system_namespaces AS object_namespace
          ON object_namespace.oid = collation_row.collnamespace
        WHERE (n.oid IS NOT NULL OR (
            collation_row.oid >= 16384 AND object_namespace.oid IS NOT NULL
        ))
          AND NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_collation'::pg_catalog.regclass
              AND extension_dependency.objid = collation_row.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY catalog_namespace.nspname, collation_row.collname, collation_row.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_conversion', candidate.oid
    FROM (
        SELECT conversion.oid
        FROM pg_catalog.pg_conversion AS conversion
        JOIN pg_catalog.pg_namespace AS catalog_namespace
          ON catalog_namespace.oid = conversion.connamespace
        LEFT JOIN candidate_namespaces AS n ON n.oid = conversion.connamespace
        LEFT JOIN protected_system_namespaces AS object_namespace
          ON object_namespace.oid = conversion.connamespace
        WHERE (n.oid IS NOT NULL OR (
            conversion.oid >= 16384 AND object_namespace.oid IS NOT NULL
        ))
          AND NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_conversion'::pg_catalog.regclass
              AND extension_dependency.objid = conversion.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY catalog_namespace.nspname, conversion.conname, conversion.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_event_trigger', candidate.oid
    FROM (
        SELECT event_trigger.oid
        FROM pg_catalog.pg_event_trigger AS event_trigger
        WHERE NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_event_trigger'::pg_catalog.regclass
              AND extension_dependency.objid = event_trigger.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY event_trigger.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_foreign_data_wrapper', candidate.oid
    FROM (
        SELECT wrapper.oid
        FROM pg_catalog.pg_foreign_data_wrapper AS wrapper
        WHERE NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_foreign_data_wrapper'::pg_catalog.regclass
              AND extension_dependency.objid = wrapper.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY wrapper.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_foreign_server', candidate.oid
    FROM (
        SELECT server.oid
        FROM pg_catalog.pg_foreign_server AS server
        WHERE NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_foreign_server'::pg_catalog.regclass
              AND extension_dependency.objid = server.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY server.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_language', candidate.oid
    FROM (
        SELECT language.oid
        FROM pg_catalog.pg_language AS language
        WHERE language.lanispl
          AND NOT EXISTS (
              SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
              WHERE extension_dependency.classid = 'pg_catalog.pg_language'::pg_catalog.regclass
                AND extension_dependency.objid = language.oid
                AND extension_dependency.deptype = 'e'
          )
        ORDER BY language.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_largeobject_metadata', candidate.oid
    FROM (
        SELECT large_object.oid
        FROM pg_catalog.pg_largeobject_metadata AS large_object
        WHERE NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_largeobject'::pg_catalog.regclass
              AND extension_dependency.objid = large_object.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY large_object.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_operator', candidate.oid
    FROM (
        SELECT operator.oid
        FROM pg_catalog.pg_operator AS operator
        JOIN pg_catalog.pg_namespace AS catalog_namespace
          ON catalog_namespace.oid = operator.oprnamespace
        LEFT JOIN candidate_namespaces AS n ON n.oid = operator.oprnamespace
        LEFT JOIN protected_system_namespaces AS object_namespace
          ON object_namespace.oid = operator.oprnamespace
        WHERE (n.oid IS NOT NULL OR (
            operator.oid >= 16384 AND object_namespace.oid IS NOT NULL
        ))
          AND NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_operator'::pg_catalog.regclass
              AND extension_dependency.objid = operator.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY catalog_namespace.nspname, operator.oprname, operator.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_opclass', candidate.oid
    FROM (
        SELECT operator_class.oid
        FROM pg_catalog.pg_opclass AS operator_class
        JOIN pg_catalog.pg_namespace AS catalog_namespace
          ON catalog_namespace.oid = operator_class.opcnamespace
        LEFT JOIN candidate_namespaces AS n ON n.oid = operator_class.opcnamespace
        LEFT JOIN protected_system_namespaces AS object_namespace
          ON object_namespace.oid = operator_class.opcnamespace
        WHERE (n.oid IS NOT NULL OR (
            operator_class.oid >= 16384 AND object_namespace.oid IS NOT NULL
        ))
          AND NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_opclass'::pg_catalog.regclass
              AND extension_dependency.objid = operator_class.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY catalog_namespace.nspname, operator_class.opcname, operator_class.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_opfamily', candidate.oid
    FROM (
        SELECT operator_family.oid
        FROM pg_catalog.pg_opfamily AS operator_family
        JOIN pg_catalog.pg_namespace AS catalog_namespace
          ON catalog_namespace.oid = operator_family.opfnamespace
        LEFT JOIN candidate_namespaces AS n ON n.oid = operator_family.opfnamespace
        LEFT JOIN protected_system_namespaces AS object_namespace
          ON object_namespace.oid = operator_family.opfnamespace
        WHERE (n.oid IS NOT NULL OR (
            operator_family.oid >= 16384 AND object_namespace.oid IS NOT NULL
        ))
          AND NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_opfamily'::pg_catalog.regclass
              AND extension_dependency.objid = operator_family.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY catalog_namespace.nspname, operator_family.opfname, operator_family.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_publication', candidate.oid
    FROM (
        SELECT publication.oid
        FROM pg_catalog.pg_publication AS publication
        WHERE NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_publication'::pg_catalog.regclass
              AND extension_dependency.objid = publication.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY publication.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_subscription', candidate.oid
    FROM (
        SELECT subscription.oid
        FROM pg_catalog.pg_subscription AS subscription
        JOIN current_database_row AS database_row ON database_row.oid = subscription.subdbid
        WHERE NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_subscription'::pg_catalog.regclass
              AND extension_dependency.objid = subscription.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY subscription.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_statistic_ext', candidate.oid
    FROM (
        SELECT statistics.oid
        FROM pg_catalog.pg_statistic_ext AS statistics
        JOIN pg_catalog.pg_namespace AS catalog_namespace
          ON catalog_namespace.oid = statistics.stxnamespace
        LEFT JOIN candidate_namespaces AS n ON n.oid = statistics.stxnamespace
        LEFT JOIN protected_system_namespaces AS object_namespace
          ON object_namespace.oid = statistics.stxnamespace
        WHERE (n.oid IS NOT NULL OR (
            statistics.oid >= 16384 AND object_namespace.oid IS NOT NULL
        ))
          AND NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_statistic_ext'::pg_catalog.regclass
              AND extension_dependency.objid = statistics.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY catalog_namespace.nspname, statistics.stxname, statistics.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_transform', candidate.oid
    FROM (
        SELECT transform.oid
        FROM pg_catalog.pg_transform AS transform
        WHERE NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_transform'::pg_catalog.regclass
              AND extension_dependency.objid = transform.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY transform.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_ts_config', candidate.oid
    FROM (
        SELECT configuration.oid
        FROM pg_catalog.pg_ts_config AS configuration
        JOIN pg_catalog.pg_namespace AS catalog_namespace
          ON catalog_namespace.oid = configuration.cfgnamespace
        LEFT JOIN candidate_namespaces AS n ON n.oid = configuration.cfgnamespace
        LEFT JOIN protected_system_namespaces AS object_namespace
          ON object_namespace.oid = configuration.cfgnamespace
        WHERE (n.oid IS NOT NULL OR (
            configuration.oid >= 16384 AND object_namespace.oid IS NOT NULL
        ))
          AND NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_ts_config'::pg_catalog.regclass
              AND extension_dependency.objid = configuration.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY catalog_namespace.nspname, configuration.cfgname, configuration.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_ts_dict', candidate.oid
    FROM (
        SELECT dictionary.oid
        FROM pg_catalog.pg_ts_dict AS dictionary
        JOIN pg_catalog.pg_namespace AS catalog_namespace
          ON catalog_namespace.oid = dictionary.dictnamespace
        LEFT JOIN candidate_namespaces AS n ON n.oid = dictionary.dictnamespace
        LEFT JOIN protected_system_namespaces AS object_namespace
          ON object_namespace.oid = dictionary.dictnamespace
        WHERE (n.oid IS NOT NULL OR (
            dictionary.oid >= 16384 AND object_namespace.oid IS NOT NULL
        ))
          AND NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_ts_dict'::pg_catalog.regclass
              AND extension_dependency.objid = dictionary.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY catalog_namespace.nspname, dictionary.dictname, dictionary.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_ts_parser', candidate.oid
    FROM (
        SELECT parser.oid
        FROM pg_catalog.pg_ts_parser AS parser
        JOIN pg_catalog.pg_namespace AS catalog_namespace
          ON catalog_namespace.oid = parser.prsnamespace
        LEFT JOIN candidate_namespaces AS n ON n.oid = parser.prsnamespace
        LEFT JOIN protected_system_namespaces AS object_namespace
          ON object_namespace.oid = parser.prsnamespace
        WHERE (n.oid IS NOT NULL OR (
            parser.oid >= 16384 AND object_namespace.oid IS NOT NULL
        ))
          AND NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_ts_parser'::pg_catalog.regclass
              AND extension_dependency.objid = parser.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY catalog_namespace.nspname, parser.prsname, parser.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_ts_template', candidate.oid
    FROM (
        SELECT template.oid
        FROM pg_catalog.pg_ts_template AS template
        JOIN pg_catalog.pg_namespace AS catalog_namespace
          ON catalog_namespace.oid = template.tmplnamespace
        LEFT JOIN candidate_namespaces AS n ON n.oid = template.tmplnamespace
        LEFT JOIN protected_system_namespaces AS object_namespace
          ON object_namespace.oid = template.tmplnamespace
        WHERE (n.oid IS NOT NULL OR (
            template.oid >= 16384 AND object_namespace.oid IS NOT NULL
        ))
          AND NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_ts_template'::pg_catalog.regclass
              AND extension_dependency.objid = template.oid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY catalog_namespace.nspname, template.tmplname, template.oid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_user_mapping', candidate.oid
    FROM (
        SELECT mapping.umid AS oid
        FROM pg_catalog.pg_user_mappings AS mapping
        WHERE NOT EXISTS (
            SELECT 1 FROM pg_catalog.pg_depend AS extension_dependency
            WHERE extension_dependency.classid = 'pg_catalog.pg_user_mapping'::pg_catalog.regclass
              AND extension_dependency.objid = mapping.umid
              AND extension_dependency.deptype = 'e'
        )
        ORDER BY mapping.umid LIMIT $1
    ) AS candidate
    UNION ALL
    SELECT 'pg_roles', candidate.oid
    FROM (
        SELECT role_row.oid
        FROM pg_catalog.pg_roles AS role_row
        WHERE role_row.rolname = '$database_owner'
           OR role_row.rolname = '$superuser'
           OR role_row.rolname = 'ALL'
           OR role_row.rolname = 'PUBLIC'
           OR pg_catalog.starts_with(role_row.rolname, '$other_owner:')
        ORDER BY role_row.rolname, role_row.oid LIMIT $1
    ) AS candidate
),
candidate_extensions AS MATERIALIZED (
    SELECT extension_row.oid
    FROM pg_catalog.pg_extension AS extension_row
    ORDER BY extension_row.extname
    LIMIT $1
),
candidate_extension_members AS MATERIALIZED (
    SELECT
        candidate.oid AS extension_oid,
        dependency.classid,
        dependency.objid,
        dependency.objsubid
    FROM candidate_extensions AS candidate
    JOIN pg_catalog.pg_extension AS extension_row ON extension_row.oid = candidate.oid
    JOIN pg_catalog.pg_depend AS dependency
      ON dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
     AND dependency.refobjid = candidate.oid
     AND dependency.deptype = 'e'
    CROSS JOIN LATERAL pg_catalog.pg_identify_object_as_address(
        dependency.classid,
        dependency.objid,
        dependency.objsubid
    ) AS identified
    ORDER BY
        extension_row.extname,
        identified.type,
        identified.object_names,
        identified.object_args,
        dependency.objsubid
    LIMIT $3
),
extension_member_budget AS MATERIALIZED (
    SELECT pg_catalog.count(*) AS extension_member_count
    FROM candidate_extension_members
),
bounded_extension_members AS MATERIALIZED (
    SELECT member.extension_oid, member.classid, member.objid, member.objsubid
    FROM candidate_extension_members AS member
    CROSS JOIN extension_member_budget AS budget
    WHERE budget.extension_member_count < $3
),
safe_extension_members AS MATERIALIZED (
    SELECT member.extension_oid, member.classid, member.objid, member.objsubid
    FROM bounded_extension_members AS member
    WHERE member.objsubid = 0
      AND member.classid IN (
          'pg_catalog.pg_am'::pg_catalog.regclass,
          'pg_catalog.pg_cast'::pg_catalog.regclass,
          'pg_catalog.pg_class'::pg_catalog.regclass,
          'pg_catalog.pg_language'::pg_catalog.regclass,
          'pg_catalog.pg_opclass'::pg_catalog.regclass,
          'pg_catalog.pg_operator'::pg_catalog.regclass,
          'pg_catalog.pg_opfamily'::pg_catalog.regclass,
          'pg_catalog.pg_proc'::pg_catalog.regclass,
          'pg_catalog.pg_type'::pg_catalog.regclass
      )
),
extension_relation_members AS MATERIALIZED (
    SELECT member.extension_oid, member.objid
    FROM safe_extension_members AS member
    JOIN pg_catalog.pg_class AS relation ON relation.oid = member.objid
    WHERE member.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
),
candidate_schema_acls AS MATERIALIZED (
    SELECT
        n.oid AS namespace_oid,
        selected.grantor,
        selected.grantee,
        selected.privilege_type,
        selected.is_grantable
    FROM candidate_namespaces AS n
    CROSS JOIN database_owner AS own
    CROSS JOIN LATERAL (
        SELECT acl.grantor, acl.grantee, acl.privilege_type, acl.is_grantable
        FROM pg_catalog.aclexplode(coalesce(n.nspacl, pg_catalog.acldefault('n', n.nspowner)))
            AS acl
        ORDER BY
            CASE
                WHEN acl.grantee = 0 THEN 'PUBLIC'
                WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(acl.grantee)
            END,
            acl.privilege_type,
            CASE
                WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(acl.grantor)
            END,
            acl.is_grantable
        LIMIT $1
    ) AS selected
),
governed_parent_relations AS MATERIALIZED (
    SELECT parent.oid, parent_ns.nspname, parent.relname
    FROM governed_parents AS governed
    JOIN pg_catalog.pg_namespace AS parent_ns ON parent_ns.nspname = 'public'
    JOIN pg_catalog.pg_class AS parent
      ON parent.relnamespace = parent_ns.oid
     AND parent.relname = governed.parent_name
     AND parent.relkind = 'p'
),
governed_child_relations AS MATERIALIZED (
    SELECT parent.oid AS parent_oid, children.child_oid
    FROM governed_parent_relations AS parent
    CROSS JOIN LATERAL (
        SELECT inheritance.inhrelid AS child_oid
        FROM pg_catalog.pg_inherits AS inheritance
        JOIN pg_catalog.pg_class AS child ON child.oid = inheritance.inhrelid
        JOIN pg_catalog.pg_namespace AS child_ns ON child_ns.oid = child.relnamespace
        WHERE inheritance.inhparent = parent.oid
        ORDER BY child_ns.nspname, child.relname
        LIMIT $2
    ) AS children
),
candidate_relations AS MATERIALIZED (
    SELECT c.oid
    FROM pg_catalog.pg_class AS c
    JOIN candidate_namespaces AS n ON n.oid = c.relnamespace
    WHERE c.relkind IN ('r', 'p', 'v', 'm', 'S', 'f')
      AND NOT EXISTS (
          SELECT 1
          FROM governed_child_relations AS governed_child
          WHERE governed_child.child_oid = c.oid
      )
      AND NOT EXISTS (
          SELECT 1
          FROM pg_catalog.pg_depend AS dependency
          WHERE dependency.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
            AND dependency.objid = c.oid
            AND dependency.deptype = 'e'
      )
    ORDER BY n.nspname, c.relname, c.oid
    LIMIT $1
),
shape_targets AS MATERIALIZED (
    SELECT candidate.oid FROM candidate_relations AS candidate
    UNION
    SELECT parent.oid FROM governed_parent_relations AS parent
    UNION
    SELECT child.child_oid FROM governed_child_relations AS child
    UNION
    SELECT member.objid FROM extension_relation_members AS member
),
candidate_attributes AS MATERIALIZED (
    SELECT target.oid AS relation_oid, selected.attnum
    FROM shape_targets AS target
    CROSS JOIN LATERAL (
        SELECT attribute.attnum
        FROM pg_catalog.pg_attribute AS attribute
        WHERE attribute.attrelid = target.oid
          AND attribute.attnum > 0
        ORDER BY attribute.attnum
        LIMIT $1
    ) AS selected
),
candidate_attribute_options AS MATERIALIZED (
    SELECT candidate.relation_oid, candidate.attnum, selected.value AS option_value
    FROM candidate_attributes AS candidate
    JOIN pg_catalog.pg_attribute AS attribute
      ON attribute.attrelid = candidate.relation_oid
     AND attribute.attnum = candidate.attnum
    CROSS JOIN LATERAL (
        SELECT option_item.value
        FROM pg_catalog.unnest(coalesce(attribute.attoptions, ARRAY[]::pg_catalog.text[]))
            AS option_item(value)
        ORDER BY option_item.value
        LIMIT $1
    ) AS selected
),
candidate_attribute_fdw_options AS MATERIALIZED (
    SELECT candidate.relation_oid, candidate.attnum, selected.value AS option_value
    FROM candidate_attributes AS candidate
    JOIN pg_catalog.pg_attribute AS attribute
      ON attribute.attrelid = candidate.relation_oid
     AND attribute.attnum = candidate.attnum
    CROSS JOIN LATERAL (
        SELECT option_item.value
        FROM pg_catalog.unnest(coalesce(attribute.attfdwoptions, ARRAY[]::pg_catalog.text[]))
            AS option_item(value)
        ORDER BY option_item.value
        LIMIT $1
    ) AS selected
),
candidate_relation_options AS MATERIALIZED (
    SELECT target.oid AS relation_oid, selected.value AS option_value
    FROM shape_targets AS target
    JOIN pg_catalog.pg_class AS relation ON relation.oid = target.oid
    CROSS JOIN LATERAL (
        SELECT option_item.value
        FROM pg_catalog.unnest(coalesce(relation.reloptions, ARRAY[]::pg_catalog.text[]))
            AS option_item(value)
        ORDER BY option_item.value
        LIMIT $1
    ) AS selected
),
candidate_relation_acls AS MATERIALIZED (
    SELECT
        target.oid AS relation_oid,
        selected.grantor,
        selected.grantee,
        selected.privilege_type,
        selected.is_grantable
    FROM shape_targets AS target
    JOIN pg_catalog.pg_class AS c ON c.oid = target.oid
    CROSS JOIN database_owner AS own
    CROSS JOIN LATERAL (
        SELECT acl.grantor, acl.grantee, acl.privilege_type, acl.is_grantable
        FROM pg_catalog.aclexplode(
            coalesce(
                c.relacl,
                pg_catalog.acldefault(
                    (CASE WHEN c.relkind = 'S' THEN 's' ELSE 'r' END)::pg_catalog."char",
                    c.relowner
                )
            )
        ) AS acl
        ORDER BY
            CASE
                WHEN acl.grantee = 0 THEN 'PUBLIC'
                WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(acl.grantee)
            END,
            acl.privilege_type,
            CASE
                WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(acl.grantor)
            END,
            acl.is_grantable
        LIMIT $1
    ) AS selected
),
candidate_column_acls AS MATERIALIZED (
    SELECT
        candidate.relation_oid,
        candidate.attnum,
        selected.grantor,
        selected.grantee,
        selected.privilege_type,
        selected.is_grantable
    FROM candidate_attributes AS candidate
    JOIN pg_catalog.pg_attribute AS attribute
      ON attribute.attrelid = candidate.relation_oid
     AND attribute.attnum = candidate.attnum
    JOIN pg_catalog.pg_class AS c ON c.oid = candidate.relation_oid
    CROSS JOIN database_owner AS own
    CROSS JOIN LATERAL (
        SELECT acl.grantor, acl.grantee, acl.privilege_type, acl.is_grantable
        FROM pg_catalog.aclexplode(
            coalesce(attribute.attacl, pg_catalog.acldefault('c', c.relowner))
        ) AS acl
        ORDER BY
            CASE
                WHEN acl.grantee = 0 THEN 'PUBLIC'
                WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(acl.grantee)
            END,
            acl.privilege_type,
            CASE
                WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(acl.grantor)
            END,
            acl.is_grantable
        LIMIT $1
    ) AS selected
),
candidate_constraints AS MATERIALIZED (
    SELECT target.oid AS relation_oid, selected.constraint_oid
    FROM shape_targets AS target
    CROSS JOIN LATERAL (
        SELECT constraint_row.oid AS constraint_oid
        FROM pg_catalog.pg_constraint AS constraint_row
        WHERE constraint_row.conrelid = target.oid
        ORDER BY constraint_row.conname, constraint_row.oid
        LIMIT $1
    ) AS selected
),
candidate_policies AS MATERIALIZED (
    SELECT target.oid AS relation_oid, selected.policy_oid
    FROM shape_targets AS target
    CROSS JOIN LATERAL (
        SELECT policy.oid AS policy_oid
        FROM pg_catalog.pg_policy AS policy
        WHERE policy.polrelid = target.oid
        ORDER BY policy.polname, policy.oid
        LIMIT $1
    ) AS selected
),
candidate_policy_roles AS MATERIALIZED (
    SELECT candidate.relation_oid, candidate.policy_oid, selected.role_oid
    FROM candidate_policies AS candidate
    JOIN pg_catalog.pg_policy AS policy ON policy.oid = candidate.policy_oid
    CROSS JOIN LATERAL (
        SELECT role_item.role_oid
        FROM pg_catalog.unnest(policy.polroles::pg_catalog.oid[]) AS role_item(role_oid)
        ORDER BY role_item.role_oid
        LIMIT $1
    ) AS selected
),
candidate_rules AS MATERIALIZED (
    SELECT target.oid AS relation_oid, selected.rule_oid
    FROM shape_targets AS target
    CROSS JOIN LATERAL (
        SELECT rule_row.oid AS rule_oid
        FROM pg_catalog.pg_rewrite AS rule_row
        WHERE rule_row.ev_class = target.oid
        ORDER BY rule_row.rulename, rule_row.oid
        LIMIT $1
    ) AS selected
),
candidate_triggers AS MATERIALIZED (
    SELECT target.oid AS relation_oid, selected.trigger_oid
    FROM shape_targets AS target
    CROSS JOIN LATERAL (
        SELECT trigger_row.oid AS trigger_oid
        FROM pg_catalog.pg_trigger AS trigger_row
        WHERE trigger_row.tgrelid = target.oid
        ORDER BY trigger_row.tgisinternal, trigger_row.tgname, trigger_row.oid
        LIMIT $1
    ) AS selected
),
candidate_indexes AS MATERIALIZED (
    SELECT target.oid AS relation_oid, selected.indexrelid
    FROM shape_targets AS target
    CROSS JOIN LATERAL (
        SELECT index_row.indexrelid
        FROM pg_catalog.pg_index AS index_row
        JOIN pg_catalog.pg_class AS index_class ON index_class.oid = index_row.indexrelid
        JOIN pg_catalog.pg_namespace AS index_ns ON index_ns.oid = index_class.relnamespace
        WHERE index_row.indrelid = target.oid
        ORDER BY index_ns.nspname, index_class.relname, index_row.indexrelid
        LIMIT $1
    ) AS selected
),
candidate_index_options AS MATERIALIZED (
    SELECT candidate.indexrelid AS index_oid, selected.value AS option_value
    FROM candidate_indexes AS candidate
    JOIN pg_catalog.pg_class AS index_class ON index_class.oid = candidate.indexrelid
    CROSS JOIN LATERAL (
        SELECT option_item.value
        FROM pg_catalog.unnest(coalesce(index_class.reloptions, ARRAY[]::pg_catalog.text[]))
            AS option_item(value)
        ORDER BY option_item.value
        LIMIT $1
    ) AS selected
),
candidate_relation_parents AS MATERIALIZED (
    SELECT candidate.oid AS relation_oid, selected.parent_oid
    FROM shape_targets AS candidate
    CROSS JOIN LATERAL (
        SELECT inheritance.inhparent AS parent_oid
        FROM pg_catalog.pg_inherits AS inheritance
        JOIN pg_catalog.pg_class AS parent ON parent.oid = inheritance.inhparent
        JOIN pg_catalog.pg_namespace AS parent_ns ON parent_ns.oid = parent.relnamespace
        WHERE inheritance.inhrelid = candidate.oid
        ORDER BY parent_ns.nspname, parent.relname, parent.oid
        LIMIT $1
    ) AS selected
),
candidate_sequence_dependencies AS MATERIALIZED (
    SELECT candidate.oid AS relation_oid, selected.refobjid, selected.refobjsubid, selected.deptype
    FROM shape_targets AS candidate
    JOIN pg_catalog.pg_class AS sequence_relation
      ON sequence_relation.oid = candidate.oid
     AND sequence_relation.relkind = 'S'
    CROSS JOIN LATERAL (
        SELECT dependency.refobjid, dependency.refobjsubid, dependency.deptype
        FROM pg_catalog.pg_depend AS dependency
        WHERE dependency.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
          AND dependency.objid = candidate.oid
          AND dependency.objsubid = 0
          AND dependency.refclassid = 'pg_catalog.pg_class'::pg_catalog.regclass
          AND dependency.refobjsubid > 0
          AND dependency.deptype IN ('a', 'i')
        ORDER BY dependency.refobjid, dependency.refobjsubid, dependency.deptype
        LIMIT $5
    ) AS selected
),
relation_shapes AS (
    SELECT
        c.oid AS relation_oid,
        pg_catalog.jsonb_build_object(
            'kind', c.relkind,
            'relnatts', c.relnatts,
            'is_partition', c.relispartition,
            'has_subclass', c.relhassubclass,
            'partition_key', CASE
                WHEN c.relkind = 'p' THEN pg_catalog.pg_get_partkeydef(c.oid)
                ELSE ''
            END,
            'columns', coalesce((
                SELECT pg_catalog.jsonb_agg(
                    pg_catalog.jsonb_build_object(
                        'num', a.attnum,
                        'dropped', a.attisdropped
                    ) || CASE WHEN a.attisdropped THEN '{}'::pg_catalog.jsonb
                    ELSE pg_catalog.jsonb_build_object(
                        'name', a.attname,
                        'type', pg_catalog.format_type(a.atttypid, a.atttypmod),
                        'not_null', a.attnotnull,
                        'default', coalesce(pg_catalog.pg_get_expr(ad.adbin, ad.adrelid), ''),
                        'identity', a.attidentity,
                        'generated', a.attgenerated,
                        'dimensions', a.attndims,
                        'statistics_target', a.attstattarget,
                        'storage', a.attstorage,
                        'compression', a.attcompression,
                        'has_missing', a.atthasmissing,
                        'missing_value', pg_catalog.to_jsonb(a.attmissingval),
                        'is_local', a.attislocal,
                        'inheritance_count', a.attinhcount,
                        'options', coalesce((
                            SELECT pg_catalog.jsonb_agg(
                                option_row.option_value ORDER BY option_row.option_value
                            )
                            FROM candidate_attribute_options AS option_row
                            WHERE option_row.relation_oid = c.oid
                              AND option_row.attnum = a.attnum
                        ), '[]'::pg_catalog.jsonb),
                        'fdw_options', coalesce((
                            SELECT pg_catalog.jsonb_agg(
                                option_row.option_value ORDER BY option_row.option_value
                            )
                            FROM candidate_attribute_fdw_options AS option_row
                            WHERE option_row.relation_oid = c.oid
                              AND option_row.attnum = a.attnum
                        ), '[]'::pg_catalog.jsonb),
                        'column_acl', coalesce((
                            SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                                'grantor', CASE
                                    WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                                    ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                                END,
                                'grantee', CASE
                                    WHEN acl.grantee = 0 THEN 'PUBLIC'
                                    WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                                    ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                                END,
                                'privilege', acl.privilege_type,
                                'grantable', acl.is_grantable
                            ) ORDER BY
                                CASE
                                    WHEN acl.grantee = 0 THEN 'PUBLIC'
                                    WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                                    ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                                END,
                                acl.privilege_type,
                                CASE
                                    WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                                    ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                                END,
                                acl.is_grantable)
                            FROM candidate_column_acls AS acl
                            WHERE acl.relation_oid = c.oid
                              AND acl.attnum = a.attnum
                        ), '[]'::pg_catalog.jsonb),
                        'collation', CASE
                            WHEN a.attcollation = 0 THEN ''
                            ELSE pg_catalog.quote_ident(coll_ns.nspname) || '.' ||
                                 pg_catalog.quote_ident(coll.collname)
                        END
                    ) END
                    ORDER BY a.attnum
                )
                FROM candidate_attributes AS candidate_attribute
                JOIN pg_catalog.pg_attribute AS a
                  ON a.attrelid = candidate_attribute.relation_oid
                 AND a.attnum = candidate_attribute.attnum
                LEFT JOIN pg_catalog.pg_attrdef AS ad
                  ON ad.adrelid = a.attrelid AND ad.adnum = a.attnum
                LEFT JOIN pg_catalog.pg_collation AS coll ON coll.oid = a.attcollation
                LEFT JOIN pg_catalog.pg_namespace AS coll_ns ON coll_ns.oid = coll.collnamespace
                WHERE candidate_attribute.relation_oid = c.oid
            ), '[]'::pg_catalog.jsonb),
            'constraints', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'name', con.conname,
                    'kind', con.contype,
                    'deferrable', con.condeferrable,
                    'deferred', con.condeferred,
                    'validated', con.convalidated,
                    'definition', pg_catalog.pg_get_constraintdef(con.oid, true)
                ) ORDER BY con.conname)
                FROM candidate_constraints AS candidate_constraint
                JOIN pg_catalog.pg_constraint AS con
                  ON con.oid = candidate_constraint.constraint_oid
                WHERE candidate_constraint.relation_oid = c.oid
            ), '[]'::pg_catalog.jsonb),
            'partition_constraints', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'kind', con.contype,
                    'deferrable', con.condeferrable,
                    'deferred', con.condeferred,
                    'validated', con.convalidated,
                    'definition', pg_catalog.pg_get_constraintdef(con.oid, true)
                ) ORDER BY
                    con.contype,
                    pg_catalog.pg_get_constraintdef(con.oid, true),
                    con.condeferrable,
                    con.condeferred,
                    con.convalidated)
                FROM candidate_constraints AS candidate_constraint
                JOIN pg_catalog.pg_constraint AS con
                  ON con.oid = candidate_constraint.constraint_oid
                WHERE candidate_constraint.relation_oid = c.oid
            ), '[]'::pg_catalog.jsonb),
            'acl', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'grantor', CASE
                        WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                    END,
                    'grantee', CASE
                        WHEN acl.grantee = 0 THEN 'PUBLIC'
                        WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                    END,
                    'privilege', acl.privilege_type,
                    'grantable', acl.is_grantable
                ) ORDER BY
                    CASE
                        WHEN acl.grantee = 0 THEN 'PUBLIC'
                        WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                    END,
                    acl.privilege_type,
                    CASE
                        WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                    END,
                    acl.is_grantable)
                FROM candidate_relation_acls AS acl
                WHERE acl.relation_oid = c.oid
            ), '[]'::pg_catalog.jsonb),
            'schema', relation_ns.nspname,
            'owner', CASE
                WHEN c.relowner = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(c.relowner)
            END,
            'persistence', c.relpersistence,
            'access_method', coalesce((
                SELECT access_method.amname
                FROM pg_catalog.pg_am AS access_method
                WHERE access_method.oid = c.relam
            ), ''),
            'tablespace', CASE
                WHEN c.reltablespace = 0 THEN ''
                ELSE coalesce((
                    SELECT tablespace.spcname
                    FROM pg_catalog.pg_tablespace AS tablespace
                    WHERE tablespace.oid = c.reltablespace
                ), '')
            END,
            'relation_options', coalesce((
                SELECT pg_catalog.jsonb_agg(option_row.option_value ORDER BY option_row.option_value)
                FROM candidate_relation_options AS option_row
                WHERE option_row.relation_oid = c.oid
            ), '[]'::pg_catalog.jsonb),
            'row_security', c.relrowsecurity,
            'force_row_security', c.relforcerowsecurity,
            'replica_identity', c.relreplident,
            'policies', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'name', policy.polname,
                    'permissive', policy.polpermissive,
                    'command', policy.polcmd,
                    'roles', coalesce((
                        SELECT pg_catalog.jsonb_agg(
                            CASE
                                WHEN role_item.role_oid = 0 THEN 'PUBLIC'
                                ELSE pg_catalog.pg_get_userbyid(role_item.role_oid)
                            END
                            ORDER BY CASE
                                WHEN role_item.role_oid = 0 THEN 'PUBLIC'
                                ELSE pg_catalog.pg_get_userbyid(role_item.role_oid)
                            END
                        )
                        FROM candidate_policy_roles AS role_item
                        WHERE role_item.relation_oid = c.oid
                          AND role_item.policy_oid = policy.oid
                    ), '[]'::pg_catalog.jsonb),
                    'using', coalesce(
                        pg_catalog.pg_get_expr(policy.polqual, policy.polrelid, true),
                        ''
                    ),
                    'check', coalesce(
                        pg_catalog.pg_get_expr(policy.polwithcheck, policy.polrelid, true),
                        ''
                    )
                ) ORDER BY policy.polname)
                FROM candidate_policies AS candidate_policy
                JOIN pg_catalog.pg_policy AS policy ON policy.oid = candidate_policy.policy_oid
                WHERE candidate_policy.relation_oid = c.oid
            ), '[]'::pg_catalog.jsonb),
            'rules', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'name', rule_row.rulename,
                    'event', rule_row.ev_type,
                    'enabled', rule_row.ev_enabled,
                    'instead', rule_row.is_instead,
                    'definition', pg_catalog.pg_get_ruledef(rule_row.oid, true)
                ) ORDER BY rule_row.rulename)
                FROM candidate_rules AS candidate_rule
                JOIN pg_catalog.pg_rewrite AS rule_row ON rule_row.oid = candidate_rule.rule_oid
                WHERE candidate_rule.relation_oid = c.oid
                  AND NOT (
                      c.relkind IN ('v', 'm')
                      AND rule_row.rulename = '_RETURN'
                  )
            ), '[]'::pg_catalog.jsonb),
            'triggers', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'name', trigger_row.tgname,
                    'enabled', trigger_row.tgenabled,
                    'type', trigger_row.tgtype,
                    'arguments', pg_catalog.encode(trigger_row.tgargs, 'hex'),
                    'attributes', trigger_row.tgattr::pg_catalog.text,
                    'qualifier', coalesce(
                        pg_catalog.pg_get_expr(trigger_row.tgqual, trigger_row.tgrelid, true),
                        ''
                    ),
                    'function', pg_catalog.jsonb_build_array(
                        function_ns.nspname,
                        function_row.proname,
                        pg_catalog.pg_get_function_identity_arguments(function_row.oid)
                    ),
                    'old_table', coalesce(trigger_row.tgoldtable, ''),
                    'new_table', coalesce(trigger_row.tgnewtable, '')
                ) ORDER BY trigger_row.tgname)
                FROM candidate_triggers AS candidate_trigger
                JOIN pg_catalog.pg_trigger AS trigger_row
                  ON trigger_row.oid = candidate_trigger.trigger_oid
                JOIN pg_catalog.pg_proc AS function_row
                  ON function_row.oid = trigger_row.tgfoid
                JOIN pg_catalog.pg_namespace AS function_ns
                  ON function_ns.oid = function_row.pronamespace
                WHERE candidate_trigger.relation_oid = c.oid
                  AND NOT trigger_row.tgisinternal
            ), '[]'::pg_catalog.jsonb),
            'internal_trigger_modes', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'enabled', trigger_row.tgenabled,
                    'type', trigger_row.tgtype,
                    'function', pg_catalog.jsonb_build_array(
                        function_ns.nspname,
                        function_row.proname,
                        pg_catalog.pg_get_function_identity_arguments(function_row.oid)
                    )
                ) ORDER BY
                    trigger_row.tgtype,
                    function_ns.nspname,
                    function_row.proname,
                    trigger_row.tgenabled)
                FROM candidate_triggers AS candidate_trigger
                JOIN pg_catalog.pg_trigger AS trigger_row
                  ON trigger_row.oid = candidate_trigger.trigger_oid
                JOIN pg_catalog.pg_proc AS function_row
                  ON function_row.oid = trigger_row.tgfoid
                JOIN pg_catalog.pg_namespace AS function_ns
                  ON function_ns.oid = function_row.pronamespace
                WHERE candidate_trigger.relation_oid = c.oid
                  AND trigger_row.tgisinternal
            ), '[]'::pg_catalog.jsonb)
        ) AS child_shape
    FROM shape_targets AS target
    JOIN pg_catalog.pg_class AS c ON c.oid = target.oid
    JOIN pg_catalog.pg_namespace AS relation_ns ON relation_ns.oid = c.relnamespace
    CROSS JOIN database_owner AS own
),
index_shapes AS (
    SELECT
        index_row.indexrelid AS index_oid,
        pg_catalog.jsonb_build_object(
            'attribute_numbers', index_row.indkey::pg_catalog.text,
            'key_attributes', index_row.indnkeyatts,
            'total_attributes', index_row.indnatts,
            'unique', index_row.indisunique,
            'nulls_not_distinct', index_row.indnullsnotdistinct,
            'primary', index_row.indisprimary,
            'exclusion', index_row.indisexclusion,
            'immediate', index_row.indimmediate,
            'clustered', index_row.indisclustered,
            'valid', index_row.indisvalid,
            'ready', index_row.indisready,
            'live', index_row.indislive,
            'replident', index_row.indisreplident,
            'expressions', coalesce(
                pg_catalog.pg_get_expr(index_row.indexprs, index_row.indrelid, true),
                ''
            ),
            'predicate', coalesce(
                pg_catalog.pg_get_expr(index_row.indpred, index_row.indrelid, true),
                ''
            ),
            'opclasses', coalesce((
                SELECT pg_catalog.jsonb_agg(
                    pg_catalog.quote_ident(opclass_ns.nspname) || '.' ||
                    pg_catalog.quote_ident(opclass.opcname)
                    ORDER BY item.ordinality
                )
                FROM pg_catalog.unnest(index_row.indclass::pg_catalog.oid[]) WITH ORDINALITY
                    AS item(opclass_oid, ordinality)
                JOIN pg_catalog.pg_opclass AS opclass ON opclass.oid = item.opclass_oid
                JOIN pg_catalog.pg_namespace AS opclass_ns
                  ON opclass_ns.oid = opclass.opcnamespace
            ), '[]'::pg_catalog.jsonb),
            'collations', coalesce((
                SELECT pg_catalog.jsonb_agg(
                    CASE
                        WHEN item.collation_oid = 0 THEN ''
                        ELSE pg_catalog.quote_ident(collation_ns.nspname) || '.' ||
                             pg_catalog.quote_ident(collation_row.collname)
                    END
                    ORDER BY item.ordinality
                )
                FROM pg_catalog.unnest(index_row.indcollation::pg_catalog.oid[]) WITH ORDINALITY
                    AS item(collation_oid, ordinality)
                LEFT JOIN pg_catalog.pg_collation AS collation_row
                  ON collation_row.oid = item.collation_oid
                LEFT JOIN pg_catalog.pg_namespace AS collation_ns
                  ON collation_ns.oid = collation_row.collnamespace
            ), '[]'::pg_catalog.jsonb),
            'options', index_row.indoption::pg_catalog.text,
            'access_method', access_method.amname,
            'relation_options', coalesce((
                SELECT pg_catalog.jsonb_agg(option_row.option_value ORDER BY option_row.option_value)
                FROM candidate_index_options AS option_row
                WHERE option_row.index_oid = index_row.indexrelid
            ), '[]'::pg_catalog.jsonb),
            'tablespace', CASE
                WHEN index_class.reltablespace = 0 THEN ''
                ELSE coalesce(tablespace.spcname, '')
            END
        ) AS index_shape
    FROM candidate_indexes AS candidate
    JOIN pg_catalog.pg_index AS index_row ON index_row.indexrelid = candidate.indexrelid
    JOIN pg_catalog.pg_class AS index_class ON index_class.oid = index_row.indexrelid
    JOIN pg_catalog.pg_am AS access_method ON access_method.oid = index_class.relam
    LEFT JOIN pg_catalog.pg_tablespace AS tablespace ON tablespace.oid = index_class.reltablespace
),
partition_children AS (
    SELECT
        parent.oid AS parent_oid,
        parent.nspname AS parent_schema,
        parent.relname AS parent_name,
        child.oid AS child_oid,
        child_ns.nspname AS child_schema,
        child.relname AS child_name,
        pg_catalog.pg_get_expr(child.relpartbound, child.oid) AS child_bound,
        child_shape.child_shape - 'constraints' AS partition_shape,
        coalesce((
            SELECT pg_catalog.jsonb_agg(
                child_index_shape.index_shape
                ORDER BY child_index_shape.index_shape::pg_catalog.text
            )
            FROM candidate_indexes AS child_index
            JOIN index_shapes AS child_index_shape
              ON child_index_shape.index_oid = child_index.indexrelid
            WHERE child_index.relation_oid = child.oid
        ), '[]'::pg_catalog.jsonb) AS child_indexes,
        (
            SELECT pg_catalog.count(*)
            FROM candidate_indexes AS parent_index
            WHERE parent_index.relation_oid = parent.oid
        ) AS parent_index_count,
        (
            SELECT pg_catalog.count(*)
            FROM candidate_indexes AS child_index
            WHERE child_index.relation_oid = child.oid
        ) AS child_index_count,
        NOT EXISTS (
            SELECT 1
            FROM candidate_indexes AS parent_candidate
            JOIN pg_catalog.pg_index AS parent_index
              ON parent_index.indexrelid = parent_candidate.indexrelid
            JOIN index_shapes AS parent_index_shape
              ON parent_index_shape.index_oid = parent_index.indexrelid
            WHERE parent_candidate.relation_oid = parent.oid
              AND NOT EXISTS (
                  SELECT 1
                  FROM pg_catalog.pg_inherits AS index_link
                  JOIN pg_catalog.pg_index AS child_index
                    ON child_index.indexrelid = index_link.inhrelid
                  JOIN candidate_indexes AS child_candidate
                    ON child_candidate.indexrelid = child_index.indexrelid
                   AND child_candidate.relation_oid = child.oid
                  JOIN index_shapes AS child_index_shape
                    ON child_index_shape.index_oid = child_index.indexrelid
                  WHERE index_link.inhparent = parent_index.indexrelid
                    AND child_index.indrelid = child.oid
                    AND child_index_shape.index_shape = parent_index_shape.index_shape
              )
        ) AS attached_indexes_valid
    FROM governed_parent_relations AS parent
    LEFT JOIN governed_child_relations AS governed_child
      ON governed_child.parent_oid = parent.oid
    LEFT JOIN pg_catalog.pg_class AS child ON child.oid = governed_child.child_oid
    LEFT JOIN pg_catalog.pg_namespace AS child_ns ON child_ns.oid = child.relnamespace
    LEFT JOIN relation_shapes AS child_shape ON child_shape.relation_oid = child.oid
),
partition_defaults AS (
    SELECT
        children.parent_oid,
        children.partition_shape AS default_shape,
        children.child_indexes AS default_indexes
    FROM partition_children AS children
    WHERE children.child_name = children.parent_name || '_default'
      AND children.child_bound = 'DEFAULT'
),
partition_validity AS (
    SELECT
        children.parent_oid,
        pg_catalog.count(children.child_oid) < $2
        AND pg_catalog.count(children.child_oid) FILTER (
            WHERE children.child_name = children.parent_name || '_default'
              AND children.child_bound = 'DEFAULT'
        ) = 1
        AND coalesce(pg_catalog.bool_and(
            children.child_oid IS NOT NULL
            AND children.child_schema = children.parent_schema
            AND children.partition_shape = defaults.default_shape
            AND children.child_indexes = defaults.default_indexes
            AND children.parent_index_count = children.child_index_count
            AND children.attached_indexes_valid
            AND (
                (
                    children.child_name = children.parent_name || '_default'
                    AND children.child_bound = 'DEFAULT'
                )
                OR (
                    children.child_name ~ ('^' || children.parent_name || '_p_[0-9a-f]{32}$')
                    AND children.child_bound = pg_catalog.format(
                        'FOR VALUES IN (''%s-%s-%s-%s-%s'')',
                        pg_catalog.substr(children.child_name, pg_catalog.length(children.parent_name) + 4, 8),
                        pg_catalog.substr(children.child_name, pg_catalog.length(children.parent_name) + 12, 4),
                        pg_catalog.substr(children.child_name, pg_catalog.length(children.parent_name) + 16, 4),
                        pg_catalog.substr(children.child_name, pg_catalog.length(children.parent_name) + 20, 4),
                        pg_catalog.substr(children.child_name, pg_catalog.length(children.parent_name) + 24, 12)
                    )
                )
            )
        ), false) AS children_valid,
        pg_catalog.jsonb_build_object(
            'shape', defaults.default_shape,
            'indexes', defaults.default_indexes
        ) AS default_partition_template
    FROM partition_children AS children
    LEFT JOIN partition_defaults AS defaults ON defaults.parent_oid = children.parent_oid
    GROUP BY children.parent_oid, defaults.default_shape, defaults.default_indexes
),
relation_payloads AS (
    SELECT
        c.oid AS relation_oid,
        CASE c.relkind
            WHEN 'r' THEN 'relation'
            WHEN 'p' THEN 'partitioned_table'
            WHEN 'v' THEN 'view'
            WHEN 'm' THEN 'materialized_view'
            WHEN 'S' THEN 'sequence'
            WHEN 'f' THEN 'foreign_table'
            ELSE c.relkind::pg_catalog.text
        END AS kind,
        n.nspname AS schema_name,
        c.relname AS object_name,
        pg_catalog.jsonb_build_object(
            'census_version', 1,
            'object', pg_catalog.jsonb_build_array(n.nspname, c.relname),
            'kind', c.relkind,
            'owner', CASE
                WHEN c.relowner = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(c.relowner)
            END,
            'acl', shape.child_shape -> 'acl',
            'persistence', c.relpersistence,
            'relation_options', coalesce((
                SELECT pg_catalog.jsonb_agg(option_row.option_value ORDER BY option_row.option_value)
                FROM candidate_relation_options AS option_row
                WHERE option_row.relation_oid = c.oid
            ), '[]'::pg_catalog.jsonb),
            'row_security', c.relrowsecurity,
            'force_row_security', c.relforcerowsecurity,
            'replica_identity', c.relreplident,
            'has_subclass', c.relhassubclass,
            'is_partition', c.relispartition,
            'partition_bound', coalesce(pg_catalog.pg_get_expr(c.relpartbound, c.oid), ''),
            'partition_key', shape.child_shape -> 'partition_key',
            'partition_children_valid', CASE
                WHEN c.relkind = 'p' THEN coalesce(validity.children_valid, false)
                ELSE true
            END,
            'default_partition_template', CASE
                WHEN c.relkind = 'p' THEN coalesce(
                    validity.default_partition_template,
                    '{}'::pg_catalog.jsonb
                )
                ELSE '{}'::pg_catalog.jsonb
            END,
            'relnatts', shape.child_shape -> 'relnatts',
            'columns', shape.child_shape -> 'columns',
            'constraints', shape.child_shape -> 'constraints',
            'policies', shape.child_shape -> 'policies',
            'rules', shape.child_shape -> 'rules',
            'internal_trigger_modes', shape.child_shape -> 'internal_trigger_modes',
            'access_method', shape.child_shape -> 'access_method',
            'tablespace', shape.child_shape -> 'tablespace',
            'indexes', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'schema', index_ns.nspname,
                    'name', index_class.relname,
                    'shape', index_shape.index_shape
                ) ORDER BY index_ns.nspname, index_class.relname)
                FROM candidate_indexes AS candidate_index
                JOIN pg_catalog.pg_class AS index_class
                  ON index_class.oid = candidate_index.indexrelid
                JOIN pg_catalog.pg_namespace AS index_ns
                  ON index_ns.oid = index_class.relnamespace
                JOIN index_shapes AS index_shape
                  ON index_shape.index_oid = candidate_index.indexrelid
                WHERE candidate_index.relation_oid = c.oid
            ), '[]'::pg_catalog.jsonb),
            'view_definition', CASE
                WHEN c.relkind IN ('v', 'm')
                THEN coalesce(pg_catalog.pg_get_viewdef(c.oid, true), '')
                ELSE ''
            END,
            'triggers', shape.child_shape -> 'triggers',
            'parents', coalesce((
                SELECT pg_catalog.jsonb_agg(
                    pg_catalog.jsonb_build_array(parent_ns.nspname, parent.relname)
                    ORDER BY parent_ns.nspname, parent.relname
                )
                FROM candidate_relation_parents AS candidate_parent
                JOIN pg_catalog.pg_class AS parent ON parent.oid = candidate_parent.parent_oid
                JOIN pg_catalog.pg_namespace AS parent_ns ON parent_ns.oid = parent.relnamespace
                WHERE candidate_parent.relation_oid = c.oid
            ), '[]'::pg_catalog.jsonb),
            'sequence', coalesce((
                SELECT pg_catalog.jsonb_build_object(
                    'start', sequence_row.seqstart,
                    'increment', sequence_row.seqincrement,
                    'max', sequence_row.seqmax,
                    'min', sequence_row.seqmin,
                    'cache', sequence_row.seqcache,
                    'cycle', sequence_row.seqcycle,
                    'type', pg_catalog.format_type(sequence_row.seqtypid, NULL),
                    'owned_by', (
                        SELECT pg_catalog.jsonb_build_object(
                            'schema', owned_namespace.nspname,
                            'relation', owned_relation.relname,
                            'column', owned_column.attname,
                            'dependency_type', dependency.deptype
                        )
                        FROM candidate_sequence_dependencies AS dependency
                        JOIN pg_catalog.pg_class AS owned_relation
                          ON owned_relation.oid = dependency.refobjid
                        JOIN pg_catalog.pg_namespace AS owned_namespace
                          ON owned_namespace.oid = owned_relation.relnamespace
                        JOIN pg_catalog.pg_attribute AS owned_column
                          ON owned_column.attrelid = dependency.refobjid
                         AND owned_column.attnum = dependency.refobjsubid
                        WHERE dependency.relation_oid = c.oid
                        ORDER BY
                            owned_namespace.nspname,
                            owned_relation.relname,
                            owned_column.attname,
                            dependency.deptype
                        LIMIT 1
                    )
                )
                FROM pg_catalog.pg_sequence AS sequence_row
                WHERE sequence_row.seqrelid = c.oid
            ), '{}'::pg_catalog.jsonb)
        ) AS payload
    FROM shape_targets AS candidate
    JOIN pg_catalog.pg_class AS c ON c.oid = candidate.oid
    JOIN pg_catalog.pg_namespace AS n ON n.oid = c.relnamespace
    CROSS JOIN database_owner AS own
    JOIN relation_shapes AS shape ON shape.relation_oid = c.oid
    LEFT JOIN partition_validity AS validity ON validity.parent_oid = c.oid
),
rel_objects AS (
    SELECT payload.kind, payload.schema_name, payload.object_name, payload.payload
    FROM candidate_relations AS candidate
    JOIN relation_payloads AS payload ON payload.relation_oid = candidate.oid
),
candidate_domains AS MATERIALIZED (
    SELECT domain_type.oid
    FROM pg_catalog.pg_type AS domain_type
    JOIN candidate_namespaces AS n ON n.oid = domain_type.typnamespace
    WHERE domain_type.typtype = 'd'
      AND NOT EXISTS (
          SELECT 1
          FROM pg_catalog.pg_depend AS dependency
          WHERE dependency.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
            AND dependency.objid = domain_type.oid
            AND dependency.deptype = 'e'
      )
    ORDER BY n.nspname, domain_type.typname, domain_type.oid
    LIMIT $1
),
domain_targets AS MATERIALIZED (
    SELECT candidate.oid FROM candidate_domains AS candidate
    UNION
    SELECT member.objid
    FROM safe_extension_members AS member
    JOIN pg_catalog.pg_type AS domain_type
      ON domain_type.oid = member.objid
     AND domain_type.typtype = 'd'
    WHERE member.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
),
candidate_domain_acls AS MATERIALIZED (
    SELECT
        candidate.oid AS domain_oid,
        selected.grantor,
        selected.grantee,
        selected.privilege_type,
        selected.is_grantable
    FROM domain_targets AS candidate
    JOIN pg_catalog.pg_type AS domain_type ON domain_type.oid = candidate.oid
    CROSS JOIN database_owner AS own
    CROSS JOIN LATERAL (
        SELECT acl.grantor, acl.grantee, acl.privilege_type, acl.is_grantable
        FROM pg_catalog.aclexplode(
            coalesce(domain_type.typacl, pg_catalog.acldefault('T', domain_type.typowner))
        ) AS acl
        ORDER BY
            CASE
                WHEN acl.grantee = 0 THEN 'PUBLIC'
                WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(acl.grantee)
            END,
            acl.privilege_type,
            CASE
                WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(acl.grantor)
            END,
            acl.is_grantable
        LIMIT $1
    ) AS selected
),
candidate_domain_constraints AS MATERIALIZED (
    SELECT candidate.oid AS domain_oid, selected.constraint_oid
    FROM domain_targets AS candidate
    CROSS JOIN LATERAL (
        SELECT constraint_row.oid AS constraint_oid
        FROM pg_catalog.pg_constraint AS constraint_row
        WHERE constraint_row.contypid = candidate.oid
        ORDER BY constraint_row.conname, constraint_row.oid
        LIMIT $1
    ) AS selected
),
domain_payloads AS (
    SELECT
        domain_type.oid AS domain_oid,
        'domain' AS kind,
        n.nspname AS schema_name,
        domain_type.typname AS object_name,
        pg_catalog.jsonb_build_object(
            'census_version', 1,
            'object', pg_catalog.jsonb_build_array(n.nspname, domain_type.typname),
            'kind', 'domain',
            'owner', CASE
                WHEN domain_type.typowner = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(domain_type.typowner)
            END,
            'acl', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'grantor', CASE
                        WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                    END,
                    'grantee', CASE
                        WHEN acl.grantee = 0 THEN 'PUBLIC'
                        WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                    END,
                    'privilege', acl.privilege_type,
                    'grantable', acl.is_grantable
                ) ORDER BY
                    CASE
                        WHEN acl.grantee = 0 THEN 'PUBLIC'
                        WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                    END,
                    acl.privilege_type,
                    CASE
                        WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                    END,
                    acl.is_grantable)
                FROM candidate_domain_acls AS acl
                WHERE acl.domain_oid = domain_type.oid
            ), '[]'::pg_catalog.jsonb),
            'base_type', pg_catalog.format_type(domain_type.typbasetype, domain_type.typtypmod),
            'not_null', domain_type.typnotnull,
            'default', coalesce(pg_catalog.pg_get_expr(domain_type.typdefaultbin, 0), ''),
            'constraints', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'name', con.conname,
                    'definition', pg_catalog.pg_get_constraintdef(con.oid, true),
                    'validated', con.convalidated
                ) ORDER BY con.conname)
                FROM candidate_domain_constraints AS candidate_constraint
                JOIN pg_catalog.pg_constraint AS con
                  ON con.oid = candidate_constraint.constraint_oid
                WHERE candidate_constraint.domain_oid = domain_type.oid
            ), '[]'::pg_catalog.jsonb)
        ) AS payload
    FROM domain_targets AS candidate
    JOIN pg_catalog.pg_type AS domain_type ON domain_type.oid = candidate.oid
    JOIN pg_catalog.pg_namespace AS n ON n.oid = domain_type.typnamespace
    CROSS JOIN database_owner AS own
),
domain_objects AS (
    SELECT payload.kind, payload.schema_name, payload.object_name, payload.payload
    FROM candidate_domains AS candidate
    JOIN domain_payloads AS payload ON payload.domain_oid = candidate.oid
),
database_objects AS (
    SELECT
        'database' AS kind,
        'pg_database' AS schema_name,
        'current_database' AS object_name,
        pg_catalog.jsonb_build_object(
            'census_version', 1,
            'environment_version', 1,
            'database_identity', '$current_database',
            'server_version_num',
                pg_catalog.current_setting('server_version_num')::pg_catalog.int4,
            'encoding', pg_catalog.pg_encoding_to_char(database_row.encoding),
            'locale_provider', database_row.datlocprovider,
            'collate', database_row.datcollate,
            'ctype', database_row.datctype,
            'locale', coalesce(database_row.datlocale, ''),
            'icu_rules', coalesce(database_row.daticurules, ''),
            'recorded_collation_version', coalesce(database_row.datcollversion, ''),
            'actual_collation_version', coalesce(
                pg_catalog.pg_database_collation_actual_version(database_row.oid),
                ''
            ),
            'tablespace', tablespace.spcname,
            'owner', '$database_owner',
            'owner_is_babylon_intel',
                pg_catalog.pg_get_userbyid(database_row.datdba) = 'babylon_intel',
            'is_template', database_row.datistemplate,
            'allow_connections', database_row.datallowconn,
            'has_login_event_triggers', database_row.dathasloginevt,
            'connection_limit', database_row.datconnlimit,
            'acl', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'grantor', CASE
                        WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                    END,
                    'grantee', CASE
                        WHEN acl.grantee = 0 THEN 'PUBLIC'
                        WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                    END,
                    'privilege', acl.privilege_type,
                    'grantable', acl.is_grantable
                ) ORDER BY
                    CASE
                        WHEN acl.grantee = 0 THEN 'PUBLIC'
                        WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                    END,
                    acl.privilege_type,
                    CASE
                        WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                    END,
                    acl.is_grantable)
                FROM candidate_database_acls AS acl
            ), '[]'::pg_catalog.jsonb),
            'role_settings', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'role', CASE
                        WHEN settings.setrole = 0 THEN 'ALL'
                        WHEN settings.setrole = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(settings.setrole)
                    END,
                    'config', coalesce((
                        SELECT pg_catalog.jsonb_agg(config_item.value ORDER BY config_item.value)
                        FROM candidate_database_setting_configs AS config_item
                        WHERE config_item.setrole = settings.setrole
                    ), '[]'::pg_catalog.jsonb)
                ) ORDER BY CASE
                    WHEN settings.setrole = 0 THEN 'ALL'
                    WHEN settings.setrole = own.owner_oid THEN '$database_owner'
                    ELSE pg_catalog.pg_get_userbyid(settings.setrole)
                END)
                FROM database_settings AS settings
            ), '[]'::pg_catalog.jsonb),
            'role_settings_complete',
                (SELECT pg_catalog.count(*) FROM database_settings) < $1
        ) AS payload
    FROM current_database_row AS database_row
    JOIN pg_catalog.pg_tablespace AS tablespace ON tablespace.oid = database_row.dattablespace
    CROSS JOIN database_owner AS own
),
candidate_extension_configs AS MATERIALIZED (
    SELECT
        candidate.oid AS extension_oid,
        selected.config_oid,
        selected.condition
    FROM candidate_extensions AS candidate
    JOIN pg_catalog.pg_extension AS extension_row ON extension_row.oid = candidate.oid
    CROSS JOIN LATERAL (
        SELECT
            config_item.config_oid,
            coalesce(extension_row.extcondition[config_item.ordinality], '') AS condition
        FROM pg_catalog.unnest(
            coalesce(extension_row.extconfig, ARRAY[]::pg_catalog.oid[])
        ) WITH ORDINALITY AS config_item(config_oid, ordinality)
        ORDER BY
            config_item.config_oid,
            coalesce(extension_row.extcondition[config_item.ordinality], '')
        LIMIT $1
    ) AS selected
),
extension_config_identities AS (
    SELECT
        config.extension_oid,
        config.condition,
        pg_catalog.jsonb_build_object(
            'type', identified.type,
            'names', coalesce(pg_catalog.to_jsonb(identified.object_names), '[]'::pg_catalog.jsonb),
            'args', coalesce(pg_catalog.to_jsonb(identified.object_args), '[]'::pg_catalog.jsonb)
        ) AS address
    FROM candidate_extension_configs AS config
    CROSS JOIN LATERAL pg_catalog.pg_identify_object_as_address(
        'pg_catalog.pg_class'::pg_catalog.regclass,
        config.config_oid,
        0
    ) AS identified
),
schema_objects AS (
    SELECT
        'schema' AS kind,
        'pg_namespace' AS schema_name,
        n.nspname AS object_name,
        pg_catalog.jsonb_build_object(
            'census_version', 1,
            'schema', n.nspname,
            'owner', CASE
                WHEN n.nspowner = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(n.nspowner)
            END,
            'acl', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'grantor', CASE
                        WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                    END,
                    'grantee', CASE
                        WHEN acl.grantee = 0 THEN 'PUBLIC'
                        WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                    END,
                    'privilege', acl.privilege_type,
                    'grantable', acl.is_grantable
                ) ORDER BY
                    CASE
                        WHEN acl.grantee = 0 THEN 'PUBLIC'
                        WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                    END,
                    acl.privilege_type,
                    CASE
                        WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                    END,
                    acl.is_grantable)
                FROM candidate_schema_acls AS acl
                WHERE acl.namespace_oid = n.oid
            ), '[]'::pg_catalog.jsonb)
        ) AS payload
    FROM candidate_namespaces AS n
    CROSS JOIN database_owner AS own
    WHERE n.nspname NOT IN ('public', 'babylon_meta')
      AND NOT EXISTS (
          SELECT 1
          FROM pg_catalog.pg_depend AS dependency
          WHERE dependency.classid = 'pg_catalog.pg_namespace'::pg_catalog.regclass
            AND dependency.objid = n.oid
            AND dependency.deptype = 'e'
      )
),
schema_grants AS (
    SELECT
        'schema_grant' AS kind,
        'pg_namespace' AS schema_name,
        n.nspname AS object_name,
        pg_catalog.jsonb_build_object(
            'census_version', 1,
            'schema', n.nspname,
            'owner', CASE
                WHEN n.nspowner = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(n.nspowner)
            END,
            'acl', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'grantor', CASE
                        WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                    END,
                    'grantee', CASE
                        WHEN acl.grantee = 0 THEN 'PUBLIC'
                        WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                    END,
                    'privilege', acl.privilege_type,
                    'grantable', acl.is_grantable
                ) ORDER BY
                    CASE
                        WHEN acl.grantee = 0 THEN 'PUBLIC'
                        WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                    END,
                    acl.privilege_type,
                    CASE
                        WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                    END,
                    acl.is_grantable)
                FROM candidate_schema_acls AS acl
                WHERE acl.namespace_oid = n.oid
            ), '[]'::pg_catalog.jsonb)
        ) AS payload
    FROM candidate_namespaces AS n
    CROSS JOIN database_owner AS own
    WHERE n.nspname IN ('public', 'babylon_meta')
),
candidate_routines AS MATERIALIZED (
    SELECT routine.oid
    FROM pg_catalog.pg_proc AS routine
    JOIN candidate_namespaces AS n ON n.oid = routine.pronamespace
    WHERE NOT EXISTS (
        SELECT 1
        FROM pg_catalog.pg_depend AS dependency
        WHERE dependency.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
          AND dependency.objid = routine.oid
          AND dependency.deptype = 'e'
    )
    ORDER BY
        n.nspname,
        routine.proname,
        pg_catalog.pg_get_function_identity_arguments(routine.oid),
        routine.oid
    LIMIT $1
),
routine_targets AS MATERIALIZED (
    SELECT candidate.oid FROM candidate_routines AS candidate
    UNION
    SELECT member.objid
    FROM safe_extension_members AS member
    JOIN pg_catalog.pg_proc AS routine ON routine.oid = member.objid
    WHERE member.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
),
candidate_routine_configs AS MATERIALIZED (
    SELECT candidate.oid AS routine_oid, selected.value
    FROM routine_targets AS candidate
    JOIN pg_catalog.pg_proc AS routine ON routine.oid = candidate.oid
    CROSS JOIN LATERAL (
        SELECT config_item.value
        FROM pg_catalog.unnest(coalesce(routine.proconfig, ARRAY[]::pg_catalog.text[]))
            AS config_item(value)
        ORDER BY config_item.value
        LIMIT $1
    ) AS selected
),
candidate_routine_acls AS MATERIALIZED (
    SELECT
        candidate.oid AS routine_oid,
        selected.grantor,
        selected.grantee,
        selected.privilege_type,
        selected.is_grantable
    FROM routine_targets AS candidate
    JOIN pg_catalog.pg_proc AS routine ON routine.oid = candidate.oid
    CROSS JOIN database_owner AS own
    CROSS JOIN LATERAL (
        SELECT acl.grantor, acl.grantee, acl.privilege_type, acl.is_grantable
        FROM pg_catalog.aclexplode(
            coalesce(routine.proacl, pg_catalog.acldefault('f', routine.proowner))
        ) AS acl
        ORDER BY
            CASE
                WHEN acl.grantee = 0 THEN 'PUBLIC'
                WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(acl.grantee)
            END,
            acl.privilege_type,
            CASE
                WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(acl.grantor)
            END,
            acl.is_grantable
        LIMIT $1
    ) AS selected
),
routine_overloads AS (
    SELECT
        routine.oid AS routine_oid,
        routine_ns.nspname AS schema_name,
        routine.proname AS object_name,
        pg_catalog.jsonb_build_object(
            'identity_arguments', pg_catalog.pg_get_function_identity_arguments(routine.oid),
            'arguments', pg_catalog.pg_get_function_arguments(routine.oid),
            'result', coalesce(pg_catalog.pg_get_function_result(routine.oid), ''),
            'kind', routine.prokind,
            'owner', CASE
                WHEN routine.proowner = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(routine.proowner)
            END,
            'language', language.lanname,
            'cost', routine.procost,
            'rows', routine.prorows,
            'volatility', routine.provolatile,
            'parallel', routine.proparallel,
            'security_definer', routine.prosecdef,
            'leakproof', routine.proleakproof,
            'strict', routine.proisstrict,
            'returns_set', routine.proretset,
            'config', coalesce((
                SELECT pg_catalog.jsonb_agg(config_item.value ORDER BY config_item.value)
                FROM candidate_routine_configs AS config_item
                WHERE config_item.routine_oid = routine.oid
            ), '[]'::pg_catalog.jsonb),
            'acl', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'grantor', CASE
                        WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                    END,
                    'grantee', CASE
                        WHEN acl.grantee = 0 THEN 'PUBLIC'
                        WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                    END,
                    'privilege', acl.privilege_type,
                    'grantable', acl.is_grantable
                ) ORDER BY
                    CASE
                        WHEN acl.grantee = 0 THEN 'PUBLIC'
                        WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                    END,
                    acl.privilege_type,
                    CASE
                        WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                    END,
                    acl.is_grantable)
                FROM candidate_routine_acls AS acl
                WHERE acl.routine_oid = routine.oid
            ), '[]'::pg_catalog.jsonb),
            'implementation', pg_catalog.jsonb_build_object(
                'source', routine.prosrc,
                'binary', coalesce(routine.probin, ''),
                'sql_body', CASE
                    WHEN routine.prosqlbody IS NULL THEN ''
                    ELSE pg_catalog.pg_get_functiondef(routine.oid)
                END
            ),
            'definition', CASE
                WHEN routine.prokind IN ('f', 'p', 'w')
                THEN pg_catalog.pg_get_functiondef(routine.oid)
                ELSE ''
            END
        ) AS overload
    FROM routine_targets AS candidate
    JOIN pg_catalog.pg_proc AS routine ON routine.oid = candidate.oid
    JOIN pg_catalog.pg_namespace AS routine_ns ON routine_ns.oid = routine.pronamespace
    JOIN pg_catalog.pg_language AS language ON language.oid = routine.prolang
    CROSS JOIN database_owner AS own
),
routine_objects AS (
    SELECT
        'routine' AS kind,
        routine.schema_name,
        routine.object_name,
        pg_catalog.jsonb_build_object(
            'census_version', 1,
            'object', pg_catalog.jsonb_build_array(routine.schema_name, routine.object_name),
            'overloads', pg_catalog.jsonb_agg(
                routine.overload
                ORDER BY routine.overload ->> 'identity_arguments'
            )
        ) AS payload
    FROM routine_overloads AS routine
    JOIN candidate_routines AS candidate ON candidate.oid = routine.routine_oid
    GROUP BY routine.schema_name, routine.object_name
),
candidate_user_types AS MATERIALIZED (
    SELECT user_type.oid
    FROM pg_catalog.pg_type AS user_type
    JOIN candidate_namespaces AS n ON n.oid = user_type.typnamespace
    LEFT JOIN pg_catalog.pg_class AS type_relation ON type_relation.oid = user_type.typrelid
    WHERE user_type.typtype <> 'd'
      AND coalesce(type_relation.relkind, 'c') NOT IN ('r', 'p', 'v', 'm', 'f')
      AND NOT EXISTS (
          SELECT 1
          FROM pg_catalog.pg_type AS base_type
          WHERE base_type.typarray = user_type.oid
      )
      AND NOT EXISTS (
          SELECT 1
          FROM pg_catalog.pg_depend AS dependency
          WHERE dependency.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
            AND dependency.objid = user_type.oid
            AND dependency.deptype = 'e'
      )
    ORDER BY n.nspname, user_type.typname, user_type.oid
    LIMIT $1
),
user_type_targets AS MATERIALIZED (
    SELECT candidate.oid FROM candidate_user_types AS candidate
    UNION
    SELECT member.objid
    FROM safe_extension_members AS member
    JOIN pg_catalog.pg_type AS user_type
      ON user_type.oid = member.objid
     AND user_type.typtype <> 'd'
    WHERE member.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
),
candidate_user_type_acls AS MATERIALIZED (
    SELECT
        candidate.oid AS type_oid,
        selected.grantor,
        selected.grantee,
        selected.privilege_type,
        selected.is_grantable
    FROM user_type_targets AS candidate
    JOIN pg_catalog.pg_type AS user_type ON user_type.oid = candidate.oid
    CROSS JOIN database_owner AS own
    CROSS JOIN LATERAL (
        SELECT acl.grantor, acl.grantee, acl.privilege_type, acl.is_grantable
        FROM pg_catalog.aclexplode(
            coalesce(user_type.typacl, pg_catalog.acldefault('T', user_type.typowner))
        ) AS acl
        ORDER BY
            CASE
                WHEN acl.grantee = 0 THEN 'PUBLIC'
                WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(acl.grantee)
            END,
            acl.privilege_type,
            CASE
                WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(acl.grantor)
            END,
            acl.is_grantable
        LIMIT $1
    ) AS selected
),
candidate_enum_labels AS MATERIALIZED (
    SELECT candidate.oid AS type_oid, selected.enum_oid
    FROM user_type_targets AS candidate
    CROSS JOIN LATERAL (
        SELECT enum_row.oid AS enum_oid
        FROM pg_catalog.pg_enum AS enum_row
        WHERE enum_row.enumtypid = candidate.oid
        ORDER BY enum_row.enumsortorder, enum_row.oid
        LIMIT $1
    ) AS selected
),
candidate_composite_attributes AS MATERIALIZED (
    SELECT candidate.oid AS type_oid, selected.attnum
    FROM user_type_targets AS candidate
    JOIN pg_catalog.pg_type AS user_type ON user_type.oid = candidate.oid
    CROSS JOIN LATERAL (
        SELECT attribute.attnum
        FROM pg_catalog.pg_attribute AS attribute
        WHERE attribute.attrelid = user_type.typrelid
          AND attribute.attnum > 0
        ORDER BY attribute.attnum
        LIMIT $1
    ) AS selected
),
user_type_payloads AS (
    SELECT
        user_type.oid AS type_oid,
        'user_type' AS kind,
        type_ns.nspname AS schema_name,
        user_type.typname AS object_name,
        pg_catalog.jsonb_build_object(
            'census_version', 1,
            'object', pg_catalog.jsonb_build_array(type_ns.nspname, user_type.typname),
            'kind', user_type.typtype,
            'relnatts', coalesce(type_relation.relnatts, 0),
            'category', user_type.typcategory,
            'preferred', user_type.typispreferred,
            'defined', user_type.typisdefined,
            'delimiter', user_type.typdelim,
            'length', user_type.typlen,
            'by_value', user_type.typbyval,
            'alignment', user_type.typalign,
            'storage', user_type.typstorage,
            'owner', CASE
                WHEN user_type.typowner = own.owner_oid THEN '$database_owner'
                ELSE pg_catalog.pg_get_userbyid(user_type.typowner)
            END,
            'acl', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'grantor', CASE
                        WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                    END,
                    'grantee', CASE
                        WHEN acl.grantee = 0 THEN 'PUBLIC'
                        WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                    END,
                    'privilege', acl.privilege_type,
                    'grantable', acl.is_grantable
                ) ORDER BY
                    CASE
                        WHEN acl.grantee = 0 THEN 'PUBLIC'
                        WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantee)
                    END,
                    acl.privilege_type,
                    CASE
                        WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(acl.grantor)
                    END,
                    acl.is_grantable)
                FROM candidate_user_type_acls AS acl
                WHERE acl.type_oid = user_type.oid
            ), '[]'::pg_catalog.jsonb),
            'enum_labels', coalesce((
                SELECT pg_catalog.jsonb_agg(enum_row.enumlabel ORDER BY enum_row.enumsortorder)
                FROM candidate_enum_labels AS candidate_label
                JOIN pg_catalog.pg_enum AS enum_row ON enum_row.oid = candidate_label.enum_oid
                WHERE candidate_label.type_oid = user_type.oid
            ), '[]'::pg_catalog.jsonb),
            'composite_columns', coalesce((
                SELECT pg_catalog.jsonb_agg(
                    pg_catalog.jsonb_build_object(
                        'number', attribute.attnum,
                        'dropped', attribute.attisdropped
                    ) || CASE WHEN attribute.attisdropped THEN '{}'::pg_catalog.jsonb
                    ELSE pg_catalog.jsonb_build_object(
                        'name', attribute.attname,
                        'type', pg_catalog.format_type(attribute.atttypid, attribute.atttypmod),
                        'has_missing', attribute.atthasmissing,
                        'missing_value', pg_catalog.to_jsonb(attribute.attmissingval),
                        'is_local', attribute.attislocal,
                        'inheritance_count', attribute.attinhcount,
                        'collation', CASE
                            WHEN attribute.attcollation = 0 THEN ''
                            ELSE pg_catalog.quote_ident(collation_ns.nspname) || '.' ||
                                 pg_catalog.quote_ident(collation_row.collname)
                        END
                    ) END
                    ORDER BY attribute.attnum
                )
                FROM candidate_composite_attributes AS candidate_attribute
                JOIN pg_catalog.pg_attribute AS attribute
                  ON attribute.attrelid = user_type.typrelid
                 AND attribute.attnum = candidate_attribute.attnum
                LEFT JOIN pg_catalog.pg_collation AS collation_row
                  ON collation_row.oid = attribute.attcollation
                LEFT JOIN pg_catalog.pg_namespace AS collation_ns
                  ON collation_ns.oid = collation_row.collnamespace
                WHERE candidate_attribute.type_oid = user_type.oid
            ), '[]'::pg_catalog.jsonb)
        ) AS payload
    FROM user_type_targets AS candidate
    JOIN pg_catalog.pg_type AS user_type ON user_type.oid = candidate.oid
    JOIN pg_catalog.pg_namespace AS type_ns ON type_ns.oid = user_type.typnamespace
    LEFT JOIN pg_catalog.pg_class AS type_relation ON type_relation.oid = user_type.typrelid
    CROSS JOIN database_owner AS own
),
user_type_objects AS (
    SELECT payload.kind, payload.schema_name, payload.object_name, payload.payload
    FROM candidate_user_types AS candidate
    JOIN user_type_payloads AS payload ON payload.type_oid = candidate.oid
),
candidate_extension_member_addresses AS MATERIALIZED (
    SELECT member.extension_oid, member.classid, member.objid, member.objsubid
    FROM safe_extension_members AS member
    JOIN pg_catalog.pg_extension AS extension_row ON extension_row.oid = member.extension_oid
    ORDER BY extension_row.extname, member.classid, member.objid, member.objsubid
    LIMIT $4
),
extension_member_addresses AS (
    SELECT
        member.extension_oid,
        member.classid,
        member.objid,
        member.objsubid,
        class_row.relname AS catalog_name,
        pg_catalog.jsonb_build_object(
            'type', identified.type,
            'names', coalesce(pg_catalog.to_jsonb(identified.object_names), '[]'::pg_catalog.jsonb),
            'args', coalesce(pg_catalog.to_jsonb(identified.object_args), '[]'::pg_catalog.jsonb)
        ) AS address
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_class AS class_row ON class_row.oid = member.classid
    CROSS JOIN LATERAL pg_catalog.pg_identify_object_as_address(
        member.classid,
        member.objid,
        member.objsubid
    ) AS identified
),
candidate_extension_initial_privileges AS MATERIALIZED (
    SELECT
        member.extension_oid,
        member.classid,
        member.objid,
        member.objsubid,
        selected.initial_objsubid,
        selected.privilege_type,
        selected.initial_acl
    FROM candidate_extension_member_addresses AS member
    CROSS JOIN LATERAL (
        SELECT
            initial.objsubid AS initial_objsubid,
            initial.privtype AS privilege_type,
            initial.initprivs AS initial_acl
        FROM pg_catalog.pg_init_privs AS initial
        WHERE initial.classoid = member.classid
          AND initial.objoid = member.objid
        ORDER BY initial.objsubid, initial.privtype
        LIMIT $1
    ) AS selected
),
candidate_extension_initial_acl_role_entries AS MATERIALIZED (
    SELECT
        initial.extension_oid,
        initial.classid,
        initial.objid,
        initial.objsubid,
        initial.initial_objsubid,
        initial.privilege_type,
        selected.grantor,
        selected.grantee,
        selected.acl_privilege,
        selected.is_grantable
    FROM candidate_extension_initial_privileges AS initial
    CROSS JOIN LATERAL (
        SELECT
            acl.grantor,
            acl.grantee,
            acl.privilege_type AS acl_privilege,
            acl.is_grantable
        FROM pg_catalog.aclexplode(initial.initial_acl) AS acl
        ORDER BY
            CASE WHEN acl.grantee = 0 THEN 'PUBLIC'
                 ELSE pg_catalog.pg_get_userbyid(acl.grantee) END,
            acl.privilege_type,
            pg_catalog.pg_get_userbyid(acl.grantor),
            acl.is_grantable
        LIMIT $1
    ) AS selected
),
candidate_extension_language_acl_role_entries AS MATERIALIZED (
    SELECT
        member.extension_oid,
        member.classid,
        member.objid,
        member.objsubid,
        selected.grantor,
        selected.grantee,
        selected.privilege_type,
        selected.is_grantable
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_language AS language ON language.oid = member.objid
    CROSS JOIN LATERAL (
        SELECT acl.grantor, acl.grantee, acl.privilege_type, acl.is_grantable
        FROM pg_catalog.aclexplode(
            coalesce(language.lanacl, pg_catalog.acldefault('l', language.lanowner))
        ) AS acl
        ORDER BY
            CASE WHEN acl.grantee = 0 THEN 'PUBLIC'
                 ELSE pg_catalog.pg_get_userbyid(acl.grantee) END,
            acl.privilege_type,
            pg_catalog.pg_get_userbyid(acl.grantor),
            acl.is_grantable
        LIMIT $1
    ) AS selected
    WHERE member.classid = 'pg_catalog.pg_language'::pg_catalog.regclass
),
candidate_extension_member_owners AS MATERIALIZED (
    SELECT
        member.extension_oid,
        member.classid,
        member.objid,
        member.objsubid,
        relation.relowner AS role_oid
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_class AS relation ON relation.oid = member.objid
    WHERE member.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
    UNION
    SELECT member.extension_oid, member.classid, member.objid, member.objsubid, routine.proowner
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_proc AS routine ON routine.oid = member.objid
    WHERE member.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
    UNION
    SELECT member.extension_oid, member.classid, member.objid, member.objsubid, type_row.typowner
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_type AS type_row ON type_row.oid = member.objid
    WHERE member.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
    UNION
    SELECT member.extension_oid, member.classid, member.objid, member.objsubid, language.lanowner
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_language AS language ON language.oid = member.objid
    WHERE member.classid = 'pg_catalog.pg_language'::pg_catalog.regclass
    UNION
    SELECT member.extension_oid, member.classid, member.objid, member.objsubid, operator_row.oprowner
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_operator AS operator_row ON operator_row.oid = member.objid
    WHERE member.classid = 'pg_catalog.pg_operator'::pg_catalog.regclass
    UNION
    SELECT member.extension_oid, member.classid, member.objid, member.objsubid,
           operator_class.opcowner
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_opclass AS operator_class ON operator_class.oid = member.objid
    WHERE member.classid = 'pg_catalog.pg_opclass'::pg_catalog.regclass
    UNION
    SELECT member.extension_oid, member.classid, member.objid, member.objsubid,
           operator_family.opfowner
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_opfamily AS operator_family ON operator_family.oid = member.objid
    WHERE member.classid = 'pg_catalog.pg_opfamily'::pg_catalog.regclass
),
candidate_extension_role_reference_pairs AS MATERIALIZED (
    SELECT referenced.extension_oid, referenced.role_oid
    FROM (
        SELECT candidate.oid AS extension_oid, extension_row.extowner AS role_oid
        FROM candidate_extensions AS candidate
        JOIN pg_catalog.pg_extension AS extension_row ON extension_row.oid = candidate.oid
        UNION
        SELECT member_owner.extension_oid, member_owner.role_oid
        FROM candidate_extension_member_owners AS member_owner
        UNION
        SELECT acl.extension_oid, acl.grantor
        FROM candidate_extension_initial_acl_role_entries AS acl
        UNION
        SELECT acl.extension_oid, acl.grantee
        FROM candidate_extension_initial_acl_role_entries AS acl
        WHERE acl.grantee <> 0
        UNION
        SELECT acl.extension_oid, acl.grantor
        FROM candidate_extension_language_acl_role_entries AS acl
        UNION
        SELECT acl.extension_oid, acl.grantee
        FROM candidate_extension_language_acl_role_entries AS acl
        WHERE acl.grantee <> 0
        UNION
        SELECT member.extension_oid, acl.grantor
        FROM candidate_extension_member_addresses AS member
        JOIN candidate_relation_acls AS acl ON acl.relation_oid = member.objid
        WHERE member.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
        UNION
        SELECT member.extension_oid, acl.grantee
        FROM candidate_extension_member_addresses AS member
        JOIN candidate_relation_acls AS acl ON acl.relation_oid = member.objid
        WHERE member.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
          AND acl.grantee <> 0
        UNION
        SELECT member.extension_oid, acl.grantor
        FROM candidate_extension_member_addresses AS member
        JOIN candidate_column_acls AS acl ON acl.relation_oid = member.objid
        WHERE member.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
        UNION
        SELECT member.extension_oid, acl.grantee
        FROM candidate_extension_member_addresses AS member
        JOIN candidate_column_acls AS acl ON acl.relation_oid = member.objid
        WHERE member.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
          AND acl.grantee <> 0
        UNION
        SELECT member.extension_oid, acl.grantor
        FROM candidate_extension_member_addresses AS member
        JOIN candidate_routine_acls AS acl ON acl.routine_oid = member.objid
        WHERE member.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
        UNION
        SELECT member.extension_oid, acl.grantee
        FROM candidate_extension_member_addresses AS member
        JOIN candidate_routine_acls AS acl ON acl.routine_oid = member.objid
        WHERE member.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
          AND acl.grantee <> 0
        UNION
        SELECT member.extension_oid, acl.grantor
        FROM candidate_extension_member_addresses AS member
        JOIN candidate_domain_acls AS acl ON acl.domain_oid = member.objid
        WHERE member.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
        UNION
        SELECT member.extension_oid, acl.grantee
        FROM candidate_extension_member_addresses AS member
        JOIN candidate_domain_acls AS acl ON acl.domain_oid = member.objid
        WHERE member.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
          AND acl.grantee <> 0
        UNION
        SELECT member.extension_oid, acl.grantor
        FROM candidate_extension_member_addresses AS member
        JOIN candidate_user_type_acls AS acl ON acl.type_oid = member.objid
        WHERE member.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
        UNION
        SELECT member.extension_oid, acl.grantee
        FROM candidate_extension_member_addresses AS member
        JOIN candidate_user_type_acls AS acl ON acl.type_oid = member.objid
        WHERE member.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
          AND acl.grantee <> 0
        UNION
        SELECT member.extension_oid, dependency.refobjid
        FROM candidate_extension_member_addresses AS member
        JOIN pg_catalog.pg_shdepend AS dependency
          ON dependency.classid = member.classid
         AND dependency.objid = member.objid
         AND dependency.objsubid = member.objsubid
        JOIN current_database_row AS database_row ON database_row.oid = dependency.dbid
        WHERE dependency.refclassid = 'pg_catalog.pg_authid'::pg_catalog.regclass
        UNION
        SELECT member.extension_oid, dependency.objid
        FROM candidate_extension_member_addresses AS member
        JOIN pg_catalog.pg_shdepend AS dependency
          ON dependency.refclassid = member.classid
         AND dependency.refobjid = member.objid
        JOIN current_database_row AS database_row ON database_row.oid = dependency.dbid
        WHERE dependency.classid = 'pg_catalog.pg_authid'::pg_catalog.regclass
    ) AS referenced
    WHERE referenced.role_oid <> 0
),
candidate_extension_role_references AS MATERIALIZED (
    SELECT DISTINCT pair.role_oid
    FROM candidate_extension_role_reference_pairs AS pair
    ORDER BY pair.role_oid
    LIMIT $6
),
extension_role_identity_profile AS MATERIALIZED (
    SELECT
        role_row.oid AS role_oid,
        CASE
            WHEN role_row.rolname = 'babylon_intel' THEN 'babylon_intel'
            WHEN role_row.rolsuper THEN '$superuser'
            WHEN role_row.oid = own.owner_oid THEN '$database_owner'
            ELSE '$other_owner:' || role_row.rolname
        END AS identity
    FROM pg_catalog.pg_roles AS role_row
    JOIN candidate_extension_role_references AS referenced
      ON referenced.role_oid = role_row.oid
    CROSS JOIN database_owner AS own
),
extension_role_identity_counts AS (
    SELECT
        extension_row.oid AS extension_oid,
        pg_catalog.count(DISTINCT reference_pair.role_oid) AS referenced_role_oid_count,
        pg_catalog.count(DISTINCT profile.role_oid) AS resolved_role_oid_count,
        pg_catalog.count(DISTINCT profile.identity) AS canonical_identity_count
    FROM candidate_extensions AS candidate
    JOIN pg_catalog.pg_extension AS extension_row ON extension_row.oid = candidate.oid
    LEFT JOIN candidate_extension_role_reference_pairs AS reference_pair
      ON reference_pair.extension_oid = extension_row.oid
    LEFT JOIN extension_role_identity_profile AS profile
      ON profile.role_oid = reference_pair.role_oid
    GROUP BY extension_row.oid
),
candidate_extension_dependency_edges AS MATERIALIZED (
    SELECT DISTINCT edge.*
    FROM (
        SELECT
            member.extension_oid,
            member.classid AS member_classid,
            member.objid AS member_objid,
            member.objsubid AS member_objsubid,
            'outbound'::pg_catalog.text AS direction,
            'pg_depend'::pg_catalog.text AS dependency_catalog,
            dependency.refclassid AS other_classid,
            dependency.refobjid AS other_objid,
            dependency.refobjsubid AS other_objsubid,
            dependency.deptype
        FROM candidate_extension_member_addresses AS member
        JOIN pg_catalog.pg_depend AS dependency
          ON dependency.classid = member.classid
         AND dependency.objid = member.objid
         AND dependency.objsubid = member.objsubid
        WHERE NOT (
            dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
            AND dependency.refobjid = member.extension_oid
            AND dependency.refobjsubid = 0
            AND dependency.deptype = 'e'
        )
        UNION ALL
        SELECT
            member.extension_oid,
            member.classid,
            member.objid,
            member.objsubid,
            'inbound',
            'pg_depend',
            dependency.classid,
            dependency.objid,
            dependency.objsubid,
            dependency.deptype
        FROM candidate_extension_member_addresses AS member
        JOIN pg_catalog.pg_depend AS dependency
          ON dependency.refclassid = member.classid
         AND dependency.refobjid = member.objid
         AND dependency.refobjsubid = member.objsubid
        UNION ALL
        SELECT
            member.extension_oid,
            member.classid,
            member.objid,
            member.objsubid,
            'outbound',
            'pg_shdepend',
            dependency.refclassid,
            dependency.refobjid,
            0,
            dependency.deptype
        FROM candidate_extension_member_addresses AS member
        JOIN pg_catalog.pg_shdepend AS dependency
          ON dependency.classid = member.classid
         AND dependency.objid = member.objid
         AND dependency.objsubid = member.objsubid
        JOIN current_database_row AS database_row ON database_row.oid = dependency.dbid
        UNION ALL
        SELECT
            member_owner.extension_oid,
            member_owner.classid,
            member_owner.objid,
            member_owner.objsubid,
            'outbound',
            'pg_shdepend',
            'pg_catalog.pg_authid'::pg_catalog.regclass,
            member_owner.role_oid,
            0,
            'o'
        FROM candidate_extension_member_owners AS member_owner
        UNION ALL
        SELECT
            member.extension_oid,
            member.classid,
            member.objid,
            member.objsubid,
            'inbound',
            'pg_shdepend',
            dependency.classid,
            dependency.objid,
            dependency.objsubid,
            dependency.deptype
        FROM candidate_extension_member_addresses AS member
        JOIN pg_catalog.pg_shdepend AS dependency
          ON dependency.refclassid = member.classid
         AND dependency.refobjid = member.objid
        JOIN current_database_row AS database_row ON database_row.oid = dependency.dbid
    ) AS edge
    ORDER BY
        edge.extension_oid,
        edge.member_classid,
        edge.member_objid,
        edge.member_objsubid,
        edge.direction,
        edge.dependency_catalog,
        edge.other_classid,
        edge.other_objid,
        edge.other_objsubid,
        edge.deptype
    LIMIT $4
),
candidate_internal_toast_dependencies AS MATERIALIZED (
    SELECT
        edge.*,
        'internal_toast_table'::pg_catalog.text AS storage_kind,
        resolved.parent_oid,
        resolved.owner_count
    FROM candidate_extension_dependency_edges AS edge
    JOIN pg_catalog.pg_class AS toast_relation
      ON toast_relation.oid = edge.other_objid
     AND toast_relation.relkind = 't'
    JOIN pg_catalog.pg_namespace AS toast_namespace
      ON toast_namespace.oid = toast_relation.relnamespace
     AND toast_namespace.nspname = 'pg_toast'
    CROSS JOIN LATERAL (
        SELECT
            pg_catalog.min(owner_relation.oid) AS parent_oid,
            pg_catalog.count(*) AS owner_count
        FROM pg_catalog.pg_class AS owner_relation
        WHERE owner_relation.reltoastrelid = toast_relation.oid
    ) AS resolved
    WHERE edge.other_classid = 'pg_catalog.pg_class'::pg_catalog.regclass
    UNION ALL
    SELECT
        edge.*,
        'internal_toast_index',
        resolved.parent_oid,
        resolved.owner_count
    FROM candidate_extension_dependency_edges AS edge
    JOIN pg_catalog.pg_class AS toast_index
      ON toast_index.oid = edge.other_objid
     AND toast_index.relkind = 'i'
    JOIN pg_catalog.pg_namespace AS toast_namespace
      ON toast_namespace.oid = toast_index.relnamespace
     AND toast_namespace.nspname = 'pg_toast'
    LEFT JOIN pg_catalog.pg_index AS toast_index_link
      ON toast_index_link.indexrelid = toast_index.oid
    LEFT JOIN pg_catalog.pg_class AS toast_relation
      ON toast_relation.oid = toast_index_link.indrelid
     AND toast_relation.relkind = 't'
    CROSS JOIN LATERAL (
        SELECT
            pg_catalog.min(owner_relation.oid) AS parent_oid,
            pg_catalog.count(*) AS owner_count
        FROM pg_catalog.pg_class AS owner_relation
        WHERE owner_relation.reltoastrelid = toast_relation.oid
    ) AS resolved
    WHERE edge.other_classid = 'pg_catalog.pg_class'::pg_catalog.regclass
),
extension_dependency_addresses AS (
    SELECT
        edge.extension_oid,
        edge.member_classid,
        edge.member_objid,
        edge.member_objsubid,
        edge.direction,
        pg_catalog.jsonb_build_object(
            'catalog', edge.dependency_catalog,
            'member', member.address,
            'other', CASE
                WHEN edge.other_classid = 'pg_catalog.pg_authid'::pg_catalog.regclass
                THEN pg_catalog.jsonb_build_object(
                    'type', 'role',
                    'names', CASE
                        WHEN dependency_role_identity.role_oid IS NULL
                        THEN '[]'::pg_catalog.jsonb
                        ELSE pg_catalog.jsonb_build_array(dependency_role_identity.identity)
                    END,
                    'args', '[]'::pg_catalog.jsonb
                )
                WHEN toast.storage_kind IS NULL THEN pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', coalesce(
                        pg_catalog.to_jsonb(identified.object_names),
                        '[]'::pg_catalog.jsonb
                    ),
                    'args', coalesce(
                        pg_catalog.to_jsonb(identified.object_args),
                        '[]'::pg_catalog.jsonb
                    )
                )
                WHEN toast.owner_count = 1 THEN pg_catalog.jsonb_build_object(
                    'type', toast.storage_kind,
                    'names', coalesce(
                        pg_catalog.to_jsonb(parent_identified.object_names),
                        '[]'::pg_catalog.jsonb
                    ),
                    'args', coalesce(
                        pg_catalog.to_jsonb(parent_identified.object_args),
                        '[]'::pg_catalog.jsonb
                    ) || pg_catalog.jsonb_build_array(edge.other_objsubid)
                )
                ELSE pg_catalog.jsonb_build_object(
                    'type', 'unresolved_internal_toast',
                    'names', '[]'::pg_catalog.jsonb,
                    'args', '[]'::pg_catalog.jsonb
                )
            END,
            'other_address_complete', CASE
                WHEN edge.other_classid = 'pg_catalog.pg_authid'::pg_catalog.regclass
                THEN dependency_role_identity.role_oid IS NOT NULL
                ELSE coalesce(toast.owner_count = 1, true)
            END,
            'dependency_type', edge.deptype
        ) AS edge_payload
    FROM candidate_extension_dependency_edges AS edge
    JOIN extension_member_addresses AS member
      ON member.extension_oid = edge.extension_oid
     AND member.classid = edge.member_classid
     AND member.objid = edge.member_objid
     AND member.objsubid = edge.member_objsubid
    LEFT JOIN candidate_internal_toast_dependencies AS toast
      ON toast.extension_oid = edge.extension_oid
     AND toast.member_classid = edge.member_classid
     AND toast.member_objid = edge.member_objid
     AND toast.member_objsubid = edge.member_objsubid
     AND toast.direction = edge.direction
     AND toast.dependency_catalog = edge.dependency_catalog
     AND toast.other_classid = edge.other_classid
     AND toast.other_objid = edge.other_objid
     AND toast.other_objsubid = edge.other_objsubid
     AND toast.deptype = edge.deptype
    LEFT JOIN extension_role_identity_profile AS dependency_role_identity
      ON edge.other_classid = 'pg_catalog.pg_authid'::pg_catalog.regclass
     AND dependency_role_identity.role_oid = edge.other_objid
    LEFT JOIN LATERAL pg_catalog.pg_identify_object_as_address(
        edge.other_classid,
        edge.other_objid,
        edge.other_objsubid
    ) AS identified
      ON toast.storage_kind IS NULL
     AND edge.other_classid <> 'pg_catalog.pg_authid'::pg_catalog.regclass
    LEFT JOIN LATERAL pg_catalog.pg_identify_object_as_address(
        'pg_catalog.pg_class'::pg_catalog.regclass,
        toast.parent_oid,
        0
    ) AS parent_identified ON toast.owner_count = 1
),
extension_member_dependency_payloads AS MATERIALIZED (
    SELECT
        dependency.extension_oid,
        dependency.member_classid,
        dependency.member_objid,
        dependency.member_objsubid,
        coalesce(pg_catalog.jsonb_agg(
            dependency.edge_payload
            ORDER BY dependency.edge_payload::pg_catalog.text
        ) FILTER (WHERE dependency.direction = 'outbound'), '[]'::pg_catalog.jsonb)
            AS outbound_dependencies,
        coalesce(pg_catalog.jsonb_agg(
            dependency.edge_payload
            ORDER BY dependency.edge_payload::pg_catalog.text
        ) FILTER (WHERE dependency.direction = 'inbound'), '[]'::pg_catalog.jsonb)
            AS inbound_dependencies
    FROM extension_dependency_addresses AS dependency
    GROUP BY
        dependency.extension_oid,
        dependency.member_classid,
        dependency.member_objid,
        dependency.member_objsubid
),
extension_address_budget_rows AS (
    SELECT member.extension_oid, 'member'::pg_catalog.text AS address_kind
    FROM extension_member_addresses AS member
    UNION ALL
    SELECT dependency.extension_oid, 'dependency'
    FROM extension_dependency_addresses AS dependency
),
candidate_extension_initial_acl_entries AS MATERIALIZED (
    SELECT
        acl.extension_oid,
        acl.classid,
        acl.objid,
        acl.objsubid,
        acl.initial_objsubid,
        acl.privilege_type,
        grantor_identity.identity AS grantor_identity,
        CASE
            WHEN acl.grantee = 0 THEN 'PUBLIC'
            ELSE grantee_identity.identity
        END AS grantee_identity,
        acl.acl_privilege,
        acl.is_grantable
    FROM candidate_extension_initial_acl_role_entries AS acl
    JOIN extension_role_identity_profile AS grantor_identity
      ON grantor_identity.role_oid = acl.grantor
    LEFT JOIN extension_role_identity_profile AS grantee_identity
      ON grantee_identity.role_oid = acl.grantee
),
extension_initial_privileges AS (
    SELECT
        initial.extension_oid,
        initial.classid,
        initial.objid,
        initial.objsubid,
        pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
            'subobject', initial.initial_objsubid,
            'privilege_type', initial.privilege_type,
            'acl', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'grantor', acl.grantor_identity,
                    'grantee', acl.grantee_identity,
                    'privilege', acl.acl_privilege,
                    'grantable', acl.is_grantable
                ) ORDER BY
                    acl.grantee_identity,
                    acl.acl_privilege,
                    acl.grantor_identity,
                    acl.is_grantable)
                FROM candidate_extension_initial_acl_entries AS acl
                WHERE acl.extension_oid = initial.extension_oid
                  AND acl.classid = initial.classid
                  AND acl.objid = initial.objid
                  AND acl.objsubid = initial.objsubid
                  AND acl.initial_objsubid = initial.initial_objsubid
                  AND acl.privilege_type = initial.privilege_type
            ), '[]'::pg_catalog.jsonb)
        ) ORDER BY initial.initial_objsubid, initial.privilege_type) AS initial_acl
    FROM candidate_extension_initial_privileges AS initial
    GROUP BY initial.extension_oid, initial.classid, initial.objid, initial.objsubid
),
extension_relation_acls AS MATERIALIZED (
    SELECT
        member.extension_oid,
        member.objid AS relation_oid,
        grantor_identity.identity AS grantor_identity,
        CASE
            WHEN acl.grantee = 0 THEN 'PUBLIC'
            ELSE grantee_identity.identity
        END AS grantee_identity,
        acl.privilege_type,
        acl.is_grantable
    FROM candidate_extension_member_addresses AS member
    JOIN candidate_relation_acls AS acl ON acl.relation_oid = member.objid
    JOIN extension_role_identity_profile AS grantor_identity
      ON grantor_identity.role_oid = acl.grantor
    LEFT JOIN extension_role_identity_profile AS grantee_identity
      ON grantee_identity.role_oid = acl.grantee
    WHERE member.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
),
extension_column_acls AS MATERIALIZED (
    SELECT
        member.extension_oid,
        member.objid AS relation_oid,
        acl.attnum,
        grantor_identity.identity AS grantor_identity,
        CASE
            WHEN acl.grantee = 0 THEN 'PUBLIC'
            ELSE grantee_identity.identity
        END AS grantee_identity,
        acl.privilege_type,
        acl.is_grantable
    FROM candidate_extension_member_addresses AS member
    JOIN candidate_column_acls AS acl ON acl.relation_oid = member.objid
    JOIN extension_role_identity_profile AS grantor_identity
      ON grantor_identity.role_oid = acl.grantor
    LEFT JOIN extension_role_identity_profile AS grantee_identity
      ON grantee_identity.role_oid = acl.grantee
    WHERE member.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
),
extension_routine_acls AS MATERIALIZED (
    SELECT
        member.extension_oid,
        member.objid AS routine_oid,
        grantor_identity.identity AS grantor_identity,
        CASE
            WHEN acl.grantee = 0 THEN 'PUBLIC'
            ELSE grantee_identity.identity
        END AS grantee_identity,
        acl.privilege_type,
        acl.is_grantable
    FROM candidate_extension_member_addresses AS member
    JOIN candidate_routine_acls AS acl ON acl.routine_oid = member.objid
    JOIN extension_role_identity_profile AS grantor_identity
      ON grantor_identity.role_oid = acl.grantor
    LEFT JOIN extension_role_identity_profile AS grantee_identity
      ON grantee_identity.role_oid = acl.grantee
    WHERE member.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
),
extension_type_acls AS MATERIALIZED (
    SELECT
        member.extension_oid,
        member.objid AS type_oid,
        grantor_identity.identity AS grantor_identity,
        CASE
            WHEN acl.grantee = 0 THEN 'PUBLIC'
            ELSE grantee_identity.identity
        END AS grantee_identity,
        acl.privilege_type,
        acl.is_grantable
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_type AS type_row ON type_row.oid = member.objid
    JOIN candidate_domain_acls AS domain_acl
      ON domain_acl.domain_oid = member.objid
     AND type_row.typtype = 'd'
    CROSS JOIN LATERAL (
        SELECT
            domain_acl.grantor,
            domain_acl.grantee,
            domain_acl.privilege_type,
            domain_acl.is_grantable
    ) AS acl
    JOIN extension_role_identity_profile AS grantor_identity
      ON grantor_identity.role_oid = acl.grantor
    LEFT JOIN extension_role_identity_profile AS grantee_identity
      ON grantee_identity.role_oid = acl.grantee
    WHERE member.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
    UNION ALL
    SELECT
        member.extension_oid,
        member.objid,
        grantor_identity.identity,
        CASE
            WHEN acl.grantee = 0 THEN 'PUBLIC'
            ELSE grantee_identity.identity
        END,
        acl.privilege_type,
        acl.is_grantable
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_type AS type_row ON type_row.oid = member.objid
    JOIN candidate_user_type_acls AS acl
      ON acl.type_oid = member.objid
     AND type_row.typtype <> 'd'
    JOIN extension_role_identity_profile AS grantor_identity
      ON grantor_identity.role_oid = acl.grantor
    LEFT JOIN extension_role_identity_profile AS grantee_identity
      ON grantee_identity.role_oid = acl.grantee
    WHERE member.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
),
extension_relation_payloads AS (
    SELECT
        member.extension_oid,
        member.classid,
        member.objid,
        member.objsubid,
        normalized.definition,
        pg_catalog.jsonb_build_object(
            'object', normalized.definition -> 'acl',
            'columns', normalized.definition -> 'columns'
        ) AS current_acl
    FROM candidate_extension_member_addresses AS member
    JOIN relation_payloads AS relation ON relation.relation_oid = member.objid
    JOIN pg_catalog.pg_class AS relation_row ON relation_row.oid = member.objid
    JOIN extension_role_identity_profile AS owner_identity
      ON owner_identity.role_oid = relation_row.relowner
    CROSS JOIN LATERAL (
        SELECT
            relation.payload - 'owner' - 'acl' - 'columns' ||
            pg_catalog.jsonb_build_object(
                'owner', owner_identity.identity,
                'acl', coalesce((
                    SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                        'grantor', acl.grantor_identity,
                        'grantee', acl.grantee_identity,
                        'privilege', acl.privilege_type,
                        'grantable', acl.is_grantable
                    ) ORDER BY
                        acl.grantee_identity,
                        acl.privilege_type,
                        acl.grantor_identity,
                        acl.is_grantable)
                    FROM extension_relation_acls AS acl
                    WHERE acl.extension_oid = member.extension_oid
                      AND acl.relation_oid = member.objid
                ), '[]'::pg_catalog.jsonb),
                'columns', coalesce((
                    SELECT pg_catalog.jsonb_agg(
                        column_item.column_payload - 'column_acl' ||
                        pg_catalog.jsonb_build_object(
                            'column_acl', coalesce((
                                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                                    'grantor', acl.grantor_identity,
                                    'grantee', acl.grantee_identity,
                                    'privilege', acl.privilege_type,
                                    'grantable', acl.is_grantable
                                ) ORDER BY
                                    acl.grantee_identity,
                                    acl.privilege_type,
                                    acl.grantor_identity,
                                    acl.is_grantable)
                                FROM extension_column_acls AS acl
                                WHERE acl.extension_oid = member.extension_oid
                                  AND acl.relation_oid = member.objid
                                  AND acl.attnum = (
                                      column_item.column_payload ->> 'num'
                                  )::pg_catalog.int2
                            ), '[]'::pg_catalog.jsonb)
                        )
                        ORDER BY column_item.column_index
                    )
                    FROM pg_catalog.jsonb_array_elements(relation.payload -> 'columns')
                        WITH ORDINALITY AS column_item(column_payload, column_index)
                ), '[]'::pg_catalog.jsonb)
            ) AS definition
    ) AS normalized
    WHERE member.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
),
candidate_extension_language_acls AS MATERIALIZED (
    SELECT
        acl.extension_oid,
        acl.classid,
        acl.objid,
        acl.objsubid,
        grantor_identity.identity AS grantor_identity,
        CASE
            WHEN acl.grantee = 0 THEN 'PUBLIC'
            ELSE grantee_identity.identity
        END AS grantee_identity,
        acl.privilege_type,
        acl.is_grantable
    FROM candidate_extension_language_acl_role_entries AS acl
    JOIN extension_role_identity_profile AS grantor_identity
      ON grantor_identity.role_oid = acl.grantor
    LEFT JOIN extension_role_identity_profile AS grantee_identity
      ON grantee_identity.role_oid = acl.grantee
),
candidate_extension_routine_transforms AS MATERIALIZED (
    SELECT member.extension_oid, member.objid AS routine_oid, selected.type_oid
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_proc AS routine ON routine.oid = member.objid
    CROSS JOIN LATERAL (
        SELECT transform_type.type_oid
        FROM pg_catalog.unnest(coalesce(routine.protrftypes, ARRAY[]::pg_catalog.oid[]))
            AS transform_type(type_oid)
        ORDER BY transform_type.type_oid
        LIMIT $1
    ) AS selected
    WHERE member.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
),
extension_routine_transform_addresses AS (
    SELECT
        transform.extension_oid,
        transform.routine_oid,
        pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
            'type', identified.type,
            'names', coalesce(pg_catalog.to_jsonb(identified.object_names), '[]'::pg_catalog.jsonb),
            'args', coalesce(pg_catalog.to_jsonb(identified.object_args), '[]'::pg_catalog.jsonb)
        ) ORDER BY identified.type, identified.object_names, identified.object_args) AS transforms
    FROM candidate_extension_routine_transforms AS transform
    CROSS JOIN LATERAL pg_catalog.pg_identify_object_as_address(
        'pg_catalog.pg_type'::pg_catalog.regclass,
        transform.type_oid,
        0
    ) AS identified
    GROUP BY transform.extension_oid, transform.routine_oid
),
extension_aggregate_payloads AS MATERIALIZED (
    SELECT
        member.extension_oid,
        member.objid AS routine_oid,
        pg_catalog.jsonb_build_object(
            'kind', aggregate_row.aggkind,
            'direct_arguments', aggregate_row.aggnumdirectargs,
            'transition_function', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    aggregate_row.aggtransfn,
                    0
                ) AS identified
            ),
            'final_function', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    aggregate_row.aggfinalfn,
                    0
                ) AS identified
                WHERE aggregate_row.aggfinalfn <> 0
            ),
            'combine_function', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    aggregate_row.aggcombinefn,
                    0
                ) AS identified
                WHERE aggregate_row.aggcombinefn <> 0
            ),
            'serial_function', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    aggregate_row.aggserialfn,
                    0
                ) AS identified
                WHERE aggregate_row.aggserialfn <> 0
            ),
            'deserial_function', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    aggregate_row.aggdeserialfn,
                    0
                ) AS identified
                WHERE aggregate_row.aggdeserialfn <> 0
            ),
            'moving_transition_function', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    aggregate_row.aggmtransfn,
                    0
                ) AS identified
                WHERE aggregate_row.aggmtransfn <> 0
            ),
            'moving_inverse_function', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    aggregate_row.aggminvtransfn,
                    0
                ) AS identified
                WHERE aggregate_row.aggminvtransfn <> 0
            ),
            'moving_final_function', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    aggregate_row.aggmfinalfn,
                    0
                ) AS identified
                WHERE aggregate_row.aggmfinalfn <> 0
            ),
            'final_extra', aggregate_row.aggfinalextra,
            'moving_final_extra', aggregate_row.aggmfinalextra,
            'final_modify', aggregate_row.aggfinalmodify,
            'moving_final_modify', aggregate_row.aggmfinalmodify,
            'sort_operator', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_operator'::pg_catalog.regclass,
                    aggregate_row.aggsortop,
                    0
                ) AS identified
                WHERE aggregate_row.aggsortop <> 0
            ),
            'transition_type', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_type'::pg_catalog.regclass,
                    aggregate_row.aggtranstype,
                    0
                ) AS identified
            ),
            'transition_space', aggregate_row.aggtransspace,
            'moving_transition_type', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_type'::pg_catalog.regclass,
                    aggregate_row.aggmtranstype,
                    0
                ) AS identified
            ),
            'moving_transition_space', aggregate_row.aggmtransspace,
            'initial_value', coalesce(aggregate_row.agginitval, ''),
            'moving_initial_value', coalesce(aggregate_row.aggminitval, '')
        ) AS aggregate_payload
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_proc AS routine ON routine.oid = member.objid AND routine.prokind = 'a'
    JOIN pg_catalog.pg_aggregate AS aggregate_row ON aggregate_row.aggfnoid = routine.oid
    WHERE member.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
),
extension_routine_payloads AS (
    SELECT
        member.extension_oid,
        member.classid,
        member.objid,
        member.objsubid,
        normalized.overload || pg_catalog.jsonb_build_object(
            'variadic', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_type'::pg_catalog.regclass,
                    routine.provariadic,
                    0
                ) AS identified
                WHERE routine.provariadic <> 0
            ),
            'transform', coalesce(transform.transforms, '[]'::pg_catalog.jsonb),
            'support', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    routine.prosupport,
                    0
                ) AS identified
                WHERE routine.prosupport <> 0
            ),
            'aggregate', coalesce(aggregate.aggregate_payload, '{}'::pg_catalog.jsonb)
        ) AS definition,
        normalized.overload -> 'acl' AS current_acl
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_proc AS routine ON routine.oid = member.objid
    JOIN routine_overloads AS overload ON overload.routine_oid = member.objid
    JOIN extension_role_identity_profile AS owner_identity
      ON owner_identity.role_oid = routine.proowner
    CROSS JOIN LATERAL (
        SELECT
            overload.overload - 'owner' - 'acl' ||
            pg_catalog.jsonb_build_object(
                'owner', owner_identity.identity,
                'acl', coalesce((
                    SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                        'grantor', acl.grantor_identity,
                        'grantee', acl.grantee_identity,
                        'privilege', acl.privilege_type,
                        'grantable', acl.is_grantable
                    ) ORDER BY
                        acl.grantee_identity,
                        acl.privilege_type,
                        acl.grantor_identity,
                        acl.is_grantable)
                    FROM extension_routine_acls AS acl
                    WHERE acl.extension_oid = member.extension_oid
                      AND acl.routine_oid = member.objid
                ), '[]'::pg_catalog.jsonb)
            ) AS overload
    ) AS normalized
    LEFT JOIN extension_routine_transform_addresses AS transform
      ON transform.extension_oid = member.extension_oid
     AND transform.routine_oid = member.objid
    LEFT JOIN extension_aggregate_payloads AS aggregate
      ON aggregate.extension_oid = member.extension_oid
     AND aggregate.routine_oid = member.objid
    WHERE member.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
),
extension_type_payloads AS (
    SELECT
        member.extension_oid,
        member.classid,
        member.objid,
        member.objsubid,
        normalized.base_payload || pg_catalog.jsonb_build_object(
            'catalog', pg_catalog.jsonb_build_object(
                'relation', (
                    SELECT pg_catalog.jsonb_build_object(
                        'type', identified.type,
                        'names', pg_catalog.to_jsonb(identified.object_names),
                        'args', pg_catalog.to_jsonb(identified.object_args))
                    FROM pg_catalog.pg_identify_object_as_address(
                        'pg_catalog.pg_class'::pg_catalog.regclass,
                        type_row.typrelid,
                        0
                    ) AS identified
                    WHERE type_row.typrelid <> 0
                ),
                'subscript', (
                    SELECT pg_catalog.jsonb_build_object(
                        'type', identified.type,
                        'names', pg_catalog.to_jsonb(identified.object_names),
                        'args', pg_catalog.to_jsonb(identified.object_args))
                    FROM pg_catalog.pg_identify_object_as_address(
                        'pg_catalog.pg_proc'::pg_catalog.regclass,
                        type_row.typsubscript,
                        0
                    ) AS identified
                    WHERE type_row.typsubscript <> 0
                ),
                'element', (
                    SELECT pg_catalog.jsonb_build_object(
                        'type', identified.type,
                        'names', pg_catalog.to_jsonb(identified.object_names),
                        'args', pg_catalog.to_jsonb(identified.object_args))
                    FROM pg_catalog.pg_identify_object_as_address(
                        'pg_catalog.pg_type'::pg_catalog.regclass,
                        type_row.typelem,
                        0
                    ) AS identified
                    WHERE type_row.typelem <> 0
                ),
                'array', (
                    SELECT pg_catalog.jsonb_build_object(
                        'type', identified.type,
                        'names', pg_catalog.to_jsonb(identified.object_names),
                        'args', pg_catalog.to_jsonb(identified.object_args))
                    FROM pg_catalog.pg_identify_object_as_address(
                        'pg_catalog.pg_type'::pg_catalog.regclass,
                        type_row.typarray,
                        0
                    ) AS identified
                    WHERE type_row.typarray <> 0
                ),
                'input', (
                    SELECT pg_catalog.jsonb_build_object(
                        'type', identified.type,
                        'names', pg_catalog.to_jsonb(identified.object_names),
                        'args', pg_catalog.to_jsonb(identified.object_args))
                    FROM pg_catalog.pg_identify_object_as_address(
                        'pg_catalog.pg_proc'::pg_catalog.regclass,
                        type_row.typinput,
                        0
                    ) AS identified
                ),
                'output', (
                    SELECT pg_catalog.jsonb_build_object(
                        'type', identified.type,
                        'names', pg_catalog.to_jsonb(identified.object_names),
                        'args', pg_catalog.to_jsonb(identified.object_args))
                    FROM pg_catalog.pg_identify_object_as_address(
                        'pg_catalog.pg_proc'::pg_catalog.regclass,
                        type_row.typoutput,
                        0
                    ) AS identified
                ),
                'receive', (
                    SELECT pg_catalog.jsonb_build_object(
                        'type', identified.type,
                        'names', pg_catalog.to_jsonb(identified.object_names),
                        'args', pg_catalog.to_jsonb(identified.object_args))
                    FROM pg_catalog.pg_identify_object_as_address(
                        'pg_catalog.pg_proc'::pg_catalog.regclass,
                        type_row.typreceive,
                        0
                    ) AS identified
                    WHERE type_row.typreceive <> 0
                ),
                'send', (
                    SELECT pg_catalog.jsonb_build_object(
                        'type', identified.type,
                        'names', pg_catalog.to_jsonb(identified.object_names),
                        'args', pg_catalog.to_jsonb(identified.object_args))
                    FROM pg_catalog.pg_identify_object_as_address(
                        'pg_catalog.pg_proc'::pg_catalog.regclass,
                        type_row.typsend,
                        0
                    ) AS identified
                    WHERE type_row.typsend <> 0
                ),
                'modifier_input', (
                    SELECT pg_catalog.jsonb_build_object(
                        'type', identified.type,
                        'names', pg_catalog.to_jsonb(identified.object_names),
                        'args', pg_catalog.to_jsonb(identified.object_args))
                    FROM pg_catalog.pg_identify_object_as_address(
                        'pg_catalog.pg_proc'::pg_catalog.regclass,
                        type_row.typmodin,
                        0
                    ) AS identified
                    WHERE type_row.typmodin <> 0
                ),
                'modifier_output', (
                    SELECT pg_catalog.jsonb_build_object(
                        'type', identified.type,
                        'names', pg_catalog.to_jsonb(identified.object_names),
                        'args', pg_catalog.to_jsonb(identified.object_args))
                    FROM pg_catalog.pg_identify_object_as_address(
                        'pg_catalog.pg_proc'::pg_catalog.regclass,
                        type_row.typmodout,
                        0
                    ) AS identified
                    WHERE type_row.typmodout <> 0
                ),
                'analyze', (
                    SELECT pg_catalog.jsonb_build_object(
                        'type', identified.type,
                        'names', pg_catalog.to_jsonb(identified.object_names),
                        'args', pg_catalog.to_jsonb(identified.object_args))
                    FROM pg_catalog.pg_identify_object_as_address(
                        'pg_catalog.pg_proc'::pg_catalog.regclass,
                        type_row.typanalyze,
                        0
                    ) AS identified
                    WHERE type_row.typanalyze <> 0
                ),
                'base_type', (
                    SELECT pg_catalog.jsonb_build_object(
                        'type', identified.type,
                        'names', pg_catalog.to_jsonb(identified.object_names),
                        'args', pg_catalog.to_jsonb(identified.object_args))
                    FROM pg_catalog.pg_identify_object_as_address(
                        'pg_catalog.pg_type'::pg_catalog.regclass,
                        type_row.typbasetype,
                        0
                    ) AS identified
                    WHERE type_row.typbasetype <> 0
                ),
                'collation', (
                    SELECT pg_catalog.jsonb_build_object(
                        'type', identified.type,
                        'names', pg_catalog.to_jsonb(identified.object_names),
                        'args', pg_catalog.to_jsonb(identified.object_args))
                    FROM pg_catalog.pg_identify_object_as_address(
                        'pg_catalog.pg_collation'::pg_catalog.regclass,
                        type_row.typcollation,
                        0
                    ) AS identified
                    WHERE type_row.typcollation <> 0
                ),
                'default_expression', coalesce(
                    pg_catalog.pg_get_expr(type_row.typdefaultbin, 0),
                    ''
                ),
                'default_text', coalesce(type_row.typdefault, '')
            ),
            'range', coalesce((
                SELECT pg_catalog.jsonb_build_object(
                    'range_type', (
                        SELECT pg_catalog.jsonb_build_object(
                            'type', identified.type,
                            'names', pg_catalog.to_jsonb(identified.object_names),
                            'args', pg_catalog.to_jsonb(identified.object_args))
                        FROM pg_catalog.pg_identify_object_as_address(
                            'pg_catalog.pg_type'::pg_catalog.regclass,
                            range_row.rngtypid,
                            0
                        ) AS identified
                    ),
                    'subtype', (
                        SELECT pg_catalog.jsonb_build_object(
                            'type', identified.type,
                            'names', pg_catalog.to_jsonb(identified.object_names),
                            'args', pg_catalog.to_jsonb(identified.object_args))
                        FROM pg_catalog.pg_identify_object_as_address(
                            'pg_catalog.pg_type'::pg_catalog.regclass,
                            range_row.rngsubtype,
                            0
                        ) AS identified
                    ),
                    'multirange_type', (
                        SELECT pg_catalog.jsonb_build_object(
                            'type', identified.type,
                            'names', pg_catalog.to_jsonb(identified.object_names),
                            'args', pg_catalog.to_jsonb(identified.object_args))
                        FROM pg_catalog.pg_identify_object_as_address(
                            'pg_catalog.pg_type'::pg_catalog.regclass,
                            range_row.rngmultitypid,
                            0
                        ) AS identified
                    ),
                    'collation', (
                        SELECT pg_catalog.jsonb_build_object(
                            'type', identified.type,
                            'names', pg_catalog.to_jsonb(identified.object_names),
                            'args', pg_catalog.to_jsonb(identified.object_args))
                        FROM pg_catalog.pg_identify_object_as_address(
                            'pg_catalog.pg_collation'::pg_catalog.regclass,
                            range_row.rngcollation,
                            0
                        ) AS identified
                        WHERE range_row.rngcollation <> 0
                    ),
                    'operator_class', (
                        SELECT pg_catalog.jsonb_build_object(
                            'type', identified.type,
                            'names', pg_catalog.to_jsonb(identified.object_names),
                            'args', pg_catalog.to_jsonb(identified.object_args))
                        FROM pg_catalog.pg_identify_object_as_address(
                            'pg_catalog.pg_opclass'::pg_catalog.regclass,
                            range_row.rngsubopc,
                            0
                        ) AS identified
                    ),
                    'canonical_function', (
                        SELECT pg_catalog.jsonb_build_object(
                            'type', identified.type,
                            'names', pg_catalog.to_jsonb(identified.object_names),
                            'args', pg_catalog.to_jsonb(identified.object_args))
                        FROM pg_catalog.pg_identify_object_as_address(
                            'pg_catalog.pg_proc'::pg_catalog.regclass,
                            range_row.rngcanonical,
                            0
                        ) AS identified
                        WHERE range_row.rngcanonical <> 0
                    ),
                    'difference_function', (
                        SELECT pg_catalog.jsonb_build_object(
                            'type', identified.type,
                            'names', pg_catalog.to_jsonb(identified.object_names),
                            'args', pg_catalog.to_jsonb(identified.object_args))
                        FROM pg_catalog.pg_identify_object_as_address(
                            'pg_catalog.pg_proc'::pg_catalog.regclass,
                            range_row.rngsubdiff,
                            0
                        ) AS identified
                        WHERE range_row.rngsubdiff <> 0
                    )
                )
                FROM pg_catalog.pg_range AS range_row
                WHERE range_row.rngtypid = type_row.oid
                   OR range_row.rngmultitypid = type_row.oid
            ), '{}'::pg_catalog.jsonb)
        ) AS definition,
        normalized.base_payload -> 'acl' AS current_acl
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_type AS type_row ON type_row.oid = member.objid
    LEFT JOIN domain_payloads AS domain ON domain.domain_oid = member.objid
    LEFT JOIN user_type_payloads AS user_type_payload ON user_type_payload.type_oid = member.objid
    JOIN extension_role_identity_profile AS owner_identity
      ON owner_identity.role_oid = type_row.typowner
    CROSS JOIN LATERAL (
        SELECT
            coalesce(domain.payload, user_type_payload.payload) - 'owner' - 'acl' ||
            pg_catalog.jsonb_build_object(
                'owner', owner_identity.identity,
                'acl', coalesce((
                    SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                        'grantor', acl.grantor_identity,
                        'grantee', acl.grantee_identity,
                        'privilege', acl.privilege_type,
                        'grantable', acl.is_grantable
                    ) ORDER BY
                        acl.grantee_identity,
                        acl.privilege_type,
                        acl.grantor_identity,
                        acl.is_grantable)
                    FROM extension_type_acls AS acl
                    WHERE acl.extension_oid = member.extension_oid
                      AND acl.type_oid = member.objid
                ), '[]'::pg_catalog.jsonb)
            ) AS base_payload
    ) AS normalized
    WHERE member.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
),
candidate_extension_amops AS MATERIALIZED (
    SELECT member.extension_oid, member.objid AS family_oid, selected.amop_oid
    FROM candidate_extension_member_addresses AS member
    CROSS JOIN LATERAL (
        SELECT operator_member.oid AS amop_oid
        FROM pg_catalog.pg_amop AS operator_member
        WHERE operator_member.amopfamily = member.objid
        ORDER BY
            operator_member.amoplefttype,
            operator_member.amoprighttype,
            operator_member.amopstrategy,
            operator_member.amoppurpose,
            operator_member.oid
        LIMIT $1
    ) AS selected
    WHERE member.classid = 'pg_catalog.pg_opfamily'::pg_catalog.regclass
),
candidate_extension_amprocs AS MATERIALIZED (
    SELECT member.extension_oid, member.objid AS family_oid, selected.amproc_oid
    FROM candidate_extension_member_addresses AS member
    CROSS JOIN LATERAL (
        SELECT procedure_member.oid AS amproc_oid
        FROM pg_catalog.pg_amproc AS procedure_member
        WHERE procedure_member.amprocfamily = member.objid
        ORDER BY
            procedure_member.amproclefttype,
            procedure_member.amprocrighttype,
            procedure_member.amprocnum,
            procedure_member.oid
        LIMIT $1
    ) AS selected
    WHERE member.classid = 'pg_catalog.pg_opfamily'::pg_catalog.regclass
),
extension_amop_payloads AS (
    SELECT
        candidate.extension_oid,
        candidate.family_oid,
        pg_catalog.jsonb_build_object(
            'left_type', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_type'::pg_catalog.regclass,
                    operator_member.amoplefttype,
                    0
                ) AS identified
            ),
            'right_type', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_type'::pg_catalog.regclass,
                    operator_member.amoprighttype,
                    0
                ) AS identified
            ),
            'strategy', operator_member.amopstrategy,
            'purpose', operator_member.amoppurpose,
            'operator', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_operator'::pg_catalog.regclass,
                    operator_member.amopopr,
                    0
                ) AS identified
            ),
            'access_method', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_am'::pg_catalog.regclass,
                    operator_member.amopmethod,
                    0
                ) AS identified
            ),
            'sort_family', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_opfamily'::pg_catalog.regclass,
                    operator_member.amopsortfamily,
                    0
                ) AS identified
                WHERE operator_member.amopsortfamily <> 0
            )
        ) AS payload
    FROM candidate_extension_amops AS candidate
    JOIN pg_catalog.pg_amop AS operator_member ON operator_member.oid = candidate.amop_oid
),
extension_amproc_payloads AS (
    SELECT
        candidate.extension_oid,
        candidate.family_oid,
        pg_catalog.jsonb_build_object(
            'left_type', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_type'::pg_catalog.regclass,
                    procedure_member.amproclefttype,
                    0
                ) AS identified
            ),
            'right_type', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_type'::pg_catalog.regclass,
                    procedure_member.amprocrighttype,
                    0
                ) AS identified
            ),
            'procedure_number', procedure_member.amprocnum,
            'procedure', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    procedure_member.amproc,
                    0
                ) AS identified
            )
        ) AS payload
    FROM candidate_extension_amprocs AS candidate
    JOIN pg_catalog.pg_amproc AS procedure_member ON procedure_member.oid = candidate.amproc_oid
),
extension_access_method_payloads AS (
    SELECT
        member.extension_oid,
        member.classid,
        member.objid,
        member.objsubid,
        pg_catalog.jsonb_build_object(
            'name', access_method.amname,
            'kind', access_method.amtype,
            'handler', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    access_method.amhandler,
                    0
                ) AS identified
            )
        ) AS definition,
        '[]'::pg_catalog.jsonb AS current_acl
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_am AS access_method ON access_method.oid = member.objid
    WHERE member.classid = 'pg_catalog.pg_am'::pg_catalog.regclass
),
extension_cast_payloads AS (
    SELECT
        member.extension_oid,
        member.classid,
        member.objid,
        member.objsubid,
        pg_catalog.jsonb_build_object(
            'source', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_type'::pg_catalog.regclass,
                    cast_row.castsource,
                    0
                ) AS identified
            ),
            'target', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_type'::pg_catalog.regclass,
                    cast_row.casttarget,
                    0
                ) AS identified
            ),
            'function', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    cast_row.castfunc,
                    0
                ) AS identified
                WHERE cast_row.castfunc <> 0
            ),
            'context', cast_row.castcontext,
            'method', cast_row.castmethod
        ) AS definition,
        '[]'::pg_catalog.jsonb AS current_acl
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_cast AS cast_row ON cast_row.oid = member.objid
    WHERE member.classid = 'pg_catalog.pg_cast'::pg_catalog.regclass
),
extension_language_payloads AS (
    SELECT
        member.extension_oid,
        member.classid,
        member.objid,
        member.objsubid,
        pg_catalog.jsonb_build_object(
            'name', language.lanname,
            'owner', owner_identity.identity,
            'procedural', language.lanispl,
            'trusted', language.lanpltrusted,
            'call_handler', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    language.lanplcallfoid,
                    0
                ) AS identified
                WHERE language.lanplcallfoid <> 0
            ),
            'inline_handler', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    language.laninline,
                    0
                ) AS identified
                WHERE language.laninline <> 0
            ),
            'validator', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    language.lanvalidator,
                    0
                ) AS identified
                WHERE language.lanvalidator <> 0
            )
        ) AS definition,
        coalesce((
            SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                'grantor', acl.grantor_identity,
                'grantee', acl.grantee_identity,
                'privilege', acl.privilege_type,
                'grantable', acl.is_grantable
            ) ORDER BY
                acl.grantee_identity,
                acl.privilege_type,
                acl.grantor_identity,
                acl.is_grantable)
            FROM candidate_extension_language_acls AS acl
            WHERE acl.extension_oid = member.extension_oid
              AND acl.objid = member.objid
        ), '[]'::pg_catalog.jsonb) AS current_acl
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_language AS language ON language.oid = member.objid
    JOIN extension_role_identity_profile AS owner_identity
      ON owner_identity.role_oid = language.lanowner
    WHERE member.classid = 'pg_catalog.pg_language'::pg_catalog.regclass
),
extension_operator_payloads AS (
    SELECT
        member.extension_oid,
        member.classid,
        member.objid,
        member.objsubid,
        pg_catalog.jsonb_build_object(
            'name', operator_row.oprname,
            'owner', owner_identity.identity,
            'kind', operator_row.oprkind,
            'merge_joinable', operator_row.oprcanmerge,
            'hash_joinable', operator_row.oprcanhash,
            'left_type', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_type'::pg_catalog.regclass,
                    operator_row.oprleft,
                    0
                ) AS identified
                WHERE operator_row.oprleft <> 0
            ),
            'right_type', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_type'::pg_catalog.regclass,
                    operator_row.oprright,
                    0
                ) AS identified
                WHERE operator_row.oprright <> 0
            ),
            'result_type', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_type'::pg_catalog.regclass,
                    operator_row.oprresult,
                    0
                ) AS identified
            ),
            'commutator', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_operator'::pg_catalog.regclass,
                    operator_row.oprcom,
                    0
                ) AS identified
                WHERE operator_row.oprcom <> 0
            ),
            'negator', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_operator'::pg_catalog.regclass,
                    operator_row.oprnegate,
                    0
                ) AS identified
                WHERE operator_row.oprnegate <> 0
            ),
            'function', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    operator_row.oprcode,
                    0
                ) AS identified
            ),
            'restriction_estimator', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    operator_row.oprrest,
                    0
                ) AS identified
                WHERE operator_row.oprrest <> 0
            ),
            'join_estimator', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_proc'::pg_catalog.regclass,
                    operator_row.oprjoin,
                    0
                ) AS identified
                WHERE operator_row.oprjoin <> 0
            )
        ) AS definition,
        '[]'::pg_catalog.jsonb AS current_acl
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_operator AS operator_row ON operator_row.oid = member.objid
    JOIN extension_role_identity_profile AS owner_identity
      ON owner_identity.role_oid = operator_row.oprowner
    WHERE member.classid = 'pg_catalog.pg_operator'::pg_catalog.regclass
),
extension_operator_class_payloads AS (
    SELECT
        member.extension_oid,
        member.classid,
        member.objid,
        member.objsubid,
        pg_catalog.jsonb_build_object(
            'name', operator_class.opcname,
            'owner', owner_identity.identity,
            'access_method', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_am'::pg_catalog.regclass,
                    operator_class.opcmethod,
                    0
                ) AS identified
            ),
            'family', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_opfamily'::pg_catalog.regclass,
                    operator_class.opcfamily,
                    0
                ) AS identified
            ),
            'input_type', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_type'::pg_catalog.regclass,
                    operator_class.opcintype,
                    0
                ) AS identified
            ),
            'default', operator_class.opcdefault,
            'key_type', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_type'::pg_catalog.regclass,
                    operator_class.opckeytype,
                    0
                ) AS identified
                WHERE operator_class.opckeytype <> 0
            )
        ) AS definition,
        '[]'::pg_catalog.jsonb AS current_acl
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_opclass AS operator_class ON operator_class.oid = member.objid
    JOIN extension_role_identity_profile AS owner_identity
      ON owner_identity.role_oid = operator_class.opcowner
    WHERE member.classid = 'pg_catalog.pg_opclass'::pg_catalog.regclass
),
extension_operator_family_payloads AS (
    SELECT
        member.extension_oid,
        member.classid,
        member.objid,
        member.objsubid,
        pg_catalog.jsonb_build_object(
            'name', operator_family.opfname,
            'owner', owner_identity.identity,
            'access_method', (
                SELECT pg_catalog.jsonb_build_object(
                    'type', identified.type,
                    'names', pg_catalog.to_jsonb(identified.object_names),
                    'args', pg_catalog.to_jsonb(identified.object_args))
                FROM pg_catalog.pg_identify_object_as_address(
                    'pg_catalog.pg_am'::pg_catalog.regclass,
                    operator_family.opfmethod,
                    0
                ) AS identified
            ),
            'operators', coalesce((
                SELECT pg_catalog.jsonb_agg(operator_member.payload ORDER BY operator_member.payload::pg_catalog.text)
                FROM extension_amop_payloads AS operator_member
                WHERE operator_member.extension_oid = member.extension_oid
                  AND operator_member.family_oid = member.objid
            ), '[]'::pg_catalog.jsonb),
            'procedures', coalesce((
                SELECT pg_catalog.jsonb_agg(procedure_member.payload ORDER BY procedure_member.payload::pg_catalog.text)
                FROM extension_amproc_payloads AS procedure_member
                WHERE procedure_member.extension_oid = member.extension_oid
                  AND procedure_member.family_oid = member.objid
            ), '[]'::pg_catalog.jsonb)
        ) AS definition,
        '[]'::pg_catalog.jsonb AS current_acl
    FROM candidate_extension_member_addresses AS member
    JOIN pg_catalog.pg_opfamily AS operator_family ON operator_family.oid = member.objid
    JOIN extension_role_identity_profile AS owner_identity
      ON owner_identity.role_oid = operator_family.opfowner
    WHERE member.classid = 'pg_catalog.pg_opfamily'::pg_catalog.regclass
),
extension_member_payload_components AS MATERIALIZED (
    SELECT * FROM extension_access_method_payloads
    UNION ALL
    SELECT * FROM extension_cast_payloads
    UNION ALL
    SELECT * FROM extension_relation_payloads
    UNION ALL
    SELECT * FROM extension_language_payloads
    UNION ALL
    SELECT * FROM extension_operator_class_payloads
    UNION ALL
    SELECT * FROM extension_operator_payloads
    UNION ALL
    SELECT * FROM extension_operator_family_payloads
    UNION ALL
    SELECT * FROM extension_routine_payloads
    UNION ALL
    SELECT * FROM extension_type_payloads
),
extension_member_payloads AS (
    SELECT
        member.extension_oid,
        member.classid,
        member.objid,
        member.objsubid,
        pg_catalog.jsonb_build_object(
            'catalog', member.catalog_name,
            'address', member.address,
            'definition', component.definition,
            'current_acl', component.current_acl,
            'initial_acl', coalesce(initial.initial_acl, '[]'::pg_catalog.jsonb),
            'outbound_dependencies', coalesce(
                dependencies.outbound_dependencies,
                '[]'::pg_catalog.jsonb
            ),
            'inbound_dependencies', coalesce(
                dependencies.inbound_dependencies,
                '[]'::pg_catalog.jsonb
            )
        ) AS member_payload
    FROM extension_member_addresses AS member
    JOIN extension_member_payload_components AS component
      ON component.extension_oid = member.extension_oid
     AND component.classid = member.classid
     AND component.objid = member.objid
     AND component.objsubid = member.objsubid
    LEFT JOIN extension_initial_privileges AS initial
      ON initial.extension_oid = member.extension_oid
     AND initial.classid = member.classid
     AND initial.objid = member.objid
     AND initial.objsubid = member.objsubid
    LEFT JOIN extension_member_dependency_payloads AS dependencies
      ON dependencies.extension_oid = member.extension_oid
     AND dependencies.member_classid = member.classid
     AND dependencies.member_objid = member.objid
     AND dependencies.member_objsubid = member.objsubid
),
extension_member_counts AS (
    SELECT
        extension_row.oid AS extension_oid,
        (SELECT pg_catalog.count(*)
         FROM candidate_extension_members AS member
         WHERE member.extension_oid = extension_row.oid) AS raw_member_count,
        (SELECT pg_catalog.count(*)
         FROM safe_extension_members AS member
         WHERE member.extension_oid = extension_row.oid) AS safe_member_count,
        (SELECT pg_catalog.count(*)
         FROM extension_member_payloads AS member
         WHERE member.extension_oid = extension_row.oid) AS payload_member_count,
        (SELECT pg_catalog.count(*)
         FROM candidate_extension_members AS member
         WHERE member.extension_oid = extension_row.oid
           AND NOT EXISTS (
               SELECT 1
               FROM safe_extension_members AS safe
               WHERE safe.extension_oid = member.extension_oid
                 AND safe.classid = member.classid
                 AND safe.objid = member.objid
                 AND safe.objsubid = member.objsubid
           )) AS unsupported_member_count,
        (SELECT pg_catalog.count(*) - pg_catalog.count(DISTINCT (
             member.classid, member.objid, member.objsubid
         ))
         FROM extension_member_payloads AS member
         WHERE member.extension_oid = extension_row.oid) AS duplicate_payload_count
    FROM candidate_extensions AS candidate
    JOIN pg_catalog.pg_extension AS extension_row ON extension_row.oid = candidate.oid
),
extension_objects AS (
    SELECT
        'extension' AS kind,
        'pg_extension' AS schema_name,
        extension_row.extname AS object_name,
        pg_catalog.jsonb_build_object(
            'census_version', 1,
            'name', extension_row.extname,
            'version', extension_row.extversion,
            'schema', n.nspname,
            'owner_authority', owner_identity.identity,
            'relocatable', extension_row.extrelocatable,
            'config', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'address', config.address,
                    'condition', config.condition
                ) ORDER BY config.address::pg_catalog.text, config.condition)
                FROM extension_config_identities AS config
                WHERE config.extension_oid = extension_row.oid
            ), '[]'::pg_catalog.jsonb),
            'raw_member_count', counts.raw_member_count,
            'safe_member_count', counts.safe_member_count,
            'payload_member_count', counts.payload_member_count,
            'unsupported_member_count', counts.unsupported_member_count,
            'duplicate_payload_count', counts.duplicate_payload_count,
            'role_identity_count', role_counts.canonical_identity_count,
            'role_identities_complete', true,
            'member_payload_complete',
                counts.raw_member_count = counts.safe_member_count
                AND counts.safe_member_count = counts.payload_member_count
                AND counts.unsupported_member_count = 0
                AND counts.duplicate_payload_count = 0,
            'members', coalesce((
                SELECT pg_catalog.jsonb_agg(
                    member.member_payload
                    ORDER BY member.member_payload -> 'address'
                )
                FROM extension_member_payloads AS member
                WHERE member.extension_oid = extension_row.oid
            ), '[]'::pg_catalog.jsonb)
        ) AS payload
    FROM candidate_extensions AS candidate
    JOIN pg_catalog.pg_extension AS extension_row ON extension_row.oid = candidate.oid
    JOIN pg_catalog.pg_namespace AS n ON n.oid = extension_row.extnamespace
    JOIN extension_member_counts AS counts ON counts.extension_oid = extension_row.oid
    JOIN extension_role_identity_counts AS role_counts
      ON role_counts.extension_oid = extension_row.oid
     AND role_counts.referenced_role_oid_count = role_counts.resolved_role_oid_count
    JOIN extension_role_identity_profile AS owner_identity
      ON owner_identity.role_oid = extension_row.extowner
),
candidate_role_configs AS MATERIALIZED (
    SELECT role_row.oid AS role_oid, selected.value
    FROM pg_catalog.pg_roles AS role_row
    CROSS JOIN LATERAL (
        SELECT config_item.value
        FROM pg_catalog.unnest(coalesce(role_row.rolconfig, ARRAY[]::pg_catalog.text[]))
            AS config_item(value)
        ORDER BY config_item.value
        LIMIT $1
    ) AS selected
    WHERE role_row.rolname = 'babylon_intel'
),
candidate_parameter_acl_entries AS MATERIALIZED (
    SELECT
        parameter_acl.parname,
        CASE
            WHEN grantor_role.rolsuper THEN '$superuser'
            WHEN acl.grantor = role_row.oid THEN 'babylon_intel'
            ELSE pg_catalog.pg_get_userbyid(acl.grantor)
        END AS grantor_identity,
        CASE WHEN acl.grantee = 0 THEN 'PUBLIC' ELSE 'babylon_intel' END AS grantee_identity,
        acl.privilege_type,
        acl.is_grantable
    FROM pg_catalog.pg_parameter_acl AS parameter_acl
    CROSS JOIN LATERAL pg_catalog.aclexplode(parameter_acl.paracl) AS acl
    JOIN pg_catalog.pg_roles AS role_row ON role_row.rolname = 'babylon_intel'
    LEFT JOIN pg_catalog.pg_roles AS grantor_role ON grantor_role.oid = acl.grantor
    WHERE acl.grantee = 0 OR acl.grantee = role_row.oid
    ORDER BY
        parameter_acl.parname,
        CASE WHEN acl.grantee = 0 THEN 'PUBLIC' ELSE 'babylon_intel' END,
        acl.privilege_type,
        CASE
            WHEN grantor_role.rolsuper THEN '$superuser'
            WHEN acl.grantor = role_row.oid THEN 'babylon_intel'
            ELSE pg_catalog.pg_get_userbyid(acl.grantor)
        END,
        acl.is_grantable
    LIMIT $1
),
candidate_role_memberships AS MATERIALIZED (
    SELECT
        membership.roleid,
        membership.member,
        membership.grantor,
        membership.admin_option,
        membership.inherit_option,
        membership.set_option
    FROM pg_catalog.pg_auth_members AS membership
    JOIN pg_catalog.pg_roles AS granted_role ON granted_role.oid = membership.roleid
    JOIN pg_catalog.pg_roles AS member_role ON member_role.oid = membership.member
    CROSS JOIN database_owner AS own
    WHERE granted_role.rolname = 'babylon_intel'
       OR member_role.rolname = 'babylon_intel'
    ORDER BY
        granted_role.rolname,
        member_role.rolname,
        CASE
            WHEN membership.grantor = own.owner_oid THEN '$database_owner'
            ELSE pg_catalog.pg_get_userbyid(membership.grantor)
        END,
        membership.admin_option,
        membership.inherit_option,
        membership.set_option
    LIMIT $1
),
candidate_default_acl_entries AS MATERIALIZED (
    SELECT normalized.*
    FROM (
        SELECT
            CASE
                WHEN defaults.defaclrole = own.owner_oid THEN '$database_owner'
                WHEN defaults.defaclrole = intel_role.oid THEN 'babylon_intel'
                ELSE '$other_owner:' || owner_role.rolname
            END AS owner_identity,
            coalesce(default_ns.nspname, '') AS schema_name,
            defaults.defaclobjtype AS object_type,
            CASE
                WHEN acl.grantor = own.owner_oid THEN '$database_owner'
                WHEN acl.grantor = intel_role.oid THEN 'babylon_intel'
                WHEN grantor_role.rolsuper THEN '$superuser'
                ELSE '$other_owner:' || grantor_role.rolname
            END AS grantor_identity,
            CASE
                WHEN acl.grantee = 0 THEN 'PUBLIC'
                WHEN acl.grantee = own.owner_oid THEN '$database_owner'
                WHEN acl.grantee = intel_role.oid THEN 'babylon_intel'
                WHEN grantee_role.rolsuper THEN '$superuser'
                ELSE '$other_owner:' || grantee_role.rolname
            END AS grantee_identity,
            acl.privilege_type,
            acl.is_grantable
        FROM pg_catalog.pg_default_acl AS defaults
        CROSS JOIN database_owner AS own
        JOIN pg_catalog.pg_roles AS intel_role ON intel_role.rolname = 'babylon_intel'
        JOIN pg_catalog.pg_roles AS owner_role ON owner_role.oid = defaults.defaclrole
        LEFT JOIN pg_catalog.pg_namespace AS default_ns
          ON default_ns.oid = defaults.defaclnamespace
        CROSS JOIN LATERAL pg_catalog.aclexplode(defaults.defaclacl) AS acl
        JOIN pg_catalog.pg_roles AS grantor_role ON grantor_role.oid = acl.grantor
        LEFT JOIN pg_catalog.pg_roles AS grantee_role ON grantee_role.oid = acl.grantee
        WHERE defaults.defaclrole IN (own.owner_oid, intel_role.oid)
    ) AS normalized
    ORDER BY
        normalized.owner_identity,
        normalized.schema_name,
        normalized.object_type,
        normalized.grantee_identity,
        normalized.privilege_type,
        normalized.grantor_identity,
        normalized.is_grantable
    LIMIT $1
),
role_objects AS (
    SELECT
        'role' AS kind,
        'pg_roles' AS schema_name,
        role_row.rolname AS object_name,
        pg_catalog.jsonb_build_object(
            'census_version', 1,
            'name', role_row.rolname,
            'superuser', role_row.rolsuper,
            'inherit', role_row.rolinherit,
            'create_role', role_row.rolcreaterole,
            'create_db', role_row.rolcreatedb,
            'can_login', role_row.rolcanlogin,
            'replication', role_row.rolreplication,
            'bypass_rls', role_row.rolbypassrls,
            'connection_limit', role_row.rolconnlimit,
            'valid_until', coalesce(role_row.rolvaliduntil::pg_catalog.text, ''),
            'config', coalesce((
                SELECT pg_catalog.jsonb_agg(config_item.value ORDER BY config_item.value)
                FROM candidate_role_configs AS config_item
                WHERE config_item.role_oid = role_row.oid
            ), '[]'::pg_catalog.jsonb),
            'parameter_privileges', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'parameter', parameter_acl.parname,
                    'grantor', parameter_acl.grantor_identity,
                    'grantee', parameter_acl.grantee_identity,
                    'privilege', parameter_acl.privilege_type,
                    'grantable', parameter_acl.is_grantable
                ) ORDER BY
                    parameter_acl.parname,
                    parameter_acl.grantee_identity,
                    parameter_acl.privilege_type,
                    parameter_acl.grantor_identity,
                    parameter_acl.is_grantable)
                FROM candidate_parameter_acl_entries AS parameter_acl
            ), '[]'::pg_catalog.jsonb),
            'memberships', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'role', granted_role.rolname,
                    'member', member_role.rolname,
                    'grantor', CASE
                        WHEN membership.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(membership.grantor)
                    END,
                    'admin', membership.admin_option,
                    'inherit', membership.inherit_option,
                    'set', membership.set_option
                ) ORDER BY
                    granted_role.rolname,
                    member_role.rolname,
                    CASE
                        WHEN membership.grantor = own.owner_oid THEN '$database_owner'
                        ELSE pg_catalog.pg_get_userbyid(membership.grantor)
                    END,
                    membership.admin_option,
                    membership.inherit_option,
                    membership.set_option)
                FROM candidate_role_memberships AS membership
                JOIN pg_catalog.pg_roles AS granted_role ON granted_role.oid = membership.roleid
                JOIN pg_catalog.pg_roles AS member_role ON member_role.oid = membership.member
            ), '[]'::pg_catalog.jsonb),
            'default_privileges', coalesce((
                SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_object(
                    'owner', defaults.owner_identity,
                    'schema', defaults.schema_name,
                    'object_type', defaults.object_type,
                    'grantor', defaults.grantor_identity,
                    'grantee', defaults.grantee_identity,
                    'privilege', defaults.privilege_type,
                    'grantable', defaults.is_grantable
                ) ORDER BY
                    defaults.owner_identity,
                    defaults.schema_name,
                    defaults.object_type,
                    defaults.grantee_identity,
                    defaults.privilege_type,
                    defaults.grantor_identity,
                    defaults.is_grantable)
                FROM candidate_default_acl_entries AS defaults
            ), '[]'::pg_catalog.jsonb)
        ) AS payload
    FROM pg_catalog.pg_roles AS role_row
    CROSS JOIN database_owner AS own
    WHERE role_row.rolname = 'babylon_intel'
),
unsupported_catalog_objects AS (
    SELECT
        'unsupported_catalog' AS kind,
        'pg_catalog' AS schema_name,
        candidate.family AS object_name,
        pg_catalog.jsonb_build_object(
            'census_version', 1,
            'family', candidate.family
        ) AS payload
    FROM candidate_unsupported_catalog AS candidate
    GROUP BY candidate.family
),
objects AS (
    SELECT * FROM database_objects
    UNION ALL
    SELECT * FROM rel_objects
    UNION ALL
    SELECT * FROM domain_objects
    UNION ALL
    SELECT * FROM extension_objects
    UNION ALL
    SELECT * FROM routine_objects
    UNION ALL
    SELECT * FROM schema_objects
    UNION ALL
    SELECT * FROM schema_grants
    UNION ALL
    SELECT * FROM user_type_objects
    UNION ALL
    SELECT * FROM role_objects
    UNION ALL
    SELECT * FROM unsupported_catalog_objects
),
overflow_candidates AS MATERIALIZED (
    SELECT 10 AS priority, 'sequence_ownership'::pg_catalog.text AS resource,
           pg_catalog.count(*) AS actual, $5 - 1 AS max_value
    FROM candidate_sequence_dependencies AS dependency
    GROUP BY dependency.relation_oid
    UNION ALL
    SELECT 20, 'extension_members', budget.extension_member_count, $3 - 1
    FROM extension_member_budget AS budget
    UNION ALL
    SELECT 25, 'extension_dependency_addresses', pg_catalog.count(*), $4 - 1
    FROM (SELECT 1 FROM extension_address_budget_rows LIMIT $4) AS bounded_addresses
    UNION ALL
    SELECT 27, 'extension_role_identities', pg_catalog.count(*), $6 - 1
    FROM candidate_extension_role_references
    UNION ALL
    SELECT 30, 'partition_rows', pg_catalog.count(*), $2 - 1
    FROM governed_child_relations AS child
    GROUP BY child.parent_oid
    UNION ALL
    SELECT 100, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_database_setting_configs AS settings
    GROUP BY settings.setrole
    UNION ALL
    SELECT 110, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_extension_configs AS config
    GROUP BY config.extension_oid
    UNION ALL
    SELECT 111, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_extension_initial_privileges AS initial
    GROUP BY initial.extension_oid, initial.classid, initial.objid, initial.objsubid
    UNION ALL
    SELECT 112, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_extension_initial_acl_entries AS acl
    GROUP BY
        acl.extension_oid,
        acl.classid,
        acl.objid,
        acl.objsubid,
        acl.initial_objsubid,
        acl.privilege_type
    UNION ALL
    SELECT 113, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_extension_language_acls AS acl
    GROUP BY acl.extension_oid, acl.classid, acl.objid, acl.objsubid
    UNION ALL
    SELECT 114, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_extension_amops AS operator_member
    GROUP BY operator_member.extension_oid, operator_member.family_oid
    UNION ALL
    SELECT 115, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_extension_amprocs AS procedure_member
    GROUP BY procedure_member.extension_oid, procedure_member.family_oid
    UNION ALL
    SELECT 116, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_extension_routine_transforms AS transform
    GROUP BY transform.extension_oid, transform.routine_oid
    UNION ALL
    SELECT 120, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_schema_acls AS acl
    GROUP BY acl.namespace_oid
    UNION ALL
    SELECT 130, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_domain_acls AS acl
    GROUP BY acl.domain_oid
    UNION ALL
    SELECT 140, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_domain_constraints AS constraint_row
    GROUP BY constraint_row.domain_oid
    UNION ALL
    SELECT 150, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_routine_configs AS config
    GROUP BY config.routine_oid
    UNION ALL
    SELECT 160, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_routine_acls AS acl
    GROUP BY acl.routine_oid
    UNION ALL
    SELECT 170, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_user_type_acls AS acl
    GROUP BY acl.type_oid
    UNION ALL
    SELECT 180, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_enum_labels AS label
    GROUP BY label.type_oid
    UNION ALL
    SELECT 190, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_composite_attributes AS attribute
    GROUP BY attribute.type_oid
    UNION ALL
    SELECT 200, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_attributes AS attribute
    GROUP BY attribute.relation_oid
    UNION ALL
    SELECT 210, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_relation_options AS option_row
    GROUP BY option_row.relation_oid
    UNION ALL
    SELECT 220, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_relation_acls AS acl
    GROUP BY acl.relation_oid
    UNION ALL
    SELECT 230, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_constraints AS constraint_row
    GROUP BY constraint_row.relation_oid
    UNION ALL
    SELECT 240, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_policies AS policy
    GROUP BY policy.relation_oid
    UNION ALL
    SELECT 250, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_rules AS rule_row
    GROUP BY rule_row.relation_oid
    UNION ALL
    SELECT 260, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_triggers AS trigger_row
    GROUP BY trigger_row.relation_oid
    UNION ALL
    SELECT 270, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_indexes AS index_row
    GROUP BY index_row.relation_oid
    UNION ALL
    SELECT 280, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_column_acls AS acl
    GROUP BY acl.relation_oid, acl.attnum
    UNION ALL
    SELECT 290, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_attribute_options AS option_row
    GROUP BY option_row.relation_oid, option_row.attnum
    UNION ALL
    SELECT 300, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_attribute_fdw_options AS option_row
    GROUP BY option_row.relation_oid, option_row.attnum
    UNION ALL
    SELECT 310, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_policy_roles AS role_row
    GROUP BY role_row.relation_oid, role_row.policy_oid
    UNION ALL
    SELECT 320, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_index_options AS option_row
    GROUP BY option_row.index_oid
    UNION ALL
    SELECT 330, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_relation_parents AS parent
    GROUP BY parent.relation_oid
    UNION ALL
    SELECT 400, 'catalog_rows', pg_catalog.count(*), $1 - 1 FROM candidate_namespaces
    UNION ALL
    SELECT 410, 'catalog_rows', pg_catalog.count(*), $1 - 1 FROM candidate_relations
    UNION ALL
    SELECT 420, 'catalog_rows', pg_catalog.count(*), $1 - 1 FROM candidate_domains
    UNION ALL
    SELECT 430, 'catalog_rows', pg_catalog.count(*), $1 - 1 FROM candidate_extensions
    UNION ALL
    SELECT 440, 'catalog_rows', pg_catalog.count(*), $1 - 1 FROM candidate_routines
    UNION ALL
    SELECT 450, 'catalog_rows', pg_catalog.count(*), $1 - 1 FROM candidate_user_types
    UNION ALL
    SELECT 460, 'catalog_rows', pg_catalog.count(*), $1 - 1 FROM database_settings
    UNION ALL
    SELECT 470, 'catalog_rows', pg_catalog.count(*), $1 - 1 FROM candidate_database_acls
    UNION ALL
    SELECT 480, 'catalog_rows', pg_catalog.count(*), $1 - 1 FROM candidate_role_configs
    UNION ALL
    SELECT 490, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM candidate_parameter_acl_entries
    UNION ALL
    SELECT 500, 'catalog_rows', pg_catalog.count(*), $1 - 1 FROM candidate_role_memberships
    UNION ALL
    SELECT 510, 'catalog_rows', pg_catalog.count(*), $1 - 1 FROM candidate_default_acl_entries
    UNION ALL
    SELECT 520, 'catalog_rows', pg_catalog.count(*), $1 - 1
    FROM (SELECT 1 FROM candidate_unsupported_catalog LIMIT $1) AS bounded_unsupported
    UNION ALL
    SELECT 900, 'census_rows', pg_catalog.count(*), $1 - 1
    FROM (SELECT 1 FROM objects LIMIT $1) AS bounded_objects
),
catalog_overflow AS MATERIALIZED (
    SELECT candidate.resource AS overflow_resource,
           candidate.actual AS overflow_actual,
           candidate.max_value AS overflow_max
    FROM overflow_candidates AS candidate
    WHERE candidate.actual > candidate.max_value
    ORDER BY candidate.priority, candidate.actual DESC, candidate.resource
    LIMIT 1
),
catalog_status AS (
    SELECT overflow.overflow_resource, overflow.overflow_actual, overflow.overflow_max
    FROM (SELECT 1 AS anchor) AS status_anchor
    LEFT JOIN catalog_overflow AS overflow ON true
),
catalog_output AS (
    SELECT
        objects.kind,
        objects.schema_name,
        objects.object_name,
        pg_catalog.encode(
            pg_catalog.sha256(pg_catalog.convert_to(objects.payload::pg_catalog.text, 'UTF8')),
            'hex'
        ) AS digest_hex,
        status.overflow_resource,
        status.overflow_actual,
        status.overflow_max
    FROM objects
    CROSS JOIN catalog_status AS status
    WHERE status.overflow_resource IS NULL
    UNION ALL
    SELECT
        NULL::pg_catalog.text AS kind,
        NULL::pg_catalog.text AS schema_name,
        NULL::pg_catalog.text AS object_name,
        NULL::pg_catalog.text AS digest_hex,
        status.overflow_resource,
        status.overflow_actual,
        status.overflow_max
    FROM catalog_status AS status
    WHERE status.overflow_resource IS NOT NULL
)
SELECT
    output.kind,
    output.schema_name,
    output.object_name,
    output.digest_hex,
    output.overflow_resource,
    output.overflow_actual,
    output.overflow_max
FROM catalog_output AS output
ORDER BY output.kind, output.schema_name, output.object_name
LIMIT $1
