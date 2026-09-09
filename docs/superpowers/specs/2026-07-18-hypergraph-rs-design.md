# hypergraph-rs — Design Spec

**Status:** Draft, owner-approved in outline 2026-07-18. Standalone library;
Babylon adoption deferred to a later ADR (see §11).

**Constitutional framing:** Standalone, decoupled from Babylon. Article II.7
(Edges vs Hyperedges) remains in `[TRANSITION STATE — Pending Amendment D]`.
This library does NOT touch Babylon's hyperedge code; it is a portable
computational rendering package that Babylon may adopt later via a separate
spec/ADR when Amendment D is resolved. The library is "XGI semantics in Rust,
as a rustworkx-core plugin" — not an ideological package.

**Theory sources:** XGI 0.10.2 (`.venv/lib/python3.13/site-packages/xgi/`,
20,412 LOC, 164 public symbols, BSD-3-Clause) and the rustworkx-core crate
(Apache-2.0, pure-Rust, built on petgraph). Babylon's existing XGI usage
lives in 7 files under `src/babylon/` (community system, bifurcation,
graph_wrappers).

---

## 1. Vision

A standalone, open-source (BSD-3-Clause) Rust port of XGI — the Python
hypergraph library — built as a genuine `rustworkx-core` plugin, with three
distribution surfaces:

1. **Python extension** (PyO3 + Maturin) — `import hypergraph_rs as xgi` is a
   literal 1-for-1 swap for `import xgi`.
2. **WASM module** (wasm-bindgen) — React frontends can call the Rust core
   directly in the browser.
3. **CLI binary** (clap) — inspect, convert, validate, and render hypergraphs
   without writing Python.

Babylon does a literal `s/^import xgi$/import hypergraph_rs as xgi/` swap
across 7 consumer files, removing the `xgi` Poetry dependency entirely.

---

## 2. Workspace architecture

Sibling repo `../hypergraph-rs` (relative to the babylon repo root), its **own
git repository** (moved out of the babylon tree 2026-07-22; the earlier
owner ruling 2026-07-18 had it mounted at `/hypergraph-rs/` inside babylon,
parent-gitignored — the mount pattern is retired, the ignore entry stays as a
guard). Becomes a submodule/remote repo when published — publishing is an
owner decision. Babylon declares it via a **uv path dependency** at swap time
(Phase 11) and imports only the built PyO3 extension.

```
hypergraph-rs/                          # top-level workspace, external dep
├── Cargo.toml                          # workspace root
├── LICENSE                             # BSD-3-Clause
├── README.md
├── crates/
│   ├── hypergraph-rs/                  # core library (no bindings)
│   │   ├── Cargo.toml                  # rustworkx-core, petgraph, indexmap, ndarray, sprs, serde, rand, rand_pcg
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── core/                   # Hypergraph, DiHypergraph, SimplicialComplex, views
│   │       ├── algorithms/             # centrality, clustering, paths, assortativity
│   │       ├── linalg/                 # incidence, adjacency, overlap matrices
│   │       ├── generators/             # random, classic, lattice, uniform
│   │       ├── readwrite/              # JSON, edgelist, HIF, BIGG (serde)
│   │       ├── stats/                  # node/edge stat functions
│   │       ├── convert/                # to/from rustworkx, bipartite, dict
│   │       ├── dynamics/               # epidemic models
│   │       ├── communities/            # spectral clustering
│   │       ├── layout/                 # hyperedge-aware layouts (delegates to rustworkx-core)
│   │       └── viz/                    # scene-graph serializer (JSON + SVG emit)
│   ├── hypergraph-rs-python/           # PyO3 bindings (maturin)
│   │   ├── Cargo.toml                  # hypergraph-rs, pyo3, numpy
│   │   ├── pyproject.toml              # maturin build backend
│   │   ├── src/lib.rs                  # PyO3 extension
│   │   └── hypergraph_rs/              # pure-Python package skeleton mirroring XGI's __init__.py
│   │       ├── __init__.py
│   │       ├── drawing/                # matplotlib shim (mpl feature flag)
│   │       └── ...
│   ├── hypergraph-rs-wasm/             # wasm-bindgen bindings
│   │   ├── Cargo.toml                  # hypergraph-rs, wasm-bindgen, js-sys, serde-wasm-bindgen
│   │   └── src/lib.rs
│   └── hypergraph-rs-cli/              # CLI binary
│       ├── Cargo.toml                  # hypergraph-rs, clap, anyhow
│       └── src/main.rs                 # 8 subcommands
├── tests/
│   ├── conftest.py
│   └── ported/                         # XGI's 50 test files with import line swapped
└── npm/
    └── @hypergraph-rs/react/           # optional React components consuming SceneGraph JSON
```

**Key points:**

- **One Rust core, three surfaces.** `hypergraph-rs` (pure Rust, no binding
  deps) + three thin binding crates. The core does all the work; bindings are
  thin adapters.
- **`rustworkx-core` is the backbone.** The hypergraph IS a
  `rustworkx_core::petgraph::stable_graph::StableDiGraph` under the hood
  (see §3) — this is what makes it a "genuine plugin."
- **The npm package is optional/secondary.** The scene-graph JSON is the
  contract; the React renderer is one consumer.

---

## 3. Core data structure — bipartite rustworkx graph (Approach B)

The bipartite representation is the textbook one: Bretto, *Hypergraph
Theory: An Introduction* (Springer 2013, §2 — reference copy at
`sources/bretto_hypergraph-theory.pdf` with a `pdftotext` extraction
alongside) defines the **incidence graph** `IG(H)` as the bipartite graph
on `V ⊔ E` with `x ∈ V` adjacent to `e ∈ E` iff `x ∈ e`. Two consequences
the design exploits: the **incidence matrix** `A` (n×m, `a_ij = 1 iff
v_i ∈ e_j`, Bretto §1.3) is exactly the bipartite adjacency matrix, and the
**dual hypergraph** `H*` (incidence matrix `A^t`) is the same substrate
with the `NodeKind` roles swapped — `dual()` is a view, not a rebuild.
(Classical convention caveat: Bretto §1.1 excludes empty hyperedges by
default; XGI permits them. XGI parity wins — divergence D1.)

The hypergraph IS a `rustworkx_core::petgraph::stable_graph::StableDiGraph`
with two node kinds. This is what makes it a genuine rustworkx plugin — we
extend petgraph's data structure with n-ary semantics.

```rust
use rustworkx_core::petgraph::stable_graph::{NodeIndex, StableDiGraph};
use indexmap::IndexMap;

/// The two roles in the bipartite structure.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum NodeKind<N, E> {
    /// An entity node (XGI "node"). Carries its own attributes.
    Agent(N),
    /// A hyperedge node (XGI "edge"). The members of the hyperedge are
    /// exactly the Agent neighbors.
    Hyperedge(E),
}

/// A membership edge in the bipartite graph: Agent <-> Hyperedge.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MembershipEdge<M> {
    pub member_data: M,
}

/// The hypergraph. One field. Everything else is views over it.
pub struct Hypergraph<N = serde_json::Value, E = serde_json::Value, M = serde_json::Value> {
    /// Bipartite: Agent nodes + Hyperedge nodes + membership edges.
    /// Index-stable under removal (StableDiGraph leaves holes — required
    /// for III.7 determinism parity; see §3.5).
    inner: StableDiGraph<NodeKind<N, E>, MembershipEdge<M>>,

    /// Insertion-ordered bimaps for O(1) id-based lookup (mirrors
    /// BabylonGraph's `_ids` / `_index_to_id` pattern from ADR052).
    agent_ids:     IndexMap<String, NodeIndex>,   // agent_id  -> NodeIndex
    hyperedge_ids: IndexMap<String, NodeIndex>,   // edge_id   -> NodeIndex

    /// Graph-level attributes (XGI's `H.graph` dict).
    graph_attrs: serde_json::Map<String, serde_json::Value>,
}
```

### 3.1 XGI operation → bipartite implementation

| XGI API | Bipartite implementation | Complexity |
|---|---|---|
| `H.add_node(id, **attrs)` | Insert `NodeKind::Agent(attrs)`; record in `agent_ids` | O(1) |
| `H.add_edge(members, idx=id, **attrs)` | Insert `NodeKind::Hyperedge(attrs)`; for each member insert a `MembershipEdge` agent→hyperedge (and reverse for undirected) | O(\|members\|) |
| `H.nodes` | Iterate `agent_ids` keys in insertion order | O(\|agents\|) |
| `H.edges` | Iterate `hyperedge_ids` keys in insertion order | O(\|edges\|) |
| `H.nodes.memberships(agent_id)` | Bimap-filter: iterate `hyperedge_ids`, keep `contains_edge(agent, he)` — insertion-ordered (§3.5) | O(\|edges\|·deg) |
| `H.edges.members(edge_id)` | Bimap-filter: iterate `agent_ids`, keep `contains_edge(agent, he)` — insertion-ordered (§3.5) | O(\|agents\|·deg) |
| `H.edges[id]` (attrs) | Look up `NodeIndex`, read `NodeKind::Hyperedge` payload | O(1) |
| `H.nodes[id]` (attrs) | Look up `NodeIndex`, read `NodeKind::Agent` payload | O(1) |
| `xgi.incidence_matrix(H)` | The bipartite adjacency matrix — `sprs::CsMat` directly from `inner` | O(\|edges\|·\|agents\|) nnz |
| `H.remove_node(id)` | `inner.remove_node(idx)` + purge from `agent_ids` | O(degree) |
| `H.remove_edge(id)` | `inner.remove_node(hyperedge_idx)` + purge from `hyperedge_ids` | O(\|members\|) |
| `H.copy()` | `inner.clone()` + clone the two IndexMaps | O(V+E) |

### 3.2 Generic type parameters

- `N` — agent node attribute type (defaults to `serde_json::Value` for
  XGI-style dynamic attrs; Babylon can specialize to a Pydantic-validated
  struct via PyO3).
- `E` — hyperedge attribute type.
- `M` — per-membership attribute type (XGI's edge-member attrs).

Babylon can later type the generics tightly
(`Hypergraph<BabylonAgent, CommunityState, MembershipRole>`) while the
open-source lib ships with `serde_json::Value` defaults for XGI parity.

### 3.3 Directed vs undirected hypergraphs

- **`Hypergraph`** (undirected, XGI's main class) — bidirectional membership
  edges (agent↔hyperedge, both directions).
- **`DiHypergraph`** (XGI's directed class) — directed membership edges:
  tail agents → hyperedge → head agents (a flag on `MembershipEdge`).

Both share the same bipartite substrate; `DiHypergraph` adds direction
semantics on top.

### 3.4 SimplicialComplex

```rust
pub struct SimplicialComplex<N, E, M> {
    inner: Hypergraph<N, E, M>,
    // Cached face lattice: for each simplex, its cofaces and faces.
    // Computed lazily, invalidated on mutation (OnceCell pattern).
    face_lattice: OnceCell<rustworkx_core::petgraph::DiGraph<SimplexId, ()>>,
}
```

### 3.5 Determinism (III.7 parity)

The substrate is `petgraph::stable_graph::StableDiGraph` (an earlier draft
of this section named `petgraph::graph::DiGraph` — wrong on two counts:
`Graph` compacts indices on removal, scrambling insertion order; only the
`Stable*` family preserves index stability under removal, which the
insertion-order contract requires).

`StableDiGraph` removes nodes by leaving a hole (the index is marked
removed, not reused) — better for determinism than rustworkx's
`PyDiGraph` (the exact issue ADR052 worked around). Re-added nodes reuse
the hole's `NodeIndex`; because all id→index resolution goes through the
bimaps, holes are invisible to the public API.

Two hard-won clarifications (post-implementation review):

- **The bimaps ARE the ordering mechanism.** This section originally
  claimed "the bipartite substrate is naturally insertion ordered, no
  mirror dicts needed for ordering" — false. `inner.node_indices()`
  iteration order after removals does not match id-insertion order, and
  `neighbors()` yields **reverse** (LIFO) insertion order. The
  `agent_ids` / `hyperedge_ids` IndexMaps provide both O(1) id lookup
  AND the insertion-ordered iteration contract (`node_ids`, `edge_ids`).
- **Membership queries are bimap-filtered, not neighbor-iterated.**
  `members()` / `memberships()` iterate the opposite bimap and filter by
  `inner.contains_edge(...)`, which is insertion-ordered by construction
  (XGI returns unordered sets — divergence D5; we are strictly more
  defined).

---

## 4. API surface — XGI conformance mapping

The conformance gate is "XGI's 50 test files pass against the Rust port."
That means the public Python API must be `import hypergraph_rs as xgi` — a
literal swap.

### 4.1 Core API

```rust
impl<N, E, M> Hypergraph<N, E, M> {
    // Constructors
    pub fn new() -> Self;
    pub fn from_memberships(memberships: Vec<(String, Vec<String>)>) -> Self;
    pub fn from_bipartite_graph(g: rustworkx_core::petgraph::stable_graph::StableDiGraph<NodeKind<N, E>, MembershipEdge<M>>) -> Self;

    // Node CRUD
    pub fn add_node(&mut self, node_id: &str, attrs: N) -> bool;
    pub fn add_nodes_from(&mut self, nodes: impl IntoIterator<Item = (String, N)>);
    pub fn remove_node(&mut self, node_id: &str, strong: bool) -> Result<(), NodeError>;
    pub fn has_node(&self, node_id: &str) -> bool;
    pub fn num_nodes(&self) -> usize;

    // Edge CRUD — XGI uses `idx=` for the edge ID (v0.9 breaking change, pinned).
    // Fallible: duplicate idx returns Err(AlreadyExists) — divergence D2.
    pub fn add_edge(&mut self, members: Vec<String>, idx: Option<String>, attrs: E) -> Result<String, EdgeError>;
    pub fn add_edges_from(&mut self, edges: impl IntoIterator<Item = (Vec<String>, Option<String>, E)>);
    pub fn remove_edge(&mut self, edge_id: &str) -> Result<(), EdgeError>;
    pub fn has_edge(&self, edge_id: &str) -> bool;
    pub fn num_edges(&self) -> usize;

    // Membership queries (the slice Babylon uses heavily)
    pub fn memberships(&self, node_id: &str) -> Option<Vec<String>>;
    pub fn members(&self, edge_id: &str) -> Option<Vec<String>>;
    pub fn node_attrs(&self, node_id: &str) -> Option<&N>;
    pub fn edge_attrs(&self, edge_id: &str) -> Option<&E>;
    pub fn node_attrs_mut(&mut self, node_id: &str) -> Option<&mut N>;
    pub fn edge_attrs_mut(&mut self, edge_id: &str) -> Option<&mut E>;

    // Bulk
    pub fn copy(&self) -> Self where N: Clone, E: Clone, M: Clone;
    pub fn clear(&mut self);

    // XGI `H.graph` dict equivalent
    pub fn graph_attr(&self, key: &str) -> Option<&serde_json::Value>;
    pub fn set_graph_attr(&mut self, key: &str, value: serde_json::Value);

    // 1-skeleton and bipartite projections (rustworkx-native algorithms run on these)
    pub fn skeleton(&self) -> rustworkx_core::petgraph::graph::Graph<String, f64>;
    pub fn bipartite_graph(&self) -> &rustworkx_core::petgraph::stable_graph::StableDiGraph<NodeKind<N, E>, MembershipEdge<M>>;

    // Iteration (insertion-ordered, III.7 parity — eager Vec, not lazy
    // iterator: the borrow would hold the bimap for the iterator's life,
    // which fights callers; cloning ids is cheap at our scale)
    pub fn node_ids(&self) -> Vec<String>;
    pub fn edge_ids(&self) -> Vec<String>;
}
```

### 4.2 Views — PyO3 proxy objects

XGI uses proxy view objects (`H.nodes`, `H.edges`) with their own methods.
The PyO3 binding exposes `H.nodes` and `H.edges` as Python proxy objects to
match XGI's surface exactly:

```rust
#[pyclass(name = "NodeView")]
pub struct PyNodeView { /* ref to Hypergraph */ }
#[pymethods]
impl PyNodeView {
    fn memberships(&self, node_id: &str) -> HashSet<String>;
    fn __iter__(&self) -> PyNodeIter;
    fn __len__(&self) -> usize;
    fn __contains__(&self, node_id: &str) -> bool;
    fn __getitem__(&self, node_id: &str) -> PyResult<PyNodeAttrs>;
}
```

### 4.3 Full XGI module mapping (164 symbols → Rust crates)

| XGI submodule | LOC | Rust module | Porting strategy |
|---|---|---|---|
| `core/` | 4,926 | `hypergraph-rs::core` | Direct port. The bipartite substrate makes most methods one-liners. |
| `linalg/` | 840 | `hypergraph-rs::linalg` | Trivially derived from the bipartite graph via `sprs`. Returns `CsMat` (sparse) or `Array2` (dense). |
| `generators/` | 2,086 | `hypergraph-rs::generators` | Seeded `rand_pcg` for deterministic reproducibility. |
| `algorithms/` | 2,521 | `hypergraph-rs::algorithms` | Most delegate to `rustworkx-core` on the 1-skeleton or bipartite graph. |
| `stats/` | 2,414 | `hypergraph-rs::stats` | Pure stat functions over node/edge attributes. Direct port. |
| `convert/` | 1,741 | `hypergraph-rs::convert` | `networkx` → `rustworkx`; `pandas` → `Vec<Record>` (PyO3 converts to pandas on the Python side). |
| `readwrite/` | 1,122 | `hypergraph-rs::readwrite` | `serde` + `serde_json` for JSON/HIF; custom parsers for edgelist/BIGG. `xgi_data` gated behind `network` feature. |
| `dynamics/` | 296 | `hypergraph-rs::dynamics` | Small. Direct port. |
| `communities/` | 140 | `hypergraph-rs::communities` | Small. Direct port using `ndarray`/`sprs` eigendecomposition. |
| `utils/` | 1,006 | `hypergraph-rs::utils` | Utilities. Direct port. |
| `drawing/` | 3,236 | `hypergraph-rs::layout` + `hypergraph-rs::viz` | **NOT a direct port.** See §5. |

### 4.4 What we explicitly do NOT port

- **`xgi.drawing` matplotlib code** — replaced by `layout` (pure math) + `viz`
  (scene-graph serializer). The Python binding exposes a `draw_hypergraph()`
  shim (feature-gated `mpl`) for backward compat that uses matplotlib on the
  Python side over our layout.
- **`xgi.utils` pure-Python conveniences** (e.g., `dispatch` decorator
  machinery) — replaced with idiomatic Rust patterns.
- **`pandas` integration** — replaced with `Vec<Record>` and converted to
  pandas on the Python side if needed (Babylon doesn't use this slice).

### 4.5 Python import surface (the 1-for-1 swap)

```python
# Before (Babylon today):
import xgi
H = xgi.Hypergraph()
H.add_edge(members, idx=comm_type.value, heat=0.5)

# After (the swap):
import hypergraph_rs as xgi
H = xgi.Hypergraph()
H.add_edge(members, idx=comm_type.value, heat=0.5)
```

The PyO3 `__init__.py` (auto-generated by maturin, with a pure-Python
skeleton) exposes the same names as XGI's `__init__.py` — `Hypergraph`,
`DiHypergraph`, `SimplicialComplex`, `incidence_matrix`, `adjacency_matrix`,
all generator functions, all algorithms, etc. The 164 symbols.

### 4.6 Determinism contract (Constitution III.7 parity)

XGI's iteration order is insertion order (Python dicts). Our
`StableDiGraph` + `IndexMap` combo is insertion-ordered by construction
(bimap iteration, §3.5). The conformance gate (XGI tests) will catch any
order drift. We add a property-based test (like ADR052's
`test_graph_iteration_order.py`) that differential-tests against the real
XGI oracle on random hypergraph operation sequences.

### 4.7 Divergence register — behavioral truth, binding contracts

Ground rule: **parity claims are made against the XGI runtime, never its
docstrings.** D1 below exists because the plan initially trusted
`add_edge`'s docstring ("Raises XGIError") over its behavior (creates an
empty edge — XGI's own test suite asserts it). The conformance harness
(`crates/hypergraph-rs/conformance/`) encodes this register
**executably**: each divergence pins both XGI's recorded truth and Rust's
deliberate behavior, so drift on either side fails loudly.

| # | XGI v0.10.2 behavior (verified) | Rust core behavior | Binding contract (Phase 2) |
|---|---|---|---|
| D1 | `add_edge([])` creates an empty edge (docstring lies). Classical theory excludes empty hyperedges by convention (Bretto §1.1) — XGI parity wins | Same — creates it | pass-through |
| D2 | Duplicate `idx` → `UserWarning` + no-op, returns `None`. Same error-channel class: missing id on `remove_edge`/`remove_node` → `IDNotFound` raised | `Err(EdgeError::AlreadyExists)`; `Err(EdgeError::NotFound)` / `Err(NodeError::NotFound)` | translate to `warnings.warn(...)` + `None` (dup) resp. `IDNotFound` (missing) |
| D3 | String idx `"5"` does NOT bump the uid counter (only int/float idx do) | Bumps iff `idx.parse::<u64>()` succeeds (`"5"` → next auto `"6"`). Rationale: the ID boundary is stringly-typed (D7); XGI's int-idx behavior is the common case | pass-through; XGI's str-idx no-bump is pathological, not preserved |
| D4 | Float idx `5.0` bumps (`float.is_integer()`) | No bump (`"5.0"` fails `parse::<u64>`) — the D3 rule is exact | pass-through |
| D5 | `members()`/`memberships()` return unordered sets | Insertion-ordered `Vec` (strictly more defined) | convert to `set` at the boundary if a test asserts set semantics |
| D6 | `add_node` on existing node merges attrs (`{x:1}` then `{y:2}` → both) | Replaces attrs (a generic `N` cannot merge) | merge dicts before calling core |
| D7 | Ids are typed hashables (auto-ids are `int`; explicit ids arbitrary) | All ids are `String` at the core boundary | PyO3 converts: `int` ↔ `String` losslessly for integer-like ids; non-string hashables via `str()` (documented) |
| D8 | `add_edge` returns `None` | Returns `Ok(edge_id)` | binding discards, returns `None` |
| D9 | `remove_node(n, strong=False, remove_empty=True)` — three-mode. `remove_nodes_from` warns + skips + CONTINUES on a missing id ("Node {n} not in hypergraph", no "the"); `remove_edges_from` RAISES `IDNotFound` mid-iteration on the first missing id (ids before it already removed — partial effects; ids after it never attempted) | `remove_node(id, strong, remove_empty)` — all three modes exposed (implemented Phase 2 Task 2; strong ignores the flag, probed). Bulks return per-item `Vec<Result>`: `remove_nodes_from` continues past `Err` (warn→Err translation), `remove_edges_from` stops after the first `Err` (raise→Err translation; Vec length = attempted count) | expose `remove_empty`; translate per-item `Err`s to `warnings.warn(...)` + continue (nodes) resp. truncate at the first `Err` and raise `IDNotFound` (edges) |
| D10 | `H.clear()` empties everything but does NOT reset the auto-id counter (next auto id continues, e.g. `1`). **Extension (Phase 2 Task 5):** `DiHypergraph.clear()` behaves identically — counter continues at `1` (probed v0.10.2) | `clear()` resets `edge_uid_counter` — a cleared hypergraph ≡ `new()` (III.7 replay-from-empty determinism). The extension applies the same reset to `DiHypergraph.clear()` | pass-through (documented) |
| D11 | `add_node_to_edge` never bumps `_edge_uid` — auto-creating numeric edge `5` leaves the next auto id at `0`; combined with XGI's auto-id `add_edge` not existence-checking, the sequence **silently overwrites** an existing edge's members and leaves XGI internally inconsistent (stale `_node` entry). **Extension (Phase 2 Task 4):** `DiHypergraph.add_node_to_edge` does NOT call `update_uid_counter` either — the identical footgun, probed on `DiHypergraph` v0.10.2 (auto-created edge `5`, next auto id `0`) | Bumps iff `edge_id.parse::<u64>()` succeeds (D3 rule extended) — forecloses XGI's silent-overwrite class. The extension applies the same bump in `DiHypergraph.add_node_to_edge` | pass-through (XGI's footgun is not preserved) |
| D12 | `freeze()` monkey-patches a fixed method list that omits `clear_edges` — a FROZEN network can still have every edge deleted. **Extension (Phase 2 Task 5):** XGI's `DiHypergraph.freeze()` list ALSO omits `add_node_to_edge` and `remove_node_from_edge` (unlike the undirected class, which guards both) — probed: both mutate a frozen DiHypergraph unimpeded | The `frozen` flag guards ALL structural mutators uniformly, `clear_edges` included — a freeze that permits wholesale edge deletion is not a freeze. The extension guards the membership ops on `DiHypergraph` too (Task 4 landed the guard; Task 5 pinned XGI's omission) — a freeze that permits membership surgery is not a freeze | pass-through (XGI's omission is a swizzling artifact, not semantics) |
| D13 | `copy()` of a frozen network is NOT frozen (freeze is per-instance method-swizzling; the fresh instance never gets the patch) | `frozen` is data; `copy()` deep-clones data and carries the flag | pass-through (a clone that silently drops immutability is a footgun) |
| D14 | `DiHypergraph.add_edge` raises `XGIError("Directed edge must be a list or tuple!")` on non-pair members (a list of ints slips past the check and dies with a raw `TypeError`); `add_node_to_edge`/`remove_node_from_edge` raise `XGIError("Invalid direction!")` on a bad direction string | Both error classes are compile-time-impossible: `add_edge` takes `(Vec<String>, Vec<String>)` and direction is the `Direction` enum — type-level prevention | expose shims taking unvalidated members/direction strings and raising the XGIErrors verbatim for conformance |
| D15 | `SimplicialComplex`'s "blocked" methods raise `XGIError` at runtime: `add_edge` / `add_edges_from` / `add_weighted_edges_from` ("... is not available for SimplicialComplex. Use add_simplex/add_simplices_from/add_weighted_simplices_from instead."), `remove_edge` / `remove_edges_from` ("... Use remove_simplex_id/remove_simplex_ids_from instead."), and `add_node_to_edge` ("add_node_to_edge is not implemented in SimplicialComplex.") — all probed v0.10.2 | All six are ABSENT at the type level — `SimplicialComplex` has no such methods, so invalid calls are compile-time-impossible (the D14 type-level-prevention class) | expose shims raising the XGIErrors verbatim for conformance |
| D16 | `DiHypergraph.clear_edges` does not exist — attribute lookup falls through XGI's `__getattr__` stat fallback and raises `AttributeError("clear_edges is not a method of DiHypergraph or a recognized DiNodeStat or DiEdgeStat")` (probed v0.10.2) | Provided for API uniformity with `Hypergraph.clear_edges`: removes every edge, keeps all nodes/node-attrs/net-attrs, does NOT reset the uid counter (the undirected semantics — node state survives, so counter continuity matches state continuity); freeze-guarded per D12 | omit `clear_edges` from the DiHypergraph binding (AttributeError parity) |
| D17 | `SimplicialComplex.add_simplex` treats a FALSY idx (`0`, `""`) as ABSENT and replaces it with an auto id — `idx = next(self._edge_uid) if not idx else idx` (unique to SimplicialComplex: `Hypergraph`/`DiHypergraph.add_edge` test `idx is None`; probed v0.10.2: after idx `5` bumps the counter to 9, `idx=0` lands at auto id `9`, not `0`) | `Option<String>` is exact — `Some("0")` is an explicit id, `None` is auto. Rationale: falsy-means-absent is a Python idiom that silently discards a legitimate id; the D7 stringly-typed boundary makes the two channels unambiguous | binding maps Python falsy idx values to `None` (auto) before calling the core |

The register is append-only: new deliberate divergences get the next
number, a conformance vector, and a row here — never an undocumented
behavioral delta.

---

## 5. Visualization — layout + scene-graph + three rendering surfaces

### 5.1 Architecture: separate layout from render

```
Layout (pure math, Rust)        →    Scene-graph (serialized JSON)    →    Render (3 surfaces)
==================================    ====================================    =====================
hypergraph-rs::layout                hypergraph-rs::viz::SceneGraph         1. PyO3 → matplotlib shim (mpl feature)
- node positions (rustworkx)         - nodes [{id, x, y, attrs}]           2. WASM → JS objects
- hyperedge geometry                 - hyperedges [{id, members,           3. CLI → SVG file
  (halos, convex hulls,                 geometry, attrs}]                  (also JSON dump)
   bipartite stripes)                 - bounding box, viewport
- 4 layout strategies
  (spring, bipartite, hyperedge-aware, radial)
```

### 5.2 Layout (Rust, reuses rustworkx-core)

```rust
pub enum LayoutStrategy {
    /// Delegate to rustworkx-core's spring layout on the 1-skeleton.
    Spring { iterations: usize, seed: Option<u64> },
    /// Bipartite layout — reuses rustworkx-core's bipartite_layout on the
    /// inner bipartite graph.
    Bipartite { align: BipartiteAlign },
    /// Agent positions from spring layout on 1-skeleton, then each
    /// hyperedge's geometry is the convex hull of its members (rendered as
    /// a colored halo/region around the member nodes).
    HyperedgeAware { iterations: usize, halo_padding: f64 },
    /// Hyperedges as concentric rings, agents placed on rings based on
    /// membership count.
    Radial { rings: usize },
}

pub fn compute_layout(
    h: &Hypergraph<N, E, M>,
    strategy: LayoutStrategy,
) -> LayoutResult;
```

### 5.3 Scene-graph (the contract between Rust and all renderers)

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SceneGraph {
    pub nodes: Vec<NodeGlyph>,
    pub hyperedges: Vec<HyperedgeGlyph>,
    pub bounding_box: (f64, f64, f64, f64),  // min_x, min_y, max_x, max_y
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct NodeGlyph {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub attrs: serde_json::Value,
    pub style: NodeStyle,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HyperedgeGlyph {
    pub id: String,
    pub member_ids: Vec<String>,
    pub geometry: HyperedgeGeometry,
    pub attrs: serde_json::Value,
    pub style: HyperedgeStyle,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum HyperedgeGeometry {
    /// Convex hull around member nodes (XGI "halo" style)
    ConvexHull { points: Vec<(f64, f64)> },
    /// Closed spline through members (XGI "elliptical" style)
    Spline { control_points: Vec<(f64, f64)> },
    /// Bipartite stripe (for bipartite layout)
    Stripe { x: f64, y_min: f64, y_max: f64 },
    /// Radial arc (for radial layout)
    Arc { center: (f64, f64), radius: f64, angle_range: (f64, f64) },
}

impl Hypergraph<N, E, M> {
    pub fn render(&self, strategy: LayoutStrategy) -> SceneGraph;
    pub fn render_to_svg(&self, strategy: LayoutStrategy) -> String;
    pub fn render_to_json(&self, strategy: LayoutStrategy) -> String;
}
```

### 5.4 What we reuse from rustworkx visualizations

| rustworkx viz asset | Reuse strategy |
|---|---|
| `spring_layout` (Rust) | Called directly via `rustworkx-core` on the 1-skeleton |
| `kamada_kawai_layout` (Rust) | Same — direct call |
| `circular_layout`, `shell_layout`, `spiral_layout`, `random_layout` (Rust) | Same — direct calls |
| `bipartite_layout` (Rust) | Called on the inner bipartite graph directly |
| `mpl_draw` (Python, matplotlib) | NOT reused — replaced by scene-graph + optional matplotlib shim |
| `graphviz_draw` (Python, subprocess) | NOT reused — replaced by CLI SVG emit |

**Net**: all 14 Rust layout functions are reused. The two Python rendering
functions are replaced by our scene-graph + multi-surface renderers.

---

## 6. Python bindings & the Babylon swap mechanics

### 6.1 PyO3 binding architecture

The binding crate has BOTH Rust code (`src/lib.rs` for the PyO3 extension)
AND a Python package skeleton (`hypergraph_rs/` directory with `__init__.py`
and submodule shims). Maturin merges them at build time.

```python
# crates/hypergraph-rs-python/hypergraph_rs/__init__.py (auto-generated, mirrors XGI)
from . import (utils, core, algorithms, communities, convert, drawing,
               dynamics, generators, linalg, readwrite, stats)
from .utils import *
from .core import *
# ... 164 symbols total
__version__ = "0.10.2-xgi-parity"
__all__ = (core.__all__ + algorithms.__all__ + ...)
```

### 6.2 The 1-for-1 swap in Babylon

**Before** (7 Babylon files use XGI today):
```python
import xgi  # type: ignore[import-untyped, unused-ignore]
```

**After** (the swap):
```python
import hypergraph_rs as xgi  # fully typed — no ignore needed
```

The migration is literally `s/^import xgi$/import hypergraph_rs as xgi/`
across 7 consumer files. No other Python code changes.

### 6.3 The matplotlib shim — explicit optional feature

**Declaration**: The matplotlib shim is an **optional feature**, gated
behind the `mpl` extra. It is NOT installed by default. Users must opt in:

```bash
pip install hypergraph-rs          # no matplotlib
pip install 'hypergraph-rs[mpl]'   # includes matplotlib + the shim
```

**Documentation**: The crate's README, the PyO3 crate's docstring, and the
`drawing` submodule's docstring all declare:

> The `drawing` submodule provides `draw_hypergraph()` and
> `draw_dihypergraph()` functions that use matplotlib to render hypergraphs.
> This is an OPTIONAL feature gated behind the `mpl` extra
> (`pip install 'hypergraph-rs[mpl]'`). The primary visualization pathway is
> the scene-graph JSON via `H.render()` — see `viz` module. The matplotlib
> shim exists for backward compatibility with XGI's `xgi.draw()` API.

```python
# crates/hypergraph-rs-python/hypergraph_rs/drawing/__init__.py
"""Matplotlib-based drawing for backward compat with XGI.

OPTIONAL FEATURE: requires `pip install 'hypergraph-rs[mpl]'`.
Primary viz pathway is scene-graph JSON via ``H.render()``.
"""

try:
    import matplotlib.pyplot as plt
    HAS_MPL = True
except ImportError:
    HAS_MPL = False

__all__ = ["draw_hypergraph", "draw_dihypergraph", "NodeColor", "EdgeColor"]

def draw_hypergraph(H, ax=None, layout="hyperedge_aware", **kwargs):
    """Draw a Hypergraph using matplotlib (XGI ``xgi.draw`` parity).

    Optional — requires the ``mpl`` extra. Uses the Rust layout engine
    for node positions, then renders with matplotlib.
    """
    if not HAS_MPL:
        raise ImportError(
            "matplotlib drawing requires the `mpl` extra: "
            "`pip install 'hypergraph-rs[mpl]'`. "
            "For matplotlib-free rendering, use `H.render()` for a scene-graph."
        )
    from . import _core
    scene = _core.compute_scene_graph(H, layout_strategy=layout)
    ax = ax or plt.subplots()[1]
    # ... render scene-graph with matplotlib
    return ax
```

The shim calls into the Rust layout engine (no layout math in Python), then
uses matplotlib only for the actual pixel rendering. This means:

- **Layout determinism is preserved** (Rust does the math).
- **matplotlib is a pure renderer**, not a layout engine.
- The same scene-graph can be rendered to SVG/React/WASM without matplotlib.

### 6.4 Poetry integration for Babylon

```toml
[tool.poetry.dependencies]
# Existing:
rustworkx = "^0.18.0"
# Removed:
# xgi = "^0.10"
# New (path dependency to the top-level workspace):
hypergraph-rs = { path = "../hypergraph-rs/crates/hypergraph-rs-python", develop = true }
```

Babylon installs the PyO3 extension via Poetry's path dependency. Maturin
builds the Rust extension at `poetry install` time. No separate `pip install`
step.

**Fallback path** (during migration): Both `xgi` and `hypergraph-rs` can
coexist temporarily. The 7 consumer files swap one at a time, each verified
by `mise run qa:regression` (byte-identical baselines). After all 7 swap,
`xgi` is removed from `pyproject.toml`.

### 6.5 The mypy story — strict typing as a Babylon win

XGI has `py.typed` but is `type: ignore[import-untyped]`-suppressed in
Babylon's 7 consumers because XGI's stubs are incomplete. The PyO3 binding
ships complete `.pyi` stubs (maturin generates them from the PyO3
signatures), so the `type: ignore` comments are removed in the swap:

```python
# Before:
import xgi  # type: ignore[import-untyped, unused-ignore]

# After:
import hypergraph_rs as xgi  # fully typed — no ignore needed
```

This is a small mypy-strictness win for Babylon, and a requirement since
Babylon's CI runs mypy strict. **Owner directive: the richer and more
principled and stricter our typing, the easier our Python life becomes.**

### 6.6 Semgrep / vocabulary guard

Babylon has a semgrep rule banning `networkx` imports (the ADR052 "permanent
migration" enforcement). We add a parallel rule banning `xgi` imports after
the swap completes:

```yaml
# .semgrep.yml addition
- id: no-xgi-import
  patterns:
    - pattern: import xgi
  message: "xgi has been replaced by hypergraph_rs; use `import hypergraph_rs as xgi`"
  language: python
  paths: {include: ["src/babylon/**/*.py"]}
```

This makes the migration permanent (same pattern ADR052 used for NetworkX).

### 6.7 What the Babylon integration spec/ADR will need to cover

When Babylon adopts `hypergraph-rs`, a new ADR (call it ADR083) will codify:
- The substrate swap (XGI → hypergraph-rs) as a continuation of Amendment L
- The conformance gate (XGI test suite passes; Babylon qa:regression byte-identical)
- The semgrep rule making the swap permanent
- The matplotlib shim's optionality declaration
- Any baselines that needed regeneration (the ADR052 precedent: hopefully
  none, but the gate proves it)

This ADR is **deferred** — we're building the standalone lib first, swapping
Babylon second. The ADR is written when the swap is ready to land.

---

## 7. WASM binding — `hypergraph-rs-wasm`

The WASM crate is a thin adapter over the core, exposing the same API to
JavaScript/TypeScript. This is what lets React call the Rust core directly.

### 7.1 Crate structure

```rust
// crates/hypergraph-rs-wasm/Cargo.toml
[package]
name = "hypergraph-rs-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
hypergraph-rs = { path = "../hypergraph-rs" }
wasm-bindgen = "0.2"
js-sys = "0.2"
serde = { version = "1", features = ["derive"] }
serde-wasm-bindgen = "0.6"
getrandom = { version = "0.2", features = ["js"] }  # rand for browsers

[profile.release]
opt-level = "s"     # size-optimized for web
lto = true
```

### 7.2 Binding pattern — JsValue-passing for attrs, typed methods for structure

```rust
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::to_value;
use hypergraph_rs::Hypergraph as RustHypergraph;
use hypergraph_rs::layout::LayoutStrategy;
use hypergraph_rs::viz::SceneGraph;

#[wasm_bindgen]
pub struct Hypergraph {
    inner: RustHypergraph<serde_json::Value, serde_json::Value, serde_json::Value>,
}

#[wasm_bindgen]
impl Hypergraph {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self { Self { inner: RustHypergraph::new() } }

    #[wasm_bindgen(js_name = addNode)]
    pub fn add_node(&mut self, id: &str, attrs: JsValue) -> bool {
        let attrs = serde_wasm_bindgen::from_value(attrs).unwrap_or(serde_json::Value::Null);
        self.inner.add_node(id, attrs)
    }

    #[wasm_bindgen(js_name = addEdge)]
    pub fn add_edge(&mut self, members: Vec<String>, idx: Option<String>, attrs: JsValue) -> String {
        let attrs = serde_wasm_bindgen::from_value(attrs).unwrap_or(serde_json::Value::Null);
        self.inner.add_edge(members, idx, attrs)
    }

    #[wasm_bindgen(js_name = memberships)]
    pub fn memberships(&self, node_id: &str) -> Vec<String> {
        self.inner.memberships(node_id).unwrap_or_default()
    }

    /// Returns the scene-graph as a JS object (the rendering contract).
    #[wasm_bindgen(js_name = render)]
    pub fn render(&self, strategy: JsValue) -> JsValue {
        let strategy: LayoutStrategy = if strategy.is_undefined() {
            LayoutStrategy::HyperedgeAware { iterations: 100, halo_padding: 0.5 }
        } else {
            serde_wasm_bindgen::from_value(strategy).unwrap_or(LayoutStrategy::HyperedgeAware { iterations: 100, halo_padding: 0.5 })
        };
        let scene = self.inner.render(strategy);
        serde_wasm_bindgen::to_value(&scene).unwrap_or(JsValue::NULL)
    }

    /// Load from JSON (the readwrite::json_load parity).
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<Hypergraph, JsValue> {
        let inner = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("JSON parse error: {e}")))?;
        Ok(Self { inner })
    }

    /// Serialize to JSON.
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).unwrap_or_else(|_| "{}".to_string())
    }
}
```

### 7.3 Consumption from React

```typescript
import init, { Hypergraph, LayoutStrategy } from 'hypergraph-rs-wasm';

let wasmReady: Promise<void> | null = null;
function ensureWasm() {
  if (!wasmReady) wasmReady = init();
  return wasmReady;
}

export function useHypergraph() {
  const [scene, setScene] = useState<SceneGraph | null>(null);
  const ref = useRef<Hypergraph | null>(null);

  const build = useCallback(async (memberships: Membership[]) => {
    await ensureWasm();
    const H = new Hypergraph();
    for (const m of memberships) H.addEdge(m.members, m.id, m.attrs);
    ref.current = H;
    setScene(H.render(LayoutStrategy.HyperedgeAware));
  }, []);

  return { scene, build };
}
```

### 7.4 Distribution

Two npm packages:
- **`hypergraph-rs-wasm`** — the WASM module + TypeScript types (published
  by `wasm-pack publish`)
- **`@hypergraph-rs/react`** — the optional React components that consume a
  `SceneGraph` (a separate pure-TypeScript package, no WASM dependency of its
  own — it just takes JSON)

### 7.5 Size budget

WASM modules are size-sensitive for web. The release profile uses
`opt-level = "s"` + `lto = true`. Target: <500KB gzipped for the core (no
algorithms) and <1.5MB gzipped with all algorithms. Feature flags let users
exclude `dynamics`, `communities`, `generators` if they only need core + viz.

### 7.6 Determinism in the browser

The `getrandom` crate with the `js` feature uses `crypto.getRandomValues` —
non-deterministic. For **seeded** generators (the `generators` module), we
use `rand_pcg::Pcg64Mcg` seeded explicitly, same as the native build.
Determinism is opt-in per call (pass a seed), not a global property. This
matches XGI's `seed=` parameter semantics.

---

## 8. CLI — `hypergraph-rs-cli`

The CLI is the third surface — a binary for inspecting, converting,
validating, and rendering hypergraphs without writing Python.

### 8.1 Command surface (clap-derive)

```rust
#[derive(Parser)]
#[command(name = "hypergraph-rs", version, about = "XGI-compatible hypergraph toolkit (Rust)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Inspect a hypergraph file (stats summary)
    Inspect { input: PathBuf, #[arg(long, default_value = "text")] format: InspectFormat },
    /// Convert between formats (json, edgelist, hif, incidence-matrix)
    Convert { input: PathBuf, #[arg(long, value_enum)] from: Format, #[arg(long, value_enum)] to: Format, #[arg(long, default_value = "-")] output: String },
    /// Validate a hypergraph file against XGI semantics
    Validate { input: PathBuf, #[arg(long)] strict: bool },
    /// Render a hypergraph to SVG or scene-graph JSON
    Render { input: PathBuf, #[arg(long, default_value = "output.svg")] output: PathBuf, #[arg(long, default_value = "hyperedge-aware")] layout: LayoutStrategyArg, #[arg(long, default_value = "svg")] format: RenderFormat, #[arg(long)] width: Option<f32>, #[arg(long)] height: Option<f32> },
    /// Compute the incidence or adjacency matrix
    Matrix { input: PathBuf, #[arg(long, value_enum, default_value = "incidence")] kind: MatrixKind, #[arg(long, default_value = "dense")] format: MatrixFormat, #[arg(long, default_value = "-")] output: String },
    /// Run an algorithm (centrality, components, etc.) and print results
    Run { input: PathBuf, #[arg(long)] algorithm: String, #[arg(long)] node: Option<String> },
    /// Generate a hypergraph from a named generator
    Generate { #[arg(long)] generator: String, #[arg(long)] output: PathBuf, #[arg(long)] seed: Option<u64>, #[arg(long)] params: Option<String> },
    /// Self-test: run a subset of the conformance Suite
    Selftest { #[arg(long, default_value = "fast")] mode: SelftestMode },
}
```

### 8.2 Example sessions

```bash
# Inspect
$ hypergraph-rs inspect community.json
Hypergraph: community.json
  Nodes:           247
  Hyperedges:      18
  Density:         0.43
  Membership stats: min=1, max=87, mean=12.3, median=8

# Render to SVG (the visual use case)
$ hypergraph-rs render community.json --output community.svg --layout hyperedge-aware

# Render to scene-graph JSON (for React consumption via CLI pipeline)
$ hypergraph-rs render community.json --layout spring --format json --output scene.json

# Validate
$ hypergraph-rs validate community.json --strict
OK: 247 nodes, 18 hyperedges, no semantic violations.

# Self-test (CI use)
$ hypergraph-rs selftest --mode fast
Running 47 conformance tests (fast mode)...
PASS: 47/47
```

### 8.3 Design principles

1. **Thin wrapper** — the CLI has no business logic. Every command calls
   into `hypergraph-rs` core.
2. **Streaming where possible** — `--output -` writes to stdout for piping.
3. **CI-friendly** — `validate` and `selftest` exit with non-zero on failure.
4. **Render is the killer feature** — simplest way to get a hypergraph
   visualization without writing any code.

### 8.4 Exit codes

- `0` — success
- `1` — generic error (file not found, parse error)
- `2` — validation failure (`validate` command)
- `3` — conformance failure (`selftest` command)

### 8.5 Distribution

- **Cargo install**: `cargo install hypergraph-rs-cli` (Rust users)
- **Pre-built binaries** on GitHub releases (Linux x86_64, macOS arm64/x86_64,
  Windows x86_64) via `cargo-dist` or similar
- **Not published to npm or PyPI** — it's a standalone binary

---

## 9. Conformance gate — XGI test suite as oracle

The gate is "XGI's 50 test files pass against the Rust port." This is the
primary definition of done for the port.

### 9.1 The oracle: XGI's own test suite

Source: `.pre-commit-cache/repov2jlq728/` (XGI 0.10.1 full clone with
`tests/`).

We copy these 50 test files into `hypergraph-rs/tests/ported/` and adapt
them with a single transformation at port time:

```python
# Before (XGI's test):
import xgi
H = xgi.Hypergraph()
assert H.num_nodes == 0

# After (our port):
import hypergraph_rs as xgi   # ← the only change
H = xgi.Hypergraph()
assert H.num_nodes == 0
```

The test logic is **unchanged** — same assertions, same fixtures, same
expected values. If a test fails, it means our Rust port diverges from XGI's
semantics.

### 9.2 Test file inventory (the 50 files, ~650 assertions)

| Path | ~Tests | Priority |
|---|---|---|
| `tests/core/test_hypergraph.py` | 80 | P0 — most-used API |
| `tests/core/test_dihypergraph.py` | 50 | P0 |
| `tests/core/test_views.py` | 30 | P0 — `H.nodes`/`H.edges` |
| `tests/core/test_simplicialcomplex.py` | 40 | P1 |
| `tests/core/test_globalviews.py` | 15 | P1 |
| `tests/algorithms/test_centrality.py` | 25 | P1 |
| `tests/algorithms/test_clustering.py` | 20 | P1 |
| `tests/algorithms/test_shortest_path.py` | 15 | P1 |
| `tests/algorithms/test_assortativity.py` | 15 | P2 |
| `tests/algorithms/test_connected.py` | 10 | P1 |
| `tests/algorithms/test_properties.py` | 20 | P2 |
| `tests/algorithms/test_simpliciality.py` | 15 | P2 |
| `tests/stats/test_nodestats.py` | 20 | P2 |
| `tests/stats/test_edgestats.py` | 20 | P2 |
| `tests/stats/test_dinodestats.py` | 15 | P2 |
| `tests/stats/test_diedgestats.py` | 15 | P2 |
| `tests/stats/test_core_stats_functions.py` | 20 | P2 |
| `tests/generators/test_random.py` | 20 | P1 — determinism-sensitive |
| `tests/generators/test_classic.py` | 15 | P1 |
| `tests/generators/test_lattice.py` | 10 | P2 |
| `tests/generators/test_uniform.py` | 10 | P2 |
| `tests/generators/test_randomizing.py` | 10 | P2 |
| `tests/generators/test_simplicial_complexes.py` | 10 | P2 |
| `tests/readwrite/test_json.py` | 15 | P1 |
| `tests/readwrite/test_edgelist.py` | 10 | P2 |
| `tests/readwrite/test_incidence_matrix.py` | 15 | P1 — Babylon uses this |
| `tests/readwrite/test_bigg_data.py` | 10 | P3 |
| `tests/readwrite/test_hif.py` | 10 | P3 |
| `tests/readwrite/test_bipartite_edgelist.py` | 10 | P2 |
| `tests/readwrite/test_xgi_data.py` | 5 | P3 (network) |
| `tests/communities/test_spectral.py` | 10 | P2 |
| `tests/drawing/test_draw.py` | 20 | P3 — replaced by scene-graph tests |
| `tests/drawing/test_draw_utils.py` | 10 | P3 |
| `tests/drawing/test_layout.py` | 15 | P1 — layout is our viz base |
| `tests/utils/test_utilities.py` | 15 | P2 |
| `tests/conftest.py` | (fixtures) | P0 — shared fixtures |

### 9.3 Drawing tests — special handling

XGI's `tests/drawing/test_draw.py` uses matplotlib's image comparison
(`@pytest.mark.mpl_image_compare`) — pixel-level comparison of rendered
PNGs. We **do not port these as-is**. Instead:

- `test_layout.py` → ported (layout math is reusable, asserts on node
  position values not pixels)
- `test_draw.py`, `test_draw_utils.py` → replaced with scene-graph tests:

```python
def test_render_produces_valid_scene_graph():
    import hypergraph_rs as xgi
    H = xgi.Hypergraph()
    H.add_edge(["a", "b", "c"], idx="e1")
    scene = H.render(xgi.LayoutStrategy.HYPEREDGE_AWARE)
    assert len(scene["nodes"]) == 3
    assert len(scene["hyperedges"]) == 1
    assert scene["hyperedges"][0]["member_ids"] == ["a", "b", "c"]
    assert scene["bounding_box"][2] > scene["bounding_box"][0]  # max_x > min_x
```

### 9.4 Generators — determinism sensitivity

XGI's generator tests use `numpy.random` with explicit seeds. Our Rust port
uses `rand_pcg` (Pcg64Mcg) with the same seeds. **This is a known conformance
risk**: different RNGs produce different sequences even with the same seed.

**Two options for the generator tests:**
- **(a) Seed-portable tests**: XGI's tests that assert on *structural
  properties* (e.g., "the generated hypergraph has 50 nodes and 10
  hyperedges") pass regardless of RNG. Tests that assert on *exact
  membership* (e.g., "edge 3 contains nodes [a, b, c]") will fail because
  the RNG sequence differs.
- **(b) Record-replay**: For exact-membership tests, we record XGI's output
  once, save as fixtures, and assert against those fixtures. This makes the
  tests deterministic but they no longer test the generator's *behavior*,
  only its *output parity*.

**Recommended**: Option (a) for most generator tests (structural property
assertions), Option (b) for a small set of "golden output" tests where exact
membership matters. We document which generator tests are property-based vs
parity-based in the test file headers.

### 9.5 Property-based drift backstop (the ADR052 pattern)

Beyond XGI's tests, we add a property-based differential test mirroring
ADR052's `test_graph_iteration_order.py`:

```rust
// hypergraph-rs/tests/property_differential.rs
use proptest::prelude::*;

proptest! {
    /// Differential test: any sequence of add/remove operations on our
    /// Hypergraph must produce the same membership/iteration behavior
    /// as XGI's Hypergraph (run via PyO3 in the same test).
    #[test]
    fn differential_vs_xgi(ops: Vec<HyperOp>) {
        let mut ours = hypergraph_rs::Hypergraph::new();
        let mut theirs = xgi::Hypergraph::new();  // via PyO3

        for op in ops {
            // apply op to both
            // assert membership queries, iteration order, num_nodes, num_edges all match
        }
    }
}
```

This runs the real XGI as a Python oracle (via PyO3) inside a Rust proptest.
Random operation sequences, same assertions on both. Catches any semantic
drift the fixed test suite misses. Same pattern that proved BabylonGraph's
order contract in ADR052.

### 9.6 Babylon qa:regression as a secondary gate

The user chose "XGI test suite as oracle" (not "both gates"). Babylon's
`mise run qa:regression` (5-scenario byte-identical baselines) is a
**secondary** gate that activates only when Babylon swaps. It's not part of
the hypergraph-rs conformance gate itself, but it's the gate that proves the
swap didn't perturb the simulation. Two independent verification strategies —
matches Constitution III.12 (Redundant verification), but each is its own
gate at its own phase.

### 9.7 CI integration

```yaml
# hypergraph-rs/.github/workflows/ci.yml
jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - run: cargo test -p hypergraph-rs
      - run: cargo test -p hypergraph-rs-python

  xgi-conformance:
    runs-on: ubuntu-latest
    steps:
      - run: pip install maturin && maturin develop
      - run: pytest tests/ported/                      # XGI's 50 test files, adapted
      - run: cargo test --test property_differential   # the proptest oracle

  wasm-build:
    runs-on: ubuntu-latest
    steps:
      - run: wasm-pack build crates/hypergraph-rs-wasm --target web

  cli-build:
    runs-on: ubuntu-latest
    steps:
      - run: cargo build -p hypergraph-rs-cli --release
```

The `xgi-conformance` job is the primary gate. It runs the 50 ported test
files + the property-based differential test. If it passes, the port is
conformant.

---

## 10. Build, release, and implementation phasing

### 10.1 Build system

**Rust workspace** (`hypergraph-rs/Cargo.toml`):
```toml
[workspace]
members = [
    "crates/hypergraph-rs",
    "crates/hypergraph-rs-python",
    "crates/hypergraph-rs-wasm",
    "crates/hypergraph-rs-cli",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.79"  # match rustworkx-core MSRV
license = "BSD-3-Clause"
authors = ["Babylon Project"]

[workspace.dependencies]
rustworkx-core = "0.18"
petgraph = "0.6"        # re-exported by rustworkx-core, pinned to match
indexmap = "2"
ndarray = "0.16"
sprs = "0.11"           # sparse matrices (incidence, adjacency, overlap)
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rand = "0.8"
rand_pcg = "0.3"        # seeded RNG for deterministic generators
rayon = "1"             # parallelism (matches rustworkx-core)
```

**Feature flags** (`crates/hypergraph-rs/Cargo.toml`):
```toml
[features]
default = ["algorithms", "generators", "stats", "readwrite", "layout", "viz"]
all = ["default", "dynamics", "communities", "convert", "network"]
algorithms = []
generators = []
stats = []
readwrite = []
layout = []
viz = []
dynamics = []
communities = []
convert = []
network = ["reqwest"]  # xgi_data fetches from network — off by default
```

This lets the WASM build exclude `dynamics`, `communities`, `network` to hit
the size budget, while the CLI build uses `all`.

### 10.2 Three build targets, one core

| Surface | Build command | Output | Distributes via |
|---|---|---|---|
| **Rust core** | `cargo build -p hypergraph-rs` | `libhypergraph_rs.rlib` | crates.io (`hypergraph-rs`) |
| **Python ext** | `maturin develop` (or `maturin build --release`) | `hypergraph_rs._core.so` + `.pyi` | PyPI (`hypergraph-rs`) |
| **WASM** | `wasm-pack build crates/hypergraph-rs-wasm --target web --release` | `hypergraph_rs_wasm.js` + `.wasm` + `.d.ts` | npm (`hypergraph-rs-wasm`) |
| **CLI** | `cargo build -p hypergraph-rs-cli --release` | `hypergraph-rs` binary | GitHub Releases + `cargo install` |

### 10.3 Implementation phasing — the full-port-then-swap roadmap

The user chose "Full port first, then swap." Phased breakdown:

#### Phase 0: Workspace bootstrap (1-2 days)
- Create `/hypergraph-rs/` workspace with 4 empty crates
- `Cargo.toml` workspace + per-crate manifests with feature flags
- CI skeleton (the matrix from §9.7, with placeholder tests)
- LICENSE (BSD-3-Clause), README.md scaffold, CHANGELOG.md
- Verify `cargo build` on all 4 crates passes

#### Phase 1: Core data structure + minimal API (1-2 weeks)
- `Hypergraph<N, E, M>` struct (bipartite `StableDiGraph`)
- `add_node`, `add_edge`, `remove_node`, `remove_edge`, `memberships`,
  `members`, `num_nodes`, `num_edges`
- `agent_ids` / `hyperedge_ids` IndexMap bimaps
- Insertion-ordered iteration (the III.7 parity contract)
- Property-based differential test harness (PyO3 to real XGI as oracle)
- **Conformance:** XGI's `tests/core/test_hypergraph.py` passes

#### Phase 2: DiHypergraph + SimplicialComplex + views (1 week)
- `DiHypergraph` (directed membership edges)
- `SimplicialComplex` (hypergraph + face lattice cache)
- `NodeView`, `EdgeView` proxy objects (Rust side; PyO3 side in Phase 7)
- **Error-variant hardening (deferred from Phase 1 review):** `remove_node_from_edge`'s missing-edge/not-in-edge branches currently reuse `NodeError::NotFound` with synthetic ids (brief-specified, behavior-correct, Display-garbled). Design the proper membership-error family (dedicated `MembershipError` or new variants) once, for all membership ops, BEFORE the Phase 7 binding ossifies the synthetic strings into Python-visible messages.
- **Conformance:** `test_dihypergraph.py`, `test_simplicialcomplex.py`,
  `test_views.py`, `test_globalviews.py` pass

#### Phase 3: Linear algebra + algorithms (1-2 weeks)
- `linalg/`: `incidence_matrix` (sparse + dense), `adjacency_matrix`, overlap
  matrix — all derived from the bipartite graph via `sprs`
- `algorithms/`: centrality, clustering, shortest paths, connected components,
  assortativity — delegate to `rustworkx-core` on the 1-skeleton or bipartite
  graph
- **Conformance:** `tests/linalg/`, `tests/algorithms/` pass

#### Phase 4: Generators + stats + readwrite (1-2 weeks)
- `generators/`: random, classic, lattice, uniform, simplicial — seeded `rand_pcg`
- `stats/`: nodestats, edgestats, diedgestats, dinodestats
- `readwrite/`: JSON (serde), edgelist, HIF, incidence_matrix I/O, BIGG
- **Conformance:** `tests/generators/`, `tests/stats/`, `tests/readwrite/`
  pass (with the generator-test split from §9.4)

#### Phase 5: Layout + viz (1 week)
- `layout/`: 4 strategies, delegating to `rustworkx-core` layouts where possible
- `viz/`: `SceneGraph` struct, `render()` method, `render_to_svg()`,
  `render_to_json()`
- **Conformance:** `tests/drawing/test_layout.py` passes (position
  assertions); drawing tests replaced with scene-graph tests

#### Phase 6: Secondary modules (3-5 days)
- `convert/`: to/from rustworkx, bipartite, dict, edgelist
- `dynamics/`: SIR, SIS epidemic models
- `communities/`: spectral clustering
- `utils/`: utilities
- **Conformance:** `tests/convert/`, `tests/dynamics/`,
  `tests/communities/`, `tests/utils/` pass

#### Phase 7: Python bindings (1-2 weeks) — **the conformance gate**
- PyO3 bindings for the full 164-symbol surface
- Maturin build configuration, `pyproject.toml`
- `hypergraph_rs/__init__.py` mirroring XGI's exactly
- `.pyi` stubs (auto-generated from PyO3 signatures)
- Matplotlib shim (`drawing` submodule, `mpl` feature flag)
- **Gate:** All 50 XGI test files pass with `import hypergraph_rs as xgi`.
  The conformance gate is met.

#### Phase 8: WASM bindings (1 week)
- `wasm-bindgen` wrappers
- `serde-wasm-bindgen` for attribute passing
- TypeScript types auto-generated by `wasm-pack`
- `wasm-pack test` headless tests
- Size budget verified (<500KB core gzipped, <1.5MB all algorithms)

#### Phase 9: CLI (3-5 days)
- `clap`-derive command surface (8 subcommands from §8.1)
- Smoke tests for each command
- Pre-built binaries via `cargo-dist` on GitHub Releases

#### Phase 10: Optional `@hypergraph-rs/react` package (2-3 days)
- Pure-TypeScript React components consuming a `SceneGraph`
- Storybook for component development
- npm publish

#### Phase 11: Babylon swap (separate effort, post-port)
- Deferred ADR083 written
- `s/^import xgi$/import hypergraph_rs as xgi/` across 7 Babylon files
- Babylon `pyproject.toml`: remove `xgi`, add `hypergraph-rs` path dep
- Babylon `mise run qa:regression` byte-identical (the secondary gate)
- Semgrep rule banning `import xgi` (the permanent-migration enforcement)
- XGI removed from Babylon's dependency tree

### 10.4 Estimated timeline

| Phase | Duration | Cumulative |
|---|---|---|
| 0: Bootstrap | 1-2 days | 2 days |
| 1: Core data structure | 1-2 weeks | 2.5 weeks |
| 2: DiHypergraph + Simplicial + views | 1 week | 3.5 weeks |
| 3: linalg + algorithms | 1-2 weeks | 5.5 weeks |
| 4: generators + stats + readwrite | 1-2 weeks | 7.5 weeks |
| 5: layout + viz | 1 week | 8.5 weeks |
| 6: convert + dynamics + communities + utils | 3-5 days | 9.5 weeks |
| 7: Python bindings (the conformance gate) | 1-2 weeks | 11 weeks |
| 8: WASM | 1 week | 12 weeks |
| 9: CLI | 3-5 days | 13 weeks |
| 10: React package | 2-3 days | 13.5 weeks |
| **Total to conformance gate** (Phase 7) | **~11 weeks** | |
| **Total to all three surfaces shipping** (Phase 9) | **~13 weeks** | |
| 11: Babylon swap | separate effort | |

Solo-dev estimates with TDD discipline (the conformance tests are written
alongside the implementation, red→green). Parallelizing with subagents could
compress Phases 3-6 (the algorithm ports are largely independent).

### 10.5 Risks and mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| Generator RNG divergence (XGI uses numpy.random, we use rand_pcg) | High | Property-based tests for structural invariants; recorded fixtures for exact-membership tests (§9.4) |
| Petgraph insertion order not matching XGI's dict order under churn | Medium | Property-based differential test (ADR052 pattern) catches any drift; the IndexMap bimaps enforce insertion order independently |
| WASM size budget blown by algorithm code | Medium | Feature flags exclude unused modules; `opt-level = "s"` + `lto = true`; size CI check |
| PyO3 binding overhead for 164 symbols | Low | Maturin auto-generates stubs; binding crate is thin (delegates to Rust core) |
| rustworkx-core missing algorithms we need | Low | We can re-implement missing algorithms in `hypergraph-rs::algorithms` directly on petgraph; rustworkx-core is open to upstream contributions |
| Babylon's qa:regression shifts on swap | Low (ADR052 precedent) | The conformance gate (XGI tests passing) makes this unlikely; if it happens, regenerate baselines with written proof (ADR052 pattern) |

### 10.6 Definition of done

**For hypergraph-rs v0.1.0 (the standalone library):**
1. All 50 XGI test files pass with `import hypergraph_rs as xgi`
2. The property-based differential test passes (proptest vs real XGI)
3. All 4 crates build on Linux, macOS, Windows
4. Python wheels for cp310-cp313 on all three OSes
5. WASM module <1.5MB gzipped with all features, <500KB core
6. CLI binary smoke-tested on all three OSes
7. Documentation: README, crate docs, migration guide (XGI → hypergraph-rs)

**For the Babylon swap (separate effort, v0.2.0):**
1. ADR083 written and accepted
2. 7 Babylon files swapped (`import xgi` → `import hypergraph_rs as xgi`)
3. `mise run qa:regression` byte-identical (no baseline regeneration, or
   regenerations with written proof)
4. Semgrep rule banning `import xgi` active
5. `xgi` removed from `pyproject.toml`

---

## 11. Constitutional framing & Amendment D

### 11.1 Why this is standalone, not the v2 reimplementation

Article II.7 (Edges vs Hyperedges) is in `[TRANSITION STATE — Pending
Amendment D]` (CONSTITUTION.md:245, also I.18, II.3, VIII.9, IX.3 Transition
State Protocol). The Constitution's own protocol states: *"If a principle is
marked `[TRANSITION STATE]`, the agent MUST treat it as blocked. It MAY
propose a spec to resolve the transition state, but it MUST NOT implement
code that depends on the unresolved principle."*

`hypergraph-rs` is therefore built as a **standalone, decoupled library** —
XGI semantics in Rust, as a rustworkx-core plugin. It does NOT touch
Babylon's hyperedge code. Babylon may adopt it later via a separate spec/ADR
when Amendment D is resolved. The library is a portable computational
rendering package, not an ideological package.

### 11.2 Why the swap is a continuation of Amendment L, not a new amendment

When Babylon does adopt `hypergraph-rs` (Phase 11, deferred), the swap is
framed as a 1-for-1 substrate swap of the existing v1 XGI usage — analogous
to ADR052 (NetworkX→rustworkx). Amendment L already says the dual-graph
commitment is "rustworkx+XGI" — swapping XGI for a rustworkx-plugin
hypergraph lib is a continuation of that, NOT the v2 reimplementation.
Byte-identical behavior is the gate. No new amendment needed for the swap
itself; ADR083 codifies the swap, not new constitutional principles.

### 11.3 What WOULD require Amendment D

If, after adopting `hypergraph-rs`, Babylon wants to evolve its hyperedge
usage BEYOND a 1-for-1 swap of v1 XGI semantics — e.g., the "v2
reimplementation" that reconciles the rustworkx+hypergraph dual-graph with
the strictly-dyadic morphism constraint (II.9) while preserving Anti-Pattern
VIII.9 — THAT would require drafting and ratifying Amendment D. The
`hypergraph-rs` library is a tool that COULD be used to implement Amendment
D's reconciliation, but the library itself does not depend on the amendment.

### 11.4 License compatibility

`hypergraph-rs` is BSD-3-Clause (matches XGI, the source we're porting).
Babylon is AGPL-v3. rustworkx-core is Apache-2.0. Compatibility:

- BSD-3-Clause → AGPL-v3: ✅ compatible (BSD-3 is permissive, works as a
  dependency under AGPL)
- BSD-3-Clause → Apache-2.0: ✅ compatible
- Apache-2.0 (rustworkx-core) → BSD-3-Clause (our crate): ✅ compatible

Owner directive: "babylon is AGPL-v3 so BSD should work fine with it" —
confirmed correct.

---

## 12. Open questions / future work (out of scope for v0.1.0)

- **Performance benchmarking vs XGI**: ADR052 produced a benchmark table
  (`tools/benchmarks/graph_backend_bench.py`) showing 2.7-5x speedups. We
  should produce the equivalent for hypergraph-rs vs XGI. Deferred to
  post-conformance.
- **`xgi_data` network integration**: fetching datasets from the XGI data
  repository. Gated behind the `network` feature flag; not a v0.1.0 blocker.
- **GPU acceleration**: trivially parallelizable algorithms (matrix ops,
  centrality) could use `wgpu` in the WASM target. Future work.
- **Babylon-specific generic typing**: `Hypergraph<BabylonAgent,
  CommunityState, MembershipRole>` — a typed wrapper that Babylon can build
  on top of the dynamic `serde_json::Value` defaults. Future work, post-swap.
- **Amendment D reconciliation**: using `hypergraph-rs` to implement the v2
  hyperedge semantics that resolve the II.7 transition state. Future
  constitutional work, separate from this library.
