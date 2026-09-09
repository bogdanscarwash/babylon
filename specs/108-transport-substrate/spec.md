# spec-108 — The Transport Substrate (Constitution II.13, Amendment O)

**Program**: 11 (`project/programs/11-transport-substrate.md`, ratified 2026-07-08)
— authored under Program 26 (International Trade) Unit U5,
`project/programs/26-international-trade.md`. **Depends on**: spec-107 (§
Consumption seams — σ acting on value flows), spec-101 U2/U3 (interactive
Φ-flow wiring, FAF freight grounding), the P25 lane merging (non-overlap
covenant, Program 26 §3 — this unit is explicitly **post-P25**).
**Consumed by**: nothing yet — this is the authoring pass. Engine wiring is
a separate, later unit (tracked as the "engine-step" throughout this
document); Program 26 §4 calls it "U5 — the engine train."
**Status**: authored (this document + plan/research/tasks); zero source code
changes accompany it (docs-only pass, matching spec-107's precedent).

## Why

Capital Volume II: surplus value produced is nothing until **realized** —
commodities and labor-power must physically move through circulation, and
circulation time gates turnover and therefore the annual rate of surplus
value. Transport is the material substrate of realization: where it is cut,
value is stranded (a realization crisis); where it is dense, turnover
accelerates. The substrate also makes political action economically real —
severing a corridor (riot, strike, sabotage, state siege) strands value;
rebuilding it is a class act with a class character (owner framing,
program-11 §"Why"). Program 10's σ-gradient (spec-107) says which way value
flows (up-gradient, colony→metropole); this program supplies the CHANNELS it
flows through. Program 26 (International Trade) needs this because U5's
charter (`project/programs/26-international-trade.md` §4) explicitly folds
transport in: "resource flows over the transport substrate (author
spec-108) priced by Vol III money."

Program 11 itself was ratified nine months before Program 26 reopened trade,
and — exactly like Program 10/spec-107 — was never written up as a spec.
This document is that missing write-up: Program 11's own charter section
("Spec-108 charter (author via speckit; this is the scope contract)")
already pins slice-1/slice-2 boundaries and four Round-2 rulings; this spec
transcribes those into house format, re-verifies every as-built claim
against the CURRENT tree (the program doc is dated 2026-07-08 and several
paths it cites have since moved under Program 14's package reorganization),
and hands the engine train (U5-code) a concrete, file-cited implementation
contract (tasks.md).

## Constitution II.13 (verbatim, current text)

> **13. Transport Substrate** — The movement of value, goods, and people is
> modeled as a transport substrate with two mechanisms: **min-cost flow**
> for deterministic routing `[RATIFIED · IMPLEMENTED]` (roads, rail,
> shipping lanes) and **slime-mold conductivity** for emergent routing
> `[RATIFIED · PENDING CODE]` (networks that optimize under pressure, like
> informal supply chains or migration routes). Transport edges have types:
> AIR_LINK (high speed, high visibility), SHIPPING_LANE (bulk, slow,
> regulated), the **road tier** (HIGHWAY / ARTERIAL / LOCAL_ROAD — flexible,
> medium visibility), RAIL (capacity-constrained, infrastructure-dependent),
> and **INFORMAL** (slime-mold-only routing, no built infrastructure). The
> transport substrate is a Volume II/III mechanic: it mediates between
> production (Volume I) and realization (Volume III), and its topology
> determines where crises of disproportionality and realization propagate.
> **Extension** `[RATIFIED · PENDING CODE]` (owner-ratified 2026-07-08):
> corridors are state-owned; edges carry a **condition** that **degrades**
> with use and neglect; agents **build and repair** them through the
> existing `BUILD_INFRASTRUCTURE` verb (no new verb); the implementing spec
> is not yet authored. See Also: V (BUILD_INFRASTRUCTURE), III.11 (a missing
> transport input fails loud, never silently no-ops).
> — `CONSTITUTION.md:416` (Article II, Amendment J baseline + Amendment O
> extension, `CONSTITUTION.md:571`)

Binding constraints this spec inherits directly from the clause above (not
negotiable by this unit or the engine-step that follows it):

1. **Two-mechanism split is fixed**: min-cost flow (deterministic) +
   slime-mold conductivity (emergent). Neither mechanism substitutes for the
   other; program-11's slice-1/slice-2 split (below) sequences them, it does
   not merge them.
2. **Five edge types are fixed nomenclature**: `AIR_LINK`, `SHIPPING_LANE`,
   the road tier (`HIGHWAY`/`ARTERIAL`/`LOCAL_ROAD`), `RAIL`, `INFORMAL`. A
   sixth type is a new primitive and requires a constitutional amendment
   (III's "Escalate to Amendment" rule — see `CONSTITUTION.md:607`).
3. **Volume II/III mediation is fixed**: the substrate sits between
   production (Vol I) and realization (Vol III); its topology determines
   where realization crises propagate. This spec's consumption-seam design
   (§ below) must land the routing math on that boundary, not invent a
   parallel crisis channel.
4. **State ownership + BUILD_INFRASTRUCTURE reuse is fixed** (Amendment O):
   corridors are state-owned in slice 1; construction/repair MUST go through
   the *existing* `BUILD_INFRASTRUCTURE` ActionType — Amendment O explicitly
   forecloses a new verb.
5. **III.11 Loud Failure applies**: "a missing transport input fails loud,
   never silently no-ops" is cited directly in the clause's "See Also." Any
   engine-step gate on absent transport data (LODES/FAF/HPMS) must raise or
   warn, never fabricate a zero silently.

## What ships (functional requirements)

This spec's write surface is `specs/108-transport-substrate/**` only — no
source file is created or modified. FR-108-N below are *specification*
requirements (what this document must pin precisely enough for the
engine-step to consume); each names its corresponding future engine
deliverable in tasks.md.

- **FR-108-1 — Corrected as-built inventory.** Program-11's as-built table
  (2026-07-08) is re-verified against the current tree and corrected in
  research.md: the `src/babylon/infrastructure/` package it cites no longer
  exists — Program 14's reorganization moved it to
  `src/babylon/domain/geography/` (confirmed: `find` returns zero hits under
  `infrastructure/`, sixteen `.py` files present under `domain/geography/`).
  Feature 036 (spec-036, `specs/036-infrastructure-topology/spec.md`,
  **Status: Draft**) is more built than program-11's table credits: a
  full typed link/capacity/condition DTO layer already exists
  (`domain/geography/types.py::InfrastructureLinkState`, carrying `capacity:
  dict[FlowCategory, float]` **and** `condition: float ∈ [0,1]` **and**
  `effective_capacity() = capacity × condition` already, plus
  `DefaultEdgeCapacityCalculator` in `capacity.py` that aggregates per-edge
  capacity and zeroes WATER-WATER edges) — but it is **engine-orphaned**:
  `DefaultEdgeCapacityCalculator(...)` and `DefaultInfrastructureInventory(...)`
  are constructed nowhere outside `tests/` (verified by
  `rg -n "DefaultEdgeCapacityCalculator\(|DefaultInfrastructureInventory\("
  --type py -g '!tests/*'` → zero hits). This is the single most important
  correction this spec makes to program-11: **the "condition" degradation
  field US3 asks for already exists as a typed Pydantic field with an
  effective-capacity formula** — the gap is wiring it to a graph/tick, not
  building it from scratch.
- **FR-108-2 — Corridor mesh model (US1, engine-only).** A SPARSE res-8
  corridor graph, never a full national res-8 tiling (~11M cells; would
  violate the spec-087/089 storage regime). Nodes = R8 cells touched by
  NE/HPMS/NTAD linear features plus junction cells (mirrors the *existing*
  `R8LinearFeature`/`HexR8State` per-cell pattern in
  `domain/geography/r8_types.py` — note that today's `R8LinearFeature` is a
  per-cell marker, not an edge-with-capacity; the corridor mesh's edges are
  new construction built ON TOP of the R8 cell inventory, reusing
  `R8FeatureType`/`InfrastructureType`/`InfrastructureLinkState` rather than
  inventing parallel vocabulary). Slice-1 edge types: the road tier +
  `RAIL` + `INFORMAL`; `AIR_LINK` + `SHIPPING_LANE` deferred to slice 2
  (Program 11 Round-2 ruling R2-4). Per-edge state: `{capacity, condition ∈
  [0,1], conductivity D}` — `capacity`/`condition` reuse
  `InfrastructureLinkState` verbatim; `conductivity` is new (§FR-108-3).
  Terrain taxonomy extension (mountain/wetland/desert/forest/water classes
  from NE physical polygons) + a traversal-cost table in
  `TransportDefines`; impassability = cost that excludes the edge from the
  routing graph entirely (not a literal `inf` float — see tasks.md
  discussion). Aggregates to per-R7-pair connectivity coefficients consumed
  by the economic layer, mirroring the existing `r8_aggregation.py`
  R8→R7 pattern.
- **FR-108-3 — Routing (US2).** Two coexisting mechanisms per Constitution
  II.13:
  - **Min-cost flow** (deterministic) over the corridor mesh for FAF5 goods
    O-D by SCTG commodity (BEA I-O fallback when FAF is silent for a
    commodity), composing with — NOT replacing — the LODES labor base layer
    already live in `vol2_circulation.py`. **No min-cost-flow solver
    currently exists in this codebase** (verified: `rg -ni
    "min_cost_flow|network_simplex" --type py` and `rg -n "csgraph"` both
    return zero hits outside this survey; `Vol2CirculationStep` is a fixed
    proportional-share redistribution over historical OD shares — v[A,t+1] =
    Σ_j OD[j,A]·v[j,t]/rowsum[j] — not a capacitated optimization. It is the
    right PATTERN to imitate (deterministic, sparse-matrix, no RNG) but not
    a routing algorithm the freight case can reuse directly, because freight
    routing must respect `effective_capacity` constraints the labor
    redistribution never enforces.) The engine-step must pick and implement
    one (candidates in research.md).
  - **Slime-mold conductivity** (emergent), per-edge EMA `D(t+1) = (1−α)·D +
    α·|Q|` — `INFORMAL` edges carry conductivity WITHOUT built substrate
    (no NE/HPMS feature, no `condition` variable; pure D dynamics over
    terrain traversal costs), modeling migration routes and the informal
    economy; they emerge where flux persists and die back with disuse.
    Deterministic (III.7): no RNG anywhere in the update, matching the
    existing `vol2_circulation.py` GATE-5 docstring boundary
    ("Slime-mold conductivity... is out of scope" for that module — it stays
    out of scope there; it lives here).
  - **Realization-crisis coupling**: unrouted demand → unrealized value.
    This spec finds the CORRECT existing landing point:
    `domain/economics/circulation/crisis.py::assess_circulation_crisis`
    already computes a `commodity_overhang` ratio that trips
    `CirculationCrisisAssessment.realization_crisis` above
    `commodity_overhang_threshold` (`CapitalVolumeIIDefines`,
    `capital_vol2.py:76`) — theoretically the exact right home (unrouted
    goods are commodity-capital stuck at C', unable to become M'). The
    engine-step's job is to feed transport-stranded volume INTO
    `commodity_overhang` (or a new additive term `_compute_county_
    circulation_state` passes into it), not to build a second, parallel
    crisis type. `RealizationCrisis`/`compute_realization_metrics` (cited in
    `capital_vol2.py`'s exclusion note) currently has **zero production
    callers** — this is a second, adjacent dormant construct the
    engine-step should audit before deciding which of the two realization
    paths (the `CirculationCrisisAssessment` one, live; or
    `RealizationCrisis`, dormant) is the correct landing point. **Engine-step
    decision, not resolved here.**
- **FR-108-4 — Degradation (US3).** `condition` decays with flux + time,
  calibrated against the HPMS pavement-condition distribution (`dot/
  HPMS_Spatial_All_Sections_-_2024.csv`, confirmed present — § Data
  contract); maintenance spend (state budget, "faux frais of circulation" in
  Marx's own phrase) offsets decay; disused corridors lose conductivity (the
  slime-mold decay term IS the disuse mechanic — no separate "abandonment"
  system). Reuses `InfrastructureLinkState.condition` and
  `effective_capacity()` (FR-108-1) rather than inventing a new decay
  field; the *rate* is new (`TransportDefines`, tasks.md). The existing
  `formulas/community.py::calculate_infrastructure_decay` (registered in
  `formula_registry.py`, consumed today by `engine/systems/community.py` for
  a DIFFERENT scalar — the per-Territory civic-infrastructure float, not any
  per-edge corridor `condition`) is a candidate pattern to imitate (same
  decay-with-neglect shape) but is NOT reused directly: it operates on a
  single Territory-scoped float, while corridor decay is per-edge on the R8
  mesh. Flag this as a naming-collision risk for the engine-step: two
  "infrastructure decays with neglect" mechanics will coexist
  (community-scoped and corridor-scoped) and must stay clearly distinguished
  in code and in any player-facing narrative.
- **FR-108-5 — Construction / repair / destruction (US4).** Zero new
  ActionTypes (Amendment O + Program-11 Round-2 ruling R2-3, both binding):
  `ActionType.BUILD_INFRASTRUCTURE` (`models/enums/actions.py:83`) carries
  BOTH construction (new edge) and repair (existing degraded edge),
  distinguished by action params; `ActionType.ATTACK_INFRASTRUCTURE`
  (`:84`) carries damage. **Verified asymmetry the engine-step must close**:
  `ATTACK_INFRASTRUCTURE` already has a registered player-verb resolver
  (`engine/actions/attack.py::resolve_attack`, wired into
  `VERB_RESOLVERS` in `engine/actions/__init__.py`) whose material effect
  flows through the already-wired `ooda/layer3.py::_propagate_infrastructure`
  consequence pass. **`BUILD_INFRASTRUCTURE` has NO resolver at all** —
  `VERB_RESOLVERS` has no entry for it, and there is no
  `engine/actions/build.py` file (verified: `rg -n "resolve_build|
  BUILD_INFRASTRUCTURE" src/babylon/engine/actions/` → zero hits). This is
  the single sharpest, most concrete engine-step deliverable this spec
  surfaces: write `engine/actions/build.py::resolve_build` and register it.
  Riot/uprising events (`StruggleSystem` `UPRISING`, `EXCESSIVE_FORCE`
  aftermath) damage `condition` via the same `layer3._propagate_infrastructure`
  delta seam `ATTACK_INFRASTRUCTURE` already uses — no new seam needed,
  PROVIDED the engine-step resolves the FR-108-1 tension between the
  community-scoped `infrastructure` float that seam currently writes and the
  per-edge `InfrastructureLinkState.condition` this spec's corridor mesh
  needs (§ Director ruling required, item 4).
- **FR-108-6 — Corridors as substrate overlay, blocs as graph-level
  alignment (Program 26 U5 framing).** The transport substrate is the
  IMMUTABLE spatial layer (Constitution: "the spatial substrate is
  immutable; political claims are overlays" — `CONSTITUTION.md` core
  principle, restated in program-11 §"Why"). Program 26 U5 additionally asks
  this spec to establish how "blocs" (the 8 international engine nodes —
  `canada, china, eu, india, sub_saharan_africa, latin_america, russia_csi,
  southeast_asia`, per spec-101's D3) relate to the substrate: **blocs are
  graph-level ALIGNMENTS over `NodeType.SOVEREIGN` nodes, not a new node
  type and not a new edge type.** Verified against `models/enums/topology.py`:
  `NodeType.SOVEREIGN` already exists (spec-070 Balkanization,
  production-stamped) and already carries `CLAIMS`/`ADMINISTERS` edges to
  territories; a "bloc" is a label/grouping attribute over a set of
  `SOVEREIGN` node IDs (or, for the 8 external nodes, `ExternalNode` rows —
  a Postgres table, not a graph node at all, per spec-101's
  `dynamic_external_node_state`), never a graph primitive. **No new
  NodeType or EdgeType is required for blocs themselves.** The transport
  substrate's own vocabulary gap is narrower: `InfrastructureType`
  (`models/enums/topology.py:177`) already declares
  `HIGHWAY/ARTERIAL/LOCAL_ROAD/RAIL/PIPELINE/TRANSMISSION/SHIPPING_LANE/
  AIR_LINK` — **the only missing member is `INFORMAL`** (Amendment O's
  fifth edge type). Adding it is a one-line enum addition, but per this
  worktree's CLAUDE.md vocabulary-sentinel discipline it still needs its
  own production stamper named explicitly in tasks.md (a bare enum add with
  no stamper is exactly the "declared but not production-stamped" failure
  mode `NodeType`'s own docstring warns about).
- **FR-108-7 — Vol III money + σ consumption seams named concretely.**
  Program 26 U5 requires flows "priced by Vol III money" and "σ acting on
  Vol I/II value flows." This spec names the actual modules (no new math
  written here, per the P26 §2 "no shadow value system" constraint):
  - **Vol III pricing**: `engine/systems/market_scissors.py`
    (`MarketScissorsSystem`, position 17.8 — the price⟷value axis; ADR077/
    ADR078) is the canonical price-conversion machinery; endogenous interest
    lives in Vol III Part V (`domain/economics/` — see
    `ai/state.yaml`'s `endogenous-interest-vol3` entry). Freight/labor
    volumes routed over the substrate become dollar-valued via this
    machinery, not a new pricing formula.
  - **σ coupling**: spec-107's declared-not-inserted consumption seams
    (`specs/107-sigma-gradient/spec.md` § Consumption seams) are the exact
    landing points — in particular the "Transfer lever" row
    (`engine/systems/economic.py`'s national-flat `extraction_efficiency`,
    replaceable per-edge via `formulas/unequal_exchange.py::
    calculate_exchange_ratio`/`calculate_value_transfer`, already
    registered in `formula_registry.py` with zero callers today). This
    spec does not touch those files; it confirms they are the correct
    seam for "σ acting on Vol I/II value flows over the substrate."
- **FR-108-8 — Φ as an actual inter-national transfer.** Today, per
  spec-101 D1-D3, Φ is a *distributed national aggregate* attributed to
  blocs by a trade-share proxy and then rained down onto counties by
  `phi_distribution.distribute_phi_week_to_counties` — it never "flows"
  anywhere physically. Program 26 U5 wants Φ to become "an actual
  inter-national transfer." This spec's contribution: the transport
  substrate is the physical channel FAF freight tonnage and Ricci
  GVC-transfer dollars ALREADY have a declared home for
  (`BoundaryEdgeKind.PHYSICAL_EXCHANGE`, `domain/economics/node_kinds.py:54`
  — "FAF freight or USGS minerals" — **verified zero producers anywhere**,
  `rg -n "PHYSICAL_EXCHANGE" --type py` returns only its two declaration
  lines). U5's engine-step should route FAF-freight-derived Φ/trade
  magnitude through the corridor mesh and emit `PHYSICAL_EXCHANGE` register
  rows at the boundary — the register-row PATTERN (`TRADE_EDGE`/
  `DRAIN_EDGE`/`COMMUTE_OUT`, all in `Vol2CirculationStep.step`) already
  exists and should be imitated, not reinvented. **This spec does not
  design the Φ-attribution model itself** — that is U4's job, Director-ruled,
  orthogonal to the substrate (Program 26 §4).
- **FR-108-9 — Conservation invariant, restated for the substrate.**
  Program 26 §2 states "conservation invariants extend (Σ_nodes Φ_node =
  national Φ stays; add freight/value closure as flows materialize)." This
  spec's obligation: state the EXTENDED invariant precisely so the
  engine-step has a falsifiable target: for any tick, `Σ (flow into a
  corridor edge) = Σ (flow out of that edge) + Δ(stock held on the edge, if
  any) `, and at the national boundary, `Σ_corridors (freight/value crossing
  the study-area boundary) ≡ Σ_bloc (PHYSICAL_EXCHANGE register magnitude for
  that bloc)` — mirroring the existing FR-101-5 pattern
  (`Σ DRAIN_EDGE credits this tick ≡ Φ_week`) exactly, with the same
  relative-residual reporting convention (spec-101 D4) so absolute float
  error at large USD magnitudes doesn't trip `--strict`.
- **FR-108-10 — Vol2CirculationStep's first production constructor
  (data-path deliverable named by this spec, not built here).** Verified:
  `Vol2CirculationStep(od_loader=..., hex_county_adjunction=...,
  classifier=...)` is constructed nowhere in `src/` outside `tests/`
  (`rg -n "Vol2CirculationStep\(" --type py -g '!tests/*'` → zero hits).
  Every input it needs already has a real production supplier, just never
  composed together:
  - `od_loader` (`LODESCommuteMatrixLoader`): production-constructible via
    the CHECKED-IN Detroit tri-county artifact
    (`domain/economics/lodes_study_area.py`:
    `LODES_ARTIFACT_ROOT = src/babylon/data/reference/lodes/`,
    `LODES_ARTIFACT_CROSSWALK`, `lodes_tri_county_hexes_res7()`,
    `LODES_STUDY_AREA_STATES = {"26"}`) via
    `headless_runner/lodes_hydration.py::resolve_lodes_hydration_kwargs()`
    — **this is a small pruned artifact, NOT the full mounted drive**. The
    full drive is separately verified present on this box at
    `/media/user/data/babylon-data/lodes/` (`od/` holds 50-state ×
    2010-2021 `*_od_main_JT00_*.csv.gz` files, e.g.
    `mi_od_main_JT00_2018.csv.gz`; `us_xwalk.csv.gz` is 143 MB) but the
    CI-no-drive rule (`ai/anti-patterns.yaml`, restated in the U2 contracts
    doc) forbids depending on that mounted path at runtime — production
    code must use the checked-in artifact only.
  - `hex_county_adjunction` (`ScaleAdjunction`): production-constructible via
    `persistence/hex_hydrator.py::read_hex_county_adjunction(runtime,
    session_id)` (`:276`) — already exercised by five integration tests,
    never by any `src/` production caller.
  - **The missing piece is the composer function itself.** `game/trade.py::
    build_interactive_trade_wiring` already accepts a `vol2_step:
    Vol2CirculationStep | None = None` parameter (`:110`) and its sole
    production caller, `cli/play.py:195`, never passes one — it defaults to
    `None`, so the sentinel row `vol2_circulation_vol2_step`
    (`sentinels/seam_algebra/registry.py:449`) being CLOSED (a supplier
    file exists: `game/session.py`, per ADR162) does **not** mean real
    `Vol2CirculationStep` data ever flows in an interactive campaign — it
    means the WIRING PATH exists and defaults safely to the gated no-op.
    tasks.md names the exact composer (`build_vol2_circulation_step()`) and
    its call site (`cli/play.py`, right after the existing
    `build_interactive_trade_wiring` call, scoped to the same
    `DETROIT_TRI_COUNTY_FIPS` the checked-in artifact covers — the
    substrate's slice-1 geography is bound to the SAME tri-county limit as
    the LODES data, not a national rollout).

## Non-goals

- Any tick-pipeline mutation, `simulation_engine.py` change, or new/reordered
  System registration by THIS document — those are engine-step (tasks.md)
  content, explicitly out of scope for a docs-only spec (mirrors spec-107's
  own non-goals discipline). This spec proposes a candidate system position;
  it does not create the System class.
- `AIR_LINK` / `SHIPPING_LANE` edges (deferred to slice 2 with the NTAD
  marine + aviation geodatabases, Program-11 Round-2 ruling R2-4).
- Ownership/rent extraction on corridors (Round-2 ruling R2-1: state-owned
  only in slice 1; the σ-ownership coupling is a recorded follow-up, not
  scoped here or in the near-term engine-step).
- The Φ-attribution model itself (Program 26 U4's job, Director-ruled).
- Position mobility, spatial visualization of res-8 (program-11 owner ruling
  1: "nothing in `web/` needs to render them" — still binding; the Archive
  TUI's future trade surfaces, Program 26 U6, consume PROJECTIONS of
  substrate state, never the res-8 mesh itself).
- Generating any data artifact (FAF freight hash-stamped artifact is U3's
  job; this spec only declares the consumption seam, FR-108-8).
- Settling whether the community-scoped `infrastructure` float and the
  corridor-scoped `InfrastructureLinkState.condition` unify into one
  mechanic or stay two — flagged as Director ruling required, item 4, not
  resolved here.

## Key decisions (recorded)

- **D1 — `InfrastructureLinkState`/`DefaultEdgeCapacityCalculator`
  (spec-036) are the correct foundation for the corridor mesh's per-edge
  state, not a fresh model.** Verified identical field shape to what
  program-11's US1 charter asked for (`{capacity, condition}` — the third
  member, `conductivity`, is new). Building a second, parallel
  capacity/condition DTO would violate this worktree's DRY Super Rule and
  create exactly the kind of dual-representation drift the vocabulary
  sentinel exists to catch. **Engine-step MUST reuse `domain/geography/
  types.py::InfrastructureLinkState` and `capacity.py::
  DefaultEdgeCapacityCalculator`**, extending rather than replacing them.
- **D2 — The corridor mesh is a NEW graph, not new `EdgeType` members on
  `BabylonGraph`.** `EdgeType` (`models/enums/topology.py:78`) is the
  vocabulary for the MAIN simulation graph (EXPLOITATION, SOLIDARITY,
  TRIBUTE, …) — political/economic relations between `SOCIAL_CLASS`/
  `ORGANIZATION`/`TERRITORY`/`SOVEREIGN` nodes. The transport substrate is a
  SEPARATE R8-resolution mesh (per program-11 owner ruling 1: "Res-8 hexes
  are the underlying engine, NOT a visualization layer... program
  connectivity invisibly"). Corridor edges use `InfrastructureType`
  (`topology.py:177`), a DIFFERENT enum, already scoped correctly for this
  purpose. **No `EdgeType` member is added by this spec's design.**
- **D3 — `INFORMAL` is the one real vocabulary gap, and it needs a named
  production stamper, not a bare enum add.** `InfrastructureType` is missing
  exactly one member relative to Constitution II.13's five-type list. Per
  this worktree's vocabulary-sentinel discipline (CLAUDE.md "Gotchas"),
  adding `InfrastructureType.INFORMAL` without a production code path that
  actually stamps it (the slime-mold conductivity overlay minting an
  `INFORMAL` link when `D` crosses a threshold with no built substrate
  backing it) reproduces the exact "declared but not production-stamped"
  failure class the sentinel was built to catch. tasks.md names the stamper.
- **D4 — Unrouted freight demand feeds `commodity_overhang`, not a new
  crisis type.** See FR-108-3's third bullet. This is the closest existing
  theoretical home (goods stuck at C', unable to realize as M') and avoids
  building a parallel "transport crisis" concept that the dialectics
  catalog (`domain/dialectics/instances/catalog.py`) would then need a
  SECOND opposition/field wiring for. **Director-adjacent, not fully
  resolved**: whether the coupling coefficient (stranded-freight-value →
  commodity_overhang delta) is itself a `TransportDefines` tunable or a
  `CapitalVolumeIIDefines` one is an engine-step call, not decided here.
- **D5 — `INFRASTRUCTURE_CHANGE` (EventType, `models/enums/events.py:125`)
  is the correct event for BUILD/REPAIR/ATTACK corridor narrative, reused
  not reinvented.** Verified zero current emitters
  (`rg -n "INFRASTRUCTURE_CHANGE" --type py` → only its declaration) — a
  second dormant construct exactly waiting for this work, matching the
  `PHYSICAL_EXCHANGE` pattern (D... wait, FR-108-8) above. One NEW EventType
  is still needed for the realization-crisis coupling signal specifically
  (tasks.md names it) since `INFRASTRUCTURE_CHANGE` narrates the corridor
  itself, not the downstream stranded-value consequence.
- **D6 — Blocs are graph-level alignments, confirmed no new NodeType.** See
  FR-108-6. Cross-checked against `NodeType`'s closed vocabulary
  (`models/enums/topology.py:12`): `SOVEREIGN` already exists and already
  carries the needed edge types (`CLAIMS`, `ADMINISTERS`) for a bloc to be
  expressed as a labeled grouping over existing sovereigns/external nodes.

## Data contract

Re-verified 2026-07-27 against the mounted `/media/user/data/babylon-data/`
drive (present and readable in this worktree, unlike spec-107's authoring
session where it was absent) and the current source tree.

| Ingredient | Source | Status |
|---|---|---|
| Road network + class | NE 10m roads (`domain/geography/natural_earth_reader.py`) + DOT HPMS `dot/HPMS_Spatial_All_Sections_-_2024.csv` | **PRESENT** (verified `ls`: file exists, plus `hpms-spatial.json`/`-metadata.json` siblings) |
| Road condition (calibrates degradation) | HPMS pavement/section attributes, same file | **PRESENT** (loader still needed — no HPMS ORM/parser found in `src/` by this pass) |
| Railways | NE 10m railroads (reader built, `natural_earth_reader.py`) | **PRESENT** |
| Airports / intermodal / marine RoRo (slice 2) | `dot/NTAD_Aviation_Facilities_*.geodatabase`, `dot/NTAD_Intermodal_Freight_Facilities_{Air_to_Truck,Marine_Roll_on_Roll_off,Pipeline_Terminals,Rail_TOFC_COFC}_*.geodatabase` | **PRESENT** (verified `ls dot/`; six geodatabases present including `NTAD_North_American_Rail_Network_Lines` and `NTAD_Military_Bases`) |
| Freight commodity O-D by SCTG | FAF5 `freight/faf/FAF5.7.1_State_2018-2024.csv` + `freight/faf/region/FAF5.7.1_2018-2024.csv` + `freight/faf/county/*.zip` (50 state zips) + mode-split factor CSVs (`{truck,rail,water,pipeline}_{origin,destination}_factors.csv`) | **PRESENT** (verified `ls freight/faf/`; county-level zips confirmed for at least Alabama, `FAF5-County-Level-Estimates-Technical-Report.pdf` present) — loader still needed; this is U3's declared job, not this spec's |
| Labor O-D | LODES `fact_lodes_commuter_flow` (Postgres, hydrated) + on-disk `lodes/od/*.csv.gz` (all 50 states, 2010–2021, verified present) + `lodes/us_xwalk.csv.gz` (143 MB, verified present) | **LOADED** (Postgres) / **PRESENT** (full on-disk drive) — but production code only ever reads the smaller CHECKED-IN tri-county artifact (`src/babylon/data/reference/lodes/`), never this mounted path (CI-no-drive rule) |
| County/hex geometry | TIGER (res-7 pipeline `tools/ingest_tiger_geometry.py`) | LOADED (unchanged from program-11) |
| Terrain polygons | NE physical layers in `natural_earth_vector.sqlite` | PRESENT (taxonomy extension still needed — verified `domain/geography/r8_types.py`'s `_VALID_TERRAIN_TYPES = {LAND, WATER, RESOURCE}` only; no mountain/wetland/desert/forest split exists in code today) |
| Commodity structure fallback | BEA I-O USE/TOTAL_REQ `fact_bea_io_coefficient` | LOADED (same table spec-107 confirmed LOADED) |
| Corridor per-edge capacity/condition DTO | `domain/geography/types.py::InfrastructureLinkState` | **BUILT, engine-orphaned** (FR-108-1) |
| Min-cost-flow solver | none | **ABSENT** — no `scipy.sparse.csgraph`, `network_simplex`, or equivalent found anywhere in `src/` (only `scipy.optimize.linprog`, used for a DIFFERENT purpose — Ollivier-Ricci curvature in `formulas/curvature.py`) |

Loaders for freight/HPMS/NTAD live in the **babylon-data repo** per the
standing owner ruling program-11 already recorded; unchanged by this pass.

## Consumption seams (declared, not inserted)

None of these files are modified by this unit (this spec's own write
surface is `specs/108-transport-substrate/**` only).

| Seam | Location | What the engine-step would do |
|---|---|---|
| Corridor per-edge state | `domain/geography/types.py::InfrastructureLinkState`, `capacity.py::DefaultEdgeCapacityCalculator` | Construct these in production for the first time; extend with `conductivity: float` |
| INFORMAL edge type | `models/enums/topology.py::InfrastructureType` | Add `INFORMAL = "informal"`; name its production stamper |
| BUILD_INFRASTRUCTURE resolver | `engine/actions/__init__.py::VERB_RESOLVERS`, new `engine/actions/build.py` | Register `ActionType.BUILD_INFRASTRUCTURE: resolve_build` |
| Corridor damage (already wired) | `ooda/layer3.py::_propagate_infrastructure`, `engine/actions/attack.py::resolve_attack` | Reconcile the community-scoped `infrastructure` float this seam writes today against the new per-edge `condition` (Director ruling required, item 4) |
| Vol2CirculationStep production composer | `game/trade.py::build_interactive_trade_wiring` (`vol2_step` param, currently always `None` from its sole caller `cli/play.py:195`) | Add `build_vol2_circulation_step()` composing `resolve_lodes_hydration_kwargs()` + `read_hex_county_adjunction()`; pass it at the `cli/play.py` call site |
| Realization-crisis coupling | `domain/economics/circulation/crisis.py::assess_circulation_crisis`, `commodity_overhang` computation in `_compute_county_circulation_state` | Add a stranded-freight-value additive term (D4) |
| Physical-exchange register row | `domain/economics/node_kinds.py::BoundaryEdgeKind.PHYSICAL_EXCHANGE` | First producer — emit register rows for freight crossing the study-area boundary |
| Vol III pricing | `engine/systems/market_scissors.py::MarketScissorsSystem` (position 17.8) | Convert routed freight/labor volumes to dollar value via the existing price⟷value scissors, no new pricing formula |
| σ transfer lever | `engine/systems/economic.py` national-flat `extraction_efficiency`; `formulas/unequal_exchange.py` (registered, zero callers) | Per-edge ε on corridor-routed flows, per spec-107's own declared seam |
| INFRASTRUCTURE_CHANGE narrative event | `models/enums/events.py::EventType.INFRASTRUCTURE_CHANGE` | First producer — BUILD/REPAIR/ATTACK corridor narrative |
| Blocs as alignment | `models/enums/topology.py::NodeType.SOVEREIGN`; `persistence/` `dynamic_external_node_state` | No new primitive — a grouping label over existing sovereigns/external nodes (D6) |

## Director ruling required

> **ALL FIVE RULED 2026-07-27 — see
> `ai/decisions/ADR165_p26_director_rulings_trade_slate.yaml`.** 1: static slice-1 blocs
> (dynamic realignment = chartered future unit). 2: design reframe — the flux overlay is
> the DEMAND SIGNAL feeding the state's budget OODA loop; corridor expansion/repair goes
> through BUILD_INFRASTRUCTURE, no autonomous INFORMAL minting in slice 1. 3: damped
> coefficient in `CapitalVolumeIIDefines`. 4: uniform territory splash. 5: aggregated
> connectivity indicator surfaces in the Archive client (mesh stays invisible). Plus a
> same-session directive: tariff/duty/tax instruments join the trade system, adjusted via
> the P25 Policy/Electoral machinery (first concrete P25↔P26 coupling — own spec section
> in the U5 train).

Recorded here per this worktree's IX.5 discipline (theory-line content is
the Director's, not an agent's to improvise) — none resolved unilaterally by
this spec.

1. **Which blocs exist, and alignment semantics.** Program 26 U5's own
   charter text ("blocs as graph-level alignments over sovereigns") leaves
   open whether a "bloc" is: (a) a static label set fixed at campaign start
   (mirroring spec-101's fixed 8-node `INTERNATIONAL_NODES` enumeration), or
   (b) a dynamic, tick-computed grouping that can shift (e.g., a sovereign
   defecting from one bloc to another under pressure — closer to the
   Balkanization faction-alignment machinery, spec-070). This spec assumes
   (a) is the slice-1 default (matches D6's "labeled grouping" framing and
   avoids inventing new emergent-alignment math this spec's non-overlap
   covenant forbids), but the Director should confirm before U5-code locks
   it in.
2. **INFORMAL edge stamping trigger.** The slime-mold conductivity overlay
   is theory-grounded (Program 11 §"Why", Constitution II.13) but the
   PRECISE threshold at which sustained flux mints a new `INFORMAL` edge
   (vs. just raising `D` on an edge that doesn't yet exist) is a game-design
   / calibration call, not derivable from the constitutional text alone.
3. **Coupling coefficient for FR-108-3's realization-crisis term** (D4):
   whether stranded freight value maps 1:1 into `commodity_overhang` or via
   a scaled/dampened coefficient, and whether that coefficient lives in
   `TransportDefines` or `CapitalVolumeIIDefines`, is a balance decision.
4. **Community-scoped `infrastructure` float vs. corridor-scoped
   `InfrastructureLinkState.condition` — unify or keep separate?** FR-108-5
   surfaces a real fork: `ATTACK_INFRASTRUCTURE`'s existing consequence pass
   (`layer3._propagate_infrastructure`) already writes a single scalar onto
   Territory nodes; this spec's corridor mesh needs PER-EDGE condition. Does
   an attack on a territory degrade every corridor edge touching it
   uniformly (cheap, reuses the existing seam), or does the player/AI target
   a specific edge (richer, needs `action.target_id` to resolve to an edge
   id rather than a territory id — a real `Action` schema question)? This is
   a gameplay-feel decision, not purely technical.
5. **Corridor visibility to the player.** Program-11 owner ruling 1 says
   res-8 stays invisible/non-rendered — but does the AGGREGATED per-R7-pair
   connectivity coefficient (FR-108-2's stated aggregation target) surface
   anywhere in the Archive TUI (e.g., "supply lines to this county are cut")
   even though the underlying mesh doesn't render? Program 26 U6 (Archive
   trade surfaces) will need this answered before its own spec authoring.

## Gate (this unit)

- Docs-only: no `mise run check`/`test:q`/`qa:regression` obligation (no
  source changed). The four files under `specs/108-transport-substrate/`
  are the deliverable.
- Every code claim in this document is file:line cited above or in
  research.md; every data claim is `ls`/`find`-verified against the mounted
  drive during this authoring pass (2026-07-27).
- No file outside `specs/108-transport-substrate/**` touched.
- Five Director-ruling items recorded for the engine-step (and, where
  theory-line, the Director) to carry forward; none resolved unilaterally.
