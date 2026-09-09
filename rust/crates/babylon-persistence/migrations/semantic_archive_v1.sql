-- Additive client-owned semantic Archive schema; not a persistence-authority epoch.
CREATE TABLE babylon_meta.semantic_archive_schema_v1 (
    contract_id TEXT PRIMARY KEY CHECK (contract_id = 'babylon.semantic-archive-schema.v1')
);

CREATE TABLE babylon_meta.archive_knowledge_grant_v1 (
    campaign_id UUID NOT NULL,
    subject_kind TEXT NOT NULL CHECK (subject_kind IN ('county', 'place')),
    subject_id TEXT NOT NULL CHECK (
        (subject_kind = 'county' AND subject_id ~ '^[0-9]{5}$') OR
        (subject_kind = 'place' AND subject_id ~ '^[0-9]{7}$')
    ),
    grant_key TEXT NOT NULL CHECK (grant_key ~ '^[a-z0-9][a-z0-9-]{0,127}$'),
    granted_tick BIGINT NOT NULL CHECK (granted_tick >= 0),
    provenance_source_id TEXT NOT NULL CHECK (
        pg_catalog.octet_length(provenance_source_id) BETWEEN 1 AND 4096
    ),
    provenance_locator TEXT NOT NULL CHECK (
        pg_catalog.octet_length(provenance_locator) BETWEEN 1 AND 4096
    ),
    PRIMARY KEY (campaign_id, subject_kind, subject_id, grant_key),
    FOREIGN KEY (campaign_id) REFERENCES babylon_meta.campaign(campaign_id) ON DELETE CASCADE
);

CREATE TABLE babylon_meta.archive_receipt_consumption_v1 (
    campaign_id UUID NOT NULL,
    resolve_tick BIGINT NOT NULL CHECK (resolve_tick >= 1),
    tick_content_hash BYTEA NOT NULL CHECK (pg_catalog.octet_length(tick_content_hash) = 32),
    batch_sha256 BYTEA NOT NULL CHECK (pg_catalog.octet_length(batch_sha256) = 32),
    worker_contract_sha256 BYTEA NOT NULL CHECK (
        pg_catalog.octet_length(worker_contract_sha256) = 32
    ),
    knowledge_sha256 BYTEA NOT NULL CHECK (pg_catalog.octet_length(knowledge_sha256) = 32),
    PRIMARY KEY (campaign_id, resolve_tick),
    FOREIGN KEY (campaign_id, resolve_tick)
        REFERENCES babylon_state.archive_dirty_receipt_v1(campaign_id, resolve_tick)
        ON DELETE CASCADE,
    FOREIGN KEY (campaign_id, resolve_tick)
        REFERENCES babylon_state.tick_commit(campaign_id, resolve_tick)
        ON DELETE CASCADE
);

CREATE TABLE babylon_meta.archive_page_v1 (
    campaign_id UUID NOT NULL,
    subject_kind TEXT NOT NULL CHECK (subject_kind IN ('county', 'place')),
    subject_id TEXT NOT NULL,
    subject_grant_key TEXT NOT NULL DEFAULT 'subject' CHECK (subject_grant_key = 'subject'),
    title TEXT NOT NULL CHECK (pg_catalog.octet_length(title) BETWEEN 1 AND 4096),
    verified_tick BIGINT NOT NULL CHECK (verified_tick >= 1),
    source_resolve_tick BIGINT NOT NULL CHECK (source_resolve_tick >= 1),
    source_tick_content_hash BYTEA NOT NULL CHECK (
        pg_catalog.octet_length(source_tick_content_hash) = 32
    ),
    template_sha256 BYTEA NOT NULL CHECK (pg_catalog.octet_length(template_sha256) = 32),
    content_sha256 BYTEA NOT NULL CHECK (pg_catalog.octet_length(content_sha256) = 32),
    markdown TEXT NOT NULL CHECK (pg_catalog.octet_length(markdown) <= 1048576),
    search_text TEXT NOT NULL CHECK (pg_catalog.octet_length(search_text) <= 1048576),
    provenance_json TEXT NOT NULL CHECK (pg_catalog.octet_length(provenance_json) <= 1048576),
    PRIMARY KEY (campaign_id, subject_kind, subject_id),
    FOREIGN KEY (campaign_id, subject_kind, subject_id, subject_grant_key)
        REFERENCES babylon_meta.archive_knowledge_grant_v1(
            campaign_id, subject_kind, subject_id, grant_key
        ),
    -- PER-318: a page's provenance anchors at the durable dirty receipt, not
    -- the consumption marker, so a staged batch writes pages while its
    -- receipt stays pending and the drain pages across sweeps.
    FOREIGN KEY (campaign_id, source_resolve_tick)
        REFERENCES babylon_state.archive_dirty_receipt_v1(campaign_id, resolve_tick)
        ON DELETE CASCADE
);

INSERT INTO babylon_meta.semantic_archive_schema_v1 (contract_id)
VALUES ('babylon.semantic-archive-schema.v1');

REVOKE ALL ON TABLE babylon_meta.semantic_archive_schema_v1 FROM PUBLIC;
REVOKE ALL ON TABLE babylon_meta.archive_knowledge_grant_v1 FROM PUBLIC;
REVOKE ALL ON TABLE babylon_meta.archive_receipt_consumption_v1 FROM PUBLIC;
REVOKE ALL ON TABLE babylon_meta.archive_page_v1 FROM PUBLIC;
