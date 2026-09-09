# TransportSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `TransportSystem` (185 lines, position 9.5, `MATERIAL_BASE`,
default-OFF) decays a corridor-condition mesh, publishes a per-territory demand
signal, aggregates county-pair connectivity, and derives a damped
realization-crisis coupling — five sub-computations over one shared corridor-mesh
substrate. That substrate (H3 hex-pair-keyed `InfrastructureLinkState` lists inside
`DefaultInfrastructureInventory`) lives entirely OUTSIDE `BabylonGraph`/
`GraphProtocol` by deliberate design (spec-108's own D2 ruling: transport edges are
"a SEPARATE vocabulary," never `EdgeType` members) — this is not a missing-BSL-slice
problem, it is a missing-BSL-*primitive* problem. Production dormancy is total and
doubly-caused: `TransportDefines.enabled` defaults `False` (ADR166, a declared
`qa:regression` coverage gap), **and** no code anywhere in the repository populates
`context.persistent_data["corridor_mesh"]` — so the system is a full no-op on every
real run today, independent of the gate. Numerically the reachable call graph is
clean: zero libm transcendentals, zero `Real→Int` demotions, one consistent clamp
shape throughout (unlike Territory's two-clamp inconsistency). Two of its three real
outputs (`transport_demand_signal`, `corridor_connectivity`) are confirmed dead
writes with zero production consumers anywhere in the repo; the third
(`transport_overhang_delta`) is explicitly "not yet wired" by its own docstring.

**Verdict:** **NOT-A-PACK today / BLOCKED on a prerequisite architecture ruling, not
a BSL-language slice** — the corridor-mesh substrate has no graph representation to
port *against* at all, and the system provably never executes on any current
production or canonical-scenario code path.

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/transport.py` | 185 | **The target.** `TransportSystem`, single-phase `step()` (transport.py:118-182) plus its two pure helper functions `compute_overhang_delta` (:56-86) and `_demand_signal` (:89-104). |
| `src/babylon/domain/geography/corridor_mesh.py` | 212 | `CorridorMesh` dataclass + the pure functions `TransportSystem.step()` calls directly: `decay_all_links` (:124-154), `aggregate_connectivity_by_county_pair` (:157-203), `touching_link_ids` (:82-89, via `_touching_links` :66-79). Also defines `apply_uniform_territory_splash` (:92-121) — **not called by `TransportSystem.step()`**, called instead by `ooda/layer3.py` on the SAME mesh object later the same tick (§5). |
| `src/babylon/domain/geography/inventory.py` | 342 | `DefaultInfrastructureInventory` — the mesh's actual storage (`_edge_links: dict[tuple[str,str], list[InfrastructureLinkState]]`, a plain Python dict, **not** `BabylonGraph`/`GraphProtocol`). `decay_all_links` calls its `adjust_link_condition` (:115-155). `get_all_edges`/`get_edge_links` (:157-163, :45-60) are read by both `decay_all_links` and `aggregate_connectivity_by_county_pair`. `degrade_link` (:81-113) and the vertex/junction/nonlocal-edge machinery (:165-342) are **not exercised by TransportSystem's call graph** (grep-confirmed; `degrade_link`'s only caller anywhere is its own unit test). |
| `src/babylon/domain/geography/types.py` | 384 (relevant: 99-152) | `InfrastructureLinkState` — the DTO every link is; `.effective_capacity()` (:143-152) is called once, inside `aggregate_connectivity_by_county_pair`. |
| `src/babylon/config/defines/transport.py` | 161 | `TransportDefines` Pydantic model — the coefficient source, including the master `enabled` gate (:50-58). |
| `src/babylon/config/defines/capital_vol2.py` | 294 (relevant: 240-optional) | `CapitalVolumeIIDefines.transport_overhang_damping_coefficient` (:240-251) — the ONE coefficient `TransportSystem` reads from a defines module other than its own, by explicit Director ruling (ADR165 item 3: "commodity_overhang is a Vol II quantity"). |
| `src/babylon/data/defines.yaml` | (transport block: 1136-1146; capital_vol2 coefficient: 1045) | Player-editable coefficient values. |
| `src/babylon/models/enums/topology.py` | 253 (relevant: 177-221) | `InfrastructureType` (8 members, :177-201), `FlowCategory` (5 members, :204-221) — both closed vocabularies distinct from the main `NodeType`/`EdgeType` enums (:12-127) — **no `EdgeType` member exists for a transport/corridor/infrastructure edge** (grep-confirmed; see §5's D2 citation). |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase` — `TransportSystem` subclasses it but calls **none** of its helpers (`_write_clamped`/`_publish`/`_get_persistent_data`); it reads `context.persistent_data` as a raw dict and calls `graph.get_node`/`graph.update_node` directly. |
| `src/babylon/kernel/tick_partition.py` | — | `TickPartition.MATERIAL_BASE` — `TransportSystem.partition` value. |
| `src/babylon/kernel/graph_protocol.py` | 494 | `GraphProtocol.get_node`/`.update_node` signatures (:77, :88). |
| `src/babylon/topology/graph.py` | (relevant: 651-670) | Concrete `BabylonGraph.get_node`/`.update_node` — the same "plain dict merge, no type coercion or quantization" shape the Territory inventory already documented (:660-670). |
| `src/babylon/engine/context.py` | 113 | `TickContext.persistent_data: dict[str, Any]` (:47) — the one non-graph input/output channel `TransportSystem` uses for `corridor_mesh`/`corridor_connectivity`/`transport_overhang_delta`. |
| `src/babylon/engine/simulation_engine.py` | 611 (relevant: 328-364) | `_SYSTEM_CLASSES` — confirms tick position 9.5, between `ImperialRentSystem` (9.0, `economic.py:37`) and `DispossessionEventSystem` (10.0, `dispossession_events.py:36`). |
| `src/babylon/ooda/layer3.py` | 222 | `process_layer3` (:24-68) and `_propagate_infrastructure` (:162-214) — **downstream consumer** of the SAME corridor mesh `TransportSystem` decays this tick, via `apply_uniform_territory_splash` (§5). Not part of `TransportSystem` itself but load-bearing for the cross-system-channel analysis. |
| `src/babylon/engine/systems/ooda.py` | 478 (relevant: 199-213) | `OODASystem.step()` — reads `context.persistent_data.get("corridor_mesh")` (:206) and threads it into `process_layer3`. |
| `src/babylon/persistence/hex_hydrator.py` | 931 (relevant: 292-310) | `read_hex_county_adjunction` — the seam `corridor_mesh.py`'s own docstring names as the future `territory_hexes` supplier; its docstring states outright "**no production code stamps a hex node**" (:301-302) — confirms hex cells are not even live graph nodes in a real run today. |
| `src/babylon/sentinels/vocabulary/registry.py` | 754 (relevant: 201-246) | `EXTRA_STAMPABLE_ATTRIBUTES[NodeType.TERRITORY]` — `transport_demand_signal` is listed (:243) as a confirmed-live production graph-only write (correctly exempted, unlike the "active"/"s_bio"/"s_class" decorative trio next to it). |
| `src/babylon/models/world_state.py` | 1161 (relevant: 147-153) | `TERRITORY_EXCLUDED_FIELDS` — `transport_demand_signal` is dropped on every `WorldState.from_graph()` round-trip (:152) — it is NOT a declared `Territory` Pydantic field. |
| `tools/regression_scenarios.py` | 2925 (relevant: 2727-2735) | `COVERAGE_GAPS_DATA` — the declared, named `qa:regression` coverage gap for `TransportSystem`, citing ADR166 and default-OFF explicitly. |
| `ai/decisions/ADR166_p26_u5e_transport_substrate_slice1.yaml` | 53 | The governing ADR: position/gate ruling, scope (slice 1 vs. the routing-solver/BUILD_INFRASTRUCTURE-registration follow-up), and its own "negative consequences" list. |
| `specs/108-transport-substrate/research.md` | (relevant: §10, ~280-306) | **D2 ruling**: "transport edges are correctly a SEPARATE vocabulary (`InfrastructureType`), not additions to [`EdgeType`]" — the authoritative source for §5's central architectural finding. |

**Not exercised by `TransportSystem.step()` at all:** `domain/geography/capacity.py`
(`DefaultEdgeCapacityCalculator` — a different, unused capacity-aggregation path),
`inventory.py`'s vertex/junction/nonlocal-edge machinery, `inventory.degrade_link`,
`corridor_mesh.apply_uniform_territory_splash` (called by `ooda/layer3.py` instead —
see §5), `engine/actions/build.py::resolve_build` (exists, tested, deliberately
**not registered** as a 10th verb per ADR166).

**Reference BSL packs/docs read for format:** `docs/reference/bsl-language.rst`
(EdgeRef/`field-of` chapter C1, §2.6-2.7 query-lane chapters, the declared-intrinsics
table) and the Territory Phase-1 inventory
(`reports/territory-port-phase1-inventory-2026-08-11.md`) as the house template.

## 2. COMPUTATION CATALOG (execution order, `transport.py:118-182`)

### 0 — Master enable gate (`transport.py:134-136`)
- **(a)** If the transport substrate is disabled, do nothing at all.
- **(b)** `if not defines.enabled: return` (:135-136).
- **(c) Reads:** `TransportDefines.enabled`.
- **(d) Writes:** none.
- **(e) Defines:** `transport.enabled` (`bool`, default `False` — defines.yaml:1137).
- **(f) Events:** none.

### 0.5 — Mesh-presence gate (`transport.py:138-140`)
- **(a)** If no campaign has composed a corridor mesh yet, do nothing — an honest
  absence (Constitution III.11), never a fabricated signal (module docstring,
  transport.py:26-31).
- **(b)** `mesh = context.persistent_data.get("corridor_mesh"); if mesh is None: return`.
- **(c) Reads:** `context.persistent_data["corridor_mesh"]` (`CorridorMesh | None`).
- **(d) Writes:** none.
- **(e) Defines:** none.
- **(f) Events:** none.
- **Load-bearing fact (§5):** on every current production and canonical-scenario
  code path, this key is **never populated**, so this gate fires on every real run
  today even when `enabled=True` — see §5.

### 1 — Corridor decay (`decay_all_links`, `corridor_mesh.py:124-154`, called at `transport.py:142-146`)
- **(a)** Every corridor link's `condition` degrades a little every tick from base
  neglect plus extra decay proportional to how hard it's being used
  (`conductivity`).
- **(b)** For every link on every mesh edge: `delta = -(decay_rate_per_tick +
  flux_coefficient * link.conductivity)` (corridor_mesh.py:151); applied via
  `inventory.adjust_link_condition(link.link_id, delta)` (:152), whose body is
  `new_condition = max(0.0, min(1.0, link.condition + condition_delta))`
  (`inventory.py:149`).
- **(c) Reads:** every `InfrastructureLinkState.condition`, `.conductivity` on every
  edge in `mesh.inventory` (iterated `sorted(mesh.inventory.get_all_edges())`,
  corridor_mesh.py:149 — deterministic order, Constitution III.7).
- **(d) Writes:** every `InfrastructureLinkState.condition` in the mesh (mutated
  in place via `model_copy`, `inventory.py:150`).
- **(e) Defines:** `transport.condition_decay_rate_per_tick` (`float`, `(0,1)`
  exclusive-exclusive, default `0.01`), `transport.condition_decay_flux_coefficient`
  (`float`, `(0,∞)` i.e. `gt=0.0` with **no upper bound**, default `0.02`) —
  defines.yaml:1138-1139.
- **(f) Events:** none.

### 2 — Connectivity aggregation + publish (`aggregate_connectivity_by_county_pair`, `corridor_mesh.py:157-203`, called at `transport.py:148-149`)
- **(a)** For every corridor edge that crosses a territory boundary, sum its
  effective FREIGHT capacity into a running per-county-pair connectivity total
  (a proxy for "how well-connected are these two counties").
- **(b)** For every mesh edge with links on both a source and target territory:
  `edge_capacity = sum(link.effective_capacity(FlowCategory.FREIGHT) for link in
  ...)` (:193-194), where `effective_capacity = capacity.get(category, 0.0) *
  condition` (`types.py:152`); same-territory edges are skipped (`if t_a == t_b:
  continue`, :199-200); accumulated as `totals[pair] = totals.get(pair, 0.0) +
  edge_capacity` (:202) into a canonically-sorted `(t_a, t_b)` key (:201).
- **(c) Reads:** `InfrastructureLinkState.capacity["freight"]`, `.condition` on
  every inter-territory mesh edge; `mesh.territory_hexes` (the hex→territory
  reverse index, `_reverse_hex_index`, :57-63).
- **(d) Writes:** `context.persistent_data["corridor_connectivity"]` (a
  `dict[tuple[str,str], float]` — **not** a graph node/edge attribute at all).
- **(e) Defines:** none directly (uses `FlowCategory.FREIGHT`, a hardcoded literal
  enum member, not a define).
- **(f) Events:** none.

### 3 — Per-territory demand-signal pass (`transport.py:151-168`)
- **(a)** For every territory touched by at least one corridor link, compute a
  synthetic "needs attention" signal from how busy and how degraded its links are,
  and stamp it on the territory's graph node — but skip any territory id the mesh
  names that isn't (yet) an actual node in the main graph.
- **(b)** `links_by_id` built from every mesh edge (:151-155, no arithmetic). Per
  territory (`sorted(mesh.territory_hexes)`, :158): `overhang_ratios.append(1.0 -
  sum(link.condition for link in links) / len(links))` (:163 — used by
  computation 4, not this one's own output); if the territory id has no matching
  graph node, `continue` (:165-166, honest skip, never raises — grep-confirmed by
  `test_territory_not_present_in_the_graph_is_skipped_not_raised`); else `signal =
  _demand_signal(links, defines.demand_signal_threshold)` (:167) where
  `_demand_signal` (transport.py:89-104) is `max(0.0, avg_conductivity -
  threshold) + (1.0 - avg_condition)` over the territory's touching links, then
  `graph.update_node(territory_id, transport_demand_signal=signal)` (:168).
- **(c) Reads:** `InfrastructureLinkState.condition`, `.conductivity` on every link
  touching the territory (`touching_link_ids`, corridor_mesh.py:82-89); `graph.get_node(territory_id)`
  (existence check only, not an attribute read).
- **(d) Writes:** `TERRITORY.transport_demand_signal` — graph-only (not a declared
  `Territory` Pydantic field; `EXTRA_STAMPABLE_ATTRIBUTES[NodeType.TERRITORY]`,
  `registry.py:243`; dropped from every `WorldState.from_graph()` round-trip,
  `TERRITORY_EXCLUDED_FIELDS`, `world_state.py:152`).
- **(e) Defines:** `transport.demand_signal_threshold` (`float`, `[0,∞)` i.e.
  `ge=0.0` with **no upper bound**, default `0.3` — defines.yaml:1144).
- **(f) Events:** none.
- **Cross-system note (§5):** the docstring for `demand_signal_threshold`
  (config/defines/transport.py:122-134) and ADR166's own decision text both claim
  this signal "feeds the sovereign's OODA budget evaluation" — **grep-confirmed
  false on current dev**: no code anywhere reads `transport_demand_signal`. Recorded
  verbatim per port-as-is law, not softened.

### 4 — National overhang aggregate + damped coupling (`transport.py:170-178`)
- **(a)** Average, across every territory this tick touched, how degraded its
  corridor links are; damp that national ratio by a coefficient and stash it for a
  (not-yet-wired) downstream realization-crisis consumer.
- **(b)** `national_ratio = sum(overhang_ratios) / len(overhang_ratios) if
  overhang_ratios else 0.0` (:174); `compute_overhang_delta(stranded_value_ratio,
  damping_coefficient)` (:56-86) is `clamped_ratio = max(0.0, min(1.0,
  stranded_value_ratio)); return clamped_ratio * damping_coefficient` (:85-86).
- **(c) Reads:** `overhang_ratios` (accumulated in computation 3, per-territory `1.0
  - avg_condition`); `CapitalVolumeIIDefines.transport_overhang_damping_coefficient`.
- **(d) Writes:** `context.persistent_data["transport_overhang_delta"]` (a lone
  `float`, bounded `[0.0, damping_coefficient]` ⊆ `[0.0, 1.0]` by construction —
  **not** a graph attribute).
- **(e) Defines:** `capital_vol2.transport_overhang_damping_coefficient` (`float`,
  `(0,1]` i.e. `gt=0.0, le=1.0`, default `0.3` — defines.yaml:1045; homed in
  `CapitalVolumeIIDefines`, **not** `TransportDefines`, by explicit Director ruling,
  ADR165 item 3).
- **(f) Events:** none.
- **Cross-system note (§5):** `compute_overhang_delta`'s own docstring
  (transport.py:64-73) states outright it is "**Not yet wired**" into
  `assess_circulation_crisis`'s call site — the one honesty caveat this system's
  own authors already flagged, matching the grep-confirmed zero-consumer finding.

### 5 — Mesh write-back (`transport.py:180-182`)
- **(a)** Keep the (already-mutated) mesh available in `persistent_data` for
  `OODASystem`'s layer-3 consequence pass later the same tick.
- **(b)** `context.persistent_data["corridor_mesh"] = mesh` (:182) — a
  self-reassignment; the mesh object was already mutated in place by computation 1
  (`decay_all_links` calls `adjust_link_condition`, which mutates
  `mesh.inventory`'s internal list). No new arithmetic.
- **(c) Reads:** the local `mesh` reference.
- **(d) Writes:** `context.persistent_data["corridor_mesh"]`.
- **(e) Defines:** none.
- **(f) Events:** none.

**Events emitted by the whole system: zero.** Grep-confirmed: no `EventType`,
`emit`, `.publish(`, or `_publish` reference anywhere in `transport.py`. Same
silence as `TerritorySystem`.

## 3. TYPE INVENTORY

Runtime storage note (identical shape to the Territory inventory's finding, cited
here because it governs the one graph write this system makes):
`BabylonGraph.update_node` (`topology/graph.py:660-670`) is a plain dict merge —
**no type coercion or quantization** — so `transport_demand_signal` lands as a raw
Python `float` with no grid snap.

The second, larger fact this system adds beyond Territory's shape: **most of its
state is not graph state at all.** `CorridorMesh` (a `dataclass`, not a Pydantic
`BaseModel`, `corridor_mesh.py:36-54`) and `DefaultInfrastructureInventory` (a plain
class with a private `dict`, `inventory.py:22-37`) are ordinary Python objects
living in `context.persistent_data`, entirely outside `GraphProtocol`.

| Attribute | Node/location | Python model type | Domain | Category |
|---|---|---|---|---|
| `TransportDefines.enabled` | defines | `bool` | `{T,F}` | master-gate boolean |
| `TransportDefines.condition_decay_rate_per_tick` | defines | `float` | `(0.0, 1.0)` | coefficient |
| `TransportDefines.condition_decay_flux_coefficient` | defines | `float` | `(0.0, ∞)` | **unbounded-above coefficient** |
| `TransportDefines.demand_signal_threshold` | defines | `float` | `[0.0, ∞)` | **unbounded-above coefficient** |
| `CapitalVolumeIIDefines.transport_overhang_damping_coefficient` | defines (different module) | `float` | `(0.0, 1.0]` | coefficient |
| `TERRITORY.transport_demand_signal` | TERRITORY (graph write) | plain `float` (via `**attributes`) | `[0.0, ∞)` in practice (sum of a `max(0.0,...)` term and a `[0,1]`-bounded term; conductivity is unbounded above so the whole signal is **unbounded above, unproven upper bound**) | graph-only, **not a declared `Territory` field** |
| `CorridorMesh.inventory` | `context.persistent_data["corridor_mesh"]` | `DefaultInfrastructureInventory` (plain class) | — | **not a graph node or edge** |
| `CorridorMesh.territory_hexes` | same | `Mapping[str, frozenset[str]]` | — | not a graph structure |
| `InfrastructureLinkState.link_id` | inside the mesh | `str` | — | identifier |
| `InfrastructureLinkState.infra_type` | inside the mesh | `str` (`InfrastructureType` StrEnum value, 8 members) | closed set | **enum discriminant, unused by Transport's own arithmetic** — carried only |
| `InfrastructureLinkState.capacity` | inside the mesh | `dict[str, float]` | keyed by `FlowCategory` (5 members); **no `ge=`/`le=` Field constraint on the dict's float values at all** | **composite/dict-valued, unbounded, unvalidated real** |
| `InfrastructureLinkState.condition` | inside the mesh | `float` | `[0.0, 1.0]`, default `1.0` | unit-interval |
| `InfrastructureLinkState.conductivity` | inside the mesh | `float` | `[0.0, ∞)` i.e. `ge=0.0`, **no upper bound**, default `0.0` | **unbounded-above real** |
| `InfrastructureLinkState.owner_org_id` | inside the mesh | `str \| None` | — | unused by Transport's own math |
| `InfrastructureLinkState.ne_source_id` | inside the mesh | `str \| None` | — | unused |
| `context.persistent_data["corridor_connectivity"]` | context | `dict[tuple[str,str], float]` | pair-keyed, unbounded (sum of unvalidated `capacity` values) | **composite, not a node/edge attribute** |
| `context.persistent_data["transport_overhang_delta"]` | context | `float` | `[0.0, 1.0]` by construction (clamp then multiply by a `(0,1]` coefficient) | scalar, cross-tick |

**Currency/enum flags: not applicable here** — this system has no `Currency`-typed
field and no per-node stored enum field of its own (`infra_type` lives on the
mesh's link DTOs, which are not graph attributes at all — the enum-storage gap
Territory's inventory named for `profile`/`territory_type` does not even arise for
Transport, because the whole DTO carrying `infra_type` is outside the graph).

**The load-bearing type-inventory finding is architectural, not per-field**: the
overwhelming majority of this system's state (`InfrastructureLinkState`,
`DefaultInfrastructureInventory`, `CorridorMesh`) is never expressed as a
`NodeType`/`EdgeType` member at all. `specs/108-transport-substrate/research.md`
§10 (D2 ruling) states this is **deliberate**: "`EdgeType` … No transport-specific
member exists here, confirming D2: transport edges are correctly a SEPARATE
vocabulary (`InfrastructureType`), not additions to this enum." A BSL port
therefore has no existing graph shape to target — see §5 and §6.

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (`float`), no libm transcendentals anywhere across
`transport.py`, `corridor_mesh.py`, `inventory.py`, `types.py` — grep-confirmed
zero `exp`/`log`/`pow`/`sigmoid`/`math.` hits. **No `int()`/`round()`/floor casts
anywhere in this system's call graph either** — zero `Real→Int` demotions, a
cleaner numeric profile than Territory's (which had two). Shapes, in execution
order:

1. **Additive decay delta:** `delta = -(decay_rate_per_tick + flux_coefficient *
   link.conductivity)` (`corridor_mesh.py:151`) — one multiply, one add, one
   negate.
2. **Two-sided clamp (shape A):** `new_condition = max(0.0, min(1.0,
   link.condition + condition_delta))` (`inventory.py:149`, `adjust_link_condition`
   — the **only** clamp `TransportSystem`'s own reachable call graph exercises).
   Bare literals `0.0`/`1.0` — the same "no bare non-integer literal" BSL-parser
   problem Territory's inventory flagged, needing a `c`-suffixed const or the
   Real-zero-promotion idiom.
3. **Effective-capacity multiply:** `capacity.get(category, 0.0) * condition`
   (`types.py:152`) — dict lookup with a bare-`0.0` default, then multiply.
4. **Running-total accumulation, order-independent by construction:**
   `totals[pair] = totals.get(pair, 0.0) + edge_capacity` (`corridor_mesh.py:202`)
   — a collect-into-dict pattern, same favorable structural shape Territory's
   spillover accumulation had (§4 item 6 there): deterministic because the outer
   loop is over `sorted(mesh.inventory.get_all_edges())`.
5. **Two guarded averages:** `avg_condition = sum(...)/len(links)`,
   `avg_conductivity = sum(...)/len(links)` (`transport.py:102-103`, inside
   `_demand_signal`) — both guarded by `if not links: return 0.0` (:100-101), so
   division-by-zero is statically unreachable.
6. **Demand-signal combination:** `max(0.0, avg_conductivity - threshold) + (1.0 -
   avg_condition)` (`transport.py:104`) — subtract, `max(0.0, …)`, subtract-from-`1.0`,
   add. Two bare `0.0`/`1.0` literals.
7. **Per-territory overhang-ratio average:** `1.0 - sum(link.condition for link in
   links) / len(links)` (`transport.py:163`) — guarded by the `if not links:
   continue` two lines above (:160-161).
8. **National average with a ternary default:** `sum(overhang_ratios) /
   len(overhang_ratios) if overhang_ratios else 0.0` (`transport.py:174`) —
   guarded division, bare `0.0` default.
9. **Two-sided clamp (shape A again) + multiply:** `clamped_ratio = max(0.0,
   min(1.0, stranded_value_ratio)); return clamped_ratio * damping_coefficient`
   (`transport.py:85-86`, `compute_overhang_delta`) — the **same** nested
   `max(lo, min(hi, x))` shape as item 2 above.

**Clamp-consistency finding (favorable contrast to Territory):** every clamp this
system's reachable call graph exercises (`inventory.py:149`, `transport.py:85`) is
the identical `max(lo, min(hi, x))` shape — unlike Territory's `_write_clamped`
vs. hand-written `min(1.0, …)` inconsistency. One clamp shape exists elsewhere in
the SAME `inventory.py` file but is **not reachable** from `TransportSystem`:
`degrade_link`'s lower-only `max(0.0, link.condition - condition_delta)`
(`inventory.py:107`) — recorded for honesty since it lives beside the reachable
code, but it is dead code from this system's perspective (§1).

**No libm hazard anywhere in this system.** This matches Territory, not
Metabolism.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 9.5** (`transport.py:115`), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-364`): `... → ImperialRentSystem (9.0) →
  TransportSystem (9.5) → DispossessionEventSystem (10.0) → ...` (positions
  confirmed directly: `economic.py:37`, `dispossession_events.py:36`).
- **Reads from a same-tick prior system: none.** `TransportSystem.step()` reads
  zero graph-node *attribute values* written by any earlier system this tick — its
  only graph interaction before its own write is `graph.get_node(territory_id) is
  None` (:165), an existence check, not an attribute read. Every other input is
  either a `defines` coefficient or `context.persistent_data["corridor_mesh"]`.
- **Writes consumed later this tick / downstream ticks — three of four channels
  are dead:**
  - `TERRITORY.transport_demand_signal` — **read by no other system anywhere in
    the repository** (exhaustive grep across `src/`, `tests/`, `web/`, `rust/`
    outside `transport.py` itself, its own tests, `world_state.py`'s exclusion
    entry, and the vocabulary registry's exemption entry). This directly
    contradicts the field's own docstring claim ("feeding the sovereign's OODA
    budget evaluation," `config/defines/transport.py:126-129`) and ADR166's
    decision text ("consumed by the sovereign's budget OODA evaluation"). `rg -n
    'demand' src/babylon/ooda/*.py` finds nothing. Recorded verbatim, port-as-is —
    this is a genuine defect/gap in the frozen system, not something this
    inventory proposes to fix.
  - `context.persistent_data["corridor_connectivity"]` — **read by no production
    code anywhere** (only test assertions reference the key).
  - `context.persistent_data["transport_overhang_delta"]` — **read by no
    production code anywhere**; the system's own docstring (`transport.py:64-73`)
    already discloses this ("Not yet wired").
  - `context.persistent_data["corridor_mesh"]` (mutated in place, then
    self-reassigned) — **the one real downstream channel**: `OODASystem.step()`
    (@14.0, Action phase) reads it (`engine/systems/ooda.py:206`,
    `context.persistent_data.get("corridor_mesh")`) and threads it into
    `process_layer3` (`ooda/layer3.py:207-213`), whose `_propagate_infrastructure`
    calls `apply_uniform_territory_splash` (`ooda/layer3.py:208-214`) — the SAME
    `corridor_mesh.py` module `TransportSystem` uses, for BUILD/ATTACK_INFRASTRUCTURE
    uniform splash resolution. This channel only carries real content once a
    campaign-level composer has ALREADY populated the key before tick 9.5 runs —
    which, per the next finding, no production code path does.
- **Context/service usage with no BSL equivalent — three distinct gaps, not one:**
  1. `context.persistent_data["corridor_mesh"]`: a whole `CorridorMesh` object
     (wrapping `DefaultInfrastructureInventory`'s own internal
     `dict[tuple[str,str], list[InfrastructureLinkState]]`, keyed by H3 hex pairs,
     each key holding a LIST of composite-attribute link records) stored and read
     back as an opaque Python object. This is categorically different from
     Territory's `TickContext.displacement_mode` (a single enum override): it is
     this system's *entire* operative state, and it is not a graph node or edge at
     all — see §3's architectural finding and the D2 citation below.
  2. `context.persistent_data["corridor_connectivity"]`: a
     `dict[tuple[str,str], float]` keyed by sorted TERRITORY-ID PAIRS — not a
     per-node or per-edge attribute, so even a hypothetically fully-landed
     edge-attribute lane (Slice 2/3) would give it no home; it needs an entirely
     new "publish an aggregate map to context" primitive that nothing in the
     current or planned BSL surface describes.
  3. `context.persistent_data["transport_overhang_delta"]`: a lone scalar
     `float` — the least severe of the three shapes (a cross-tick scalar
     `:const`-like publish channel is at least conceivable), but no such channel
     is named anywhere in the current BSL surface either.
  4. `services.defines.transport` / `services.defines.capital_vol2.
     transport_overhang_damping_coefficient` — ordinary `GameDefines` coefficient
     reads, the same shape every landed pack already uses via `defconst`. **Not**
     a gap.
- **The central architectural fact (D2 ruling, `specs/108-transport-substrate/
  research.md` §10, ~lines 297-300):** *"`EdgeType` (78-127): the MAIN graph's
  relation vocabulary … No transport-specific member exists here, confirming D2:
  transport edges are correctly a SEPARATE vocabulary (`InfrastructureType`), not
  additions to this enum."* This was a **deliberate, ruled** decision in the
  frozen Python engine itself — not an oversight a port could route around with a
  content-modeling D-record. Bringing the corridor mesh into BSL's `GraphSubstrate`
  paradigm at all means either (a) inventing a new BSL concept for multi-link,
  composite-attribute, hex-pair-keyed edge collections disjoint from the closed
  `NodeType`/`EdgeType` vocabulary — squarely "invent primitives without a
  constitutional amendment," CLAUDE.md's MUST NOT list — or (b) diverging from the
  frozen engine's own D2 ruling by re-modeling corridor links as ordinary
  `BabylonGraph` edges, which is itself a port-as-is violation (transcribe the
  frozen shape, don't redesign it) unless separately Director-ruled.
- **DORMANCY on canonical scenarios: total, and doubly-caused.**
  `tools/regression_scenarios.py:2727-2735` declares `TransportSystem` a coverage
  gap by name: *"P26 U5e slice 1 lands DEFAULT-OFF (`TransportDefines.enabled=False`,
  ADR166): no canonical scenario enables the corridor mesh, so the demand-signal/
  decay/connectivity pass never mutates state on any gated run — byte-identical by
  design."* Beyond that declared gap, an exhaustive repo-wide grep for
  `persistent_data\["corridor_mesh"\]\s*=` finds **exactly one write site in the
  entire codebase: `transport.py:182`, TransportSystem's own write-back.** No
  loader, composer, session, or persistence module populates the key *initially* —
  the module docstring (`transport.py:23-31`) and ADR166's own "negative
  consequences" list both confirm this is chartered, not-yet-built future work.
  `persistence/hex_hydrator.py:301-302`'s own docstring goes further: **"no
  production code stamps a hex node"** — the H3 hexes this whole substrate keys
  off of are not even live `BabylonGraph` nodes in a real run today. So
  `TransportSystem` is dormant **twice over**: (1) `enabled` defaults `False`
  (a flippable defines value), AND (2) even flipped, `if mesh is None: return`
  fires unconditionally until a wholly separate, unbuilt unit exists. A port's
  conformance fixtures would have to be hand-built from primitives with no
  precedent anywhere in production code to imitate, not merely "harvested from
  the canonical scenarios" (the phrase Territory's inventory used for its milder
  dormancy finding) — there is no live code path to imitate at all.

## 6. BLOCKER ASSESSMENT

**Root-cause framing.** Every computation below except the master gate is blocked
by the SAME architectural fact (§5): the corridor-mesh substrate has no BSL/graph
representation, by the frozen engine's own D2 design ruling — not by an
as-yet-unbuilt BSL slice with a known remediation path. Where a row would ALSO hit
a *second*, independent gap (e.g., the connectivity dict's own representation
problem, or Slice 2/3's edge-attribute-read gap), it is named in addition.

| Computation | Verdict | Detail |
|---|---|---|
| Master enable gate (`transport.py:134-136`) | **PORTABLE NOW** | A single `bool` `:const` check — trivial, precedented by every other landed pack's `defconst` usage and by plain `bool` comparisons already used for latch fields (e.g. Territory's `under_eviction`). No blocking surface on its own. |
| Mesh-presence gate + the entire corridor-mesh substrate (`transport.py:138-140` and everything computations 1-5 read from `mesh`) | **BLOCKED — no BSL substrate primitive for the out-of-graph corridor mesh** | Not one of Slices 2-4 (edge-attribute reads, hyperedge/metric lane, attribute-storage widening) — those all presuppose the thing being read is already a `NodeType`/`EdgeType` graph element. The corridor mesh is, by the frozen engine's own D2 ruling, deliberately NOT a graph element at all. Landing this requires a new primitive (amendment-gated) or a scope-reserved architecture decision about whether/how to fold H3 hex-pair infrastructure links into `GraphSubstrate`. Not a task a delegated port train can D-record its way past. |
| `decay_all_links` (`corridor_mesh.py:124-154`, via `inventory.py:115-155`) | **BLOCKED — same substrate gap** | If the substrate existed, the arithmetic itself (item 1 of §4: one multiply, one add, one two-sided clamp) would be trivially portable — no libm, no bare-literal blocker beyond the standard `0.0`/`1.0` const-suffix workaround, no `Real→Int` demotion. The blocker is entirely representational, not computational. |
| `aggregate_connectivity_by_county_pair` (`corridor_mesh.py:157-203`) | **BLOCKED — substrate gap, PLUS a second, independent representation gap** | Even with the substrate available, the output (`dict[(county,county), float]`) is not a per-node or per-edge attribute — it needs a "publish an aggregate pairwise map" primitive that does not exist in any named BSL slice, present or planned (§5). |
| Per-territory demand-signal computation (READ side: `touching_link_ids` + `_demand_signal`, `corridor_mesh.py:82-89`, `transport.py:89-104`) | **BLOCKED — substrate gap, PLUS Slice 2/3 (edge-attribute reads)** | Even granting the substrate, reading `condition`/`conductivity` per link is exactly the shape Slice 2/3's `EdgeRef`/`field-of` chapter targets and which "CURRENT BSL surface" states is NOT BUILT. Two stacked gaps, not one. |
| Per-territory demand-signal computation (WRITE side: `graph.update_node(territory_id, transport_demand_signal=signal)`, `transport.py:168`) | **PORTABLE NOW, in isolation** | `update-node` against a real `NodeRef` for an already-typed `TERRITORY` node is exactly the Slice-1 `update-node` shape that landed (ADR197). Unreachable in practice because its input is blocked upstream (previous row) — and per §5, its own output is a confirmed dead write with zero consumers, so porting it would spend port effort reproducing a value nothing reads. |
| National overhang-ratio average (`transport.py:170-174`) | **BLOCKED — same substrate gap** (its input, `overhang_ratios`, is populated inside the blocked per-territory pass) | The averaging arithmetic itself (a guarded `sum/len` with a ternary `0.0` default) is trivial once/if an input were available. |
| `compute_overhang_delta` pure kernel (`transport.py:56-86`), GIVEN a Real `national_ratio` | **PORTABLE WITH D-RECORD** | In isolation: `max(0.0, min(1.0, x)) * damping_coefficient` — the same clamp+multiply shape landed packs already use, `damping_coefficient` is a plain `(0,1]`-domain coefficient (trivial `c`-suffixed `defconst`). D-record needed only for the standard bare-`0.0`/`1.0`-literal workaround. Unreachable today because its input is blocked upstream, and its OWN output (`context.persistent_data["transport_overhang_delta"]`) has no BSL cross-tick scalar-publish channel named either (§5 gap 3) — so even this "portable" kernel has nowhere to write its result. |
| Mesh write-back (`transport.py:182`) | **BLOCKED — trivially, by the same substrate gap** | There is no mesh object to write back if it cannot be represented or read in the first place; the Python-level self-reassignment carries no independent arithmetic content to port. |

**Net verdict for the whole system: NOT-A-PACK today.** Unlike Territory's
"DEFER — query-evaluation train first" (a single named, already-in-flight train
that demonstrably unblocked most of that system's rows), TransportSystem has no
comparable single unblock: its root gap is a substrate/primitive question that sits
*prior to* BSL-language-feature work, and its production dormancy (§5) means there
is no urgency pressure from the canonical estate either way — the
`qa:regression` byte-gate structurally cannot see any of this system's outputs even
if it ran (see §7's finding on `graph_content_hash`'s field-set exclusions).

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_transport.py` | 180 | **Primary conformance-oracle candidate for `TransportSystem.step()` itself.** Master gate (disabled / enabled-without-mesh), decay + connectivity publication, demand-signal publication (present/absent/territory-not-in-graph), `compute_overhang_delta`'s own unit tests plus its `step()`-level storage test. |
| `tests/unit/engine/laws/test_law_transport.py` | 227 | **Property-based (Hypothesis) invariant contracts** — L1 (decay monotonic-non-increasing and clamped), L2 (demand signal never negative), L3 (disabled-or-meshless is a full no-op, no key written), L4 (connectivity aggregation has no self-pairs and canonically-sorted keys). Strong behavioral-contract candidate, same genre as Territory's `test_law_territory_system.py`. |
| `tests/unit/domain/geography/test_corridor_mesh.py` | 175 | Pure-function unit tests for `corridor_mesh.py`'s functions `TransportSystem` calls directly (`touching_link_ids`, `decay_all_links`, `aggregate_connectivity_by_county_pair`) plus `apply_uniform_territory_splash` (used by `ooda/layer3.py`, not `TransportSystem` itself, but the same shared module). Strong conformance-oracle candidate for the substrate module. |
| `tests/unit/infrastructure/test_inventory.py` | 309 | Shared-dependency tests for `DefaultInfrastructureInventory`. Lines ~62-206 (`conductivity` validation, `degrade_link`, `adjust_link_condition` — including its two clamp-direction tests) are relevant to Transport's reachable call graph; lines ~207-309 (vertices, junctions, nonlocal edges) are Feature-036 scope Transport never touches. |
| `tests/unit/ooda/test_layer3.py` | 287 | Tests `process_layer3`'s consumption of the mesh `TransportSystem` decays (the uniform-splash cross-system channel, §5) — only partially about `TransportSystem`; the splash logic itself is `OODASystem`'s own code, not the target system's. |
| `tests/unit/config/test_transport_defines.py` | 70 | `TransportDefines` Pydantic model validation (field bounds/defaults) — schema-level, not tick-behavior. |
| `tests/unit/sentinels/test_gate_coverage.py` | 140 | General estate-wide gate-coverage sentinel; one assertion (`test_real_engine_system_count`, :24-34) pins the 34-system count and cites ADR166 + the declared coverage gap by name — infra-level pin, not `TransportSystem` behavior. |
| `tests/unit/engine/test_system_order.py` | 300 | General system-ordering sentinel; pins `TransportSystem`'s name/position (9.5) among all 34 systems (:72, :196, :241) — infra-level ordering pin, not behavioral. |
| `tests/unit/test_public_import_surface.py` | 309 | General public-API import-surface pin; `TransportSystem`/`TransportDefines` appear as members of a broader tracked list — infra-level, not behavioral. |

**`qa:regression` byte-gate coverage: structurally zero, not merely dormant.**
`tools/regression_test.py::graph_content_hash` (:924-964) hashes the
`WorldState`→graph projection — "nodes/edges/actions" only, explicitly excluding
graph *metadata* (`:940-943`: "the spec's field set is nodes/edges/actions").
`context.persistent_data` (`corridor_connectivity`, `transport_overhang_delta`,
`corridor_mesh`) is not part of that projection at all — the hash gate cannot see
it even in principle. `transport_demand_signal`, the one output that DOES touch a
graph node, is separately dropped by `TERRITORY_EXCLUDED_FIELDS`
(`world_state.py:152`) before `WorldState.from_graph()` ever hands data to
`graph_content_hash`. So — independent of the default-OFF gate and independent of
the mesh never being populated (§5) — even a hypothetical future canonical
scenario that flipped `enabled=True` AND wired a mesh composer would still leave
every one of `TransportSystem`'s real outputs **outside the byte-identical
regression gate's reach**, by construction of the hash's own field set. A port's
conformance fixtures need to be hand-built `.bscn` scenarios exercising
`corridor_mesh.py`'s pure functions directly (mirroring
`tests/unit/domain/geography/test_corridor_mesh.py`'s own fixture shapes) — there
is no canonical-scenario byte-gate to lean on at all, today or under any presently
planned remediation.

---

## Adjudication (2026-08-12)

Adjudicated against the dev tree at `9324482f`. Two corrections, four confirmations. This is the
one verdict in the batch that turns on a *frozen-engine architecture ruling* rather than a BSL
slice, and that framing survives adjudication intact.

1. **CONFIRMATION — the double dormancy is real and provable, exactly as claimed.**
   `TransportDefines.enabled: bool = Field(default=False, …)`
   (`src/babylon/config/defines/transport.py:50-51`; `defines.yaml:1137`). And the second cause
   holds under a clean grep: the ONLY `persistent_data["corridor_mesh"] =` assignment anywhere in
   `src/` is `transport.py:182` — the system's own write-back. Every other assignment is a test
   fixture (`tests/unit/engine/systems/test_transport.py:55,96,117,132,148,175`;
   `tests/unit/engine/laws/test_law_transport.py:123,157,183`). The only production reader is
   `engine/systems/ooda.py:206`. No loader, composer, session or persistence module seeds the key.
   The declared coverage-gap row is verbatim at `tools/regression_scenarios.py:2728-2735`, citing
   ADR166 and default-OFF by name.

2. **CONFIRMATION — `transport_demand_signal` is a confirmed dead write, and the docstring claim
   is confirmed false.** A repo-wide grep finds the sole writer at `transport.py:168` and, beyond
   it, only: the defines description asserting it feeds "the sovereign's OODA budget evaluation"
   (`defines.yaml:1144`, `config/defines/transport.py:127`), the `TERRITORY_EXCLUDED_FIELDS` drop
   (`models/world_state.py:152`), the vocabulary exemption (`sentinels/vocabulary/registry.py:243`)
   and two test assertions. Zero readers. Recording it verbatim under port-as-is law rather than
   softening it was the right call.

3. **CONFIRMATION — the D2 ruling is where the inventory says it is and says what it says.**
   `specs/108-transport-substrate/research.md:290` (§10 header, "NodeType / EdgeType /
   InfrastructureType vocabulary audit (D2/D3/D6)") and `:300` ("No transport-specific member
   exists here, confirming D2: transport edges …"). The inference the inventory draws from it —
   that a BSL port has no existing graph shape to target, and that inventing one is either
   amendment territory or a port-as-is violation — is sound and is the load-bearing half of the
   verdict.

4. **CORRECTION — §6's master-gate row cites a precedent that does not exist, and the correct
   evidence is a `defconst`, not a field.** The row reads: *"precedented by every other landed
   pack's `defconst` usage and by plain `bool` comparisons already used for latch fields (e.g.
   Territory's `under_eviction`)."* No landed pack contains a `bool` `deffield` at all, and
   `TerritorySystem` has not been ported — its own Phase-1 inventory closed on a DEFER verdict
   (`reports/territory-port-phase1-inventory-2026-08-11.md:3`), so `under_eviction` has no BSL
   existence to be precedent for. The two landed packs that actually needed a boolean encode 0/1
   and say why in-file: *"0/1 rather than #t/#f: BSL has Bool (§3.1) but `deffield` has no bool"*
   (`content/scenarios/vitality-conformance.bscn:20`,
   `content/scenarios/vitality-lifecycle-combined-conformance.bscn:34`).
   **The verdict survives on different evidence, which the row should carry instead:** a
   `defconst` MAY hold a `Bool` literal — `scenario.rs:466-470`'s own enumeration is *"`Int`,
   `Scaled` (`p`/`i`/`c`/`r`), and `Bool` (defines carry toggles as well as magnitudes); `Currency`
   is refused"*, handled at `scenario.rs:525` — and a `Value::Bool` const IS usable directly as a
   `<cond>` (`evaluator.rs:1315-1320`). So the enable gate is PORTABLE NOW **as a bool
   `defconst`**, and would NOT be portable as a bool node field.
   Two riders worth recording, both verified: (i) those in-file "no bool" comments are themselves
   STALE — `"bool" => Ok(BslType::Bool)` at `declarations.rs:649`; (ii) the landing buys almost
   nothing at evaluation, because `bind_field_value` renders every non-enum declared type as
   `Value::Real(stored)` (`tick.rs:312-327`), `field_of_node` returns `Value::Real` unconditionally
   (`evaluator.rs:1281-1291`), and `numeric_write_value` refuses a `Value::Bool` write outright —
   *"cannot store {other:?} as a numeric node attribute"* (`structural_verbs.rs:1231-1233`). The
   0/1 `int` convention remains the operative idiom for any stored boolean.

5. **CORRECTION — §6's demand-signal READ row mis-attributes its second gap to "Slice 2/3
   (edge-attribute reads)".** Reading `condition`/`conductivity` off an `InfrastructureLinkState`
   is not a Slice-2 shape even in principle. Slice 2 supplies the *accessors* (`edges`/
   `edge-between`, `evaluator.rs:503-505`) that produce an `EdgeRef` over a **dyadic `EdgeType`
   member**; it supplies no storage. `GraphSubstrate`'s only attribute reader is `node_attribute`
   (`substrate.rs:142`), and a dyadic edge's entire state is the single `f64 strength` that
   `add_edge` takes (`substrate.rs:111-117`; the implicit `<edge-type>/strength` field, D32,
   `declarations.rs:13, 317-320`). A link record carrying `condition` **plus** `conductivity`
   **plus** a five-key `capacity` map therefore needs the D35/D65 edge-attribute-*storage*
   widening — the same gap `structural_verbs.rs:387-398` names when it refuses `update-edge`
   (*"GraphSubstrate keys an edge to one f64 strength"*) — on top of the substrate/vocabulary
   question this inventory correctly identifies. The row's "two stacked gaps, not one" reading is
   right; the second gap is D35/D65, not Slice 2. Same re-attribution applies to
   `aggregate_connectivity_by_county_pair`'s per-link reads.

6. **CONFIRMATION — the byte-gate finding (§7) is exactly right, and it is the sharpest thing in
   this inventory.** `graph_content_hash`'s own docstring states the exclusion verbatim:
   *"Graph metadata (`g.graph`: economy, event log, opposition states) is also excluded, because
   the spec's field set is nodes/edges/actions"* (`tools/regression_test.py:939-943`), and
   `transport_demand_signal` is dropped by `TERRITORY_EXCLUDED_FIELDS` (`world_state.py:152`)
   before the projection is ever hashed. So even a hypothetical enabled-and-meshed canonical
   scenario leaves every one of this system's outputs outside the byte gate. Tick position 9.5
   confirmed (`transport.py:115`), between `ImperialRentSystem` (9.0, `economic.py:37`) and
   `DispossessionEventSystem` (10.0, `dispossession_events.py:36`), against `_SYSTEM_CLASSES`
   (`simulation_engine.py:328-360`), which `_DEFAULT_SYSTEMS` derives by sorting on `position`
   (`:376-378`). **No RESERVED-LINE surface** — independently confirmed: no doctrine content, no
   National Question parameter, no outcome-definition logic in this system's reachable call graph.

**FINAL VERDICT: NOT-A-PACK today / BLOCKED on a prerequisite architecture ruling (not a BSL
language slice) — UPHELD.** The corridor-mesh substrate is D2-ruled deliberately outside
`BabylonGraph`/`GraphProtocol` with no BSL shape to port against, and the system provably never
executes on any production or canonical-scenario path (default-OFF plus a key nothing populates).
Two rows change their evidence without changing their grade: the master enable gate is portable as
a bool **`defconst`**, not on a fabricated bool-latch-field precedent (correction 4); and the
per-link reads stack the D35/D65 **storage** gap on the substrate gap, not Slice 2 (correction 5).
Recording the second one matters for sequencing: this system does not become cheaper when slices
2-3 land, which is precisely why "NOT-A-PACK" rather than "DEFER" is the right verdict.

**INADEQUATE-COVERAGE NOTE.** §1's "Reference BSL packs/docs read for format" lists
`docs/reference/bsl-language.rst` and the Territory inventory — **no Rust source at all**. Every
§6 row is nonetheless graded against the Rust surface, and both corrections above are direct
consequences: the fabricated bool-latch precedent (correction 4) and the Slice-2/D35 conflation
(correction 5) are each one file-read away from being caught. A re-read must ground §6 in
`rust/crates/babylon-graph/src/substrate.rs` (the trait's actual attribute surface — 249 lines,
read the whole fn list), `rust/crates/babylon-bsl/src/structural_verbs.rs:387-398` (the D35/D65
refusal that owns the second gap) and `rust/crates/babylon-bsl/src/scenario.rs:460-530` (the
`defconst` literal set that owns the master-gate row's real evidence). The Python-side coverage is
complete and needs nothing added.
