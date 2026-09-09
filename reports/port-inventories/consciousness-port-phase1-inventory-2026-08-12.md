# ConsciousnessSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `ConsciousnessSystem` (`src/babylon/engine/systems/ideology.py`, 443
lines, tick position 17.0) drifts each `social_class` node's `(class_consciousness,
national_identity, agitation)` triple from material inputs and routes agitation to the
revolutionary or fascist pole by SOLIDARITY-edge presence — the George Jackson bifurcation and
THE GAME LOOP's Φ-disruption mechanic. Two edge-attribute reads (`WAGES.value_flow`,
`SOLIDARITY.solidarity_strength`) are hard-blocked on query-lane Slice 2 (EdgeRef field access,
unbuilt); one dependency function (`sustained_exploitation_magnitude`) calls `math.exp` in a
hand-shaped Gaussian bump that is both a libm-nondeterminism hazard and squarely the kind of
"imposed functional form" ADR172/173 reserves for Director escalation. A genuine, previously
undocumented defect was found: the `material_conditions` dict this system writes is silently
discarded every tick by a stale Pydantic unpack rule that shares its dict key with an unrelated,
older component — the write is dead-on-arrival, not merely unconsumed. Cross-tick memory lives
in `TickContext.persistent_data` (off-graph Python state), not the graph, requiring new carrier
fields to port.

**Verdict: BLOCKED — query-lane Slice 2 (edge-attribute reads) is required for `core_wages` and
`solidarity_pressure`, the two reads that decide bifurcation direction; a large portable-with-
D-record remainder exists but cannot land as a coherent pack ahead of that slice, and the
Gaussian chauvinist-pressure term is a separate port-question row for the Director regardless of
Slice 2's status.**

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/ideology.py` | 443 | **The target.** `ConsciousnessSystem`, one phase, one per-`social_class`-node loop (`step`, lines 115-442). Also declares module-level helpers `_popular_front_suppression` (40-54) and `_get_ideology_profile_from_node` (57-91). |
| `src/babylon/formulas/consciousness_routing.py` | 523 | Spec-043 tensor→consciousness pipeline. ConsciousnessSystem calls exactly 4 of its 8 exported functions: `compute_agitation_delta`, `compute_exploitation_visibility`, `compute_reification_buffer`, `route_agitation_to_ternary`. **NOT called by this system** (called elsewhere or nowhere in production): `normalize_to_simplex`, `assimilation_ratio`, `ideological_contestation`, `apply_fr_gate`. Module-level `_LOG3 = math.log(3.0)` (line 45) executes at import time regardless (a pure constant, only consumed by the uncalled `ideological_contestation`). |
| `src/babylon/formulas/sustained_exploitation.py` | 201 | The sustained wage-value-defect term. ConsciousnessSystem calls only `sustained_exploitation_magnitude` (line 102); the sibling `sustained_exploitation_agitation` (line 61) is exported but NOT called by this system (kept for a separate characterization test per its own docstring, lines 34-43). **Contains the system's one `math.exp` call** (line 198). |
| `src/babylon/formulas/contradiction.py` | 154 | ConsciousnessSystem calls only `calculate_wealth_asymmetry_balance` (line 67), once per node (ideology.py:243-245). `calculate_wealth_asymmetry_gap` and the deprecated `calculate_contradiction_intensity` are declared here but **not called by this system**. |
| `src/babylon/domain/economics/working_day/resolver.py` | 175 | `resolve_working_day_visibility_modifier` (95-121) — the ONE resolver of its two exports ConsciousnessSystem calls (once per tick, ideology.py:136-138); `resolve_absolute_relative_surplus_ratio` (124-176) is a sibling U6 consumer for a different system, **not called here**. Shares `resolve_working_day_state` (52-92) internally. |
| `src/babylon/domain/economics/working_day/classifier.py` | 78 | `DefaultWorkingDayClassifier.compute_visibility_modifier` — invoked transitively via the resolver above. Pure arithmetic (classify + linear interpolation), no transcendentals. |
| `src/babylon/domain/economics/working_day/types.py` | 31 | `WorkingDayState` — frozen Pydantic carrier for the two FRED-derived scalars. Read-only data shape, not itself computation. |
| `src/babylon/domain/economics/working_day/data_sources.py` | 36 | `ProductivityDataSource` Protocol — the interface `services.productivity_data_source` must satisfy. No implementation lives here (see §5 for the wiring/dormancy finding). |
| `src/babylon/config/defines/consciousness.py` | 583 | `ConsciousnessDefines` (47-269) and `SolidarityDefines` (12-45) — every coefficient this system reads. `ContradictionFieldDefines`/`EdgeTransitionDefines`/`BifurcationDefines` also live in this file but are **not read by ConsciousnessSystem** (consumed by ContradictionFieldSystem/EdgeTransitionSystem/the separate Feature-033 bifurcation-topology analysis — see next row). |
| `src/babylon/config/defines/economy_labor.py` (`WorkingDayDefines`, 261-360) | — | The 6 working-day coefficients threaded through the resolver/classifier. |
| `src/babylon/config/defines/survival.py` | — | `default_repression` (37-...) — the `DEFAULT_REPRESSION_FACED` baseline this system subtracts (via `_assembler.py`'s property alias, §2 C5). |
| `src/babylon/config/defines/tunables.py` | — | `weeks_per_year` — consumed transitively by the resolver's year derivation. |
| `src/babylon/data/defines.yaml` | consciousness: 210-229; solidarity: 182-187; working_day: 436-442; survival.default_repression: 167; timescale.weeks_per_year: 374 | Player-editable coefficient values. |
| `src/babylon/models/entities/social_class.py` | 522 | `SocialClass` (172-522), `IdeologicalProfile` (61-152), `MaterialConditionsComponent` (164-169, **the name-collision culprit**, see §5), the `unpack_components_and_convert_legacy` `mode="before"` validator (248-283) that silently discards this system's `material_conditions` write every tick. |
| `src/babylon/models/components/material_conditions.py` | 68 | `MaterialConditionsBuffer` — the shape ConsciousnessSystem actually writes (`agitation`/`exploitation_visibility`/`reification_buffer`), distinct from and colliding with `social_class.py`'s `MaterialConditionsComponent` (only `repression_faced`). Never instantiated by ConsciousnessSystem itself — it writes a raw dict, not this model. |
| `src/babylon/models/world_state.py` | — | `SOCIAL_CLASS_COMPUTED_FIELDS` (59-92, excludes `w_paid`/`v_produced` from reconstruction — confirms they are transient graph-only, not `SocialClass` fields) and `WorldState.from_graph()` (898-1023, line 1023 is the `SocialClass(**entity_data)` call that triggers the `material_conditions` discard every tick). |
| `src/babylon/engine/systems/economic.py` | — | Writes `w_paid`/`v_produced`/`repression_faced`(subsidy path)/WAGES-edge `value_flow` at position 9.0 — the same-tick prior writer for 3 of ConsciousnessSystem's reads (see §5). |
| `src/babylon/engine/systems/struggle.py` | — | Writes `ideology` (class_consciousness boost, national_identity fascist-boost) at position 16.0 — the same-tick IMMEDIATELY-prior writer of the exact dict key ConsciousnessSystem reads and rewrites (see §5). |
| `src/babylon/ooda/action_effects.py` | — | Writes `repression_faced` (POGROM/VIGILANTISM bumps) at position 14.0 (inside OODA) — same-tick prior writer. |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase` — ConsciousnessSystem inherits only the ClassVar/ABC shape; **no `self.` call appears anywhere in `step()`** — `_write_clamped`/`_publish`/other helpers are unused by this system (unlike TerritorySystem). |
| `src/babylon/kernel/system_protocol.py` | 41 | `ContextType`/`System` Protocol. |
| `src/babylon/kernel/tick_partition.py` | 30 | `TickPartition.CONSEQUENCE` — confirms this system's partition. |
| `src/babylon/engine/context.py` | 113 | `TickContext` — `persistent_data: dict[str, Any]` (line 49) is the **off-graph, cross-tick memory carrier** ConsciousnessSystem uses for `previous_wages`/`previous_wealth` (see §5 — no BSL equivalent exists; the graph is the only persistent state a BSL scenario carries). |
| `src/babylon/kernel/graph_protocol.py` | 494 | `query_nodes`/`query_edges`/`get_node`/`update_node`/`get_graph_attr` (350) signatures. |
| `src/babylon/topology/graph.py` | 660-670 (`update_node`) | Confirmed (re-verified against current dev, unchanged from the Territory inventory's citation): plain dict merge, no type coercion or `SnapToGrid` quantization mid-tick. |
| `src/babylon/models/enums/topology.py` | — | `NodeType.SOCIAL_CLASS`/`.ORGANIZATION` (62-63), `EdgeType.SOLIDARITY`/`.WAGES` (100, 104). |

**Not exercised by ideology.py at all (confirmed by import-list read, ideology.py:11-33, and
grep):**
- `src/babylon/formulas/consciousness.py` (112 lines) — `compute_ternary_consciousness`, called
  by `CommunitySystem` (@6.0), a **different, separate consciousness concept** (per-community
  ternary `TernaryConsciousness`, not this system's per-class binary `IdeologicalProfile`).
- `src/babylon/models/entities/consciousness.py` (463 lines) — `TernaryConsciousness`,
  `OrgContribution` — also CommunitySystem's, confirmed by grep (only `community.py` imports it).
- `src/babylon/models/enums/consciousness.py` (93 lines) — `ContradictionType`, `IntensityLevel`,
  `ContradictionCharacter`, `ConsciousnessTendency` — none imported by ideology.py.
- `src/babylon/domain/bifurcation/consciousness.py` — Feature-033's `consciousness_sigmoid_*`
  weighting (the one genuine **sigmoid** in the `Bifurcation*` defines family, `consciousness.py`
  config lines 458-486) is a **separate, standalone bifurcation-topology analysis**
  (`bifurcation_monitor.py`), not part of the tick pipeline and not called by ConsciousnessSystem
  — confirmed by grep: only `engine/bifurcation_monitor.py` and `struggle.py` (a different sibling
  module, `domain.bifurcation.legitimation`) import from `domain.bifurcation`. This is the
  cleanest-cut file-map ambiguity to flag: three different "consciousness" packages exist in this
  tree and only one is this system's.

## 2. COMPUTATION CATALOG (execution order, `step`, ideology.py:115-442)

Unlike TerritorySystem's four phases, this is one phase with a **tick-scoped preamble** (runs
once) followed by a **per-`social_class`-node loop** (`query_nodes(node_type="social_class")`,
ideology.py:200) whose body is one long straight-line computation per node. Sub-computations are
numbered in the order they execute.

### C0 — Working-day visibility modifier (tick-scoped, once; ideology.py:128-138)
- **(a)** Resolve this tick's Ch. 10 (Marx, *Capital* Vol I) working-day regime ONCE for the whole
  tick (not per node) — the wired FRED adapter is national-level and uniform, and no per-class
  county/sector identity exists on `social_class` nodes to honestly vary the call by.
- **(b)** `working_day_modifier = resolve_working_day_visibility_modifier(graph, services,
  context.tick)` (ideology.py:136-138) → internally: `year = base_year + tick //
  weeks_per_year` (resolver.py:88); `state = source.get_working_day_state(...)` (91); if `state`
  is `None`, returns `None`. Else `DefaultWorkingDayClassifier(...).compute_visibility_modifier
  (state)` (resolver.py:120-121) → classify by two threshold comparisons (classifier.py:47-51)
  then either return a fixed visibility or linearly interpolate: `t = (hours -
  relative_hours_threshold) / (absolute_hours_threshold - relative_hours_threshold)`, clamped
  `[0,1]`; `relative_visibility + t * (absolute_visibility - relative_visibility)`
  (classifier.py:76-78).
- **(c) Reads:** graph attr `base_year` (default 2022, resolver.py:44,87); `services.
  productivity_data_source` (an injected `ProductivityDataSource | None`); `services.defines.
  timescale.weeks_per_year`; `services.defines.working_day.*` (6 fields).
- **(d) Writes:** none (returns a local `float | None`, consumed later in C13).
- **(e) Defines:** `timescale.weeks_per_year` (52, `int >= 1`, defines.yaml:374);
  `working_day.absolute_hours_threshold` (45.0, `(0, 168]`, defines.yaml:437);
  `working_day.relative_hours_threshold` (40.0, `(0, 168]`, :438); `working_day.
  intensity_threshold_high` (1.2, `>0`, :439); `working_day.intensity_threshold_low` (1.1, `>0`,
  :440); `working_day.absolute_visibility` (1.0, `[0,1]`, :441); `working_day.relative_visibility`
  (0.3, `[0,1]`, :442).
- **(f) Events:** none.

### C1 — Wage-opposition deterioration (tick-scoped, once; ideology.py:153-157)
- **(a)** A LAST-tick "is the wage/value relation actively sharpening AND is labor losing"
  gate — contributes a flat per-node addend to every node's agitation this tick (same value for
  all nodes; see §5 for why this is class-independent and thus a documented, not-yet-repaired,
  theoretical flattening).
- **(b)** `wage_deterioration = max(0.0, rate) if balance < 0.0 else 0.0`
  (ideology.py:157), where `rate, balance = opposition_states["wage"]["rate"],
  opposition_states["wage"]["balance"]` (154-156), read off graph attr `opposition_states`
  (default `{}`,153).
- **(c) Reads:** graph attr `opposition_states` (a dict, written by `ContradictionSystem`
  @18.0 — LAST tick's value, since 17.0 < 18.0).
- **(d) Writes:** none (local scalar, added into every node's `new_agitation` in the loop, line
  380).
- **(e) Defines:** none (no coefficient — the gate is pass-through, not scaled).
- **(f) Events:** none.

### C2 — Persistent cross-tick state init (tick-scoped, once; ideology.py:186-198, 440-442)
- **(a)** Initialize-or-retrieve two node-id-keyed dicts from `context.persistent_data` (NOT the
  graph) that carry last tick's per-node wages and wealth forward; refreshed at the end of the
  tick to this tick's values for next tick's diff.
- **(b)** `persistent[PREVIOUS_WAGES_KEY] = persistent.get(PREVIOUS_WAGES_KEY, {})` (186-188, same
  for `PREVIOUS_WEALTH_KEY`, 192-194); at tick end: `persistent[PREVIOUS_WAGES_KEY] =
  current_wages; persistent[PREVIOUS_WEALTH_KEY] = current_wealth_map` (441-442).
- **(c) Reads:** `context.persistent_data` (a plain Python dict, `TickContext.persistent_data`,
  context.py:49) — **not graph state**.
- **(d) Writes:** `context.persistent_data` (same off-graph store).
- **(e) Defines:** none.
- **(f) Events:** none.

### C3 — Active-gate skip (per node; ideology.py:203-205)
- **(a)** Dead classes cannot develop consciousness.
- **(b)** `if not attrs.get("active", True): continue`.
- **(c) Reads:** `SOCIAL_CLASS.active` (bool, default True).
- **(d) Writes:** none.
- **(e) Defines:** none.
- **(f) Events:** none.

### C4 — Sustained wage-value balance / chauvinist pressure (per node; ideology.py:207-259)
- **(a)** If this class was actually paid this tick (both `w_paid` and `v_produced` present),
  compute the signed wage-vs-value balance and, from its POSITIVE part only, a "chauvinist
  pressure" that will later bias routing toward the fascist pole (the imperial-bribe theory,
  ADR082/Emmanuel/MIM/Amin, cited at length in the source).
- **(b)** `class_wage_balance = calculate_wealth_asymmetry_balance(v_produced, w_paid)`
  (243-245) = `min(1.0, max(-1.0, (w_paid - v_produced) / (w_paid + v_produced)))`
  (contradiction.py:98-102, `epsilon=1e-9` zero-guard, contradiction.py:99); `chauvinist_pressure =
  max(0.0, class_wage_balance) * chauvinist_pressure_scale` (252-255). Absent either field:
  `wage_data_present = False; class_wage_balance = chauvinist_pressure = 0.0` (257-259) —
  presence-gated, not a fabricated `0.0` fallback on the balance itself (the code's own extensive
  comment at 217-238 explains why: the Gaussian positive branch, C9 below, peaks NEAR 0, so a
  silent absent→0.0 default would fabricate near-peak agitation).
- **(c) Reads:** `SOCIAL_CLASS.w_paid`, `SOCIAL_CLASS.v_produced` (both transient, **not**
  `SocialClass` model fields — `SOCIAL_CLASS_COMPUTED_FIELDS`, world_state.py:67-68 — written only
  by `EconomicSystem` @9.0 on ticks it actually paid this class, economic.py:529-530).
- **(d) Writes:** none (locals feeding C9/C10).
- **(e) Defines:** `consciousness.chauvinist_pressure_scale` (1.0, `[0,1]`, defines.yaml:228).
- **(f) Events:** none.

### C5 — Continuous repression term (per node; ideology.py:261-296)
- **(a)** Only repression PRODUCED above the ambient model baseline counts, to avoid measuring
  `SocialClass`'s own default (0.5) as if it were a real repression event — documented as the
  root cause of a proven +0.00012 tick-1 canonical drift before this fix.
- **(b)** `effective_repression = max(0.0, repression_faced - DEFAULT_REPRESSION_FACED) if
  repression_faced is not None else None` (288-296).
- **(c) Reads:** `SOCIAL_CLASS.repression_faced` (Probability `[0,1]`, default 0.5 — see §3 for
  the presence-gate nuance: since the field carries a real Pydantic default, it is present on
  every node after the first `to_graph()`/`from_graph()` cycle, so the `is not None` gate is
  effectively always-true in production and the real work is the baseline subtraction, not
  absence-detection).
- **(d) Writes:** none.
- **(e) Defines:** `survival.default_repression` (0.5, `[0,1]`, defines.yaml:167, exposed via
  `GameDefines.DEFAULT_REPRESSION_FACED` property, `_assembler.py:266-269`).
- **(f) Events:** none.

### C6 — Core wages / wage_change (per node; ideology.py:298-309)
- **(a)** Sum this class's incoming WAGES-edge value flow this tick, and diff against last tick's
  sum (from C2's persistent store) to detect a wage cut.
- **(b)** `core_wages = Σ edge.attributes.get("value_flow", 0.0)` over `query_edges(edge_type=
  EdgeType.WAGES)` filtered to `edge.target_id == node.id` (299-302); `wage_change = core_wages -
  previous_wages.get(node.id, core_wages)` (308-309) (first-tick fallback: no baseline ⇒ 0
  change).
- **(c) Reads:** `EdgeType.WAGES.value_flow` (Currency `[0,∞)`, default 0.0) on every incoming
  WAGES edge; `previous_wages[node.id]` (C2's store).
- **(d) Writes:** `current_wages[node.id] = core_wages` (local dict, persisted at tick end via
  C2).
- **(e) Defines:** none.
- **(f) Events:** none.

### C7 — Wealth change (per node; ideology.py:311-317)
- **(a)** Periphery workers are extracted via EXPLOITATION edges, not wage cuts, so wealth itself
  (not just wages) must be diffed to detect crisis.
- **(b)** `wealth_change = current_wealth - previous_wealth.get(node.id, current_wealth)` (315-316)
  (same first-tick-fallback shape as C6).
- **(c) Reads:** `SOCIAL_CLASS.wealth` (Currency `[0,∞)`, default 10.0); `previous_wealth[node.id]`
  (C2's store).
- **(d) Writes:** `current_wealth_map[node.id] = current_wealth` (local, persisted via C2).
- **(e) Defines:** none.
- **(f) Events:** none.

### C8 — Solidarity pressure (per node; ideology.py:319-356)
- **(a)** Sum incoming SOLIDARITY-edge strength, with two DIFFERENT source-shape gates (ADR087):
  an organization-sourced edge's strength counts directly above a noise floor (organized mass work
  is itself the signal); a class-sourced edge counts only if the SOURCE class already has
  revolutionary consciousness above an activation threshold (an unconscious/bribed class transmits
  nothing).
- **(b)** For each incoming SOLIDARITY edge: `strength = edge.attributes.get("solidarity_strength",
  0.0)`; skip if `strength <= 0` (343-344); `src_node = graph.get_node(edge.source_id)`; if
  `src_node.node_type == NodeType.ORGANIZATION.value`: add `strength` iff `strength >
  negligible_transmission` (348-350); else (a class source): add `strength` iff
  `_get_ideology_profile_from_node(src_node.attributes)["class_consciousness"] >
  activation_threshold` (352-356).
- **(c) Reads:** `EdgeType.SOLIDARITY.solidarity_strength` (Coefficient `[0,1]`, default 0.0) on
  every incoming SOLIDARITY edge; the source node's `node_type` discriminant; the source node's
  `ideology.class_consciousness` (only for non-organization sources).
- **(d) Writes:** none (local `solidarity_pressure`, feeds C10).
- **(e) Defines:** `solidarity.negligible_transmission` (0.01, `>=0`, defines.yaml:186);
  `solidarity.activation_threshold` (0.3, `[0,1]`, defines.yaml:184).
- **(f) Events:** none.

### C9 — Agitation delta (per node; ideology.py:361-380)
- **(a)** Convert the tick's material deltas/levels into a single non-negative agitation
  increment, then add it plus C1's flat term onto last tick's carried agitation.
- **(b)** `agitation_increment = compute_agitation_delta(exploitation_rate_delta=abs(wage_change)
  if wage_change<0 else 0.0, imperial_rent_delta=wealth_change, visibility_delta=0.0,
  wage_balance=class_wage_balance if wage_data_present else None, repression_level=
  effective_repression, defines=services.defines.consciousness)` (372-379) →
  `exploit_component = max(0.0,exploitation_rate_delta)*exploitation_sensitivity +
  rent_component = max(0.0,-imperial_rent_delta)*rent_decline_sensitivity + vis_component =
  max(0.0,visibility_delta)*reproduction_visibility_coefficient + balance_component (C4's
  Gaussian, see §4) + repression_component = max(0.0,repression_level)*
  repression_level_sensitivity` (consciousness_routing.py:157-190). `visibility_delta` is a
  **hardcoded literal `0.0`** (ideology.py:375, comment: "g₃₃ changes handled in community
  system") — `vis_component` is therefore provably always `0.0` in this system's call site, dead
  weight in THIS system's arithmetic though the parameter exists for a different caller shape.
  `new_agitation = current_profile["agitation"] + agitation_increment + wage_deterioration` (380).
- **(c) Reads:** locals from C4/C6/C7/C5/C1; `current_profile["agitation"]` (from
  `_get_ideology_profile_from_node(attrs)`, 359).
- **(d) Writes:** none (local, feeds C10/C12).
- **(e) Defines:** `consciousness.exploitation_sensitivity` (0.15, `[0,1]`, :215);
  `consciousness.rent_decline_sensitivity` (0.2, `[0,1]`, :216); `consciousness.
  reproduction_visibility_coefficient` (0.1, `[0,1]`, :217, dead weight here per above);
  `consciousness.repression_level_sensitivity` (0.02, `[0,1]`, :229); plus C4's Gaussian
  coefficients (see C9-Gaussian note in §4).
- **(f) Events:** none.

### C10 — Ternary routing + popular-front throttle (per node; ideology.py:382-409)
- **(a)** Route the tick's agitation into revolutionary (`delta_r`) and fascist (`delta_f`) shifts
  by how much solidarity vs. chauvinist pressure is present, then throttle the fascist channel by
  any committed popular front's suppression share (P25 U12).
- **(b)** `delta_r, _delta_l, delta_f = route_agitation_to_ternary(agitation=new_agitation,
  solidarity_factor=min(1.0, solidarity_pressure), education_pressure=0.0, defines=services.
  defines.consciousness, chauvinist_pressure=chauvinist_pressure)` (394-400) →
  if `agitation<=0: return 0,0,0`; else `consumed = agitation *
  agitation_consumption_rate`; `effective_solidarity = max(0.0, min(1.0, min(1.0,
  solidarity_factor+education_pressure) - chauvinist_pressure))` (357-358, clamp AFTER the
  subtraction, deliberately ordered); `delta_r = consumed*effective_solidarity*routing_scale`;
  `delta_f = consumed*(1-effective_solidarity)*routing_scale`; `delta_l = -(delta_r+delta_f)`
  (consciousness_routing.py:347-368). `education_pressure` is a **hardcoded literal `0.0`**
  (ideology.py:397, "handled in community system") — dead weight here, same shape as C9's
  `visibility_delta`. Then: `delta_f *= 1.0 - _popular_front_suppression(graph)` (409), where
  `_popular_front_suppression` reads graph attr `popular_front` (a dict `{active, suppression}`),
  returns `0.0` unless `active` is truthy, else `max(0.0, min(1.0, float(suppression)))`
  (ideology.py:48-54).
- **(c) Reads:** locals from C4/C8/C9; graph attr `popular_front` (written by `ElectoralSystem`
  @17.45 — LAST tick's value, since 17.0 < 17.45).
- **(d) Writes:** none (locals, feed C11).
- **(e) Defines:** `consciousness.agitation_consumption_rate` (0.6, `[0,1]`, :220); `consciousness.
  routing_scale` (0.2, `[0,1]`, :213); `consciousness.education_pressure_decay` (0.1, `[0,1]`,
  :224, **not actually read on this call path** — belongs to CommunitySystem's own decay, listed
  in `ConsciousnessDefines` but not consumed here); `consciousness.liberal_drift_rate` (0.02,
  `[0,1]`, :221, likewise declared but not read on this path); `consciousness.educate_base_effect`
  / `consciousness.agitation_education_threshold` (declared, not read here — EDUCATE-verb-side
  consumers).
- **(f) Events:** none.

### C11 — Class/nation write (per node; ideology.py:410-411)
- **(a)** Apply the routed deltas, ceiling-clamped at 1.0 only (no floor — see §4 for why that is
  sufficient here, not a bug).
- **(b)** `new_class = min(1.0, current_profile["class_consciousness"] + delta_r)`; `new_nation =
  min(1.0, current_profile["national_identity"] + delta_f)`.
- **(c) Reads:** `current_profile["class_consciousness"]`, `current_profile["national_identity"]`
  (from `ideology`, C9's read); `delta_r`, `delta_f` (C10).
- **(d) Writes:** none (locals, applied in C13).
- **(e) Defines:** none (already folded into C10).
- **(f) Events:** none.

### C12 — Agitation decay (per node; ideology.py:412-414)
- **(a)** Entropy-decay the remaining agitation after routing consumed its fraction.
- **(b)** `new_agitation = max(0.0, new_agitation * (1.0 - agitation_decay_rate))` (floor-only
  clamp, correct for an unbounded-above `[0,∞)` domain).
- **(c) Reads:** `new_agitation` (post-C9, pre-decay).
- **(d) Writes:** none (local, applied in C13).
- **(e) Defines:** `consciousness.agitation_decay_rate` (0.1, `[0,1]`, defines.yaml:214).
- **(f) Events:** none.

### C13 — Graph write (per node; ideology.py:418-438)
- **(a)** Persist the new ideology triple, and separately compute+write a
  `material_conditions` buffer of three more derived scalars for (per the module's own docstring)
  "downstream systems" — **which, per §5, do not exist in production today.**
- **(b)** `graph.update_node(node.id, ideology={"class_consciousness": new_class,
  "national_identity": new_nation, "agitation": new_agitation}, material_conditions={"agitation":
  new_agitation, "exploitation_visibility": compute_exploitation_visibility(exploitation_rate=
  abs(wage_change) if wage_change<0 else 0.0, imperial_rent=max(0.0,wealth_change), defines=
  services.defines.consciousness, working_day_modifier=working_day_modifier),
  "reification_buffer": compute_reification_buffer(imperial_rent=max(0.0,wealth_change),
  total_v=max(1.0,core_wages))})`. `compute_exploitation_visibility`: `effective_rent =
  max(0.0,imperial_rent)*rent_opacity_factor`; `denominator = exploitation_rate+effective_rent+
  1e-10`; `visibility = max(0.0, min(1.0, exploitation_rate/denominator))` (returns `0.0` early if
  `denominator<=0`); `if working_day_modifier is not None: visibility *=
  max(0.0,min(1.0,working_day_modifier))` (consciousness_routing.py:240-255).
  `compute_reification_buffer`: `abs_rent = abs(imperial_rent)`; `max(0.0,min(1.0,
  abs_rent/(abs_rent+total_v+1e-10)))` (258-285).
- **(c) Reads:** locals from C11/C12/C7/C6/C0; `rent_opacity_factor` define.
- **(d) Writes:** `SOCIAL_CLASS.ideology` (all three sub-fields, unconditional per-node write);
  `SOCIAL_CLASS.material_conditions` (**silently discarded on every `WorldState.from_graph()`
  reconstruction** — see §5's name-collision finding; not a `SocialClass` model field or property
  setter, and the write's own dict shape has no key in common with the collision's `field_mapping`
  except the coincidental `repression_faced` no-op).
- **(e) Defines:** `consciousness.rent_opacity_factor` (1.0, `>=0`, defines.yaml:219).
- **(f) Events:** none.

**Events emitted by the whole system: zero.** Grep-confirmed across `ideology.py`,
`consciousness_routing.py`, `contradiction.py`, `sustained_exploitation.py`, and every
`working_day/*.py` module — no `EventType`/`event_bus`/`.publish(` reference anywhere in the call
chain.

## 3. TYPE INVENTORY

Same runtime-storage caveat as the Territory inventory (re-verified, `topology/graph.py:660-670`
unchanged on current dev): `update_node` is a plain dict merge, no coercion/`SnapToGrid`
mid-tick — everything below is raw Python `float`/`bool`/`dict`/`str` in-tick.

| Attribute | Node/edge/graph scope | Python model type | Domain | Category |
|---|---|---|---|---|
| `active` | SOCIAL_CLASS | `bool` | `{T,F}` | boolean gate |
| `w_paid` | SOCIAL_CLASS | **not a model field** — transient dict key | unconstrained (economic.py caps it non-negative in practice) | TRANSIENT COMPUTED (excluded on reconstruction, world_state.py:67-68) |
| `v_produced` | SOCIAL_CLASS | **not a model field** — transient dict key | same as above | TRANSIENT COMPUTED |
| `repression_faced` | SOCIAL_CLASS | `Probability` (`Annotated[float, ge=0,le=1]`) | `[0,1]` | unit-interval, real default (0.5) |
| `wealth` | SOCIAL_CLASS | `Currency` (`Annotated[float, ge=0.0]`) | `[0,∞)` | **unbounded real, money-semantic** (same Python-vs-BSL Currency mismatch flagged in the Territory inventory) |
| `ideology.class_consciousness` | SOCIAL_CLASS (nested dict field) | `IdeologicalProfile.class_consciousness` (`Annotated[float, ge=0,le=1]`) | `[0,1]` | unit-interval |
| `ideology.national_identity` | SOCIAL_CLASS (nested) | `IdeologicalProfile.national_identity` | `[0,1]` | unit-interval, default 0.5 |
| `ideology.agitation` | SOCIAL_CLASS (nested) | `IdeologicalProfile.agitation` (`ge=0.0`, **no upper bound**) | `[0,∞)` | **unbounded real** |
| `material_conditions.agitation` | SOCIAL_CLASS (nested) | `MaterialConditionsBuffer.agitation` (`ge=0.0`, no `le`) | `[0,∞)` | unbounded real; **write-only, never read back (§5)** |
| `material_conditions.exploitation_visibility` | SOCIAL_CLASS (nested) | `MaterialConditionsBuffer.exploitation_visibility` | `[0,1]` | unit-interval; **write-only, never read back** |
| `material_conditions.reification_buffer` | SOCIAL_CLASS (nested) | `MaterialConditionsBuffer.reification_buffer` | `[0,1]` | unit-interval; **write-only, `unconsumed` sentinel-exempted (task #42)** |
| `EdgeType.WAGES.value_flow` | edge | `Relationship.value_flow` (`Currency`) | `[0,∞)` | unbounded real, needs **EdgeRef field access** |
| `EdgeType.SOLIDARITY.solidarity_strength` | edge | `Relationship.solidarity_strength` (`Coefficient`) | `[0,1]` | unit-interval, needs **EdgeRef field access** |
| `node_type` (discriminant on a SOLIDARITY edge's source) | node (structural) | `GraphNode.node_type: str` | `{"social_class","organization",...}` (`NodeType` StrEnum) | **structural discriminant**, not a content `deffield` — servable by typed-neighbor query typing, not `defenum` |
| `opposition_states` | **graph attr** (not node/edge) | `dict[str, dict[str, float]]` | `wage.rate: [0,∞)`\* implied non-negative-clamped by `max(0.0,...)`; `wage.balance: [-1,1]` | graph-SCOPE register (R9 §3.6 carrier-node-type territory, see §5/§6) |
| `popular_front` | **graph attr** | `dict[str, Any]` (`{active: bool, suppression: float}`) | `suppression: [0,1]` (clamped at read, ideology.py:54) | graph-SCOPE register |
| `TickContext.persistent_data["previous_wages"]` | **off-graph** Python dict | `dict[str, float]` | unconstrained | cross-tick memory, **no BSL storage class exists for this at all** (see §5/§6) |
| `TickContext.persistent_data["previous_wealth"]` | **off-graph** Python dict | `dict[str, float]` | unconstrained | cross-tick memory, same gap |
| every `ConsciousnessDefines`/`SolidarityDefines`/`WorkingDayDefines` coefficient | — | `float` (most `[0,1]`; `chauvinist_peak_falloff` `(0,1]`; `absolute_hours_threshold`/`relative_hours_threshold` `(0,168]`; `intensity_threshold_*` `>0`; `sustained_exploitation_sensitivity` `[0,1]` **but interpreted against a `[-1,1]`-domain input**) | see §2 per-computation listing | unit-interval coefficients, one bounded-above-168 pair, one unbounded-above pair (`rent_opacity_factor >=0`) |

**Ideology/material_conditions nested-dict flag — the same class of gap the Territory inventory
found for enums, one level worse.** `deffield`'s closed type vocabulary
(`int`/`bool`/`currency`/`probability`/`intensity`/`coefficient`/`enum`, bsl-language.rst §3.1) has
no dict/struct/object row at all — not even the enum workaround applies, since these are not
discrete-membership types. Both `ideology` (3 sub-fields) and `material_conditions` (3 more)
must be flattened into 6 independent scalar `deffield`s per `social_class` node. This is
mechanically trivial (BSL already supports arbitrarily many flat fields per node — no vocabulary
gap, unlike Territory's enum finding) but is a real, undocumented-elsewhere content-modeling
decision needing its own D-record naming the 6 target field names.

**Graph-scope register flag.** `opposition_states`/`popular_front` are dicts hung off the graph
itself, not any node — bsl-language.rst §3.6 (R9 chapter C3, a **draft ruling**) already
prescribes the answer ("graph-scope state is ordinary node state on a declared carrier node type
… no new grammar and no new storage class") but **no landed pack on dev demonstrates this
pattern** (grep across `rust/crates/babylon-tick/content/{rules,scenarios}/*` for `carrier`/
`:ceiling 1` found zero hits) — the mechanism is specified, not yet exercised.

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`) except where flagged. In execution order
(C-numbers reference §2):

1. **Linear interpolation + two comparisons (C0):** `hours > threshold and intensity < low`;
   `hours <= threshold and intensity > high` (classifier.py:47-50); MIXED-branch `t = (hours -
   relative)/(absolute-relative)`, clamped `max(0.0,min(t,1.0))` (classifier.py:76-77) — a
   **two-sided clamp**, division guarded by `if span<=0.0: return average` (classifier.py:73-74).
2. **Sign/threshold gate, no multiply (C1):** `max(0.0, rate) if balance<0.0 else 0.0`
   (ideology.py:157) — bare comparison against literal `0.0`.
3. **Scale-free signed ratio (C4):** `(w_paid - v_produced)/(w_paid + v_produced)`, two-sided
   clamp `min(1.0, max(-1.0, balance))`, `epsilon=1e-9` zero-guard on the SUM (not added into the
   ratio itself — contradiction.py:98-102, same "exactly numeraire-invariant" design already
   established by TerritorySystem/Metabolism's Lawverian-rewrite precedent). `chauvinist_pressure
   = max(0.0, balance) * scale` — one multiply.
4. **THE LIBM HAZARD — Gaussian bump (C4/C9, via `sustained_exploitation_magnitude`,
   `formulas/sustained_exploitation.py:195-198`):**
   ```
   if balance < 0.0: return -balance * sensitivity
   distance = balance - chauvinist_peak_location
   return sensitivity * math.exp(-(distance**2) / (2.0 * chauvinist_peak_falloff**2))
   ```
   **`math.exp` — a libm transcendental.** Two compounding concerns, both explicitly named in this
   task's brief: (i) cross-implementation nondeterminism — per the standing CLAUDE.md rule, basic
   IEEE-754 ops reproduce across languages but libm transcendentals do not, and no tolerance policy
   is written anywhere in this codebase for this specific call; (ii) **ADR172 ruling 5 / ADR173
   ("no imposed functional forms — sigmoids must EMERGE … never be stipulated by a mechanic")** —
   this is not a sigmoid, but it is unambiguously a **stipulated, hand-shaped curve**: the
   docstring (sustained_exploitation.py:108-198) explicitly derives its SHAPE (a Gaussian bump
   peaked at a small positive `balance`, not a monotone function) from a theoretical claim (MIM:
   "the marginal labor aristocracy is the most reactionary of all") and explicitly rejects the
   "naive symmetric" alternative shape as ideologically wrong. `exp` is a **declared intrinsic**
   (`DECLARABLE_INTRINSICS = ["exp","log","floor"]`, `rust/crates/babylon-bsl/src/
   declarations.rs:110`, verified on current dev) so it is EXPRESSIBLE, but declarability does not
   settle the ADR173 question — this is a **port-question row for the Director**, not an
   auto-portable computation, independent of any query-lane blocker. Two-sided pow (`distance**2`,
   `falloff**2`) feeds the exponent; `chauvinist_peak_falloff`'s domain is `gt=0.0` (defines-
   enforced), so the denominator can never be zero — no epsilon needed structurally.
5. **Presence-gated subtract+floor (C5):** `max(0.0, repression_faced -
   DEFAULT_REPRESSION_FACED)` — one subtract, one floor-clamp.
6. **Edge-attribute fold-sum (C6, BLOCKED — see §6):** `Σ value_flow` over filtered edges — a
   summation, not yet expressible pending Slice 2.
7. **Diff against carried state (C6/C7):** `core_wages - previous_wages.get(id, core_wages)`;
   `current_wealth - previous_wealth.get(id, current_wealth)` — plain subtracts, but against
   **off-graph** state (§3/§5/§6).
8. **Edge-attribute fold-sum with dual gate (C8, BLOCKED):** conditional accumulation over
   filtered SOLIDARITY edges, one branch keyed on a structural node-type discriminant, the other
   on a nested-dict field read off the SOURCE node.
9. **Five-term linear sum (C9, `compute_agitation_delta`):** four `max(0.0, x) * coefficient`
   terms plus the Gaussian's output (item 4) — all pure linear scaling except item 4. One term
   (`vis_component`) is **provably always `0.0`** at this call site since `visibility_delta` is a
   hardcoded literal `0.0` (ideology.py:375) — dead arithmetic in THIS system's context, though the
   coefficient (`reproduction_visibility_coefficient`) is real and used by other callers of the
   same formula (none currently exist — grep-confirmed `compute_agitation_delta`'s only production
   caller is `ideology.py`).
10. **Consumption + clamp-after-subtract routing (C10, `route_agitation_to_ternary`):** `consumed
    = agitation * rate`; `effective_solidarity = max(0.0, min(1.0, min(1.0, solidarity+education) -
    chauvinist))` — **clamp intentionally applied AFTER the subtraction** (documented,
    consciousness_routing.py:353-358, "so a large chauvinist_pressure floors at 0.0 rather than
    going negative and inverting the term") — this is a genuinely deliberate, non-arbitrary
    ordering choice, worth transcribing exactly, not simplifying. `delta_r = consumed *
    effective_solidarity * scale`; `delta_f = consumed * (1-effective_solidarity) * scale`;
    `delta_l = -(delta_r+delta_f)` — all linear.
11. **Popular-front throttle (C10):** `delta_f *= 1.0 - suppression` — one multiply against a
    LAST-tick graph-scope register value (§5).
12. **Upper-clamp-only writes (C11):** `min(1.0, current + delta_r)`; `min(1.0, current +
    delta_f)` — **no lower clamp**, but this is mathematically sufficient, not a bug: every term
    feeding `delta_r`/`delta_f` is provably `>=0` (agitation `>=0` by construction — `IdeologicalProfile.
    agitation` has `ge=0.0` and every additive term into `new_agitation` is itself `max(0.0,...)`-
    gated or `0.0`; `route_agitation_to_ternary` returns `(0,0,0)` for non-positive agitation; both
    `effective_solidarity` and `1-effective_solidarity` are clamped `[0,1]`), so `current ∈ [0,1] +
    nonneg` never needs a floor. Contrast with TerritorySystem's two-INCONSISTENT-clamps finding —
    this system's clamp shapes are internally consistent with each field's actual reachable range.
13. **Floor-only decay (C12):** `max(0.0, agitation * (1.0 - decay_rate))` — correct shape for the
    `[0,∞)` domain (no upper bound needed).
14. **Two more scale-free ratios with epsilon guards (C13):** `compute_exploitation_visibility`'s
    `x/(x+y+1e-10)` (two-sided clamp, `denominator<=0` early-return) and
    `compute_reification_buffer`'s `|Φ|/(|Φ|+v+1e-10)` (two-sided clamp, caller floors `total_v` at
    `max(1.0, core_wages)` as a SECOND, belt-and-suspenders zero-guard on top of the formula's own
    epsilon).

**Real→Int demotions: NONE.** Grep-confirmed zero `int(...)` calls anywhere in `ideology.py` —
unlike TerritorySystem/MetabolismSystem, this system never truncates a Real into an Int (no
population-style integer field is touched here).

**Bare non-integer literals (BSL parser concern, same class as the Territory finding):** `0.0`
(dozens of sites), `1.0` (dozens), `2.0` (the Gaussian's `2.0 * falloff**2`), `1e-10` (two
epsilon sites in `consciousness_routing.py`), `1e-9` (`contradiction.py`'s epsilon). Every one
needs a `c`-suffixed `defconst` or the Real-zero-promotion idiom already established by the
landed packs.

**Clamp-shape audit (this system's version of the Territory two-clamp-inconsistency check):** NO
inconsistency found — every field's clamp shape (upper-only for `[0,1]`-with-monotone-increase
fields, two-sided for `[0,1]`-with-both-direction fields, floor-only for `[0,∞)` fields) matches
its actual reachable domain. This is a structurally cleaner system than Territory on this specific
axis; the one genuine hazard is the Gaussian (item 4), which is a functional-form question, not a
clamp-consistency question.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 17.0** (ideology.py:109), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-364`, per-class `position` ClassVars grepped individually):
  `… → SurvivalSystem (15.0) → StruggleSystem (16.0) → ConsciousnessSystem (17.0) →
  FascistFactionSystem (17.4) → AllegianceSystem (17.42) → ElectoralSystem (17.45) →
  PolicySystem (17.47) → SovereigntySystem (17.5) → MarketScissorsSystem (17.8) →
  ContradictionSystem (18.0) → …`.

- **Reads from same-tick prior systems:**
  - `w_paid`, `v_produced`, `EdgeType.WAGES.value_flow` — written by `EconomicSystem` @9.0
    (`economic.py:513-536`, capped at employer wealth, presence-gated on an actual payment this
    tick).
  - `repression_faced` — written by `EconomicSystem` @9.0 (subsidy-repression coupling,
    `economic.py:640`) AND by OODA-phase `ooda/action_effects.py` (@14.0, POGROM/VIGILANTISM
    bumps, `_bump_repression_edge`, lines 305-306/345-346/472-473).
  - `active` — written by `VitalitySystem` @1.0 (`vitality.py:169`, Reaper phase) and
    `DecompositionSystem` @11.0 (`decomposition.py:251,331,339`).
  - `wealth` — written by `VitalitySystem` @1.0, `ProductionSystem` @3.0, `EconomicSystem` @9.0,
    and, most immediately, **`StruggleSystem` @16.0** (`struggle.py:382,503`) — the LAST writer
    before this system runs.
  - **`ideology` (the exact dict key this system reads AND rewrites) — written earlier the SAME
    tick by `StruggleSystem` @16.0** (`struggle.py:409`, a class-consciousness boost gated on an
    UPRISING event firing, and `struggle.py:652`, a national-identity fascist-boost gated on a
    separate condition). This is a genuine, precisely-cited same-tick composition channel:
    ConsciousnessSystem's `current_profile = _get_ideology_profile_from_node(attrs)` (359) reads
    the graph AFTER Struggle's conditional write, so when Struggle fires, ConsciousnessSystem's
    `new_class`/`new_nation` are computed on top of Struggle's boosted baseline, not last tick's
    raw value.
  - `opposition_states` (graph attr) — written by `ContradictionSystem` @**18.0**, i.e. AFTER
    this system (17.0 < 18.0) — this system therefore always reads **LAST tick's** snapshot
    (documented in-source, ideology.py:143-144).
  - `popular_front` (graph attr) — written by `ElectoralSystem` @**17.45**, also AFTER this system
    — same one-tick lag, documented in-source (ideology.py:406-408, which additionally states "the
    qa six never carry the register" — see dormancy below).

- **Writes consumed later this tick / downstream ticks:**
  - `ideology.class_consciousness` — read same-tick, downstream, by: `FascistFactionSystem`
    @17.4 does NOT read `class_consciousness` (only `agitation`, next bullet); `AllegianceSystem`
    @17.42 (`allegiance.py:290`, explicit comment "the @17.0 write"); `ElectoralSystem` @17.45
    (`electoral.py:403-404`); `WealthDistributionSystem` @21.5 (`wealth_distribution.py:178`);
    `EpistemicHorizonSystem` @22.0 (`epistemic_horizon.py:96`, via `class_consciousness_from_node`).
    Read NEXT tick, upstream-of-17.0, by: `EconomicSystem` @9.0 (`economic.py:286`),
    `SolidaritySystem` @8.0 (`solidarity.py:140,148`), `StruggleSystem` @16.0
    (`struggle.py:404,561`, which ALSO further mutates it before this system runs again — the same
    same-tick composition channel above, recurring every tick).
  - `ideology.national_identity` — read same-tick downstream by `ElectoralSystem` @17.45
    (`electoral.py:403`); read next tick by `StruggleSystem` @16.0 (which can also boost it,
    `struggle.py:652`).
  - `ideology.agitation` — read same-tick downstream by `FascistFactionSystem` @17.4
    (`reactionary.py:353`) and `AllegianceSystem` @17.42 (`allegiance.py:467`).
  - `material_conditions.*` (all three sub-fields) — **read by nothing, anywhere, ever** (see
    finding below).

- **The material_conditions defect — a genuine, previously-undocumented root cause beyond what
  the existing `unconsumed` sentinel names.** `src/babylon/sentinels/unconsumed/registry.py`
  (verified read, lines 1-178) already declares `reification_buffer` as computed-but-unread and
  carries a dated exemption (owner Persephone Raskova, 2026-07-18, tracking task #42) — but its
  own diagnosis stops at "nothing downstream reads the key back." Tracing further: `SocialClass`'s
  `mode="before"` validator `unpack_components_and_convert_legacy`
  (`social_class.py:248-283`) calls `_unpack_component(data, "material_conditions",
  MaterialConditionsComponent, {"repression_faced": ("repression_faced", 0.5)})`
  (`social_class.py:268-273`) on EVERY `SocialClass(**entity_data)` construction — which
  `WorldState.from_graph()` performs at `world_state.py:1023`, itself called at the end of
  **every single `step()` call** (`simulation_engine.py`, the `return WorldState.from_graph(G,
  …)` tail). `_unpack_component` unconditionally **pops** the `material_conditions` key
  (`social_class.py:225`) and maps it through `field_mapping = {"repression_faced": (...)}` — a
  mapping authored for the OLDER, DIFFERENT `MaterialConditionsComponent` shape
  (`social_class.py:164-169`, only `repression_faced`), not the `MaterialConditionsBuffer` shape
  (`models/components/material_conditions.py:33-68`, `agitation`/`exploitation_visibility`/
  `reification_buffer`) ConsciousnessSystem actually writes. Since none of ConsciousnessSystem's
  three written keys is named `"repression_faced"`, all three values are **read via `.get(
  "repression_faced", 0.5)`, silently discarded, and never land on the reconstructed entity at
  all** — the whole `material_conditions` write is popped and thrown away every tick, not merely
  unread. `agitation`/`exploitation_visibility` are not even covered by the `unconsumed`
  sentinel's `DECLARED_COMPUTED_FIELDS` (only `reification_buffer` is a registered row) — they are
  a SECOND, currently entirely unmonitored instance of the same failure class. This is a real,
  verbatim defect in the frozen system (two unrelated Pydantic components sharing one dict key,
  one shadowing the other) that port-as-is law requires transcribing faithfully (the port must
  still perform the write, since that is what production code does), D-recorded as a known-dead
  write rather than silently repaired.

- **Context/service usage with no BSL equivalent:** `context.persistent_data` (`TickContext.
  persistent_data`, a plain mutable Python dict) carries `previous_wages`/`previous_wealth`
  ACROSS ticks, entirely off the graph. Unlike TerritorySystem's `TickContext.displacement_mode`
  (a per-run override with no live production writer, discharged as a provably-uniform `:const`),
  this is LIVE, load-bearing, every-tick state with no graph-side shadow at all — there is no
  existing BSL storage class for "memory that isn't a node/edge/hyperedge field." The
  `lifecycle.bsl` precedent (`content/rules/lifecycle.bsl:338`, `(binding prev-crisis :field
  territory/legitimation-crisis)`) shows the GENERAL pattern BSL already uses for "previous tick"
  values — but that pattern works because `legitimation-crisis` is ALREADY a graph field being
  diffed against its own prior value. `previous_wages`/`previous_wealth` are diffs against a
  DERIVED quantity (an edge-fold sum, and a plain field respectively) that Python currently
  memoizes off-graph — porting requires PROMOTING them to real per-node carrier fields (e.g.
  `social-class/prev-wage int`, `social-class/prev-wealth int`) written at the end of every tick,
  which is a content-modeling deviation from the frozen shape (new fields with no Python
  counterpart) though not a language blocker.

- **`services.productivity_data_source` — structurally, provably DORMANT on every canonical
  `qa:regression` scenario, code-level confirmed (not a dynamical guess).** `tools/
  regression_test.py`'s `_build_vol3_calculator_overrides` (186-207) and
  `build_single_county_overrides` (215-258) — the ONLY two `calculator_overrides` builders
  `_run_scenario_ticks` uses (1031-1035) — never set `productivity_data_source`. The FRED-backed
  wiring only exists via `create_vol1_services` (`domain/economics/factory.py:815`), itself only
  reachable through `engine/headless_runner/runner.py`'s `scope_fips`-gated branch (needs a real
  reference-DB session), which `qa:regression` never invokes. `ServiceContainer.create()`'s own
  default (`services.py:214,309,421`) is `None`. So `resolve_working_day_visibility_modifier`
  always returns `None` on all 11 canonical scenarios, and `compute_exploitation_visibility`'s
  `working_day_modifier` branch (consciousness_routing.py:252-253) never executes — the entire Ch.
  10 working-day term (C0, and half of C13) is dead weight on the canonical estate today,
  independent of any language-level blocker.

- **`popular_front` register — DORMANT on every canonical scenario, per the source's own
  comment** (ideology.py:406-408: "the qa six never carry the register") — `ElectoralSystem`
  writes it only on party-bearing scenarios with a genuinely seated/committed government; the
  gate defaults to the pre-U12 arithmetic bit-for-bit.

- **`opposition_states`/wage_deterioration — PLUMBED, not structurally dormant** (unlike
  `popular_front`): `ContradictionSystem` @18.0 is one of the 34 always-running default systems
  and writes this register every tick from tick 1 onward on every scenario; whether `rate>0 AND
  balance<0` ever co-occurs long enough to matter is a dynamical fact this static read did not
  trace numerically.

- **SOLIDARITY-edge dormancy — traced per canonical scenario, code-verified:**
  - `imperial_circuit`, `starvation`, `glut`, `fascist_bifurcation` (all four via
    `create_imperial_circuit_scenario`, `_legacy.py:255-263,446-454`): a SOLIDARITY edge
    (P_w→C_w) IS seeded, but `solidarity_strength` **defaults to `0.0`** and none of the four
    canonical `defines_overrides` (`tools/regression_scenarios.py:38-70`) touch it — so
    `solidarity_pressure` is provably `0.0` for the labor-aristocracy class on all four (the
    `strength<=0: continue` skip fires every tick), meaning **the class-sourced revolutionary
    routing branch (nonzero `delta_r` via this edge) never fires on any of these four**, including
    the scenario literally named `fascist_bifurcation` ("Consciousness routing to national
    identity" — its `defines_overrides` only retune `extraction_efficiency`/`sensitivity`, not
    `solidarity_strength`).
  - `two_node` (`_legacy.py:46-…`): no SOLIDARITY edge at all (grep-confirmed).
  - `single_county` (`engine/scenarios/single_county.py:50-…`): no SOLIDARITY edge at all
    (only EXPLOITATION/WAGES/TENANCY, per its own docstring).
  - `mitterrand`, `syriza`, `weimar` (`electoral_goldens.py`): no SOLIDARITY edge — the file
    contains exactly two `_solidarity(...)` call sites total (156-161), and neither is in these
    three factories; `weimar`'s own docstring context confirms ("no SOLIDARITY bridge anywhere",
    line 313 vicinity).
  - `debs` (`electoral_goldens.py:474`) and `bernie_valve` (`:534`): each seeds ONE real
    SOLIDARITY edge at `solidarity_strength=0.4` — well above `negligible_transmission` (0.01) —
    so the class-sourced revolutionary-routing branch is **topologically live** on these two.
    Whether the source class's `class_consciousness` ever crosses `activation_threshold=0.3`
    within these scenarios' tick horizons (the SECOND gate, `ideology.py:355`) is a dynamical
    fact not traced here.
  - `org_probe` (`engine/scenarios/org_probe.py`): no statically-seeded SOLIDARITY edge
    (grep-confirmed zero hits) despite being "the Organization estate's byte-gate anchor" — the
    org-sourced branch (ADR087, edges created at runtime by OODA mass-work verbs
    EDUCATE/PROPAGANDIZE/PROVIDE_SERVICE, `engine/actions/_mass_work.py`) is, if exercised at all
    on this scenario, populated DYNAMICALLY, not visible to static grep. **UNVERIFIED** whether
    org_probe's tick horizon ever actually fires one of these verbs — the search run was `rg -n
    'SOLIDARITY|solidarity_strength' src/babylon/engine/scenarios/org_probe.py`, zero hits, no
    further dynamic trace attempted (read-only constraint).

## 6. BLOCKER ASSESSMENT

Adjudicated against the CURRENT BSL surface stated in this task's brief (query lane Slice 1
landed; Slices 2-4 not built; enum fields landed; `exp`/`log`/`floor` declarable; Currency-field
storage refused; events unpinnable; two-rules-one-position pre-state sharing open).

| Computation | Verdict | Detail |
|---|---|---|
| C0 working-day visibility modifier | **PORTABLE WITH D-RECORD** | Pure arithmetic (comparisons + linear interpolation), no transcendentals, all coefficients `[0,1]`-or-bounded. Structurally DORMANT on every canonical scenario (§5) — the honest port declares `working_day_modifier` a `:const` multiplicative-identity (skip the multiply), Metabolism-D-2-style, until a real Rust-side FRED adapter exists. |
| C1 wage-opposition deterioration | **PORTABLE WITH D-RECORD** | The gate itself is trivial (`max`+comparison). Reads `opposition_states`, a graph-scope register with no landed carrier-node precedent (§3) AND written by `ContradictionSystem`, not yet ported. Sequencing-dependent: honest const-zero today (matches the frozen Python's own absent-safe default at tick 1), must be revisited once both the carrier-node pattern is demonstrated and ContradictionSystem lands. |
| C2 persistent cross-tick state (`previous_wages`/`previous_wealth`) | **PORTABLE WITH D-RECORD** | No language blocker — needs 2 new carrier `deffield`s per social-class node (D-record: name them, document they have no Python-model counterpart) written at tick-end, read at tick-start, mirroring the `lifecycle.bsl:338` prev-field precedent. |
| C3 active-gate | **PORTABLE NOW** | Plain bool guard, exact precedent in every landed pack. |
| C4 sustained wage-value balance (ratio only) | **PORTABLE NOW** | Scale-free signed ratio, same class as `calculate_wealth_asymmetry_gap`/`balance`'s already-accepted Lawverian-rewrite pattern; `w_paid`/`v_produced` are `:field`-sourced (once EconomicSystem ports) so evaluate as `Value::Real`, same D-1 bare-scaled-Int-adjacent class already precedented for Metabolism/Territory. |
| C4 chauvinist-pressure Gaussian (`sustained_exploitation_magnitude`'s positive branch) | **PORT-QUESTION — Director escalation required** | `math.exp` call (§4 item 4): a libm nondeterminism hazard AND an explicitly stipulated, hand-shaped, non-monotonic curve chosen to encode a specific theoretical claim — squarely ADR172 ruling 5 / ADR173's "no imposed functional forms" territory, independent of `exp`'s declarability. **RESERVED-LINE**: the peak location/falloff coefficients and the Emmanuel/MIM/Amin citations grounding them are Director-owned theoretical content (correct-revolutionary-theory pedagogy), not an engineering parameter to retune informally. |
| C5 continuous repression term | **PORTABLE WITH D-RECORD** | Pure arithmetic. `repression_faced`'s live producers (OODA POGROM/VIGILANTISM, `economic.py` subsidy coupling) are not yet ported — honest const-zero (matches the frozen Python's own canonical-scenario behavior, since none of the 11 fire POGROM/VIGILANTISM per the existing theory notes) until those systems land. |
| C6 core_wages (WAGES edge `value_flow` fold) | **BLOCKED — query-lane Slice 2 (EdgeRef field access)** | bsl-language.rst §2.9/§2.10 already NAME the target construct — `(fold sum (edges EdgeType/WAGES) (field-of it wages/value-flow))` — but per this task's brief, edge-attribute reads are unbuilt; independently verified (`score_class.rs`, bsl-language.rst:811: "no §2.7 production could read anything off an EdgeRef"). This is the single most load-bearing read in the whole system — it decides `wage_change`, which feeds `agitation_increment`. |
| C7 wealth_change | **PORTABLE WITH D-RECORD** | `wealth` itself is an ordinary field; only the cross-tick diff needs C2's carrier-field workaround. |
| C8 solidarity_pressure (SOLIDARITY edge `solidarity_strength` fold + dual source-shape gate) | **BLOCKED — query-lane Slice 2 (EdgeRef field access)** | Same edge-attribute gap as C6, compounded: also needs a neighbor-node structural `node_type` discriminant check (servable NOW via typed-neighbor query typing, Slice 1) and a neighbor's flattened `ideology.class_consciousness` field read (servable NOW via `field-of` once C13's flattening D-record lands) — but the `solidarity_strength` value itself is the hard blocker regardless. This is THE read that decides bifurcation direction (revolutionary vs. fascist pole) — the system's central game-loop mechanic is unportable until Slice 2 lands. |
| C9 agitation-delta linear terms (excluding the Gaussian) | **PORTABLE NOW** | Four `max(0,x)*coefficient` terms, pure linear scaling, `[0,1]`-domain coefficients throughout. `vis_component` is provably dead weight at this call site (hardcoded `0.0` input) — port-as-is transcribes it anyway. |
| C10 ternary routing (linear part) | **PORTABLE NOW** | All multiply/subtract/clamp, no transcendentals; the clamp-after-subtraction ordering (§4 item 10) is deliberate and must transcribe exactly, not simplify. |
| C10 popular-front throttle | **PORTABLE WITH D-RECORD** | Reads `popular_front`, a second graph-scope register with no landed carrier-node precedent, written by not-yet-ported `ElectoralSystem`. Const-zero today is a FAITHFUL transcription, not a deviation — the frozen Python's own comment confirms the qa estate never exercises it either. |
| C11 class/nation write | **PORTABLE NOW** | Upper-clamp-only, mathematically sufficient given the monotonic-increase invariant (§4 item 12) — the port must transcribe the missing lower clamp AS MISSING (port-as-is), not add one defensively. |
| C12 agitation decay | **PORTABLE NOW** | Floor-only clamp, linear decay, exact precedent (`vitality.bsl`'s `agitation_decay_rate`-shaped terms). |
| C13 ideology dict write | **PORTABLE WITH D-RECORD** | Needs the 3-field flattening decision (§3); mechanically trivial, no vocabulary gap. |
| C13 material_conditions dict write | **PORTABLE WITH D-RECORD (transcribe a known-dead write)** | Needs the same 3-field flattening. Per §5's defect finding, this write is discarded by the frozen Python itself every tick — port-as-is law requires the BSL port to still perform the write (faithful transcription of what production code does, defect included), D-recorded explicitly as dead output so a future porter does not "fix" it by silently dropping it OR treat its zero-consumer status as license to skip transcribing the arithmetic. |
| `TickContext.displacement_mode`-equivalent | **N/A** | ConsciousnessSystem has no analogous test/API-only override; not applicable. |

**Summary verdict logic:** two computations (C6, C8) are hard-blocked on the same missing lane
(query evaluation Slice 2, EdgeRef field access), and C8 is the mechanic the whole system exists
to compute (bifurcation routing direction). A dozen other computations are individually portable
or portable-with-D-record, but landing them as a partial pack ahead of Slice 2 would ship
"consciousness drift with the bifurcation gate permanently defaulted to fascist" (since
`solidarity_pressure` would read as a `:const 0` in the interim, identical in EFFECT to the
already-observed canonical dormancy on 9 of 11 scenarios) — a sliver-only port of exactly the
kind the Territory inventory's DEFER precedent rejected as silent scope shrink. The Gaussian
(C4's positive branch) is additionally a Director-escalation item independent of Slice 2's
status, so even a Slice-2-unblocked pack cannot fully auto-port without that ruling.

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_ideology.py` | 627 | **Primary conformance oracle.** Originally a TDD red-phase suite for wealth-extraction tracking (periphery `wealth_change` alongside core `wage_change`); has since accreted general per-node behavior coverage. |
| `tests/unit/formulas/test_consciousness_routing.py` | 624 | **Formula-level conformance oracle** for the whole Spec-043 pipeline (`compute_agitation_delta`, `compute_exploitation_visibility`, `compute_reification_buffer`, `route_agitation_to_ternary`, `normalize_to_simplex`) — the most reusable byte-for-byte-checkable layer, decoupled from graph plumbing. |
| `tests/unit/formulas/test_sustained_exploitation.py` | 242 | **Formula-level oracle for the Gaussian bump.** Pins the non-monotonic shape directly (`TestNonMonotonicFromOneTowardZero` per the source docstring's citation) — the single most important file for verifying any BSL transcription of the `exp` call reproduces the SAME curve, not just the same asymptotic behavior. |
| `tests/unit/engine/systems/test_ideology_sustained_term.py` | 427 | System-level integration of the sustained wage-value-balance term; explicitly documents and guards the per-class-not-global defect fix (the variance-error/erasure-of-theory bug this task's own project memory also flags). |
| `tests/unit/engine/systems/test_ideology_repression_continuous_term.py` | 259 | System-level oracle for C5 (the continuous repression term), including the tick-1 ambient-default drift regression this term exists to fix. |
| `tests/unit/engine/systems/test_ideology_working_day.py` | 251 | System-level oracle for C0/C13's working-day wiring — the ONLY test surface that exercises `productivity_data_source` non-`None` (via a fixture double), since no canonical scenario does (§5). |
| `tests/unit/engine/systems/test_ideology_chauvinist_recoupling.py` | 224 | System-level oracle for C4/C10's chauvinist-pressure wiring (the Consciousness Recoupling correction). |
| `tests/unit/engine/systems/test_consciousness_integration.py` | 255 | Phase-3 integration test: pins that `ConsciousnessSystem` writes `MaterialConditionsBuffer`-shaped data — **written against the write site, not the (nonexistent) consumer**, so it cannot by itself surface §5's discard defect (a test asserting the WRITE happened, not that a later READ recovers it). |
| `tests/unit/engine/systems/test_ideology_defines_passthrough.py` | 107 | Regression guard: every `consciousness_routing` call in the per-node loop must thread live `services.defines.consciousness` through (a class-wide bug once found in two of three formula calls). |
| `tests/unit/economics/working_day/test_resolver.py` | 294 | Dependency-level oracle for `resolve_working_day_visibility_modifier`/`resolve_absolute_relative_surplus_ratio` (the latter NOT called by this system, §1). |
| `tests/unit/config/test_consciousness_value_defines.py` | 167 | Schema-level: pins `ConsciousnessDefines` field existence/bounds/provenance — not tick-behavior. |
| `tests/property/invariants/test_simplex_pipeline.py` | 147 | **Property-based invariant contract** (INV-008/spec-054 US3): single-tick and 5-tick simplex-preservation predicates over the full pipeline — a genuine behavioral-contract layer, not implementation-coupled. |
| `tests/test_simplex_invariants.py` | 173 | Hypothesis-based property test over `normalize_to_simplex`/`route_agitation_to_ternary` directly — another independent property-based oracle, at the formula layer. |
| `tests/unit/sentinels/test_unconsumed.py` | 287 | **Directly relevant to §5's finding** — tests the sentinel that (partially) documents the `reification_buffer` discard; does not itself test `agitation`/`exploitation_visibility` (not registered rows) or the root-cause `_unpack_component` collision. |
| `tests/integration/test_consciousness_evolution.py` | 242 | Integration-level, 520-tick canonical-run drift assertions (SC-005/SC-006/SC-010) — exercises the REAL headless-runner wiring, closest thing to an end-to-end oracle, but couples to many other systems' behavior too. |
| `tests/integration/mechanics/test_proletarian_internationalism.py` | 396 | Cross-system integration (SolidaritySystem + ConsciousnessSystem together) for the solidarity-transmission counterforce; its "events emitted" coverage is `SolidaritySystem`'s own emissions, not this system's (this system emits none, §2). |
| `tests/unit/bifurcation/test_consciousness.py` | (not read in full) | Tests the OUT-OF-SCOPE `domain/bifurcation/consciousness.py` sigmoid module (§1) — NOT a conformance oracle for this system; flagged here only to prevent a future porter from conflating the two "consciousness" test suites. |

**qa:regression byte-gate coverage.** As with Territory, `tools/regression_test.py::
graph_content_hash` hashes every node/edge attribute of the `WorldState→graph` projection, so
any change to `ConsciousnessSystem`'s outputs on any of the 11 canonical scenarios is caught by
the byte-identical hash gate. Given §5's dormancy findings, that coverage is real for: the active
gate, the wage/wealth-change deltas (where WAGES edges exist), the linear agitation terms, and
the ternary routing's fascist-dominant path (9 of 11 scenarios). It is **NOT real** for: the
working-day modifier (dormant on all 11), the popular-front throttle (dormant on all 11), the
class-sourced revolutionary-routing branch (topologically live only on `debs`/`bernie_valve`, and
only past the `activation_threshold=0.3` crossing, unverified here), and the org-sourced
solidarity branch (possibly live only dynamically on `org_probe`, unverified here). A port's
conformance fixtures will need hand-built `.bscn` scenarios for all four gaps, matching the
Metabolism/Territory precedent.

---

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`) with fresh `rg`/Read. Four corrections,
six confirmations.

### CORRECTIONS

1. **CORRECTION — there is no `EconomicSystem`.** The class at
   `src/babylon/engine/systems/economic.py:26` is `ImperialRentSystem`
   (`position: ClassVar[float] = 9.0`, economic.py:37), and `simulation_engine.py:54` imports it
   under that name; no `EconomicSystem` appears anywhere in `_SYSTEM_CLASSES`
   (simulation_engine.py:328-363) or in `economic.py`. Every line number the inventory attributes to
   "EconomicSystem @9.0" checks out — `w_paid`/`v_produced` at economic.py:529-530, the WAGES
   `value_flow` write at :535, the subsidy-repression write at :640 — only the class name is wrong.
   It recurs ~8 times across §1 and §5 and would send a porter hunting a class that does not exist.

2. **CORRECTION — "11 canonical scenarios" is 12.** `tools/regression_scenarios.py`'s `SCENARIOS`
   dict has twelve keys: `imperial_circuit`, `two_node`, `starvation`, `glut`,
   `fascist_bifurcation`, `single_county`, `mitterrand`, `syriza`, `weimar`, `debs`,
   `bernie_valve`, `org_probe`. `tests/baselines/` carries twelve matching JSON baselines and twelve
   dense CSVs, and `tools/regression_test.py:1363,1424,1777` iterate `SCENARIOS` wholesale with no
   exclusion list. Every ratio in §5/§7 ("9 of 11", "dormant on all 11") restates against 12.

3. **CORRECTION — C1's `opposition_states` is NOT "PLUMBED, not structurally dormant"; it is
   provably empty on every canonical tick.** `WorldState.opposition_states` is a **write-only seed**:
   `to_graph` stamps it (`G.graph["opposition_states"] = dict(self.opposition_states)`,
   world_state.py:721) and `from_graph` never reconstructs it — the field's own docstring says so
   verbatim ("``from_graph`` does NOT reconstruct it — the authoritative cross-tick carrier is the
   persisted graph itself (bridged runner). The in-memory Simulation facade rebuilds the graph from
   WorldState each tick and therefore recomputes the snapshot fresh (no cross-tick memory)",
   world_state.py:539-552); grep confirms the only `opposition_states` sites in `world_state.py` are
   539/544/581/721, none of them a read. `simulation_engine.step()` performs the full round trip
   every tick (`G = state.to_graph()` at :552, `return WorldState.from_graph(` at :606), and **no
   `SCENARIOS` factory seeds `opposition_states`** (`rg -n "opposition_states"` across
   `src/babylon/engine/scenarios/*.py`, `tools/regression_scenarios.py`, `tools/regression_test.py`:
   zero hits). So ContradictionSystem @18.0's write is discarded before ConsciousnessSystem @17.0
   next reads it, and `wage_deterioration` (ideology.py:153-157) is provably `0.0` on all twelve
   canonical scenarios — not "a dynamical fact this static read did not trace numerically". The C1
   blocker-row remedy (const-zero) is therefore **exact**, in the Metabolism-D-2 /
   Territory-`displacement_mode` "provably uniform `:const`" class, not merely "honest today".

4. **CORRECTION — §7's byte-gate paragraph over-reaches on three counts.** `graph_content_hash`
   (`tools/regression_test.py:924-964`) hashes `state.to_graph()`'s **nodes and edges only**, and its
   own docstring states: "Graph *metadata* (``g.graph``: economy, event log, opposition states) is
   also excluded". Consequently:
   - **`material_conditions` is invisible to the gate twice over.** Beyond §5's `_unpack_component`
     discard, it is not a `SocialClass` model field at all — `to_graph` emits
     `**entity.model_dump()` (world_state.py:741) and `material_conditions` is a plain `@property`
     (social_class.py:517-521), not a `computed_field`, so `model_dump()` never carries it. The write
     could never reach the hash even if the validator were repaired.
   - **`w_paid`/`v_produced` are equally unhashed** — both are `SOCIAL_CLASS_COMPUTED_FIELDS`
     members (world_state.py:66-68), dropped on reconstruction. C4's entire presence gate (and hence
     the Gaussian's only input) sits outside the byte gate on every scenario.
   - **`opposition_states`/`popular_front` are `g.graph` metadata**, excluded by construction.
   What the gate actually covers for this system is exactly `SOCIAL_CLASS.ideology` (a declared model
   field). The §7 sentence "any change to `ConsciousnessSystem`'s outputs … is caught by the
   byte-identical hash gate" must be narrowed to that one field.

### CONFIRMATIONS

5. **CONFIRMATION, STRENGTHENED — C6/C8's edge-attribute blocker is real and deeper than "Slice 2
   mints `EdgeKey`".** `evaluator.rs:503-512` lists `("edges","slice 2")`,
   `("edge-between","slice 2")`, `("the","slice 2")` in `UNSERVED_EXPRESSION_HEADS`, so the
   expression heads are indeed unserved. But `babylon-graph`'s `GraphSubstrate` trait has **no
   edge-attribute reader of any kind**: its entire edge surface is
   `add_edge(edge_type, from, to, strength: f64)` (substrate.rs:111-117), `remove_edge` (:124) and
   `edges(edge_type) -> Vec<(NodeId, NodeId)>` (:166). `node_attribute(id, attribute)` exists at :141
   with **no edge counterpart**, and even `add_edge`'s mandatory `:strength` has no reader
   (`rg -n "strength" substrate.rs` → lines 22, 105, 116 only, all write-side). Slice 2 must mint the
   substrate read method as well as the `EdgeRef`. Verdict unchanged; record the substrate half in
   the eventual Slice-2 scope.

6. **CONFIRMATION — the `material_conditions` discard defect is real, verbatim as described.**
   `_unpack_component` pops the key unconditionally (`component = data.pop(key)`,
   social_class.py:225) and maps it through `{"repression_faced": ("repression_faced", 0.5)}`
   (social_class.py:268-273); none of the three keys ConsciousnessSystem writes
   (`agitation`/`exploitation_visibility`/`reification_buffer`, ideology.py:425-437) is named
   `repression_faced`, so `component.get("repression_faced", 0.5)` yields the default and the
   surviving `data.setdefault("repression_faced", 0.5)` is a no-op on any node already carrying the
   attribute. The two colliding shapes are `MaterialConditionsComponent` (social_class.py:164-169,
   one field) and `MaterialConditionsBuffer` (`models/components/material_conditions.py`, three
   fields). Port-as-is transcription verdict stands.

7. **CONFIRMATION — the Gaussian PORT-QUESTION.** `sustained_exploitation.py:198`:
   `return sensitivity * math.exp(-(distance**2) / (2.0 * chauvinist_peak_falloff**2))`. One
   navigational correction inside the confirmation: §1's row reads "ConsciousnessSystem calls only
   `sustained_exploitation_magnitude` (line 102)" — 102 is the **definition** site, not a call site.
   `ideology.py` never calls it; the real chain is `ideology.py:372` → `compute_agitation_delta` →
   `consciousness_routing.py:171-176` (`balance_component`) → `sustained_exploitation.py:198`. The
   ADR172 ruling-5 / ADR173 escalation verdict is correct and unaffected.

8. **CONFIRMATION — `productivity_data_source` dormancy, code-level.** Neither
   `_build_vol3_calculator_overrides` (regression_test.py:186) nor `build_single_county_overrides`
   (:215) sets it; `_run_scenario_ticks` uses only those two (:1031-1034); the sole setter tree-wide
   is `domain/economics/factory.py:815`; `ServiceContainer` defaults it `None` (services.py:214). C0
   and half of C13 are dead weight on the whole canonical estate.

9. **CONFIRMATION — the SOLIDARITY dormancy trace, in every particular.**
   `_legacy.py:262,455,961,1008` default `solidarity_strength=0.0`; `electoral_goldens.py` has
   exactly two `_solidarity(...)` call sites — `:474` (`debs`, 0.4) and `:534` (`bernie_valve`, 0.4);
   `single_county.py:69-136` seeds only EXPLOITATION/WAGES/TENANCY; `org_probe.py` has zero SOLIDARITY
   hits. One near-miss the trace correctly avoids: `_legacy_wayne.py:526` seeds a
   `solidarity_strength=0.05` edge, but it belongs to `create_wayne_county_scenario`, which is **not**
   a `SCENARIOS` key — the trace is unaffected.

10. **CONFIRMATION with an addendum — the StruggleSystem same-tick `ideology` channel.** Position 17.0
    confirmed at ideology.py:109. `struggle.py` writes `ideology=` at **three** sites, not two:
    `:409`, `:608` (`graph.update_node(p_w_id, p_revolution=1.0, ideology=new_ideology)`) and `:658`
    (the inventory's "652"). Site `:608` is a third same-tick composition channel §5's table omits; a
    port's conformance fixture must reproduce all three.

### RESERVED-LINE addendum

The reader flagged the Gaussian's peak/falloff coefficients (correct). One further ideological
surface is unflagged: `_get_ideology_profile_from_node`'s absent-data default
`"national_identity": 0.5` (ideology.py:74, :82, :89 — three copies) encodes a theoretical prior
(a class with no ideology record is half-nationalist) that a BSL port must re-declare explicitly as a
`deffield` default. Name it in the flattening D-record and route the value past the Director rather
than inheriting it silently.

### FINAL VERDICT

**BLOCKED — query-lane Slice 2 (EdgeRef field access *and* the `GraphSubstrate` edge-attribute
reader it presupposes) for `core_wages` (C6) and `solidarity_pressure` (C8), the two reads that
decide bifurcation direction; the C4 chauvinist-pressure Gaussian is a separate Director-escalation
PORT-QUESTION under ADR172 ruling 5 / ADR173 regardless of Slice 2. Sustained, with two verdicts
hardened: C1's `opposition_states` read is provably `0.0` on all twelve canonical scenarios (a
`:const` in the Metabolism-D-2 class, not an open dynamical question), and the byte-gate cover for
this system is exactly `SOCIAL_CLASS.ideology` — `material_conditions`, `w_paid`/`v_produced` and
both graph-scope registers lie outside `graph_content_hash` entirely.**
