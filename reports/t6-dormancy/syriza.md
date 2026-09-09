# T6 Dormancy Re-Derivation — `syriza`

Program 29 train T6 (issue #563, charter ADR198). Per-scenario re-read of
`TickDynamicsSystem` @4.0 (`src/babylon/domain/economics/tick/system/__init__.py:112`),
replacing the withdrawn blanket-dormancy premise with citation. Read-only; no test
suites run. Golden evidence quoted from the committed dense baseline
`tests/baselines/dense/syriza.csv` (53 rows: tick 0 + ticks 1–52).

## Scenario header

- **Registry row:** `tools/regression_scenarios.py:88-96` — "The captured governance
  road: capitulate with dual-power organs live; the PASOK slow bleed (U13 golden,
  ADR140)". Defines overrides: `politics.cycle_ticks {federal:8, state:8, local:4}`,
  `politics.betrayal_threshold 2.0e7` — neither is read by TickDynamics.
- **Factory:** `create_syriza_scenario`
  (`src/babylon/engine/scenarios/electoral_goldens.py:253-307`). Built on
  `create_single_county_scenario()` (called at electoral_goldens.py:267), then
  `apply_political_terrain(state, worker_id=_WAYNE_WORKER, owner_id=_WAYNE_OWNER)`
  (electoral_goldens.py:270) with `include_michigan` defaulting **True**
  (`electoral_fixture.py:113-118`) — syriza keeps the Michigan sub-sovereign and adds
  the contested `SOV_MI_STATE → T001` CLAIMS row (electoral_goldens.py:293-299), the
  "dual-power organ" of the charter narrative. Closed by `_warm()`'s round-trip
  fixed point (electoral_goldens.py:178-190, 307).
- **County-bearing — VERIFIED.** The substrate stamps Wayne County (FIPS `26163`)
  three times: `Territory(id="T001", county_fips="26163")`
  (`src/babylon/engine/scenarios/single_county.py:113-120`), and both `SocialClass`
  entities carry `county_fips="26163"` (single_county.py:79, 92). `Territory.county_fips`
  is a real declared field, `str | None`, 5-char constrained
  (`src/babylon/models/entities/territory.py:81-89`). `apply_political_terrain` adds
  no territory and no further `county_fips` (electoral_fixture.py:179-262).
- **county_states non-empty — VERIFIED.** At the annual firing, county identity
  resolves via `resolve_county_identity(T001) → "26163"`
  (`src/babylon/domain/economics/tick/graph_bridge.py:44-77`), so
  `county_fips = ["26163"]` (system/__init__.py:222-225, 425-449) and
  `county_states = {"26163": CountyEconomicState(...)}` — never the empty dict the
  county-free dormancy pattern (tools/regression_scenarios.py:406-451) is premised on.
- **Runner path:** qa:regression harness `_run_scenario_ticks`
  (`tools/regression_test.py:997-1093`), `DEFAULT_MAX_TICKS = 52`
  (regression_test.py:81), engine entry `simulation_engine.step`
  (regression_test.py:74; `src/babylon/engine/simulation_engine.py:503-611`),
  `calculator_overrides = build_single_county_overrides(defines)`
  (regression_test.py:1031-1035 → 215-258).

### What is wired (the whole service story)

`build_single_county_overrides` supplies exactly 14 keys: the 13 Vol III keys from
`_build_vol3_calculator_overrides` (regression_test.py:186-207 —
`distribution_calculator`, `interest_calculator`, `credit_cycle_detector`,
`credit_aggregate_source`, `fictitious_capital_calculator`, `rent_calculator`,
`housing_calculator`, `counter_tendency_calculator`, `value_basis_converter`,
`financial_crisis_assessor`, `z1_source`, `housing_data_source` via
`create_financial_services`, `src/babylon/domain/economics/factory.py:477-490`, plus
the fixture-backed `melt_calculator`, regression_test.py:206) **plus** a real
`tensor_registry` holding exactly one tensor: Wayne 26163, **year 2010**
(regression_test.py:239-256; `tests/fixtures/single_county_wayne.json`, derived
profit rate `0.05944656600261918` per its `_provenance`). Every other optional
`ServicesProtocol` field defaults to `None`
(`src/babylon/engine/services.py:212-295`); `economics_fallbacks` is a fresh tally
per container (services.py:247).

## Annual pipeline — when it fires (the tick-52 claim: CORRECTED)

- Gate: `if tick % WEEKS_PER_YEAR != 0: ... return` (system/__init__.py:174-187);
  `WEEKS_PER_YEAR = 52` (defines `timescale.weeks_per_year`,
  `src/babylon/config/defines/tunables.py:77-81`, surfaced at
  `src/babylon/formulas/constants.py:37`).
- The qa harness builds `TickContext(tick=state.tick, ...)` — the **pre-increment**
  tick (simulation_engine.py:566-569) — and the factory state starts at `tick=0`
  (single_county.py:132; `_warm` preserves 0, electoral_goldens.py:190). The runner
  loop `for tick in range(1, 53)` (regression_test.py:1054) therefore presents
  **context ticks 0..51**. The boundary condition `tick % 52 == 0` is met at
  **context tick 0 — the FIRST `step()` call** — and never again inside a 52-tick
  run (context tick 52 is never reached).
- This is documented in-repo, not just derived here: `_save_graph_context`'s
  docstring (simulation_engine.py:476-483) records that "`0 % 52 == 0` fires the
  financial layer on the very FIRST `step()` call rather than 'never'".
- **Correction to the issue premise:** "the annual pipeline at tick 52" holds only
  for the headless-runner scopes (`tick_range = range(1, config.ticks)`,
  `src/babylon/engine/headless_runner/runner.py:1694`) running ≥53 ticks. For
  syriza (qa harness) the pipeline fires at **context tick 0**, exactly once per run.
- **Golden proof:** in `tests/baselines/dense/syriza.csv`, `financial_endogenous_rate`
  is `0.0` only on the tick-0 (pre-engine) row and `0.017833969800785755` on every
  row 1–52 (2 distinct values); same shape for `financial_profit_rate_ceiling`
  (`0.05944656600261918`) and all four `county_26163_*` distribution columns. One
  computation, verbatim re-stamp thereafter (spec-109 A7 re-stamp,
  system/__init__.py:175-186 + graph_bridge.py:113-170).
- Year at the firing: `base_year` graph attr is never set by `WorldState.to_graph()`
  (grep: no `base_year` in `src/babylon/models/world_state.py`), so
  `_determine_year` returns `2010 + 0//52 = 2010` (system/__init__.py:409-423) —
  exactly the fixture's tensor year. The single firing hits real data.

### What it computes for syriza (year 2010, county 26163), step by step

- **Step 2 — national params (system/__init__.py:514-628):** τ = MELT(2010) from the
  committed fixture = 14,754,993,740,000 / (126,464,161 × 2080) ≈ 56.09 $/hr
  (`src/babylon/domain/economics/melt/melt_calculator.py:172-197`; fixture
  `tests/fixtures/vol3_melt_national.json`). `gamma_basket` / `gamma_III` fall back to
  defines defaults with WARNING + tally (lines 550-588) — the fallback path is
  exercised, the calculators are unwired. Smoothing runs in raw-passthrough mode
  (prev None → lines 894-902).
- **Step 3a — county state (715-852):** every per-county source unwired → documented
  bootstrap defaults: `capital_stock=0.0`, `throughput_position=1.0`,
  `supply_chain_depth=2.0`, `unemployment_rate=0.05`, `renter_share=0.0`,
  `median_wage=21.0`, `employment=100_000.0`, `phi_hour=0.0`, `bracket_ratio=0.0`,
  `real_wage_deflator=1.0`, class shares 0.01/0.09/0.40/0.35/0.15 (lines 822-830).
- **Step 3a+ — precarity (854-878):** `PrecarityDeriver.derive(0.05, 0.15)` →
  U-6 = 0.20, PTER = 0.06, NILF = 0.09
  (`src/babylon/domain/economics/tick/precarity.py:44-62`). Computed live, but note:
  U-6/PTER/NILF are **not** among the graph-stamped attrs (stamp list
  graph_bridge.py:173-295) — they ride the persisted `county_states` object only.
- **Step 3.5 — Vol I wage pressure (1175-1241):** `reserve_army_data_source is None`
  → early return (1196-1197). Skipped.
- **Step 3.6 — accumulation loop (1243-1334):** `tensor_registry` wired, so the loop
  enters; `occ_current` real (2010 hit), `occ_prior` None (2009 <
  `TensorRegistry.MIN_YEAR` 2010 → `NoDataSentinel`, tensor_registry.py:200-206) →
  mechanization branch never taken; `dispossession_data_source` None → bankruptcy
  None → firm-failures branch never taken → `compute_dynamics` returns None
  (`src/babylon/domain/economics/reserve_army/accumulation.py:103-126`) → `updates`
  empty → **zero graph writes** (1333-1334). Enters, does nothing.
- **Step 4 — imperial rent (912-939 →
  `src/babylon/domain/economics/tick/system/imperial_rent.py:45-88`):** all 4 Spec 057
  fields + `bea_industries` are None → `_spec_057_pipeline_wired` False
  (imperial_rent.py:158-169) → publishes ONE `CALIBRATION_QCEW_CARRY_FORWARD`
  sentinel event (172-194) and passes `county_states` through unchanged
  (221-230). `phi_hour` stays the bootstrap 0.0 — the "MEASURED 0.0" of the
  electoral_goldens docstring (electoral_goldens.py:10-13).
- **Step 4.5 — circulation (1367-1441):** `turnover_profile_source is None` → early
  return (1409-1410). `circulation_state` stays `CirculationCrisisState.default()`.
- **Step 5 — crisis triggers (941-1017):** detector built from `defines.crisis`
  (`r_threshold=0.05`, `src/babylon/config/defines/economy_basic.py:67-90`); Wayne's
  realized tensor profit rate 0.05945 > 0.05 → the 4 quarterly evaluations all stay
  NORMAL; no events, no wage compression (982-997). Machinery runs, data keeps it
  quiet.
- **Step 5.5 — Vol III financial layer (1737-2086): FIRES IN FULL.**
  `_economy_wide_profit_rate` = 0.05944656600261918 (single-county aggregate,
  1858-1919); `reserve_army_signal` = 0.0 (ρ̄=0.05 < ρ_ref=0.08 → clamped,
  graph_bridge.py:511-542); `loan_market_tightness` = 0.0
  (`src/babylon/domain/economics/credit/endogenous_interest.py:98-116`);
  endogenous rate ≈ **0.017833969800785755** (golden-verified);
  `NationalFinancialParameters` published to the graph (graph_bridge.py:433-450) and
  persisted save-side into `persistent_context["_national_financial"]`
  (simulation_engine.py:489-490) — consumed same-tick by MarketScissors/Contradiction
  (per simulation_engine.py:472-474). Per-county distribution over
  `total_surplus = tensor.total_s = 3,234,158,620.500001`:
  interest 970,247,586.15 / ground rent 199,896,129.03 / taxes 175,656,129.03 /
  profit of enterprise 1,888,358,776.29 — all four golden-verified, and
  s = p+i+r+t holds byte-exactly in the CSV. `debt_accumulation` skipped
  (first-pass None guard, 2040-2049). `rent_calculator` → `NoDataSentinel` (stub
  adapter returns None, factory.py:454-466 → rent/calculator.py:109-115) →
  `record_vol3_rent_sentinel` (2052-2056). `housing_calculator` → `NoDataSentinel`
  (CensusHousingLoader's hardcoded table covers ("26163", 2015/2020/2022) only —
  `src/babylon/domain/economics/data_adapters.py:121-144,164-166` — a (26163, 2010)
  miss) → `record_vol3_housing_sentinel` (2061-2065). `financial_crisis_assessor`
  fires (2070-2082) on quiet inputs (spread 0.0).
- **Step 6 — class transitions (2346-2458):** `transition_engine is None` → early
  return (2366-2367). `EconomicConditions`, `should_halt_accumulation`, real
  dispossession rates, and `_check_dispossession_cascade` all unreached.
- **Step 5b — bifurcation risk (2241-2306):** `prev_county_states` is `{}` (bootstrap
  found no `tick_capital_stock` on a fresh T001, 451-512) — **not** None, so the
  calculator IS constructed and four defines ARE read
  (`bifurcation_solidarity_weight`, `bifurcation_burden_weight`,
  `class_burden_epsilon`, 2269-2275; `bifurcation_event_threshold`, 2277) — but the
  per-county `prev_county_states.get("26163")` is None → `continue` (2281-2284).
  `BifurcationRiskCalculator.compute()` is **never called**; no
  `BIFURCATION_THRESHOLD` event; `bifurcation_risk` stays the neutral default and
  `tick_bifurcation_score=0.0` is stamped (graph_bridge.py:181).
- **Steps 7-8 — validation + summary (2460-2555):** sum-to-one check executes
  (0.01+0.09+0.40+0.35+0.15=1.0). `DerivedRateCalculator` computes from the bootstrap
  state: v = 12.0 × 2.08e8 = 2.496e9, s ≈ 56.09 × 2.08e8 − 2.496e9, K=0 →
  `tick_profit_rate = s/v ≈ 3.7` (the rate-of-exploitation lookalike the
  `_economy_wide_profit_rate` docstring warns about, 1887-1891), `tick_occ=0.0`,
  `phi_aggregate=0.0` (`src/babylon/domain/economics/tick/derived_rates.py:40-109`).
  Note the graph then carries TWO different profit rates: the stamped derived
  `tick_profit_rate ≈ 3.7` and the tensor-realized 0.05945 inside the financial
  layer / `tick_dynamics` payload.
- **Step 9 — hex substrate (374-407):** `hex_grid` None → no-op (392-394).
- **Flow accrual (non-boundary ticks 1-51):** re-stamp + `_accrue_flows`
  (315-355) run 51×, but both counters are **structurally zero**: `phi_hour=0.0`
  zeroes `flow_phi_accrued`, and `tick_employment` has **no write site anywhere in
  `src/babylon`** (grep-verified: only reads at system/__init__.py:343, 506 and
  graph_bridge.py:392) — so `data.get("tick_employment", 0.0)` yields 0.0 and
  `flow_wage_accrued ≡ 0.0` despite a real stamped `tick_median_wage=21.0`. Asymmetric
  defaults over the never-written attr: 0.0 at :343 vs 100_000.0 at :506 and
  graph_bridge.py:392. `_reset_flow_accrual` (357-372) zeroes the same counters at
  the boundary.

## Live fields (exercised by syriza — producer or consumer fires)

ServicesProtocol boundary fields (protocol list: `src/babylon/kernel/services.py:34-88`):

| # | Field | Evidence |
|---|-------|----------|
| 1 | `melt_calculator` | The master gate (system/__init__.py:198-200); fixture τ(2010)≈56.09 computed (melt_calculator.py:172-197) |
| 2 | `tensor_registry` | Wayne 2010 tensor read for profit rate (1042), surplus (2233), OCC (1356), best-year fallback (2181); carry-forward logic present but unneeded (exact year hit) |
| 3 | `distribution_calculator` | `compute_distribution` fires; s=p+i+r+t golden-verified (system/__init__.py:2017-2039) |
| 4 | `credit_aggregate_source` | `_build_credit_state` reads `get_total_credit(2010)` (1845-1856) |
| 5 | `fictitious_capital_calculator` | `compute_fictitious_capital(2010)` runs (1964-1970); FRED fixture covers 2010 (regression_test.py:186-207) |
| 6 | `financial_crisis_assessor` | `assess()` fires (2070-2082, 2146-2155) — inputs quiet (spread 0.0) |
| 7 | `event_bus` | Carries the one `CALIBRATION_QCEW_CARRY_FORWARD` sentinel (imperial_rent.py:188-194); crisis/bifurcation/cascade events never emitted |
| 8 | `defines` | `crisis.*` (963-974, 2270-2277), `capital_vol3.*` (2030, 2135, 2151), `capital_vol2.*` and `timescale.days_per_year` read at the layers that fire |
| 9 | `economics_fallbacks` | `observe_wiring` + `record_gamma_basket_calculator_none` + `record_gamma_iii_calculator_none` + rent/housing sentinel tallies (534-588, 2036, 2055, 2064) |

Graph/state surfaces live every tick: `graph.graph["tick_dynamics"]` write/read +
`persistent_context` bridge (graph_bridge.py:97-108, simulation_engine.py:446-451,
487-494); the 36-attr `tick_*` stamp onto T001 (graph_bridge.py:173-295) written at
the boundary and re-stamped verbatim ticks 1-51; `national_financial` graph
publication (graph_bridge.py:450) with same-tick consumers.

## Dormant fields (declared but never exercised by syriza)

Unwired (`None`) — gated paths skip silently or degrade with a logged fallback:

| Field | Consequence | Cite |
|---|---|---|
| `basket_calculator` | defines-default γ_basket + WARNING | system/__init__.py:550-564 |
| `gamma_calculator` | defines-default γ_III + WARNING | 567-588 |
| `capital_calculator` | K=0.0 bootstrap | 747-751 |
| `throughput_calculator` | π=1.0, D=2.0 defaults | 753-760 |
| `transition_engine` | **Step 6 skipped wholesale** — Feature 016 inert | 2366-2367 |
| `reserve_army_data_source` | Step 3.5 skipped | 1196-1197 |
| `dispossession_data_source` | no bankruptcy/foreclosure/eviction anywhere | 1286-1288, 1325-1331, 2386-2396 |
| `employment_source` | 100k default | 793-797 |
| `unemployment_source` | 0.05 default | 766-770 |
| `housing_source` | renter_share 0.0 | 630-659 |
| `wage_source` | median_wage 21.0 | 784-788 |
| `income_source` | bracket_ratio 0.0 | 661-682 |
| `cpi_source` | deflator 1.0 | 684-713 |
| `turnover_profile_source` | circulation layer skipped | 1409-1410 |
| `inventory_data_source` / `depreciation_data_source` | national circulation params never read | 1455-1473 (unreachable) |
| `hex_grid` | Step 9 no-op | 392-394 |
| `periphery_labor_source` / `final_demand_source` / `industry_county_allocator` / `production_chain_calculator` / `bea_industries` | Spec 057 Leontief pipeline unwired → sentinel + pass-through | imperial_rent.py:86-88, 158-169 |
| `productivity_data_source`, `community_hypergraph`, `field_registry`, `persistence`, `tracer`, `boundary_register`, `auditor` | never read by TickDynamics | kernel/services.py:43-71 (grep: no reads in `domain/economics/tick/`) |

Wired-but-unread by TickDynamics (in syriza's override dict yet never touched at
this boundary — grep-verified no references in `src/babylon/domain/economics/tick/`):
`interest_calculator` (calibration-only since U9, system/__init__.py:1763-1765),
`counter_tendency_calculator`, `value_basis_converter`, `credit_cycle_detector`
(the `credit_cycle_phase:"expansion"` attr is a hardcoded stamp,
graph_bridge.py:106), `z1_source`, `housing_data_source` (consumed one level down
inside the calculators, not at this boundary).

Wired-and-called but data-dormant: `rent_calculator` (all-None stub adapter →
`NoDataSentinel`, rent/calculator.py:109-115), `housing_calculator` (no (26163,2010)
row → `NoDataSentinel`, data_adapters.py:121-144).

Structurally-zero channels (machinery runs, value can never move in this harness):
`flow_phi_accrued` / `flow_wage_accrued` (no `tick_employment` write site repo-wide;
phi measured 0.0) — and the register already marks `financial_s_r` /
`financial_tightness` at rest on this terrain (tools/regression_scenarios.py:2380-2387),
golden-verified flat 0.0 across all 53 rows.

## Rounding and identity encoding

- **`round()`:** Python's built-in is round-half-**even** (banker's). Five sites in
  the TickDynamics estate, **all in branches syriza never reaches**: the
  DISPOSSESSION_CASCADE payload (system/__init__.py:1164-1167; Step 6 unwired), the
  BIFURCATION_THRESHOLD payload (2336-2340; `compute()` never called), and the two
  accumulator roundings (`reserve_army/accumulation.py:115,121`; both inputs None →
  early return at :125-126). Remaining economics `round()` sites
  (circulation/turnover.py:236-247, factory.py:762-763, border_commute_synthesis.py)
  sit in modules unwired for this scenario. **Zero `round()` executions in syriza's
  TickDynamics path.** The half-even semantics are port-relevant surface, but this
  scenario supplies no behavioral witness for them.
- **County identity encoding (ADR198 R7):** the frozen engine carries FIPS as a
  **5-char string end-to-end** — `Territory.county_fips: str | None` (5-char
  constrained, territory.py:81-89) → graph attr → `resolve_county_identity` `str()`
  passthrough (graph_bridge.py:73-77) → `CountyEconomicState.fips` /
  `ClassDistribution.fips` (5-char constrained, tick/types.py:320;
  dynamics/types.py). R7 rules the **port** carries FIPS as int fields "with the
  leading-zero trap D-recorded" (ADR198, R7 section). Syriza exercises **no**
  leading-zero hazard (26163); the only `zfill` in the economics tree is the
  aggregate path (`tensor_registry.py:388`), unreached here. No int-encoded FIPS
  surface is exercised by this scenario.

## Reserved-line surfaces (survey register row 21 — name + cite only; rulings are the Director's)

Register row 21 (`reports/port-estate-survey-2026-08-12.md:325`): the
bifurcation directional score + its three defines; the five-share
`ClassDistribution` + Feature-016 transition engine; `crisis.dispossession_cascade_milestones`.

1. **Bifurcation directional score.** `BifurcationRiskMetric`
   (tick/types.py:155-196): score ∈ [−1,+1], −1 revolutionary / +1 fascist; the
   direction string is minted at system/__init__.py:2329. For syriza: calculator
   constructed and its defines read (`bifurcation_solidarity_weight`,
   `bifurcation_burden_weight`, `class_burden_epsilon` at 2270-2275;
   `bifurcation_event_threshold` at 2277) but **`compute()` never runs** — the sole
   boundary has no prior county state (2281-2284). The stamped
   `tick_bifurcation_score` is the neutral default 0.0. The metric's inputs would be
   topological (SOLIDARITY density read off the graph); syriza's worker deliberately
   carries no SOLIDARITY bridges (factory docstring, electoral_goldens.py:263-265) —
   moot here, since the calculator never fires. DORMANT.
2. **Five-share `ClassDistribution` / Feature-016.** The **data structure is live**:
   bootstrap construction (822-830), year clamp to [2007, 2030] (810-824),
   lumpen-share consumed by the precarity deriver (870), sum-to-one validation
   (2472-2486), graph stamp + read-back (graph_bridge.py:183-189, 369-378). The
   **transition engine is dormant**: `services.transition_engine is None` →
   `simulate_transitions` never called (2366-2367); the five shares are static for
   the whole 52-tick run.
3. **`crisis.dispossession_cascade_milestones`** (register row 21's third surface).
   Defines row: `[0.05, 0.10, 0.15]` (`economy_basic.py:137-140`; yaml mirror
   `src/babylon/data/defines.yaml:46`). Sole read site: system/__init__.py:1147
   inside `_check_dispossession_cascade`, reachable only from `_simulate_transitions`
   (2445-2452) under `transition_engine` wired AND crisis-phase ≠ NORMAL AND prior
   county state. All three gates fail for syriza — **the defines row is never even
   read** in this scenario. DORMANT.

## Open questions

1. **`tick_employment` has no writer.** `_accrue_flows` reads it defaulting 0.0
   (system/__init__.py:343); the bootstrap and read-back paths default it to
   100_000.0 (:506, graph_bridge.py:392). Grep finds no `tick_employment=` write
   anywhere in `src/babylon`. Intentional (a flow lane awaiting a producer) or a
   dead counter? Not syriza-specific — affects every scenario through
   `simulation_engine.step()`.
2. **Two profit rates coexist on the graph** after the boundary: stamped derived
   `tick_profit_rate ≈ 3.7` (K=0 bootstrap ⇒ s/v) vs the tensor-realized 0.05945 the
   financial layer actually used. Consumers reading the stamped attr get a
   materially different quantity. Port note, not a ruling request.
3. **`_WAYNE_WORKER`/`_WAYNE_OWNER` naming inversion.** electoral_goldens.py:47-48
   set `_WAYNE_WORKER = "C003"`, `_WAYNE_OWNER = "C004"`, but
   `CORE_BOURGEOISIE_ID = "C003"` and `LABOR_ARISTOCRACY_ID = "C004"`
   (`src/babylon/models/entity_registry.py:35,38`), and the register's own
   single_county note calls C003 the owner (tools/regression_scenarios.py:2245-2248).
   So syriza's "worker" re-seed (socdem 0.7, agitation 0.5 — electoral_goldens.py:273-278)
   lands on the core-bourgeoisie id, and vice versa. Invisible to TickDynamics (it
   keys on `county_fips`, not roles); whether the inversion is intentional ADR140
   calibration is UNVERIFIED from source.
4. **fictitious-capital value for 2010 is not golden-visible** (no dense column); its
   liveness is inferred from fixture coverage (2010-2024) and the absence of a
   sentinel row in the harness manifest — UNVERIFIED against a manifest artifact.
5. The issue's "tick 52" framing: correct only for headless-runner scopes ≥53 ticks
   (runner.py:1694); qa-harness scenarios fire at context tick 0. If the T6 roll-up
   normalizes on "tick 52", syriza's row should read "context tick 0 (first step()
   call); single firing per 52-tick run".
