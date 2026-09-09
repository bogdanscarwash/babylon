# TickDynamics @4.0 — the Class-Dynamics Engine Port Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the **Feature-016 class-dynamics engine** — the five-share `ClassDistribution`, its four rate
constructors, the phased crisis amplifier, the three flow equations, the clamp/normalize, the
`DISPOSSESSION_CASCADE` beat on a **restored cumulative baseline**, the ratified **Φ → savings → LA-mobility**
coupling, and the **retired-scalar** bifurcation county readout — into BSL rule content, executing **ADR210
R5–R11** verbatim (every Director gate on this estate cleared 2026-08-17).

**Architecture:** one new rule pack (`class-dynamics.bsl`, **13 rules**), one **additive edit** to an existing pack
(`fundamental-theorem.bsl`, **+1 rule** — R9's ruled home) **plus the declaration-only extension of
`two-classes.bscn` that edit forces** (§2.2 — the ruled home is a SHARED file with four consumers), **one** Rust
registration string, **one** Rust unit-test repair (`lib.rs`'s `per_rule_fired.len() == 1`), **46 named
`defconst`s** across **eight** conformance worlds plus **two** co-load worlds, with four Python frozen mirrors, **18 additive golden pins**
(8 load pins at tick 1 + 8 boundary pins at tick 52 + 2 further arc pins at ticks 104/156 — §Worlds; the **17** pre-existing pinned hashes
byte-identical: **16** in `tick_goldens.rs` **plus `babylon-client`'s `startup_tick_matches_the_pinned_hash`**,
which rev 1 never inventoried), **32 D-rows** and **ADR-NF**. **No language slice is needed** — every construct is
landed and cited in §4/§7 — but **two** constructs remain *served-but-content-unprecedented* and **Task 1 spikes
both before any rule depends on them** (rev 1 listed five; three were answered at the byte during the rev-2
verification pass — §9's "NOT blockers").

**Tech Stack:** Rust workspace (`rust/crates/{babylon-bsl,babylon-tick,babylon-client}`), BSL content, cargo via
`mise run rust:check`, Python 3.12 host venv for the frozen mirrors. **No `babylon-graph` change. No engine lane.
`babylon-client` gains no code — but it holds a pinned hash on the content this train edits, so its test leg runs
in this train's gates** (§2.2.1; rev 1 never named the crate).

**Rulings that govern.** ADR210 **R4** (STANDING THEORY RULING — the bifurcation score's revolutionary-term zeroing
under no-SOLIDARITY-seeding is a **FEATURE**: "revolutionary crisis direction must be EARNED BY ORGANIZING …
fascism is the default drift of unorganized crisis. The asymmetry … is the theory, not a defect"), **R5** (the
`0.40` LA bootstrap share, the `5pp/10pp/15pp` milestones, and **highest-milestone-only** semantics all CONFIRMED as
intended), **R6** (the bifurcation SCALAR RETIRES — Option B; the county readout becomes the population-weighted
mean of `(f − r)` over the landed ternary; four defines collapse to one; D2/D3 MOOT), **R7 ★** (the five class
shares are **MEASURED MEMBERSHIPS** — Option B; the percentile-band DESCRIPTIONS are the defect and die, the
confirmed bootstrap values survive as SEEDS; ADR070's emergent partition remains the post-cutover target), **R8**
(the transition-engine coefficients port as **NAMED BSL DEFCONSTS** — Option C: "declared, moddable, hash-covered
content; **no defines.yaml churn; no §6.5 ceremony**"), **R9** (the Φ → savings → LA-mobility coupling RATIFIED as
explicit law, **homed in `fundamental-theorem.bsl`**; the `wage·s²` double-application defect repairs at the port),
**R10** (the cascade baseline becomes **CUMULATIVE** — Option A, on the landed `p7-persist-baselines` pattern, with
the divergence D-row from the frozen previous-boundary read), **R11** (the 14-row defect ledger disposes **AS A
CLASS** per ADR183's repaired-at-the-port doctrine — each repair carries its D-row at this landing, no per-defect
sitting). ADR183 **R1** (the frozen engine is a **contract source for STRUCTURE and ORDERING, not a correctness
oracle**) and **R2** (defects repair at the port, **never in the frozen lane**). ADR172 ruling 5 / ADR173 ruling 1
(**no imposed functional forms**; the frozen logistic is reference-only, never the going-forward law). ADR208
**R14** (Checkpoint A = all 13 Material Base systems ported; WS3 HELD until then) and **R15** (the no-sliver
precedent BINDS). ADR198 **R6** (carrier-node idiom blessed) and **R7** (int-FIPS / node-identity keying). ADR070
(reads are population-weighted aggregates). ADR195 (enum member order is hash-bearing). ADR181 (merge protocol +
Copilot harvest). Constitution **III.11 / invariant S-11** (loud absence; no warning level, no degraded mode).

**Prior art to read before Task 0 (in this order):**
1. **The four charter dossiers for this train**, in
   `/tmp/claude-1000/-home-user-projects-game-babylon/*/scratchpad/tickdynamics-charter/` —
   `dossier-frozen-estate.md` (the frozen system, the 33-field ServicesProtocol boundary, the test estate, the
   defines census), `dossier-rulings.md` (**every ADR210 ruling verbatim** + the nine Director-reserved open
   questions), `dossier-landed-boundary.md` (**the boundary verdict** and the qname census — *its §2 table governs
   §2 of this plan*), `dossier-precedents.md` (the numbering tails, the house plan shape, the landed
   conformance-pack conventions). **Quote these rulings; never paraphrase them.**
2. `reports/tickdynamics-trio-dossier-2026-08-17.md` — the **1,142-line** read-through, findings F3–F22, the 14-row
   defect ledger, D1–D7. This plan transcribes from its tables **and re-verifies against the cited lines**.
   **THE FILENAME IS THAT ONE.** Rev 1 cited a second name, `dossier-tickdynamics-trio.md`, in §4.7, §5 and every
   §10 row; that path exists **nowhere in the repo** — it is the charter session's scratch-dir copy, inherited
   through `dossier-rulings.md`. Every line number rev 1 quoted resolves exactly against the real file (`:404-407`,
   `:690-694`, `:930-937` re-verified), so the CONTENT is sound and only the path is wrong. **Rev 2 normalizes
   every citation to `reports/tickdynamics-trio-dossier-2026-08-17.md`; a reviewer seeing the old name in any
   later commit should treat it as an un-rebased edit.**
3. `src/babylon/domain/economics/dynamics/` **in full** — the **seven named modules total exactly 1,476 lines**
   (`transition_engine.py` 346, `types.py` 321, `validation.py` 300, `crisis.py` 181, `dispossession.py` 127,
   `accumulation.py` 106, `savings_schedule.py` 95 — measured 2026-08-18, and this is the figure F9's
   "1,476 lines of the seven modules" names). The package's **ten** files total 1,927 (`data_sources.py` 228,
   `hardcoded_data.py` 125, `__init__.py` 98 on top). **The seven are the transcription source.**
4. `src/babylon/domain/economics/tick/system/__init__.py:2346-2458` (the call site), `:1115-1170` (the cascade),
   `:2241-2344` (the retired bifurcation surface), `:800-852` (the bootstrap).
5. `rust/crates/babylon-tick/content/rules/consciousness.bsl` **in full** — `p6-route` (`:294-338`) is the ADR016
   bifurcation law's **already-landed, strictly richer** home (R6's premise), and `p7-persist-baselines`
   (`:340-351`) is **R10's cited carrier pattern, verbatim**.
6. `rust/crates/babylon-tick/content/rules/{production,decomposition,control-ratio,dispossession,territory}.bsl` —
   the `neighbors`-scoped fold idiom **and its `exists`-protector**
   (`territory.bsl:168-172` — the ONE landed territory-side fold, and the idiom `a13` must copy, §4.6/C5), the
   `(nodes NodeType/…)` fold idiom (`decomposition.bsl:284-291`, `control-ratio.bsl:281-287` — **landed since the
   2026-08-17 draft claimed otherwise; that claim is STALE**), the reset-then-accumulate split
   (`production.bsl` `p0`), the **write-once latch** (`decomposition.bsl:248-260`'s
   `(= crisis-known 0)` guard-then-latch — **`a10`'s real precedent**, §7/I3), the seven-companion-scenario
   branch-coverage pattern (`dispossession.bsl`), the **D136 territory-side-fold divergence record**
   (`production.bsl:83-107` — *not* D45, which is the `select-max` ascending-id tiebreak, §2.4), and both
   **`(intrinsic floor …)` declarations — `territory.bsl:78` AND `decomposition.bsl:212`, byte-identical, which
   is why those two files ALREADY cannot co-load (`E-LOAD-001`, open issue #646, disclosed in both headers:
   `territory.bsl:67-77`, `decomposition.bsl:30-45`).**
7. `rust/crates/babylon-bsl/src/{bindings,tick,typecheck,rule_pipeline,scenario,score_class,grammar,reader}.rs` —
   the served/refused calendar bindings (`tick.rs:419-470`), the five fold arms and the weighted-mean law
   (`typecheck.rs:130-236`), `field_ref_for`'s three-shape fold-body law (`rule_pipeline.rs:624-773`), the seeding
   arms (`scenario.rs:1093-1330`), `FoldOp`'s closed set (`grammar.rs:672-683`), `ARITH`'s closed set
   (`grammar.rs:724` — `+ - * /` and nothing else), and the scaled-literal lexer (`reader.rs:863-930`).
8. `ai/bsl-architecture-standard.md` §3.2 (no imposed functional forms), §4.5 (the fuel declare-bound+1 readback),
   §5.4 (defects not to transcribe), §6.2 (the carrier-node idiom + the two-homes D-record convention).

---

## Global Constraints

**Every task's requirements implicitly include this section.**

- **Port-as-is (Director law, ADR183 R1).** The frozen Python is the **structure and ordering contract, not a
  correctness oracle**. Transcribe exactly; **every divergence earns a D-row** (the D-record table enumerates all
  28). Where ADR210 rules otherwise, **the ruling wins and the D-row records which ruling**.
- **THE RULINGS ARE LAW — transcribe them, never re-open them.** R4 (what the asymmetry means), R5 (which constants
  are intended), R6 (which surface retires), R7 (what a class IS), R8 (where the coefficients live), R9 (whether the
  Φ channel exists and where it is homed), R10 (baseline semantics), R11 (the defect class) are **Director
  rulings**. If an implementation detail seems to require re-litigating one: **STOP and escalate**; do not resolve
  it in content. The nine still-open Director-reserved questions live in §10 and **no task decides one**.
- **Defects repair at the port, NEVER in the frozen lane (ADR183 R2).** **No Python source changes, none.** Not
  `dynamics/`, not `tick/`, not `config/defines/`, not `data/defines.yaml`. `mise run qa:regression` and
  `mise run qa:vault-regression-ci` are therefore byte-identical **trivially** — run them once anyway as proof
  (Task 12). **No file under `tests/baselines/**` may move**; if one does, **STOP**.
  **This is STRICTER than ADR210's own expectation, and rev 2 records that rather than restating a Director
  disposition as its opposite.** B10, verbatim (`ADR210:183-188`): *"R10 earns its divergence D-row, and NO
  baseline ceremony is EXPECTED — the cascade emits in ZERO committed artifacts, the michigan_canada_e2e baseline
  included (where the transition engine IS live) — **but that is an expectation, not a law: if R10's landing drifts
  the michigan E2E baseline, the standard §6.5 ceremony applies at that landing**."* The reconciliation: B10
  anticipates a landing that could touch the Python lane; **this train touches none**, so a baseline move here has
  no causal path from this train's diff and is a **bug in something else**, not a drift this train may bless. So
  the STOP is a STOP **to diagnose**, and it resolves exactly two ways — (i) the cause is outside this train and
  this train proceeds unchanged, or (ii) the cause is inside it, which means the no-Python-changes premise was
  false, in which case **B10's standard §6.5 ceremony applies at that landing, exactly as B10 says**. What is
  forbidden is re-blessing a baseline without passing through that fork. **D-NF+7 carries the reconciliation;
  never paraphrase B10 without quoting it.** The two motions an earlier draft of this plan scheduled into the
  frozen lane (a `phi_cap` define; a `ClassDistribution` docstring rewrite) are **both removed** — see the next two
  bullets.
- **`phi_cap` is a BSL `defconst`, NOT a `GameDefines` define.** R9's text reads "with `phi_cap` promoted to a
  define", but it is one sitting-mate of R8, which rules the **same coefficient estate** (`savings_schedule.py`'s
  "5 class rates + phi cap") into **named BSL defconsts** with "**no defines.yaml churn; no §6.5 ceremony**", and
  the dossier's own D6-A language ("give `phi_cap` a define with provenance; record it in the rule's
  `:material-basis`") names **BSL rule metadata**, not a `GameDefines` field. Adding a real define would move
  `canonical_defines_hash` (gated at `tools/regression_test.py:1279-1283`), cost an **11-baseline §6.5 ceremony**,
  and have **zero effect on the ported engine — the Rust/BSL engine does not read `GameDefines` at all**. The
  defconst is therefore the reading that satisfies both rulings at zero cost. **D-NF+19 records the resolution;
  §10 DG-9 puts the reading in front of the Director rather than assuming it.**
- **R7's membership semantics land in CONTENT, not in a Python docstring edit.** ADR210's consequence text says R7
  "rewrites the `ClassDistribution` model's field DESCRIPTIONS, not its values" — but ADR183 R2 forbids frozen-lane
  repair, and the frozen model is reference-only after the `p27-python-freeze` pin. This plan lands the ruling where
  it is load-bearing: the five `deffield` rows, their `:material-basis` provenance and the pack header state
  **measured membership**, and **no percentile-band language is transcribed anywhere**. **D-NF+26** records it;
  **§10 DG-11** asks the Director whether the frozen-lane docstring edit is additionally wanted (rev 1 filed it
under DG-8, which §10's table already assigns to the R8 question — one of the three DG-numbering collisions M6
names, all fixed in rev 2). **Do not edit
  `types.py` on this train.**
- **No imposed functional forms (ADR172 ruling 5; ADR173 ruling 1; NORTH_STAR.md).** This binds TickDynamics
  hardest of any Material Base system, and the surface **passes as it stands**: dossier finding **F9** is a
  negative — *"there is no imposed functional form anywhere in the transition engine. No exp, no log, no tanh, no
  sigmoid, no Gaussian, no power law — no transcendental of any kind across all 1,476 lines of the seven modules …
  ADR172 ruling 5 is satisfied by this surface as it stands."* **This pack declares NO intrinsic.** If an
  implementer reaches for a curve, they have mis-transcribed. `sigmoid` is additionally a prohibited intrinsic name
  (`E-LOAD-024`), and spelling a logistic out of `exp`/`log` is the same prohibited motion. **The one sigmoid
  inside @4.0 — `reserve_army/calculator.py:44-46`'s `sigmoid_k`/`sigmoid_r0` wage-pressure curve (frozen Step 3.5)
  — is OUT OF THIS PACK's boundary** (§0), is named to its residual train, and **must not be dragged in**.
- **Communities are never graph nodes; this pack mints no node type, no edge type, no hyperedge, no carrier.**
  Every datum is per-county, and the county **is** an existing `NodeType/TERRITORY` node. `tick.rs::subject_type_of`
  derives a rule's subject NODE type from its `:field` binding namespace via `namespace_to_node_type` (uppercase +
  `-`→`_`), so a `class-dynamics/`-namespaced `:field` binding would instruct the tick loop to iterate a
  `NodeType/CLASS_DYNAMICS` that does not exist. **Every `:field` binding in this pack is `territory/…` or
  `social-class/…`; the `class-dynamics/` namespace appears only in rule ids and `:const` references.** A permanent
  guard test asserts it (§7c).
- **Kinds are closed (Amendment AE (ii), AG (iii)).** This pack mints one `defenum` (`CrisisPhase`, five members
  transcribed in the landed order) and no verb, no intrinsic, no element kind, no adjunction, no rung.
- **Every theory call not already ruled goes to §10's DIRECTOR GATE, popup-ready. No task decides one.**
- **CROSS-TRAIN SAFETY — the ImperialRent (#563-adjacent) and Community (#536) trains land in UNKNOWN order
  relative to this one, and no qname this pack writes may collide with what their plans claim.** The authoritative
  disjointness argument is §2.3, built on the landed-boundary dossier's §2.1/§3 tables. Three hard rules: (1) this
  pack **never writes** `social-class/wages-paid`, `social-class/value-produced`, `wages/value-flow`,
  `social-class/wage-balance`, `social-class/wages-inbox`, `social-class/agitation`, `social-class/revolutionary`,
  `/liberal`, `/fascist`, `social-class/imperial-rent`, `institution/superwage-crisis-known`/`-tick`, or any
  `institution/rent-*` or `community/*` qname; (2) every field this pack **does** write is either `territory/…`
  (no other train claims the territory namespace) or the **one** net-new `social-class/ternary-net-fascist`
  publication, whose name is checked for zero prior hits at Task 0 **and re-checked at Task 12**; (3) the additive
  `fundamental-theorem.bsl` rule is the **ruled home** (R9) and ImperialRent's plan explicitly defers it — *"When
  D6-A lands it will extend `fundamental-theorem.bsl` with **TickDynamics** content … **on that train, not this
  one**"* — so the file is claimed by exactly one train. **Disjointness of WRITE sets is necessary and not
  sufficient: `fundamental-theorem.bsl` is a FILE that three other crates' tests load (§2.2), so this train's blast
  radius is wider than its write set and is bounded by §2.2's consumer table, not by this bullet.**
- **Numbering is NEXT-FREE-AT-LANDING.** This plan writes **`ADR-NF`** and **`D-NF+1 … D-NF+32`**, **never
  literals** — including when citing another train's rows. The tails measured in this worktree on 2026-08-18 are
  **D180** (`docs/reference/bsl-language.rst:8158`) and **ADR214** (`ai/decisions/index.yaml`), and the tail is
  **FOUR-WAY CONTENDED**: #491 has already committed **D181** in its own worktree (unmerged); the ImperialRent plan
  claims **D181–D201** as literals and targets `ADR214_imperial_rent_port_handoff.yaml` **by a name already taken**
  (its real next-free ADR is **ADR215**); the Community plan claims 25 `D-NF+n` rows plus one `ADR-NF`; this train
  is the fourth claimant. **Task 0 re-measures both tails and fixes this train's allocation; Task 12 re-measures
  again immediately before filing** and uses whatever is free then. A literal number written before Task 12 is a
  review failure.
- **SEVENTEEN pinned hashes are pre-existing, not sixteen — and the seventeenth lives in another crate.**
  `tick_goldens.rs` holds **18** `#[test]` functions, **16** of them `*_hashes_are_pinned` (dossier 3 §4;
  cross-confirmed by the ImperialRent plan's own count). **`babylon-client/tests/engine_link.rs`'s
  `startup_tick_matches_the_pinned_hash` is a 17th**, asserting `hex(&report.after)` =
  `783f651d…7679` on the SAME content pair `two_classes_fundamental_theorem_hashes_are_pinned` uses
  (`babylon-client/src/engine_link.rs:16-26` `include_str!`s both files). Rev 1 never inventoried it; §2.2 does.
  Two obligations, never conflated: (1) all **17** stay byte-identical in **every commit of every PR** — a move is
  a STOP, not a re-measure, and the Task-9 gate now names the client pin explicitly; (2) a pin this train adds is
  re-measured whenever a later rule changes its world, with the per-rule-id `fired` arithmetic explaining the delta
  recorded in the commit body. A pin that moves **without** a matching new rule id in the `fired` breakdown is the
  STOP condition.
- **A GOLDEN PIN RUN AT TICK 1 PROVES NOTHING ABOUT A RULE GATED TO TICK 52 — the pin tick is part of the pin's
  design.** `run_once` hard-codes tick 1 (`babylon-tick/src/lib.rs:517-531`, its own comment: *"`run_once` is one
  tick, and it is tick 1"*), `TickSession::new` starts at `tick: 0` and `advance` runs tick 1 first
  (`session.rs:60-66,120-124`), so **tick 0 is never executed by any driver**; `:tick-in-cycle 52` evaluates to
  `tick.rem_euclid(52)` (`tick.rs:269`), so this pack's boundary first opens at **tick 52**. Every pin this train
  adds therefore declares WHICH tick it pins and WHAT that tick can prove: a **load pin** (tick 1, `run_once`)
  pins the seeded world plus the pack's off-boundary inertness (`before == after`, this pack's `fired = 0` — true
  for **all thirteen** rules because **all thirteen carry the gate, `a12` included**; that is what rev 2.1's N1
  decision buys, and §7's `a12` row is where it is argued), and a **boundary pin** (tick 52,
  `TickSession::advance` ×52) pins the arithmetic. **A world with only a load pin has
  an unpinned engine output — that is the C2 defect rev 1 shipped, and it is a review-failure condition here.**
- **Golden pins MEASURED, never derived.** Run the engine once against the committed content, read the printed hash
  back, paste it. Never hand-compute, never carry a hash forward by reasoning. Same law for every `report.fired`
  count, which gets an inline per-rule-id arithmetic breakdown in its assertion message. **Every numeric assertion
  in every Rust test is measured from THIS engine's own run** — never copied from a mirror's printed float, even
  when the two happen to agree (the `control_ratio_conformance.rs` convention).
- **AN EDIT TO A LANDED RULE FILE IS AN EDIT TO EVERY CONSUMER OF THAT FILE. Enumerate them before writing a
  line.** `rg -n 'include_str!.*<file>' rust/` plus a grep for the file name across all four crates, recorded in
  the task's own step. §2.2 does this for `fundamental-theorem.bsl` (four consumers, one of them the shipped Bevy
  client) — rev 1 read the file and not its consumers, which is C1. The same obligation binds any later
  edit to `two-classes.bscn`, `consciousness.bsl` or `territory.bsl` this train might reach for.
- **BINDINGS EVALUATE UNCONDITIONALLY, EVERY TICK, BEFORE THE GUARD.** `run_tick` calls
  `check_sources_servable` then binds every subject before `guard_and_effects` is consulted (`tick.rs:583-609`);
  the landed estate states it in its own words (`control-ratio.bsl` `c03`'s `:material-basis`). So **`(when (=
  phase-of-year 0))` protects NOTHING inside a binding** — an aggregate that can abort must carry its own
  protector in the binding (`(if (exists …) … (- 0 0c))`, `territory.bsl:168-172`). `mean`/`min`/`max` (and
  `select-max`/`select-min`) over an empty set is `E-EVAL-021`, a loud tick-killing failure with no null
  (`evaluator.rs:143-147`).
- **A DECLARED FIELD'S RANGE IS ENFORCED AT THE STORE, LOUDLY.** `probability`/`intensity`/`coefficient` are
  `[0,1]` (`types.rs:230-237`); `real` carries no range law (`:239-244`); a store outside the declared range is
  `E-EVAL-020`, *"a loud failure, never a clamp"* (`evaluator.rs:139-142`). **Every field this pack declares picks
  its type against the widest value its own rules can produce** — §4.2's staging rows carry the derivation, and
  the `raw-share-*` trio is `real` precisely because `a06` produces negatives that `a07` then clamps (I6).
- **Fuel is MEASURED, never guessed (declare-bound+1 readback).** For every rule: declare a deliberately low
  `:fuel N`, load, read the `E-LOAD-040: … static bound B exceeds its declared :fuel N` refusal verbatim, set
  `:fuel B+1`, confirm it clears load **and** runtime against **every** scenario that loads the rule. **Fuel is a
  MAX over worlds, not per-world** — this pack's folds are bounded by seeded population, so a later, larger world
  that reds the load is the intended loud failure and the figure is re-measured. Task 10 is the dedicated sweep;
  no rule ships a guessed figure before it.
- **Mutation evidence per behavioral rule, per clamp, per guard, per constant, per dispatch arm:** break → a
  **named** test flips red → restore byte-identical (`git diff` clean), recorded in the commit body with the exact
  AST mutation. **Every one of the 46 `defconst`s must be mutation-provable by a fixture that exercises it**, plus
  a converse vector proving the other fixtures do **not** move — a `defconst` no test can kill is dead content, and
  that is precisely why eight content worlds exist rather than one. A clamp whose fixture cannot make it bind is not
  exempt: it owes a converse vector **plus** a recorded reachability proof. The 5-arm amplifier dispatch owes a
  vector **per arm** plus one proving the unexercised arms are unreachable in that world.
- **Frozen mirrors pasted VERBATIM and DATED.** Each Rust conformance file's doc-comment header carries: this plan's
  path, the frozen source file + line count, the exact `PYTHONPATH="$PWD/src" uv run python <mirror>.py` command,
  its **full verbatim stdout**, the date it was captured, and the "why exact equality, no tolerance" paragraph
  citing `bsl-language.rst` §4.3 + ADR183. The mirror is a **standalone, dependency-light script** driving
  `DefaultClassTransitionEngine` **directly** over a literal `WORLD` dict — the oracle, not the frozen engine
  (the D146/ADR183 convention). §8 is its recipe. **Two mirrors run twice** — once against the frozen `wage·s²` and
  once against the repaired `wage·s` — and both printouts are recorded, so the F11 repair is evidence, not a claim.
- **Every oracle exists or is created by a named task.** The four mirrors are created by Tasks 2, 4, 7 and 9. **No
  task may cite an oracle no task creates.**
- **Declare only what this pack's own rules read.** No speculative `deffield`, no speculative `defconst`. In
  particular: the three P→L dispossession weights, the four unread savings rates, and the two legacy
  `DefaultCrisisAmplifier` multipliers are **recorded in D-rows and declared nowhere** (D-NF+11, D-NF+12, D-NF+6).
- **Vocabulary discipline.** `CrisisPhase` transcribes from `src/babylon/domain/economics/tick/types.py` in the
  landed order **`NORMAL, ONSET, EARLY, DEEP, RECOVERY`**. **Enum member order is hash-bearing (ADR195)** — never
  re-group. `defenum` is not shared across scenarios: **every world re-declares it**, and the suite carries one
  ordinal-parity test per mint. This pack mints **no** new `NodeType` or `EdgeType` member.
- **`floor` is ALREADY DECLARED TWICE, and the estate ALREADY has a co-load landmine (#646).** Rev 1's law —
  *"`floor` is declared once, by `territory.bsl:78`"* — is **false at the byte**: `decomposition.bsl:212` carries a
  byte-identical `(intrinsic floor :params (real) :returns int :cost 5)`, and BOTH headers disclose the collision
  (`territory.bsl:67-77`, `decomposition.bsl:30-45`: *"ANY content set that co-loads `decomposition.bsl` with
  `territory.bsl` dies at load with `E-LOAD-001`, a landmine on the Checkpoint A path… Follow-up filed: #646"*) —
  the loader refuses a duplicate BY NAME ONLY, with no content comparison (`declarations.rs:1037-1046` (the doc at `:1037`, the `DeclError::Duplicate` raise at `:1044`; `:1009` is the SignatureMismatch arm rev 1 inherited from `territory.bsl`'s own header — N12, mechanism unchanged)), and
  `load_scenario_with_prelude` is scenario-side only, with no counterpart for rule-file intrinsics. What survives
  as this pack's law: **this pack declares NO intrinsic at all**, so it adds no third `floor` and cannot be the
  cause of an `E-LOAD-001`; if a rule appears to require `floor`, re-derive — the annual gate is `:tick-in-cycle`,
  not modulo (§4.4). **The consequence for Task 10 is structural, not cosmetic: there is no single world that can
  co-load this pack with both `territory.bsl` and `decomposition.bsl`, so §2.3's obligation 2 lands as TWO co-load
  worlds** (§2.3, Task 10 Step 2). D-NF+22 records the corrected hazard and names #646 as the retirement trigger
  that would collapse the two worlds back into one.
- **Six-leg cargo gate per commit** (from `rust/`): `cargo fmt --check`; `cargo clippy --workspace --all-targets --
  -D warnings`; `cargo test --workspace`; `cargo clippy -p babylon-kernel --all-targets -- -D warnings -D
  clippy::pedantic` and the same for `-p babylon-bsl`; `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`;
  `cargo test -p babylon-tick --test tick_goldens --locked`. `mise run rust:check` green after every task.
- **MACHINE SAFETY — heavy runs are SINGLE-FLIGHT, and cargo is CRATE-SCOPED by default.** The box is a 12-core /
  31 GB solo dev box that has been frozen twice. Inner-loop verification uses **crate-scoped** cargo
  (`cargo test -p babylon-tick --test <one test file>`), never `--workspace`, until a task's closing gate. The
  full `mise run rust:check` / `cargo test --workspace` runs **one at a time, never fanned out across parallel
  agents, and never concurrently with a Python `test:unit` leg or with another worktree's gate**. Three sibling
  worktrees (`wt-imperialrent`, `wt-community`, `wt-491`) are live on this box — **check before you start a heavy
  leg**. Each task below states its gate explicitly; a task's gate is a **serial step**, not a background job.
  Parallel agents in this train are for **read-only investigation and doc work only**.
- **After any `docs/reference/bsl-language.rst` edit:** `PYTHONPATH="$PWD/src" UV_FROZEN=1 uv run pytest
  tests/unit/reference/test_bsl_grammar_sync.py -q`. If a register probe reds because a new row cross-references an
  earlier D-code, repair the **test anchor** — never weaken an assertion.
- **Branch from `dev` in an isolated worktree.** The worktree exists at `/media/user/data/worktrees/wt-tickdynamics`
  on `feature/tickdynamics-port-bsl` (PR A). Each later PR branches off **merged dev** — **never stacked** (#193).
  Conventional commits via `mise run commit`; merges only via `mise run pr:merge -- N`, after harvesting the
  Copilot review (ADR181).
- **Token economy:** subagents write artifacts to files and return ≤15-line summaries.

---

## 0. SCOPE RULING — the pack boundary, the no-sliver test, and the honest Checkpoint-A accounting

**This is the plan's single most important section. Read it before disagreeing with the task count.**

### 0.1 Why @4.0 is not one train

`TickDynamicsSystem` @4.0 is **2,558 lines** carrying a nineteen-computation catalog behind two system-wide
blockers: the **external-service/data-source boundary** (33 `ServicesProtocol` fields, §3) and graph-level
opaque-object metadata storage. **No single train can port @4.0.** ADR208 **R15** forbids the obvious escape —
*"the no-sliver precedent BINDS — no honest-partial pack exception; a carrier pilot needs its own evidenced
proposal."*

**The resolution is not a partial pack. It is a correctly-drawn pack boundary.** The estate ADR210 R4–R11 cleared
is not a fragment of one system; it is a **complete, self-contained frozen subsystem** — Feature 016, the class
dynamics engine, living in its own package `src/babylon/domain/economics/dynamics/` (7 modules), behind its own
protocol (`ClassTransitionEngine`), with its own feature spec and its own 686-line behavioral test suite.
`TickDynamicsSystem` is its **caller**, not its home (`system/__init__.py:2424` — one opaque call).

Apply the no-sliver rule's own stated criterion (`reports/register-memos/rows-21-24.md:155-204`, *"it refuses
spending a train on a sliver whose blocked half changes the ported half's behavior"*):

| The criterion | This pack |
|---|---|
| Is the ported scope the whole of a frozen unit's purpose? | **Yes.** Every rate constructor, every flow equation, the amplifier table, the normalize, the cascade beat, and the Φ channel — the complete Feature-016 mechanism. Nothing of the class-dynamics engine is stubbed or dropped. |
| Does a blocked neighbour change the ported half's behavior? | **No.** The engine's entire input surface is `EconomicConditions`' ten fields plus three dispossession rates plus a `CrisisPhase`. Every one becomes **declared content**. Vol II circulation, the Leontief imperial-rent pipeline and the Vol III financial layer never touch it. |
| Is the ported half independently meaningful in play? | **Yes.** A county's labor aristocracy proletarianizes, the cascade beat fires, Φ buys mobility. That is the game's central arc, not a diagnostic. |

**Therefore:** this train ports the class-dynamics engine **completely**, and it **does not claim "TickDynamics
@4.0 is ported."** The residual @4.0 computations get a **named, Director-visible follow-on charter** (Task 11
Step 4) rather than silent omission — which is exactly what R15 exists to prevent.

| Residual @4.0 train | Computations | Gating blocker |
|---|---|---|
| **National parameters** | MELT/γ read + EMA smoothing (Step 2), coefficient smoothing (Step 3b), tick summary (Step 8) | carrier-node scope — ADR198 R6 blessed, **portable now** |
| **County state surface** | bootstrap + carry-forward + literal defaults (Step 3a), precarity derivation, the non-boundary re-stamp, flow accrual (`_accrue_flows`/`_reset_flow_accrual`) | overlaps this pack's field surface; **sequence AFTER this train** |
| **Vol I wage pressure + the accumulation loop** | Step 3.5, Step 3.6 | **the `math.exp` sigmoid** (`reserve_army/calculator.py:44-46` — ADR172 r5 / ADR173: must be re-derived as a MEASURE, never transcribed) **plus the 2 state-affecting `round()` sites** (`reserve_army/accumulation.py:115-123`) |
| **Crisis-phase detection** | Step 5 — the `MultiPeriodCrisisDetector` 5-phase machine + the 4-quarterly-evals-per-boundary loop | a 5-phase state machine on a carrier; **produces this pack's `crisis-phase` input** (BLOCKER-4) |
| **Vol II / Vol III / imperial rent / hex substrate** | Steps 4, 4.5, 5.5, 9 | Leontief externals; the ImperialRent train (#563-adjacent) owns the rent half |

**The `graph_bridge.py` stamping layer does not port — it DISSOLVES.** `stamp_county_attrs_to_territories` and
`write_tick_state_to_graph` exist only because the frozen engine keeps tick state in a Pydantic tree and mirrors it
onto nodes. **In BSL the declared node fields ARE the storage; there is no second copy to synchronize.** The
inventory's "dominant blocker #2" is a **non-computation** in the target estate, not an omission. **D-NF+2, with
the argument written out** — this is the single largest cost reduction in the whole @4.0 program and it must be
recorded, not assumed.

### 0.2 The Checkpoint-A accounting — stated plainly, because the charter's framing and the measured roster differ

This train is chartered as "**the 13th and FINAL Material Base system**, whose landing closes Checkpoint A (ADR208
R14) and fires WS3." **The measured roster in this worktree does not yet support that claim, and the plan says so
rather than inheriting it.** `rust/crates/babylon-tick/src/lib.rs:277-343` registers **13 system namespaces**, of
which nine are Material Base ports (`vitality`, `territory`, `production`, `lifecycle`, `solidarity`,
`dispossession`, `decomposition`, `control-ratio`, `metabolism`); `economics`, `consciousness`, `organization` and
`social-class` are not Material Base ports. **`tick-dynamics` in any spelling has zero hits.** Against
CLAUDE.md's Material Base list (positions 1–13: Vitality, Territory, Production, TickDynamics, ReserveArmy,
Community, Lifecycle, Solidarity, ImperialRent, Dispossession, Decomposition, ControlRatio, Metabolism):

| system | position | status at this HEAD |
|---|---|---|
| Vitality, Territory, Production, Lifecycle, Solidarity, Dispossession, Decomposition, ControlRatio, Metabolism | 1,2,3,7,8,10,11,12,13 | **PORTED** (9) |
| Community | 6.0 | **IN FLIGHT** — `wt-community`, 13 tasks, unmerged |
| ImperialRent | 9.0 | **IN FLIGHT** — `wt-imperialrent`, unmerged |
| **ReserveArmy** | **5.0** | **UNPORTED AND UNSTARTED** — ADR210 **R2** names its own train ("the first `reserve_army` .bsl ships without the valve"); no registration, no plan in any worktree |
| **TickDynamics** | **4.0** | **THIS TRAIN — and only its Feature-016 half (§0.1)** |

**Two consequences this plan carries openly.** (1) Checkpoint A is **not** closed by this train, both because
ReserveArmy @5.0 is unstarted and because §0.1's boundary leaves five residual @4.0 trains. (2) **WS3 therefore
stays HELD** under R14's own words — "an early sweep catches six systems' workarounds and misses seven." Task 11
Step 4 updates #557's Checkpoint-A accounting to the measured roster and files the residual issues; **§10 DG-10**
puts the question to the Director as a popup rather than letting a plan decide what R14 counts.

> **REVIEWER GATE.** If the reviewer rejects §0.1's boundary and rules that R15 requires all of @4.0 in one train,
> **STOP** — do not shrink the pack to fit. The correct response is an ADR amending R15's application to compound
> systems, or a Director escalation. **Do not improvise a middle.**

---

## 1. Frozen-source archaeology — the transcription source

Verified by direct line-by-line read of all seven `dynamics/` modules plus the call site. **Transcribe from here
and re-verify against the cited lines at the moment of writing each rule.**

### 1.1 The state object — `dynamics/types.py:27-139`

Frozen Pydantic, five `float` shares each `ge=0.0 le=1.0`; a `model_validator(mode="after")` enforces sum-to-one at
tolerance `0.001` (`:70-83`). `year: int = Field(ge=2007, le=2030)` (`:57`) — **the only year field in the tick
tree that still hard-enforces the 2030 ceiling**, which is why every construction site pre-clamps.
`dynamic_shares()` returns `(LA, proletariat, lumpen)` (`:100-110`). `with_updated_dynamics(la, prol, lumpen)`
rebuilds the model preserving bourgeoisie + petit-bourgeoisie and **incrementing `year` by 1** (`:131-139`).

Bootstrap `0.01 / 0.09 / 0.40 / 0.35 / 0.15` — **four duplicate literal sites** (rev 1 counted three and missed
the fourth): `system/__init__.py:486-490`, `:822-830`, `graph_bridge.py:373-377`, and
`tick/initializer.py:31-37`'s named `_DEFAULT_BOURGEOISIE` … `_DEFAULT_LUMPENPROLETARIAT` block (`:34` is
`_DEFAULT_LABOR_ARISTOCRACY: float = 0.40`). **R5 confirms the values verbatim.** Finding F18 proves `0.40 ≡ 0.90 − 0.50`
is percentile-band arithmetic exactly, and **R7 rules the percentile reading the defect**: the five values survive
as **SEEDS**, the band descriptions (`"Top 1% wealth share"`, `"90th-99th percentile share"`,
`"50th-90th percentile share"`, `"Bottom 50% employed share"`, `"Bottom 50% excluded share"`, `types.py:37-41`,
`:58-68`) **die and are transcribed nowhere**.

### 1.2 The six-step engine — `dynamics/transition_engine.py:107-198`

1. **Accumulation rate** (`:133-140` → `_convert_accumulation_to_rate`, `:200-217`):
   `0.0` if `annual_accumulation <= 0.0`, else `min(annual_accumulation / 142_000.0, 0.08)`.
   `annual_accumulation` comes from `accumulation.py:85-90`:
   `base_rate = savings[PROLETARIAT] = 0.03` (`savings_schedule.py:25`; the call site hardcodes
   `ClassPosition.PROLETARIAT`, `transition_engine.py:136`); `phi_adj = min(phi_hour · 2080 / wage, 0.05)` with
   `wage == 0.0 or phi_hour == 0.0 → 0.0` (`savings_schedule.py:90-92`).
   **THE UNIT OF `wage` IN THAT EXPRESSION IS ANNUAL, AND EVERY DOWNSTREAM CLAIM DEPENDS ON IT.** The byte trace:
   `system/__init__.py:2378` `effective_wage = county.median_wage * HOURS_PER_YEAR` (hourly → annual), zeroed by
   the FR-017 halt at `:2379-2380`; `:2402` `EconomicConditions(median_wage=effective_wage, …)` — the field now
   holds the ANNUAL figure; `transition_engine.py:133-137` passes `conditions.median_wage` as `wage`;
   `accumulation.py:86` hands that same `wage` to `get_phi_adjustment` (`:85` is the `base_rate` line; the whole
   computation is `:85-90`, N6); `savings_schedule.py:92` divides by it.
   **So the frozen quantity is `min(phi_hour · 2080 / (median_wage_hourly · 2080), cap)` — the 2080 cancels — and
   the zero guard tests the HALTED wage, so every county below the $9.60/hr floor gets `phi_adj = 0` on the same
   branch.** Rev 1's Φ rule divided by the raw hourly field, a **2080× error** that saturated the cap for any
   `phi_hour > median_wage/41600` and collapsed R9's ratified gradient into a switch (§7's rule is corrected; the
   correction is why §7a's `a01_and_phi_coupling_agree_on_the_wage_base` row is writable at all);
   `effective_savings = min(base_rate + phi_adj, 1.0)`; `consumption = wage · (1 − s)`;
   **`accumulation = (wage − consumption) · s ≡ wage · s²`** — the **F11 defect**, docstring-confirmed at
   `accumulation.py:40-41` ("*Which simplifies to: annual_accumulation = wage \* effective_savings^2*"),
   **33× understated at the proletariat's 0.03 rate**, **repaired at the port** per R9 + R11.
2. **Dispossession rate** (`:143-146`): `disp_calc.compute(fips, year)`; a `NoDataSentinel` **aborts the whole
   transition** by early return; otherwise `la_to_p_rate` from `dispossession.py:102-106`:
   `0.6·foreclosure + 0.3·bankruptcy + 0.1·eviction`. `p_to_l` (`:107-111`, weights `0.1/0.3/0.6`) is **computed,
   returned, and never read anywhere** — the register-row-24 category (D-NF+11).
3. **Precaritization** (`:219-236`): `clamp(u · 0.5 + eviction · (1 − 0.5), 0, 1)`. **The constant named
   `_DEFAULT_EVICTION_WEIGHT` multiplies UNEMPLOYMENT, not eviction** (`:233-235`) — a naming defect; the
   arithmetic transcribes exactly, the name does not (D-NF+14).
   **Stabilization** (`:238-253`): `clamp(0.15 · (1 − u), 0, 1)`. The constant is `0.15` (`:53`); the docstring says
   `0.10` **twice** (`:74`, `:98`) — **F13**; disambiguate at the port, never transcribe the pair (D-NF+13).
4. **Crisis amplification** (`:153-165`): a `TransitionRates` model (all four fields `ge=0.0 le=1.0`,
   `types.py:209-212`) passed through `amplify_phased(rates, phase)` **if the collaborator has that attribute**,
   else `amplify(rates, conditions.crisis)` — selected by a **runtime `hasattr` duck-type check** (`:162`).
   `PhasedCrisisAmplifier.amplify_phased` (`crisis.py:153-178`) applies the FR-006 table (`crisis.py:24-55`), each
   product `min(·, 1.0)`:

   | phase | dispossession | precaritization | accumulation | stabilization |
   |---|---|---|---|---|
   | NORMAL | 1.0 | 1.0 | 1.0 | 1.0 |
   | ONSET | 1.2 | 1.5 | 0.8 | 0.7 |
   | EARLY | 1.8 | 2.5 | 0.4 | 0.4 |
   | DEEP | **3.0** | **3.5** | **0.1** | **0.2** |
   | RECOVERY | 1.3 | 1.2 | 0.6 | 0.5 |

   The DEEP row is the one the rulings dossier's **open question 4** flags as *"how sharply crisis proletarianizes
   … pedagogy, not calibration"* — **ported as ruled content, magnitude question routed to §10 DG-4.**
   `DefaultCrisisAmplifier` (the legacy `2.5` / `0.3` path, `crisis.py:20-21`, `:58-109`) is **dead once the
   duck-type check is gone** — recorded, not ported (D-NF+6).
5. **Flow equations** (`_apply_flows`, `:255-289`):
   ```
   LA'     = LA     − disp·LA     + acc·Prol
   Prol'   = Prol   + disp·LA     − acc·Prol − precar·Prol + stab·Lumpen
   Lumpen' = Lumpen + precar·Prol − stab·Lumpen
   ```
   **F10: exactly mass-conserving** — every term cancels, so `LA' + Prol' + Lumpen' ≡ LA + Prol + Lumpen` in exact
   arithmetic. This is what makes BLOCKER-5 cheap (§9).
6. **Clamp and normalize** (`_normalize`, `:291-331`): `max(·, 0)` each share (`:313-315`); `total_dynamic`;
   `target = 1.0 − fixed_share`; if `total_dynamic > 0` rescale by `target / total_dynamic` (**≡ 1.0 by F10 — a
   floating-point re-anchor, not a correction**); **else assign `target / 3.0` to each** (`:326-329`) — the
   arbitrary equal-thirds reset **R11 names for repair** (D-NF+8).

Then `validate_class_shares` logs (`:188-192`) and `with_updated_dynamics` returns.

### 1.3 The call site — `system/__init__.py:2346-2458`

- `:2366-2367` `if services.transition_engine is None: return county_states` — a **whole-step** no-op.
- `:2369` `floor_ratio = services.defines.crisis.wage_compression_floor_ratio` = **0.8**.
- `:2374` `clamped_year = min(max(county.year, 2007), 2030)`; `:2375` `crisis_phase = county.crisis_state.phase`.
- `:2378` `effective_wage = county.median_wage · HOURS_PER_YEAR` (**2080**, `formulas/constants.py`).
- `:2379-2380` `if should_halt_accumulation(county.median_wage, DEFAULT_V_REPRODUCTION, floor_ratio):
  effective_wage = 0.0`. `should_halt_accumulation` is **`wage < subsistence · floor_ratio`, strict `<`**;
  `DEFAULT_V_REPRODUCTION = 12.0` (`system/__init__.py:104`) — the halt floor is **exactly $9.60/hr** (FR-017).
- `:2382-2396` dispossession rates seeded from module defaults **`0.006 / 0.006 / 0.063`**
  (`system/__init__.py:107-109`), each overridden only if the source is wired **and** its getter returns non-`None`.
  **Defaults are behavior, not fallbacks** — they are declared content in the port.
- `:2398-2409` `EconomicConditions(...)` — ten fields; `crisis = crisis_phase != CrisisPhase.NORMAL` (the 5-phase
  enum collapsed to a bool **and** passed separately as the full enum at `:2424-2428`), and
  **`melt = national_params.tau` is never read by the engine** — a dead input, dropped with a D-row.
- `:2412-2422` distribution-year re-clamp (verbatim-share rebuild); `:2430` `if result and isinstance(result,
  ClassDistribution)`; `:2432-2442` result-year re-clamp; `:2444-2452` the cascade check; `:2454` write-back;
  `:2456` **`else` → county unchanged** — **F15: this output is byte-identical to "transitions ran and produced no
  net change"** (D-NF+10).

### 1.4 The cascade — `system/__init__.py:1115-1170`, call site `:2444-2452`

```python
baseline_la = prev_county.class_distribution.labor_aristocracy_share   # :1140  ← the PREVIOUS BOUNDARY
current_la  = new_dist.labor_aristocracy_share                        # :1141
decline     = baseline_la - current_la                                # :1142
if decline <= 0: return                                               # :1144-1145
crossed = None
for milestone in sorted(milestones):                                  # :1149
    if decline >= milestone: crossed = milestone                      # :1150-1151  ← HIGHEST crossed wins
```
Payload (`:1153-1170`): `fips`, `cumulative_la_decline = round(decline, 6)`, `milestone_crossed`,
`current_la_share = round(·, 6)`, `baseline_la_share = round(·, 6)`. Milestones `[0.05, 0.10, 0.15]`
(`config/defines/economy_basic.py:136-140`). Three gates: `crisis_phase != NORMAL`, `prev_county_states` truthy,
`prev_county_states.get(fips) is not None`.

**F19, proved analytically:** with `baseline_la` = the previous boundary and F11's near-zero accumulation, the
maximum single-boundary decline is `disp · la`; inside the engine's own EXPECTED envelope
(`validation.py:30`, `:61`) that ceiling is **~2.5pp — half the smallest milestone**. The event has never fired in
any committed artifact for an **arithmetic** reason. **R10 restores the cumulative baseline**, under which F20's
4–13pp over nine boundaries spans 5pp and 10pp and **R5's confirmed constants and R5's confirmed highest-only rule
are simultaneously correct**. That reconciliation is R10's entire justification and Task 7 makes it executable.

### 1.5 The boundary, the year, and the quarterly loop

Gate `if tick % WEEKS_PER_YEAR != 0` (`:174`) — **the whole 8-step annual pipeline is skipped on 51 of every 52
ticks**. Boundary year: `existing_state.year + 1` (`:207`) when state exists, else `_determine_year` =
`graph.get_graph_attr("base_year", 2010) + tick // WEEKS_PER_YEAR` (`:423`). Year is clamped
**two-sidedly** `min(max(y, 2007), 2030)` at **six sites tree-wide — re-measured 2026-08-18 and enumerated here
because rev 1 asserted the count without them**: `system/__init__.py:482`, `:810`, `:824`, `:2374`, `:2432`, and
`tick/initializer.py:160`. **TWO of the six are inside this pack's boundary** (`:2374` the pre-transition clamp,
`:2432` the result clamp) — which is what the trio dossier's *"the caller clamps to `[2007,2030]` **twice**"*
(`reports/tickdynamics-trio-dossier-2026-08-17.md:694`) means, and the two figures do not conflict. The other
`2007` hits are **floor-only** `max(year, 2007)` (`:496`, `:617`, `:741`, `:2543`, `initializer.py:195`) or
Pydantic `Field` constraints, and are not clamp sites. The `[2007, 2030]` pair is a hardcoded Pydantic `Field`
constraint, **not a define**. `_check_crisis_triggers` internally loops **4 "quarterly" evaluations per single annual boundary
call** (`quarterly_evals = 4`, `:972`) — **out of this pack's boundary** (§0.1), and the reason `crisis-phase` is
declared input rather than computed.

### 1.6 The coefficient inventory — MEASURED, and the dossier's figure is CORRECTED

The trio dossier's finding F12 gives the transition engine's hardcoded module-level constant estate as "≈66", split
"`validation.py` — 17 rate thresholds + 12 share thresholds = 29". **Re-measured directly in this worktree on
2026-08-18: `validation.py` carries 32 module-level float constants (20 rate + 12 share), not 29** — the line
ranges cited (`:29-54`, `:60-73`) are right, the counts are not. Corrected inventory:

| module | constants | count |
|---|---|---|
| `transition_engine.py:51-54` | wealth threshold `142_000.0`, eviction weight `0.5`, base stabilization `0.15`, max accumulation `0.08` | 4 |
| `crisis.py:20-21, 24-55` | 2 legacy multipliers + 5 phases × 4 multipliers | 22 |
| `dispossession.py:30-36` | 6 composite weights (3 LA→P, 3 P→L) | 6 |
| `savings_schedule.py:21-27, 30` | 5 class savings rates + `phi_cap` | 6 |
| `validation.py:29-54, 60-73` | **20** rate thresholds + 12 share thresholds | **32** |
| **total** | | **70** |

**R8 rules the CLASS, not the arithmetic** — the correction changes no ruling. **Zero of the 70 are in
`defines.yaml`/`GameDefines`** (direct grep for `tick_dynamics`/`tickdynamics` in `src/babylon/data/defines.yaml`
returns **zero matches**), which is exactly what makes them F12 and exactly what R8 disposes. §4.5 gives the
70-row disposition arithmetic (**29 live defconsts + 9 recorded-not-declared + 32 conformance bounds = 70**).

**Separately and not to be conflated:** `TickDynamicsSystem`'s **own** `GameDefines` reads are **30 distinct
coefficients** (24 direct dotted fields + 6 one-hop `reserve_army` fields) across five pre-existing categories —
a different axis entirely, and **26 of the 30 are outside this pack's boundary**. The four that are inside are
`crisis.dispossession_cascade_milestones`, `crisis.bifurcation_event_threshold`,
`crisis.wage_compression_floor_ratio`, and (post-R6) nothing else; each becomes a `defconst` in §4.5.

---

## 2. THE BOUNDARY — what is already landed, what this pack owns, and the cross-train disjointness proof

**Source of law: the landed-boundary dossier (grounding reader 3/4), whose §2 qname census and §3 ImperialRent
overlap table are quoted per row below. Every rule in §7 cites its row here.**

### 2.1 What landed content ALREADY writes — and therefore what this pack must NOT

| qname | landed writer | this pack's relation |
|---|---|---|
| `social-class/imperial-rent` | `economics/fundamental-theorem` (only) | **reads nothing of it in this train**; §2.2's rule is additive to the same FILE, not to this field |
| `social-class/wage-balance` | `consciousness/p4-wage-balance` | **never written here** |
| `social-class/agitation` | `consciousness/p5-agitation`, `p6-route` | **never written here** |
| `social-class/revolutionary`, `/liberal`, `/fascist` (the ternary) | `consciousness/p0-position` (seed `(0,1,0)`), `consciousness/p6-route` (routed) | **READ ONLY.** `p6-route`'s own header names itself *"the ADR016 bifurcation law RE-POINTED at the stored ternary — the headliner"* — **ADR016's law is ALREADY LIVE there, not in TickDynamics.** This is R6's premise, executed. |
| `social-class/dominant-worldview` | `consciousness/p8-dominant-worldview` ("ONE DECLARED HOME") | **never written here**; this pack adds no second home for any readout |
| `social-class/solidarity-inbox`, `/wages-inbox`, `/previous-wages`, `/previous-wealth` | `consciousness/p1`–`p3`, `p2-wages-push`, `p7-persist-baselines` | **never written here**; `p7`'s *pattern* is reused (R10), its *fields* are not touched |
| `social-class/la-census-*`, `/la-approaching-flag`, `/la-dying-flag` | `decomposition/p01-la-census` | **never written here** |
| `institution/superwage-crisis-known`, `/-tick` | `decomposition/p02-superwage-warning` **and** ImperialRent's `r05` | **never touched here** — the latch belongs to `decomposition/` and `imperial-rent/`; nothing in R9's scope requires it |
| `institution/la-*`, `/decomposition-*`, `/enforcer-*`, `/ip-*` | `decomposition/p03-trigger` | **never written here** |
| `social-class/population`, `/wealth`, `/active` | `decomposition/p04`–`p06`, `production/p2-employed-routing` (**RESERVED LINE**) | **`population` READ ONLY** (the R6 fold's weight); `wealth`/`active` untouched |
| `social-class/wages-paid`, `/value-produced`, `wages/value-flow` | **scenario-seeded fixtures today**; ImperialRent's `r06-wages-pay` becomes their **first writer** | **never written here** |

**Net verdict, quoted from the dossier:** *"What remains genuinely unclaimed is the class-mobility/transition-engine
math itself: the five-share `ClassDistribution` taxonomy, the flow equations, the savings/accumulation channel, and
the dispossession-cascade milestone arithmetic."* That set **is** this pack.

### 2.2 `fundamental-theorem.bsl` — the ratified home, its FOUR consumers, and how the R9 edit lands without moving a pin

The file is **12 lines**, one rule (`economics/fundamental-theorem`), two required bindings
(`social-class/wages`, `social-class/value-produced`), one guard (`wages > value-produced`), one write
(`social-class/imperial-rent := wages − value-produced`). **No `defconst`s. No Φ coupling of any kind.**
`rg -n -i 'savings|phi_cap' rust/crates/babylon-tick/content/` returns **zero hits** at this HEAD (the same grep
widened with `mobility` returns **7**, all of them `lifecycle.bsl:16,32,59,62,79` and
`lifecycle_conformance.py:18,22`, none Φ-related — rev 1's "zero hits" was written against the wider pattern and
is corrected here rather than quietly dropped).

R9 homes the Φ → savings → LA circuit **here**, and the ImperialRent plan states in its own words that it does not
touch it: *"When D6-A lands it will extend `fundamental-theorem.bsl` with **TickDynamics** content. `phi_cap` stays
a BSL **DEFCONST** when that happens — **on that train, not this one**."* **This train is that train.**

#### 2.2.1 The file is NOT private to this train — four consumers, verified by `include_str!`

| consumer | line | what it does | what it asserts |
|---|---|---|---|
| `rust/crates/babylon-tick/tests/tick_goldens.rs` | `:35`, `:60-75` | `two_classes_fundamental_theorem_hashes_are_pinned` | `hex(before)` = `5a44ab0c…a205`, `hex(after)` = `783f651d…7679`. **No `fired` assertion.** |
| `rust/crates/babylon-client/src/engine_link.rs` | `:16-26` | the shipped Bevy client's startup `engine_link_probe()` | — |
| `rust/crates/babylon-client/tests/engine_link.rs` | `:1-14` | `startup_tick_matches_the_pinned_hash` — **the 17th pinned hash, in another crate** | `hex(after)` = `783f651d…7679` **only** |
| `rust/crates/babylon-tick/src/lib.rs` | `:549-596` | three unit tests over the same pair | `run_once_is_deterministic` (`a.after == b.after`, `before != after`); **`single_rule_content_still_reports_fired_and_a_one_entry_per_rule_fired` (`per_rule_fired.len() == 1`)**; `node_content_ids_reach_prepared_rules_through_the_real_wiring_seam` |

`babylon-bsl/tests/fundamental_theorem_tick.rs` carries its **own inline copy** of both the scenario and the rule
(`:56-75`) and does **not** `include_str!` either file — it is unaffected, and that is verified, not assumed.

The scenario all four load is `content/scenarios/two-classes.bscn` — **18 lines, three `deffield`s (all
`social-class/*`), two SOCIAL_CLASS nodes, zero `(defconst …)`, zero TERRITORY nodes.**

#### 2.2.2 Why the naive edit dies at LOAD — two independent, byte-verified refusals

1. **`E-LOAD-010`, binding resolution.** `resolve_bindings` refuses a `:field`/`:const` qname the scenario's
   vocabulary does not hold (`bindings.rs:458-494`). **`:optional` + `:default` does NOT rescue it** — the
   function's own doc is explicit: *"the source must resolve for EVERY binding — `:optional` licenses per-node
   absence of a VALUE, never an unknown NAME"* (`bindings.rs:448-451`), and the module header says the same
   (`:3-7`). **So the field-presence-guard escape does not exist in this language**, and any plan that assumes it
   is wrong; the only lever is the scenario's declaration set.
2. **`check_sources_servable`**, called unconditionally at `run_tick` entry (`tick.rs:583`) **before** the subject
   loop, so an empty TERRITORY population is no escape: `BindSource::Const(qname) if !defines.contains_key(qname)`
   → refusal by name (`tick.rs:439-451`). `two-classes.bscn` supplies no defconsts.

And `run_prepared_tick` propagates the first rule error with `?` (`lib.rs:517-531`) — one bad rule kills the whole
tick, not just itself. **Rev 1 scheduled exactly this edit with no mitigation; that is C1.**

#### 2.2.3 THE RESOLUTION — a DECLARATION-ONLY extension of `two-classes.bscn`, which is hash-neutral by the canonical layout

Both laws hold simultaneously. R9 lands **verbatim** — the circuit homes in `fundamental-theorem.bsl`, `phi_cap`
is a BSL `defconst` — and **all 17 pre-existing pinned hashes stay byte-identical**, because the rows the scenario
gains are **declarations, and declarations are not graph state**:

- **The state hash covers graph state and nothing else.** `babylon-graph/src/state_hash.rs:10-30` specifies the
  canonical byte layout normatively: section `0x01` nodes, `0x02` node attributes, `0x03` edges, `0x04`
  hyperedges, `0x05` edge attributes. **A `deffield` row and a `defconst` row appear in NO section** — they
  populate the `TypeEnv` and the `DefinesEnv`, which `ContentDigest` (a different, unpinned fingerprint)
  covers, not `state_hash`.
- **Hydration writes only what a node explicitly seeds.** `scenario.rs:1236-1275` adds the node, then loops the
  node's own `(field value)` pairs; a declared-but-unseeded field is **never stamped**, and there is no
  default-materialization path. Adding `territory/*` declarations to a world with zero TERRITORY nodes therefore
  writes zero attributes.
- **The new rule fires zero times.** `subject_type_of` derives the subject NODE type from the rule's single
  `:field` binding namespace (`tick.rs:166-189`) — `territory/…` → `TERRITORY` — and
  `graph.nodes("TERRITORY")` on this world returns an empty vec (`hypergraph_store.rs:310-319`: a filter, never
  an error, never a refusal for an unpopulated type). Zero subjects ⇒ zero writes ⇒ `after` unmoved.

**What `two-classes.bscn` gains (declaration-only, no node form, no edge form, no attribute):** the `deffield`
rows for the fields the Φ rule binds and writes — `territory/median-wage`, `territory/phi-hour`,
`territory/phi-savings-adjustment` — and the `defconst` rows it reads — `class-dynamics/phi-cap`,
`class-dynamics/hours-per-year`, `class-dynamics/v-reproduction`,
`class-dynamics/accumulation-halt-floor-ratio` (the last two because the corrected rule shares `a01`'s
halt-gated wage base, §7). Each row carries a one-line comment naming this train, the rule that needs it, and
**the fact that this world declares them without seeding them because it holds no territory** — the honesty the
file's own header style already uses.

**The ONE Rust-source consequence, budgeted rather than discovered:**
`babylon-tick/src/lib.rs`'s `single_rule_content_still_reports_fired_and_a_one_entry_per_rule_fired` asserts
`report.per_rule_fired.len() == 1`, and the pair now holds **two** rules. Task 9 Step 2 updates it to `== 2`,
renames it to drop the now-false "single_rule_content" premise, and **keeps the property it exists to pin** (the
per-rule breakdown sums to `report.fired`) plus a new assertion that the second rule contributes **0** — which is
the executable form of this whole subsection. **This is a test repair, not a pin move and not a baseline motion.**

#### 2.2.4 Rejected alternatives, with their costs

| alternative | why rejected |
|---|---|
| **Seed a TERRITORY node into `two-classes.bscn`** so the rule has a subject | Moves the pre-tick hash `5a44ab0c…` and the post-tick hash `783f651d…` — **a STOP under this plan's own law, and it breaks `babylon-client` too.** Also fabricates a county in a world whose whole purpose is two classes. |
| **Split the Φ rule into its own `.bsl`** | Forfeits R9's *ruled home*. The ruling says "homed in `fundamental-theorem.bsl`", and a sibling file is a dilution the workforce may not decide (§Global's rulings-are-law bullet). **If a reviewer prefers the split, that is a Director escalation, not an implementation choice.** |
| **Guard the rule with `:optional`/`:default` so it no-ops in worlds not declaring the fields** | **Not expressible.** `resolve_bindings` refuses the unknown qname before optionality is consulted (`bindings.rs:448-451`), and `check_sources_servable` refuses the unsupplied `:const` at `run_tick` entry. |
| **Move the golden to a different scenario** | Re-pinning by relocation is re-blessing without a ceremony. |

**The edit is therefore: one new rule in the `economics/` namespace, the file's first `D-N` header block, the
declaration-only rows in `two-classes.bscn`, and one Rust unit-test repair.** D-NF+29 records the whole motion.

### 2.3 CROSS-TRAIN WRITE-SET DISJOINTNESS — the proof, per concurrent train

The three trains may land in **any order**. Disjointness is proved by construction, not by sequencing:

| concurrent train | its claimed WRITE set | overlap with this pack |
|---|---|---|
| **ImperialRent** (`wt-imperialrent`, 10 rules, `imperial-rent/`) | `social-class/wages-paid`, `/value-produced`, `/class-consciousness`, `wages/value-flow`, `institution/superwage-crisis-known`/`-tick`, `institution/rent-*` | **∅.** This pack writes no `social-class/` field except the net-new `ternary-net-fascist`, and no `institution/` field at all. ImperialRent's B1/B2/B6/B10 negatives confirm it never writes `social-class/imperial-rent`, `/wages`, `/wage-balance`, `/wages-inbox`, `/agitation`, and it declares **no** Phase-4 subsidy vocabulary. |
| **Community** (`wt-community`, 14 rules, `community/`) | `community/*` hyperedge fields, `institution/community-*` on the singleton carrier, `social-class/community-cost-modifier` | **∅.** This pack mints no hyperedge, declares no `community/` qname, and **anchors no rule on `NodeType/INSTITUTION`** — so it neither adds nor requires a carrier and cannot double-apply Community's carrier-subject rules. |
| **#491** (rung/ladder) | register rows only | **∅** on content; **contended on numbering only** (D181 already committed in its worktree) — handled by the NEXT-FREE-AT-LANDING law. |
| **This pack** | `territory/*` (**17 declared + 7 staging**, §4.2/§4.2.1 — the third stale "16" M4 named, N8) + **one** net-new `social-class/ternary-net-fascist` | — |

**Two mechanical obligations that make the proof real rather than asserted.**
1. **Task 0 Step 4 greps `rust/crates/babylon-tick/content/` and both sibling worktrees' plans for
   `ternary-net-fascist` and for every `territory/` qname in §4.2.** Zero hits is the precondition. **Task 12
   re-runs the identical grep against `dev` at filing time.** A hit is a STOP, and the resolution is a rename in
   *this* pack (the newest claimant yields), never a silent co-write.
2. **Task 10 lands TWO CO-LOAD worlds, because ONE is impossible at this HEAD.** `territory.bsl:78` and
   `decomposition.bsl:212` both declare `(intrinsic floor …)`, so **any content set holding both dies at
   `E-LOAD-001` before this pack is even consulted** (#646, disclosed in both headers — §Global). Rev 1 specified
   a single world loading both and asserted it would prove *"no duplicate `floor` declaration"*; that world cannot
   load, which makes the obligation undischargeable as written. The repair splits it, and **names what each world
   can and cannot prove**:
   - **Co-load world A — `class-dynamics.bsl` + `consciousness.bsl` + `production.bsl` + `territory.bsl`.** Proves
     (a) no rule-id collision; (b) no III.11 hard error from a subject-type rule meeting a node lacking a bound
     field; (c) `a12`'s `social-class/ternary-net-fascist` write does not disturb `consciousness/p6-route`'s
     ternary; (d) the pack produces the SAME `territory/*` values under co-load as alone; (e) **every TERRITORY in
     a foreign-shaped world survives `a13`'s fold** (§4.6's `exists` protector, the C5 fix) — this is the world
     most exposed to it, since `territory-conformance`-shaped territories carry no TENANCY-incident class.
   - **Co-load world B — `class-dynamics.bsl` + `consciousness.bsl` + `decomposition.bsl`.** Proves the same list
     against the OTHER `floor`-declaring pack, and additionally that this pack's `social-class/population` READ
     coexists with `decomposition/p04`–`p06`'s writes to it.
   - **What the split loses, stated plainly:** there is **no** world on this train proving `class-dynamics.bsl`
     co-loads with `territory.bsl` and `decomposition.bsl` **simultaneously** — because no such world can exist
     for any pack until #646 lands. **That is a pre-existing estate defect this train inherits and does not
     cause**, and it is recorded in D-NF+22 with #646 as the named retirement trigger. Neither world proves
     anything about `floor`; this pack declares no intrinsic, and §7c guard 2 asserts that at source level.
   - **Both worlds re-assert the 17 pre-existing pinned hashes** (16 in `tick_goldens.rs` + the `babylon-client`
     pin) as part of the same gate.

### 2.4 The one relation this pack reads across the county↔class boundary — and the precedent named CORRECTLY

R6's readout folds the county's classes. The landed relation is the **`EdgeType/TENANCY` edge**. **Rev 1 cited
this shape to `production.bsl` and to "D45", and both citations are wrong at the byte — rev 2 replaces them with
the real record and makes the argument rev 1 substituted a citation for.**

- **`production.bsl` contains no fold at all** — its own header says so: *"`… total` via a plain `:field` binding,
  **no fold anywhere in this pack**"* (`production.bsl:145`). Its `:135` is a comment explaining why a naive
  territory-side TENANCY fold **could never load**, and `:169-174` is a `select-max`, not a fold.
- **D45 is the `select-max`/`select-min` extremising-element tiebreak** (ascending-id first-wins,
  `docs/reference/bsl-language.rst:5103-5108`; `evaluator.rs:143-147` reuses its code for the empty case). It is
  what *avoids* an ambiguity, not a double-count hazard.
- **The double-count record is D136** (`production.bsl:83-107`): an earlier draft computed
  `territory/extraction-intensity` with a territory-side `fold sum` that **double-counted `worker-pp-two-lands`**;
  the fix **deleted the territory-side fold**, and that draft's register row additionally claimed no `.bsl`-level
  fix existed — *"FALSE, caught by adversarial verification"*.
- **The argument this pack actually owes, made rather than cited.** D136's hazard is specific to a **sum**: a
  class incident to two territories contributes its whole mass to both, inflating a conserved quantity. R6's
  readout is a **population-weighted mean**, which is not a conserved quantity and is not inflated by dual
  membership — a class tenanted in two counties is genuinely a member of both counties' class composition, and
  each county's mean should include it with its own population weight. **The shape is therefore correct here and
  wrong there, for a reason, and the fixture pins the reason:** world 1's `shared-class` (two TENANCY edges) is
  asserted to appear in BOTH counties' means with identical weight, and the companion assertion is that neither
  county's mean equals what it would be with the class excluded. D-NF+23 carries it, citing **D136** and D45
  correctly.

---

## 3. The 33-field `ServicesProtocol` boundary — every field disposed

`ServicesProtocol` (`src/babylon/kernel/services.py:34-88`) is **6 core + 46 optional = 52 fields**, of which
**everything except `bea_industries: list[str] | None` and `event_bus: EventBus` is typed bare `Any`**.
`TickDynamicsSystem` + its `imperial_rent.py` delegate read **33** of them (re-verified independently in this
worktree: 27 distinct `services.X` reads in `system/__init__.py`, 5 `imperial_rent.py`-only fields, plus
`credit_aggregate_source` via `getattr` at `:1845`). **That `Any` convention does not survive into the port
undeclared — every one of the 33 is dispositioned here.**

**Tally: 6 PORT · 1 ALREADY-LANDED · 1 DISSOLVES · 25 DEFER-WITH-D-ROW = 33.**

| # | field | disposition | landing |
|---|---|---|---|
| 1 | `transition_engine` | **PORT — THE SUBJECT** | `class-dynamics.bsl`'s 13 rules. One opaque call at `system/__init__.py:2424` becomes declared content. |
| 2 | `defines` | **PORT** | 46 named `defconst`s (§4.5). The four in-boundary `crisis.*` reads become `class-dynamics/*` constants; **no `defines.yaml` motion** (R8). |
| 3 | `unemployment_source` | **PORT** | `territory/unemployment-rate` (`probability intensive`); the frozen unwired bootstrap `0.05` (`:766`) becomes a seeded value, **not a hidden default**. |
| 4 | `wage_source` | **PORT** | `territory/median-wage` (`real intensive`); the frozen bootstrap `21.0` (`:784`) becomes a seeded value. |
| 5 | `tensor_registry` | **PORT — one field only** | Only `phi_hour` reaches the class-dynamics engine → `territory/phi-hour` (`real intensive`). Every other registry read (profit rate, organic composition, departments, county surplus) is Vol I/II/III — **deferred**. Its *compound* None-guard (`tensor_registry is None AND dispossession_source is None`, `:1287`) belongs to the accumulation-loop train. |
| 6 | `dispossession_data_source` | **PORT** | `territory/foreclosure-rate`, `/bankruptcy-rate`, `/eviction-rate` (`probability intensive`) **plus three default `defconst`s `0.006 / 0.006 / 0.063`** — defaults are behavior (§1.3). |
| 7 | `event_bus` | **ALREADY-LANDED** | The landed `emit` verb + `CollectingSink`. `EventType` is a kind-checked closed vocabulary (`vocabulary.rs:39-71`) that a scenario opts into per-kind; **zero Rust changes**, and both of this pack's events are assertable key-by-key today (`dispossession_conformance.rs:139-200` idiom). |
| 8 | `economics_fallbacks` | **DISSOLVES (D-NF+10)** | 9 call sites tallying graceful degradation (`:535,545,564,579,588,1967,2036,2055,2064`). **Constitution III.11 / invariant S-11 has no warning level and no degraded mode to port them into.** The absence is a ruling, not an omission. |
| 9–33 | `melt_calculator`, `basket_calculator`, `gamma_calculator`, `capital_calculator`, `throughput_calculator`, `housing_source`, `employment_source`, `cpi_source`, `income_source`, `reserve_army_data_source`, `turnover_profile_source`, `inventory_data_source`, `depreciation_data_source`, `distribution_calculator`, `rent_calculator`, `housing_calculator`, `financial_crisis_assessor`, `fictitious_capital_calculator`, `credit_aggregate_source`, `hex_grid`, `periphery_labor_source`, `final_demand_source`, `industry_county_allocator`, `production_chain_calculator`, `bea_industries` | **DEFER-WITH-D-ROW (25)** | §0.1's residual-train table names the owner of each: **national parameters** (`melt_calculator` — the ONE hard-required field, `:198-200`; `basket_calculator`, `gamma_calculator`), **county state surface** (`housing_source`, `employment_source`, `cpi_source`, `income_source`), **Vol I** (`reserve_army_data_source`, plus `tensor_registry`'s other reads), **Vol II** (`turnover_profile_source`, `inventory_data_source`, `depreciation_data_source`, `capital_calculator`, `throughput_calculator`), **Vol III** (`distribution_calculator`, `rent_calculator`, `housing_calculator`, `financial_crisis_assessor`, `fictitious_capital_calculator`, `credit_aggregate_source`), **imperial rent / hex** (`periphery_labor_source`, `final_demand_source`, `industry_county_allocator`, `production_chain_calculator`, `bea_industries`, `hex_grid`). **Two carry explicit "defect not to transcribe" flags for whoever lands them:** `employment_source`'s `100_000.0` (`:793` — *not a fallback, the value*) and the hardcoded national dispossession/housing dicts (`dynamics/hardcoded_data.py`). |

**Two non-`services` inputs owed the same honesty:**
- **`crisis_phase`** comes from `county.crisis_state.phase` — the Step-5 five-phase state machine, **unported**
  (§0.1). It becomes a **seeded `territory/crisis-phase` (`enum CrisisPhase`)**: declared input, **produced by
  nothing in this pack**. **This is BLOCKER-4** (§9), stated in the pack header, the ADR and the follow-on issue.
- **`melt` / `tau`** is carried in `EconomicConditions` (`:2403`) and **never read by the engine** — dropped, D-row
  in the same class as D-NF+11.

**No member of the 33 lacks an honest BSL representation within this pack's boundary.** The genuine blockers are
elsewhere and are named in §9.

---

## 4. The reformulation — where the state lives, the field roster, and the coefficient estate

### 4.1 The county is a node; there is no FIPS field

The frozen per-county state is a `dict[str, CountyEconomicState]` keyed by a 5-char FIPS string. In BSL the county
**is** a `NodeType/TERRITORY` node (landed practice: every `*-county` node in every conformance scenario), and its
class distribution is a set of declared `territory/*` fields. **There is no FIPS field and none is needed** — the
landed precedent is explicit (`dispossession.bsl:99-110`: *"DEAD FIELD — fips_code/year are dropped … They are
passthrough identification fields on the Pydantic state object, not formula inputs. Dropping them changes no
observable output."*), and it is exactly ADR198 **R7**'s *"where the string was really naming a node, key by node
identity instead"* clause. **Consequence to state, not to claim coverage of: ADR198 R7's int-FIPS leading-zero trap
gains NO witness on this train** (D-NF+21).

**No carrier node is needed** — every datum is per-county, and the one cross-node fold (R6's readout) is territory-
anchored over TENANCY neighbours. This pack therefore **anchors no rule on `NodeType/INSTITUTION`**, which is also
what keeps it disjoint from Community's singleton-carrier invariant (§2.3).

### 4.2 Territory field roster — 17 declared + 7 staging = 24, every one read by a rule in this pack

**Rev 1 headed this table "16 fields", listed 17 rows, said "16" again in §2.3 and "17-field roster" in Task 2 —
three counts of one roster. The measured count is 17 in the table below, plus the 7 staging fields §4.2.1 now
declares in full (rev 1 deferred them to "§7's rule table", which gave no types either).**

| field | type / kind | frozen origin |
|---|---|---|
| `territory/share-bourgeoisie` | `probability intensive` | `bourgeoisie_share` — externally fixed, **never written** by this pack |
| `territory/share-petit-bourgeoisie` | `probability intensive` | `petit_bourgeoisie_share` — externally fixed, **never written** |
| `territory/share-labor-aristocracy` | `probability intensive` | `labor_aristocracy_share` |
| `territory/share-proletariat` | `probability intensive` | `proletariat_share` |
| `territory/share-lumpenproletariat` | `probability intensive` | `lumpenproletariat_share` |
| `territory/dist-year` | `int extensive` | `ClassDistribution.year`; incremented once per boundary (D-NF+3/D-NF+4) |
| `territory/baseline-la-share` | `probability intensive` | **NET-NEW** — R10's cumulative baseline (D-NF+15) |
| `territory/baseline-la-known` | `int extensive` (0/1) | the III.11 loud-absence companion latch — **there is no `:default` at `deffield`, so a known-flag is mandatory, not stylistic** |
| `territory/unemployment-rate` | `probability intensive` | `EconomicConditions.unemployment_rate` |
| `territory/median-wage` | `real intensive` | `median_wage` (hourly; the ×2080 happens in-rule) |
| `territory/phi-hour` | `real intensive` | `phi_hour` |
| `territory/foreclosure-rate` | `probability intensive` | frozen default `0.006` |
| `territory/bankruptcy-rate` | `probability intensive` | frozen default `0.006` |
| `territory/eviction-rate` | `probability intensive` | frozen default `0.063` |
| `territory/crisis-phase` | `enum CrisisPhase` | `county.crisis_state.phase` — **declared input, BLOCKER-4** |
| `territory/phi-savings-adjustment` | `coefficient intensive` | **NET-NEW** — R9's published coupling, written by `fundamental-theorem.bsl` (D-NF+18) |
| `territory/bifurcation-score` | `real intensive` | **NET-NEW** — R6's readout; range `[−1,+1]` **by construction**, so `real`: a unit-interval type refuses the negative half (the landed `wage-balance` precedent) |

Plus **one** net-new class-side publication and four landed reads:

| field | type / kind | role |
|---|---|---|
| `social-class/ternary-net-fascist` | `real intensive` | **NET-NEW, written by this pack** — the per-class `(fascist − revolutionary)` publication that lets the county fold use a **bare accessor** body (§4.6) |
| `social-class/revolutionary`, `/fascist` | `probability intensive` (landed) | **READ ONLY** — `consciousness.bsl`'s ternary |
| `social-class/population` | `int extensive` (landed) | **READ ONLY** — the fold's extensive `:weight` (ADR070's population-weighted read policy) |

#### 4.2.1 The seven staging fields — DECLARED HERE, with the type derivation that keeps world 4 alive

These exist so each rate constructor is **independently mutation-killable**; the split's cost is closed by §7a's
duplication ledger. **Their types are load-bearing, not bookkeeping** — a store outside a declared range is
`E-EVAL-020`, *"a loud failure, never a clamp"* (`evaluator.rs:139-142`), and
`probability`/`intensity`/`coefficient` are `[0,1]` while `real` carries no range law (`types.rs:230-244`).

| field | type / kind | widest value its own rule can produce | why that type |
|---|---|---|---|
| `territory/rate-accumulation` | `probability intensive` | `min(annual-acc ÷ 142000, 0.08)` ∈ [0, 0.08] | clamped by construction; the declared range makes the clamp's absence a LOUD load-time failure, so a mutation that deletes the clamp cannot pass silently |
| `territory/rate-dispossession` | `probability intensive` | `0.6f + 0.3b + 0.1e` with each rate ∈ [0,1] ⇒ ∈ [0,1] | as above |
| `territory/rate-precaritization` | `probability intensive` | `clamp(·, 0, 1)` | as above |
| `territory/rate-stabilization` | `probability intensive` | `clamp(0.15·(1−u), 0, 1)` | as above |
| `territory/raw-share-labor-aristocracy` | **`real intensive`** | **CAN BE NEGATIVE** — `LA − disp·LA + acc·Prol` has no floor; `_normalize`'s `max(·,0)` at `transition_engine.py:313-315` exists precisely because of it | **`probability` would abort `a06` at E-EVAL-020 and world 4 — the ONLY fixture that reaches the degenerate branch — would never reach it.** World 4 is deliberately seeded so all three go negative (§Worlds), so this is not a hypothetical |
| `territory/raw-share-proletariat` | **`real intensive`** | same | same |
| `territory/raw-share-lumpenproletariat` | **`real intensive`** | same | same |

**The amplified rates overwrite the same four `rate-*` fields** (`a05` reads and re-writes them, each product
`min(·,1)`) rather than minting four more — one datum, one home, and the `[0,1]` declaration is what proves the
per-product clamp landed. **Rev 1 reasoned about exactly this range hazard for `territory/bifurcation-score` and
did not carry it to the staging fields; that is I6, and it was a world-4-shaped hole.**

### 4.3 Seeding — fractional per-node seeds ARE legal (do not copy `dispossession.bsl`'s stale reasoning)

`dispossession.bsl:29-40` argues its five per-territory rates must be `:const` because *"there is no legal way to
seed a genuinely fractional per-node value in slice 1 at all, on any field, of any declared type."* **That claim is
STALE.** Train B item 6 landed `real` as a declarable type and rebuilt the seeding arms (`scenario.rs:1093-1330`):

| declared type | accepted seed literals |
|---|---|
| `int` | `Atom::Int` only (exact to 2⁵³) |
| `real` | `Int`, `p`/`i`/`c` scaled, `r` scaled |
| `probability` / `intensity` / `coefficient` | `Int` or `p`/`i`/`c` scaled; `[0,1]` enforced **at load, loudly, never clamped**; `r` refused |
| `currency` | refused (deferred to its first consumer) |
| `enum` | `<EnumType>/<MEMBER>` only (`E-LOAD-056`); the ordinal is never a surface value |

So `(node wayne NodeType/TERRITORY (share-labor-aristocracy 0.40c) …)` seeds directly. **Every share, rate,
baseline and wage in §4.2 is seeded per-node, not flattened to a `:const`.** This is the plan's largest ergonomic
gain over the dispossession-era pattern and the implementer must not regress it. `bool` remains absent — latches
are `int` 0/1. Task 0 Step 5 re-verifies this table at the byte and marks the stale header row with its citation.

### 4.4 The annual boundary — `:tick-in-cycle`, not modulo

`ARITH` is the closed set `+ - * /` (`grammar.rs:724`) — **there is no `%`, no integer floor-division, and no
`min`/`max` scalar intrinsic** (`solidarity.bsl:63` states the last one in the landed estate's own words). The
frozen gate `tick % WEEKS_PER_YEAR != 0` is therefore not expressible arithmetically, and the **`floor` escape is
forbidden to this pack** (§Global: `territory.bsl:78` owns the one declaration; a duplicate is `E-LOAD-001`).

**The served construct is `:tick-in-cycle <int-lit>`** (`bindings.rs:55-59`, `:410-416`; evaluated at
`tick.rs:269`; `ScoreClass::Scalar` at `score_class.rs:156`; `E-PARSE-014` on a non-positive length). Every rule in
the pack carries `(binding phase-of-year :tick-in-cycle 52)` and `(when (= phase-of-year 0) …)`. `:year` and
`:tick-of-year` are **refused** with texts this plan quotes verbatim (`tick.rs:456`, `:462`):
*":year — slice 1 pins no epoch; §2.5 puts the epoch and the …"* / *":tick-of-year — slice 1 pins no ticks-per-year
figure (§2.5, as for :year)"*. So the frozen `base_year + tick // WEEKS_PER_YEAR` derivation is **not reproduced**;
the year is a per-territory `int` field seeded per scenario and incremented once per boundary — **which is exactly
what `with_updated_dynamics` does** (`types.py:132-133`). Nothing is lost. **D-NF+3.**

**`52` is an integer LITERAL on the binding, not a `defconst`** (`:tick-in-cycle` takes an integer literal).

**THE TICK-0 QUESTION IS ANSWERED FROM SOURCE, AND THE ANSWER RESHAPES EVERY PIN IN THIS TRAIN.** Rev 1 deferred it
to a Task-1 spike and framed it as *"the frozen qa harness's boundary IS tick 0"*, which invites the wrong answer.
At the byte: `TickSession::new` sets `tick: 0` and `advance` computes `next_tick = self.tick + 1`, so **the first
executed tick is 1** (`session.rs:60-66,120-124`, and its own doc: *"The first call runs tick 1 … the second tick
2"*); `run_once` hard-codes tick 1 (`lib.rs:517-531`). **Tick 0 is never executed by any driver.** With
`tick.rem_euclid(52)` (`tick.rs:269`), `1 % 52 = 1 ≠ 0`, so:

- **the pack's first boundary is tick 52**, then 104, then 156;
- **a `run_once` pin over any world of this pack pins an INERT pack** — `fired = 0`, `before == after`. Rev 1's
  seven `run_once` pins would have recorded a table of zeros while claiming to pin the arithmetic (**C2**);
- `a09`'s *"≥105-tick session"* gives **two** boundaries (52, 104), not three; world 5's *"≥3-boundary arc"* is a
  **≥156-tick session** (M12 — rev 1 never sized it in ticks, and it is the longest run in the train);
- **the pin design splits in two** (§Global, §Worlds): a **load pin** at tick 1 via `run_once` — which is now a
  *deliberate* artifact, pinning the seeded world AND the off-boundary inertness — and a **boundary pin** at tick
  52 via `TickSession::advance` ×52, which pins the engine output. Every world takes both.

Task 1 Step 2's spike survives only as a **cheap confirmation** of this reading against the real driver, not as
the source of it. D-NF+30 records the convention.

### 4.5 The `defconst` estate — 46 named coefficients (R8), and the 70-row disposition arithmetic

`defconst`s are **scenario-side**: zero `(defconst …)` forms exist in any landed `.bsl`; all live in `.bscn` files
and are referenced from rules by `:const`. **Consequence: all 46 are re-declared in each of the eight content worlds (and the relevant subset in each co-load world and in `two-classes.bscn`).**
That is a fidelity risk (**46 × 8 = 368 literals** across the eight content worlds, plus each co-load world's
subset and `two-classes.bscn`'s four — rev 2 left the seven-world figure, N9), so **Task 2 lands a canonical block
copied byte-identically** and
a **cross-world constant-parity test** asserting identity for every constant a world does not *deliberately* vary —
with each deliberate variation named in that world's header (the landed seven-environment `dispossession.bsl`
practice, and `carceral-arc-conformance.bscn`'s "companion-varied to 0" convention).

**Live defconsts — declared and read by a rule in this pack (46):**

| group | count | values | notes |
|---|---|---|---|
| engine parameters | 4 | `142000` (Int), `0.5c` precaritization-**unemployment** weight (D-NF+14), `0.15c` base stabilization (D-NF+13), `0.08c` max accumulation rate | `transition_engine.py:51-54` |
| phased amplifier table | 20 | §1.2's 5×4 grid | **7 exceed 1.0 and are fractional** (1.2, 1.5, 1.8, 2.5, 3.5, 1.3, 1.2) → the scaled-int `x1e6` lane, D-NF+5 |
| dispossession composite (LA→P only) | 3 | `0.6c`, `0.3c`, `0.1c` | the P→L trio is **not declared** — D-NF+11 |
| savings + cap | 2 | proletariat rate `0.03c`, **`phi-cap 0.05c`** | four unread rates **not declared** — D-NF+12. **`phi-cap` is a DEFCONST (D-NF+19), not a define.** |
| dispossession-rate defaults | 3 | `0.006p`, `0.006p`, `0.063p` | defaults are behavior (§1.3) |
| wage / subsistence | 3 | `hours-per-year 2080`, `v-reproduction 12`, `accumulation-halt-floor-ratio 0.8c` | FR-017; the halt floor is exactly `12 × 0.8 = 9.60` |
| bootstrap shares | 5 | `0.01c / 0.09c / 0.40c / 0.35c / 0.15c` | **R5-confirmed SEEDS** under R7's membership reading — also `defconst`s so the seeding-parity test can prove the world matches the ruling |
| cascade + threshold | 4 | `0.05c`, `0.10c`, `0.15c`, `bifurcation-event-threshold 0.5c` | R5; the threshold is **the one surviving define of R6's four** |
| year window | 2 | `2007`, `2030` | D-NF+4 |
| **total** | **46** | | |

**The 70-row disposition (§1.6's corrected inventory), stated as arithmetic so nothing is silently dropped:**

| bucket | count | rows |
|---|---|---|
| **live `defconst`s** | **29** | transition_engine 4 + amplifier 20 + LA→P weights 3 + (proletariat rate, `phi_cap`) 2 |
| **recorded-not-declared** | **9** | legacy `DefaultCrisisAmplifier` 2.5/0.3 (**2**, D-NF+6) + P→L weights (**3**, D-NF+11) + unread savings rates (**4**, D-NF+12) |
| **conformance BOUNDS, not defconsts** | **32** | `validation.py`'s 20 rate + 12 share thresholds (**D-NF+9**, and **§10 DG-8** puts the interpretation to the Director) |
| | **70** | |

The remaining **17** live defconsts (bootstrap shares, cascade milestones, threshold, rate defaults, wage/
subsistence, year window) come from `system/__init__.py` and `defines.yaml`, **not** from the 70-row estate:
`29 + 17 = 46`. Every one of the 46 owes a mutation vector (§Global).

**R8's binding list is 41 rows wider than this plan declares, and ALL 41 go to ONE Director gate — not 32.**
`dossier-rulings.md:136-138` records R8 as binding on *"`transition_engine.py`'s 4 constants, `crisis.py`'s **2
legacy** + 20 phase-table multipliers, `dispossession.py`'s **6** composite weights, `savings_schedule.py`'s **5
class rates** + phi cap, `validation.py`'s thresholds."* This plan declares 4 of 4, 20 of 22, 3 of 6, 2 of 6, and
0 of 32 — **41 rows declined in total**: the 32 thresholds *plus* the 2 legacy `DefaultCrisisAmplifier`
multipliers (D-NF+6), the 3 P→L weights (D-NF+11) and the 4 unread savings rates (D-NF+12). **Rev 1 routed only
the 32 to a Director gate and disposed the other 9 on its own "declare only what you read" authority — taking
both positions in one document (I1).** Rev 2 takes ONE: **every declined row is the same question**, and
**§10 DG-8 now asks it about all 41**, with the 9 broken out by name so the Director sees the whole surface. The
plan's default answer is unchanged (declare only what a rule reads); what changes is that the workforce no longer
narrows a Director ruling's binding list on its own authority for some rows and escalates for others.

**Why the 32 validator thresholds are bounds, not content.** They drive **only** `logger.warning` /
`logger.error` (`transition_engine.py:188-192`, `:333-343`). Constitution III.11 / invariant S-11 —
*"No warning level, no degraded mode … An error is never converted to a default, a skipped effect, or a log line"*
— means BSL has **no warning level to port them into**. Declaring 32 constants no rule reads would violate this
plan's own declare-only-what-you-read law and R8's own "declared, moddable, **hash-covered**" framing (a constant
no rule reads is not hash-covered in any meaningful sense). They land as **behavioral bounds in the conformance
suite**, which is where the rewrite test says durable knowledge belongs. **This is an interpretation of a ruling
and is flagged as such in the PR body, the D-row and §10 DG-8 — never absorbed silently.** Note `_MAX_ACCUMULATION_
RATE = 0.08` **equals** `ACCUMULATION_WARNING_MAX` exactly (`validation.py:39`) — **F14**, preserved deliberately,
and **under the F11 repair the clamp begins to bind for the first time**, which makes it mutation-provable at last.

### 4.6 R6's readout — the two fold shapes, and the one that must be spiked

R6 ratifies:

> **`score_county = ( Σ_c population_c · (fascist_c − revolutionary_c) ) / ( Σ_c population_c )`**
> — the population-weighted net fascist-minus-revolutionary mass of the county's classes.

**`field_ref_for` reduces a fold body — and a fold's `:weight` — to exactly FOUR shapes** (a bare `<qname>`; a
binding symbol, including through an `:expr` chain up to `MAX_BINDING_CHAIN = 8`; a `field-of` accessor; a nested
`fold`) and refuses everything else, **including arithmetic**, as `compound_fold_error`
(`rule_pipeline.rs:640-708`; rev 1 said three — M9). **So `(f − r)` cannot be a fold body.** The reformulation is
the publish-then-fold pattern applied to a readout: a **class-anchored rule publishes
`social-class/ternary-net-fascist`**, and the territory folds **that** field with a bare accessor.

Four facts, all four now verified — **rev 1 listed the third as UNVERIFIED and ranked it the train's #1 variance
item; it is answered by the exact lines rev 1 listed as required reading (I5)**:

- **VERIFIED — and it closes the T4 dossier's blank result-kind cell.** `FoldOp` is the closed set
  `{sum, mean, min, max, count}` (`grammar.rs:672-683`); `score_class.rs:210-223` gives `mean`/`sum`/`min`/`max`
  the **body's own** score class and `count` `Scalar`. `fold mean` over an **intensive** field is legal **iff** an
  **extensive** `:weight` is supplied — unweighted is `UnweightedMeanOfIntensive`, a non-extensive weight is
  `NonExtensiveWeight` (`typecheck.rs:178-202`). `social-class/population` is `int extensive`. **`(fold mean …
  :weight …)` is therefore the SANCTIONED shape here, not a workaround** — record it as the answer to the dossier's
  standing "blank result-kind cell" obligation.
- **VERIFIED.** `fold sum` refuses intensive outright, *"no weight rescues it"* (`typecheck.rs:166-176`) — so the
  two-published-extensive-sums alternative would need two extra contribution fields. Recorded as the **rejected**
  alternative, with its cost.
- **VERIFIED — there is ONE checker with an adapter, not "two checkers, two shapes".**
  `rule_pipeline.rs:744-760` runs `field_ref_for` over the `:weight` **exactly as over the body**, and only then
  hands the *adapted* form (bare `QName`s in both slots) to `typecheck.rs::destructure_aggregation`, whose
  Symbol/QName-only acceptance is therefore about the ADAPTED form, never the authored one. **`(field-of it
  social-class/population)` is a legal `:weight` spelling**, and a `weight` that `field_ref_for` cannot reduce is
  `compound_fold_error`, not a silent pass. Landed content proves the body half already
  (`decomposition.bsl:284-291`, `control-ratio.bsl:281-287`). Task 1 Step 1 keeps a **one-line confirmation**
  against the real driver — cheap, and it costs nothing to be sure — but **BLOCKER-6's "re-plan of Task 8"
  framing, its fallback branch and its #1 variance ranking are all retired** (§9, §Estimate).
- **VERIFIED — and it is a HAZARD, not a footnote: the fold aborts on an empty query, unconditionally.**
  `mean` over an empty set is `E-EVAL-021` (`evaluator.rs:143-147`: *"there is no element to return and there is
  no null"*), and **bindings evaluate before the guard** (`tick.rs:583-609`; `control-ratio.bsl` `c03`'s
  `:material-basis` states the law in the estate's own words). So `(when (= phase-of-year 0))` does **not** protect
  `a13`'s fold: **any TERRITORY, in any world loading this pack, with no incoming-TENANCY SOCIAL_CLASS neighbour
  kills EVERY tick, tick 1 included** — and rules are not selectable per world, so worlds 2–5's "exercises
  `a01`–`a11` only" gives no relief. **`a13`'s fold therefore carries the landed protector verbatim**
  (`territory.bsl:168-172`): `(binding score :expr (if (exists (neighbors self EdgeType/TENANCY :in
  NodeType/SOCIAL_CLASS) #t) (fold mean … :weight …) (- 0 0c)))`. Rev 1 cited `territory.bsl`'s fold line and did
  not carry its protector across — **that is C5**, and it is why every world's header must state the TENANCY
  invariant (§Worlds) and why world A's foreign-shaped territories are the fixture that proves it.

**Fold RANGE.** Both landed idioms exist and the plan uses the second: `(neighbors self EdgeType/… …)`
(`territory.bsl:169`, `production.bsl` ×11) **and** `(nodes NodeType/…)` (`decomposition.bsl:284-291`,
`control-ratio.bsl:281-287`). **The 2026-08-17 draft's claim that "no landed content folds over `(nodes …)`" is
STALE — correct it in Task 0's dossier.** This pack still uses `neighbors`/TENANCY, because the fold must be
**county-scoped**, and `(nodes …)` is world-scoped. Inherit the **D136** territory-side-fold question — and
answer it for a mean rather than a sum (§2.4).

### 4.7 R4 — the asymmetry is the theory, and this pack must PROVE it, not merely avoid breaking it

ADR210 **R4** is a STANDING THEORY RULING, quoted in full in the pack header:

> *"the bifurcation score's revolutionary-term zeroing under the no-SOLIDARITY-seeding policy is a **FEATURE** —
> revolutionary crisis direction must be EARNED BY ORGANIZING; SOLIDARITY edges exist only when organizing creates
> them in play; fascism is the default drift of unorganized crisis. The asymmetry between the (−) solidarity term
> and the (+) burden term is the theory, not a defect."*

Its prior art (ADR016, quoted at `reports/tickdynamics-trio-dossier-2026-08-17.md:39-46`): *"Material disruption (wage decline)
creates 'agitation energy' that has NO INHERENT DIRECTION… If solidarity infrastructure exists: agitation → class
awakening → revolution. If solidarity infrastructure absent: agitation → fascist turn → reaction"*, and
*"`solidarity_strength = 0.0` means NO solidarity infrastructure. Must be BUILT through player/system actions."*

**Mechanically, R6 moves the score onto `consciousness/p6-route`'s ternary — which is where the asymmetry already
lives** (`p6-route`: `sol-factor` from `solidarity-inbox`, `delta-r = consumed · eff-sol · scale`,
`delta-f = consumed · (1 − eff-sol) · scale · (1 − suppression)`; with no SOLIDARITY edge the inbox is 0, `eff-sol`
is 0, `delta-r` is exactly 0 and all routed agitation goes fascist-ward). **This pack does not re-implement the
asymmetry and must not.** What it owes is a **guard that the readout carries it**: world 6 (§Worlds) seeds a
crisis county with **no SOLIDARITY edge at all** and pins `bifurcation-score > 0` strictly, with a companion county
that has one and pins `< 0` — the executable form of "earned by organizing", named
`the_unorganized_county_drifts_fascist_and_the_organized_one_does_not`. **D-NF+20**, and the test's name appears in
the PR body verbatim because it is the train's clearest Director-facing artifact after R10's arc.

---

## 5. The 14-row defect ledger — disposed AS A CLASS (R11), one D-row each

R11: *"the dossier's 14-row defect ledger (incl. the wage·s² double savings-rate application, the `_normalize`
degenerate branch's bare `target/3` constant) disposes **AS A CLASS** per ADR183's repaired-at-the-port doctrine —
each repair carries its D-row at the TickDynamics landing, no per-defect sitting."*

**The architect does not re-litigate whether to fix these, only how — and must not touch the frozen Python to do
it (ADR183 R2).** Two rows carry consequences that must not be silently absorbed.

| dossier row | disposition on this train | D-row |
|---|---|---|
| **F3, F4, F5, F6** (the `BifurcationRiskCalculator` surface: `w_s`, `w_b`, `class_burden_epsilon`, the burden ratio, the legitimation dampener, the `node.id == fips` lookup, the unweighted `mean(agitation)`) | **MOOT under R6, not separately disposed.** The whole formula retires; **F5 is explicitly NOT ported as a live repair** — *"MOOT under R6 (the scalar and its blend both retire), so this repair is now moot, not owed."* | D-NF+17 |
| **F11** — savings rate applied twice (`wage·s²`), **33× understatement** at the proletariat's 0.03 | **REPAIRED** (`accumulation = wage · s`), at **`accumulation.py:90`** (`consumption` is `:89`; the docstring admission is `:39-41`). **Rev 2 "corrected" this to `:89` by accepting critique row M8 without re-measuring — a regression of correct text, caught as N3 and reverted here; D-NF+7's `:90` was right all along.** **Consequence not to absorb:** repairing it *"moves `michigan_canada_e2e`'s class shares materially… owes a §6.5 ceremony in the Rust lane's own vectors."* This train's Rust vectors are **new**, so the ceremony is discharged by measuring them fresh; **the Python `tests/baselines/**` estate is untouched and MUST stay byte-identical** (§Global). If it moves: STOP **to diagnose**, then B10's own fork — quoted verbatim in §Global — decides whether it is another train's bug or this train's ceremony. | D-NF+7 |
| **F10 / `_normalize` degenerate branch** — the bare `target / 3.0` equal-thirds reset | **REPAIRED**: the degenerate arm **writes nothing** — preserve the previous distribution, never fabricate a number (III.11). A dedicated world reaches the branch and makes the repair mutation-provable, plus an explicit **anti-assertion** (`a07_does_not_write_equal_thirds`) so a future reader cannot restore the constant silently. | D-NF+8 |
| **F13** — `base_stabilization` docstring says `0.10` twice, the constant is `0.15` | **DISAMBIGUATED**: `0.15` transcribes, the docstring pair does not. | D-NF+13 |
| **F14** — `_MAX_ACCUMULATION_RATE` equals `ACCUMULATION_WARNING_MAX` exactly | **PRESERVED DELIBERATELY**; under F11's repair it becomes **REACHABLE** — but only by a declared synthetic probe, and rev 1's "binds for the first time" overstated it. **The arithmetic, derived rather than asserted (I8):** post-repair the clamp binds when `median_wage_hourly · 2080 · s ≥ 0.08 · 142000 = 11360`, i.e. **`$182.05/hr` at the proletariat rate `s = 0.03`**, or **`$68.27/hr`** at the maximum Φ-boosted `s = 0.08`. World 1 seeds `21.0`, and no real US county median comes near either figure. So the clamp's vector is an explicitly-labelled **bound probe** — one county, absurd-by-construction, its header carrying this derivation and stating that under R7's measured-membership reading the wage is a *probe*, not a claim about any county — on the landed `control-ratio-conformance.bscn` "companion-varied, licensed, never silent drift" precedent. Without that label the fixture is a lie about the world; without the fixture the clamp is unkillable content. | D-NF+13 |
| **F15** — "no data" and "no net change" are byte-identical outputs | **NOT TRANSLATED.** Declared content cannot be "unwired" and a missing field is a load error (III.11), so the `NoDataSentinel` abort path has **no ported trigger**; the two encodings can no longer collapse because the former is unrepresentable. | D-NF+10 |
| **F16** — four of five savings rates are dead (the call site hardcodes `PROLETARIAT`) | **DECLARED NOWHERE**; recorded as the WS4 ledger question the dossier names: *"are they reserved for a consumer, or is the schedule four-fifths dead?"* | D-NF+12 |
| **the `_DEFAULT_EVICTION_WEIGHT` misnomer** — it multiplies unemployment | **arithmetic exact, name not transcribed**; a fixture where swapping the two weights changes the answer proves it. | D-NF+14 |
| **`p_to_l_component`** — computed, returned, never read | **RETIRES** (register row 24 / WS4); its three weights are not declared. **Note: the general "may the workforce retire a never-read output on its own authority?" question is STILL OPEN on the record** — §10 DG-7. | D-NF+11 |
| **the "Step 5b executes after Step 6" doc row** | **transcription note only** — the frozen comment numbering is stale relative to call order (`:270` then `:279`); this pack's rule-id byte order is the contract and is mapped explicitly in §7. | D-NF+1 |
| **the `round()` half-even × 7 payload sites** | **DROPPED.** BSL declares `{exp, log, floor, rng-draw}` and has **no `round`**; `floor(x+0.5)` is half-**up** and diverges at exact ties. **All in-scope sites are payload PRESENTATION**; the 2 state-affecting demotions live in `reserve_army/accumulation.py:115-123`, **outside this pack**. Payloads emit full precision. **Task 0 Step 2 re-verifies the census independently; if a state-affecting site turns up inside the boundary, STOP** — the fix is an intrinsic rider, not a workaround. | D-NF+16 |
| **the `hasattr` amplifier selection** | **not expressible, not ported**; `PhasedCrisisAmplifier` is the ported amplifier and the legacy path is a WS4 row. | D-NF+6 |
| **F18** — the percentile-band descriptions contradict the membership reading | **R7-ruled**: the descriptions die and are transcribed nowhere; the values survive as seeds. | D-NF+26 |
| **F19/F20/F22** — the cascade never fires under the per-boundary read | **R10-ruled**: the baseline becomes cumulative; the divergence is the D-row and the arc world is the proof. | D-NF+15 |

**Standing engineering obligations the dossier names as "surviving every option"** — owed to whoever lands the
port regardless of how §10's open questions resolve: the `round()` half-even row (**D-NF+16**), the blank
result-kind cell for a weighted `fold mean` (**closed by §4.6, recorded in D-NF+17**), ADR198 R7's unexercised
int-FIPS leading-zero trap (**D-NF+21**), the **D136** territory-side-fold double-count record inherited by any
territory-side TENANCY aggregation (**§2.4 — rev 1 called this "D45" four times; D45 is the `select-max` tiebreak,
and the distinction changes the argument, not just the citation**; pinned by world 1's `shared-class`), and F16's
WS4 ledger question (**D-NF+12**).

---

## 6. Transcendentals and functional forms — the verdict

**This pack declares NO intrinsic and contains NO transcendental.** Finding F9 is the gate-pass record, quoted in
the pack header verbatim:

> *"F9 — the headline, and it is a negative: there is no imposed functional form anywhere in the transition engine.
> No exp, no log, no tanh, no sigmoid, no Gaussian, no power law — no transcendental of any kind across all 1,476
> lines of the seven modules… ADR172 ruling 5 is satisfied by this surface as it stands."*

Every shape that could be mistaken for an imposed form is named here as what it actually is:

| shape | what it is | why it is not an imposed form |
|---|---|---|
| the 5×4 amplifier grid | a **lookup table on a measured phase**, transcribed | no curve is fitted or assumed; each row is a discrete ruled multiplier, and the DEEP row's magnitudes are routed to §10 DG-4 as pedagogy |
| `min(annual_acc / 142000, 0.08)` | a **clamped ratio** | a bound, not a shape; the clamp is a declared ceiling with a mutation vector |
| the normalize rescale `target / total` | a **re-anchor** | ≡ 1.0 by F10's exact conservation; it corrects float drift, it does not shape anything |
| R6's population-weighted mean | a **MEASURE** over the landed ternary (ADR070's read policy) | it aggregates existing per-class state; it stipulates nothing |
| the cascade's three milestones | **discrete thresholds on a measured decline**, R5-confirmed | no continuous response function |
| Φ → savings (`min(φ·2080/wage_annual, cap)` ≡ `min(φ/wage_hourly, cap)` — §1.2) | a **clamped linear ratio** | R9-ratified as explicit law; no curve. The unit matters: divide by the HOURLY wage and the cap saturates for every county, which is a switch, not a ratio (C3) |

**The one sigmoid inside @4.0 is out of this pack.** `reserve_army/calculator.py` (the three defines `sigmoid_k`,
`sigmoid_r0`, `wage_pressure_ceiling` at `:44-46`, exact; the `math.exp` calls at **`:52` and `:57`** — `:51` is
the overflow clamp on the exponent, not the call, N5 — reached via frozen Step 3.5) is an **imposed logistic** that ADR173 ruling 1's precedent
requires to be **re-derived as an emergent measure, never transcribed**. It belongs to the Vol I residual train
(§0.1) and this plan names it there so the next architect inherits the obligation rather than the code.
`sigmoid` is additionally a prohibited intrinsic name (`E-LOAD-024`), and spelling a logistic out of `exp`/`log`
is the same prohibited motion.

---

## 7. Rule layout — 13 rules in one new pack, +1 in an existing pack

Execution order is **ascending rule-id byte order** across all loaded packs, subjects in ascending node id
(`tick.rs:38`). Ids are chosen so byte order equals intended order; **every same-tick dependency is a deliberate
D116 reliance**, ledgered in §7b. **Every rule IN THIS PACK carries the boundary gate** `(binding phase-of-year
:tick-in-cycle 52)` + `(when (= phase-of-year 0) …)` (§4.4) — **all thirteen, `a12` included (rev 2.1's N1
decision, recorded in its row below)**; the added `economics/` rule in the OTHER pack deliberately does not, and
§7's pack-edit block says why. **This law is absolute inside this pack, and it is load-bearing three times over**:
§7a's `the_pack_is_inert_off_the_boundary` covers 13 copies, every tick-1 load pin asserts this pack's
`fired == 0`, and the co-load worlds assert the pack changes nothing on a non-boundary tick. A rule that needs to
fire off-boundary is a plan amendment with those three consequences re-derived, never a local choice.
**The gate does not protect bindings**, which evaluate first (§4.6, C5). **Fuel figures are MEASURED at Task 10, never guessed** — no rule
ships a number before that sweep.

### Pack — `content/rules/class-dynamics.bsl`, namespace `class-dynamics/`

| id | subject | boundary row (§2) | does |
|---|---|---|---|
| `a01-rate-accumulation` | TERRITORY | unclaimed | `effective-wage = median-wage · 2080`, **zeroed** when `median-wage < v-reproduction · halt-floor-ratio` (strict `<`, FR-017); `s = min(savings-proletariat + phi-savings-adjustment, 1)` as a nested `if`; **`annual-acc = effective-wage · s`** (the F11 repair, D-NF+7); publishes `rate-accumulation = if (<= annual-acc 0) 0 else min(annual-acc ÷ 142000, 0.08)` |
| `a02-rate-dispossession` | TERRITORY | unclaimed | `0.6·foreclosure + 0.3·bankruptcy + 0.1·eviction`, each rate a per-node field seeded from its `defconst` default |
| `a03-rate-precaritization` | TERRITORY | unclaimed | `clamp(u · 0.5 + eviction · (1 − 0.5), 0, 1)` — D-NF+14's misnomer recorded, arithmetic exact |
| `a04-rate-stabilization` | TERRITORY | unclaimed | `clamp(0.15 · (1 − u), 0, 1)` — D-NF+13 |
| `a05-amplify` | TERRITORY | unclaimed | the 5×4 grid as a nested `if` on `crisis-phase`, each product `min(·, 1)`. `if` takes **exactly three operands — the else branch is mandatory** (`grammar.rs:649`) and both branches share one static type; use the landed `(- 0 0c)` / `(- 1 0c)` promotion idiom. Publishes the four amplified rates |
| `a06-flows` | TERRITORY | unclaimed | the three flow equations verbatim (§1.2 step 5) into three published `raw-share-*` fields |
| `a07-normalize` | TERRITORY | unclaimed | `max(·,0)` each; `total = Σ`; `target = 1 − (bourgeoisie + petit-bourgeoisie)`; `(guard (> total 0) …)` rescales by `target ÷ total`; **`(guard (= total 0) …)` writes NOTHING** — D-NF+8's repair |
| `a08-commit-shares` | TERRITORY | unclaimed | writes the three dynamic shares; **the two fixed shares are never written** |
| `a09-year` | TERRITORY | unclaimed | `dist-year := min(max(dist-year + 1, 2007), 2030)` — `with_updated_dynamics`' increment plus the collapsed clamp (six two-sided sites tree-wide, **two** of them in this pack's boundary — §1.5, D-NF+4) |
| `a10-baseline` | TERRITORY | **R10's ruled SEMANTICS; the landed LATCH idiom is `decomposition/p02-superwage-warning`, not `p7`** | `(guard (= baseline-la-known 0) …)` seeds `baseline-la-share` from the **PRE-transition** LA share and sets the latch; on every later boundary it is untouched. **Citation corrected (I3):** `consciousness/p7-persist-baselines` (`consciousness.bsl:340-353`, not `:340-351`) writes `previous-wages`/`previous-wealth` **every tick under its guard** — it is a ROLLING previous-value persister, the *opposite* of a run-start latch, so rev 1's "R10's cited carrier pattern, verbatim" mis-described it. The landed write-once latch is `decomposition.bsl:248-260`: bind the flag, `(when (… (= crisis-known 0)))`, act, then set it. `a10` copies THAT, and R10's "on the landed `p7-persist-baselines` pattern" is honoured as what it is — the precedent for **persisting a baseline in a node field at all** (the frozen `context.persistent_data` having no BSL analogue), not for latching. **Ordering is decided BY TEST, not by argument** (Task 7 Step 1): either `a10` sorts before `a08`'s commit, or `a06` publishes a `la-share-prior` field |
| `a11-cascade` | TERRITORY | unclaimed | `decline = baseline-la-share − share-labor-aristocracy`; gates `crisis-phase ≠ NORMAL`, `baseline-la-known = 1`, `decline > 0`; **highest-milestone-only as three ASCENDING guards, last wins** — the frozen `for milestone in sorted(...)` loop's exact semantics, never a `max`; `(emit EventType/DISPOSSESSION_CASCADE …)` with the four payload keys at **full precision** (D-NF+16) |
| `a12-publish-net-fascist` | SOCIAL_CLASS | reads `consciousness.bsl`'s ternary | publishes `social-class/ternary-net-fascist = fascist − revolutionary`; **`(when (= phase-of-year 0))` — the pack's boundary gate, like every other rule here. DECIDED IN REV 2.1 (finding N1), and the reasoning belongs in the plan, not in a diff.** Rev 1 wrote "no `when` — every class writes, so nothing goes stale (the D127 hash-neutral idiom)"; rev 2's M7 fix corrected the SPELLING to `(when #t)` (the landed unconditional idiom, 8 sites: `territory.bsl:97,178`, `production.bsl:159,262`, `decomposition.bsl:240,302`, `control-ratio.bsl:270,287`) and thereby made a12 fire every tick — which **contradicted §7's own gate law, falsified §7a's 13-copy inertness row, and falsified the tick-1 load pin's `fired == 0`** in every world holding classes. **Re-derived: what M7 fixed was the spelling of a guard, never the claim that a12 must fire unconditionally — and under the gate that point is honoured a fortiori, since a12 now carries a `when` form exactly like every landed rule.** What the D127 freshness idiom was protecting is preserved where it is observable: **`a13` is the field's ONLY reader, it reads at the boundary, and `a12` sorts first, so the fold always consumes a publication written in the SAME tick** (§7b). Between boundaries the field is stale — **which is this pack's uniform annual semantics, not an anomaly**: every `rate-*`, `raw-share-*` and `share-*` field it publishes is equally an annual value. **The ONE class-side write in the whole pack** (§2.3), and now the pack touches `social-class/*` only on boundary ticks, which is what makes the co-load worlds' same-values assertion hold tick by tick |
| `a13-bifurcation-readout` | TERRITORY | **R6's ruled replacement** | **Two bindings, and the first one is the C5 fix.** `(binding has-classes :expr (exists (neighbors self EdgeType/TENANCY :in NodeType/SOCIAL_CLASS) #t))`, then `(binding score :expr (if has-classes (fold mean (neighbors self EdgeType/TENANCY :in NodeType/SOCIAL_CLASS) (field-of it social-class/ternary-net-fascist) :weight (field-of it social-class/population)) (- 0 0c)))` — the `territory.bsl:168-172` protector verbatim, because **bindings evaluate before the guard and `mean` over an empty set is `E-EVAL-021`, a tick-killing failure** (§4.6). The write is then `(guard has-classes (update-node self territory/bifurcation-score (set score)))`, so a classless county gets **no score at all** rather than a fabricated `0` — the same III.11 refusal-to-invent `a07`'s degenerate arm makes. Then `(guard (>= score threshold) (emit …))` and `(guard (<= score (- 0 threshold)) (emit …))` — **`abs` is not an intrinsic**, so the two-guard split also carries the frozen `direction` string as a numeric key (`1` = fascist, `0` = revolutionary), strings being unrepresentable in `Value` |

**Byte order:** `a01 … a13` ascending, and `class-dynamics/` sorts before `consciousness/`, `control-ratio/`,
`decomposition/`, `dispossession/`, `economics/`, `lifecycle/`, `metabolism/`, `production/`, `solidarity/`,
`territory/`, `vitality/`, `worldview/`. **Two cross-pack consequences, both deliberate:** (i)
`territory/phi-savings-adjustment` is written by `economics/`, which sorts **after** — that is D-NF+18's one-tick
lag; (ii) `a12` reads `consciousness/`'s ternary from the **previous** tick for the same reason, which is correct
— the readout is of the state the ternary reached, not of a mid-tick intermediate.

### Pack edit — `content/rules/fundamental-theorem.bsl` (+1 rule, R9's ruled home)

```
economics/phi-savings-coupling            subject TERRITORY
  ; the SAME wage base a01 computes — hourly x 2080, FR-017-halted — because
  ; the frozen divisor IS the annual, halt-zeroed wage (system/__init__.py:2378-2380,
  ; :2402 -> transition_engine.py:133-137 -> accumulation.py:85 -> savings_schedule.py:92)
  raw-annual-wage  = median-wage * 2080
  effective-wage   = if (< median-wage (* v-reproduction halt-floor-ratio)) 0 else raw-annual-wage
  phi-adjustment   = if (or (= effective-wage 0) (= phi-hour 0))
                       0
                       else min(phi-hour * 2080 / effective-wage, phi-cap)
  -> (update-node self territory/phi-savings-adjustment (set phi-adjustment))
```

**THE DIVISOR IS THE ANNUAL, HALT-ZEROED WAGE. Rev 1 divided by the raw hourly field — a 2080x error (C3).**
Two consequences rev 1 asserted and could not have delivered, both repaired by the form above: (i) at
`median-wage 21.0`, `phi-hour 1.0` the frozen value is `min(0.047619…, 0.05) = 0.047619…` while rev 1's form gave
`min(99.0476…, 0.05) = 0.05` — **saturating the cap for any `phi_hour` above ~$0.0005/hr and collapsing R9's
ratified gradient into a binary switch**, which is the opposite of "imperial rent PURCHASES entry"; (ii) the frozen
zero guard tests the **halted** wage, so a county at `$9.00/hr` yields `phi_adj = 0` — rev 1's guard on the raw
field never fired there, a silent semantic divergence with no D-row. The `2080` is written on both sides rather
than algebraically cancelled to `phi-hour / median-wage`: the two are equal in the reals but **the frozen program
computes the un-cancelled form**, and the mirror is the contract (§8). `phi_coupling_binds_at_the_cap` therefore
needs a genuine high-Φ fixture again, which is what world 7 seeds.

Its `:material-basis` states the theorem in the Director's own terms: **imperial rent purchases entry into the
labor aristocracy** — Φ raises savings, savings raise accumulation, accumulation raises the P→LA rate, the LA share
grows. The rule runs **every tick** (no boundary gate) so the published value is always current-minus-one at the
boundary. The file gains its **first `D-N` header block** with this train, carrying D-NF+18 and its named re-open
trigger (**when the ImperialRent train makes `phi-hour` move within a year, the lag becomes observable**).

### 7a. The duplication ledger — every expression transcribed more than once

The rate/flow/normalize split buys independent mutation-killability at the cost of repeated sub-expressions.
**Single-sourcing is not available in the language** — a `.bsl`'s top-level forms are a closed set, there is no
`defexpr`, no macro and no cross-rule `let`, and a rule's `:expr` bindings are private to that rule. So the copies
get **copies-agree rows**, each a first-class named conformance test, not an assertion tucked inside another.

| duplicated expression | in | copies-agree row | asserts |
|---|---|---|---|
| `effective-wage` (`median-wage · 2080`, FR-017-halted) | `a01`, and `economics/phi-savings-coupling`'s divisor | `a01_and_phi_coupling_agree_on_the_wage_base` | the Φ adjustment computed from the same wage the accumulation rate uses, **bit-exact, including the halt** — a fixture at `$9.59/hr` must show BOTH a zero accumulation rate and a zero Φ adjustment. **This row was unwritable under rev 1's rule** (the two sides differed by 2080× and by the halt); it is the copies-agree row that would have caught C3, which is why it stays first in this table |
| the boundary gate `(= phase-of-year 0)` | all 13 rules — **including `a12`, which is why this row is true and not nearly-true** (rev 2.1, N1) | `the_pack_is_inert_off_the_boundary` | **every** published field byte-identical across two consecutive non-boundary ticks, `social-class/ternary-net-fascist` included — one test covering 13 copies. **The co-load worlds are where this bites hardest**: `consciousness/p6-route` moves the ternary every tick there, so an ungated `a12` would make this row false by construction |
| `target = 1 − (bourgeoisie + petit-bourgeoisie)` | `a07`, `a08`'s invariant assertion | `a07_and_a08_agree_on_the_fixed_share` | the committed shares sum to `target` bit-exactly |
| the three milestone constants | `a11`'s three guards, and the seeding-parity block | `the_milestones_match_the_ruled_values` | R5's `0.05/0.10/0.15` present exactly once per world, identical across every world that declares them |

Mutation evidence: perturb **one** copy only; the agreement row must flip while every single-rule row stays green.

### 7b. The D116 same-tick cross-rule ledger

`bsl-language.rst`'s D116 is a **RECORDED GAP, not a semantics guarantee**: each rule runs to completion (collect
and apply) before the next starts against the same mutable graph, so rule N+1 observes rule N's applied writes from
this tick. Its own text defers the repair (collect-across-rules-then-apply) to its own train. **Every reliance
below is deliberate and breaks loudly if D116 is repaired** — that is the point of recording it.

| reader | reads | written same tick by | breaks how, when D116 is repaired |
|---|---|---|---|
| `a05` | the four `territory/rate-*` fields | `a01`–`a04` | amplifies last tick's rates ⟹ every share moves by one boundary's lag |
| `a06` | the four amplified rates | `a05` | flows apply un-amplified rates ⟹ the whole crisis mechanism goes inert |
| `a07` | the three `raw-share-*` fields | `a06` | normalizes last boundary's raw shares ⟹ mass conservation still holds but the year is wrong |
| `a08` | `a07`'s normalized values | `a07` | commits un-normalized shares ⟹ sum-to-one fails |
| `a10` | the **pre-transition** LA share | `a08` (ordering decided at Task 7 Step 1) | the baseline captures the post-transition share ⟹ R10's arc under-reports by one boundary |
| `a11` | `a08`'s committed LA share and `a10`'s latch | `a08`, `a10` | the cascade compares mismatched vintages ⟹ spurious or missing events |
| `a13` | `social-class/ternary-net-fascist` | `a12` (**same tick**, `a12` sorts first) | folds the PREVIOUS BOUNDARY's publication ⟹ **the readout lags the ternary by a full year, not a tick** — the cost of rev 2.1's N1 decision, stated where the reliance lives: with both rules gated to the boundary, a D116 repair (collect-across-rules-then-apply) turns a same-tick read into a same-*year* read. Under the rejected `(when #t)` shape the same repair would have cost one tick. **This is the one thing option (b) was better at, and it is a recorded-gap consequence, not a live defect** — nothing observes it until D116's own train lands, and that train's ledger now names this row |

### 7c. Permanent anti-pattern guards

Three guards ship as named tests and stay in the suite after the train lands:

1. **`no_rule_binds_a_field_in_the_pack_namespace`** — a source-level assertion over `class-dynamics.bsl` that no
   `:field` binding's qname begins `class-dynamics/`. `subject_type_of` would otherwise demand a
   `NodeType/CLASS_DYNAMICS` that does not exist (§Global).
2. **`the_pack_declares_no_intrinsic`** — a source-level assertion that `class-dynamics.bsl` contains no
   `(intrinsic …)` form, protecting both the `floor` duplicate-declaration hazard (D-NF+22) and §6's
   no-transcendental verdict in one line.
3. **`the_pack_writes_only_territory_fields_and_one_class_field`** — a source-level assertion enumerating every
   `update-node` target qname, pinning the §2.3 cross-train disjointness argument mechanically so a future rule
   cannot quietly claim `social-class/agitation`.

---

## 8. The frozen-mirror recipe

Each mirror is a **standalone Python script** under `rust/crates/babylon-tick/content/scenarios/`, named
`class_dynamics_<world>_conformance.py`, and it is **the oracle** — not the frozen engine wrapped in a harness.

1. **Header:** this plan's path; the frozen source files with line counts; the ADR183 disclaimer verbatim
   (*structure/ordering oracle, not a byte oracle*); the exact reproduction command
   `PYTHONPATH="$PWD/src" uv run python <mirror>.py`.
2. **Body:** a literal `WORLD` dict mirroring the `.bscn`'s seeds **by node id**, then the rules' binding order
   transcribed **term for term** — not a call into `simulate_transitions`, except in the corroboration pass below.
3. **Corroboration pass:** a second function that *does* drive `DefaultClassTransitionEngine` +
   `DefaultAccumulationCalculator` + `DefaultSavingsRateSchedule` + `DefaultDispossessionCalculator` +
   `PhasedCrisisAmplifier` over the identical inputs, printing every intermediate rate and every output share at
   full `repr` precision.
   **A DISAGREEMENT IS A STOP FIRST AND A D-ROW ONLY AFTER IT IS DIAGNOSED.** Rev 1 wrote *"where the two disagree,
   the disagreement IS a D-row"*, and that rule would have **laundered C3**: the term-for-term half transcribes
   *this plan's terms*, so a unit error in the plan reproduces in the mirror, disagrees with the corroboration
   pass, and is then **recorded as an intended divergence** instead of caught. The repaired protocol, in order:
   (a) every disagreement halts the task; (b) the implementer classifies it against a **closed list of intended
   divergences** — the D-rows this plan has already declared (F11's `wage·s²`→`wage·s`, F10's degenerate arm,
   R10's cumulative baseline, and nothing else; **D-NF+16's `round()`-drop is deliberately NOT on the list,
   because it cannot arise on this path — the mirrors drive `DefaultClassTransitionEngine`, while the payload
   rounding lives at the call site, `system/__init__.py:1153-1170`, outside the oracle. If a rounding
   disagreement ever appears here, the oracle boundary has moved and that is itself the finding**, N13); (c) a disagreement **not on that list is a transcription bug in
   the port or in this plan**, and it is repaired, never recorded; (d) only a disagreement matching a declared
   D-row is written up as evidence for it. **The list is closed at Task 2, and any addition is a plan amendment
   with its reason attached** — that is what makes F11's repair evidence rather than a category into which any
   error can be filed.
   **Unit discipline, added because C3 was a unit error no float comparison alone could name:** every mirror prints
   each intermediate with its UNIT in the label (`wage_hourly=`, `wage_annual=`, `phi_per_hour=`,
   `rate_per_year=`), and the Rust test's doc comment carries the same labels. A quantity whose unit is not printed
   is not corroborated.
4. **The F11 double run:** the accumulation mirror prints **both** `wage·s²` (frozen) and `wage·s` (repaired) for
   every fixture, and both printouts are pasted into the Rust doc-comment. **The 33× factor is re-derived from
   these printouts, never trusted from this plan.**
5. **Paste, dated:** the mirror's **full verbatim stdout** goes into the Rust test file's module doc comment with
   the capture date, plus the "why exact equality, no tolerance" paragraph (ops here are `Real × Real` and
   `Real ÷ Real` only — correctly-rounded binary64 on both sides; no libm transcendental appears anywhere, §6).
6. **Never assert against a printed float.** Every Rust numeric assertion is measured from the engine's own run and
   compared bit-exactly via `.to_bits()`; the mirror's value appears in the doc comment as corroboration only.

---

## 9. BLOCKERS — flagged, not planned around

**BLOCKER-1 (HARD, disposition ready): `:year` / `:tick-of-year` are UNSERVED.** §4.4. The year becomes a
per-territory `int` field incremented once per boundary — exactly what `with_updated_dynamics` does. **No
escalation needed.** D-NF+3.

**BLOCKER-2 (language gap, named owner): no bare-float literal above 1.0.** `E-LEX-021` refuses bare non-integer
literals; `p`/`i`/`c` are `[0,1]`-bounded at lex (`reader.rs:869-886`); `r` (`Ratio`, `(0,∞)`) exists but its
operator surface is narrow. **Seven amplifier multipliers exceed 1 and are fractional.** Disposition: the landed
`x1e6` scaled-bare-Int lane, whose rationale is already written out in `metabolism-conformance.bscn:22-24` and
`territory-conformance.bscn:101`.
**Two corrections rev 1 owed here (I9).** (1) **Rev 1's spike form cannot load.** It wrote *"`(defconst
class-dynamics/deep-precaritization-x1e6 3500000)` read via `:const`, divided by `1000000`"* — both operands are
`Atom::Int`, and `Int ÷ Int` is a **loud error**, *"truncation is never implicit"* (`evaluator.rs:35`, `:1739`,
test at `:2044`) — the same trap rev 1's own type-trap list names two sections later. The legal form promotes
first: `(binding m :expr (/ (- deep-precaritization-x1e6 0c) 1000000))`, the landed `(- x 0c)` promotion idiom.
(2) **The cited precedent uses the OPPOSITE operand order, and the worse one.** `metabolism.bsl:386-387` multiplies
first (`(* raw-extraction entropy-factor-x1e6)`) then divides — **two roundings**, not bit-equivalent to the frozen
`rate * 1.2`. Divide-first is **one** rounding: `1200000/1000000` is the correctly-rounded double nearest `1.2`,
identical to the literal the frozen engine holds, so `rate * (scaled/1e6)` reproduces `rate * 1.2` bit-for-bit.
**This plan therefore diverges deliberately from the landed operand order**, and the divergence carries its
derivation in D-NF+5 rather than inheriting a precedent that would cost last bits. **Task 1 Step 3 still confirms
the readback empirically BEFORE 20 multipliers are written**; if it fails, choose a different scale or a declared
tolerance **with a written derivation** — never absorb a last-bit difference silently. D-NF+5.

**BLOCKER-3 (resolved by scope, and this is load-bearing): the `round()` half-even gap does not bind this pack.**
The census is precise: **7 payload-presentation sites** and **2 state-affecting integer demotions**, both of the
latter in `reserve_army/accumulation.py:115-123` — **outside this pack**. **In-scope state-affecting `round()`
sites: zero.** Payloads emit full precision. **Task 0 Step 2 verifies the census independently; a state-affecting
site inside the boundary is a STOP.** D-NF+16.

**BLOCKER-4 (dependency, named follow-on): `crisis-phase` has no producer on the ported estate.** The Step-5
five-phase detector is unported (§0.1), so `territory/crisis-phase` is declared input seeded per world. Every
amplifier row and the cascade's first gate read it, so **the pack's most theory-laden behavior is exercised only by
authored worlds until the crisis-detector train lands.** Honest and testable (eight content worlds cover five phases), but
it must appear in the pack header, the ADR, and the follow-on issue — **never discovered by the next reader**.
D-NF+27.

**BLOCKER-5 (III.11, resolved): there is no assertion construct.** The sum-to-one `model_validator`
(`types.py:70-83`) and `_validate_distributions`' `raise ValueError` (`:2460-2486`) have no BSL analogue. **F10
makes this cheap**: the flows conserve mass exactly, so sum-to-one is a **theorem of the arithmetic**, not a
runtime check. It lands as a **property test over every world** (`|Σ shares − 1| ≤ 1e-12`, tighter than the frozen
`0.001` tolerance) plus §4.5's validator bounds. A runtime guard would be a warning level, which S-11 forbids.
D-NF+28.

**BLOCKER-6 — RETIRED AT THE BYTE (I5). It was never a blocker.** §4.6: `rule_pipeline.rs:744-760` runs
`field_ref_for` over the `:weight` exactly as over the body and only then calls `destructure_aggregation` on the
adapted form, so `(field-of it social-class/population)` is a legal `:weight` spelling. There is **one checker with
an adapter**, not "two checkers, two shapes". The two-`fold sum` fallback, Task 8 Step 3's fallback branch and the
"re-plan of Task 8" warning are all **struck**; Task 1 Step 1 keeps a one-line confirmation only. **Rev 1 ranked
this the train's #1 variance item on the strength of an unread answer inside a file it listed as required
reading** — the re-ranked list is in §Estimate.

**BLOCKER-7 (HARD, resolved by design change — C2): a tick-1 golden pins nothing about a tick-52 rule.**
`run_once` is tick 1, tick 0 is never executed, and this pack's gate first opens at tick 52 (§4.4). Rev 1's seven
`run_once` pins would each have recorded `fired = 0` and `before == after`. **Disposition: every world takes TWO
pins** — a tick-1 **load pin** (`run_once`, which now deliberately pins the seeded world *and* the pack's
off-boundary inertness) and a tick-52 **boundary pin** (`TickSession::advance` ×52, `session.rs:120-160`, whose
`TickReport` carries the same `before`/`after` hashes `run_once` returns). `TickSession` is landed estate with a
landed multi-tick consumer (`carceral_arc_conformance.rs`, ticks 1/53/105/106); what is new is a **session-driven
hash pin inside `tick_goldens.rs`**, whose header today states a single-tick convention (`:697-706`). **Task 6
Step 4 lands the convention extension explicitly, with its own header paragraph, and it is a declared step — not a
side effect.** D-NF+30.

**BLOCKER-8 (HARD, resolved by §2.2 — C1): R9's ruled home is a SHARED file with four consumers.**
`fundamental-theorem.bsl` is `include_str!`d by `tick_goldens.rs`, `babylon-client/src/engine_link.rs`,
`babylon-client/tests/engine_link.rs` and `babylon-tick/src/lib.rs`; the world they all load, `two-classes.bscn`,
declares no `territory/*` field and no `defconst`, so the added rule dies at load twice over (`E-LOAD-010`;
`check_sources_servable`). **Disposition: the declaration-only scenario extension of §2.2.3** — hash-neutral by
`state_hash.rs`'s canonical layout, zero-firing by `subject_type_of` + `nodes()` — plus **one** Rust unit-test
repair. **If a reviewer rejects the scenario extension, STOP: the alternatives are splitting the file (which
forfeits R9's ruled home and is a Director call) or moving a pin (which is never allowed).** D-NF+29.

**BLOCKER-9 (PRE-EXISTING ESTATE DEFECT, inherited not caused — C4): `territory.bsl` and `decomposition.bsl`
cannot co-load at this HEAD.** Both declare `(intrinsic floor …)`; the loader refuses duplicates by name
(`declarations.rs:1037-1046` (the doc at `:1037`, the `DeclError::Duplicate` raise at `:1044`; `:1009` is the SignatureMismatch arm rev 1 inherited from `territory.bsl`'s own header — N12, mechanism unchanged)); issue **#646** is filed and open. **Disposition: two co-load worlds instead of one**
(§2.3), each naming what it proves and what the split costs. **This pack cannot fix it** — it declares no
intrinsic — and must not pretend a single co-load world exists. D-NF+22.

**NOT blockers — corrections Task 0's dossier must carry:**
- **`(nodes …)`-ranged folds ARE landed** (`decomposition.bsl:284-291`, `control-ratio.bsl:281-287`) — the
  2026-08-17 draft's "no landed content folds over `(nodes …)`" is **STALE**.
- **Fractional per-node seeding IS legal** (§4.3) — `dispossession.bsl:29-40`'s header claim is **STALE**.
- **`field-of` over an enum field is DISCHARGED** (D102) — `territory/crisis-phase` may be an honest
  `enum CrisisPhase`, not an int ordinal.
- **The string-identity gap does not bind** — this pack drops FIPS entirely (§4.1, ADR198 R7).
- **Events are observable and pinnable today** — `CollectingSink`'s `events: Vec<(String, Vec<(String, Value)>)>`
  is asserted key-by-key in landed tests; the "unpinnable pending WS1 (#502)" inventory row is **stale**.
- **`EventType` needs no Rust change** — it is a kind-checked closed vocabulary a scenario opts into per-kind.
- **`validation.py` carries 32 constants, not 29** (§1.6) — the dossier's split is corrected, the ruling is not.
- **The `:weight` operand shape is SERVED for `field-of`** (`rule_pipeline.rs:744-760`) — BLOCKER-6 retired.
- **Tick 0 is never executed; the first boundary is tick 52** (`session.rs:60-66,120-124`, `lib.rs:517-531`) — the
  spike question is answered from source, and the answer reshapes the pin design (BLOCKER-7).
- **No landed `.bscn` declares an `EventType` vocabulary** — verified, zero files under `content/` carry a
  `(defvocabulary EventType …)` form, and `vitality`/`lifecycle`/`solidarity`/`dispossession` all `emit` under
  pinned goldens. So the opt-in question rev 1 sent to a spike (Task 1 Step 5) is answered: **a world need not
  declare one** (M11). The spike survives only as the payload-key-idiom rehearsal for `a11`.
- **`production.bsl` contains no fold at all, and the double-count record is D136, not D45** (§2.4).

**Type traps to verify at Task 1, not at Task 8:** `Int ÷ Int` is a **loud error** (truncation is never implicit,
`evaluator.rs:35`, `:1739`) — check every quotient's operand types up front, **including the `x1e6` descale, which
rev 1 wrote in the illegal form** (I9). `if` takes exactly three operands and both branches share one static
type. The fold element name is the implicit `it`. `E-LEX-023` caps `p`/`i`/`c`/`r` at 9 fractional digits.
`update-node`'s op set is the closed `add | sub | set | scale`. There is **no `abs`**, **no `round`**, **no
`min`/`max`**, and **no `%`**.

---

## 10. DIRECTOR GATE — eleven questions, popup-ready

**No task in this plan decides any of these.** Seven are the rulings dossier's own still-open items, carried
forward verbatim with their citations; four are raised by this plan (DG-8's widened scope, DG-9, DG-10, and rev
2's DG-11). **Task 0 Step 1 posts all eleven to the docket (#564) in the SAME step, not at the last task.**
**Which answers gate which PR, corrected (M6):** **DG-7** gates Task 5's `p_to_l_component` retirement, which
lands in **PR C** — rev 1 said PR B, and Task 5 is not in PR B; **DG-8** (now the whole 41-row declined set, I1)
gates §4.5's disposition and Task 11's records, in **PR E**; **DG-9** gates Task 9's `phi_cap`, in **PR E**;
**DG-10** gates Task 11 Step 4's accounting, in **PR E**. **DG-11** gates nothing and is filed for the record.
A gating question still unanswered when its task runs follows that task's own written fallback — it never
resolves by implementation choice.

| # | question | why it is the Director's | gates |
|---|---|---|---|
| **DG-1** | **Is `f − r` the right one-axis projection of the three-way simplex?** The population-weighted `(fascist − revolutionary)` measure R6 ratifies discards the liberal middle's *magnitude*: a county at `(0.1, 0.8, 0.1)` and one at `(0.5, 0.0, 0.5)` both read `0.0`. *"That may be exactly right (net direction) or may hide the difference between hegemonic stability and polarized deadlock — which is a substantive claim about what bifurcation means."* (`reports/tickdynamics-trio-dossier-2026-08-17.md:404-407`) | a claim about **what bifurcation means** | nothing — `a13` ports the ruled formula either way; a "no" answer is a follow-on train, not a re-plan |
| **DG-2** | **Does the trio ruling touch ADR016's −1/+1 direction law itself, or only its TickDynamics instantiation?** R6 says the law keeps "ONE expression" but does not state whether the sign convention was re-opened or only re-homed. (`:408-410`; `memo-tickdynamics-reserved-trio.md:104-106`) | the ideological line | nothing — `a13` transcribes the ruled sign |
| **DG-3** | **Is the two-fixed / three-dynamic split still correct?** Bourgeoisie and petit-bourgeoisie are fixed by fiat (`types.py:31-32`) while LA / proletariat / lumpen are engine-driven. *"a claim that the top 10% has no mobility the model needs to represent — defensible for a game about the collapse of the core's labor aristocracy, but it is a claim."* (`:770-773`) R7 did not touch it. | a claim about class mobility | nothing — `a08` never writes the two fixed shares, per frozen |
| **DG-4** | **The theory-laden coefficient VALUES** — the `0.6/0.3/0.1` dispossession composite (foreclosure weighted **6×** eviction for LA→P), the five-class savings ladder, and the DEEP row `3.0/3.5/0.1/0.2` (how sharply crisis proletarianizes). *"These are pedagogy, not calibration."* (`:774-777`) R8 authorized their **home**, not their magnitudes; R5 blessed only the 0.40 / 5-10-15pp trio. | pedagogy | nothing — **treat all 46 defconsts as provisionally-valued content**, and say so in the ADR |
| **DG-5** | **Should ADR070 / Program 19's emergent partition eventually REPLACE the five-share taxonomy (D4 option C)?** R7 preserves it as *"the post-cutover target exactly as previously ruled"* — a target, not executed, no train chartered inside this port. (`ADR210:122-123`) | roadmap | nothing |
| **DG-6 ★ (upgraded in rev 2)** | **Run-start baseline vs. rolling window.** *"A run-start baseline means a county that recovers and re-declines never re-fires. A rolling multi-year window would. Both are cumulative; they teach different things about whether dispossession is reversible."* (`reports/tickdynamics-trio-dossier-2026-08-17.md:933-935`) **Rev 1 disposed this as "resolved by construction", on the ground that R10 names the `p7-persist-baselines` pattern and that pattern is a run-start seed. Read at the byte, `p7` writes its baselines EVERY TICK under its guard — a ROLLING persister** (`consciousness.bsl:340-353`; I3). The construction argument therefore runs the wrong way, and the question is **live**, not settled by inheritance. | **whether dispossession is reversible** — a teaching claim, and now one the plan cannot claim was already answered | **nothing blocks:** `a10` implements **run-start**, which is what R10's own words ("carried forward untouched") rule. The Director is asked in the knowledge that the cited pattern does not itself imply run-start; a "rolling" answer changes `a10`'s guard on a follow-on train, not this plan |
| **DG-7** | **May the workforce retire an output/event that has never fired, on its own authority?** Moot for `DISPOSSESSION_CASCADE` (R10 restores it) but **still open on the record** for register row 24's general case (`:936-937`) — and this train wants to retire `p_to_l_component` (computed, returned, never read) under exactly that authority. | governance precedent | **D-NF+11 / Task 5.** If the answer is "no", the P→L trio is declared as inert content instead and the D-row changes |
| **DG-8 (widened in rev 2)** | **Does R8's binding list reach the 41 rows this plan declines to declare — not just 32?** R8 binds on *"`transition_engine.py`'s 4 constants, `crisis.py`'s **2 legacy** + 20 phase-table multipliers, `dispossession.py`'s **6** composite weights, `savings_schedule.py`'s **5 class rates** + phi cap, `validation.py`'s thresholds"* (`dossier-rulings.md:136-138`). This plan declines **32** log-only `validation.py` thresholds (III.11 / S-11 has no warning level to receive them), **2** legacy `DefaultCrisisAmplifier` multipliers (D-NF+6), **3** P→L composite weights (D-NF+11) and **4** unread savings rates (D-NF+12) — all on the same "declare only what a rule reads" ground. **Rev 1 escalated the 32 and disposed the other 9 on its own authority, taking both positions in one document (I1).** | an interpretation of a Director ruling, and a governance question about whether the workforce may narrow a ruling's binding list at all | **§4.5 / Task 11.** A "declare them all" answer adds 41 `defconst`s × every world plus a mutation-vector obligation this plan does not budget; a "declare the 9, bound the 32" answer is also available and cheaper |
| **DG-9** | **Does R9's "with `phi_cap` promoted to a define" mean a `GameDefines` define, or the BSL defconst its sitting-mate R8 rules for the same estate?** This plan reads **defconst** (§Global, D-NF+19) because a real define moves `canonical_defines_hash`, costs an 11-baseline §6.5 ceremony, and has **zero effect on the Rust engine, which does not read `GameDefines` at all** — against R8's explicit "no defines.yaml churn; no §6.5 ceremony." | reconciling two clauses of one sitting | **Task 9.** A "real define" answer adds a declared ceremony and a Python-lane commit this plan otherwise forbids |
| **DG-10** | **Checkpoint-A accounting.** This train is chartered as the 13th and FINAL Material Base system, but the measured roster (§0.2) shows **ReserveArmy @5.0 unstarted**, Community @6.0 and ImperialRent @9.0 in flight, and this train porting Feature-016 rather than all of @4.0. Does R14's "all 13 Material Base systems ported" mean **whole-system**, and does **WS3 stay HELD** after this landing? | what closes a checkpoint, and when WS3 fires | **Task 11 Step 4's #557 accounting update.** The plan's default is: **Checkpoint A NOT closed, WS3 stays HELD** |
| **DG-11 (new in rev 2)** | **Does the Director additionally want the frozen-lane `ClassDistribution` docstring edit R7's consequence text names?** ADR210's consequences say R7 *"rewrites the `ClassDistribution` model's field DESCRIPTIONS, not its values"*, but ADR183 R2 forbids frozen-lane repair and the model is reference-only after the `p27-python-freeze` pin. This plan lands R7 in CONTENT (the `deffield` rows, the `:material-basis` provenance, the pack header) and **does not touch `types.py`**. Rev 1 asked this question under the label DG-8, which its own §10 table had already assigned to R8 — one of three DG collisions (M6), fixed here. | reconciling a consequence clause with a standing prohibition | **nothing blocks** — D-NF+26 records the reading either way; a "yes" answer is a separate Python-lane commit on the WS4/python-deletion ledger, never this train |

**Also carried to the docket, not as questions but as recorded facts the Director asked to see:** R4's asymmetry is
implemented as a **guarded, named test** (§4.7) rather than as a comment; R5's three confirmations are each pinned
by a mutation vector; and R10's arc test is the executable form of the ruling and is quoted in PR D's body.

---

## File Structure

| File | Responsibility |
|---|---|
| Create `reports/class-dynamics-bsl-surface-facts-2026-08-18.md` | Task 0's dossier — the owed re-reads, the **eleven** corrections (§9's list), the `round()` census, the numbering allocation, the collision grep, **and the `fundamental-theorem.bsl` consumer enumeration (§2.2.1) re-run at that HEAD** |
| Modify `rust/crates/babylon-tick/src/lib.rs` | **One** registration string (`"class-dynamics"`) in the systems `HashSet`, with the landed comment style and the §0.1 boundary stated in it |
| Create `rust/crates/babylon-tick/content/rules/class-dynamics.bsl` | 13 rules + the pack-local `D-N` header block + the SPIKE RESULTS block |
| Modify `rust/crates/babylon-tick/content/rules/fundamental-theorem.bsl` | **+1 rule** (R9's ruled home) + the file's **first** `D-N` block |
| **Modify `rust/crates/babylon-tick/content/scenarios/two-classes.bscn`** | **DECLARATION-ONLY** — the `deffield`/`defconst` rows R9's rule binds, no node form, no attribute, no edge. Hash-neutral by `state_hash.rs`'s canonical layout (§2.2.3). **The one landed-content file this train touches beyond its own pack, and the one a reviewer must read hardest** |
| Create 10 × `content/scenarios/class-dynamics-*.bscn` | The worlds matrix — **eight content worlds + two co-load worlds** (rev 1's File Structure said "7" while its own tasks created eight, M5; rev 2 adds the empty-fold witness world, C5, and splits the co-load world in two, C4) |
| Create 4 × `content/scenarios/class_dynamics_*_conformance.py` | Frozen mirrors (primary, deep-crisis, cascade-arc, phi) |
| Create `rust/crates/babylon-tick/tests/class_dynamics_conformance.rs` | Rates / amplifier / flows / normalize / year / readout + mutation vectors + the three §7c guards |
| Create `rust/crates/babylon-tick/tests/class_dynamics_cascade.rs` | The multi-boundary arc, R10's headline test, both events |
| Modify `rust/crates/babylon-tick/tests/tick_goldens.rs` | **18 additive pins** — 8 load pins (tick 1, `run_once`) + 8 boundary pins (tick 52, `TickSession`) + 2 further arc pins (ticks 104, 156) — **plus the file-header paragraph declaring the first session-driven pin convention in this file** (BLOCKER-7). The **16** existing pins untouched, and `babylon-client`'s 17th unmoved |
| **Modify `rust/crates/babylon-tick/src/lib.rs` (test module)** | **One** unit-test repair: `single_rule_content_still_reports_fired_and_a_one_entry_per_rule_fired` → `per_rule_fired.len() == 2`, renamed, with the added assertion that the Φ rule contributes **0** on this world (§2.2.3). Not a pin, not a baseline |
| Modify `docs/reference/bsl-language.rst` | Register rows `D-NF+1 … D-NF+32` (allocated at Task 12) |
| Create `ai/decisions/ADR-NF_class_dynamics_port_handoff.yaml` + the `index.yaml` row | Handoff record (allocated at Task 12) |
| Modify `reports/port-inventories/tick-dynamics-port-phase1-inventory-2026-08-12.md` | Post-train UPDATE block (verdict, the §0 boundary, the corrections) |
| Modify `ai/state.yaml` | Closing entry (prepend to `current_focus.recently_completed`; bump `updated:`) |
| **Never modified** | **anything under `src/babylon/`, `tests/`, `tests/baselines/**`, `src/babylon/data/defines.yaml`** — and **no hash in `tick_goldens.rs`'s existing 16 or in `babylon-client/tests/engine_link.rs`** |

---

### Task 0: Governance, measurement, and the starting line

**Files:** Create `reports/class-dynamics-bsl-surface-facts-2026-08-18.md`.

- [ ] **Step 1: Open the implementation issue** on project 8 under the Checkpoint A umbrella (#557), linking #563
      (the trio charter), #564 row 21, ADR208 R14/R15, ADR210 R4–R11, ADR183, and this plan. State §0.1's
      Feature-016 boundary **and §0.2's Checkpoint-A accounting** in the issue body so the scope is public before
      code lands. **Post all ELEVEN §10 DIRECTOR GATE questions to the docket (#564) in this same step** — DG-7
      gates Task 5's `p_to_l_component` retirement (**PR C**, not PR B — rev 1 mis-stated it, M6), DG-8 gates
      §4.5's disposition of all **41** declined coefficient rows (I1), DG-9 gates Task 9's `phi_cap`, DG-10 gates
      Task 11's accounting; DG-6 is posted **upgraded** (its "resolved by construction" premise does not hold,
      I3) and DG-11 is posted for the record. Note in the issue that answers are needed **before PR C opens**
      (DG-7) and **before PR E opens** (DG-8, DG-9, DG-10).
- [ ] **Step 2: Re-verify the `round()` census INDEPENDENTLY** — `rg -n '\bround\(' src/babylon/domain/economics/`
      across all seven `dynamics/` modules **and** `tick/system/__init__.py`. Record every site with its line and
      classify presentation-vs-state. **If any state-affecting site lands inside §0.1's boundary, STOP** —
      BLOCKER-3's resolution depends on the in-scope count being **zero**.
- [ ] **Step 3: RE-MEASURE both numbering tails and fix this train's allocation.**
      `rg -o 'D[0-9]+' docs/reference/bsl-language.rst | sort -u -V | tail -5` and `tail -8 ai/decisions/index.yaml`.
      Measured 2026-08-18: **D180** (`bsl-language.rst:8158`), **ADR214**. Record the tail measured **today**, record
      the four-way contention (#491's committed D181; ImperialRent's literal D181–D201 and its already-taken
      `ADR214_…` filename; Community's 25 `D-NF+n` + `ADR-NF`), and fix this train's allocation as
      `D<tail+1> … D<tail+32>` and `ADR<tail+1>`. **Every later task uses that allocation; Task 12 re-measures once
      more before filing.**
- [ ] **Step 4: The CROSS-TRAIN COLLISION GREP (§2.3).** `rg -n 'ternary-net-fascist'` and one grep per §4.2
      `territory/` qname across `rust/crates/babylon-tick/content/`, plus
      `/media/user/data/worktrees/wt-imperialrent/docs/superpowers/plans/` and
      `/media/user/data/worktrees/wt-community/docs/superpowers/plans/`. **Zero hits is the precondition.** A hit
      is a STOP; the resolution is a rename in *this* pack.
- [ ] **Step 4b (NEW, rev 2): THE SHARED-FILE CONSUMER ENUMERATION.** Before any task touches
      `fundamental-theorem.bsl`, re-run §2.2.1's census **at that HEAD**:
      `rg -n 'fundamental-theorem' rust/ --glob '!*.md'` plus `rg -n 'two-classes.bscn' rust/`. Record every
      `include_str!` consumer, every hash it asserts and every non-hash assertion it makes (today: 4 consumers,
      2 pinned hashes, `per_rule_fired.len() == 1`). **A consumer this plan does not list is a STOP** — rev 1
      missed `babylon-client` entirely, which is C1.
- [ ] **Step 5: Owed re-reads, recorded verbatim with line numbers** — (a) `tick.rs:419-470`'s `:year` /
      `:tick-of-year` refusal texts and `bindings.rs:55-59`/`:410-416`'s `:tick-in-cycle`; (b)
      `typecheck.rs:130-236`'s five fold arms, `UnweightedMeanOfIntensive`, `NonExtensiveWeight`, and
      `destructure_aggregation`'s accepted operand shapes; (c) `rule_pipeline.rs:640-708`'s `field_ref_for`
      **four**-shape law, `:744-760`'s weight adapter and its `compound_fold_error`; (d) `scenario.rs:1093-1330`'s
      five seeding arms and `load_defconst`, **plus `:1236-1275`'s node hydration — the proof that a
      declared-but-unseeded field is never stamped, on which §2.2.3's hash-neutrality rests**; (e)
      `grammar.rs:672-683` (`FoldOp`), `:724` (`ARITH`), `:649` (`if` arity); (f) `reader.rs:869-886` (the
      `p`/`i`/`c` `[0,1]` bound, the 9-digit cap and the `r` domain); (g) `declarations.rs:125-131`
      (`DECLARABLE_INTRINSICS` at `:125`, `PROHIBITED_INTRINSIC_NAMES` at **`:131`** — measured; **rev 2's
      "`:132`, outside rev 1's cited range" was wrong twice, from the same unmeasured M8 row, N4**) and
      **BOTH** `floor` declarations (`territory.bsl:78`, `decomposition.bsl:212`) with #646's disclosure text;
      (h) `bindings.rs:448-451` (`:optional` licenses absent VALUES, never unknown NAMES) and `tick.rs:439-451`
      (`check_sources_servable`) — the two refusals §2.2.2 turns on; (i) `state_hash.rs:10-30`'s canonical layout,
      quoted, as the evidence for §2.2.3; (j) `evaluator.rs:139-147` (`E-EVAL-020` store range, `E-EVAL-021`
      empty aggregate) and `territory.bsl:168-172`'s `exists` protector.
- [ ] **Step 6: Observe the starting line** — run `mise run rust:check` **single-flight** (§Global machine safety;
      check no sibling worktree is mid-gate first) and record: the exact count of `#[test]` functions and
      `*_hashes_are_pinned` pins in `tick_goldens.rs` (**expected 18 / 16**), the registered-systems list verbatim
      from `lib.rs:277-352` (**expected 13 strings, no `tick-dynamics`** — the range is `:277-352`, not rev 1's
      `:277-343`, M8), and **all 17 pinned hashes pasted into the dossier as the byte-identity baseline every later
      gate compares against — the 16 in `tick_goldens.rs` AND `babylon-client/tests/engine_link.rs`'s
      `783f651d…7679`.** Run `cargo test -p babylon-client --test engine_link` in the same single-flight leg.
- [ ] **Step 7: Write the dossier** with sections: (1) confirmed findings; (2) **CORRECTIONS** — §9's eleven
      "NOT blockers" rows, each with its citation, including the corrected `validation.py` count (**32, not 29**),
      the two stale landed-header claims, the `:weight`-is-served finding, the tick-0 finding, the
      D45→D136 correction and the `EventType`-vocabulary finding; (3) the type-trap list; (4) the `round()` census; (5) the numbering
      allocation + contention; (6) the collision-grep result; (7) §0.2's Checkpoint-A roster table, measured.
- [ ] **Step 8: Commit** `docs(port): class-dynamics BSL surface-facts dossier (owed re-reads, eleven corrections, the numbering allocation)`.

**Gate:** docs only — but run `vale` over the new Markdown, and `mise run rust:check` **once**, single-flight, for
Step 6's baseline. **Estimate:** ~3h · ~45k tokens.

---

### Task 1: THE SPIKE — two unprecedented shapes proved, three source-answered facts confirmed

**Files:** temporary spike rules in `rust/crates/babylon-tick/content/rules/class-dynamics.bsl` and a temporary
`content/scenarios/class-dynamics-spike.bscn` (both deleted at the end of the task; verdicts recorded in the pack
header's SPIKE RESULTS block and in Task 0's dossier).

**Rev 2 rescopes this task. Three of rev 1's five "unverified" items were answered at the byte during the rev-2
verification pass (§9's NOT-blockers list), so they land here as CONFIRMATIONS with their source citation
attached, not as open questions — and the two genuinely unprecedented shapes get the real spike.** A reviewer
should check that the two remaining spikes landed as REAL spikes, and that no confirmation was upgraded back into
an open question to look thorough.

- [ ] **Step 1: CONFIRM the weighted fold (BLOCKER-6 — retired, I5).** Source answer, recorded first:
      `rule_pipeline.rs:744-760` reduces the `:weight` through `field_ref_for` exactly as the body, so
      `(field-of it social-class/population)` is legal. Load ONE rule with that exact shape and record that it
      loads. **No fallback branch, no two-`fold sum` alternative, no Task-8 re-plan.** If it somehow refuses,
      that is a STOP and a source contradiction to escalate, not a fallback to take.
- [ ] **Step 2: CONFIRM the boundary gate; the tick-0 question is already answered (BLOCKER-7).** Source answer,
      recorded first: `TickSession::new` starts at `tick: 0`, `advance` runs tick 1 first (`session.rs:60-66,
      120-124`), `run_once` is tick 1 (`lib.rs:517-531`), so **tick 0 never executes and the first boundary is
      tick 52**. Confirm with a `(binding phase-of-year :tick-in-cycle 52)` + `(when (= phase-of-year 0))` rule
      over a **≥105-tick** session: `fired` must be 0 on ticks 1–51, non-zero on 52, 0 on 53–103, non-zero on 104.
      **Record the `fired` series**; it is the arithmetic every world's boundary pin inherits.
- [ ] **Step 2b (NEW): Spike the session-driven GOLDEN, since it is a new convention in `tick_goldens.rs`.**
      Drive `TickSession::advance` ×52 over a throwaway world and read `hex(&report.before)`/`hex(&report.after)`
      back; confirm the hashes are stable across two runs in one process and across two processes (the
      determinism half). **This is the shape Task 6 Step 4 lands as the file's first multi-tick pin** — prove it
      here, not there.
- [ ] **Step 3: Spike the scaled-int `x1e6` lane (BLOCKER-2), IN THE LEGAL FORM.**
      `(defconst class-dynamics/deep-precaritization-x1e6 3500000)` read via `:const`, then **promoted before the
      divide** — `(binding m :expr (/ (- deep-precaritization-x1e6 0c) 1000000))` — because **`Int ÷ Int` is a
      loud error** and rev 1's spelling could not have loaded (I9). Assert the product `m × rate` reads back
      **bit-exactly** equal to the mirror's `3.5 × rate`. **Also record, in the SPIKE RESULTS block, that this
      operand order deliberately differs from `metabolism.bsl:386-387`'s multiply-first order, with the
      one-rounding-vs-two derivation** (§9 BLOCKER-2). If the readback is not bit-exact, **record it and choose**
      (a) a different scale, or (b) a declared tolerance **with a written derivation** per the
      cross-implementation-tolerance standard. **Do not discover this at Task 4 with 20 multipliers written.**
- [ ] **Step 4: Spike the enum seed + read pair.** `(node … (crisis-phase CrisisPhase/DEEP))` plus
      `(binding phase :field territory/crisis-phase)` and `(= phase CrisisPhase/DEEP)` inside an `if`, confirming
      D102's discharge on **both** the seeding path (`E-LOAD-056`'s member-only rule) and the read path. Record the
      `defenum` ordinal parity for `NORMAL, ONSET, EARLY, DEEP, RECOVERY` (**hash-bearing**, ADR195).
- [ ] **Step 5: Rehearse `emit`'s payload-key idiom.** One `(emit EventType/DISPOSSESSION_CASCADE (fips 1)
      (decline 0.05c))` and one assertion over the `CollectingSink`'s `events` vector, key by key — fixing the
      idiom `a11` will use. **The vocabulary question is already answered (M11): no landed `.bscn` declares an
      `EventType` vocabulary and four packs emit under pinned goldens, so a world need not opt in.** Record the
      confirmation, not the question.
- [ ] **Step 5b (NEW): Spike the empty-TENANCY fold protector (C5).** A TERRITORY with **no** incoming-TENANCY
      class, and `a13`'s two-binding shape: first WITHOUT the `exists` protector — **record the exact
      `E-EVAL-021` text and confirm it kills tick 1 despite the `(when (= phase-of-year 0))` gate**, which is the
      whole point — then WITH it, confirming the tick survives and no score is written. **This one refusal is the
      evidence for the C5 repair; without it the protector reads as defensive decoration.**
- [ ] **Step 6: Delete every spike artifact**; write the **SPIKE RESULTS** block (dated: two spikes, one new-
      convention spike, one protector spike, and three source-answered confirmations, each with its evidence
      line) into the pack header and into Task 0's dossier.
- [ ] **Step 7: Commit** `test(tick): class-dynamics spike — x1e6 descale, empty-fold protector, session golden, and three source-answered confirmations`.

**Gate:** crate-scoped `cargo test -p babylon-tick` during the loop; the six-leg gate once at the end,
single-flight. All **17** pre-existing pinned hashes byte-identical (16 in `tick_goldens.rs` + `babylon-client`'s). **Estimate:** ~5h · ~70k tokens — matching the §Estimate table's Task-1 column and the rev-2 delta's "Task 1 −1h"; rev 2 left the pre-rescope ~6h/~80k here, and the per-task lines then summed to 72h/1,020k against the table's 71h/1,010k (N7).

---

### Task 2: Registration, the declaration surface, world 1, and the primary mirror (PR A)

**Files:** Modify `rust/crates/babylon-tick/src/lib.rs`; create
`content/scenarios/class-dynamics-conformance.bscn`, `content/scenarios/class_dynamics_conformance.py`,
`rust/crates/babylon-tick/tests/class_dynamics_conformance.rs`.

**Interfaces:** produces the node ids, the 17-field roster, the 46-constant canonical block, and the mirror numbers
every later task asserts against.

- [ ] **Step 1: Failing load-smoke test** — `class_dynamics_conformance.rs` with
      `const SCENARIO: &str = include_str!(…)` calling the real loader against an empty rule source. Expected:
      FAIL (unregistered system / `E-LOAD-002`). **Assert the refusal text first**, so the behaviour change is
      visible in the diff (the `production_conformance.rs` registration-probe idiom).
- [ ] **Step 2: Register the system** — add `"class-dynamics".to_owned()` to the `HashSet` (`lib.rs:277-352`) with
      a comment in the landed `"dispossession"`/`"production"` style naming the port train, the frozen system and
      tick position, **whether the registration is genuinely new** (Task 0 Step 6 measured it as such — zero prior
      hits), **and §0.1's boundary in one clause**: *Material Base @4.0's Feature-016 class-dynamics engine — NOT
      all of @4.0.* Hyphenated spelling follows the ruled convention (`"social-class"`, `"control-ratio"`).
- [ ] **Step 3: Write world 1** — `(defenum CrisisPhase (NORMAL ONSET EARLY DEEP RECOVERY))` in the landed order;
      `(defvocabulary NodeType (SOCIAL_CLASS TERRITORY))` and `(defvocabulary EdgeType (TENANCY))`; every
      `deffield` of §4.2; the **46-constant canonical block** of §4.5, with the seven `x1e6` rows carrying D-NF+5's
      rationale inline and each constant carrying its frozen `file:line` provenance comment. Nodes in **declaration
      order = NodeId order** (never renumber when extending):

      | node | type | role |
      |---|---|---|
      | `wayne` | TERRITORY | the primary county: the five bootstrap shares, `dist-year 2010`, `crisis-phase NORMAL`, `median-wage 21.0`, `unemployment-rate 0.05`, `phi-hour 0.0`, the three default rates, `baseline-la-known 0` |
      | `oakland` | TERRITORY | a second county with **different** shares and rates — proves nothing is globally flattened |
      | `wayne-prole` | SOCIAL_CLASS | TENANCY→`wayne`; `population`, ternary at the ruled rest state `(0,1,0)` |
      | `wayne-la` | SOCIAL_CLASS | TENANCY→`wayne`; non-zero `fascist` so `a13`'s readout is non-zero |
      | `shared-class` | SOCIAL_CLASS | TENANCY→**both** counties — the **D136 dual-membership vector** (§2.4): it enters BOTH counties' means at its own weight, and that is correct for a mean where it was wrong for a sum |
      | `orphan-class` | SOCIAL_CLASS | **no** TENANCY edge — must contribute to no county |

      **Seed every declared field on every node of its namespace** (the no-defaults law). Fractional seeds are
      legal on `real`/`probability`/`intensity`/`coefficient` (§4.3); `int` fields refuse them. The world's header
      states, in the `control-ratio-conformance.bscn` style, **which seeded value proves which gate, by name**.
- [ ] **Step 4: Write the primary mirror** (`class_dynamics_conformance.py`) per §8 — the term-for-term
      transcription **plus** the `DefaultClassTransitionEngine` corroboration pass, **run twice** for F11 (frozen
      `wage·s²` and repaired `wage·s`), both printouts recorded.
- [ ] **Step 5: Load-smoke green**, plus the `defenum` ordinal-parity test mirroring the mint (ADR195), plus the
      **cross-world constant-parity harness** the later worlds will extend, plus §7c's three anti-pattern guards
      (they can assert over an empty pack today and must stay green as rules land).
- [ ] **Step 6: Commit** `test(tick): class-dynamics registration, declaration surface, world 1 and the frozen mirror`.
- [ ] **Step 7: Open PR A** on `feature/tickdynamics-port-bsl` (Tasks 0–2, 3 commits). **Review lens:** the
      declaration surface against §4.2/§4.5 field by field, the spike verdicts as evidence, and §0's boundary
      argument read as the train's scope contract.

**Gate:** six legs, single-flight; all **17** pre-existing pinned hashes byte-identical. **Estimate:** ~7h · ~95k tokens.

---

### Task 3: `a01`–`a04` — the four rate constructors, with the F11 repair (PR B)

**Files:** Modify `content/rules/class-dynamics.bsl`; extend `class_dynamics_conformance.rs`.

- [ ] **Step 1: Failing tests** — `a01_computes_the_repaired_accumulation_rate` (bit-exact against the **repaired**
      mirror printout, with the frozen `wage·s²` number recorded in the test comment as the divergence D-NF+7
      names); `a01_halts_accumulation_strictly_below_the_floor` (**two** fixtures at `9.60` and `9.59` — the strict
      `<`); `a01_clamps_at_the_max_accumulation_rate` (the F14 clamp, which **only binds after the repair**);
      `a02_weights_the_dispossession_composite_6_3_1`; `a03_weights_unemployment_not_eviction` (D-NF+14 — a fixture
      where **swapping** the two weights changes the answer); `a04_stabilization_uses_0_15_not_0_10` (D-NF+13).
- [ ] **Step 2: Write the pack header** — the pack-local `D-N` block reserving a row for every D-record-table entry
      that touches this pack, each citing its global number; **R4's ruling quoted verbatim** with its ADR016 prior
      art; **R5/R6/R7/R8/R9/R10/R11 each quoted in one line with its citation**; the byte-order map `a01 → a13`
      with every same-tick dependency named (§7b); the SPIKE RESULTS block; §0.1's boundary statement; and
      BLOCKER-4's `crisis-phase` honesty paragraph.
- [ ] **Step 3: Write `a01`–`a04`** per §7, each with the boundary gate. **Check every quotient's operand types
      against the `Int ÷ Int` loud error before writing it.** Use nested `if` for every `min`/`max` (there is no
      scalar intrinsic).
- [ ] **Step 4: Tests green; pin the exact bits** measured from the engine's own run and cross-checked against the
      mirror's doc-comment printout.
- [ ] **Step 5: Mutation** — swap the `0.6/0.3/0.1` weights; flip the halt comparison to `<=`; change `0.15` to
      `0.10`; change `0.08` to `0.15`; restore the `wage·s²` square. **Each must flip a NAMED test.** Restore
      byte-identical and record every mutation in the commit body.
- [ ] **Step 6: Commit** `feat(tick): class-dynamics a01-a04 — the four rate constructors, with the F11 repair`.

**Gate:** crate-scoped during the loop; six legs at the end, single-flight; all **17** pre-existing pinned hashes byte-identical.
**Estimate:** ~6h · ~85k tokens.

---

### Task 4: `a05` — the FR-006 phase amplification table + two worlds

**Files:** Modify `content/rules/class-dynamics.bsl`; create
`content/scenarios/class-dynamics-deep-crisis-conformance.bscn`,
`content/scenarios/class-dynamics-phase-matrix-conformance.bscn`,
`content/scenarios/class_dynamics_deep_crisis_conformance.py`; extend the test.

- [ ] **Step 1: Failing tests** — one per amplifier row (`a05_normal_is_passthrough`, `a05_onset_row`,
      `a05_early_row`, `a05_deep_row`, `a05_recovery_row`), each asserting **all four** amplified rates bit-exactly;
      `a05_clamps_each_product_at_one` (a fixture whose base rate × multiplier exceeds 1);
      `a05_every_multiplier_is_individually_provable` (the **converse** vector: the other fixtures must NOT move
      when one multiplier is mutated); and `a05_unexercised_arms_are_unreachable_in_this_world` per §Global's
      dispatch-arm rule.
- [ ] **Step 2: Write `a05`** as a nested `if` on `crisis-phase` — three operands each, one static type per branch,
      the landed `(- 0 0c)` / `(- 1 0c)` promotion idiom. The seven `x1e6` constants divide inside the rule, per
      Task 1 Step 3's verdict.
- [ ] **Step 3: Write the two worlds** — DEEP alone (the R6/DG-4-flagged `3.0/3.5/0.1/0.2` row) and a three-county
      ONSET/EARLY/RECOVERY matrix. Each header names **the constants it makes provable, by name**, and states
      **why the phase is seeded rather than computed** (BLOCKER-4). Extend the constant-parity harness to three
      worlds and name every deliberate variation.
- [ ] **Step 4: Mutation — all 20.** Change each multiplier in turn; record which named test flips for each.
      **A multiplier with no killer is dead content — add a fixture, never accept it.** Restore byte-identical.
- [ ] **Step 5: Commit** `feat(tick): class-dynamics a05 — the FR-006 phase amplification table`.
- [ ] **Step 6: Open PR B** `feature/class-dynamics-rates`, off **merged dev** (Tasks 3–4, 2 commits).
      **Review lens:** transcription fidelity against §1.2 line by line, and the mutation ledger's completeness
      (**20/20 multipliers killed**). **DG-7's answer must be in hand before this PR merges** if Task 5's
      `p_to_l_component` retirement is to land as written.

**Gate:** six legs, single-flight; all **17** pre-existing pinned hashes byte-identical. **Estimate:** ~6h · ~85k tokens.

---

### Task 5: `a06`–`a08` — the flow equations, the rescale, and the degenerate repair (PR C)

**Files:** Modify `content/rules/class-dynamics.bsl`; create
`content/scenarios/class-dynamics-degenerate-conformance.bscn`; extend the test.

- [ ] **Step 1: Failing tests** — `a06_applies_the_three_flow_equations` (bit-exact per share);
      `a06_conserves_mass_exactly` (**F10**: `|Σ raw − Σ prior| ≤ 1e-15` — the property that makes BLOCKER-5 cheap);
      `a07_rescale_is_the_identity_when_nothing_clamped` (the scale factor is provably 1.0);
      `a07_rescales_after_a_clamp` (a fixture where `max(·,0)` bites);
      `a07_preserves_the_distribution_in_the_degenerate_case` (**D-NF+8's repair**) **plus**
      `a07_does_not_write_equal_thirds` (the explicit **anti-assertion**, so a future reader cannot restore the
      frozen constant silently); `a08_never_writes_the_two_fixed_shares`; and the world-wide property test
      `every_world_sums_to_one_within_1e_12`.
- [ ] **Step 2: Write `a06`–`a08`** per §7. `a07`'s two `guard`s partition on `total > 0` / `total = 0`, and the
      zero arm's effect list is **empty** — no number is fabricated (III.11).
- [ ] **Step 3: Write the degenerate world** — rates seeded so `max(·,0)` zeroes all three dynamic shares. Its
      header states that this world exists **only** to make D-NF+8 provable, that the frozen engine writes
      `target/3` here, and that **this is the only fixture that reaches the branch** — without it the repair is
      unprovable.
- [ ] **Step 4: Retire `p_to_l_component` explicitly** — record D-NF+11 in the pack header with **DG-7's answer
      quoted**. If DG-7 is unanswered when this task runs: **land the retirement as planned but mark the D-row
      PROVISIONAL and say so in the PR body**; if DG-7 returns "no", declare the P→L trio as inert content and
      rewrite the D-row instead. **Do not decide it here.**
- [ ] **Step 5: Mutation** — restore the equal-thirds write (`a07_does_not_write_equal_thirds` flips); drop the
      `max(·,0)` clamp (`a07_rescales_after_a_clamp` flips); negate one flow term (`a06_conserves_mass_exactly`
      flips); write a fixed share in `a08` (`a08_never_writes_the_two_fixed_shares` flips). Restore byte-identical.
- [ ] **Step 6: Commit** `feat(tick): class-dynamics a06-a08 — the flow equations, the rescale and the degenerate repair`.

**Gate:** six legs, single-flight; all **17** pre-existing pinned hashes byte-identical. **Estimate:** ~5h · ~75k tokens.

---

### Task 6: `a09` — the year axis, the boundary contract, and the FIRST SESSION-DRIVEN GOLDEN CONVENTION

**Files:** Modify `content/rules/class-dynamics.bsl`; modify `rust/crates/babylon-tick/tests/tick_goldens.rs`;
extend the test.

- [ ] **Step 1: Failing tests** — `a09_increments_the_year_once_per_boundary` (a **≥105-tick** session giving
      exactly **two** boundaries, ticks 52 and 104: 2010 → 2011 → 2012, and **no** movement on any other tick —
      indexed per §4.4's source-answered tick-0 fact, which Task 1 Step 2 confirms rather than discovers);
      `a09_clamps_at_2030` (a fixture seeded at 2030); `a09_clamps_at_2007` (seeded at 2006 — **record whether the
      `int` field accepts it at load at all**; if it does, the clamp must lift it);
      **`the_pack_is_inert_off_the_boundary`** — every published field byte-identical across two consecutive
      non-boundary ticks, the executable form of the frozen `tick % 52` gate and §7a's 13-copy agreement row.
- [ ] **Step 2: Write `a09`** per §7 — the increment plus the collapsed two-sided clamp in one expression
      (§1.5's six tree-wide sites, two of them in boundary; D-NF+4).
- [ ] **Step 3: Mutation** — change `:tick-in-cycle 52` to `26` (`the_pack_is_inert_off_the_boundary` flips);
      remove the `min(·, 2030)` (`a09_clamps_at_2030` flips); remove the `max(·, 2007)` (`a09_clamps_at_2007`
      flips). Restore byte-identical.
- [ ] **Step 4: LAND THE PIN CONVENTION, then the first six pins (BLOCKER-7 / C2).** This step is where
      `tick_goldens.rs` gains its **first session-driven pin**, and the convention change is declared in the file,
      not implied by a diff:
      (a) add a `run_to_tick(scenario, rules, n)` helper built on `TickSession::advance` (the landed multi-tick
      driver `carceral_arc_conformance.rs` already uses), returning the final tick's `TickReport`;
      (b) add a paragraph to the file's module doc-comment stating **why** — the existing convention is tick 1
      alone (`:697-706`), and a pack whose rules gate on `(= phase-of-year 0)` is **inert at tick 1**, so a
      tick-1 pin over this pack would record `fired = 0` and `before == after` and pin nothing;
      (c) add **six pins for the three worlds available at this task** (world 1, deep-crisis, degenerate): one
      **load pin** each at tick 1 — asserting `before == after` and this pack's `fired == 0` **across all thirteen
      rules, `a12` included (rev 2.1's N1 decision is what makes this assertion true in a world holding classes)**,
      which makes the pin the executable form of `the_pack_is_inert_off_the_boundary` — and one **boundary pin**
      each at tick 52,
      with the per-rule-id `fired` arithmetic in the assertion message;
      (d) every hash **MEASURED** from the engine's own run. All **17** pre-existing pinned hashes byte-identical
      — **a move is a STOP, not a re-measure.**
- [ ] **Step 5: Commit** `feat(tick): class-dynamics a09 — the annual boundary, the year axis, and the first session-driven goldens`.
- [ ] **Step 6: Open PR C** `feature/class-dynamics-flows`, off **merged dev** (Tasks 5–6, 2 commits).
      **Review lens:** the mass-conservation property test read as the reason no runtime assertion is needed;
      D-NF+3/D-NF+4's year reformulation against the frozen clamp sites; **and the pin-convention paragraph —
      a reviewer should be able to tell from the file alone why a session pin exists and what a tick-1 pin over
      this pack does and does not prove.**

**Gate:** six legs, single-flight. **Estimate:** ~5h · ~70k tokens.

---

### Task 7: `a10`/`a11` — the cumulative baseline and the dispossession cascade (R10) (PR D)

**Files:** Modify `content/rules/class-dynamics.bsl`; create
`content/scenarios/class-dynamics-cascade-arc-conformance.bscn`,
`content/scenarios/class_dynamics_cascade_arc_conformance.py`,
`rust/crates/babylon-tick/tests/class_dynamics_cascade.rs`.

**This task lands R10 — the ruling with the clearest behavioral consequence in the train.**

- [ ] **Step 1: Decide `a10`'s ORDERING BY TEST, not by argument.** The baseline must capture the **pre-transition**
      LA share. Write the failing test first (`a10_seeds_the_baseline_from_the_pre_transition_share`), **then**
      choose between (i) ordering `a10` before `a08`'s commit, or (ii) publishing a `la-share-prior` field in
      `a06`. **Record the choice and the rejected alternative in the pack header**, and update §7b's D116 row to
      match what actually landed. **Write the latch on `decomposition/p02-superwage-warning`'s idiom
      (`decomposition.bsl:248-260`: bind the flag, guard `(= flag 0)`, act, set it) and cite THAT — not
      `p7-persist-baselines`, which writes every tick and is a rolling persister (I3).** The pack header records
      the corrected precedent in one line so a reader does not inherit rev 1's mis-citation.
- [ ] **Step 2: More failing tests, on a session SIZED IN TICKS.** The arc is **≥156 ticks** — boundaries at 52,
      104 and 156 (§4.4; rev 1 never sized it, M12, and it is the longest run in the train).
      `a10_seeds_once_and_never_moves` (the baseline is identical at every boundary — R10's *"carried forward
      untouched"*);
      `a11_fires_at_the_highest_crossed_milestone_only` (a decline past 12pp emits **exactly one** event, at 10pp —
      R5's confirmed semantics); `a11_is_silent_when_the_la_grows`; `a11_is_silent_in_the_normal_phase`;
      `a11_is_silent_before_the_baseline_latch_is_set`; `a11_payload_carries_full_precision` (D-NF+16 — the payload
      value equals the mirror's **unrounded** value, and the 6-decimal rounded value is asserted **different**
      where they differ); and **the headline:**
      **`the_cascade_fires_under_a_cumulative_baseline_and_never_under_a_per_boundary_one`** — a ≥3-boundary arc
      asserting 5pp then 10pp, with the frozen per-boundary decline recorded in the test comment as **below 2.5pp
      at every boundary** (F19's analytic proof, made executable).
- [ ] **Step 3: Write `a10`/`a11`** per §7. The three milestones are **three ascending guards, last wins** —
      transcribing the frozen `for milestone in sorted(...)` loop's exact semantics, **never a `max`**. The
      `emit` payload uses Task 1 Step 5's rehearsed key idiom (no `EventType` vocabulary opt-in is needed, M11).
- [ ] **Step 4: Write the arc world + its mirror** — the mirror runs the frozen engine over the same boundaries
      with **BOTH** baseline readings and prints **both decline series** (units labelled per §8), so R10's premise
      is evidence in the repo rather than a citation. Extend the constant-parity harness to **five** worlds — 1, 2, 3 (Tasks 2/4), 4 (Task 5) and 5 (this task); worlds
6–8 join at Tasks 8 and 9 (rev 2 said six, N10). **The arc
      world takes THREE pins** — tick 52, tick 104 and tick 156 — because a single end-state hash cannot
      distinguish "fired at 5pp then 10pp" from "fired once at 10pp".
- [ ] **Step 5: Mutation** — swap the cumulative baseline for the previous boundary's share (**the headline test
      flips**); change highest-wins to first-wins (`a11_fires_at_the_highest_crossed_milestone_only` flips); change
      `decline > 0` to `>= 0` (a dedicated no-change fixture flips); mutate each of the three milestones in turn
      (each flips its own named test). Restore byte-identical.
- [ ] **Step 6: Commit** `feat(tick): class-dynamics a10/a11 — the cumulative baseline and the dispossession cascade (ADR210 R10)`.

**Gate:** six legs, single-flight; all **17** pre-existing pinned hashes byte-identical; this train's own six pins
re-measured with the `fired` arithmetic if the arc world changes them. **Estimate:** ~8h · ~110k tokens.

---

### Task 8: `a12`/`a13` — the ternary-derived bifurcation readout (R6) and R4's guard

**Files:** Modify `content/rules/class-dynamics.bsl`; create
`content/scenarios/class-dynamics-organizing-conformance.bscn` **and
`content/scenarios/class-dynamics-classless-county-conformance.bscn` (world 8, the C5 witness — rev 2 listed it in
§Worlds, the File Structure, D-NF+31 and the pin count but gave it no creating task; finding N2, closed here)**;
extend `class_dynamics_conformance.rs`.

- [ ] **Step 1: Failing tests. `a12` and `a13` both fire only at a boundary (rev 2.1's N1 decision), so every
      test below drives a session to tick 52 — a tick-1 assertion about either rule's OUTPUT would assert the
      seed, not the rule.** `a12_publishes_net_fascist_at_the_boundary_for_every_class` (including a rest-state
      class at `(0,1,0)` publishing **exactly 0.0**); **`a12_writes_nothing_off_the_boundary`** (the field is
      byte-identical across two consecutive non-boundary ticks **even while `consciousness/p6-route` moves the
      ternary underneath it** — the assertion that makes §7a's 13-copy inertness row true rather than nearly
      true, and the one a co-load world would have caught); `a13_is_the_population_weighted_mean` (two classes with different
      populations **and** different `(f−r)`, chosen so the **unweighted** mean differs — the intensive-aggregation
      guard, ADR070's read policy and F4's repair made provable); `a13_reaches_minus_one_when_all_revolutionary`
      and `a13_reaches_plus_one_when_all_fascist` (the range-by-construction claim);
      `a13_is_zero_at_the_hegemonic_rest_state`; `a13_ignores_the_orphan_class`;
      `a13_counts_the_shared_class_into_both_counties` (**D136's territory-side-fold record read correctly — a
      MEAN is not inflated by dual membership the way a SUM is, §2.4; the companion assertion is that neither
      county's mean equals the class-excluded value**); `a13_emits_the_threshold_event_in_both_directions` (the
      two-guard split, numeric direction key); **and the C5 pair:
      `a13_survives_a_territory_with_no_tenancy_class` (the county gets NO score — not a fabricated zero) and
      `a13_without_the_exists_protector_kills_the_tick` (the mutation form: remove the protector, and the named
      failure carries the `E-EVAL-021` text, at TICK 1, proving the `when` gate never protected it)**.
- [ ] **Step 1b (NEW, N2): Create world 8, the classless-county witness.**
      `class-dynamics-classless-county-conformance.bscn` — the smallest world in the train: one TERRITORY with
      every declared field seeded and **no incoming-TENANCY class at all**, beside one ordinary county so the
      contrast is visible in one world. Its header states that it exists **solely** so C5's abort is a fixture
      rather than a production discovery, and that it is the only world whose purpose is a REFUSAL. Rev 2 named
      this world in §Worlds, the File Structure, D-NF+31 and the pin count and gave it no creating task; this step
      closes that.
- [ ] **Step 2: R4's guard — the train's clearest Director-facing artifact after R10.** Create the organizing
      world: a crisis county with **no SOLIDARITY edge at all** beside one that has one, and the test
      **`the_unorganized_county_drifts_fascist_and_the_organized_one_does_not`** pinning `score > 0` strictly for
      the first and `< 0` for the second. Its header quotes R4 **verbatim** and cites ADR016's prior art, and
      states plainly that **this pack does not implement the asymmetry — `consciousness/p6-route` does — and this
      test proves the readout carries it** (D-NF+20).
- [ ] **Step 3: Write `a12`/`a13`** per §7 — **`a12` with the pack's boundary gate `(when (= phase-of-year 0))`,
      NOT `(when #t)`** (rev 2.1's N1 decision; its row in §7 carries the rationale, and the pack header records
      it in one line so a future reader sees a decision rather than an inconsistency),
      `a13` with the **`exists` protector on the fold binding AND the guard on the write** (C5), and the
      `(field-of it social-class/population)` `:weight` Task 1 Step 1 confirmed. **There is no fallback branch —
      BLOCKER-6 is retired (I5); a refusal here is a source contradiction and a STOP, not a fallback.**
- [ ] **Step 4: Mutation** — drop the `:weight` (**the load must REFUSE** with `UnweightedMeanOfIntensive`, and the
      test must show the refusal as a **named failure**, never a silent pass); swap `(f − r)` for `(r − f)` (the
      two range tests flip); change the threshold constant (the both-directions test flips); **replace `a12`'s
      boundary gate with `(when #t)` — `a12_writes_nothing_off_the_boundary` AND
      `the_pack_is_inert_off_the_boundary` must BOTH flip, and the tick-1 load pin's `fired == 0` must fail: the
      executable form of the N1 decision, so a future reader who re-tries the unconditional shape is stopped by a
      test rather than by a comment**; remove `a12`'s write entirely (the rest-state test flips). Restore
      byte-identical.
- [ ] **Step 5: Add the next pins** — phase-matrix, cascade-arc (×3: ticks 52/104/156) and organizing, each with
      **both** a tick-1 load pin and its boundary pin(s), measured; re-measure any earlier pin whose world changed,
      with the `fired` arithmetic recorded in the commit body. **Running total after this task: 14 of the 18.**
- [ ] **Step 6: Commit** `feat(tick): class-dynamics a12/a13 — the ternary-derived bifurcation readout (ADR210 R6) and the R4 organizing guard`.
- [ ] **Step 7: Open PR D** `feature/class-dynamics-cascade-and-readout`, off **merged dev** (Tasks 7–8, 2
      commits). **Review lens:** (a) R10's headline test read as a Director-facing artifact; (b) the weighted-fold
      reformulation's equivalence argument and the **D136** inheritance read correctly; (c) **R4's guard test read
      as the theory line made executable** — quote its name in the PR body; (d) the empty-fold protector and its
      mutation vector, which is the one place in this pack where a missing guard kills every tick rather than
      moving a number.

**Gate:** six legs, single-flight; all **17** pre-existing pinned hashes byte-identical. **Estimate:** ~7h · ~100k tokens.

---

### Task 9: The Φ → savings → LA-mobility coupling in `fundamental-theorem.bsl` (R9) (PR E)

**Files:** Modify `content/rules/fundamental-theorem.bsl`; **modify
`content/scenarios/two-classes.bscn` (DECLARATION-ONLY)**; **modify `rust/crates/babylon-tick/src/lib.rs`'s test
module (one test)**; create `content/scenarios/class-dynamics-phi-conformance.bscn`,
`content/scenarios/class_dynamics_phi_conformance.py`; extend the test.

**Precondition: DG-9's answer.** This plan lands `phi_cap` as a **BSL `defconst`** (§Global, D-NF+19). If DG-9
returns "a real `GameDefines` define", **STOP** and re-plan: that adds a Python-lane commit, an
`11-baseline §6.5 ceremony` (`Baselines: blessed(<slug>)` trailer, generated via
`tools/generate_ceremony_message.py`), and a gate analysis this task does not budget. **Do not improvise the
middle.**

**THIS TASK EDITS A SHARED FILE. §2.2 is its specification, and Steps 0–2 are not optional preliminaries — they
are the reason the task is executable at all (C1).**

- [ ] **Step 0: Re-run the consumer enumeration** (Task 0 Step 4b) against THIS branch's HEAD and paste it into
      the commit body: every `include_str!` consumer of `fundamental-theorem.bsl` and of `two-classes.bscn`, with
      the hash each asserts. **Expected: 4 consumers, 2 pinned hashes (`tick_goldens.rs`'s pre+post,
      `babylon-client`'s post), 1 structural assertion (`per_rule_fired.len() == 1`).** A consumer this plan does
      not list is a STOP.
- [ ] **Step 1: The DECLARATION-ONLY extension of `two-classes.bscn`, with its no-op proof written FIRST.**
      Add the `deffield` rows (`territory/median-wage`, `territory/phi-hour`,
      `territory/phi-savings-adjustment`) and the `defconst` rows (`class-dynamics/phi-cap`,
      `/hours-per-year`, `/v-reproduction`, `/accumulation-halt-floor-ratio`) — **no node form, no attribute, no
      edge, and nothing removed**. Each row carries a one-line comment naming this train and stating that this
      world declares without seeding because it holds no territory. **Then run
      `cargo test -p babylon-tick --test tick_goldens` and `cargo test -p babylon-client --test engine_link`
      BEFORE writing the rule** — both must be green with the scenario edit alone, which is the executable form
      of §2.2.3's hash-neutrality argument (`state_hash.rs:10-30`; `scenario.rs:1236-1275`). **If either moves:
      STOP — the argument is wrong and the task re-plans, it does not re-pin.**
- [ ] **Step 2: Repair `babylon-tick/src/lib.rs`'s one affected unit test.** After the rule lands,
      `single_rule_content_still_reports_fired_and_a_one_entry_per_rule_fired` sees **two** rules. Rename it
      (the "single_rule_content" premise is now false), assert `per_rule_fired.len() == 2`, keep the property it
      exists to pin (**the per-rule breakdown sums to `report.fired`**), and **add the assertion that the Φ rule's
      entry is `0`** — a world with no TERRITORY nodes gives the rule no subjects, which is the whole reason the
      pins hold. Its comment cites §2.2.3. **This is the train's ONLY Rust-source test edit; it is not a pin and
      not a baseline.**
- [ ] **Step 3: Failing tests** — `phi_coupling_raises_the_savings_rate` (bit-exact against the mirror);
      `phi_coupling_binds_at_the_cap` (a fixture where `phi_hour · 2080 ÷ effective_wage` exceeds `phi_cap` —
      **against the ANNUAL, halt-zeroed wage, C3**; with the corrected divisor this needs a genuinely high
      `phi-hour`, e.g. `phi-hour ≥ 0.05 × median-wage`, and the world seeds one);
      `phi_coupling_is_zero_when_either_operand_is_zero` (**both** frozen guards, `savings_schedule.py:90-91`);
      **`phi_coupling_is_zero_for_a_halted_county`** (a `$9.59/hr` fixture — the frozen guard tests the HALTED
      wage, and rev 1's rule would have silently diverged here);
      **`a01_and_phi_coupling_agree_on_the_wage_base`** (§7a's copies-agree row, now writable);
      and the pedagogy assertion **`phi_coupling_raises_the_la_share`** — identical worlds differing **only** in
      `phi-hour` end the boundary with different LA shares, the higher-Φ county higher, **and neither at the cap**
      (the gradient, not the switch: a test that passes with both counties saturated would prove nothing, which
      is exactly what rev 1's rule would have produced). **That test is R9's material claim made executable and
      belongs in the PR body.**
- [ ] **Step 4: Write the rule** per §7 — **divisor = the annual, FR-017-halted wage**, `2080` written on both
      sides rather than algebraically cancelled — with a `:material-basis` stating the theorem in the Director's
      own framing (*imperial rent purchases entry into the labor aristocracy*), and **`fundamental-theorem.bsl`'s
      first `D-N` header block** carrying D-NF+18's one-tick lag, D-NF+29's shared-file blast radius, and their
      named re-open triggers.
- [ ] **Step 5: Declare `phi-cap` as a `defconst`** in **every world that loads the rule — which now includes
      `two-classes.bscn`** — with its `savings_schedule.py:30` provenance and **D-NF+19's reconciliation of R9 and
      R8 written out inline**, so a future reader sees a decision, not a drift.
- [ ] **Step 6: Assert the lag explicitly** — a test proving the boundary reads the **previous** tick's published
      adjustment, plus a comment recording that the lag is **unobservable in these worlds because `phi-hour` is
      static content**, and that **it becomes observable when the ImperialRent train makes `phi-hour` move within a
      year**.
- [ ] **Step 7: Mutation** — remove the `phi-cap` clamp (`phi_coupling_binds_at_the_cap` flips); drop the
      zero-operand guard (a division-by-zero fixture must abort the tick **loudly** and the test must name the
      error code); change `2080` on ONE side only (**`a01_and_phi_coupling_agree_on_the_wage_base` flips — the
      C3 killer**); drop the halt from the coupling's wage base (`phi_coupling_is_zero_for_a_halted_county`
      flips); **delete one `deffield` row from `two-classes.bscn` (the load must refuse with `E-LOAD-010` naming
      the qname) and one `defconst` row (the refusal must name `check_sources_servable`'s text)** — the two
      refusals that make §2.2's argument executable rather than asserted. Restore byte-identical.
- [ ] **Step 8: Commit** `feat(tick): the Phi -> savings -> LA-mobility coupling in fundamental-theorem.bsl (ADR210 R9)` — the commit body carries the consumer enumeration, the two-classes declaration-only diff, and the lib.rs test repair with its rationale.

**Gate:** six legs, single-flight; **all 17 pre-existing pinned hashes byte-identical, and the gate explicitly
runs `cargo test -p babylon-client` — the crate rev 1 never mentioned in 1,700 lines**; `class-dynamics.bsl`'s own
pins re-measured where the coupling changed a world. **Estimate:** ~7h · ~95k tokens.

---

### Task 10: The fuel sweep, the TWO co-load worlds, and pin completion

**Files:** Modify `content/rules/class-dynamics.bsl` and `content/rules/fundamental-theorem.bsl` (`:fuel` figures
only); create `content/scenarios/class-dynamics-coload-a-conformance.bscn` and
`content/scenarios/class-dynamics-coload-b-conformance.bscn`; modify `tick_goldens.rs`.

- [ ] **Step 1: The declare-bound+1 fuel sweep, all 14 rules.** For each rule: declare a deliberately low
      `:fuel N`, load, **paste the exact `E-LOAD-040` refusal text** into the pack header's FUEL block, set
      `:fuel B+1`, confirm it clears load **and** runtime against **every** world that loads the rule — **which
      for `economics/phi-savings-coupling` now includes `two-classes.bscn`** (§2.2.3). **The declared figure is the
      MAX over worlds** — re-measure after the co-load worlds land, since their larger seeded populations raise
      the folds' static bounds. Record the per-rule table (rule id · low declaration · measured bound · shipped
      fuel · binding world).
- [ ] **Step 2: The TWO co-load worlds (§2.3 obligation 2, re-scoped by C4).** A single world loading both
      `territory.bsl` and `decomposition.bsl` **cannot exist at this HEAD** — both declare `(intrinsic floor …)`
      and the loader refuses duplicates by name (#646). So:
      **World A** = `class-dynamics.bsl` + `consciousness.bsl` + `production.bsl` + `territory.bsl`;
      **World B** = `class-dynamics.bsl` + `consciousness.bsl` + `decomposition.bsl`.
      Each proves: (a) no rule-id collision; (b) no III.11 hard error from any subject-type rule meeting a node
      that lacks a bound field; (c) `a12`'s `social-class/ternary-net-fascist` write does not disturb
      `consciousness/p6-route`'s ternary; (d) the pack produces the SAME `territory/*` values under co-load as
      alone (a same-values assertion, not just "it loads"); (e) **every TERRITORY survives `a13`'s fold**,
      including the foreign-shaped ones that carry no TENANCY-incident class (C5 — world A is the exposed case).
      World B additionally proves this pack's `social-class/population` READ coexists with `decomposition/p04`–
      `p06`'s writes to it.
      **Write into both headers what the split costs:** neither world proves a three-way co-load, no world can
      until #646 lands, and **this pack causes none of it** — it declares no intrinsic (§7c guard 2 asserts that
      at source level). **A first attempt at the single combined world is worth making and recording: paste the
      `E-LOAD-001` refusal text into the header as the evidence #646 is real, then split.** D-NF+22.
- [ ] **Step 3: Complete the golden pins — 18 total.** Per content world: one **load pin** (tick 1, `run_once`:
      `before == after`, this pack `fired == 0` — all thirteen rules gated, `a12` included, N1) and one **boundary pin** (tick 52, `run_to_tick`), plus the arc
      world's two extra boundary pins (ticks 104, 156). Each MEASURED, each with the per-rule-id `fired`
      arithmetic in its assertion message. Re-measure every earlier pin whose world changed and record the delta's
      rule-id attribution in the commit body. **Whether the two co-load worlds take their own pins is MEASURED,
      then STATED** — if they do, the additive count is higher and the header's figure is updated in the same
      commit, never left stale. **All 17 pre-existing pinned hashes byte-identical.**
- [ ] **Step 4: Commit** `test(tick): class-dynamics fuel sweep, the two co-load worlds and the eighteen golden pins`.

**Gate:** six legs, single-flight. **Estimate:** ~5h · ~70k tokens.

---

### Task 11: Records — register rows, the ADR, the inventory verdict, and issue hygiene

**Files:** Modify `docs/reference/bsl-language.rst`, both packs' `D-N` blocks,
`reports/port-inventories/tick-dynamics-port-phase1-inventory-2026-08-12.md`; create
`ai/decisions/ADR-NF_class_dynamics_port_handoff.yaml` + the `index.yaml` row.

- [ ] **Step 1: Register rows `D-NF+1 … D-NF+32`** using **Task 0 Step 3's allocation, re-confirmed at Task 12**,
      one per D-record-table row, each with `file:line` evidence and each mirrored as a pack-local `D-N` citing the
      global number (the two-homes convention). **The seven rows a reviewer will read hardest:** the scope boundary
      (D-NF+1), the R8 interpretation (D-NF+9), what R6 retires and what it deliberately does **not** delete from
      Python (D-NF+17), the `phi_cap` reconciliation (D-NF+19), R4's asymmetry-as-feature (D-NF+20), **the shared-
      file blast radius of R9's ruled home (D-NF+29), and the two-tick pin convention (D-NF+30)**.
- [ ] **Step 2: `ADR-NF`** — records: the pack (13 + 1 rules, 46 defconsts, 8 content worlds + 2 co-load worlds,
      4 mirrors, 18 additive pins); **§0.1's boundary and the explicit statement that Checkpoint A is NOT closed by
      this train, with §0.2's measured roster**; **R4–R11 executed one ruling at a time, each with the D-row that
      carries it**; the spike verdicts and the three source-answered confirmations; the **eleven** corrections to
      prior documents (including `validation.py`'s **32, not 29**, D45→D136, and the `p7`-is-rolling correction);
      the cross-train disjointness proof and its two mechanical obligations; **the `two-classes.bscn`
      declaration-only extension with its hash-neutrality argument, and the one `lib.rs` test repair**; **zero
      Python-lane changes**; and the five residual @4.0 trains. Add the `index.yaml` row.
- [ ] **Step 3: Inventory UPDATE block** — verdict **PARTIALLY SUPERSEDED**: the Feature-016 estate is PORTED; the
      bifurcation row is retired by R6; the `graph_bridge` stamping blocker **DISSOLVES** (D-NF+2); the `round()`
      census's in-scope count is **zero**; the "unpinnable pending WS1" row is **stale**; the remaining rows keep
      their grades and are re-homed to the five residual trains.
- [ ] **Step 4: Issue hygiene — the anti-silent-shrink step.** Comment on **#563** with the train's evidence and
      **close it** (its charter — the dormancy re-read + the ServicesProtocol boundary design — is discharged by
      §3 plus the frozen-estate dossier). File **five fresh issues** under umbrella **#557**, one per residual @4.0
      train (§0.1), each naming its **gating blocker verbatim** — and the Vol I issue **must** carry §6's warning
      that `reserve_army/calculator.py:44-46`'s logistic is an imposed form to be **re-derived as a measure, never
      transcribed**. File **one** for the crisis-phase producer (BLOCKER-4) and **one** for the
      `w_s`/`w_b`/`class_burden_epsilon` Python deletion on the WS4/python-deletion ledger (D-NF+17). **Comment on
      #646** recording that this train hit its co-load landmine, what it cost (two worlds instead of one, §2.3),
      and what a fix would collapse. Post
      **§0.2's measured roster** to **#557** and update **#578**'s Material-Base row: **WS3 stays HELD**, pending
      DG-10. Post a comment on **#564** row 21 recording that ADR210 R4–R11 are now executed in content, with the
      test names that carry R4 and R10.
- [ ] **Step 5: Commit** `docs(p27): class-dynamics port handoff — 32 register rows, ADR-NF, inventory verdict, issue hygiene`.

**Gate:** `vale` over every touched Markdown/RST; `PYTHONPATH="$PWD/src" UV_FROZEN=1 uv run pytest
tests/unit/reference/test_bsl_grammar_sync.py -q`. **Estimate:** ~4h · ~60k tokens.

---

### Task 12: Final gates, the tail re-measure, and the closing entry

**Files:** Modify `docs/reference/bsl-language.rst` and `ai/decisions/*` (numbering only, if the tail moved);
`ai/state.yaml`.

- [ ] **Step 1: RE-MEASURE both numbering tails immediately before filing** and re-allocate if any of the three
      contending trains landed since Task 0. **Rename `ADR-NF_…yaml` and renumber every `D-NF+n` in one commit**;
      re-run the grammar-sync probe afterwards.
- [ ] **Step 2: RE-RUN the cross-train collision grep** (Task 0 Step 4) against **merged `dev`**. A hit is a STOP;
      the newest claimant renames.
- [ ] **Step 3: Full gates, once, single-flight, in this order** — `mise run rust:check`; `mise run check`;
      `mise run qa:regression`; `mise run qa:vault-regression-ci`; `tests/unit/reference/test_bsl_grammar_sync.py`;
      `vale`. **`qa:regression` and `qa:vault-regression-ci` must be byte-identical TRIVIALLY** (zero Python-lane
      changes). **Nothing under `tests/baselines/**` may move. Any drift is a red gate — STOP.**
- [ ] **Step 4: Confirm all 17 pre-existing pinned hashes byte-identical** against Task 0 Step 6's pasted baseline
      — hash by hash, **including `babylon-client`'s `783f651d…7679`, which needs its own
      `cargo test -p babylon-client --test engine_link` leg** — and paste the comparison into the PR body.
- [ ] **Step 5: `ai/state.yaml`** — prepend one entry to `current_focus.recently_completed` in the landed style
      (bold label, dense run-on body, closing `Gates:` clause naming every green gate) and bump `updated:`.
      **The entry must state that Checkpoint A is NOT closed and WS3 stays HELD**, pending DG-10 and ReserveArmy.
- [ ] **Step 6: Commit** `docs(state): class-dynamics port closing entry (ADR-NF, D-NF+1..32, Checkpoint-A accounting)`.
- [ ] **Step 7: Open PR E** `feature/class-dynamics-phi-and-records`, off **merged dev** (Tasks 9–12, 4 commits).
      **Review lens:** (a) the D-row ledger read as the train's honesty contract — every divergence traceable to a
      ruling or a language fact; (b) **the Checkpoint-A accounting**, which the PR body must state plainly; (c) the
      three Director-gate answers (DG-7/8/9) and what each decided; (d) **§2.2's shared-file argument read as the
      train's one irreversible interaction with landed content — the declaration-only diff, the two green pin
      legs, and the renamed `lib.rs` test.**

**Gate:** everything in Step 3, green. **Estimate:** ~3h · ~50k tokens.

---

## PR structure — FIVE PRs, and why the split falls where it does

| | branch | tasks | commits | contents | review lens |
|---|---|---|---|---|---|
| **PR A** | `feature/tickdynamics-port-bsl` (worktree exists) | 0–2 | 3 | the surface-facts dossier (11 corrections, the consumer enumeration); the spikes and the three source-answered confirmations; the registration string; world 1 + the primary mirror; the declaration surface (17 territory fields + 7 staging + 1 class field, 46 defconsts); the three §7c anti-pattern guards | **scope + capability** — is §0's boundary the right one, and is every unprecedented construct proved rather than assumed? Also: **is every "spike" that became a source-answered fact actually cited, not just asserted?** |
| **PR B** | `feature/class-dynamics-rates`, off **merged dev** | 3–4 | 2 | `a01`–`a05`; the F11 repair; the 5×4 amplifier grid; two crisis worlds | **transcription fidelity** — §1.2 line by line, and the mutation ledger's completeness (20/20 multipliers, plus the F11 factor re-derived from the mirror's double run) |
| **PR C** | `feature/class-dynamics-flows`, off **merged dev** | 5–6 | 2 | `a06`–`a09`; the degenerate repair; the year axis; the boundary-inertness proof; **the session-driven pin convention + the first six pins**; DG-7's answer applied to Task 5 | **the arithmetic's own laws** — mass conservation as the reason no runtime assertion exists; the year reformulation against the frozen clamp sites; **and the pin convention: what a tick-1 pin proves and why a tick-52 pin had to exist** |
| **PR D** | `feature/class-dynamics-cascade-and-readout`, off **merged dev** | 7–8 | 2 | `a10`–`a13`; R10's cumulative baseline + the ≥156-tick arc; R6's weighted readout **with its empty-fold protector**; **R4's organizing guard**; the arc + organizing worlds | **theory made executable** — R10's headline test and R4's guard test are the two Director-facing artifacts; the **D136** inheritance and the weighted-fold argument; the C5 protector's mutation vector |
| **PR E** | `feature/class-dynamics-phi-and-records`, off **merged dev** | 9–12 | 4 | R9's coupling in `fundamental-theorem.bsl` **+ the `two-classes.bscn` declaration-only extension + the one `lib.rs` test repair**; the fuel sweep; **both** co-load worlds; all 18 pins; 32 register rows; `ADR-NF`; inventory verdict; issue hygiene; state entry | **governance closure + blast radius** — the D-row ledger as an honesty contract; the Checkpoint-A accounting; the three DG answers; **and the shared-file edit: does the declaration-only diff hold, and are all 17 pre-existing pinned hashes shown unmoved, `babylon-client` included?** |

**Why five, not two.** The split follows **failure mode**, not size. PR A's failures are **loud and immediate**
(a refused construct, an unregistered namespace) and its content is a *capability claim* — bundling it with rule
arithmetic would put a spike verdict in a diff whose reviewer is checking a transcription. PRs B and C are
**arithmetic**, where the failure mode is a silently-wrong number that only a mutation vector catches. PR D is
**theory**, where the failure mode is a correct number that teaches the wrong thing — it is reviewed by a different
question ("does this carry the ruling?") and it is the PR the Director reads. PR E is **governance**, whose failure
mode is a divergence that never got recorded.

**The dependency direction is clean.** A: register + declare + prove the constructs. B: rules that read A's fields.
C: rules that read B's rates. D: rules that read C's committed shares. E: a rule in a *different* pack that
publishes an input A already declared, plus the records. **No earlier PR's rule reads anything a later PR writes**
— with one deliberate, recorded exception: `territory/phi-savings-adjustment` is declared in PR A and read by `a01`
in PR B, and is not **written** until PR E. Until then it is a seeded per-node value, which is exactly the
declared-input discipline BLOCKER-4 already uses, and `phi_coupling_raises_the_la_share` in PR E is what proves the
seam closed.

**What is NOT guaranteed:** PR C's six pins do **not** survive PR D — they will move, by construction, and
Task 8 Step 5 / Task 10 Step 3 re-measure them with the `fired` arithmetic. **Never stacked** (#193): each PR
branches off merged `dev`.

**One PR-boundary consequence rev 2 adds.** PR E is now the only PR that touches **landed content outside this
train's own pack** (`two-classes.bscn`, `fundamental-theorem.bsl`, `lib.rs`'s test module) — and PR A no longer
carries "the only PR touching Rust source", since PR E does too. **That concentration is deliberate**: the
shared-file edit and its no-op proof read as one reviewable unit, and a reviewer who rejects §2.2's argument
rejects one PR rather than unpicking five.

---

## Worlds / conformance matrix — eight content worlds + two co-load worlds

**Every world re-declares `CrisisPhase` (`defenum` is not shared), seeds every declared field on every node of its
namespace (the no-defaults law), and re-declares the 46-constant canonical block — with every deliberate variation
named in its own header.**

**THREE WORLD-DESIGN LAWS rev 2 adds, each of which a rev-1 world would have violated:**
1. **Every TERRITORY in every world that loads this pack must either carry an incoming-TENANCY SOCIAL_CLASS
   neighbour or be a deliberate empty-fold fixture** — because `a13`'s binding evaluates before the guard and
   `mean` over an empty set is `E-EVAL-021`, killing the tick (C5). Each world's header states which of the two
   each of its territories is. Rules are **not** selectable per world: the "rules exercised" column below says
   which rules do interesting work, **not** which rules run.
2. **Every world takes TWO pins** — a **load pin** at tick 1 (`before == after`, this pack `fired == 0`: the
   executable form of off-boundary inertness, **and true only because all thirteen rules carry the gate — see
   rev 2.1's N1 decision in §7's `a12` row**) and a **boundary pin** at tick 52 (the arithmetic). Tick 0 never
   executes and the gate first opens at 52 (§4.4, C2).
3. **Sessions are sized in ticks in the world header** — 52 for a single-boundary world, 105 for a two-boundary
   world, **156 for the arc**.

| world | file | proves | rules doing work | mirror | pins |
|---|---|---|---|---|---|
| **1 — primary** | `class-dynamics-conformance.bscn` | the four rate constructors at NORMAL; the flow equations; the rescale-is-identity case; the year increment; the baseline seed; **the two-TENANCY `shared-class` counted into both counties' means (D136 read correctly, §2.4)**; the orphan class contributing to nothing; two counties not globally flattened | `a01`–`a13` | `class_dynamics_conformance.py` (**+ the F11 double run**) | load (t1) + boundary (t52) |
| **2 — deep crisis** | `class-dynamics-deep-crisis-conformance.bscn` | the DEEP row `3.0/3.5/0.1/0.2` (DG-4's flagged pedagogy); a base rate × multiplier that **exceeds 1** so the clamp binds | `a01`–`a11` | `class_dynamics_deep_crisis_conformance.py` (**+ the F11 double run — the SECOND of the two double-run mirrors §Global promises, named here because rev 1 named only one, M10**) | load + boundary |
| **3 — phase matrix** | `class-dynamics-phase-matrix-conformance.bscn` | ONSET / EARLY / RECOVERY in one world (three counties, three phases) — the remaining **12 multipliers individually**; each county's header names the four constants it makes provable | `a01`–`a11` | world 2's mirror, second `WORLD` | load + boundary |
| **4 — degenerate** | `class-dynamics-degenerate-conformance.bscn` | rates driven so `max(·,0)` zeroes all three dynamic shares — **the ONLY fixture that reaches `_normalize`'s degenerate branch**; without it D-NF+8's repair is unprovable. **Depends on `raw-share-*` being `real`: declared `probability`, `a06` aborts at `E-EVAL-020` and this world never reaches the branch it exists for (§4.2.1, I6)** | `a01`–`a09` | world 1's mirror, second `WORLD` | load + boundary |
| **5 — cascade arc** | `class-dynamics-cascade-arc-conformance.bscn` | **a ≥156-tick session, boundaries at 52 / 104 / 156**: the cascade fires at 5pp then 10pp under the cumulative baseline and **NEVER** under the per-boundary one (F19's ≤2.5pp ceiling recorded per boundary); highest-milestone-only; the payload's full precision | `a01`–`a11` | `class_dynamics_cascade_arc_conformance.py` (**both baseline readings, both decline series, units labelled**) | load (t1) + **three** boundary pins (t52, t104, t156) — a single end-state hash cannot tell "5pp then 10pp" from "10pp once" |
| **6 — organizing (R4)** | `class-dynamics-organizing-conformance.bscn` | **`the_unorganized_county_drifts_fascist_and_the_organized_one_does_not`** — a crisis county with **no SOLIDARITY edge** beside one with; the population-weighted mean where the **unweighted** mean differs; both event directions; the rest-state zero | `a01`–`a13` | world 1's mirror, third `WORLD` | load + boundary |
| **7 — Φ** | `class-dynamics-phi-conformance.bscn` | `phi-hour` large enough to bind `phi-cap` **against the ANNUAL wage** (C3 — under rev 1's divisor every fixture bound and the test proved nothing); both zero-operand guards; **the FR-017 halt at exactly `12 × 0.8 = 9.60` with the strict-`<` pair `9.60`/`9.59`, and the halted county's Φ adjustment forced to 0**; `a01_and_phi_coupling_agree_on_the_wage_base`; `phi_coupling_raises_the_la_share` **with neither county at the cap** | `a01`–`a13` + `economics/phi-savings-coupling` | `class_dynamics_phi_conformance.py` | load + boundary |
| **8 — empty-fold witness** | `class-dynamics-classless-county-conformance.bscn` | **a TERRITORY with NO incoming-TENANCY class**: the tick survives, `a13` writes no score, and the mutation that removes the `exists` protector fails with `E-EVAL-021` **at tick 1** (C5). Smallest world in the train, and the only one whose purpose is a refusal | `a01`–`a13` | none needed — the assertion is structural, not numeric | load + boundary |
| **co-load A** | `class-dynamics-coload-a-conformance.bscn` | co-load with `consciousness.bsl` + `production.bsl` + `territory.bsl`: no rule-id collision, no III.11 hard error, `p6-route`'s ternary undisturbed, same `territory/*` values as alone, **and foreign-shaped territories surviving `a13`** | all | — | **measured, then stated** (Task 10 Step 3) |
| **co-load B** | `class-dynamics-coload-b-conformance.bscn` | the same list against `decomposition.bsl` (which cannot co-load with `territory.bsl`, #646), plus `social-class/population` READ coexisting with `p04`–`p06`'s writes | all | — | **measured, then stated** |

**Also modified, not created:** `two-classes.bscn` — **declaration-only**, so `economics/phi-savings-coupling`
resolves in the world R9's ruled home already serves (§2.2.3). It is the ninth world this train touches and the
only pre-existing one.

**Ceremony accounting.** All eight content worlds and both co-load worlds are **new content**, and the one
pre-existing world this train touches gains **declarations only, which the canonical state-hash layout does not
cover** (§2.2.3): **zero existing pins move, so no §6.5 baseline ceremony is owed by any task in this train.** F11's repair moves nothing in `tests/baselines/**` because the repair
lands only in Rust content and the Python lane is untouched — its cost is discharged in **this train's own new
vectors**, measured fresh (D-NF+7). R10's "the event begins firing where it never fired" lands the same way. **If
any `tests/baselines/**` file moves, that is a STOP, not a ceremony.**

---

## D-record table — 32 rows, allocated **NEXT-FREE-AT-LANDING** (Task 0 Step 3, re-confirmed Task 12 Step 1)

**Never written as literals.** Measured tail 2026-08-18: **D180** (`bsl-language.rst:8158`) — **four-way
contended** (§Global). Each row lands in two homes: the global register, and the pack header as a pack-local `D-N`
citing the global number.

| row | subject | one-line rationale |
|---|---|---|
| **D-NF+1** | **The pack boundary — Feature 016, not all of @4.0** | §0.1's no-sliver argument applied to a **complete frozen subsystem** with its own package, protocol, spec and test suite; the five residual @4.0 trains are named and filed, and **Checkpoint A is explicitly not claimed** (ADR208 R15's own criterion, §0.2's measured roster). Also carries the stale "Step 5b executes after Step 6" doc row: the frozen comment numbering is not the call order (`:270` then `:279`), and this pack's rule-id byte order is the contract |
| **D-NF+2** | **The `graph_bridge` stamping layer DISSOLVES, it is not omitted** | Declared node fields **are** the storage in BSL; there is no second Pydantic copy to synchronize, so the inventory's "dominant blocker #2" is a non-computation in the target estate. The single largest cost reduction in the @4.0 program — recorded, never assumed |
| **D-NF+3** | **`:year` / `:tick-of-year` are UNSERVED; the year is a per-territory `int` field** | `tick.rs:456,462`'s refusals quoted verbatim ("slice 1 pins no epoch"); the frozen `base_year + tick // 52` derivation is not expressible **and is not needed** — `with_updated_dynamics` already just increments. The boundary gate is `:tick-in-cycle 52`, not modulo (`ARITH` is `+ - * /` only, `grammar.rs:724`) |
| **D-NF+4** | **Six year-clamp sites collapse to one; `[2007, 2030]` become two `defconst`s** | The pair is a hardcoded Pydantic `Field` constraint (`types.py:57`), never a define; `a09` carries the whole clamp once |
| **D-NF+5** | **The scaled-int `x1e6` lane for the seven multipliers above 1.0** | `E-LEX-021` refuses bare non-integer literals and `p`/`i`/`c` are `[0,1]`-bounded at lex (`reader.rs:877-883`); the landed `metabolism-conformance.bscn:22-24` / `territory-conformance.bscn:101` escape hatch is reused, **with Task 1 Step 3's bit-exactness proof** and #591 item 5 as the named retirement trigger |
| **D-NF+6** | **The `hasattr` amplifier selection is not expressible and `DefaultCrisisAmplifier` is not ported** | `transition_engine.py:162`'s runtime duck-type check has no BSL analogue; `PhasedCrisisAmplifier` is the ported amplifier and the legacy `2.5`/`0.3` path is dead content — **recorded, declared nowhere**, WS4 row |
| **D-NF+7** | **F11 REPAIRED — `wage · s²` → `wage · s`** (R9 + R11) | `accumulation.py:90` applies the savings rate twice against the docstring's own admission (`:40-41`); the P→LA channel moves by `1/s` (**33× at 0.03**). ADR183 R1: the frozen values are **not** the oracle. The `0.08` clamp begins to bind. The dossier's "owes a §6.5 ceremony in the Rust lane's own vectors" is discharged by measuring this train's **new** vectors; `tests/baselines/**` stays byte-identical |
| **D-NF+8** | **`_normalize`'s `target/3` degenerate branch REPAIRED — preserve, never fabricate** | `transition_engine.py:326-329` assigns arbitrary equal thirds; the ported zero arm has an **empty effect list** (III.11: no number is invented). World 4 is the only fixture that reaches the branch, and `a07_does_not_write_equal_thirds` is the anti-assertion that stops a silent restoration |
| **D-NF+9** | **The 32 `validation.py` thresholds land as CONFORMANCE BOUNDS, not `defconst`s** | They drive only `logger.warning`/`logger.error`; III.11 / S-11 has **no warning level** to port them into, and declaring 32 constants no rule reads would break declare-only-what-you-read and R8's own "hash-covered" framing. **An interpretation of a Director ruling, flagged as such** — §10 DG-8 |
| **D-NF+10** | **F15's two absence encodings cannot collapse in the port, and `economics_fallbacks` DISSOLVES** | Declared content cannot be "unwired" and a missing field is a load error, so the `NoDataSentinel` abort has **no ported trigger** — unported rather than translated. The frozen system's 9 fallback-tally sites have no III.11-legal analogue |
| **D-NF+11** | **`p_to_l_component` RETIRES; its three weights are not declared** | Computed, returned, read nowhere (`dispossession.py:107-111,120`) — the register-row-24 category. **The general "may the workforce retire a never-fired output?" question is STILL OPEN** (§10 DG-7); if DG-7 returns "no", the trio is declared as inert content and this row is rewritten |
| **D-NF+12** | **Four of five savings rates are dead and are declared nowhere** | The only call site hardcodes `ClassPosition.PROLETARIAT` (`transition_engine.py:136`), so the Fed-SCF ladder's other four rates have no reader. **WS4 ledger question, not Director-reserved**: *"are they reserved for a consumer, or is the schedule four-fifths dead?"* |
| **D-NF+13** | **F13/F14 disambiguated** | `base_stabilization` is `0.15` (`:53`) and the docstring's `0.10` is wrong **twice** (`:74`, `:98`) — the constant transcribes, the pair does not. `_MAX_ACCUMULATION_RATE` **equals** `ACCUMULATION_WARNING_MAX` exactly (`validation.py:39`); preserved deliberately, and killable for the first time after D-NF+7 |
| **D-NF+14** | **`_DEFAULT_EVICTION_WEIGHT` multiplies UNEMPLOYMENT — a misnomer** | `transition_engine.py:52, 233-235`; the arithmetic transcribes exactly, **the name does not**, and a swap-the-weights fixture proves the distinction is observable |
| **D-NF+15** | **R10's CUMULATIVE baseline — the divergence from the frozen previous-boundary read** | `system/__init__.py:1140` reads the previous boundary; **F19 proves the per-boundary ceiling is ~2.5pp, half the smallest milestone**, so the event has never fired for an arithmetic reason. The cumulative read is the **only** reading under which R5's confirmed constants and highest-only rule are simultaneously correct. One new persisted field on `consciousness/p7-persist-baselines`' pattern; **the event begins firing where it never fired** |
| **D-NF+16** | **`round(x, 6)` presentation rounding is dropped** | BSL declares `{exp, log, floor, rng-draw}` and has **no `round`**; `floor(x+0.5)` is half-**up** and diverges at exact ties. **In-scope state-affecting sites: ZERO** (the 2 that exist are `reserve_army/accumulation.py:115-123`, outside the boundary). Payloads emit full precision. Task 0 Step 2 re-verifies; a state-affecting in-scope site is a STOP |
| **D-NF+17** | **R6's readout, the four retired defines, and what is deliberately NOT deleted** | The frozen `BifurcationRiskCalculator.compute()` formula retires **wholesale, not repaired** — findings F3/F4/F5/F6 are **MOOT, not separately disposed**, and **F5's `node.id == fips` repair is explicitly not owed**. Four defines collapse to one: `bifurcation_event_threshold` survives as a `defconst`; `w_s`, `w_b`, `class_burden_epsilon` are **not transcribed**, and their **Python deletion is a WS4/python-deletion-ledger motion, NOT executed here** (deleting them would move `defines_hash` and cost an 11-baseline ceremony for zero engine effect). Also **closes the T4 dossier's blank result-kind cell**: a weighted `fold mean` over an intensive body with an extensive `:weight` is explicitly sanctioned (`typecheck.rs:178-202`) |
| **D-NF+18** | **The Φ rule's one-tick lag** | `economics/` sorts **after** `class-dynamics/` in rule-id byte order, so the adjustment `a01` reads at a boundary is the previous tick's. Provably unobservable in this train's worlds (`phi-hour` is static content). **Re-open trigger named: when the ImperialRent train makes `phi-hour` move within a year** |
| **D-NF+19** | **`phi_cap` is a BSL `defconst`, NOT a `GameDefines` define** | R9 says "promoted to a define"; R8 — the same sitting, the same coefficient estate — rules "**no defines.yaml churn; no §6.5 ceremony**", and the dossier's D6-A language names `:material-basis` provenance, which is **BSL rule metadata**. A real define moves `canonical_defines_hash` (gated at `tools/regression_test.py:1279-1283`), costs 11 baselines, and has **zero effect on an engine that does not read `GameDefines`**. §10 DG-9 puts the reading to the Director rather than assuming it |
| **D-NF+20** | **R4's asymmetry is a FEATURE, and the readout is guarded to prove it carries it** | The revolutionary term zeroes under no-SOLIDARITY-seeding **by design** — *"revolutionary crisis direction must be EARNED BY ORGANIZING … fascism is the default drift of unorganized crisis"* (ADR016 prior art quoted in the pack header). The asymmetry lives in `consciousness/p6-route`, which R6 makes the one expression; **this pack re-implements nothing and instead ships world 6's `the_unorganized_county_drifts_fascist_and_the_organized_one_does_not`** as the executable form |
| **D-NF+21** | **FIPS is dropped; node identity carries county identity** | The landed `dispossession.bsl:99-110` precedent plus ADR198 R7's *"where the string was really naming a node, key by node identity instead"*. **Consequence stated, not covered: ADR198 R7's int-FIPS leading-zero trap gains NO witness on this train** and remains unexercised |
| **D-NF+22** | **This pack declares NO intrinsic — and `floor` is ALREADY declared TWICE in the estate** | Rev 1's law (*"`floor` is declared once, by `territory.bsl:78`"*) is **false**: `decomposition.bsl:212` carries a byte-identical declaration, both headers disclose it, the loader refuses duplicates BY NAME with no content comparison (`declarations.rs:1037-1046` (the doc at `:1037`, the `DeclError::Duplicate` raise at `:1044`; `:1009` is the SignatureMismatch arm rev 1 inherited from `territory.bsl`'s own header — N12, mechanism unchanged)), and **issue #646 is open**. Consequence for this train: **no single world can co-load this pack with both `territory.bsl` and `decomposition.bsl`**, so §2.3's obligation lands as TWO co-load worlds (Task 10 Step 2), each naming what the split costs. This pack causes none of it — it needs no intrinsic (§4.4's gate is `:tick-in-cycle`), §6 has no transcendental, and §7c guard 2 asserts the absence at source level. **Re-open trigger: #646's dedup/prelude for rule-file intrinsics, which collapses the two worlds back into one** |
| **D-NF+23** | **The D116 same-tick cross-rule ledger for this pack, and the D136 territory-side-fold record read correctly** | Seven deliberate reliances (§7b), each named with how it breaks when D116's collect-across-rules repair lands. **Plus the citation rev 1 got wrong four times:** the territory-side double-count record is **D136** (`production.bsl:83-107` — an abandoned `fold sum` that double-counted `worker-pp-two-lands`, whose own register row's "no `.bsl`-level fix" claim was itself caught false), **not D45** (`bsl-language.rst:5103-5108`, the `select-max`/`select-min` ascending-id tiebreak), and `production.bsl` **contains no fold at all** (`:145`). The argument this pack owes and now makes: D136's hazard is specific to a conserved **sum**; R6's readout is a population-weighted **mean**, which a class tenanted in two counties should legitimately enter twice, once per county, at its own weight. World 1's `shared-class` pins it |
| **D-NF+24** | **The duplication ledger** | Four expressions transcribed more than once (§7a); single-sourcing is unavailable in the language (no `defexpr`, no macro, no cross-rule `let`), so each pair owes a **named copies-agree row** and a perturb-one-copy vector |
| **D-NF+25** | **Cross-train write-set disjointness** | The ImperialRent and Community trains land in unknown order; the proof is by construction (§2.3) plus two mechanical obligations — the Task-0/Task-12 collision grep and Task 10's co-load world. This pack writes only `territory/*` plus the one net-new `social-class/ternary-net-fascist`, and **touches no `institution/*`, no `community/*`, no `rent-*`, and none of `consciousness.bsl`'s or `decomposition.bsl`'s vocabulary** |
| **D-NF+26** | **R7's measured-membership semantics land in CONTENT; the frozen docstrings are NOT edited** | The percentile-band descriptions (`types.py:37-41,58-68`) are the F18 defect and are **transcribed nowhere**; the five bootstrap values survive as SEEDS (R5-confirmed; `0.40 ≡ 0.90 − 0.50` is percentile arithmetic, which is exactly why the *description* dies and the *value* does not). ADR183 R2 forbids frozen-lane repair, so the ruling lands in the `deffield` rows, the `:material-basis` provenance and the pack header. §10 DG-11 asks whether the Director additionally wants the Python edit |
| **D-NF+27** | **`crisis-phase` is declared input with no producer on the ported estate** | The Step-5 `MultiPeriodCrisisDetector` (5 phases, 4 quarterly evaluations per annual boundary) is outside the boundary, so the pack's most theory-laden behavior is exercised **only by authored worlds** until the crisis-detector train lands. Stated in the pack header, the ADR and a filed issue — never left for the next reader to discover |
| **D-NF+28** | **Sum-to-one is a THEOREM, not a runtime check** | The frozen `model_validator` (`types.py:70-83`) and `_validate_distributions`' `raise ValueError` have no BSL analogue, and S-11 forbids a warning level. **F10's exact mass conservation** makes the invariant provable arithmetically; it lands as a property test at `1e-12` — **tighter than the frozen `0.001` tolerance** |
| **D-NF+29** | **R9's ruled home is a SHARED file: the blast radius, the declaration-only scenario extension, and the one Rust test repair** | `fundamental-theorem.bsl` is `include_str!`d by `tick_goldens.rs`, `babylon-client/src/engine_link.rs`, `babylon-client/tests/engine_link.rs` and `babylon-tick/src/lib.rs`, all over `two-classes.bscn` — a world with **no `territory/*` field and no `defconst`**, so the added rule dies twice at load (`E-LOAD-010` at `resolve_bindings`, which `:optional` does **not** rescue — `bindings.rs:448-451`; and `check_sources_servable`'s `:const` refusal at `tick.rs:439-451`, reached from `run_tick`'s entry at `:583`). **Resolution: `two-classes.bscn` gains DECLARATIONS ONLY** — no node, no attribute, no edge — which the canonical state-hash layout does not cover (`state_hash.rs:10-30`: sections `0x01`–`0x05` are nodes/attributes/edges/hyperedges/edge-attributes) and which hydration never materializes (`scenario.rs:1236-1275` stamps only explicitly seeded pairs); the rule then finds **zero TERRITORY subjects** (`tick.rs:166-189` + `hypergraph_store.rs:310-319`) and writes nothing. **All 17 pre-existing pinned hashes hold; the single consequence is `lib.rs`'s `per_rule_fired.len() == 1` unit test, renamed and updated to 2 with the Φ rule's contribution asserted as 0.** Rejected: seeding a TERRITORY (moves two pins), splitting the file (forfeits R9's ruled home — a Director call), optional bindings (not expressible) |
| **D-NF+30** | **The pin tick is part of the pin: tick-1 LOAD pins plus tick-52 BOUNDARY pins** | `run_once` is tick 1 (`lib.rs:517-531`), `TickSession` starts at 0 and runs 1 first (`session.rs:60-66,120-124`), so **tick 0 never executes**; with `tick.rem_euclid(52)` (`tick.rs:269`) this pack's gate first opens at **tick 52**. A `run_once` pin over this pack therefore records `fired = 0` and `before == after` — it pins the seeded world and the pack's inertness, and **nothing of the arithmetic**. Every world takes both kinds; `tick_goldens.rs` gains its **first session-driven pin**, declared in the file's own header against its existing tick-1-alone convention (`:697-706`), on the landed `TickSession` estate `carceral_arc_conformance.rs` already drives |
| **D-NF+31** | **`a13`'s fold carries an `exists` protector, and a classless county gets NO score** | Bindings evaluate unconditionally, before the guard (`tick.rs:583-609`; `control-ratio.bsl` `c03`'s own `:material-basis`), and `mean` over an empty set is `E-EVAL-021` — *"there is no element to return and there is no null"* (`evaluator.rs:143-147`). So `(when (= phase-of-year 0))` protects nothing: **any TERRITORY without an incoming-TENANCY class would kill every tick, tick 1 included, in every world loading this pack.** The landed protector is `territory.bsl:168-172`'s `(if (exists …) <fold> (- 0 0c))`, copied verbatim; the WRITE is additionally guarded so no zero is fabricated for a county that has no classes (III.11, the same refusal `a07`'s degenerate arm makes). World 8 is the witness; the mutation vector is the removal of the protector |
| **D-NF+32** | **The staging fields' declared types are load-bearing** | A store outside a declared range is `E-EVAL-020`, *"a loud failure, never a clamp"* (`evaluator.rs:139-142`), and `probability`/`intensity`/`coefficient` are `[0,1]` while `real` carries no range law (`types.rs:230-244`). The three `raw-share-*` fields are therefore **`real`**: `a06`'s flow equations produce negatives — which is precisely why `_normalize` clamps at `transition_engine.py:313-315` — and world 4 is seeded so **all three go negative**. Declared `probability`, `a06` would abort and the only fixture that reaches the degenerate branch would never reach it. The four `rate-*` fields stay `probability` deliberately: their `[0,1]` declaration is what makes a deleted clamp a LOUD failure rather than a silent one |

---

## Estimate

**13 tasks · 13 commits · 5 PRs · ~71 agent-hours** — a **point estimate**, the exact sum of the per-task figures:

| task | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | **Σ** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| hours | 3 | 5 | 7 | 6 | 6 | 5 | 5 | 8 | 7 | 7 | 5 | 4 | 3 | **71** |
| tokens (k) | 45 | 70 | 95 | 85 | 85 | 75 | 70 | 110 | 100 | 95 | 70 | 60 | 50 | **1010** |

**~1,010k tokens**, plus **five** Copilot-harvest review cycles (ADR181). Commits: one per task = **13**
(PR A 3, PR B 2, PR C 2, PR D 2, PR E 4 — the PR table's column sums to 13).

**Rev-2 delta, itemized** (+4h / +40k): Task 1 **−1h** (three of five spike questions became source-answered
confirmations — the `:weight` shape, tick-0, the `EventType` opt-in); Task 6 **+1h** (the session-pin convention
and six pins instead of three); Task 8 **+1h** (the C5 protector, its mutation vector and the world-8 witness);
Task 9 **+2h** (the shared-file consumer enumeration, the `two-classes.bscn` declaration-only edit with its
green-before-the-rule proof, the `lib.rs` test repair, the corrected Φ divisor and its two extra fixtures);
Task 10 **+1h** (two co-load worlds instead of one, 18 pins instead of 7).

**Where the weight sits.** Capability + declarations (Tasks 0–2) = **15h**; rule arithmetic (Tasks 3–6) = **22h**;
theory execution (Tasks 7–9) = **22h**; closure (Tasks 10–12) = **12h**. **This is still a content train**, but rev
2 corrects one claim rev 1 made twice: it is **not** true that "one Rust source line changes". Three Rust-source
touches land — the registration string (Task 2), **one unit-test repair in `babylon-tick/src/lib.rs` (Task 9)**,
and **a helper plus a header paragraph in `tick_goldens.rs` (Task 6)** — and the train's gate must run
`cargo test -p babylon-client`, a crate rev 1 never named. No crate gains *logic*, and `babylon-graph` is still
untouched.

**Three highest-variance items, RE-RANKED (rev 1's #1 was answered from source, I5).**
1. **Task 9 — the shared-file landing (C1/BLOCKER-8).** The declaration-only extension of `two-classes.bscn` is
   argued from three separate source facts (the canonical hash layout, hydration's seed-only stamping, the
   empty-subject fold-through). The argument is strong and **verified**, but it is the only place in this train
   where being wrong moves a pin in a shipped client's test — and Step 1's "green before the rule lands" check is
   what converts the argument into evidence. **If it fails, the train stops for a Director call on splitting R9's
   ruled home.**
2. **Task 7 — R10's arc.** A **≥156-tick** session whose expected event sequence depends on `a10`'s ordering
   (decided by test) and on the F11 repair's magnitude (re-derived from the mirror). Two coupled unknowns in one
   test — rev 1 counted three, but the tick-0 unknown is now a source-answered fact — and it is the test the
   Director reads.
3. **Task 1 Step 3 — the `x1e6` bit-exactness.** If the promoted `(defconst … 3500000)` ÷ `1000000` is not
   bit-identical to the frozen `rate * 3.5`, **seven** amplifier multipliers need either a re-scale or a declared
   tolerance with a written derivation — cheap at Task 1, expensive at Task 4 with 20 constants written. Rev 2
   additionally fixes the spike's *form* (Int ÷ Int is a loud error) and its *operand order* (divide-first is one
   rounding; the cited `metabolism.bsl` precedent multiplies first, which is two).

---

## Self-review notes (plan author)

- **Every construct is landed and cited.** `:tick-in-cycle` served (`bindings.rs:55-59,410-416`; `tick.rs:269`;
  `score_class.rs:156`); `:year`/`:tick-of-year` refused (`tick.rs:456,462`); the ONE landed `neighbors`-scoped
  fold, **with its `exists` protector** (`territory.bsl:168-172`) **and** `(nodes …)`-scoped folds
  (`decomposition.bsl:284-291`, `control-ratio.bsl:281-287`) — **rev 1's "`production.bsl` ×11" counted
  `exists`/`select-max` uses as folds; that pack contains no fold at all (`:145`)**; the five-member `FoldOp` set (`grammar.rs:672-683`); weighted `fold mean` over an
  intensive body with an extensive `:weight` (`typecheck.rs:178-202`); the body's score class flowing through
  (`score_class.rs:210-223`); `field-of` over an enum field (D102 discharged); enum seeding (`E-LOAD-056`);
  fractional `real`/`p`/`i`/`c` seeding (`scenario.rs:1093-1330`); `defconst` literal lanes; `guard` inside
  effects; `emit` with a `CollectingSink` assertion; the closed `add|sub|set|scale` op set; the `x1e6` lane
  (`metabolism-conformance.bscn:22-24`); `(binding tick :tick)` (`decomposition.bsl:256`).
- **The genuine capability risks are TWO, not five, and Task 1 converts each into evidence before a rule depends
  on it:** the `x1e6` descale's bit-exactness, and the enum seed+read pair. **Three of rev 1's five were
  answerable from files rev 1 itself listed as required reading** — the `:weight` shape
  (`rule_pipeline.rs:744-760`), tick-0 (`session.rs:60-66`, `lib.rs:517-531`) and the `EventType` opt-in (no
  landed `.bscn` declares one) — and rev 2 records them as confirmations with citations, because a spike that
  re-asks an answered question spends a task's budget on reassurance.
- **What I could not verify and left as a task obligation:** every `:fuel` figure (Task 10's declare-bound+1
  sweep — **no rule ships a guessed number**); whether an `int` field seeded at 2006 loads at all (Task 6 Step 1);
  the exact pin count if either co-load world takes its own pin (Task 10 Step 3 measures, then states it); and
  whether `two-classes.bscn`'s two pinned hashes hold under the declaration-only edit — **argued from three source
  facts in §2.2.3 and made executable by Task 9 Step 1's green-before-the-rule check, but not executed here.**
- **Corrections this plan makes to its own predecessors, each verifiable in one command.** `validation.py` carries
  **32** constants, not 29 — so the estate is **70**, not "~66" (§1.6). The pre-existing pinned hashes are **17**
  (16 in `tick_goldens.rs` + `babylon-client`'s), not 16 and not 8. `(nodes …)`-ranged folds **are** landed. Fractional per-node seeding **is** legal. `phi_cap` is a
  **defconst**, not a define — which deletes an entire ceremony-bearing frozen-lane task the earlier draft
  scheduled. **Zero Python-lane changes**, which deletes the second one. Numbering is **next-free-at-landing**
  against a **four-way**-contended tail, never literal. And the Checkpoint-A claim is **measured, not inherited**
  (§0.2): ReserveArmy @5.0 is unstarted, so **WS3 stays HELD**.
- **Numbers this plan asserts that the implementer must RE-DERIVE, not trust:** every rate and share value, the
  33× F11 factor, F19's ~2.5pp envelope, the arc's per-boundary decline series, every `report.fired` count, and
  every golden hash. All come from the mirrors and the engine's own runs, and **the mirrors are the contract**.
- **Fixture design intent, stated so a reviewer can audit it.** `shared-class` (two TENANCY edges) exists so
  **D136's double-count question is answered for a MEAN rather than inherited from a SUM** (§2.4); `orphan-class` proves edge-scoped
  aggregation misses nothing it should catch; the two-populations/two-`(f−r)` pair makes the population weighting
  provable (ADR070's read policy, F4's repair); the `9.60`/`9.59` pair makes the strict-`<` halt provable; world 4
  exists **solely** so D-NF+8's repair is not a silent divergence (and only because `raw-share-*` is `real`);
  world 6 exists **solely** so R4's ruling is executable rather than commented; **world 8 exists solely so the
  empty-TENANCY abort is a fixture rather than a production discovery**; and the `$182.05/hr` accumulation-clamp
  probe is labelled absurd-by-construction rather than passed off as a county.
- **The FOUR places a reviewer disagreement should STOP the train rather than reshape it.** (1) **§0.1's pack
  boundary** — the plan's one irreversible structural choice; if R15 is read to require all of @4.0 in one train,
  the answer is an ADR or a Director escalation, never a shrunken pack. (2) **§4.5 / DG-8's reading of R8** —
  41 of the 70 coefficients are declined here, 32 of them driving only log lines III.11 has no warning level to
  receive; the alternative (declare all 41 in every world as content nothing reads) is available, cheaper to
  argue, and worse. (3) **D-NF+19 / DG-9's reading of `phi_cap`** — a "real define" answer adds a Python commit
  and an 11-baseline ceremony this plan is built to avoid. (4) **NEW — §2.2.3's declaration-only extension of
  `two-classes.bscn`.** If a reviewer holds that a landed golden's world may not gain declarations at all, then
  R9's ruled home and the pin law genuinely cannot both be satisfied by this train, and the escalation is a
  Director call on splitting `fundamental-theorem.bsl` — **not a quiet re-pin, and not a sibling file chosen by
  the workforce.**
- **What this plan deliberately does NOT do, each refusal a D-row or an issue, never a silence:** it does not touch
  any Python source; it does not delete the four bifurcation defines from `GameDefines`; it does not port the
  three-tier validator, `DefaultCrisisAmplifier`, `p_to_l_component`, or four of five savings rates; it does not
  port the Vol I wage-pressure sigmoid (and names it as an **imposed form to be re-derived, never transcribed**);
  it does not implement the crisis detector; it does not answer a single §10 question; and **it does not claim
  Checkpoint A**.

---

## What revision 2 changed

**Input:** the adversarial critique of rev 1 (5 Critical · 10 Important · 12 Minor, verdict EXECUTION-READY: NO).
**Method:** every finding re-verified at the byte in this worktree before any edit was made; three findings were
found partly or wholly wrong and are rejected **with the measurement that rejects them**, because a revision that
accepts a critique uncritically is the same failure as a plan that accepts a dossier uncritically.

**Rev 2 is a repair, not a re-charter.** §0's boundary argument, §0.2's Checkpoint-A refusal, the frozen-source
archaeology, the 70-row/46-defconst arithmetic, the ruling transcriptions and the PR split all survive intact.
What changed is concentrated in the Rust/BSL execution model — the half rev 1 cited confidently and did not
execute.

### The five Criticals

| # | what was wrong | what rev 2 does |
|---|---|---|
| **C1** | The R9 edit to `fundamental-theorem.bsl` dies at LOAD against `two-classes.bscn` (`E-LOAD-010`; `check_sources_servable`), breaking a golden pin, three unit tests, and **a 17th pinned hash in `babylon-client`** that rev 1 never inventoried. | **§2.2 rewritten** into four subsections: the four-consumer table, the two refusals with their source lines, and the resolution — a **declaration-only** extension of `two-classes.bscn`, proved hash-neutral from `state_hash.rs:10-30` (the canonical layout hashes graph state only) and `scenario.rs:1236-1275` (hydration stamps only seeded pairs), with the rule firing zero times via `subject_type_of` + `nodes()`. **R9 lands verbatim and all 17 pins hold.** One `lib.rs` unit test is repaired. Rejected alternatives and their costs are tabled. New: BLOCKER-8, D-NF+29, Task 0 Step 4b, Task 9 Steps 0–2, a Global bullet making consumer enumeration mandatory before any shared-file edit. **The `:optional`/`:default` escape the brief hypothesised does NOT exist — `resolve_bindings` refuses an unknown qname regardless of optionality (`bindings.rs:448-451`), and that closure is recorded so no later reader retries it.** |
| **C2** | All 7 additive pins were inert: `tick_goldens.rs` pins run at tick 1, the pack's gate opens at tick 52. | **§4.4 now answers the tick-0 question from source** (tick 0 never executes; first boundary is tick 52), and the pin design splits into **load pins (tick 1) + boundary pins (tick 52)**, 18 in total. Task 6 Step 4 lands the session-driven convention in `tick_goldens.rs` with its own header paragraph; Task 1 Step 2b spikes the shape first. New: BLOCKER-7, D-NF+30, a Global bullet. The arc world is sized at **≥156 ticks** and takes three boundary pins. |
| **C3** | The Φ rule divided by the **hourly** wage where the frozen engine divides by the **annual, halt-zeroed** wage — a 2080× error pinning R9's gradient at its cap. | **§1.2 carries the full unit trace**; **§7's rule is rewritten** to compute `a01`'s exact wage base (×2080, FR-017-halted) and divide by it, with `2080` written on both sides rather than cancelled. §7a's copies-agree row becomes writable and is named as the row that would have caught it; Task 9 gains a halted-county fixture and a "neither county at the cap" assertion. **§8 is repaired too**: a mirror disagreement is now a **STOP-then-diagnose against a closed list of intended divergences**, not an automatic D-row — rev 1's protocol would have *recorded* this bug instead of catching it. Units are now printed in every mirror label. |
| **C4** | The co-load world cannot load: `territory.bsl` and `decomposition.bsl` both declare `(intrinsic floor …)` (#646). | **The Global "floor is declared once" law is struck and replaced with the measured fact.** §2.3's obligation 2 becomes **two co-load worlds** (A: territory-side; B: decomposition-side), each naming what it proves and **what the split loses** — no three-way co-load exists for any pack until #646 lands. Task 10 Step 2 records the `E-LOAD-001` refusal text as evidence before splitting. D-NF+22 rewritten with #646 as the retirement trigger. |
| **C5** | `a13`'s fold hard-aborts (`E-EVAL-021`) on any TERRITORY with no TENANCY-incident class — bindings evaluate before the guard, so the boundary `when` protects nothing. | **§4.6 carries the hazard as a first-class fact**, `a13` is rewritten with `territory.bsl:168-172`'s `exists` protector **and** a guarded write (a classless county gets no score, never a fabricated zero). New: **world 8**, the empty-fold witness; a Task-1 spike that records the refusal before the fix; a Task-8 mutation vector; three world-design laws in §Worlds; D-NF+31; a Global bullet on unconditional binding evaluation. |

### The ten Importants — all resolved

**I1** R8's binding list: DG-8 now covers all **41** declined rows, not 32; the plan no longer narrows a ruling for
9 rows and escalates for 32. **I2** B10 quoted verbatim in §Global with the STOP-vs-ceremony fork written out.
**I3** `p7-persist-baselines` is a **rolling** persister; `a10`'s real precedent is `decomposition.bsl:248-260`'s
write-once latch, and **DG-6 is upgraded from "resolved by construction" to a live question**. **I4** D45 → D136
throughout, `production.bsl` has no folds, and the mean-vs-sum argument is now *made* rather than cited. **I5**
BLOCKER-6 retired at the byte; Task 8's fallback branch struck; variance re-ranked. **I6** the seven staging
fields are declared with types and derivations — `raw-share-*` is **`real`**, without which world 4 aborts instead
of exercising the D-NF+8 repair. **I7** re-measured: **six** two-sided clamp sites tree-wide, **two** in boundary,
all enumerated (**the critique's "four" undercounted — `:810` and `:824` are two-sided too**). **I8** the `0.08`
clamp's fixture is a labelled synthetic bound probe with its `$182.05/hr` derivation, and "binds for the first
time" is softened to "becomes reachable". **I9** the `x1e6` spike is rewritten in the legal promoted form, and the
divide-first operand order is justified against `metabolism.bsl`'s multiply-first precedent. **I10** the dossier
path is normalized to `reports/tickdynamics-trio-dossier-2026-08-17.md` (1,142 lines).

### The twelve Minors — 10 fixed, 2 rejected with reasons

Fixed: **M1** (the grep returns 7 `mobility` hits, not zero — claim narrowed), **M3** (four bootstrap literal
sites, `initializer.py:34` added), **M4** (one roster count: 17 + 7 staging), **M5** (10 scenario files),
**M6** (DG numbering de-collided; DG-11 minted; DG-7 gates **PR C**, not PR B), **M7** (`a12` carries a `when`
form, not "no `when`" — rev 2 chose `(when #t)`, which **rev 2.1 supersedes with the boundary gate**, see N1),
**M8** (six line citations corrected), **M9** (`field_ref_for` reduces **four** shapes), **M10** (the second
double-run mirror is named — world 2's), **M11** (the `EventType` opt-in is answered from the tree), **M12** (the
arc is sized at ≥156 ticks).

**Rejected, with the measurement:** **M2** — the critique says the seven named `dynamics/` modules total 1,276
lines; measured, they total **exactly 1,476** (346+321+300+181+127+106+95), which is precisely the figure F9 cites.
Rev 1's phrasing was loose about which files reach 1,927 (all ten, including `__init__.py`), and §Prior-art now
says both numbers exactly. **I7's count** is likewise rejected as stated (four two-sided sites) in favour of the
measured six, while its *substantive* point — that only two are in this pack's boundary — is adopted.

### Totals

| | rev 1 | rev 2 |
|---|---|---|
| tasks / commits / PRs | 13 / 13 / 5 | **13 / 13 / 5** (unchanged) |
| D-rows | 28 | **32** (+D-NF+29 shared-file blast radius, +30 pin convention, +31 empty-fold protector, +32 staging-field types) |
| conformance worlds | 7 stated / 8 built | **8 content + 2 co-load**, and one pre-existing world extended |
| additive golden pins | 7 (all inert) | **18** (8 load + 8 boundary + 2 arc) |
| pre-existing pinned hashes held | 16 | **17** (the `babylon-client` pin inventoried) |
| Director-gate questions | 10 | **11** |
| estimate | ~67h / ~970k | **~71h / ~1,010k** |
| Rust-source touches | "one line" | **three** (registration string, one unit-test repair, one goldens helper + header) |

**No STOP finding.** R9 lands verbatim — the circuit homes in `fundamental-theorem.bsl` with `phi_cap` as a BSL
`defconst` — and every pre-existing pinned hash stays byte-identical, because the mechanism that reconciles them
(declarations are not graph state) is a property of the canonical layout, not a compromise between the two laws.
The one place that reconciliation could fail is named, made executable **before** the rule lands (Task 9 Step 1),
and given its escalation path if it does.

---

## Revision 2.1 — the one owed decision (N1), and the drift it touched

The rev-2 re-verify returned **0 Critical · 1 Important · 11 Minor, EXECUTION-READY: YES**, with the Important
(**N1**) owed **before Task 6 Step 4**. This section records the decision and its consequences; it is a decision,
not a patch, and the reasoning lives beside the law it settles (§7's `a12` row) rather than only here.

### The contradiction

Rev 2's M7 fix corrected `a12`'s guard from "no `when`" to **`(when #t)`** — the right *spelling* of the landed
unconditional idiom. But `(when #t)` **replaces** the boundary guard, so `a12` fired every tick, and three of the
plan's own laws said otherwise: §7's preamble ("every rule in this pack carries the boundary gate", naming exactly
one exception, in the *other* pack), §7a's `the_pack_is_inert_off_the_boundary` ("all 13 rules"), and the tick-1
load pin's `fired == 0` — false by construction in any world holding classes, and worst in the co-load worlds,
where `consciousness/p6-route` moves the ternary every tick.

### The decision: **(a) — `a12` carries the boundary gate**

`a12`'s guard is `(when (= phase-of-year 0))`, like every other rule in the pack. §7's law stays **absolute**; the
load pin's `fired == 0` stays **true**; §7a's 13-copy row stays **true**; no world-authoring requirement is added.

**Why (a) and not (b), in the terms the two options actually differ on.**

1. **The freshness the D127 idiom protects is preserved where it is observable.** `a13` is the field's ONLY
   reader; it reads at a boundary; `a12` sorts first and runs to completion first (§7b, D116). So the fold always
   consumes a publication written **in the same tick**. Between boundaries the field is stale — and that is this
   pack's uniform annual semantics, shared by every `rate-*`, `raw-share-*` and `share-*` field it publishes. (b)
   would have bought freshness no consumer can see, at the price of three false laws.
2. **(a) is the more faithful transcription.** The frozen bifurcation readout is an annual-boundary computation
   (Step 5b, called once per boundary); R6 rules **what** the readout is, not how often it is taken. Gating `a12`
   matches the frozen cadence; ungating it would have made one rule of a strictly annual pack run 52× a year for
   no ruled reason.
3. **(b) carried an unstated obligation.** Its `before == after` half depends on `social-class/ternary-net-fascist`
   being seeded at exactly `fascist − revolutionary` in every world — an idempotent-seed requirement no test
   enforces and no law states. (a) needs no such requirement.
4. **What M7 was fixing is not lost.** M7 was about the *spelling* of a guard, never about unconditionality; under
   (a) `a12` carries a `when` form exactly like every landed rule, so M7's point holds a fortiori.

**The cost of (a), recorded where the reliance lives (§7b).** With both rules gated, a D116 repair
(collect-across-rules-then-apply) would make `a13` fold the **previous boundary's** publication — a one-**year**
lag, where the rejected shape would have cost one tick. Nothing observes it until D116's own train lands; that
train's ledger now names this row. **This is the single respect in which (b) was better, and it is stated rather
than traded away silently.**

**Made executable, not asserted:** Task 8 Step 1 adds `a12_writes_nothing_off_the_boundary` (the field
byte-identical across two non-boundary ticks *while `p6-route` moves the ternary underneath it*), and Task 8
Step 4's mutation **re-tries the rejected shape** — replacing the gate with `(when #t)` must flip that test, flip
`the_pack_is_inert_off_the_boundary`, and break the tick-1 load pin. A future reader who reaches for the
unconditional form is stopped by a test, not by a comment.

### The 11 Minors — all closed in the same pass, each re-measured first

**Method note, because rev 2 failed exactly here:** rev 2 accepted critique row M8's citation "fixes" without
re-measuring and thereby *regressed three correct citations* (N3/N4/N5). Every finding below was measured in this
worktree before the edit.

| # | measured | disposition |
|---|---|---|
| **N2** | world 8 appeared in §Worlds, File Structure, D-NF+31 and the pin count with no creating task | **FIXED** — Task 8 Files line + new **Step 1b** create `class-dynamics-classless-county-conformance.bscn` |
| **N3** | `accumulation = (wage − consumption) * effective_savings` is at **`:90`**; `consumption` at `:89` | **REVERTED** — rev 1's `:90` was right; rev 2's "fix" to `:89` was the regression, and D-NF+7's `:90` no longer contradicts §5 |
| **N4** | `PROHIBITED_INTRINSIC_NAMES` is at **`:131`**, inside `:125-131` | **REVERTED** — rev 2's "`:132`, outside rev 1's range" was wrong twice |
| **N5** | `math.exp` at **`:52`** and `:57`; `:51` is the overflow clamp; the defines at `:44-46` are exact | **REVERTED + widened** |
| **N6** | `get_phi_adjustment` is called at **`:86`**; `:85` is `base_rate`; the computation is `:85-90` | **FIXED** in both §1.2 sites |
| **N7** | Task 1's own Estimate line still read ~6h/~80k against the table's 5h/70k (per-task lines summed to 72h/1,020k) | **FIXED** — the line now matches the table; the sums close at **71h / 1,010k** |
| **N8** | §2.3's write-set row still said "16 fields" | **FIXED** — "17 declared + 7 staging"; the third stale count M4 named |
| **N9** | §4.5's "46 × 7 = 322 literals" | **FIXED** — **46 × 8 = 368**, plus the co-load subsets and `two-classes.bscn`'s four |
| **N10** | Task 7 Step 4 said "six worlds"; five exist at that point | **FIXED** — five, with the world numbers named |
| **N11** | `dossier-tickdynamics-trio.md` still cited at two sites against §Prior-art's "normalizes every citation" | **FIXED** — both normalized; the one remaining mention is §Prior-art's own explanation of why the name is wrong |
| **N12** | `declarations.rs:1010-1017` is the `SignatureMismatch` arm; the duplicate refusal is `:1037-1046` (`Duplicate` raised at `:1044`) | **FIXED** at all three sites; the mechanism was always real, only the range was inherited wrong from `territory.bsl`'s header |
| **N13** (note) | §8's closed divergence list omitted D-NF+16's `round()`-drop | **CLOSED BY ARGUMENT** — the list now says why it cannot arise (the mirrors drive the engine; payload rounding is call-site, outside the oracle) **and** that a rounding disagreement appearing there would itself be the finding |

### Totals after 2.1

Unchanged: 13 tasks / 13 commits / 5 PRs / 32 D-rows / 8 content + 2 co-load worlds / 18 additive pins /
17 pre-existing pinned hashes held / 11 Director-gate questions / **~71h · ~1,010k** (N7's fix is what makes the
per-task lines sum to the table). One rule's guard changed; two tests were added to prove it; no scope moved.
