-- 0034_market_corrections.sql
-- Program 23 Phase 2 (ADR078): the correction ledger reaches the timeseries.
--
-- tick_summary gains market_corrections — the CUMULATIVE count of correction
-- snaps as of that tick (MarketState.corrections, an event-sourced
-- accumulator carried as a real model field). The cockpit derives the snap
-- ticks from increments. NULL means the market axis was absent that tick
-- (honest absence per Constitution III.11, matching price_log/fictitious_log
-- from 0033) — never a fabricated 0.
--
-- postgres_schema.py's TICK_SUMMARY_DDL is updated in the same commit so
-- fresh databases get the column directly; this migration is for existing
-- databases created before that change.

-- Guarded: tick_summary is created by the spec-037 bootstrap
-- (postgres_schema.py TICK_SUMMARY_DDL), not by any migration, so a
-- database that only ran migrations must not hard-fail here. ADD COLUMN
-- IF NOT EXISTS makes re-application a no-op (the runner re-applies every
-- migration each start).
DO $market_corrections$
BEGIN
    IF to_regclass('tick_summary') IS NOT NULL THEN
        ALTER TABLE tick_summary ADD COLUMN IF NOT EXISTS market_corrections INT;
    END IF;
END
$market_corrections$;
