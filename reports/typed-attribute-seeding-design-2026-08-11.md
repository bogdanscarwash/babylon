# Typed-Attribute Scenario Seeding — Design (Program 27 Phase 2)

**Date.** 2026-08-11. **Author.** Design engineer, typed-attribute seeding train (read-only
survey against dev tip `94533b77`, no branch cut). **Purpose.** The declared Phase-2 trait
revision `docs/reference/phase-1-exit-checklist.md`'s DEFERRED table names ("Typed attribute
storage (Currency i128 exactness on node fields) | Phase 2 trait revision | `f64` attributes
cannot hold i128; Currency writes are LOUD errors today, never lossy casts") — scoped here to
what actually blocks the port lane: the scenario **loader**'s refusal to seed any field-value
combination except an integer literal into an `int`-declared field
(`rust/crates/babylon-bsl/src/scenario.rs::attribute_value`, line 653).

**Headline finding the task brief's framing gets wrong in one place.** The brief says this
train "retires the D-1 `:const` workarounds across Dispossession/Lifecycle/Metabolism" as if
uniform. It is not. Dispossession's D-1 (five rate fields) and Lifecycle's D-1/D-2 (eleven
fields) are exactly what this design retires — they are fractional-seeding gaps, full stop.
**Metabolism's own D-1 (`entropy_factor`, `content/rules/metabolism.bsl` lines 21–207) is a
different gap this design does not touch**: even with a `Ratio`-typed `defconst` (D99/ADR194),
the only legal `Ratio` operator is `Currency × Ratio`, and `entropy_factor`'s multiplicand is
`Real` (both factors are `:field` reads, always `Value::Real` — see §D below) — there is no
`Real × Ratio` operator on the language surface at all. That gap is explicitly chartered as
"workstream 3 of the post-port refactor program" (metabolism.bsl lines 178–183, GitHub issue #502)
and is **out of scope here**, exactly as edge/hyperedge field storage (ADR192) is out of scope
here. What this design DOES retire from Metabolism is its **D-2** — `biocapacity`/
`max-biocapacity`/`extraction-intensity`, already `:field`-bound but seeded today as
workaround `int`-declared fields (`metabolism-conformance.bscn` lines 25–27) standing in for
what the source-of-truth Pydantic model declares `Currency`/`float [0,1]`
(`src/babylon/models/entities/territory.py:155–172`).

Second correction, smaller: the brief cites "Territory's STOP record (blocker 2: heat/
organization fixtures need fractional seeds)" as an existing artifact to read. No such file
exists in the tree today — there is no Territory BSL port-assessment or plan document yet
(unlike Dispossession's, Lifecycle's, and Metabolism's, all of which landed one before their
`.bsl`). The underlying claim is independently true and verified below (`Territory.heat:
Intensity`, `territory.py:130–132`; `Organization.heat: Probability`, `organization.py:184`;
`bsl-gap-analysis-2026-08-10.md` row 2.0 names "heat dynamics, camp decay, heat spillover" as
Territory's three blocked families) — but it is a claim this design corroborates from source,
not a citation this design found.

---

## 0. Scope

**In scope.** Widening the scenario loader (`scenario.rs::attribute_value`) and, for Currency,
the substrate storage layer, so a `.bscn` file can seed a fractional or Currency value into a
node field of the field's own declared type, instead of every non-`int` field being an
unconditional load refusal.

**Out of scope, named so a later reader does not re-derive the boundary:**

- **Edge/hyperedge field storage** — ADR192's lane. That ruling already grants "dyadic edges
  and hyperedges MAY carry declared, typed, hash-covered fields" as machinery, with its own
  Aleksandrov-per-field gate and its own `CanonicalState` widening. This design's Half 2 (§B)
  reuses ADR192's *pattern* (additive widening, ceremony on first declaration) for **node**
  attributes, but does not touch edges or hyperedges at all.
- **`Real × Ratio` / the scaled-Int workaround class** — issue #502 workstream 3. Typed
  attribute storage does not create a `Real × Ratio` operator; Metabolism's D-1 stays exactly
  as workaround'd as it is today after this design lands.
- **Per-territory reference-data hydration** (§3.9's hydration contract — FRED-derived
  foreclosure rates, real Census legitimation data) — that is a data-build-pipeline seam
  (`docs/reference/bsl-language.rst` §3.9 clause 4), and this design only widens what the
  **scenario loader** can legally seed. Whether the eventual hydration pipeline writes through
  the same `attribute_value`-equivalent path or a different one is that seam's own decision.
- **Currency's sign domain.** `babylon_kernel::Currency` is deliberately left signed at the
  type level (`currency.rs` lines 4–12, "OPEN — flagged in the Phase-1 plan's open
  questions") even though the BSL spec table declares Currency's domain `[0, ∞)`
  (`bsl-language.rst` lines 2154–2155). §B recommends enforcing `[0, ∞)` at the typed-attribute store
  boundary, matching the spec table, but does not resolve the kernel type's own open flag.

---

## A. The two-half split

### Half 1 — fractional f64-representable lanes (Probability / Intensity / Coefficient / Real)

**Claim: seedable with no trait revision, by widening `attribute_value` alone. Verified true
at three independent levels:**

1. **The storage layer already holds arbitrary `f64`.** `GraphSubstrate::update_node`/
   `node_attribute` are `f64` in and out today (`rust/crates/babylon-graph/src/substrate.rs`
   lines 133, 142) — nothing about the trait, `MemoryGraph`, or `HypergraphStore`
   (`hypergraph_store.rs` line 77: `attributes: HashMap<(NodeId, String), f64>`) restricts
   values to integers. The restriction is **entirely** in the scenario loader's
   `attribute_value` function, which the refusal's own doc names precisely: "slice 1 stores
   only `int`-declared fields ... the scaled and Currency lanes need typed attribute storage"
   (`scenario.rs` lines 659–666). That sentence conflates two different gaps under one
   refusal; Half 1 is the half that needs no typed storage at all.
2. **The runtime write path already enforces exactly the domain check Half 1 needs, and Half
   1 should mirror it verbatim.** `structural_verbs.rs::store_range_check` (lines 707–728)
   already refuses a runtime `update-node` write outside `[0,1]` for a Probability/Intensity/
   Coefficient-declared field, loud, `E-EVAL-020`, never a clamp. Widening `attribute_value`
   to accept these three types should apply the **identical predicate**
   (`matches!(decl.ty, BslType::Probability | BslType::Intensity | BslType::Coefficient)` then
   `(0.0..=1.0).contains(&value)`) at load time — not a new invented rule, the same rule the
   runtime store boundary already lives by, just checked one call frame earlier. `Real` gets no
   range check at either boundary today (§3.3: "the unbounded intermediate type"; not
   independently storable per §3.1's table — a `deffield` cannot declare `real` as a type at
   all, confirmed against the table's six `<type-name>` rows, `bsl-language.rst` lines 2148–2165)
   — so `Real` is not actually a fourth seedable *declared* type, only the runtime lane every
   arithmetic result promotes into (§3.3).
3. **`CanonicalState`/`state_hash.rs` is untouched.** The canonical byte layout's attribute
   section (`0x02`) is already `u64 id ‖ str name ‖ u64 value-bits` over an `f64`
   (`state_hash.rs` lines 20–21, 141–152) regardless of what BSL-level type declared the field.
   A `0.358` seeded into a `coefficient`-declared field and a `0.358` seeded into a
   (hypothetically legal) `int`-declared field hash **byte-identically** — the section's shape
   does not depend on `BslType` at all, only on the stored `f64`. Widening `attribute_value`'s
   accepted `(value, decl.ty)` combinations therefore changes **zero** bytes of the encoding
   for any existing scenario (every existing `.bscn` seeds only `int` fields, and the `int`
   arm of `attribute_value` is untouched) — Half 1 is provably additive at the hash level,
   with no ceremony, no golden move, and no `qa:regression`/vault consequence.

**The quantization contract, precisely, correcting an assumption in the task brief.** The
brief frames this as "a decimal literal like `0.358` must enter storage as the exact same
f64 the evaluator's 1e-6-grid lanes would produce." Verified against the actual code: **there
is no 1e-6-grid lane in the BSL evaluator today.** `babylon_kernel::grid::quantize` (the
`(value * 1e6 + 0.5).floor() / 1e6` half-up law, `grid.rs` lines 17–32) is invoked in exactly
one place in the entire `babylon-bsl` crate: `reader.rs`'s `Ratio`-literal legality check
(does an `r`-suffixed literal quantize to zero, `E-LEX-027`). It is **not** invoked when a
`p`/`i`/`c` literal is lexed (`reader.rs::classify_unit_interval`, lines 761–806, is a plain
`[0,1]` range check over up to 9 exact fractional digits, no grid snap), **not** invoked when
a scaled literal becomes a `Value::Real` (`tick.rs::atom_to_value` line 264 and
`scenario.rs::load_defconst` line 311 both compute `unscaled as f64 / 10f64.powi(scale)` —
one IEEE-754 division, nothing else), and **not** invoked at the runtime store boundary
(`numeric_write_value`/`store_range_check`, `structural_verbs.rs` lines 668–728, check
finiteness and `[0,1]` range only). The 1e-6 grid is exclusively a `babylon_kernel` **scalar
newtype** invariant (`scalars.rs`'s `Probability::new`/`Intensity::new`/`Coefficient::new`/
`Ratio::new`, each calling `quantize` on construction) — a code path nothing in
`babylon-bsl` currently reaches, because nothing in the BSL crates constructs a kernel scalar
newtype from a graph-read `f64`; the evaluator's binary64 lane is raw `f64` (`Value::Real`)
end to end.

**Recommendation: do not invent a grid-snap for the seed path alone.** Doing so would create
exactly the asymmetry it is trying to prevent — a value entering storage via
`(update-node self field (+ x y))` during a tick is not grid-quantized today, so if the seed
path alone snapped to the grid, a rule-loaded-then-written value and a scenario-seeded value
of the identical field would follow two different rules for the same declared type. **The
seed-path contract Half 1 should state is the one that already governs every other BSL
numeric literal**: the stored value is the single, correctly-rounded IEEE-754 result of
`unscaled / 10^scale` (up to 9 fractional digits, `ScaledLit`'s own bound, `reader.rs` line
107), identical byte-for-byte to what a `:const`/`:default` binding of the same literal
already produces via `Value::Real`. That conversion is a **basic IEEE-754 operation** (a
single division of two exactly-representable operands — `unscaled` as an integer, `10^scale`
exact in `f64` for `scale ≤ 22`), which reproduces bit-identically across conforming
implementations (the same class of guarantee `grid.rs`'s own cross-language conformance
vector relies on, and the class CLAUDE.md's Tests-as-Behavioral-Contracts principle 4 already
names as safe: "basic IEEE-754 ops reproduce across languages"). No new rule is minted; the
existing rule is extended verbatim to the node-attribute seed path. **Watch item, not a
blocker**: if a future intrinsic host or native Rust system ever constructs a
`babylon_kernel::Intensity`/`Probability`/`Coefficient` directly from a graph-read attribute,
that construction's own `quantize()` call could silently re-round a value seeded with more
than 6 significant fractional digits, diverging from the hashed value. Flagged for whoever
builds that bridge (mirroring `E-EVAL-041`'s own "defense in depth... re-check at the point of
use" discipline) — unreachable today, so not this design's problem to solve.

### Half 2 — Currency (i128 micro-units)

**Genuinely needs typed storage — not an extension of Half 1's fix.** Two independent reasons,
both grounded in code already in the tree:

1. **Exactness.** `Currency` is `i128` micro-units (`currency.rs` line 36); `f64` is exact
   only to `2^53` (≈9.007×10¹⁵), which bounds exact-integer *micro-units* to about $9.007
   billion — inside county-scale headroom today, but the exactness argument
   `attribute_value`'s own `Int` arm already makes for `i64` (lines 670–672: "Exact in f64 to
   2^53; past that the stored value would differ from the declared one, which is a lie the
   state hash would faithfully record") applies with a **narrower** headroom to `i128`.
2. **Rounding semantics, the deeper reason.** Every `Currency`-mixed operation in §3.2's table
   is pinned to a specific rounding law — half-even, at a specific intermediate width (`i256`
   for `Currency ÷ Currency`) — that a raw `f64` round-trip cannot preserve once a value has
   passed through even one arithmetic step: IEEE-754 binary rounding and the spec's
   half-even-on-the-micro-unit-grid rounding are not the same rounding, and they diverge after
   composition even when they agree on the first operation. This is why `attribute_value`
   refuses Currency outright today (line 681) rather than merely widening its integer-exactness
   check — the gap is not "we haven't raised the exactness ceiling yet," it is "an `f64` cannot
   carry the operator table's own guarantees across a write/read round trip."

**No candidate shape stores Currency as two f64 halves — rejected, per the prompt's own
instruction to reject it loudly.** A hi/lo or dollars/micro-remainder split reintroduces the
identical `2^53`-exactness ceiling on whichever half carries the larger magnitude (a
dollars-component past ~9 quadrillion micro-units is exactly the failure this design exists to
retire, just relocated), and it invents a **second, un-normative cross-language contract**
(how the two halves recombine into one `i128`) duplicating the work of specifying `i128`
storage directly, with strictly more surface for a bug. Rejected without further analysis.

---

## B. The trait-revision shape for Half 2

Two candidate shapes, plus the rejected third from §A:

### Shape 1 (recommended) — a parallel typed-attribute surface on `GraphSubstrate`

Two new trait methods, additive to the existing 14 (`substrate.rs`'s own module doc already
counts them as 14):

```rust
fn update_node_currency(&mut self, id: NodeId, attribute: &str, value: Currency)
    -> Result<(), GraphError>;
fn node_attribute_currency(&self, id: NodeId, attribute: &str)
    -> Result<Currency, GraphError>;
```

backed by a second map per store (`HashMap<(NodeId, String), Currency>`, sitting alongside
`attributes: HashMap<(NodeId, String), f64>` unchanged in both `MemoryGraph` and
`HypergraphStore`). `CanonicalState` gains a **fifth** section, `0x05`, `u64 id ‖ str name ‖
i128 value (16 bytes, big-endian)`, sorted `(id, name)` exactly as section `0x02` is, emitted
after `0x04` so existing readers' section-tag dispatch is untouched. **Existing content
(nothing declares a `currency`-typed field today) encodes with an empty `0x05` section — four
bytes, `u32 count = 0` — appended after today's four sections.** That is a **byte-shape
change to every existing golden** (the trailing `0x05 00000000` is new bytes even for zero
Currency attributes), which is the one place this shape is NOT free — see the ceremony note
below.

**`encode_state`'s provided default keeps doing all the sorting**, exactly as it already does
for the other four sections (`state_hash.rs` lines 257–283) — a fifth `write_currency_attributes`
call added to the same provided method, no new per-store logic beyond the two new listing
methods (`all_currency_attributes`, mirroring `all_attributes`).

### Shape 2 (rejected as this train's shape, correct as a LATER shape) — widen the attribute value to an enum

Replace `f64` with `AttributeValue { F64(f64), Currency(Currency) }` across the trait's two
existing methods (`update_node`, `node_attribute`). This is not additive: it changes the
signature of two methods every existing caller uses (`scenario.rs`, `structural_verbs.rs`,
`tick.rs::bind_subject`), forces every implementor (`MemoryGraph`, `HypergraphStore`, any
future store) to re-type its attribute map, and — because the *shape* of every attribute row
changes (a tag byte minimum, even for a plain `f64`) — re-encodes **every existing attribute
row in section `0x02`**, not just adds a new empty section. Every stored golden hash moves,
not just gains four trailing bytes. Reject for this train: strictly more blast radius than
Shape 1 for the identical behavioral outcome, and it forecloses ever giving Currency its own
section-level ceremony discipline the way ADR192 gives edge/hyperedge fields theirs. (It could
still be the right shape for a hypothetical *third* typed lane arriving later, if the
per-lane-parallel-map pattern stops scaling — not a call this design needs to make.)

### Which goldens move, and under what ceremony

Shape 1 moves **every existing golden hash** the moment it lands, even though no content
declares a Currency field yet — because the empty `0x05` section is still four new bytes in
the encoding. This is the one place Half 2 is **not** free, unlike Half 1. The repo's own
`test(baselines): …` ceremony (CLAUDE.md "Definition of done," `Baselines: blessed(<slug>)`
trailer, `tools/generate_ceremony_message.py`) is the mechanism — this is a **mechanical**
drift (every cell's hash changes, none of its *value*), so the ceremony's drift table will
read as "every baseline moved, zero semantic deltas," which the ceremony format already
supports (it records attribution, and "new empty section" is a one-line attribution repeated
identically across every scenario). **Recommendation: land Shape 1's trait/encoding change
and its baseline ceremony as its own PR, separate from and before the first Currency-typed
`deffield`** — isolating "the encoding widened" from "content started using it" mirrors
ADR192's own clause 3 verbatim ("content declaring no relation fields encodes byte-identically
... the ceremony fires when content FIRST DECLARES such a field, not when the machinery
lands") *except* that ADR192's edge/hyperedge widening achieves TRUE zero-golden-movement
(a new section only appears once a hyperedge or edge actually carries a field), while Shape 1
here cannot: an *empty* `0x05` section still writes four bytes for every scenario, because
`CanonicalState::encode_state`'s four-then-five-section shape is unconditional. If literal
byte-parity with ADR192's discipline is wanted, the alternative is to make section `0x05`
**conditionally emitted** — omitted entirely when a store's Currency-attribute set is empty —
which restores true zero-movement for every currently-Currency-free scenario at the cost of a
single `if !currency_attrs.is_empty()` branch in `encode_state`. **Recommended over the
unconditional variant above** for exactly this reason: it makes Half 2's landing PR truly
byte-free until content opts in, matching the precedent this design is explicitly modeling
itself on.

---

## C. What is Director-gated

One item rises to a genuine reserved-line decision; everything else below it is settled
engineering under an existing ruling, cited.

### Gated — sequencing: does Half 2 land now, unconsumed, or wait for its first real consumer?

Even with the conditional-section fix (§B), Half 2 is real, non-trivial surface (a second
per-store map, two new trait methods, a fifth `CanonicalState` section, a `bind_subject`
plumbing change — §D) landing with **no BSL content that uses it**, since nothing in
`content/rules/*.bsl` declares a `currency` field today (every pack currently fakes Currency
fields as `int`). This sits directly against two standing, previously-ruled positions that
pull opposite ways:

- *For landing it now:* `feedback_full_vision_no_mvps.md` — "never propose MVP/Phase-1
  splits of fully-specced features" — and the Phase-1 exit checklist already scoped "Typed
  attribute storage" as one Phase-2 item, not two.
- *For deferring it:* the D99/#492 addendum's own precedent — "Porting these five rules to
  BSL is explicitly OUT of scope for the change that adds this addendum — it lands the
  machinery only, in its own right-sized, reviewable unit; each consumer ports in its own
  train" (`bsl-language.rst` line 2293) — landed the OPERATOR unconsumed, deliberately, in
  living memory, one session before this design.

This is presented as an open question rather than resolved by citation, because the two
precedents disagree and neither is more authoritative than the other:

**AskUserQuestion-ready:**

> Typed Currency-attribute storage (Half 2) can land two ways. Which do you want?
>
> - **Option A (recommended): sequence it.** Ship Half 1 alone in this train — it unblocks
>   Territory immediately and is byte-free (§A). Charter Half 2 as its own follow-on train,
>   opened by whichever port first needs a real Currency field (Dispossession's
>   `territory/wealth`, Lifecycle's `wealth-d-prime`, or Metabolism's `biocapacity`/
>   `max-biocapacity` are the three live candidates — all three are `Currency`-typed in the
>   frozen Pydantic source and all three are seeded as `int` workarounds today). Mirrors the
>   D99/#492 precedent exactly.
> - **Option B: land both halves together, now.** Keeps the "declared Phase-2 trait revision"
>   as one delivered unit matching the exit checklist's single line item, and gets the
>   ceremony-movement pain (§B) over in one PR instead of two. Costs: a second train's worth of
>   surface (new trait methods, new store map, new `CanonicalState` section, `bind_subject`
>   plumbing) ships with zero consumers to prove it against beyond hand-written tests.

### Settled — everything else, cited

- **Half 1's shape needs no ruling.** It is provably additive at the hash level (§A, point 3),
  it is exactly what Dispossession's and Lifecycle's own D-1 records already anticipated
  verbatim ("these five become genuine per-territory `:field` reads with NO OTHER CHANGE...
  When real per-county hydration lands (Phase 2's Currency/Probability-typed field storage)",
  `dispossession.bsl` lines 41–44), and it mints no new primitive, no new algebra, no new
  BSL grammar.
- **Half 2's SHAPE choice (Shape 1 over Shape 2) needs no ruling.** Phase-1 exit checklist's
  own words on the sibling storage-pick decision apply verbatim: "The trait is the insulation
  layer; a storage pick is an engineering choice under a settled shape" (line 41). ADR191 R8
  ruled the identical class of question — "NO constitutional text required for hypergraph-rs
  engine-storage adoption" — for the hyperedge-storage pick; the same reasoning covers the
  Currency-storage pick.
- **Widening the canonical carrier set at all needs no NEW ruling — ADR192 already ruled the
  general pattern**, one relation-carrier lane over: "dyadic edges and hyperedges MAY carry
  declared, typed, hash-covered fields, widening the canonical carrier set beyond {node,
  node-attr, edge-strength, hyperedge-members}" (ADR192 decision clause 1), on the warrant that
  the addition "mints no new mathematics and no new primitive." A sixth canonical carrier
  (`{node, node-attr, edge-strength, hyperedge-members, [edge-attr, hyperedge-attr — ADR192],
  node-currency-attr — this design}`) is the same class of move ADR192 already licensed, not
  a new one. This design treats ADR192 as the standing precedent for "how a canonical-carrier
  widening gets ceremonied," not as literal authority over node attributes (ADR192's text is
  scoped to edges/hyperedges) — flagged so a reviewer can independently judge whether that
  extension-by-analogy is sound, rather than this document asserting it is self-evidently
  covered.
- **Enforcing `[0, ∞)` at the Currency-attribute store boundary needs no ruling.** It is a
  direct application of the BSL spec's own declared domain for the type (`bsl-language.rst`
  table row: `currency | i128 micro-units, [0, ∞)`, lines 2154–2155), mirroring
  `store_range_check`'s existing precedent for the three unit-interval types
  (`structural_verbs.rs` lines 707–728). Recommended, not gated.

---

## D. The read path

**What Value lane a `:field` read enters today, verified against `tick.rs`, not assumed.**
`bind_subject` (`tick.rs` lines 179–254) resolves a `BindSource::Field(qname)` binding at line
237: `graph.node_attribute(subject, qname).map(Value::Real)` — **every** field read is wrapped
`Value::Real`, unconditionally. `bind_subject`'s signature takes no `TypeEnv` at all (line
180–184: `subject, bindings, graph, defines, tick` — no `types` parameter), so it is
**structurally incapable** of consulting a field's declared `BslType` even if it wanted to.
This is true for a field declared `int` too — `social-class/wages` is `int`-declared in every
existing scenario, and a `:field wages` binding still reads back as `Value::Real`, never
`Value::Int`. Confirmed directly in the test suite's own assumption (`dispossession.bsl`'s
comment at lines 280–290, on why its D-4 clamp binding adds `(- 0 0c)` rather than returning a
bare `:const`: "readable correctly by `real_lane` either way" — the code already treats
`Value::Real` as the field-read lane's universal, type-blind output).

### Verifying the D-1 claim: "they slot into the SAME positions... with NO OTHER CHANGE"

**True for Half 1, with one caveat; false as stated for two named conformance-fixture classes.**

For an **in-domain** value, the claim holds exactly, and Half 1 needs zero `tick.rs` changes:
a `:const` binding of a scaled `c`/`p`/`i` literal already produces `Value::Real` via
`atom_to_value` (line 264); a `:field` binding of the same value, once Half 1 lands, produces
`Value::Real` via `bind_subject` line 238. Same runtime lane, same downstream arithmetic —
the swap really is a one-line binding-source edit, exactly as claimed.

**The caveat: bare, unsuffixed `Int` `:const` literals used as deliberate domain-check
bypasses cannot make the same swap.** Dispossession's own D-2/D-4 records document, correctly,
that a bare `Int` `:const` "carries NO domain check at all" (`E-LEX-024` bounds only
suffixed literals) — and four of its own conformance scenarios exploit exactly that gap on
purpose: `dispossession-ceiling-matrix-conformance.bscn` (`eviction-rate 6`,
`displacement-rate 8`, `concentrated-ownership 9`, `absentee-landlord-share 4`) and
`dispossession-negative-input-conformance.bscn` (`foreclosure-rate 5`, `eviction-rate -3`,
`displacement-rate -8`, `concentrated-ownership -2`, `absentee-landlord-share -9`) — deliberately
out-of-`[0,1]`-domain values, seeded specifically to prove the rule's own in-body clamps
(D-2/D-3/D-4) actually fire. §A's Half 1 recommendation is to mirror `store_range_check`'s
`[0,1]` enforcement at seed time — which means, once Half 1 lands, a properly `probability`-
or `intensity`-declared field **cannot** be seeded with `6`, `-3`, or any other out-of-domain
probe value; the loader would refuse it (a new load-time error, analogous to `E-LOAD-052`'s
Ratio-bound check, not yet numbered — `E-LOAD-053` is next-free at time of writing per a
sweep of every `E-LOAD-0xx` code currently in the tree). **This is correct and desired**
(domain-typed storage should refuse an out-of-domain seed — that is the whole point of
declaring the type), but it means these four adversarial scenarios must **keep** using
`:const`'s domain-blind escape hatch even after Half 1 lands, permanently, by design — they
cannot "become genuine per-territory `:field` reads" the way the D-1 header's general
aspiration describes, because what they are testing is specifically unreachable through a
properly-typed field. A later reader migrating Dispossession's `:const` bindings to `:field`
should keep these two scenarios' five rate consts as `:const`, and migrate only the
in-domain scenarios (`dispossession-conformance.bscn`, `dispossession-zero-rate-conformance.bscn`,
`dispossession-single-rate-conformance.bscn`, `dispossession-saturation-conformance.bscn`,
`dispossession-negative-weight-conformance.bscn`) — five of the pack's seven conformance
scenarios, not all seven. (The header's own count of "four more, added in the adversarial-review
fix round" names four scenarios total in that round — `saturation`, `negative-input`,
`ceiling-matrix`, `negative-weight` — of which only `negative-input` and `ceiling-matrix` are
actually out-of-`[0,1]`-domain; `saturation` and `negative-weight` probe the domain's own
closed boundary (`1c`/`0c`) and stay legally in-range.)

### Half 2's consequence for the read path — a real plumbing change, not a no-op

For Currency, the "no other change" claim does not hold even in the best case, because
`bind_subject` cannot dispatch to `node_attribute_currency` vs `node_attribute` without
knowing the field's declared type — and it currently has no access to one at all. The fix is
small and mostly already in place: `run_tick` (`tick.rs` line 356) **already receives**
`types: &TypeEnv` as a parameter (used today only to construct `EffectExecutor::new(types)` at
line 415) but does not forward it to `bind_subject`'s call at line 373. Threading `types`
through `bind_subject` and switching on `types.fields.get(qname).map(|d| &d.ty)` to choose
`node_attribute` (wrap `Value::Real`) vs `node_attribute_currency` (wrap `Value::Currency`) is
the whole change — no new parameter needs to be invented at the `run_tick`/`run_once_into`
call sites, since `prepared.types` (`babylon-tick/src/lib.rs` line 209) is already constructed
and already flows to `run_tick` today.

---

## E. Sequencing

Plan-ready granularity, assuming §C Option A (Half 1 now, Half 2 deferred):

1. **Widen `attribute_value`** (`scenario.rs`) to accept `Probability`/`Intensity`/
   `Coefficient`-declared fields, applying the `[0,1]` domain check mirrored from
   `store_range_check`. `Int`'s existing arm is untouched (no behavior change for any field
   declared `int`). Currency's refusal arm is untouched (still refused, same message).
   *Proves*: the existing test `a_currency_attribute_is_refused_not_cast`
   (`scenario.rs` line 849) still passes unmodified — Currency's refusal must survive Half 1
   verbatim, since Half 2 is deferred.
2. **New unit tests, mirroring the existing `attribute_value` suite's own style**: a `p`/`i`/
   `c` literal seeds correctly into a matching-typed field; an out-of-`[0,1]` literal (bare
   unsuffixed `Int`, or an in-range-suffix-but-wrong-magnitude case — there is no
   out-of-range-suffixed case, `E-LEX-024` already refuses that at lex time) into a
   Probability/Intensity/Coefficient field is refused at load, loud, naming the field; a
   `Real`-typed `deffield` is impossible to write at all (confirms §A point 2's finding that
   `Real` is not a legal `<type-name>`, so this is a grammar-level refusal already covered by
   existing `deffield` type-name tests, not new).
3. **Byte-identity regression, the "provably additive" claim made concrete.** A test that
   loads a scenario seeding one `int` field and one new `coefficient` field, computes
   `state_hash()`, and separately hand-constructs the expected `0x02` section bytes for both
   rows via `StateEncoder` directly (mirroring `state_hash.rs`'s own
   `the_canonical_encoding_is_pinned_byte_for_byte` test) — proving no format branch was
   introduced by field type. A second test re-runs every EXISTING scenario fixture
   (`dispossession-*.bscn`, `lifecycle-*.bscn`, `metabolism-*.bscn`, `vitality-*.bscn`) and
   asserts each `state_hash()` is unchanged from its pre-change value — the actual
   "provably additive" proof for real content, not just a synthetic fixture. This is the gate
   that should block the PR if it fails; per §A's analysis it should pass trivially (no
   existing scenario seeds anything but `int`), which is itself the point of running it.
4. **Migrate Lifecycle's eleven D-1/D-2 `:const` bindings to `:field`**, five of Dispossession's
   seven `:const` conformance-scenario declarations (the in-domain ones — §D), and Metabolism's
   D-2 three
   fields (`biocapacity`/`max-biocapacity`/`extraction-intensity` — note `biocapacity`/
   `max-biocapacity` are `Currency`-typed in the source model and can only become properly
   typed with Half 2; **this step upgrades their `deffield` declaration from `int` to
   `intensity`/`coefficient` where the field is genuinely unit-interval
   (`extraction-intensity`), and leaves `biocapacity`/`max-biocapacity` as `int` workarounds
   still, pending Half 2** — a partial win, stated as such, not oversold). Each migration is
   its own small PR per pack (three packs, three PRs), each re-running that pack's full
   conformance suite and asserting unchanged output — the migration is a storage-representation
   change only; the D-1/D-2 header comments retire in the same PR that migrates their fields
   (the record no longer describes current behavior once the field is genuinely per-node).
5. **Territory port re-enters here.** Once step 1 lands, Territory's `heat` (`Intensity`,
   confirmed `territory.py:130–132`) and any per-territory Probability/Intensity/Coefficient
   fields the eventual Territory BSL plan needs are seedable. This design does not write that
   plan — the gap-analysis row (2.0, `bsl-gap-analysis-2026-08-10.md`) still names three
   blocked families (heat dynamics, camp decay, heat spillover) and "eviction routing has no
   expressible form" as **separate**, unresolved blockers this train does not touch; typed
   seeding removes exactly one blocking dependency (fractional field storage), not all of
   them. A Territory port-assessment document, matching the Dispossession/Lifecycle/Metabolism
   convention, is the correct next artifact before any Territory `.bsl` is written — outside
   this design's scope to produce.
6. **(Deferred, per §C Option A) Half 2's own train**, opened by whichever of
   `territory/wealth`, `lifecycle/wealth-d-prime`, or `metabolism/biocapacity` +
   `max-biocapacity` first needs real Currency fidelity — landing the trait methods, the
   second per-store map, the conditionally-emitted `0x05` `CanonicalState` section (§B), the
   `bind_subject` type-aware dispatch (§D), its own baseline ceremony (empty-section
   byte-movement, even with zero consumers, per §B's analysis), and only then migrating its
   one triggering field.

---

## What this document does not cover, restated

Edge/hyperedge field storage (ADR192's own lane, untouched); the `Real × Ratio` operator gap
(issue #502 WS3, untouched — Metabolism's D-1 is not retired by this design); per-territory
reference-data hydration's own bind-src/pipeline question (§3.9's seam, a Phase-2/3 content
question independent of what the loader can legally store); Territory's own port plan (named
as the next artifact, not written here); Currency's kernel-level sign-domain openness
(`currency.rs`'s own flagged question, orthogonal to whether the BSL store boundary enforces
`[0,∞)`, which this design does recommend).

---

## Rulings postscript

Director ruling 2026-08-11 (popup): Half 2 Currency i128 typed storage DEFERRED TO FIRST
CONSUMER — Half 1 proceeds as settled engineering; both halves remain fully specced in this
document; this is train sequencing, not scope reduction.
