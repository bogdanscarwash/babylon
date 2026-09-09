# Amendment AE — The Refoundation (BSL + the Rust Kernel)

- **Date**: 2026-07-29
- **Status**: **DRAFT — DIRECTOR GATE.** Not ratified. Phase 1 of Program 27
  does not begin until this text is ratified (spec §10).
- **Version impact**: **MAJOR** — v2.18.0 → **v3.0.0**. Redefinition of a
  principle's binding (II.6's engine), plus a *removal power* over ratified
  formal constructs (clause (iii)) — both MAJOR triggers under IX.1.
- **Letter**: **AE** (A–Y, AA, AC, AD are registered; Z is
  ratified-but-unregistered in the IX.2 list — see drafting note D4; AB is
  reserved for the drafted Material-Triad amendment,
  `ai/_inbox/material-triad-program-brief.md` W2).
- **Amends**: II.3 (substrate binding), II.6 (State is Data, Engine is
  Transformation), II.12 (Matrix Representation Layer), III.12(a) (canonical
  serialization scope), Article IX registry.
- **Adds**: no new principle text in Articles I–VIII; one registry entry (IX.2)
  and the rider mechanism it carries.
- **Supersedes**: NORTH_STAR §6.2 (engine-language sentence), NORTH_STAR §0
  (formalism-closure sentence, additively and subtractively), NORTH_STAR §6.3 +
  invariant 8 (the finish line retargets), ADR063's Rust deferral, the
  2026-07-22 crate-extraction ruling *for engine crates*, and Amendment AC
  clause (iv)'s packaging consequence *at cutover*.
- **Authorizes**: Program 27 Phases 1–4
  (`docs/superpowers/specs/2026-07-28-program-27-refoundation-design.md`).
- **Source**: Director rulings R1–R9, 2026-07-28 → 2026-07-29, recorded in spec
  §3. Recording ADR: the Program 27 block; the next free number at drafting time
  is **ADR172** (P26 consumed ADR160–169; ADR170–171 are taken) — the ratifying
  commit assigns it.

---

## 0. The problem (IX.3.3 required element)

Babylon's rules live as Python code. Three consequences follow, and all three are
now load-bearing:

1. **The trusted surface is the whole language.** ~200k lines of Python semantics
   must be trusted for a tick to mean what it says. There is no smaller thing to
   verify.
2. **The rule substrate is fragmented.** Three disjoint mini-grammars already
   exist — the doctrine trap-condition string DSL
   (`src/babylon/domain/doctrine/mechanics.py`), the flat Pydantic
   event-precondition tree (`src/babylon/engine/event_evaluator.py`), and the
   4-op effect enum — each with its own failure semantics, and four
   silent-degradation defects under III.11 in the event evaluator alone (spec §5,
   grammar-superset honesty).
3. **The sentinel estate is a compiler front-end with no language.** 84 files /
   20,642 lines across 24 families read Python source via `ast` to enforce
   grammar conformance, closed vocabulary, and literal-freeness — the checks a
   typechecker performs for free in a language designed for the job. IX.5 names
   that estate as the license for agent autonomy; today it is hand-rolled.

The Director's ruling (R1) is that the driver is **BSL-as-portable-core plus
one-toolchain convergence**, not a measured CPU wall: performance is a suspicion
to be converted into evidence (Phase 0 publishes the per-system tick profile),
never an assumed motive. The engine is rewritten **kernel-first in Rust** (R4)
around BSL. This amendment is the governance half of that program; it authorizes
nothing that is not in the approved spec.

---

## 1. Proposed constitutional text (IX.3.3 required element)

*Registry entry, to be inserted in IX.2 after Amendment AD, in the house voice.*

> **Amendment AE — The Refoundation (BSL + the Rust Kernel)** (ratified v3.0.0):
> Rebinds the **engine language** and re-opens the formalism surface for exactly
> one additive construct and one class of subtractive rulings. Operative
> clauses: **(i) engine language** — **Rust is the engine language**; Python
> survives as the **data-build pipeline** (the parquet → sha-pinned reference-DB
> estate, ADR098), the **out-of-process AI observer** (with the vault baker), and
> the engine-decoupled CLI periphery; this supersedes NORTH_STAR §6.2's "Python
> stays the engine language" and moves II.6's implementation binding (the frozen
> Pydantic `World`, `tick(world, actions) → (new_world, events)`) onto the Rust
> kernel **exactly as Amendment L moved II.3's substrate binding** — II.6's
> principle (state is pure data; the engine is pure transformation; they never
> mix) is language-independent, and only the binding changes. **(ii) formalism
> closure, additive** — NORTH_STAR §0's closed formalism surface re-opens for
> **exactly one** additive construct, the **Babylon Scripting Language (BSL)**:
> a total, fuel-metered, homoiconic s-expression language whose rules are
> content data. BSL **expresses** the existing closed algebra and **mints no new
> mathematics** — no new generator, no new constructor family (C/G/P stand), no
> new adjunction, no new level lattice, no new severity rule. Everything else
> stays closed; new formalism still costs an amendment. **(iii) formalism
> closure, subtractive** — the same closure re-opens **subtractively** for
> III.10 Earn-Its-Keep retirements arising out of the numeric-annex per-site
> rulings: a retirement is **not sign-off-only** — *each III.10 retirement
> enacted under §6.2 rulings is recorded as a rider to this amendment
> enumerating the retired construct* — since removing a construct changes what
> the algebra can express. **(iv) Invariant 2 rebound** — "the engine
> adjudicates; AI narrates; clients render — no exceptions without amendment"
> survives **verbatim**, with "engine" rebound to the Rust kernel; II.5
> (narrator-only) and II.8/Amendment V (the `observe()` client contract) are
> untouched, and moving the AI observer out of process strengthens the
> separation rather than relaxing it. **(v) ADR063 superseded** — ADR063's
> deferral of a Rust kernel behind a measured national-scale CPU profile is
> superseded by Director ruling R2; the tick profile is still published in
> Phase 0 as **evidence hygiene, not a gate**. ADR063's two surviving
> conclusions are **retained**: the monorepo argument (one determinism contract
> spans the sim — strengthened when engine and client share a language) and its
> characterization of a language port as a **re-baselining project** under
> III.12(b), which R3's hybrid correctness bar and R8's stream-change ruling
> accept explicitly rather than deny. **(vi) Amendment D** — II.7's
> `[TRANSITION STATE]` hyperedge reconciliation is resolved **inside this
> program**, by a Phase-0 analysis PR the Director ratifies **before**
> `babylon-graph` commits a data shape (R7); per IX.3.4 it is **not** resolved
> by engineering default, and the analysis MUST state how VIII.9 (community as
> pairwise edge) and I.18's material/ideological distinction fare under each of
> II.7's three options. Until that ratification, `babylon-graph`'s data shape is
> blocked; the dyadic `StableDiGraph` working assumption commits nothing.
> **(vii) Windows (Amendment AA duty)** — Rust + cargo **improves**
> native-Windows feasibility versus the Nix-pinned CPython/numpy stack; residual
> foreclosure risks are the game-managed embedded Postgres cluster (X.8/D1) and
> whatever LAPACK linkage, if any, the per-site numeric-annex rulings retain —
> each retained linkage is itself a foreclosure entry. AA's **Shield** is
> unchanged in letter and now covers a longer pre-1.0 period by operation of
> clause (ix). **(viii) the Python engine freezes** — at the end of Phase 0 the
> Python engine is frozen at an **executable pin** (the freeze tag: source +
> `flake.lock` rev + `uv.lock` + reference-DB sha + Postgres migration head),
> and a scheduled CI job rebuilds and runs the tagged engine on the 11 canon
> scenarios through cutover — **failure of that job is a red gate**. After the
> tag the Python engine is reference-only; a mid-program fix to the frozen
> branch requires **Director sign-off plus contract re-extraction**. **(ix) v1.0
> retargets** — **v1.0 is redefined as the Rust engine's release**; NORTH_STAR
> invariant 8 ("the game ships") and §6.3's forcing function retarget onto it,
> save-compat semver resets with it (`docs/versioning.md`), and every in-flight
> v1.0 stop receives an explicit disposition — closed-as-superseded, absorbed,
> or carried (client-side stops carry, since the client survives under clause
> (iv)). **(x) repo layout** — engine crates land in the existing **in-tree**
> `rust/` cargo workspace alongside `babylon-tui`/`babylon-md`, extending
> Amendment AC clause (i) from client crates to engine crates and superseding
> the 2026-07-22 extraction ruling for them; `hypergraph-rs` remains a **sibling
> library consumed as a dependency and its charter does not expand**.
> **Continuity condition (IX.5):** the sentinel estate is the license for agent
> autonomy and MUST NOT lapse — the per-family disposition table (subsumed /
> ported / git-level) is a **hard cutover gate**, and a family whose port slips
> either blocks cutover or degrades to a declared, Director-signed exemption
> row; there is no silent lapse. Windows-impact note (AA duty (i)): recorded in
> clause (vii). Source: Director rulings R1–R9 (2026-07-28 → 2026-07-29,
> in-session), design `docs/superpowers/specs/2026-07-28-program-27-refoundation-design.md`
> §4; recording ADR172. Director-ratified 2026-07-__.

---

## 2. Clause-by-clause rationale (one clause per spec §4 item)

| Clause | Spec §4 item | Rationale |
|---|---|---|
| **(i)** engine language | §4.1 | Spec §1 + R1: the trusted surface shrinks from ~200k lines of Python semantics to one spec'd evaluator plus a named intrinsic table; toolchain convergence with the client (Amendment AC) is the second driver. Python's surviving jobs are enumerated so "Python survives" cannot erode into "Python still adjudicates": data-build (spec §7), AI observer (spec §7), CLI periphery verified zero-engine-coupling (spec §6.5). |
| **(ii)** additive re-opening | §4.2 first half | Spec §5: BSL is a *language*, not mathematics. Per the Director's framing (spec §1), THE_FORMALISM.md Part III is already a language spec — BNF, typing judgments, totality theorems — with no language behind it; §0's closure exists to stop new mathematics, and BSL adds none. The "exactly one" quantifier is what keeps §0 a closure rather than a formality. |
| **(iii)** subtractive re-opening + rider | §4.2 second half | Spec §6.2: the per-site numeric-annex rulings may retire constructs under III.10 (the Ollivier-Ricci curvature LP is the leading candidate — a degenerate LP has multiple optima, so it is *behavioral* drift, not float noise). A retirement changes what the algebra can express, which is a MAJOR-class act; the rider makes each one visible in the constitutional record instead of buried in an ADR. |
| **(iv)** Invariant 2 rebound | §4.3 | Spec §1 + §7: the client survives unchanged in contract and gains an in-process engine; the AI observer moves out of process. Rebinding "engine" without touching the sentence is the minimum change that keeps II.5/II.8/Amendment V intact. |
| **(v)** ADR063 superseded | §4.4 | Spec §2: ADR063 deferred Rust behind a CPU-profile trigger. R1/R2 replace the trigger with a different driver, so the deferral falls — but ADR063 was *right* about the monorepo and about re-baselining, and the amendment says so rather than quietly dropping it. Publishing the profile anyway is evidence hygiene: the instrumentation has existed at `simulation_engine.py:139-157` and its output has never been published. |
| **(vi)** Amendment D | §4.5 | Spec §6 crate table + R7: `babylon-graph`'s data shape IS the II.7 question in executable form. IX.3.4 forbids implementing against a `[TRANSITION STATE]` principle, so shipping the working assumption without ratification would be an engineering default resolving a constitutional question — exactly what IX.3.4 exists to prevent. |
| **(vii)** Windows note | §4.6 | Amendment AA duty (i) is mandatory for any ADR adopting a load-bearing platform or language; this amendment adopts both. The honest form names the two residual foreclosures instead of claiming a clean win. |
| **(viii)** freeze tag | §4 "Python engine disposition" | Spec §10 + §11 (frozen-Python drift row): a source-only tag rots. The executable pin plus the scheduled job is what makes the Phase-4 parallel-run ceremony's Python half known-runnable when it is needed, and it makes drift loud (III.11) rather than discovered at cutover. |
| **(ix)** v1.0 retargets | §4 "v1.0 is redefined" (R2) | Spec §1 + §10: R2 supersedes the v1.0 critical path. Naming the retarget in the constitution is what prevents two live definitions of "v1.0" — and it forces the disposition table for in-flight stops rather than letting them decay. |
| **(x)** repo layout | §4 "Repo layout" | Spec §6: ADR063's one-determinism-contract argument is *strengthened* when engine and client share a language, so engine crates go in-tree next to the client crates AC already placed there. The explicit non-expansion of `hypergraph-rs`'s charter keeps the AC (vi) raster arrangement (rev-pinned git dependency, ratatui never in its dependency graph) intact. |

---

## 3. The rider mechanism

**Verbatim, as it appears in clause (iii):**

> each III.10 retirement enacted under §6.2 rulings is recorded as a rider to
> this amendment enumerating the retired construct

**Disambiguation (drafting note D1 — for the Director to accept or reject).**
Inside `CONSTITUTION.md` a bare "§6.2" is ambiguous (Article VI item 2, "Zoom
Where Data Exists", is a Scope Control clause; CONTRIBUTORS.md §6.5 is the
ceremony law; NORTH_STAR §6.2 is the sentence clause (i) supersedes — three live
referents). The intended referent is the
**Program 27 spec's §6.2, the numeric annex**. Recommended ratified form appends
a parenthetical without changing the sentence: *"…enacted under §6.2 rulings
(the Program 27 numeric annex, spec §6.2)…"*. If the Director prefers the
sentence untouched, the disambiguation lives in the SYNC IMPACT REPORT instead.

**Filing a rider.** A rider is a bullet appended to this registry entry, in the
same commit as the retirement, carrying: (a) the retired construct by name and
call site; (b) the III.10 clause it failed (law / prediction / running
computation); (c) what replaces it, or the written statement that nothing does
and what the engine loses; (d) the ADR that records the per-site ruling; (e) the
Director sign-off marker. Riders are PATCH-class registry edits against v3.x —
they record an act the amendment already authorized. A retirement of a construct
**not** reachable from the §6.2 numeric-annex audit is out of scope and needs its
own amendment.

**Rider ledger:** *(empty at ratification)*

---

## 4. Principles affected, and how their text moves (IX.3.3 required element)

| Principle | Change class | Delta |
|---|---|---|
| **II.6 State is Data, Engine is Transformation** | binding change | "frozen Pydantic `World` model" → the Rust kernel's world value; `tick(world, actions) → (new_world, events)` unchanged as a signature-level commitment. The Trinity sentence is unchanged: Postgres runtime, rustworkx→`babylon-graph` topology, pgvector Archive, read-only SQLite reference fixture. "No DB I/O during tick" unchanged. |
| **II.3 Graph as Discretized Manifold** | binding change, follows (i) | "Implementation binding: rustworkx (Amendment L)" → the `babylon-graph` crate over the rustworkx-core/petgraph re-export (petgraph **via re-export only** — the hypergraph-rs lesson). The manifold commitment is library-independent (Amendment L's own words); the `[TRANSITION STATE — Amendment D]` marker stays until clause (vi) discharges it. |
| **II.12 Matrix Representation Layer** | binding change | "scipy.sparse is the computation layer" → **faer** as the default numeric layer, with same-LAPACK linkage available per site **only** where a Phase-0 ruling records that faer cannot reproduce the required decomposition. The three-layer separability law (authoring → sparse matrix → operator expression) and "the operator algebra is the source of truth" are unchanged. |
| **III.12(a) canonical serialization** | scope extension | The language-agnostic reference gains: the tick hash, the BSL canonical AST serialization, `ContentDigest`, the per-AST-node fuel cost model, and RNG seeding. Corollary (a)'s `[IMPLEMENTED]` status is **not** revoked; the existing `docs/reference/determinism-contract.rst` is extended, not replaced. |
| **III.10 Earn-Its-Keep** | operative power | Unchanged in text; clause (iii) makes it the instrument of a bounded removal power and attaches the rider duty to its exercise. |
| **III.7 Determinism and Replayability** | **not relaxed** — see §6.1 row 15 | Every tick still produces a deterministic hash; non-determinism is still a bug. Cross-implementation replay is *already* excluded by III.12(b) ("byte-identical replay is guaranteed only within a single implementation and libm"). R8's stream change is therefore a re-baselining event, and clause (viii)'s executable freeze pin is the mechanism that keeps pre-cutover logs replayable — **against the implementation that recorded them**. |
| **IX.2 amendment registry** | addition | This entry. |
| **III.9 tiers, Articles I, IV, V, VII, VIII, X** | untouched | No theoretical commitment, test case, verb, visual principle, anti-pattern, or deployment principle changes. |

---

## 5. Draft invariance proof (IX.3.3 required element)

IX.2 requires that affected principles be **at least as constrained** as their
predecessors. Three arguments, one per direction of change, then two procedural
checks (§5.4, §5.5).

### 5.1 The engine rebinding is strictly more constrained

Every constraint the Python engine carried is preserved, and BSL adds
constraints Python could not express:

| Constraint | Python engine | Rust + BSL |
|---|---|---|
| Bounded loops (Power-of-10 Rule 2) | convention + review | **static property**: no general recursion; folds over finite graph-query results only; worst-case fuel bound computed at content load against declared cardinality ceilings; over-budget rules **rejected at load** |
| Closed vocabulary | `ast`-reading sentinels | closed enums in the type system; new members remain amendment territory |
| Unweighted intensive aggregation | sentinel family, post hoc | **type error** at typecheck, with a declared exemption ledger |
| Silent degradation (III.11) | 4 known silent defaults in the event evaluator | all four **deliberately broken** as III.11 corrections, with a documented conformance delta |
| Absent bindings | `.get(x, 0.0)` fallbacks | **load-time error**; only `(:optional … :default …)` bindings may be absent, enumerated in the migration corpus, new ones need Director sign-off |
| Aleksandrov (III.8) | review-only | `:material-basis` **mandatory at parse**, presence/non-emptiness enforced mechanically; the semantic obligation stays with Director review and the sentinel successor — scoped honestly, not overclaimed |
| Numeric determinism | BLAS=1 pin + convention | IEEE-754 basic ops and fixed-point integer only in the evaluator; transcendentals are named intrinsics with pinned implementations; `f64::round` lint-banned in engine crates |

No constraint is dropped. The estate that *enforced* constraints (the sentinel
families) is preserved by the clause-level continuity condition: cutover blocks
on the disposition table being green.

### 5.2 The additive re-opening adds no expressive power to the algebra

§0's closure protects the **algebra**: one generator 𝔇 = (A, Ā, w, T, σ), three
constructor families C/G/P, the level lattices, the production adjunctions, the
derived severity rule, the boundary operator ∂L. BSL introduces none of these and
cannot: its value types are the existing kernel scalars and closed enums; its
effects are the existing 4-op arithmetic set plus a **typed structural verb set**
already exercised by the shipped systems today (5 of 39 system modules by the
strict add/remove-verb grep, 29 when `update_node` payload writes count — spec
§5, on the Phase-0-reproducible AST definition); formula composition is over
**registered** intrinsics only; defining a new intrinsic is **not expressible**.
BSL is therefore a *notation* for the closed algebra. The one honest expansion —
BSL's condition set is a superset of the two existing grammars' expressible sets
— is an expansion of the **rule** surface, not the algebra, and it is bounded by
the closed vocabulary.

### 5.3 The subtractive re-opening is bounded and visible

Three bounds: (a) **scope** — only constructs reachable from the numeric
dependency closure of spec §6.2 (`run_tick`-reachable numpy/scipy sites);
(b) **test** —
III.10 as already ratified (name the law, the prediction, or the running
computation); (c) **visibility** — the rider duty means a retirement cannot be
enacted without appearing in the constitutional record with its name attached.
A construct that fails III.10 was never earning its constitutional rent; removing
it *increases* the fraction of the formal surface that satisfies III.10. The
constraint that would be relaxed by an unbounded version of this clause —
"ratified constructs are permanent" — is not a ratified constraint; III.10 has
always been a live test.

### 5.4 IX.2 staged-series check

**No Article-I primitive changes.** The dialectic 𝔇 is untouched (I.19); the
partition remains derived (II.1/Amendment A); ValueTensor4x3 remains derived. A
staged amendment series under IX.2 is therefore **not triggered**. MAJOR is
claimed on the IX.1 versioning definition's *other* limb — redefinition of a
principle (II.6's engine binding) plus the bounded removal power of clause
(iii) — not on primitive removal.

### 5.5 IX.1 compliance triggers tripped

**Scope expansion** (Program 27 supersedes the v1.0 critical path),
**infrastructure/deployment change** (the toolchain and the X.8 distribution
closure gain a cargo half; the maturin/PyO3 boundary retires at cutover), and
**primitive redefinition** in the weak sense of clause (i)'s binding move. New
system: none — Program 27 R2 **forbids new feature trains**; Phase 0 is
contracts, evidence, and one authorized bug fix. Formula change: none at
ratification; any arising from clause (iii) files a rider. Data source addition:
none.

---

## 6. IX.3.3 cross-check — every re-opened ruling, named

*A re-opening this amendment forgets is a future constitutional bug. This is the
complete list; it is the auditable half of the draft.*

### 6.1 Re-opened, superseded, or retargeted

| # | Ruling / text | Where it lives | Disposition | Clause |
|---|---|---|---|---|
| 1 | **"Python stays the engine language"** | NORTH_STAR §6.2, governor 2 (Personal familiarity) | **Superseded.** The governor itself survives — Rust is named in the same sentence as "the sanctioned compiled lane", and the Director's hands are Python/Rust, so the 2 a.m.-maintainability test still passes. Only the instance changes. | (i) |
| 2 | **"The formalism surface is closed for v1.0"** | NORTH_STAR §0 | **Re-opened additively**, for exactly one construct (BSL). | (ii) |
| 3 | **The same closure, in the removal direction** | NORTH_STAR §0 + III.10 | **Re-opened subtractively**, bounded to the spec §6.2 numeric annex, with the rider duty. | (iii) |
| 4 | **"we are rigorous enough" / "the rigor budget is spent wiring existing mathematics to the player"** | NORTH_STAR §0 | **Reaffirmed, not re-opened** — BSL mints no mathematics; but the *budget* sentence is strained by a rewrite, and the Director should rule knowingly that Program 27 is the exception R2 already made. **Flagged, not assumed.** | (ii), D2 |
| 5 | **ADR063's Rust deferral** ("no measured CPU wall exists…triggered by evidence, not vibes") | `ai/decisions/ADR063_program14_correspondence.yaml` | **Superseded by R2**; its monorepo and re-baselining conclusions retained. Status flips accepted → superseded-in-part. | (v) |
| 6 | **NORTH_STAR invariant 8 — "The game ships"** | NORTH_STAR §8 | **Retargeted**: the shipped artifact is the Rust engine's v1.0. The invariant does not move; its referent does. | (ix) |
| 7 | **"The v1.0.0 Definition of Done is the only finish line; Gate 3 is the only gate that matters between here and there"** | NORTH_STAR §6.3, governor 3 | **Retargeted** with #6. Gate 3 (#262) is one of the in-flight stops the disposition table must place. | (ix) |
| 8 | **NORTH_STAR invariant 2** | NORTH_STAR §8 | **Rebound, verbatim** — "engine" now denotes the Rust kernel. | (iv) |
| 9 | **Amendment D's pending status** (II.7 `[TRANSITION STATE]`) | CONSTITUTION IX.2, II.3, II.7 | **Scheduled for resolution inside this program**, by Director ratification of the Phase-0 analysis, before `babylon-graph` commits a shape. Not resolved by this amendment. | (vi) |
| 10 | **The 2026-07-22 crate-extraction ruling** (generic Rust libraries live in sibling repos) | Amendment AC (i) narrative | **Superseded for engine crates**, as AC already superseded it for client crates. Generic libraries (`hypergraph-rs`) still live outside. | (x) |
| 11 | **Amendment AC (iv) packaging consequence** ("the maturin wheel joins the default install and the T7 uv2nix player closure at M7") | CONSTITUTION IX.2, Amendment AC | **Superseded at cutover**: `babylon-tui-python` (the PyO3 cdylib) retires when the client links the engine in-process; the player closure gains cargo-built binaries instead. Until cutover, AC (iv) stands as written. | (i), (x) |
| 12 | **Amendment L's implementation binding** (rustworkx) | CONSTITUTION II.3, IX.2 | **Follows the language rebinding** to `babylon-graph` over the rustworkx-core/petgraph re-export. Amendment L's own determinism rider (insertion-ordered iteration surfaces; baselines regenerate only with written proof of an unavoidable order shift) is **carried forward verbatim** into the port. | (i) |
| 13 | **II.12's scipy.sparse computation layer** | CONSTITUTION II.12 | **Rebound to faer**, per-site LAPACK retention only by recorded Phase-0 ruling. | §4 table |
| 14 | **The 2026-07-28 three-log Director directive** (`babylon.log` / `rust-client.log` / `client-capture.log`) | Director directive, CLAUDE.md | **Proposed for supersession** at cutover (spec §6.5 logging row): the game process collapses to one JSONL sink; `client-capture.log` retires with the PyO3 boundary. **This is spec open ruling #7 and is NOT carried by this amendment** — listed here so the interaction is not lost. | none (D3) |
| 15 | **R8: RNG streams change at cutover** | spec §3, §8.5 | Not a constitutional re-opening (III.12(b) already excludes cross-implementation byte-identity), but it *does* mean pre-cutover action logs replay only against the frozen engine. Named so the interaction with III.7 is on the record. | (viii) |

### 6.2 Extended, but not re-opened

- **III.12 / Amendment Q (behavioral contracts).** Extended in scope by
  clause-adjacent Phase-0 work (III.12(a) gains the BSL AST, fuel model, tick
  hash, `ContentDigest`, RNG seeding). The rewrite test is not weakened — R3's
  hybrid bar (evaluator hard-proof + game law-fidelity + the 11 canon scenarios)
  is what the artifacts are validated against, and the spec states plainly that
  ensemble envelopes are *weaker* than stream-compatible comparison rather than
  claiming parity.
- **Amendment AA (Windows).** Duty (i) is discharged by clause (vii). The Shield
  is unchanged in letter — and, by operation of clause (ix), covers a longer
  calendar period. No pre-1.0 Windows work is authorized by this amendment.
- **Amendment AD / IX.5 (Agentic Engineering).** Not re-opened. Its autonomy
  license depends on the gates, so the amendment carries the sentinel-continuity
  condition as an operative clause-level duty rather than a footnote.
- **CONTRIBUTORS.md §6.5 baseline ceremonies.** Survive as git-level tooling
  (the sentinel disposition table's class (c)); the cutover ceremony extends the
  drift-table format rather than replacing the law.

### 6.3 Explicitly NOT re-opened (the closure ledger)

Named so that ratifying this amendment cannot be read as opening them:

- **NORTH_STAR §0's destination sentence** — "a terminal-based, first-class,
  keyboard-driven simulation video game, installed by one shell script, in the
  hands of the masses." **Reaffirmed.** The client survives; only what runs
  underneath it changes.
- **Article I in full** — every MLM-TW theoretical commitment, the Fundamental
  Theorem, the Survival Calculus, the bifurcation law, I.2a's σ spectrum
  (Amendment N), I.18's material/ideological distinction. Article I is the
  Director's reserved line (IX.5); this amendment does not touch it.
- **II.5 / Amendment V** — narrator-only AI, no LLM in the input path.
- **II.8 / Amendment V** — the `observe()` client contract; clients stay
  disposable presentation layers.
- **III.13 / Amendment W** — deterministic materialization and the golden vault;
  the vault baker stays Python (spec §7).
- **VIII.9** — the oppressor-hyperedge anti-pattern; it is a *constraint on*
  clause (vi)'s analysis, never a casualty of it.
- **Amendment T** — the divergence channel stays observes-only with its
  implementation queued; the port carries the queue, not a promotion.
- **Amendment AC clauses (ii), (iii), (v), (vi)** — the client contract, the
  tutorial-BDD parity gate, the 3D lane charter, and the hypergraph-rs raster
  arrangement all stand; clause (x) explicitly declines to expand hypergraph-rs's
  charter.
- **Article X** — the production estate, the Metropole's observes-never-
  adjudicates clause, and X.7's single-pinning-authority law (the flake still
  pins the toolchain; cargo/rustup enter *through* it, never beside it).
- **Amendment B** — still pending; unaffected.

### 6.4 IX.3.4 transition-state interaction

II.7 is `[TRANSITION STATE]`. IX.3.4 forbids implementing code that depends on an
unresolved principle. Clause (vi) is the only lawful route: the analysis PR is a
*spec*, which IX.3.4 explicitly permits, and the Director's ratification is what
unblocks `babylon-graph`. Any Phase-1 work that would commit a graph data shape
before that ratification is a IX.3.4 violation regardless of how green its gates
are.

---

## 7. Draft SYNC IMPACT REPORT block (paste-ready)

```text
Version Change: 2.18.0 → 3.0.0 (2026-07-__)
Bump Rationale: MAJOR — Amendment AE (The Refoundation: BSL + the Rust Kernel)
  registered, Director-ratified (rulings R1–R9, 2026-07-28 → 2026-07-29).
  Rebinds the ENGINE LANGUAGE to Rust (Python survives as data-build pipeline,
  out-of-process AI observer, and engine-decoupled CLI periphery), superseding
  NORTH_STAR §6.2. Re-opens NORTH_STAR §0's formalism closure ADDITIVELY for
  exactly one construct — the Babylon Scripting Language (BSL), which expresses
  the closed algebra and mints no new mathematics — and SUBTRACTIVELY for III.10
  retirements out of the Program 27 numeric annex, each recorded as a RIDER to
  this amendment enumerating the retired construct. Invariant 2 survives
  verbatim with "engine" rebound. ADR063's Rust deferral superseded (its
  monorepo + re-baselining conclusions retained; the tick profile is published
  as evidence hygiene, not a gate). Amendment D scheduled for resolution inside
  the program, before babylon-graph commits a data shape (IX.3.4). Amendment AA
  Windows note recorded. The Python engine freezes at an executable pin with a
  scheduled runnable-through-cutover CI job. v1.0 retargets to the Rust engine's
  release (invariant 8 + §6.3 forcing function follow; save-compat semver
  resets). Engine crates land in the in-tree rust/ workspace (extends Amendment
  AC (i); supersedes the 2026-07-22 extraction ruling for engine crates;
  hypergraph-rs's charter does not expand). Sentinel-estate continuity is a hard
  cutover gate (IX.5's autonomy license). MAJOR: a principle's binding is
  redefined (II.6) and a bounded removal power over ratified constructs is
  created (clause iii). No Article-I primitive changes, so no IX.2 staged series
  is triggered.

Modified Principles:
  - II.3  — implementation binding rustworkx → babylon-graph (rustworkx-core/
            petgraph re-export ONLY); [TRANSITION STATE — Amendment D] preserved
  - II.6  — engine binding: frozen Pydantic World / Python tick → Rust kernel;
            Trinity + "no DB I/O during tick" unchanged
  - II.12 — computation layer scipy.sparse → faer; per-site LAPACK retention
            only by recorded Phase-0 ruling; three-layer separability unchanged
  - III.12(a) — reference scope extended (tick hash, BSL canonical AST
            serialization, ContentDigest, fuel cost model, RNG seeding);
            [IMPLEMENTED] status retained, the document is extended not replaced

Added Sections:
  - IX.2 Amendment AE — The Refoundation (BSL + the Rust Kernel)

Artifacts Requiring Update (IX.1 step 3):
  - NORTH_STAR.md §0, §6.2, §6.3, §7 (the road), §8 invariant 8
  - CLAUDE.md / AGENTS.md (Constitutional Compact block, engine section)
  - docs/versioning.md (save-compat semver reset)
  - ai/architecture.yaml, ai/state.yaml, ai/wiring-doctrine.md
  - ai/decisions/ADR063 status → superseded-in-part; index.yaml entry
  - CONTRIBUTORS.md (§6.5 unchanged; cutover-ceremony extension noted)
  - project/roadmap.md, GitHub Project 8 (the in-flight-stop disposition table)

Follow-up TODOs:
  - AMENDMENT D: Phase-0 analysis PR → Director ratification (blocks
    babylon-graph; IX.3.4)
  - RIDERS: each III.10 numeric-annex retirement files one against this entry
  - AMENDMENT Z: registry entry missing from IX.2 (ratified v2.14.0, recorded
    only in the sync-impact history and ADR102) — housekeeping, PATCH class
```

---

## 8. DIRECTOR RULING REQUIRED

This draft is not ratified and authorizes nothing. Ratification requires the
Director to rule on each of the following:

1. **Ratify / amend / decline** the clause set (i)–(x) as drafted, at v3.0.0.
2. **Letter assignment: AE** — confirm (AB stays reserved for the Material
   Triad; Z's missing registry entry is separate housekeeping).
3. **Rider disambiguation (D1)** — append "(the Program 27 numeric annex, spec
   §6.2)" to clause (iii), or keep the sentence verbatim and disambiguate in the
   SYNC IMPACT REPORT only.
4. **The rigor-budget question (D2, §6.1 row 4)** — confirm knowingly that
   Program 27 is the exception to NORTH_STAR §0's "the rigor budget is spent
   wiring existing mathematics to the player", since a rewrite spends it
   elsewhere for the duration.
5. **The log-estate interaction (D3, §6.1 row 14)** — confirm whether the
   cutover supersession of the 2026-07-28 three-log directive belongs in this
   amendment or stays an in-program ruling (spec §13 item 7). Drafted as the
   latter.
6. **Clause (ix) scope** — confirm that retargeting v1.0 resets save-compat
   semver and that in-flight client-side stops carry rather than close.

---

## 9. Drafting notes

- **D1** — rider-reference ambiguity; see §3.
- **D2** — the rigor-budget strain; see §6.1 row 4 and ruling 4 above.
- **D3** — the log-estate directive is deliberately left out of the clause set;
  see §6.1 row 14 and ruling 5 above.
- **D4** — **Amendment Z is ratified (v2.14.0, Environment Sovereignty, ADR102)
  but has no IX.2 registry entry** — it appears only in the sync-impact history
  and in X.7's revision note. Found while assigning this amendment's letter.
  Housekeeping, PATCH class, out of scope here; recorded so it is not lost.
- **D5** — `THE_FORMALISM.md` (cited by spec §1 and echoed in §2 above) is **not
  present in this repository** at drafting time — `rg -il the_formalism` matches
  only the spec and this draft. The claim is carried as the Director's framing,
  not as a repo citation; if the document lives elsewhere, the ratifying commit
  should give its path.
- **D6** — the constitution's `[TRANSITION STATE]`, `[PENDING CODE]`, and
  `[IMPLEMENTED]` markers on II.13, III.13, X.8 and others describe the **Python**
  estate. After cutover a marker sweep is owed; it is a documentation ceremony,
  not an amendment, and belongs to Phase 4.
- **Voice** — modeled on Amendments AC and AD: one dense registry paragraph,
  bolded operative clauses, sources and ratification marker at the end,
  MAJOR/MINOR class argued rather than asserted.
