<!-- vale off -->

# Amendment AI — Same-Tick Ordering Declarations (`:prior-tick` + a complementary-guard marker)

> **Historical disposition (2026-08-23): never ratified; superseded as an
> implementation path by ADR224.** ADR224 retains deterministic sequential
> same-rank composition and activates rank-aware E-LOAD-058/E-LOAD-059 with
> exact governed disposition tables. It does not mint `:prior-tick`,
> `:complementary-writers`, or `:sequential-accumulator`; none is active BSL
> vocabulary. The draft below remains unchanged historical design evidence for
> the hazards and alternatives considered on 2026-08-18.

- **Date**: 2026-08-18
- **Status**: **HISTORICAL — NEVER RATIFIED — SUPERSEDED BY ADR224.** This text was Task
  W2.6's deliverable (`docs/superpowers/plans/2026-08-18-bsl-hygiene-
  knockout.md` §Task W2, amended by `.superpowers/sdd/2026-08-18-bsl-
  hygiene-knockout/task-w2-brief.md`'s R-W2a ruling): the Director
  approved the *amendment path* for a per-binding same-tick-ordering
  exemption at the 2026-08-18 evening sitting, over narrowing refusal 1's
  own definition instead. Drafting is workforce work; **ratification is a
  future Director sitting**, not this document.
- **Letter**: **AI** — reserved by the controller 2026-08-18 against the
  concurrently-drafted **AH** (`ai/_inbox/amendment-defevent-draft.md`,
  commit `116cfa44`, worktree `wt-p4`, R4.6 of the BSL refactor program):
  AH claimed the next-free letter after AG first. Both letters are
  re-verified at whichever sitting ratifies first; if AH ratifies before
  this draft, AI stands as drafted here — if this draft ratifies first,
  the controller's numbering still holds (AH claimed first, so AI
  cannot collide backward). No `Amendment AI` text exists anywhere in the
  tree as of this draft (checked: `grep -n "Amendment AI" CONSTITUTION.md`
  — no hits).
- **Version impact**: **MINOR** — two new, purely additive author
  declarations (a binding-level flag, a rule-level cross-reference list);
  no principle redefined or removed, no verb/intrinsic/severity-rule/
  constructor-family minted (§3 below). Precedent: Amendments AG and AH,
  both MINOR, for the same reason.
- **Amends**: NORTH_STAR §0's closed-formalism-surface sentence (a
  further named additive construct, alongside BSL/Amendment AE,
  attributed membership + lattice instances/Amendment AG, and — if it
  ratifies — declared event schemas/Amendment AH, DRAFT, not yet
  ratified as of this text); Article IX.2's amendment registry (new
  entry, inserted after whichever of AH/AI ratifies second).
- **Adds**: no new Article I–VIII principle text. `docs/reference/
  bsl-language.rst` §4.2 already carries this draft's TARGET semantics in
  prose (Task W2.3, landed ahead of ratification, explicitly marked
  "POST-RATIFICATION enforcement") — ratification authorizes a follow-up
  PR to (a) mint the two declarations' grammar, (b) teach refusal 1/2 to
  recognize them, (c) resolve §4's wealth/population open question (does
  `:complementary-writers` cover the sequential-accumulator argument too,
  or does a third construct), annotate the FULL corpus-wide set §4
  now names — not just the `consciousness` group — and re-verify the
  W2.4 audit inventory is empty against the fully-annotated corpus
  (**corrected, W2 fix round 1, review finding C1**: the original text
  here under-scoped the annotation set to one group, which would have
  left three landed packs refusing under a naive flip), and (d) flip
  `babylon_bsl::same_tick_order::ENFORCE_SAME_TICK_ORDERING` to `true`.
  **Step (d) needs no companion test-helper update** (W2 fix round 2,
  review finding NEW-1, already discharged on this branch): the crate's
  own RED fixtures and the corpus-wide audit test build their content
  sets through `rule_pipeline::split_content_unchecked` — the
  `(intrinsic …)`-split plus `E-LOAD-001` half of `split_content`,
  WITHOUT the same-tick-ordering gate — precisely so those tests stay
  gate-independent by construction and keep measuring the corpus
  correctly the moment this step flips the constant; the flip cannot
  make them start refusing their own fixtures.
  This draft does not do any of those four things (§7).
- **Supersedes**: nothing.
- **Authorizes**: the Rust/BSL implementation above, plus annotating the
  13 named rows (§4) with `:prior-tick` and the r/l/f/agitation writer
  group with the complementary-guard marker — as a future, separately-
  scoped, cargo-gated task, **not built by this draft**.
- **Source**: the W2 pre-audit table and its adjudication
  (`.superpowers/sdd/2026-08-18-bsl-hygiene-knockout/w2-preaudit-table.md`,
  `prep-adjudication.md` §(c)/(d)); the Director's R-W2a/R-W2b rulings
  (`task-w2-brief.md` §1); the load/type-refusal-vs-amendment boundary
  ruling (`docs/superpowers/plans/2026-08-18-bsl-hygiene-knockout.md:59-
  62`); issue #531 (modding expansion — the declarations are player-
  facing content vocabulary, not repo-internal escape hatches, §4 below);
  the D75 hygiene precedent against minting a synonym for an existing
  class (cited against reusing D127 for refusal 2, §2). **Recording
  ADR**: the next free number as of this draft is above ADR215 (the
  highest file in `ai/decisions/` today; the concurrently-drafted
  Amendment AH cites the same ceiling) — re-checked at ratification time
  per this repo's standing convention; the ratifying commit assigns it.

---

## 0. The problem (IX.3.3 required element)

Task W2 minted two content-set-wide load refusals, `E-LOAD-058` (stale-
default read) and `E-LOAD-059` (unreset fan-in), both computable from
*existing* declarations — no amendment needed to land the checks
themselves (the boundary ruling this train's plan records: semantic
coherence is a load/type refusal, checked entirely from what a rule
already declares). But `consciousness.bsl`, a real, working, byte-pinned
pack, ships **13 bindings** whose `:optional :default` is mechanically
"exposed" under refusal 1's literal rule (a same-content-set writer of
the read field sorts on/after the reader) and **one field group**
(`revolutionary`/`liberal`/`fascist`/`agitation`) whose two writers'
guards are individually conditional, mutually exclusive, and jointly
exhaustive — a shape refusal 2's literal rule cannot recognize.

Both are *safe by author-verified design*, not by anything the loader can
currently prove:

- The 13 rows fall into THREE safety arguments, not two — corrected in
  the W2 fix round 1 (review finding I2: the repair this same task lands
  falsifies the original two-argument split for 3 of the 13 rows).
  - A documented one-tick-lag idiom (`previous-wages`/`previous-wealth`,
    rows 18/20).
  - A default-path provably inert under the SAME rule's own guard (rows
    3-5, 9, 14-16, 22 — `consciousness/p0-position`'s and
    `consciousness/p5-agitation`'s own `r`/`l`/`f`/`agitation` reads and
    `consciousness/p3-class-solidarity-push`'s `r` — the guard that would
    observe the stale default never passes when the default is live).
    Unaffected by W2.5's repair: none of these three rules' guards
    changed.
  - **Dead bindings, not guard-inert ones (rows 6-8:
    `consciousness/p1-inbox-reset`'s `r`/`l`/`f`).** W2.5's repair
    (Director-ruled, adjudication §(d)) widened `p1`'s guard to
    `(when #t)` and left its effects as two literal
    `(update-node self … (set 0))` writes on the two inbox carriers —
    neither the guard NOR the effects reference `r`/`l`/`f`
    anymore. This is a STRONGER safety argument than guard-inertness, not
    a weaker one: whatever value each binding resolves to — the stale
    `0.0p` default, or the field's real value — can never affect this
    rule's behavior at all, unconditionally, not merely "when the guard
    happens to be false." The pre-repair argument for these three rows
    ("the guard that would observe the stale default never passes")
    is now FALSE on its own terms (the guard is `#t`; it always passes),
    and no future draft should cite it for them. The cleaner fix deletes
    these
    three now-unreferenced bindings outright, discharging their
    `E-LOAD-058` exposure by removing the read rather than annotating it
    — but that is a further pin-moving edit (it moves canonical bytes and
    fuel accounting) this draft does not make and W2's own fix round did
    not execute (Director ruling: the repair's landed shape is
    `(when #t)` only, "nothing else in the rule body"). Until a future
    commit deletes them, they remain mechanically exposed under refusal 1
    exactly as any other row here, and stay in the §4 table below with
    this corrected justification.
  Nobody has taught the loader to verify any of the three arguments; a
  human verified each once, by reading the guard/effects structure and
  the design intent (`w2-preaudit-table.md`'s own per-row notes, and — for
  rows 6-8 — the post-repair rule body directly).
- The `revolutionary`/`liberal`/`fascist`/`agitation` writer pair
  (`consciousness/p0-position` fires iff the ternary sum is exactly zero;
  `consciousness/p6-route` fires iff the sum exceeds zero) is safe
  because the two guards partition the domain, which is a **guard-
  dominance argument refusal 2 explicitly does not attempt**
  (`prep-adjudication.md` §(d): "nothing finer — no guard-dominance
  analysis... a static-analysis project W2 should not attempt").

Turning `same_tick_order::ENFORCE_SAME_TICK_ORDERING` on today would
refuse to load `consciousness.bsl` outright (W2.4's own audit,
gate forced ON: refusal 1 fires on exactly these 13 rows; refusal 2 fires
on exactly this field group, post the W2.5 repair). The Director's ruling
(R-W2a): mint a per-binding **author declaration** the loader can check
mechanically instead of re-deriving the argument itself, and narrow
refusal 1's final semantics to *every UNDECLARED exposed read*. Per the
boundary ruling this train's plan already states — "checks needing NEW
author declarations → amendment-class" — minting that declaration is
this document's job, not W2's own loader-hardening commit.

## 1. Proposed constitutional text (IX.3.3 required element)

*Registry entry — insert into IX.2 after whichever of AH/AI
ratifies second, in the house voice:*

> **Amendment AI — Same-Tick Ordering Declarations** (ratified vX.Y.Z):
> re-opens NORTH_STAR §0's formalism closure for **exactly two** additive
> author declarations, both scoped to `bsl-language.rst` §4.2's same-tick
> evaluation-order law. Operative clauses: **(i) `:prior-tick`** — a
> binding-level flag keyword, legal only on an `:optional :default`
> binding (`(binding b :field f :optional :default d :prior-tick)`),
> asserting that the author has verified this binding's exposure to a
> same-content-set writer of `f` sorting on/after the reading rule is
> SAFE — because `f` is genuinely a one-tick-lag carrier by design, or
> because the reading rule's own guard makes the default-path provably
> unreachable whenever a stale write could matter. `E-LOAD-058`'s final
> semantics: refuse every UNDECLARED exposed read; a `:prior-tick`-
> flagged binding that is mechanically exposed loads clean. The flag
> asserts a claim the loader does not itself verify (the same posture
> `EXTENSIVE_INTENSIVE_EXEMPTIONS` and the `:default` migration-corpus
> allowlist already take for their own author-attested exemptions) —
> Director/reviewer sign-off is the check, not a second static proof.
> **(ii) the complementary-writer-group marker** — a rule-level keyword
> operand, `:complementary-writers (<rule-id>+)`, naming every OTHER
> rule in the content set that writes at least one field this rule also
> writes, where the two (or more) rules' own `(when …)` guards are
> claimed by the author to be mutually exclusive over their shared
> domain. Legal only when every named rule reciprocally names this one
> back (a symmetric, closed group — checked at load, structurally, never
> by evaluating the guards themselves: this is a cross-reference
> completeness check, not a guard-dominance proof). `E-LOAD-059`'s final
> semantics: a multi-writer field whose writers ALL belong to one closed,
> mutually-declared `:complementary-writers` group loads clean without
> needing an unconditional `set` among them. **(iii) the closure re-
> seals** — this amendment adds no verb, no intrinsic, no severity rule,
> no constructor family, no adjunction, no level lattice; both
> declarations are inert metadata a rule already carrying `:optional`/
> `:when`/`:material-basis` gains a sibling keyword for, exactly as
> `deffield`'s `:enum-type` keyword (D101) added a shape without minting
> new mathematics. Windows-impact note (AA duty): none — a content-
> declaration and loader change only. Source: R-W2a/R-W2b (2026-08-18,
> evening sitting); the W2 pre-audit and its adjudication; the boundary
> ruling (`docs/superpowers/plans/2026-08-18-bsl-hygiene-knockout.
> md:59-62`). Director-ratified `<DATE>`.

## 2. Clause-by-clause rationale

**(i) `:prior-tick` as a flag, not a valued keyword.** It needs no
operand — the declaration is binary (verified-safe or not), and its
*safety reason* belongs in the binding's own surrounding
`:material-basis`/comment prose (already the convention every row in
`consciousness.bsl` follows), not in a second machine-parsed argument the
loader still would not verify. `FLAG_KEYWORDS` (`canonical_ast.rs`)
already has a closed, extensible set for exactly this shape (`:optional`,
`:invariant`, `:any`, …); `:prior-tick` joins it additively.

**Why a declaration, not a smarter check.** The two arguments that make
the 13 rows safe are irreducibly different in KIND from each other (a
naming convention for one-tick lag vs. a same-rule guard-inertness proof
for the rest) and neither is a general static-analysis result BSL's
Phase 1/2 checkers compute anywhere else. Building either prover is a
much larger, open-ended project than a one-bit author attestation the
Director/reviewer signs off on once, at declaration time — the same
tradeoff `default_lint.rs`'s allowlist and `exemptions.rs`'s
`EXTENSIVE_INTENSIVE_EXEMPTIONS` already made for their own author-
attested exemption classes.

**Why the name `:prior-tick`.** It names the OBSERVABLE consequence for
the reader (this default may, in fact, be a value the writer left on a
PRIOR tick, not this one), not the safety ARGUMENT (which varies row to
row, per the paragraph above) — matching `:optional`/`:default`'s own
convention of naming the read-time shape, not the write-time reason for
it. This is a working name only; the Director may rule a different one
at ratification (the same posture `task-w2-brief.md` §1 itself takes:
"the draft proposes, the Director ratifies the name").

**(ii) `:complementary-writers` as a rule-level, symmetric,
cross-reference list — the weaker-specified of the two mints, flagged as
such.** This draft weighed two designs:

- **A new marker (chosen).** A rule-level keyword listing sibling
  writers, checked for RECIPROCITY only (every named rule must name this
  one back) — never for guard correctness. This is honest about what the
  loader can and cannot verify: it makes the AUTHOR'S claim of mutual
  exclusivity checkable for internal CONSISTENCY (did every writer in the
  group actually get annotated, symmetrically) without pretending to
  verify the claim's substance.
- **Extending D127 (rejected).** Refusal 2 already recognizes a
  `:material-basis` citation of "D127" as an alternative discharge (the
  W2 loader implementation, `same_tick_order::cites_d127`) — extending
  that SAME citation to also cover complementary-guard safety was
  considered and rejected: D127's own registered text
  (`bsl-language.rst` D127 row) is about hash-neutral no-op writes under
  an unconditional recompute, a genuinely different safety argument from
  "two conditional writers whose guards partition the domain." Reusing
  one citation for two unrelated safety claims is exactly the hygiene
  defect D75 rules against ("minting a synonym for an existing class") —
  run in the opposite direction: collapsing two distinct classes onto one
  existing code, rather than minting a redundant new one for an existing
  class. A NEW, distinct marker keeps the two safety arguments separately
  named and separately auditable.

This clause is the draft's least-engineered part, flagged honestly in §8
for the Director's own call rather than asserted as settled.

**Normative note for the ratifying sitting (W2 fix round 2, review
finding NEW-2): the D127 discharge path is `:material-basis`-only, and
the corpus already contains a D127 citation it cannot see.**
`same_tick_order::cites_d127` reads ONLY a rule's OWN `:material-basis`
string (the reader strips comments before any `SExpr` exists, so no
other AST-visible citation surface exists). `decomposition.bsl:215,
274` and `control-ratio.bsl:254` cite "D127" inside `:material-basis` and
discharge correctly; `production.bsl:122-128` identifies
`production/p2-employed-routing`'s `(add 0)`/`(set 0)` write as "the
D127 idiom" in a **file-level HEADER COMMENT**, not inside `p2`'s own
`:material-basis` string — so the loader cannot see it, and `p2` would
not discharge via D127 even though its author already made the same
claim in prose. No classification changes today (`production/wealth`'s
byte-earliest writer is `p1`, not `p2`, so this particular citation
would not have discharged that field's finding regardless), and this
draft takes no action on it. The ratifying sitting should rule
explicitly, once, rather than let the first author whose D127 claim
lives in a header comment read a refusal as a false positive: either
state normatively in §4.2 that the discharge surface is
`:material-basis`-only (and header-comment citations do not count), or
require every future D127 citation to live in the citing rule's own
`:material-basis` string as a content-authoring convention.

## 3. What this does NOT mint

- No new verb, intrinsic, severity rule, constructor family, adjunction,
  or level-lattice construct (NORTH_STAR §0's closure — everything else
  under it stays closed).
- No new `E-LOAD`/`E-PARSE`/`E-TYPE`/`E-EVAL` code: `E-LOAD-058`/
  `E-LOAD-059` are already allocated (Task W2.2/W2.3); this amendment
  narrows their FINAL semantics, it does not mint a third or fourth code.
- No change to `:optional`/`:default`'s own grammar or semantics — a
  binding without `:prior-tick` behaves exactly as it does under W2's
  landed (pre-ratification) refusal 1.
- No change to any OTHER field's guard, write, or read law — both
  declarations are pure metadata read by the loader's same-tick-ordering
  checks alone.

## 4. What refuses at load — the two classes, and the one open question

**Class 1 — `E-LOAD-058`, post-ratification.** An `:optional :default`
binding that is mechanically exposed (a same-content-set writer of its
field sorts on/after the reading rule, self-exclusion by rule identity
per adjudication §(c)) refuses UNLESS the binding carries `:prior-tick`.
The 13 rows this draft proposes annotating (the W2.4 audit's own
inventory, verified against the landed corpus with the enforcement gate
forced ON):

| Rule | Bindings |
|---|---|
| `consciousness/p0-position` | `r`, `l`, `f` |
| `consciousness/p1-inbox-reset` | `r`, `l`, `f` |
| `consciousness/p3-class-solidarity-push` | `r` |
| `consciousness/p5-agitation` | `r`, `l`, `f`, `prev-wages`, `prev-wealth`, `agitation` |

`consciousness/p1-inbox-reset`'s row carries the DEAD-BINDING
justification (§0), not the guard-inert one every other row in this
table uses — see §0's corrected three-way split. Refusal 1's own
mechanics do not distinguish the two arguments (both are `:optional
:default` bindings mechanically exposed the same way), so the row
count stays 13, not 10; only the STATED REASON for rows 6-8 changed.

**Class 2 — `E-LOAD-059`, post-ratification.** A multi-writer field
refuses UNLESS its writer set has an earlier unconditional `set`/D127
shape (unchanged from W2's landed semantics) OR every writer belongs to
one closed, mutually-declared `:complementary-writers` group.

**SCOPE CORRECTED, W2 fix round 1 (review finding C1, Critical — the
original draft under-scoped its own annotation set).** W2.4's original
audit measured only `consciousness.bsl`/`solidarity.bsl` against
refusal 2; refusal 1 needs only those two (the only packs with any
`:optional :default` binding), but refusal 2 has NO such precondition —
it can fire on any multi-writer field in ANY pack. A corpus-wide
re-measurement (gate forced ON against all 13 landed packs solo plus
both committed co-loads —
`same_tick_order::tests::refusal_2_inventory_over_the_whole_landed_corpus`,
`rust/crates/babylon-bsl/src/same_tick_order.rs`) found **six further
fields across three packs**, none a real latent defect, but NONE
covered by the one group this draft originally proposed — flipping the
gate with only that group annotated would have refused `decomposition.bsl`,
`production.bsl`, and `territory.bsl`, three landed, byte-pinned packs,
one of them (`decomposition.bsl`) inside the committed
`decomposition+control-ratio` co-load. Full classification:
`task-w2-report.md`'s "Fix round 1" section.

*Groups this draft now proposes (four, not one):*

1. `consciousness/p0-position`, `consciousness/p6-route`,
   `consciousness/p5-agitation` — as originally drafted, jointly
   discharging `social-class/{revolutionary,liberal,fascist,agitation}`
   (`p5` needed for `agitation` alone; its own guard, anchored ∧
   positioned, is disjoint from both `p0`'s zero-sum and `p6`'s
   positive-sum guards).
2. `decomposition/p04-enforcer-intake`, `decomposition/p05-ip-intake`,
   `decomposition/p06-la-deactivate` — jointly discharging
   `social-class/{active,population,wealth}` (`p06` writes only
   `active`). Safety argument: `social-class/role` is scenario-seeded
   and never written by ANY rule in the corpus (checked: zero
   `update-node … social-class/role` sites anywhere), and each rule's
   own `fire-tick == tick` guard reads a carrier `p03-trigger` sets to
   the current tick **at most once, ever**, per its own `:material-basis`
   ("this rule's own complete gate makes its OWN re-fire idempotent") —
   so no two of the three rules can ever fire for the same subject, at
   any tick, for the life of the game. The SAME guard-partition argument
   group 1 uses, on an even simpler (single enum-equality) test.
3. `production/p1-direct-production`, `production/p2-employed-routing`,
   `production/p3-employed-fallback` — jointly discharging
   `social-class/production-value`. Safety argument: `role`
   immutability (as above) partitions `p1` from `p2`/`p3`; `p2`/`p3`
   partition each other on WAGES-in-edge existence, which is ALSO
   immutable at runtime (checked: zero `add-edge`/`remove-edge` sites on
   `EdgeType/WAGES` anywhere in the corpus) — and the pack's own header
   states the split is deliberate, not incidental: *"p2's effect ref …
   ABORTS on an employer-less subject … so employer existence must be
   split at the `when` level: p2 guards `(exists …)`, p3 guards `(not
   (exists …))`"* (`production.bsl:26-30`).

**A fourth field pair needs a DIFFERENT discharge argument this draft's
one mechanism does not yet cover — a second open question, named rather
than silently folded into group 3/4 above:**

- `production/{p1,p2,p3}` ALSO jointly write `social-class/wealth`
  (`p1`/`p3` onto `self`, `p2` onto the WAGES-payer neighbor, all via
  `add`), and `territory/p2-eviction-pipeline` +
  `territory/p4-camp-decay` jointly write `territory/population` (`p2`
  via `sub`, `p4` via `set`, both onto `self`). Neither pair's safety
  argument is guard-partition — `wealth` genuinely fans in from
  concurrent economic relations BY DESIGN (an employer legitimately
  receives wealth from two or more employees in one tick), and
  `territory/population` is DELIBERATE SEQUENTIAL COMPOSITION of one
  evolving stock, not fan-in at all (`territory.bsl`'s own header:
  *"FOUR RULES, ONE PER PHASE, BYTE-ORDERED … deliberately relying on
  D116's recorded cross-rule divergence … camp decay eats this-tick
  displaced arrivals"*). Both are **permanent, legitimately-accumulating
  state** — refusal 2's own premise (a field needs a periodic
  unconditional reset before repeat writes) does not apply to either one,
  which is a DIFFERENT safety claim from "the writers' guards partition
  the domain."
- **Option A (this draft's lean):** widen `:complementary-writers`'
  semantics to cover BOTH arguments under one mechanical shape — a
  reciprocal, symmetric, rule-level list the loader checks for
  COMPLETENESS only (every named rule names every other back), never for
  semantic correctness, exactly as clause (ii) already specifies; the
  AUTHOR states which of the two justifications applies in the rule's own
  `:material-basis`, the same way every other exemption in this corpus
  carries its reason in prose rather than in a second machine-checked
  argument. One marker, two named sub-cases, checked identically.
- **Option B:** mint a THIRD, textually distinct declaration (working
  name `:sequential-accumulator`) so a reader never has to infer which
  argument a `:complementary-writers` group is making from its
  `:material-basis` prose alone — trading one marker for a small
  vocabulary of them, mirroring how `:prior-tick` and
  `:complementary-writers` are already two separate mints rather than
  one overloaded flag covering both refusals.
- This draft does NOT choose between them — unlike the D127-reuse
  question in §2, where an EXISTING, already-meaning-bearing citation was
  the wrong vehicle for a new claim, both options here mint text this
  draft itself controls, so the choice is a naming/clarity tradeoff, not
  a hygiene violation either way. Flagged for the ratifying sitting,
  alongside the pre-existing per-field-vs-per-rule-pair question below.
  **Until resolved, `production/wealth` and `territory/population` are
  named open items, not yet assigned to any group** — the ratifying
  sitting must settle this before anyone can call the annotation set
  (and the ratification checklist's "empty inventory" promise) complete.

**Open question for the ratifying sitting, named rather than silently
picked:** should `:complementary-writers` require the SAME group (same
member set) for every field the writers jointly discharge, or may a rule
pair belong to different groups for different fields (e.g. if `p5` only
partially overlapped `p0`/`p6`'s field set)? This draft's own worked
examples (`revolutionary`/`liberal`/`fascist` at one group, `agitation`
needing a three-way group in the SAME pack; `decomposition`'s `active`
needing all three of `p04`/`p05`/`p06` while `population`/`wealth` need
only `p04`/`p05`) suggest per-FIELD groups are necessary, not
per-RULE-PAIR ones — but the grammar sketch in §1 clause (ii) names the
marker at the RULE level, which the ratifying sitting should either
confirm scopes correctly (a rule declares its writer group once, and the
group's claim is "for every field we both write, our guards are
complementary") or revise to a per-field list.

**Ratification-checklist correction (was: item (c) below, "re-verify the
W2.4 audit inventory is empty against the annotated corpus"):** that
promise now depends on resolving the wealth/population open question
above FIRST — with only groups 1-3 annotated, the post-ratification
inventory is `production/wealth` + `territory/population`, not empty.
The `**Adds**` bullet and §8 below correct item (c).

## 5. Principles affected, and how their text moves (IX.3.3 required element)

- **NORTH_STAR §0** — the closed-formalism-surface sentence gains a
  further named additive construct in its parenthetical.
- **Article IX.2** — one new registry entry (§1 above).
- **This amendment redefines no Article I–VIII principle.** `bsl-language.rst` §4.2
  already carries the TARGET normative text (Task W2.3, explicitly
  marked "POST-RATIFICATION enforcement") — ratification does not itself
  rewrite that prose; it authorizes the follow-up PR that (a) mints the
  grammar, (b) teaches the loader to recognize it, (c) annotates the 13
  rows + the writer group, and (d) flips the enforcement gate.

## 6. Draft invariance proof (IX.3.3 required element)

**Declaring neither keyword leaves a content set unaffected.**
`:prior-tick` and `:complementary-writers` are both NEW keyword
positions no existing `.bsl` file uses (grep-confirmed: `rg -n
'prior-tick|complementary-writers' rust/crates/babylon-tick/content` —
zero hits across all 13 rule files). A rule with neither keyword parses,
typechecks, and evaluates identically to before this amendment; refusal
1/2's behavior for such a rule is EXACTLY W2's own landed behavior
(unchanged) until ratification's follow-up PR flips the enforcement
gate — and even then, an undeclared rule that was SAFE-ordered/NO-WRITER
under the mechanical rule stays clean either way (the declarations only
ever WIDEN what loads, never narrow it).

**The additive re-opening adds no expressive power to the closed
algebra.** §3 above states this directly: both declarations are inert
metadata, read only by the loader's same-tick-ordering checks; they
carry no runtime evaluation semantics, no CAS byte encoding beyond the
existing flag/keyword-option machinery `FLAG_KEYWORDS`/`fixed_
positionals` already generalizes over, and no new hash surface beyond
what any other `:material-basis`/binding-option edit already produces.

## 7. What this draft does NOT do

- It does not write `bsl-language.rst`'s grammar production for either
  keyword — that lands in a follow-up spec PR once ratified, the same
  sequencing D101's own ADR195 and the AH draft (§7) both followed.
- It does not write any Rust grammar, loader-recognition, or CAS-
  encoding code for `:prior-tick`/`:complementary-writers`.
- It does not annotate `consciousness.bsl`'s 13 rows or the writer group
  named in §4 — that annotation is the follow-up pull request's own
  content change, itself a pin-relevant edit needing its own §6.5 ceremony if the
  hash moves (it does not have to: adding a keyword flag DOES change
  canonical bytes, so it WILL move the pin, and that PR owns disclosing
  it).
- It does not flip `same_tick_order::ENFORCE_SAME_TICK_ORDERING`.
- It does not resolve §4's open per-field-vs-per-rule-pair grouping
  question — this draft's one live decision for the ratifying sitting,
  named explicitly rather than silently picked (matching the AH
  precedent's own §8 item 2 posture).

All six are correctly out of scope for W2.6, whose own brief names this
document's job as "a draft document... DRAFT status explicit —
ratification is a Director sitting, not this PR."

## 8. DIRECTOR RULING REQUIRED

> **Rulings recorded (Director popup sequence, 2026-08-18 ~21:15 EDT) —
> four of this draft's open questions are RULED; fold them into §1's text
> before the ratifying sitting:**
> 1. **Accumulator coverage (§4's Option A/B): Option B** — mint
>    `:sequential-accumulator` as a THIRD, textually distinct declaration
>    (against this draft's own Option-A lean): a reader never infers the
>    safety claim from `:material-basis` prose; `production/wealth` and
>    `territory/population` are its first two rows.
> 2. **Group granularity: per-FIELD groups** — `:complementary-writers`
>    names its field(s); a rule pair may belong to different groups for
>    different fields, as this draft's own worked examples showed
>    necessary.
> 3. **The D127 registry: `:material-basis` only** — header comments carry
>    no normative weight; `production.bsl:122-128`'s header citation MOVES
>    into p2's `:material-basis` as a one-line content fix riding the
>    ratification ceremony.
> 4. **Row 37 (the solidarity off-simplex write window): refusal-1
>    (E-LOAD-058) IS the tripwire** — no content change now; a composed
>    consciousness+solidarity load refuses by design, forcing the explicit
>    redesign-or-annotate decision at composition time; the audit row's
>    DIRECTOR-GATE marker stays.
>
> Remaining for the ratifying sitting proper: items 1-2 below (the §1
> text + the `:prior-tick` name) and the ceremony itself.

1. **Ratify or revise the proposed text in §1** (letter AI — or the
   letter Amendment AH already claims, if the controller's reservation
   needs revisiting at the ratifying sitting — MINOR version bump).
2. **Ratify or rename `:prior-tick`** (§2's naming rationale; the brief's
   own text treats the name as provisional).
3. **Resolve §2/§4's `:complementary-writers` design**: the new-marker
   shape this draft proposes, the rejected D127-extension alternative, or
   a third design the Director prefers — and, if the new marker, whether
   it groups per field or per rule-pair (§4's open question).
4. **Resolve §4's second open question (W2 fix round 1, review finding
   C1)**: does `:complementary-writers` also cover the sequential-
   accumulator argument (`production/wealth`, `territory/population`),
   or does a third, textually distinct construct (working name
   `:sequential-accumulator`) — §4's Option A/B.
5. **Confirm or revise the §4 table** — the 13 `:prior-tick` rows (10 on
   the original guard-inert/lag-idiom argument, 3 on the corrected
   dead-binding argument) and the FOUR `:complementary-writers` groups
   (`consciousness` p0/p5/p6; `decomposition` p04/p05/p06;
   `production` p1/p2/p3 for `production-value`; plus whatever ruling 4
   resolves for `production/wealth` and `territory/population`) — as the
   intended annotation scope for the follow-up implementing PR.
   `same_tick_order::tests::refusal_2_inventory_over_the_whole_landed_
   corpus` measures the corpus-wide inventory this scope targets.
6. **Assign the recording ADR number** at ratification time (header;
   next free above ADR215 as of this draft).

## 9. Drafting notes (not part of the proposed text)

- This draft follows the `ai/_inbox/amendment-*-draft.md` convention this
  repository already uses for a Director-gated, not-yet-ratified
  amendment text — the same convention `amendment-v3-refoundation-
  draft.md` (Amendment AE) and `amendment-defevent-draft.md` (Amendment
  AH, DRAFT, this same sitting's sibling document) both follow, rather
  than `ai/decisions/`. Per that same convention (confirmed against the
  AH draft's own §9, which found "no drafted-but-unratified amendment in
  this repo's history has an `ai/decisions/*.yaml` ADR filed for it
  before ratification"), **no ADR stub accompanies this draft.** The
  ratifying commit creates the recording ADR when this amendment
  ratifies.
- **Letter reservation, verbatim per the controller's mid-task
  instruction (2026-08-18):** "Letter AI reserved by controller
  2026-08-18 against the concurrently-drafted AH (defevent); both
  letters re-verified at the ratifying sitting."
- This draft checks every factual claim about current grammar/loader/
  audit state directly against source on this branch
  (`feature/bsl-hygiene-knockout`, `wt-hygiene`) on 2026-08-18: the W2
  pre-audit table and its adjudication (both fully read before this
  draft's own writing); `rust/crates/babylon-bsl/src/same_tick_order.rs`
  (this task's own implementation, landed in an earlier commit on this
  branch — `cites_d127`, the `WriteOp`/`is_unconditional` shapes, the
  self-exclusion logic); `rust/crates/babylon-bsl/src/canonical_ast.rs`
  (`FLAG_KEYWORDS`, `fixed_positionals`); `rust/crates/babylon-tick/
  content/rules/consciousness.bsl` (fully read; the §4 table's row/
  field names come straight from the actual file, not from the audit
  table's own citations, and cross-check against the passing
  `refusal_1_fires_on_exactly_the_13_exposed_bindings_of_consciousness_
  bsl` test); and `docs/reference/bsl-language.rst` §4.2 (Task W2.3's own
  landed prose on this same branch, an earlier commit).
