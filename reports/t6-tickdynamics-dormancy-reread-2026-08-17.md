# TickDynamics engineering memo — per-scenario dormancy + ServicesProtocol boundary charter

**Issue:** #563 (Program 29 train T6) · **Charter:** ADR198 R8 · **Ruling:** ADR208 R13
(2026-08-17 docket, "TICKDYNAMICS — SPLIT: the engineering half proceeds now"). System:
`TickDynamicsSystem` @4.0, `src/babylon/domain/economics/tick/system/__init__.py:112`.

**Method — this is a RE-READ, not a re-derivation.** T6 already delivered the required
per-scenario dormancy re-read on 2026-08-14: six memos in `reports/t6-dormancy/`
(`single_county`, `mitterrand`, `syriza`, `bernie_valve`, `detroit_tri_county`,
`michigan_canada_e2e`) plus a synthesis, `reports/t6-tickdynamics-services-charter-2026-08-14.md`.
Register memo row 21 (`reports/register-memos/rows-21-24.md`) independently re-verified the
reserved-trio surfaces the same day. This memo re-reads those four documents plus the Phase-1
port inventory (`reports/port-inventories/tick-dynamics-port-phase1-inventory-2026-08-12.md`)
and cross-checks their load-bearing claims directly against `src/babylon/kernel/services.py`,
`system/__init__.py`, and `imperial_rent.py` (all `rg`/`sed` reads, nothing executed). Every
number below that could be independently verified from source (the 33-field count, the
9 `round()` sites, the accumulation-loop early-return guard) WAS independently re-verified for
this memo, not just copied from the prior write-up — see the confirmation/correction notes
inline. No scenario was run.

---

## 1. Headline — the "tick 52" premise, re-confirmed REFUTED as stated

The issue framed six county-bearing scenarios as "driving the annual pipeline at tick 52."
The T6 synthesis's re-derivation holds against a fresh read:

| Scenario | Harness | When the annual pipeline (`tick % 52 == 0`) actually fires |
|---|---|---|
| `single_county` | qa (`SimulationEngine.step`) | Context tick 0 — the FIRST `step()` call (pre-increment `TickContext`, `simulation_engine.py:566-569`); once per run |
| `mitterrand` | qa | Context tick 0 — golden-verified (`financial_*` 0.0 at tick 0, flat from tick 1) |
| `syriza` | qa | Context tick 0 — same golden proof |
| `bernie_valve` | qa | Context tick 0 — same golden proof |
| `detroit_tri_county` | headless runner | NEVER in any committed artifact — the 5-tick/3-tick gates run engine ticks {1..4}, all diverted by `tick % 52 != 0`; tick 52 needs `--ticks ≥ 53`, uncommitted |
| `michigan_canada_e2e` | headless runner, 520 ticks | CONFIRMED — fires 9×, ticks 52…468, years 2011–2019 |

Only `michigan_canada_e2e` matches the issue's literal "tick 52" framing. The withdrawn
county-free blanket-dormancy premise (`tools/regression_scenarios.py:406-451`) applies to
**none** of the six — all six stamp real `county_fips`. Dormancy in this estate is
**wiring-driven, not substrate-driven**.

---

## 2. Per-scenario / per-computation dormancy ledger

### 2.1 Three wiring classes (re-confirmed from the T6 synthesis, independently re-verified against `system/__init__.py`)

**Class A — unwired in EVERY scenario, scenario-invariant.** Seven fields are `None` in
both harnesses: `basket_calculator`, `capital_calculator`, `throughput_calculator`,
`employment_source`, `housing_source`, `income_source`, `hex_grid`. Kill-chain:
`capital_calculator` unwired ⟹ `capital_stock ≡ 0.0` (`system/__init__.py:747-751`) ⟹ the
Feature-023 per-county circulation layer early-returns on `capital_stock <= 0`
(`:1603-1605`) in **every** gated run — the circuit advance, inventory, depreciation fund,
and reproduction schema (`:1631-1732`) never execute anywhere, including `michigan-e2e`
(where `turnover_profile_source` IS wired but sits after the dead early return). The flat
`employment = 100_000.0` default (`:793-797`) equalizes every employment-weighted aggregate
in all six scenarios.

**Class B — harness-dependent.**
- **qa four** (`single_county`/`mitterrand`/`syriza`/`bernie_valve`, all Wayne County
  26163, all in `WAYNE_CALCULATOR_SCENARIOS`): 9–11 fields live (melt, tensor_registry,
  distribution/rent/housing/fictitious/credit calculators, financial_crisis_assessor,
  defines/event_bus/economics_fallbacks). Unwired: `reserve_army_data_source`,
  `dispossession_data_source`, `transition_engine`, `turnover_profile_source`, and all
  5 Spec-057 Leontief fields — so Vol I wage pressure (§2.7), Vol II circulation (§2.10),
  Feature-016 transitions (§2.13), and the imperial-rent Leontief pipeline (§2.9) are
  dormant in all four. **Re-verified for `single_county` specifically** (its own memo,
  `reports/t6-dormancy/single_county.md:180-197`): the accumulation loop (§2.8) IS
  *invoked* — `tensor_registry` is wired, so the early-return guard at
  `accumulation.py:1287` (`if tensor_registry is None and dispossession_source is None:
  return`) does not fire — but it *yields no dynamics* for Wayne 2011 (no year-over-year
  OCC delta clears the `delta_occ > 0.0` gate at `accumulation.py:113`, and
  `bankruptcy_rate=None` since `dispossession_data_source` is unwired). Net: invoked but
  inert, not skipped — a distinct dormancy shape from "never called."
- **runner two** (`detroit_tri_county` counterfactual / `michigan_canada_e2e` live): the
  full 33-field estate is wired except Class A. **Re-verified for `michigan_canada_e2e`**
  (`reports/t6-dormancy/michigan_canada_e2e.md:44-45,125,140`): `reserve_army_data_source`
  AND `dispossession_data_source` are BOTH wired here (factory `vol1`, `factory.py:813-818`)
  — this is the only scenario where the Vol I wage-pressure sigmoid (`math.exp`, §2.7)
  actually executes, and where the accumulation-loop `round()` half-even hazard (§2.8)
  actually fires, every boundary (9× across the run).

**Class C — wired but never read by TickDynamics in ANY scenario.** `interest_calculator`
(calibration-only since U9), `credit_cycle_detector` (the `credit_cycle_phase: "expansion"`
graph attr is a hardcoded literal, `graph_bridge.py:106` — never produced by this system),
`counter_tendency_calculator`, `value_basis_converter`; `z1_source`/`housing_data_source`
are consumed one level down inside calculators, never at this system's own boundary.
`ThresholdCrisisDetector` (constructed `system/__init__.py:129`) is dead construction —
never called anywhere. **None of these six fields are part of the 33-field surface below**
(§3) — TickDynamics never reads them directly, so they are out of THIS system's
ServicesProtocol boundary even though they exist on the shared protocol.

### 2.2 Live-but-dead-end channels (computed, never consumed — re-confirmed)

- **`flow_wage_accrued ≡ 0.0` everywhere.** `_accrue_flows` reads `tick_employment` with
  default 0.0 (`system/__init__.py:343`); `tick_employment` has no write site repo-wide
  (three independent memos grep-verified this). The counter can never move.
  `flow_phi_accrued` accrues genuinely only in `michigan-e2e`. Both counters are stripped
  by every `WorldState` round-trip (`world_state.py:253-256`).
- **Precarity triple** (U-6/PTER/NILF) is derived every boundary (`precarity.py:44-62`,
  live in all six scenarios per §2.1) and has no consumer anywhere in the codebase.
- **Two profit rates coexist** post-boundary: the stamped `tick_profit_rate` (K=0 ⟹ s/v,
  an exploitation-rate lookalike when `capital_calculator` is unwired — i.e. always, per
  Class A) vs. the tensor-realized rate the financial layer actually used
  (0.0594 for Wayne 2011). Consumers of the stamped attr get a materially different number
  than the one that drove the tick's own financial-layer math.
- **`financial_s_r`/`financial_tightness` are computed zeros, not structural zeros**
  (ρ̄=0.05 < `interest_reserve_reference`=0.08 ⟹ clamped in the qa four). A port must
  reproduce the computation, not hardcode the zero.
- **Bifurcation pseudo-identity:** `bifurcation.py:101-104` compares `node.id == fips` —
  `T001`-style node ids never equal `"26xxx"` FIPS strings, so the Feature-030
  legitimation blend never engages in any scenario; legitimation always falls to the
  agitation-inverse branch.
- **Bifurcation `compute()` itself never fires within one 52-tick horizon** — the sole qa
  boundary has empty `prev` state (`system/__init__.py:2281-2284`). It needs a second
  boundary (tick ≥ 104). Detail in §4 of the reserved-trio memo.

### 2.3 Per-scenario computation table (synthesized from the six T6 memos, verified spot-checks noted)

| Computation | qa four (single_county/mitterrand/syriza/bernie_valve) | detroit_tri_county | michigan_canada_e2e |
|---|---|---|---|
| §2.3 National MELT/gamma | LIVE (real τ; γ from defines-fallback) | never reaches a boundary | LIVE |
| §2.4 County-state carry+defaults | LIVE on documented defaults (Class A unwired) | n/a | LIVE, mostly real (Class A still unwired) |
| §2.5 Precarity | LIVE, over defaults | n/a | LIVE |
| §2.6 Coefficient smoothing | LIVE | n/a | LIVE |
| §2.7 Vol I wage pressure (`math.exp` sigmoid) | DORMANT (`reserve_army_data_source` unwired) | n/a | **LIVE — the only firing scenario** |
| §2.8 Accumulation loop | invoked, **yields no dynamics** (verified: single_county) | n/a | LIVE, `round()` fires every boundary |
| §2.9 Imperial rent (Leontief) | DORMANT (all 5 Spec-057 fields unwired) | n/a | status not independently confirmed this pass — synthesis lists Spec-057 as Class B-unwired for qa four only; runner wiring not re-checked here |
| §2.10 Vol II circulation | DORMANT (`turnover_profile_source` unwired) | n/a | DORMANT everywhere (Class A `capital_calculator` early-return, §2.1) |
| §2.11 Crisis-phase detection | LIVE (real Wayne r=0.0594, NORMAL throughout by read) | n/a | LIVE |
| §2.12 Vol III financial layer | LIVE (the one consistently-live per-county subsystem) | n/a | LIVE |
| §2.13 Feature-016 transitions | DORMANT (`transition_engine` unwired) | n/a | LIVE (83 counties × 9 boundaries) |
| §2.14 Bifurcation risk | constructed, `.compute()` never called (no prev state) | n/a | fires from 2nd boundary, structurally NEUTRAL (crisis stays NORMAL) |
| §2.18 Hex substrate | DORMANT (`hex_grid` in Class A) | n/a | DORMANT (Class A) |

`detroit_tri_county`'s committed artifact never crosses a boundary at all — every row is
"n/a" for it as delivered; a counterfactual ≥53-tick run is needed to populate this
column (§5).

---

## 3. The ServicesProtocol boundary charter — 33 fields, independently re-counted

**The issue's own framing ("~28-field ServicesProtocol boundary") undercounts.** Re-verified
by direct `rg 'services\.[a-zA-Z_]+'` + `getattr(services, "...")` sweep of both files
against `src/babylon/kernel/services.py:34-88`:

- `system/__init__.py` touches **28** distinct fields (27 by direct `services.X` access,
  plus `credit_aggregate_source` accessed via `getattr(services, "credit_aggregate_source",
  None)` at line 1845 — easy to miss on a naive grep, which is likely why prior counts
  landed at "~28" without the +5 below).
- `imperial_rent.py` (delegated from `_compute_imperial_rent`, `system/__init__.py:912-939`)
  touches **5 fields not read anywhere in `system/__init__.py`**: `periphery_labor_source`,
  `final_demand_source`, `industry_county_allocator`, `production_chain_calculator`,
  `bea_industries` (the one field typed precisely — `list[str] | None` — every other
  optional field on the protocol is bare `Any`).
- **Total distinct fields TickDynamics reads across both modules: 33.** This matches the
  T6 synthesis's corrected count exactly and is now independently re-verified from source,
  not just re-cited. `services.py`'s full protocol has 46 optional fields + 6 core = 52;
  TickDynamics's boundary is 33 of those 52 — the other 19 (including the Class-C dead
  reads named in §2.1) are either never touched by this system or touched only inside a
  producer it calls, out of this system's own boundary.

**Every one of the 33 is typed `Any` at the protocol** except `bea_industries`
(`list[str] | None`) and `event_bus` (`EventBus`). The real interface contract — method
name, argument shape, return type — is not declared anywhere as a shared type; it exists
only as a convention between each producer class and the one call site in
`system/__init__.py`/`imperial_rent.py` that invokes it. This IS the dominant blocker named
by the Phase-1 inventory (§6: "no BSL equivalent for the external-service/data-source
boundary") and confirmed unaltered by T6 — a closed BSL rules-as-content algebra has no
construct for "call an injected Python object's method mid-tick and branch on `None`."

### 3.1 The 33 fields, grouped by step structure, with absent-semantics and rounding/encoding notes

| Field | Step(s) | Concrete call (verified) | Wiring class | Absent-semantics |
|---|---|---|---|---|
| `event_bus` | always-on | `.publish(Event(...))`, 7 call sites | always wired | n/a — core protocol field |
| `defines` | always-on | `GameDefines` sub-model reads throughout | always wired | n/a — core protocol field |
| `economics_fallbacks` | always-on | `.record_*` counters on every graceful-degradation path | wired in all 6 | increments a counter, never blocks |
| `melt_calculator` | §2.3 national | `.get_melt(year) -> float \| None` | wired in all 6 | **hard-required** — `None` short-circuits the ENTIRE annual pipeline |
| `basket_calculator` | §2.3 national | `.get_gamma_basket(year)` | **Class A — always None** | falls back to `economy.gamma_basket_default`=0.68 |
| `gamma_calculator` | §2.3 national | `.compute(year)` | **Class A — always None** | falls back to `economy.gamma_iii_default`=0.33 |
| `capital_calculator` | §2.4 county | `.get_K(fips, year)` | **Class A — always None** | `capital_stock ≡ 0.0` — kills §2.10 downstream in every scenario |
| `throughput_calculator` | §2.4 county | `.compute_metrics(fips, year)` | **Class A — always None** | bootstrap `throughput_position=1.0`, `supply_chain_depth=2.0` |
| `unemployment_source` | §2.4 county | `.get_county_unemployment_rate(fips, year)` | wired in all 6 | bootstrap `0.05` |
| `housing_source` | §2.4 county | `.get_county_renter_share` (via `_resolve_renter_share`) | **Class A — always None** | bootstrap `0.0` |
| `wage_source` | §2.4 county | `.get_county_median_hourly_wage(fips, year)` | wired in all 6, bootstrap-only | bootstrap `21.0` |
| `employment_source` | §2.4 county | `.get_county_total_employment(fips, year)` | **Class A — always None** | flat `100_000.0` — equalizes every employment-weighted aggregate, every scenario |
| `cpi_source` | §2.4 county | `.get_cpi_deflator` (via `_resolve_real_wage_deflator`) | wired in all 6 | bootstrap `1.0` |
| `income_source` | §2.4 county | `.get_county_bracket_ratio` (via `_wired_bracket_ratio`) | **Class A — always None** | bootstrap `0.0` |
| `reserve_army_data_source` | §2.7 Vol I | `.get_unemployment_decomposition(fips, year).reserve_ratio` | qa four: None; runner: wired | Vol I wage-pressure sigmoid dormant when None (§2.1) |
| `tensor_registry` | §2.8/§2.10/§2.11/§2.12 (shared) | `.get(fips, year)` → `ValueTensor4x3` computed fields | wired in all 6 | early-return guard is `tensor_registry is None AND dispossession_source is None` (§2.1) — asymmetric with the other 32 fields' simple None-check |
| `dispossession_data_source` | §2.8/§2.13 (shared) | `.get_bankruptcy_rate`/`.get_foreclosure_rate`/`.get_eviction_rate` | qa four: None; runner: wired | falls back to module-level `DEFAULT_FORECLOSURE_RATE=0.006`/`DEFAULT_BANKRUPTCY_RATE=0.006`/`DEFAULT_EVICTION_RATE=0.063` |
| `turnover_profile_source` | §2.10 Vol II | `.get_turnover_profile(fips[:2])` | qa four: None; runner: wired but dead behind Class A early-return | whole circulation layer no-ops (`:1409-1410`) |
| `inventory_data_source` | §2.10 Vol II | `.get_days_inventory_raw/finished`, `.get_national_inventory` | dead behind Class A (§2.1) | — |
| `depreciation_data_source` | §2.10 Vol II | `.get_annual_depreciation`/`.get_gross_investment` | dead behind Class A (§2.1) | — |
| `distribution_calculator` | §2.12 Vol III | `.compute_distribution(...)` | wired in all 6 | live, produces `SurplusValueDistribution` (s=p+i+r+t) |
| `rent_calculator` | §2.12 Vol III | `.compute_rent_extraction(...)` | wired in all 6 (sentinel path) | — |
| `housing_calculator` | §2.12 Vol III | `.decompose_housing_value(...)` | wired in all 6 (sentinel path) | — |
| `financial_crisis_assessor` | §2.12 Vol III | `.assess(...)` | wired in all 6 | — |
| `fictitious_capital_calculator` | §2.12 Vol III | `.compute_fictitious_capital(year)` | wired in all 6 | — |
| `credit_aggregate_source` | §2.12 Vol III | `.get_total_credit(year)` (getattr-guarded, `:1845`) | wired in qa four | — |
| `transition_engine` | §2.13 Feature-016 | `.simulate_transitions(dist, conditions, crisis_phase=...)` | qa four: None; runner: wired | gate `transition_engine is None → return` (`:2366-2367`) — five-share distribution frozen |
| `hex_grid` | §2.18 hex substrate | (aggregation, module not read — §2.18 UNVERIFIED) | **Class A — always None** | no-op guard at `:392-394` |
| `periphery_labor_source` | §2.9 imperial rent | `.get_coefficients(year)` | not re-verified this pass (imperial_rent.py-only field) | all-or-nothing with the other 4 Spec-057 fields |
| `final_demand_source` | §2.9 imperial rent | `.get_final_demand(year)` | same | same |
| `industry_county_allocator` | §2.9 imperial rent | `.allocate(...)` | same | same |
| `production_chain_calculator` | §2.9 imperial rent | `.flow_source`/`.decomposer`/`.calculator` sub-objects | same | same |
| `bea_industries` | §2.9 imperial rent | industry-list alignment validation | same | same — the one precisely-typed field (`list[str] \| None`) |

### 3.2 Rounding — `round()` half-even, precisely re-verified (9 sites, not estimated)

Independently re-counted by `rg '\bround\('` (not trusted from the prior memo alone):

- **7 sites are 6-decimal event-payload presentation**, `round(x, 6)`: `DISPOSSESSION_CASCADE`
  payload — `cumulative_la_decline`, `current_la_share`, `baseline_la_share`
  (`system/__init__.py:1164,1166,1167`); `BIFURCATION_THRESHOLD` payload — `score`,
  `solidarity_density`, `legitimation`, `class_burden_ratio` (`:2336,2338,2339,2340`).
- **2 sites are integer demotions with NO `ndigits` arg**, affecting persisted graph
  STATE, not just an event payload: `mechanization_displacement = round(delta_occ *
  employment * mechanization_displacement_rate)` (`reserve_army/accumulation.py:115-117`)
  and `firm_failures = round(bankruptcy_rate * employment * firm_failure_conversion_rate)`
  (`:121-123`) — both feed `reserve_army_stock`/`reserve_ratio`, which
  `ReserveArmySystem` (#5) and `DispossessionEventSystem` (#10) read later the SAME tick.

Python `round()` is round-half-to-even (banker's rounding, IEEE-754 correct); the CURRENT
BSL surface declares `floor` as an intrinsic (`declarations.rs:110`,
`DECLARABLE_INTRINSICS = ["exp", "log", "floor"]`) but **no `round` intrinsic exists**.
`floor(x + 0.5)` is round-half-up, which diverges from round-half-to-even at exact
half-integer ties — a genuine, distinct gap, not auto-portable via `floor` alone. Per T6:
live TODAY only in `michigan_canada_e2e` (the 2 integer-demotion sites, every boundary);
the 7 payload sites are dormant in every canonical scenario (no boundary reaches the
bifurcation-threshold gate or the dispossession-cascade gate anywhere in the qa four or
in `detroit_tri_county`'s committed artifact). Event payloads are byte-compared in
baselines, so this needs a **specified** semantic (a D-row: does the port reproduce
half-to-even exactly, or declare a divergence with a written tolerance derivation per this
repo's own cross-implementation-tolerance standard?) — not an assumed one.

### 3.3 County-identity encoding — ADR198 R7, unexercised by every gated scenario

`resolve_county_identity` (`graph_bridge.py:73-77`) returns `str(county_fips)` everywhere
in the frozen Python path — county dicts are keyed by `str`, `CountyEconomicState.fips` is
a `min_length=5, max_length=5` str field, determinism rides `sorted(fips)` string sort
(`system/__init__.py:1907`, `graph_bridge.py:499`). No int encoding exists in the Python
tick path today. **ADR198 R7 rules the port keys FIPS as int, leading-zero trap
D-recorded** (`ai/decisions/ADR198_program29_substrate_widening_charter.yaml:75-79`):
*"FIPS-keyed carries port as int fields with the leading-zero trap D-recorded; where the
string was really naming a node, key by node identity instead."* Re-verified: every
Michigan FIPS in every canonical gate is `26xxx` (no leading zero) — **the trap has zero
behavioral witness in the gated estate**; a `"06037"`-style county (California) would be
needed to exercise it, and none exists in any committed scenario. R7's "string really
naming a node" clause is already honored in shape in the frozen code: node id `T001` and
county identity `"26163"` are kept separate via the `fips_to_node` map
(`graph_bridge.py:146-164`) — the port's int-FIPS key and the node's own identity are
not the same axis.

---

## 4. Port hazards carried into the packet (not ruled here — evidence only)

1. **`round()` half-even ×9** — §3.2 above.
2. **ADR198 R7 leading-zero trap** — unexercised, §3.3 above.
3. **Carry-forward asymmetry.** `_get_best_tensor_year` carries missing years forward
   (financial layer, `system/__init__.py:1019-1059`) while `_get_organic_composition`
   does not (accumulation layer, `:1336-1362`) — same `tensor_registry`, two different
   absence semantics, both golden-visible in scenarios where either path is live.
4. **Three encodings of "absent."** `None` (unwired service — a Class-A/B fact),
   `NoDataSentinel` (wired, data missing for this fips/year), and a documented default
   (wired, this specific field missing) drive different downstream behavior and different
   golden bytes. A port's Option/Result types must not collapse these three into one.
5. **Year offset.** The runner's first boundary computes year 2011 (tick 0 never runs the
   engine — the 2010 annual frame is hydration-owned); the qa harness's tick-0 boundary
   computes year 2010 directly. Same system, two year bases, both golden-pinned in
   different scenarios.
6. **`tensor_registry`'s asymmetric guard** (§3.1 table) — the ONE field among the 33 whose
   None-check is compound (`tensor_registry is None AND dispossession_source is None`)
   rather than a simple per-field None-check; a port that treats every field's absence
   independently will silently change §2.8's firing condition.

---

## 5. Port-shaped consequences

- **The ServicesProtocol charter is ONE document, not a split**, per the T6 synthesis's
  verdict (§4 there) and independently supported by this re-read: the 33-field surface is
  coherent in shape (grouped by step structure, §3.1 above); what varies across scenarios
  is wiring policy in the qa harness vs. the runner, which is a harness fact, not a
  protocol-shape fact. **The qa-harness dormancy pattern must not be reified as the
  protocol's contract** — most of the 33 fields being `None` in the qa four is a property
  of that harness's calibration overrides, not of what the port needs to serve.
- **Defaults are behavior.** Every documented default in §3.1 (employment 100k, U-3 0.05,
  wage 21.0, deflator 1.0, the five bootstrap class shares 0.01/0.09/0.40/0.35/0.15,
  γ_basket 0.68/γ_III 0.33) is golden-visible in the qa four. The port either reproduces
  them byte-exactly or diverges by declared §6.5 ceremony — it may not "fix" them silently.
- **Computed zeros are pinned as computations**, not constants (`financial_s_r`/
  `financial_tightness`, §2.2) — these belong on the mutation-test target list, not the
  D-record-and-forget list.
- **Dead-ends (§2.1 Class C, §2.2) port verbatim or retire on the WS4 ledger per register
  row 24's ruling — no third option.** This memo does not re-open that ruling.
- **No conformance oracle exists today for ~10 of the 19 computations in the Phase-1
  catalog** (§2.4–§2.11, §2.13, §2.14, §2.16, §2.18 by that report's own count) — a port
  targeting those needs hand-built `.bscn` fixtures; nothing in the canonical estate can be
  harvested for them. `single_county`'s and `michigan_canada_e2e`'s memos are the two
  richest sources for hand-building such fixtures precisely because they are the two
  scenarios where the most computations are actually live (§2.3 table).
- **Open Director residue carried forward from T6 (not part of this charter, flagged for
  awareness):** whether the runner's `employment_source` unwiring is deliberate
  scope-trimming or a data bug — it poisons `reserve_army_signal`'s employment weighting
  in the one scenario (`michigan_canada_e2e`) that otherwise wires almost everything
  (T6 synthesis §5.1). This is separate from the reserved trio in memo 2 and does not need
  to wait for it.
- **What this memo does NOT do, by ADR208 R13's instruction:** it does not transcribe or
  rule on the bifurcation directional score, the five-share ClassDistribution, or
  `dispossession_cascade_milestones` — those are posed precisely in the companion Director
  memo, `reports/register-memos/tickdynamics-reserved-trio-2026-08-17.md`, now that this charter exists to make that
  question precise per R13's own condition.

---

## Sources re-read for this memo

- `reports/t6-tickdynamics-services-charter-2026-08-14.md` (synthesis, full)
- `reports/t6-dormancy/{single_county,michigan_canada_e2e,detroit_tri_county}.md` (targeted sections; the other three — `mitterrand`, `syriza`, `bernie_valve` — re-read via the synthesis's Class-B qa-four summary, which the single_county spot-check independently confirmed matches source)
- `reports/register-memos/rows-21-24.md` (row 21, full)
- `reports/port-inventories/tick-dynamics-port-phase1-inventory-2026-08-12.md` (full, including its own "Adjudication (2026-08-12)" corrections section)
- `ai/decisions/ADR198_program29_substrate_widening_charter.yaml` (R6/R7)
- `ai/decisions/ADR208_docket_sitting_2026_08_17.yaml` (R13)
- `src/babylon/kernel/services.py`, `src/babylon/domain/economics/tick/system/__init__.py`, `src/babylon/domain/economics/tick/system/imperial_rent.py`, `src/babylon/domain/economics/reserve_army/accumulation.py` (direct source re-verification of the 33-field count and the 9 `round()` sites)
