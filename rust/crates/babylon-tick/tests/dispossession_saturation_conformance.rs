//! The SATURATION conformance suite for `dispossession/territory-transfer`
//! — the adversarial-review follow-up (F1/F3, PR #498) that proves every
//! clamp this pack transcribes actually fires when pushed, not just that
//! the shipped fixtures never happen to reach it.
//!
//! # Provenance
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/dispossession_saturation_conformance.py
//! ```
//!
//! Its output, verbatim:
//!
//! ```text
//! post-tick state:
//!   maxed-county       wealth=0.0 dispossession_intensity=1.0
//!
//! events:
//!   value_transfer {'territory': 'maxed-county', 'total_transferred': 1000000.0,
//!                    'net_received': 0.0, 'deadweight_loss': 1000000.0}
//!   dispossession_event {'territory': 'maxed-county', 'intensity': 1.0,
//!                         'foreclosure_rate': 1.0, 'eviction_rate': 1.0,
//!                         'displacement_rate': 1.0}
//! ```
//!
//! Three clamps fire in this ONE subject: the intensity ceiling (raw sum
//! `5.0` -> `1.0`), the transfer-amount ceiling (`wealth * 1.0 * 12 =
//! 12_000_000` -> `1_000_000`, spending the territory to exactly `0.0`
//! rather than driving it negative), and the deadweight-fraction ceiling
//! (`3` -> `1.0`, so `net_received` is exactly `0.0` rather than negative).
//! See `dispossession_saturation_conformance.py`'s header for why
//! `transfer_scale`/`deadweight_loss_fraction` need the
//! `DispossessionDefines.model_construct` bypass to reach through the real
//! engine's own Pydantic-gated configuration surface, and why that bypass
//! is legitimate provenance rather than a hack.

use babylon_bsl::evaluator::Value;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str =
    include_str!("../content/scenarios/dispossession-saturation-conformance.bscn");
const RULE: &str = include_str!("../content/rules/dispossession.bsl");

fn run() -> (MemoryGraph, CollectingSink) {
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink)
        .expect("the Dispossession pack must run under a saturating const environment");
    (graph, sink)
}

fn attribute(graph: &MemoryGraph, id: u64, field: &str) -> f64 {
    graph
        .node_attribute(NodeId(id), field)
        .unwrap_or_else(|e| panic!("node {id} field {field}: {}", e.message))
}

/// D-3's intensity ceiling: raw sum `5.0` saturates to exactly `1.0`, not
/// left unclamped and not merely "large".
#[test]
fn the_intensity_ceiling_saturates_at_exactly_one() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, 0, "territory/dispossession-intensity"),
        1.0
    );
}

/// D-2's restored transfer-amount ceiling: `transfer_amount_raw` is twelve
/// times wealth (`wealth * intensity(1.0) * transfer_scale(12)`), and the
/// clamp must cap it at exactly `wealth` — the territory is spent to `0.0`,
/// never driven negative.
#[test]
fn the_transfer_amount_ceiling_caps_at_wealth_not_beyond() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, 0, "territory/wealth"),
        0.0,
        "1_000_000 * 1.0 * 12 must clamp DOWN to 1_000_000, spending wealth to exactly zero"
    );
}

/// D-2's restored deadweight-fraction ceiling: `3` clamps to `1.0`, so the
/// ENTIRE transfer is deadweight and `net_received` is exactly `0.0` — not
/// negative, which is what an unclamped `total * 3` would produce.
#[test]
fn the_deadweight_fraction_ceiling_leaves_net_received_at_zero_not_negative() {
    let (_, sink) = run();
    let transfers: Vec<_> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "VALUE_TRANSFER")
        .collect();
    assert_eq!(transfers.len(), 1);
    let (_, payload) = transfers[0];
    assert_eq!(
        payload[0],
        ("territory".to_owned(), Value::NodeRef(NodeId(0)))
    );
    assert_eq!(
        payload[1],
        ("total-transferred".to_owned(), Value::Real(1_000_000.0))
    );
    assert_eq!(payload[2], ("net-received".to_owned(), Value::Real(0.0)));
    assert_eq!(
        payload[3],
        ("deadweight-loss".to_owned(), Value::Real(1_000_000.0))
    );
}

/// The full `DISPOSSESSION_EVENT` payload, per key — F4: no payload key in
/// this pack ships unasserted anywhere.
#[test]
fn the_dispossession_event_payload_is_asserted_in_full() {
    let (_, sink) = run();
    let events: Vec<_> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "DISPOSSESSION_EVENT")
        .collect();
    assert_eq!(events.len(), 1);
    let (_, payload) = events[0];
    assert_eq!(
        payload[0],
        ("territory".to_owned(), Value::NodeRef(NodeId(0)))
    );
    assert_eq!(payload[1], ("intensity".to_owned(), Value::Real(1.0)));
    assert_eq!(
        payload[2],
        ("foreclosure-rate".to_owned(), Value::Real(1.0))
    );
    assert_eq!(payload[3], ("eviction-rate".to_owned(), Value::Real(1.0)));
    assert_eq!(
        payload[4],
        ("displacement-rate".to_owned(), Value::Real(1.0))
    );
}

/// Byte-determinism under the saturating environment.
#[test]
fn the_saturation_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after);
    assert_ne!(a.before, a.after);
    assert_eq!(a.fired, 1);
}
