# Program 27 Phase 0 — Tolerance Derivations, Stochastic-Family Designation, Ensemble Envelopes, Cutover-Ceremony Procedure

Task 13 of `docs/superpowers/plans/2026-07-29-program-27-phase-0-contracts-and-evidence.md`.
Spec anchor: §8.5 (deterministic families tolerance-bounded with written derivations,
stochastic families under ensemble envelopes) and §10 (cutover ceremony design).

Scope: the 11 canon `qa:regression` scenarios (`imperial_circuit`, `two_node`,
`starvation`, `glut`, `fascist_bifurcation`, `single_county`, `bernie_valve`,
`debs`, `mitterrand`, `syriza`, `weimar`) plus the two Article IV acceptance
gates (`michigan-e2e.json`, `detroit-tri-county-5t.json`).

## 1. The mechanical stochastic-family trace (Step 1)

**Tool:** `tools/stochastic_family_trace.py` (stdlib-only: `re` + `pathlib`, no
AST, no third-party deps). It seeds taint from the six RNG-touched systems
named in Task 10's porting-contract table
(`reports/p27-porting-contract-table.md` rows 16–24: `FactionInfluenceSystem`,
`DoctrineSystem`, `StruggleSystem`, `ElectoralSystem`, `FascistFactionSystem`,
and `OODASystem` transitively via `ooda/npc_stub.py` →
`ooda/state_ai/{decision,repress_effects,administer_effects}.py`), extracts
every bracket-string/`.get(...)` graph-attribute name touched in each RNG
system's file, then propagates the taint **one hop** to any other
`engine/systems/*.py` file that mentions one of those attribute names
anywhere, tainting that file's own touched attributes in turn.

Run: `python3 tools/stochastic_family_trace.py` (full table archived at
`/tmp/claude-1000/-home-user-projects-game-babylon/7ab6f51d-aa76-44b6-9a3c-c7b98939ea53/scratchpad/stochastic_trace_output.md`,
not committed — regenerate on demand, it's deterministic given the tree).

**Finding — the mechanical rule over-taints severely, exactly as the plan
anticipated ("conservative, over-taints; say so in the report"), and the
degree is worth naming precisely:** of 212 distinct attribute names touched
across `engine/systems/*.py`, the one-hop rule marks **203 (96%)**
`STOCHASTIC` and only **9** `DETERMINISTIC` (`biocapacity_stock`,
`border_regime`, `c`, `count`, `distribute_phi_week_to_counties`, `k`,
`raw_material_stock`, `s`, `v`).

**Root cause, verified by direct read, not guessed:** this codebase's real
graph-write call is `graph.update_node(node_id, attr=value, ...)` —
**keyword arguments, not a dict literal** (e.g.
`graph.update_node(node.id, wealth=new_wealth)` at `struggle.py:382`). The
tool's `update_node({...})` dict-literal extractor therefore never fires on
the codebase's actual write style; every taint the tool found instead came
from the generic bracket-string/`.get(...)` matcher, which **cannot
distinguish a read from a write**. A struggle-eligibility check like
`attrs.get("wealth", 0.0)` (reading wealth to decide whether to act) taints
`wealth` identically to an actual write of wealth — and the one-hop
propagation then spreads through any co-occurring common attribute name
(`role`, `tick`, `wealth`, `target`) that appears, for unrelated reasons, in
nearly every system's file. This is the accepted, named cost of the Step-1
rule (over-approximation over silent under-approximation, per the plan) —
but it makes the raw tool output **unusable as-is for Step 2's per-family
tolerance table**, which needs a real deterministic/stochastic split, not a
96%-stochastic wash. Section 2 below does the additional, source-verified
work the plan's Step 2 requires.

## 2. Source-verified per-family classification (Step 2 input)

For every column family that actually appears in a baseline file, the
family's owning system was identified by reading the real `update_node(...)`
kwarg call that writes it (not the mechanical bracket-scan), then
cross-checked against Task 10's RNG-usage column.

### 2.1 The 11-canon `checkpoints[]` columns

Columns (from `tests/baselines/imperial_circuit.json`): `tick`,
`p_w_wealth`, `p_c_wealth`, `c_b_wealth`, `c_w_wealth`,
`imperial_rent_pool`, `exploitation_tension`, `p_w_consciousness`,
`p_w_p_revolution`, `p_w_active`. `p_w`/`p_c`/`c_b`/`c_w` are fixed entity
ids for `PERIPHERY_PROLETARIAT` / `COMPRADOR_BOURGEOISIE` /
`CORE_BOURGEOISIE` / `LABOR_ARISTOCRACY` respectively
(`src/babylon/engine/scenarios/_legacy.py:313,328,343,358`).

`StruggleSystem.step()` (`struggle.py:298`) only processes nodes whose role
is `PERIPHERY_PROLETARIAT` or `LUMPENPROLETARIAT` (`struggle.py:286-288`
docstring, `_STRUGGLING_ROLES` guard at `:328`); its RNG draw
(`spark_occurred = rng.random() < spark_probability`, `:341-342`) gates
whether `update_node(node.id, wealth=new_wealth)` (`:382`) and
`update_node(node.id, ideology=new_ideology)` (`:409`) fire for that node
this tick.

| Column family | Classification | Evidence |
|---|---|---|
| `tick` | DETERMINISTIC | engine loop counter, no RNG system touches it |
| `p_w_wealth` | **STOCHASTIC(StruggleSystem)** | RNG-gated `update_node(..., wealth=...)` at `struggle.py:382`; fires only for the `PERIPHERY_PROLETARIAT`/`LUMPENPROLETARIAT` role, which `p_w` is |
| `p_w_consciousness` | **STOCHASTIC(StruggleSystem, one hop)** | derived from the `ideology` field RNG-gated-written at `struggle.py:409,608`; `tools/regression_test.py:938` reads `entity.consciousness`, a projection of `ideology` |
| `p_w_p_revolution` | **STOCHASTIC(StruggleSystem)** | `update_node(p_w_id, p_revolution=1.0, ideology=new_ideology)` at `struggle.py:608`, inside the same RNG-gated uprising block |
| `p_w_active` | DETERMINISTIC in this table's direct sense, but **causally downstream of `p_w_wealth`** | no write to `active` found anywhere in `struggle.py`; `active` is set elsewhere (survival/death threshold, a deterministic sigmoid over wealth) — but since its input (`wealth`) is itself stochastic-tainted, `p_w_active` should be treated as stochastic for tolerance-bounding purposes (a deterministic function of a stochastic input is not tolerance-boundable; flagged, not independently re-traced to its writer in this pass) |
| `p_c_wealth`, `c_b_wealth`, `c_w_wealth` | DETERMINISTIC | none of `COMPRADOR_BOURGEOISIE`/`CORE_BOURGEOISIE`/`LABOR_ARISTOCRACY` is a `_STRUGGLING_ROLES` member, so `StruggleSystem`'s RNG-gated write path never reaches these nodes; no other RNG system (`FactionInfluenceSystem`, `DoctrineSystem`, `ElectoralSystem`, `FascistFactionSystem`) writes a `wealth` kwarg anywhere (confirmed: `reactionary.py` writes only `fascist_alignment`/`aligned_faction_id`; `faction_influence.py` writes only to `persistent_data`, never a graph node, per Task 10's coverage note — this system is a no-op on graph state in every one of the 6 non-electoral canon scenarios today) |
| `imperial_rent_pool` | DETERMINISTIC | `ImperialRentSystem` — Task 10 row 10: "none found directly"; the one gated optional sub-stage (`n.py`/`Vol2CirculationStep`) is default-off on these scenarios (unresolved past one hop by Task 10, flagged for Task 17, not re-litigated here) |
| `exploitation_tension` | DETERMINISTIC | `ContradictionSystem`/`ContradictionFieldSystem`/`FieldDerivativeSystem` — Task 10 rows 28–30, all "none found", all fully-intrinsic numeric cores with no RNG term |

**Scenario-level correction — `single_county` has ZERO RNG exposure.**
`src/babylon/engine/scenarios/single_county.py:72,85` seeds only
`CORE_BOURGEOISIE` and `LABOR_ARISTOCRACY` — neither is a
`_STRUGGLING_ROLES` member, so `StruggleSystem`'s node loop finds nothing
eligible and `rng.random()` is never called. Every column family in
`single_county.json` is therefore fully DETERMINISTIC. This is the cleanest
of the 11 — worth stating explicitly rather than lumping it in with the
other five imperial-circuit-family scenarios.

**The other five non-electoral scenarios** (`two_node`, `imperial_circuit`,
`starvation`, `glut`, `fascist_bifurcation`) all seed a
`PERIPHERY_PROLETARIAT` node (`two_node`: `_legacy.py:83`;
`imperial_circuit`-family: `:313`, reused via `defines_overrides` for
`starvation`/`glut`/`fascist_bifurcation` per `tools/regression_scenarios.py`)
and so all five carry the same `{wealth, ideology→consciousness,
p_revolution, active(-downstream)}` stochastic family on that one role,
with every other column deterministic.

**The 5 electoral goldens** (`bernie_valve`, `debs`, `mitterrand`, `syriza`,
`weimar`) carry the same `StruggleSystem`-gated family on their
worker-role node(s) **plus** the families directly written by
`ElectoralSystem` (`faction_balance` at `electoral.py:1040`,
`legitimation_index` at `:1091`, `internal_balance` at `:1128`),
`DoctrineSystem` (`doctrine_tags`, `acquired_doctrine_ids`,
`congress_tag_snapshot`, `party_id`, etc. — kwarg names constructed into an
`updates` dict at `doctrine.py:585-594` then applied via
`update_node(org_id, **updates)` at `:595`), and `FascistFactionSystem`
(`fascist_alignment` at `reactionary.py:140`, `aligned_faction_id` at
`:165`) — matching the spec's own framing that the electoral goldens are
"the RNG-heaviest" (§8, item 5). This is a genuine, source-grounded
confirmation of that framing, not an assumption.

### 2.2 Michigan/tri-county (`terminal_state` / `county_terminal_snapshot`)

Columns (from `tests/baselines/michigan-e2e.json` and
`detroit-tri-county-5t.json`, identical shape):
`terminal_state`: `tick, counties_alive, counties_with_population,
total_population, total_v, total_c, total_s, total_k,
mean_p_acquiescence, mean_p_revolution, mean_ideology_r, mean_ideology_l,
mean_ideology_f, max_tension`; `county_terminal_snapshot[]`: `entity_id, v,
c, s, k, p_acquiescence, p_revolution, ideology_r, ideology_l, ideology_f,
population, delta_k_vs_initial`.

| Family | Classification | Basis |
|---|---|---|
| `total_v`/`total_c`/`total_s`/`total_k`, per-county `v`/`c`/`s`/`k`, `population`, `delta_k_vs_initial`, `counties_alive`, `counties_with_population` | DETERMINISTIC | Fundamental-Theorem value categories, production/tensor-derived (`ProductionSystem`, `TickDynamicsSystem`, `WealthDistributionSystem`) — none carries direct RNG per Task 10 |
| `mean_p_acquiescence`/`mean_p_revolution`, per-county `p_acquiescence`/`p_revolution`, `mean_ideology_*`/`ideology_*` | **STOCHASTIC(StruggleSystem), by attribute-name correspondence with §2.1's confirmed mechanism** — **not independently re-traced for the county-scoped aggregate model in this pass; flagged for confirmation before the cutover ceremony** | same attribute names (`p_acquiescence`, `p_revolution`, `ideology`) as the RNG-gated write confirmed in §2.1; whether the county-level aggregation node carries the same `_STRUGGLING_ROLES` gate was not re-verified here |
| `max_tension` | DETERMINISTIC | `ContradictionSystem`/`ContradictionFieldSystem` family, no RNG (§2.1) |
| `imperial_rent_phi_week_distribution` (the sole `conservation_audit` invariant present) | DETERMINISTIC, conserved-sum | `ImperialRentSystem`, no RNG (§2.1); see §3.3 for its residual bound |

## 3. Tolerance derivations (Step 2 — written, not vibes, per III.12(b))

### 3.1 The size-scaled bound

**Formula:** `tolerance = max(1e-10, 1e-11 × N_entities)`, per the plan's
Step 2 instruction. `N_entities` is read from the comparison's own entity
count at runtime (checkpoint-column families: `len(state.entities)` at the
scenario's node count; Michigan/tri-county: county count, 83 / 3).

**Independent validation against existing, already-shipped practice:**
`src/babylon/domain/economics/substrate/circulation.py:12-14`
already documents, for the sparse `od_matrix.T @ v_vec` LODES matvec with
"~1000+ hexes," an observed ~1e-9 accumulated error and a chosen tolerance
of **1e-8** — which is exactly `1e-11 × 1000 = 1e-8`. The formula was not
invented for this report; it reconstructs a bound this codebase already
uses in production, and this is the first place it is written down as a
general rule rather than a single module's private constant.

### 3.2 Per-family justification (deterministic families only)

| Family | N (typical) | Tolerance | Justification |
|---|---|---|---|
| `tick`, `counties_alive`, `counties_with_population` | any | exact (integer) | integer/counter values, no floating-point accumulation — not a tolerance-bounded comparison at all, an equality check |
| `p_c_wealth`/`c_b_wealth`/`c_w_wealth` (11-canon, 4-node scenarios) | 4 | `max(1e-10, 4e-11) = 1e-10` | pure IEEE-754 basic arithmetic (production/tribute/rent tensor lookups — additions, multiplications; no transcendental, no iterative solver) — the tight end of the bound |
| `imperial_rent_pool` | 4 | `1e-10` | same class: rent-pool tensor math, basic ops only (Task 10 row 10) |
| `exploitation_tension` | 4 | `1e-10` | dialectics core (coupling/opposition/regime) — basic ops; `ContradictionFieldSystem`/`FieldDerivativeSystem` are field-derivative numeric cores but Task 10 records no named float hazard (no transcendental) for either |
| `total_v`/`total_c`/`total_s`/`total_k` and per-county `v`/`c`/`s`/`k` (Michigan: 83 counties; tri-county: 3) | 83 / 3 | `max(1e-10, 8.3e-9) = 8.3e-9` (Michigan); `1e-10` (tri-county) | production tensor sums across counties — basic ops, size-scaled for the larger county count exactly as the LODES precedent (§3.1) validates |
| Any family downstream of the six named transcendental/solver sites (Task 5/10's numeric-closure audit: `np.linalg.inv` Leontief inverse, `np.linalg.eig` class-transition, `scipy.optimize.linprog` Ollivier-Ricci curvature, `scipy.sparse` circulation/LODES matvecs) | site-dependent | the site's own documented/derived vector tolerance, propagated to the consuming family — **not** the generic `1e-11 × N` bound | per spec §8, item 3: "per-intrinsic golden vectors ... with written tolerance derivations against Python values" is the intrinsics layer's job, not this game-level layer's; this report does not re-derive those six sites' tolerances (that's `reports/numeric-closure-audit-2026-07-29.md`'s scope) — it only notes that any *game-level* family whose value chain passes through one of those sites inherits that site's tolerance rather than the flat bound |

**None of the six spec-named numeric sites (inverse/eig/linprog/sparse ×2)
are on the direct write-path of any of the concrete checkpoint or
terminal-state families audited in §2** — `inter_industry.py`,
`production_chain_rent.py`, and `class_transition.py` are BEA-scale
industry/class tensor modules consumed through
`TickDynamicsSystem`/`ProductionSystem` at a coarser level, and
`circulation.py`/`lodes_commute_matrix.py` feed the Michigan/tri-county
wage-redistribution path (`v`/`wealth`-adjacent, per §3.1's own worked
example). A precise family-by-family trace of which one of the six sites
each family passes through was not performed in this pass (would require a
multi-hop dataflow trace beyond Task 13's one-hop scope) — flagged as a
Task 17 pre-freeze item alongside the two open items Task 10 already
named.

### 3.3 Conserved-sum columns get an additional residual bound

`conservation_audit` rows in `michigan-e2e.json` already carry a computed
residual per tick: the sole invariant present,
`imperial_rent_phi_week_distribution`, shows a **max observed |residual| of
4.44 × 10⁻¹⁶** across all 3,633 audited ticks (2 ULPs of float64) — a pure
IEEE-754 accumulation error over a small sum, nowhere near any of the
tolerances in §3.2. **Conservation-residual bound: `1e-9`** — generous
headroom over the observed 4.44e-16, tight enough to catch a genuine
conservation break (a broken conservation law produces residuals many
orders of magnitude larger, not a marginal one).

## 4. The ensemble declaration (Step 3)

### 4.1 Parameters

- **N = 32 seeds** (power of two; per plan, sized to fit a single-flight
  overnight Michigan batch).
- **Envelope statistics:**
  - **Per-invariant pass rate: 100%.** Invariants (conservation laws,
    `Currency` non-negativity, the endgame priority order) are laws, not
    distributions — every one of the 32 seeds must pass every invariant, or
    the ensemble fails outright. This is not an envelope in the statistical
    sense; it is a hard gate applied per-seed.
  - **Endgame-outcome distribution: exact count bands** against a 32-seed
    Python-side reference run (this task; see §4.2 below for the pilot that
    stands in for it, and the scheduling note for the real N=32 run).
  - **Stochastic-family continuous statistics** (§2's tainted families:
    `wealth`/`ideology`-derived `consciousness`/`p_revolution` for the
    non-electoral six, plus `faction_balance`/`legitimation_index`/doctrine
    tags for the five electoral goldens): compared as **(mean, stdev)
    envelopes** across the 32 seeds — a family passes if the candidate
    engine's 32-seed sample mean falls within ± 2 sample-stdev of the
    Python reference's — not a per-value tolerance (R8: the RNG algorithm
    itself changes at cutover, so per-value comparison is meaningless for
    these families; that weakening is accepted and stated, per spec §8.5's
    own framing, not hidden).

### 4.2 Pilot run (this task's authorized deviation)

Per the orchestrating task's pre-authorization: Task 2's tick-profile
evidence showed even a **5-tick** `michigan-statewide-no-canada` profiling
run running past 8 minutes in background without completing. This report's
own sanity check (§4.2.1) confirms the same order of magnitude
independently. Both readings satisfy the pre-authorized fallback
("N = 2 if a single run exceeds 8 minutes") — so this task ran a **pilot
of N = 2 seeds**, serially, single-flight, on the real target scenario
(`--scope michigan-canada --ticks 520`, matching `sim:e2e-michigan`/the
`michigan-e2e.json` baseline's own parameters) to validate the harness
end-to-end, not to produce the statistically meaningful 32-seed reference
distribution.

**The full N = 32 reference distribution is NOT run in this task.** It is
scheduled as a single-flight overnight job, to run before the freeze tag
(Task 17), using this same command shape (`--scope michigan-canada --ticks
520 --seed <0..31> --output-dir <scratch>/seed_<n>`, looped serially, never
fanned out) and the aggregation approach validated by this pilot.

#### 4.2.1 Pilot mechanics and result

Command (seeds 3001, 3002 — chosen distinct from the baseline's own seed
2010 to avoid any confusion with `defines.rng_seed` default):

```bash
export BABYLON_PG_DSN="host=localhost port=5433 dbname=babylon_test user=test password=test"
for seed in 3001 3002; do
  uv run python -m babylon.engine.headless_runner \
    --scope michigan-canada --ticks 520 --seed "$seed" \
    --output-dir "<scratch>/seed_$seed"
done
```

A `--ticks 2` sanity pass was run first and killed once it confirmed the
harness bootstraps correctly (session partitioning, SQLite reference
hydration, hex hydration for all 83 counties, LODES map construction) — it
had not completed a full run within several minutes, independently
corroborating Task 2's "session/hydration overhead dominates short runs"
finding, and confirming the N=2 pilot-size decision before committing
wall-clock to the full 520-tick pair.

**Result — seed 3001 CRASHED (`ENGINE_FAILURE`), before seed 3002 was
attempted.** The run reached `ERROR ENGINE_FAILURE: ValidationError: 1
validation error for EconomicConditions / phi_hour / Input should be
greater than or equal to 0 [input_value=-9.8384306576833e-07]` roughly 5
minutes in (well before a full 520-tick completion, per the process's
CPU-time and log timestamps). `src/babylon/domain/economics/dynamics/types.py:176`
declares `phi_hour: float = Field(..., ge=0.0, ...)` — a strict
non-negativity Pydantic constraint with **no epsilon tolerance** for
floating-point accumulation noise. The checked-in `michigan-e2e.json`
baseline (seed 2010, the `defines.rng_seed` default) completes all 520
ticks without hitting this — so the bug is **seed/RNG-path-dependent**: it
is not that seed 3001 is unlucky in some trivial sense, but that *some*
accumulated floating-point path under *some* seeds pushes `phi_hour`
infinitesimally negative (`-9.8e-7`, itself smaller than several of this
report's own §3 tolerances) and a strict `ge=0` gate has zero headroom for
that.

**This is exactly the class of finding an ensemble/pilot is for**: the
single fixed-seed baseline (2010) never exercises this path; varying the
seed surfaced it on the very first alternate seed tried. **This is
correctness-adjacent evidence, not this task's to fix** — Task 13 is
Phase-0 evidence-gathering under R2's "no new Python engine features / one
authorized bug fix" constraint (that authorized fix is Task 1's
`defines_hash` unification, not this). Recorded here as a finding for
whoever owns `EconomicConditions`/`phi_hour` next (candidate fix shape: an
epsilon floor/clamp at the accumulation site, or a small negative tolerance
on the Pydantic constraint — a decision for that owner, not asserted here).

A retry with two further seeds (1, 42) was launched to get a *completed*
run's worth of continuous-family data; seed 1 was still running when this
report was finalized and was terminated for cleanup rather than left
unsupervised in the background. **Net pilot outcome count: 1 of 3 attempted
seeds (3001) reached a terminal state (crash) within the observation
window; 2 (1, 42) were inconclusive (killed mid-run, no result).** This
still discharges Step 3's "validate the harness" goal — the harness
correctly launched, ran, and surfaced a real engine failure with a
complete, actionable traceback — but it does **not** produce a completed
continuous-statistic sample to report a mean/stdev envelope from. The
scheduled overnight N = 32 run (§4.2's opening paragraph) is the first
point at which a real outcome-count table exists; it should budget for this
crash mode explicitly (i.e., decide up front whether an
`ENGINE_FAILURE` counts as a distinct outcome band or invalidates that
seed's run for ensemble purposes — a call for whoever runs that job, since
it changes what "100% per-invariant pass rate" (§4.1) means when the
process itself doesn't reach a terminal tick).

### 4.3 What the pilot found (finding, not yet the reference distribution)

**No endgame outcome fires in the Michigan scope within 520 ticks, in
either baseline or pilot evidence.** `tests/baselines/michigan-e2e.json`'s
86,891-row `events` array contains zero events of the five terminal-outcome
types (`lifecycle_transition`, `surplus_extraction`, `organizational_action`,
`level_transition`, `population_attrition`, and one calibration-warning type
are the only six event kinds present — none is
`REVOLUTIONARY_VICTORY`/`ECOLOGICAL_COLLAPSE`/`FASCIST_CONSOLIDATION`/
`RED_OGV`/`FRAGMENTED_COLLAPSE`). This matches Task 10's coverage findings:
`SovereigntySystem`, `FactionInfluenceSystem`, and `CollapseTransitionSystem`
are all `GAPPED` in the canon scenarios for want of `SOVEREIGN`/`FACTION`
nodes — and the Michigan county-aggregate scope seeds neither. **Consequence
for the ensemble design:** an "endgame-outcome distribution" comparison
across the 32 Michigan seeds is expected to be **degenerate** — 32/32
`exit_reason=COMPLETED`, no qualitative outcome, in both the Python
reference and (once it exists) the Rust candidate. This is not a defect in
the ensemble design; it is a finding about *which* families this game
scope's ensemble should actually gate on: not a 5-way outcome distribution,
but the continuous stochastic-family envelopes named in §4.1 (mean
p_revolution/p_acquiescence/ideology spread across the 83 counties). **The
5 electoral goldens are where the discrete endgame-outcome-count envelope
is the meaningful instrument** (those scenarios' `final_outcome` field is
populated and varies — that's the RNG-heaviest cluster per §2.1).

## 5. The cutover-ceremony procedure (Step 4)

Under the §6.5 baseline-ceremony law (`CONTRIBUTORS.md`), this is a
**bounded parallel-run window**, not a standing oracle:

1. **Freeze.** Check out the tag `p27-python-freeze` (cut at the end of
   Phase 0, per spec §10) in both the Python tree and the Rust
   (`babylon-engine`/`babylon-kernel`/`babylon-bsl`) tree. Both engines run
   against the **same frozen content**: the 11 canon scenarios' TOML/JSON
   definitions, `defines.yaml`, and the two Article-IV scope definitions
   (`michigan-canada`, `detroit-tri-county`).
2. **Run order — single-flight, one engine at a time, never concurrently:**
   1. Python engine, all 11 canon scenarios (existing `qa:regression`
      shape) + both Article-IV scopes, each once — this reproduces the
      already-frozen baselines (a no-op check that the freeze tag matches
      the last-blessed baseline).
   2. Rust engine, same 11 + 2, same defines, same content, each once.
   3. Python engine, the 32-seed Michigan ensemble (§4.2's scheduled
      overnight job) — the stochastic-family reference distribution.
   4. Rust engine, the same 32 seeds, same scope/tick count.
3. **Produce the extended drift table** (Task 14's format,
   `tools/generate_ceremony_message.py --ensemble-report`): one row per
   deterministic family with `max |d|` against its §3.2 tolerance (or the
   §3.3 residual bound for conserved sums); a second table, one row per
   stochastic family, with `(N=32, envelope, observed statistic, pass/fail)`
   per §4.1.
4. **Pass criteria — all of the following, no partial credit:**
   - Every deterministic family's `max |d|` ≤ its Step-2 tolerance, on
     every one of the 11 + 2 scenarios.
   - Every stochastic family's 32-seed sample mean within ± 2 sample-stdev
     of the Python reference's, on every scenario where that family is
     tainted (§2).
   - Every per-invariant pass-rate row at 100% (§4.1) — no invariant may
     fail on any single seed.
   - The qualitative endgame-outcome contract (priority order
     RED_OGV > FRAGMENTED_COLLAPSE > ECOLOGICAL_COLLAPSE >
     FASCIST_CONSOLIDATION > REVOLUTIONARY_VICTORY) holds wherever an
     outcome fires at all (expected only on the 5 electoral goldens per
     §4.3).
5. **Failure mode: cutover blocked, no partial bless.** If any row fails,
   the ceremony does not commit. The finding is written up (which family,
   which scenario, by how much), the Rust side is fixed, and the whole
   ceremony re-runs from step 2 — **not** a re-run of only the failing
   scenario, since a partial re-run cannot prove the fix didn't regress
   something the first pass caught.
6. **The single blessed mega-ceremony commit.** One commit, subject
   `test(baselines): P27 cutover — Rust engine re-baseline (§10, ADR TBD)`,
   body carrying the full drift table (both halves) and the
   `Baselines: blessed(p27-cutover)` trailer, generated via
   `tools/generate_ceremony_message.py --slug p27-cutover --summary "..."
   --ensemble-report <path>`. This is the one-time artifact-level
   differential the spec describes (§8) — not a standing dual-engine
   oracle; after this commit, Python's baselines are the historical record
   and Rust's are canonical.
7. **Who signs it: the Director** (Constitution §IX.5, Amendment AD) — the
   cutover ceremony is exactly the kind of irreversible, ideological-line-
   adjacent commitment (Python's engine retiring as the source of truth)
   that the Agentic Engineering model reserves to the Director, not a
   green-gate self-merge. **What a failure does: cutover blocked, Phase 2
   (whatever depends on the Rust engine being canonical) does not begin,
   and the finding routes back to whichever Phase 1/Phase 2/Phase 3 work
   produced the drift** — never a quiet tolerance-widening to force a pass.

## 6. Known gaps / follow-ups (honest, not hidden)

1. The mechanical one-hop trace (§1) over-taints to 96% and is not,
   on its own, usable for family designation — §2's source-verified
   classification is the one Step 2 actually consumed. A precise dataflow
   tracer (real write/read distinction via AST, multi-hop) is future work,
   not attempted here (out of Task 13's one-hop-scope).
2. `p_w_active`'s true writer was not traced (§2.1) — flagged for Task 17.
3. Michigan/tri-county's `p_acquiescence`/`p_revolution`/`ideology_*`
   families (§2.2) are classified by attribute-name correspondence with the
   confirmed 11-canon mechanism, not independently re-verified against the
   county-aggregate model's actual role-eligibility gate — flagged for
   confirmation before the cutover ceremony.
4. Which of the six Task-5/10 numeric sites feeds which concrete game-level
   family (§3.2's closing paragraph) was not traced past one hop — a Task 17
   item, alongside the two Task 10 already named (the gated `n.py`
   sub-stage, and `OODASystem`'s `observability.py` `round()` sites).
5. The full N = 32 Michigan reference distribution is **scheduled, not
   run** — a single-flight overnight job before the freeze tag (§4.2).
6. **A real, reproducible `ENGINE_FAILURE` was discovered during the
   pilot** (§4.2.1): `EconomicConditions.phi_hour`'s strict `ge=0.0`
   Pydantic constraint (`domain/economics/dynamics/types.py:176`) rejects a
   tiny negative floating-point noise value (`-9.8e-7`) reached under seed
   3001 but not under the baseline's seed 2010. Not fixed here (out of
   Task 13's scope and R2's no-new-features constraint) — flagged for
   whoever owns that model next. The overnight N = 32 job (item 5) should
   decide up front how an `ENGINE_FAILURE` mid-run counts against the
   ensemble's pass criteria.
