# Tasks: The Wire — 4-Tab Window, Deterministic Narrator

**Input**: `specs/094-the-wire/{spec,plan}.md`
**Prerequisites**: spec-090 (Cold Collapse tokens) done, spec-092 (event stream) done.
**Tests**: TDD red-first per unit. `tests/unit/web/` green; Vitest green; Playwright
owner-run flow documented.

## Format: `[ID] [Story] Description`

## Phase 1 — Speckit (spec-094)

- [x] T001 Create `specs/094-the-wire/` directory.
- [x] T002 Write `spec.md` — overview, user scenarios, requirements, success criteria.
- [x] T003 Write `plan.md` — architecture, file changes, constitution gate.
- [x] T004 Write `tasks.md` — this file.
- [x] T005 Write `research.md` — mockup analysis, data shape, R-NARR compliance.
- [x] T006 Write `contracts/wire.yaml` — OpenAPI 3.1 contract for wire feed endpoint.

## Phase 2 — Backend: NarratorProvider + DeterministicNarrator (US4/US5/US6, red-first)

- [x] T007 RED: `tests/unit/web/test_narrator.py` — narrator-determinism test (same
  events to byte-identical output), Article III structural test (no `babylon.*` imports,
  no engine state writes), euphemism-sync test (every term has c+l, filter is valid),
  provider-swap test (mock provider honored).
- [x] T008 GREEN: `web/game/narrator.py` — `NarratorProvider` Protocol +
  `DeterministicNarrator` with event-type template map. Pure dict-to-dict function.
  Zero `babylon.*` imports.
- [x] T009 Backend wire feed: `EngineBridge.get_wire_feed(session_id)` + `game_wire`
  view + URL route. Tests in `test_engine_bridge.py`.

## Phase 3 — Frontend contract: wire feed MSW (US1, red-first)

- [x] T010 RED: `wire-contract.test.tsx` asserting WireFeed shape via `fetch()` — no
  MSW handler yet.
- [x] T011 GREEN: MSW handler for `/api/games/:id/wire/` + `types/wire.ts`. Contract
  test passes.

## Phase 4 — Wire UI: triptych + 4 tabs (US1/US2/US3)

- [x] T012 `useWire.ts` hook — polling GET wrapper matching `useJournal` pattern.
- [x] T013 `WireApp.tsx` + `WireWindow.tsx` — main shell with 4 tabs, tick badge.
- [x] T014 `ContinentalColumn.tsx` — corporate press column with euphemism spans,
  bibliography, superscript citations.
- [x] T015 `LiberatedColumn.tsx` — pirate-radio phosphor terminal with margin notes.
- [x] T016 `IntelColumn.tsx` — SIGINT cable with structured fields, redaction bars.
- [x] T017 `IndexPage.tsx` — story archive with severity filters, 3-column headline
  preview.
- [x] T018 `PatternsPage.tsx` — Manufacturing Consent dashboard (5 filters, consent
  score, euphemism table).
- [x] T019 `CorpusPage.tsx` — corpus browser with channel filter.
- [x] T020 `TranslationFooter.tsx` — euphemism sync footer with always-on toggle.
- [x] T021 `wire.css` — channel-specific textures (phosphor, scanlines, redaction bars).
- [x] T022 `WirePage.tsx` route wrapper + `App.tsx` route + `NavRail.tsx` entry.
- [x] T023 Component tests: triptych renders, euphemism sync-highlight, index filter,
  patterns dashboard, corpus browser.

## Phase 5 — Quality gate + Playwright

- [x] T024 `mise run web:check` green (tsc + eslint + prettier + Vitest).
- [x] T025 `PYTHONPATH=src uv run pytest tests/unit/web/ -q` green.
- [x] T026 Playwright `wire-50-tick.spec.ts` — owner-run, gated on
  `SPEC061_TEST_SESSION_ID`.

## Phase 6 — Close-out

- [x] T027 Update `ai-docs/state.yaml`.
- [x] T028 Update `project/09-program-full-game.md` section 2 spec-094 status: DONE.
- [x] T029 Final gate run + report.
