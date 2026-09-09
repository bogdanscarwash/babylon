# spec-113 integration ledger (orchestrator-owned wiring between waves)

Cross-lane seams deliberately left to the orchestrator so concurrent lanes stay
file-disjoint. Checked off as done; anything unchecked at program end becomes an
owner item.

## After Wave 2

- [x] Lane C unwraps the explain envelope — ExplainResponse types mirror
      `_explain_result_to_dict` + MSW fixtures; live payload confirmed 2026-07-11 vs
      session 5ad0c6ae (exploitation_rate 0.0 @ tick 0, recursive
      value_extraction_ratio ref).
- [x] `narrationPanel` registered (TAKEOVER_PANEL_KEYS + PanelsSlice) + `useNarration`
      hook + EventTray narrator strip — commit `0cc2f38a`.
- [x] `NarrationBlock` mounts: EventTray ✓; EndStateScreen epitaph (scope=endgame) +
      WireApp strip (scope=tick) ✓ Wave 3 SKIN-MENUS; county InspectionCard section →
      deferred (needs fips↔beat adapter, queue for Phase D/owner).
- [ ] Three-channel critical events (bible §5.2): map-anchored cue for criticals —
      post-Wave-3 polish (event→geo ref data exists; cue layer is map-side).
- [x] `inspectSlice` tick refetch: C used `api.subscribe` from its own slice file;
      unit-tested.
- [x] mapSlice default-framing flip residue: MapStage test fixed by orchestrator in
      the wave-2 commit; C decoupled mapSlice.test from B's DEFAULT_LENS.
- [~] `region-dock` / `region-bottomstrip` placements: real-loop rewritten (Lane G,
      Wave 3) — live browser validation still Phase V.
- [x] TopBar StatChip metric props: wired to real provenance keys (profit_rate,
      imperial_rent); Pop stays non-clickable (no manifest key, III.11).
- [x] `lib/inspectorMapping.ts` absorbed into EventsFeed (Lane G); org→org branch
      coverage restored by orchestrator (institution/hyperedge rows statically
      enforced, unreachable via classifyEvents today).

## After Wave 3 (orchestrator residue — all committed in `04c3c93f`)

- [x] SKIN-MENUS literal-hex fallbacks → landed `--ksbc-*` tokens
      (LoginRoute/LobbyRoute/TakeoverOverlay hold no literal hex).
- [x] TimeControls seam closed: reskinned to installerKit grammar;
      `keyButtonUrgentClass` added (crimson Resume, never gold).
- [x] event-popup.spec.ts added to `AUTHENTICATED_SPECS`.
- [x] Dead `Selection`/`InspectionFrame` barrel re-exports removed (Lane G finding).
- [x] `FormEvent` → `React.SubmitEvent` (deprecated in current React types).

### Flagged for owner review (Wave-3 lane judgment calls)

- `--ksbc-plate` #2a0d0d is DERIVED (field + one step toward crimson), not a literal
  Kitty-palette value — swap if an exact ksbc value is preferred.
- Map lens/framing buttons got selection grammar + lean chips, NOT the full chunky
  key-button shadow (SKIN-CHROME judged ~10 packed buttons would clutter; bible item 7
  said "styling + selection grammar only"). Revisit if the full treatment was intended.
- TakeoverOverlay title tabs say "Wire Dispatch"/"Chronicle"/"Dialectic" — deliberately
  distinct from internal titles ("THE WIRE") to avoid duplicate-text collisions.

### Phase V results (2026-07-11 evening — commits `2eae6d5a`, `a49c5598`)

Live e2e vs real Django+engine: **27/30 green**. The three reds:
- 9-lens cycle: **was NOT load-flaky — a real structural defect** (see the
  Wave-4-close update below; FIXED in `704eb9a9`).
- end-turn spacebar + event-popup: PINNED ENGINE DEFECTS (red by design, see
  owner items below).

Root causes found & fixed en route (full detail: HANDOFF-PHASE-V.md):
tailwind-scan spin on vendored TopoJSON (`@source not`), SwiftShader picking
wedge (GPU launch flags), chrome z-strata (lens bar over TopBar, drawer over
composer Submit), hex clicks passing row-ids not h3, Escape double-pop
(effect re-subscription mid-dispatch — jsdom can't see it), formula-card
testid was never a metric discriminator.

### Owner items from Phase V (engine-side, charter forbids fixing here)

- **P1: UniqueViolation `ux_simulation_event_session_tick_natural`** — two
  same-tick events serialize as `event_type=UNKNOWN` + empty entity →
  natural-key collision → resolve 500 → session dies mid-play (~tick 17-18,
  wayne_county, reproduced twice). `postgres_runtime/_legacy.py:2344`.
- **P1: events reach the web layer as UNKNOWN type** → nothing classifies
  urgent → zero toasts in 20 live ticks → the game never speaks (event-popup
  spec pins this; ties into owner item 25's static early economy).
- `engine_bridge.get_inspector_hex` returns `{}` (stub) — hex InspectionCards
  render honest nulls live; when implemented, note event deep-links push
  TERRITORY ids at the h3-keyed endpoint (id↔h3 mapping needed).

### Wave 4 + Phase-V close (2026-07-11 late — commits `67caa3ab`, `704eb9a9`, `50ceda02`)

Wave-4 fan-out was stopped mid-cleanup (4 Opus/Sonnet lanes thrashing on
`tsc --noEmit` — each saw the OTHER lanes' in-progress files in the shared
worktree and could never reach a clean tree; the classic contention hazard).
Orchestrator finished all four inline (the code was ~90% there, only 9 real
tsc errors + lint):
- **PULSE** (`67caa3ab`): critical-event map pulse — one-shot crimson ring per
  new critical w/ resolvable geography, wired above the base map, stable-empty
  at rest (stability contract intact), reduced-motion static ring.
- **STRIPE**: TRUE diagonal-hatch contested fill via FillStyleExtension + a
  hand-rolled regenerable PNG atlas (no binary asset); claim-redraw seam on
  polity membership deltas.
- **DELTA**: verb predicted-delta ▲/▼ chips in VerbForm pre-commit, honest-null.
- **DS-SYNC** (`50ceda02`): barrel + 19 previews + config (49→68 srcMap, 42→54
  overrides), verified by a real esbuild bundle. NOT uploaded to claude.ai/design
  yet — that's the interactive Phase-D step.

**Structural fix — chrome layout SoT (`704eb9a9`).** The 9-lens red was a REAL
z-strata pointer-interception, not load-flakiness: the grouped lens bar
(flex-wrap) was anchored only by its right edge, so its wrapping row extended
LEFT under the outliner rail (measured live: button at x:87 under the outliner
header). Owner-approved fix: new `chrome/layout.ts` single source of truth —
rail widths declared once, every map-control offset DERIVES from it, controls
bounded to a `MAP_SAFE_*` inter-rail box so a wide control can't reach a rail
by construction. Consumers rewired: MapControls (right-anchored + max-width
cap), OutlinerOverlay/EventTray/ObjectivesTray/InspectionStack. Guard: new
MapControls unit test + the live 9-lens e2e (1-fail-90s-timeout → 4-pass-11.5s).

**Final live e2e: 28/30.** The only 2 reds are the engine-pinned owner items
below (red by design). 796 vitest, tsc/eslint/prettier green.

**Polish fixes (`79ba9bf0`):**
- ✅ FIXED `BottomDrawer`→`TimeseriesChart` 0-height: `h-full`→`h-48` gives the
  recharts ResponsiveContainer a definite parent (Trends chart now renders,
  confirmed on live data).
- ✅ FIXED `KeyHints` redundancy: it rendered in SIX hosts (Outliner, EventTray,
  BottomDrawer, ObjectivesTray, ActionDock, CriticalEventModal) → kept only the
  Outliner (persistent home) + CriticalEventModal (exclusive take-over dialog).

**Dep-compat + toolchain pre-alignment (`2077db80`):** ultracode audit (empirical
isolated build + 7 analysts) → branch FULLY COMPATIBLE with dev's wave; adopted
dev's exact toolchain (vite 8, eslint 10, plugin-react 6, ts 5.9, prettier 3.9.5,
@tailwindcss/vite 4.3) + reconciled 7 union-type files + pre-commit prettier pin.
Verified: vite-8 build OK, dev server /game 14ms, 796 vitest green, @source-not
scanner fix holds. Engine determinism inherits dev's byte-identical qa:regression.

### Live-state audit (2026-07-11 evening, real tick-4 Wayne County session)

Captured every major state against the LIVE engine (fresh admin login → real
Django/engine; shots in `output/demos/spec-113-living-map/`, gallery published):
- **Map** (county + HEX framing): real data — Rent Φ 118.94, 4 factions w/ live
  colonial stances, member-hex hull over TIGER borders, live imperial-rent trend.
- **InspectionStack drill**: WORKS — Victoria-style breadcrumb; the `/explain/`
  provenance is a highlight (honest formula + reason-for-null, III.11).
- **DIALECTIC takeover**: real data (tenant⇄rent principal contradiction,
  tension 1.00 → reproduction). Polished.
- **WIRE / CHRONICLE takeovers**: structurally complete, honest-empty (narrator
  off + events UNKNOWN → no wire stories; no terminal outcome at tick 4).

**New owner items from the live audit:**
- **P2 (data-source, engine-adjacent):** the hex `InspectionCard` reads all
  "no data" (County/Population/Habitability/Biocapacity/Heat/Rent/Dominant Class)
  because it sources the stubbed `get_inspector_hex` (`{}`), while the hover
  `HexTooltip` shows the SAME hex's real values from the map snapshot (Heat 0.00,
  Rent 3.50, Pop 8000, Biocapacity 80, Sector residential). Same hex, two
  readouts. Owner call: (a) implement engine `get_inspector_hex`, or (b) wire the
  card to source the map-feature data the tooltip already holds. NOT rewired
  unilaterally (inspection data-architecture decision).
- **Minor:** an open InspectionStack card overlaps the lens bar's rightmost group
  (both at the safe-area right edge) — transient, lens bar still usable.

**Phase D remaining:** upload the 19 registered previews to the "Babylon Cockpit"
design project (interactive); narrator backend (`BABYLON_LLM_NARRATOR`) for wire
depth; national-run choropleth for a richer map fill.

NOTE: the "Juice Pass" inventory below predates DESIGN_BIBLE §9b (The Installer,
owner ruling) — §9b's re-aim SUPERSEDES the gradient/glow items; the performance
budget remains binding law.

## Phase D (design/reskin) queue

### The Juice Pass (owner directive 2026-07-11: "really fancy Tailwind + effects,
### as long as they don't bog down performance — make it feel like a **game**")

Effects inventory (Tailwind v4 + custom keyframes; extend the takeovers'
phosphor/scanline language into all chrome):
- Panel chrome: gradient hairline borders (conic accents at corners), layered
  box-shadow glow tokens (cyan ambient + red urgency), backdrop-blur + saturate
  on floating panels, subtle noise texture via data-URI background.
- Text: phosphor bloom (text-shadow token) on display-register numerals; wire
  headlines get a one-shot CRT-reveal on arrival.
- Motion (@keyframes): tick pulse on the time cluster (every resolve);
  toast slam-in + settle; critical-alert throb (border/glow only); claim-redraw
  shimmer along the new border path (≥600ms, cause-named per bible §2.2);
  legend flash on domain change; endgame takeover iris/fade.
- Map chrome: vignette + scanline mask on takeover overlays only (never the
  live map canvas); selection halo pulse; hover glow signifiers (Norman).

PERFORMANCE BUDGET (hard rules):
- Compositor-only animation: transform + opacity ONLY on persistent/looping
  animations; no animated filter/backdrop-filter/box-shadow loops on large
  surfaces; one-shot entrance effects may animate shadow/filter.
- backdrop-blur is expensive over a live WebGL canvas: small panels only
  (≤ ~25% viewport each), never full-screen scrims while unpaused.
- `prefers-reduced-motion` honored globally (all loops off, one-shots reduced
  to fades) + a settings kill-switch.
- Nothing animates on the deck.gl render path itself except deck-native layer
  transitions; DeckGLMap.stability render-count test must stay green.
- Verify: Chrome DevTools performance profile on the wayne_county session —
  no long tasks from CSS, 60fps pan/zoom preserved with all effects on.

- [ ] Extend takeover diegetic language into shell chrome (audit: two visual
      languages; reskin = extend the first into the second).
- [ ] OrgSelect + faction `<select>` restyle in-register (bible §9.6).
- [ ] TimeseriesChart de-chart-ification toward BblData idiom (bible §9.7).
- [ ] BottomDrawer bottom-right toggle per bible layout (Lane A shipped full-width
      footer as structure-now placeholder).
- [ ] Contested-claim TRUE striping (FillStyleExtension pattern) if Lane B shipped
      dash-only.
- [ ] Empty-state copy sweep — every surface in-register (Lanes E/F purge their own;
      sweep the rest).
- [ ] ds-sync: new chrome components → barrel + componentSrcMap; re-sync to
      "Babylon Cockpit" project; grade renders.

## Vision-doc audit (owner re-affirmed the end-state doc 2026-07-11; gaps enumerated)

Checked `desired-end-state-for-ui.md` clause-by-clause against the Wave-3 build:
map-first shell, TIGER cartography, Φ default lens, lens groups + Q/E, register
zoom, click-pin InspectionStack→FormulaCard, wages-never-naked adapters, two-stream
events, MIM voice, self-hosted basemap, Installer feel (§9b refined the doc's
phosphor paragraph) — ALL LIVE. Remaining distance:

- [x] Map-anchored pulse channel for critical events (third channel, bible §5.2)
      — Wave-4 PULSE lane, `67caa3ab`.
- [x] Contested-claim TRUE striping (FillStyleExtension + PNG atlas) — Wave-4
      STRIPE lane, `67caa3ab`.
- [x] Verb predicted-delta arrows before commit — Wave-4 DELTA lane, `67caa3ab`
      (`evaluatePredictedEffect` → ▲/▼ chip in VerbForm, honest-null).
- [~] `claim-shimmer`: STRIPE exposed the cause-attributable claim-redraw seam on
      polity membership deltas; the wire-headline consumer is still out of scope
      (documented in `political.ts`).
- [ ] Endgame "never neutral scoreboard text": epitaph mount exists; substance
      arrives with the AI narrator backend (BABYLON_LLM_NARRATOR).

## Owner items raised en route

- bridge_county_h3 res-5/res-7 vs Constitution II.13 res-8 (Program 11 flag) —
  recorded in charter.
- BLS LAUS county file needs re-fetch (data survey).
- profit_rate/occ have no live engine source (Lane D honest nulls) — engine-side
  wiring is an engine-team item, not frontend.
