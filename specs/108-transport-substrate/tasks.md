# tasks.md — spec-108 Transport Substrate

This is the **implementation contract** the engine-step (Program 26 U5,
post-P25) executes against. Every task names its files, cites the seam it
closes, and states its Director-ruling dependency if any. This spec (the
authoring pass) does NOT execute any of these tasks — zero source files are
touched by this document.

Timing note: several tasks touch files Program 26 §3's non-overlap covenant
currently locks as P25 territory (`engine/systems/*`,
`models/enums/events.py`, `formulas/__init__.py`, `models/world_state.py`,
`tests/baselines/*.json`, `tools/regression_scenarios.py`,
`web/game/engine_bridge.py`, `config/defines/politics.py` +
`defines.yaml`). Program 26 §4 already marks U5 **post-P25** for exactly
this reason — none of these tasks are safe to start before the P25 lane
merges to `dev`.

## Task group 1 — Corridor mesh + degradation plumbing (Phase A)

### T1 — Extend `InfrastructureLinkState` with `conductivity`

- **Files**: `src/babylon/domain/geography/types.py`
  (`InfrastructureLinkState`), `src/babylon/domain/geography/capacity.py`
  (if the aggregation needs to account for conductivity separately from
  capacity — TBD at implementation time).
- **Seam closed**: FR-108-1/D1 — reuse, not duplicate, the existing
  capacity/condition DTO.
- **Not gated on a Director ruling.**

### T2 — Build the corridor mesh graph structure

- **Files**: new module, e.g. `src/babylon/domain/geography/corridor_mesh.py`.
- **Seam closed**: FR-108-2. Assembles `InfrastructureLinkState`-typed
  edges over R8 cells touched by NE/HPMS/NTAD features + junctions, SPARSE
  (never full national res-8 tiling — storage-budget gate, spec-087/089).
  Terrain taxonomy extension (mountain/wetland/desert/forest/water) lives
  here or in `domain/geography/terrain.py` (existing `DefaultTerrainClassifier`
  currently only distinguishes LAND/WATER/RESOURCE per
  `r8_types.py::_VALID_TERRAIN_TYPES`).
- **Not gated on a Director ruling**, but the terrain traversal-cost TABLE
  shape (impassability representation) is an open engineering decision:
  research.md flags that a literal `inf` float is a poor `defines.yaml`
  citizen (YAML/JSON serialization, Pydantic `Field` bounds) — prefer an
  explicit `impassable: bool` alongside a bounded `traversal_cost: float`,
  or exclude impassable terrain classes from the mesh edge set entirely at
  construction time (never encode "infinity" as data).

### T3 — Add `InfrastructureType.INFORMAL` + its production stamper

- **Files**: `src/babylon/models/enums/topology.py` (`InfrastructureType`),
  `src/babylon/sentinels/vocabulary/` (whichever rule file enumerates
  stampable types — verify against `EXTRA_STAMPABLE_ATTRIBUTES`/production-
  stamp registry at implementation time), the slime-mold overlay module
  (T7) as the actual stamper.
- **Seam closed**: D3 — the one real II.13 vocabulary gap.
- **Director ruling required**: item 2 (INFORMAL stamping trigger
  threshold).

### T4 — `engine/actions/build.py::resolve_build` + `VERB_RESOLVERS` registration

- **Files**: new `src/babylon/engine/actions/build.py` (mirror
  `attack.py`'s shape: acting-org heat/AP cost via `services.defines.ooda`,
  return an `ActionResult` carrying `ActionType.BUILD_INFRASTRUCTURE` so
  `ooda/layer3.py::_propagate_infrastructure`'s EXISTING `BUILD_INFRASTRUCTURE`
  branch — line 159, already present — applies the positive delta);
  `src/babylon/engine/actions/__init__.py` (`VERB_RESOLVERS` dict — add
  `ActionType.BUILD_INFRASTRUCTURE: resolve_build`).
- **Seam closed**: FR-108-5's sharpest, most concrete gap. Independently
  shippable — does not require T2/T5/T6 (routing) to exist first, since
  `_propagate_infrastructure`'s BUILD branch already exists and currently
  just has no player-verb path feeding it.
- **Director ruling required**: item 4 (territory-scoped vs. edge-scoped
  target) determines whether `action.target_id` resolves against a
  Territory node (reuse today's shape, zero `Action` schema change) or a
  corridor edge id (needs a new target-resolution path). **This task
  should default to the Territory-scoped shape if the Director ruling is
  not yet available**, since that is byte-compatible with today's
  `ATTACK_INFRASTRUCTURE` targeting and does not foreclose a later edge-
  scoped extension — but implementers must not silently assume this; flag
  it in the PR body per this worktree's escalation discipline.

### T5 — `TransportDefines` category

- **Files**: new `src/babylon/config/defines/transport.py`;
  `src/babylon/config/defines/_assembler.py` (add
  `transport: TransportDefines = Field(default_factory=TransportDefines)`
  following the exact `capital_vol2`/`infrastructure` pattern already in
  the file); `src/babylon/data/defines.yaml` (regenerate via `uv run python
  tools/generate_defines_config.py` — never hand-edit); sync guard
  `tests/unit/config/test_constants_sync.py` runs automatically.
- **Proposed exact tunables** (each needs a full three-part
  `description=` per the `capital_vol2.py` pattern when actually written —
  names/types/defaults below are this spec's proposal, not final):
  - `enabled: bool = False` — the master gate. Program-11's own constraint:
    "defines-gated... default OFF → baselines byte-identical." Every other
    field below is inert until this is `True`.
  - `condition_decay_rate_per_tick: float` — base per-tick decay applied
    to every corridor edge's `condition`, calibrated against the HPMS
    pavement-condition distribution (US3).
  - `condition_decay_flux_coefficient: float` — additional decay
    proportional to `|Q|` (flow volume) — "degrades with use," not just
    neglect.
  - `maintenance_condition_restore_rate: float` — the `BUILD_INFRASTRUCTURE`
    repair-mode effect size (how much `condition` a repair action restores
    per AP/resource spent).
  - `construction_base_condition: float = 1.0` — starting `condition` for a
    newly `BUILD_INFRASTRUCTURE`-constructed edge.
  - `state_maintenance_budget_share: float` — fraction of state budget
    spent as ambient "faux frais of circulation" maintenance (offsets decay
    without a player action, per US3's text).
  - `conductivity_ema_alpha: float` — the α in `D(t+1) = (1−α)·D + α·|Q|`
    (slime-mold EMA).
  - `conductivity_informal_mint_threshold: float` — sustained-`D` threshold
    above which an `INFORMAL` edge is minted where none existed (Director
    ruling 2 supplies the calibration philosophy; this is its numeric
    knob).
  - `conductivity_prune_threshold: float` — `D` floor below which an
    `INFORMAL` edge is removed (disuse die-back, US3's "the slime-mold
    decay term IS the disuse mechanic").
  - `terrain_traversal_cost: dict[str, float]` — per-taxonomy-class cost
    (mountain/wetland/desert/forest; LAND/WATER/RESOURCE stay as today's
    baseline). Structured mapping, not a scalar — matches
    `InfrastructureDefines`'s existing per-type field-naming convention
    (`{type}_{category}`) if flattened, or a genuine nested dict if the
    Pydantic schema supports it cleanly (implementation-time call).
  - `terrain_impassable_classes: frozenset[str]` — taxonomy classes
    excluded from the routable graph entirely (T2's "exclude, don't encode
    infinity" resolution).
  - `unrouted_demand_overhang_coefficient: float` — D4's coupling
    coefficient (stranded freight value → `commodity_overhang` delta).
    **Open question this task must resolve**: whether this field belongs
    in `TransportDefines` or `CapitalVolumeIIDefines` (Director ruling 3
    leaves both open) — this spec's default recommendation is
    `TransportDefines`, since the coefficient is about how MUCH of a
    transport failure translates to economic consequence, a transport-side
    knob, not a Vol II circulation-side one; but the alternative is
    defensible and not resolved here.

### T6 — Reconcile community-scoped vs. corridor-scoped condition (Director ruling 4 dependent)

- **Files**: `src/babylon/ooda/layer3.py` (`_propagate_infrastructure`),
  possibly `src/babylon/engine/actions/attack.py` (if the Director ruling
  changes ATTACK's target resolution too, not just BUILD's).
- **Blocked** on Director ruling 4. Not started until answered.

## Task group 2 — Vol2CirculationStep production composer (Phase B, independent of Group 1)

### T7 — `build_vol2_circulation_step()` composer

- **Files**: `src/babylon/game/trade.py` (new function, sibling to
  `build_interactive_trade_wiring`) OR a new small module if `trade.py`
  would grow past this worktree's function-length discipline (Power-of-10
  rule 3, ~100 lines/function) — implementer's call. Composes:
  `resolve_lodes_hydration_kwargs(scope_fips)` →
  `LODESCommuteMatrixLoader(**kwargs)` (mirrors
  `postgres_initialization.py:881`'s existing construction exactly) +
  `read_hex_county_adjunction(runtime, session_id)`
  (`persistence/hex_hydrator.py:276`) → `Vol2CirculationStep(od_loader=...,
  hex_county_adjunction=...)`.
- **Files (call site)**: `src/babylon/cli/play.py` — pass
  `vol2_step=build_vol2_circulation_step(...)` into the existing
  `build_interactive_trade_wiring(...)` call (`:195`), scoped to the SAME
  `DETROIT_TRI_COUNTY_FIPS` the checked-in LODES artifact covers (research
  §12 — the substrate's slice-1 geography is bound to this limit, not
  national).
- **Seam closed**: FR-108-10. Honest-`None` behavior preserved: when
  `resolve_lodes_hydration_kwargs` returns `None` (scope doesn't intersect
  the tri-county area), `build_vol2_circulation_step` returns `None` too —
  no fabricated step, matching the existing degraded-path discipline
  (`TradeDataUnavailableError`/loud-warning precedent in `cli/play.py`).
- **Not gated on a Director ruling.**
- **Sentinel row**: `sentinels/seam_algebra/registry.py`'s
  `vol2_circulation_vol2_step` row is already closed (ADR162); this task
  does not need a NEW sentinel row, but should confirm the existing row's
  `supplier_files` comment stays accurate once real (non-`None`) data
  starts flowing through it — currently the comment only claims the WIRING
  PATH exists, not that live data flows; update the comment if it implies
  more than T7 actually delivers.

## Task group 3 — Routing (Phase C)

### T8 — Min-cost-flow solver design + implementation

- **Files**: new module, e.g.
  `src/babylon/domain/economics/transport_routing.py` or
  `src/babylon/domain/geography/routing.py` (naming call at implementation
  time — geography-layer since it operates on the corridor mesh, or
  economics-layer since it's fundamentally a Vol II/III flow computation;
  either is defensible, pick based on which package's existing import
  layering — `kernel < models/formulas < topology < domain < persistence <
  engine` — the routing module's dependencies naturally fall into).
- **Seam closed**: FR-108-3, first bullet. Candidate algorithms surveyed in
  research.md §4 — **this task must pick one and justify it against
  Constitution III.7 determinism (static loop bounds, no RNG, reproducible
  tie-breaking) and III.10 Earn-Its-Keep (any new dependency needs
  justification)** before writing code.
- **Not gated on a Director ruling**, but genuinely substantial new
  algorithmic work — flag for careful TDD red-phase test authoring
  (deterministic fixture graphs with known optimal flows) before
  implementation, per this worktree's TDD discipline.

### T9 — Slime-mold conductivity EMA + INFORMAL minting/pruning

- **Files**: same module as T8, or a sibling
  (`conductivity.py`); consumes T3's `InformalType.INFORMAL` +
  T5's `conductivity_*` defines.
- **Seam closed**: FR-108-3, second bullet.
- **Director ruling required**: item 2.

### T10 — Unrouted-demand → `commodity_overhang` coupling

- **Files**: `src/babylon/domain/economics/circulation/crisis.py` (read-
  only reference — do NOT change `assess_circulation_crisis`'s signature,
  per research.md §9's finding that the coupling point is upstream);
  whatever module builds `CircuitState` for
  `_compute_county_circulation_state` (the Volume II tick wiring caller —
  locate at implementation time; likely
  `domain/economics/tick/system/__init__.py` per `capital_vol2.py`'s own
  citations, though that file is P25/covenant-sensitive — verify current
  ownership before touching).
- **Seam closed**: FR-108-3 third bullet, D4.
- **Director ruling required**: item 3 (coupling coefficient value +
  its defines-category home).
- **Prerequisite audit**: resolve research.md §8's `CirculationCrisisAssessment`
  vs. `RealizationCrisis` ambiguity BEFORE writing this task's code — landing
  on the wrong one re-creates a dormant-construct problem instead of closing
  one.

## Task group 4 — Physical-exchange + bloc alignment (Phase D, post-U4)

### T11 — `BoundaryEdgeKind.PHYSICAL_EXCHANGE` first producer

- **Files**: the routing module (T8) or a new boundary-emission helper
  mirroring `Vol2CirculationStep.step()`'s existing register-row emission
  pattern (`engine/systems/vol2_circulation.py:296-317` — imitate this
  shape exactly: `register.record(session_id=..., tick=..., source_node_id=...,
  source_kind=..., dest_node_id=..., dest_kind=..., flow_type=
  BoundaryEdgeKind.PHYSICAL_EXCHANGE, magnitude=...)`).
- **Seam closed**: FR-108-8.
- **Depends on**: T8 (routing must exist to know what crosses the
  boundary), U3 (FAF freight checked-in artifact must exist as the demand
  source), U4 (Φ-attribution model — informs how much of the routed
  magnitude is Φ-relevant vs. ordinary trade).

### T12 — Bloc alignment (label over `NodeType.SOVEREIGN`/external nodes)

- **Files**: TBD at implementation time — likely a new lightweight
  lookup/registry module rather than any graph mutation (D6: no new
  NodeType/EdgeType). Possibly `domain/economics/` alongside the existing
  `county_exposure.py`/bilateral-trade machinery, or `domain/geopolitics/`
  if that package exists by the time U5 starts (check at implementation
  time — this worktree's package layout evolves fast, per the repeated
  "moved since program doc was written" pattern this spec already hit
  once).
- **Seam closed**: FR-108-6.
- **Director ruling required**: item 1 (bloc semantics: static label set
  vs. dynamic).

### T13 — Extend conservation invariant into qa:regression / conservation auditor

- **Files**: wherever `imperial_rent_phi_week_distribution`'s evaluator
  lives today (spec-101 FR-101-5 — the conservation auditor estate;
  locate via `rg -n "imperial_rent_phi_week_distribution"` at
  implementation time) — add the FR-108-9 freight/value closure check
  alongside it, same relative-residual reporting convention (spec-101 D4).
- **Seam closed**: FR-108-9.
- **Depends on**: T11 (needs `PHYSICAL_EXCHANGE` rows to exist before their
  sum can be checked against anything).

## Task group 5 — Vol III pricing + σ transfer lever wiring

### T14 — Route through `MarketScissorsSystem` for dollar valuation

- **Files**: the routing module (T8) as a CONSUMER of
  `engine/systems/market_scissors.py`'s existing price⟷value machinery —
  no changes to `market_scissors.py` itself anticipated (P25/covenant-
  sensitive file; confirm it's still clear of P25 changes before touching,
  per Program 26 §3's living list).
- **Seam closed**: FR-108-7, first bullet.

### T15 — Per-edge σ extraction-efficiency (spec-107's declared seam)

- **Files**: `engine/systems/economic.py` (the national-flat
  `extraction_efficiency` — P25/covenant-sensitive, confirm clearance),
  `formulas/unequal_exchange.py` (already registered, zero callers —
  register the first caller here).
- **Seam closed**: FR-108-7, second bullet. This is explicitly spec-107's
  own declared seam, not new design — U5 is simply the unit that finally
  calls it.
- **Depends on**: spec-107's Director ruling #1 (composite-σ combination
  method) being resolved, since this task consumes σ VALUES that ruling
  determines how to compute.

## Summary table — new EventTypes, defines category, system position

| Deliverable | Exact proposal | File | Director-gated? |
|---|---|---|---|
| New EventType | **None** — reuse `EventType.INFRASTRUCTURE_CHANGE` (`models/enums/events.py:125`, zero current emitters) for corridor BUILD/REPAIR/ATTACK narrative | n/a (reuse) | No |
| Realization-crisis signal | **None new** — routes through existing `CirculationCrisisAssessment.realization_crisis` (D4); confirm no EventType is independently needed once T10's audit (research §8) completes | n/a (reuse, pending audit) | No |
| GameDefines category | `transport: TransportDefines` (T5's ~13 fields listed above) | `config/defines/transport.py` (new) | Partial (2 fields gated) |
| Candidate system position | **9.5** (primary — adjacent to ImperialRent @9.0) or **17.9** (alternative — adjacent to MarketScissors @17.8) | `engine/systems/transport.py` (new, not created by this spec) | No (engineering call) |
| New InfrastructureType member | `INFORMAL = "informal"` | `models/enums/topology.py` | Yes (ruling 2, stamping threshold) |
| New NodeType/EdgeType | **None** (D2, D6 — blocs and corridors both reuse existing vocabulary) | n/a | No |

## Director ruling required — consolidated

Restated from spec.md for task-tracking convenience; the Director's actual
answers, when given, should be recorded as amendments to THIS file (or a
successor ADR) rather than silently assumed by whichever task depends on
them:

1. Bloc alignment semantics (static vs. dynamic) — gates T12.
2. INFORMAL edge stamping trigger/threshold philosophy — gates T3, T9
   (T5's `conductivity_informal_mint_threshold` is the numeric knob; this
   ruling is the philosophy behind picking its value).
3. Realization-crisis coupling coefficient + its defines-category home —
   gates T10.
4. Community-scoped vs. corridor-scoped `condition` targeting — gates T4
   (with a documented default-to-Territory-scoped fallback if unanswered)
   and T6.
5. Corridor visibility in the Archive TUI (aggregated connectivity
   coefficient only, or more) — gates Program 26 U6's own future spec
   authoring, not directly any task in this file, but should be answered
   before U6 starts.
