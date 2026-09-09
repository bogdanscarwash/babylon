//! Conformance vectors for `lifecycle/dpd-circuit` under the DISCRIMINATING
//! `:const` environment — companion to `lifecycle_conformance.rs`.
//!
//! # Why this file exists
//!
//! Adversarial review of PR #493 found that `lifecycle-conformance.bscn`'s
//! shipped-defaults `:const` environment makes the legitimation index
//! identical (0.6039, STABLE) for every subject, and — since
//! `caregiver-ideology-default`/`institutional-hegemony-default` are both
//! `0.5c` — `transmitted_ideology` identical (0.5) regardless of
//! `caregiver-weight`/`institutional-weight`. Two mutations that should have
//! been caught passed the original 8-test suite unnoticed: swapping the two
//! ideology weights, and collapsing the crisis-classification ladder to a
//! constant. Both are real coverage gaps in the ORIGINAL suite (confirmed:
//! after this file was added, both mutations flip a test here — see the
//! PR body for the exact mutation-verification transcript).
//!
//! This file exercises the same rule pack (`lifecycle.bsl` is unchanged)
//! against `lifecycle-crisis-conformance.bscn`, a `:const` environment
//! DIFFERENT from `defines.yaml`'s shipped values: legitimation components
//! lowered to 0.1 (index 0.1, below the 0.3 crisis threshold) and the
//! ideology defaults set to the frozen `compute_ideology_transmission`
//! doctest's own worked example (0.3/0.8, not 0.5/0.5). See that file's
//! header for the full justification — these are genuinely runtime-moddable
//! `defines.yaml` coefficients (D-1 in the `.bsl` header), not made-up
//! numbers.
//!
//! # Provenance
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/lifecycle_crisis_conformance.py
//! ```
//!
//! Its output on 2026-08-11, verbatim:
//!
//! ```text
//! defines used (deliberately DISCRIMINATING, not defines.yaml's shipped values):
//!   legitimation_state components = {'pension_coverage': 0.1, 'ss_replacement_rate': 0.1,
//!     'healthcare_security': 0.1, 'home_ownership_rate': 0.1, 'retirement_confidence': 0.1}
//!   caregiver_ideology = 0.3, institutional_hegemony = 0.8 (doctest example)
//!   lifecycle.legitimation_crisis_threshold = 0.3
//!   lifecycle.ideology_caregiver_weight = 0.7
//!   lifecycle.ideology_institutional_weight = 0.3
//!   lifecycle.ideology_regression_coefficient = 0.4
//!
//! post-tick state:
//!   entering-crisis    pop_d=2095.195 pop_p=6040.675 pop_d_prime=1858.665
//!     wealth_d_prime=9610000.0 dependency_ratio=0.6545394347486001
//!     legitimation_index=0.1 legitimation_crisis='crisis' (code=2)
//!     transmitted_ideology=0.47
//!   already-crisis     pop_d=2886.7 pop_p=5060.3 pop_d_prime=1548.0
//!     wealth_d_prime=4805000.0 dependency_ratio=0.876370966148252
//!     legitimation_index=0.1 legitimation_crisis='crisis' (code=2)
//!     transmitted_ideology=0.47
//!
//! events (frozen engine's ACTUAL output, D-5 bug included):
//!   lifecycle_transition {'territory_id': 'entering-crisis', ...}
//!   legitimation_crisis {'territory_id': 'entering-crisis', 'legitimation_index': 0.1}
//!   inheritance_transfer {'territory_id': 'entering-crisis', ...}
//!   lifecycle_transition {'territory_id': 'already-crisis', ...}
//!   legitimation_crisis {'territory_id': 'already-crisis', 'legitimation_index': 0.1}
//!   inheritance_transfer {'territory_id': 'already-crisis', ...}
//! ```
//!
//! The frozen engine fires `legitimation_crisis` for BOTH subjects — D-5's
//! over-firing bug, now demonstrated on the CRISIS side too:
//! `already-crisis` is seeded PRE-crisis `"crisis"` and stays classified
//! CRISIS, so a correct edge check must NOT re-fire for it, but
//! `prev_crisis != "CRISIS"` (comparing a lowercase `StrEnum.value` against
//! the uppercase literal) is true regardless of the actual previous state.
//! This pack's int-coded, genuinely edge-triggered guard fires
//! `LEGITIMATION_CRISIS` for `entering-crisis` ONLY.

use babylon_bsl::evaluator::Value;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str = include_str!("../content/scenarios/lifecycle-crisis-conformance.bscn");
const RULE: &str = include_str!("../content/rules/lifecycle.bsl");

struct Expected {
    name: &'static str,
    id: u64,
    pop_d: f64,
    pop_p: f64,
    pop_d_prime: f64,
    wealth_d_prime: f64,
    dependency_ratio: f64,
    legitimation_index: f64,
    legitimation_crisis: f64,
    transmitted_ideology: f64,
}

const EXPECTED: [Expected; 2] = [
    Expected {
        name: "entering-crisis",
        id: 0,
        pop_d: 2095.195,
        pop_p: 6040.675,
        pop_d_prime: 1858.665,
        wealth_d_prime: 9_610_000.0,
        dependency_ratio: 0.6545394347486001,
        legitimation_index: 0.1,
        legitimation_crisis: 2.0,
        transmitted_ideology: 0.47,
    },
    Expected {
        name: "already-crisis",
        id: 1,
        pop_d: 2886.7,
        pop_p: 5060.3,
        pop_d_prime: 1548.0,
        wealth_d_prime: 4_805_000.0,
        dependency_ratio: 0.876370966148252,
        legitimation_index: 0.1,
        legitimation_crisis: 2.0,
        transmitted_ideology: 0.47,
    },
];

fn run() -> (MemoryGraph, CollectingSink) {
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink).expect("the lifecycle pack must run");
    (graph, sink)
}

fn attribute(graph: &MemoryGraph, id: u64, field: &str) -> f64 {
    graph
        .node_attribute(NodeId(id), field)
        .unwrap_or_else(|e| panic!("node {id} field {field}: {}", e.message))
}

/// Every ported state field, against the frozen engine's own numbers,
/// exactly (no tolerance) — under the DISCRIMINATING `:const` environment.
#[test]
fn post_tick_state_matches_the_frozen_engine_exactly() {
    let (graph, _) = run();
    for e in &EXPECTED {
        assert_eq!(
            attribute(&graph, e.id, "territory/pop-d"),
            e.pop_d,
            "{}: pop-d",
            e.name
        );
        assert_eq!(
            attribute(&graph, e.id, "territory/pop-p"),
            e.pop_p,
            "{}: pop-p",
            e.name
        );
        assert_eq!(
            attribute(&graph, e.id, "territory/pop-d-prime"),
            e.pop_d_prime,
            "{}: pop-d-prime",
            e.name
        );
        assert_eq!(
            attribute(&graph, e.id, "territory/wealth-d-prime"),
            e.wealth_d_prime,
            "{}: wealth-d-prime",
            e.name
        );
        assert_eq!(
            attribute(&graph, e.id, "territory/dependency-ratio"),
            e.dependency_ratio,
            "{}: dependency-ratio",
            e.name
        );
        assert_eq!(
            attribute(&graph, e.id, "territory/legitimation-index"),
            e.legitimation_index,
            "{}: legitimation-index",
            e.name
        );
        assert_eq!(
            attribute(&graph, e.id, "territory/legitimation-crisis"),
            e.legitimation_crisis,
            "{}: legitimation-crisis",
            e.name
        );
        assert_eq!(
            attribute(&graph, e.id, "territory/transmitted-ideology"),
            e.transmitted_ideology,
            "{}: transmitted-ideology",
            e.name
        );
    }
}

/// Kills the "collapse the classification ladder to a constant" mutation:
/// both subjects must land CRISIS (code 2) under an index of 0.1 against a
/// 0.3 threshold. If `new-crisis-class` were hardcoded (or the threshold
/// comparisons were no-ops), this index/classification pair would not hold.
#[test]
fn the_index_actually_drives_the_classification_ladder_into_crisis() {
    let (graph, _) = run();
    assert_eq!(attribute(&graph, 0, "territory/legitimation-index"), 0.1);
    assert_eq!(attribute(&graph, 0, "territory/legitimation-crisis"), 2.0);
    assert_eq!(attribute(&graph, 1, "territory/legitimation-index"), 0.1);
    assert_eq!(attribute(&graph, 1, "territory/legitimation-crisis"), 2.0);
}

/// D-5's repair, from the CRISIS side: `entering-crisis` (PRE-crisis
/// STABLE, this tick CRISIS) fires `LEGITIMATION_CRISIS`; `already-crisis`
/// (PRE-crisis CRISIS, this tick STILL CRISIS) must NOT re-fire it — the
/// frozen engine's broken comparison fires for BOTH (see this file's
/// header). Exactly one event, naming the territory that actually crossed
/// the edge.
#[test]
fn the_repaired_edge_check_fires_crisis_only_on_the_transition_into_it() {
    let (_, sink) = run();
    let crises: Vec<_> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "LEGITIMATION_CRISIS")
        .collect();
    assert_eq!(
        crises.len(),
        1,
        "exactly entering-crisis crosses non-CRISIS -> CRISIS this tick"
    );
    let (_, payload) = crises[0];
    assert_eq!(
        payload[0],
        ("territory-id".to_owned(), Value::NodeRef(NodeId(0)))
    );
    assert_eq!(
        payload[1],
        ("legitimation-index".to_owned(), Value::Real(0.1))
    );

    let recoveries: Vec<_> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "LEGITIMATION_RECOVERY")
        .collect();
    assert!(
        recoveries.is_empty(),
        "no subject transitions out of CRISIS under this :const environment"
    );
}

/// Kills the "swap caregiver-weight <-> institutional-weight" mutation:
/// with the doctest's discriminated inputs (0.3/0.8), the two weights are
/// no longer interchangeable, so `transmitted_ideology` pins the
/// weight-to-input pairing exactly. Swapping the weights would instead pair
/// `caregiver-weight` (0.7) with `institutional-hegemony` (0.8) and
/// `institutional-weight` (0.3) with `caregiver-ideology` (0.3), producing a
/// materially different `raw` (0.65 vs 0.45) and hence a different
/// `transmitted` value — this pins the CORRECT pairing's frozen-engine
/// number (see this file's header) so that swap is a real, catchable
/// mutation rather than a coincidental match.
#[test]
fn ideology_transmission_discriminates_the_two_weights() {
    let (graph, _) = run();
    for id in [0, 1] {
        assert_eq!(
            attribute(&graph, id, "territory/transmitted-ideology"),
            0.47
        );
    }
}

/// Byte-determinism, same shape as the shipped-defaults suite.
#[test]
fn the_crisis_environment_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after, "two runs, one post-state");
    assert_ne!(a.before, a.after, "the pack must move state");
    assert_eq!(a.fired, 2, "both territories pass the (unconditional) rule");
}
