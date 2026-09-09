//! The multi-rule conformance vector: `vitality/subsistence-and-death` and
//! `lifecycle/dpd-circuit` run TOGETHER, in ONE content set (Program 28 B2,
//! Phase A Task 5).
//!
//! # What this proves
//!
//! 1. **The executable phase registry orders rules by governed system slot,**
//!    not by declaration order or global rule-id bytes. Vitality @1 runs
//!    before Lifecycle @7 even though `'l' < 'v'`. The second test proves
//!    both source permutations produce a byte-identical `TickReport`,
//!    including `per_rule_fired`'s resolved order.
//! 2. **The phase-ordered result reproduces the frozen engine's own combined
//!    output** for this pair. The rules' domains are disjoint (the plan's
//!    Multi-Rule Decision section), which
//!    `vitality_lifecycle_combined_conformance.py` proves EMPIRICALLY (not
//!    just by reading the bindings): it runs both engine-order and
//!    reverse-order against two independently-built copies of the same
//!    ten-node state and diffs the results field-for-field. Its own output
//!    printed `MATCH` — engine order and reverse order are byte-identical.
//!
//! # Provenance
//!
//! Every expected value below was printed by
//! `vitality_lifecycle_combined_conformance.py`. The command, from the
//! repository root:
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/vitality_lifecycle_combined_conformance.py
//! ```
//!
//! Its output on 2026-08-11 (post-tick state section, verbatim) matches
//! `lifecycle_conformance.py`'s own four-territory output and
//! `vitality_conformance.py`'s own six-social-class output EXACTLY,
//! field for field — the union of two already-proven fixtures produces
//! the union of their already-proven post-tick vectors, unperturbed by
//! running together. Node ids follow scenario declaration order
//! (`scenario.rs`), and this scenario declares the four territories
//! FIRST, then the six social classes: `core-county`=0,
//! `growing-county`=1, `recovering-county`=2, `young-county`=3, `core`=4,
//! `bourgeoisie`=5, `hermit`=6, `last-worker`=7, `remnant`=8,
//! `dissolved`=9.
//!
//! # Why exact equality and no tolerance
//!
//! Both sides run IEEE-754 basic operations on binary64 — `+ − × ÷` and
//! comparison, correctly rounded, reproducing bit-exactly across
//! implementations (`bsl-language.rst` §4.3). The decimals below are
//! Python `repr` output, i.e. the shortest round-tripping decimal for each
//! double, and Rust parses a float literal correctly rounded — so e.g.
//! `999.95_f64` IS the double Python printed.

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::run_once_into; // `hex` is not needed here — neither test formats a hash

const SCENARIO: &str =
    include_str!("../content/scenarios/vitality-lifecycle-combined-conformance.bscn");
const VITALITY: &str = include_str!("../content/rules/vitality.bsl");
const LIFECYCLE: &str = include_str!("../content/rules/lifecycle.bsl");

/// One territory subject's post-tick vector.
struct ExpectedTerritory {
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

const EXPECTED_TERRITORIES: [ExpectedTerritory; 4] = [
    ExpectedTerritory {
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
    ExpectedTerritory {
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
    ExpectedTerritory {
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
    ExpectedTerritory {
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

/// One social-class subject's post-tick vector: `(name, id, active,
/// population, wealth)`.
const EXPECTED_SOCIAL_CLASSES: [(&str, u64, f64, f64, f64); 6] = [
    ("core", 4, 1.0, 100.0, 999.95),
    ("bourgeoisie", 5, 1.0, 4.0, 499.99),
    ("hermit", 6, 1.0, 1.0, 99.9995),
    ("last-worker", 7, 0.0, 0.0, 0.9995),
    ("remnant", 8, 0.0, 0.0, 0.0),
    ("dissolved", 9, 0.0, 5.0, 10.0),
];

fn attribute(graph: &HypergraphStore, id: u64, field: &str) -> f64 {
    graph
        .node_attribute(NodeId(id), field)
        .unwrap_or_else(|e| panic!("node {id} field {field}: {}", e.message))
}

fn assert_post_tick_state_matches(graph: &HypergraphStore) {
    for e in &EXPECTED_TERRITORIES {
        assert_eq!(
            attribute(graph, e.id, "territory/pop-d"),
            e.pop_d,
            "{}: pop-d",
            e.name
        );
        assert_eq!(
            attribute(graph, e.id, "territory/pop-p"),
            e.pop_p,
            "{}: pop-p",
            e.name
        );
        assert_eq!(
            attribute(graph, e.id, "territory/pop-d-prime"),
            e.pop_d_prime,
            "{}: pop-d-prime",
            e.name
        );
        assert_eq!(
            attribute(graph, e.id, "territory/wealth-d-prime"),
            e.wealth_d_prime,
            "{}: wealth-d-prime",
            e.name
        );
        assert_eq!(
            attribute(graph, e.id, "territory/dependency-ratio"),
            e.dependency_ratio,
            "{}: dependency-ratio",
            e.name
        );
        assert_eq!(
            attribute(graph, e.id, "territory/legitimation-index"),
            e.legitimation_index,
            "{}: legitimation-index",
            e.name
        );
        assert_eq!(
            attribute(graph, e.id, "territory/legitimation-crisis"),
            e.legitimation_crisis,
            "{}: legitimation-crisis",
            e.name
        );
        assert_eq!(
            attribute(graph, e.id, "territory/transmitted-ideology"),
            e.transmitted_ideology,
            "{}: transmitted-ideology",
            e.name
        );
    }
    for &(name, id, active, population, wealth) in &EXPECTED_SOCIAL_CLASSES {
        assert_eq!(
            attribute(graph, id, "social-class/active"),
            active,
            "{name}: active"
        );
        assert_eq!(
            attribute(graph, id, "social-class/population"),
            population,
            "{name}: population"
        );
        assert_eq!(
            attribute(graph, id, "social-class/wealth"),
            wealth,
            "{name}: wealth"
        );
    }
}

#[test]
fn governed_phase_order_reproduces_the_frozen_engine() {
    // Concatenation order here is arbitrary on purpose (vitality text
    // first) — the second test below proves the OTHER concatenation order
    // gives an identical report, which is the actual claim this task makes.
    let rule_src = format!("{VITALITY}\n{LIFECYCLE}");
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, &rule_src, &mut graph, &mut sink).expect("tick");

    // THE ORDER PROOF — the phase registry puts Vitality @1 before
    // Lifecycle @7 even though the rule ids sort in the opposite order.
    // The final hash would not distinguish the two for this disjoint pair,
    // so per_rule_fired's own order is the load-bearing assertion.
    assert_eq!(report.per_rule_fired.len(), 2);
    assert_eq!(report.per_rule_fired[0].0, "vitality/subsistence-and-death");
    assert_eq!(report.per_rule_fired[1].0, "lifecycle/dpd-circuit");
    // Exact counts, pinned from the Python reference: lifecycle fires
    // unconditionally on every territory (no `(when …)` guard on the
    // rule); vitality fires on 5 of 6 social classes (`dissolved` fails
    // the `(and (= active 1) (> population 0))` guard) — the SAME counts
    // vitality-conformance.bscn's and lifecycle-conformance.bscn's own
    // individually-pinned tests already assert, unchanged by union.
    assert_eq!(report.per_rule_fired[0].1, 5, "vitality fired count");
    assert_eq!(report.per_rule_fired[1].1, 4, "lifecycle fired count");

    // Per-node field values match the Python reference's printed output —
    // proven order-invariant by that script's own engine-order vs.
    // reverse-order diff (printed `MATCH`).
    assert_post_tick_state_matches(&graph);
}

#[test]
fn file_order_is_never_observable_per_section_4_2() {
    // The actual promise §4.2/D16 makes, now a committed test: two content
    // sets built from the SAME two rules in DIFFERENT concatenation order
    // must produce BYTE-IDENTICAL TickReports — not merely the same hash,
    // the same report in full, including per_rule_fired's order (which
    // must be IDENTICAL, not flipped, because the driver resolves the
    // governed phase order rather than preserving file order).
    let forward = format!("{VITALITY}\n{LIFECYCLE}");
    let reversed = format!("{LIFECYCLE}\n{VITALITY}");

    let mut graph_a = HypergraphStore::new();
    let mut sink_a = CollectingSink::default();
    let report_a = run_once_into(SCENARIO, &forward, &mut graph_a, &mut sink_a).expect("tick a");

    let mut graph_b = HypergraphStore::new();
    let mut sink_b = CollectingSink::default();
    let report_b = run_once_into(SCENARIO, &reversed, &mut graph_b, &mut sink_b).expect("tick b");

    assert_eq!(report_a.before, report_b.before);
    assert_eq!(report_a.after, report_b.after);
    assert_eq!(report_a.fired, report_b.fired);
    assert_eq!(
        report_a.per_rule_fired, report_b.per_rule_fired,
        "file/concatenation order must never be observable in the report — §4.2"
    );
}
