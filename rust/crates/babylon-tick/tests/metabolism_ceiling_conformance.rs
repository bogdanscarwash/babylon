//! The CEILING-CLAMP conformance suite for `metabolism/biocapacity-update`
//! — proves the `min(new_max, current + delta)` half of the frozen
//! formula's double clamp actually binds.
//!
//! # Provenance
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/metabolism_ceiling_conformance.py
//! ```
//!
//! Its output on 2026-08-11, verbatim:
//!
//! ```text
//! post-tick state:
//!   ceiling-county biocapacity=100.0 max_biocapacity=100.0 (seed biocapacity=50.0 seed max_biocapacity=100.0)
//! ```
//!
//! See `content/scenarios/metabolism_ceiling_conformance.py`'s module
//! docstring for why `regeneration_rate` is boosted to `0.9` here: the
//! production-default `0.02` does NOT make the clamp unreachable in
//! general (a `current` within ~2% of `max_biocapacity` already exceeds it
//! even at the default), but the boost makes the clamp fire from an
//! ORDINARY mid-range stock (`50`, not `99`) with a dramatic, unambiguous
//! margin. This scenario deliberately keeps `extraction_intensity=0` so
//! the ceiling clamp is isolated with NO hysteresis interaction — the
//! ratchet is exactly zero here (`damage = 0`, `new_max = max_biocapacity`
//! unchanged). **A node exercising BOTH a strictly ratcheted ceiling
//! (`damage > 0`) AND a binding ceiling clamp against that ratcheted value
//! is proven REACHABLE** by
//! `metabolism_ratcheted_ceiling_conformance.rs` — an earlier claim here
//! that the combination was "provably unreachable with int-seeded fields"
//! was FALSE (it fixed `entropy_factor`/`hysteresis_rate` at their
//! production defaults while reasoning about reachability, missing that
//! both are ALSO per-scenario coefficients).

use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str = include_str!("../content/scenarios/metabolism-ceiling-conformance.bscn");
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

/// With `regeneration_rate=0.9` and `extraction_intensity=0`,
/// `current + delta = 50 + 90 = 140` — comfortably past
/// `max_biocapacity = 100` — and the frozen formula's
/// `max(0.0, min(new_max, current + delta))` clamps it back to EXACTLY
/// `100.0`, not `140.0` and not some intermediate value.
#[test]
fn the_ceiling_clamp_binds_and_saturates_at_exactly_the_ceiling() {
    let graph = run();
    assert_eq!(attribute(&graph, 0, "territory/biocapacity"), 100.0);
}

/// `extraction_intensity = 0` here means `raw_extraction = 0`, so the
/// hysteresis damage is exactly zero and the ceiling ITSELF is unratcheted
/// — `max_biocapacity` ends the tick bit-identical to its seed, unlike the
/// zero-floor-county vector in `metabolism_conformance.rs`, whose ceiling
/// DOES ratchet. Mutation-verified: flipping `metabolism.bsl`'s
/// `capped-at-ceiling` binding's comparator from `<` to `>` (so the clamp
/// picks the LARGER of `current + delta`/`new-max` instead of the smaller)
/// flips `the_ceiling_clamp_binds_and_saturates_at_exactly_the_ceiling`
/// (`140.0` unclamped instead of `100.0`) while leaving this test green —
/// confirmed by hand during authoring, reverted before commit.
#[test]
fn the_ceiling_is_unratcheted_when_extraction_is_zero() {
    let graph = run();
    assert_eq!(attribute(&graph, 0, "territory/max-biocapacity"), 100.0);
}

/// Byte-determinism.
#[test]
fn the_ceiling_scenario_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after);
    assert_eq!(a.fired, 1);
}
