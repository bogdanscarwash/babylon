-- 0041_trend_playability.sql
-- M6 Task 41 (contract docs/superpowers/specs/2026-07-28-m6-market-contracts.md
-- §1b): re-declare v_national_trend to window the five LIVE playability
-- columns migration 0035 added to tick_summary — crisis_pop_share,
-- bifurcation_score_mean, wage_compression_mean, capital_stock_total,
-- unemployment_rate_mean — plus their five LAG deltas.
--
-- Rationale: 0038's stated exclusion covers only PERMANENTLY-NULL columns
-- ("a trend of a permanently NULL column is not a signal"). These five are
-- genuinely computed each committed tick by
-- babylon.projection.tick_summary.build_tick_summary_kwargs's county-dedup
-- pass (population-weighted means / extensive capital sum; None only
-- before the first year-boundary stamp — honest sparsity, not dead
-- columns), so by the view's own logic they belong in the window: live
-- signal was invisible to ANY trend read. They ARE the playability series
-- a market dashboard exists to show.
--
-- NUMBER DEVIATION (recorded, M6 contract §5): the contract allocated
-- "0039" pre-P26; P26 landed 0039_domain_contracts + 0040_receipts_vocabulary
-- first, and the runner globs 00*.sql sorted — next free slot is 0041.
--
-- Idiom is 0038's verbatim: guarded on the spec-037 bootstrap table's
-- presence (a migrations-only database must not hard-fail), DROP VIEW +
-- CREATE VIEW — never CREATE OR REPLACE (Postgres forbids OR REPLACE from
-- changing a view's declared column set, which is exactly what this does).

DO $trend_playability$
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
            ) AS market_corrections_delta,
            crisis_pop_share,
            crisis_pop_share - LAG(crisis_pop_share) OVER (
                PARTITION BY session_id ORDER BY tick
            ) AS crisis_pop_share_delta,
            bifurcation_score_mean,
            bifurcation_score_mean - LAG(bifurcation_score_mean) OVER (
                PARTITION BY session_id ORDER BY tick
            ) AS bifurcation_score_mean_delta,
            wage_compression_mean,
            wage_compression_mean - LAG(wage_compression_mean) OVER (
                PARTITION BY session_id ORDER BY tick
            ) AS wage_compression_mean_delta,
            capital_stock_total,
            capital_stock_total - LAG(capital_stock_total) OVER (
                PARTITION BY session_id ORDER BY tick
            ) AS capital_stock_total_delta,
            unemployment_rate_mean,
            unemployment_rate_mean - LAG(unemployment_rate_mean) OVER (
                PARTITION BY session_id ORDER BY tick
            ) AS unemployment_rate_mean_delta
        FROM tick_summary;
    END IF;
END
$trend_playability$;
