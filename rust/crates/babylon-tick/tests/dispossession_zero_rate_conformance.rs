//! The DISCRIMINATING conformance suite for `dispossession/territory-transfer`
//! — proves the frozen `when`-equivalent gate is inert on the shipped
//! canonical world, exactly the way `dispossession_events.py` is today.
//!
//! # Provenance
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/dispossession_zero_rate_conformance.py
//! ```
//!
//! Its output on 2026-08-11, verbatim:
//!
//! ```text
//! post-tick state (must equal the pre-tick seed exactly — no writes):
//!   dormant-county-1   wealth=1000000.0 dispossession_intensity=None (seed wealth was 1000000.0)
//!   dormant-county-2   wealth=500000.0 dispossession_intensity=None (seed wealth was 500000.0)
//!
//! events:
//!   (none)
//! ```
//!
//! `dispossession_intensity=None` means the frozen engine's graph dict never
//! gained the key at all — the Rust seed uses `0` as a placeholder (BSL
//! requires every declared field to carry SOME seed value; Python's dict can
//! simply lack the key), so the assertion below checks the seed value is
//! UNCHANGED, not that it equals Python's `None`.
//!
//! `dispossession_events.py:75-76`'s `continue` reads only the three PRIMARY
//! rates (foreclosure/eviction/displacement) — `concentrated_ownership`/
//! `absentee_landlord_share` are deliberately nonzero in this scenario, and
//! the frozen engine still skips both subjects whole. That is the exact
//! fidelity claim this suite exists to prove, not an incidental detail.

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str =
    include_str!("../content/scenarios/dispossession-zero-rate-conformance.bscn");
const RULE: &str = include_str!("../content/rules/dispossession.bsl");

fn run() -> (MemoryGraph, CollectingSink) {
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink)
        .expect("the Dispossession pack must run, even when the guard excludes everyone");
    (graph, sink)
}

fn attribute(graph: &MemoryGraph, id: u64, field: &str) -> f64 {
    graph
        .node_attribute(NodeId(id), field)
        .unwrap_or_else(|e| panic!("node {id} field {field}: {}", e.message))
}

/// Neither subject's wealth moves: the `when` guard excludes both subjects'
/// EFFECTS — bindings are still resolved (and fuel-charged) before the
/// guard in slice 1, but no write consumes them.
#[test]
fn no_subject_s_wealth_moves() {
    let (graph, _) = run();
    assert_eq!(attribute(&graph, 0, "territory/wealth"), 1_000_000.0);
    assert_eq!(attribute(&graph, 1, "territory/wealth"), 500_000.0);
}

/// Neither subject's intensity is written — the seed placeholder survives
/// the tick untouched. (The intensity VALUE is still computed as a binding
/// before the guard in slice 1; what this pins is that the gate keeps the
/// write from ever consuming it.)
#[test]
fn no_subject_s_intensity_is_written() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, 0, "territory/dispossession-intensity"),
        0.0
    );
    assert_eq!(
        attribute(&graph, 1, "territory/dispossession-intensity"),
        0.0
    );
}

/// No event fires — not `DISPOSSESSION_EVENT`, not `VALUE_TRANSFER` — for
/// either subject.
#[test]
fn no_event_fires() {
    let (_, sink) = run();
    assert!(
        sink.events.is_empty(),
        "the gate must exclude both subjects before any effect runs"
    );
}

/// The `(when …)` gate excludes both subjects: `run_tick`'s `fired` counter
/// only increments past the guard, so zero subjects fire even though two
/// are `considered`.
#[test]
fn zero_subjects_pass_the_guard() {
    let report = run_once(SCENARIO, RULE).expect("the pack must run");
    assert_eq!(report.fired, 0, "both subjects fail the primary-rate guard");
}

/// Byte-determinism, AND the strongest form of "nothing happened": the
/// pre-tick and post-tick hashes are IDENTICAL, not merely reproducible.
#[test]
fn the_tick_leaves_state_completely_unchanged() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after, "two runs, one post-state");
    assert_eq!(
        a.before, a.after,
        "a tick where the guard excludes every subject must not move state at all"
    );
}
