-- 0043: the simulation_event natural-key unique index (#388; the CI-
-- authorizations train, ADR176 ruling 33 surfaced it: the PG tier un-gate
-- makes persist_tick(events=...) run everywhere the runtime schema runs).
--
-- _persist_events writes with ON CONFLICT (session_id, tick, event_type,
-- COALESCE(entity_id,''), COALESCE(community_type,'')) DO NOTHING — an
-- idempotency spec whose index existed ONLY in the legacy web/Django chain
-- (its migration 0009). On every runtime-schema-only database the INSERT
-- raised InvalidColumnReference instead.
--
-- Dedupe first: a database that lived without the index may hold natural-
-- key duplicates, and a UNIQUE index over them fails to build. Keeping the
-- earliest id per signature IS the declared ON CONFLICT semantic ("one row
-- per distinct event signature per tick"), applied retroactively.
--
-- Rerunnable: the DELETE is a no-op once deduped; the CREATE is IF NOT
-- EXISTS. Mirrors SIMULATION_EVENT_NATURAL_KEY_DDL in postgres_schema.py.
DELETE FROM simulation_event a USING simulation_event b
WHERE a.id > b.id
  AND a.session_id = b.session_id
  AND a.tick = b.tick
  AND a.event_type = b.event_type
  AND COALESCE(a.entity_id, '') = COALESCE(b.entity_id, '')
  AND COALESCE(a.community_type, '') = COALESCE(b.community_type, '');
CREATE UNIQUE INDEX IF NOT EXISTS ux_simulation_event_session_tick_natural
ON simulation_event (
    session_id, tick, event_type,
    COALESCE(entity_id, ''), COALESCE(community_type, '')
);
