-- 0033_market_scissors_summary.sql
-- Program 23 (ADR077): the Market Scissors axis reaches the timeseries.
--
-- tick_summary gains two nullable observables from the Phase-1 shadow
-- MarketScissorsSystem (@17.8): price_log (ln price-index / value anchor)
-- and fictitious_log (ln fictitious / real capitalization). NULL means the
-- axis was absent that tick (no paid-worker value substrate yet) — honest
-- absence per Constitution III.11, never a fabricated 0.0.
--
-- postgres_schema.py's TICK_SUMMARY_DDL is updated in the same commit so
-- fresh databases get the columns directly; this migration is for existing
-- databases created before that change.

-- Guarded: tick_summary is created by the spec-037 bootstrap
-- (postgres_schema.py TICK_SUMMARY_DDL), not by any migration, so a
-- database that only ran migrations must not hard-fail here. ADD COLUMN
-- IF NOT EXISTS makes re-application a no-op (the runner re-applies every
-- migration each start).
DO $market_scissors$
BEGIN
    IF to_regclass('tick_summary') IS NOT NULL THEN
        ALTER TABLE tick_summary ADD COLUMN IF NOT EXISTS price_log FLOAT;
        ALTER TABLE tick_summary ADD COLUMN IF NOT EXISTS fictitious_log FLOAT;
    END IF;
END
$market_scissors$;
