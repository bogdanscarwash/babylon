-- 0036_babylon_intel_role.sql
-- ADR096 D4 (queue item 13): the babylon_intel least-privilege connection role.
--
-- Database-enforced backing for Amendment V's observes-never-adjudicates
-- posture (A7.2's three-way boundary): the intelligence lane connects AS this
-- role and can therefore ONLY read the projection surface and append to the
-- narrator-prose / embedding tables. No UPDATE, no DELETE, no DDL, no Ledger.
--
-- "Hoist" (the proposed dedicated projection layer) does not exist yet, so the
-- SELECT grant targets today's projection surface: the five composition views
-- (postgres_schema.py) plus the five current-state / value-aggregate views
-- (0030_views_current.sql). When Hoist lands, a later migration narrows this.
--
-- Idempotent: guarded CREATE ROLE + re-runnable GRANTs. Both appliers re-run
-- every migration on each start; a to_regclass guard skips grants for objects
-- absent from a given database (the sim DB lacks Django's narration_record).

DO $babylon_intel_role$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'babylon_intel') THEN
        -- LOGIN: this is a CONNECTION role (the lane authenticates as it),
        -- not a NOLOGIN group. NOSUPERUSER/NOCREATEDB/NOCREATEROLE are the
        -- least-privilege floor. Password is set out-of-band by the owner
        -- (ALTER ROLE ... PASSWORD), never in a committed migration.
        CREATE ROLE babylon_intel LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT;
    END IF;
END
$babylon_intel_role$;

-- SELECT on the composition views (today's projection surface).
DO $grant_composition$
DECLARE
    v text;
    composition_views text[] := ARRAY[
        'v_hex_economic', 'v_hex_mobilize', 'v_hex_aid', 'v_hex_heat', 'v_hex_intel',
        'v_hex_state_asof', 'v_county_value_aggregate', 'v_state_value_aggregate',
        'v_national_value_aggregate', 'v_global_phi_balance'
    ];
BEGIN
    FOREACH v IN ARRAY composition_views LOOP
        IF to_regclass(v) IS NOT NULL THEN
            EXECUTE format('GRANT SELECT ON %I TO babylon_intel', v);
        END IF;
    END LOOP;
END
$grant_composition$;

-- INSERT + SELECT on the embedding table ONLY (append narrator embeddings).
DO $grant_document_chunk$
BEGIN
    IF to_regclass('document_chunk') IS NOT NULL THEN
        GRANT SELECT, INSERT ON document_chunk TO babylon_intel;
    END IF;
END
$grant_document_chunk$;

-- INSERT + SELECT on the Django-side narrator-prose table WHEN PRESENT. Absent
-- from the sim DB (applied by Django migrate on the web DB); the guard keeps
-- this migration green on both databases. Table name verified against
-- web/game/migrations/0015_narrationrecord.py: db_table = "narration_record"
-- (NOT the Django default "game_narrationrecord").
DO $grant_narration_record$
BEGIN
    IF to_regclass('narration_record') IS NOT NULL THEN
        GRANT SELECT, INSERT ON narration_record TO babylon_intel;
    END IF;
END
$grant_narration_record$;
