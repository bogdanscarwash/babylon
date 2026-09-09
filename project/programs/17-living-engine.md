# Program 17 — The Living Engine

**Ratified:** from Percy's north-star description of the desired end state for the UI
(2026-07-11, verbatim below) plus the Program 16 handoff (Phase V env-ready, skin/chrome
shipped, `/explain/` live). Executed ultracode; all subagents Sonnet 5; Fable orchestrates
and synthesizes. Branch `feature/17-living-engine` off `dev` @ `28bef52f`.

**Thesis:** Program 16 built the shell — map-first chrome, lens registry, InspectionStack,
the skin. Program 17 turns that shell into an actual **living** engine underneath it: light
the seam between the sim and the UI so the numbers the player sees are the numbers the engine
really computed, not zeros, stubs, or stale defaults. **Mantra carried from CLAUDE.md: Graph +
Math = History** — Program 17 is the "= History" clause getting wired to the screen.

## North star — the desired end state (owner, 2026-07-11, verbatim)

> Gladly — here's the game I'm building toward, as a player would live it:
>
> **You open the game and you're looking at America.** Not a dashboard with a map widget in it — the map IS the screen, edge to edge. Real TIGER county geometry, 3,235 counties, hairline borders aggregating into the heavier "colonial" state borders. The default lens is **Imperial Rent Φ** — the first thing you ever see is extraction: which counties are being drained, which are enriched, signed and shaded in a single-hue Cold Collapse ramp. That's deliberate (the MIM theory agent was emphatic): a GDP or population default would visually reproduce the exact "backwardness" ideology the game exists to refute. The map's first impression is the thesis.
>
> **Everything else floats.** A top bar with 4 clusters — date/speed, the two permanent vitals (Φ chip paired with ecological overshoot, sharing one color grammar), alerts, takeover buttons. Your organizations live in a Stellaris-style outliner drawer on the left. Verbs sit in a bottom dock — at most 3 prominent at first contact, showing live cost and predicted delta arrows *before* you commit. The game starts paused, camera framed on your scenario. Nothing ever navigates you away from the map; even the wire takeover lifts off of it and settles back.
>
> **You read the map like a Paradox player.** Q/E cycles lenses grouped into Extraction / Struggle / Political / Reproduction. Every lens: one variable, one legend (always visible, with a marker showing where the world currently sits on the ramp), same camera, same geometry — only the fill function changes. Zoom is registers, not magnification: national = states dominant, regional = counties and polity claims, deep zoom = the H3 hexes finally appear as tactical tiles with *more* information per inch, not just bigger colors.
>
> **You interrogate it like Victoria 3 wishes it had shipped.** Click anything — a county, an org, a stat chip in the top bar — and an InspectionCard pins open (click-pin stacks, never hover-mazes; that was Vic3's #1 launch complaint and we're skipping the year it took them to patch it). The top of the card speaks in human values — "organized," "bleeding value," "besieged" — and every number underneath is a Probe: click it and a child card opens explaining its inputs, recursively, down to the actual formula from the engine's registry with its live inputs. **Wages are never shown naked** — every wage figure appears with the value produced and the imperial-rent transfer explaining the gap, and the apologist counter-story ("skill premium") sits one click from its refutation. The inspection stack literally teaches Cope and Amin through play.
>
> **Then the borders start to move.** This is the heart of it. The county mesh is immutable substrate — geometry never morphs. But the *claims* over it redraw: as your organizations build solidarity, as counties rupture, as sovereignty fractures, polity fills re-dissolve along county seams (clean shared-arc merges, no slivers), animated over 600ms+ with a wire headline naming the cause. Contested counties render striped — de facto control disagreeing with de jure claim, the CK3 convention. By late game, if you're winning, the colonial cartography you started with is visibly dissolving into something new — and the UI frames that exactly as the theory does: not map damage, but boundary-as-artifact coming apart.
>
> **Events reach you like a newspaper, not a log.** Two streams — urgent/actionable vs ambient/narrative wire. Critical events hit three channels at once: wire entry, toast, and a visual pulse on the affected geography. Toasts you miss land in a recoverable tray. The voice is the MIM register — headlines lead actor + action + date, state euphemisms in scare quotes, urgency reserved for real stakes so you never learn to ignore it. Endgames are never neutral scoreboard text.
>
> **And the feel** — the takeovers' phosphor/terminal aesthetic (which already exists and is genuinely good) extends into the whole shell, replacing the devtools micro-typography. Red enters as the single structural urgency accent, newsprint/woodcut sensibility over gloss, cyan kept to chrome and never over data. The Carto CDN basemap is gone; the political map is self-hosted and self-sufficient.
>
> Wave 1 is literally building the skeleton of this right now — cartography pipeline, shell teardown, and the explain endpoint. The bones by tonight; the soul in the design pass.

*(Verbatim capture of `desired-end-state-for-ui.md`, owner-authored 2026-07-11. Binding vision
— do not paraphrase or edit; regenerate this section only if the owner issues a superseding
statement.)*

## Governing documents

- `CONSTITUTION.md` — VIII.12 (no silent no-op) and III.11 (Loud Failure) are the two articles
  the Sentinels family exists to enforce; III.7 governs the determinism sentinel.
- `project/research/16-living-map/DESIGN_BIBLE.md` — design law inherited from Program 16:
  five pillars, lens taxonomy (default = Imperial Rent Φ), disclosure rules, event system,
  visual language, voice/lexicon, acceptance gates. Program 17 lights the data this bible's
  chrome already expects.
- `ai/_inbox/deferred-repo-refactors.md` — the priority-ordered list of what's next once this
  program's own scope closes; §1 (reference-DB reproducibility) is the standing top blocker.
- `reports/seam-wiring-punchlist.md` — the living, sentinel-generated backlog for the UI/UX
  wiring pass (regenerate with `mise run check:seams`, never hand-edit).
- `project/research/melt-tvt/` — MELT/TSSI (Monetary Expression of Labour Time / Temporal
  Single-System Interpretation) research corpus, landed 2026-07-14; the future program that
  would extend the value-form math this wave lit (not yet ratified as its own program).

## Ratified plan

Three waves, owner-ordered: **Wave 1 "Light the Seam" → Wave 2 "Topology/Hypergraph" → Wave 3
"Game-feel."** Owner rulings at kickoff: **D1** = build both topology surfaces (map edge-lens
*and* the sigma canvas), **D2** = full Leontief input-output inversion now (not a cheaper
approximation), **D3** = AI narration deferred and mocked for this program.

## Waves

| Wave | Content | Status |
|------|---------|--------|
| 1 | Light the Seam — wire the real tensor/Leontief/MELT/γ pipeline end-to-end; widen the event whitelist; wire the endgame detector; implement the 5 inspectors; live verb cost + arrows; item-25 A/B/C derived-rate lenses | ✅ 2026-07-12 → 2026-07-14 |
| 2 | Topology/Hypergraph — light `get_org_network` + `get_hypergraph_communities`, wire the installed-but-unused `sigma`/`graphology` canvas, add the contradiction-field lens | pending |
| 3 | Game-feel — 5-state eligibility grammar, phosphor/Installer reskin polish, moving-border animation polish (AI narration stays mocked per D3) | pending |

### Wave 1 — Light the Seam (complete)

Commits `5e569fcf` (1a Φ-wiring) → `aa21a7fc` (1b event-whitelist merge) → `13e5fc4b` (1e
verb-cost merge) → `9d8a90b6` (1c endgame) → `784ae0a6` (1d inspectors) → `915df586`
(hex-UPSERT fix) → `fb53debf` (import-loader fix) → `db9cda51` (map-decimal fix) →
`e7804cd7` (hex_cell schema heal) → `a8918338`/`875ae76c`/`a2ab6e9e` (item-25 A/B/C) through
the Sentinels family (`48f0d104` → `499b54a3` → Sensor-3 `aa2a7cc7`/`a32b2f2a`/`0bb2e0c9`) and
the seam-instrumentation close-out (`3d80bb0c` → `013da395`). Every gate green throughout:
full suite **9588→9648+ passed / 0 failed**, **qa:regression 5/5 byte-identical** at every
step, mypy/ruff/hygiene clean.

Delivered:

- **Per-county Φ lit end-to-end** — the genuinely-unwired tensor/Leontief (`L=(I−A)⁻¹`)/MELT/γ
  pipeline wired into TickDynamics + the engine bridge's serializers, backed by a real,
  gross-output-normalized IMPORT_USE load (31,688 BEA io-coefficient rows, 2010–2024;
  `import_coeff = import_$ᵢⱼ / gross_output_millionsⱼ`, textbook penetration values —
  petroleum 0.38, autos 0.23 vs housing 0.008).
- **Events widened 14 → 34** EventTypes converted at the bus→pydantic boundary (+5 new payload
  files), plus a 3rd `event_type` typo fixed in `runtime_db.py`.
- **EndgameDetector wired into `resolve_tick`** — games can now actually end (per-session
  cache; durable `tick_event` read via `get_endgame_state()`).
- **5 inspectors real** — all `get_inspector_*` methods implemented from raw-graph reads
  (were `return {}`), wages-never-naked structure honored.
- **Live verb cost + predicted-delta arrows** — 5/9 static cost labels were wrong, now
  grounded; ▲/▼ arrows for 7/9 verbs; `investigate`/`negotiate` left honest-null (zero scalar
  engine effect) and pinned as a test contract rather than faked.
- **Item 25 A/B/C — all 4 derived-rate metric lenses honest and distinct**: (A) carry
  `tick_profit_rate`/`tick_occ`/`tick_exploitation_rate` through the web bridge's post-`step()`
  round-trip (previously dropped, only `tick_phi_hour` survived); (B) wire the real
  `CapitalStockCalculator` (was `K=0`, degenerating `profit_rate` to `exploitation_rate`); (C)
  wire real QCEW county employment (was a 100k placeholder). Live-verified on `wayne_county`:
  `imperial_rent≈3.7e-5`, `profit_rate=0.175`, `occ=20.6`, `exploitation_rate=3.79`,
  `K=346B` labor-hours — profit ≪ exploitation makes the falling rate of profit visible.

## The instrumentation — the Sentinels family + seam close-out

Program 17 also built the reusable anti-regression apparatus that made lighting Φ safe to ship:
the **Sentinels family** (`babylon.sentinels`, renamed from `babylon.seams`) — five
mechanically-enforced, growable declared-invariant gates:

1. **Seam** (Sensor 1 static continuity + Sensor 2 liveness) — `check_map_metrics`,
   `check_tick_payloads_exist`, `check_severity_vocabulary` gating; `check_tick_coverage`,
   `check_narrator_vocabulary`, `check_event_coverage` advisory.
2. **Determinism** — seed-identical replay hash equality (III.7).
3. **Round-Trip** — `from_graph(to_graph())` conserves a declared 16-field core set.
4. **Economic-Conservation** — finiteness + imperial-rent-pool depletion monotonicity across
   all 53 `imperial_circuit` rows (Amendment Q tolerance).
5. **Data-Coverage** — static slice: every declared reference-data requirement names a source
   adapter that actually exists.

Plus **Sensor 3 (provenance)**, three-then-two more pieces: a rename-aware AST emission-diff
(`_aggregate_hex_features` vs `AdminFeatureProperties` — 4 genuine phantoms found, not the
originally-estimated 14), a `StubBridge` production guard (raises instead of warning when
`DEBUG` is off), and frontend honesty guards (ESLint import bans on render-tree→fixtures, an
MSW bundle-honesty test, a golden field-value snapshot).

**Seam close-out (this week, 2026-07-14):** the typed **`endpoints.ts`** manifest became the
API-contract source of truth (5 hardcoded API paths migrated onto it); an **autonomous
bridge-serialization sweep** (Sensor 3 extension) AST-walks all `get_*`/`_serialize_*`
functions, discovering non-`get_*` serializers and loud no-serializer blind spots the first
pass missed; the coverage sentinel was routed through the family CLI
(`tools/sentinel_check.py <sensor> --check`); and the whole cross-referenced result was
published as **`reports/seam-wiring-punchlist.md`** — 82 advisory findings, a living
sentinel-generated backlog (not hand-maintained) sorted into three data-flow gaps (computed→
not-emitted, emitted→not-consumed, declared→not-emitted) plus two adjacent vocabularies
(narrator templates, event-type routing). It regenerates itself as items get wired — the punch
list is the map for the UI/UX pass, not a one-time report.

**Owner ruling (2026-07-12): no nightly CI for the Sentinels.** Solo project, not enterprise —
mutmut stays a manual on-demand leg (every sentinel already ships a hand-authored efficacy
proof); no Parquet reference-DB CI plumbing (the 6 GB reference DB and its Φ-blank check are a
local on-demand run before a deploy, not a cron).

## Known gaps / deferred

- **IMPORT_USE reference-DB load is local-only — top priority.** The Φ-lighting mutation lives
  only in this machine's local `data/sqlite/marxist-data-3NF.sqlite`; CI fetches a pre-built
  `ci-data-v1` subset that was never regenerated with it, so the crown test
  (`test_imperial_rent_real_wiring.py`) and the "map lights up" claim are unreproduced in CI,
  for other devs, or on the Hetzner deploy. See `ai/_inbox/deferred-repo-refactors.md` §1 for
  the full spec (wire `tools/ingest_bea_imports.py` into the reference-DB artifact builder,
  republish, confirm the crown test green in CI).
- **Hickel-ERDI ecological-rift data ends 2017** — Φ (and the ecological-overshoot metric it
  pairs with) freezes at 2017 values for any tick past that year; no live data source extends
  it yet.
- **`imperial_rent` hex aggregation is a SUM, not a population-weighted mean** — flagged for
  owner metric-design review; a county-level aggregate that sums per-hex Φ will scale with hex
  count rather than represent an intensive rate.
- **`g33_visibility` is NULL** — an unlit gamma-tensor visibility field, not yet traced to a
  root cause.
- Waves 2 and 3 have not started: Wave 2's two endpoints (`get_org_network`,
  `get_hypergraph_communities`) are dead — routes and TS types exist, the bridge methods don't;
  `sigma` and `graphology` are installed with zero imports. Wave 3's game-feel work (5-state
  grammar, phosphor/Installer reskin polish, moving-border animation) is entirely pending, with
  AI narration staying mocked per ruling D3.
- **Owner authorized subrepositories** (2026-07-12), overruling the Program 14 rejection, for
  `src/frontend` specifically ("complex enough to justify it") — tracked as Program 18, not
  part of this program's scope. `ai/_inbox/deferred-repo-refactors.md` carries the fuller
  priority-ordered list of what that decision leaves undone (contract mechanism, `poetry build`
  defect, `babylon_data` submodule candidacy, `web/` split — likely never).

## Owner rulings — 2026-07-14 (engine direction, post-Wave-1)

Two rulings dropped after the epochs vision-gap audit landed. Both are recorded here as
*direction* — neither is designed or scheduled yet.

1. **Emergence over scripted conclusions.** Verbatim spirit: "I want Babylon to be as emergent
   as possible. No conclusion should be pre-programmed, the conclusions emerge from the physics
   engine." Specifics: (a) the EndgameDetector's five pre-determined terminal states are disliked
   as-designed; the model to move toward is Victoria-3-style — a game is a fixed span ("a game is
   a century, it ends when it ends"), with overarching *patterns* rather than adjudicated endings.
   Likely implementation shape when designed: demote the detector from adjudicator (terminates the
   run with a verdict) to recognizer (annotates emergent configurations as wire events; the run
   ends at its horizon). (b) Standard abstract event templates (e.g. a RIOT the AI fills with
   game-universe-specific detail) are good; over-scripting is not — this resolves the pending
   narrator-remediation triage in favor of outcome-aware template narration rather than deleting
   the crafted templates. Blast radius when executed: EndgameDetector + the spec-070 priority
   gates, e2e tests that expect a terminal outcome, qa:regression baselines, and the CLAUDE.md /
   `ai/architecture.yaml` passages naming the 5 outcomes — an ADR (possibly a constitutional
   touch), not a quick fix.

2. **Energy/Labor/Money ternary simplex — named, deferred.** The owner suspects the M/L/E
   triplet forms a *bounding simplex* central to the engine, alongside (not replacing) the
   price/value contradiction tracked in the MELT/TSSI program. Explicitly research-intensive and
   deferred; named now so the direction is on the record. Seed note with research anchors:
   `project/research/energy-labor-money-simplex.md`. Existing engine anchors: MELT (the M↔L
   edge, `project/research/melt-tvt/`), MetabolismSystem's `ΔB = R − (E·η)` + overshoot (E is
   already a first-class per-tick quantity), and the Leontief/Φ tensor stack (L).
