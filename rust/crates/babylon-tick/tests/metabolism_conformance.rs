//! Conformance vectors for `metabolism/biocapacity-update`, taken from the
//! frozen Python engine's live behaviour.
//!
//! # Provenance
//!
//! Every state value below was printed by the frozen `MetabolismSystem`
//! running one `step()` over a fixture that mirrors
//! `content/scenarios/metabolism-conformance.bscn` node for node. The
//! command, from the repository root:
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/metabolism_conformance.py
//! ```
//!
//! Its output on 2026-08-11, verbatim:
//!
//! ```text
//! defines (src/babylon/data/defines.yaml, metabolism: section):
//!   metabolism.entropy_factor = 1.2
//!   metabolism.overshoot_threshold = 1.0
//!   metabolism.hysteresis_rate = 0.005
//!   metabolism.max_overshoot_ratio = 999.0
//!
//! post-tick state:
//!   nominal-county           biocapacity=1.0 max_biocapacity=99.975 (seed biocapacity=5.0 seed max_biocapacity=100.0)
//!   hysteresis-inert-county  biocapacity=52.0 max_biocapacity=100.0 (seed biocapacity=50.0 seed max_biocapacity=100.0)
//!   zero-floor-county        biocapacity=0.0 max_biocapacity=99.5 (seed biocapacity=100.0 seed max_biocapacity=100.0)
//!
//! events:
//!   (none)
//! ```
//!
//! This pack ports only Phase 1 of the frozen system (per-territory
//! biocapacity delta + hysteresis ratchet + double clamp) — the spec-070
//! sovereign pre-pass and Phases 2-3 (global overshoot) are BLOCKED, per
//! `reports/metabolism-port-assessment-2026-08-11.md`. No event is ever
//! emitted by this rule (the frozen Phase 1 loop publishes nothing).
//!
//! # Why exact equality and no tolerance — and where that claim STOPS
//! applying
//!
//! Both sides run IEEE-754 basic operations on binary64 — `+ − × ÷` and
//! comparison, correctly rounded, reproducing bit-exactly across
//! implementations for any term where BOTH engines perform the SAME
//! operation sequence (`bsl-language.rst` §4.3): the decimals below are
//! Python `repr` output — the shortest round-tripping decimal for each
//! double — and Rust parses a float literal correctly rounded, so e.g.
//! `99.975_f64` IS the double Python printed. `regeneration`, `damage`
//! (the hysteresis path), and both clamps hold to this exactly, because
//! this pack's binding chain performs the IDENTICAL sequence of `+ − × ÷`
//! the frozen formulas do for those terms.
//!
//! **This does NOT extend to `ecological-cost`.** The frozen engine
//! computes `raw_extraction * entropy_factor` as ONE multiply; this pack's
//! D-1 workaround computes `(raw_extraction * entropy_factor_x1e6) /
//! 1000000` — a DIFFERENT operation sequence for the same real-valued
//! function, which can double-round to an adjacent double (see
//! `metabolism.bsl`'s own D-1 and
//! `metabolism_rounding_divergence_conformance.rs`, which PINS a case
//! where this happens). The vectors in THIS file's fixture
//! (`nominal-county`'s `raw_extraction=5`, `entropy_factor=1.2`) happen to
//! land on a value where the two operation sequences agree bit for bit —
//! confirmed by execution, not assumed — but that agreement is a property
//! of these SPECIFIC inputs, not a general guarantee this pack's D-1
//! workaround provides.

use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str = include_str!("../content/scenarios/metabolism-conformance.bscn");
const RULE: &str = include_str!("../content/rules/metabolism.bsl");

fn run() -> MemoryGraph {
    let mut graph = MemoryGraph::new();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink).expect("the Metabolism pack must run");
    graph
}

fn attribute(graph: &MemoryGraph, id: u64, field: &str) -> f64 {
    graph
        .node_attribute(NodeId(id), field)
        .unwrap_or_else(|e| panic!("node {id} field {field}: {}", e.message))
}

/// The nominal territory: a small stock, full extraction, neither clamp
/// binds. `biocapacity=1.0` (`5.0 + delta`, `delta = 2.0 (regeneration) -
/// 6.0 (ecological cost) = -4.0`, `5.0 - 4.0 = 1.0`); `max_biocapacity =
/// 99.975` (`100.0 - 0.025` damage — `raw_extraction = 1*5 = 5`,
/// `damage = 5 * 0.005 = 0.025`).
///
/// Mutation-verified: changing `metabolism.bsl`'s D-1 descaling divisor
/// from `1000000` to `100000` (a 10x scale bug in the `entropy_factor`
/// workaround) flips this test's `biocapacity` from `1.0` to `0.0` — the
/// same mutation also flips `metabolism_entropy_low_conformance.rs`'s own
/// vector but NOT `metabolism_entropy_high_conformance.rs`'s (the floor
/// already binds there at the true `entropy_factor = 3.0`, and stays bound
/// at the mutated effective `30.0`) — verified by hand during authoring,
/// reverted before commit.
#[test]
fn the_nominal_territory_matches_the_frozen_engine_exactly() {
    let graph = run();
    assert_eq!(attribute(&graph, 0, "territory/biocapacity"), 1.0);
    assert_eq!(attribute(&graph, 0, "territory/max-biocapacity"), 99.975);
}

/// `extraction_intensity = 0`: `raw_extraction = 0`, so BOTH the ecological
/// cost AND the hysteresis damage are exactly zero — pure regeneration
/// (`biocapacity = 50 + 0.02*100 = 52.0`), and `max_biocapacity` is
/// UNCHANGED bit for bit from its seed (`100.0`), proving the hysteresis
/// ratchet is genuinely inert here, not merely small.
#[test]
fn zero_extraction_leaves_hysteresis_completely_inert() {
    let graph = run();
    assert_eq!(attribute(&graph, 1, "territory/biocapacity"), 52.0);
    assert_eq!(
        attribute(&graph, 1, "territory/max-biocapacity"),
        100.0,
        "max_biocapacity must be bit-identical to its seed when extraction_intensity is 0"
    );
}

/// The territory at its own ceiling (`biocapacity == max_biocapacity`, so
/// the frozen formula's `if current_biocapacity >= max_biocapacity:
/// regeneration = 0.0` suppresses regeneration entirely) with full
/// extraction: the ecological cost alone drives `current + delta` deeply
/// negative (`100 - 120 = -20`), and `max(0.0, ...)` floors it to EXACTLY
/// `0.0` — not merely close to zero, not negative.
///
/// Mutation-verified (twice, by hand during authoring, reverted before
/// commit): flipping `metabolism.bsl`'s `new-biocapacity` binding's
/// comparator from `>` to `<` (so the floor clamp picks the unclamped
/// negative value instead of `0.0`) flips this test to `-20.0`, and also
/// flips `the_nominal_territory_matches_the_frozen_engine_exactly` and
/// `zero_extraction_leaves_hysteresis_completely_inert`. Separately,
/// flipping the `regeneration` binding's `>= current max-cap` guard to
/// `< current max-cap` (so regeneration fires AT the ceiling instead of
/// being suppressed there) does NOT flip this specific test — both
/// mutated and original regeneration values are still deeply negative
/// once floored — but DOES flip `the_nominal_territory_matches_the_frozen_
/// engine_exactly` and `zero_extraction_leaves_hysteresis_completely_
/// inert`, so the branch is still covered by this suite as a whole.
#[test]
fn heavy_extraction_at_the_ceiling_floors_biocapacity_at_exactly_zero() {
    let graph = run();
    assert_eq!(attribute(&graph, 2, "territory/biocapacity"), 0.0);
}

/// The SAME territory's `max_biocapacity` strictly decreases from its seed
/// (`100.0` -> `99.5`) — proving the hysteresis ratchet
/// (`new_max = max(0.0, max_cap - damage)`) is live and non-trivial,
/// independent of whether the floor or ceiling clamp binds for
/// `biocapacity` itself.
#[test]
fn heavy_extraction_permanently_damages_the_ceiling() {
    let graph = run();
    let max_biocapacity = attribute(&graph, 2, "territory/max-biocapacity");
    assert_eq!(max_biocapacity, 99.5);
    assert!(
        max_biocapacity < 100.0,
        "the ratchet must strictly lower the ceiling below its seed"
    );
}

/// Byte-determinism: the same content twice is the same post-state hash,
/// and the tick moved state at all.
#[test]
fn the_metabolism_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after, "two runs, one post-state");
    assert_ne!(a.before, a.after, "the pack must move state");
    assert_eq!(
        a.fired, 3,
        "all three territories are unconditional (no when guard)"
    );
}

/// No event fires — the frozen Phase 1 loop publishes nothing (unlike
/// Dispossession's two emits); Phases 2-3 (`ECOLOGICAL_OVERSHOOT`) are
/// BLOCKED and not ported (see the module doc and this pack's D-4).
#[test]
fn no_event_fires() {
    let mut graph = MemoryGraph::new();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink).expect("the pack must run");
    assert!(sink.events.is_empty(), "Phase 1 publishes no event");
}

/// A rule reading a coefficient the scenario never declared fails at LOAD,
/// with the coefficient named — the same discipline every other landed
/// pack's own conformance suite pins.
#[test]
fn a_rule_reading_an_undeclared_coefficient_is_refused_at_load() {
    let rule = "(rule metabolism/typo \
                :role mechanic :evidence derived :material-basis \"a territory has a biocapacity\" :fuel 32 \
                (bindings \
                  (binding current :field territory/biocapacity) \
                  (binding rate :const metabolism/regeneration-rat)) \
                (effects (update-node self territory/biocapacity (set (* rate current)))))";
    let Err(err) = run_once(SCENARIO, rule) else {
        panic!("a mistyped coefficient must not load");
    };
    assert!(
        err.contains("metabolism/regeneration-rat"),
        "the rejection must name the coefficient: {err}"
    );
}
