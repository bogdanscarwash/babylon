# spec-114 — Wealth-Share Axis Phase 2: feedback + per-county derived sections

**Program**: 21 Data Constitution (ADR075), Phase 2 of the wealth axis.
**Spec number**: 114. **Status**: SPEC ONLY — owner ruling 3 (2026-07-16)
authorized Phase 2 "in principle, spec-first, sequenced after the data work";
ruling 4 fixed the altitude at PER-COUNTY via principled H3/sheaf downward
projection. Nothing in this spec is implemented.
**Depends on**: Phase 1 shadow (landed, PR #202/#203 — `WealthDistributionSystem`
@21.5); task #16 fills (`fact_census_median_income` + `fact_census_rent_burden`
wiring) for Phase 2b; Program 22 does NOT block this spec.
**Branch (future)**: `feature/114-wealth-axis-phase-2` off `dev`.

## Why

Phase 1 wired the FRED-DFA class-dynamics ODE as an observe-only national
4-bracket vector (`src/babylon/engine/systems/wealth_distribution.py`,
position 21.5). Nothing reads it, so qa:regression stayed byte-identical —
and the axis is inert: wealth distribution neither shapes consciousness nor
responds to crisis. The contrapositive empirical invariant recorded in
ADR075 ("distribution leaves the capitalist band ⟹ a rupture event exists
in history") is vacuous under pure relaxation dynamics — the vector never
leaves the band because nothing pushes it. Phase 2 closes the loop in both
directions and projects the national anchor down to counties as derived
sections, per the owner's altitude ruling.

## Phase 1 state (verified, file:line)

- National vector `(w1..w4)` + velocities on `G.graph["wealth_distribution"]`,
  written only-when-set (`engine/systems/wealth_distribution.py:237-241`;
  round-trip `models/world_state.py:653-654, 957-959`).
- Per-node `wealth_share` bracket projection onto `social_class` nodes
  (`wealth_distribution.py:242-247`; declared field
  `models/entities/social_class.py:312`).
- Euler step of `formulas/class_dynamics.py:calculate_full_dynamics` with
  per-bracket mean `class_consciousness` as extraction resistance
  (`wealth_distribution.py:139-158`) and `dt = 1/ticks_per_quarter`
  (`config/defines/economy_class.py:33`).
- Calibration bands pinned by
  `tests/unit/config/test_wealth_distribution_invariants.py:54-66`
  (WID⋃DFA union bands; bottom-50 historical ceiling 0.04; top-decile
  floor 0.60) — calibration-only, NEVER asserted on live trajectories
  (conditionality ruling in that file's docstring, lines 33-40).

## What ships (functional requirements)

### FR-114-1 — `WealthFeedbackDefines` (new defines category)

New Pydantic sub-model `wealth_feedback` in `src/babylon/config/defines/`
(40th category), regenerated into `data/defines.yaml`. Proposed fields:

| define | default | bounds | role |
|---|---|---|---|
| `enabled` | `False` | bool | master gate; flipped to `True` in the Phase-2a landing commit (the Program-20 narrator flag-off pattern) |
| `agitation_gain` | `0.5` | ge=0, le=5 | bracket immiseration → agitation |
| `subsistence_gain` | `0.5` | ge=0, le=5 | bracket deprivation → survival subsistence pressure |
| `rupture_shock_gain` | `0.01` | ge=0, le=0.1 | RUPTURE gap → velocity impulse magnitude |
| `band_top1_min/max` | `0.27/0.37` | ge=0, le=1 | capitalist band, w1 |
| `band_p90_99_min/max` | `0.33/0.41` | ge=0, le=1 | capitalist band, w2 |
| `band_p50_90_min/max` | `0.26/0.33` | ge=0, le=1 | capitalist band, w3 |
| `band_bottom50_min/max` | `-0.02/0.04` | ge=-0.1, le=1 | capitalist band, w4 |

Band defaults are the exact union bands from
`test_wealth_distribution_invariants.py:54-57`; a sync test asserts the
defines defaults equal the test-file constants (two SoTs pinned, the
`test_constants_sync.py` pattern). Bands live in defines (not a constants
module) so a total-conversion mod modeling a different mode of production
can move them — consistent with "one moddable source of truth".

### FR-114-2 — Forward coupling: wealth axis → consciousness

`ConsciousnessSystem` (`engine/systems/ideology.py:72`, Consequence phase)
gains a third agitation source alongside `wage_change` and
`wage_deterioration` (`ideology.py:211-216`): **bracket immiseration** —
the negative velocity of the node's own bracket share from the PREVIOUS
tick's vector (`G.graph["wealth_distribution"]["velocities"]`, stable
because @21.5 runs after Consciousness; no reordering needed):

```
immiseration_b = max(0.0, -velocities[bracket_of_role(role)])
agitation_increment += wealth_feedback.agitation_gain * immiseration_b
```

Routing through `route_agitation_to_ternary` (`ideology.py:230`) is
untouched — the SOLIDARITY-gated fascism/revolution bifurcation applies to
this energy exactly as to wage-cut energy. That IS the bifurcation
coupling: a falling bracket share radicalizes or fascizes depending on
edge structure, no separate bifurcation hook required.

### FR-114-3 — Forward coupling: wealth axis → survival calculus

`SurvivalSystem` (`engine/systems/survival.py:79-143`) computes
`P(S|A) = Sigmoid(wealth_per_capita − subsistence)`. Phase 2a scales the
node's subsistence threshold by bracket deprivation — distance of the
bracket's share below its equilibrium:

```
deprivation_b = max(0.0, equilibrium[b] - shares[b]) / equilibrium[b]   # [0, 1]
subsistence  *= 1.0 + wealth_feedback.subsistence_gain * deprivation_b
```

Material reading (Aleksandrov): when a bracket's share of social wealth
collapses, the cost of remaining acquiescent rises for its members even at
constant nominal wealth — the relative-immiseration mechanism. `w4`'s
equilibrium is 0.02, so proletarian deprivation saturates quickly; the
labor aristocracy (0.294) needs a real collapse — which is the
Fundamental Theorem's shape.

### FR-114-4 — Reverse coupling: crisis → vector

`ContradictionSystem` fires RUPTURE on principal-gap > threshold AND
rising (`engine/systems/contradiction.py:408`,
`rupture_gap_threshold` at `config/defines/survival.py:145`). Phase 2a
adds one graph-metadata stamp at the fire site:
`G.graph["last_rupture"] = {"tick": t, "gap": gap}` (written only when a
RUPTURE fires — the only-when-set byte-safety rule). On the NEXT tick,
`WealthDistributionSystem._advance` applies a conservation-preserving
velocity impulse before the Euler step:

```
kick = wealth_feedback.rupture_shock_gain * gap
velocities = (v1 - kick, v2, v3, v4 + kick)     # expropriation impulse, Σ impulses = 0
```

then clears the stamp (one shock per rupture). Next-tick application
avoids intra-tick ordering coupling to position 18 and keeps the
`opposition_states` precedent (`ideology.py:119` reads the same way).

### FR-114-5 — Recorder surface + contrapositive replay invariant

`SessionRecorder._persist_extended_state`
(`engine/observers/session_recorder.py:188-201`) persists only
`economy`/`state_finances`/`tick_dynamics` — the wealth vector is
currently NOT in the recorded history. Phase 2a extends
`PostgresRuntimeExtensions.persist_graph_metadata` with an optional
`wealth_distribution` payload (protocol + PostgresRuntime + pg DDL via
`ensure_ddl_applied` — never bare-loop migrations, per ADR074 law).

The invariant then lands as a **replay assertion** (new
`tests/integration/persistence/test_wealth_band_contrapositive.py` +
a `babylon.sentinels` declared-invariant gate in the sim-artifacts lane):

> For every recorded session: if `shares(t)` lies outside the capitalist
> band (any of the four `band_*` intervals) at some tick `t`, then at
> least one event of rupture class (`EventType.RUPTURE`,
> `PERIPHERAL_REVOLT`, or a `CollapseTransition`/endgame event) exists in
> the session's event history at some tick `≤ t`.

Band-exit without a rupture in history = the engine invented a
non-capitalist wealth distribution out of nothing — a determinism-class
bug. The assertion is vacuously green on Phase-1 recordings (relaxation
never exits the band) and becomes load-bearing the moment FR-114-4 can
push the vector out; that is by design and was recorded in ADR075.

### FR-114-6 — Per-county derived sections (Phase 2b, ruling 4)

The national measured vector is the **anchor**; counties get **derived**
within-county bracket distributions, computed at hydration time (not per
tick), by iterative proportional fitting (IPF/raking) — the
maximum-entropy disaggregation consistent with both marginals:

- **Matrix**: `M[c, b]` = wealth mass of bracket `b` in county `c`, over
  the hydrated county scope.
- **Column marginals** (the gluing condition): `Σ_c M[c, b] = W_b × T`
  where `W` is the national vector and `T` total scope wealth — county
  sections MUST aggregate back to the anchor exactly (final iteration
  ends on a column-normalization step).
- **Row marginals**: county wealth proxy `T_c` from observables:
  `fact_census_median_income` (`reference/schema.py:990`) capitalized,
  depressed by `fact_census_rent_burden` (renters hold near-zero net
  worth), with `fact_qcew_annual` county wages as the flow-side check.
  Both census tables are task-#16 FILL rows — **Phase 2b is blocked until
  those fills land.**
- **Seed**: county income-bracket distribution mapped through a monotone
  income→wealth-percentile link (worse than measured wealth data, better
  than uniform; the honest option available).
- **Termination**: fixed iteration cap (`32`) or `L1 < 1e-9`, whichever
  first — statically provable loop bound (Power-of-10 rule 2).
  Deterministic: sorted county order, no RNG.
- **Labeling**: sections carry `provenance: "derived-ipf-v1"` and are
  written to a separate overlay (`county_wealth_sections` graph metadata /
  hydration output), never to any measured field. Every consumer (Living
  Map included) must surface derived-not-measured. This is the owner's
  "never fabricated precision" clause made mechanical.

Sheaf framing: counties form the cover; the IPF output is a section
assignment whose gluing condition is the aggregation identity above;
restriction county→hex uses `bridge_county_h3`
(`engine/hydration/reference.py:102`) — Phase 2b restricts uniformly
within county; hex-level refinement by local observables is Phase 2c.
(The codebase's existing sheaf vocabulary lives in the dialectics core —
`domain/dialectics/core/cylinder.py:89`, `engine/systems/solidarity.py:17`;
the geographic use here is new and should say so in its docstrings.)

## Baseline protocol (the EH/doctrine gate pattern)

Phase 2a moves EVERY qa:regression baseline — that is its purpose. The
gate mirrors how the EpistemicHorizon and Doctrine systems were staged
(shadow first, feedback flipped in a declared commit):

1. Red phase lands with `wealth_feedback.enabled = False` — all baselines
   byte-identical, new unit tests green against synthetic graphs.
2. The landing commit flips `enabled = True`, runs
   `mise run qa:regression-generate` AND `qa:regression-generate-dense`
   (`.mise.toml:724-735`, `tools/regression_test.py`), commits the
   regenerated sampled JSONs + dense CSVs **in the same commit** as the
   flip, and the commit message declares the movement and why
   (CLAUDE.md definition-of-done: "if intentionally, regenerate baselines
   and say so"). `qa:e2e-regression` re-baselines likewise.
3. The five scenarios (`imperial_circuit`, `two_node`, `starvation`,
   `glut`, `fascist_bifurcation`) get eyeballed deltas in the PR body:
   direction of drift per scenario, not just "regenerated".
4. Determinism gate: two seeded runs post-flip, identical tick hashes.

## TDD plan (red first, per phase)

Phase 2a (feedback):
- `test_wealth_feedback_defines.py` — category exists; band defaults ==
  invariant-test constants (sync); `enabled` default False.
- `test_consciousness_wealth_coupling.py` — falling bracket velocity with
  solidarity ⇒ `class_consciousness` rises; without ⇒ `national_identity`
  rises (bifurcation preserved); zero velocity ⇒ byte-identical to
  Phase 1 (enabled=True, no signal ⇒ no drift).
- `test_survival_wealth_coupling.py` — deprivation raises effective
  subsistence ⇒ `p_acquiescence` falls; deprivation=0 ⇒ unchanged.
- `test_rupture_shock.py` — RUPTURE stamp ⇒ next-tick velocity impulse,
  Σ shares still 1, stamp cleared; no stamp ⇒ no impulse.
- `test_wealth_feedback_disabled_bytesafe.py` — enabled=False ⇒ tick
  hash equals Phase-1 hash on the same seed (the flag is truly inert).
- Recorder: `test_persist_wealth_metadata.py` (integration tier —
  PG-requiring, per ADR074 tiering).
- Replay invariant: `test_wealth_band_contrapositive.py` — synthetic
  recorded history with band-exit + rupture passes; band-exit without
  rupture fails (the red case is the contract).

Phase 2b (sections): IPF unit tests — gluing identity to 1e-12;
determinism (permuted input, same output); fixed-bound termination;
provenance label present; blocked-on-#16 marker test
(`requires_reference_db` + skips-if-empty is NOT acceptable — the fill
must land first, then the test asserts non-emptiness through the catalog
sentinel's KEEP law).

## Out of scope

- Program 22 minerals/flows coupling into the wealth axis (own program).
- Enum-level merge of INTERNAL_/PERIPHERY_PROLETARIAT (noted in ADR075
  ruling 2, not executed).
- Hex-level observable refinement (Phase 2c, needs hex observables the
  Data Constitution has not yet filled).
- Any UI beyond the derived-label requirement on existing map surfaces.

## Open questions (owner input wanted before Phase 2a starts)

1. **Rupture shock direction**: FR-114-4 kicks w1→w4 (expropriation). A
   FAILED rupture (repression wins — no mechanic distinguishes this yet)
   arguably kicks the other way. Ship the single expropriation impulse
   first, or gate on a struggle-outcome signal that does not exist yet?
2. **Survival coupling form**: multiplying subsistence (FR-114-3) vs
   subtracting from `wealth_per_capita` — the former was chosen because
   it never manufactures negative wealth, but the sigmoid's steepness
   (`survival.steepness_k`) interacts; calibration sweep needed at
   red→green boundary.
3. **Band constants as defines**: FR-114-1 puts the capitalist band in
   `defines.yaml` for moddability. If the owner prefers empirical
   constants stay un-moddable (like `HOURS_PER_YEAR`), they move to a
   shared constants module and the sync test inverts.
4. **Rupture-class event set** for the contrapositive (FR-114-5):
   proposed {RUPTURE, PERIPHERAL_REVOLT, endgame transitions}. The full
   82-value `EventType` enum should be swept once for other
   rupture-class members before the invariant hardens.
