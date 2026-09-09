# spec-113 — The Living Map: Map-First Grand-Strategy UI (architecture)

Program 16. Authored 2026-07-11 by the planning agent (Fable-orchestrated, ultracode);
§0–§6 are the architect's design verbatim. §7 is the cartography addendum from Percy's
binding mid-planning clarification (real county/state borders as the visible map, hexes
as tiles, borders redrawn by revolution/collapse), which postdates §0–§6 and adjusts
lane ownership where noted. The Phase-R Design Bible
(`project/research/16-living-map/DESIGN_BIBLE.md`) skins this structure and may amend it.

## 0. Guiding architecture decision

The transformation is a **re-composition, not a rewrite**. Almost every leaf component
(DeckGLMap, EventsFeed, ActionComposer, TimeseriesChart, takeovers, ObjectivesTracker)
survives; what dies is the CSS-grid dashboard in `AppShell.tsx` that pins them into five
fixed regions. The new shell is two stacked layers:

- **Layer 0 — MapStage**: `DeckGLMap` at `absolute inset-0`, always mounted, the only
  scroll/drag surface.
- **Layer 1 — Chrome**: a `pointer-events-none` full-viewport overlay whose children
  (top bar, outliner, event tray, action dock, inspection stack) individually re-enable
  `pointer-events-auto`. Map interactions pass through the gaps.
- **Layer 2 — Takeovers**: unchanged (`TakeoverOverlay` already renders fixed over
  everything; it is already map-first-compatible).

"Structure now, skin later": all new chrome is built from one primitive
(`FloatingPanel`) that consumes only existing Cold Collapse tokens plus a small additive
set of *chrome tokens* (elevation/backdrop/blur). The Design Bible later changes token
values and FloatingPanel's internals — never call sites.

## 1. Target information architecture

### 1.1 New shell layout (`src/frontend/src/components/shell/AppShell.tsx`, rewritten)

```
┌──────────────────────────────────────────────────────────────┐
│ TopBar (floating, full-width strip): brand · tick/date ·     │
│   stat chips · alert badges · takeover buttons · SpeedControls│
├────────┬──────────────────────────────────────┬──────────────┤
│Outliner│                                      │ EventTray    │
│Overlay │            MAP (full bleed,          │ (collapsible │
│(collap-│         layer 0, always visible)     │  right rail) │
│ sible) │                                      ├──────────────┤
│        │   [InspectionStack floats here,      │ Objectives   │
│        │    anchored left-of-tray, when a     │ Tray (badge) │
│        │    selection exists]                 │              │
├────────┴───────────┬──────────────────┬───────┴──────────────┤
│ MapLensBar (bottom-│  ActionDock      │ BottomDrawer toggle  │
│ left: lens+framing)│ (verb buttons)   │ (Trends/Events)      │
└────────────────────┴──────────────────┴──────────────────────┘
```

### 1.2 Component migration map

| Existing | Destination | Fate |
|---|---|---|
| `shell/AppShell.tsx` | rewritten: 3-layer composition, no grid | rewrite |
| `shell/MapPanel.tsx` | `shell/MapStage.tsx` (full-bleed, keeps `data-testid="region-map"`, keeps sole ownership of `panels.map` mount/fetch) | rename+rewrite |
| `shell/StatusBar.tsx` | `chrome/TopBar.tsx` (keeps `region-statusbar`, `tick-value` testids) | migrate |
| `shell/StatChip.tsx` | `chrome/StatChip.tsx` — each chip becomes clickable → pushes an InspectionStack frame ("every number explains itself" starts at the top bar) | migrate+extend |
| `shell/TimeControls.tsx` | `chrome/SpeedControls.tsx` (pause/1x/2x/5x, keeps `time-status` testid) | rewrite |
| `shell/Outliner.tsx` + `OutlinerSection/Row` | `chrome/OutlinerOverlay.tsx` — collapsible floating left panel (Stellaris outliner idiom), collapsed-to-icon-rail state in `uiSlice`; keeps `region-outliner` testid | migrate |
| `shell/RightDock.tsx` | **deleted**. Its three tabs disperse: Actions → `chrome/ActionDock.tsx` (bottom-center verb bar; clicking a verb opens `ActionComposer` in a FloatingPanel); Inspector → `inspect/InspectionStack.tsx`; Objectives → `chrome/ObjectivesTray.tsx` | disperse |
| `shell/BottomStrip.tsx` | **deleted**. EventsFeed → `chrome/EventTray.tsx` (right rail, always-mounted, badge counts); TimeseriesChart → `chrome/BottomDrawer.tsx` ("Trends" drawer, keeps the always-mounted-while-hidden rule so `panels.timeseries` stays fanned out) | disperse |
| `action/ActionComposer.tsx` + VerbGrid/TargetPicker/VerbForm/ParamFields | unchanged internally; hosted by ActionDock's FloatingPanel. TargetPicker gains "pick on map" affordance later (post-Bible) | keep |
| `inspector/InspectorPanel.tsx`, `Stat.tsx`, `ConsciousnessBreakdown.tsx` | superseded by `inspect/*`; ConsciousnessBreakdown becomes an `InspectionCard` section renderer | absorb |
| `events/EventsFeed.tsx` | hosted in EventTray + BottomDrawer | keep |
| `map/DeckGLMap.tsx` + lens files | stays in `components/map/`, controls extracted (see §3) | evolve |
| `takeovers/*` (wire, dialectic, chronicle+EndStateScreen) | unchanged; chronicle auto-opens on endgame (§4.4) | keep |
| `objectives/ObjectivesTracker.tsx` | hosted in ObjectivesTray | keep |
| `bbl/*` (BblData, BblLabel, Sparkline) | becomes the number-formatting layer of InspectionStack rows | keep |

### 1.3 New primitive

`src/frontend/src/components/chrome/FloatingPanel.tsx` — props: `title`, `anchor`
(`"left" | "right" | "bottom" | "top" | "free"`), `collapsed`, `onToggle`, `width`,
`children`, `testId`. Renders concrete/rebar-bordered panel with backdrop-blur over the
map (the `bg-void/80 backdrop-blur-sm` idiom already used by DeckGLMap's control cluster
— promote it to the primitive). All chrome panels are instances. No drag in v1; the
anchor enum is the extension point for the Design Bible.

### 1.4 Store shape (uiSlice evolution, additive then subtractive)

`ui.chrome: { outlinerOpen: boolean; eventTrayOpen: boolean; objectivesOpen: boolean;
bottomDrawer: "none" | "trends" | "events"; composerOpen: boolean }` plus existing
`takeover`. `rightDockTab`/`bottomStripCollapsed`/`activeDockTab` are deleted in the
same lane that deletes their consumers.

## 2. Progressive disclosure: `InspectionStack`

### 2.1 Component system (`src/frontend/src/components/inspect/`)

- `InspectionStack.tsx` — renders the stack of frames as nested/cascaded cards over the
  map (Victoria 3 idiom: breadcrumb header, each frame offset, Escape/backdrop pops,
  "pin" keeps a frame open). Reads `inspect.stack` from the store.
- `InspectionCard.tsx` — one frame: title, kind badge, sections of `ValueRow`s,
  loading/error/no-data states (Constitution III.11 null-honesty preserved: absent value
  renders "no data", never 0).
- `ValueRow.tsx` — label + `BblData`-formatted value + optional sparkline; if the row
  carries an `InspectionRef`, it renders the "explain" affordance (chevron/underline)
  and `onClick` pushes a child frame.
- `BreakdownBar.tsx` — proportional bar for composition rows (consciousness vector,
  wealth_by_class_role); absorbs `ConsciousnessBreakdown`.
- `FormulaCard.tsx` — terminal frame for a formula: expression string, per-input rows
  (each input itself a `ValueRow`, recursively explainable), constants with provenance
  note.

### 2.2 Data model (`src/frontend/src/types/inspection.ts`)

```
InspectionRef = { kind: "hex"|"org"|"node"|"edge"|"community"|"metric"|"formula";
                 id: string; scope?: string }         // scope: "hex:<h3>"|"org:<id>"|"global"
InspectionNode = { ref; title; sections: InspectionSection[] }
InspectionSection = { label?; rows: InspectionRow[] }
InspectionRow = { label; value: number|string|null; format: BblFormat; ref?: InspectionRef;
                  composition?: {key,value,color}[] }
```

### 2.3 Store (`src/frontend/src/store/slices/inspectSlice.ts`)

`inspect: { stack: InspectionFrame[]; push(ref); pop(); popTo(i); clear(); }` where each
frame owns `{ ref, data: InspectionNode|null, loading, error }`. `mapSlice.setSelection`
reroutes its fan-out from `panels.inspector.fetchForSelection` to `inspect.reset + push`.
`panels/inspectorPanel.ts` is deleted once InspectorPanel is gone. Frame data caches by
`refKey(ref)` per tick; the worldSlice tick fan-out refetches only the top frame (stale
lower frames refetch lazily on re-focus).

### 2.4 Resolvers (`src/frontend/src/lib/inspect/resolvers.ts` + `adapters/*.ts`)

One resolver per `ref.kind`, mapping ref → endpoint → adapter → `InspectionNode`:

**Breakdowns already exposed today (no backend work):**
- `hex` → `GET /api/games/:id/hex/:h3/` (`EngineBridge.inspect_hex` — full territory
  snapshot entry: heat, biocapacity, population, etc.). Adapter marks
  heat/imperial_rent/profit_rate rows explainable via `metric` refs.
- `org` → `GET /org/:id/` — consciousness vector (composition row), OODA profile,
  vanguard resources.
- `node`/`edge` → `/node/:id/`, `/edge/:id/`.
- `metric` scope=global → `/economy/` (value_produced, rent_extracted,
  exploitation_rate, profit_rate, occ, imperial_rent_pool, wage/tribute flow totals,
  `wealth_by_class_role` composition), `/summary/`, `/contradiction/` (oppositions with
  gap/rate/principal — a ready-made breakdown list), `/trade-flows/` (per-bloc Φ
  breakdown).
- `community` → `/communities/`.

**Additive backend for formula provenance (`formula` and deep `metric` refs):**

New endpoint `GET /api/games/:id/explain/?metric=<name>&scope=<scope>`:
- `web/game/provenance.py` (new module): a static **provenance manifest**
  `METRIC_PROVENANCE: dict[str, MetricProvenance]` mapping each explainable metric
  (exploitation_rate, profit_rate, occ, imperial_rent, consciousness_drift,
  revolution_probability, acquiescence_probability, solidarity_transmission,
  legitimation_index, dependency_ratio, …) to: the `FormulaRegistry` formula name, a
  human expression string (from the formula docstring's first line, e.g.
  `"dPsi/dt = k(1 - Wc/Vc) - lambda*Psi + bifurcation"`), and per-scope input extractors
  that read the already-hydrated state/graph (same `hydrate_state` pattern every other
  bridge read uses). `inspect.signature()` over the registered callable yields input
  names; the manifest supplies the value extraction and marks which inputs are
  themselves explainable metrics vs. constants (with `babylon.formulas.constants`
  provenance).
- `web/game/api.py`: one new view `game_explain` (additive), `web/game/urls.py`: one new
  path (additive). Response: `{ metric, scope, value, formula: {name, expression, doc},
  inputs: [{name, label, value, kind: "metric"|"constant"|"state", ref?}],
  constants: [...] }`. Engine (`src/babylon/`) untouched — `FormulaRegistry.default()`
  and `list_formulas()` are already importable read-only.
- `src/frontend/src/lib/inspect/provenance.ts`: frontend mirror of which metric names
  are explainable (same single-source-of-truth pattern `lib/lens.ts` uses against
  `map_contract.py`), so `ValueRow` renders the affordance without a probe request.

Recursion is thereby uniform: entity frame → metric row → explain frame → input row
(itself a metric) → explain frame → … bottoming out at constants/state values.

## 3. Map lens framework generalization

### 3.1 Registry (`src/frontend/src/lib/lenses/`)

- `registry.ts` — `MapLensDef = { id: LensId; group: LensGroup; label; tooltip; hotkey?;
  legend: {kind:"ramp"; stops} | {kind:"categorical"; entries: {label;color}[]} |
  {kind:"none"}; toLens(): Lens; availableWhen(ctx): boolean }` with
  `LensGroup = "political" | "economic" | "social" | "tension" | "ecology"`. The
  registry is *presentation metadata over the existing `Lens` union* — `lib/lens.ts`,
  `mapLensLayers.ts`, `regionFill.ts` remain the fill engine (their tests and the token
  contract keep passing untouched). New lens ids extend the `Lens` union in
  `lib/lens.ts` only when they need new fill logic.
- `groups.ts` — Paradox-style grouping for the lens bar.

### 3.2 Lens roster (all backed by data that exists today unless starred)

| Group | Lens | Backing |
|---|---|---|
| Political | Stance, Faction (with picker), Collapse | `metadata.balkanization` (exists) |
| Tension | Heat | `{kind:"heat"}` (exists) |
| Economic | Profit rate, Exploitation rate, OCC, Imperial rent | `{kind:"metric"}` (exist) |
| Social | Population, Org presence | `{kind:"metric"}` (exist) |
| Social | Class composition* | needs additive `/map/` property (dominant `SocialRole` per hex) — additive column in `map_contract.py` `MAP_METRIC_PROPERTIES` + `_hex_feature_properties`; categorical legend |
| Ecology | Habitability | `{kind:"habitability"}` (exists) |
| Solidarity | Solidarity index* | needs additive `/map/` property (per-territory SOLIDARITY-edge density; the `/communities/` builder already walks these edges) |

Starred lenses ship `availableWhen: metadata.available_metrics.includes(...)` so the
frontend lands before/without the backend property (honest "no data", matching the
existing balkanization degradation pattern).

### 3.3 Controls unification

- `components/map/MapLensBar.tsx` — replaces `MapModeSelector` + the metric sub-select:
  grouped lens buttons (bottom-left, Paradox map-mode idiom), rendering from the
  registry; keeps `data-testid="map-mode-selector"` and `lens-mode-<id>` testids so
  `map-lens-cycling.spec.ts` survives (extend, don't rename).
- `FramingSelector` stays a separate orthogonal control (LOD ≠ lens) but mounts inside
  the same bar cluster.
- `MapLegend` v2 renders from `MapLensDef.legend` — gains categorical swatch mode
  (stance/faction/collapse/class finally get a real legend instead of a text chip).
- `DeckGLMap.tsx` sheds its embedded control cluster: controls move to a sibling
  `components/map/MapControls.tsx` that `MapStage` composes next to `DeckGLMap`.
  DeckGLMap becomes pure canvas + tooltips (props unchanged otherwise — its interface is
  the Lane B/Lane A contract).
- Q/E lens cycling (`useLensCycleShortcut`) re-targets the registry order.

## 4. Game chrome details

### 4.1 Speed controls (`chrome/SpeedControls.tsx` + `timeSlice` extension)

Extend `timeSlice` with `speed: 1 | 2 | 5` and `setSpeed()`. Semantics: the
strictly-serialized resolve loop is preserved exactly; speed only sets the
**inter-resolve delay** injected in `settleAfterResolve` before recursing
(`BASE_DELAY_MS / speed`, where 5x ≈ 0 delay, i.e. current behavior). Pause stays the
existing `playIntent` mechanism; `step` stays. UI: `⏸ ▶1 ▶▶2 ▶▶▶5` cluster + status text
(keeps `time-status`, adds `speed-<n>` testids). Spacebar toggles pause/last-speed.
Autopause and error states unchanged.

### 4.2 Event popups and tray

- `chrome/EventToasts.tsx` — transient toast queue for `important` events on tick
  advance. Source: a small `store/slices/eventsSlice.ts` that
  `worldSlice.onTickAdvanced` feeds with `classifyEvents(snap.events)` output
  (worldSlice change is one line calling `events.ingest(...)`).
- `chrome/CriticalEventModal.tsx` — Paradox-style modal rendered whenever
  `time.status === "autopaused"`: shows the `autopauseEventIds` events (from the last
  snapshot), CTA buttons "Open Wire" (`ui.openTakeover("wire")` — the wire index already
  carries severity) and "Resume". This gives the existing autopause machinery its
  missing face.
- `chrome/EventTray.tsx` — persistent right rail hosting `EventsFeed`; header badges
  mirror `summary.event_counts` (same data StatusBar shows today); TopBar alert badges
  click-open the tray.

### 4.3 Alert badges

Stay sourced from `panels.summary.event_counts` (real endpoint, already fetched by
TopBar's predecessor). Badge → `ui.chrome.eventTrayOpen = true`.

### 4.4 Endgame integration

**Correction (ds-sync NOTES + owner item 37):** `GameSnapshot.endgame`
(`types/game.ts:44-56`) is a DEAD field with zero readers — the real endgame path is
`panels.endgame` (reading `types/dialectic.ts`). Lane E keys the auto-open off the
`panels.endgame` fan-out instead: when its fetched state transitions null → non-null,
`time.pause()` + `ui.openTakeover("chronicle")`. (Optionally retire the dead field in
Lane G's sweep — resolves owner item 37.) `ChronicleTakeover`/`EndStateScreen` already
render from `panels.endgame` — zero changes inside takeovers.

## 5. Lane decomposition (parallel Sonnet agents, git worktree)

Ownership is file-exclusive; interfaces between lanes are frozen in Lane A. Merge order:
**Carto (§7) and A first, then B–F fully parallel, G last.**

### Lane A — Shell skeleton & contracts (MERGE FIRST, blocking)
**Owns:** `components/shell/AppShell.tsx`, `components/shell/MapStage.tsx` (new),
deletion of `MapPanel.tsx`/`RightDock.tsx`/`BottomStrip.tsx`/`StatusBar.tsx` (contents
dispersed as stubs), `components/chrome/FloatingPanel.tsx`, `components/chrome/TopBar.tsx`,
**stub files** for `OutlinerOverlay.tsx`, `EventTray.tsx`, `ActionDock.tsx`,
`ObjectivesTray.tsx`, `BottomDrawer.tsx`, `EventToasts.tsx`, `CriticalEventModal.tsx`,
`SpeedControls.tsx` (each stub renders its legacy content or placeholder, carries the
frozen props interface `{gameId}` + testid), `store/slices/uiSlice.ts`,
`types/inspection.ts`, `index.css` additive chrome tokens, shell tests.
**Key rule:** after A merges, A never touches the stub files again — each is handed to
its lane.
**Tests:** rewrite `AppShell.test.tsx` (three layers mounted, map full-bleed, chrome
pointer-events); keep `region-*` testids so `real-loop.spec.ts` passes unmodified
against the skeleton.

### Lane B — Lens framework (parallel)
**Owns:** everything in `components/map/` (DeckGLMap control extraction →
`MapControls.tsx`, `MapLensBar.tsx`, `MapLegend.tsx` v2, `MapModeSelector`/
`FramingSelector` migration/deletion), `lib/lens.ts`, `lib/lenses/*` (new),
`lib/regionFill.ts`, `store/orchestrator.ts` (lens-cycle hotkeys).
**Interface:** DeckGLMap props unchanged; `MapStage` (Lane A) mounts `<DeckGLMap/>` +
`<MapControls/>` — MapControls stub created by A.
**Tests:** vitest for registry (`availableWhen` degradation, group ordering, legend
metadata), keep all existing `mapLensLayers.test.ts`/`regionFill.test.ts` green; keep
`lens-mode-*` testids.

### Lane C — InspectionStack frontend (parallel)
**Owns:** `components/inspect/*` (new), `store/slices/inspectSlice.ts` (new),
`store/slices/mapSlice.ts` (selection rerouting), `store/slices/panels/inspectorPanel.ts`
(delete), `lib/inspect/*` (new: resolvers, adapters, provenance mirror),
`components/inspector/*` (absorb/delete), `lib/inspectorFields.ts`/`inspectorMapping.ts`
(absorb).
**Interface:** consumes `types/inspection.ts` (Lane A); consumes the `explain` endpoint
contract behind a resolver that degrades to "no provenance" until Lane D merges — C and
D are decoupled by the frozen JSON contract.
**Tests:** MSW-fixture vitest per resolver/adapter; recursion depth test (metric →
formula → input-metric); null-honesty tests ported from `InspectorPanel.test.tsx`.

### Lane D — Provenance & map-property backend (parallel, backend-only)
**Owns:** `web/game/provenance.py` (new), `web/game/api.py` (one additive view),
`web/game/urls.py` (one additive path), `web/game/map_contract.py` + the `/map/`
feature-property builder in `engine_bridge.py` (two additive properties:
`dominant_class`, `solidarity_index`), `web/game/stub_bridge.py` (matching stubs),
`web/game/tests/test_provenance.py` (new).
**Constraint:** additive-read-only against `src/babylon/` (imports `FormulaRegistry`,
`formulas.constants`; zero engine edits); existing endpoint payloads must stay
byte-identical minus the two new map properties — verify `qa:regression` and decide
whether map properties need a query-param gate (`?include=class,solidarity`) if
regression pins `/map/` bytes.
**Tests:** contract test that every `METRIC_PROVENANCE` entry names a registered formula
and that extractor input names match `inspect.signature` of the callable.

### Lane E — Time, events, endgame chrome (parallel)
**Owns:** `store/slices/timeSlice.ts` (speed), `store/slices/worldSlice.ts` (events
ingest + endgame auto-open), `store/slices/eventsSlice.ts` (new),
`lib/eventClassifier.ts` (extend map coverage of the 79 EventTypes), chrome stubs:
`SpeedControls.tsx`, `EventToasts.tsx`, `CriticalEventModal.tsx`, `EventTray.tsx`,
`BottomDrawer.tsx`, plus `components/events/EventsFeed.tsx`,
`components/timeseries/TimeseriesChart.tsx` (host adjustments only).
**Tests:** timeSlice speed-delay state-machine vitest (pause during delay, speed change
mid-loop); autopaused → modal render; endgame transition → `openTakeover("chronicle")`
exactly once.

### Lane F — Outliner, action dock, objectives (parallel)
**Owns:** chrome stubs `OutlinerOverlay.tsx`, `ActionDock.tsx`, `ObjectivesTray.tsx`;
`components/shell/Outliner*.tsx` (migrate/delete), `components/action/*` (hosting
changes only — composer internals frozen), `components/objectives/*`.
**Interface:** selection writes still go through `map.setSelection` (Lane C owns the
slice; F only calls the existing frozen signature).
**Tests:** collapse/expand state, verb-dock → composer FloatingPanel open, keeps
`verb-grid`/`target-picker`/`action-composer`/`pending-actions` testids (verb-submit
e2e untouched).

### Lane G — Test consolidation & design-sync (LAST)
**Owns:** `e2e/*.spec.ts` (deliberate updates: end-turn-flow gains speed controls; new
`inspection-stack.spec.ts`, `event-popup.spec.ts`), `theme/tokens.contract.test.ts` +
`type-roles.contract.test.ts` (deliberate additive chrome-token pinning — never delete
canon assertions), `.design-sync/` manifest updates for the new chrome components,
dead-file sweep.

## 6. Risk register

| Risk | Where | Mitigation |
|---|---|---|
| **e2e breakage on testids** — `real-loop`/`end-turn-flow`/`map-lens-cycling` pin `region-*`, `tick-value`, `time-status`, `lens-mode-*`, `framing-*`, `map-mode-selector` | Lanes A, B, E | Testids are contract: carried onto successor components in the owning lane; only Lane G may retire one, updating the spec in the same commit |
| **Token-contract churn** — `tokens.contract.test.ts` regex-pins canon tokens in `index.css` | Lane A | Chrome tokens are strictly additive; the contract file itself is Lane G-owned; canon values never edited |
| **qa:regression parity** — backend must stay byte-identical | Lane D | Only additive view/path/module; the two new `/map/` properties are the one parity risk — gate behind opt-in query param if the regression suite pins `/map/` bytes; everything else read-only |
| **deck.gl perf at state/national zoom** — `H3ClusterLayer` synthesizes polygons from `member_h3` client-side; national scale = 10⁴–10⁵ hexes/feature; per-render `computeFillDomain` scans + `updateTriggers: [lens, domain]` object identities force GPU re-uploads | Lane B | Use `lensKey(lens)` (already exists) + scalar domain tuple in updateTriggers; memoize member_h3 → polygon by feature id across ticks (geometry is tick-invariant, only properties change); keep aggregated `?zoom=` fetches server-side as today. §7 removes most polygon synthesis entirely (static county TopoJSON) |
| **Store churn re-rendering the map** — chrome panels toggling `ui.*` must never invalidate `MapStage` | Lanes A, C, E | Selector discipline (narrow zustand selectors, no `s.ui` object subscriptions); MapStage subscribes only to `world.snapshot`, `panels.map.data`, `map.*`; add a render-count vitest like the existing `DeckGLMap.stability.test.tsx` |
| **Inspector fan-out regression** — `mapSlice.setSelection` currently drives `panels.inspector`; three call sites (map click, outliner, table views) | Lane C | inspectSlice keeps the same `setSelection` signature; delete `inspectorPanel.ts` only after all consumers are in `inspect/*` |
| **Resolve-loop regressions from speed** — `timeSlice` is the most delicate state machine in the app (409 handling, autopause races) | Lane E | Speed is delay-only, injected at exactly one point (`settleAfterResolve`); every existing timeSlice test must pass unmodified plus new delay tests |
| **Aesthetic decisions landing late** | all | All chrome styling flows through FloatingPanel + tokens; Design Bible lands as a token/`FloatingPanel`-internals patch, not call-site churn |
| **Stub-file ownership drift** — two lanes editing one chrome file | A→B–F | Ownership manifest in the worktree README; Lane A stops touching stubs at merge; CI check: no two lanes' PRs modify the same path |

## 7. Cartography addendum (Lane Carto — merges before Lane B)

**Binding requirement (Percy, 2026-07-11):** the visible map is real political
cartography, not abstract hexes. De jure = colonial county borders aggregated into
states (the starting map). De facto = polity fill whose borders REDRAW as counties
change hands through revolution, liberation movements, political collapse, instability.
Hexes are tiles: they surface at deep zoom and as contested-frontline shading inside a
county, never as the default look. Constitution mapping: immutable spatial substrate =
county cartography; political claims = overlays.

**Lane Carto owns** (disjoint from Lane B, which consumes its exports):
- `tools/build_geo_assets.py` (or mise task): TIGER county shapefiles
  (`/media/user/data/babylon-data/tiger/`) → simplified TopoJSON keyed by 5-digit FIPS
  (mapshaper/topojson pipeline; tolerance tuned so national view ≤ ~2 MB), a state-level
  dissolve, and natural-earth context layers. Vintage/projection/count verified by the
  Phase-R data survey.
- `src/frontend/public/geo/*` — checked-in generated assets (counties.topojson,
  states.topojson, context layers) + provenance README.
- `src/frontend/src/lib/geo/*` — TopoJSON load/parse, `feature()` extraction,
  `mergePolity(fipsList) → GeoJSON` via topojson-client shared-arc merge (clean
  dissolves, no slivers), memoized per polity-membership hash. Vitest fixture: county
  set → expected merged ring count.
- `src/frontend/src/components/map/layers/political.ts` — deck.gl GeoJsonLayer stack:
  county hairlines, state borders, de-facto polity fills + thick dynamic borders; hex
  layers demoted to deep-zoom/frontline overlays. (File placed in Lane B territory but
  authored by Carto BEFORE B branches — B consumes it as a frozen export, same pattern
  as the A-stub rule.)
- Basemap: replace Carto Dark Matter with a minimal self-authored MapLibre style (mute
  land/water only — the political layer IS the map). Removes the external tile
  dependency.
- New deps: `topojson-client` (+ types). Pipeline dev-dep: mapshaper (tools venv or
  npx, not a frontend dep).

**Data join:** county polygon ⇄ game state via `county_fips` (first-class in
`web/game/models.py` TerritorySnapshot; `?zoom=county` aggregates already flow).
Polity membership (which counties belong to which sovereign/claim) comes from the
balkanization/sovereignty metadata that already reaches the client (spec-070 lenses);
de-facto border redraw is pure client-side merge — no backend change.
