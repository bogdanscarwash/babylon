//! Phase D Task 11 — measure, and claim only what the measurement shows.
//!
//! **Not part of the gate.** Wall-clock assertions in CI are determinism
//! poison (the standing rule — the same reason mutation testing stays
//! local-only). `#[ignore]`d so `cargo test` never runs it; run explicitly:
//!
//! ```text
//! cargo test -p babylon-graph --release --test storage_benchmark -- --ignored --nocapture
//! ```
//!
//! **No committed scenario at production scale exists yet** (Phase 2/3
//! content work is still ahead) — `two-classes.bscn` and
//! `vitality-conformance.bscn` mint a handful of nodes each. The sizes below
//! are therefore illustrative, chosen to span a hundredfold range with
//! hyperedge/member cardinalities in the shape a county-to-organization or
//! class-to-sector relation would plausibly have (2-20 members), not
//! measured against real content — recorded as a limitation, not hidden.
//!
//! The one mechanism this train can point to: `MemoryGraph::hyperedges_of`
//! scans every hyperedge and calls `contains` on each member list (a
//! full-store linear scan); `HypergraphStore::hyperedges_of` reads the
//! library's `memberships(node)` (proportional to that NODE's own
//! membership count) and intersects with the type index. That predicts an
//! asymptotic win for `hyperedges_of` as hyperedge count grows, and predicts
//! nothing about the other operations — both directions stay open
//! hypotheses until measured (the identity map costs two `String`
//! conversions per crossing; the type index costs a second lookup).

use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::{Direction, GraphSubstrate, NodeId};
use std::time::{Duration, Instant};

/// Build a store with `hyperedge_count` hyperedges of `members_per_edge`
/// members each, drawn from a pool of `hyperedge_count * 3` nodes, plus a
/// dyadic edge from each node to the next (a light connectivity graph so
/// `edges`/`neighbors` have something to measure too).
fn build<G: GraphSubstrate + CanonicalState, F: Fn() -> G>(
    make: F,
    hyperedge_count: usize,
    members_per_edge: usize,
) -> (G, Vec<NodeId>) {
    // Copilot review, PR #494 (D6-2): `node_count - 1` and
    // `node_count - members_per_edge` below underflow (usize) when
    // hyperedge_count == 0 or members_per_edge >= node_count. Unreachable
    // at the three committed call sites (20/200/2_000, all with
    // members_per_edge=5), but a guard costs nothing and makes the helper
    // safe to reuse with different arguments later.
    assert!(hyperedge_count > 0, "build() needs at least one hyperedge");
    assert!(
        members_per_edge < hyperedge_count * 3,
        "members_per_edge must be less than the node pool (hyperedge_count * 3)"
    );
    let mut graph = make();
    let node_count = hyperedge_count * 3;
    let mut nodes = Vec::with_capacity(node_count);
    for i in 0..node_count {
        let ty = if i % 2 == 0 {
            "social_class"
        } else {
            "territory"
        };
        nodes.push(graph.add_node(ty).unwrap());
    }
    for i in 0..(node_count - 1) {
        graph
            .add_edge("adjacency", nodes[i], nodes[i + 1], 1.0)
            .unwrap();
    }
    for e in 0..hyperedge_count {
        let start = (e * members_per_edge) % (node_count - members_per_edge);
        let members: Vec<NodeId> = (0..members_per_edge).map(|m| nodes[start + m]).collect();
        graph.add_hyperedge("economic_sector", &members).unwrap();
    }
    (graph, nodes)
}

fn time_it<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let result = f();
    (result, start.elapsed())
}

fn measure_size(hyperedge_count: usize, members_per_edge: usize) {
    println!("\n=== hyperedge_count={hyperedge_count} members_per_edge={members_per_edge} ===");

    let ((memory, memory_nodes), t) =
        time_it(|| build(MemoryGraph::new, hyperedge_count, members_per_edge));
    println!("MemoryGraph::build              {t:?}");
    let ((hyper, hyper_nodes), t) =
        time_it(|| build(HypergraphStore::new, hyperedge_count, members_per_edge));
    println!("HypergraphStore::build          {t:?}");
    let probe_node = memory_nodes[memory_nodes.len() / 2];
    assert_eq!(probe_node, hyper_nodes[hyper_nodes.len() / 2]);

    let (_, t) = time_it(|| memory.hyperedges_of(probe_node, "economic_sector").unwrap());
    println!("MemoryGraph::hyperedges_of      {t:?}");
    let (_, t) = time_it(|| hyper.hyperedges_of(probe_node, "economic_sector").unwrap());
    println!("HypergraphStore::hyperedges_of  {t:?}");

    let some_edge = memory.hyperedges_of(probe_node, "economic_sector").unwrap()[0];
    let (_, t) = time_it(|| memory.members_of(some_edge).unwrap());
    println!("MemoryGraph::members_of         {t:?}");
    let (_, t) = time_it(|| hyper.members_of(some_edge).unwrap());
    println!("HypergraphStore::members_of     {t:?}");

    let (_, t) = time_it(|| memory.nodes("social_class"));
    println!("MemoryGraph::nodes              {t:?}");
    let (_, t) = time_it(|| hyper.nodes("social_class"));
    println!("HypergraphStore::nodes          {t:?}");

    let (_, t) = time_it(|| memory.edges("adjacency"));
    println!("MemoryGraph::edges              {t:?}");
    let (_, t) = time_it(|| hyper.edges("adjacency"));
    println!("HypergraphStore::edges          {t:?}");

    let (_, t) = time_it(|| {
        memory
            .neighbors(probe_node, "adjacency", Direction::Any)
            .unwrap()
    });
    println!("MemoryGraph::neighbors          {t:?}");
    let (_, t) = time_it(|| {
        hyper
            .neighbors(probe_node, "adjacency", Direction::Any)
            .unwrap()
    });
    println!("HypergraphStore::neighbors      {t:?}");

    let (_, t) = time_it(|| memory.encode_state().unwrap());
    println!("MemoryGraph::encode_state       {t:?}");
    let (_, t) = time_it(|| hyper.encode_state().unwrap());
    println!("HypergraphStore::encode_state   {t:?}");
}

#[test]
#[ignore = "wall-clock measurement, not a gate — run explicitly, see module docs"]
fn measure_across_a_hundredfold_range() {
    measure_size(20, 5); // small
    measure_size(200, 5); // 10x
    measure_size(2_000, 5); // 100x
}

/// D4 (PR #494 adversarial review): the original three points (`20/200/2_000`)
/// undersold the shape of the curve. Extended to `5_000/10_000/20_000` — a
/// 1000x range from the smallest point — to make the complexity CLASS
/// visible rather than merely its sign at one scale. See ADR193 and
/// `docs/reference/graph-storage-capability-delta.md` for the reading:
/// `encode_state` and `members_of` are QUADRATIC in hyperedge count on
/// `HypergraphStore`, not merely "worse and super-linear" — n doubling
/// roughly quadruples the time, matching the root cause
/// (`percy-raskova/hypergraph-rs`'s own `members()`/`memberships()`
/// resolving each `petgraph::NodeIndex` by a linear scan of the whole id
/// bimap, per member, per hyperedge).
#[test]
#[ignore = "wall-clock measurement, not a gate — run explicitly, see module docs"]
fn measure_the_quadratic_cliff() {
    measure_size(2_000, 5);
    measure_size(5_000, 5);
    measure_size(10_000, 5);
    measure_size(20_000, 5);
}
