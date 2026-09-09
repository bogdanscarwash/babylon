# Engine defects surfaced by the spec-113 live audit

Two P1 defects the Living Map audit surfaced live. **Both are engine-side
(`src/babylon/`), so the charter forbids fixing them on `feature/113-living-map`.**
This is the engine-team-ready writeup: exact root cause, one-line fix, a proof it
does not touch determinism, and a failing test. Apply on a fresh branch off `dev`
(suggested: `fix/event-type-persistence`), not on the frontend branch.

## TL;DR — one root cause, two symptoms

A **field-name mismatch** in the web/interactive persistence path makes *every*
event persist as `event_type = "UNKNOWN"`:

- **Symptom A (resolve 500 → session dies ~tick 18):** two events at one tick both
  become `UNKNOWN` with empty `entity_id`/`community_type`, so they collide on the
  unique index `ux_simulation_event_session_tick_natural`, and `_persist_events`
  has no `ON CONFLICT` clause → `psycopg.errors.UniqueViolation` → the resolve
  request 500s → the game can't be played past the first same-tick event pair.
- **Symptom B (the game never speaks):** the web layer classifies urgency by
  event type; with everything `UNKNOWN`, nothing is ever urgent → zero toasts, an
  empty wire, in 20+ live ticks.

## Root cause (evidence chain)

1. The event model — `src/babylon/models/events/_legacy.py:93` — declares the
   field as **`event_type`** (an `EventType` enum):
   ```python
   class SimulationEvent(BaseModel):
       event_type: EventType = Field(..., description="Type of simulation event")
   ```
2. `web/game/engine_bridge.py` serializes events with
   `event.model_dump(mode="json")` → the dict key is therefore **`"event_type"`**.
3. But `src/babylon/persistence/postgres_runtime/_legacy.py:2354`
   (`_persist_events`) reads the wrong key:
   ```python
   e.get("type", "UNKNOWN"),   # ← always misses; there is no "type" key
   ```
   → every row is inserted with `event_type = "UNKNOWN"`.
4. The unique index (`web/game/migrations/0009_action_result_unique.py:46`) is
   `(session_id, tick, event_type, COALESCE(entity_id,''), COALESCE(community_type,''))`.
   With `event_type` constant-`UNKNOWN` and base events carrying no
   `entity_id`/`community_type`, two events at one tick share a natural key.
5. `_persist_events` inserts via a plain `executemany` with **no `ON CONFLICT`**
   (unlike the spec-061 `persist_full_tick`, which has one — but that method has
   **zero callers**; the live path is `persist_tick` → `_persist_events`).

## The fix (primary — one line)

`src/babylon/persistence/postgres_runtime/_legacy.py:2354`:
```python
-                        e.get("type", "UNKNOWN"),
+                        e.get("event_type", "UNKNOWN"),
```
This alone fixes **both** symptoms: real event types make the natural keys
distinct (no more collision) *and* let the web layer classify urgency (toasts +
wire come alive).

### Defence-in-depth (recommended, same commit)

Make the insert idempotent so a genuine same-signature retry can never 500 again
(mirrors the intent the migration docstring already documents):
```python
INSERT INTO simulation_event (...)
VALUES (%s, %s, %s, %s, %s, %s)
ON CONFLICT (session_id, tick, event_type,
             COALESCE(entity_id, ''), COALESCE(community_type, ''))
DO NOTHING
```
(Postgres needs the `COALESCE` expressions to match the index exactly, or target
the index by a named constraint.)

## Why this is safe for determinism / qa:regression

`qa:regression` and the golden traces run through the **headless runner**, which
captures events via `e.event_type` (`src/babylon/engine/headless_runner/
runner.py`, `event_capture.py`) — the *correct* attribute — and never calls
`_persist_events` (that is the Postgres/interactive path only). So this fix
changes **only what the web session stores**, not any tick math or hash. Expect
`qa:regression` to stay byte-identical; run it once to certify per the DoD.

## Failing test to add (TDD, `tests/unit/persistence/`)

```python
def test_persist_events_uses_event_type_key_not_type(pg_runtime, session_id):
    # Two DIFFERENT-typed events at the same tick must persist as distinct rows,
    # not collapse to UNKNOWN and collide on ux_simulation_event_session_tick_natural.
    events = [
        {"event_type": "SURPLUS_EXTRACTION", "tick": 3},
        {"event_type": "RADICALIZATION", "tick": 3},
    ]
    pg_runtime.persist_tick(session_id, tick=3, graph=empty_graph(), events=events)
    rows = query("SELECT event_type FROM simulation_event "
                 "WHERE session_id=%s AND tick=3 ORDER BY event_type", session_id)
    assert [r.event_type for r in rows] == ["RADICALIZATION", "SURPLUS_EXTRACTION"]
    # (RED before the fix: raises UniqueViolation — both rows are "UNKNOWN".)
```

## Verification after the fix

1. `uv run pytest tests/unit/persistence/ -q` — the new test green.
2. `mise run qa:regression` — expect **byte-identical** (proof above).
3. Live: play a `wayne_county` session past tick 18 without a resolve 500, and
   confirm the spec-113 `event-popup.spec.ts` + `end-turn-flow.spec.ts` (spacebar)
   e2e go green — they are red **by design** today, pinning exactly this defect.

## Downstream (once events carry real types)

The two spec-113 e2e reds flip green, the wire/toasts come alive, and the
`get_inspector_hex` stub (the hex InspectionCard "no data" owner item) becomes the
next thing to wire — but that is a separate data-source decision (see the
integration ledger's Phase-V owner items).
