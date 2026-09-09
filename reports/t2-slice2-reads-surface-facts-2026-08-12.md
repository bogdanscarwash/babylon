# T2 Slice-2 Reads Surface Facts — Verified 2026-08-12

Scout pass for Program 29 train T2 (issue #559): BSL query-evaluation Slice 2 — dyadic edge
reads (`edges`, `edge-between`, `field-of` over an `EdgeRef`; hash-free, strength-only until T3
lands edge-attribute storage). Read-only (`Read`/`Bash rg` only, no cargo, no writes outside this
file) against `dev` in the main checkout (`/home/user/projects/game/babylon`). The worktree
`/home/user/projects/game/wt-query-eval` was not touched.

**Provenance.** `dev` moved once during this pass: PR #571 ("production-port-bsl") merged mid-session.
Every file read here was read live off the working tree, so every citation below reflects `dev` at
`2508e9bc` (post-#571), not a stale snapshot — there is nothing to reconcile the way the Production
dossier had to. Started at/after `93363c7b` as the task specified.

**Executive summary (10 lines).** Slice 2's *static* surface — grammar arity (`edges`/`edge-between`/
`the` all in `grammar.rs::ARITIES`/`ENUM_REF_POSITIONS`), the §3.7 fuel-cost rows, the §3.4
kind-legality machinery, the `ScoreClass::EdgeReference` classification, and a full load-time
conformance suite (`r9_chapters.rs`) — is **already built and tested**; what is missing is purely
*evaluation*: no `Value::EdgeRef` variant exists, `query::Element` has one variant (`Node`), and
`GraphSubstrate` has no edge-attribute **read** method (it has `edges()`, which strips strength).
ADR197 itself already scoped slice 2 as "a new `Value::EdgeRef`/`EdgeKey` type and one read-only
edge-attribute lookup, hash-free" — this pass confirms that scope exactly, against the four
independent adjudications the port-estate survey cites making the same correction to ADR197's own
text. `MemoryGraph` and `HypergraphStore` both already store edges as
`HashMap<(String, NodeId, NodeId), f64>`, so the new method is a one-line getter on both, mirroring
the `node_type_of` precedent (III.7 read-only widening, zero `CanonicalState` bytes moved) exactly.
The one genuine landed-code gap this pass found and the prior inventories did not: the implicit
`<edge-type>/strength` field (D32) is fully built and tested in `declarations.rs::FieldRegistry`
but **that registry has zero production callers** — the live `TypeEnv` scenarios actually typecheck
against (`babylon-tick/src/lib.rs:127`) is built from `scenario.fields` alone, and the §3.4
kind-checker's `resolve_field` hard-fails "unknown field" on anything not in it. T2 must therefore
either wire the implicit field into that pipeline or require every content pack to hand-declare
`<edge-type>/strength` (diverging from D32's "needs no `deffield`" promise). Solidarity — T2's named
"outright" unblock — is only fully unblocked if its port uses the self-anchored
`neighbors`+`edge-between` idiom §3.8 item 8 itself demonstrates; a naive `for-each` over
`(edges EdgeType/SOLIDARITY)` needs an edge-endpoint accessor (`source-of`/`target-of`) that is an
**open, unscoped absence** (§3.8 item 8) — not in T2's charter, not in T3's, not anywhere.

---

## 1. The three heads, spec promise → refusal site → what serving requires

### 1.1 `edges` (query head, §2.6)

**Spec promise**, `docs/reference/bsl-language.rst:945,994-997`:
> `"(" "edges" <enum-ref> <edge-pred>? ")"` … Result `EdgeSet`, `it` is `EdgeRef`, ranges over
> "Every dyadic edge of the given `EdgeType`."

Iteration order (`bsl-language.rst:1040-1046,1064-1070`): ascending `(source-id, target-id,
edge-type)` lexicographic byte order — for a single-type query this degenerates to ascending
`(source-id, target-id)`, which is exactly what the substrate already returns (below).

**Current refusal**: `query.rs::UNSERVED_QUERY_HEADS` (`rust/crates/babylon-bsl/src/query.rs:76-81`)
names `edges` → `"slice 2"`; `materialize()`'s match falls through to that table
(`query.rs:110-124`). `evaluator.rs::UNSERVED_EXPRESSION_HEADS` (`evaluator.rs:518-527`) independently
names it for the bare-`<expr>`-position shape-error case (§2.6's own production has no `<query>`
alternative under `<expr>`, so reaching `edges` through generic dispatch is a grammar-shape bug, not
an unimplemented seam — `evaluator.rs:529-541`'s own doc explains the two tables answer different
questions).

**What serving it requires**:
1. `query::Element` gains an `Edge(EdgeKey)` variant. `EdgeKey` does not exist yet anywhere in the
   crate (`query.rs:16-20`'s own doc: *"`Edge(EdgeKey)` … `EdgeKey` is not a type this codebase has
   minted"*). Because `GraphSubstrate` mints no `EdgeId` (only `NodeId`/`HyperedgeId` exist,
   `substrate.rs:33,41`), an edge's identity is structurally the `(edge_type, source, target)` triple
   itself — `bsl-language.rst:1896-1904`'s own ruling: *"`edge-between` is well-defined because the
   triple is a key … 'the edge between a and b of type T' denotes at most one element."* The natural
   `EdgeKey` shape is therefore `{edge_type: String, source: NodeId, target: NodeId}` (or an interned
   equivalent), not a wrapped integer.
2. `Element`'s `#[derive(Ord)]` (`query.rs:38-49`) becomes **cross-variant lexicographic by
   declaration order** the moment `Edge` is added — the module's own comment flags this as a decision
   that "MUST be pinned against the spec, explicitly, BEFORE that variant lands." In production, no
   single `materialize()` call ever returns a mixed `Vec<Element>` (each of the six query heads
   returns one homogeneous kind), so the derived cross-kind order is not empirically exercised by any
   landed caller — but the crate's own compile-time trip-wire
   (`query.rs::tests::element_kind_name`, an exhaustive match with no wildcard, `query.rs:496-499`)
   forces a conscious decision at the point `Edge` is added regardless. T2 should pin this explicitly
   rather than accept the derive silently, per the module's own standing instruction.
3. `materialize_edges` (new function, sibling to `materialize_nodes`) calls
   `GraphSubstrate::edges(edge_type)` (already exists, `substrate.rs:164-166` — see §3 below) and maps
   each `(NodeId, NodeId)` pair to `Element::Edge(EdgeKey { edge_type, source, target })`. No new
   fuel/ceiling work: `cost(query) = 1 + cost(element predicate, if any)` (`bsl-language.rst:2743`)
   already applies uniformly; `ceiling(edges)` is the manifest ceiling of the queried `EdgeType`
   (`bsl-language.rst:2813` — already required infrastructure since `neighbors` uses the same
   `EdgeType` ceiling today).
4. `<edge-pred>` (the optional second operand, §2.6 grammar) is, like `nodes`' `<node-pred>`, a real
   grammar production with **zero exercised conformance vectors** — `materialize_nodes` already
   refuses its own predicate operand loudly by name rather than guessing at an unreviewed reading
   (`query.rs:127-133,145-152`); `materialize_edges` should mirror that refusal for symmetry rather
   than attempt to serve it.

### 1.2 `edge-between` (accessor, §2.10)

**Spec promise**, `bsl-language.rst:1817,1834-1837`:
> `"(" "edge-between" <enum-ref> <expr> <expr> ")"` … Result `EdgeRef` … "The edge of the given
> `EdgeType` from the first node operand to the second. Absence is `E-EVAL-034`."

Well-definedness (`bsl-language.rst:1896-1908`): the `(source, target, type)` triple is a **key** —
`add-edge` on an existing triple is `E-EVAL-031` and hydration seeding one triple twice is
`E-LOAD-044` — so "the edge between a and b of type T" denotes at most one element, never a set and
never a silent no-op on absence.

**Current refusal**: same two tables as `edges` (`evaluator.rs:519-521`, and grammar-level it already
parses/arity-checks — `grammar.rs::ARITIES` pins it at exactly 3 operands, `grammar.rs:636-647`;
`grammar.rs::ENUM_REF_POSITIONS` pins its first operand as `EdgeType`, kind-checked `E-TYPE-011`,
`grammar.rs:197-206`). There is no `eval_edge_between` function anywhere in `evaluator.rs` yet — it is
absent, not merely gated.

**What serving it requires**:
1. A new `eval_edge_between` function in `evaluator.rs`'s accessor family (sibling to
   `eval_field_of`): evaluate the two node-expr operands to `NodeId`s (refusing non-`NodeRef` values
   the same way `materialize_neighbors`' source operand does, `query.rs:182-190`), then look the
   triple up against the substrate's new edge-attribute-read method (§3). Absence → `EvalCode::NoSuchEdge`
   → `"E-EVAL-034"` — **already minted** in the `EvalCode` enum and its `spec_code()` match
   (`evaluator.rs:157-158,207`); nothing to add there, only to reach it.
2. Cost: `cost(edge-between) = 1 + Σ cost(operands)` (`bsl-language.rst:2750`) — a keyed lookup, never
   multiplied by a ceiling (`bsl-language.rst:2773-2781`). This is **already tested** at the
   static-cost level: `r9_chapters.rs::edge_between_costs_one_plus_its_two_endpoint_operands`
   (`r9_chapters.rs:370-382`) asserts `cost("(edge-between EdgeType/SOLIDARITY self other)") == 3`
   and the §3.8-item-8 worked shape (`(field-of (edge-between EdgeType/SOLIDARITY it self)
   solidarity/strength)`) totals correctly already — only the runtime evaluator needs to agree with a
   cost model that is already pinned and tested.
3. `edge-between`'s own §6.2 required-vector family (chapter C2, `bsl-language.rst:4193-4203`) names:
   resolving successfully, failing to resolve (`E-EVAL-034`), an enum-ref naming a `NodeType`
   (`E-TYPE-011` — **already tested**, `r9_chapters.rs::an_edge_between_naming_a_node_type_is_e_type_011`,
   `r9_chapters.rs:430-434`), and the unresolvable case
   (`r9_chapters.rs::an_unresolvable_edge_between_is_e_eval_034`, `r9_chapters.rs:463-472` —
   currently pinned at whatever level that test operates at pre-evaluation; T2 turns this into a real
   evaluation-level vector).

### 1.3 `field-of` over an `EdgeRef` (accessor, §2.10)

**Spec promise**, `bsl-language.rst:1829-1833`: `field-of`'s result is "the field's declared type",
reading "A declared field of the node, edge or hyperedge the `<expr>` denotes." The shared discipline
(`bsl-language.rst:1856-1880`) — qname carries the type annotation, absence is never a value, kind/type
propagate from the declaration — is stated once for all three referent kinds, not specially for
`EdgeRef`.

**Current refusal**: `eval_field_of` (`evaluator.rs:1216-1242`) matches `Value::NodeRef` and
`Value::HyperedgeRef` (the latter always refusing — a hyperedge carries no attributes of its own,
`evaluator.rs:1231-1236`) and falls through to a generic "not a reference … edge referents ride slice
2" message for anything else (`evaluator.rs:1237-1240`) — reachable today only because no expression
form yet produces a `Value::EdgeRef` at all (the doc comment at `evaluator.rs:1209-1215` states this
directly: *"an `EdgeRef` referent is unreachable today (no expression form produces one yet; slice 2
mints `EdgeKey`)"*).

**Is `strength` a pseudo-field, a real field, or unspecified? — ANSWERED: a real, implicitly declared
field, D32.** `bsl-language.rst:1709-1715`: *"Every `EdgeType` carries one implicitly declared field,
`<edge-type>/strength`, with `:type coefficient` and `:kind extensive`. It needs no `deffield`… Re-
declaring it explicitly is `E-LOAD-001`."* The `extensive` kind is load-bearing: it is chosen
specifically so §2.4's coverage row (`sum_strength`/`avg_strength`) is expressible under §3.4's
aggregation law (`bsl-language.rst:1716-1730`).

**What serving it requires**:
1. `eval_field_of`'s match gains a `Value::EdgeRef(key) => field_of_edge(key, qname, env)` arm
   (`evaluator.rs:1229-1240`).
2. A new `field_of_edge` (sibling to `field_of_node`, `evaluator.rs:1313-1342`), performing the same
   two disciplines: (a) the qname's owning type must equal the `EdgeKey`'s own `edge_type` — this
   check is **cheaper** than the `NodeRef` case, because an `EdgeKey` carries its type inline (no
   substrate call needed, unlike `check_node_referent_type`'s `graph.node_type_of(id)` round-trip,
   `evaluator.rs:1261-1286`); (b) absence-is-not-a-value via the new substrate read (§3), rendered
   through the SAME `tick::bind_field_value` rendering path `field_of_node` already uses
   (`evaluator.rs:1340-1341`) — reused, not re-derived, matching D102's own precedent.
3. **The implicit-strength wiring gap (this pass's headline finding, §5 below)**: `field_of_edge`
   calling `bind_field_value("solidarity/strength", stored, types, enums)` degrades gracefully to
   `Value::Real(stored)` when the qname is unregistered (`tick.rs:318-334`'s own fallback, since
   `<edge-type>/strength` is not enum-typed) — so the **accessor itself** works even with an empty
   `TypeEnv.fields`. But any `fold`/`exists`/`select-*` over it does **not**: `typecheck.rs::resolve_field`
   (`typecheck.rs:247-252`) hard-fails `"unknown field: '{name}'"` on any qname `TypeEnv.fields`
   doesn't carry, and the live pipeline (`babylon-tick/src/lib.rs:127-128`) builds `TypeEnv.fields`
   from `scenario.fields.clone()` alone — the scenario's own textual `deffield` forms
   (`scenario.rs::load_deffield`, `scenario.rs:879-951`), which never auto-seed `<edge-type>/strength`.
   `declarations.rs::FieldRegistry::with_implicit_edge_strength` (`declarations.rs:317-330`) already
   builds exactly this seed set, fully tested (`declarations.rs:1042-1068`), but its own module doc
   states plainly it "**has no production caller today**" (`declarations.rs:288-304`). T2 is the
   consumer that registry has been waiting on since the ADR109 wiring-doctrine gap was filed.

## 2. `<edge-type>/strength` — the strength-only read story

D32 (§1.3 above) is the whole story: `strength` is a real, extensive, `Coefficient`-typed field every
`EdgeType` carries for free, already fully hashed (`CanonicalState` section `0x03`,
`state_hash.rs:154-171` — `push_f64(*strength, …)` per edge, present in the format since before T2).
Reading it via `field-of` adds **no new hash bytes and no new `CanonicalState` section** — the
"hash-free" framing in T2's charter row is literal: nothing new is hashed, only a new read path over
data already hashed since the substrate's `add_edge`. T3's R1 (ADR198) widens this to arbitrary
`deffield`-declared edge attributes behind a *new, empty-elided* fifth `CanonicalState` section — that
section does not exist yet and T2 must not touch it. Until T3 lands, `field-of` over an `EdgeRef` can
resolve exactly one non-strength scenario: a `deffield` naming an `EdgeType` first segment, hand-declared
in the content pack itself (already legal grammar and already load-time tested,
`declarations.rs:1071-1089`'s `a_deffield_may_own_off_a_node_edge_or_hyperedge_type`) — but with **no
storage behind it**: nothing writes such a field (T3's `update-edge` write-verb parity, R3, is what
gives a hand-declared edge field somewhere to be written to). T2's `field_of_edge` should therefore be
written generically (any qname whose owning type is an `EdgeType`), not hard-coded to `strength`
alone — the generic path naturally degrades to "declared but never written" → `E-EVAL-033`
(`bsl-language.rst:1866-1868`) for anything except `strength`, which is the correct, honest behavior
without any special-casing.

## 3. The substrate read surface

`GraphSubstrate` (`rust/crates/babylon-graph/src/substrate.rs`) **today**:
- `fn edges(&self, edge_type: &str) -> Vec<(NodeId, NodeId)>` (`substrate.rs:164-166`) — already
  exists, already ascending-sorted (both implementations `sort_unstable()` before returning,
  `memory.rs:220-229`, and the `HypergraphStore` equivalent at `hypergraph_store.rs:270`). This alone
  covers `edges`' materialization need — **no substrate change required for the `edges` query head
  itself.**
- **No edge-attribute read method exists.** `add_edge` writes `strength: f64` (`substrate.rs:111-117`)
  but there is no `edge_attribute`/`edge_strength`/`edge_between`-equivalent read — the asymmetry
  `node_attribute` has (`substrate.rs:142`) and edges do not. This is the ONE new method ADR197
  scoped ("a new `Value::EdgeRef`/`EdgeKey` type **and one read-only edge-attribute lookup**") and
  that four independent port-estate-survey adjudications (survey `reports/port-estate-survey-2026-08-12.md:129,285-286,379-381`)
  independently re-derived as a correction to ADR197's own text — this pass's own read of
  `substrate.rs` confirms the same gap by inspection, a fifth independent confirmation.
- **Both storage backends already hold exactly the data the new method needs, keyed exactly right.**
  `MemoryGraph::edges: HashMap<(String, NodeId, NodeId), f64>` (`memory.rs:47`) and
  `HypergraphStore::edges: HashMap<(String, NodeId, NodeId), f64>` (`hypergraph_store.rs:78`) are
  byte-identical in shape (the module doc says as much: `hypergraph_store.rs:4-14`, "identical in
  shape to `MemoryGraph`'s"). A new trait method — e.g.
  `fn edge_attribute(&self, edge_type: &str, from: NodeId, to: NodeId) -> Result<f64, GraphError>`
  (naming pattern mirrors `node_attribute`) — is a **one-line `HashMap::get` on each backend**,
  exactly the shape `node_attribute` itself already has (`memory.rs:189-203`). This is the same size
  and risk class as slice 1's one new method (`node_type_of`) and should get the same III.7 proof
  obligation: a dedicated fixture-hash test showing the new read-only method moves zero
  `CanonicalState` bytes (the precedent test is named directly:
  `adding_a_read_only_query_method_does_not_move_the_state_hash`, cited in ADR197's own decision text).
- The method also directly serves `edge-between`'s existence check: `edges.get(&(edge_type, from, to))`
  returning `None` is exactly `E-EVAL-034`'s condition, and `Some(strength)` is simultaneously the
  strength READ — `edge-between` and `field-of` over its result can share the identical substrate call
  (`edge-between` needs only the existence half; `field-of` needs the value half; both derive from one
  `HashMap::get`).
- **Determinism**: `edges()`'s existing sort already gives §2.6's total order for the single-type case
  (`bsl-language.rst:1040-1046`); the new attribute-read method is a keyed point lookup with no
  iteration order to pin at all — no `HashMap` iteration is exposed by it, so Constraint 2 (no
  `HashMap` iteration on a result path) is satisfied by construction, matching `node_attribute`'s own
  precedent.

## 4. Typecheck state

- `rule_pipeline.rs::field_ref_for` (`rule_pipeline.rs:617-686`) — the §3.4 kind-legality reduction
  function the mission asked about — is **referent-kind-agnostic already**: its `(field-of <expr>
  <qname>)` arm (`rule_pipeline.rs:654-661`) matches on the qname alone, never inspecting what
  `<expr>` evaluates to. A fold over `edges` reading `(field-of it solidarity/strength)` reduces
  through this function exactly the same way a fold over `nodes` reading `(field-of it
  social-class/wealth)` does — **no change needed here for T2.**
- `typecheck.rs::typecheck_aggregation`/`resolve_field` (`typecheck.rs:123-252`) is where the real gap
  lives (§1.3 point 3, §5 below): it requires the field name to be a key of `TypeEnv.fields`,
  unconditionally, regardless of which graph-element kind owns it.
- `score_class.rs::ScoreClass::EdgeReference` **already exists and is already exercised** for `edges`'
  classification (`r9_chapters.rs::the_result_type_is_the_querys_element_type_for_all_six_heads`,
  `r9_chapters.rs:1054-1082`, asserting `("(edges EdgeType/SOLIDARITY)", ScoreClass::EdgeReference)`).
  This is the `select-max`/`select-min` legality machinery (D46/D67: references are non-comparable
  scores, `typecheck.rs:254-292`) — already correctly refusing an `EdgeRef` as a score, no work needed.
- `grammar.rs::ARITIES`/`ENUM_REF_POSITIONS` (`grammar.rs:197-206,636-647`) already parse and
  arity/kind-check `edges` (1-2 operands), `edge-between` (exactly 3, first operand `EdgeType`) and
  `the` (exactly 1, operand `NodeType`) — confirmed directly, matching the Solidarity port
  inventory's own citation of the same lines.
- `declarations.rs::RESERVED_FORM_TAGS` already reserves `"edges"`/`"edge-between"` (and `"the"`)
  against the intrinsic namespace (`declarations.rs:32-82`) — no new reservation needed.
- **`r9_chapters.rs` already carries a load-time-only conformance suite exercising `edge-between`/`the`
  at the parse/cost/kind level** (module doc, `r9_chapters.rs:1-23`: *"the dyadic edge lane —
  edges/edge-between/the"*), including the §3.8-item-8 worked shape's exact cost total
  (`r9_chapters.rs:384-403`), the E-TYPE-011 kind check (`r9_chapters.rs:428-434`), and the *shape* of
  an unresolvable `edge-between` (`r9_chapters.rs:463-472`). T2's job is to make these evaluate for
  real, not to invent the vectors from nothing.

## 5. The implicit-strength wiring gap — this pass's headline finding

Restated in one place since it spans §§1.3/3/4: `declarations.rs::FieldRegistry` (the D31/D32-correct,
fully load-time-validated field registry — kernel agreement `E-LOAD-022`, duplicate detection
`E-LOAD-001`, the works) is **dormant** — its own doc says outright: *"This type, its `declare` … and
`with_implicit_edge_strength` have no production caller today … Production builds its `TypeEnv`
straight from a loaded scenario's own `deffield` forms instead"* (`declarations.rs:288-304`). The
scenario-level `load_deffield` (`scenario.rs:879-951`) is a **separate, simpler, unvalidated** parser:
it does not check an owning-type prefix resolves to a registered `NodeType`/`EdgeType`/`HyperedgeType`
at all (contrast `declarations.rs`'s `E-LOAD-023`), and it has no notion of "implicit" fields to guard
against re-declaration.

Two ways forward, presented for the plan author's judgment (not decided here):

- **(a) Wire the seed into the live pipeline.** Extend `babylon-tick/src/lib.rs:127-128`'s `TypeEnv`
  construction to also seed one `<edge-type>/strength` entry per `EdgeType` the scenario's vocabulary
  census already knows about (the census already exists — `scenario.rs`'s own `edge_types` field,
  landed in the ADR197 PR-5 fix per that ADR's own text, `ADR197…yaml` item 5). This is the
  spec-compliant reading of D32 and finally connects the type-checked, tested `FieldRegistry`
  machinery to a real caller — but it is new wiring work in a currently-untouched seam
  (`babylon-tick/src/lib.rs`), not merely new `babylon-bsl` code.
- **(b) Require explicit re-declaration in content.** Have every `.bscn`/rule pack that reads
  `<edge-type>/strength` hand-write `(deffield solidarity/strength coefficient extensive)` in the
  scenario's own `deffield` block. This needs zero pipeline changes, but it silently reproduces the
  exact E-LOAD-001 violation D32 says should be impossible (scenario-level `load_deffield` has no
  "this is implicit, refuse the re-declaration" check — see above — so it would simply succeed,
  diverging from the normative spec text quoted in §1.3).

Either way, **this is a PLAN-MUST-VERIFY item**: I did not run cargo and cannot confirm which path (if
either) the T2 implementation should take compiles cleanly against `babylon-tick`'s existing scenario-
census plumbing without a live build. What I can state with file:line certainty is that *some*
resolution is required before a `fold`/`select-*` over `<edge-type>/strength` will typecheck — the
bare accessor (`field-of it solidarity/strength` with no aggregation) degrades gracefully without
either fix (§1.3 point 3), but the §2.4 coverage row itself (`sum`/`mean` over `strength`) is exactly
an aggregation and would not.

## 6. The `the` disposition question

`the` (§2.10, `bsl-language.rst:1818,1838-1842,1915-1931`) is tagged `"slice 2"` in
`evaluator.rs::UNSERVED_EXPRESSION_HEADS` (`evaluator.rs:521`) and ADR197's own handoff text lists it
as part of slice 2's scope verbatim: *"slice 2 (the dyadic edge lane — `edges`, `edge-between`,
`field-of` over an `EdgeRef`, `the`)"* (`ADR197…yaml`, decision §1). **T2's own charter row in the
Program 29 spec omits it**: *"T2 | Query Slice 2 — dyadic edge reads: `edges`, `edge-between`,
`field-of` over an EdgeRef"* (`docs/superpowers/specs/2026-08-12-program-29-substrate-widening-design.md:45`)
names three heads, not four. This is a live, unresolved tension between ADR197's slice boundary and
ADR198/the Program 29 spec's T2 scope line — not something to resolve here, per the task's own
instruction, but the evidence for the plan author:

- `the` has **nothing to do with edges** structurally — it resolves a `:ceiling 1` singleton `NodeType`
  carrier (result type `NodeRef`, not `EdgeRef`) and needs zero new substrate methods or `Value`
  variants (`graph.nodes(node_type)` already exists and a ceiling-1 type has at most one member to
  find — or, more likely, a dedicated substrate lookup by type is unnecessary since `nodes()` already
  returns the (0-or-1-element) set). It groups with the "dyadic edge lane" by ADR197's own naming
  convention (all four heads shared one plan/PR group), not by any technical dependency on
  `EdgeKey`/`Value::EdgeRef`.
- **Live, named first consumers exist right now, independent of T2's Solidarity story**: the
  port-estate survey names `the` as a real blocker for Allegiance @17.42 ("only `popular_front`'s
  singleton half needs `the`", `port-estate-survey-2026-08-12.md:101`), Electoral @17.45
  ("graph-scope singletons via `the` (Slice 2)", `:102`), Policy @17.47 ("`national_financial`
  singleton via `the`", `:103`), WealthDistribution @21.5 ("`the` is Slice-2 and `(domain :graph)`
  does not execute at all", `:111`, register row 22's sibling framing), and Sovereignty @17.5's
  `persistent_data` note explicitly contrasts *"portable today on Slice 1, **no `the`**"* (`:104`) —
  i.e., Sovereignty's own carrier read does NOT need `the` (a `:field`-anchored carrier suffices), but
  four *other* systems' authors reached for `the` specifically. `the`'s own §6.2 required-vector family
  (chapter C3, `bsl-language.rst:4204-4213`) is fully specified with concrete E-codes
  (`E-LOAD-043`/`E-EVAL-035`/`E-LOAD-045`/`E-TYPE-011`) and its own accumulation-vector shape, and
  `r9_chapters.rs` already carries load-time tests against most of them (`r9_chapters.rs:506-578` —
  cost, `E-LOAD-045`, the carrier-write shape).
- **Cost is trivial and separately proven**: `cost(the) = 1` (`bsl-language.rst:2751`), independently
  cheaper than `edge-between`'s `1 + Σ operands`, and already asserted
  (`r9_chapters.rs::the_costs_one_where_the_degenerate_fold_cost_a_ceiling_factor`, `r9_chapters.rs:506-511`).
- The plan author's live options, stated without a recommendation: (i) fold `the` into T2 since its
  evaluation work is small, self-contained, and shares no dependency surface with the `EdgeKey`/
  `Value::EdgeRef` work, clearing four more named blockers (Allegiance/Electoral/Policy/
  WealthDistribution's singleton halves) for roughly the cost of `edge-between`; (ii) leave it out of
  T2 exactly as chartered, on the reasoning that Program 29's own T2 row is presumably a deliberate,
  Director-reviewed narrowing rather than an oversight, and a T2.5/separate micro-train picks it up
  later. Both are defensible; this dossier does not choose.

## 7. What T3 changes out from under T2, and how T2 stays forward-compatible

ADR198 R1-R3 (T3's scope): full symmetric `deffield`-declared edge attributes (any type/kind, enum
included, Currency storage still refused), a **new, empty-elided fifth `CanonicalState` section**
hashed only when at least one edge attribute exists (so every existing golden stays byte-identical at
T3's landing too — the elision rule is the same discipline slice 1's `node_type_of` and T2's new method
both already follow), and full `update-edge` write parity (`set`/`add`/`sub`/`scale` through the same
`PendingWrite` collect-then-apply machinery as `update-node`, D104).

What this means for T2's own design, concretely:
- `field_of_edge` (§1.3) must be written **generically over any qname whose owning type is an
  `EdgeType`**, not hard-coded to read `strength` alone — T3 will make other qnames resolvable through
  the exact same accessor without a second `field-of` code path. The substrate call `field_of_edge`
  makes should be the SAME "keyed lookup by (edge_type, from, to, attribute-name)" shape T3's storage
  will generalize to, even though T2 only ever populates it with the one implicit `strength` value
  (i.e., design the new substrate method's signature as if a T3-era version might take an `attribute:
  &str` parameter the way `node_attribute` does, rather than being strength-hard-coded — a plan
  decision, flagged not made).
- T2 must **not** add any `CanonicalState` section or touch `state_hash.rs`'s section count/tags — the
  strength read rides the existing section `0x03` exactly as-is (§2 above). If a T2 substrate method
  signature anticipates T3's generalization (previous bullet), it must still resolve, for T2's own
  landing, to reading ONLY the existing `edges: HashMap<(String, NodeId, NodeId), f64>` map — no new
  storage field on either `MemoryGraph` or `HypergraphStore`.
- `EdgeKey`'s shape (§1.1) should be reusable as T3's `update-edge` referent type without redesign —
  it already needs to be (per the spec's own worked example, `bsl-language.rst:1910-1913`:
  `(update-edge (edge-between EdgeType/SOLIDARITY self other) solidarity/strength (scale 0.95c))`) —
  so `edge-between`'s result type is shared, unmodified, across T2 (read) and T3 (write referent).

## 8. Solidarity's exact read needs — is it servable on T2 alone?

`src/babylon/engine/systems/solidarity.py:121-202` (full read, corroborating
`reports/port-inventories/solidarity-port-phase1-inventory-2026-08-12.md` §§1-2, itself already
adjudicated 2026-08-12 against dev `9324482f`). The system's edge-resident read is exactly one datum:
`edge.attributes.get("solidarity_strength", 0.0)` (`solidarity.py:133`) — maps 1:1 onto the implicit
`<edge-type>/strength` field (D32), confirmed independently by the inventory's own adjudication point 2
(`solidarity-port-phase1-inventory-2026-08-12.md:350-359`: *"Solidarity needs **no** edge-attribute-
storage widening… slice 2 alone unblocks this system"*).

**But the frozen loop shape (`for edge in graph.query_edges(edge_type=EdgeType.SOLIDARITY):`,
`solidarity.py:121`) reads BOTH endpoints of every edge it visits** (`edge.source_id`/`edge.target_id`
at `:123-124`, and the write target is `edge.target_id` at `:169`). A literal transcription — a
`for-each` over `(edges EdgeType/SOLIDARITY)` — needs a way to read an `EdgeRef`'s source/target node,
and **no such accessor exists**: `bsl-language.rst` §3.8 item 8 (`:2935-2965`) names this explicitly as
*"an open item, not a settled absence… No form yields an `EdgeRef`'s source or target node… a system
that iterates edges it did not start from is **not authorable in BSL**, and that is a port blocker
(D78)."* This item is **not scoped to T2** (three heads, no `source-of`/`target-of`), **not scoped to
T3** (attribute storage, not endpoint accessors), and not scoped anywhere else in Program 29 — it is a
free-floating open item the whole ADR197/ADR198 lineage has left unresolved.

**The reformulation that sidesteps it entirely — and IS fully servable on T2 alone — is the one §3.8
item 8's own worked example gives**, self-anchored rather than edge-iterating:

```scheme
(fold sum (neighbors self EdgeType/SOLIDARITY :in NodeType/SOCIAL_CLASS)
      (field-of (edge-between EdgeType/SOLIDARITY it self) solidarity/strength))
```

Per-target-node (the write side, `self`), walk incoming `SOLIDARITY` neighbors (`neighbors`, already
landed slice 1) and resolve each edge's strength via `edge-between` (T2) — no `edges` query head, no
endpoint accessor, needed at all. This changes the write target from "the edge's target, computed"
(the survey's assumed for-each-over-edges shape) to "always `self`" — actually *simpler* than the
"update-node against a computed reference" pattern the Territory train already proved
(`query_lane_e2e.rs` shape B), since here the target IS the rule's own subject.

**Correction to the Solidarity inventory's own adjudication.** Its point 4
(`solidarity-port-phase1-inventory-2026-08-12.md:384-400`) reasons entirely in terms of *"a slice-2
`for-each` over `(edges EdgeType/SOLIDARITY)`"* and derives a "last-write-wins" divergence finding from
that assumed shape (multiple inbound edges to one target, sequential frozen semantics vs. BSL's
collect-then-apply). That divergence finding is real **for the for-each-over-edges shape**, but that
shape itself needs the open §3.8 item 8 accessor the inventory's own §6 correctly names elsewhere in
the SAME document (its original, pre-adjudication §6 last row) as unresolved — the adjudication's
point 4 does not reconcile this internal tension. Under the self-anchored `neighbors`+`edge-between`
reformulation above, the multi-inbound-edge question resolves differently and arguably more cleanly:
a `fold sum` over all inbound edges naturally reads every edge's strength against the SAME pre-tick
target consciousness (§4.2 chapter C4's pre-state law, already landed per D103/D104) and produces one
combined delta with one clamp — no sequential-vs-batch ambiguity to D-record at all, because there is
only one write, not N sequential writes. This is a content-design choice for the actual Solidarity port
train to make, not an engineering question T2 must answer, but T2's plan author should know the
"Slice 2 ALONE" verdict is correct only under this reformulation, not under the shape the adjudication's
own prose assumes.

## 9. E2E vector shapes T2 owes, and the NOT-LANDED list

**Vector-shape precedent**: `rust/crates/babylon-tick/tests/query_lane_e2e.rs` (428 lines, four
Territory-shaped vectors, ADR197 PR-5/Task-15). Every vector: (a) runs through the REAL production
entry point `babylon_tick::run_once_into` against `HypergraphStore` (never `MemoryGraph` directly —
`MemoryGraph` is the crate's own unit-test-only backend); (b) loads a shared, hand-built `.bscn`
fixture (`content/scenarios/query-lane-e2e.bscn`) with a `<subject>/shape` discriminator field so
multiple shapes can share one scenario file without cross-firing; (c) derives every expected numeric
value independently (Python `repr()`/`struct.pack`) and pins exact `f64::to_bits()` where the value
is not a clean decimal, never comparing against this crate's own printed output; (d) includes a
same-file determinism test running every shape twice and asserting the full `TickReport` (hash +
`fired` + `per_rule_fired`) is byte-identical across runs. T2's own e2e file should follow this exact
shape, adding a fixture with `SOLIDARITY`-typed (or similarly-scoped) edges and covering, at minimum:
`(edges EdgeType/X)` materialized and folded/summed for strength; `edge-between` resolving successfully
and failing (`E-EVAL-034`) inside a real tick; `field-of` over the result of both `edges`-produced and
`edge-between`-produced `EdgeRef`s reading the SAME field consistently (the two paths must agree — an
easy place for an off-by-one in `EdgeKey` construction to hide).

**D-rows/E-codes to expect on landing**: `E-EVAL-033` (accessor type/absence — already minted, reused,
not new), `E-EVAL-034` (edge-between failure — already minted, `evaluator.rs:157-158`), `E-TYPE-011`
(enum-ref kind mismatch — already minted and already tested at the grammar level for both `edges`' and
`edge-between`'s operands). No new `EvalCode`/`TypeCode` variant appears necessary from this pass's
reading — everything T2 needs already exists in the taxonomy, which is itself worth stating plainly as
a scope-shrinking fact for the plan.

**NOT-LANDED after T2** (explicitly out of scope, for the plan's own boundary statement):
- `update-edge` and all edge writes (T3, R3).
- Any `deffield`-declared edge attribute beyond the implicit `strength` — readable in the grammar
  today, but with no write path until T3, so any such field reads as permanently `E-EVAL-033`
  ("never written") until T3 lands (§2 above).
- The fifth `CanonicalState` section (T3, R2) — does not exist after T2.
- `source-of`/`target-of` edge-endpoint accessors (§3.8 item 8) — open, unscoped by any Program 29
  train; a genuine gap T2 does not close, and Solidarity's literal for-each-over-edges transcription
  stays blocked on it even after T2 lands (§8 above).
- `hyperedges`/`members-of`/`hyperedges-of`/`metric-of` (slice 3) and `membership-field-of` (slice 4)
  — untouched, per their own `UNSERVED_EXPRESSION_HEADS` rows.
- `the` — disposition open per §6, pending the plan author's decision.

## 10. Surprises, with evidence

1. **Most of slice 2's "engineering" is already built and tested — only evaluation is missing.** Grammar
   arity, `E-TYPE-011` kind checks, the §3.7 cost rows (including the exact worked §3.8-item-8 shape's
   total), the §3.4 kind-legality reduction (referent-agnostic already), `ScoreClass::EdgeReference`,
   and the `EvalCode` variants for `E-EVAL-034`/`E-EVAL-035` are ALL present and tested on `dev` today
   (§§1-4 above). This narrows T2's real engineering surface to: one `Value` variant, one `Element`
   variant + its Ord decision, `materialize_edges`, `eval_edge_between`, `field_of_edge`, one new
   substrate method × 2 backends, and the implicit-strength wiring decision (§5).
2. **`GraphSubstrate::edges()` already returns exactly the right order for `edges`' materialization,
   with no new substrate work** — confirmed by direct inspection of both `MemoryGraph::edges` and
   `HypergraphStore`'s equivalent, matching the already-correct `sort_unstable()` on
   `Vec<(NodeId, NodeId)>`.
3. **The one new substrate method is a one-line `HashMap::get` on both backends**, because both
   already store `strength` keyed by the exact `(edge_type, from, to)` triple `edge-between` needs
   (§3) — this is a materially smaller substrate change than slice 1's `node_type_of`, which needed a
   `HashMap<NodeId, String>` lookup plus a `&str`-lifetime decision; here the map already exists in
   exactly the needed shape.
4. **The implicit-strength `FieldRegistry` machinery already exists, fully tested, completely
   disconnected from production** (§5) — a genuine "declared-not-wired" (ADR109) case that no prior
   inventory or ADR flagged by name; it was found only by tracing `TypeEnv.fields`'s actual runtime
   construction path (`babylon-tick/src/lib.rs:127`) back from `typecheck.rs::resolve_field`'s hard
   failure, not by reading `declarations.rs` in isolation (which reads as fully production-ready on
   its own).
5. **`the` has zero technical dependency on `EdgeKey`/`Value::EdgeRef`** despite being grouped under
   "slice 2" by every document that names it, and has four independent, currently-blocked, named
   consumers waiting on it RIGHT NOW (Allegiance, Electoral, Policy, WealthDistribution) — none of
   which are Solidarity, and none of which T2's own charter row currently serves (§6).
6. **§3.8 item 8 (edge-endpoint accessors) is a genuine gap in the WHOLE Program 29 lane structure**,
   not merely a T2 boundary question — it blocks the literal transcription of any system that iterates
   edges without already holding one endpoint, and nothing in T2, T3, T4, T5 or T6's charter rows
   closes it (§8 above). This should be visible to whoever schedules the next round of trains, even
   though it is not T2's job to fix.

## PLAN-MUST-VERIFY items (no cargo run in this pass — flagged, not asserted)

1. **§5's wiring decision (a) vs (b)** — whether extending `babylon-tick/src/lib.rs`'s `TypeEnv`
   construction with an implicit-strength seed compiles cleanly and interacts correctly with
   `scenario.rs`'s existing `edge_types` census, or whether requiring hand-declared `deffield` rows is
   simpler in practice than this dossier's file-reading can settle. Needs an actual implementation
   attempt.
2. **`EdgeKey`'s exact field types and hashing/equality semantics** — whether `edge_type: String` per
   instance is acceptable or whether the crate's existing `&str`-vs-`String` conventions (per
   `substrate.rs:19-22`'s own "Task 16 revisions" note about avoiding `Box::leak`) push toward an
   interned/enum representation instead. This dossier establishes the SHAPE requirement (must carry
   type + both endpoints, must support the D96 total order) but not the exact Rust representation —
   a real implementation choice, not something file-reading alone resolves.
3. **`Element`'s cross-kind `Ord` pinning (§1.1 point 2)** — this dossier confirms no production caller
   currently sorts a mixed `Vec<Element>`, but cannot rule out by reading alone that some future
   in-T2-scope combinator does; the compile-time exhaustive-match trip-wire
   (`query.rs::element_kind_name`) will force the question to be answered at the point `Edge` is
   added, but whether "keep the derive" or "hand-write `Ord`" is correct is an implementation-time
   judgment call, not a fact this pass can settle without writing the code.
4. **Whether the new substrate method should be named/shaped for T3-forward-compatibility** (§7) — a
   design recommendation, not a verified requirement; whether `HypergraphStore`'s own `edges` field
   needs any additional consideration beyond `MemoryGraph`'s (e.g., interaction with the
   `hypergraph-rs`-backed hyperedge half) was checked by reading the struct definition only, not by
   compiling a change against it.
5. **Whether `field_of_edge`'s generic-qname design (§2) actually round-trips through
   `tick::bind_field_value` without a subtle divergence from `field_of_node`'s behavior** — the code
   reading suggests it should (same function, same fallback), but this is asserted from static reading
   of `tick.rs:318-334`, not from running the evaluator.

---

## CORRECTIONS/FLAGS

Against `reports/port-inventories/solidarity-port-phase1-inventory-2026-08-12.md` (including its own
2026-08-12 adjudication section) and, where relevant, `reports/port-estate-survey-2026-08-12.md`:

1. **The Solidarity adjudication's point 4 assumes a `for-each`-over-`(edges EdgeType/SOLIDARITY)` port
   shape that needs an edge-endpoint accessor §3.8 item 8 itself says is an open, unscoped absence** —
   the adjudication does not reconcile this against the ORIGINAL inventory's own §6 last row (which
   correctly flagged the endpoint question as unresolved before being overwritten by the adjudication's
   "already ruled, not open" correction). §8 above gives the self-anchored `neighbors`+`edge-between`
   reformulation that sidesteps the gap entirely and is genuinely servable on T2 alone — the "Slice 2
   ALONE" verdict for Solidarity is right, but only under this reformulation, which neither the
   inventory nor its adjudication states.
2. **Neither the Solidarity inventory nor its adjudication names the `FieldRegistry`/`TypeEnv` wiring
   gap (§5)** — both correctly identify that `field-of` over an `EdgeRef` is unreachable today, but
   neither traces what happens to a fold/aggregation over `<edge-type>/strength` once the accessor
   itself is built; this pass's read of `typecheck.rs::resolve_field` alongside
   `babylon-tick/src/lib.rs`'s `TypeEnv` construction is, as far as this pass can tell, a new finding.
3. **The port-estate survey's own §4.5 ranking (`port-estate-survey-2026-08-12.md:285-286`) states
   "Slice 2 — 4 named, 1 cleared alone" without listing which four** — cross-referencing the survey's
   own per-system table (§2 rows), the four most plausibly meant are Solidarity (clears alone),
   Consciousness (@17.0, "Slice 2 + substrate edge reader"), Survival (@15.0, "Slice 2 (solidarity
   multiplier — live on `debs`/`bernie_valve`)"), and Policy (@17.47, "Slice 2 for THREE computations"
   — though Policy is graded PORTABLE WITH D-RECORDS overall, with Slice 2 blocking only a partial
   sub-surface). This is a reconstruction, not a claim the survey states explicitly, and should be
   treated as tentative — flagged, not asserted, per the task's own instruction on impossibility/scope
   claims.
