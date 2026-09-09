# Spec 115 — Market Scissors (Program 23)

**Status:** Phase 1 EXECUTED 2026-07-17 overnight (ADR077); Phase 2 EXECUTED 2026-07-17
morning (ADR078, owner approval: "I approve of Phase 2 and any other phases you must do.
Make this feature complete") · **Branch:** `feature/market-scissors-opposition` (PR #205)

## Thesis

Don't simulate a market — simulate the **divergence between price and value**. The market is
the phenomenal form; the c/v/s value substrate is what's real; the gap between them is the
contradiction. Marx's crisis theory as mechanism: prices can wander, but validation must
eventually come from actually-produced surplus value, and the correction is the deterministic
snap-back of an opened scissors — never an RNG event.

The design maps term-for-term onto the constitutional dialectic `D = (A, Ā, w, T, σ)`:

| Dialectic component | Market instance |
|---|---|
| `A` (form pole) | Price — dollars, the phenomenal form |
| `Ā` (substance pole) | Value — socially necessary labor (the w_paid/v_produced flow; c/v/s substrate at depth) |
| translation | MELT — the unit of the money⇄labor adjunction (`value_form.py`) |
| `T` (tension) | The scissors — `price_log`, plus `fictitious_log` as tension-on-tension |
| resolution | The correction — the violent re-identification of price with value |

Because the poles derive from existing `GapReading`/`OppositionState` shapes, this is a
**recording ADR** (ADR070/ADR073 pattern) — no amendment. Amendment T (sigma_authored) is
untouched and still awaiting BD ratification; this opposition uses two *material* poles.

## What Phase 1 shipped (observe-only, byte-identical)

1. **Generic shadow oppositions** — `BoundOpposition.shadow`, registry exclusion from
   principal scoring, `shadow_opposition_states` graph attr in `ContradictionSystem`.
   This is the reusable slice of ADR072's blueprint; chauvinism⟷internationalism
   registers on the same mechanism later.
2. **`MarketScissorsSystem` @17.8** (30th system) — national `price_log` and
   `fictitious_log` as damped-driven oscillators (`formulas/market.py`); drive = realized
   value/surplus growth (+ price momentum for speculation); reversion = the law of value;
   coefficients in `GameDefines.market`. Honest absence without a value substrate.
   State: `G.graph["market"]` + `WorldState.market` (absent-axis byte-safety).
3. **`price_value` opposition** (6th binding, `shadow=True`) — the engine derives
   `market_balance = tanh(price_log / scissors_balance_scale)` into `GraphInputs`;
   positive balance = price above value = form pole dominant.
4. **Exposure** — `tick_summary.price_log/fictitious_log` (migration 0033);
   timeseries arrays `value_produced`/`surplus`/`profit_rate` (substance) and
   `price_index`/`fictitious_ratio` (form, exp-mapped); cockpit **"The Scissors"**
   BottomDrawer tab charting both form series against a dashed `y=1` "value" baseline.

## The visual storytelling (vision, partially shipped)

Play the dramatic irony. The surface is a diegetic ticker — DOW-style index, CNBC-cadence
headlines celebrating the rally — while the X-ray lens shows the two-line scissors: dollars
up top, labor-hours underneath, visibly diverging. When it snaps, the correction isn't a
surprise; it's a vindication of having read the material base. The game teaches the law of
value through UI alone.

Shipped: the scissors chart (the X-ray half). Deferred: the diegetic ticker (narration
panel is owner-frozen unwired), the MELT-drift gauge ("$1 = X minutes of socially necessary
labor"), per-sector/county price heat on the map (needs a spatial market axis first).

## Phase 2 — the correction as feedback (EXECUTED, ADR078)

When the fictitious/real divergence exceeds what the rate of profit can service
(`serviceable = correction_threshold_base + correction_profit_slope × profit_rate`, the
profit rate read as the mean territory `tick_profit_rate`, honest-None fallback to the
base), the correction fires — once per `correction_cooldown_ticks`:

1. **The snap**: both oscillators close by their severities (`correction_severity` /
   `correction_price_severity` — credit tightening deflates prices less violently than
   claims); upward momentum dies, downward momentum survives (panic overshoot).
2. **Wealth evaporation**: claim-holder brackets 0/1 (the ADR075 fold — bourgeoisies +
   petty bourgeoisie) lose `min(evaporation_gain × overhang, 1)` of node wealth. Labor's
   wealth is untouched here; the crisis reaches labor through channel 3.
3. **Reserve-army influx**: active territories carrying `median_wage` gain
   `unemployment_gain × overhang` of `reserve_ratio`; the @5 system converts it to wage
   pressure next tick.
4. **Wealth-axis shock**: the `market_correction_shock` stamp, consumed same-tick by
   WealthDistributionSystem @21.5 as a conservation-preserving velocity impulse (w1 −kick,
   others +kick/3 — the spec-114 FR-114-4 form, crisis direction).
5. **The event**: `EventType.MARKET_CORRECTION` with full before/after payload.

**Promotion executed**: `price_value` is canonical (shadow=False) — it competes for
principal contradiction and enters frames/rupture/regime, so crisis-as-principal falls
out of the existing machinery. The generic shadow mechanism remains (empty) as the
ADR072/Amendment T landing surface. Also landed: `pole_measure` (= `_wage_poles`, the D5
shared-defect precedent — labor-power is the one commodity with per-node price AND value
accounting), the `wage feeds price_value` coupling edge, per-county scissors
(`market_county` / `WorldState.market_county`, observe-only — the correction never snaps
county oscillators; credit is one national system) with the `price_divergence` map lens
(signed diverging ramp, honest-null), `tick_summary.market_corrections` (migration 0034)
with correction ReferenceLines + MELT-drift readout + the diegetic `MarketTicker` in the
Scissors tab (fixed headline table, mechanical selection — the narration panel stays
frozen per the standing ruling).

Still deferred (recorded in ADR078): FRED/NBER numeric anchors (blocked on deterministic
data artifacts), per-node claims/portfolio sigma, county-level corrections, hex-level
refinement.

## Invariants this spec pins

- The reversion term is the ONLY closure force under zero drive; the correction is the
  ONLY discontinuous one, and it fires solely on unserviceable overhang + elapsed cooldown
  (`test_market_calibration.py::TestCorrectionDiscipline`).
- Restoration envelope half-life in [4, 100] ticks; the fictitious envelope strictly
  exceeds the price envelope — bubbles outlive price swings
  (`test_bubbles_outlive_price_swings`, the load-bearing gravity asymmetry).
- Boundedness: no defines-reachable parameterization escapes `max_abs_log`; the rail
  zeroes momentum.
- Balanced growth NEVER fires the correction — the trigger is divergence, not size.
- A shadow binding is NEVER principal (the mechanism outlives its now-empty channel).
- Absent substrate ⇒ absent axis ⇒ absent everything downstream (`None`/no-key/chart gap,
  never fabricated zeros or 1.0s); de-positioned territories get honest `None`.
- The wealth-axis shock conserves Σ shares = 1 exactly (Σ impulses = 0).
- Baselines: regenerated once in the ADR078 promotion commit with per-scenario drift
  declared; byte-identical thereafter.
