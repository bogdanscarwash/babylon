# Tasks: Endgame Chronicle + Journal + Dialectic Screen

**Spec**: 095-endgame-chronicle

## Phase 0 — Speckit

- [x] T001 Create `specs/095-endgame-chronicle/spec.md`
- [x] T002 Create `specs/095-endgame-chronicle/plan.md`
- [x] T003 Create `specs/095-endgame-chronicle/tasks.md`
- [x] T004 Create `specs/095-endgame-chronicle/research.md`
- [x] T005 Create `specs/095-endgame-chronicle/contracts/contradiction.yaml`
- [x] T006 Create `specs/095-endgame-chronicle/contracts/endgame.yaml`
- [x] T007 Create `specs/095-endgame-chronicle/contracts/objectives.yaml`

## Phase 1 — RED tests (TDD red phase)

- [ ] T010 Red: `tests/unit/web/test_endgame_priority.py` — EndgameDetector FR-033 order
      (FASCIST_CONSOLIDATION wins over REVOLUTIONARY_VICTORY when both hold)
- [ ] T011 Red: `tests/unit/web/test_engine_bridge.py` — `get_contradiction_snapshot`
      returns real contradiction data from mock persistence
- [ ] T012 Red: `tests/unit/web/test_engine_bridge.py` — `get_endgame_state` surfaces
      all 5 outcomes (3-of-5 bridge bug)
- [ ] T013 Red: `tests/unit/web/test_engine_bridge.py` — `get_journal_objectives`
      returns well-formed objectives
- [ ] T014 Red: `tests/unit/web/test_api.py` — 3 new views return envelopes
- [ ] T015 Red: `web/frontend/src/__tests__/integration/contradiction-contract.test.tsx`
- [ ] T016 Red: `web/frontend/src/__tests__/integration/endgame-contract.test.tsx`
- [ ] T017 Red: `web/frontend/src/__tests__/integration/objectives-contract.test.tsx`

## Phase 2 — Backend GREEN

- [ ] T020 Implement `get_contradiction_snapshot` in `engine_bridge.py`
- [ ] T021 Implement `get_endgame_state` in `engine_bridge.py`
- [ ] T022 Implement `get_journal_objectives` in `engine_bridge.py`
- [ ] T023 FIX `resolve_tick` endgame_types (3→5) in `engine_bridge.py`
- [ ] T024 Implement `game_contradiction` view in `api.py`
- [ ] T025 Implement `game_endgame` view in `api.py`
- [ ] T026 Implement `game_objectives` view in `api.py`
- [ ] T027 Add 3 routes in `urls.py`
- [ ] T028 Verify backend tests green: `PYTHONPATH=src uv run pytest tests/unit/web/ -q`

## Phase 3 — Frontend GREEN

- [ ] T030 Create `types/dialectic.ts` (ContradictionSnapshot, EndgameState, Objective)
- [ ] T031 Create `hooks/useContradiction.ts`, `hooks/useEndgame.ts`, `hooks/useObjectives.ts`
- [ ] T032 Create `components/dialectic/DialecticSpread.tsx` + `dialectic.css`
- [ ] T033 Create `components/chronicle/EndStateScreen.tsx` + `chronicle.css`
- [ ] T034 Create `components/objectives/ObjectivesTracker.tsx` + `objectives.css`
- [ ] T035 Create 3 page wrappers: `DialecticPage`, `ChroniclePage`, `ObjectivesPage`
- [ ] T036 Add 3 routes in `App.tsx`
- [ ] T037 Add 3 NavRail entries
- [ ] T038 Add 3 MSW handlers in `test/handlers.ts`
- [ ] T039 Verify frontend tests green: `mise run web:check`

## Phase 4 — Gates

- [ ] T040 `mise run web:check` green
- [ ] T041 `PYTHONPATH=src uv run pytest tests/unit/web/ -q` green
- [ ] T042 Playwright e2e (owner-run, gated on `SPEC061_TEST_SESSION_ID`)
- [ ] T043 Contradiction-snapshot contract test passes
- [ ] T044 EndgameDetector priority test passes (FR-033 order pinned)
- [ ] T045 Cross-lane flag documented for EndgameDetector docstring fix

## Phase 5 — Governance

- [ ] T050 Update `ai-docs/state.yaml`
- [ ] T051 Create ADR in `ai-docs/decisions.yaml`
- [ ] T052 Commit all work
