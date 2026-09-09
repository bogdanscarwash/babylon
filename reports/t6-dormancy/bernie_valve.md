# T6 dormancy re-read — `bernie_valve`

Program 29 train T6 (issue #563, charter ADR198). Scenario: `bernie_valve`
(registry entry `tools/regression_scenarios.py:115-127`, factory
`src/babylon/engine/scenarios/electoral_goldens.py:481-543`). System under
re-read: `TickDynamicsSystem` @4.0 (`src/babylon/domain/economics/tick/system/__init__.py:112`).
Method: source read + the committed goldens (`tests/baselines/bernie_valve.json`,
`tests/baselines/dense/bernie_valve.csv`). No test/build runs (swarm rule).

## Scenario header

- **Substrate: Wayne single_county, county-BEARING — verified.** The factory calls
  `create_single_county_scenario()` (electoral_goldens.py:496-498), which stamps
  `county_fips="26163"` on the owner (`single_county.py:79`), the worker
  (`single_county.py:92`), and the sole Territory T001 (`single_county.py:113-120`;
  `WAYNE_COUNTY_FIPS="26163"` at `single_county.py:41`). Twins C005/C006 inherit
  `county_fips` via `model_copy(update={"id": ...})` (electoral_goldens.py:128,
  invoked at 502-503); `_voter` re-seeds preserve it (electoral_goldens.py:114).
  `county_fips` is a real declared field on `SocialClass`
  (`models/entities/social_class.py:426`) and survives the per-tick round-trip
  (`to_graph` stamps `territory.model_dump()`, `world_state.py:746`;
  `_reconstruct_territory` drops only `tick_`/`flow_`-prefixed attrs,
  `world_state.py:253-256`). TickDynamics resolves county identity from
  **territory nodes only** (`system/__init__.py:441-449`, 471-476;
  `graph_bridge.resolve_county_identity`, `graph_bridge.py:73-77`) — the class-level
  stamps are for other systems (spec-065).
- **STALE DOCSTRING (flag, not fixed — read-only charter):** electoral_goldens.py:14-17
  claims "weimar / debs / bernie_valve stand on the **two_node substrate**". The code
  builds bernie_valve on the Wayne single_county substrate, and the golden bytes prove
  it (`county_26163_*` columns populated). Docstring contradicts code.
- **Wiring:** harness `_run_scenario_ticks` (regression_test.py:996-1093) drives
  `simulation_engine.step()` with `build_single_county_overrides`
  (regression_test.py:1031-1035, 215-258) = `create_financial_services` 12 keys
  (factory.py:477-490) + fixture `melt_calculator` (regression_test.py:206) +
  fixture `tensor_registry` (Wayne 2010, regression_test.py:254-256;
  `tests/fixtures/single_county_wayne.json`, provenance: production hydrator chain,
  derived profit_rate 0.05944656600261918). Everything else in
  `ServiceContainer.create` defaults to None (engine/services.py:298-346).
  `event_bus` (services.py:412) and `economics_fallbacks` (services.py:247) are
  always constructed.
- **Run shape:** `DEFAULT_MAX_TICKS=52` (regression_test.py:81); baseline
  `ticks_survived=52`, `SURVIVED`, checkpoints [0,10,20,30,40,50,52]
  (tests/baselines/bernie_valve.json). No C001 entity ⇒ the periphery-worker
  death check (regression_test.py:1074-1078) never fires.
- bernie_valve's defines_overrides are all `politics.*` (regression_scenarios.py:119-126)
  — TickDynamics runs on default economy/crisis defines.

## Live fields (11 of the 33 ServicesProtocol fields TickDynamics references)

LIVE = wired AND exercised at the boundary (sentinel-branch invocations count as exercised).

| Field | Evidence |
|---|---|
| `melt_calculator` | Gate `system/__init__.py:198`; `get_melt(2010)` line 542; fixture-backed (regression_test.py:162-183). Pipeline aborts without it. |
| `tensor_registry` | `_get_profit_rate` line 1042 (crisis step, real 0.0594); `_get_organic_composition` line 1356 (accumulation loop); `_get_best_tensor_year`/`_get_county_surplus`/`_get_county_profit_rate` lines 2157-2239 (financial layer + `_economy_wide_profit_rate` line 1907-1916). |
| `distribution_calculator` | Gate line 1765; `compute_distribution` line 2023 → real s=p+i+r+t (golden tick-1: total_s 3,234,158,620.500001 = p 1,888,358,776.2854843 + i 970,247,586.1500003 + r 199,896,129.03225806 + t 175,656,129.03225806 — identity closes to the last digit). |
| `fictitious_capital_calculator` | Line 1964-1970; FRED fixture TCMDO covers 2010 (`tests/fixtures/vol3_fred_series.json`) → real stock, so `fin_ratio` computed (line 2136-2137), not the sentinel branch. |
| `credit_aggregate_source` | `_build_credit_state` lines 1845-1856 → real `CreditState` (TCMDO 2010). |
| `rent_calculator` | Line 2052-2058 — invoked, but the county rental adapter is all-None (factory.py:454-466) → `NoDataSentinel` (rent/calculator.py:109-115) → `record_vol3_rent_sentinel` + once-per-year log; `rent_extraction` never set. (The golden's `ground_rent` comes from the distribution calculator's FRED rental series, not this path.) |
| `housing_calculator` | Line 2061-2067 → real decomposition from hardcoded Census defaults (factory.py:393, 472-474). Value UNVERIFIED (not a dense column). |
| `financial_crisis_assessor` | Lines 2070-2082, 2146-2155 → real assessment fed by the real distribution + fin_ratio. Value UNVERIFIED (not a dense column). |
| `event_bus` | Publishes `CALIBRATION_QCEW_CARRY_FORWARD` with the `county_fips="*"` sentinel at the boundary (imperial_rent.py:86-88, 172-194) — the Spec-057-unwired marker. No other TickDynamics event fires (see dormant). |
| `economics_fallbacks` | `observe_wiring` lines 534-539; `record_gamma_basket_calculator_none` line 564; `record_gamma_iii_calculator_none` line 588; `record_vol3_rent_sentinel` line 2055. |
| `defines` | `economy.gamma_*_default` (lines 550, 567), `crisis.*` (lines 963-974, 2270-2277), `capital_vol3.*` (lines 1854, 2030, 2135, 2151; `interest_profit_share_base=0.30`, `interest_reserve_reference=0.08` — capital_vol3.py:156-157, 193-194), `reserve_army` via getattr line 1290. |

Live machinery beyond service fields: the gate itself (line 174), `_accrue_flows`
(lines 315-355), `_reset_flow_accrual` (357-372), the restamp path
(`read_tick_state_from_graph` → `stamp_county_attrs_to_territories`,
graph_bridge.py:298-422, 113-295), `PrecarityDeriver.derive` (u6=0.05+0.15=0.20,
pter=0.06, nilf=0.09 from the frozen default distribution; precarity.py:59-61),
`CoefficientSmoother` (init branch, lines 591-601), the crisis detector (4 quarterly
evals run; 0.0594 > `r_threshold=0.05` ⇒ stays NORMAL, no events, no compression —
lines 976-1017, economy_basic.py:67-72), `DerivedRateCalculator`
(derived_rates.py:40-92; K=0 ⇒ occ=0.0, profit/exploitation from tau — exact values
UNVERIFIED, not golden-pinned), `_validate_distributions` (passes: 0.01+0.09+0.40+
0.35+0.15=1.0, lines 2460-2486), `_compute_tick_summary` (phi_aggregate=0.0 since
phi_hour=0.0, lines 2488-2555), graph writes (~40 `tick_*` attrs onto T001,
graph_bridge.py:173-295, plus `tick_dynamics`/`national_financial` graph attrs).

**Downstream consumer note:** PolicySystem @17.47 reads `tick_taxes_on_surplus` /
`tick_phi_hour` off territory nodes (engine/systems/policy.py:258-260), and
bernie_valve seeds both a seated government and an 8-item agenda
(electoral_goldens.py:537-540); its declared live evidence is
`PolicySystem delivery_gap_crossed` (regression_scenarios.py:2546-2552). So the
boundary's stamps are live inputs to the scenario's headline valve mechanics on every
tick via the restamp — this is the strongest "LIVE" signal in the scenario.

## Dormant fields (22 unwired + 6 wired-but-unreferenced)

Unwired (None ⇒ early-return or documented default branch), each cited at its gate:

- `basket_calculator` (line 552 — defines default used), `gamma_calculator` (line 568 — same),
  `capital_calculator` (line 748 ⇒ K=0.0), `throughput_calculator` (line 756 ⇒ π=1.0, D=2.0),
  `unemployment_source` (line 767 ⇒ U-3=0.05), `housing_source` (line 655 ⇒ renter_share=0.0),
  `income_source` (line 678 ⇒ bracket_ratio=0.0), `cpi_source` (line 709 ⇒ deflator=1.0),
  `wage_source` (line 785 ⇒ median_wage=21.0 bootstrap), `employment_source`
  (line 794 ⇒ employment=100,000.0).
- `reserve_army_data_source` (line 1196-1197 — the whole Vol I wage-pressure layer
  skipped), `dispossession_data_source` (lines 1286, 2386 — no foreclosure/bankruptcy/
  eviction reads), `turnover_profile_source` (line 1409-1410 — the whole Vol II
  circulation layer skipped, which also leaves `inventory_data_source` /
  `depreciation_data_source` (lines 1455-1456) and the tensor-fed Vol II reproduction
  calculators (lines 1475-1567) unreached), `transition_engine` (line 2366-2367 — the
  whole Feature-016 transition step skipped), `hex_grid` (line 392-394 — step 9 no-op).
- The five Spec-057 fields — `periphery_labor_source`, `final_demand_source`,
  `industry_county_allocator`, `production_chain_calculator`, `bea_industries`
  (imperial_rent.py:158-169) — unwired ⇒ `_stub_zero_pass_through`: `phi_hour` stays
  0.0 with the sentinel event above (imperial_rent.py:86-88).

Wired by the harness but **never referenced by TickDynamics** (not on its boundary):
`interest_calculator` (deliberately unused post-U9 — comment lines 1763-1764),
`credit_cycle_detector` (`credit_cycle_phase` is a hardcoded `"expansion"` string,
graph_bridge.py:106), `counter_tendency_calculator`, `value_basis_converter`,
`z1_source`, `housing_data_source` (factory.py:477-490).

Consequences worth porting attention:

- The accumulation loop **executes but writes nothing** for Wayne: `occ_prior` at
  2009 is a `NoDataSentinel` (fixture holds only 2010) and `bankruptcy_rate` is None
  ⇒ both flows zero ⇒ `compute_dynamics` returns None (accumulation.py:125-126) ⇒ no
  `reserve_army_stock`/`reserve_ratio` stamps (system/__init__.py:1317-1323). Matches
  the coverage row "no reserve ratio seeded" (regression_scenarios.py:2586-2592).
- `financial_s_r`/`financial_tightness` are **computed zeros, not structural zeros**:
  `reserve_army_signal` runs on the live county dict (ρ̄=0.05 < ρ_ref=0.08 ⇒ 0.0,
  graph_bridge.py:534-542) and `loan_market_tightness(0.0)` ⇒ 0.0
  (endogenous_interest.py:98-116). The port must reproduce the computation; a county
  with U-3 > 0.08 would go nonzero.

## Annual pipeline — the tick-52 claim is WRONG for this scenario

**Claim to verify:** "drives the annual pipeline at tick 52." **Refuted by citation and
by golden bytes.** The boundary fires on the **first `step()` call** (`context.tick=0`),
and only there, inside the 52-tick run:

- `TickContext(tick=state.tick, ...)` uses the **pre-increment** tick
  (simulation_engine.py:566-569); the runner's loop iteration *k* passes a state with
  `state.tick = k−1` (regression_test.py:1054-1061), so context ticks are 0..51 and the
  only multiple of 52 (`system/__init__.py:174`; `WEEKS_PER_YEAR=52`,
  tunables.py:77-81) is **0**.
- The engine says so itself: "`0 % 52 == 0` fires the financial layer on the very
  FIRST `step()` call rather than 'never'" (simulation_engine.py:474-481); the dense
  harness repeats it: "the annual pipeline fires exactly once per 52-tick run, on the
  first `step()` call" (regression_test.py:497-506); the fixture provenance agrees:
  "bootstrapping year 2010 on the very first, tick-0 boundary"
  (tests/fixtures/single_county_wayne.json `_provenance`).
- Golden bytes: tick-0 row all zeros (pre-loop factory capture), tick-1 row fully
  populated (`financial_endogenous_rate=0.017833969800785755`,
  `financial_profit_rate_ceiling=0.05944656600261918`, `county_26163_total_s=
  3234158620.500001`, …), flat tick 1→52 (persisted/restamped annual facts).

What the single boundary (year = 2010, `_determine_year` lines 409-423 with default
`base_year=2010`) computes for Wayne: national params (real MELT tau, defines-default
gammas, smoother init); county state from documented defaults (K=0, π=1.0, U-3=0.05,
wage=21.0, employment=100k, the bootstrap five-share distribution); precarity derived;
Vol I + circulation layers skipped; crisis detector evaluates (stays NORMAL); the Vol
III financial layer fully fires — endogenous rate `i = r × base = 0.05944656600261918
× 0.30 = 0.017833969800785755` (endogenous_interest.py:43-96, tightness 0), real
`CreditState`/fictitious stock, the county s=p+i+r+t distribution (golden values
above), rent sentinel, housing + crisis assessment; transitions skipped; bifurcation
skipped (below); validation + summary; ~40 `tick_*` attrs stamped on T001; flow
counters reset.

Non-boundary ticks (context 1..51): restamp the same values (graph attr restored from
`persistent_context["_tick_dynamics"]`, simulation_engine.py:450-451) then
`_accrue_flows` adds one 1/52 slice (system/__init__.py:174-187). Note: the `flow_*`
counters are stripped by every round-trip (world_state.py:256) and the restamp does
not restore them, so in this harness each non-boundary tick recomputes a single slice
from zero (`flow_wage_accrued` = 21.0×2080×100000/52 = 84,000,000.0, `flow_phi_accrued`
= 0.0) — the within-year cumulative never actually accumulates on the
`simulation_engine.step()` path. No in-repo consumer of `flow_*_accrued` found beyond
a policy.py docstring mention (policy.py:33-35); they never reach `WorldState` or the
goldens.

## Rounding + identity encoding

- `round(x, 6)` appears in TickDynamics only in the two **dormant-here** event payloads
  (dispossession cascade lines 1164-1167; bifurcation lines 2336-2340) and in the
  accumulation calculator's unreached branches (accumulation.py:115, 121). Python
  `round()` is half-even on the decimal representation — but **bernie_valve exercises
  none of these sites**, so this scenario cannot pin half-even semantics for the port.
- The golden byte contract rounds nothing: dense cells are `repr(float)` — shortest
  round-trippable IEEE-754 decimal (trace_format.py:44-57). That is the formatting
  surface the port must match for these columns.
- County identity is the **string** `"26163"` end-to-end in the frozen engine:
  `Territory.county_fips` (single_county.py:116), `resolve_county_identity` returns
  `str(county_fips)` (graph_bridge.py:73-77), `dict[str, CountyEconomicState]` keys,
  CSV column names (regression_test.py:469). ADR198 **R7** rules the port carries
  FIPS-keyed state as **int fields with the leading-zero trap D-recorded**
  (ADR198 decision text, R7). For bernie_valve the only FIPS is 26163 — **no leading
  zero, so this scenario exercises the int round-trip cleanly but cannot pin the
  trap**. Live string-shaped operations the port must re-express as int logic:
  `sorted(county_states)` float-accumulation orderings (III.7) at system/__init__.py:1907
  and graph_bridge.py:499, and the `fips_to_node` map (graph_bridge.py:146-164).
  (`fips[:2]` state-prefix slicing at system/__init__.py:1610 is string-only but
  dormant here.)

## Reserved-line surfaces (named + cited; no rulings proposed)

1. **Bifurcation directional score.** `BifurcationRiskCalculator` is lazily
   **constructed** (system/__init__.py:2269-2275) but its `.compute()` is **never
   invoked** in this scenario: the only boundary has `prev_county_states={}` (empty
   bootstrap, lines 210-213 + 451-512), so every county takes the
   `prev_county is None → skip` branch (lines 2280-2284). `BIFURCATION_THRESHOLD`
   (FR-022, lines 2294-2302) never fires; the direction mapping
   (`"revolutionary" if score < 0 else "fascist"`, line 2329) is never reached;
   `tick_bifurcation_score` stamps 0.0 from the default metric (graph_bridge.py:181,
   406-408). Caution for the survey: bernie_valve's headline "bifurcation" (the
   topology-routed disillusion, Bernie→DSA vs Obama→Trump, electoral_goldens.py:481-494)
   is the Electoral/Allegiance surface, **not** this metric — same word, different
   machinery.
2. **Five-share ClassDistribution / Feature-016.** Touched but **frozen**: the
   bootstrap default shares (0.01/0.09/0.40/0.35/0.15, lines 822-830) are
   sum-validated (step 7), employment-weighted into the national distribution
   (lines 2521-2538), and stamped as `tick_class_distribution`
   (graph_bridge.py:183-189) — and never move, because `transition_engine` is None
   (early return lines 2366-2367). The `EconomicConditions` synthesis, wage-halt
   branch, per-county dispossession reads, and year clamps inside
   `_simulate_transitions` (lines 2369-2456) are all unreached.
3. **`dispossession_cascade_milestones` (register row 21).** The define exists
   (`[0.05, 0.10, 0.15]`, economy_basic.py:137-140); its sole reader is
   `_check_dispossession_cascade` (system/__init__.py:1147), reachable only from
   `_simulate_transitions` (lines 2444-2452) behind two closed gates
   (transition_engine None; crisis never leaves NORMAL since 0.0594 > 0.05).
   `DISPOSSESSION_CASCADE` never emitted; the milestone comparison and the
   `round(..., 6)` payload formatting (lines 1147-1170) are unexercised.

## Open questions

- Exact 2010 MELT tau and the derived-rate stamps (`tick_profit_rate`,
  `tick_exploitation_rate`) are not pinned by the dense golden columns — UNVERIFIED
  from artifacts; derivable from `tests/fixtures/` data if the port needs them.
- `housing_decomposition` / `financial_crisis` outputs are invoked live but their
  values are not byte-pinned by any committed bernie_valve artifact — UNVERIFIED.
- The issue's "tick 52" phrasing presumably describes the runner-backed scopes
  (detroit-tri-county / michigan-canada, persistent-graph drivers); for the four
  WAYNE_CALCULATOR_SCENARIOS through `simulation_engine.step()` the boundary is
  context.tick=0. Whether the two runner scopes really bound at literal tick 52 is
  for their readers to confirm.
- `_economy_wide_profit_rate` on a one-county domain degenerates to that county's
  tensor rate (r = s/(s/r)); whether the port intends a single-county scenario to
  define a "national" rate is a design observation, not a defect — flagged for the
  dossier.
- Twin starvation (C005/C006 "starve mid-run, attrs frozen thereafter",
  electoral_goldens.py:515-518) is outside TickDynamics' boundary — not verified here.
