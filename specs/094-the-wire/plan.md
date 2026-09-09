# Implementation Plan: The Wire — 4-Tab Window, Deterministic Narrator

**Branch**: `094-the-wire` (off `e3024c8c`) | **Spec**: `specs/094-the-wire/spec.md`
**Program**: 09 Full-Game Build — Lane W. Kit refs: `project/09-program-full-game.md` §2
(spec-094), §119-126 (R-NARR ruling).

## Summary

Port the 9-file `design/mockups/wire/` design canon into a real 4-tab presentation window
fed by the event stream (spec-092). Introduce a `NarratorProvider` interface (Python
Protocol + TS interface) whose first implementation is a deterministic template-based
narrator: same events → byte-identical output. No LLM calls (R-NARR). The narrator is
structurally isolated from the engine (Article III): zero `babylon.*` imports, pure
`dict → dict` function.

## Technical Context

**Language**: Python 3.12 (backend), TypeScript 5.7 (frontend).
**Stack**: Django 5.x + DRF, React 19 + Zustand 5 + Vite 6 + Vitest + Playwright — all
already installed.
**Constraints**: `mise run web:check` green; `PYTHONPATH=src uv run pytest
tests/unit/web/` green; no engine dynamics changed (presentation only).
**Scope of ownership (Lane W)**: `web/**` + `specs/094-the-wire/**` + `tests/unit/web/`.
MUST NOT touch `src/babylon/**` (engine code, READ ONLY).

## Constitution Check

*GATE: Must pass before implementation. Constitution v2.7.0.*

| Gate | Requirement | This feature | Status |
|------|-------------|--------------|--------|
| **III (AI observes)** | Narrator consumes events out-of-tick, writes only presentation | Narrator reads `tick_event` rows (already written by spec-092's `resolve_tick`); produces `WireFeed` JSON; zero `babylon.*` imports; no DB writes. | PASS |
| **III.7 Determinism** | Same inputs → same outputs | `DeterministicNarrator.narrate()` is a pure function of events + meta. No RNG, no wall-clock, no global mutable state. Byte-identical output test is the gate. | PASS |
| **III.1 No Magic Numbers** | Constants trace to grounded source | Template strings trace to event-type semantics. Filter hit rules trace to Herman/Chomsky's 5-filter model (1988). Named constants, not bare literals. | PASS |
| **VII Color=meaning** | No decorative glow | Wire CSS uses Cold Collapse tokens (spec-090). Channel colors: cadre (corporate), solidarity (liberated), rupture (intel) — each carries semantic meaning. | PASS |
| **VIII.9 Hyperedge rendering** | Hyperedges never pairwise | The Wire does not render community/hyperedge topology. Corpus tab shows chunk references (strings), not graph structures. | N/A |
| **R-NARR** | Deterministic narrator first | No LLM calls. `NarratorProvider` interface designed for future LLM drop-in (M8). | PASS |

**Gate resolution**: No conflicts. This is presentation-layer work over already-committed
engine output (spec-092's `tick_event` table). No dynamics, no engine state writes.

## Architecture

### Data Flow

```
tick_event table (written by resolve_tick, spec-092)
  → query_session_events(session_id, limit=200)  [spec-092]
  → EngineBridge.get_journal_dashboard(session_id)  [spec-092]
  → DeterministicNarrator.narrate(events, meta)     [NEW: web/game/narrator.py]
  → EngineBridge.get_wire_feed(session_id)          [NEW: web/game/engine_bridge.py]
  → game_wire Django view                            [NEW: web/game/api.py]
  → GET /api/games/{id}/wire/                        [NEW: web/game/urls.py]
  → useWire hook polls                               [NEW: web/frontend/src/hooks/useWire.ts]
  → Wire components render                            [NEW: web/frontend/src/components/wire/]
```

### NarratorProvider Interface (Python Protocol)

```python
# web/game/narrator.py
from typing import Protocol

class NarratorProvider(Protocol):
    def narrate(
        self,
        events: list[dict[str, Any]],
        meta: dict[str, Any],
        visibility: dict[str, Any] | None = None,  # no-op pass-through (spec-077)
    ) -> dict[str, Any]:
        """Produce a WireFeed from GameEvents. PURE FUNCTION."""
        ...
```

### DeterministicNarrator

Template-based: maps event types to pre-written copy templates with euphemism spans.
Each template produces:
- `slug`, `hed:{c,l,i}`, `severity` for the index
- `continental`/`liberated`/`intel` channel bodies with euphemism spans
- `euphemisms` map (term_id → {c, l, filter, note})
- `filters` (5 Manufacturing Consent filters with hit counts)

The narrator is a pure function. It uses:
- `_EVENT_TEMPLATES: dict[str, EventTemplate]` — static map from event type to template
- `_classify_severity(event_type)` — maps to critical/warning/info
- `_build_index(events)` — builds the story index from events
- `_build_story(event, template)` — builds the full triptych story
- `_compute_filters(events)` — computes Manufacturing Consent filter hits

### WireFeed Shape (from wire-data.jsx)

```typescript
interface WireFeed {
  meta: {
    tick: number;
    session: string;
    operator: string;
    freq: string;
    qth: string;
    classification: string;
    cable_id: string;
    page_of: string;
    timestamp_utc: string;
  };
  index: WireStoryIndex[];
  euphemisms: Record<string, EuphemismEntry>;
  story: WireStory | null;  // active story (latest critical/warning, or latest)
  filters: ManufacturingConsentFilter[];
}
```

## Project Structure — touched files

```
# Backend
web/game/narrator.py                              # NEW — NarratorProvider protocol + DeterministicNarrator
web/game/engine_bridge.py                         # + get_wire_feed(session_id)
web/game/api.py                                    # + game_wire view
web/game/urls.py                                   # + /wire/ route

# Frontend types + hook
web/frontend/src/types/wire.ts                     # NEW — WireFeed, WireStory, Euphemism types
web/frontend/src/hooks/useWire.ts                  # NEW — polling hook

# Frontend components (ported from mockups, fresh code)
web/frontend/src/components/wire/WireApp.tsx       # NEW — main shell, 4 tabs
web/frontend/src/components/wire/WireWindow.tsx     # NEW — app chrome
web/frontend/src/components/wire/ContinentalColumn.tsx  # NEW
web/frontend/src/components/wire/LiberatedColumn.tsx    # NEW
web/frontend/src/components/wire/IntelColumn.tsx        # NEW
web/frontend/src/components/wire/IndexPage.tsx          # NEW — story archive
web/frontend/src/components/wire/PatternsPage.tsx       # NEW — Mfg Consent dashboard
web/frontend/src/components/wire/CorpusPage.tsx         # NEW — corpus browser
web/frontend/src/components/wire/TranslationFooter.tsx # NEW — euphemism sync
web/frontend/src/components/wire/wire.css              # NEW — channel textures

# Route + nav
web/frontend/src/components/pages/WirePage.tsx     # NEW — route wrapper
web/frontend/src/App.tsx                           # + /wire route
web/frontend/src/components/layout/NavRail.tsx      # + Wire entry

# MSW + tests
web/frontend/src/test/handlers.ts                  # + /wire/ handler
web/frontend/src/__tests__/integration/wire-contract.test.tsx  # NEW
web/frontend/src/components/wire/__tests__/*.test.tsx           # NEW

# Backend tests
tests/unit/web/test_narrator.py                     # NEW — determinism, Article III, euphemism sync, provider swap
tests/unit/web/test_engine_bridge.py               # + get_wire_feed tests

# Spec
specs/094-the-wire/{spec,plan,tasks,research}.md   # NEW
specs/094-the-wire/contracts/wire.yaml              # NEW

# E2E (owner-run)
web/frontend/e2e/wire-50-tick.spec.ts              # NEW, gated on SPEC061_TEST_SESSION_ID
```

## Phased Approach (each phase = one commit, TDD red-first)

1. **RED**: narrator-determinism test + Article III test + euphemism-sync test in
   `tests/unit/web/test_narrator.py`. Confirmed RED (module doesn't exist).
2. **GREEN**: `web/game/narrator.py` — `NarratorProvider` protocol +
   `DeterministicNarrator` with event-type templates. All narrator tests pass.
3. **Backend wire feed**: `get_wire_feed` bridge method + `game_wire` view + URL route.
   Backend tests green.
4. **Frontend contract RED**: `wire-contract.test.tsx` against unmocked route.
5. **Frontend contract GREEN**: MSW handler + WireFeed types. Contract test passes.
6. **Wire UI**: Port all components from mockups (fresh code, Cold Collapse tokens).
   Component tests for each tab.
7. **Route + nav**: `WirePage` route, NavRail entry. App test updated.
8. **Quality gate**: `mise run web:check` + `pytest tests/unit/web/` green.
9. **Playwright**: `wire-50-tick.spec.ts` (owner-run, gated).

## Complexity Tracking

| Divergence from the mockup | Why unavoidable | Resolution |
|---|---|---|
| Mockup hand-crafts a single rich story (WCLF raid) with 5 paragraphs/channel | Real events are simpler; the narrator maps event types to templates | Templates produce 1-3 paragraphs/channel depending on event data richness |
| Mockup's CorpusPage shows live ChromaDB chunks with similarity scores | ChromaDB RAG not wired through web bridge | Corpus tab renders narrator-produced chunk refs (deterministic placeholders); live RAG is future spec |
| Mockup's filter hit counts are hand-authored | Real filter detection needs NLP (LLM) | Static rule table per event type; known simplification documented in Known Gaps |
| Mockup uses CDN React + Babel standalone | Project uses Vite + React 19 | Fresh code, not imported; mockup is reference only |
