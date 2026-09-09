//! The NEGATIVE-INPUT (per-term floor, plus `foreclosure-rate`'s ceiling)
//! conformance suite for `dispossession/territory-transfer` — the
//! adversarial-review follow-up (F2, PR #498) that proves the per-term
//! clamps D-4 restores are structurally different from clamping only the
//! total. This scenario covers `foreclosure-rate`'s CEILING plus every
//! OTHER input's FLOOR (`eviction-rate`/`displacement-rate`/
//! `concentrated-ownership`/`absentee-landlord-share`, all pushed negative
//! at once — one primary rate has to stay positive to keep the `(when …)`
//! gate open, so `foreclosure-rate` draws double duty as ceiling probe and
//! anchor). `dispossession-ceiling-matrix-conformance.rs` completes the
//! remaining four ceilings.
//!
//! # Provenance
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/dispossession_negative_input_conformance.py
//! ```
//!
//! Its output, verbatim:
//!
//! ```text
//! post-tick state:
//!   negative-input-county  wealth=996000.0 dispossession_intensity=0.4
//!
//! events:
//!   value_transfer {'territory': 'negative-input-county', 'total_transferred': 4000.0,
//!                    'net_received': 4000.0, 'deadweight_loss': 0.0}
//!   dispossession_event {'territory': 'negative-input-county', 'intensity': 0.4,
//!                         'foreclosure_rate': 5.0, 'eviction_rate': 0.0,
//!                         'displacement_rate': 0.0}
//! ```
//!
//! Three things worth flagging explicitly:
//!
//! - `intensity` is exactly `0.4` (`weight_foreclosure * 1.0`, the CEILING-
//!   clamped value of `foreclosure-rate=5`) — the negative `eviction-rate`
//!   (`-3`) contributes NOTHING, floored to `0.0` before it is ever
//!   weighted. A total-only floor (this pack's pre-fix shape) would instead
//!   have summed ALL FIVE raw terms — `0.4*5 + 0.3*(-3) + 0.15*(-8) +
//!   0.05*(-2) + 0.02*(-9) = -0.38` — and THEN floored the total, landing
//!   on `0.0`, not `0.4`: a materially different number reached by a
//!   materially different (and wrong) route.
//! - `foreclosure-rate=5` proves the per-input CEILING clamp does real
//!   work: ceiling-clamped to `1.0` it lands on the SAME intensity a seed
//!   of `1.0` already at the boundary would — which a fixture that never
//!   exceeds `1.0` (`dispossession-saturation-conformance.bscn`'s own rates
//!   included) cannot prove, since `min(1.0, 1.0)` is a no-op whether or
//!   not the clamp exists.
//! - The `DISPOSSESSION_EVENT` payload's `foreclosure_rate` is `5.0` (the
//!   FLOOR-ONLY, NOT ceiling-clamped, raw seed) and `eviction_rate` is
//!   `0.0` (floored from `-3.0`) — the frozen engine's payload reads the
//!   FLOOR-ONLY outer variables (`dispossession_events.py`'s own
//!   `foreclosure_rate`/`eviction_rate`, never `state.*`), confirming D-4's
//!   floor/ceiling split is the payload's actual behavior, not merely this
//!   pack's convenience.

use babylon_bsl::evaluator::Value;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str =
    include_str!("../content/scenarios/dispossession-negative-input-conformance.bscn");
const RULE: &str = include_str!("../content/rules/dispossession.bsl");

fn run() -> (MemoryGraph, CollectingSink) {
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink)
        .expect("the Dispossession pack must run under a negative-input const environment");
    (graph, sink)
}

fn attribute(graph: &MemoryGraph, id: u64, field: &str) -> f64 {
    graph
        .node_attribute(NodeId(id), field)
        .unwrap_or_else(|e| panic!("node {id} field {field}: {}", e.message))
}

/// The negative `eviction-rate` contributes NOTHING (floored to `0.0`) and
/// the out-of-ceiling `foreclosure-rate=5` contributes exactly as much as a
/// seed of `1.0` would (ceiling-clamped) — intensity is exactly the
/// isolated `weight-foreclosure * 1` term, proving BOTH per-term clamps
/// (not a total-only clamp landing on a different number).
#[test]
fn the_per_term_floor_and_ceiling_both_fire_correctly() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, 0, "territory/dispossession-intensity"),
        0.4
    );
}

/// Wealth moves by the correctly (per-term-floored) computed transfer
/// amount — `1_000_000 * 0.4 * 0.01 = 4_000`.
#[test]
fn wealth_moves_by_the_correctly_floored_transfer_amount() {
    let (graph, _) = run();
    assert_eq!(attribute(&graph, 0, "territory/wealth"), 996_000.0);
}

/// D-2's restored fraction floor: `-1` clamps to `0.0`, so NOTHING is
/// deadweight and the full transfer amount is `net-received`.
#[test]
fn the_deadweight_fraction_floor_leaves_the_full_amount_as_net_received() {
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
        ("total-transferred".to_owned(), Value::Real(4_000.0))
    );
    assert_eq!(
        payload[2],
        ("net-received".to_owned(), Value::Real(4_000.0))
    );
    assert_eq!(payload[3], ("deadweight-loss".to_owned(), Value::Real(0.0)));
}

/// F4: the full `DISPOSSESSION_EVENT` payload, per key — `foreclosure-rate`
/// must read `5.0` (the raw, floor-only seed — NOT ceiling-clamped to
/// `1.0`) and `eviction-rate` must read `0.0` (floored from `-3.0`),
/// matching the frozen engine's own outer-variable payload read exactly.
#[test]
fn the_dispossession_event_payload_shows_the_floor_only_values_not_ceiling_clamped_or_raw_negative()
{
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
    assert_eq!(payload[1], ("intensity".to_owned(), Value::Real(0.4)));
    assert_eq!(
        payload[2],
        ("foreclosure-rate".to_owned(), Value::Real(5.0)),
        "the payload must show the RAW seed (5.0), not the ceiling-clamped sum input (1.0)"
    );
    assert_eq!(
        payload[3],
        ("eviction-rate".to_owned(), Value::Real(0.0)),
        "the payload must show the FLOORED value (0.0), not the raw seed (-3.0)"
    );
    assert_eq!(
        payload[4],
        ("displacement-rate".to_owned(), Value::Real(0.0))
    );
}

/// Byte-determinism under the negative-input environment.
#[test]
fn the_negative_input_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after);
    assert_ne!(a.before, a.after);
    assert_eq!(a.fired, 1);
}
