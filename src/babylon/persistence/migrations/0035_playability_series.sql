-- 0035_playability_series.sql
-- Playability Spine Task 19 (spec-116 4d.5, ADR079): the crisis/bifurcation
-- history reaches the timeseries.
--
-- tick_summary gains five nullable county-deduped aggregates of the
-- year-boundary tick_* territory attrs (crisis phase share, bifurcation
-- score, wage compression, capital stock, unemployment). NULL = no county
-- carried boundary state that tick (honest absence per Constitution III.11
-- — the attrs stamp at year boundaries only, so each series is a step
-- function with a NULL head, never fabricated smoothing).
--
-- postgres_schema.py's TICK_SUMMARY_DDL is updated in the same commit so
-- fresh databases get the columns directly; this migration is for existing
-- databases created before that change.

-- Guarded: tick_summary is created by the spec-037 bootstrap
-- (postgres_schema.py TICK_SUMMARY_DDL), not by any migration, so a
-- database that only ran migrations must not hard-fail here. ADD COLUMN
-- IF NOT EXISTS makes re-application a no-op (both appliers re-run every
-- migration each start).
DO $playability_series$
BEGIN
    IF to_regclass('tick_summary') IS NOT NULL THEN
        ALTER TABLE tick_summary ADD COLUMN IF NOT EXISTS crisis_pop_share FLOAT;
        ALTER TABLE tick_summary ADD COLUMN IF NOT EXISTS bifurcation_score_mean FLOAT;
        ALTER TABLE tick_summary ADD COLUMN IF NOT EXISTS wage_compression_mean FLOAT;
        ALTER TABLE tick_summary ADD COLUMN IF NOT EXISTS capital_stock_total FLOAT;
        ALTER TABLE tick_summary ADD COLUMN IF NOT EXISTS unemployment_rate_mean FLOAT;
    END IF;
END
$playability_series$;
