//! The `(when …)` gate's OR-not-AND conformance suite for
//! `dispossession/territory-transfer`.
//!
//! # Why this scenario exists
//!
//! `dispossession-conformance.bscn` (all three primary rates nonzero) and
//! `dispossession-zero-rate-conformance.bscn` (all three zero) agree on
//! whether the gate should pass under BOTH an `or`-shaped condition and an
//! `and`-shaped mistake — neither vector can tell the two apart. This
//! scenario isolates exactly one nonzero primary rate (`foreclosure-rate`;
//! `eviction-rate`, `displacement-rate` and both structural factors are
//! zero), which passes under the frozen engine's De Morgan-equivalent OR
//! and would NOT pass under an AND.
//!
//! # Provenance
//!
//! Frozen-engine run (ad hoc, single subject, not a checked-in script — the
//! numbers are simple enough to state and verify inline):
//!
//! ```text
//! wealth: 998000.0
//! intensity: 0.2
//! value_transfer {'territory': 'foreclosure-only-county', 'total_transferred': 2000.0,
//!                  'net_received': 1900.0, 'deadweight_loss': 100.0}
//! dispossession_event {'territory': 'foreclosure-only-county', 'intensity': 0.2,
//!                       'foreclosure_rate': 0.5, 'eviction_rate': 0.0, 'displacement_rate': 0.0}
//! ```
//!
//! `0.4 * 0.5 = 0.2` exactly (both operands terminate in binary — no
//! rounding to chase), so this suite's numbers are exact literals rather
//! than `repr()`-pinned decimals.

use babylon_bsl::evaluator::Value;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str =
    include_str!("../content/scenarios/dispossession-single-rate-conformance.bscn");
const RULE: &str = include_str!("../content/rules/dispossession.bsl");

fn run() -> (MemoryGraph, CollectingSink) {
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink)
        .expect("one nonzero primary rate must pass the (when …) gate");
    (graph, sink)
}

fn attribute(graph: &MemoryGraph, id: u64, field: &str) -> f64 {
    graph
        .node_attribute(NodeId(id), field)
        .unwrap_or_else(|e| panic!("node {id} field {field}: {}", e.message))
}

/// One nonzero primary rate is enough: the subject fires, the intensity
/// equals the isolated `weight_foreclosure * foreclosure_rate` term exactly,
/// and wealth moves.
#[test]
fn a_single_nonzero_primary_rate_passes_the_gate() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, 0, "territory/dispossession-intensity"),
        0.2
    );
    assert_eq!(attribute(&graph, 0, "territory/wealth"), 998_000.0);
}

/// Both events fire, in the frozen source's order, and — F4 — the
/// `DISPOSSESSION_EVENT` payload is asserted per key, not just its
/// count/type. `eviction-rate`/`displacement-rate` show `0.0` (both zero
/// inputs, never touched by the floor), proving the payload carries the
/// isolated single active rate honestly.
#[test]
fn both_events_fire_in_source_order_with_full_payloads() {
    let (_, sink) = run();
    let types: Vec<&str> = sink.events.iter().map(|(ty, _)| ty.as_str()).collect();
    assert_eq!(types, vec!["VALUE_TRANSFER", "DISPOSSESSION_EVENT"]);

    let (_, transfer_payload) = &sink.events[0];
    assert_eq!(
        transfer_payload[0],
        ("territory".to_owned(), Value::NodeRef(NodeId(0)))
    );
    assert_eq!(
        transfer_payload[1],
        ("total-transferred".to_owned(), Value::Real(2_000.0))
    );
    assert_eq!(
        transfer_payload[2],
        ("net-received".to_owned(), Value::Real(1_900.0))
    );
    assert_eq!(
        transfer_payload[3],
        ("deadweight-loss".to_owned(), Value::Real(100.0))
    );

    let (_, event_payload) = &sink.events[1];
    assert_eq!(
        event_payload[0],
        ("territory".to_owned(), Value::NodeRef(NodeId(0)))
    );
    assert_eq!(event_payload[1], ("intensity".to_owned(), Value::Real(0.2)));
    assert_eq!(
        event_payload[2],
        ("foreclosure-rate".to_owned(), Value::Real(0.5))
    );
    assert_eq!(
        event_payload[3],
        ("eviction-rate".to_owned(), Value::Real(0.0))
    );
    assert_eq!(
        event_payload[4],
        ("displacement-rate".to_owned(), Value::Real(0.0))
    );
}

/// The gate is a genuine OR: exactly one subject, and it fires.
#[test]
fn the_single_subject_passes_the_when_guard() {
    let report = run_once(SCENARIO, RULE).expect("the pack must run");
    assert_eq!(
        report.fired, 1,
        "one nonzero disjunct must be enough — an (and …) mistake would leave this 0"
    );
}
