# T6 dormancy re-derivation — `michigan_canada_e2e`

**Issue:** #563 (Program 29 train T6) · **Charter:** ADR198 (`ai/decisions/ADR198_program29_substrate_widening_charter.yaml`)
**System under re-read:** `TickDynamicsSystem` @4.0 — `src/babylon/domain/economics/tick/system/__init__.py:112`
**Scenario:** `--scope michigan-canada`, 520 ticks, seed 2010, start_year 2010 → baseline `tests/baselines/michigan-e2e.json`
(a Git-LFS pointer — 134 bytes — in this workspace; baseline *contents* not inspectable here; claims that
would need them are marked UNVERIFIED).

## Scenario header — how the run is built

- Scope resolution: `src/babylon/engine/headless_runner/scopes.py:148-149` → `MICHIGAN_FIPS` (83 county FIPS,
  hard-coded `scopes.py:34-120`, all state-26) + external node `{"canada"}`. CLI default is this scope
  (`argparse_cli.py:65-67`); the canonical command is `.mise.toml:763-764` (`--ticks 520 --write-baseline`).
  `SimulationRunConfig` defaults: `start_year=2010`, `random_seed=2010` (`headless_runner/models.py:107-113`).
- **county_fips is carried.** The bridge seeds one `Territory` per county with a real 5-char string FIPS:
  `bridge.py:790-816` (`Territory(id=f"T{i:03d}", county_fips=county_fips, ...)`); the field is
  `str | None`, `min_length=5, max_length=5` (`src/babylon/models/entities/territory.py:81-84`). Node ids are
  graph-local `T001..T083` labels — identity lives ONLY in the `county_fips` attribute
  (`graph_bridge.py:44-77`, `resolve_county_identity`). Per-county `social_class` entities also carry it
  (`bridge.py:769-786`; field at `models/entities/social_class.py:426-434`).
- Edges seeded per county: EXPLOITATION, TENANCY, WAGES (`bridge.py:858-894`) plus TIGER-derived ADJACENCY
  (`bridge.py:895-907`). **SOLIDARITY is deliberately NOT seeded** (`bridge.py:842-846`, Constitution III.5/Q4).
- **county_states is non-empty.** First boundary: `_bootstrap_county_states` returns `{}` (no `tick_capital_stock`
  attr yet, `system/__init__.py:480`), so `county_fips = self._get_territory_fips(graph)` → all 83
  (`system/__init__.py:222-225`, `:425-449`). Later boundaries key off `prev_county_states` (`:222-223`).
  The withdrawn blanket-dormancy premise (county-free pattern documented at `tools/regression_scenarios.py:406-454`)
  does **not** apply here.

## Live fields (ServicesProtocol surface TickDynamics touches; wired AND exercised)

25 of the 33 fields TickDynamics can read (28 in the main file + 5 in `imperial_rent.py`):

| Field | Wiring (runner.py) | Exercised at |
|---|---|---|
| `melt_calculator` | `:1075` | gate `:198`; `get_melt(year)` `:542`; data window [2010,2024] (`melt/melt_calculator.py:189-194`) covers all 9 boundary years |
| `gamma_calculator` | `:1057` | `:568-571` |
| `defines` | `:1326` | crisis/vol2/vol3/reserve_army coefficients throughout (`:963,:974,:1147,:1290,:1556,:2270,:2277,:2369` …) |
| `economics_fallbacks` | auto (`engine/services.py:247`) | `:534-539`, sentinel tallies `:1967,:2036,:2055,:2064` |
| `event_bus` | `:1329` | crisis events `:1087,:1103`; cascade `:1158`; bifurcation `:2330`; imperial-rent sentinels `imperial_rent.py:188,:212` |
| `unemployment_source` | `:1078` | real BLS LAUS U-3 per county-year, re-queried every boundary `:767-770` |
| `wage_source` | `:1081` | **bootstrap-only**: QCEW p50 seed when `prev is None` `:785-788` (83 calls at tick 52, then endogenous) |
| `cpi_source` | `:1084` | real CPIAUCSL deflator every boundary `:709-713` |
| `tensor_registry` | `:1121` (hydrated 2010–2024, `:96,:1118-1120`) | profit rates `:1042`, OCC `:1303-1304`, department rows `:1503-1511`, surplus `:2214-2239` |
| `reserve_army_data_source` | factory `vol1` (`factory.py:813-818`) | `get_unemployment_decomposition` per county `:1235` → sigmoid wage pressure `:1239-1241` |
| `dispossession_data_source` | same | bankruptcy/foreclosure/eviction `:1307,:1326-1331,:2386-2396` |
| `transition_engine` | same | `simulate_transitions` per county per boundary `:2424-2454` |
| `distribution_calculator` | factory `financial` (`factory.py:477-490`) | layer gate `:1765`; s = p+i+r+t per county `:2023-2034` |
| `fictitious_capital_calculator` | same | `:1964-1970` |
| `rent_calculator` | same | `:2052-2058` |
| `housing_calculator` | same | `:2061-2067` |
| `financial_crisis_assessor` | same | `:2070`, `:2146-2154` |
| `credit_aggregate_source` | same | `:1845-1856` (via `getattr`) |
| `inventory_data_source` | factory `circulation` (`factory.py:617-621`) | national reads only `:1464-1467` — county products discarded (see dormant) |
| `depreciation_data_source` | same | national reads only `:1469-1471` — same caveat |
| `periphery_labor_source` | factory `leontief` (`factory.py:154+`, fields `:161-164`) | `imperial_rent.py:96` |
| `final_demand_source` | same | `imperial_rent.py:102` |
| `production_chain_calculator` | same | `imperial_rent.py:107,:126-128` |
| `industry_county_allocator` | same | `imperial_rent.py:138` → per-county `phi_hour` |
| `bea_industries` | same | `imperial_rent.py:90,:115-121` |

Consequence: the endogenous interest rate (`_economy_wide_profit_rate`, `:1858-1919` →
`endogenous_interest_rate` `:1958`) is computed from 83 real tensor (s, r) pairs and is **structurally
nonzero** here — the exact inverse of the county-free dormancy chain (`tools/regression_scenarios.py:406-454`).
Whether any county's crisis phase actually leaves NORMAL (events fire) is UNVERIFIED (runtime-data-dependent;
baseline LFS-unhydrated).

## Dormant fields

**Unwired in this scenario (7)** — none appear anywhere in `runner.py`'s overrides:

- `basket_calculator` → `gamma_basket` falls to the defines default with a WARNING + tally
  (`:550-564`). LIVE-as-fallback, never computed.
- `capital_calculator` → `capital_stock` ≡ 0.0 for all 83 counties all run (`:747-751`; bootstrap 0.0,
  prev-carried thereafter). **Knock-on dormancy:** the entire Feature-023 per-county circulation layer
  early-returns on `capital_stock <= 0` (`:1603-1605`) — circuit advance, inventory, depreciation fund,
  reproduction schema (`:1631-1732`) never execute for any county, and `tick_liquidity_ratio` etc. stamp
  default `CirculationCrisisState` values (`graph_bridge.py:200-234`).
- `throughput_calculator` → `throughput_position=1.0`, `supply_chain_depth=2.0` constants (`:754-760`).
- `housing_source` → `renter_share` ≡ 0.0 (`:654-659`).
- `income_source` → `bracket_ratio` ≡ 0.0 (`:677-682`).
- `employment_source` → `employment` ≡ 100 000.0 flat for every county every year (`:793-797`).
- `hex_grid` → Step 9 `_write_hex_substrate` is a no-op (`:392-394`, called `:307`).

**Wired-but-dormant (1):** `turnover_profile_source` — wired (`factory.py:618`) but its
`get_turnover_profile` call site (`:1609-1610`) sits *after* the `capital_stock <= 0` early return
(`:1603-1605`), so it is never reached in this scenario.

**Sub-channel dormancies found inside live paths:**

- `tick_employment` is **never stamped** to territory nodes (write list `graph_bridge.py:173-295`; reads at
  `graph_bridge.py:392`, `system/__init__.py:343,:506`). Hence `_accrue_flows` computes
  `annual_wage = tick_median_wage · 2080 · 0.0` → **`flow_wage_accrued` ≡ 0.0 all run** (`:342-354`), while
  `flow_phi_accrued` accrues genuinely (`phi_hour·2080/52` per tick). Masked elsewhere only because the unwired
  `employment_source` makes 100 000.0 the steady-state value anyway.
- `flow_phi_accrued` / `flow_wage_accrued` have **no in-run reader** in this scenario: the sole consumer is the
  legacy web bridge's `county_flow` seam (`src/babylon/sentinels/seam/registry.py:2897-2919`); PolicySystem
  recomputes the Φ slice instead of reading the counters (`engine/systems/policy.py:31-35`, ADR135).
  Write-live, read-dormant.
- Precarity triple (U-6/PTER/NILF) is derived every boundary (`:854-878`, `precarity.py:44-62`) but has **no
  consumer** — never stamped to the graph (absent from `graph_bridge.py:173-295`), and no reader of `.u6_rate`
  exists outside the module (rg-confirmed). Computed-but-unread.
- `ThresholdCrisisDetector` (`_legacy_crisis_detector`, constructed `:129`) is never called anywhere — dead
  construction.
- Bifurcation's lifecycle-legitimation read (`bifurcation.py:99-104`) compares `node.id == fips` — `T001`-style
  ids never equal `"26xxx"` FIPS, so the Feature-030 blend path is unreachable in this scenario; legitimation
  always falls to the agitation-inverse branch (`bifurcation.py:207-235`).

## Annual pipeline — when it fires, what it computes

**The tick-52 claim is CONFIRMED, with precision added.** Gate: `if tick % WEEKS_PER_YEAR != 0`
(`system/__init__.py:174`); `WEEKS_PER_YEAR = 52` (`config/defines/tunables.py:77-81` →
`formulas/constants.py:37`; `defines.yaml:374`). The runner's loop is `range(1, config.ticks)`
(`runner.py:1694`), so the pipeline fires at ticks **52, 104, …, 468 — 9 times** in the 520-tick canonical run
(tick 520 is excluded by the half-open range; tick 0 belongs to hydration). Years: first fire
`_determine_year(52) = 2010 + 52//52 = 2011` (`:409-423`; `base_year` graph attr default 2010 `:420-422` —
the attr is only provably set in the qa-harness path, `engine/simulation_engine.py:448-449`; identical value
here since `start_year=2010`), then `existing_state.year + 1` (`:206-208`) → **years 2011–2019**, all inside the
tensor window (2010–2024) and the MELT window (2010–2024). Non-boundary ticks re-stamp boundary values and
accrue flows (`:183-187`); boundary ticks close out accrual first, then recompute, then reset counters
(`:195,:313`).

Per boundary, for 83 counties: national params (MELT real; gamma_basket defines-default; gamma_III computed;
EMA-smoothed `:592-601`) → county states (real U-3, real CPI deflator, QCEW-seeded-then-endogenous median
wage; flat 100k employment; K≡0; renter_share/bracket_ratio ≡ 0) → precarity (unread) → Vol I wage-pressure
sigmoid (`reserve_army/calculator.py:32-65`) → accumulation loop writing `reserve_army_stock`/`reserve_ratio`/
`foreclosure_rate`/`eviction_rate` onto territory nodes for same-tick consumers (`:1293-1334`) → Leontief
`phi_hour` per county (real, modulo per-source sentinels that stub the whole year to pass-through,
`imperial_rent.py:96-110,:143-145`) → circulation (national-only; county layer gated off) → crisis detector
(4 quarterly evals/county/boundary, `:971-984`) → Vol III financial layer (endogenous rate from real
surplus-weighted profit; s=p+i+r+t; rent; housing; crisis assessment) → Feature-016 class transitions →
bifurcation risk (from 2nd boundary; neutral unless crisis) → sum-to-one validation (±0.001, `:2481`) →
`TickSummary` (`:2488-2555`; note `phi_aggregate` and the mean rates use the MELT/100k-employment quantities
via `derived_rates.py:54-109`, not tensor values; national distribution is uniformly weighted because
employment is flat) → `write_tick_state_to_graph` (`:303`; graph metadata carries **live Pydantic objects**
incl. full `county_states`, `graph_bridge.py:97-108` — the survey row-4.0 "graph-scope live Pydantic objects"
blocker, present and load-bearing here for circulation continuity, `graph_bridge.py:323-336`).

## round() and identity encoding

- **`round()` half-even, live:** `reserve_army/accumulation.py:115,121` — `round(delta_occ · employment ·
  rate)` / `round(bankruptcy_rate · employment · rate)` to *integer* persons, exercised every boundary via
  `:1309-1316`. Python `round()` is round-half-even over the IEEE-754 double — the port hazard the survey
  names ("`round()` half-even absent", `reports/port-estate-survey-2026-08-12.md:78`).
- **`round(x, 6)` — event payloads only, never state:** DISPOSSESSION_CASCADE (`:1164-1167`) and
  BIFURCATION_THRESHOLD (`:2336-2340`) payloads. The persisted/graph state (`county.bifurcation_risk.score`,
  class shares) is unrounded; rounding is emission-side cosmetics. Both events fire only in non-NORMAL crisis
  (cascade additionally needs an LA-decline ≥ milestone) — payload rounding therefore possibly never executes
  in this run: UNVERIFIED.
- **Identity encoding (ADR198 R7):** FIPS rides as 5-char strings end-to-end (`territory.py:81-84`;
  `CountyEconomicState.fips` min/max-5; `ClassDistribution.fips` `dynamics/types.py:56`);
  `resolve_county_identity` defensively `str()`-wraps (`graph_bridge.py:73-77`). For R7's int-encoding: **all
  83 Michigan FIPS are state-26 — no leading zero — so int-encoding is lossless over this scenario's entire
  county domain**; the R7 leading-zero trap is *not exercised* by any Michigan row. The external node
  `"canada"` is not a FIPS at all — it falls under R7's "where the string was really naming a node, key by
  node identity" clause (`scopes.py:149`). One live identity-encoding defect class present:
  `bifurcation.py:101` compares node id to FIPS string (T-label vs `"26xxx"`) — silently never-matching, the
  exact pseudo-identity class `resolve_county_identity`'s docstring warns about (`graph_bridge.py:46-64`).

## Reserved-line surfaces (register row 21 — `reports/port-estate-survey-2026-08-12.md:325`; name + cite only, no rulings)

1. **Bifurcation directional score.** Formula `raw = −w_s·solidarity + w_b·burden`, dampened by
   `(1−legitimation)`, clamped [−1,+1]; −1 = revolutionary, +1 = fascist (`crisis/bifurcation.py:9-17,
   :114-124`); direction string at `system/__init__.py:2329`. The three defines:
   `bifurcation_solidarity_weight`/`bifurcation_burden_weight`/`bifurcation_event_threshold` = 1.0/1.0/0.5
   (`config/defines/economy_basic.py:113-134`; `defines.yaml:42-45`), plus `class_burden_epsilon`
   (`economy_basic.py:123-128`). In this scenario: computed per county from the 2nd boundary on
   (`:2265-2304`); **structurally NEUTRAL (score 0) whenever crisis phase is NORMAL** (`bifurcation.py:93-94`);
   solidarity density is computable (2 class entities per county carry `county_fips`,
   `bifurcation.py:144-180`) but no SOLIDARITY edges are seeded (`bridge.py:842-846`) and none can be assumed
   to emerge in null-play (UNVERIFIED) — so the revolutionary (−) term is at risk of being structurally zeroed
   while the burden (+, fascist) term is not. Event emission at |score| ≥ 0.5 UNVERIFIED for this run.
2. **Five-share `ClassDistribution` / Feature-016.** Five shares, bourgeoisie + petit-bourgeoisie externally
   fixed, engine operates on LA/proletariat/lumpen (`dynamics/types.py:27-57`);
   `DefaultClassTransitionEngine.simulate_transitions` (`dynamics/transition_engine.py:107-186`), wired
   (`factory.py:813-818`) and **exercised live**: 83 counties × 9 boundaries (`:2424-2454`), real FRED-backed
   dispossession rates (`:2386-2396`), phase-aware crisis amplification, sum-to-one enforced at ±0.001
   (`:2460-2486`). Bootstrap shares 0.01/0.09/0.40/0.35/0.15 (`:822-830`).
3. **`crisis.dispossession_cascade_milestones`.** `[0.05, 0.10, 0.15]` (`economy_basic.py:137-140`;
   `defines.yaml:46`), read at `:1147`; DISPOSSESSION_CASCADE emitted per county when LA-share decline crosses
   a milestone, gated on non-NORMAL crisis (`:2445-2452`, `:1115-1170`). Firing in this run: UNVERIFIED.

Adjacent (survey row 4.0 libm hazards, not row 21): the wage-pressure **sigmoid** (`reserve_army/calculator.py:32-65`,
ADR188 Row 7 curve-estate surface) IS exercised live here every boundary via `:1239-1241`.

## Open questions

- Did any of the 83 counties leave CrisisPhase.NORMAL during years 2011–2019 (i.e., do CRISIS_PHASE_TRANSITION /
  ECONOMIC_CRISIS / BIFURCATION_THRESHOLD / DISPOSSESSION_CASCADE events exist in `michigan-e2e.json`)?
  Unverifiable locally — the baseline is an LFS pointer (134 bytes) in this workspace.
- Does SolidaritySystem (or any null-play system) mint SOLIDARITY edges in this run? If not, the bifurcation
  score's solidarity term is identically zero and only the fascist-direction burden term can move it.
- `phi_hour` provenance by year: the tensor-hierarchy brief expects 0 unwired sentinels for 2010–2017 and a
  per-year periphery sentinel for 2018+ (`project/execution/briefs/feat-tensor-hierarchy-resolution.md:233`);
  whether 2018–2019 `phi_hour` is real or stub-pass-through (`imperial_rent.py:96-99`) is UNVERIFIED by me.
- The baseline's `county_terminal_snapshot[*].k` (values ~1e10–9e10 per the currency census,
  `reports/currency-magnitude-census-2026-07-29.md:49-54`) is read from `view_runtime_trace_emission`
  (`runner.py:865-876`) — a persistence-path k, **not** TickDynamics' `capital_stock` (≡ 0.0 here). The port
  must not conflate the two k's.
- Is the flat `employment = 100 000.0` (unwired `employment_source`) intended for the canonical 520-tick run?
  It silently equalizes every employment-weighted aggregate (`reserve_army_signal`, `phi_aggregate`, national
  class distribution) and zeroes `flow_wage_accrued` via the never-stamped `tick_employment`.
