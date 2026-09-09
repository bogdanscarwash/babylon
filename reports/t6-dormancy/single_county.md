# T6 dormancy re-derivation — `single_county`

Program 29 train T6 (issue #563, charter ADR198). Scenario: `single_county`
(Wayne County, MI, FIPS `26163`). Reader scope: this scenario only. Read-only
source audit; nothing executed (swarm rule). Every claim cites file:line;
anything not checkable from source is marked UNVERIFIED.

## Scenario header

- Factory: `create_single_county_scenario()` — `src/babylon/engine/scenarios/single_county.py:50`.
  One Territory `T001` with `county_fips="26163"` (lines 113-120), two
  SocialClasses both stamped `county_fips=WAYNE_COUNTY_FIPS` (lines 79, 92),
  edges EXPLOITATION + WAGES + TENANCY (lines 95-129). `county_fips` is a real
  declared field on both models (docstring lines 10-17). Registered in
  `SCENARIOS` at `tools/regression_scenarios.py:71-76` with empty
  `defines_overrides`; member of `WAYNE_CALCULATOR_SCENARIOS`
  (`tools/regression_scenarios.py:151-153`).
- Calculator wiring: `tools/regression_test.py:215-258`
  (`build_single_county_overrides`) = `_build_vol3_calculator_overrides`
  (lines 186-207 → `create_financial_services`,
  `src/babylon/domain/economics/factory.py:477-490`: distribution, interest,
  credit_cycle, credit_aggregate, fictitious_capital, rent, housing,
  counter_tendency, value_basis, financial_crisis, z1, housing_data) **plus**
  `melt_calculator` (regression_test.py:206) **plus** a real-FIPS
  `TensorRegistry` hydrated from `tests/fixtures/single_county_wayne.json`
  (fips `"26163"` str, year **2010**, verified from the fixture itself).
  Everything else in `ServiceContainer.create` defaults to `None`
  (`src/babylon/engine/services.py:212-258`, 298-446).
- Run shape: `DEFAULT_MAX_TICKS = 52` (`tools/regression_test.py:81`); loop
  `for tick in range(1, max_ticks + 1)` (regression_test.py:1054) through
  `simulation_engine.step()` with a shared `persistent_context`
  (regression_test.py:1021-1061; `src/babylon/engine/simulation_engine.py:503-575`).
  The qa harness never injects `_base_year` (that is the headless runner only,
  `src/babylon/engine/simulation/_legacy.py:590-592`), so
  `graph.get_graph_attr("base_year", 2010)` → 2010
  (`src/babylon/domain/economics/tick/system/__init__.py:420-423`).

## Live fields (producer/consumer fires in this scenario)

Services boundary (of the fields TickDynamics reads; protocol list at
`src/babylon/kernel/services.py:34-88`):

1. `melt_calculator` — the gate itself (system:198) and Step 2 tau
   (system:542). Fixture covers 2011 (`tests/fixtures/vol3_melt_national.json`,
   gdp 2010-2023 / employment 2010-2024, verified). LIVE.
2. `tensor_registry` — profit rate (system:2203-2212), surplus
   (system:2230-2239), organic composition (system:1356-1362), crisis-detector
   feed (system:1019-1059), `_get_best_tensor_year` carry-forward
   (system:2157-2185). Wayne 2010 tensor present; tick-52 year is 2011, so the
   **2-year carry-forward is exercised on the very first boundary**
   (2011 → sentinel → 2010 hit). LIVE.
3. `distribution_calculator` — real `s = p + i + r + t` for Wayne
   (system:2017-2039); FRED fixture series B230RC0Q173SBEA / A054RC1Q027SBEA /
   FEDFUNDS / TCMDO all cover 2011 (verified in
   `tests/fixtures/vol3_fred_series.json`). LIVE — this is the scenario's
   stated purpose (factory docstring, single_county.py:1-26).
4. `fictitious_capital_calculator` — (system:1964-1970); NCBEILQ027S covers
   2011. LIVE.
5. `credit_aggregate_source` — `_build_credit_state` (system:1845-1856);
   TCMDO 2011 present. LIVE.
6. `financial_crisis_assessor` — `_assess_county_financial_crisis`
   (system:2070-2082, 2146-2155) runs because `surplus_distribution` is
   present. LIVE (whether it returns non-None for Wayne 2011: UNVERIFIED).
7. `rent_calculator` — fires (system:2052-2058) but its stub adapter returns
   None for all three categories (factory.py:454-463), so it always yields
   `NoDataSentinel("Agricultural rent unavailable for 26163/2011")`
   (rent/calculator.py:109-115) → sentinel recorded + throttled log.
   LIVE-but-permanently-sentinel; `rent_extraction` stays `None`.
8. `housing_calculator` — fires (system:2061-2067) but `CensusHousingLoader`'s
   hardcoded table has Wayne only for 2015/2020/2022
   (`src/babylon/domain/economics/data_adapters.py:121-137`), not 2011 →
   `NoDataSentinel` (rent/calculator.py:192-198). LIVE-but-sentinel;
   `housing_decomposition` stays `None`.
9. `defines`, `event_bus`, `economics_fallbacks` — LIVE (crisis config,
   the imperial-rent pipeline-unwired event, gamma fallback records, Vol III
   sentinel tallies: system:534-588, 1158, 1967, 2036, 2055, 2064, 2330;
   imperial_rent.py:172-195).
10. `z1_source` / `housing_data_source` — wired and consumed *inside* the
    fictitious/housing calculators (factory.py:392-393, 416, 472-474);
    TickDynamics never reads them directly. Indirectly live.

CountyEconomicState fields (`src/babylon/domain/economics/tick/types.py:320-396`)
that carry real or computed values: `fips` ("26163"), `year` (2011),
`u6_rate`/`pter_rate`/`nilf_rate` (precarity deriver, system:854-878),
`class_distribution` (bootstrap defaults, validated live, system:2460-2486),
`crisis_state` (4 quarterly evals against the real Wayne r = 0.0594 — computed
from the fixture: s=3.2342e9, c=2.4037e10, v=3.0368e10; 0.0594 > `r_threshold`
0.05, `src/babylon/config/defines/economy_basic.py:67-72`, so the phase stays
NORMAL by read; UNVERIFIED by execution), `surplus_distribution`,
`financial_crisis`.

## Dormant fields (declared, never exercised here)

Unwired (`None` in the qa overrides) → their branches early-return or carry
defaults:

- `basket_calculator`, `gamma_calculator` — Step 2 uses GameDefines defaults
  and records fallbacks (system:550-588).
- `capital_calculator` — `capital_stock` stays 0.0 (system:747-751); derived
  `profit_rate` therefore collapses to s/v ≈ exploitation rate
  (tick/derived_rates.py:66-83).
- `throughput_calculator` — π=1.0, D=2.0 defaults (system:754-760).
- `unemployment_source` (0.05), `housing_source` (renter 0.0), `wage_source`
  (21.0), `employment_source` (100_000.0 bootstrap), `cpi_source` (deflator
  1.0), `income_source` (bracket 0.0) — system:766-804, 630-713. Note the
  100k employment default propagates into `reserve_army_signal` weighting,
  `county_employment` for distribution, and TickSummary weighting.
- `reserve_army_data_source` → Vol I layer early-returns (system:1196-1197).
- `dispossession_data_source` → no bankruptcy/foreclosure/eviction reads
  (system:1286-1288, 2386-2396).
- `transition_engine` → Step 6 early-returns (system:2366-2367). **The
  Feature-016 five-share transition engine never runs in this scenario.**
- `turnover_profile_source` → circulation layer early-returns
  (system:1409-1410); `inventory_data_source` / `depreciation_data_source`
  unreachable behind it (system:1455-1473). `circulation_state` stays the
  default and its `tick_liquidity_ratio` etc. stamps serialize default values
  (graph_bridge.py:200-234).
- `hex_grid` → Step 9 no-op (system:392-394).
- `periphery_labor_source`, `final_demand_source`, `industry_county_allocator`,
  `production_chain_calculator`, `bea_industries` → Step 4 graceful
  degradation: `_spec_057_pipeline_wired` false (imperial_rent.py:158-169),
  one `QcewCarryForwardEvent` unwired signal published (imperial_rent.py:86-88,
  172-195), `phi_hour` stays 0.0 via `_stub_zero_pass_through`
  (imperial_rent.py:221-230).
- Accumulation loop (Step 3.6, system:1243-1334): invoked (tensor_registry
  wired) but produces nothing — `occ_current` for 2011 is None (exact-year
  read, no carry-forward, system:1336-1362) while `occ_prior` for 2010 is
  real, and `compute_dynamics` needs BOTH occs
  (reserve_army/accumulation.py:106-117) → returns None → no
  `reserve_army_stock`/`reserve_ratio` stamp. Note the **inverted carry
  semantics**: `_get_organic_composition` misses 2011 while
  `_get_best_tensor_year` (used by the financial layer) carries 2011→2010.

Wired but never read by TickDynamics (grep over `src/babylon/domain/economics/tick/`):

- `interest_calculator` — "calibration-only" per the U9 comment
  (system:1763-1765). DORMANT.
- `credit_cycle_detector` — never read; the graph attr
  `"credit_cycle_phase": "expansion"` is a **hardcoded literal**
  (graph_bridge.py:106). DORMANT.
- `counter_tendency_calculator`, `value_basis_converter` — no reference in the
  tick estate. DORMANT. (The latter's CPI path is unrelated to the unwired
  `cpi_source`; deflator stays 1.0 regardless.)

Flow-accrual channel (spec-109 A7): `_accrue_flows` (system:315-355) no-ops
for all 52 ticks — T001 has no `tick_phi_hour` until the tick-52 write, so
`flow_phi_accrued`/`flow_wage_accrued` are only ever *reset to 0.0* by
`_reset_flow_accrual` (system:357-372) at tick 52, never accrued. DORMANT
within the canonical horizon (needs a second boundary, tick ≥ 104).

## The annual pipeline

The tick-52 claim **verifies, with a sharpening**: `WEEKS_PER_YEAR = 52`
(`src/babylon/config/defines/tunables.py:77-80`); the gate is
`tick % WEEKS_PER_YEAR != 0` (system:174). Ticks 1-51 take the non-boundary
branch (re-stamp skipped — nothing persisted yet; accrual empty-domain no-op;
system:176-187). Tick 52 is the **only** boundary in the canonical run, and it
is the **final** tick. So the annual pipeline fires exactly once per run, as
year **2011** (base 2010 + 52//52, system:420-423; `existing_state` is None on
this first boundary, system:203-213).

What that one firing computes for `single_county` (all line refs in
`tick/system/__init__.py` unless noted):

1. Bootstrap: `prev_county_states = {}` — T001 has no `tick_capital_stock`
   yet (480, 451-512). `county_fips = ["26163"]` via `_get_territory_fips` →
   `resolve_county_identity` (425-449; graph_bridge.py:73-77). county_states
   non-empty — the county-free dormancy pattern documented at
   `tools/regression_scenarios.py:406-451` does **not** apply here; those
   `financial_*` "at_rest" channels are LIVE in this scenario.
2. Step 2 (514-628): real 2011 MELT; gamma_basket/gamma_III from defines
   defaults with fallback records; `tau_effective = tau * gamma_basket`.
3. Step 3a (715-852): all per-county data sources unwired → documented
   defaults (K=0, π=1.0, D=2.0, U3=0.05, renter 0, wage 21.0, employment
   100k, deflator 1.0, bracket 0.0); five-share default distribution
   (0.01/0.09/0.40/0.35/0.15, 822-830).
4. Step 3a+ (854-878): precarity (U-6/PTER/NILF) derived from the lumpen
   share — live computation over defaults.
5. Step 3b (880-910): coefficients initialized from raw values.
6. Step 3.5 Vol I: dormant (unwired).
7. Step 3.6 accumulation loop: invoked, yields no dynamics (see above).
8. Step 4 imperial rent: dormant — unwired signal event, φ stays 0.0.
9. Step 4.5 circulation: dormant (unwired).
10. Step 5 crisis triggers (941-1017): detector built from defines; 4
    quarterly evals with the real Wayne r=0.0594 (carry-forward 2011→2010,
    1019-1059); r > r_threshold=0.05 → NORMAL throughout by read (UNVERIFIED
    by execution); no wage compression, no crisis events.
11. Step 5.5 financial layer (1737-2155): economy-wide profit rate real
    (`r = Σs / Σ(s/r)` telescopes to Wayne's 0.0594, 1858-1919); reserve-army
    signal s_r = 0 (U3 0.05 < `interest_reserve_reference` 0.08,
    graph_bridge.py:511-542, capital_vol3.py:193-200) → tightness 0 →
    endogenous rate = r × base (credit/endogenous_interest.py:43-116);
    credit state real; fictitious real; Wayne distribution real (s = p+i+r+t);
    rent + housing sentinels; crisis assessor runs;
    `NATIONAL_FINANCIAL_ATTR` published (graph_bridge.py:433-450) and saved to
    `persistent_context` (simulation_engine.py:487-491).
12. Step 6 transitions: dormant (unwired).
13. Step 5b bifurcation (2241-2306): calculator lazily constructed
    (2269-2275) but `.compute()` **never called** — the sole fips has
    `prev_county is None` → skip (2281-2284). Neutral default stamped
    (`tick_bifurcation_score` 0.0, graph_bridge.py:181).
14. Step 7 validation (2460-2486): live; defaults sum to 1.0.
15. Step 8 summary (2488-2555): live; with k=0 the county "profit rate" is
    s/v; `phi_aggregate` 0; national dist = Wayne's (weight 1).
16. Write (graph_bridge.py:80-295): ~40 `tick_` attrs stamped on T001, the
    `tick_dynamics` graph attr written, then persisted into
    `persistent_context["_tick_dynamics"]` (simulation_engine.py:491-500).
17. Step 9 hex: dormant. `_reset_flow_accrual` writes the two zero counters
    on T001.

## round() and identity encoding

- All `round()` in the TickDynamics estate is Python built-in round —
  **round-half-to-even** — and every call site is in a path this scenario
  never reaches: cascade payload (system:1164-1167, behind the unwired
  transition engine), bifurcation payload (system:2336-2340, behind the
  no-prev skip), accumulation-loop flow counts
  (reserve_army/accumulation.py:115,121, behind the None-occs early return),
  precarity docstring example (tick/precarity.py:32). **No round() executes
  in this scenario's TickDynamics path within 52 ticks.** All sites are
  6-dp event-payload presentation, not state.
- County identity is **str FIPS everywhere**: `resolve_county_identity`
  returns `str(county_fips)` (graph_bridge.py:73-77);
  `CountyEconomicState.fips` is `min_length=5, max_length=5` str
  (types.py:320); county dicts keyed by str; determinism via `sorted(fips)`
  string sort (system:1907; graph_bridge.py:499). No int encoding exists in
  the Python tick path. ADR198 **R7** rules the port keys FIPS as **int with
  the leading-zero trap D-recorded**
  (`ai/decisions/ADR198_program29_substrate_widening_charter.yaml:74-79`).
  Wayne `"26163"` has no leading zero, so this scenario does **not** exercise
  the trap (every Michigan county is 26xxx; the trap needs a 0xxxx county).
  R7's "string really naming a node" clause is already honored in shape: node
  id `T001` vs county identity `"26163"` are separated by the `fips_to_node`
  map (graph_bridge.py:146-164).

## Reserved-line surfaces (named + cited; no rulings proposed)

1. **Bifurcation directional score.** `BifurcationRiskCalculator.compute`
   call site system:2286-2292; direction mapping `"revolutionary" if
   metric.score < 0 else "fascist"` at system:2329; event gate
   `abs(score) >= bifurcation_event_threshold` (0.5,
   economy_basic.py:129-134) at system:2294-2302; rounded payload at
   2336-2340. In this scenario the calculator is constructed but `.compute()`
   is never invoked (first-and-only boundary has no previous state,
   2265-2284) — within ANY 52-tick canonical horizon this surface needs a
   second boundary (tick ≥ 104) to fire at all.
2. **Five-share ClassDistribution / Feature-016.** `_simulate_transitions`
   gates on `services.transition_engine is None` → early return
   (system:2366-2367); the qa harness wires no transition engine
   (regression_test.py:215-258). The five shares stay the bootstrap defaults
   (system:822-830); only the sum-to-one validator (2460-2486) and the
   precarity deriver (lumpen share, 870-874) touch them live. The [2007,2030]
   year clamps at 810, 2374, 2432 apply but are inert here.
3. **`dispossession_cascade_milestones` (register row 21).** Defines field at
   `src/babylon/config/defines/economy_basic.py:137-140` (default
   `[0.05, 0.10, 0.15]`); sole read site system:1147 inside
   `_check_dispossession_cascade` (1115-1170); sole call site system:2444-2452,
   gated on the transition engine AND non-NORMAL crisis AND prev states —
   unreachable in this scenario (engine unwired). DORMANT.

## Open questions

- Exact stamped values at tick 52 (2011 tau, endogenous rate, the Wayne
  s=p+i+r+t split, assessor verdict) are computable only by running the
  scenario — forbidden in this swarm. Fixture presence verified; values
  UNVERIFIED.
- Crisis-detector phase staying NORMAL follows from r=0.0594 > 0.05 by read
  of the fixture; UNVERIFIED by execution (recovery-hysteresis path not
  traced).
- `gamma_basket_default` / `gamma_iii_default` numeric values not read
  (defines defaults path is live but its outputs unrecorded here).
- Whether `financial_crisis_assessor.assess` returns non-None for Wayne 2011
  — UNVERIFIED.
- The `interest_reserve_reference` 0.08 vs the unwired U3 default 0.05 means
  s_r is *structurally* 0 in this scenario (default < reference); whether any
  of the six scenarios produces s_r > 0 depends on their U3 wiring — outside
  my scope.
