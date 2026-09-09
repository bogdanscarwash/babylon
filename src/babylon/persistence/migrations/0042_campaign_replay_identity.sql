-- 0042: replay-identity columns on the campaign catalog (ADR176 ruling 28,
-- P-J defect 3/3). Determinism makes a campaign a pure function of
-- (rng_seed, ContentDigest, tick) — persisting both converts the worst
-- failure mode from "save lost" to "rebuild save".
--
-- rng_seed: the campaign's SimulationConfig seed (Constitution III.7),
--   stamped at session creation; NULL = minted before these columns.
-- content_digest: the canonical P27 ContentDigest serialized form
--   ({"defines_hash":"<64-hex>","rules_hash":"<64-hex>"}, compact JSON,
--   sorted keys — docs/reference/determinism-contract.rst, "ContentDigest
--   and the Canonical BSL AST Serialization"). NULL in the Python era:
--   rules_hash hashes BSL rule content, which does not exist before the
--   Rust engine — a fabricated stand-in would poison the canonical value.
--
-- Rerunnable: both ALTERs are IF NOT EXISTS. Mirrors
-- BABYLON_META_CAMPAIGN_MIGRATIONS_DDL in postgres_schema.py.
ALTER TABLE babylon_meta.campaign ADD COLUMN IF NOT EXISTS rng_seed BIGINT;
ALTER TABLE babylon_meta.campaign ADD COLUMN IF NOT EXISTS content_digest TEXT;
