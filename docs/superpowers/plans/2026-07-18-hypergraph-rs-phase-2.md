# hypergraph-rs Phase 2 Implementation Plan — DiHypergraph + SimplicialComplex + Views

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the Phase 0+1 core with directed hypergraphs, simplicial complexes, Rust-side view facades, and the deferred membership-error hardening — all XGI v0.10.2-runtime-probed, conformance-vector-first.

**Architecture:** Same bipartite `StableDiGraph<NodeKind<N,E>, MembershipEdge<M>>` substrate. `DiHypergraph` encodes direction as **arc presence** (tail = arc agent→edge, head = arc edge→agent; both arcs = node in both) — realizing spec §3.3's "tail agents → hyperedge → head agents" literally, with NO change to `MembershipEdge` (the spec's "(a flag on MembershipEdge)" parenthetical is superseded; spec updated in Task 13). `SimplicialComplex` wraps `Hypergraph` (spec §3.4) and computes subfaces directly (the OnceCell face-lattice cache is a deferred optimization — YAGNI until profiling; spec updated in Task 13). Views are lifetimed borrow-facades over `&Hypergraph`/`&DiHypergraph` holding a materialized, insertion-ordered id subset.

**Tech Stack:** Rust (toolchain 1.91.1 pinned, MSRV 1.85), petgraph StableDiGraph, IndexMap bimaps, serde_json defaults, thiserror. XGI v0.10.2 oracle at `/home/user/projects/game/babylon/.venv/lib/python3.13/site-packages/xgi/` (its own test files at `.../site-packages/tests/core/test_{dihypergraph,simplicialcomplex,views,globalviews}.py` — 41+20+29+1 tests; the Phase 7 gate reference).

## Global Constraints

- Edition 2021; **no `unsafe`**; no new dependencies without a plan-level note (Task 13 records any).
- **XGI parity is probed against the runtime, never docstrings.** Every XGI-facing behavior gets a conformance vector FIRST (amendment H): add vectors to `hypergraph-rs/crates/hypergraph-rs/conformance/generate_fixtures.py`, regenerate with `/home/user/projects/game/babylon/.venv/bin/python crates/hypergraph-rs/conformance/generate_fixtures.py` (cwd = `hypergraph-rs/`), replay in `crates/hypergraph-rs/tests/conformance/main.rs` pinning BOTH XGI truth and Rust behavior.
- **Divergence register is append-only** (spec §4.7): new deliberate divergences get the next number (D14, D15, …), a vector, and a table row — in the SAME commit as the behavior, or the immediately following one.
- Determinism (III.7): all public iteration is insertion-ordered via bimap-filter (D5-consistent), never `neighbors()` LIFO order.
- TDD: vectors → replay red → unit tests red → implement → green → `mise run rust:check` → commit.
- Tests APPEND in-crate: unit tests to `crates/hypergraph-rs/tests/core/test_<area>.rs` (new files mirroring XGI names; register each in `crates/hypergraph-rs/tests/core/main.rs` as `mod test_<area>;`). Run with `cargo test --test core` / `cargo test --test conformance` from `hypergraph-rs/`; gate from worktree root is `mise run rust:check` (+ `mise run rust:msrv` at phase end).
- Commits: conventional + `Co-Authored-By: opencode <opencode@local>` trailer; raw `git commit` (pre-commit hooks pass).
- Pre-existing Phase 1 facts this plan relies on (verified): `Hypergraph` API per spec §4.1 minus from_memberships/from_bipartite_graph/skeleton/bipartite_graph (Phase 3+); `remove_node_from_edge` already takes `remove_empty: bool` and returns `Result<(), NodeError>` with GARBLED Display for missing-edge/not-a-member (synthetic ids — Task 1 fixes); `add_node_to_edge` returns a vestigial `Result<(), EdgeError>` with no error path (Task 1 simplifies); `MembershipEdge<M>` has only `member_data: M`; undirected membership = BOTH arcs (agent↔edge).
- Worktree: `/home/user/projects/game/babylon/.claude/worktrees/feature-hypergraph-rs-phase-0-1`; branch `feature/hypergraph-rs-phase-0-1` (Phase 2 continues on it — rename is BD business).

### Probe discipline (every task)

Each task lists **PROBES**: exact Python snippets to run with `/home/user/projects/game/babylon/.venv/bin/python -c '...'` (or a heredoc script) against real XGI BEFORE writing vectors. Record probe results as comments in the generator next to the vectors they inform. Never assert what you have not probed.

---

### Task 1: MembershipError family + membership-op unification (error-variant hardening)

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/error.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs:496-615` (`add_node_to_edge`, `remove_node_from_edge`)
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/lib.rs` (export `MembershipError`)
- Test: `hypergraph-rs/crates/hypergraph-rs/tests/core/test_hypergraph.rs` (append)
- Vectors: `conformance/generate_fixtures.py` + replay `tests/conformance/main.rs`

**Interfaces:**
- Consumes: existing `NodeError`, `EdgeError`, `Hypergraph::{add_node_to_edge, remove_node_from_edge}`.
- Produces (all later tasks rely on these):
```rust
#[derive(Debug, Clone, Error, PartialEq)]
pub enum MembershipError {
    #[error("edge {edge_id} does not exist")]
    EdgeNotFound { edge_id: String },
    #[error("node {node_id} does not exist")]
    NodeNotFound { node_id: String },
    #[error("node {node_id} is not a member of edge {edge_id}")]
    NotAMember { node_id: String, edge_id: String },
}
// Hypergraph::remove_node_from_edge(edge_id, node_id, remove_empty) -> Result<(), MembershipError>
// Hypergraph::add_node_to_edge(edge_id, node_id) -> ()        // infallible: XGI auto-creates both
```

- [ ] **Step 1: Probe XGI error behavior** — for `H.remove_node_from_edge`: (a) missing edge, (b) missing node, (c) node not in edge → confirm each raises XGIError and capture exact messages; for `H.add_node_to_edge`: confirm auto-create of missing edge AND missing node, and confirm re-adding an existing membership is a no-op (no error, no dup).
- [ ] **Step 2: Vectors** — `v_membership_errors`: XGI truth = which exception class per case + exact messages. `v_add_node_to_edge_autocreate`: state after (edge created, node created, counter bumps per D11).
- [ ] **Step 3: Replay red** — `conform_membership_errors` (XGI raises; Rust returns the matching `MembershipError` variant; binding-translation note in comment) + `conform_add_node_to_edge_autocreate`.
- [ ] **Step 4: Unit red** — tests: `remove_node_from_edge` missing edge → `MembershipError::EdgeNotFound` (Display `edge e1 does not exist`); missing node → `NodeNotFound`; not-a-member → `NotAMember` (Display `node 1 is not a member of edge e1`); `add_node_to_edge` compiles as a `()`-returning call (fix existing call sites in tests).
- [ ] **Step 5: Implement** — add `MembershipError` to error.rs; migrate `remove_node_from_edge`; change `add_node_to_edge` to return `()` (delete the `Ok(())`); update lib.rs export; fix all call sites (`tests/core/test_hypergraph.rs` currently does `.unwrap()`/asserts on the old signatures).
- [ ] **Step 6: Green + gate + commit**

Run: `mise run rust:check` — green.
```bash
git add -A && git commit -m "refactor(hypergraph-rs): MembershipError family; infallible add_node_to_edge (Phase 2 Task 1, deferred Phase 1 M1)"
```

---

### Task 2: D9 — remove_node/remove_nodes_from `remove_empty` + remove_edges_from

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs` (`remove_node` at :372, new `remove_nodes_from`, new `remove_edges_from`)
- Test: `tests/core/test_hypergraph.rs` (append)
- Vectors + replay

**Interfaces:**
- Produces:
```rust
pub fn remove_node(&mut self, node_id: &str, strong: bool, remove_empty: bool) -> Result<(), NodeError>;
pub fn remove_nodes_from(&mut self, nodes: impl IntoIterator<Item = String>, strong: bool, remove_empty: bool) -> Vec<Result<(), NodeError>>;
pub fn remove_edges_from(&mut self, edges: impl IntoIterator<Item = String>) -> Vec<Result<(), EdgeError>>;
```
(`remove_node` gains the third param — breaking change to Phase 1 signature; fix call sites. Per-item `Vec<Result>` mirrors the add_edges_from D2-class pattern.)

- [ ] **Step 1: Probe** — XGI `remove_node(n, strong=False, remove_empty=False)`: weak removal leaves an emptied edge in place (verify `H.edges` still lists it, `members(e) == set()`); `remove_empty=True` (default) drops it. Strong mode: removes all incident edges regardless — verify remove_empty is irrelevant in strong mode. Probe `remove_nodes_from` with a missing id (warn+skip? raise? capture) and `remove_edges_from` with a missing id (same).
- [ ] **Step 2: Vectors** — `v_remove_node_remove_empty` (weak+False keeps emptied edge; weak+True drops; strong ignores flag), `v_remove_nodes_from_missing` (XGI truth for missing id), `v_remove_edges_from_missing`.
- [ ] **Step 3: Replay red** (`conform_*`/`diverge_*` as probed — if XGI warns+skips, Rust per-item `Err` is the D2-class translation: `diverge_d2_remove_*_from`).
- [ ] **Step 4: Unit red** — the three behaviors + emptied-edge survival queryable via `members(e) == Some(vec![])` + num_edges unchanged.
- [ ] **Step 5: Implement** — `remove_node(node_id, strong, remove_empty)`; weak loop currently removes each emptied edge unconditionally — gate that on `remove_empty`. `remove_nodes_from`/`remove_edges_from` loop collecting per-item results. Fix existing `remove_node(id, strong)` call sites.
- [ ] **Step 6: Spec §4.7 D9 row** — flip "unimplemented (Phase 2 task)" to implemented; note `remove_empty` now exposed. Same commit.
- [ ] **Step 7: Green + gate + commit**

```bash
git commit -m "feat(hypergraph-rs): remove_empty mode on remove_node(s)_from; remove_edges_from (D9, Phase 2 Task 2)"
```

---

### Task 3: DiHypergraph — struct, node CRUD, add_edge, directed queries

**Files:**
- Create: `hypergraph-rs/crates/hypergraph-rs/src/core/dihypergraph.rs`
- Modify: `src/core/kinds.rs` (add `Direction`), `src/core/mod.rs` (+`pub mod dihypergraph;`), `src/lib.rs` (exports)
- Test: `tests/core/test_dihypergraph.rs` (NEW; register `mod test_dihypergraph;` in `tests/core/main.rs`)
- Vectors + replay

**Interfaces:**
- Consumes: `NodeKind`, `MembershipEdge`, `EdgeError`, `NodeError`, the bimap-filter pattern (hypergraph.rs:252-277).
- Produces:
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction { In, Out }   // In = tail (arc agent→edge); Out = head (arc edge→agent)

pub struct DiHypergraph<N = serde_json::Value, E = serde_json::Value, M = serde_json::Value> {
    inner: StableDiGraph<NodeKind<N, E>, MembershipEdge<M>>,
    agent_ids: IndexMap<String, NodeIndex>,
    hyperedge_ids: IndexMap<String, NodeIndex>,
    edge_uid_counter: u64,
    net_attrs: HashMap<String, serde_json::Value>,
    frozen: bool,
}
impl<N, E, M> DiHypergraph<N, E, M> {
    pub fn new() -> Self;
    pub fn add_node(&mut self, node_id: &str, attrs: N) -> bool;
    pub fn add_nodes_from(&mut self, nodes: impl IntoIterator<Item = (String, N)>);
    pub fn has_node(&self, node_id: &str) -> bool;
    pub fn num_nodes(&self) -> usize;
    pub fn num_edges(&self) -> usize;
    pub fn node_ids(&self) -> Vec<String>;
    pub fn edge_ids(&self) -> Vec<String>;
    pub fn has_edge(&self, edge_id: &str) -> bool;
    pub fn node_attrs(&self, node_id: &str) -> Option<&N>;
    pub fn edge_attrs(&self, edge_id: &str) -> Option<&E>;
    pub fn graph_attr(&self, key: &str) -> Option<&serde_json::Value>;
    pub fn set_graph_attr(&mut self, key: &str, value: serde_json::Value);
    // members = (tail, head); the (Vec, Vec) type makes XGI's
    // "Directed edge must be a list or tuple!" runtime error compile-time-impossible (D14).
    pub fn add_edge(&mut self, members: (Vec<String>, Vec<String>), idx: Option<String>, attrs: E) -> Result<String, EdgeError>
        where N: Default, M: Default + Clone;
    // Directed queries — insertion-ordered via bimap-filter (D5):
    pub fn tail(&self, edge_id: &str) -> Option<Vec<String>>;       // arcs agent→edge
    pub fn head(&self, edge_id: &str) -> Option<Vec<String>>;       // arcs edge→agent
    pub fn dimembers(&self, edge_id: &str) -> Option<(Vec<String>, Vec<String>)>; // (tail, head)
    pub fn members(&self, edge_id: &str) -> Option<Vec<String>>;    // tail ∪ head, insertion order, dedup
    pub fn dimemberships(&self, node_id: &str) -> Option<(Vec<String>, Vec<String>)>; // probe tuple order!
    pub fn memberships(&self, node_id: &str) -> Option<Vec<String>>; // union
}
```

**Arc semantics (the core invariant):** tail membership = ONLY arc `agent_idx → he_idx`; head = ONLY arc `he_idx → agent_idx`; a node in both has both arcs. `tail()` filters `agent_ids` by `inner.contains_edge(agent, he)`; `head()` by `inner.contains_edge(he, agent)`. This supersedes spec §3.3's "(a flag on MembershipEdge)" — Task 13 updates the spec.

- [ ] **Step 1: Probes** — (a) `DH.add_edge(([1,2,3],[2,3,4]))` then `DH.edges.dimembers()`, `.head()`, `.tail()`, `.members()`, `DH.nodes.dimemberships(2)` (capture exact tuple order — `(in, out)` = which first?), `DH.nodes.memberships(2)`; (b) auto-uid: first two auto edges get int ids `0`, `1`; explicit `idx="5"` bumps next auto to `6` (D3 probe for DiHypergraph — XGI `update_uid_counter` is shared); (c) `idx="5.0"`/`idx="x"` no-bump (D4/D3); (d) dup `idx` → warn+no-op, returns None; (e) `add_edge(([], []))` — empty tail+head: allowed? (D1-class probe); (f) node in BOTH tail and head survives round-trip via dimembers.
- [ ] **Step 2: Vectors** — `v_di_add_edge_dimembers` (from a), `v_di_uid_counter` (b, c), `v_di_dup_idx` (d — D2 class), `v_di_empty_edge` (e), `v_di_both_directions` (f).
- [ ] **Step 3: Replay red** — conform tests replaying each vector; for the (Vec,Vec) signature add `diverge_d14_members_must_be_pair` documenting the type-level prevention (vector records XGI's XGIError on `add_edge([1,2,3])`).
- [ ] **Step 4: Unit red** — construction, add_node bool (false on existing — probe D6-class replace semantics apply here too: `add_node` replaces attrs), add_edge returns `Ok("0")`, tail/head/dimembers/membership queries on a 2-edge fixture, missing-id queries return None, insertion order under interleaved adds.
- [ ] **Step 5: Implement** — mirror hypergraph.rs structure; add_edge creates missing nodes (N::default), dedups within tail and head separately (XGI set semantics per direction — a node in both lists lands in both), inserts arcs per the invariant, uid-counter logic IDENTICAL to hypergraph.rs:308-328 (D3/D4/D11-class — probe (b/c) verifies DiHypergraph shares `update_uid_counter`).
- [ ] **Step 6: Green + gate + register D14** (spec §4.7 row: XGI runtime XGIError on non-pair members AND on invalid direction strings → Rust type-level prevention via `(Vec,Vec)` + `Direction`; Phase 7 binding exposes shims raising XGIError). Same commit.
- [ ] **Step 7: Commit**

```bash
git commit -m "feat(hypergraph-rs): DiHypergraph core — arc-presence direction, add_edge, directed queries (Phase 2 Task 3)"
```

---

### Task 4: DiHypergraph — membership ops and removals

**Files:**
- Modify: `src/core/dihypergraph.rs`
- Test: `tests/core/test_dihypergraph.rs` (append)
- Vectors + replay

**Interfaces:**
- Consumes: Task 1 `MembershipError`, Task 3 `Direction`.
- Produces:
```rust
// Auto-creates missing edge/node (XGI parity — probe); invalid direction is compile-time-impossible (D14).
pub fn add_node_to_edge(&mut self, edge_id: &str, node_id: &str, direction: Direction) -> ()
    where N: Default, E: Default, M: Default + Clone;
pub fn remove_node_from_edge(&mut self, edge_id: &str, node_id: &str, direction: Direction, remove_empty: bool) -> Result<(), MembershipError>;
pub fn remove_edge(&mut self, edge_id: &str) -> Result<(), EdgeError>;
pub fn remove_edges_from(&mut self, edges: impl IntoIterator<Item = String>) -> Vec<Result<(), EdgeError>>;
pub fn remove_node(&mut self, node_id: &str, strong: bool, remove_empty: bool) -> Result<(), NodeError>;
pub fn remove_nodes_from(&mut self, nodes: impl IntoIterator<Item = String>, strong: bool, remove_empty: bool) -> Vec<Result<(), NodeError>>;
```

- [ ] **Step 1: Probes** — (a) `add_node_to_edge("e","n","in")` on missing edge/node → auto-created (probe uid-counter bump for numeric edge id — does DiHypergraph's version bump? XGI's DiHypergraph.add_node_to_edge does NOT call update_uid_counter — the D11 footgun! Confirm at runtime, register Rust's bump as D11-extension); (b) `remove_node_from_edge(e, n, "in", remove_empty=False)` — node removed from TAIL only; what is "empty" for a directed edge — tail∪head empty, or the DIRECTION's side empty? Capture exact XGI behavior (read L891-943 + probe); (c) `remove_node(n, strong=False)` weak: node removed from all edges (both directions); emptied edges dropped iff remove_empty — directed "emptied" definition per (b); strong: incident edges removed entirely? Capture; (d) `remove_node_from_edge` error cases → XGI messages.
- [ ] **Step 2: Vectors** — `v_di_add_node_to_edge` (a, incl. D11 divergence pin), `v_di_remove_node_from_edge` (b, d), `v_di_remove_node_modes` (c).
- [ ] **Step 3: Replay red.**
- [ ] **Step 4: Unit red** — each behavior incl. `MembershipError` variants, remove_empty=False preserving an emptied edge (`dimembers(e) == Some((vec![], vec![]))` if that is the probed definition), set-semantics re-add no-op.
- [ ] **Step 5: Implement** — remove only the arcs for `direction`; "emptied" per probed definition; D11-extension bump in add_node_to_edge; register the D11-extension note in spec §4.7 D11 row (append text, same commit).
- [ ] **Step 6: Green + gate + commit**

```bash
git commit -m "feat(hypergraph-rs): DiHypergraph membership ops + removals (Phase 2 Task 4)"
```

---

### Task 5: DiHypergraph — bulk add, attributes, copy/clear/freeze, Debug, PartialEq

**Files:**
- Modify: `src/core/dihypergraph.rs`, `src/lib.rs` if needed
- Test: `tests/core/test_dihypergraph.rs` (append)
- Vectors + replay

**Interfaces:**
- Produces:
```rust
pub fn add_edges_from(&mut self, edges: impl IntoIterator<Item = (Vec<String>, Vec<String>, Option<String>, E)>) -> Vec<Result<String, EdgeError>> where N: Default, M: Default + Clone;
pub fn node_attrs_mut(&mut self, node_id: &str) -> Option<&mut N>;
pub fn edge_attrs_mut(&mut self, edge_id: &str) -> Option<&mut E>;
pub fn clear(&mut self);            // D10 parity: resets uid counter ≡ new()
pub fn clear_edges(&mut self);
pub fn freeze(&mut self);           // guards ALL mutators incl. clear_edges (D12)
pub fn is_frozen(&self) -> bool;
pub fn copy(&self) -> Self where N: Clone, E: Clone, M: Clone;  // carries frozen (D13)
// impl Debug — XGI repr parity (probe format: "DiHypergraph([(tail, head), ...])"?)
// impl PartialEq where N/E/M: PartialEq — XGI uses algorithms.equal (probe directed-awareness)
// Value-only: set_node_attributes / set_edge_attributes (dict-of-dicts, warn+skip missing — mirror hypergraph.rs:615+)
```

- [ ] **Step 1: Probes** — (a) `repr(DH)` exact format (member order unstable → record stable projections: prefix, sorted members); (b) `DH1 == DH2` semantics: does XGI `equal` compare edge directions? attrs? net attrs? (probe: same nodes, edge (1,2)→(3) vs (3)→(1,2) — equal?); (c) `add_edges_from` formats: (members), (members, idx), (members, idx, attrdict), + `**attr` broadcast; per-edge failure (dup idx) → warn + CONTINUE (probe); (d) `set_node_attributes` scalar-broadcast vs dict-of-dicts vs name= form — mirror Phase 1 probes; (e) `clear()` counter behavior on DiHypergraph (XGI: does clear reset _edge_uid? Hypergraph did NOT — D10; probe DiHypergraph.clear + freeze guards incl. clear_edges — XGI freeze list for DiHypergraph may differ! Capture for D12-row accuracy).
- [ ] **Step 2: Vectors** — `v_di_repr`, `v_di_eq`, `v_di_add_edges_from`, `v_di_set_attrs`, `v_di_clear_freeze`.
- [ ] **Step 3: Replay red.**
- [ ] **Step 4: Unit red.**
- [ ] **Step 5: Implement** — mirror Phase 1 patterns (Tasks 10-17 precedent): attrs_mut via NodeKind match; clear resets counter (D10, documented divergence); freeze guards all mutators; copy deep-clones via inner.clone() + bimaps; Debug per probed repr with D5 ordering note; PartialEq per probed equal semantics (likely: node sets, dimembers per edge id, attrs — mirror Hypergraph's impl:80 but direction-aware).
- [ ] **Step 6: Green + gate + commit**

```bash
git commit -m "feat(hypergraph-rs): DiHypergraph bulk/attrs/copy/freeze/Debug/eq (Phase 2 Task 5)"
```

---

### Task 6: SimplicialComplex — struct, add_simplex with subface closure, has_simplex

**Files:**
- Create: `hypergraph-rs/crates/hypergraph-rs/src/core/simplicialcomplex.rs`
- Modify: `src/core/mod.rs`, `src/lib.rs`
- Test: `tests/core/test_simplicialcomplex.rs` (NEW; register mod)
- Vectors + replay

**Interfaces:**
- Consumes: `Hypergraph`, `EdgeError`.
- Produces:
```rust
pub struct SimplicialComplex<N = serde_json::Value, E = serde_json::Value, M = serde_json::Value> {
    inner: Hypergraph<N, E, M>,
}
impl<N, E, M> SimplicialComplex<N, E, M> {
    pub fn new() -> Self;
    // Blocked XGI methods (add_edge, add_edges_from, remove_edge(s)_from, add_node_to_edge,
    // add_weighted_edges_from) are ABSENT — type-level prevention (D15; binding shims raise XGIError in Phase 7).
    pub fn add_simplex(&mut self, members: Vec<String>, idx: Option<String>, attrs: E) -> Result<String, EdgeError>
        where N: Default, M: Default + Clone;  // Ok(id) for new AND already-present (XGI returns None both ways — D8 class)
    pub fn has_simplex(&self, members: &[String]) -> bool;  // member-SET comparison
    pub fn num_nodes(&self) -> usize;
    pub fn num_edges(&self) -> usize;
    pub fn node_ids(&self) -> Vec<String>;
    pub fn edge_ids(&self) -> Vec<String>;
    pub fn members(&self, edge_id: &str) -> Option<Vec<String>>;
    pub fn memberships(&self, node_id: &str) -> Option<Vec<String>>;
    pub fn node_attrs(&self, node_id: &str) -> Option<&N>;
    pub fn edge_attrs(&self, edge_id: &str) -> Option<&E>;
    pub fn graph_attr(&self, key: &str) -> Option<&serde_json::Value>;
    pub fn set_graph_attr(&mut self, key: &str, value: serde_json::Value);
    // internal: fn subfaces(members: &[String]) -> Vec<Vec<String>>  (proper non-empty subsets per probe)
    // internal: fn add_face(&mut self, members: Vec<String>) where ... (auto-idx, E::default)
}
```

- [ ] **Step 1: Probes** — (a) `S.add_simplex([1,2,3])` → `sorted(map(sorted, S.edges.members()))` — confirm subfaces are EXACTLY sizes 2..n-1 (doctest says no singletons: `[[1,2],[1,3],[2,3]]`); probe a 4-simplex (expect sizes 2,3 only — NOT singletons); (b) `S.add_simplex([1,2])` — no subfaces; (c) re-add `[3,2,1]` (same set) → silent no-op, no new edge (`num_edges` unchanged), returns None; (d) dup `idx` → warn+no-op; (e) empty simplex `S.add_simplex([])` — probe exact behavior (XGI Notes say "cannot add empty"; does it raise, skip, or create an empty edge?); (f) attrs do NOT propagate to subfaces (`S.add_simplex([1,2,3], color="red")` → `S.edges[face_id] == {}` for subfaces); (g) auto-uid: subfaces consume counter ids in what order? (probe: after add_simplex([1,2,3]) with auto ids, which id is the 3-simplex and which are faces — `_add_simplex` first with idx, then `_add_face` auto for each); (h) `S.has_simplex([2,1])` set-comparison true.
- [ ] **Step 2: Vectors** — `v_sc_add_simplex_closure` (a, b, g), `v_sc_redundant_simplex` (c), `v_sc_dup_idx` (d), `v_sc_empty_simplex` (e), `v_sc_attrs_no_propagate` (f), `v_sc_has_simplex` (h).
- [ ] **Step 3: Replay red.**
- [ ] **Step 4: Unit red** — closure contents + counts, member-set dedup, attrs isolation, auto-id sequence, `add_simplex` returns `Ok(id)` in both fresh and redundant cases (redundant → Ok(id of EXISTING edge) — probe (c) gives XGI truth that it cannot report the id; document).
- [ ] **Step 5: Implement** — `subfaces()`: all subsets of sizes `2..members.len()` (adjust to probe), computed via index-combinations (no new deps — hand-roll or `itertools` if already in tree; check Cargo.toml first, prefer hand-rolled `fn combinations`). Closure adds only faces whose member-set is not already present. `_add_face` uses next auto-id. All structural mutators assert_not_frozen.
- [ ] **Step 6: Register D15** (type-level blocked methods) + any D-rows probes revealed (e.g., empty-simplex handling, redundant-add Ok(id) divergence from XGI's None — D8-class, cite it). Same commit.
- [ ] **Step 7: Green + gate + commit**

```bash
git commit -m "feat(hypergraph-rs): SimplicialComplex — add_simplex with subface closure, has_simplex (Phase 2 Task 6)"
```

---

### Task 7: SimplicialComplex — add_simplices_from, weighted, close

**Files:**
- Modify: `src/core/simplicialcomplex.rs`
- Test: `tests/core/test_simplicialcomplex.rs` (append)
- Vectors + replay

**Interfaces:**
- Produces:
```rust
pub fn add_simplices_from(&mut self, simplices: impl IntoIterator<Item = (Vec<String>, Option<String>, E)>, max_order: Option<usize>) -> Vec<Result<String, EdgeError>> where N: Default, M: Default + Clone;
pub fn add_weighted_simplices_from(&mut self, simplices: impl IntoIterator<Item = (Vec<String>, f64, Option<String>, E)>, max_order: Option<usize>, weight_attr: &str) -> Vec<Result<String, EdgeError>> where N: Default, M: Default + Clone, E: ...;  // weight lands as attr — see probe
pub fn close(&mut self) -> ();  // adds all missing subfaces of all current edges
```

- [ ] **Step 1: Probes** — (a) `max_order` semantics: `S.add_simplices_from([[1,2,3,4]], max_order=1)` — is the 3-simplex skipped, or broken into size≤max_order+1 faces? (read XGI L373-636 + probe both `max_order=1` and `max_order=2`); (b) weighted: `S.add_weighted_simplices_from([[1,2,3,4]], max_order=2, weight="weight")` — which simplices get the weight attr (only the top? all faces? value or divided?) — XGI's weighted variant is a classic reading-comprehension trap: READ L664-709 AND probe; (c) `close()`: `S.add_simplex([1,2,3]); S2 = manual...` — construct a complex missing faces ONLY via internal paths? close() on an already-closed complex is a no-op (probe num_edges before/after); to see close() do work you need XGI's `_add_simplex` — probe `S._add_simplex(frozenset({1,2,3}), idx="x")` then `S.close()` → faces appear (private-method probe is legitimate for semantics; our Rust has no such hole — note that close() is a no-op on any complex built through our public API, which is FINE and matches XGI's public-API behavior; vector records both); (d) add_simplices_from with explicit idx entries + attr dicts + `**attr` broadcast (XGI formats); per-item dup → warn+continue.
- [ ] **Step 2: Vectors** — `v_sc_max_order`, `v_sc_weighted`, `v_sc_close`, `v_sc_add_simplices_from_formats`.
- [ ] **Step 3: Replay red.**
- [ ] **Step 4: Unit red.**
- [ ] **Step 5: Implement** — per probed semantics; weight attr: since `E` is generic, the weighted variant needs `E: Extend`? NO — simpler: bounded impl block `impl SimplicialComplex<serde_json::Value, serde_json::Value, M>` for the weighted variant ONLY (mirrors Phase 1's Value-only set_*_attributes pattern, hypergraph.rs:615), inserting `weight_attr: number` into the attr object. Register that scoping choice in a comment.
- [ ] **Step 6: Green + gate + commit**

```bash
git commit -m "feat(hypergraph-rs): SC add_simplices_from/weighted/close with max_order (Phase 2 Task 7)"
```

---

### Task 8: SimplicialComplex — removals, copy/freeze, Debug, PartialEq

**Files:**
- Modify: `src/core/simplicialcomplex.rs`
- Test: `tests/core/test_simplicialcomplex.rs` (append)
- Vectors + replay

**Interfaces:**
- Produces:
```rust
pub fn remove_simplex_id(&mut self, edge_id: &str) -> Result<(), EdgeError>;  // supface cascade + self
pub fn remove_simplex_ids_from(&mut self, edges: impl IntoIterator<Item = String>) -> Vec<Result<(), EdgeError>>;
pub fn remove_node(&mut self, node_id: &str) -> Result<(), NodeError>;        // probe SC semantics (strong?)
pub fn remove_nodes_from(&mut self, nodes: impl IntoIterator<Item = String>) -> Vec<Result<(), NodeError>>;
pub fn copy(&self) -> Self where N: Clone, E: Clone, M: Clone;
pub fn freeze(&mut self);
pub fn is_frozen(&self) -> bool;
// impl Debug — probe repr ("SimplicialComplex([...])")
// impl PartialEq where N/E/M: PartialEq — inherits Hypergraph.__eq__ semantics (probe)
// internal: fn supfaces_of(&self, members: &[String]) -> Vec<String>  // ids whose member-set ⊇ given set
```

- [ ] **Step 1: Probes** — (a) cascade: `S.add_simplex([1,2,3]); S.add_simplex([1,2,3,4])` (ids 0..?) then `S.remove_simplex_id(<id of {1,2,3}>)` → {1,2,3} AND {1,2,3,4} gone, other faces ({1,2} etc.) SURVIVE? (supface = strict superset, or ⊇? probe by removing a face id and checking the 3-simplex); (b) missing id → XGIError `Simplex {idx} is not in the Simplicialcomplex` (capture EXACT string incl. the "Simplicialcomplex" typo — the binding contract will reproduce it); (c) `remove_nodes_from`/`remove_node` on SC: removes node from all simplices AND cascades? (read L165-214 + probe: after removing node 1 from the {[1,2,3]} complex, which simplices remain?); (d) `repr(S)`; (e) `S1 == S2` (inherited equal on member sets); (f) freeze guards (SC freeze list — probe whether remove_simplex_id etc. raise after freeze; D12 applies).
- [ ] **Step 2: Vectors** — `v_sc_remove_cascade`, `v_sc_remove_missing_msg`, `v_sc_remove_node`, `v_sc_repr`, `v_sc_eq`, `v_sc_freeze`.
- [ ] **Step 3: Replay red.**
- [ ] **Step 4: Unit red.**
- [ ] **Step 5: Implement** — `supfaces_of` = member-set superset filter; cascade removes supface ids then self; `remove_node` per probed semantics (likely: for each simplex containing n, remove-simplex-id cascade? or remove node from each? PROBE DECIDES — record in vector); copy via inner.copy() (frozen carried, D13); Debug/PartialEq per probes.
- [ ] **Step 6: Green + gate + commit**

```bash
git commit -m "feat(hypergraph-rs): SC removals with supface cascade, copy/freeze/Debug/eq (Phase 2 Task 8)"
```

---

### Task 9: Views — NodeView/EdgeView core on Hypergraph

**Files:**
- Create: `hypergraph-rs/crates/hypergraph-rs/src/core/views.rs`
- Modify: `src/core/mod.rs`, `src/lib.rs`, `src/core/hypergraph.rs` (+`nodes()`/`edges()` view constructors)
- Test: `tests/core/test_views.rs` (NEW; register mod)
- Vectors + replay

**Interfaces:**
- Produces:
```rust
pub struct NodeView<'g, N, E, M> { graph: &'g Hypergraph<N, E, M>, ids: Vec<String> }  // insertion-ordered subset
pub struct EdgeView<'g, N, E, M> { graph: &'g Hypergraph<N, E, M>, ids: Vec<String> }
impl<'g, N, E, M> NodeView<'g, N, E, M> {
    pub fn ids(&self) -> &[String];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn contains(&self, node_id: &str) -> bool;
    pub fn attrs(&self, node_id: &str) -> Option<&N>;          // XGI __getitem__
    pub fn filter_bunch(&self, bunch: &[String]) -> Self;       // XGI __call__(bunch) — probe semantics
    pub fn memberships(&self, node_id: &str) -> Option<Vec<String>>;
    pub fn isolates(&self, ignore_singletons: bool) -> Self;    // returns filtered view (XGI returns NodeView)
}
impl<'g, N, E, M> EdgeView<'g, N, E, M> {
    // same core six (attrs -> Option<&E>)
    pub fn members(&self, edge_id: &str) -> Option<Vec<String>>;
    pub fn singletons(&self) -> Self;
    pub fn empty(&self) -> Self;
}
// On Hypergraph: pub fn nodes(&self) -> NodeView<'_, N, E, M>;
//                pub fn edges(&self) -> EdgeView<'_, N, E, M>;
```

- [ ] **Step 1: Probes** — (a) `H.nodes([2,99,1])` (bunch with missing id 99 + reorder): resulting `list(view.ids)` — intersection in BUNCH order or graph order? missing silently dropped? (read IDView.__init__ + probe); (b) `H.nodes[1]` missing → KeyError/IDNotFound?; (c) `isolates(ignore_singletons=True)` exact definition (node ONLY in singleton edges counts as isolate? probe with a node in exactly one singleton edge); (d) `H.nodes.memberships(<missing>)` → IDNotFound?; (e) `repr(H.nodes)`/`str(H.nodes)` formats; (f) `H.edges.empty()` / `.singletons()` on a graph with 0/1/2-member edges.
- [ ] **Step 2: Vectors** — `v_view_bunch`, `v_view_getitem_missing`, `v_view_isolates`, `v_view_memberships_missing`, `v_view_empty_singletons`.
- [ ] **Step 3: Replay red.**
- [ ] **Step 4: Unit red.**
- [ ] **Step 5: Implement** — views materialize `ids` at construction (bimap order); `filter_bunch` per probe (likely: keep bunch∩existing in BUNCH order — probe decides; insertion-order determinism either way); delegating queries scope to the view's ids (memberships/members are graph-level facts — XGI view.scoped or graph? probe: does `H.nodes([1]).memberships(1)` differ from `H.nodes.memberships(1)`? No — memberships are node facts. But `view.attrs` for an id NOT in the view → XGI __getitem__ raises even if in graph? PROBE — record.)
- [ ] **Step 6: Green + gate + commit**

```bash
git commit -m "feat(hypergraph-rs): NodeView/EdgeView core facades (Phase 2 Task 9)"
```

---

### Task 10: Views — maximal, neighbors, duplicates, lookup, filterby_attr

**Files:**
- Modify: `src/core/views.rs`, `src/lib.rs`
- Test: `tests/core/test_views.rs` (append)
- Vectors + replay

**Interfaces:**
- Produces:
```rust
pub trait AttrAccess { fn attr_get(&self, key: &str) -> Option<&serde_json::Value>; }
impl AttrAccess for serde_json::Value { ... }  // self.get(key)

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterMode { Eq, Neq, Lt, Gt, Leq, Geq, Between }  // Between: val = 2-elem JSON array

impl EdgeView { pub fn maximal(&self, strict: bool) -> Self; }
impl NodeView/EdgeView {
    pub fn neighbors(&self, id: &str, s: usize) -> Vec<String>;   // bipartite BFS — probe semantics
    pub fn duplicates(&self) -> Self;                              // same neighbor-set as another id in view
    pub fn lookup(&self, neighbor_ids: &[String]) -> Vec<String>;  // ids whose neighbor-SET == given set
    pub fn filterby_attr(&self, attr: &str, val: &serde_json::Value, mode: FilterMode, missing: &serde_json::Value) -> Self
        where N/E: AttrAccess;
}
// NOTE: filterby(stat, ...) is DEFERRED to Phase 4 (needs stats machinery) — spec §4.2 note in Task 13.
```

- [ ] **Step 1: Probes** — (a) `maximal(strict=False)` vs `strict=True` on edges {1,2}, {1,2,3}, {3,4}: exact difference (strict affects duplicate-maximal handling? read L777-847 + probe); (b) `H.nodes.neighbors(1, s=1)` vs `s=2`: same-kind neighbors within bipartite distance 2s? (probe a path graph 1-e1-2-e2-3); (c) `duplicates()`: two edges with identical members both returned? nodes with identical memberships?; (d) `lookup(["e1"])`: returns node ids whose membership-set == {"e1"} exactly; (e) `filterby_attr("x", 2, "gt")`, missing attr + `missing=` sentinel, `"between"` with (lo,hi) INCLUSIVE bounds? (probe edges of range); comparisons on non-numeric values (string lt?) → XGI behavior/error.
- [ ] **Step 2: Vectors** — `v_view_maximal`, `v_view_neighbors`, `v_view_duplicates`, `v_view_lookup`, `v_view_filterby_attr`.
- [ ] **Step 3: Replay red.**
- [ ] **Step 4: Unit red.**
- [ ] **Step 5: Implement** — maximal: O(E²) member-set comparisons (fine at our scale); neighbors: BFS on the bipartite `inner` graph from the id's index, collecting SAME-KIND ids per probed distance rule, insertion-ordered output (sort by bimap position, not BFS order — D5); duplicates/lookup: neighbor-set maps via bimap-filtered sets; filterby_attr: Value numeric comparison for Lt..Between (non-numeric → probe-decided: likely treat as non-match; XGI may raise — pin in vector, D-row if diverging).
- [ ] **Step 6: Green + gate + commit**

```bash
git commit -m "feat(hypergraph-rs): view maximal/neighbors/duplicates/lookup/filterby_attr (Phase 2 Task 10)"
```

---

### Task 11: Views — DiNodeView/DiEdgeView

**Files:**
- Modify: `src/core/views.rs` (or create `src/core/diviews.rs` if views.rs grows past ~600 lines — implementer's call, note in report), `src/core/dihypergraph.rs` (+`nodes()`/`edges()` constructors), `src/lib.rs`
- Test: `tests/core/test_views.rs` (append — keep XGI-file mirroring)
- Vectors + replay

**Interfaces:**
- Produces:
```rust
pub struct DiNodeView<'g, N, E, M> { graph: &'g DiHypergraph<N, E, M>, ids: Vec<String> }
pub struct DiEdgeView<'g, N, E, M> { graph: &'g DiHypergraph<N, E, M>, ids: Vec<String> }
impl DiNodeView { // core six as Task 9
    pub fn dimemberships(&self, node_id: &str) -> Option<(Vec<String>, Vec<String>)>;
    pub fn memberships(&self, node_id: &str) -> Option<Vec<String>>;   // union
    pub fn isolates(&self) -> Self;                                     // probe: union-empty definition
}
impl DiEdgeView { // core six
    pub fn dimembers(&self, edge_id: &str) -> Option<(Vec<String>, Vec<String>)>;
    pub fn members(&self, edge_id: &str) -> Option<Vec<String>>;        // union
    pub fn head(&self, edge_id: &str) -> Option<Vec<String>>;
    pub fn tail(&self, edge_id: &str) -> Option<Vec<String>>;
    pub fn sources(&self, edge_id: &str) -> Option<Vec<String>>;        // probe: == tail?
    pub fn targets(&self, edge_id: &str) -> Option<Vec<String>>;        // probe: == head?
    pub fn empty(&self) -> Self;                                        // probe: tail∪head empty?
}
```

- [ ] **Step 1: Probes** — sources/targets alias direction (read L1210-1229 + probe); empty() definition; isolates() definition for directed (no memberships in EITHER direction).
- [ ] **Step 2: Vectors** — `v_diview_aliases`, `v_diview_empty_isolates`.
- [ ] **Step 3-6: Replay red → unit red → implement → gate + commit**

```bash
git commit -m "feat(hypergraph-rs): DiNodeView/DiEdgeView facades (Phase 2 Task 11)"
```

---

### Task 12: globalviews — subhypergraph

**Files:**
- Create: `hypergraph-rs/crates/hypergraph-rs/src/core/globalviews.rs`
- Modify: `src/core/mod.rs`, `src/lib.rs`, `src/core/hypergraph.rs` (+`graph_attrs()` iterator — needed to clone net attrs)
- Test: `tests/core/test_globalviews.rs` (NEW; register mod)
- Vectors + replay

**Interfaces:**
- Produces:
```rust
// hypergraph.rs addition:
pub fn graph_attrs(&self) -> impl Iterator<Item = (&String, &serde_json::Value)> + '_;

// globalviews.rs:
pub fn subhypergraph<N, E, M>(
    h: &Hypergraph<N, E, M>,
    nodes: Option<&[String]>,
    edges: Option<&[String]>,
    keep_isolates: bool,
) -> Hypergraph<N, E, M>
where N: Clone, E: Clone, M: Clone;   // returns a FROZEN eager copy (XGI globalviews.py semantics)
```

Semantics (from XGI source, verified): nodes = all if None else ∩ existing; edges = all if None else ∩ existing; copy net attrs; add nodes with attrs; add edges (preserving ids + attrs) whose member-set ⊆ nodes; drop isolates iff !keep_isolates; freeze; return. Note: `add_edge` with preserved explicit ids on a fresh graph cannot collide; uid counter bumps per D3 (documented — the subhypergraph's NEXT auto id will differ from XGI's if ids are non-numeric... probe a numeric-id case and pin).

- [ ] **Step 1: Probes** — (a) nodes-only filter (induced subhypergraph: edges restricted to subsets; isolates kept by default — verify); (b) edges-only filter (all nodes kept); (c) both; (d) keep_isolates=False drops nodes left edgeless; (e) result is frozen (mutating raises) and is a NEW object (mutating original after? original unaffected); (f) missing ids in filters silently dropped (set ∩).
- [ ] **Step 2: Vectors** — `v_subhypergraph_{nodes,edges,both,no_isolates,frozen}`.
- [ ] **Step 3-6: Replay red → unit red → implement → gate + commit**

```bash
git commit -m "feat(hypergraph-rs): subhypergraph eager frozen view (Phase 2 Task 12)"
```

---

### Task 13: Spec/register reconciliation + Phase 2 completion record + final gate

**Files:**
- Modify: `docs/superpowers/specs/2026-07-18-hypergraph-rs-design.md` (§3.3 arc-presence note; §3.4 cache-deferral note; §4.2 filterby deferral note; §4.7 new rows D14/D15/+ any probe-found; §10.3 Phase 2 checkmarks)
- Modify: this plan (append "Phase 2 ACTUAL Completion Record")
- No test file.

- [ ] **Step 1:** §3.3 — replace "(a flag on MembershipEdge)" with the arc-presence invariant (tail = agent→edge arc only, head = edge→agent arc only, both arcs = both directions), noting the parenthetical was superseded during Phase 2 implementation (MembershipEdge unchanged).
- [ ] **Step 2:** §3.4 — note the OnceCell face-lattice cache is deferred (YAGNI; subfaces computed directly; profiling may reintroduce).
- [ ] **Step 3:** §4.2 — note Rust-side views landed Phase 2 with `filterby` (stat-based) deferred to Phase 4 (stats dependency); PyO3 view proxies remain Phase 7.
- [ ] **Step 4:** §4.7 — verify every new D-row from Tasks 1-12 is present with vector names; fix numbering collisions.
- [ ] **Step 5:** §10.3 Phase 2 bullets — mark complete; note XGI test-file mapping for the Phase 7 gate (test_dihypergraph 41, test_simplicialcomplex 20, test_views 29, test_globalviews 1 — vector coverage is the Phase 2 subset, full file conformance needs bindings).
- [ ] **Step 6:** Append "Phase 2 ACTUAL Completion Record" to this plan: task list w/ commits, final test counts (core/conformance/vectors), D-rows added, gate + MSRV evidence.
- [ ] **Step 7:** Run FULL gate from worktree root: `mise run rust:check` AND `mise run rust:msrv` — both green.
- [ ] **Step 8: Commit**

```bash
git commit -m "docs(spec,plan): Phase 2 reconciliation — arc-presence §3.3, register D14+, completion record"
```

---

## Self-review notes (author)

- **Spec coverage:** §10.3 Phase 2 bullets → Tasks 3-5 (DiHypergraph), 6-8 (SimplicialComplex), 9-12 (views incl. globalviews); error-variant hardening → Task 1; D9 row → Task 2. ✔
- **Type consistency:** `MembershipError` (T1) consumed by T4; `Direction` (T3) consumed by T4/T11; view constructors `nodes()`/`edges()` defined T9 (Hypergraph) / T11 (DiHypergraph); `graph_attrs()` defined T12. `remove_node(node_id, strong, remove_empty)` signature change lands T2 and is mirrored in T4's DiHypergraph from the start. ✔
- **Placeholder scan:** every probe is an explicit runnable target; no "handle edge cases" — edge cases are named vectors. ✔
