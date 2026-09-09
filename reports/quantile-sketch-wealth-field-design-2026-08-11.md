# Design — the empirical quantile-sketch wealth field (issue #491 / ADR194 R1)

**Status:** RULED. Drafted read-only against `dev` on 2026-08-11; the Director answered the
blocking open questions the same day (popup batch + follow-up exchange, recorded in the
postscript below and as #491 comments). The body below is preserved as drafted — where a
ruling supersedes a workforce recommendation (notably §5.2 and OQ-H), the postscript governs.
No code, no spec text, no ADR is changed by this document.

---

## 0. Director rulings postscript (2026-08-11) — this section governs

| OQ | Disposition | The ruling |
|---|---|---|
| **OQ-A** | **RULED** | **K = 16 per-class `coefficient` mass fields**, B19001 edges transcribed, carried over the mean-relative `Ratio` `defconst` grid — the §2.4 shape as recommended. |
| **OQ-B** | **RULED — STEP** | The within-bracket reading is the **step function** (candidate (i)): `c_k ∈ {0, 1}`, count only brackets whose lower edge clears S. Zero assumptions; the visible staircase (at most K = 16 distinct values) is accepted. The draft's linear recommendation is **superseded** — §5.2's candidate (ii) stays in the text as the record of the road not taken. |
| **OQ-H** | **RULED** | **κ exists, as a moddable `defines.yaml` time-constant** with a calibrated default — a stock→flow conversion (per-tick hazard scale), never a shape parameter. The consumer formula is `deaths = floor(population × failing × κ)`. κ scales the flow uniformly and bends nothing, which is why it clears S-7 where `attrition_base_factor` did not. |
| **OQ-J** | **RULED — un-defer NOW** | This design **is** Half 2's first consumer; Currency typed-attribute storage (the `CanonicalState` widening, `reports/typed-attribute-seeding-design-2026-08-11.md`) charters alongside Phase 1. |
| **OQ-E / OQ-F** | **PROVISIONALLY RULED** | The shared county ACS **income shape** is allowed as every class's within-class dispersion — for gameplay and development — with stratification entering **only** through the engine-computed, theory-laden per-class means. **Independence is never acceptable** (it would encode "class doesn't predict material position", against the Fundamental Theorem). This proxy is a **declared approximation with a Director-mandated expiry**: the revisit is **issue #510** (candidate data: ACS occupation×income + occupation→class mapping, SCF microdata, Fed DFA blends; the coupling choice is Director-reserved when #510 charters). Director, verbatim: *"for gameplay i'll allow this approximation and for development but at some point in the future we will want to investigate to see if there's any data that will allow us to have a more theoretically rigorous conception of class than just this income stuff… eventually i want to revisit it and ensure we're theoretically coherent."* Every place the proxy enters — the §4.4 generator, the manifest `material_relation`, the seeding `.bscn` header — MUST mark it PROVISIONAL and cite #510. |
| **OQ-C** | open — rides the review | Redistributive-write deferral (§3.3) stands as the workforce recommendation until ruled. |
| **OQ-D** | open — rides the review | The C/G/P derivation under Axiom A0 is not discharged by this design (§8's row stands). |
| **OQ-G** | open — rides the review | The top-bracket `midratio` is still needed by I3's hydration normalisation even under the step reading. |
| **OQ-I** | open — rides the review | The §3.4 `extensive ÷ extensive` repair (GAP-B) remains spec work to charter. |

**Scope:** answer the five design questions ADR194 R1 explicitly left open — *"the concrete field
design (bracket count, the quantile schema `social_class` gains, and how OQ-1e's member-population
carrier is shaped) remains design work still to charter, not decided by this ruling"*
(`ai/decisions/ADR194_director_rulings_batch2_2026_08_11.yaml:92-96`).

---

## 1. Context and the rulings that bind this design

### 1.1 What was ruled

| Ruling | What it settles | Citation |
|---|---|---|
| **ADR194 R1** | Audit Q3 resolved IN DIRECTION: the canonical within-class wealth distribution is an **empirical quantile sketch** — *"Data-driven brackets (ACS-derived quantiles) carried as a first-class field. No imposed functional form"*. Lognormal, Pareto and staying-inert all rejected. | `ai/decisions/ADR194_director_rulings_batch2_2026_08_11.yaml:80-96` |
| **ADR191 R3** | The mortality family is **re-derived as a measure**: attrition deaths = the mass of the within-class wealth distribution that fails to clear subsistence. The frozen piecewise-linear form with `attrition_base_factor` is **NOT transcribed**. | `ai/decisions/ADR191_director_rulings_batch_2026_08_11.yaml:97-110` |
| **ADR173 (1)** | `P(S|A)` = the measure of class members whose wealth clears subsistence; the S-curve is *read off* the distribution; `steepness_k` ceases to exist; lands **Rust/BSL-only**, Python freezes with its logistic by design. | `ai/decisions/ADR173_audit_and_stops_dispositions.yaml:35-45` |
| **ADR172 r5 / S-7** | No functional form may be imposed on a mechanic; a sigmoid is a *result*, never a stipulated mechanism. Emergence is a **content-side** obligation, not a language property. | `ai/bsl-architecture-standard.md:633`, `:307-309` |
| **ADR183** | The frozen Python engine is a contract source for **structure and ordering**, never a correctness oracle for a **shape**. | cited at `ai/decisions/ADR191_director_rulings_batch_2026_08_11.yaml:108-110` |
| **ADR188 Row 2 / D97** | `floor` is a **declarable intrinsic** — RATIFIED AND LANDED. | `docs/reference/bsl-language.rst:3018-3021`; `rust/crates/babylon-bsl/src/declarations.rs:110` |

### 1.2 What is still open, and what this draft is for

- **Audit Q3** — the diagram node still reads `"within-class wealth distribution (canonical form
  OPEN — audit Q3)"` (`ai/bsl-architecture-standard.md:267`). ADR194 R1 closes the *direction*;
  the field design is this document.
- **OQ-1e** — *"a population measure over an intra-class distribution is not among A0's enumerated
  G-members …, `social_class` nodes carry no member population (no carrier), and the canonical
  within-class distribution is undecided"* (`ai/bsl-architecture-standard.md:684`). The **carrier**
  half is what §3 below designs; the **C/G/P derivation under Axiom A0** half is NOT discharged by
  this draft and is flagged in §8.
- **The second inheriting site** — `reports/wiring-completeness-2026-07-29.md:552-555`: *"ADR173
  obligations inherited, not closed. The holdout term reuses the ruled 'measure of members clearing
  subsistence', still owing its C/G/P derivation (OQ-1e) and canonical within-class distribution
  (audit Q3). Must not smuggle a steepness knob back as a 'strike resolve' coefficient (fresh S-7
  violation)."* One field design unblocks all three consumers.

### 1.3 What the STOP context actually says now (and where it is stale)

`rust/crates/babylon-tick/content/rules/vitality.bsl:12-30` records **two** blockers for Grinding
Attrition. **Both have moved since it was written:**

1. `vitality.bsl:16-19` says `floor` is *"a PROPOSAL, outside the {exp, log} intrinsic cap"*.
   **STALE.** `floor` is ratified and landed — `DECLARABLE_INTRINSICS: [&str; 3] = ["exp", "log",
   "floor"]` (`rust/crates/babylon-bsl/src/declarations.rs:110`); the normative paragraph is
   `docs/reference/bsl-language.rst:3018-3021` and D97 at `:3175-3207`. Its domain is
   non-negative reals only, and a negative / non-finite / out-of-`i64` argument is `E-EVAL-039`
   (`docs/reference/bsl-language.rst:3187-3195`).
2. `vitality.bsl:20-25` says the rate is a stipulated form whose emergent re-derivation is behind an
   ADR175 per-family review *"which has not happened"*. **STALE.** ADR191 R3 held that review and
   discharged the family (`ADR191…:97-110`).

**What is left is exactly this document's subject:** the distribution carrier. The plan's own
statement of the residue is `docs/superpowers/plans/2026-08-10-vitality-bsl-rule-pack.md:325-330`.

**Recommendation (housekeeping, not a Director question):** the `vitality.bsl` header should be
corrected in the same train that lands the sketch, so the file stops recording two discharged
blockers as live.

### 1.4 The frozen form being replaced (for the record, not for transcription)

```
coverage_ratio = wealth_per_capita / subsistence_needs        formulas/vitality.py:38
threshold      = 1.0 + inequality                             formulas/vitality.py:39
deficit        = threshold - coverage_ratio                   formulas/vitality.py:46
attrition_rate = clamp(deficit * (attrition_base_factor + inequality), 0, 1)   :47-50
deaths         = int(population * attrition_rate)             engine/systems/vitality.py:253
```

`attrition_base_factor = 0.5`, described in its own schema as *"Base multiplier in grinding
attrition"* (`src/babylon/config/defines/survival.py:88-93`) — a feel-tier knob with no Aleksandrov
chain. ADR191 R3 rules it not transcribed. The *structure* it carries that IS a contract under
ADR183: mortality is computed **after** the subsistence drain, off the re-read post-drain node
(`src/babylon/engine/systems/vitality.py:125-128`), deaths reduce **population only and never
wealth** (`:28-30`, `:200-218`), and the decrement is floored to whole people (`:253`).

### 1.5 What `social_class` carries today

| Carrier | Where | Note |
|---|---|---|
| `wealth: Currency` | `src/babylon/models/entities/social_class.py:308-311` | one scalar, the whole class |
| `population: int` | `:406-410` | the block size — a count, **not** a member distribution |
| `inequality: Gini` | `:411-415` | a **single scalar Gini**; the only dispersion carrier that exists |
| `wealth_share: Probability` | `:312-321` | Program-21 **national 4-bracket** axis projection, per-class, shadow-only |

In BSL content today (`rust/crates/babylon-tick/content/scenarios/vitality-conformance.bscn:22-33`)
`social-class/inequality` is *declared and seeded, read by NO rule* — the file says so in as many
words. There is no distribution, no quantile, no dispersion field anywhere in the language or the
content.

---

## 2. Question 1 — REPRESENTATION

### 2.1 The constraints any representation must satisfy

**C1 — The `deffield` type vocabulary is closed and was re-sealed twice this month.**
`<type-name>` is a lowercase symbol drawn from exactly six names — `int`, `bool`, `currency`,
`probability`, `intensity`, `coefficient` (`docs/reference/bsl-language.rst:2148-2165`,
`:2203-2217` = ADR191 R4 / D94). `Real` and `Ratio` are explicitly **non-storable** and *"no
`<type-name>` position can name"* them (`:2209-2222`). The Rust loader admits exactly five
(`rust/crates/babylon-bsl/src/scenario.rs:567-579`). **There is no vector type and no way to name
one.**

**C2 — The tick hash covers scalar attributes only, by byte layout.**
`CanonicalState` section `0x02` is `u32 count ‖ per attribute (u64 id ‖ str name ‖ f64 bits)`, and
the encoder's signature is literally `&[(NodeId, String, f64)]`
(`rust/crates/babylon-graph/src/state_hash.rs:19-24`, `:136-152`, `:241-243`). Sorting is by
`(id, name)` (`:265`). A vector-valued attribute has **no byte layout in this encoder** — it would
need a new section or a widened row, and ADR192 already names a `CanonicalState` encoder widening a
**declared ceremony**.

**C3 — Graph attribute storage is `f64`, full stop.**
`GraphSubstrate` attributes are `f64`; that is why Currency seeding is a loud refusal to this day
(`rust/crates/babylon-bsl/src/scenario.rs:40-48`) and why Half 2 of typed-attribute seeding is
DEFERRED TO FIRST CONSUMER by Director ruling (same lines, and PR #505's body).

**C4 — The seed-vs-runtime bit-equality contract exists and is pinned, for the `f64` lane only.**
PR #505 (merged) pins `a_seeded_literal_bit_matches_the_same_literal_written_by_a_rule` by
`to_bits()` equality off the live graph, with the conversion contract mutation-verified
(`0.7c → 0x3fe6666666666666`). **Any representation that reuses the existing scalar `f64` lane
inherits that contract for free. Any new lane owes its own.**

**C5 — Hydrated reference data enters as declared node fields, by ruling.**
*"the data-build pipeline materialises keyed series as declared node fields at hydration, so a rule
reads them with an ordinary `:field`. That keeps §2.8's no-I/O prohibition intact, keeps the values
inside the content hash rather than beside it, and needs no language change at all"*
(`docs/reference/bsl-language.rst:2830-2840`). The same ruling explicitly refuses a `:reference`
bind-src. **This is a direct precedent against option (c) below.**

**C6 — There is no query head over fields.** The five query heads are `nodes`, `edges`, `neighbors`,
`members-of`, `hyperedges-of` (`docs/reference/bsl-language.rst:933-938`). A set of *fields on one
node* is not a `<query>` and therefore not foldable. K scalar fields must be consumed by explicit
K-term arithmetic.

**C7 — Modding.** Coefficients live in `GameDefines`/`defines.yaml` and reach a rule as `:const`
(`docs/reference/bsl-language.rst:2830-2840` for the boundary; `scenario.rs:251-360` for the
`.bscn` `defconst` environment). **Per-class empirical masses are DATA, not coefficients** — they
belong in hydration. **Bracket cut points are SCHEMA** — they belong in `defconst`, where a modder
can move them.

### 2.2 The three candidates the task named, weighed

**(a) K brackets as K scalar fields.**
Satisfies C1–C7 with **zero language change**: `deffield` already admits `coefficient`, the encoder
already hashes each attribute as its own `(id, name, f64)` row, the seed path is bit-pinned, and
`:field` bindings already work. Costs: K names baked into rule source (C6 — no fold), a wide
`deffield` block, and changing K is a content edit across every consuming rule.

**(b) A new vector-valued field kind.**
Costs, itemised, because they are not obvious: a seventh `<type-name>` re-opening a table sealed
twice this month (C1); a `§3.4` kind rule for a vector (is a mass vector intensive or extensive? —
undefined today); a `§5.2` canonical-AST encoding and CAS vectors; a `CanonicalState` section
widening (C2) which ADR192 makes a declared ceremony; a new set kind and/or query head so the
vector can be folded — precisely the *"fourth reference kind … a set type, a query head"* bundle
§2.12 refused for memberships and recorded *"so it is not re-proposed"*
(`docs/reference/bsl-language.rst:2062-2071`); and a fresh seed/runtime bit-equality contract (C4).
Under Amendment AE (ii) — *BSL expresses this algebra and mints no new mathematics* — a *storage*
kind is arguably machinery, not mathematics, so it is probably not an *amendment*; but it is
unambiguously a **spec chapter + determinism-contract edit + encoder ceremony**, i.e. the largest
of the three by an order of magnitude.

**(c) Kernel-side typed storage consumed via bindings.**
Directly against C5's standing ruling, and against C2: state that lives beside the graph is either
outside `CanonicalState` (a III.7 determinism hole — *"Every tick produces a deterministic hash"*)
or needs its own hash section anyway, at which point it has paid (b)'s price without (b)'s
expressiveness. **Reject.**

### 2.3 A fourth candidate, raised and rejected here so it is not re-proposed

**(d) Brackets as graph elements** — one `WEALTH_BRACKET` node (or hyperedge member) per
(class, bracket), joined by an `IN_DISTRIBUTION` edge. This is the only option that makes the sketch
**foldable** (`(fold sum (neighbors self EdgeType/IN_DISTRIBUTION …) …)`), K-agnostic in rule
source, and ceiling-bounded for free (`docs/reference/bsl-language.rst:2787-2792` is the pattern,
recommended there for the scale lattice).

Rejected on three grounds: (i) ADR194 R1's own words are *"carried as a first-class **field**"*;
(ii) node-count blow-up — 3,143 counties × classes × K is a substrate-scale multiplication for what
is an attribute of one class; (iii) the Aleksandrov test is at best strained — a bracket is a
*measurement partition of* a class, not a distinct material relation, and minting node types for
measurement artefacts is the failure mode the hex/community disposition already warns against.
Recorded because it is the ergonomically attractive wrong answer.

### 2.4 RECOMMENDATION — option (a), in a specific shape

> **K per-class `coefficient` mass fields carrying the empirical SHAPE, over a universal
> mean-relative cut grid carried as `Ratio` `defconst`s. No new type, no new operator, no encoder
> ceremony, no amendment.**

Concretely, the schema `social_class` gains:

```scheme
; ---- the shape: K per-class masses, hydrated, empirical, the whole ACS content ----
(deffield social-class/wealth-mass-01 coefficient intensive)   ; share of members in bracket 1
(deffield social-class/wealth-mass-02 coefficient intensive)
...
(deffield social-class/wealth-mass-16 coefficient intensive)

; ---- the grid: K+1 cut points, universal, MEAN-RELATIVE, moddable ----
; cut_k = (per-capita wealth) x ratio_k.  ratio_0 = 0 is implicit; ratio_K is the open top.
(defconst wealth-sketch/cut-01 0.18r)
(defconst wealth-sketch/cut-02 0.27r)
...
(defconst wealth-sketch/cut-15 3.6r)
```

**Why mean-relative `Ratio` cut points and not stored money.** This is the load-bearing half of the
recommendation and it is what makes option (a) work *without* a maintenance burden:

- `Ratio` is a real, landed type with domain `(0, ∞)`, produced by a `defconst` literal, and its
  **one** legal operation is `Currency × Ratio → Currency`, half-even
  (`docs/reference/bsl-language.rst:2269-2278`, D99). The implementation is in the tree today:
  `reader::classify_ratio`, `scenario::load_ratio_defconst`
  (`rust/crates/babylon-bsl/src/scenario.rs:446-507`), `evaluator`'s `*` arm
  (`rust/crates/babylon-bsl/src/evaluator.rs:553-566`), `babylon_kernel::Currency::mul_ratio`
  (`rust/crates/babylon-kernel/src/currency.rs:137`).
- Because the grid carries **no money**, it never goes stale. Every system that writes scalar
  `wealth` automatically re-anchors the whole sketch, at read time, with no write of its own. That
  is the entire answer to Question 2 (§3).
- Because the grid is `defconst`, it is in `rules_hash`/`ContentDigest` and it is **moddable** — a
  modder re-cuts the distribution by editing constants, exactly the `defines.yaml` shape C7 wants.
- Because the masses are `coefficient` fields, they are `f64`, they hash as ordinary section-`0x02`
  rows (C2), they seed through the bit-pinned Half-1 lane (C4, PR #505 legalised exactly
  `probability`/`intensity`/`coefficient` seeding), and they are per-class/per-county **data**
  where data belongs (C5).

**Bracket count: K = 16, cut points transcribed from ACS B19001.** Reasons in §4.3. The grid is
`defconst`, so K is changeable by ceremony rather than by amendment — but K *does* appear in rule
source (C6), so changing it is a rule-pack edit plus a vector re-bless, not a free knob. Say so
out loud in the schema comment.

**`:kind intensive` on the masses.** A bracket share is per-member; an unweighted mean of one across
classes is exactly the recorded variance error §3.4 exists to reject
(`docs/reference/bsl-language.rst:2379-2411`). Declaring them intensive means any future
cross-class aggregation is forced to carry an extensive `:weight` (population) or fail loudly at
load with `E-TYPE-042`. That is the correct and desirable friction.

**OQ-1e's "member-population carrier".** It already exists and needs no new field:
`social-class/population` is the member count, and `wealth-mass-k × population` is the member
measure in bracket *k* (`Int` extensive × `coefficient` intensive → extensive; §3.4's `*` bullet,
`:2373-2376`). The sketch supplies the *distribution over* that population, which is the half
OQ-1e says is missing. **What OQ-1e ALSO asks — the C/G/P derivation under Axiom A0 — is not
supplied by any field design and stays open (§8, OQ-D).**

---

## 3. Question 2 — UPDATE SEMANTICS

### 3.1 The invariants

| # | Invariant | Enforced where |
|---|---|---|
| **I1** | **Mass conservation.** `Σ_k mass_k = 1` exactly, at hydration and after any shape write. | hydration-time check (§4.5); a load error if violated |
| **I2** | **Monotone non-decreasing grid.** `0 < ratio_1 ≤ ratio_2 ≤ … ≤ ratio_15`. | `defconst` declaration check; the grid is content, so this is a load-time check over constants |
| **I3** | **Mean consistency.** `Σ_k mass_k × midratio_k = 1` — the sketch's own mean equals the class's per-capita wealth by construction. | hydration-time normalisation (§4.5) |
| **I4** | **Non-negativity.** each `mass_k ∈ [0,1]`. | free — `coefficient`'s declared domain, checked at the store boundary (`E-EVAL-020`, `docs/reference/bsl-language.rst:2337-2345`) |

I3 is what makes the scalar `wealth` field and the sketch **one object rather than two**. It is the
reason no desynchronisation bug class exists here.

### 3.2 What happens on a wealth write — the answer is "nothing, and that is the point"

Every one of the 34 systems writes scalar `wealth` today and none of them know the sketch exists.
Under the recommended representation **that stays true**:

- The sketch stores **shape only** (dimensionless masses over a dimensionless grid). It carries no
  money and therefore cannot be stale in money terms.
- A wealth write changes per-capita wealth; the consuming rule multiplies the grid by the *current*
  per-capita wealth at read time; every cut point moves proportionally. **The sketch re-anchors to
  the scalar mean implicitly, every tick, for free.**
- I3 is preserved by construction under any *uniform proportional* change: scaling `wealth` scales
  every cut point by the same factor and leaves masses untouched.

This answers the task's framing directly: **not "shift additively", not "re-anchor by an explicit
write" — re-anchor implicitly by never storing the scale.**

### 3.3 What happens on a *redistributive* write — declared as a typed motion, and DEFERRED

A uniform per-capita drain (Vitality's subsistence cost, `vitality.bsl:47`) is *not* uniform in
proportional terms: subtracting the same absolute amount from every member compresses the bottom of
the distribution and can push the lowest bracket below zero. Under the shape-only design the
delivered semantics of a subsistence drain is *proportional*, not *additive* — a knowingly
approximate reading.

**Recommendation: do not fix this in the first train.** Land the sketch shape-only and **frozen** —
no rule writes a mass field in Phase 1. Systems that genuinely redistribute within a class
(Dispossession, Decomposition, MarketScissors, WealthDistribution) get shape writes in their own
ports, each one entered as an ADR109 typed motion with its own sentinel row
(`ai/wiring-doctrine.md`; the doctrine is summarised in `CLAUDE.md`'s wiring-doctrine bullet). A
mass write is a **W-A4 conservation closure** by nature: it must preserve I1 and I3 or it is a
defect, and that is exactly the kind of obligation the doctrine's sentinel rows exist to pin.

Recorded as **OQ-C** in §8: whether the additive-vs-proportional distinction is a Phase-1 blocker or
a declared Phase-2 refinement is a Director call, because it is a fidelity question about the
measure the ruling names, not an engineering one.

---

## 4. Question 3 — DATA DERIVATION

### 4.1 What ACS data is actually in the tree

`fact_census_income` — **7,207,200 rows**, sha-pinned parquet at
`dist/data-artifacts/fact_census_income.parquet`, described as *"ACS household-income-bracket-by-county
rows — the nationwide bracket-ratio source for the labor-aristocracy/class-proxy signal (Constitution
Amendment R canonical scale)"* (`data-artifacts.yaml:681-691`).

Verified schema (read-only query against `data/sqlite/marxist-data-3NF.sqlite`):

```sql
CREATE TABLE fact_census_income (
    county_id, source_id, bracket_id, time_id, race_id, household_count,
    PRIMARY KEY (county_id, source_id, bracket_id, time_id, race_id));
```

Measured cardinality: **3,221 distinct counties × 16 brackets × 10 race codes × 14 years
(2010–2023)**, single source `ACS 5-Year Estimates 2010 (Census API)`. Precision note (verified
2026-08-11): the row count is 7,207,200 while the full cross-product would be 7,215,040 — 49
(county, year) pairs are absent, so the panel is **near-complete, not complete**; the §4.4
generator must tolerate missing pairs rather than assume the grid.

`dim_income_bracket` — the **B19001** schedule, 16 real rows plus one `NAM` artefact row
(`bracket_order` 17, "Geographic Area Name", not a bracket):

| order | code | label |
|---|---|---|
| 1 | B19001_002 | Less than $10,000 |
| 2–9 | B19001_003…010 | $10,000–14,999 … $45,000–49,999 (five-thousand-dollar steps) |
| 10 | B19001_011 | $50,000 to $59,999 |
| 11 | B19001_012 | $60,000 to $74,999 |
| 12 | B19001_013 | $75,000 to $99,999 |
| 13–15 | B19001_014…016 | $100,000–124,999, $125,000–149,999, $150,000–199,999 |
| 16 | B19001_017 | $200,000 or more |

**Finding — `bracket_min_usd` and `bracket_max_usd` are NULL for every row.** The numeric edges
exist only inside `bracket_label` strings. Any derivation must either parse the labels or carry the
B19001 edge schedule as a declared constant table. Recommend the latter (declared, cited, checked
against the labels by a test) — parsing money out of a display string at build time is the kind of
silent-failure surface `fact_census_hours`'s *"aggregate_hours column 100% NULL from a silent
label-match bug in the deleted loader"* (`data-artifacts.yaml:665-670`) already burned this repo on.

### 4.2 The two honest gaps between this data and the ruled construct

**GAP-1 — the data is INCOME; the construct is WEALTH.** B19001 is household income. The engine's
`wealth` is a stock (`social_class.py:308-311`). The tree *does* carry a wealth distribution, but it
is national and 4-bracket, not county-level: `fact_fred_wealth_shares` (480 rows) over
`dim_wealth_class` — `LT01 Top 1% → core_bourgeoisie`, `N09 90-99% → petty_bourgeoisie`,
`N40 50-90% → labor_aristocracy`, `B50 Bottom 50% → internal_proletariat` — surfaced by
`view_wealth_concentration`. Program 21's four equilibrium shares are its game-side rendering
(`src/babylon/data/defines.yaml:462-465`: `equilibrium_w1 0.305 / w2 0.382 / w3 0.294 / w4 0.02`).

**GAP-2 — no class × income cross-tab exists.** `fact_census_worker_class` carries
`marxian_class` (`dim_worker_class`: proletariat 6 rows, state_worker 6, petty_bourgeois 4,
unpaid_labor 2, NULL 4), surfaced per-county by `view_class_composition`. `fact_census_income`
carries income × race × county. **They share county and nothing else.** A per-class-per-county
bracket vector is therefore **not readable from the data**; it must be *constructed* from two
marginals, and any construction (independence, iterative proportional fitting, a class-conditional
shift) is an assumption.

Both gaps are Director-reserved (§8, OQ-E and OQ-F) — **and both are now PROVISIONALLY RULED (§0):
the shared county income shape is the allowed proxy, stratification via theory-laden per-class
means only, never independence, revisit = issue #510.** §4.4's derivation puts the assumption
where a reviewer can see it; every entry point marks it PROVISIONAL citing #510.

### 4.3 Why K = 16 and why the ACS edges are transcribed rather than re-cut

The measure this sketch exists to serve is *mass failing/clearing **subsistence***. Subsistence sits
near the bottom of the income distribution. B19001's schedule is **finest exactly there** — nine
cut points below $50,000, then coarsening upward. Any re-bracketing to a smaller K would spend its
resolution budget in the wrong place, and *"weak lower/middle-mass fit precisely where
subsistence-clearing bites — the region the measure cares about most"* is the Director's own stated
reason for rejecting Pareto (`ADR194…:87-90`). Transcribing the ACS schedule is the choice most
consistent with the ruling that selected it.

Aggregating adjacent ACS brackets (K = 8) would be *arithmetically* honest — summing counts assumes
nothing — but it is a resolution choice, and this one is free.

### 4.4 The pipeline stage, per ADR098

`data-artifacts.yaml` + `dist/data-artifacts/schema.sql` are canonical; `tools/build_reference_db.py`
rebuilds `data/sqlite/marxist-data-3NF.sqlite` sha-identically on the pinned toolchain (sqlite
`3.53.1`, `tools/build_reference_db.py:82`); every `CREATE TABLE` in `schema.sql` must have a
manifest entry and vice versa (`:23-31`). Loaders never write the DB.

**Proposed stages:**

1. **Reference stage (new derived table).** `dim_wealth_sketch_bracket` (K rows: order, code, label,
   `min_usd`, `max_usd` — the *declared* B19001 edges, closing the NULL gap of §4.1) and
   `fact_class_wealth_sketch` (`county_id, marxian_class, bracket_id, time_id, member_share`).
   Both enter through `schema.sql` + a `data-artifacts.yaml` manifest entry + a generator in
   `tools/make_data_artifacts.py`, exactly as every other fact table does. The construction
   assumption of GAP-2 lives in **one** generator function with a `material_relation` string that
   names it.
2. **Normalisation stage.** Convert absolute USD edges to the **mean-relative grid**: divide each
   edge by the county-and-class mean income, then take the **national median across counties** of
   each normalised edge as the universal `defconst` grid. This is what makes the grid universal and
   the masses per-class. Recompute masses against the universal grid per (county, class) so I1 and
   I3 hold exactly.
3. **Hydration stage.** §3.9 clause 1 — hydration *"creates elements of declared types and writes
   declared fields, and nothing else"* (`docs/reference/bsl-language.rst:2842-2847`) and clause 4
   makes a `:field` binding's seeding a **blocking dependency** (`:2852-2856`). The Rust engine has
   no general hydration path yet — `us-counties-lifecycle-demo.bscn` hard-codes its twelve counties
   from a printed run (`:7-24`). So Phase 1 seeds the sketch through a **generated `.bscn`**, with
   the generator committed and its output pinned, matching the demo scenario's existing discipline.

### 4.5 Hydration-time checks (where I1 and I3 are enforced)

The masses are produced by a build tool, so the checks belong there and in a load-time assertion:

- I1: `|Σ mass_k − 1| = 0` after normalisation, expressed in the same `f64` the encoder hashes
  (compute in exact rational, round once, and make the last bracket absorb the residue — declared,
  not incidental).
- I3: `|Σ mass_k × midratio_k − 1| = 0` under the same discipline.
- The open top bracket ($200,000+) has **no midpoint**. Its `midratio` must be a declared, cited
  constant, not an invented one. Flagged as **OQ-G** in §8 — this is a real modelling decision on
  the tail, and the tail is where the bourgeoisie lives.

---

## 5. Question 4 — CONSUMERS, with exact arithmetic

### 5.1 The one measure, stated once

Let `S` = per-member subsistence = `s-bio + s-class` (both declared `intensive` in
`vitality-conformance.bscn:29-30`), `w̄` = per-capita wealth, `cut_k = w̄ × ratio_k`, `m_k` = the
bracket masses.

```
clearing = Σ_k  m_k · c_k          where  c_k = the fraction of bracket k lying at or above S
failing  = 1 − clearing
```

`c_k` is 1 for brackets entirely above S, 0 for brackets entirely below, and for the **one** bracket
containing S it is the fraction of that bracket above S. Both consumers read this same quantity:

- **P(S|A)** (ADR173) = `clearing` — the measure of class members whose wealth clears subsistence.
- **Grinding Attrition** (ADR191 R3) = `floor(population × failing × κ)` where κ is the per-tick
  hazard scaling. `failing` is a *stock* (who is below the line right now), while attrition is a
  *flow* (who dies this tick); any constant converting one to the other risked being an S-7 knob
  wearing a new name. **RULED (§0, OQ-H): κ exists as a moddable `defines.yaml` time-constant
  with a calibrated default** — a temporal scale that multiplies the flow uniformly and bends no
  curve, which is the property that distinguishes it from the killed `attrition_base_factor`.

**No functional form appears anywhere above.** There is no exponent, no steepness, no `exp`, no
`sigmoid`. Every operation is a multiplication, an addition, a comparison, or a linear fraction of
two measured quantities. The S-curve emerges because `clearing` as a function of `S/w̄` **is** the
class's own complementary CDF, read off the data.

### 5.2 The BSL-expressible shape

`c_k` needs a within-bracket reading. Two candidates were presented; **the Director ruled for
(i), the step function (§0, OQ-B)** — candidate (ii) is preserved below as the record of the
alternative, not as a live option:

**(i) Step (no interpolation, zero assumptions).** `c_k ∈ {0, 1}` — count only brackets whose
*lower* edge clears S.

```scheme
; bracket k's LOWER edge is cut-(k-1); bracket 1's lower edge is the implicit 0,
; which no positive subsistence ever clears, so mass-01 never counts. Bracket 16
; (the open top) is guarded by cut-15.
(binding clearing :expr
  (+ (if (>= cut-01 subsistence) mass-02 0.0c)
     (+ (if (>= cut-02 subsistence) mass-03 0.0c)
        ... )))
```
Pure measure arithmetic, no assumption at all: every counted member *provably* clears. Cost:
`clearing` takes at most K = 16 distinct values per class (S falls in exactly one bracket;
`clearing` is the mass strictly above it), so the emergent curve is a visible staircase.

**(ii) Linear within bracket (the standard empirical-CDF convention).**

```scheme
; per bracket k, with lo = cut_{k-1}, hi = cut_k:
;   S >= hi  -> 0          (bracket entirely below subsistence)
;   S <= lo  -> mass_k     (bracket entirely above)
;   else     -> mass_k x (hi - S) / (hi - lo)
(binding clear-07 :expr
  (if (>= subsistence cut-07)
      0.0c
      (if (<= subsistence cut-06)
          mass-07
          (* mass-07 (/ (- cut-07 subsistence) (- cut-07 cut-06))))))
```

**Type-legality of (ii), operator by operator:**

| Sub-expression | Types | Legal? |
|---|---|---|
| `(/ wealth population)` → `w̄` | `Currency ÷ Int → Currency`, half-even | ✔ `docs/reference/bsl-language.rst:2235` |
| `(* w̄ cut-ratio-07)` → `cut-07` | `Currency × Ratio → Currency`, half-even | ✔ `:2269-2278` (D99, landed) |
| `(>= subsistence cut-07)` | Currency vs Currency comparison | ✔ |
| `(- cut-07 subsistence)`, `(- cut-07 cut-06)` | `Currency ± Currency → Currency`, checked, `E-EVAL-010` below zero | ✔ `:2252-2255` |
| `(/ … …)` of those two | `Currency ÷ Currency → Coefficient`, i256 intermediate, half-even | ✔ `:2234-2235`; the result **must** land in `[0,1]` or `E-EVAL-013` (`:2256-2257`) — **guaranteed by the enclosing `if`s**, which is why the guards are written as nested `if` and not as a clamp |
| `(* mass-07 frac)` | `coefficient × coefficient` in the binary64 lane, promoting to `Real` | ✔ `:2328-2345` |
| `(+ …)` of K terms | binary64 lane, `Real` | ✔ `:2337-2345`; the range check happens once at the **store boundary** (`E-EVAL-020`), never as a clamp |
| `(floor (* population failing))` | `Int` promotes to `Real`; `floor : Real → Int`, non-negative domain, `E-EVAL-039` otherwise | ✔ `:3018-3021`, `:3187-3195` |

**Nested `if` rather than a clamp is required, not stylistic.** §3.10 rider row 5 declines scalar
`min`/`max` precisely *"so a saturation stays legible in the source rather than hiding in an
operator"* (`docs/reference/bsl-language.rst:3093-3098`), and `vitality.bsl:53-61` already carries
that discipline in prose.

**Fuel.** Per §3.7's cost model (`docs/reference/bsl-language.rst:2524-2560`): each bracket term is
roughly `cost(if) 1 + cond ~3 + max(branch)` where the deep branch is `~12` → **~16 per bracket**;
K = 16 brackets → **~256**, plus the drain algebra already in `vitality.bsl` (`:fuel 512`, `:33`).
A rule budget of **1024** is comfortable; it is a static, declared number and the load-time bound
check (`ceiling(query)`-driven) is unaffected because there are **no folds** — the sketch is read
by field bindings only.

### 5.3 The two live language gaps this arithmetic exposes

**GAP-A — `wealth` is not a `Currency` field today.** `vitality-conformance.bscn:22-24` declares
`social-class/wealth` as `int`, because `GraphSubstrate` attributes are `f64` and **Currency seeding
is a loud refusal** — Half 2 of typed-attribute seeding is DEFERRED TO FIRST CONSUMER by Director
ruling (`rust/crates/babylon-bsl/src/scenario.rs:40-48`). With `wealth : int`, `(/ wealth
population)` lands in the **binary64 lane**, `w̄` is a `Real`, and `Real × Ratio` **is not a legal
operation** — `Ratio`'s only operator is `Currency × Ratio`
(`rust/crates/babylon-bsl/src/evaluator.rs:56-62`, `:553-566`). Issue #502 workstream 3 names
exactly this: *"mint the missing Real-lane declared-domain op, then sweep every bare-Int workaround
out in one pass"*.

> **Therefore: this design's first consumer IS Half 2.** The sketch is the concrete, named consumer
> the Director's defer-to-first-consumer ruling was waiting for. Landing Currency field storage
> makes the arithmetic above legal as written, with **no new operator at all**. The alternative —
> waiting for #502 WS3's Real-lane op — is a larger and later change.

**GAP-B — `extensive ÷ extensive` is `E-TYPE-040` in the spec.** `(/ wealth population)` divides two
extensive fields, and §3.4's `*`//` bullet rejects that: *"`E-TYPE-040` if both are extensive (an
area-of-an-area) — this is deliberately conservative and a Phase-1 review item"*
(`docs/reference/bsl-language.rst:2373-2376`). **Per-capita wealth — the canonical
intensive-from-two-extensives construction — is spec-illegal as the bullet is written.** The bullet
conflates `*` and `/`: an area-of-an-area is a real error for multiplication; `density = mass ÷
volume` is the *definition* of an intensive quantity. §3.4 is already aware of the consequence — D90
had to state a fold's result kind in the table precisely because *"deriving it through the `*`//`
bullet is deliberately unavailable — that bullet rejects extensive ÷ extensive as `E-TYPE-040`"*
(`:2421-2426`).

**Not yet caught.** The kind checker's arithmetic arm does not exist: *"Kind-NEUTRAL bodies
(literals, `:const` bindings, arithmetic over them) arrive with the expression typechecker in later
tasks, as does `E-TYPE-040` kind mixing"* (`rust/crates/babylon-bsl/src/typecheck.rs:15-19`); the
only `E-TYPE-040` mention in the crate is that comment. Building on an unenforced prohibition would
plant a trap that detonates the day the expression typechecker lands. Raised as **OQ-B** in §8: the
repair is one clause — `extensive ÷ extensive → intensive` — and it is unit algebra, not new
mathematics, the same standing D90's own repair took.

---

## 6. Question 5 — DETERMINISM AND BASELINES

### 6.1 Hash implications — precise, and larger than "none until consumed"

**Seeding the sketch moves the tick hash immediately, before any rule reads it.** Section `0x02`
holds one row per `(node, attribute)` pair (`rust/crates/babylon-graph/src/state_hash.rs:19-24`,
`:136-152`), so K new attributes on N classes adds K×N rows to both the **pre-tick** and post-tick
hash. This is not a subtlety to discover later:

- `rust/crates/babylon-tick/tests/tick_goldens.rs:58-69` pins **pre- and post-tick** hashes for
  `vitality-conformance.bscn`; `:41-52` for `two-classes.bscn`; `:82-96` for the counties demo, the
  last cross-confirmed against `babylon-client`'s `engine_link` and `tick_loop`.
- **Mitigation: seed the sketch into a NEW scenario** (`vitality-attrition-conformance.bscn`), not
  into the existing `vitality-conformance.bscn`. Then every pinned golden stays byte-identical and
  the new scenario arrives with its own measured pins. `vitality-conformance.bscn`'s header already
  explains that its fixture is chosen so the un-ported phase contributes nothing (`:8-14`) — that
  fixture is *deliberately* the wrong world for an attrition test, since the frozen engine kills
  nobody in it.

### 6.2 Which goldens move

| Estate | Moves? | Why |
|---|---|---|
| `mise run qa:regression` (11 Python scenarios) | **No** | ADR173 lands the construct **Rust/BSL-only**; the Python engine is frozen at `p27-python-freeze` and gains no field |
| `mise run qa:vault-regression-ci` | **No** | same — a Python-side `observe()` estate |
| `tests/baselines/**` (§6.5 ceremony gate) | **No** | Python-side; no `Baselines: blessed(…)` trailer is owed by this train |
| `rust/crates/babylon-tick/tests/tick_goldens.rs` | **Only the new scenario's pins** if §6.1's mitigation is taken; **all three pairs** if the sketch is added to existing scenarios | section `0x02` row count |
| `babylon-client` `engine_link` / `tick_loop` hash pins | **No** under the mitigation | they observe the demo scenario |

### 6.3 Conformance strategy — SUBSTITUTE, and it is already ruled

ADR191 R3 rules the frozen piecewise-linear form **NOT TRANSCRIBED**
(`ADR191…:106-110`), and ADR183 holds the frozen engine to be a contract source for *structure and
ordering*, never a correctness oracle for a *shape*. So the transcribe-or-substitute decision is
**already made and is SUBSTITUTE** — the workforce may not derive attrition vectors by running the
Python engine. This mirrors ADR173's own instruction for the survival family: *"Phase 1 conformance
vectors encode IT, not the logistic"* (`ADR173…:40-42`), and *"cross-implementation checks for
survival quantities compare against the emergent formulation's own vectors, not Python replay"*
(`:70-73`).

**What the vectors must therefore be:**

1. **Measure-arithmetic vectors** authored from the sketch itself, with hand-computed expected
   values from an independent implementation (the PR #505 discipline: expected bits computed via a
   *different* language's conversion, never by the code under test computing its own oracle).
   Exact equality, no tolerance — `vitality_conformance.rs`'s own standing rule is *"a tolerance
   here would hide exactly the transcription error it would appear to absorb"*.
2. **Boundary vectors** where the measure is exactly determined regardless of interpolation:
   S below every cut (`clearing = 1 − mass-01` under the ruled step reading — bracket 1's
   implicit-zero lower edge never provably clears), S at or above every cut (`clearing = 0`),
   S exactly on a cut point
   (the half-even rounding case), a degenerate one-member class, an all-mass-in-one-bracket class.
3. **Monotonicity property vectors**: `clearing` is non-increasing in S and non-decreasing in `w̄`.
   These are the ones that pin *emergence* — they hold for any admissible sketch and are what makes
   "the S-curve is read off the distribution" a checkable claim rather than a slogan.
4. **What is still preserved from the frozen engine (ADR183 structure/ordering):** attrition runs
   *after* the drain and off the re-read post-drain state
   (`src/babylon/engine/systems/vitality.py:125-128`); deaths reduce population and **never** wealth
   (`:28-30`); the decrement is floored (`:253`); the two `continue` guards at the top of the loop
   (`vitality.bsl:65-66`). Those are transcribed. The *shape* is not.

### 6.4 The S-7 obligation this train owes

S-7's proof column requires *"a written derivation of the form from the algebraic operations"* and
notes **no automated check exists yet** (`ai/bsl-architecture-standard.md:633`). This design's
derivation is §5.1, and it should ship *in the rule's `:material-basis` string and its header*, not
only in a plan file — that is where a reviewer will look. The mechanical half is already in place:
`sigmoid` is a reserved prohibited intrinsic name, `E-LOAD-024`
(`docs/reference/bsl-language.rst:3041-3046`; `PROHIBITED_INTRINSIC_NAMES`,
`rust/crates/babylon-bsl/src/declarations.rs:116`).

---

## 7. Implementation sketch — train phases

Each phase is independently mergeable and gate-green. Phases 1–3 are engineering; phase 0 is the
Director gate.

**Phase 0 — Director gate (this document). DISCHARGED 2026-08-11 (§0):** OQ-A (K = 16, ACS
grid), OQ-B (step), OQ-H (κ = moddable time-constant), OQ-J (Half 2 un-deferred) are ruled;
OQ-E/F provisionally ruled with the #510 expiry. Phases 1–3 are unblocked; Phase 4's generator
carries the PROVISIONAL marking.

**Phase 1 — the carrier, inert.**
- `deffield` block for `social-class/wealth-mass-01..16` (`coefficient intensive`) and the
  `wealth-sketch/cut-01..15` `Ratio` `defconst` grid, in a **new** scenario
  `vitality-attrition-conformance.bscn`.
- No rule reads them. Ship the pre/post-tick golden pins for the new scenario, measured.
- Verify: `cargo test -p babylon-tick` green; the three existing `tick_goldens` pairs
  **byte-identical**; `mise run qa:regression` 11/11 byte-identical (it must be — Python is
  untouched).

**Phase 2 — Half 2 typed-attribute seeding (the deferred half, now with its first consumer).**
- Currency field storage per `reports/typed-attribute-seeding-design-2026-08-11.md` Half 2 — the
  `CanonicalState` fifth section and the `bind_subject` type-aware dispatch. This is GAP-A (§5.3).
- **The `CanonicalState` widening is a declared ceremony** (ADR192's own consequence queue).
- Re-declare `social-class/wealth` as `currency` in the new scenario only, leaving the existing
  vitality/lifecycle scenarios on `int` until their own trains move them.
- Verify: seed/runtime bit-equality extended to the Currency lane with the PR #505 mutation
  discipline; existing goldens byte-identical.

**Phase 3 — the measure, and its two consumers.**
- 3a: the `clearing` binding and its vectors (§6.3 families 1–3). No effect yet — a binding and a
  condition only. This lands **P(S|A)** as a computable quantity and discharges the ADR173 landing.
- 3b: Grinding Attrition — `(floor (* population failing))`, the population decrement, the
  `POPULATION_ATTRITION` emit, in the structural order §6.3(4) preserves. Gated on OQ-H (κ).
- Verify: new conformance suite green with exact equality; `mise run rust:check`;
  `cargo clippy --workspace --all-targets -- -D warnings`.

**Phase 4 — the ACS derivation.**
- `dim_wealth_sketch_bracket` + `fact_class_wealth_sketch` through `schema.sql` +
  `data-artifacts.yaml` + `tools/make_data_artifacts.py`, with the GAP-2 construction assumption
  isolated in one named function and stated in the manifest's `material_relation`.
- The `.bscn` generator, committed, output pinned.
- Verify: `mise run data:build-db` reproduces the DB sha-identically on the pinned toolchain
  (`tools/build_reference_db.py:82`, sqlite 3.53.1); I1/I3 checks green on every (county, class).

**Phase 5 — housekeeping.**
- Correct `vitality.bsl:12-30`'s two now-stale blockers (§1.3).
- Update `ai/bsl-architecture-standard.md:267` (the diagram's `canonical form OPEN` label),
  `:684` (OQ-1e's carrier half), and `reports/wiring-completeness-2026-07-29.md:552-555`.
- ADR recording the field design as landed.

---

## 8. Open Director questions (as drafted — dispositions now in §0)

These are the questions the workforce should not answer for itself. **OQ-A and OQ-B block Phase 1.**
*(Postscript: OQ-A/B/H/J ruled, OQ-E/F provisionally ruled, OQ-C/D/G/I still open — see §0. The
table below is the historical record as presented.)*

| # | Question | Why it is reserved | Workforce recommendation |
|---|---|---|---|
| **OQ-A** | **Bracket count and grid.** K = 16 with the B19001 edges transcribed, carried as mean-relative `Ratio` `defconst`s? Or a coarser K? | ADR194 R1 explicitly leaves *"bracket count"* to chartered design, and the Director's own reason for rejecting Pareto was lower/middle-mass fit — a resolution argument. | **K = 16, ACS edges transcribed** (§4.3) |
| **OQ-B** | **Within-bracket reading.** Step (zero assumptions, 17-step staircase) or linear-within-bracket (smooth, assumes uniform density inside a bracket)? | This is the *only* place a shape assumption enters, and S-7 is about exactly that. Note the distinction: it is an interpolation of a **measured distribution**, not a form imposed on a **mechanism** — but the Director owns the line. | **Linear**, on the grounds that it is a data-reading convention and the staircase would otherwise show through into gameplay as visible quantisation. Flagged, not assumed. Verify the Census's own grouped-median convention before citing it as precedent. |
| **OQ-C** | **Redistributive writes.** Is the shape-only design's *proportional* reading of a uniform per-capita drain acceptable for Phase 1, with additive-shift semantics deferred to each redistributing system's own port? | A fidelity question about the ruled measure. | **Yes, defer** (§3.3) |
| **OQ-D** | **OQ-1e's C/G/P derivation under Axiom A0.** *"a population measure over an intra-class distribution is not among A0's enumerated G-members"* (`ai/bsl-architecture-standard.md:684`, `ai/THE_FORMALISM.md:172`). This field design supplies the **carrier** and does not supply the **derivation**. | Formalism-surface question; AE (ii) territory if it turns out to need a new G-member. | Present the derivation attempt separately; do not let Phase 1 imply it is discharged |
| **OQ-E** | **GAP-1: income ≠ wealth.** ACS B19001 is a household **income** distribution; the construct is a **wealth** distribution. Accept income as the shape proxy, or blend with the 4-bracket Fed/FRED wealth data (`fact_fred_wealth_shares`, `dim_wealth_class`)? | A theoretical call about what the class's material position *is*. | Accept income as the county-level **shape**, cite it as a proxy in the manifest's `material_relation`, and record the substitution rather than hide it |
| **OQ-F** | **GAP-2: no class × income cross-tab.** A per-class-per-county bracket vector must be constructed from two marginals (income × county, class × county). Which construction? | Any construction is an assumption about how class and income co-vary — an ideological claim, not a statistical detail. Independence in particular asserts that Marxian class does **not** predict income. | Escalate with options; do **not** improvise. Independence is almost certainly the *wrong* answer and it is the one a defaulting implementation would reach for |
| **OQ-G** | **The open top bracket.** B19001's top row is "$200,000 or more" and has no midpoint. Its declared `midratio` sets where the bourgeoisie's wealth sits and therefore what I3 normalises against. | A tail decision that moves the whole grid. | Declare and cite a constant; do not let a default emerge from the arithmetic |
| **OQ-H** | **κ, the stock→flow scaling for attrition.** `failing` is a stock (who is below the line now); attrition is a flow (who dies this tick). Any constant converting one to the other is a candidate S-7 knob wearing a new name. | This is precisely `attrition_base_factor`'s replacement seat, and ADR191 R3 killed that knob. `reports/wiring-completeness-2026-07-29.md:552-555` names the identical trap for the strike-resolve term. | Present **κ = 1** (no knob) and a defines-sourced alternative side by side; let the Director decide whether κ exists at all |
| **OQ-I** | **§3.4 repair (GAP-B).** `extensive ÷ extensive → intensive` — per-capita wealth is currently `E-TYPE-040` in the spec and unenforced in the implementation (`docs/reference/bsl-language.rst:2373-2376`; `rust/crates/babylon-bsl/src/typecheck.rs:15-19`). | The bullet self-declares as *"deliberately conservative and a Phase-1 review item"*; changing it is spec work. Arguably delegable as unit algebra (D90's own standing). | Repair the bullet: split `*` from `/`; `extensive ÷ extensive → intensive`. Otherwise every per-capita quantity in the engine is illegal the day the expression typechecker lands |
| **OQ-J** | **Phase 2 sequencing.** Does this design count as the "first consumer" that un-defers Half 2 Currency field storage (`rust/crates/babylon-bsl/src/scenario.rs:40-48`)? | The deferral was a Director ruling keyed to a first consumer. | **Yes** — it is the cleanest available landing, and the alternative (#502 WS3's Real-lane operator) is larger and later |

---

## 9. Summary of the recommendation in one paragraph

Carry the sketch as **K = 16 per-class `coefficient` mass fields** (`social-class/wealth-mass-01..16`,
`:kind intensive`) over a **universal, mean-relative cut grid of `Ratio` `defconst`s**
(`wealth-sketch/cut-01..15`). This needs **no new type, no new operator, no encoder ceremony and no
amendment**: it rides `deffield`'s existing `coefficient` type, the existing section-`0x02` scalar
hash rows, the bit-pinned Half-1 seed lane, the landed `Currency × Ratio` operator and the landed
`floor` intrinsic. Because the grid stores no money, the sketch re-anchors to the scalar mean at
read time and no wealth-writing system needs to know it exists. Both consumers — Grinding Attrition
and P(S|A) — read one binding, `clearing = Σ_k mass_k · c_k`, built from multiplications, additions,
comparisons and one guarded linear fraction: pure measure arithmetic with no functional form
anywhere. Two live language gaps stand in the way and both are already-known, already-owned items:
Currency field storage (Half 2, deferred to its first consumer — this is that consumer) and §3.4's
`extensive ÷ extensive` bullet.

**As ruled (§0):** the reading is the **step** function, κ is a **moddable time-constant**, Half 2
is **un-deferred now**, and the income-shape proxy is **PROVISIONAL under issue #510** — allowed
for gameplay and development, never independence, revisited when the Director charters the
class-conditional construction.
