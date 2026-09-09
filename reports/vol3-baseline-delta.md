# Vol III Money — Baseline Delta

**Status:** APPROVED 2026-07-19 (Option B — baselines kept; see "Owner Approval Gate" and "Post-approval record")
**Design:** `docs/superpowers/specs/2026-07-18-vol3-money-scissors-design.md`
**Precedent:** ADR077/ADR078 (Program 23 Market Scissors) — the same
"prove the delta in writing, then regenerate baselines in a dedicated
ceremony commit" pattern (D3 owner ruling here mirrors that precedent
explicitly).
**Raw evidence:** `reports/vol3-baseline-delta-raw-diff.txt` (qa:regression),
`reports/vol3-e2e-regression-raw-diff.txt` (qa:e2e-regression) — both
captured verbatim, real output from this branch, not paraphrased.

> **This report was rewritten after the final whole-branch review.** The prior
> draft's headline ("no behavioral delta … not a sign the layer is inert") was
> **false for the interest sub-layer**: the U9 endogenous rate was structurally
> inert (`i ≡ 0.0` on all real data) and the report omitted the live SC-001
> identity test that exposed it. That inertness has since been **repaired**
> (commits below), the interest layer now produces real non-zero values on real
> data, and this document reports the honest post-repair state.

## Headline finding (read this first)

**The Vol III interest layer is now LIVE on real data, and the frozen gate
scenarios remain byte-identical.** Both statements are true at once, and the
reconciliation is the whole point:

1. **`qa:regression` is byte-identical: 5 of 5 scenarios PASS, exit 0.**
   `qa:e2e-regression` (detroit-tri-county, 5 ticks) PASSES unchanged. There is
   **no behavioral delta** on any of these six gate runs — because they are all
   county-free or sub-annual (mechanism per scenario below), so the now-live
   Vol III interest layer honestly resolves to absence there.
2. **On real county data the interest term is non-zero and materially sound.**
   The live SC-001 identity test
   (`tests/integration/economics/test_vol3_surplus_distribution_live.py`) is now
   **GREEN** (it was RED). On Wayne 26163/2011 the layer produces:

   | quantity | value |
   |---|---|
   | economy-wide rate of profit `r` (`profit_rate_ceiling`) | `0.059676` |
   | reserve-army signal `s_r` = tightness `τ` | `0.050320` |
   | endogenous interest rate `i` | `0.019855` |
   | fragility premium (`i − r·base`) | `0.001952` |
   | county `interest_payments` | `1.178853e9` (**33.3% of surplus**) |
   | `profit_of_enterprise` residual | `1.98e9` (**positive** — not a debt spiral) |

   `i/s = 0.333 ≈ base share`, `profit_of_enterprise` stays positive, and all
   four claim terms (surplus, interest, rent, taxes) are strictly positive — so
   SC-001's identity `s = p + i + r + t` now holds **substantively**, not
   vacuously.

The previous inertness was a graph-timing bug: `_compute_national_financial_state`
read the profit rate and reserve signal off `tick_profit_rate` /
`tick_capital_stock` / `reserve_ratio` graph attrs that `state.to_graph()` strips
at the top of every tick and `write_tick_state_to_graph` re-stamps only *after*
this layer runs. `r` was therefore `None` every tick → `i = 0.0` every county-year.
The repair sources `r` and `s_r` from the tick's own `county_states` in scope
(realized surplus/profit-rate tensors for `r = Σs/Σ(c+v)`; employment-weighted
U-3 for `s_r`), eliminating the timing dependency entirely.

Two source-selection disclosures (adversarial-verification pass, 2026-07-19):

- **Why tensors, not the county MELT quantities, for `r`:** on real Wayne
  26163/2011 the MELT path carries `capital_stock = 0` and the pre-existing
  `employment = 100000.0` placeholder, so `Σs/Σ(c+v)` computed from MELT
  collapses to the rate of *exploitation* (~3.79 → a 114% interest rate). The
  realized reference tensors carry the actual profit rate (0.0597); `r` must
  source from them until the capital-stock data gap is repaired.
- **The employment-100k placeholder still touches this identity, though not
  the interest headline:** `interest_payments` reads no employment (verified:
  `distribution/calculator.py:160`), and in the single-county Wayne proof the
  `s_r` employment *weight* is inert (real U-3 = 0.126 drives it). But the
  `ground_rent` (2.01e8) and `taxes_on_surplus` (1.81e8) claim terms in
  SC-001's identity ARE placeholder-scaled via `county_share = 100000/155e6`.
  SC-001 asserts positivity and closure, which hold regardless — but the
  magnitudes of those two terms should not be read as calibrated until the
  employment placeholder (Program 17 honesty gap) is closed.

## Why the frozen gate scenarios still show no delta

Constitution III.7's falsifiability gate (`qa:regression`) compares against the
**pre-Vol-III frozen baselines**. It stays byte-identical (exit 0) because:

1. **The five `qa:regression` scenarios carry zero `county_fips`**
   (`tools/regression_test.py:159`). Every Vol III term is county-keyed or fed by
   county surplus distributions; with no counties, the now-live interest layer
   computes `r = None → i = 0.0` (honest absence, III.11) and drops out of the
   tick. This is the SAME resolution the layer reaches when there is genuinely no
   county data — the difference from the pre-repair state is that it is now
   *reachable* on real data, proven by SC-001 above.
2. **The `qa:e2e-regression` run is only 5 ticks long** and the summary contract
   it pins (counties_alive, population, total_v within ±1.0%) does not serialize
   the interest term; the interest burden reaches material state only through the
   scissors correction, which does not fire in this short window.

The only thing a baseline regeneration *could* encode is the advisory
`defines_hash` (new `MarketDefines`/`capital_vol3` coefficients from U4/U6/U9
altered `GameDefines.model_dump_json`). `compare_baselines` classifies a
`defines_hash` change as a `WARNING` and excludes it from the pass filter
(`passed = len([d for d in diffs if not d.startswith("WARNING")]) == 0`,
`tools/regression_test.py:944`), and the WARNING is printed only on a FAILing
scenario — which is why the captured PASS output shows no WARNING line. **The
behavioral baseline is byte-identical; no tick value, outcome, or dense trace
moves. Baseline regeneration, if run at all, only refreshes that advisory hash.**

## Verification evidence

| Check | Command | Result | Evidence |
|---|---|---|---|
| Live SC-001 surplus-distribution identity (was RED, now GREEN) | `mise run test:q -- tests/integration/economics/test_vol3_surplus_distribution_live.py` | **3 passed** — `interest_payments=1.179e9` (was `0.0`) | see live values above |
| qa:regression (checkpoint + dense) | `mise run qa:regression` / `tools/capture_qa_diff.py` | **5 passed, 0 failed, exit 0** (byte-identical) | `reports/vol3-baseline-delta-raw-diff.txt` |
| qa:e2e-regression (detroit-tri-county, 5 ticks) | `mise run qa:e2e-regression` | **PASS** — counties_alive==3, total_v Δ=0.000%, exit=0 | `reports/vol3-e2e-regression-raw-diff.txt` |
| Full DoD gate (lint+format+typecheck+test:unit) | `mise run check` | **green** | see repair-commit list below |
| Import layering | `mise run lint:imports` | **6 kept, 0 broken** | domain must not import engine — held |

## SC-001: the RED that the prior report omitted (now GREEN)

The prior report's central thesis was falsified by exactly one test it never ran.
Pre-repair, on Wayne 26163/2011:

```
26163/2011: interest_payments is 0.0 — a distributed term is dark or zero, so
SC-001's identity is being satisfied vacuously rather than by a real decomposition
```

Post-repair the same run yields `interest_payments = 1.178853e9` and the test is
green. This is the load-bearing proof that the layer is live on real data; it is
recorded here so no future reader mistakes "byte-identical on the frozen gate" for
"inert."

## The repair (final-review disposition)

| # | Severity | Finding | Disposition |
|---|---|---|---|
| 1 | Critical | U9 endogenous rate structurally inert (`i ≡ 0.0`) | **Fixed** — `r`/`s_r` sourced in-scope from county tensors; SC-001 green |
| 2 | Important | Duplicate divergent `_capital_weighted_mean` (interest ceiling vs scissors serviceability) | **Fixed** — unified onto one published `profit_rate_ceiling`; scissors delegates |
| 3 | Critical ×2 | Two red-and-stale anti-inertness tests | **Fixed** — both rewritten to pin post-U9 reality; the roundtrip pins a non-zero rate on real data |
| 4 | Minor | Vestigial `NationalFinancialParameters.interest_rate_state` | **Fixed** — field removed; `endogenous_interest` is the sole carrier |
| 5 | — | This report false + citation nit | **Fixed** — rewritten; `:962` → `:944` |
| 6 | Minor | `initializer.py` year-2040 clamp | **Fixed** — ceiling dropped, 2007 floor kept |

## Per-scenario delta

Each section names the mechanism that *would* have produced a delta and why it
instead resolved to honest absence on the frozen scenario. The observed delta on
every field is **none (byte-identical)**.

**Endogenous interest (U9):** the rate is now computed endogenously as
`i = r·share(τ)` from the tick's realized county surplus/profit-rate tensors
(Capital Vol. III Part V), never from FRED. On a county-free scenario there is no
realized surplus, so `r = None → i = 0.0` (honest absence, III.11): the
`interest_payments` that feed the county `SurplusValueDistribution` have no county
distributions to carry them, and `_national_serviceability`
(`market_scissors.py`) returns `None` at its `county_states` guard, so the
scissors interest-burden term drops out.

**Serviceability line unified onto one `r` (final-review #1):** the scissors
`_mean_profit_rate` now reads the SAME published `endogenous_interest.profit_rate_ceiling`
that sets the interest ceiling, not a second, independently-aggregated territory
mean. On the county-free scenarios the published ceiling is `0.0` → treated as
honest absence → base-fallback serviceability — byte-identical to the pre-change
read over an empty territory layer.

**Wealth-weighted asymmetry field (U7.6b):** the reweighting lives in the
dialectical opposition layer, which the regression contract does not serialize —
so it changes no baseline-captured value.

All five frozen scenarios share one mechanism (they carry no `county_fips`), so
the per-scenario tables are identical; each is listed individually so the report
can never silently drop a scenario the gate covers.

### imperial_circuit

**Description:** 4-node default scenario (`tools/regression_test.py` SCENARIOS).

| Field | Before (pre-Vol III) | After (this branch) | Named mechanism |
|---|---|---|---|
| First checkpoint tick with a value delta | none (byte-identical) | none (byte-identical) | no `county_fips` → every Vol III term resolves to `None` |
| `final_outcome` / `ticks_survived` | SURVIVED / 52 | SURVIVED / 52 | unchanged — no financial term reached material state |
| Correction fired differently | no | no | anchor pull + interest-burden both `None` (county-absent) |
| Principal contradiction at terminal tick | (unchanged) | (unchanged) | the four new oppositions registered (catalog 6→10) but receive no county input → never rank |
| First dense-trace divergence | n/a | none (`dense/imperial_circuit.csv` byte-identical) | — |

**Materiality argument:** no `county_fips`, so the surplus-distribution layer sees
no county to distribute over — `ground_rent`, `interest_payments`,
`taxes_on_surplus` all absent, `_national_serviceability` short-circuits to `None`,
and the scissors correction runs with exactly its pre-Vol-III inputs. Honestly
zero; the reason is county coverage, not inert code (proven live by SC-001 above).

### two_node

**Description:** Minimal worker vs owner (`tools/regression_test.py` SCENARIOS).

| Field | Before | After | Named mechanism |
|---|---|---|---|
| First checkpoint tick with a value delta | none (byte-identical) | none (byte-identical) | no `county_fips` → every Vol III term `None` |
| `final_outcome` / `ticks_survived` | SURVIVED / 52 | SURVIVED / 52 | unchanged |
| Correction fired differently | no | no | anchor pull + interest-burden both `None` |
| Principal contradiction at terminal tick | (unchanged) | (unchanged) | four new oppositions registered but unpopulated (no county surplus) |
| First dense-trace divergence | n/a | none (`dense/two_node.csv` byte-identical) | — |

**Materiality argument:** the honest-absence case — `two_node`'s Territory nodes
carry no `county_fips`, so the observed result is exactly no delta.

### starvation

**Description:** Low extraction efficiency stress (`tools/regression_test.py` SCENARIOS).

| Field | Before | After | Named mechanism |
|---|---|---|---|
| First checkpoint tick with a value delta | none (byte-identical) | none (byte-identical) | no `county_fips` → Vol III terms `None` |
| `final_outcome` / `ticks_survived` | SURVIVED / 52 | SURVIVED / 52 | unchanged |
| Correction fired differently | no | no | interest-burden term `None` via `_national_serviceability` |
| Principal contradiction at terminal tick | (unchanged) | (unchanged) | financial oppositions unpopulated |
| First dense-trace divergence | n/a | none (`dense/starvation.csv` byte-identical) | — |

**Materiality argument:** low extraction stresses the Vol I production/survival
path, which the Vol III layer does not touch without county surplus data.

### glut

**Description:** High extraction with metabolic overshoot (`tools/regression_test.py` SCENARIOS).

| Field | Before | After | Named mechanism |
|---|---|---|---|
| First checkpoint tick with a value delta | none (byte-identical) | none (byte-identical) | no `county_fips` → Vol III terms `None` |
| `final_outcome` / `ticks_survived` | SURVIVED / 52 | SURVIVED / 52 | unchanged |
| Correction fired differently | no | no | anchor pull `None`; scissors runs on pre-Vol-III `price_value` inputs |
| Principal contradiction at terminal tick | (unchanged) | (unchanged) | financial oppositions unpopulated |
| First dense-trace divergence | n/a | none (`dense/glut.csv` byte-identical) | — |

**Materiality argument:** the metabolic-rift path (`ΔB = R − E·η`) is orthogonal to
the Vol III financial layer; even the live price⟷value scissors reads only
`price_value`, and the county interest/debt terms are county-absent.

### fascist_bifurcation

**Description:** Consciousness routing to national identity (`tools/regression_test.py` SCENARIOS).

| Field | Before | After | Named mechanism |
|---|---|---|---|
| First checkpoint tick with a value delta | none (byte-identical) | none (byte-identical) | no `county_fips` → Vol III terms `None` |
| `final_outcome` / `ticks_survived` | SURVIVED / 52 | SURVIVED / 52 | unchanged |
| Correction fired differently | no | no | interest-burden + anchor pull both `None` |
| Principal contradiction at terminal tick | (unchanged) | (unchanged) | `financial`/`debt_spiral` never populated to displace the prior principal |
| First dense-trace divergence | n/a | none (`dense/fascist_bifurcation.csv` byte-identical) | — |

**Materiality argument:** bifurcation routes agitation by SOLIDARITY-edge presence
— a consciousness-layer mechanism the Vol III money layer only feeds through the
(absent) financial oppositions; with no county surplus the routing is unchanged.

## qa:e2e-regression (detroit-tri-county, 5 ticks)

**PASS, unchanged.** The captured run (`reports/vol3-e2e-regression-raw-diff.txt`)
completes with `counties_alive == 3`, `population liveness: 3/3`,
`total_v: actual=1.497e+09, expected=1.497e+09, Δ=0.000%`, no critical
conservation violations, exit=0. This scenario hydrates real county-keyed
reference series (`hydrate_counties: Loaded 45 tensors`, `fred_rates: 6`), but the
summary contract it pins (counties_alive, population, total_v within ±1.0%) does
not serialize the interest term, and the scissors correction does not fire in the
5-tick window — so the summary matches exactly (Δ=0.000%). County-level interest
liveness is proven separately by SC-001, not by this 5-tick gate.

## Risks realized vs mitigated (design spec §7)

| Risk | Realized? | Evidence |
|---|---|---|
| #1 turning on never-executed code surfaces latent bugs | Partially, then fixed | The dormant code WAS inert (structural `i≡0`); the final review caught it and it is now repaired and proven live (SC-001). |
| #4 catalog growth 6→10 changes principal-contradiction ranking | No | No scenario saw a principal-contradiction change (county-absent → the four new oppositions never populate on the frozen scenarios). |

## Owner Approval Gate

> **STOP. This document is the complete, factual record of the Vol III money
> branch's behavior against the frozen baselines and against real data.**
>
> **What the owner is approving:** merge of the Vol III money-through-the-scissors
> branch, whose flagship U9 endogenous interest layer is now proven LIVE on real
> county data (SC-001 green; Wayne interest = 1.179e9, 33.3% of surplus) AND
> byte-identical against every frozen `qa:regression`/`qa:e2e-regression` scenario
> (they are county-free or sub-annual).
>
> **Baseline regeneration is OPTIONAL, not required.** `qa:regression` exits 0
> against the existing frozen baselines — there is no behavioral value to
> regenerate. The only difference a regeneration would encode is the advisory
> `defines_hash` (a WARNING excluded from the pass filter). If the owner wants the
> stored `defines_hash` refreshed to match the new `MarketDefines`/`capital_vol3`
> coefficients, that is a cosmetic dedicated ceremony commit; if not, the branch
> merges as-is with the gate green.
>
> **What is NOT changing:** no tick value, no outcome, no dense trace. The
> pre-Vol-III behavioral baseline and this branch's behavioral baseline are
> identical on the frozen scenarios.

**Approved by:** Persephone Raskova (Benevolent Dictator / owner)
**Date:** 2026-07-19
**Approval text (verbatim):** "I approve of your plan and we can work that
into your current workflow , go ahead" — given in direct reply to the
controller's Option-B recommendation message, whose explicit ask was: approve
this report to close U8.4, with the variant choice "keep existing baselines"
(no regeneration; the `defines_hash` refresh lands once, in the qa:regression
modernization program's E5 ceremony where the hash gains gating force). The
same reply approved sequencing that modernization program BEFORE the Volume II
circulation program.

## Post-approval record

Regeneration **declined as unnecessary** (owner-approved Option B,
2026-07-19): the gate is byte-identical against the pre-Vol-III frozen
baselines, so `tests/baselines/` is untouched by this program — preserving
the invariant that a baselines commit always records a behavioral change.
The stale advisory `defines_hash` values (pre-existing on all five baselines)
are inherited by the qa:regression modernization program
(`docs/superpowers/specs/2026-07-19-qa-regression-modernization-design.md`,
element E5), which refreshes them in the same ceremony that promotes the
hash to a gating leg. `qa:regression` re-run at sign-off time
(2026-07-19 ~8:50pm): **5 passed, 0 failed — "All regression tests
passed!"**.

## Task 11 Ceremony Addendum (2026-07-20): single_county baseline minted

> **Added by the qa:regression modernization program Task 11 ceremony,
> 2026-07-20.** This section documents the `single_county` scenario baseline
> minted in that ceremony commit (`test(baselines): modernization ceremony —
> county scenarios + financial channels + determinism leg (E2+E3+E5b)`).
> Unlike the five sections above — each diffing an EXISTING frozen scenario
> against its pre-Vol-III baseline and finding no delta, because all five
> carry no `county_fips` — `single_county` (Task 8, E2a) is a brand-new
> scenario added specifically to exercise the Vol III financial layer on a
> real county (Wayne, FIPS 26163). There is no pre-Vol-III baseline for it to
> diff against, so the table below reports its own within-run before/after
> (pre- vs. post-financial-layer-engagement), read directly from the
> just-minted `tests/baselines/single_county.json` and
> `tests/baselines/dense/single_county.csv` — real numbers, not placeholders.

### single_county

**Description:** Wayne-seeded minimal county: Vol III financial layer, MELT
path, and distribution identity all fire (`tools/regression_scenarios.py`
SCENARIOS).

| Field | Before (tick 0, pre-engagement) | After (tick 1..52, live) | Named mechanism |
|---|---|---|---|
| `financial_endogenous_rate` (endogenous interest rate `i`) | `0.0` | `0.017834` (`0.017833969800785755`) | TickDynamicsSystem's annual pipeline (Vol III Part V `endogenous_interest_rate`) fires on the harness's first `step()` call (`context.tick` is the pre-increment `state.tick` = 0, `0 % 52 == 0`) and is carried forward every subsequent tick (SAVE-ONLY semantics, Task 7) |
| `financial_profit_rate_ceiling` (`r`) | `0.0` | `0.059447` (`0.05944656600261918`) | same annual-pipeline computation, sourced from Wayne's realized 2010 surplus/profit-rate tensor |
| `county_26163_interest` (`interest_payments`) | `0.0` | `9.702476e+08` (`970247586.1500003`) | `SurplusValueDistribution.interest_payments` — this is the U9 channel the modernization program exists to keep visible in the gate; real, not fixture-stamped |
| `county_26163_total_s` (surplus `s`) | `0.0` | `3.234159e+09` (`3234158620.500001`) | `SurplusValueDistribution.total_surplus_produced` |
| `county_26163_ground_rent` (`r`, rent term) | `0.0` | `1.998961e+08` (`199896129.03225806`) | `SurplusValueDistribution.ground_rent` |
| `county_26163_taxes` (`t`) | `0.0` | `1.756561e+08` (`175656129.03225806`) | `SurplusValueDistribution.taxes_on_surplus` |
| `county_26163_profit_enterprise` (`p`) | `0.0` | `1.888359e+09` (`1888358776.2854843`) | `SurplusValueDistribution.profit_of_enterprise` |
| Distribution identity `s = p + i + r + t` | vacuous (all 0) | closes exactly: `1888358776.2854843 + 970247586.1500003 + 199896129.03225806 + 175656129.03225806 = 3234158620.500001`, `Δ = 0.0` | same annual pipeline; all four terms strictly positive |
| Interest share of surplus (`i/s`) | n/a | `30.0000%` | `970247586.15 / 3234158620.50` |
| `final_outcome` / `ticks_survived` | — | `SURVIVED` / `52` | scenario runs the full 52-tick horizon, no death event |
| First dense-trace divergence vs this ceremony's minted golden | n/a (NEW golden — nothing to diverge from) | none — `dense/single_county.csv` is this run's own just-committed golden, re-verified byte-identical on regeneration before commit | — |

**Materiality argument:** this is the smallest graph where the Vol III
financial layer fires THROUGH THE PRODUCTION PATH — the scenario this
program exists for (U9's interest-rate inertness shipped invisible because
no gate scenario carried a county; see the headline finding above). All four
distribution terms are strictly positive and the identity closes exactly, so
SC-001's claim now has a live, gate-enforced witness inside `qa:regression`
itself, not only in the ad hoc integration test the headline finding cites.

**Principal contradiction at terminal tick:** exploitation tension between
the Wayne Core Bourgeoisie (C003) and Labor Aristocracy (C004) rises
monotonically over the run (`exploitation_tension`: `0.0` at tick 0 →
`0.571239` at tick 52, sampled checkpoints in
`tests/baselines/single_county.json`) — the live interest burden does not,
at these coefficients, reroute the principal contradiction away from the
wage/production axis.

**Dead-column declarations (E3):** `financial_s_r` and `financial_tightness`
are declared `at_rest` for this scenario in
`tools/regression_scenarios.py`'s `SCENARIO_COVERAGE` (Task 9, verified
genuine — Wayne's real 2010 U-3 = 0.05 is below
`capital_vol3.interest_reserve_reference` = 0.08, so
`reserve_army_signal()` clamps to 0.0 for this county's real calibration,
not because the code path is unreachable); `financial_endogenous_rate` and
`financial_profit_rate_ceiling` are live, per the table above.


### mitterrand

**Named mechanism:** the U13 calibration burst — a seated reform government drains a
24-item social-wage agenda at the boundary tick against Wayne's real fiscal facts
(t_claim ≈ $175.7M, endogenous rate 1.78%). Borrowing on items 1–2, bond discipline
binds at item 3, delivery collapses to the funded floor (the forced austerity turn);
CAPITAL_STRIKE fires per item past the periphery-contracted tolerance; the governance
fork capitulates on fiscal contact; both betrayal integrals cross; the electorate
re-seats the betrayer at the federal clock (ADR140).

**Materiality argument:** every quantity traces to the delivery ledger, the
sovereign fiscal register, and Wayne's hydrator-extracted surplus split — no
scripted outcomes; the arc is the funding identity's own arithmetic.

**Principal contradiction:** unchanged from single_county's terrain — the goldens
route measured surplus, never mint it (L-CEILING).

### syriza

**Named mechanism:** the captured governance road — institutional_pull 0.65 past
governance_capture_threshold with dual-power organs LIVE (a second Michigan claim);
the fork must still capitulate (capture dominates organs, §3.5). Delivery bleeds at
one constant ratio (0.9623, the PASOK trajectory); the betrayal integral crosses
mid-run (tick 4); the atomized base drifts toward the fascist vehicle.

**Materiality argument:** the arm resolves from the same pure laws
`resolve_governance_arm`/`dual_power_live` pin at unit level, over live CLAIMS rows.

**Principal contradiction:** as mitterrand.

### weimar

**Named mechanism:** fascist consolidation THROUGH the ballot — the ruined
Mittelstand (owner wealth 0.35 on a falling trajectory) votes its desperation into a
real FPTP victory at the federal clock (turnout 0.0066, never a coin flip); the win
perturbs the state apparatus's FactionBalance toward SETTLER_POPULIST (ADR136's
write-back); bridgeless disillusion routes worker despair into fascist alignment.

**Materiality argument:** the vehicle wins by allegiance mass × desperation-scaled
turnout; no script, no forced outcome — remove the material squeeze and the machine
seats the duopoly.

**Principal contradiction:** as two_node's terrain.

### debs

**Named mechanism:** the independent ballot line under FPTP — the spoiler shift
lands on the same-pole machine (ELECTION_HELD carries spoiler_target/shift since
U13), the share ceiling binds, and SOLIDARITY-bridged conversion accumulates
regardless of the ballot. Worker repression 0.5 holds the wage relation (a
near-subsistence voter powers hope AND revolt — the Survival Calculus severs the
exploitation edge below that).

**Materiality argument:** the tax is arithmetic over live vote tallies; the
accumulation is the valve's own conversion law.

**Principal contradiction:** as two_node's terrain.

### bernie_valve

**Named mechanism:** the hope valve, both routings, one deterministic run — three
worker classes share the entryist hope machine (committed popular-front viability
1.0, wealth near subsistence); H spikes for all three; the host machine derecognizes
the surge (U12-C); the seated reform's ledger betrays; disillusion windows open
stamped True/False/True by topology, and the SAME operator routes the bridged
workers into organization and the atomized twin into fascist alignment (§2.5).

**Materiality argument:** the twins are byte-identical in every seed except the
SOLIDARITY edge — the divergence is the topology's alone.

**Principal contradiction:** as mitterrand (Wayne substrate).

### org_probe

**Named mechanism:** none — org_probe seeds no county terrain and no WAGES/
EXPLOITATION edges (one SocialClass, no relationships at all), so the Vol III
interest/rent/tax layer never engages: `_tick_dynamics.county_states` is empty
every tick, the same structural absence single_county's own `at_rest` rows
document for a county-free run.

**Materiality argument:** N/A for this branch. org_probe is not a Vol III
scenario — it exists to anchor the Organization estate's `NodeType.ORGANIZATION`
vocabulary in the `qa:regression` byte gate (Organization foundation plan,
Task 12, spec §11), not to exercise the money-scissors distribution identity.

**Principal contradiction:** N/A — no financial layer exercised.
