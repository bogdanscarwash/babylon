# TickDynamicsSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `TickDynamicsSystem` (position 4.0, Material Base) is not a
self-contained system like Territory (378 lines) — it is a 2,558-line *orchestrator*
that glues together ~28 externally-injected `ServicesProtocol` calculators (Features
012–016, 021, 023, 024, 057/058) into an annual (`tick % 52 == 0`) per-county pipeline,
plus a weekly flow-accrual side-channel. Its own arithmetic (smoothing, precarity,
crisis-phase state machine, endogenous-interest formula, bifurcation risk) is mostly
portable, ordinary float math; its two dominant, system-wide blockers are (1) **no BSL
equivalent for the external-service/data-source boundary** the whole pipeline is built
on, and (2) **no BSL equivalent for graph-level opaque-object metadata storage**
(`graph.set_graph_attr("tick_dynamics", …)`), which is how `CountyEconomicState`'s ~15
nested sub-models round-trip across ticks. One live libm/sigmoid hazard was found
(`DefaultWagePressureCalculator.compute_wage_pressure`, `math.exp`) that is also a
NO-IMPOSED-SIGMOIDS PORT-QUESTION under ADR172/173. Under the qa:regression hermetic
harness, almost the entire per-county pipeline (Vol I wage-pressure, Vol II
circulation, the Leontief imperial-rent pipeline, the accumulation-loop reserve-ratio
producer, Feature-016 transitions, hex substrate) is **structurally dormant** — no
canonical scenario wires the services that would exercise it; only Step 2 (national
MELT/gamma) and the Vol III financial-distribution layer (on the single `single_county`
scenario alone) run with real per-county data anywhere in the canonical estate.

**Verdict:** BLOCKED (dominant: external-service/data-source boundary has no BSL
equivalent; secondary: graph-level opaque-object metadata storage has no BSL
equivalent) — with a small PORTABLE-NOW/WITH-D-RECORD island (annual gate, flow
accrual, coefficient smoothing, precarity derivation, bifurcation risk, tick-summary
aggregation) that could plausibly port ahead of the rest as its own slice, and one
PORT-QUESTION (the wage-pressure sigmoid) reserved for a NO-IMPOSED-SIGMOIDS ruling
before any port attempt, dormant or not.

---

## 1. FILE MAP

**Scope note (read before the table):** unlike Territory, this system's actual
mathematics live overwhelmingly in code it does not own — ~28 fields on
`ServicesProtocol` (`melt_calculator`, `basket_calculator`, `gamma_calculator`,
`capital_calculator`, `throughput_calculator`, `unemployment_source`,
`housing_source`, `income_source`, `cpi_source`, `wage_source`, `employment_source`,
`reserve_army_data_source`, `dispossession_data_source`, `tensor_registry`,
`turnover_profile_source`, `inventory_data_source`, `depreciation_data_source`,
`distribution_calculator`, `rent_calculator`, `housing_calculator`,
`financial_crisis_assessor`, `fictitious_capital_calculator`,
`credit_aggregate_source`, `transition_engine`, `periphery_labor_source`,
`final_demand_source`, `industry_county_allocator`, `production_chain_calculator`,
`hex_grid`), injected by the engine's DI container and called through the protocol
boundary. Each of these is itself a previously-shipped Feature (012–016, 021, 023,
024, 057/058) with its own formula set, deserving its own dossier. **This inventory
fully catalogues TickDynamicsSystem's own module tree** (the main file plus every
module it directly imports and calls a free function/class from) **and treats the
ServicesProtocol-injected calculators as service dependencies** — named, their wiring
status on canonical scenarios established (§5), but their internal formulas out of
scope, the same way Territory's inventory treated `TickContext.displacement_mode` as
context usage rather than re-deriving `DisplacementPriorityMode` logic. This is a
disciplined scope boundary, not an oversight — see §5/§6 for why it does not change
the verdict (the boundary itself, not what's behind it, is the blocker).

| File | Lines | Role |
|---|---|---|
| `src/babylon/domain/economics/tick/system/__init__.py` | 2,558 | **The target.** `TickDynamicsSystem`, the full annual/weekly pipeline. Read completely, line by line. |
| `src/babylon/domain/economics/tick/system/imperial_rent.py` | 304 | `compute()` — thin orchestration of the Leontief imperial-rent pipeline (Spec 057/058); delegates to 4 injected services, own logic is validation + sentinel-event publishing. |
| `src/babylon/domain/economics/tick/types.py` | 591 | All tick-pipeline Pydantic models: `SimulationTickState`, `NationalTickParameters`, `CountyEconomicState` (the ~15-submodel root), `CrisisState`/`CrisisPhase`, `BifurcationRiskMetric`, `SmoothedCoefficients`, `TickSummary`, `NationalFinancialParameters`, `DerivedRates`. |
| `src/babylon/domain/economics/tick/graph_bridge.py` | 587 | `write_tick_state_to_graph`/`read_tick_state_from_graph`/`stamp_county_attrs_to_territories`/`write_national_financial_state_to_graph`/`read_national_financial_state_from_graph`/`reserve_army_signal`/`resolve_county_identity`. The graph I/O seam — see the graph-level-metadata blocker (§4/§6). |
| `src/babylon/domain/economics/tick/crisis_detector.py` | 333 | `MultiPeriodCrisisDetector` (5-phase state machine, live) + `ThresholdCrisisDetector` (instantiated in `__init__` line 129, **never called** — dead field). |
| `src/babylon/domain/economics/tick/derived_rates.py` | 112 | `DerivedRateCalculator` — profit rate / OCC / exploitation rate / phi-aggregate, pure arithmetic. |
| `src/babylon/domain/economics/tick/precarity.py` | 65 | `PrecarityDeriver` — U-6/PTER/NILF derivation, pure arithmetic, hardcoded (non-defines) fractions. |
| `src/babylon/domain/economics/tick/smoothing.py` | 60 | `CoefficientSmoother` — EMA smoothing, pure arithmetic. |
| `src/babylon/domain/economics/circulation/circuit.py` | 207 | `advance_circuit`/`initialize_circuit_state` — Capital Vol. II circuit-of-capital recurrence (M-C-P-C'-M'). |
| `src/babylon/domain/economics/circulation/crisis.py` | 145 | `assess_circulation_crisis` — 3-way crisis flag detection from circuit/inventory/reproduction state. |
| `src/babylon/domain/economics/circulation/fixed_circulating.py` | 146 | `update_depreciation_fund` (used); `decompose_constant_capital`/`compute_moral_depreciation` (imported by nothing in this system — **not exercised**). |
| `src/babylon/domain/economics/circulation/reproduction.py` | 215 | `check_simple_reproduction`/`check_extended_reproduction`/`combine_departments_ii`/`compute_disproportionality` — Vol. II reproduction-schema math. |
| `src/babylon/domain/economics/circulation/defaults.py` | 148 | `FALLBACK_PROFILE` (a default `TurnoverProfile`) — imported, used as the fallback turnover profile. |
| `src/babylon/domain/economics/circulation/types/__init__.py` + `.../types/_legacy.py` | 90 + 1,476 | `CircuitState`, `TurnoverProfile`, `DepreciationFundState`, `InventoryState`, `ReproductionBalance`, `ReproductionAnalysis`, `DisproportionalityCrisis`, `CirculationCrisisAssessment`, `CirculationCrisisState` — all with `@computed_field` properties that ARE part of the executed formula chain (`liquidity_ratio`, `commodity_overhang`, `production_time`, `circulation_time`, `fund_adequacy`, `replacement_cycle_position`, `inventory_problem`, `actual_i_share`, `imbalance`, `imbalance_direction`). |
| `src/babylon/domain/economics/credit/endogenous_interest.py` | 117 | `endogenous_interest_rate`/`loan_market_tightness` — Capital Vol. III Part V endogenous interest formula. Pure arithmetic, no libm. |
| `src/babylon/domain/economics/credit/types.py` | 337 | `CreditState`, `FictitiousCapitalStock`, `EndogenousInterestRate` (carries a `model_validator` invariant with no BSL equivalent — §4/§6). |
| `src/babylon/domain/economics/crisis/bifurcation.py` | 264 | `BifurcationRiskCalculator` — the one module in this system with genuine, portable-shaped **graph reads** (SOLIDARITY edges, node attrs). |
| `src/babylon/domain/economics/crisis/wage_compression.py` | 89 | `should_halt_accumulation` (used); `apply_wage_compression` (imported by nothing — the system re-implements its formula inline instead, §4 finding). |
| `src/babylon/domain/economics/reserve_army/accumulation.py` | 177 | `DefaultAccumulationLoopCalculator` — Ch. 25 mechanization-displacement/firm-failure reserve-army producer. `round()`-based Real→Int demotion (not `floor`, §4 finding). |
| `src/babylon/domain/economics/reserve_army/calculator.py` | 65 | `DefaultWagePressureCalculator` — **the libm/sigmoid hazard** (`math.exp`, bounded-sigmoid reserve-ratio→wage-pressure mapping). |
| `src/babylon/domain/economics/reserve_army/types.py` | 77 (relevant span) | `ReserveArmyDynamics.net_inflow` computed property — used by the accumulation loop. |
| `src/babylon/domain/economics/dynamics/types.py` | 322 | `ClassDistribution` (sum-to-one `model_validator`), `EconomicConditions` — pure data, no computation of its own; consumed by the external `transition_engine`. |
| `src/babylon/domain/economics/tensor.py` | 559 | `NoDataSentinel`, `DepartmentRow`, `ValueTensor4x3` — `ValueTensor4x3`'s `@computed_field`s (`profit_rate`, `organic_composition`, `total_s`, …) ARE read via `getattr` from the injected `tensor_registry`'s tensors, so they are part of the live execution chain even though `tensor_registry` itself is a service dependency. |
| `src/babylon/formulas/constants.py` | 38 | `HOURS_PER_YEAR`=2080, `WEEKS_PER_YEAR`=52 — **process-cached at import time** from `GameDefines.load_default()` (§4 finding: defines-desync hazard, same class as `circulation/types/_legacy.py`'s threshold accessors). |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase`. TickDynamicsSystem uses **none** of `_write_clamped`/`_read`/`_publish` — all graph/event access is inline (`graph.update_node(...)`, `services.event_bus.publish(Event(...))` directly). |
| `src/babylon/kernel/tick_partition.py` | 30 | `TickPartition.MATERIAL_BASE`. |
| `src/babylon/models/types.py` | 337 (relevant span 40–310) | `Currency`/`LaborHours`/`SignedLaborHours`/`Probability` — `Annotated[float, ...]` + `SnapToGrid` (1e-5/1e-6 grid), applied only at Pydantic model construction, never at `graph.update_node()` time (same runtime-storage caveat Territory's report established for `BabylonGraph.update_node`). |
| `src/babylon/models/enums/events.py` | — | `EventType` — 5 distinct values emitted by this system (§2/§7). |

**Not exercised by this system at all despite being imported/adjacent:**
`decompose_constant_capital`, `compute_moral_depreciation` (`fixed_circulating.py`),
`apply_wage_compression` (`wage_compression.py`), `ThresholdCrisisDetector.is_crisis`
(instantiated, never called), `PureCirculationCosts`, `TransportationValue`,
`RealizationCrisis`, `MoralDepreciation` (circulation types, never constructed by this
pipeline), `services.counter_tendency_calculator`, `services.value_basis_converter`
(built by `create_financial_services` but **never read** by this system — verified,
zero references — so `NationalFinancialParameters.counter_tendencies` and
`.monetary_adjustment` are always `None` from this producer).

**Reference read for format:** `reports/territory-port-phase1-inventory-2026-08-11.md`
(this report's own template) and its cited `metabolism.bsl`/`vitality.bsl` packs, for
the D-record/`defconst`/bare-literal/Currency-workaround conventions reused below.

---

## 2. COMPUTATION CATALOG (execution order, `step()`, `system/__init__.py:149-313`)

### 2.0 — Annual-boundary gate (`step`, lines 163-196)
- **(a)** The whole annual pipeline (everything below) runs only when
  `tick % WEEKS_PER_YEAR == 0`; every other tick just re-stamps last boundary's values
  and accrues one week's flow slice.
- **(b)** `tick: int = context.tick` (line 163); `if tick % WEEKS_PER_YEAR != 0:` (line
  174) → `read_tick_state_from_graph` + `stamp_county_attrs_to_territories` (re-stamp,
  lines 183-185) + `self._accrue_flows(graph)` (line 186) + `return` (line 187). On a
  boundary tick: `self._accrue_flows(graph)` (line 195, closes out the OUTGOING year
  using pre-recompute state) runs BEFORE the annual recompute overwrites it.
- **(c) Reads:** `context.tick`; on non-boundary ticks, the graph's persisted
  `tick_dynamics` state (`read_tick_state_from_graph`).
- **(d) Writes:** none directly (delegates to `stamp_county_attrs_to_territories`/
  `_accrue_flows`, catalogued separately below).
- **(e) Defines:** `timescale.weeks_per_year` = 52, `>= 1` — `defines.yaml:374`;
  schema `TimescaleDefines` (`src/babylon/config/defines/tunables.py:53`). **Consumed
  via the process-cached module-level `WEEKS_PER_YEAR` constant
  (`formulas/constants.py:37`), not `services.defines` — see §4 finding (defines
  desync hazard).**
- **(f) Events:** none.

### 2.1 — Flow accrual (`_accrue_flows`, lines 315-355)
- **(a)** Every tick (boundary or not), each territory that has ever seen an annual
  boundary accrues one `1/52` slice of its boundary-authoritative annual `phi`/`wage`
  flow onto running counters, so per-tick observers see smooth intra-year flow
  accumulation instead of a step function at year boundaries.
- **(b)** `annual_phi = phi_hour * HOURS_PER_YEAR` (line 345); `annual_wage =
  median_wage * HOURS_PER_YEAR * employment` (line 346); write
  `flow_phi_accrued = prior_phi + annual_phi / WEEKS_PER_YEAR`, `flow_wage_accrued =
  prior_wage + annual_wage / WEEKS_PER_YEAR` (lines 351-355).
- **(c) Reads:** `TERRITORY.tick_phi_hour` (guard: `None` ⇒ skip node, line 339 —
  empty-domain, not a fabricated zero), `TERRITORY.tick_median_wage` (default 0.0),
  `TERRITORY.tick_employment` (default 0.0), `TERRITORY.flow_phi_accrued`/
  `flow_wage_accrued` (default 0.0 each, prior accumulator).
- **(d) Writes:** `TERRITORY.flow_phi_accrued`, `TERRITORY.flow_wage_accrued`.
- **(e) Defines:** `HOURS_PER_YEAR`=2080 (`timescale.hours_per_year`,
  `defines.yaml:375`), `WEEKS_PER_YEAR`=52 (`timescale.weeks_per_year`,
  `defines.yaml:374`) — both **process-cached at import time**, see §4.
- **(f) Events:** none.

### 2.2 — Flow-accrual reset (`_reset_flow_accrual`, lines 357-372)
- **(a)** Immediately after a fresh annual recompute lands, zero the running
  accumulators so the next non-boundary tick starts the new year's sum from zero.
- **(b)** `flow_phi_accrued=0.0, flow_wage_accrued=0.0` (line 372) for every territory
  with a `tick_phi_hour` (guard at line 370-371, same empty-domain semantics as 2.1).
- **(c) Reads:** `TERRITORY.tick_phi_hour` (presence guard only).
- **(d) Writes:** `TERRITORY.flow_phi_accrued=0.0`, `TERRITORY.flow_wage_accrued=0.0`.
- **(e) Defines:** none directly (bare `0.0` literals).
- **(f) Events:** none.

### 2.3 — Step 2: national parameters (`_compute_national_params`, lines 514-628)
- **(a)** Read national MELT (τ) from the injected MELT calculator (hard-required —
  `None` short-circuits the WHOLE annual pipeline); read basket-visibility γ_basket and
  reproductive-visibility γ_III from two more injected, OPTIONAL calculators
  (falling back to `GameDefines` defaults when unwired); EMA-smooth both γ's across
  ticks; compute effective MELT.
- **(b)** `tau = float(tau_result)` (line 547, from `services.melt_calculator.get_melt(year)`,
  line 542 — **external service call**); `gamma_basket`/`gamma_III` = smoothed via
  `CoefficientSmoother.smooth` (lines 592-601, formula in §2.9 below); `tau_effective =
  tau * gamma_basket` (line 603, one multiply); `floor_year = max(year, 2007)` (line
  617).
- **(c) Reads:** `services.melt_calculator.get_melt(year)` (external), optionally
  `services.basket_calculator.get_gamma_basket(year)` / `services.gamma_calculator.compute(year)`
  (external, both optional with GameDefines fallback), `prev_coefficients` (in-memory,
  from the prior tick's `SimulationTickState.coefficients`).
- **(d) Writes:** none directly (returns a `NationalTickParameters`, written to the
  graph later by `write_tick_state_to_graph`).
- **(e) Defines:** `economy.gamma_basket_default`=0.68, `> 0.0, <= 1.0`
  (`defines.yaml:100`); `economy.gamma_iii_default`=0.33, `> 0.0, <= 1.0`
  (`defines.yaml:101`); `DEFAULT_V_REPRODUCTION`=12.0 — **module-level Python
  constant, not defines-backed** (`system/__init__.py:104`, port-as-is: hardcoded,
  not a `GameDefines` field, D-record).
- **(f) Events:** none (only `fallbacks.record_*` counter increments, §5).

### 2.4 — Step 3a: county-level state (`_compute_county_states`, lines 715-852, +
helpers `_resolve_renter_share` 630-659, `_wired_bracket_ratio` 661-682,
`_resolve_real_wage_deflator` 684-713)
- **(a)** For every county FIPS in scope, carry forward the prior tick's state and
  overlay any newly-available exogenous reading (capital stock, throughput,
  unemployment, renter share, median wage bootstrap, employment, CPI deflator,
  bracket ratio) from up to 9 different OPTIONAL injected data sources; each falls
  back to the prior value (or a documented MVP literal on the very first tick) when
  its source is unwired or returns no data for this county-year (Constitution III.11
  graceful degradation, not fabrication).
- **(b)** Per-field pattern, repeated ~9 times with a different source object each
  time: `x = prev.x if prev else DEFAULT; if services.x_source is not None: result =
  services.x_source.get_x(fips, year); if result is not None and isinstance(result,
  (int, float)): x = float(result)`. No arithmetic beyond `float(...)` casts; the only
  computed value is `class_dist` year-reclamping (`min(max(year_floor, 2007), 2030)`,
  line 810/824).
- **(c) Reads:** `services.capital_calculator.get_K` (line 749), `services.throughput_calculator.compute_metrics`
  (line 757), `services.unemployment_source.get_county_unemployment_rate` (line 768),
  `services.housing_source.get_county_renter_share` (via `_resolve_renter_share`, line
  656), `services.wage_source.get_county_median_hourly_wage` (bootstrap-only, line
  786), `services.employment_source.get_county_total_employment` (line 795),
  `services.cpi_source.get_cpi_deflator` (via `_resolve_real_wage_deflator`, line
  710), `services.income_source.get_county_bracket_ratio` (via `_wired_bracket_ratio`,
  line 679) — **all 9 external services**; prior `CountyEconomicState` (in-memory).
- **(d) Writes:** none directly (returns `dict[fips, CountyEconomicState]`, in-memory
  only until `write_tick_state_to_graph`).
- **(e) Defines:** none of the bootstrap literals (`0.0` capital_stock, `1.0`
  throughput_position, `2.0` supply_chain_depth, `0.05` unemployment_rate, `0.10`
  u6_rate, `0.04` pter_rate, `0.06` nilf_rate, `21.0` median_wage, `100_000.0`
  employment) are `GameDefines`-backed — **all are bare Python literals in the method
  body** (lines 747, 754-755, 766, 776-778, 784, 793) — port-as-is: transcribe
  verbatim, D-record each (documented "MVP hardcoded national averages" per the module
  docstring, lines 106-109, for `DEFAULT_FORECLOSURE_RATE`/`DEFAULT_BANKRUPTCY_RATE`/
  `DEFAULT_EVICTION_RATE` which are used later in §2.13).
- **(f) Events:** none.

### 2.5 — Step 3a+: precarity derivation (`_derive_precarity`, lines 854-878,
delegating to `PrecarityDeriver.derive`, `precarity.py:44-62`)
- **(a)** Map lumpenproletariat class share to a "precaritization rate," then blend it
  with unemployment to produce U-6/PTER/NILF broad-unemployment indicators.
- **(b)** `u6 = min(max(unemployment_rate + precaritization_rate, 0.0), 1.0)`
  (`precarity.py:59`); `pter = min(max(precaritization_rate * pter_fraction, 0.0),
  1.0)` (line 60); `nilf = min(max(precaritization_rate * nilf_fraction, 0.0), 1.0)`
  (line 61). Nested-`min`/`max` clamp, matching the landed-pack clamp convention
  (no scalar `min`/`max` builtin in BSL grammar per Territory's §4 finding).
- **(c) Reads:** `county.unemployment_rate`, `county.class_distribution.lumpenproletariat_share`
  (in-memory).
- **(d) Writes:** `county.u6_rate`, `county.pter_rate`, `county.nilf_rate` (in-memory,
  `model_copy`).
- **(e) Defines:** `pter_fraction=0.4`, `nilf_fraction=0.6` — **`PrecarityDeriver()`
  constructed with NO arguments** (`system/__init__.py:132`), so these are the
  class's own hardcoded Python defaults (`precarity.py:38-39`), **never read from
  `GameDefines`/`defines.yaml` at all** — a genuine "not data-driven" finding, D-record
  (transcribe verbatim, port-as-is).
- **(f) Events:** none.

### 2.6 — Step 3b: coefficient smoothing (`_update_coefficients`, lines 880-910,
delegating to `CoefficientSmoother.smooth`, `smoothing.py:39-57`)
- **(a)** Exponential moving average of γ_basket/γ_III across ticks so a data source
  update doesn't jump the effective visibility coefficient instantaneously.
- **(b)** First tick (`not is_initialized`): `return raw` (passthrough,
  `smoothing.py:56`). Subsequent ticks: `previous + alpha * (raw - previous)`
  (line 57).
- **(c) Reads:** `national_params.gamma_basket`/`.gamma_III` (this tick's raw-smoothed
  values from §2.3), `prev_coefficients.gamma_basket`/`.gamma_III` (in-memory).
- **(d) Writes:** none directly (returns a `SmoothedCoefficients`, written to graph
  later).
- **(e) Defines:** `alpha=0.3` — **`CoefficientSmoother(alpha=0.3)` constructed with a
  hardcoded literal** (`system/__init__.py:133`), not `GameDefines`-backed (same class
  of finding as §2.5); `gamma_import=0.35` — **bare MVP-comment literal**
  (`system/__init__.py:900`, "MVP default"), also not defines-backed.
- **(f) Events:** none.

### 2.7 — Step 3.5: Vol I wage pressure (`_compute_vol1_layer`/`_compute_vol1_county_state`,
lines 1175-1241, delegating to `DefaultWagePressureCalculator.compute_wage_pressure`,
`reserve_army/calculator.py:32-65`)
- **(a)** For each county (up to 3,300, line 1202), read a real reserve-army
  decomposition and apply a **bounded sigmoid** mapping reserve ratio → downward wage
  pressure, then discount `median_wage` by that pressure. No-ops entirely if
  `services.reserve_army_data_source` is unwired (line 1196-1197).
- **(b)** **`exponent = -k * (reserve_ratio - r0)`, clamped to `[-500, 500]`
  (`calculator.py:49-51`); `raw = 1.0 / (1.0 + math.exp(exponent))` (line 52) —
  LIBM TRANSCENDENTAL. Then normalized: `baseline = 1.0 / (1.0 + math.exp(-k*(0-r0)))`
  (lines 55-57, a SECOND `math.exp` call), `normalized = (raw - baseline) / (1.0 -
  baseline)` (line 64, clamped `[0,1]`), `wage_pressure = ceiling * normalized`
  (line 65).** In `_compute_vol1_county_state`: `adjusted_wage = county.median_wage *
  (1.0 - pressure)` (`system/__init__.py:1240`).
- **(c) Reads:** `services.reserve_army_data_source.get_unemployment_decomposition(fips,
  year)` → `.reserve_ratio` (external service), `county.median_wage` (in-memory).
- **(d) Writes:** `county.median_wage` (in-memory, `model_copy`).
- **(e) Defines:** `reserve_army.sigmoid_k`=20.0, `> 0.0, <= 100.0`
  (`defines.yaml:414`); `reserve_army.sigmoid_r0`=0.08, `> 0.0, <= 1.0`
  (`defines.yaml:415`); `reserve_army.wage_pressure_ceiling`=0.5, `> 0.0, <= 1.0`
  (`defines.yaml:416`); schema `ReserveArmyDefines`
  (`src/babylon/config/defines/economy_labor.py:45`). **These ARE live,
  intentionally-tuned coefficients** — this is not a stub.
- **(f) Events:** none.
- **PORT-QUESTION, separate from the libm finding:** per the standing ruling ("NO
  IMPOSED SIGMOIDS" / ADR172 ruling 5 / ADR173, restated in this task's CURRENT BSL
  surface), a stipulated sigmoid in frozen Python is not auto-portable regardless of
  whether `exp` is a declarable BSL intrinsic (it is, per the CURRENT BSL surface) —
  the S-curve is supposed to EMERGE from a measure over class-member dispersion, not
  be hand-written. This computation needs a Director/architecture ruling before any
  transcription, not just a D-record.

### 2.8 — Step 3.6: the accumulation loop (`_compute_accumulation_loop`, lines
1243-1334, delegating to `DefaultAccumulationLoopCalculator.compute_dynamics`/
`.compute_reserve_ratio`, `reserve_army/accumulation.py:72-173`)
- **(a)** Ch. 25 (General Law of Capitalist Accumulation): a rising organic
  composition of capital (year-over-year) displaces workers; bankrupt-firm failures
  add a second inflow; both accumulate into a persistent per-territory reserve-army
  stock whose share of (stock + employment) is `reserve_ratio` — written directly to
  the graph so `ReserveArmySystem` (#5) and `DispossessionEventSystem` (#10), both
  later this SAME tick, read a real value.
- **(b)** `mechanization_displacement = round(delta_occ * employment *
  mechanization_displacement_rate)` when `delta_occ = occ_current - occ_prior > 0`
  (`accumulation.py:113-117`, **Real→Int demotion via `round()`, NOT `floor`** — see
  §4/§6, a distinct hazard from Territory's landed-`floor` cases); `firm_failures =
  round(bankruptcy_rate * employment * firm_failure_conversion_rate)`
  (lines 121-123, same `round()` shape); `net_inflow = (mechanization_displacement +
  firm_failures) - (expansion_absorption + emigration)` (`types.py:74-76`, both latter
  terms always 0 — never produced by any producer in this system); `new_stock =
  max(0.0, prior_stock + net_inflow)` (`accumulation.py:167`); `reserve_ratio =
  new_stock / (new_stock + employment)`, clamped to `[0, 1 - min_employed_fraction]`
  (lines 171-173). Separately, `foreclosure_rate`/`eviction_rate` are read straight
  through with a `max(0.0, min(x, 1.0))` clamp (`system/__init__.py:1328,1331`).
- **(c) Reads:** `services.tensor_registry.get(fips, year).organic_composition` (this
  year AND `year-1`, via `_get_organic_composition`, lines 1336-1362 — external
  service), `services.dispossession_data_source.get_bankruptcy_rate`/
  `get_foreclosure_rate`/`get_eviction_rate` (external service), `TERRITORY.reserve_army_stock`
  (prior tick, default 0.0), `county.employment` (in-memory, from §2.4).
- **(d) Writes:** `TERRITORY.reserve_army_stock`, `TERRITORY.reserve_ratio`,
  `TERRITORY.foreclosure_rate`, `TERRITORY.eviction_rate` — **written directly to the
  graph**, not through `CountyEconomicState`/`write_tick_state_to_graph` (a distinct
  channel from every other computation in this catalog).
- **(e) Defines:** `reserve_army.mechanization_displacement_rate`=0.05, `> 0.0`
  (`defines.yaml:418`); `reserve_army.firm_failure_conversion_rate`=0.5, `> 0.0, <=
  1.0` (`defines.yaml:419`); `reserve_army.min_employed_fraction`=0.01, `>= 0.0, <=
  1.0` (`defines.yaml:417`).
- **(f) Events:** none.

### 2.9 — Step 4: imperial rent (`_compute_imperial_rent`, lines 912-939, thin
delegation to `imperial_rent.compute`, `system/imperial_rent.py:45-150`)
- **(a)** Six-stage Leontief pipeline (BEA I-O decomposition → periphery wage
  coefficients → per-industry rent → QCEW employment-share allocation → per-county
  `phi_hour`), gated behind 5 required external services (`periphery_labor_source`,
  `final_demand_source`, `industry_county_allocator`, `production_chain_calculator`,
  `bea_industries`) all-or-nothing (`imperial_rent.py:86,158-169`).
- **(b)** The module's OWN code is validation (industry-list length/order alignment,
  raising `ValueError` on mismatch, `_validate_industry_alignment`, lines 233-285) and
  sentinel-event publishing; it performs **no arithmetic of its own** — the actual
  Leontief math (`decomposer.decompose`, `calculator.calculate`, `allocator.allocate`)
  is entirely inside the 3 injected `production_chain_calculator`/
  `industry_county_allocator` sub-objects, out of this inventory's scope (§ FILE MAP
  note).
- **(c) Reads:** `services.periphery_labor_source.get_coefficients(year)`,
  `services.final_demand_source.get_final_demand(year)`,
  `services.production_chain_calculator.flow_source.get_direct_requirements(year)`,
  `.import_shares_source.get_import_shares(year)`, `.decomposer.decompose(...)`,
  `.calculator.calculate(...)`, `services.industry_county_allocator.allocate(...)` —
  **all external**.
- **(d) Writes:** `county.phi_hour` (in-memory, `model_copy`, `_apply_allocation`,
  lines 288-303) — counties absent from the allocation keep their prior `phi_hour`
  (not zeroed, `imperial_rent.py:226-230`).
- **(e) Defines:** none read directly by this module (the injected calculators may
  read their own).
- **(f) Events:** `EventType.CALIBRATION_QCEW_CARRY_FORWARD` — emitted on EVERY
  graceful-degradation path (pipeline unwired, or any of the 3 upstream sources
  returning `NoDataSentinel`), 2 call sites (`_publish_pipeline_unwired_signal` line
  181-194, `_publish_no_data_signal` line 205-218), both with the wildcard-fips
  `county_fips="*"` sentinel-marker pattern.

### 2.10 — Step 4.5: circulation layer (`_compute_circulation_layer`, lines
1367-1441, + `_compute_national_circulation_state` 1443-1473, `_get_county_departments`
1475-1511, `_compute_reproduction_state` 1513-1567, `_compute_county_circulation_state`
1569-1732)
- **(a)** Advance each county's M-C-P-C'-M' capital circuit and depreciation fund one
  year, using the PRIOR TICK's own circuit/depreciation state (threaded explicitly
  through `prev_county_states`, not a system-instance cache — a documented U3
  determinism fix, lines 1380-1396); assess simple/extended reproduction balance from
  tensor department data; run the 3-way circulation-crisis assessment. No-ops entirely
  if `services.turnover_profile_source` is unwired (line 1409-1410).
- **(b)** Circuit advance (`circuit.py:43-127`): `purchase_frac =
  min(1.0, elapsed_days/purchase_time_days)` (and 2 more phase fractions, lines
  38-40,98-100); `m_to_p = money_capital * purchase_frac`, `p_to_c =
  productive_capital * production_frac`, `c_to_m = commodity_capital * sale_frac`
  (lines 103-105); `surplus_created = surplus_value * production_frac` (line 108);
  `new_money = money_capital - m_to_p + c_to_m`, `new_productive = productive_capital
  - p_to_c + m_to_p`, `new_commodity = commodity_capital - c_to_m + p_to_c +
  surplus_created` (lines 111-113), each floored at 0 via `max(0.0, ...)` (lines
  122-125); `new_fixed = new_productive * fixed_capital_ratio`, `new_circulating =
  new_productive - new_fixed` (lines 116-117). Depreciation-fund update
  (`fixed_circulating.py:98-107`): `new_accumulated = accumulated_depreciation +
  annual_depreciation` (one add). Reproduction (`reproduction.py:102-158`): `gap =
  (dept_i.v + dept_i.s) - dept_ii.c`, `condition_met = abs(gap) < tolerance`
  (lines 102-103); `labor_power_demand = dept_i.v + dept_ii.v + dept_iii.v`,
  `reproduction_capacity = dept_iii.c + dept_iii.v + dept_iii.s`, `gap = demand -
  capacity`, `sustainability = gap <= 0.0` (lines 148-151). Crisis assessment
  (`crisis.py:97-131`): `realization_crisis = commodity_overhang > threshold`;
  `turnover_crisis = (liquidity_ratio < threshold) and (circulation_time >
  production_time)`; `reproduction_crisis = None` when either reproduction input is
  `None` else `not condition_met or not sustainability` (III.11 honest-absence, not a
  fabricated `False`).
- **(c) Reads:** `services.turnover_profile_source.get_turnover_profile(fips[:2])`,
  `services.inventory_data_source.get_days_inventory_raw/finished/get_national_inventory`,
  `services.depreciation_data_source.get_annual_depreciation/get_gross_investment`,
  `services.tensor_registry.get(fips, best_year).dept_I/IIa/IIb/III` (all external);
  `prev_county.circulation_state` (in-memory, prior tick).
- **(d) Writes:** `county.circulation_state` (in-memory `CirculationCrisisState`,
  itself nesting `CircuitState`/`InventoryState`/`DepreciationFundState`/
  `DisproportionalityCrisis`/`CirculationCrisisAssessment` — ~15 scalar fields total).
- **(e) Defines:** `capital_vol2.commodity_overhang_threshold`=0.3
  (`defines.yaml:1035`), `.liquidity_crisis_ratio`=0.1 (`:1036`),
  `.reproduction_tolerance`=0.01 (`:1033`), `.dept_i_share_required`=0.6667 (`:1034`),
  `.national_employment`=155,000,000.0 (`:1042`), `.fallback_days_inventory`=30.0
  (`:1043`), `.min_annual_depreciation_floor`=1.0 (`:1044`),
  `.supply_crisis_days_threshold`=7.0 (`:1037`), `.overproduction_days_threshold`=60.0
  (`:1038`), `.replacement_boom_ratio`=1.5/`.replacement_expansion_ratio`=1.0/
  `.replacement_maintenance_ratio`=0.7 (`:1039-1041`); schema
  `src/babylon/config/defines/capital_vol2.py`.
- **(f) Events:** none.
- **Runtime-storage caveat (own finding, distinct from Territory's):**
  `InventoryState.inventory_problem` and `DepreciationFundState.replacement_cycle_position`
  read their thresholds via **module-level accessor functions called with NO
  argument** (`supply_crisis_days_threshold()`, `overproduction_days_threshold()`,
  `replacement_boom_ratio()` etc., `circulation/types/_legacy.py:903-905,776-781`) —
  these fall back to a **process-cached `GameDefines.load_default()`**
  (`_default_defines()`, `_legacy.py:74-82`), NOT the `services.defines` instance
  actually threaded through this tick's call (which IS available at the call site but
  not plumbed into the `@computed_field` property, per that module's own comment,
  lines 60-71: "cannot take a call-time parameter"). A run with a non-default
  `GameDefines` override would silently see stale thresholds here — same hazard class
  as `formulas/constants.py`'s import-time `HOURS_PER_YEAR`/`WEEKS_PER_YEAR` (§2.0/2.1),
  independently discovered in a different module.

### 2.11 — Step 5: crisis-trigger detection (`_check_crisis_triggers`, lines 941-1017,
delegating to `MultiPeriodCrisisDetector.evaluate`, `crisis_detector.py:107-330`)
- **(a)** Batch-evaluate 4 quarterly periods per annual tick against a 5-phase crisis
  lifecycle (NORMAL→ONSET→EARLY→DEEP→RECOVERY→NORMAL), driven by profit rate vs a
  threshold; apply compounding wage compression during every DEEP quarter; emit an
  event on every phase transition.
- **(b)** Phase machine (`crisis_detector.py`, per-phase methods `_evaluate_normal`/
  `_evaluate_onset`/`_evaluate_early`/`_evaluate_deep`/`_evaluate_recovery`, lines
  140-312): integer counters (`consecutive_below`/`consecutive_recovery`/
  `crisis_duration`) incremented/reset by threshold comparisons; `peak_severity =
  min(current_peak, profit_rate)` (`_update_severity`, line 330); `recovery_target =
  min(crisis_duration, r_cap)` (line 298). **Wage compression, INLINE-DUPLICATED, not
  via the dedicated `apply_wage_compression` function** (`system/__init__.py:988-997`):
  `median_wage = median_wage * (1.0 - wage_compression_rate)` (line 989);
  `new_cumulative = min(1.0 - (1.0 - prev_cumulative) * (1.0 - wage_compression_rate),
  1.0)` (lines 991-994) — byte-identical formula to `crisis/wage_compression.py:47-52`'s
  `apply_wage_compression`, but that function is dead code (§ FILE MAP); the port must
  transcribe THIS inline copy, not the unused named function (port-as-is law).
- **(c) Reads:** `self._get_profit_rate(fips, year, services)` →
  `services.tensor_registry.get(fips, year).profit_rate` with a carry-forward fallback
  to the most recent available year (`_get_profit_rate`, lines 1019-1059 — external
  service); `county.crisis_state`, `county.median_wage` (in-memory).
- **(d) Writes:** `county.crisis_state`, `county.median_wage` (in-memory,
  `model_copy`).
- **(e) Defines:** `crisis.r_threshold`=0.05, `> 0, <= 1` (`defines.yaml:35`);
  `crisis.n_consecutive`=3, `>= 1, <= 20` (`:36`); `crisis.m_recovery`=2, `>= 1, <= 20`
  (`:37`); `crisis.r_cap`=8, `>= 1, <= 52` (`:38`); `crisis.wage_compression_rate`=0.02,
  `>= 0, <= 0.5` (`:40`); `quarterly_evals = 4` — **bare Python literal**
  (`system/__init__.py:972`), not defines-backed; schema `CrisisDefines`
  (`src/babylon/config/defines/economy_basic.py:47`).
- **(f) Events:** `EventType.CRISIS_PHASE_TRANSITION` (every phase change, up to 4x
  per tick per county, `_emit_crisis_event`, line 1087-1099); `EventType.ECONOMIC_CRISIS`
  (only on NORMAL→ONSET, lines 1101-1113).

### 2.12 — Step 5.5: Vol III financial layer (`_compute_financial_layer`, lines
1737-1794, + `_compute_national_financial_state` 1921-1979, `_economy_wide_profit_rate`
1858-1919, `_compute_county_financial_state` 1981-2086, `_assess_county_financial_crisis`
2088-2155)
- **(a)** Compute the ENDOGENOUS national interest rate (Capital Vol. III Part V — no
  FRED read; derived from the economy-wide realized profit rate and reserve-army-driven
  loan-market tightness), then per county: surplus distribution (s=p+i+r+t), ground
  rent, housing decomposition, and an integrated financial-crisis assessment, each
  gated on its own optional injected calculator.
- **(b) National (own arithmetic, no libm):** `_economy_wide_profit_rate`
  (lines 1905-1919): `total_surplus += surplus_i` (only `surplus>0 and profit_rate>0`),
  `total_capital_advanced += surplus_i / profit_rate_i`, `r = total_surplus /
  total_capital_advanced` — **the surplus-weighted aggregate, NOT an unweighted mean**
  (avoids the intensive-aggregation-variance-error pattern, own docstring lines
  1873-1879 names this explicitly). `endogenous_interest_rate`
  (`credit/endogenous_interest.py:43-95`): `tau = clamp(tightness, 0, 1)`;
  `if profit_rate <= 0: rate=0, fragility_premium=0` (III.11, "nothing to divide" —
  the model's own `EndogenousInterestRate` schema then ENFORCES `rate < ceiling`
  via a `model_validator`, `credit/types.py:315-336`, raising `ValueError` if
  violated — a genuine runtime invariant assertion with **no BSL equivalent**, §4/§6);
  else `share = base + (ceiling - base) * tau`, `rate = profit_rate * share`,
  `premium = profit_rate * (ceiling - base) * tau` (lines 85-87). `loan_market_tightness`
  (lines 98-116): `demand = gain * reserve_army_signal`, `tau = clamp(demand - 0.0, 0,
  1)` (the `_IDLE_MONEY_CAPITAL_SUPPLY` constant is hardcoded `0.0`, line 23 — a
  documented deferred term, "no graph quantity yet"). `reserve_army_signal`
  (`graph_bridge.py:511-542`): employment-weighted mean unemployment `rho_bar`
  (`_employment_weighted_unemployment`, lines 476-508, sorted-FIPS accumulation per
  Constitution III.7); `raw = (rho_bar - rho_ref) / (1 - rho_ref)`, clamped `[0,1]`
  (lines 541-542).
- **(c) Reads:** `county_states` in-memory scope (surplus/profit-rate via
  `services.tensor_registry`, external), `services.distribution_calculator.compute_distribution(...)`,
  `services.rent_calculator.compute_rent_extraction(...)`,
  `services.housing_calculator.decompose_housing_value(...)`,
  `services.financial_crisis_assessor.assess(...)`,
  `services.fictitious_capital_calculator.compute_fictitious_capital(year)`,
  `services.credit_aggregate_source.get_total_credit(year)` — **all external, own
  formulas out of scope**.
- **(d) Writes:** graph attr `NATIONAL_FINANCIAL_ATTR` = `"national_financial"`
  (`write_national_financial_state_to_graph`, `graph_bridge.py:433-450` — a
  `params.model_dump()` PLAIN DICT written under `graph.set_graph_attr(...)`, i.e.
  graph-LEVEL metadata, not a node attribute); `county.surplus_distribution`,
  `.debt_accumulation`, `.rent_extraction`, `.housing_decomposition`,
  `.financial_crisis` (in-memory, `model_copy`).
- **(e) Defines:** `capital_vol3.interest_profit_share_base`=0.3 (`defines.yaml:1067`),
  `.interest_profit_share_ceiling`=0.95 (`:1068`), `.interest_reserve_demand_gain`=1.0
  (`:1069`), `.interest_reserve_reference`=0.08 (`:1070`), `.national_county_count`=3300
  (`:1062`), `.default_rate_estimate`=0.02 (`:1063`),
  `.housing_capitalization_rate_default`=0.05 (`:1065` — construction-time snapshot,
  never re-read per-tick per `factory.py:467-470`), `.profit_rate_fallback`=0.05
  (`:1061`); schema `src/babylon/config/defines/capital_vol3.py`.
- **(f) Events:** none directly (this step's degradation paths use
  `services.economics_fallbacks.record_vol3_*` counters + throttled `logger.warning`,
  `_log_vol3_sentinel_once_per_year`, lines 1796-1829 — not event-bus emissions).

### 2.13 — Step 6: class transitions (`_simulate_transitions`, lines 2346-2458, +
`_check_dispossession_cascade` 1115-1170)
- **(a)** Synthesize `EconomicConditions` for the injected Feature-016
  `transition_engine`, apply an accumulation halt when wages fall below a subsistence
  floor, run one year of class transitions, and emit a milestone event on sustained
  labor-aristocracy decline.
- **(b)** `effective_wage = median_wage * HOURS_PER_YEAR` (line 2378, hourly→annual);
  `if should_halt_accumulation(median_wage, DEFAULT_V_REPRODUCTION, floor_ratio):
  effective_wage = 0.0` (line 2379-2380, `should_halt_accumulation`:
  `wage < subsistence * floor_ratio`, `wage_compression.py:84-85`). Dispossession
  cascade (`_check_dispossession_cascade`, lines 1136-1170): `decline = baseline_la -
  current_la` (line 1142); `for milestone in sorted(milestones): if decline >=
  milestone: crossed = milestone` (lines 1149-1150, takes the HIGHEST milestone
  crossed).
- **(c) Reads:** `services.transition_engine.simulate_transitions(dist, conditions,
  crisis_phase=...)` (external, entirely opaque — own math out of scope);
  `services.dispossession_data_source.get_foreclosure_rate/get_bankruptcy_rate/get_eviction_rate`
  (external, optional — falls back to `DEFAULT_FORECLOSURE_RATE`=0.006,
  `DEFAULT_BANKRUPTCY_RATE`=0.006, `DEFAULT_EVICTION_RATE`=0.063, module-level
  hardcoded Python constants, `system/__init__.py:107-109`); `national_params.tau`,
  `county.phi_hour`, `county.crisis_state.phase` (in-memory).
- **(d) Writes:** `county.class_distribution` (in-memory, `model_copy`).
- **(e) Defines:** `crisis.wage_compression_floor_ratio`=0.8, `>= 0, <= 1`
  (`defines.yaml:41`); `crisis.dispossession_cascade_milestones`=[0.05, 0.10, 0.15]
  (`defines.yaml:46-49`).
- **(f) Events:** `EventType.DISPOSSESSION_CASCADE` — on the highest LA-decline
  milestone crossed, only when `crisis_phase != NORMAL` and a prior county state
  exists (line 2445-2452).

### 2.14 — Step 5b: bifurcation risk (`_compute_bifurcation_risk`, lines 2241-2306,
delegating to `BifurcationRiskCalculator.compute`, `crisis/bifurcation.py:71-260`)
- **(a)** During active crisis only, synthesize cross-class solidarity-edge density,
  a legitimation index (blending a lifecycle-produced structural score with inverse
  mean agitation), and a class-burden ratio into a directional [-1,+1] score.
  **The one computation in this whole system with genuine, portable-shaped GRAPH
  reads** (not external-service calls).
- **(b)** `raw = -w_s * solidarity + w_b * burden` (`bifurcation.py:115`); `dampened =
  raw * (1.0 - legitimation)` (line 116); `score = max(-1.0, min(1.0, dampened))`
  (line 117). Solidarity density (`_compute_solidarity_density`, lines 126-180):
  double loop over same-county `social_class` nodes counting possible vs actual
  cross-class `SOLIDARITY` edges, `actual / possible` (line 180) — an O(n²)
  all-pairs shape, exactly the fold/exists/typed-neighbor query pattern. Legitimation
  (`_compute_legitimation`, lines 182-235): `agitation_inverse = max(0.0, min(1.0, 1.0
  - mean(agitation)))` (lines 224-225); when a lifecycle-produced value is present,
  `blended = blend_weight * lifecycle_legitimation + (1 - blend_weight) *
  agitation_inverse`, clamped (lines 230-233). Class burden
  (`_compute_class_burden_ratio`, lines 237-260): `delta_la = |prev.LA - curr.LA|`,
  `delta_prol = |prev.Prol - curr.Prol|`, `ratio = min(delta_la / max(delta_prol,
  epsilon), 1.0)` (line 259-260, epsilon division-guard clamp, `if delta_la == 0.0:
  return 0.0` short-circuit at line 256-257).
- **(c) Reads:** **graph** — `graph.query_nodes(node_type=TERRITORY)` for
  `legitimation_index` (own-county match by id, lines 101-104 — set by
  `LifecycleSystem`, a LATER-this-tick system at position 7.0, so this reads LAST
  tick's value); `graph.query_nodes()` filtered `node_type == "social_class"` for
  `attrs["county_fips"]`, `attrs["role"]`, `attrs["ideology"]["agitation"]` (dict OR
  attribute-style access, lines 214-218 — handles both shapes); `graph.get_edge(a, b,
  EdgeType.SOLIDARITY)` (lines 176). Also in-memory `previous_distribution`/
  `current_distribution` (`ClassDistribution`, before/after §2.13's transitions).
- **(d) Writes:** none directly (returns a `BifurcationRiskMetric`, later serialized
  as `tick_bifurcation_score` on the territory node by `stamp_county_attrs_to_territories`).
- **(e) Defines:** `crisis.bifurcation_solidarity_weight`=1.0, `>= 0` (`defines.yaml:42`);
  `crisis.bifurcation_burden_weight`=1.0, `>= 0` (`:43`); `crisis.class_burden_epsilon`=0.001,
  `> 0, <= 0.1` (`:44`); `crisis.bifurcation_event_threshold`=0.5, `>= 0, <= 1`
  (`:45`); `blend_weight=0.6` — **constructor default, NOT threaded from
  `GameDefines`** (`crisis/bifurcation.py:64`, `BifurcationRiskCalculator(...)`
  constructed at `system/__init__.py:2271` with only `solidarity_weight`/
  `burden_weight`/`epsilon` from defines — `blend_weight` stays at its hardcoded 0.6
  default, a D-record same class as §2.5/§2.6/§2.14's `epsilon`).
- **(f) Events:** `EventType.BIFURCATION_THRESHOLD` — when `abs(score) >= threshold`
  (line 2295-2302).

### 2.15 — Step 7: distribution validation (`_validate_distributions`, lines
2460-2486)
- **(a)** Loud invariant check: every county's 5 class shares must still sum to
  1.0 (tolerance 0.001) after transitions.
- **(b)** `total = sum(5 shares)` (lines 2474-2480); `if abs(total - 1.0) > 0.001:
  raise ValueError(...)` (lines 2481-2486) — **exception-based abort, no BSL
  equivalent** (§4/§6).
- **(c) Reads:** `county.class_distribution` (in-memory).
- **(d) Writes:** none (validation only).
- **(e) Defines:** `0.001` tolerance — **bare Python literal**, not defines-backed.
- **(f) Events:** none (raises, does not publish).

### 2.16 — Step 8: tick summary (`_compute_tick_summary`, lines 2488-2555, delegating
to `DerivedRateCalculator`, `derived_rates.py:40-109`)
- **(a)** Aggregate per-county derived rates (profit rate, OCC, exploitation rate) and
  a national imperial-rent total and employment-weighted national class distribution.
- **(b)** `DerivedRateCalculator.compute_county_rates` (`derived_rates.py:54-92`):
  `annual_hours = employment * HOURS_PER_YEAR`; `v = v_reproduction * annual_hours`;
  `total_value = tau * annual_hours`; `s = total_value - v`; `profit_rate = s / (K +
  v)` when `K+v > 0` else `None`; `organic_composition = K / v` when `v > 0` else
  `None`; `exploitation_rate = s / v` when `v > 0` else `None`. `compute_phi_aggregate`
  (lines 94-109): `total += phi_hour * employment * HOURS_PER_YEAR` per county — a
  `fold`-shaped sum. National weighted class distribution
  (`system/__init__.py:2521-2538`): `weight = county.employment / total_employment`;
  `national_dist[k] += county_share_k * weight` for each of 5 shares — **extensive
  (employment-weighted) aggregation, correctly avoiding the unweighted-mean
  intensive-aggregation-variance-error pattern** (per this repo's own memory note).
  Plain arithmetic mean for `mean_profit_rate`/`mean_occ`/`mean_exploitation_rate`
  (lines 2549-2553, `sum(...)/len(...) if ... else 0.0` — these ARE unweighted means,
  but over a list of already-derived RATIOS, not raw extensives, so not the same
  defect class).
- **(c) Reads:** `county_states` (in-memory, all counties), `national_params.tau`.
- **(d) Writes:** none directly (returns a `TickSummary`).
- **(e) Defines:** `HOURS_PER_YEAR`=2080 (process-cached import-time constant, §4).
- **(f) Events:** none.

### 2.17 — Graph serialization (`write_tick_state_to_graph`/`stamp_county_attrs_to_territories`,
`graph_bridge.py:80-295`)
- **(a)** Publish the fully-assembled `SimulationTickState` to the graph: national
  metadata as an opaque dict under a single graph-level key, and ~30 flattened
  `tick_`-prefixed scalar/dict attributes per territory node.
- **(b)** No arithmetic of its own (pure serialization); the ~30 per-territory writes
  are direct field reads off `CountyEconomicState` and its nested sub-models
  (`graph_bridge.py:175-294`).
- **(c) Reads:** the whole `SimulationTickState` (in-memory).
- **(d) Writes:** **graph-level** `graph.set_graph_attr("tick_dynamics", {year,
  national_params, coefficients, tick_summary, is_year_boundary, county_states,
  credit_cycle_phase})` (lines 97-108) — **the entire nested Pydantic object graph,
  stored as an opaque dict under ONE key, not per-node/per-field**; per-territory
  `tick_capital_stock`, `tick_throughput_position`, `tick_supply_chain_depth`,
  `tick_phi_hour`, `tick_crisis_phase` (enum `.value`), `tick_crisis_duration`,
  `tick_bifurcation_score`, `tick_wage_compression`, `tick_class_distribution` (a
  5-key dict), `tick_unemployment_rate`, `tick_renter_share`, `tick_median_wage`,
  `tick_bracket_ratio`, `tick_real_wage_deflator`, `tick_profit_rate`, `tick_occ`,
  `tick_exploitation_rate`, `tick_liquidity_ratio`, `tick_commodity_overhang`,
  `tick_replacement_cycle`, `tick_inventory_diagnosis`, `tick_realization_crisis`,
  `tick_turnover_crisis`, `tick_reproduction_crisis`, `tick_disproportionality`,
  `tick_interest_burden`, `tick_ground_rent`, `tick_rentier_share`,
  `tick_profit_of_enterprise`, `tick_financialization_share`, `tick_taxes_on_surplus`,
  `tick_total_surplus`, `tick_accumulated_debt`, `tick_claims_exceed_surplus`,
  `tick_housing_fictitious_fraction`, `tick_financial_crisis_signals` (32 distinct
  node attributes, `graph_bridge.py:175-294`).
- **(e) Defines:** none (pure I/O).
- **(f) Events:** none.

### 2.18 — Step 9: hex substrate (`_write_hex_substrate`, lines 374-407)
- **(a)** Aggregate R7 hex-level economic data to R6 territory resolution and write
  `hex_`-prefixed territory attrs, enabling organizational/player verbs to read
  spatialized economic metrics. **No-op if `services.hex_grid is None`** (line
  393-394).
- **(b)-(f):** **NOT READ** — `src/babylon/domain/economics/substrate/hex_graph_bridge.py`
  (408 lines, `aggregate_r7_to_r6`/`write_hex_state_to_graph`) was not opened for this
  inventory: `rg -n 'hex_grid' tools/regression_test.py tools/regression_scenarios.py`
  returned zero hits, confirming `hex_grid` is never wired anywhere in the qa:regression
  harness, so this whole step is dormant on every canonical scenario (§5) and reading
  its internals would not change any verdict in §6. Flagged UNVERIFIED for its own
  content; the no-op guard clause itself (`system/__init__.py:392-394`) IS verified.

**Events emitted by the whole system: 5 distinct `EventType` values, 7 call sites** —
`CALIBRATION_QCEW_CARRY_FORWARD` (×2, `imperial_rent.py:190,214`),
`CRISIS_PHASE_TRANSITION` (`system/__init__.py:1089`), `ECONOMIC_CRISIS` (`:1105`),
`DISPOSSESSION_CASCADE` (`:1160`), `BIFURCATION_THRESHOLD` (`:2332`). Per the CURRENT
BSL surface, `TickReport` carries no event log — every one of these is a WS1 (#502)
ledger row, unpinnable by goldens today, same as every other System's emissions.

---

## 3. TYPE INVENTORY

**Runtime storage note (same finding class as Territory's §3):**
`BabylonGraph.update_node` (`src/babylon/topology/graph.py:660-670`) is a plain dict
merge with no type coercion or quantization. `Currency`/`LaborHours`/
`SignedLaborHours`/`Probability` (`models/types.py`) apply `SnapToGrid` (1e-5/1e-6
grid) only at Pydantic model CONSTRUCTION — i.e. inside `CountyEconomicState`,
`ClassDistribution`, `CircuitState`, etc. — never at `graph.update_node()` time. Every
`tick_*`/`flow_*`/`reserve_*`/`hex_*` graph attribute is a raw Python `float`/`int`/
`str`/`bool`/`dict`, un-quantized, un-typed at the storage layer.

**Graph node-level attributes (the portable surface — every attribute a downstream
System or the qa byte-gate can actually see):**

| Attribute | Node type | Python type | Domain | Category |
|---|---|---|---|---|
| `county_fips` | TERRITORY | `str` | 5-digit string | identity (read-only input) |
| `tick_capital_stock` | TERRITORY | `float` | `[0, ∞)` | unbounded real, money-semantic |
| `tick_throughput_position` | TERRITORY | `float` | `(0, ∞)` (schema `gt=0`) | unbounded real |
| `tick_supply_chain_depth` | TERRITORY | `float` | `[0, 5]` | bounded real |
| `tick_phi_hour` | TERRITORY | `float` | `[0, ∞)` | unbounded real, money-semantic |
| `tick_crisis_phase` | TERRITORY | `str` (enum `.value`) | `{normal,onset,early,deep,recovery}` | **Enum discriminant** |
| `tick_crisis_duration` | TERRITORY | `int` | `≥ 0` | integer |
| `tick_bifurcation_score` | TERRITORY | `float` | `[-1, 1]` | bounded real |
| `tick_wage_compression` | TERRITORY | `float` | `[0, 1]` | unit-interval |
| `tick_class_distribution` | TERRITORY | `dict[str, float]` | 5 keys, each `[0,1]`, sum≈1 | **structured/composite — no BSL equivalent as one field** |
| `tick_unemployment_rate` | TERRITORY | `float` | `[0, 1]` | unit-interval |
| `tick_renter_share` | TERRITORY | `float` | `[0, 1]` | unit-interval |
| `tick_median_wage` | TERRITORY | `float` | `[0, ∞)` | unbounded real, money-semantic |
| `tick_bracket_ratio` | TERRITORY | `float` | `[0, ∞)` | unbounded real |
| `tick_real_wage_deflator` | TERRITORY | `float` | `(0, ∞)` | unbounded real |
| `tick_profit_rate`, `tick_occ`, `tick_exploitation_rate` | TERRITORY | `float \| None` | `[0, ∞)` or absent | **nullable real** — division-by-zero yields `None`, distinct from `NoDataSentinel` |
| `tick_liquidity_ratio`, `tick_commodity_overhang` | TERRITORY | `float` | `[0, 1]` | unit-interval |
| `tick_replacement_cycle` | TERRITORY | `str` (enum `.value`) | `{investment_boom,expansion,maintenance,disinvestment}` | **Enum discriminant** |
| `tick_inventory_diagnosis` | TERRITORY | `str` (enum `.value`) | `{normal,supply_crisis,overproduction}` | **Enum discriminant** |
| `tick_realization_crisis`, `tick_turnover_crisis` | TERRITORY | `bool` | `{T,F}` | boolean |
| `tick_reproduction_crisis` | TERRITORY | `bool \| None` | `{T,F,None}` | **nullable boolean** — honest-absence, not fabricated `False` |
| `tick_disproportionality` | TERRITORY | `float \| None` | signed real or absent | nullable signed real |
| `tick_interest_burden`, `tick_ground_rent`, `tick_rentier_share`, `tick_profit_of_enterprise`, `tick_financialization_share`, `tick_taxes_on_surplus`, `tick_total_surplus`, `tick_accumulated_debt` | TERRITORY | `float` (0.0 default) | `[0, ∞)` typically | unbounded real, money-semantic |
| `tick_claims_exceed_surplus` | TERRITORY | `bool` | `{T,F}` | boolean |
| `tick_housing_fictitious_fraction` | TERRITORY | `float \| None` | `[0,1]` or absent | nullable unit-interval |
| `tick_financial_crisis_signals` | TERRITORY | `int` (0 default) | `≥ 0` (bit-count-like) | integer |
| `flow_phi_accrued`, `flow_wage_accrued` | TERRITORY | `float` | `[0, ∞)` (accumulates within-year) | unbounded real |
| `reserve_army_stock` | TERRITORY | `float` | `[0, ∞)` | unbounded real |
| `reserve_ratio` | TERRITORY | `float` | `[0, 1 - min_employed_fraction]` | bounded real (defines-parameterized ceiling) |
| `foreclosure_rate`, `eviction_rate` | TERRITORY | `float` | `[0, 1]` | unit-interval |
| `legitimation_index` | TERRITORY | `float` | `[0, 1]` (read-only, LifecycleSystem-owned) | unit-interval, cross-system READ |
| `role`, `ideology.agitation` | SOCIAL_CLASS | `str`, nested `float` | open string / `[-1,1]`-ish | **`ideology` is a 2-level dict — no flat BSL field today** (D-record: flatten to `agitation`) |
| graph attr `"tick_dynamics"` | — (graph-level) | opaque `dict` (nested Pydantic objects) | — | **structured/composite graph-LEVEL metadata — no BSL equivalent at all** |
| graph attr `"national_financial"` | — (graph-level) | `dict` (`model_dump()`) | — | same category |

**In-memory-only Pydantic fields (never reach the graph, exist only inside
`CountyEconomicState` between Step 3a and `write_tick_state_to_graph`):** ~50+
additional scalar fields across `SurplusValueDistribution`, `RentExtraction`,
`DebtAccumulation`, `FinancialCrisisAssessment`, `CircuitState`'s
`money_capital`/`productive_capital`/`commodity_capital`/`fixed_capital`/
`circulating_capital`, `InventoryState`'s `raw_materials`/`work_in_progress`/
`finished_goods`/`days_inventory_raw`/`days_inventory_finished`,
`DepreciationFundState`'s `total_fixed_capital`/`accumulated_depreciation`/
`annual_depreciation_flow`/`replacement_expenditure`. These are the fields a genuine
port of §2.10/§2.12 would need as individual `deffield`s if the graph-level-metadata
blocker (§4/§6) were ever lifted — not tabulated exhaustively here (out of a Phase-1
inventory's proportionate depth for a system this size), but their COUNT (dozens) is
itself load-bearing evidence for how large that blocker's payload is.

**Enum discriminants found:** `CrisisPhase` (5-valued, `tick/types.py:43-63`),
`ReplacementCyclePosition` (4-valued, `circulation/types/_legacy.py:261`),
`InventoryDiagnosis` (3-valued, `_legacy.py:284`), `CreditCyclePhase` (5-valued,
`credit/types.py:17-40`, declared but **never produced by this system** — `phase`
stays at its `Field(default=CreditCyclePhase.EXPANSION)` forever, since
`_build_credit_state` (`system/__init__.py:1831-1856`) never sets `phase`/`prev_phase`
— a dead-enum finding, D-record if ever ported). Per the CURRENT BSL surface, `enum`
fields ARE landed (ADR195/196) — these are all candidates for `defenum`, not blocked
by the enum-storage gap Territory's report found (that gap is resolved on current
dev).

---

## 4. FLOAT-OP INVENTORY

Execution order follows §2's numbering.

1. **Modulo + comparison (§2.0):** `tick % WEEKS_PER_YEAR != 0` — integer modulo,
   portable.
2. **Multiply/divide chain (§2.1):** `phi_hour * HOURS_PER_YEAR`; `median_wage *
   HOURS_PER_YEAR * employment`; `prior + annual/WEEKS_PER_YEAR` — 4 multiplies, 2
   divides, all plain `float`. **`HOURS_PER_YEAR`/`WEEKS_PER_YEAR` are process-cached
   at IMPORT TIME** (`formulas/constants.py:17,32,37`, `_DEFINES: Final[GameDefines] =
   GameDefines.load_default()`) — a scenario/test that overrides `GameDefines` after
   import sees STALE values here, independent of the `services.defines` instance
   threaded through every other call in this same file. Same hazard class
   independently found in `circulation/types/_legacy.py` (§2.10).
3. **EMA smoothing (§2.6):** `previous + alpha * (raw - previous)` — one multiply, two
   adds, no libm. Bare `alpha=0.3` literal is a BSL-parser concern (non-integer
   literal needs `c`-suffix or the Real-zero-promotion idiom, per Territory's §4
   finding) but not a math hazard.
4. **`math.exp` — TWO call sites, one function (§2.7):** `reserve_army/calculator.py:52,57`
   — **LIBM TRANSCENDENTAL, the system's one nondeterminism hazard.** Both calls are
   pre-clamped to `[-500, 500]` (lines 51,56) to prevent overflow, but `exp` itself is
   not guaranteed bit-identical across libm implementations (cross-platform/
   cross-language determinism hazard per this repo's own behavioral-contracts
   standard). Flagged also as a PORT-QUESTION (§2.7, §6) independent of the libm issue
   — even a bit-identical `exp` would still be an "imposed functional form."
5. **`round()` — Real→Int demotion, NOT `floor` (§2.8):** `accumulation.py:115-117,121-123`
   — `round(delta_occ * employment * rate)`, `round(bankruptcy_rate * employment *
   rate)`. Python's `round()` is round-half-to-even (banker's rounding); the CURRENT
   BSL surface declares `floor` as an intrinsic (landed PR #489) but **no `round`
   intrinsic**. `floor(x + 0.5)` is NOT equivalent to Python `round()` at exact
   half-integer ties (round-half-to-even vs round-half-up) — this is a genuine,
   distinct gap from Territory's landed-`floor` cases (those were plain truncating
   casts on non-negative operands, where trunc≡floor exactly). **Named blocker, not
   auto-portable via the landed `floor` intrinsic alone.**
6. **Circuit-of-capital recurrence (§2.10):** 3 phase-fraction `min(1.0, days/duration)`
   clamps (`circuit.py:38-40`), 3 multiplies (flows), 1 multiply (surplus), 3
   subtract-then-add reconstructions, each wrapped in `max(0.0, ...)` (floor-at-zero
   clamp, lines 122-125) — a THIRD clamp idiom in this system (alongside `_write_clamped`'s
   two-sided `max(lo,min(hi,v))` from Territory and this system's own `min(...,1.0)`/
   `max(0.0,min(...))` two-sided forms below) — all nested `min`/`max`, no scalar
   clamp builtin needed, consistent with landed-pack convention.
7. **Depreciation fund (§2.10):** one add (`accumulated + annual_depreciation`); ratio
   divisions (`fund_adequacy`, `replacement_cycle_position`'s
   `replacement_expenditure/annual_depreciation_flow`) — `annual_depreciation_flow` is
   schema-guaranteed `> 0` (`gt=0`, `_legacy.py:735`), so no explicit zero-guard in the
   division itself (an implicit-precondition pattern, not a runtime `if`).
8. **Reproduction schema (§2.10):** `gap = (v+s) - c`; `abs(gap) < tolerance` compare;
   `demand - capacity`; `gap <= 0.0` compare — pure add/subtract/compare, no libm.
9. **Circulation crisis flags (§2.10):** 2 threshold compares (`>`, `<`), 1 boolean
   `and`, 1 tri-state `None`-vs-`bool` branch — no arithmetic beyond the compares.
10. **Crisis-phase counters (§2.11):** integer increments/resets, `min(current_peak,
    profit_rate)`, `min(crisis_duration, r_cap)` — no libm. Wage compression:
    `wage * (1.0 - rate)` (one multiply); `1.0 - (1.0-prev)*(1.0-rate)`, clamped
    `min(...,1.0)` — a THIRD independent hand-written copy of the SAME formula shape
    Territory's report never saw, present here in duplicate (live inline copy vs. the
    dead `apply_wage_compression` function, §2.11(b)).
11. **Endogenous interest (§2.12):** `share = base + (ceiling-base)*tau`; `rate =
    profit_rate * share`; `premium = profit_rate*(ceiling-base)*tau` — 2 multiplies,
    1 add, no libm; guarded by a schema-level `model_validator` invariant
    (`rate < ceiling`) that RAISES `ValueError` on violation — an assertion construct,
    not itself float arithmetic, but load-bearing (§6).
12. **`_economy_wide_profit_rate` (§2.12):** `total_surplus += s_i` (guarded
    `s_i>0 and r_i>0`), `total_capital_advanced += s_i/r_i`, final `total_surplus /
    total_capital_advanced` — sorted-FIPS accumulation (Constitution III.7-compliant
    ordering), 1 division per county plus 1 final division, no libm.
    `reserve_army_signal`: `(rho_bar - rho_ref)/(1-rho_ref)`, clamped `[0,1]` — one
    division, explicit `denom <= 0.0` zero-guard (lines 539-540, a THIRD distinct
    division-by-zero handling idiom in this system alongside the schema-precondition
    idiom (#7) and the epsilon-guard idiom (#14 below)).
13. **`_get_organic_composition`/tensor computed fields (§2.8, `tensor.py`):**
    `organic_composition = c/v` or `float('inf')` if `v==0` (`tensor.py:225-227,434-436`);
    `profit_rate = s/(c+v)` or `float('inf')` if denominator 0 (lines 397-400);
    `exploitation_rate = s/v` or `inf`. **`float('inf')` as an explicit sentinel
    return value** — a genuine IEEE-754 special-value dependency; the accumulation
    loop (§2.8) explicitly guards against it reaching arithmetic
    (`math.isfinite(occ_current) and math.isfinite(occ_prior)`,
    `accumulation.py:110-111`), but the type itself (`float | inf`) has no natural BSL
    analogue beyond "a very large declared-domain real," and BSL's own arithmetic
    would need an explicit inf-guard convention wherever this value is consumed.
14. **Bifurcation risk (§2.14):** `raw = -w_s*solidarity + w_b*burden`; `dampened =
    raw*(1-legitimation)`; `score = max(-1,min(1,dampened))` — 2 multiplies, 1
    subtract, 1 two-sided clamp; `agitation_inverse = max(0,min(1, 1-mean(agitation)))`;
    `blended = bw*lifecycle + (1-bw)*agitation_inverse`, clamped; `ratio =
    min(delta_la/max(delta_prol, epsilon), 1.0)` — **epsilon-guarded division, a
    FOURTH distinct div-by-zero idiom** (`class_burden_epsilon`=0.001 default guard,
    vs. #7's schema-precondition and #12's explicit `<= 0.0` branch and #13's
    `float('inf')` sentinel — four different idioms for the same underlying hazard
    class across this one system, worth flagging as a port-as-is transcription
    hazard: **do not unify them**, transcribe each faithfully per its own site).
15. **`_validate_distributions` (§2.15):** `sum(5 floats)`, `abs(total-1.0) > 0.001`
    compare, then `raise ValueError` — the raise itself has no BSL equivalent (§6).
16. **Tick summary (§2.16):** `annual_hours = employment*HOURS_PER_YEAR`; `v =
    v_reproduction*annual_hours`; `total_value = tau*annual_hours`; `s = total_value -
    v`; `profit_rate = s/(K+v)` guarded `K+v>0`; `organic_composition = K/v` guarded
    `v>0`; `exploitation_rate = s/v` guarded `v>0` — **a THIRD div-by-zero idiom
    variant** (`if denominator > 0: ... else: None`, an explicit `if`-guard rather
    than epsilon or schema-precondition). `phi_aggregate`: `sum(phi_hour*employment*
    HOURS_PER_YEAR)` — fold-shaped. National class distribution:
    `weight=employment/total_employment` (explicit `if total_employment > 0` guard,
    line 2530), `national_dist[k] += share_k*weight` — 5 multiply-accumulates,
    extensive (employment-weighted) aggregation, no libm.

**Bare non-integer literals found (BSL-parser concern, not a math hazard by itself):**
`1.0`/`0.0` throughout (dozens of sites); `0.3` (alpha, §2.6); `0.4`/`0.6` (precarity
fractions, §2.5); `0.001` (validation tolerance, §2.15; class-burden epsilon default,
§2.14); `500.0`/`-500.0` (exp-overflow clamp bounds, §2.7); `0.05`/`0.006`/`0.063`
(dispossession defaults, module-level); `12.0` (`DEFAULT_V_REPRODUCTION`); `21.0`,
`100_000.0`, `0.10`, `0.04`, `0.06` (county bootstrap defaults, §2.4); `0.35`
(`gamma_import` MVP default, §2.6); `2007`, `2030`, `2010` (year clamp bounds).

**Real→Int demotions found:** `round()` ×2 (§2.8, item 5 above — the one named
blocker); no `int(x)`-truncation sites found in this system (unlike Territory's two
`floor`-equivalent truncating casts) — this system's demotions are exclusively via
`round()`.

**Clamp implementations found (count and enumerate, per the template's "flag clamp
implementations and inconsistencies" instruction):** (i) nested `min(max(x,lo),hi)` —
precarity §2.5, accumulation-loop reserve-ratio via `min(ratio, ceiling)` paired with
`max(0.0, prior+inflow)` §2.8, bifurcation §2.14 (twice); (ii) `max(0.0, min(1.0,
foreclosure))` two-sided — accumulation-loop foreclosure/eviction §2.8; (iii)
`min(1.0, phase_fraction)` upper-only — circuit-of-capital §2.10 (×3 sites); (iv)
`max(0.0, new_money)` lower-only — circuit-of-capital §2.10 (×3 sites); (v)
`_clamp_unit` — a NAMED helper function in `endogenous_interest.py:34-40` (the ONLY
site in this entire system that factors the clamp into a reusable function rather than
inlining it) — used twice (tau, and the deferred idle-money-capital term). **Five
distinct clamp shapes across 4 modules — this system never uses Territory's/
`SystemBase._write_clamped` at all** (confirmed, §1) — port-as-is: transcribe every
site's own shape, do not unify, per the same discipline Territory's report already
established for its own two-clamp inconsistency.

**Libm hazard summary:** exactly one function, two call sites (`math.exp` in
`DefaultWagePressureCalculator.compute_wage_pressure`) — everything else in this
sizeable system is `+`/`-`/`*`/`/`/`min`/`max`/`round`/`abs`/comparisons.

---

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 4.0** (`system/__init__.py:124`), confirmed against
  `_SYSTEM_CLASSES`: `VitalitySystem (1.0) → TerritorySystem (2.0) → SubstrateSystem
  (2.5) → ProductionSystem (3.0) → TickDynamicsSystem (4.0) → ...`.
- **Reads from a same-tick prior system: none of substance.** Vitality(1.0)/
  Substrate(2.5)/Production(3.0) write `wealth`, `population`, `extraction_intensity`
  (grep-confirmed: `rg -n 'update_node' src/babylon/engine/systems/{vitality,substrate,production}.py`)
  — zero overlap with anything this system reads. The one apparent cross-system read,
  `TERRITORY.legitimation_index` (§2.14), is written by `LifecycleSystem` at position
  **7.0 — LATER this same tick** — so `_compute_bifurcation_risk` (position 4.0)
  necessarily reads LAST TICK's `legitimation_index`, never this tick's (a
  same-tick-ordering nuance worth transcribing faithfully, not "fixing," in any port).
- **Writes consumed later this tick / downstream ticks:**
  - `TERRITORY.reserve_ratio` — read by `engine/systems/reserve_army.py`
    (ReserveArmySystem, #5, immediately next position) and `engine/systems/market_scissors.py`
    (@17.8).
  - `TERRITORY.foreclosure_rate`/`.eviction_rate` — read by
    `engine/systems/dispossession_events.py` (DispossessionEventSystem, #10).
  - `TERRITORY.tick_phi_hour`, `.tick_profit_rate` — read by `engine/systems/policy.py`
    (@17.47) and (profit_rate only) `engine/systems/market_scissors.py`.
  - Graph attr `national_financial` — read by `engine/systems/policy.py` and
    `engine/systems/market_scissors.py`.
  - `TERRITORY.tick_median_wage`, `.tick_employment`, `.tick_class_distribution` —
    grep-confirmed **read by no other System** (`rg -l` over
    `src/babylon/engine/systems/` returns zero hits for all three) — terminal/
    observational outputs, same category as Territory's `heat`/`rent_level`.
  - `TERRITORY.reserve_army_stock` — grep-confirmed read by no other System (only this
    system reads it back, as `prior_stock` next year, §2.8(c)).
- **Context/service usage with no BSL equivalent — this is the system-wide finding,
  not a per-attribute one.** Unlike Territory (one `TickContext.displacement_mode`
  override), this system's ENTIRE per-county pipeline (Steps 3a through 6, §2.3-2.13)
  is gated behind ~28 `ServicesProtocol` fields, each an injected Python object
  satisfying an ad-hoc protocol (`melt_calculator`, `basket_calculator`,
  `gamma_calculator`, `capital_calculator`, `throughput_calculator`,
  `unemployment_source`, `housing_source`, `income_source`, `cpi_source`,
  `wage_source`, `employment_source`, `reserve_army_data_source`,
  `dispossession_data_source`, `tensor_registry`, `turnover_profile_source`,
  `inventory_data_source`, `depreciation_data_source`, `distribution_calculator`,
  `rent_calculator`, `housing_calculator`, `financial_crisis_assessor`,
  `fictitious_capital_calculator`, `credit_aggregate_source`, `transition_engine`,
  `periphery_labor_source`, `final_demand_source`, `industry_county_allocator`,
  `production_chain_calculator`, `hex_grid`). Every one of these is a runtime
  method-call boundary with no analogue in a closed BSL rules-as-content algebra.
- **DORMANCY on canonical scenarios — the dominant §5/§6 finding, established via
  `tools/regression_scenarios.py` + `tools/regression_test.py`:**
  - **13 scenarios are declared** in `regression_scenarios.py` (`imperial_circuit`,
    `two_node`, `starvation`, `glut`, `fascist_bifurcation`, `detroit_tri_county`,
    `single_county`, `mitterrand`, `syriza`, `weimar`, `debs`, `bernie_valve`,
    `org_probe`).
  - Of the 6 scenarios `regression_test.py` wires with `_build_vol3_calculator_overrides`
    (melt_calculator + all of `create_financial_services`'s bundle: distribution/
    rent/housing/financial_crisis/fictitious_capital/credit_aggregate — comment at
    `regression_scenarios.py:220`: "the other 5 canonical scenarios use"), **NONE
    carry `county_fips` on any territory** except `single_county` and
    `detroit_tri_county` (the file's own comment, `regression_test.py:172-175`: "the
    five regression scenarios carry no `county_fips`"). Grep-confirmed (`rg -n
    "_tick_dynamics.county_states is empty every tick"`, ~24 hits across the file):
    Step 3a's `county_fips` list is EMPTY on those 5, so **the entire per-county
    pipeline (§2.4-§2.16) executes zero iterations** — only Step 2 (national MELT/
    gamma, §2.3) and the national half of Step 5.5 (`_compute_national_financial_state`
    over an empty `county_states`, structurally `rate=0.0`/`tightness=0.0` — verified
    live, dated 2026-07-20 in the file's own comments) actually run.
  - **`detroit_tri_county`** carries 3 real counties (Macomb 26099, Oakland 26125,
    Wayne 26163) but its committed golden is only 5 ticks
    (`regression_scenarios.py:2059-2064`: `tick_range = range(1, 5)`, none a multiple
    of 52) — **the annual boundary NEVER crosses in this scenario's baseline**, so
    even though it is county-bearing, Steps 2-9 have never fired in it either (only
    `_accrue_flows`'s no-op-until-first-boundary path runs, and even that is inert
    since no `tick_phi_hour` has ever been stamped). 13 named channels
    (`county_<fips>_{interest,ground_rent,taxes}` ×3 + `financial_*` ×4) are
    explicitly declared `at_rest` for this exact mechanism.
  - **`single_county`** (Wayne County, FIPS 26163, `tools/regression_test.py:215-258`,
    real reference-DB tensor via a committed fixture) is the **ONLY canonical
    scenario where the annual pipeline has been verified to fire with real per-county
    data** — a documented live 52-tick spot-run (`regression_scenarios.py:2179-2188`)
    with `distribution.interest_payments == 970247586.15` and
    `endogenous_interest.rate == 0.01783`. Even here: `services.turnover_profile_source`,
    `.reserve_army_data_source`, `.dispossession_data_source`, `.capital_calculator`,
    `.throughput_calculator`, `.unemployment_source`, `.housing_source`,
    `.income_source`, `.cpi_source`, `.wage_source`, `.employment_source`,
    `.transition_engine`, `.periphery_labor_source`, `.final_demand_source`,
    `.industry_county_allocator`, `.production_chain_calculator` are grep-confirmed
    **absent from `regression_test.py`/`regression_scenarios.py` entirely** — so
    **§2.7 (Vol I wage pressure/the sigmoid), §2.8 (accumulation loop/reserve_ratio),
    §2.9 (imperial rent Leontief pipeline), §2.10 (Vol II circulation), §2.13 (Feature
    016 transitions), §2.18 (hex substrate) are ALL structurally dormant even on
    `single_county`** — the ONLY live per-county subsystem anywhere in the canonical
    estate is §2.12 (Vol III financial distribution) plus its Step-5 crisis-detector
    dependency (which needs only `tensor_registry`, which `single_county` DOES wire).
  - **Net effect:** a port's conformance fixtures for §2.4-§2.11, §2.13, §2.14,
    §2.16, §2.18 (i.e., almost the entire computation catalog except §2.0-2.3, §2.6,
    §2.12, §2.15, §2.17) must be hand-built from scratch — nothing here can be
    harvested from the canonical estate, mirroring Territory's own finding for its
    ADJACENCY-dependent phases, but at a much larger scale (10 of 19 catalog entries,
    vs. Territory's 3 of 4 phases).

---

## 6. BLOCKER ASSESSMENT

Adjudicated against the CURRENT BSL surface stated in this task (query lane Slice 1
LANDED; Slices 2-4 NOT built — no edge-attribute reads, no hyperedge/metric lane, no
attribute-storage widening; enum fields LANDED; `exp`/`log`/`floor` declarable;
Currency-typed field storage refused, `int`-workaround the accepted deviation class;
no imposed functional forms is a standing ruling; events unpinnable — WS1 row for
every emission; two same-position rules don't yet share pre-state).

| Computation | Verdict | Detail |
|---|---|---|
| §2.0 Annual-boundary gate | **PORTABLE NOW** | Plain integer modulo + compare. `WEEKS_PER_YEAR` process-cache desync (§4) is a D-record, not a blocker. |
| §2.1/§2.2 Flow accrual + reset | **PORTABLE NOW** | Plain node-attr read/multiply/divide/write, no query lane needed. `HOURS_PER_YEAR`/`WEEKS_PER_YEAR` same D-record as above. Reads `tick_phi_hour`, which only a live upstream (§2.9, blocked) would ever populate with real data — expressible today against a `:const`/seeded field regardless. |
| §2.3 National params (own smoothing arithmetic) | **PORTABLE WITH D-RECORD** | The EMA-smoothing formula itself is trivial `defconst` arithmetic. BLOCKED for its INPUT: `services.melt_calculator.get_melt(year)` etc. are external-service calls (see the system-wide row below) — a port would need to pre-bake τ/γ as scenario-seeded `:field`s or `:const`s rather than calling out mid-tick. |
| §2.4 County-state carry/defaults | **PORTABLE WITH D-RECORD** | Read-with-fallback-default pattern is trivially `field-of`/`:field`-with-default shaped. Same external-service-input caveat as §2.3 for the "live" branch of each of the 9 sources; the fallback-default branch alone is portable today. |
| §2.5 Precarity derivation | **PORTABLE NOW** | Pure add/multiply/nested-clamp, no query lane, no libm. Hardcoded (non-defines) `pter_fraction`/`nilf_fraction` — D-record, transcribe verbatim. |
| §2.6 Coefficient smoothing | **PORTABLE NOW** | Same shape as §2.3's own arithmetic. Hardcoded `alpha`/`gamma_import` — D-record. |
| §2.7 Vol I wage pressure (the sigmoid) | **PORT-QUESTION — RESERVED, architecture ruling needed before any transcription** | `exp` IS a declarable intrinsic (landed), so this is NOT a missing-lane blocker in the mechanical sense — it is blocked by the standing "no imposed functional forms" ruling (ADR172 ruling 5/ADR173) restated in this task's CURRENT BSL surface: a stipulated sigmoid must not simply be copied in, the S-curve is supposed to emerge from a measure over dispersion. Escalate to the Director/architecture gate, do not D-record around it. Also fully dormant on every canonical scenario (§5) — no conformance oracle exists to even prove a transcription correct today. |
| §2.8 Accumulation loop (reserve-ratio producer) | **BLOCKED — Real→Int demotion via `round()`, no BSL intrinsic** | `floor` (landed) does not cover this: `round()` is round-half-to-even, `floor(x+0.5)` diverges at exact half-integer ties. Name the gap precisely: needs either a declared `round` intrinsic or a proven-equivalent `floor`-based reformulation with a written derivation (per this repo's own cross-implementation-tolerance standard) — neither exists today. Also structurally dormant on every canonical scenario including `single_county` (§5) — `dispossession_data_source` unwired everywhere in the qa harness, so `compute_dynamics` always returns `None` there. |
| §2.9 Imperial rent (Leontief pipeline) | **NOT-A-PACK for this system's own code; BLOCKED (external-service boundary) for what it delegates to** | The module's OWN logic (`imperial_rent.py`) is pure Python validation/orchestration over 4 injected objects with zero graph reads/writes and zero arithmetic of its own — there is no "rule" here for BSL to express; the actual Leontief math lives entirely inside `production_chain_calculator`/`industry_county_allocator`, out of scope for this inventory and requiring its own dossier if pursued. Structurally dormant on every canonical scenario, including `single_county` (§5) — zero conformance evidence exists anywhere in the estate. |
| §2.10 Vol II circulation | **BLOCKED — graph-level opaque-object cross-tick state, no BSL equivalent** | `advance_circuit`/`update_depreciation_fund`'s own arithmetic (§4 items 6-9) is portable, ordinary float math — but its cross-tick continuity is threaded via `prev_county_states.circulation_state`, itself only reachable because the WHOLE `SimulationTickState` round-trips through `graph.set_graph_attr("tick_dynamics", ...)` as one opaque dict (§2.17) — there is no per-node/per-field storage path for `CircuitState`'s 5 fields + `InventoryState`'s 5 + `DepreciationFundState`'s 4 (14 scalars, before even reaching `DisproportionalityCrisis`/`CirculationCrisisAssessment`) short of individually `deffield`-declaring every one of them, which is possible in principle once attribute-storage widening lands but is NOT today (Slice 4, Director-escalation-gated per the CURRENT BSL surface). Also dormant on every canonical scenario including `single_county` (`turnover_profile_source` unwired everywhere, §5). Own process-cached-defines-accessor desync (§2.10 own finding) is a separate D-record, subordinate to the storage blocker. |
| §2.11 Crisis-phase detection | **BLOCKED (same storage mechanism as §2.10), but the SMALLEST nested object in this system — the cheapest thing to unblock first if the graph-metadata gap is ever partially lifted** | `CrisisState` is only 6 scalars (`phase` enum + 2 int counters + duration + nullable `peak_severity` + `cumulative_wage_compression`), all individually simple `deffield` candidates (enum landed, ints/reals landed) — unlike §2.10's ~14+ fields. Still blocked TODAY by the same all-or-nothing graph-level-dict round-trip, but name it separately: a partial per-System attribute-widening exception for this one 6-field object would unblock it well before Vol II/III. Wage-compression duplicate-vs-dead-function divergence (§2.11(b)) is a D-record (transcribe the live inline copy). |
| §2.12 Vol III financial layer | **NOT-A-PACK overall; the national endogenous-interest formula alone is PORTABLE NOW** | `endogenous_interest_rate`/`loan_market_tightness`/`reserve_army_signal`/`_economy_wide_profit_rate` (§4 items 11-12) are all plain arithmetic reachable via a `fold` over territory/county nodes (LANDED query lane) once county-level fields are portable — genuinely one of the stronger candidates here. But: (a) its INPUT (`tensor_registry`, `distribution_calculator`, etc.) is the same external-service blocker as §2.3/§2.4; (b) its OUTPUT is written to the SAME graph-level `"national_financial"` opaque dict as §2.10/§2.17 (blocked); (c) `EndogenousInterestRate`'s `model_validator` invariant (`rate < ceiling`, raising `ValueError`) has no BSL assertion-construct equivalent (minor, named gap). This is the ONE step with real canonical-scenario evidence (`single_county`, §5) — highest-value target for a future dedicated dossier once the storage/service blockers are addressed elsewhere. |
| §2.13 Class transitions (Feature 016) | **NOT-A-PACK** | This system's own glue is a year-clamp + a milestone-threshold loop (`sorted(milestones)`, portable) — the entire transition mathematics is a single opaque `services.transition_engine.simulate_transitions(...)` call, 100% out of scope, deserving its own dossier if pursued. Dormant on every canonical scenario (`transition_engine` unwired everywhere, §5). |
| §2.14 Bifurcation risk | **PORTABLE WITH D-RECORD — the strongest candidate in this entire system** | Genuine GRAPH reads (typed `nodes`/`SOLIDARITY` edges), the fold/exists/typed-neighbor shape the LANDED query lane serves directly; only in-memory inputs are `previous_distribution`/`current_distribution` (comparable to reading two prior/current node-attr snapshots). D-records needed: flatten `ideology.agitation` (a 2-level dict on SOCIAL_CLASS) to a top-level field; `blend_weight=0.6` hardcoded-not-defines-backed default. No libm, no round/floor demotion, no graph-level-metadata dependency. |
| §2.15 Distribution validation | **BLOCKED — no assertion/abort construct** | `raise ValueError` on an invariant violation has no named BSL equivalent in the CURRENT surface (distinct, minor gap — same class as §2.12's `model_validator`). The sum+tolerance arithmetic itself is trivial. |
| §2.16 Tick summary | **PORTABLE WITH D-RECORD** | Every aggregate is `fold`-shaped (sum/count/weighted-sum over `county_states`, i.e., over TERRITORY nodes) — the LANDED query lane's `fold` over `nodes` should serve this directly (name the D-record as "confirm whole-node-type `fold`, not just typed-neighbor `fold`, at the query-evaluation train's next design gate" if that distinction is not yet nailed down). `HOURS_PER_YEAR` process-cache desync — subordinate D-record. |
| §2.17 Graph serialization (`tick_dynamics`/`national_financial` graph-level dicts) | **BLOCKED — no BSL equivalent, not on any current roadmap slice** | This is the SYSTEM-WIDE, dominant blocker. `graph.set_graph_attr`/`get_graph_attr` store an arbitrary nested-Pydantic-object dict under ONE graph-level key — BSL's `GraphSubstrate` (per Territory's own report and the CURRENT BSL surface) is node/edge typed-`deffield` storage only; there is no graph-level metadata concept at all. None of Slices 2 (edge attributes), 3 (hyperedge/metric), or 4 (attribute-storage widening, Director-gated) address graph-LEVEL storage — this needs a NEW, currently-unnamed lane, or a full redesign of this system's cross-tick continuity mechanism around per-node fields (a genuine deviation from port-as-is, since it would change WHERE state lives, not just how it's computed). Every one of §2.10/§2.11/§2.12's cross-tick continuity ultimately depends on this row. |
| §2.18 Hex substrate | **NOT-A-PACK / UNVERIFIED** | No-op guard is trivial and portable; internals unread (§2.18, dormant on every canonical scenario — `hex_grid` never wired). Would need its own read-through before any verdict on the substance. |
| **System-wide: the ~28-field ServicesProtocol external-service boundary** | **BLOCKED — no BSL equivalent, the OTHER dominant blocker** | A closed BSL rules-as-content algebra has no construct for "call an injected Python object's method mid-tick and branch on whether it returned `None`." Every one of the 9 Wave-6 data sources (§2.4), the 3 Feature-012/013/014 national calculators (§2.3), the Vol I/II/III calculator families (§2.7-§2.13), and the Leontief pipeline's 4 sources (§2.9) sits behind this exact same wall. The only tractable path is pre-baking each source's output into scenario-seeded/`:const` fields BEFORE a BSL tick runs (a data-build-time responsibility, not a tick-time one) — which is workable in principle for sources that are genuinely once-per-scenario or slowly-varying, but is a real architectural decision, not a mechanical D-record, for sources meant to be re-queried per county-year (all 9 of §2.4's). |
| Enum fields (`CrisisPhase`, `ReplacementCyclePosition`, `InventoryDiagnosis`) | **PORTABLE NOW (mechanically)** | `defenum`/`deffield ... enum` is LANDED (ADR195/196) — no gap here, contingent on the fields' HOST OBJECT being portable at all (most are nested inside the blocked §2.10/§2.11 objects). |
| Money-like scalar fields (`tick_phi_hour`, `tick_capital_stock`, `tick_median_wage`, `reserve_army_stock`, the ~14 Vol II/III currency fields) | **PORTABLE WITH D-RECORD** | Same `deffield ... int extensive` bare-scaled-Int workaround Territory's report already established (ADR183 declared-deviation class) — Currency-typed field storage is refused; every landed pack already routes around it this way. |

---

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/economics/tick/test_system.py` | 3,099 | **Primary conformance oracle.** 152 `def test_` functions across 26 `class Test...` groups (`TestNationalParameterComputation`, `TestCountyStateComputation`, `TestComputeVol1Layer`, `TestComputeAccumulationLoop`, `TestFullTickPipeline`, `TestBootstrapCountyStates`, `TestDetermineYear`, `TestGetTerritoryFips`, `TestGetProfitRate`, `TestCheckCrisisTriggers`, `TestSimulateTransitions` + `TestSimulateTransitionsMutationKillers`, `TestStepContextExtraction`, `TestUpdateCoefficients`, `TestComputeBifurcationRisk`, `TestComputeTickSummary`, `TestDerivePrecarity`, `TestValidateDistributions`, `TestComputeNationalParams`, `TestEconomicsFallbackInstrumentation`, `TestGammaFallbackHonestySweep`, `TestEconomyWideProfitRate`, `TestVol3FinancialLayerSentinelObservability`, `TestComputeCountyStates`, `TestWriteHexSubstrate`, `TestComputeNationalFinancialState`) — near-1:1 coverage of every method in §2's catalog. The single richest per-method conformance-vector source in the repo for this system; a port's `.bscn` fixtures should mine this file class-by-class. |
| `tests/unit/economics/tick/test_types.py` | 718 | Schema-level: field bounds, `model_validator` invariants (sum-to-one, `rate < ceiling`) — not tick-behavior, but documents every invariant a port must also either enforce or explicitly drop (§6's assertion-construct gap). |
| `tests/unit/economics/tick/test_graph_bridge.py` | 593 | Round-trip serialization tests for `write_tick_state_to_graph`/`read_tick_state_from_graph` — directly documents the graph-level-metadata shape (§6's dominant blocker) byte-for-byte; essential reading for whatever design eventually addresses that blocker. |
| `tests/unit/economics/tick/test_circulation_layer.py` | 306 | Vol II circulation behavioral tests — conformance-oracle candidate for §2.10 once/if its storage blocker is resolved. |
| `tests/unit/economics/tick/test_flow_accrual.py` | 316 | §2.1/§2.2 behavioral contract — good conformance-oracle candidate (PORTABLE-NOW territory). |
| `tests/unit/economics/tick/test_financial_integration.py` | 538 | §2.12 behavioral tests — the one live-on-canonical-scenario subsystem; strong conformance-oracle candidate for a future Vol III dossier. |
| `tests/unit/economics/tick/test_u9_9_national_financial_layer_propagation.py` | 140 | Endogenous-interest-rate specific — directly tests §2.12's portable-arithmetic core. |
| `tests/unit/economics/tick/test_year_ceiling_crossing.py` | 61 | Regression test for the U2 honesty-sweep year-ceiling removal (§2.3's `floor_year`/no-ceiling finding) — narrow but load-bearing for that specific historical defect. |
| `tests/unit/economics/tick/test_county_fips_bridge.py` | 292 | `resolve_county_identity` behavioral tests — directly relevant to §5's dormancy mechanism (county-free vs county-bearing scenarios). |
| `tests/unit/economics/tick/test_{bracket_ratio,cpi_deflator,employment,renter_share,unemployment,wage}_source.py` | 47-61 each | 6 small, narrow behavioral tests, one per §2.4 Wave-6 data source's wiring/fallback contract — schema/wiring tests, not conformance oracles for this system's own math. |
| `tests/integration/economics/test_tick_integration.py` | 184 | Cross-module integration (not this System's unit boundary) — useful for confirming the full pipeline wires together, not a per-formula oracle. |
| `tests/integration/economics/test_detroit_wiring.py` | 202 | Documents the `detroit_tri_county` 5-tick-never-crosses-boundary dormancy mechanism (§5) directly. |
| `tests/integration/economics/tick/test_facade_behavioral_fence.py` | 99 | Guards `_compute_imperial_rent`'s behavioral fence (return type, exception hierarchy, event-ordering) per Spec 058/FR-007 — schema/contract test, not a math oracle (§2.9 is NOT-A-PACK anyway). |
| `tests/integration/economics/tick/test_imperial_rent_real_wiring.py` | 111 | One of the only places the Leontief pipeline (§2.9) is exercised with anything resembling real wiring — worth a closer read if that dossier is ever opened. |
| `tests/unit/config/test_defines.py` | 429 | Broad `GameDefines` schema test, not system-specific — confirms bounds cited in §2/§6's defines table are schema-enforced. |
| `tests/unit/engine/test_system_order.py` | 300 | Confirms tick-position ordering (§5's "position 4.0" claim) across the whole engine, not this system specifically. |
| `tests/baselines/detroit-tri-county-5t.json` | 932 | The committed 5-tick golden itself — the primary EVIDENCE artifact for §5's dormancy finding (13 `at_rest` channels), not a test file per se. |

**qa:regression byte-gate coverage.** Per §5: on the 6 scenarios `regression_test.py`
wires with the Vol III bundle, this system's output is captured by
`graph_content_hash` (same mechanism Territory's report cited,
`tools/regression_test.py::graph_content_hash`) — but that coverage is REAL only for
Step 2 (national params) on 5 "county-free" scenarios, and for §2.12/§2.11's
county-level Vol III+crisis-detector path on `single_county` alone. **§2.4, §2.5, §2.6
(with real data), §2.7, §2.8, §2.9, §2.10, §2.13, §2.14, §2.16, §2.18 have ZERO
canonical-scenario byte-gate coverage** — a port targeting any of those needs
dedicated hand-built `.bscn` conformance fixtures, mirroring Territory's own finding
but for roughly half of this system's 19-entry computation catalog rather than one
phase of four.

---

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`) by a second, read-only
pass, in the manner of the Territory inventory's own "Adjudicated verdict"
section. Five corrections, three confirmations, one inadequate-coverage note.
The two dominant blockers survive; §5's dormancy analysis does not.

1. **CORRECTION — there are 12 canonical `qa:regression` scenarios, not 13;
   `detroit_tri_county` is not one of them.** §5 states "13 scenarios are
   declared in `regression_scenarios.py`" and lists `detroit_tri_county` among
   them. `SCENARIOS` (`tools/regression_scenarios.py:37-133`) has exactly 12
   keys — `imperial_circuit, two_node, starvation, glut, fascist_bifurcation,
   single_county, mitterrand, syriza, weimar, debs, bernie_valve, org_probe` —
   and `create_scenario`'s factory dispatch (`:167-190`) names nine factories,
   none of which builds it. The file says so in its own words at `:2011-2013`:
   *"detroit_tri_county: not one of the five canonical qa:regression scenarios
   (it is the committed e2e headless-runner baseline, spec-102/spec-065)"* — it
   appears only as `bundle_field`/`at_rest` fixture rows checked statically
   against `tests/baselines/detroit-tri-county-5t.json`. Net effect: the
   report's *conclusion* about that baseline strengthens (it carries no
   `qa:regression` byte-gate weight at all, being a different estate), but the
   scenario inventory it rests on is wrong.
2. **CORRECTION — load-bearing, and it reverses §5's central claim: SIX of the
   twelve canonical scenarios carry `county_fips`, not one.** §5 asserts "NONE
   carry `county_fips` on any territory except `single_county` and
   `detroit_tri_county`", citing `tools/regression_test.py:172-175`. **That is a
   stale source**: its subject is the ORIGINAL FIVE (*"the five regression
   scenarios carry no `county_fips`"*), written before the U13/ADR140 electoral
   goldens joined `SCENARIOS`. Verified against the tree —
   `src/babylon/engine/scenarios/single_county.py:79,92,116` stamps
   `county_fips=WAYNE_COUNTY_FIPS`; `mitterrand`
   (`src/babylon/engine/scenarios/electoral_goldens.py:225`), `syriza` and
   `bernie_valve` (`:498`) build on `create_single_county_scenario()` and
   inherit it; `weimar` (`:331-332`) and `debs` (`:423-424`) stamp it by
   `model_copy`. This system scopes its county list from exactly that attribute
   (`_get_territory_fips`, `src/babylon/domain/economics/tick/system/__init__.py:225,425-428`,
   via `resolve_county_identity`, `graph_bridge.py:44-77`), and the harness runs
   ticks 1..52 (`DEFAULT_MAX_TICKS = 52`, `tools/regression_test.py:81`;
   `for tick in range(1, max_ticks + 1)`, `:1054`), so the `tick % 52 == 0` gate
   (`system/__init__.py:174`) fires once, at tick 52, on every scenario.
   Furthermore four of the six —
   `WAYNE_CALCULATOR_SCENARIOS = {single_county, mitterrand, syriza,
   bernie_valve}` (`tools/regression_scenarios.py:151-153`) — take
   `build_single_county_overrides`, i.e. a real Wayne `tensor_registry`
   (`tools/regression_test.py:1030-1034`), which is precisely what §2.8's
   accumulation loop and §2.11's crisis detector need: `_compute_accumulation_loop`
   returns early **only when `tensor_registry` AND `dispossession_source` are
   both `None`** (`system/__init__.py:1285-1288`), not when either is. **§5's
   "the entire per-county pipeline executes zero iterations" and "`single_county`
   is the ONLY canonical scenario where the annual pipeline fires with real
   per-county data" are both over-broad**, and the byte-gate coverage surface
   for §2.4/§2.5/§2.6/§2.11/§2.12/§2.15/§2.16 is materially larger than the
   report claims. What remains genuinely UNVERIFIED (and is what a re-read must
   settle, per-scenario): whether `compute_dynamics` returns non-`None` on those
   four given `dispossession_data_source` is unwired, i.e. whether a positive
   `reserve_ratio` is actually produced.
3. **CORRECTION — §2.14 is not "the strongest candidate in this entire
   system"; it is doubly blocked.** (a) **D102**: `_compute_solidarity_density`
   reads `attrs.get("role", "")` off every scanned `social_class` node, not off
   any rule subject (`src/babylon/domain/economics/crisis/bifurcation.py:146-153`,
   all-pairs at `:161-179`), and `_compute_legitimation` does the same for
   `ideology.agitation` (`:210-218`). As a fold that is `(field-of it
   social-class/role)`, and `field-of` naming an `:enum-type`-declared field is
   **REFUSED AT LOAD** — `rust/crates/babylon-bsl/src/typecheck.rs:266-289`
   (`check_no_field_of_on_enum_field`): *"field-of is not extended to
   enum-declared fields (§2.13, D102)"*. This directly contradicts §3's own
   claim that `CrisisPhase`/`role`-class enums "are all candidates for
   `defenum`, not blocked by the enum-storage gap Territory's report found".
   (b) **String identity**: the same loop filters on
   `attrs.get("county_fips") != fips` (`bifurcation.py:150-151`) — a 5-char
   string comparison, and `deffield` has no string row at all (`Str` is
   typechecker-only, confined to `:material-basis`/vector ids,
   `docs/reference/bsl-language.rst:2394-2398`). §2.14(c) lists `county_fips`
   among its reads and never flags it. The row must move from **PORTABLE WITH
   D-RECORD** to **BLOCKED — D102 + no string-identity field type**, portable
   only under an int-ordinal `role` encoding *and* a county-identity encoding
   the estate has not designed. The same string-identity gap applies to §2.4,
   whose whole per-county carry is FIPS-keyed and which the table also rates
   PORTABLE WITH D-RECORD.
4. **CORRECTION — §2.7's "PORT-QUESTION … escalate to the Director" understates
   a CLOSED ruling.** The escalation has already happened:
   `ai/decisions/ADR188_intrinsic_rider_slate_dispositions.yaml:54-57` — *"Row 7
   exp — the three stipulated-sigmoid sites (P(S|A) survival calculus, the
   defection probability, the wage-pressure sigmoid) RE-DERIVE AS MEASURES at
   the port"* — names **this exact function** by name, and
   `rust/crates/babylon-bsl/src/declarations.rs:116` mechanically refuses an
   intrinsic named `sigmoid` (`PROHIBITED_INTRINSIC_NAMES`, enforced at `:684`).
   What is owed is undone *design* work, not a ruling. The ReserveArmySystem
   inventory in this same batch — covering the *same* `DefaultWagePressureCalculator`
   through its other caller — got this framing right; this report should be
   reconciled to it rather than left contradicting it.
5. **CORRECTION — no RESERVED-LINE finding is recorded anywhere in this
   inventory** (`grep -c -i "RESERVED-LINE"` → 0), yet this is the
   ideologically densest system in the batch. At least three surfaces are
   described and never flagged: the **bifurcation-risk directional score**
   (§2.14, `[-1,+1]` fascism↔revolution — the Constitution's own bifurcation
   law, and `crisis.bifurcation_solidarity_weight`/`.bifurcation_burden_weight`/
   `.bifurcation_event_threshold`, `defines.yaml:42-45`); the five-share
   `ClassDistribution` and the Feature-016 transition engine that moves it
   (§2.13); and `crisis.dispossession_cascade_milestones`
   (`defines.yaml:46-49`). A port describes these and never re-tunes them; the
   inventory owes the explicit section.
6. **CONFIRMATION — the `round()` gap is real and precisely named.**
   `src/babylon/domain/economics/reserve_army/accumulation.py:115,121` use
   Python `round()` (round-half-to-even);
   `rust/crates/babylon-bsl/src/declarations.rs:110` —
   `DECLARABLE_INTRINSICS: [&str; 3] = ["exp", "log", "floor"]`, pinned by the
   test at `:1177`. No `round` intrinsic exists, and `floor(x+0.5)` diverges at
   exact half-integer ties. §2.8's BLOCKED row stands as written.
7. **CONFIRMATION, with added teeth — §2.17's graph-level metadata blocker is
   the system-wide dominant one, and is harder than described.** Zero graph-level
   attribute construct exists in either Rust crate (`rg -n
   "graph_attr|graph_attribute|GraphAttr" rust/crates/babylon-graph/src/
   rust/crates/babylon-bsl/src/` → 0 hits). The R9 chapter-C3 carrier-NodeType
   ruling the estate would reach for is itself unserved: it prescribes
   `(field-of (the NodeType/…) …)` / `(update-node (the NodeType/…) …)`
   (`docs/reference/bsl-language.rst:2650-2669`) and `the` is in
   `UNSERVED_EXPRESSION_HEADS` at **slice 2**
   (`rust/crates/babylon-bsl/src/evaluator.rs:504-506`). Every §2.10/§2.11/§2.12
   cross-tick continuity claim inherits this. **BLOCKED stands, reinforced.**
8. **CONFIRMATION — tick position 4.0, and the channel table's load-bearing
   writes.** `src/babylon/domain/economics/tick/system/__init__.py:124`
   (`position: ClassVar[float] = 4.0`) against the 34-member `_SYSTEM_CLASSES`
   (`src/babylon/engine/simulation_engine.py:328-363`; order derived by sorting
   on `position`, `:376-377`). `rg -ln "reserve_ratio"
   src/babylon/engine/systems/` → `market_scissors.py`, `reserve_army.py`
   exactly as claimed; `rg -ln "foreclosure_rate|eviction_rate"
   src/babylon/engine/systems/` → `dispossession_events.py` alone. Both confirmed.

**FINAL VERDICT: BLOCKED — CONFIRMED on both named blockers (the ~28-field
ServicesProtocol external-service boundary; graph-level opaque-object metadata
storage, the latter reinforced by C3's own accessor being slice-2 unserved) —
but the portable island is SMALLER and the canonical coverage is LARGER than
this report finds.** §2.14 leaves the island (D102 + no string-identity field
type, correction 3), §2.4's FIPS-keyed carry inherits the same string gap, and
§2.7 is a ruled prohibition with owed design work rather than an open
port-question (correction 4). Simultaneously, six canonical scenarios — not one
— drive the annual pipeline with a non-empty county list at tick 52
(correction 2), so the "structurally dormant on every canonical scenario"
premise under §2.4-§2.16 does not hold as written.

**INADEQUATE COVERAGE — a re-read is required, and must add exactly this.**
(i) A per-scenario re-derivation of §5's dormancy table across all **six**
county-bearing scenarios (`single_county, mitterrand, syriza, weimar, debs,
bernie_valve`), distinguishing the four with a live Wayne `tensor_registry`
(`WAYNE_CALCULATOR_SCENARIOS`) from the two with MELT only, and stating for each
of §2.4-§2.16 whether it fires at tick 52 — replacing the stale
`regression_test.py:172-175` citation, which speaks only to the original five.
(ii) An explicit **RESERVED-LINE** section (correction 5). (iii) A D102 +
string-identity sweep over every row currently rated PORTABLE, since §2.4 and
§2.14 are both affected and neither gap is mentioned once in the report. (iv)
§2.18 (hex substrate) remains self-declared UNVERIFIED with its module unread;
that is acceptable for a Phase-1 scope boundary only if the re-read restates it
as an open row rather than a verdict.
