# research.md — spec-108 Transport Substrate

Verification log backing spec.md's claims. All commands run 2026-07-27 in
this worktree (`101-trade-activation`, tree at the `feature/ratatui-m3`
ancestor per the session's git status) against the mounted
`/media/user/data/babylon-data/` drive, which was PRESENT and readable
during this authoring pass (unlike spec-107's session, where it was
absent — a materially different environment condition worth flagging for
anyone re-running this audit later).

## 1. Program-11's as-built inventory table is stale on file paths

Program-11 (`project/programs/11-transport-substrate.md`, written
2026-07-08) cites `src/babylon/infrastructure/` as the home of the R8
substrate (16 modules named: `r8_types.py`, `r8_mesh.py`, `r8_pipeline.py`,
`r8_aggregation.py`, `snapping.py`, `capacity.py`, `nonlocal_edges.py`, …).

```
$ find .../src/babylon/infrastructure/ -iname '*.py'
ls: cannot access '.../src/babylon/infrastructure/': No such file or directory
```

The package exists, relocated:

```
$ find .../src -iname '*r8*' -o -iname '*infrastructure*'
src/babylon/domain/geography/r8_mesh.py
src/babylon/domain/geography/r8_aggregation.py
src/babylon/domain/geography/r8_pipeline.py
src/babylon/domain/geography/r8_types.py
```

```
$ ls src/babylon/domain/geography/
capacity.py  h3_mesh.py  __init__.py  internet.py  inventory.py
natural_earth_reader.py  nonlocal_edges.py  protocols.py  r8_aggregation.py
r8_mesh.py  r8_pipeline.py  r8_types.py  snapping.py  terrain.py  types.py
```

This matches this worktree's own `ai/anti-patterns.yaml`/CLAUDE.md
correspondence note: "Post-Program-14 package map" — the reorganization
that moved `infrastructure/` under `domain/geography/` predates program-11's
2026-07-08 write-up date by enough that program-11 was ALREADY describing
stale paths when it was ratified, or the move happened after. Either way,
spec.md's citations use the current, verified location.

## 2. Spec-036 is more built than program-11 credits

Program-11's table says: "**BUILT + TESTED, engine-orphaned**... no engine
system consumes it at runtime" — true, but it undersells the DEPTH of what's
built. Reading `domain/geography/types.py` in full surfaces:

- `InfrastructureLinkState` (lines 99-137): `capacity: dict[str, float]`
  keyed by `FlowCategory`, `condition: float ∈ [0,1]` default 1.0, and a
  method `effective_capacity(category) -> capacity.get(category, 0) *
  condition`. This is EXACTLY program-11's US1 charter request
  ("per-edge state `{capacity, condition ∈ [0,1], conductivity D}`" — two
  of three fields, verbatim, already exist as a frozen Pydantic model).
- `EdgeCapacityResult` (140-159): aggregate + natural + total capacity per
  `FlowCategory`, for a source/target hex pair.
- `NonlocalEdgeState` (223-245): source/target vertex + `InfrastructureLinkState`
  + `distance_km` + `locality_class` — the DTO shape `AIR_LINK`/
  `SHIPPING_LANE` slice-2 edges would use.
- `JunctionState`/`VertexState` (167-216): junction infra at triple-junction
  vertices, ALSO carrying `condition`.
- `capacity.py::DefaultEdgeCapacityCalculator.compute_edge_capacity`: sums
  `link.effective_capacity(category)` across links, adds
  `natural_capacity_coefficient` for LAND-LAND COMMUTER/CONSCIOUSNESS edges,
  zeroes WATER-WATER edges entirely (FR-013 from spec-036).

Verified zero production construction:

```
$ rg -n "DefaultEdgeCapacityCalculator\(|DefaultInfrastructureInventory\(" --type py -g '!tests/*'
(no output)
```

`specs/036-infrastructure-topology/spec.md` itself still reads
**`Status: Draft`** at its header — confirming this is a fully speculative,
never-shipped feature spec whose CODE nonetheless landed (a "spec trapped
in implementation" pattern the Constitution's Amendment Q / VIII.13 exists
to name, though this is the inverse — implementation ahead of a spec that
was never marked Implemented).

**Conclusion for spec.md FR-108-1/D1**: the engine-step should EXTEND
`InfrastructureLinkState` (add `conductivity: float`) rather than build a
parallel corridor-edge model. The only genuinely missing piece at the DTO
layer is the `conductivity` field itself and the mesh-graph structure that
holds edges as first-class routable objects (today's DTOs describe a single
edge's state; nothing assembles them into a connected, traversable graph
with a min-cost-flow or conductivity-update operation over it).

## 3. R8LinearFeature is a per-cell marker, not an edge

```
$ sed -n '167,191p' src/babylon/domain/geography/r8_types.py
```
shows `R8LinearFeature.h3_index: str` — "R8 cell this feature passes
through" — singular. It is not `(source_h3, target_h3)`. This confirms the
corridor MESH (an edge-and-capacity graph over R8 cells) is new
construction layered on top of the R8 cell inventory + `InfrastructureLinkState`,
not something `r8_types.py` already provides. Program-11's US1 charter
phrase "aggregates to per-R7-pair connectivity coefficients... pattern:
existing `r8_aggregation.py`" is the correct precedent to imitate for the
AGGREGATION direction (R8→R7); the mesh-EDGE construction itself has no
existing precedent to imitate at the R8 level.

## 4. No min-cost-flow solver exists anywhere in this codebase

```
$ rg -ni "min.cost.flow|min_cost_flow|network_simplex|linprog" --type py -g '!tests/*'
src/babylon/formulas/curvature.py:26:from scipy.optimize import linprog
src/babylon/formulas/curvature.py:225:    result = linprog(
web/game/engine_bridge.py:4287:  Spec 111 C2. Constitution II.13's transport substrate (min-cost-flow...
src/babylon/domain/economics/lodes_commute_matrix.py:14/239: (docstring references only)
src/babylon/engine/systems/vol2_circulation.py:39: (docstring reference only)

$ rg -n "csgraph" --type py
(no output)
```

`linprog` exists but is used for Ollivier-Ricci curvature (a totally
different LP, transportation-distance-flavored but not this problem).
`Vol2CirculationStep.step()` (`engine/systems/vol2_circulation.py:146-356`)
implements:

```
v[A, t+1] = sum_j(OD[j, A] × v[j, t] / row_sum[j])   for in-area A
```

— a fixed proportional redistribution keyed to HISTORICAL OD shares
(`year_matrix.matrix`, loaded once per year from LODES commute data), not an
optimization that respects capacity constraints or minimizes cost subject to
supply/demand. It never enforces `effective_capacity`. This is the right
PATTERN (deterministic sparse-matrix-vector, no RNG, CSR representation per
Constitution II.12) but not directly reusable code for freight, because
freight routing genuinely needs a capacitated flow algorithm: FAF5 O-D
demand by SCTG commodity must be assigned to specific paths through the
corridor mesh subject to each edge's `effective_capacity`, with excess
demand becoming "unrouted" (feeding the realization-crisis coupling).

**Candidate approaches for the engine-step** (not decided by this spec,
listed for the design doc that phase C of plan.md will need):
- A deterministic successive-shortest-augmenting-path algorithm
  hand-implemented over the sparse corridor graph (full control over tie-
  breaking for determinism; more code to write and test).
- `scipy.sparse.csgraph.min_weight_full_bipartite_matching` or
  `dijkstra`/`shortest_path` primitives composed into a manual flow
  algorithm (scipy provides shortest-path primitives, not a full min-cost-
  flow solver with capacities — would still need a wrapper).
- A third-party min-cost-flow library (e.g., NetworkX's
  `network_simplex` — but NetworkX was REMOVED from this codebase per
  Amendment L/ADR052 rustworkx migration; reintroducing it for this ONE
  algorithm would be a regression the engine-step must not casually do).
  rustworkx itself does not (as of this survey) expose a capacitated
  min-cost-flow primitive — worth double-checking against the pinned
  rustworkx version at implementation time.
- OR-Tools / a dedicated LP solver — a new dependency, higher-weight
  decision requiring the same "earn its keep" (III.10) justification any
  new library needs in this codebase.

Each of these determinism and dependency-footprint tradeoffs is an
engine-step design decision, explicitly deferred by this spec.

## 5. BUILD_INFRASTRUCTURE has no resolver; ATTACK_INFRASTRUCTURE does

```
$ rg -n "ATTACK_INFRASTRUCTURE|BUILD_INFRASTRUCTURE" --type py -g '!tests/*'
```
surfaced (relevant excerpt):
- `models/enums/actions.py:83-84`: both ActionTypes declared.
- `engine/actions/attack.py`: full resolver (`resolve_attack`), registered
  in `VERB_RESOLVERS` (`engine/actions/__init__.py:66`).
- `ooda/layer3.py:145-161`: `_propagate_infrastructure` applies BOTH
  BUILD's and ATTACK's effects to a Territory's `infrastructure` scalar —
  but only ATTACK has a player-verb resolver feeding into it via
  `ActionResult`.
- `engine/actions/__init__.py`'s `VERB_RESOLVERS` dict lists nine verbs;
  `BUILD_INFRASTRUCTURE` is not among them, and `ls src/babylon/engine/
  actions/` shows no `build.py` file.

`resolve_attack`'s own docstring is explicit about the seam: "Layer 3
writes `infrastructure` onto the target node... The resolver's own
material effect is to raise the acting org's `heat`... Layer 3 applies the
infrastructure decrement." This means `resolve_build` (to be written) needs
the SAME shape: return an `ActionResult` carrying a `BUILD_INFRASTRUCTURE`
action so `_propagate_infrastructure` can apply the (positive) delta on the
target — layer3 already branches on `action_type ==
ActionType.BUILD_INFRASTRUCTURE` (`layer3.py:159`) vs. `ATTACK_INFRASTRUCTURE`
(`:161`), so the consequence-pass HALF of BUILD already exists; only the
player-verb dispatch half is missing.

## 6. Two separate "infrastructure condition" concepts coexist

- **Community-scoped** (existing, wired): `Territory.infrastructure: float`
  (a single scalar), decayed by
  `formulas/community.py::calculate_infrastructure_decay` (registered in
  `formula_registry.py:126`), consumed by `engine/systems/community.py:634-657`,
  and mutated by `ooda/layer3._propagate_infrastructure` for both BUILD and
  ATTACK.
- **Corridor-scoped** (spec-036, engine-orphaned): `InfrastructureLinkState.condition`
  — one value PER EDGE, not per territory.

`world_state.py:121-125` even documents the ATTACK/BUILD target as "Territory"
scoped: "ATTACK/BUILD_INFRASTRUCTURE target node (ooda/layer3.py:_propagate_
...) / NPC CIVIL_SOCIETY BUILD_INFRASTRUCTURE — targets a territory." This
confirms today's verb targets a TERRITORY, not an edge — a real design fork
the engine-step must resolve (spec.md's Director ruling 4) before wiring
`resolve_build`/extending `resolve_attack` to touch corridor `condition`.

## 7. EventType / BoundaryEdgeKind dormant-construct audit

```
$ rg -n "INFRASTRUCTURE_CHANGE" --type py
src/babylon/models/enums/events.py:125:    INFRASTRUCTURE_CHANGE = "infrastructure_change"  # BUILD or ATTACK infrastructure
```
Zero emitters — declared, comment explicitly anticipates this exact use
case ("BUILD or ATTACK infrastructure").

```
$ rg -n "PHYSICAL_EXCHANGE" --type py
src/babylon/domain/economics/node_kinds.py:47:    PHYSICAL_EXCHANGE: FAF freight or USGS minerals.
src/babylon/domain/economics/node_kinds.py:54:    PHYSICAL_EXCHANGE = "physical_exchange"
```
Zero producers — declared, docstring explicitly anticipates FAF freight.
Both are exactly the "reuse over recreation" case this worktree's CLAUDE.md
DRY rule asks for: the engine-step should NOT invent
`CORRIDOR_CONSTRUCTED`/`TRADE_FLOW_RECORDED`-style new EventTypes/register
kinds when these two already exist, named for precisely this purpose, and
unused.

`models/enums/events.py` was also grepped for any existing
"realization"/"transport"/"corridor" signal and found none — confirming
FR-108-3's claim that the realization-crisis coupling needs either a new
EventType (for the narrative-layer signal) or, more likely per D4, simply
flows through the EXISTING `CirculationCrisisAssessment.realization_crisis`
boolean with no new EventType at all (crisis detection already has its own
signal path independent of EventType; check
`domain/economics/circulation/crisis.py` callers at implementation time to
confirm whether an EventType is even needed here or whether the existing
crisis-assessment plumbing already narrates it).

## 8. Realization-crisis landing point: two dormant candidates, not one

```
$ rg -n "unrealized_value|RealizationCrisis|realization_crisis" --type py -g '!tests/*'
```
surfaced TWO distinct constructs sharing similar names:
1. `CirculationCrisisAssessment.realization_crisis` (live boolean,
   `domain/economics/circulation/crisis.py::assess_circulation_crisis`,
   driven by `commodity_overhang_threshold` from `CapitalVolumeIIDefines`)
   — LIVE, feeds `tick_realization_crisis` graph attr
   (`web/game/engine_bridge.py:8158` reads it), feeds
   `realization_crisis_share` in the dialectics catalog
   (`domain/dialectics/instances/catalog.py:309-350`).
2. `RealizationCrisis` (a class cited in `capital_vol2.py`'s exclusion
   note as having "zero production callers" via its sole constructor
   `compute_realization_metrics`) — DORMANT, a different, apparently
   never-wired construct with an overlapping name.

`capital_vol2.py:20-24`'s own comment flags this exact ambiguity ("the sole
production constructor of `RealizationCrisis`, has zero production
callers") without resolving which of the two is meant to be THE realization-
crisis concept long-term. Spec.md flags this explicitly as an engine-step
audit item rather than silently picking one.

## 9. `assess_circulation_crisis` signature (confirms D4's landing point)

```python
def assess_circulation_crisis(
    circuit_state: CircuitState,
    turnover: TurnoverProfile,
    inventory: InventoryState,
    reproduction_balance: ReproductionBalance | None,
    reproduction_analysis: ReproductionAnalysis | None,
    commodity_overhang_threshold: float = 0.3,
    liquidity_crisis_ratio: float = 0.1,
) -> CirculationCrisisAssessment:
```
(`domain/economics/circulation/crisis.py:42-50`). `commodity_overhang` is
computed from `circuit_state` (a `CircuitState`, presumably tracking
commodity-capital share of the circuit — M-C-P-C'-M'), not passed in
directly. The engine-step's coupling point is therefore upstream, in
whatever builds `circuit_state` for `_compute_county_circulation_state`
(the Volume II tick wiring caller) — the stranded-freight-value term needs
to land in THAT construction, not as a new parameter to
`assess_circulation_crisis` itself (which would break its existing pure-math
signature/callers).

## 10. NodeType / EdgeType / InfrastructureType vocabulary audit (D2/D3/D6)

Full enum bodies read from `models/enums/topology.py`:
- `NodeType` (12-76): 7 production-stamped members (TERRITORY, SOCIAL_CLASS,
  ORGANIZATION, INSTITUTION, INDUSTRY, SOVEREIGN, FACTION) + 6
  declared-not-stamped fixture members (HEX, COMMUNITY, PERSON, KEY_FIGURE,
  ENTITY, EXTERNAL, COUNTY). `SOVEREIGN` (spec-070) is the correct anchor
  for bloc alignment — already carries `CLAIMS`/`ADMINISTERS` edges.
- `EdgeType` (78-127): the MAIN graph's relation vocabulary
  (EXPLOITATION/SOLIDARITY/TRIBUTE/WAGES/CLIENT_STATE/TENANCY/ADJACENCY/…).
  No transport-specific member exists here, confirming D2: transport edges
  are correctly a SEPARATE vocabulary (`InfrastructureType`), not additions
  to this enum.
- `InfrastructureType` (177-201): HIGHWAY, ARTERIAL, LOCAL_ROAD, RAIL,
  PIPELINE, TRANSMISSION, SHIPPING_LANE, AIR_LINK — 8 members, matching
  Constitution II.13's road-tier/RAIL/SHIPPING_LANE/AIR_LINK exactly, PLUS
  two members (PIPELINE, TRANSMISSION) the constitutional clause doesn't
  mention at all (spec-036 scope, energy-flow-focused, predates Amendment
  O). **`INFORMAL` is the only clause-mandated member missing.**

## 11. Data trove verification (mounted, readable, this session)

```
$ ls /media/user/data/babylon-data/lodes/
od/  us_xwalk.csv.gz
$ ls /media/user/data/babylon-data/lodes/od/ | head
ak_od_main_JT00_2010.csv.gz ... al_od_main_JT00_2021.csv.gz ...
$ find /media/user/data/babylon-data/freight -maxdepth 3
freight/faf/water_origin_factors.csv
freight/faf/FAF5.7.1_State_2018-2024.csv
freight/faf/region/FAF5.7.1_2018-2024.csv
freight/faf/county/01_Alabama.zip
freight/faf/FAF5-County-Level-Estimates-Technical-Report.pdf
... (truck/rail/water/pipeline origin+destination factor CSVs, all present)
$ ls /media/user/data/babylon-data/dot/
HPMS_Spatial_All_Sections_-_2024.csv
hpms-spatial.json / hpms-spatial-metadata.json
NTAD_Aviation_Facilities_*.geodatabase
NTAD_Intermodal_Freight_Facilities_{Air_to_Truck,Marine_Roll_on_Roll_off,
  Pipeline_Terminals,Rail_TOFC_COFC}_*.geodatabase
NTAD_Military_Bases_*.geodatabase
NTAD_North_American_Rail_Network_Lines_*.geodatabase
```

Full 50-state LODES OD + all NTAD/HPMS/FAF freight source files confirmed
present on this box. This CONTRADICTS neither spec-107's finding (which
concerned different reference-DB tables, e.g. FAAt3.1ESI fixed assets,
confirmed STAGED/absent there) nor the CI-no-drive rule (which governs
production CODE, not this authoring session's ability to verify claims).

## 12. Vol2CirculationStep production-constructor chain (FR-108-10 detail)

Traced end-to-end:

1. `domain/economics/lodes_study_area.py` — `LODES_ARTIFACT_ROOT =
   src/babylon/data/reference/lodes/`, `LODES_ARTIFACT_CROSSWALK`,
   `lodes_tri_county_hexes_res7()` (cached, computed from a hardcoded
   Wayne+Oakland+Macomb bounding polygon), `LODES_STUDY_AREA_STATES =
   frozenset({"26"})`.
2. `engine/headless_runner/lodes_hydration.py::resolve_lodes_hydration_kwargs(scope_fips)`
   — returns the four `initialize_session` LODES kwargs, or `None` if
   `scope_fips` doesn't intersect `DETROIT_TRI_COUNTY_FIPS` (honest-absence,
   no fabrication).
3. `persistence/postgres_initialization.py::initialize_session` (lines
   657-900+) — when all four LODES kwargs are non-`None`, constructs
   `LODESCommuteMatrixLoader(lodes_root=..., crosswalk_path=...,
   study_area_hexes=..., study_area_states=...)` (line 881) and persists
   each scenario year to Postgres. **This is a real, live production
   constructor of the LOADER** — contradicts nothing in spec-101/U2's
   audit, which specifically said `Vol2CirculationStep` (the SUB-STAGE, one
   level up) has zero production writers, not the loader itself.
4. `persistence/hex_hydrator.py::read_hex_county_adjunction(runtime,
   session_id)` (line 276) — production-ready, exercised only by five
   `tests/integration/*.py` files, never by `src/`.
5. `game/trade.py::TradeWiring.vol2_step: Vol2CirculationStep | None = None`
   (line 86) and `build_interactive_trade_wiring(..., vol2_step:
   Vol2CirculationStep | None = None, ...)` (line 110) — the carrier and
   composer both ALREADY accept a constructed step; nothing ever builds one
   to pass in.
6. `cli/play.py:195` — the sole production call site of
   `build_interactive_trade_wiring`, does not pass `vol2_step=`.
7. `game/session.py:1003-1004` — `if self._trade.vol2_step is not None:
   context["vol2_step"] = self._trade.vol2_step` — the consumption side is
   fully wired and WAITING; it is a pure pass-through with no fallback
   construction of its own (correctly — composition-root logic doesn't
   belong in the session class).
8. `sentinels/seam_algebra/registry.py:449-460` — the
   `vol2_circulation_vol2_step` gate row is marked CLOSED as of ADR162 (P26
   U2) because `game/session.py` is now a genuine "supplier file" for the
   `context["vol2_step"]` write — but the sentinel checks WIRING PATH
   EXISTENCE, not "does real non-None data ever flow." Both facts are true
   simultaneously: the sentinel is correctly green, AND no interactive
   campaign has ever run with a real `Vol2CirculationStep`. Spec.md's
   FR-108-10 names the exact missing composer
   (`build_vol2_circulation_step()`) to close this for real.

## 13. GameDefines category pattern (for TransportDefines' shape)

`config/defines/capital_vol2.py::CapitalVolumeIIDefines` read in full as the
template: every field is a `pydantic.Field` with a `description=` that
traces the coefficient to (a) the exact call site consuming it, (b) a
theoretical/empirical derivation, (c) which wiring unit introduced it. This
is the pattern `TransportDefines` must follow (tasks.md's proposed fields
each need this same three-part justification when actually written).
`InfrastructureDefines` (`config/defines/territory.py:486+`) ALREADY exists
for spec-036's per-type base capacity coefficients (`highway_freight`,
`arterial_commuter`, etc., all marked `SYNTHETIC` — game-design values, not
derived from HPMS/FAF data) — the engine-step should ADD decay/conductivity/
maintenance fields to a NEW `TransportDefines` category (matching
program-11's own naming: "defines-gated (`TransportDefines`...)") rather
than growing `InfrastructureDefines` further, since the latter's scope is
"capacity coefficients + internet ops," not corridor lifecycle
(decay/repair/conductivity) — keeping the categories aligned with Feature
036 vs. Program 11's distinct concerns.

## 14. System position registry (`_DEFAULT_SYSTEMS`)

Full position ladder read from `engine/systems/*.py` `position: ClassVar[float]`
declarations: 1.0 Vitality, 2.0 Territory, 2.5 Substrate, 3.0 Production,
5.0 ReserveArmy, 6.0 Community, 7.0 Lifecycle, 8.0 Solidarity, **9.0
ImperialRent** (hosts the `vol2_step` sub-stage), 10.0
DispossessionEvents, 11.0 Decomposition, 12.0 ControlRatio, 13.0
Metabolism, 14.0 OODA, 14.5 FactionInfluence, 14.7 Doctrine, 15.0 Survival,
16.0 Struggle, 17.0 Ideology, 17.4 Reactionary, 17.42 Allegiance, 17.45
Electoral, 17.47 Policy, 17.5 Sovereignty, **17.8 MarketScissors**, 18.0
Contradiction, 19.0 ContradictionField, 20.0 FieldDerivative, 20.5
CollapseTransition, 21.5 WealthDistribution, 22.0 EpistemicHorizon.
Candidate gap slots for a new TransportSystem: 9.5 (adjacent to
ImperialRent) or 17.9 (adjacent to MarketScissors) — both real gaps in the
float sequence, confirmed no existing System currently claims either value.
