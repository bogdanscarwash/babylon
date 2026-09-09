# hypergraph-rs Phase 0+1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bootstrap the `hypergraph-rs` Cargo workspace with 4 crates and implement the core `Hypergraph<N, E, M>` data structure (bipartite `StableDiGraph`) with full node/edge CRUD, membership queries, attribute access, and insertion-ordered iteration — the foundation that Phase 2 (DiHypergraph, views, conformance testing) builds on.

**Architecture:** The hypergraph IS a `rustworkx_core::petgraph::stable_graph::StableDiGraph` with two node kinds (`Agent` and `Hyperedge`) connected by `MembershipEdge` edges. This bipartite representation makes it a genuine rustworkx-core plugin. `IndexMap` bimaps provide O(1) id-based lookup while preserving insertion order (III.7 determinism parity). **Spec alignment:** the spec originally named `petgraph::graph::DiGraph` while describing `StableGraph` hole-leaving behavior (`Graph::remove_node` swap-compacts, which breaks insertion order under churn); the spec was corrected in the same PR that landed this plan — both documents now specify `StableDiGraph`.

**Tech Stack:** Rust 1.85+ (rustworkx-core 0.18's declared MSRV — verified from crates.io metadata 2026-07-18; local toolchain 1.91.1), `rustworkx-core` 0.18 (re-exports `petgraph` 0.8 — there is NO direct petgraph dependency; `rustworkx_core::petgraph` is the only petgraph path, a direct `petgraph = "0.6"` pin was empirically shown to pull a second type-incompatible copy), `indexmap` 2, `serde` + `serde_json`, `thiserror` 2. BSD-3-Clause license.

## Global Constraints

- **Rust edition 2021**, MSRV 1.85 (rustworkx-core 0.18 declares `rust-version = "1.85"`; an edition-2021 crate can depend on its edition-2024 code)
- **License:** BSD-3-Clause (every crate manifest declares it)
- **Workspace location:** `/hypergraph-rs/` at the babylon repo root (external dep; NOT inside `src/`)
- **Determinism (III.7 parity):** All iteration is insertion-ordered. `StableDiGraph` preserves insertion order for nodes and edges even under removal (leaves holes, doesn't compact). `IndexMap` bimaps enforce id-lookup order independently. No `HashMap` anywhere in the public iteration path.
- **No `unsafe` code** in the core crate.
- **TDD discipline:** Every task follows red → green → commit. Write the failing test first, verify it fails, implement minimal code to pass, verify it passes, commit.
- **Commit convention:** Conventional commits (`type(scope): desc`). End with `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` trailer.
- **Generic type defaults:** `N = serde_json::Value, E = serde_json::Value, M = serde_json::Value` — XGI-style dynamic attrs by default.
- **XGI API fidelity** (confirmed from `.venv/lib/python3.13/site-packages/xgi/core/hypergraph.py`):
  - `add_edge` auto-creates member nodes that don't exist (hypergraph.py:620-623)
  - `add_edge` with `idx=None` auto-generates integer IDs via a counter (hypergraph.py:617)
  - `add_edge` with duplicate `idx` warns and returns (no-op) (hypergraph.py:613-615)
  - `add_edge` deduplicates members via `set(members)` (hypergraph.py:611)
  - `remove_node` has weak (default) and strong modes (hypergraph.py:435-478)
  - `H.nodes.memberships(n)` returns a `set` of edge IDs (views.py:604-630); missing ids raise `IDNotFound` (a `KeyError` subclass, NOT an `XGIError` subclass)
  - `H.edges.members(e)` returns a `set` of node IDs when `e` is given (views.py:706-747; the no-arg form returns a list/dict of sets)
- **Deliberate deviations from XGI** (each verified against the installed 0.10.2 source in the 2026-07-18 preflight; the Phase 7 Python binding shims where noted):
  - Duplicate `idx` on `add_edge`: XGI warns and silently no-ops (hypergraph.py:613-615); we return `Err(EdgeError::AlreadyExists)`. Binding shims to warn+no-op for conformance.
  - Empty members on `add_edge`: XGI's docstring claims `XGIError` but the code has NO check — it silently creates an empty edge (verified empirically); we return `Err(EdgeError::EmptyMembers)` — deliberate strictness.
  - Member iteration order: XGI stores members in plain hash-ordered `set`s — order varies with `PYTHONHASHSEED` (verified empirically); our substrate is insertion-ordered. A determinism IMPROVEMENT, not parity. Conformance tests must compare memberships as sets, never as ordered lists.
  - XGI's `update_uid_counter` call is guarded by `if idx:` (hypergraph.py:630) — a falsy explicit id (`0`, `""`) skips the counter update, so the next auto-id `add_edge` collides and silently loses the edge. We implement the correct semantics (any numeric id updates the counter, including `"0"`), fixing the upstream bug.
  - `clear()` / `clear_edges()`: XGI does NOT reset the edge-uid counter (hypergraph.py:1231-1255 never touch `_edge_uid`) — neither do we.
  - `copy()` of a frozen hypergraph returns an UNFROZEN copy in XGI (`copy()` builds a fresh instance; freeze monkeypatches only the original) — we match that.

---

## File Structure

```
hypergraph-rs/                                    # workspace root (repo top-level)
├── Cargo.toml                                    # workspace manifest
├── LICENSE                                       # BSD-3-Clause
├── README.md
├── .gitignore
├── crates/
│   ├── hypergraph-rs/                            # core library (this plan's focus)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                            # crate root, re-exports
│   │   │   └── core/
│   │   │       ├── mod.rs
│   │   │       ├── hypergraph.rs                 # Hypergraph<N,E,M> struct + impl
│   │   │       ├── kinds.rs                      # NodeKind<N,E>, MembershipEdge<M>
│   │   │       └── error.rs                      # NodeError, EdgeError
│   │   └── tests/
│   │       └── test_hypergraph.rs                # integration tests (TDD) — MUST be a
│   │                                             #   direct child of the CRATE's tests/
│   │                                             #   (cargo discovers targets only there;
│   │                                             #   a workspace-root tests/ belongs to no
│   │                                             #   package and is never compiled)
│   ├── hypergraph-rs-python/                     # PyO3 bindings (stub in Phase 0)
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── hypergraph-rs-wasm/                       # WASM bindings (stub in Phase 0)
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── hypergraph-rs-cli/                        # CLI binary (stub in Phase 0)
│       ├── Cargo.toml
│       └── src/main.rs
└── (workspace-root tests/ arrives in Phase 7 — Python conftest.py + ported/
     XGI pytest files; it is NOT a cargo test location)
```

**Responsibility split:**
- `core/kinds.rs` — `NodeKind<N,E>` enum and `MembershipEdge<M>` struct. Pure data.
- `core/hypergraph.rs` — `Hypergraph<N,E,M>` struct and all methods. The bipartite substrate + IndexMap bimaps + all CRUD/query/iteration logic.
- `core/error.rs` — error types for missing nodes/edges.
- `lib.rs` — crate root, re-exports `core::*`.

---

## Phase 0: Workspace Bootstrap

### Task 1: Create workspace root with LICENSE, README, .gitignore

**Files:**
- Create: `hypergraph-rs/LICENSE`
- Create: `hypergraph-rs/README.md`
- Create: `hypergraph-rs/.gitignore`

- [ ] **Step 1: Create the LICENSE file (BSD-3-Clause)**

Write to `hypergraph-rs/LICENSE`:

```
BSD 3-Clause License

Copyright (c) 2026, Babylon Project

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its
   contributors may be used to endorse or promote products derived from
   this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```

- [ ] **Step 2: Create the .gitignore**

Write to `hypergraph-rs/.gitignore`:

```
/target
**/*.rs.bk
```

Note: `Cargo.lock` is deliberately **committed** (not ignored) — the workspace
contains a binary crate (`hypergraph-rs-cli`) and cdylibs, and committing the
lockfile pins the exact `rustworkx-core`/`petgraph`/`indexmap` resolution
(reproducible builds; the III.7 determinism culture applies to the toolchain
too).

- [ ] **Step 3: Create the README.md**

Write to `hypergraph-rs/README.md`:

```markdown
# hypergraph-rs

A Rust port of [XGI](https://github.com/xgi-org/xgi) (the Python hypergraph library), built as a genuine [rustworkx-core](https://github.com/Qiskit/rustworkx/tree/main/rustworkx-core) plugin.

## License

BSD-3-Clause (same as XGI, compatible with rustworkx-core's Apache-2.0).

## Status

Work in progress — see `docs/superpowers/specs/2026-07-18-hypergraph-rs-design.md` for the design spec.
```

- [ ] **Step 4: Commit**

```bash
cd hypergraph-rs
git add LICENSE README.md .gitignore
git commit -m "docs(hypergraph-rs): bootstrap workspace root with LICENSE, README, .gitignore

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 2: Create workspace Cargo.toml and 4 crate skeletons

**Files:**
- Create: `hypergraph-rs/Cargo.toml`
- Create: `hypergraph-rs/crates/hypergraph-rs/Cargo.toml`
- Create: `hypergraph-rs/crates/hypergraph-rs/src/lib.rs`
- Create: `hypergraph-rs/crates/hypergraph-rs/src/core/mod.rs`
- Create: `hypergraph-rs/crates/hypergraph-rs/src/core/kinds.rs`
- Create: `hypergraph-rs/crates/hypergraph-rs/src/core/error.rs`
- Create: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`
- Create: `hypergraph-rs/crates/hypergraph-rs-python/Cargo.toml`
- Create: `hypergraph-rs/crates/hypergraph-rs-python/src/lib.rs`
- Create: `hypergraph-rs/crates/hypergraph-rs-wasm/Cargo.toml`
- Create: `hypergraph-rs/crates/hypergraph-rs-wasm/src/lib.rs`
- Create: `hypergraph-rs/crates/hypergraph-rs-cli/Cargo.toml`
- Create: `hypergraph-rs/crates/hypergraph-rs-cli/src/main.rs`

- [ ] **Step 1: Create the workspace Cargo.toml**

Write to `hypergraph-rs/Cargo.toml`:

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
rust-version = "1.85"
license = "BSD-3-Clause"
authors = ["Babylon Project"]
repository = "https://github.com/percy-raskova/babylon"

[workspace.dependencies]
# NO direct petgraph dependency: rustworkx-core 0.18 re-exports petgraph 0.8;
# all petgraph types are reached via rustworkx_core::petgraph. A direct pin
# (the spec's original petgraph = "0.6") resolves to a SECOND type-incompatible
# petgraph copy (verified empirically via cargo tree, 2026-07-18 preflight).
rustworkx-core = "0.18"
indexmap = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
rand = "0.8"
rand_pcg = "0.3"
rayon = "1"
```

- [ ] **Step 2: Create the core crate Cargo.toml**

Write to `hypergraph-rs/crates/hypergraph-rs/Cargo.toml`:

```toml
[package]
name = "hypergraph-rs"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "A Rust port of XGI — hypergraph library built on rustworkx-core"

[dependencies]
rustworkx-core.workspace = true
indexmap.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
```

- [ ] **Step 3: Create the core crate src/lib.rs**

Write to `hypergraph-rs/crates/hypergraph-rs/src/lib.rs`:

```rust
//! # hypergraph-rs
//!
//! A Rust port of XGI (the Python hypergraph library), built as a genuine
//! rustworkx-core plugin. The hypergraph IS a
//! [`rustworkx_core::petgraph::stable_graph::StableDiGraph`] with two node
//! kinds (`Agent` and `Hyperedge`) connected by `MembershipEdge` edges.

pub mod core;

pub use core::hypergraph::Hypergraph;
pub use core::kinds::{MembershipEdge, NodeKind};
pub use core::error::{EdgeError, NodeError};
```

- [ ] **Step 4: Create the core module files**

Write to `hypergraph-rs/crates/hypergraph-rs/src/core/mod.rs`:

```rust
//! Core hypergraph data structures.

pub mod error;
pub mod hypergraph;
pub mod kinds;
```

Write to `hypergraph-rs/crates/hypergraph-rs/src/core/kinds.rs`:

```rust
//! The two node kinds and membership edge type for the bipartite representation.

use serde::{Deserialize, Serialize};

/// The two roles in the bipartite structure.
///
/// `Agent` nodes are the hypergraph's nodes (entities). `Hyperedge` nodes
/// are the hypergraph's edges — their members are exactly their `Agent`
/// neighbors in the bipartite graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeKind<N, E> {
    /// An entity node (XGI "node"). Carries its own attributes.
    Agent(N),
    /// A hyperedge node (XGI "edge"). The members of the hyperedge are
    /// exactly the Agent neighbors.
    Hyperedge(E),
}

/// A membership edge in the bipartite graph: Agent <-> Hyperedge.
///
/// Carries per-membership attributes (XGI's edge-member attrs).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MembershipEdge<M> {
    /// The per-membership data (e.g., role, strength, visibility).
    pub member_data: M,
}
```

Write to `hypergraph-rs/crates/hypergraph-rs/src/core/error.rs`:

```rust
//! Error types for hypergraph operations.

use thiserror::Error;

/// Error raised when a node operation fails.
#[derive(Debug, Clone, Error)]
pub enum NodeError {
    /// The node ID was not found in the hypergraph.
    #[error("node {node_id} does not exist")]
    NotFound { node_id: String },
}

/// Error raised when an edge operation fails.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum EdgeError {
    /// The edge ID was not found in the hypergraph.
    #[error("edge {edge_id} does not exist")]
    NotFound { edge_id: String },
    /// The edge ID already exists (duplicate on add_edge).
    #[error("edge {edge_id} already exists")]
    AlreadyExists { edge_id: String },
    /// The members collection was empty.
    #[error("cannot add an empty edge")]
    EmptyMembers,
}
```

Write to `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`:

```rust
//! The core `Hypergraph` data structure.

use rustworkx_core::petgraph::stable_graph::{NodeIndex, StableDiGraph};
use indexmap::IndexMap;

use super::error::{EdgeError, NodeError};
use super::kinds::{MembershipEdge, NodeKind};

/// A hypergraph, represented as a bipartite graph.
///
/// The bipartite graph has two node kinds: `Agent` (the hypergraph's nodes)
/// and `Hyperedge` (the hypergraph's edges). Membership edges connect agents
/// to their hyperedges. This representation makes the hypergraph a genuine
/// rustworkx-core plugin — all petgraph/rustworkx-core algorithms work on
/// the bipartite graph directly.
///
/// # Type Parameters
///
/// - `N` — agent node attribute type (defaults to `serde_json::Value`)
/// - `E` — hyperedge attribute type (defaults to `serde_json::Value`)
/// - `M` — per-membership attribute type (defaults to `serde_json::Value`)
pub struct Hypergraph<N = serde_json::Value, E = serde_json::Value, M = serde_json::Value> {
    /// The bipartite graph: Agent nodes + Hyperedge nodes + membership edges.
    /// StableDiGraph preserves insertion order under removal (leaves holes,
    /// doesn't compact) — required for III.7 determinism parity.
    inner: StableDiGraph<NodeKind<N, E>, MembershipEdge<M>>,

    /// Insertion-ordered bimap: agent_id -> NodeIndex in `inner`.
    agent_ids: IndexMap<String, NodeIndex>,

    /// Insertion-ordered bimap: edge_id -> NodeIndex in `inner`.
    hyperedge_ids: IndexMap<String, NodeIndex>,

    /// Auto-incrementing counter for edges added without an explicit `idx`.
    edge_uid_counter: u64,

    /// Graph-level attributes (XGI's `H.graph` dict).
    graph_attrs: serde_json::Map<String, serde_json::Value>,
}

impl<N, E, M> Default for Hypergraph<N, E, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N, E, M> Hypergraph<N, E, M> {
    /// Create an empty hypergraph.
    pub fn new() -> Self {
        Self {
            inner: StableDiGraph::new(),
            agent_ids: IndexMap::new(),
            hyperedge_ids: IndexMap::new(),
            edge_uid_counter: 0,
            graph_attrs: serde_json::Map::new(),
        }
    }

    /// The number of agent nodes in the hypergraph.
    pub fn num_nodes(&self) -> usize {
        self.agent_ids.len()
    }

    /// The number of hyperedges in the hypergraph.
    pub fn num_edges(&self) -> usize {
        self.hyperedge_ids.len()
    }
}
```

- [ ] **Step 5: Create the 3 binding crate skeletons**

Write to `hypergraph-rs/crates/hypergraph-rs-python/Cargo.toml`:

```toml
[package]
name = "hypergraph-rs-python"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "PyO3 bindings for hypergraph-rs"

[lib]
name = "_hypergraph_rs_core"
crate-type = ["cdylib"]

[dependencies]
hypergraph-rs = { path = "../hypergraph-rs" }
pyo3 = { version = "0.29", features = ["extension-module"] }
```

Note: pyo3 0.29 is the current release line (0.22 was stale at plan time).
pyo3's build script needs a Python interpreter at build time
(`pyo3-build-config` resolves the ABI from `python3` on PATH or
`PYO3_PYTHON`) — present on this box, an explicit prerequisite for any
clean CI runner.

Write to `hypergraph-rs/crates/hypergraph-rs-python/src/lib.rs`:

```rust
//! PyO3 bindings for hypergraph-rs (stub — Phase 7 implements the full surface).

use pyo3::prelude::*;

#[pymodule]
fn _hypergraph_rs_core(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
```

Write to `hypergraph-rs/crates/hypergraph-rs-wasm/Cargo.toml`:

```toml
[package]
name = "hypergraph-rs-wasm"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "WASM bindings for hypergraph-rs"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
hypergraph-rs = { path = "../hypergraph-rs" }
wasm-bindgen = "0.2"
```

Write to `hypergraph-rs/crates/hypergraph-rs-wasm/src/lib.rs`:

```rust
//! WASM bindings for hypergraph-rs (stub — Phase 8 implements the full surface).

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn _init() {
    // Stub — Phase 8 will implement the full WASM surface.
}
```

The `use wasm_bindgen::prelude::*;` import is MANDATORY — without it
`#[wasm_bindgen]` is an unresolved attribute and the whole-workspace
`cargo build` in Step 6 fails.

Write to `hypergraph-rs/crates/hypergraph-rs-cli/Cargo.toml`:

```toml
[package]
name = "hypergraph-rs-cli"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "CLI for hypergraph-rs"

[[bin]]
name = "hypergraph-rs"
path = "src/main.rs"

[dependencies]
hypergraph-rs = { path = "../hypergraph-rs" }
clap = { version = "4", features = ["derive"] }
```

Write to `hypergraph-rs/crates/hypergraph-rs-cli/src/main.rs`:

```rust
// CLI for hypergraph-rs (stub — Phase 9 implements the full command surface).

fn main() {
    println!("hypergraph-rs CLI (stub — Phase 9 will implement commands)");
}
```

- [ ] **Step 6: Verify the workspace builds**

Run: `cd hypergraph-rs && cargo build`
Expected: Builds successfully. Warnings OK (unused imports in stubs).

- [ ] **Step 7: Verify the core crate tests pass (0 tests, compiles)**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs`
Expected: PASS (0 tests, 0 failures — no tests defined yet).

- [ ] **Step 8: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): bootstrap workspace with 4 crates + core data structure stub

Workspace: hypergraph-rs (core), hypergraph-rs-python (PyO3 stub),
hypergraph-rs-wasm (wasm-bindgen stub), hypergraph-rs-cli (clap stub).

Core crate: Hypergraph<N,E,M> struct with StableDiGraph bipartite substrate
+ IndexMap bimaps. NodeKind<N,E> and MembershipEdge<M> types. Error types
for NodeError and EdgeError.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Phase 1: Core Data Structure

### Task 3: Test and implement `add_node` + `has_node`

**Files:**
- Create: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

**Interfaces:**
- Produces: `Hypergraph::add_node(&mut self, node_id: &str, attrs: N) -> bool` — returns `true` if new, `false` if existing. `Hypergraph::has_node(&self, node_id: &str) -> bool`.

- [ ] **Step 1: Write the failing test**

Write to `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
use hypergraph_rs::Hypergraph;

#[test]
fn test_add_node_creates_new_node() {
    let mut h: Hypergraph = Hypergraph::new();
    let created = h.add_node("a", serde_json::Value::Null);
    assert!(created);
    assert_eq!(h.num_nodes(), 1);
    assert!(h.has_node("a"));
}

#[test]
fn test_add_node_returns_false_for_existing() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_node("a", serde_json::Value::Null);
    let created = h.add_node("a", serde_json::Value::Null);
    assert!(!created);
    assert_eq!(h.num_nodes(), 1);
}

#[test]
fn test_has_node_returns_false_for_missing() {
    let h: Hypergraph = Hypergraph::new();
    assert!(!h.has_node("nonexistent"));
}

#[test]
fn test_num_nodes_starts_at_zero() {
    let h: Hypergraph = Hypergraph::new();
    assert_eq!(h.num_nodes(), 0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — `add_node` and `has_node` don't exist. Compile error.

- [ ] **Step 3: Implement `add_node` and `has_node`**

Add inside the `impl<N, E, M> Hypergraph<N, E, M>` block in `hypergraph.rs`, after `num_edges`:

```rust
    /// Add a node with attributes. Returns `true` if a new node was created,
    /// `false` if it already existed.
    ///
    /// XGI parity: `H.add_node(node, **attr)`.
    pub fn add_node(&mut self, node_id: &str, attrs: N) -> bool {
        if self.agent_ids.contains_key(node_id) {
            return false;
        }
        let idx = self.inner.add_node(NodeKind::Agent(attrs));
        self.agent_ids.insert(node_id.to_string(), idx);
        true
    }

    /// Check if a node exists in the hypergraph.
    ///
    /// XGI parity: `n in H` / `H.has_node(n)`.
    pub fn has_node(&self, node_id: &str) -> bool {
        self.agent_ids.contains_key(node_id)
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — 4 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement add_node + has_node

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 4: Test and implement `add_edge` with auto-node-creation and auto-id

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

**Interfaces:**
- Produces: `Hypergraph::add_edge(&mut self, members: Vec<String>, idx: Option<String>, attrs: E) -> Result<String, EdgeError>` where `N: Default, M: Default + Clone`. Auto-creates member nodes. Deduplicates members. Returns `EdgeError::AlreadyExists` for duplicate `idx`, `EdgeError::EmptyMembers` for empty members.

- [ ] **Step 1: Write the failing test**

Append to `crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
use hypergraph_rs::EdgeError;

#[test]
fn test_add_edge_with_explicit_idx() {
    let mut h: Hypergraph = Hypergraph::new();
    let edge_id = h.add_edge(
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
        Some("myedge".to_string()),
        serde_json::Value::Null,
    ).unwrap();
    assert_eq!(edge_id, "myedge");
    assert_eq!(h.num_edges(), 1);
    assert_eq!(h.num_nodes(), 3);
}

#[test]
fn test_add_edge_auto_generates_id() {
    let mut h: Hypergraph = Hypergraph::new();
    let id1 = h.add_edge(vec!["a".to_string(), "b".to_string()], None, serde_json::Value::Null).unwrap();
    assert_eq!(id1, "0");
    let id2 = h.add_edge(vec!["c".to_string(), "d".to_string()], None, serde_json::Value::Null).unwrap();
    assert_eq!(id2, "1");
}

#[test]
fn test_add_edge_duplicate_idx_returns_error() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    let result = h.add_edge(vec!["b".to_string()], Some("e1".to_string()), serde_json::Value::Null);
    assert!(matches!(result, Err(EdgeError::AlreadyExists { .. })));
    assert_eq!(h.num_edges(), 1);
}

#[test]
fn test_add_edge_deduplicates_members() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string(), "a".to_string(), "b".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    assert_eq!(h.num_nodes(), 2);
}

#[test]
fn test_add_edge_empty_members_returns_error() {
    let mut h: Hypergraph = Hypergraph::new();
    let result = h.add_edge(vec![], Some("e1".to_string()), serde_json::Value::Null);
    assert!(matches!(result, Err(EdgeError::EmptyMembers)));
}

#[test]
fn test_add_edge_auto_id_after_explicit_idx() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string()], Some("5".to_string()), serde_json::Value::Null).unwrap();
    let next = h.add_edge(vec!["b".to_string()], None, serde_json::Value::Null).unwrap();
    assert_eq!(next, "6");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — `add_edge` doesn't exist.

- [ ] **Step 3: Implement `add_edge`**

Add inside the impl block:

```rust
    /// Add a hyperedge connecting the given members.
    ///
    /// XGI parity: `H.add_edge(members, idx=id, **attr)`.
    pub fn add_edge(
        &mut self,
        members: Vec<String>,
        idx: Option<String>,
        attrs: E,
    ) -> Result<String, EdgeError>
    where
        N: Default,
        M: Default + Clone,
    {
        if members.is_empty() {
            return Err(EdgeError::EmptyMembers);
        }

        let edge_id = match &idx {
            Some(id) => {
                if self.hyperedge_ids.contains_key(id) {
                    return Err(EdgeError::AlreadyExists { edge_id: id.clone() });
                }
                id.clone()
            }
            None => self.edge_uid_counter.to_string(),
        };

        if let Some(id) = &idx {
            if let Ok(n) = id.parse::<u64>() {
                if n >= self.edge_uid_counter {
                    self.edge_uid_counter = n + 1;
                }
            }
        } else {
            self.edge_uid_counter += 1;
        }

        let mut seen = std::collections::HashSet::new();
        let unique_members: Vec<String> = members
            .into_iter()
            .filter(|m| seen.insert(m.clone()))
            .collect();

        for member in &unique_members {
            if !self.agent_ids.contains_key(member) {
                let nidx = self.inner.add_node(NodeKind::Agent(N::default()));
                self.agent_ids.insert(member.clone(), nidx);
            }
        }

        let he_idx = self.inner.add_node(NodeKind::Hyperedge(attrs));
        self.hyperedge_ids.insert(edge_id.clone(), he_idx);

        for member in &unique_members {
            let agent_idx = self.agent_ids[member];
            let membership = MembershipEdge { member_data: M::default() };
            self.inner.add_edge(agent_idx, he_idx, membership.clone());
            self.inner.add_edge(he_idx, agent_idx, membership);
        }

        Ok(edge_id)
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — all 10 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement add_edge with auto-node-creation and auto-id

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 5: Test and implement `has_edge`

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
#[test]
fn test_has_edge_returns_true_for_existing() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string(), "b".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    assert!(h.has_edge("e1"));
}

#[test]
fn test_has_edge_returns_false_for_missing() {
    let h: Hypergraph = Hypergraph::new();
    assert!(!h.has_edge("nonexistent"));
}

#[test]
fn test_num_edges_counts_correctly() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    h.add_edge(vec!["b".to_string()], Some("e2".to_string()), serde_json::Value::Null).unwrap();
    h.add_edge(vec!["c".to_string()], Some("e3".to_string()), serde_json::Value::Null).unwrap();
    assert_eq!(h.num_edges(), 3);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — `has_edge` doesn't exist.

- [ ] **Step 3: Implement `has_edge`**

Add inside the impl block:

```rust
    /// Check if a hyperedge exists.
    /// XGI parity: `id in H.edges`.
    pub fn has_edge(&self, edge_id: &str) -> bool {
        self.hyperedge_ids.contains_key(edge_id)
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — all 13 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement has_edge

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 6: Test and implement `memberships` + `members` queries

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

**Interfaces:**
- Produces: `Hypergraph::memberships(&self, node_id: &str) -> Option<Vec<String>>`, `Hypergraph::members(&self, edge_id: &str) -> Option<Vec<String>>`.

- [ ] **Step 1: Write the failing test**

Append to `crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
#[test]
fn test_memberships_returns_edge_ids() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string(), "b".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    h.add_edge(vec!["a".to_string(), "c".to_string()], Some("e2".to_string()), serde_json::Value::Null).unwrap();
    let mships = h.memberships("a").unwrap();
    assert_eq!(mships.len(), 2);
    assert!(mships.contains(&"e1".to_string()));
    assert!(mships.contains(&"e2".to_string()));
}

#[test]
fn test_memberships_returns_empty_for_isolate() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_node("lonely", serde_json::Value::Null);
    assert!(h.memberships("lonely").unwrap().is_empty());
}

#[test]
fn test_memberships_returns_none_for_missing() {
    let h: Hypergraph = Hypergraph::new();
    assert!(h.memberships("nonexistent").is_none());
}

#[test]
fn test_members_returns_node_ids() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string(), "b".to_string(), "c".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    let members = h.members("e1").unwrap();
    assert_eq!(members.len(), 3);
    assert!(members.contains(&"a".to_string()));
}

#[test]
fn test_members_returns_none_for_missing() {
    let h: Hypergraph = Hypergraph::new();
    assert!(h.members("nonexistent").is_none());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — `memberships` and `members` don't exist.

- [ ] **Step 3: Implement `memberships` and `members`**

Add inside the impl block:

```rust
    /// Get the edge IDs of which a node is a member.
    /// XGI parity: `H.nodes.memberships(n)`.
    pub fn memberships(&self, node_id: &str) -> Option<Vec<String>> {
        let agent_idx = *self.agent_ids.get(node_id)?;
        let mut result: Vec<String> = Vec::new();
        for neighbor_idx in self.inner.neighbors(agent_idx) {
            if let Some(NodeKind::Hyperedge(_)) = self.inner.node_weight(neighbor_idx) {
                for (eid, &idx) in &self.hyperedge_ids {
                    if idx == neighbor_idx {
                        result.push(eid.clone());
                        break;
                    }
                }
            }
        }
        Some(result)
    }

    /// Get the node IDs that are members of an edge.
    /// XGI parity: `H.edges.members(e)`.
    pub fn members(&self, edge_id: &str) -> Option<Vec<String>> {
        let he_idx = *self.hyperedge_ids.get(edge_id)?;
        let mut result: Vec<String> = Vec::new();
        for neighbor_idx in self.inner.neighbors(he_idx) {
            if let Some(NodeKind::Agent(_)) = self.inner.node_weight(neighbor_idx) {
                for (nid, &idx) in &self.agent_ids {
                    if idx == neighbor_idx {
                        result.push(nid.clone());
                        break;
                    }
                }
            }
        }
        Some(result)
    }
```

Note: The linear search through bimaps is O(n) per neighbor — a placeholder. A later optimization adds a reverse bimap for O(1) lookup. Babylon's scale (~1000 nodes / ~50 edges) makes this acceptable for now.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — all 18 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement memberships + members queries

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 7: Test and implement insertion-ordered `node_ids` + `edge_ids`

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
#[test]
fn test_node_ids_insertion_order() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_node("c", serde_json::Value::Null);
    h.add_node("a", serde_json::Value::Null);
    h.add_node("b", serde_json::Value::Null);
    assert_eq!(h.node_ids(), vec!["c", "a", "b"]);
}

#[test]
fn test_edge_ids_insertion_order() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["x".to_string()], Some("e3".to_string()), serde_json::Value::Null).unwrap();
    h.add_edge(vec!["x".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    h.add_edge(vec!["x".to_string()], Some("e2".to_string()), serde_json::Value::Null).unwrap();
    assert_eq!(h.edge_ids(), vec!["e3", "e1", "e2"]);
}

#[test]
fn test_node_ids_empty() {
    let h: Hypergraph = Hypergraph::new();
    assert!(h.node_ids().is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — `node_ids` and `edge_ids` don't exist.

- [ ] **Step 3: Implement `node_ids` and `edge_ids`**

Add inside the impl block:

```rust
    /// All node IDs in insertion order (III.7 determinism parity).
    /// XGI parity: `list(H.nodes)`.
    pub fn node_ids(&self) -> Vec<String> {
        self.agent_ids.keys().cloned().collect()
    }

    /// All edge IDs in insertion order (III.7 determinism parity).
    /// XGI parity: `list(H.edges)`.
    pub fn edge_ids(&self) -> Vec<String> {
        self.hyperedge_ids.keys().cloned().collect()
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — all 21 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement insertion-ordered node_ids + edge_ids

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 8: Test and implement `remove_node` (weak + strong modes)

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

**Interfaces:**
- Produces: `Hypergraph::remove_node(&mut self, node_id: &str, strong: bool) -> Result<(), NodeError>`.

- [ ] **Step 1: Write the failing test**

Append to `crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
use hypergraph_rs::NodeError;

#[test]
fn test_remove_node_weak_removes_from_edges() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string(), "b".to_string(), "c".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    h.remove_node("b", false).unwrap();
    assert!(!h.has_node("b"));
    assert!(h.has_edge("e1"));
    let members = h.members("e1").unwrap();
    assert!(!members.contains(&"b".to_string()));
    assert!(members.contains(&"a".to_string()));
}

#[test]
fn test_remove_node_weak_removes_singleton_edges() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    h.remove_node("a", false).unwrap();
    assert!(!h.has_node("a"));
    assert!(!h.has_edge("e1"));
}

#[test]
fn test_remove_node_strong_removes_all_containing_edges() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string(), "b".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    h.add_edge(vec!["a".to_string(), "c".to_string(), "d".to_string()], Some("e2".to_string()), serde_json::Value::Null).unwrap();
    h.remove_node("a", true).unwrap();
    assert!(!h.has_node("a"));
    assert!(!h.has_edge("e1"));
    assert!(!h.has_edge("e2"));
    assert!(h.has_node("b"));
    assert!(h.has_node("c"));
}

#[test]
fn test_remove_node_missing_returns_error() {
    let mut h: Hypergraph = Hypergraph::new();
    let result = h.remove_node("nonexistent", false);
    assert!(matches!(result, Err(NodeError::NotFound { .. })));
}

#[test]
fn test_remove_node_preserves_insertion_order() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_node("a", serde_json::Value::Null);
    h.add_node("b", serde_json::Value::Null);
    h.add_node("c", serde_json::Value::Null);
    h.remove_node("b", false).unwrap();
    assert_eq!(h.node_ids(), vec!["a", "c"]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — `remove_node` doesn't exist.

- [ ] **Step 3: Implement `remove_node`**

Add inside the impl block:

```rust
    /// Remove a node from the hypergraph.
    /// XGI parity: `H.remove_node(n, strong=False, remove_empty=True)`.
    pub fn remove_node(&mut self, node_id: &str, strong: bool) -> Result<(), NodeError> {
        let agent_idx = *self.agent_ids.get(node_id).ok_or(NodeError::NotFound {
            node_id: node_id.to_string(),
        })?;

        let edge_ids: Vec<String> = self.memberships(node_id).unwrap_or_default();

        if strong {
            for eid in edge_ids {
                self.remove_edge(&eid).map_err(|_| NodeError::NotFound {
                    node_id: node_id.to_string(),
                })?;
            }
        } else {
            for eid in &edge_ids {
                let he_idx = self.hyperedge_ids[eid];
                if let Some(e) = self.inner.find_edge(agent_idx, he_idx) {
                    self.inner.remove_edge(e);
                }
                if let Some(e) = self.inner.find_edge(he_idx, agent_idx) {
                    self.inner.remove_edge(e);
                }
                let has_members = self.inner.neighbors(he_idx).any(|n| {
                    matches!(self.inner.node_weight(n), Some(NodeKind::Agent(_)))
                });
                if !has_members {
                    self.inner.remove_node(he_idx);
                    self.hyperedge_ids.shift_remove(eid);
                }
            }
        }

        self.inner.remove_node(agent_idx);
        self.agent_ids.shift_remove(node_id);
        Ok(())
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — all 26 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement remove_node with weak + strong modes

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 9: Test and implement `remove_edge`

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
#[test]
fn test_remove_edge_basic() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string(), "b".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    h.remove_edge("e1").unwrap();
    assert!(!h.has_edge("e1"));
    assert_eq!(h.num_edges(), 0);
    assert!(h.has_node("a"));
    assert!(h.has_node("b"));
    assert!(h.memberships("a").unwrap().is_empty());
}

#[test]
fn test_remove_edge_missing_returns_error() {
    let mut h: Hypergraph = Hypergraph::new();
    let result = h.remove_edge("nonexistent");
    assert!(matches!(result, Err(EdgeError::NotFound { .. })));
}

#[test]
fn test_remove_edge_preserves_other_edges() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string(), "b".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    h.add_edge(vec!["a".to_string(), "c".to_string()], Some("e2".to_string()), serde_json::Value::Null).unwrap();
    h.remove_edge("e1").unwrap();
    assert!(!h.has_edge("e1"));
    assert!(h.has_edge("e2"));
    assert_eq!(h.memberships("a").unwrap(), vec!["e2"]);
}

#[test]
fn test_remove_edge_preserves_insertion_order() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    h.add_edge(vec!["a".to_string()], Some("e2".to_string()), serde_json::Value::Null).unwrap();
    h.add_edge(vec!["a".to_string()], Some("e3".to_string()), serde_json::Value::Null).unwrap();
    h.remove_edge("e2").unwrap();
    assert_eq!(h.edge_ids(), vec!["e1", "e3"]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — `remove_edge` doesn't exist.

- [ ] **Step 3: Implement `remove_edge`**

Add inside the impl block:

```rust
    /// Remove a hyperedge from the hypergraph.
    /// XGI parity: `H.remove_edge(e)`.
    pub fn remove_edge(&mut self, edge_id: &str) -> Result<(), EdgeError> {
        let he_idx = *self.hyperedge_ids.get(edge_id).ok_or(EdgeError::NotFound {
            edge_id: edge_id.to_string(),
        })?;
        self.inner.remove_node(he_idx);
        self.hyperedge_ids.shift_remove(edge_id);
        Ok(())
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — all 30 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement remove_edge

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 10: Test and implement attribute access (`node_attrs`, `edge_attrs`, mut variants)

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

**Interfaces:**
- Produces: `node_attrs(&self, node_id: &str) -> Option<&N>`, `node_attrs_mut(&mut self, node_id: &str) -> Option<&mut N>`, `edge_attrs(&self, edge_id: &str) -> Option<&E>`, `edge_attrs_mut(&mut self, edge_id: &str) -> Option<&mut E>`.

- [ ] **Step 1: Write the failing test**

Append to `crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
#[test]
fn test_node_attrs_read() {
    let mut h: Hypergraph = Hypergraph::new();
    let attrs = serde_json::json!({"color": "red", "weight": 42});
    h.add_node("a", attrs.clone());
    let read = h.node_attrs("a").unwrap();
    assert_eq!(read, &attrs);
}

#[test]
fn test_node_attrs_mut() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_node("a", serde_json::json!({"count": 0}));
    {
        let attrs = h.node_attrs_mut("a").unwrap();
        attrs["count"] = serde_json::json!(5);
    }
    assert_eq!(h.node_attrs("a").unwrap()["count"], 5);
}

#[test]
fn test_node_attrs_missing_returns_none() {
    let h: Hypergraph = Hypergraph::new();
    assert!(h.node_attrs("nonexistent").is_none());
}

#[test]
fn test_edge_attrs_read() {
    let mut h: Hypergraph = Hypergraph::new();
    let attrs = serde_json::json!({"heat": 0.5, "cohesion": 0.8});
    h.add_edge(vec!["a".to_string()], Some("e1".to_string()), attrs.clone()).unwrap();
    assert_eq!(h.edge_attrs("e1").unwrap(), &attrs);
}

#[test]
fn test_edge_attrs_mut() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string()], Some("e1".to_string()), serde_json::json!({"heat": 0.3})).unwrap();
    {
        let attrs = h.edge_attrs_mut("e1").unwrap();
        attrs["heat"] = serde_json::json!(0.9);
    }
    assert_eq!(h.edge_attrs("e1").unwrap()["heat"], 0.9);
}

#[test]
fn test_edge_attrs_missing_returns_none() {
    let h: Hypergraph = Hypergraph::new();
    assert!(h.edge_attrs("nonexistent").is_none());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — attribute accessor methods don't exist.

- [ ] **Step 3: Implement attribute accessors**

Add inside the impl block:

```rust
    /// Read a node's attributes. XGI parity: `H.nodes[n]`.
    pub fn node_attrs(&self, node_id: &str) -> Option<&N> {
        let idx = self.agent_ids.get(node_id)?;
        match self.inner.node_weight(*idx)? {
            NodeKind::Agent(attrs) => Some(attrs),
            _ => None,
        }
    }

    /// Mutably access a node's attributes.
    pub fn node_attrs_mut(&mut self, node_id: &str) -> Option<&mut N> {
        let idx = *self.agent_ids.get(node_id)?;
        match self.inner.node_weight_mut(idx)? {
            NodeKind::Agent(attrs) => Some(attrs),
            _ => None,
        }
    }

    /// Read a hyperedge's attributes. XGI parity: `H.edges[e]`.
    pub fn edge_attrs(&self, edge_id: &str) -> Option<&E> {
        let idx = self.hyperedge_ids.get(edge_id)?;
        match self.inner.node_weight(*idx)? {
            NodeKind::Hyperedge(attrs) => Some(attrs),
            _ => None,
        }
    }

    /// Mutably access a hyperedge's attributes.
    pub fn edge_attrs_mut(&mut self, edge_id: &str) -> Option<&mut E> {
        let idx = *self.hyperedge_ids.get(edge_id)?;
        match self.inner.node_weight_mut(idx)? {
            NodeKind::Hyperedge(attrs) => Some(attrs),
            _ => None,
        }
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — all 36 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement node/edge attribute accessors

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 11: Test and implement graph-level attributes + `clear`

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
#[test]
fn test_graph_attr_set_and_get() {
    let mut h: Hypergraph = Hypergraph::new();
    h.set_graph_attr("name", serde_json::json!("wayne_county"));
    h.set_graph_attr("tick", serde_json::json!(42));
    assert_eq!(h.graph_attr("name"), Some(&serde_json::json!("wayne_county")));
    assert_eq!(h.graph_attr("tick"), Some(&serde_json::json!(42)));
}

#[test]
fn test_graph_attr_missing_returns_none() {
    let h: Hypergraph = Hypergraph::new();
    assert!(h.graph_attr("nonexistent").is_none());
}

#[test]
fn test_graph_attr_overwrite() {
    let mut h: Hypergraph = Hypergraph::new();
    h.set_graph_attr("tick", serde_json::json!(1));
    h.set_graph_attr("tick", serde_json::json!(99));
    assert_eq!(h.graph_attr("tick"), Some(&serde_json::json!(99)));
}

#[test]
fn test_clear_removes_everything() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string(), "b".to_string()], Some("e1".to_string()), serde_json::json!({"heat": 0.5})).unwrap();
    h.add_node("lonely", serde_json::json!({"x": 1}));
    h.set_graph_attr("name", serde_json::json!("test"));
    h.clear();
    assert_eq!(h.num_nodes(), 0);
    assert_eq!(h.num_edges(), 0);
    assert!(h.node_ids().is_empty());
    assert!(h.edge_ids().is_empty());
    assert!(h.graph_attr("name").is_none());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — `graph_attr`, `set_graph_attr`, `clear` don't exist.

- [ ] **Step 3: Implement graph attrs and clear**

Add inside the impl block:

```rust
    /// Read a graph-level attribute. XGI parity: `H["name"]`.
    pub fn graph_attr(&self, key: &str) -> Option<&serde_json::Value> {
        self.graph_attrs.get(key)
    }

    /// Set a graph-level attribute. XGI parity: `H["name"] = value`.
    pub fn set_graph_attr(&mut self, key: &str, value: serde_json::Value) {
        self.graph_attrs.insert(key.to_string(), value);
    }

    /// Remove all nodes and edges. XGI parity: `H.clear()`.
    ///
    /// The edge-uid counter is deliberately NOT reset — XGI's `clear()`
    /// never touches `_edge_uid` (verified 0.10.2: auto-ids continue the
    /// pre-clear sequence).
    pub fn clear(&mut self) {
        self.inner = StableDiGraph::new();
        self.agent_ids.clear();
        self.hyperedge_ids.clear();
        self.graph_attrs.clear();
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — all 40 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement graph-level attrs + clear

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 12: Test and implement `copy` (deep clone)

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
#[test]
fn test_copy_produces_independent_clone() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string(), "b".to_string()], Some("e1".to_string()), serde_json::json!({"heat": 0.5})).unwrap();
    h.set_graph_attr("name", serde_json::json!("test"));

    let mut h2 = h.copy();
    assert_eq!(h2.num_nodes(), 2);
    assert_eq!(h2.num_edges(), 1);
    assert_eq!(h2.edge_attrs("e1").unwrap()["heat"], 0.5);

    h2.add_node("c", serde_json::Value::Null);
    h2.set_graph_attr("name", serde_json::json!("modified"));
    assert_eq!(h.num_nodes(), 2);
    assert!(!h.has_node("c"));
    assert_eq!(h.graph_attr("name"), Some(&serde_json::json!("test")));
}

#[test]
fn test_copy_of_empty() {
    let h: Hypergraph = Hypergraph::new();
    let h2 = h.copy();
    assert_eq!(h2.num_nodes(), 0);
    assert_eq!(h2.num_edges(), 0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — `copy` doesn't exist.

- [ ] **Step 3: Implement `copy`**

Add inside the impl block:

```rust
    /// Return an independent deep copy. XGI parity: `H.copy()`.
    pub fn copy(&self) -> Self
    where
        N: Clone,
        E: Clone,
        M: Clone,
    {
        Self {
            inner: self.inner.clone(),
            agent_ids: self.agent_ids.clone(),
            hyperedge_ids: self.hyperedge_ids.clone(),
            edge_uid_counter: self.edge_uid_counter,
            graph_attrs: self.graph_attrs.clone(),
        }
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — all 42 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement copy (deep clone)

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 13: Test and implement bulk operations

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
#[test]
fn test_add_nodes_from_bulk() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_nodes_from(vec![
        ("a".to_string(), serde_json::json!({"x": 1})),
        ("b".to_string(), serde_json::json!({"x": 2})),
    ]);
    assert_eq!(h.num_nodes(), 2);
    assert_eq!(h.node_attrs("a").unwrap()["x"], 1);
}

#[test]
fn test_add_edges_from_bulk() {
    let mut h: Hypergraph = Hypergraph::new();
    let results = h.add_edges_from(vec![
        (vec!["a".to_string(), "b".to_string()], Some("e1".to_string()), serde_json::json!({"w": 1})),
        (vec!["b".to_string(), "c".to_string()], None, serde_json::Value::Null),
    ]);
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.is_ok()));
    assert_eq!(h.num_edges(), 2);
    assert_eq!(h.num_nodes(), 3);
    assert_eq!(h.edge_ids(), vec!["e1", "0"]);
}

#[test]
fn test_add_edges_from_with_duplicate_idx() {
    let mut h: Hypergraph = Hypergraph::new();
    let results = h.add_edges_from(vec![
        (vec!["a".to_string()], Some("e1".to_string()), serde_json::Value::Null),
        (vec!["b".to_string()], Some("e1".to_string()), serde_json::Value::Null),
    ]);
    assert!(results[0].is_ok());
    assert!(results[1].is_err());
    assert_eq!(h.num_edges(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — bulk methods don't exist.

- [ ] **Step 3: Implement bulk operations**

Add inside the impl block:

```rust
    /// Add multiple nodes. XGI parity: `H.add_nodes_from(nodes_for_adding)`.
    pub fn add_nodes_from(&mut self, nodes: impl IntoIterator<Item = (String, N)>) {
        for (node_id, attrs) in nodes {
            self.add_node(&node_id, attrs);
        }
    }

    /// Add multiple edges. Returns a result per edge.
    pub fn add_edges_from(
        &mut self,
        edges: impl IntoIterator<Item = (Vec<String>, Option<String>, E)>,
    ) -> Vec<Result<String, EdgeError>>
    where
        N: Default,
        M: Default + Clone,
    {
        edges.into_iter().map(|(m, i, a)| self.add_edge(m, i, a)).collect()
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — all 45 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement add_nodes_from + add_edges_from

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 14: Test and implement `PartialEq` (structural equality)

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
#[test]
fn test_eq_same_structure() {
    let mut h1: Hypergraph = Hypergraph::new();
    h1.add_edge(vec!["a".to_string(), "b".to_string()], Some("e1".to_string()), serde_json::json!({"w": 1})).unwrap();
    let mut h2: Hypergraph = Hypergraph::new();
    h2.add_edge(vec!["a".to_string(), "b".to_string()], Some("e1".to_string()), serde_json::json!({"w": 1})).unwrap();
    assert!(h1 == h2);
}

#[test]
fn test_eq_different_edge_attrs() {
    let mut h1: Hypergraph = Hypergraph::new();
    h1.add_edge(vec!["a".to_string()], Some("e1".to_string()), serde_json::json!({"w": 1})).unwrap();
    let mut h2: Hypergraph = Hypergraph::new();
    h2.add_edge(vec!["a".to_string()], Some("e1".to_string()), serde_json::json!({"w": 2})).unwrap();
    assert!(h1 != h2);
}

#[test]
fn test_eq_different_members() {
    let mut h1: Hypergraph = Hypergraph::new();
    h1.add_edge(vec!["a".to_string(), "b".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    let mut h2: Hypergraph = Hypergraph::new();
    h2.add_edge(vec!["a".to_string(), "c".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    assert!(h1 != h2);
}

#[test]
fn test_eq_both_empty() {
    let h1: Hypergraph = Hypergraph::new();
    let h2: Hypergraph = Hypergraph::new();
    assert!(h1 == h2);
}
```

Note: these tests use `assert!(h1 == h2)` / `assert!(h1 != h2)` rather than
`assert_eq!`/`assert_ne!` — those macros require `Debug` on the operands
(their failure branch formats with `{:?}`, type-checked even on pass), and
`Hypergraph`'s `Debug` impl doesn't exist until Task 18.

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — `PartialEq` not implemented.

- [ ] **Step 3: Implement `PartialEq`**

Add OUTSIDE the inherent impl block (trait impl):

```rust
impl<N: PartialEq, E: PartialEq, M: PartialEq> PartialEq for Hypergraph<N, E, M> {
    /// Two hypergraphs are equal if they have the same nodes, edges,
    /// memberships, and attributes. XGI parity: `H1 == H2`.
    fn eq(&self, other: &Self) -> bool {
        if self.agent_ids.len() != other.agent_ids.len() { return false; }
        for (nid, _) in &self.agent_ids {
            match (self.node_attrs(nid), other.node_attrs(nid)) {
                (Some(a), Some(b)) if a == b => {}
                _ => return false,
            }
        }
        if self.hyperedge_ids.len() != other.hyperedge_ids.len() { return false; }
        for (eid, _) in &self.hyperedge_ids {
            match (self.edge_attrs(eid), other.edge_attrs(eid)) {
                (Some(a), Some(b)) if a == b => {}
                _ => return false,
            }
            let mut m1 = self.members(eid).unwrap_or_default();
            let mut m2 = other.members(eid).unwrap_or_default();
            m1.sort(); m2.sort();
            if m1 != m2 { return false; }
        }
        self.graph_attrs == other.graph_attrs
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — all 49 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement PartialEq (structural equality)

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 15: Test and implement `add_node_to_edge` + `remove_node_from_edge`

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

**Interfaces:**
- Produces: `add_node_to_edge(&mut self, edge_id: &str, node_id: &str) -> Result<(), EdgeError>` where `N: Default` — adds a node to an existing edge; auto-creates both if missing. `remove_node_from_edge(&mut self, edge_id: &str, node_id: &str, remove_empty: bool) -> Result<(), NodeError>` — removes a node from an edge; optionally removes the edge if it becomes empty.

**XGI source reference** (verified 2026-07-18: `add_node_to_edge` at hypergraph.py:1080-1117, `remove_node_from_edge` at hypergraph.py:1166-1208): `add_node_to_edge` auto-creates edge and node if either doesn't exist. `remove_node_from_edge` raises XGIError if edge/node missing or node not in edge; removes empty edge only when `remove_empty=True` (the default).

- [ ] **Step 1: Write the failing test**

Append to `crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
#[test]
fn test_add_node_to_edge_existing_edge() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string(), "b".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    h.add_node_to_edge("e1", "c").unwrap();
    let members = h.members("e1").unwrap();
    assert!(members.contains(&"c".to_string()));
    assert_eq!(members.len(), 3);
}

#[test]
fn test_add_node_to_edge_auto_creates_edge_and_node() {
    let mut h: Hypergraph = Hypergraph::new();
    // Neither edge "new_edge" nor node "new_node" exist
    h.add_node_to_edge("new_edge", "new_node").unwrap();
    assert!(h.has_edge("new_edge"));
    assert!(h.has_node("new_node"));
    let members = h.members("new_edge").unwrap();
    assert_eq!(members, vec!["new_node"]);
}

#[test]
fn test_remove_node_from_edge_basic() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string(), "b".to_string(), "c".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    h.remove_node_from_edge("e1", "b", true).unwrap();
    let members = h.members("e1").unwrap();
    assert!(!members.contains(&"b".to_string()));
    assert_eq!(members.len(), 2);
    // Node b still exists (just not in e1 anymore)
    assert!(h.has_node("b"));
}

#[test]
fn test_remove_node_from_edge_removes_empty_edge() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    h.remove_node_from_edge("e1", "a", true).unwrap();
    // e1 was a singleton, now empty — should be removed
    assert!(!h.has_edge("e1"));
    assert!(h.has_node("a"));
}

#[test]
fn test_remove_node_from_edge_keep_empty() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    h.remove_node_from_edge("e1", "a", false).unwrap();
    // remove_empty=false: edge stays (now empty)
    assert!(h.has_edge("e1"));
    assert!(h.members("e1").unwrap().is_empty());
}

#[test]
fn test_remove_node_from_edge_missing_edge_returns_error() {
    let mut h: Hypergraph = Hypergraph::new();
    let result = h.remove_node_from_edge("nonexistent", "a", true);
    assert!(result.is_err());
}

#[test]
fn test_remove_node_from_edge_node_not_in_edge_returns_error() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    h.add_node("b", serde_json::Value::Null);
    let result = h.remove_node_from_edge("e1", "b", true);
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — `add_node_to_edge` and `remove_node_from_edge` don't exist.

- [ ] **Step 3: Implement both methods**

Add inside the impl block:

```rust
    /// Add a node to an existing edge. Auto-creates both if missing.
    /// XGI parity: `H.add_node_to_edge(edge, node)`.
    pub fn add_node_to_edge(&mut self, edge_id: &str, node_id: &str) -> Result<(), EdgeError>
    where
        N: Default,
        E: Default,
        M: Default + Clone,
    {
        // Auto-create edge if missing
        if !self.hyperedge_ids.contains_key(edge_id) {
            let he_idx = self.inner.add_node(NodeKind::Hyperedge(E::default()));
            self.hyperedge_ids.insert(edge_id.to_string(), he_idx);
            // Update uid counter if edge_id is numeric
            if let Ok(n) = edge_id.parse::<u64>() {
                if n >= self.edge_uid_counter {
                    self.edge_uid_counter = n + 1;
                }
            }
        }
        // Auto-create node if missing
        if !self.agent_ids.contains_key(node_id) {
            let nidx = self.inner.add_node(NodeKind::Agent(N::default()));
            self.agent_ids.insert(node_id.to_string(), nidx);
        }
        // Add the membership edges (both directions for undirected)
        let agent_idx = self.agent_ids[node_id];
        let he_idx = self.hyperedge_ids[edge_id];
        // Check if membership already exists
        if self.inner.find_edge(agent_idx, he_idx).is_none() {
            let membership = MembershipEdge { member_data: M::default() };
            self.inner.add_edge(agent_idx, he_idx, membership.clone());
            self.inner.add_edge(he_idx, agent_idx, membership);
        }
        Ok(())
    }

    /// Remove a node from an existing edge.
    /// XGI parity: `H.remove_node_from_edge(edge, node, remove_empty=True)`.
    pub fn remove_node_from_edge(
        &mut self,
        edge_id: &str,
        node_id: &str,
        remove_empty: bool,
    ) -> Result<(), NodeError> {
        let he_idx = *self.hyperedge_ids.get(edge_id).ok_or(NodeError::NotFound {
            node_id: edge_id.to_string(),
        })?;
        let agent_idx = *self.agent_ids.get(node_id).ok_or(NodeError::NotFound {
            node_id: node_id.to_string(),
        })?;

        // Check if the node is actually in the edge
        let edge_to_he = self.inner.find_edge(agent_idx, he_idx);
        let edge_from_he = self.inner.find_edge(he_idx, agent_idx);
        if edge_to_he.is_none() {
            return Err(NodeError::NotFound {
                node_id: format!("node {} not in edge {}", node_id, edge_id),
            });
        }

        // Remove the bipartite membership edges
        if let Some(e) = edge_to_he {
            self.inner.remove_edge(e);
        }
        if let Some(e) = edge_from_he {
            self.inner.remove_edge(e);
        }

        // Check if edge is now empty
        if remove_empty {
            let has_members = self.inner.neighbors(he_idx).any(|n| {
                matches!(self.inner.node_weight(n), Some(NodeKind::Agent(_)))
            });
            if !has_members {
                self.inner.remove_node(he_idx);
                self.hyperedge_ids.shift_remove(edge_id);
            }
        }
        Ok(())
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — all 56 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement add_node_to_edge + remove_node_from_edge

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 16: Test and implement `set_node_attributes` + `set_edge_attributes`

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

**Interfaces:**
- Produces: `set_node_attributes(&mut self, values: impl Iterator<Item = (String, serde_json::Map)>)` — bulk set node attrs from (id, attr_map) pairs. `set_edge_attributes` — same for edges. Skips missing IDs silently (XGI behavior with a warning).

- [ ] **Step 1: Write the failing test**

Append to `crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
#[test]
fn test_set_node_attributes_from_pairs() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_node("a", serde_json::Value::Null);
    h.add_node("b", serde_json::Value::Null);

    let mut attrs_a = serde_json::Map::new();
    attrs_a.insert("color".to_string(), serde_json::json!("red"));
    let mut attrs_b = serde_json::Map::new();
    attrs_b.insert("color".to_string(), serde_json::json!("blue"));

    h.set_node_attributes(vec![
        ("a".to_string(), attrs_a),
        ("b".to_string(), attrs_b),
    ]);

    assert_eq!(h.node_attrs("a").unwrap()["color"], "red");
    assert_eq!(h.node_attrs("b").unwrap()["color"], "blue");
}

#[test]
fn test_set_node_attributes_skips_missing() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_node("a", serde_json::Value::Null);

    let mut attrs = serde_json::Map::new();
    attrs.insert("x".to_string(), serde_json::json!(1));

    h.set_node_attributes(vec![
        ("a".to_string(), attrs.clone()),
        ("nonexistent".to_string(), attrs),
    ]);

    // "a" got the attribute, "nonexistent" was silently skipped
    assert_eq!(h.node_attrs("a").unwrap()["x"], 1);
}

#[test]
fn test_set_edge_attributes_from_pairs() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string()], Some("e1".to_string()), serde_json::Value::Null).unwrap();
    h.add_edge(vec!["b".to_string()], Some("e2".to_string()), serde_json::Value::Null).unwrap();

    let mut attrs_e1 = serde_json::Map::new();
    attrs_e1.insert("weight".to_string(), serde_json::json!(5));
    let mut attrs_e2 = serde_json::Map::new();
    attrs_e2.insert("weight".to_string(), serde_json::json!(10));

    h.set_edge_attributes(vec![
        ("e1".to_string(), attrs_e1),
        ("e2".to_string(), attrs_e2),
    ]);

    assert_eq!(h.edge_attrs("e1").unwrap()["weight"], 5);
    assert_eq!(h.edge_attrs("e2").unwrap()["weight"], 10);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — `set_node_attributes` and `set_edge_attributes` don't exist.

- [ ] **Step 3: Implement both methods**

Add a SEPARATE concrete-typed impl block AFTER the generic `impl<N, E, M>`
block (NOT inside it): these methods call `as_object_mut()`, an inherent
method of `serde_json::Value` that does not exist on bare type parameters
`N`/`E` — placing them in the generic block is a hard compile error
(`no method named as_object_mut found for type parameter N`).

```rust
impl<M> Hypergraph<serde_json::Value, serde_json::Value, M> {
    /// Set node attributes from (id, attr_map) pairs.
    /// XGI parity: `H.set_node_attributes(values, name=None)`.
    /// Silently skips missing node IDs (XGI warns).
    pub fn set_node_attributes(
        &mut self,
        values: impl IntoIterator<Item = (String, serde_json::Map<String, serde_json::Value>)>,
    ) {
        for (node_id, attrs) in values {
            if let Some(node_attrs) = self.node_attrs_mut(&node_id) {
                if let Some(obj) = node_attrs.as_object_mut() {
                    for (k, v) in attrs {
                        obj.insert(k, v);
                    }
                }
            }
            // Silently skip missing nodes (XGI warns)
        }
    }

    /// Set edge attributes from (id, attr_map) pairs.
    /// XGI parity: `H.set_edge_attributes(values, name=None)`.
    /// Silently skips missing edge IDs (XGI warns).
    pub fn set_edge_attributes(
        &mut self,
        values: impl IntoIterator<Item = (String, serde_json::Map<String, serde_json::Value>)>,
    ) {
        for (edge_id, attrs) in values {
            if let Some(edge_attrs) = self.edge_attrs_mut(&edge_id) {
                if let Some(obj) = edge_attrs.as_object_mut() {
                    for (k, v) in attrs {
                        obj.insert(k, v);
                    }
                }
            }
        }
    }
}
```

Note: these methods live in a concrete `impl<M> Hypergraph<serde_json::Value,
serde_json::Value, M>` block because `as_object_mut` is inherent to
`serde_json::Value`. The default-typed test Hypergraph (`N=E=M=Value`) picks
them up from this specialized impl; Babylon can later add typed setters for
specialized `N`/`E`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — all 59 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement set_node_attributes + set_edge_attributes

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 17: Test and implement `clear_edges` + `freeze` / `is_frozen`

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

**Interfaces:**
- Produces: `clear_edges(&mut self)` — removes all edges, keeps nodes. `freeze(&mut self)` — sets a frozen flag. `is_frozen(&self) -> bool`. When frozen, mutation methods should panic.

- [ ] **Step 1: Write the failing test**

Append to `crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
#[test]
fn test_clear_edges_keeps_nodes() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string(), "b".to_string()], Some("e1".to_string()), serde_json::json!({"w": 1})).unwrap();
    h.add_node("lonely", serde_json::json!({"x": 1}));
    h.clear_edges();
    assert_eq!(h.num_nodes(), 3); // a, b, lonely all kept
    assert_eq!(h.num_edges(), 0);
    // Node attrs preserved
    assert_eq!(h.node_attrs("lonely").unwrap()["x"], 1);
    // No memberships
    assert!(h.memberships("a").unwrap().is_empty());
}

#[test]
fn test_freeze_prevents_mutation() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_node("a", serde_json::Value::Null);
    h.freeze();
    assert!(h.is_frozen());
    // Mutation should panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut h = h;
        h.add_node("b", serde_json::Value::Null);
    }));
    assert!(result.is_err(), "add_node should panic when frozen");
}

#[test]
fn test_is_frozen_false_by_default() {
    let h: Hypergraph = Hypergraph::new();
    assert!(!h.is_frozen());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — `clear_edges`, `freeze`, `is_frozen` don't exist.

- [ ] **Step 3: Add `frozen` field and implement the methods**

First, add a `frozen` field to the `Hypergraph` struct:

```rust
pub struct Hypergraph<N = serde_json::Value, E = serde_json::Value, M = serde_json::Value> {
    inner: StableDiGraph<NodeKind<N, E>, MembershipEdge<M>>,
    agent_ids: IndexMap<String, NodeIndex>,
    hyperedge_ids: IndexMap<String, NodeIndex>,
    edge_uid_counter: u64,
    graph_attrs: serde_json::Map<String, serde_json::Value>,
    frozen: bool,  // NEW
}
```

Update `new()`:
```rust
    pub fn new() -> Self {
        Self {
            inner: StableDiGraph::new(),
            agent_ids: IndexMap::new(),
            hyperedge_ids: IndexMap::new(),
            edge_uid_counter: 0,
            graph_attrs: serde_json::Map::new(),
            frozen: false,  // NEW
        }
    }
```

`clear()` needs NO frozen change: XGI freezes `clear` too (hypergraph.py:1550
puts `clear` in the frozen-method list, so a frozen hypergraph's `clear()`
raises `XGIError`) — with the frozen guard added below, a `self.frozen = false`
inside `clear()` would be unreachable dead state-change.

Update `copy()` to initialize `frozen` (XGI parity: `copy()` of a frozen
hypergraph returns an UNFROZEN copy — XGI's `copy()` constructs a fresh
instance and `freeze()` monkeypatches only the original, verified 0.10.2):
```rust
    pub fn copy(&self) -> Self where N: Clone, E: Clone, M: Clone {
        Self {
            inner: self.inner.clone(),
            agent_ids: self.agent_ids.clone(),
            hyperedge_ids: self.hyperedge_ids.clone(),
            edge_uid_counter: self.edge_uid_counter,
            graph_attrs: self.graph_attrs.clone(),
            frozen: false,  // NEW — XGI parity: copies are always unfrozen
        }
    }
```

Add the new methods:

```rust
    /// Remove all edges, keeping nodes.
    ///
    /// XGI parity: `H.clear_edges()` — the edge-uid counter is NOT reset
    /// (XGI's `clear_edges` never touches `_edge_uid`, verified 0.10.2).
    pub fn clear_edges(&mut self) {
        let hyperedge_indices: Vec<NodeIndex> = self.hyperedge_ids.values().copied().collect();
        for he_idx in hyperedge_indices {
            self.inner.remove_node(he_idx);
        }
        self.hyperedge_ids.clear();
    }

    /// Freeze the hypergraph, preventing modification.
    /// XGI parity: `H.freeze()`.
    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    /// Check if the hypergraph is frozen.
    /// XGI parity: `H.is_frozen`.
    pub fn is_frozen(&self) -> bool {
        self.frozen
    }
```

Add a frozen guard to `add_node`, `add_edge`, `remove_node`, `remove_edge`, `add_node_to_edge`, `remove_node_from_edge`, `clear`, `clear_edges`:

```rust
    fn assert_not_frozen(&self) {
        if self.frozen {
            panic!("Frozen hypergraph can't be modified");
        }
    }
```

Call `self.assert_not_frozen();` at the top of each mutation method.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — all 62 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement clear_edges + freeze/is_frozen

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 18: Test and implement `__repr__` / `Debug` formatting

**Files:**
- Modify: `hypergraph-rs/crates/hypergraph-rs/tests/test_hypergraph.rs`
- Modify: `hypergraph-rs/crates/hypergraph-rs/src/core/hypergraph.rs`

**Interfaces:**
- Produces: `impl std::fmt::Debug for Hypergraph` — XGI v0.10.2 `__repr__` parity: `Hypergraph([{members}, ...])`.

- [ ] **Step 1: Write the failing test**

Append to `crates/hypergraph-rs/tests/test_hypergraph.rs`:

```rust
#[test]
fn test_debug_format_matches_xgi_repr() {
    let mut h: Hypergraph = Hypergraph::new();
    h.add_edge(vec!["a".to_string(), "b".to_string(), "c".to_string()], None, serde_json::Value::Null).unwrap();
    h.add_edge(vec!["b".to_string(), "c".to_string()], None, serde_json::Value::Null).unwrap();
    let repr = format!("{:?}", h);
    // XGI __repr__ returns: Hypergraph([{a, b, c}, {b, c}])
    // Our Debug should include the class name and edge members
    assert!(repr.contains("Hypergraph"));
    assert!(repr.contains("a"));
    assert!(repr.contains("b"));
    assert!(repr.contains("c"));
}

#[test]
fn test_debug_format_empty() {
    let h: Hypergraph = Hypergraph::new();
    let repr = format!("{:?}", h);
    assert!(repr.contains("Hypergraph"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: FAIL — `Debug` not implemented (the derive on the struct doesn't give XGI-parity repr).

- [ ] **Step 3: Implement `Debug`**

Add OUTSIDE the inherent impl block:

```rust
impl<N, E, M> std::fmt::Debug for Hypergraph<N, E, M>
where
    N: std::fmt::Debug,
    E: std::fmt::Debug,
    M: std::fmt::Debug,
{
    /// XGI parity: `__repr__` returns `Hypergraph([{members}, ...])`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hypergraph(")?;
        let edges: Vec<String> = self.hyperedge_ids.keys().map(|eid| {
            let members = self.members(eid).unwrap_or_default();
            format!("{{{}}}", members.join(", "))
        }).collect();
        write!(f, "[{}])", edges.join(", "))
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd hypergraph-rs && cargo test -p hypergraph-rs --test test_hypergraph`
Expected: PASS — all 64 tests, 0 failures.

- [ ] **Step 5: Commit**

```bash
cd hypergraph-rs
git add -A
git commit -m "feat(hypergraph-rs): implement Debug (XGI __repr__ parity)

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 19: Run clippy, fix warnings, verify full suite

**Files:**
- Modify: any files with clippy warnings

- [ ] **Step 1: Run clippy on the core crate**

Run: `cd hypergraph-rs && cargo clippy -p hypergraph-rs -- -D warnings`
Expected: May have warnings. Fix all of them.

- [ ] **Step 2: Fix any clippy warnings**

Common fixes: remove unused imports, use `&str` instead of `&String`, add `#[allow(clippy::xxx)]` with a comment for false positives.

If `clippy::map_entry` fires on `add_edge`/`add_node_to_edge` (the
`if !map.contains_key(k) { … map.insert(k, v) }` shape), prefer
`#[allow(clippy::map_entry)]` with a one-line justification — the inserted
value comes from a separate `self.inner.add_node(...)` `&mut self` borrow,
so the entry API would conflict — over an entry-API rewrite.

- [ ] **Step 3: Run clippy on all workspace crates**

Run: `cd hypergraph-rs && cargo clippy --workspace -- -D warnings`
Expected: PASS — no warnings.

- [ ] **Step 4: Run the full test suite**

Run: `cd hypergraph-rs && cargo test --workspace`
Expected: PASS — all 64 tests, 0 failures (the pyo3/wasm/cli stub crates each
contribute 0 tests; the "49" that stood here previously was a stale copy of
Task 14's checkpoint).

- [ ] **Step 5: Run rustfmt**

Run: `cd hypergraph-rs && cargo fmt --all`

- [ ] **Step 6: Commit formatting/clippy fixes**

```bash
cd hypergraph-rs
git add -A
git commit -m "chore(hypergraph-rs): clippy + fmt cleanup

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Phase 0+1 Completion Summary

After completing all 19 tasks, the `hypergraph-rs` workspace has:

- **4 crates**: `hypergraph-rs` (core), `hypergraph-rs-python` (PyO3 stub), `hypergraph-rs-wasm` (WASM stub), `hypergraph-rs-cli` (CLI stub)
- **Core `Hypergraph<N, E, M>` struct** (XGI v0.10.2 API parity) with:
  - `new`, `default`, `clear`, `clear_edges`, `copy`, `freeze`, `is_frozen`
  - `add_node`, `has_node`, `remove_node` (weak + strong)
  - `add_edge` (auto-node, auto-id, dedup), `has_edge`, `remove_edge`
  - `add_node_to_edge`, `remove_node_from_edge`
  - `memberships`, `members`
  - `node_attrs`/`mut`, `edge_attrs`/`mut`
  - `set_node_attributes`, `set_edge_attributes` (bulk)
  - `graph_attr`, `set_graph_attr`
  - `node_ids`, `edge_ids` (insertion-ordered)
  - `num_nodes`, `num_edges`
  - `add_nodes_from`, `add_edges_from` (bulk)
  - `PartialEq` (structural equality), `Debug` (XGI `__repr__` parity)
- **64 unit tests** all passing
- **Clippy clean**, **rustfmt clean**
- **StableDiGraph bipartite substrate** (genuine rustworkx-core plugin)
- **IndexMap bimaps** for insertion-ordered id lookup (III.7 determinism parity)
- **Frozen guard** on all mutation methods (XGI `freeze()` parity)

**XGI v0.10.2 core API coverage**: all methods in `xgi/core/hypergraph.py` are now implemented except `double_edge_swap`, `random_edge_shuffle`, `merge_duplicate_edges`, `cleanup`, `dual`, and `update` — these are deferred to Phase 2 (they're higher-level operations, not core CRUD).

**Next: Phase 2** — DiHypergraph, SimplicialComplex, NodeView/EdgeView proxy objects, and porting XGI's `test_hypergraph.py` as the first conformance gate.

---

## Post-Review Hardening (2026-07-18, executed directly by the orchestrating agent)

A whole-work review (Kimi K3, empirical — claims verified against the XGI
runtime, not its docstrings) after Task 7 found one hard conformance break
and several drifts. Fixed as the following TDD units (each red→green→commit,
ledger in `.superpowers/sdd/progress.md`):

- **H1 (D1, CRITICAL):** `add_edge([])` now CREATES an empty edge. XGI's
  docstring claims XGIError; the runtime (and XGI's own test suite) creates
  it. The plan's Task 4 spec trusted the docstring — the conformance gate
  would have failed. `EdgeError::EmptyMembers` removed. (commit 9827de1a)
- **H2 (D5):** `members()`/`memberships()` are insertion-ordered via
  bimap-filter; petgraph `neighbors()` is LIFO (verified empirically). The
  O(deg·V) reverse-lookup placeholder is gone. (commit 222e906e)
- **H3 (D6):** `add_node` on an existing node REPLACES attrs (XGI merges;
  binding merges before calling). Added `node_attrs`/`edge_attrs` readers
  (spec §4.1). `NodeError` gained `PartialEq`. (commit 559cd7fb)
- **H4:** `graph_attr`/`set_graph_attr` accessors (spec §4.1); clippy is
  warning-free. (commit 75addfd2)
- **H5:** Integration tests moved into the crate
  (`tests/core/main.rs` + mod, auto-discovered) — publishable; the manual
  `[[test]]` path entry is gone. (commit 495f1f15)
- **H6:** Conformance fixture harness — XGI-runtime ground truth as
  committed JSON + Rust replay with the executable divergence register
  (D1–D6 + D7 id-typing; spec §4.7). (commit a0d44ea3)
- **H7:** `rust-toolchain.toml` pin (1.91.1), honest MSRV 1.85 (deps declare
  it; the plan's 1.79 was false), committed Cargo.lock. (commit 25a06f07)
- **H8:** `rust:*` mise namespace + sequential `rust:check` gate +
  `ai/tooling.yaml` rust section; CLI bin renamed `hypergraph` (cargo doc
  collision, cargo#6313). MSRV leg verified locally. (commit 14522c7b)
- **H9:** `.github/workflows/rust.yml` — path-filtered gate + MSRV floor,
  invoking the same mise tasks (III.11 house rules). (commit d90904e8)

**Process amendment (the durable lesson):** parity claims are made against
the XGI RUNTIME, never its docstrings. New behavioral work on XGI-facing
surfaces adds a conformance vector FIRST (the fixture is the spec), then
implements. The divergence register (spec §4.7) is append-only.

**Remaining plan tasks are unchanged** (Task 8 `remove_node` is next; its
strong branch's `remove_edge` dependency is noted in its brief). The
register's D9 flags `remove_empty=False` as a Phase 2 addition.

---

## Phase 0+1 ACTUAL Completion Record (2026-07-18)

**19/19 tasks complete, all reviewed.** Executed via SDD (implementer +
reviewer subagents per task/batch) plus the H1–H9 post-review hardening.

- Tasks 1–7: sequential, reviews clean.
- Hardening H1–H9 (commits 9827de1a..d90904e8): the D1 empty-edge
  conformance break + everything the empirical review found; conformance
  harness, divergence register, toolchain pin, mise gate, CI.
- Tasks 9→8 (dependency swap: remove_edge before remove_node), 10+11,
  12+13, 14+15, 16+17, 18+19: batched, all reviews APPROVE/MERGE.
- Final state: **105 workspace tests** (71 core + 34 conformance replay),
  **31 XGI ground-truth fixture vectors** (regeneration byte-identical
  across hash seeds), **divergences D1–D13 registered** (spec §4.7,
  executable), `mise run rust:check` + `mise run rust:msrv` (1.85) green,
  clippy/fmt/doc `-D warnings` clean, zero `unsafe`.
- XGI probes during execution corrected the plan twice more (XGI has
  `__eq__` via algorithms.equal; XGI freeze monkey-patching artifacts →
  D12/D13) and exposed XGI's silent-overwrite footgun (D11).
- Process: every XGI-facing behavior is vector-first (the fixture is the
  spec); the register is append-only; briefs' stale paths/counts were
  handled by dispatch rulings (in-crate tests, `--test core`).
