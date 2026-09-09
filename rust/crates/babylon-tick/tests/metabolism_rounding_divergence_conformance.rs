//! PINS the D-1 scaled-`Int` workaround's numeric DEVIATION from the frozen
//! engine (F1 fix round, adversarial review of PR #501). An earlier
//! revision of `metabolism.bsl`'s D-1 claimed the workaround "preserves the
//! formula's exact value ... for ANY legal (1.0, 3.0] modded value" —
//! FALSE, disproved by execution. This suite asserts what the engine
//! ACTUALLY computes, including the exact bit pattern where it diverges
//! from the frozen Python.
//!
//! # Provenance
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/metabolism_rounding_divergence_conformance.py
//! ```
//!
//! Its output on 2026-08-11, verbatim:
//!
//! ```text
//! frozen engine (MetabolismSystem.step, unmodified):
//!   biocapacity     = 1.4000000000000004  3ff6666666666668
//!   max_biocapacity = 99.985  4058ff0a3d70a3d7
//!
//! this pack's value (pure-Python replica of metabolism.bsl):
//!   biocapacity     = 1.4  3ff6666666666666
//!   max_biocapacity = 99.985  4058ff0a3d70a3d7
//!
//! biocapacity equal:     False
//! max_biocapacity equal: True
//! ```
//!
//! # Why `biocapacity` diverges and `max_biocapacity` does not
//!
//! `biocapacity`'s computation routes through `ecological-cost`, which
//! uses the D-1 workaround (`(raw-extraction * entropy-factor-x1e6) /
//! 1000000` — an exact integer multiply then a correctly-rounded division)
//! where the frozen engine performs ONE binary64 multiply
//! (`raw_extraction * entropy_factor`) instead. `max_biocapacity`'s
//! computation (the hysteresis damage) never touches `entropy_factor` at
//! all — `damage = raw_extraction * hysteresis_rate`, a plain multiply on
//! both sides — so it is bit-identical.
//!
//! `0x3ff6666666666666` (this pack) versus `0x3ff6666666666668` (frozen
//! Python) differ by exactly `2` in the raw bit pattern — 2 ULP, for a
//! `biocapacity=3` seed where `entropy_factor` is at the shipped default
//! `1.2` (grid-quantization error is exactly ZERO here — `1200000 / 1e6
//! == 1.2` exactly as a real number, so this is purely the double-rounding
//! residual `metabolism.bsl`'s D-1 derives).

use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str =
    include_str!("../content/scenarios/metabolism-rounding-divergence-conformance.bscn");
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

/// `biocapacity` is `1.4` exactly (`0x3ff6666666666666`) — NOT the frozen
/// engine's `1.4000000000000004` (`0x3ff6666666666668`). Asserted on the
/// raw bit pattern, not just the decimal `==`, so a future accidental
/// "fix" that happens to print the same short decimal cannot silently pass.
#[test]
fn biocapacity_is_this_packs_value_not_the_frozen_engines() {
    let graph = run();
    let bio = attribute(&graph, 0, "territory/biocapacity");
    assert_eq!(bio, 1.4);
    assert_eq!(
        bio.to_bits(),
        0x3ff6_6666_6666_6666,
        "this pack's value, exactly 2 ULP below the frozen engine's 0x3ff6666666666668"
    );
    assert_ne!(
        bio.to_bits(),
        0x3ff6_6666_6666_6668,
        "must NOT match the frozen engine's bit pattern — that would mean the double-rounding \
         deviation stopped reproducing, which is worth investigating, not silently accepting"
    );
}

/// `max_biocapacity` — the hysteresis damage path — never touches
/// `entropy_factor` and so is bit-identical to the frozen engine's
/// `99.985`, confirming the D-1 workaround's deviation is confined to the
/// ecological-cost term exactly as derived.
#[test]
fn max_biocapacity_matches_the_frozen_engine_exactly() {
    let graph = run();
    let max = attribute(&graph, 0, "territory/max-biocapacity");
    assert_eq!(max, 99.985);
    assert_eq!(max.to_bits(), 0x4058_ff0a_3d70_a3d7);
}

/// Byte-determinism: the deviation is reproducible, which is what III.7
/// actually requires — not bit-parity with the frozen Python reference.
#[test]
fn the_rounding_divergence_scenario_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after);
    assert_eq!(a.fired, 1);
}
