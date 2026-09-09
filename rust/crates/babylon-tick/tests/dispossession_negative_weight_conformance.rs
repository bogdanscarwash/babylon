//! The NEGATIVE-WEIGHT conformance suite for `dispossession/territory-transfer`
//! — closes the LAST gap in the per-clamp mutation table (F3, PR #498):
//! D-3's total-sum FLOOR clamp is mutation-dead against any RATE mutation
//! alone once the ten per-input clamps guarantee every rate/structural term
//! is `[0, 1]`, but a NEGATIVE WEIGHT (the exact same unguarded `:const`
//! surface F1 found) still reaches it.
//!
//! # Provenance
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/dispossession_negative_weight_conformance.py
//! ```
//!
//! Its output, verbatim:
//!
//! ```text
//! post-tick state:
//!   negative-weight-county  wealth=1000000.0 dispossession_intensity=0.0
//!
//! events:
//!   dispossession_event {'territory': 'negative-weight-county', 'intensity': 0.0,
//!                         'foreclosure_rate': 1.0, 'eviction_rate': 0.0,
//!                         'displacement_rate': 0.0}
//! ```
//!
//! Raw intensity is `weight_foreclosure(-1) * foreclosure_rate(1) = -1`;
//! D-3's floor clamp must land on exactly `0.0`, not a negative value — and
//! since intensity is `0.0`, `transfer_amount` is `0.0` too, so the
//! `(guard (> transfer-amount 0) …)` block does not fire: no wealth write,
//! no `VALUE_TRANSFER`, only the unconditional `DISPOSSESSION_EVENT`.

use babylon_bsl::evaluator::Value;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str =
    include_str!("../content/scenarios/dispossession-negative-weight-conformance.bscn");
const RULE: &str = include_str!("../content/rules/dispossession.bsl");

fn run() -> (MemoryGraph, CollectingSink) {
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink)
        .expect("the Dispossession pack must run under a negative-weight const environment");
    (graph, sink)
}

fn attribute(graph: &MemoryGraph, id: u64, field: &str) -> f64 {
    graph
        .node_attribute(NodeId(id), field)
        .unwrap_or_else(|e| panic!("node {id} field {field}: {}", e.message))
}

/// D-3's floor clamp: a negative weight against an in-domain positive rate
/// produces a genuinely negative raw sum, and the clamp must land on
/// exactly `0.0`.
#[test]
fn the_total_sum_floor_clamps_a_negative_weight_to_exactly_zero() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, 0, "territory/dispossession-intensity"),
        0.0
    );
}

/// Zero intensity means zero transfer amount: the guard does not fire, so
/// wealth is untouched and no `VALUE_TRANSFER` fires.
#[test]
fn zero_intensity_means_no_transfer() {
    let (graph, sink) = run();
    assert_eq!(attribute(&graph, 0, "territory/wealth"), 1_000_000.0);
    let transfers = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "VALUE_TRANSFER")
        .count();
    assert_eq!(transfers, 0);
}

/// The unconditional `DISPOSSESSION_EVENT` still fires (the `(when …)`
/// gate reads the RATE, not the weight, and `foreclosure-rate=1` passes
/// it) — F4: full payload asserted.
#[test]
fn the_dispossession_event_still_fires_with_zero_intensity() {
    let (_, sink) = run();
    assert_eq!(sink.events.len(), 1);
    let (ty, payload) = &sink.events[0];
    assert_eq!(ty, "DISPOSSESSION_EVENT");
    assert_eq!(
        payload[0],
        ("territory".to_owned(), Value::NodeRef(NodeId(0)))
    );
    assert_eq!(payload[1], ("intensity".to_owned(), Value::Real(0.0)));
    assert_eq!(
        payload[2],
        ("foreclosure-rate".to_owned(), Value::Real(1.0))
    );
    assert_eq!(payload[3], ("eviction-rate".to_owned(), Value::Real(0.0)));
    assert_eq!(
        payload[4],
        ("displacement-rate".to_owned(), Value::Real(0.0))
    );
}

/// Byte-determinism under the negative-weight environment.
#[test]
fn the_negative_weight_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after);
    assert_eq!(a.fired, 1);
}
