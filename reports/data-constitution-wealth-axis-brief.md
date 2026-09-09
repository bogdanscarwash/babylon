I have everything needed. Here is the design brief.

---

# Design Brief — Wealth-Share Axis for `social_class` Nodes

**WHY:** `tests/unit/config/test_wealth_distribution_invariants.py:33-40` pins the *calibration* defines `equilibrium_w1..w4`, but explicitly defers the *runtime* contrapositive ("distribution leaves the capitalist band ⟹ a rupture event exists in history") because there is **no wealth-share axis in the tick engine**. `src/babylon/formulas/class_dynamics.py` (the ODE that would drive it) is exported (`formulas/__init__.py:41-46`) but has **zero runtime callers** — confirmed orphaned. This brief specifies the minimal runtime consumer.

## 0. Core finding — two partitions, one conflation

There are **two distinct substrates** and the codebase conflates population with wealth:

- **Population partition (5-class, per-county).** The tick engine (`domain/economics/tick/`) carries `ClassDistribution` on `territory` nodes: fractions **of people** — `0.01/0.09/0.40/0.35/0.15` (bourgeoisie…lumpen). These sum to 1 and are *headcount* shares. **But** `dynamics/types.py:37-41` field docstrings mislabel them `"Top 1% wealth share"` etc. — the label lies; the values are population.
- **Wealth partition (4-class, national).** `class_dynamics.py` tracks `(w1,w2,w3,w4)` = wealth held by top-1% / p90-99 / p50-90 / bottom-50, relaxing toward `equilibrium_w1..w4 = 0.305/0.382/0.294/0.020` (`economy_class.py:33-56`). Nobody runs it.
- **`social_class` nodes** (the 28-system engine substrate) already carry `wealth: Currency` (per-capita, mutated by `economic.py:302-304` extraction) and `population: int` (`social_class.py:308,383`). They have **no distributional wealth-share axis**. `SocialClass.model_config` is `extra="forbid"` (`social_class.py:201-202`) → any new node attribute **must be a declared field** or `from_graph` rejects it (the "real field, never a shadow attr" lesson from the EH Phase-2 work).

## 1. Exact current behavior at each hardcoded site

All four sites hardcode the **population** partition; none is a wealth axis.

1. **`initializer.py:31-36`** — module constants, then used at `:182-186`:
   ```
   _DEFAULT_BOURGEOISIE=0.01  _DEFAULT_PETIT_BOURGEOISIE=0.09
   _DEFAULT_LABOR_ARISTOCRACY=0.40  _DEFAULT_PROLETARIAT=0.35  _DEFAULT_LUMPENPROLETARIAT=0.15
   ```
   Seeds the first-tick `ClassDistribution` when no census data flows in.

2. **`graph_bridge.py:243-247`** — `from_graph`-side reconstruction fallbacks:
   ```
   bourgeoisie_share=dist_dict.get("bourgeoisie", 0.01)  … proletariat…0.35  lumpen…0.15
   ```

3. **`system/__init__.py:416-420`** — identical `dist_dict.get(...)` fallbacks when reading existing `tick_class_distribution`.

4. **`system/__init__.py:732-736`** — literal `ClassDistribution(bourgeoisie_share=0.01, …, lumpenproletariat_share=0.15)` for territories with no prior distribution.

These are the **population** substrate and stay as-is. The wealth axis is **additive**, not a rewrite of these.

## 2. Minimal design — the wealth-share axis

**Axis owner = national 4-vector; per-node read = projection.** The ODE is national (one `(w1..w4)`), so:

- **State home:** round-trip via graph metadata `G.graph["wealth_distribution"]` — the exact pattern `to_graph` already uses for `economy`/`state_finances`/`tick_dynamics` (`world_state.py:630-643`, `session_recorder.py:188-192`). Add a frozen `WealthDistribution` Pydantic model (4 shares + a `tick` stamp) as a `WorldState` field; mutate via `model_copy` (frozen-state gotcha).
- **Per-node read field:** add `wealth_share: Probability = Field(default=0.0, …)` to `SocialClass` (`social_class.py`) — a declared field so it survives `extra="forbid"` round-trip. It is the node's *bracket* share, projected from the national vector by the node's `SocialRole` bracket (§6 mapping).
- **New system — `WealthDistributionSystem`**, Phase-1 **observe-only shadow**, positioned **last in the Consequence phase** mirroring `EpistemicHorizonSystem` (`simulation_engine.py:412`) — it reads the fully-mutated tick and writes **only** its own axis, touching no field the regression checkpoints sample. Add it to `CONSEQUENCE_SYSTEMS` (`simulation_engine.py:442-458`; the partition-drift assertion at `:468` forces classification).
- **Per tick:** seed on first tick from `equilibrium_w*` mapped by role; thereafter relax via `calculate_full_dynamics(shares, velocities, params, second_order, resistances)` (`class_dynamics.py:231`) where `params`/`second_order` come from `GameDefines.class_dynamics` (already wired, `formulas/class_dynamics.py:34`). `resistances` read from each bracket's aggregate `class_consciousness`. Store new `(shares, velocities)` back to metadata; project shares onto nodes.
- **One new define** likely required: `ticks_per_quarter` (the ODE rates are quarterly, `class_dynamics.py:41`) — add to `ClassDynamicsDefines`, regenerate `defines.yaml`. Never hardcode it.

Phase-1 is **decoupled**: `wealth_share` is an overlay; it does **not** feed back into `wealth`/consciousness/bifurcation. That keeps the sampled regression surface byte-identical (§3).

## 3. Determinism / qa:regression impact

- **The frozen contract is the sampled baseline**, not the full dump. `tools/regression_test.py:83-84,528-531` samples per-entity `wealth`/`effective_wealth`/tension — **not** node `model_dump()`. A Phase-1 observe-only system that writes only `wealth_share` (and national metadata) **does not move those checkpoints** → `qa:regression` stays **5/5 green on values**.
- **`defines_hash` WILL change** the moment we add `ticks_per_quarter` (or any define). The comparator emits a **WARNING, not a hard fail** (`regression_test.py:789-791`), but the baselines' `defines_hash` field must be regenerated (`mise run qa:regression-generate --force`) and the move **declared as intentional** per Definition-of-Done. This is the one guaranteed baseline movement in Phase 1.
- **Phase 2 (feedback, owner-gated) DOES move value checkpoints** — once `wealth_share` drives consciousness/bifurcation, `wealth`/tension trajectories change → full baseline regeneration is mandatory and intentional.
- Determinism-proper ("every tick a deterministic hash") is preserved iff the ODE is seeded from defines and iterates in **sorted node order** with **no unseeded RNG** (the StruggleSystem:312 `random.random()` bug is the cautionary precedent). BLAS pin already guarantees FP reduction order.

## 4. How the runtime contrapositive invariant reads the axis

The deferred runtime check (`test_wealth_distribution_invariants.py:33-40`) becomes: **for any tick where `G.graph["wealth_distribution"]` leaves the capitalist band** (`w1∉TOP1_BAND ∨ w4>BOTTOM50_HISTORICAL_CEILING ∨ (w1+w2)<TOP_DECILE_FLOOR`, reusing the bands at `test_…:54-66`), **assert a rupture event exists in `history`** — an `EventType.UPRISING / PERIPHERAL_REVOLT / REVOLUTIONARY_OFFENSIVE` (emitted by `struggle.py:389,704,583`) at or before that tick. Implemented as a **replay assertion over `SessionRecorder` history** (events are persisted per tick, `session_recorder.py:213-216`), not a live per-tick guard — matching the owner ruling that these are contrapositive, never asserted forward on a live trajectory. A distribution that breaks the band **without** a logged rupture is the failure signal.

## 5. TDD plan (red first)

1. **RED — orphan is wired:** assert `WealthDistributionSystem in _DEFAULT_SYSTEMS` and in `CONSEQUENCE_SYSTEMS`; assert `calculate_full_dynamics` has ≥1 runtime caller (guards against re-orphaning).
2. **RED — axis exists & round-trips:** build a graph with `social_class` nodes, `to_graph`→`from_graph`, assert `wealth_share` survives (catches the `extra="forbid"` landmine) and `G.graph["wealth_distribution"]` is preserved.
3. **RED — seeding matches calibration:** first tick, national vector == `equilibrium_w1..w4` under the role mapping; `abs(sum-1.0)<1e-9`.
4. **RED — relaxation is mean-reverting & conserving:** perturb off-equilibrium, run N ticks, assert monotone approach to equilibrium and `Σshares==1` each tick (property law over `calculate_class_dynamics_derivative` constraint, `class_dynamics.py:195`).
5. **RED — determinism:** two runs, identical seed ⇒ identical per-tick `wealth_distribution` bytes.
6. **RED — contrapositive:** a fixture scenario forced out of band produces a rupture event in replay history; an in-band run produces a valid (vacuously-true) check. (Outcome is a *fixture vehicle*, per the 2026-07-16 testing corollary — test the mechanic, never a specific adjudicated outcome.)
7. **GREEN → REFACTOR**, then `mise run qa:regression` (expect only `defines_hash` warning), regenerate, declare.

## 6. Risks + owner-ruling vs mechanical

**Needs an owner ruling:**
- **Bracket mapping (8 → 4).** `SocialRole` has 8 members (`enums/social.py:35-43`); the ODE has 4 wealth classes. Which roles fold into w3 (p50-90) vs w4 (bottom-50)? The test docstring hints w3↔p50-90(=labor aristocracy), w4↔bottom-50(=proletariat+lumpen+internal_proletariat), but `equilibrium_w3` is *labelled* "proletariat" in `economy_class.py:49` — a naming collision that must be resolved authoritatively.
- **Altitude:** national single vector (recommended, matches the ODE's FRED-DFA national fit) vs per-county wealth distributions. Per-county needs county wealth-share data we do not have (Data-Constitution territory).
- **Decoupling vs identity:** is `wealth_share` an independent ODE axis (Phase 1) or eventually *derived* from `wealth×population` with the ODE as calibration attractor? These can diverge; the reconciliation is a modeling decision, and Constitution Aleksandrov-test grounding ("every formal construct traces to a material relation") applies.
- **Phase-2 feedback authorization** (wiring the axis into bifurcation/consciousness) — moves all baselines; gate it like the EH phases.

**Mechanical (no ruling needed):** adding the `wealth_share` field + `WealthDistribution` model; the metadata round-trip (copy the `economy` pattern); registering the system + partition-set entry; adding `ticks_per_quarter` define + `generate_defines_config.py` regen; the red-phase battery; baseline `defines_hash` regeneration.

**Relevant paths:** `src/babylon/formulas/class_dynamics.py`, `src/babylon/config/defines/economy_class.py:33-56`, `src/babylon/data/defines.yaml:411-433`, `src/babylon/models/entities/social_class.py:201,308,383`, `src/babylon/models/world_state.py:630-656`, `src/babylon/engine/simulation_engine.py:378-458`, `src/babylon/engine/observers/session_recorder.py:152-216`, `src/babylon/domain/economics/dynamics/types.py:27-98`, `src/babylon/domain/economics/tick/{initializer.py:31-36,182-186, graph_bridge.py:243-247, system/__init__.py:416-420,732-736}`, `tests/unit/config/test_wealth_distribution_invariants.py`, `tools/regression_test.py:83-84,789-791`.
