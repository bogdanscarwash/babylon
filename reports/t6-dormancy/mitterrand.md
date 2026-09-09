# T6 dormancy re-read — scenario: `mitterrand`

Program 29 train T6 (issue #563, charter ADR198). Reader scope: the `mitterrand`
qa:regression scenario only. Read-only re-derivation from source + committed goldens;
no test runs. Citations are `file:line` against the frozen Python engine.

## Scenario header

- Factory: `create_mitterrand_scenario()` — `src/babylon/engine/scenarios/electoral_goldens.py:209-250`.
  Builds on `create_single_county_scenario()` (`electoral_goldens.py:223-225`), then
  `apply_political_terrain(..., worker_id="C003", owner_id="C004", include_michigan=False)`
  (`electoral_goldens.py:226-228`), re-seeded voters, and two `superstructure_registers`
  (`electoral_governments` seated socdem + a 24-item `policy_agenda`,
  `electoral_goldens.py:244-247`). Returned through the `_warm()` round-trip
  (`electoral_goldens.py:250`, `:178-190`).
- County substrate (inherited from single_county): Territory `T001` with
  `county_fips="26163"` (Wayne County) — `src/babylon/engine/scenarios/single_county.py:113-120`;
  both classes (`C003` Core Bourgeoisie, `C004` Labor Aristocracy — note the electoral
  fixture remaps the worker/owner ids; `_WAYNE_WORKER="C003"`, `_WAYNE_OWNER="C004"`,
  `electoral_goldens.py:46-48`) also carry `county_fips` (`single_county.py:79,92`).
  `county_fips` is a declared `Territory` field (`src/babylon/models/entities/territory.py:81`),
  so it survives every per-tick `WorldState` round-trip. **County-bearing: VERIFIED.**
- Calculator wiring: `mitterrand ∈ WAYNE_CALCULATOR_SCENARIOS`
  (`tools/regression_scenarios.py:151-153`) → `build_single_county_overrides(defines)`
  (`tools/regression_test.py:1031-1035`) = `create_financial_services(fred fixture)`
  (12 keys, `src/babylon/domain/economics/factory.py:477-490`) + fixture `melt_calculator`
  (`tools/regression_test.py:206`) + one-county `tensor_registry` (Wayne 2010,
  `tools/regression_test.py:236-256`, fixture `tests/fixtures/single_county_wayne.json`).
  Everything else in `ServiceContainer.create` defaults to `None`
  (`src/babylon/engine/services.py:298-346`).
- Run shape: `DEFAULT_MAX_TICKS=52` (`tools/regression_test.py:81`), loop
  `range(1, 53)` (`tools/regression_test.py:1054`). Baseline: `tests/baselines/mitterrand.json`
  (`ticks_survived=52`, `final_outcome="SURVIVED"`); dense golden
  `tests/baselines/dense/mitterrand.csv` (54 lines = header + ticks 0..52) with county columns
  `county_26163_{total_s,interest,ground_rent,taxes,profit_enterprise}` and national
  `financial_{endogenous_rate,profit_rate_ceiling,s_r,tightness}` (column contract:
  `tools/regression_test.py:415-433`, `:459-469`).

## The annual pipeline — when it fires (premise correction)

**The issue's "tick 52" claim is wrong for this scenario's harness path.** The gate is
`tick % WEEKS_PER_YEAR != 0` (`src/babylon/domain/economics/tick/system/__init__.py:174`;
`WEEKS_PER_YEAR=52`, `src/babylon/data/defines.yaml:374`), and `simulation_engine.step()`
builds `TickContext(tick=state.tick, ...)` from the PRE-increment tick
(`src/babylon/engine/simulation_engine.py:566-568`). The 52 harness calls therefore run
context ticks **0..51**, and the single year boundary is **context tick 0 — the very first
`step()` call** — computing year 2010 (`_determine_year`, `system/__init__.py:409-423`,
`base_year` default 2010). In-harness documentation says so verbatim:
"the annual pipeline fires exactly once per 52-tick run, on the first step() call"
(`tools/regression_test.py:500-502`); the save-side note at `simulation_engine.py:478-485`
says the same ("`0 % 52 == 0` fires the financial layer on the very FIRST step() call");
the fixture provenance agrees ("year 2010 is what that shared harness path serves at its
tick-0 boundary", `tests/fixtures/single_county_wayne.json:2`).

Golden proof: in `tests/baselines/dense/mitterrand.csv` all four `financial_*` and all five
`county_26163_*` cells are `0.0` at tick 0 and at their full values **from tick 1 onward,
flat through tick 52** (e.g. `financial_endogenous_rate=0.017833969800785755`,
`county_26163_total_s=3234158620.500001`). A tick-52 boundary would show zeros through
row 51; it does not.

Non-boundary ticks 1–51 are NOT inert either: `_save_graph_context` persists the
`tick_dynamics` payload (`simulation_engine.py:491-494`), `_restore_graph_context`
re-stamps it onto the rebuilt graph (`simulation_engine.py:450-451`), so the non-boundary
branch re-stamps T001's ~35 `tick_*` attrs verbatim and accrues the spec-109 A7 flow slices
(`system/__init__.py:183-187`, `_accrue_flows` `:315-355`; `flow_wage_accrued` grows by
21.0×2080×100000/52 = 84,000,000.0/tick, `flow_phi_accrued` by 0.0). These stamps are
graph-only: `Territory` declares no `tick_*`/`flow_*` fields
(`src/babylon/models/entities/territory.py:76-289`, contrast comment `:236`), so
`from_graph` drops them — invisible to the tick hash and to every dense column.

What the one boundary (year 2010) computes for mitterrand:

- **Step 2 (national params):** `tau = get_melt(2010)` from the committed fixture (2010
  present in `tests/fixtures/vol3_melt_national.json` — verified); `gamma_basket=0.68`,
  `gamma_III=0.33` from defines defaults (both calculators unwired — warning + fallback
  tally, `system/__init__.py:550-588`, defaults `defines.yaml:100-101`); smoother
  first-init (`:894-902`).
- **Step 3a (county state):** domain = `["26163"]` from T001 (`_get_territory_fips`,
  `:425-449`); all unwired-source defaults (`capital_stock=0.0`, throughput `1.0/2.0`,
  U-3 `0.05`, renter `0.0`, median_wage `21.0`, employment `100_000.0`, deflator `1.0`,
  bracket_ratio `0.0` — `:743-804`). Precarity derived live from the frozen lumpen share:
  u6 `0.20`, pter `0.06`, nilf `0.09` (`src/babylon/domain/economics/tick/precarity.py:59-62`).
- **Step 3.5 (Vol I wage pressure): SKIPPED** — `reserve_army_data_source is None`
  (`:1196-1197`).
- **Step 3.6 (accumulation loop): runs but writes NOTHING.** `tensor_registry` is wired so
  the guard at `:1287` passes, but `occ_prior` (2009) is `NoDataSentinel→None` and
  `bankruptcy_rate=None` ⟹ `compute_dynamics` returns `None`
  (`src/babylon/domain/economics/reserve_army/accumulation.py:103-126`) ⟹ empty `updates`,
  no `reserve_army_stock`/`reserve_ratio`/`foreclosure_rate`/`eviction_rate` stamps (`:1317-1334`).
- **Step 4 (imperial rent): unwired pass-through.** All 5 Spec-057 fields are `None` ⟹
  `_spec_057_pipeline_wired` False ⟹ one wildcard sentinel event
  (`CALIBRATION_QCEW_CARRY_FORWARD`, `county_fips="*"`, **payload tick hardcoded 0**,
  `src/babylon/domain/economics/tick/system/imperial_rent.py:172-194`) and `phi_hour`
  passes through as the bootstrap `0.0` (`:86-88`, `:221-230`). Note: the "Wayne's
  `tick_phi_hour` is a MEASURED 0.0" framing (`electoral_goldens.py:11-13`) describes the
  runner terrain; in THIS harness the 0.0 is the unwired bootstrap default, not a measurement.
- **Step 4.5 (Vol II circulation): SKIPPED entirely** — `turnover_profile_source is None`
  (`:1409-1410`); `circulation_state` stays `CirculationCrisisState.default()`.
- **Step 5 (crisis triggers):** 4 quarterly evaluations (`:972-984`) with Wayne's native-2010
  `profit_rate=0.05944656600261918 ≥ r_threshold=0.05` (`defines.yaml:35`) ⟹ stays NORMAL —
  no phase events, no wage compression.
- **Step 5.5 (Vol III financial): FIRES FOR REAL.** `_economy_wide_profit_rate` =
  0.05944656600261918 (single-county telescoping, `:1858-1919`); `s_r=0.0` (U-3 0.05 <
  `interest_reserve_reference=0.08`, `graph_bridge.py:511-542`, `defines.yaml:1070`);
  `tightness=0.0`; endogenous rate 0.017833969800785755 — all golden-verified.
  `distribution_calculator.compute_distribution` on `total_s=$3,234,158,620.50` yields the
  identity **s = p + i + r + t**: 1,888,358,776.29 + 970,247,586.15 + 199,896,129.03 +
  175,656,129.03 = 3,234,158,620.50 (golden cells; the `taxes` cell IS the scenario's
  advertised `t_claim ≈ $175.7M`, `electoral_goldens.py:10`). Rent and housing calculators
  fire but return `NoDataSentinel` (rent: stub adapter all-None, `factory.py:454-466` +
  `rent/calculator.py:109-115`; housing: `("26163", 2010)` absent from the Census defaults,
  which carry only 2015/2020/2022 — `src/babylon/domain/economics/data_adapters.py:121-137`)
  ⟹ two sentinel tallies + throttled warnings (`system/__init__.py:2052-2067`).
  Fictitious capital: all inputs present at 2010 (GFDEBTN/NCBEILQ027S fixture + Z1 2010 row,
  `data_adapters.py:42-46`) ⟹ real `FictitiousCapitalStock` (modeled-window bounds imported
  at `fictitious_capital.py:24-25`, not opened — near-certain pass at 2010).
  `financial_crisis_assessor.assess` fires (`:2146-2154`). `national_financial` published to
  the graph and saved to `persistent_context` (`graph_bridge.py:433-450`,
  `simulation_engine.py:489-490`) — the dense columns' read path (`regression_test.py:548-553`).
- **Step 6 (Feature-016 transitions): SKIPPED** — `transition_engine is None` (`:2366-2367`);
  the five shares stay frozen at bootstrap defaults 0.01/0.09/0.40/0.35/0.15 all run.
- **Step 5b (bifurcation): NEVER COMPUTED.** `prev_county_states` at the first boundary is
  the bootstrap `{}` (`:213`, `:451-512` — T001 is bare post-round-trip, so no
  `tick_capital_stock`, `:480`), which is not `None`, so the `:2265` guard passes — but
  `prev_county_states.get("26163")` is `None` ⟹ per-county skip (`:2281-2284`). The metric
  stays the default (`score=0.0`, `tick/types.py:173`).
- **Steps 7–8:** sum-to-one validation passes on the frozen defaults (`:2472-2486`);
  `TickSummary(counties_processed=1, phi_aggregate=0.0, ...)` (`:2544-2555`).
- **Write + Step 9:** ~35 `tick_*` attrs stamped on T001 (`graph_bridge.py:173-295`);
  `hex_grid is None` ⟹ no-op (`:392-394`); flow counters reset (`:357-372`).

## Live fields (11 of 33; the ServicesProtocol touch-set is 28 direct reads + 5 Spec-057)

| services field | evidence |
|---|---|
| `melt_calculator` | gate `:198`; `get_melt(2010)` `:542`; fixture-backed (`regression_test.py:177-183`) |
| `defines` | thresholds/weights throughout (`:963-974`, `:2270-2277`, `:2369`, …) |
| `economics_fallbacks` | auto-built (`engine/services.py:247`); `observe_wiring` `:534-539`; basket/gamma None tallies `:564,:588`; rent+housing sentinels `:2055,:2064` |
| `tensor_registry` | Wayne 2010 tensor: profit rate `:1042`, departments/surplus `:2178-2239`, OCC `:1303-1304` |
| `distribution_calculator` | `compute_distribution` `:2023-2034` — the golden identity |
| `fictitious_capital_calculator` | `:1964-1970`; 2010 inputs verified present |
| `credit_aggregate_source` | `_build_credit_state` `:1845-1856`; TCMDO 2010 present |
| `financial_crisis_assessor` | `:2070-2082`, `:2146-2154` |
| `rent_calculator` | fires → `NoDataSentinel` (sentinel path, `:2052-2058`) |
| `housing_calculator` | fires → `NoDataSentinel` (sentinel path, `:2061-2067`) |
| `event_bus` | exactly ONE emission per run: the pipeline-unwired sentinel (`imperial_rent.py:188-194`) |

## Dormant fields (22 of 33)

Unwired (`None`) in this harness, so only the degradation branch fires:

- `basket_calculator` (`:552`), `gamma_calculator` (`:568`) — defines defaults used.
- `capital_calculator` (`:748`), `throughput_calculator` (`:756`), `unemployment_source`
  (`:767`), `housing_source` (`:655`), `income_source` (`:678`), `cpi_source` (`:709`),
  `wage_source` (`:785`), `employment_source` (`:794`) — all county fields keep documented
  defaults.
- `reserve_army_data_source` (`:1196`) — whole Vol I layer skipped.
- `dispossession_data_source` (`:1286`, `:2386`) — no rates; DEFAULT literals moot (Step 6 skipped).
- `turnover_profile_source` (`:1409`) — whole Vol II layer skipped; `inventory_data_source`
  /`depreciation_data_source` (`:1455-1456`) unreachable with it.
- `transition_engine` (`:2366`) — Feature-016 frozen (reserved line, below).
- `hex_grid` (`:392`) — Step 9 no-op.
- Spec-057 five: `periphery_labor_source`, `final_demand_source`, `industry_county_allocator`,
  `production_chain_calculator`, `bea_industries` (`imperial_rent.py:158-169`) — unwired
  sentinel path.

Wired by `create_financial_services` but **never read by TickDynamics** (boundary-adjacent,
not part of the 33): `interest_calculator` ("calibration-only", `:1763-1765`),
`credit_cycle_detector`, `counter_tendency_calculator`, `value_basis_converter`, `z1_source`,
`housing_data_source` (`factory.py:477-490`).

## round() and identity encoding

- `round(x, 6)` sites — Python built-in round = **half-even** on the IEEE-754 double:
  dispossession-cascade payload (`system/__init__.py:1164-1167`), bifurcation payload
  (`:2336-2340`), reserve-army displacement/failures (`accumulation.py:115,121`).
  **None fire in mitterrand** (Step 6 skipped; bifurcation never computed; dynamics None) —
  the survey row's "round() half-even absent" gap (`reports/port-estate-survey-2026-08-12.md:78`)
  is confirmed for this scenario by non-execution, not just by code shape.
- Golden serialization does NOT round: floats are `repr()` shortest-round-trip
  (`src/babylon/engine/trace_format.py:20-22,55-57`); the tick hash encodes floats as
  `struct.pack(">d").hex()` and passes strings through verbatim
  (`src/babylon/kernel/tick_hash.py:100-115,164-166`).
- County identity: `resolve_county_identity` reads ONLY the `county_fips` attr; node id
  `T001` never substitutes (`graph_bridge.py:44-77`). FIPS is `str` with
  `min_length=max_length=5` (`tick/types.py:320`, `dynamics/types.py:56`) — the frozen Python
  never int-encodes it. ADR198 R7's int-encoded-FIPS ruling is a Rust-port surface; Wayne's
  `"26163"` has no leading zero, so **this scenario does not exercise the R7 leading-zero
  trap** (a `"06037"`-style county would). `county_states == {"26163": …}` non-empty at the
  boundary is golden-verified (the `county_26163_*` columns).

## Reserved-line surfaces (register row 21 — `reports/port-estate-survey-2026-08-12.md:325`; named + cited, no rulings)

1. **Bifurcation directional score** — `BifurcationRiskCalculator`
   (`src/babylon/domain/economics/crisis/bifurcation.py`), lazily built with
   `bifurcation_solidarity_weight=1.0` / `bifurcation_burden_weight=1.0` /
   `class_burden_epsilon=0.001` (`defines.yaml:42-44`) at `system/__init__.py:2269-2275`;
   direction mapping `"revolutionary" if score < 0 else "fascist"` at `:2329`; event
   threshold `bifurcation_event_threshold=0.5` (`defines.yaml:45`) at `:2295`.
   In mitterrand: **constructed, never computed** (first-and-only boundary has empty
   `prev_county_states`, `:2281-2284`); `tick_bifurcation_score` stamped `0.0`
   (`graph_bridge.py:181`).
2. **Five-share `ClassDistribution` / Feature-016** — model at
   `src/babylon/domain/economics/dynamics/types.py:27-66` (five shares, string FIPS);
   bootstrap defaults `0.01/0.09/0.40/0.35/0.15` (`system/__init__.py:822-830`);
   sum-to-one validator runs (`:2460-2486`); the transition engine call `:2424` is gated by
   `transition_engine is None` (`:2366`) — **the five shares are schema-present,
   validation-exercised, and transition-frozen for the entire run**.
3. **`crisis.dispossession_cascade_milestones`** — `[0.05, 0.10, 0.15]`
   (`defines.yaml:46-49`; `src/babylon/config/defines/economy_basic.py:137`); read only at
   `system/__init__.py:1147` inside `_check_dispossession_cascade`, callable only from the
   skipped Step 6 (`:2444-2452`, additionally gated on `crisis_phase != NORMAL`) —
   **unreachable in this scenario**.

## Open questions

- The issue's "annual pipeline at tick 52" premise contradicts this harness's own documented
  behavior (first `step()` call, context tick 0). Does the claim originate from the
  runner-backed scopes' tick numbering (detroit-tri-county / michigan-canada)? Out of
  mitterrand's scope — flagging for the synthesis reader.
- `year_within_modeled_range(2010)` bounds (`MODELED_YEAR_FLOOR/CEILING`, imported at
  `fictitious_capital.py:24-25`) not opened; fictitious-capital liveness inferred from all
  three fixture inputs being present at 2010. UNVERIFIED at the window-guard line only.
- The `electoral_goldens.py:11` "MEASURED 0.0" phi wording vs the qa harness's unwired
  bootstrap `0.0` — a doc nuance the port should not inherit uncritically (the Leontief
  stage is dormant here).
- `Territory.reserve_ratio`/`reserve_army_stock`/`foreclosure_rate`/`eviction_rate` ARE
  declared model fields (`territory.py:220,226,259,265`) — had the accumulation loop written
  them they would enter the WorldState and the hash. In mitterrand it writes nothing; the
  distinction (hash-visible declared fields vs graph-only `tick_*`/`flow_*` stamps) is a
  port-relevant asymmetry this scenario leaves unexercised.
