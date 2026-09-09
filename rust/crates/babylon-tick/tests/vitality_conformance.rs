//! Conformance vectors for `vitality/subsistence-and-death`, taken from the
//! frozen Python engine's live behaviour.
//!
//! # Provenance
//!
//! Every expected value below was printed by the frozen `VitalitySystem`
//! running one `step()` over a fixture that mirrors
//! `content/scenarios/vitality-conformance.bscn` node for node. The command,
//! from the repository root:
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/vitality_conformance.py
//! ```
//!
//! Its output on 2026-08-10, verbatim:
//!
//! ```text
//! defines (src/babylon/data/defines.yaml):
//!   economy.base_subsistence = 0.0005
//!   economy.death_threshold  = 0.001
//!
//! Grinding Attrition (the un-ported phase), verified inert here:
//!   core         attrition_rate=0.0 deaths=0
//!   bourgeoisie  attrition_rate=0.0 deaths=0
//!   hermit       attrition_rate=0.0 deaths=0
//!   last-worker  attrition_rate=0.250125 deaths=0
//!   remnant      attrition_rate=0.5 deaths=0
//!
//! post-tick state:
//!   core         active=True   population=100  wealth=999.95
//!   bourgeoisie  active=True   population=4    wealth=499.99
//!   hermit       active=True   population=1    wealth=99.9995
//!   last-worker  active=False  population=0    wealth=0.9995
//!   remnant      active=False  population=0    wealth=0.0
//!   dissolved    active=False  population=5    wealth=10
//!
//! events:
//!   entity_death {'entity_id': 'last-worker', 'wealth': 0.9995,
//!                 'consumption_needs': 2, 's_bio': 1, 's_class': 1,
//!                 'cause': 'starvation'}
//!   entity_death {'entity_id': 'remnant', 'wealth': 0.0,
//!                 'consumption_needs': 4, 's_bio': 3, 's_class': 1,
//!                 'cause': 'wealth_threshold'}
//! ```
//!
//! Two of those lines carry weight beyond their numbers. The attrition block
//! is the script's own assertion that the phase this rule pack does **not**
//! port kills nobody in this fixture — which is what lets the state below be
//! compared against the FULL frozen system rather than against a subset of
//! it. And `cause` is the one payload key that does not cross: §2.8 admits no
//! string in a payload (`E-PARSE-010`), so the discriminant would need a
//! registered closed enum. It stays recoverable — `wealth < 0.001` means
//! `wealth_threshold`, else `starvation`.
//!
//! # Why exact equality and no tolerance
//!
//! Both sides run IEEE-754 basic operations on binary64 — `+ − × ÷` and
//! comparison, correctly rounded, reproducing bit-exactly across
//! implementations (`bsl-language.rst` §4.3). No transcendental, no libm, no
//! ambiguity about accumulation order: `<arith>` is strictly binary
//! (`E-PARSE-040`), so the rule states the association Python's
//! `(base × population) × multiplier` uses rather than implying it. The
//! decimals below are Python `repr` output, i.e. the shortest round-tripping
//! decimal for each double, and Rust parses a float literal correctly
//! rounded — so `999.95_f64` IS the double Python printed. A tolerance here
//! would hide exactly the transcription error it would appear to absorb.

use babylon_bsl::evaluator::Value;
use babylon_bsl::scenario::load_scenario;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str = include_str!("../content/scenarios/vitality-conformance.bscn");
const RULE: &str = include_str!("../content/rules/vitality.bsl");

/// One subject's post-tick vector: `(scenario name, node id, active,
/// population, wealth)`.
///
/// Node ids follow scenario declaration order (`scenario.rs`: "declaration
/// order is the id order"), so the names here are the `.bscn`'s and the
/// `.py`'s, in the one order both files use.
const EXPECTED: [(&str, u64, f64, f64, f64); 6] = [
    ("core", 0, 1.0, 100.0, 999.95),
    ("bourgeoisie", 1, 1.0, 4.0, 499.99),
    ("hermit", 2, 1.0, 1.0, 99.9995),
    ("last-worker", 3, 0.0, 0.0, 0.9995),
    ("remnant", 4, 0.0, 0.0, 0.0),
    ("dissolved", 5, 0.0, 5.0, 10.0),
];

fn run() -> (MemoryGraph, CollectingSink) {
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink).expect("the Vitality pack must run");
    (graph, sink)
}

fn attribute(graph: &MemoryGraph, id: u64, field: &str) -> f64 {
    graph
        .node_attribute(NodeId(id), field)
        .unwrap_or_else(|e| panic!("node {id} field {field}: {}", e.message))
}

/// The Drain and The Reaper, against the frozen engine's own numbers.
#[test]
fn post_tick_state_matches_the_frozen_engine_exactly() {
    let (graph, _) = run();
    for (name, id, active, population, wealth) in EXPECTED {
        assert_eq!(
            attribute(&graph, id, "social-class/active"),
            active,
            "{name}: active"
        );
        assert_eq!(
            attribute(&graph, id, "social-class/population"),
            population,
            "{name}: population"
        );
        assert_eq!(
            attribute(&graph, id, "social-class/wealth"),
            wealth,
            "{name}: wealth"
        );
    }
}

/// The elite's per-head burn is five times the core class's, and both come
/// out of the same `:const` coefficient.
///
/// Asserted separately from the table because it is the whole *point* of the
/// Drain — "elites with higher subsistence multipliers burn faster when cut
/// off from imperial rent flows" (the frozen module's own words) — and a
/// table row proves it only to a reader who does the division.
#[test]
fn the_drain_scales_with_population_and_standard_of_living() {
    let (graph, _) = run();
    // FORWARD arithmetic, in the rule's own association order. Recovering
    // the cost by subtracting the post-tick wealth from the seed instead
    // would lose most of its significant digits to cancellation and then
    // fail an exact comparison for a reason that has nothing to do with the
    // port — the numbers below are exact in binary64, the difference is not.
    let core_cost = (0.0005_f64 * 100.0) * 1.0;
    let elite_cost = (0.0005_f64 * 4.0) * 5.0;
    assert_eq!(
        attribute(&graph, 0, "social-class/wealth"),
        1000.0 - core_cost
    );
    assert_eq!(
        attribute(&graph, 1, "social-class/wealth"),
        500.0 - elite_cost
    );
    assert_eq!(
        core_cost / 100.0,
        0.0005,
        "core burns base_subsistence/head"
    );
    assert_eq!(
        elite_cost / 4.0,
        0.0025,
        "the elite burns five times as much per head"
    );
}

/// A block of one that can still cover its own reproduction survives the
/// tick. Without this the death guard would look like "population == 1".
#[test]
fn a_solvent_block_of_one_survives() {
    let (graph, _) = run();
    assert_eq!(attribute(&graph, 2, "social-class/active"), 1.0);
    assert_eq!(attribute(&graph, 2, "social-class/population"), 1.0);
}

/// An inactive class is skipped whole: the guard runs before any effect, so
/// even the unconditional wealth write does not touch it.
#[test]
fn a_dissolved_class_is_untouched() {
    let (graph, _) = run();
    assert_eq!(
        attribute(&graph, 5, "social-class/wealth"),
        10.0,
        "the drain must not run on an inactive class"
    );
}

/// The Reaper's two deaths, with the payload the frozen engine emitted —
/// minus `cause`, which §2.8 gives no way to carry.
#[test]
fn the_reaper_emits_one_entity_death_per_dissolution() {
    let (_, sink) = run();
    assert_eq!(
        sink.events.len(),
        2,
        "exactly the two blocks the frozen engine killed"
    );
    for (event_type, _) in &sink.events {
        assert_eq!(event_type, "ENTITY_DEATH");
    }

    let expected: [(u64, f64, f64, f64, f64); 2] = [
        // last-worker: starvation — 0.9995 against needs of 2.
        (3, 0.9995, 2.0, 1.0, 1.0),
        // remnant: the zombie failsafe — 0.0, below death_threshold.
        (4, 0.0, 4.0, 3.0, 1.0),
    ];
    for (i, (id, wealth, needs, s_bio, s_class)) in expected.into_iter().enumerate() {
        let payload = &sink.events[i].1;
        let expected_payload = vec![
            ("entity-id".to_owned(), Value::NodeRef(NodeId(id))),
            ("wealth".to_owned(), Value::Real(wealth)),
            ("consumption-needs".to_owned(), Value::Real(needs)),
            ("s-bio".to_owned(), Value::Real(s_bio)),
            ("s-class".to_owned(), Value::Real(s_class)),
        ];
        assert_eq!(payload, &expected_payload, "payload of death {i}");
    }
}

/// Byte-determinism: the same content twice is the same post-state hash, and
/// the tick moved state at all.
#[test]
fn the_vitality_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after, "two runs, one post-state");
    assert_ne!(a.before, a.after, "the pack must move state");
    assert_eq!(a.fired, 5, "five of six subjects pass the guard");
}

/// A rule naming a coefficient the scenario never declared fails at LOAD,
/// with the coefficient named.
///
/// The defines environment and the binding vocabulary come from one place —
/// the scenario's `(defconst …)` rows — so `E-LOAD-010` catches a mistyped
/// coefficient before any subject runs, exactly as it catches a mistyped
/// field. A defaulted coefficient would be the quiet degradation §6.3
/// forbids, and one discovered at the read would report an unbound variable
/// instead of a missing define.
#[test]
fn a_rule_reading_an_undeclared_coefficient_is_refused_at_load() {
    let rule = "(rule vitality/typo \
                :role mechanic :evidence derived :material-basis \"subsistence is paid out of wealth\" :fuel 32 \
                (bindings \
                  (binding wealth :field social-class/wealth) \
                  (binding base :const economy/base-subsistance)) \
                (when (> wealth base)) \
                (effects (update-node self social-class/wealth (set base))))";
    let Err(err) = run_once(SCENARIO, rule) else {
        panic!("a mistyped coefficient must not load");
    };
    assert!(
        err.contains("economy/base-subsistance"),
        "the rejection must name the coefficient: {err}"
    );
}

/// **FIXTURE GUARD — NOT CONTENT.**
///
/// The whole conformance claim above rests on one precondition: that the
/// phase this rule pack does *not* port — Grinding Attrition — kills nobody
/// in this fixture, so the pinned post-tick state is the FULL frozen
/// system's rather than a subset's. Until this test existed, that
/// precondition was asserted only inside `vitality_conformance.py`, and
/// **nothing runs that script**: no mise task, no CI leg, no pytest
/// collection. A `.bscn` edit nudging one subject into killing range would
/// void every vector in this file while every gate stayed green — absence
/// of a check reading as success, which III.11 exists to forbid.
///
/// So the envelope is recomputed here, in the gate that already runs.
///
/// **What the formula below is, and is not.** It transcribes
/// `formulas/vitality.py::calculate_mortality_rate` and `vitality.py:253`'s
/// truncation, INSIDE A TEST, purely to bound the fixture. It is **not**
/// mechanics and does not ship: no rule reads it, no content declares it,
/// and it writes nothing. That distinction is the point — §6.2 of the plan
/// declines to transcribe this form as a MECHANIC because it is a
/// stipulated functional form with a tuned knob, and ADR175 puts its
/// emergent re-derivation behind a per-family Director review. A fixture
/// guard asserting "this world stays outside the blocked phase's reach"
/// takes no position on what that phase should eventually compute; a
/// runtime guard inside the rule would, which is why there is none.
///
/// Mutation-verified: seeding `last-worker` with `population 4` (which puts
/// `int(4 × 0.250125) == 1` inside killing range) reds this test with the
/// subject named, and restoring the seed greens it again.
#[test]
fn the_fixture_keeps_the_un_ported_phase_inert() {
    /// `vitality.attrition_base_factor`, `src/babylon/data/defines.yaml:177`.
    ///
    /// A literal here rather than a `(defconst …)` row because no rule in
    /// this pack reads it: putting it in the scenario would ship a
    /// coefficient for a mechanic that does not exist. It belongs to the
    /// guard, so it lives with the guard.
    const ATTRITION_BASE_FACTOR: f64 = 0.5;

    let mut seeds = MemoryGraph::new();
    let scenario = load_scenario(SCENARIO, &mut seeds).expect("the scenario must load");
    let Some(Value::Real(base_subsistence)) = scenario.consts.get("economy/base-subsistence")
    else {
        panic!("the scenario must declare economy/base-subsistence as a scaled literal");
    };

    for (name, id, ..) in EXPECTED {
        let seed = |field: &str| {
            seeds
                .node_attribute(NodeId(id), field)
                .unwrap_or_else(|e| panic!("{name}: {}", e.message))
        };
        let population = seed("social-class/population");
        // The frozen loop's two `continue`s: an inactive or empty class
        // never reaches Phase 2 at all.
        if seed("social-class/active") == 0.0 || population <= 0.0 {
            continue;
        }
        let cost = (base_subsistence * population) * seed("social-class/subsistence-multiplier");
        let wealth = seed("social-class/wealth");
        let drained = if wealth > cost { wealth - cost } else { 0.0 };
        let needs = seed("social-class/s-bio") + seed("social-class/s-class");
        let inequality = seed("social-class/inequality");

        let coverage = (drained / population) / needs;
        let threshold = 1.0 + inequality;
        let rate = if coverage >= threshold {
            0.0
        } else {
            ((threshold - coverage) * (ATTRITION_BASE_FACTOR + inequality)).clamp(0.0, 1.0)
        };
        // `int(population * rate)` — truncation toward zero, as Python's
        // `int()` does on a non-negative float.
        let deaths = (population * rate).trunc();

        assert_eq!(
            deaths, 0.0,
            "{name} loses {deaths} member(s) to Grinding Attrition (rate {rate}). \
             The pack does not port that phase, so a fixture where it fires \
             cannot carry an exact conformance vector — either move the seed \
             back inside the envelope or stop claiming an exact match."
        );
    }
}
