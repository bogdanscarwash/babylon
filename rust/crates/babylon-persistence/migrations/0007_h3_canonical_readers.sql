-- Replace the frozen Python reader views only after every legacy H3 identity
-- has an exact canonical BIGINT shadow. A fresh Rust-owned database has none
-- of the 15 public predecessor relations and therefore takes the no-op lane.
DO $h3_canonical_reader_preflight$
DECLARE
    named_relation_count BIGINT;
    present_relation_count BIGINT;
    named_view_count BIGINT;
    present_view_count BIGINT;
    shadow_column_count BIGINT;
    invalid_row_count BIGINT;
    mapping TEXT[];
BEGIN
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
    )
    SELECT pg_catalog.count(*) FILTER (WHERE relation.oid IS NOT NULL),
           pg_catalog.count(*) FILTER (
               WHERE relation.oid IS NOT NULL
                 AND relation.relkind IN ('r', 'p')
           )
      INTO named_relation_count, present_relation_count
      FROM expected_relations AS expected
      LEFT JOIN pg_catalog.pg_class AS relation
        ON relation.oid = pg_catalog.to_regclass(
            pg_catalog.format('public.%I', expected.relation_name)
        );

    IF named_relation_count NOT IN (0, 15)
       OR present_relation_count <> named_relation_count THEN
        RAISE EXCEPTION
            'epoch 7 requires either zero or all 15 exact governed H3 relations; found % names and % exact relations',
            named_relation_count,
            present_relation_count;
    END IF;
    IF present_relation_count = 0 THEN
        RETURN;
    END IF;

    WITH expected_views(view_name) AS (
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
    )
    SELECT pg_catalog.count(*) FILTER (WHERE relation.oid IS NOT NULL),
           pg_catalog.count(*) FILTER (
               WHERE relation.oid IS NOT NULL AND relation.relkind = 'v'
           )
      INTO named_view_count, present_view_count
      FROM expected_views AS expected
      LEFT JOIN pg_catalog.pg_class AS relation
        ON relation.oid = pg_catalog.to_regclass(
            pg_catalog.format('public.%I', expected.view_name)
        );

    IF named_view_count <> 10 OR present_view_count <> named_view_count THEN
        RAISE EXCEPTION
            'epoch 7 requires all 10 exact governed reader views; found % names and % exact views',
            named_view_count,
            present_view_count;
    END IF;

    WITH expected_columns(relation_name, column_name) AS (
        VALUES
            ('dynamic_hex_state'::pg_catalog.text, 'cell_id'::pg_catalog.text),
            ('hex_activity', 'cell_id'),
            ('hex_cell', 'cell_id'),
            ('hex_cell', 'ancestor_r5'),
            ('hex_cell', 'ancestor_r6'),
            ('hex_latest', 'cell_id'),
            ('hex_map', 'cell_id'),
            ('hex_r8_linear_features_reference', 'cell_id'),
            ('hex_r8_reference', 'cell_id'),
            ('hex_r8_reference', 'parent_cell_id'),
            ('hex_spatial_map', 'cell_id'),
            ('hex_state', 'cell_id'),
            ('hex_substrate', 'cell_id'),
            ('hex_substrate', 'ancestor_r7'),
            ('hex_terrain_state', 'cell_id'),
            ('immutable_reference_lodes_od_matrix', 'home_cell_id'),
            ('immutable_reference_lodes_od_matrix', 'workplace_cell_id'),
            ('infrastructure_link_state', 'source_cell_id'),
            ('infrastructure_link_state', 'target_cell_id'),
            ('org_snapshot', 'home_cell_id'),
            ('tick_event', 'cell_id')
    ),
    exact_columns AS (
        SELECT attribute.attrelid, attribute.attnum
          FROM expected_columns AS expected
          JOIN pg_catalog.pg_attribute AS attribute
            ON attribute.attrelid = pg_catalog.to_regclass(
                pg_catalog.format('public.%I', expected.relation_name)
            )
           AND attribute.attname = expected.column_name
           AND attribute.attnum > 0
           AND NOT attribute.attisdropped
           AND attribute.atttypid = 'pg_catalog.int8'::pg_catalog.regtype
           AND NOT attribute.attnotnull
           AND NOT attribute.atthasdef
           AND attribute.attidentity = ''
           AND attribute.attgenerated = ''
    )
    SELECT pg_catalog.count(*)
      INTO shadow_column_count
      FROM exact_columns;

    IF shadow_column_count <> 21 THEN
        RAISE EXCEPTION
            'epoch 7 requires the exact 21-column nullable BIGINT shadow shape; found %',
            shadow_column_count;
    END IF;

    FOREACH mapping SLICE 1 IN ARRAY ARRAY[
        ['dynamic_hex_state', 'h3_index', 'cell_id', 'required'],
        ['hex_activity', 'h3_index', 'cell_id', 'required'],
        ['hex_cell', 'h3_index', 'cell_id', 'required'],
        ['hex_cell', 'res5_parent', 'ancestor_r5', 'required'],
        ['hex_cell', 'res6_parent', 'ancestor_r6', 'required'],
        ['hex_latest', 'h3_index', 'cell_id', 'required'],
        ['hex_map', 'h3_index', 'cell_id', 'required'],
        ['hex_r8_linear_features_reference', 'h3_index', 'cell_id', 'required'],
        ['hex_r8_reference', 'h3_index', 'cell_id', 'required'],
        ['hex_r8_reference', 'parent_h3', 'parent_cell_id', 'required'],
        ['hex_spatial_map', 'h3_index', 'cell_id', 'required'],
        ['hex_state', 'h3_index', 'cell_id', 'required'],
        ['hex_substrate', 'h3_index', 'cell_id', 'required'],
        ['hex_substrate', 'r7_parent', 'ancestor_r7', 'required'],
        ['hex_terrain_state', 'h3_index', 'cell_id', 'required'],
        ['immutable_reference_lodes_od_matrix', 'home_hex', 'home_cell_id', 'required'],
        ['immutable_reference_lodes_od_matrix', 'workplace_dest', 'workplace_cell_id', 'tagged'],
        ['infrastructure_link_state', 'source_h3', 'source_cell_id', 'required'],
        ['infrastructure_link_state', 'target_h3', 'target_cell_id', 'required'],
        ['org_snapshot', 'home_hex', 'home_cell_id', 'nullable'],
        ['tick_event', 'h3_index', 'cell_id', 'nullable']
    ]
    LOOP
        IF mapping[4] = 'required' THEN
            EXECUTE pg_catalog.format(
                'SELECT pg_catalog.count(*) FROM public.%1$I AS source '
                'WHERE source.%2$I IS NULL OR source.%3$I IS NULL '
                'OR source.%2$I !~ ''^[0-9a-f]{15}$'' '
                'OR source.%3$I <= 0 '
                'OR source.%3$I <> '
                '(((''x'' || pg_catalog.lpad(source.%2$I, 16, ''0''))'
                '::pg_catalog.bit(64))::pg_catalog.int8)',
                mapping[1],
                mapping[2],
                mapping[3]
            ) INTO invalid_row_count;
        ELSIF mapping[4] = 'nullable' THEN
            EXECUTE pg_catalog.format(
                'SELECT pg_catalog.count(*) FROM public.%1$I AS source '
                'WHERE (source.%2$I IS NULL) <> (source.%3$I IS NULL) '
                'OR (source.%2$I IS NOT NULL AND ('
                'source.%2$I !~ ''^[0-9a-f]{15}$'' '
                'OR source.%3$I <= 0 '
                'OR source.%3$I <> '
                '(((''x'' || pg_catalog.lpad(source.%2$I, 16, ''0''))'
                '::pg_catalog.bit(64))::pg_catalog.int8)))',
                mapping[1],
                mapping[2],
                mapping[3]
            ) INTO invalid_row_count;
        ELSE
            EXECUTE pg_catalog.format(
                'SELECT pg_catalog.count(*) FROM public.%1$I AS source '
                'WHERE CASE source.workplace_dest_kind '
                'WHEN ''hex'' THEN source.%2$I IS NULL '
                'OR source.%2$I !~ ''^[0-9a-f]{15}$'' '
                'OR source.%3$I IS NULL OR source.%3$I <= 0 '
                'OR source.%3$I <> '
                '(((''x'' || pg_catalog.lpad(source.%2$I, 16, ''0''))'
                '::pg_catalog.bit(64))::pg_catalog.int8) '
                'WHEN ''external'' THEN source.%2$I IS NULL '
                'OR source.%2$I NOT IN (''canada'', ''rest_of_usa'') '
                'OR source.%3$I IS NOT NULL '
                'ELSE TRUE END',
                mapping[1],
                mapping[2],
                mapping[3]
            ) INTO invalid_row_count;
        END IF;

        IF invalid_row_count <> 0 THEN
            RAISE EXCEPTION
                'epoch 7 refuses non-canonical H3 identity %.% -> % (% invalid rows)',
                mapping[1],
                mapping[2],
                mapping[3],
                invalid_row_count;
        END IF;
    END LOOP;
END
$h3_canonical_reader_preflight$;

-- The fresh-origin RETURN above cannot guard top-level SQL statements.  Keep
-- every replacement inside a second, independently guarded procedural unit so
-- the zero-relation lane remains a literal no-op.
DO $h3_canonical_reader_views$
DECLARE
    present_relation_count pg_catalog.int8;
BEGIN
    SELECT pg_catalog.count(*)
      INTO present_relation_count
      FROM pg_catalog.unnest(ARRAY[
          'dynamic_hex_state',
          'hex_activity',
          'hex_cell',
          'hex_latest',
          'hex_map',
          'hex_r8_linear_features_reference',
          'hex_r8_reference',
          'hex_spatial_map',
          'hex_state',
          'hex_substrate',
          'hex_terrain_state',
          'immutable_reference_lodes_od_matrix',
          'infrastructure_link_state',
          'org_snapshot',
          'tick_event'
      ]::pg_catalog.text[]) AS expected(relation_name)
     WHERE pg_catalog.to_regclass(
               pg_catalog.format('public.%I', expected.relation_name)
           ) IS NOT NULL;

    IF present_relation_count = 0 THEN
        RETURN;
    END IF;
    IF present_relation_count <> 15 THEN
        RAISE EXCEPTION
            'epoch 7 view replacement requires all 15 governed H3 relations';
    END IF;

    EXECUTE 'DROP VIEW public.v_county_value_aggregate';
    EXECUTE 'DROP VIEW public.v_hex_aid';
    EXECUTE 'DROP VIEW public.v_hex_economic';
    EXECUTE 'DROP VIEW public.v_hex_heat';
    EXECUTE 'DROP VIEW public.v_hex_intel';
    EXECUTE 'DROP VIEW public.v_hex_mobilize';
    EXECUTE 'DROP VIEW public.v_hex_state_asof';
    EXECUTE 'DROP VIEW public.v_national_value_aggregate';
    EXECUTE 'DROP VIEW public.v_state_value_aggregate';
    EXECUTE 'DROP VIEW public.view_runtime_trace_emission';

    EXECUTE $view$
        CREATE VIEW public.v_hex_economic AS
        SELECT game_id, tick, cell_id, center_lat, center_lng,
               county_fips, county_name,
               profit_rate, exploitation_rate, occ, imperial_rent,
               g33_visibility, pop_total, heat
        FROM public.hex_latest
    $view$;

    EXECUTE $view$
        CREATE VIEW public.v_hex_mobilize AS
        SELECT game_id, tick, cell_id, center_lat, center_lng,
               county_fips,
               pop_proletariat + pop_lumpenproletariat AS mobilizable_pop,
               pop_labor_aristocracy, heat,
               org_count AS org_presence, heat_delta AS hex_heat,
               org_ids
        FROM public.hex_latest
    $view$;

    EXECUTE $view$
        CREATE VIEW public.v_hex_aid AS
        SELECT game_id, tick, cell_id, center_lat, center_lng,
               county_fips,
               pop_lumpenproletariat, pop_proletariat,
               imperial_rent, g33_visibility,
               attributes->'reproduction_deficit' AS reproduction_deficit
        FROM public.hex_latest
    $view$;

    EXECUTE $view$
        CREATE VIEW public.v_hex_heat AS
        SELECT game_id, tick, cell_id, center_lat, center_lng,
               heat AS heat_total, heat_delta,
               org_count, was_target
        FROM public.hex_latest
        WHERE heat > 0
    $view$;

    EXECUTE $view$
        CREATE VIEW public.v_hex_intel AS
        SELECT game_id, tick, cell_id, center_lat, center_lng,
               county_fips, county_name,
               profit_rate, exploitation_rate, occ, imperial_rent,
               g33_visibility,
               pop_bourgeoisie, pop_petit_bourgeoisie,
               pop_labor_aristocracy, pop_proletariat,
               pop_lumpenproletariat, pop_total,
               heat,
               faction_finance_capital, faction_security_state,
               faction_settler_populist,
               org_ids, org_count,
               heat AS hex_heat
        FROM public.hex_latest
    $view$;

    EXECUTE $view$
        CREATE VIEW public.v_hex_state_asof AS
        WITH spine AS (
            SELECT session_id, tick FROM public.tick_commit
            UNION
            SELECT DISTINCT session_id, tick FROM public.dynamic_hex_state
        ),
        intervals AS (
            SELECT h.session_id, h.cell_id, h.tick,
                   COALESCE(m.county_fips, h.county_fips) AS county_fips,
                   COALESCE(m.state_fips, h.state_fips) AS state_fips,
                   COALESCE(m.region_id, h.region_id) AS region_id,
                   h.c, h.v, h.s, h.k,
                   h.biocapacity_stock, h.energy_stock, h.raw_material_stock,
                   h.internet_access_pct, h.surveillance_coupling,
                   LEAD(h.tick) OVER (
                       PARTITION BY h.session_id, h.cell_id ORDER BY h.tick
                   ) AS next_tick
            FROM public.dynamic_hex_state AS h
            LEFT JOIN public.hex_spatial_map AS m
              ON m.cell_id = h.cell_id AND m.session_id = h.session_id
        )
        SELECT sp.session_id, sp.tick,
               hi.cell_id, hi.county_fips, hi.state_fips, hi.region_id,
               hi.c, hi.v, hi.s, hi.k,
               hi.biocapacity_stock, hi.energy_stock, hi.raw_material_stock,
               hi.internet_access_pct, hi.surveillance_coupling,
               hi.tick AS written_at_tick
        FROM spine AS sp
        JOIN intervals AS hi
          ON hi.session_id = sp.session_id
         AND hi.tick <= sp.tick
         AND (hi.next_tick IS NULL OR sp.tick < hi.next_tick)
    $view$;

    EXECUTE $view$
        CREATE VIEW public.v_county_value_aggregate AS
        WITH change_ticks AS (
            SELECT DISTINCT session_id, tick FROM public.dynamic_hex_state
        ),
        intervals AS (
            SELECT h.session_id, h.cell_id, h.tick,
                   COALESCE(m.county_fips, h.county_fips) AS county_fips,
                   h.c, h.v, h.s, h.k, h.biocapacity_stock,
                   LEAD(h.tick) OVER (
                       PARTITION BY h.session_id, h.cell_id ORDER BY h.tick
                   ) AS next_tick
            FROM public.dynamic_hex_state AS h
            LEFT JOIN public.hex_spatial_map AS m
              ON m.cell_id = h.cell_id AND m.session_id = h.session_id
        ),
        county_events AS (
            SELECT ct.session_id, ct.tick, hi.county_fips,
                   SUM(hi.c) AS c_sum,
                   SUM(hi.v) AS v_sum,
                   SUM(hi.s) AS s_sum,
                   SUM(hi.k) AS k_sum,
                   SUM(hi.biocapacity_stock) AS biocapacity_sum,
                   COUNT(*) AS hex_count
            FROM change_ticks AS ct
            JOIN intervals AS hi
              ON hi.session_id = ct.session_id
             AND hi.tick <= ct.tick
             AND (hi.next_tick IS NULL OR ct.tick < hi.next_tick)
            GROUP BY ct.session_id, ct.tick, hi.county_fips
        ),
        county_intervals AS (
            SELECT ce.*,
                   LEAD(ce.tick) OVER (
                       PARTITION BY ce.session_id, ce.county_fips ORDER BY ce.tick
                   ) AS next_tick
            FROM county_events AS ce
        ),
        spine AS (
            SELECT session_id, tick FROM public.tick_commit
            UNION
            SELECT DISTINCT session_id, tick FROM public.dynamic_hex_state
        )
        SELECT sp.session_id, sp.tick, ci.county_fips,
               ci.c_sum, ci.v_sum, ci.s_sum, ci.k_sum, ci.biocapacity_sum,
               ci.hex_count
        FROM spine AS sp
        JOIN county_intervals AS ci
          ON ci.session_id = sp.session_id
         AND ci.tick <= sp.tick
         AND (ci.next_tick IS NULL OR sp.tick < ci.next_tick)
    $view$;

    EXECUTE $view$
        CREATE VIEW public.v_state_value_aggregate AS
        WITH change_ticks AS (
            SELECT DISTINCT session_id, tick FROM public.dynamic_hex_state
        ),
        intervals AS (
            SELECT h.session_id, h.cell_id, h.tick,
                   COALESCE(m.state_fips, h.state_fips) AS state_fips,
                   h.c, h.v, h.s, h.k, h.biocapacity_stock,
                   LEAD(h.tick) OVER (
                       PARTITION BY h.session_id, h.cell_id ORDER BY h.tick
                   ) AS next_tick
            FROM public.dynamic_hex_state AS h
            LEFT JOIN public.hex_spatial_map AS m
              ON m.cell_id = h.cell_id AND m.session_id = h.session_id
        ),
        state_events AS (
            SELECT ct.session_id, ct.tick, hi.state_fips,
                   SUM(hi.c) AS c_sum,
                   SUM(hi.v) AS v_sum,
                   SUM(hi.s) AS s_sum,
                   SUM(hi.k) AS k_sum,
                   SUM(hi.biocapacity_stock) AS biocapacity_sum,
                   COUNT(*) AS hex_count
            FROM change_ticks AS ct
            JOIN intervals AS hi
              ON hi.session_id = ct.session_id
             AND hi.tick <= ct.tick
             AND (hi.next_tick IS NULL OR ct.tick < hi.next_tick)
            GROUP BY ct.session_id, ct.tick, hi.state_fips
        ),
        state_intervals AS (
            SELECT se.*,
                   LEAD(se.tick) OVER (
                       PARTITION BY se.session_id, se.state_fips ORDER BY se.tick
                   ) AS next_tick
            FROM state_events AS se
        ),
        spine AS (
            SELECT session_id, tick FROM public.tick_commit
            UNION
            SELECT DISTINCT session_id, tick FROM public.dynamic_hex_state
        )
        SELECT sp.session_id, sp.tick, si.state_fips,
               si.c_sum, si.v_sum, si.s_sum, si.k_sum, si.biocapacity_sum,
               si.hex_count
        FROM spine AS sp
        JOIN state_intervals AS si
          ON si.session_id = sp.session_id
         AND si.tick <= sp.tick
         AND (si.next_tick IS NULL OR sp.tick < si.next_tick)
    $view$;

    EXECUTE $view$
        CREATE VIEW public.v_national_value_aggregate AS
        WITH change_ticks AS (
            SELECT DISTINCT session_id, tick FROM public.dynamic_hex_state
        ),
        intervals AS (
            SELECT h.session_id, h.cell_id, h.tick,
                   h.c, h.v, h.s, h.k, h.biocapacity_stock,
                   LEAD(h.tick) OVER (
                       PARTITION BY h.session_id, h.cell_id ORDER BY h.tick
                   ) AS next_tick
            FROM public.dynamic_hex_state AS h
        ),
        national_events AS (
            SELECT ct.session_id, ct.tick,
                   SUM(hi.c) AS c_sum,
                   SUM(hi.v) AS v_sum,
                   SUM(hi.s) AS s_sum,
                   SUM(hi.k) AS k_sum,
                   SUM(hi.biocapacity_stock) AS biocapacity_sum,
                   COUNT(*) AS hex_count
            FROM change_ticks AS ct
            JOIN intervals AS hi
              ON hi.session_id = ct.session_id
             AND hi.tick <= ct.tick
             AND (hi.next_tick IS NULL OR ct.tick < hi.next_tick)
            GROUP BY ct.session_id, ct.tick
        ),
        national_intervals AS (
            SELECT ne.*,
                   LEAD(ne.tick) OVER (
                       PARTITION BY ne.session_id ORDER BY ne.tick
                   ) AS next_tick
            FROM national_events AS ne
        ),
        spine AS (
            SELECT session_id, tick FROM public.tick_commit
            UNION
            SELECT DISTINCT session_id, tick FROM public.dynamic_hex_state
        )
        SELECT sp.session_id, sp.tick,
               'USA'::pg_catalog.text AS national_id,
               ni.c_sum, ni.v_sum, ni.s_sum, ni.k_sum, ni.biocapacity_sum,
               ni.hex_count
        FROM spine AS sp
        JOIN national_intervals AS ni
          ON ni.session_id = sp.session_id
         AND ni.tick <= sp.tick
         AND (ni.next_tick IS NULL OR sp.tick < ni.next_tick)
    $view$;

    EXECUTE $view$
        CREATE VIEW public.view_runtime_trace_emission AS
        WITH change_ticks AS (
            SELECT DISTINCT session_id, tick FROM public.dynamic_hex_state
        ),
        intervals AS (
            SELECT h.session_id, h.cell_id, h.tick,
                   COALESCE(m.county_fips, h.county_fips) AS county_fips,
                   h.c, h.v, h.s, h.k,
                   h.biocapacity_stock, h.energy_stock, h.raw_material_stock,
                   h.internet_access_pct, h.surveillance_coupling,
                   LEAD(h.tick) OVER (
                       PARTITION BY h.session_id, h.cell_id ORDER BY h.tick
                   ) AS next_tick
            FROM public.dynamic_hex_state AS h
            LEFT JOIN public.hex_spatial_map AS m
              ON m.cell_id = h.cell_id AND m.session_id = h.session_id
        ),
        county_events AS (
            SELECT ct.session_id, ct.tick, hi.county_fips,
                   SUM(hi.v) AS v,
                   SUM(hi.c) AS c,
                   SUM(hi.s) AS s,
                   SUM(hi.k) AS k,
                   AVG(hi.surveillance_coupling) AS surveillance_coupling,
                   AVG(hi.internet_access_pct) AS internet_access_pct,
                   SUM(hi.biocapacity_stock) AS biocapacity_stock,
                   SUM(hi.energy_stock) AS energy_stock,
                   SUM(hi.raw_material_stock) AS raw_material_stock,
                   CASE WHEN SUM(hi.c) + SUM(hi.v) > 0
                        THEN SUM(hi.s) / (SUM(hi.c) + SUM(hi.v))
                        ELSE NULL
                   END AS profit_rate,
                   CASE WHEN SUM(hi.v) > 0
                        THEN SUM(hi.s) / SUM(hi.v)
                        ELSE NULL
                   END AS exploitation_rate
            FROM change_ticks AS ct
            JOIN intervals AS hi
              ON hi.session_id = ct.session_id
             AND hi.tick <= ct.tick
             AND (hi.next_tick IS NULL OR ct.tick < hi.next_tick)
            GROUP BY ct.session_id, ct.tick, hi.county_fips
        ),
        county_intervals AS (
            SELECT ce.*,
                   LEAD(ce.tick) OVER (
                       PARTITION BY ce.session_id, ce.county_fips ORDER BY ce.tick
                   ) AS next_tick
            FROM county_events AS ce
        ),
        spine AS (
            SELECT session_id, tick FROM public.tick_commit
            UNION
            SELECT DISTINCT session_id, tick FROM public.dynamic_hex_state
        )
        SELECT sp.session_id, sp.tick,
               ci.county_fips AS entity_id,
               'county'::pg_catalog.text AS entity_kind,
               ci.v, ci.c, ci.s, ci.k,
               cs.p_acquiescence, cs.p_revolution,
               cs.ideology_r, cs.ideology_l, cs.ideology_f,
               ci.surveillance_coupling, ci.internet_access_pct,
               ci.biocapacity_stock, ci.energy_stock, ci.raw_material_stock,
               ci.profit_rate, ci.exploitation_rate,
               dem.population, emp.employment_proxy
        FROM spine AS sp
        JOIN county_intervals AS ci
          ON ci.session_id = sp.session_id
         AND ci.tick <= sp.tick
         AND (ci.next_tick IS NULL OR sp.tick < ci.next_tick)
        LEFT JOIN public.dynamic_consciousness_state AS cs
          ON cs.session_id = sp.session_id
         AND cs.tick = sp.tick
         AND cs.county_fips = ci.county_fips
        LEFT JOIN public.dynamic_demographics_state AS dem
          ON dem.session_id = sp.session_id
         AND dem.tick = sp.tick
         AND dem.county_fips = ci.county_fips
        LEFT JOIN public.dynamic_employment_state AS emp
          ON emp.session_id = sp.session_id
         AND emp.tick = sp.tick
         AND emp.county_fips = ci.county_fips
    $view$;

    -- Restore the exact grants and comments carried by the predecessor views.
    EXECUTE 'GRANT SELECT ON public.v_hex_state_asof TO PUBLIC';
    EXECUTE 'GRANT SELECT ON public.view_runtime_trace_emission TO PUBLIC';
    IF EXISTS (
        SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'babylon_intel'
    ) THEN
        EXECUTE
            'GRANT SELECT ON '
            'public.v_hex_economic, public.v_hex_mobilize, '
            'public.v_hex_aid, public.v_hex_heat, public.v_hex_intel, '
            'public.v_hex_state_asof, public.v_county_value_aggregate, '
            'public.v_state_value_aggregate, public.v_national_value_aggregate '
            'TO babylon_intel';
    END IF;
    EXECUTE $comment$
        COMMENT ON VIEW public.view_runtime_trace_emission IS
            'spec-065 trace emission contract; canonical Rust epoch-7 reader '
            'definition (spec-088 FR-003). As-of fill-forward over '
            'delta-persisted hex rows (spec-089 S1c): every committed tick yields '
            'a row per county. 22-column trace_csv_schema.yaml contract unchanged.'
    $comment$;
    EXECUTE $comment$
        COMMENT ON VIEW public.v_hex_state_asof IS
            'spec-089 FR-009: full-resolution hex frame reconstructed at every '
            'committed tick (checkpoint + deltas, fill-forward). The declared '
            'hex-level history read interface; hex res-7 remains the only '
            'persisted source of truth (FR-019).'
    $comment$;
END
$h3_canonical_reader_views$;
