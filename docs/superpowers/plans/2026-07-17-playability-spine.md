# Playability Spine Implementation Plan (spec-116, Program 24)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the base loop playable under any information model: a 100-year fixed-horizon
campaign that holds tension under null play, an event rail that is signal not wallpaper, a
first session that explains itself, and nine evidence-backed wirings connecting existing
engine riches to existing UI surfaces.

**Architecture:** Engine changes are confined to (a) one seeding-bug exemption, (b) the
EndgameDetector's repurposing from adjudicator to pattern recognizer, (c) a
defines-level pacing recalibration executed as declared ceremony #1 (Task 6), and (d) the
event-whitelist widening executed as declared ceremony #2 (Task 23). Everything else lives
at the serialization boundary (bridge) or in the frontend — byte-safe by construction.

**Tech Stack:** Python 3.11 / Pydantic frozen models / Django bridge (`web/game/`) /
React+TS+zustand cockpit (`src/frontend/`) / pytest + vitest + Playwright / mise tasks.

## Global Constraints

- **Owner ruling (2026-07-17):** campaign = fixed horizon of 100 in-game years
  (5200 ticks at `timescale.weeks_per_year: 52`); the five `GameOutcome` values are
  recognized patterns, never terminators; nothing ends a session early except the
  player's explicit accept-outcome.
- Branch `feature/116-playability-spine`; conventional commits; commit after each task
  (plain `git commit` — hooks run; `mise run commit --` sweeps partial staging).
- Per-task gate: `mise run test:q -- <scoped path>` green; per-cluster gate:
  `mise run check` green (includes `check:seams`).
- `mise run qa:regression` must stay 5/5. `defines_hash` drift alone is advisory
  (WARNING) and passes; dense-CSV drift fails. Baselines are regenerated ONLY in the
  TWO declared ceremony commits — Task 6 Step 7 (pacing calibration, ceremony #1) and
  Task 23 Step 8 (event-whitelist widening, ceremony #2) — each with per-scenario drift
  declared in the commit body. Every other task leaves baselines byte-identical.
- Every new bridge-serialized wire key gets a `SeamEntry` row in
  `src/babylon/sentinels/seam/registry.py` (otherwise `check:seams` reds the build).
- New defines fields: edit `src/babylon/config/defines/*.py`, regenerate with
  `poetry run python tools/generate_defines_config.py`, never hand-edit
  `src/babylon/data/defines.yaml` (`tests/unit/config/test_constants_sync.py` guards).
- Bridge/API tests live in `tests/unit/web/` (NOT `web/game/tests/`); frontend unit =
  vitest (`cd src/frontend && npx vitest run <path>`); any new session-mutating e2e spec
  MUST be added to `AUTHENTICATED_SPECS` in `src/frontend/playwright.config.ts`.
- **FOLLOW-PATTERN(<file>) convention:** where a step marks a fixture/helper with
  FOLLOW-PATTERN, open that file first and reuse its existing builder/mocking idiom
  verbatim; the assertion blocks in this plan are normative, the fixture scaffolding
  follows the named file. (This prevents plan-fabricated fixture APIs.)
- No `test_` prefix in production code; strict typing; RST docstrings on public
  classes/functions; frozen pydantic models; loops statically bounded (Power-of-10).
- Machine safety: tests run single-flight, scoped (`test:q`); never fan out parallel
  pytest.

---
## Cluster A — Fixed horizon, pattern recognition, pacing (FR-116-1, FR-116-5)

### Task 1: Defines — campaign horizon, pattern lock, fascist fraction, balkanization composition

**Files:**
- Modify: `src/babylon/config/defines/endgame.py` (EndgameDefines, ~lines 39-69)
- Modify: `src/babylon/config/defines/_assembler.py` (~lines 38, 145, 291)
- Regenerate: `src/babylon/data/defines.yaml` (via tool, never by hand)
- Test: `tests/unit/config/test_endgame_defines_spine.py` (new)

**Interfaces:**
- Consumes: existing `EndgameDefines`, `BalkanizationDefines` (src/babylon/config/defines/balkanization.py), assembler pattern (`endgame: EndgameDefines = Field(default_factory=EndgameDefines)` at _assembler.py:145, from-dict at :291).
- Produces: `GameDefines().endgame.campaign_horizon_years: int = 100`,
  `.endgame.pattern_lock_ticks: int = 26`, `.endgame.fascist_majority_fraction: float = 0.75`,
  `GameDefines().balkanization: BalkanizationDefines`. (`fascist_majority_threshold` stays
  for now; Task 3 deletes it with its last consumer.)

- [ ] **Step 1: Write the failing test**

```python
"""Spec-116 FR-116-1: fixed-horizon + recognizer defines exist and load from YAML."""

import pytest

from babylon.config.defines import GameDefines


@pytest.mark.unit
def test_campaign_horizon_and_pattern_defines_exist() -> None:
    defines = GameDefines.load_default()
    assert defines.endgame.campaign_horizon_years == 100
    assert defines.endgame.pattern_lock_ticks == 26
    assert defines.endgame.fascist_majority_fraction == 0.75


@pytest.mark.unit
def test_balkanization_is_composed_into_game_defines() -> None:
    defines = GameDefines.load_default()
    assert defines.balkanization.red_ogv_habitability_floor == 0.4
    assert defines.balkanization.fragmented_collapse_min_sovereigns == 3


@pytest.mark.unit
def test_horizon_ticks_derivation() -> None:
    defines = GameDefines.load_default()
    horizon = defines.endgame.campaign_horizon_years * defines.timescale.weeks_per_year
    assert horizon == 5200
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/config/test_endgame_defines_spine.py`
Expected: FAIL — `AttributeError: ... has no attribute 'campaign_horizon_years'` (and
`'GameDefines' object has no attribute 'balkanization'`).

- [ ] **Step 3: Add the three EndgameDefines fields**

In `src/babylon/config/defines/endgame.py`, inside `EndgameDefines` after
`fascist_majority_threshold`:

```python
    campaign_horizon_years: int = Field(
        default=100,
        ge=1,
        le=1000,
        description=(
            "Game design: fixed campaign horizon in in-game years. The game ends "
            "only when tick >= horizon_years * timescale.weeks_per_year (owner "
            "ruling 2026-07-17: outcomes are recognized patterns, never terminators)."
        ),
    )
    pattern_lock_ticks: int = Field(
        default=26,
        ge=1,
        le=520,
        description=(
            "Game design: consecutive ticks a recognized outcome pattern must hold "
            "before it is 'locked' and the Council may accept the outcome early."
        ),
    )
    fascist_majority_fraction: float = Field(
        default=0.75,
        ge=0.5,
        le=1.0,
        description=(
            "Game design: fraction of social-class nodes with national_identity > "
            "class_consciousness required to recognize FASCIST_CONSOLIDATION "
            "(replaces the scenario-size-degenerate absolute count)."
        ),
    )
```

- [ ] **Step 4: Compose BalkanizationDefines into the assembler**

In `src/babylon/config/defines/_assembler.py`: add the import next to the endgame import
(~line 38), the field next to `endgame:` (~line 145), and the from-dict entry next to the
endgame entry (~line 291) — copy the endgame lines' exact shape:

```python
from babylon.config.defines.balkanization import BalkanizationDefines
...
    balkanization: BalkanizationDefines = Field(default_factory=BalkanizationDefines)
...
            balkanization=BalkanizationDefines(**data.get("balkanization", {})),
```

Also update the assembler's category docstring list (~line 109) with
`- balkanization: Sovereignty/collapse and RED_OGV/FRAGMENTED endgame thresholds`.

- [ ] **Step 5: Regenerate defines.yaml**

Run: `poetry run python tools/generate_defines_config.py`
Expected: `src/babylon/data/defines.yaml` gains the three endgame keys + a
`balkanization:` block. Verify: `rg -n 'campaign_horizon_years|balkanization:' src/babylon/data/defines.yaml`

- [ ] **Step 6: Run the new test + the sync guard**

Run: `mise run test:q -- tests/unit/config/test_endgame_defines_spine.py tests/unit/config/test_constants_sync.py`
Expected: PASS (all).

- [ ] **Step 7: Verify regression is value-stable**

Run: `mise run qa:regression`
Expected: `5 passed, 0 failed`. `defines_hash` WARNING lines are expected (schema grew)
and pass; any dense-CSV diff is a STOP — no engine code changed yet, values cannot move.

- [ ] **Step 7b: Seed ADR079 (spec header requires it in the FIRST commit batch)**

Create `ai/decisions/ADR079_playability_spine.yaml` with `status: proposed`, the
fixed-horizon owner ruling (100y / 5200 ticks, outcomes = recognized patterns), and
placeholder sections for the ceremony drift tables; add its row to
`ai/decisions/index.yaml` (FOLLOW-PATTERN: the ADR078 entry). Task 25 Step 5 flips it
to `accepted` and completes it — do not wait for Task 25 to create the file.

- [ ] **Step 8: Commit**

```bash
git add src/babylon/config/defines/endgame.py src/babylon/config/defines/_assembler.py \
  src/babylon/data/defines.yaml tests/unit/config/test_endgame_defines_spine.py \
  ai/decisions/ADR079_playability_spine.yaml ai/decisions/index.yaml
git commit -m "feat(defines): campaign horizon, pattern lock, fascist fraction; compose balkanization; seed ADR079 (spec-116 FR-116-1)"
```

---

### Task 2: CollapseTransition — exempt the exterior null sovereign (tick-0 Sovereign Collapse bug)

**Files:**
- Modify: `src/babylon/engine/systems/collapse_transition.py:69-79` (Phase-1 predicate)
- Test: `tests/unit/balkanization/test_collapse_transition_system.py` (extend)

**Interfaces:**
- Consumes: nothing new.
- Produces: `SOV_EXTERIOR_NULL` never collapses; no `SOVEREIGN_COLLAPSE` /
  `TERRITORY_TRANSITION` spam at tick 0 or any tick from the null sovereign.

- [ ] **Step 1: Write the failing test** — FOLLOW-PATTERN
  (`tests/unit/balkanization/test_collapse_transition_system.py`: reuse that file's
  existing graph/state fixture builder for sovereigns with CLAIMS edges). Normative
  assertions:

```python
@pytest.mark.unit
def test_exterior_null_sovereign_never_collapses() -> None:
    # Fixture: SOV_EXTERIOR_NULL with legitimacy 0.0 + CLAIMS on two territories,
    # plus a regular sovereign "SOV_TEST" with legitimacy 0.0 and one CLAIMS edge.
    # Run CollapseTransitionSystem.step once (tick 0 semantics).
    ...
    # The null sovereign is exempt (FR-040b boundary fallback, spec-116 FR-116-1):
    assert null_sovereign_still_has_claims          # CLAIMS edges intact
    assert "SOV_EXTERIOR_NULL" not in collapse_event_subjects
    # The regular sovereign still collapses (the predicate itself is unchanged):
    assert "SOV_TEST" in collapse_event_subjects
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/balkanization/test_collapse_transition_system.py`
Expected: FAIL — the null sovereign collapses (CLAIMS stripped, SOVEREIGN_COLLAPSE emitted).

- [ ] **Step 3: Implement the exemption**

In `collapse_transition.py`, at the top of the Phase-1 loop body (immediately after
`for sovereign_id in sovereign_ids:`):

```python
            # The exterior null sovereign is the FR-040b boundary fallback —
            # not a polity that can collapse. Mirrors the Phase-3
            # orphan-cleanup exemption below.
            if sovereign_id == "SOV_EXTERIOR_NULL":
                continue
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/balkanization/test_collapse_transition_system.py`
Expected: PASS (whole file — pre-existing cases must stay green).

- [ ] **Step 5: Byte-safety check**

Run: `mise run qa:regression`
Expected: `5 passed, 0 failed` — the qa scenarios build without the bridge's
balkanization seeding, so no sovereign nodes exist in the baselines and this change is
byte-identical there. Any drift is a STOP-and-investigate.

- [ ] **Step 6: Commit**

```bash
git add src/babylon/engine/systems/collapse_transition.py tests/unit/balkanization/test_collapse_transition_system.py
git commit -m "fix(engine): exempt SOV_EXTERIOR_NULL from Phase-1 collapse — kills tick-0 Sovereign Collapse spam (spec-116)"
```

---

### Task 3: EndgameDetector → pattern recognizer

**Files:**
- Modify: `src/babylon/engine/observers/endgame_detector.py` (major rework, 677 lines today)
- Modify: `src/babylon/config/defines/endgame.py` (DELETE `fascist_majority_threshold`) +
  regenerate `src/babylon/data/defines.yaml`
- Modify (mechanical caller updates, semantics preserved this task):
  `web/game/engine_bridge.py:4643-4654, 4777-4782, 3894-3940 (references only)`,
  `src/babylon/engine/headless_runner/argparse_cli.py` (the `--endgame-detector` poll)
- Test: `tests/unit/engine/test_endgame_detector.py` (rework),
  `tests/scenarios/test_endgame_flow.py` + `tests/integration/test_endgame_detection_round_trip.py` (adapt)

**Interfaces:**
- Consumes: Task 1's `defines.endgame.fascist_majority_fraction`, `defines.balkanization`.
- Produces (the recognizer API every later task relies on):
  - `EndgameDetector.recognized_pattern -> GameOutcome | None` (property)
  - `EndgameDetector.pattern_since_tick -> int | None` (property)
  - `EndgameDetector.axis_progress() -> dict[str, float]` with EXACTLY the keys
    `"revolutionary_victory" | "ecological_collapse" | "fascist_consolidation" |
    "red_ogv" | "fragmented_collapse"`, values in [0.0, 1.0].
  - `is_game_over`, `outcome`, and `_emit_endgame_event` are DELETED.

Recognition semantics: `on_tick` re-evaluates ALL five axes every tick (no early-return
once recognized — patterns can dissolve). Each axis evaluates to
`(progress: float, matched: bool)` where `progress` = mean of that axis's clamped gate
ratios and `matched ⟺ every gate ratio clamps to 1.0` (so `matched ⟺ progress == 1.0`).
DISJUNCTIVE-AXIS EXCEPTION (adjudicated at Task 3 review): FASCIST_CONSOLIDATION has
two pre-existing independent routes (false-consciousness fraction OR the three-gate
political-violence route); its `progress` = `max(route_a_mean, route_b_mean)` and
`matched ⟺ either route saturates` — max, not a pooled mean, is the honest "how close"
reading for an OR-gated axis, and preserves the engine's pre-existing either-route
recognition semantics. The load-bearing invariants (value ∈ [0,1], matched ⟺
progress == 1.0) hold for all five axes.
The recognized pattern is the FIRST matched axis in the existing spec-070 priority order
(RED_OGV → FRAGMENTED_COLLAPSE → ECOLOGICAL_COLLAPSE → FASCIST_CONSOLIDATION →
REVOLUTIONARY_VICTORY); `pattern_since_tick` is set when `recognized_pattern` changes
value (including to `None`). Gate-ratio conventions:

- "value must reach threshold T" gates: `clamp01(value / T)`.
- "value must stay at/below floor F" gates: `1.0 if value <= F else clamp01(F / value)`.
- binary gates (slope sign, window populated, stance majority): `1.0` or `0.0`.
- counter gates (sustained/duration): `clamp01(count / required)`.
- fascist axis: `clamp01(fascist_fraction / defines.endgame.fascist_majority_fraction)`
  where `fascist_fraction = fascist_node_count / max(1, total_ideology_bearing_nodes)`.

`on_tick` builds ONE `graph = new_state.to_graph()` and threads it through all five axis
evaluators (today each `_check_*` re-serializes — up to ~6 `to_graph()` calls/tick, a
real cost at ~1100 territories).

- [ ] **Step 1: Rework the detector unit tests (red)** — FOLLOW-PATTERN
  (`tests/unit/engine/test_endgame_detector.py`: reuse its WorldState fixture builders;
  outcomes are fixture vehicles, never asserted narrative subjects — ADR074). Replace
  termination asserts with recognizer asserts. Normative new cases:

```python
@pytest.mark.unit
def test_recognition_sets_pattern_and_since_tick(...):
    # Fixture drives the fascist axis to matched at tick N.
    assert detector.recognized_pattern is GameOutcome.FASCIST_CONSOLIDATION
    assert detector.pattern_since_tick == N

@pytest.mark.unit
def test_pattern_dissolves_when_conditions_recede(...):
    # Same fixture, then consciousness recovers below the fraction.
    assert detector.recognized_pattern is None
    assert detector.pattern_since_tick is None

@pytest.mark.unit
def test_fascist_axis_uses_fraction_not_count(...):
    # 6 ideology-bearing nodes, 4 fascist (0.667) => NOT matched at fraction 0.75;
    # flip a 5th (0.833) => matched.

@pytest.mark.unit
def test_axis_progress_keys_and_bounds(...):
    progress = detector.axis_progress()
    assert set(progress) == {
        "revolutionary_victory", "ecological_collapse", "fascist_consolidation",
        "red_ogv", "fragmented_collapse",
    }
    assert all(0.0 <= v <= 1.0 for v in progress.values())

@pytest.mark.unit
def test_matched_iff_progress_saturates(...):
    # Whenever recognized_pattern == P, axis_progress()[P.value] == 1.0.

@pytest.mark.unit
def test_on_tick_serializes_graph_once(monkeypatch, ...):
    # Count WorldState.to_graph calls during one on_tick: exactly 1.
```

- [ ] **Step 2: Run to verify red**

Run: `mise run test:q -- tests/unit/engine/test_endgame_detector.py`
Expected: FAIL — `AttributeError: 'EndgameDetector' object has no attribute
'recognized_pattern'` etc.

- [ ] **Step 3: Implement the recognizer rework**

In `endgame_detector.py`:
1. Constructor: keep signature `EndgameDetector(defines=...)`; read
   `self._balkanization = defines.balkanization` (delete the direct
   `BalkanizationDefines()` construction at ~line 104).
2. Replace `_outcome: GameOutcome | None` bookkeeping with
   `_recognized: GameOutcome | None = None`, `_since_tick: int | None = None`,
   `_last_progress: dict[str, float]` (initialized to the five keys at 0.0).
3. Refactor each `_check_<axis>(state) -> bool` into
   `_axis_<axis>(state, graph) -> tuple[float, bool]` per the gate-ratio conventions
   above (keep every existing predicate term; only the fascist absolute count changes,
   to the fraction). Keep the cross-tick counters (`_overshoot_consecutive_ticks`,
   `_fragmented_consecutive_ticks`, `_habitability_history`) exactly as they update
   today.
4. New `on_tick`: update habitability window; `graph = new_state.to_graph()` once;
   evaluate all five axes; store `_last_progress`; pick first matched in priority
   order; if the pick differs from `_recognized`, set `_recognized` and
   `_since_tick = new_state.tick` (or `None`/`None` when nothing matches).
5. Public API: `recognized_pattern` / `pattern_since_tick` properties;
   `axis_progress()` returns `dict(self._last_progress)`.
6. DELETE `is_game_over`, `outcome`, `_emit_endgame_event` and the early-return
   `if self.is_game_over: return`.
7. In `endgame.py` defines: DELETE `fascist_majority_threshold`; regenerate
   `defines.yaml` (`poetry run python tools/generate_defines_config.py`).

- [ ] **Step 4: Mechanically adapt callers (same behavior this task)**

- `web/game/engine_bridge.py` resolve_tick (~4643-4654): replace
  `if not detector.is_game_over: detector.on_tick(...)` / `if detector.is_game_over:`
  with `previous = detector.recognized_pattern; detector.on_tick(state, new_state);
  game_over = detector.recognized_pattern is not None` and
  `outcome = detector.recognized_pattern` in the EndgameEvent/snapshot block
  (~4777-4782). (Task 4 replaces this recognition-ends-game behavior with the horizon —
  this step only keeps the build green and the tests honest.)
- `src/babylon/engine/headless_runner/argparse_cli.py`: the `--endgame-detector` poll
  halts when `detector.recognized_pattern is not None` (early-halt instrumentation
  semantics preserved; update the flag help text to say "halts on pattern recognition").
- Fix any remaining `is_game_over`/`outcome` references:
  `rg -n 'is_game_over|\.outcome' src/ web/ tests/ | rg -i endgame`

- [ ] **Step 5: Run the detector + adapted suites**

Run: `mise run test:q -- tests/unit/engine/test_endgame_detector.py tests/unit/web/test_endgame_wiring.py tests/unit/web/test_endgame_priority.py tests/scenarios/test_endgame_flow.py`
Expected: PASS. (test_endgame_wiring.py's autouse `_clear_endgame_detector_cache`
fixture is mandatory for anything touching `_session_endgame_detectors`.)

- [ ] **Step 6: Full fast gate + regression**

Run: `mise run check` then `mise run qa:regression`
Expected: check green; regression `5 passed` (baselines run no detector; defines_hash
WARNING from the deleted field is advisory).

- [ ] **Step 7: Commit**

```bash
git add src/babylon/engine/observers/endgame_detector.py src/babylon/config/defines/endgame.py \
  src/babylon/data/defines.yaml web/game/engine_bridge.py src/babylon/engine/headless_runner/argparse_cli.py \
  tests/unit/engine/test_endgame_detector.py tests/scenarios/test_endgame_flow.py \
  tests/integration/test_endgame_detection_round_trip.py
git commit -m "refactor(engine): EndgameDetector -> pattern recognizer with real axis progress (spec-116 FR-116-1)"
```

---

### Task 4: Bridge — fixed-horizon game-over, PATTERN_SHIFT, endgame_progress payload, real objectives

**Files:**
- Modify: `src/babylon/models/enums/events.py` (EventType ~line 30: add
  `PATTERN_SHIFT = "pattern_shift"`; GameOutcome ~line 170: add
  `UNRESOLVED = "unresolved"`)
- Create: `PatternShiftEvent` beside `EndgameEvent` (FOLLOW-PATTERN:
  `src/babylon/models/events/_legacy.py` — same frozen-model shape as EndgameEvent, with
  fields `pattern: str | None`, `previous: str | None`, `since_tick: int`)
- Modify: `web/game/engine_bridge.py` (resolve_tick ~4643-4654 + ~4777-4782;
  get_journal_objectives ~3894-3940; `_EVENT_SEVERITY` map: `pattern_shift: "warning"`)
- Modify: `web/game/stub_bridge.py` (payload parity for `endgame_progress`)
- Modify: `src/babylon/sentinels/seam/registry.py` (SeamEntry rows for the new wire keys)
- Modify: `src/frontend/src/types/game.ts` (add `EndgameProgress` interface to the
  snapshot type). Do NOT touch `eventClassifier.ts` here — the frontend classifier
  entry for `pattern_shift` (`important` / category `system`) is owned by Task 7;
  `"warning"` is not a member of the frontend `EventSeverity` union.
- Test: `tests/unit/web/test_endgame_wiring.py` (extend), `tests/unit/web/test_engine_bridge.py`

**Interfaces:**
- Consumes: Task 3's recognizer API; Task 1's horizon defines.
- Produces:
  - `snapshot["endgame_progress"] = {"axes": {<5 axis keys>: float}, "pattern": str | None,
    "since_tick": int | None, "horizon_tick": int, "locked": bool}` on EVERY tick
    (`locked = pattern is not None and (tick - since_tick + 1) >= defines.endgame.pattern_lock_ticks`).
  - `snapshot["endgame"]` ONLY at game over, with
    `{"outcome": <pattern-or-"unresolved">, "tick": int, "summary": ""}`.
  - A `PATTERN_SHIFT` event in `new_state.events` exactly when the recognized pattern
    changes (including dissolving to `None`).
  - `get_journal_objectives` progress values = the persisted `endgame_progress.axes`
    from the latest snapshot (the proxy math — `min(1.0, principal_gap)` etc. — is
    deleted). Survives worker restarts because it reads the snapshot, not the
    in-process detector cache.
  - Game over iff `new_state.tick >= horizon_tick` (accept-outcome comes in Task 5);
    `outcome = recognized_pattern or GameOutcome.UNRESOLVED`.

- [ ] **Step 1: Write the failing bridge tests** — FOLLOW-PATTERN
  (`tests/unit/web/test_endgame_wiring.py`: real detector through consecutive
  resolve_tick calls, MagicMock persistence, autouse `_clear_endgame_detector_cache`).
  Normative cases:

```python
@pytest.mark.unit
def test_recognition_does_not_end_game(...):
    # Drive a fixture into a recognized pattern; resolve one more tick.
    assert "endgame" not in snapshot
    assert snapshot["endgame_progress"]["pattern"] == "fascist_consolidation"

@pytest.mark.unit
def test_pattern_shift_event_fires_exactly_on_change(...):
    # Tick of recognition: one PATTERN_SHIFT event; next tick (same pattern): none.

@pytest.mark.unit
def test_horizon_ends_game_unresolved(...):
    # defines override: campaign_horizon_years=1 (=> horizon 52); fixture holds no
    # pattern; resolve to tick >= 52.
    assert snapshot["endgame"]["outcome"] == "unresolved"

@pytest.mark.unit
def test_endgame_progress_every_tick_with_lock(...):
    # pattern held pattern_lock_ticks ticks => locked True; before that False.

@pytest.mark.unit
def test_objectives_read_snapshot_progress(...):
    # get_journal_objectives returns the same axes values as the latest snapshot.
```

- [ ] **Step 2: Run to verify red**

Run: `mise run test:q -- tests/unit/web/test_endgame_wiring.py`
Expected: FAIL — `KeyError: 'endgame_progress'` / missing PATTERN_SHIFT.

- [ ] **Step 3: Implement enums + event model**

`events.py`: add `PATTERN_SHIFT = "pattern_shift"` to EventType and
`UNRESOLVED = "unresolved"` to GameOutcome. Add `PatternShiftEvent` beside
`EndgameEvent` (same file/idiom as EndgameEvent; frozen; `event_type` fixed to
`EventType.PATTERN_SHIFT`).

- [ ] **Step 4: Implement the resolve_tick block**

Replace the Task-3 interim block in `engine_bridge.py` with:

```python
        detector = _session_endgame_detectors.get(session_id)
        if detector is None:
            detector = EndgameDetector(defines=game_defines)
            detector.on_simulation_start(state, sim_config)
            _session_endgame_detectors[session_id] = detector
        previous_pattern = detector.recognized_pattern
        detector.on_tick(state, new_state)
        pattern = detector.recognized_pattern
        if pattern is not previous_pattern:
            shift = PatternShiftEvent(
                tick=new_state.tick,
                pattern=pattern.value if pattern else None,
                previous=previous_pattern.value if previous_pattern else None,
                since_tick=detector.pattern_since_tick or new_state.tick,
            )
            new_state = new_state.model_copy(
                update={"events": [*new_state.events, shift]}
            )
        horizon_tick = (
            game_defines.endgame.campaign_horizon_years
            * game_defines.timescale.weeks_per_year
        )
        game_over = new_state.tick >= horizon_tick
        if game_over:
            outcome = pattern or GameOutcome.UNRESOLVED
            endgame_event = EndgameEvent(tick=new_state.tick, outcome=outcome)
            new_state = new_state.model_copy(
                update={"events": [*new_state.events, endgame_event]}
            )
```

and at the snapshot block (~4777):

```python
        since = detector.pattern_since_tick
        snapshot["endgame_progress"] = {
            "axes": detector.axis_progress(),
            "pattern": pattern.value if pattern else None,
            "since_tick": since,
            "horizon_tick": horizon_tick,
            "locked": (
                pattern is not None
                and since is not None
                and (new_state.tick - since + 1)
                >= game_defines.endgame.pattern_lock_ticks
            ),
        }
        if game_over:
            snapshot["endgame"] = {
                "outcome": (pattern or GameOutcome.UNRESOLVED).value,
                "tick": new_state.tick,
                "summary": "",
            }
```

- [ ] **Step 5: Replace get_journal_objectives proxy math**

The five objectives' `progress` values come from the latest persisted snapshot's
`endgame_progress["axes"]` (fetch alongside the existing snapshot read in that method;
honest `0.0` with `"status": "unknown"` only if no snapshot exists yet). Delete the
`principal_gap`/`consciousness_avg`/`heat_avg` proxy lines. Map objective ids to axis
keys 1:1 (`revolution → revolutionary_victory`, `ecological_collapse`,
`fascist_consolidation`, `red_ogv`, `fragmented_collapse`).

- [ ] **Step 6: Severity, classifier, types, seams, stub parity**

- `_EVENT_SEVERITY` (engine_bridge.py ~6786): add `"pattern_shift": "warning"` — a
  STRING-literal key, matching the map's `dict[str, str]` declaration and every
  existing entry (never an `EventType` member key). Backend/wire vocabulary only —
  the frontend classifier entry is Task 7's, which maps `pattern_shift` to
  `important` / category `system`; do not add it here.
- `types/game.ts`: `interface EndgameProgress { axes: Record<string, number>;
  pattern: string | null; since_tick: number | null; horizon_tick: number;
  locked: boolean; }` and add `endgame_progress?: EndgameProgress` to the snapshot
  payload type.
- `src/babylon/sentinels/seam/registry.py`: SeamEntry rows for wire keys
  `endgame_progress` (+ the event wire key `pattern_shift` if events are registered
  individually — FOLLOW-PATTERN: copy how `market_correction` was registered by
  Program 23).
- `stub_bridge.py`: stub snapshot gains a static-but-shape-true `endgame_progress`
  block (all axes 0.0, pattern null, locked false, horizon_tick 5200).

- [ ] **Step 7: Run green + gates**

Run: `mise run test:q -- tests/unit/web/test_endgame_wiring.py tests/unit/web/test_engine_bridge.py`
Expected: PASS.
Run: `mise run check`
Expected: green (check:seams passes with the new rows).

- [ ] **Step 8: Commit**

```bash
git add src/babylon/models/enums/events.py src/babylon/models/events/ web/game/engine_bridge.py \
  web/game/stub_bridge.py src/babylon/sentinels/seam/registry.py \
  src/frontend/src/types/game.ts tests/unit/web/
git commit -m "feat(bridge): fixed-horizon game-over, PATTERN_SHIFT events, per-tick endgame_progress, real objectives (spec-116 FR-116-1)"
```

---

### Task 5: Accept-outcome — the mercy affordance (FR-116-5)

**Files:**
- Modify: `web/game/engine_bridge.py` (new `accept_outcome(session_id)` method beside
  `get_endgame_state` ~3813)
- Modify: `web/game/api.py` (new view beside `game_pause` ~349) + `web/game/urls.py`
- Modify: `src/frontend/src/api/endpoints.ts` (route entry),
  `src/frontend/src/store/slices/worldSlice.ts` (accept action),
  `src/frontend/src/components/objectives/ObjectivesTray.tsx` (button)
- Test: `tests/unit/web/test_accept_outcome.py` (new; FOLLOW-PATTERN
  `tests/unit/web/test_endgame_wiring.py`), `src/frontend/src/store/slices/worldSlice.test.ts` (extend)

**Interfaces:**
- Consumes: Task 4's `endgame_progress.locked` + durable ENDGAME event persistence path.
- Produces: `POST /api/games/{id}/accept-outcome/` → 200 with the standard envelope
  around `{"outcome": str, "tick": int, "accepted": true}`; 409-style `_error` when not
  locked. Frontend testid `accept-outcome` button, visible iff `locked`.

- [ ] **Step 1: Failing bridge test**

```python
@pytest.mark.unit
def test_accept_outcome_requires_lock(...):
    # No pattern locked -> ValueError("outcome not locked").
@pytest.mark.unit
def test_accept_outcome_stamps_durable_endgame(...):
    # Locked fascist pattern at tick T -> accept_outcome persists the ENDGAME tick_event
    # row with outcome fascist_consolidation and payload {"accepted_at_tick": T};
    # get_endgame_state now returns that outcome.
```

- [ ] **Step 2: Red run**

Run: `mise run test:q -- tests/unit/web/test_accept_outcome.py`
Expected: FAIL — `AttributeError: ... no attribute 'accept_outcome'`.

- [ ] **Step 3: Implement bridge method**

`accept_outcome(self, session_id)` reads the latest snapshot's `endgame_progress`;
raises `ValueError("outcome not locked")` unless `locked`; persists the ENDGAME event
through the same tick_event path `resolve_tick` uses (FOLLOW-PATTERN: the persistence
call feeding `_fetch_endgame_event_row`, engine_bridge.py:561-593) with
`outcome = pattern` and payload `{"accepted_at_tick": tick}`; returns
`{"outcome": pattern.value, "tick": tick, "accepted": True}` — `.value`, never the
raw `GameOutcome` enum, so the JSON contract (`outcome: str`) holds (same rule as
Task 4's `(pattern or GameOutcome.UNRESOLVED).value`). Stub parity: `stub_bridge.py`
raises the same ValueError (stub never locks).

- [ ] **Step 4: API view + url**

FOLLOW-PATTERN (`game_pause` at api.py:349-357): `accept_outcome` view POST-only,
IsAuthenticated, `_get_session_or_none`, translate `ValueError` to `_error(...)`,
envelope on success; url `games/<str:game_id>/accept-outcome/` in urls.py beside
pause/resume (`<str:>`, matching every sibling lifecycle route — game-pause,
game-resume, game-recover all use `<str:game_id>`; the view signature takes
`game_id: str`).

- [ ] **Step 5: Green run (backend)**

Run: `mise run test:q -- tests/unit/web/test_accept_outcome.py tests/unit/web/test_api.py`
Expected: PASS.

- [ ] **Step 6: Frontend affordance (red then green)**

Extend `worldSlice.test.ts` (FOLLOW-PATTERN: its requestLog/mock-handler idiom): an
`acceptOutcome(gameId)` store action POSTs the endpoint then refetches the endgame
panel; ObjectivesTray renders a `data-testid="accept-outcome"` button ONLY when
`endgame_progress.locked` ("ACCEPT THIS OUTCOME — end the campaign"); clicking calls
the action; the existing worldSlice outcome watcher then opens the chronicle takeover
(no new watcher). Endpoint entry in `endpoints.ts` typed as
`{ outcome: string; tick: number; accepted: boolean }`.

Run: `cd src/frontend && npx vitest run src/store/slices/worldSlice.test.ts src/components/objectives`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add web/game/engine_bridge.py web/game/stub_bridge.py web/game/api.py web/game/urls.py \
  tests/unit/web/test_accept_outcome.py src/frontend/src/api/endpoints.ts \
  src/frontend/src/store/slices/worldSlice.ts src/frontend/src/store/slices/worldSlice.test.ts \
  src/frontend/src/components/objectives/ObjectivesTray.tsx
git commit -m "feat(game): accept-outcome mercy affordance when a pattern locks (spec-116 FR-116-5)"
```

---

### Task 6: Pacing instrument + declared ceremony #1

**Files:**
- Create: `web/game/management/commands/pacing_probe.py` (FOLLOW-PATTERN:
  `web/game/management/commands/seed_initial_game.py` for command shape)
- Modify: `.mise.toml` (task `sim:pacing`)
- Test: `tests/unit/web/test_pacing_probe.py` (new)
- Ceremony (values only): `src/babylon/config/defines/*` via regeneration, possibly
  `src/babylon/engine/scenarios/_legacy.py` seed ideologies; `tests/baselines/*.json` +
  `tests/baselines/dense/*.csv` regenerated
- Report artifact: `reports/pacing-calibration-2026-07-17.md` (instrument output + drift table)

**Interfaces:**
- Consumes: `_build_initial_state_for_scenario` (engine_bridge.py:5814, includes
  balkanization seeding), `step(state, sim_config, persistent_context, defines)`
  (FOLLOW-PATTERN: `tools/regression_test.py:_run_scenario_ticks` 565-600), Task 3's
  recognizer.
- Produces: `python manage.py pacing_probe --scenario us --ticks 5200 --seed 0
  --report <path.json>` → JSON report
  `{"scenario", "ticks_completed", "first_recognition": {axis: tick|null},
  "final_pattern", "axis_curves": {axis: [[tick, value], ...]},  # every 26 ticks
  "event_type_counts": {type: count}}`. No DB writes; fully in-memory; deterministic
  (no wall-clock in the report body).

- [ ] **Step 1: Failing smoke test**

```python
@pytest.mark.unit
def test_pacing_probe_produces_report(tmp_path) -> None:
    from django.core.management import call_command
    report = tmp_path / "r.json"
    call_command("pacing_probe", scenario="two_node", ticks=3, seed=0, report=str(report))
    data = json.loads(report.read_text())
    assert data["ticks_completed"] == 3
    assert set(data["first_recognition"]) == {
        "revolutionary_victory", "ecological_collapse", "fascist_consolidation",
        "red_ogv", "fragmented_collapse",
    }
    assert data["event_type_counts"]  # non-empty histogram
```

- [ ] **Step 2: Red run**

Run: `mise run test:q -- tests/unit/web/test_pacing_probe.py`
Expected: FAIL — `CommandError: Unknown command: 'pacing_probe'`.

- [ ] **Step 3: Implement the command**

RST module docstring with Usage block; argparse args `--scenario` (default `us`),
`--ticks` (int, default 5200, argparse-validated `1 <= ticks <= 10000` — the loop bound
is the validated arg, statically bounded), `--seed` (default 0), `--report` (path).
Module constants (top of the command file, after imports):

```python
# The five recognizer axes — must equal EndgameDetector.axis_progress()'s key set
# (pinned by the Step 1 test's set-equality assertion).
AXES: tuple[str, ...] = (
    "revolutionary_victory", "ecological_collapse", "fascist_consolidation",
    "red_ogv", "fragmented_collapse",
)
SAMPLE_EVERY: int = 26  # curve sampling cadence in ticks (half a year); a reporting
                        # choice — numerically equal to pattern_lock_ticks by
                        # coincidence, deliberately NOT read from defines
```

Body (no DB access):

```python
        state = _build_initial_state_for_scenario(options["scenario"])
        defines = GameDefines.load_default()
        sim_config = SimulationConfig(rng_seed=options["seed"])
        detector = EndgameDetector(defines=defines)
        detector.on_simulation_start(state, sim_config)
        persistent: dict[str, Any] = {}
        first_recognition: dict[str, int | None] = {k: None for k in AXES}
        curves: dict[str, list[list[float]]] = {k: [] for k in AXES}
        event_counts: Counter[str] = Counter()
        for tick in range(1, options["ticks"] + 1):
            previous = state
            state = step(state, sim_config, persistent, defines)
            detector.on_tick(previous, state)
            for event in state.events:
                event_counts[str(event.event_type)] += 1
            progress = detector.axis_progress()
            pattern = detector.recognized_pattern
            if pattern is not None and first_recognition[pattern.value] is None:
                first_recognition[pattern.value] = tick
            if tick % SAMPLE_EVERY == 0 or tick == options["ticks"]:
                for axis, value in progress.items():
                    curves[axis].append([tick, round(value, 4)])
```

then write the JSON report + a stdout summary table. mise task:

```toml
[tasks."sim:pacing"]
description = "Headless null-play pacing probe on the web scenario path (spec-116 ceremony instrument)"
run = "cd web && poetry run python manage.py pacing_probe"
```

- [ ] **Step 4: Green run**

Run: `mise run test:q -- tests/unit/web/test_pacing_probe.py`
Expected: PASS.

- [ ] **Step 5: Commit the instrument (pre-ceremony)**

```bash
git add web/game/management/commands/pacing_probe.py tests/unit/web/test_pacing_probe.py .mise.toml
git commit -m "feat(tools): pacing_probe — headless null-play instrument on the web scenario path (spec-116)"
```

- [ ] **Step 6: Timing + calibration runs (instrument-first)**

1. `mise run sim:pacing -- --scenario us --ticks 50 --report /tmp/pacing-us-50.json` —
   record wall-clock; extrapolate the full-horizon cost before running it.
2. Full null-play runs: `--scenario us --ticks 5200` and
   `--scenario wayne_county --ticks 5200` (single-flight; expect minutes-to-tens-of-minutes).
3. Read the reports. Calibration targets (write results into
   `reports/pacing-calibration-2026-07-17.md`):
   - `first_recognition[*] > 520` for every axis under null play on `us` (no pattern in
     the first 10 years);
   - at least one axis crosses 0.5 progress by tick 2600 (tension exists);
   - `ticks_completed == 5200` (no crash);
   - event histogram: no type fires every tick.

- [ ] **Step 7: The declared ceremony (ONE commit)**

Tune ONLY defines values (`fascist_majority_fraction`, `endgame.*`, `balkanization.*`
thresholds — via the pydantic defaults + `poetry run python tools/generate_defines_config.py`)
and, if the fascist axis still saturates at seed, the `_legacy.py` seed ideologies
(NOTE: this moves qa baselines — that is what the ceremony is for). Loop: adjust →
re-run the probe → check targets. When targets hold:

```bash
mise run qa:regression                 # see exactly which scenarios drift
mise run qa:regression-generate        # regenerate JSON checkpoints
mise run qa:regression-generate-dense  # regenerate golden dense CSVs
mise run test:q -- tests/unit/config tests/unit/engine/test_endgame_detector.py
git add src/babylon/config/defines/ src/babylon/data/defines.yaml \
  src/babylon/engine/scenarios/_legacy.py tests/baselines/ reports/pacing-calibration-2026-07-17.md
git commit -m "feat(pacing): ceremony #1 — null-play horizon calibration; baselines regenerated

Per-scenario drift (dense CSVs): <scenario>: <summary of what moved and why>
Calibration report: reports/pacing-calibration-2026-07-17.md"
```

The commit body MUST name each drifted scenario and the cause (Market Scissors
promotion pattern). If a scenario drifts that the tuned values cannot explain, STOP —
that is an unintended engine change, not calibration.

- [ ] **Step 8: Post-ceremony gate**

Run: `mise run check && mise run qa:regression`
Expected: both green (regression now compares against the regenerated baselines).

---
## Cluster: Event salience (FR-116-2) — Tasks 7–9

**Scope note (interface ledger):** dedup + severity re-tier + autopause-once are
**frontend-side**. The bridge payload stays a plain per-tick event list; the engine is never
touched. Zero new wire keys → zero new SeamEntry rows; zero new GameDefines fields;
`qa:regression` is untouched by construction (no engine/bridge/defines change in any of these
three tasks). Nothing here terminates a session — autopause pauses, never ends (owner ruling).

**Shared design (all three tasks):**

- The stable identity for "the same thing still happening" is `(event_type, subject)`. Every
  id that exists today is tick-scoped (backend UUID5 hashes the tick; frontend ids are
  positional `${tick}-${index}`; journal ids are `${game_id}-${tick}-${serial}`), so the key
  is **newly derived frontend-side** in a pure lib: `dedupKey(event) =
  "${event.type}:${eventSubject(event)}"`, with `eventSubject` reading a fixed payload-field
  precedence chain, **graph-independent fields first** (`node_id` before `territory_id` —
  the bridge's uprising `territory_id` enrichment differs between graph-present and
  graph-absent serialization paths, `web/game/engine_bridge.py:6905-6910`).
- Severity re-tier happens in the frontend `EVENT_SEVERITY_MAP`
  (`src/frontend/src/lib/eventClassifier.ts:44`): **crimson (`critical`) = `endgame_reached`
  only**; the nine former criticals demote to `important` (gold); `pattern_shift` (new
  Cluster-A EventType) enters at `important`; everything else keeps its tier (political churn
  already sits at gold/muted). The stream layer (`toStreamSeverity`) and every crimson border/
  throb follow automatically because they derive from this map.
- Autopause-once state extends the event-tray-mutes machinery: a session-scoped in-memory
  string list on `eventsSlice` (same lifecycle as `mutedCategories` — reset on page load,
  which is the ledger's "per session"). `endgame_reached` is exempt from session-muting: it
  acknowledges per-occurrence (`key@tick`), so it always autopauses on a new occurrence but
  still cannot double-fire in the load race.

---

### Task 7: Salience pure lib — severity re-tier + `eventDedup`

**Files:**

- Create: `src/frontend/src/lib/eventDedup.ts`
- Create: `src/frontend/src/lib/__tests__/eventDedup.test.ts`
- Modify: `src/frontend/src/lib/eventClassifier.ts:44-132` (EVENT_SEVERITY_MAP re-tier),
  `:230-326` (CATEGORY_MAP + `pattern_shift`)
- Modify (test expectations for the re-tier): `src/frontend/src/lib/__tests__/eventClassifier.test.ts:27-56, 95-136`
- Modify (mechanical critical-driver swaps so the whole suite stays green in this commit):
  `src/frontend/src/store/slices/worldSlice.test.ts:143`,
  `src/frontend/src/store/slices/timeSlice.test.ts:113,179,388`,
  `src/frontend/src/store/slices/eventsSlice.test.ts:25-39`,
  `src/frontend/src/components/chrome/EventToasts.test.tsx:27-35,64-72,95,108,128-140`,
  `src/frontend/src/components/events/EventsFeed.test.tsx:111-127`

**Interfaces:**

- Consumes: `GameEvent`, `ClassifiedEvent` (`src/frontend/src/types/game.ts:393,631`);
  `classifyEvent`/`classifyEvents` (existing).
- Produces (Tasks 8/9 rely on these exact names):
  - `eventSubject(event: GameEvent): string`
  - `dedupKey(event: GameEvent): string` — `` `${type}:${subject}` ``
  - `interface DedupableItem { event: GameEvent; tick: number }`
  - `interface DedupedRun<T extends DedupableItem> { key: string; representative: T; events: T[]; count: number; firstTick: number; lastTick: number }`
  - `dedupeEvents<T extends DedupableItem>(items: readonly T[]): DedupedRun<T>[]`
  - `ALWAYS_AUTOPAUSE_TYPES: ReadonlySet<string>` (= `{"endgame_reached"}`)
  - `interface AutopauseDecision { firingKeys: string[]; acknowledgementKeys: string[] }`
  - `computeAutopauseDecision(criticalEvents: readonly GameEvent[], acknowledged: ReadonlySet<string>): AutopauseDecision`
  - Re-tiered `EVENT_SEVERITY_MAP`: `critical` ⇢ `endgame_reached` only; `pattern_shift: "important"`.

- [ ] **Step 1: Write the failing tests** — create
      `src/frontend/src/lib/__tests__/eventDedup.test.ts`:

```ts
/**
 * Tests for eventDedup (spec-116 FR-116-2) — tick-independent salience
 * identity, consecutive-run collapse, and the autopause-once core.
 *
 * The property-style suites use a seeded deterministic PRNG (mulberry32)
 * with FIXED iteration bounds (Power-of-10 rule 2: every loop statically
 * bounded) rather than a hypothesis-style framework; the failing seed is
 * carried in each assertion message.
 */

import { describe, it, expect } from "vitest";
import {
  ALWAYS_AUTOPAUSE_TYPES,
  computeAutopauseDecision,
  dedupKey,
  dedupeEvents,
  eventSubject,
} from "@/lib/eventDedup";
import { classifyEvents } from "@/lib/eventClassifier";
import type { GameEvent } from "@/types/game";

function makeEvent(type: string, overrides: Partial<GameEvent> = {}): GameEvent {
  return {
    id: "test-event",
    type,
    tick: 1,
    severity: "informational",
    title: "Test",
    body: "",
    data: {},
    ...overrides,
  };
}

describe("eventSubject / dedupKey — tick-independent salience identity", () => {
  it("prefers node_id over the bridge-enriched territory_id (graph-independence)", () => {
    const e = makeEvent("uprising", { data: { node_id: "class-42", territory_id: "26163" } });
    expect(eventSubject(e)).toBe("class-42");
    expect(dedupKey(e)).toBe("uprising:class-42");
  });

  it("keys org events on org_id", () => {
    const e = makeEvent("state_repression", { data: { org_id: "org-maga" } });
    expect(dedupKey(e)).toBe("state_repression:org-maga");
  });

  it("stringifies numeric subjects (fips)", () => {
    const e = makeEvent("dispossession_cascade", { data: { fips: 26163 } });
    expect(dedupKey(e)).toBe("dispossession_cascade:26163");
  });

  it("falls back to source->target for flow events", () => {
    const e = makeEvent("surplus_extraction", {
      data: { source_id: "periphery-1", target_id: "core-1" },
    });
    expect(dedupKey(e)).toBe("surplus_extraction:periphery-1->core-1");
  });

  it("falls back to 'global' when no subject field is present", () => {
    expect(dedupKey(makeEvent("endgame_reached", { data: { outcome: "red_ogv" } }))).toBe(
      "endgame_reached:global",
    );
  });

  it("is tick-independent: the same (type,subject) on different ticks yields the same key", () => {
    const a = makeEvent("uprising", { tick: 3, data: { node_id: "n1" } });
    const b = makeEvent("uprising", { tick: 9, data: { node_id: "n1" } });
    expect(dedupKey(a)).toBe(dedupKey(b));
  });
});

describe("dedupeEvents — consecutive same-(type,subject) collapse", () => {
  it("collapses a consecutive same-key run into one card with count and first/last tick", () => {
    const cards = dedupeEvents(
      classifyEvents([
        makeEvent("dispossession_event", { tick: 4, data: { territory: "26163" } }),
        makeEvent("dispossession_event", { tick: 4, data: { territory: "26163" } }),
        makeEvent("dispossession_event", { tick: 4, data: { territory: "26163" } }),
      ]),
    );
    expect(cards).toHaveLength(1);
    expect(cards[0]).toMatchObject({
      key: "dispossession_event:26163",
      count: 3,
      firstTick: 4,
      lastTick: 4,
    });
    expect(cards[0]!.representative.id).toBe("4-0");
  });

  it("does NOT collapse the same type with different subjects", () => {
    const cards = dedupeEvents(
      classifyEvents([
        makeEvent("dispossession_event", { tick: 4, data: { territory: "26163" } }),
        makeEvent("dispossession_event", { tick: 4, data: { territory: "26099" } }),
      ]),
    );
    expect(cards).toHaveLength(2);
  });

  it("does NOT collapse a non-consecutive repeat (A B A stays three cards)", () => {
    const cards = dedupeEvents(
      classifyEvents([
        makeEvent("uprising", { tick: 4, data: { node_id: "n1" } }),
        makeEvent("state_repression", { tick: 4, data: { org_id: "o1" } }),
        makeEvent("uprising", { tick: 4, data: { node_id: "n1" } }),
      ]),
    );
    expect(cards.map((c) => c.key)).toEqual([
      "uprising:n1",
      "state_repression:o1",
      "uprising:n1",
    ]);
  });
});

describe("computeAutopauseDecision — the autopause-once core", () => {
  const acked = (...keys: string[]): ReadonlySet<string> => new Set(keys);

  it("declares endgame_reached an always-autopause type", () => {
    expect(ALWAYS_AUTOPAUSE_TYPES.has("endgame_reached")).toBe(true);
  });

  it("fires each distinct (type,subject) once, deduping same-tick repeats", () => {
    const events = [
      makeEvent("uprising", { tick: 4, data: { node_id: "n1" } }),
      makeEvent("uprising", { tick: 4, data: { node_id: "n1" } }),
      makeEvent("uprising", { tick: 4, data: { node_id: "n2" } }),
    ];
    const d = computeAutopauseDecision(events, acked());
    expect(d.firingKeys).toEqual(["uprising:n1", "uprising:n2"]);
    expect(d.acknowledgementKeys).toEqual(["uprising:n1", "uprising:n2"]);
  });

  it("suppresses a key that already fired this session", () => {
    const events = [makeEvent("uprising", { tick: 9, data: { node_id: "n1" } })];
    const d = computeAutopauseDecision(events, acked("uprising:n1"));
    expect(d.firingKeys).toEqual([]);
    expect(d.acknowledgementKeys).toEqual([]);
  });

  it("endgame_reached fires per occurrence: same tick suppressed (load race), new tick fires", () => {
    const endgame = makeEvent("endgame_reached", { tick: 52, data: { outcome: "red_ogv" } });
    const first = computeAutopauseDecision([endgame], acked());
    expect(first.firingKeys).toEqual(["endgame_reached:global"]);
    expect(first.acknowledgementKeys).toEqual(["endgame_reached:global@52"]);

    const raced = computeAutopauseDecision([endgame], acked("endgame_reached:global@52"));
    expect(raced.firingKeys).toEqual([]);

    const later = makeEvent("endgame_reached", { tick: 53, data: { outcome: "red_ogv" } });
    const again = computeAutopauseDecision([later], acked("endgame_reached:global@52"));
    expect(again.firingKeys).toEqual(["endgame_reached:global"]);
    expect(again.acknowledgementKeys).toEqual(["endgame_reached:global@53"]);
  });
});

// ---------------------------------------------------------------------------
// Property-style suites — seeded PRNG, fixed bounds.
// ---------------------------------------------------------------------------

function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const GEN_TYPES = ["uprising", "state_repression", "endgame_reached"] as const;
const GEN_SUBJECTS: readonly Record<string, unknown>[] = [
  { node_id: "n1" },
  { node_id: "n2" },
  { org_id: "o1" },
  {},
];

function randomEvents(rand: () => number, count: number, tick: number): GameEvent[] {
  const out: GameEvent[] = [];
  for (let i = 0; i < count; i++) {
    const type = GEN_TYPES[Math.floor(rand() * GEN_TYPES.length)]!;
    const data = GEN_SUBJECTS[Math.floor(rand() * GEN_SUBJECTS.length)]!;
    out.push(makeEvent(type, { tick, data: { ...data } }));
  }
  return out;
}

describe("salience properties (seeded, fixed bounds)", () => {
  const CASES = 100;
  const MAX_EVENTS = 30;
  const MAX_TICKS = 12;

  it("dedupeEvents partitions its input in order with no adjacent equal keys", () => {
    for (let c = 0; c < CASES; c++) {
      const rand = mulberry32(c + 1);
      const n = 1 + Math.floor(rand() * MAX_EVENTS);
      const items = classifyEvents(randomEvents(rand, n, 7));
      const runs = dedupeEvents(items);

      const total = runs.reduce((sum, r) => sum + r.count, 0);
      expect(total, `seed ${c + 1}: counts must sum to input length`).toBe(items.length);
      expect(
        runs.flatMap((r) => r.events),
        `seed ${c + 1}: flattened runs must reproduce the input in order`,
      ).toEqual(items);
      for (let i = 1; i < runs.length; i++) {
        expect(
          runs[i]!.key,
          `seed ${c + 1}: adjacent runs ${i - 1},${i} must differ (acceptance gate 2)`,
        ).not.toBe(runs[i - 1]!.key);
      }
      for (const r of runs) {
        expect(
          r.events.every((e) => dedupKey(e.event) === r.key),
          `seed ${c + 1}: every member of a run shares its key`,
        ).toBe(true);
      }
    }
  });

  it("dedupeEvents is idempotent on collapsed representatives", () => {
    for (let c = 0; c < CASES; c++) {
      const seed = 1000 + c;
      const rand = mulberry32(seed);
      const n = 1 + Math.floor(rand() * MAX_EVENTS);
      const runs = dedupeEvents(classifyEvents(randomEvents(rand, n, 3)));
      const again = dedupeEvents(runs.map((r) => r.representative));
      expect(
        again.map((r) => r.key),
        `seed ${seed}: re-collapsing must preserve the key sequence`,
      ).toEqual(runs.map((r) => r.key));
      expect(
        again.every((r) => r.count === 1),
        `seed ${seed}: an already-collapsed sequence has nothing left to merge`,
      ).toBe(true);
    }
  });

  it("autopause-once: a non-ALWAYS key fires at most once across any tick sequence", () => {
    for (let c = 0; c < CASES; c++) {
      const seed = 2000 + c;
      const rand = mulberry32(seed);
      const acknowledged = new Set<string>();
      const firedPerKey = new Map<string, number>();
      const ticks = 1 + Math.floor(rand() * MAX_TICKS);
      for (let tick = 1; tick <= ticks; tick++) {
        const n = Math.floor(rand() * 6);
        const decision = computeAutopauseDecision(randomEvents(rand, n, tick), acknowledged);
        for (const key of decision.acknowledgementKeys) acknowledged.add(key);
        for (const key of decision.firingKeys) {
          firedPerKey.set(key, (firedPerKey.get(key) ?? 0) + 1);
        }
      }
      for (const [key, fired] of firedPerKey) {
        if (!key.startsWith("endgame_reached:")) {
          expect(fired, `seed ${seed}: key ${key} fired ${fired}×`).toBeLessThanOrEqual(1);
        }
      }
    }
  });
});
```

- [ ] **Step 2: Run to verify it fails**:
      `cd src/frontend && npx vitest run src/lib/__tests__/eventDedup.test.ts`
      — expected: `Error: Failed to resolve import "@/lib/eventDedup"` (module does not exist).

- [ ] **Step 3: Write the implementation** — create `src/frontend/src/lib/eventDedup.ts`:

```ts
/**
 * Event salience (spec-116 FR-116-2) — tick-independent dedup identity,
 * consecutive-run collapse, and the autopause-once core.
 *
 * Frontend-side by design (interface ledger 2026-07-17): the bridge payload
 * stays a plain per-tick event list, and every id the backend serializes is
 * tick-scoped (the UUID5 seed hashes the tick), so a persisting condition
 * gets a NEW id every tick. The stable identity for "the same thing still
 * happening" is `(event_type, subject)`, derived here from payload fields.
 */

import type { GameEvent } from "@/types/game";

/**
 * Subject-field precedence, graph-independent payload fields FIRST:
 * `uprising` events carry both `node_id` (engine payload) and a
 * bridge-enriched `territory_id` that differs between graph-present and
 * graph-absent serialization paths (`engine_bridge._serialize_event`) —
 * keying on `node_id` keeps the identity stable across both.
 */
const SUBJECT_FIELDS: readonly string[] = [
  "node_id",
  "org_id",
  "entity_id",
  "territory_id",
  "territory",
  "fips",
  "county_fips",
  "sovereign_id",
  "faction_id",
  "comprador_id",
  "periphery_id",
  "core_worker_id",
];

/** Resolve an event's subject: first present subject field, else source->target, else "global". */
export function eventSubject(event: GameEvent): string {
  const data = event.data ?? {};
  for (const field of SUBJECT_FIELDS) {
    const value = data[field];
    if (typeof value === "string" && value !== "") return value;
    if (typeof value === "number") return String(value);
  }
  const source = data["source_id"];
  const target = data["target_id"];
  if (typeof source === "string" && typeof target === "string") return `${source}->${target}`;
  if (typeof source === "string") return source;
  return "global";
}

/** Tick-independent salience identity: `${type}:${subject}`. */
export function dedupKey(event: GameEvent): string {
  return `${event.type}:${eventSubject(event)}`;
}

/** Anything carrying a GameEvent + tick can be run-collapsed (ClassifiedEvent, StreamEvent). */
export interface DedupableItem {
  event: GameEvent;
  tick: number;
}

/** One collapsed run of consecutive same-(type,subject) items. */
export interface DedupedRun<T extends DedupableItem> {
  key: string;
  /** FIRST item of the run — carries the run's id/severity/deep-link. */
  representative: T;
  events: T[];
  count: number;
  firstTick: number;
  lastTick: number;
}

/**
 * Collapse CONSECUTIVE same-(type,subject) items into one card each
 * (FR-116-2 i / acceptance gate 2: "no two consecutive identical event
 * cards"). Order-preserving partition; a non-consecutive repeat stays a
 * separate card. Loop bound: `items.length`.
 */
export function dedupeEvents<T extends DedupableItem>(items: readonly T[]): DedupedRun<T>[] {
  const runs: DedupedRun<T>[] = [];
  for (const item of items) {
    const key = dedupKey(item.event);
    const last = runs[runs.length - 1];
    if (last !== undefined && last.key === key) {
      last.events.push(item);
      last.count += 1;
      last.firstTick = Math.min(last.firstTick, item.tick);
      last.lastTick = Math.max(last.lastTick, item.tick);
    } else {
      runs.push({
        key,
        representative: item,
        events: [item],
        count: 1,
        firstTick: item.tick,
        lastTick: item.tick,
      });
    }
  }
  return runs;
}

/**
 * Event types that autopause on EVERY occurrence — never session-muted by
 * the once-per-key rule (interface ledger: "endgame_reached always
 * autopauses"). They acknowledge per-occurrence (`key@tick`), which still
 * suppresses the same-tick double-fire in the load race.
 */
export const ALWAYS_AUTOPAUSE_TYPES: ReadonlySet<string> = new Set(["endgame_reached"]);

/** What one tick's critical events should do to the pause machinery. */
export interface AutopauseDecision {
  /** Dedup keys to pause on — what `time.autopause` receives and the modal joins on. */
  firingKeys: string[];
  /** Session-memory entries to record — `key` normally, `key@tick` for ALWAYS types. */
  acknowledgementKeys: string[];
}

/**
 * The autopause-once core (FR-116-2 iii). PURE: the caller passes ONLY the
 * tick's critical-severity events plus its acknowledged set, and must add
 * the returned `acknowledgementKeys` to that set when it fires the pause.
 * Loop bound: `criticalEvents.length`.
 */
export function computeAutopauseDecision(
  criticalEvents: readonly GameEvent[],
  acknowledged: ReadonlySet<string>,
): AutopauseDecision {
  const firing = new Set<string>();
  const acks: string[] = [];
  const seen = new Set<string>();
  for (const event of criticalEvents) {
    const key = dedupKey(event);
    const ackKey = ALWAYS_AUTOPAUSE_TYPES.has(event.type) ? `${key}@${event.tick}` : key;
    if (seen.has(ackKey) || acknowledged.has(ackKey)) continue;
    seen.add(ackKey);
    acks.push(ackKey);
    firing.add(key);
  }
  return { firingKeys: [...firing], acknowledgementKeys: acks };
}
```

- [ ] **Step 4: Run to verify it passes**:
      `cd src/frontend && npx vitest run src/lib/__tests__/eventDedup.test.ts`
      — expected: `Test Files  1 passed`, all tests green.

- [ ] **Step 5: Write the failing re-tier tests** — edit
      `src/frontend/src/lib/__tests__/eventClassifier.test.ts`:

  1. Replace the `rupture`-critical test (lines 28-31) with:

```ts
  it("classifies a 'rupture' event as important (spec-116 FR-116-2 — crimson is ENDGAME-only)", () => {
    const ce = classifyEvent(makeEvent("rupture"), 0);
    expect(ce.severity).toBe("important");
  });

  it("classifies 'endgame_reached' as critical — the only crimson tier left", () => {
    const ce = classifyEvent(makeEvent("endgame_reached"), 0);
    expect(ce.severity).toBe("critical");
  });

  it("classifies 'pattern_shift' as important (warning tier — never crimson, never autopauses)", () => {
    const ce = classifyEvent(makeEvent("pattern_shift"), 0);
    expect(ce.severity).toBe("important");
  });
```

  2. In the stream describe, replace the critical-tier test (lines 96-101) with:

```ts
  it("maps the sole critical severity (endgame_reached) to the urgent stream and 'critical' tier", () => {
    const se = classifyEventForStream(makeEvent("endgame_reached"), 0);
    expect(se.severity).toBe("critical");
    expect(se.stream).toBe("urgent");
    expect(se.category).toBe("system");
  });

  it("maps a demoted former-critical (rupture) to 'notable'/urgent — gold, not crimson", () => {
    const se = classifyEventForStream(makeEvent("rupture"), 0);
    expect(se.severity).toBe("notable");
    expect(se.stream).toBe("urgent");
    expect(se.category).toBe("struggle");
  });
```

  3. Replace the `sovereign_collapse` stream test (lines 130-135) with:

```ts
  it("classifies a political-family event (sovereign_collapse) as notable/urgent/political", () => {
    const se = classifyEventForStream(makeEvent("sovereign_collapse"), 0);
    expect(se.severity).toBe("notable");
    expect(se.stream).toBe("urgent");
    expect(se.category).toBe("political");
  });
```

- [ ] **Step 6: Run to verify the new expectations fail**:
      `cd src/frontend && npx vitest run src/lib/__tests__/eventClassifier.test.ts`
      — expected: `AssertionError: expected 'critical' to be 'important'` (rupture) and
      `expected 'informational' to be 'important'` (pattern_shift — unknown key falls through).

- [ ] **Step 7: Re-tier the map** — edit `src/frontend/src/lib/eventClassifier.ts`. Replace the
      "Critical" block of `EVENT_SEVERITY_MAP` (lines 45-55) and the head of the "Important"
      block (lines 57-58) with:

```ts
  // Critical — crimson is reserved for the ENDGAME alone (spec-116
  // FR-116-2 salience re-tier; interface ledger 2026-07-17). Everything
  // that used to sit here — rupture, coups, sovereign collapse, the
  // endgame-adjacent pattern events — is real drama but not the horizon:
  // it re-tiers to "important" (gold) so the crimson channel keeps its
  // meaning across a 5200-tick campaign, and autopause (worldSlice fires
  // it on critical severity only) stops interrupting mid-campaign churn.
  endgame_reached: "critical",

  // Important — phase transitions, strategic shifts, and the nine former
  // criticals demoted by the FR-116-2 re-tier. `pattern_shift` (Cluster A,
  // ADR079: the recognizer's pattern changed) enters here — a warning-tier
  // signal, deliberately never crimson and never autopausing.
  rupture: "important",
  terminal_decision: "important",
  control_ratio_crisis: "important",
  civil_war_declared: "important",
  red_brown_coup: "important",
  sovereign_collapse: "important",
  red_ogv_endgame: "important",
  fragmented_collapse_endgame: "important",
  doctrine_trap_sprung: "important",
  pattern_shift: "important",
```

  (The pre-existing `important` entries from `bifurcation_threshold:` down, and the whole
  `informational` block, are untouched. Net diff: `endgame_reached` moves up to a
  one-entry critical block; the other nine ex-criticals move into the important block;
  `pattern_shift` is added.)

  Then add `pattern_shift` to `CATEGORY_MAP`'s system block (after line 325
  `endgame_reached: "system",`):

```ts
  pattern_shift: "system",
```

- [ ] **Step 8: Mechanical critical-driver swaps** (the suite's "a critical event" fixtures used
      `rupture`; post-re-tier the only critical type is `endgame_reached`). Exact edits:

  - `src/frontend/src/store/slices/worldSlice.test.ts:143`:
    `makeEvent({ type: "rupture", tick: 2 })` → `makeEvent({ type: "endgame_reached", tick: 2 })`
    (the `["2-0"]` expectation at :148 is still correct — ids stay positional until Task 9).
  - `src/frontend/src/store/slices/timeSlice.test.ts:113`, `:179`, `:388`: same swap,
    `"rupture"` → `"endgame_reached"` (three `makeEvent` sites; tick args unchanged).
  - `src/frontend/src/store/slices/eventsSlice.test.ts:25-39` — replace the whole
    "pops one persistent toast per critical event" test with:

```ts
  it("pops a persistent toast for a critical event", () => {
    useStore
      .getState()
      .events.ingest(1, [makeEvent({ type: "endgame_reached", tick: 1, id: "e1", data: {} })]);

    const { toasts } = useStore.getState().events;
    expect(toasts).toHaveLength(1);
    expect(toasts[0]!.severity).toBe("critical");
    expect(toasts[0]!.lifetime).toBe("persistent");
    expect(toasts[0]!.events).toHaveLength(1);
  });
```

  - `src/frontend/src/components/chrome/EventToasts.test.tsx`:
    - `:28` (persistent CTA test), `:65` (Open Wire test), `:131` (no-auto-dismiss test):
      `makeEvent({ type: "rupture", tick: 1, id: "e1" })` →
      `makeEvent({ type: "endgame_reached", tick: 1, id: "e1" })` (these three tests require a
      *persistent* toast; the dismiss/mute tests at `:54`/`:75` stay on `rupture` — a notable
      batch toast dismisses and mutes identically, ids are read dynamically).
    - `:95` and `:108` (rupture Mao-score tests): rupture now rides the notable *batch* toast,
      whose toast id (`batch-1`) no longer equals the member event id the score testid uses.
      Replace `const id = useStore.getState().events.toasts[0]!.id;` with
      `const id = useStore.getState().events.toasts[0]!.events[0]!.id;` in BOTH tests.
  - `src/frontend/src/components/events/EventsFeed.test.tsx:111-127` — the
    "critical with no linked entity opens Chronicle" test: replace
    `makeEvent({ id: "e1", type: "rupture", title: "Rupture", tick: 3, data: {} })` with
    `makeEvent({ id: "e1", type: "endgame_reached", title: "The Horizon", tick: 3, data: {} })`
    and the two `screen.getByText("Rupture")` calls with `screen.getByText("The Horizon")`
    (the `autopauseEventIds: ["3-0"]` line is untouched until Task 9).

- [ ] **Step 9: Full frontend gate, then commit**:
      `cd src/frontend && npm run check` — expected: tsc, eslint, prettier clean; vitest
      `Test Files  N passed` (zero failures). Then:

```bash
cd /home/user/projects/game/babylon
git add src/frontend/src/lib/eventDedup.ts \
        src/frontend/src/lib/__tests__/eventDedup.test.ts \
        src/frontend/src/lib/eventClassifier.ts \
        src/frontend/src/lib/__tests__/eventClassifier.test.ts \
        src/frontend/src/store/slices/worldSlice.test.ts \
        src/frontend/src/store/slices/timeSlice.test.ts \
        src/frontend/src/store/slices/eventsSlice.test.ts \
        src/frontend/src/components/chrome/EventToasts.test.tsx \
        src/frontend/src/components/events/EventsFeed.test.tsx
mise run commit -- "feat(frontend): salience re-tier (crimson=ENDGAME only) + tick-independent event dedup lib (spec-116 FR-116-2)"
```

Determinism: frontend-only; no bridge/engine/defines change — `qa:regression` untouched by
construction.

---

### Task 8: Collapsed cards across EventsFeed / EventToasts / EventTray

**Files:**

- Modify: `src/frontend/src/store/slices/eventsSlice.ts:42-120` (ToastEntry shape + ingest
  cross-tick accumulation)
- Modify: `src/frontend/src/components/events/EventsFeed.tsx:54-115` (deduped cards: count
  badge + age)
- Modify: `src/frontend/src/components/chrome/EventToasts.tsx:61-124` (EventLine count prop,
  batch-run collapse, toast count badge)
- Modify: `src/frontend/src/components/chrome/EventTray.tsx:102-125` (accumulated count on
  Missed entries)
- Modify (ToastEntry literal factories gain the new required fields):
  `src/frontend/src/components/map/layers/stormMarkers.test.ts:52-54`,
  `src/frontend/src/components/map/layers/criticalPulse.test.ts:51-53`
- Test: `src/frontend/src/store/slices/eventsSlice.test.ts`,
  `src/frontend/src/components/events/EventsFeed.test.tsx`,
  `src/frontend/src/components/chrome/EventToasts.test.tsx`,
  `src/frontend/src/components/chrome/EventTray.test.tsx`

**Interfaces:**

- Consumes: `dedupeEvents`, `DedupedRun` from Task 7's `@/lib/eventDedup`.
- Produces (Task 9 and the map layers rely on these):
  - `ToastEntry` v2: `{ id: string; dedupKey: string | null; tick: number; lastTick: number;
    count: number; severity: StreamSeverity; lifetime: "persistent" | "ephemeral";
    events: StreamEvent[] }` — `tick` = first occurrence; `dedupKey` non-null iff critical.
  - Render contracts: feed card testid `event-${representative.id}` (unchanged), count badge
    `event-count-${representative.id}`, toast count badge `toast-count-${toast.id}`.

- [ ] **Step 1: Write the failing store tests** — append to
      `src/frontend/src/store/slices/eventsSlice.test.ts`:

```ts
describe("events slice — cross-tick salience dedup (spec-116 FR-116-2)", () => {
  it("collapses same-tick same-(type,subject) criticals into one toast with a count", () => {
    useStore.getState().events.ingest(1, [
      makeEvent({ type: "endgame_reached", tick: 1, id: "e1", data: {} }),
      makeEvent({ type: "endgame_reached", tick: 1, id: "e2", data: {} }),
    ]);

    const { toasts } = useStore.getState().events;
    expect(toasts).toHaveLength(1);
    expect(toasts[0]!.count).toBe(2);
    expect(toasts[0]!.dedupKey).toBe("endgame_reached:global");
  });

  it("a persisting critical on the next tick updates the existing toast instead of stacking", () => {
    useStore.getState().events.ingest(1, [makeEvent({ type: "endgame_reached", tick: 1, data: {} })]);
    useStore.getState().events.ingest(2, [makeEvent({ type: "endgame_reached", tick: 2, data: {} })]);

    const { toasts } = useStore.getState().events;
    expect(toasts).toHaveLength(1);
    expect(toasts[0]!.count).toBe(2);
    expect(toasts[0]!.tick).toBe(1); // first occurrence
    expect(toasts[0]!.lastTick).toBe(2); // still happening
  });

  it("a dismissed critical's key accumulates silently in the tray — never re-pops", () => {
    useStore.getState().events.ingest(1, [makeEvent({ type: "endgame_reached", tick: 1, data: {} })]);
    const id = useStore.getState().events.toasts[0]!.id;
    useStore.getState().events.dismissToast(id);

    useStore.getState().events.ingest(2, [makeEvent({ type: "endgame_reached", tick: 2, data: {} })]);

    expect(useStore.getState().events.toasts).toHaveLength(0);
    expect(useStore.getState().events.tray).toHaveLength(1);
    expect(useStore.getState().events.tray[0]!.count).toBe(2);
    expect(useStore.getState().events.tray[0]!.lastTick).toBe(2);
  });
});
```

- [ ] **Step 2: Run to verify failure**:
      `cd src/frontend && npx vitest run src/store/slices/eventsSlice.test.ts`
      — expected: TS/assert failures — `dedupKey`/`count`/`lastTick` do not exist on
      `ToastEntry`, and the persisting-critical test finds 2 toasts.

- [ ] **Step 3: Implement the slice** — edit `src/frontend/src/store/slices/eventsSlice.ts`.
      Add `import { dedupeEvents } from "@/lib/eventDedup";` after the eventClassifier import.
      Replace the `ToastEntry` interface (lines 42-50) with:

```ts
/**
 * One popped toast: a single critical *condition* (accumulating across
 * ticks by salience key, spec-116 FR-116-2), or a same-tick batch of
 * notable events.
 */
export interface ToastEntry {
  id: string;
  /** Salience identity `${type}:${subject}` for critical toasts; `null` for notable batches. */
  dedupKey: string | null;
  /** Tick of FIRST occurrence. */
  tick: number;
  /** Tick of the most recent occurrence (== tick until the condition persists). */
  lastTick: number;
  /** Raw events this card has absorbed (same-tick repeats + cross-tick recurrences). */
  count: number;
  severity: StreamSeverity;
  /** Persistent-until-acted (critical/decision) vs ephemeral-with-generous-timing (flavor). */
  lifetime: "persistent" | "ephemeral";
  events: StreamEvent[];
}
```

  Replace the whole `ingest` action (lines 81-120) with:

```ts
    ingest: (tick, rawEvents) => {
      if (get().events.ingestedTicks.includes(tick)) return;

      const muted = new Set(get().events.mutedCategories);
      const classified = classifyEventsForStream(rawEvents).filter(
        (e) => e.stream === "urgent" && !muted.has(e.category),
      );

      // Critical conditions collapse by (type,subject): same-tick repeats
      // into one card, and a condition already toasted — or dismissed into
      // the tray — on an earlier tick ACCUMULATES (count/lastTick) instead
      // of stacking a duplicate (FR-116-2 / acceptance gate 2). A dismissed
      // condition stays dismissed: silent accumulation, never a re-pop.
      const toasts = [...get().events.toasts];
      const tray = [...get().events.tray];
      const fresh: ToastEntry[] = [];
      for (const run of dedupeEvents(classified.filter((e) => e.severity === "critical"))) {
        const bump = (t: ToastEntry): ToastEntry => ({
          ...t,
          count: t.count + run.count,
          lastTick: tick,
          events: run.events,
        });
        const activeIdx = toasts.findIndex((t) => t.dedupKey === run.key);
        if (activeIdx >= 0) {
          toasts[activeIdx] = bump(toasts[activeIdx]!);
          continue;
        }
        const trayIdx = tray.findIndex((t) => t.dedupKey === run.key);
        if (trayIdx >= 0) {
          tray[trayIdx] = bump(tray[trayIdx]!);
          continue;
        }
        fresh.push({
          id: run.representative.id,
          dedupKey: run.key,
          tick,
          lastTick: tick,
          count: run.count,
          severity: "critical" as const,
          lifetime: "persistent" as const,
          events: run.events,
        });
      }

      const notable = classified.filter((e) => e.severity === "notable");
      const batchToast: ToastEntry[] =
        notable.length > 0
          ? [
              {
                id: `batch-${tick}`,
                dedupKey: null,
                tick,
                lastTick: tick,
                count: notable.length,
                severity: "notable" as const,
                lifetime: "ephemeral" as const,
                events: notable,
              },
            ]
          : [];

      set((s) => ({
        events: {
          ...s.events,
          ingestedTicks: [...s.events.ingestedTicks, tick],
          toasts: [...toasts, ...fresh, ...batchToast],
          tray,
        },
      }));
    },
```

- [ ] **Step 4: Fix the two ToastEntry literal factories** (compile-breakers, not behavior):
      `src/frontend/src/components/map/layers/stormMarkers.test.ts:52-54` →

```ts
function toast(events: StreamEvent[], severity: ToastEntry["severity"] = "notable"): ToastEntry {
  return {
    id: `t-${severity}`,
    dedupKey: null,
    tick: 5,
    lastTick: 5,
    count: events.length,
    severity,
    lifetime: "ephemeral",
    events,
  };
}
```

  `src/frontend/src/components/map/layers/criticalPulse.test.ts:51-53` →

```ts
function toast(severity: ToastEntry["severity"], events: StreamEvent[]): ToastEntry {
  return {
    id: `t-${severity}`,
    dedupKey: null,
    tick: 5,
    lastTick: 5,
    count: events.length,
    severity,
    lifetime: "persistent",
    events,
  };
}
```

  Re-run: `cd src/frontend && npx vitest run src/store/slices/eventsSlice.test.ts
  src/components/map/layers/stormMarkers.test.ts src/components/map/layers/criticalPulse.test.ts`
  — expected: all pass.

- [ ] **Step 5: Write the failing component tests.**
      Append to `src/frontend/src/components/events/EventsFeed.test.tsx`:

```ts
  it("collapses consecutive same-(type,subject) events into one card with count and age (FR-116-2)", () => {
    useStore.setState((s) => ({
      world: {
        ...s.world,
        snapshot: makeSnapshot({
          events: [
            makeEvent({
              id: "e1",
              type: "dispossession_event",
              title: "Dispossession",
              tick: 5,
              data: { territory: "26163" },
            }),
            makeEvent({
              id: "e2",
              type: "dispossession_event",
              title: "Dispossession",
              tick: 5,
              data: { territory: "26163" },
            }),
            makeEvent({
              id: "e3",
              type: "dispossession_event",
              title: "Dispossession",
              tick: 5,
              data: { territory: "26099" },
            }),
          ],
        }),
      },
    }));
    render(<EventsFeed />);

    // The 26163 run collapses into one card; 26099 stays separate.
    expect(screen.getAllByText("Dispossession")).toHaveLength(2);
    expect(screen.getByTestId("event-count-5-0")).toHaveTextContent("×2");
    expect(screen.queryByTestId("event-count-5-2")).not.toBeInTheDocument();
    // Age label (per-tick feed: single tick).
    expect(screen.getAllByText("t5")).toHaveLength(2);
  });
```

  Append to `src/frontend/src/components/chrome/EventToasts.test.tsx`:

```ts
  it("shows a count badge with the latest tick on an accumulated critical toast", () => {
    useStore.getState().events.ingest(1, [makeEvent({ type: "endgame_reached", tick: 1, data: {} })]);
    useStore.getState().events.ingest(2, [makeEvent({ type: "endgame_reached", tick: 2, data: {} })]);
    render(<EventToasts gameId="game-1" />);

    const id = useStore.getState().events.toasts[0]!.id;
    expect(screen.getByTestId(`toast-count-${id}`)).toHaveTextContent("×2 · through tick 2");
  });
```

  Append to `src/frontend/src/components/chrome/EventTray.test.tsx` (inside the main describe):

```ts
  it("shows the accumulated count on a dismissed critical in the Missed tray", async () => {
    useStore.getState().events.ingest(1, [makeEvent({ type: "endgame_reached", tick: 1, data: {} })]);
    const id = useStore.getState().events.toasts[0]!.id;
    useStore.getState().events.dismissToast(id);
    useStore.getState().events.ingest(2, [makeEvent({ type: "endgame_reached", tick: 2, data: {} })]);

    render(<EventTray gameId={DEFAULT_GAME_ID} />);
    expect(screen.getByTestId(`tray-restore-${id}`)).toHaveTextContent("×2");
  });
```

- [ ] **Step 6: Run to verify failure**:
      `cd src/frontend && npx vitest run src/components/events/EventsFeed.test.tsx
      src/components/chrome/EventToasts.test.tsx src/components/chrome/EventTray.test.tsx`
      — expected: `Unable to find an element by: [data-testid="event-count-5-0"]` /
      `[data-testid="toast-count-…"]` / tray text without `×2`.

- [ ] **Step 7: Implement the renders.**

  `src/frontend/src/components/events/EventsFeed.tsx` — add
  `import { dedupeEvents } from "@/lib/eventDedup";` and replace the `EventsFeed` function
  (lines 54-115) with:

```tsx
export function EventsFeed(): React.JSX.Element {
  const events = useStore((s) => s.world.snapshot?.events);
  const autopauseEventIds = useStore((s) => s.time.autopauseEventIds);
  const setSelection = useStore((s) => s.map.setSelection);
  const openTakeover = useStore((s) => s.ui.openTakeover);

  // Consecutive same-(type,subject) events collapse into one card with a
  // count badge and age (spec-116 FR-116-2 / acceptance gate 2) — the
  // run's FIRST event carries the card's id, severity, and deep-link.
  const cards = dedupeEvents(classifyEvents(events ?? []));

  // The honest empty states carry the same testid as the populated feed —
  // "renders classified events OR the honest empty copy" is one surface
  // (Constitution III.11), and e2e asserts on the container either way.
  // Copy is in-register (DESIGN_BIBLE §7's "purge the admin voice" — "the
  // wire is silent", not "No events loaded yet.").
  if (!events) {
    return (
      <div className="flex flex-col gap-1 p-2" data-testid="events-feed">
        <p className="p-3 text-[11px] italic text-shroud">The wire is silent — no dispatch yet.</p>
      </div>
    );
  }
  if (cards.length === 0) {
    return (
      <div className="flex flex-col gap-1 p-2" data-testid="events-feed">
        <p className="p-3 text-[11px] italic text-shroud">The wire is quiet this tick.</p>
      </div>
    );
  }

  function handleClick(event: ClassifiedEvent): void {
    const kind = inspectorKindForEvent(event);
    if (kind && event.linkedEntityId) {
      setSelection({ kind, id: event.linkedEntityId });
      return;
    }
    if (event.severity === "critical") {
      openTakeover("chronicle");
    }
  }

  return (
    <div className="flex flex-col gap-1 p-2" data-testid="events-feed">
      {cards.map((card) => {
        const rep = card.representative;
        return (
          <button
            key={rep.id}
            onClick={() => handleClick(rep)}
            disabled={!rep.linkedEntityId && rep.severity !== "critical"}
            data-testid={`event-${rep.id}`}
            data-autopause={autopauseEventIds.includes(rep.id) || undefined}
            className="flex items-center gap-2 rounded px-1.5 py-1 text-left hover:bg-rebar disabled:cursor-default disabled:hover:bg-transparent"
          >
            <span className={`text-[10px] ${SEVERITY_COLOR[rep.severity]}`}>●</span>
            <span className="min-w-[90px] font-mono text-[9px] uppercase tracking-widest text-ash">
              {rep.event.type}
            </span>
            <span className="flex-1 truncate text-[11px] text-bone">
              {rep.event.title || rep.event.body || rep.event.type}
            </span>
            {card.count > 1 && (
              <span
                data-testid={`event-count-${rep.id}`}
                className="border border-accent-gold px-1 font-mono text-[9px] text-accent-gold"
              >
                ×{card.count}
              </span>
            )}
            <span className="font-mono text-[9px] text-ksbc-muted-2">
              {card.firstTick === card.lastTick
                ? `t${card.firstTick}`
                : `t${card.firstTick}–${card.lastTick}`}
            </span>
          </button>
        );
      })}
    </div>
  );
}
```

  `src/frontend/src/components/chrome/EventToasts.tsx` — add
  `import { dedupeEvents } from "@/lib/eventDedup";`; give `EventLine` a count prop
  (replace lines 61-80):

```tsx
function EventLine({ event, count = 1 }: { event: StreamEvent; count?: number }): React.JSX.Element {
  // "Headlines lead with actor + action + tick" (DESIGN_BIBLE §7).
  const headline = event.event.title || event.event.type;
  const scoreCopy = ruptureScoreCopy(event);
  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-[11px] text-ink">
        {headline}
        {count > 1 && <span className="text-accent-gold"> ×{count}</span>}
        <span className="text-ksbc-muted-2"> — tick {event.tick}</span>
      </span>
      {event.event.body && (
        <span className="text-[10px] text-ksbc-muted-2">{event.event.body}</span>
      )}
      {scoreCopy && (
        <span data-testid={`toast-rupture-score-${event.id}`} className="text-[10px] text-rupture">
          {scoreCopy}
        </span>
      )}
    </div>
  );
}
```

  In `ToastCard`, replace the expanded event list (lines 118-124) with run-collapsed lines +
  the count badge:

```tsx
        <div className="flex flex-col gap-1.5">
          {dedupeEvents(toast.events).map((run) => (
            <EventLine key={run.representative.id} event={run.representative} count={run.count} />
          ))}
        </div>
      )}

      {toast.dedupKey !== null && toast.count > 1 && (
        <span
          data-testid={`toast-count-${toast.id}`}
          className="mt-1 inline-block font-mono text-[9px] text-accent-gold"
        >
          ×{toast.count} · through tick {toast.lastTick}
        </span>
      )}
```

  (i.e. the badge block sits immediately after the `isBatch ? … : …` ternary closes, before
  the CTA row `<div className="mt-1.5 flex items-center justify-between gap-2">`.)

  `src/frontend/src/components/chrome/EventTray.tsx` — replace the Missed-entry label
  (lines 115-119) with:

```tsx
                  <span className="truncate">
                    {toast.events.length > 1
                      ? `${toast.events.length} developments — tick ${toast.tick}`
                      : `${toast.count > 1 ? `×${toast.count} ` : ""}${
                          toast.events[0]?.event.title ?? toast.events[0]?.event.type
                        }`}
                  </span>
```

- [ ] **Step 8: Full frontend gate, then commit**:
      `cd src/frontend && npm run check` — expected all green (the classifier/dedup/store/
      component suites plus tsc/eslint/prettier). Then:

```bash
cd /home/user/projects/game/babylon
git add src/frontend/src/store/slices/eventsSlice.ts \
        src/frontend/src/store/slices/eventsSlice.test.ts \
        src/frontend/src/components/events/EventsFeed.tsx \
        src/frontend/src/components/events/EventsFeed.test.tsx \
        src/frontend/src/components/chrome/EventToasts.tsx \
        src/frontend/src/components/chrome/EventToasts.test.tsx \
        src/frontend/src/components/chrome/EventTray.tsx \
        src/frontend/src/components/chrome/EventTray.test.tsx \
        src/frontend/src/components/map/layers/stormMarkers.test.ts \
        src/frontend/src/components/map/layers/criticalPulse.test.ts
mise run commit -- "feat(frontend): collapsed event cards with count+age across feed/toasts/tray (spec-116 FR-116-2)"
```

Determinism: frontend-only; `qa:regression` untouched by construction.

---

### Task 9: Autopause-once — store machinery, CriticalEventModal, e2e dwell simplification

**Files:**

- Modify: `src/frontend/src/store/slices/eventsSlice.ts:52-80` (acknowledged-keys state +
  action, extending the mutes machinery)
- Modify: `src/frontend/src/store/slices/worldSlice.ts:16,65-70` (decision-driven
  mark-then-pause)
- Modify: `src/frontend/src/store/slices/timeSlice.ts:4,55-83,151-192` (rename
  `autopauseEventIds` → `autopauseEventKeys`)
- Modify: `src/frontend/src/components/chrome/CriticalEventModal.tsx` (key join + collapsed
  firing cards)
- Modify: `src/frontend/src/components/events/EventsFeed.tsx:56,101-area` (data-autopause
  joins on `card.key`)
- Modify: `src/frontend/e2e/fixtures.ts:73-119` (`acknowledgeAutopauseIfPresent` dwell
  simplification — bounds stay fixed constants)
- Test: `src/frontend/src/store/slices/worldSlice.test.ts`,
  `src/frontend/src/store/slices/timeSlice.test.ts:140,144,150,416`,
  `src/frontend/src/store/slices/eventsSlice.test.ts`,
  `src/frontend/src/components/chrome/CriticalEventModal.test.tsx`,
  `src/frontend/src/components/events/EventsFeed.test.tsx:62,119`

**Interfaces:**

- Consumes: `computeAutopauseDecision`, `dedupKey`, `dedupeEvents` (Task 7); ToastEntry v2
  (Task 8); `classifyEvents` (existing).
- Produces:
  - `eventsSlice`: `acknowledgedAutopauseKeys: string[]`,
    `acknowledgeAutopauseKeys(keys: string[]): void`.
  - `timeSlice`: `autopauseEventKeys: string[]` (REPLACES `autopauseEventIds` — same
    semantics, now holding tick-independent dedup keys), `autopause(eventKeys: string[])`.
  - Modal firing-card testid: `autopause-event-${dedupKey}`.

- [ ] **Step 1: Write the failing store tests.**
      Append to `src/frontend/src/store/slices/eventsSlice.test.ts`:

```ts
describe("events slice — acknowledged autopause keys (autopause-once, FR-116-2 iii)", () => {
  it("accumulates unique keys across calls (session-scoped, like mutes)", () => {
    useStore.getState().events.acknowledgeAutopauseKeys(["uprising:n1", "uprising:n2"]);
    useStore
      .getState()
      .events.acknowledgeAutopauseKeys(["uprising:n1", "endgame_reached:global@5"]);

    expect(useStore.getState().events.acknowledgedAutopauseKeys).toEqual([
      "uprising:n1",
      "uprising:n2",
      "endgame_reached:global@5",
    ]);
  });
});
```

  In `src/frontend/src/store/slices/worldSlice.test.ts`, replace the autopause test
  (lines 142-149) with:

```ts
  it("autopauses the time slice with dedup keys when the newly-observed tick carries a critical event", async () => {
    resetMockGameState({ events: [makeEvent({ type: "endgame_reached", tick: 2, data: {} })] });

    await useStore.getState().world.fetchState(DEFAULT_GAME_ID);

    expect(useStore.getState().time.status).toBe("autopaused");
    expect(useStore.getState().time.autopauseEventKeys).toEqual(["endgame_reached:global"]);
    expect(useStore.getState().events.acknowledgedAutopauseKeys).toEqual([
      "endgame_reached:global@2",
    ]);
  });
```

  and append a new describe:

```ts
describe("world slice — autopause-once (spec-116 FR-116-2 iii)", () => {
  it("does not re-autopause when the same tick is re-observed after a reload-style reset", async () => {
    resetMockGameState({ events: [makeEvent({ type: "endgame_reached", tick: 2, data: {} })] });
    await useStore.getState().world.fetchState(DEFAULT_GAME_ID);
    useStore.getState().time.resume();

    // Reload: the world slice loses its tick memory; the session-scoped
    // acknowledged set does not (same store instance, same session).
    useStore.setState((s) => ({ world: { ...s.world, snapshot: null, lastTick: null } }));
    await useStore.getState().world.fetchState(DEFAULT_GAME_ID);

    expect(useStore.getState().time.status).toBe("paused");
  });

  it("does not double-autopause when two load fetches race (both see prevTick===null)", async () => {
    resetMockGameState({ events: [makeEvent({ type: "endgame_reached", tick: 2, data: {} })] });

    await Promise.all([
      useStore.getState().world.fetchState(DEFAULT_GAME_ID),
      useStore.getState().world.fetchState(DEFAULT_GAME_ID),
    ]);

    // Whichever racer won, exactly one acknowledgement exists and a single
    // resume STICKS — the loser found the keys already acknowledged.
    expect(useStore.getState().time.status).toBe("autopaused");
    useStore.getState().time.resume();
    expect(useStore.getState().time.status).toBe("paused");
    expect(
      useStore
        .getState()
        .events.acknowledgedAutopauseKeys.filter((k) => k === "endgame_reached:global@2"),
    ).toHaveLength(1);
  });

  it("a NEW endgame occurrence on a later tick still autopauses (always-autopause)", async () => {
    resetMockGameState({ events: [makeEvent({ type: "endgame_reached", tick: 2, data: {} })] });
    await useStore.getState().world.fetchState(DEFAULT_GAME_ID);
    useStore.getState().time.resume();

    setMockSnapshot({
      ...useStore.getState().world.snapshot!,
      tick: 3,
      events: [makeEvent({ type: "endgame_reached", tick: 3, data: {} })],
    });
    await useStore.getState().world.fetchState(DEFAULT_GAME_ID);

    expect(useStore.getState().time.status).toBe("autopaused");
  });
});
```

- [ ] **Step 2: Run to verify failure**:
      `cd src/frontend && npx vitest run src/store/slices/worldSlice.test.ts
      src/store/slices/eventsSlice.test.ts`
      — expected: `acknowledgeAutopauseKeys is not a function`, and
      `time.autopauseEventKeys` undefined (`expected undefined to deeply equal […]`).

- [ ] **Step 3: Implement the slices.**

  `src/frontend/src/store/slices/eventsSlice.ts` — in the `EventsSlice` interface, after
  `mutedCategories` (line ~61), add:

```ts
    /**
     * Salience keys (`${type}:${subject}`, or `key@tick` for always-autopause
     * types) that have already fired an autopause this session — the
     * autopause-once memory (spec-116 FR-116-2 iii). Session-scoped and
     * in-memory, exactly like `mutedCategories` (this slice's mute
     * machinery, which this extends): a page reload starts a fresh session.
     */
    acknowledgedAutopauseKeys: string[];
```

  after `toggleMuteCategory`'s declaration (line ~70), add:

```ts
    /** Record autopause acknowledgement keys as fired (unique, append-order). */
    acknowledgeAutopauseKeys: (keys: string[]) => void;
```

  in the state object, after `mutedCategories: [],` add `acknowledgedAutopauseKeys: [],`
  and after the `toggleMuteCategory` action add:

```ts
    acknowledgeAutopauseKeys: (keys) =>
      set((s) => ({
        events: {
          ...s.events,
          acknowledgedAutopauseKeys: Array.from(
            new Set([...s.events.acknowledgedAutopauseKeys, ...keys]),
          ),
        },
      })),
```

  `src/frontend/src/store/slices/timeSlice.ts` — rename the field and parameter (4 sites +
  docs): line 61 `autopauseEventIds: string[];` →

```ts
    /** Dedup keys (`lib/eventDedup` `${type}:${subject}`) that fired the most recent autopause. */
    autopauseEventKeys: string[];
```

  line 78 `autopause: (eventIds: string[]) => void;` →
  `autopause: (eventKeys: string[]) => void;` (keep the docstring line above it, reworded:
  `/** Called by \`worldSlice\` when a newly-observed tick carries an unacknowledged critical event. */`);
  line 155 `autopauseEventIds: [],` → `autopauseEventKeys: [],`;
  line 185 (`resume`) `autopauseEventIds: []` → `autopauseEventKeys: []`;
  lines 189-192 →

```ts
      autopause: (eventKeys) =>
        set((s) => ({
          time: { ...s.time, status: "autopaused", autopauseEventKeys: eventKeys, playIntent: false },
        })),
```

  Also update the module docstring's `autopaused(eventIds)` mention (line 4) to
  `autopaused(eventKeys)`.

  `src/frontend/src/store/slices/worldSlice.ts` — line 16 add the import:

```ts
import { computeAutopauseDecision } from "@/lib/eventDedup";
```

  and replace lines 65-70 (`const criticalIds … }`) with:

```ts
  // Autopause-once (spec-116 FR-116-2 iii): a distinct (event_type, subject)
  // fires at most once per session; ENDGAME acknowledges per-occurrence
  // (key@tick) so it always fires on a new occurrence. Mark-then-pause is
  // a synchronous check-and-set, so the GameRoute-vs-heartbeat load race
  // (both racers seeing prevTick===null) cannot double-fire — the loser
  // finds the keys already acknowledged.
  const criticalEvents = classifyEvents(snap.events)
    .filter((e) => e.severity === "critical")
    .map((e) => e.event);
  const acknowledged = new Set(get().events.acknowledgedAutopauseKeys);
  const decision = computeAutopauseDecision(criticalEvents, acknowledged);
  if (decision.firingKeys.length > 0) {
    get().events.acknowledgeAutopauseKeys(decision.acknowledgementKeys);
    get().time.autopause(decision.firingKeys);
  }
```

- [ ] **Step 4: Repair the renamed-field call sites** (tsc is the guide — `npm run check`'s
      first leg lists every remaining `autopauseEventIds`):

  - `src/frontend/src/store/slices/timeSlice.test.ts:140` and `:416`:
    `expect(useStore.getState().time.autopauseEventIds).toEqual(["2-0"]);` →
    `expect(useStore.getState().time.autopauseEventKeys).toEqual(["endgame_reached:global"]);`
    (the Task-7 fixture swap made these ticks carry `endgame_reached`); `:150` →
    `expect(useStore.getState().time.autopauseEventKeys).toEqual([]);` (the `autopause(["e1"])`
    call at `:144` needs no change — keys are opaque strings).
  - `src/frontend/src/components/events/EventsFeed.tsx:56`:
    `const autopauseEventIds = useStore((s) => s.time.autopauseEventIds);` →
    `const autopauseEventKeys = useStore((s) => s.time.autopauseEventKeys);` and the card
    attribute (Task 8 version) becomes
    `data-autopause={autopauseEventKeys.includes(card.key) || undefined}` (the join is now
    tick-independent — a card for a still-firing condition keeps its marker across ticks).
  - `src/frontend/src/components/events/EventsFeed.test.tsx:62`:
    `time: { ...s.time, autopauseEventIds: ["3-0"] },` →
    `time: { ...s.time, autopauseEventKeys: ["rupture:territory-downtown"] },`; `:119` (post
    Task 7 the event is `endgame_reached` with `data: {}`):
    `time: { ...s.time, autopauseEventKeys: ["endgame_reached:global"] },`.

- [ ] **Step 5: Rewrite CriticalEventModal (red → green).** First the failing tests — in
      `src/frontend/src/components/chrome/CriticalEventModal.test.tsx`, replace the firing-list
      test (lines 31-42) with:

```ts
  it("lists the firing conditions resolved from time.autopauseEventKeys against the current tick", () => {
    const rupture = makeEvent({ type: "rupture", tick: 3, id: "rupture-id" });
    useStore.setState((s) => ({
      time: { ...s.time, status: "autopaused", autopauseEventKeys: ["rupture:global"] },
      world: { ...s.world, snapshot: makeSnapshot({ tick: 3, events: [rupture] }) },
    }));

    render(<CriticalEventModal gameId="game-1" />);

    expect(screen.getByTestId("autopause-event-rupture:global")).toBeInTheDocument();
    expect(screen.getByTestId("autopause-event-rupture:global")).toHaveTextContent(rupture.title);
  });

  it("collapses same-key repeats into one firing card with a count (FR-116-2)", () => {
    useStore.setState((s) => ({
      time: { ...s.time, status: "autopaused", autopauseEventKeys: ["rupture:global"] },
      world: {
        ...s.world,
        snapshot: makeSnapshot({
          tick: 3,
          events: [
            makeEvent({ type: "rupture", tick: 3, id: "r1" }),
            makeEvent({ type: "rupture", tick: 3, id: "r2" }),
          ],
        }),
      },
    }));

    render(<CriticalEventModal gameId="game-1" />);

    expect(screen.getAllByTestId("autopause-event-rupture:global")).toHaveLength(1);
    expect(screen.getByTestId("autopause-event-rupture:global")).toHaveTextContent("×2");
  });
```

  and at line 55 `autopauseEventIds: ["e1"]` → `autopauseEventKeys: ["e1"]`. Run
  `cd src/frontend && npx vitest run src/components/chrome/CriticalEventModal.test.tsx` —
  expected failure: `Unable to find an element by: [data-testid="autopause-event-rupture:global"]`.
  Then in `src/frontend/src/components/chrome/CriticalEventModal.tsx` add
  `import { dedupKey, dedupeEvents } from "@/lib/eventDedup";`, drop the now-unused
  `import type { ClassifiedEvent } …`, rename the selector (line 25) to
  `const autopauseEventKeys = useStore((s) => s.time.autopauseEventKeys);`, and replace the
  firing computation + list (lines 32-34 and 50-62) with:

```tsx
  // Key join is tick-independent: if the tick advanced but the condition
  // persists, the modal still finds it. Same-key repeats collapse into one
  // card with a count (FR-116-2). The zero-match fallback below stays —
  // an honestly empty record is still possible (Constitution III.11).
  const firing = dedupeEvents(
    classifyEvents(events ?? []).filter((e) => autopauseEventKeys.includes(dedupKey(e.event))),
  );
```

```tsx
            {firing.length === 0 ? (
              <p className="text-[11px] italic text-ksbc-muted-2">
                The firing events are no longer on this tick&apos;s record.
              </p>
            ) : (
              firing.map((run) => (
                <div key={run.key} data-testid={`autopause-event-${run.key}`} className="text-[11px]">
                  <span className="text-ink">
                    {run.representative.event.title || run.representative.event.type}
                  </span>
                  {run.count > 1 && <span className="text-accent-gold"> ×{run.count}</span>}
                  <span className="text-ksbc-muted-2">
                    {" "}
                    — tick {run.firstTick}
                    {run.lastTick !== run.firstTick ? `–${run.lastTick}` : ""}
                  </span>
                  {run.representative.event.body && (
                    <p className="text-[10px] text-ksbc-muted-2">{run.representative.event.body}</p>
                  )}
                </div>
              ))
            )}
```

  Also update the module docstring (lines 4-6): `time.autopauseEventIds … the same id scheme`
  → `time.autopauseEventKeys, joined by salience key (lib/eventDedup) — tick-independent, so
  a persisting condition stays listed after the tick advances`.

- [ ] **Step 6: Run the store + component suites**:
      `cd src/frontend && npx vitest run src/store/slices src/components/chrome
      src/components/events` — expected: all green (including the Task-1-of-9 red tests).

- [ ] **Step 7: Simplify the e2e dwell loop** — in `src/frontend/e2e/fixtures.ts`, replace
      `acknowledgeAutopauseIfPresent` and its docstring (lines 73-119) with:

```ts
/**
 * Acknowledge the CriticalEventModal if it is up. Loading (or stepping) a
 * game whose current tick carries a critical event AUTOPAUSES and raises
 * the alertdialog — its full-screen backdrop intercepts every pointer
 * action until acknowledged. `resume` returns the time slice to plain
 * "paused" (timeSlice.resume), so this never sets the loop playing.
 *
 * Since spec-116 FR-116-2 (autopause-once) the firing keys are recorded in
 * `events.acknowledgedAutopauseKeys` BEFORE the pause lands, so a second
 * in-flight fetch can never re-autopause on the same condition — a single
 * Resume sticks. The short dwell below only covers the FIRST autopause
 * landing late (worldSlice.onTickAdvanced awaits its panel fan-out before
 * pausing); it no longer guards against re-fires. Bounds are fixed
 * constants (statically provable per this repo's loop rule).
 */
export async function acknowledgeAutopauseIfPresent(page: Page): Promise<void> {
  const status = page.getByTestId("time-status");
  const modal = page.getByTestId("critical-event-modal");
  const ATTEMPTS = 6;
  const HOLD = 3; // ~1.2s of continuous PAUSED + no modal
  let consecutivePaused = 0;
  for (let i = 0; i < ATTEMPTS; i++) {
    await expect(status).toHaveText(/^(PAUSED|AUTOPAUSED)$/, { timeout: 15000 });
    if ((await status.textContent()) === "AUTOPAUSED") {
      consecutivePaused = 0;
      await page.getByTestId("autopause-resume").click();
    } else if ((await modal.count()) === 0 && ++consecutivePaused >= HOLD) {
      return; // held clean long enough that a late first autopause would have landed.
    }
    await page.waitForTimeout(400);
  }
  await expect(status).toHaveText("PAUSED", { timeout: 5000 });
  await expect(modal).toHaveCount(0);
}
```

  No new spec file → `AUTHENTICATED_SPECS` in `src/frontend/playwright.config.ts` is
  untouched; `event-popup.spec.ts` / `end-turn-flow.spec.ts` keep calling the same helper.
  Live verification (deferred to the plan's e2e phase, needs `mise run web:dev` + Postgres):
  `cd src/frontend && npx playwright test e2e/event-popup.spec.ts e2e/end-turn-flow.spec.ts`.

- [ ] **Step 8: Full frontend gate**:
      `cd src/frontend && npm run check` — expected: tsc clean (proves no
      `autopauseEventIds` stragglers), eslint/prettier clean, vitest all green.

- [ ] **Step 9: Commit**:

```bash
cd /home/user/projects/game/babylon
git add src/frontend/src/store/slices/eventsSlice.ts \
        src/frontend/src/store/slices/eventsSlice.test.ts \
        src/frontend/src/store/slices/worldSlice.ts \
        src/frontend/src/store/slices/worldSlice.test.ts \
        src/frontend/src/store/slices/timeSlice.ts \
        src/frontend/src/store/slices/timeSlice.test.ts \
        src/frontend/src/components/chrome/CriticalEventModal.tsx \
        src/frontend/src/components/chrome/CriticalEventModal.test.tsx \
        src/frontend/src/components/events/EventsFeed.tsx \
        src/frontend/src/components/events/EventsFeed.test.tsx \
        src/frontend/e2e/fixtures.ts
mise run commit -- "feat(frontend): autopause-once per (type,subject) with session acknowledgement keys (spec-116 FR-116-2)"
```

Determinism: frontend-only; `qa:regression` untouched by construction.

---

### Cluster risks / flags for the lead

1. **Post-re-tier, the only live autopause trigger is `endgame_reached`** (fires at
   horizon/player-accept under Cluster A). HOI4-style pause-on-crisis effectively retires for
   mid-campaign events; the once-machinery is the durable invariant guarding any type later
   promoted back to frontend-critical (a one-line map change). If the owner wants mid-campaign
   autopause drama, that promotion is a deliberate follow-up ruling, not part of FR-116-2.
2. **Bridge `_EVENT_SEVERITY` is deliberately untouched** (interface ledger scopes the re-tier
   to eventClassifier): the EventTray count badges (`summary.event_counts`) and the alerts
   dashboard still use the bridge vocabulary, where 13 types remain "critical" — crimson
   *counts* can appear without any crimson *card*. Residual two-vocabulary seam; flag for a
   follow-up ruling (out of scope here to avoid touching persisted `tick_event` severities).
3. **`pattern_shift` map entries are dormant until Cluster A lands** the EventType + bridge
   emission. Harmless: the frontend map is not gated by Seam Sensor 1 (which gates the
   bridge's `_EVENT_SEVERITY` only), and an unknown type already degrades gracefully.
4. **Acknowledged keys and mutes are per-page-load** (no zustand persist — same lifecycle as
   the existing `mutedCategories`): a reload re-fires at most one autopause per still-firing
   condition. This matches the ledger's "per session"; localStorage persistence is a
   deliberate non-goal.
5. **`timeSlice.autopauseEventIds` → `autopauseEventKeys` rename**: any other cluster's task
   that references the old field name must switch (tsc catches it). Cluster A's ledger
   entries don't touch it, but flag in the plan preamble.
6. **The race test's interleaving is nondeterministic** (which fetch wins), so its assertions
   are interleaving-invariant (status, resume-sticks, exactly one acknowledgement). Do not
   "strengthen" it to assert which racer fired.
## Cluster: Lobby & Briefing (FR-116-3)

Generated operation codenames (deterministic, UUID-derived — never rng_seed, which is 0 on
every existing session), dates/tick/status in the lobby list, delete/archive, a post-create
Scenario Briefing interstitial framed by the fixed-horizon ruling (100 years / 5,200 ticks;
the five outcomes are recognized patterns, never terminators), and curated difficulty
exposure through the existing `CreateGameSerializer` overrides — never raw defines.

Determinism note for the whole cluster: every change is web/API/frontend-layer — no engine,
no `GameDefines` schema, no formulas. `mise run qa:regression` stays byte-identical by
construction (no touched code path executes inside a tick).

Seam note: no task in this cluster adds a bridge-serialized wire key. `codename`,
`{"deleted": true}`, and `{"status": "abandoned"}` are emitted by `web/game/api.py` DB-listing
views (no `bridge.get_*` call), which Sensor 1's gating checks do not scan (its gating tier is
map-contract / tick-writes / severity-vocabulary; the endpoints.ts↔urls.py join is the
ADVISORY `check_bridge_serialization`). Zero SeamEntry rows required; the endpoints.ts
manifest rows land in Task 11 so the advisory join stays clean.

### Task 10: Backend — operation codenames, list fields, delete/archive

**Files:**
- Create: `web/game/codenames.py`
- Create: `tests/unit/web/test_lobby_lifecycle.py` (new file rather than extending
  `test_api.py` — Cluster A's accept-outcome task also extends `test_api.py`, and a
  self-contained file avoids collision churn; patterns copied from
  `test_api.py::TestURLRouting` / `TestCreateGameScenarioValidation`)
- Modify: `web/game/api.py` (imports :32; `game_list` dict comprehension :229-238;
  `game_detail` :329-344 gains DELETE + codename; new `game_archive` view after
  `game_resume` :370)
- Modify: `web/game/serializers.py:364-371` (`GameSessionListSerializer` + `codename`)
- Modify: `web/game/urls.py:28` (archive route after `game-recover`)

**Interfaces:**
- Consumes: nothing from other tasks (pure backend; independent of Cluster A).
- Produces:
  - `game.codenames.operation_codename(session_id: uuid.UUID) -> str` — pure,
    deterministic, `"ADJECTIVE NOUN"` uppercase.
  - `codename` (str) key in `GET /api/games/` rows AND `GET /api/games/{id}/` data.
  - `DELETE /api/games/{id}/` → `{"status":"ok","data":{"deleted":true},"session_id":...}`;
    404 for missing/other-user sessions. Hard delete: every child table declares
    `ON DELETE CASCADE` in `postgres_schema.py`, and the Django FK models
    (`PlayerAction`/`ActionResult`/`HexState`/all snapshot models, `web/game/models.py`)
    mirror `on_delete=CASCADE`, so it cascades in both the PG runtime and the SQLite
    test DB.
  - `POST /api/games/{id}/archive/` → `{"status":"ok","data":{"status":"abandoned"}}`;
    URL name `game:game-archive`. Reversible soft delete — `"abandoned"` is already in
    the frontend `GameStatus` union.

- [ ] **Step 1: Write the failing test** — create `tests/unit/web/test_lobby_lifecycle.py`:

```python
"""Lobby lifecycle surface (spec-116 FR-116-3).

Covers the three backend seams the lobby rebuild rides on:

- ``web/game/codenames.py`` — deterministic UUID-derived operation codenames;
- ``codename`` surfaced by ``GET /api/games/`` (list) and ``GET /api/games/{id}/``;
- ``DELETE /api/games/{id}/`` (hard delete, FK cascade) and
  ``POST /api/games/{id}/archive/`` (reversible ``status='abandoned'``).

Pattern provenance: Client/django_db view tests follow
``tests/unit/web/test_api.py::TestCreateGameScenarioValidation``; URL-resolution
tests follow ``TestURLRouting``. Imports stay inside test bodies so a missing
module fails the TEST (red phase), not collection.
"""

from __future__ import annotations

import json
import uuid

import pytest
from django.test import Client
from django.urls import reverse


def _login_client(username: str) -> tuple[Client, int]:
    """Create a fresh user and return a logged-in test client plus the user id."""
    from django.contrib.auth.models import User

    user = User.objects.create_user(username=username, password="lobbypass123")  # type: ignore[no-untyped-call]
    client = Client()
    client.login(username=username, password="lobbypass123")
    return client, int(user.id)


@pytest.mark.unit
class TestOperationCodename:
    """The codename generator is a pure, deterministic function of the UUID."""

    def test_same_uuid_always_yields_the_same_codename(self) -> None:
        from game.codenames import operation_codename

        sid = uuid.UUID("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")
        assert operation_codename(sid) == operation_codename(sid)

    def test_codename_is_two_uppercase_words_from_the_curated_lists(self) -> None:
        from game.codenames import _LEFT, _RIGHT, operation_codename

        parts = operation_codename(uuid.uuid4()).split(" ")
        assert len(parts) == 2
        assert parts[0] in _LEFT
        assert parts[1] in _RIGHT

    def test_codename_derives_from_uuid_not_rng_seed(self) -> None:
        """Sessions differing only in leading UUID bytes get distinct names.

        Guards the recon gotcha: ``rng_seed`` is 0 for every existing session
        (serializer default, lobby never sends one), so a seed-derived codename
        would collide across ALL games. UUID bytes 0-1 select the left word and
        bytes 2-3 the right word, so these two UUIDs provably differ.
        """
        from game.codenames import operation_codename

        a = operation_codename(uuid.UUID("00000000-0000-0000-0000-000000000000"))
        b = operation_codename(uuid.UUID("00010001-0000-0000-0000-000000000000"))
        assert a != b


@pytest.mark.unit
class TestGameSessionListSerializerCodename:
    """``codename`` must be a DECLARED field — DRF silently drops undeclared keys."""

    def test_codename_round_trips_through_the_list_serializer(self) -> None:
        from game.serializers import GameSessionListSerializer

        row = {
            "id": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
            "scenario": "wayne_county",
            "current_tick": 3,
            "status": "active",
            "created_at": "2026-07-17T12:00:00Z",
            "codename": "CRIMSON HARVEST",
        }
        serializer = GameSessionListSerializer(row)
        assert serializer.data["codename"] == "CRIMSON HARVEST"


@pytest.mark.unit
@pytest.mark.django_db
class TestLobbyCodenameSurfacing:
    """Both list and detail views emit the derived codename."""

    def test_game_list_rows_carry_the_derived_codename(self) -> None:
        from game.codenames import operation_codename
        from game.models import GameSession

        client, user_id = _login_client("lister")
        session = GameSession.objects.create(player_id=user_id, scenario="wayne_county")

        response = client.get("/api/games/")

        assert response.status_code == 200
        rows = json.loads(response.content)["data"]
        assert rows[0]["codename"] == operation_codename(session.id)

    def test_game_detail_carries_the_derived_codename(self) -> None:
        from game.codenames import operation_codename
        from game.models import GameSession

        client, user_id = _login_client("detailer")
        session = GameSession.objects.create(player_id=user_id, scenario="wayne_county")

        response = client.get(f"/api/games/{session.id}/")

        assert response.status_code == 200
        data = json.loads(response.content)["data"]
        assert data["codename"] == operation_codename(session.id)


@pytest.mark.unit
@pytest.mark.django_db
class TestGameDeleteAndArchive:
    """DELETE = permanent (cascade); archive = reversible status flip."""

    def test_archive_url_resolves(self) -> None:
        url = reverse(
            "game:game-archive",
            kwargs={"game_id": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"},
        )
        assert url == "/api/games/aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee/archive/"

    def test_delete_removes_the_session_row(self) -> None:
        from game.models import GameSession

        client, user_id = _login_client("deleter")
        session = GameSession.objects.create(player_id=user_id, scenario="wayne_county")

        response = client.delete(f"/api/games/{session.id}/")

        assert response.status_code == 200
        body = json.loads(response.content)
        assert body["data"] == {"deleted": True}
        assert GameSession.objects.filter(id=session.id).count() == 0

    def test_delete_another_users_session_is_a_404(self) -> None:
        from game.models import GameSession

        _, owner_id = _login_client("owner")
        session = GameSession.objects.create(player_id=owner_id, scenario="wayne_county")
        intruder, _ = _login_client("intruder")

        response = intruder.delete(f"/api/games/{session.id}/")

        assert response.status_code == 404
        assert GameSession.objects.filter(id=session.id).count() == 1

    def test_archive_sets_status_abandoned(self) -> None:
        from game.models import GameSession

        client, user_id = _login_client("archiver")
        session = GameSession.objects.create(player_id=user_id, scenario="wayne_county")

        response = client.post(f"/api/games/{session.id}/archive/")

        assert response.status_code == 200
        assert json.loads(response.content)["data"] == {"status": "abandoned"}
        session.refresh_from_db()
        assert session.status == "abandoned"

    def test_archiving_an_archived_game_is_a_loud_400(self) -> None:
        from game.models import GameSession

        client, user_id = _login_client("rearchiver")
        session = GameSession.objects.create(
            player_id=user_id, scenario="wayne_county", status="abandoned"
        )

        response = client.post(f"/api/games/{session.id}/archive/")

        assert response.status_code == 400
        assert json.loads(response.content)["status"] == "error"
```

- [ ] **Step 2: Run test to verify it fails**

```bash
mise run test:q -- tests/unit/web/test_lobby_lifecycle.py
```

Expected: 10 failures — `ModuleNotFoundError: No module named 'game.codenames'` (codename
tests), `KeyError: 'codename'` (serializer/list/detail), `NoReverseMatch: ... 'game-archive'
not found` (URL test), `assert 405 == 200` (DELETE — method not yet allowed), `assert 404 ==
200` (archive POST — route absent).

- [ ] **Step 3: Write minimal implementation — the codename module.**
      Create `web/game/codenames.py` (complete file):

```python
"""Deterministic operation codenames (spec-116 FR-116-3).

A session's codename is a pure function of its UUID primary key — the same
session always renders the same name, with zero schema change to the
``managed=False`` ``game_session`` table (computed on read, per the FR-116-3
recon ruling). Deliberately NOT derived from ``rng_seed``: that column is 0
for every pre-existing session (serializer default; the lobby historically
never sent one), so a seed-derived name would collide across all games.

32 x 32 curated single-word lists give 1,024 distinct codenames; indices come
from the UUID's first four bytes (big-endian, two bytes per list) so the
mapping is byte-stable across processes and platforms.
"""

from __future__ import annotations

from typing import Final
from uuid import UUID

#: Left word — evocative modifier (indexed by UUID bytes 0-1).
_LEFT: Final[tuple[str, ...]] = (
    "CRIMSON",
    "IRON",
    "EMBER",
    "GRANITE",
    "SCARLET",
    "HOLLOW",
    "SILENT",
    "NORTHERN",
    "RUSTED",
    "VELVET",
    "COPPER",
    "MIDNIGHT",
    "BURNING",
    "FALLOW",
    "WINTER",
    "SALT",
    "ASH",
    "LONG",
    "BROKEN",
    "PATIENT",
    "RED",
    "STONE",
    "HUNGRY",
    "DISTANT",
    "EARLY",
    "LAST",
    "SOVEREIGN",
    "UNION",
    "HARBOR",
    "SIGNAL",
    "QUIET",
    "FERAL",
)

#: Right word — concrete noun (indexed by UUID bytes 2-3).
_RIGHT: Final[tuple[str, ...]] = (
    "HARVEST",
    "DAWN",
    "FURNACE",
    "ANVIL",
    "RIVER",
    "LANTERN",
    "THRESHOLD",
    "SPINDLE",
    "GRANARY",
    "PICKET",
    "TELEGRAPH",
    "FOUNDRY",
    "ORCHARD",
    "CROSSING",
    "EMBANKMENT",
    "TURBINE",
    "QUARRY",
    "SICKLE",
    "BALLAST",
    "MERIDIAN",
    "WATCHTOWER",
    "CAUSEWAY",
    "DYNAMO",
    "ARCHIVE",
    "BULWARK",
    "TRELLIS",
    "COMPASS",
    "VIADUCT",
    "SEMAPHORE",
    "TANNERY",
    "MILLSTONE",
    "ACCORD",
)


def operation_codename(session_id: UUID) -> str:
    """Derive a stable two-word operation codename from a session UUID.

    :param session_id: The game session's primary key.
    :returns: ``"LEFT RIGHT"`` in uppercase, e.g. ``"CRIMSON HARVEST"``.
    """
    left_index = int.from_bytes(session_id.bytes[0:2], "big") % len(_LEFT)
    right_index = int.from_bytes(session_id.bytes[2:4], "big") % len(_RIGHT)
    return f"{_LEFT[left_index]} {_RIGHT[right_index]}"
```

- [ ] **Step 4: Write minimal implementation — wire the views, serializer, and route.**

  (a) `web/game/serializers.py:364-371` — add the field:

```python
class GameSessionListSerializer(serializers.Serializer[dict[str, Any]]):
    """Serialize a game session for list views."""

    id = serializers.UUIDField()
    codename = serializers.CharField()
    scenario = serializers.CharField()
    current_tick = serializers.IntegerField()
    status = serializers.CharField()
    created_at = serializers.DateTimeField()
```

  (b) `web/game/api.py:32` — add the import (alphabetically before `.log_handler`):

```python
from .codenames import operation_codename
from .log_handler import log_game_event, sanitize_for_log
```

  (c) `web/game/api.py:229-238` — `game_list`'s GET dict comprehension gains the key
  (recon gotcha: the field must exist in BOTH the dict AND the serializer or DRF
  silently drops it):

```python
        session_data: list[dict[str, Any]] = [
            {
                "id": s.id,
                "codename": operation_codename(s.id),
                "scenario": s.scenario,
                "current_tick": s.current_tick,
                "status": s.status,
                "created_at": s.created_at,
            }
            for s in sessions
        ]
```

  (d) `web/game/api.py:329-344` — `game_detail` gains DELETE and the codename:

```python
@api_view(["GET", "DELETE"])
@permission_classes([IsAuthenticated])
def game_detail(request: Request, game_id: str) -> JsonResponse:
    """GET /api/games/{id}/ — session metadata. DELETE — destroy the session.

    DELETE is the permanent path (FR-116-3 "delete"): every child table
    declares ``REFERENCES game_session(id) ON DELETE CASCADE``
    (``postgres_schema.py``) and the Django FK models mirror
    ``on_delete=CASCADE``, so turns/results/snapshots go with the session in
    both the PG runtime and the SQLite test DB. The reversible alternative is
    ``POST /api/games/{id}/archive/``.
    """
    session = _get_session_or_none(game_id, request.user.id)
    if session is None:
        return _error("Game not found", http_status=404)

    if request.method == "DELETE":
        GameSession.objects.filter(id=session.id).delete()
        logger.info("Game deleted session=%s user=%s", session.id, request.user.id)
        return _envelope({"deleted": True}, session_id=str(session.id))

    data = {
        "id": str(session.id),
        "codename": operation_codename(session.id),
        "scenario": session.scenario,
        "current_tick": session.current_tick,
        "status": session.status,
        "created_at": session.created_at.isoformat() if session.created_at else None,
    }
    return _envelope(data, tick=session.current_tick, session_id=str(session.id))
```

  (e) `web/game/api.py` — insert after `game_resume` (line 370), modeled byte-for-byte on
  `game_pause`:

```python
@api_view(["POST"])
@permission_classes([IsAuthenticated])
def game_archive(request: Request, game_id: str) -> JsonResponse:
    """POST /api/games/{id}/archive/ — archive a game (reversible soft delete).

    Sets ``status='abandoned'`` — a status the frontend ``GameStatus`` union
    already carries. Any live status may be archived; re-archiving is a loud
    400, not a silent no-op (Constitution III.11).
    """
    session = _get_session_or_none(game_id, request.user.id)
    if session is None:
        return _error("Game not found", http_status=404)
    if session.status == "abandoned":
        return _error("Game is already archived")
    GameSession.objects.filter(id=session.id).update(
        status="abandoned", updated_at=timezone.now()
    )
    return _envelope({"status": "abandoned"}, session_id=str(session.id))
```

  (f) `web/game/urls.py:28` — register the route in the lifecycle block:

```python
    path("games/<str:game_id>/recover/", api.game_recover, name="game-recover"),
    path("games/<str:game_id>/archive/", api.game_archive, name="game-archive"),
```

- [ ] **Step 5: Run test to verify it passes**

```bash
mise run test:q -- tests/unit/web/test_lobby_lifecycle.py
```

Expected: `10 passed`. Then prove the neighbors and the seam gate didn't move:

```bash
mise run test:q -- tests/unit/web/test_api.py tests/unit/web/test_serializers.py
mise run check:seams
```

Expected: all passed; `check:seams` clean (no bridge serializer touched — `game_list`/
`game_detail`/`game_archive` are DB-listing views outside Sensor 1's gating scans, and
`game_archive` calls no bridge serializer so even the advisory join stays silent).

- [ ] **Step 6: Commit**

```bash
git add web/game/codenames.py web/game/api.py web/game/serializers.py web/game/urls.py tests/unit/web/test_lobby_lifecycle.py
mise run commit -- "feat(web): operation codenames + delete/archive lifecycle (spec-116 FR-116-3)"
```

### Task 11: LobbyRoute rebuild — codename rows, delete/archive, curated difficulty

**Files:**
- Modify: `src/frontend/src/types/game.ts:21-27` (`GameSummary.codename`)
- Modify: `src/frontend/src/api/endpoints.ts:113-116` (`gameDelete`/`gameArchive` rows —
  the Seam Observatory bridge sentinel parses this file; new backend routes need rows here)
- Modify: `src/frontend/src/api/client.ts:100-106` (add `del` helper — no DELETE helper
  exists today; `endpoints.actionDelete` is declared but nothing calls through it)
- Create: `src/frontend/src/lib/difficulty.ts`
- Modify: `src/frontend/src/store/slices/sessionSlice.ts:17-38` (interface) and `:120-131`
  (`deleteGame`/`archiveGame` next to `createGame`)
- Modify: `src/frontend/src/routes/LobbyRoute.tsx:29-119` (SelectionItem `trailing`/`dimmed`),
  `:121-157` (state + handlers), `:192-232` (difficulty picker in the New Operation plate),
  `:240-269` (games plate rows)
- Modify: `src/frontend/src/test/fixtures.ts:299-308` (`makeGameSummary` + codename)
- Modify: `src/frontend/src/test/handlers.ts:112-115` (default DELETE + archive handlers)
- Test: `src/frontend/src/store/slices/sessionSlice.test.ts`,
  `src/frontend/src/routes/LobbyRoute.test.tsx`

**Interfaces:**
- Consumes (Task 10): `codename` in `GET /api/games/` rows; `DELETE /api/games/{id}/` →
  `{"deleted": true}`; `POST /api/games/{id}/archive/` → `{"status": "abandoned"}`.
- Produces:
  - `GameSummary.codename: string` (`types/game.ts`).
  - `endpoints.gameDelete` (`"/api/games/:id/"`, DELETE), `endpoints.gameArchive`
    (`"/api/games/:id/archive/"`, POST).
  - `del<T>(url: string): Promise<ApiResponse<T>>` in `api/client.ts`.
  - `session.deleteGame(id: string) => Promise<boolean>`,
    `session.archiveGame(id: string) => Promise<boolean>`.
  - `DIFFICULTY_PRESETS: readonly DifficultyPreset[]` + `rollRngSeed(): number`
    (`lib/difficulty.ts`).
  - testids: `game-archive-${id}`, `game-delete-${id}`, `difficulty-option-${key}`
    (row testid contract `game-option-${id}` and the literal "New Operation" plate label —
    the e2e `login()` landmark — both PRESERVED).
- Task 11 keeps `handleCreate` navigating to `/game/${id}` — Task 12 flips it to the
  briefing.

- [ ] **Step 1: Update test fixtures and default handlers (test infra for the reds).**

  (a) `src/frontend/src/test/fixtures.ts:299-308` — `makeGameSummary` gains the codename:

```ts
export function makeGameSummary(overrides?: Partial<GameSummary>): GameSummary {
  return {
    id: "game-001",
    codename: "CRIMSON HARVEST",
    scenario: "default",
    current_tick: 5,
    status: "active",
    created_at: "2026-03-01T12:00:00Z",
    ...overrides,
  };
}
```

  (b) `src/frontend/src/test/handlers.ts` — after the `http.post("/api/games/", ...)`
  handler (line 112-114), add the two lifecycle defaults:

```ts
  http.delete("/api/games/:id/", () => {
    logRequest("DELETE game");
    return HttpResponse.json({ status: "ok", data: { deleted: true } });
  }),

  http.post("/api/games/:id/archive/", () => {
    logRequest("POST archive");
    return HttpResponse.json({ status: "ok", data: { status: "abandoned" } });
  }),
```

- [ ] **Step 2: Write the failing tests.**

  (a) Append to `src/frontend/src/store/slices/sessionSlice.test.ts` (and add
  `import { makeGameSummary } from "@/test/fixtures";` to its imports):

```ts
describe("session slice — delete/archive (spec-116 FR-116-3)", () => {
  it("deleteGame issues DELETE /api/games/:id/ and refreshes the list", async () => {
    let deleted = false;
    server.use(
      http.delete("/api/games/:id/", () => {
        deleted = true;
        return HttpResponse.json({ status: "ok", data: { deleted: true } });
      }),
      http.get("/api/games/", () =>
        HttpResponse.json({ status: "ok", data: deleted ? [] : [makeGameSummary()] }),
      ),
    );

    const ok = await useStore.getState().session.deleteGame(DEFAULT_GAME_ID);

    expect(ok).toBe(true);
    expect(deleted).toBe(true);
    expect(useStore.getState().session.games).toEqual([]);
  });

  it("deleteGame failure records the error and returns false", async () => {
    server.use(
      http.delete("/api/games/:id/", () =>
        HttpResponse.json({ status: "error", message: "Game not found" }, { status: 404 }),
      ),
    );

    const ok = await useStore.getState().session.deleteGame(DEFAULT_GAME_ID);

    expect(ok).toBe(false);
    expect(useStore.getState().session.error).toBe("Game not found");
  });

  it("archiveGame POSTs /archive/ and refreshes the list", async () => {
    let archived = false;
    server.use(
      http.post("/api/games/:id/archive/", () => {
        archived = true;
        return HttpResponse.json({ status: "ok", data: { status: "abandoned" } });
      }),
      http.get("/api/games/", () =>
        HttpResponse.json({
          status: "ok",
          data: [makeGameSummary({ status: archived ? "abandoned" : "active" })],
        }),
      ),
    );

    const ok = await useStore.getState().session.archiveGame(DEFAULT_GAME_ID);

    expect(ok).toBe(true);
    expect(useStore.getState().session.games[0]?.status).toBe("abandoned");
  });

  it("archiveGame failure records the error and returns false", async () => {
    server.use(
      http.post("/api/games/:id/archive/", () =>
        HttpResponse.json({ status: "error", message: "Game is already archived" }, { status: 400 }),
      ),
    );

    const ok = await useStore.getState().session.archiveGame(DEFAULT_GAME_ID);

    expect(ok).toBe(false);
    expect(useStore.getState().session.error).toBe("Game is already archived");
  });
});
```

  (b) Replace `src/frontend/src/routes/LobbyRoute.test.tsx` in full (the old
  `getByText("default")` label assertions are INTENTIONALLY retired — the row label is now
  the codename; recon gotcha says update them in the same commit):

```tsx
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router";
import { http, HttpResponse } from "msw";
import { server } from "@/test/server";
import { LobbyRoute } from "./LobbyRoute";
import { resetStore } from "@/test/resetStore";
import { resetMockGameState } from "@/test/handlers";
import { makeGameSummary } from "@/test/fixtures";

beforeEach(() => {
  resetStore();
  resetMockGameState();
});

function renderLobby(): void {
  render(
    <MemoryRouter initialEntries={["/lobby"]}>
      <Routes>
        <Route path="/lobby" element={<LobbyRoute />} />
        <Route path="/game/:id" element={<div>GAME SHELL</div>} />
        <Route path="/login" element={<div>LOGIN</div>} />
      </Routes>
    </MemoryRouter>,
  );
}

describe("LobbyRoute", () => {
  it("lists real games by codename and navigates on click", async () => {
    renderLobby();
    await waitFor(() => expect(screen.getByText("CRIMSON HARVEST")).toBeInTheDocument());
    await userEvent.click(screen.getByText("CRIMSON HARVEST"));
    expect(screen.getByText("GAME SHELL")).toBeInTheDocument();
  });

  it("renders scenario, tick, status, and date on each row", async () => {
    renderLobby();
    await waitFor(() => expect(screen.getByText("CRIMSON HARVEST")).toBeInTheDocument());
    // makeGameSummary: scenario "default" (no catalog name -> raw key fallback),
    // tick 5, active, created 2026-03-01T12:00:00Z -> ISO date prefix.
    expect(screen.getByText("default · Tick 5 · ACTIVE · 2026-03-01")).toBeInTheDocument();
  });

  it("creates a new game via the real /api/games/ POST and navigates to it", async () => {
    renderLobby();
    await waitFor(() => expect(screen.getByText("Wayne County Organizer")).toBeInTheDocument());
    await userEvent.click(screen.getByRole("button", { name: /new game/i }));
    await waitFor(() => expect(screen.getByText("GAME SHELL")).toBeInTheDocument());
  });

  it("create sends the selected curated difficulty preset and a rolled rng_seed", async () => {
    let createBody: Record<string, unknown> | null = null;
    server.use(
      http.post("/api/games/", async ({ request }) => {
        createBody = (await request.json()) as Record<string, unknown>;
        return HttpResponse.json(
          { status: "ok", data: { session_id: "game-001" } },
          { status: 201 },
        );
      }),
    );
    renderLobby();
    await waitFor(() =>
      expect(screen.getByTestId("difficulty-option-besieged")).toBeInTheDocument(),
    );

    await userEvent.click(screen.getByTestId("difficulty-option-besieged"));
    await userEvent.click(screen.getByRole("button", { name: /new game/i }));

    await waitFor(() => expect(createBody).not.toBeNull());
    expect(createBody?.defines).toEqual({
      economy: { extraction_efficiency: 0.9 },
      survival: { default_subsistence: 0.4 },
    });
    expect(typeof createBody?.rng_seed).toBe("number");
  });

  it("delete is arm-then-confirm: two clicks issue the DELETE and drop the row", async () => {
    let deleted = false;
    server.use(
      http.delete("/api/games/:id/", () => {
        deleted = true;
        return HttpResponse.json({ status: "ok", data: { deleted: true } });
      }),
      http.get("/api/games/", () =>
        HttpResponse.json({ status: "ok", data: deleted ? [] : [makeGameSummary()] }),
      ),
    );
    renderLobby();
    await waitFor(() => expect(screen.getByTestId("game-delete-game-001")).toBeInTheDocument());

    await userEvent.click(screen.getByTestId("game-delete-game-001"));
    expect(deleted).toBe(false); // first click only arms the button
    await userEvent.click(screen.getByTestId("game-delete-game-001"));

    await waitFor(() =>
      expect(screen.queryByTestId("game-option-game-001")).not.toBeInTheDocument(),
    );
    expect(deleted).toBe(true);
    expect(screen.queryByText("GAME SHELL")).not.toBeInTheDocument(); // no row navigation
  });

  it("archive issues POST /archive/ without navigating and re-lists as abandoned", async () => {
    let archived = false;
    server.use(
      http.post("/api/games/:id/archive/", () => {
        archived = true;
        return HttpResponse.json({ status: "ok", data: { status: "abandoned" } });
      }),
      http.get("/api/games/", () =>
        HttpResponse.json({
          status: "ok",
          data: [makeGameSummary({ status: archived ? "abandoned" : "active" })],
        }),
      ),
    );
    renderLobby();
    await waitFor(() => expect(screen.getByTestId("game-archive-game-001")).toBeInTheDocument());

    await userEvent.click(screen.getByTestId("game-archive-game-001"));

    await waitFor(() =>
      expect(screen.getByText("default · Tick 5 · ABANDONED · 2026-03-01")).toBeInTheDocument(),
    );
    expect(screen.queryByText("GAME SHELL")).not.toBeInTheDocument();
    // An archived row offers no second archive affordance.
    expect(screen.queryByTestId("game-archive-game-001")).not.toBeInTheDocument();
  });

  it("logs out and returns to /login", async () => {
    renderLobby();
    await waitFor(() => expect(screen.getByText("CRIMSON HARVEST")).toBeInTheDocument());
    await userEvent.click(screen.getByRole("button", { name: /logout/i }));
    await waitFor(() => expect(screen.getByText("LOGIN")).toBeInTheDocument());
  });
});
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
cd src/frontend && npx vitest run src/routes/LobbyRoute.test.tsx src/store/slices/sessionSlice.test.ts
```

Expected: sessionSlice reds with `TypeError: useStore.getState().session.deleteGame is not
a function` (and archiveGame); LobbyRoute reds with `Unable to find an element with the
text: CRIMSON HARVEST` / `Unable to find an element by: [data-testid="difficulty-option-besieged"]`
etc. Pre-existing tests in both files stay green except the two retired `"default"`-label
assertions, now rewritten.

- [ ] **Step 4: Write minimal implementation.**

  (a) `src/frontend/src/types/game.ts:21-27` — `GameSummary` gains the field:

```ts
/** Game session summary (from GET /api/games/). */
export interface GameSummary {
  id: string;
  /** Deterministic operation codename derived server-side from the session UUID. */
  codename: string;
  scenario: string;
  current_tick: number;
  status: GameStatus;
  created_at: string;
}
```

  (b) `src/frontend/src/api/endpoints.ts:113-116` — two manifest rows next to their
  siblings (same-pattern `gameDelete` differs from `gameDetail` only by method — both
  `Untyped`, so the sentinel's advisory join is unchanged):

```ts
  gameDetail: ep<Untyped>("/api/games/:id/"),
  gameDelete: ep<Untyped>("/api/games/:id/", "DELETE"),
  gameArchive: ep<Untyped>("/api/games/:id/archive/", "POST"),
  gamePause: ep<Untyped>("/api/games/:id/pause/", "POST"),
```

  (c) `src/frontend/src/api/client.ts` — after `post` (line 106):

```ts
/** DELETE request. */
export async function del<T>(url: string): Promise<ApiResponse<T>> {
  return request<T>(url, { method: "DELETE" });
}
```

  (d) Create `src/frontend/src/lib/difficulty.ts` (complete file):

```ts
/**
 * Curated difficulty presets (spec-116 FR-116-3).
 *
 * The lobby exposes difficulty ONLY through this vetted map — never a raw
 * defines editor. Each `defines` object is a partial GameDefines override,
 * validated server-side by `GameDefines(**defines)` inside
 * `EngineBridge.create_game` (engine_bridge.py:1975) — which is AFTER
 * serializer validation, so an invalid value would surface as a 500 (recon
 * gotcha). Every value here therefore stays inside the schema's declared
 * field constraints, so a preset can never 500 the create call:
 *
 *  - `consciousness.sensitivity`      ge=0, le=1, default 0.5
 *  - `economy.extraction_efficiency`  ge=0, le=1, default 0.8
 *  - `survival.default_subsistence`   ge=0, le=1, default 0.3
 *
 * (src/babylon/config/defines/{consciousness,economy_basic,survival}.py —
 * the same three knobs tools/regression_test.py's scenario overrides already
 * exercise, so their validity is regression-proven.)
 */

export interface DifficultyPreset {
  key: string;
  label: string;
  description: string;
  defines: Record<string, unknown>;
}

export const DIFFICULTY_PRESETS: readonly DifficultyPreset[] = [
  {
    key: "agitator",
    label: "AGITATOR",
    description: "Consciousness drifts faster — a forgiving conjuncture",
    defines: { consciousness: { sensitivity: 0.7 } },
  },
  {
    key: "cadre",
    label: "CADRE",
    description: "The standard conjuncture — schema defaults untouched",
    defines: {},
  },
  {
    key: "besieged",
    label: "BESIEGED",
    description: "Deeper extraction, thinner margins of survival",
    defines: {
      economy: { extraction_efficiency: 0.9 },
      survival: { default_subsistence: 0.4 },
    },
  },
];

/**
 * Fresh 31-bit rng seed for a new session (spec-061 FR-024 threading). The
 * historical default of 0-for-everyone made per-session replay seeds
 * meaningless; the lobby now rolls one at create time. (Codenames derive from
 * the session UUID server-side — the seed drives engine replay determinism,
 * not naming.)
 */
export function rollRngSeed(): number {
  return Math.floor(Math.random() * 2147483647);
}
```

  (e) `src/frontend/src/store/slices/sessionSlice.ts` — import the helper, extend the
  interface, add the two actions after `createGame` (:120-131):

```ts
import { get as apiGet, post as apiPost, postForm as apiPostForm, del as apiDel } from "@/api/client";
```

  Interface additions (inside `session`, after `createGame`):

```ts
    /** Returns the new session id on success, or null on failure. */
    createGame: (params: CreateGameParams) => Promise<string | null>;
    /** Permanently deletes the session server-side. True on success. */
    deleteGame: (id: string) => Promise<boolean>;
    /** Archives the session (status='abandoned', reversible). True on success. */
    archiveGame: (id: string) => Promise<boolean>;
    setActiveGame: (id: string | null) => void;
```

  Implementation (after the `createGame` implementation):

```ts
    deleteGame: async (id) => {
      const res = await apiDel(endpoints.gameDelete.path({ id }));
      if (res.status === "ok") {
        await get().session.fetchGames();
        return true;
      }
      set((s) => ({ session: { ...s.session, error: res.message ?? "Failed to delete game" } }));
      return false;
    },

    archiveGame: async (id) => {
      const res = await apiPost(endpoints.gameArchive.path({ id }));
      if (res.status === "ok") {
        await get().session.fetchGames();
        return true;
      }
      set((s) => ({ session: { ...s.session, error: res.message ?? "Failed to archive game" } }));
      return false;
    },
```

  (f) `src/frontend/src/routes/LobbyRoute.tsx` — four edits.

  Edit 1 — imports (top of file):

```tsx
import { useEffect, useState, type KeyboardEvent } from "react";
import { useNavigate } from "react-router";
import { useStore } from "@/store";
import { DIFFICULTY_PRESETS, rollRngSeed } from "@/lib/difficulty";
```

  Edit 2 — `SelectionItem` (lines 29-34) gains two optional render fields, and the row
  renderer (lines 90-116) renders them:

```tsx
/** One row in a Guix-style listbox. */
interface SelectionItem {
  key: string;
  label: string;
  sublabel?: string;
  /** Right-aligned extra content (per-row actions); its buttons must stopPropagation. */
  trailing?: React.JSX.Element;
  /** Render at reduced opacity (archived operations). */
  dimmed?: boolean;
}
```

```tsx
      {items.map((item) => {
        const active = item.key === activeKey;
        return (
          <div
            key={item.key}
            role="option"
            aria-selected={active}
            data-testid={`${testIdPrefix}-${item.key}`}
            onClick={() => {
              onSelect(item.key);
              onActivate(item.key);
            }}
            className="flex w-full cursor-pointer items-center justify-between px-3 py-2 text-[13px]"
            style={{
              background: active ? GOLD : FIELD,
              color: active ? SHADOW : INK,
              opacity: item.dimmed ? 0.55 : 1,
            }}
          >
            <span className="font-semibold">{item.label}</span>
            <span className="flex items-center gap-3">
              {item.sublabel && (
                <span className="font-mono text-[11px]" style={{ opacity: active ? 0.85 : 0.7 }}>
                  {item.sublabel}
                </span>
              )}
              {item.trailing}
            </span>
          </div>
        );
      })}
```

  Edit 3 — `LobbyRoute` state + handlers (lines 121-162 region). Add the two new store
  selectors, difficulty + arm-to-confirm state, the scenario-name join, and the preset-
  aware create (navigation target stays `/game/${id}` in this task):

```tsx
  const deleteGame = useStore((s) => s.session.deleteGame);
  const archiveGame = useStore((s) => s.session.archiveGame);

  // Curated difficulty (spec-116 FR-116-3): vetted preset keys only,
  // never raw defines. "cadre" = schema defaults.
  const [selectedDifficulty, setSelectedDifficulty] = useState("cadre");

  // Arm-to-confirm for the permanent delete: first click arms, second fires.
  const [pendingDeleteId, setPendingDeleteId] = useState("");

  // Human scenario names live in the catalog; rows fall back to the raw key.
  const scenarioNames = new Map(scenarios.map((s) => [s.key, s.name]));

  async function handleCreate(): Promise<void> {
    if (!effectiveScenario) return;
    const preset = DIFFICULTY_PRESETS.find((p) => p.key === selectedDifficulty);
    setCreating(true);
    const id = await createGame({
      scenario: effectiveScenario,
      defines: preset?.defines ?? {},
      rng_seed: rollRngSeed(),
    });
    setCreating(false);
    if (id) navigate(`/game/${id}`);
  }

  function handleDeleteClick(id: string): void {
    if (pendingDeleteId === id) {
      setPendingDeleteId("");
      void deleteGame(id);
    } else {
      setPendingDeleteId(id);
    }
  }
```

  Edit 4a — the New Operation plate (after the scenario `SelectionList`, before the
  create button, lines 202-210 region) gains the difficulty picker:

```tsx
            <div>
              <p
                className="mb-1 text-[10px] uppercase tracking-[0.2em]"
                style={{ color: MUTED_LIGHT }}
              >
                Difficulty
              </p>
              <SelectionList
                items={DIFFICULTY_PRESETS.map((p) => ({
                  key: p.key,
                  label: p.label,
                  sublabel: p.description,
                }))}
                activeKey={selectedDifficulty}
                onSelect={setSelectedDifficulty}
                onActivate={setSelectedDifficulty}
                testIdPrefix="difficulty-option"
                emptyText="No presets."
              />
            </div>
```

  Edit 4b — the Your Games plate rows (lines 254-267) become codename-labeled with the
  status/date sublabel and per-row actions (the `game-option-${id}` testid contract and
  keyboard navigation are unchanged — they live in `SelectionList`):

```tsx
            {!gamesLoading && (
              <SelectionList
                items={games.map((g) => ({
                  key: g.id,
                  label: g.codename,
                  sublabel: `${scenarioNames.get(g.scenario) ?? g.scenario} · Tick ${g.current_tick} · ${g.status.toUpperCase()} · ${g.created_at.slice(0, 10)}`,
                  dimmed: g.status === "abandoned",
                  trailing: (
                    <span className="flex items-center gap-2">
                      {g.status !== "abandoned" && (
                        <button
                          data-testid={`game-archive-${g.id}`}
                          title="Archive operation (reversible)"
                          onClick={(e) => {
                            e.stopPropagation();
                            void archiveGame(g.id);
                          }}
                          className="border px-1.5 text-[10px] uppercase tracking-[0.1em]"
                          style={{
                            borderColor: MUTED,
                            color: MUTED_LIGHT,
                            background: "transparent",
                          }}
                        >
                          archive
                        </button>
                      )}
                      <button
                        data-testid={`game-delete-${g.id}`}
                        title="Delete operation (permanent)"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDeleteClick(g.id);
                        }}
                        className="border px-1.5 text-[10px] font-bold uppercase tracking-[0.1em]"
                        style={{
                          borderColor: CRIMSON,
                          color: pendingDeleteId === g.id ? SHADOW : CRIMSON,
                          background: pendingDeleteId === g.id ? CRIMSON : "transparent",
                        }}
                      >
                        {pendingDeleteId === g.id ? "confirm" : "×"}
                      </button>
                    </span>
                  ),
                }))}
                activeKey={effectiveGameId}
                onSelect={setFocusedGameId}
                onActivate={(id) => navigate(`/game/${id}`)}
                testIdPrefix="game-option"
                emptyText="No operations on record — start one above."
              />
            )}
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cd src/frontend && npx vitest run src/routes/LobbyRoute.test.tsx src/store/slices/sessionSlice.test.ts
```

Expected: all tests pass (7 LobbyRoute + 13 sessionSlice). Then the full frontend gate
(tsc catches any `GameSummary.codename` consumer misses that vitest's esbuild transform
would not):

```bash
mise run web:check
```

Expected: tsc + eslint + prettier + full vitest suite green.

- [ ] **Step 6: Commit**

```bash
git add src/frontend/src/types/game.ts src/frontend/src/api/endpoints.ts src/frontend/src/api/client.ts src/frontend/src/lib/difficulty.ts src/frontend/src/store/slices/sessionSlice.ts src/frontend/src/routes/LobbyRoute.tsx src/frontend/src/test/fixtures.ts src/frontend/src/test/handlers.ts src/frontend/src/store/slices/sessionSlice.test.ts src/frontend/src/routes/LobbyRoute.test.tsx
mise run commit -- "feat(frontend): lobby rebuild — codenames, status/date rows, delete/archive, curated difficulty (spec-116 FR-116-3)"
```

### Task 12: Scenario Briefing interstitial + first-session flow + e2e

**Files:**
- Create: `src/frontend/src/routes/BriefingRoute.tsx`
- Create: `src/frontend/src/routes/BriefingRoute.test.tsx`
- Create: `src/frontend/e2e/lobby-briefing.spec.ts`
- Modify: `src/frontend/src/App.tsx:30-44` (briefing route above `/game/:id`)
- Modify: `src/frontend/src/routes/LobbyRoute.tsx` `handleCreate` (navigate to briefing)
- Modify: `src/frontend/src/routes/LobbyRoute.test.tsx:14-24, 46-51` (briefing stub route +
  create-test assertion)
- Modify: `src/frontend/src/types/game.ts` (`GameDetailData` after `GameSummary`)
- Modify: `src/frontend/src/test/handlers.ts` (GET `/api/games/:id/` detail handler)
- Modify: `src/frontend/playwright.config.ts:30-35` (`AUTHENTICATED_SPECS` + new spec)

**Interfaces:**
- Consumes: Task 10's `codename` on `GET /api/games/{id}/`; the existing
  `GET /api/games/{id}/objectives/` (`get_journal_objectives` — 5 objectives with ids
  `revolution | ecological_collapse | fascist_consolidation | red_ogv |
  fragmented_collapse`; per the interface ledger its progress values become the
  detector-derived `axis_progress()` under Cluster A, but the payload SHAPE
  (`ObjectivesTracker`) and ids are unchanged, so this task is independent of Cluster A's
  landing order); Task 11's lobby (`difficulty-option-*` testids in e2e).
- Produces:
  - Route `/game/:id/briefing` + `BriefingRoute` (NO `useHeartbeat` — polling/autopause
    must not start before Begin Operation; recon gotcha).
  - `GameDetailData` interface (`types/game.ts`) — local typed contract for the
    `Untyped` `gameDetail` manifest row (deliberately NOT retyping the manifest row:
    `game_detail` calls no bridge serializer, so a typed row would add an advisory
    "unverifiable" seam finding).
  - testids: `briefing-codename`, `briefing-horizon`, `briefing-pattern-${id}`,
    `briefing-win-badge`, `briefing-begin`.
  - e2e `lobby-briefing.spec.ts`, registered in `AUTHENTICATED_SPECS`.
- Fixed-horizon ruling compliance: all briefing copy frames a 100-year / 5,200-tick
  campaign and five recognized PATTERNS ("where the century can land"), with
  Revolutionary Victory named as the win condition — no win/lose termination language.
- `e2e/real-loop.spec.ts` is deliberately UNTOUCHED: its create-test asserts
  `toHaveURL(new RegExp(`/game/${gameId}`))`, which the briefing URL still matches, and
  every later serial test does `page.goto(`/game/${gameId}`)` directly (verified at
  lines 74, 81, 101, 133, 157).

- [ ] **Step 1: Test infra — detail handler + `GameDetailData`.**

  (a) `src/frontend/src/types/game.ts` — insert after `GameSummary` (line 27):

```ts
/** GET /api/games/:id/ response body (`game_detail`, web/game/api.py). */
export interface GameDetailData {
  id: string;
  codename: string;
  scenario: string;
  current_tick: number;
  status: GameStatus;
  created_at: string | null;
}
```

  (b) `src/frontend/src/test/handlers.ts` — after the archive handler added in Task 11
  (path-to-regexp `:id` matches exactly one segment, so this cannot shadow
  `/api/games/:id/state/` and friends):

```ts
  http.get("/api/games/:id/", ({ params }) => {
    logRequest("GET game-detail");
    return HttpResponse.json({
      status: "ok",
      data: {
        id: String(params.id),
        codename: "CRIMSON HARVEST",
        scenario: "wayne_county",
        current_tick: mockSnapshot.tick,
        status: "active",
        created_at: "2026-03-01T12:00:00Z",
      },
    });
  }),
```

- [ ] **Step 2: Write the failing tests.**

  (a) Create `src/frontend/src/routes/BriefingRoute.test.tsx`:

```tsx
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router";
import { http, HttpResponse } from "msw";
import { server } from "@/test/server";
import { BriefingRoute } from "./BriefingRoute";
import { resetStore } from "@/test/resetStore";
import { resetMockGameState, requestLog } from "@/test/handlers";
import { makeObjective, makeObjectivesTracker } from "@/test/fixtures";

beforeEach(() => {
  resetStore();
  resetMockGameState();
});

/** The five real objective ids `get_journal_objectives` emits, at tick 0. */
const FIVE_PATTERNS = makeObjectivesTracker({
  tick: 0,
  objectives: [
    makeObjective({ progress: 0.01 }),
    makeObjective({
      id: "ecological_collapse",
      title: "Ecological Collapse",
      category: "collapse",
      progress: 0.02,
    }),
    makeObjective({
      id: "fascist_consolidation",
      title: "Fascist Consolidation",
      category: "fascist",
      progress: 0.0,
    }),
    makeObjective({ id: "red_ogv", title: "Red OGV Trap", category: "red_ogv", progress: 0.0 }),
    makeObjective({
      id: "fragmented_collapse",
      title: "Fragmented Collapse",
      category: "fragmented",
      progress: 0.0,
    }),
  ],
});

function renderBriefing(): void {
  render(
    <MemoryRouter initialEntries={["/game/game-001/briefing"]}>
      <Routes>
        <Route path="/game/:id/briefing" element={<BriefingRoute />} />
        <Route path="/game/:id" element={<div>GAME SHELL</div>} />
      </Routes>
    </MemoryRouter>,
  );
}

describe("BriefingRoute", () => {
  it("renders the operation codename, scenario, and fixed-horizon stakes", async () => {
    renderBriefing();
    await waitFor(() =>
      expect(screen.getByTestId("briefing-codename")).toHaveTextContent(
        "OPERATION CRIMSON HARVEST",
      ),
    );
    expect(screen.getByText("Wayne County Organizer")).toBeInTheDocument();
    // Owner ruling 2026-07-17: 100-year fixed horizon, patterns not terminators.
    expect(screen.getByTestId("briefing-horizon").textContent).toContain("100 years");
    expect(screen.getByTestId("briefing-horizon").textContent).toContain("5,200");
  });

  it("lists all five patterns from the real objectives payload, win condition named", async () => {
    server.use(
      http.get("/api/games/:id/objectives/", () =>
        HttpResponse.json({ status: "ok", data: FIVE_PATTERNS }),
      ),
    );
    renderBriefing();
    await waitFor(() => expect(screen.getAllByTestId(/^briefing-pattern-/)).toHaveLength(5));

    const revolution = screen.getByTestId("briefing-pattern-revolution");
    expect(within(revolution).getByTestId("briefing-win-badge")).toBeInTheDocument();
    expect(within(revolution).getByText("Revolutionary Victory")).toBeInTheDocument();
    // Only the win condition carries the badge.
    expect(screen.getAllByTestId("briefing-win-badge")).toHaveLength(1);
  });

  it("Begin Operation hands off to the cockpit — and no heartbeat ran before it", async () => {
    renderBriefing();
    await waitFor(() => expect(screen.getByTestId("briefing-begin")).toBeInTheDocument());
    // The briefing must not start GameRoute's polling machinery (recon gotcha).
    expect(requestLog.filter((r) => r === "GET state")).toHaveLength(0);

    await userEvent.click(screen.getByTestId("briefing-begin"));

    expect(screen.getByText("GAME SHELL")).toBeInTheDocument();
  });
});
```

  (b) `src/frontend/src/routes/LobbyRoute.test.tsx` — `renderLobby` gains the briefing
  stub route, and the create test now lands there:

```tsx
function renderLobby(): void {
  render(
    <MemoryRouter initialEntries={["/lobby"]}>
      <Routes>
        <Route path="/lobby" element={<LobbyRoute />} />
        <Route path="/game/:id/briefing" element={<div>BRIEFING</div>} />
        <Route path="/game/:id" element={<div>GAME SHELL</div>} />
        <Route path="/login" element={<div>LOGIN</div>} />
      </Routes>
    </MemoryRouter>,
  );
}
```

```tsx
  it("creates a new game via the real /api/games/ POST and lands on the briefing", async () => {
    renderLobby();
    await waitFor(() => expect(screen.getByText("Wayne County Organizer")).toBeInTheDocument());
    await userEvent.click(screen.getByRole("button", { name: /new game/i }));
    await waitFor(() => expect(screen.getByText("BRIEFING")).toBeInTheDocument());
  });
```

  (Opening an EXISTING game from the list still navigates straight to `/game/:id` — the
  briefing is a creation landing only; the row-click test from Task 11 continues to assert
  GAME SHELL.)

- [ ] **Step 3: Run tests to verify they fail**

```bash
cd src/frontend && npx vitest run src/routes/BriefingRoute.test.tsx src/routes/LobbyRoute.test.tsx
```

Expected: BriefingRoute suite fails at import — `Failed to resolve import "./BriefingRoute"`;
the rewritten Lobby create test fails with `Unable to find an element with the text:
BRIEFING` (create still navigates to `/game/:id`).

- [ ] **Step 4: Write minimal implementation.**

  (a) Create `src/frontend/src/routes/BriefingRoute.tsx` (complete file):

```tsx
/**
 * Scenario Briefing — the post-create interstitial (spec-116 FR-116-3).
 *
 * Landed on from the lobby's create flow at `/game/:id/briefing`, BEFORE the
 * cockpit mounts: who you are (the Cadre Council), the stakes, and the five
 * terminal PATTERNS in plain language with the win condition named. Pattern
 * titles/descriptions/progress come verbatim from
 * `GET /api/games/:id/objectives/` (`get_journal_objectives`) — real data,
 * presented as stakes (tick-0 readings are honestly near zero).
 *
 * FIXED-HORIZON FRAMING (owner ruling 2026-07-17): the campaign runs 100
 * in-game years (5,200 weekly ticks). The five outcomes are patterns the
 * world settles into — never terminators — so the copy frames them as
 * "where the century can land", not win/lose conditions that end the game.
 *
 * Deliberately NOT GameRoute: no useHeartbeat/polling/autopause machinery
 * may start until the player clicks Begin Operation (recon gotcha).
 *
 * SKIN: Design Bible §9b "THE INSTALLER" — same plate treatment as
 * LobbyRoute; all colors via the `--ksbc-*` role tokens, no literal hex.
 */

import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router";
import { get as apiGet } from "@/api/client";
import { endpoints } from "@/api/endpoints";
import { useStore } from "@/store";
import type { GameDetailData } from "@/types/game";
import type { ObjectivesTracker } from "@/types/dialectic";

const FIELD = "var(--ksbc-field)";
const CRIMSON = "var(--ksbc-accent-crimson)";
const GOLD = "var(--ksbc-accent-gold)";
const INK = "var(--ksbc-ink)";
const MUTED = "var(--ksbc-muted-1)";
const MUTED_LIGHT = "var(--ksbc-muted-2)";
const SHADOW = "var(--ksbc-key-shadow)";

/** The named win condition among the five recognized patterns. */
const WIN_OBJECTIVE_ID = "revolution";

/** Crimson tab label sitting on a plate's top border (LobbyRoute's plate idiom). */
function PlateLabel({ children }: { children: string }): React.JSX.Element {
  return (
    <span
      className="absolute -top-[11px] left-6 px-2 text-[11px] font-bold uppercase tracking-[0.3em]"
      style={{ background: FIELD, color: CRIMSON }}
    >
      ┤ {children} ├
    </span>
  );
}

export function BriefingRoute(): React.JSX.Element {
  const { id: gameId } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const scenarios = useStore((s) => s.session.scenarios);
  const fetchScenarios = useStore((s) => s.session.fetchScenarios);

  const [detail, setDetail] = useState<GameDetailData | null>(null);
  const [objectives, setObjectives] = useState<ObjectivesTracker | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!gameId) return;
    if (scenarios.length === 0) void fetchScenarios();
    void (async () => {
      const [detailRes, objectivesRes] = await Promise.all([
        apiGet<GameDetailData>(endpoints.gameDetail.path({ id: gameId })),
        apiGet<ObjectivesTracker>(endpoints.objectives.path({ id: gameId })),
      ]);
      if (detailRes.status === "ok") setDetail(detailRes.data);
      else setError(detailRes.message ?? "Failed to load operation");
      if (objectivesRes.status === "ok") setObjectives(objectivesRes.data);
      else setError(objectivesRes.message ?? "Failed to load patterns");
    })();
  }, [gameId, scenarios.length, fetchScenarios]);

  if (!gameId) {
    return <div className="flex h-screen items-center justify-center text-laser">No game id.</div>;
  }

  const scenario = scenarios.find((s) => s.key === detail?.scenario);

  return (
    <div className="flex min-h-screen flex-col font-mono" style={{ background: FIELD }}>
      <div className="mx-auto flex w-full max-w-2xl flex-col gap-8 px-6 py-10">
        {/* Operation plate */}
        <section className="relative border-2 p-6" style={{ borderColor: CRIMSON }}>
          <PlateLabel>Scenario Briefing</PlateLabel>
          <h1
            data-testid="briefing-codename"
            className="mt-2 text-lg font-bold tracking-[3px]"
            style={{ color: GOLD }}
          >
            {detail ? `OPERATION ${detail.codename}` : "OPERATION —"}
          </h1>
          <p className="mt-1 text-[12px] font-bold" style={{ color: INK }}>
            {scenario?.name ?? detail?.scenario ?? ""}
          </p>
          <p className="mt-2 text-[12px] leading-relaxed" style={{ color: MUTED_LIGHT }}>
            {scenario?.description ?? ""}
          </p>
        </section>

        {/* Who you are */}
        <section className="relative border-2 p-6" style={{ borderColor: CRIMSON }}>
          <PlateLabel>Who You Are</PlateLabel>
          <p className="mt-2 text-[12px] leading-relaxed" style={{ color: INK }}>
            You are the Cadre Council — the collective leadership of the organization.
            You direct cadre, funds, and lines of struggle; the world answers with its
            own motion. The engine adjudicates the material consequences; nothing is
            scripted.
          </p>
        </section>

        {/* The stakes — fixed horizon */}
        <section className="relative border-2 p-6" style={{ borderColor: CRIMSON }}>
          <PlateLabel>The Stakes</PlateLabel>
          <p
            data-testid="briefing-horizon"
            className="mt-2 text-[12px] leading-relaxed"
            style={{ color: INK }}
          >
            The campaign runs 100 years — 5,200 weekly turns. Nothing ends early: as
            material conditions move, the world settles toward one of five recognized
            patterns. The campaign closes at the horizon, or when you choose to accept
            a pattern once it has locked in.
          </p>
        </section>

        {/* Five patterns, real data */}
        <section className="relative border-2 p-6" style={{ borderColor: CRIMSON }}>
          <PlateLabel>Five Ways the Century Can Land</PlateLabel>
          <div className="mt-2 flex flex-col gap-3">
            {(objectives?.objectives ?? []).map((obj) => (
              <div
                key={obj.id}
                data-testid={`briefing-pattern-${obj.id}`}
                className="border p-3"
                style={{ borderColor: obj.id === WIN_OBJECTIVE_ID ? GOLD : MUTED }}
              >
                <div className="flex items-center justify-between gap-2">
                  <span className="text-[12px] font-bold" style={{ color: INK }}>
                    {obj.title}
                  </span>
                  {obj.id === WIN_OBJECTIVE_ID && (
                    <span
                      data-testid="briefing-win-badge"
                      className="px-1.5 text-[10px] font-bold uppercase tracking-[0.15em]"
                      style={{ background: GOLD, color: SHADOW }}
                    >
                      the win condition
                    </span>
                  )}
                </div>
                <p className="mt-1 text-[11px] leading-relaxed" style={{ color: MUTED_LIGHT }}>
                  {obj.description}
                </p>
                <p className="mt-1 font-mono text-[10px]" style={{ color: MUTED_LIGHT }}>
                  current reading: {obj.progress.toFixed(2)}
                </p>
              </div>
            ))}
            {objectives !== null && objectives.objectives.length === 0 && (
              <p className="text-[12px]" style={{ color: MUTED_LIGHT }}>
                No patterns declared this session.
              </p>
            )}
          </div>
        </section>

        {error && (
          <p role="alert" className="m-0 text-[12px]" style={{ color: CRIMSON }}>
            {error}
          </p>
        )}

        <button
          data-testid="briefing-begin"
          onClick={() => navigate(`/game/${gameId}`)}
          className="self-start border-2 px-4 py-2 text-[11px] font-bold uppercase tracking-[0.2em] transition-transform active:translate-x-[2px] active:translate-y-[2px]"
          style={{
            background: CRIMSON,
            color: INK,
            borderColor: SHADOW,
            boxShadow: `3px 3px 0 0 ${SHADOW}`,
          }}
        >
          Begin Operation
        </button>
      </div>
    </div>
  );
}
```

  (b) `src/frontend/src/App.tsx` — import and mount the route ABOVE `/game/:id` (React
  Router ranks the static `briefing` segment higher regardless, but the explicit order
  documents intent):

```tsx
import { BriefingRoute } from "@/routes/BriefingRoute";
```

```tsx
      <Route
        path="/game/:id/briefing"
        element={isAuthed ? <BriefingRoute /> : <Navigate to="/login" replace />}
      />
      <Route
        path="/game/:id"
        element={isAuthed ? <GameRoute /> : <Navigate to="/login" replace />}
      />
```

  (c) `src/frontend/src/routes/LobbyRoute.tsx` — `handleCreate`'s navigation becomes:

```tsx
    if (id) navigate(`/game/${id}/briefing`);
```

- [ ] **Step 5: Run tests to verify they pass, then the full frontend gate**

```bash
cd src/frontend && npx vitest run src/routes/BriefingRoute.test.tsx src/routes/LobbyRoute.test.tsx
mise run web:check
```

Expected: 3 BriefingRoute + 7 LobbyRoute tests pass; full gate green.

- [ ] **Step 6: Write the e2e spec.** Create `src/frontend/e2e/lobby-briefing.spec.ts`:

```ts
/**
 * Lobby & Scenario Briefing e2e (spec-116 FR-116-3) — the first-session
 * flow: create with a curated difficulty preset, land on the briefing
 * (codename, five patterns, win condition, fixed-horizon copy), Begin
 * Operation into the cockpit, then manage the operation from the lobby
 * (archive -> abandoned, arm-and-confirm delete -> row gone).
 *
 * Serial like real-loop.spec.ts: later tests reuse the session the first
 * test created. Creates its OWN session (never shared across spec files —
 * game_turn UNIQUE(session_id, tick, org)). Runs on the
 * "chromium-authenticated" project (registered in AUTHENTICATED_SPECS).
 */
import { expect, test } from "./fixtures";

/** Session id created by the first test. */
let gameId = "";

test.describe("lobby & briefing (spec-116 FR-116-3)", () => {
  test.describe.configure({ mode: "serial" });

  test("creating an operation lands on the Scenario Briefing", async ({ page }) => {
    await page.goto("/lobby");
    await page.getByTestId("scenario-option-wayne_county").click();
    await page.getByTestId("difficulty-option-cadre").click();

    const [createResp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/api/games/") && r.request().method() === "POST",
        { timeout: 60000 },
      ),
      page.getByRole("button", { name: /new game/i }).click(),
    ]);
    expect(createResp.status()).toBe(201);
    const body = (await createResp.json()) as { data?: { session_id?: string } };
    gameId = body.data?.session_id ?? "";
    expect(gameId, "create response must carry data.session_id").toBeTruthy();

    await expect(page).toHaveURL(new RegExp(`/game/${gameId}/briefing`), { timeout: 15000 });

    // Codename is server-derived from the session UUID: two uppercase words.
    await expect(page.getByTestId("briefing-codename")).toHaveText(/^OPERATION [A-Z]+ [A-Z]+$/, {
      timeout: 15000,
    });
    // Five real patterns from get_journal_objectives, win condition named.
    await expect(page.locator('[data-testid^="briefing-pattern-"]')).toHaveCount(5, {
      timeout: 15000,
    });
    await expect(page.getByTestId("briefing-win-badge")).toBeVisible();
    // Fixed-horizon framing (owner ruling): a century, not a termination condition.
    await expect(page.getByTestId("briefing-horizon")).toContainText("100 years");

    await page.getByTestId("briefing-begin").click();
    await expect(page).toHaveURL(new RegExp(`/game/${gameId}$`), { timeout: 15000 });
    await expect(page.getByTestId("tick-value")).toHaveText("0", { timeout: 15000 });
  });

  test("the lobby row carries codename metadata; archive then delete retire it", async ({
    page,
  }) => {
    expect(gameId, "created-session test ran first").toBeTruthy();
    await page.goto("/lobby");

    const row = page.getByTestId(`game-option-${gameId}`);
    await expect(row).toBeVisible({ timeout: 10000 });
    await expect(row).toContainText(/Tick \d+/);
    await expect(row).toContainText("ACTIVE");

    // Archive: reversible soft delete — the row re-lists as ABANDONED.
    await page.getByTestId(`game-archive-${gameId}`).click();
    await expect(row).toContainText("ABANDONED", { timeout: 10000 });

    // Delete: arm-then-confirm, then the row is gone for good.
    await page.getByTestId(`game-delete-${gameId}`).click();
    await page.getByTestId(`game-delete-${gameId}`).click();
    await expect(row).toHaveCount(0, { timeout: 10000 });
  });
});
```

  Register it — `src/frontend/playwright.config.ts:30-35`:

```ts
const AUTHENTICATED_SPECS = [
  "real-loop.spec.ts",
  "end-turn-flow.spec.ts",
  "verb-submit.spec.ts",
  "event-popup.spec.ts",
  "lobby-briefing.spec.ts",
];
```

- [ ] **Step 7: Run the e2e locally** (live stack; single-flight per machine-safety rules —
      never in a parallel agent fan-out):

```bash
mise run web:dev
cd src/frontend && npx playwright test e2e/lobby-briefing.spec.ts e2e/real-loop.spec.ts
mise run web:stop
```

Expected: both specs pass — `lobby-briefing` proves the new flow; `real-loop` proves the
briefing handoff did not break the load-bearing create test (its URL regex tolerates the
`/briefing` suffix and its later serial tests goto `/game/:id` directly).

- [ ] **Step 8: Commit**

```bash
git add src/frontend/src/routes/BriefingRoute.tsx src/frontend/src/routes/BriefingRoute.test.tsx src/frontend/src/App.tsx src/frontend/src/routes/LobbyRoute.tsx src/frontend/src/routes/LobbyRoute.test.tsx src/frontend/src/types/game.ts src/frontend/src/test/handlers.ts src/frontend/e2e/lobby-briefing.spec.ts src/frontend/playwright.config.ts
mise run commit -- "feat(frontend): Scenario Briefing interstitial + first-session flow + e2e (spec-116 FR-116-3)"
```
## Cluster D — The Voice heartbeat (spec FR-116-4.1 + design §6)

CausalChainObserver is fully tested but wired nowhere; its only output is a
`logger.warning("[NARRATIVE_JSON]", ...)` line (`causal.py:201`). These two tasks give it a
frame-capture API, wire it per-session into `resolve_tick` (the exact `_session_endgame_detectors`
pattern), persist frames as deterministic `NarrationRecord` beats every tick, and surface them in
the narration panel + Wire strip. **Everything here is observer/serialization-layer: byte-safe by
construction, outside the tick hash. No graph or WorldState mutation anywhere in the new path;
`mise run qa:regression` must stay byte-identical (no ceremony).** No new bridge-serialized snapshot
wire keys are introduced (the narration endpoint's beat shape is unchanged, and no `get_wire_feed`/
snapshot key is added), so **no SeamEntry rows are needed** — the three gating seam checks
(map metrics, `tick_*` payloads, `_EVENT_SEVERITY`) are untouched; `mise run check:seams` is run in
Task 13 Step 15 to prove it. Frames are narration, NOT events: nothing is appended to
`new_state.events` (the `_EVENT_SEVERITY` sentinel enforces EventType-only there).

Deliberately deferred (surgical-changes): migrating `CRASH_THRESHOLD`/`BUFFER_SIZE` into a
`GameDefines` category. They are pre-existing class constants on an observer outside the tick hash,
this cluster adds **no new numeric coefficient** (the pulse frame renders deltas unconditionally,
no thresholds), and folding a defines-schema change + `defines.yaml` regeneration into this diff
couples it to the FR-116-1 ceremony for zero player-visible gain. Flagged for the plan's follow-ups
ledger.

### Task 13: CausalChainObserver frame capture → bridge heartbeat → NarrationRecord → narration endpoint

**Files:**
- Modify: `src/babylon/engine/observers/causal.py:85-135` (`__init__`, `on_simulation_start`,
  `on_tick`), `:174-202` (`_detect_shock_doctrine`), + new `latest_frames` property and
  `_capture_pulse_frame`/`_build_pulse_frame`/`_prune_emitted_windows` helpers
- Create: `web/game/causal_voice.py` (pure deterministic frame→beat templates; copy lives in
  module-level data constants)
- Modify: `web/game/engine_bridge.py:35` (import), `:87` (module cache, directly under
  `_session_endgame_detectors`), `:4655` (resolve_tick wiring, immediately after the
  EndgameDetector block and BEFORE `to_graph()` — the observer reads WorldState MODEL fields:
  `economy.imperial_rent_pool`, `economy.current_super_wage_rate`, `entities[*].p_revolution`),
  `:4736` (persist call, next to `_persist_tick_events_safe`), `:7277` (new
  `_persist_causal_beats_safe` helper, after `_persist_tick_events_safe`)
- Modify: `web/game/api.py:745-798` (`game_narration` — serve beats regardless of
  `BABYLON_LLM_NARRATOR`; conscious contract flip)
- Test: `tests/unit/engine/observers/test_causal_chain.py` (extend)
- Test: `tests/unit/web/test_causal_voice.py` (new)
- Test: `tests/unit/web/test_engine_bridge.py` (extend)
- Test: `tests/unit/web/test_narration_endpoint.py:61-97` (rewrite `TestFlagOff` — deliberate
  behavioral-contract flip, done in the red phase, never silently)

**Interfaces:**
- Consumes: nothing from other tasks. Coexists with Cluster A's EndgameDetector repurposing —
  the causal block is a self-contained sibling that never touches detector API.
- Produces (Task 14 + later Voice waves rely on these exact names):
  - `CausalChainObserver.latest_frames -> tuple[dict[str, object], ...]` — frames captured by the
    most recent `on_tick`: one `TICK_PULSE` frame every tick
    (`{"pattern": "TICK_PULSE", "tick": int, "deltas": {"pool"|"wage"|"p_rev": {"before": float, "after": float}}}`),
    plus the existing `SHOCK_DOCTRINE` frame (shape unchanged from `_build_frame`) on detection
    ticks, deduped so the same 3-tick window never re-emits.
  - `game.causal_voice`: `CausalBeatSpec(beat_id, headline, body, register)` (NamedTuple),
    `render_frame_beats(frames) -> list[CausalBeatSpec]`,
    `CAUSAL_MODEL_ID = "deterministic-causal-v1"`, `CAUSAL_PROMPT_VERSION` (12-hex content hash).
  - Beat identities (stable across refetches — the panel's `mergeBeats` dedups by id):
    `causal-pulse-t{tick}` (scope `"tick"`, register `"wire"`, headline
    `"The week's ledger, tick {tick}."`) and `causal-shock-t{crashTick}` (scope `"tick"`, register
    `"analysis"`, headline `"Shock, austerity, radicalization — the causal chain closed."`).
  - `GET /api/games/{id}/narration/` serves persisted beats unconditionally; server-side status is
    only `"ready"`/`"pending"` (`"offline"` remains the client's degradation state in
    `lib/narration/client.ts`).

- [ ] **Step 1: Write the failing observer tests** — append to
      `tests/unit/engine/observers/test_causal_chain.py` (existing `create_state` helper +
      `SimulationConfig()` idiom; existing caplog tests stay untouched and must stay green):

```python
# =============================================================================
# TEST FRAME-CAPTURE API (spec-116 FR-4.1 — the Voice heartbeat)
# =============================================================================


@pytest.mark.unit
class TestFrameCaptureApi:
    """Frames must exit through an API, not only the log line.

    Spec-116 FR-4.1: the web bridge needs a per-tick return path. ``latest_frames``
    exposes what the MOST RECENT ``on_tick`` captured: one TICK_PULSE delta frame
    every tick (the design-§6 heartbeat — never empty), plus a SHOCK_DOCTRINE
    pattern frame on detection ticks, deduped across the rolling buffer.
    """

    def test_latest_frames_empty_before_any_tick(self) -> None:
        from babylon.engine.observers.causal import CausalChainObserver

        observer = CausalChainObserver()
        observer.on_simulation_start(create_state(tick=0), SimulationConfig())
        assert observer.latest_frames == ()

    def test_pulse_frame_emitted_every_tick(self) -> None:
        from babylon.engine.observers.causal import CausalChainObserver

        observer = CausalChainObserver()
        observer.on_simulation_start(
            create_state(tick=0, pool=100.0, wage=0.20, p_rev=0.30), SimulationConfig()
        )
        observer.on_tick(
            create_state(tick=0), create_state(tick=1, pool=90.0, wage=0.20, p_rev=0.35)
        )

        frames = observer.latest_frames
        assert len(frames) == 1
        assert frames[0]["pattern"] == "TICK_PULSE"
        assert frames[0]["tick"] == 1
        assert frames[0]["deltas"] == {
            "pool": {"before": 100.0, "after": 90.0},
            "wage": {"before": 0.20, "after": 0.20},
            "p_rev": {"before": 0.30, "after": 0.35},
        }

    def test_frames_are_cleared_each_tick(self) -> None:
        from babylon.engine.observers.causal import CausalChainObserver

        observer = CausalChainObserver()
        observer.on_simulation_start(create_state(tick=0, pool=100.0), SimulationConfig())
        observer.on_tick(create_state(tick=0), create_state(tick=1, pool=95.0))
        observer.on_tick(create_state(tick=1), create_state(tick=2, pool=92.0))

        frames = observer.latest_frames
        assert len(frames) == 1  # only tick 2's pulse — tick 1's was cleared
        assert frames[0]["tick"] == 2
        assert frames[0]["deltas"]["pool"] == {"before": 95.0, "after": 92.0}

    def test_shock_frame_captured_alongside_pulse(self) -> None:
        from babylon.engine.observers.causal import CausalChainObserver

        observer = CausalChainObserver()
        observer.on_simulation_start(
            create_state(tick=10, pool=100.0, wage=0.20, p_rev=0.30), SimulationConfig()
        )
        observer.on_tick(
            create_state(tick=10), create_state(tick=11, pool=70.0, wage=0.20, p_rev=0.30)
        )
        observer.on_tick(
            create_state(tick=11), create_state(tick=12, pool=70.0, wage=0.15, p_rev=0.45)
        )

        patterns = [f["pattern"] for f in observer.latest_frames]
        assert patterns == ["TICK_PULSE", "SHOCK_DOCTRINE"]

    def test_shock_frame_not_reemitted_on_later_ticks(
        self, caplog: pytest.LogCaptureFixture
    ) -> None:
        """The rolling buffer re-matches the SAME (crash, austerity, radical)
        window for up to 3 more ticks — without a dedup key the same frame
        re-emits every tick (the recon-identified re-emission bug)."""
        from babylon.engine.observers.causal import CausalChainObserver

        observer = CausalChainObserver()
        observer.on_simulation_start(
            create_state(tick=10, pool=100.0, wage=0.20, p_rev=0.30), SimulationConfig()
        )
        observer.on_tick(
            create_state(tick=10), create_state(tick=11, pool=70.0, wage=0.20, p_rev=0.30)
        )
        with caplog.at_level(logging.WARNING):
            observer.on_tick(
                create_state(tick=11), create_state(tick=12, pool=70.0, wage=0.15, p_rev=0.45)
            )
            observer.on_tick(
                create_state(tick=12), create_state(tick=13, pool=70.0, wage=0.15, p_rev=0.45)
            )

        assert caplog.text.count("[NARRATIVE_JSON]") == 1  # backward-compat log, once
        patterns = [f["pattern"] for f in observer.latest_frames]
        assert patterns == ["TICK_PULSE"]  # tick 13: pulse only, no shock re-emit

    def test_on_simulation_start_resets_capture_state(self) -> None:
        from babylon.engine.observers.causal import CausalChainObserver

        observer = CausalChainObserver()
        observer.on_simulation_start(create_state(tick=0, pool=100.0), SimulationConfig())
        observer.on_tick(create_state(tick=0), create_state(tick=1, pool=95.0))
        assert observer.latest_frames

        observer.on_simulation_start(create_state(tick=0, pool=100.0), SimulationConfig())
        assert observer.latest_frames == ()
```

- [ ] **Step 2: Run to verify failure** —
      `mise run test:q -- tests/unit/engine/observers/test_causal_chain.py`
      Expected: 6 failures in `TestFrameCaptureApi`, each
      `AttributeError: 'CausalChainObserver' object has no attribute 'latest_frames'`; all
      pre-existing tests still pass.

- [ ] **Step 3: Implement the frame-capture API** in `src/babylon/engine/observers/causal.py`.
      Replace `__init__`, `on_simulation_start`, `on_tick`, `_detect_shock_doctrine` and add the
      new members (everything else, including `_build_frame` and the class constants, is
      untouched):

```python
    def __init__(self, logger: logging.Logger | None = None) -> None:
        """Initialize CausalChainObserver.

        Args:
            logger: Logger instance for narrative JSON output.
                Defaults to module-level logger if not provided.
        """
        self._logger: logging.Logger = logger or logging.getLogger(__name__)
        self._history: deque[TickSnapshot] = deque(maxlen=self.BUFFER_SIZE)
        # Spec-116 FR-4.1: frames captured by the MOST RECENT on_tick call
        # (rebound, not mutated, each tick — a previously returned tuple
        # stays valid), plus the emitted-window dedup set. The rolling
        # buffer re-matches the same 3-tick window for up to BUFFER_SIZE-3
        # further ticks; keying emissions by the window's crash tick (one
        # snapshot per tick => a window is fully identified by it) stops
        # cross-tick re-emission.
        self._tick_frames: list[dict[str, object]] = []
        self._emitted_crash_ticks: set[int] = set()

    @property
    def latest_frames(self) -> tuple[dict[str, object], ...]:
        """Frames captured during the most recent ``on_tick`` call.

        One ``TICK_PULSE`` delta frame per tick (from the second recorded
        snapshot onward — the spec-116 §6 heartbeat), plus a
        ``SHOCK_DOCTRINE`` frame on the tick a new pattern window is
        detected. Empty before the first tick and immediately after
        ``on_simulation_start``.

        Returns:
            Immutable snapshot of this tick's frames (possibly empty).
        """
        return tuple(self._tick_frames)

    def on_simulation_start(
        self,
        initial_state: WorldState,
        config: SimulationConfig,  # noqa: ARG002
    ) -> None:
        """Called when simulation begins.

        Clears the history buffer, the captured frames, and the
        emitted-window dedup set, then records the initial state as baseline.

        Args:
            initial_state: WorldState at tick 0.
            config: SimulationConfig for this run (unused).
        """
        self._history.clear()
        self._tick_frames = []
        self._emitted_crash_ticks.clear()
        self._record_snapshot(initial_state)

    def on_tick(
        self,
        previous_state: WorldState,  # noqa: ARG002
        new_state: WorldState,
    ) -> None:
        """Called after each tick completes with both states for delta analysis.

        Records the new state, captures this tick's pulse frame, and checks
        for the Shock Doctrine pattern. Detected frames are appended to
        ``latest_frames`` (and the shock frame is still logged with the
        ``[NARRATIVE_JSON]`` prefix for backward compatibility).

        Args:
            previous_state: WorldState before the tick (unused, history is internal).
            new_state: WorldState after the tick.
        """
        self._tick_frames = []
        self._record_snapshot(new_state)
        self._capture_pulse_frame()
        self._detect_shock_doctrine()
        self._prune_emitted_windows()

    def _capture_pulse_frame(self) -> None:
        """Append the per-tick TICK_PULSE delta frame (needs >= 2 snapshots)."""
        if len(self._history) < 2:
            return
        self._tick_frames.append(
            self._build_pulse_frame(self._history[-2], self._history[-1])
        )

    def _build_pulse_frame(
        self, prev: TickSnapshot, cur: TickSnapshot
    ) -> dict[str, object]:
        """Build the TICK_PULSE frame from two consecutive snapshots.

        Args:
            prev: The prior tick's snapshot.
            cur: This tick's snapshot.

        Returns:
            ``{"pattern": "TICK_PULSE", "tick": int, "deltas": {...}}`` with
            before/after pairs for pool, wage, and peak p_revolution.
        """
        return {
            "pattern": "TICK_PULSE",
            "tick": cur.tick,
            "deltas": {
                "pool": {"before": prev.pool, "after": cur.pool},
                "wage": {"before": prev.wage, "after": cur.wage},
                "p_rev": {"before": prev.p_rev, "after": cur.p_rev},
            },
        }

    def _detect_shock_doctrine(self) -> None:
        """Check history for Shock Doctrine pattern and emit if found.

        Pattern requirements (sequential):
        1. ECONOMIC_SHOCK: Pool drops >= 20% between ticks
        2. AUSTERITY_RESPONSE: Wage decreases in subsequent tick
        3. RADICALIZATION: P(Revolution) increases in subsequent tick

        A detected frame is logged at WARNING level (``[NARRATIVE_JSON]``,
        backward compat) AND appended to ``latest_frames``. Each 3-tick
        window emits at most once across its lifetime in the rolling buffer
        (dedup keyed by crash tick).
        """
        # Need at least 3 snapshots to detect the pattern
        if len(self._history) < 3:
            return

        history_list = list(self._history)

        # Check all possible 3-tick windows in the buffer
        max_window_size = len(history_list)
        for i in range(max_window_size - 2):
            snapshot_n = history_list[i]
            snapshot_n1 = history_list[i + 1]
            snapshot_n2 = history_list[i + 2]

            if snapshot_n.tick in self._emitted_crash_ticks:
                continue  # this window already emitted on an earlier tick

            # Check for pattern
            if self._is_shock_doctrine_pattern(snapshot_n, snapshot_n1, snapshot_n2):
                frame = self._build_frame(snapshot_n, snapshot_n1, snapshot_n2)
                self._logger.warning("[NARRATIVE_JSON] %s", json.dumps(frame))
                self._tick_frames.append(frame)
                self._emitted_crash_ticks.add(snapshot_n.tick)
                return  # Only emit once per detection

    def _prune_emitted_windows(self) -> None:
        """Drop dedup keys older than the buffer window (bounded memory).

        Once a crash tick has scrolled out of the rolling buffer it can
        never re-match, so its key is dead weight — over a 5200-tick
        campaign the set would otherwise grow without bound.
        """
        if not self._history:
            return
        oldest = self._history[0].tick
        self._emitted_crash_ticks = {
            t for t in self._emitted_crash_ticks if t >= oldest
        }
```

- [ ] **Step 4: Run to verify pass** —
      `mise run test:q -- tests/unit/engine/observers/test_causal_chain.py`
      Expected: all pass (existing caplog/schema/lifecycle tests included), `0 failed`.

- [ ] **Step 5: Commit** —
      `git add src/babylon/engine/observers/causal.py tests/unit/engine/observers/test_causal_chain.py`
      then `mise run commit -- "feat(observers): frame-capture API + tick-pulse frames + shock dedup on CausalChainObserver (spec-116 FR-4.1)"`

- [ ] **Step 6: Write the failing voice + bridge tests.** Create
      `tests/unit/web/test_causal_voice.py`:

```python
"""Deterministic causal voice — frame→beat rendering (spec-116 FR-4.1).

Pure-function tests: no Django, no DB, no engine. The templates are the
behavioral contract the narration panel and Wire render — pinned
byte-for-byte so a copy change is a conscious diff, never drift.
"""

from __future__ import annotations

import pytest

from game.causal_voice import (
    CAUSAL_MODEL_ID,
    CAUSAL_PROMPT_VERSION,
    CausalBeatSpec,
    render_frame_beats,
)


def _pulse_frame() -> dict[str, object]:
    return {
        "pattern": "TICK_PULSE",
        "tick": 12,
        "deltas": {
            "pool": {"before": 100.0, "after": 70.0},
            "wage": {"before": 0.20, "after": 0.20},
            "p_rev": {"before": 0.30, "after": 0.45},
        },
    }


def _shock_frame() -> dict[str, object]:
    """Literal frame matching CausalChainObserver._build_frame's shape."""
    return {
        "pattern": "SHOCK_DOCTRINE",
        "causal_graph": {
            "nodes": [
                {
                    "id": "shock_t10",
                    "type": "ECONOMIC_SHOCK",
                    "tick": 10,
                    "data": {"pool_before": 100.0, "pool_after": 70.0, "drop_percent": -30.0},
                },
                {
                    "id": "austerity_t11",
                    "type": "AUSTERITY_RESPONSE",
                    "tick": 11,
                    "data": {"wage_before": 0.20, "wage_after": 0.15},
                },
                {
                    "id": "radical_t12",
                    "type": "RADICALIZATION",
                    "tick": 12,
                    "data": {"p_rev_before": 0.30, "p_rev_after": 0.45},
                },
            ],
            "edges": [
                {"source": "shock_t10", "target": "austerity_t11", "relation": "TRIGGERS_REACTION"},
                {"source": "austerity_t11", "target": "radical_t12", "relation": "CAUSES_RADICALIZATION"},
            ],
        },
    }


@pytest.mark.unit
class TestPulseRendering:
    def test_pulse_renders_three_causal_sentences(self) -> None:
        beats = render_frame_beats([_pulse_frame()])

        assert beats == [
            CausalBeatSpec(
                beat_id="causal-pulse-t12",
                headline="The week's ledger, tick 12.",
                body=(
                    "The imperial rent pool fell from 100.00 to 70.00. "
                    "The super-wage rate held at 0.2000. "
                    "Peak revolutionary probability rose from 0.300 to 0.450."
                ),
                register="wire",
            )
        ]

    def test_rendering_is_deterministic(self) -> None:
        assert render_frame_beats([_pulse_frame()]) == render_frame_beats([_pulse_frame()])


@pytest.mark.unit
class TestShockRendering:
    def test_shock_renders_the_causal_chain(self) -> None:
        beats = render_frame_beats([_shock_frame()])

        assert len(beats) == 1
        beat = beats[0]
        assert beat.beat_id == "causal-shock-t10"
        assert beat.register == "analysis"
        assert beat.headline == "Shock, austerity, radicalization — the causal chain closed."
        assert beat.body == (
            "The rent pool crashed 30.0% at tick 10. "
            "In the aftermath the super-wage rate was cut from 0.2000 to 0.1500. "
            "Peak revolutionary probability climbed from 0.300 to 0.450 — "
            "the shock is being answered."
        )


@pytest.mark.unit
class TestContractLimits:
    def test_beat_ids_fit_the_64_char_column(self) -> None:
        beats = render_frame_beats([_pulse_frame(), _shock_frame()])
        assert all(len(b.beat_id) <= 64 for b in beats)
        # 5200-tick horizon worst case
        assert len(f"causal-pulse-t{5200}") <= 64

    def test_model_pins_fit_their_columns(self) -> None:
        assert CAUSAL_MODEL_ID == "deterministic-causal-v1"
        assert len(CAUSAL_MODEL_ID) <= 128
        assert len(CAUSAL_PROMPT_VERSION) == 12  # content hash, <= 32-char column
        int(CAUSAL_PROMPT_VERSION, 16)  # hex — raises if not

    def test_unknown_pattern_is_loud(self) -> None:
        with pytest.raises(ValueError, match="unknown causal frame pattern"):
            render_frame_beats([{"pattern": "NOT_A_PATTERN"}])
```

Append to `tests/unit/web/test_engine_bridge.py` (module-level `_make_mock_persistence` /
`_make_mock_new_state` helpers already exist; `TestHexStateProjection` is the django_db model):

```python
# ---------------------------------------------------------------------- #
# Spec-116 FR-4.1: the Voice heartbeat — CausalChainObserver wiring
# ---------------------------------------------------------------------- #


@pytest.mark.unit
class TestCausalHeartbeatWiring:
    """resolve_tick runs the per-session CausalChainObserver and persists its
    frames as deterministic NarrationRecord beats (spec-116 FR-4.1).
    Observer-layer only: no state/graph mutation, outside the tick hash."""

    _SID = uuid.UUID("dddddddd-bbbb-cccc-dddd-eeeeeeeeeeee")

    @patch("game.engine_bridge.step")
    def test_resolve_tick_caches_observer_per_session(self, mock_step: MagicMock) -> None:
        from game.engine_bridge import _session_causal_observers

        _session_causal_observers.clear()
        mock_persistence = _make_mock_persistence()
        mock_persistence.get_pending_turns.return_value = []
        mock_step.return_value = _make_mock_new_state()

        bridge = EngineBridge(mock_persistence)
        bridge.resolve_tick(self._SID)
        first = _session_causal_observers.get(self._SID)
        assert first is not None, "resolve_tick must create the per-session observer"

        bridge.resolve_tick(self._SID)
        assert _session_causal_observers.get(self._SID) is first, (
            "the SAME observer instance must survive across resolve_tick calls "
            "(the 5-tick history buffer lives on it)"
        )

    def test_persist_causal_beats_safe_never_raises_without_db(self) -> None:
        """Best-effort sibling contract (_persist_*_safe family): a DB failure
        — here pytest-django blocking access (no django_db mark) — is
        swallowed and logged, never fails tick resolution."""
        from game.engine_bridge import _persist_causal_beats_safe

        frame = {
            "pattern": "TICK_PULSE",
            "tick": 3,
            "deltas": {
                "pool": {"before": 100.0, "after": 90.0},
                "wage": {"before": 0.2, "after": 0.2},
                "p_rev": {"before": 0.3, "after": 0.3},
            },
        }
        _persist_causal_beats_safe(self._SID, 3, (frame,))  # must not raise


@pytest.mark.unit
@pytest.mark.django_db
class TestCausalHeartbeatPersistence:
    """The pulse beat lands in narration_record on every resolved tick."""

    _SID = uuid.UUID("eeeeeeee-bbbb-cccc-dddd-eeeeeeeeeeee")

    @patch("game.engine_bridge.step")
    def test_resolve_tick_persists_pulse_beat(self, mock_step: MagicMock) -> None:
        from game.causal_voice import CAUSAL_MODEL_ID, CAUSAL_PROMPT_VERSION
        from game.engine_bridge import _session_causal_observers
        from game.models import GameSession, NarrationRecord

        _session_causal_observers.clear()
        GameSession.objects.create(
            id=self._SID, scenario="default", current_tick=0, status="active"
        )
        mock_persistence = _make_mock_persistence()
        mock_persistence.get_pending_turns.return_value = []
        mock_step.return_value = _make_mock_new_state(tick=1)

        bridge = EngineBridge(mock_persistence)
        bridge.resolve_tick(self._SID)

        record = NarrationRecord.objects.get(
            session_id=self._SID, tick=1, beat_id="causal-pulse-t1"
        )
        assert record.scope == "tick"
        assert record.register == "wire"
        assert record.model_id == CAUSAL_MODEL_ID
        assert record.prompt_version == CAUSAL_PROMPT_VERSION
        assert record.degraded is False
        assert record.headline == "The week's ledger, tick 1."
```

- [ ] **Step 7: Run to verify failure** —
      `mise run test:q -- tests/unit/web/test_causal_voice.py tests/unit/web/test_engine_bridge.py`
      Expected: `test_causal_voice.py` errors at collection with
      `ModuleNotFoundError: No module named 'game.causal_voice'`; the three new bridge tests fail
      with `ImportError: cannot import name '_session_causal_observers' from 'game.engine_bridge'`;
      every pre-existing bridge test still passes.

- [ ] **Step 8: Implement the voice module + bridge wiring.**

Create `web/game/causal_voice.py` in full:

```python
"""Deterministic causal voice — frame→beat templates (spec-116 FR-4.1, design §6).

Renders ``CausalChainObserver`` frames into ``NarrationRecord`` beat specs
with fixed templates: pure data + pure functions — no Django, no engine
imports, no randomness. The same frame always renders the same bytes, so
beat ids stay stable across refetches (the panel's ``mergeBeats`` dedups by
id) and re-runs. Copy lives in module-level data constants, not
conditionals (spec-116 Constraints); the only branching is on the frame's
own before/after arithmetic.

LLM garnish (``NarrativeService``) remains a separate, flag-gated channel
(``BABYLON_LLM_NARRATOR``). Absent a model, these templates are what the
narration panel serves — nothing is ever empty (design §6).

:data:`CAUSAL_PROMPT_VERSION` is a content hash of the template constants —
the same pin discipline as the LLM path's ``prompt_version`` (Constitution
III.6): a copy edit changes the hash, so persisted rows carry the exact
template generation that rendered them.
"""

from __future__ import annotations

import hashlib
from collections.abc import Mapping, Sequence
from typing import Any, NamedTuple

#: ``NarrationRecord.model_id`` pin for deterministic causal beats.
CAUSAL_MODEL_ID: str = "deterministic-causal-v1"

_PULSE_HEADLINE: str = "The week's ledger, tick {tick}."

#: Per-metric sentence templates when the value moved ({verb} = rose|fell).
_PULSE_MOVED: dict[str, str] = {
    "pool": "The imperial rent pool {verb} from {before:.2f} to {after:.2f}.",
    "wage": "The super-wage rate {verb} from {before:.4f} to {after:.4f}.",
    "p_rev": "Peak revolutionary probability {verb} from {before:.3f} to {after:.3f}.",
}

#: Per-metric sentence templates when the value held exactly.
_PULSE_HELD: dict[str, str] = {
    "pool": "The imperial rent pool held at {after:.2f}.",
    "wage": "The super-wage rate held at {after:.4f}.",
    "p_rev": "Peak revolutionary probability held at {after:.3f}.",
}

_SHOCK_HEADLINE: str = "Shock, austerity, radicalization — the causal chain closed."

_SHOCK_BODY: str = (
    "The rent pool crashed {drop:.1f}% at tick {crash_tick}. "
    "In the aftermath the super-wage rate was cut from {wage_before:.4f} to {wage_after:.4f}. "
    "Peak revolutionary probability climbed from {p_before:.3f} to {p_after:.3f} — "
    "the shock is being answered."
)

#: Content hash of every template above — the deterministic prompt_version pin.
CAUSAL_PROMPT_VERSION: str = hashlib.sha256(
    "\n".join(
        [
            _PULSE_HEADLINE,
            *(_PULSE_MOVED[k] for k in sorted(_PULSE_MOVED)),
            *(_PULSE_HELD[k] for k in sorted(_PULSE_HELD)),
            _SHOCK_HEADLINE,
            _SHOCK_BODY,
        ]
    ).encode("utf-8")
).hexdigest()[:12]

#: Fixed render order for pulse sentences (deterministic bytes).
_METRIC_ORDER: tuple[str, ...] = ("pool", "wage", "p_rev")


class CausalBeatSpec(NamedTuple):
    """One rendered beat, ready for ``NarrationRecord`` persistence.

    :param beat_id: Deterministic id, <= 64 chars (``narration_record`` column).
    :param headline: Rendered headline text.
    :param body: Rendered body text (3 causal sentences).
    :param register: ``NarrationRecord.Register`` value — ``"wire"`` or ``"analysis"``.
    """

    beat_id: str
    headline: str
    body: str
    register: str


def render_frame_beats(frames: Sequence[Mapping[str, Any]]) -> list[CausalBeatSpec]:
    """Render observer frames into beat specs, preserving frame order.

    :param frames: ``CausalChainObserver.latest_frames`` for one tick.
    :returns: One :class:`CausalBeatSpec` per frame.
    :raises ValueError: On a frame pattern this voice has no template for —
        loud failure (Constitution III.11), surfaced by the caller's
        best-effort log, never a silently dropped frame.
    """
    beats: list[CausalBeatSpec] = []
    for frame in frames:
        pattern = frame.get("pattern")
        if pattern == "TICK_PULSE":
            beats.append(_render_pulse(frame))
        elif pattern == "SHOCK_DOCTRINE":
            beats.append(_render_shock(frame))
        else:
            raise ValueError(f"unknown causal frame pattern: {pattern!r}")
    return beats


def _render_pulse(frame: Mapping[str, Any]) -> CausalBeatSpec:
    """Render a TICK_PULSE frame into the per-tick heartbeat beat."""
    tick = int(frame["tick"])
    deltas: Mapping[str, Mapping[str, Any]] = frame["deltas"]
    sentences: list[str] = []
    for metric in _METRIC_ORDER:
        before = float(deltas[metric]["before"])
        after = float(deltas[metric]["after"])
        if after == before:
            sentences.append(_PULSE_HELD[metric].format(after=after))
        else:
            verb = "rose" if after > before else "fell"
            sentences.append(
                _PULSE_MOVED[metric].format(verb=verb, before=before, after=after)
            )
    return CausalBeatSpec(
        beat_id=f"causal-pulse-t{tick}",
        headline=_PULSE_HEADLINE.format(tick=tick),
        body=" ".join(sentences),
        register="wire",
    )


def _render_shock(frame: Mapping[str, Any]) -> CausalBeatSpec:
    """Render a SHOCK_DOCTRINE frame into the pattern-analysis beat."""
    nodes = {n["type"]: n for n in frame["causal_graph"]["nodes"]}
    shock = nodes["ECONOMIC_SHOCK"]
    austerity = nodes["AUSTERITY_RESPONSE"]
    radical = nodes["RADICALIZATION"]
    crash_tick = int(shock["tick"])
    body = _SHOCK_BODY.format(
        drop=abs(float(shock["data"]["drop_percent"])),
        crash_tick=crash_tick,
        wage_before=float(austerity["data"]["wage_before"]),
        wage_after=float(austerity["data"]["wage_after"]),
        p_before=float(radical["data"]["p_rev_before"]),
        p_after=float(radical["data"]["p_rev_after"]),
    )
    return CausalBeatSpec(
        beat_id=f"causal-shock-t{crash_tick}",
        headline=_SHOCK_HEADLINE,
        body=body,
        register="analysis",
    )
```

Then four edits to `web/game/engine_bridge.py`:

(a) Line 35 — extend the observers import:

```python
from babylon.engine.observers import CausalChainObserver, EndgameDetector
```

(b) Directly below the `_session_endgame_detectors` dict (line 87):

```python
# Per-session CausalChainObserver instance (in-memory, not persisted).
# Spec-116 FR-4.1 (the Voice heartbeat): the rolling 5-tick history buffer
# must survive across separate ``resolve_tick`` HTTP calls, exactly like
# ``_session_endgame_detectors`` above. Same known limitation: per-process
# only, lost on worker restart (the buffer restarts empty and an in-flight
# shock window spanning the restart is missed — accepted, as for the
# detector), not shared across horizontally-scaled replicas.
_session_causal_observers: dict[UUID, CausalChainObserver] = {}
```

(c) In `resolve_tick`, immediately after the EndgameDetector block (after line 4654's
`model_copy` close-paren, before the `# Persist the new tick` comment):

```python
        # Spec-116 FR-4.1 (the Voice heartbeat): run the CausalChainObserver,
        # cached per-session like the EndgameDetector above. It reads
        # WorldState MODEL fields (economy.imperial_rent_pool,
        # economy.current_super_wage_rate, entities[*].p_revolution), so it
        # must run HERE on state/new_state, before to_graph(). Observer
        # layer only — reads state, mutates nothing, outside the tick hash;
        # its frames ride _persist_causal_beats_safe below, never
        # new_state.events (frames are narration, not EventTypes — the
        # _EVENT_SEVERITY seam sentinel enforces that boundary).
        causal_observer = _session_causal_observers.get(session_id)
        if causal_observer is None:
            causal_observer = CausalChainObserver()
            causal_observer.on_simulation_start(state, sim_config)
            _session_causal_observers[session_id] = causal_observer
        causal_observer.on_tick(state, new_state)
        causal_frames = causal_observer.latest_frames
```

(d) Right after the `_persist_tick_events_safe(...)` call (line 4736):

```python
        # Spec-116 FR-4.1: land this tick's causal frames as deterministic
        # NarrationRecord beats (the same table the LLM path writes and
        # game_narration serves). Best-effort — never fails tick resolution.
        _persist_causal_beats_safe(session_id, new_state.tick, causal_frames)
```

(e) After `_persist_tick_events_safe`'s function body (~line 7277), the new helper:

```python
def _persist_causal_beats_safe(
    session_id: UUID,
    tick: int,
    frames: tuple[dict[str, Any], ...] | list[dict[str, Any]],
) -> None:
    """Best-effort write of a tick's causal frames as ``NarrationRecord`` beats.

    Spec-116 FR-4.1 (the Voice heartbeat): renders the per-tick
    ``CausalChainObserver`` frames through the deterministic templates in
    :mod:`game.causal_voice` and lands them in ``narration_record`` — the
    same table the LLM path writes (``NarrativeService._persist``) and
    ``GET /api/games/{id}/narration/`` serves. A synchronous Django-ORM
    write on the request thread (no background-thread machinery needed),
    mirroring :func:`_persist_hex_state_safe`'s never-raise contract — a
    narration-write failure must not fail tick resolution — and
    ``NarrativeService._persist``'s idempotent ``update_or_create`` keyed
    ``(session, tick, beat_id)``. Observer-layer only: outside the tick
    hash, touches neither state nor graph.

    Args:
        session_id: The game session UUID.
        tick: The tick the frames were captured on.
        frames: ``CausalChainObserver.latest_frames`` for this tick.
    """
    if not frames:
        return
    try:
        from django.db import transaction

        from game.causal_voice import (
            CAUSAL_MODEL_ID,
            CAUSAL_PROMPT_VERSION,
            render_frame_beats,
        )
        from game.models import GameSession, NarrationRecord

        beats = render_frame_beats(frames)
        if not beats:
            return
        with transaction.atomic():
            session = GameSession.objects.get(pk=session_id)
            for beat in beats:
                NarrationRecord.objects.update_or_create(
                    session=session,
                    tick=tick,
                    beat_id=beat.beat_id,
                    defaults={
                        "scope": NarrationRecord.Scope.TICK,
                        "subject_ref": None,
                        "headline": beat.headline,
                        "body": beat.body,
                        "register": beat.register,
                        "model_id": CAUSAL_MODEL_ID,
                        "prompt_version": CAUSAL_PROMPT_VERSION,
                        "degraded": False,
                        "error": "",
                    },
                )
    except Exception:  # noqa: BLE001 — diagnostic; never blocks tick resolution
        logger.exception(
            "Failed to persist causal narration beats session=%s tick=%d",
            session_id,
            tick,
        )
```

- [ ] **Step 9: Run to verify pass** —
      `mise run test:q -- tests/unit/web/test_causal_voice.py tests/unit/web/test_engine_bridge.py tests/unit/engine/observers/test_causal_chain.py`
      Expected: all pass, `0 failed`. (Pre-existing resolve_tick tests without `django_db` keep
      passing: the observer runs harmlessly on `MagicMock` states — `float(MagicMock()) == 1.0`,
      empty `entities` gives `p_rev = 0.0` — and the blocked DB write is swallowed by the
      never-raise contract.)

- [ ] **Step 10: Commit** —
      `git add web/game/causal_voice.py web/game/engine_bridge.py tests/unit/web/test_causal_voice.py tests/unit/web/test_engine_bridge.py`
      then `mise run commit -- "feat(web): deterministic causal voice — per-session observer wiring + NarrationRecord heartbeat (spec-116 FR-4.1)"`

- [ ] **Step 11: Write the failing endpoint tests — the CONSCIOUS contract flip.** In
      `tests/unit/web/test_narration_endpoint.py`, replace the whole `TestFlagOff` class
      (lines 61–97) with (this deliberately inverts
      `test_flag_off_ignores_existing_records` — the old pin said flag-off hides records; the
      deterministic voice makes that a lie):

```python
class TestFlagIndependentReads:
    """Spec-116 FR-4.1 (the Voice heartbeat): persisted beats are served
    regardless of ``BABYLON_LLM_NARRATOR``. CONSCIOUS CONTRACT FLIP — the
    pre-116 contract (flag off ⇒ ``"offline"`` + records hidden) asserted
    the opposite; the deterministic causal voice writes records with the
    flag off, so the flag now gates ONLY LLM generation
    (``NarrativeService.schedule``), never the read path. Server-side
    status is ``"ready"``/``"pending"`` only; ``"offline"`` survives as the
    CLIENT's degradation state (``lib/narration/client.ts``)."""

    def test_flag_off_with_no_records_is_pending(self) -> None:
        client, session = _login_client_with_session()

        response = client.get(_narration_url(session.id))

        assert response.status_code == 200
        body = json.loads(response.content)
        assert body["status"] == "ok"
        assert body["data"] == {"status": "pending", "beats": []}

    def test_flag_off_serves_existing_records(self) -> None:
        """The heartbeat's whole point: deterministic beats reach the panel
        with the LLM flag off (design §6 — never empty)."""
        from game.models import NarrationRecord

        client, session = _login_client_with_session()
        NarrationRecord.objects.create(
            session=session,
            tick=1,
            beat_id="causal-pulse-t1",
            scope="tick",
            subject_ref=None,
            headline="The week's ledger, tick 1.",
            body="The imperial rent pool held at 100.00.",
            register="wire",
            model_id="deterministic-causal-v1",
            prompt_version="0" * 12,
        )

        response = client.get(_narration_url(session.id))

        data = json.loads(response.content)["data"]
        assert data["status"] == "ready"
        assert [b["id"] for b in data["beats"]] == ["causal-pulse-t1"]
        assert data["beats"][0]["register"] == "wire"
        assert data["beats"][0]["scope"] == "tick"
```

Also update the module docstring's flag-off sentence (line 7 area) to: `Beats are served
regardless of the BABYLON_LLM_NARRATOR flag (spec-116 FR-4.1); the flag gates only LLM
generation.`

- [ ] **Step 12: Run to verify failure** —
      `mise run test:q -- tests/unit/web/test_narration_endpoint.py`
      Expected: both new tests fail —
      `AssertionError: assert {'status': 'offline', 'beats': []} == {'status': 'pending', 'beats': []}`
      (and the seeded variant likewise gets `'offline'`). All `TestFlagOn*` tests still pass.

- [ ] **Step 13: Rework `game_narration`** (`web/game/api.py:745-798`). Delete the
      `from .narrative_service import is_enabled` import and the flag-gate early return; the view
      becomes:

```python
@api_view(["GET"])
@permission_classes([IsAuthenticated])
def game_narration(request: Request, game_id: str) -> JsonResponse:
    """GET /api/games/{id}/narration/?since_tick=N — narration beats.

    Program 20 Track B (task B5), reworked by spec-116 FR-4.1 (the Voice
    heartbeat): the deterministic causal voice (``game.causal_voice``,
    written synchronously every tick by ``resolve_tick`` via
    ``_persist_causal_beats_safe``) files ``NarrationRecord`` beats with or
    without a model, so this view serves persisted records UNCONDITIONALLY.
    ``BABYLON_LLM_NARRATOR`` now gates ONLY the LLM generation path
    (``NarrativeService.schedule``), never the read path — absent a model,
    templates render; nothing is ever empty (design §6).

    ``"ready"`` when records exist at/after ``since_tick``; ``"pending"``
    when none do yet (the narrator is live by construction, so the old
    flag-off ``"offline"`` answer is retired server-side — ``"offline"``
    remains the CLIENT's degradation state for failed requests, see
    ``src/frontend/src/lib/narration/client.ts``). Degraded beats are
    included, never filtered out (III.11).

    A non-integer ``since_tick`` is a loud 400 (III.11), never coerced to
    0; a missing ``since_tick`` defaults to 0 (full history).
    """
    session = _get_session_or_none(game_id, request.user.id)
    if session is None:
        return _error("Game not found", http_status=404)

    since_tick_query = request.query_params.get("since_tick")
    try:
        since_tick = int(since_tick_query) if since_tick_query is not None else 0
    except ValueError:
        return _error("Invalid since_tick parameter", http_status=400)

    records = session.narration_records.filter(tick__gte=since_tick).order_by("tick", "beat_id")
    beats = [
        {
            "id": r.beat_id,
            "tick": r.tick,
            "scope": r.scope,
            "subjectRef": r.subject_ref,
            "headline": r.headline,
            "body": r.body,
            "register": r.register,
        }
        for r in records
    ]
    data = {"status": "ready" if beats else "pending", "beats": beats}
    return _envelope(data, tick=session.current_tick, session_id=str(session.id))
```

- [ ] **Step 14: Run to verify pass** —
      `mise run test:q -- tests/unit/web/test_narration_endpoint.py tests/unit/web/test_narrative_service.py tests/unit/web/test_narration_record.py`
      Expected: all pass, `0 failed` (the service/persistence suites are unaffected — generation
      stays flag-gated; only the read path changed).

- [ ] **Step 15: Full gate + byte-safety proof, then commit** — run `mise run check`
      (includes `check:seams`: expected clean — no new bridge-serialized wire key exists, the
      gating sensors are untouched) and `mise run qa:regression`
      (expected: `Results: 5 passed, 0 failed` — the in-memory engine path never instantiates
      this observer and nothing in the new path mutates state; **if ANY value moves, STOP — that
      is a bug in this task, never a ceremony**). Then
      `git add web/game/api.py tests/unit/web/test_narration_endpoint.py`
      and `mise run commit -- "feat(web): serve narration beats regardless of LLM flag — conscious contract flip (spec-116 FR-4.1)"`

### Task 14: Heartbeat surfacing — narration panel + Wire strip fixtures and contract pins

Zero production frontend changes are required — that is the point, and this task **proves** it:
`worldSlice.onTickAdvanced` already fans out to the always-mounted narration panel (EventTray is
its canonical host), `narrationPanel.fetch` already cursors with `since_tick` + `mergeBeats` id
dedup (Task 13's deterministic beat ids make the inclusive-`tick__gte` refetch idempotent), and
`WireApp`'s narrator strip already renders the latest `scope === "tick"` beat. The beat wire shape
is unchanged, so `types/narration.ts` gains nothing (no phantom fields — the disease this spine
cures). The work: honest causal-beat fixtures in the MSW mocks + three contract pins that would
catch any regression in the flow. Frontend-only: byte-safe, no engine surface, outside the tick
hash.

**Files:**
- Modify: `src/frontend/src/mocks/narration/fixtures.ts` (add + export two causal fixture beats,
  text byte-identical to Task 13's templates)
- Test: `src/frontend/src/hooks/useNarration.test.ts` (extend — causal beats flow through the
  panel)
- Test: `src/frontend/src/components/chrome/EventTray.test.tsx` (extend — narrator slot renders
  the pulse beat)
- Test: `src/frontend/src/components/takeovers/TakeoverOverlay.test.tsx` (extend — Wire narrator
  strip renders the shock beat via the `scope === "tick"` filter)

**Interfaces:**
- Consumes (from Task 13): beat ids `causal-pulse-t{tick}` / `causal-shock-t{crashTick}`, scope
  `"tick"`, registers `"wire"` / `"analysis"`, headline/body template text, and the
  `"ready"`/`"pending"` server-status semantics.
- Produces: `CAUSAL_PULSE_FIXTURE_BEAT: NarrationBeat` and `CAUSAL_SHOCK_FIXTURE_BEAT:
  NarrationBeat` exported from `@/mocks/narration/fixtures` (available to later Voice-wave and
  e2e work).

- [ ] **Step 1: Write the failing tests.** All three import the not-yet-existing fixture exports,
      so all three fail loudly until Step 3.

Append to `src/frontend/src/hooks/useNarration.test.ts` (inside the existing
`describe("useNarration")`; add `CAUSAL_PULSE_FIXTURE_BEAT`/`CAUSAL_SHOCK_FIXTURE_BEAT` to the
existing `@/mocks/narration/fixtures` import — the file currently imports only from
`@/mocks/narration/handlers`, so add
`import { CAUSAL_PULSE_FIXTURE_BEAT, CAUSAL_SHOCK_FIXTURE_BEAT } from "@/mocks/narration/fixtures";`):

```tsx
  it("carries the deterministic causal heartbeat beats (spec-116 FR-4.1)", async () => {
    const { result } = renderHook(() => useNarration(DEFAULT_GAME_ID));
    await waitFor(() => {
      expect(result.current.status).toBe("ready");
    });

    const causal = result.current.beats.filter((b) => b.id.startsWith("causal-"));
    expect(causal.map((b) => b.id)).toEqual([
      CAUSAL_PULSE_FIXTURE_BEAT.id,
      CAUSAL_SHOCK_FIXTURE_BEAT.id,
    ]);
    // scope "tick" is what WireApp's narrator strip filters on
    expect(causal.every((b) => b.scope === "tick")).toBe(true);
    expect(causal.map((b) => b.register)).toEqual(["wire", "analysis"]);
  });
```

Append to `src/frontend/src/components/chrome/EventTray.test.tsx` (add
`import { CAUSAL_PULSE_FIXTURE_BEAT } from "@/mocks/narration/fixtures";`):

```tsx
  it("renders the causal heartbeat in the narrator slot (spec-116 FR-4.1)", () => {
    useStore.setState((s) => ({
      panels: {
        ...s.panels,
        narration: {
          ...s.panels.narration,
          status: "ready" as const,
          beats: [CAUSAL_PULSE_FIXTURE_BEAT],
        },
      },
    }));

    render(<EventTray gameId={DEFAULT_GAME_ID} />);

    const slot = screen.getByTestId("event-tray-narration");
    expect(slot).toHaveTextContent("The week's ledger, tick 104.");
    expect(screen.getByTestId("narration-block")).toHaveAttribute("data-register", "wire");
  });
```

Append to `src/frontend/src/components/takeovers/TakeoverOverlay.test.tsx` (add
`import { CAUSAL_SHOCK_FIXTURE_BEAT } from "@/mocks/narration/fixtures";`):

```tsx
  it("renders the causal shock beat in the Wire narrator strip (spec-116 FR-4.1)", async () => {
    useStore.setState((s) => ({
      panels: {
        ...s.panels,
        narration: {
          ...s.panels.narration,
          status: "ready" as const,
          beats: [CAUSAL_SHOCK_FIXTURE_BEAT],
        },
      },
    }));
    useStore.getState().ui.openTakeover("wire");
    render(<TakeoverOverlay gameId={DEFAULT_GAME_ID} />);

    const strip = await screen.findByTestId("wire-narrator-strip");
    expect(strip).toHaveTextContent(
      "Shock, austerity, radicalization — the causal chain closed.",
    );
    expect(strip.querySelector('[data-register="analysis"]')).not.toBeNull();
  });
```

- [ ] **Step 2: Run to verify failure** —
      `cd src/frontend && npx vitest run src/hooks/useNarration.test.ts src/components/chrome/EventTray.test.tsx src/components/takeovers/TakeoverOverlay.test.tsx`
      Expected: all three files fail at import with
      `SyntaxError: The requested module '/src/mocks/narration/fixtures.ts' does not provide an export named 'CAUSAL_PULSE_FIXTURE_BEAT'`
      (or the shock constant, per file).

- [ ] **Step 3: Add the causal fixture beats** to `src/frontend/src/mocks/narration/fixtures.ts`.
      Insert immediately BEFORE the two `beat-endgame-*` entries (keeps the existing
      `latest === "beat-endgame-analysis"` pin intact — these sit at ticks ≤ 104), and export the
      constants so tests reference one source of truth. Prose is byte-identical to
      `web/game/causal_voice.py`'s templates (honest mocks — the fixture IS the backend's output
      shape for the preview fiction's tick-104 crash):

```ts
/**
 * Spec-116 FR-4.1 — the deterministic causal voice (the Voice heartbeat).
 * Text renders `web/game/causal_voice.py`'s templates byte-for-byte over
 * the preview fiction's tick-102→104 rent-pool crash: a TICK_PULSE beat
 * every tick (register "wire") and a SHOCK_DOCTRINE analysis beat on the
 * detection tick, id keyed by the crash tick (`causal-shock-t102`, stable
 * across refetches). Both scope "tick" — WireApp's narrator strip filter.
 */
export const CAUSAL_PULSE_FIXTURE_BEAT: NarrationBeat = {
  id: "causal-pulse-t104",
  tick: 104,
  scope: "tick",
  subjectRef: null,
  headline: "The week's ledger, tick 104.",
  body: "The imperial rent pool fell from 118.40 to 96.10. The super-wage rate fell from 0.1920 to 0.1840. Peak revolutionary probability rose from 0.310 to 0.335.",
  register: "wire",
};

export const CAUSAL_SHOCK_FIXTURE_BEAT: NarrationBeat = {
  id: "causal-shock-t102",
  tick: 104,
  scope: "tick",
  subjectRef: null,
  headline: "Shock, austerity, radicalization — the causal chain closed.",
  body: "The rent pool crashed 21.4% at tick 102. In the aftermath the super-wage rate was cut from 0.1920 to 0.1840. Peak revolutionary probability climbed from 0.310 to 0.335 — the shock is being answered.",
  register: "analysis",
};
```

And inside `NARRATION_FIXTURE_BEATS`, before the `beat-endgame-dual-power` entry, append the two
new members (pulse first, then shock — the stable `mergeBeats` sort preserves this order among
equal-tick beats):

```ts
  CAUSAL_PULSE_FIXTURE_BEAT,
  CAUSAL_SHOCK_FIXTURE_BEAT,
```

- [ ] **Step 4: Run to verify pass, including the mocks-family guard** —
      `cd src/frontend && npx vitest run src/hooks/useNarration.test.ts src/components/chrome/EventTray.test.tsx src/components/takeovers/TakeoverOverlay.test.tsx src/mocks`
      Expected: all pass (the pre-existing `latest returns the newest beat by tick` pin still
      resolves to `beat-endgame-analysis` at tick 312; `browser.bundle-honesty` and the narration
      handler tests in `src/mocks` stay green — fixture additions only).

- [ ] **Step 5: Full frontend gate** — `cd src/frontend && npm run check`
      (tsc + eslint + prettier + full vitest; same legs as `mise run web:check`). Expected: clean.
      If prettier objects to the fixture formatting, run `npx prettier --write src/mocks/narration/fixtures.ts`
      and re-run.

- [ ] **Step 6: Commit** —
      `git add src/frontend/src/mocks/narration/fixtures.ts src/frontend/src/hooks/useNarration.test.ts src/frontend/src/components/chrome/EventTray.test.tsx src/frontend/src/components/takeovers/TakeoverOverlay.test.tsx`
      then `mise run commit -- "feat(frontend): pin the causal heartbeat through narration panel + Wire strip (spec-116 FR-4.1)"`
### Task 15: Six distinct endgame epilogues — backend data module + bridge payload

**Files:**
- Create: `web/game/epilogues.py`
- Create: `tests/unit/web/test_epilogues.py`
- Modify: `web/game/engine_bridge.py` (delete `_OUTCOME_HEADLINES` at :126-135; add sibling import near :63-64; add `_accepted_tick_from_endgame_row` after `_outcome_from_endgame_row` at :595-608; rework `get_endgame_state` at :3813-3858)
- Modify: `web/game/stub_bridge.py:1405-1419` (`StubEngineBridge.get_endgame_state` parity keys)
- Modify: `src/babylon/sentinels/seam/registry.py` (append `_ENDGAME_METRICS` section + add it to the `SEAM_REGISTRY` sum at the file tail)
- Modify: `specs/095-endgame-chronicle/contracts/endgame.yaml` (`EndgameState` schema: 3 new properties, `unresolved` in the outcome enum, headline description no longer the BUNKER-FAILS binary)
- Test (modify): `tests/unit/web/test_spec095_bridge.py` (extend after `TestGetEndgameState`, :166-231), `tests/unit/web/test_stub_bridge_parity.py:219-223`

**Interfaces:**
- Consumes: `GameOutcome.UNRESOLVED = "unresolved"` (Cluster A recognizer task — MUST land before this task; the coverage + parametrized tests below iterate the real enum). Durable ENDGAME `tick_event` row with `detail = {"outcome": <GameOutcome value>, ...}`; Cluster A's `POST /api/games/{id}/accept-outcome/` additionally stamps `detail["accepted_at_tick"]: int` on player-accept.
- Produces: `web/game/epilogues.py::Epilogue` (frozen pydantic: `headline: str`, `body: str`, `palette: Literal["rupture", "defeat", "unresolved"]`) and `EPILOGUES: dict[str, Epilogue]` keyed by **lowercase** `GameOutcome.value` (kills the old `.upper()` case split). `get_endgame_state` payload gains `epilogue: str`, `palette: str` (`""` while in progress), `accepted_at_tick: int | None`. Task 16 consumes all three; `_OUTCOME_HEADLINES` no longer exists.

Determinism: pure web-bridge serialization + a web-side data module — no engine, economics, or defines change. `qa:regression` is byte-identical by construction; no baseline regeneration.

- [ ] **Step 1: Write the failing data-module test**

  Create `tests/unit/web/test_epilogues.py`:

  ```python
  """Spec-116 FR-116-4.2: the six-epilogue data module contract.

  Kills the "THE BUNKER FAILS" x4 duplicate: every ``GameOutcome`` except
  ``IN_PROGRESS`` (including the fixed-horizon ``UNRESOLVED``) carries its own
  headline + body + palette, pairwise distinct (spec-116 acceptance gate 4).
  """

  from __future__ import annotations

  import pytest
  from pydantic import ValidationError

  from babylon.models.enums import GameOutcome
  from game.epilogues import EPILOGUES, Epilogue

  pytestmark = pytest.mark.unit


  class TestEpiloguesCoverage:
      """One epilogue per outcome — drift-safe against enum growth."""

      def test_covers_every_outcome_except_in_progress(self) -> None:
          expected = {o.value for o in GameOutcome} - {GameOutcome.IN_PROGRESS.value}
          assert set(EPILOGUES) == expected

      def test_unresolved_is_covered(self) -> None:
          # The sixth epilogue of the fixed-horizon ruling (owner 2026-07-17).
          assert "unresolved" in EPILOGUES


  class TestEpiloguesDistinctness:
      """Acceptance gate 4: every recognized outcome renders a DISTINCT epilogue."""

      def test_headlines_pairwise_distinct(self) -> None:
          headlines = [e.headline for e in EPILOGUES.values()]
          assert len(set(headlines)) == len(headlines)

      def test_bodies_pairwise_distinct(self) -> None:
          bodies = [e.body for e in EPILOGUES.values()]
          assert len(set(bodies)) == len(bodies)

      def test_the_bunker_fails_duplicate_is_dead(self) -> None:
          assert all(e.headline != "THE BUNKER FAILS" for e in EPILOGUES.values())

      def test_bodies_are_prose_not_labels(self) -> None:
          for outcome, entry in EPILOGUES.items():
              assert len(entry.body) >= 120, f"{outcome} body too short to be an epilogue"
              assert entry.body != entry.headline


  class TestEpiloguesPalettes:
      """Palette mapping: rupture for victory, unresolved for the open horizon."""

      def test_palette_mapping(self) -> None:
          assert EPILOGUES["revolutionary_victory"].palette == "rupture"
          assert EPILOGUES["unresolved"].palette == "unresolved"
          for outcome in (
              "ecological_collapse",
              "fascist_consolidation",
              "red_ogv",
              "fragmented_collapse",
          ):
              assert EPILOGUES[outcome].palette == "defeat"

      def test_epilogue_is_frozen(self) -> None:
          entry = EPILOGUES["revolutionary_victory"]
          with pytest.raises(ValidationError):
              entry.headline = "MUTATED"  # type: ignore[misc]

      def test_palette_literal_is_enforced(self) -> None:
          with pytest.raises(ValidationError):
              Epilogue(headline="X", body="Y" * 130, palette="triumph")  # type: ignore[arg-type]
  ```

- [ ] **Step 2: Run test to verify it fails**

  ```bash
  mise run test:q -- tests/unit/web/test_epilogues.py
  ```

  Expected: collection error, `ModuleNotFoundError: No module named 'game.epilogues'` (1 error, exit non-zero).

- [ ] **Step 3: Write the epilogues data module (the copy is data — verbatim below)**

  Create `web/game/epilogues.py`:

  ```python
  """Endgame epilogues — the six terminal texts of the hundred-year campaign.

  Spec-116 FR-116-4.2 (Playability Spine): kills the ``"THE BUNKER FAILS"`` x4
  duplicate by giving every :class:`~babylon.models.enums.events.GameOutcome`
  (including the fixed-horizon ``UNRESOLVED``) its own headline, body, and
  palette. Copy is **data** (spec-116 constraint: "copy lives in data, not
  conditionals"): the bridge looks the recognized pattern up at render time,
  the engine never imports this module, and the (flag-off) LLM narrator
  eulogizes through its own separate channel — the engine adjudicates, this
  module frames, the AI narrates.

  Source material: the three crafted (structurally unreachable) Wire triptychs
  in ``web/game/narrator.py`` (``revolutionary_victory`` /
  ``ecological_collapse`` / ``fascist_consolidation``) supplied the voice for
  those outcomes; the ``red_ogv`` / ``fragmented_collapse`` / ``unresolved``
  texts are original to this module (no prose existed anywhere for them).
  """

  from __future__ import annotations

  from typing import Literal

  from pydantic import BaseModel, ConfigDict

  from babylon.models.enums import GameOutcome


  class Epilogue(BaseModel):
      """One terminal outcome's end-screen copy.

      :ivar headline: The end screen's h1 (rendered in the outcome palette).
      :ivar body: Deterministic 2-4 sentence epilogue prose, distinct per outcome.
      :ivar palette: Which of the three end-screen palette families frames this
          outcome (``rupture`` bronze-gold / ``defeat`` laser-red /
          ``unresolved`` cold spire-cyan).
      """

      model_config = ConfigDict(frozen=True, extra="forbid")

      headline: str
      body: str
      palette: Literal["rupture", "defeat", "unresolved"]


  #: The six terminal texts, keyed by lowercase ``GameOutcome.value`` — the same
  #: case ``_outcome_from_endgame_row`` returns and the frontend compares,
  #: eliminating the old ``_OUTCOME_HEADLINES`` ``.upper()`` case split.
  EPILOGUES: dict[str, Epilogue] = {
      GameOutcome.REVOLUTIONARY_VICTORY.value: Epilogue(
          headline="BABYLON FALLS",
          body=(
              "The regime change is real: not a transfer of management but the "
              "end of the manager. The imperial circuit is broken — the rent "
              "that bought the core's silence has stopped arriving, and no one "
              "is owed silence anymore. The people hold the line they spent a "
              "century building. What the wire called impossible was only ever "
              "unprofitable."
          ),
          palette="rupture",
      ),
      GameOutcome.ECOLOGICAL_COLLAPSE.value: Epilogue(
          headline="THE EARTH BETRAYED",
          body=(
              "The crisis was never a surprise; it was a business plan running "
              "to completion. Capital metabolized forest, watershed, and season "
              "into quarterly filings until the biosphere stopped extending "
              "credit. There is no bunker deep enough to secede from a dead "
              "metabolism. The earth was betrayed by capital, and the earth "
              "does not negotiate."
          ),
          palette="defeat",
      ),
      GameOutcome.FASCIST_CONSOLIDATION.value: Epilogue(
          headline="ORDER IS RESTORED",
          body=(
              "That is what the wire calls it: order, restored. Wages fell, and "
              "the anger that could have become class war was routed into "
              "national costume — the oldest trick empire knows. The fash take "
              "hold of the state because the state was always shaped to receive "
              "them. We do not yield; the cadre goes under, and the work "
              "continues in the dark."
          ),
          palette="defeat",
      ),
      GameOutcome.RED_OGV.value: Epilogue(
          headline="RED FLAGS OVER EMPIRE",
          body=(
              "A socialist government now administers an unbroken imperial "
              "circuit. Core wages still exceed core value; the difference "
              "still arrives from the periphery — collected, as before, only "
              "now in the people's name. The settler bargain was not "
              "repudiated, it was rebranded. When the periphery presents its "
              "ledger, it will not distinguish between the empire's managers."
          ),
          palette="defeat",
      ),
      GameOutcome.FRAGMENTED_COLLAPSE.value: Epilogue(
          headline="THE MAP SHATTERS",
          body=(
              "The center failed and nothing organized enough replaced it. "
              "Sovereignty splintered faster than solidarity could bind it — "
              "three flags, then five, each defending a shrinking perimeter of "
              "rent. Where no class rules, geography rules. Collapse is not "
              "liberation; it is the empire's debris, still falling on the "
              "same people it always fell on."
          ),
          palette="defeat",
      ),
      GameOutcome.UNRESOLVED.value: Epilogue(
          headline="THE STRUGGLE CONTINUES",
          body=(
              "One hundred years, and no verdict. The contradiction did not "
              "resolve; it deepened, changed terrain, and outlived every "
              "administration that claimed to manage it. History does not end "
              "because the observation window closes. The line holds where you "
              "built it; the rest belongs to the next century, and to whoever "
              "organizes it."
          ),
          palette="unresolved",
      ),
  }
  ```

- [ ] **Step 4: Run test to verify it passes**

  ```bash
  mise run test:q -- tests/unit/web/test_epilogues.py
  ```

  Expected: `9 passed`.

- [ ] **Step 5: Write the failing bridge + stub-parity tests**

  In `tests/unit/web/test_spec095_bridge.py`: add `from typing import Any` and
  `from babylon.models.enums import GameOutcome` and
  `from game.epilogues import EPILOGUES` to the imports (:8-15), then append
  directly after `TestGetEndgameState` (after :231):

  ```python
  def _mock_persistence_with_endgame_row(
      detail: dict[str, Any], tick: int = 5200, summary: str = "Endgame Reached"
  ) -> MagicMock:
      """A mock persistence whose pool serves one durable endgame tick_event row.

      Same cursor-mock shape as ``test_returns_outcome_when_endgame_fires``:
      ``fetchone`` returns the positional ``(tick, detail, summary)`` tuple
      ``_fetch_endgame_event_row`` expects.
      """
      mock_persistence = MagicMock()
      mock_persistence.get_metadata.return_value = None
      mock_persistence.hydrate_graph.return_value = _mock_graph_with_contradictions()
      mock_persistence._pool = MagicMock()
      cursor = MagicMock()
      cursor.fetchone.return_value = (tick, detail, summary)
      cursor.__enter__ = MagicMock(return_value=cursor)
      cursor.__exit__ = MagicMock(return_value=False)
      conn = MagicMock()
      conn.cursor.return_value = cursor
      conn.__enter__ = MagicMock(return_value=conn)
      conn.__exit__ = MagicMock(return_value=False)
      mock_persistence._pool.connection.return_value = conn
      return mock_persistence


  class TestEndgameEpilogues:
      """Spec-116 FR-116-4.2: get_endgame_state serves the six distinct epilogues."""

      @pytest.mark.parametrize(
          "outcome",
          sorted({o.value for o in GameOutcome} - {GameOutcome.IN_PROGRESS.value}),
      )
      def test_each_outcome_serves_its_own_epilogue(self, outcome: str) -> None:
          mock_persistence = _mock_persistence_with_endgame_row(
              {"kind": "endgame_reached", "outcome": outcome}
          )
          bridge = EngineBridge(mock_persistence)

          result = bridge.get_endgame_state(_SESSION)

          assert result["outcome"] == outcome
          assert result["headline"] == EPILOGUES[outcome].headline
          assert result["epilogue"] == EPILOGUES[outcome].body
          assert result["palette"] == EPILOGUES[outcome].palette

      def test_accepted_at_tick_surfaces_from_detail(self) -> None:
          # FR-116-5: the accept-outcome endpoint stamps accepted_at_tick.
          mock_persistence = _mock_persistence_with_endgame_row(
              {
                  "kind": "endgame_reached",
                  "outcome": "fascist_consolidation",
                  "accepted_at_tick": 3120,
              }
          )
          bridge = EngineBridge(mock_persistence)

          result = bridge.get_endgame_state(_SESSION)

          assert result["accepted_at_tick"] == 3120

      def test_accepted_at_tick_is_none_without_player_accept(self) -> None:
          mock_persistence = _mock_persistence_with_endgame_row(
              {"kind": "endgame_reached", "outcome": "revolutionary_victory"}
          )
          bridge = EngineBridge(mock_persistence)

          result = bridge.get_endgame_state(_SESSION)

          assert result["accepted_at_tick"] is None

      def test_in_progress_serves_no_fabricated_copy(self) -> None:
          # Constitution III.11: a running game has no epilogue — never fabricate.
          mock_persistence = MagicMock()
          mock_persistence.get_metadata.return_value = None
          mock_persistence.hydrate_graph.return_value = _mock_graph_with_contradictions()
          bridge = EngineBridge(mock_persistence)

          result = bridge.get_endgame_state(_SESSION)

          assert result["headline"] == ""
          assert result["epilogue"] == ""
          assert result["palette"] == ""
          assert result["accepted_at_tick"] is None

      def test_outcome_headlines_dict_is_deleted(self) -> None:
          # FR-116-4.2 kills the x4 duplicate at its source, not around it.
          import game.engine_bridge as engine_bridge_module

          assert not hasattr(engine_bridge_module, "_OUTCOME_HEADLINES")
  ```

  In `tests/unit/web/test_stub_bridge_parity.py` replace :219-223:

  ```python
      def test_get_endgame_state(self) -> None:
          bridge, session_id = _stub_session()
          result = bridge.get_endgame_state(session_id)
          assert result["outcome"] is None
          assert "final_tick" in result["stats"]
  ```

  with:

  ```python
      def test_get_endgame_state(self) -> None:
          bridge, session_id = _stub_session()
          result = bridge.get_endgame_state(session_id)
          assert result["outcome"] is None
          assert "final_tick" in result["stats"]
          # Spec-116 FR-116-4.2 parity: the epilogue keys exist and are honest.
          assert result["epilogue"] == ""
          assert result["palette"] == ""
          assert result["accepted_at_tick"] is None
  ```

- [ ] **Step 6: Run tests to verify they fail**

  ```bash
  mise run test:q -- tests/unit/web/test_spec095_bridge.py tests/unit/web/test_stub_bridge_parity.py
  ```

  Expected: the 6 parametrized cases and the accepted/in-progress cases fail with
  `KeyError: 'epilogue'` / `KeyError: 'accepted_at_tick'`,
  `test_outcome_headlines_dict_is_deleted` fails with `AssertionError`
  (`_OUTCOME_HEADLINES` still exists), and the parity test fails with
  `KeyError: 'epilogue'` — ~10 failed, rest passed.

- [ ] **Step 7: Minimal implementation — bridge, stub, contract, seam registry**

  (a) `web/game/engine_bridge.py` — **delete** lines 126-135 (the whole
  `_OUTCOME_HEADLINES` block including its 3 comment lines). Add the sibling
  import next to the existing relative imports (after :63
  `from .log_handler import sanitize_for_log`):

  ```python
  from .epilogues import EPILOGUES
  ```

  (b) Insert after `_outcome_from_endgame_row` (:595-608):

  ```python
  def _accepted_tick_from_endgame_row(row: dict[str, Any] | None) -> int | None:
      """Extract the player-accept tick from an endgame row's ``detail`` blob.

      Spec-116 FR-116-5: ``POST /api/games/{id}/accept-outcome/`` stamps
      ``detail["accepted_at_tick"]`` when the player fast-forwards to the
      epilogue; a horizon-terminated game has no such key. ``bool`` is rejected
      explicitly (``True`` is an ``int`` in Python) rather than coerced.
      """
      if row is None:
          return None
      detail = row.get("detail")
      raw = detail.get("accepted_at_tick") if isinstance(detail, dict) else None
      if isinstance(raw, bool) or not isinstance(raw, int):
          return None
      return raw
  ```

  (c) Replace `get_endgame_state` (:3813-3858, the whole method) with:

  ```python
      def get_endgame_state(self, session_id: UUID) -> dict[str, Any]:
          """Return the terminal outcome + epilogue + chronicle stat cards.

          Spec 095 FR-095-02 + spec-116 FR-116-4.2. All six GameOutcome values
          (incl. the fixed-horizon ``unresolved``) resolve to a distinct
          epilogue from ``web/game/epilogues.py``. Returns ``outcome: None``
          (and empty copy — Constitution III.11) while the game is in progress.

          Args:
              session_id: The game session UUID.

          Returns:
              ``EndgameState`` dict matching
              ``specs/095-endgame-chronicle/contracts/endgame.yaml``.
          """
          graph = self._persistence.hydrate_graph(tick=None, session_id=session_id)
          graph_attrs: dict[str, Any] = getattr(graph, "graph", {}) or {}
          tick = int(graph_attrs.get("tick", 0))

          # Program 17 / Item 1c: WorldState.events is per-tick, not cumulative
          # — the latest graph's events list loses an endgame event the moment
          # even one more tick elapses. tick_event (durable, cumulative) is the
          # only correct source for "has this game ever ended".
          row = _fetch_endgame_event_row(getattr(self._persistence, "_pool", None), session_id)
          outcome = _outcome_from_endgame_row(row)
          summary = str(row.get("summary") or "") if row is not None and outcome else ""
          if row is not None and outcome:
              tick = int(row["tick"])

          # Spec-116 FR-116-4.2: EPILOGUES is keyed by lowercase GameOutcome
          # values — the same case _outcome_from_endgame_row returns — so no
          # .upper() case split. An unrecognized outcome string degrades to
          # empty copy rather than fabricated copy (III.11).
          entry = EPILOGUES.get(outcome) if outcome else None
          headline = entry.headline if entry is not None else ""
          epilogue = entry.body if entry is not None else ""
          palette: str = entry.palette if entry is not None else ""
          accepted_at_tick = _accepted_tick_from_endgame_row(row) if outcome else None

          consciousness_avg = _compute_avg_node_attr(graph, "class_consciousness", 0.0)
          heat_avg = _compute_avg_node_attr(graph, "heat", 0.0)
          solidarity_edges = _count_edges_by_mode(graph, frozenset({"solidarity"}))

          return {
              "tick": tick,
              "outcome": outcome,
              "headline": headline,
              "summary": summary,
              "epilogue": epilogue,
              "palette": palette,
              "accepted_at_tick": accepted_at_tick,
              "stats": {
                  "final_tick": tick,
                  "consciousness": consciousness_avg,
                  "solidarity_edges": solidarity_edges,
                  "heat": heat_avg,
              },
          }
  ```

  (d) `web/game/stub_bridge.py:1405-1419` — add the three parity keys to the
  return dict (after `"summary": "",`):

  ```python
              "epilogue": "",
              "palette": "",
              "accepted_at_tick": None,
  ```

  (e) `specs/095-endgame-chronicle/contracts/endgame.yaml` — in
  `components.schemas.EndgameState.properties`, replace the `outcome`,
  `headline`, and `summary` entries and add the three new ones:

  ```yaml
          outcome:
            type: string
            enum: [revolutionary_victory, ecological_collapse, fascist_consolidation, red_ogv, fragmented_collapse, unresolved]
            description: "Recognized outcome pattern at game over (spec-116 fixed horizon), or null while in progress."
          headline: { type: string, description: "Per-outcome epilogue headline (web/game/epilogues.py, spec-116 FR-116-4.2 — six distinct texts); empty while in progress." }
          summary: { type: string, description: "Degraded tick_event machine summary; superseded visually by `epilogue`, kept for contract compat." }
          epilogue: { type: string, description: "Deterministic 2-4 sentence epilogue body, pairwise distinct per outcome; empty while in progress." }
          palette:
            type: string
            enum: ["rupture", "defeat", "unresolved", ""]
            description: "End-screen palette family; empty while in progress."
          accepted_at_tick:
            type: [integer, "null"]
            description: "Tick at which the player accepted the locked pattern (FR-116-5 fast-forward); null for horizon termination or in progress."
  ```

  Also update the top-of-file `info.description` sentence "Recognizes all 5
  GameOutcome terminal types (…)" to "Recognizes all six GameOutcome values
  (REVOLUTIONARY_VICTORY, ECOLOGICAL_COLLAPSE, FASCIST_CONSOLIDATION, RED_OGV,
  FRAGMENTED_COLLAPSE, UNRESOLVED — spec-116 fixed horizon)."

  (f) `src/babylon/sentinels/seam/registry.py` — append a new section directly
  before the `SEAM_REGISTRY` closing sum, and add `+ _ENDGAME_METRICS` to the
  sum:

  ```python
  # ---------------------------------------------------------------------------
  # ENDGAME scope — spec-116 FR-116-4.2 epilogue keys on the
  # ``get_endgame_state`` payload (GET /api/games/{id}/endgame/).
  # Pre-existing keys (tick/outcome/headline/summary/stats) predate the
  # registry; only the Playability Spine's NEW wire keys are declared here.
  # ---------------------------------------------------------------------------

  _ENDGAME_READ_PATHS: tuple[str, ...] = (
      "web/game/engine_bridge.py::EngineBridge.get_endgame_state",
      "src/frontend/src/components/takeovers/chronicle/EndStateScreen.tsx",
  )

  _ENDGAME_METRICS: tuple[SeamEntry, ...] = (
      SeamEntry(
          payload="epilogue",
          wire_keys=("epilogue",),
          scope=SeamScope.ENDGAME,
          owner_layer="web bridge (game.epilogues data module)",
          liveness_class=LivenessClass.DECLARED_CONDITIONAL,
          liveness_condition=(
              "non-empty only once the durable ENDGAME tick_event row exists "
              "(horizon or player-accept); '' while the campaign runs"
          ),
          dtype="str",
          write_site="web/game/epilogues.py::EPILOGUES (data module, render-time lookup)",
          derivation_site="web/game/engine_bridge.py::EngineBridge.get_endgame_state",
          read_paths=_ENDGAME_READ_PATHS,
          nullable=False,
          spec_ref="specs/116-playability-spine/spec.md · FR-116-4.2",
          notes=(
              "Deterministic 2-4 sentence epilogue body, pairwise distinct across "
              "all six GameOutcome values incl. 'unresolved'. Deliberately separate "
              "from the LLM epitaph channel (NarrationRecord Scope.ENDGAME): the "
              "engine adjudicates, copy is data, AI narrates."
          ),
      ),
      SeamEntry(
          payload="palette",
          wire_keys=("palette",),
          scope=SeamScope.ENDGAME,
          owner_layer="web bridge (game.epilogues data module)",
          liveness_class=LivenessClass.DECLARED_CONDITIONAL,
          liveness_condition=(
              "same durable-ENDGAME-row gate as 'endgame.epilogue'; '' while the "
              "campaign runs"
          ),
          dtype="enum:EpiloguePalette",
          write_site="web/game/epilogues.py::EPILOGUES (data module, render-time lookup)",
          derivation_site="web/game/engine_bridge.py::EngineBridge.get_endgame_state",
          read_paths=_ENDGAME_READ_PATHS,
          nullable=False,
          spec_ref="specs/116-playability-spine/spec.md · FR-116-4.2",
          notes=(
              "One of 'rupture' | 'defeat' | 'unresolved' — drives the three "
              "end-screen palette families (six texts, three palettes)."
          ),
      ),
      SeamEntry(
          payload="accepted_at_tick",
          wire_keys=("accepted_at_tick",),
          scope=SeamScope.ENDGAME,
          owner_layer="web bridge (accept-outcome endpoint stamp)",
          liveness_class=LivenessClass.DECLARED_CONDITIONAL,
          liveness_condition=(
              "present only when the player accepted a locked pattern via "
              "POST /api/games/{id}/accept-outcome/ (FR-116-5); null for horizon "
              "termination and while in progress"
          ),
          dtype="int",
          write_site="web/game/api.py::accept-outcome view (tick_event detail stamp)",
          derivation_site="web/game/engine_bridge.py::_accepted_tick_from_endgame_row",
          read_paths=_ENDGAME_READ_PATHS,
          nullable=True,
          spec_ref="specs/116-playability-spine/spec.md · FR-116-5",
          notes="Accepted-at-tick framing on the end screen for player-accepted outcomes.",
      ),
  )
  ```

  Then APPEND `+ _ENDGAME_METRICS` as the last term of the existing `SEAM_REGISTRY`
  sum expression — do NOT paste a full assembly block: earlier spine tasks (Task 4)
  have already appended their own tuples to this sum, and later ones (Tasks 18/19)
  will; the sum at execution time must retain every tuple already present. The
  edit is one line:

  ```python
      + _ENDGAME_METRICS  # appended as the new final term; leave all prior terms
  )
  ```

  (If the accept-outcome view's landed function name differs from the ledger's
  endpoint spelling, correct the `write_site` string to the real symbol in the
  same commit — the row must cite reality, not the plan.)

- [ ] **Step 8: Run tests + seam gate to verify green**

  ```bash
  mise run test:q -- tests/unit/web/test_epilogues.py tests/unit/web/test_spec095_bridge.py tests/unit/web/test_stub_bridge_parity.py
  mise run check:seams
  ```

  Expected: all tests pass (the pre-existing `TestGetEndgameState` cases stay
  green — they assert key presence, not the old headline strings). `check:seams`
  exits 0 with `Seam continuity (Sensor 1): clean — 92 registered observables.`
  (89 rows + these 3; the count is indicative — other spine tasks may have
  added rows first). The narrator-vocabulary advisory still lists the six
  unreachable `_TEMPLATES` keys — untouched by design (its pin test
  `tests/unit/sentinels/test_seam_registry_check.py:173-176` must still pass).

- [ ] **Step 9: Commit**

  ```bash
  git add web/game/epilogues.py web/game/engine_bridge.py web/game/stub_bridge.py \
    src/babylon/sentinels/seam/registry.py \
    specs/095-endgame-chronicle/contracts/endgame.yaml \
    tests/unit/web/test_epilogues.py tests/unit/web/test_spec095_bridge.py \
    tests/unit/web/test_stub_bridge_parity.py
  mise run commit -- "feat(web): six distinct endgame epilogues — data module + bridge payload (spec-116 FR-116-4.2)"
  ```

### Task 16: EndStateScreen 6-way epilogue rendering + unresolved palette

**Files:**
- Create: `src/frontend/src/components/takeovers/chronicle/EndStateScreen.test.tsx`
- Modify: `src/frontend/src/types/dialectic.ts` (:53-59 `TerminalOutcome`; :69-76 `EndgameState`; :124-135 `EMPTY_ENDGAME`)
- Modify: `src/frontend/src/test/fixtures.ts` (:787-797 `makeEndgameState`)
- Modify: `src/frontend/src/components/takeovers/chronicle/EndStateScreen.tsx` (:1-121, header comment + component body)
- Modify: `src/frontend/src/components/takeovers/chronicle/chronicle.css` (append after :164 — additive only, the ratified `end-state-*` rules stay unchanged)

**Interfaces:**
- Consumes: Task 15's payload keys exactly — `epilogue: string`, `palette: "rupture" | "defeat" | "unresolved" | ""`, `accepted_at_tick: number | null`, and `outcome` now possibly `"unresolved"`. MSW test doubles: `server` (`@/test/server`), `makeEndgameState` (`@/test/fixtures`), `resetStore`/`resetMockGameState`/`DEFAULT_GAME_ID` (per the timeSlice.test.ts reset pattern).
- Produces: `EpiloguePalette` TS type (exported from `@/types/dialectic`); `.end-state--unresolved` palette class; `data-testid="end-state"` + `data-outcome` attribute on the end-state root (hooks for later e2e tasks); `data-testid="end-state-accepted"` accepted-at-tick framing line.

Note: if a Cluster A task has already added `"unresolved"` to `TerminalOutcome`, skip that one union edit (verify, don't duplicate); all other edits here are unique to this task. Frontend-only change — no engine surface; `qa:regression` untouched.

- [ ] **Step 1: Write the failing component test**

  Create `src/frontend/src/components/takeovers/chronicle/EndStateScreen.test.tsx`
  (colocated, modeled on `NarrationBlock.test.tsx` + the MSW `server.use()`
  override pattern from `TakeoverOverlay.test.tsx`):

  ```tsx
  /**
   * EndStateScreen — six distinct epilogues (spec-116 FR-116-4.2).
   *
   * The six-row table mirrors the backend SoT (web/game/epilogues.py): real
   * headlines + palettes, synthetic bodies. The component under test is pure
   * pass-through — distinctness itself is pinned backend-side in
   * tests/unit/web/test_epilogues.py; this file pins the 6-way rendering:
   * per-outcome headline/body/palette class/data-outcome, accepted-at-tick
   * framing, and the honest pending state.
   */

  import { describe, it, expect, beforeEach } from "vitest";
  import { render, screen, waitFor } from "@testing-library/react";
  import { http, HttpResponse } from "msw";
  import { server } from "@/test/server";
  import { EndStateScreen } from "./EndStateScreen";
  import { resetStore } from "@/test/resetStore";
  import { resetMockGameState, DEFAULT_GAME_ID } from "@/test/handlers";
  import { makeEndgameState } from "@/test/fixtures";
  import type { EndgameState } from "@/types/dialectic";

  beforeEach(() => {
    resetStore();
    resetMockGameState();
  });

  function mockEndgame(overrides: Partial<EndgameState>): void {
    server.use(
      http.get("/api/games/:id/endgame/", () =>
        HttpResponse.json({ status: "ok", data: makeEndgameState(overrides) }),
      ),
    );
  }

  const SIX_OUTCOMES = [
    { outcome: "revolutionary_victory", palette: "rupture", headline: "BABYLON FALLS" },
    { outcome: "ecological_collapse", palette: "defeat", headline: "THE EARTH BETRAYED" },
    { outcome: "fascist_consolidation", palette: "defeat", headline: "ORDER IS RESTORED" },
    { outcome: "red_ogv", palette: "defeat", headline: "RED FLAGS OVER EMPIRE" },
    { outcome: "fragmented_collapse", palette: "defeat", headline: "THE MAP SHATTERS" },
    { outcome: "unresolved", palette: "unresolved", headline: "THE STRUGGLE CONTINUES" },
  ] as const;

  describe("EndStateScreen — six distinct epilogues (spec-116 FR-116-4.2)", () => {
    it.each(SIX_OUTCOMES)(
      "renders $outcome with its own headline, body, and $palette palette",
      async ({ outcome, palette, headline }) => {
        mockEndgame({
          outcome,
          headline,
          epilogue: `${outcome} epilogue body.`,
          palette,
          tick: 5200,
          stats: { final_tick: 5200, consciousness: 0.42, solidarity_edges: 3, heat: 0.31 },
        });
        render(<EndStateScreen gameId={DEFAULT_GAME_ID} />);
        await waitFor(() => expect(screen.getByText(headline)).toBeInTheDocument());
        expect(screen.getByText(`${outcome} epilogue body.`)).toBeInTheDocument();
        const root = screen.getByTestId("end-state");
        expect(root).toHaveClass(`end-state--${palette}`);
        expect(root).toHaveAttribute("data-outcome", outcome);
      },
    );

    it("renders the accepted-at-tick framing for a player-accepted outcome", async () => {
      mockEndgame({
        outcome: "fascist_consolidation",
        headline: "ORDER IS RESTORED",
        epilogue: "The fash take hold.",
        palette: "defeat",
        accepted_at_tick: 3120,
      });
      render(<EndStateScreen gameId={DEFAULT_GAME_ID} />);
      await waitFor(() =>
        expect(screen.getByTestId("end-state-accepted")).toHaveTextContent(
          /accepted at tick 3120/i,
        ),
      );
    });

    it("omits the accepted framing for a horizon-terminated outcome", async () => {
      mockEndgame({
        outcome: "unresolved",
        headline: "THE STRUGGLE CONTINUES",
        epilogue: "One hundred years, and no verdict.",
        palette: "unresolved",
        accepted_at_tick: null,
      });
      render(<EndStateScreen gameId={DEFAULT_GAME_ID} />);
      await waitFor(() =>
        expect(screen.getByText("THE STRUGGLE CONTINUES")).toBeInTheDocument(),
      );
      expect(screen.queryByTestId("end-state-accepted")).not.toBeInTheDocument();
    });

    it("does not render the degraded machine summary once an epilogue exists", async () => {
      mockEndgame({
        outcome: "red_ogv",
        headline: "RED FLAGS OVER EMPIRE",
        summary: "Endgame Reached",
        epilogue: "The settler bargain was rebranded.",
        palette: "defeat",
      });
      render(<EndStateScreen gameId={DEFAULT_GAME_ID} />);
      await waitFor(() =>
        expect(screen.getByText("The settler bargain was rebranded.")).toBeInTheDocument(),
      );
      expect(screen.queryByText("Endgame Reached")).not.toBeInTheDocument();
    });

    it("keeps the honest pending state while the game is in progress", async () => {
      // Default fixture: outcome null (Constitution III.11 — never fabricate).
      render(<EndStateScreen gameId={DEFAULT_GAME_ID} />);
      await waitFor(() =>
        expect(
          screen.getByText("Operation in progress — no terminal outcome yet."),
        ).toBeInTheDocument(),
      );
      const root = screen.getByTestId("end-state");
      expect(root).toHaveClass("end-state--pending");
      expect(root).toHaveAttribute("data-outcome", "pending");
    });
  });
  ```

- [ ] **Step 2: Run test to verify it fails**

  ```bash
  cd src/frontend && npx vitest run src/components/takeovers/chronicle/EndStateScreen.test.tsx
  ```

  Expected behavioral failures (vitest transpiles without typechecking, so the
  red is runtime, not TS): `Unable to find an element with the text:
  revolutionary_victory epilogue body.` for each parametrized case (the
  component has no epilogue block yet), `Unable to find an element by:
  [data-testid="end-state"]`, and the accepted-framing test failing on a
  missing `end-state-accepted` testid — 8 failed, 1 passed (the pending case's
  text renders, but its testid/data-outcome assertions fail it too: 9 failed
  total).

- [ ] **Step 3: Extend the TS types + fixtures (green half 1)**

  `src/frontend/src/types/dialectic.ts` — replace :53-59 with (skip the union
  edit if Cluster A already added `"unresolved"`):

  ```ts
  /** Terminal GameOutcome (lowercase, matching the backend enum values). */
  export type TerminalOutcome =
    | "revolutionary_victory"
    | "ecological_collapse"
    | "fascist_consolidation"
    | "red_ogv"
    | "fragmented_collapse"
    | "unresolved";

  /** End-screen palette family (spec-116 FR-116-4.2): six texts, three palettes. */
  export type EpiloguePalette = "rupture" | "defeat" | "unresolved";
  ```

  Replace the `EndgameState` interface (:69-76) with:

  ```ts
  /** GET /api/games/:id/endgame/ response body. */
  export interface EndgameState {
    tick: number;
    outcome: TerminalOutcome | null;
    headline: string;
    /** Degraded tick_event machine text — kept on the wire, no longer rendered. */
    summary: string;
    /** Deterministic per-outcome epilogue body (spec-116 FR-116-4.2); "" in progress. */
    epilogue: string;
    /** "" while in progress. */
    palette: EpiloguePalette | "";
    /** Tick of player-accepted fast-forward (FR-116-5); null at horizon / in progress. */
    accepted_at_tick: number | null;
    stats: EndgameStats;
  }
  ```

  In `EMPTY_ENDGAME` (:124-135) add after `summary: "",`:

  ```ts
    epilogue: "",
    palette: "",
    accepted_at_tick: null,
  ```

  In `src/frontend/src/test/fixtures.ts` `makeEndgameState` (:787-797) add the
  same three defaults after `summary: "",`:

  ```ts
      epilogue: "",
      palette: "",
      accepted_at_tick: null,
  ```

- [ ] **Step 4: Rework EndStateScreen + additive CSS (green half 2)**

  Replace `src/frontend/src/components/takeovers/chronicle/EndStateScreen.tsx`
  :1-17 header and :53-120 component with (buildStats and the Props/StatCard
  interfaces are unchanged):

  ```tsx
  /**
   * EndStateScreen - chronicle end-screen for terminal outcomes.
   * Spec 095 FR-095-09 + spec-116 FR-116-4.2 (six distinct epilogues).
   *
   * The backend payload drives everything: `headline`/`epilogue` come from
   * web/game/epilogues.py (one distinct text per GameOutcome incl.
   * "unresolved"), `palette` picks one of three palette families (rupture
   * bronze-gold / defeat laser-red / unresolved spire-cyan), and
   * `accepted_at_tick` frames a player-accepted fast-forward (FR-116-5).
   * Fed by useEndgame polling GET /api/games/:id/endgame/.
   *
   * The deterministic epilogue is rendered separately from the "Last
   * Dispatch" epitaph (the flag-off LLM narration channel) — deterministic
   * copy must never masquerade as AI narration.
   *
   * Constitution III: pure read.
   */

  import { useEndgame } from "@/hooks/useEndgame";
  import { useNarration } from "@/hooks/useNarration";
  import { NarrationBlock } from "@/components/narration/NarrationBlock";
  import type { EndgameState } from "@/types/dialectic";
  import "@/components/takeovers/chronicle/chronicle.css";
  ```

  ```tsx
  /** Palette-keyed kickers — the one-line framing above the headline. */
  const KICKERS: Record<"rupture" | "defeat" | "unresolved", string> = {
    rupture: "▸ Rupture Achieved",
    defeat: "✕ Organizational Collapse",
    unresolved: "◌ Horizon Reached — The Struggle Continues",
  };

  export function EndStateScreen({ gameId, onRestart }: Props) {
    const { data: state, loading, error } = useEndgame(gameId);
    const { status: narrationStatus, beats } = useNarration(gameId);
    const endgameBeat = beats.filter((b) => b.scope === "endgame").at(-1) ?? null;

    const isRupture = state.outcome === "revolutionary_victory";
    let palette: string;
    if (!state.outcome) {
      palette = "end-state--pending";
    } else if (state.palette !== "") {
      palette = `end-state--${state.palette}`;
    } else {
      // Defensive fallback for a payload predating spec-116.
      palette = isRupture ? "end-state--rupture" : "end-state--defeat";
    }

    if (!state.outcome) {
      let pendingText: string;
      if (loading) {
        pendingText = "Reading terminal state…";
      } else if (error) {
        pendingText = `Error: ${error}`;
      } else {
        pendingText = "Operation in progress — no terminal outcome yet.";
      }
      return (
        <div className={`end-state ${palette}`} data-testid="end-state" data-outcome="pending">
          <div className="end-state-scanlines" />
          <div className="end-state-content">
            <div className="end-state-pending-text">{pendingText}</div>
          </div>
        </div>
      );
    }

    const stats = buildStats(state);
    const kicker =
      state.palette !== ""
        ? KICKERS[state.palette]
        : isRupture
          ? KICKERS.rupture
          : KICKERS.defeat;

    return (
      <div className={`end-state ${palette}`} data-testid="end-state" data-outcome={state.outcome}>
        <div className="end-state-scanlines" />
        <div className="end-state-content">
          <div className="end-state-kicker">{kicker}</div>
          <h1 className="end-state-headline">{state.headline}</h1>
          {state.accepted_at_tick !== null && (
            <div className="end-state-accepted" data-testid="end-state-accepted">
              ▸ Outcome accepted at tick {state.accepted_at_tick} — fast-forwarded to the
              epilogue.
            </div>
          )}
          {/* Deterministic epilogue (spec-116 FR-116-4.2). The wire `summary`
              field is degraded machine text ("Endgame Reached") — superseded
              here, still on the wire for contract compat. */}
          {state.epilogue && <p className="end-state-epilogue-body">{state.epilogue}</p>}
          <div className="end-state-stats">
            {stats.map((s) => (
              <div key={s.label} className="end-state-stat">
                <div className="end-state-stat-label">{s.label}</div>
                <div className="end-state-stat-value" style={{ color: s.color }}>
                  {s.value}
                </div>
              </div>
            ))}
          </div>
          {/* Epitaph — the narrator's last word on this operation. Endgames are
              never neutral scoreboard text (Design Bible §7); honest
              offline/pending states render via NarrationBlock, never blank. */}
          <div className="end-state-epitaph">
            <div className="end-state-epitaph-label">Last Dispatch</div>
            <NarrationBlock beat={endgameBeat} state={narrationStatus} />
          </div>
          {onRestart && (
            <button className="end-state-restart" onClick={onRestart}>
              ▸ New Operation
            </button>
          )}
        </div>
      </div>
    );
  }
  ```

  Append to `src/frontend/src/components/takeovers/chronicle/chronicle.css`
  (after :164; `.end-state-summary` stays — additive rule, ratified block
  unchanged):

  ```css
  /* Spec-116 FR-116-4.2 — six distinct epilogues, three palettes. Additive:
     the ratified end-state-* rules above are unchanged. */
  .end-state--unresolved {
    background: radial-gradient(ellipse at center, #0a1418 0%, var(--babylon-void) 75%);
  }

  .end-state--unresolved .end-state-kicker {
    color: var(--babylon-spire);
    text-shadow: 0 0 16px rgba(77, 217, 230, 0.4);
  }

  .end-state--unresolved .end-state-headline {
    color: var(--babylon-bone);
  }

  .end-state-accepted {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.22em;
    text-transform: uppercase;
    color: var(--babylon-ash);
    margin-bottom: 16px;
  }

  .end-state-epilogue-body {
    font-size: 16px;
    color: var(--babylon-bone);
    line-height: 1.7;
    margin-bottom: 36px;
  }
  ```

- [ ] **Step 5: Run test to verify it passes**

  ```bash
  cd src/frontend && npx vitest run src/components/takeovers/chronicle/EndStateScreen.test.tsx
  ```

  Expected: `Test Files 1 passed`, `Tests 10 passed` (6 parametrized + 4).

- [ ] **Step 6: Full frontend gate (types ripple check)**

  ```bash
  cd src/frontend && npm run check
  ```

  Expected: `tsc --noEmit` clean (the three new required `EndgameState` fields
  are satisfied by `EMPTY_ENDGAME` + `makeEndgameState`; no other fixture
  builds `EndgameState` literals), eslint + prettier clean, full vitest suite
  green — including the untouched `TakeoverOverlay.test.tsx` chronicle case
  (default fixture outcome is null → pending branch unchanged).

- [ ] **Step 7: Commit**

  ```bash
  git add src/frontend/src/types/dialectic.ts src/frontend/src/test/fixtures.ts \
    src/frontend/src/components/takeovers/chronicle/EndStateScreen.tsx \
    src/frontend/src/components/takeovers/chronicle/chronicle.css \
    src/frontend/src/components/takeovers/chronicle/EndStateScreen.test.tsx
  mise run commit -- "feat(frontend): 6-way distinct end-state epilogues + unresolved palette (spec-116 FR-116-4.2)"
  ```
### Task 17: Pre-submit preview & cost visibility (ActionComposer, FR-116-4.3)

**Files:**
- Modify: `src/frontend/src/components/action/VerbForm.tsx:108-138` (insert a visible cost line; no other JSX touched)
- Test: `src/frontend/src/components/action/VerbForm.test.tsx` (extend — new describe block + one stub helper)
- Test: `src/frontend/src/components/action/ActionComposer.test.tsx:139-171` (extend the integrated preview test)
- Test: `src/frontend/e2e/verb-submit.spec.ts:68-93` (extend the campaign submit test with the gate-5 assertion; file is ALREADY in `AUTHENTICATED_SPECS` — no `playwright.config.ts` change)

**Interfaces:**
- Consumes: `LiveVerbCost {label, canAfford}` (`src/frontend/src/lib/verbs/types.ts:20-25`) — already fetched per-verb by `useVerbTargets` (`useVerbTargets.ts:85`) and today forwarded ONLY to VerbGrid's hover `title`; `ActionPreviewResult.action_point_cost` (`src/frontend/src/types/game.ts:710-717`) — already fetched by `useActionPreview`. **No backend change, no new wire keys, no TS type change.**
- Produces: `data-testid="verb-cost"` — the visible pre-submit cost line (asserted by the e2e gate and by ActionComposer.test.tsx).

**Scope rulings (drafter, from the recon brief's open questions):**
- `!canAfford` renders a loud crimson "insufficient" flag but does **not** disable submit — the spec gate is "preview visible before every submit", the backend already rejects unaffordable actions loudly, and disabling on a stale cost fetch would block legal submits. (Owner may veto; one-line change.)
- `preview_action` does NOT grow a `resource_cost` key in this task. Resource cost is already fetched per-verb via the targets GET (`cost` envelope) — rendering it closes 4d.3 without touching the latent `preview.get("resource_cost", {})` contract at `web/game/api.py:1657` (documented, deliberately untouched).
- Determinism: frontend-only — no engine, no defines, `qa:regression` untouched by construction.

- [ ] **Step 1: Write the failing VerbForm tests**

  Append to `src/frontend/src/components/action/VerbForm.test.tsx` (after the existing `describe("VerbForm action preview", ...)` block). Add one import at the top of the file:

  ```tsx
  import { parseFlatCost } from "@/lib/verbs/cost";
  ```

  ```tsx
  function stubTargetsWithCost(canAfford = true): void {
    server.use(
      http.get("/api/games/:id/actions/educate/targets/", () =>
        HttpResponse.json({
          targets: [{ community_id: "comm-1", territory_name: "Downtown" }],
          cost: {
            action_points: 1,
            cadre_labor: 3.0,
            sympathizer_labor: 0.0,
            material: 0.0,
            can_afford: canAfford,
            over_budget: false,
            over_budget_penalty: null,
          },
        }),
      ),
    );
  }

  describe("VerbForm pre-submit cost line (spec-116 FR-116-4.3)", () => {
    it("shows the live resource cost before any target is selected", async () => {
      stubTargetsWithCost();
      stubPreview();
      renderForm(makeConfig({ parseCost: parseFlatCost }));

      await waitFor(() => expect(screen.getByTestId("target-picker")).toBeInTheDocument());

      const costLine = await screen.findByTestId("verb-cost");
      expect(costLine).toHaveTextContent("3 CL");
      // Preview not composable yet (no target): no AP segment, no fabrication.
      expect(costLine).not.toHaveTextContent("AP");
    });

    it("appends the preview's AP cost once a target is selected", async () => {
      stubTargetsWithCost();
      stubPreview();
      renderForm(makeConfig({ parseCost: parseFlatCost }));

      await selectDowntown();

      await waitFor(() =>
        expect(screen.getByTestId("verb-cost")).toHaveTextContent("3 CL · 1 AP"),
      );
    });

    it("flags an unaffordable cost in crimson but never disables submit", async () => {
      stubTargetsWithCost(false);
      stubPreview();
      renderForm(makeConfig({ parseCost: parseFlatCost }));

      await selectDowntown();

      const costLine = screen.getByTestId("verb-cost");
      expect(costLine).toHaveTextContent("insufficient");
      expect(costLine.className).toContain("text-accent-crimson");
      expect(screen.getByRole("button", { name: /submit educate/i })).toBeEnabled();
    });

    it("renders no cost line for a cost-less verb until the preview lands (honest null)", async () => {
      stubTargets(); // no cost envelope, config has no parseCost (campaign-shaped)
      stubPreview();
      renderForm(makeConfig());

      await waitFor(() => expect(screen.getByTestId("target-picker")).toBeInTheDocument());
      expect(screen.queryByTestId("verb-cost")).not.toBeInTheDocument();

      await selectDowntown();
      await waitFor(() => expect(screen.getByTestId("verb-cost")).toHaveTextContent("1 AP"));
    });
  });
  ```

- [ ] **Step 2: Run the tests to verify they fail**

  ```bash
  cd src/frontend && npx vitest run src/components/action/VerbForm.test.tsx
  ```

  Expected: the 4 new tests fail with `Unable to find an element by: [data-testid="verb-cost"]`; the 6 pre-existing preview tests stay green.

- [ ] **Step 3: Write the minimal implementation**

  In `src/frontend/src/components/action/VerbForm.tsx`, insert between the `{preview && (...)}` block (ends line 129) and the submit `<button>` (line 131):

  ```tsx
      {(cost !== null || preview !== null) && (
        <p
          data-testid="verb-cost"
          className={`font-mono text-[10px] uppercase tracking-widest ${
            cost !== null && !cost.canAfford ? "text-accent-crimson" : "text-fog"
          }`}
        >
          {[
            cost?.label,
            preview ? `${preview.action_point_cost} AP` : null,
            cost !== null && !cost.canAfford ? "insufficient" : null,
          ]
            .filter(Boolean)
            .join(" · ")}
        </p>
      )}
  ```

  And extend the component docstring (lines 1-7) with one sentence:

  ```
   * FR-116-4.3: the live per-verb cost (`useVerbTargets` cost envelope) and
   * the preview's AP cost render as a visible line above the submit button —
   * previously the only cost surface was VerbGrid's hover tooltip. Honest
   * null: the line renders only once a real cost or preview has resolved.
  ```

  No hook changes: `cost` and `preview` are already in scope (lines 77, 86). `!canAfford` does not touch `canSubmit`.

- [ ] **Step 4: Run the tests to verify they pass**

  ```bash
  cd src/frontend && npx vitest run src/components/action/VerbForm.test.tsx
  ```

  Expected: `Test Files  1 passed`, 10 tests passed (6 pre-existing + 4 new).

- [ ] **Step 5: Extend the integrated ActionComposer assertion and run it**

  In `src/frontend/src/components/action/ActionComposer.test.tsx`, in the test that stubs the preview POST and asserts `predicted-delta` (the `submit educate` flow ending ~line 170), append after the delta assertion:

  ```tsx
      // FR-116-4.3: the cost line is visible before submit. This educate stub
      // returns no cost envelope, so the line carries the preview's AP cost.
      expect(screen.getByTestId("verb-cost")).toHaveTextContent("1 AP");
  ```

  ```bash
  cd src/frontend && npx vitest run src/components/action/ActionComposer.test.tsx
  ```

  Expected: all ActionComposer tests pass (the new assertion is green because Step 3 landed; if run before Step 3 it fails with the missing-testid error — that ordering is fine, it is the same red).

- [ ] **Step 6: Extend the e2e campaign flow with the acceptance-gate assertion**

  In `src/frontend/e2e/verb-submit.spec.ts`, inside `"campaign submits through the UI and lands in the pending list"`, insert between the target-picker click (line 75) and the submit-button lookup (line 77):

  ```ts
      // Spine acceptance gate #5 (spec-116): preview visible BEFORE every
      // submit. Campaign deltas can be zero (DeltaChip is honest-null), so the
      // guaranteed pre-submit surfaces are the probability line and the cost
      // line (campaign has no live cost envelope — GET 405s — so the line
      // carries the preview's AP cost).
      await expect(page.getByTestId("preview-probability")).toBeVisible({ timeout: 10000 });
      await expect(page.getByTestId("verb-cost")).toBeVisible();
      await expect(page.getByTestId("verb-cost")).toContainText("AP");
  ```

  Run against the live stack (Django :8000 + Vite :5173):

  ```bash
  mise run web:dev
  cd src/frontend && npx playwright test e2e/verb-submit.spec.ts
  ```

  Expected: `4 passed` (setup project + the serial suite; the submit test now proves preview-before-submit in the same flow that submits). `real-loop.spec.ts` stays untouched and passing.

- [ ] **Step 7: Full frontend gate**

  ```bash
  cd src/frontend && npm run check
  ```

  Expected: tsc, eslint, prettier, and the full vitest suite all green.

- [ ] **Step 8: Commit**

  ```bash
  git add src/frontend/src/components/action/VerbForm.tsx \
          src/frontend/src/components/action/VerbForm.test.tsx \
          src/frontend/src/components/action/ActionComposer.test.tsx \
          src/frontend/e2e/verb-submit.spec.ts
  mise run commit -- "feat(cockpit): pre-submit cost + preview visibility in ActionComposer (spec-116 FR-116-4.3)"
  ```

---

### Task 18: Per-target expected deltas — bridge enrichment, serializer reconciliation, TargetPicker chips (FR-116-4.4)

**Files:**
- Modify: `src/babylon/config/defines/ooda.py:256-262` (append `attack_self_heat_gain` after the `contestation_threshold` field)
- Modify: `src/babylon/engine/actions/attack.py:29-31,34-59` (delete the `_ATTACK_SELF_HEAT_GAIN` module literal; resolver reads `services.defines.ooda.attack_self_heat_gain`)
- Regenerate: `src/babylon/data/defines.yaml` (via `tools/generate_defines_config.py` — never hand-edited)
- Modify: `web/game/engine_bridge.py:5041-5075` (educate row), `:5124-5146` (aid population row), `:5277,5305-5325` (attack org + institution rows)
- Modify: `web/game/serializers.py:460-465` (FeedforwardSerializer reconciliation), `:468-479` (EducateTargetSerializer), `:580-594` (AidProjectionSerializer + PopulationAidTargetSerializer), `:740-750` (AttackTargetOrgSerializer), `:769-774` (AttackTargetInstitutionModelSerializer), plus one new `ExpectedDeltasSerializer`
- Modify: `src/babylon/sentinels/seam/types.py:84-92` (new `SeamScope.ACTION` member + docstring cvar), `src/babylon/sentinels/seam/registry.py:2016-2024` (new `_ACTION_METRICS` row, appended to `SEAM_REGISTRY`)
- Modify: `src/frontend/src/lib/verbs/types.ts:6-13` (VerbTarget.expectedDeltas), `src/frontend/src/lib/verbs/educate.ts:4-23`, `aid.ts:4-33`, `attack.ts:3-67`
- Create: `src/frontend/src/lib/verbs/expectedDeltas.ts` (shared wire→VerbTarget mapper)
- Modify: `src/frontend/src/components/action/TargetPicker.tsx` (per-row delta chips)
- Test: `tests/contract/verbs/test_effects.py:183-198` (extend `TestAttack`), `tests/unit/web/test_engine_bridge.py` (new `TestExpectedDeltas`), `tests/unit/web/test_per_verb_views.py` (new `TestTargetRowSerialization`)
- Test: `src/frontend/src/components/action/TargetPicker.test.tsx` (CREATE — none exists today), `src/frontend/src/lib/verbs/__tests__/verbs.test.ts` (extend)

**Interfaces:**
- Consumes: `_preview_consciousness_delta(org_data, target_id, action_type, graph) -> float` (`web/game/engine_bridge.py:8524-8552` — the resolvers' own `compute_consciousness_delta`, so preview == resolution; reuses its exact `GameDefines()` construction, guarded identical to defines.yaml by `tests/unit/config/test_constants_sync.py`); `VERB_RESOLVERS`/`resolve_attack` (`src/babylon/engine/actions/`).
- Produces: **new wire keys** `expected_deltas` / `expected_deltas.consciousness_delta` / `expected_deltas.heat_delta` on educate targets rows, aid `population_targets` rows, and attack `organizations`+`institutions` rows; **new defines field** `OODADefines.attack_self_heat_gain: float = 0.1`; `SeamScope.ACTION`; frontend `VerbTarget.expectedDeltas?: {consciousness?: number; heat?: number}`; `data-testid="target-delta"` chips.

**Scope rulings (drafter, from the recon brief's open questions):**
- Deltas ship ONLY where resolver math is per-target-real: educate/aid consciousness (`compute_consciousness_delta`), attack self-heat (the resolver's own coefficient, promoted to a define). The axis with no formula serializes as an honest `null`, never a fabricated 0.0; the category heuristics in `preview_action` (0.02 / 0.08·cohesion) are NOT copied into per-row payloads (Loud Failure).
- Campaign rows stay delta-less (snapshot-sourced, its targets GET 405s — honest absence). Investigate `territory_scans` already carry `heat`/`resource_cost`/`projected_reveals`; its `targeted_scans` are hardcoded fixture rows — nothing is built on them. Aid `org_targets` carry no `expected_deltas` key (CI math targets communities, not orgs).
- Enrichment happens at the bridge (one graph hydration per GET), NEVER as N frontend `POST /actions/preview/` calls — 81+ rows in wayne_county would re-hydrate the full graph per row (`hydrate_state` at `engine_bridge.py:5665`).
- **CONFIRMED pre-existing 500s fixed here** (verified against DRF `to_representation` semantics: a missing required key raises `KeyError`; a `None` value passes; `required=False` nested serializers skip cleanly): GET attack targets 500s on any populated org/institution row (`description`/`attack_projection` required at `serializers.py:747,750,774` but never produced), and GET educate/aid targets 500 on any populated row (`FeedforwardSerializer`/`AidProjectionSerializer` require projection keys but the bridge emits note-only dicts — `engine_bridge.py:5071-5073,5142-5144`). Only empty-row payloads are pinned by today's tests, which is why this never fired in CI.
- Determinism: the ONLY engine edit is the value-identical define promotion (0.1 == 0.1) — `qa:regression` must stay byte-identical (`defines_hash` drift alone is WARNING-only); bridge/serializer/frontend edits are serialization-boundary, byte-safe by construction.

- [ ] **Step 1: Write the failing engine test (define-driven attack heat)**

  Append to `class TestAttack` in `tests/contract/verbs/test_effects.py`:

  ```python
      def test_attack_self_heat_gain_is_defines_driven(self, verb_graph) -> None:
          """The self-heat coefficient is OODADefines.attack_self_heat_gain, not a literal.

          spec-116 FR-116-4.4 promotes the old ``_ATTACK_SELF_HEAT_GAIN = 0.1``
          module literal into GameDefines so the web bridge's per-target heat
          estimate and the resolver share one source of truth.
          """
          from babylon.config.defines import GameDefines
          from babylon.engine.services import ServiceContainer

          defines = GameDefines()
          modded = defines.model_copy(
              update={"ooda": defines.ooda.model_copy(update={"attack_self_heat_gain": 0.25})}
          )
          services = ServiceContainer.create(defines=modded)

          heat_before = float(verb_graph.nodes[ORG_ID]["heat"])
          result = _dispatch(verb_graph, services, ActionType.ATTACK_INFRASTRUCTURE, HOME_TERRITORY)

          assert result.success is True
          assert float(verb_graph.nodes[ORG_ID]["heat"]) == pytest.approx(heat_before + 0.25)
          assert result.direct_effects["heat_self_delta"] == pytest.approx(0.25)
  ```

- [ ] **Step 2: Run it to verify it fails**

  ```bash
  mise run test:q -- tests/contract/verbs/test_effects.py::TestAttack
  ```

  Expected: the new test fails on the first `pytest.approx` (heat rose by the hardcoded 0.1, not 0.25 — `model_copy(update=...)` does not validate, so the unknown field is carried but never read); the pre-existing `test_attack_infra_channel_fires_via_layer3` stays green.

- [ ] **Step 3: Implement the define promotion**

  1. `src/babylon/config/defines/ooda.py` — insert after the `contestation_threshold` field block (line 262), before `# --- Lifecycle modifiers ---`:

  ```python
      # --- Attack resolver effects (spec-116 FR-116-4.4) ---
      attack_self_heat_gain: float = Field(
          default=0.1,
          ge=0.0,
          le=1.0,
          description=(
              "State attention (heat) the acting org draws on itself when an "
              "ATTACK_INFRASTRUCTURE action resolves. Promoted from the "
              "engine.actions.attack module literal so the resolver and the "
              "web preview/target rows share one source of truth."
          ),
      )
  ```

  2. `src/babylon/engine/actions/attack.py` — delete lines 30-31 (`#: State attention...` + `_ATTACK_SELF_HEAT_GAIN = 0.1`); drop the `# noqa: ARG001` on the `services` parameter (it is now used); replace the heat block (lines 53-59) with:

  ```python
      heat_gain = float(services.defines.ooda.attack_self_heat_gain)
      org_node = graph.nodes.get(action.org_id)
      heat_self_delta = 0.0
      if org_node is not None and org_node.get("_node_type") == "organization":
          heat = float(org_node.get("heat", 0.0))
          new_heat = min(1.0, heat + heat_gain)
          graph.update_node(action.org_id, heat=new_heat)
          heat_self_delta = new_heat - heat
  ```

  and update the docstring's `services:` arg line to: `services: ServicesProtocol — supplies OODADefines.attack_self_heat_gain.`

  3. Regenerate the canonical YAML (same commit, never hand-edit):

  ```bash
  poetry run python tools/generate_defines_config.py
  ```

- [ ] **Step 4: Verify green + byte-identical regression, then commit (unit A)**

  ```bash
  mise run test:q -- tests/contract/verbs/test_effects.py tests/unit/config/test_constants_sync.py
  mise run qa:regression
  ```

  Expected: all tests pass; `Results: 5 passed, 0 failed` / `All regression tests passed!` — value-identical promotion, dense CSVs byte-identical (a `defines_hash` WARNING line alone is acceptable and expected). If ANY tick value moves: STOP — the promotion was not value-identical.

  ```bash
  git add src/babylon/config/defines/ooda.py src/babylon/engine/actions/attack.py \
          src/babylon/data/defines.yaml tests/contract/verbs/test_effects.py
  mise run commit -- "refactor(defines): promote attack self-heat gain to OODADefines.attack_self_heat_gain (spec-116 FR-116-4.4)"
  ```

- [ ] **Step 5: Write the failing bridge + view tests**

  1. Append to `tests/unit/web/test_engine_bridge.py` (uses the existing `_make_mock_persistence` / `_make_balkanization_graph` / `_patched_hydrate_state` helpers, lines 36-58, 1431-1524):

  ```python
  @pytest.mark.unit
  class TestExpectedDeltas:
      """Spec-116 FR-116-4.4: per-target expected_deltas on verb-target rows,
      sourced from the resolvers' own math (preview == resolution). The axis a
      verb has no per-target formula for is an honest None, never 0.0."""

      def test_educate_rows_carry_resolver_parity_consciousness_delta(self) -> None:
          from babylon.models.enums import ActionType
          from game.engine_bridge import _preview_consciousness_delta

          mock_persistence = _make_mock_persistence()
          bridge = EngineBridge(mock_persistence)
          graph = _make_balkanization_graph()

          with _patched_hydrate_state(bridge, graph):
              result = bridge.get_educate_targets(uuid.uuid4(), "org-player")

          target = result["targets"][0]
          expected = round(
              _preview_consciousness_delta(
                  dict(graph.nodes["org-player"]),
                  "sc-genesee-proles",
                  ActionType.EDUCATE,
                  graph,
              ),
              4,
          )
          assert target["expected_deltas"]["consciousness_delta"] == expected
          assert target["expected_deltas"]["heat_delta"] is None

      def test_aid_population_rows_carry_deltas_and_org_rows_do_not(self) -> None:
          from babylon.models.enums import ActionType
          from game.engine_bridge import _preview_consciousness_delta

          mock_persistence = _make_mock_persistence()
          bridge = EngineBridge(mock_persistence)
          graph = _make_balkanization_graph()

          with _patched_hydrate_state(bridge, graph):
              result = bridge.get_aid_targets(uuid.uuid4(), "org-player")

          pop = result["population_targets"][0]
          expected = round(
              _preview_consciousness_delta(
                  dict(graph.nodes["org-player"]),
                  pop["community_id"],
                  ActionType.PROVIDE_SERVICE,
                  graph,
              ),
              4,
          )
          assert pop["expected_deltas"]["consciousness_delta"] == expected
          assert pop["expected_deltas"]["heat_delta"] is None
          for org_row in result["org_targets"]:
              assert "expected_deltas" not in org_row

      def test_attack_rows_carry_defines_driven_heat_delta(self) -> None:
          from babylon.config.defines import GameDefines

          mock_persistence = _make_mock_persistence()
          bridge = EngineBridge(mock_persistence)
          graph = _make_balkanization_graph()
          graph.add_node(
              "org-rivals",
              "organization",
              name="Citizens Council",
              org_type="business",
              budget=340.0,
              territory_ids=["T1"],
          )
          graph.add_node(
              "inst-court",
              "institution",
              name="County Court",
              factional_composition={"security_state": 0.6},
              territory_ids=["T1"],
          )

          with _patched_hydrate_state(bridge, graph):
              result = bridge.get_attack_targets(uuid.uuid4(), "org-player")

          heat_gain = round(GameDefines().ooda.attack_self_heat_gain, 4)
          org_rows = result["targets"]["organizations"]
          inst_rows = result["targets"]["institutions"]
          assert len(org_rows) >= 1 and len(inst_rows) >= 1
          for row in [*org_rows, *inst_rows]:
              assert row["expected_deltas"]["heat_delta"] == heat_gain
              assert row["expected_deltas"]["consciousness_delta"] is None
  ```

  2. Append to `tests/unit/web/test_per_verb_views.py`:

  ```python
  _ORG_SUMMARY: dict = {
      "id": "org_1",
      "name": "Vanguard Cell",
      "type": "political_faction",
      "consciousness_strategy": "REVOLUTIONARY",
      "resources": {"cadre_labor": 5.0, "sympathizer_labor": 10.0, "material": 100.0},
      "ooda": {"action_points_remaining": 3, "action_points_max": 3, "cycle_time": 2},
      "cadre_level": 5.0,
      "cohesion": 0.6,
  }

  _FLAT_COST: dict = {
      "action_points": 1,
      "cadre_labor": 3.0,
      "sympathizer_labor": 0.0,
      "material": 0.0,
      "can_afford": True,
      "over_budget": False,
      "over_budget_penalty": None,
  }


  @pytest.mark.unit
  @pytest.mark.django_db
  class TestTargetRowSerialization:
      """Spec-116 FR-116-4.4: POPULATED target rows serialize through the GET
      targets endpoints (today only empty-row payloads are pinned, which hid a
      500 on every populated educate/aid/attack row), including expected_deltas."""

      def _setup(self) -> tuple:
          from unittest.mock import MagicMock

          from django.contrib.auth.models import User

          from game.models import GameSession

          user = User.objects.create_user(  # type: ignore[no-untyped-call]
              username=f"rowser_user_{uuid_mod.uuid4().hex[:8]}",
              password="testpass123",
          )
          client = Client()
          client.login(username=user.username, password="testpass123")
          session = GameSession.objects.create(
              id=uuid_mod.uuid4(),
              player_id=user.id,
              scenario="two_node",
              current_tick=0,
              status="active",
          )
          mock_bridge = MagicMock()

          import game.api

          game.api._bridge_instance = mock_bridge
          return client, session, mock_bridge

      def test_educate_targets_get_serializes_populated_rows(self) -> None:
          client, session, mock_bridge = self._setup()
          mock_bridge.get_educate_targets.return_value = {
              "status": "ok",
              "tick": 0,
              "verb": "educate",
              "acting_org": _ORG_SUMMARY,
              "cost": _FLAT_COST,
              "targets": [
                  {
                      "community_id": "sc-1",
                      "community_type": "PROLETARIAT",
                      "category": "social_class",
                      "territory_name": "Genesee County",
                      "territory_id": "T1",
                      "credibility": 0.6,
                      "credibility_explanation": "60% org cohesion",
                      "consciousness": {
                          "r": 0.0,
                          "l": 0.0,
                          "f": 0.0,
                          "dominant_tendency": "unknown",
                          "collective_identity": None,
                          "ideological_contestation": None,
                      },
                      "material_readiness": {
                          "avg_agitation": 0.62,
                          "readiness_score": 1.0,
                          "readiness_explanation": "Real SocialClass.agitation.",
                      },
                      "education_pressure": {
                          "current": 0.0,
                          "projected_delta": None,
                          "projected_new": None,
                          "decay_per_tick": None,
                      },
                      "feedforward": {"note": "No per-tick routing-shift projection yet."},
                      "expected_deltas": {"consciousness_delta": 0.0123, "heat_delta": None},
                  }
              ],
              "unavailable_communities": [],
          }
          url = reverse("game:verb-educate-targets", kwargs={"game_id": str(session.id)})
          response = client.get(url, {"org_id": "org_1"})
          assert response.status_code == 200, response.content
          row = json.loads(response.content)["targets"][0]
          assert row["expected_deltas"] == {"consciousness_delta": 0.0123, "heat_delta": None}

      def test_attack_targets_get_serializes_populated_org_and_institution_rows(self) -> None:
          client, session, mock_bridge = self._setup()
          mock_bridge.get_attack_targets.return_value = {
              "status": "ok",
              "tick": 0,
              "verb": "attack",
              "acting_org": _ORG_SUMMARY,
              "cost": {
                  "action_points": 3,
                  "cadre_labor_if_targeted": 2.5,
                  "sympathizer_labor_if_mass": 25.0,
                  "material": 100.0,
                  "can_afford_targeted": True,
                  "can_afford_mass": False,
                  "over_budget_ap": False,
                  "cost_explanation": "TARGETED uses cadre; MASS uses sympathizers.",
              },
              "ultra_left_warning": {
                  "active": False,
                  "trap_score": 0.0,
                  "indicators": [],
                  "explanation": "No trap detection has run yet this session.",
              },
              "warsaw_ghetto_flag": {
                  "active": False,
                  "population_p_acquiescence": None,
                  "threshold": 0.05,
                  "explanation": "Desperation endorsement threshold.",
              },
              "targets": {
                  "organizations": [
                      {
                          # NOTE: no "description", no "attack_projection" — the
                          # bridge never produces them (engine_bridge.py:5305-5315);
                          # this row 500s against today's serializer.
                          "target_id": "org-rivals",
                          "target_type": "BUSINESS",
                          "name": "Citizens Council",
                          "territory_name": "Genesee County",
                          "territory_id": "T1",
                          "defensive_capacity": 340.0,
                          "extractive_edges": [],
                          "expected_deltas": {"consciousness_delta": None, "heat_delta": 0.1},
                      }
                  ],
                  "edges": [],
                  "institutions": [
                      {
                          "target_id": "inst-court",
                          "target_type": "INSTITUTION",
                          "name": "County Court",
                          "factional_control": {"security_state": 0.6},
                          "expected_deltas": {"consciousness_delta": None, "heat_delta": 0.1},
                      }
                  ],
              },
              "unavailable_targets": [],
          }
          url = reverse("game:verb-attack-targets", kwargs={"game_id": str(session.id)})
          response = client.get(url, {"org_id": "org_1"})
          assert response.status_code == 200, response.content
          body = json.loads(response.content)
          assert body["targets"]["organizations"][0]["expected_deltas"]["heat_delta"] == 0.1
          assert body["targets"]["institutions"][0]["expected_deltas"]["heat_delta"] == 0.1

      def test_aid_targets_get_serializes_populated_population_rows(self) -> None:
          client, session, mock_bridge = self._setup()
          mock_bridge.get_aid_targets.return_value = {
              "status": "ok",
              "tick": 0,
              "verb": "aid",
              "acting_org": _ORG_SUMMARY,
              "cost": _FLAT_COST,
              "population_targets": [
                  {
                      "community_id": "sc-1",
                      "community_name": "Genesee Proletariat",
                      "population": 5000,
                      "class_name": "PROLETARIAT",
                      "material_conditions": {"v_value_produced": 812.4},
                      "edge_status": {},
                      # note-only feedforward — exactly what the bridge emits
                      # (engine_bridge.py:5142-5144); 500s against today's serializer.
                      "feedforward": {"note": "No per-tick aid-effect projection yet."},
                      "expected_deltas": {"consciousness_delta": 0.004, "heat_delta": None},
                  }
              ],
              "org_targets": [],
              "unavailable_targets": [],
          }
          url = reverse("game:verb-aid-targets", kwargs={"game_id": str(session.id)})
          response = client.get(url, {"org_id": "org_1"})
          assert response.status_code == 200, response.content
          row = json.loads(response.content)["population_targets"][0]
          assert row["expected_deltas"] == {"consciousness_delta": 0.004, "heat_delta": None}
  ```

- [ ] **Step 6: Run them to verify they fail**

  ```bash
  mise run test:q -- tests/unit/web/test_engine_bridge.py::TestExpectedDeltas \
                     tests/unit/web/test_per_verb_views.py::TestTargetRowSerialization
  ```

  Expected: all 6 fail — the bridge tests with `KeyError: 'expected_deltas'`; the view tests with `assert 500 == 200` (the serializer `KeyError` on the missing required fields — the CONFIRMED pre-existing defect).

- [ ] **Step 7: Implement bridge enrichment + serializer reconciliation + seam registration**

  1. `web/game/engine_bridge.py` — `get_educate_targets`: inside the social-class loop, append to the row dict after the `"feedforward"` entry (line 5071-5073):

  ```python
                          "expected_deltas": {
                              "consciousness_delta": round(
                                  _preview_consciousness_delta(
                                      org_data, sc_id, ActionType.EDUCATE, graph
                                  ),
                                  4,
                              ),
                              "heat_delta": None,
                          },
  ```

  2. `get_aid_targets`: append to the `population_targets` row dict after its `"feedforward"` entry (line 5142-5144) — `org_targets` rows deliberately get NO key:

  ```python
                              "expected_deltas": {
                                  "consciousness_delta": round(
                                      _preview_consciousness_delta(
                                          org_data, node_id, ActionType.PROVIDE_SERVICE, graph
                                      ),
                                      4,
                                  ),
                                  "heat_delta": None,
                              },
  ```

  3. `get_attack_targets`: after `org_data = graph.nodes.get(org_id, {})` (line 5277) insert:

  ```python
          # Resolver-parity heat estimate: the ATTACK resolver's own self-heat
          # coefficient. Same GameDefines() construction _preview_consciousness_delta
          # uses (schema defaults; test_constants_sync guards them identical to
          # defines.yaml) — one source of truth, per the Step-3 promotion.
          attack_heat_gain = round(GameDefines().ooda.attack_self_heat_gain, 4)
  ```

  then add to the `organizations.append({...})` dict after `"extractive_edges"` (line 5313) and to the `institutions.append({...})` dict after `"factional_control"` (line 5323):

  ```python
                          "expected_deltas": {
                              "consciousness_delta": None,
                              "heat_delta": attack_heat_gain,
                          },
  ```

  (`ActionType` and `GameDefines` are already module imports — `engine_bridge.py:25,43`.)

  4. `web/game/serializers.py` — insert before `EducateTargetSerializer` (line 468):

  ```python
  class ExpectedDeltasSerializer(serializers.Serializer[dict[str, Any]]):
      """Per-target expected deltas (spec-116 FR-116-4.4).

      Bridge-derived from the resolvers' own math. An axis is null when no
      per-target formula exists for that verb (honest absence, Constitution
      III.11) — never a fabricated 0.0.
      """

      consciousness_delta = serializers.FloatField(allow_null=True)
      heat_delta = serializers.FloatField(allow_null=True)
  ```

  Attach `expected_deltas = ExpectedDeltasSerializer(required=False)` to `EducateTargetSerializer`, `PopulationAidTargetSerializer`, `AttackTargetOrgSerializer`, and `AttackTargetInstitutionModelSerializer` (required=False: absence skips cleanly — rollout-safe and matches aid org rows).

  Reconcile the CONFIRMED bridge/serializer mismatches (fields the bridge never produces become `required=False`; note-only dicts gain a `note` passthrough):

  ```python
  class FeedforwardSerializer(serializers.Serializer[dict[str, Any]]):
      projected_routing_shift = FeedforwardRoutingShiftSerializer(required=False)
      state_ai_visibility = serializers.CharField(required=False)
      state_ai_likely_response = serializers.CharField(required=False)
      turns_to_dominant_tendency_shift = serializers.IntegerField(allow_null=True, required=False)
      turns_explanation = serializers.CharField(required=False)
      note = serializers.CharField(required=False)
  ```

  ```python
  class AidProjectionSerializer(serializers.Serializer[dict[str, Any]]):
      consumption_ratio_delta = serializers.FloatField(required=False)
      agitation_delta = serializers.FloatField(required=False)
      solidarity_added = serializers.FloatField(required=False)
      economism_risk = serializers.CharField(allow_null=True, required=False)
      note = serializers.CharField(required=False)
  ```

  In `AttackTargetOrgSerializer`: `description = serializers.CharField(required=False)` and `attack_projection = AttackProjectionSerializer(required=False)`. In `AttackTargetInstitutionModelSerializer`: `attack_projection = AttackProjectionSerializer(required=False)`.

  5. Seam registration. `src/babylon/sentinels/seam/types.py`: add `ACTION = "action"` after `DOCTRINE = "doctrine"` (line 92) and this cvar to the `SeamScope` docstring:

  ```
      :cvar ACTION: the ActionComposer verb-target rows — the per-verb GET
          ``.../actions/{verb}/targets/`` payloads' ``expected_deltas``
          sub-object (spec-116 FR-116-4.4), bridge-derived resolver-parity
          estimates rendered as TargetPicker row chips.
  ```

  `src/babylon/sentinels/seam/registry.py`: add before the `SEAM_REGISTRY` assembly (line 2016):

  ```python
  # ---------------------------------------------------------------------------
  # ACTION scope — per-target expected deltas on the verb-target rows
  # (spec-116 FR-116-4.4). One row for the shared sub-object across its three
  # emitters; the axis a verb has no formula for is an honest null.
  # ---------------------------------------------------------------------------

  _ACTION_EMITTERS: tuple[str, ...] = (
      "web/game/engine_bridge.py::EngineBridge.get_educate_targets",
      "web/game/engine_bridge.py::EngineBridge.get_aid_targets (population_targets)",
      "web/game/engine_bridge.py::EngineBridge.get_attack_targets (organizations+institutions)",
  )

  _ACTION_METRICS: tuple[SeamEntry, ...] = (
      SeamEntry(
          payload="verb_target_expected_deltas",
          wire_keys=("expected_deltas", "consciousness_delta", "heat_delta"),
          scope=SeamScope.ACTION,
          owner_layer=(
              "bridge-derived (babylon.ooda.action_effects.compute_consciousness_delta via "
              "_preview_consciousness_delta; OODADefines.attack_self_heat_gain)"
          ),
          liveness_class=LivenessClass.DECLARED_CONDITIONAL,
          liveness_condition=(
              "consciousness_delta live only on educate/aid population rows (the resolvers' "
              "own CI math); heat_delta live only on attack rows (the resolver's self-heat "
              "define); the opposite axis is an honest null, never a fabricated 0.0"
          ),
          dtype="json",
          read_paths=_ACTION_EMITTERS,
          derivation_site="web/game/engine_bridge.py::_preview_consciousness_delta",
          spec_ref="spec-116 FR-116-4.4",
          notes=(
              "Rendered as TargetPicker per-row chips (no blind picks). Campaign rows are "
              "snapshot-sourced (its targets GET 405s) and carry none; investigate/move/"
              "negotiate/reproduce rows carry none (no per-target resolver math)."
          ),
      ),
  )
  ```

  and append `+ _ACTION_METRICS` to the `SEAM_REGISTRY` concatenation (line 2016-2024). (Verified safe: the seam gating checks are MAP/tick/severity-scoped and Sensor-2 gates only MUST_BE_LIVE MAP rows, so a DECLARED_CONDITIONAL ACTION row cannot red an unrelated gate; the bridge sweep is advisory.)

- [ ] **Step 8: Verify green + seams, then commit (unit B)**

  ```bash
  mise run test:q -- tests/unit/web/test_engine_bridge.py::TestExpectedDeltas \
                     tests/unit/web/test_per_verb_views.py \
                     tests/unit/web/test_serializers.py \
                     tests/unit/sentinels/
  mise run check:seams
  ```

  Expected: all pass (including the pre-existing empty-row GET tests — required=False changes are strictly-widening); `check:seams` exits 0.

  ```bash
  git add web/game/engine_bridge.py web/game/serializers.py \
          src/babylon/sentinels/seam/types.py src/babylon/sentinels/seam/registry.py \
          tests/unit/web/test_engine_bridge.py tests/unit/web/test_per_verb_views.py
  mise run commit -- "feat(web): per-target expected_deltas on educate/aid/attack target rows (spec-116 FR-116-4.4)"
  ```

- [ ] **Step 9: Write the failing frontend tests**

  1. CREATE `src/frontend/src/components/action/TargetPicker.test.tsx`:

  ```tsx
  /**
   * TargetPicker — per-row expected-delta chips (spec-116 FR-116-4.4).
   * Pins the honest-null convention shared with VerbForm's DeltaChip:
   * rows without expectedDeltas render label+group only; a zero/absent
   * axis renders no chip; ▲ gold / ▼ crimson by sign.
   */
  import { describe, it, expect, vi } from "vitest";
  import { render, screen } from "@testing-library/react";
  import userEvent from "@testing-library/user-event";
  import type { VerbTarget } from "@/lib/verbs";
  import { TargetPicker } from "./TargetPicker";

  function renderPicker(targets: VerbTarget[]): ReturnType<typeof vi.fn> {
    const onSelect = vi.fn();
    render(
      <TargetPicker
        targets={targets}
        loading={false}
        error={null}
        selectedId={null}
        onSelect={onSelect}
      />,
    );
    return onSelect;
  }

  describe("TargetPicker expected-delta chips", () => {
    it("renders no chips for rows without expectedDeltas (honest null)", () => {
      renderPicker([{ id: "t1", label: "Downtown" }]);
      expect(screen.queryByTestId("target-delta")).not.toBeInTheDocument();
    });

    it("renders gold for a positive CI delta and crimson for a negative heat delta", () => {
      renderPicker([
        { id: "t1", label: "Downtown", expectedDeltas: { consciousness: 0.0123, heat: -0.05 } },
      ]);
      const chips = screen.getAllByTestId("target-delta");
      expect(chips).toHaveLength(2);
      expect(chips[0]).toHaveTextContent("▲CI +0.0123");
      expect(chips[0]!.className).toContain("text-accent-gold");
      expect(chips[1]).toHaveTextContent("▼Heat -0.05");
      expect(chips[1]!.className).toContain("text-accent-crimson");
    });

    it("hides zero axes and keeps the row clickable", async () => {
      const onSelect = renderPicker([
        { id: "t1", label: "Downtown", expectedDeltas: { consciousness: 0, heat: 0.1 } },
      ]);
      expect(screen.getAllByTestId("target-delta")).toHaveLength(1);
      await userEvent.click(screen.getByText("Downtown"));
      expect(onSelect).toHaveBeenCalledWith("t1");
    });
  });
  ```

  2. Append to `src/frontend/src/lib/verbs/__tests__/verbs.test.ts`:

  ```tsx
  describe("expected_deltas mapping (spec-116 FR-116-4.4)", () => {
    it("educate.parseTargets maps expected_deltas and drops null axes", () => {
      const config = VERB_REGISTRY.educate!;
      const targets = config.parseTargets({
        targets: [
          {
            community_id: "sc-1",
            territory_name: "Genesee",
            category: "social_class",
            credibility: 0.6,
            expected_deltas: { consciousness_delta: 0.0123, heat_delta: null },
          },
        ],
      });
      expect(targets[0]!.expectedDeltas).toEqual({ consciousness: 0.0123 });
    });

    it("attack.parseTargets maps the heat axis on org and institution rows", () => {
      const config = VERB_REGISTRY.attack!;
      const targets = config.parseTargets({
        targets: {
          organizations: [
            {
              target_id: "o1",
              name: "Citizens Council",
              expected_deltas: { consciousness_delta: null, heat_delta: 0.1 },
            },
          ],
          institutions: [
            {
              target_id: "i1",
              name: "County Court",
              expected_deltas: { consciousness_delta: null, heat_delta: 0.1 },
            },
          ],
          edges: [],
        },
      });
      expect(targets[0]!.expectedDeltas).toEqual({ heat: 0.1 });
      expect(targets[1]!.expectedDeltas).toEqual({ heat: 0.1 });
    });

    it("aid.parseTargets maps population rows; org rows honestly carry none", () => {
      const config = VERB_REGISTRY.aid!;
      const targets = config.parseTargets({
        population_targets: [
          {
            community_id: "sc-1",
            community_name: "Genesee Proles",
            expected_deltas: { consciousness_delta: 0.004, heat_delta: null },
          },
        ],
        org_targets: [{ org_id: "o2", org_name: "Tenants Union" }],
      });
      expect(targets[0]!.expectedDeltas).toEqual({ consciousness: 0.004 });
      expect(targets[1]!.expectedDeltas).toBeUndefined();
    });
  });
  ```

  Run to verify they fail:

  ```bash
  cd src/frontend && npx vitest run src/components/action/TargetPicker.test.tsx src/lib/verbs/__tests__/verbs.test.ts
  ```

  Expected: chip tests fail with `Unable to find an element by: [data-testid="target-delta"]` (plus a TS error on `expectedDeltas` until Step 10 lands — vitest surfaces it as a transform/type failure); mapping tests fail with `expected undefined to deeply equal { consciousness: 0.0123 }`.

- [ ] **Step 10: Implement the frontend seam**

  1. `src/frontend/src/lib/verbs/types.ts` — extend `VerbTarget` (lines 6-13):

  ```ts
  /** A target option parsed from the API's verb-target endpoint response. */
  export interface VerbTarget {
    /** Unique target identifier. */
    id: string;
    /** Human-readable label. */
    label: string;
    /** Optional group name for <optgroup> rendering. */
    group?: string;
    /**
     * Per-target expected deltas parsed from the row's `expected_deltas`
     * wire object (spec-116 FR-116-4.4). An axis is present only when the
     * backend produced a real number for it — a null wire axis means "no
     * per-target formula exists for this verb" and is dropped, never
     * coerced to 0 (honest absence, Constitution III.11).
     */
    expectedDeltas?: {
      consciousness?: number;
      heat?: number;
    };
  }
  ```

  2. CREATE `src/frontend/src/lib/verbs/expectedDeltas.ts`:

  ```ts
  /**
   * Shared wire→VerbTarget mapping for the `expected_deltas` row sub-object
   * the educate/aid/attack GET targets endpoints emit (spec-116 FR-116-4.4,
   * ExpectedDeltasSerializer in web/game/serializers.py). A null wire axis
   * means "no per-target formula exists for this verb" — it is dropped,
   * never coerced to 0 (honest absence, Constitution III.11).
   */

  import type { VerbTarget } from "./types";

  /** The row sub-object as serialized by ExpectedDeltasSerializer. */
  export interface WireExpectedDeltas {
    consciousness_delta?: number | null;
    heat_delta?: number | null;
  }

  export function parseExpectedDeltas(
    raw: WireExpectedDeltas | undefined,
  ): VerbTarget["expectedDeltas"] {
    if (!raw) return undefined;
    const out: NonNullable<VerbTarget["expectedDeltas"]> = {};
    if (typeof raw.consciousness_delta === "number" && Number.isFinite(raw.consciousness_delta)) {
      out.consciousness = raw.consciousness_delta;
    }
    if (typeof raw.heat_delta === "number" && Number.isFinite(raw.heat_delta)) {
      out.heat = raw.heat_delta;
    }
    return Object.keys(out).length > 0 ? out : undefined;
  }
  ```

  3. `educate.ts` — add `expected_deltas?: WireExpectedDeltas;` to the `EducateTarget` interface, import `{ parseExpectedDeltas, type WireExpectedDeltas } from "./expectedDeltas"`, and extend the mapper:

  ```ts
      return targets.map((t) => ({
        id: t.community_id,
        label: `${t.territory_name} (${t.category} — Credibility: ${t.credibility})`,
        expectedDeltas: parseExpectedDeltas(t.expected_deltas),
      }));
  ```

  4. `aid.ts` — same import; `AidPopTarget` gains `expected_deltas?: WireExpectedDeltas;`; only the `popTargets.map` gains `expectedDeltas: parseExpectedDeltas(t.expected_deltas),` (org rows untouched — they never carry the key).

  5. `attack.ts` — same import; `AttackTargetEntry` gains `expected_deltas?: WireExpectedDeltas;`; both the `organizations` and `institutions` mappers gain `expectedDeltas: parseExpectedDeltas(t.expected_deltas),` (edges untouched).

  6. `src/frontend/src/components/action/TargetPicker.tsx` — add the chip renderer above the component and render chips in the row (docstring gains the FR-116-4.4 sentence from the test file header):

  ```tsx
  /** One compact ▲/▼ chip for a non-zero expected delta — null otherwise
   *  (the same honest-null convention as VerbForm's preview DeltaChip). */
  function TargetDeltaChip({
    value,
    label,
  }: {
    value: number | undefined;
    label: string;
  }): React.JSX.Element | null {
    if (value === undefined || !Number.isFinite(value) || value === 0) return null;
    const up = value > 0;
    return (
      <span
        data-testid="target-delta"
        title={`${label}: ${up ? "+" : ""}${value}`}
        className={`font-mono text-[9px] ${up ? "text-accent-gold" : "text-accent-crimson"}`}
      >
        {up ? "▲" : "▼"}
        {label} {up ? "+" : "-"}
        {parseFloat(Math.abs(value).toPrecision(3))}
      </span>
    );
  }
  ```

  and replace the row body (lines 49-50) with:

  ```tsx
            <span className="truncate">{t.label}</span>
            <span className="ml-2 flex shrink-0 items-center gap-1.5">
              <TargetDeltaChip value={t.expectedDeltas?.consciousness} label="CI" />
              <TargetDeltaChip value={t.expectedDeltas?.heat} label="Heat" />
              {t.group && <span className="text-[9px] text-ash">{t.group}</span>}
            </span>
  ```

  (Row height stays stable inside the `max-h-40` scroll box — chips are inline `text-[9px]` spans in the existing flex row; no test pins the old group-span nesting, verified.)

- [ ] **Step 11: Verify green, full gates, commit (unit C)**

  ```bash
  cd src/frontend && npx vitest run src/components/action/TargetPicker.test.tsx \
      src/lib/verbs/__tests__/verbs.test.ts src/components/action/VerbForm.test.tsx
  cd src/frontend && npm run check
  ```

  Expected: all green (`Test Files 3 passed` on the scoped run; tsc/eslint/prettier/vitest green on the gate). Then the repo fast gate (single-flight, never parallel with other agents):

  ```bash
  mise run check
  ```

  Expected: green — includes `check:seams` with the new ACTION row and the full `test:unit` leg.

  ```bash
  git add src/frontend/src/lib/verbs/types.ts src/frontend/src/lib/verbs/expectedDeltas.ts \
          src/frontend/src/lib/verbs/educate.ts src/frontend/src/lib/verbs/aid.ts \
          src/frontend/src/lib/verbs/attack.ts \
          src/frontend/src/components/action/TargetPicker.tsx \
          src/frontend/src/components/action/TargetPicker.test.tsx \
          src/frontend/src/lib/verbs/__tests__/verbs.test.ts
  mise run commit -- "feat(cockpit): per-target expected-delta chips in TargetPicker (spec-116 FR-116-4.4)"
  ```
## Cluster E — Dark data (FR-116-4.5 + 4.6 + 4.9 + the Trends-empty bug)

Binding facts from recon (they correct the spec's stale figures — cite these, not the audit):

- **16** tick_* attrs remain never-serialized (not "26"): Group C circulation (7, Feature 023) +
  Group D financial distribution (9, Feature 024). All 16 are `STRUCTURALLY_IMPOSSIBLE` in
  `SEAM_REGISTRY` — their engine writes are fallback constants until `turnover_profile_source`
  (`domain/economics/tick/system/__init__.py:1050`) and `interest_calculator` (`:1248`) are wired.
  Wiring those services is an ENGINE change (baseline drift) and is **out of scope** for this
  cluster (spec constraint: observe-only outside the one FR-116-1 ceremony). Ruling: serialize
  them **declared-dark** — the wire exists and is honest; the registry rows keep
  `STRUCTURALLY_IMPOSSIBLE`; **no UI chrome is built on frozen constants**.
- **0** payload phantoms on `/economy/`: all 8 named `EconomyDashboardPayload` fields are emitted
  today. The audit's dead chips at tick 26 were pre-boundary honesty (profit/occ stamp at year
  boundaries, tick 52 at weekly cadence) plus the un-migrated-DB session. 4d.6 therefore becomes
  a **pin-the-contract** task, not a rewiring task.
- **Trends-empty root cause** (verified in the live DB): the web 5432 `tick_summary` lacks the
  0033/0034 columns (`price_log`/`fictitious_log`/`market_corrections`). Migrations are applied
  ONLY by the headless runner; the web path runs `init_schema` whose `CREATE TABLE IF NOT EXISTS`
  no-ops on the pre-existing table. Both the INSERT and SELECT now name the missing columns, both
  raise `UndefinedColumn`, and both legs are swallowed by design (`logger.exception`) → `rows=[]`
  → the honest-empty Trends tab forever.
- **Series cadence rule** (from the brief, non-negotiable): tick_* writes happen at year
  boundaries only and are carried forward between boundaries. Every history series in this
  cluster is therefore **NULL before the first boundary, a step function after** — honest sparse,
  never fabricated smoothing (Constitution III.11).
- **County-dedup rule**: every territory in a county carries the SAME county-level tick_* stamps
  (`_carry_tick_dynamics_flows`), so per-territory aggregation inflates county quantities N-fold
  (the documented `_county_flow_snapshot` hazard, `web/game/engine_bridge.py:6587`). All new
  tick_summary aggregates dedupe by `county_fips`.

Placement note: the 16 dead-attr serialization lands in Task 20 (with the economy contract work)
rather than Task 19 — Task 19 owns the tick_summary spine (Trends fix + history series), Task 21
owns the PROFIT chip and widget consumption. All assigned content is covered across 19–21.

Determinism: every change in this cluster is bridge/persistence/frontend-side (serialization
boundary) — byte-safe by construction. No engine, formula, or defines file is touched;
`qa:regression` must stay byte-identical (verified once at cluster end, Task 21 Step 9).

---

### Task 19: Trends root-cause fix + tick_* history series (the tick_summary spine)

**Files:**
- Modify: `web/game/engine_bridge.py:8646-8687` (init_persistence — apply packaged migrations),
  `:7115-7167` (`_build_tick_summary` — graph-aggregate columns), `:7236` (thread `graph=` into
  the summary call), `:2615-2670` (`get_game_timeseries` — five new arrays), new helper after
  `_county_flow_snapshot` (~`:6620`)
- Create: `src/babylon/persistence/migrations/0035_playability_series.sql`
- Modify: `src/babylon/persistence/postgres_schema.py:369-398` (TICK_SUMMARY_DDL columns)
- Modify: `src/babylon/persistence/postgres_runtime/_legacy.py:842-942` (INSERT/SELECT columns)
- Modify: `web/game/stub_bridge.py:977-978` (timeseries shape parity — the `{"data": []}` drift
  would make `TimeseriesChart`'s `data.ticks.length` throw if the stub ever serves the cockpit)
- Modify: `src/babylon/sentinels/seam/registry.py` (new `_ECONOMY_SERIES_METRICS` tuple + add it
  to the `SEAM_REGISTRY` concatenation at `:2016-2024`)
- Modify: `src/frontend/src/types/game.ts:930-951` (TimeseriesPayload),
  `src/frontend/src/test/fixtures.ts:398-415` (makeTimeseriesPayload),
  `src/frontend/src/test/handlers.ts:162-182` (timeseries MSW handler)
- Test: `tests/integration/web/test_full_persistence.py` (migration heal, live PG),
  `tests/unit/web/test_engine_bridge.py` (aggregate unit tests),
  `tests/integration/test_timeseries_endpoint.py` (series arrays, in-process stub)

**Interfaces:**
- Consumes: nothing from other tasks (independent; touches no Cluster-A surface).
- Produces (Tasks 20/21 and the frontend rely on these exact names):
  - `_apply_runtime_migrations(pool: Any) -> None` — module function in `web/game/engine_bridge.py`.
  - `_build_tick_summary(state: WorldState, organizations: list[dict[str, Any]], *, graph: Any = None) -> dict[str, Any]`
    — five new summary keys: `crisis_pop_share`, `bifurcation_score_mean`,
    `wage_compression_mean`, `capital_stock_total`, `unemployment_rate_mean` (all `float | None`).
  - `_county_tick_series_aggregates(graph: Any) -> dict[str, float | None]` — same five keys.
  - `get_game_timeseries` payload gains five parallel arrays under the same five names.
  - TS `TimeseriesPayload` gains the same five as optional `(number | null)[]` fields.
  - Migration `0035_playability_series.sql`; five new `tick_summary` FLOAT columns.
  - Five new `SeamEntry` rows, scope=`ECONOMY`, keys `economy.crisis_pop_share` etc.

- [ ] **Step 1: Write the failing migration-heal test** (append to
  `tests/integration/web/test_full_persistence.py`; requires the :5433 test Postgres —
  `mise run db:up` first):

```python
class TestWebPathMigrations:
    """Playability Spine Task 19 — the Trends-empty root cause (spec-116).

    The live web DB was created before migrations 0033/0034, and the web path
    never applies ``persistence/migrations/00*.sql`` — ``init_schema``'s
    ``CREATE TABLE IF NOT EXISTS`` no-ops on the pre-existing ``tick_summary``,
    so ``persist_tick_summary``/``query_tick_summary_series`` raised
    ``UndefinedColumn`` on every call and BOTH legs were swallowed by their
    best-effort catches (``logger.exception``) — the permanent "No timeseries
    data yet" Trends tab. Assert on ROWS/COLUMNS, never on the absence of
    exceptions (both failure legs are silent by design).
    """

    def test_apply_runtime_migrations_heals_pre_0033_tick_summary(self, bridge: object) -> None:
        """Simulate the pre-0033 web DB, then prove the web applier heals it."""
        from game.engine_bridge import _apply_runtime_migrations

        persistence = bridge._persistence  # noqa: SLF001
        pool = persistence._pool  # noqa: SLF001
        dropped = ("price_log", "fictitious_log", "market_corrections")
        with pool.connection() as conn:
            conn.autocommit = True
            for column in dropped:
                conn.execute(f"ALTER TABLE tick_summary DROP COLUMN IF EXISTS {column}")
            # ensure_ddl_applied is digest-stamped: clear the stamps so the
            # applier cannot fast-path past the freshly-broken table. Guarded
            # DDL makes the later re-apply idempotent for sibling tests.
            conn.execute("DELETE FROM _babylon_schema_stamp")

        _apply_runtime_migrations(pool)

        with pool.connection() as conn:
            rows = conn.execute(
                "SELECT column_name FROM information_schema.columns "
                "WHERE table_name = 'tick_summary'"
            ).fetchall()
        columns = {row[0] for row in rows}
        for column in dropped:
            assert column in columns, f"migration did not restore tick_summary.{column}"

    def test_summary_write_and_series_read_round_trip_on_healed_schema(
        self, bridge: object
    ) -> None:
        """The exact write/read pair that failed silently on the live 5432 DB."""
        session_id = bridge.create_game(scenario="wayne_county", rng_seed=0)
        bridge.resolve_tick(session_id)

        ts = bridge.get_game_timeseries(session_id)

        assert ts["ticks"] == [0, 1], (
            "tick_summary rows must exist and be readable — a swallowed "
            "UndefinedColumn on either leg yields [] here"
        )
```

- [ ] **Step 2: Run test to verify it fails**

```bash
POSTGRES_HOST=localhost POSTGRES_PORT=5433 POSTGRES_DB=babylon_test \
POSTGRES_USER=test POSTGRES_PASSWORD=test \
mise run test:q -- tests/integration/web/test_full_persistence.py::TestWebPathMigrations
```

Expected: `ImportError: cannot import name '_apply_runtime_migrations' from 'game.engine_bridge'`.

- [ ] **Step 3: Write minimal implementation** — in `web/game/engine_bridge.py`, insert directly
  above `init_persistence` (after the module-level `_pool: Any = None` at `:8643`):

```python
def _apply_runtime_migrations(pool: Any) -> None:
    """Apply every packaged runtime migration to the web database.

    Playability Spine Task 19 (spec-116, the Trends-empty root cause): the
    headless runner has applied ``persistence/migrations/00*.sql`` on every
    start (``headless_runner.runner._apply_migrations``), but the web path
    only ever ran ``init_schema`` — whose ``CREATE TABLE IF NOT EXISTS``
    no-ops on a pre-existing table, so ALTER-bearing migrations (0033/0034's
    ``tick_summary`` columns) never reached the live DB and both
    ``persist_tick_summary`` and ``query_tick_summary_series`` raised
    ``UndefinedColumn`` into their best-effort catches forever.

    Mirrors the runner exactly: package-relative glob (never the process
    CWD), sorted apply order, and :func:`ensure_ddl_applied` — digest-stamped
    and advisory-locked, NEVER a bare loop over migration files (ADR074 law;
    the no-stamp-on-failure semantics are what the test-conftest self-heal
    contract depends on).

    :param pool: An open ``psycopg_pool.ConnectionPool`` for the web DB.
    :raises RuntimeError: If the packaged migrations directory is empty or
        missing — refusing to run unmigrated is louder than degrading.
    """
    from pathlib import Path

    import babylon.persistence as _persistence_pkg
    from babylon.persistence.postgres_schema import ensure_ddl_applied

    migrations_dir = Path(_persistence_pkg.__file__).resolve().parent / "migrations"
    sql_files = sorted(migrations_dir.glob("00*.sql"))
    if not sql_files:
        raise RuntimeError(
            f"No migrations found at {migrations_dir} — refusing to run unmigrated"
        )
    with pool.connection() as conn:
        conn.autocommit = True
        ensure_ddl_applied(conn, [sql_file.read_text() for sql_file in sql_files])
```

Then edit `init_persistence` (`:8673-8674`):

```python
    try:
        persistence.init_schema()
        # Playability Spine Task 19 (spec-116): the web DB must also receive
        # the ALTER-bearing migration files — init_schema alone left the live
        # tick_summary missing the 0033/0034 columns (the Trends-empty bug).
        _apply_runtime_migrations(_pool)
    except (psycopg.Error, RuntimeError) as exc:
```

(the existing `except (psycopg.Error, RuntimeError)` block and its degraded-state `logger.error`
are unchanged — migration failure shares the loud-but-bootable contract).

- [ ] **Step 4: Run test to verify it passes, then commit A**

```bash
POSTGRES_HOST=localhost POSTGRES_PORT=5433 POSTGRES_DB=babylon_test \
POSTGRES_USER=test POSTGRES_PASSWORD=test \
mise run test:q -- tests/integration/web/test_full_persistence.py
git add web/game/engine_bridge.py tests/integration/web/test_full_persistence.py
mise run commit -- "fix(web): apply runtime migrations on web boot — Trends-empty root cause (spec-116)"
```

Expected: all `test_full_persistence.py` tests pass (`N passed`).

- [ ] **Step 5: Write the failing series tests.** (a) Append to
  `tests/unit/web/test_engine_bridge.py` (after `TestBuildTickSummaryMarketAxis`, `:3113`):

```python
@pytest.mark.unit
class TestBuildTickSummarySeriesAggregates:
    """Task 19 (spec-116 4d.5): county-deduped tick_* aggregates ride tick_summary.

    The series is HONEST-SPARSE by design: tick_* attrs stamp at year
    boundaries only (weekly campaign => yearly points) and are carried
    forward between boundaries — NULL before the first boundary, a step
    function after, never fabricated smoothing (Constitution III.11).
    """

    _SERIES_KEYS = (
        "crisis_pop_share",
        "bifurcation_score_mean",
        "wage_compression_mean",
        "capital_stock_total",
        "unemployment_rate_mean",
    )

    @staticmethod
    def _graph_with_two_counties() -> BabylonGraph:
        graph = BabylonGraph()
        # T1/T2 share one county and carry IDENTICAL county-level stamps —
        # they must count ONCE (the _county_flow_snapshot N-fold-inflation
        # hazard), never once per territory.
        graph.add_node(
            "T1", node_type="territory", county_fips="26163", population=1_000_000,
            tick_crisis_phase="deep", tick_bifurcation_score=-0.5,
            tick_wage_compression=0.2, tick_capital_stock=1e9,
            tick_unemployment_rate=0.10,
        )
        graph.add_node(
            "T2", node_type="territory", county_fips="26163", population=500_000,
            tick_crisis_phase="deep", tick_bifurcation_score=-0.5,
            tick_wage_compression=0.2, tick_capital_stock=1e9,
            tick_unemployment_rate=0.10,
        )
        graph.add_node(
            "T3", node_type="territory", county_fips="26125", population=500_000,
            tick_crisis_phase="normal", tick_bifurcation_score=0.3,
            tick_wage_compression=0.0, tick_capital_stock=2e9,
            tick_unemployment_rate=0.05,
        )
        return graph

    def test_aggregates_are_county_deduped_and_population_weighted(self) -> None:
        from babylon.models.world_state import WorldState

        from game.engine_bridge import _build_tick_summary

        summary = _build_tick_summary(
            WorldState(tick=52), organizations=[], graph=self._graph_with_two_counties()
        )

        # Wayne pop 1.5M (deep) vs 26125 pop 0.5M (normal): 1.5/2.0.
        assert summary["crisis_pop_share"] == pytest.approx(0.75)
        # Weighted over COUNTIES: (-0.5 * 1.5e6 + 0.3 * 0.5e6) / 2e6.
        assert summary["bifurcation_score_mean"] == pytest.approx(-0.3)
        assert summary["wage_compression_mean"] == pytest.approx(0.15)
        # Extensive sum, ONE term per county: 1e9 + 2e9 — never 1e9*2 + 2e9.
        assert summary["capital_stock_total"] == pytest.approx(3e9)
        assert summary["unemployment_rate_mean"] == pytest.approx(0.0875)

    def test_no_graph_or_no_boundary_yet_is_honest_null(self) -> None:
        from babylon.models.world_state import WorldState

        from game.engine_bridge import _build_tick_summary

        no_graph = _build_tick_summary(WorldState(tick=1), organizations=[])
        bare_graph = BabylonGraph()
        bare_graph.add_node("T1", node_type="territory", county_fips="26163", population=10)
        pre_boundary = _build_tick_summary(WorldState(tick=1), organizations=[], graph=bare_graph)

        for key in self._SERIES_KEYS:
            assert no_graph[key] is None, f"{key} must be NULL without a graph"
            assert pre_boundary[key] is None, f"{key} must be NULL before the first boundary"
```

(b) Append to `tests/integration/test_timeseries_endpoint.py`:

```python
class TestCrisisSeries:
    """Task 19 (spec-116 4d.5): the crisis/bifurcation history arrays."""

    def test_series_arrays_ride_the_payload(self) -> None:
        rows = [
            {
                "tick": 0,
                "crisis_pop_share": None,
                "bifurcation_score_mean": None,
                "wage_compression_mean": None,
                "capital_stock_total": None,
                "unemployment_rate_mean": None,
            },
            {
                "tick": 1,
                "crisis_pop_share": 0.75,
                "bifurcation_score_mean": -0.3,
                "wage_compression_mean": 0.15,
                "capital_stock_total": 3e9,
                "unemployment_rate_mean": 0.0875,
            },
        ]
        bridge = EngineBridge(persistence=_StubPersistence(rows))
        out = bridge.get_game_timeseries(uuid.uuid4())

        assert out["crisis_pop_share"] == [None, 0.75]
        assert out["bifurcation_score_mean"] == [None, -0.3]
        assert out["wage_compression_mean"] == [None, 0.15]
        assert out["capital_stock_total"] == [None, 3e9]
        assert out["unemployment_rate_mean"] == [None, 0.0875]

    def test_rows_without_series_columns_chart_as_gaps(self) -> None:
        """Pre-0035 rows (rollout skew) become None slots, never 0.0."""
        rows = [{"tick": 0, "imperial_rent": 1.0}]
        bridge = EngineBridge(persistence=_StubPersistence(rows))
        out = bridge.get_game_timeseries(uuid.uuid4())
        assert out["crisis_pop_share"] == [None]
        assert out["unemployment_rate_mean"] == [None]
```

- [ ] **Step 6: Run tests to verify they fail**

```bash
mise run test:q -- tests/unit/web/test_engine_bridge.py::TestBuildTickSummarySeriesAggregates \
  tests/integration/test_timeseries_endpoint.py::TestCrisisSeries
```

Expected: `TypeError: _build_tick_summary() got an unexpected keyword argument 'graph'` and
`KeyError: 'crisis_pop_share'`.

- [ ] **Step 7: Write the implementation** (seven edits, one logical unit):

(7a) Create `src/babylon/persistence/migrations/0035_playability_series.sql`:

```sql
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
```

(7b) `src/babylon/persistence/postgres_schema.py` — in `TICK_SUMMARY_DDL`, after
`market_corrections INT,` (`:395`):

```sql
    -- Playability Spine Task 19 (spec-116 4d.5): county-deduped crisis/
    -- bifurcation history; NULL = no county boundary state that tick.
    -- Existing DBs: migrations/0035.
    crisis_pop_share FLOAT,
    bifurcation_score_mean FLOAT,
    wage_compression_mean FLOAT,
    capital_stock_total FLOAT,
    unemployment_rate_mean FLOAT,
```

(7c) `src/babylon/persistence/postgres_runtime/_legacy.py` — replace both method bodies in full.
`persist_tick_summary` (`:842-907`):

```python
    def persist_tick_summary(
        self,
        tick: int,
        summary: dict[str, Any],
        *,
        session_id: UUID,
    ) -> None:
        """Persist pre-aggregated tick summary."""
        with self._pool.connection() as conn:
            conn.execute(
                """
                INSERT INTO tick_summary
                    (session_id, tick, year, total_c, total_v, total_s,
                     exploitation_rate, profit_rate, imperial_rent,
                     avg_consciousness, solidarity_edge_count,
                     antagonistic_edge_count, co_optive_edge_count,
                     org_count, player_org_count, uprising_count,
                     repression_count, conservation_check,
                     price_log, fictitious_log, market_corrections,
                     crisis_pop_share, bifurcation_score_mean,
                     wage_compression_mean, capital_stock_total,
                     unemployment_rate_mean)
                VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s,
                        %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
                ON CONFLICT (session_id, tick) DO UPDATE SET
                    year = EXCLUDED.year, total_c = EXCLUDED.total_c,
                    total_v = EXCLUDED.total_v, total_s = EXCLUDED.total_s,
                    exploitation_rate = EXCLUDED.exploitation_rate,
                    profit_rate = EXCLUDED.profit_rate,
                    imperial_rent = EXCLUDED.imperial_rent,
                    avg_consciousness = EXCLUDED.avg_consciousness,
                    solidarity_edge_count = EXCLUDED.solidarity_edge_count,
                    antagonistic_edge_count = EXCLUDED.antagonistic_edge_count,
                    co_optive_edge_count = EXCLUDED.co_optive_edge_count,
                    org_count = EXCLUDED.org_count,
                    player_org_count = EXCLUDED.player_org_count,
                    uprising_count = EXCLUDED.uprising_count,
                    repression_count = EXCLUDED.repression_count,
                    conservation_check = EXCLUDED.conservation_check,
                    price_log = EXCLUDED.price_log,
                    fictitious_log = EXCLUDED.fictitious_log,
                    market_corrections = EXCLUDED.market_corrections,
                    crisis_pop_share = EXCLUDED.crisis_pop_share,
                    bifurcation_score_mean = EXCLUDED.bifurcation_score_mean,
                    wage_compression_mean = EXCLUDED.wage_compression_mean,
                    capital_stock_total = EXCLUDED.capital_stock_total,
                    unemployment_rate_mean = EXCLUDED.unemployment_rate_mean
                """,
                (
                    session_id,
                    tick,
                    summary.get("year"),
                    summary.get("total_c"),
                    summary.get("total_v"),
                    summary.get("total_s"),
                    summary.get("exploitation_rate"),
                    summary.get("profit_rate"),
                    summary.get("imperial_rent"),
                    summary.get("avg_consciousness"),
                    summary.get("solidarity_edge_count"),
                    summary.get("antagonistic_edge_count"),
                    summary.get("co_optive_edge_count"),
                    summary.get("org_count"),
                    summary.get("player_org_count"),
                    summary.get("uprising_count"),
                    summary.get("repression_count"),
                    summary.get("conservation_check"),
                    # Program 23 (ADR077): NULL when the market axis is
                    # absent that tick — honest absence, never 0.0.
                    summary.get("price_log"),
                    summary.get("fictitious_log"),
                    # ADR078: cumulative snap count, same NULL contract.
                    summary.get("market_corrections"),
                    # Playability Spine Task 19 (spec-116 4d.5): county-
                    # deduped year-boundary aggregates, same NULL contract.
                    summary.get("crisis_pop_share"),
                    summary.get("bifurcation_score_mean"),
                    summary.get("wage_compression_mean"),
                    summary.get("capital_stock_total"),
                    summary.get("unemployment_rate_mean"),
                ),
            )
```

`query_tick_summary_series` (`:909-942`) — the SELECT list becomes:

```python
            cur.execute(
                """
                SELECT tick, year, total_c, total_v, total_s,
                       exploitation_rate, profit_rate, imperial_rent,
                       avg_consciousness, solidarity_edge_count,
                       antagonistic_edge_count, co_optive_edge_count,
                       org_count, player_org_count, uprising_count,
                       repression_count, conservation_check,
                       price_log, fictitious_log, market_corrections,
                       crisis_pop_share, bifurcation_score_mean,
                       wage_compression_mean, capital_stock_total,
                       unemployment_rate_mean
                FROM tick_summary
                WHERE session_id = %s
                ORDER BY tick
                """,
                (session_id,),
            )
```

(7d) `web/game/engine_bridge.py` — insert after `_county_flow_snapshot` (~`:6620`):

```python
#: Crisis phases that count as "in crisis" for the summary series — mirrors
#: the frontend strip's CRISIS_IN_PROGRESS_PHASES (CrisisTimeline.tsx).
_CRISIS_ACTIVE_PHASES: frozenset[str] = frozenset({"onset", "early", "deep"})


def _county_tick_series_aggregates(graph: Any) -> dict[str, float | None]:
    """County-deduped aggregates of the year-boundary ``tick_*`` attrs.

    Playability Spine Task 19 (spec-116 4d.5): the ``tick_summary`` history
    columns behind the CrisisTimeline / BifurcationGauge sparklines. Every
    territory in a county carries the SAME county-level ``tick_*`` stamps
    (:func:`_carry_tick_dynamics_flows`), so aggregating per TERRITORY would
    inflate every county quantity N-fold — the documented
    :func:`_county_flow_snapshot` hazard. This dedupes to one representative
    value per ``county_fips`` (first non-``None`` seen per attr), weights the
    intensive means by summed county population (plain mean when no weight),
    and sums the extensive capital stock.

    Honest-sparse by construction (Constitution III.11): ``tick_*`` attrs
    stamp at year boundaries only and carry forward between them, so every
    aggregate is ``None`` before the first boundary this session and a step
    function after — never a fabricated 0.0, never smoothed.

    :param graph: A live post-tick graph whose territory nodes may carry the
        ``tick_*`` attrs.
    :returns: Dict with ``crisis_pop_share`` / ``bifurcation_score_mean`` /
        ``wage_compression_mean`` / ``capital_stock_total`` /
        ``unemployment_rate_mean``, each ``float | None``.
    """
    pops: dict[str, float] = {}
    reps: dict[str, dict[str, Any]] = {}
    for _node_id, data in graph.nodes(data=True):
        if data.get("_node_type") != "territory":
            continue
        fips = data.get("county_fips")
        if not fips:
            continue
        pops[fips] = pops.get(fips, 0.0) + max(0.0, float(data.get("population") or 0))
        rep = reps.setdefault(fips, {})
        for key in (
            "tick_crisis_phase",
            "tick_bifurcation_score",
            "tick_wage_compression",
            "tick_capital_stock",
            "tick_unemployment_rate",
        ):
            if rep.get(key) is None and data.get(key) is not None:
                rep[key] = data[key]

    def weighted_mean(key: str) -> float | None:
        rows = [
            (float(rep[key]), pops[fips])
            for fips, rep in reps.items()
            if rep.get(key) is not None
        ]
        if not rows:
            return None
        total_weight = sum(weight for _value, weight in rows)
        if total_weight > 0:
            return sum(value * weight for value, weight in rows) / total_weight
        return sum(value for value, _weight in rows) / len(rows)

    phased = [
        (rep["tick_crisis_phase"], pops[fips])
        for fips, rep in reps.items()
        if rep.get("tick_crisis_phase") is not None
    ]
    crisis_pop_share: float | None = None
    if phased:
        total = sum(weight for _phase, weight in phased)
        if total > 0:
            crisis_pop_share = (
                sum(weight for phase, weight in phased if phase in _CRISIS_ACTIVE_PHASES) / total
            )
        else:
            crisis_pop_share = sum(
                1 for phase, _weight in phased if phase in _CRISIS_ACTIVE_PHASES
            ) / len(phased)

    capitals = [
        float(rep["tick_capital_stock"])
        for rep in reps.values()
        if rep.get("tick_capital_stock") is not None
    ]
    return {
        "crisis_pop_share": crisis_pop_share,
        "bifurcation_score_mean": weighted_mean("tick_bifurcation_score"),
        "wage_compression_mean": weighted_mean("tick_wage_compression"),
        "capital_stock_total": sum(capitals) if capitals else None,
        "unemployment_rate_mean": weighted_mean("tick_unemployment_rate"),
    }
```

Then change `_build_tick_summary`'s signature (`:7115`) and return (`:7144-7167`):

```python
def _build_tick_summary(
    state: WorldState,
    organizations: list[dict[str, Any]],
    *,
    graph: Any = None,
) -> dict[str, Any]:
```

(add to its docstring, after the existing sources paragraph:)

```
    Playability Spine Task 19 (spec-116 4d.5): when ``graph`` is supplied
    (the resolve path), five county-deduped year-boundary aggregates ride
    the row via :func:`_county_tick_series_aggregates`; without a graph
    (bootstrap call sites) they are honest ``None`` — tick-0 has no
    TickDynamics output.
```

(and append inside the returned dict, after `"market_corrections"`:)

```python
        # Playability Spine Task 19 (spec-116 4d.5): county-deduped crisis/
        # bifurcation history. NULL without a graph or before the first year
        # boundary — honest sparse (step function), never smoothed.
        **(
            _county_tick_series_aggregates(graph)
            if graph is not None
            else {
                "crisis_pop_share": None,
                "bifurcation_score_mean": None,
                "wage_compression_mean": None,
                "capital_stock_total": None,
                "unemployment_rate_mean": None,
            }
        ),
```

Update the call site in `_persist_snapshots_safe` (`:7236`):

```python
        summary_fn(
            state.tick,
            _build_tick_summary(state, organizations, graph=graph),
            session_id=session_id,
        )
```

In `get_game_timeseries`, declare after `market_corrections` (`:2627`):

```python
        crisis_pop_share: list[float | None] = []
        bifurcation_score_mean: list[float | None] = []
        wage_compression_mean: list[float | None] = []
        capital_stock_total: list[float | None] = []
        unemployment_rate_mean: list[float | None] = []
```

append inside the row loop (after the `market_corrections.append(...)` at `:2653-2655`):

```python
            # Task 19 (spec-116 4d.5): county-deduped crisis history — a
            # year-boundary step function; missing columns stay None (gaps).
            crisis_pop_share.append(_optional_float(row.get("crisis_pop_share")))
            bifurcation_score_mean.append(_optional_float(row.get("bifurcation_score_mean")))
            wage_compression_mean.append(_optional_float(row.get("wage_compression_mean")))
            capital_stock_total.append(_optional_float(row.get("capital_stock_total")))
            unemployment_rate_mean.append(_optional_float(row.get("unemployment_rate_mean")))
```

and extend the return dict (`:2656-2670`):

```python
            "crisis_pop_share": crisis_pop_share,
            "bifurcation_score_mean": bifurcation_score_mean,
            "wage_compression_mean": wage_compression_mean,
            "capital_stock_total": capital_stock_total,
            "unemployment_rate_mean": unemployment_rate_mean,
```

(7e) `web/game/stub_bridge.py:977-978` — shape parity (the `{"data": []}` drift would make
`TimeseriesChart` throw on `data.ticks.length` whenever the DEBUG stub serves the cockpit):

```python
    def get_game_timeseries(self, _session_id: UUID) -> dict[str, Any]:
        """Empty-but-real-shaped timeseries payload (parity with EngineBridge)."""
        return {
            "ticks": [],
            "imperial_rent": [],
            "consciousness": [],
            "solidarity": [],
            "heat": [],
            "wealth": [],
            "biocapacity": [],
            "value_produced": [],
            "surplus": [],
            "profit_rate": [],
            "price_index": [],
            "fictitious_ratio": [],
            "market_corrections": [],
            "crisis_pop_share": [],
            "bifurcation_score_mean": [],
            "wage_compression_mean": [],
            "capital_stock_total": [],
            "unemployment_rate_mean": [],
        }
```

(7f) `src/babylon/sentinels/seam/registry.py` — insert a new section before the
`SEAM_REGISTRY` assembly (`:2015`) and add `+ _ECONOMY_SERIES_METRICS` to the concatenation:

```python
# ---------------------------------------------------------------------------
# ECONOMY scope — the ``tick_summary`` history series behind ``/timeseries/``
# (Playability Spine Task 19, spec-116 4d.5). Wire keys are the parallel
# arrays ``EngineBridge.get_game_timeseries`` emits; each is a county-deduped
# aggregate ``_build_tick_summary`` persists per tick. First use of the
# ECONOMY scope.
# ---------------------------------------------------------------------------

_TIMESERIES_EMITTERS: tuple[str, ...] = (
    "web/game/engine_bridge.py::EngineBridge.get_game_timeseries",
    "src/frontend/src/components/chrome/CrisisTimeline.tsx (history sparkline)",
    "src/frontend/src/components/chrome/BifurcationGauge.tsx (history sparkline)",
)

_SERIES_CADENCE: str = (
    "non-null only for ticks persisted after the first year boundary this "
    "session stamped county tick_* state; carried forward between boundaries "
    "— a step-function series with a NULL head (weekly campaign = yearly "
    "points; honest sparse, never smoothed; Constitution III.11)"
)

_ECONOMY_SERIES_METRICS: tuple[SeamEntry, ...] = (
    SeamEntry(
        payload="crisis_pop_share",
        wire_keys=("crisis_pop_share",),
        scope=SeamScope.ECONOMY,
        owner_layer="bridge (county-deduped aggregate of tick_crisis_phase)",
        liveness_class=LivenessClass.DECLARED_CONDITIONAL,
        liveness_condition=_SERIES_CADENCE,
        dtype="float",
        write_site=(
            "web/game/engine_bridge.py::_county_tick_series_aggregates "
            "-> tick_summary.crisis_pop_share"
        ),
        read_paths=_TIMESERIES_EMITTERS,
        spec_ref="spec-116 4d.5 · ADR079",
        notes="Population share [0, 1] of counties in an active crisis phase (onset/early/deep).",
    ),
    SeamEntry(
        payload="bifurcation_score_mean",
        wire_keys=("bifurcation_score_mean",),
        scope=SeamScope.ECONOMY,
        owner_layer="bridge (county-deduped aggregate of tick_bifurcation_score)",
        liveness_class=LivenessClass.DECLARED_CONDITIONAL,
        liveness_condition=_SERIES_CADENCE,
        dtype="float",
        write_site=(
            "web/game/engine_bridge.py::_county_tick_series_aggregates "
            "-> tick_summary.bifurcation_score_mean"
        ),
        read_paths=_TIMESERIES_EMITTERS,
        spec_ref="spec-116 4d.5 · ADR079",
        notes=(
            "Population-weighted county mean of the political trajectory "
            "[-1 revolutionary, +1 fascist] (Feature 018 FR-011)."
        ),
    ),
    SeamEntry(
        payload="wage_compression_mean",
        wire_keys=("wage_compression_mean",),
        scope=SeamScope.ECONOMY,
        owner_layer="bridge (county-deduped aggregate of tick_wage_compression)",
        liveness_class=LivenessClass.DECLARED_CONDITIONAL,
        liveness_condition=_SERIES_CADENCE,
        dtype="float",
        write_site=(
            "web/game/engine_bridge.py::_county_tick_series_aggregates "
            "-> tick_summary.wage_compression_mean"
        ),
        read_paths=_TIMESERIES_EMITTERS,
        spec_ref="spec-116 4d.5 · ADR079",
        notes="Population-weighted county mean of cumulative wage compression [0, 1].",
    ),
    SeamEntry(
        payload="capital_stock_total",
        wire_keys=("capital_stock_total",),
        scope=SeamScope.ECONOMY,
        owner_layer="bridge (county-deduped SUM of tick_capital_stock)",
        liveness_class=LivenessClass.DECLARED_CONDITIONAL,
        liveness_condition=_SERIES_CADENCE,
        dtype="float",
        write_site=(
            "web/game/engine_bridge.py::_county_tick_series_aggregates "
            "-> tick_summary.capital_stock_total"
        ),
        read_paths=_TIMESERIES_EMITTERS,
        spec_ref="spec-116 4d.5 · ADR079",
        notes=(
            "EXTENSIVE: one term per county (never per territory — the "
            "_county_flow_snapshot N-fold hazard); a falling total is devaluation."
        ),
    ),
    SeamEntry(
        payload="unemployment_rate_mean",
        wire_keys=("unemployment_rate_mean",),
        scope=SeamScope.ECONOMY,
        owner_layer="bridge (county-deduped aggregate of tick_unemployment_rate)",
        liveness_class=LivenessClass.DECLARED_CONDITIONAL,
        liveness_condition=_SERIES_CADENCE,
        dtype="float",
        write_site=(
            "web/game/engine_bridge.py::_county_tick_series_aggregates "
            "-> tick_summary.unemployment_rate_mean"
        ),
        read_paths=_TIMESERIES_EMITTERS,
        spec_ref="spec-116 4d.5 · ADR079",
        notes="Population-weighted county mean of the BLS LAUS unemployment rate.",
    ),
)
```

and APPEND `+ _ECONOMY_SERIES_METRICS` as the last term of the existing
`SEAM_REGISTRY` sum — do NOT paste a full assembly block: by this point the sum
already carries Task 4's endgame_progress rows, Task 15's `_ENDGAME_METRICS`, and
Task 18's `_ACTION_METRICS`; every prior term must be retained. The edit is one line:

```python
    + _ECONOMY_SERIES_METRICS  # appended as the new final term; leave all prior terms
)
```

(Payload names deliberately have NO `tick_` prefix so the GATING
`check_tick_payloads_exist` — which AST-matches `tick_*` payloads against `graph_bridge.py`'s
write-set — does not apply; these are bridge-derived aggregates, not engine attrs.)

(7g) Frontend contract — `src/frontend/src/types/game.ts`, append inside `TimeseriesPayload`
(after `market_corrections?`, `:950`):

```ts
  /**
   * Playability Spine Task 19 (spec-116 4d.5) — county-deduped crisis/
   * bifurcation history off the year-boundary `tick_*` attrs. Step-function
   * series: `null` until the first year boundary (~tick 52 at weekly
   * cadence), then the last boundary's value carried forward — honest
   * sparse, never smoothed. Optional: pre-spine backends omit them
   * (rollout skew, the `market_corrections` precedent).
   */
  crisis_pop_share?: (number | null)[];
  bifurcation_score_mean?: (number | null)[];
  wage_compression_mean?: (number | null)[];
  capital_stock_total?: (number | null)[];
  unemployment_rate_mean?: (number | null)[];
```

`src/frontend/src/test/fixtures.ts` — in `makeTimeseriesPayload` after `market_corrections: [0, 0],`:

```ts
    crisis_pop_share: [null, 0.75],
    bifurcation_score_mean: [null, -0.3],
    wage_compression_mean: [null, 0.15],
    capital_stock_total: [null, 3e9],
    unemployment_rate_mean: [null, 0.0875],
```

`src/frontend/src/test/handlers.ts` — in the `/timeseries/` handler after
`market_corrections: [0, 0],`:

```ts
        crisis_pop_share: [null, 0.75],
        bifurcation_score_mean: [null, -0.3],
        wage_compression_mean: [null, 0.15],
        capital_stock_total: [null, 3e9],
        unemployment_rate_mean: [null, 0.0875],
```

(7h) Extend the Step-1 heal test's `dropped` tuple with the five 0035 columns (they now exist):

```python
        dropped = (
            "price_log",
            "fictitious_log",
            "market_corrections",
            "crisis_pop_share",
            "bifurcation_score_mean",
            "wage_compression_mean",
            "capital_stock_total",
            "unemployment_rate_mean",
        )
```

- [ ] **Step 8: Run tests to verify green**

```bash
mise run test:q -- tests/unit/web/test_engine_bridge.py \
  tests/integration/test_timeseries_endpoint.py tests/unit/web/test_stub_bridge_parity.py
POSTGRES_HOST=localhost POSTGRES_PORT=5433 POSTGRES_DB=babylon_test \
POSTGRES_USER=test POSTGRES_PASSWORD=test \
mise run test:q -- tests/integration/web/test_full_persistence.py
mise run check:seams
cd src/frontend && npx tsc --noEmit && npx vitest run src/components/timeseries && cd ../..
```

Expected: all green; `check:seams` exits 0 (the five ECONOMY rows cover the five new wire keys).

- [ ] **Step 9: Commit B**

```bash
git add src/babylon/persistence/migrations/0035_playability_series.sql \
  src/babylon/persistence/postgres_schema.py \
  src/babylon/persistence/postgres_runtime/_legacy.py \
  web/game/engine_bridge.py web/game/stub_bridge.py \
  src/babylon/sentinels/seam/registry.py \
  src/frontend/src/types/game.ts src/frontend/src/test/fixtures.ts \
  src/frontend/src/test/handlers.ts \
  tests/unit/web/test_engine_bridge.py tests/integration/test_timeseries_endpoint.py \
  tests/integration/web/test_full_persistence.py
mise run commit -- "feat(web): tick_summary crisis/bifurcation history series (spec-116 4d.5)"
```

Byte-safety: bridge/persistence/frontend only — no engine, formula, or defines change; the
5-scenario `qa:regression` baselines cannot move (verified at cluster end, Task 21 Step 9).

---

### Task 20: Economy dark data — Group C/D serialized declared-dark + the chip contract

**The 4d.6 wired-or-deleted ledger** (binding; the corrected audit found ZERO phantoms — every
chip is WIRED, none deleted; the contract tests below make regression impossible):

| Chip (EconomyDashboard) | Payload field | Real source | Ruling |
|---|---|---|---|
| Tick | `tick` | `state.tick` | WIRED (live from tick 0) |
| Value Produced | `value_produced` | `_aggregate_graph_economy` | WIRED |
| Rent Extracted | `rent_extracted` | `_aggregate_graph_economy` | WIRED |
| Exploitation | `exploitation_rate` | `_aggregate_graph_economy` (None-honest) | WIRED |
| Profit Rate | `profit_rate` | `_mean_territory_attr("tick_profit_rate")` | WIRED — "no data" before tick 52 is CORRECT (year-boundary cadence, Constitution III.11); do not "fix" |
| OCC | `occ` | `_mean_territory_attr("tick_occ")` | WIRED — same cadence honesty |
| Rent Pool | `imperial_rent_pool` | `state.economy` (ImperialRentSystem, real from tick 1) | WIRED |
| Super-Wage Rate | `current_super_wage_rate` | `state.economy` | WIRED |
| Wage Flow | `wage_flow_total` | `_sum_edge_value_flow_by_mode({"wages"})` — honest 0.0 when the scenario seeds no WAGES edges | WIRED |
| Tribute Flow | `tribute_flow_total` | `_sum_edge_value_flow_by_mode({"tribute"})` | WIRED |

No new chips are built on the Group C/D attrs — they carry write-site fallback constants until
`turnover_profile_source`/`interest_calculator` are wired (a chip of frozen 0.0s would be
dishonest chrome). Serialization-only, declared-dark.

**Files:**
- Modify: `web/game/engine_bridge.py:6238-6285` (`_carry_tick_dynamics_flows` boundary arm),
  `:6302-6339` (carry arm), `:7893-8030` (`_serialize_territory` — 16 keys + docstring paragraph)
- Modify: `src/babylon/sentinels/seam/registry.py:498-521` (emitter constants + gate strings),
  `:748-935` (the 16 Group C/D rows' `read_paths`)
- Modify: `src/frontend/src/types/game.ts:108-209` (TerritoryState — 16 optional fields)
- Create: `tests/unit/web/test_carry_group_c_d.py`
- Test (modify): `tests/unit/web/test_engine_bridge.py` (chip-contract class),
  `src/frontend/src/components/economy/EconomyDashboard.test.tsx` (chip-contract cases)

**Interfaces:**
- Consumes: nothing from Task 19 (independent surfaces; safe to execute in either order after 19).
- Produces:
  - 16 new territory wire keys, serialized under their registry-declared names (all keep the
    `tick_` prefix — none collides with an existing `_serialize_territory` key or Territory model
    field): `tick_liquidity_ratio`, `tick_commodity_overhang`, `tick_replacement_cycle`,
    `tick_inventory_diagnosis`, `tick_realization_crisis`, `tick_turnover_crisis`,
    `tick_reproduction_crisis`, `tick_interest_burden`, `tick_ground_rent`, `tick_rentier_share`,
    `tick_profit_of_enterprise`, `tick_financialization_share`, `tick_accumulated_debt`,
    `tick_claims_exceed_surplus`, `tick_housing_fictitious_fraction`,
    `tick_financial_crisis_signals`. These reuse their EXISTING SeamEntry rows (Groups C/D) —
    `read_paths` updated, liveness class unchanged.
  - Backend + frontend chip-contract tests pinning the `EconomyDashboardPayload` key set.

- [ ] **Step 1: Write the failing tests** — create `tests/unit/web/test_carry_group_c_d.py`
  (modeled byte-for-byte on `test_carry_group_a_b.py`):

```python
"""Playability Spine Task 20 (spec-116 4d.5): serialize Groups C/D declared-dark.

``write_tick_state_to_graph`` stamps the circulation family (Group C, Feature
023) and the financial-distribution family (Group D, Feature 024) onto
territory nodes at a year boundary (``graph_bridge.py:128-197``), but both
families were in NEITHER arm of ``_carry_tick_dynamics_flows`` — so on the
web path they evaporated on the very next ``WorldState`` round-trip
(``Territory`` is ``extra="forbid"``) — and ``_serialize_territory`` never
read them. The gating services (``turnover_profile_source`` /
``interest_calculator``) are still unwired, so the carried values are the
write-site FALLBACK CONSTANTS: declared-dark (the SEAM_REGISTRY rows stay
STRUCTURALLY_IMPOSSIBLE, never relabeled live) — but the wire must exist and
be honest so that wiring the services later lights the data with zero
serialization work.
"""

from __future__ import annotations

import pytest

from babylon.domain.economics.tick.types import (
    BifurcationRiskMetric,
    ClassDistribution,
    CountyEconomicState,
    CrisisPhase,
    CrisisState,
    NationalTickParameters,
)
from babylon.models.entities.territory import Territory
from babylon.models.enums import OperationalProfile, SectorType
from babylon.models.world_state import WorldState

WAYNE_FIPS = "26163"

#: wire key -> the value graph_bridge.py's fallback expressions produce for a
#: default CountyEconomicState (circulation defaults + absent financial
#: state). tick_housing_fictitious_fraction is the one honest-None fallback.
GROUP_C_D_FALLBACKS: dict[str, object] = {
    "tick_realization_crisis": False,
    "tick_turnover_crisis": False,
    "tick_reproduction_crisis": False,
    "tick_interest_burden": 0.0,
    "tick_ground_rent": 0.0,
    "tick_rentier_share": 0.0,
    "tick_profit_of_enterprise": 0.0,
    "tick_financialization_share": 0.0,
    "tick_accumulated_debt": 0.0,
    "tick_claims_exceed_surplus": False,
    "tick_housing_fictitious_fraction": None,
    "tick_financial_crisis_signals": 0,
}


def _boundary_county_and_params() -> tuple[CountyEconomicState, NationalTickParameters]:
    """A post-boundary county (default circulation state, absent financial state)."""
    dist = ClassDistribution(
        fips=WAYNE_FIPS,
        year=2011,
        bourgeoisie_share=0.02,
        petit_bourgeoisie_share=0.08,
        labor_aristocracy_share=0.30,
        proletariat_share=0.40,
        lumpenproletariat_share=0.20,
    )
    county = CountyEconomicState(
        fips=WAYNE_FIPS,
        year=2011,
        capital_stock=1e9,
        throughput_position=0.90,
        supply_chain_depth=2.1,
        unemployment_rate=0.081,
        u6_rate=0.10,
        pter_rate=0.04,
        nilf_rate=0.06,
        median_wage=21.0,
        employment=500_000.0,
        class_distribution=dist,
        phi_hour=3.50,
        real_wage_deflator=0.87,
        crisis_state=CrisisState(
            phase=CrisisPhase.DEEP,
            consecutive_below=6,
            crisis_start_period=3,
            crisis_duration=7,
            cumulative_wage_compression=0.22,
        ),
        bifurcation_risk=BifurcationRiskMetric(
            score=-0.65,
            solidarity_density=0.4,
            legitimation=0.3,
            class_burden_ratio=0.5,
        ),
    )
    params = NationalTickParameters(
        year=2011,
        tau=62.0,
        gamma_basket=0.68,
        gamma_basket_raw=0.68,
        gamma_III=0.33,
        gamma_III_raw=0.33,
        tau_effective=42.16,
        v_reproduction=12.0,
        estimated=True,
    )
    return county, params


def _wayne_graph() -> object:
    """A one-territory graph whose node carries the real Wayne FIPS."""
    territory = Territory(
        id="T001",
        name="Wayne County",
        sector_type=SectorType.INDUSTRIAL,
        profile=OperationalProfile.LOW_PROFILE,
        biocapacity=500.0,
        county_fips=WAYNE_FIPS,
    )
    state = WorldState(tick=0, entities={}, territories={"T001": territory}, relationships=[])
    return state.to_graph()


def test_boundary_carry_stamps_group_c_and_d() -> None:
    """A year-boundary carry re-applies Groups C/D — write-site expressions
    mirrored byte-for-byte, fallback constants included."""
    from game.engine_bridge import _carry_tick_dynamics_flows

    county, params = _boundary_county_and_params()
    new_graph = _wayne_graph()
    old_graph = _wayne_graph()
    ctx = {
        "_tick_dynamics": {
            "county_states": {WAYNE_FIPS: county},
            "national_params": params,
        }
    }

    _carry_tick_dynamics_flows(old_graph, new_graph, ctx)

    node = new_graph.nodes["T001"]
    circuit = county.circulation_state.circuit_state
    assert node["tick_liquidity_ratio"] == pytest.approx(circuit.liquidity_ratio)
    assert node["tick_commodity_overhang"] == pytest.approx(circuit.commodity_overhang)
    assert (
        node["tick_replacement_cycle"]
        == county.circulation_state.depreciation_fund.replacement_cycle_position.value
    )
    assert (
        node["tick_inventory_diagnosis"]
        == county.circulation_state.inventory_state.inventory_problem.value
    )
    for key, expected in GROUP_C_D_FALLBACKS.items():
        assert node[key] == expected, f"{key} must carry the write-site fallback constant"


def test_non_boundary_carry_forwards_group_c_and_d() -> None:
    """Between boundaries the last boundary's Group C/D values persist
    byte-identical (like tick_phi_hour), never evaporating to a flicker."""
    from game.engine_bridge import _carry_tick_dynamics_flows

    old_graph = _wayne_graph()
    old_graph.update_node(
        "T001",
        tick_phi_hour=3.50,
        tick_median_wage=21.0,
        tick_employment=500_000.0,
        tick_year=2011,
        tick_liquidity_ratio=0.42,
        tick_commodity_overhang=0.13,
        tick_replacement_cycle="mid_cycle",
        tick_inventory_diagnosis="balanced",
        tick_realization_crisis=True,
        tick_turnover_crisis=False,
        tick_reproduction_crisis=True,
        tick_interest_burden=120.5,
        tick_ground_rent=44.0,
        tick_rentier_share=0.31,
        tick_profit_of_enterprise=-12.0,
        tick_financialization_share=0.27,
        tick_accumulated_debt=9_000.0,
        tick_claims_exceed_surplus=True,
        tick_housing_fictitious_fraction=0.55,
        tick_financial_crisis_signals=3,
    )
    new_graph = _wayne_graph()
    ctx: dict[str, object] = {}  # no _tick_dynamics => not a boundary

    _carry_tick_dynamics_flows(old_graph, new_graph, ctx)

    node = new_graph.nodes["T001"]
    assert node["tick_liquidity_ratio"] == pytest.approx(0.42)
    assert node["tick_commodity_overhang"] == pytest.approx(0.13)
    assert node["tick_replacement_cycle"] == "mid_cycle"
    assert node["tick_inventory_diagnosis"] == "balanced"
    assert node["tick_realization_crisis"] is True
    assert node["tick_reproduction_crisis"] is True
    assert node["tick_interest_burden"] == pytest.approx(120.5)
    assert node["tick_ground_rent"] == pytest.approx(44.0)
    assert node["tick_rentier_share"] == pytest.approx(0.31)
    # Negative profit-of-enterprise is a debt-spiral signal — never clamp.
    assert node["tick_profit_of_enterprise"] == pytest.approx(-12.0)
    assert node["tick_financialization_share"] == pytest.approx(0.27)
    assert node["tick_accumulated_debt"] == pytest.approx(9_000.0)
    assert node["tick_claims_exceed_surplus"] is True
    assert node["tick_housing_fictitious_fraction"] == pytest.approx(0.55)
    assert node["tick_financial_crisis_signals"] == 3


def test_serialize_territory_emits_group_c_and_d_wire_keys() -> None:
    """The 16 keys ride every territory row; un-stamped attrs are honest None."""
    from game.engine_bridge import _serialize_territory

    territory = Territory(
        id="T001",
        name="Wayne County",
        sector_type=SectorType.INDUSTRIAL,
        profile=OperationalProfile.LOW_PROFILE,
        biocapacity=500.0,
        county_fips=WAYNE_FIPS,
    )
    state = WorldState(tick=0, entities={}, territories={"T001": territory}, relationships=[])
    graph = state.to_graph()
    graph.update_node("T001", tick_liquidity_ratio=0.42, tick_financial_crisis_signals=2)

    row = _serialize_territory(territory, graph=graph)

    assert row["tick_liquidity_ratio"] == pytest.approx(0.42)
    assert row["tick_financial_crisis_signals"] == 2
    for key in (
        "tick_commodity_overhang",
        "tick_replacement_cycle",
        "tick_inventory_diagnosis",
        "tick_realization_crisis",
        "tick_turnover_crisis",
        "tick_reproduction_crisis",
        "tick_interest_burden",
        "tick_ground_rent",
        "tick_rentier_share",
        "tick_profit_of_enterprise",
        "tick_financialization_share",
        "tick_accumulated_debt",
        "tick_claims_exceed_surplus",
        "tick_housing_fictitious_fraction",
    ):
        assert row[key] is None, f"{key} must be honest None when un-stamped, never a default"
```

And append the chip-contract class to `tests/unit/web/test_engine_bridge.py`:

```python
@pytest.mark.unit
class TestEconomyDashboardChipContract:
    """spec-116 4d.6: the payload key set is PINNED so a phantom chip
    (TS-declared, never-emitted) can never return. Corrected audit figures:
    all fields are emitted today; the tick-26 dead chips were pre-boundary
    honesty (profit_rate/occ) on an un-migrated DB, not phantoms."""

    def test_dashboard_emits_exactly_the_declared_key_set(self) -> None:
        mock_persistence = _make_mock_persistence()
        bridge = EngineBridge(mock_persistence)

        result = bridge.get_economy_dashboard(uuid.uuid4())

        assert set(result.keys()) == {
            "tick",
            "has_data",
            "value_produced",
            "rent_extracted",
            "exploitation_rate",
            "profit_rate",
            "occ",
            "imperial_rent_pool",
            "current_super_wage_rate",
            "wage_flow_total",
            "tribute_flow_total",
            "wealth_by_class_role",
            "county_flow",
        }, (
            "EconomyDashboardPayload drifted — update types/game.ts, the "
            "chips, and this pin in the SAME commit (no phantoms, no orphans)"
        )
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
mise run test:q -- tests/unit/web/test_carry_group_c_d.py \
  tests/unit/web/test_engine_bridge.py::TestEconomyDashboardChipContract
```

Expected: `KeyError: 'tick_liquidity_ratio'` (three times); the chip-contract test PASSES already
(it pins the status quo — that is the point; it guards Tasks 20/21's edits).

- [ ] **Step 3: Write the implementation.**

(3a) `web/game/engine_bridge.py` — boundary arm of `_carry_tick_dynamics_flows`: inside the
first `new_graph.update_node(...)` call, insert immediately before `**rate_updates,` (`:6284`):

```python
                # Playability Spine Task 20 (spec-116 4d.5): Group C
                # (circulation, Feature 023) + Group D (financial
                # distribution, Feature 024) join the carry — the write-site
                # expressions from graph_bridge.py:128-197 mirrored
                # byte-for-byte, fallback constants included. DECLARED-DARK:
                # both gating services are unwired, so these are the frozen
                # fallbacks until then (SEAM_REGISTRY rows stay
                # STRUCTURALLY_IMPOSSIBLE — never relabeled live).
                tick_liquidity_ratio=county.circulation_state.circuit_state.liquidity_ratio,
                tick_commodity_overhang=(
                    county.circulation_state.circuit_state.commodity_overhang
                ),
                tick_replacement_cycle=(
                    county.circulation_state.depreciation_fund.replacement_cycle_position.value
                ),
                tick_inventory_diagnosis=(
                    county.circulation_state.inventory_state.inventory_problem.value
                ),
                tick_realization_crisis=(
                    county.circulation_state.latest_assessment.realization_crisis
                    if county.circulation_state.latest_assessment is not None
                    else False
                ),
                tick_turnover_crisis=(
                    county.circulation_state.latest_assessment.turnover_crisis
                    if county.circulation_state.latest_assessment is not None
                    else False
                ),
                tick_reproduction_crisis=(
                    county.circulation_state.latest_assessment.reproduction_crisis
                    if county.circulation_state.latest_assessment is not None
                    else False
                ),
                tick_interest_burden=(
                    county.surplus_distribution.interest_payments
                    if county.surplus_distribution is not None
                    else 0.0
                ),
                tick_ground_rent=(
                    county.rent_extraction.total_rent
                    if county.rent_extraction is not None
                    else 0.0
                ),
                tick_rentier_share=(
                    county.surplus_distribution.rentier_share
                    if county.surplus_distribution is not None
                    else 0.0
                ),
                tick_profit_of_enterprise=(
                    county.surplus_distribution.profit_of_enterprise
                    if county.surplus_distribution is not None
                    else 0.0
                ),
                tick_financialization_share=(
                    county.surplus_distribution.financialization_share
                    if county.surplus_distribution is not None
                    else 0.0
                ),
                tick_accumulated_debt=(
                    county.debt_accumulation.accumulated_debt
                    if county.debt_accumulation is not None
                    else 0.0
                ),
                tick_claims_exceed_surplus=(
                    county.surplus_distribution.claims_exceed_surplus
                    if county.surplus_distribution is not None
                    else False
                ),
                tick_housing_fictitious_fraction=(
                    county.housing_decomposition.fictitious_fraction
                    if county.housing_decomposition is not None
                    else None
                ),
                tick_financial_crisis_signals=(
                    county.financial_crisis.active_signals
                    if county.financial_crisis is not None
                    else 0
                ),
```

(3b) Carry arm — inside the second `new_graph.update_node(...)` call, insert after
`tick_supply_chain_depth=old_data.get("tick_supply_chain_depth"),` (`:6338`):

```python
            # Task 20 (spec-116 4d.5): Groups C/D are annual too — carry the
            # last boundary's values forward byte-identical, same pattern as
            # Group A/B above (a serialized attr missing from EITHER arm is a
            # flickering lens: present on boundary ticks, None on the other 51).
            tick_liquidity_ratio=old_data.get("tick_liquidity_ratio"),
            tick_commodity_overhang=old_data.get("tick_commodity_overhang"),
            tick_replacement_cycle=old_data.get("tick_replacement_cycle"),
            tick_inventory_diagnosis=old_data.get("tick_inventory_diagnosis"),
            tick_realization_crisis=old_data.get("tick_realization_crisis"),
            tick_turnover_crisis=old_data.get("tick_turnover_crisis"),
            tick_reproduction_crisis=old_data.get("tick_reproduction_crisis"),
            tick_interest_burden=old_data.get("tick_interest_burden"),
            tick_ground_rent=old_data.get("tick_ground_rent"),
            tick_rentier_share=old_data.get("tick_rentier_share"),
            tick_profit_of_enterprise=old_data.get("tick_profit_of_enterprise"),
            tick_financialization_share=old_data.get("tick_financialization_share"),
            tick_accumulated_debt=old_data.get("tick_accumulated_debt"),
            tick_claims_exceed_surplus=old_data.get("tick_claims_exceed_surplus"),
            tick_housing_fictitious_fraction=old_data.get("tick_housing_fictitious_fraction"),
            tick_financial_crisis_signals=old_data.get("tick_financial_crisis_signals"),
```

(3c) `_serialize_territory` — append inside the returned dict, after
`"vision_state": ...` (`:8029`):

```python
        # Playability Spine Task 20 (spec-116 4d.5): Group C (circulation,
        # Feature 023) + Group D (financial distribution, Feature 024),
        # serialized DECLARED-DARK under their registry wire keys (tick_
        # prefix kept — the tick_median_wage collision precedent). Values are
        # the engine's fallback constants until turnover_profile_source /
        # interest_calculator are wired; None before the first boundary.
        "tick_liquidity_ratio": _territory_graph_attr(graph, territory_id, "tick_liquidity_ratio"),
        "tick_commodity_overhang": _territory_graph_attr(
            graph, territory_id, "tick_commodity_overhang"
        ),
        "tick_replacement_cycle": _territory_graph_attr(
            graph, territory_id, "tick_replacement_cycle"
        ),
        "tick_inventory_diagnosis": _territory_graph_attr(
            graph, territory_id, "tick_inventory_diagnosis"
        ),
        "tick_realization_crisis": _territory_graph_attr(
            graph, territory_id, "tick_realization_crisis"
        ),
        "tick_turnover_crisis": _territory_graph_attr(graph, territory_id, "tick_turnover_crisis"),
        "tick_reproduction_crisis": _territory_graph_attr(
            graph, territory_id, "tick_reproduction_crisis"
        ),
        "tick_interest_burden": _territory_graph_attr(graph, territory_id, "tick_interest_burden"),
        "tick_ground_rent": _territory_graph_attr(graph, territory_id, "tick_ground_rent"),
        "tick_rentier_share": _territory_graph_attr(graph, territory_id, "tick_rentier_share"),
        "tick_profit_of_enterprise": _territory_graph_attr(
            graph, territory_id, "tick_profit_of_enterprise"
        ),
        "tick_financialization_share": _territory_graph_attr(
            graph, territory_id, "tick_financialization_share"
        ),
        "tick_accumulated_debt": _territory_graph_attr(
            graph, territory_id, "tick_accumulated_debt"
        ),
        "tick_claims_exceed_surplus": _territory_graph_attr(
            graph, territory_id, "tick_claims_exceed_surplus"
        ),
        "tick_housing_fictitious_fraction": _territory_graph_attr(
            graph, territory_id, "tick_housing_fictitious_fraction"
        ),
        "tick_financial_crisis_signals": _territory_graph_attr(
            graph, territory_id, "tick_financial_crisis_signals"
        ),
```

and append to the `_serialize_territory` docstring (after the Program 23 paragraph, `:7969`):

```
    Playability Spine Task 20 (spec-116 4d.5): the Feature-023 circulation
    family and Feature-024 financial-distribution family join the same
    ``tick_``-prefixed graph-attr pattern, serialized DECLARED-DARK — the
    gating services (``turnover_profile_source``/``interest_calculator``)
    are unwired, so post-boundary values are the engine's fallback constants
    (0.0/False/0, plus ``tick_housing_fictitious_fraction``'s honest
    ``None``). The wire keys keep their ``tick_`` prefix (registry
    ``wire_keys``) — none collides with an existing payload key or Territory
    model field. SEAM_REGISTRY rows: Groups C/D, still
    ``STRUCTURALLY_IMPOSSIBLE`` (frozen constants are never relabeled live).
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
mise run test:q -- tests/unit/web/test_carry_group_c_d.py tests/unit/web/test_engine_bridge.py
```

Expected: all pass (`N passed`).

- [ ] **Step 5: Update the seam registry** (`src/babylon/sentinels/seam/registry.py`). Replace
  the comment + constant block at `:500-506` with:

```python
#: Groups C/D are SERIALIZED since spec-116 4d.5 (declared-dark): the write
#: site emits fallback constants until the gating services are wired, and
#: ``_serialize_territory`` faithfully carries those constants (or ``None``
#: before the first boundary). ``read_paths`` cites BOTH ends of the wire;
#: the liveness class stays STRUCTURALLY_IMPOSSIBLE — the registry's core
#: honesty rule: frozen constants are never relabeled as live data.
_TICK_WRITE_SITE: tuple[str, ...] = (
    "src/babylon/domain/economics/tick/graph_bridge.py::write_tick_state_to_graph "
    "(year-boundary graph.update_node call, :102-195)",
)

_TICK_DARK_EMITTERS: tuple[str, ...] = _TICK_WRITE_SITE + _TERRITORY_EMITTERS
```

Update the two gate strings (`:508-521`):

```python
_TURNOVER_GATE: str = (
    "STRUCTURALLY_IMPOSSIBLE: gated on the unwired `turnover_profile_source` service "
    "(domain/economics/tick/system/__init__.py:1050) — the circulation layer never "
    "computes this without a real turnover-profile source. Serialized declared-dark "
    "since spec-116 4d.5: the wire carries the write-site fallback constant (or None "
    "pre-boundary), never relabeled as live."
)

_INTEREST_GATE: str = (
    "STRUCTURALLY_IMPOSSIBLE: gated on the unwired `interest_calculator` service "
    "(domain/economics/tick/system/__init__.py:1248) — the financial distribution "
    "layer never computes this without a real interest calculator. Serialized "
    "declared-dark since spec-116 4d.5: the wire carries the write-site fallback "
    "constant (or None pre-boundary), never relabeled as live."
)
```

Then swap the read paths on all 16 Group C/D rows (`:748-935`) — a well-specified mechanical
edit, exactly 16 occurrences in the file:

```
replace_all:  "read_paths=_TICK_WRITE_SITE,"  ->  "read_paths=_TICK_DARK_EMITTERS,"
```

Verify the gate and sentinel suite:

```bash
mise run check:seams
mise run test:q -- tests/unit/sentinels/
```

Expected: exit 0 (payload names are unchanged, so the GATING `check_tick_payloads_exist` still
matches the graph_bridge write-set byte-for-byte; `check_bridge_serialization` is advisory and
now sees the 16 keys covered by rows).

- [ ] **Step 6: Declare the fields in TS** — `src/frontend/src/types/game.ts`, append inside
  `TerritoryState` (after `price_divergence?`, `:208`):

```ts
  /**
   * Playability Spine Task 20 (spec-116 4d.5): the Feature-023 circulation +
   * Feature-024 financial-distribution families, serialized DECLARED-DARK.
   * The engine's gating services (`turnover_profile_source` /
   * `interest_calculator`) are unwired, so post-boundary values are the
   * write-site fallback constants (0.0 / false / 0; the housing fraction's
   * honest `null`) and `null` before the first year boundary. Do NOT build
   * player-facing chrome on these until the SEAM_REGISTRY rows leave
   * STRUCTURALLY_IMPOSSIBLE — a chip of frozen constants is dishonest.
   * Wire keys keep the `tick_` prefix (registry `wire_keys`).
   */
  tick_liquidity_ratio?: number | null;
  tick_commodity_overhang?: number | null;
  tick_replacement_cycle?: string | null;
  tick_inventory_diagnosis?: string | null;
  tick_realization_crisis?: boolean | null;
  tick_turnover_crisis?: boolean | null;
  tick_reproduction_crisis?: boolean | null;
  tick_interest_burden?: number | null;
  tick_ground_rent?: number | null;
  tick_rentier_share?: number | null;
  tick_profit_of_enterprise?: number | null;
  tick_financialization_share?: number | null;
  tick_accumulated_debt?: number | null;
  tick_claims_exceed_surplus?: boolean | null;
  tick_housing_fictitious_fraction?: number | null;
  tick_financial_crisis_signals?: number | null;
```

- [ ] **Step 7: Frontend chip-contract tests** — append to
  `src/frontend/src/components/economy/EconomyDashboard.test.tsx` (inside the existing
  `describe("EconomyDashboard", ...)`; StatChip testids derive as
  `` `stat-${label.toLowerCase()}` ``):

```tsx
  it("renders every chip live from a fully-populated payload — no phantoms (spec-116 4d.6)", async () => {
    server.use(
      http.get("/api/games/:id/economy/", () =>
        HttpResponse.json({
          status: "ok",
          data: makeEconomyDashboardPayload({ profit_rate: 0.153, occ: 2.4 }),
        }),
      ),
    );
    render(<EconomyDashboard gameId={DEFAULT_GAME_ID} />);
    await waitFor(() => expect(screen.getByTestId("economy-stat-chips")).toBeInTheDocument());

    expect(screen.getByTestId("stat-profit rate")).toHaveTextContent("0.153");
    expect(screen.getByTestId("stat-occ")).toHaveTextContent("2.40");
    // With a full payload, no chip in the row may fall back to "no data" —
    // that would be a phantom (a TS-declared field the backend never sent).
    expect(screen.getByTestId("economy-stat-chips")).not.toHaveTextContent("no data");
  });

  it("keeps honest 'no data' on exactly the year-boundary chips pre-boundary (spec-116 4d.6)", async () => {
    // Default fixture: profit_rate/occ null (pre-tick-52 cadence honesty).
    render(<EconomyDashboard gameId={DEFAULT_GAME_ID} />);
    await waitFor(() => expect(screen.getByTestId("economy-stat-chips")).toBeInTheDocument());

    expect(screen.getByTestId("stat-profit rate")).toHaveTextContent("no data");
    expect(screen.getByTestId("stat-occ")).toHaveTextContent("no data");
    // Every other chip stays live — tick-26 "all dead" can never recur.
    expect(screen.getByTestId("stat-value produced")).not.toHaveTextContent("no data");
    expect(screen.getByTestId("stat-rent pool")).not.toHaveTextContent("no data");
    expect(screen.getByTestId("stat-wage flow")).not.toHaveTextContent("no data");
  });
```

- [ ] **Step 8: Run the full surface to verify green**

```bash
mise run test:q -- tests/unit/web/test_carry_group_c_d.py tests/unit/web/test_engine_bridge.py \
  tests/unit/sentinels/
mise run check:seams
cd src/frontend && npx tsc --noEmit && \
  npx vitest run src/components/economy/EconomyDashboard.test.tsx && cd ../..
```

Expected: all green; tsc clean.

- [ ] **Step 9: Commit**

```bash
git add web/game/engine_bridge.py src/babylon/sentinels/seam/registry.py \
  src/frontend/src/types/game.ts \
  src/frontend/src/components/economy/EconomyDashboard.test.tsx \
  tests/unit/web/test_carry_group_c_d.py tests/unit/web/test_engine_bridge.py
mise run commit -- "feat(web): serialize Group C/D tick_* attrs declared-dark + economy chip contract (spec-116 4d.5/4d.6)"
```

Byte-safety: bridge serializer + carry + registry + TS types only — engine write-site
(`graph_bridge.py`) untouched, so `check_tick_payloads_exist` and the regression baselines
cannot move.

---

### Task 21: PROFIT chip wired + CrisisTimeline/BifurcationGauge consume the history series

**The 4d.9 ruling (binding):** the TopBar PROFIT chip is **WIRED, not removed** — the stale
"engine computes no c/v/s decomposition" docstring predates `_mean_territory_attr`; the real
value is one call away (the `get_economy_dashboard:2713` pattern). It remains honest "no data"
until the first year boundary (tick 52 at weekly cadence) — permanent chrome no more, honest
absence still. `profit_rate` is an EXISTING wire key on `GameSummaryPayload` (currently emitted
as literal `None`), so no new SeamEntry row is required.

**Files:**
- Modify: `web/game/engine_bridge.py:2533-2535` (docstring), `:2581` (the hardcoded `None`)
- Create: `tests/unit/web/test_summary_profit_wire.py`,
  `src/frontend/src/components/chrome/sparkline.ts`,
  `src/frontend/src/components/chrome/sparkline.test.ts`
- Modify: `src/frontend/src/components/chrome/TopBar.test.tsx` (real-profit case),
  `src/frontend/src/components/chrome/CrisisTimeline.tsx:221-274` (history sparkline),
  `src/frontend/src/components/chrome/CrisisTimeline.test.tsx` (render cases),
  `src/frontend/src/components/chrome/BifurcationGauge.tsx:207-251` (history sparkline),
  `src/frontend/src/components/chrome/BifurcationGauge.test.tsx` (render case)

**Interfaces:**
- Consumes (from Task 19): `TimeseriesPayload.crisis_pop_share` and
  `TimeseriesPayload.bifurcation_score_mean` (optional `(number | null)[]`), delivered through
  `s.panels.timeseries.data` — populated because `BottomDrawer` keeps `TimeseriesChart`
  always-mounted (CSS-hidden) and `worldSlice.onTickAdvanced` fans out re-fetches per tick. The
  widgets read this slice PASSIVELY and must NOT call `panels.timeseries.setMounted` (it is a
  boolean, not a ref-count — a second toggler would fight the chart's mount effect).
- Produces:
  - `get_game_summary` `profit_rate` = `_mean_territory_attr(graph, "tick_profit_rate")`.
  - `sparklineSeries(values: readonly (number | null | undefined)[]): SparklinePoint[]` and
    `sparklinePoints(values, width: number, height: number): string | null` from
    `src/frontend/src/components/chrome/sparkline.ts`.
  - Testids `crisis-history-sparkline`, `bifurcation-history-sparkline`.

- [ ] **Step 1: Write the failing backend test** — create
  `tests/unit/web/test_summary_profit_wire.py`:

```python
"""spec-116 4d.9: the TopBar PROFIT chip is WIRED (not removed).

``get_game_summary`` hardcoded ``profit_rate: None`` behind a stale docstring
("the engine computes no c/v/s decomposition on the live graph") written
before ``_mean_territory_attr`` existed — while ``get_economy_dashboard``
served the real mean one method below. Honest ``None`` persists until the
first year boundary (tick 52 at weekly cadence): the chip's pre-boundary
"no data" is CORRECT, never fabricated (Constitution III.11).
"""

from __future__ import annotations

import uuid
from unittest.mock import MagicMock, patch

import pytest

from babylon.topology.graph import BabylonGraph
from game.engine_bridge import EngineBridge


def _bridge_and_state() -> tuple[EngineBridge, MagicMock]:
    """A bridge with list-returning event queries and an empty-world state."""
    persistence = MagicMock()
    persistence.query_tick_events.return_value = []
    bridge = EngineBridge(persistence)
    state = MagicMock()
    state.tick = 60
    state.entities = {}
    state.territories = {}
    state.organizations = {}
    state.economy = None
    return bridge, state


@pytest.mark.unit
class TestSummaryProfitRate:
    def test_profit_rate_is_the_territory_mean_after_a_boundary(self) -> None:
        graph = BabylonGraph()
        graph.add_node("T1", node_type="territory", tick_profit_rate=0.10)
        graph.add_node("T2", node_type="territory", tick_profit_rate=0.20)
        bridge, state = _bridge_and_state()

        with patch.object(bridge, "hydrate_state", return_value=(state, graph)):
            out = bridge.get_game_summary(uuid.uuid4())

        assert out["profit_rate"] == pytest.approx(0.15)

    def test_profit_rate_stays_none_before_first_boundary(self) -> None:
        graph = BabylonGraph()
        graph.add_node("T1", node_type="territory")
        bridge, state = _bridge_and_state()

        with patch.object(bridge, "hydrate_state", return_value=(state, graph)):
            out = bridge.get_game_summary(uuid.uuid4())

        assert out["profit_rate"] is None
```

- [ ] **Step 2: Run test to verify it fails**

```bash
mise run test:q -- tests/unit/web/test_summary_profit_wire.py
```

Expected: `assert None == 0.15 ± 1.5e-07` (the boundary test fails; the pre-boundary test
already passes — it pins the honest half of the contract).

- [ ] **Step 3: Write the backend fix** — `web/game/engine_bridge.py:2581`:

```python
            "profit_rate": _mean_territory_attr(graph, "tick_profit_rate"),
```

and replace the stale docstring sentence (`:2533-2535`):

```
        ``profit_rate`` is the mean of every territory's year-boundary
        ``tick_profit_rate`` (:func:`_mean_territory_attr` — the exact
        :meth:`get_economy_dashboard` pattern, spec-116 4d.9): honest
        ``None`` until the first year boundary this session stamps county
        state, never a fabricated 0.0 (Constitution III.11).
```

- [ ] **Step 4: Verify backend green**

```bash
mise run test:q -- tests/unit/web/test_summary_profit_wire.py tests/unit/web/test_engine_bridge.py
```

Expected: all pass.

- [ ] **Step 5: TopBar real-profit test** — append to
  `src/frontend/src/components/chrome/TopBar.test.tsx` (inside `describe("TopBar", ...)`; the
  existing "no data" case at `:34` keeps guarding pre-boundary honesty):

```tsx
  it("renders a real profit rate once the year boundary lands (spec-116 4d.9)", async () => {
    server.use(
      http.get("/api/games/:id/summary/", () =>
        HttpResponse.json({
          status: "ok",
          data: makeGameSummaryPayload({ profit_rate: 0.153 }),
        }),
      ),
    );
    render(<TopBar gameId={DEFAULT_GAME_ID} />);
    await waitFor(() => expect(screen.getByTestId("stat-profit")).toHaveTextContent("0.153"));
  });
```

Run: `cd src/frontend && npx vitest run src/components/chrome/TopBar.test.tsx && cd ../..` —
expected green (the fixture's default `profit_rate: 0.18` and this override both exercise the
real-value path; the MSW default handler keeps `null`).

- [ ] **Step 6: Write the failing sparkline tests** — create
  `src/frontend/src/components/chrome/sparkline.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { sparklinePoints, sparklineSeries } from "./sparkline";

describe("sparklineSeries", () => {
  it("skips null/undefined entries without interpolating", () => {
    expect(sparklineSeries([null, 1, undefined, 3])).toEqual([
      { x: 1, y: 1 },
      { x: 3, y: 3 },
    ]);
  });
});

describe("sparklinePoints", () => {
  it("returns null below two real points — a lone value is not a trend", () => {
    expect(sparklinePoints([], 60, 16)).toBeNull();
    expect(sparklinePoints([null, null], 60, 16)).toBeNull();
    expect(sparklinePoints([null, 0.5], 60, 16)).toBeNull();
  });

  it("scales points into the box, preserving tick order and gaps", () => {
    expect(sparklinePoints([0, null, 1], 60, 16)).toBe("1.0,15.0 59.0,1.0");
  });

  it("handles a flat series without dividing by zero", () => {
    expect(sparklinePoints([0.5, 0.5], 60, 16)).toBe("1.0,15.0 59.0,15.0");
  });
});
```

And the widget render cases. Append to
`src/frontend/src/components/chrome/CrisisTimeline.test.tsx` (this file is pure-function-only
today — add the render imports at the top alongside the existing ones):

```tsx
import { beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { useStore } from "@/store";
import { resetStore } from "@/test/resetStore";
import { makeTimeseriesPayload } from "@/test/fixtures";
import { CrisisTimeline } from "./CrisisTimeline";
```

```tsx
describe("CrisisTimeline history sparkline (spec-116 4d.5)", () => {
  beforeEach(() => {
    resetStore();
  });

  it("renders the history sparkline when the series has two real points", () => {
    useStore.setState((s) => ({
      panels: {
        ...s.panels,
        timeseries: {
          ...s.panels.timeseries,
          data: makeTimeseriesPayload({
            ticks: [0, 52, 104],
            crisis_pop_share: [null, 0.4, 0.75],
          }),
        },
      },
    }));
    render(<CrisisTimeline gameId="g1" />);
    expect(screen.getByTestId("crisis-history-sparkline")).toBeInTheDocument();
  });

  it("renders no sparkline before the first year boundary (honest sparse)", () => {
    useStore.setState((s) => ({
      panels: {
        ...s.panels,
        timeseries: {
          ...s.panels.timeseries,
          data: makeTimeseriesPayload({ ticks: [0, 1], crisis_pop_share: [null, null] }),
        },
      },
    }));
    render(<CrisisTimeline gameId="g1" />);
    expect(screen.queryByTestId("crisis-history-sparkline")).not.toBeInTheDocument();
  });
});
```

Append to `src/frontend/src/components/chrome/BifurcationGauge.test.tsx` (same import
additions if not already present in that file):

```tsx
describe("BifurcationGauge history sparkline (spec-116 4d.5)", () => {
  beforeEach(() => {
    resetStore();
  });

  it("renders the trajectory sparkline when the series has two real points", () => {
    useStore.setState((s) => ({
      panels: {
        ...s.panels,
        timeseries: {
          ...s.panels.timeseries,
          data: makeTimeseriesPayload({
            ticks: [0, 52, 104],
            bifurcation_score_mean: [null, -0.2, -0.5],
          }),
        },
      },
    }));
    render(<BifurcationGauge gameId="g1" />);
    expect(screen.getByTestId("bifurcation-history-sparkline")).toBeInTheDocument();
  });
});
```

- [ ] **Step 7: Run to verify red**

```bash
cd src/frontend && npx vitest run src/components/chrome/sparkline.test.ts \
  src/components/chrome/CrisisTimeline.test.tsx \
  src/components/chrome/BifurcationGauge.test.tsx && cd ../..
```

Expected: `Failed to resolve import "./sparkline"` and
`Unable to find an element by: [data-testid="crisis-history-sparkline"]`.

- [ ] **Step 8: Write the frontend implementation.**

(8a) Create `src/frontend/src/components/chrome/sparkline.ts`:

```ts
/**
 * Shared sparkline geometry for the chrome widgets' history strips
 * (Playability Spine Task 21, spec-116 4d.5).
 *
 * Maps a parallel-indexed nullable series (the `TimeseriesPayload` arrays)
 * onto an SVG polyline `points` string. Null/undefined entries are SKIPPED,
 * never interpolated to a fabricated baseline (Constitution III.11) — the
 * series is a year-boundary step function with a null head, so the line
 * simply starts at the first real point.
 */

export interface SparklinePoint {
  x: number;
  y: number;
}

/** Extract the plottable (tick-index, value) pairs from a nullable series. */
export function sparklineSeries(values: readonly (number | null | undefined)[]): SparklinePoint[] {
  const points: SparklinePoint[] = [];
  values.forEach((value, index) => {
    if (typeof value === "number" && Number.isFinite(value)) {
      points.push({ x: index, y: value });
    }
  });
  return points;
}

/**
 * SVG polyline `points` for a nullable series scaled into a width x height
 * box (1px padding). Returns `null` when fewer than two real points exist —
 * a single point is not a trend and renders nothing rather than a dot of
 * fabricated significance.
 */
export function sparklinePoints(
  values: readonly (number | null | undefined)[],
  width: number,
  height: number,
): string | null {
  const series = sparklineSeries(values);
  if (series.length < 2) return null;
  const xMin = series[0].x;
  const xMax = series[series.length - 1].x;
  const ys = series.map((p) => p.y);
  const yMin = Math.min(...ys);
  const yMax = Math.max(...ys);
  const xSpan = xMax - xMin || 1;
  const ySpan = yMax - yMin || 1;
  const pad = 1;
  return series
    .map((p) => {
      const x = pad + ((p.x - xMin) / xSpan) * (width - 2 * pad);
      const y = height - pad - ((p.y - yMin) / ySpan) * (height - 2 * pad);
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
}
```

(8b) `src/frontend/src/components/chrome/CrisisTimeline.tsx` — add the import:

```ts
import { sparklinePoints } from "./sparkline";
```

in the component body (after `const capital = aggregateCapitalStock(territories);`, `:233`):

```tsx
  // History (spec-116 4d.5): the county-deduped crisis_pop_share series from
  // panels.timeseries — populated by BottomDrawer's always-mounted
  // TimeseriesChart + the per-tick fan-out. Read PASSIVELY: never touch
  // panels.timeseries.setMounted from here (boolean, not a ref-count).
  const timeseries = useStore((s) => s.panels.timeseries.data);
  const crisisHistory = sparklinePoints(timeseries?.crisis_pop_share ?? [], 168, 16);
```

and render, after the `crisis-capital-line` paragraph (`:264-268`, inside the `hasAnySignal`
fragment):

```tsx
            {crisisHistory !== null && (
              <svg
                width={168}
                height={16}
                className="block"
                data-testid="crisis-history-sparkline"
                role="img"
                aria-label="Population-in-crisis history (yearly points, carried between boundaries)"
              >
                <polyline
                  points={crisisHistory}
                  fill="none"
                  stroke="var(--babylon-rupture)"
                  strokeWidth="1"
                />
              </svg>
            )}
```

Note: the sparkline lives inside the `hasAnySignal` branch by design — pre-boundary there is
neither a current-tick signal nor a series, and the existing "No crisis data yet" line already
tells that story honestly.

(8c) `src/frontend/src/components/chrome/BifurcationGauge.tsx` — add the import:

```ts
import { sparklinePoints } from "./sparkline";
```

in the component body (after `const fascistScore = ...`, `:219-220`):

```tsx
  // History (spec-116 4d.5): county-deduped bifurcation trajectory. Passive
  // read of panels.timeseries — same contract as CrisisTimeline's strip.
  const timeseries = useStore((s) => s.panels.timeseries.data);
  const bifurcationHistory = sparklinePoints(timeseries?.bifurcation_score_mean ?? [], 176, 16);
```

and render, after the `BifurcationAxis` conditional (`:231-233`):

```tsx
        {bifurcationHistory !== null && (
          <svg
            width={AXIS_W}
            height={16}
            className="block"
            data-testid="bifurcation-history-sparkline"
            role="img"
            aria-label="Bifurcation trajectory history: revolution down, fascism up (yearly points)"
          >
            <polyline
              points={bifurcationHistory}
              fill="none"
              stroke="var(--babylon-solidarity)"
              strokeWidth="1"
            />
          </svg>
        )}
```

- [ ] **Step 9: Verify green + cluster-wide gates**

```bash
cd src/frontend && npx tsc --noEmit && npx vitest run src/components/chrome && cd ../..
mise run test:q -- tests/unit/web/
mise run check
mise run qa:regression
```

Expected: vitest + tsc green; `mise run check` green (includes `check:seams`);
`qa:regression` **byte-identical, 5/5 passed** — this cluster never touched engine/defines, so
any drift is a scope violation: STOP and investigate, do not regenerate baselines.

- [ ] **Step 10: Commit**

```bash
git add web/game/engine_bridge.py tests/unit/web/test_summary_profit_wire.py \
  src/frontend/src/components/chrome/sparkline.ts \
  src/frontend/src/components/chrome/sparkline.test.ts \
  src/frontend/src/components/chrome/CrisisTimeline.tsx \
  src/frontend/src/components/chrome/CrisisTimeline.test.tsx \
  src/frontend/src/components/chrome/BifurcationGauge.tsx \
  src/frontend/src/components/chrome/BifurcationGauge.test.tsx \
  src/frontend/src/components/chrome/TopBar.test.tsx
mise run commit -- "feat(web): wire TopBar profit chip + crisis/bifurcation history sparklines (spec-116 4d.9/4d.5)"
```

**Live verification (after all three tasks):** `mise run web:dev`, create a fresh wayne_county
session, then (1) Trends tab shows real lines from tick 1 (imperial_rent/consciousness are live
from tick 1 — the tick-26 "No timeseries data yet" symptom is gone); (2) step past tick 52 (or
drive a seeded long session) and confirm the TopBar PROFIT chip, the Economy Profit/OCC chips,
and both history sparklines light up together at the first year boundary; (3) confirm the two
pre-existing 2026-07-17 sessions still open (their historical ticks stay empty — rows are only
derivable going forward; new-session correctness is the acceptance bar).
# Tasks 22–23 — Event whitelist widening (FR-116-4.7)

**Scope ruling encoded here (from the recon brief's dropped-36 classification):**

- **Task 22** makes the three reactionary verbs (POGROM / LOCKOUT / VIGILANTISM)
  first-class wire events. *Framing correction vs the work-item text:* the "three
  hardcoded ORGANIZATIONAL_ACTION sites" in `action_effects.py` (`resolve_action`:179,
  `_resolve_agitate`:261, `_resolve_assimilate`:326) belong to genuinely-organizational
  verbs and are **kept** — `_resolve_fascist_verb` (action_effects.py:245) already emits
  the correct first-class values into `ActionResult.events_generated`. The real defect is
  that `events_generated` never reaches the bus (nothing publishes it; the only OODA
  publish is the per-tick ORGANIZATIONAL_ACTION summary at ooda.py:186-199). Task 22
  therefore creates the missing bus-publish seam in `OODASystem.step`, keeping the
  summary publish unchanged for its existing consumers.
- **Task 23** widens the converter for the **14 remaining EventTypes with live bus
  publishers**: MARKET_CORRECTION, ENTITY_DEATH, POPULATION_ATTRITION,
  CRISIS_PHASE_TRANSITION, BIFURCATION_THRESHOLD, EDGE_MODE_TRANSITION,
  CO_OPTIVE_BREAKDOWN, LATENT_CONTRADICTION_RELEASE, ASPECT_REVERSAL, LEVEL_TRANSITION,
  SECESSION_DECLARED, CALIBRATION_AXIOM_VIOLATION, CALIBRATION_QCEW_CARRY_FORWARD,
  CALIBRATION_PHI_HOUR_OUTLIER.
- **Explicitly excluded — stay dropped (19):** the 18 dead enum values with no live
  publisher (SOLIDARITY_AWAKENING, POPULATION_DEATH, EXPLOITATION_MODE_SHIFT,
  DUAL_CIRCUIT_INTERFERENCE, CONSCIOUSNESS_SHIFT, INITIATIVE_CONTESTED,
  INFRASTRUCTURE_CHANGE, CALIBRATION_DISAGREEMENT, STATE_ACTION_EXECUTED,
  FASCIST_CONVERGENCE, FACTION_SHIFT, THREAD_ESCALATION, LEGAL_FRAMEWORK_ENACTED,
  LEGAL_FRAMEWORK_REVOKED, INSTITUTION_REPRODUCTION, BIFURCATION_TENDENCY_CHANGE,
  RED_OGV_ENDGAME, FRAGMENTED_COLLAPSE_ENDGAME) **plus ENDGAME_REACHED**, which is not
  dead but must NOT get a converter branch — EndgameDetector already delivers an
  already-typed EndgameEvent via `persistent_context["_observer_events"]`
  (simulation_engine.py:1149-1160); a branch would double-deliver it.
  Ledger: 47 handled today → 50 after Task 22 → 64 after Task 23; 19 stay dropped.
- **Durability decision:** all new payload classes use the light Program-17 pattern (no
  `kind`, not in the `TickEvent` union) — matching the `reactionary_payloads`/
  `struggle_payloads` family they sit beside. Wire delivery is same-tick from live
  `WorldState.events`, so toasts/journal are unaffected; on a graph round-trip they
  replay as bare `SimulationEvent` with a loud WARNING (`world_state._validate_event`),
  identical to FASCIST_DRIFT et al. today. The doctrine kind+union pattern (touches
  TickEvent, EVENT_CLASS_MAP, graph-roundtrip + postgres tests) is deliberately NOT
  taken — no consumer replays these payloads typed.
- **Backend severity rulings** (three-bucket `_EVENT_SEVERITY` vocabulary; the FR-116-2
  retier reserves crimson for genuine rupture/endgame proximity, so the reactionary
  verbs land in the repression-family warning tier, not critical):
  pogrom/lockout/vigilantism = `warning`; secession_declared = `critical`
  (fragmented-collapse proximity, peer of power_vacuum); market_correction /
  entity_death / crisis_phase_transition / bifurcation_threshold / co_optive_breakdown /
  level_transition = `warning`; population_attrition / edge_mode_transition /
  latent_contradiction_release / aspect_reversal / calibration trio = `informational`.
  These are drafter rulings on the brief's open question — flagged for owner taste.
- **No new wire keys**: the serialized event dict keeps its exact shape
  (`id/type/tick/severity/title/body/data`); the pogrom-family `data["territory_id"]`
  reuses the key the uprising anchor already serializes from the same function. No
  SeamEntry rows required; `check_severity_vocabulary` (GATING) passes because every new
  severity key is a real `EventType` value (including the dotted calibration values).

---

### Task 22: Reactionary actions first-class (POGROM / LOCKOUT / VIGILANTISM)

**Files:**

- Modify: `src/babylon/models/events/reactionary_payloads.py:75-83` (append 3 payload
  classes; extend `__all__`)
- Modify: `src/babylon/engine/simulation_engine.py:129-134` (import block),
  `:513-517` (widening ledger in the converter docstring), `:775` (insert 3 branches
  immediately after the RED_BROWN_COUP branch, before the `# Feature-030 lifecycle`
  comment at :777)
- Modify: `src/babylon/engine/systems/ooda.py:29-36` (module constant after imports),
  `:185-199` (per-action publish inside the existing `if services.event_bus:` block,
  before the unchanged summary publish)
- Modify: `web/game/engine_bridge.py:6829-6830` (warning-tier entries in
  `_EVENT_SEVERITY`), `:6838` (anchor-set constant right after the map),
  `:6905-6910` (`_serialize_event` territory-anchor `elif`)
- Test: `tests/unit/engine/test_event_conversion.py:68-73` (import block) + append 3
  test classes at EOF; `tests/unit/ooda/test_ooda_system.py` (imports + 1 test class at
  EOF); `tests/unit/web/test_engine_bridge.py` (append 1 test class after
  `TestSerializeEventUprisingTerritoryAnchoring`, ~line 2790, reusing its
  `_graph_with_tenancy` helper); `src/frontend/src/lib/__tests__/eventClassifier.test.ts`
  (verification pins)

**Interfaces:**

- Consumes (existing code only, no dependency on Tasks 1–21):
  `_FASCIST_VERBS: dict[ActionType, EventType]` (action_effects.py:32),
  `ActionResult.events_generated: list[str]` / `.direct_effects: dict[str, Any]`
  (ooda/types.py), `OODASystem._resolve_for_organization` (the documented spy seam),
  `_class_to_territory(_tenancy_members_by_territory(graph))` (engine_bridge uprising
  anchor), `_graph_with_tenancy` test helper (test_engine_bridge.py:~2673).
- Produces:
  `PogromEvent(org_id, target_id, repression_increment, wealth_destroyed)`,
  `LockoutEvent(org_id, target_id, wage_attenuation)`,
  `VigilantismEvent(org_id, target_id, repression_increment)` in
  `babylon.models.events.reactionary_payloads`;
  `_FIRST_CLASS_ACTION_EVENTS: frozenset[str]` (ooda.py);
  `_TERRITORY_ANCHORED_VERB_EVENTS: frozenset[str]` (engine_bridge.py);
  `_EVENT_SEVERITY` entries `"pogrom"/"lockout"/"vigilantism" -> "warning"`.
  Task 23 consumes the converter-ledger state (50 of 83).

- [ ] **Step 1: Write the failing conversion tests** — extend the
      `reactionary_payloads` import block in
      `tests/unit/engine/test_event_conversion.py:68-73` and append at EOF:

```python
# import block at :68-73 becomes:
from babylon.models.events.reactionary_payloads import (
    FascistDriftEvent,
    FascistRecruitmentEvent,
    LockoutEvent,
    OrganizationalFractureEvent,
    PogromEvent,
    RedBrownCoupEvent,
    VigilantismEvent,
)
```

```python
class TestPogromEventConversion:
    """spec-116 FR-116-4.7: POGROM converts first-class (mandatory RED-phase test)."""

    def test_converts_pogrom_event(self) -> None:
        """POGROM events convert to PogromEvent with the resolver's effects."""
        bus_event = Event(
            type=EventType.POGROM,
            tick=11,
            payload={
                "org_id": "ORG_FASH",
                "target_id": PERIPHERY_WORKER_ID,
                "repression_increment": 0.15,
                "wealth_destroyed": 12.5,
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, PogromEvent)
        assert result.event_type == EventType.POGROM
        assert result.tick == 11
        assert result.org_id == "ORG_FASH"
        assert result.target_id == PERIPHERY_WORKER_ID
        assert result.repression_increment == 0.15
        assert result.wealth_destroyed == 12.5

    def test_pogrom_with_string_event_type_and_absent_effects(self) -> None:
        """The events_generated string form converts; missing effects default 0.0
        (target node absent in _resolve_fascist_verb -> empty effects dict)."""
        bus_event = Event(
            type="pogrom",  # type: ignore[arg-type]
            tick=2,
            payload={"org_id": "ORG_FASH", "target_id": PERIPHERY_WORKER_ID},
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert isinstance(result, PogromEvent)
        assert result.repression_increment == 0.0
        assert result.wealth_destroyed == 0.0


class TestLockoutEventConversion:
    """spec-116 FR-116-4.7: LOCKOUT converts first-class."""

    def test_converts_lockout_event(self) -> None:
        """LOCKOUT events convert to LockoutEvent."""
        bus_event = Event(
            type=EventType.LOCKOUT,
            tick=12,
            payload={
                "org_id": "ORG_EMPLOYER",
                "target_id": LABOR_ARISTOCRACY_ID,
                "wage_attenuation": 0.3,
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, LockoutEvent)
        assert result.event_type == EventType.LOCKOUT
        assert result.tick == 12
        assert result.org_id == "ORG_EMPLOYER"
        assert result.target_id == LABOR_ARISTOCRACY_ID
        assert result.wage_attenuation == 0.3


class TestVigilantismEventConversion:
    """spec-116 FR-116-4.7: VIGILANTISM converts first-class."""

    def test_converts_vigilantism_event(self) -> None:
        """VIGILANTISM events convert to VigilantismEvent."""
        bus_event = Event(
            type=EventType.VIGILANTISM,
            tick=13,
            payload={
                "org_id": "ORG_FASH",
                "target_id": PERIPHERY_WORKER_ID,
                "repression_increment": 0.1,
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, VigilantismEvent)
        assert result.event_type == EventType.VIGILANTISM
        assert result.tick == 13
        assert result.org_id == "ORG_FASH"
        assert result.target_id == PERIPHERY_WORKER_ID
        assert result.repression_increment == 0.1
```

- [ ] **Step 2: Run to verify failure** —
      `mise run test:q -- tests/unit/engine/test_event_conversion.py`
      Expected: collection error
      `ImportError: cannot import name 'PogromEvent' from 'babylon.models.events.reactionary_payloads'`.

- [ ] **Step 3: Minimal implementation — payload classes + converter branches.**
      Append to `src/babylon/models/events/reactionary_payloads.py` (before `__all__`)
      and extend `__all__`:

```python
class PogromEvent(SimulationEvent):
    """POGROM event payload (spec-116 FR-116-4.7; publish site ooda.py step).

    First-class reactionary verb event: targeted communal violence. Payload
    mirrors ``{org_id, target_id, **direct_effects}`` built from the
    ``_resolve_fascist_verb`` ActionResult (action_effects.py:211-226) —
    effect fields default 0.0 when the target node was absent at resolution.
    """

    event_type: EventType = Field(default=EventType.POGROM)
    org_id: str
    target_id: str
    repression_increment: float = 0.0
    wealth_destroyed: float = 0.0


class LockoutEvent(SimulationEvent):
    """LOCKOUT event payload (spec-116 FR-116-4.7; publish site ooda.py step).

    First-class reactionary verb event: the employer withdraws wages —
    incoming WAGES value_flow attenuated (action_effects.py:228-239).
    """

    event_type: EventType = Field(default=EventType.LOCKOUT)
    org_id: str
    target_id: str
    wage_attenuation: float = 0.0


class VigilantismEvent(SimulationEvent):
    """VIGILANTISM event payload (spec-116 FR-116-4.7; publish site ooda.py step).

    First-class reactionary verb event: extra-state local repression —
    target's ``repression_faced`` raised (action_effects.py:211-221).
    """

    event_type: EventType = Field(default=EventType.VIGILANTISM)
    org_id: str
    target_id: str
    repression_increment: float = 0.0


__all__ = [
    "FascistDriftEvent",
    "FascistRecruitmentEvent",
    "LockoutEvent",
    "OrganizationalFractureEvent",
    "PogromEvent",
    "RedBrownCoupEvent",
    "VigilantismEvent",
]
```

In `src/babylon/engine/simulation_engine.py` extend the import block at :129-134:

```python
from babylon.models.events.reactionary_payloads import (
    FascistDriftEvent,
    FascistRecruitmentEvent,
    LockoutEvent,
    OrganizationalFractureEvent,
    PogromEvent,
    RedBrownCoupEvent,
    VigilantismEvent,
)
```

Insert after the RED_BROWN_COUP branch (ends :775, before the `# Feature-030
lifecycle` comment):

```python
    # Spec-116 FR-116-4.7: first-class reactionary verb events (OODASystem
    # per-action publish; payload = org/target + the resolver's direct_effects).
    if event_type == EventType.POGROM:
        return PogromEvent(
            tick=tick,
            timestamp=timestamp,
            org_id=payload.get("org_id", ""),
            target_id=payload.get("target_id", ""),
            repression_increment=payload.get("repression_increment", 0.0),
            wealth_destroyed=payload.get("wealth_destroyed", 0.0),
        )

    if event_type == EventType.LOCKOUT:
        return LockoutEvent(
            tick=tick,
            timestamp=timestamp,
            org_id=payload.get("org_id", ""),
            target_id=payload.get("target_id", ""),
            wage_attenuation=payload.get("wage_attenuation", 0.0),
        )

    if event_type == EventType.VIGILANTISM:
        return VigilantismEvent(
            tick=tick,
            timestamp=timestamp,
            org_id=payload.get("org_id", ""),
            target_id=payload.get("target_id", ""),
            repression_increment=payload.get("repression_increment", 0.0),
        )
```

Append to the converter docstring ledger (after the Unit 6a line at :516-517 — this
also corrects the stale "of 82"):

```python
    Spec-116 FR-116-4.7 (Playability Spine): widened to 50 of 83 EventTypes
    (POGROM / LOCKOUT / VIGILANTISM first-class).
```

Keep the flat if-chain and the `# noqa: C901` — the seam sentinel's
`eventtype_names_in_func` parses this exact function by name; no dispatch-dict
refactor.

- [ ] **Step 4: Run to verify pass** —
      `mise run test:q -- tests/unit/engine/test_event_conversion.py`
      Expected: all pass (existing classes + 3 new, `N passed`).

- [ ] **Step 5: Write the failing OODA publish test** — in
      `tests/unit/ooda/test_ooda_system.py`, change the import header to:

```python
from unittest.mock import MagicMock, patch

from babylon.config.defines import GameDefines
from babylon.engine.systems.ooda import OODASystem
from babylon.models.enums import ActionType, EventType, OrgType
from babylon.ooda.types import Action, ActionResult
from babylon.topology.graph import BabylonGraph
```

and append at EOF:

```python
class TestFirstClassReactionaryVerbPublish:
    """spec-116 FR-116-4.7: POGROM/LOCKOUT/VIGILANTISM ActionResults publish
    their own first-class bus events (payload = org/target + direct_effects);
    the per-tick ORGANIZATIONAL_ACTION summary is unchanged. Uses the
    documented ``_resolve_for_organization`` spy seam — the verbs have no
    NPC/player route yet, so resolution is patched in."""

    @staticmethod
    def _graph_single_faction() -> BabylonGraph:
        graph = BabylonGraph()
        graph.add_node(
            "fash_org",
            _node_type="organization",
            org_type=OrgType.POLITICAL_FACTION.value,
            territory_ids=["detroit"],
            ooda_profile={"action_points": 3, "decision_mode": "autocratic"},
        )
        graph.add_node("detroit", _node_type="territory")
        return graph

    @staticmethod
    def _verb_result(
        action_type: ActionType, event_value: str, effects: dict
    ) -> ActionResult:
        return ActionResult(
            action=Action(org_id="fash_org", action_type=action_type, target_id="C900"),
            success=True,
            direct_effects=effects,
            events_generated=[event_value],
        )

    @staticmethod
    def _published(services: MagicMock) -> list:
        return [c.args[0] for c in services.event_bus.publish.call_args_list]

    def test_pogrom_result_publishes_first_class_event(self) -> None:
        system = OODASystem()
        services = _make_services()
        result = self._verb_result(
            ActionType.POGROM,
            EventType.POGROM.value,
            {"repression_increment": 0.15, "wealth_destroyed": 12.5},
        )
        with patch.object(
            OODASystem, "_resolve_for_organization", return_value=[result]
        ):
            system.step(self._graph_single_faction(), services, {"tick": 3})

        pogroms = [e for e in self._published(services) if e.type == EventType.POGROM]
        assert len(pogroms) == 1
        assert pogroms[0].tick == 3
        assert pogroms[0].payload == {
            "org_id": "fash_org",
            "target_id": "C900",
            "repression_increment": 0.15,
            "wealth_destroyed": 12.5,
        }

    def test_lockout_and_vigilantism_publish(self) -> None:
        system = OODASystem()
        services = _make_services()
        results = [
            self._verb_result(
                ActionType.LOCKOUT, EventType.LOCKOUT.value, {"wage_attenuation": 0.3}
            ),
            self._verb_result(
                ActionType.VIGILANTISM,
                EventType.VIGILANTISM.value,
                {"repression_increment": 0.1},
            ),
        ]
        with patch.object(
            OODASystem, "_resolve_for_organization", return_value=results
        ):
            system.step(self._graph_single_faction(), services, {"tick": 4})

        types = [e.type for e in self._published(services)]
        assert types.count(EventType.LOCKOUT) == 1
        assert types.count(EventType.VIGILANTISM) == 1

    def test_summary_unchanged_and_plain_actions_stay_summary_only(self) -> None:
        system = OODASystem()
        services = _make_services()
        plain = ActionResult(
            action=Action(
                org_id="fash_org", action_type=ActionType.EDUCATE, target_id="C900"
            ),
            success=True,
            events_generated=[EventType.ORGANIZATIONAL_ACTION.value],
        )
        with patch.object(
            OODASystem, "_resolve_for_organization", return_value=[plain]
        ):
            system.step(self._graph_single_faction(), services, {"tick": 5})

        published = self._published(services)
        summaries = [e for e in published if e.type == EventType.ORGANIZATIONAL_ACTION]
        assert len(summaries) == 1
        assert summaries[0].payload["action_count"] == 1
        assert not any(
            e.type in {EventType.POGROM, EventType.LOCKOUT, EventType.VIGILANTISM}
            for e in published
        )
```

- [ ] **Step 6: Run red, then implement the publish seam, then run green.**
      `mise run test:q -- tests/unit/ooda/test_ooda_system.py` — expected failure:
      `assert len(pogroms) == 1` → `assert 0 == 1`.
      Then in `src/babylon/engine/systems/ooda.py` add after the imports (below
      the `TYPE_CHECKING` block, near :36):

```python
#: spec-116 FR-116-4.7: ``ActionResult.events_generated`` values that surface
#: as their own first-class bus events (payload = org/target + the resolver's
#: ``direct_effects``) instead of drowning in the ORGANIZATIONAL_ACTION
#: summary. Only the spec-071 reactionary verbs — STATE_REPRESSION /
#: STATE_SURVEILLANCE keep their existing converter-only path so nothing
#: double-delivers if a bus publisher lands for them later.
_FIRST_CLASS_ACTION_EVENTS: frozenset[str] = frozenset(
    {EventType.POGROM.value, EventType.LOCKOUT.value, EventType.VIGILANTISM.value}
)
```

and replace the `# Emit summary event` block (:185-199) with:

```python
        # Emit events
        if services.event_bus:
            from babylon.kernel.event_bus import Event

            # spec-116 FR-116-4.7: first-class reactionary verb events.
            # Deterministic order (III.7): action_phase_results is
            # initiative-ordered; events_generated iterates in list order.
            for result in action_phase_results:
                for event_value in result.events_generated:
                    if event_value not in _FIRST_CLASS_ACTION_EVENTS:
                        continue
                    services.event_bus.publish(
                        Event(
                            type=EventType(event_value),
                            tick=tick,
                            payload={
                                "org_id": result.action.org_id,
                                "target_id": result.action.target_id,
                                **result.direct_effects,
                            },
                        )
                    )

            # Emit summary event (unchanged — OrganizationalActionEvent
            # consumers expect the aggregate counts payload)
            services.event_bus.publish(
                Event(
                    type=EventType.ORGANIZATIONAL_ACTION,
                    tick=tick,
                    payload={
                        "layer0_count": len(layer0_results),
                        "action_count": len(action_phase_results),
                        "org_count": len(initiative_order),
                    },
                )
            )
```

Re-run: `mise run test:q -- tests/unit/ooda/test_ooda_system.py` — expected all pass
(including the pre-existing classes in the file).

- [ ] **Step 7: Bridge severity + territory anchor (red then green).** Append to
      `tests/unit/web/test_engine_bridge.py` directly after
      `TestSerializeEventUprisingTerritoryAnchoring` (~:2790; reuses the module-level
      `_graph_with_tenancy` helper):

```python
@pytest.mark.unit
class TestReactionaryVerbSeverityAndAnchoring:
    """spec-116 FR-116-4.7: pogrom/lockout/vigilantism severity tier +
    uprising-pattern territory anchoring on the TARGET community id."""

    @staticmethod
    def _verb_event(event_type: str, target_id: str) -> MagicMock:
        event = MagicMock()
        event.event_type = event_type
        event.tick = 5
        event.data = {
            "org_id": "ORG_FASH",
            "target_id": target_id,
            "repression_increment": 0.15,
        }
        event.narrative = None
        return event

    def test_verbs_classify_as_warning(self) -> None:
        from game.engine_bridge import _classify_event

        assert _classify_event("pogrom") == "warning"
        assert _classify_event("lockout") == "warning"
        assert _classify_event("vigilantism") == "warning"

    def test_pogrom_anchors_to_target_territory(self) -> None:
        from game.engine_bridge import _serialize_event

        graph = _graph_with_tenancy(class_to_territory={"C001": "T001"})
        result = _serialize_event(self._verb_event("pogrom", "C001"), uuid.uuid4(), graph=graph)

        assert result["data"]["territory_id"] == "T001"

    def test_lockout_and_vigilantism_anchor_too(self) -> None:
        from game.engine_bridge import _serialize_event

        graph = _graph_with_tenancy(class_to_territory={"C001": "T001"})
        for verb in ("lockout", "vigilantism"):
            result = _serialize_event(self._verb_event(verb, "C001"), uuid.uuid4(), graph=graph)
            assert result["data"]["territory_id"] == "T001"

    def test_unresolvable_target_yields_honest_none(self) -> None:
        from game.engine_bridge import _serialize_event

        graph = _graph_with_tenancy(class_to_territory={"C001": "T001", "C999": None})
        result = _serialize_event(
            self._verb_event("vigilantism", "C999"), uuid.uuid4(), graph=graph
        )

        assert "territory_id" in result["data"]
        assert result["data"]["territory_id"] is None

    def test_absent_graph_yields_none_never_guessed(self) -> None:
        from game.engine_bridge import _serialize_event

        result = _serialize_event(self._verb_event("pogrom", "C001"), uuid.uuid4())

        assert result["data"]["territory_id"] is None
```

Run `mise run test:q -- tests/unit/web/test_engine_bridge.py -k ReactionaryVerb` —
expected failure: `assert 'informational' == 'warning'`. Then in
`web/game/engine_bridge.py`:

(a) add to `_EVENT_SEVERITY`'s warning tier, after `"doctrine_purge_failed": "warning",`
(:6830):

```python
    # spec-116 FR-116-4.7: first-class reactionary verb events (OODASystem
    # per-action publish). Repression-family tier — peers of state_repression
    # / excessive_force; FR-116-2 reserves the critical tier for genuine
    # rupture/endgame proximity, so targeted reactionary violence tiers as
    # warning, not crimson.
    "pogrom": "warning",
    "lockout": "warning",
    "vigilantism": "warning",
```

(b) add immediately after the `_EVENT_SEVERITY` dict closes (:6838):

```python
#: spec-116 FR-116-4.7: reactionary verb events anchor to the TARGET
#: community's territory via the same TENANCY inversion as "uprising"
#: (their payload subject is ``target_id``, a social_class id).
_TERRITORY_ANCHORED_VERB_EVENTS: frozenset[str] = frozenset(
    {"pogrom", "lockout", "vigilantism"}
)
```

(c) extend the anchoring special case in `_serialize_event` (:6905-6910) to:

```python
    if event_type_str == "uprising":
        node_id = data.get("node_id")
        territory_id = None
        if graph is not None and node_id is not None:
            territory_id = _class_to_territory(_tenancy_members_by_territory(graph)).get(node_id)
        data = {**data, "territory_id": territory_id}
    elif event_type_str in _TERRITORY_ANCHORED_VERB_EVENTS:
        # spec-116 FR-116-4.7: same TENANCY inversion, subject is target_id.
        # Unresolvable/absent graph honestly yields None, never a guess (III.11).
        target_id = data.get("target_id")
        territory_id = None
        if graph is not None and target_id is not None:
            territory_id = _class_to_territory(_tenancy_members_by_territory(graph)).get(target_id)
        data = {**data, "territory_id": territory_id}
```

Re-run `mise run test:q -- tests/unit/web/test_engine_bridge.py` — expected: full file
passes (severity + anchoring + all pre-existing).

- [ ] **Step 8: Frontend verification pins (already-green characterization, not a red
      phase — eventClassifier.ts:81-83/236-238 already tier the three verbs; these pins
      stop silent drift).** Append to
      `src/frontend/src/lib/__tests__/eventClassifier.test.ts`:

```typescript
describe("spec-116 4d.7 — first-class reactionary verb events", () => {
  it.each(["pogrom", "lockout", "vigilantism"])(
    "classifies '%s' as important / struggle / urgent",
    (type) => {
      expect(classifyEvent(makeEvent(type), 0).severity).toBe("important");
      const se = classifyEventForStream(makeEvent(type), 0);
      expect(se.category).toBe("struggle");
      expect(se.stream).toBe("urgent");
    },
  );
});
```

Run: `cd src/frontend && npx vitest run src/lib/__tests__/eventClassifier.test.ts` —
expected: all pass immediately.

- [ ] **Step 9: Gate + commit.** This task is byte-safe by construction for the
      regression scenarios: converter/severity/serializer are observation-boundary, and
      the new OODA publish fires only when an ActionResult carries a fascist-verb value —
      the verbs are unreachable in the live loop (not in VERB_RESOLVERS /
      VERB_TO_ACTION_TYPE / npc_stub), so no baseline scenario produces one. Verify:
      `mise run check` green, then `mise run qa:regression` — expected
      `Results: 5 passed, 0 failed` byte-identical. Then:

```
git add src/babylon/models/events/reactionary_payloads.py \
        src/babylon/engine/simulation_engine.py \
        src/babylon/engine/systems/ooda.py \
        web/game/engine_bridge.py \
        tests/unit/engine/test_event_conversion.py \
        tests/unit/ooda/test_ooda_system.py \
        tests/unit/web/test_engine_bridge.py \
        src/frontend/src/lib/__tests__/eventClassifier.test.ts
git commit  # via: mise run commit -- "feat(events): spec-116 4d.7 — POGROM/LOCKOUT/VIGILANTISM first-class wire events (47->50/83)"
```

---

### Task 23: Whitelist sweep — the 14 remaining live-emitter EventTypes

**Files:**

- Create: `src/babylon/models/events/spine_payloads.py` (10 new payload classes; the
  other 4 types reuse existing classes: `SecessionDeclaredPayload`
  [balkanization_payloads.py:65-72] and `AxiomViolationEvent` /
  `QcewCarryForwardEvent` / `PhiHourOutlierEvent` [_legacy.py:1039-1133])
- Modify: `src/babylon/engine/simulation_engine.py:78-97` (add
  `AxiomViolationEvent, PhiHourOutlierEvent, QcewCarryForwardEvent` to the
  `from babylon.models.events import (...)` block), `:98-105` (add
  `SecessionDeclaredPayload` to the balkanization block), after `:134` (new
  `spine_payloads` import block, sorts between `reactionary_payloads` and
  `struggle_payloads`), converter docstring ledger, and 14 branches inserted
  immediately before the terminal `return None` drop comment (post-Task-22 the
  DOCTRINE_PURGE_FAILED branch ends ~:1045; anchor on the
  `# Feature 002 events ... graceful degradation` comment) + rewrite of that terminal
  comment
- Modify: `web/game/engine_bridge.py` — `_EVENT_SEVERITY` (+14 entries),
  new `_EVENT_TITLE_OVERRIDES` dict + `_humanize_event_type` (:6850-6852)
- Modify: `src/frontend/src/lib/eventClassifier.ts:44-132` (`EVENT_SEVERITY_MAP` +
  `market_correction`) and `:230-326` (`CATEGORY_MAP` + `market_correction`) — the
  other 13 types already have entries (spec-113 Lane E); MARKET_CORRECTION (82nd
  member, added by P23 after Lane E — PATTERN_SHIFT from Task 4 is the 83rd) is
  missing from both maps
- Test: `tests/unit/engine/test_event_conversion.py` (+7 test classes),
  `tests/unit/web/test_engine_bridge.py` (+1 severity/title class),
  `src/frontend/src/lib/__tests__/eventClassifier.test.ts` (+market_correction),
  `tests/baselines/*.json` + `tests/baselines/dense/*.csv` (ceremony regeneration)

**Interfaces:**

- Consumes: converter state after Task 22 (50 of 83 handled; ledger line present);
  existing payload classes named above; emitter payload shapes verified at
  market_scissors.py:342-356, vitality.py:131-142/181-194,
  domain/economics/tick/system/__init__.py:994-1006 + 1701-1715,
  edge_transition/_legacy.py:666-679/772-795/848-859, contradiction.py:524-536,
  faction_influence.py:229-240, leontief_rent emitters (typed `model_dump()` payloads).
- Produces: `MarketCorrectionEvent`, `EntityDeathEvent`, `PopulationAttritionEvent`,
  `CrisisPhaseTransitionEvent`, `BifurcationThresholdEvent`, `EdgeModeTransitionEvent`,
  `CoOptiveBreakdownEvent`, `LatentContradictionReleaseEvent`, `AspectReversalEvent`,
  `LevelTransitionEvent` in `babylon.models.events.spine_payloads`;
  `_EVENT_TITLE_OVERRIDES: dict[str, str]` (engine_bridge); converter at 64 of 83.
  Later salience tasks (FR-116-2 dedup/retier) may rely on every one of these types
  carrying a real severity tier instead of the silent informational default.

- [ ] **Step 1: Write the failing conversion tests** — append to
      `tests/unit/engine/test_event_conversion.py`, adding these imports
      (`SecessionDeclaredPayload` joins the balkanization import block at :37-44; the
      calibration trio import via `babylon.models.events`; new module import after the
      reactionary block):

```python
from babylon.models.events import (  # noqa: F811 — extend the existing block instead
    AxiomViolationEvent,
    PhiHourOutlierEvent,
    QcewCarryForwardEvent,
)
from babylon.models.events.spine_payloads import (
    AspectReversalEvent,
    BifurcationThresholdEvent,
    CoOptiveBreakdownEvent,
    CrisisPhaseTransitionEvent,
    EdgeModeTransitionEvent,
    EntityDeathEvent,
    LatentContradictionReleaseEvent,
    LevelTransitionEvent,
    MarketCorrectionEvent,
    PopulationAttritionEvent,
)
```

(Implementation note: fold the three `_legacy` names into the existing
`from babylon.models.events import (...)` block at :23-36 rather than a duplicate
import — shown separately here only for reviewability.)

```python
class TestMarketCorrectionEventConversion:
    """spec-116 FR-116-4.7 sweep: MARKET_CORRECTION (market_scissors.py:342-356)."""

    def test_converts_market_correction_event(self) -> None:
        bus_event = Event(
            type=EventType.MARKET_CORRECTION,
            tick=40,
            payload={
                "overhang": 1.8,
                "serviceable": 1.2,
                "profit_rate": 0.05,
                "fictitious_log_before": 2.4,
                "fictitious_log_after": 0.96,
                "price_log_before": 1.1,
                "price_log_after": 0.7,
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, MarketCorrectionEvent)
        assert result.event_type == EventType.MARKET_CORRECTION
        assert result.tick == 40
        assert result.overhang == 1.8
        assert result.serviceable == 1.2
        assert result.profit_rate == 0.05
        assert result.fictitious_log_before == 2.4
        assert result.fictitious_log_after == 0.96
        assert result.price_log_before == 1.1
        assert result.price_log_after == 0.7


class TestVitalityEventConversion:
    """spec-116 FR-116-4.7 sweep: ENTITY_DEATH + POPULATION_ATTRITION (vitality.py)."""

    def test_converts_entity_death_event(self) -> None:
        bus_event = Event(
            type=EventType.ENTITY_DEATH,
            tick=30,
            payload={
                "entity_id": PERIPHERY_WORKER_ID,
                "wealth": 0.4,
                "consumption_needs": 2.0,
                "s_bio": 1.5,
                "s_class": 0.5,
                "cause": "starvation",
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, EntityDeathEvent)
        assert result.event_type == EventType.ENTITY_DEATH
        assert result.tick == 30
        assert result.entity_id == PERIPHERY_WORKER_ID
        assert result.wealth == 0.4
        assert result.consumption_needs == 2.0
        assert result.s_bio == 1.5
        assert result.s_class == 0.5
        assert result.cause == "starvation"

    def test_converts_population_attrition_event(self) -> None:
        bus_event = Event(
            type=EventType.POPULATION_ATTRITION,
            tick=29,
            payload={
                "entity_id": PERIPHERY_WORKER_ID,
                "deaths": 42,
                "remaining_population": 958,
                "attrition_rate": 0.042,
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, PopulationAttritionEvent)
        assert result.event_type == EventType.POPULATION_ATTRITION
        assert result.entity_id == PERIPHERY_WORKER_ID
        assert result.deaths == 42
        assert result.remaining_population == 958
        assert result.attrition_rate == 0.042


class TestCountyCrisisSignalConversion:
    """spec-116 FR-116-4.7 sweep: CRISIS_PHASE_TRANSITION + BIFURCATION_THRESHOLD.

    Both publishers pass ``.value`` STRINGS (tick/system/__init__.py:996, :1703),
    so these tests exercise the string-normalization path deliberately."""

    def test_converts_crisis_phase_transition_event(self) -> None:
        bus_event = Event(
            type="crisis_phase_transition",  # type: ignore[arg-type]
            tick=12,
            payload={
                "fips": "26163",
                "previous_phase": "normal",
                "new_phase": "onset",
                "profit_rate": 0.08,
                "crisis_duration": 0,
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, CrisisPhaseTransitionEvent)
        assert result.event_type == EventType.CRISIS_PHASE_TRANSITION
        assert result.fips == "26163"
        assert result.previous_phase == "normal"
        assert result.new_phase == "onset"
        assert result.profit_rate == 0.08
        assert result.crisis_duration == 0

    def test_crisis_phase_transition_none_profit_rate(self) -> None:
        bus_event = Event(
            type="crisis_phase_transition",  # type: ignore[arg-type]
            tick=12,
            payload={"fips": "26163", "previous_phase": "onset", "new_phase": "acute"},
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert isinstance(result, CrisisPhaseTransitionEvent)
        assert result.profit_rate is None

    def test_converts_bifurcation_threshold_event(self) -> None:
        bus_event = Event(
            type="bifurcation_threshold",  # type: ignore[arg-type]
            tick=13,
            payload={
                "fips": "26163",
                "score": -0.41,
                "direction": "revolutionary",
                "solidarity_density": 0.3,
                "legitimation": 0.55,
                "class_burden_ratio": 1.2,
                "threshold": 0.35,
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, BifurcationThresholdEvent)
        assert result.event_type == EventType.BIFURCATION_THRESHOLD
        assert result.fips == "26163"
        assert result.score == -0.41
        assert result.direction == "revolutionary"
        assert result.solidarity_density == 0.3
        assert result.legitimation == 0.55
        assert result.class_burden_ratio == 1.2
        assert result.threshold == 0.35


class TestEdgeTransitionFamilyConversion:
    """spec-116 FR-116-4.7 sweep: the Feature-002 family the drop comment named
    (edge_transition/_legacy.py:666-679, :772-795, :848-859)."""

    def test_converts_edge_mode_transition_event(self) -> None:
        bus_event = Event(
            type=EventType.EDGE_MODE_TRANSITION,
            tick=8,
            payload={
                "source_id": CORE_BOURGEOISIE_ID,
                "target_id": LABOR_ARISTOCRACY_ID,
                "from_mode": "co-optive",
                "to_mode": "antagonistic",
                "predicate": "TENSION_THRESHOLD",
                "description": "Co-optation exhausted",
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, EdgeModeTransitionEvent)
        assert result.event_type == EventType.EDGE_MODE_TRANSITION
        assert result.source_id == CORE_BOURGEOISIE_ID
        assert result.target_id == LABOR_ARISTOCRACY_ID
        assert result.from_mode == "co-optive"
        assert result.to_mode == "antagonistic"
        assert result.predicate == "TENSION_THRESHOLD"
        assert result.description == "Co-optation exhausted"

    def test_converts_co_optive_breakdown_event(self) -> None:
        bus_event = Event(
            type=EventType.CO_OPTIVE_BREAKDOWN,
            tick=9,
            payload={
                "source_id": CORE_BOURGEOISIE_ID,
                "target_id": LABOR_ARISTOCRACY_ID,
                "latent_released": {"class_tension": 0.4},
                "multiplier": 1.5,
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, CoOptiveBreakdownEvent)
        assert result.latent_released == {"class_tension": 0.4}
        assert result.multiplier == 1.5

    def test_converts_latent_contradiction_release_event(self) -> None:
        bus_event = Event(
            type=EventType.LATENT_CONTRADICTION_RELEASE,
            tick=9,
            payload={
                "node_id": CORE_BOURGEOISIE_ID,
                "released_fields": {"class_tension": 0.6},
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, LatentContradictionReleaseEvent)
        assert result.node_id == CORE_BOURGEOISIE_ID
        assert result.released_fields == {"class_tension": 0.6}

    def test_converts_aspect_reversal_event(self) -> None:
        bus_event = Event(
            type=EventType.ASPECT_REVERSAL,
            tick=10,
            payload={
                "source_id": PERIPHERY_WORKER_ID,
                "target_id": COMPRADOR_ID,
                "previous_dominant": COMPRADOR_ID,
                "new_dominant": PERIPHERY_WORKER_ID,
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, AspectReversalEvent)
        assert result.previous_dominant == COMPRADOR_ID
        assert result.new_dominant == PERIPHERY_WORKER_ID


class TestLevelTransitionEventConversion:
    """spec-116 FR-116-4.7 sweep: LEVEL_TRANSITION (contradiction.py:524-536)."""

    def test_converts_level_transition_event(self) -> None:
        bus_event = Event(
            type=EventType.LEVEL_TRANSITION,
            tick=15,
            payload={
                "opposition": "price_value",
                "from_level": "county",
                "to_level": "national",
                "gap": 0.2,
                "rate": 0.01,
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, LevelTransitionEvent)
        assert result.opposition == "price_value"
        assert result.from_level == "county"
        assert result.to_level == "national"
        assert result.gap == 0.2
        assert result.rate == 0.01


class TestSecessionDeclaredEventConversion:
    """spec-116 FR-116-4.7 sweep: SECESSION_DECLARED reuses the spec-070
    SecessionDeclaredPayload (faction_influence.py:229-240)."""

    def test_converts_secession_declared_event(self) -> None:
        bus_event = Event(
            type=EventType.SECESSION_DECLARED,
            tick=200,
            payload={
                "secessionist_faction_id": "FAC_RED_STATE",
                "parent_sovereign_id": "SOV_USA",
                "contiguous_territory_ids": ("T001", "T002"),
                "observer_triggered": False,
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, SecessionDeclaredPayload)
        assert result.event_type == EventType.SECESSION_DECLARED
        assert result.secessionist_faction_id == "FAC_RED_STATE"
        assert result.parent_sovereign_id == "SOV_USA"
        assert result.contiguous_territory_ids == ("T001", "T002")
        assert result.observer_triggered is False


class TestCalibrationWarningConversion:
    """spec-116 FR-116-4.7 sweep: the dotted-value calibration trio. Publishers
    pass ``.value`` strings and payload = the typed event's model_dump()
    (periphery_labor_coefficients.py:222-238, industry_to_county_allocator.py:269-305)."""

    def test_converts_axiom_violation_event(self) -> None:
        bus_event = Event(
            type="calibration_warning.axiom_violation",  # type: ignore[arg-type]
            tick=0,
            payload={"industry": "334", "year": 2019, "ratio": 0.87, "threshold": 1.0},
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, AxiomViolationEvent)
        assert result.event_type == EventType.CALIBRATION_AXIOM_VIOLATION
        assert result.industry == "334"
        assert result.year == 2019
        assert result.ratio == 0.87
        assert result.threshold == 1.0

    def test_converts_qcew_carry_forward_event(self) -> None:
        bus_event = Event(
            type="calibration_warning.qcew_carry_forward",  # type: ignore[arg-type]
            tick=0,
            payload={
                "county_fips": "26163",
                "year": 2020,
                "look_back_year": 2018,
                "look_back_distance": 2,
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, QcewCarryForwardEvent)
        assert result.county_fips == "26163"
        assert result.year == 2020
        assert result.look_back_year == 2018
        assert result.look_back_distance == 2

    def test_converts_phi_hour_outlier_event(self) -> None:
        bus_event = Event(
            type="calibration_warning.phi_hour_outlier",  # type: ignore[arg-type]
            tick=0,
            payload={
                "county_fips": "26163",
                "phi_hour": 1500.0,
                "threshold_low": -1000.0,
                "threshold_high": 1000.0,
            },
        )
        result = _convert_bus_event_to_pydantic(bus_event)

        assert result is not None
        assert isinstance(result, PhiHourOutlierEvent)
        assert result.county_fips == "26163"
        assert result.phi_hour == 1500.0
        assert result.threshold_low == -1000.0
        assert result.threshold_high == 1000.0
```

(`SecessionDeclaredPayload` joins the existing balkanization import at :37-44.)

- [ ] **Step 2: Run to verify failure** —
      `mise run test:q -- tests/unit/engine/test_event_conversion.py`
      Expected: collection error
      `ModuleNotFoundError: No module named 'babylon.models.events.spine_payloads'`.

- [ ] **Step 3: Minimal implementation — payload module + 14 branches.**
      Create `src/babylon/models/events/spine_payloads.py`:

```python
"""Spec-116 FR-116-4.7 event-whitelist payloads (Playability Spine).

Each payload is a frozen Pydantic model mirroring the ``payload={...}`` dict
built at its live bus-publish site (named per class). These are the ten
event types the 4d.7 whitelist sweep promoted onto the wire that had no
existing payload class — SECESSION_DECLARED reuses
:class:`~babylon.models.events.balkanization_payloads.SecessionDeclaredPayload`
and the calibration trio reuses the ``_legacy`` classes.

Light Program-17 pattern (like ``reactionary_payloads`` /
``struggle_payloads``): no ``kind`` field, NOT in the ``TickEvent``
discriminated union — on a WorldState graph round-trip these replay as bare
:class:`~babylon.models.events._legacy.SimulationEvent` with a loud WARNING
(``world_state._validate_event``). Wire delivery is same-tick from live
``WorldState.events``, so the toast/journal path is unaffected.

Naming convention: ``{EventType}Event`` (legacy suffix).
"""

from __future__ import annotations

from pydantic import Field

from babylon.models.enums import EventType
from babylon.models.events._legacy import SimulationEvent


class MarketCorrectionEvent(SimulationEvent):
    """MARKET_CORRECTION event payload (market_scissors.py:342-356).

    ADR078: the scissors snapped — fictitious/real divergence exceeded
    profit-rate serviceability and the correction fired live.
    """

    event_type: EventType = Field(default=EventType.MARKET_CORRECTION)
    overhang: float
    serviceable: float
    profit_rate: float
    fictitious_log_before: float
    fictitious_log_after: float
    price_log_before: float
    price_log_after: float


class EntityDeathEvent(SimulationEvent):
    """ENTITY_DEATH event payload (vitality.py:181-194).

    A social-class node's full extinction (extinction / starvation /
    wealth-threshold zombie trap).
    """

    event_type: EventType = Field(default=EventType.ENTITY_DEATH)
    entity_id: str
    wealth: float
    consumption_needs: float
    s_bio: float
    s_class: float
    cause: str


class PopulationAttritionEvent(SimulationEvent):
    """POPULATION_ATTRITION event payload (vitality.py:131-142).

    Grinding attrition — coverage-ratio threshold mortality below extinction.
    """

    event_type: EventType = Field(default=EventType.POPULATION_ATTRITION)
    entity_id: str
    deaths: int
    remaining_population: int
    attrition_rate: float


class CrisisPhaseTransitionEvent(SimulationEvent):
    """CRISIS_PHASE_TRANSITION event payload (tick/system/__init__.py:994-1006).

    A county's crisis phase changed (FR-004/FR-022 of the tick-dynamics spec).
    ``profit_rate`` is honestly None when the publisher had none.
    """

    event_type: EventType = Field(default=EventType.CRISIS_PHASE_TRANSITION)
    fips: str
    previous_phase: str
    new_phase: str
    profit_rate: float | None = None
    crisis_duration: int


class BifurcationThresholdEvent(SimulationEvent):
    """BIFURCATION_THRESHOLD event payload (tick/system/__init__.py:1701-1715).

    A county's bifurcation-risk metric crossed the threshold; ``direction``
    is "revolutionary" (score < 0) or "fascist".
    """

    event_type: EventType = Field(default=EventType.BIFURCATION_THRESHOLD)
    fips: str
    score: float
    direction: str
    solidarity_density: float
    legitimation: float
    class_burden_ratio: float
    threshold: float


class EdgeModeTransitionEvent(SimulationEvent):
    """EDGE_MODE_TRANSITION event payload (edge_transition/_legacy.py:666-679).

    Feature 002: an edge's qualitative contradiction mode changed.
    """

    event_type: EventType = Field(default=EventType.EDGE_MODE_TRANSITION)
    source_id: str
    target_id: str
    from_mode: str
    to_mode: str
    predicate: str
    description: str


class CoOptiveBreakdownEvent(SimulationEvent):
    """CO_OPTIVE_BREAKDOWN event payload (edge_transition/_legacy.py:772-783).

    A CO-OPTIVE edge broke down; suppressed contradiction is released.
    """

    event_type: EventType = Field(default=EventType.CO_OPTIVE_BREAKDOWN)
    source_id: str
    target_id: str
    latent_released: dict[str, float] = Field(default_factory=dict)
    multiplier: float


class LatentContradictionReleaseEvent(SimulationEvent):
    """LATENT_CONTRADICTION_RELEASE event payload (edge_transition/_legacy.py:785-795).

    The multiplier-scaled latent field spike accompanying a breakdown.
    """

    event_type: EventType = Field(default=EventType.LATENT_CONTRADICTION_RELEASE)
    node_id: str
    released_fields: dict[str, float] = Field(default_factory=dict)


class AspectReversalEvent(SimulationEvent):
    """ASPECT_REVERSAL event payload (edge_transition/_legacy.py:848-859).

    FR-019: the dominant party (material power) on a directed edge switched.
    """

    event_type: EventType = Field(default=EventType.ASPECT_REVERSAL)
    source_id: str
    target_id: str
    previous_dominant: str
    new_dominant: str


class LevelTransitionEvent(SimulationEvent):
    """LEVEL_TRANSITION event payload (contradiction.py:524-536).

    Lawverian sublation — an opposition's contradiction lifted to a higher
    level of the lattice (aufhebung).
    """

    event_type: EventType = Field(default=EventType.LEVEL_TRANSITION)
    opposition: str
    from_level: str
    to_level: str
    gap: float
    rate: float


__all__ = [
    "AspectReversalEvent",
    "BifurcationThresholdEvent",
    "CoOptiveBreakdownEvent",
    "CrisisPhaseTransitionEvent",
    "EdgeModeTransitionEvent",
    "EntityDeathEvent",
    "LatentContradictionReleaseEvent",
    "LevelTransitionEvent",
    "MarketCorrectionEvent",
    "PopulationAttritionEvent",
]
```

In `simulation_engine.py`: add `AxiomViolationEvent`, `PhiHourOutlierEvent`,
`QcewCarryForwardEvent` (alphabetically) to the `from babylon.models.events import
(...)` block; add `SecessionDeclaredPayload` to the balkanization block; add:

```python
from babylon.models.events.spine_payloads import (
    AspectReversalEvent,
    BifurcationThresholdEvent,
    CoOptiveBreakdownEvent,
    CrisisPhaseTransitionEvent,
    EdgeModeTransitionEvent,
    EntityDeathEvent,
    LatentContradictionReleaseEvent,
    LevelTransitionEvent,
    MarketCorrectionEvent,
    PopulationAttritionEvent,
)
```

Insert the 14 branches immediately before the terminal drop comment (which the same
edit rewrites — see below):

```python
    # Spec-116 FR-116-4.7 sweep: every remaining EventType with a live bus
    # publisher. Payload shapes mirror the publish sites named on each class.
    if event_type == EventType.MARKET_CORRECTION:
        return MarketCorrectionEvent(
            tick=tick,
            timestamp=timestamp,
            overhang=payload.get("overhang", 0.0),
            serviceable=payload.get("serviceable", 0.0),
            profit_rate=payload.get("profit_rate", 0.0),
            fictitious_log_before=payload.get("fictitious_log_before", 0.0),
            fictitious_log_after=payload.get("fictitious_log_after", 0.0),
            price_log_before=payload.get("price_log_before", 0.0),
            price_log_after=payload.get("price_log_after", 0.0),
        )

    if event_type == EventType.ENTITY_DEATH:
        return EntityDeathEvent(
            tick=tick,
            timestamp=timestamp,
            entity_id=payload.get("entity_id", ""),
            wealth=payload.get("wealth", 0.0),
            consumption_needs=payload.get("consumption_needs", 0.0),
            s_bio=payload.get("s_bio", 0.0),
            s_class=payload.get("s_class", 0.0),
            cause=payload.get("cause", "unknown"),
        )

    if event_type == EventType.POPULATION_ATTRITION:
        return PopulationAttritionEvent(
            tick=tick,
            timestamp=timestamp,
            entity_id=payload.get("entity_id", ""),
            deaths=payload.get("deaths", 0),
            remaining_population=payload.get("remaining_population", 0),
            attrition_rate=payload.get("attrition_rate", 0.0),
        )

    if event_type == EventType.CRISIS_PHASE_TRANSITION:
        return CrisisPhaseTransitionEvent(
            tick=tick,
            timestamp=timestamp,
            fips=payload.get("fips", ""),
            previous_phase=payload.get("previous_phase", ""),
            new_phase=payload.get("new_phase", ""),
            profit_rate=payload.get("profit_rate"),
            crisis_duration=payload.get("crisis_duration", 0),
        )

    if event_type == EventType.BIFURCATION_THRESHOLD:
        return BifurcationThresholdEvent(
            tick=tick,
            timestamp=timestamp,
            fips=payload.get("fips", ""),
            score=payload.get("score", 0.0),
            direction=payload.get("direction", ""),
            solidarity_density=payload.get("solidarity_density", 0.0),
            legitimation=payload.get("legitimation", 0.0),
            class_burden_ratio=payload.get("class_burden_ratio", 0.0),
            threshold=payload.get("threshold", 0.0),
        )

    if event_type == EventType.EDGE_MODE_TRANSITION:
        return EdgeModeTransitionEvent(
            tick=tick,
            timestamp=timestamp,
            source_id=payload.get("source_id", ""),
            target_id=payload.get("target_id", ""),
            from_mode=str(payload.get("from_mode", "")),
            to_mode=str(payload.get("to_mode", "")),
            predicate=payload.get("predicate", ""),
            description=payload.get("description", ""),
        )

    if event_type == EventType.CO_OPTIVE_BREAKDOWN:
        return CoOptiveBreakdownEvent(
            tick=tick,
            timestamp=timestamp,
            source_id=payload.get("source_id", ""),
            target_id=payload.get("target_id", ""),
            latent_released=dict(payload.get("latent_released", {})),
            multiplier=payload.get("multiplier", 0.0),
        )

    if event_type == EventType.LATENT_CONTRADICTION_RELEASE:
        return LatentContradictionReleaseEvent(
            tick=tick,
            timestamp=timestamp,
            node_id=payload.get("node_id", ""),
            released_fields=dict(payload.get("released_fields", {})),
        )

    if event_type == EventType.ASPECT_REVERSAL:
        return AspectReversalEvent(
            tick=tick,
            timestamp=timestamp,
            source_id=payload.get("source_id", ""),
            target_id=payload.get("target_id", ""),
            previous_dominant=payload.get("previous_dominant", ""),
            new_dominant=payload.get("new_dominant", ""),
        )

    if event_type == EventType.LEVEL_TRANSITION:
        return LevelTransitionEvent(
            tick=tick,
            timestamp=timestamp,
            opposition=payload.get("opposition", ""),
            from_level=payload.get("from_level", ""),
            to_level=payload.get("to_level", ""),
            gap=payload.get("gap", 0.0),
            rate=payload.get("rate", 0.0),
        )

    if event_type == EventType.SECESSION_DECLARED:
        return SecessionDeclaredPayload(
            tick=tick,
            timestamp=timestamp,
            secessionist_faction_id=payload.get("secessionist_faction_id", ""),
            parent_sovereign_id=payload.get("parent_sovereign_id", ""),
            contiguous_territory_ids=tuple(payload.get("contiguous_territory_ids", ())),
            observer_triggered=payload.get("observer_triggered", False),
        )

    # Calibration-warning trio: bus payloads are model_dump()s of the typed
    # events themselves (leontief_rent emitters), republished here typed.
    if event_type == EventType.CALIBRATION_AXIOM_VIOLATION:
        return AxiomViolationEvent(
            tick=tick,
            timestamp=timestamp,
            industry=payload.get("industry", ""),
            year=payload.get("year", 0),
            ratio=payload.get("ratio", 0.0),
            threshold=payload.get("threshold", 1.0),
        )

    if event_type == EventType.CALIBRATION_QCEW_CARRY_FORWARD:
        return QcewCarryForwardEvent(
            tick=tick,
            timestamp=timestamp,
            county_fips=payload.get("county_fips", ""),
            year=payload.get("year", 0),
            look_back_year=payload.get("look_back_year", 0),
            look_back_distance=payload.get("look_back_distance", 0),
        )

    if event_type == EventType.CALIBRATION_PHI_HOUR_OUTLIER:
        return PhiHourOutlierEvent(
            tick=tick,
            timestamp=timestamp,
            county_fips=payload.get("county_fips", ""),
            phi_hour=payload.get("phi_hour", 0.0),
            threshold_low=payload.get("threshold_low", -1000.0),
            threshold_high=payload.get("threshold_high", 1000.0),
        )
```

(Malformed payloads fail loud via Pydantic validation — the existing balkanization-
branch idiom; the `.get` defaults for constrained fields like ``year`` are
deliberately invalid so a missing key raises instead of masking, III.11.)

Rewrite the terminal drop comment (:1014-1017 pre-shift) as:

```python
    # Dead enum values (no live bus publisher) stay dropped — graceful
    # degradation: SOLIDARITY_AWAKENING, POPULATION_DEATH,
    # EXPLOITATION_MODE_SHIFT, DUAL_CIRCUIT_INTERFERENCE, CONSCIOUSNESS_SHIFT,
    # INITIATIVE_CONTESTED, INFRASTRUCTURE_CHANGE, CALIBRATION_DISAGREEMENT,
    # STATE_ACTION_EXECUTED, FASCIST_CONVERGENCE, FACTION_SHIFT,
    # THREAD_ESCALATION, LEGAL_FRAMEWORK_ENACTED, LEGAL_FRAMEWORK_REVOKED,
    # INSTITUTION_REPRODUCTION, BIFURCATION_TENDENCY_CHANGE, RED_OGV_ENDGAME,
    # FRAGMENTED_COLLAPSE_ENDGAME. ENDGAME_REACHED is deliberately absent:
    # EndgameDetector injects an already-typed EndgameEvent via
    # persistent_context["_observer_events"] — a branch here would
    # double-deliver it.
    return None
```

Append to the docstring ledger:

```python
    Spec-116 FR-116-4.7 sweep: widened to 64 of 83 EventTypes — every
    EventType with a live bus publisher now converts; the 19 remaining are
    dead enum values (no publisher) or ENDGAME_REACHED (observer path).
```

- [ ] **Step 4: Run to verify pass** —
      `mise run test:q -- tests/unit/engine/test_event_conversion.py`
      Expected: all pass. Also
      `mise run test:q -- tests/unit/sentinels/test_seam_registry_check.py`
      — the coverage advisory test asserts message format only (`"drop to None"`), and
      19 types still drop, so it stays green with updated counts in the message.

- [ ] **Step 5: Bridge severity + humanized titles (red).** Append to
      `tests/unit/web/test_engine_bridge.py`:

```python
@pytest.mark.unit
class TestSpineWhitelistSeverityAndTitles:
    """spec-116 FR-116-4.7 sweep: severity tiers + humanized titles for the
    14 newly wired types (FR-116-2 reserves critical for genuine
    rupture/endgame proximity; secession IS fragmented-collapse proximity)."""

    @pytest.mark.parametrize(
        ("event_type", "expected"),
        [
            ("market_correction", "warning"),
            ("entity_death", "warning"),
            ("population_attrition", "informational"),
            ("crisis_phase_transition", "warning"),
            ("bifurcation_threshold", "warning"),
            ("edge_mode_transition", "informational"),
            ("co_optive_breakdown", "warning"),
            ("latent_contradiction_release", "informational"),
            ("aspect_reversal", "informational"),
            ("level_transition", "warning"),
            ("secession_declared", "critical"),
            ("calibration_warning.axiom_violation", "informational"),
            ("calibration_warning.qcew_carry_forward", "informational"),
            ("calibration_warning.phi_hour_outlier", "informational"),
        ],
    )
    def test_severity(self, event_type: str, expected: str) -> None:
        from game.engine_bridge import _classify_event

        assert _classify_event(event_type) == expected

    @pytest.mark.parametrize(
        ("event_type", "expected"),
        [
            # Overrides — dotted/hyphenated values the naive title() mangles,
            # plus the one player-facing rename.
            ("co_optive_breakdown", "Co-optive Breakdown"),
            ("calibration_warning.axiom_violation", "Calibration: Axiom Violation"),
            ("calibration_warning.qcew_carry_forward", "Calibration: QCEW Carry-Forward"),
            ("calibration_warning.phi_hour_outlier", "Calibration: Phi-Hour Outlier"),
            ("entity_death", "Class Extinction"),
            # Default humanization still applies to everything else.
            ("market_correction", "Market Correction"),
            ("secession_declared", "Secession Declared"),
            ("pogrom", "Pogrom"),
        ],
    )
    def test_humanized_titles(self, event_type: str, expected: str) -> None:
        from game.engine_bridge import _humanize_event_type

        assert _humanize_event_type(event_type) == expected
```

Run `mise run test:q -- tests/unit/web/test_engine_bridge.py -k SpineWhitelist` —
expected failures: `assert 'informational' == 'warning'` (severity) and
`assert 'Calibration Warning.Axiom Violation' == 'Calibration: Axiom Violation'`
(titles).

- [ ] **Step 6: Bridge implementation (green).** In `web/game/engine_bridge.py`:

(a) `_EVENT_SEVERITY` — critical tier, after `"doctrine_trap_sprung": "critical",`:

```python
    # spec-116 FR-116-4.7 sweep: a declared secession is fragmented-collapse
    # proximity — genuine rupture, peer of power_vacuum.
    "secession_declared": "critical",
```

warning tier, after the Task-22 verb entries:

```python
    # spec-116 FR-116-4.7 sweep: threshold-cross tier.
    "market_correction": "warning",
    "entity_death": "warning",
    "crisis_phase_transition": "warning",
    "bifurcation_threshold": "warning",
    "co_optive_breakdown": "warning",
    "level_transition": "warning",
```

informational tier, after `"reserve_army_pressure": "informational",`:

```python
    # spec-116 FR-116-4.7 sweep: routine-flow tier (edge/aspect churn is
    # high-volume per tick; calibration warnings are data-quality signals).
    "population_attrition": "informational",
    "edge_mode_transition": "informational",
    "latent_contradiction_release": "informational",
    "aspect_reversal": "informational",
    "calibration_warning.axiom_violation": "informational",
    "calibration_warning.qcew_carry_forward": "informational",
    "calibration_warning.phi_hour_outlier": "informational",
```

(b) replace `_humanize_event_type` (:6850-6852) with an override-aware version:

```python
#: spec-116 FR-116-4.7: title overrides where the naive ``title()`` humanization
#: mangles the value (dotted calibration values, hyphenated co-optive) or where
#: the player-facing name should say what the event MEANS (entity_death is a
#: social class dying out). Copy lives in data, not conditionals.
_EVENT_TITLE_OVERRIDES: dict[str, str] = {
    "co_optive_breakdown": "Co-optive Breakdown",
    "calibration_warning.axiom_violation": "Calibration: Axiom Violation",
    "calibration_warning.qcew_carry_forward": "Calibration: QCEW Carry-Forward",
    "calibration_warning.phi_hour_outlier": "Calibration: Phi-Hour Outlier",
    "entity_death": "Class Extinction",
}


def _humanize_event_type(event_type_str: str) -> str:
    """Convert ``"economic_crisis"`` to ``"Economic Crisis"`` for UI titles.

    spec-116 FR-116-4.7: ``_EVENT_TITLE_OVERRIDES`` wins where the naive
    underscore-to-space humanization would mangle the value.
    """
    return _EVENT_TITLE_OVERRIDES.get(
        event_type_str, event_type_str.replace("_", " ").title()
    )
```

Run `mise run test:q -- tests/unit/web/test_engine_bridge.py` — expected: full file
passes. (`check_severity_vocabulary` GATES on every `_EVENT_SEVERITY` key being a real
`EventType` value — all 14 are, including the dotted trio; verify with
`mise run check:seams`, expected exit 0.)

- [ ] **Step 7: Frontend classifier — market_correction (red then green).** The other
      13 types already carry spec-113 Lane E entries; MARKET_CORRECTION (82nd member,
      P23 — post-Lane-E) is missing from both maps. Append to
      `src/frontend/src/lib/__tests__/eventClassifier.test.ts`:

```typescript
describe("spec-116 4d.7 — market_correction joins the classifier (P23, post-Lane-E)", () => {
  it("classifies 'market_correction' as important", () => {
    expect(classifyEvent(makeEvent("market_correction"), 0).severity).toBe("important");
  });

  it("streams 'market_correction' as notable / urgent / economy", () => {
    const se = classifyEventForStream(makeEvent("market_correction"), 0);
    expect(se.severity).toBe("notable");
    expect(se.stream).toBe("urgent");
    expect(se.category).toBe("economy");
  });
});
```

Run `cd src/frontend && npx vitest run src/lib/__tests__/eventClassifier.test.ts` —
expected failure: `expected 'informational' to be 'important'` and
`expected 'system' to be 'economy'`. Then in
`src/frontend/src/lib/eventClassifier.ts` add to `EVENT_SEVERITY_MAP`'s important
tier (after `crisis_phase_transition: "important",`):

```typescript
  // spec-116 4d.7: P23's MARKET_CORRECTION promoted to a discrete wire event
  // (the aggregate market_corrections timeseries counter is unchanged).
  market_correction: "important",
```

and to `CATEGORY_MAP`'s economy block (after `reserve_army_pressure: "economy",`):

```typescript
  market_correction: "economy",
```

Re-run vitest — expected: all pass. Then `cd src/frontend && npm run check` (tsc +
eslint + prettier + vitest) — expected green.

- [ ] **Step 8: Ceremony #2 (declared) — regression baselines.** Widening the converter changes
      `WorldState.events` content wherever a newly whitelisted event fires (starvation
      scenario: ENTITY_DEATH/POPULATION_ATTRITION are near-certain; glut /
      fascist_bifurcation: MARKET_CORRECTION and the edge-transition family are
      plausible). Run `mise run qa:regression` — expected `REGRESSION DETECTED!`
      naming the drifted scenarios. This is the declared intentional move (Market
      Scissors promotion pattern): regenerate with `mise run qa:regression-generate`
      and `mise run qa:regression-generate-dense`, re-run `mise run qa:regression` —
      expected `Results: 5 passed, 0 failed`. Record the per-scenario drift list
      (which scenarios' baselines moved, which stayed byte-identical) for the commit
      body. If compare unexpectedly comes back clean (events not represented in the
      trace bytes), skip regeneration and declare byte-identical instead — either way
      the outcome is stated in the commit message.

- [ ] **Step 9: Gate + commit.** `mise run check` green (includes `check:seams`:
      severity gate passes, coverage advisory now reports 19/83 dropped — advisory,
      non-gating). Then:

```
git add src/babylon/models/events/spine_payloads.py \
        src/babylon/engine/simulation_engine.py \
        web/game/engine_bridge.py \
        src/frontend/src/lib/eventClassifier.ts \
        tests/unit/engine/test_event_conversion.py \
        tests/unit/web/test_engine_bridge.py \
        src/frontend/src/lib/__tests__/eventClassifier.test.ts \
        tests/baselines
git commit  # via: mise run commit -- "feat(events): spec-116 4d.7 — whitelist sweep, 14 live-emitter EventTypes on the wire (50->64/83)"
# Commit body MUST declare: baselines regenerated (per-scenario drift list from
# Step 8), or byte-identical if no scenario fired a newly whitelisted event.
```
### Task 24: Verb dead-ends → disabled-with-reason (FR-116-4.8)

**Files:**

- Create: `web/game/verb_copy.py` (reason/remedy copy table — stdlib-only so BOTH bridges import it)
- Modify: `web/game/engine_bridge.py` (import block :45 area; insert `get_verb_eligibility` after `get_available_actions`, i.e. between :4849 and `submit_action` :4851)
- Modify: `web/game/stub_bridge.py` (import block :22; insert stub twin after `get_org_status` :1545, before the verb-target block :1547)
- Modify: `web/game/api.py` (insert `verb_eligibility` view after `actions_available` :1249-1263)
- Modify: `web/game/urls.py` (insert route after `actions-available` :146-150)
- Modify: `src/frontend/src/types/game.ts` (insert after `ActionPreviewResult` :710-717)
- Modify: `src/frontend/src/api/endpoints.ts` (type import list :27-48; row next to `actionsAvailable` :188)
- Create: `src/frontend/src/lib/verbs/fetchVerbEligibility.ts`
- Modify: `src/frontend/src/lib/verbs/index.ts` (:1-10, add two export lines)
- Create: `src/frontend/src/components/action/useVerbEligibility.ts`
- Modify: `src/frontend/src/components/action/VerbGrid.tsx` (full rewrite, :1-51)
- Modify: `src/frontend/src/components/action/ActionComposer.tsx` (:12-18 imports, :36, :57-71)
- Modify: `src/frontend/src/components/action/VerbForm.tsx` (:9-29 props, :63-98)
- Modify: `src/frontend/src/components/action/TargetPicker.tsx` (full rewrite, :1-55)
- Modify: `src/frontend/src/test/handlers.ts` (insert default handler next to the verb-targets handler ~:338)
- Test: `tests/unit/web/test_engine_bridge.py` (import block :19-32; new classes after `TestDefixturedQueryCorrectness` ~:1801)
- Test: `tests/unit/web/test_per_verb_views.py` (new classes after `TestPerVerbURLRouting` :37-59)
- Test: `tests/unit/web/test_stub_bridge_parity.py` (new smoke class at end, after :300)
- Test: `src/frontend/src/components/action/ActionComposer.test.tsx` (:44-51 deliberate update + new cases)
- Test: `src/frontend/src/components/action/VerbForm.test.tsx` (new empty-state case)
- Test: `src/frontend/e2e/verb-submit.spec.ts` (:33-53 deliberate replacement — already registered in `AUTHENTICATED_SPECS`, no `playwright.config.ts` change)

**Interfaces:**

- Consumes (all pre-existing, no dependency on Cluster A):
  - `VERB_TO_ACTION_TYPE: dict[str, ActionType]` / `CANONICAL_VERBS` (engine_bridge.py:148-160)
  - `check_can_afford(resources: VanguardResources, verb: str) -> tuple[bool, str]` and `VanguardResources.from_organization(...)` (already imported in engine_bridge.py:45)
  - `_envelope(data, tick=None, session_id=None, http_status=200) -> JsonResponse`, `_error`, `_get_session_or_none`, `_get_bridge` (api.py)
  - `_patched_hydrate_state(bridge, graph, tick)` + `_make_mock_persistence()` (test_engine_bridge.py:36, :1431)
  - `get as apiGet` (`@/api/client`), `endpoints`/`ep<T>` manifest (`@/api/endpoints`)
- Produces (later tracks may rely on these exact names):
  - `VERB_INELIGIBILITY_COPY: dict[str, tuple[str, str]]` (web/game/verb_copy.py)
  - `EngineBridge.get_verb_eligibility(session_id: UUID, org_id: str) -> dict[str, Any]` returning `{"session_id": str, "tick": int, "org_id": str, "verbs": [{"verb", "eligible", "reason", "remedy", "can_afford", "afford_note"}]}` (or `{"status": "error", "error": "Org not found"}`); identical shape on `StubEngineBridge`
  - `GET /api/games/{id}/actions/eligibility/?org_id=` (url name `game:actions-eligibility`), standard `{status, data}` envelope
  - TS `VerbEligibilityEntry` / `VerbEligibilityPayload` (types/game.ts), `endpoints.verbEligibility`
  - `fetchVerbEligibility(gameId, orgId)` (`@/lib/verbs`), `useVerbEligibility(gameId, orgId, tick) -> VerbEligibilityMap | null` and `VerbEligibilityMap = Record<string, VerbEligibilityEntry>`
  - `VerbGridProps.eligibility: VerbEligibilityMap | null`; `data-testid="verb-ineligible-reasons"`; `data-testid="targets-empty"`

**Determinism:** web-tier only (`web/game/**`, `src/frontend/**`, tests). No engine, defines, or economics change — byte-safe by construction; `qa:regression` stays byte-identical with **no** baseline regeneration.

**Seam registry (verified, not assumed):** the new bridge keys (`verbs`, `eligible`, `reason`, `remedy`, `can_afford`, `afford_note`) need **no `SeamEntry` rows**. `check:seams`' gating tier is exactly `check_map_metrics` + `check_tick_payloads_exist` + `check_severity_vocabulary` (`src/babylon/sentinels/seam/checks.py:227-232`); the action-verb surface has no `SeamScope` member (types.py:85-93), and all nine existing verb-target payloads ride unregistered today with a green gate. The advisory Sensor-3 sweep (`check_bridge_serialization`) is satisfied cleanly by the **typed** `endpoints.ts` row: `_interface_fields` finds `VerbEligibilityPayload` in `types/game.ts` and its four fields (`session_id`, `tick`, `org_id`, `verbs`) are a subset of the serializer's returned dict keys, so no phantom finding is added. Misfiling rows under `INSPECTOR`/`NETWORK` would repeat the exact misclassification the `NETWORK` scope docstring warns against — if the lead wants an `ACTION` scope, that is a separate registry decision, not this task.

**Honesty rulings baked into this task (from the recon brief's gotchas):**

1. EDUCATE is structurally ineligible in every real scenario at every tick (`SocialClass` has no `territory_ids` field; `to_graph` dumps model fields verbatim — world_state.py:692, engine_bridge.py:697-706). The spec's example remedy "organize a community first" would promise an action that does not exist (CLAUDE.md Verifiability rule). We ship the honest reason and an honest remedy that names the gap. Copy referencing MOVE is verifiable: `resolve_move` genuinely mutates `territory_ids` (src/babylon/engine/actions/move.py:62-74).
2. MOVE/NEGOTIATE target lists and INVESTIGATE's targeted scans are hardcoded fixtures (engine_bridge.py:5545-5567, 5599-5631, 5480-5500). Their **eligibility** derives from real predicates (any territory node exists / any other org exists / own territories non-empty); the fixture target lists remain pre-existing debt, explicitly NOT laundered into `eligible: true`-by-fixture. De-fixturing those three endpoints is out of scope and flagged for the owner.
3. One shared hydration: the naive 9× `get_*_targets` loop would cost ~18 `hydrate_state` passes (each targets call hydrates twice); `get_verb_eligibility` does exactly one, pinned by test.
4. Honest-null frontend rule: while the eligibility fetch is unresolved or failed, every verb renders enabled exactly as today — a fabricated disabled state is a phantom (Constitution III.11). `DISABLED_VERBS` (static "no engine handler" gate) stays a separate mechanism.

---

- [ ] **Step 1: Create the copy table + write the failing bridge tests**

Create `web/game/verb_copy.py` (pure data — creating it before the red run keeps test collection clean; the failing *behavior* is the bridge method):

```python
"""Player-facing ineligibility copy for the nine verbs (spec-116 FR-4.8).

One ``(reason, remedy)`` pair per canonical verb, shown when the verb's
target-existence predicate is empty — the SAME predicates that yield the
per-verb empty target lists (``EngineBridge.get_verb_eligibility``).

Kept in its own stdlib-only module so BOTH bridges (``engine_bridge`` and
``stub_bridge``) read one table and can never disagree on player-facing
text (``stub_bridge`` may not import engine modules — see
``tests/unit/web/test_import_boundary.py``). Copy is prose, not a
coefficient: it deliberately does NOT live in ``GameDefines``.

Verifiability (CLAUDE.md): every remedy names only capabilities that
exist. MOVE genuinely mutates ``territory_ids``
(``babylon/engine/actions/move.py`` — relocate/expand). EDUCATE's remedy
promises no action, because none can create an organized community today:
``SocialClass`` has no ``territory_ids`` field, so no social_class node
ever matches ``_nodes_in_territory`` at runtime — the honest structural
gap the educate payload's own notes name.
"""

from __future__ import annotations

#: verb -> (reason, remedy); read only when ``eligible`` is False.
VERB_INELIGIBILITY_COPY: dict[str, tuple[str, str]] = {
    "educate": (
        "No organized community in your territories yet.",
        "No action can organize a community yet — political education "
        "unlocks the moment an organized class appears where you operate.",
    ),
    "aid": (
        "No community or organization within your territories to aid.",
        "Expand your presence into a populated territory first (MOVE).",
    ),
    "attack": (
        "No hostile organization or institution within your reach.",
        "Expand your presence (MOVE), or wait for state forces to enter "
        "your territories.",
    ),
    "mobilize": (
        "No business or civil-society organization within your "
        "territories to mobilize against.",
        "Expand toward workplaces and civil society (MOVE), or wait for "
        "new organizations to emerge.",
    ),
    "campaign": (
        "No territories exist in this session yet.",
        "Campaign targets appear once the scenario map is seeded.",
    ),
    "move": (
        "No territory exists to relocate into.",
        "Territories appear once the scenario map is seeded.",
    ),
    "investigate": (
        "Your organization has no territorial presence to scan from.",
        "Establish presence in a territory first (MOVE).",
    ),
    "reproduce": (
        "Your organization is not present in the world graph.",
        "Reload the session — this indicates a corrupted game state.",
    ),
    "negotiate": (
        "No other organization exists to negotiate with.",
        "Rival and allied organizations emerge as the simulation unfolds.",
    ),
}
```

In `tests/unit/web/test_engine_bridge.py`, add `VERB_TO_ACTION_TYPE,` to the existing `from game.engine_bridge import (...)` block (:19-32) and, below it, `from game.verb_copy import VERB_INELIGIBILITY_COPY`. Then append after `TestDefixturedQueryCorrectness` (~:1801):

```python
def _make_wayne_tick0_graph() -> BabylonGraph:
    """Tick-0 wayne_county-shaped graph for eligibility tests.

    ORG001 (civil_society, the player) and ORG002 (state_apparatus,
    Detroit PD) share hex-1/hex-2, mirroring ``_legacy_wayne.py``. The
    social_class node is production-faithful: NO ``territory_ids``
    (``SocialClass`` has no such field; ``to_graph`` dumps model fields
    verbatim), so EDUCATE is structurally ineligible — the exact tick-0
    dead-end FR-116-4.8 converts into disabled-with-reason.
    """
    g = BabylonGraph()
    g.graph["tick"] = 0
    g.add_node(
        "ORG001",
        "organization",
        name="Wayne County Organizing Committee",
        org_type="civil_society",
        cadre_level=0.1,
        cohesion=0.5,
        budget=100.0,
        heat=0.0,
        territory_ids=["hex-1", "hex-2"],
    )
    g.add_node(
        "ORG002",
        "organization",
        name="Detroit Police Department",
        org_type="state_apparatus",
        cadre_level=5.0,
        cohesion=0.8,
        budget=5000.0,
        heat=0.3,
        territory_ids=["hex-1", "hex-2", "hex-3"],
    )
    g.add_node("hex-1", "territory", name="Downtown Detroit", heat=0.2, population=5000)
    g.add_node("hex-2", "territory", name="Dearborn", heat=0.1, population=4000)
    g.add_node(
        "C001",
        "social_class",
        name="Detroit Proletariat",
        role="proletariat",
        wealth=15.0,
        agitation=0.2,
    )
    return g


@pytest.mark.unit
class TestVerbEligibility:
    """Spec-116 FR-4.8: get_verb_eligibility derives per-verb eligibility
    from the SAME predicates that empty the per-verb target lists, on ONE
    shared hydration, with the server-side reason/remedy copy table."""

    def _eligibility(self, graph: BabylonGraph) -> tuple[dict[str, Any], MagicMock]:
        mock_persistence = _make_mock_persistence()
        bridge = EngineBridge(mock_persistence)
        with _patched_hydrate_state(bridge, graph, tick=0) as mock_hydrate:
            result = bridge.get_verb_eligibility(uuid.uuid4(), "ORG001")
        return result, mock_hydrate

    def test_all_nine_verbs_present_with_full_row_shape(self) -> None:
        result, _ = self._eligibility(_make_wayne_tick0_graph())
        assert result["org_id"] == "ORG001"
        assert result["tick"] == 0
        assert {v["verb"] for v in result["verbs"]} == set(VERB_TO_ACTION_TYPE)
        for row in result["verbs"]:
            assert set(row) == {
                "verb",
                "eligible",
                "reason",
                "remedy",
                "can_afford",
                "afford_note",
            }

    def test_tick0_wayne_educate_ineligible_with_reason_and_remedy(self) -> None:
        result, _ = self._eligibility(_make_wayne_tick0_graph())
        by_verb = {v["verb"]: v for v in result["verbs"]}
        assert by_verb["educate"]["eligible"] is False
        assert (
            by_verb["educate"]["reason"]
            == "No organized community in your territories yet."
        )
        assert by_verb["educate"]["remedy"] == VERB_INELIGIBILITY_COPY["educate"][1]

    def test_tick0_wayne_mobilize_ineligible_state_apparatus_does_not_count(self) -> None:
        result, _ = self._eligibility(_make_wayne_tick0_graph())
        by_verb = {v["verb"]: v for v in result["verbs"]}
        assert by_verb["mobilize"]["eligible"] is False
        assert by_verb["mobilize"]["reason"] == VERB_INELIGIBILITY_COPY["mobilize"][0]
        assert by_verb["mobilize"]["remedy"] == VERB_INELIGIBILITY_COPY["mobilize"][1]

    def test_tick0_wayne_eligible_verbs_carry_null_copy(self) -> None:
        result, _ = self._eligibility(_make_wayne_tick0_graph())
        by_verb = {v["verb"]: v for v in result["verbs"]}
        # ORG002 shares hex-1/hex-2 -> aid (org arm), attack, negotiate.
        # Territory nodes exist -> campaign, move. Own territories -> investigate.
        for verb in ("aid", "attack", "negotiate", "campaign", "move", "investigate", "reproduce"):
            assert by_verb[verb]["eligible"] is True, verb
            assert by_verb[verb]["reason"] is None, verb
            assert by_verb[verb]["remedy"] is None, verb

    def test_educate_flips_eligible_when_social_class_shares_territory(self) -> None:
        graph = _make_wayne_tick0_graph()
        graph.update_node("C001", territory_ids=["hex-1"])
        result, _ = self._eligibility(graph)
        by_verb = {v["verb"]: v for v in result["verbs"]}
        assert by_verb["educate"]["eligible"] is True
        assert by_verb["educate"]["reason"] is None

    def test_exactly_one_hydration(self) -> None:
        _result, mock_hydrate = self._eligibility(_make_wayne_tick0_graph())
        assert mock_hydrate.call_count == 1, (
            "get_verb_eligibility must evaluate all nine predicates on ONE "
            "shared hydration (the 9x get_*_targets loop would cost ~18)"
        )

    def test_org_not_found_is_honest_error(self) -> None:
        mock_persistence = _make_mock_persistence()
        bridge = EngineBridge(mock_persistence)
        with _patched_hydrate_state(bridge, _make_wayne_tick0_graph(), tick=0):
            result = bridge.get_verb_eligibility(uuid.uuid4(), "NOT-AN-ORG")
        assert result == {"status": "error", "error": "Org not found"}
```

- [ ] **Step 2: Run the bridge tests to verify they fail**

```bash
mise run test:q -- tests/unit/web/test_engine_bridge.py -k TestVerbEligibility
```

Expected: `7 failed`, each with `AttributeError: 'EngineBridge' object has no attribute 'get_verb_eligibility'`.

- [ ] **Step 3: Implement `EngineBridge.get_verb_eligibility`; bridge tests green, parity red**

In `web/game/engine_bridge.py`, add to the import section (near :45, after the `babylon.models.vanguard_resources` import):

```python
from .verb_copy import VERB_INELIGIBILITY_COPY
```

Insert after `get_available_actions` (between :4849 and `submit_action`):

```python
    def get_verb_eligibility(self, session_id: UUID, org_id: str) -> dict[str, Any]:
        """Per-verb eligibility for the acting org (spec-116 FR-4.8).

        Evaluates, on ONE shared hydration, the same target-existence
        predicate each ``get_<verb>_targets`` method applies — so a verb
        is marked ineligible exactly when its target list would come back
        empty — and pairs every ineligible verb with the player-facing
        ``(reason, remedy)`` copy from :mod:`game.verb_copy`.
        Affordability rides along per verb via :func:`check_can_afford`
        (the same function that gates ``submit_action``, so the note can
        never disagree with a submit rejection); the UI disables on
        ``eligible`` only, never on ``can_afford``.

        MOVE/NEGOTIATE/INVESTIGATE honesty note: their eligibility comes
        from real graph predicates (a territory node exists / another org
        exists / own territories non-empty) even though their target
        lists are partly hardcoded fixtures today — eligibility must
        never launder a fixture into ``eligible: true``.

        :param session_id: The game session UUID.
        :param org_id: The acting organization id.
        :returns: ``{"session_id", "tick", "org_id", "verbs"}`` where
            ``verbs`` holds one ``{"verb", "eligible", "reason",
            "remedy", "can_afford", "afford_note"}`` dict per canonical
            verb in ``VERB_TO_ACTION_TYPE`` order, or
            ``{"status": "error", "error": "Org not found"}``.
        """
        state, graph = self.hydrate_state(session_id)
        if org_id not in graph.nodes:
            return {"status": "error", "error": "Org not found"}

        org_data = graph.nodes[org_id]
        own_tids = {str(t) for t in org_data.get("territory_ids", [])}

        # One bounded pass over the graph gathers every predicate input.
        has_social_class = False  # educate; aid (population arm)
        has_org_in_reach = False  # aid (org arm); attack (org arm)
        has_mobilizable_org = False  # mobilize (business/civil_society)
        has_institution_in_reach = False  # attack (institution arm)
        has_other_org = False  # negotiate (anywhere in the graph)
        has_territory_node = False  # campaign; move
        for node_id, data in graph.nodes(data=True):
            node_type = data.get("_node_type")
            if node_type == "territory":
                has_territory_node = True
                continue
            node_tids = {str(t) for t in data.get("territory_ids", [])}
            if node_type == "organization" and node_id != org_id:
                has_other_org = True
                if node_tids & own_tids:
                    has_org_in_reach = True
                    if str(data.get("org_type", "")) in ("business", "civil_society"):
                        has_mobilizable_org = True
            elif node_type == "social_class" and node_tids & own_tids:
                has_social_class = True
            elif node_type == "institution" and node_tids & own_tids:
                has_institution_in_reach = True

        eligible_by_verb: dict[str, bool] = {
            "educate": has_social_class,
            "aid": has_social_class or has_org_in_reach,
            "attack": has_org_in_reach or has_institution_in_reach,
            "mobilize": has_mobilizable_org,
            "campaign": has_territory_node,
            "move": has_territory_node,
            "investigate": bool(own_tids),
            "reproduce": True,  # always targets the acting org itself
            "negotiate": has_other_org,
        }

        resources = VanguardResources.from_organization(
            cadre_level=float(org_data.get("cadre_level", 0.0)),
            cohesion=float(org_data.get("cohesion", 0.0)),
            budget=float(org_data.get("budget", 0.0)),
            heat=float(org_data.get("heat", 0.0)),
            territory_count=len(own_tids),
        )

        verbs: list[dict[str, Any]] = []
        for verb in VERB_TO_ACTION_TYPE:
            eligible = eligible_by_verb[verb]
            reason, remedy = (None, None) if eligible else VERB_INELIGIBILITY_COPY[verb]
            can_afford, afford_reason = check_can_afford(resources, verb)
            verbs.append(
                {
                    "verb": verb,
                    "eligible": eligible,
                    "reason": reason,
                    "remedy": remedy,
                    "can_afford": can_afford,
                    "afford_note": None if can_afford else afford_reason,
                }
            )

        return {
            "session_id": str(session_id),
            "tick": state.tick,
            "org_id": org_id,
            "verbs": verbs,
        }
```

Verify green, then verify the parity guard reds (the next honest failure):

```bash
mise run test:q -- tests/unit/web/test_engine_bridge.py -k TestVerbEligibility
mise run test:q -- tests/unit/web/test_stub_bridge_parity.py
```

Expected: first command `7 passed`; second command `1 failed` — `test_every_engine_bridge_get_method_exists_on_stub` with `StubEngineBridge is missing get_* methods: ['get_verb_eligibility']`.

- [ ] **Step 4: Stub twin + stub smoke tests; parity green**

In `web/game/stub_bridge.py`, extend the module import (:22) to:

```python
from .map_contract import MAP_HISTORY_REPLAYABLE_METRICS, MAP_METRIC_PROPERTIES
from .verb_copy import VERB_INELIGIBILITY_COPY
```

Insert after `get_org_status` (:1545), before the verb-target block:

```python
    def get_verb_eligibility(self, session_id: UUID, org_id: str) -> dict[str, Any]:
        """Per-verb eligibility (spec-116 FR-4.8) mirroring the real
        bridge's exact row shape, derived honestly from the stub's own
        mock world: ORG001 is the ONLY organization, so MOBILIZE (needs
        another business/civil-society org — matching
        ``get_mobilize_targets``' honest empty list) and NEGOTIATE (needs
        any other org) are ineligible; every other verb's stub target
        list is non-empty. Copy comes from the shared ``verb_copy`` table
        so stub and engine can never disagree on player-facing text.
        """
        org_status = self.get_org_status(session_id, org_id)
        if not org_status:
            return {"status": "error", "error": "Org not found"}
        session = _stub_sessions.get(session_id, {"tick": 0})
        resources = org_status["resources"]

        # ACTION_COSTS literal snapshot (babylon/models/vanguard_resources
        # .py:139-152) — this module cannot import babylon.models
        # (import-boundary guard, tests/unit/web/test_import_boundary.py);
        # same documented pattern as get_mobilize_targets' mobilize_cost_cl.
        costs: dict[str, tuple[float, float, float]] = {
            "educate": (2.0, 0.5, 5.0),
            "reproduce": (1.5, 1.0, 10.0),
            "attack": (1.0, 3.0, 15.0),
            "mobilize": (0.5, 4.0, 8.0),
            "campaign": (1.0, 3.0, 20.0),
            "aid": (0.5, 1.0, 25.0),
            "investigate": (3.0, 0.0, 2.0),
            "move": (1.0, 0.5, 5.0),
            "negotiate": (2.0, 0.0, 10.0),
        }
        eligible_by_verb: dict[str, bool] = {
            "educate": True,  # get_educate_targets: C001/C004
            "reproduce": True,  # the acting org itself
            "attack": True,  # get_attack_targets: INST001
            "mobilize": False,  # honest empty list — only ORG001 exists
            "campaign": True,  # T001-T004 territories
            "aid": True,  # population targets C001/C004
            "investigate": True,  # ORG001 territory scans
            "move": True,  # territories exist
            "negotiate": False,  # no other organization in the stub world
        }

        verbs: list[dict[str, Any]] = []
        for verb in (
            "educate",
            "reproduce",
            "attack",
            "mobilize",
            "campaign",
            "aid",
            "investigate",
            "move",
            "negotiate",
        ):
            cl_cost, sl_cost, budget_cost = costs[verb]
            eligible = eligible_by_verb[verb]
            reason, remedy = (None, None) if eligible else VERB_INELIGIBILITY_COPY[verb]
            cl = float(resources["cadre_labor"])
            sl = float(resources["sympathizer_labor"])
            budget = float(resources["material"])
            if cl < cl_cost:
                can_afford, afford_note = False, f"Need {cl_cost} CL, have {cl:.1f}"
            elif sl < sl_cost:
                can_afford, afford_note = False, f"Need {sl_cost} SL, have {sl:.1f}"
            elif budget < budget_cost:
                can_afford, afford_note = False, f"Need ${budget_cost}, have ${budget:.1f}"
            else:
                can_afford, afford_note = True, None
            verbs.append(
                {
                    "verb": verb,
                    "eligible": eligible,
                    "reason": reason,
                    "remedy": remedy,
                    "can_afford": can_afford,
                    "afford_note": afford_note,
                }
            )

        return {
            "session_id": str(session_id),
            "tick": session.get("tick", 0),
            "org_id": org_id,
            "verbs": verbs,
        }
```

Append to `tests/unit/web/test_stub_bridge_parity.py` (after :300):

```python
class TestVerbEligibilityStub:
    """Spec-116 FR-4.8 twin: coherent with the stub's own target methods
    (mobilize honestly empty; negotiate ineligible because ORG001 is the
    stub world's only organization)."""

    def test_shape_and_world_coherence(self) -> None:
        bridge, session_id = _stub_session()
        result = bridge.get_verb_eligibility(session_id, _ORG_ID)
        assert len(result["verbs"]) == 9
        by_verb = {v["verb"]: v for v in result["verbs"]}
        assert by_verb["mobilize"]["eligible"] is False
        assert by_verb["mobilize"]["reason"]
        assert by_verb["mobilize"]["remedy"]
        assert by_verb["negotiate"]["eligible"] is False
        assert by_verb["educate"]["eligible"] is True
        assert by_verb["educate"]["reason"] is None
        for row in result["verbs"]:
            assert set(row) == {
                "verb",
                "eligible",
                "reason",
                "remedy",
                "can_afford",
                "afford_note",
            }

    def test_unknown_org_is_honest_error(self) -> None:
        bridge, session_id = _stub_session()
        assert bridge.get_verb_eligibility(session_id, "NOT-AN-ORG") == {
            "status": "error",
            "error": "Org not found",
        }
```

```bash
mise run test:q -- tests/unit/web/test_stub_bridge_parity.py
```

Expected: all passed (existence + signature + 2 new smoke tests), 0 failed.

- [ ] **Step 5: Write the failing view/routing tests**

Append to `tests/unit/web/test_per_verb_views.py`:

```python
@pytest.mark.unit
class TestVerbEligibilityRouting:
    """Spec-116 FR-4.8: the aggregate eligibility endpoint resolves."""

    def test_eligibility_url_resolves(self) -> None:
        url = reverse("game:actions-eligibility", kwargs={"game_id": GAME_ID})
        assert "/actions/eligibility/" in url


@pytest.mark.unit
@pytest.mark.django_db
class TestVerbEligibilityView:
    """GET /api/games/{id}/actions/eligibility/ (spec-116 FR-4.8)."""

    _PAYLOAD = {
        "session_id": GAME_ID,
        "tick": 0,
        "org_id": "ORG001",
        "verbs": [
            {
                "verb": "educate",
                "eligible": False,
                "reason": "No organized community in your territories yet.",
                "remedy": (
                    "No action can organize a community yet — political "
                    "education unlocks the moment an organized class "
                    "appears where you operate."
                ),
                "can_afford": False,
                "afford_note": "Need 2.0 CL, have 0.3",
            }
        ],
    }

    def _setup(self) -> tuple:
        from unittest.mock import MagicMock

        from django.contrib.auth.models import User

        from game.models import GameSession

        user = User.objects.create_user(  # type: ignore[no-untyped-call]
            username=f"elig_user_{uuid_mod.uuid4().hex[:8]}",
            password="testpass123",
        )
        client = Client()
        client.login(username=user.username, password="testpass123")

        session = GameSession.objects.create(
            id=uuid_mod.uuid4(),
            player_id=user.id,
            scenario="wayne_county",
            current_tick=0,
            status="active",
        )

        mock_bridge = MagicMock()
        mock_bridge.get_verb_eligibility.return_value = self._PAYLOAD

        import game.api

        game.api._bridge_instance = mock_bridge
        return client, session, mock_bridge

    def test_get_requires_org_id(self) -> None:
        client, session, _mock = self._setup()
        url = reverse("game:actions-eligibility", kwargs={"game_id": str(session.id)})
        response = client.get(url)
        assert response.status_code == 400

    def test_get_returns_envelope_with_verb_rows(self) -> None:
        client, session, mock_bridge = self._setup()
        url = reverse("game:actions-eligibility", kwargs={"game_id": str(session.id)})
        response = client.get(url, {"org_id": "ORG001"})
        assert response.status_code == 200
        body = json.loads(response.content)
        assert body["status"] == "ok"
        assert body["data"]["verbs"][0]["verb"] == "educate"
        assert body["data"]["verbs"][0]["eligible"] is False
        assert body["data"]["verbs"][0]["remedy"]
        mock_bridge.get_verb_eligibility.assert_called_once()

    def test_get_unknown_game_404s(self) -> None:
        client, _session, _mock = self._setup()
        url = reverse(
            "game:actions-eligibility", kwargs={"game_id": str(uuid_mod.uuid4())}
        )
        response = client.get(url, {"org_id": "ORG001"})
        assert response.status_code == 404

    def test_bridge_error_maps_to_400(self) -> None:
        client, session, mock_bridge = self._setup()
        mock_bridge.get_verb_eligibility.return_value = {
            "status": "error",
            "error": "Org not found",
        }
        url = reverse("game:actions-eligibility", kwargs={"game_id": str(session.id)})
        response = client.get(url, {"org_id": "NOPE"})
        assert response.status_code == 400
```

```bash
mise run test:q -- tests/unit/web/test_per_verb_views.py -k VerbEligibility
```

Expected: `5 failed`, each with `django.urls.exceptions.NoReverseMatch: Reverse for 'actions-eligibility' not found`.

- [ ] **Step 6: Implement view + route; green; commit the backend half**

In `web/game/api.py`, insert directly after `actions_available` (:1263):

```python
@api_view(["GET"])
@permission_classes([IsAuthenticated])
def verb_eligibility(request: Request, game_id: str) -> JsonResponse:
    """GET /api/games/{id}/actions/eligibility/ — per-verb eligibility.

    Spec-116 FR-4.8: one row per canonical verb — ``{verb, eligible,
    reason, remedy, can_afford, afford_note}`` — derived from the SAME
    predicates that yield the per-verb empty target lists, so the
    VerbGrid can disable dead-end verbs with the reason and remedy
    visible instead of offering a click into "No eligible targets."
    Requires ``?org_id=`` like the per-verb target GETs.
    """
    session = _get_session_or_none(game_id, request.user.id)
    if session is None:
        return _error("Game not found", http_status=404)

    org_id = request.query_params.get("org_id")
    if not org_id:
        return _error("org_id query parameter is required")

    bridge = _get_bridge()
    data = bridge.get_verb_eligibility(uuid.UUID(str(session.id)), org_id)
    if data.get("status") == "error":
        return _error(str(data.get("error", "Verb eligibility unavailable")))

    return _envelope(data, tick=session.current_tick, session_id=str(session.id))
```

In `web/game/urls.py`, insert after the `actions-available` entry (:150):

```python
    # API: Spec-116 FR-4.8 — per-verb eligibility (VerbGrid disabled-with-reason)
    path(
        "games/<str:game_id>/actions/eligibility/",
        api.verb_eligibility,
        name="actions-eligibility",
    ),
```

(Exact-match `path()` routing — no collision with `actions/<int:action_id>/`; placed with the other `actions/` utility endpoints for readability.)

```bash
mise run test:q -- tests/unit/web/test_per_verb_views.py tests/unit/web/test_engine_bridge.py -k "VerbEligibility"
mise run test:q -- tests/unit/web/test_import_boundary.py tests/unit/web/test_stub_bridge_parity.py
mise run commit -- "feat(web): per-verb eligibility surface — get_verb_eligibility + /actions/eligibility/ (spec-116 FR-4.8)"
```

Expected: all selected tests pass (12 eligibility tests + import-boundary + parity); commit lands with the standard `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` trailer (hooks run on staged files).

- [ ] **Step 7: Write the failing frontend unit tests**

In `src/frontend/src/components/action/ActionComposer.test.tsx`: replace the test at :44-51 (deliberate update — its "every verb enabled" claim becomes the honest-null contract) and add two new tests. The fixture rows are plain object literals (no type import — the TS types land in Step 9; vitest transpiles without typechecking, so the red run stays clean):

```tsx
  it("renders the flat 9-verb grid all-enabled while eligibility is unresolved (honest-null, spec-116 FR-4.8)", async () => {
    seedPlayerOrg();
    render(<ActionComposer gameId={DEFAULT_GAME_ID} />);
    const grid = screen.getByTestId("verb-grid");
    expect(grid.querySelectorAll("button")).toHaveLength(9);
    expect(screen.getByRole("button", { name: /investigate/i })).toBeEnabled();
    // Settle the eligibility fetch (default handler: empty verbs list).
    await waitFor(() => expect(screen.getByRole("button", { name: /educate/i })).toBeEnabled());
    expect(screen.queryByTestId("verb-ineligible-reasons")).not.toBeInTheDocument();
  });

  it("disables an ineligible verb with reason + remedy visible (spec-116 FR-4.8)", async () => {
    server.use(
      http.get("/api/games/:id/actions/eligibility/", () =>
        HttpResponse.json({
          status: "ok",
          data: {
            session_id: DEFAULT_GAME_ID,
            tick: 1,
            org_id: "org-1",
            verbs: [
              {
                verb: "educate",
                eligible: false,
                reason: "No organized community in your territories yet.",
                remedy:
                  "No action can organize a community yet — political education unlocks the moment an organized class appears where you operate.",
                can_afford: true,
                afford_note: null,
              },
              {
                verb: "mobilize",
                eligible: false,
                reason:
                  "No business or civil-society organization within your territories to mobilize against.",
                remedy:
                  "Expand toward workplaces and civil society (MOVE), or wait for new organizations to emerge.",
                can_afford: true,
                afford_note: null,
              },
              {
                verb: "attack",
                eligible: true,
                reason: null,
                remedy: null,
                can_afford: true,
                afford_note: null,
              },
            ],
          },
        }),
      ),
    );
    seedPlayerOrg();
    render(<ActionComposer gameId={DEFAULT_GAME_ID} />);

    const educate = screen.getByRole("button", { name: /educate/i });
    await waitFor(() => expect(educate).toBeDisabled());
    expect(educate).toHaveAttribute(
      "title",
      expect.stringContaining("no eligible targets yet: No organized community"),
    );
    expect(screen.getByRole("button", { name: /mobilize/i })).toBeDisabled();

    // Reason + remedy VISIBLE (not tooltip-only), per FR-116-4.8.
    const reasons = screen.getByTestId("verb-ineligible-reasons");
    expect(reasons).toHaveTextContent(/educate/i);
    expect(reasons).toHaveTextContent("No organized community in your territories yet.");
    expect(reasons).toHaveTextContent(/political education unlocks/);

    // Article V: eligible verbs stay enabled; nothing is ever hidden.
    expect(screen.getByRole("button", { name: /attack/i })).toBeEnabled();
    expect(screen.getByTestId("verb-grid").querySelectorAll("button")).toHaveLength(9);
  });

  it("keeps all verbs enabled when the eligibility fetch fails (honest-null)", async () => {
    server.use(
      http.get("/api/games/:id/actions/eligibility/", () =>
        HttpResponse.json({ status: "error", message: "boom" }, { status: 500 }),
      ),
    );
    seedPlayerOrg();
    render(<ActionComposer gameId={DEFAULT_GAME_ID} />);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /educate/i })).toBeEnabled(),
    );
    expect(screen.queryByTestId("verb-ineligible-reasons")).not.toBeInTheDocument();
  });
```

In `src/frontend/src/components/action/VerbForm.test.tsx`, add (uses the file's existing `makeConfig`, `makeSnapshot`, `DEFAULT_GAME_ID`, `server`, `vi` imports):

```tsx
  it("renders reason + remedy in the empty state when the verb is ineligible (spec-116 FR-4.8)", async () => {
    server.use(
      http.get("/api/games/:id/actions/educate/targets/", () =>
        HttpResponse.json({ targets: [] }),
      ),
    );
    render(
      <VerbForm
        gameId={DEFAULT_GAME_ID}
        orgId="org-1"
        verb="educate"
        config={makeConfig()}
        snapshot={makeSnapshot()}
        submitting={false}
        onSubmit={vi.fn()}
        eligibility={{
          verb: "educate",
          eligible: false,
          reason: "No organized community in your territories yet.",
          remedy:
            "No action can organize a community yet — political education unlocks the moment an organized class appears where you operate.",
          can_afford: true,
          afford_note: null,
        }}
      />,
    );
    const empty = await screen.findByTestId("targets-empty");
    expect(empty).toHaveTextContent(
      "No eligible targets yet: No organized community in your territories yet.",
    );
    expect(empty).toHaveTextContent(/political education unlocks/);
  });
```

- [ ] **Step 8: Run the frontend tests to verify they fail**

```bash
cd src/frontend && npx vitest run src/components/action/ActionComposer.test.tsx src/components/action/VerbForm.test.tsx
```

Expected failures: the disabled-with-reason test times out on `expect(educate).toBeDisabled()` (the component never fetches eligibility, so the button stays enabled); the VerbForm test fails with `Unable to find an element by: [data-testid="targets-empty"]`. The two honest-null tests may pass vacuously — that is fine; the disabled path is the red.

- [ ] **Step 9: Implement the frontend; vitest green**

(a) `src/frontend/src/types/game.ts` — insert after `ActionPreviewResult` (:717):

```ts
/** One verb's row from GET /actions/eligibility/ (spec-116 FR-4.8). */
export interface VerbEligibilityEntry {
  verb: string;
  eligible: boolean;
  /** Player-facing reason the verb has no eligible targets; null when eligible. */
  reason: string | null;
  /** What the player can do about it; null when eligible. */
  remedy: string | null;
  /** From the same check_can_afford that gates submit — advisory only;
   *  the UI never disables on affordability. */
  can_afford: boolean;
  afford_note: string | null;
}

/** Response payload of GET /api/games/{id}/actions/eligibility/. */
export interface VerbEligibilityPayload {
  session_id: string;
  tick: number;
  org_id: string;
  verbs: VerbEligibilityEntry[];
}
```

(b) `src/frontend/src/api/endpoints.ts` — add `VerbEligibilityPayload` to the `@/types/game` type import list (:27-48) and this row directly under `actionsAvailable` (:188):

```ts
  // Spec-116 FR-4.8: per-verb eligibility (VerbGrid disabled-with-reason).
  verbEligibility: ep<VerbEligibilityPayload>("/api/games/:id/actions/eligibility/"),
```

(Typed on purpose: the seam Sensor-3 sweep reads this row, resolves `VerbEligibilityPayload` under `types/`, and checks its four fields against `get_verb_eligibility`'s returned dict keys — all four are emitted, so the sweep stays clean with zero new advisory findings.)

(c) Create `src/frontend/src/lib/verbs/fetchVerbEligibility.ts`:

```ts
/**
 * fetchVerbEligibility — per-verb eligibility fetch, store-free (spec-116
 * FR-4.8). Unlike the per-verb target endpoints (flat bodies, see
 * `fetchVerbTargets`), `GET /actions/eligibility/` is a new surface and
 * uses the standard `{status, data}` envelope — same normalization shape
 * as `fetchActionPreview`.
 */

import { get as apiGet } from "@/api/client";
import { endpoints } from "@/api/endpoints";
import type { VerbEligibilityPayload } from "@/types/game";

export interface VerbEligibilityFetchResult {
  /** False when the request failed (network error or HTTP/API error). */
  ok: boolean;
  /** The eligibility payload, or null on failure. */
  data: VerbEligibilityPayload | null;
  /** Present when `ok` is false. */
  message?: string;
}

/**
 * Fetch per-verb eligibility from
 * ``/api/games/{gameId}/actions/eligibility/?org_id={orgId}``.
 *
 * :param gameId: Active game session id.
 * :param orgId: Acting organization id.
 * :returns: The normalized result — never throws.
 */
export async function fetchVerbEligibility(
  gameId: string,
  orgId: string,
): Promise<VerbEligibilityFetchResult> {
  const res = await apiGet<VerbEligibilityPayload>(
    `${endpoints.verbEligibility.path({ id: gameId })}?org_id=${orgId}`,
  );
  if (res.status === "error" || !res.data) {
    return {
      ok: false,
      data: null,
      message: res.message ?? "Failed to fetch verb eligibility",
    };
  }
  return { ok: true, data: res.data };
}
```

(d) `src/frontend/src/lib/verbs/index.ts` — append:

```ts
export { fetchVerbEligibility } from "./fetchVerbEligibility";
export type { VerbEligibilityFetchResult } from "./fetchVerbEligibility";
```

(e) Create `src/frontend/src/components/action/useVerbEligibility.ts`:

```ts
/**
 * Per-org verb eligibility for the VerbGrid (spec-116 FR-4.8) — one
 * fetch per (game, org, tick), mirroring `useVerbTargets`'s
 * cancel-on-unmount pattern; the tick dependency re-fetches after every
 * resolved tick so the grid tracks the moving world.
 *
 * HONEST-NULL: while the fetch is unresolved or failed the map is null
 * and every verb renders enabled exactly as before — a fabricated
 * disabled state would be a phantom (Constitution III.11).
 */

import { useEffect, useState } from "react";
import { fetchVerbEligibility } from "@/lib/verbs";
import type { VerbEligibilityEntry } from "@/types/game";

export type VerbEligibilityMap = Record<string, VerbEligibilityEntry>;

export function useVerbEligibility(
  gameId: string,
  orgId: string,
  tick: number | null,
): VerbEligibilityMap | null {
  const [map, setMap] = useState<VerbEligibilityMap | null>(null);

  useEffect(() => {
    if (!orgId) {
      setMap(null);
      return;
    }
    let cancelled = false;
    // fetchVerbEligibility never throws (normalized result object).
    fetchVerbEligibility(gameId, orgId).then((res) => {
      if (cancelled) return;
      if (!res.ok || res.data === null) {
        setMap(null);
        return;
      }
      const next: VerbEligibilityMap = {};
      for (const entry of res.data.verbs) {
        next[entry.verb] = entry;
      }
      setMap(next);
    });
    return () => {
      cancelled = true;
    };
  }, [gameId, orgId, tick]);

  return map;
}
```

(f) Rewrite `src/frontend/src/components/action/VerbGrid.tsx` in full:

```tsx
/**
 * Flat 9-verb grid (Constitution Article V — no invented tabs/groupings).
 * Verbs are NEVER hidden, only disabled with an honest reason:
 *  - `DISABLED_VERBS` (static): a verb shipped without an engine handler
 *    (empty today — AW3-R1).
 *  - `eligibility` (dynamic, spec-116 FR-4.8): a verb whose target
 *    predicate is empty right now renders disabled, with the backend's
 *    reason + remedy visible below the grid — the same predicates that
 *    would otherwise produce a dead-end "No eligible targets." click.
 * HONEST-NULL: while `eligibility` is null (fetch unresolved or failed)
 * every verb renders enabled exactly as before — a fabricated disabled
 * state would be a phantom (Constitution III.11).
 */

import { VERBS, DISABLED_VERBS } from "@/lib/verb-config";
import type { LiveVerbCost } from "@/lib/verbs";
import type { PlayerVerb } from "@/types/game";
import type { VerbEligibilityMap } from "./useVerbEligibility";

interface VerbGridProps {
  selectedVerb: PlayerVerb | null;
  onSelect: (verb: PlayerVerb) => void;
  /** Live cost for the SELECTED verb only (null for the other 8 buttons —
   *  a fetch only ever runs for the currently-composed verb, see
   *  useVerbTargets.ts) — falls back to the static cost_label hint until
   *  the fetch resolves or when the verb has no parseCost. */
  liveCost: LiveVerbCost | null;
  /** Per-verb dynamic eligibility keyed by verb; null while unresolved. */
  eligibility: VerbEligibilityMap | null;
}

export function VerbGrid({
  selectedVerb,
  onSelect,
  liveCost,
  eligibility,
}: VerbGridProps): React.JSX.Element {
  const ineligible = VERBS.flatMap((v) => {
    const dyn = eligibility?.[v.verb];
    return dyn && dyn.eligible === false
      ? [{ verb: v.verb, label: v.label, reason: dyn.reason, remedy: dyn.remedy }]
      : [];
  });

  return (
    <div className="flex flex-col gap-1">
      <div className="grid grid-cols-3 gap-1.5" data-testid="verb-grid">
        {VERBS.map((v) => {
          const staticDisabled = DISABLED_VERBS.has(v.verb);
          const dyn = eligibility?.[v.verb];
          const dynDisabled = dyn?.eligible === false;
          const disabled = staticDisabled || dynDisabled;
          const active = selectedVerb === v.verb;
          const costText = active && liveCost ? liveCost.label : v.cost_label;
          const title = staticDisabled
            ? `${v.label}: no engine handler yet`
            : dynDisabled
              ? `${v.label} — no eligible targets yet: ${[dyn?.reason, dyn?.remedy]
                  .filter(Boolean)
                  .join(" ")}`
              : `${v.label} — ${costText}`;
          return (
            <button
              key={v.verb}
              disabled={disabled}
              title={title}
              onClick={() => onSelect(v.verb as PlayerVerb)}
              className={`flex flex-col items-center gap-0.5 rounded border p-2 text-center disabled:cursor-not-allowed disabled:opacity-40 ${
                active
                  ? "border-spire bg-spire/10 text-spire"
                  : "border-rebar text-fog hover:border-wet-steel"
              }`}
            >
              <span className="text-base">{v.glyph}</span>
              <span className="text-[9px] uppercase tracking-widest">{v.label}</span>
            </button>
          );
        })}
      </div>
      {ineligible.length > 0 && (
        <ul data-testid="verb-ineligible-reasons" className="flex flex-col gap-0.5">
          {ineligible.map((row) => (
            <li key={row.verb} className="text-[10px] italic text-shroud">
              <span className="font-mono not-italic uppercase tracking-widest">
                {row.label}
              </span>{" "}
              — no eligible targets yet: {[row.reason, row.remedy].filter(Boolean).join(" ")}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
```

(g) `src/frontend/src/components/action/ActionComposer.tsx` — three edits:

```tsx
// imports (after the VerbForm import, :18):
import { useVerbEligibility } from "./useVerbEligibility";

// after the liveCost state (:36):
  const eligibility = useVerbEligibility(gameId, activeOrgId, snapshot?.tick ?? null);

// replace the VerbGrid mount (:57):
          <VerbGrid
            selectedVerb={verb}
            onSelect={setVerb}
            liveCost={verb ? liveCost : null}
            eligibility={eligibility}
          />

// and add one prop to the VerbForm mount (:59-71):
              eligibility={eligibility?.[verb] ?? null}
```

(h) `src/frontend/src/components/action/VerbForm.tsx` — add to the props interface (:17-29):

```tsx
  /** The verb's eligibility row (spec-116 FR-4.8) — feeds the reason-
   *  bearing empty state in TargetPicker; null/absent falls back to the
   *  legacy bare line. */
  eligibility?: VerbEligibilityEntry | null;
```

with `import type { GameSnapshot, PlayerVerb, VerbEligibilityEntry } from "@/types/game";` replacing the existing type import (:11), destructure `eligibility` in the component signature, and pass the composed reason to the picker (:90-98):

```tsx
  const emptyReason =
    eligibility && eligibility.eligible === false
      ? [eligibility.reason, eligibility.remedy].filter(Boolean).join(" ")
      : null;
```

```tsx
      {showPicker && (
        <TargetPicker
          targets={targets}
          loading={loading}
          error={error}
          selectedId={targetId}
          onSelect={setTargetId}
          emptyReason={emptyReason}
        />
      )}
```

(i) Rewrite `src/frontend/src/components/action/TargetPicker.tsx` in full. CAUTION:
Task 18 has ALREADY landed the `TargetDeltaChip` renderer + per-row chips in this file
(FR-116-4.4) — this rewrite RETAINS them verbatim; only the empty state changes. Task
18's `TargetPicker.test.tsx` must stay green after this step (it is in the Step 9 run
below):

```tsx
/**
 * Target picker — flat list of `VerbTarget`s, grouped when the config's
 * targets carry a `group` (e.g. Aid's Communities/Organizations split).
 * Rows carry expected-delta chips (spec-116 FR-4.4). The empty state
 * carries the verb's eligibility reason + remedy when known
 * (spec-116 FR-4.8) so an empty list is never a mute dead-end.
 */

import type { VerbTarget } from "@/lib/verbs";

/** One compact ▲/▼ chip for a non-zero expected delta — null otherwise
 *  (the same honest-null convention as VerbForm's preview DeltaChip). */
function TargetDeltaChip({
  value,
  label,
}: {
  value: number | undefined;
  label: string;
}): React.JSX.Element | null {
  if (value === undefined || !Number.isFinite(value) || value === 0) return null;
  const up = value > 0;
  return (
    <span
      data-testid="target-delta"
      title={`${label}: ${up ? "+" : ""}${value}`}
      className={`font-mono text-[9px] ${up ? "text-accent-gold" : "text-accent-crimson"}`}
    >
      {up ? "▲" : "▼"}
      {label} {up ? "+" : "-"}
      {parseFloat(Math.abs(value).toPrecision(3))}
    </span>
  );
}

interface TargetPickerProps {
  targets: VerbTarget[];
  loading: boolean;
  error: string | null;
  selectedId: string | null;
  onSelect: (id: string) => void;
  /** Reason + remedy for an empty list; null falls back to the bare
   *  legacy line (honest-null — never fabricate a reason). */
  emptyReason?: string | null;
}

export function TargetPicker({
  targets,
  loading,
  error,
  selectedId,
  onSelect,
  emptyReason,
}: TargetPickerProps): React.JSX.Element {
  if (loading) {
    return <p className="text-[11px] text-ash">Loading targets…</p>;
  }
  if (error) {
    return (
      <p role="alert" className="text-[11px] text-laser">
        {error}
      </p>
    );
  }
  if (targets.length === 0) {
    return (
      <p className="text-[11px] italic text-shroud" data-testid="targets-empty">
        {emptyReason ? `No eligible targets yet: ${emptyReason}` : "No eligible targets."}
      </p>
    );
  }

  return (
    <div className="flex max-h-40 flex-col gap-1 overflow-y-auto" data-testid="target-picker">
      {targets.map((t) => (
        <button
          key={t.id}
          onClick={() => onSelect(t.id)}
          className={`flex items-center justify-between rounded border px-2 py-1 text-left text-[11px] ${
            t.id === selectedId
              ? "border-spire bg-spire/10 text-spire"
              : "border-rebar text-bone hover:border-wet-steel"
          }`}
        >
          <span className="truncate">{t.label}</span>
          <span className="ml-2 flex shrink-0 items-center gap-1.5">
            <TargetDeltaChip value={t.expectedDeltas?.consciousness} label="CI" />
            <TargetDeltaChip value={t.expectedDeltas?.heat} label="Heat" />
            {t.group && <span className="text-[9px] text-ash">{t.group}</span>}
          </span>
        </button>
      ))}
    </div>
  );
}
```

(j) `src/frontend/src/test/handlers.ts` — insert into the `handlers` array next to the verb-targets handler (~:338). REQUIRED in this same step: `setup.ts` runs `server.listen({ onUnhandledRequest: "error" })`, so once ActionComposer fetches eligibility, every existing composer test would red without this default:

```ts
  // ---- Spec-116 FR-4.8: per-verb eligibility (VerbGrid disabled-with-
  // reason). Default: empty verbs list — the honest-null path (nothing
  // disabled). Tests that need real rows override with server.use().
  http.get("/api/games/:id/actions/eligibility/", ({ request }) => {
    logRequest("GET actions:eligibility");
    const orgId = new URL(request.url).searchParams.get("org_id") ?? "";
    return HttpResponse.json({
      status: "ok",
      data: { session_id: DEFAULT_GAME_ID, tick: 1, org_id: orgId, verbs: [] },
    });
  }),
```

Run:

```bash
cd src/frontend && npx vitest run src/components/action/ActionComposer.test.tsx \
    src/components/action/VerbForm.test.tsx src/components/action/TargetPicker.test.tsx
cd src/frontend && npx vitest run
```

Expected: the scoped run fully green (8 ActionComposer tests + VerbForm suite +
Task 18's TargetPicker chip tests — proof the rewrite kept the chips); then the whole
suite green (`Test Files N passed`), proving no other component reds on the new fetch.

- [ ] **Step 10: E2E (tick-0 wayne_county dead-end → disabled-with-reason), gates, commit**

In `src/frontend/e2e/verb-submit.spec.ts`, replace the test at :33-53 ("ActionComposer renders the 9-verb grid; all 9 verbs are enabled") — its claim is superseded by FR-116-4.8 — and update the file docstring's "all 9 verbs enabled" sentence to note the tick-0 eligibility contract:

```ts
  test("VerbGrid tick-0 eligibility: EDUCATE and MOBILIZE disabled with visible reason, no dead-end clicks (spec-116 FR-4.8)", async ({
    page,
  }) => {
    expect(gameId, "session-creation test ran first").toBeTruthy();
    await page.goto(`/game/${gameId}`);
    await expect(page.getByTestId("action-composer")).toBeVisible({ timeout: 15000 });
    const verbGrid = page.getByTestId("verb-grid");
    await expect(verbGrid).toBeVisible();

    // Tick-0 wayne_county: no social_class node carries the org's
    // territories (structural — SocialClass has no territory_ids field)
    // and the only co-located org is the state apparatus, so EDUCATE and
    // MOBILIZE are disabled-with-reason instead of dead-ending into
    // "No eligible targets."
    const educate = verbGrid.getByRole("button", { name: /educate/i });
    await expect(educate).toBeDisabled({ timeout: 15000 });
    await expect(educate).toHaveAttribute(
      "title",
      /no eligible targets yet: No organized community/,
    );
    await expect(verbGrid.getByRole("button", { name: /mobilize/i })).toBeDisabled();

    // Reason + remedy are VISIBLE, not tooltip-only.
    const reasons = page.getByTestId("verb-ineligible-reasons");
    await expect(reasons).toContainText("No organized community in your territories yet.");
    await expect(reasons).toContainText("political education unlocks");

    // Article V: the other seven verbs stay enabled; nothing is hidden.
    for (const verb of [
      "Aid",
      "Attack",
      "Campaign",
      "Move",
      "Investigate",
      "Reproduce",
      "Negotiate",
    ]) {
      await expect(verbGrid.getByRole("button", { name: new RegExp(verb, "i") })).toBeEnabled();
    }
    await expect(verbGrid.getByRole("button")).toHaveCount(9);
  });
```

`verb-submit.spec.ts` is already in `AUTHENTICATED_SPECS` (playwright.config.ts:30-35) — no config change. The spec's later campaign tests are untouched (campaign stays eligible and submittable, proving disabled-with-reason never blocks a live verb).

Run everything:

```bash
mise run web:dev            # Django :8000 (autoreloader sets RUN_MAIN => real bridge) + Vite :5173
cd src/frontend && npx playwright test e2e/verb-submit.spec.ts
# expected: 4 passed
cd ../.. && mise run web:check   # tsc + eslint + prettier + vitest — green
mise run check                   # fast gate incl. check:seams + test:unit — green
mise run qa:regression           # untouched web-tier change: "Results: 5 passed, 0 failed" with NO baseline regen
mise run commit -- "feat(frontend): VerbGrid disabled-with-reason + eligibility-aware empty states (spec-116 FR-4.8)"
```

Expected: e2e 4 passed; `web:check` green; `check` green (seam gate untouched — verified: the action surface has no gating seam check, and the typed `endpoints.ts` row keeps the advisory sweep clean); `qa:regression` byte-identical; commit lands with the `Co-Authored-By` trailer.
## Cluster Z — Integration, acceptance, governance

### Task 25: First-session trunk e2e + acceptance-gate sweep + ADR079

**Files:**
- Create: `src/frontend/e2e/first-session.spec.ts`
- Modify: `src/frontend/playwright.config.ts` (add `"first-session.spec.ts"` to
  `AUTHENTICATED_SPECS`)
- Create: `ai/decisions/ADR079_playability_spine.yaml` + entry in
  `ai/decisions/index.yaml`
- Modify: `ai/state.yaml` (recently_completed entry), `specs/116-playability-spine/spec.md`
  (Status → EXECUTED + gate results)
- Test: the spec file itself is the test.

**Interfaces:**
- Consumes: every prior task's shipped surface (codename lobby, briefing route,
  disabled-with-reason verb grid, preview, dedup/autopause-once, endgame_progress).
- Produces: the spine's acceptance evidence.

- [ ] **Step 1: Write the trunk e2e (red where surfaces are missing)** — FOLLOW-PATTERN
  (`src/frontend/e2e/real-loop.spec.ts` + `e2e/fixtures.ts`; fresh session via the
  fixtures helper; bounded loops only):

```ts
import { expect, test } from "@playwright/test";
// FOLLOW-PATTERN: import the login/create helpers from "./fixtures".

test("a fresh player reaches their first submitted action unaided", async ({ page }) => {
  // 1. Login → lobby shows generated operation codenames (no unnamed rows).
  // 2. Create a game → lands on the Scenario Briefing (cadre-council framing,
  //    five patterns listed, win condition named) → "Begin" enters the cockpit.
  await expect(page.getByTestId("scenario-briefing")).toBeVisible();
  // 3. Verb grid: ineligible verbs are disabled WITH visible reasons (no dead ends).
  // 4. Pick an eligible verb → target picker rows show expected deltas →
  //    preview (cost + warnings) visible BEFORE submit → submit succeeds.
  //    (The preview surfaces are Task 17's: probability line + verb cost.)
  await expect(page.getByTestId("preview-probability")).toBeVisible();
  await expect(page.getByTestId("verb-cost")).toBeVisible();
  // 5. Resolve two ticks: no two consecutive identical event cards; the critical-event
  //    modal appears at most once for the same (type, subject).
  // 6. endgame_progress axes render in the objectives tray (no 1.00-at-tick-1 bars).
});
```

- [ ] **Step 2: Run the full e2e suite locally**

`mise run web:dev` (if not already running), then:
Run: `cd src/frontend && npx playwright test`
Expected: ALL specs pass (including the pre-existing 9 — the takeover/testid freeze
holds).

- [ ] **Step 3: Acceptance-gate sweep (record results in spec-116 §gates)**

| Gate | Command / evidence |
|---|---|
| 1. Null-play horizon | `reports/pacing-calibration-2026-07-17.md` (Task 6) — first_recognition all-null on `us`-260 and on the FULL 5200-tick `wayne_county` run; the full `us` century run was KILLED AND DEFERRED by owner ruling 2026-07-17 (report §4 addendum — "a lot of work to be done before I put that kind of work into it"); PLUS Task 2's tick-0 exemption test (no tick-0 Sovereign Collapse) and Task 3/4's horizon-only-termination tests (no terminator paths remain) |
| 2. No consecutive identical cards | dedup vitest + e2e step 5 |
| 3. Autopause ≤ 1 per distinct event | autopause-once vitest + e2e step 5 |
| 4. Distinct epilogues | epilogue unit tests (6 outcomes, 6 distinct headline+body) |
| 5. Preview before every submit | e2e step 4 |
| 6. Fresh player unaided | this spec passing |

- [ ] **Step 4: Full verification battery (single-flight)**

```bash
mise run check          # lint + format + typecheck + unit + seams
mise run qa:regression  # 5/5 against the ceremony baselines
cd src/frontend && npm run check   # frontend lint+types+vitest
```
Expected: all green. Any failure: STOP and fix before proceeding.

- [ ] **Step 5: Governance artifacts**

- `ai/decisions/ADR079_playability_spine.yaml` (seeded `proposed` in Task 1 Step 7b —
  UPDATE it, don't recreate): flip to `status: accepted`; complete it covering:
  - fixed-horizon ruling (100y / 5200 ticks; outcomes = recognized patterns, never
    terminators; accept-outcome is the only early exit);
  - EndgameDetector → pattern-recognizer conversion (old `is_game_over` /
    `outcome` / `_emit_endgame_event` API deleted);
  - BOTH declared ceremonies with drift-table pointers: ceremony #1 (Task 6, pacing
    calibration → `reports/pacing-calibration-2026-07-17.md`) and ceremony #2
    (Task 23, event-whitelist widening → its commit-body drift list);
  - **schema-ownership change (Task 19):** the web path now applies persistence
    migrations on session bootstrap — previously the headless runner was the sole
    schema owner; record the new ownership boundary and why;
  - **MARKET_CORRECTION promoted to a discrete wire event** (scope extension over
    Program 23's counter-only design) — record the rationale;
  - salience re-tier (frontend-only vocabulary; bridge `_EVENT_SEVERITY` untouched
    except additive entries), lobby/briefing, and the nine quick-win wirings.
  Add the entry to `ai/decisions/index.yaml`.
- `ai/state.yaml`: recently_completed entry for spec-116.
- `specs/116-playability-spine/spec.md`: Status → EXECUTED, gates table filled.

- [ ] **Step 6: Commit + PR**

```bash
git add src/frontend/e2e/first-session.spec.ts src/frontend/playwright.config.ts \
  ai/decisions/ ai/state.yaml specs/116-playability-spine/spec.md
git commit -m "feat(spine): first-session trunk e2e + acceptance gates + ADR079 (spec-116 complete)"
git push -u origin feature/116-playability-spine
gh pr create --base dev --title "feat: Playability Spine — spec-116 (Program 24)" --body "<see below>"
```

PR body MUST contain, in this order:
1. The acceptance-gates table (Step 3) with evidence links.
2. Ceremony declarations: ceremony #1 (Task 6) and ceremony #2 (Task 23) — per-scenario
   drift lists copied from the two ceremony commit bodies.
3. **Owner flags** (decisions Percy should ratify or overrule, none blocking merge):
   - EDUCATE remains structurally ineligible in every reachable state (making it
     eligible is an engine change, out of spine scope) — honest disabled-with-reason
     copy shipped instead; needs an owner ruling on whether a future track fixes it.
   - MARKET_CORRECTION promoted to a discrete wire event — scope extension vs
     Program 23's counter-only design.
   - Fascist verbs (POGROM/LOCKOUT/VIGILANTISM): Task 22 wires the resolver pipe, but
     no live-loop emitter reaches them yet — reachability gap flagged, not fixed.
   - Two-vocabulary severity seam: frontend salience re-tier vs bridge
     `_EVENT_SEVERITY` are now deliberately different vocabularies; follow-up ruling
     on whether to unify.
   - Backend severity tiers for the 17 newly whitelisted/first-class event types are
     drafter rulings on an open taste question (pogrom/lockout/vigilantism = warning,
     secession_declared = critical, market_correction et al. = warning, rest
     informational) — owner may re-tier any of them, one-line edits.
   - Preview ruling: `!canAfford` warns loudly (crimson "insufficient") but does NOT
     disable submit — deliberate; owner may flip to hard-disable.
   - Schema-ownership change (Task 19): web path now applies persistence migrations
     on session bootstrap (headless runner was previously sole schema owner) —
     recorded in ADR079, surfaced here for BD sign-off.
   - **Ceremony accounting deviation:** the spine carries a second declared
     regeneration (Task 23, whitelist widening) — parent design §3 reserved
     "ceremony #2" for Track 3 Unit 6. Sanction the extra spine ceremony, or the
     converter widening defers to Track 3 (spec-116 §Constraints records the
     deviation and the fallback).
   - Pattern-lock does NOT autopause: after the salience re-tier, only
     `endgame_reached` autopauses; `PATTERN_SHIFT`/lock is gold (`important`). The
     spec's "first occurrence or escalation" could be read to include pattern-lock —
     confirm the drafters' crimson-reserved-for-rupture reading, or promote
     `pattern_shift` to critical (one-line classifier edit).
   - Quick-win 5 is honest-partial: 5 aggregate series + 16 territory keys wired;
     the remaining dead `tick_*` attrs stay declared-dark (write-site fallback
     constants — serializing them would be dishonest chrome per III.11).
   - Fascist-axis progress reading: the FASCIST_CONSOLIDATION HUD bar reports
     `max` of its two route-progresses (false-consciousness fraction vs political
     violence), not a pooled mean — preserves pre-existing either-route recognition;
     recognition boundaries across all axes are now inclusive (>= / <=) per the
     plan's gate conventions. Confirm or re-rule the max-of-routes display.
   - Ceremony #1 calibration outcome: `fascist_majority_fraction` default 0.75 → 0.9
     — a DEFENSIVE granularity fix (6 archetypal entities quantize the fraction in
     1/6 steps; at 0.9 only total 6/6 capture matches), zero baseline drift, but it
     is a shipped balance value coupled to the current entity count — confirm 0.9.
   - Null-play finding (engine follow-up, out of spine scope): all five recognizer
     axes are FLAT for entire runs — archetypal class ideology never moves under
     null play; "tension" is a static 0.74, not a building arc. Needs a
     ConsciousnessSystem/SolidaritySystem investigation program.
4. The standard Claude Code attribution footer.

Wait for CI green; merge per delegated authority; verify the merge commit's CI.
