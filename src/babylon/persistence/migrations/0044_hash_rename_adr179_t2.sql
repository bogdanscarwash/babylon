-- 0044: honest hash names (ADR179 T2, Director-ruled 2026-07-30; closes
-- owner-queue item 31).
--
-- Three unrelated SHA-256 values shared the name `determinism_hash`, and the
-- name advertised a guarantee none of them provides alone:
--
--   tick_commit.determinism_hash        -> replay_identity_hash
--     sha256(session_id:tick:rng_seed) — proves replay LINEAGE, carries no
--     world state, structurally blind to content loss.
--   conservation_audit_log.determinism_hash -> hex_frame_hash
--     a genuine content hash, but over the 15-field DynamicHexState frame
--     only — its honest name says what it covers.
--
-- The third role — a full-content per-tick digest — is the P27 tick hash
-- (babylon.kernel.tick_hash, `content_hash` naming), which never used the
-- old column name and needs no migration.
--
-- Rerunnable: plain RENAME COLUMN has no IF EXISTS form, so each rename is
-- guarded by an information_schema probe; a database that already renamed
-- (or was created after the code rename plus a future fresh-DDL update)
-- no-ops. tick_commit is LIST-partitioned; RENAME COLUMN on the parent
-- cascades to all partitions (PG 17).
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'tick_commit' AND column_name = 'determinism_hash'
    ) THEN
        ALTER TABLE tick_commit RENAME COLUMN determinism_hash TO replay_identity_hash;
    END IF;
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'conservation_audit_log' AND column_name = 'determinism_hash'
    ) THEN
        ALTER TABLE conservation_audit_log RENAME COLUMN determinism_hash TO hex_frame_hash;
    END IF;
END $$;
