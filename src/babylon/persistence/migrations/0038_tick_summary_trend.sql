-- 0038_tick_summary_trend.sql
-- T5 Unit U2: the national trend read-model — "the wind is blowing".
--
-- Declares v_national_trend (Constitution II.11) over tick_summary
-- (spec-037 bootstrap; extended by migrations 0033/0034/0035):
-- per-session, per-tick LAG-window deltas for the three headline series a
-- real Archive campaign now actually writes
-- (babylon.projection.tick_summary.build_tick_summary_kwargs, wired at
-- GameSession's commit boundary, same unit) — imperial_rent (Phi), the
-- Program 23 Market Scissors price<->value axis (price_log/fictitious_log,
-- ADR077/078), and the correction-snap ledger's own foreshadowed increment
-- read (market_corrections — migration 0034's own docstring: "the cockpit
-- derives the snap ticks from increments"). Before this unit,
-- persist_tick_summary's only caller was the legacy web bridge, so a real
-- Archive campaign wrote nothing to tick_summary at all — any view over it
-- was empty (the same dormant-construct pattern T3 closed for field-state).
--
-- tick_summary's remaining columns (year/total_c/total_v/total_s/
-- exploitation_rate/profit_rate/co_optive_edge_count/conservation_check)
-- carry no computed value from any engine system yet (see
-- build_tick_summary_kwargs's own docstring) — a trend of a permanently
-- NULL column is not a signal, so this view does not window them. Kept to
-- ONE view: tick_summary has no per-scope (county/state) breakdown of its
-- own to window separately.
--
-- Guarded: tick_summary is created by the spec-037 bootstrap
-- (postgres_schema.py TICK_SUMMARY_DDL), not by any migration, so a
-- database that only ran migrations must not hard-fail here. DROP VIEW
-- IF EXISTS + CREATE (0030/0033's idiom) rather than CREATE OR REPLACE —
-- Postgres forbids CREATE OR REPLACE VIEW from changing a view's declared
-- column set.

DO $tick_summary_trend$
BEGIN
    IF to_regclass('tick_summary') IS NOT NULL THEN
        DROP VIEW IF EXISTS v_national_trend;

        CREATE VIEW v_national_trend AS
        SELECT
            session_id,
            tick,
            imperial_rent,
            imperial_rent - LAG(imperial_rent) OVER (
                PARTITION BY session_id ORDER BY tick
            ) AS imperial_rent_delta,
            price_log,
            price_log - LAG(price_log) OVER (
                PARTITION BY session_id ORDER BY tick
            ) AS price_log_delta,
            fictitious_log,
            fictitious_log - LAG(fictitious_log) OVER (
                PARTITION BY session_id ORDER BY tick
            ) AS fictitious_log_delta,
            market_corrections,
            market_corrections - LAG(market_corrections) OVER (
                PARTITION BY session_id ORDER BY tick
            ) AS market_corrections_delta
        FROM tick_summary;
    END IF;
END
$tick_summary_trend$;
