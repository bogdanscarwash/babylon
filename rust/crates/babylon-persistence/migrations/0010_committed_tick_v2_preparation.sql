-- Additive preparation for the V2-only committed-tick authority boundary.
-- The caller holds the schema advisory lock, executes this file in one
-- serializable transaction, and inserts the prepared authority row last.

DO $committed_tick_v2_predecessor_preflight$
DECLARE
    exact_rows BIGINT;
BEGIN
    IF pg_catalog.to_regclass('babylon_meta.persistence_authority_ledger') IS NULL THEN
        RAISE EXCEPTION USING
            ERRCODE = 'P0001',
            MESSAGE = 'committed_tick_v2_preparation_refused_missing_v1_authority';
    END IF;

    SELECT pg_catalog.count(*)
      INTO exact_rows
      FROM babylon_meta.persistence_authority_ledger AS authority
     WHERE
        (authority.ordinal = 1
         AND authority.state_tag = 1
         AND authority.schema_epoch = 8
         AND authority.predecessor_sha256 IS NULL)
        OR
        (authority.ordinal = 2
         AND authority.state_tag = 2
         AND authority.schema_epoch = 9
         AND authority.predecessor_sha256 = (
             SELECT prepared.row_sha256
               FROM babylon_meta.persistence_authority_ledger AS prepared
              WHERE prepared.ordinal = 1
         ));

    IF exact_rows <> 2 OR (
        SELECT pg_catalog.count(*)
          FROM babylon_meta.persistence_authority_ledger
    ) <> 2 THEN
        RAISE EXCEPTION USING
            ERRCODE = 'P0001',
            MESSAGE = 'committed_tick_v2_preparation_refused_noncanonical_v1_authority';
    END IF;
END
$committed_tick_v2_predecessor_preflight$;

CREATE TABLE babylon_meta.committed_tick_v2_authority_ledger (
    ordinal SMALLINT NOT NULL,
    state_tag SMALLINT NOT NULL,
    activation_epoch SMALLINT NOT NULL,
    contract_sha256 BYTEA NOT NULL,
    reader_contract_sha256 BYTEA NOT NULL,
    predecessor_sha256 BYTEA NOT NULL,
    row_sha256 BYTEA NOT NULL,
    CONSTRAINT committed_tick_v2_authority_ledger_pkey PRIMARY KEY (ordinal),
    CONSTRAINT committed_tick_v2_authority_ledger_state_key UNIQUE (state_tag),
    CONSTRAINT committed_tick_v2_authority_ledger_row_digest_key UNIQUE (row_sha256),
    CONSTRAINT committed_tick_v2_authority_ledger_ordinal CHECK (ordinal IN (1, 2)),
    CONSTRAINT committed_tick_v2_authority_ledger_state CHECK (state_tag IN (1, 2)),
    CONSTRAINT committed_tick_v2_authority_ledger_epoch CHECK (activation_epoch IN (10, 11)),
    CONSTRAINT committed_tick_v2_authority_ledger_digest_widths CHECK (
        pg_catalog.octet_length(contract_sha256) = 32
        AND pg_catalog.octet_length(reader_contract_sha256) = 32
        AND pg_catalog.octet_length(predecessor_sha256) = 32
        AND pg_catalog.octet_length(row_sha256) = 32
    ),
    CONSTRAINT committed_tick_v2_authority_ledger_closed_rows CHECK (
        (ordinal = 1 AND state_tag = 1 AND activation_epoch = 10)
        OR
        (ordinal = 2 AND state_tag = 2 AND activation_epoch = 11)
    )
);

CREATE TABLE babylon_meta.committed_tick_v2_incompatible_inventory (
    relation_name TEXT COLLATE pg_catalog."C" NOT NULL,
    observed_row_count BIGINT NOT NULL,
    disposition_tag SMALLINT NOT NULL,
    CONSTRAINT committed_tick_v2_incompatible_inventory_pkey PRIMARY KEY (relation_name),
    CONSTRAINT committed_tick_v2_incompatible_inventory_name CHECK (
        relation_name OPERATOR(pg_catalog.~) '^babylon_state\.[a-z0-9_]+$'
    ),
    CONSTRAINT committed_tick_v2_incompatible_inventory_count CHECK (observed_row_count >= 0),
    CONSTRAINT committed_tick_v2_incompatible_inventory_disposition CHECK (
        (observed_row_count = 0 AND disposition_tag = 1)
        OR
        (observed_row_count > 0 AND disposition_tag = 2)
    )
);

WITH observed(relation_name, observed_row_count) AS (
    SELECT 'babylon_state.campaign', pg_catalog.count(*) FROM babylon_state.campaign
    UNION ALL
    SELECT 'babylon_state.campaign_foundation', pg_catalog.count(*) FROM babylon_state.campaign_foundation
    UNION ALL
    SELECT 'babylon_state.tick_commit', pg_catalog.count(*) FROM babylon_state.tick_commit
    UNION ALL
    SELECT 'babylon_state.tick_action_batch_v1', pg_catalog.count(*) FROM babylon_state.tick_action_batch_v1
    UNION ALL
    SELECT 'babylon_state.graph_node_v1', pg_catalog.count(*) FROM babylon_state.graph_node_v1
    UNION ALL
    SELECT 'babylon_state.graph_node_f64_v1', pg_catalog.count(*) FROM babylon_state.graph_node_f64_v1
    UNION ALL
    SELECT 'babylon_state.graph_edge_v1', pg_catalog.count(*) FROM babylon_state.graph_edge_v1
    UNION ALL
    SELECT 'babylon_state.graph_hyperedge_v1', pg_catalog.count(*) FROM babylon_state.graph_hyperedge_v1
    UNION ALL
    SELECT 'babylon_state.graph_hyperedge_member_v1', pg_catalog.count(*) FROM babylon_state.graph_hyperedge_member_v1
    UNION ALL
    SELECT 'babylon_state.graph_edge_f64_v1', pg_catalog.count(*) FROM babylon_state.graph_edge_f64_v1
    UNION ALL
    SELECT 'babylon_state.graph_node_currency_v1', pg_catalog.count(*) FROM babylon_state.graph_node_currency_v1
    UNION ALL
    SELECT 'babylon_state.graph_hyperedge_f64_v1', pg_catalog.count(*) FROM babylon_state.graph_hyperedge_f64_v1
    UNION ALL
    SELECT 'babylon_state.world_register_v1', pg_catalog.count(*) FROM babylon_state.world_register_v1
    UNION ALL
    SELECT 'babylon_state.territory_state_v1', pg_catalog.count(*) FROM babylon_state.territory_state_v1
    UNION ALL
    SELECT 'babylon_state.territory_state_field_v1', pg_catalog.count(*) FROM babylon_state.territory_state_field_v1
    UNION ALL
    SELECT 'babylon_state.hex_state_delta_v1', pg_catalog.count(*) FROM babylon_state.hex_state_delta_v1
    UNION ALL
    SELECT 'babylon_state.organization_state_v1', pg_catalog.count(*) FROM babylon_state.organization_state_v1
    UNION ALL
    SELECT 'babylon_state.organization_territory_v1', pg_catalog.count(*) FROM babylon_state.organization_territory_v1
    UNION ALL
    SELECT 'babylon_state.organization_state_field_v1', pg_catalog.count(*) FROM babylon_state.organization_state_field_v1
    UNION ALL
    SELECT 'babylon_state.tick_event_v1', pg_catalog.count(*) FROM babylon_state.tick_event_v1
    UNION ALL
    SELECT 'babylon_state.tick_event_field_v1', pg_catalog.count(*) FROM babylon_state.tick_event_field_v1
    UNION ALL
    SELECT 'babylon_state.checkpoint_manifest', pg_catalog.count(*) FROM babylon_state.checkpoint_manifest
    UNION ALL
    SELECT 'babylon_state.checkpoint_section_v1', pg_catalog.count(*) FROM babylon_state.checkpoint_section_v1
    UNION ALL
    SELECT 'babylon_state.archive_dirty_receipt_v1', pg_catalog.count(*) FROM babylon_state.archive_dirty_receipt_v1
)
INSERT INTO babylon_meta.committed_tick_v2_incompatible_inventory (
    relation_name,
    observed_row_count,
    disposition_tag
)
SELECT
    observed.relation_name,
    observed.observed_row_count,
    CASE WHEN observed.observed_row_count = 0 THEN 1 ELSE 2 END
FROM observed
ORDER BY observed.relation_name;

CREATE TABLE babylon_state.tick_choice_receipt_v1 (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL,
    encounter_ordinal BIGINT NOT NULL,
    rule_id TEXT COLLATE pg_catalog."C" NOT NULL,
    sample TEXT COLLATE pg_catalog."C" NOT NULL,
    slot BIGINT NOT NULL,
    outcome_enum TEXT COLLATE pg_catalog."C" NOT NULL,
    stable_carrier BYTEA NOT NULL,
    draw_ticket NUMERIC(20, 0) NOT NULL,
    selected_outcome TEXT COLLATE pg_catalog."C" NOT NULL,
    allocation_digest BYTEA NOT NULL,
    instance_digest BYTEA NOT NULL,
    CONSTRAINT tick_choice_receipt_v1_pkey PRIMARY KEY (
        campaign_id, resolve_tick, encounter_ordinal
    ),
    CONSTRAINT tick_choice_receipt_v1_resolve_tick CHECK (resolve_tick >= 1),
    CONSTRAINT tick_choice_receipt_v1_encounter CHECK (
        encounter_ordinal BETWEEN 0 AND 4294967295
    ),
    CONSTRAINT tick_choice_receipt_v1_slot CHECK (slot BETWEEN 0 AND 4294967295),
    CONSTRAINT tick_choice_receipt_v1_draw CHECK (
        draw_ticket BETWEEN 0 AND 18446744073709551615
    ),
    CONSTRAINT tick_choice_receipt_v1_digests CHECK (
        pg_catalog.octet_length(allocation_digest) = 32
        AND pg_catalog.octet_length(instance_digest) = 32
    ),
    CONSTRAINT tick_choice_receipt_v1_tick_fkey FOREIGN KEY (campaign_id, resolve_tick)
        REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
        DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE babylon_state.tick_choice_receipt_branch_v1 (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL,
    encounter_ordinal BIGINT NOT NULL,
    position BIGINT NOT NULL,
    outcome_member TEXT COLLATE pg_catalog."C" NOT NULL,
    mass_nanounits NUMERIC(20, 0) NOT NULL,
    ticket_start NUMERIC(20, 0) NOT NULL,
    ticket_end_exclusive NUMERIC(20, 0) NOT NULL,
    ticket_count NUMERIC(20, 0) NOT NULL,
    CONSTRAINT tick_choice_receipt_branch_v1_pkey PRIMARY KEY (
        campaign_id, resolve_tick, encounter_ordinal, position
    ),
    CONSTRAINT tick_choice_receipt_branch_v1_member_key UNIQUE (
        campaign_id, resolve_tick, encounter_ordinal, outcome_member
    ),
    CONSTRAINT tick_choice_receipt_branch_v1_position CHECK (
        position BETWEEN 0 AND 4294967295
    ),
    CONSTRAINT tick_choice_receipt_branch_v1_mass CHECK (
        mass_nanounits BETWEEN 0 AND 18446744073709551615
    ),
    CONSTRAINT tick_choice_receipt_branch_v1_start CHECK (
        ticket_start BETWEEN 0 AND 18446744073709551616
    ),
    CONSTRAINT tick_choice_receipt_branch_v1_end CHECK (
        ticket_end_exclusive BETWEEN 0 AND 18446744073709551616
    ),
    CONSTRAINT tick_choice_receipt_branch_v1_count CHECK (
        ticket_count BETWEEN 0 AND 18446744073709551616
    ),
    CONSTRAINT tick_choice_receipt_branch_v1_interval CHECK (
        ticket_end_exclusive >= ticket_start
        AND ticket_count = ticket_end_exclusive - ticket_start
    ),
    CONSTRAINT tick_choice_receipt_branch_v1_positive_mass CHECK (
        (mass_nanounits = 0 AND ticket_count = 0)
        OR
        (mass_nanounits > 0 AND ticket_count > 0)
    ),
    CONSTRAINT tick_choice_receipt_branch_v1_parent_fkey FOREIGN KEY (
        campaign_id, resolve_tick, encounter_ordinal
    ) REFERENCES babylon_state.tick_choice_receipt_v1(
        campaign_id, resolve_tick, encounter_ordinal
    ) ON DELETE CASCADE
);

CREATE TABLE babylon_state.tick_choice_receipt_carrier_element_v1 (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL,
    encounter_ordinal BIGINT NOT NULL,
    position BIGINT NOT NULL,
    stable_element BYTEA NOT NULL,
    CONSTRAINT tick_choice_receipt_carrier_element_v1_pkey PRIMARY KEY (
        campaign_id, resolve_tick, encounter_ordinal, position
    ),
    CONSTRAINT tick_choice_receipt_carrier_element_v1_position CHECK (
        position BETWEEN 0 AND 4294967295
    ),
    CONSTRAINT tick_choice_receipt_carrier_element_v1_parent_fkey FOREIGN KEY (
        campaign_id, resolve_tick, encounter_ordinal
    ) REFERENCES babylon_state.tick_choice_receipt_v1(
        campaign_id, resolve_tick, encounter_ordinal
    ) ON DELETE CASCADE
);

CREATE FUNCTION babylon_state.verify_tick_choice_receipt_v1_continuity()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path TO pg_catalog
AS $verify_tick_choice_receipt_v1_continuity$
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
$verify_tick_choice_receipt_v1_continuity$;

CREATE CONSTRAINT TRIGGER tick_choice_receipt_v1_continuity
AFTER INSERT OR UPDATE OR DELETE ON babylon_state.tick_choice_receipt_v1
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION babylon_state.verify_tick_choice_receipt_v1_continuity();

CREATE CONSTRAINT TRIGGER tick_choice_receipt_branch_v1_continuity
AFTER INSERT OR UPDATE OR DELETE ON babylon_state.tick_choice_receipt_branch_v1
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION babylon_state.verify_tick_choice_receipt_v1_continuity();

CREATE CONSTRAINT TRIGGER tick_choice_receipt_carrier_element_v1_continuity
AFTER INSERT OR UPDATE OR DELETE ON babylon_state.tick_choice_receipt_carrier_element_v1
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION babylon_state.verify_tick_choice_receipt_v1_continuity();

CREATE TABLE babylon_state.tick_event_v2 (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL,
    ordinal BIGINT NOT NULL,
    event_type TEXT COLLATE pg_catalog."C" NOT NULL,
    emitting_rule TEXT COLLATE pg_catalog."C" NOT NULL,
    choice_receipt_ordinal BIGINT,
    CONSTRAINT tick_event_v2_pkey PRIMARY KEY (campaign_id, resolve_tick, ordinal),
    CONSTRAINT tick_event_v2_resolve_tick CHECK (resolve_tick >= 1),
    CONSTRAINT tick_event_v2_ordinal CHECK (ordinal BETWEEN 0 AND 4294967295),
    CONSTRAINT tick_event_v2_choice_ordinal CHECK (
        choice_receipt_ordinal IS NULL
        OR choice_receipt_ordinal BETWEEN 0 AND 4294967295
    ),
    CONSTRAINT tick_event_v2_tick_fkey FOREIGN KEY (campaign_id, resolve_tick)
        REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT tick_event_v2_choice_receipt_fkey FOREIGN KEY (
        campaign_id, resolve_tick, choice_receipt_ordinal
    ) REFERENCES babylon_state.tick_choice_receipt_v1(
        campaign_id, resolve_tick, encounter_ordinal
    )
);

CREATE TABLE babylon_state.tick_event_field_v2 (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL,
    ordinal BIGINT NOT NULL,
    position BIGINT NOT NULL,
    field_name TEXT COLLATE pg_catalog."C" NOT NULL,
    value_tag SMALLINT NOT NULL,
    int_value BIGINT,
    currency_value NUMERIC(39, 0),
    real_bits BIGINT,
    ratio_bits BIGINT,
    ratio_min_bits BIGINT,
    ratio_max_bits BIGINT,
    bool_value BOOLEAN,
    enum_type TEXT COLLATE pg_catalog."C",
    enum_member TEXT COLLATE pg_catalog."C",
    stable_key BYTEA,
    CONSTRAINT tick_event_field_v2_pkey PRIMARY KEY (
        campaign_id, resolve_tick, ordinal, position
    ),
    CONSTRAINT tick_event_field_v2_position CHECK (
        position BETWEEN 0 AND 4294967295
    ),
    CONSTRAINT tick_event_field_v2_tag CHECK (value_tag BETWEEN 1 AND 9),
    CONSTRAINT tick_event_field_v2_value CHECK (
        (value_tag = 1 AND int_value IS NOT NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR
        (value_tag = 2 AND int_value IS NULL AND currency_value IS NOT NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR
        (value_tag = 3 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NOT NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR
        (value_tag = 4 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NOT NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR
        (value_tag = 5 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NOT NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR
        (value_tag = 6 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NOT NULL AND enum_member IS NOT NULL AND stable_key IS NULL)
        OR
        (value_tag BETWEEN 7 AND 9 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NOT NULL)
    ),
    CONSTRAINT tick_event_field_v2_parent_fkey FOREIGN KEY (
        campaign_id, resolve_tick, ordinal
    ) REFERENCES babylon_state.tick_event_v2(campaign_id, resolve_tick, ordinal)
      ON DELETE CASCADE
);

CREATE FUNCTION babylon_state.verify_tick_event_v2_continuity()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path TO pg_catalog
AS $verify_tick_event_v2_continuity$
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
$verify_tick_event_v2_continuity$;

CREATE CONSTRAINT TRIGGER tick_event_v2_continuity
AFTER INSERT OR UPDATE OR DELETE ON babylon_state.tick_event_v2
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION babylon_state.verify_tick_event_v2_continuity();

CREATE CONSTRAINT TRIGGER tick_event_field_v2_continuity
AFTER INSERT OR UPDATE OR DELETE ON babylon_state.tick_event_field_v2
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION babylon_state.verify_tick_event_v2_continuity();

REVOKE ALL ON TABLE babylon_meta.committed_tick_v2_authority_ledger FROM PUBLIC;
REVOKE ALL ON TABLE babylon_meta.committed_tick_v2_incompatible_inventory FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.tick_choice_receipt_v1 FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.tick_choice_receipt_branch_v1 FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.tick_choice_receipt_carrier_element_v1 FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.tick_event_v2 FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.tick_event_field_v2 FROM PUBLIC;
REVOKE ALL ON FUNCTION babylon_state.verify_tick_choice_receipt_v1_continuity() FROM PUBLIC;
REVOKE ALL ON FUNCTION babylon_state.verify_tick_event_v2_continuity() FROM PUBLIC;
