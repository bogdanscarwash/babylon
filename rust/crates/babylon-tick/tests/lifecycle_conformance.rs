//! Conformance vectors for `lifecycle/dpd-circuit`, taken from the frozen
//! Python engine's live behaviour.
//!
//! # Provenance
//!
//! Every state value below was printed by the frozen `LifecycleSystem`
//! running one `step()` over a fixture that mirrors
//! `content/scenarios/lifecycle-conformance.bscn` node for node. The
//! command, from the repository root:
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/lifecycle_conformance.py
//! ```
//!
//! Its output on 2026-08-11, verbatim:
//!
//! ```text
//! defines (src/babylon/data/defines.yaml, lifecycle: section):
//!   lifecycle.birth_rate = 0.0107
//!   lifecycle.rate_d_to_p = 0.0556
//!   lifecycle.rate_p_to_d_prime = 0.0213
//!   lifecycle.rate_d_prime_to_death = 0.039
//!   lifecycle.pension_coverage_rate = 0.73
//!   lifecycle.home_ownership_rate = 0.656
//!   lifecycle.ss_replacement_rate = 0.426
//!   lifecycle.healthcare_security = 0.6
//!   lifecycle.retirement_confidence = 0.5
//!   lifecycle.legit_w_home_ownership = 0.35
//!   lifecycle.legit_w_healthcare_security = 0.3
//!   lifecycle.legit_w_retirement_confidence = 0.2
//!   lifecycle.legit_w_pension_coverage = 0.1
//!   lifecycle.legit_w_ss_replacement = 0.05
//!   lifecycle.legitimation_crisis_threshold = 0.3
//!   lifecycle.legitimation_unstable_threshold = 0.5
//!   lifecycle.ideology_caregiver_weight = 0.7
//!   lifecycle.ideology_institutional_weight = 0.3
//!   lifecycle.ideology_regression_coefficient = 0.4
//!
//! post-tick state:
//!   core-county        pop_d=2095.195 pop_p=6040.675 pop_d_prime=1858.665
//!     wealth_d_prime=9610000.0 dependency_ratio=0.6545394347486001
//!     legitimation_index=0.6038999999999999 legitimation_crisis='stable'
//!     (code=0) transmitted_ideology=0.5
//!   growing-county     pop_d=2886.7 pop_p=5060.3 pop_d_prime=1548.0
//!     wealth_d_prime=4805000.0 dependency_ratio=0.876370966148252
//!     legitimation_index=0.6038999999999999 legitimation_crisis='stable'
//!     (code=0) transmitted_ideology=0.5
//!   recovering-county  pop_d=1963.7 pop_p=6962.099999999999
//!     pop_d_prime=2071.1 wealth_d_prime=19220000.0
//!     dependency_ratio=0.5795377831401446
//!     legitimation_index=0.6038999999999999 legitimation_crisis='stable'
//!     (code=0) transmitted_ideology=0.5
//!   young-county       pop_d=3836.45 pop_p=5605.25
//!     pop_d_prime=117.14999999999999 wealth_d_prime=0.0
//!     dependency_ratio=0.7053387449266313
//!     legitimation_index=0.6038999999999999 legitimation_crisis='stable'
//!     (code=0) transmitted_ideology=0.5
//!
//! events (frozen engine's ACTUAL output, D-5 bug included):
//!   lifecycle_transition {'territory_id': 'core-county', ...}
//!   inheritance_transfer {'territory_id': 'core-county', ...}
//!   lifecycle_transition {'territory_id': 'growing-county', ...}
//!   inheritance_transfer {'territory_id': 'growing-county', ...}
//!   lifecycle_transition {'territory_id': 'recovering-county', ...}
//!   inheritance_transfer {'territory_id': 'recovering-county', ...}
//!   lifecycle_transition {'territory_id': 'young-county', ...}
//!   inheritance_transfer {'territory_id': 'young-county', ...}
//! ```
//!
//! Notably absent from the frozen engine's event log: **no**
//! `legitimation_crisis`/`legitimation_recovery` event fires anywhere,
//! including for `recovering-county`, which is seeded `legitimation_crisis:
//! "crisis"` and ends the tick classified `stable`. That is D-5 (the `.bsl`
//! header): the frozen comparison `prev_crisis == "CRISIS"` (uppercase)
//! never matches a `StrEnum.value` (`"crisis"`, lowercase), so
//! `LEGITIMATION_RECOVERY` is dead code in the frozen engine. This pack's
//! int-coded classification has no case axis to inherit that bug through,
//! so it implements the edge-triggered semantics the frozen code's own
//! structure intends — a documented, deliberate divergence for the two
//! event types only. Every state field matches the frozen engine exactly
//! (see the table below): `inheritance_transfer` is the OTHER un-ported
//! branch (director-gate #492), and it writes no graph state in the frozen
//! engine either, so its absence here changes nothing this pack claims.
//!
//! # Why exact equality and no tolerance
//!
//! Both sides run IEEE-754 basic operations on binary64 — `+ − × ÷` and
//! comparison, correctly rounded, reproducing bit-exactly across
//! implementations (`bsl-language.rst` §4.3). `<arith>` is strictly binary
//! (`E-PARSE-040`), so the rule states each association Python's formulas
//! use rather than implying it (`formulas/lifecycle.py:52-59` for
//! population flow, `:133-139` for the legitimation index). The decimals
//! below are Python `repr` output — the shortest round-tripping decimal for
//! each double — and Rust parses a float literal correctly rounded, so
//! e.g. `2095.195_f64` IS the double Python printed. A tolerance here would
//! hide exactly the transcription error it would appear to absorb.

use babylon_bsl::evaluator::Value;
use babylon_bsl::scenario::load_scenario;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str = include_str!("../content/scenarios/lifecycle-conformance.bscn");
const RULE: &str = include_str!("../content/rules/lifecycle.bsl");

/// One subject's post-tick vector, in scenario declaration order.
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

const EXPECTED: [Expected; 4] = [
    Expected {
        name: "core-county",
        id: 0,
        pop_d: 2095.195,
        pop_p: 6040.675,
        pop_d_prime: 1858.665,
        wealth_d_prime: 9_610_000.0,
        dependency_ratio: 0.6545394347486001,
        legitimation_index: 0.6038999999999999,
        legitimation_crisis: 0.0,
        transmitted_ideology: 0.5,
    },
    Expected {
        name: "growing-county",
        id: 1,
        pop_d: 2886.7,
        pop_p: 5060.3,
        pop_d_prime: 1548.0,
        wealth_d_prime: 4_805_000.0,
        dependency_ratio: 0.876370966148252,
        legitimation_index: 0.6038999999999999,
        legitimation_crisis: 0.0,
        transmitted_ideology: 0.5,
    },
    Expected {
        name: "recovering-county",
        id: 2,
        pop_d: 1963.7,
        pop_p: 6962.099999999999,
        pop_d_prime: 2071.1,
        wealth_d_prime: 19_220_000.0,
        dependency_ratio: 0.5795377831401446,
        legitimation_index: 0.6038999999999999,
        legitimation_crisis: 0.0,
        transmitted_ideology: 0.5,
    },
    Expected {
        name: "young-county",
        id: 3,
        pop_d: 3836.45,
        pop_p: 5605.25,
        pop_d_prime: 117.14999999999999,
        wealth_d_prime: 0.0,
        dependency_ratio: 0.7053387449266313,
        legitimation_index: 0.6038999999999999,
        legitimation_crisis: 0.0,
        transmitted_ideology: 0.5,
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
/// exactly (no tolerance).
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

/// `young-county` seeds `pop-d-prime` and `wealth-d-prime` at zero: no D'
/// cohort exists yet, so `deaths == 0` and the surviving-fraction guard
/// takes its ELSE branch — wealth stays exactly `0.0`, not "decayed from
/// zero". `pop-d-prime` itself is NOT zero post-tick (the P->D' inflow
/// still runs), which is why this checks wealth specifically rather than
/// asserting the whole cohort stayed empty.
#[test]
fn a_territory_with_no_d_prime_cohort_leaves_its_wealth_untouched() {
    let (graph, _) = run();
    assert_eq!(attribute(&graph, 3, "territory/wealth-d-prime"), 0.0);
}

/// The Drain-equivalent for D': wealth decays by the surviving fraction of
/// the D' cohort exactly, at every subject that HAS a D' cohort to begin
/// with (every subject but young-county).
#[test]
fn wealth_d_prime_decays_by_the_surviving_fraction() {
    let (graph, _) = run();
    // core-county: old_total 1800, deaths = 0.039 * 1800 = 70.2,
    // surviving_fraction = 1 - 70.2/1800 = 0.961.
    let surviving = 1.0 - (0.039_f64 * 1800.0) / 1800.0;
    assert_eq!(
        attribute(&graph, 0, "territory/wealth-d-prime"),
        10_000_000.0 * surviving
    );
}

/// D-5's empirical proof: `recovering-county` is seeded PRE-crisis
/// `"crisis"` and this tick's classification is STABLE for every subject
/// (§ the .bsl header's D-1) — the frozen engine's own case-broken
/// comparison never fires `LEGITIMATION_RECOVERY` here (see this file's
/// header), but this pack's int-coded, correctly edge-triggered guard
/// does. `core-county`/`growing-county`/`young-county` start STABLE or
/// UNSTABLE and stay non-CRISIS, so neither guard fires for them.
#[test]
fn the_repaired_edge_check_fires_recovery_exactly_where_the_frozen_engine_should_have() {
    let (_, sink) = run();
    let recoveries: Vec<_> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "LEGITIMATION_RECOVERY")
        .collect();
    assert_eq!(
        recoveries.len(),
        1,
        "exactly recovering-county crosses CRISIS -> non-CRISIS this tick"
    );
    let (_, payload) = recoveries[0];
    assert_eq!(
        payload[0],
        ("territory-id".to_owned(), Value::NodeRef(NodeId(2)))
    );
    assert_eq!(
        payload[1],
        (
            "legitimation-index".to_owned(),
            Value::Real(0.6038999999999999)
        )
    );

    let crises: Vec<_> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "LEGITIMATION_CRISIS")
        .collect();
    assert!(
        crises.is_empty(),
        "no subject classifies CRISIS under the current defines (legit-index 0.6039 > \
         unstable-threshold 0.5) — matches the frozen engine exactly, D-1's own point"
    );
}

/// One `LIFECYCLE_TRANSITION` per subject, unconditionally.
#[test]
fn every_subject_emits_lifecycle_transition() {
    let (_, sink) = run();
    let transitions = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "LIFECYCLE_TRANSITION")
        .count();
    assert_eq!(transitions, 4);
}

/// Byte-determinism: the same content twice is the same post-state hash,
/// and the tick moved state at all.
#[test]
fn the_lifecycle_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after, "two runs, one post-state");
    assert_ne!(a.before, a.after, "the pack must move state");
    assert_eq!(
        a.fired, 4,
        "all four territories pass the (unconditional) rule"
    );
}

/// A rule reading a coefficient the scenario never declared fails at LOAD,
/// with the coefficient named — the same discipline Vitality's own
/// conformance suite pins.
#[test]
fn a_rule_reading_an_undeclared_coefficient_is_refused_at_load() {
    let rule = "(rule lifecycle/typo \
                :role mechanic :evidence derived :material-basis \"a population has a birth rate\" :fuel 32 \
                (bindings \
                  (binding pop-p :field territory/pop-p) \
                  (binding rate :const lifecycle/birth-rat)) \
                (effects (update-node self territory/pop-p (set (* rate pop-p)))))";
    let Err(err) = run_once(SCENARIO, rule) else {
        panic!("a mistyped coefficient must not load");
    };
    assert!(
        err.contains("lifecycle/birth-rat"),
        "the rejection must name the coefficient: {err}"
    );
}

/// Sanity check on this file's own provenance claim: the scenario's
/// `:const` rows carry the exact `defines.yaml` values the conformance
/// script printed, so a future edit to either side cannot silently drift
/// out of step with the other.
#[test]
fn the_scenario_consts_match_the_provenance_script_exactly() {
    let mut seeds = MemoryGraph::new();
    let scenario = load_scenario(SCENARIO, &mut seeds).expect("the scenario must load");
    let expect_const = |name: &str, value: f64| {
        let Some(Value::Real(actual)) = scenario.consts.get(name) else {
            panic!("the scenario must declare {name} as a scaled literal");
        };
        assert_eq!(*actual, value, "{name}");
    };
    expect_const("lifecycle/birth-rate", 0.0107);
    expect_const("lifecycle/rate-d-to-p", 0.0556);
    expect_const("lifecycle/rate-p-to-d-prime", 0.0213);
    expect_const("lifecycle/rate-d-prime-to-death", 0.039);
    expect_const("lifecycle/pension-coverage-rate", 0.73);
    expect_const("lifecycle/home-ownership-rate", 0.656);
    expect_const("lifecycle/ss-replacement-rate", 0.426);
    expect_const("lifecycle/healthcare-security", 0.6);
    expect_const("lifecycle/retirement-confidence", 0.5);
    expect_const("lifecycle/legit-w-home-ownership", 0.35);
    expect_const("lifecycle/legit-w-healthcare-security", 0.3);
    expect_const("lifecycle/legit-w-retirement-confidence", 0.2);
    expect_const("lifecycle/legit-w-pension-coverage", 0.1);
    expect_const("lifecycle/legit-w-ss-replacement", 0.05);
    expect_const("lifecycle/legitimation-crisis-threshold", 0.3);
    expect_const("lifecycle/legitimation-unstable-threshold", 0.5);
    expect_const("lifecycle/ideology-caregiver-weight", 0.7);
    expect_const("lifecycle/ideology-institutional-weight", 0.3);
    expect_const("lifecycle/ideology-regression-coefficient", 0.4);
}
