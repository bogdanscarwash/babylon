CREATE TABLE babylon_state.campaign (
    campaign_id UUID NOT NULL,
    replay_layout_version SMALLINT NOT NULL,
    rng_layout_version SMALLINT NOT NULL,
    replay_session_id TEXT COLLATE pg_catalog."C" NOT NULL,
    rng_seed BIGINT NOT NULL,
    defines_hash BYTEA NOT NULL,
    rules_hash BYTEA NOT NULL,
    ref_digest BYTEA NOT NULL,
    CONSTRAINT campaign_pkey PRIMARY KEY (campaign_id),
    CONSTRAINT campaign_replay_layout_v1 CHECK (replay_layout_version = 1),
    CONSTRAINT campaign_rng_layout_v2 CHECK (rng_layout_version = 2),
    CONSTRAINT campaign_replay_session_length CHECK (
        pg_catalog.octet_length(replay_session_id) BETWEEN 1 AND 256
    ),
    CONSTRAINT campaign_replay_session_ascii_graphic CHECK (
        replay_session_id OPERATOR(pg_catalog.~) '^[!-~]+$'
    ),
    CONSTRAINT campaign_defines_hash_length CHECK (
        pg_catalog.octet_length(defines_hash) = 32
    ),
    CONSTRAINT campaign_rules_hash_length CHECK (
        pg_catalog.octet_length(rules_hash) = 32
    ),
    CONSTRAINT campaign_ref_digest_length CHECK (
        pg_catalog.octet_length(ref_digest) = 32
    ),
    CONSTRAINT campaign_reference_cohort_fkey FOREIGN KEY (ref_digest)
        REFERENCES babylon_ref.h3_reference_cohort(ref_digest)
        DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE babylon_state.tick_commit (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL,
    envelope_layout_version SMALLINT NOT NULL,
    tick_content_hash BYTEA NOT NULL,
    envelope_digest BYTEA NOT NULL,
    CONSTRAINT tick_commit_pkey PRIMARY KEY (campaign_id, resolve_tick),
    CONSTRAINT tick_commit_resolve_tick_sql_range CHECK (resolve_tick >= 0),
    CONSTRAINT tick_commit_envelope_layout_v1 CHECK (envelope_layout_version = 1),
    CONSTRAINT tick_commit_content_hash_length CHECK (
        pg_catalog.octet_length(tick_content_hash) = 32
    ),
    CONSTRAINT tick_commit_envelope_digest_length CHECK (
        pg_catalog.octet_length(envelope_digest) = 32
    ),
    CONSTRAINT tick_commit_campaign_fkey FOREIGN KEY (campaign_id)
        REFERENCES babylon_state.campaign(campaign_id)
        DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE babylon_state.tick_graph_row (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL,
    row_ordinal INTEGER NOT NULL,
    row_key BYTEA NOT NULL,
    row_payload BYTEA NOT NULL,
    CONSTRAINT tick_graph_row_pkey PRIMARY KEY (campaign_id, resolve_tick, row_ordinal),
    CONSTRAINT tick_graph_row_resolve_tick_sql_range CHECK (resolve_tick >= 0),
    CONSTRAINT tick_graph_row_ordinal_range CHECK (
        row_ordinal BETWEEN 0 AND 1048575
    ),
    CONSTRAINT tick_graph_row_key_length CHECK (
        pg_catalog.octet_length(row_key) BETWEEN 1 AND 67108856
    ),
    CONSTRAINT tick_graph_row_payload_length CHECK (
        pg_catalog.octet_length(row_payload) BETWEEN 0 AND 67108855
    ),
    CONSTRAINT tick_graph_row_campaign_tick_fkey FOREIGN KEY (campaign_id, resolve_tick)
        REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
        DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE babylon_state.tick_state_row (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL,
    row_ordinal INTEGER NOT NULL,
    row_key BYTEA NOT NULL,
    row_payload BYTEA NOT NULL,
    CONSTRAINT tick_state_row_pkey PRIMARY KEY (campaign_id, resolve_tick, row_ordinal),
    CONSTRAINT tick_state_row_resolve_tick_sql_range CHECK (resolve_tick >= 0),
    CONSTRAINT tick_state_row_ordinal_range CHECK (
        row_ordinal BETWEEN 0 AND 1048575
    ),
    CONSTRAINT tick_state_row_key_length CHECK (
        pg_catalog.octet_length(row_key) BETWEEN 1 AND 67108856
    ),
    CONSTRAINT tick_state_row_payload_length CHECK (
        pg_catalog.octet_length(row_payload) BETWEEN 0 AND 67108855
    ),
    CONSTRAINT tick_state_row_campaign_tick_fkey FOREIGN KEY (campaign_id, resolve_tick)
        REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
        DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE babylon_state.tick_event_row (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL,
    row_ordinal INTEGER NOT NULL,
    row_key BYTEA NOT NULL,
    row_payload BYTEA NOT NULL,
    CONSTRAINT tick_event_row_pkey PRIMARY KEY (campaign_id, resolve_tick, row_ordinal),
    CONSTRAINT tick_event_row_resolve_tick_sql_range CHECK (resolve_tick >= 0),
    CONSTRAINT tick_event_row_ordinal_range CHECK (
        row_ordinal BETWEEN 0 AND 1048575
    ),
    CONSTRAINT tick_event_row_key_length CHECK (
        pg_catalog.octet_length(row_key) BETWEEN 1 AND 67108856
    ),
    CONSTRAINT tick_event_row_payload_length CHECK (
        pg_catalog.octet_length(row_payload) BETWEEN 0 AND 67108855
    ),
    CONSTRAINT tick_event_row_campaign_tick_fkey FOREIGN KEY (campaign_id, resolve_tick)
        REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
        DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE babylon_state.tick_subsystem_row (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL,
    row_ordinal INTEGER NOT NULL,
    row_key BYTEA NOT NULL,
    row_payload BYTEA NOT NULL,
    CONSTRAINT tick_subsystem_row_pkey PRIMARY KEY (campaign_id, resolve_tick, row_ordinal),
    CONSTRAINT tick_subsystem_row_resolve_tick_sql_range CHECK (resolve_tick >= 0),
    CONSTRAINT tick_subsystem_row_ordinal_range CHECK (
        row_ordinal BETWEEN 0 AND 1048575
    ),
    CONSTRAINT tick_subsystem_row_key_length CHECK (
        pg_catalog.octet_length(row_key) BETWEEN 1 AND 67108856
    ),
    CONSTRAINT tick_subsystem_row_payload_length CHECK (
        pg_catalog.octet_length(row_payload) BETWEEN 0 AND 67108855
    ),
    CONSTRAINT tick_subsystem_row_campaign_tick_fkey FOREIGN KEY (campaign_id, resolve_tick)
        REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
        DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE babylon_state.tick_conservation_row (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL,
    row_ordinal INTEGER NOT NULL,
    row_key BYTEA NOT NULL,
    row_payload BYTEA NOT NULL,
    CONSTRAINT tick_conservation_row_pkey PRIMARY KEY (campaign_id, resolve_tick, row_ordinal),
    CONSTRAINT tick_conservation_row_resolve_tick_sql_range CHECK (resolve_tick >= 0),
    CONSTRAINT tick_conservation_row_ordinal_range CHECK (
        row_ordinal BETWEEN 0 AND 1048575
    ),
    CONSTRAINT tick_conservation_row_key_length CHECK (
        pg_catalog.octet_length(row_key) BETWEEN 1 AND 67108856
    ),
    CONSTRAINT tick_conservation_row_payload_length CHECK (
        pg_catalog.octet_length(row_payload) BETWEEN 0 AND 67108855
    ),
    CONSTRAINT tick_conservation_row_campaign_tick_fkey FOREIGN KEY (campaign_id, resolve_tick)
        REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
        DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE babylon_state.tick_boundary_flow_row (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL,
    row_ordinal INTEGER NOT NULL,
    row_key BYTEA NOT NULL,
    row_payload BYTEA NOT NULL,
    CONSTRAINT tick_boundary_flow_row_pkey PRIMARY KEY (campaign_id, resolve_tick, row_ordinal),
    CONSTRAINT tick_boundary_flow_row_resolve_tick_sql_range CHECK (resolve_tick >= 0),
    CONSTRAINT tick_boundary_flow_row_ordinal_range CHECK (
        row_ordinal BETWEEN 0 AND 1048575
    ),
    CONSTRAINT tick_boundary_flow_row_key_length CHECK (
        pg_catalog.octet_length(row_key) BETWEEN 1 AND 67108856
    ),
    CONSTRAINT tick_boundary_flow_row_payload_length CHECK (
        pg_catalog.octet_length(row_payload) BETWEEN 0 AND 67108855
    ),
    CONSTRAINT tick_boundary_flow_row_campaign_tick_fkey FOREIGN KEY (campaign_id, resolve_tick)
        REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
        DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE babylon_state.tick_checkpoint_row (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL,
    row_ordinal INTEGER NOT NULL,
    row_key BYTEA NOT NULL,
    row_payload BYTEA NOT NULL,
    CONSTRAINT tick_checkpoint_row_pkey PRIMARY KEY (campaign_id, resolve_tick, row_ordinal),
    CONSTRAINT tick_checkpoint_row_resolve_tick_sql_range CHECK (resolve_tick >= 0),
    CONSTRAINT tick_checkpoint_row_ordinal_range CHECK (
        row_ordinal BETWEEN 0 AND 1048575
    ),
    CONSTRAINT tick_checkpoint_row_key_length CHECK (
        pg_catalog.octet_length(row_key) BETWEEN 1 AND 67108856
    ),
    CONSTRAINT tick_checkpoint_row_payload_length CHECK (
        pg_catalog.octet_length(row_payload) BETWEEN 0 AND 67108855
    ),
    CONSTRAINT tick_checkpoint_row_campaign_tick_fkey FOREIGN KEY (campaign_id, resolve_tick)
        REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
        DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE babylon_state.tick_archive_dirty_receipt_row (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL,
    row_ordinal INTEGER NOT NULL,
    row_key BYTEA NOT NULL,
    row_payload BYTEA NOT NULL,
    CONSTRAINT tick_archive_dirty_receipt_row_pkey PRIMARY KEY (
        campaign_id, resolve_tick, row_ordinal
    ),
    CONSTRAINT tick_archive_dirty_receipt_row_resolve_tick_sql_range CHECK (
        resolve_tick >= 0
    ),
    CONSTRAINT tick_archive_dirty_receipt_row_ordinal_range CHECK (
        row_ordinal BETWEEN 0 AND 1048575
    ),
    CONSTRAINT tick_archive_dirty_receipt_row_key_length CHECK (
        pg_catalog.octet_length(row_key) BETWEEN 1 AND 67108856
    ),
    CONSTRAINT tick_archive_dirty_receipt_row_payload_length CHECK (
        pg_catalog.octet_length(row_payload) BETWEEN 0 AND 67108855
    ),
    CONSTRAINT tick_archive_dirty_receipt_row_campaign_tick_fkey FOREIGN KEY (
        campaign_id, resolve_tick
    ) REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
        DEFERRABLE INITIALLY DEFERRED
);
REVOKE ALL ON TABLE babylon_state.campaign FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.tick_commit FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.tick_graph_row FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.tick_state_row FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.tick_event_row FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.tick_subsystem_row FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.tick_conservation_row FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.tick_boundary_flow_row FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.tick_checkpoint_row FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.tick_archive_dirty_receipt_row FROM PUBLIC;
