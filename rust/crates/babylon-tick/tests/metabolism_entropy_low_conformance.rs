//! One half of the mutation-verification pair for D-1's scaled-`Int`
//! workaround (`metabolism.bsl`'s own D-1;
//! `reports/metabolism-port-assessment-2026-08-11.md` §3):
//! `entropy_factor` NEAR its declared floor (`1.01`, domain `(1.0, 3.0]`).
//! See `metabolism_entropy_high_conformance.rs` for the companion at the
//! declared cap — the SAME territory, differing only in `entropy_factor`.
//!
//! # Provenance
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/metabolism_entropy_low_conformance.py
//! ```
//!
//! Its output on 2026-08-11, verbatim:
//!
//! ```text
//! entropy_factor = 1.01
//! post-tick state:
//!   low-entropy-county   biocapacity=1.9000000000000004 max_biocapacity=99.95
//! ```

use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str = include_str!("../content/scenarios/metabolism-entropy-low-conformance.bscn");
const RULE: &str = include_str!("../content/rules/metabolism.bsl");

fn run() -> MemoryGraph {
    let mut graph = MemoryGraph::new();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink).expect("the Metabolism pack must run");
    graph
}

fn attribute(graph: &MemoryGraph, id: u64, field: &str) -> f64 {
    graph
        .node_attribute(NodeId(id), field)
        .unwrap_or_else(|e| panic!("node {id} field {field}: {}", e.message))
}

/// At `entropy_factor = 1.01`: `raw_extraction = 1*10 = 10`,
/// `ecological_cost = 10 * 1.01 = 10.1`, `delta = 2 - 10.1 = -8.1`,
/// `current + delta = 10 - 8.1 = 1.9` — the floor does NOT bind (contrast
/// `metabolism_entropy_high_conformance.rs`, the IDENTICAL territory at
/// `entropy_factor = 3.0`, where it does). Exact IEEE-754 float, not a
/// rounded `1.9`.
/// Mutation-verified: changing `metabolism.bsl`'s D-1 descaling divisor
/// from `1000000` to `100000` (a 10x scale bug — the effective
/// `entropy_factor` becomes `10.1` instead of `1.01`) flips this test's
/// `biocapacity` from `1.9000000000000004` to `0.0` — verified by hand
/// during authoring, reverted before commit.
#[test]
fn a_low_entropy_factor_leaves_the_floor_inert() {
    let graph = run();
    assert_eq!(
        attribute(&graph, 0, "territory/biocapacity"),
        1.900_000_000_000_000_4
    );
}

/// `damage = raw_extraction * hysteresis_rate = 10 * 0.005 = 0.05`,
/// unaffected by `entropy_factor` — `max_biocapacity = 100 - 0.05 = 99.95`,
/// identical to the high-entropy companion's own `max_biocapacity` (proving
/// the D-1 workaround affects ONLY the ecological-cost term, not the
/// hysteresis damage, exactly as the frozen formulas keep the two
/// independent).
#[test]
fn the_hysteresis_damage_is_unaffected_by_entropy_factor() {
    let graph = run();
    assert_eq!(attribute(&graph, 0, "territory/max-biocapacity"), 99.95);
}

/// Byte-determinism.
#[test]
fn the_low_entropy_scenario_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after);
    assert_eq!(a.fired, 1);
}
