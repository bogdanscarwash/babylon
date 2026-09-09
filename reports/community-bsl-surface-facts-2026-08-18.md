# Community BSL surface-facts dossier (Task 0)

**Train:** `feature/community-port-bsl`, worktree `/media/user/data/worktrees/wt-community`, BASE
`72a7e02b` (#665 merge). Plan: `docs/superpowers/plans/2026-08-18-community-port.md` (rev 2, tracked
this session at `fda96de0`). Umbrella: #557 (Program 29 — The Substrate Widening); Checkpoint A gate:
ADR208 R14.

This is the hyperedge-lane refusals, subject-type trap and numbering-allocation dossier Task 0
Step 7 commits. Every citation is `file:line` against this worktree's HEAD at time of writing.

---

## 1. Numbering allocation (Step 2)

Measured **2026-08-18**, this worktree, this HEAD:

```
$ rg -no 'D[0-9]+' docs/reference/bsl-language.rst | sort -t D -k2 -n -u | tail -5
3549:D176
8037:D177
8098:D178
8132:D179
8158:D180
$ tail ai/decisions/index.yaml
...
  ADR214_national_incidence_artifact_train: ...
```

**D-tail: D180. ADR-tail: ADR214.** Both confirmed identical to the plan's own 2026-08-17 measurement
(`docs/superpowers/plans/2026-08-18-community-port.md:269`).

**Three concurrent trains CONTEND for this allocation, so this dossier frames it against this
tree's own measured tail, never as an absolute literal** — per the launching agent's explicit
instruction:

- The **ImperialRent** train's plan claims **D181–D201** and has **re-anchored its own ADR to
  ADR215** — i.e. it does not merely count from the same D180/ADR214 tail this dossier measured, it
  has already claimed the NEXT ADR slot as well.
- **#491** has a **committed, unmerged** D181 row on its own branch (not in this tree — confirmed
  absent: this HEAD's own tail is D180, so #491's D181 has not landed here).
- This train (Community) needs its own block.

**This train's provisional, RE-MEASURE-AT-FILING allocation:**

- **D-rows:** `D<tail+1> … D<tail+25>` — measured tonight, that is the range **D181–D205** — but
  every task in this plan writes `D-NF+1 … D-NF+25` in its own prose (per the plan's own "Numbering
  is NEXT-FREE-AT-LANDING" law, plan line 265), never a literal, and **Task 12 re-measures the tail
  immediately before filing** and uses whatever range is actually free then. Given the ImperialRent
  train's claim on D181-D201, this train's real allocation at filing is likely to start north of
  D201, not D181 — that is exactly why the plan and this dossier never write the literal.
- **ADR:** `ADR<tail+1>` — measured tonight, that is **ADR215** — but ImperialRent's plan has
  **already claimed ADR215** for itself. This train's ADR will almost certainly land as **ADR216 or
  later**, decided by whichever of the two trains files first. This plan and this dossier write
  every reference to "this train's ADR" as **ADR-NF**, never a literal, for exactly this reason.

**Action for every later task in this train:** cite `D-NF+k` / `ADR-NF` only. Re-measure both tails
immediately before Task 12 files, and record what was actually free at that moment, not what this
dossier measured here.

---

## 2. Starting-line baseline (Step 3)

Machine safety: `/proc/loadavg` checked before the first cargo invocation of this task —
**5.17** (1-min), well under the 24 threshold; proceeded without a wait.

Ran `mise run rust:check` single-flight (per the plan's "heavy runs are SINGLE-FLIGHT" law). The
**first** attempt carried a `timeout 590` shell guard as a harness safety net; that guard killed the
build mid-compile (still compiling the large Bevy-dependency tree for `babylon-client`, which
`cargo test --workspace` legitimately builds as part of the workspace even though this train never
touches that crate) — `mise` reported `ERROR sh exited with non-zero status: no exit status`, the
signature of a killed-by-signal process, not a real gate failure. **Retried without the artificial
timeout**; see §2.1 for the result.

Meanwhile the box carried heavy concurrent load from other lanes sharing it (a second worktree,
`wt-b3`, running `cargo build -p babylon-client` at the same time) — `/proc/loadavg` peaked at
**146.13** (1-min) mid-run. This matches the "other lanes share the box, one holds the main build
slot" condition the launching agent's context flagged. No second cargo invocation ran while the
first was in flight (single-flight preserved); the retry is the SAME logical Step-3 run, not another
one.

### 2.1 Test/pin counts

```
$ rg -c '#\[test\]' rust/crates/babylon-tick/tests/tick_goldens.rs
18
```

18 `#[test]` functions, confirmed by name enumeration — 16 are `*_hashes_are_pinned`, the other 2 are
`worldview_member_order_is_the_ruled_ordinal` and `worldview_prelude_member_order_is_the_ruled_ordinal`
(ordinal-parity guards, not golden hash pins). **Matches the plan's expected 18/16 exactly.**

### 2.2 The 16 pinned hashes (byte-identity baseline)

Read directly from `rust/crates/babylon-tick/tests/tick_goldens.rs`'s own `assert_eq!` literals — the
committed, currently-passing pins. Paired `(before, after)` per test where the test asserts both; some
tests assert only one because `before == after` (a no-op tick) and the source states the shared
literal once.

| test fn | before | after |
|---|---|---|
| `two_classes_fundamental_theorem_hashes_are_pinned` (:61) | `5a44ab0c426eca240a0010cc70321bd0ff944d2eee2408454899a942dc85a2` `05` | `783f651d04d32fffd0109e88423eb7a57b1e0836ed4a9f645d3a8a55` `4e42767 9` |
| `vitality_conformance_hashes_are_pinned` (:78) | `20dbc24fc6ba17067cb26eb4ce4c2792c51cb0402395dc55363a5e4e3` `8572fea` | `4c7f95d967e2bf28cd5be91bbd439b61652d2c8d4103e8b5d7a3a8ad7` `89baf64` |
| `us_counties_lifecycle_demo_hashes_are_pinned` (:102) | `c190053e6d5d6eb261f1325bf87a6347dad8bb99f4e6fb7f2e297d35` `5ccc28ab` | `f4ea98647520ca8e5b2b74e4970626a179236b48efde144c91850c5` `2640f2b5d` |
| `organization_foundation_hashes_are_pinned` (:141) | `5d8d5c43088440787f993ce91bd9a676d4adf60fa35904b2afbafec` `caab93a1e` | (same as before — no-op) |
| `territory_conformance_hashes_are_pinned` (:177) | `3794b114d302a8466889795573ecf3f87547af5c200e1ead11c4fc9f` `cac88ad6` | — |
| `production_conformance_hashes_are_pinned` (:224) | `83192431e51d9be36aea347cec0861ebe352e47ee8f9bce4f39840f3` `e581ad4b` | `1538162e443afd4b1dcc020bec886e616c91bc680dffce50e52d48d` `f4af8f1eb` |
| `worldview_foundation_hashes_are_pinned` (:261) | `098ef6bd62ebc072de94d370242430d84b1b8cf2223b3b190b359ed` `6e871edbf` | (same as before — no-op) |
| `consciousness_ternary_foundation_hashes_are_pinned` (:366) | `e2582dd4f3537a6baa26fdb273e9aaf39299ab4994cf0dcf2664a90` `b920821fe` | — |
| `solidarity_conformance_hashes_are_pinned` (:458) | `20124f5ca91da3cb30fba41bc373175fdf3b06dc82f3c3b162da172` `951bb29de` | `978dbe30363c3b306bd7fa668e25d48de18c36b93930e9c4d195b5` `997ed67312` |
| `decomposition_conformance_hashes_are_pinned` (:502) | `4001e15449fbf467624417f3c4a9cca22e27bdea3320c81669808c` `5940a7eb8a` | `6bcc49d18b1e2494adf96bada45425616b955373293494d314ecd` `f20679d9b0f` |
| `decomposition_delay_conformance_hashes_are_pinned` (:535) | `40f0facb177fb535af415f99f70244663cc0ffe4fc26352efc91d3` `08301f5e1e` | `0eaf7f1459559645510efd57c71739f3ef8813409f3944b9eba51` `492d141748b` |
| `control_ratio_conformance_hashes_are_pinned` (:591) | `54f7a559a3c047561979994bd058460a3bd12ba361511117bb5227` `a32f4ad583` | `cececdab38bc6ba483baf60ee4df32cb4043073ce18fdd54ce9c86` `6c922b6e5b` |
| `control_ratio_revolution_conformance_hashes_are_pinned` (:617) | `af67a81e16e480adfc621e8617eb1edef99921a45e67b5544451d8` `f10edc4c1f` | `0ebd2a90c4868a84dd8547c5c37a99fd44cd612f2cbc53c06163847` `e7c34cb0a` |
| `control_ratio_within_capacity_conformance_hashes_are_pinned` (:642) | `f4c8d6b0a12047e713ec3d995cb70f519a4136dadb852192116d237` `ecdb0834a` | `67aa4f7bfcc2ad807331354ea786001a6dc46a7ea5a7514c87ad963` `f90860470` |
| `control_ratio_zero_enforcer_conformance_hashes_are_pinned` (:671) | `62f02edb2de87305b34ec7efd5b0a638929300a60ac8473aace3e9` `b86ccad100` | `897c1939b9f798026ddc41d9732b0b676a0b628f00b8a845a1c826` `1d5f725204` |
| `carceral_arc_conformance_hashes_are_pinned` (:709) | `504a4515c4e6d4d4c369a535c58a21ab98e8ee37ba852819c7b489` `3473881e74` | `04b2a84623e25fdf7fd7761e3c591baa8b42aa96300c76b02caca5` `9e0c74b3d6` |

(Hashes are SHA-256 hex, split across two cells above only for table-width readability — each is one
64-hex-char literal in the source, unbroken.)

**These are the byte-identity baseline every later task's gate compares against.** Per the plan's own
law: "The 16 pre-existing golden pins are byte-identical at landing. This train's OWN pins are
expected to move, and moving them is a declared step, not a STOP" (plan line 189). If any of these 16
move in a later task's `cargo test` output without a matching `MissingCeiling`/new-rule-id
explanation, **STOP**.

### 2.3 rust:check outcome

The retried single-flight run's first two legs completed; this dossier records the results
verbatim from the run's own log, `cargo fmt --all -- --check`:

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 24.44s
```

`cargo test --workspace --locked` — **every suite green**, including the load-bearing one:

```
     Running tests/tick_goldens.rs (target/debug/deps/tick_goldens-92d2e4cf008c7def)

running 18 tests
test organization_foundation_hashes_are_pinned ... ok
test two_classes_fundamental_theorem_hashes_are_pinned ... ok
test territory_conformance_hashes_are_pinned ... ok
test production_conformance_hashes_are_pinned ... ok
test vitality_conformance_hashes_are_pinned ... ok
test worldview_member_order_is_the_ruled_ordinal ... ok
test worldview_prelude_member_order_is_the_ruled_ordinal ... ok
test worldview_foundation_hashes_are_pinned ... ok
test control_ratio_zero_enforcer_conformance_hashes_are_pinned ... ok
test control_ratio_within_capacity_conformance_hashes_are_pinned ... ok
test control_ratio_revolution_conformance_hashes_are_pinned ... ok
test control_ratio_conformance_hashes_are_pinned ... ok
test solidarity_conformance_hashes_are_pinned ... ok
test decomposition_conformance_hashes_are_pinned ... ok
test decomposition_delay_conformance_hashes_are_pinned ... ok
test us_counties_lifecycle_demo_hashes_are_pinned ... ok
test carceral_arc_conformance_hashes_are_pinned ... ok
test consciousness_ternary_foundation_hashes_are_pinned ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

Every other workspace test suite in the same run (`vitality_conformance` 8/8, `us_counties_demo`
1/1, the carceral/decomposition/control-ratio suites 22/22, five crates' doc-tests 0/0) is also
green — no failures anywhere in `cargo test --workspace`. **This confirms §2.2's 16 hashes are the
currently-passing byte-identity baseline, measured by execution, not merely read from source.**

**Final status: the full six-leg gate completed GREEN, `EXIT:0`.** The run took roughly an hour
under sustained heavy multi-lane box contention (`/proc/loadavg` 1-min sustained 60–150 for most of
it — a second worktree, `wt-b3`, building `babylon-client`/Bevy concurrently, plus other lane
activity), so the remaining four legs (`cargo clippy --workspace --all-targets -- -D warnings -D
clippy::cognitive_complexity`; the three per-crate pedantic clippy+test legs for
`babylon-kernel`/`babylon-bsl`/`babylon-graph`, including `babylon-bsl`'s own 600+152+79-test
suites — every one `0 failed`; `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`) were
still mid-flight when this dossier's Step 7 commit first landed (`ba1f3249`). The background process
finished afterward and reported back:

```
    Checking babylon-tick v0.1.0 (/media/user/data/worktrees/wt-community/rust/crates/babylon-tick)
 Documenting babylon-tick v0.1.0 (/media/user/data/worktrees/wt-community/rust/crates/babylon-tick)
 Documenting babylon-client v0.1.0 (/media/user/data/worktrees/wt-community/rust/crates/babylon-client)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 05s
   Generated /media/user/data/worktrees/wt-community/rust/target/doc/babylon_bsl/index.html and 4 other files
EXIT:0
```

Grepping the full log for `error[` and `test result: FAILED` returns nothing; every `test result:`
line reads `0 failed`. **Nothing in this task wrote any Rust code** — Task 0 is docs-only — so this
green run was purely confirmatory, not a gate this task's own changes needed to pass. Once the
background completion notification arrived, I corrected this section, replacing an earlier draft
written while the run was still in flight that reported it as unfinished — left uncorrected in git
history (`ba1f3249`) per the "immutable history" documentation principle, superseded here rather
than silently rewritten.

---

## 3. Owed re-reads, recorded verbatim (Step 4)

### 3.1 `update-hyperedge` — both refusal sites

**Execute path** (`rust/crates/babylon-bsl/src/structural_verbs.rs:452-466`):

```rust
"update-hyperedge" => {
    // The verb EXISTS (D65) — this is a storage gap, not an
    // unknown head, and the message must not confuse the two.
    // T3 (ADR198 R1-R3) served update-edge's storage; hyperedge
    // own-field storage is chartered by no Program 29 train
    // (AG(i) membership payloads are #536's separate ceremony,
    // ADR198 R4).
    Err(plain(
        "(update-hyperedge …) has no substrate storage: GraphSubstrate gives a \
         hyperedge no attributes at all. Widening that state widens the canonical \
         state_hash field set, which is a declared substrate decision (Constitution \
         III.7), never a silently-dropped write"
            .to_owned(),
    ))
}
```

**Collect path** (`structural_verbs.rs:873-879`):

```rust
"update-hyperedge" => Err(plain(
    "(update-hyperedge …) has no substrate storage: GraphSubstrate gives a \
     hyperedge no attributes at all. Widening that state widens the canonical \
     state_hash field set, which is a declared substrate decision (Constitution \
     III.7), never a silently-dropped write"
        .to_owned(),
)),
```

**Neither arm destructures `items` before refusing** — both refuse unconditionally on the head symbol
alone, before either arm inspects any operand. This is load-bearing for §5(a) below.

### 3.2 Both unserved-head tables

`query.rs:99-103` (`UNSERVED_QUERY_HEADS`, the table `materialize()` consults):

```rust
const UNSERVED_QUERY_HEADS: [(&str, &str); 3] = [
    ("hyperedges", "slice 3"),
    ("members-of", "slice 3"),
    ("hyperedges-of", "slice 3"),
];
```

`evaluator.rs:544-551` (`UNSERVED_EXPRESSION_HEADS`, kept in sync per its own doc):

```rust
const UNSERVED_EXPRESSION_HEADS: [(&str, &str); 6] = [
    ("the", "slice 2"),
    ("hyperedges", "slice 3"),
    ("members-of", "slice 3"),
    ("hyperedges-of", "slice 3"),
    ("metric-of", "slice 3"),
    ("membership-field-of", "slice 4"),
];
```

`evaluator.rs:567` (`SERVED_QUERY_HEADS`, for contrast — the served set as of this HEAD):

```rust
const SERVED_QUERY_HEADS: [&str; 3] = ["nodes", "neighbors", "edges"];
```

None of `hyperedges` / `members-of` / `hyperedges-of` is in the served set yet — confirms §3.2 rows
9-10's gap is real at this HEAD, not stale.

### 3.3 `babylon-bsl`'s `tick.rs::subject_type_of` + `namespace_to_node_type` + III.11 site

`rust/crates/babylon-bsl/src/tick.rs:161-163` (`namespace_to_node_type`):

```rust
pub(crate) fn namespace_to_node_type(namespace: &str) -> String {
    namespace.to_uppercase().replace('-', "_")
}
```

`tick.rs:166-189` (`subject_type_of`):

```rust
fn subject_type_of(bindings: &[BindingDecl]) -> Result<String, TickError> {
    let mut namespaces: Vec<&str> = Vec::new();
    for binding in bindings {
        if let BindSource::Field(qname) = &binding.source {
            let namespace = qname.split('/').next().unwrap_or_default();
            if !namespaces.contains(&namespace) {
                namespaces.push(namespace);
            }
        }
    }
    match namespaces.as_slice() {
        [] => Err(err(
            "the rule declares no :field binding, so it names no subject type — \
             slice 1 runs rules over a population, not over the graph as a whole",
        )),
        [one] => Ok(namespace_to_node_type(one)),
        many => Err(err(format!(
            "the rule's :field bindings span {} namespaces ({}), so its subject type \
             is ambiguous; a field is a field OF self's node type",
            many.len(),
            many.join(", ")
        ))),
    }
}
```

`tick.rs:212-216` (the III.11 doc note on binding resolution):

```rust
/// Read one subject's **external** bindings out of the world.
///
/// An `:optional` binding with no stored value falls back to its declared
/// `:default`; a required one that was never written propagates the
/// substrate's loud error, because III.11 says absence is not zero.
```

**This is the mechanical trap the Global Constraints section names**: a `community/`-namespaced
`:field` binding on any rule literally instructs `subject_type_of` to derive `NodeType/COMMUNITY` via
`namespace_to_node_type("community")` — which does not exist as a node type (communities are
hyperedges, Anti-Pattern VIII.9). No rule in `community.bsl` may take a `:field` binding in the
`community/` namespace for exactly this reason; hyperedge fields must read through `field-of` only.

### 3.4 `scenario.rs`'s top-form dispatch and id-order law

`scenario.rs:41-45` (declaration-order-is-id-order):

```
//! **Declaration order is the id order.** Nodes are minted top to bottom, so
//! the same file always produces the same [`NodeId`] assignment and hence the
//! same state hash. Reordering two `node` forms is a real change to the
//! scenario, not a cosmetic one — which is honest, since it changes what
//! `NodeId(0)` denotes.
```

`scenario.rs:63-64` (the "no hyperedges yet" line Task 1 must retire):

```
//! - **No hyperedges yet.** The grammar has room for them; nothing in slice 1
//!   needs one, and an unused form is an untested form.
```

`scenario.rs:570-611` (the top-form dispatch, closed set at this HEAD — `defenum`, `defvocabulary`,
`deffield`, `defconst`, `node`, `edge`, `edge-attr`; no `hyperedge`/`hyperedge-attr` arm exists):

```rust
match parts.first() {
    Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "defenum" => {
        load_defenum(form, &mut enums)?;
    }
    Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "defvocabulary" => {
        load_defvocabulary(
            form,
            &mut vocabulary_members,
            &mut vocabulary_kinds_declared,
        )?;
        vocabulary_so_far = Some(ClosedVocabulary::new(vocabulary_members.clone())?);
    }
    Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "deffield" => {
        load_deffield(parts, &mut fields, &enums)?;
    }
    Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "defconst" => {
        load_defconst(parts, &mut consts)?;
    }
    Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "node" => {
        let minted = load_node(
            parts, graph, &mut named, &fields, &enums, vocabulary_so_far.as_ref(),
        )?;
        *node_types.entry(minted).or_insert(0) += 1;
        node_count += 1;
    }
    Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "edge" => {
        let minted = load_edge(
            parts, graph, &named, &mut seeded_edges, &enums, vocabulary_so_far.as_ref(),
        )?;
        *edge_types.entry(minted).or_insert(0) += 1;
        edge_count += 1;
    }
    Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "edge-attr" => {
        load_edge_attr(parts, graph, &named, /* … */)
    }
    // …
}
```

### 3.5 `state_hash.rs`'s `encode_state` elision site and `CanonicalState`'s "five-way listing" doc

`state_hash.rs:292-320` (the "five-way listing" trait doc, in full):

```rust
/// A store's five-way listing of its own contents, plus the ONE canonical
/// encoder built on top of them.
///
/// **Why this trait exists rather than widening [`crate::substrate::GraphSubstrate`].**
/// The substrate trait offers only type-keyed ranges
/// (`nodes(node_type)`, `edges(edge_type)`) and keyed point lookups — no way
/// to list which types exist, no way to list attribute names. It cannot
/// yield the canonical encoding. Listing the whole store is a storage
/// capability a store must declare separately, on a trait about
/// serialization rather than about the structural-verb surface Amendment D
/// ratified.
///
/// **The point is not tidiness — it is that a second store cannot move the
/// bytes by encoding differently, because it does not encode.** A store
/// reports facts through the five required methods; [`Self::encode_state`]
/// sorts them on the ruled key and writes the five sections (the fifth
/// elided when empty, ADR198 R2), and every store shares that one
/// implementation. A swap can change the hash only by
/// reporting a different set of facts, which is a real defect rather than a
/// formatting difference — turning an open-ended "did the bytes move?"
/// question into a closed one.
///
/// **The fifth listing is REQUIRED, not defaulted** (T3, issue #560 — the
/// dossier's design hazard, taken): a default-empty `all_edge_attributes`
/// would let a store silently forget to report edge attributes, which is
/// exactly the "reporting different facts" failure the one-encoder design
/// exists to surface. A required method makes every implementor answer the
/// question out loud, at compile time.
pub trait CanonicalState {
```

`state_hash.rs:100-104` (the TAG constants + layout-version doc, the elision's other half):

```rust
const TAG_NODES: u8 = 0x01;
const TAG_ATTRIBUTES: u8 = 0x02;
const TAG_EDGES: u8 = 0x03;
const TAG_HYPEREDGES: u8 = 0x04;
const TAG_EDGE_ATTRIBUTES: u8 = 0x05;
```

`state_hash.rs:250-256` (`write_edge_attributes`'s own "the elision decision is the CALLER's" note):

```rust
/// Section `0x05` (layout version 2 — the module doc's "Layout
/// versions"). `edge_attributes` must already be sorted ascending by
/// `(type, from, to, qname)`. **The elision decision is the CALLER's**:
/// this method writes what it is given, unconditionally;
/// [`CanonicalState::encode_state`] is the one place that decides an
/// empty fifth listing contributes zero bytes (ADR198 R2).
```

The `encode_state` elision site itself (`state_hash.rs`, inside `encode_state`'s body):

```rust
let mut edge_attributes = self.all_edge_attributes();
edge_attributes.sort_unstable_by(|a, b| { /* … */ });
// ADR198 R2: elided when empty — no tag, no count, no bytes at all.
if !edge_attributes.is_empty() {
    encoder.write_edge_attributes(&edge_attributes)?;
}
```

Section `0x06` (hyperedge attributes) does not exist yet — the file declares only `0x01`–`0x05`.
This train's E2 lane adds it, plus the sixth REQUIRED `CanonicalState` listing method and the same
caller-side elision discipline this file already establishes for `0x05`.

### 3.6 `babylon-tick/src/lib.rs:263-276` — the ceiling construction, verbatim

```rust
let ceilings = CardinalityCeilings::new(
    scenario
        .node_types
        .iter()
        .map(|(member, count)| (format!("NodeType/{member}"), *count))
        .chain(
            scenario
                .edge_types
                .iter()
                .map(|(member, count)| (format!("EdgeType/{member}"), *count)),
        )
        .collect(),
    HashMap::new(),
);
```

**Confirmed at the byte: the gap is two-part, not one.** `CardinalityCeilings::new` takes
`(ceilings: HashMap<String, u64>, max_members: HashMap<String, u64>)`
(`rust/crates/babylon-bsl/src/fuel.rs:95`). The driver chains `NodeType/*` and `EdgeType/*` into the
first map only — **it chains in no `HyperedgeType/*` entries at all** — and passes a hard-coded
empty `HashMap::new()` for the second. Today, BOTH axes fail for any hyperedge-querying rule:
`ceiling("HyperedgeType/X")` (needed by `hyperedges`/`hyperedges-of`) is `None` because no
`HyperedgeType/*` key is ever inserted, and `max_members("HyperedgeType/X")` (needed by `members-of`)
is `None` because the map is always empty. Task 4 must supply both, not just the `max_members` half.

---

## 4. Rider-4 renegotiation, both clauses (Step 5)

Quoted in full, `docs/superpowers/plans/2026-08-18-community-port.md:306-308` (charter rider 4, as
originally stated against #536's issue body and requoted by the plan) and `:303-323` (the plan's own
renegotiation of it, both clauses):

> 4. Query-side hyperedge heads stay with query-eval slices 2/3 (**CT4P #525's A5 Element-Ord pin
>    precedes them**); the WS2 duality principle (#502 comment 2026-08-12: algebra closed under the
>    dual) governs their eventual shape.

**This plan renegotiates rider 4 on BOTH clauses — not just the second, which is all revision 1
recorded.**

**Clause 1 — the A5 Element-Ord pin.** Serving `hyperedges`/`hyperedges-of` requires adding the
`Element::Hyperedge(HyperedgeId)` variant that `query.rs:17` currently calls "deliberately not"
added. That puts a THIRD kind into the cross-kind `Ord` the enum's own standing instruction (register
row D140, CT4P A5 / #525) already governs — confirmed at the byte, `query.rs:56-63`:

```rust
/// **T2's cross-kind Ord ruling (register row D140, CT4P A5 / issue #525, T2 issue #559).** §2.6
/// defines a total order WITHIN each query kind's own result set only — it is silent on comparing a
/// `Node` to an `Edge`. No production `materialize()` call ever mixes kinds (`edges` returns only
/// `Edge`; `nodes`/`neighbors` only `Node`), so this ordering is UNREACHABLE in practice — pinned
/// anyway, per this enum's own standing instruction, rather than left to whatever `#[derive(Ord)]`
/// happens to produce from declaration order. RULED: `Node` sorts before `Edge`, by declaration
/// order below — arbitrary, deliberate, tested (`tests::node_sorts_before_edge_regardless_of_id`).
```

`query.rs:68-74` — the `Element` enum as it stands today (two variants only, `Node` then `Edge`,
`#[derive(..., PartialOrd, Ord)]`):

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Element {
    /// A materialized node — see the module doc.
    Node(NodeId),
    /// A materialized dyadic edge (slice 2, T2). Declared SECOND: see this enum's own cross-kind
    /// Ord ruling above.
    Edge(EdgeKey),
}
```

**Discharging the pin in this train**, at the moment the variant lands, honours the rider's
"precedes them" clause: Task 3 declares `Hyperedge` THIRD in the enum's declaration order, rules
`Node < Edge < Hyperedge`, lands the companion test beside
`tests::node_sorts_before_edge_regardless_of_id`, amends `query.rs:17`'s "deliberately not added"
paragraph, and takes **D-NF+24**.

**Clause 2 — WS2 duality.** `members-of` and `hyperedges-of` land TOGETHER, as duals, never one
alone — the plan's Task 3/§3.4 design.

**What still does not land in this train:** `metric-of` and the `the` head (§3.4 of the plan argues
the split; out of scope here).

This dossier posts this paragraph to #536 in the same step as the implementation issue (Task 0
Step 1), so the Director sees a decision, not a drift.

---

## 5. Open shape questions, settled at the byte (Step 6)

### (a) Can a `for-each` body target a hyperedge element for `update-hyperedge` — is `it` a legal first operand?

**Yes, syntactically and by typecheck — but the question is moot at runtime.**
`element_bindings_of` (`typecheck.rs:376-395`) binds `it` to the `ScoreClass` the query's own head
maps to via `selection_result_class` (`score_class.rs:230-238`), which already has a
`"hyperedges" | "hyperedges-of" => ScoreClass::HyperedgeReference` arm — so `it` bound inside
`(for-each (hyperedges HyperedgeType/COMMUNITY) …)` legitimately carries the `HyperedgeReference`
class through to the body, exactly as `Node`/`Edge`-classed `it` does today. No operand-class check
specific to `update-hyperedge` exists anywhere in `typecheck.rs`.

But per §3.1 above, **both** `update-hyperedge` dispatch arms (execute at `:452-466`, collect at
`:873-879`) refuse unconditionally on the head symbol alone, without ever destructuring or inspecting
`items` — so whatever `it` denotes, the verb refuses before examining it. The refusal is unconditional
on ALL shapes, not merely hyperedge-typed ones. This train's E2 lane (Task 5/6) is what would make
the operand question actually matter.

### (b) Does `scale` accumulate multiplicatively across repeated writes in one rule the way `add` accumulates?

**Yes, confirmed at the byte.** `apply_pending_write` (`structural_verbs.rs:1027-1039` for the node
arm) reads `current` FRESH from the graph at APPLY time for `Add | Sub | Scale` alike:

```rust
UpdateOp::Add | UpdateOp::Sub | UpdateOp::Scale => {
    self.refuse_arithmetic_on_enum_field(&write.field, "update-node")?;
    let current = graph.node_attribute(*id, &write.field).map_err(from_graph)?;
    let combined = match write.op {
        UpdateOp::Add => current + write.operand,
        UpdateOp::Sub => current - write.operand,
        UpdateOp::Scale => current * write.operand,
        UpdateOp::Set => unreachable!("Set is handled in the arm above"),
    };
    // …
}
```

And `tick.rs:611-617` applies every `PendingWrite` **sequentially, in collection order**, writing
each result back to the graph before the next apply runs:

```rust
// ---- Pass 2: apply, in the order collected (subject order outer,
// source order inner) — `graph` is mutable again, `collect_pass`'s
// immutable borrow having already ended. ----
let mut applier = EffectExecutor::new(types, enums, None);
for write in &all_pending {
    applier.apply_pending_write(write, graph)?;
}
```

A second `scale` `PendingWrite` on the same field sees the FIRST scale's already-applied
result as its `current` — multiplicative accumulation across repeated writes in one rule (or across rules in one
tick, in collection order), the same shape `add` has for additive accumulation. This directly grounds
the port's cost-modifier design (§1.4 of the plan): reset via `set 1.0` then repeated `scale` per
community membership reproduces frozen's per-community product fold.

### (c) Does a `defconst` reference resolve inside an `if`-chain 14 levels deep, and is there an arity or nesting ceiling?

**Yes, it resolves; no explicit nesting ceiling exists.** `if` has fixed arity exactly 3
(`grammar.rs:650`, `("if", 3, 3, "exactly 3")`) — each branch is itself an arbitrary `<expr>`,
including another `if`, and `eval_if` (`evaluator.rs:681-700`) recurses through ordinary
`evaluate(taken, env, host, fuel)` calls with no depth counter of any kind. `bound_checker.rs`'s
static-cost twin, `if_cost` (`:316-331`), recurses the same way over BOTH branches (cost bounding must
account for the untaken branch too) — also no depth counter. Symbol/name resolution (`atom_value`,
`evaluator.rs:372-451`) is a flat lookup — `env.elements` (a runtime stack, for `it`/`:as` names) or
`env.bindings` (a `HashMap`, for everything else, including whatever backs a `:const`-sourced
binding) — keyed purely by name, never by AST position, so resolution correctness does not depend on
nesting depth at all. Rust's native call stack trivially accommodates 14 levels (the reader's own
iterative, non-recursive parser — `reader.rs:340` — is a different concern, hostile *parse*-time
nesting, not evaluator recursion). No `MAX_NESTING`/depth-ceiling constant exists anywhere in
`grammar.rs`, `evaluator.rs`, or `bound_checker.rs` (grepped, none found). The only ceiling that
applies is the ordinary fuel budget (`IF_BASE` charged per `if`, `:fuel` declared per rule).

### (d) Does `(field-of h community/kind)` over an `:enum-type` hyperedge field typecheck for equality against an enum-ref — does the D102 discharge generalize from nodes?

**Not reachable today (no hyperedge own-field storage exists, `field-of` over any `HyperedgeRef`
refuses unconditionally, `evaluator.rs:1318-1322` — see below), but the mechanism the question asks
about is generic over referent kind, so the generalization holds by construction once E2 lands.**

`evaluator.rs:1318-1322`, the current refusal:

```rust
Value::HyperedgeRef(_) => Err(EvalError::plain(
    "(field-of …) over a HyperedgeRef is not meaningful — a \
     hyperedge carries no attributes of its own (§2.8); a \
     membership's payload reads through membership-field-of instead \
     (slice 4)",
)),
```

D102's discharge is the read-side rendering `field_of_node` performs (`evaluator.rs:1400-1428`): it
round-trips the graph's raw `f64` through `crate::tick::bind_field_value(qname, value, types, enums)`
to produce a `Value::Enum { enum_type, member }` for an `:enum-type`-declared field. That renderer
keys on `(qname, raw value, the declared-type registry, the enum registry)` — nothing in the key is
node-specific; an analogous `field_of_hyperedge` (which E2 must build) would call the identical
`bind_field_value`. Equality itself (`apply_equality`, `evaluator.rs:1930-1948`) is a `Value`-level
match with a dedicated `(Value::Enum, Value::Enum)` arm that compares `enum_type` then `member` — the
match arm cannot tell whether either operand's `Value::Enum` came from a node, an edge, or (once
built) a hyperedge:

```rust
(
    Value::Enum { enum_type: ta, member: ma },
    Value::Enum { enum_type: tb, member: mb },
) => {
    if ta != tb {
        return Err(EvalError::plain(format!(
            "Enum<{ta}> compares only to the same enum type, found \
             Enum<{tb}> (§3.1)"
        )));
    }
    ma == mb
}
```

D102's discharge generalizes to hyperedges as soon as E2 lands `field_of_hyperedge` on the same
`bind_field_value` renderer — the equality half needs no new code at all.

### (e) Where do the two ceiling maps come from?

**RESOLVED BEFORE THE TASK, revision 2: `ceiling_of_query` (`bound_checker.rs:544-572`) already
bounds all three query heads — nowhere new needed on the bound-checker axis.** What Task 0 records
instead is the supply chain, confirmed empty at this HEAD:

- `LoadedScenario` (`scenario.rs:235-261`) declares `pub node_types: HashMap<String, u64>` and
  `pub edge_types: HashMap<String, u64>` — **no `hyperedge_types` field exists**. Task 1 must add it.
- The driver (`babylon-tick/src/lib.rs:263-276`, quoted in full at §3.6 above) chains only
  `node_types`/`edge_types` into `CardinalityCeilings::new`'s first argument and hard-codes
  `HashMap::new()` for the second. **Neither map carries any `HyperedgeType/*` entry today** —
  confirmed at the byte, not merely inferred: `ceiling("HyperedgeType/X")` and
  `max_members("HyperedgeType/X")` both return `None` unconditionally at this HEAD, for every
  hyperedge type, in every world. Task 4 owns supplying both.
- `:max-members`'s derivation (D-NF+22) is the seeded population of a `HyperedgeType`'s member —
  i.e., `max_members` is per-scenario, computed from what the scenario actually seeds, the same way
  `node_types`/`edge_types` counts are today (`scenario.rs`'s `*node_types.entry(minted).or_insert(0)
  += 1` idiom, §3.4 above) — never a manifest-declared constant independent of the world.

### (f) Are negative literals needed anywhere here?

**Confirmed: no.** Every constant this pack transcribes is non-negative by construction: the three
decay alphas (`0.05`, `0.03`, `0.1`, plan §1.5) are positive fractions; the two degeneracy/`lf`-sum
epsilons (`1e-10`) and the argmax epsilon (`1e-6`) are positive; the 14-row ADR214 floor table
(`src/babylon/models/entities/consciousness.py:356-455`) declares its values entirely in
`Probability(...)` calls, a type whose domain is `[0, 1]` by construction — grepping for a leading
`-` inside any `floor_value=Probability(...)` call across all 14 rows finds none.

### (g) What error does an unknown `HyperedgeType` member raise in a scenario type position?

**`E-LOAD-031` `UnknownEnumMember`, confirmed at `vocabulary.rs:118` (doc) and `:160` (the
`spec_code()` match arm).** `E-LOAD-023` is a DIFFERENT error — `UnknownFieldOwner`, raised when a
FIELD QNAME's first segment names no registered `NodeType`/`EdgeType`/`HyperedgeType` member at all
(`vocabulary.rs:144-146` doc, `:163` match arm) — not the "member not found within an otherwise-known
enum type" case `(g)` asks about. Revision 1 of the plan prescribed the wrong code
(`E-LOAD-023`) for this case. Confirmed at the byte before Task 1 writes any refusal against it:

```rust
/// `E-LOAD-031` — a member the registered enum type does not carry.
UnknownEnumMember {
    enum_type: String,
    member: String,
    declared: Vec<String>,
},
// …
/// `E-LOAD-023` — a field qname whose first segment names no registered
/// `NodeType`, `EdgeType` or `HyperedgeType` member.
UnknownFieldOwner {
    segment: String,
},
// …
impl VocabularyError {
    pub fn spec_code(&self) -> &'static str {
        match self {
            Self::UnknownEnumType { .. } => "E-LOAD-030",
            Self::WrongEnumKind { .. } => "E-TYPE-011",
            Self::UnknownEnumMember { .. } => "E-LOAD-031",
            Self::RenderingCollision { .. } => "E-LOAD-032",
            Self::InvalidRendering { .. } => "E-LOAD-033",
            Self::UnknownFieldOwner { .. } => "E-LOAD-023",
        }
    }
}
```

---

## 6. Cross-reference

- Implementation issue: [#667](https://github.com/percy-raskova/babylon/issues/667) — opened on
  project 8 (Babylon — Agentic Backlog), under umbrella #557, linking #536/#653/#664/ADR214/ADR198
  R4/this plan, stating the four-of-six-phases scope.
- #536 rider-4 renegotiation comment:
  [issuecomment-5324182814](https://github.com/percy-raskova/babylon/issues/536#issuecomment-5324182814)
- #564 DIRECTOR GATE popup comment (all 8 questions, posted in the same step as #667, not deferred to
  Task 12):
  [issuecomment-5324185817](https://github.com/percy-raskova/babylon/issues/564#issuecomment-5324185817)
  — note: #564 itself sits CLOSED (a prior docket-sitting issue); GitHub permits comments on closed
  issues, and the plan names #564 by number explicitly, so the comment landed there as directed.
  If the Director's active docket has since moved to a different open issue, that is a fact for a
  later task to reconcile, not something Task 0 should guess at.
