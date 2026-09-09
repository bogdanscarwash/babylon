CREATE VIEW public.v_observer_economy_foundation_v1 AS
SELECT campaign_id, foundation_sha256,
       pg_catalog.sha256(pg_catalog.convert_to(scenario_source, 'UTF8')) AS scenario_sha256
FROM babylon_state.campaign_foundation;

CREATE VIEW public.v_observer_county_economy_v1 AS
SELECT foundation.campaign_id, 0::bigint AS resolve_tick, mapping.county_geoid,
       NULL::bigint AS annual_avg_estabs_count, NULL::bigint AS annual_avg_emplvl,
       NULL::bigint AS total_annual_wages, NULL::bigint AS annual_avg_wkly_wage,
       true AS establishments_granted, true AS employment_granted,
       true AS annual_wages_granted, true AS weekly_wage_granted
FROM babylon_state.campaign_foundation AS foundation
JOIN babylon_meta.territory_county_map_v1 AS mapping USING (campaign_id)
UNION ALL
SELECT fields.campaign_id, fields.resolve_tick,
       pg_catalog.lpad(identity.int_value::text, 5, '0') AS county_geoid,
       max(fields.int_value) FILTER (WHERE fields.field_name = 'qcew-establishments'),
       max(fields.int_value) FILTER (WHERE fields.field_name = 'qcew-employment'),
       max(fields.int_value) FILTER (WHERE fields.field_name = 'qcew-total-annual-wages'),
       max(fields.int_value) FILTER (WHERE fields.field_name = 'qcew-average-weekly-wage'),
       true, true, true, true
FROM babylon_state.territory_state_field_v1 AS fields
JOIN babylon_state.territory_state_field_v1 AS identity
  ON identity.campaign_id = fields.campaign_id AND identity.resolve_tick = fields.resolve_tick
 AND identity.territory_id = fields.territory_id
 AND identity.field_name = 'county-fips' AND identity.value_tag = 1
JOIN babylon_state.tick_commit AS marker
  ON marker.campaign_id = fields.campaign_id AND marker.resolve_tick = fields.resolve_tick
WHERE fields.value_tag = 1 AND fields.field_name IN
  ('qcew-establishments', 'qcew-employment', 'qcew-total-annual-wages', 'qcew-average-weekly-wage')
GROUP BY fields.campaign_id, fields.resolve_tick, identity.int_value;

CREATE VIEW public.v_known_county_economy_v1 WITH (security_barrier = true) AS
SELECT raw.campaign_id, raw.resolve_tick, raw.county_geoid,
       CASE WHEN permission.establishments THEN raw.annual_avg_estabs_count END AS annual_avg_estabs_count,
       CASE WHEN permission.employment THEN raw.annual_avg_emplvl END AS annual_avg_emplvl,
       CASE WHEN permission.annual_wages THEN raw.total_annual_wages END AS total_annual_wages,
       CASE WHEN permission.weekly_wage THEN raw.annual_avg_wkly_wage END AS annual_avg_wkly_wage,
       permission.establishments AS establishments_granted,
       permission.employment AS employment_granted,
       permission.annual_wages AS annual_wages_granted,
       permission.weekly_wage AS weekly_wage_granted
FROM public.v_observer_county_economy_v1 AS raw
CROSS JOIN LATERAL (
  SELECT
    EXISTS (SELECT 1 FROM babylon_meta.archive_knowledge_grant_v1 AS grant_row
      WHERE grant_row.campaign_id = raw.campaign_id AND grant_row.subject_kind = 'county'
        AND grant_row.subject_id = raw.county_geoid AND grant_row.granted_tick <= raw.resolve_tick
        AND grant_row.grant_key = 'qcew-establishments') AS establishments,
    EXISTS (SELECT 1 FROM babylon_meta.archive_knowledge_grant_v1 AS grant_row
      WHERE grant_row.campaign_id = raw.campaign_id AND grant_row.subject_kind = 'county'
        AND grant_row.subject_id = raw.county_geoid AND grant_row.granted_tick <= raw.resolve_tick
        AND grant_row.grant_key = 'qcew-employment') AS employment,
    EXISTS (SELECT 1 FROM babylon_meta.archive_knowledge_grant_v1 AS grant_row
      WHERE grant_row.campaign_id = raw.campaign_id AND grant_row.subject_kind = 'county'
        AND grant_row.subject_id = raw.county_geoid AND grant_row.granted_tick <= raw.resolve_tick
        AND grant_row.grant_key = 'qcew-total-annual-wages') AS annual_wages,
    EXISTS (SELECT 1 FROM babylon_meta.archive_knowledge_grant_v1 AS grant_row
      WHERE grant_row.campaign_id = raw.campaign_id AND grant_row.subject_kind = 'county'
        AND grant_row.subject_id = raw.county_geoid AND grant_row.granted_tick <= raw.resolve_tick
        AND grant_row.grant_key = 'qcew-average-weekly-wage') AS weekly_wage
) AS permission;

GRANT SELECT ON public.v_observer_economy_foundation_v1 TO babylon_observer, babylon_reader;
GRANT SELECT ON public.v_observer_county_economy_v1 TO babylon_observer;
GRANT SELECT ON public.v_committed_tick_status_v1 TO babylon_observer;
GRANT SELECT ON public.v_known_county_economy_v1 TO babylon_reader;
