; `class-dynamics/*` rule pack — Feature-016 Class Dynamics Engine
; (Material Base @4.0), port train `docs/superpowers/plans/2026-08-18-
; tickdynamics-port.md`.
;
; **§0.1 boundary.** This pack ports ONLY the class-dynamics engine — the
; seven modules under `src/babylon/domain/economics/dynamics/` and the single
; caller at `src/babylon/domain/economics/tick/system/__init__.py:2346-2458`.
; It does NOT port all of TickDynamicsSystem @4.0.
;
; **Header-only at Task 2.** The 13 rules (a01-a13) and the `economics/phi-
; savings-coupling` edit land in later tasks. This file today carries only the
; SPIKE RESULTS block from Task 0's dossier (§10), copied here per the plan's
; instruction that the pack header owns the surviving spike record.
;
; D-NF+1: the "Step 5b executes after Step 6" doc comment numbering in
; transition_engine.py is stale relative to actual call order; this pack's
; rule-id byte order is the contract.
;
; ---- SPIKE RESULTS (Task 1, 2026-08-18) ----
;
; Task 1's own spike, executed against temporary content
; (`content/rules/class-dynamics.bsl`, `content/scenarios/class-dynamics-
; spike.bscn`, `tests/class_dynamics_spike.rs` — all three deleted at Step 6;
; this section is the authoritative surviving record. Rule ids used the
; already-registered `social-class`/`territory` namespaces (babylon-tick's
; `lib.rs` registered-systems set) rather than `class-dynamics` — Task 1
; touches no production code; `class-dynamics` registration is Task 2's own
; job. All 6 spike tests passed; verbatim evidence below.
;
; **Three CONFIRMATIONS (source-answered, cited first — never upgraded back
; into open questions):**
;
; 1. **Step 1 — BLOCKER-6, retired (I5).** Source: `rule_pipeline.rs:744-760`
;    reduces the `:weight` operand through `field_ref_for` exactly as the fold
;    body. Loaded a rule with `(fold mean (nodes NodeType/SOCIAL_CLASS)
;    (field-of it social-class/wealth) :weight (field-of it social-class/
;    population))` — **loads and fires** (`fired == 2`, both class-a and
;    class-b). No fallback branch, no two-`fold sum` alternative needed.
;    **Verdict: CONFIRMED.**
; 2. **Step 2 — BLOCKER-7, retired.** Source: `session.rs:60-66,120-124`
;    (`TickSession::new` starts `tick: 0`; `advance` computes `next_tick =
;    self.tick + 1`) and `lib.rs:517-531` (`run_once`/`run_prepared_tick`
;    hard-code tick `1`) — tick 0 never executes, the first boundary is tick
;    52. Executed a `(binding phase-of-year :tick-in-cycle 52)` +
;    `(when (= phase-of-year 0))` rule over a 105-tick `TickSession`.
;    **Measured `fired` series (ticks 1..=105, this rule's own `per_rule_fired`
;    count each tick):** zero for ticks 1-51, **2 at tick 52**, zero for ticks
;    53-103, **2 at tick 104**, zero for tick 105 — exactly: `[0×51, 2, 0×51,
;    2, 0]`. This is the arithmetic every world's boundary pin inherits.
;    **Verdict: CONFIRMED.**
; 3. **Step 5 — M11, already answered.** Source: `rg -n 'defvocabulary EventType'
;    rust/crates/babylon-tick/content/` returns zero hits — no landed `.bscn`
;    opts into an `EventType` vocabulary. Executed `(emit EventType/
;    DISPOSSESSION_CASCADE (fips 1) (decline 0.05c))` verbatim (the brief's
;    own literal), no `defvocabulary EventType` declared anywhere in the spike
;    scenario. **Loaded and fired clean** — 2 `DISPOSSESSION_CASCADE` events
;    (one per SOCIAL_CLASS subject, both carrying the same literal payload),
;    `fips` read back as `Value::Int(1)`, `decline` read back as `Value::Real`
;    with `.to_bits() == 0.05_f64.to_bits()` bit-exact.
;    **Verdict: CONFIRMED — a world need not opt in.**
;
; **Four REAL spikes (actually run; observed evidence recorded verbatim):**
;
; 4. **Step 2b (NEW) — the session-driven golden convention.** Drove
;    `TickSession::advance` ×52 over a throwaway world (the same content set
;    as Step 2/5b), read `hex(&report.before)`/`hex(&report.after)` at tick
;    52. **Same-process stability:** two independent, freshly constructed
;    `TickSession`s (same content, same `SessionId`) produced byte-identical
;    tick-52 hashes:
;    `before=5c2b5dbc01023ab10a1aa38115d8e8b1bb45009c1a99c1c00f799ffaa48a7af1`,
;    `after=b666db12f811dfad8757240335ddfe22accb823c9ac102235613d9e29d5b9bac`.
;    **Cross-process stability:** ran `cargo test -p babylon-tick --test
;    class_dynamics_spike step2_boundary_series_step2b_session_golden_and_
;    step5b_positive -- --nocapture` as two genuinely separate OS process
;    invocations, each capturing the printed tick-52 hash lines to a file;
;    `diff`'d byte for byte — **identical, zero diff output** across both
;    processes. This is exactly the shape Task 6 Step 4 will land as the
;    file's first multi-tick pin. **Verdict: SPIKED AND PROVEN — the mechanism
;    is safe to build the real pin on.**
; 5. **Step 3 — BLOCKER-2, in the legal form.** `(defconst class-dynamics/
;    deep-precaritization-x1e6 3500000)` bound as the defines-environment
;    coefficient `class-dynamics/deep-precaritization-x1e6`, promoted before
;    the divide with `(binding m :expr (/ (- deep-precaritization-x1e6 0c)
;    1000000))` (the brief's own verbatim spelling — `Int ÷ Int` is a loud
;    error, rev 1's spelling could not have loaded, I9). **Measured: `m`
;    reads back bit-exactly `3.5_f64`** (3500000/1000000 is a terminating
;    binary fraction, so this promote-then-divide order introduces ZERO
;    rounding of its own — confirmed by `m.to_bits() == 3.5_f64.to_bits()`,
;    not merely `m == 3.5`). **`product` (= `m * rate`) reads back BIT-EXACTLY
;    equal to the mirror** — computed as `m_readback * rate_readback` natively
;    in Rust from the SAME f64 values the engine itself emitted
;    (`product.to_bits() == (m*rate).to_bits()`), never re-derived from an
;    independent decimal-string parse (which is not guaranteed bit-identical
;    to the engine's own `unscaled as f64 / 10^scale` conversion — the cross-
;    implementation-tolerance discipline). The value reads back bit-exact —
;    this spike needed no fallback (a different scale, or a declared
;    tolerance).
;    **The deliberate operand-order deviation from `metabolism.bsl:386-387`,
;    with its one-rounding-vs-two derivation:** metabolism's D-1 pattern
;    multiplies a per-node, generically inexact field value by the scaled-Int
;    constant FIRST (`raw-extraction * entropy-factor-x1e6`), THEN divides by
;    `1000000` — two operations, each rounding a generically-inexact
;    intermediate (metabolism's own header measures up to 2^15 ULP of
;    divergence from the frozen engine as a consequence). THIS spike's order
;    instead descales the **constant alone** first — a division of two exact
;    integers-as-reals whose quotient terminates in binary here
;    (`3500000/1000000 = 3.5` exactly) — before the result ever touches per-
;    node data. Only the FINAL multiply by `rate` then rounds: **one rounding,
;    in general, for THIS operand order, versus metabolism's generically
;    two.** This is a property of *this specific pattern* (divide the constant
;    alone, independent of any field read, before combining with anything
;    else) — it does **not** generalize to every scaled-Int constant: a
;    constant whose descaled decimal has no exact base-2 expansion would still
;    round at the constant-alone division step, so a future author reusing
;    this pattern should not assume zero-rounding without checking their own
;    constant the same way this spike checked `3500000`.
;    **Verdict: SPIKED — bit-exact for this constant, order documented, no
;    fallback needed.**
; 6. **Step 4 — D102's discharge (enum seed + read pair).** Seeded `(node
;    county-deep NodeType/TERRITORY (territory/crisis-phase CrisisPhase/DEEP)
;    …)` (the seeding path, `E-LOAD-056`'s member-only rule — the scenario
;    loads clean, confirming the seed is legal) and read it back via `(binding
;    phase :field territory/crisis-phase)` + `(if (= phase CrisisPhase/DEEP)
;    …)` (the read path). **Measured:** the rule fires for `county-deep` alone
;    (`fired == 1`; `county-tenanted`/`county-empty` are both `NORMAL` and do
;    not fire), and the emitted `phase` payload value reads back as
;    `Value::Enum { enum_type: "CrisisPhase", member: "DEEP" }`.
;    **`defenum` ordinal parity (hash-bearing, ADR195), asserted directly
;    against the registry:** `NORMAL=0, ONSET=1, EARLY=2, DEEP=3, RECOVERY=4`
;    — declaration order is the storage ordinal, confirmed exact.
;    **Verdict: SPIKED AND CONFIRMED.**
; 7. **Step 5b (NEW) — the empty-TENANCY fold protector (C5).** `county-empty`
;    is a TERRITORY with **zero** incoming-TENANCY SOCIAL_CLASS neighbours.
;    **WITHOUT the `exists` protector** (isolated inline rule, never co-loaded
;    with anything else — it kills every tick of any content set holding it):
;    ran tick 1 (`(when (= phase-of-year 0))` is **false** at tick 1,
;    `phase-of-year == 1`). **The tick still died.** Exact verbatim error
;    text:
;    ```
;    tick failed in rule territory/spike-step5b-without-protector: E-EVAL-021:
;    mean over an empty query (§4.4) — there is no element to average
;    ```
;    This is the whole point: bindings evaluate before the `when` guard, so
;    the boundary gate protects nothing against an empty fold.
;    **WITH the protector** (`territory.bsl:168-172`'s `exists` idiom copied
;    verbatim ahead of the fold, plus a guarded write — `(guard has-classes
;    (update-node self territory/spike-score (set score)))`): drove the SAME
;    content set to tick 52. **The tick survived.**
;    `county-tenanted/territory/spike-score` became **220.0** exactly
;    (`(100*40 + 300*60)/(40+60)`, the population-weighted mean, confirmed
;    via `assert_eq!` against the raw f64 value the tick itself wrote).
;    `county-empty/territory/spike-score` **stayed at its 42 sentinel,
;    untouched** — no score at all, not a fabricated zero (III.11).
;    **Verdict: SPIKED AND PROVEN — this ONE refusal is the evidence for the
;    C5 repair; without it the protector would read as defensive decoration.**
;
; **One INCIDENTAL FINDING (not one of the six named spikes, but load-bearing
; for Task 8's `a12`/`a13` and every future emit-only/constant-only/expr-only
; rule in this pack):**
; `(domain NodeType/…)` is **not** what `run_tick` uses to pick a rule's
; subject population at this engine slice. `babylon-bsl/src/tick.rs::
; subject_type_of` derives the subject type from the rule's own `:field`
; bindings ALONE — it does not consult an explicit `(domain …)` declaration,
; does not look at `update-node` targets, and does not look at `field-of self
; …` accessors. A rule declaring `(domain NodeType/SOCIAL_CLASS)` with zero
; `:field` bindings refuses **at run time** (not load time) with: `"the rule
; declares no :field binding, so it names no subject type — slice 1 runs
; rules over a population, not over the graph as a whole"` — confirmed
; empirically this task (Steps 1/2/3/5's first draft all hit this exactly).
; This independently reproduces `metabolism.bsl`'s own D-4 finding word for
; word (`"tick.rs::run_tick NEVER reads loaded.domain"`) from the content-
; author's side rather than the engine-reader's. **Consequence for this
; train's own plan text:** `a13`, **as literally quoted in the plan
; (`docs/superpowers/plans/2026-08-18-tickdynamics-port.md:1137`), carries
; ZERO `:field` bindings** — both `has-classes` and `score` are `:expr`
; -sourced, and `a13`'s own `update-node` target (`territory/bifurcation-
; score`) does **not** count toward subject-type inference either. **As
; literally written, `a13` would refuse at run time with the same "no :field
; binding" error the protector spike above hit before its own fix.** Task 8
; must add a genuine self-scoped `:field` binding to `a13` (any already-
; declared `territory/*` field the rule can legitimately read, even if only
; to anchor the subject type) before landing it — this spike's own `spike-
; step5b-with-protector` rule (`(binding subject-anchor :field territory/
; crisis-phase)`, unused beyond anchoring) is the minimal worked fix, and
; `crisis-phase` is not a `class-dynamics`-owned field so `a13`'s own author
; should pick a real anchor from among its OWN pack's `territory/*` fields
; instead (e.g. whichever field this train's plan already has `a13` reading
; for real, or a field `a01`-`a11` already publish onto `territory/*`, if any
; exist — worth checking at Task 8 time, not re-derived here).
;
; **Step 6 discharged:** this task's own commit deletes every spike artifact
; (the temporary `.bsl`/`.bscn`/`tests/*.rs` triple) — see the commit message
; for the disclosed test-support delta this spike legitimately needed (none
; beyond the three deleted files: no production code changed).
