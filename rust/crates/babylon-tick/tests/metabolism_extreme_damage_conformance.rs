//! The EXTREME-DAMAGE conformance suite for `metabolism/biocapacity-update`
//! — proves the `new-max` floor clamp (`max(0.0, max_cap - damage)`) is
//! reachable and mutation-catchable (F4 fix round, adversarial review of
//! PR #501: this clamp was never exercised by any prior fixture — deleting
//! it left all 16 tests green).
//!
//! # Provenance
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/metabolism_extreme_damage_conformance.py
//! ```
//!
//! Its output on 2026-08-11, verbatim:
//!
//! ```text
//! post-tick state:
//!   extreme-county   biocapacity=0.0 max_biocapacity=0.0
//! ```
//!
//! See `metabolism-extreme-damage-conformance.bscn` for why
//! `damage > max_biocapacity` is legal and reachable, and why this clamp's
//! mutation signal lives entirely in `max_biocapacity` (`biocapacity`
//! floors to `0.0` regardless, for the independent reason that the
//! ecological cost alone drives `current + delta` far more negative than
//! `max_cap - damage`).

use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str =
    include_str!("../content/scenarios/metabolism-extreme-damage-conformance.bscn");
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

/// `max_cap - damage = 100 - 500 = -400`; the `max(0.0, ...)` floor must
/// clamp `new-max` to EXACTLY `0.0`, not leave it negative. Mutation-
/// verified (by hand during authoring, reverted before commit): deleting
/// `metabolism.bsl`'s `new-max` floor clamp (using `max-cap-minus-damage`
/// directly instead of the `if`-guarded floored version) flips this test
/// from `0.0` to `-400.0`.
#[test]
fn the_new_max_floor_clamp_binds_at_exactly_zero() {
    let graph = run();
    assert_eq!(attribute(&graph, 0, "territory/max-biocapacity"), 0.0);
}

/// `biocapacity` also floors to `0.0`, but for the INDEPENDENT reason that
/// the ecological cost alone (`raw_extraction * entropy_factor =
/// 100000 * 1.2 = 120000`) drives `current + delta` far below zero — this
/// does NOT discriminate the `new-max` floor clamp (see the module doc);
/// asserted here only for completeness of the post-tick state.
#[test]
fn biocapacity_also_floors_at_exactly_zero() {
    let graph = run();
    assert_eq!(attribute(&graph, 0, "territory/biocapacity"), 0.0);
}

/// Byte-determinism.
#[test]
fn the_extreme_damage_scenario_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after);
    assert_eq!(a.fired, 1);
}
