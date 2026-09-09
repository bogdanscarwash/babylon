# Market Scissors Phase 2 — The Correction (Program 23 feature-complete)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this
> plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Program 23 feature-complete: promote `price_value` out of shadow, wire the
correction as deterministic feedback into the material base (wealth evaporation, reserve-army
influx, wealth-axis shock), add per-county scissors + map lens, MELT-drift gauge + diegetic
ticker, calibration behavioral contracts, and run the promotion ceremony (regenerated baselines
+ authorizing ADR078).

**Owner authorization:** `/goal` 2026-07-17 ("finish the entirety of the thing we've been
building about price-value divergence") + explicit mid-turn message: "I approve of Phase 2 and
any other phases you must do. Make this feature complete." This IS the promotion-ceremony gate
recorded in ADR077 §7 opening.

**Architecture:** Two-stage landing (the EH/Doctrine gate pattern): every mechanism lands
red→green behind `market.feedback_enabled = False` with `shadow=True` untouched — qa:regression
stays byte-identical, proving inertness. Then ONE promotion commit flips `shadow=False` +
`feedback_enabled=True`, regenerates the sampled JSONs + dense CSVs + e2e baselines in the same
commit, and declares per-scenario drift in the PR body.

**Tech stack:** existing — Pydantic frozen models, GameDefines, rustworkx GraphProtocol,
pure formulas, recharts, vitest/pytest.

## Global constraints

- Determinism (Constitution III.7): sorted-id iteration everywhere, zero RNG, no wall clock.
- Honest absence (III.11): no profit-rate observable → base serviceability only; no
  county_fips → no county axis; never fabricate zeros.
- Aleksandrov Test: every construct below names its material relation inline.
- Power-of-10: fixed loop bounds, functions ≤100 lines, explicit types, no bare excepts.
- Machine safety: single-flight test runs (`mise run test:q -- <path>`), never parallel pytest.
- Commit per task via `mise run commit -- "type(scope): msg"`.
- Baseline regeneration happens ONLY in Task 10, never earlier.

---

### Task 1: EventType.MARKET_CORRECTION

**Files:** Modify `src/babylon/models/enums/events.py`, `tests/unit/models/test_enums.py:334-335`.

- [ ] Add `MARKET_CORRECTION = "market_correction"` next to the crisis events, with docstring
  line: "MARKET_CORRECTION: fictitious/real divergence exceeded profit-rate serviceability —
  the scissors snapped (Program 23 Phase 2)".
- [ ] Update count test 82→83, docstring lists the addition.
- [ ] `mise run test:q -- tests/unit/models/test_enums.py` → PASS. Commit
  `feat(models): EventType.MARKET_CORRECTION — the scissors snap (P23 Phase 2)`.

### Task 2: MarketDefines Phase-2 coefficients + MarketState correction fields

**Files:** Modify `src/babylon/config/defines/market.py`, `src/babylon/models/market.py`,
regenerate `src/babylon/data/defines.yaml`; tests
`tests/unit/config/test_market_defines.py` (extend), `tests/unit/formulas/test_market.py`.

New defines (all provenance-tagged descriptions):

| define | default | bounds | material relation |
|---|---|---|---|
| `feedback_enabled` | `False` (flipped Task 10) | bool | master gate, EH/Doctrine pattern |
| `correction_threshold_base` | `0.55` | ge=0, le=2 | log-divergence serviceable at zero profit |
| `correction_profit_slope` | `4.0` | ge=0, le=20 | how much a healthy profit rate can service |
| `correction_severity` | `0.6` | ge=0, le=1 | fraction of fictitious log closed by the snap |
| `correction_price_severity` | `0.3` | ge=0, le=1 | price-log fraction closed (credit tightening) |
| `correction_cooldown_ticks` | `8` | ge=1, le=520 | one snap per crisis, not per tick |
| `evaporation_gain` | `0.15` | ge=0, le=0.5 | overhang → claim-holder wealth destruction |
| `unemployment_gain` | `0.08` | ge=0, le=0.5 | overhang → reserve-army influx |
| `wealth_axis_kick_gain` | `0.02` | ge=0, le=0.1 | overhang → w1 velocity impulse (spec-114 FR-114-4 form) |

MarketState gains `corrections: int = Field(default=0, ge=0)` (cumulative — event-sourced
accumulated state = real model field, per the epochs-wave gotcha) and
`last_correction_tick: int | None = Field(default=None)`.

- [ ] Red: tests for defaults/bounds + MarketState new-field round-trip. Run → FAIL.
- [ ] Implement; `poetry run python tools/generate_defines_config.py`; sync test green.
- [ ] `mise run test:q -- tests/unit/config tests/unit/formulas/test_market.py` → PASS.
  Commit `feat(defines): market correction coefficients + MarketState correction ledger`.

### Task 3: Pure correction laws in formulas/market.py

**Files:** Modify `src/babylon/formulas/market.py`, `src/babylon/formulas/__init__.py`,
`tests/unit/formulas/test_market.py`.

**Produces:**
- `calculate_serviceable_divergence(profit_rate: float | None, *, base: float, slope: float) -> float`
  = `base + slope * max(profit_rate, 0.0)` (None → base; a falling rate of profit shrinks what
  the claims structure can service — Vol. III part 3 meets part 5).
- `calculate_overhang(fictitious_log: float, serviceable: float) -> float` = `max(f - s, 0.0)`.
- `calculate_correction_snap(log_ratio: float, velocity: float, *, severity: float) -> tuple[float, float]`
  = `(log_ratio * (1.0 - severity), min(velocity, 0.0))` — the snap destroys the bubble's
  momentum; residual downward velocity survives (panic overshoot is real).

- [ ] Red: tests — serviceability None/negative/positive profit; overhang clamps at 0; snap
  antisymmetry (negative log snaps toward 0 too), momentum kill (positive v → 0, negative v
  preserved). Run → FAIL, then implement, then PASS.
- [ ] Commit `feat(formulas): correction laws — serviceability, overhang, snap (P23 Phase 2)`.

### Task 4: The correction in MarketScissorsSystem

**Files:** Modify `src/babylon/engine/systems/market_scissors.py`;
test `tests/unit/engine/systems/test_market_system.py` (new `TestCorrection` class).

**Consumes:** Task 3 laws, Task 2 defines. **Produces:** graph stamp
`market_correction_shock = {"tick": int, "overhang": float}` (only-when-fired);
`MARKET_CORRECTION` event; wealth/reserve mutations.

Mechanism (all inside `if defines.feedback_enabled` after `_advance`):
1. `profit_rate = _mean_profit_rate(graph)` — mean territory `tick_profit_rate` over sorted
   active territory nodes carrying the attr; `None` when none do (honest absence).
2. `serviceable = calculate_serviceable_divergence(...)`; `overhang = calculate_overhang(...)`.
3. Fire iff `overhang > 0` AND cooldown elapsed (`last_correction_tick is None or
   tick - last_correction_tick >= cooldown`).
4. On fire: snap both oscillators (severity / price_severity); `corrections += 1`;
   `last_correction_tick = tick`; publish `MARKET_CORRECTION` (payload: overhang, serviceable,
   profit_rate, fictitious_log before/after, price_log before/after).
5. Wealth evaporation — the fictitious claims were counted as wealth; the snap un-counts them.
   For sorted active `social_class` nodes whose role brackets to 0 or 1
   (`bracket_of_role(role) <= 1` — CORE_/COMPRADOR_BOURGEOISIE, PETTY_BOURGEOISIE, the
   claim-holding brackets per the ratified ADR075 fold): `wealth *= (1 - min(evaporation_gain
   * overhang, 1.0))`.
6. Reserve-army influx — crisis unemployment where the wage relation exists: sorted active
   `territory` nodes carrying `median_wage`: `reserve_ratio = min(current + unemployment_gain
   * overhang, 1.0)` (absent current → the shock IS the ratio).
7. Stamp `market_correction_shock` graph attr for the wealth axis (consumed Task 5).

Split into helpers ≤100 lines each: `_mean_profit_rate`, `_maybe_correct`,
`_evaporate_wealth`, `_swell_reserve_army`.

- [ ] Red tests: disabled flag ⇒ byte-identical state + no mutations (drive a euphoric graph);
  enabled + overhang ⇒ snap + event + wealth cut on bourgeois nodes only + reserve bump only on
  median_wage territories + stamp written; cooldown suppresses second snap; no profit-rate attr
  ⇒ base serviceability used; overhang=0 ⇒ nothing. Run → FAIL → implement → PASS.
- [ ] Commit `feat(engine): the correction — snap, evaporation, reserve influx (P23 Phase 2)`.

### Task 5: Wealth-axis shock consumption in WealthDistributionSystem

**Files:** Modify `src/babylon/engine/systems/wealth_distribution.py`;
test `tests/unit/engine/systems/test_wealth_distribution_system.py` (extend).

Next-tick application (the spec-114 FR-114-4 pattern, market-crisis direction): before the
Euler step, if `market_correction_shock` stamp present:
`kick = market.wealth_axis_kick_gain * overhang`; `velocities = (v1 - kick, v2 + kick/3,
v3 + kick/3, v4 + kick/3)` (Σ impulses = 0 — fictitious evaporation deflates the top share,
mechanically raising the rest); clear the stamp (one shock per correction). Material relation:
top-bracket paper wealth vanishes relative to total — shares are relative, conservation holds.

- [ ] Red: stamp ⇒ impulse applied, Σ shares still 1, stamp cleared; no stamp ⇒ Phase-1
  identical. Run → FAIL → implement → PASS.
- [ ] Commit `feat(engine): wealth axis consumes the correction shock (P23↔P21 seam)`.

### Task 6: Promotion wiring (still shadow) — pole measure + coupling edge

**Files:** Modify `src/babylon/domain/dialectics/instances/catalog.py`;
tests `tests/unit/dialectics/test_catalog.py`.

- [ ] `_price_value_poles = _wage_poles` with docstring: labor-power is the ONE commodity
  carrying per-node price (w_paid) AND value (v_produced) accounting, so the per-node
  price⟷value position is read there — the D5 shared-defect precedent (`_imperial_poles`);
  a per-node claims/portfolio signal replaces this when financial data lands.
- [ ] Add `pole_measure=_price_value_poles` to the price_value binding (shadow stays True
  until Task 10).
- [ ] Add `Coupling(source="wage", target="price_value", kind="feeds")` to
  `_DEFAULT_COUPLINGS` — the wage relation's realized (w, v) flow IS the scissors' drive.
- [ ] Tests: coupling survives the builder; pole readings appear for price_value on
  wage_value_id_pairs inputs. Verify registry.read_poles handles shadow bindings (it should —
  check; if shadow excluded from read_poles, note and include, poles are observational).
- [ ] `mise run test:q -- tests/unit/dialectics` → PASS.
  Commit `feat(dialectics): price_value pole measure + wage→price_value coupling`.

### Task 7: Per-county scissors + map lens

**Files:** Modify `src/babylon/engine/systems/market_scissors.py`,
`src/babylon/models/world_state.py`, `web/game/map_contract.py`, the bridge's feature
properties, `src/frontend/src/lib/lens.ts`; tests: market system + world_state round-trip +
map contract test + lens vitest.

- [ ] `_aggregate_wage_value_by_county(graph) -> dict[str, tuple[float, float]]` — same
  selection as national, grouped by `county_fips`, sorted counties; nodes without county
  contribute to national only. Empty → no county axis (honest absence — the 5 qa scenarios
  carry no county_fips, so this is byte-invisible pre-ceremony AND post).
- [ ] Per-county oscillators (observe-only — no county-level correction; the correction is
  national credit, counties are exposure): `G.graph["market_county"] = {fips: MarketState
  dump}`, advanced with the same defines. Loop bound = number of counties (finite hydrated
  set).
- [ ] `WorldState.market_county: dict[str, MarketState] | None = None` — round-trip via
  `_write_optional_axes` + `from_graph` (only-when-set).
- [ ] Map contract: new numeric metric `price_divergence` (territory's county `price_log`,
  null when absent) in `MAP_METRIC_PROPERTIES` + feature properties; frontend
  `SELECTABLE_METRICS` entry + ramp (use an existing diverging ramp — crimson/gold aesthetic).
- [ ] Tests: county aggregation determinism (permuted insertion, same output); round-trip;
  contract mirror test; lens renders. Run scoped → PASS.
- [ ] Commit `feat(map): per-county scissors axis + price_divergence lens (P23 Phase 2)`.

### Task 8: Persistence + cockpit — corrections, MELT gauge, ticker

**Files:** New `src/babylon/persistence/migrations/0034_market_corrections.sql`; modify
`postgres_schema.py`, `postgres_runtime/_legacy.py`, `web/game/engine_bridge.py`,
`src/frontend/src/types/game.ts`, `ScissorsChart.tsx`, new
`src/frontend/src/components/timeseries/MarketTicker.tsx`, test fixtures/handlers.

- [ ] Migration 0034 (guarded DO block, 0032/0033 precedent): `tick_summary` +=
  `market_corrections INT` (cumulative count); base DDL updated same commit; INSERT/UPSERT +
  SELECT extended. Apply to babylon_test via `mise run db:sql` and verify.
- [ ] Bridge: `_build_tick_summary` writes `state.market.corrections`; timeseries payload
  gains `market_corrections: (number | null)[]`.
- [ ] ScissorsChart: vertical `ReferenceLine` at each tick where the cumulative count
  increments (label "correction"); MELT-drift readout above the chart:
  `drift = price_index − 1`; copy: `MELT drift +X.X% — $1 commands X.X% less labor than its
  value basis` (negative ⇒ "more"). Derived purely from `price_index` — no fabricated τ.
- [ ] MarketTicker (diegetic, inside the Scissors tab — the narration panel stays frozen):
  deterministic headline from the latest payload bucket: fictitious_ratio ≥ 1.3 "euphoria",
  rising 1.05–1.3 "rally", correction tick "CRASH", drift ≤ −0.05 "slump", else "steady" —
  fixed copy table, index number = `round(10000 * fictitious_ratio)` styled DOW-like. Empty
  state honest.
- [ ] Extend `fixtures.ts`/`handlers.ts`; vitest for marker derivation + ticker buckets +
  pre-P23 payload defensiveness. `mise run web:check` leg scoped (`npx vitest run` on the two
  files) → PASS.
- [ ] Commit `feat(cockpit): correction markers, MELT drift, diegetic ticker (P23 Phase 2)`.

### Task 9: Calibration behavioral contracts

**Files:** New `tests/unit/formulas/test_market_calibration.py`.

Model-derived contracts only (no un-verifiable empirical literals — FRED/NBER numeric anchors
stay deferred until data files land, recorded in ADR078):

- [ ] Restoration half-life: default defines, zero drive, initial price_log 0.5 → half-life
  within [10, 200] ticks (law of value acts on historical, not instantaneous, time).
- [ ] No limit cycle: zero drive, 520 ticks → |price_log| monotone non-increasing after
  velocity sign settles; terminal < 1e-3.
- [ ] Sustained euphoria fires: constant positive drive s.t. fictitious_log exceeds base
  serviceability → correction fires within 520 ticks (fixed bound); cooldown respected
  (≥ cooldown between consecutive corrections).
- [ ] Balanced growth never fires: constant equal EMA growth (zero relative drive) → zero
  corrections in 520 ticks.
- [ ] Commit `test(market): calibration behavioral contracts (P23 Phase 2)`.

### Task 10: THE PROMOTION CEREMONY (single commit)

**Files:** `catalog.py` (drop `shadow=True`), `market.py` defines (`feedback_enabled=True`),
regenerate `defines.yaml`, regenerate ALL baselines; update the canonical-count tests
(`test_catalog.py` shadow expectations → canonical 6; `test_contradiction_system.py`
TestShadowChannel E2E now expects price_value in `opposition_states`;
`test_market_system.py` flag-default assertions).

- [ ] Flip both; regenerate defines.yaml; fix tests that pin the old defaults (deliberate,
  commented).
- [ ] Check `_contradiction()` ContradictionType mapping for price_value (likely CLASS
  fallback — inspect enum; if an ECONOMIC/CRISIS member exists, map it, else CLASS with
  comment).
- [ ] `mise run qa:regression-generate` && `mise run qa:regression-generate-dense`; re-run
  `mise run qa:regression` → 5/5 green against NEW baselines. Check for an e2e baseline task
  (`rg qa:e2e .mise.toml`) and regenerate likewise if present.
- [ ] Determinism gate: run the two-seeded-runs check (`mise run` determinism task) —
  identical hashes.
- [ ] Record per-scenario drift: diff old→new sampled JSONs, one line per scenario
  (direction + which fields), goes in commit message AND PR body.
- [ ] `mise run check` full gate green.
- [ ] Commit `feat(dialectics)!: PROMOTION — price_value canonical + correction feedback live;
  baselines regenerated (P23 Phase 2, ADR078)` — body carries the drift table.

### Task 11: Governance + PR

**Files:** New `ai/decisions/ADR078_market_correction.yaml`; modify `ai/decisions/index.yaml`
(1.26.0), `specs/115-market-scissors/spec.md` (Phase 2 → EXECUTED, invariants updated),
`ai/state.yaml`, auto-memory `market-scissors-program-23.md`.

- [ ] ADR078: full 7-key schema — context (owner authorization quotes), decision (promotion +
  feedback + county axis + exposure), consequences (baseline movement declared per scenario;
  deferred-with-reason: FRED numeric anchors, per-node claims pole measure, county-level
  corrections, hex refinement), evidence paths.
- [ ] spec-115: Phase 2 section rewritten as-executed; invariant list updated ("qa:regression
  byte-identical while the feedback path stays unbuilt" → replaced by the contrapositive-style
  correction invariants).
- [ ] Update memory file + MEMORY.md hook line.
- [ ] Push branch; update PR #205 title/body (Program 23 COMPLETE: Phase 1 + Phase 2, drift
  table, review guide). Do NOT merge.
- [ ] Final: `mise run check` + `mise run qa:regression` + `mise run web:check` all green,
  reported honestly.
