-- Explicit layout metadata for newly created foundations.
-- The installer refuses existing unversioned foundations under the table lock.
CREATE TABLE babylon_state.campaign_foundation_content_layout_v2 (
    campaign_id uuid PRIMARY KEY
        REFERENCES babylon_state.campaign_foundation(campaign_id) ON DELETE CASCADE,
    content_layout_version smallint NOT NULL CHECK (content_layout_version IN (1, 2))
);
REVOKE ALL ON babylon_state.campaign_foundation_content_layout_v2 FROM PUBLIC;
CREATE TABLE babylon_meta.foundation_content_schema_v2 (
    singleton boolean PRIMARY KEY CHECK (singleton),
    migration_sha256 bytea NOT NULL CHECK (octet_length(migration_sha256) = 32)
);
REVOKE ALL ON babylon_meta.foundation_content_schema_v2 FROM PUBLIC;
