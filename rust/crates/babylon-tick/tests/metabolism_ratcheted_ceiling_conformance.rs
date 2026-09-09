//! The RATCHETED-CEILING conformance suite for
//! `metabolism/biocapacity-update` — proves the ceiling clamp
//! (`capped-at-ceiling`) binds against the RATCHETED value (`new-max`),
//! not the original `max-biocapacity` seed (F2 fix round, adversarial
//! review of PR #501).
//!
//! # Why this suite exists
//!
//! `metabolism_ceiling_conformance.rs`'s own scenario uses
//! `extraction_intensity=0`, so its hysteresis damage is exactly zero and
//! its `new-max` binding equals the seed exactly — the ceiling clamp binds
//! there, but against an UNRATCHETED ceiling. That left the
//! `capped-at-ceiling` binding's SECOND operand
//! (`(if (< current-plus-delta new-max) current-plus-delta new-max)`)
//! mutation-dead: swapping `new-max` for `max-cap` in that `if` form
//! changed nothing observable, because in every prior scenario the two
//! values were either equal (damage zero) or the clamp never bound at all.
//!
//! # Provenance
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/metabolism_ratcheted_ceiling_conformance.py
//! ```
//!
//! Its output on 2026-08-11, verbatim:
//!
//! ```text
//! entropy_factor = 1.005, hysteresis_rate = 0.01
//! post-tick state:
//!   ratcheted-ceiling-county biocapacity=99.5 max_biocapacity=99.5
//! ```
//!
//! See `metabolism-ratcheted-ceiling-conformance.bscn` for the derivation:
//! `regeneration_rate=1.0`/`entropy_factor=1.005`/`hysteresis_rate=0.01`
//! (all legal, at their declared extremes) make `current + delta = 99.75`
//! exceed the RATCHETED `new_max = 99.5` (`damage = 50 * 0.01 = 0.5`) —
//! the correct clamp result is `99.5`; a rule comparing against the
//! UNRATCHETED `max_biocapacity = 100` instead would wrongly leave
//! `biocapacity` at `99.75`.

use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str =
    include_str!("../content/scenarios/metabolism-ratcheted-ceiling-conformance.bscn");
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

/// The clamp binds against the RATCHETED ceiling (`99.5`), not the
/// unratcheted seed (`100.0`) and not the unclamped `current + delta`
/// (`99.75`). Mutation-verified (by hand during authoring, reverted before
/// commit): swapping `metabolism.bsl`'s `capped-at-ceiling` binding's
/// second/third operand from `new-max` to `max-cap` flips this test from
/// `99.5` to `99.75` — the exact "clamps against the wrong ceiling"
/// mutation this vector exists to catch.
#[test]
fn the_ceiling_clamp_binds_against_the_ratcheted_ceiling_not_the_original() {
    let graph = run();
    assert_eq!(attribute(&graph, 0, "territory/biocapacity"), 99.5);
}

/// The ratchet itself: `max_biocapacity` ends the tick at `99.5`, strictly
/// below its `100.0` seed.
#[test]
fn the_ceiling_is_strictly_ratcheted() {
    let graph = run();
    let max = attribute(&graph, 0, "territory/max-biocapacity");
    assert_eq!(max, 99.5);
    assert!(max < 100.0);
}

/// Byte-determinism.
#[test]
fn the_ratcheted_ceiling_scenario_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after);
    assert_eq!(a.fired, 1);
}
