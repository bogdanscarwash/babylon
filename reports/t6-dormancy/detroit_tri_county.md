# T6 dormancy re-read — `detroit_tri_county`

Program 29 train T6 per-scenario re-derivation (GitHub #563, charter ADR198).
Reader scope: this scenario only. All claims cite file:line; anything not
checkable from source is marked UNVERIFIED. No test suites were run
(swarm machine-safety rule); baseline files were inspected read-only.

## Scenario header

- **Scope resolution:** `--scope detroit-tri-county` →
  `Scope(frozenset({"26163","26125","26099"}), frozenset({"canada"}))`
  (`src/babylon/engine/headless_runner/scopes.py:122-123,152-153`) — Wayne,
  Oakland, Macomb + one external node (`canada`).
- **Canonical artifacts:** `tests/baselines/detroit-tri-county-5t.json`
  (5 ticks; `run_metadata.ticks_requested=5`), the e2e dense golden
  `tests/baselines/dense/detroit_tri_county.csv` (5 rows, generated with
  `--scope detroit-tri-county --ticks 5 --strict` per
  `tests/baselines/README.md:49`), and the vault golden
  `tests/baselines/vault/detroit_tri_county/` (**3 ticks** —
  `manifest.json:"ticks": 3`). The CI comparator runs `--ticks 5`
  (`tests/integration/test_ci_gate_baseline_compare.py:58-59`).
  `detroit_tri_county` is NOT a `SCENARIOS` key; the coverage probe checks
  it only via committed-bundle rows (`tools/gate_coverage_probe.py:17,127`;
  `tools/regression_scenarios.py:2010-2021`).
- **county_fips carried?** YES. Territories `T001/T002/T003` are minted with
  `county_fips=county_fips` from `sorted(scope_fips)`
  (`src/babylon/engine/headless_runner/bridge.py` `_build_per_county_territories`,
  ~:805-820; grep hits :812-813). Social classes `C001-C003` (labor
  aristocracy, pop 85) and `C501-C503` (bourgeoisie, pop 15) also carry
  `county_fips` (bridge.py:769-786). So `resolve_county_identity` would find
  3 real FIPS — the county-free dormancy pattern of
  `tools/regression_scenarios.py:406-451` does NOT apply structurally.
- **Wiring:** `_build_economics_overrides(session_factory, event_bus,
  defines, scope_fips)` is called at `runner.py:1319-1324` and wires the
  full calculator estate (see below); `ServiceContainer.create` defaults
  cover `defines`/`event_bus`/`economics_fallbacks`
  (`src/babylon/engine/services.py:247,409-459`; `event_bus`/
  `boundary_register`/`auditor` set post-create at `runner.py:1329-1331`).

## The headline fact (read first)

**In every committed detroit_tri_county artifact, TickDynamicsSystem's annual
pipeline fires ZERO times.** The runner persists tick 0 WITHOUT running the
engine (`runner.py:1660-1668`: `bridge.persist_tick(world, 0, ...)`; the
engine runs only inside `_advance_tick`, `runner.py:448`) and loops
`range(1, config.ticks)` (`runner.py:1694`). A `--ticks 5` run therefore
executes engine ticks {1,2,3,4}; the vault golden {1,2}. The annual gate
`if tick % WEEKS_PER_YEAR != 0` (`tick/system/__init__.py:174`,
`WEEKS_PER_YEAR=52` at `src/babylon/data/defines.yaml:374`) diverts ALL of
them to the non-boundary branch, which returns at :187. Verified against the
artifacts:

- The dense golden's 13 annual-pipeline columns
  (`county_{26099,26125,26163}_{interest,ground_rent,taxes}` + 4
  `financial_*`) are `"0.0"` in every one of the 5 rows (cell inspection;
  matches the Task-11 at_rest ceremony rows,
  `tools/regression_scenarios.py:2047-2324`, "Verified live, 2026-07-20").
- The 5t JSON contains no `CRISIS_PHASE_TRANSITION`, `BIFURCATION_THRESHOLD`,
  `DISPOSSESSION_CASCADE`, or `CALIBRATION_QCEW_CARRY_FORWARD` events (rg
  count: 0) — all four emitters live inside the annual pipeline.

This CONTRADICTS the issue's premise as stated for this scenario: the six
scenarios are indeed county-bearing, but detroit_tri_county's gated
baselines never "drive the annual pipeline at tick 52" — only the
520-tick `--scope michigan-canada` e2e does (`.mise.toml` e2e task;
`tests/baselines/michigan-e2e.json` is a 105 MB LFS pointer). For detroit,
tick 52 is a counterfactual any `--ticks ≥53` run would reach, not an
exercised one.

## Live fields (as-run, committed artifacts)

TickDynamics touches **zero** ServicesProtocol fields in the baseline. Per
tick it executes only:

1. The gate read of `context.tick` (:163) and the modulo (:174).
2. `read_tick_state_from_graph(graph)` (:183) → always `None`, because
   `graph.graph["tick_dynamics"]` (`TICK_DYNAMICS_KEY`,
   `graph_bridge.py:41,314-316`) is written only by
   `write_tick_state_to_graph` at the END of a boundary run (:303) — which
   never happens. The P25-U13 restamp (:184-185) therefore never fires.
3. `_accrue_flows(graph)` (:186, :315-355) → iterates the 3 TERRITORY
   nodes, reads `tick_phi_hour` (:338), finds it absent, and skips each
   (:340 "empty domain"). `tick_phi_hour`'s ONLY write site repo-wide is
   `graph_bridge.py:178` (inside the annual stamp), so the accrue is a
   **guaranteed** no-op, not a contingent one. No `flow_phi_accrued`/
   `flow_wage_accrued` is ever written.

The dense golden's live columns (`county_<fips>_{total_v,total_c,total_s,
total_k,population}`) are populated by session hydration + other systems,
not by TickDynamics (`tools/regression_scenarios.py:2065-2067`).

## Dormant fields

Two distinct dormancy classes.

### Class A — wired but never consulted (annual-pipeline-gated)

All 25 of the protocol fields TickDynamics' code reads that ARE wired for
this scenario are dormant as-run, because the non-boundary branch returns
before any service is touched (:187 precedes the `melt_calculator` gate at
:198):

| Field | Wired at | Read at |
|---|---|---|
| `melt_calculator` | runner.py:1075 | :198, :542 |
| `gamma_calculator` | runner.py:1055-1057 | :568 |
| `unemployment_source` | runner.py:1078 | :767 |
| `wage_source` | runner.py:1081 | :785 |
| `cpi_source` | runner.py:1084 | :709 |
| `tensor_registry` (3 FIPS × 2010-2024) | runner.py:1110-1121 | :228, :1038, :1285, :1356, :1503, :2178, :2204, :2231 |
| `periphery_labor_source` / `final_demand_source` / `industry_county_allocator` / `production_chain_calculator` / `bea_industries` | `factory.py:244-248` via runner.py:1089-1092 | `imperial_rent.py:96,102,107,126-128,138,90` |
| `distribution_calculator` / `fictitious_capital_calculator` / `credit_aggregate_source` / `rent_calculator` / `housing_calculator` / `financial_crisis_assessor` | `factory.py:478-489` via runner.py:1123-1126 | :1765/:2017, :1964, :1845, :2052, :2061, :2070/:2146 |
| `turnover_profile_source` / `inventory_data_source` / `depreciation_data_source` | `factory.py:618-620` via runner.py:1148-1154 | :1409/:1609, :1455, :1456 |
| `reserve_army_data_source` / `dispossession_data_source` / `transition_engine` | `factory.py:814-817` via runner.py:1156-1162 | :1196/:1235, :1286/:1326-1329/:2386, :2366 |
| `defines`, `event_bus`, `economics_fallbacks` | services.py:247,414; runner.py:1329 | passim; :1087/:2330/:1158; :534/:545/:564 etc. |

### Class B — unwired even at tick 52 (would stay dormant/defaulted in ANY detroit run)

Seven protocol fields TickDynamics reads are NOT in
`_build_economics_overrides` and default to `None` in
`ServiceContainer.create` (services.py:304-346), so even the counterfactual
tick-52 pipeline degrades on them:

- `basket_calculator` → gamma_basket falls back to defines default 0.68 +
  WARNING + `record_gamma_basket_calculator_none()` (:550-564;
  `economy_basic.py:386-395`).
- `capital_calculator` → `capital_stock` stuck at bootstrap `0.0` (:747-751).
- `throughput_calculator` → `throughput_position=1.0`, `supply_chain_depth=2.0`
  defaults (:754-760).
- `employment_source` → `employment=100_000.0` default per county (:793-797).
  (Side effect: `reserve_army_signal`'s employment-weighted U-3 aggregate,
  `graph_bridge.py:476-508`, weights all 3 counties equally; the dense
  golden's real per-county populations are a DIFFERENT data path.)
- `housing_source` → `renter_share` stays `0.0` (:654-659).
- `income_source` → `bracket_ratio` stays `0.0` (:677-682).
- `hex_grid` → `_write_hex_substrate` no-ops (:392-394; hex hydration exists
  in Postgres via `initialize_session`, runner.py:1224/1230-1233, but the
  in-memory `services.hex_grid` is never set).

### Class C — wired but never read by this System (any scenario)

`productivity_data_source`, `interest_calculator`, `credit_cycle_detector`,
`counter_tendency_calculator`, `value_basis_converter`, `z1_source`,
`housing_data_source` — all wired (factory.py:478-489,814-817) but no
reference exists in `tick/system/__init__.py` or `imperial_rent.py`
(the U9 comment at :1763-1764 documents `interest_calculator` as
calibration-only). Dormant-by-design for TickDynamics; consumed elsewhere
or nowhere.

## Annual pipeline

- **When it fires:** at `tick ≡ 0 (mod 52)` (:174). In the runner the first
  engine-processed boundary is **tick 52** (tick 0 is persist-only,
  runner.py:1660-1668; loop `range(1, ticks)`, runner.py:1694) — the issue's
  "tick 52" is correct for the runner path, but only for runs of ≥53 ticks.
  No committed detroit artifact qualifies (5t JSON, 5-row dense CSV, 3-tick
  vault). UNVERIFIED: whether any non-gated long detroit run exists in
  project history.
- **What it would compute at tick 52 for these 3 counties** (year =
  `base_year 2010 default + 52//52 = 2011`, :409-423; no `base_year` graph
  attr is set by the runner — rg finds no writer outside the system's own
  default):
  1. Step 2 national params (:216, :514-628): real MELT τ for 2011;
     gamma_basket from defines default (basket unwired); gamma_III from the
     wired calculator; EM smoothing α=0.3 (:133, :591-601).
  2. Step 3a county states (:231, :715-852): bootstrap path
     (`_bootstrap_county_states` :213 finds no `tick_capital_stock` on the
     bare territories, :480, so returns empty; FIPS then resolved from the
     graph via `_get_territory_fips` :225/:425-449 → the 3 real FIPS;
     `county_states` NON-EMPTY — the county-free pattern of
     regression_scenarios.py:406-451 is escaped). Per county: real U-3
     (LAUS), real p50 wage seed (QCEW), real CPI deflator; capital/
     throughput/employment/renter/bracket at defaults (Class B).
  3. Precarity from lumpen share (:234, :854-878); coefficients (:237);
     Vol I wage pressure via FRED reserve-army decomposition (:240,
     :1196-1241); accumulation loop writing `reserve_army_stock`,
     `reserve_ratio`, `foreclosure_rate`, `eviction_rate` onto T001-T003
     (:247, :1243-1334).
  4. Step 4 imperial rent: full Spec-057 Leontief pipeline → real per-county
     `phi_hour` (:250, `imperial_rent.py:45-150`; all 4 services + bea_industries
     wired, so NOT the stub path).
  5. Step 4.5 Vol II circulation (:253, :1367-1441) with real FRED inventory/
     depreciation and tensor department rows (2011 inside the 2010-2024
     hydration window, runner.py:96/:1118-1120).
  6. Step 5 crisis triggers — 4 quarterly evaluations per boundary (:972-984)
     against tensor profit rates with carry-forward (:1019-1059).
  7. Step 5.5 Vol III financial layer (:261, :1737-1794): endogenous national
     interest from the surplus-weighted economy-wide profit rate
     (:1858-1919) + reserve-army signal (`graph_bridge.py:511-542`);
     per-county surplus distribution s = p+i+r+t (:2017-2039), rent
     extraction, housing decomposition, crisis assessment.
  8. Step 6 Feature-016 class transitions (:270, :2346-2458) with real FRED
     dispossession rates (:2386-2396); Step 5b bifurcation (:279);
     Step 7 sum-to-one validation (:288, :2460-2486); Step 8 TickSummary
     (:291, :2488-2555); write to graph (:303); hex substrate no-op (:307);
     flow-accrual reset (:313).
- Subsequent boundaries (104, 156, …) would take the `existing_state` path
  (:206-209) with year = prior+1 and full cross-tick continuity through the
  graph metadata (never instance state — U3 determinism fix, :1380-1396).

## Rounding + identity encoding

- **round():** Python `round()` (half-even / banker's, on binary doubles)
  appears at exactly 9 sites reachable from this system:
  - DISPOSSESSION_CASCADE payload: `round(decline,6)`, `round(current_la,6)`,
    `round(baseline_la,6)` (:1164-1167).
  - BIFURCATION_THRESHOLD payload: `round(score,6)`, `round(solidarity,6)`,
    `round(legitimation,6)`, `round(burden,6)` (:2336-2340).
  - Accumulation-loop headcounts: `round(delta_occ * employment * rate)` and
    `round(bankruptcy_rate * employment * rate)` — no ndigits, i.e.
    round-half-even TO INTEGER
    (`src/babylon/domain/economics/reserve_army/accumulation.py:115,121`).
  - None in `derived_rates.py`, `smoothing.py`, `crisis_detector.py`,
    `imperial_rent.py`, or `dynamics/` (transition engine). `precarity.py:32`
    is a doctest only. Port note for ADR198: Python half-even ≠ Rust
    `f64::round` (half-away); the event payloads are byte-compared in
    baselines, so the semantic must be specified, not assumed. All 9 sites
    are dormant as-run anyway (Class A).
- **County identity:** STRING FIPS end-to-end, NOT int-encoded. Territory
  nodes carry `county_fips` as a 5-char string ("26099" etc., from
  `sorted(scope_fips)` of `scopes.py:123` literals); `resolve_county_identity`
  is a plain `str()` passthrough (`graph_bridge.py:44-77`);
  `ClassDistribution.fips` enforces `min_length=5, max_length=5`
  (`dynamics/types.py:54`). Graph node ids are `T001`-style labels; the stamp
  path maps FIPS→node id via `resolve_county_identity`
  (`graph_bridge.py:146-164`). If ADR198 R7's int-encoded FIPS is the Rust
  target, the Python reference gives no int semantics to port — encoding is
  a new decision, and zero-padding preservation is the only invariant the
  reference pins (via the 5-char string constraint).
- **Latent identity mismatch (R7-adjacent, as-designed):**
  `BifurcationRiskCalculator` reads lifecycle legitimation via
  `node.id == fips` on TERRITORY nodes (`crisis/bifurcation.py:101-104`) —
  never true in this scenario (ids are `T001`-`T003`), so
  `lifecycle_legitimation` is always `None` and legitimation falls back to
  the agitation-inverse (bifurcation.py:221-225). Its social-class lookups
  DO match on the `county_fips` attribute (bifurcation.py:150, 212), so
  solidarity/agitation inputs would resolve; only the Feature-030 blend is
  unreachable. Dormant as-run regardless (Class A).

## Reserved-line surfaces (name + cite only; no rulings)

1. **Bifurcation directional score.** Semantics: `-1 revolutionary … +1
   fascist` (`crisis/bifurcation.py:9-17`); formula `raw = -w_s*solidarity +
   w_b*burden; dampened = raw*(1-legitimation); clamp[-1,+1]`
   (bifurcation.py:114-117); direction string `"revolutionary" if score < 0
   else "fascist"` (:2329); event gate `|score| >= 0.5` (:2295,
   `economy_basic.py:129-134`). Detroit status: never computed as-run; even
   at tick 52 it returns `BifurcationRiskMetric.neutral()` while crisis phase
   is NORMAL (bifurcation.py:93-94), and the seeded graph has only
   EXPLOITATION edges (bridge.py:826 ff.), so `solidarity_density` starts at
   0.0 (bifurcation.py:144-180) unless SolidaritySystem adds edges mid-run
   (UNVERIFIED).
2. **Five-share ClassDistribution / Feature-016.** Model:
   `dynamics/types.py:27-52` + sum-to-one validator (tolerance 0.001);
   engine: `DefaultClassTransitionEngine.simulate_transitions`
   (`dynamics/transition_engine.py:57,107`); invoked per county at step 6
   (:2424-2428) with phase-aware amplification and the wage-floor
   accumulation halt (:2377-2380, floor ratio 0.8 at `economy_basic.py:105-110`);
   year clamped to [2007,2030] at both entry and result (:2374, :2411-2422,
   :2432-2442); re-validated at step 7 (:2472-2486). Detroit status: wired
   (`transition_engine` from factory.py:817) but never executed as-run.
3. **dispossession_cascade_milestones (register row 21).** Define:
   `economy_basic.py:137-140`, default `[0.05, 0.10, 0.15]`; consumed at
   :1147-1151 with the highest-crossed-milestone selection; emitted as
   `DISPOSSESSION_CASCADE` with half-even-rounded payload (:1158-1169);
   gated on `crisis_phase != NORMAL` AND non-None prev states (:2444-2452)
   and on prev-county presence (:1136-1138). Detroit status: never emitted
   as-run (baseline contains zero such events; gate requires both a boundary
   and an active crisis).

## Open questions

1. Should a ≥53-tick detroit-tri-county run exist as a gated artifact so the
   Class-A estate is exercised for THIS scope (Wayne/Oakland/Macomb), rather
   than inferred live from single_county (qa) and michigan-canada (520-tick
   e2e)? The current gates pin only the 5-tick/3-tick behavior.
2. Is the Class-B unwiring (basket/capital/throughput/employment/housing/
   income/hex_grid) deliberate runner scope-trimming or an oversight?
   `employment_source` especially: the runner wires wage + unemployment but
   not employment, so every county's `employment` is the 100k default at a
   boundary — a materially wrong input for reserve-army scaling in a scope
   whose real counties differ 2× in population.
3. The `node.id == fips` legitimation lookup (bifurcation.py:101-104) can
   never match `T001`-style runner territory ids — dead input in every
   runner scenario. Intentional (Feature-030 web-only seam) or defect?
   UNVERIFIED against the Feature-030 spec.
4. First in-runner boundary computes year **2011**, not 2010 (tick 0 never
   runs the engine, :409-423) — the 2010 annual frame is hydration-owned.
   Confirm this year-offset is the intended port semantics.
5. `tools/regression_scenarios.py:2062` cites `runner.py:1566` for
   `tick_range`; the line is now 1694 — doc drift only, no behavioral
   discrepancy (the ceremony text's mechanism re-verified true here).
