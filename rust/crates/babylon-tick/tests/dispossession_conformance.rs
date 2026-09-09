//! Conformance vectors for `dispossession/territory-transfer`, taken from the
//! frozen Python engine's live behaviour.
//!
//! # Provenance
//!
//! Every state value below was printed by the frozen
//! `DispossessionEventSystem` running one `step()` over a fixture that
//! mirrors `content/scenarios/dispossession-conformance.bscn` node for node.
//! The command, from the repository root:
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/dispossession_conformance.py
//! ```
//!
//! Its output on 2026-08-11, verbatim:
//!
//! ```text
//! defines (src/babylon/data/defines.yaml, dispossession: section):
//!   dispossession.weight_foreclosure = 0.4
//!   dispossession.weight_eviction = 0.3
//!   dispossession.weight_displacement = 0.15
//!   dispossession.weight_tax_sale = 0.05
//!   dispossession.weight_eminent_domain = 0.02
//!   dispossession.deadweight_loss_fraction = 0.05
//!   dispossession.transfer_scale = 0.01
//!
//! post-tick state:
//!   foreclosed-county  wealth=996420.0 dispossession_intensity=0.3580000000000001
//!   insolvent-county   wealth=0.0 dispossession_intensity=0.3580000000000001
//!
//! events:
//!   value_transfer {'territory': 'foreclosed-county', 'total_transferred': 3580.0000000000014,
//!                    'net_received': 3401.0000000000014, 'deadweight_loss': 179.00000000000009}
//!   dispossession_event {'territory': 'foreclosed-county', 'intensity': 0.3580000000000001,
//!                         'foreclosure_rate': 0.5, 'eviction_rate': 0.3, 'displacement_rate': 0.2}
//!   dispossession_event {'territory': 'insolvent-county', 'intensity': 0.3580000000000001,
//!                         'foreclosure_rate': 0.5, 'eviction_rate': 0.3, 'displacement_rate': 0.2}
//! ```
//!
//! `insolvent-county` proves the `(guard (> transfer-amount 0) …)` split: its
//! `transfer_amount` computes to exactly `0.0` (zero wealth), so it gets a
//! `dispossession_event` and an intensity write but no `value_transfer` and
//! no wealth write — the frozen engine's own event log confirms only ONE
//! `value_transfer` fires, for `foreclosed-county`.
//!
//! # Why exact equality and no tolerance
//!
//! Both sides run IEEE-754 basic operations on binary64 — `+ − × ÷` and
//! comparison, correctly rounded, reproducing bit-exactly across
//! implementations (`bsl-language.rst` §4.3). `<arith>` is strictly binary
//! (`E-PARSE-040`), so the rule states each association
//! `intensity.py:41-47`/`dispossession_events.py:95` use rather than
//! implying it. The decimals below are Python `repr` output — the shortest
//! round-tripping decimal for each double — and Rust parses a float literal
//! correctly rounded, so e.g. `996420.0_f64` IS the double Python printed.

use babylon_bsl::evaluator::Value;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str = include_str!("../content/scenarios/dispossession-conformance.bscn");
const RULE: &str = include_str!("../content/rules/dispossession.bsl");

fn run() -> (MemoryGraph, CollectingSink) {
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink).expect("the Dispossession pack must run");
    (graph, sink)
}

fn attribute(graph: &MemoryGraph, id: u64, field: &str) -> f64 {
    graph
        .node_attribute(NodeId(id), field)
        .unwrap_or_else(|e| panic!("node {id} field {field}: {}", e.message))
}

/// Both subjects' post-tick state, against the frozen engine's own numbers,
/// exactly (no tolerance).
#[test]
fn post_tick_state_matches_the_frozen_engine_exactly() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, 0, "territory/wealth"),
        996_420.0,
        "foreclosed-county: wealth"
    );
    assert_eq!(
        attribute(&graph, 0, "territory/dispossession-intensity"),
        0.358_000_000_000_000_1,
        "foreclosed-county: dispossession-intensity"
    );
    assert_eq!(
        attribute(&graph, 1, "territory/wealth"),
        0.0,
        "insolvent-county: wealth"
    );
    assert_eq!(
        attribute(&graph, 1, "territory/dispossession-intensity"),
        0.358_000_000_000_000_1,
        "insolvent-county: dispossession-intensity"
    );
}

/// The weighted intensity formula, in the rule's own forward association
/// order — proving the exact five-term sum rather than trusting a single
/// pinned number.
#[test]
fn the_intensity_is_the_five_term_weighted_sum() {
    let t1 = 0.4_f64 * 0.5;
    let t2 = 0.3_f64 * 0.3;
    let t3 = 0.15_f64 * 0.2;
    let t4 = 0.05_f64 * 0.6;
    let t5 = 0.02_f64 * 0.4;
    // The rule's own left-to-right nesting: (+ (+ (+ (+ t1 t2) t3) t4) t5).
    let raw = (((t1 + t2) + t3) + t4) + t5;
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, 0, "territory/dispossession-intensity"),
        raw
    );
}

/// `insolvent-county` (zero wealth, same rates as `foreclosed-county`) hits
/// the `transfer-amount == 0` branch: the guard does not fire, so wealth
/// stays exactly `0.0` rather than going negative or being written at all.
#[test]
fn a_territory_with_zero_wealth_transfers_nothing() {
    let (graph, _) = run();
    assert_eq!(attribute(&graph, 1, "territory/wealth"), 0.0);
}

/// The value-transfer split: `net_received + deadweight_loss ==
/// total_transferred` exactly, and the deadweight fraction is applied
/// forward (`transfer_amount * deadweight_loss_fraction`), not derived by
/// subtraction.
#[test]
fn the_transfer_splits_into_net_received_and_deadweight_loss() {
    let (_, sink) = run();
    let transfers: Vec<_> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "VALUE_TRANSFER")
        .collect();
    assert_eq!(
        transfers.len(),
        1,
        "only the wealthy subject transfers value"
    );
    let (_, payload) = transfers[0];
    assert_eq!(
        payload[0],
        ("territory".to_owned(), Value::NodeRef(NodeId(0)))
    );
    assert_eq!(
        payload[1],
        (
            "total-transferred".to_owned(),
            Value::Real(3_580.000_000_000_001_4)
        )
    );
    assert_eq!(
        payload[2],
        (
            "net-received".to_owned(),
            Value::Real(3_401.000_000_000_001_4)
        )
    );
    assert_eq!(
        payload[3],
        (
            "deadweight-loss".to_owned(),
            Value::Real(179.000_000_000_000_09)
        )
    );
}

/// Every subject that passes the `when` guard gets a `DISPOSSESSION_EVENT`
/// — unconditionally, unlike `VALUE_TRANSFER` — and F4: the FULL payload of
/// each is asserted per key, not just its count/type. Both subjects share
/// the same rates, so both payloads are identical except the `territory`
/// ref.
#[test]
fn every_subject_emits_dispossession_event_with_the_full_payload() {
    let (_, sink) = run();
    let events: Vec<_> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "DISPOSSESSION_EVENT")
        .collect();
    assert_eq!(events.len(), 2, "both subjects pass the when guard");

    let (_, foreclosed_payload) = events[0];
    assert_eq!(
        foreclosed_payload[0],
        ("territory".to_owned(), Value::NodeRef(NodeId(0)))
    );
    assert_eq!(
        foreclosed_payload[1],
        ("intensity".to_owned(), Value::Real(0.358_000_000_000_000_1))
    );
    assert_eq!(
        foreclosed_payload[2],
        ("foreclosure-rate".to_owned(), Value::Real(0.5))
    );
    assert_eq!(
        foreclosed_payload[3],
        ("eviction-rate".to_owned(), Value::Real(0.3))
    );
    assert_eq!(
        foreclosed_payload[4],
        ("displacement-rate".to_owned(), Value::Real(0.2))
    );

    let (_, insolvent_payload) = events[1];
    assert_eq!(
        insolvent_payload[0],
        ("territory".to_owned(), Value::NodeRef(NodeId(1)))
    );
    assert_eq!(
        insolvent_payload[1],
        ("intensity".to_owned(), Value::Real(0.358_000_000_000_000_1))
    );
    assert_eq!(
        insolvent_payload[2],
        ("foreclosure-rate".to_owned(), Value::Real(0.5))
    );
    assert_eq!(
        insolvent_payload[3],
        ("eviction-rate".to_owned(), Value::Real(0.3))
    );
    assert_eq!(
        insolvent_payload[4],
        ("displacement-rate".to_owned(), Value::Real(0.2))
    );
}

/// Event ORDER matches the frozen source exactly: `VALUE_TRANSFER` (inside
/// the `if transfer_amount > 0.0:` block) fires before
/// `DISPOSSESSION_EVENT` (outside it, later in the same subject's
/// processing) — see the `.bsl` header's transcription note.
#[test]
fn value_transfer_fires_before_dispossession_event_for_the_same_subject() {
    let (_, sink) = run();
    let types: Vec<&str> = sink.events.iter().map(|(ty, _)| ty.as_str()).collect();
    assert_eq!(
        types,
        vec![
            "VALUE_TRANSFER",
            "DISPOSSESSION_EVENT",
            "DISPOSSESSION_EVENT"
        ],
        "foreclosed-county's VALUE_TRANSFER, then both subjects' DISPOSSESSION_EVENT \
         in subject-declaration order"
    );
}

/// Byte-determinism: the same content twice is the same post-state hash,
/// and the tick moved state at all.
#[test]
fn the_dispossession_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after, "two runs, one post-state");
    assert_ne!(a.before, a.after, "the pack must move state");
    assert_eq!(a.fired, 2, "both subjects pass the (when …) guard");
}

/// A rule reading a coefficient the scenario never declared fails at LOAD,
/// with the coefficient named — the same discipline Vitality/Lifecycle's
/// own conformance suites pin.
#[test]
fn a_rule_reading_an_undeclared_coefficient_is_refused_at_load() {
    let rule = "(rule dispossession/typo \
                :role mechanic :evidence derived :material-basis \"a territory has a foreclosure rate\" :fuel 32 \
                (bindings \
                  (binding wealth :field territory/wealth) \
                  (binding rate :const dispossession/foreclosure-rat)) \
                (effects (update-node self territory/wealth (set (* rate wealth)))))";
    let Err(err) = run_once(SCENARIO, rule) else {
        panic!("a mistyped coefficient must not load");
    };
    assert!(
        err.contains("dispossession/foreclosure-rat"),
        "the rejection must name the coefficient: {err}"
    );
}
