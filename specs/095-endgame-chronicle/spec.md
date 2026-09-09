# Feature Specification: Endgame Chronicle + Journal + Dialectic Screen

**Feature Branch**: `095-endgame-chronicle`
**Created**: 2026-07-05
**Status**: In Progress
**Program**: 09 Full-Game Build — Lane W (web product). Stacks on `8a521a33` (094 HEAD).
**Deps**: spec-090 (Cold Collapse tokens) ✅, spec-091 (frontend consolidation) ✅, spec-092 (event stream / journal) ✅.

## Overview

Spec-095 delivers the ruled victory-UX: the chronicle end-screen, Vic3-style Journal
objectives, and the Dialectic screen visualizing the LIVE contradiction layer. Together
these close the player's narrative loop — the player can SEE the dialectic in motion
(`DialecticSpread`), track their objectives (`JournalPage`), and read the final verdict
(`EndStateScreen` / chronicle).

The design canon lives in `design/mockups/ui_kits/webapp/` (`EndState.jsx`,
`DialecticSpread.jsx`). Those mockups are standalone CDN-React JSX — reference only, not
imported. This spec ports them as fresh project code against real bridge methods that
read the engine's contradiction layer (ADR051: OppositionRegistry defects,
`contradiction_field` rows, `dialectical_regime`, level lattices / Aufhebung).

NOTE: this partial-forwards an 081 deliverable (chronicle) — 081 later enriches content;
flagged in §6.

### Constitution III (AI observes, never controls)

The chronicle / dialectic visualization is **presentation only**:
- All endpoints are GET reads over already-persisted engine state.
- Zero `babylon.*` imports in frontend; zero engine state writes.
- The Dialectic screen reads the `contradiction_field` table and graph attributes that
  the engine's `ContradictionSystem` already writes each tick — it never computes
  dialectical state itself.

### EndgameDetector priority discrepancy (known non-blocker, resolved here)

`project/01-state-of-the-world.md` line 284 documents: "EndgameDetector docstring claims
REVOLUTIONARY_VICTORY-first priority; code checks it last (FR-033)." This spec is the
discrepancy's first consumer. The red test is written HERE (TDD red phase).

**Resolution decision**: FR-033 (spec-070) is the authoritative priority order
(`RED_OGV → FRAGMENTED_COLLAPSE → ECOLOGICAL_COLLAPSE → FASCIST_CONSOLIDATION →
REVOLUTIONARY_VICTORY`, first-match-wins). The stale docstring is from Slice 1.6 (3
outcomes, pre-spec-070). The CODE is correct; the DOCSTRING is stale. The docstring fix
lives in `src/babylon/engine/observers/endgame_detector.py` (engine code, cross-lane) —
FLAGGED, not edited by this spec. The red test asserts FR-033's order and passes against
the current code.

A **bridge-layer bug** is also found and fixed here: `resolve_tick` (line 1058) only
recognizes 3 of 5 endgame event types (`REVOLUTIONARY_VICTORY`, `ECOLOGICAL_COLLAPSE`,
`FASCIST_CONSOLIDATION`), missing `RED_OGV` and `FRAGMENTED_COLLAPSE`. This IS in
`web/game/` — fixed by this spec.

## User Scenarios & Testing *(mandatory)*

### US1 — Dialectic screen shows live opposition defects (Priority: P1, gate)

A player navigates to `/games/:id/dialectic` mid-game. They see a card grid of active
contradictions — each card shows the thesis ↔ antithesis poles, the tension (gap), the
rate of development, and the current regime (reproduction / crisis / sublation). The
principal contradiction is highlighted. Data comes from real `contradiction_field` rows
and graph attributes — never fixtures.

**Independent test**: pytest — `get_contradiction_snapshot` returns a well-formed
ContradictionSnapshot from a mock persistence layer seeded with `contradiction_field`
rows. Vitest — `DialecticPage` renders cards from a contract-faithful fixture; asserts
principal is highlighted; asserts tension bar renders.

### US2 — Chronicle renders a terminal outcome (Priority: P1, gate)

A Playwright driver drives a seeded game to a terminal outcome. The chronicle
end-screen renders the correct outcome label, headline, stat cards (final tick,
consciousness, solidarity edges, heat), and a "New Operation" button. The 5 outcomes
each render with their canonical framing:
- REVOLUTIONARY_VICTORY → "BABYLON FALLS" (rupture palette)
- ECOLOGICAL_COLLAPSE → "THE BUNKER FAILS" (laser palette)
- FASCIST_CONSOLIDATION → "THE BUNKER FAILS" (laser palette)
- RED_OGV → "THE BUNKER FAILS" (laser palette)
- FRAGMENTED_COLLAPSE → "THE BUNKER FAILS" (laser palette)

**Independent test**: pytest — `get_endgame_state` returns the terminal outcome from a
mock bridge; asserts all 5 outcome types surface (resolves the 3-of-5 bridge bug).
Vitest — `EndStateScreen` renders the correct headline for each outcome.

### US3 — Journal objectives tracker (Vic3-style) (Priority: P1)

A player navigates to `/games/:id/journal` (renamed from the event log's journal — the
event log stays at `/log`). They see a Vic3-style objectives tracker: each objective
has a title, description, progress bar (0–1), status (active/complete/failed), and
category. Objectives derive from the 5 endgame conditions — the player can see their
progress toward revolution or their drift toward collapse.

**Independent test**: Vitest — `ObjectivesTracker` renders objectives from a fixture;
asserts progress bars render; asserts status badges are correct. pytest —
`get_journal_objectives` returns well-formed objectives from a mock state.

### US4 — EndgameDetector priority contract (Priority: P1, gate)

The EndgameDetector evaluates predicates in FR-033 order
(`RED_OGV → FRAGMENTED_COLLAPSE → ECOLOGICAL_COLLAPSE → FASCIST_CONSOLIDATION →
REVOLUTIONARY_VICTORY`), first-match-wins. The stale Slice-1.6 docstring
("REVOLUTIONARY_VICTORY-first") is documented as cross-lane.

**Independent test**: pytest — construct a synthetic `EndgameDetector`, drive `on_tick`
with a state satisfying BOTH REVOLUTIONARY_VICTORY and FASCIST_CONSOLIDATION conditions;
assert the outcome is FASCIST_CONSOLIDATION (checked first per FR-033, NOT
REVOLUTIONARY_VICTORY). This is the red test that pins the discrepancy's resolution.

### US5 — Contradiction snapshot contract (Priority: P1, gate)

`GET /api/games/{id}/contradiction/` returns a `ContradictionSnapshot` with: the
principal contradiction, all opposition states (key, gap, rate, balance, leading_pole,
is_principal), the dialectical regime, and the contradiction frame (principal + secondary
aspects). Matches `contracts/contradiction.yaml`.

**Independent test**: Vitest — MSW contract test for `/api/games/:id/contradiction/`.

## Requirements *(mandatory)*

- **FR-095-01**: `EngineBridge.get_contradiction_snapshot(session_id)` MUST read
  `contradiction_field` rows (via the persistence pool's SQL, same pattern as
  `_fetch_session_rng_seed_from_pool`) for the latest tick AND graph attributes
  (`contradiction_frames`, `dialectical_regime`) via `hydrate_graph`. Returns a
  `ContradictionSnapshot` dict.
- **FR-095-02**: `EngineBridge.get_endgame_state(session_id)` MUST return the terminal
  outcome from the latest snapshot. The `endgame` key on the resolve_tick snapshot MUST
  recognize ALL 5 GameOutcome event types (`REVOLUTIONARY_VICTORY`,
  `ECOLOGICAL_COLLAPSE`, `FASCIST_CONSOLIDATION`, `RED_OGV`, `FRAGMENTED_COLLAPSE`) —
  fixing the 3-of-5 bridge bug at `engine_bridge.py:1058`.
- **FR-095-03**: `EngineBridge.get_journal_objectives(session_id)` MUST derive Vic3-style
  objectives from the current game state. Each objective has: `id`, `title`,
  `description`, `progress` (0.0–1.0), `status` ("active"|"complete"|"failed"),
  `category`. Objectives map to the 5 endgame conditions.
- **FR-095-04**: `GET /api/games/{id}/contradiction/` MUST serve the
  `ContradictionSnapshot` via a new `game_contradiction` Django view, following the
  `game_journal` / `game_wire` pattern.
- **FR-095-05**: `GET /api/games/{id}/endgame/` MUST serve the endgame state via a new
  `game_endgame` Django view.
- **FR-095-06**: `GET /api/games/{id}/objectives/` MUST serve the journal objectives via
  a new `game_objectives` Django view.
- **FR-095-07**: The frontend `useContradiction(gameId)` hook MUST poll
  `GET /api/games/{id}/contradiction/` on a 2s interval, exposing
  `{data, loading, error, refresh}`.
- **FR-095-08**: The `DialecticPage` MUST render a card grid of active contradictions
  ported from `DialecticSpread.jsx`, using Cold Collapse tokens (spec-090). Each card
  shows thesis ↔ antithesis, tension bar, rate, regime. Principal is highlighted.
- **FR-095-09**: The `EndStateScreen` MUST render the chronicle end-screen ported from
  `EndState.jsx`, using Cold Collapse tokens. The `outcome` prop drives the palette
  (rupture = bronze-gold, defeat = laser-red) and headline.
- **FR-095-10**: The `ObjectivesTracker` MUST render Vic3-style objective cards with
  progress bars and status badges, using Cold Collapse tokens.
- **FR-095-11**: Routes MUST be added in `App.tsx` under `GameRouteShell`:
  `<Route path="dialectic" element={<DialecticPage />} />`,
  `<Route path="chronicle" element={<ChroniclePage />} />`,
  `<Route path="objectives" element={<ObjectivesPage />} />`.
- **FR-095-12**: NavRail entries MUST be added: Dialectic (⊛), Chronicle (◉),
  Objectives (▣).
- **FR-095-13**: MSW handlers MUST exist for `/api/games/:id/contradiction/`,
  `/api/games/:id/endgame/`, `/api/games/:id/objectives/` in `handlers.ts`.
- **FR-095-14**: An EndgameDetector priority test MUST be written in
  `tests/unit/web/test_endgame_priority.py` asserting FR-033 order
  (FASCIST_CONSOLIDATION wins over REVOLUTIONARY_VICTORY when both hold). The stale
  docstring fix in `src/babylon/engine/observers/endgame_detector.py` is FLAGGED as
  cross-lane — not edited by this spec.

## Success Criteria *(mandatory)*

- **SC-095-01**: `mise run web:check` green (tsc + eslint + prettier + Vitest).
- **SC-095-02**: `PYTHONPATH=src uv run pytest tests/unit/web/ -q` green, including
  contradiction-snapshot contract test, endgame priority test, and 5-outcome test.
- **SC-095-03**: Contradiction-snapshot contract test passes (bridge returns real
  contradiction data, not fixtures).
- **SC-095-04**: EndgameDetector priority test passes (FR-033 order pinned).
- **SC-095-05**: Playwright: drives a seeded game to a terminal outcome, asserts the
  chronicle renders (owner-run, gated on `SPEC061_TEST_SESSION_ID`).
- **SC-095-06**: MSW contract tests for all 3 new endpoints pass.

## Known Gaps (documented, not fixed here)

1. **Chronicle content enrichment (spec-081)**: This spec ships the chronicle
   end-SCREEN (layout + stat cards + outcome label). The rich narrative content —
   per-outcome chronicle paragraphs, historical ledger, "what happened" timeline — is
   spec-081's deliverable. Flagged in §6.

2. **EndgameDetector docstring**: The stale Slice-1.6 docstring in
   `src/babylon/engine/observers/endgame_detector.py` (lines 18-19, 196-199) says
   "REVOLUTIONARY_VICTORY-first" while the code follows FR-033
   (REVOLUTIONARY_VICTORY-last). The docstring fix is engine code (cross-lane) — flagged,
   not edited. The red test (US4) pins the correct order.

3. **Level lattice / Aufhebung visualization**: The Dialectic screen surfaces the
   `dialectical_regime` (reproduction/crisis/sublation) but does not render the full
   level-lattice Aufhebung tree — that visualization is a future spec. The regime label
   is the surface-level affordance.

4. **Node-level contradiction fields**: `contradiction_field` rows persist only the
   `global` frame (one row per opposition key, `node_id="global"`). Per-node fields live
   on graph nodes (`contradiction_fields` attr) but are not yet surfaced as a per-node
   heatmap — future spec.
