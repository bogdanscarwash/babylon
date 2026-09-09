# Director memo — TickDynamics reserved trio, posed per ADR208 R13

**For:** the Director's next docket sitting. **From:** the engineering workforce, per
ADR208 R13 (2026-08-17): *"TICKDYNAMICS — SPLIT. The engineering half proceeds now: #563's
per-scenario dormancy re-read and the ServicesProtocol boundary charter. The reserved trio
(bifurcation directional score, five-share ClassDistribution, dispossession_cascade_milestones)
returns to the Director at the next sitting as a precise question once the charter output
exists."* The charter output now exists —
`reports/t6-tickdynamics-dormancy-reread-2026-08-17.md` (this repository). This memo is that precise question.

**What this memo is not.** No recommendation is offered on any of the three surfaces below.
Register memo row 21 (`reports/register-memos/rows-21-24.md`, 2026-08-14) already ran a
workforce-lean analysis on this exact trio and reached "B" with reasoning attached — that
analysis is cited below as prior art (it is where the file:line evidence and the option
letters below come from), but this memo deliberately does not repeat or endorse its lean.
Constitution IX.5: the ideological/theoretical line is the Director's alone. The three
questions below are option spaces, not proposals.

**Why these three, together.** All three live inside `TickDynamicsSystem` @4.0
(`src/babylon/domain/economics/tick/system/__init__.py`), the largest unported
Material-Base surface and, per the Phase-1 port inventory's own self-correction, "the
ideologically densest system in the batch" whose own inventory never flagged any of them
as reserved (`reports/port-inventories/tick-dynamics-port-phase1-inventory-2026-08-12.md:1136-1146`).
Porting any of the three verbatim, by silence, ratifies a theory-laden surface without a
ruling. That is the risk this memo exists to prevent.

---

## 1. The bifurcation directional score

**What it is.** A single scalar in `[-1, +1]` computed once per county per annual boundary,
`BifurcationRiskCalculator.compute` (`src/babylon/domain/economics/crisis/bifurcation.py:71-260`).
Formula (`:9-17`, `:115-117`):

```
raw      = -w_s · solidarity + w_b · burden
dampened = raw · (1 - legitimation)
score    = clamp(dampened, -1, +1)
```

`w_s`, `w_b` are `crisis.bifurcation_solidarity_weight` / `.bifurcation_burden_weight`
(both default 1.0, `config/defines/economy_basic.py:113-134`); `class_burden_epsilon`
(0.001) guards the burden-ratio division. **Four defines are read at the construction
site** (`system/__init__.py:2269-2275,2277,2294-2302`), not three as an earlier draft of
this register row had it: the three weight/epsilon constructor args plus
`bifurcation_event_threshold=0.5`, which gates the `BIFURCATION_THRESHOLD` event
(`:2294-2302`). Sign convention: negative = revolutionary, positive = fascist
(`direction` string set at `:2329`).

Two sub-computations feed it. **Solidarity density** (`_compute_solidarity_density`,
`bifurcation.py:126-180`): actual-over-possible cross-class `SOLIDARITY` edges within a
county, an O(n²) all-pairs count. **Legitimation** (`_compute_legitimation`,
`:182-235`): blends a LifecycleSystem-produced structural index with inverse mean
agitation; the blend never actually engages because
`node.id == fips` (`:101-104`) compares a node id like `T001` against a county FIPS like
`"26163"` — always false — so legitimation always falls to the agitation-inverse branch
alone in every scenario, every tick.

**Dormancy (re-confirmed by the T6 re-read, memo 1 §2.2).** `.compute()` never fires in
any qa scenario — the sole annual boundary has empty `prev` state
(`system/__init__.py:2281-2284`), and `.compute()` needs a second boundary (tick ≥ 104)
to run at all. It DOES fire in `michigan_canada_e2e` from the second boundary onward, but
is **structurally NEUTRAL while crisis stays NORMAL** (`bifurcation.py:93-94`) — and the
revolutionary (−) term is at further structural risk: SOLIDARITY edges are **deliberately
never seeded** in the frozen engine (`bridge.py:842-846`, Constitution III.5/Q4), while the
fascist (+) burden term carries no equivalent restraint. In every scenario this memo's
evidence covers, the score has never been observed producing a negative (revolutionary)
value from live data — only the two structural facts above (no boundary reached; NEUTRAL
while NORMAL) are confirmed, not the counterfactual of what the score would do if a crisis
phase and SOLIDARITY edges were both present.

**What port-as-is means concretely.** Transcribe the formula, the four defines, the two
sub-computations, and the direction-string convention verbatim as BSL `defconst`s and rule
forms; keep the `node.id == fips` mismatch (and hence the permanently-disengaged
legitimation blend) as documented dormancy, per the same discipline applied to every other
dead-end in this system (memo 1 §2.2, §5). The `[-1,+1]` semantics and the sign convention
carry over unchanged, sourced to ADR016 (the George Jackson direction law — cited by
register row 21 as the ruling under which "the score's semantics ARE its metric form").

**What re-derivation would mean.** Any of: rebalancing `w_s`/`w_b` so the two terms are not
structurally asymmetric under standing SOLIDARITY-seeding policy; fixing the
`node.id == fips` comparison so the legitimation blend actually engages (a behavior change,
not a typo fix, since it would alter which branch computes `legitimation` in every scenario
that reaches this code); or re-deriving the score's functional form entirely under the
standing no-imposed-forms doctrine (ADR172/173) the way the Vol I wage-pressure sigmoid
elsewhere in this same system already must (memo 1's port-inventory citations; NOT itself
part of this trio, already ruled ADR188 Row 7).

**Option space** (from register row 21, presented without a lean):
- **A. Port-as-is on all three surfaces in this memo.** Verbatim transcription. Cheapest;
  ratifies the current formula, weights, and the disengaged legitimation blend by silence.
- **B. Confirm the ClassDistribution taxonomy (§2 below) verbatim; give the score its own
  theory note first.** The note would state explicitly whether the revolutionary term's
  structural disarmament under standing SOLIDARITY policy is intended (crisis direction is
  earned by organizing, not given) or a defect to fix before port.
- **C. Full dossier treatment.** Route the score (and the other two surfaces) through a
  T4-style dedicated dossier before any port attempt. Most rigorous; delays the largest
  unported Material-Base system behind a new document.
- **D. Defer to the port charter.** Bundle the ruling into the TickDynamics port packet
  alongside T6's other open Director residue (the `employment_source` runner-gap question,
  memo 1 §5).

**Reserved-line flags — Director-only, not answerable by the workforce:**
1. The −1/+1 direction semantics themselves (this is ADR016's law — whether this memo's
   trio ruling touches that law at all, or only its TickDynamics instantiation, is itself
   a question the Director may want to settle explicitly).
2. Whether the revolutionary term's structural zeroing under no-SOLIDARITY-seeding is a
   **feature** (revolutionary crisis direction must be earned by organizing, matching
   Constitution III.5/Q4) or a **defect** (the mechanic should be able to express a
   revolutionary bifurcation without requiring pre-seeded SOLIDARITY, since nothing else
   in the engine currently produces SOLIDARITY edges for it to read).

---

## 2. The five-share ClassDistribution + Feature-016 transition engine

**What it is.** A five-key class-share distribution per county
(`dynamics/types.py:27-70`, `ClassDistribution`, sum-to-one `model_validator`): two shares
are **externally fixed** (bourgeoisie, petit-bourgeoisie); the engine's own dynamics
operate on the remaining three — labor aristocracy (LA), proletariat, lumpenproletariat.
Bootstrap values `0.01/0.09/0.40/0.35/0.15` (bourgeoisie/petit-bourgeoisie/LA/proletariat/
lumpen, in that order) are set at `system/__init__.py:822-830`. The engine that actually
MOVES these three shares year over year is entirely external to this system —
`dynamics/transition_engine.py:107-186`, invoked as one opaque call,
`services.transition_engine.simulate_transitions(dist, conditions, crisis_phase=...)`,
gated `transition_engine is None → return` (`system/__init__.py:2366-2367`).

**Dormancy (re-confirmed).** The taxonomy and its bootstrap values are LIVE in every
scenario (they exist on every county from tick 0, and the sum-to-one validator runs every
boundary in all six). The transition ENGINE that moves them is live only in
`michigan_canada_e2e` (83 counties × 9 boundaries, `system/__init__.py:2424-2454`) — frozen
at bootstrap values in all four qa scenarios and in `detroit_tri_county`'s committed
artifact (memo 1 §2.3 table).

**What port-as-is means concretely.** The `ClassDistribution` shape and its five bootstrap
shares transcribe as-is — register row 21 characterizes this half as "the frozen engine's
core MLM-TW structure, golden-pinned and heavily validated." The transition engine
(`dynamics/transition_engine.py`) is a separate, much larger surface (80 lines of its own
math, not counted in TickDynamics' own catalog) that would need its own read-through before
any port verdict — it is out of THIS system's Phase-1 inventory scope by explicit design
(the inventory treats external `ServicesProtocol` calculators as named dependencies, not as
things it re-derives).

**What re-derivation would mean.** Revisiting which two classes are externally fixed vs.
engine-driven (a modeling choice already made under ADR171 — "the LA share is the theory's
central stratum" — and Program 19/ADR070's emergent-class-partition cutover, which this
taxonomy is stated to coexist with, not be superseded by); or re-deriving the transition
engine's own mechanics, which is a materially larger undertaking than this memo's scope.

**Option space** — same A/B/C/D structure as §1 above (register row 21 treats all three
surfaces as one bundled ruling with shared options): under B specifically, this surface
would be confirmed **verbatim**, distinguishing it from the score, which would get the
theory note.

**Reserved-line flags — Director-only:**
1. Whether the 0.40 LA bootstrap share is the intended starting pedagogy of the
   dispossession/proletarianization arc, or an artifact worth revisiting.
2. Whether the taxonomy's two-fixed/three-dynamic split is still correct given Program 19's
   emergent class-partition work landing independently — this memo does not resolve that
   coexistence question, only flags that both exist and were rulled to coexist (ADR070).

---

## 3. `crisis.dispossession_cascade_milestones`

**What it is.** A three-element list, `[0.05, 0.10, 0.15]`
(`config/defines/economy_basic.py:137-140`) — cumulative labor-aristocracy decline
thresholds (5pp, 10pp, 15pp). Sole reader: `_check_dispossession_cascade`
(`system/__init__.py:1115-1170`), which takes the HIGHEST milestone crossed
(`decline = baseline_la - current_la`; `for milestone in sorted(milestones): if decline >=
milestone: crossed = milestone`, `:1149-1150`) and, when crossed, emits
`EventType.DISPOSSESSION_CASCADE` with a 6-decimal-rounded payload (memo 1 §3.2).

**What "reachable" means.** The check only runs from `system/__init__.py:2444-2452`, gated
on THREE conditions simultaneously: the transition engine must be wired (§2's gate), crisis
phase must be non-NORMAL, and a prior county state must exist. It rides the exact same
Feature-016 machinery as §2 — the milestones cannot fire independently of the transition
engine being live.

**Dormancy (re-confirmed).** `DISPOSSESSION_CASCADE` is emitted in **zero committed
artifacts** across the estate (memo 1 §3.2, cross-checked against the same three-gate
condition). Even in `michigan_canada_e2e`, where the transition engine IS wired, no
committed golden shows this event firing — whether that is because no county's decline
ever crosses 5pp in the committed run, or because crisis phase never leaves NORMAL when a
county does cross it, is not settled by this re-read (settling it would require running the
scenario and inspecting the event log, which this task's scope prohibits — see the
runnable checklist below).

**What port-as-is means concretely.** Transcribe the three-value list and the
highest-milestone-crossed logic as a BSL `defconst` + rule form, riding the same
Feature-016 machinery §2's port decision already has to build. Register row 21 groups this
explicitly with §2 ("milestones ride with it, same FR-022 machinery") — there is no
independent port decision for this surface separate from the transition-engine question.

**What re-derivation would mean.** Revisiting the specific pp thresholds (5/10/15) as
calibration, or the "highest milestone crossed, not all milestones crossed" semantics
(a county that jumps straight past 5pp and 10pp to 16pp in one boundary only ever emits
ONE event, at the 15pp milestone — the 5pp and 10pp crossings are silent).

**Option space** — same bundle as §§1–2; register row 21 does not offer an independent
option letter for this surface alone.

**Reserved-line flags — Director-only:**
1. Whether 5pp/10pp/15pp is the intended pedagogy of dispossession's pace, or a calibration
   placeholder.
2. Whether "highest milestone only" is the intended narrative beat (one dramatic event per
   crisis, not a stepped sequence) or an under-specified corner the Director wants named
   explicitly before it ships silently.

---

## 4. What would need to change for a genuine per-scenario answer on the open sub-questions above

Two open items in this memo (§1's "has the score ever gone negative from live data" and
§3's "why has DISPOSSESSION_CASCADE never fired") are read-only-unanswerable — they need
execution, which was out of scope for this task. If the Director wants those settled before
ruling, here is the exact runnable checklist (none of this was run for this memo):

1. **Run `michigan_canada_e2e` to completion** (`mise run test:q -- <its qa entry>` or the
   headless runner directly, however that scenario is currently invoked) **and inspect the
   event log for `BIFURCATION_THRESHOLD` and `DISPOSSESSION_CASCADE` events across all 9
   annual boundaries** (ticks 52…468). Record: does `score` ever go negative? At which
   counties/years? Does any county cross a dispossession milestone, and if so does crisis
   phase happen to be NORMAL at that exact boundary (suppressing the event) or not?
2. **If no milestone is ever crossed in `michigan_canada_e2e`,** run a synthetic scenario
   seeding a county with a fast LA-share decline (e.g., a modified `single_county`-style
   fixture with `transition_engine` wired and a crisis-inducing shock) to produce at least
   one witnessed `DISPOSSESSION_CASCADE` emission, confirming the highest-milestone-only
   semantics fire as read.
3. **If the Director's ruling on §1 depends on whether the revolutionary term CAN go
   negative under any reachable state,** construct a synthetic scenario that seeds
   SOLIDARITY edges between same-county classes (bypassing the standing no-seeding policy
   for this diagnostic run only, NOT for canonical baselines) and confirm the score responds
   as the formula predicts. This is a diagnostic run, not a proposal to change canonical
   seeding policy.

None of these runs is required to pose the ruling questions above — the option spaces in
§§1–3 stand on the read-only evidence already gathered. They would only sharpen the
"is this a feature or a defect" reserved-line flags with observed data instead of
structural inference.

---

## Sources

`src/babylon/domain/economics/crisis/bifurcation.py`,
`src/babylon/domain/economics/dynamics/types.py`,
`src/babylon/domain/economics/dynamics/transition_engine.py` (cited, not read this pass —
out of TickDynamics' own Phase-1 scope by design),
`src/babylon/domain/economics/tick/system/__init__.py`,
`src/babylon/config/defines/economy_basic.py`,
`reports/register-memos/rows-21-24.md` (row 21, source of the option letters and the
file:line evidence for all three surfaces),
`reports/t6-tickdynamics-services-charter-2026-08-14.md`,
`ai/decisions/ADR208_docket_sitting_2026_08_17.yaml` (R13),
`ai/decisions/ADR016*.yaml` (cited by row 21, not independently re-read this pass),
`ai/decisions/ADR070*` / Program 19, ADR171 (cited by row 21 for §2's binding rulings, not
independently re-read this pass).
