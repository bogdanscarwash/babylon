# T2 Slice-2 Edge Reads Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Serve the BSL query-evaluation lane's slice 2 — `edges`, `edge-between`, `field-of` over
an `EdgeRef` — through the real `babylon-tick::run_once_into` production seam. Hash-free by
construction (zero new `CanonicalState` bytes); strength-only until T3 (ADR198 R1-R3) widens edge
attribute storage. This is the language train, not a port: it anticipates Solidarity's own read
shape (§3.8 item 8's worked example) but ships no Solidarity content.

**Architecture:** One new `GraphSubstrate` method (`edge_attribute`, both backends), one D32
implicit-field wiring fix (`babylon-tick`'s `TypeEnv` construction), one new `Value`/`Element`
variant pair (`EdgeRef`/`Edge(EdgeKey)`), three new evaluator functions
(`materialize_edges`/`eval_edge_between`/`field_of_edge`), one new e2e test file following the
`query_lane_e2e.rs` genre. No grammar, cost, or vocabulary work — the scout dossier
(`reports/t2-slice2-reads-surface-facts-2026-08-12.md`, read in full before Task 1; every claim in
this plan traceable to it or to this plan author's own direct reading of the cited files) confirms
grammar arity, `E-TYPE-011` kind checks, the §3.7 cost rows, `ScoreClass::EdgeReference`, and the
`EvalCode` variants (`E-EVAL-034`/`035`) are already landed and tested. `the` is **not** in this
train's scope — see the disposition record, Task 7.

**Tech Stack:** Rust (`babylon-graph`, `babylon-bsl`, `babylon-tick`), BSL content
(`.bscn`/`.bsl`), deterministic table-driven tests only (no proptest).

**Evidence base (read all four before starting):**
- `reports/t2-slice2-reads-surface-facts-2026-08-12.md` — the scout dossier. Its five
  PLAN-MUST-VERIFY items are resolved below (three settled by this plan author's own direct
  reading — cited inline; two remain genuine implementation-time judgment calls, flagged in
  Self-review).
- `ai/decisions/ADR197_bsl_query_evaluation_slice1_handoff.yaml` — slice 1's closed scope,
  the `node_type_of` III.7 precedent this plan's substrate task repeats verbatim.
- `ai/decisions/ADR198_program29_substrate_widening_charter.yaml` — R1-R3 (T3's storage scope,
  which this train must not touch), R8 (T2 = issue #559).
- `rust/crates/babylon-tick/tests/query_lane_e2e.rs` — the e2e vector genre this train's own
  `edge_lane_e2e.rs` (Task 6) follows exactly: real driver, hand-derived expecteds, a shared
  discriminator-scoped fixture, a same-file determinism test.

## Amendment log

- **2026-08-12 — Task 6a: the `load_edge` strength-literal widening; Task 6's fixture
  literals `c`-suffixed.** Task 6's fixtures as first merged could not load — execution-proven
  through the real driver seam (`run_once`, `babylon-tick/src/lib.rs:72`, whose `run_once_into`
  core at `:330` funnels through `prepare_rules` → `load_scenario`, `:126`), twice over: a bare
  `0.125` strength cannot LEX
  (`E-LEX-021`, "a non-integer literal requires a kind suffix ($, p, i, c) — §1.5",
  `reader.rs:774`), and the lex repair `0.125c` is refused by the loader's deliberate int-only
  strength restriction (`rust/crates/babylon-bsl/src/scenario.rs:1325-1336`, its own comment:
  "the int-literal restriction is stated directly here"; the `.bscn` dialect is spec-unpinned per
  D93, so this is a loader fact, not a spec fact — and the already-LANDED Task 2 probe hit the
  same wall at execution time, falling back to an int strength `1` where this plan's Task 2 text
  still shows `0.5`, `babylon-tick/src/lib.rs:458`). The fractional strengths are load-bearing
  (Shape 1/2a/3's expecteds `0.796875`/`0.03125`/`0.375` and the Blocker-2 coefficient
  store-domain arithmetic), and pre-seeding edges on the graph before `run_once_into` would
  bypass the hydration seam these vectors exist to prove — so Task 6a (inserted before Task 6;
  nothing renumbered, so every existing cross-reference stays valid) rules the widening:
  `load_edge` accepts int + `c`-suffixed Coefficient strength literals (D32's own ruled kind for
  `<edge-type>/strength`), every other literal kind stays refused, and every `(edge …)` strength
  literal in Task 6's fixture is now `c`-suffixed. No expected value, shape, or store-domain
  argument changed — each suffixed literal converts bit-exactly (Task 6a design decision 2).
  (scope clause SUPERSEDED — see next entry)

- **2026-08-12 (fix round) — Task 6a's scope adjudicated to KIND-BLIND `p`/`i`/`c` (reviewer
  MAJOR-1, option (i)); supersedes the previous entry's scope clause.** The first amendment's
  `c`-only scope was reviewed EXECUTE AFTER FIXES: its justification misread the node path
  (`attribute_value_unit_interval` does not defer to the declaration — it takes the declared type
  and deliberately ignores the literal's kind, `scenario.rs:1171-1177`, because kinds do not
  survive evaluation) and never examined the runtime writer of the same position
  (`structural_verbs.rs::add_edge` accepts any `Value::Real` at `:strength`, `:929-936` —
  kind-blind for the same reason). `c`-only would have minted a new bidirectional load/runtime
  divergence; the ruling is now: hydration mirrors the runtime writer — int + `p`/`i`/`c`
  accepted, `r`/`$` refused (both also refused by the runtime `:strength` match, which takes only
  `Value::Real`). Citation repairs landed in the same round: the conversion-site census corrected
  to eight existing inline sites + Task 6a's ninth (with the DRY tradeoff owned explicitly),
  `E-LEX-024` cited at its constructor/fire sites (`reader.rs:877-883`/`:887,:889,:893`),
  `attribute_value_unit_interval`'s span extended to `:1210-1261`, §3.9 clause 1
  (`bsl-language.rst:3042-3045`) added as design decision 3's spec hook, the remaining
  int-at-hydration vs. runtime-`Value::Int`-refusal divergence recorded beside that open
  question, the driver-seam naming corrected (`run_once`, `lib.rs:72`), and Task 6a added to the
  Sequencing-note and PR-grouping enumerations of PR B. Two reviewer citations were themselves
  corrected against a fresh Read during the fix round: `evaluator.rs:382` IS the p/i/c division
  line (`:381` is its `#[allow]`), and E-LEX-024's fire sites are `:887`/`:889`/`:893`.

## Global Constraints

- **Port-as-is discipline does not apply here** — T2 is new language surface, not a transcription
  of frozen Python. What DOES apply: every claim about *why* a design is shaped a certain way must
  cite the spec section or the precedent it follows (§2.6/§2.10/§3.4/§3.7, D32, D96, D46, the
  `node_type_of` III.7 precedent) — never an unstated assumption.
- **No new `CanonicalState` section, no new stored field.** T2's substrate method reads
  `edges: HashMap<(String, NodeId, NodeId), f64>` — already-hashed data (section `0x03`,
  `babylon-graph/src/state_hash.rs:22-24,154-171`) — on both backends. Nothing is added to either
  struct.
- **Six-leg cargo gate per commit** (from `rust/`): `cargo fmt --check`;
  `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`;
  `cargo clippy -p babylon-kernel --all-targets -- -D warnings -D clippy::pedantic` and the same for
  `-p babylon-bsl` and `-p babylon-graph`; `RUSTDOCFLAGS='-D warnings' cargo doc --workspace
  --no-deps`; `cargo test -p babylon-tick --test tick_goldens --locked`. All six pre-existing golden
  pins (two-classes, vitality, us-counties, organization, territory, production) plus
  `babylon-client/tests/engine_link.rs`'s pin must stay byte-identical in every commit (Task 7 makes
  this a proof, not an assumption).
- **`Value`'s OWN blast radius is verified LOW.** Every existing match over `&Value`/`Value` in
  `evaluator.rs` (`apply_arith`, `apply_ordering`, `apply_equality`) and `structural_verbs.rs`
  (`resolve_node`, `resolve_hyperedge`, `numeric_write_value`) carries a wildcard `other =>`/`_ =>`
  arm — confirmed by direct reading, no exhaustive match over the full `Value` enum exists anywhere.
  Adding `Value::EdgeRef` touches no call site by compiler force at all (`Value` was never `Copy`,
  so nothing about ownership changes either).
- **`Element` is a DIFFERENT story — it derives `Copy` today (`query.rs:50`), and `EdgeKey` cannot
  be `Copy` (it owns a `String`).** Adding `Element::Edge(EdgeKey)` forces `Element` to drop `Copy`,
  which ripples to nine verified call sites: `evaluator.rs:396,420,1021` (`element.to_value()`/
  `best_element.to_value()` called on an owned or referenced `Element` — no change needed at any of
  the three once `to_value` takes `&self`, see Task 3), `:949,1001,1081,1110,1190` (`for &element in
  elements`/`&elements` — a dereferencing pattern that only compiles under `Copy`; the loop at
  `:1001` additionally needs one internal `.clone()`, since it consumes `element` a second time in
  the SAME iteration — see Task 3 Step 4), `:999` (`elements[0]` — an index-and-move that only
  compiles under `Copy`), and `query.rs:511-512` (`element_kind_name(a)` then reusing `a` — a
  move-then-reuse that only compiles under `Copy`). Task 3 enumerates the exact fix at each site —
  this is a real, larger blast radius than `Value`'s own, but still bounded: `structural_verbs.rs`
  DOES touch `Element` directly (`:454,489,686` all hold/iterate `Vec<Element>`), but never through
  a Copy-dependent pattern (`for element in elements` over an OWNED `Vec`, never `for &element in
  &elements`) — so it needs zero changes. No Copy-dependent site exists outside `evaluator.rs`/
  `query.rs`.
- **`GraphSubstrate` has exactly two implementors in-tree** (`MemoryGraph`, `HypergraphStore`;
  verified by `rg -l "impl GraphSubstrate for"` — zero other hits), matching the `node_type_of`
  precedent's own blast radius exactly.
- **Machine safety:** cargo single-flight; no parallel test fan-out.

## Sequencing note

PR A (Tasks 1-2) is a strict prerequisite for PR B (Tasks 3-7): `field_of_edge` and
`eval_edge_between` (Tasks 4-5) call the new substrate method Task 1 adds, and the e2e fold-over-
edge-strength vectors (Task 6) need the D32 wiring Task 2 lands to typecheck at all. PR A is
narrow, mechanical, and low-risk (a one-line-per-backend trait method plus a ~15-line merge in one
function) — it can land and be reviewed fast, derisking the foundation before the semantically
loaded evaluator work. Within PR B, Tasks 3-5 land in that order because `field_of_edge`
(Task 5) pattern-matches on `Value::EdgeRef`, which Task 3 mints, and reuses the cost/referent-type
machinery Task 4 establishes the shape for. Task 6a (amendment, 2026-08-12) lands after Task 5 and
before Task 6, inside PR B: a hydration-loader widening rides in the evaluator PR because Task 6's
fixtures cannot load without it — split out on its own, either PR would be unshippable alone (a
loader widening with no consumer, then an e2e task whose fixture is plan-blocked).

---

### Task 1: The one new substrate method — `edge_attribute`

**Files:**
- Modify: `rust/crates/babylon-graph/src/substrate.rs` (the `GraphSubstrate` trait)
- Modify: `rust/crates/babylon-graph/src/memory.rs` (impl)
- Modify: `rust/crates/babylon-graph/src/hypergraph_store.rs` (impl + fixture-hash test doc)
- Modify: `rust/crates/babylon-graph/src/conformance.rs` (four shared backend-contract rows)

**Interfaces:**
- Produces: `GraphSubstrate::edge_attribute(edge_type, from, to, attribute) -> Result<f64,
  GraphError>` — the read half `edge-between`'s existence check (Task 4) and `field-of` over an
  `EdgeRef` (Task 5) share.

**Design decision (dossier §7, PLAN-MUST-VERIFY item 4, resolved here — REVISED against adversarial
review Major 6):** `attribute` is the FULL QNAME (e.g. `"solidarity/strength"`), mirroring
`node_attribute`'s own convention EXACTLY — `field_of_node` passes it the full qname unmodified
(`evaluator.rs:1320`, `graph.node_attribute(id, qname)`), and `MemoryGraph::node_attribute` keys its
`attributes: HashMap<(NodeId, String), f64>` on that full string as-is (no segment-splitting inside
the trait method). The FIRST version of this plan had `field_of_edge` pass only the qname's LAST
SEGMENT ("strength") — that breaks the mirror claim (`node_attribute` never splits) and would force
a SIGNATURE-MEANING change at T3 (from "bare attribute name" to "full qname"), contradicting ADR198
R1's "edges get declared, typed fields exactly as nodes have them." Corrected: the qname is passed
through whole; the method's own body checks whether it ends in `"/strength"` (a plain string split
on `attribute`, needing no case-rendering — the qname arrives from content already in the correct
kebab-case, and `babylon-graph` cannot import `babylon-bsl::vocabulary::render_member` regardless,
since `babylon-graph` sits BELOW `babylon-bsl` in the crate dependency graph — verified: `babylon-
graph/Cargo.toml`'s `[dependencies]` names only `babylon-kernel` and `hypergraph-rs`, no
`babylon-bsl`). Storage still holds exactly ONE value per edge
(`edges: HashMap<(String, NodeId, NodeId), f64>`, identical shape in both backends, `memory.rs:47`,
`hypergraph_store.rs:78`), so any OTHER full qname reads as "never written" — the SAME `GraphError`
shape `node_attribute` already gives an unwritten node field, so `field_of_edge`'s existing
error-to-`E-EVAL-033` mapping (Task 5) handles a not-yet-storable edge field with **zero special
casing**. At T3, `edge_attribute`'s SIGNATURE needs no change at all — only its body widens to a
real per-`(edge, full-qname)` lookup and the "must end in /strength" branch is deleted. Ownership of
the OWNER-SEGMENT check (does the qname's first segment actually name `edge_type`?) stays with the
CALLER (`field_of_edge`'s `check_edge_referent_type`, Task 5) — exactly the division of labor
`node_attribute`/`check_node_referent_type` already have; `edge_attribute` itself, like
`node_attribute`, does no ownership validation of its own.

**Placement (adversarial review nit, verified):** append `edge_attribute` at the END of the
`GraphSubstrate` trait, immediately after `node_type_of` (`substrate.rs:236-248`) — matching where
`node_type_of` itself landed (a widening tacked onto the trait's tail, not woven into the stable
`// ---- §2.6 query surface (dyadic half) ----` region alongside `nodes`/`edges`/`neighbors`).

- [ ] **Step 1: Add the trait method.** In `substrate.rs`, after `node_type_of`'s closing `}` (the
  trait's last member today), as the new last member:

```rust
    /// Read one dyadic edge's attribute (§2.10's `edge-between`/`field-of` share this) — the read
    /// half `edge-between`'s existence check and `field-of` over an `EdgeRef` both derive from.
    /// `attribute` is the FULL QNAME (e.g. `"solidarity/strength"`), mirroring
    /// [`Self::node_attribute`]'s own convention exactly — never a bare segment.
    ///
    /// **T2 scope (issue #559): the only PATTERN resolvable against real storage today is a qname
    /// ENDING IN `/strength`** — every `EdgeType` carries one implicit, always-written `Coefficient`
    /// field (D32, `bsl-language.rst` §2.9), and `add_edge`'s mandatory `:strength` operand is the
    /// only thing this trait's edge storage holds. **This method does NOT verify that `attribute`'s
    /// OWNER segment names `edge_type`** — exactly as [`Self::node_attribute`] performs no
    /// ownership check of its own, that half of §2.10 discipline 1 is the CALLER's obligation
    /// (`field_of_edge`'s `check_edge_referent_type`, upstream of every call this trait receives).
    /// A qname whose ATTRIBUTE segment is anything but `strength` is legal grammar (a `deffield`
    /// may own off an `EdgeType`, dossier-confirmed) but has no storage behind it until T3
    /// (ADR198 R1) — it reads exactly like a never-written node field: `GraphError`, never a
    /// default `0.0`. **READ-ONLY**: reports a fact `CanonicalState` section `0x03` already hashes
    /// (III.7, the `node_type_of` precedent — `ai/decisions/ADR197_bsl_query_evaluation_slice1_handoff.yaml`).
    ///
    /// # Errors
    /// Returns [`GraphError`] if no `(edge_type, from, to)` edge exists, or if `attribute` does not
    /// END IN `/strength` — absence is never a default `0.0`.
    fn edge_attribute(
        &self,
        edge_type: &str,
        from: NodeId,
        to: NodeId,
        attribute: &str,
    ) -> Result<f64, GraphError>;
```

- [ ] **Step 2: `MemoryGraph` impl.** In `memory.rs`, immediately after `edges` (the existing
  `Vec<(NodeId, NodeId)>`-returning method — the impl's OWN method order need not mirror the
  trait's, unlike the trait declaration itself):

```rust
    fn edge_attribute(
        &self,
        edge_type: &str,
        from: NodeId,
        to: NodeId,
        attribute: &str,
    ) -> Result<f64, GraphError> {
        // The owner-segment half of §2.10 discipline 1 (does `attribute`'s first segment name
        // `edge_type`?) is the CALLER's job (field_of_edge's check_edge_referent_type) — this
        // method, like node_attribute, does no ownership validation of its own. Here we only ask:
        // is the ATTRIBUTE half "strength", the one thing T2 actually stores?
        if !attribute.ends_with("/strength") {
            return Err(GraphError {
                message: format!(
                    "edge attribute '{attribute}' was never written — T2 stores a .../strength \
                     attribute only (D32; the owner segment is not checked here — see this \
                     method's own doc); other deffield-declared edge attributes land with T3 \
                     (ADR198 R1), never a default 0.0"
                ),
            });
        }
        self.edges
            .get(&(edge_type.to_owned(), from, to))
            .copied()
            .ok_or_else(|| GraphError {
                message: format!(
                    "no such edge: ({edge_type}, {from:?}, {to:?}) — never a default 0.0"
                ),
            })
    }
```

- [ ] **Step 3: `HypergraphStore` impl.** Identical body (same `edges` field shape,
  `hypergraph_store.rs:78`), placed after its own `edges` method.

- [ ] **Step 4: The III.7 hash-invariance proof.** `hypergraph_store.rs`'s existing test
  `adding_a_read_only_query_method_does_not_move_the_state_hash` already asserts the exact hex
  digest of a fixed fixture. Run it unmodified after Steps 1-3 land; it must stay green with the
  SAME hex string (`9577d95124a7c4ed6faad2c4aca5980b435fb73e7b58813413500a5fdef798ed`). Update
  BOTH: (a) the doc comment, adding one sentence recording that T2's `edge_attribute` addition
  (issue #559) is the second widening this test proves clean, alongside `node_type_of`; (b) the
  assertion's own failure message, currently `"state_hash moved — node_type_of must be III.7-clean
  ..."` — broaden it to `"state_hash moved — a read-only GraphSubstrate method addition (node_type_of
  or edge_attribute) must be III.7-clean (read-only, no new CanonicalState section, no byte moved)"`
  so a future regression's failure text does not misattribute the cause to `node_type_of` alone. Do
  not duplicate the fixture into a second test; the existing one already generalizes.

- [ ] **Step 5: The four shared backend-contract rows.** `conformance.rs` is the suite BOTH
  backends run (`run_substrate_conformance`, called from each backend's own test module) — the
  `node_type_of` precedent added two functions here (`node_type_of_reports_the_declared_type`,
  `node_type_of_a_dangling_id_is_loud_not_untyped`), registered in `run_substrate_conformance`'s own
  dispatch list (`conformance.rs:44-45`). **Type-name convention (adversarial review nit): this
  file's own fixtures use LOWERCASE `snake_case` type strings** (`conformance.rs:59`,
  `"social_class"`/`"solidarity"` — the raw `GraphSubstrate` trait treats type names as opaque
  strings; only BSL's own vocabulary layer requires uppercase enum members), unlike the
  `node_type_of` precedent's own `"SOCIAL_CLASS"` fixtures — match `:59`'s convention here since
  these four rows sit in the same file. Add four, after `node_type_of_a_dangling_id_is_loud_
  not_untyped`:

```rust
/// T2 (issue #559, `bsl-language.rst` §2.10): `edge_attribute` reads back the strength seeded at
/// `add_edge` — the same fact `CanonicalState` section `0x03` already hashes, read through a keyed
/// lookup instead of `edges`' ranged listing.
fn edge_attribute_reads_back_the_seeded_strength<G, F>(make: &F)
where
    G: GraphSubstrate + CanonicalState,
    F: Fn() -> G,
{
    let mut graph = make();
    let a = graph.add_node("social_class").unwrap();
    let b = graph.add_node("social_class").unwrap();
    graph.add_edge("solidarity", a, b, 0.5).unwrap();
    assert_eq!(
        graph
            .edge_attribute("solidarity", a, b, "solidarity/strength")
            .unwrap(),
        0.5
    );
}

/// A dangling `(edge_type, from, to)` triple must never read as an untyped edge's strength — the
/// same honest-null discipline `node_attribute`/`node_type_of` already hold (III.11).
fn edge_attribute_on_a_missing_edge_is_loud_not_zero<G, F>(make: &F)
where
    G: GraphSubstrate + CanonicalState,
    F: Fn() -> G,
{
    let mut graph = make();
    let a = graph.add_node("social_class").unwrap();
    let b = graph.add_node("social_class").unwrap();
    assert!(
        graph
            .edge_attribute("solidarity", a, b, "solidarity/strength")
            .is_err(),
        "no edge was ever added — never a default 0.0"
    );
}

/// A non-strength qname is loud, never silently resolved to the strength value or a default 0.0 —
/// T2 stores exactly one edge attribute per edge (D32); T3 (ADR198 R1) widens this.
fn edge_attribute_of_an_unstored_qname_is_loud<G, F>(make: &F)
where
    G: GraphSubstrate + CanonicalState,
    F: Fn() -> G,
{
    let mut graph = make();
    let a = graph.add_node("social_class").unwrap();
    let b = graph.add_node("social_class").unwrap();
    graph.add_edge("solidarity", a, b, 0.5).unwrap();
    assert!(
        graph
            .edge_attribute("solidarity", a, b, "solidarity/tension")
            .is_err(),
        "T2 stores <edge-type>/strength only — a different edge-owned qname is 'never written', \
         not the strength value"
    );
}

/// **Deliberate: the OWNER segment is NOT checked here (adversarial review, Major 6 residue).**
/// `edge_attribute` performs no ownership validation of its own — exactly `node_attribute`'s own
/// division of labor with `check_node_referent_type` (evaluator-side, upstream of every call this
/// trait receives). A qname whose owner segment names a DIFFERENT `EdgeType` than `edge_type`
/// still SUCCEEDS, because only the ATTRIBUTE segment (`strength`) is checked. Pinned here so a
/// future reader does not "fix" the suffix check into an owner check by surprise.
fn edge_attribute_does_not_check_the_owner_segment<G, F>(make: &F)
where
    G: GraphSubstrate + CanonicalState,
    F: Fn() -> G,
{
    let mut graph = make();
    let a = graph.add_node("social_class").unwrap();
    let b = graph.add_node("social_class").unwrap();
    graph.add_edge("solidarity", a, b, 0.5).unwrap();
    assert_eq!(
        graph
            .edge_attribute("solidarity", a, b, "tenancy/strength")
            .unwrap(),
        0.5,
        "the owner segment ('tenancy' vs the edge's real type 'solidarity') is not verified here \
         — deliberately, by design; ownership is the CALLER's obligation"
    );
}
```

  Add all four to `run_substrate_conformance`'s dispatch list (after
  `node_type_of_a_dangling_id_is_loud_not_untyped(&make);`).

- [ ] **Step 6: Six legs + commit** `feat(graph): edge_attribute — the one new GraphSubstrate
  method T2 needs (D32, full-qname-keyed, forward-shaped for T3)`.

### Task 2: The D32 implicit-strength wiring decision

**Files:**
- Modify: `rust/crates/babylon-tick/src/lib.rs` (`prepare_rules`'s `TypeEnv` construction,
  `:127-130`, plus its `use babylon_bsl::declarations::parse_intrinsic_decls;` import line)
- Test: `rust/crates/babylon-tick/src/lib.rs`'s own `#[cfg(test)] mod tests` (the
  `a_declared_vocabulary_typo_refuses_through_the_production_seam` precedent's home)

**Interfaces:**
- Produces: `TypeEnv.fields` — built in `prepare_rules`, threaded to BOTH the load-time typechecker
  (`LoadContext.types`, `typecheck_aggregation`/`resolve_field`) and the runtime evaluator
  (`EvalEnv.types`, `field_of_node`/`field_of_edge`) — carries one `<edge-type>/strength` entry per
  `EdgeType` the scenario's `defvocabulary` declared, without requiring a hand-written `deffield`.

**Wiring decision (dossier §5, resolved — option (a), refined):** wire the seed into
`babylon-tick/src/lib.rs::prepare_rules`'s existing `TypeEnv` construction, using
`scenario.vocabulary: Option<ClosedVocabulary>` (built from the scenario's own `defvocabulary`
forms, `scenario.rs:282-289`) — **not** `scenario.edge_types` (the node/edge-count census the
dossier's own text pointed at). `scenario.vocabulary` is ALREADY threaded to
`LoadContext.vocabulary_registry` at `lib.rs:235`, so the exact object this wiring needs is already
in scope at the wiring site, zero new plumbing. `FieldRegistry::type_env_fields()` (`declarations.rs`,
already `pub`) returns exactly `TypeEnv.fields`'s own type (`HashMap<String, FieldDecl>`) — the
merge is a direct `HashMap` extend, not a re-derivation.

**This choice is a RULING, not merely a type-shape convenience (adversarial review Major 8,
corrected).** A `FieldDecl { ty: BslType::Coefficient, kind: FieldKind::Extensive }` is trivially
constructible directly from `scenario.edge_types`' keys (the census) WITHOUT `ClosedVocabulary` at
all — the "wrong type shape" framing this plan's first draft gave is not, by itself, a sufficient
reason to prefer `FieldRegistry`. The REAL reasons, stated honestly: (a) `FieldRegistry::
with_implicit_edge_strength` is ALREADY-BUILT, ALREADY-TESTED production-shaped machinery
(`r9_chapters.rs`'s own `type_env()` fixture already exercises it) — reusing it is DRY over hand-
rolling an equivalent seed loop against the census; (b) it is the **Phase-2-shaped call**
(`declarations.rs`'s own module doc: "rule-side `deffield` CONTENT PACKS ... are what will wire it
in") — a census-derived shortcut is a dead end Phase 2's content-pack registries would have to
replace outright, where wiring `FieldRegistry` now is the first real step toward that direction, not
a parallel path. **Consequence, recorded honestly, not smoothed over:** this is a genuine
NARROWING of D32's literal text ("needs no `deffield`" — unconditionally, for every `EdgeType`).
Under this wiring, a scenario declaring an `EdgeType` member WITHOUT ALSO declaring a
`defvocabulary EdgeType` block for it gets NO implicit seeding (`scenario.vocabulary` is `None`
when no `defvocabulary` form appears at all — the opt-in-per-scenario contract `scenario.rs:281-289`
itself documents: "`None` for a scenario declaring none — opt-in per scenario, so every EXISTING
content set ... is unaffected"; this is a distinct ruling from F1, which is `vocabulary.rs`'s own
EventType/enum-kind-absence inertness at `check_enum_ref`, not this field's own doc). The BARE
accessor (`field-of it <edge-type>/strength`, no aggregation) still works regardless, via
`tick::bind_field_value`'s graceful fallback — only an UNWEIGHTED FOLD/aggregation over the implicit
field needs the seeding, and therefore needs `defvocabulary EdgeType` declared. Task 6's own e2e
fixture is living proof of this requirement: it declares `(defvocabulary EdgeType (SOLIDARITY))`
for exactly this reason — Task 2 Step 4's zero-regression check should also confirm this
requirement is a real, load-bearing one (not merely stated), by pointing at Task 6's own aggregation
vector as the requirement's positive proof.

**A stronger PLAN-MUST-VERIFY resolution than the dossier's framing.** The dossier flagged "does the
FieldRegistry/TypeEnv/typecheck_aggregation chain compile and work" as unverified without a cargo
run. This plan author's own direct reading closes MOST of that gap without needing to run cargo:
`rust/crates/babylon-bsl/tests/r9_chapters.rs`'s own `type_env()` fixture (lines ~110-130) ALREADY
calls `FieldRegistry::with_implicit_edge_strength(&v).type_env_fields()` today, and
`c1_edge_and_hyperedge_attributes::the_edge_condition_coverage_row_is_writable_as_sum_and_as_mean`
(same file, ~line 138) ALREADY asserts `typecheck_aggregation(&e("(sum solidarity/strength)"),
&type_env())` returns `Ok` — this exact chain is proven correct and GREEN on `dev` today, in
isolation. **What remains genuinely unverified, and is this task's real PLAN-MUST-VERIFY item, is
narrower: does WIRING this chain into `babylon-tick::prepare_rules` compile and reach
`typecheck_aggregation` through the real production seam** (`run_once_into`'s own load path, not a
hand-built `TypeEnv`) — that needs an actual implementation attempt, so it is Step 1 below.

- [ ] **Step 1 (RED — the PLAN-MUST-VERIFY experiment): a load-only probe through the real seam.**
  Add to `lib.rs`'s test module (siblings of the existing `VOCAB_WIRING_SCENARIO`/
  `a_declared_vocabulary_typo_refuses_through_the_production_seam` pair):

```rust
    // T2 (issue #559) PLAN-MUST-VERIFY probe: proves the D32 implicit-strength seed reaches
    // typecheck_aggregation through prepare_rules's REAL production wiring, using only
    // already-served slice-1 infrastructure (neighbors) — provable BEFORE edges/field-of-over-
    // EdgeRef land in PR B (Tasks 3-5). The rule LOADS today only if the wiring works; it would
    // still refuse E-EVAL-033 if actually RUN (its fold body reads a NodeRef's field-of an
    // edge-owned qname, a referent-type mismatch) — this probe tests LOADING only, deliberately.
    const D32_WIRING_PROBE_SCENARIO: &str = r"
(scenario ft/d32-wiring-probe
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defvocabulary EdgeType (SOLIDARITY))
  (deffield social-class/shape int extensive)
  (node core NodeType/SOCIAL_CLASS (social-class/shape 1))
  (node other NodeType/SOCIAL_CLASS (social-class/shape 1))
  (edge EdgeType/SOLIDARITY core other 0.5))
";
    const D32_WIRING_PROBE_RULE: &str = r#"(rule vitality/d32-wiring-probe
  :material-basis "PLAN-MUST-VERIFY probe (T2, issue #559): the D32 implicit-strength field must resolve through prepare_rules's real TypeEnv construction, not merely in isolation (r9_chapters.rs::type_env already proves the isolated chain)"
  :fuel 128
  (bindings (binding shape :field social-class/shape))
  (when (= shape 1))
  (effects (emit EventType/PROBE
    (s (fold sum (neighbors self EdgeType/SOLIDARITY :any NodeType/SOCIAL_CLASS)
             (field-of it solidarity/strength))))))"#;

    #[test]
    fn the_d32_implicit_strength_field_resolves_through_the_real_wiring_seam() {
        let mut graph = babylon_graph::hypergraph_store::HypergraphStore::new();
        let result = prepare_rules(D32_WIRING_PROBE_SCENARIO, D32_WIRING_PROBE_RULE, &mut graph);
        assert!(
            result.is_ok(),
            "the D32 implicit-strength field must resolve through prepare_rules's real \
             TypeEnv construction: {result:?}"
        );
    }
```

  Run it. Expected: FAIL — `resolve_field`'s "unknown field: 'solidarity/strength'" (`typecheck.rs:247-252`),
  surfacing as the rule-rejected error `prepare_rules` returns.

- [ ] **Step 2: Implement the wiring.** In `lib.rs`, change the import line and the `TypeEnv`
  construction:

```rust
use babylon_bsl::declarations::{parse_intrinsic_decls, FieldRegistry};
```

```rust
    // The scenario's `deffield` forms ARE the registries for slice 1 ... [existing comment
    // unchanged] ... D32 (bsl-language.rst §2.9): every EdgeType carries one implicit
    // <edge-type>/strength field, needing no deffield. FieldRegistry::with_implicit_edge_strength
    // already builds this seed set, fully tested (declarations.rs, r9_chapters.rs's own type_env()
    // fixture) but had no production caller until T2 (issue #559) — this is that caller. Seeded
    // from the scenario's declared defvocabulary EdgeType members (scenario.vocabulary; `None` for
    // a scenario declaring no defvocabulary at all — the loop below is then a no-op, matching every
    // pre-T2 scenario's behavior exactly). An explicit deffield re-declaring an implicit field is
    // D32's own named violation (E-LOAD-001, `FieldRegistry::declare`'s own duplicate guard) —
    // checked here rather than through that guard, because scenario.rs's simpler load_deffield
    // builds `scenario.fields` with no notion of "implicit" to check against; this is that check's
    // only home until Phase 2's content-pack field registries replace scenario.fields wholesale
    // (declarations.rs's own module doc).
    let mut fields = scenario.fields.clone();
    if let Some(vocabulary) = scenario.vocabulary.as_ref() {
        let implicit = FieldRegistry::with_implicit_edge_strength(vocabulary).type_env_fields();
        for (qname, decl) in implicit {
            if fields.contains_key(&qname) {
                return Err(format!(
                    "E-LOAD-001: {qname} is the implicit <edge-type>/strength field (D32) — \
                     re-declaring it with an explicit deffield is a duplicate declaration, never \
                     a silent override (bsl-language.rst §2.9)"
                ));
            }
            fields.insert(qname, decl);
        }
    }
    let types = TypeEnv {
        fields,
        exemptions: &[],
    };
```

- [ ] **Step 3: Run Step 1's test.** Expected: PASS.

- [ ] **Step 4: The collision-refusal test (proves the E-LOAD-001 half, not just the happy path).**
  Add a second test: same scenario but with a rule-irrelevant EXPLICIT
  `(deffield solidarity/strength int extensive)` line added to `D32_WIRING_PROBE_SCENARIO`'s body
  (a second const, `D32_WIRING_PROBE_SCENARIO_WITH_EXPLICIT_REDECLARATION`); assert
  `prepare_rules(...)` returns `Err` containing `"E-LOAD-001"`. **Zero-regression check (verified,
  not assumed):** `rg -n "strength" rust/crates/babylon-tick/content/scenarios/*.bscn` returns no
  hits today — no committed scenario explicitly declares an edge-owned `strength` field, so this
  new refusal cannot regress any existing content.

- [ ] **Step 5: Mutation evidence.** Comment out the `if let Some(vocabulary) = ...` block (restore
  the bare `scenario.fields.clone()`) → Step 1's probe test flips red (the exact pre-Step-2 failure).
  Restore byte-identical.

- [ ] **Step 6: Six legs (including `tick_goldens.rs` — must stay byte-identical; this task touches
  no shipped content) + commit** `feat(tick): wire the D32 implicit-strength field into
  prepare_rules's real TypeEnv construction (issue #559)`.

**PR A ends here.**

---

### Task 3: `EdgeKey`, `Value::EdgeRef`, `Element::Edge`, and the `edges` query head

**Files:**
- Modify: `rust/crates/babylon-bsl/src/query.rs` (`EdgeKey`, `Element::Edge` — dropping `Copy`,
  `materialize_edges`, `UNSERVED_QUERY_HEADS`, the `element_kind_name` trip-wire, `use` import)
- Modify: `rust/crates/babylon-bsl/src/evaluator.rs` (`Value::EdgeRef`, `SERVED_QUERY_HEADS`,
  `UNSERVED_EXPRESSION_HEADS` — array AND doc prose, the nine `Element`-Copy-dependent call sites,
  the `use` import, `refusal_messages_name_their_slice`'s `edges` leg)
- Modify: `rust/crates/babylon-bsl/tests/r9_chapters.rs` (the landed four-heads selection-refusal
  test, shrinking to three)
- Modify: `rust/crates/babylon-bsl/tests/conformance_corpus.rs` (promote `event_edge_count.bsl`'s
  pinned-at-load test to a real evaluated pair)

**Interfaces:**
- Produces: `query::EdgeKey { source: NodeId, target: NodeId, edge_type: String }`,
  `Element::Edge(EdgeKey)`, `Value::EdgeRef(EdgeKey)`, `(edges <enum-ref>)` evaluating for real.

**Design decisions, made explicitly per the module's own standing instructions:**

1. **`EdgeKey`'s field order is `(source, target, edge_type)`, not `(edge_type, source, target)`.**
   §2.6's own total order for edges is "ascending `(source-id, target-id, edge-type)` lexicographic
   byte order" (`bsl-language.rst:1040-1046`). Declaring the struct fields in THAT order means
   `#[derive(Ord)]`'s natural field-by-field lexicographic comparison matches the spec's order —
   **a property of `EdgeKey` itself, proven by its own direct unit test (Step 9), NOT a claim about
   `materialize_edges`.** `materialize_edges`' own output order comes ENTIRELY from
   `GraphSubstrate::edges`' own contract (`memory.rs:220-228`/`hypergraph_store.rs:270-278` both
   `sort_unstable()` a `Vec<(NodeId, NodeId)>` BEFORE this function ever sees it, and `edge_type` is
   constant across one query so it never enters that sort at all) — `EdgeKey`'s own derived `Ord` is
   NEVER INVOKED by `materialize_edges`'s code path (adversarial review Major 4, verified true: the
   first draft of this plan conflated the two). `EdgeKey`'s field-order choice therefore matters for
   a SEPARATE, forward-looking reason: the moment anything ever sorts a `Vec<Element>`/`Vec<EdgeKey>`
   directly via the derive (design decision 2, below), that sort should already agree with §2.6.
2. **`Element`'s cross-kind `Ord` is RULED here, not silently inherited from the derive (CT4P A5,
   issue #525 — the same finding `query.rs`'s own doc on `Element` names).** §2.6 defines a total
   order WITHIN each query kind's own result set only and is silent on cross-kind comparison (no
   production `materialize()` call ever mixes kinds — `edges` returns only `Edge`, `nodes`/
   `neighbors` only `Node`). T2 rules it: `Node` sorts before `Edge`, by declaration order — an
   arbitrary but DELIBERATE, DOCUMENTED, TESTED choice, filed as a register row (Task 7), unreachable
   in production by construction but pinned per the enum's own standing instruction.
3. **`Element` drops `#[derive(Copy)]` (adversarial review Blocker 1, verified: `query.rs:50`).**
   `EdgeKey` owns a `String` and cannot be `Copy`; `Element::Edge(EdgeKey)` therefore forces
   `Element` itself to drop `Copy`. Nine call sites depend on it today, all enumerated and fixed in
   Step 4 below — none needs anything beyond the mechanical pattern that step gives.

- [ ] **Step 1: Red — the exact-triple readability test.** In `query.rs`'s test module, add
  (mirroring `query_materialization_charges_the_3_7_query_base`'s fuel-charge style, small and
  hand-readable — the M2 doc below explains why this alone does not discriminate the ordering law):

```rust
    #[test]
    fn edges_materializes_in_ascending_source_target_order_and_charges_query_base() {
        let mut graph = MemoryGraph::new();
        let a = graph.add_node("SOCIAL_CLASS").unwrap();
        let b = graph.add_node("SOCIAL_CLASS").unwrap();
        let c = graph.add_node("SOCIAL_CLASS").unwrap();
        // Insertion order deliberately SHUFFLED against ascending (source, target).
        graph.add_edge("SOLIDARITY", c, a, 0.9).unwrap();
        graph.add_edge("SOLIDARITY", a, b, 0.1).unwrap();
        graph.add_edge("SOLIDARITY", a, c, 0.5).unwrap();
        let costs = costs();
        let mut fuel = 10;
        let result =
            materialize_src("(edges EdgeType/SOLIDARITY)", &graph, &costs, &mut fuel).unwrap();
        let keys: Vec<EdgeKey> = result
            .iter()
            .map(|element| match element {
                Element::Edge(key) => key.clone(),
                Element::Node(_) => panic!("edges must materialize only Edge elements"),
            })
            .collect();
        assert_eq!(
            keys,
            vec![
                EdgeKey { source: a, target: b, edge_type: "SOLIDARITY".to_owned() },
                EdgeKey { source: a, target: c, edge_type: "SOLIDARITY".to_owned() },
                EdgeKey { source: c, target: a, edge_type: "SOLIDARITY".to_owned() },
            ],
            "ascending (source-id, target-id) — §2.6; materialize_edges performs no sort of its \
             own, it maps graph.edges()'s ALREADY-sorted output (see design decision 1 above)"
        );
        // QUERY_BASE(1) + enum-ref(0) = 1.
        assert_eq!(fuel, 9);
    }
```

  Expected: FAIL to compile (`EdgeKey`/`Element::Edge` do not exist).

- [ ] **Step 2: Red — the shuffled discriminating vector (adversarial review Major 5).** A 3-element
  fixture is not a real proof: roughly 1 in 6 orderings of 3 items happens to be ascending by
  accident, and `HashMap` iteration order (what `MemoryGraph`/`HypergraphStore` actually store edges
  in before `edges()`'s own `sort_unstable()` runs) never preserves insertion order anyway, so the
  small test above only checks that SOME sort ran, not that it is genuinely governed by
  `(source, target)` rather than some other accidental order. Mirror the exact M2 repair
  `nodes_materializes_in_ascending_id_order` already made for nodes (`query.rs:290-309`: "Fifty
  nodes makes an accidental sorted match astronomically unlikely"):

```rust
    #[test]
    fn edges_materializes_in_ascending_source_target_order_at_scale() {
        const N: usize = 50;
        let mut graph = MemoryGraph::new();
        let mut ids = Vec::with_capacity(N);
        for _ in 0..N {
            ids.push(graph.add_node("SOCIAL_CLASS").unwrap());
        }
        // Add edges over a SHUFFLED pairing of the id space (every id i -> id (i*17+3) % N,
        // a fixed permutation with no monotonic relationship to insertion/id order) so the
        // resulting edge set's ascending order cannot coincide with any single simple pattern
        // the loop itself might accidentally produce.
        for (i, &from) in ids.iter().enumerate() {
            let to = ids[(i * 17 + 3) % N];
            if from != to {
                graph.add_edge("SOLIDARITY", from, to, 0.1).unwrap();
            }
        }
        let costs = costs();
        let mut fuel = 10_000;
        let result =
            materialize_src("(edges EdgeType/SOLIDARITY)", &graph, &costs, &mut fuel).unwrap();
        let pairs: Vec<(NodeId, NodeId)> = result
            .iter()
            .map(|element| match element {
                Element::Edge(key) => (key.source, key.target),
                Element::Node(_) => panic!("edges must materialize only Edge elements"),
            })
            .collect();
        assert!(
            pairs.windows(2).all(|w| w[0] < w[1]),
            "materialized edges must be strictly ascending by (source, target): {pairs:?}"
        );
    }
```

  Expected: FAIL to compile (same reason as Step 1).

- [ ] **Step 3: Mint `EdgeKey` and `Element::Edge` — `Element` loses `Copy`.**

```rust
/// A materialized edge's identity — the `(source, target, edge_type)` triple IS the identity
/// (§2.10's own "well-defined because the triple is a key" ruling, `bsl-language.rst:1896-1904`);
/// `GraphSubstrate` mints no separate `EdgeId` (only `NodeId`/`HyperedgeId` exist,
/// `substrate.rs:33,41`). Field order is `(source, target, edge_type)` DELIBERATELY — see design
/// decision 1 above (this crate's own direct Ord test, Step 9, is where this is actually
/// exercised — NOT `materialize_edges`, which never invokes this derive).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdgeKey {
    pub source: NodeId,
    pub target: NodeId,
    pub edge_type: String,
}
```

  Update `Element` — **`Copy` is REMOVED from the derive list** (design decision 3):

```rust
/// One materialized graph element (§2.6).
///
/// **T2's cross-kind Ord ruling (register row D140, CT4P A5 / issue #525, T2 issue #559).** §2.6
/// defines a total order WITHIN each query kind's own result set only — it is silent on comparing a
/// `Node` to an `Edge`. No production `materialize()` call ever mixes kinds (`edges` returns only
/// `Edge`; `nodes`/`neighbors` only `Node`), so this ordering is UNREACHABLE in practice — pinned
/// anyway, per this enum's own standing instruction, rather than left to whatever `#[derive(Ord)]`
/// happens to produce from declaration order. RULED: `Node` sorts before `Edge`, by declaration
/// order below — arbitrary, deliberate, tested (`tests::node_sorts_before_edge_regardless_of_id`).
///
/// **No longer `Copy` (T2, issue #559): `EdgeKey` owns a `String`.** Every call site that relied on
/// `Copy` is fixed at this variant's landing (see evaluator.rs's own Task-3 call-site fixes) —
/// `Clone` is unaffected and remains the currency for every place that needs an owned `Element`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Element {
    /// A materialized node — see the module doc.
    Node(NodeId),
    /// A materialized dyadic edge (slice 2, T2). Declared SECOND: see this enum's own cross-kind
    /// Ord ruling above.
    Edge(EdgeKey),
}

impl Element {
    /// This element's runtime value. Takes `&self` (not `self`) now that `Element` is no longer
    /// `Copy` — every existing caller already holds a `&Element` at the point it calls this
    /// (`env.elements`'s own `(Option<String>, Element)` tuples are read by reference throughout),
    /// so no caller needs to change; only the signature does, and the `Edge` arm clones its key.
    #[must_use]
    pub fn to_value(&self) -> Value {
        match self {
            Self::Node(id) => Value::NodeRef(*id),
            Self::Edge(key) => Value::EdgeRef(key.clone()),
        }
    }
}
```

  Update the `use` import at the top of `query.rs` if `EdgeKey` needs no new import there (it is
  minted in this module), but add the corresponding import in `evaluator.rs` (Step 6 below).

  Update the compile-time trip-wire in the test module — it also drops the by-value parameter that
  only compiled under `Copy`:

```rust
    fn element_kind_name(element: &Element) -> &'static str {
        match element {
            Element::Node(_) => "node",
            Element::Edge(_) => "edge",
        }
    }
```

  Fix the one caller of `element_kind_name` that relied on move-then-reuse
  (`element_kind_name(a)` at `query.rs:511`, followed by reusing `a` at `:512-513` — only compiled
  because `Element: Copy`; now `element_kind_name(&a)`):

```rust
    #[test]
    fn element_ordering_matches_ascending_node_id() {
        let a = Element::Node(NodeId(1));
        let b = Element::Node(NodeId(2));
        assert_eq!(element_kind_name(&a), "node");
        assert!(a < b);
        assert!(b >= a);
        assert_eq!(Element::Node(NodeId(7)), Element::Node(NodeId(7)));
    }

    /// T2's cross-kind Ord ruling, value-pinned: Node < Edge regardless of id/key magnitude.
    #[test]
    fn node_sorts_before_edge_regardless_of_id() {
        let node = Element::Node(NodeId(u64::MAX));
        let edge = Element::Edge(EdgeKey {
            source: NodeId(0),
            target: NodeId(0),
            edge_type: String::new(),
        });
        assert_eq!(element_kind_name(&node), "node");
        assert_eq!(element_kind_name(&edge), "edge");
        assert!(node < edge, "T2's ruling: kind dominates value");
    }
```

- [ ] **Step 4: Fix the remaining `Element`-Copy-dependent call sites in `evaluator.rs`
  (adversarial review Blocker 1, all nine sites verified by direct reading, PLUS a tenth
  `to_value`-caller site named for completeness — three of the ten needed no change, covered by
  Step 3's signature change alone).** **Correction to this step's own first draft:** "iterator
  expression only, leaving every loop body untouched" holds for FOUR of the five loops, not all
  five — `eval_selection`'s own loop (`:1001`) also needs one internal `.clone()`, below.

  - **`evaluator.rs:949,1081,1110,1190` — `for &element in elements` / `for &element in
    &elements`.** Each loop feeds an OWNED `Element` into `with_element`/`eval_body_and_weight`
    (both take `element: Element` by value, `evaluator.rs:689-703,774-789`) — change the iterator
    expression only, leaving every loop BODY untouched:
    ```rust
    for element in elements.iter().cloned() {
    ```
    (replacing `for &element in elements` or `for &element in &elements` respectively — `.iter()`
    works identically whether `elements` is `&[Element]` or an owned `Vec<Element>`/`&Vec<Element>`
    at each call site, and `.cloned()` requires only `Element: Clone`, which is untouched by this
    task).
  - **`evaluator.rs:1001` — `eval_selection`'s own loop is the ONE exception: `element` is consumed
    THREE times per iteration** (`with_element(env, elem_name.clone(), element)` unconditionally,
    then `best_element = element;` in EITHER the `None =>` arm or the `Some(prev_best) =>` arm's
    `strictly_better` branch — mutually exclusive, so only one of the two ever fires per iteration,
    but `with_element`'s own consumption happens on EVERY iteration regardless). Apply the SAME
    iterator-expression change as the other four loops, but ALSO clone at the `with_element` call
    ONLY — leave both `best_element = element;` assignments as plain moves, unchanged:
    ```rust
    for element in elements.iter().cloned() {
        let child = with_element(env, elem_name.clone(), element.clone());
        let score = evaluate(score_expr, &child, host, fuel)?;
        best_score = Some(match best_score {
            None => {
                best_element = element;
                score
            }
            Some(prev_best) => {
                let strictly_better =
                    matches!(apply_ordering(op, &score, &prev_best)?, Value::Bool(true));
                if strictly_better {
                    best_element = element;
                    score
                } else {
                    prev_best
                }
            }
        });
    }
    ```
    (This compiles because `element.clone()` at the `with_element` call leaves the ORIGINAL
    `element` binding intact and unmoved; the two `best_element = element;` assignments are each the
    LAST use of `element` in their own, mutually-exclusive branch, so moving it there is legal
    exactly as it was before this task, no `.clone()` needed at either.)
  - **`evaluator.rs:999` — `let mut best_element = elements[0];`** (inside `eval_selection`, an
    index-and-move that only compiled under `Copy`):
    ```rust
    let mut best_element = elements[0].clone();
    ```
  - **`evaluator.rs:396,420,1021` — `element.to_value()` / `best_element.to_value()` called on an
    already-owned or already-referenced `Element`** (`:396,420`: from
    `env.elements.last().map(|(_, element)| element.to_value())` and the `:as`-name lookup's
    equivalent, where `element` is already `&Element`; `:1021`: `Ok(best_element.to_value())`,
    `eval_selection`'s own final return, where `best_element` is an owned `Element` local). **No
    change needed at any of the three**: `to_value` now takes `&self` (Step 3), and Rust's method
    resolution calls `Element::to_value(&element)`/`Element::to_value(&best_element)` via auto-ref
    at all three sites regardless of whether the receiver is already a reference or an owned value —
    exactly as it worked before this task.

- [ ] **Step 5: `materialize_edges`.** Mirroring `materialize_nodes` exactly, including its refusal
  of the `<edge-pred>` operand (zero exercised vectors — same reasoning `materialize_nodes`' own doc
  gives):

```rust
/// `(edges <enum-ref> <edge-pred>?)`. Like `nodes`' `<node-pred>`, the `<edge-pred>` operand is a
/// real §2.6 grammar production with zero exercised conformance vectors and zero content rules
/// (T2 scout dossier §1.1 point 4) — refused loudly by name, mirroring `materialize_nodes` exactly,
/// rather than served on an unreviewed reading.
///
/// **Performs no sort of its own.** `GraphSubstrate::edges` already returns a canonically sorted
/// `Vec<(NodeId, NodeId)>` (both backends' `sort_unstable()`, before this function ever runs) — this
/// maps that ALREADY-ordered output element-for-element; `EdgeKey`'s own `Ord` derive is never
/// consulted here (design decision 1, above).
fn materialize_edges(
    items: &[SExpr],
    env: &EvalEnv<'_>,
    fuel: &mut u64,
) -> Result<Vec<Element>, EvalError> {
    charge(fuel, cost::QUERY_BASE)?;
    let [_, type_ref, extra @ ..] = items else {
        return Err(EvalError::plain(
            "(edges <enum-ref> <edge-pred>?) — missing the EdgeType operand",
        ));
    };
    if !extra.is_empty() {
        return Err(EvalError::plain(
            "(edges <enum-ref> <edge-pred>) — the element-predicate operand is a real §2.6 \
             production this evaluator does not yet serve (no exercised vector or content rule \
             needs it); T2 serves the unpredicated (edges <enum-ref>) form only",
        ));
    }
    let edge_type = enum_member(type_ref)?;
    let graph = require_graph(env, "edges")?;
    Ok(graph
        .edges(edge_type)
        .into_iter()
        .map(|(source, target)| {
            Element::Edge(EdgeKey {
                source,
                target,
                edge_type: edge_type.to_owned(),
            })
        })
        .collect())
}
```

  Wire it into `materialize`'s dispatch (`"edges" => materialize_edges(items, env, fuel),`, alongside
  `"nodes"`/`"neighbors"`) and remove `("edges", "slice 2")` from `UNSERVED_QUERY_HEADS` — **bump its
  type annotation from `[(&str, &str); 4]` to `[(&str, &str); 3]`** (now `hyperedges`/`members-of`/
  `hyperedges-of`). Update `edges_hyperedges_and_members_of_name_their_slice`'s test to drop the
  `edges` case (or rename it `hyperedges_and_members_of_name_their_slice`, dropping the
  now-inaccurate part of the name).

- [ ] **Step 6: Wire `evaluator.rs`.** Add the import: `use crate::query::{EdgeKey, Element};`
  (widening the existing `use crate::query::Element;` line). Add `Value::EdgeRef(EdgeKey)` to the
  `Value` enum (doc: `"EdgeRef" (§3.1) — produced by edges/edge-between query elements. No
  arithmetic, no ordering; refs are identities, same discipline as NodeRef/HyperedgeRef.`). Add
  `"edges"` to `SERVED_QUERY_HEADS` — **bump its type annotation from `[&str; 2]` to `[&str; 3]`**
  (now `["nodes", "neighbors", "edges"]`). Remove `("edges", "slice 2")` from
  `UNSERVED_EXPRESSION_HEADS` — **bump its type annotation from `[(&str, &str); 8]` to
  `[(&str, &str); 7]`** (`edge-between`/`the` still there until Task 4 and the disposition record
  respectively). **Also fix the stale prose in `UNSERVED_EXPRESSION_HEADS`'s own doc comment**
  (`evaluator.rs:507`, "`edges`/`edge-between`/`the` (slice 2, the dyadic edge lane)") — `rustdoc -D
  warnings` does not catch stale prose, only broken links/malformed syntax, so this needs a manual
  edit: change that clause to `` `the` (slice 2, the carrier-read lane — `edges`/`edge-between`
  served as of T2, issue #559) ``.

- [ ] **Step 7: Sweep the three landed edges-refusal assertions this task flips (NEW BLOCKER,
  adversarial review — the plan's own Task 7 Step 1 grep claim was FALSE: `event_edge_count.bsl`
  DOES use `edges`; verified directly). Serving `edges` for real flips FOUR landed assertions total
  — `query.rs`'s own `edges_hyperedges_and_members_of_name_their_slice` (Step 5, above) is the
  first; the three below are the rest, and none of them is optional cleanup — an un-swept one is a
  RED test the moment this step's own predecessors land.**

  1. **`evaluator.rs:2231-2233`, inside `refusal_messages_name_their_slice`.** Delete the `edges`
     leg entirely (it has nothing left to refuse):
     ```rust
     // DELETE these three lines:
     // // `edges` is unserved until slice 2.
     // let edges_err = eval("(edges EdgeType/SOLIDARITY)").unwrap_err();
     // assert!(edges_err.message.contains("slice 2"), "{edges_err}");
     ```
     The function's own doc comment ("After the split, each refusal names what it actually is")
     needs no change — it is still true of every OTHER leg in the test; only the `edges` leg itself
     is removed.

  2. **`r9_chapters.rs:1211-1235`, `the_other_four_selection_heads_stay_pinned_named_by_their_slice`
     (`tests/r9_chapters.rs`, `mod` `c5_...`).** Once `edges` evaluates, `(select-max (edges
     EdgeType/SOLIDARITY) it)` no longer errors "slice 2" — it materializes `[]` (an empty graph)
     and hits `select-max`'s OWN empty-query refusal, `E-EVAL-021` (§4.4/D45), a DIFFERENT code and
     a DIFFERENT message. The test's own name and membership shrink from four heads to three. Full
     replacement:
     ```rust
     /// The other THREE §2.6 query heads a selection could in principle run over stay refused at
     /// EVALUATION, each naming the slice that will serve it (Constraint 4) — never a silent skip
     /// and never an `E-LOAD-021` misdiagnosis. `edges` is no longer among them (T2, issue #559) —
     /// see `edge_count_evaluates_for_real_on_an_empty_graph` (`conformance_corpus.rs`) for its own
     /// positive vector.
     #[test]
     fn the_other_three_selection_heads_stay_pinned_named_by_their_slice() {
         // `self` binds to a REAL node: were it a dangling id, a future
         // referent-validation pass could fire before the slice refusal
         // and this vector would pin the wrong error (Copilot harvest,
         // #520).
         let mut graph = MemoryGraph::new();
         let subject = graph.add_node("SOCIAL_CLASS").unwrap();
         for (query, slice) in [
             ("(hyperedges HyperedgeType/ECONOMIC_SECTOR)", "slice 3"),
             ("(members-of self HyperedgeType/ECONOMIC_SECTOR)", "slice 3"),
             (
                 "(hyperedges-of self HyperedgeType/ECONOMIC_SECTOR)",
                 "slice 3",
             ),
         ] {
             let mut fuel = 1_000;
             let err = eval_expr(
                 &format!("(select-max {query} it)"),
                 &graph,
                 HashMap::from([("self".to_owned(), Value::NodeRef(subject))]),
                 &mut fuel,
             )
             .unwrap_err();
             assert!(err.message.contains(slice), "{query}: {err}");
         }
     }
     ```

  3. **`conformance_corpus.rs:799-817`, `edge_count_stays_pinned_and_names_slice_2` — PROMOTE, not
     patch.** That module's own doc (`conformance_corpus.rs:15-29`) already promises this exact
     promotion: "`event_edge_count.bsl` is the one exception ... it stays pinned at load only" —
     T2 is the train that discharges the promise. `(fold count (edges EdgeType/SOLIDARITY) it)`
     over an EMPTY graph now evaluates to `Value::Int(0)` (`fold_count` has no empty-query refusal
     — verified, `evaluator.rs:1044-1052`, unlike `select-max`/`mean`/`min`/`max`), so `(>= 0 1)` is
     `Value::Bool(false)`: `.unwrap_err()` would PANIC, not merely assert the wrong thing. Rename
     and split into a positive pair — the empty-graph case (what the old test's own fixture already
     built) plus a non-empty companion:
     ```rust
     /// **T2 (issue #559, Task 3): event_edge_count.bsl now evaluates for real**, promoted from the
     /// load-only pin `edge_count_stays_pinned_and_names_slice_2` used to be — `edges` is served as
     /// of T2. An empty graph has zero SOLIDARITY edges: `(fold count (edges EdgeType/SOLIDARITY)
     /// it)` is `0`, and `(>= 0 1)` is `false` — the exact vector T2 exists to unblock.
     #[test]
     fn edge_count_evaluates_for_real_on_an_empty_graph() {
         let loaded = load(EDGE_COUNT, "x.bsl").unwrap();
         let graph = MemoryGraph::new();
         let costs = IntrinsicCosts::default();
         let env_map = bind_environment(&loaded.bindings, &HashMap::new())
             .expect("EDGE_COUNT's bindings are empty — nothing to resolve");
         let env = EvalEnv {
             bindings: env_map,
             intrinsic_costs: &costs,
             graph: Some(&graph as &dyn GraphSubstrate),
             types: None,
             enums: None,
             elements: Vec::new(),
         };
         let mut fuel = 10_000;
         let result =
             evaluate(when_clause(&loaded), &env, &EmptyIntrinsicHost, &mut fuel).unwrap();
         assert_eq!(result, Value::Bool(false), "an empty graph has zero SOLIDARITY edges");
     }

     /// The non-empty companion: one SOLIDARITY edge is enough to cross the `>= 1` threshold.
     #[test]
     fn edge_count_evaluates_for_real_on_a_non_empty_graph() {
         let loaded = load(EDGE_COUNT, "x.bsl").unwrap();
         let mut graph = MemoryGraph::new();
         let a = graph.add_node("SOCIAL_CLASS").unwrap();
         let b = graph.add_node("SOCIAL_CLASS").unwrap();
         graph.add_edge("SOLIDARITY", a, b, 0.5).unwrap();
         let costs = IntrinsicCosts::default();
         let env_map = bind_environment(&loaded.bindings, &HashMap::new())
             .expect("EDGE_COUNT's bindings are empty — nothing to resolve");
         let env = EvalEnv {
             bindings: env_map,
             intrinsic_costs: &costs,
             graph: Some(&graph as &dyn GraphSubstrate),
             types: None,
             enums: None,
             elements: Vec::new(),
         };
         let mut fuel = 10_000;
         let result =
             evaluate(when_clause(&loaded), &env, &EmptyIntrinsicHost, &mut fuel).unwrap();
         assert_eq!(result, Value::Bool(true), "one SOLIDARITY edge crosses the >= 1 threshold");
     }
     ```
     Update BOTH of that module's own doc paragraphs that name the old pinned-at-load behavior:
     - The file-level module doc (`:15-29`): the sentence "`event_edge_count.bsl` is the one
       exception: its query is `(edges …)`, which slice 1 does not serve ... it stays pinned at
       load only, and evaluating it anyway is asserted to refuse LOUDLY, naming slice 2" becomes
       "`event_edge_count.bsl` ALSO executes for real now (T2, issue #559, slice 2's dyadic edge
       lane) — `edge_count_evaluates_for_real_on_an_empty_graph`/`..._on_a_non_empty_graph` below,
       promoted from the load-only pin `edge_count_stays_pinned_and_names_slice_2` used to be."
     - `aggregation_fixtures_load_and_bound`'s own doc comment (`:567-572`): "`EDGE_COUNT` is the
       one exception (`edges` is slice 2, `edge_count_stays_pinned_and_names_slice_2`)" becomes
       "`EDGE_COUNT`'s LOAD verdict is pinned here too, but it ALSO executes for real now (T2,
       issue #559) — see `edge_count_evaluates_for_real_on_an_empty_graph`/`..._on_a_non_empty_graph`
       below."
     **The two `EDGE_COUNT.replace(...)`-based tests further down this file
     (`intensive_aggregation_is_rejected_where_python_allowed_it`,
     `correction_2_unknown_aggregation_is_e_parse_015_not_false`) need NO change** — both assert
     LOAD-time failures (`E-TYPE-041`/`042`, `E-PARSE-015`) that fire before evaluation is ever
     reached, entirely independent of whether `edges` evaluates (verified directly).

  Add `rust/crates/babylon-bsl/tests/r9_chapters.rs` and
  `rust/crates/babylon-bsl/tests/conformance_corpus.rs` to this task's **Files** block (both were
  missing from the first draft).

- [ ] **Step 8: Run Steps 1-2's tests.** Expected: PASS.

- [ ] **Step 9: `EdgeKey`'s own Ord, directly (adversarial review Major 4's replacement mutation) —
  plus the delegation proof.**

```rust
    /// EdgeKey's own law, directly: field order is (source, target, edge_type), so `source`
    /// dominates `edge_type` in comparison — proven with a pair CONSTRUCTED so the two possible
    /// field orderings DISAGREE (a lower source but alphabetically LATER edge_type vs. a higher
    /// source but alphabetically EARLIER edge_type), making the field-declaration choice
    /// mutation-provable. This is independent of materialize_edges (design decision 1) — the direct
    /// test this crate's own trip-wire has been asking for since `Element`'s derive doc was written.
    #[test]
    fn edge_key_ord_prioritizes_source_over_edge_type() {
        let lower_source_higher_type = EdgeKey {
            source: NodeId(1),
            target: NodeId(2),
            edge_type: "ZZZZ".to_owned(),
        };
        let higher_source_lower_type = EdgeKey {
            source: NodeId(2),
            target: NodeId(1),
            edge_type: "AAAA".to_owned(),
        };
        assert!(
            lower_source_higher_type < higher_source_lower_type,
            "source must dominate edge_type — §2.6's (source-id, target-id, edge-type) order"
        );
    }

    /// materialize_edges' own ordering guarantee comes from GraphSubstrate::edges' own contract,
    /// NOT from EdgeKey's derived Ord (which this function never invokes) — proven by direct
    /// equality against the substrate's own output, not merely by eyeballing one hand-picked
    /// fixture (Step 1).
    #[test]
    fn edges_materializes_in_exactly_graph_edges_own_order() {
        let mut graph = MemoryGraph::new();
        let a = graph.add_node("SOCIAL_CLASS").unwrap();
        let b = graph.add_node("SOCIAL_CLASS").unwrap();
        let c = graph.add_node("SOCIAL_CLASS").unwrap();
        graph.add_edge("SOLIDARITY", c, a, 0.9).unwrap();
        graph.add_edge("SOLIDARITY", a, b, 0.1).unwrap();
        graph.add_edge("SOLIDARITY", a, c, 0.5).unwrap();
        let costs = costs();
        let mut fuel = 10;
        let result =
            materialize_src("(edges EdgeType/SOLIDARITY)", &graph, &costs, &mut fuel).unwrap();
        let materialized: Vec<(NodeId, NodeId)> = result
            .iter()
            .map(|element| match element {
                Element::Edge(key) => (key.source, key.target),
                Element::Node(_) => panic!("edges must materialize only Edge elements"),
            })
            .collect();
        assert_eq!(
            materialized,
            graph.edges("SOLIDARITY"),
            "materialize_edges must reproduce GraphSubstrate::edges' own order exactly, unchanged"
        );
    }
```

  **Mutation (proving `edge_key_ord_prioritizes_source_over_edge_type`, not the delegation test —
  the delegation test correctly does NOT flip under this mutation, which is itself part of the
  proof):** swap `EdgeKey`'s field declaration order to `(edge_type, source, target)` →
  `edge_key_ord_prioritizes_source_over_edge_type` flips (edge_type now dominates, so the assertion's
  direction reverses); `edges_materializes_in_exactly_graph_edges_own_order` stays green throughout
  (confirming it truly does not depend on `EdgeKey`'s Ord). Restore byte-identical.

- [ ] **Step 10: Six legs + commit** `feat(bsl): EdgeKey, Value::EdgeRef, Element::Edge (Copy
  dropped, 9 call sites fixed), the edges query head, and the three landed refusal-assertion sweep
  (T2 slice 2, issue #559)`.

### Task 4: `edge-between`

**Files:**
- Modify: `rust/crates/babylon-bsl/src/evaluator.rs`
- Modify: `rust/crates/babylon-bsl/src/query.rs` (widen `enum_member` from private to
  `pub(crate)` — no other change; reused, not re-implemented)

**Interfaces:**
- Produces: `eval_edge_between`, wired into `eval_form`'s dispatch.

- [ ] **Step 1: Red.** In `evaluator.rs`'s test module, a fixture with two nodes and one SOLIDARITY
  edge between them; assert `evaluate(&e("(edge-between EdgeType/SOLIDARITY self other)"), &env,
  ...)` returns `Ok(Value::EdgeRef(EdgeKey { source: self_id, target: other_id, edge_type:
  "SOLIDARITY".to_owned() }))`; a second test with NO edge between two nodes asserts
  `EvalCode::NoSuchEdge`/`"E-EVAL-034"`. Expected: FAIL (`eval_edge_between` does not exist;
  `edge-between` still falls into `UNSERVED_EXPRESSION_HEADS`).

- [ ] **Step 2: Implement.**

```rust
/// `(edge-between <enum-ref> <expr> <expr>)` (§2.10). A keyed lookup (§2.10's own "well-defined
/// because the triple is a key" ruling) — never a set, never a silent no-op on absence.
fn eval_edge_between(
    items: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    charge(fuel, cost::ACCESSOR_BASE)?;
    let [_, type_ref, from_expr, to_expr] = items else {
        return Err(EvalError::plain(
            "(edge-between <enum-ref> <expr> <expr>) — unrecognized shape",
        ));
    };
    let edge_type = crate::query::enum_member(type_ref)?;
    let from = match evaluate(from_expr, env, host, fuel)? {
        Value::NodeRef(id) => id,
        other => {
            return Err(EvalError::plain(format!(
                "(edge-between …)'s first node operand must evaluate to a NodeRef, got {other:?}"
            )))
        }
    };
    let to = match evaluate(to_expr, env, host, fuel)? {
        Value::NodeRef(id) => id,
        other => {
            return Err(EvalError::plain(format!(
                "(edge-between …)'s second node operand must evaluate to a NodeRef, got {other:?}"
            )))
        }
    };
    let graph = require_graph(env, "edge-between")?;
    // Full qname (Major 6, matching `edge_attribute`'s node_attribute-mirroring convention,
    // Task 1) — constructed here because `render_member` lives in THIS crate (`crate::vocabulary`)
    // and `eval_edge_between` has only the raw enum member (e.g. "SOLIDARITY"), never a
    // content-authored qname to pass through unmodified the way `field_of_edge` does.
    let strength_qname = format!("{}/strength", crate::vocabulary::render_member(edge_type));
    graph
        .edge_attribute(edge_type, from, to, &strength_qname)
        .map_err(|_| {
            EvalError::coded(
                EvalCode::NoSuchEdge,
                format!(
                    "(edge-between EdgeType/{edge_type} …): no edge from {from:?} to {to:?} \
                     (§2.10) — the accessor never yields an absent reference"
                ),
            )
        })?;
    Ok(Value::EdgeRef(EdgeKey {
        source: from,
        target: to,
        edge_type: edge_type.to_owned(),
    }))
}
```

  Make `query::enum_member` `pub(crate)` (currently private; reused here rather than
  re-implemented — DRY over the exact same `Atom::EnumRef { member, .. } => Ok(member)` match
  `materialize`'s helper already is).

  Wire the dispatch in `eval_form`: `"edge-between" => eval_edge_between(&items[..], env, host,
  fuel),` alongside the existing `"field-of" => eval_field_of(...)` arm. Remove `("edge-between",
  "slice 2")` from `UNSERVED_EXPRESSION_HEADS` — **bump its type annotation from
  `[(&str, &str); 7]` to `[(&str, &str); 6]`** (`the` remains, per the disposition record). **Update
  the array's own doc-comment prose again** (Task 3 Step 6 already updated it once for `edges`'
  removal): drop `edge-between` from the `` `the` (slice 2, the carrier-read lane — `edges`/
  `edge-between` served as of T2, issue #559) `` clause, leaving only `the` named against slice 2.

  **No sweep needed here (verified for the reviewer's own claim): no landed `edge-between`
  evaluation-refusal test exists anywhere in the crate** — unlike `edges` (Task 3 Step 7's three-file
  sweep), `edge-between` was never exercised by a passing "refuses, names slice 2" vector at the
  evaluation level, only by `edges`'s OWN dispatch-table entry sharing one array literal with it —
  removing THAT array entry (above) is the whole fix; no test file elsewhere asserts on
  `edge-between`'s refusal message the way the three `edges` sites did.

- [ ] **Step 3: Run Step 1's tests.** Expected: PASS.

- [ ] **Step 4: The static/runtime fuel agreement (adversarial review nit).**
  `r9_chapters.rs::edge_between_costs_one_plus_its_two_endpoint_operands` already pins the STATIC
  bound-checker cost at `cost("(edge-between EdgeType/SOLIDARITY self other)") == 3`
  (`r9_chapters.rs:370-382` — `ACCESSOR_BASE(1) + self(1) + other(1) = 3`). Add a runtime fuel
  assertion alongside Step 1's tests confirming `eval_edge_between`'s actual `charge()` calls sum to
  the SAME 3 for the identical shape (an initial `fuel = 10`, after evaluating
  `(edge-between EdgeType/SOLIDARITY self other)` against the two-node fixture, `fuel == 7`) — the
  static bound and the runtime meter must agree, per §4.5's own loud-failure-inversion discipline
  (the same discipline `query.rs::query_materialization_charges_the_3_7_query_base` already proves
  for `nodes`/`neighbors`).

- [ ] **Step 5: Mutation.** Swap `from`/`to` in the substrate call
  (`graph.edge_attribute(edge_type, to, from, &strength_qname)`) → the happy-path test flips (a
  directional edge, looked up backwards, is `E-EVAL-034` since only one direction was seeded).
  Restore byte-identical.

- [ ] **Step 6: Six legs + commit** `feat(bsl): eval_edge_between — the §2.10 accessor (T2 slice 2)`.

### Task 5: `field-of` over an `EdgeRef`

**Files:**
- Modify: `rust/crates/babylon-bsl/src/evaluator.rs`

**Interfaces:**
- Produces: `field_of_edge`, `check_edge_referent_type`, wired into `eval_field_of`'s match.

**Design decision (dossier §2, confirmed, not re-litigated — REVISED against adversarial review
Major 6):** written generically over any qname whose owning type is an `EdgeType` — never
hard-coded to `"strength"` alone. Task 1's `edge_attribute` now takes the FULL QNAME (matching
`node_attribute`'s own convention, corrected from this plan's first draft — see Task 1), so
`field_of_edge` passes `qname` straight through UNMODIFIED, exactly mirroring
`field_of_node`'s own `graph.node_attribute(id, qname)` call shape (`evaluator.rs:1320`) — no
segment-splitting inside `field_of_edge` at all. A T3-era field name resolves through this same
function with zero changes to it — only `GraphSubstrate::edge_attribute`'s body widens.

- [ ] **Step 1: Red.** Extend Task 4's fixture: assert `(field-of (edge-between EdgeType/SOLIDARITY
  self other) solidarity/strength)` evaluates to `Value::Real(0.5)` (the seeded strength); a second
  test with a qname owned by the WRONG type (e.g. `social-class/wealth` against an `EdgeRef`) asserts
  `EvalCode::AccessorTypeOrValueMismatch`/`"E-EVAL-033"`; a third test with a LEGALLY-typed but
  never-written edge qname (a `deffield`-declared `solidarity/tension`, no storage behind it) ALSO
  asserts `E-EVAL-033` — proving Task 1's "not yet stored" branch degrades identically to a
  never-written field, with zero special-casing. Expected: FAIL (`eval_field_of`'s match has no
  `Value::EdgeRef` arm; falls to the generic "edge referents ride slice 2" refusal).

- [ ] **Step 2: Implement.**

```rust
/// The `EdgeRef` half of §2.10 discipline 1. Cheaper than `check_node_referent_type`: an `EdgeKey`
/// carries its type inline (§2.6's own key), so no substrate round-trip is needed to learn it —
/// unlike a `NodeRef`, whose type needs `graph.node_type_of`.
fn check_edge_referent_type(key: &EdgeKey, qname: &str, form: &str) -> Result<(), EvalError> {
    let owner_segment = qname.split('/').next().unwrap_or(qname);
    let expected_type = crate::tick::namespace_to_node_type(owner_segment);
    if key.edge_type != expected_type {
        return Err(EvalError::coded(
            EvalCode::AccessorTypeOrValueMismatch,
            format!(
                "{form} {qname}: the referent is a {} edge, not {expected_type} — the qname's \
                 owning type does not match the referent's declared type (§2.10 discipline 1)",
                key.edge_type
            ),
        ));
    }
    Ok(())
}

/// The `EdgeRef` half of `field-of`'s shared discipline (§2.10) — generic over any qname whose
/// owning type is an EdgeType (T2 dossier §2 design: not hard-coded to `strength` alone). `qname`
/// is passed to the substrate UNMODIFIED, exactly as `field_of_node` passes it to `node_attribute`
/// — the FULL qname is the key convention both share (Task 1's Major-6 correction); a T3-era edge
/// field resolves through this SAME function unmodified — only `GraphSubstrate::edge_attribute`'s
/// body widens.
///
/// **`edge_attribute`'s "no such edge" failure mode (E-EVAL-034's own condition) is UNREACHABLE
/// here.** Every `Value::EdgeRef` this function's caller (`eval_field_of`) can ever hand it was
/// already validated to reference a LIVE edge before construction: `materialize_edges` builds one
/// only from `graph.edges(edge_type)`'s own output (which by construction never names a dangling
/// triple), and `eval_edge_between` only returns a `Value::EdgeRef` AFTER its own existence check
/// already succeeded (erroring `E-EVAL-034` itself, before ever constructing one, otherwise). So the
/// ONLY way `graph.edge_attribute(...)` can fail from inside this function is the "qname is not
/// `<edge-type>/strength`" branch — the mapping below is therefore always `E-EVAL-033`
/// ("declared/legal but never written") in practice, never a laundered `E-EVAL-034`.
fn field_of_edge(key: &EdgeKey, qname: &str, env: &EvalEnv<'_>) -> Result<Value, EvalError> {
    let graph = require_graph(env, "field-of")?;
    check_edge_referent_type(key, qname, "field-of")?;
    let value = graph
        .edge_attribute(&key.edge_type, key.source, key.target, qname)
        .map_err(|e| {
            EvalError::coded(
                EvalCode::AccessorTypeOrValueMismatch,
                format!(
                    "field-of {qname}: {} (§2.10 discipline 2 — absence is not a value)",
                    e.message
                ),
            )
        })?;
    let (Some(types), Some(enums)) = (env.types, env.enums) else {
        return Err(EvalError::plain(format!(
            "(field-of …) needs the declared field-type registry (§2.13) but this EvalEnv \
             carries none for {qname} — a driver error, the same shape as require_graph's"
        )));
    };
    crate::tick::bind_field_value(qname, value, types, enums)
        .map_err(|e| EvalError::plain(e.to_string()))
}
```

  Wire `eval_field_of`'s match:

```rust
    match referent {
        Value::NodeRef(id) => field_of_node(id, qname, env),
        Value::EdgeRef(key) => field_of_edge(&key, qname, env),
        Value::HyperedgeRef(_) => Err(EvalError::plain(
            "(field-of …) over a HyperedgeRef is not meaningful — a hyperedge carries no \
             attributes of its own (§2.8); a membership's payload reads through \
             membership-field-of instead (slice 4)",
        )),
        other => Err(EvalError::plain(format!(
            "(field-of …)'s first operand must evaluate to a reference, got {other:?} (§2.10)"
        ))),
    }
```

  (Drop the now-stale "edge referents ride slice 2" wording from the fallback arm and from the
  function's own doc comment above it — both currently name T2's own not-yet-landed gap.)

- [ ] **Step 3: Run Step 1's tests.** Expected: PASS.

- [ ] **Step 4: Mutation.** In `field_of_edge`, change the substrate call's last argument from
  `qname` to `&key.edge_type` (`graph.edge_attribute(&key.edge_type, key.source, key.target,
  &key.edge_type)`) → the happy-path test flips (`edge_attribute` is called with `"SOLIDARITY"`
  instead of `"solidarity/strength"`, which does not end in `"/strength"` and is refused as "never
  written," `E-EVAL-033`, where the test expects `Value::Real(0.5)`). Restore byte-identical.

- [ ] **Step 5: Six legs + commit** `feat(bsl): field_of_edge — field-of over an EdgeRef, generic
  over any edge-owned qname (T2 slice 2)`.

### Task 6a: Widen `load_edge` — unit-interval (`p`/`i`/`c`) strength literals (amendment, 2026-08-12)

**Sequencing:** runs BEFORE Task 6 (lettered, not renumbered, so every existing cross-reference to
Tasks 6/7 stays valid) — Task 6's fixture cannot load without this widening. See the Amendment log
at the top of this plan for the execution-proven impossibility that forced it; the landed Task 2
probe already hit the same wall and fell back to an int strength `1` (`babylon-tick/src/lib.rs:458`,
where this plan's Task 2 text still shows `0.5` — left as written, Task 2 is executed history).

**Files:**
- Modify: `rust/crates/babylon-bsl/src/scenario.rs` (`load_edge`'s strength match, `:1325-1336`;
  its fn doc comment, `:1282-1284`; the destructure-refusal shape string, `:1297`; three new tests
  in the same file's own `#[cfg(test)] mod tests` — the ONLY file this task touches)

**Interfaces:**
- Produces: `load_edge` accepting, in the strength position, exactly (a) integer literals — the
  existing arm, byte-for-byte unchanged — and (b) `p`/`i`/`c`-suffixed unit-interval scaled
  literals, KIND-BLIND among the three (adjudicated in the 2026-08-12 fix round — reviewer
  MAJOR-1, option (i)), converted `unscaled / 10^scale`. `r`-suffixed Ratio and `$`-suffixed
  Currency literals stay refused, loudly — exactly as the runtime `:strength` position refuses
  their evaluated forms.

**Design decisions, made explicitly per the module's own standing conventions:**

1. **Scope: int + kind-blind `p`/`i`/`c` — ADJUDICATED (fix round 2026-08-12, reviewer MAJOR-1,
   option (i)).** This task's first draft accepted `c` alone, reasoning from D32's kind ruling;
   the adversarial review found that justification doubly wrong, and the controller adjudicated
   kind-blind. The governing facts, all read directly: (a) the RUNTIME writer of this exact
   position is kind-blind — `structural_verbs.rs::add_edge` (`:898`) accepts any `Value::Real` at
   `:strength` (`:929-936`), and kinds do not survive evaluation at all (`evaluator.rs:378-384`:
   every `p`/`i`/`c` literal becomes an untagged `Value::Real`), so that function's own `0.5c`
   test fixtures (`structural_verbs.rs:1734,1790,1850,2028`) would behave identically written
   `0.5p`; (b) the node-field seed path made the SAME choice for the SAME reason and RECORDED it —
   `attribute_value_unit_interval` accepts any of the three suffixes for any of the three declared
   unit-interval types precisely because the runtime store check it mirrors is kind-blind
   (`scenario.rs:1171-1177` — it does not defer to the declaration; it takes the declared type and
   deliberately ignores the literal's kind, because `Value::Real` carries no `p`/`i`/`c` tag once
   evaluated); (c) a `c`-only loader would therefore have MINTED a new bidirectional load/runtime
   divergence — `(add-edge … :strength 0.5p)` writes at runtime while `(edge … 0.5p)` would
   refuse at hydration — the exact asymmetry class `attribute_value_unit_interval`'s own doc
   forbids for rounding (`scenario.rs:1199-1203`: a scenario-seeded value and a rule-computed
   write of the identical field must never follow two different rules). So hydration MIRRORS THE
   RUNTIME WRITER, exactly as the node path does. D32 still kinds the field `Coefficient` — that
   fact belongs in the code comment (and Task 6's fixture keeps `c` as the idiomatic spelling) —
   but the refusal message must not imply c-only. What stays out, each also mirroring the runtime:
   `r` (Ratio) — the node path's own recorded exclusion (`scenario.rs:1177-1184`: `Value::Ratio`
   is a genuinely distinct runtime type with its own `(0, ∞)` domain, not the binary64 lane; the
   runtime `:strength` match refuses it identically, since `:930-935` accepts only `Value::Real`)
   — and `$` (Currency) — the `currency_refusal_message` precedent (`scenario.rs:1273-1280`; the
   runtime refuses `Value::Currency` at `:strength` the same way). The refusal stays a
   PLAIN `err(...)` (code `None`), matching the arm it replaces (`:1331-1335`) — the `.bscn` dialect
   is spec-unpinned (D93), so no `E-LOAD` code exists to cite, and `coded_err` is reserved for
   §3.9-coded hydration failures (`E-LOAD-044`'s own comment, `:1338-1341`). For the same D93
   reason this task edits NO `bsl-language.rst` text and mints NO register row — the record homes
   are ADR201 (Task 7 Step 4's amended records list) and this plan's own Amendment log — and the
   Python grammar-sync guard is NOT needed for this task (nothing in `docs/reference/` moves).
2. **Conversion: `unscaled as f64 / 10_f64.powi(i32::from(scale))` — the crate's ONE conversion
   contract, copied verbatim, never re-derived.** There is no named helper to call — verified by
   direct reading, the arithmetic appears inline at EIGHT existing sites (fix-round census,
   MINOR-1): `attribute_value_unit_interval`'s p/i/c arm (`scenario.rs:1240-1242`) AND its
   Ratio-refusal arm's message-building division (`:1228-1230`), `load_defconst` (`:521-523`),
   `ratio_from_scaled` (`:744`), `tick.rs::atom_to_value` (`:392-395`), `rule_pipeline.rs:529`,
   and `evaluator.rs`'s two atom arms (`:363` Ratio, `:382` p/i/c — `:382` IS the division line,
   Read-verified against the fix round's own citation: `:381` is its `#[allow]`, `:383` its
   `Ok(Value::Real(value))`). `attribute_value_unit_interval`'s own doc comment (`:1186-1209`) is the
   contract's normative home: one correctly-rounded IEEE-754 division of two exactly-representable
   operands (`unscaled` as an integer, `10^scale` exact in f64 for `scale <= 9`, the literal's own
   lex bound `E-LEX-023`), deliberately NOT `babylon_kernel::grid::quantize` — read that comment
   before writing the arm; this task adds the NINTH inline site, same three lines, same
   `#[allow(clippy::cast_precision_loss)]`. That is a real DRY tradeoff, owned deliberately:
   extracting a shared helper across nine sites is a refactor with its own blast radius,
   out of this amendment's surgical scope — the post-port refactor program's D-record ledger is
   where that extraction belongs if it ever earns its cost. Every fixture strength in Task 6 (`0.125`, `0.03125`,
   `0.015625`, `0.0625`, `0.1875`, `0.25`) and both fold expecteds (`0.796875`, `0.375`) are dyadic
   rationals exactly representable in binary64, so the correctly-rounded division returns each one
   BIT-EXACTLY (a correctly-rounded op whose true result is representable returns it exactly —
   e.g. `125/1000` is exactly `0.125 = 2^-3`); no tolerance appears anywhere in this lane.
3. **Range law: MIRRORED, meaning NO new check.** Today's int path range-checks NOTHING — verified
   by direct reading: `scenario.rs:1325-1330` widens any `i64` with a bare `as f64` (so
   `(edge … 5)` loads as `5.0`, `(edge … -1)` as `-1.0`; unlike `attribute_value_int`'s own
   `2^53` exactness guard at `:1150-1153`, the strength arm does not even bound magnitude), and
   neither backend's `add_edge` checks either (`memory.rs:147-167`, `hypergraph_store.rs:194-215`
   — both insert the f64 as given). Mirroring that law, the new scaled arm mints NO range check in
   `load_edge` — and needs none: the LEXER already bounds every `p`/`i`/`c` literal to `[0,1]`
   (`E-LEX-024`: the constructor closure `out_of_range`, `reader.rs:877-883`; fire sites `:887`,
   `:889`, `:893` — Read-verified against the fix round's own citation, which was off by
   one), so a `p`/`i`/`c`-suffixed strength arrives
   in-domain by construction. The resulting asymmetry — an int strength of `5` loads today while
   `5c` cannot even lex — PRE-EXISTS in the lexer (bare ints carry no lex-time domain, the same
   fact `attribute_value_unit_interval`'s Int arm documents at `:1217-1222`, where the node path's
   load-time `[0,1]` check catches it — that check, `:1252-1259`, is the load-bearing tail of the
   fn's full `:1210-1261` span (MINOR-5); the strength position has no such check) and is recorded
   here as an OPEN QUESTION for a future ruling, deliberately not resolved by this task: D32 rules
   the field Coefficient-kinded (`[0,1]`) yet out-of-`[0,1]` int strengths load — and §3.9
   clause 1 (`bsl-language.rst:3042-3045`, hydration "creates elements of declared types and
   writes declared fields, and nothing else") is the spec hook that makes this a real OWED
   ruling rather than a mere curiosity: an out-of-domain write into a Coefficient-ruled field
   sits poorly against that clause. Tightening the
   int arm could refuse previously-loadable content and is a separate, content-auditable ruling,
   not a side effect to smuggle into a literal-kind widening. One MORE divergence remains under
   the kind-blind widening, pre-existing and recorded beside this same open question rather than
   resolved: an int strength LOADS at hydration while the runtime `:strength` position refuses
   `Value::Int` outright (`structural_verbs.rs:929-936` accepts only `Value::Real`) — whatever
   future ruling settles the int arm's domain owns both halves of that divergence too.
4. **Test placement: `scenario.rs`'s own `#[cfg(test)] mod tests`, where every existing `load_edge`
   test lives** (`an_edge_to_an_undeclared_name_is_loud` `:1450`,
   `load_edge_demands_edgetype_regardless_of_what_is_declared` `:2744`,
   `an_edge_under_an_undeclared_edge_type_is_inert_not_e_load_031` `:2802`; the bit-exact
   scaled-literal genre this task's green test follows is
   `a_scaled_literal_seeds_correctly_into_each_unit_interval_type` `:1528`) — NOT `r9_chapters.rs`
   (which holds only the `E-LOAD-044` chapter vector, `r9_chapters.rs:436-447`). Conventions
   matched: `load_scenario(source, &mut graph)` + `unwrap_err()` + `err.message.contains(…)` for
   refusals; `to_bits()` pins, never tolerances, for seeded values. The module's existing imports
   already cover everything needed (`load_scenario` `:1359`, `MemoryGraph` `:1367`,
   `GraphSubstrate`/`NodeId` `:1369`); `ScaledKind` is already imported by the non-test code
   (`:61`). Note the loader-level test IS the real seam's own function: the driver funnel —
   `run_once` (`babylon-tick/src/lib.rs:72`) → `run_once_into` (`:330`) → `prepare_rules`
   (called at `:336`) — reaches this same `load_scenario` (`:126`), and Task 6's
   e2e vectors then re-prove the widening end-to-end through `run_once_into` itself.

- [ ] **Step 1: Red — the widening test.** In `scenario.rs`'s test module, after
  `a_scaled_literal_seeds_correctly_into_each_unit_interval_type` (its genre neighbor):

```rust
    #[test]
    fn a_coefficient_strength_literal_seeds_bit_exactly() {
        // Task 6a (T2 plan amendment, 2026-08-12): D32 rules
        // <edge-type>/strength Coefficient-kinded — a c-suffixed literal is
        // hydration's idiomatic way (D32 kinds the field Coefficient; p/i
        // seed identically — see the companion test) to seed a fractional
        // strength (a bare decimal
        // is E-LEX-021, and pre-6a this loader refused every non-int
        // strength). Bit-exact pin, NOT a tolerance — the same conversion
        // contract `attribute_value_unit_interval`'s doc comment states;
        // 0.125 = 2^-3 is dyadic-exact, so 125/1000 divides to it exactly.
        let source = r"
(scenario ft/coeff-strength
  (node core NodeType/SOCIAL_CLASS)
  (node periphery NodeType/SOCIAL_CLASS)
  (edge EdgeType/SOLIDARITY core periphery 0.125c))
";
        let mut graph = MemoryGraph::new();
        load_scenario(source, &mut graph).unwrap();
        let strength = graph
            .edge_attribute("SOLIDARITY", NodeId(0), NodeId(1), "solidarity/strength")
            .unwrap();
        assert_eq!(
            strength.to_bits(),
            (0.125_f64).to_bits(),
            "a c-suffixed Coefficient strength must seed bit-exactly, got {strength}"
        );
    }
```

  (`NodeId(0)`/`NodeId(1)`: declaration order fixes id assignment — the same contract Task 6's
  fixture comment already cites, and the same direct-`NodeId` convention the Half-1 bit-pin test
  uses at `:1562`. The read-back goes through Task 1's landed `edge_attribute` — full qname, owner
  segment deliberately unchecked.)

- [ ] **Step 2: Run it RED.** Expected exact failure: the `unwrap()` panics on
  `` Err(ScenarioError { message: "edge SOLIDARITY: expected an integer strength literal, found
  Scaled(ScaledLit { kind: Coefficient, unscaled: 125, scale: 3 })", code: None }) `` — the same
  refusal already execution-proven through the real driver (`run_once`, `lib.rs:72`, funneling
  through `run_once_into` `:330` → `prepare_rules` `:336` → this same `load_scenario`, `:126`),
  byte for byte. `0.125c` canonicalizes to `unscaled: 125, scale: 3` (§1.5
  minimal-scale canonicalization, `reader.rs:134-148`).

- [ ] **Step 3: The minimal widening.** Replace the strength match (`scenario.rs:1323-1336`,
  comment included):

```rust
    // An edge strength is not a node field, so no deffield governs it; the
    // literal restriction is stated directly here: an integer, or a
    // p/i/c-suffixed unit-interval literal — KIND-BLIND among the three,
    // mirroring both attribute_value_unit_interval (kinds do not survive
    // evaluation; its doc records the choice) and the runtime writer of
    // this exact position (structural_verbs.rs::add_edge accepts any
    // Value::Real at :strength). D32 kinds the field Coefficient; the
    // loader does not narrow to c-only because the runtime cannot (T2 plan
    // Task 6a, amendment 2026-08-12, adjudicated in its fix round). Ratio
    // (r) stays out for the node path's own recorded reason — Value::Ratio
    // is not the binary64 lane, and the runtime :strength match refuses it
    // identically. No range check on either arm — ints were never checked
    // here (an OPEN asymmetry, recorded in the T2 plan's Task 6a design
    // decision 3, not silently resolved), and a p/i/c literal is already
    // [0,1]-bounded at lex (E-LEX-024).
    let strength = match strength {
        Atom::Int(value) => {
            #[allow(clippy::cast_precision_loss)]
            let widened = *value as f64;
            widened
        }
        Atom::Scaled(scaled)
            if matches!(
                scaled.kind,
                ScaledKind::Probability | ScaledKind::Intensity | ScaledKind::Coefficient
            ) =>
        {
            // `unscaled / 10^scale` — the crate's one scaled-literal
            // conversion contract (attribute_value_unit_interval's doc
            // comment is its normative home), copied verbatim.
            #[allow(clippy::cast_precision_loss)]
            let numerator = scaled.unscaled as f64;
            numerator / 10_f64.powi(i32::from(scaled.scale))
        }
        other => {
            return Err(err(format!(
                "edge {member}: expected an integer or p/i/c-suffixed \
                 unit-interval strength literal, found {other:?}"
            )))
        }
    };
```

  (The three `ScaledKind` variant names are Read-verified against `reader.rs:123-132`:
  `Probability`/`Intensity`/`Coefficient`, plus `Ratio` which the guard deliberately omits.)

  Also update the two neighboring strings that still say `<int>`:
  - the fn doc comment (`:1282`): `` `(edge <enum-ref> <local-name> <local-name> <int>)` `` becomes
    `` `(edge <enum-ref> <local-name> <local-name> <strength>)` — `<strength>` is an int or a
    `p`/`i`/`c`-suffixed unit-interval literal (kind-blind, mirroring the runtime `:strength`
    position; D32 kinds the field Coefficient; T2 plan Task 6a) `` (the rest of the doc
    sentence — the return-convention half — is unchanged);
  - the destructure-refusal shape string (`:1297`):
    `"expected (edge <EdgeType/MEMBER> <from-name> <to-name> <int>)"` becomes
    `"expected (edge <EdgeType/MEMBER> <from-name> <to-name> <strength: int | p/i/c-lit>)"`.

- [ ] **Step 4: GREEN.** Run Step 1's test — passes, the exact loaded f64 asserted bit-for-bit via
  `edge_attribute`. Re-run the whole `scenario.rs` test module: every existing test stays green
  (the int arm is byte-identical; nothing else in `load_edge` moved).

- [ ] **Step 5: The kind sweep — the kind-blind half pinned GREEN, and every kind that stays out
  pinned refused.** Two tests (the first is the fix round's own addition: without it, a
  regression narrowing the guard back to c-only — the adjudication's REJECTED option — would
  flip nothing):

```rust
    #[test]
    fn a_probability_or_intensity_strength_literal_seeds_identically() {
        // The kind-blind half of Task 6a's ruling (fix round, MAJOR-1
        // option (i)), pinned GREEN: kinds do not survive evaluation
        // (evaluator.rs's atom arm makes every p/i/c literal an untagged
        // Value::Real), so hydration accepts p/i exactly as it accepts c —
        // mirroring the runtime writer (structural_verbs.rs::add_edge, any
        // Value::Real at :strength).
        for literal in ["0.125p", "0.125i"] {
            let source = format!(
                "(scenario ft/kind-blind-strength\n  \
                 (node core NodeType/SOCIAL_CLASS)\n  \
                 (node periphery NodeType/SOCIAL_CLASS)\n  \
                 (edge EdgeType/SOLIDARITY core periphery {literal}))"
            );
            let mut graph = MemoryGraph::new();
            load_scenario(&source, &mut graph).unwrap();
            let strength = graph
                .edge_attribute("SOLIDARITY", NodeId(0), NodeId(1), "solidarity/strength")
                .unwrap();
            assert_eq!(strength.to_bits(), (0.125_f64).to_bits(), "{literal}");
        }
    }

    #[test]
    fn a_ratio_or_currency_strength_literal_stays_refused() {
        // What stays out under the kind-blind widening, each mirroring the
        // runtime :strength match (which accepts only Value::Real): r
        // evaluates to Value::Ratio, $ to Value::Currency. r is a ScaledLit
        // kind the guard refuses; $ (Currency) is a DIFFERENT Atom variant
        // that falls to the same refusal arm without ever reaching the kind
        // guard (see the Step-6 mutation split in the T2 plan).
        for literal in ["0.5r", "1$"] {
            let source = format!(
                "(scenario ft/wrong-kind-strength\n  \
                 (node core NodeType/SOCIAL_CLASS)\n  \
                 (node periphery NodeType/SOCIAL_CLASS)\n  \
                 (edge EdgeType/SOLIDARITY core periphery {literal}))"
            );
            let mut graph = MemoryGraph::new();
            let err = load_scenario(&source, &mut graph).unwrap_err();
            assert!(
                err.message.contains(
                    "expected an integer or p/i/c-suffixed unit-interval strength literal"
                ),
                "{literal}: {}",
                err.message
            );
        }
    }
```

  The `$` row's exact full message is `` edge SOLIDARITY: expected an integer or p/i/c-suffixed
  unit-interval strength literal, found Currency(Currency(1000000)) `` (`Atom::Currency` wraps
  `babylon_kernel::Currency(i128)` in
  micro-units — `1$` = 1,000,000 micro-units; both derive `Debug`, `currency.rs:35-36`); the
  `0.5r` row's Debug tail is `` Scaled(ScaledLit { kind: Ratio, unscaled: 5, scale: 1 }) `` (§1.5
  minimal-scale canonicalization: `0.5` → `5 / 10^1`). The
  assertion pins the stable head via `.contains`, matching every sibling refusal test in this
  module — no existing test pins the `{other:?}` Debug tail today either (verified: nothing
  asserts on the current "found Scaled(…)" tail).

- [ ] **Step 6: Mutation evidence — the guard, proven in BOTH directions, split honestly.**
  - **Mutation A (widen):** append `| ScaledKind::Ratio` to the guard's `matches!` list → the
    `0.5r` row of `a_ratio_or_currency_strength_literal_stays_refused` flips RED (`0.5r` now
    converts `5 / 10^1` and loads, so its `unwrap_err` panics) while the `1$` row stays green
    (`Atom::Currency` is a different VARIANT; no mutation of the kind guard alone can accept a
    `$` literal, which is why the sweep pins it separately), and Step 1's test plus the p/i
    companion stay green too. Restore byte-identical.
  - **Mutation B (narrow — the adjudication's rejected option):** shrink the guard to
    `scaled.kind == ScaledKind::Coefficient` → both rows of
    `a_probability_or_intensity_strength_literal_seeds_identically` flip RED (each `unwrap()`
    panics on the refusal) while Step 1's `0.125c` test and both refusal rows stay green — the
    kind-blind ruling is mutation-provable in both directions, not just at the Ratio boundary.
    Restore byte-identical.

- [ ] **Step 7: Goldens + the six-leg gate.** All seven pins (six `tick_goldens.rs` +
  `engine_link.rs`) must stay byte-identical — structurally guaranteed here, twice over: the int
  arm is byte-identical, and no shipped scenario CAN contain a scaled strength literal (one could
  not have LOADED before this task — every committed `.bscn` seeds int strengths, verified:
  `rg -n "\(edge " rust/crates/babylon-tick/content/scenarios/*.bscn` shows integer literals
  only). Run the full six-leg gate anyway — the gate is the proof, not the assumption.

- [ ] **Step 8: Commit** `feat(bsl): load_edge accepts p/i/c unit-interval strength literals
  (runtime-mirroring kind-blind widening, T2 Task 6a)`.

### Task 6: The e2e vectors

**Files:**
- Create: `rust/crates/babylon-tick/content/scenarios/edge-lane-e2e.bscn`
- Create: `rust/crates/babylon-tick/tests/edge_lane_e2e.rs`

**Interfaces:**
- Produces: the query_lane_e2e.rs-genre proof that all three heads evaluate correctly through the
  real `run_once_into` seam, over a hand-built fixture — anticipatory of Solidarity's own read shape,
  shipping no Solidarity content (Solidarity's PORT is a separate Wave C train).

- [ ] **Step 1: Write the fixture.** **Every strength HALVED from this plan's first draft
  (adversarial review Blocker 2, verified): `social-class/fold-total` is declared `coefficient`,
  whose store boundary refuses any value outside `[0,1]` — `E-EVAL-020`
  (`structural_verbs.rs::store_range_check`, `:1319-1341`) — and a tick that hits it ABORTS.** The
  first draft's Shape 1 total (`1.59375`) exceeded 1.0 and would have aborted every time; the
  genre precedent (`query-lane-e2e.bscn`) deliberately keeps every value under the cap for the same
  reason. Every value below is still an exact dyadic rational (a fraction with a power-of-two
  denominator — 0.1875 = 3/16 and 0.375 = 3/8 are not themselves powers of two, but both terminate
  exactly in binary64, which is the property that matters here; no bit-pinning needed), and
  every FOLD SUM below is now re-verified to land inside `[0,1]`. **Every strength literal is
  `c`-suffixed (Task 6a, amendment 2026-08-12):** a bare decimal cannot LEX at all (`E-LEX-021`),
  and post-6a `load_edge` accepts int + `p`/`i`/`c`-suffixed unit-interval strength literals
  (kind-blind, mirroring the runtime `:strength` writer — Task 6a's fix-round adjudication; `c`
  stays this fixture's idiomatic spelling, since D32 kinds the field Coefficient) — the
  suffix changes NO value (each literal converts bit-exactly, Task 6a design decision 2), and
  `E-LEX-024` additionally bounds each literal to `[0,1]` at lex, one gate upstream of the
  `E-EVAL-020` store-domain argument above:

```scheme
; The T2 (Program 29, issue #559) edge-lane e2e fixture — dyadic edge reads (edges/edge-between/
; field-of over an EdgeRef). Anticipatory of Solidarity's own read shape (bsl-language.rst §3.8
; item 8's worked example) but ships NO Solidarity content — Solidarity's own PORT is a separate
; Wave C train (ADR198 consequences).
;
; Node groups, in declaration order (fixes id assignment, scenario.rs's own contract):
;   0-1   fold-a/fold-b        the edges-fold graph, first pair (0.125)
;   2     fold-reporter        Shape 1 subject — sums EVERY SOLIDARITY edge in the whole graph
;   3-4   pair-x/pair-y        Shape 2a — edge-between resolves (0.03125, pair-x -> pair-y)
;   5     pair-z                Shape 2b — edge-between fails on the REVERSED direction
;   6-9   hub/spoke-1/spoke-2/spoke-3   Shape 3 — self-anchored neighbors+edge-between
;   10-11 fold-c/fold-d        the edges-fold graph, second pair (0.25)
;
; social-class/fold-total is COEFFICIENT-typed (D32's own kind for strength) — every strength
; below, and every fold SUM over them, is chosen to stay inside [0,1] (E-EVAL-020's own domain);
; see this step's own note for the arithmetic that was corrected.
;
; `social-class/shape` discriminates which rule fires on which node (the territory/shape
; convention, query_lane_e2e.rs). Only the SUBJECT of each shape carries a nonzero shape value;
; every other node is shape 0 (inert — exists only as an edge endpoint).
(scenario social-class/edge-lane-e2e
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defvocabulary EdgeType (SOLIDARITY))
  (deffield social-class/shape int extensive)
  (deffield social-class/fold-total coefficient extensive)
  ; solidarity/strength is IMPLICIT (D32) — deliberately NOT deffield'd here; this fixture is the
  ; proof that Task 2's wiring seeds it without one.

  (node fold-a NodeType/SOCIAL_CLASS (social-class/shape 0) (social-class/fold-total 0))
  (node fold-b NodeType/SOCIAL_CLASS (social-class/shape 0) (social-class/fold-total 0))
  (node fold-reporter NodeType/SOCIAL_CLASS (social-class/shape 1) (social-class/fold-total 0))

  (node pair-x NodeType/SOCIAL_CLASS (social-class/shape 2) (social-class/fold-total 0))
  (node pair-y NodeType/SOCIAL_CLASS (social-class/shape 0) (social-class/fold-total 0))
  (node pair-z NodeType/SOCIAL_CLASS (social-class/shape 4) (social-class/fold-total 0))

  (node hub NodeType/SOCIAL_CLASS (social-class/shape 3) (social-class/fold-total 0))
  (node spoke-1 NodeType/SOCIAL_CLASS (social-class/shape 0) (social-class/fold-total 0))
  (node spoke-2 NodeType/SOCIAL_CLASS (social-class/shape 0) (social-class/fold-total 0))
  (node spoke-3 NodeType/SOCIAL_CLASS (social-class/shape 0) (social-class/fold-total 0))

  (node fold-c NodeType/SOCIAL_CLASS (social-class/shape 0) (social-class/fold-total 0))
  (node fold-d NodeType/SOCIAL_CLASS (social-class/shape 0) (social-class/fold-total 0))

  (edge EdgeType/SOLIDARITY fold-a fold-b 0.125c)
  (edge EdgeType/SOLIDARITY pair-x pair-y 0.03125c)
  (edge EdgeType/SOLIDARITY pair-z pair-x 0.015625c)
  (edge EdgeType/SOLIDARITY spoke-1 hub 0.0625c)
  (edge EdgeType/SOLIDARITY spoke-2 hub 0.125c)
  (edge EdgeType/SOLIDARITY spoke-3 hub 0.1875c)
  (edge EdgeType/SOLIDARITY fold-c fold-d 0.25c))
```

  `pair-z`'s edge points AT `pair-x` (not the reverse) — deliberately, so Shape 2b's reversed lookup
  fails.

- [ ] **Step 2: Shape 1 — `edges`-fold (graph-scope, no `the` needed).**

```scheme
(rule social-class/edges-fold-e2e
  :material-basis "T2's edges query head materializing every dyadic SOLIDARITY edge in the graph and folding its implicit strength (D32) — proves (edges <enum-ref>) evaluates for real and Task 2's D32 wiring resolves <edge-type>/strength through an UNWEIGHTED aggregation (typecheck.rs::resolve_field), not merely the bare accessor's graceful fallback"
  :fuel 512
  (bindings (binding shape :field social-class/shape))
  (when (= shape 1))
  (effects
    (update-node self social-class/fold-total
      (set (fold sum (edges EdgeType/SOLIDARITY) (field-of it solidarity/strength))))))
```

  Test `shape_1_edges_fold_sums_every_solidarity_edge_in_the_graph`: run through `run_once_into`;
  `report.fired == 1`; `fold-reporter`'s `fold-total` reads back exactly
  `0.125 + 0.03125 + 0.015625 + 0.0625 + 0.125 + 0.1875 + 0.25 = 0.796875` (re-derived and
  re-verified in [0,1] against `E-EVAL-020`'s own domain — derive in the test's own comment,
  matching `query_lane_e2e.rs`'s provenance discipline).

- [ ] **Step 3: Shape 2 — `edge-between` resolves and fails.**

```scheme
(rule social-class/edge-between-resolves-e2e
  :material-basis "edge-between resolving successfully and its field-of read agreeing with the strength seeded at the edge's own declaration — the R9 chapter-C2 required-vector family, turned into a real evaluation-level vector"
  :fuel 256
  (bindings (binding shape :field social-class/shape))
  (when (= shape 2))
  (effects
    (update-node self social-class/fold-total
      (set (field-of (edge-between EdgeType/SOLIDARITY self
                        (select-max (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS) 1))
                      solidarity/strength)))))

(rule social-class/edge-between-missing-is-e-eval-034-e2e
  :material-basis "edge-between is directional (§2.10's key is the ORDERED triple) — looking up the reverse of a seeded edge finds nothing and must fail loudly through the real driver, never a silent absent reference"
  :fuel 256
  (bindings (binding shape :field social-class/shape))
  (when (= shape 4))
  (effects
    (update-node self social-class/fold-total
      (set (field-of (edge-between EdgeType/SOLIDARITY
                        (select-max (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS) 1)
                        self)
                      solidarity/strength)))))
```

  **Determinism caveat, stated explicitly (adversarial review nit):** both rules' `select-max`
  scores by the CONSTANT `1` — this is deterministic ONLY because `pair-x`'s and `pair-z`'s own
  `:out` `SOLIDARITY` neighbor sets are SINGLETONS (one candidate each) — a `select-max` over a
  constant score genuinely exercises D46's ascending-id tiebreak only when TWO OR MORE candidates
  tie, which this fixture does not attempt (Territory's own worker-pp-two-lands vector is the
  precedent for that shape; T2's own e2e vectors are not it). If a future edit of this fixture ever
  gives `pair-x` a second outgoing `SOLIDARITY` edge, this rule's constant score must become a real
  field (or the tiebreak must be deliberately exercised and asserted), not left as an accidental
  single-candidate selection.

  Two tests: `shape_2a_edge_between_resolves_and_reads_strength` — `pair-x`'s `fold-total` reads
  back `0.03125` exactly; `shape_2b_edge_between_on_a_missing_pair_is_e_eval_034` —
  `run_once_into(...)` returns `Err` whose message contains `"E-EVAL-034"` (the whole tick aborts,
  matching `query_lane_e2e.rs`'s own documented "TICK ABORT" semantics for an unguarded evaluation
  error — verify this against the real driver's actual error propagation as this step's own
  red-then-green pass, since no existing e2e vector currently exercises a genuine mid-tick abort
  this way). **This second test does NOT go through Step 5's `TickReport`-comparison loop** — see
  Step 5's own split.

- [ ] **Step 4: Shape 3 — the self-anchored `neighbors`+`edge-between` idiom (§3.8 item 8's worked
  example, Solidarity-anticipating).**

```scheme
(rule social-class/self-anchored-solidarity-fold-e2e
  :material-basis "the §3.8 item 8 worked example — Solidarity's own anticipated read shape, self-anchored via neighbors+edge-between rather than iterating (edges ...) and needing an endpoint accessor the language does not have (§3.8 item 8's own open item, dossier §8). T2 proves this idiom evaluates for real, and this is the vector that unblocks Solidarity's own port train without needing source-of/target-of."
  :fuel 512
  (bindings (binding shape :field social-class/shape))
  (when (= shape 3))
  (effects
    (update-node self social-class/fold-total
      (set (fold sum (neighbors self EdgeType/SOLIDARITY :in NodeType/SOCIAL_CLASS)
                 (field-of (edge-between EdgeType/SOLIDARITY it self) solidarity/strength))))))
```

  Test `shape_3_self_anchored_neighbors_and_edge_between_sums_incoming_strength`: `hub`'s
  `fold-total` reads back `0.0625 + 0.125 + 0.1875 = 0.375` exactly (re-derived and re-verified in
  [0,1]).

- [ ] **Step 5: Determinism — split, because Shape 2b is DESIGNED to `Err` (adversarial review
  Blocker 3).** `query_lane_e2e.rs::every_shape_is_deterministic_across_two_independent_runs`'s own
  loop `.unwrap_or_else(|e| panic!(...))`s every `run_once_into` result — reused unmodified, that
  loop would panic on Shape 2b's own vector, whose entire point is to `Err`. Split into two legs:

  (a) The three SUCCEEDING shapes (edges-fold, edge-between-resolves, self-anchored-fold) through
  the unmodified `query_lane_e2e.rs`-style `TickReport`-comparison loop: run each twice,
  independently-loaded graphs, assert `TickReport` byte-identical (`before`/`after`/`fired`/
  `per_rule_fired`) both times.

  (b) Shape 2b gets its OWN two-run leg:

```rust
    #[test]
    fn shape_2b_error_is_deterministic_across_two_independent_runs() {
        let mut graph_a = HypergraphStore::new();
        let mut sink_a = CollectingSink::default();
        let err_a = run_once_into(SCENARIO, RULE_EDGE_BETWEEN_MISSING, &mut graph_a, &mut sink_a)
            .expect_err("shape 2b must abort the tick");

        let mut graph_b = HypergraphStore::new();
        let mut sink_b = CollectingSink::default();
        let err_b = run_once_into(SCENARIO, RULE_EDGE_BETWEEN_MISSING, &mut graph_b, &mut sink_b)
            .expect_err("shape 2b must abort the tick");

        assert_eq!(err_a, err_b, "the same content must fail the same way both times");
        assert!(err_a.contains("E-EVAL-034"), "{err_a}");
    }
```

- [ ] **Step 6: Six legs + commit** `test(tick): edge_lane_e2e — the three T2 slice-2 vectors
  through the real run_once_into seam (issue #559)`.

### Task 7: Hash-free proof, D-rows, ADR, and the `the` disposition record

**Files:**
- Modify: `docs/reference/bsl-language.rst` (register rows — next free is D139, RE-CHECK at PR time
  per the D105 discipline)
- Create: `ai/decisions/ADR201_t2_edge_reads_handoff.yaml` (next free ADR number, verified against
  `ai/decisions/index.yaml` — currently ADR200) + index.yaml row
- Modify: GitHub issue #559 (close, with evidence) and #572 (resolve the disposition rider, per
  below — comment and close, or leave open per the Director's own call if the disposition needs
  ratification)

- [ ] **Step 1: The hash-free proof (not an assertion) — the real argument, stated in full.**
  Two separate facts, chained (adversarial review nit — the first draft's "T2 ships no new content
  pack" framing was incomplete: Task 2's D32 wiring DOES touch three shipped scenarios' in-memory
  `TypeEnv.fields`, even though it touches zero files on disk):
  1. **`state_hash.rs`'s hashing code never touches `Value`/`Element` (babylon-bsl types) BY
     CONSTRUCTION**, not merely in practice: `write_edges` (`state_hash.rs:150-171`, section `0x03`)
     operates purely on `(String, NodeId, NodeId, f64)` tuples, and `babylon-graph/Cargo.toml`'s
     `[dependencies]` names only `babylon-kernel`/`hypergraph-rs` — `babylon-graph` cannot import a
     `babylon-bsl` type even if it wanted to (the crate boundary makes it impossible, not merely
     unexercised). Task 1's substrate method is therefore hash-inert by the SAME argument every
     other `GraphSubstrate` read method already is.
  2. **Task 2's D32 wiring is real, and it DOES touch three shipped scenarios' `TypeEnv.fields`**:
     `territory-conformance.bscn:95`, `production-conformance.bscn:108`, and
     `organization-foundation.bscn:42` each declare `(defvocabulary EdgeType ...)`, so Task 2's
     seeding loop adds new (unwritten, unread-by-anything-yet) entries to those three scenarios'
     in-memory field registries at LOAD time. **This is inert in practice for the reason that
     matters, scoped correctly this time: zero content in `content/rules/*.bsl` — the seven
     GOLDEN-backing packs this hash-free proof is actually about — reads an edge-owned qname or
     uses `edges`/`edge-between` at all.** Verified directly, scoped to that one directory (the
     first draft's claim was WRONG when read as covering the whole crate — corrected):
     `rg -n "strength" rust/crates/babylon-tick/content/rules/*.bsl` and a check for `(edges
     ...)`/`(edge-between ...)` use, BOTH scoped to `content/rules/*.bsl` only, return zero real
     hits (the only substring matches are `"strengthening"`/`"strengthens"` inside unrelated
     comments) — so the new `TypeEnv.fields` entries are never consulted by any LOAD or EVALUATION
     path any of the SEVEN PINNED GOLDENS exercises, and the hash-free proof survives exactly as
     stated. **This does NOT extend to every `.bsl` fixture in the crate — it does not need to**:
     `rust/crates/babylon-bsl/tests/conformance/event_edge_count.bsl` (a TEST-ONLY conformance
     fixture, not a golden-backing content pack) DOES use `(fold count (edges EdgeType/SOLIDARITY)
     it)`, and serving `edges` genuinely changes ITS behavior (three landed assertions flip from
     refusing to evaluating) — that is Task 3 Step 7's own sweep, not a hash-free-proof concern
     (nothing in `tests/conformance/` or `tests/conformance_corpus.rs` participates in any of the
     seven pinned tick-hash goldens). THIS zero-content-hit grep over `content/rules/*.bsl`
     specifically, not "T2 touches no content anywhere," is the real inertness proof for the
     GOLDENS — state it as such in the ADR (Step 4), alongside a pointer to Task 3 Step 7 for the
     one fixture that DOES change.
  3. **Task 6a's `load_edge` widening (amendment, 2026-08-12) is golden-inert by construction:**
     the int arm is byte-identical, and no shipped scenario CAN contain a scaled strength literal
     — one could not have LOADED before 6a (the pre-6a int-only refusal is execution-proven; the
     landed Task 2 probe's own fallback to an int strength `1`, `babylon-tick/src/lib.rs:458`, is
     its in-tree trace) — so the new unit-interval arm is exercised by nothing but Task 6's own
     new fixture, which backs no golden.

  Run, unmodified: all six `tick_goldens.rs` pins (`two_classes_fundamental_theorem_hashes_are_pinned`,
  `vitality_conformance_hashes_are_pinned`, `us_counties_lifecycle_demo_hashes_are_pinned`,
  `organization_foundation_hashes_are_pinned`, `territory_conformance_hashes_are_pinned`,
  `production_conformance_hashes_are_pinned`) plus `babylon-client/tests/engine_link.rs`'s pin —
  all seven must stay byte-identical. Running them is the confirmation; the two facts above are the
  proof of WHY they must.

- [ ] **Step 2: Register rows.**
  - **D139** (the D32 wiring, and its own narrowing ruling): Task 2's seeding site
    (`babylon-tick::prepare_rules`), the `scenario.vocabulary`-not-`scenario.edge_types` correction,
    the E-LOAD-001 collision refusal, AND the RULING (Major 8, corrected from this plan's first
    draft): seeding only under a declared `defvocabulary EdgeType` is a deliberate NARROWING of
    D32's literal "needs no deffield" text, chosen for reuse-of-already-tested-machinery and the
    Phase-2-registry direction (`declarations.rs`'s own module doc), not because a `FieldDecl`
    couldn't be built from the census directly (it trivially could) — recorded honestly as a
    divergence, not smoothed into an unconditional reading; the bare accessor still degrades
    gracefully without a `defvocabulary` block regardless (`tick::bind_field_value`'s own fallback),
    only unweighted aggregation needs the declaration.
  - **D140** (the `Element` cross-kind Ord ruling, CT4P A5 / issue #525): Node-before-Edge,
    declaration order, unreachable in production, pinned per the enum's own standing instruction
    (Task 3) — plus the companion note that `Element` drops `Copy` at this same landing (Blocker 1,
    nine call sites fixed, Task 3 Step 4).
  - **D141** (the strength-only read story / forward-compat substrate shape, CORRECTED from this
    plan's first draft per Major 6, wording corrected AGAIN per the same finding's residue):
    `edge_attribute` takes the FULL QNAME (mirroring `node_attribute`'s own convention exactly —
    never a bare segment). **States precisely what IS and is NOT checked, not "resolvable only
    against `<edge-type>/strength`"** (the first correction's own wording overstated it): the method
    checks ONLY that `attribute` ENDS IN `/strength` — the OWNER segment (does it actually name
    `edge_type`?) is NOT verified inside `edge_attribute` at all, exactly as `node_attribute`
    performs no ownership check of its own; that half of §2.10 discipline 1 is the CALLER's
    obligation (`field_of_edge`'s `check_edge_referent_type`, upstream of every call) — the SAME
    division of labor `node_attribute`/`check_node_referent_type` already have. This is the contract
    T3 (ADR198 R1) must PRESERVE when it widens storage: the suffix check widens to a real per-
    qname lookup, but ownership validation stays the caller's job, never migrates into the
    substrate method. The SIGNATURE does not change. Records the E-EVAL-033 unification this buys
    `field_of_edge` (Task 1/5), the layering reason the check is a plain string suffix test rather
    than a call into `babylon-bsl::vocabulary::render_member` (`babylon-graph` cannot depend on
    `babylon-bsl`), and the conformance row that pins the owner-segment non-check as deliberate
    (`edge_attribute_does_not_check_the_owner_segment`, Task 1 Step 5) so a future reader does not
    "fix" it into an owner check by surprise.
  - **D142** (the `the` disposition record — see below).

- [ ] **Step 3: The `the` disposition record.** **Recommendation: option 2 — `the`'s own micro-train
  after T2, not folded in.** This plan CONTRADICTS the dossier's neutral "both are defensible"
  framing with new evidence this plan author found by direct reading, not present in the dossier:
  `the`'s own spec-normative load-time legality gate — `E-LOAD-043` ("declared `:ceiling` other than
  1") and `E-LOAD-045` ("no manifest row") — is implemented ONLY in `babylon-bsl/src/manifest.rs`'s
  `Manifest`/`check_rule_against_manifest`, exercised ONLY by `tests/r9_chapters.rs`'s own
  test-only harness. **`rule_pipeline.rs` and `babylon-tick/src/lib.rs` reference `manifest`/
  `Manifest` NOWHERE outside comments** (verified: `rg -n "manifest|Manifest"
  rust/crates/babylon-bsl/src/rule_pipeline.rs rust/crates/babylon-tick/src/lib.rs` — THREE comment
  hits, all in `rule_pipeline.rs` (`:339,356,384`), zero calls, and zero hits at all in
  `babylon-tick/src/lib.rs`). The REAL production driver builds `CardinalityCeilings` straight from the
  scenario's own hydrated node/edge COUNTS (`scenario.node_types`/`edge_types`), never from a
  declared `(manifest ...)` form's `:ceiling` row — so `the`'s spec-mandated legality check is
  **not reachable through `run_once_into` at all today**, in either direction (no check fires, AND
  no declared-ceiling concept exists to check against). Serving `the` for real therefore needs a
  DESIGN DECISION this dossier's "cost is trivial" framing did not surface (because that framing
  measured only `manifest.rs`'s test-only STATIC cost, never checked whether the underlying
  legality machinery is wired to the real seam): either (a) wire the `(manifest ...)` form into
  `rule_pipeline::load_rule_form`/`lib.rs::prepare_rules` — new load-pipeline surface, non-trivial —
  or (b) redefine `the`'s load-time legality against the ALREADY-wired `CardinalityCeilings` census
  number instead of a separately-declared manifest ceiling — a real semantic choice (the two numbers
  coincide today by accident, not by design, and diverge the moment a scenario declares a
  larger-than-hydrated ceiling anywhere else in the codebase) that deserves its own scoped review,
  not a decision folded silently into T2's dyadic-edge-focused deliverable. This is a stronger
  reason than issue #572's original framing (which reasoned only from `the`'s LACK of technical
  dependency on `EdgeKey`) — the blocker is not entanglement with T2's own new type, it is that
  making `the` REAL surfaces a genuine, unscoped design gap in the load pipeline. Comment this
  finding on issue #572 and leave it open for its own micro-train (T2.5); do not close it as part of
  this train.

- [ ] **Step 4: ADR201.** Records: the substrate method (Task 1) and its forward-compat shape, the
  D32 wiring site and its correction of the dossier's own `scenario.edge_types` pointer (Task 2),
  the `EdgeKey`/`Value::EdgeRef`/`Element::Edge` additions and the cross-kind Ord ruling (Task 3),
  the three-file edges-refusal sweep (Task 3 Step 7), `eval_edge_between`/`field_of_edge`
  (Tasks 4-5), the `load_edge` strength-literal widening (Task 6a, amendment 2026-08-12,
  adjudicated KIND-BLIND `p`/`i`/`c` in its fix round — hydration mirrors
  `structural_verbs.rs::add_edge`'s own kind-blind `:strength` rule, and `r`/`$` stay refused
  exactly as the runtime refuses their evaluated forms; the int-arm open questions — no range
  check, and int loading at hydration while the runtime `:strength` refuses `Value::Int` — are
  recorded, not resolved; per D93 the
  widening minted no register row, so ADR201 and the plan's Amendment log are its record homes),
  the three e2e vectors (Task 6), the hash-free proof (Step 1), and the `the`
  disposition finding (Step 3) — filed as the evidence issue #572's own comment references. **Also
  records a known pre-existing divergence T2 does NOT fix (adversarial review MINOR, verified):**
  `rust/crates/babylon-bsl/tests/conformance_corpus.rs:101-104` declares `solidarity/strength` as
  `BslType::Intensity`/`FieldKind::Intensive` in that file's own hand-built `TypeEnv` fixture — the
  OPPOSITE of D32's `Coefficient`/`Extensive` and of `FieldRegistry::with_implicit_edge_strength`'s
  own kinds. No breakage results (that harness builds an entirely separate, isolated `TypeEnv` with
  `vocabulary_registry: None`, never touched by Task 2's wiring), but it is a real, pre-existing
  spec/fixture disagreement outside T2's own scope to repair — the ADR should name it rather than
  let a future reader discover it and wonder whether T2's own D32 reading is the wrong one.

- [ ] **Step 5: RST sync + all cargo legs across the whole diff.** Run
  `PYTHONPATH="$PWD/src" UV_FROZEN=1 uv run pytest tests/unit/reference/test_bsl_grammar_sync.py -q`
  (bsl-language.rst was edited) plus the full six-leg gate.

- [ ] **Step 6: Commit** `docs(p29): the T2 slice-2 edge-reads handoff — D139-D142, ADR201, the-
  disposition finding (issue #559)`.

**PR B ends here.**

---

## PR grouping

**PR A (Tasks 1-2), branch `feat/t2-edge-reads-substrate`:** the `GraphSubstrate` widening + D32
wiring. Narrow, mechanical, precedented (mirrors `node_type_of`'s own III.7 shape exactly for the
substrate half); merges alone so PR B's evaluator work reviews against a stable, already-hash-proven
foundation — matching the Territory port plan's own PR-A/PR-B split rationale (a babylon-bsl-
adjacent surface slice merging first so the larger content-facing PR reviews against something
already settled). **PR B (Tasks 3-7), branch `feat/t2-edge-reads-evaluation`:** the `Value`/
`Element` additions, the three evaluator functions, the Task 6a `load_edge` strength-literal
widening (amendment, 2026-08-12 — a hydration widening riding in the evaluator PR because Task 6's
fixtures cannot load without it; splitting it out would ship a plan-blocked PR), the e2e proof, and
the closing records. Kept as
ONE PR (not split further) because `edges`/`edge-between`/`field-of-over-EdgeRef` share one minted
type (`EdgeKey`) and one Ord ruling that only makes sense read together — splitting further would
leave a variant nothing yet constructs mid-review, which `query.rs`'s own doc calls "dead weight,
not forward-compatibility." **Verified, not assumed, what this split's actual risk is — smaller
than the `resolve_expr_bindings` 24-fixture-site precedent on one axis, larger on another**:
`Value`'s own blast radius is genuinely zero call sites by compiler force (every existing match
carries a wildcard arm, and `Value` was never `Copy` — Global Constraints), but `Element`'s loss of
`Copy` (forced by `EdgeKey` owning a `String`) is a real, nine-site ripple, fully enumerated and
fixed in Task 3 Step 4 — smaller than 24, not zero. Both are bounded, both are explicit line-item
work in this plan, and neither is a surprise left for the implementer to discover mid-PR.

## Self-review notes

- **Spec coverage:** all three slice-2 heads (`edges`, `edge-between`, `field-of` over `EdgeRef`)
  have named tasks with real code, red-first tests, and mutation evidence. The `<edge-pred>`
  operand is refused loudly (mirroring `nodes`' own `<node-pred>` refusal), not served — zero
  exercised vectors, per the dossier. `the` is explicitly out of scope, with a stronger-than-dossier
  justification recorded on issue #572 (Task 7 Step 3). **Serving a head that was previously
  refused necessarily flips every LANDED assertion that pinned the refusal — this plan enumerates
  all four such sites** (adversarial review NEW BLOCKER, initially missed for three of them):
  `query.rs`'s own dispatch-table test (Task 3 Step 5), plus `evaluator.rs`'s
  `refusal_messages_name_their_slice`, `r9_chapters.rs`'s four-heads selection test, and
  `conformance_corpus.rs`'s `event_edge_count.bsl` pin — all three swept in Task 3 Step 7, the last
  one PROMOTED to a real evaluated vector rather than merely patched, discharging that module's own
  standing doc promise. `edge-between` needed no equivalent sweep (verified: no landed
  evaluation-level refusal test exists for it).
- **Placeholder scan:** every code block in this plan is complete, real Rust/BSL — no `TODO`, no
  `// ... rest of code`, no elided match arms. Every numeric fixture value in Task 6 is an exact
  dyadic rational (a fraction with a power-of-two denominator, not necessarily a power of two
  itself — e.g. `0.1875 = 3/16`) with a hand-derivable exact sum (no bit-pinning needed; re-derived
  and re-verified
  against `E-EVAL-020`'s `[0,1]` domain after the Blocker-2 halving — the sums are: Shape 1 =
  0.796875, Shape 2a = 0.03125, Shape 3 = 0.375, all simple enough for the implementer to re-verify
  against Python's `repr()` per `query_lane_e2e.rs`'s own provenance discipline even though no
  rounding is expected).
- **Type consistency:** `EdgeKey.edge_type: String` (owned, matching `edges: HashMap<(String,
  NodeId, NodeId), f64>`'s own key shape on both backends — never `&str`, since `Value` carries no
  lifetime parameter). `GraphSubstrate::edge_attribute`'s FULL-QNAME `attribute` parameter (Major-6
  correction) is used identically by both its callers — `eval_edge_between` constructs the qname via
  `render_member` (it has only the raw enum member, never a content-authored qname); `field_of_edge`
  passes `qname` straight through unmodified (it already has the full qname from content) — no third
  call site invents a different convention, and `edge_attribute`'s own body never re-derives or
  splits either form, it only checks the suffix.
- **`Element` losing `Copy` is the largest surface-area change this plan makes, and it is now fully
  enumerated, not merely flagged** (adversarial review Blocker 1): nine call sites, all fixed with
  one of two mechanical patterns (Task 3 Step 4), zero left to the implementer's judgment.
- **Judgment calls left to the implementer, flagged per the dossier's own PLAN-MUST-VERIFY items not
  fully closed by this plan's own reading:**
  1. Whether `field_of_edge`'s round-trip through `tick::bind_field_value` produces any subtle
     divergence from `field_of_node`'s behavior (dossier item 5) — this plan's design reuses the
     identical call shape, but only a real evaluator run (Task 5 Step 3) confirms it.
  2. Shape 2b's exact tick-abort error-propagation shape (Task 6 Step 3's own flagged caveat) — no
     existing e2e vector currently exercises a genuine mid-tick `E-EVAL` abort through
     `run_once_into`'s own `Result` plumbing; this plan states the expected behavior from
     `query_lane_e2e.rs`'s prose ("a TICK ABORT") but the implementer's red-then-green pass (Task 6
     Step 5b) is the actual proof — this is exactly why Step 5b runs Shape 2b through the driver
     TWICE and compares the two `Err` strings, rather than asserting the error shape once and
     assuming it.
