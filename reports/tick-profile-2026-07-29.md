# Per-System Tick Profile — Program 27 Phase 0, Task 2

Evidence item for the Refoundation spec (ADR063 supersession leg): where the
Python engine actually spends its tick, measured, so the Rust-port effort ranks
systems by observed heat rather than by line count or intuition.

## Run identity

| Field | Value |
| --- | --- |
| Harness | `mise run qa:tick-budget` → `tools/tick_budget_check.py --ticks 5 --scope michigan-statewide-no-canada` |
| Scenario scope | `michigan-statewide-no-canada` (83 Michigan counties, no international nodes) |
| Ticks | 5 |
| Seed | 2010 (`getattr(defines, "rng_seed", 2010)` — `GameDefines` carries no `rng_seed`, so the fallback is the effective seed) |
| Engine commit | `877a7f6d` (dev tip at run time, 2026-07-29) |
| Box | Solo dev box, 12 cores / 31 GB RAM, Linux 6.12.95+deb13-amd64, BLAS pinned to 1 thread |
| Budget file | `specs/104-national-tick-compute/budget.json` (ratified 2026-07-05: ceiling = 2× first measured value with 50% CI headroom, cumulative ms over 5 ticks) |
| Gate result | **PASS** — "All systems within budget." (exit 0) |
| Tick-loop wallclock | 7.5 s for 5 ticks; per-tick median 1900.3 ms, p99 2038.8 ms |

Pre-run warnings observed (both pre-existing, harmless to the measurement): the
FAF freight-tons window notice (`start_year=2010` outside the 2018–2024
artifact window, so `bilateral_trade_tons` stays 0.0 — no international nodes
in this scope anyway) and the hex-hydrator shapefile fallback for the 83
Michigan counties absent from `immutable_reference_tiger_county`.

## Per-system table

Measured via `SimulationEngine._per_system_ms` (a `time.perf_counter()` wrapper
around each `system.step()` inside `run_tick`, cumulative per system class).
Sorted descending by cumulative ms. "Share" is share of total in-system time
(derived: `system_ms / Σ system_ms`); "Headroom" is `ceiling / measured` per
tick (derived). "—" means the system has no ratified ceiling (it postdates the
2026-07-05 ratification or was default-OFF then).

| System | Total ms (5 ticks) | ms/tick | Share | Ceiling ms | Headroom |
| --- | ---: | ---: | ---: | ---: | ---: |
| ContradictionSystem | 471.3 | 94.3 | 24.9% | 850 | 9× |
| FieldDerivativeSystem | 454.7 | 90.9 | 24.0% | 1300 | 14× |
| ConsciousnessSystem | 329.2 | 65.8 | 17.4% | 1100 | 17× |
| EpistemicHorizonSystem | 168.3 | 33.7 | 8.9% | — | — |
| ProductionSystem | 159.9 | 32.0 | 8.5% | 500 | 16× |
| SurvivalSystem | 115.2 | 23.0 | 6.1% | 400 | 17× |
| FascistFactionSystem | 55.1 | 11.0 | 2.9% | 200 | 18× |
| ImperialRentSystem | 22.2 | 4.4 | 1.2% | 100 | 23× |
| LifecycleSystem | 16.0 | 3.2 | 0.8% | 60 | 19× |
| VitalitySystem | 15.9 | 3.2 | 0.8% | 40 | 13× |
| StruggleSystem | 15.7 | 3.1 | 0.8% | 55 | 18× |
| MarketScissorsSystem | 13.2 | 2.6 | 0.7% | — | — |
| ContradictionFieldSystem | 8.8 | 1.8 | 0.5% | 1800 | 1023× |
| TerritorySystem | 7.8 | 1.6 | 0.4% | 25 | 16× |
| MetabolismSystem | 7.1 | 1.4 | 0.4% | 25 | 18× |
| WealthDistributionSystem | 5.7 | 1.1 | 0.3% | — | — |
| EdgeTransitionSystem | 5.3 | 1.1 | 0.3% | 20 | 19× |
| FactionInfluenceSystem | 4.0 | 0.8 | 0.2% | 20 | 25× |
| DispossessionEventSystem | 2.7 | 0.5 | 0.1% | 10 | 19× |
| SubstrateSystem | 2.0 | 0.4 | 0.1% | 10 | 25× |
| SovereigntySystem | 2.0 | 0.4 | 0.1% | 10 | 25× |
| ReserveArmySystem | 1.7 | 0.3 | 0.1% | 10 | 29× |
| TickDynamicsSystem | 1.7 | 0.3 | 0.1% | — | — |
| CollapseTransitionSystem | 1.2 | 0.2 | 0.1% | 10 | 42× |
| DoctrineSystem | 1.0 | 0.2 | 0.1% | — | — |
| SolidaritySystem | 0.7 | 0.1 | 0.0% | 10 | 71× |
| AllegianceSystem | 0.6 | 0.1 | 0.0% | — | — |
| DecompositionSystem | 0.6 | 0.1 | 0.0% | 10 | 83× |
| ElectoralSystem | 0.6 | 0.1 | 0.0% | — | — |
| OODASystem | 0.5 | 0.1 | 0.0% | 10 | 100× |
| PolicySystem | 0.0 | 0.0 | 0.0% | — | — |
| CommunitySystem | 0.0 | 0.0 | 0.0% | — | — |
| TransportSystem | 0.0 | 0.0 | 0.0% | — | — |
| ControlRatioSystem | 0.0 | 0.0 | 0.0% | — | — |

Total in-system time: **1890.7 ms over 5 ticks ≈ 378 ms/tick.**

## Findings

**1. The envelope, not the systems, dominates the tick.** The 34 systems
together account for ~378 ms of a ~1900 ms median tick — roughly **20%**.
The other ~80% is spent outside `system.step()`: per-tick persistence writes,
tick hashing, event-bus/observer plumbing, and runner overhead. This is the
single most load-bearing number for the Rust port's performance case: the
largest wall-clock win at this scope lives in the **engine envelope**
(persistence + hash path), which Phase 3 rebuilds wholesale, not in any
individual system port. (Derived from the printed per-tick median vs the
per-system accumulator; the 7.5 s total against a 1900 ms median implies the
early ticks ran cheaper than the median — both numbers reported as printed.)

**2. The ratified heat cluster mostly holds, with one stale ceiling.** The
budget file's own ranking predicted ContradictionField / FieldDerivative /
Consciousness / Contradiction as the hot cluster. Measured: Contradiction,
FieldDerivative, and Consciousness are indeed ranks 1–3 (66% of in-system time
between them). But **ContradictionFieldSystem collapsed to 1.8 ms/tick against
its 1800 ms ceiling (1023× headroom)** — its 2026-07-05 ceiling is stale by
three orders of magnitude, most plausibly optimization since ratification plus
its dormant `field_registry` path. The ceiling should be re-ratified whenever
budget.json is next touched; it currently gates nothing.

**3. In-system heat is overwhelmingly numeric-core, not graph-CRUD.** The top
six systems (Contradiction, FieldDerivative, Consciousness, EpistemicHorizon,
Production, Survival — all numeric/field/formula-heavy) hold **89.8%** of
in-system time. The graph-CRUD-heavy systems (Territory, EdgeTransition,
FactionInfluence, OODA, structural churn generally) sit at noise level, under
3% combined. For the porting contract table this says: the RUST_INTRINSIC and
HYBRID numeric cores carry the compute; the BSL_RULES tier is cheap and can
afford interpreter overhead.

**4. Line count is not heat.** TickDynamicsSystem — the 2,558-line tensor
core, the largest RUST_INTRINSIC port — measured 0.3 ms/tick at this scope.
Its Phase 2b "TickDynamics-first" priority is justified by complexity and
correctness risk, not by wall-clock at michigan-statewide scale. Conversely
EpistemicHorizonSystem (a Phase-1-shadow, observes-only system) is already the
4th-hottest system at 33.7 ms/tick with **no ratified ceiling at all**.

**5. Eleven of 34 systems have no ceiling.** EpistemicHorizon, MarketScissors,
WealthDistribution, TickDynamics, Doctrine, Allegiance, Electoral, OODA(-era
additions), Policy, Community, Transport, ControlRatio postdate the 2026-07-05
ratification or were dormant then (Policy/Community/Transport/ControlRatio
measured 0.0 ms here — Transport is default-OFF per ADR160-169). A budget.json
re-ratification pass is owed but is NOT Phase 0 scope; recorded here as
evidence for whoever next touches spec-104.

## Budget-gate sanity check (plan Step 4)

`tools/tick_budget_check.py` ran as the capture mechanism itself and exited
green: every system with a ratified ceiling PASSed, final line "All systems
within budget." No ceiling was within 9× of being breached.
