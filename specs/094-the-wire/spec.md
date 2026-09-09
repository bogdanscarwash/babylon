# Feature Specification: The Wire — 4-Tab Window, Deterministic Narrator

**Feature Branch**: `094-the-wire`
**Created**: 2026-07-04
**Status**: In Progress
**Program**: 09 Full-Game Build — Lane W (web product). Stacks on `e3024c8c` (093 post-fix HEAD).
**Deps**: spec-090 (Cold Collapse tokens) ✅, spec-092 (event stream / journal) ✅.

## Overview

The Wire is Babylon's narrative presentation layer — a 4-tab window that renders the
simulation's event stream through three ideological channels (Corporate / Liberated /
Intel) with synchronized euphemism highlighting, plus an Index archive, a Manufacturing
Consent patterns dashboard, and a Corpus browser.

The design canon lives in `design/mockups/wire/` (9 files). Those mockups are standalone
CDN-React JSX — reference only, not imported. This spec ports them as fresh project code
against a real `NarratorProvider` interface whose first implementation is a deterministic
template-based narrator: **same events → byte-identical output**.

Per the R-NARR ruling (project/09 §119-126), the Wire ships with the DETERMINISTIC
narrator. The Workers-AI/LoRA stack is M8/Wave-6 infrastructure. The `NarratorProvider`
interface is designed so the LLM provider drops in later without frontend changes.

### Constitution III (AI observes, never controls)

The narrator is **structurally** constrained:
- It consumes the event stream OUT-OF-TICK (reads `tick_event` rows after `resolve_tick`).
- It writes ONLY presentation content (the `WireFeed` JSON).
- It has NO access to engine state — zero `babylon.*` imports, zero DB writes, zero
  graph mutations. The module is a pure `dict → dict` function.
- Hegemony-driven visibility hooks land as no-op pass-throughs until spec-077.

## User Scenarios & Testing *(mandatory)*

### US1 — Wire renders a live 50-tick game (Priority: P1, gate)

A player navigates to `/games/:id/wire` after 50 ticks have resolved. They see the
triptych: Continental (corporate press), Free Signal (pirate radio), and Cable (SIGINT)
columns rendering the latest significant story, each in its channel's register. Hovering
a flagged euphemism in one column sync-highlights the equivalent term in the other
columns and surfaces a translation chip in the footer.

**Independent test**: Playwright, gated on a live seeded session. Backend unit test
asserts the wire feed endpoint returns a well-formed `WireFeed` from 50 events.

### US2 — Wire Index shows all stories (Priority: P1)

The INDEX tab lists every story from the event stream, filterable by severity
(critical/warning/info). Each row shows the tick, slug, three-channel headline preview,
and channel coverage marks. Clicking a row opens it in the WIRE tab.

**Independent test**: Vitest — render IndexPage with a 6-story fixture; assert all
appear; click each severity filter; assert only matching rows remain; click a row;
assert WIRE tab activates.

### US3 — Patterns dashboard audits Manufacturing Consent (Priority: P1)

The PATTERNS tab shows the five Herman/Chomsky filters with hit counts, a
manufactured-consent score, and a full euphemism table mapping corporate phrases to
their liberated equivalents with editorial intervention notes.

**Independent test**: Vitest — render PatternsPage with fixture data; assert all 5
filters render with correct hit counts; assert euphemism table has the right number of
rows; assert consent score is computed from filter hits.

### US4 — Narrator determinism (Priority: P1, gate)

The same input events MUST produce byte-identical `WireFeed` output across calls. This
is the core falsifiability improvement: the narrator is a pure function of events, not
of wall-clock time or random state.

**Independent test**: pytest — call `DeterministicNarrator.narrate(events, meta)` twice
with the same inputs; assert `json.dumps(output_a, sort_keys=True) == json.dumps(output_b,
sort_keys=True)`.

### US5 — Provider swap (Priority: P1, gate)

The `NarratorProvider` interface is honored — a mock provider can be swapped in and the
bridge + endpoint still serve a well-formed `WireFeed`.

**Independent test**: pytest — inject a `MockNarrator` (returns a canned WireFeed) into
the bridge; call `get_wire_feed`; assert the response matches the mock's output.

### US6 — Article III structural test (Priority: P1, gate)

The narrator writes NO engine state. This is enforced structurally: the narrator module
imports no `babylon.*` modules, performs no DB I/O, and accepts only plain dicts.

**Independent test**: pytest — (a) inspect `narrator.py`'s loaded modules; assert none
are in `babylon.engine`, `babylon.models`, `babylon.persistence`; (b) run the narrator;
assert no global mutable state changed.

## Requirements *(mandatory)*

- **FR-094-01**: A `NarratorProvider` Python Protocol MUST be defined in
  `web/game/narrator.py` with a single method `narrate(events: list[dict], meta: dict)
  -> dict` that is a pure function (no side effects, no engine state writes).
- **FR-094-02**: `DeterministicNarrator` MUST implement `NarratorProvider` using
  template-based narration. Same input events → byte-identical JSON output. No LLM calls.
- **FR-094-03**: `EngineBridge.get_wire_feed(session_id)` MUST read journal events via
  the existing `get_journal_dashboard` path, pass them through a `NarratorProvider`, and
  return a `WireFeed` dict matching the `contracts/wire.yaml` schema.
- **FR-094-04**: `GET /api/games/{id}/wire/` MUST serve the `WireFeed` as JSON via a new
  `game_wire` Django view, following the established `game_journal` / `game_alerts`
  pattern (session lookup → bridge call → `_envelope`).
- **FR-094-05**: The frontend `useWire(gameId)` hook MUST poll
  `GET /api/games/{id}/wire/` on a 2s interval (matching `useJournal`), exposing
  `{data, loading, error, refresh}`.
- **FR-094-06**: The Wire UI MUST render 4 tabs (WIRE triptych, INDEX, PATTERNS, CORPUS)
  using Cold Collapse tokens (spec-090). The triptych renders Continental / Liberated /
  Intel columns. Euphemism spans sync-highlight across columns on hover.
- **FR-094-07**: Euphemism mappings MUST be consistent: every `term_id` in the
  euphemisms map MUST have both `c` (corporate) and `l` (liberated) phrases. Each
  euphemism MUST reference one of the 5 Manufacturing Consent filters.
- **FR-094-08**: Hegemony visibility hooks MUST be no-op pass-throughs (all channels
  visible) until spec-077 supplies the mechanic. The interface MUST accept a
  `visibility` context dict that future specs can populate.
- **FR-094-09**: The narrator module (`web/game/narrator.py`) MUST NOT import any
  `babylon.*` modules. Article III is enforced structurally at the import level.
- **FR-094-10**: An MSW handler for `/api/games/:id/wire/` MUST exist in
  `web/frontend/src/test/handlers.ts` serving a contract-faithful WireFeed fixture.
- **FR-094-11**: A route `<Route path="wire" element={<WirePage />} />` MUST be added
  under `GameRouteShell` in `App.tsx`. A NavRail entry MUST be added.

## Success Criteria *(mandatory)*

- **SC-094-01**: `mise run web:check` green (tsc + eslint + prettier + Vitest).
- **SC-094-02**: `PYTHONPATH=src uv run pytest tests/unit/web/ -q` green, including
  narrator-determinism test, Article III structural test, euphemism-sync tests, and
  provider-swap test.
- **SC-094-03**: Narrator-determinism test passes: same events → byte-identical output.
- **SC-094-04**: Provider-swap test passes: interface honored with mock provider.
- **SC-094-05**: Article III structural test passes: narrator writes no engine state.
- **SC-094-06**: Playwright: Wire renders a live 50-tick game (owner-run, gated on
  `SPEC061_TEST_SESSION_ID`).
- **SC-094-07**: MSW contract test for `/api/games/:id/wire/` passes.

## Known Gaps (documented, not fixed here)

1. **RAG / Corpus retrieval**: The mockup's CorpusPage shows ChromaDB chunk references
   with similarity scores. The real ChromaDB RAG pipeline is not wired through the web
   bridge yet (existing `NarrativeDirector` has RAG, but that's engine-side). This spec
   renders the Corpus tab with chunk references from the narrator's template output
   (deterministic placeholder refs), not live ChromaDB queries. Live RAG is a future spec.

2. **Manufacturing Consent filter detection**: The mockup hand-crafts filter hit counts.
   The deterministic narrator derives filter hits from event type + data via a static
   rule table — it does not perform NLP source analysis. This is a known simplification;
   real filter detection would require the LLM provider (M8).

3. **Hegemony-driven visibility**: All channels are always visible. Spec-077 will
   supply the hegemony mechanic that gates channel visibility (e.g., low-hegemony players
   can't see the Cable/intel channel).
