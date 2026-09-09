//! The CEILING-MATRIX conformance suite for `dispossession/territory-transfer`
//! — completes the per-input mutation table `dispossession_negative_input_
//! conformance.rs` starts (adversarial-review follow-up, F2/F3, PR #498).
//!
//! Together the two suites individually exercise all TEN per-input
//! floor/ceiling clamps `DispossessionEventSystem` applies: this one covers
//! `eviction-rate`'s ceiling (and gate anchor) plus `displacement-rate`/
//! `concentrated-ownership`/`absentee-landlord-share`'s ceilings, and
//! `foreclosure-rate`'s floor; the companion covers the other five.
//!
//! # Provenance
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/dispossession_ceiling_matrix_conformance.py
//! ```
//!
//! Its output, verbatim:
//!
//! ```text
//! post-tick state:
//!   ceiling-matrix-county  wealth=994800.0 dispossession_intensity=0.5199999999999999
//!
//! events:
//!   value_transfer {'territory': 'ceiling-matrix-county', 'total_transferred': 5199.999999999999,
//!                    'net_received': 4939.999999999999, 'deadweight_loss': 259.99999999999994}
//!   dispossession_event {'territory': 'ceiling-matrix-county', 'intensity': 0.5199999999999999,
//!                         'foreclosure_rate': 0.0, 'eviction_rate': 6.0, 'displacement_rate': 8.0}
//! ```
//!
//! `intensity` is `weight_eviction + weight_displacement + weight_tax_sale +
//! weight_eminent_domain` exactly (`0.3 + 0.15 + 0.05 + 0.02`) — every one
//! of the four ceiling-clamped inputs (`6`, `8`, `9`, `4`) contributes
//! EXACTLY as much as a `1.0` seed would, and `foreclosure-rate`'s floored
//! `-6` contributes nothing. This is deliberate, not incidental: it proves
//! every one of the four ceiling clamps here does real work — each maps a
//! genuinely different out-of-domain input down to the SAME `1.0`
//! contribution — which a fixture that never exceeds `1.0` cannot prove
//! (`min(1.0, 1.0)` is a no-op whether or not the clamp exists).
//! `foreclosure_rate` in the payload is `0.0` (floored from `-6.0`), not the
//! raw seed.

use babylon_bsl::evaluator::Value;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str =
    include_str!("../content/scenarios/dispossession-ceiling-matrix-conformance.bscn");
const RULE: &str = include_str!("../content/rules/dispossession.bsl");

fn run() -> (MemoryGraph, CollectingSink) {
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink)
        .expect("the Dispossession pack must run under the ceiling-matrix const environment");
    (graph, sink)
}

fn attribute(graph: &MemoryGraph, id: u64, field: &str) -> f64 {
    graph
        .node_attribute(NodeId(id), field)
        .unwrap_or_else(|e| panic!("node {id} field {field}: {}", e.message))
}

/// The four ceiling-clamped inputs each contribute exactly a `1.0`-seed
/// worth; the floored `foreclosure-rate` contributes nothing.
#[test]
fn all_four_ceiling_clamps_and_the_foreclosure_floor_fire_correctly() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, 0, "territory/dispossession-intensity"),
        0.519_999_999_999_999_9
    );
}

/// Wealth moves by the correctly clamped transfer amount.
#[test]
fn wealth_moves_by_the_correctly_clamped_transfer_amount() {
    let (graph, _) = run();
    assert_eq!(attribute(&graph, 0, "territory/wealth"), 994_800.0);
}

/// F4: the full payload of both events, per key.
#[test]
fn both_event_payloads_are_asserted_in_full() {
    let (_, sink) = run();
    assert_eq!(sink.events.len(), 2);

    let (transfer_ty, transfer_payload) = &sink.events[0];
    assert_eq!(transfer_ty, "VALUE_TRANSFER");
    assert_eq!(
        transfer_payload[0],
        ("territory".to_owned(), Value::NodeRef(NodeId(0)))
    );
    assert_eq!(
        transfer_payload[1],
        (
            "total-transferred".to_owned(),
            Value::Real(5_199.999_999_999_999)
        )
    );
    assert_eq!(
        transfer_payload[2],
        (
            "net-received".to_owned(),
            Value::Real(4_939.999_999_999_999)
        )
    );
    assert_eq!(
        transfer_payload[3],
        (
            "deadweight-loss".to_owned(),
            Value::Real(259.999_999_999_999_94)
        )
    );

    let (event_ty, event_payload) = &sink.events[1];
    assert_eq!(event_ty, "DISPOSSESSION_EVENT");
    assert_eq!(
        event_payload[0],
        ("territory".to_owned(), Value::NodeRef(NodeId(0)))
    );
    assert_eq!(
        event_payload[1],
        ("intensity".to_owned(), Value::Real(0.519_999_999_999_999_9))
    );
    assert_eq!(
        event_payload[2],
        ("foreclosure-rate".to_owned(), Value::Real(0.0),),
        "the payload must show the FLOORED value (0.0), not the raw seed (-6.0)"
    );
    assert_eq!(
        event_payload[3],
        ("eviction-rate".to_owned(), Value::Real(6.0))
    );
    assert_eq!(
        event_payload[4],
        ("displacement-rate".to_owned(), Value::Real(8.0))
    );
}

/// Byte-determinism under the ceiling-matrix environment.
#[test]
fn the_ceiling_matrix_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after);
    assert_ne!(a.before, a.after);
    assert_eq!(a.fired, 1);
}
