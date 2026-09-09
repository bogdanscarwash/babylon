CREATE TABLE babylon_meta.persistence_authority_ledger (
    ordinal SMALLINT NOT NULL,
    state_tag SMALLINT NOT NULL,
    schema_epoch SMALLINT NOT NULL,
    contract_sha256 BYTEA NOT NULL,
    reader_contract_sha256 BYTEA NOT NULL,
    predecessor_sha256 BYTEA,
    row_sha256 BYTEA NOT NULL,
    CONSTRAINT persistence_authority_ledger_pkey PRIMARY KEY (ordinal),
    CONSTRAINT persistence_authority_ledger_ordinal CHECK (ordinal IN (1, 2)),
    CONSTRAINT persistence_authority_ledger_state CHECK (state_tag IN (1, 2)),
    CONSTRAINT persistence_authority_ledger_epoch CHECK (schema_epoch IN (8, 9)),
    CONSTRAINT persistence_authority_ledger_contract_digest CHECK (
        pg_catalog.octet_length(contract_sha256) = 32
    ),
    CONSTRAINT persistence_authority_ledger_reader_digest CHECK (
        pg_catalog.octet_length(reader_contract_sha256) = 32
    ),
    CONSTRAINT persistence_authority_ledger_predecessor_digest CHECK (
        predecessor_sha256 IS NULL OR pg_catalog.octet_length(predecessor_sha256) = 32
    ),
    CONSTRAINT persistence_authority_ledger_row_digest CHECK (
        pg_catalog.octet_length(row_sha256) = 32
    ),
    CONSTRAINT persistence_authority_ledger_closed_rows CHECK (
        (ordinal = 1 AND state_tag = 1 AND schema_epoch = 8 AND predecessor_sha256 IS NULL)
        OR
        (ordinal = 2 AND state_tag = 2 AND schema_epoch = 9 AND predecessor_sha256 IS NOT NULL)
    )
);

CREATE TABLE babylon_meta.python_relation_disposition_v1 (
    relation_name TEXT COLLATE pg_catalog."C" NOT NULL,
    observed_row_count BIGINT NOT NULL,
    ordered_semantic_sha256 BYTEA NOT NULL,
    disposition_tag SMALLINT NOT NULL,
    CONSTRAINT python_relation_disposition_v1_pkey PRIMARY KEY (relation_name),
    CONSTRAINT python_relation_disposition_v1_name CHECK (
        relation_name OPERATOR(pg_catalog.~) '^(public|babylon_meta)\.[a-z0-9_]+$'
    ),
    CONSTRAINT python_relation_disposition_v1_zero_rows CHECK (observed_row_count = 0),
    CONSTRAINT python_relation_disposition_v1_digest CHECK (
        pg_catalog.octet_length(ordered_semantic_sha256) = 32
    ),
    CONSTRAINT python_relation_disposition_v1_unreachable_drop CHECK (disposition_tag = 1)
);

CREATE TABLE babylon_state.campaign_foundation (
    campaign_id UUID NOT NULL,
    stable_graph BYTEA NOT NULL,
    world_registers BYTEA NOT NULL,
    resolver_manifest BYTEA NOT NULL,
    prepared_environment BYTEA NOT NULL,
    replay_session_id TEXT COLLATE pg_catalog."C" NOT NULL,
    rng_seed BIGINT NOT NULL,
    defines_hash BYTEA NOT NULL,
    rules_hash BYTEA NOT NULL,
    ref_digest BYTEA NOT NULL,
    scenario_source TEXT COLLATE pg_catalog."C" NOT NULL,
    prelude_source TEXT COLLATE pg_catalog."C",
    rule_source TEXT COLLATE pg_catalog."C" NOT NULL,
    defines_bytes BYTEA NOT NULL,
    reference_manifest_bytes BYTEA NOT NULL,
    foundation_sha256 BYTEA NOT NULL,
    CONSTRAINT campaign_foundation_pkey PRIMARY KEY (campaign_id),
    CONSTRAINT campaign_foundation_campaign_fkey FOREIGN KEY (campaign_id)
        REFERENCES babylon_state.campaign(campaign_id),
    CONSTRAINT campaign_foundation_hashes CHECK (
        pg_catalog.octet_length(defines_hash) = 32
        AND pg_catalog.octet_length(rules_hash) = 32
        AND pg_catalog.octet_length(ref_digest) = 32
        AND pg_catalog.octet_length(foundation_sha256) = 32
    )
);

CREATE TABLE babylon_state.tick_action_batch_v1 (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    layout_version SMALLINT NOT NULL CHECK (layout_version = 1),
    action_batch_digest BYTEA NOT NULL CHECK (pg_catalog.octet_length(action_batch_digest) = 32),
    exact_action_batch_bytes BYTEA NOT NULL CHECK (
        pg_catalog.octet_length(exact_action_batch_bytes) BETWEEN 55 AND 9302326
    ),
    PRIMARY KEY (campaign_id, resolve_tick),
    FOREIGN KEY (campaign_id, resolve_tick)
        REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
        DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE babylon_state.graph_node_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    local_name TEXT COLLATE pg_catalog."C" NOT NULL,
    node_type TEXT COLLATE pg_catalog."C" NOT NULL,
    PRIMARY KEY (campaign_id, resolve_tick, local_name)
);
CREATE TABLE babylon_state.graph_node_f64_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    local_name TEXT COLLATE pg_catalog."C" NOT NULL,
    qname TEXT COLLATE pg_catalog."C" NOT NULL, value_bits BIGINT NOT NULL,
    PRIMARY KEY (campaign_id, resolve_tick, local_name, qname)
);
CREATE TABLE babylon_state.graph_edge_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    edge_type TEXT COLLATE pg_catalog."C" NOT NULL,
    source_local_name TEXT COLLATE pg_catalog."C" NOT NULL,
    target_local_name TEXT COLLATE pg_catalog."C" NOT NULL,
    strength_bits BIGINT NOT NULL,
    PRIMARY KEY (campaign_id, resolve_tick, edge_type, source_local_name, target_local_name)
);
CREATE TABLE babylon_state.graph_hyperedge_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    local_name TEXT COLLATE pg_catalog."C" NOT NULL,
    hyperedge_type TEXT COLLATE pg_catalog."C" NOT NULL,
    PRIMARY KEY (campaign_id, resolve_tick, local_name)
);
CREATE TABLE babylon_state.graph_hyperedge_member_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    local_name TEXT COLLATE pg_catalog."C" NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    member TEXT COLLATE pg_catalog."C" NOT NULL,
    PRIMARY KEY (campaign_id, resolve_tick, local_name, position),
    UNIQUE (campaign_id, resolve_tick, local_name, member),
    FOREIGN KEY (campaign_id, resolve_tick, local_name)
        REFERENCES babylon_state.graph_hyperedge_v1(campaign_id, resolve_tick, local_name)
        ON DELETE CASCADE
);
CREATE TABLE babylon_state.graph_edge_f64_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    edge_type TEXT COLLATE pg_catalog."C" NOT NULL,
    source_local_name TEXT COLLATE pg_catalog."C" NOT NULL,
    target_local_name TEXT COLLATE pg_catalog."C" NOT NULL,
    qname TEXT COLLATE pg_catalog."C" NOT NULL, value_bits BIGINT NOT NULL,
    PRIMARY KEY (campaign_id, resolve_tick, edge_type, source_local_name, target_local_name, qname)
);
CREATE TABLE babylon_state.graph_node_currency_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    local_name TEXT COLLATE pg_catalog."C" NOT NULL,
    qname TEXT COLLATE pg_catalog."C" NOT NULL,
    micro_units NUMERIC(39, 0) NOT NULL,
    PRIMARY KEY (campaign_id, resolve_tick, local_name, qname)
);
CREATE TABLE babylon_state.graph_hyperedge_f64_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    local_name TEXT COLLATE pg_catalog."C" NOT NULL,
    qname TEXT COLLATE pg_catalog."C" NOT NULL, value_bits BIGINT NOT NULL,
    PRIMARY KEY (campaign_id, resolve_tick, local_name, qname)
);

CREATE TABLE babylon_state.world_register_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    register_name TEXT COLLATE pg_catalog."C" NOT NULL,
    value_tag SMALLINT NOT NULL CHECK (value_tag BETWEEN 1 AND 9),
    int_value BIGINT, currency_value NUMERIC(39, 0), real_bits BIGINT,
    ratio_bits BIGINT, ratio_min_bits BIGINT, ratio_max_bits BIGINT,
    bool_value BOOLEAN, enum_type TEXT COLLATE pg_catalog."C",
    enum_member TEXT COLLATE pg_catalog."C", stable_key BYTEA,
    CHECK (
        (value_tag = 1 AND int_value IS NOT NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 2 AND int_value IS NULL AND currency_value IS NOT NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 3 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NOT NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 4 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NOT NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 5 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NOT NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 6 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NOT NULL AND enum_member IS NOT NULL AND stable_key IS NULL)
        OR (value_tag BETWEEN 7 AND 9 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NOT NULL)
    ),
    PRIMARY KEY (campaign_id, resolve_tick, register_name)
);
CREATE TABLE babylon_state.territory_state_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    territory_id BYTEA NOT NULL,
    PRIMARY KEY (campaign_id, resolve_tick, territory_id)
);
CREATE TABLE babylon_state.territory_state_field_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    territory_id BYTEA NOT NULL, position INTEGER NOT NULL CHECK (position >= 0),
    field_name TEXT COLLATE pg_catalog."C" NOT NULL,
    value_tag SMALLINT NOT NULL CHECK (value_tag BETWEEN 1 AND 9),
    int_value BIGINT, currency_value NUMERIC(39, 0), real_bits BIGINT,
    ratio_bits BIGINT, ratio_min_bits BIGINT, ratio_max_bits BIGINT,
    bool_value BOOLEAN, enum_type TEXT COLLATE pg_catalog."C",
    enum_member TEXT COLLATE pg_catalog."C", stable_key BYTEA,
    CHECK (
        (value_tag = 1 AND int_value IS NOT NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 2 AND int_value IS NULL AND currency_value IS NOT NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 3 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NOT NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 4 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NOT NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 5 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NOT NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 6 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NOT NULL AND enum_member IS NOT NULL AND stable_key IS NULL)
        OR (value_tag BETWEEN 7 AND 9 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NOT NULL)
    ),
    PRIMARY KEY (campaign_id, resolve_tick, territory_id, position),
    UNIQUE (campaign_id, resolve_tick, territory_id, field_name),
    FOREIGN KEY (campaign_id, resolve_tick, territory_id)
        REFERENCES babylon_state.territory_state_v1(campaign_id, resolve_tick, territory_id)
        ON DELETE CASCADE
);
CREATE TABLE babylon_state.hex_state_delta_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    cell_id BIGINT NOT NULL CHECK (cell_id > 0),
    c_bits BIGINT NOT NULL, v_bits BIGINT NOT NULL, s_bits BIGINT NOT NULL, k_bits BIGINT NOT NULL,
    biocapacity_stock_bits BIGINT NOT NULL, energy_stock_bits BIGINT NOT NULL,
    raw_material_stock_bits BIGINT NOT NULL, internet_access_pct_bits BIGINT NOT NULL,
    surveillance_coupling_bits BIGINT NOT NULL,
    PRIMARY KEY (campaign_id, resolve_tick, cell_id)
);
CREATE TABLE babylon_state.organization_state_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    organization_id BYTEA NOT NULL,
    organization_kind_tag SMALLINT NOT NULL CHECK (organization_kind_tag BETWEEN 1 AND 9),
    organization_kind_int BIGINT, organization_kind_currency NUMERIC(39, 0),
    organization_kind_real_bits BIGINT, organization_kind_ratio_bits BIGINT,
    organization_kind_ratio_min_bits BIGINT, organization_kind_ratio_max_bits BIGINT,
    organization_kind_bool BOOLEAN, organization_kind_enum_type TEXT COLLATE pg_catalog."C",
    organization_kind_enum_member TEXT COLLATE pg_catalog."C", organization_kind_stable_key BYTEA,
    CHECK (
        (organization_kind_tag = 1 AND organization_kind_int IS NOT NULL AND organization_kind_currency IS NULL AND organization_kind_real_bits IS NULL AND organization_kind_ratio_bits IS NULL AND organization_kind_ratio_min_bits IS NULL AND organization_kind_ratio_max_bits IS NULL AND organization_kind_bool IS NULL AND organization_kind_enum_type IS NULL AND organization_kind_enum_member IS NULL AND organization_kind_stable_key IS NULL)
        OR (organization_kind_tag = 2 AND organization_kind_int IS NULL AND organization_kind_currency IS NOT NULL AND organization_kind_real_bits IS NULL AND organization_kind_ratio_bits IS NULL AND organization_kind_ratio_min_bits IS NULL AND organization_kind_ratio_max_bits IS NULL AND organization_kind_bool IS NULL AND organization_kind_enum_type IS NULL AND organization_kind_enum_member IS NULL AND organization_kind_stable_key IS NULL)
        OR (organization_kind_tag = 3 AND organization_kind_int IS NULL AND organization_kind_currency IS NULL AND organization_kind_real_bits IS NOT NULL AND organization_kind_ratio_bits IS NULL AND organization_kind_ratio_min_bits IS NULL AND organization_kind_ratio_max_bits IS NULL AND organization_kind_bool IS NULL AND organization_kind_enum_type IS NULL AND organization_kind_enum_member IS NULL AND organization_kind_stable_key IS NULL)
        OR (organization_kind_tag = 4 AND organization_kind_int IS NULL AND organization_kind_currency IS NULL AND organization_kind_real_bits IS NULL AND organization_kind_ratio_bits IS NOT NULL AND organization_kind_bool IS NULL AND organization_kind_enum_type IS NULL AND organization_kind_enum_member IS NULL AND organization_kind_stable_key IS NULL)
        OR (organization_kind_tag = 5 AND organization_kind_int IS NULL AND organization_kind_currency IS NULL AND organization_kind_real_bits IS NULL AND organization_kind_ratio_bits IS NULL AND organization_kind_ratio_min_bits IS NULL AND organization_kind_ratio_max_bits IS NULL AND organization_kind_bool IS NOT NULL AND organization_kind_enum_type IS NULL AND organization_kind_enum_member IS NULL AND organization_kind_stable_key IS NULL)
        OR (organization_kind_tag = 6 AND organization_kind_int IS NULL AND organization_kind_currency IS NULL AND organization_kind_real_bits IS NULL AND organization_kind_ratio_bits IS NULL AND organization_kind_ratio_min_bits IS NULL AND organization_kind_ratio_max_bits IS NULL AND organization_kind_bool IS NULL AND organization_kind_enum_type IS NOT NULL AND organization_kind_enum_member IS NOT NULL AND organization_kind_stable_key IS NULL)
        OR (organization_kind_tag BETWEEN 7 AND 9 AND organization_kind_int IS NULL AND organization_kind_currency IS NULL AND organization_kind_real_bits IS NULL AND organization_kind_ratio_bits IS NULL AND organization_kind_ratio_min_bits IS NULL AND organization_kind_ratio_max_bits IS NULL AND organization_kind_bool IS NULL AND organization_kind_enum_type IS NULL AND organization_kind_enum_member IS NULL AND organization_kind_stable_key IS NOT NULL)
    ),
    PRIMARY KEY (campaign_id, resolve_tick, organization_id)
);
CREATE TABLE babylon_state.organization_territory_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    organization_id BYTEA NOT NULL, position INTEGER NOT NULL CHECK (position >= 0),
    territory_id BYTEA NOT NULL,
    PRIMARY KEY (campaign_id, resolve_tick, organization_id, position),
    UNIQUE (campaign_id, resolve_tick, organization_id, territory_id),
    FOREIGN KEY (campaign_id, resolve_tick, organization_id)
        REFERENCES babylon_state.organization_state_v1(campaign_id, resolve_tick, organization_id)
        ON DELETE CASCADE
);
CREATE TABLE babylon_state.organization_state_field_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    organization_id BYTEA NOT NULL, position INTEGER NOT NULL CHECK (position >= 0),
    field_name TEXT COLLATE pg_catalog."C" NOT NULL,
    value_tag SMALLINT NOT NULL CHECK (value_tag BETWEEN 1 AND 9),
    int_value BIGINT, currency_value NUMERIC(39, 0), real_bits BIGINT,
    ratio_bits BIGINT, ratio_min_bits BIGINT, ratio_max_bits BIGINT,
    bool_value BOOLEAN, enum_type TEXT COLLATE pg_catalog."C",
    enum_member TEXT COLLATE pg_catalog."C", stable_key BYTEA,
    CHECK (
        (value_tag = 1 AND int_value IS NOT NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 2 AND int_value IS NULL AND currency_value IS NOT NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 3 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NOT NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 4 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NOT NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 5 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NOT NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 6 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NOT NULL AND enum_member IS NOT NULL AND stable_key IS NULL)
        OR (value_tag BETWEEN 7 AND 9 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NOT NULL)
    ),
    PRIMARY KEY (campaign_id, resolve_tick, organization_id, position),
    UNIQUE (campaign_id, resolve_tick, organization_id, field_name),
    FOREIGN KEY (campaign_id, resolve_tick, organization_id)
        REFERENCES babylon_state.organization_state_v1(campaign_id, resolve_tick, organization_id)
        ON DELETE CASCADE
);
CREATE TABLE babylon_state.tick_event_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    ordinal BIGINT NOT NULL CHECK (ordinal BETWEEN 0 AND 4294967295),
    event_type TEXT COLLATE pg_catalog."C" NOT NULL,
    PRIMARY KEY (campaign_id, resolve_tick, ordinal)
);
CREATE TABLE babylon_state.tick_event_field_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    ordinal BIGINT NOT NULL CHECK (ordinal BETWEEN 0 AND 4294967295),
    position INTEGER NOT NULL CHECK (position >= 0),
    field_name TEXT COLLATE pg_catalog."C" NOT NULL,
    value_tag SMALLINT NOT NULL CHECK (value_tag BETWEEN 1 AND 9),
    int_value BIGINT, currency_value NUMERIC(39, 0), real_bits BIGINT,
    ratio_bits BIGINT, ratio_min_bits BIGINT, ratio_max_bits BIGINT,
    bool_value BOOLEAN, enum_type TEXT COLLATE pg_catalog."C",
    enum_member TEXT COLLATE pg_catalog."C", stable_key BYTEA,
    CHECK (
        (value_tag = 1 AND int_value IS NOT NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 2 AND int_value IS NULL AND currency_value IS NOT NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 3 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NOT NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 4 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NOT NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 5 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NOT NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NULL)
        OR (value_tag = 6 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NOT NULL AND enum_member IS NOT NULL AND stable_key IS NULL)
        OR (value_tag BETWEEN 7 AND 9 AND int_value IS NULL AND currency_value IS NULL AND real_bits IS NULL AND ratio_bits IS NULL AND ratio_min_bits IS NULL AND ratio_max_bits IS NULL AND bool_value IS NULL AND enum_type IS NULL AND enum_member IS NULL AND stable_key IS NOT NULL)
    ),
    PRIMARY KEY (campaign_id, resolve_tick, ordinal, position),
    UNIQUE (campaign_id, resolve_tick, ordinal, field_name),
    FOREIGN KEY (campaign_id, resolve_tick, ordinal)
        REFERENCES babylon_state.tick_event_v1(campaign_id, resolve_tick, ordinal)
        ON DELETE CASCADE
);
CREATE TABLE babylon_state.checkpoint_manifest (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    completeness_tag SMALLINT NOT NULL CHECK (completeness_tag IN (1, 2)),
    manifest_bytes BYTEA NOT NULL,
    manifest_sha256 BYTEA NOT NULL CHECK (pg_catalog.octet_length(manifest_sha256) = 32),
    PRIMARY KEY (campaign_id, resolve_tick)
);
CREATE TABLE babylon_state.checkpoint_section_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    section_tag SMALLINT NOT NULL CHECK (section_tag BETWEEN 1 AND 9),
    ordinal BIGINT NOT NULL CHECK (ordinal BETWEEN 0 AND 4294967295),
    exact_section_bytes BYTEA NOT NULL,
    PRIMARY KEY (campaign_id, resolve_tick, section_tag, ordinal)
);
CREATE TABLE babylon_state.archive_dirty_receipt_v1 (
    campaign_id UUID NOT NULL, resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    tick_content_hash BYTEA NOT NULL CHECK (pg_catalog.octet_length(tick_content_hash) = 32),
    PRIMARY KEY (campaign_id, resolve_tick)
);

REVOKE ALL ON TABLE babylon_meta.persistence_authority_ledger FROM PUBLIC;
REVOKE ALL ON TABLE babylon_meta.python_relation_disposition_v1 FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.campaign_foundation FROM PUBLIC;
REVOKE ALL ON TABLE babylon_state.tick_action_batch_v1 FROM PUBLIC;
