-- One current Rust schema. Installed atomically with current_archive.sql.

CREATE SCHEMA babylon_meta;

CREATE SCHEMA babylon_ref;

CREATE SCHEMA babylon_state;

CREATE FUNCTION babylon_state.verify_tick_choice_receipt_v1_continuity() RETURNS trigger
    LANGUAGE plpgsql
    SET search_path TO 'pg_catalog'
    AS $$
DECLARE
    affected_campaign UUID;
    affected_tick BIGINT;
    affected_receipt BIGINT;
    expected_position BIGINT;
    observed_position BIGINT;
    expected_start NUMERIC(20, 0);
    observed_start NUMERIC(20, 0);
    observed_end NUMERIC(20, 0);
    parent_draw NUMERIC(20, 0);
    parent_selected TEXT;
    selected_matches BIGINT;
BEGIN
    IF TG_OP = 'DELETE' THEN
        affected_campaign := OLD.campaign_id;
        affected_tick := OLD.resolve_tick;
        affected_receipt := OLD.encounter_ordinal;
    ELSE
        affected_campaign := NEW.campaign_id;
        affected_tick := NEW.resolve_tick;
        affected_receipt := NEW.encounter_ordinal;
    END IF;

    expected_position := 0;
    FOR observed_position IN
        SELECT receipt.encounter_ordinal
          FROM babylon_state.tick_choice_receipt_v1 AS receipt
         WHERE receipt.campaign_id = affected_campaign
           AND receipt.resolve_tick = affected_tick
         ORDER BY receipt.encounter_ordinal
    LOOP
        IF observed_position <> expected_position THEN
            RAISE EXCEPTION USING
                ERRCODE = 'P0001',
                MESSAGE = 'tick_choice_receipt_v1_refused_noncontinuous_encounter_order';
        END IF;
        expected_position := expected_position + 1;
    END LOOP;

    SELECT receipt.draw_ticket, receipt.selected_outcome
      INTO parent_draw, parent_selected
      FROM babylon_state.tick_choice_receipt_v1 AS receipt
     WHERE receipt.campaign_id = affected_campaign
       AND receipt.resolve_tick = affected_tick
       AND receipt.encounter_ordinal = affected_receipt;
    IF NOT FOUND THEN
        RETURN NULL;
    END IF;

    expected_position := 0;
    expected_start := 0;
    FOR observed_position, observed_start, observed_end IN
        SELECT branch.position, branch.ticket_start, branch.ticket_end_exclusive
          FROM babylon_state.tick_choice_receipt_branch_v1 AS branch
         WHERE branch.campaign_id = affected_campaign
           AND branch.resolve_tick = affected_tick
           AND branch.encounter_ordinal = affected_receipt
         ORDER BY branch.position
    LOOP
        IF observed_position <> expected_position OR observed_start <> expected_start THEN
            RAISE EXCEPTION USING
                ERRCODE = 'P0001',
                MESSAGE = 'tick_choice_receipt_v1_refused_noncontinuous_branch_order';
        END IF;
        expected_position := expected_position + 1;
        expected_start := observed_end;
    END LOOP;
    IF expected_position = 0 OR expected_start <> 18446744073709551616 THEN
        RAISE EXCEPTION USING
            ERRCODE = 'P0001',
            MESSAGE = 'tick_choice_receipt_v1_refused_incomplete_ticket_measure';
    END IF;

    SELECT pg_catalog.count(*)
      INTO selected_matches
      FROM babylon_state.tick_choice_receipt_branch_v1 AS branch
     WHERE branch.campaign_id = affected_campaign
       AND branch.resolve_tick = affected_tick
       AND branch.encounter_ordinal = affected_receipt
       AND branch.outcome_member = parent_selected
       AND parent_draw >= branch.ticket_start
       AND parent_draw < branch.ticket_end_exclusive;
    IF selected_matches <> 1 THEN
        RAISE EXCEPTION USING
            ERRCODE = 'P0001',
            MESSAGE = 'tick_choice_receipt_v1_refused_selected_outcome_mismatch';
    END IF;

    expected_position := 0;
    FOR observed_position IN
        SELECT carrier.position
          FROM babylon_state.tick_choice_receipt_carrier_element_v1 AS carrier
         WHERE carrier.campaign_id = affected_campaign
           AND carrier.resolve_tick = affected_tick
           AND carrier.encounter_ordinal = affected_receipt
         ORDER BY carrier.position
    LOOP
        IF observed_position <> expected_position THEN
            RAISE EXCEPTION USING
                ERRCODE = 'P0001',
                MESSAGE = 'tick_choice_receipt_v1_refused_noncontinuous_carrier_order';
        END IF;
        expected_position := expected_position + 1;
    END LOOP;
    RETURN NULL;
END
$$;

CREATE FUNCTION babylon_state.verify_tick_event_v2_continuity() RETURNS trigger
    LANGUAGE plpgsql
    SET search_path TO 'pg_catalog'
    AS $$
DECLARE
    affected_campaign UUID;
    affected_tick BIGINT;
    affected_event BIGINT;
    expected_position BIGINT;
    observed_position BIGINT;
BEGIN
    IF TG_OP = 'DELETE' THEN
        affected_campaign := OLD.campaign_id;
        affected_tick := OLD.resolve_tick;
        affected_event := OLD.ordinal;
    ELSE
        affected_campaign := NEW.campaign_id;
        affected_tick := NEW.resolve_tick;
        affected_event := NEW.ordinal;
    END IF;

    expected_position := 0;
    FOR observed_position IN
        SELECT event.ordinal
          FROM babylon_state.tick_event_v2 AS event
         WHERE event.campaign_id = affected_campaign
           AND event.resolve_tick = affected_tick
         ORDER BY event.ordinal
    LOOP
        IF observed_position <> expected_position THEN
            RAISE EXCEPTION USING
                ERRCODE = 'P0001',
                MESSAGE = 'tick_event_v2_refused_noncontinuous_event_order';
        END IF;
        expected_position := expected_position + 1;
    END LOOP;

    IF NOT EXISTS (
        SELECT 1
          FROM babylon_state.tick_event_v2 AS event
         WHERE event.campaign_id = affected_campaign
           AND event.resolve_tick = affected_tick
           AND event.ordinal = affected_event
    ) THEN
        RETURN NULL;
    END IF;

    expected_position := 0;
    FOR observed_position IN
        SELECT field.position
          FROM babylon_state.tick_event_field_v2 AS field
         WHERE field.campaign_id = affected_campaign
           AND field.resolve_tick = affected_tick
           AND field.ordinal = affected_event
         ORDER BY field.position
    LOOP
        IF observed_position <> expected_position THEN
            RAISE EXCEPTION USING
                ERRCODE = 'P0001',
                MESSAGE = 'tick_event_v2_refused_noncontinuous_field_order';
        END IF;
        expected_position := expected_position + 1;
    END LOOP;
    RETURN NULL;
END
$$;

CREATE TABLE babylon_meta.breadcrumb (
    campaign_id uuid NOT NULL,
    "position" integer NOT NULL,
    entity_id text NOT NULL,
    CONSTRAINT breadcrumb_position_check CHECK (("position" >= 0))
);

CREATE TABLE babylon_meta.campaign (
    campaign_id uuid NOT NULL,
    slug text NOT NULL,
    engine_version text NOT NULL,
    defines_hash text NOT NULL,
    last_tick bigint DEFAULT 0 NOT NULL,
    status text DEFAULT 'ACTIVE'::text NOT NULL,
    last_played_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    rng_seed bigint,
    content_digest text,
    CONSTRAINT campaign_last_tick_check CHECK ((last_tick >= 0)),
    CONSTRAINT campaign_status_check CHECK ((status = ANY (ARRAY['ACTIVE'::text, 'ABANDONED'::text])))
);

CREATE TABLE babylon_meta.jumplist (
    campaign_id uuid NOT NULL,
    "position" integer NOT NULL,
    entity_id text NOT NULL,
    CONSTRAINT jumplist_position_check CHECK (("position" >= 0))
);

CREATE TABLE babylon_meta.watchlist (
    campaign_id uuid NOT NULL,
    "position" integer NOT NULL,
    entity_id text NOT NULL,
    CONSTRAINT watchlist_position_check CHECK (("position" >= 0))
);

CREATE TABLE babylon_ref.county_h3_land_area (
    ref_digest bytea NOT NULL,
    product_code text NOT NULL COLLATE pg_catalog."C",
    cell_id bigint NOT NULL,
    membership_origin smallint NOT NULL,
    county_geoid text NOT NULL COLLATE pg_catalog."C",
    land_area_m2 bigint NOT NULL,
    CONSTRAINT county_h3_land_area_cell_positive CHECK ((cell_id > 0)),
    CONSTRAINT county_h3_land_area_direct_origin CHECK ((membership_origin = 1)),
    CONSTRAINT county_h3_land_area_positive CHECK ((land_area_m2 > 0)),
    CONSTRAINT county_h3_land_area_product_code CHECK ((product_code = 'census_county_h3_land_overlap_mi_2023'::text))
);

CREATE TABLE babylon_ref.county_identity (
    ref_digest bytea NOT NULL,
    product_code text NOT NULL COLLATE pg_catalog."C",
    county_id bigint NOT NULL,
    county_geoid text NOT NULL COLLATE pg_catalog."C",
    state_id integer NOT NULL,
    county_fips text NOT NULL COLLATE pg_catalog."C",
    county_name text NOT NULL COLLATE pg_catalog."C",
    CONSTRAINT county_identity_county_id_positive CHECK ((county_id > 0)),
    CONSTRAINT county_identity_fips_format CHECK ((county_fips ~ '^[0-9]{3}$'::text)),
    CONSTRAINT county_identity_geoid_format CHECK ((county_geoid ~ '^[0-9]{5}$'::text)),
    CONSTRAINT county_identity_geoid_suffix CHECK (("right"(county_geoid, 3) = county_fips)),
    CONSTRAINT county_identity_name_nonempty CHECK ((county_name <> ''::text)),
    CONSTRAINT county_identity_product_code CHECK ((product_code = 'dim_county'::text)),
    CONSTRAINT county_identity_state_id_positive CHECK ((state_id > 0))
);

CREATE TABLE babylon_ref.county_place_h3_land_area (
    ref_digest bytea NOT NULL,
    product_code text NOT NULL COLLATE pg_catalog."C",
    cell_id bigint NOT NULL,
    membership_origin smallint NOT NULL,
    county_geoid text NOT NULL COLLATE pg_catalog."C",
    place_geoid text NOT NULL COLLATE pg_catalog."C",
    place_land_area_m2 bigint NOT NULL,
    cell_mi_land_area_m2 bigint NOT NULL,
    place_land_area_share_ppb integer NOT NULL,
    CONSTRAINT county_place_h3_land_area_cell_positive CHECK ((cell_id > 0)),
    CONSTRAINT county_place_h3_land_area_denominator_positive CHECK ((cell_mi_land_area_m2 > 0)),
    CONSTRAINT county_place_h3_land_area_direct_origin CHECK ((membership_origin = 1)),
    CONSTRAINT county_place_h3_land_area_numerator_bound CHECK ((place_land_area_m2 <= cell_mi_land_area_m2)),
    CONSTRAINT county_place_h3_land_area_place_positive CHECK ((place_land_area_m2 > 0)),
    CONSTRAINT county_place_h3_land_area_product_code CHECK ((product_code = 'census_county_place_h3_land_overlap_mi_2023'::text)),
    CONSTRAINT county_place_h3_land_area_share_formula CHECK ((place_land_area_share_ppb = (floor((((place_land_area_m2)::numeric * (1000000000)::numeric) / (cell_mi_land_area_m2)::numeric)))::bigint)),
    CONSTRAINT county_place_h3_land_area_share_range CHECK (((place_land_area_share_ppb >= 0) AND (place_land_area_share_ppb <= 1000000000)))
);

CREATE TABLE babylon_ref.h3_cell (
    cell_id bigint NOT NULL,
    resolution smallint NOT NULL,
    immediate_parent bigint,
    ancestor_r4 bigint,
    ancestor_r5 bigint,
    ancestor_r6 bigint,
    ancestor_r7 bigint,
    CONSTRAINT h3_cell_ancestor_r4_matches CHECK (
CASE
    WHEN ((resolution >= 0) AND (resolution <= 3)) THEN (ancestor_r4 IS NULL)
    WHEN ((resolution >= 4) AND (resolution <= 15)) THEN ((ancestor_r4 IS NOT NULL) AND (ancestor_r4 = (((cell_id & (~ ((15)::bigint << 52))) | ((4)::bigint << 52)) | (((1)::bigint << 33) - 1))))
    ELSE false
END),
    CONSTRAINT h3_cell_ancestor_r5_matches CHECK (
CASE
    WHEN ((resolution >= 0) AND (resolution <= 4)) THEN (ancestor_r5 IS NULL)
    WHEN ((resolution >= 5) AND (resolution <= 15)) THEN ((ancestor_r5 IS NOT NULL) AND (ancestor_r5 = (((cell_id & (~ ((15)::bigint << 52))) | ((5)::bigint << 52)) | (((1)::bigint << 30) - 1))))
    ELSE false
END),
    CONSTRAINT h3_cell_ancestor_r6_matches CHECK (
CASE
    WHEN ((resolution >= 0) AND (resolution <= 5)) THEN (ancestor_r6 IS NULL)
    WHEN ((resolution >= 6) AND (resolution <= 15)) THEN ((ancestor_r6 IS NOT NULL) AND (ancestor_r6 = (((cell_id & (~ ((15)::bigint << 52))) | ((6)::bigint << 52)) | (((1)::bigint << 27) - 1))))
    ELSE false
END),
    CONSTRAINT h3_cell_ancestor_r7_matches CHECK (
CASE
    WHEN ((resolution >= 0) AND (resolution <= 6)) THEN (ancestor_r7 IS NULL)
    WHEN ((resolution >= 7) AND (resolution <= 15)) THEN ((ancestor_r7 IS NOT NULL) AND (ancestor_r7 = (((cell_id & (~ ((15)::bigint << 52))) | ((7)::bigint << 52)) | (((1)::bigint << 24) - 1))))
    ELSE false
END),
    CONSTRAINT h3_cell_id_positive CHECK ((cell_id > 0)),
    CONSTRAINT h3_cell_immediate_parent_matches CHECK (
CASE
    WHEN (resolution = 0) THEN (immediate_parent IS NULL)
    WHEN ((resolution >= 1) AND (resolution <= 15)) THEN ((immediate_parent IS NOT NULL) AND (immediate_parent = (((cell_id & (~ ((15)::bigint << 52))) | (((resolution - 1))::bigint << 52)) | (((1)::bigint << (3 * (16 - resolution))) - 1))))
    ELSE false
END),
    CONSTRAINT h3_cell_resolution_matches_id CHECK ((resolution = (((cell_id >> 52) & (15)::bigint))::smallint)),
    CONSTRAINT h3_cell_resolution_range CHECK (((resolution >= 0) AND (resolution <= 15)))
);

CREATE TABLE babylon_ref.h3_land_fraction (
    ref_digest bytea NOT NULL,
    product_code text NOT NULL COLLATE pg_catalog."C",
    cell_id bigint NOT NULL,
    membership_origin smallint NOT NULL,
    source_county_geoid text NOT NULL COLLATE pg_catalog."C",
    land_fraction_ppm integer NOT NULL,
    CONSTRAINT h3_land_fraction_cell_positive CHECK ((cell_id > 0)),
    CONSTRAINT h3_land_fraction_direct_origin CHECK ((membership_origin = 1)),
    CONSTRAINT h3_land_fraction_product_code CHECK ((product_code = 'h3_res7_land_mask'::text)),
    CONSTRAINT h3_land_fraction_range CHECK (((land_fraction_ppm >= 0) AND (land_fraction_ppm <= 1000000)))
);

CREATE TABLE babylon_ref.h3_population_count (
    ref_digest bytea NOT NULL,
    product_code text NOT NULL COLLATE pg_catalog."C",
    cell_id bigint NOT NULL,
    membership_origin smallint NOT NULL,
    population_count bigint NOT NULL,
    CONSTRAINT h3_population_count_cell_positive CHECK ((cell_id > 0)),
    CONSTRAINT h3_population_count_direct_origin CHECK ((membership_origin = 1)),
    CONSTRAINT h3_population_count_positive CHECK ((population_count > 0)),
    CONSTRAINT h3_population_count_product_code CHECK ((product_code = 'h3_res7_population'::text))
);

CREATE TABLE babylon_ref.h3_reference_cohort (
    ref_digest bytea NOT NULL,
    format_version smallint NOT NULL,
    artifact_name text NOT NULL,
    artifact_manifest_version text NOT NULL,
    artifact_digest bytea NOT NULL,
    source_digest bytea NOT NULL,
    source_r5_digest bytea NOT NULL,
    source_r7_digest bytea NOT NULL,
    closure_digest bytea NOT NULL,
    membership_digest bytea NOT NULL,
    direct_cell_count bigint NOT NULL,
    derived_ancestor_count bigint NOT NULL,
    closure_cell_count bigint NOT NULL,
    CONSTRAINT h3_reference_cohort_artifact_digest_length CHECK ((octet_length(artifact_digest) = 32)),
    CONSTRAINT h3_reference_cohort_artifact_manifest_version_length CHECK (((octet_length(artifact_manifest_version) >= 1) AND (octet_length(artifact_manifest_version) <= 64))),
    CONSTRAINT h3_reference_cohort_artifact_name_length CHECK (((octet_length(artifact_name) >= 1) AND (octet_length(artifact_name) <= 255))),
    CONSTRAINT h3_reference_cohort_closure_count_matches CHECK ((((closure_cell_count >= 1) AND (closure_cell_count <= 1048576)) AND (closure_cell_count = (direct_cell_count + derived_ancestor_count)))),
    CONSTRAINT h3_reference_cohort_closure_digest_length CHECK ((octet_length(closure_digest) = 32)),
    CONSTRAINT h3_reference_cohort_derived_count_nonnegative CHECK (((derived_ancestor_count >= 0) AND (derived_ancestor_count <= 1048576))),
    CONSTRAINT h3_reference_cohort_direct_count_positive CHECK (((direct_cell_count >= 1) AND (direct_cell_count <= 65536))),
    CONSTRAINT h3_reference_cohort_format_v1 CHECK ((format_version = 1)),
    CONSTRAINT h3_reference_cohort_membership_digest_length CHECK ((octet_length(membership_digest) = 32)),
    CONSTRAINT h3_reference_cohort_ref_digest_length CHECK ((octet_length(ref_digest) = 32)),
    CONSTRAINT h3_reference_cohort_source_digest_length CHECK ((octet_length(source_digest) = 32)),
    CONSTRAINT h3_reference_cohort_source_r5_digest_length CHECK ((octet_length(source_r5_digest) = 32)),
    CONSTRAINT h3_reference_cohort_source_r7_digest_length CHECK ((octet_length(source_r7_digest) = 32))
);

CREATE TABLE babylon_ref.h3_reference_membership (
    ref_digest bytea NOT NULL,
    cell_id bigint NOT NULL,
    origin smallint NOT NULL,
    CONSTRAINT h3_reference_membership_cell_positive CHECK ((cell_id > 0)),
    CONSTRAINT h3_reference_membership_origin_closed CHECK ((origin = ANY (ARRAY[1, 2]))),
    CONSTRAINT h3_reference_membership_ref_digest_length CHECK ((octet_length(ref_digest) = 32))
);

CREATE TABLE babylon_ref.h3_workplace_count (
    ref_digest bytea NOT NULL,
    product_code text NOT NULL COLLATE pg_catalog."C",
    cell_id bigint NOT NULL,
    membership_origin smallint NOT NULL,
    workplace_count bigint NOT NULL,
    CONSTRAINT h3_workplace_count_cell_positive CHECK ((cell_id > 0)),
    CONSTRAINT h3_workplace_count_direct_origin CHECK ((membership_origin = 1)),
    CONSTRAINT h3_workplace_count_positive CHECK ((workplace_count > 0)),
    CONSTRAINT h3_workplace_count_product_code CHECK ((product_code = 'h3_res7_workplace'::text))
);

CREATE TABLE babylon_ref.place_identity (
    ref_digest bytea NOT NULL,
    product_code text NOT NULL COLLATE pg_catalog."C",
    place_geoid text NOT NULL COLLATE pg_catalog."C",
    state_fips text NOT NULL COLLATE pg_catalog."C",
    place_fips text NOT NULL COLLATE pg_catalog."C",
    place_ns text NOT NULL COLLATE pg_catalog."C",
    name text NOT NULL COLLATE pg_catalog."C",
    name_lsad text NOT NULL COLLATE pg_catalog."C",
    lsad text NOT NULL COLLATE pg_catalog."C",
    class_fp text NOT NULL COLLATE pg_catalog."C",
    principal_city_indicator text NOT NULL COLLATE pg_catalog."C",
    mtfcc text NOT NULL COLLATE pg_catalog."C",
    functional_status text NOT NULL COLLATE pg_catalog."C",
    CONSTRAINT place_identity_class_format CHECK ((class_fp ~ '^[A-Z0-9]{2}$'::text)),
    CONSTRAINT place_identity_geoid_composition CHECK ((place_geoid = (state_fips || place_fips))),
    CONSTRAINT place_identity_geoid_format CHECK ((place_geoid ~ '^[0-9]{7}$'::text)),
    CONSTRAINT place_identity_lsad_format CHECK ((lsad ~ '^[0-9]{2}$'::text)),
    CONSTRAINT place_identity_mtfcc_format CHECK ((mtfcc ~ '^[A-Z][0-9]{4}$'::text)),
    CONSTRAINT place_identity_name_lsad_nonempty CHECK ((name_lsad <> ''::text)),
    CONSTRAINT place_identity_name_nonempty CHECK ((name <> ''::text)),
    CONSTRAINT place_identity_ns_format CHECK ((place_ns ~ '^[0-9]{8}$'::text)),
    CONSTRAINT place_identity_place_fips_format CHECK ((place_fips ~ '^[0-9]{5}$'::text)),
    CONSTRAINT place_identity_principal_city CHECK ((principal_city_indicator = ANY (ARRAY['N'::text, 'Y'::text]))),
    CONSTRAINT place_identity_product_code CHECK ((product_code = 'census_place_identity_mi_2023'::text)),
    CONSTRAINT place_identity_state CHECK ((state_fips = '26'::text)),
    CONSTRAINT place_identity_status_format CHECK ((functional_status ~ '^[A-Z]$'::text))
);

CREATE TABLE babylon_ref.reference_product (
    ref_digest bytea NOT NULL,
    product_code text NOT NULL COLLATE pg_catalog."C",
    artifact_sha256 bytea NOT NULL,
    semantic_sha256 bytea,
    row_count bigint NOT NULL,
    evidence_class text NOT NULL COLLATE pg_catalog."C",
    measure_unit text COLLATE pg_catalog."C",
    denominator text COLLATE pg_catalog."C",
    CONSTRAINT reference_product_artifact_digest_length CHECK ((octet_length(artifact_sha256) = 32)),
    CONSTRAINT reference_product_code_format CHECK ((product_code ~ '^[a-z0-9_]+$'::text)),
    CONSTRAINT reference_product_denominator CHECK (((denominator IS NULL) OR (denominator = ANY (ARRAY['one_million'::text, 'cell_michigan_land_area_m2'::text])))),
    CONSTRAINT reference_product_evidence_class CHECK ((evidence_class = ANY (ARRAY['Observed'::text, 'Derived'::text]))),
    CONSTRAINT reference_product_measure_unit CHECK (((measure_unit IS NULL) OR (measure_unit = ANY (ARRAY['identity'::text, 'parts_per_million'::text, 'count'::text, 'square_metres'::text])))),
    CONSTRAINT reference_product_ref_digest_length CHECK ((octet_length(ref_digest) = 32)),
    CONSTRAINT reference_product_row_count_positive CHECK ((row_count > 0)),
    CONSTRAINT reference_product_semantic_digest_length CHECK (((semantic_sha256 IS NULL) OR (octet_length(semantic_sha256) = 32)))
);

CREATE TABLE babylon_state.archive_dirty_receipt_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    tick_content_hash bytea NOT NULL,
    CONSTRAINT archive_dirty_receipt_v1_resolve_tick_check CHECK ((resolve_tick >= 1)),
    CONSTRAINT archive_dirty_receipt_v1_tick_content_hash_check CHECK ((octet_length(tick_content_hash) = 32))
);

CREATE TABLE babylon_state.campaign (
    campaign_id uuid NOT NULL,
    replay_layout_version smallint NOT NULL,
    rng_layout_version smallint NOT NULL,
    replay_session_id text NOT NULL COLLATE pg_catalog."C",
    rng_seed bigint NOT NULL,
    defines_hash bytea NOT NULL,
    rules_hash bytea NOT NULL,
    ref_digest bytea NOT NULL,
    CONSTRAINT campaign_defines_hash_length CHECK ((octet_length(defines_hash) = 32)),
    CONSTRAINT campaign_ref_digest_length CHECK ((octet_length(ref_digest) = 32)),
    CONSTRAINT campaign_replay_layout_v1 CHECK ((replay_layout_version = 1)),
    CONSTRAINT campaign_replay_session_ascii_graphic CHECK ((replay_session_id ~ '^[!-~]+$'::text)),
    CONSTRAINT campaign_replay_session_length CHECK (((octet_length(replay_session_id) >= 1) AND (octet_length(replay_session_id) <= 256))),
    CONSTRAINT campaign_rng_layout_v2 CHECK ((rng_layout_version = 2)),
    CONSTRAINT campaign_rules_hash_length CHECK ((octet_length(rules_hash) = 32))
);

CREATE TABLE babylon_state.campaign_foundation (
    campaign_id uuid NOT NULL,
    stable_graph bytea NOT NULL,
    world_registers bytea NOT NULL,
    resolver_manifest bytea NOT NULL,
    prepared_environment bytea NOT NULL,
    replay_session_id text NOT NULL COLLATE pg_catalog."C",
    rng_seed bigint NOT NULL,
    defines_hash bytea NOT NULL,
    rules_hash bytea NOT NULL,
    ref_digest bytea NOT NULL,
    scenario_source text NOT NULL COLLATE pg_catalog."C",
    prelude_source text COLLATE pg_catalog."C",
    rule_source text NOT NULL COLLATE pg_catalog."C",
    defines_bytes bytea NOT NULL,
    reference_manifest_bytes bytea NOT NULL,
    foundation_sha256 bytea NOT NULL,
    CONSTRAINT campaign_foundation_hashes CHECK (((octet_length(defines_hash) = 32) AND (octet_length(rules_hash) = 32) AND (octet_length(ref_digest) = 32) AND (octet_length(foundation_sha256) = 32)))
);

CREATE TABLE babylon_state.checkpoint_manifest (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    completeness_tag smallint NOT NULL,
    manifest_bytes bytea NOT NULL,
    manifest_sha256 bytea NOT NULL,
    CONSTRAINT checkpoint_manifest_completeness_tag_check CHECK ((completeness_tag = ANY (ARRAY[1, 2]))),
    CONSTRAINT checkpoint_manifest_manifest_sha256_check CHECK ((octet_length(manifest_sha256) = 32)),
    CONSTRAINT checkpoint_manifest_resolve_tick_check CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.checkpoint_section_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    section_tag smallint NOT NULL,
    ordinal bigint NOT NULL,
    exact_section_bytes bytea NOT NULL,
    CONSTRAINT checkpoint_section_v1_ordinal_check CHECK (((ordinal >= 0) AND (ordinal <= '4294967295'::bigint))),
    CONSTRAINT checkpoint_section_v1_resolve_tick_check CHECK ((resolve_tick >= 1)),
    CONSTRAINT checkpoint_section_v1_section_tag_check CHECK (((section_tag >= 1) AND (section_tag <= 9)))
);

CREATE TABLE babylon_state.graph_edge_f64_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    edge_type text NOT NULL COLLATE pg_catalog."C",
    source_local_name text NOT NULL COLLATE pg_catalog."C",
    target_local_name text NOT NULL COLLATE pg_catalog."C",
    qname text NOT NULL COLLATE pg_catalog."C",
    value_bits bigint NOT NULL,
    CONSTRAINT graph_edge_f64_v1_resolve_tick_check CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.graph_edge_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    edge_type text NOT NULL COLLATE pg_catalog."C",
    source_local_name text NOT NULL COLLATE pg_catalog."C",
    target_local_name text NOT NULL COLLATE pg_catalog."C",
    strength_bits bigint NOT NULL,
    CONSTRAINT graph_edge_v1_resolve_tick_check CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.graph_hyperedge_f64_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    local_name text NOT NULL COLLATE pg_catalog."C",
    qname text NOT NULL COLLATE pg_catalog."C",
    value_bits bigint NOT NULL,
    CONSTRAINT graph_hyperedge_f64_v1_resolve_tick_check CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.graph_hyperedge_member_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    local_name text NOT NULL COLLATE pg_catalog."C",
    "position" integer NOT NULL,
    member text NOT NULL COLLATE pg_catalog."C",
    CONSTRAINT graph_hyperedge_member_v1_position_check CHECK (("position" >= 0)),
    CONSTRAINT graph_hyperedge_member_v1_resolve_tick_check CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.graph_hyperedge_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    local_name text NOT NULL COLLATE pg_catalog."C",
    hyperedge_type text NOT NULL COLLATE pg_catalog."C",
    CONSTRAINT graph_hyperedge_v1_resolve_tick_check CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.graph_node_currency_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    local_name text NOT NULL COLLATE pg_catalog."C",
    qname text NOT NULL COLLATE pg_catalog."C",
    micro_units numeric(39,0) NOT NULL,
    CONSTRAINT graph_node_currency_v1_resolve_tick_check CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.graph_node_f64_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    local_name text NOT NULL COLLATE pg_catalog."C",
    qname text NOT NULL COLLATE pg_catalog."C",
    value_bits bigint NOT NULL,
    CONSTRAINT graph_node_f64_v1_resolve_tick_check CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.graph_node_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    local_name text NOT NULL COLLATE pg_catalog."C",
    node_type text NOT NULL COLLATE pg_catalog."C",
    CONSTRAINT graph_node_v1_resolve_tick_check CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.hex_state_delta_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    cell_id bigint NOT NULL,
    c_bits bigint NOT NULL,
    v_bits bigint NOT NULL,
    s_bits bigint NOT NULL,
    k_bits bigint NOT NULL,
    biocapacity_stock_bits bigint NOT NULL,
    energy_stock_bits bigint NOT NULL,
    raw_material_stock_bits bigint NOT NULL,
    internet_access_pct_bits bigint NOT NULL,
    surveillance_coupling_bits bigint NOT NULL,
    CONSTRAINT hex_state_delta_v1_cell_id_check CHECK ((cell_id > 0)),
    CONSTRAINT hex_state_delta_v1_resolve_tick_check CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.organization_state_field_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    organization_id bytea NOT NULL,
    "position" integer NOT NULL,
    field_name text NOT NULL COLLATE pg_catalog."C",
    value_tag smallint NOT NULL,
    int_value bigint,
    currency_value numeric(39,0),
    real_bits bigint,
    ratio_bits bigint,
    ratio_min_bits bigint,
    ratio_max_bits bigint,
    bool_value boolean,
    enum_type text COLLATE pg_catalog."C",
    enum_member text COLLATE pg_catalog."C",
    stable_key bytea,
    CONSTRAINT organization_state_field_v1_check CHECK ((((value_tag = 1) AND (int_value IS NOT NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 2) AND (int_value IS NULL) AND (currency_value IS NOT NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 3) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NOT NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 4) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NOT NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 5) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NOT NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 6) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NOT NULL) AND (enum_member IS NOT NULL) AND (stable_key IS NULL)) OR (((value_tag >= 7) AND (value_tag <= 9)) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NOT NULL)))),
    CONSTRAINT organization_state_field_v1_position_check CHECK (("position" >= 0)),
    CONSTRAINT organization_state_field_v1_resolve_tick_check CHECK ((resolve_tick >= 1)),
    CONSTRAINT organization_state_field_v1_value_tag_check CHECK (((value_tag >= 1) AND (value_tag <= 9)))
);

CREATE TABLE babylon_state.organization_state_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    organization_id bytea NOT NULL,
    organization_kind_tag smallint NOT NULL,
    organization_kind_int bigint,
    organization_kind_currency numeric(39,0),
    organization_kind_real_bits bigint,
    organization_kind_ratio_bits bigint,
    organization_kind_ratio_min_bits bigint,
    organization_kind_ratio_max_bits bigint,
    organization_kind_bool boolean,
    organization_kind_enum_type text COLLATE pg_catalog."C",
    organization_kind_enum_member text COLLATE pg_catalog."C",
    organization_kind_stable_key bytea,
    CONSTRAINT organization_state_v1_check CHECK ((((organization_kind_tag = 1) AND (organization_kind_int IS NOT NULL) AND (organization_kind_currency IS NULL) AND (organization_kind_real_bits IS NULL) AND (organization_kind_ratio_bits IS NULL) AND (organization_kind_ratio_min_bits IS NULL) AND (organization_kind_ratio_max_bits IS NULL) AND (organization_kind_bool IS NULL) AND (organization_kind_enum_type IS NULL) AND (organization_kind_enum_member IS NULL) AND (organization_kind_stable_key IS NULL)) OR ((organization_kind_tag = 2) AND (organization_kind_int IS NULL) AND (organization_kind_currency IS NOT NULL) AND (organization_kind_real_bits IS NULL) AND (organization_kind_ratio_bits IS NULL) AND (organization_kind_ratio_min_bits IS NULL) AND (organization_kind_ratio_max_bits IS NULL) AND (organization_kind_bool IS NULL) AND (organization_kind_enum_type IS NULL) AND (organization_kind_enum_member IS NULL) AND (organization_kind_stable_key IS NULL)) OR ((organization_kind_tag = 3) AND (organization_kind_int IS NULL) AND (organization_kind_currency IS NULL) AND (organization_kind_real_bits IS NOT NULL) AND (organization_kind_ratio_bits IS NULL) AND (organization_kind_ratio_min_bits IS NULL) AND (organization_kind_ratio_max_bits IS NULL) AND (organization_kind_bool IS NULL) AND (organization_kind_enum_type IS NULL) AND (organization_kind_enum_member IS NULL) AND (organization_kind_stable_key IS NULL)) OR ((organization_kind_tag = 4) AND (organization_kind_int IS NULL) AND (organization_kind_currency IS NULL) AND (organization_kind_real_bits IS NULL) AND (organization_kind_ratio_bits IS NOT NULL) AND (organization_kind_bool IS NULL) AND (organization_kind_enum_type IS NULL) AND (organization_kind_enum_member IS NULL) AND (organization_kind_stable_key IS NULL)) OR ((organization_kind_tag = 5) AND (organization_kind_int IS NULL) AND (organization_kind_currency IS NULL) AND (organization_kind_real_bits IS NULL) AND (organization_kind_ratio_bits IS NULL) AND (organization_kind_ratio_min_bits IS NULL) AND (organization_kind_ratio_max_bits IS NULL) AND (organization_kind_bool IS NOT NULL) AND (organization_kind_enum_type IS NULL) AND (organization_kind_enum_member IS NULL) AND (organization_kind_stable_key IS NULL)) OR ((organization_kind_tag = 6) AND (organization_kind_int IS NULL) AND (organization_kind_currency IS NULL) AND (organization_kind_real_bits IS NULL) AND (organization_kind_ratio_bits IS NULL) AND (organization_kind_ratio_min_bits IS NULL) AND (organization_kind_ratio_max_bits IS NULL) AND (organization_kind_bool IS NULL) AND (organization_kind_enum_type IS NOT NULL) AND (organization_kind_enum_member IS NOT NULL) AND (organization_kind_stable_key IS NULL)) OR (((organization_kind_tag >= 7) AND (organization_kind_tag <= 9)) AND (organization_kind_int IS NULL) AND (organization_kind_currency IS NULL) AND (organization_kind_real_bits IS NULL) AND (organization_kind_ratio_bits IS NULL) AND (organization_kind_ratio_min_bits IS NULL) AND (organization_kind_ratio_max_bits IS NULL) AND (organization_kind_bool IS NULL) AND (organization_kind_enum_type IS NULL) AND (organization_kind_enum_member IS NULL) AND (organization_kind_stable_key IS NOT NULL)))),
    CONSTRAINT organization_state_v1_organization_kind_tag_check CHECK (((organization_kind_tag >= 1) AND (organization_kind_tag <= 9))),
    CONSTRAINT organization_state_v1_resolve_tick_check CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.organization_territory_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    organization_id bytea NOT NULL,
    "position" integer NOT NULL,
    territory_id bytea NOT NULL,
    CONSTRAINT organization_territory_v1_position_check CHECK (("position" >= 0)),
    CONSTRAINT organization_territory_v1_resolve_tick_check CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.territory_state_field_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    territory_id bytea NOT NULL,
    "position" integer NOT NULL,
    field_name text NOT NULL COLLATE pg_catalog."C",
    value_tag smallint NOT NULL,
    int_value bigint,
    currency_value numeric(39,0),
    real_bits bigint,
    ratio_bits bigint,
    ratio_min_bits bigint,
    ratio_max_bits bigint,
    bool_value boolean,
    enum_type text COLLATE pg_catalog."C",
    enum_member text COLLATE pg_catalog."C",
    stable_key bytea,
    CONSTRAINT territory_state_field_v1_check CHECK ((((value_tag = 1) AND (int_value IS NOT NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 2) AND (int_value IS NULL) AND (currency_value IS NOT NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 3) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NOT NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 4) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NOT NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 5) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NOT NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 6) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NOT NULL) AND (enum_member IS NOT NULL) AND (stable_key IS NULL)) OR (((value_tag >= 7) AND (value_tag <= 9)) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NOT NULL)))),
    CONSTRAINT territory_state_field_v1_position_check CHECK (("position" >= 0)),
    CONSTRAINT territory_state_field_v1_resolve_tick_check CHECK ((resolve_tick >= 1)),
    CONSTRAINT territory_state_field_v1_value_tag_check CHECK (((value_tag >= 1) AND (value_tag <= 9)))
);

CREATE TABLE babylon_state.territory_state_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    territory_id bytea NOT NULL,
    CONSTRAINT territory_state_v1_resolve_tick_check CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.tick_action_batch_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    layout_version smallint NOT NULL,
    action_batch_digest bytea NOT NULL,
    exact_action_batch_bytes bytea NOT NULL,
    CONSTRAINT tick_action_batch_v1_action_batch_digest_check CHECK ((octet_length(action_batch_digest) = 32)),
    CONSTRAINT tick_action_batch_v1_exact_action_batch_bytes_check CHECK (((octet_length(exact_action_batch_bytes) >= 55) AND (octet_length(exact_action_batch_bytes) <= 9302326))),
    CONSTRAINT tick_action_batch_v1_layout_version_check CHECK ((layout_version = 1)),
    CONSTRAINT tick_action_batch_v1_resolve_tick_check CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.tick_choice_receipt_branch_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    encounter_ordinal bigint NOT NULL,
    "position" bigint NOT NULL,
    outcome_member text NOT NULL COLLATE pg_catalog."C",
    mass_nanounits numeric(20,0) NOT NULL,
    ticket_start numeric(20,0) NOT NULL,
    ticket_end_exclusive numeric(20,0) NOT NULL,
    ticket_count numeric(20,0) NOT NULL,
    CONSTRAINT tick_choice_receipt_branch_v1_count CHECK (((ticket_count >= (0)::numeric) AND (ticket_count <= '18446744073709551616'::numeric))),
    CONSTRAINT tick_choice_receipt_branch_v1_end CHECK (((ticket_end_exclusive >= (0)::numeric) AND (ticket_end_exclusive <= '18446744073709551616'::numeric))),
    CONSTRAINT tick_choice_receipt_branch_v1_interval CHECK (((ticket_end_exclusive >= ticket_start) AND (ticket_count = (ticket_end_exclusive - ticket_start)))),
    CONSTRAINT tick_choice_receipt_branch_v1_mass CHECK (((mass_nanounits >= (0)::numeric) AND (mass_nanounits <= '18446744073709551615'::numeric))),
    CONSTRAINT tick_choice_receipt_branch_v1_position CHECK ((("position" >= 0) AND ("position" <= '4294967295'::bigint))),
    CONSTRAINT tick_choice_receipt_branch_v1_positive_mass CHECK ((((mass_nanounits = (0)::numeric) AND (ticket_count = (0)::numeric)) OR ((mass_nanounits > (0)::numeric) AND (ticket_count > (0)::numeric)))),
    CONSTRAINT tick_choice_receipt_branch_v1_start CHECK (((ticket_start >= (0)::numeric) AND (ticket_start <= '18446744073709551616'::numeric)))
);

CREATE TABLE babylon_state.tick_choice_receipt_carrier_element_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    encounter_ordinal bigint NOT NULL,
    "position" bigint NOT NULL,
    stable_element bytea NOT NULL,
    CONSTRAINT tick_choice_receipt_carrier_element_v1_position CHECK ((("position" >= 0) AND ("position" <= '4294967295'::bigint)))
);

CREATE TABLE babylon_state.tick_choice_receipt_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    encounter_ordinal bigint NOT NULL,
    rule_id text NOT NULL COLLATE pg_catalog."C",
    sample text NOT NULL COLLATE pg_catalog."C",
    slot bigint NOT NULL,
    outcome_enum text NOT NULL COLLATE pg_catalog."C",
    stable_carrier bytea NOT NULL,
    draw_ticket numeric(20,0) NOT NULL,
    selected_outcome text NOT NULL COLLATE pg_catalog."C",
    allocation_digest bytea NOT NULL,
    instance_digest bytea NOT NULL,
    CONSTRAINT tick_choice_receipt_v1_digests CHECK (((octet_length(allocation_digest) = 32) AND (octet_length(instance_digest) = 32))),
    CONSTRAINT tick_choice_receipt_v1_draw CHECK (((draw_ticket >= (0)::numeric) AND (draw_ticket <= '18446744073709551615'::numeric))),
    CONSTRAINT tick_choice_receipt_v1_encounter CHECK (((encounter_ordinal >= 0) AND (encounter_ordinal <= '4294967295'::bigint))),
    CONSTRAINT tick_choice_receipt_v1_resolve_tick CHECK ((resolve_tick >= 1)),
    CONSTRAINT tick_choice_receipt_v1_slot CHECK (((slot >= 0) AND (slot <= '4294967295'::bigint)))
);

CREATE TABLE babylon_state.tick_commit (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    envelope_layout_version smallint NOT NULL,
    tick_content_hash bytea NOT NULL,
    envelope_digest bytea NOT NULL,
    CONSTRAINT tick_commit_content_hash_length CHECK ((octet_length(tick_content_hash) = 32)),
    CONSTRAINT tick_commit_envelope_digest_length CHECK ((octet_length(envelope_digest) = 32)),
    CONSTRAINT tick_commit_envelope_layout_v3 CHECK ((envelope_layout_version = 3)),
    CONSTRAINT tick_commit_resolve_tick_sql_range CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.tick_event_field_v2 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    ordinal bigint NOT NULL,
    "position" bigint NOT NULL,
    field_name text NOT NULL COLLATE pg_catalog."C",
    value_tag smallint NOT NULL,
    int_value bigint,
    currency_value numeric(39,0),
    real_bits bigint,
    ratio_bits bigint,
    ratio_min_bits bigint,
    ratio_max_bits bigint,
    bool_value boolean,
    enum_type text COLLATE pg_catalog."C",
    enum_member text COLLATE pg_catalog."C",
    stable_key bytea,
    CONSTRAINT tick_event_field_v2_position CHECK ((("position" >= 0) AND ("position" <= '4294967295'::bigint))),
    CONSTRAINT tick_event_field_v2_tag CHECK (((value_tag >= 1) AND (value_tag <= 9))),
    CONSTRAINT tick_event_field_v2_value CHECK ((((value_tag = 1) AND (int_value IS NOT NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 2) AND (int_value IS NULL) AND (currency_value IS NOT NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 3) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NOT NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 4) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NOT NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 5) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NOT NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 6) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NOT NULL) AND (enum_member IS NOT NULL) AND (stable_key IS NULL)) OR (((value_tag >= 7) AND (value_tag <= 9)) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NOT NULL))))
);

CREATE TABLE babylon_state.tick_event_v2 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    ordinal bigint NOT NULL,
    event_type text NOT NULL COLLATE pg_catalog."C",
    emitting_rule text NOT NULL COLLATE pg_catalog."C",
    choice_receipt_ordinal bigint,
    CONSTRAINT tick_event_v2_choice_ordinal CHECK (((choice_receipt_ordinal IS NULL) OR ((choice_receipt_ordinal >= 0) AND (choice_receipt_ordinal <= '4294967295'::bigint)))),
    CONSTRAINT tick_event_v2_ordinal CHECK (((ordinal >= 0) AND (ordinal <= '4294967295'::bigint))),
    CONSTRAINT tick_event_v2_resolve_tick CHECK ((resolve_tick >= 1))
);

CREATE TABLE babylon_state.world_register_v1 (
    campaign_id uuid NOT NULL,
    resolve_tick bigint NOT NULL,
    register_name text NOT NULL COLLATE pg_catalog."C",
    value_tag smallint NOT NULL,
    int_value bigint,
    currency_value numeric(39,0),
    real_bits bigint,
    ratio_bits bigint,
    ratio_min_bits bigint,
    ratio_max_bits bigint,
    bool_value boolean,
    enum_type text COLLATE pg_catalog."C",
    enum_member text COLLATE pg_catalog."C",
    stable_key bytea,
    CONSTRAINT world_register_v1_check CHECK ((((value_tag = 1) AND (int_value IS NOT NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 2) AND (int_value IS NULL) AND (currency_value IS NOT NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 3) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NOT NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 4) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NOT NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 5) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NOT NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NULL)) OR ((value_tag = 6) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NOT NULL) AND (enum_member IS NOT NULL) AND (stable_key IS NULL)) OR (((value_tag >= 7) AND (value_tag <= 9)) AND (int_value IS NULL) AND (currency_value IS NULL) AND (real_bits IS NULL) AND (ratio_bits IS NULL) AND (ratio_min_bits IS NULL) AND (ratio_max_bits IS NULL) AND (bool_value IS NULL) AND (enum_type IS NULL) AND (enum_member IS NULL) AND (stable_key IS NOT NULL)))),
    CONSTRAINT world_register_v1_resolve_tick_check CHECK ((resolve_tick >= 1)),
    CONSTRAINT world_register_v1_value_tag_check CHECK (((value_tag >= 1) AND (value_tag <= 9)))
);

ALTER TABLE ONLY babylon_meta.breadcrumb
    ADD CONSTRAINT breadcrumb_pkey PRIMARY KEY (campaign_id, "position");

ALTER TABLE ONLY babylon_meta.campaign
    ADD CONSTRAINT campaign_pkey PRIMARY KEY (campaign_id);

ALTER TABLE ONLY babylon_meta.campaign
    ADD CONSTRAINT campaign_slug_key UNIQUE (slug);

ALTER TABLE ONLY babylon_meta.jumplist
    ADD CONSTRAINT jumplist_pkey PRIMARY KEY (campaign_id, "position");

ALTER TABLE ONLY babylon_meta.watchlist
    ADD CONSTRAINT watchlist_campaign_id_entity_id_key UNIQUE (campaign_id, entity_id);

ALTER TABLE ONLY babylon_meta.watchlist
    ADD CONSTRAINT watchlist_pkey PRIMARY KEY (campaign_id, "position");

ALTER TABLE ONLY babylon_ref.county_h3_land_area
    ADD CONSTRAINT county_h3_land_area_pkey PRIMARY KEY (ref_digest, cell_id, county_geoid);

ALTER TABLE ONLY babylon_ref.county_identity
    ADD CONSTRAINT county_identity_county_id_key UNIQUE (ref_digest, county_id);

ALTER TABLE ONLY babylon_ref.county_identity
    ADD CONSTRAINT county_identity_pkey PRIMARY KEY (ref_digest, county_geoid);

ALTER TABLE ONLY babylon_ref.county_place_h3_land_area
    ADD CONSTRAINT county_place_h3_land_area_pkey PRIMARY KEY (ref_digest, cell_id, county_geoid, place_geoid);

ALTER TABLE ONLY babylon_ref.h3_cell
    ADD CONSTRAINT h3_cell_pkey PRIMARY KEY (cell_id);

ALTER TABLE ONLY babylon_ref.h3_land_fraction
    ADD CONSTRAINT h3_land_fraction_pkey PRIMARY KEY (ref_digest, cell_id);

ALTER TABLE ONLY babylon_ref.h3_population_count
    ADD CONSTRAINT h3_population_count_pkey PRIMARY KEY (ref_digest, cell_id);

ALTER TABLE ONLY babylon_ref.h3_reference_cohort
    ADD CONSTRAINT h3_reference_cohort_artifact_identity UNIQUE (format_version, artifact_digest);

ALTER TABLE ONLY babylon_ref.h3_reference_cohort
    ADD CONSTRAINT h3_reference_cohort_pkey PRIMARY KEY (ref_digest);

ALTER TABLE ONLY babylon_ref.h3_reference_membership
    ADD CONSTRAINT h3_reference_membership_pkey PRIMARY KEY (ref_digest, cell_id);

ALTER TABLE ONLY babylon_ref.h3_workplace_count
    ADD CONSTRAINT h3_workplace_count_pkey PRIMARY KEY (ref_digest, cell_id);

ALTER TABLE ONLY babylon_ref.place_identity
    ADD CONSTRAINT place_identity_pkey PRIMARY KEY (ref_digest, place_geoid);

ALTER TABLE ONLY babylon_ref.reference_product
    ADD CONSTRAINT reference_product_pkey PRIMARY KEY (ref_digest, product_code);

ALTER TABLE ONLY babylon_state.archive_dirty_receipt_v1
    ADD CONSTRAINT archive_dirty_receipt_v1_pkey PRIMARY KEY (campaign_id, resolve_tick);

ALTER TABLE ONLY babylon_state.campaign_foundation
    ADD CONSTRAINT campaign_foundation_pkey PRIMARY KEY (campaign_id);

ALTER TABLE ONLY babylon_state.campaign
    ADD CONSTRAINT campaign_pkey PRIMARY KEY (campaign_id);

ALTER TABLE ONLY babylon_state.checkpoint_manifest
    ADD CONSTRAINT checkpoint_manifest_pkey PRIMARY KEY (campaign_id, resolve_tick);

ALTER TABLE ONLY babylon_state.checkpoint_section_v1
    ADD CONSTRAINT checkpoint_section_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, section_tag, ordinal);

ALTER TABLE ONLY babylon_state.graph_edge_f64_v1
    ADD CONSTRAINT graph_edge_f64_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, edge_type, source_local_name, target_local_name, qname);

ALTER TABLE ONLY babylon_state.graph_edge_v1
    ADD CONSTRAINT graph_edge_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, edge_type, source_local_name, target_local_name);

ALTER TABLE ONLY babylon_state.graph_hyperedge_f64_v1
    ADD CONSTRAINT graph_hyperedge_f64_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, local_name, qname);

ALTER TABLE ONLY babylon_state.graph_hyperedge_member_v1
    ADD CONSTRAINT graph_hyperedge_member_v1_campaign_id_resolve_tick_local_na_key UNIQUE (campaign_id, resolve_tick, local_name, member);

ALTER TABLE ONLY babylon_state.graph_hyperedge_member_v1
    ADD CONSTRAINT graph_hyperedge_member_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, local_name, "position");

ALTER TABLE ONLY babylon_state.graph_hyperedge_v1
    ADD CONSTRAINT graph_hyperedge_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, local_name);

ALTER TABLE ONLY babylon_state.graph_node_currency_v1
    ADD CONSTRAINT graph_node_currency_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, local_name, qname);

ALTER TABLE ONLY babylon_state.graph_node_f64_v1
    ADD CONSTRAINT graph_node_f64_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, local_name, qname);

ALTER TABLE ONLY babylon_state.graph_node_v1
    ADD CONSTRAINT graph_node_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, local_name);

ALTER TABLE ONLY babylon_state.hex_state_delta_v1
    ADD CONSTRAINT hex_state_delta_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, cell_id);

ALTER TABLE ONLY babylon_state.organization_state_field_v1
    ADD CONSTRAINT organization_state_field_v1_campaign_id_resolve_tick_organi_key UNIQUE (campaign_id, resolve_tick, organization_id, field_name);

ALTER TABLE ONLY babylon_state.organization_state_field_v1
    ADD CONSTRAINT organization_state_field_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, organization_id, "position");

ALTER TABLE ONLY babylon_state.organization_state_v1
    ADD CONSTRAINT organization_state_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, organization_id);

ALTER TABLE ONLY babylon_state.organization_territory_v1
    ADD CONSTRAINT organization_territory_v1_campaign_id_resolve_tick_organiza_key UNIQUE (campaign_id, resolve_tick, organization_id, territory_id);

ALTER TABLE ONLY babylon_state.organization_territory_v1
    ADD CONSTRAINT organization_territory_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, organization_id, "position");

ALTER TABLE ONLY babylon_state.territory_state_field_v1
    ADD CONSTRAINT territory_state_field_v1_campaign_id_resolve_tick_territory_key UNIQUE (campaign_id, resolve_tick, territory_id, field_name);

ALTER TABLE ONLY babylon_state.territory_state_field_v1
    ADD CONSTRAINT territory_state_field_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, territory_id, "position");

ALTER TABLE ONLY babylon_state.territory_state_v1
    ADD CONSTRAINT territory_state_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, territory_id);

ALTER TABLE ONLY babylon_state.tick_action_batch_v1
    ADD CONSTRAINT tick_action_batch_v1_pkey PRIMARY KEY (campaign_id, resolve_tick);

ALTER TABLE ONLY babylon_state.tick_choice_receipt_branch_v1
    ADD CONSTRAINT tick_choice_receipt_branch_v1_member_key UNIQUE (campaign_id, resolve_tick, encounter_ordinal, outcome_member);

ALTER TABLE ONLY babylon_state.tick_choice_receipt_branch_v1
    ADD CONSTRAINT tick_choice_receipt_branch_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, encounter_ordinal, "position");

ALTER TABLE ONLY babylon_state.tick_choice_receipt_carrier_element_v1
    ADD CONSTRAINT tick_choice_receipt_carrier_element_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, encounter_ordinal, "position");

ALTER TABLE ONLY babylon_state.tick_choice_receipt_v1
    ADD CONSTRAINT tick_choice_receipt_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, encounter_ordinal);

ALTER TABLE ONLY babylon_state.tick_commit
    ADD CONSTRAINT tick_commit_pkey PRIMARY KEY (campaign_id, resolve_tick);

ALTER TABLE ONLY babylon_state.tick_event_field_v2
    ADD CONSTRAINT tick_event_field_v2_pkey PRIMARY KEY (campaign_id, resolve_tick, ordinal, "position");

ALTER TABLE ONLY babylon_state.tick_event_v2
    ADD CONSTRAINT tick_event_v2_pkey PRIMARY KEY (campaign_id, resolve_tick, ordinal);

ALTER TABLE ONLY babylon_state.world_register_v1
    ADD CONSTRAINT world_register_v1_pkey PRIMARY KEY (campaign_id, resolve_tick, register_name);

CREATE INDEX county_h3_land_area_county_idx ON babylon_ref.county_h3_land_area USING btree (ref_digest, county_geoid, cell_id);

CREATE INDEX county_place_h3_land_area_place_idx ON babylon_ref.county_place_h3_land_area USING btree (ref_digest, place_geoid, cell_id, county_geoid);

CREATE INDEX h3_reference_membership_cell_id_idx ON babylon_ref.h3_reference_membership USING btree (cell_id, ref_digest);

CREATE UNIQUE INDEX h3_reference_membership_origin_key ON babylon_ref.h3_reference_membership USING btree (ref_digest, cell_id, origin);

CREATE CONSTRAINT TRIGGER tick_choice_receipt_branch_v1_continuity AFTER INSERT OR DELETE OR UPDATE ON babylon_state.tick_choice_receipt_branch_v1 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION babylon_state.verify_tick_choice_receipt_v1_continuity();

CREATE CONSTRAINT TRIGGER tick_choice_receipt_carrier_element_v1_continuity AFTER INSERT OR DELETE OR UPDATE ON babylon_state.tick_choice_receipt_carrier_element_v1 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION babylon_state.verify_tick_choice_receipt_v1_continuity();

CREATE CONSTRAINT TRIGGER tick_choice_receipt_v1_continuity AFTER INSERT OR DELETE OR UPDATE ON babylon_state.tick_choice_receipt_v1 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION babylon_state.verify_tick_choice_receipt_v1_continuity();

CREATE CONSTRAINT TRIGGER tick_event_field_v2_continuity AFTER INSERT OR DELETE OR UPDATE ON babylon_state.tick_event_field_v2 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION babylon_state.verify_tick_event_v2_continuity();

CREATE CONSTRAINT TRIGGER tick_event_v2_continuity AFTER INSERT OR DELETE OR UPDATE ON babylon_state.tick_event_v2 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION babylon_state.verify_tick_event_v2_continuity();

ALTER TABLE ONLY babylon_meta.breadcrumb
    ADD CONSTRAINT breadcrumb_campaign_id_fkey FOREIGN KEY (campaign_id) REFERENCES babylon_meta.campaign(campaign_id) ON DELETE CASCADE;

ALTER TABLE ONLY babylon_meta.jumplist
    ADD CONSTRAINT jumplist_campaign_id_fkey FOREIGN KEY (campaign_id) REFERENCES babylon_meta.campaign(campaign_id) ON DELETE CASCADE;

ALTER TABLE ONLY babylon_meta.watchlist
    ADD CONSTRAINT watchlist_campaign_id_fkey FOREIGN KEY (campaign_id) REFERENCES babylon_meta.campaign(campaign_id) ON DELETE CASCADE;

ALTER TABLE ONLY babylon_ref.county_h3_land_area
    ADD CONSTRAINT county_h3_land_area_county_fkey FOREIGN KEY (ref_digest, county_geoid) REFERENCES babylon_ref.county_identity(ref_digest, county_geoid) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.county_h3_land_area
    ADD CONSTRAINT county_h3_land_area_membership_fkey FOREIGN KEY (ref_digest, cell_id, membership_origin) REFERENCES babylon_ref.h3_reference_membership(ref_digest, cell_id, origin) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.county_h3_land_area
    ADD CONSTRAINT county_h3_land_area_product_fkey FOREIGN KEY (ref_digest, product_code) REFERENCES babylon_ref.reference_product(ref_digest, product_code) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.county_identity
    ADD CONSTRAINT county_identity_product_fkey FOREIGN KEY (ref_digest, product_code) REFERENCES babylon_ref.reference_product(ref_digest, product_code) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.county_place_h3_land_area
    ADD CONSTRAINT county_place_h3_land_area_county_cell_fkey FOREIGN KEY (ref_digest, cell_id, county_geoid) REFERENCES babylon_ref.county_h3_land_area(ref_digest, cell_id, county_geoid) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.county_place_h3_land_area
    ADD CONSTRAINT county_place_h3_land_area_membership_fkey FOREIGN KEY (ref_digest, cell_id, membership_origin) REFERENCES babylon_ref.h3_reference_membership(ref_digest, cell_id, origin) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.county_place_h3_land_area
    ADD CONSTRAINT county_place_h3_land_area_place_fkey FOREIGN KEY (ref_digest, place_geoid) REFERENCES babylon_ref.place_identity(ref_digest, place_geoid) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.county_place_h3_land_area
    ADD CONSTRAINT county_place_h3_land_area_product_fkey FOREIGN KEY (ref_digest, product_code) REFERENCES babylon_ref.reference_product(ref_digest, product_code) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.h3_cell
    ADD CONSTRAINT h3_cell_ancestor_r4_fkey FOREIGN KEY (ancestor_r4) REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.h3_cell
    ADD CONSTRAINT h3_cell_ancestor_r5_fkey FOREIGN KEY (ancestor_r5) REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.h3_cell
    ADD CONSTRAINT h3_cell_ancestor_r6_fkey FOREIGN KEY (ancestor_r6) REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.h3_cell
    ADD CONSTRAINT h3_cell_ancestor_r7_fkey FOREIGN KEY (ancestor_r7) REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.h3_cell
    ADD CONSTRAINT h3_cell_immediate_parent_fkey FOREIGN KEY (immediate_parent) REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.h3_land_fraction
    ADD CONSTRAINT h3_land_fraction_county_fkey FOREIGN KEY (ref_digest, source_county_geoid) REFERENCES babylon_ref.county_identity(ref_digest, county_geoid) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.h3_land_fraction
    ADD CONSTRAINT h3_land_fraction_membership_fkey FOREIGN KEY (ref_digest, cell_id, membership_origin) REFERENCES babylon_ref.h3_reference_membership(ref_digest, cell_id, origin) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.h3_land_fraction
    ADD CONSTRAINT h3_land_fraction_product_fkey FOREIGN KEY (ref_digest, product_code) REFERENCES babylon_ref.reference_product(ref_digest, product_code) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.h3_population_count
    ADD CONSTRAINT h3_population_count_membership_fkey FOREIGN KEY (ref_digest, cell_id, membership_origin) REFERENCES babylon_ref.h3_reference_membership(ref_digest, cell_id, origin) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.h3_population_count
    ADD CONSTRAINT h3_population_count_product_fkey FOREIGN KEY (ref_digest, product_code) REFERENCES babylon_ref.reference_product(ref_digest, product_code) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.h3_reference_membership
    ADD CONSTRAINT h3_reference_membership_cell_fkey FOREIGN KEY (cell_id) REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.h3_reference_membership
    ADD CONSTRAINT h3_reference_membership_cohort_fkey FOREIGN KEY (ref_digest) REFERENCES babylon_ref.h3_reference_cohort(ref_digest) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.h3_workplace_count
    ADD CONSTRAINT h3_workplace_count_membership_fkey FOREIGN KEY (ref_digest, cell_id, membership_origin) REFERENCES babylon_ref.h3_reference_membership(ref_digest, cell_id, origin) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.h3_workplace_count
    ADD CONSTRAINT h3_workplace_count_product_fkey FOREIGN KEY (ref_digest, product_code) REFERENCES babylon_ref.reference_product(ref_digest, product_code) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.place_identity
    ADD CONSTRAINT place_identity_product_fkey FOREIGN KEY (ref_digest, product_code) REFERENCES babylon_ref.reference_product(ref_digest, product_code) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_ref.reference_product
    ADD CONSTRAINT reference_product_cohort_fkey FOREIGN KEY (ref_digest) REFERENCES babylon_ref.h3_reference_cohort(ref_digest) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_state.campaign_foundation
    ADD CONSTRAINT campaign_foundation_campaign_fkey FOREIGN KEY (campaign_id) REFERENCES babylon_state.campaign(campaign_id);

ALTER TABLE ONLY babylon_state.campaign
    ADD CONSTRAINT campaign_reference_cohort_fkey FOREIGN KEY (ref_digest) REFERENCES babylon_ref.h3_reference_cohort(ref_digest) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_state.graph_hyperedge_member_v1
    ADD CONSTRAINT graph_hyperedge_member_v1_campaign_id_resolve_tick_local_n_fkey FOREIGN KEY (campaign_id, resolve_tick, local_name) REFERENCES babylon_state.graph_hyperedge_v1(campaign_id, resolve_tick, local_name) ON DELETE CASCADE;

ALTER TABLE ONLY babylon_state.organization_state_field_v1
    ADD CONSTRAINT organization_state_field_v1_campaign_id_resolve_tick_organ_fkey FOREIGN KEY (campaign_id, resolve_tick, organization_id) REFERENCES babylon_state.organization_state_v1(campaign_id, resolve_tick, organization_id) ON DELETE CASCADE;

ALTER TABLE ONLY babylon_state.organization_territory_v1
    ADD CONSTRAINT organization_territory_v1_campaign_id_resolve_tick_organiz_fkey FOREIGN KEY (campaign_id, resolve_tick, organization_id) REFERENCES babylon_state.organization_state_v1(campaign_id, resolve_tick, organization_id) ON DELETE CASCADE;

ALTER TABLE ONLY babylon_state.territory_state_field_v1
    ADD CONSTRAINT territory_state_field_v1_campaign_id_resolve_tick_territor_fkey FOREIGN KEY (campaign_id, resolve_tick, territory_id) REFERENCES babylon_state.territory_state_v1(campaign_id, resolve_tick, territory_id) ON DELETE CASCADE;

ALTER TABLE ONLY babylon_state.tick_action_batch_v1
    ADD CONSTRAINT tick_action_batch_v1_campaign_id_resolve_tick_fkey FOREIGN KEY (campaign_id, resolve_tick) REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_state.tick_choice_receipt_branch_v1
    ADD CONSTRAINT tick_choice_receipt_branch_v1_parent_fkey FOREIGN KEY (campaign_id, resolve_tick, encounter_ordinal) REFERENCES babylon_state.tick_choice_receipt_v1(campaign_id, resolve_tick, encounter_ordinal) ON DELETE CASCADE;

ALTER TABLE ONLY babylon_state.tick_choice_receipt_carrier_element_v1
    ADD CONSTRAINT tick_choice_receipt_carrier_element_v1_parent_fkey FOREIGN KEY (campaign_id, resolve_tick, encounter_ordinal) REFERENCES babylon_state.tick_choice_receipt_v1(campaign_id, resolve_tick, encounter_ordinal) ON DELETE CASCADE;

ALTER TABLE ONLY babylon_state.tick_choice_receipt_v1
    ADD CONSTRAINT tick_choice_receipt_v1_tick_fkey FOREIGN KEY (campaign_id, resolve_tick) REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_state.tick_commit
    ADD CONSTRAINT tick_commit_campaign_fkey FOREIGN KEY (campaign_id) REFERENCES babylon_state.campaign(campaign_id) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE ONLY babylon_state.tick_event_field_v2
    ADD CONSTRAINT tick_event_field_v2_parent_fkey FOREIGN KEY (campaign_id, resolve_tick, ordinal) REFERENCES babylon_state.tick_event_v2(campaign_id, resolve_tick, ordinal) ON DELETE CASCADE;

ALTER TABLE ONLY babylon_state.tick_event_v2
    ADD CONSTRAINT tick_event_v2_choice_receipt_fkey FOREIGN KEY (campaign_id, resolve_tick, choice_receipt_ordinal) REFERENCES babylon_state.tick_choice_receipt_v1(campaign_id, resolve_tick, encounter_ordinal);

ALTER TABLE ONLY babylon_state.tick_event_v2
    ADD CONSTRAINT tick_event_v2_tick_fkey FOREIGN KEY (campaign_id, resolve_tick) REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick) DEFERRABLE INITIALLY DEFERRED;

REVOKE ALL ON FUNCTION babylon_state.verify_tick_choice_receipt_v1_continuity() FROM PUBLIC;

REVOKE ALL ON FUNCTION babylon_state.verify_tick_event_v2_continuity() FROM PUBLIC;

CREATE TABLE babylon_state.material_campaign_foundation_v2 (
    campaign_id uuid PRIMARY KEY REFERENCES babylon_state.campaign(campaign_id),
    preset_id text NOT NULL CHECK (octet_length(preset_id) BETWEEN 1 AND 128),
    horizon_ticks bigint NOT NULL CHECK (horizon_ticks > 0),
    content_sha256 bytea NOT NULL CHECK (octet_length(content_sha256) = 32),
    initial_register_bytes bytea NOT NULL CHECK (octet_length(initial_register_bytes) <= 67108864),
    foundation_bytes bytea NOT NULL CHECK (octet_length(foundation_bytes) <= 67108864),
    foundation_sha256 bytea NOT NULL CHECK (octet_length(foundation_sha256) = 32)
);
CREATE TABLE babylon_state.material_tick_v3 (
    campaign_id uuid NOT NULL REFERENCES babylon_state.material_campaign_foundation_v2(campaign_id),
    resolve_tick bigint NOT NULL CHECK (resolve_tick > 0),
    identity_bytes bytea NOT NULL CHECK (octet_length(identity_bytes) <= 1024),
    register_bytes bytea NOT NULL CHECK (octet_length(register_bytes) <= 67108864),
    receipt_bytes bytea NOT NULL CHECK (octet_length(receipt_bytes) <= 67108864),
    PRIMARY KEY (campaign_id, resolve_tick)
);
CREATE FUNCTION babylon_state.require_material_tick_v3() RETURNS trigger
LANGUAGE plpgsql SET search_path = pg_catalog AS $$
BEGIN
    IF NEW.envelope_layout_version <> 3 THEN
        RAISE EXCEPTION 'material campaign version mismatch';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM babylon_state.material_tick_v3 t WHERE t.campaign_id = NEW.campaign_id AND t.resolve_tick = NEW.resolve_tick) THEN
        RAISE EXCEPTION 'material tick missing before commit marker';
    END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER material_tick_marker_v3 BEFORE INSERT ON babylon_state.tick_commit FOR EACH ROW EXECUTE FUNCTION babylon_state.require_material_tick_v3();
REVOKE ALL ON FUNCTION babylon_state.require_material_tick_v3() FROM PUBLIC;
REVOKE ALL ON babylon_state.material_campaign_foundation_v2, babylon_state.material_tick_v3 FROM PUBLIC;

CREATE TABLE babylon_meta.territory_county_map_v1 (
    campaign_id UUID NOT NULL,
    territory_local_name TEXT COLLATE pg_catalog."C" NOT NULL CHECK (
        pg_catalog.octet_length(territory_local_name) BETWEEN 1 AND 256
    ),
    county_geoid TEXT COLLATE pg_catalog."C" NOT NULL CHECK (county_geoid ~ '^[0-9]{5}$'),
    PRIMARY KEY (campaign_id, territory_local_name),
    FOREIGN KEY (campaign_id) REFERENCES babylon_meta.campaign(campaign_id) ON DELETE CASCADE
);
REVOKE ALL ON TABLE babylon_meta.territory_county_map_v1 FROM PUBLIC;

CREATE TABLE babylon_meta.current_schema (
    singleton BOOLEAN PRIMARY KEY CHECK (singleton),
    schema_sha256 BYTEA NOT NULL CHECK (octet_length(schema_sha256) = 32)
);
REVOKE ALL ON babylon_meta.current_schema FROM PUBLIC;
