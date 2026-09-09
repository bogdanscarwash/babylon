# Spec (deferred): wire endogenous class-position into the runtime tick loop

**Status:** ESCALATED / BLOCKED — discovered during the optimization-package program
(2026-07-11). Owner decision **D1** chose the Feature-043 *endogenous* class-shares
mechanism (property/tenure → class), with FRED wealth as a calibration anchor. During
implementation the wiring proved to be a **feature build, not a wire**, and it is
determinism-critical (moves `qa:regression` baselines). Per the Constitution's escalation
clause (rush no determinism-critical work; don't fabricate completion) it is captured here
as a scoped spec rather than forced in.

## Why it's blocked (the discovered fact)

The endogenous mechanism is a pair of complete, tested pure functions —
`evaluate_class_shares(tenure: HexTenureComposition, equity_threshold_met)` and
`check_equity_threshold(state: HexEconomicState, defines)` in
`src/babylon/domain/economics/substrate/transitions.py`. **But their input data is
populated nowhere at runtime:**

- `rg "tenure_composition" src/babylon/engine src/babylon/persistence` → **empty**.
- `HexEconomicState` appears only in `persistence/protocols.py` (a Protocol), never
  hydrated with real per-hex tenure data.
- The two functions have **zero tick-loop callers** (only each other + `wealth_proxy.py`
  + their unit tests).

So the runtime has no `HexTenureComposition` per hex → the endogenous mechanism has nothing
to evaluate. Wiring it requires **building the missing hex-tenure hydration pipeline first**.

## Required work (ordered)

1. **Hex-tenure hydration.** Populate `HexEconomicState.tenure_composition` per hex at
   session init / per tick, from property/land data (TIGER + assessor/tenure sources; the
   substrate/dispossession machinery in `domain/economics/substrate/` + `dispossession/`
   already models tenure state — this connects real data into it). Determinism: use the
   injected-RNG discipline (`resolve_rng`), never global `random`.
2. **Tick-system integration.** Insert (or extend an existing consequence-phase system) a
   call that, per hex, runs `check_equity_threshold(state, defines)` then
   `evaluate_class_shares(tenure, met)` and writes the class shares into the graph/state —
   in the correct materialist-causality position in `_DEFAULT_SYSTEMS`.
3. **FRED wealth as calibration anchor (D1).** Add a concrete `Sqlite` source reading
   `fact_fred_wealth_shares`/`_levels` (720/480 rows, currently orphaned) — used to
   *validate/calibrate* the endogenous shares (national percentile anchor), NOT as a
   per-tick input. Target the canonical **`babylon.reference.schema`** (NOT the stale
   `babylon_data.reference.schema` fork).
4. **Variable hours (HOANBS).** Wire the already-registered `productivity_data_source`
   (`_FredProductivityAdapter`, real BLS HOANBS) into the tick systems that need *empirical*
   labor hours. Keep `HOURS_PER_YEAR=2080` where it is a units constant (calendar
   conversion) — only substitute empirical hours where labor *supply* is meant.
5. **Baselines + verification.** Each wiring moves sim behavior. Regenerate the 5
   `qa:regression` goldens (sparse + dense) + `michigan-e2e.json`, **with owner sign-off**,
   recording the intended behavioral delta (Constitution III.7 drift taxonomy:
   `defines_hash`-stable, behavior-moved → deliberate regen). Use the optimization package's
   sweep harness to *measure* each wiring's outcome delta before committing baselines.

## What WAS done in this program (not blocked)

- The optimization package + the inert-defines fix (sweeps now actually vary the sim).
- The **EventCapture fix** (`e5f9042c`): headless events now carry their real EventType, so
  the Carceral objective works on the headless backend (was pinned at 0.0).
- The orphaned-data **inventory** (`ORPHANED_DATA_INVENTORY-2026-07-11.md`) — the factual
  seed for the work above.
- Dead-seam deletion was **assessed and declined**: the gamma per-country
  `MVP_ERDI_VALUES`/`DefaultGammaImportCalculator` seam and `estimate_la_share` are NOT dead
  (registered in the `protocol_kit` SourceRegistry, exported in public `__all__`, and
  `estimate_la_share` is still called internally by `wealth_proxy.py:486` + mutation tests).
  Deleting them would break the gamma/melt suites for marginal benefit.

## Effort / risk

Feature-sized (hydration pipeline + tick-order integration + baseline regen), determinism-
critical. Should run as its own spec with the owner in the loop on the baseline regen, not
as a tail-end change. The optimization harness built here is the instrument that makes each
wiring's impact measurable before baselines are committed.
