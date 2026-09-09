-- 0037_babylon_meta.sql
-- Program 24 WO-46 (charter P0 ruling 3): the babylon_meta epistemic tier.
--
-- CLIENT-owned state — the Archive TUI writes these tables (campaign
-- catalog + watchlist / jumplist / breadcrumb session navigation); the
-- engine never reads or writes them, and no tick-hash input derives from
-- them (the epistemic / material partition: player knowledge lives outside
-- the deterministic Ledger, fog-epistemic-vs-material). A dedicated
-- Postgres schema makes that boundary structural rather than conventional.
--
-- Mirrors postgres_schema.py's BABYLON_META_DDL (the single DDL source of
-- truth for fresh databases); this migration heals existing databases.

CREATE SCHEMA IF NOT EXISTS babylon_meta;

CREATE TABLE IF NOT EXISTS babylon_meta.campaign (
    campaign_id UUID PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    engine_version TEXT NOT NULL,
    defines_hash TEXT NOT NULL,
    last_tick INTEGER NOT NULL DEFAULT 0 CHECK (last_tick >= 0),
    status TEXT NOT NULL DEFAULT 'ACTIVE'
        CHECK (status IN ('ACTIVE', 'ABANDONED')),
    last_played_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS babylon_meta.watchlist (
    campaign_id UUID NOT NULL REFERENCES babylon_meta.campaign(campaign_id)
        ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    entity_id TEXT NOT NULL,
    PRIMARY KEY (campaign_id, position),
    UNIQUE (campaign_id, entity_id)
);

CREATE TABLE IF NOT EXISTS babylon_meta.jumplist (
    campaign_id UUID NOT NULL REFERENCES babylon_meta.campaign(campaign_id)
        ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    entity_id TEXT NOT NULL,
    PRIMARY KEY (campaign_id, position)
);

CREATE TABLE IF NOT EXISTS babylon_meta.breadcrumb (
    campaign_id UUID NOT NULL REFERENCES babylon_meta.campaign(campaign_id)
        ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    entity_id TEXT NOT NULL,
    PRIMARY KEY (campaign_id, position)
);
