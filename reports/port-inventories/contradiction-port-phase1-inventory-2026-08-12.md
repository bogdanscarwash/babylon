# ContradictionSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `ContradictionSystem` (position 18.0, Consequences phase, `src/babylon/engine/systems/contradiction.py`, 1127 lines) has two structurally different halves: a small, cleanly-portable Phase 1 (fresh per-edge `tension`, three edge types, pure arithmetic) and a large Phase 2 (the Lawverian `OppositionRegistry` stepping nineteen named contradictions, a typed `CouplingGraph` re-ranking pass, frame/rupture/regime derivation, and a shadow per-node pole channel) that is architecturally a small object-oriented registry/ranking subsystem, not a set of independent graph-query rules. Phase 1 is blocked today not by the query-evaluation gap Territory hit, but by a **substrate storage gap**: `update-edge` parses and typechecks but is refused at evaluation because `GraphSubstrate` has no attribute storage for edges at all (`rust/crates/babylon-bsl/src/structural_verbs.rs:16-26,387-398`) — a strictly deeper blocker than any Slice-1/2/3 query gap. Phase 2's registry/coupling/regime machinery has no BSL analog of any kind (no named-binding ranking primitive, no typed-morphism-graph primitive, no level-lattice Aufhebung primitive) and reads two graph attributes (`tick_dynamics`, `national_financial`) that carry live Pydantic **object instances**, not attribute-shaped data — BSL has no graph-level-attribute concept to receive them. Two libm transcendentals are load-bearing in the system's own code (`math.exp` at contradiction.py:455, `math.tanh` via a direct call to `formulas.market.calculate_scissors_balance` at contradiction.py:427). The qa:regression byte-gate hash explicitly **excludes graph-level attributes** (`tools/regression_test.py:940`), so five of the system's six write channels (everything but the edge `tension` write) are invisible to it; the real conformance oracle is `tests/unit/engine/systems/test_contradiction_system.py` (874 lines, 51 tests) plus the dialectics unit/property suites.

**Verdict: BLOCKED — Phase 1 on edge-attribute-storage (a substrate gap, not a query-lane gap); Phase 2 on the absence of any BSL primitive for a ranked multi-binding registry, a typed coupling graph, or a level-lattice regime classifier, plus two graph attributes carrying live Python objects with no BSL representation. Nothing in this system is portable today without either a substrate-storage decision (edge attributes) or a new architecture design for the registry/coupling/regime layer — this is a bigger, and different-shaped, problem than the query-evaluation gap that blocked Territory.**

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/contradiction.py` | 1127 | **The target.** `ContradictionSystem`, position 18.0 (contradiction.py:169-170). `step()` at 176-186 calls exactly two methods: `_write_edge_tensions` (185) and `_step_registry` (186). |
| `src/babylon/domain/dialectics/core/opposition.py` | 620 | `OppositionRegistry[I]` — the ranked-binding stepper (`.step()`, `.read_poles()`, principal-key scoring); `OppositionState`/`GapReading`/`PoleReading`/`PoleSample`/`OppositionSpec`/`BoundOpposition` models. |
| `src/babylon/domain/dialectics/core/coupling.py` | 225 | `CouplingGraph` (typed morphism graph: `feeds`/`constrains`/`transforms`/`contains`/`antagonizes`); `StanceIntervention` + `apply_interventions`. |
| `src/babylon/domain/dialectics/core/regime.py` | 86 | `classify_regime` — the fixed-point regime classifier (reproduction/crisis/sublation). |
| `src/babylon/domain/dialectics/instances/catalog.py` | 1272 | `GraphInputs` (the per-tick snapshot dataclass, 20 fields); the 19 opposition `_*_measure` functions + `_mean_asymmetry`/`_ratio_reading` shared kernels; `build_default_registry`/`build_default_coupling_graph`. |
| `src/babylon/domain/dialectics/instances/levels.py` | 651 | `level_index_for`, `spatial_lattice_for_counties` — the `county≺state≺nation` `LevelLattice` the regime classifier's sublation probe consumes. |
| `src/babylon/domain/dialectics/instances/value_form.py` | 580 | `compute_fundamental_theorem`, `phi_class` — the Fundamental Theorem Φ = W_c − V_c report `ContradictionSystem` stashes each tick. |
| `src/babylon/domain/dialectics/instances/connectivity.py` | 178 | `atomization_index`, `connectivity_cylinder` — feed the `atomization` opposition's gap/balance. |
| `src/babylon/formulas/contradiction.py` | 154 | `calculate_wealth_asymmetry_gap`/`_balance` (the shared two-pole formula, live); `calculate_contradiction_intensity` (deprecated, **not imported by contradiction.py or catalog.py** — dead in this system's live path, retained "only for the deprecation window", contradiction.py's own module docstring line 14). |
| `src/babylon/formulas/market.py` | 227 | `calculate_scissors_balance` (line 97-107, `math.tanh`) — called **directly by `ContradictionSystem`** at contradiction.py:427-430, not merely read pre-computed. |
| `src/babylon/formulas/fundamental_theorem.py` | 145 | `calculate_imperial_rent_gap` (41-70, registered as `"phi_absolute"`), `calculate_labor_aristocracy_ratio` (16-38), `is_labor_aristocracy` (73-94) — all three consumed by `compute_fundamental_theorem`. |
| `src/babylon/sentinels/partition/registry.py` | 90 | `cell_name`, `CELL_AXIS_NAMES`, `PRINCIPAL_AXES` — the single source of truth `_write_pole_shadow` imports (contradiction.py:82). |
| `src/babylon/sentinels/partition/checks.py` | 198 | `analyze_partition` — an **offline CLI consumer** (`tools/partition_probe.py`) of the `pole_readings` graph attribute; not an engine System, not part of the tick. |
| `src/babylon/engine/topology_monitor.py` | 587 (relevant: 53-90) | `extract_solidarity_subgraph` — builds the undirected SOLIDARITY subgraph the `atomization` measure reads. Read-only. |
| `src/babylon/domain/economics/working_day/resolver.py` | 175 | `resolve_absolute_relative_surplus_ratio` (124-175) — the `absolute_relative_surplus` opposition's feed. |
| `src/babylon/domain/economics/tick/graph_bridge.py` | 586 (relevant: 41, 80-108, 430-450) | `TICK_DYNAMICS_KEY`/`NATIONAL_FINANCIAL_ATTR` constants; `write_tick_state_to_graph` (writer, called from `domain/economics/tick/system/__init__.py:303`) and `write_national_financial_state_to_graph` (writer, called at `.../system/__init__.py:1977`). |
| `src/babylon/domain/economics/distribution/types.py` | 258 | `SurplusValueDistribution` (94-166: `total_surplus_produced`, `interest_payments`, `ground_rent`, `taxes_on_surplus`), `DebtAccumulation` (174-256: `accumulated_debt`). |
| `src/babylon/domain/economics/circulation/types/_legacy.py` | (relevant: 341-420, 1051-1140, 1341-1443) | `CirculationCrisisState` (`circuit_state.total_capital`/`.commodity_capital`, `latest_assessment.realization_crisis`/`.reproduction_crisis`, `.disproportionality`), `DisproportionalityCrisis` (`dept_i_output`, `dept_ii_output`). |
| `src/babylon/models/entities/contradiction.py` | 72 | `Contradiction`, `ContradictionFrame` — the pre-existing Pydantic models the registry's states are mapped onto (`_contradiction`, contradiction.py:994-1009). |
| `src/babylon/models/enums/topology.py` | — | `EdgeType.EXPLOITATION`/`.WAGES`/`.TENANCY`/`.INFLUENCES`; `NodeType.SOCIAL_CLASS`/`.FACTION`; `EdgeMode.EXTRACTIVE`. |
| `src/babylon/models/enums/balkanization.py` | — (33-51) | `ColonialStance` (`UPHOLD`/`IGNORE`/`ABOLISH`) — the national-axis pole map. |
| `src/babylon/models/enums/consciousness.py` | — (12-23) | `ContradictionType` (5 members: NATIONAL/CLASS/GENDER/IMPERIAL/ECOLOGICAL) — `_contradiction` only ever stamps IMPERIAL or CLASS (see §4 finding). |
| `src/babylon/models/enums/events.py` | — (79, 104) | `EventType.RUPTURE`, `EventType.LEVEL_TRANSITION`. |
| `src/babylon/kernel/event_bus.py` | — (33-60) | `Event` (frozen dataclass), `EventBus.publish`. |
| `src/babylon/kernel/graph_protocol.py` | 494 | `GraphProtocol` signatures: `update_node` (88), `update_edge` (152-170), `query_nodes` (258-276), `query_edges` (278-298), `get_graph_attr`/`set_graph_attr` (350-372). |
| `src/babylon/topology/graph.py` | — (660-702, 892-897) | Concrete `BabylonGraph.update_node`/`.update_edge` — **plain dict merge, no type coercion or quantization**, same finding as Territory's inventory. |
| `src/babylon/engine/services.py` | — (200-218, 380-400) | `ServiceContainer.opposition_registry`/`.coupling_graph` — wired by `create()` via `build_default_registry(rate_weight=defines.tension.principal_rate_weight)` / `build_default_coupling_graph(...)`, non-optional in production. |
| `src/babylon/engine/formula_registry.py` | — (100-103) | `"phi_absolute"` registered to `calculate_imperial_rent_gap` — `_stash_fundamental_theorem` resolves it via `services.formulas.get(...)`. |
| `src/babylon/config/defines/survival.py` | — (108-170) | `TensionDefines`: `principal_rate_weight` (152), `rupture_gap_threshold` (140), `regime_rate_epsilon` (161). |
| `src/babylon/config/defines/market.py` | — (93, 103) | `MarketDefines.scissors_balance_scale`/`.max_abs_log`. |
| `src/babylon/config/defines/capital_vol3.py` | — (34, 141) | `CapitalVol3Defines.debt_spiral_threshold`/`.credit_fragility_scale`. |
| `src/babylon/config/defines/capital_vol2.py` | — (61) | `CapitalVol2Defines.dept_i_share_required`. |
| `src/babylon/config/defines/politics.py` | — (547-556) | `PoliticsDefines.political_form_org_weight`. |
| `src/babylon/config/defines/economy_labor.py` | — (291) | `WorkingDayDefines.relative_hours_threshold` (shared with the `working_day` classifier — no second coefficient). |
| `src/babylon/data/defines.yaml` | — (203-205, 998-999, 1034, 1051, 1066, 1131, 438) | Player-editable values for every define above. |

**Not exercised by `ContradictionSystem` at all:** `formulas/contradiction.py`'s deprecated `calculate_contradiction_intensity` (105-154, uses `sqrt`/`**0.5` — a **second** libm-adjacent hazard, but dead code, never imported by contradiction.py or catalog.py — grep-confirmed). `src/babylon/engine/systems/territory_diagnostics.py`-style "not called by step()" caveats do not apply here — every module in the file map above is genuinely reached from `step()`.

**Sibling system, explicitly out of scope:** `src/babylon/engine/systems/contradiction_field.py` (`ContradictionFieldSystem`, position 19.0) is a *different* System that runs immediately after this one and is not part of this inventory; its test file `tests/unit/engine/systems/test_contradiction_field_system.py` is excluded from §7 for the same reason.

## 2. COMPUTATION CATALOG (execution order, `contradiction.py:176-186`)

`step()` calls exactly two top-level methods in order. Phase 2 is internally a nine-step sequence (`_step_registry`, 221-264); each of its steps is catalogued as its own numbered computation below, in the order they actually run.

### 1 — Fresh per-edge tension (`_write_edge_tensions`, contradiction.py:192-215)

- **(a)** For every EXPLOITATION/WAGES/TENANCY edge with two *active* endpoints, overwrite `tension` with the current wealth-asymmetry gap between the two poles — recomputed from scratch every tick, never accumulated (replaces a retired saturating add-only accumulator per the module docstring, lines 1-9).
- **(b)** For EXPLOITATION/WAGES: `tension = calculate_wealth_asymmetry_gap(src_wealth, tgt_wealth)` (contradiction.py:214, formula body at `formulas/contradiction.py:20-64`: `gap = min(1.0, max(0.0, abs(wealth_b - wealth_a) / (wealth_a + wealth_b)))`, `0.0` when the pole sum is `<= 1e-9`). For TENANCY: same formula but against `rent_level`, with a degenerate guard — `tension = 0.0 if rent <= _RENT_EPSILON (1e-9) else calculate_wealth_asymmetry_gap(src_wealth, rent)` (contradiction.py:206-211).
- **(c) Reads:** `SOCIAL_CLASS`/`TERRITORY`/other-endpoint `active` (bool, default `True`, skip if either endpoint inactive — contradiction.py:200-203); endpoint `wealth` (both edge types); target's `rent_level` (TENANCY only).
- **(d) Writes:** edge `tension` (EXPLOITATION/WAGES/TENANCY only), via `graph.update_edge(edge.source_id, edge.target_id, edge.edge_type, tension=tension)` (contradiction.py:215).
- **(e) Defines:** none (the formula is deliberately scale-free/coefficient-free, per `formulas/contradiction.py:1-15`).
- **(f) Events:** none.

### 2 — GraphInputs assembly (`_build_graph_inputs`, contradiction.py:360-490)

Pre-extracts every value the 19 opposition measures and the pole channel will read this tick, as one frozen `GraphInputs` dataclass (`catalog.py:197-352`, 20 fields). Ten sub-computations, in source order:

| # | Sub-computation | Formula / anchor | Reads |
|---|---|---|---|
| 2.1 | `exploitation_pairs`/`_id_pairs` | `(src.wealth, tgt.wealth)` per EXPLOITATION edge, active-endpoint-skipped via `_edge_wealths` (contradiction.py:869-883) | EXPLOITATION edges; endpoint `wealth`/`active` |
| 2.2 | `tenancy_pairs`/`_id_pairs` | `(src.wealth, tgt.rent_level)` per TENANCY edge (contradiction.py:397-407) — **no active-endpoint skip here** (unlike 2.1's `_edge_wealths` helper and unlike Phase 1's own TENANCY handling), a verbatim inconsistency | TENANCY edges; endpoint `wealth`/`rent_level` |
| 2.3 | `wage_value_pairs`/`_id_pairs` | `(w_paid, v_produced)` per active node where **both** attrs are present (presence-of-both selects paid-worker-class nodes with no node-type filter, contradiction.py:409-422) | all node types; `active`, `w_paid`, `v_produced` (transient, not `SocialClass` model fields — `sentinels/partition/checks.py:62-68`) |
| 2.4 | `market_balance` | `calculate_scissors_balance(price_log, scale=defines.market.scissors_balance_scale)` = `tanh(price_log/scale)` clamped `[-1,1]` (contradiction.py:427-430; `formulas/market.py:97-107`) — **libm: `math.tanh`** | graph attr `"market"` (`MARKET_ATTR`, written by `MarketScissorsSystem` @17.8, same-tick prior) |
| 2.5 | `rentier_share`, `debt_ratio` | `_county_money_ratios` (contradiction.py:659-725): ratio-of-sums `Σclaims/Σsurplus`, `(Σdebt/Σsurplus)/debt_spiral_threshold` | graph attr `tick_dynamics` → `.county_states[fips].surplus_distribution`/`.debt_accumulation` (**live Pydantic object instances**, read via `getattr`, not dict `.get`) |
| 2.6 | `commodity_overhang_share`, `realization_crisis_share`, `reproduction_crisis_share`, `disproportionality_imbalance` | `_circulation_layer_ratios` (contradiction.py:727-837): four ratio-of-sums over counties whose `circuit_state.total_capital > 0.0` | same `tick_dynamics.county_states[fips].circulation_state` object |
| 2.7 | `financialization_index` | `math.exp(clamped)` where `clamped = max(-bound, min(bound, fictitious_log))`, `bound = defines.market.max_abs_log` (contradiction.py:453-455) — **libm: `math.exp`** | graph attr `"market"` (`fictitious_log` key) |
| 2.8 | `national_balance` | `_national_chauvinism_balance` (contradiction.py:595-657): `1.0 - 2.0*(Σ influence*score / Σ influence)` over FACTION nodes weighted by summed INFLUENCES `influence_level` | INFLUENCES **edge attribute** `influence_level`; FACTION node `colonial_stance` (RESERVED-LINE, see below) |
| 2.9 | `political_labor_share` | verbatim passthrough | graph attr `"political_labor_share"` (written by `AllegianceSystem` @17.42, same-tick prior) |
| 2.10 | `political_form_positions` | `_political_form_positions` (contradiction.py:562-593): sorted `(org_id, self_organization, representation)` tuples, rows with a non-numeric pole skipped | graph attr `"political_form_org_positions"` (written by `DoctrineSystem` @14.7, same-tick prior) |
| 2.11 | `credit_fragility` | `_credit_fragility` (contradiction.py:839-867): `credit_state.credit_fragility / scale` | graph attr `"national_financial"` (`NATIONAL_FINANCIAL_ATTR`; **the value is `params.model_dump()`, a plain dict** — unlike `tick_dynamics`, this one IS dict-shaped) |
| 2.12 | `wealth_subsistence_ratio` | `_wealth_subsistence_ratio` (contradiction.py:492-532): `Σwealth/Σsubsistence_threshold` over active SOCIAL_CLASS nodes with a positive-summing `subsistence_threshold` | SOCIAL_CLASS node `wealth`, `subsistence_threshold` (written by `ImperialRentSystem`/`economic.py:598` @9.0, `SurvivalSystem`/`survival.py:156` @15.0 — **@15.0 is AFTER @18.0**, so within-tick this is a stale-by-one-tick read from `survival.py`, live same-tick only from `economic.py`) |
| 2.13 | `surplus_strategy_ratio` | delegated to `resolve_absolute_relative_surplus_ratio(graph, services, tick)` (contradiction.py:485; `working_day/resolver.py:124-175`): `labor_intensity_index * relative_hours_threshold / avg_weekly_hours` | `WorkingDayState` (via `resolve_working_day_state`, not read further here) |

- **(e) Defines used across 2.4-2.13:** `market.scissors_balance_scale` (0.5, `(0,5]`), `market.max_abs_log` (2.0, `(0,5]`), `capital_vol3.debt_spiral_threshold` (0.5, `(0,1]`), `capital_vol3.credit_fragility_scale` (0.001, `>0`), `capital_vol2.dept_i_share_required` (0.6667, `(0,1)`), `working_day.relative_hours_threshold` (40.0, `(0,168]`) — all read via `services.defines.*` at the call sites named above.
- **(f) Events:** none — this whole computation is pure read/derive.

### 3 — Fundamental Theorem stash (`_stash_fundamental_theorem`, contradiction.py:534-560)

- **(a)** Compute Φ = W_c − V_c and its labor-aristocracy readings per paid class/county node, reusing `wage_value_id_pairs` from computation 2 with zero new graph traversal.
- **(b)** `compute_fundamental_theorem` (`value_form.py:317-378`): per triple, if `v_produced > 0.0`: `phi_relative = (w_paid - v_produced) / v_produced` (`phi_class`, value_form.py:226-250); `labor_aristocracy_ratio = w_paid / v_produced` (`fundamental_theorem.py:16-38`, raises `ValueError` if `v_produced <= 0` — guarded by the `> 0.0` check first); `is_labor_aristocracy = w_paid > v_produced` (`fundamental_theorem.py:73-94`). `phi_absolute = phi_absolute_fn(w_paid, v_produced) = w_paid - v_produced` unconditionally (`fundamental_theorem.py:41-70`, no singularity, always computed) via the **hot-swapped** `services.formulas.get("phi_absolute")` (`formula_registry.py:103`), not the module's own default import.
- **(c) Reads:** `inputs.wage_value_id_pairs` (computation 2.3's output; zero new graph reads).
- **(d) Writes:** graph attr `fundamental_theorem` = `{entity_id: ClassPhiReading.model_dump()}` (contradiction.py:557-560).
- **(e) Defines:** none directly (the formulas are coefficient-free).
- **(f) Events:** none.

### 4 — Opposition registry step: 19 named contradictions (`registry.step`, contradiction.py:235; `opposition.py:479-526`)

- **(a)** For each of the 19 registered oppositions, run its bound measure against `GraphInputs`, derive `rate = gap - previous.gap` (0.0 if no prior state), derive `leading_pole` from the sign of `balance` (zero holds the previous pole — `opposition.py:604-611`), then pick exactly one `is_principal=True` among **non-shadow** states by `score = gap * (1 + rate_weight * |rate|)`, ties breaking lexicographically first-key (`opposition.py:576-602`).
- **(b) The 19 measures** (all in `catalog.py`; shared kernels: `_mean_asymmetry` at 359-393 — `gap = min(1.0,max(0.0, Σ|b-a| / Σ(a+b)))`, `balance = min(1.0,max(-1.0, Σ(b-a)/Σ(a+b)))`, pairs with pole-sum `< 1e-9` skipped, empty → `(0,0)`; `_ratio_reading` at 544-587 — `gap = x/(1+x)`, `balance = 2*gap-1`, `x<0` or `None` → `(0,0)`):

  | Key | Formula (anchor) | Shadow? | Antagonistic? |
  |---|---|---|---|
  | `capital_labor` | `_mean_asymmetry(exploitation_pairs)` (396-398) | no | **yes** |
  | `wage` | `_mean_asymmetry([(v,w) for w,v in wage_value_pairs])` (401-416) | no | no |
  | `tenancy` | `_mean_asymmetry([(t,r) for t,r in tenancy_pairs if r > 1e-9])` (419-424) | no | no |
  | `atomization` | `gap=atomization_index(solidarity_subgraph)` (`connectivity.py:153-178`, `(pieces-1)/(nodes-1)`), `balance=2*cylinder_balance-1` (427-440) | no | no |
  | `imperial` | same defect as `wage` (D5 shared-defect design) (487-497) | no | **yes** |
  | `price_value` | reads pre-derived `market_balance` (2.4); `gap=abs(balance)` (529-541) | no (CANONICAL since ADR078) | no |
  | `surplus_distribution` | `_ratio_reading(rentier_share)` (590-592) | no | no |
  | `debt_spiral` | `_ratio_reading(debt_ratio)` (595-597) | no | no |
  | `credit` | `_ratio_reading(credit_fragility)` (600-602) | no | no |
  | `financial` | `_ratio_reading(financialization_index)` (605-607) | no | no |
  | `national` | reads pre-derived `national_balance` (2.8); `gap=abs(balance)` (641-655) | **yes** | **yes** — **RESERVED-LINE, see below** |
  | `political_form` | blend of `political_labor_share` and `_political_form_org_balance(political_form_positions)` weighted by `political_form_org_weight` (658-725) | no (CANONICAL since P25 U10) | no |
  | `value_usevalue` | `_ratio_reading(wealth_subsistence_ratio)` (610-622) | **yes** | no |
  | `labor_laborpower` | same defect as `wage` (500-516) | **yes** | no |
  | `absolute_relative_surplus` | `_ratio_reading(surplus_strategy_ratio)` (625-638) | **yes** | no |
  | `circulation` | `2*commodity_overhang_share-1` (728-743) | **yes** | no |
  | `realization` | `2*realization_crisis_share-1` (746-761) | **yes** | no |
  | `reproduction` | `2*reproduction_crisis_share-1` (764-782) | **yes** | no |
  | `disproportionality` | `disproportionality_imbalance` directly, already `[-1,1]` (785-805) | **yes** | no |

  8 of 19 are `shadow=True` (measured every tick, excluded from principal scoring/frames/rupture — `opposition.py:255-267`); 11 compete for principal.
- **(c) Reads:** the entire `GraphInputs` snapshot from computation 2; `previous` states — read from graph attrs `opposition_states`/`shadow_opposition_states` (`_read_previous`, contradiction.py:346-358, merged since keys are registry-unique).
- **(d) Writes:** none directly — returns `tuple[OppositionState, ...]` (in-memory only until computation 10 stashes it).
- **(e) Defines:** `tension.principal_rate_weight` (10.0, `>=0`) — threaded into `OppositionRegistry(rate_weight=...)` at construction time (`services.py:385-387`), not re-read per tick.
- **(f) Events:** none.

**RESERVED-LINE (National Question, ADR171).** The `national` opposition (`pole_a="national-chauvinism"`, `pole_b="internationalism"`, `catalog.py:963-984`) and its feed `_STANCE_CHAUVINISM_SCORE` (`contradiction.py:159-163`: `UPHOLD→1.0`, `IGNORE→0.5`, `ABOLISH→0.0`, described as "a hardcoded categorical map, not a `GameDefines` coefficient") encode the Director-ruled National-Question line (doctrine tag `NATIONAL_CHAUVINISM`/`INTERNATIONALISM`, owner ruling 2026-07-15). Described here, not proposed for change.

### 5 — Stance intervention application (`_apply_interventions`, contradiction.py:266-277; `apply_interventions`, `coupling.py:174-225`)

- **(a)** Apply any queued player-verb "shoves" to this tick's balances, then clear the queue (consumed-once).
- **(b)** Per intervention, `deltas[target] += delta_balance`; then `new_balance = max(-1.0, min(1.0, state.balance + delta))` (coupling.py:216 — **note the `max(min(...))` clamp-nesting order, opposite of `catalog.py`'s `min(max(...))` convention used throughout computation 4**); `leading_pole` re-derived by the same zero-holds-previous rule (`_lead_after`, coupling.py:165-171). Raises `ValueError` if an intervention names an unknown key (fail-loud, not silently dropped).
- **(c) Reads:** graph attr `opposition_interventions` (`OPPOSITION_INTERVENTIONS_ATTR`).
- **(d) Writes:** graph attr `opposition_interventions` reset to `[]` (contradiction.py:276) — consumed-once. **No production writer exists for this attribute today** — "Written by verb/OODA systems (spec-071), read + CLEARED here (consumed-once). No producer writes it yet; unit tests set it directly" (contradiction.py:118-121, verbatim).
- **(e) Defines:** none.
- **(f) Events:** none.

### 6 — Coupling-direction correction (`_respect_coupling_direction`, contradiction.py:279-344)

- **(a)** Forbid a `transforms`-target opposition from holding the principal slot while the upstream opposition supplying its input reads absent (`gap==0.0 AND balance==0.0`, the catalog's canonical no-data reading). Runs only over the 11 non-shadow states.
- **(b)** `absent = {key : gap==0.0 and balance==0.0}` (319, **exact float equality** on values that are always exactly `0.0` by construction from the measures' own absent-branches, not from convergence); `blocked = {key : any(edge.kind=="transforms" and edge.source in absent for edge in coupling_graph.upstream_for(key))}` (322-329); if the current principal is blocked, re-rank the eligible pool (`key not in blocked and key not in absent`) by the same `_score` (330-340, `gap*(1+rate_weight*|rate|)`) and move `is_principal` to the top-ranked survivor (341-344). If no eligible candidate exists, the original principal stands.
- **(c) Reads:** `services.coupling_graph` (built once at container-creation time from the 15-edge `_DEFAULT_COUPLINGS` table, `catalog.py:1180-1237`, skipping any edge whose endpoint is unregistered); `services.defines.tension.principal_rate_weight`.
- **(d) Writes:** none — returns a new `tuple[OppositionState,...]` with at most one `is_principal` flag moved.
- **(e) Defines:** `tension.principal_rate_weight` (reused from computation 4).
- **(f) Events:** none.

### 7 — Frame derivation (`_write_frames`, contradiction.py:968-987)

- **(a)** Derive one global "principal vs. secondary" 2×2 frame from the (coupling-corrected) states.
- **(b)** `principal_state = the is_principal one` (falls back to `states[0]` if none — unreachable given computation 4's contract but present verbatim); `secondary_state = highest-`_score`-ranked non-principal state` (978-982, ties lexicographic), falling back to `principal_state` itself if only one state exists. Each mapped to `Contradiction` via `_contradiction` (994-1009): `type = IMPERIAL if key=="imperial" else CLASS` (997 — **every one of the other 18 opposition keys, including `national`/`financial`/`political_form`, is stamped `ContradictionType.CLASS`**, a coarse many-to-one map onto a 5-member enum, transcribed verbatim as a defect, not fixed), `principal_aspect = state.leading_pole`, `identity = 0.5` (hardcoded constant, not a define), `intensity = state.gap`, `aspect_balance = state.rate`, `form_of_struggle = EdgeMode.EXTRACTIVE` (hardcoded — no other `EdgeMode` value is ever produced here), `is_antagonistic = spec.antagonistic`.
- **(c) Reads:** the coupling-corrected `canonical` states (11 of the 19; shadow states never enter this computation).
- **(d) Writes:** graph attr `contradiction_frames = {"global": frame.model_dump()}` (987).
- **(e) Defines:** `tension.principal_rate_weight` (for the secondary-selection score).
- **(f) Events:** none.

### 8 — Rupture firing (`_maybe_rupture`, contradiction.py:1011-1033)

- **(a)** Fire a RUPTURE event iff the principal opposition's gap is both above threshold AND rising — never on hitting a static ceiling.
- **(b)** `if principal.gap > threshold and principal.rate > 0.0: publish(...)` (1022-1033) — **strict `>` on both**, no `>=`.
- **(c) Reads:** the coupling-corrected `canonical` states' principal.
- **(d) Writes:** none (event bus only).
- **(e) Defines:** `tension.rupture_gap_threshold` (0.9, `[0,1]`).
- **(f) Events:** `EventType.RUPTURE`, payload `{opposition, gap, rate}`.

### 9 — Regime classification (`_classify_regime` + `_capital_labor_field`, contradiction.py:1039-1127)

- **(a)** Classify this tick's fixed-point regime (reproduction / crisis / sublation) against the **`capital_labor`** opposition specifically (not necessarily the principal — the code deliberately diverges from the naive "classify the principal" reading, per the design note at 1057-1064), using the per-county mean EXPLOITATION-edge `tension` as the probe field for the level lattice's Aufhebung test. On the sublation branch, publish `LEVEL_TRANSITION`.
- **(b)** `field = _capital_labor_field(graph)` (1109-1127): `by_county[county_fips].append(edge.attributes["tension"])` over every EXPLOITATION edge whose source carries a `county_fips` — **reads back the SAME `tension` attribute Phase 1 just wrote this tick**, one of exactly two same-tick self-reads in the whole system (the other being `opposition_states`' own prior-tick read); `field[county] = mean(values)` — one division per county, no clamp (mean of already-`[0,1]`-clamped inputs cannot leave `[0,1]`). `counties = sorted(field)`; `lattice = spatial_lattice_for_counties(counties) if counties else None` (1073-1074, `levels.py:285-327`: `county≺state≺nation`, county→state = 2-digit FIPS prefix, state→nation = constant `"US"`). `level_index = level_index_for(spec.level_name)` falling back to `_COUNTY_LEVEL_INDEX=1` if unplaced or shallower (1075-1077). `probe_states = (target with is_principal forced True,)` (1082) — the classifier is always handed exactly one principal, by construction. `classify_regime` (`regime.py:41-86`): `rate=principal.rate`; `abs(rate)<=rate_epsilon → "reproduction"`; `rate<0.0 → "reproduction"` (falling gap = contained); else if `lattice is not None and lattice.aufhebung_of(level_index, [field]) is not None → "sublation"`; else `"crisis"`. `aufhebung_of` (`level.py:121-144`) walks levels above `level_index` looking for the first whose `sheaf(skeleton(field)) == skeleton(field)` within `1e-9` tolerance (`levels.py:82,143-147`) — i.e., the field is already flat once smoothed to that level (state or nation).
- **(c) Reads:** `states` (post-coupling-correction); EXPLOITATION-edge `tension` (self-read, see above) and source `county_fips`.
- **(d) Writes:** graph attr `dialectical_regime = {regime, opposition, rate}` (1086-1089).
- **(e) Defines:** `tension.regime_rate_epsilon` (1e-4, `>=0`).
- **(f) Events:** `EventType.LEVEL_TRANSITION` (sublation branch only), payload `{opposition, from_level, to_level, gap, rate}`.

### 10 — Stash canonical/shadow states (contradiction.py:256-263)

- **(a)** Persist this tick's states as the cross-tick + downstream-consumer channel.
- **(b)** Plain dict comprehension, no arithmetic: `{state.key: state.model_dump()}`.
- **(c) Reads:** the `canonical`/`shadow` tuples split at contradiction.py:245-246.
- **(d) Writes:** graph attr `opposition_states` (canonical, 11 keys) and, only when non-empty, `shadow_opposition_states` (8 keys) — `OPPOSITION_STATES_ATTR`/`SHADOW_OPPOSITION_STATES_ATTR`.
- **(e) Defines:** none.
- **(f) Events:** none.

### 11 — Per-node pole channel (`_step_pole_channel`, contradiction.py:889-966)

- **(a)** Phase-1-shadow (ADR070): derive and stash a per-node signed position on the two axes that have a `pole_measure` (`capital_labor`, `wage`) and, for nodes positioned on **both**, a derived class cell (`labor:exploited`, `capital:bribed`, etc.) — observational, nothing downstream adjudicates on it (module docstring 129-136, "``imperial`` stays in the graph-attr channel only — its Phase-1 proxy sigma is identical to ``wage``'s by construction").
- **(b)** `readings = registry.read_poles(inputs, previous)` (`opposition.py:528-574`): for each binding carrying a `pole_measure` — `_capital_labor_poles` (443-461, `catalog.py`): per EXPLOITATION edge, `balance = calculate_wealth_asymmetry_balance(labor_wealth, capital_wealth)` credited as `-balance` to the source and `+balance` to the target, averaged over multiple participations (`sum(values)/len(values)`); `_wage_poles`/`_imperial_poles`/`_price_value_poles` (464-484, 519-526) all alias the same per-node `calculate_wealth_asymmetry_balance(value, wage)`. `side` derives from `sigma`'s sign, zero holding the previous side (`_pole_side`, opposition.py:613-620). `_write_pole_shadow` (contradiction.py:923-966): for each node union of currently-positioned and previously-positioned ids, write `sigma_capital_labor`/`sigma_wage` (the axis's current sigma, or explicit `None` if the node lost the axis this tick — "honest null, not stale") and `derived_class_cell = cell_name({axis: side})` (`sentinels/partition/registry.py:49-63`: joins `"{capital_labor side}:{wage side}"`, `None` unless both axes present).
- **(c) Reads:** `inputs.exploitation_id_pairs`, `inputs.wage_value_id_pairs`; graph attr `pole_readings` (previous tick).
- **(d) Writes:** SOCIAL_CLASS node attrs `sigma_capital_labor`, `sigma_wage`, `derived_class_cell`; graph attr `pole_readings` (`POLE_READINGS_ATTR`).
- **(e) Defines:** none.
- **(f) Events:** none.

**Events emitted by the whole system: 2 distinct `EventType` values** — `RUPTURE` (computation 8) and `LEVEL_TRANSITION` (computation 9). Grep-confirmed no other `.publish(` call anywhere in contradiction.py.

## 3. TYPE INVENTORY

| Attribute | Node/edge type | Python model type | Domain | Category |
|---|---|---|---|---|
| `tension` | EXPLOITATION/WAGES/TENANCY edge | `Intensity` (`relationship.py:98-101`) | `[0,1]` | unit-interval; **written by this system, read back by this system same tick** (computation 9) |
| `wealth` | any node (read via edge endpoints) | `Currency` (`= Annotated[float, ge=0.0]`, same finding as Territory's inventory) | `[0,∞)` unbounded-above money-semantic | unbounded real |
| `rent_level` | TERRITORY | `Currency` | `[0,∞)` | unbounded real, money-semantic |
| `active` | any node | `bool` | `{T,F}`, default `True` | boolean gate |
| `w_paid`, `v_produced` | any node (paid-worker classes) | **plain `float`, transient — NOT declared `SocialClass` model fields** (`sentinels/partition/checks.py:62-68`); dropped on `WorldState.from_graph()` reconstruction | `[0,∞)` by convention, unenforced | transient bookkeeping |
| `subsistence_threshold` | SOCIAL_CLASS | `Currency` (`social_class.py:58,351`) | `[0,∞)`, default 5.0 | unbounded real |
| `county_fips` | SOCIAL_CLASS | `str \| None` (`social_class.py:426`) | 5-char FIPS or absent | optional identity string |
| `colonial_stance` | FACTION | `ColonialStance` (StrEnum, 3 members) | `{UPHOLD, IGNORE, ABOLISH}` | **Enum discriminant — RESERVED-LINE** |
| `influence_level` | INFLUENCES edge | `float \| None`, `ge=0.0, le=1.0` (`relationship.py:124-129`) | `[0,1]` or absent | optional unit-interval |
| `sigma_capital_labor`, `sigma_wage` | SOCIAL_CLASS (written here) | plain `float \| None`, transient — excluded field (`checks.py:79-85`), dropped on round-trip | `[-1,1]` (`Balance`) | transient, unread downstream (§5) |
| `derived_class_cell` | SOCIAL_CLASS (written here) | plain `str \| None`, transient — excluded field | one of 4 strings or `None` | transient enum-like string, unread downstream (§5) |
| `OppositionState.gap` | in-memory only | `Intensity` (`opposition.py:280`) | `[0,1]` | unit-interval |
| `OppositionState.balance` | in-memory only | `Balance = Annotated[float, ge=-1,le=1]` (`opposition.py:57-60`) | `[-1,1]` | signed unit-interval |
| `OppositionState.rate` | in-memory only | plain `float` (`opposition.py:282`) | unbounded (a gap-difference) | unbounded real |
| `OppositionState.leading_pole` | in-memory only | `Literal["a","b"]` | closed 2-set | **discriminant** |
| `OppositionState.is_principal` | in-memory only | `bool` | `{T,F}` | boolean discriminant |
| `market` (graph attr) | graph-level | plain `dict` (`MarketState.model_dump()`) | — | **byte-gate excluded** (see §7) |
| `tick_dynamics` (graph attr) | graph-level | plain `dict` whose `county_states` values are **live Pydantic model instances** (`CountyEconomicState`-shaped, read via `getattr`) | — | **object graph, not attribute-shaped; byte-gate excluded** |
| `national_financial` (graph attr) | graph-level | plain `dict` (`NationalTickParameters.model_dump()`) | — | dict-shaped, but still **byte-gate excluded** |
| `opposition_states`, `shadow_opposition_states`, `pole_readings`, `dialectical_regime`, `contradiction_frames`, `fundamental_theorem`, `opposition_interventions` (all written/read here) | graph-level | plain `dict` (`*.model_dump()` or raw list) | — | **all byte-gate excluded** (§7) |
| `Contradiction.identity` | in-memory only | `Intensity`, hardcoded `0.5` | `[0,1]` | unit-interval, **not computed — a stamped constant** |
| `Contradiction.form_of_struggle` | in-memory only | `EdgeMode`, hardcoded `EXTRACTIVE` | closed enum, one value ever produced | discriminant, degenerate range |
| `ContradictionType` (stamped) | in-memory only | StrEnum, 5 members, only 2 ever produced (`IMPERIAL`/`CLASS`) | closed 5-set, 2 reachable | discriminant, degenerate range |

**Currency flag — same finding class as Territory's.** `wealth`/`rent_level`/`subsistence_threshold` are Python "Currency" (plain unbounded float), unrelated to BSL's `i128` `Currency`. Every landed BSL pack routes around this by declaring money-like fields `int` (bare-scaled-Int workaround, ADR183). Not new here — reuse Territory's finding.

**Transient-attribute flag, three instances.** `w_paid`/`v_produced` (read), `sigma_capital_labor`/`sigma_wage`/`derived_class_cell` (written) are graph-only bookkeeping with no Pydantic model field and no round-trip survival — `WorldState.from_graph()` silently drops them (the "graph round-trip loses data" gotcha). A BSL port must decide whether these become genuine `deffield`-declared attributes (widening `CanonicalState`, a Director-escalation-gated Slice-4 move per the brief) or stay engine-side scratch with no BSL representation at all.

**Enum-field flag.** `colonial_stance` (`ColonialStance`, 3-valued) is exactly the kind of field ADR195/ADR196 now serve (`defenum`/`deffield ... enum`, declaration-order-is-ordinal, `field-of` refused for enum-declared fields — D102). Portable as a `deffield enum` **content decision**, not a language gap — but see §6: it feeds RESERVED-LINE content.

## 4. FLOAT-OP INVENTORY (execution order)

1. **Two-pole asymmetry gap/balance** (`formulas/contradiction.py:20-102`, used at contradiction.py:210,214 and throughout `catalog.py`'s per-opposition measures): `gap = min(1.0, max(0.0, abs(b-a)/(a+b)))`; `balance = min(1.0, max(-1.0, (b-a)/(a+b)))`; zero-guard `total <= 1e-9 → 0.0`. Pure rational arithmetic, no transcendentals.
2. **Weighted mean-of-sums** (`catalog.py:359-393`, `_mean_asymmetry`): `Σ|b-a| / Σ(a+b)` and `Σ(b-a)/Σ(a+b)`, each re-clamped `min(1.0,max(...))`/`min(1.0,max(-1.0,...))` at 391-392 — **provably redundant** given the per-pair formula already bounds each term (a defensive double-clamp, not a hazard, but worth noting as belt-and-suspenders inconsistent with the un-reclamped `_ratio_reading`).
3. **`math.tanh`** (`formulas/market.py:107`, called directly from `contradiction.py:427-430`): `tanh(price_log/scale)`, then `max(-1.0,min(1.0,...))` clamp against float-edge rounding. **Libm transcendental — nondeterminism hazard** per the standing cross-implementation-transcendentals rule.
4. **`math.exp`** (`contradiction.py:455`): `exp(clamped)` where `clamped = max(-bound, min(bound, fictitious_log))` (contradiction.py:454). **Libm transcendental — nondeterminism hazard.** Note the clamp here is `max(-bound, min(bound, x))` — **opposite nesting order** from item 1's `min(1.0, max(0.0, x))` convention; both are mathematically equivalent for well-ordered scalars but textually inconsistent within the same file.
5. **Ratio-of-sums family** (`_county_money_ratios` contradiction.py:659-725, `_circulation_layer_ratios` 727-837, `_credit_fragility` 839-867, `_wealth_subsistence_ratio` 492-532): plain division after summation, guarded by `<= 0.0 → None` (never a fabricated ratio). No transcendentals, no clamps (values are pre-bounded by their own construction, e.g. two `[0,1]` shares subtracted for `disproportionality_imbalance`).
6. **`_ratio_reading`** (`catalog.py:544-587`): `x/(1+x)` and `2*x/(1+x) - 1` — rational, monotone, no clamp needed (the algebra self-bounds to `[0,1)`/`(-1,1)` for `x>=0`); `ratio is None or ratio<0.0 → (0,0)`.
7. **Principal-contradiction score** (`opposition.py:576-578`, reused verbatim at `contradiction.py:990-992,336`): `gap * (1.0 + rate_weight * abs(rate))` — one multiply-add-multiply, `rate_weight=10.0` from defines.
8. **Intervention clamp** (`coupling.py:216`): `max(-1.0, min(1.0, balance+delta))` — **max-of-min nesting**, matching item 4's convention, opposite item 1/2's min-of-max convention. **Two clamp-nesting conventions coexist across this computation graph** (min-of-max: `formulas/contradiction.py`, `catalog.py`'s measures; max-of-min: `contradiction.py:454`, `coupling.py:216`) — the same "two clamp implementations for a conceptually-identical bound" finding class as Territory's `[0,1]` heat clamp, here spread across three files rather than one.
9. **`_capital_labor_field`** (`contradiction.py:1109-1127`): `sum(values)/len(values)` per county — a plain arithmetic mean of already-`[0,1]`-bounded `tension` values, no re-clamp (correctly redundant to omit one here).
10. **`_fields_equal`** (`levels.py:143-147`, used inside `aufhebung_of`'s resolution test): elementwise `abs(a-b) <= 1e-9` tolerance equality on the level-lattice closure — a genuine floating-point tolerance comparison, not exact equality.
11. **Exact float equality** (`contradiction.py:319`): `state.gap == 0.0 and state.balance == 0.0` as the "absent" sentinel for the coupling-direction correction — safe only because every measure's absent-branch returns the literal `0.0`/`0.0` (never a computed value that merely rounds to zero), verified by inspection of all 19 measures in computation 4.

**Real→Int demotions: zero.** Grep-confirmed across `contradiction.py`, `catalog.py`, `opposition.py`, `coupling.py`, `regime.py`, `value_form.py`, `levels.py`, `connectivity.py`, `formulas/fundamental_theorem.py`, `formulas/market.py`, `formulas/contradiction.py` — no `int(...)` cast anywhere in this system's live computation path (contrast with Territory's two, since-resolved-by-`floor` sites).

**Bare non-integer literals** (a BSL parser concern per the "no bare non-integer literal" rule): `0.0`, `1.0`, `-1.0`, `1e-9`, `10.0`, `0.5`, `2.0`, `1e-4`, `40.0`, `0.9`, `0.001`, `0.6667`, `0.4` all appear as bare literals throughout the computation chain (both in defines-sourced coefficients and in structural constants like the `0.5`/`1.0` identity/score terms) — same class of finding as Territory's, at much higher volume given 19 oppositions' worth of formula bodies.

**Deprecated dead code, not on this system's live path but co-located:** `calculate_contradiction_intensity` (`formulas/contradiction.py:105-154`) computes `divergence * (1 + sqrt(centrality_a*centrality_b)) * sensitivity` — a **second** libm-adjacent primitive (`** 0.5`, equivalent to `sqrt`) — but is imported by neither `contradiction.py` nor `catalog.py` (grep-confirmed). Flagged for completeness; excluded from the libm-hazard count since it never executes in this system's tick path.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 18.0** (`contradiction.py:170`), confirmed against `_SYSTEM_CLASSES` (`simulation_engine.py:328-363`): `... → SovereigntySystem(17.5) → MarketScissorsSystem(17.8) → ContradictionSystem(18.0) → ContradictionFieldSystem(19.0) → ...`. 26 systems run before it this tick.

- **Reads from same-tick prior systems:**
  - `wealth` — read on every node; every prior system that mutates class wealth (VitalitySystem@1, ImperialRentSystem@9, MarketScissors@17.8, etc.) feeds this same-tick.
  - `rent_level` — written by `TerritorySystem`@2.0 (eviction rent spike) and `MarketScissorsSystem`? — no, confirmed written only by `territory.py:251` — same-tick prior.
  - `subsistence_threshold` — written same-tick by `ImperialRentSystem`/`economic.py:598` @9.0 (prior); also written by `SurvivalSystem`/`survival.py:156` @15.0 — **also prior to 18.0**, so both writers are same-tick-prior; `DecompositionSystem`@11.0 also writes it (`decomposition.py:255`).
  - `w_paid`/`v_produced` — written by `ImperialRentSystem`/`economic.py:529-530` @9.0 (the wages phase), same-tick prior, module docstring's own claim confirmed by anchor.
  - graph attr `"market"` — written by `MarketScissorsSystem`@17.8 (`market_scissors.py:199`, `MARKET_ATTR="market"` at line 64), the **immediately preceding** system (17.8 → 18.0).
  - graph attr `"political_labor_share"` — written by `AllegianceSystem`@17.42 (`allegiance.py:204`), same-tick prior.
  - graph attr `"political_form_org_positions"` — written by `DoctrineSystem`@14.7 (`POLITICAL_FORM_POSITIONS_ATTR="political_form_org_positions"`, `doctrine.py:80`). Position ordering (14.7 < 18.0) makes this a same-tick-prior read as a matter of `step()` call order — **but `doctrine.py:76-79`'s own comment claims "(one tick stale by pipeline position: I-ORD compliant)" verbatim**, which contradicts the position arithmetic on its face. Not resolved in this pass (would require reading `doctrine.py`'s full write path, out of this system's own file map); transcribed as an open discrepancy rather than asserted either way.
  - graph attr `"tick_dynamics"` — written by `TickDynamicsSystem`@4.0 (`domain/economics/tick/system/__init__.py:112,303` calling `graph_bridge.write_tick_state_to_graph`), same-tick prior (position 4.0, far earlier in the tick).
  - graph attr `"national_financial"` — written by the same `TickDynamicsSystem`'s financial block (`.../system/__init__.py:1977`, `write_national_financial_state_to_graph`), same-tick prior.
  - graph attrs `opposition_states`/`shadow_opposition_states`/`pole_readings` — **self-read, LAST tick's own write** (`_read_previous`, `_read_previous_poles`) — the only two genuine cross-tick channels in the system besides the edge `tension` self-read below.
  - EXPLOITATION-edge `tension` — **self-read, THIS tick's own write** (computation 9's `_capital_labor_field`), the system reading back its own Phase-1 output within the same `step()` call.

- **Writes consumed downstream (same tick or later):**
  - `opposition_states` (graph attr) — read by `ImperialRentSystem`/`economic.py:776` @9.0, `ConsciousnessSystem`/`ideology.py:153-154` @17.0, `FascistFactionSystem`/`reactionary.py:97` @17.4, `StruggleSystem`/`struggle.py:730-731` @16.0 — **all four positions are BEFORE 18.0**, so every one of these reads is necessarily **last tick's** snapshot, exactly as the module docstring states (lines 33-36).
  - `dialectical_regime` (graph attr) — read by `FascistFactionSystem`/`reactionary.py:211` @17.4 (last tick, same logic as above — module comment at `reactionary.py:24-26` explicitly confirms "reads … AFTER this system @17.4, so on tick N this reads tick N-1's"), `FieldDerivativeSystem`/`field_derivative.py:99` @20.0 (**this** tick, since 20.0 > 18.0), `EdgeTransitionSystem`/`edge_transition/_legacy.py:477-478` (position not confirmed but the module is a downstream consumer regardless).
  - `fundamental_theorem` (graph attr) — read only by `projection/economy.py:36` — a **projection/observe() layer read, not a downstream engine System**.
  - EXPLOITATION/WAGES/TENANCY `tension` (edge attr) — read by `engine/headless_runner/bridge.py:959`, `engine/optimization/backends/{headless,in_memory}.py` — **observer/optimization-harness reads, not downstream engine Systems**; also self-read by computation 9 (above).
  - `sigma_capital_labor`, `sigma_wage`, `derived_class_cell` — **grep-confirmed zero readers anywhere in production** (`src/babylon/engine/`, `src/babylon/domain/`, `src/babylon/sentinels/`) — dead output. The module's own docstring claims "the partition sentinel reads them" (line 899), but the actual sentinel (`sentinels/partition/checks.py::analyze_partition`) reads `pole_readings` (the graph attr), **not** these three node attrs — the docstring's claim does not hold for the node-attribute write specifically; only the graph-attr `pole_readings` channel has a real (offline, non-engine) consumer.
  - `pole_readings` (graph attr) — consumed by `tools/partition_probe.py` → `sentinels/partition/checks.py::analyze_partition` (`checks.py:77-150`), an **offline CLI tool**, not a tick-time engine System.
  - `contradiction_frames`, `opposition_interventions` (post-clear) — grep-confirmed zero downstream engine-System readers.
  - `shadow_opposition_states` — grep-confirmed zero readers anywhere (not even `projection/`).

- **Context/service usage with no BSL equivalent:**
  - `services.opposition_registry` (`OppositionRegistry[GraphInputs]`) — a stateful, non-optional service object wired once at container-creation (`services.py:380-388`) carrying 19 closures + validated nesting/governance graphs. No BSL analog: BSL rules read/write the graph directly, they do not consult an injected named-object registry.
  - `services.coupling_graph` (`CouplingGraph`) — likewise a pre-built typed-morphism graph over the *opposition keys themselves* (not game entities), consulted by computation 6. No BSL analog.
  - `services.formulas.get("phi_absolute")` — the hot-swappable formula-registry DI pattern; BSL has no notion of runtime-injectable formula implementations (its intrinsics are fixed at content-load time).
  - `services.defines.tension`/`.market`/`.capital_vol3`/`.capital_vol2`/`.politics`/`.working_day` — ordinary coefficient reads, portable as `defconst`/`deffield` the same way Territory's were.

- **Dormancy on canonical scenarios** (`tools/regression_scenarios.py`, all 12 `create_*_scenario` factories):
  - **`national` opposition: permanently absent, by construction.** Grep-confirmed zero `BalkanizationFaction`/`NodeType.FACTION`/`EdgeType.INFLUENCES`/`colonial_stance` references anywhere in `tools/regression_scenarios.py` or `engine/scenarios/_legacy.py` — every canonical scenario reads `national_balance=None`, so the `national` measure always returns `(gap=0, balance=0)`. Matches the catalog's own docstring claim (lines 78-80) verbatim, now independently verified.
  - **Vol II circulation bindings (`circulation`/`realization`/`reproduction`/`disproportionality`): permanently absent on canonical scenarios**, per the catalog's own docstring (lines 111-134, "until Vol II data hydration — task #46 — lands"); not independently re-verified here beyond citing the module's own claim, since the underlying `tick_dynamics.county_states` population is out of this system's own code.
  - **The regime classifier's SUBLATION branch is structurally unreachable on every canonical `qa:regression` scenario.** `_capital_labor_field` (computation 9) requires `county_fips` on the EXPLOITATION edge's source node; grep-confirmed **zero** `county_fips=` call-site arguments anywhere in `tools/regression_scenarios.py` (all 12 factories are, in the file's own words, "county-free scenario[s]: no territory carries `county_fips`" — `tools/regression_scenarios.py:406,419,432,445,813`). So `field={}` always, `counties=[]` always, `lattice=None` always (contradiction.py:1073-1074's own guard), and `classify_regime`'s `lattice is not None` gate for sublation (`regime.py:84`) can never pass — every canonical scenario's `dialectical_regime` reads only `"reproduction"` or `"crisis"`, never `"sublation"`, and `EventType.LEVEL_TRANSITION` can never fire on the qa:regression estate. (The full county-bearing canonical 520-tick run, distinct from the 12 qa:regression fixtures, is out of scope for this grep — it may exercise sublation; not verified here.)
  - **Phase 1 (edge tension) and the `capital_labor`/`wage`/`imperial`/`tenancy` oppositions ARE live** on canonical scenarios — `tools/regression_scenarios.py`'s own inline documentation strings ("tension is written only on EXPLOITATION/WAGES/TENANCY edges", lines 699,718,727,1213,1232,1241,1587...) confirm these edge types and the tension write are exercised and asserted against in the golden baselines.
  - **The Vol III money oppositions (`surplus_distribution`/`debt_spiral`/`credit`/`financial`) are non-shadow** (compete for principal) and, per `defines.yaml:1051`'s own comment ("Live since U5.10 … `ContradictionSystem._county_money_ratios`"), are treated as production-live rather than permanently dormant — not independently re-verified against `TickDynamicsSystem`'s actual canonical-scenario output in this pass (would require reading the ~600-line `tick/system/__init__.py`, out of this system's own file map).

## 6. BLOCKER ASSESSMENT

| Computation | Verdict | Detail |
|---|---|---|
| 1 — Fresh per-edge tension (contradiction.py:192-215) | **BLOCKED — edge-attribute storage** | `update-edge` (the only verb that could write `tension`) parses and typechecks but is **refused at evaluation for lack of substrate storage**, not merely an unserved evaluator dispatch: `GraphSubstrate` keys an edge to one bare `f64` "strength" and has no field storage at all (`rust/crates/babylon-bsl/src/structural_verbs.rs:16-26,387-398,709-716`, verified myself, error text quoted verbatim: "has no substrate storage … Widening that state widens the canonical `state_hash` field set, which is a declared Phase-2/substrate decision"). This is a **deeper** gap than Territory's query-evaluation blocker: even a future evaluator fix cannot serve this verb until the substrate itself grows edge-attribute storage — a hash-relevant, Constitution-III.7-gated decision. The reads feeding the formula (endpoint `wealth`/`rent_level`, both NODE attributes) ARE servable by the landed Slice-1 query lane (`fold`/`field-of` over `neighbors`); only the WRITE is blocked. |
| 2 — GraphInputs assembly (contradiction.py:360-490) | **BLOCKED — mixed** | Sub-items 2.1-2.3 (node-attribute pair extraction over EXPLOITATION/TENANCY edges and all-node presence-filtering) are individually expressible via Slice-1 `fold`/`neighbors`/`exists`. Sub-item 2.4 (`market_balance`, needs `math.tanh`) and 2.7 (`financialization_index`, needs `math.exp`) both use **declarable intrinsics** (`exp`, `log` landed per the brief; `tanh` is **not** in the declared intrinsic set `{exp, log, floor}` — a **further, unnamed gap**: `tanh` has no BSL intrinsic today, so even with edge/graph-attr storage solved, computation 2.4 cannot be expressed until `tanh` is declared or the formula is rewritten in terms of `exp`/`log` — feasible algebraically (`tanh(x) = (e^{2x}-1)/(e^{2x}+1)`) but not free, and a port-question either way). Sub-items 2.5, 2.6, 2.8, 2.9, 2.10, 2.11 all read **graph-level attributes** (`tick_dynamics`, `national_financial`, `"market"`, `political_labor_share`, `political_form_org_positions`) — BSL has **no graph-level-attribute read primitive at all** in the landed surface (query lane Slice 1 is node/edge/neighbor-shaped; no analog of Python's `graph.get_graph_attr`). `tick_dynamics` additionally carries **live Pydantic object instances**, not attribute-shaped data, which no future graph-attribute primitive could ingest without a data-model redesign upstream of BSL entirely. |
| 3 — Fundamental Theorem stash (contradiction.py:534-560) | **BLOCKED — graph-attribute write** | The arithmetic itself (subtraction, division, comparison) is trivial and portable; the write target (`fundamental_theorem` graph attr, a dict keyed by arbitrary entity id) has no BSL storage primitive (same class of gap as `update-edge`, but for graph-level scalars/maps rather than edges). |
| 4 — Opposition registry step, 19 measures (contradiction.py:235; opposition.py:479-526) | **BLOCKED — no ranked-registry primitive** | Each of the 19 individual `_*_measure` functions is, in isolation, close to expressible as a Slice-1 `fold`/`select` expression **once its own input-storage gaps (above) are resolved**. The registry's own machinery — derive `rate` from a stored *previous* per-key state, hold `leading_pole` under a zero-holds-previous tie rule, and rank **19 named, heterogeneous bindings** by a computed score to select exactly one principal — has no BSL primitive: there is no "named binding set" construct, no cross-key `select-max`-over-symbolic-keys (Slice 1's `select-max` operates over a **node set**, not over 19 statically-named, differently-shaped Python closures). Representing this would require either (a) reifying all 19 oppositions as rows of one new node type with a common `gap`/`balance`/`rate` schema — a genuine architecture decision, or (b) inventing a new BSL primitive. Named here as an open **architecture question**, not a simple missing-verb gap. |
| 5 — Stance intervention application (contradiction.py:266-277) | **BLOCKED — same registry-shape gap as 4**, plus the input channel (`opposition_interventions`) is itself a graph-attribute list with no live production writer today (a genuine WS1/#502 ledger item independent of BSL). |
| 6 — Coupling-direction correction (contradiction.py:279-344) | **BLOCKED — no typed-morphism-graph primitive** | `CouplingGraph.upstream_for`/`.downstream_of` traverse a *graph of opposition keys* (five typed relations: `feeds`/`constrains`/`transforms`/`contains`/`antagonizes`) that is itself built once from a **content-declared, non-game-entity** edge table (`catalog.py:1180-1237`). Even granting a hypothetical reification of the 19 oppositions as nodes (per row 4's option (a)), this typed re-ranking pass is a second, distinct piece of machinery layered on top, with no existing BSL analog. |
| 7 — Frame derivation (contradiction.py:968-987) | **BLOCKED — depends on rows 4 and 6** | The mapping onto `Contradiction`/`ContradictionFrame` (a write of a nested Pydantic object, not a flat scalar field) has no BSL storage shape at all — BSL fields are scalar-typed (`int`/`bool`/`currency`/`probability`/`intensity`/`coefficient`/`enum`), not nested-record-typed. |
| 8 — Rupture firing (contradiction.py:1011-1033) | **BLOCKED — event ledger + upstream (row 4/6) dependency** | `emit` exists in BSL's effect-verb surface, but `TickReport` carries no event log (per the brief) — every `EventType` emission is a WS1 (#502) ledger row, **unpinnable by any current goldens**, independent of whether the *guard condition* itself (`gap>threshold and rate>0.0`, both plain scalar comparisons) is trivially portable once `principal` is derivable at all. |
| 9 — Regime classification (contradiction.py:1039-1127) | **BLOCKED — no level-lattice primitive** | `classify_regime`/`LevelLattice.aufhebung_of`/`spatial_lattice_for_counties` implement a genuinely different piece of mathematics (Lawvere skeleton/sheaf closure, resolution-tolerance equality over a keyed field aggregated up a `county≺state≺nation` chain) with no BSL analog whatsoever — not a missing verb but an entirely un-modeled construct. Additionally structurally dormant on every canonical `qa:regression` scenario for the sublation branch specifically (§5), so even a hand-built conformance fixture for it cannot be harvested from the canonical estate. |
| 10 — Stash canonical/shadow states (contradiction.py:256-263) | **BLOCKED — graph-attribute write**, same class as row 3. |
| 11 — Per-node pole channel (contradiction.py:889-966) | **PORTABLE WITH D-RECORD, modulo row 1's edge-storage gap for its inputs** | The arithmetic (`calculate_wealth_asymmetry_balance`, sign-based side selection, `cell_name`'s string join) is simple and the WRITE TARGETS are genuine node attributes (`sigma_capital_labor`, `sigma_wage`, `derived_class_cell`) — servable by `update-node` (landed) once the EXPLOITATION-edge wealth reads are available via the Slice-1 query lane. The `pole_readings` graph-attr stash (the cross-tick channel) hits the same graph-attribute-write gap as row 3/10. Since this whole channel is **zero-consumer dead output in production today** (§5), a port could legitimately choose to drop it rather than solve its storage gap — a D-record either way (port-as-is transcribes it; a declared deviation drops it, citing the zero-reader finding as justification). |

**Summary verdict: nothing in `ContradictionSystem` is `PORTABLE NOW`.** Row 11 is the closest to portable (arithmetic-trivial, node-attribute-shaped, dead-output-so-droppable), but even it depends on row 1's edge-attribute-storage resolution for its inputs. The dominant blockers are (i) the edge-attribute substrate-storage gap (row 1, deeper than any query-lane slice), (ii) a family of graph-level-attribute read/write gaps with no BSL primitive at all (rows 2,3,5,10), and (iii) two genuinely un-modeled pieces of machinery — the ranked 19-binding registry (row 4) and the typed coupling graph + level-lattice regime classifier (rows 6,9) — that are architecture questions, not missing-verb gaps. Unlike Territory's single named unblock ("the query-evaluation train"), this system needs at minimum: an edge-attribute-storage decision, a graph-level-attribute storage decision, a `tanh` intrinsic (or an `exp`/`log` rewrite), and a genuinely new design for how a small ranked registry of named contradictions is expressed as BSL content — a materially larger and differently-shaped unblock than any system inventoried so far in this train.

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_contradiction_system.py` | 874 | **Primary conformance oracle**, 51 test methods. Pins the fresh-tension contract, `opposition_states` stashing, frame derivation, rupture firing (threshold-AND-rising), the pole channel, and `FUNDAMENTAL_THEOREM_ATTR`/`OPPOSITION_INTERVENTIONS_ATTR` behavior. Imports the real `ContradictionSystem`, `ServiceContainer`, `TickContext` — genuine tick-level integration coverage of the system under test. |
| `tests/unit/dialectics/test_catalog.py` | 912 | 97 test methods. The 19 opposition measures' own contract tests (gap/balance formulas, shadow-vs-canonical partition, `_DEFAULT_COUPLINGS` edges) — the deepest oracle for computation 4's per-opposition math. |
| `tests/unit/dialectics/test_opposition_registry.py` | 521 | 49 test methods. `OppositionRegistry.step`/`.read_poles` contract: principal scoring, tie-breaking, governance/nesting validation, shadow exclusion. |
| `tests/unit/dialectics/test_coupling.py` | 318 | `CouplingGraph` construction, `contains`-edge auto-derivation, `apply_interventions`' clamp + tie rule. |
| `tests/unit/dialectics/test_levels.py` | 575 | `LevelLattice`/`spatial_lattice_for_counties`/Aufhebung resolution-tolerance tests — computation 9's deepest oracle. |
| `tests/unit/dialectics/test_level_lattice.py` | 108 | The generic `LevelLattice` primitive (level.py), one layer below `test_levels.py`. |
| `tests/unit/dialectics/test_regime.py` | 85 | 3 test methods, `classify_regime`'s three-branch contract directly. |
| `tests/unit/dialectics/test_value_form.py` | 496 | `compute_fundamental_theorem`/`phi_class`/the Φ tri-decomposition — computation 3's oracle. |
| `tests/unit/dialectics/test_connectivity_instance.py` | 259 | `atomization_index`/`connectivity_cylinder` — the `atomization` measure's oracle. |
| `tests/unit/dialectics/test_pole_readings.py` | 220 | `PoleReading`/`_pole_side` tie-inertia contract — computation 11's oracle. |
| `tests/unit/dialectics/test_composition.py` | 94 | Opposition composition combinators (product/sum) — not directly exercised by `ContradictionSystem`'s own bindings (none of the 19 use `composition!=""`), lower relevance. |
| `tests/unit/dialectics/test_contract.py` | 90 | Cross-module contract/interface tests. |
| `tests/unit/dialectics/test_edge_mode_category.py` | 87 | `EdgeMode` category-theory laws — tangential (the system stamps one hardcoded `EdgeMode.EXTRACTIVE`, never varies it). |
| `tests/unit/dialectics/test_fractal.py` | 149 | Pole-nesting fractal-composition tests — none of the 19 live bindings use `binding_a`/`binding_b` nesting, so this exercises unused-by-this-system machinery. |
| `tests/unit/dialectics/test_scale.py` | 232 | `ScaleAdjunction` — the level-lattice closure's own primitive, one layer below `test_levels.py`. |
| `tests/unit/engine/systems/test_contradiction_regime.py` | 85 | 3 test methods, `_classify_regime`'s engine-level integration (vs. `test_regime.py`'s pure-function level). |
| `tests/unit/engine/systems/test_contradiction_money_inputs.py` | 267 | 13 test methods, computations 2.5/2.11 (`rentier_share`/`debt_ratio`/`credit_fragility`) at the system level. |
| `tests/unit/engine/systems/test_contradiction_circulation_inputs.py` | 349 | 10 test methods, computation 2.6 (Vol II circulation ratios) at the system level. |
| `tests/unit/engine/systems/test_contradiction_coupling_rank.py` | 197 | 9 test methods, computation 6 (`_respect_coupling_direction`) specifically. |
| `tests/unit/sentinels/test_partition_sentinel.py` | 220 | 15 test methods, `analyze_partition`/`cell_name` — the offline pole-channel consumer's own contract, not the engine system. |
| `tests/integration/test_lawverian_contradiction_bridge.py` | 162 | Bridged-runner integration: confirms the graph-attribute cross-tick channel survives the headless-runner's per-tick `TickContext` recreation (module docstring's own load-bearing claim, lines 25-36). |
| `tests/unit/formulas/test_contradiction.py` | 96 | `calculate_wealth_asymmetry_gap`/`_balance` (+ the deprecated `calculate_contradiction_intensity`) at the pure-formula level. |
| `tests/property/dialectics/test_wealth_asymmetry_invariance.py` | 54 | Property test: numeraire-invariance of the gap/balance formula (multiply both wealths by k>0, gap/balance unchanged) — a genuine **behavioral contract** independent of implementation, directly reusable as a BSL-port law. |
| `tests/property/dialectics/test_value_form_invariance.py` | 65 | Property test: value-form adjunction round-trip laws. |
| `tests/property/dialectics/test_intervention_laws.py` | 50 | Property test: `apply_interventions` clamp/tie-rule laws. |
| `tests/property/dialectics/test_composition_laws.py` | 77 | Property test: opposition composition laws (product/sum) — same low-relevance caveat as `test_composition.py`. |
| `tests/property/dialectics/test_cylinder_laws.py` | 147 | Property test: `AdjointCylinder` laws (feeds `atomization`'s `connectivity_cylinder`). |
| `tests/property/dialectics/test_galois_laws.py` | 129 | Property test: the underlying Galois-connection laws the cylinder/adjunction machinery is built on. |

**qa:regression byte-gate coverage — the single most important caveat for this system.** `tools/regression_test.py::graph_content_hash` (924-964) hashes every node/edge attribute of the `WorldState→graph` projection but **explicitly excludes graph-level attributes** ("Graph *metadata* (`g.graph`: economy, event log, opposition states) is also excluded", line 940, verified myself). Of `ContradictionSystem`'s seven write channels, **only the edge `tension` write (computation 1) is byte-gate covered** — `opposition_states`, `shadow_opposition_states`, `pole_readings`, `dialectical_regime`, `contradiction_frames`, `fundamental_theorem`, and `opposition_interventions` all live on `g.graph` and are **structurally invisible to `qa:regression`**. The three node-attribute writes from computation 11 (`sigma_capital_labor`/`sigma_wage`/`derived_class_cell`) are additionally excluded from the hash a second way: they are `SOCIAL_CLASS_EXCLUDED_FIELDS` (`world_state.py:79-90`), dropped when `WorldState.from_graph()` reconstructs the state that `graph_content_hash` re-serializes, so they never reach `to_graph()` a second time either. **A green `qa:regression` run today provides real evidence only for Phase 1 (edge tension); it says nothing about the correctness of the entire opposition-registry/coupling/frame/rupture/regime apparatus.** The genuine conformance oracle for that apparatus is `tests/unit/engine/systems/test_contradiction_system.py` plus the `tests/unit/dialectics/` and `tests/property/dialectics/` suites — a port's own `.bscn` conformance fixtures should be modeled on those, not on the byte-gate.

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`), read-only, with fresh anchors.
The eleven-computation catalog, the clamp-nesting census and the byte-gate caveat in §7 are
excellent and stand. The **BLOCKED verdict survives**, but it is re-based: one whole family
of its blockers has a ratified answer the inventory never read, one blocker is broader than
the row it sits in, and one row is filed as a missing intrinsic when it is a doctrine gate.

1. **CORRECTION — rows 2, 3, 5 and 10's "BSL has no graph-level-attribute read primitive at
   all" / "no BSL storage primitive" miss §3.6's ratified answer, which names this system's
   own registry by name.** `grep` over this inventory returns **zero** occurrences of "3.6",
   "carrier" or "ceiling" — the reference material read for the blocker adjudication was
   `evaluator.rs`, `structural_verbs.rs`, `declarations.rs` and §2.x, and the one chapter
   that disposes of this exact gap was not among them.
   `docs/reference/bsl-language.rst:2650-2689` is a draft ruling (Phase 1 review, R9 chapter
   C3) whose opening sentence is *"Graph-scope state is ordinary node state on a declared
   carrier node type"*, and whose motivating list at `:2652-2657` reads *"Twenty-two of the
   thirty-four frozen systems read or write state that belongs to the graph rather than to
   any node — **the opposition registry**, the market-scissors axis, the electoral registers
   … through `graph.graph[...]`, `set_graph_attr` or `context.persistent_data`"*. The
   mechanism is a `NodeType` member with manifest `:ceiling` 1, read with `field-of` and
   written with **`update-node`** — the verb that is landed and that every pack already uses
   — and the ruling explicitly records the rejected `:global`/`update-global` alternative
   (`:2678-2685`) so it is not re-proposed. Nor is it unbuildable pending Slice 2's `the`:
   `tick.rs:159-181`'s `subject_type_of` derives a rule's subject `NodeType` from its
   `:field` namespace and `run_tick` iterates `graph.nodes(&subject_type)`
   (`tick.rs:536-538`), so a rule anchored on a ceiling-1 carrier runs over exactly one
   subject with no accessor at all; cross-type reads are `(fold … (nodes NodeType/CARRIER)
   (field-of it …))` with `nodes` in `SERVED_QUERY_HEADS` (`evaluator.rs:527`); and the
   carrier `NodeType` is content-declarable today via `(defvocabulary NodeType …)`
   (`scenario.rs:389-395`, `load_defvocabulary` `:811-850`; landed in
   `content/scenarios/organization-foundation.bscn:41`).
   The ruling also disposes of row 4's own option (a) in this system's favour: `:2684-2688`
   states per-sovereign and per-county registers are *"ordinary nodes of ordinary types,
   reached by ordinary queries"* — i.e. reifying the nineteen oppositions as rows of one
   node type is not an invented workaround, it is the ruled shape. `opposition_states`,
   `shadow_opposition_states`, `contradiction_frames`, `fundamental_theorem`,
   `dialectical_regime` and `pole_readings` are all per-key registers of exactly that form.
   **What survives of rows 2/3/5/10:** the `tick_dynamics` half alone — its `county_states`
   values are **live Pydantic object instances** read via `getattr`
   (`contradiction.py:659-725`, `:727-837`), which no carrier `deffield` set can ingest —
   and the nested-record shape of `Contradiction`/`ContradictionFrame` in row 7. Those are
   real. "No primitive at all" is not.

2. **CORRECTION — row 1's edge-attribute finding is the inventory's strongest, and it is
   scoped too narrowly to Phase 1's WRITE.** Row 1 says the reads feeding the Phase-1 formula
   "(endpoint `wealth`/`rent_level`, both NODE attributes) ARE servable by the landed Slice-1
   query lane; only the WRITE is blocked" — true for Phase 1 itself, but the same substrate
   gap blocks two **reads** elsewhere in the system that the blocker table files under other
   headings. `GraphSubstrate` has **no edge-attribute accessor of any kind**: its full trait
   surface is `rust/crates/babylon-graph/src/substrate.rs:80-248`, whose only edge reader is
   `fn edges(&self, edge_type: &str) -> Vec<(NodeId, NodeId)>` (`:166`) returning bare id
   pairs, alongside `fn add_edge(…, strength: f64, …)` (`:111-116`) — there is no
   `edge_attribute`, and no reader for the strength either. So:
   - **computation 9's `_capital_labor_field`** (`contradiction.py:1109-1127`) reads
     `edge.attributes["tension"]` off every EXPLOITATION edge — the system reading back its
     own Phase-1 write. Row 9 files this under "no level-lattice primitive" and never names
     the edge read, which blocks it independently of the lattice.
   - **computation 2.8's `_national_chauvinism_balance`** (`contradiction.py:595-657`) reads
     the INFLUENCES **edge** attribute `influence_level` (`relationship.py:124-129`). Row 2
     files 2.8 among the graph-level-attribute reads; its edge-attribute half is unnamed.
   Landing Slice 2 serves none of these: it mints edge references over storage that does not
   exist. The unblock is the substrate widening `structural_verbs.rs:387-398` names verbatim
   — *"Widening that state widens the canonical `state_hash` field set, which is a declared
   Phase-2/substrate decision (Constitution III.7)"* — and it is the single deepest gap in
   this system, broader than row 1 states.

3. **CORRECTION — computation 2.4's `tanh` is filed as a missing intrinsic; it is an ADR172
   ruling-5 PORT-QUESTION, and the proposed `exp`/`log` rewrite would route around a gate
   that is deliberately mechanical.** The intrinsic fact is confirmed:
   `declarations.rs:110` is `DECLARABLE_INTRINSICS: [&str; 3] = ["exp", "log", "floor"]`, no
   `tanh`. But immediately below it, `declarations.rs` also declares
   `PROHIBITED_INTRINSIC_NAMES: [&str; 1] = ["sigmoid"]`, with the reason stated in its own
   doc comment: *"`sigmoid` would hand content the exact mechanism ADR172 ruling 5 forbids,
   pre-packaged and named; it is the one part of the doctrine gate that can be made
   mechanical, so it is."* `calculate_scissors_balance` is `tanh(price_log/scale)` clamped to
   `[-1,1]` (`formulas/market.py:97-107`) — algebraically `2·sigmoid(2x) − 1`, a stipulated
   saturating curve imposed on the **CANONICAL `price_value` opposition** (ADR078), which is
   precisely the "no imposed functional forms" surface. Row 2's suggested rewrite
   (`tanh(x) = (e^{2x}−1)/(e^{2x}+1)`, "feasible algebraically … but not free") would express
   the prohibited shape out of two permitted intrinsics. Per the standing brief — *"any
   sigmoid/logistic/stipulated curve in frozen Python is a PORT-QUESTION row, not
   auto-portable"* — this is a **Director escalation**, not a D-record and not an intrinsic
   request. The same applies, less sharply, to computation 2.7's `math.exp`
   (`contradiction.py:453-455`): `exp` is declarable, but the exponential is being used to
   *shape* `financialization_index`, not to compute a physical quantity.

4. **CONFIRMATION — the RESERVED-LINE flag on the `national` opposition is correct and is
   the right level of restraint.** Verified verbatim: `contradiction.py:155-163` declares
   `_STANCE_CHAUVINISM_SCORE = {UPHOLD: 1.0, IGNORE: 0.5, ABOLISH: 0.0}` with its own comment
   *"A hardcoded categorical map, not a `GameDefines` coefficient — the same division as
   ``formulas.balkanization._STANCE_TO_POLICY``"*, and `catalog.py:963-984` carries the
   `key="national"` binding with `pole_a="national-chauvinism"`, `pole_b="internationalism"`,
   `antagonistic=True`, `shadow=True` and a `unity` string citing the owner ruling
   2026-07-15 by date. Described, not proposed for change — exactly right. **Adjacent note
   for the Director dossier, not a correction:** the sibling map
   `formulas/balkanization.py:24-28` feeds `SovereigntySystem`@17.5's `extraction_policy`,
   whose own inventory omitted the reserved-line check; the two are one line, adjudicated in
   two places.

5. **CONFIRMATION — the pole channel (computation 11) is dead output.** `rg -n
   'sigma_capital_labor|derived_class_cell'` over `src/` (tests excluded) returns the write
   sites in `contradiction.py` (`:133`, `:962`, `:964`) and the two
   `SOCIAL_CLASS_EXCLUDED_FIELDS` rows in `world_state.py:83,85` — nothing reads them. §5's
   correction of the module's own docstring (which claims the partition sentinel reads them,
   when `checks.py::analyze_partition` reads the `pole_readings` **graph attr** instead) is
   verified and is a genuine documentation defect caught.

6. **CONFIRMATION — §7's byte-gate caveat is exactly right, and it is the most consequential
   sentence in the report.** `tools/regression_test.py:924-964`'s `graph_content_hash` builds
   its digest from `graph.nodes(data=True)` and `graph.edges(data=True)` only (`:958-963`),
   and its own docstring at `:939-943` states *"Graph metadata (`g.graph`: economy, event
   log, opposition states) is also excluded, because the spec's field set is
   nodes/edges/actions"*. Of this system's seven write channels only the edge `tension` write
   reaches the hash. "A green `qa:regression` run today provides real evidence only for
   Phase 1" is confirmed.

7. **CONFIRMATION — tick position and the ordering claim.** `contradiction.py:170` declares
   `position = 18.0`; `_DEFAULT_SYSTEMS` is derived by sorting `_SYSTEM_CLASSES` on
   `position` (`simulation_engine.py:376-378`), and the neighbours are
   `MarketScissorsSystem` 17.8 (`market_scissors.py:158`) and `ContradictionFieldSystem` 19.0
   (`contradiction_field.py:63`). The §5 observation that all four `opposition_states`
   consumers sit at positions **before** 18.0 — and therefore necessarily read last tick's
   snapshot — is a real and load-bearing channel fact.

**FINAL VERDICT: BLOCKED — sustained, and "nothing in this system is PORTABLE NOW" stands —
but re-based onto three blockers, not the four families named.** (i) The **substrate
edge-attribute gap**, on both READ and WRITE and across three computations (1, 2.8, 9), not
one — a hash-relevant Constitution III.7 decision, deeper than any query-lane slice and
unscheduled on all four. (ii) The **architecture question** of rows 4, 6 and 9 — a ranked
registry of nineteen named heterogeneous bindings, a typed morphism graph over opposition
keys, and a Lawverian level-lattice/Aufhebung regime classifier — sustained in full, and
correctly identified as the largest and differently-shaped unblock in the estate; §3.6's
"ordinary nodes of ordinary types" ruling gives row 4's option (a) a sanctioned home but
does not answer the ranking/coupling/lattice mathematics. (iii) `market_balance`'s `tanh` as
an **ADR172 ruling-5 Director escalation**, not a missing intrinsic. The graph-attribute
family (rows 2/3/5/10) is **withdrawn as a blocker** except for `tick_dynamics`'s live
Pydantic objects and row 7's nested-record write shape: §3.6 rules it and Slice 1 builds it.

**INADEQUATE-COVERAGE — a re-read must add:**
(a) `docs/reference/bsl-language.rst` **§3.6** (`:2639-2689`) to the reference-material list
and a re-adjudication of rows 2, 3, 5, 7 and 10 against the carrier ruling — the single
largest gap in this otherwise very thorough report;
(b) `rust/crates/babylon-graph/src/substrate.rs:80-248` (the trait surface itself), which
proves row 1's gap covers reads as well as writes, and a re-scoping of rows 2 and 9 to name
their edge-attribute halves;
(c) `declarations.rs`'s `PROHIBITED_INTRINSIC_NAMES` alongside `DECLARABLE_INTRINSICS`, and
the promotion of computations 2.4 and 2.7 to PORT-QUESTION rows under ADR172 ruling 5;
(d) closure of the one open discrepancy §5 flags and leaves — `doctrine.py:76-79`'s "(one
tick stale by pipeline position: I-ORD compliant)" comment against the position arithmetic
14.7 < 18.0. It is cheap to settle and it decides whether computation 2.10 reads this tick's
or last tick's `political_form_org_positions`, which is a conformance-vector fact.
