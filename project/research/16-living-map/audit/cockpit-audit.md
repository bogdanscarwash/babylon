# Cockpit component audit vs. spec-113 architecture

Audited against `specs/113-living-map/architecture.md` (Program 16, §0–§7). Source: every
`.tsx` component under `src/frontend/src/components/**` and `src/frontend/src/observatory/**`
(the union of `.design-sync/config.json`'s `componentSrcMap` keys and a filesystem `find`).
51 components read; verdicts below cite the architecture's §1.2 migration-map row where one
exists, and are flagged where my reading disagrees with or extends that table.

Verdict legend: **keep** (survives as-is under the new shell) / **reskin** (survives, styling
only) / **restructure** (logic survives, composition changes) / **kill** (dies with the
dashboard shell).

## shell/ (the dashboard core — heaviest casualties)

**`AppShell.tsx`** (41 lines) — **restructure**. This *is* the five-region CSS-grid
(`grid-cols-[240px_1fr_320px]`, row heights `48px 1fr 200px`) architecture.md §0 names as what
dies; §1.2 marks it "rewritten: 3-layer composition, no grid." Confirmed byte-for-byte: the
whole file is grid wiring, nothing else to preserve.

**`StatusBar.tsx`** (123 lines) — **restructure**. Real `/summary/` data (tick, profit,
Φ, population, alert badges, takeover buttons) all survive per §1.2 ("migrate, keeps
`region-statusbar`/`tick-value` testids"); becomes `chrome/TopBar.tsx`. The takeover buttons
and alert-count logic are pure data plumbing, trivially portable.

**`StatChip.tsx`** (37 lines) — **restructure**. Already does the hard part right
(Constitution III.11 null-honesty: `"no data"` vs. fabricated `0`). §1.2 explicitly extends it
to be clickable → push an `InspectionStack` frame ("every number explains itself starts at the
top bar") — new behavior grafted onto a component that already has the right data shape.

**`TimeControls.tsx`** (96 lines) — **restructure**. The `timeSlice` state-machine wiring
(pause/step/play/resume, `autopaused`/`error` handling, the `playIntent` "resolving but still
wants to run" nuance) is exactly the delicate logic §6's risk register calls out as the most
fragile in the app — none of it should be touched. But the surface itself is rewritten per §4.1
into `SpeedControls.tsx` (adds 1×/2×/5× speed, not just play/pause). Table says "rewrite";
I'd call it restructure — the state read/dispatch pattern here is worth copying forward almost
verbatim, only the button row changes.

**`Outliner.tsx`** (101 lines) — **restructure**. Org/community/faction list logic, selection
wiring, faction-filter toggle all survive per §1.2 ("migrate... `chrome/OutlinerOverlay.tsx`,
collapsible floating left panel"). The docstring's careful mount-lifecycle contract (Outliner
and MapPanel jointly owning `panels.map`) is real, subtle, load-bearing logic that must not be
lost in the move.

**`OutlinerSection.tsx`** (42 lines) — **reskin**. Purely presentational collapsible wrapper
(button + chevron); survives the composition change untouched, just needs floating-panel
chrome instead of inline-nav chrome.

**`OutlinerRow.tsx`** (30 lines) — **reskin**. Same — label/sublabel/selected-state button,
zero coupling to the grid shell.

**`MapPanel.tsx`** (67 lines) — **restructure**. §1.2: "rename+rewrite → `shell/MapStage.tsx`
(full-bleed, keeps `data-testid="region-map"`, keeps sole ownership of `panels.map`
mount/fetch)". The lens/framing/selection prop-threading into `DeckGLMap` is exactly right and
should carry over; only the `<main className="row-start-2 ...">` grid-cell wrapper dies.

**`RightDock.tsx`** (65 lines) — **kill**. §1.2: "**deleted**. Its three tabs disperse."
Confirmed: the file is 100% tab-switcher chrome (`TabButton`, `rightDockTab` state) around
three components that all survive independently (`ActionComposer`→`ActionDock`,
`InspectorPanel`→`inspect/*`, `ObjectivesTracker`→`ObjectivesTray`). Nothing here outlives the
tab shell itself.

**`BottomStrip.tsx`** (79 lines) — **kill**. §1.2: "**deleted**." Same shape as RightDock — a
collapse/tab wrapper (`bottomStripCollapsed`, `activeDockTab`) around `EventsFeed` and
`TimeseriesChart`, both of which disperse to `EventTray`/`BottomDrawer` intact. The
"always-mounted-while-hidden" comment about `panels.timeseries` staying fanned out is the one
piece of real logic here, and §1.2 explicitly preserves that rule in `BottomDrawer.tsx`.

## action/ (verb system — architecture says "unchanged internally")

**`ActionComposer.tsx`** (135 lines) — **keep**. §1.2: "unchanged internally; hosted by
ActionDock's FloatingPanel." Confirmed: acting-org selection (player-controlled-only per
Article V), verb grid, keyed `VerbForm` remount, `pending` list — all pure logic/data, zero
coupling to `RightDock`'s tab chrome.

**`VerbGrid.tsx`** (45 lines) — **reskin**. Logic (disabled-verb honesty tooltip, Article V's
flat 9-verb rule) is exactly right and stays. But its 3-column `grid-cols-3` tile layout is
built for a narrow side-dock; §1.1's diagram puts verbs in a bottom-center `ActionDock` bar,
which reads as a horizontal layout, not a 3×3 grid — the *shape* needs work even though
selection/`onSelect` wiring doesn't.

**`VerbForm.tsx`** (78 lines) — **keep**. Pure composition of TargetPicker + ParamFields +
submit button, remount-keyed correctly; no dashboard-specific styling to purge.

**`TargetPicker.tsx`** (55 lines) — **keep** (for this migration). §1.2 flags a future
"pick on map" affordance as **post-Bible** — out of scope here. Current flat-list-of-buttons
behavior is honest and simple; no change needed until that follow-up work lands.

**`ParamFields.tsx`** (56 lines) — **keep**. Generic select/number/text field renderer driven
entirely by `VerbConfig.paramFields`; config-driven per CLAUDE.md's Paradox Pattern, nothing
dashboard-specific to strip.

## bbl/ (number-formatting primitives)

**`BblData.tsx`** (38 lines), **`BblLabel.tsx`** (26 lines), **`Sparkline.tsx`** (110 lines) —
all **keep**. §1.2: "becomes the number-formatting layer of InspectionStack rows." These three
are already the least dashboard-flavored components in the tree — mono-font data readouts,
uppercase micro-labels, and a compact inline SVG sparkline with delta arrows, all built to be
embedded inline rather than to anchor a panel. `Sparkline`'s honest empty-state placeholder
(keeps the label visible pre-first-tick) is exactly the null-honesty discipline `ValueRow` will
need. Zero grid/panel coupling in any of the three.

## inspector/ (the flat inspector — explicitly superseded)

**`InspectorPanel.tsx`** (100 lines) — **kill**. §1.2: "superseded by `inspect/*`." Confirmed
and worse than the table implies: `GenericFields` literally `JSON.stringify`s unmatched
key/value pairs into a raw admin-panel dump — the opposite of "every number explains itself."
The `kind`-switch (`OrgFields`/`TerritoryFields`/`GenericFields`) pattern is one level deep with
no recursion; `InspectionStack`'s whole point is to replace this with an infinitely-drillable
stack.

**`Stat.tsx`** (20 lines) — **kill**. Label/value row with null-honesty (`"no data"` not `0`)
— the *behavior* is right and should be copied into `ValueRow.tsx`, but the component itself
is superseded; `ValueRow` additionally needs a `ref`/explain affordance this has no slot for.

**`ConsciousnessBreakdown.tsx`** (48 lines) — **restructure**. §1.2: "absorb; becomes an
`InspectionCard` section renderer." §2.1 also names `BreakdownBar.tsx` as absorbing exactly
this shape (proportional composition rows). The three-way revolutionary/liberal/fascist
breakdown and its null-honesty guard are logic worth keeping; the container changes.

## map/ (lens framework — heavy but well-specified)

**`DeckGLMap.tsx`** (500 lines) — **restructure**. §1.2/§3.3: "evolve... sheds its embedded
control cluster." Confirmed: lines 425–448 are an absolutely-positioned control cluster
(`MapLegend` + `MapModeSelector` + `FramingSelector`) glued onto the canvas div — exactly what
moves out to a sibling `MapControls.tsx`. Everything else (layer construction, region vs. hex
branching, tooltip dispatch, `computeFillDomain`) is the real engine and stays put — this is
the single largest and most load-bearing file in the whole tree and the architecture's plan to
touch only its outer 25 lines is sound.

**`MapModeSelector.tsx`** (94 lines) — **kill**. §3.3, explicit: "`MapLensBar.tsx` — replaces
`MapModeSelector` + the metric sub-select." Confirmed: `LENS_MODES`-keyed flat button row is
exactly the un-grouped structure the registry-driven, Paradox-grouped `MapLensBar` replaces.
Testids (`map-mode-selector`, `lens-mode-<id>`) must carry forward per §3.3/§6, so this is a
true kill-and-replace, not a rename.

**`FramingSelector.tsx`** (62 lines) — **restructure**, and this is where I diverge from a
literal reading of the table. §3.3 only says it "stays a separate orthogonal control... mounts
inside the same bar cluster" — sounds like a light touch. But §7's Carto addendum fundamentally
inverts what "framing" means: today `hex` is the default/primary rendering and
`county`/`state`/etc. are LOD aggregation options reached by zooming out; after Carto, county/
state cartography *is* the map and hex becomes a deep-zoom/frontline overlay demoted below the
other five options. The six-button flat list (`ST EA MSA CZ CTY HEX`) itself can probably
survive, but its *default selection and the semantic weight of "HEX"* both change — worth
flagging explicitly for whichever lane (Carto or B) actually owns this file, since neither
lane's "Owns" list in §5 names `FramingSelector.tsx` by filename.

**`MapLegend.tsx`** (44 lines) — **restructure**. §3.3: "gains categorical swatch mode
(stance/faction/collapse/class finally get a real legend instead of a text chip)." Confirmed:
current implementation returns `null` for exactly those four lens kinds (`lensRampStops`
returns nothing for discrete-color lenses) — the ramp-swatch rendering logic stays, a parallel
categorical-swatch path is added.

**`HexTooltip.tsx`** (130 lines) — **restructure**, and this is a gap in the architecture doc
rather than a disagreement: it isn't named anywhere in §1–§7. Its per-lens metric-priority
table (`LENS_METRIC_PRIORITY`) is real, useful logic that has nothing to do with the dashboard
shell — but under §7, hex is demoted to a deep-zoom/frontline detail layer, so this tooltip's
relevance shifts from "the map's primary hover surface" to "a deep-zoom-only detail popup."
Composition survives; when/how it mounts changes. Flagging for Lane Carto or B to claim
explicitly.

## objectives/ (already closest to on-model)

**`ObjectivesTracker.tsx`** (113 lines) — **keep**. §1.2: "hosted in ObjectivesTray." This is
the strongest component in the tree relative to the target aesthetic — its own `objectives.css`
(not utility-class soup), Vic3-explicit docstring ("Vic3-style objectives tracker"), category
accent colors mapped to the 5 endgame conditions, progress bars with category glyphs (`▸`).
Genuinely game-feel already; only its *host* (RightDock tab → floating tray) changes.

## takeovers/ (already the least dashboard-flavored code in the app)

**`TakeoverOverlay.tsx`** (63 lines) — **keep**. §0/§1.2, explicit: "already renders fixed over
everything; it is already map-first-compatible" / "unchanged." Confirmed: fixed-inset overlay,
Escape-to-close, single active-family dispatch — nothing here assumes a grid shell exists
underneath it.

**`ChronicleTakeover.tsx`** (15 lines), **`EndStateScreen.tsx`** (109 lines) — **keep**. Own
`chronicle.css` with scanline overlay, palette-by-outcome (`rupture` bronze-gold vs. `defeat`
laser-red), stat-card row. Genuinely atmospheric; no `text-[Npx]` utility-class dashboard
styling anywhere in this file. One accuracy note *outside* this audit's brief but worth
flagging: `buildStats`/`palette` only distinguish `revolutionary_victory` vs. everything else,
while CLAUDE.md documents 5 terminal outcomes (REVOLUTIONARY_VICTORY, ECOLOGICAL_COLLAPSE,
FASCIST_CONSOLIDATION, RED_OGV, FRAGMENTED_COLLAPSE) — the endgame integration work in §4.4
should double check this screen actually differentiates all five, not just win/lose.

**`DialecticTakeover.tsx`** (15 lines), **`DialecticSpread.tsx`** (154 lines) — **keep**. Own
`dialectic.css`, radial-gradient glow cards, thesis↔antithesis tarot-spread layout, regime-based
accent color. Same story as Chronicle — already past "dashboard," nothing to reskin.

**Wire family — `WireTakeover.tsx`** (17), **`WireApp.tsx`** (184), **`WireWindow.tsx`** (120),
**`IndexPage.tsx`** (133), **`PatternsPage.tsx`** (216), **`CorpusPage.tsx`** (190),
**`ContinentalColumn.tsx`** (184), **`LiberatedColumn.tsx`** (165), **`IntelColumn.tsx`** (173),
**`TranslationFooter.tsx`** (172), **`BlocFlowLines.tsx`** (104) — all **keep**. §1.2:
"unchanged; chronicle auto-opens on endgame." Confirmed via headers/docstrings across the whole
subtree: dedicated `wire.css`/`bloc-flow.css`, three distinct in-universe press "voices"
(Continental corporate press, Liberated pirate-radio phosphor terminal, Intel SIGINT cable with
redaction bars) — this is the most fully-realized game-feel writing in the codebase. One
soft note: `PatternsPage.tsx`'s own docstring calls itself a "Manufacturing Consent dashboard"
— that's a deliberate in-fiction data-journalism pastiche (propaganda-pattern analytics), not an
accidental corporate-tool leak, so I'm not marking it down for it, but it's worth a naming
sanity-check during the Design Bible pass so "dashboard" doesn't creep back in as a load-bearing
identity for a component that's supposed to read as diegetic.

## timeseries/

**`TimeseriesChart.tsx`** (88 lines) — **reskin**, diverging from §1.2's "host adjustments
only" (which I'd read as implying keep-as-is). The null-gap handling (`connectNulls={false}`
falls out of real backend nulls, not fabricated zeros) is correct and should stay untouched.
But the actual rendered chart — a bare `recharts` `LineChart` with default axis ticks and a
plain tooltip box — is stock business-BI chrome with zero game-specific treatment; sitting
inside a "Trends" `BottomDrawer` next to the atmospheric Wire/Dialectic/Chronicle takeovers, it
will read as the one leftover spreadsheet-tool moment in the shell. Flagging for the Design
Bible pass even though architecture.md doesn't call for it.

## observatory/ (out of scope — confirm and move on)

**`ObservatoryRoute.tsx`** (30), **`ObservatoryPage.tsx`** (132), **`DeepPanes.tsx`** (269),
**`ObservatoryChart.tsx`** (100), **`SeriesBrowser.tsx`** (135), **`SessionPicker.tsx`** (54) —
all **keep**, but not because they pass the game-feel bar — because they're explicitly not
trying to. `ObservatoryRoute.tsx`'s own docstring: "dev-facing debug dashboard over the
simulation database... lazy-loaded so it adds no weight to the main game bundle." Every file in
this subtree self-describes as "Tufte-minimal" and deliberately un-gamed (`ObservatoryChart.tsx`:
"data-ink maximised... no decorative glow"). None of the six appear anywhere in
architecture.md's migration map, mount via a self-contained lazy route with no App.tsx edits,
and are correctly disjoint from the player-facing shell. No action needed from this program.

## Overall: systemic dashboard-isms the reskin must purge

1. **The CSS-grid five-region shell is the one real casualty.** `AppShell.tsx`'s
   `grid-cols-[240px_1fr_320px]` + fixed row heights is the textbook admin-dashboard layout
   (nav rail / content / detail rail, like a Django admin or Grafana). Everything downstream of
   it — `RightDock`'s three-tab dock, `BottomStrip`'s collapsible tray — exists only to cram
   content into that grid's fixed cells; both die outright once the grid does (§0/§1.2 confirm).

2. **Tabs-as-navigation is the dashboard's default disclosure idiom, and the architecture
   replaces it with spatial floating panels everywhere it appears** — `RightDock`'s
   Actions/Inspector/Objectives tab triple and `BottomStrip`'s Events/Time Series tab pair both
   use the identical `TabButton` pattern (`aria-pressed`, `border-b-2 border-spire` active
   state). Neither takeover family (Wire's 4-tab `WireWindow`, which *is* being kept) uses this
   same primitive under the hood despite superficially also being tabbed — worth checking during
   implementation that "tabs" as a concept isn't reintroduced by accident inside `FloatingPanel`.

3. **The flat, one-level-deep "field: value" inspector pattern is the second real casualty.**
   `InspectorPanel.tsx`'s `GenericFields` literally does `JSON.stringify(value)` for anything it
   doesn't have a named field for — an admin-tool escape hatch, not a designed UI. This is
   exactly what `InspectionStack`'s recursive drill-down is built to replace, and it's the
   clearest single piece of evidence that "every number explains itself" is a real gap today,
   not a nice-to-have.

4. **Empty-state copy is uniformly admin-tool voice, never diegetic.** Every empty state read
   across the tree says variations of "No world state loaded yet.", "No organizations in this
   session.", "Communities not loaded yet.", "No active objectives.", "No timeseries data yet." —
   accurate and honest (good — Constitution III.11 null-honesty is followed everywhere,
   including `Sparkline`'s placeholder and `StatChip`'s "no data") but written like a REST API
   error page, never in the game's own voice. The Wire family is the one place this breaks
   ("neutrality is hegemony" as static chrome text) — proof the team already knows how to write
   in-voice when it wants to; the shell/inspector/map-controls layer never does.

5. **Component naming vocabulary is IDE/dashboard-derived, not game-derived**: `RightDock`,
   `BottomStrip`, `StatusBar`, `Outliner` read as VS Code/Blender/Unity-editor panel names, not
   Paradox-strategy-game chrome names. `Outliner` specifically survives the migration as a name
   (§1.2 keeps calling it `OutlinerOverlay`) even though its *behavior* becomes the Stellaris
   floating-outliner idiom — the vocabulary mismatch will likely persist past this program unless
   the Design Bible renames it.

6. **Dropdowns/selects stand in for map-native picking.** `OrgSelect` (`<select>` of acting
   orgs) and `MapModeSelector`'s faction `<select>` both resolve a spatial/political choice
   through a generic HTML form control rather than an on-map or on-outliner interaction —
   exactly the gap `TargetPicker`'s future "pick on map" affordance (§1.2, post-Bible) is meant
   to close, but only for targets, not for acting-org or faction-filter selection; worth scoping
   whether those two also deserve the same treatment eventually.

7. **The map's own control cluster reads as a devtools overlay, not game chrome.** `MapLegend` +
   `MapModeSelector` + `FramingSelector`, stacked top-right in `bg-void/80 backdrop-blur-sm`
   boxes with plain uppercase text buttons ("STANCE", "HEAT", "ST", "CTY") — functionally exactly
   right (the architecture literally promotes this `bg-void/80 backdrop-blur-sm` idiom into the
   `FloatingPanel` primitive, §1.3), but visually it's a debug HUD, not a Paradox map-mode
   selector with icons. The Design Bible inherits working plumbing with zero visual identity.

8. **Escalation/urgency staging is absent from the persistent chrome.** `StatusBar`'s alert
   badges (`bg-laser`/`bg-heat` pills) are the *only* place severity gets any visual weight; a
   civilization-ending imperial-rent spike and a routine profit-rate tick render at identical
   type size/weight in `StatChip`. §4.2's planned `CriticalEventModal`/`EventToasts` will finally
   give the existing (already-built) autopause machinery a face — until then there's a real gap
   between "the engine knows this tick mattered" and "the UI shows it."

9. **Two visual languages already coexist in the same tree, and only one is being carried
   forward as the target.** The takeovers (Wire/Dialectic/Chronicle) use bespoke per-family CSS
   files with glow, scanlines, phosphor-terminal aesthetics, radial gradients — real game
   chrome. The shell/inspector/map-controls layer uses uniform Tailwind utility classes
   (`text-[9px] uppercase tracking-widest text-ash`) applied identically everywhere regardless of
   content. The reskin isn't inventing a new visual language from scratch — it's extending the
   half that already exists over the half that's still a generic admin panel.

10. **Chart-first thinking survives in exactly one place after this migration:
    `TimeseriesChart.tsx`.** Every other numeric display in the app (`BblData`/`Sparkline`/
    `StatChip`/`Stat`) is designed to sit inline next to a label as one fact among many; only
    `TimeseriesChart` centers a chart-as-the-whole-view, `recharts`-default axes and all. It
    survives the migration (§1.2: "host adjustments only") into a "Trends" drawer — worth a
    second look during the Design Bible pass, since it's the one surviving component built
    around "the chart is the content" rather than "the number is the content, formatted well."
