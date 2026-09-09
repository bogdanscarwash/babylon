# ImperialRentSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `ImperialRentSystem` (`src/babylon/engine/systems/economic.py`, 836 lines,
tick position 9.0) is a 5-phase pipeline (Extraction → Tribute → Wages → Subsidy → Decision) plus
two conditionally-wired sub-stages (Φ-distribution to counties, Vol II hex circulation) that are
dormant on every canonical `qa:regression` scenario. Every one of the five core phases writes an
edge attribute (`value_flow`) that the current `GraphSubstrate` trait has **no storage for at all**
— `update-edge` is a parsed-but-refused verb (`structural_verbs.rs:371-382`, citing draft rulings
D35/D65: "GraphSubstrate keys an edge to one f64 strength and gives a hyperedge no attributes at
all"), independent of and prior to the Slice-2 edge-attribute-*read* gap that separately blocks
Phase 4's `subsidy_cap` read. The pipeline's own pool state (`GlobalEconomy`, round-tripped through
`graph.get_graph_attr("economy")`) is graph-scoped singleton state with no per-node home; the
language spec names a resolution pattern (D39/D40, a `:ceiling 1` carrier `NodeType`) but no landed
pack has ever exercised it, and minting the carrier type is itself "amendment territory" per the
spec's own text. Phase 4's subsidy trigger calls a `math.exp`-based sigmoid
(`calculate_acquiescence_probability`) that is a live libm-nondeterminism hazard AND an ADR172/173
"no imposed functional forms" PORT-QUESTION, not a mechanical port. Tensor/Leontief math in
`src/babylon/domain/economics/` is confirmed **not invoked** by this system's `step()` at all — it
belongs to a same-named but distinct pipeline (`domain/economics/tick/system/imperial_rent.py`)
owned by `TickDynamicsSystem`.

**Verdict: BLOCKED — no edge-attribute-write substrate lane (D35/D65) blocks every phase's core
write; graph-scoped singleton pool state has no landed representation; Phase 4 needs Slice-2
edge reads; the acquiescence sigmoid is a reserved theory question, not a mechanical port.**

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/economic.py` | 836 | **The target.** `ImperialRentSystem`, 5 phases + 2 conditionally-wired sub-stages. |
| `src/babylon/engine/systems/phi_distribution.py` | 109 | Sub-stage 5b: `distribute_phi_week_to_counties` — writes to a `BoundaryFlowRegister` (Postgres-facing bookkeeping), touches **no graph node/edge**. Invoked from `economic.py:134-136` (local import). |
| `src/babylon/engine/systems/vol2_circulation.py` | 363 | Sub-stage 5c: `Vol2CirculationStep.step` — sparse-matrix (`scipy`/`numpy`) OD-matrix redistribution of `TERRITORY.v` via a `ScaleAdjunction` hex↔county binding, plus `BoundaryFlowRegister` rows. Invoked from `economic.py:185-191` when context-wired. |
| `src/babylon/domain/economics/node_kinds.py` | 69 | `NodeKind`/`BoundaryEdgeKind` — pure `StrEnum` vocabulary for boundary-register rows, no math. |
| `src/babylon/domain/economics/boundary_flow_register.py` | (not read in full — imported only) | `BoundaryFlowRegister`/`NodeKind`/`BoundaryEdgeKind` re-exports; the in-memory buffer + Postgres facade for the boundary-register table (`n`/`boundary_flow_register` in migrations). Persistence layer, not graph state. |
| `src/babylon/domain/economics/tick/graph_bridge.py` | (not read in full) | `resolve_county_identity` — used only inside `Vol2CirculationStep.step` (vol2_circulation.py:66,203). |
| `src/babylon/domain/dialectics/instances/scale.py` | (not read in full) | `ScaleAdjunction` (`allocate`/`aggregate`, the Lawverian adjoint) — used only inside `Vol2CirculationStep.step`. |
| `src/babylon/formulas/dynamic_balance.py` | 119 | `BourgeoisieDecision` (plain string-constant class, not `StrEnum`) + `calculate_bourgeoisie_decision` — Phase 5's decision-matrix formula. Pure branching, no libm. |
| `src/babylon/formulas/survival_calculus.py` | 111 | `calculate_acquiescence_probability` (sigmoid, **`math.exp`**) + `calculate_revolution_probability` (division, no libm) — both called only from Phase 4 (`_process_subsidy_phase`). |
| `src/babylon/engine/formula_registry.py` | — | `FormulaRegistry` — `services.formulas.get(name)` raises `KeyError` on a missing registration (`formula_registry.py:58`), never silently returns `None`. |
| `src/babylon/kernel/node_access.py` | 37 | `class_consciousness_from_node` (lines 15-37) — reads `node.ideology` (an `IdeologicalProfile` dict) `.class_consciousness`, `0.0` if absent/malformed. Called from Phase 1. |
| `src/babylon/kernel/tick_partition.py` | 30 | `TickPartition.MATERIAL_BASE` — this system's declared partition. |
| `src/babylon/kernel/system_base.py` | — | `SystemBase` — `ImperialRentSystem` overrides `creates_value: ClassVar[bool] = True` (default `False`, `system_base.py:72`) since its extraction phase legitimately mutates `wealth` outside the c+v+s conservation check (economic.py:40-44). **Does NOT call** `self._write_clamped`/`self._publish`/`self._get_persistent_data` anywhere — every clamp is hand-written `min`/`max` nesting, every event goes through `services.event_bus.publish(Event(...))` directly. |
| `src/babylon/kernel/system_protocol.py` | — | `ContextType = "TickContext"` (string type alias). |
| `src/babylon/kernel/event_bus.py` | — | `Event` — frozen dataclass, `type: str`, `tick: int`, `payload: dict[str, Any]`, `timestamp` derived from `tick` (event_bus.py:33-55). |
| `src/babylon/models/entities/economy.py` | 81 | `GlobalEconomy` — frozen Pydantic model, the "Gas Tank" (`imperial_rent_pool`, `current_super_wage_rate`, `current_repression_level`). Round-tripped through `graph.graph["economy"]`, **not** a node. |
| `src/babylon/models/entities/social_class.py` | — | `SocialClass` entity — field types/domains for every `SOCIAL_CLASS` attribute this system reads/writes (§3). |
| `src/babylon/models/entities/relationship.py` | — | `Relationship` entity — `value_flow: Currency`, `subsidy_cap: Currency`, `solidarity_strength: Coefficient` (relationship.py:92,110,116). |
| `src/babylon/models/enums/topology.py` | — | `EdgeType.EXPLOITATION`/`TRIBUTE`/`WAGES`/`CLIENT_STATE` (topology.py:99,103-105), `NodeType.SOCIAL_CLASS`. |
| `src/babylon/models/enums/events.py` | — | `EventType.SURPLUS_EXTRACTION`/`SUPERWAGE_CRISIS`/`IMPERIAL_SUBSIDY`/`ECONOMIC_CRISIS` (events.py:64,65,69,91). |
| `src/babylon/models/enums/social.py` | — | `SocialRole` — 8-member `StrEnum` (social.py:24-32); only `CORE_BOURGEOISIE` is compared by this system. |
| `src/babylon/models/types.py` | 179 | `Currency`/`Probability`/`Coefficient`/`Intensity`/`Ideology` — all `Annotated[float, Field(...), SnapToGrid]`; the Python "Currency" is a plain unbounded-above `[0,∞)` float with 1e-5-grid quantization applied only at Pydantic model instantiation, **never mid-tick** (same finding as the Territory inventory: `topology/graph.py` `update_node`/`update_edge` are plain dict merges). |
| `src/babylon/config/defines/economy_basic.py` | 585 | `EconomyDefines` Pydantic model — every coefficient this system reads, with domains (§2e). |
| `src/babylon/config/defines/_assembler.py` | — | `GameDefines.DEFAULT_ORGANIZATION`/`DEFAULT_REPRESSION_FACED`/`DEFAULT_SUBSISTENCE` — computed properties aliasing `survival.default_organization`/`default_repression`/`default_subsistence` (`_assembler.py:262-274`). |
| `src/babylon/data/defines.yaml` | economy: 70-131 (used subset 71-99), survival: 163-169, timescale: 372-375, precision: 363-367 | Player-editable coefficient values. |
| `src/babylon/engine/simulation_engine.py` | — | `_SYSTEM_CLASSES` tuple (`simulation_engine.py:329-362`) — confirms tick position 9.0, between `SolidaritySystem` (8.0) and `TransportSystem` (9.5, default-OFF). |
| `src/babylon/topology/graph.py` | `update_edge`: 690-701 | Concrete `BabylonGraph.update_edge` — plain `dict.update(attributes)` merge, **no quantization mid-tick**, matching `update_node`'s already-documented behavior. |
| `src/babylon/game/session.py` | 1560-1593 | The **only** production wiring site for the two conditional sub-stages: `context["boundary_flow_register"]`/`["external_nodes_phi"]`/`["county_exposure_by_external"]`/`["session_id"]`/`["simulated_year"]`/`["vol2_step"]`, all gated by `self._trade is not None` (session.py:1582). |

**NOT exercised by `ImperialRentSystem.step()` at all, despite living in `src/babylon/domain/economics/`:**
the entire Leontief/tensor pipeline — `domain/economics/tensor*`, `tensor_hierarchy/`,
`domain/economics/tick/system/imperial_rent.py`'s `compute()` (grep-confirmed: `economic.py` has
zero imports from `domain.economics.tick` or `domain.economics.tensor*`, `economic.py:1-22`'s full
import block). That module computes a per-county `tick_phi_hour` via BEA I-O coefficients /
QCEW / Hickel ERDI data and is `TickDynamicsSystem`'s territory (a different, earlier-position
System) — it shares the words "imperial rent" with this system by name collision only. The
`tests/integration/economics/tick/test_imperial_rent_*.py` files (pipeline/real_wiring/
calibration/perf) all target `domain.economics.tick.system.imperial_rent.compute`, confirmed by
their own docstrings (`test_imperial_rent_pipeline.py:1-11`, `test_imperial_rent_real_wiring.py:1-16`)
— **none of them are conformance oracles for the System this inventory covers.**

## 2. COMPUTATION CATALOG (execution order, `economic.py:46-86`)

### Phase 1 — Extraction (`_process_extraction_phase`, economic.py:239-345)
- **(a)** Along every EXPLOITATION edge, the owner extracts a TRPF-decayed fraction of the worker's
  wealth, reduced by the worker's class consciousness; the extracted amount transfers wealth from
  worker to owner and is recorded on the edge.
- **(b)** Per-tick efficiency: `base_extraction_efficiency = annual_extraction_efficiency /
  weeks_per_year` (economic.py:255); TRPF decay `trpf_multiplier = max(trpf_floor, 1.0 -
  (trpf_coefficient * tick))` (economic.py:261); `extraction_efficiency = base_extraction_efficiency
  * trpf_multiplier` (economic.py:262). Per edge: `rent = extraction_efficiency * worker_wealth *
  (1.0 - consciousness)` (economic.py:289), `rent = min(rent, worker_wealth)` (economic.py:292).
  Writes: `worker.wealth = max(0.0, worker_wealth - rent)` (economic.py:295),
  `target.wealth = target_wealth + rent` (economic.py:297), edge `value_flow=rent`
  (economic.py:300-302).
- **(c) Reads:** `EdgeType.EXPLOITATION` edges; `worker.active`, `target.active`, `worker.wealth`,
  `worker.ideology` (via `class_consciousness_from_node`), `target.wealth`, `target.role` (compared
  to `SocialRole.CORE_BOURGEOISIE`, economic.py:324-327); `context["tick"]` (default 0).
- **(d) Writes:** `worker.wealth`, `target.wealth`, edge `value_flow`; **local Python dict**
  `tick_context["tribute_inflow"]`/`["current_pool"]` (+= rent, only when `target_role ==
  CORE_BOURGEOISIE`, economic.py:327-329) — this dict is NOT graph state, it is a plain Python
  `dict` threaded by reference through all five `_process_*` calls in `step()`
  (economic.py:59-74) and never written to the graph until `_save_economy` (Phase 5's tail).
  Optional: a `BoundaryFlowRegister.record(...)` L-RECEIPTS provenance row (economic.py:311-321,
  EXPLOITATION_FLOW kind) when `rent > 0.0` and `services.boundary_register is not None` and its
  `session_id is not None` — `ServiceContainer.boundary_register` defaults to `None`
  (`engine/services.py:269`), and no canonical `qa:regression` path ever sets it (grep-confirmed
  zero `ServiceContainer(` construction / `boundary_register` reference in `tools/
  regression_test.py`) — **dormant on every canonical scenario**.
- **(e) Defines:** `economy.extraction_efficiency` (0.8, `[0,1]`, defines.yaml:71), `timescale.
  weeks_per_year` (52, `>=1`, defines.yaml:374), `economy.trpf_coefficient` (0.0005, `[0,0.01]`,
  defines.yaml:90), `economy.trpf_efficiency_floor` (0.1, `[0,1]`, defines.yaml:99),
  `economy.negligible_rent` (0.01, `>=0.0`, **no upper bound**, defines.yaml:86).
- **(f) Events:** `EventType.SURPLUS_EXTRACTION` when `rent > negligible_rent`
  (economic.py:332-345); payload `{source_id, target_id, amount, mechanism="imperial_rent"}`.

### Phase 2 — Tribute (`_process_tribute_phase`, economic.py:347-400)
- **(a)** Along every TRIBUTE edge, the comprador keeps a configured cut of its wealth and forwards
  the rest to the core bourgeoisie.
- **(b)** `cut_amount = comprador_wealth * comprador_cut` (economic.py:381), `tribute_amount =
  comprador_wealth - cut_amount` (economic.py:382). Writes: `source.wealth = cut_amount`
  (economic.py:385, **not** a subtraction — a hard-set to the retained cut), `target.wealth =
  target_wealth + tribute_amount` (economic.py:387), edge `value_flow=tribute_amount`
  (economic.py:390-392).
- **(c) Reads:** `EdgeType.TRIBUTE` edges; `source.active`, `target.active`, `source.wealth`
  (skipped entirely if `<= 0`, economic.py:377-378), `target.wealth`, `target.role`.
- **(d) Writes:** `source.wealth`, `target.wealth`, edge `value_flow`; local dict
  `tick_context["tribute_inflow"]`/`["current_pool"]` (+= `tribute_amount`, `CORE_BOURGEOISIE`-gated,
  economic.py:398-400).
- **(e) Defines:** `economy.comprador_cut` (0.9, `[0,1]`, defines.yaml:72).
- **(f) Events:** none.

### Phase 3 — Wages (`_process_wages_phase`, economic.py:402-544)
- **(a)** Along every WAGES edge, the labor aristocracy is paid its ProductionSystem-captured
  productivity plus a "super-wage" bribe drawn from the shared rent pool (capped at what remains in
  the pool and at what the bourgeoisie can afford); a PPP purchasing-power multiplier is stamped on
  the payee alongside bookkeeping fields for the value-form opposition read. Emits
  `SUPERWAGE_CRISIS` when the pool is exhausted, independent of whether the wage transfer itself
  proceeds.
- **(b)** `super_wage_rate = tick_context["wage_rate"] / weeks_per_year` (economic.py:421-423);
  `ppp_multiplier = 1.0 + (extraction_efficiency * superwage_multiplier * superwage_ppp_impact)`
  (economic.py:432). Per edge: `productivity_value = la_production.get(edge.target_id, 0.0)`
  (economic.py:453, reads the `la_production` **graph attribute**, not a node field);
  `max_bonus = tribute_inflow * super_wage_rate` (economic.py:457), `super_wage_bonus =
  min(max_bonus, available_pool)` (economic.py:458); `total_wages = productivity_value +
  super_wage_bonus` (economic.py:507), capped `total_wages = min(total_wages,
  bourgeoisie_wealth)` (economic.py:510). Writes: `source.wealth -= total_wages`
  (economic.py:513); `target.wealth = current_wealth + total_wages` (economic.py:515),
  `target.effective_wealth = new_nominal_wealth + total_wages * (ppp_multiplier - 1.0)`
  (economic.py:519), `target.unearned_increment = total_wages * (ppp_multiplier - 1.0)`
  (economic.py:520), `target.ppp_multiplier = ppp_multiplier` (economic.py:521),
  `target.w_paid = total_wages` (economic.py:529), `target.v_produced = productivity_value`
  (economic.py:530) — `w_paid`/`v_produced` are **not** declared `SocialClass` model fields; both
  are sanctioned graph-only extras in `EXTRA_STAMPABLE_ATTRIBUTES[NodeType.SOCIAL_CLASS]`
  (`sentinels/vocabulary/registry.py:208-209`, "engine/systems/economic.py (market pricing)").
  Edge `value_flow=total_wages` (economic.py:534-536, nominal amount, ignores the PPP uplift).
  Pool debit: `actual_bonus_paid = max(0.0, min(super_wage_bonus, total_wages -
  productivity_value))` (economic.py:540-541); `tick_context["wages_outflow"] +=
  actual_bonus_paid`, `tick_context["current_pool"] -= actual_bonus_paid`, `available_pool =
  tick_context["current_pool"]` re-read for the **next loop iteration** (economic.py:542-544) —
  a genuine sequential, edge-order-dependent accumulator within one phase (see §4 item 6).
- **(c) Reads:** `EdgeType.WAGES` edges; `source.active`, `target.active` (checked for the crisis
  event BEFORE the active-skip, economic.py:449-450, 491-496 — order matters, see the event
  condition below); `source.wealth`; graph attribute `la_production` (a `dict[node_id, float]`
  written by `ProductionSystem` @3.0, `production.py:129,194,207` — same-tick, fresh read, since
  Production runs before ImperialRent).
- **(d) Writes:** `source.wealth`, `target.wealth`, `target.effective_wealth`,
  `target.unearned_increment`, `target.ppp_multiplier`, `target.w_paid`, `target.v_produced`, edge
  `value_flow`; local dict `tick_context["wages_outflow"]`/`["current_pool"]`.
- **(e) Defines:** `economy.superwage_multiplier` (1.0, `>=0.0`, **no upper bound**,
  defines.yaml:75), `economy.superwage_ppp_impact` (0.5, `[0,1]`, defines.yaml:76),
  `economy.extraction_efficiency` (re-read, same as Phase 1's raw annual value — **not** the
  TRPF-decayed per-tick value Phase 1 actually applies; a distinct read of the same coefficient for
  a different purpose), `economy.negligible_rent` (reused as the pool-exhaustion noise floor,
  economic.py:462), `timescale.weeks_per_year`.
- **(f) Events:** `EventType.SUPERWAGE_CRISIS` when `available_pool <= negligible AND
  super_wage_bonus <= negligible` (economic.py:462-487), regardless of whether the entities are
  active — fires even for edges about to be skipped by the active-check that follows it
  (economic.py:491-496); payload includes `stability_ratio`-free fields (`payer_id`, `receiver_id`,
  `productivity_value`, `super_wage_bonus`, `available_pool`, `bourgeoisie_wealth`,
  `bourgeoisie_active`, `narrative_hint`).

### Phase 4 — Subsidy (`_process_subsidy_phase`, economic.py:546-666)
- **(a)** For every CLIENT_STATE edge, compute the client state's survival-probability ratio
  (revolution vs. acquiescence); if that ratio exceeds a trigger threshold, convert bourgeoisie
  wealth (capped by the edge's `subsidy_cap`, the tribute-inflow-derived pool share, the
  bourgeoisie's own wealth, and the remaining pool) into the target's repression capacity. Emits
  `IMPERIAL_SUBSIDY` on every successful transfer.
- **(b)** `p_acquiescence = calculate_acquiescence_probability(wealth=target_wealth,
  subsistence_threshold=target_subsistence, steepness_k=...)` (economic.py:596-600) — **sigmoid,
  see §4 for the `math.exp` hazard**. `p_revolution = calculate_revolution_probability(cohesion=
  target_organization, repression=target_repression)` (economic.py:601-604). `stability_ratio =
  p_revolution / p_acquiescence` if `p_acquiescence > 0` else (`1.0` if `p_revolution > 0` else
  `0.0`) (economic.py:609-613). Gate: `if stability_ratio < subsidy_trigger_threshold: continue`
  (economic.py:615-617). `max_subsidy = min(subsidy_cap, tribute_inflow *
  subsidy_conversion_rate)` (economic.py:622), then `min(..., source_wealth)` (economic.py:624),
  then `min(..., available_pool)` (economic.py:628); negligible-gated `continue`
  (economic.py:630-632). Writes: `source.wealth = source_wealth - max_subsidy`
  (economic.py:636); `repression_boost = max_subsidy * subsidy_conversion_rate`
  (economic.py:638); `target.repression_faced = min(1.0, target_repression +
  repression_boost)` (economic.py:639-640); edge `value_flow=max_subsidy`
  (economic.py:643-645). Pool debit: `tick_context["subsidy_outflow"] += max_subsidy`,
  `["current_pool"] -= max_subsidy` (economic.py:648-649).
- **(c) Reads:** `EdgeType.CLIENT_STATE` edges; `source.active`, `target.active`, `target.wealth`,
  `target.organization` (default `services.defines.DEFAULT_ORGANIZATION`), `target.repression_faced`
  (default `DEFAULT_REPRESSION_FACED`), `target.subsistence_threshold` (default
  `DEFAULT_SUBSISTENCE`), `source.wealth`, **edge attribute `subsidy_cap`** (default `0.0`,
  economic.py:593 — `edge.attributes.get("subsidy_cap", 0.0)`, an `EdgeRef` field read); local dict
  `tick_context["tribute_inflow"]`, `["current_pool"]`.
- **(d) Writes:** `source.wealth`, `target.repression_faced`, edge `value_flow`; local dict
  `tick_context["subsidy_outflow"]`/`["current_pool"]`.
- **(e) Defines:** `economy.subsidy_trigger_threshold` (0.8, `[0,1]`, defines.yaml:84),
  `economy.subsidy_conversion_rate` (0.1, `[0,1]`, defines.yaml:83),
  `economy.negligible_subsidy` (0.01, `>=0.0`, **no upper bound**, defines.yaml:87),
  `survival.steepness_k` (10.0, `>0.0`, defines.yaml:164, sigmoid steepness),
  `precision.epsilon` (1e-9, `(0, 0.001]`, defines.yaml:366, division-by-zero guard inside
  `calculate_revolution_probability`), `survival.default_organization`/`default_repression`/
  `default_subsistence` (defines.yaml:166-167,165, via the `DEFAULT_*` properties).
- **(f) Events:** `EventType.IMPERIAL_SUBSIDY` on every executed transfer (economic.py:651-666);
  payload includes `stability_ratio`.

### Phase 5 — Decision (`_process_decision_phase`, economic.py:668-750)
- **(a)** The bourgeoisie reads the pool's fraction of its initial value and last-tick's
  capital-labor opposition gap, and picks one of five policies (BRIBERY/AUSTERITY/IRON_FIST/
  CRISIS/NO_CHANGE) via a pure threshold decision matrix, adjusting the shared wage rate and
  repression level for next tick. Emits `ECONOMIC_CRISIS` only on the CRISIS branch.
- **(b)** `pool_ratio = current_pool / initial_pool if initial_pool > 0 else 0.0`
  (economic.py:683). `aggregate_tension = self._calculate_aggregate_tension(graph)`
  (economic.py:686) — reads graph attribute `opposition_states.capital_labor.gap`
  (economic.py:776-780), **stale by one tick**: `ContradictionSystem` @18 writes
  `opposition_states` (`contradiction.py:90`, `OPPOSITION_STATES_ATTR`), and this system runs @9.0
  — before it, this same tick — so it reads LAST tick's snapshot; `0.0` on tick 0 (documented
  in-code, economic.py:755-763). `decision, wage_delta, repression_delta =
  calculate_bourgeoisie_decision(pool_ratio, aggregate_tension, high_threshold, low_threshold,
  critical_threshold, ...)` (economic.py:704-717) — pure nested-`if` threshold matrix
  (`dynamic_balance.py:82-118`), no libm. Clamp: `new_wage_rate = max(min_wage,
  min(max_wage, current_wage_rate + wage_delta))` (economic.py:726); `new_repression =
  max(0.0, min(1.0, current_repression + repression_delta))` (economic.py:727).
- **(c) Reads:** local dict `tick_context["current_pool"]`, `["wage_rate"]`,
  `["repression_level"]`; graph attribute `opposition_states` (a `dict[str, dict]`, not a node
  field); `initial_pool` (a `step()`-local variable, `economy.initial_rent_pool` at load time).
- **(d) Writes:** local dict `tick_context["wage_rate"]`/`["repression_level"]` only (persisted to
  the graph by the immediately-following `_save_economy`, not by this phase itself).
- **(e) Defines:** `economy.pool_high_threshold` (0.7, `[0,1]`, defines.yaml:78),
  `pool_low_threshold` (0.3, `[0,1]`, defines.yaml:79), `pool_critical_threshold` (0.1, `[0,1]`,
  defines.yaml:80), `bribery_wage_delta` (0.05, `[-1,1]`, defines.yaml:92), `austerity_wage_delta`
  (-0.05, `[-1,1]`, defines.yaml:93), `iron_fist_repression_delta` (0.1, `[0,1]`, defines.yaml:94),
  `crisis_wage_delta` (-0.15, `[-1,1]`, defines.yaml:95), `crisis_repression_delta` (0.2, `[0,1]`,
  defines.yaml:96), `bribery_tension_threshold` (0.7, `[0,1]`, defines.yaml:97),
  `iron_fist_tension_threshold` (0.5, `[0,1]`, defines.yaml:98), `min_wage_rate` (0.05, `[0,1]`,
  defines.yaml:81), `max_wage_rate` (0.35, `[0,1]`, defines.yaml:82).
- **(f) Events:** `EventType.ECONOMIC_CRISIS` only when `decision ==
  BourgeoisieDecision.CRISIS` (economic.py:733-750); payload includes `decision` (a plain string,
  not a serializable enum — `BourgeoisieDecision` is a class of string constants).

### Tail — `_save_economy` (economic.py:807-836), runs unconditionally after all 5 phases
- **(a)** Applies a background TRPF decay to the pool and writes the whole `GlobalEconomy` back to
  the graph as one opaque blob.
- **(b)** `current_pool = current_pool * (1.0 - decay_rate)` (economic.py:829). Writes:
  `graph.set_graph_attr("economy", GlobalEconomy(imperial_rent_pool=max(0.0, current_pool),
  current_super_wage_rate=tick_context["wage_rate"], current_repression_level=
  tick_context["repression_level"]).model_dump())` (economic.py:831-836) — a **graph-level**
  attribute, not a node/edge write; confirmed the sole writer/reader pair anywhere in `src/`
  (grep: `get_graph_attr("economy")` matches only `economic.py:796` — `_load_economy` — and
  `world_state.py:705,924` at the `WorldState↔graph` seed/round-trip boundary; **zero** other
  System reads it).
- **(e) Defines:** `economy.rent_pool_decay` (0.002, `[0,0.01]`, defines.yaml:91).
- **(f) Events:** none.

### Sub-stage 5b — Φ-distribution (`_invoke_phi_distribution_if_wired`, economic.py:88-156)
- **(a)** For each external node carrying a nonzero annual Φ inflow, split the weekly slice across
  US counties by a caller-supplied exposure-weight map and record a `DRAIN_EDGE` boundary-register
  row per county — **no graph node or edge is touched anywhere in this sub-stage**.
- **(b)** `phi_week = phi_year_inflow / weeks_per_year` (phi_distribution.py:91); per county:
  `amount = phi_week * weight` (phi_distribution.py:94); `register.record(...)`
  (phi_distribution.py:96-105). Raises `ValueError` if `phi_year_inflow < 0` or exposure weights
  don't sum to `1.0` within `1e-9` (phi_distribution.py:79-89, no silent renormalization).
- **(c) Reads:** `context["boundary_flow_register"]`, `["session_id"]`, `["external_nodes_phi"]`,
  `["county_exposure_by_external"]` — silent no-op if ANY is `None` (economic.py:117-127).
- **(d) Writes:** `BoundaryFlowRegister` rows only (persistence, not graph).
- **(e) Defines:** `timescale.weeks_per_year` (via `services.defines.timescale.weeks_per_year` when
  `services` is supplied, else a hardcoded `52.0` fallback, economic.py:129-131 — byte-identical to
  the tunable's own default).
- **(f) Events:** none (only exceptions).
- **Wiring:** the four gate keys are set **only** by `game/session.py:1583-1589`, gated on
  `self._trade is not None` — never by `tools/regression_test.py`'s `persistent_context = {}`
  (`regression_test.py:1023`). **Dormant on every canonical `qa:regression` scenario.**

### Sub-stage 5c — Vol II Circulation (`_invoke_vol2_circulation_if_wired`, economic.py:158-199)
- **(a)** Delegates whole to `Vol2CirculationStep.step` — a sparse matrix-vector redistribution of
  variable capital (`v`) across US counties via a LODES commute OD matrix, allocated to hex grain
  and aggregated back through a `ScaleAdjunction`. See `vol2_circulation.py:1-364` in full (§1).
  `v[county, t+1] = sum_j(OD[j, county] * v[j, t] / row_sum[j])` (vol2_circulation.py:117, the
  module's own docstring formula), computed via `scipy.sparse` CSR × dense `numpy` vector
  multiplication (`vol2_circulation.py:235`, `year_matrix.matrix.T @ normalized`).
- **(c) Reads:** `context["vol2_step"]`, `["boundary_flow_register"]`, `["session_id"]`,
  `["simulated_year"]` — silent no-op if any is `None` (economic.py:175-180); inside the sub-stage:
  every `TERRITORY` node's `v` attribute for which `resolve_county_identity(node) is not None`
  (vol2_circulation.py:199-207).
- **(d) Writes:** `TERRITORY.v` for every eligible county (vol2_circulation.py:342-344); a pair of
  `COMMUTE_OUT`/`TRADE_EDGE` `BoundaryFlowRegister` rows per external-destination flow
  (vol2_circulation.py:296-317). Stashes `context["vol2_circulation_result"]`
  (economic.py:199, a `CirculationStepResult` dataclass) for the post-hoc conservation auditor.
- **(e) Defines:** none read directly by this call site (the loader/matrix/tolerance constants live
  inside `Vol2CirculationStep`'s own construction, not read per-tick here).
- **(f) Events:** none — raises `CirculationConservationViolation` (a `RuntimeError` subclass) if
  the FR-010 conservation residual exceeds `1e-9 * max(pre_total_v, 1.0)`
  (vol2_circulation.py:82,320-330) — a genuine per-tick fail-loud path, not an event.
- **Wiring:** same as 5b — `game/session.py:1588-1589` only, dormant on every canonical scenario
  (`vol2_step` additionally requires `self._trade.vol2_step is not None`, session.py:1588).

**Events emitted by `ImperialRentSystem` itself: 4 distinct `EventType` values** —
`SURPLUS_EXTRACTION`, `SUPERWAGE_CRISIS`, `IMPERIAL_SUBSIDY`, `ECONOMIC_CRISIS` (grep-confirmed,
`economic.py:336,470,655,737` — the only 4 `EventType.` occurrences in the file). Per the CURRENT
BSL surface: `TickReport` carries no event log, so every one of these 4 is a WS1 (#502) ledger row,
unpinnable by a conformance golden today.

## 3. TYPE INVENTORY

Runtime storage note (identical mechanism to the Territory inventory's finding): `BabylonGraph.
update_node`/`update_edge` (`topology/graph.py:660-670`, `690-701`) are plain dict merges with no
type coercion or quantization; the `Currency`/`Probability`/`Coefficient` Pydantic `SnapToGrid`
(1e-5 grid) applies only at model instantiation (scenario seed / `WorldState` round-trip), never
mid-tick. All in-tick arithmetic below is raw Python `float`/`str`.

| Attribute | Node/edge type | Python model type | Domain | Category |
|---|---|---|---|---|
| `wealth` | SOCIAL_CLASS | `Currency` | `[0,∞)` | unbounded real, money-semantic |
| `active` | SOCIAL_CLASS | `bool` | `{T,F}` | boolean gate (read-only here) |
| `ideology` | SOCIAL_CLASS | `IdeologicalProfile` (nested Pydantic; `.class_consciousness` read via a dict-shaped accessor, not the model directly) | `.class_consciousness` `[0,1]` | nested-object field, one sub-field read |
| `role` | SOCIAL_CLASS | `SocialRole` (`StrEnum`, 8 members) | closed set | **enum discriminant** (only `CORE_BOURGEOISIE` compared) |
| `organization` | SOCIAL_CLASS | `Probability` | `[0,1]` | unit-interval |
| `repression_faced` | SOCIAL_CLASS | `Probability` | `[0,1]` | unit-interval (read AND written — Phase 4 target) |
| `subsistence_threshold` | SOCIAL_CLASS | `Currency` | `[0,∞)` | unbounded real, money-semantic |
| `effective_wealth` | SOCIAL_CLASS | `Currency` | `[0,∞)` | unbounded real (write-only here) |
| `unearned_increment` | SOCIAL_CLASS | `Currency` | `[0,∞)` | unbounded real (write-only here) |
| `ppp_multiplier` | SOCIAL_CLASS | `float` (plain, no `Annotated` constraint found) | `>=1.0` by construction (`1.0 + non-negative`) | unbounded real (write-only here) |
| `w_paid` | SOCIAL_CLASS | **not a declared model field** — sanctioned `EXTRA_STAMPABLE_ATTRIBUTES` graph-only extra (`vocabulary/registry.py:208`) | same as `total_wages`, `[0,∞)` implied | graph-only extra (write-only here) |
| `v_produced` | SOCIAL_CLASS | **not a declared model field** — sanctioned extra (`vocabulary/registry.py:209`) | `[0,∞)` implied | graph-only extra (write-only here) |
| `value_flow` | EXPLOITATION/TRIBUTE/WAGES/CLIENT_STATE (edge) | `Currency` (`relationship.py:92`) | `[0,∞)` | **edge attribute, unbounded real** |
| `subsidy_cap` | CLIENT_STATE (edge) | `Currency` (`relationship.py:110`) | `[0,∞)` | **edge attribute, read-only here** |
| `v` | TERRITORY | **not a declared model field** — sanctioned extra (`vocabulary/registry.py:224`, "the county-grain variable-capital vector") | unconstrained `float` | graph-only extra (Vol2CirculationStep only) |
| `la_production` | graph attribute (`dict[node_id, float]`) | plain `dict`, no Pydantic wrapper | `[0,∞)` per value implied | **graph-scoped state**, not a node field |
| `opposition_states` | graph attribute (`dict[str, dict]`) | plain `dict` | `.capital_labor.gap` `[0,1]` implied | **graph-scoped state**, written by a LATER-position system, read one-tick-stale |
| `economy` | graph attribute (`GlobalEconomy.model_dump()`) | `GlobalEconomy` (frozen Pydantic, `imperial_rent_pool: Currency`, `current_super_wage_rate: Coefficient`, `current_repression_level: Probability`) | pool `[0,∞)`, rate `[0.05,0.35]` by clamp (not by the field's own domain), repression `[0,1]` | **graph-scoped singleton state**, self-round-tripped |
| `decision` (formula return) | — | plain `str` (`BourgeoisieDecision`'s class attributes are bare strings, not `StrEnum` members) | 5-valued closed set `{no_change,bribery,austerity,iron_fist,crisis}` | **enum-shaped discriminant implemented as a string constant class**, not a real Python `Enum` |
| all `economy.*` coefficient defines | — | `float` (Pydantic `Field(ge=..., le=...)`) | see §2(e) per-phase | mix of `[0,1]` coefficients and 3 unbounded-above reals (`superwage_multiplier`, `initial_rent_pool`, `negligible_rent`/`negligible_subsidy`) |

**Graph-scoped-state flag — the dominant, previously-unnamed-for-this-system gap.** Three separate
values this system reads or writes (`la_production`, `opposition_states`, `economy`) live on
`graph.graph[...]` via `get_graph_attr`/`set_graph_attr`, not on any node. `GraphSubstrate`
(`rust/crates/babylon-graph/src/substrate.rs`, full 249-line trait read) has **no graph-level
attribute method at all** — only `node_attribute`/`update_node` (per-node) and the dyadic/hyperedge
query surface. `bsl-language.rst`'s own Draft Ruling Register names the resolution pattern (D39,
§3.6): *"Graph-scope state is ordinary node state on a carrier `NodeType` whose ceiling is 1 — no
new grammar, no second storage class. Adding a carrier type is amendment territory. The `:global`/
`update-global` route is recorded as **rejected**."* Paired with D40 (`the` names the unique node of
a `:ceiling 1` type). **No landed pack or `.bscn` scenario uses a `:ceiling 1` carrier type**
(grep-confirmed zero hits for `:ceiling` across `rust/crates/babylon-tick/content/`) — the pattern
is spec-only, untested in practice, and instantiating it for `economy`/`opposition_states`/
`la_production` requires minting a NEW carrier `NodeType`, which the spec's own text calls
"amendment territory," not a pure port-time content decision.

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`) except the one flagged libm call below. Shapes,
in execution order across the 5 phases:

1. **Rate conversion (repeated 3x, once per rate):** `annual / weeks_per_year`
   (economic.py:255,421-423; also inside `phi_distribution.py:91`) — plain division.
2. **TRPF linear decay with floor:** `trpf_multiplier = max(trpf_floor, 1.0 - (trpf_coefficient *
   tick))` (economic.py:261) — the bare `1.0` literal is the same "no bare non-integer literal"
   BSL-parser problem the Territory inventory flagged; here it also multiplies against `tick` (an
   `int` context value) inside a `float` expression, an implicit int→float promotion Python performs
   silently.
3. **Multiplicative extraction with a `(1 - x)` complement term:** `rent = extraction_efficiency *
   worker_wealth * (1.0 - consciousness)` (economic.py:289) — same bare-`1.0` shape.
4. **Cap-by-min:** `rent = min(rent, worker_wealth)` (economic.py:292) — appears **7 more times**
   across the file in the identical shape (`min(super_wage_bonus, ...)`,`min(max_bonus,
   available_pool)`, `min(total_wages, bourgeoisie_wealth)`, `min(subsidy_cap, ...)` ×2 nested,
   `min(..., available_pool)`, `min(1.0, target_repression + repression_boost)`) — **every clamp in
   this file is hand-written `min`/`max` nesting**; `SystemBase._write_clamped` (the Territory
   system's Phase-1 idiom) is used **zero times** here. Unlike Territory's two-shape inconsistency,
   this file is internally consistent (100% hand-nested), but every instance still needs the
   landed-pack nested-`if` transcription (no scalar `min`/`max` in the BSL grammar).
5. **PPP multiplier, a `1 + (a*b*c)` form:** `ppp_multiplier = 1.0 + (extraction_efficiency *
   superwage_multiplier * superwage_ppp_impact)` (economic.py:432) — three-factor product plus
   bare-`1.0` offset.
6. **Sequential, order-dependent pool debit inside one edge loop (Phase 3):** `available_pool =
   tick_context["current_pool"]` is re-read at the END of each WAGES-edge iteration
   (economic.py:544) after that same iteration DEBITS `current_pool` — the very next edge's
   `super_wage_bonus = min(max_bonus, available_pool)` (economic.py:458) sees the PRIOR edge's
   debit. This is the **opposite** structural shape from Territory's Phase-3 spillover (which the
   Territory inventory flagged as "a genuinely *favorable* structural match" to BSL's per-position
   same-pre-state semantics because it collects into a dict before applying). Here, iteration order
   over `graph.query_edges(edge_type=EdgeType.WAGES)` is LOAD-BEARING for the result — edge N's
   wage payment depends on edges 1..N-1's payments THIS SAME PHASE, within THIS SAME position. This
   is incompatible with "one rule position, one shared pre-state" (the CURRENT BSL surface's own
   stated constraint: "TWO rules at one anchor position do NOT yet share pre-state, D-row Q14/D116,
   open") even setting that open item aside — it needs *sequential*, order-committed intra-position
   state, which no landed pack's per-position semantics currently expresses at all (every landed
   pack's rules are position-independent of each other's same-tick output by construction). The
   exact same shape recurs in Phase 4's `available_pool = tick_context["current_pool"]`
   (economic.py:627-628).
7. **Ratio with a manual zero-guard, no `EPSILON`:** `stability_ratio = p_revolution /
   p_acquiescence if p_acquiescence > 0 else (1.0 if p_revolution > 0 else 0.0)`
   (economic.py:609-613) — an `if`/`else` guard, not a `+EPSILON` denominator pattern; portable as
   nested `if`.
8. **`math.exp` — the one libm transcendental, LIVE on canonical scenarios (Phase 4 only):**
   `calculate_acquiescence_probability` (`survival_calculus.py:41-43`): `exponent = -steepness_k *
   (wealth - subsistence_threshold)`, clamped `max(-500, min(500, exponent))` (an overflow guard,
   not a domain clamp), then `1.0 / (1.0 + math.exp(exponent))` — a logistic sigmoid. **Flag: libm
   nondeterminism hazard** (cross-implementation `exp` does not reproduce bit-for-bit) **AND** an
   ADR172/173 "no imposed functional forms" PORT-QUESTION — the CURRENT BSL surface confirms `exp`
   is a declarable intrinsic (`declarations.rs:110`), so this is not a *language* blocker, but the
   Constitution's own ruling (`ai/bsl-architecture-standard.md §3.2`) says P(S|A) must EMERGE from
   within-class wealth dispersion in the Rust/BSL engine, not be stipulated as a mechanic — this
   specific formula is exactly the stipulated-sigmoid shape the ruling names, so a literal transcription
   would be porting a shape the Constitution has already marked for replacement, not merely a hard
   port.
9. **Division with an `EPSILON` denominator guard (no libm):** `calculate_revolution_probability`
   (`survival_calculus.py:63-65`): early-return `0.0` if `cohesion <= 0`, else `min(1.0,
   cohesion / (repression + EPSILON))` — plain division, `EPSILON = 1e-9` from
   `GameDefines().precision.epsilon` (a **module-level side effect** at import time,
   `survival_calculus.py:16-18` — `_DEFINES = GameDefines()` instantiates the full defines tree
   once per process, independent of the `services` container this system otherwise reads defines
   through everywhere else — a minor architectural inconsistency worth noting for the port, not a
   blocker).
10. **Pure branching decision matrix, no arithmetic beyond comparisons:**
    `calculate_bourgeoisie_decision` (`dynamic_balance.py:82-118`) — 4-way nested `if`/`elif` over
    `<`/`>=` threshold comparisons against `pool_ratio`/`aggregate_tension`; trivially portable.
11. **Background geometric decay (tail):** `current_pool * (1.0 - decay_rate)` (economic.py:829) —
    same bare-`1.0` shape as items 2-3, applied to the graph-scoped pool (§3's dominant blocker, not
    a float-op blocker in itself).
12. **No Real→Int demotions anywhere in `economic.py`** — unlike Territory, this system never casts
    a continuous quantity to `int` (population/displacement math is out of scope here; the one
    `int(tick)` call, `economic.py:150`, casts an already-integral tick counter, not a continuous
    value).

**libm hazard summary: exactly one call site (`math.exp`, item 8), reachable only through Phase 4,
LIVE on every canonical `imperial_circuit`/`starvation`/`glut`/`fascist_bifurcation` scenario** (§5
confirms Phase 4 is not dormant).

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 9.0** (`economic.py:37`), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:329-362`): `... SolidaritySystem (8.0) → ImperialRentSystem (9.0) →
  TransportSystem (9.5, default-OFF) → DispossessionEventSystem (10.0) → ...`.
- **Reads from same-tick prior systems:**
  - `SOCIAL_CLASS.wealth` — already mutated THIS tick by `VitalitySystem` @1.0 (subsistence burn,
    `vitality.py:121-122`) and `ProductionSystem` @3.0 (value production added to wealth,
    `production.py:181,192,198`) before ImperialRent reads it. A genuine same-tick, ordered
    dependency chain (Vitality → Production → ImperialRent, all on the same `wealth` field).
  - `la_production` (graph attribute) — written fresh THIS tick by `ProductionSystem` @3.0
    (`production.py:207`, `graph.set_graph_attr("la_production", la_production)`); ImperialRent's
    Phase 3 is its only consumer (grep-confirmed, `economic.py:438,453` are the only
    `la_production` references outside `production.py` itself).
  - `SOCIAL_CLASS.ideology` — read by Phase 1 via `class_consciousness_from_node`; written by
    `ConsciousnessSystem` (a CONSEQUENCE-phase system, position ~16.x, AFTER ImperialRent this same
    tick) — so Phase 1 reads **last tick's** consciousness value (or the scenario seed on tick 0),
    the same one-tick-lag pattern the system's own docstring documents explicitly for
    `opposition_states` (economic.py:755-763).
  - `opposition_states` (graph attribute) — written by `ContradictionSystem` @18 (CONSEQUENCE
    phase, `contradiction.py:90`), read one-tick-stale by Phase 5 (documented in-code, see §2 Phase
    5(b)); `0.0` on tick 0.
- **Writes consumed downstream:**
  - `SOCIAL_CLASS.wealth` — read by essentially every downstream System (too broad to enumerate
    exhaustively; representative: `reserve_army.py` @5.0 reads it for its own accounting, though
    that position is actually BEFORE ImperialRent — the load-bearing downstream readers are all
    CONSEQUENCE-phase: `survival.py`, `struggle.py`, `allegiance.py`, `electoral.py`, `metabolism.py`,
    among others).
  - `SOCIAL_CLASS.effective_wealth`/`unearned_increment` — read by
    `domain/economics/melt/filtration.py`, `domain/economics/melt/unified_classifier.py` (grep-
    confirmed, the only readers outside `economic.py` itself) — the Vol III MELT/money-scissors
    estate.
  - `SOCIAL_CLASS.w_paid`/`v_produced` — read by `ideology.py`, `contradiction.py`,
    `market_scissors.py`, `domain/dialectics/instances/value_form.py`,
    `domain/dialectics/instances/catalog.py` — "the wage⇄value counit-defect pair read by the
    value-form `wage`/`imperial` oppositions" (the system's own docstring, economic.py:521-528,
    confirmed by the grep).
  - `SOCIAL_CLASS.repression_faced` — read downstream by `electoral.py`, `ooda.py`, `survival.py`,
    `struggle.py`, `ideology.py`.
  - Edge `value_flow` on EXPLOITATION/TRIBUTE/WAGES/CLIENT_STATE — read by
    `tools/regression_scenarios.py`'s own coverage-gap documentation as a channel other systems do
    NOT read further (`regression_scenarios.py:2000-2006`, "value_flow is written only on
    EXPLOITATION/TRIBUTE/WAGES/CLIENT_STATE edges ... never receive it in the current model scope");
    it is however part of the `graph_content_hash` byte-gate (§7).
  - `economy` graph attribute (`imperial_rent_pool`/`current_super_wage_rate`/
    `current_repression_level`) — **read by no other System anywhere in `src/`** (grep-confirmed,
    `get_graph_attr("economy")` appears only in `economic.py` itself and at the
    `WorldState↔graph` seed/round-trip boundary, `world_state.py:705,924`). A fully self-contained
    tick-to-tick round-trip with zero engine-internal downstream consumers.
- **Context/service usage with no BSL equivalent:**
  - `context["boundary_flow_register"]`/`["session_id"]`/`["external_nodes_phi"]`/
    `["county_exposure_by_external"]`/`["vol2_step"]`/`["simulated_year"]` — all six keys are set
    **only** by `game/session.py:1583-1589`, gated on `self._trade is not None`
    (a Program-26 international-trade session feature). **Never** set by `tools/regression_test.py`
    (`persistent_context = {}` at `regression_test.py:1023`, never mutated with any of these keys)
    — confirmed by an explicit repo-wide grep for each key name excluding tests. Both sub-stages
    (5b, 5c) are **dormant on every canonical `qa:regression` scenario**, live only in the
    interactive game-session tick-advance path.
  - `services.boundary_register` (the L-RECEIPTS provenance write inside Phase 1) — defaults to
    `None` on `ServiceContainer` (`engine/services.py:269`); same dormancy conclusion.
  - `services.formulas.get(...)` (`FormulaRegistry`) — raises `KeyError` if a formula name is
    unregistered (`formula_registry.py:58`), never silently substitutes.
- **DORMANCY on canonical scenarios — checked against `tools/regression_scenarios.py` +
  `src/babylon/engine/scenarios/_legacy.py`:**
  - **Phases 1-3 (Extraction/Tribute/Wages) are LIVE**, not dormant. `create_imperial_circuit_
    scenario` (`_legacy.py:255-544`) seeds all four edge types this system needs — EXPLOITATION
    (`_legacy.py:407-414`), TRIBUTE (`:417-424`), WAGES (`:427-434`), CLIENT_STATE (`:437-445`,
    **with a nonzero `subsidy_cap=10.0`**, `_legacy.py:444`) — and this factory backs **4 of the ~12
    canonical `SCENARIOS` entries** (`imperial_circuit`, `starvation`, `glut`,
    `fascist_bifurcation`, `regression_scenarios.py:38-70`); `two_node` uses
    `create_two_node_scenario` (EXPLOITATION only, per its own docstring). All seeded classes have
    nonzero `wealth` (`_legacy.py:315,330,345,360`), so Phases 1-3's core edge loops execute with
    real transfers on every one of these 4+1 canonical scenarios.
  - **Phase 4 (Subsidy) is LIVE too** — the CLIENT_STATE edge's `subsidy_cap=10.0` is nonzero and
    the seeded comprador (`COMPRADOR_ID`, organization=0.5, repression_faced=`repression_level*0.6`,
    subsistence_threshold=0.2, `_legacy.py:325-337`) gives `calculate_acquiescence_probability` a
    real `math.exp` call on every tick of every canonical scenario using this factory — **the libm
    hazard (§4 item 8) is confirmed LIVE, not merely theoretical.**
  - **Phase 5 (Decision) always runs** — it has no edge-loop gate, only the graph-attribute reads
    described above.
  - **Sub-stages 5b/5c (Φ-distribution, Vol II Circulation) are DORMANT on every canonical
    scenario** — confirmed above; live only via `game/session.py`.
  - **The L-RECEIPTS boundary-register write inside Phase 1 is DORMANT on every canonical
    scenario** — `services.boundary_register` defaults `None` and `tools/regression_test.py` never
    sets it.
  - **The `economy` graph attribute's own downstream (zero readers) means its "liveness" is entirely
    internal to this system** — real state, real tick-to-tick effect on wage_rate/repression via
    Phase 5, but invisible to every other System.

## 6. BLOCKER ASSESSMENT

Adjudicated against the CURRENT BSL surface stated in this task (Query lane Slice 1 landed; Slices
2-4 not built — no edge-attribute reads; enum fields landed via `deffield ... enum`; `deffield` type
vocabulary has no `currency`-storage path that survives scenario load; `exp`/`log`/`floor`
declarable; no-imposed-functional-forms ruling; events unpinnable; two rules at one position don't
share pre-state) **and against the `structural_verbs.rs`/`substrate.rs` read performed for this
inventory**, which surfaces one blocker not named in the task's CURRENT BSL surface summary: edge
attribute *storage* itself.

| Computation | Verdict | Detail |
|---|---|---|
| Phase 1 Extraction — edge `value_flow` write (economic.py:300-302) | **BLOCKED — no edge-attribute-write substrate lane** | `update-edge` is parsed (D35) but refused at execution: `GraphSubstrate` keys every edge to exactly one `f64 strength` field and gives it no other storage (`structural_verbs.rs:371-382`, citing D35/D65). This is independent of and prior to any Slice-2/3/4 gap — it is a `GraphSubstrate`-level absence, not merely an unevaluated query head. Blocks the write for ALL FOUR edge types this system touches. |
| Phase 1 Extraction — node reads/writes (wealth transfer, `role`/`ideology` reads) | **PORTABLE WITH D-RECORD** (contingent on the edge-write blocker above being resolved first) | `worker.wealth`/`target.wealth` are ordinary `:field` reads/writes; `role` (`SocialRole`, 8-valued `StrEnum`) is expressible via the landed `deffield ... enum` lane (ADR195/196); `ideology.class_consciousness` needs a content-modeling decision (flatten the nested `IdeologicalProfile` to a top-level `:field`, or a D-record deferring the multi-dimensional profile to a single scalar) — no existing precedent names this specific nested-object-field shape. |
| Phase 1 — L-RECEIPTS boundary-register write (economic.py:311-321) | **NOT-A-PACK** | Writes to a `BoundaryFlowRegister` (Postgres-facing), never to the graph substrate at all — outside BSL's scope by construction, not merely dormant. Confirmed dormant on every canonical scenario in any case (§5). |
| Phase 2 Tribute — edge `value_flow` write (economic.py:390-392) | **BLOCKED — no edge-attribute-write substrate lane** | Same D35/D65 gap as Phase 1. |
| Phase 2 Tribute — node reads/writes | **PORTABLE WITH D-RECORD** (same contingency as Phase 1) | Plain wealth arithmetic, `role` enum comparison; no formula calls, no libm. |
| Phase 3 Wages — edge `value_flow` write (economic.py:534-536) | **BLOCKED — no edge-attribute-write substrate lane** | Same D35/D65 gap. |
| Phase 3 Wages — `la_production` graph-attribute read (economic.py:438,453) | **BLOCKED — graph-scoped state, no substrate representation** | `la_production` is a `dict[node_id, float]` on `graph.graph[...]`, not a node field; `GraphSubstrate` has no graph-level attribute method (confirmed by the full 249-line trait read). The D39 carrier-`NodeType` pattern is the named resolution path but is unexercised (§3) and would need `ProductionSystem`'s own port to co-design the carrier shape — not a decision this system's port can make unilaterally. |
| Phase 3 Wages — sequential intra-phase pool accumulator (economic.py:456-458,540-544) | **BLOCKED — no per-position sequential-accumulator semantics** | Edge N's payment depends on edges 1..N-1's payments THIS SAME position/phase (§4 item 6) — a stronger requirement than the open "two rules at one position share pre-state" gap (D116/Q14): this needs *committed, order-dependent* intra-position state, which no landed pack's semantics expresses at all. A reformulation (e.g., a two-pass collect-then-ration split, mirroring Territory's Phase-3 spillover) is possible in principle but would be a genuine algorithmic deviation from the frozen system, not a value-preserving reformulation — flag for Director/architecture review before attempting. |
| Phase 4 Subsidy — `subsidy_cap` edge-attribute read (economic.py:593) | **BLOCKED — Slice 2 (edge-attribute reads)** | Named explicitly in the CURRENT BSL surface as not built. |
| Phase 4 Subsidy — `calculate_acquiescence_probability` sigmoid (`survival_calculus.py:41-43`) | **PORT-QUESTION — reserved theory line, not a mechanical port** | `exp` is a declarable intrinsic (mechanically portable), but ADR172 ruling 5 / ADR173 rule this exact stipulated-sigmoid shape must EMERGE from within-class wealth dispersion, not be hardcoded — this is a Director/theory decision, not an engineering one. Flagged **RESERVED-LINE**: the correct P(S|A) functional form for the Rust/BSL engine is an ideological/theoretical modeling choice, not this inventory's to propose. |
| Phase 4 Subsidy — `calculate_revolution_probability`, decision-trigger `if`/`else` guard | **PORTABLE WITH D-RECORD** | Plain division with an `EPSILON`-style guard (already the `+EPSILON` idiom other landed packs use) — mechanically portable once the sigmoid question above is resolved and the edge-read/edge-write blockers above are cleared. |
| Phase 4 Subsidy — edge `value_flow`/`repression_faced` writes | **BLOCKED — no edge-attribute-write substrate lane** (value_flow) / **PORTABLE** (repression_faced, ordinary node field) | Same D35/D65 gap for the edge half. |
| Phase 5 Decision — `calculate_bourgeoisie_decision` matrix | **PORTABLE WITH D-RECORD** | Pure nested-`if` over threshold comparisons, no libm, no edge state — mechanically the most portable computation in the file. `BourgeoisieDecision`'s 5-valued string-constant "enum" needs the same `deffield ... enum` treatment as `role`/`TerritoryType` (precedent already landed). |
| Phase 5 Decision — `opposition_states` graph-attribute read | **BLOCKED — graph-scoped state, no substrate representation** (same class as `la_production`) | Additionally cross-system: `opposition_states` is `ContradictionSystem`'s own output (a system that has not itself been ported), so even the D39 carrier-type pattern would need to be co-designed with THAT system's port, not this one's. |
| Tail `_save_economy` — `economy` graph-attribute round-trip (economic.py:831-836) | **BLOCKED — graph-scoped singleton state, no substrate representation** | The dominant, system-defining blocker (§3): `GlobalEconomy`'s three fields have no per-node home. D39/D40 name the pattern (`:ceiling 1` carrier `NodeType` + `the`) but it is unexercised anywhere on `dev` and minting the carrier type is the spec's own "amendment territory." Distinct from — and structurally harder than — Metabolism's D-4 (`(domain :graph)` aggregate-and-emit, which at least has a load-time-implemented-but-not-executed mechanism to point at); this is not an aggregate over existing per-node state at all, it is an independent accumulator with no node analog whatsoever. |
| Sub-stage 5b Φ-distribution (whole) | **NOT-A-PACK** | Touches zero graph state; writes exclusively to `BoundaryFlowRegister` (Postgres-facing persistence). Outside BSL's scope by construction — not a query/effect-lane gap, a category mismatch. Also confirmed dormant on every canonical scenario. |
| Sub-stage 5c Vol II Circulation (whole) | **NOT-A-PACK** | `scipy.sparse`/`numpy` matrix-vector algebra over an externally-loaded LODES OD matrix, via a `ScaleAdjunction` hex↔county binding — a fundamentally different computational paradigm from BSL's per-rule, per-node-type evaluation model; no rule-language mapping exists even in principle for a sparse linear-algebra pass over an externally loaded matrix. Also confirmed dormant on every canonical scenario. |
| `SocialRole`/`role` enum reads throughout | **PORTABLE** | `deffield ... enum SocialRole` with 8 members, `:field` binding, `=` comparison against `CORE_BOURGEOISIE` — squarely inside the LANDED enum-field lane (ADR195/ADR196), no D-record needed beyond declaring the 8-member enum itself. |
| `w_paid`/`v_produced`/`v` sanctioned graph-only extras | **PORTABLE WITH D-RECORD** | Ordinary `:field` writes once the model decides these are first-class `deffield`s rather than Python-only "extra" attributes — a straightforward content-modeling normalization, no engine gap. |

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_economic_subsidy.py` | 848 | **Primary conformance-oracle candidate, Phase 4.** Exhaustive coverage of `_process_subsidy_phase`: trigger-ratio branches (both `p_acquiescence>0` and the `=0` crisis fallback), subsidy-amount capping (cap/tribute/wealth/pool, in order), repression-boost clamp, event payload shape. |
| `tests/unit/engine/systems/test_economic_decision.py` | 704 | **Primary conformance-oracle candidate, Phase 5.** Exhaustive coverage of `_process_decision_phase`: all 5 decision branches, wage/repression clamping, `ECONOMIC_CRISIS` gating. Source citation in its own docstring: `economic.py:462-538` (line numbers now stale post-refactor but the method boundary is unambiguous). |
| `tests/integration/mechanics/test_dynamic_balance.py` | 515 | **Primary conformance-oracle candidate, whole-pipeline.** Multi-tick integration tests via the module-level `step()` convenience function (same harness `tools/regression_test.py` uses) — "The Drain" (pool depletion), "The Crash" (`ECONOMIC_CRISIS` firing), "Policy Switch" (tension→repression routing). The closest thing to an end-to-end behavioral contract for this system already in the estate. |
| `tests/unit/engine/systems/test_economic_events.py` | 446 | Event-emission contract: `SURPLUS_EXTRACTION` gating on `rent > negligible_rent`, payload shape, PPP-model event fields. |
| `tests/unit/engine/systems/test_economic_wages.py` | 266 | Phase 3 unit coverage: productivity+bonus composition, pool-cap interaction, PPP multiplier arithmetic. |
| `tests/unit/engine/systems/test_economic_tribute.py` | 219 | Phase 2 unit coverage: comprador-cut arithmetic, tribute-inflow tracking. |
| `tests/unit/engine/systems/test_economic_weekly.py` | 249 | Annual→per-tick rate conversion contract (the `/weeks_per_year` shape, §4 item 1) — a genuine behavioral-contract candidate for that specific transcription detail. |
| `tests/unit/engine/systems/test_economic_accounting.py` | 114 | `w_paid`/`v_produced` bookkeeping-write contract (Phase D4 exposure) — directly relevant to the sanctioned-extras finding in §3/§6. |
| `tests/unit/engine/systems/test_superwage_crisis.py` | 210 | `SUPERWAGE_CRISIS` event-trigger contract, independent of the active/inactive skip ordering noted in §2 Phase 3(f). |
| `tests/unit/engine/systems/test_phi_wiring.py` | 222 | Sub-stage 5b wiring contract — explicitly "No DB or graph needed... pure context+register interaction" (its own docstring) — confirms by construction that this sub-stage never touches graph state, corroborating the NOT-A-PACK verdict. |
| `tests/unit/engine/systems/test_vol2_wiring.py` | 92 | Sub-stage 5c wiring contract, stub-`step()`-based — same corroboration for the Vol II NOT-A-PACK verdict (though the real `Vol2CirculationStep` itself IS graph-touching; this test only exercises the wiring gate). |
| `tests/unit/economics/circulation/test_vol2_circulation_step.py` | 279 | The real `Vol2CirculationStep.step` unit coverage — matrix algebra, conservation-residual checking, `ScaleAdjunction` binding — schema/algorithm-level, not a BSL conformance candidate (NOT-A-PACK per §6). |
| `tests/unit/engine/observers/test_economic_monitor.py` | 347 | `EconomyMonitor` observer — detects >20% pool drops for AI-narrative logging. Observer/narrative layer, not engine conformance. |
| `tests/integration/mechanics/test_proletarian_internationalism.py` | 396 | Tests `SolidaritySystem`'s SOLIDARITY-edge transmission end-to-end through the full engine — `ImperialRentSystem` runs incidentally as part of the tick loop but this file's own subject is Solidarity, not this system. |
| `tests/integration/economics/tick/test_imperial_rent_pipeline.py`, `test_imperial_rent_real_wiring.py`, `test_imperial_rent_calibration.py`, `test_imperial_rent_perf.py` | 458, 111, 66, 98 | **NOT conformance oracles for this system** — all four target `babylon.domain.economics.tick.system.imperial_rent.compute()`, a same-named but architecturally distinct Leontief Φ_hour pipeline owned by `TickDynamicsSystem` (confirmed by each file's own docstring and import block, §1). Flagged here only to prevent a future reader from misattributing them. |

**qa:regression byte-gate coverage.** `tools/regression_test.py::graph_content_hash`
(`regression_test.py:924-964`) hashes every node/edge attribute of the `WorldState→graph`
projection (same mechanism the Territory inventory found) — so `wealth`/`effective_wealth`/
`unearned_increment`/`ppp_multiplier`/`w_paid`/`v_produced`/`repression_faced`/edge `value_flow`
changes on any of the `imperial_circuit`/`starvation`/`glut`/`fascist_bifurcation`/`two_node`
canonical scenarios are caught by the byte-identical hash gate. **The `economy` graph attribute is
explicitly, deliberately excluded** — the function's own docstring states it: *"Graph metadata
(`g.graph`: economy, event log, opposition states) is also excluded, because the spec's field set
is nodes/edges/actions; four scenarios that differ only in economy therefore share a tick-0 hash
and diverge once they tick"* (`regression_test.py:939-943`). So `economy`'s pool/wage-rate/
repression-level state is **invisible to the byte gate entirely** — the same graph-scoped-state
absence that blocks its BSL port (§3/§6) already means it is unwitnessed by the frozen system's own
strongest conformance instrument, not merely by any future pack. A Phase-5 conformance oracle for
this system will need a hand-built fixture asserting on `tick_context`/the returned `GlobalEconomy`
directly (as `test_dynamic_balance.py` and `test_economic_decision.py` already do), never the
byte-hash gate.

---

## Adjudication (2026-08-12)

Adjudicated against the dev tree at `9324482f`. Three corrections, four confirmations.

1. **CONFIRMATION — the primary blocker is exactly as stated, verbatim at the source.**
   `structural_verbs.rs:387-398` refuses `update-edge`/`update-hyperedge` with *"({verb} …) has
   no substrate storage: GraphSubstrate keys an edge to one f64 strength and gives a hyperedge no
   attributes at all. Widening that state widens the canonical state_hash field set, which is a
   declared Phase-2/substrate decision (Constitution III.7), never a silently-dropped write"* —
   the module doc names D35/D65 at `structural_verbs.rs:16`. Independently confirmed against the
   trait itself: the full 249-line `rust/crates/babylon-graph/src/substrate.rs` exposes
   `update_node(id, attribute, value: f64)` (:133) and `node_attribute(id, attribute)` (:142) —
   **per-NODE only** — plus `add_edge(edge_type, from, to, strength: f64)` (:111-117). There is no
   edge-attribute accessor of any kind. The reader's central claim, that this is a
   `GraphSubstrate`-level absence prior to and independent of Slice 2, is correct.

2. **CONFIRMATION — graph-scoped state, and the named resolution path is doubly unavailable.**
   `la_production` (`production.py:207` → `economic.py:438,453`), `economy`
   (`economic.py:796,836` — the only two `*_graph_attr("economy")` sites anywhere in `src/`), and
   `opposition_states` all live on `graph.graph[...]` with no substrate home. Beyond the reader's
   "no landed pack uses `:ceiling`": **no landed content declares a `manifest` at all** — a grep
   for `manifest` across `rust/crates/babylon-tick/content/` returns zero hits, and the manifest
   IS the construct that carries `:ceiling` (`rust/crates/babylon-bsl/src/manifest.rs`). And D40's
   accessor is itself blocked: `the` sits in `UNSERVED_EXPRESSION_HEADS` tagged `"slice 2"`
   (`evaluator.rs:505`); its singleton guard exists only as a LOAD-time check — `E-LOAD-043`,
   *"(the {row}) needs a declared :ceiling of exactly 1"* (`manifest.rs:51-53, 100-104`) — and
   never evaluates. So the D39/D40 route is unexercised, un-evaluable, AND amendment-gated.

3. **CONFIRMATION — the sequential intra-phase pool accumulator (§4 item 6) is definitively
   inexpressible, on stronger grounds than the reader gives.** The reader argues from the open
   Q14/D116 row; the actual bar is higher and already settled: `tick.rs:41-52` records Task 12
   (P27 Phase 2, 2026-08-11) as **landed**, replacing in-place mutation with a two-pass `run_tick`
   that collects every subject's writes against the SAME pre-tick graph before applying any, per
   §4.2 chapter C4 (*"all firings of one rule observe the same pre-state"*). Order-committed
   intra-position state is therefore not merely unbuilt — it is ruled out by the implemented
   semantics. The reader's "flag for Director/architecture review before attempting" is the right
   disposition; the citation should be Task 12 / C4, not Q14.

4. **CONFIRMATION — the acquiescence sigmoid is a reserved line, and the engine already
   mechanizes half of that ruling.** `math.exp` confirmed live at `formulas/survival_calculus.py:43`
   inside `1.0 / (1.0 + math.exp(exponent))`, reachable via Phase 4 on every
   `create_imperial_circuit_scenario`-backed canonical scenario (`CLIENT_STATE` with
   `subsidy_cap=10.0` seeded at `engine/scenarios/_legacy.py:440-444`). `exp` remains declarable
   (`declarations.rs:110`, `DECLARABLE_INTRINSICS = ["exp","log","floor"]`) — so the reader is
   right that this is not a language blocker. Adding force to the RESERVED-LINE flag:
   `PROHIBITED_INTRINSIC_NAMES = ["sigmoid"]` (`declarations.rs:117`), whose own comment reads
   *"`sigmoid` would hand content the exact mechanism ADR172 ruling 5 forbids, pre-packaged and
   named; it is the one part of the doctrine gate that can be made mechanical, so it is."* The
   PORT-QUESTION verdict is correct and is now backed by a mechanical gate, not only by doctrine.

5. **CORRECTION — §6's `SocialRole`/`role` row ("**PORTABLE** … no D-record needed beyond
   declaring the 8-member enum itself") is wrong, and it contradicts this same batch's
   `DecompositionSystem` inventory, which reaches the correct conclusion on the identical read
   shape.** Every `role` read in this system is of a **foreign** node reached through an edge —
   `target_role = target_attrs.get("role")` at `economic.py:324-327` is the EXPLOITATION edge's
   *target*, never `self`. A foreign-node field read is `field-of`, and `field-of` is **REFUSED**
   for `:enum-type`-declared fields (D102), enforced at LOAD by `check_no_field_of_on_enum_field`
   (`rust/crates/babylon-bsl/src/typecheck.rs:246-280`, wired at `rule_pipeline.rs:297`) with the
   message *"… not extended to enum-declared fields (§2.13, D102)"*; spec text at
   `docs/reference/bsl-language.rst:2274-2284`, register row at `:5681-5693`. Declaring
   `social-class/role` as `deffield … enum SocialRole` would therefore **break** this read rather
   than enable it. Correct row: **PORTABLE WITH D-RECORD** via the int-ordinal encoding — the
   convention `content/rules/lifecycle.bsl` already uses live, and the one the Decomposition
   inventory's `_find_entity_by_role` row argues for at length. Same correction applies to Phase
   2's and Phase 5's `role` comparisons.

6. **CORRECTION — Phase 4's `subsidy_cap` row attributes the whole blocker to Slice 2; the read
   half hits D35/D65 too.** Slice 2 supplies the *accessor grammar* (`edges`/`edge-between`, and
   `field-of` over the `EdgeRef` they produce). It supplies no *store*: `GraphSubstrate`'s only
   attribute reader is `node_attribute` (`substrate.rs:142`), and an edge's entire state is the
   one `f64 strength` `add_edge` takes (`substrate.rs:111-117`; the implicit `<edge-type>/strength`
   field, D32, `declarations.rs:13, 317-320`). `subsidy_cap` is a SECOND edge attribute alongside
   `value_flow` on the same `CLIENT_STATE` edge, so it needs the same storage widening the
   `value_flow` write rows already name. Re-file as **BLOCKED — Slice 2 (accessor) AND D35/D65
   (storage)**; landing slice 2 alone does not clear this read.

7. **CORRECTION — every `active` gate in the file (`source.active`/`target.active`, all five
   phases) carries an unnamed encoding D-record, and no boolean is readable as a `Value::Bool`.**
   `bind_field_value` returns `Value::Real(stored)` for every non-enum declared type — only the
   `BslType::Enum` branch renders anything else (`tick.rs:312-327`) — and `field_of_node` returns
   `Ok(Value::Real(value))` unconditionally (`evaluator.rs:1281-1291`). Downstream, `as_bool`
   refuses a Real where a `<cond>` is required (`evaluator.rs:1315-1320`), `apply_equality`
   refuses a Real against a `#t`/`#f` literal (*"equality is defined within one lane only"*,
   `evaluator.rs:1620-1628`), and on the write side `numeric_write_value` refuses a `Value::Bool`
   outright — *"cannot store {other:?} as a numeric node attribute"* (`structural_verbs.rs:1231-1233`).
   `bool` IS a legal `deffield` type (`declarations.rs:649`), but declaring it buys a load-time
   range check and nothing at evaluation. Landed content encodes 0/1 and says so
   (`vitality-conformance.bscn:20`). Add to §3's type inventory and to every §6 row whose
   contingency includes an `active` gate.

8. **CONFIRMATION — dormancy, channels and the byte-gate boundary all re-verified.** Phases 1-4
   LIVE on the four `create_imperial_circuit_scenario`-backed canonical scenarios (edge seeding at
   `_legacy.py:407-445`, `subsidy_cap=10.0` at `:444`). Sub-stages 5b/5c dormant. Tick position
   9.0 (`economic.py:37`) between `SolidaritySystem` (8.0) and `TransportSystem` (9.5) per
   `_SYSTEM_CLASSES` (`simulation_engine.py:328-360`), whose sort by `position` IS `_DEFAULT_SYSTEMS`
   (`:376-378`). Both load-bearing channels spot-checked independently: `la_production` written
   `graph.set_graph_attr("la_production", la_production)` (`production.py:207`) and read at
   `economic.py:438,453` — the only two references outside `production.py`; `w_paid`/`v_produced`
   read downstream by `contradiction.py`, `ideology.py`, `market_scissors.py` and
   `formulas/sustained_exploitation.py`. The `graph_content_hash` exclusion is verbatim at
   `tools/regression_test.py:939-943` — *"Graph metadata (`g.graph`: economy, event log,
   opposition states) is also excluded"* — so §7's finding that `economy` is unwitnessed by the
   estate's strongest instrument stands exactly as filed.

**FINAL VERDICT: BLOCKED — UPHELD, with the blocker set widened, not narrowed.** No
edge-attribute-*storage* lane (D35/D65) blocks every phase's `value_flow` write **and** Phase 4's
`subsidy_cap` read (correction 6 — the read is not a Slice-2-only gap); graph-scoped
`GlobalEconomy`/`la_production`/`opposition_states` have no substrate carrier and the D39/D40
route is unexercised, un-evaluable (`the` is slice-2 unserved) and amendment-gated; Phase 3's
sequential intra-phase pool accumulator is ruled out by the landed two-pass `run_tick`, not merely
unbuilt; and Phase 4's acquiescence sigmoid is a reserved ADR172/173 theory question with a
mechanical gate already standing behind it. The one row that must move the OTHER way is the enum
row: `role` is PORTABLE WITH D-RECORD via int-ordinal, not PORTABLE via `deffield … enum`
(correction 5).

**INADEQUATE-COVERAGE NOTE.** §6's preamble states the adjudication ran "against the
`structural_verbs.rs`/`substrate.rs` read performed for this inventory" — those two files only.
That is why the enum row is wrong (the D102 gate lives in `typecheck.rs`/`rule_pipeline.rs`, not
in either file read) and why the boolean lane is unnamed (`tick.rs::bind_field_value`,
`evaluator.rs::as_bool`/`apply_equality`). A re-read must add: `typecheck.rs:246-280` +
`rule_pipeline.rs:297` (D102, for every foreign-node enum read — this system has three),
`tick.rs:312-327` and `evaluator.rs:1274-1292, 1315-1320, 1594-1632` (the read/compare value
lane), `structural_verbs.rs:1196-1234` (the write lane), and `tick.rs:41-52` (Task 12's pre-state
ruling, which re-grounds the §4-item-6 argument). Nothing in the file map's Python-side coverage
is thin — the gap is entirely on the Rust surface the blocker table grades against.
