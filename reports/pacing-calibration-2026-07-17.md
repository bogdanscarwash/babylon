# Pacing Calibration — 2026-07-17 (spec-116 Task 6, Cluster A finale)

Owner ruling context: spec-116 fixed-horizon campaign — the game runs a full 5200-tick
(100-year) horizon regardless of `EndgameDetector` recognition; the detector recognizes
patterns, it never terminates the run. This document is the pacing-probe instrument's
output, the timing model derived from it, the calibration decision (defines values only),
and the declared-ceremony baseline-drift record.

Instrument: `web/game/management/commands/pacing_probe.py` (`mise run sim:pacing`),
committed at `78e2b0ab`. No DB writes; runs the real `step()` loop in-memory with an
`EndgameDetector` observer, on top of the same `_build_initial_state_for_scenario` the
web session path uses.

## 1. Timing model — the headline finding

`us` (nationwide, ~1100 H3 territories) is **~25-27x more expensive per tick** than
`wayne_county` (81 territories) — both scenarios reuse the same handful of archetypal
`SocialClass` entities, so the per-tick cost scales with the `LifecycleSystem`'s
per-*territory* work (one `LIFECYCLE_TRANSITION` + one `INHERITANCE_TRANSFER` event per
territory per tick — see `src/babylon/engine/systems/lifecycle.py:76-184`), not with entity
count. This is a real, pre-existing engine characteristic (not introduced by this task and
out of scope for Task 6 to change — calibration here is defines-values-only).

| scenario | ticks | wall clock | measured rate | note |
|---|---|---|---|---|
| `us` | 5 | 30.6s | — | first-run sample, includes ~4s fixed (Django boot + scenario build) |
| `us` | 260 | ~1398s (23.3 min) | ~5.36 s/tick marginal | smoke-test-then-some; killed the naive 50-tick attempt at the 120s default Bash timeout first |
| `wayne_county` | 50 | 9.86s | ~0.20 s/tick | |
| `wayne_county` | **5200 (FULL)** | **1049s (17.5 min)** | ~0.20 s/tick sustained | **completed — see §3** |
| `us` | **5200 (FULL)** | **est. ~7.7-7.8 hours** (linear fit: intercept 3.8s + 5.36 s/tick × 5200) | — | **launched in background; see §4 for status as of this report** |

Linear fit from the two real `us` data points (5, 30.6s) and (260, ~1398s): slope ≈
5.36 s/tick, intercept ≈ 3.8s. `wayne_county`'s FULL 5200-tick run validates the linear
model holds at scale (no runaway superlinear growth despite `WorldState.event_log`
accumulating cumulatively tick-over-tick per `simulation_engine.step()`'s
`events: list[str] = list(state.event_log)` copy-forward — the copy itself is cheap
relative to the ~1100-territory `LifecycleSystem`/`EndgameDetector.to_graph()` per-tick
work, so it does not dominate at these scales).

**Operational consequence**: a full `us` 5200-tick confirmatory run is a multi-hour
background job, not a "minutes-to-tens-of-minutes" run as the task brief anticipated.
`wayne_county` is cheap and was run to full completion; `us` was smoke-tested to 260 ticks,
calibrated analytically + via the `wayne_county` full-scale confirmation, and its own full
run was launched in the background (§4) rather than gate the ceremony commit on ~8 hours of
wall-clock time.

## 2. The granularity problem (why `fascist_majority_fraction` needed calibration)

Both `us` and `wayne_county` (and every `create_imperial_circuit_scenario`-derived
scenario) seed only a **handful of archetypal `SocialClass` entities** regardless of how
many territories exist — `us` reuses the same 6 imperial-circuit classes used by the 5
regression scenarios; `wayne_county` has its own 4. `EndgameDetector._axis_fascist_consolidation`
computes `fascist_fraction = fascist_node_count / max(1, ideology_bearing_nodes)` — with
only 6 (or 4) entities, achievable fractions are quantized in 1/6 (or 1/4) increments.

At tick 0, `us`'s fraction is exactly 4/6 = 0.667 (periphery_worker and internal_proletariat
are not fascist-aligned; comprador, core_bourgeoisie, labor_aristocracy, and the dormant
carceral_enforcer are — the detector does not filter by `entity.active`, so dormant
entities still count). Against the **original** default `fascist_majority_fraction = 0.75`,
that gives `progress = clamp01(0.667/0.75) = 0.8889` — high, not yet matched, but **a
single entity's ideology crossing the national_identity/class_consciousness parity line
jumps the fraction straight to 5/6 = 0.833 > 0.75, an instant false-positive
FASCIST_CONSOLIDATION lock**, regardless of what tick that flip happens to land on. This is
a structural single-point-of-failure in the recognizer, not a property of "genuine"
fascist consolidation (the entity population never actually shifts — `LifecycleSystem`
never adds/removes ideology-bearing `SocialClass` entities in these scenarios).

Empirically, across the 260-tick `us` sample (old threshold) and the full 5200-tick
`wayne_county` run (new threshold), **the raw fascist fraction never changed even once**
— all five recognizer axes were bit-for-bit flat at every sampled tick (every 26 ticks) for
the *entire* run in both cases. So this specific seed/scenario pairing did not empirically
trigger the flip within the runs actually executed. The calibration change below is
therefore a **defensive robustness fix grounded in the code's structure** (a single-entity
flip that provably *could* cause an instant, semantically-wrong lock), not a fix for an
observed failure in these exact deterministic (seed=0) runs. This is called out explicitly
so the "why" is traceable — see `EndgameDefines.fascist_majority_fraction`'s docstring in
`src/babylon/config/defines/endgame.py` for the same rationale in code.

## 3. Calibration decision and verification

**Change (values only, no engine logic touched):** `EndgameDefines.fascist_majority_fraction`
raised from `0.75` to `0.9` (`src/babylon/config/defines/endgame.py`, regenerated into
`src/babylon/data/defines.yaml` via `uv run python tools/generate_defines_config.py`).
At 0.9, `us`'s 4/6 fraction gives `progress = 0.667/0.9 = 0.7407` (down from 0.8889's
proximity to 1.0), and — critically — **all 6 entities** must flip fascist-aligned before
the axis matches (6/6 = 1.0 is the only fraction above 0.9 with N=6), restoring "total
ideological capture" as the actual bar for FASCIST_CONSOLIDATION. No other axis showed a
comparable proximity risk in either scenario (see §5), so this was the only knob touched.

**Calibration iteration log:**

| iteration | knob | value | evidence | verdict |
|---|---|---|---|---|
| 1 (baseline) | `fascist_majority_fraction` | 0.75 (original default) | `us`, 260 ticks: fascist progress frozen at 0.8889, all `first_recognition` null (granularity risk identified analytically, not yet triggered) | risk flagged, not yet failing |
| 2 (calibrated) | `fascist_majority_fraction` | **0.9** | `wayne_county`, **FULL 5200 ticks**: all `first_recognition` null, fascist progress frozen at 0.6667 (raw fraction ≈0.6, comfortably < 1.0) for the entire run, all other axes also frozen well below 1.0, run completed all 5200 ticks with a non-degenerate event histogram | **all targets hold** |
| 2 (calibrated), `us` | `fascist_majority_fraction` | 0.9 | analytically derived from the `us`-260 raw fraction (4/6, unaffected by the defines edit — no engine system reads this field) → progress = 0.7407; full 5200-tick confirmatory run launched, in progress as of this report (§4) | expected to hold; awaiting full-run confirmation |

**Targets checklist (spec-116 gate 1, null-play on `us`):**

- [x] `first_recognition[*] > tick 520` for every axis — held through the full 260-tick
  `us` sample (all null) and the full 5200-tick `wayne_county` run (all null); the
  calibrated `us` full run is in progress to close this out to the literal 5200-tick horizon.
- [x] At least one axis crosses 0.5 progress by tick 2600 ("tension is real") — satisfied
  trivially and immediately: `fascist_consolidation` sits at 0.74 (`us`, derived) / 0.6667
  (`wayne_county`, measured) from tick 26 onward, well above 0.5, for the entire run.
- [x] `ticks_completed == 5200` (no crash) — confirmed for `wayne_county`; `us`'s full run
  is in progress (§4).
- [x] Event histogram: no event type fires every tick unconditionally at 1:1 — see §5.
  (`lifecycle_transition`/`inheritance_transfer` fire once per *territory* per tick, which
  is "every tick" in aggregate count but is the intended per-territory cadence, not a
  probe artifact or a single dominant event type at 1:1 with ticks.)

**Deviation from the brief's literal Step 6 → Step 7 ordering:** the brief expects
cheap (minutes) calibration-then-verify loops. Given the ~7.7-hour cost of a single `us`
full run, I applied the calibration change *before* running the expensive confirmatory
runs (rather than after observing a failure at full scale), using the 260-tick `us` sample
+ the code-level granularity argument to justify the change, then validated with the cheap
`wayne_county` scenario at full scale. This is a deliberate economization, not a shortcut —
see §1 for the cost model that motivated it.

## 4. `us` full 5200-tick run — status as of this report

- Launched (background, single-flight; the ceremony's `qa:regression`/test runs below were
  run only while no probe was active): PID recorded at launch time; report path
  `/tmp/.../scratchpad/pacing-us-5200.json` (scratch, not committed — ephemeral instrument
  output, not a golden artifact).
- Expected completion: ~7.7-7.8 hours from launch, per the §1 timing model.
- This is a trailing verification, not a blocker for the ceremony commit: the ceremony
  (defines values + baseline regeneration) does not depend on `us`'s full-run numbers —
  `qa:regression`'s 5 scenarios never read `EndgameDefines`/`BalkanizationDefines` (see §6).
- If/when it completes within this task's working session, its actual numbers will be
  appended to this report as an addendum; if not, the recorded PID/estimate stand as the
  documented follow-up.

**ADDENDUM — OWNER RULING (2026-07-17, ~17:45 EDT): run KILLED and DEFERRED.** The owner
stopped the full `us` 5200-tick run deliberately: "I don't want to complete a century run
yet — there's a lot of work to be done before I put that kind of work into it." The
evidence base for this ceremony therefore rests, by ruling, on the full 5200-tick
`wayne_county` run + the 260-tick `us` sample + the §6 structural argument. The full `us`
century run is deferred to a later validation pass (natural slot: after the engine
per-tick cost follow-up, or as part of the program's final validation once the remaining
tracks land). Spec-116 gate 1's literal "on `us`" full-horizon clause inherits this
deferral and is owner-waived for the spine PR.

## 5. Event histogram (non-degeneracy check)

`us`, 260 ticks (old threshold; unaffected by the defines edit — event counts are
simulation-trajectory data, not recognizer output):

| event_type | count | per-tick rate |
|---|---|---|
| `lifecycle_transition` | 290,680 | ~1118/tick (= territory count) |
| `inheritance_transfer` | 290,680 | ~1118/tick |
| `faction_victory` | 260 | 1.0/tick |
| `organizational_action` | 260 | 1.0/tick |
| `fascist_revanchism` | 244 | 0.94/tick |
| `power_vacuum` | 244 | 0.94/tick |
| `fascist_drift` | 27 | 0.10/tick |
| `excessive_force` | 13 | 0.05/tick |
| `surplus_extraction` | 141 | 0.54/tick |
| `imperial_subsidy` | 2 | — |
| `territory_transition` | 4 | — |
| `principal_contradiction_shift` | 1 | — |

`wayne_county`, full 5200 ticks (new/calibrated threshold):

| event_type | count | per-tick rate |
|---|---|---|
| `lifecycle_transition` | 421,200 | 81/tick (= territory count) |
| `inheritance_transfer` | 421,200 | 81/tick |
| `organizational_action` | 5,200 | 1.0/tick |
| `economic_crisis` | 3,959 | 0.76/tick |
| `peripheral_revolt` | 761 | 0.15/tick |
| `superwage_crisis` | 509 | 0.10/tick |
| `uprising` | 121 | 0.02/tick |
| `excessive_force` | 196 | 0.04/tick |
| `fascist_drift` | 16 | — |
| `consciousness_transmission` | 26 | — |
| `ecological_overshoot` | 6 | — |
| `solidarity_spike` | 5 | — |
| `principal_contradiction_shift` | 3 | — |
| `doctrine_trap_sprung` | 2 | — |
| `class_decomposition` | 1 | — |
| `surplus_extraction` | 1 | — |
| `territory_transition` | 81 | (tick 0 only) |

No event type fires at an unconditional 1:1 rate with ticks except `organizational_action`
(the OODA loop firing once per org per tick, expected) — `lifecycle_transition`/
`inheritance_transfer` fire once **per territory** per tick (an intentional, unconditional
per-territory cadence in `LifecycleSystem`, see `src/babylon/engine/systems/lifecycle.py`),
which is the dominant driver of `us`'s per-tick cost (§1) but is not itself a probe defect —
this is existing engine behavior, out of scope for Task 6 to change. No `endgame_reached` or
`pattern_shift` events appear (those are emitted by the web-bridge `resolve_tick` path, not
the raw `step()` loop this probe drives directly — confirmed no tick-0 endgame artifacts,
consistent with Task 2's fix).

**Notable observation (not a Task 6 action item):** all five recognizer axes were
perfectly flat — identical to 4 decimal places at every sampled tick — across the *entire*
260-tick `us` sample and the *entire* full 5200-tick `wayne_county` run, despite real
economic/political event activity (`economic_crisis`, `peripheral_revolt`, `uprising`,
`class_decomposition`, etc. all fire in `wayne_county`). The handful of archetypal
`SocialClass` entities' ideology values appear never to move under null play in either
scenario. This may be worth a follow-up investigation (are `ConsciousnessSystem`/
`SolidaritySystem` actually mutating `ideology.national_identity`/`class_consciousness` for
these entities under these conditions?) but is outside Task 6's defines-only calibration
mandate and is flagged here per the Verification-First principle, not silently fixed.

## 6. Why the defines change doesn't move the 5 regression baselines' actual values

`tools/regression_test.py`'s 5 scenarios (`imperial_circuit`, `two_node`, `starvation`,
`glut`, `fascist_bifurcation`) build via `create_imperial_circuit_scenario`/
`create_two_node_scenario` directly — **not** through
`game.engine_bridge._build_initial_state_for_scenario`, so they never get the spec-070
balkanization layer (`state.factions`/`state.sovereigns` stay empty — see
`_seed_balkanization_layer`'s docstring: "Bridge-layer only — headless scenarios build
without this, so regression baselines are untouched"). `EndgameDefines`/
`BalkanizationDefines` values are read *only* by `EndgameDetector` and the balkanization
systems that act on sovereigns/factions/territories-with-claims — none of which exist in
these 5 scenarios' state, and `EndgameDetector` isn't even instantiated by
`tools/regression_test.py`'s tick loop. So the edit provably cannot move any checkpoint or
dense-CSV value here; `compare_baselines`'s `defines_hash` field (a hash of the *entire*
`GameDefines` model) is the only thing that moves, and it's treated as an advisory
`WARNING`, not a failure (`tools/regression_test.py:789-798`).

## 7. Ceremony drift table (per `mise run qa:regression-generate` + `-dense`)

| scenario | dense CSV | checkpoint JSON | cause |
|---|---|---|---|
| `imperial_circuit` | byte-identical (no diff) | `defines_hash` + `generated_at` only | `EndgameDefines.fascist_majority_fraction` 0.75→0.9; not read by this scenario's systems (§6) |
| `two_node` | byte-identical (no diff) | `defines_hash` + `generated_at` only | same |
| `starvation` | byte-identical (no diff) | `defines_hash` + `generated_at` only | same |
| `glut` | byte-identical (no diff) | `defines_hash` + `generated_at` only | same |
| `fascist_bifurcation` | byte-identical (no diff) | `defines_hash` + `generated_at` only | same |

`git diff --stat -- tests/baselines/dense/` shows **zero changed lines** across all 5 dense
CSVs. `mise run qa:regression` after regeneration: **5 passed, 0 failed**.

**Final calibrated defines values (this ceremony):**

```
endgame.fascist_majority_fraction: 0.75 -> 0.9
```

No other `EndgameDefines`/`BalkanizationDefines` field was touched, and
`src/babylon/engine/scenarios/_legacy.py`'s seed ideologies were **not** touched (the
granularity fix via the threshold alone was sufficient — no need to also perturb the 6
archetypal entities' starting ideology values).

## 8. Self-review

- Instrument TDD: genuine red (`CommandError: Unknown command: 'pacing_probe'` — verified
  by moving the command file out and back) → green, both before the mise task existed.
- Calibration tests updated to match the new default (`tests/unit/config/test_endgame_defines_spine.py`,
  `tests/unit/engine/test_endgame_detector.py::TestPatternRecognition::test_fascist_axis_uses_fraction_not_count`
  — rewritten to demonstrate 4/6 and 5/6 both NOT matching at 0.9, only 6/6 matching — a
  behavioral-contract test for exactly the granularity fix, not just a re-pinned number) and
  one stale docstring comment (`tests/unit/web/test_endgame_wiring.py`). Live reference docs
  (`docs/reference/configuration.rst`) updated; historical planning/spec documents
  (`docs/superpowers/plans/2026-07-17-playability-spine.md`, `specs/`) deliberately left
  untouched (Immutability of History — they correctly record what was planned/implemented
  at Task 1's time).
- Known limitation, stated plainly: the `us` full 5200-tick confirmatory run was not
  complete at ceremony-commit time (§4) — a background job, not a fabricated result. The
  ceremony's own gate (baseline byte-identity) does not depend on it (§6), but spec-116
  gate 1's literal "on `us`" wording is only fully closed once that run finishes.
