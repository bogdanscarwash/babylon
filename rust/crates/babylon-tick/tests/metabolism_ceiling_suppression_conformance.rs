//! The AT-THE-CEILING conformance suite for `metabolism/biocapacity-update`
//! — proves the `(>= current max-cap)` regeneration-suppression guard fires
//! at the EXACT boundary `current == max-cap`, not just "close to it" (F3
//! fix round, adversarial review of PR #501).
//!
//! # Why this suite exists
//!
//! `metabolism_conformance.rs::zero_extraction_leaves_hysteresis_completely_inert`
//! and its sibling `heavy_extraction_at_the_ceiling_floors_biocapacity_at_exactly_zero`
//! both use `zero-floor-county` (`biocapacity == max_biocapacity == 100`,
//! `entropy_factor` at the production default `1.2`), whose discriminating
//! case floors BOTH the correct branch's result (`-20`) AND the wrong
//! branch's result (`-18`, if regeneration wrongly fired) to the identical
//! `0.0` — the `max(0.0, ...)` clamp downstream masks the guard's boundary
//! entirely. This scenario is built specifically so the two branches land
//! on DIFFERENT sides of the floor.
//!
//! # Provenance
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/metabolism_ceiling_suppression_conformance.py
//! ```
//!
//! Its output on 2026-08-11, verbatim:
//!
//! ```text
//! entropy_factor = 1.005, hysteresis_rate = 0.01
//! post-tick state:
//!   at-ceiling-county    biocapacity=0.0 max_biocapacity=9.9
//! ```

use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str =
    include_str!("../content/scenarios/metabolism-ceiling-suppression-conformance.bscn");
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

/// At `current == max_cap` exactly, regeneration is suppressed
/// (`regeneration = 0.0`), so `current + delta = 10 - 10.05 = -0.05`,
/// which floors to exactly `0.0`. Mutation-verified (by hand during
/// authoring, reverted before commit): flipping `metabolism.bsl`'s
/// `regeneration` binding's guard from `(>= current max-cap)` to
/// `(> current max-cap)` — so regeneration WRONGLY fires at this exact
/// boundary — flips this test's result to `4.949999999999999` (the guard
/// no longer suppresses the `regeneration_rate * max_cap = 5` term, which
/// survives the floor untouched).
#[test]
fn regeneration_is_suppressed_exactly_at_the_ceiling_boundary() {
    let graph = run();
    assert_eq!(attribute(&graph, 0, "territory/biocapacity"), 0.0);
}

/// The hysteresis ratchet is unaffected by this guard — `max_biocapacity`
/// ends at `9.9` regardless of which branch of the regeneration guard
/// fires, since `damage` never depends on `regeneration`.
#[test]
fn the_ceiling_still_ratchets_regardless_of_the_regeneration_guard() {
    let graph = run();
    assert_eq!(attribute(&graph, 0, "territory/max-biocapacity"), 9.9);
}

/// Byte-determinism.
#[test]
fn the_ceiling_suppression_scenario_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after);
    assert_eq!(a.fired, 1);
}
