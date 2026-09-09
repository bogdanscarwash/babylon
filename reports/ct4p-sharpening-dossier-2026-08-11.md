# CT4P Sharpening Dossier — Milewski's *Category Theory for Programmers* against the Babylon engine

*Synthesis of ten reader sweeps, verified against `dev` working tree, 2026-08-11. Read-only pass:
no code changed, no cargo run. Every claim about our code carries a `file:line` anchor checked by
hand. `docs/reference/bsl-language.rst` is cited by **section number only** — this pass read the
Rust and the architecture standard, not the 5,916-line spec, so no line anchors are offered for it.*

---

## Executive summary

1. Ten readers returned 60 suggestions; **6 survive as engineering, 7 as vocabulary, 5 as seeds**.
2. The largest single finding is a **correction**, not an addition: four readers independently proposed
   "sum/count are monoid homomorphisms — pin partition-invariance." **That law is false here.** `fold sum`
   reduces binary64 left-to-right in element order (`evaluator.rs:1046-1055`); IEEE-754 `+` is not
   associative, and the spec pins the order precisely so reassociation cannot happen. The honest law is the
   *negative* one, and it is untested. → **A2**.
3. The highest-value item is **A1**: `tick.rs:396-412` states in its own doc that the collect-phase
   pre-state discipline is *"a convention … not the compiler"*. CT names why (Reader vs State) and the
   repair is a signature change, not new math.
4. `PendingWrite` is **not** a Writer monoid the way six readers assumed — `apply_pending_write` reads the
   target's *current* value at apply time (`structural_verbs.rs:732-760`), so the batch is a free monoid but
   application is a **non-commutative monoid action**. → **B1**, a strictly sharper statement than any reader's.
5. Roughly a third of the reader suggestions targeted structures that **do not exist in `rust/`** —
   scale adjunctions, lattice rungs, attributed-membership payloads. Discarded with evidence; see §Honesty.
6. **B6** closes a registered open question: the standard's §4.1 records as a negative finding that
   "algebraic theories in the technical Lawvere-theory sense are never invoked" (`ai/bsl-architecture-standard.md:709`,
   OQ-12) — Ch.30 supplies the naming at zero formalism cost.
7. No suggestion below introduces nondeterminism, unbounded recursion, or an imposed functional form.
8. **Infrastructure note that shapes every item:** the workspace has no property-testing framework
   (`rust/crates/babylon-bsl/Cargo.toml` dev-deps = `pretty_assertions` only). Every "law test" below is
   specified as a **deterministic table-driven test**, which suits a determinism contract better than
   randomized generation anyway.
9. Effort: A1/A2/A4/A5/A6 are S; A3 is M. Bucket B is documentation only.
10. Bucket C is **seeds**, never recommendations — Amendment AE clause (ii) closes the algebra (`CONSTITUTION.md:681`).

---

## Bucket A — engineering, no new mathematics

Ranked by concrete value to this project.

### A1 · Make the collect phase's pre-state immutability type-level, not conventional — **S**

**CT concept.** Ch.21 §21.2.3 vs §21.2.5 (pp. 308-311): a *Reader* threads an unchanging environment;
a *State* threads a changing one. The distinction is carried by the type, not by discipline.

**Babylon seam.** `rust/crates/babylon-bsl/src/tick.rs:440-513` — `run_tick` holds
`graph: &mut dyn GraphSubstrate` across both passes. Its own doc, corrected in the #519 fix round,
states the gap outright (`tick.rs:396-412`):

> *"NLL re-acquires a `&mut` reborrow per subject (the verifier compiled a mutation that wrote to `graph`
> mid-Pass-1 and it built cleanly), so nothing at the TYPE level stops a future Pass-1 caller from mutating
> between subjects. … That Pass 1's *loop* never calls a mutating method between subjects is a convention —
> enforced by this module's own pre-state tests … not by the compiler."*

What *is* type-level is one call deep: `collect_effects` takes no `&mut` graph
(`structural_verbs.rs:553-563`). The loop around it does not inherit that.

**Concrete change.** Extract Pass 1 into a free function taking an immutable substrate:

```rust
fn collect_pass(graph: &dyn GraphSubstrate, /* … */) -> Result<Vec<PendingWrite>, TickError>
```

`run_tick` then reborrows `&*graph` once for the whole collect pass and `&mut *graph` for the apply loop
(`tick.rs:510-520`). The borrow checker now enforces the §4.2 chapter-C4 pre-state law for the *loop*, not
just for one callee, and the two existing convention-tests
(`all_firings_of_one_rule_observe_the_same_pre_state`, `tick.rs:710`) become redundancy rather than the
only defence.

**Why it earns its keep.** The pre-state law feeds the byte-deterministic tick hash
(`rust/crates/babylon-graph/src/state_hash.rs:1-54`). A gap the module documents about itself, closable by a signature
change, is the cheapest determinism win on this list.

---

### A2 · Pin `fold sum`'s reduction order with a non-associativity witness — **S**

**CT concept.** Ch.13 §13.1-13.2 (pp. 214-219), monoid homomorphism `h(a*b) = h a * h b` — and the
prerequisite the readers skipped: the operation must actually be associative.

**Babylon seam.** `rust/crates/babylon-bsl/src/evaluator.rs:1020-1055` — `fold_sum` accumulates
`apply_arith("+", &prev, &body_val)` over the materialized element slice, strictly left-to-right in
ascending-id order (`query.rs:296-317`). `fold_mean` carries the matching discipline in a comment
(`evaluator.rs:1114-1122`, D-row Q5): *"both sums reduced in ITERATION order — sequential accumulation into
a local, never a reordering fold"*, with an explicit note that no FMA contraction is in force.

**Correction to four readers.** `sum(A ∪ B) == sum(A) + sum(B)` is **false** in the binary64 lane. Partition
invariance is exactly what the spec forbids, because it would change bytes. The three-decade classic
`1e16 + 1.0 + 1.0` reassociates to a different double.

**Concrete change.** A deterministic table test in `evaluator.rs`'s test module:

- build an element set whose body values make `+` visibly non-associative;
- assert the fold's result equals the **left fold in ascending-id order**;
- assert it does **not** equal the reassociated / chunked value;
- mirror it for `fold_mean`'s `sum_wx`/`sum_w` accumulation, which today has the prose (Q5) and no test.

**Why it earns its keep.** "Chunk the fold to fit the fuel budget" and "parallelise the fold" are both
plausible future PRs. Today nothing fails when they land. This is the guard, and it is ~30 lines.

---

### A3 · Mint a `FoldOp` enum so the fold-op × kind legality table is exhaustive by construction — **M**

**CT concept.** Ch.9 §9.5.4 (p. 150), `a^(b+c) = a^b × a^c` — a total function out of a sum type is exactly
one arm per injection, never a wildcard.

**Babylon seam.** The closed 5-op set is a `&str` array in one place and re-dispatched by string in four
others, every one of them with a catch-all arm:

| site | form |
|---|---|
| `grammar.rs:335` | `const FOLD_OPS: [&str; 5] = ["sum", "mean", "min", "max", "count"];` |
| `typecheck.rs:87-142` | `match op { "sum" => … "count" => …, other => Err(unknown aggregation operator) }` |
| `evaluator.rs:812-857` | `match op.as_str() { … other => Err("unknown fold-op …") }` |
| `score_class.rs:198` | `"sum" \| "mean" \| "min" \| "max" => …` |
| `rule_pipeline.rs:585` | `matches!(op.as_str(), "sum" \| "mean" \| "min" \| "max")` |

`FieldKind` *is* a real 2-variant enum (`types.rs:34-40`), so the legality table is a 5 × 2 = 10-row total
function — small enough to enumerate exhaustively and large enough to get wrong quietly.

**Concrete change.** Introduce `enum FoldOp { Sum, Mean, Min, Max, Count }` parsed once at the grammar
boundary; convert the five dispatch sites to exhaustive `match` with no wildcard; add a table test walking
all 10 `(FoldOp, FieldKind)` pairs and asserting each carries an explicit accept/refuse verdict.

**Why it earns its keep.** Adding a sixth fold op today is five string edits and **zero** compile errors —
the exact silent-widening shape S-22's closed-vocabulary invariant exists to prevent
(`ai/bsl-architecture-standard.md:648`). Effort is M only because five call sites move; each move is mechanical.

**Corrects reader-9's version**, which proposed treating the kind law as "a preorder over the seven kinds."
There are two kinds, not seven, and the relation is not a preorder — it is a legality table.

---

### A4 · Pin the `min`/`max` semilattice laws the dedup representation already leans on — **S**

**CT concept.** Ch.13 (p. 214) — `min`/`max` are the idempotent, commutative, associative case; Ch.3 §3.4
(pp. 30-31) on commutativity being a *separate*, optional property.

**Babylon seam.** `evaluator.rs:1131-1168` — `fold_min_max` replaces the incumbent only on **strict**
improvement, so ties keep the first element in §2.6 order. Non-finites are already excluded
(`EvalCode::NonFinite`, `evaluator.rs`; `state_hash.rs:44-52` refuses them a second time), so over the live
domain `min`/`max` genuinely are associative, commutative, and idempotent — unlike `sum` (A2).

**Concrete change.** Two assertions in the same table test as A2: the same multiset in two element orders
folds to the same `min`/`max`; a duplicated element changes nothing.

**Why it earns its keep.** It is the one fold family where reordering *is* safe, and stating that is what
keeps A2's negative law from being over-generalised into "never touch any fold." Pairing the two tests in
one module makes the asymmetry the point.

---

### A5 · Pin `Element`'s total order before slices 2 and 3 mint new variants — **S**

**CT concept.** Ch.3 §3.3 (pp. 28-29): *"sorting algorithms … can only work correctly on total orders"*;
a derived order is a choice, not a specification.

**Babylon seam.** `rust/crates/babylon-bsl/src/query.rs:37-42`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Element { Node(NodeId) }
```

One variant today, so the derive is harmless. `query.rs:16-20` records that `Edge(EdgeKey)` lands with
slice 2 and `Hyperedge(HyperedgeId)` with slice 3. The moment a second variant lands, `#[derive(Ord)]`
silently becomes **variant-declaration-order lexicographic** — a cross-kind total order that no spec section
pins, feeding a sort whose output feeds the tick hash.

**Concrete change.** Today: a comment on the derive naming §2.6 as the authority plus a one-line test
asserting the current single-variant ordering matches ascending `NodeId`. At slice 2: a test pinning the
cross-kind order against the spec's declared order *before* the variant is added, so the derive can never
become the specification by default.

**Why it earns its keep.** This is a determinism bug that does not exist yet and costs almost nothing to
foreclose. Catching it at the cheapest moment is the whole argument.

---

### A6 · Pin the dual of "option order is a formatting concern" — **S**

**CT concept.** Ch.3 §3.4 (pp. 30-31): concatenation is a monoid *because* it does not require
commutativity — reordering is never free by default.

**Babylon seam.** `canonical_ast.rs:262-283` sorts **keyword options only** (`options.sort_by`, `:275`);
positional operands and the variadic body keep source order. `evaluator.rs:620-624` short-circuits `and`/`or`
left-to-right, and `evaluator.rs:2875-2930` already proves short-circuit charges strictly less fuel when the
deciding element is early. So `(and a b)` and `(and b a)` agree on **value** and disagree on **fuel** — and
fuel is a load-time admissibility criterion (`E-LOAD-040`, invariant S-3 at `ai/bsl-architecture-standard.md:629`).

The positive test exists (`option_order_is_a_formatting_concern`, `canonical_ast.rs:417-435`). Its **dual**
does not.

**Concrete change.** One test asserting `(and a b)` and `(and b a)` produce **different** canonical bytes,
with a docstring saying why: canonicalisation stops at keyword options precisely because positional order
carries short-circuit and fuel semantics, so a future "normalise boolean operands" optimisation is a
`rules_hash` change and a fuel change, not a formatting edit.

**Reader-10's larger version is discarded**: there is no Boolean simplifier or query optimiser in the crate,
so distributivity and double-negation-elimination tests would exercise nothing. This is the surviving,
load-bearing sliver.

---

## Bucket B — vocabulary: name what we already built

Documentation only. No code change in any item.

### B1 · The write batch is a free monoid; **application is a non-commutative monoid action**

**CT concept.** Ch.4 §4.1-4.3 (pp. 42-49) Writer/Kleisli; Ch.13 §13.1 (p. 214) free monoid; Ch.22 §22.2
(pp. 331-332) monoid in a monoidal category.

**Babylon seam.** `structural_verbs.rs:99-133` (`PendingWrite`), `:723-760` (`apply_pending_write`),
`tick.rs:453,510-511,519-520` (`all_pending.extend` then a single apply loop).

**What six readers got wrong, and the sharper truth.** They proposed "collect-then-apply is a Writer monoid;
pin associativity of `mappend` and the empty batch as identity." Half right. The *collection* half is a free
monoid on writes — list concatenation, order- and multiplicity-bearing. But **application is not a fold in
that monoid**: `apply_pending_write` reads the target's *current* value at apply time for `Add`/`Sub`/`Scale`
(`structural_verbs.rs:83-97`, D-row Q2), because — in the code's own words — *"reading the target at collect
time would make three subjects each adding to one carrier lose two of the three contributions."*

So the batch acts on state as a sequence of **endomorphisms**, composed left-to-right. `Add` and `Scale` do
not commute. Reordering a batch changes the result even when the batch itself is unchanged.

**Documentation to write.** In `structural_verbs.rs`'s `PendingWrite` doc and `ai/bsl-architecture-standard.md`:
*the collected batch is the free monoid on writes (associative concatenation, empty batch as unit); its
application is a monoid **action** on graph state, a left fold of endomorphisms whose non-commutativity is
load-bearing.* Then state the two things a future optimiser may and may not do: **may** re-chunk collection
(concatenation is associative); **may not** reorder application (the action is not commutative).

**Why it earns its keep.** "Monoid" alone is exactly the word that would license a batch reordering for cache
locality. Naming the action, and naming which half is commutative-free and which is not, is what makes the
vocabulary protective rather than decorative.

---

### B2 · Name the open rule-to-rule divergence (D-row Q14) as a Reader/State confusion

**Babylon seam.** `rust/crates/babylon-tick/src/lib.rs:272-289` — `run_once_into` runs each rule to
**completion** (collect *and* apply) against one `&mut G` before the next rule starts. Its own comment
records this as a divergence, not a design:

> *"a later rule sees an EARLIER rule's writes from the SAME tick … it is NOT what §4.2 demands: 'rules within
> one system position observe the same pre-state' … This is a divergence to fix in its own train, not a design
> feature 'inherited for free'."*

`tick.rs:57-69` records the same gap from the other side.

**Documentation to write.** One sentence on the D-row Q14 entry: *within one rule the engine is Reader-shaped
over a shared pre-state (Task 12's repair); across rules at one system position it is still State-shaped, and
the repair is precisely the promotion of the outer loop from State to Reader.* That sentence is also the
acceptance criterion for the repair train.

**Why it earns its keep.** The gap is already recorded but described operationally ("runs to completion
before the next rule starts"), which reads like an implementation detail. Named as a variance in the shape of
the effect, it reads like the defect it is — and it tells the repairer what "done" means.

---

### B3 · Two accumulation structures, deliberately different — semilattice on the query side, monoid on the write side

**CT concept.** Ch.21 §21.2.2 (pp. 305-307) list/set nondeterminism; Ch.13 §13.1 (p. 285) free monoid is
neither idempotent nor commutative.

**Babylon seam.** Query results: `substrate.rs:151-181` makes ascending, deduplicated iteration a **contract**
(*"a set, so `:any` never yields a node twice"*), proved for every implementation by
`conformance.rs:230` (`nodes_edges_neighbors_hold_contractual_order_and_dedup`) and `:446`
(`declared_order_never_leaks_through_any_ranged_accessor`). Write batches: `tick.rs:453,510-511` — a flat
`Vec` in subject-outer, source-inner order, duplicates meaningful.

**Documentation to write.** In `ai/bsl-architecture-standard.md` §2 or the §2.6 spec gloss: *a query result is
the free finite join-semilattice on element identity, realised as a canonical sorted-deduplicated form (order
and multiplicity are not data); a write batch is the free monoid on writes (order and multiplicity **are**
data). The two are different structures on purpose, and that is the whole answer to "why is this one sorted and
deduped and that one not."*

Optional half-sentence gloss, from Ch.31 §31.1 (p. 401): the substrate carries **two** morphism-like citizens
over one object class — tight dyadic edges and loose payload-bearing hyperedge memberships — the shape a
double category names. `substrate.rs:4-18` already argues this at length under "two typed halves"; the name is
free.

---

### B4 · `mean` is the quotient of two extensive monoids — say so where the KIND LAW is stated

**CT concept.** Ch.3 §3.4 (pp. 29-31); Ch.19 (pp. 285-288) free-monoid canonical form; Ch.25 §25.1 (p. 374)
"certainly fulfilled when `(a, f, z)` is a monoid."

**Babylon seam.** The KIND LAW lives at `typecheck.rs:1-19` and `:87-142`, with codes `E-TYPE-041/042/043`
(`typecheck.rs:38-55`). The implementation already does the right thing: `fold_mean`
(`evaluator.rs:1074-1129`) carries `(sum_wx, sum_w)` and divides exactly once at the end.

**Documentation to write.** Next to the KIND LAW: *extensive quantities close under an associative combine
with an honest identity — they are monoids. Intensive quantities are not; a mean exists only as the quotient
of two extensive monoids, `Σ(w·x) / Σw`. An unweighted mean of an intensive field has discarded the denominator
monoid, which is why `E-TYPE-042` refuses it. The litmus test for any future fold: does it close under an
associative combine with an honest identity, or is it secretly a quotient of two things that do?*

**Why it earns its keep.** The project's own memory carries "intensive aggregation = variance error" as a Key
Pattern and `types.rs:1-8` explains the field-vs-scalar split, but the *reason the refusal is principled rather
than stylistic* is nowhere written. This is one sentence, citable, and it hands the KIND LAW's maintainers a
decision procedure for the next fold anyone proposes.

---

### B5 · Amendment AG's "kinds closed, instances mintable" is the theory/model split of a Lawvere theory

**CT concept.** Ch.30 §30.1-30.3 (pp. 444-451): a Lawvere theory `L` is the signature plus its laws; a
**model** is a product-preserving functor `L → Set`. Ch.19 (p. 282): a free functor is the left adjoint to a
forgetful one.

**Babylon seam.** `CONSTITUTION.md:685` (Amendment AG clause ii) permits content to *"DECLARE new scale-lattice
rungs and `allocate`/`aggregate` adjunction INSTANCES of the existing adjunction schema, load-time validated;
minting a new adjunction KIND … stays closed under AE (ii)."* `CONSTITUTION.md:681` (AE clause ii) is the
closure: *"BSL expresses the existing closed algebra and mints no new mathematics."*

**Documentation to write.** In the ADR189 gloss and `ai/bsl-architecture-standard.md` §3: *the closed algebra is
a (many-sorted) algebraic theory presentation — sorts are the BSL types, operations are the fixed query / fold /
effect combinators, laws are the intensivity kind law plus the collect-then-apply ordering. Every content pack is
a **model** of that theory. "Kinds closed, instances mintable" is exactly the theory/model boundary: the
signature never moves; `Mod(theory)` is as open as content authors need. An adjunction KIND is a fixed
`L ⊣ R` pair; an adjunction INSTANCE is `L(generator)` for a new generator — applying an existing free functor,
never defining a new one.*

**Why it earns its keep — this closes a registered open question.** `ai/bsl-architecture-standard.md:709`
(OQ-12) records as a negative finding that *"algebraic theories in the technical Lawvere-theory sense are never
invoked"* despite the project's Lawverian self-description, and `:699` (OQ-7c) records that *how AE clause (ii)'s
'BSL expresses the existing closed algebra' cashes out is undocumented*. Ch.30 answers both at zero formalism cost.

**Honest caveat to carry into the write-up.** This is a **naming claim**, not a proof: nobody has exhibited the
product-preservation obligations for BSL's operations. Record it as the vocabulary OQ-12 asked for, explicitly
not as a discharge of OQ-7c's derivation obligation.

---

### B6 · BSL's semantic category is FinSet-shaped, not Hask-shaped — there is no ⊥

**CT concept.** Ch.2 §2.3 (pp. 15-17): Haskell types form `Hask`, not `Set`, precisely because ⊥ exists —
a function may fail to terminate.

**Babylon seam.** Folds are the only iteration construct; no recursion, no `while`, no user-defined functions
(`ai/bsl-architecture-standard.md:186-194`, quoting `bsl-language.rst` §4 / §3.7); `bound(rule) > :fuel` is
rejected at **load** (invariant S-3, `:629`); the AST is a finite `SExpr` tree (`reader.rs`).

**Documentation to write.** One paragraph in the standard: *because evaluation is fuel-bounded and iteration is
syntactically bounded, BSL functions are total on their declared domain in the strict sense — there is no analogue
of Haskell's ⊥, because non-termination is structurally excluded rather than avoided by discipline. BSL's semantic
category is closer to FinSet than to Hask. The consequence is the reason S-11 exists: with no ⊥ to stand in for
"didn't finish," every refusal must be an explicit typed return value with an E-code, never a silent default.*

**Why it earns its keep.** Cheap onboarding, and it forecloses a specific wrong intuition a Haskell-literate
contributor imports (laziness, silent divergence). It also ties "refusal is loud and load-time" to a reason rather
than a mandate. Note the argument is **already made structurally** in the standard §2.4; this only supplies the name.

---

### B7 · `unitDefect`'s metric is a Lawvere-enriched distance — and its asymmetry is a feature *(lowest priority)*

**CT concept.** Ch.28 §28.5 (pp. 429-430): metric spaces as categories enriched over `([0,∞], ≥, +, 0)`, due to
William Lawvere — zero self-distance and the triangle inequality come free from the enrichment; **symmetry is
explicitly not assumed**.

**Babylon seam.** `ai/bsl-architecture-standard.md:426-432` (OQ-11) already records the finding:
*"Lawvere-metric enrichment is structurally present but never named. `unitDefect`'s `d :: p -> p -> Intensity` is
exactly a metric-style map feeding an adjunction defect — the shape of Lawvere's 1973 `[0,∞]`-enriched-category
framing — but the term never appears."*

**Documentation to write.** Name it on the OQ-11 row, scoped correctly to `d` (the metric feeding `unitDefect`),
**not** to `w` — the standard's OQ-2b (`:691`) records that the Haskell draft conflates `w` (principal-aspect
weight, `[-1,1]`) with the `unitDefect`-minted gap `g` (`[0,1]`), and repeating that conflation would import a
known erratum. Add the one clause the enrichment buys us: **asymmetry is not a bug** — Lawvere's own uphill/downhill
reading — so nobody later "fixes" an intentionally asymmetric tension.

**Ranked last, honestly.** `unitDefect` lives only in `ai/BabylonCoreDraft_2.hs`, a self-declared unratified draft
with no Rust implementation. This is a note on an open question, not a change to shipping code.

---

## Bucket C — AMENDMENT-GATED SEEDS

**None of the following is a recommendation. None may be implemented.** Amendment AE clause (ii)
(`CONSTITUTION.md:681`) binds: *"BSL expresses the existing closed algebra and mints no new mathematics — no new
generator, no new constructor family (C/G/P stand), no new adjunction, no new level lattice, no new severity
rule. Everything else stays closed; new formalism still costs an amendment."* Reinforced by NORTH_STAR §0
(`NORTH_STAR.md:14-22`): *"we are rigorous enough … the rigor budget is henceforth spent wiring existing
mathematics to the player, not minting more mathematics."* Each seed is recorded so that if the need ever
arrives, the amendment proposal points at a known-sound construction instead of re-deriving one under pressure —
and so that reaching for it piecemeal is recognisable as the amendment it would be.

### C1 · Rules as an internal-hom object; rule combinators as 2-morphisms
*Ch.9 §9.1 (pp. 137-141) universal construction of `a ⇒ b`; Ch.10 §10.3-10.4 (pp. 168-177) functor category and
2-categories.* Treating a BSL rule as a genuine exponential `subject ⇒ PendingWrite`, closed under `eval`, would let
content mint higher-order rules — rule combinators, rules parameterised by rules, meta-rules rewriting rule sets —
with vertical/horizontal composition and an interchange law as the soundness obligations. The design pull is real
(line-struggle reshaping *which* rules apply to an organisation, not just its coefficients). This mints a
constructor family; AE (ii) forbids it absent an amendment.

### C2 · A general limit/colimit primitive over declared finite diagrams
*Ch.12 §12.1-12.3 (pp. 189-208).* Every construction BSL uses today — folds (products), predicate selection
(equalizers), joins (pullbacks) — is one instance of one general universal-cone construction. A BSL primitive
letting content declare an arbitrary finite diagram shape and compute its universal cone would replace the
hand-enumerated fold/query vocabulary with one uniform mechanism. Recorded **precisely because** this is the shape
of generalisation the closed-algebra clause exists to keep out: it would arrive fold-by-fold, never announcing
itself as the amendment it is. The same page range's pullback-as-unification result (pp. 205-207) belongs here as a
sub-seed — replacing today's closed-enumeration kind checks (`typecheck.rs:87-142`) with real type *inference* is
a formalism-surface change, not a typechecker patch.

### C3 · A Kan-extension generator for scale adjunctions
*Ch.27 §27.2, §27.4 (pp. 407-414): `Ran_K I ⊣ K ⊣ Lan_K I` — both adjoints of an embedding computed mechanically
from one end/coend formula.* If hand-authoring one scale-adjunction instance per rung pair (hex↔community,
county↔territory, …) under Amendment AG clause (ii) ever becomes costly, this is the textbook generator. Two
riders for the record: (a) any real formulation needs an explicit fuel bound, since ends and coends are
limit/colimit constructions and every BSL construct must be **statically** bounded (invariant S-3,
`ai/bsl-architecture-standard.md:629`); (b) a generic derivation procedure is new engine machinery, categorically
distinct from AG's licensed *instances*.

### C4 · An intuitionistic classifying object for the epistemic layer
*Ch.29 §29.3 (p. 442): a general topos has intuitionistic internal logic; excluded middle fails.* If the
epistemic/fog layer is ever given a formal predicate calculus rather than ad hoc narration, the principled move is
its **own** classifying object `Ω_epistemic`, distinct from the engine's Boolean one — "I know X", "I know not-X",
"I have no proof either way" naturally fails excluded middle. Its virtue over the likely alternative is that it
respects the standing no-imposed-functional-forms law (`NORTH_STAR.md:26-27`): a different classifying object is
not a stipulated curve, unlike an ad hoc confidence float. Doubly gated: Amendment V / II.8 keeps fog out of the
tick hash, and EpistemicHorizon is a frozen-Python-side system with no Rust existence today.

### C5 · A Store-comonad interface over the static adjacency estate
*Ch.23 §23.4-23.6 (pp. 344-351): `extend` recomputes a value at every position from a read-only view of its
neighbours — the book's own worked example is Conway's Life.* If diffusion-style mechanics (tension, solidarity,
shock propagation across ADJACENCY) ever accumulate as several hand-written "read neighbours, compute new value"
loops, a Store-comonad interface over the invariant per-resolution lookup estate is the law-checked way to unify
them (`extract ∘ duplicate = id` and friends as the obligations). New machinery, not naming: `duplicate` over a
graph substrate is nontrivial to define correctly. **No such loops exist in `rust/` today** — this is a seed
against a pain point that has not appeared.

---

## Coverage and honesty notes

### Suggestions discarded on a verified-false premise

| Reader claim | Verdict |
|---|---|
| "Seven storable kinds form a coproduct; assert the runtime variant count is exactly 7" | **FALSE.** `BslType` has **nine** variants (`types.rs:12-31`); `Value` has more still (`evaluator.rs:53-83`, incl. `Ratio` per ADR194). The substrate stores **`f64` only** — `node_attribute(…) -> Result<f64, …>` (`substrate.rs:133-142`), and `substrate.rs:24-28` records that typed attribute storage (Currency's i128 exactness) is a **declared Phase-2 gap** with Currency writes refused loudly. There is no 7-kind runtime tag to count. |
| "Const vs Identity functor split for typechecker-only kinds" | Same false premise. `NodeSet`/`EdgeSet` are `BslType` variants with no storable form, but the erasure story described does not exist in the code. |
| "Bifunctor `bimap` laws for the attributed `(member, hyperedge)` pair"; "product factorizer uniqueness for attributed membership"; "profunctor `dimap` laws for payload transport across lattice rungs" | **Premise not yet real.** Amendment AG clause (i) is ratified (`CONSTITUTION.md:685`), but `GraphSubstrate::add_hyperedge(hyperedge_type, members)` (`substrate.rs:196-200`) carries **no payload**, and `members_of` returns a bare `Vec<NodeId>` (`:205-212`). No attributed-membership object exists in `babylon-graph`. Recorded as a design note for whoever lands AG (i), not as a test to write. |
| "Triangle identities for scale adjunctions"; "two routes to the same scale transition must agree by adjoint uniqueness"; "lattice bottom/top = initial/terminal object" | **No such code exists.** A case-insensitive search for `adjunction\|adjoint\|lattice rung` across `rust/crates/` returns zero hits. Survives only as B5's documentation of the kinds/instances split; the triangle-identity checklist belongs in the AG (ii) *landing plan*, not in a test today. |
| "Kind law as a preorder over the seven kinds — pin reflexivity and transitivity" | **Wrong shape.** `FieldKind` has two variants (`types.rs:34-40`); the law is a 5 × 2 legality table (`typecheck.rs:87-142`), not an order relation. Reshaped into **A3**. |
| "sum/count are monoid homomorphisms — pin partition invariance" (four readers) | **False in the binary64 lane.** Inverted into **A2**. |
| "Boolean-topos distributivity and double-negation-elimination tests" | No Boolean simplifier or query optimiser exists to break them. Reduced to **A6**'s narrow CAS guard. |
| "Lens laws (GetSet/SetGet/SetSet) for `to_graph`/`from_graph`" | That accessor pair is in the **frozen Python** engine, reference-only since the `p27-python-freeze` pin. No Rust analogue exists. Not actionable. |

### Suggestions discarded as already satisfied

- **Empty-fold units must be individually justified.** Already done, per operator, loudly: `sum` over an empty
  set consults a static additive-identity classifier and **refuses by name** when the identity is not statically
  determinable (`evaluator.rs:1029-1044`, D-row Q12); `mean` is `E-EVAL-021` (`:1069-1073`); `min`/`max` likewise
  (`:1140-1148`); `count` is cardinality (`:1010-1018`).
- **CAS round-trip in both directions.** Already done: `the_encoding_is_self_delimiting`
  (`canonical_ast.rs:672-681`) decodes with an independent test-only decoder and requires byte-identical re-encode.
- **Materialization is sorted, deduplicated, and order-invariant.** Already contracted and tested:
  `substrate.rs:151-181` (contract), `query.rs:296-317` (50-node ascending-order test with recorded mutation
  evidence), `conformance.rs:230,339,368,446` (a reusable law suite parameterised over *any* substrate
  implementation — `run_substrate_conformance`, `conformance.rs:28`). The only residue is **A5**.
- **Catamorphism / structural-induction termination as a second line of defence.** The standard already makes
  this argument (`ai/bsl-architecture-standard.md:186-194`: *"totality is syntactic"*). No action.
- **Curry-Howard naming for load-time refusal**, **Kleisli naming for `?`** — true, free, and adding nothing over
  the existing S-3 / S-11 statements. Dropped as below the quality bar.

### Which page ranges yielded what, honestly

- **Ch. 1-3, 4, 8, 13, 19-22, 30** carried the load: orders, monoids, free monoids, Writer/Kleisli, Lawvere
  theories. These are the chapters *about structures we already have*, so they yielded engineering and vocabulary.
- **Ch. 9-10, 12, 26-27, 29, 31** yielded almost nothing but seeds — exponentials, naturality, limits/colimits,
  ends/coends, Kan extensions, topoi, bicategories. This is **plausible and expected**, not a failure of the
  readers: those chapters are about *constructing new universal objects*, which is precisely what AE (ii) forbids.
  A closed-algebra codebase should get seeds from them and nothing else.
- **Ch. 23-25** (comonads, F-algebras, lenses) yielded mostly *already-satisfied* items. Also plausible: Babylon
  arrived independently at canonical forms, structural termination, and round-trip tests for determinism reasons,
  and the CT names for them are retrospective.
- **No reader range came back empty**, and no range should have.

### One structural gap worth naming

The workspace has **no property-testing framework** — `rust/crates/babylon-bsl/Cargo.toml` dev-dependencies are
`pretty_assertions` alone, and the same holds workspace-wide. Every item above is therefore specified as a
deterministic table-driven test. That is not a workaround: a determinism contract is better served by a fixed
witness table (reproducible, byte-pinnable, ceremony-compatible) than by randomized generation, and
`conformance.rs`'s parameterised suite is already the idiomatic home for laws that must hold for every
implementation. If a future train wants generative testing, adding `proptest` is its own decision with its own
seed-determinism obligations, and none of the six A-items needs it.
