//! Conformance vectors for `solidarity/p0-transmit` (the Solidarity @8.0
//! port train, frozen `src/babylon/engine/systems/solidarity.py`, issue
//! #557 umbrella) — `docs/superpowers/plans/2026-08-17-solidarity-port.md`.
//!
//! # Task 2 scope (values)
//!
//! Task 1 landed the loadable namespace and the conformance world
//! (`content/scenarios/solidarity-conformance.bscn`) behind a never-firing
//! PROBE rule. Task 2 replaced that probe with the real
//! `content/rules/solidarity.bsl` and pins the post-tick
//! `social-class/revolutionary` value on every witness target: the plain
//! transmission, the three MASS_AWAKENING-adjacent sub-cases (crossing,
//! staying below, landing exactly at the threshold — the latter checked by
//! BIT-EXACT equality against the `solidarity/mass-awakening-threshold`
//! `:const`, a Sterbenz's-lemma equality this task is the first to read
//! back rather than merely reason about), the three skip gates
//! (`strength <= 0`, source at/below `activation_threshold`,
//! `|delta| < negligible_transmission`), the multi-inbound last-write-wins
//! divergence (D-record 2: 0.478 frozen vs 0.31 ported), the inactive
//! -source and inactive-target skips, and the clamp's upper bound.
//!
//! # Task 3 scope (events)
//!
//! Task 3 (`.superpowers/sdd/2026-08-17-solidarity-port/task-3-brief.md`)
//! adds the two emits to `solidarity.bsl` and pins `sink.events` here:
//! `CONSCIOUSNESS_TRANSMISSION` on every applied transmission,
//! `MASS_AWAKENING` only on an upward crossing of the mass-awakening
//! threshold (the frozen chained comparison's asymmetric `<`/`<=` arms,
//! `solidarity.py:190`). The full nine-event ordered list is pinned by
//! exact `Value::NodeRef`/`Value::Real` payload equality, in the shape of
//! `vitality_conformance.rs::the_reaper_emits_one_entity_death_per_dissolution`;
//! the negative case (raises but does not cross 0.6) and the exact-0.6
//! boundary crossing each additionally get their own isolated assertion.
//!
//! # Provenance
//!
//! Values are hand-computed from `delta = strength * (source_r - target_r)`,
//! `new = clamp01(target_r + delta)` (plan §2.6), using FORWARD f64
//! arithmetic in the rule's own association order wherever a witness's
//! inputs are not exact dyadic rationals (the multi-inbound witness's
//! 0.9/0.8/0.1/0.3 seeds) — the same discipline
//! `vitality_conformance.rs::the_drain_scales_with_population_and_standard_of_living`
//! uses, for the same reason: hand-rounding a decimal literal risks a
//! transcription error unrelated to the port, where recomputing the exact
//! IEEE-754 expression the rule evaluates cannot diverge from it.
//!
//! # Task 4 scope (the dual-implementation oracle)
//!
//! Task 4 (`.superpowers/sdd/2026-08-17-solidarity-port/task-4-brief.md`)
//! adds `content/scenarios/solidarity_conformance.py` — a STANDALONE,
//! dependency-free Python script (no `babylon` import, no pytest) that
//! transcribes `solidarity.bsl`'s own binding order and collect-then-apply
//! semantics term-for-term over a literal `WORLD` dict matching this
//! `.bscn` node-for-node, seed-for-seed. **This is the oracle, not the
//! frozen engine** (ADR183 + D146 precedent): the frozen
//! `SolidaritySystem` applies each edge's delta sequentially, in place, so
//! literally rerunning it would print the multi-inbound witness's FROZEN
//! answer (0.478), not the port's own accepted answer (0.31, D-record 2).
//! Reconciled by hand against every literal pinned above and in
//! `tick_goldens.rs::solidarity_conformance_hashes_are_pinned`: every value
//! agreed exactly at the time this oracle was generated (2026-08-17).
//!
//! **Post-repair note (#491 T1 S1, Director sitting 2026-08-18):**
//! `solidarity.bsl`'s `p0-transmit` write now uses a convex-combination
//! form for kind-coherence (`(1 - strength) * target + strength * source`,
//! algebraically identical to `target + delta` above but not bit-identical
//! for a non-power-of-2 `strength`). The multi-inbound witness's
//! `strength = 0.3` is exactly that case: `multi-target`'s pinned value
//! above (`id=15 = 0.31000000000000005`) and events 7-8's
//! `new-target-consciousness` (`0.33999999999999997` / `0.31000000000000005`)
//! are now stale by one ULP each — the port's actual outputs are
//! bit-identical to the `f64` literals `0.34`/`0.31` (each the 1-ULP
//! neighbor of the old form's value; "exact" here means literal bit
//! equality, not decimal representability). This oracle file (`solidarity_conformance.py`)
//! is NOT regenerated for this repair (it transcribes the OLD `target +
//! delta` form on purpose, as the frozen-adjacent reference the repair is
//! measured against); the live Rust assertions below are the ones updated,
//! at `events_land_in_declared_order_with_full_pinned_payloads` and
//! `multi_inbound_edges_diverge_from_the_frozen_sequential_apply`. Every
//! OTHER value above is unaffected (all other witnesses use power-of-2
//! strengths, bit-identical either way — verified,
//! reports/kind-straddle-repair-options-2026-08-18.md §2.1).
//!
//! Regenerate with, from the repository root:
//!
//! ```text
//! uv run python rust/crates/babylon-tick/content/scenarios/solidarity_conformance.py
//! ```
//!
//! Exact stdout (2026-08-17, `UV_FROZEN=1 uv run python
//! rust/crates/babylon-tick/content/scenarios/solidarity_conformance.py`
//! from the repository root):
//!
//! ```text
//! defines (config/defines/consciousness.py:23-39):
//!   solidarity/activation-threshold      = 0.3
//!   solidarity/mass-awakening-threshold   = 0.6
//!   solidarity/negligible-transmission    = 0.01
//!
//! fired-count table (guard-passed subjects per rule):
//!   solidarity/p0-transmit = 14
//!   total                  = 14
//!
//! post-tick social-class/revolutionary (repr):
//!   plain-source                 id=0   = 0.5
//!   plain-target                 id=1   = 0.375
//!   awaken-source                id=2   = 0.875
//!   mass-awaken-cross-target     id=3   = 0.71875
//!   mass-awaken-stays-target     id=4   = 0.546875
//!   mass-awaken-exact-source     id=5   = 0.6
//!   mass-awaken-exact-target     id=6   = 0.6
//!   zero-strength-source         id=7   = 0.75
//!   zero-strength-target         id=8   = 0.25
//!   at-threshold-source          id=9   = 0.3
//!   at-threshold-target          id=10  = 0.25
//!   negligible-source            id=11  = 0.5
//!   negligible-target            id=12  = 0.4375
//!   multi-source-a               id=13  = 0.9
//!   multi-source-b               id=14  = 0.8
//!   multi-target                 id=15  = 0.31000000000000005
//!   inactive-source              id=16  = 0.9
//!   inactive-source-target       id=17  = 0.25
//!   inactive-target-source       id=18  = 0.9
//!   inactive-target              id=19  = 0.25
//!   clamp-source                 id=20  = 1.0
//!   clamp-target                 id=21  = 1.0
//!
//! events (9):
//!   1. CONSCIOUSNESS_TRANSMISSION source-id=0 target-id=1 delta=0.125 solidarity-strength=0.5 source-consciousness=0.5 old-target-consciousness=0.25 new-target-consciousness=0.375
//!   2. CONSCIOUSNESS_TRANSMISSION source-id=2 target-id=3 delta=0.15625 solidarity-strength=0.5 source-consciousness=0.875 old-target-consciousness=0.5625 new-target-consciousness=0.71875
//!   3. MASS_AWAKENING target-id=3 old-consciousness=0.5625 new-consciousness=0.71875 triggering-source=2
//!   4. CONSCIOUSNESS_TRANSMISSION source-id=2 target-id=4 delta=0.046875 solidarity-strength=0.125 source-consciousness=0.875 old-target-consciousness=0.5 new-target-consciousness=0.546875
//!   5. CONSCIOUSNESS_TRANSMISSION source-id=5 target-id=6 delta=0.09999999999999998 solidarity-strength=1.0 source-consciousness=0.6 old-target-consciousness=0.5 new-target-consciousness=0.6
//!   6. MASS_AWAKENING target-id=6 old-consciousness=0.5 new-consciousness=0.6 triggering-source=5
//!   7. CONSCIOUSNESS_TRANSMISSION source-id=13 target-id=15 delta=0.24 solidarity-strength=0.3 source-consciousness=0.9 old-target-consciousness=0.1 new-target-consciousness=0.33999999999999997
//!   8. CONSCIOUSNESS_TRANSMISSION source-id=14 target-id=15 delta=0.21000000000000002 solidarity-strength=0.3 source-consciousness=0.8 old-target-consciousness=0.1 new-target-consciousness=0.31000000000000005
//!   9. CONSCIOUSNESS_TRANSMISSION source-id=20 target-id=21 delta=0.25 solidarity-strength=2.0 source-consciousness=1.0 old-target-consciousness=0.875 new-target-consciousness=1.0
//! ```

use babylon_bsl::evaluator::Value;
use babylon_bsl::scenario::load_scenario;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into, TickReport};

const SCENARIO: &str = include_str!("../content/scenarios/solidarity-conformance.bscn");
const RULE: &str = include_str!("../content/rules/solidarity.bsl");

// Node ids, in the `.bscn`'s own declaration order (its header's node
// census table is the single source of truth for this mapping — kept in
// sync there, not duplicated here beyond the names).
const PLAIN_SOURCE: u64 = 0;
const PLAIN_TARGET: u64 = 1;
const AWAKEN_SOURCE: u64 = 2;
const MASS_AWAKEN_CROSS_TARGET: u64 = 3;
const MASS_AWAKEN_STAYS_TARGET: u64 = 4;
const MASS_AWAKEN_EXACT_SOURCE: u64 = 5;
const MASS_AWAKEN_EXACT_TARGET: u64 = 6;
const ZERO_STRENGTH_SOURCE: u64 = 7;
const ZERO_STRENGTH_TARGET: u64 = 8;
const AT_THRESHOLD_SOURCE: u64 = 9;
const AT_THRESHOLD_TARGET: u64 = 10;
const NEGLIGIBLE_SOURCE: u64 = 11;
const NEGLIGIBLE_TARGET: u64 = 12;
const MULTI_SOURCE_A: u64 = 13;
const MULTI_SOURCE_B: u64 = 14;
const MULTI_TARGET: u64 = 15;
const INACTIVE_SOURCE: u64 = 16;
const INACTIVE_SOURCE_TARGET: u64 = 17;
const INACTIVE_TARGET_SOURCE: u64 = 18;
const INACTIVE_TARGET: u64 = 19;
const CLAMP_SOURCE: u64 = 20;
const CLAMP_TARGET: u64 = 21;

/// Runs the real pack once and hands back the graph, the sink (the event
/// tests read it — CONSCIOUSNESS_TRANSMISSION and MASS_AWAKENING land
/// there), and the `TickReport` (`fired`, the pre/post hashes). Each test calls this
/// independently — one `run_once_into` call PER TEST (deliberate isolation;
/// cost is negligible at this world size), so within a test no assertion
/// re-derives what its own run already computed.
fn run() -> (MemoryGraph, CollectingSink, TickReport) {
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, RULE, &mut graph, &mut sink)
        .expect("the Solidarity conformance world must load and run against solidarity.bsl");
    (graph, sink, report)
}

fn revolutionary(graph: &MemoryGraph, id: u64) -> f64 {
    graph
        .node_attribute(NodeId(id), "social-class/revolutionary")
        .unwrap_or_else(|e| panic!("node {id} social-class/revolutionary: {}", e.message))
}

/// The world loads through the real `run_once_into` seam, against the real
/// `solidarity/p0-transmit` rule (not Task 1's probe), and its node/edge
/// census still matches the `.bscn`'s declaration: 22 `SOCIAL_CLASS` nodes
/// and 12 `SOLIDARITY` edges.
#[test]
fn the_conformance_world_loads_with_the_declared_census() {
    let mut probe_graph = MemoryGraph::new();
    let loaded = load_scenario(SCENARIO, &mut probe_graph).expect("the scenario must load clean");
    assert_eq!(loaded.node_count, 22, "22 SOCIAL_CLASS witness nodes");
    assert_eq!(loaded.edge_count, 12, "12 SOLIDARITY witness edges");
    assert_eq!(
        loaded.node_types.get("SOCIAL_CLASS").copied(),
        Some(22),
        "every declared node is a SOCIAL_CLASS (the plan's single-subject-type ruling, §2.1)"
    );
    assert_eq!(
        loaded.edge_types.get("SOLIDARITY").copied(),
        Some(12),
        "every declared edge is SOLIDARITY"
    );

    // `fired` counts SUBJECTS whose (when …) passed — every SOCIAL_CLASS
    // node with active=1 and revolutionary > activation_threshold (0.3c),
    // regardless of whether it has any outbound SOLIDARITY edge to push
    // through (`negligible-target`, `clamp-target`, … each fire as their
    // own subject with an empty for-each, per `tick.rs::collect_pass`'s own
    // per-subject accounting). 14 of the 22 witness nodes clear the gate.
    let (_graph, _sink, report) = run();
    assert_eq!(
        report.fired, 14,
        "14 of 22 witness nodes have active=1 and revolutionary > 0.3"
    );
}

/// Witness 1 — plain transmission: source above threshold, target below,
/// differing r, no clamp, no mass awakening. delta = 0.5*(0.5-0.25) =
/// 0.125; new = 0.25+0.125 = 0.375 — exact in binary64 (dyadic).
#[test]
fn plain_transmission_moves_the_target_by_the_computed_delta() {
    let (graph, _sink, _report) = run();
    assert_eq!(revolutionary(&graph, PLAIN_TARGET), 0.375);
}

/// Witness 2a — the MASS_AWAKENING crossing case (the event itself is
/// Task 3's; this pins only the value the crossing rests on). delta =
/// 0.5*(0.875-0.5625) = 0.15625; new = 0.5625+0.15625 = 0.71875.
#[test]
fn mass_awakening_crossing_target_lands_past_the_threshold() {
    let (graph, _sink, _report) = run();
    assert_eq!(revolutionary(&graph, MASS_AWAKEN_CROSS_TARGET), 0.71875);
}

/// Witness 2b — the negative case: rises but stays below 0.6. delta =
/// 0.125*(0.875-0.5) = 0.046875; new = 0.5+0.046875 = 0.546875.
#[test]
fn mass_awakening_negative_case_stays_below_the_threshold() {
    let (graph, _sink, _report) = run();
    assert_eq!(revolutionary(&graph, MASS_AWAKEN_STAYS_TARGET), 0.546875);
}

/// Witness 2c — lands EXACTLY at the mass-awakening threshold (the frozen
/// `<=` boundary arm, `solidarity.py:190`'s `old < threshold <= new`).
/// `mass-awaken-exact-source`'s seed (`0.6p`) and the
/// `solidarity/mass-awakening-threshold` defconst (`0.6c`) parse the same
/// literal text through the identical `unscaled/10^scale` division, so they
/// are bit-identical doubles; `strength=1c` makes
/// `delta = 1.0*(0.6d - 0.5)` and Sterbenz's lemma (0.5 and 0.6d share one
/// binade, `[0.5, 1.0)`) makes that subtraction — and the re-summing
/// `0.5 + delta` — EXACT, landing back on 0.6d bit for bit, not merely
/// "close to 0.6".
///
/// Task 1's own note flagged this equality as analytically verified but
/// never read back from the store; this is that read-back, by exact
/// `to_bits()` comparison against the SAME `:const` the rule itself binds
/// (not a hand-typed `0.6_f64` literal, which would only prove Rust's
/// parser agrees with itself).
#[test]
fn mass_awakening_exact_case_lands_bit_identical_to_the_threshold_const() {
    let (graph, _sink, _report) = run();
    let mut seeds = MemoryGraph::new();
    let scenario = load_scenario(SCENARIO, &mut seeds).expect("the scenario must load");
    let Some(Value::Real(threshold)) = scenario.consts.get("solidarity/mass-awakening-threshold")
    else {
        panic!("the scenario must declare solidarity/mass-awakening-threshold as a scaled literal");
    };
    let new_r = revolutionary(&graph, MASS_AWAKEN_EXACT_TARGET);
    assert_eq!(
        new_r.to_bits(),
        threshold.to_bits(),
        "Sterbenz's-lemma equality: target_r + delta must re-sum to EXACTLY the \
         mass-awakening-threshold const, bit for bit — new={new_r:?} \
         (0x{:016x}), threshold={threshold:?} (0x{:016x})",
        new_r.to_bits(),
        threshold.to_bits()
    );
    // And the decimal value really is 0.6 — the bit-exact check above is the
    // load-bearing assertion; this is a human-readable cross-check.
    assert_eq!(*threshold, 0.6);
}

/// Witness 3a — `strength <= 0` skips the whole edge; the target is
/// untouched at its seed.
#[test]
fn zero_strength_edge_never_transmits() {
    let (graph, _sink, _report) = run();
    assert_eq!(revolutionary(&graph, ZERO_STRENGTH_TARGET), 0.25);
}

/// Witness 3b — a source AT (not above) `activation_threshold` never fires
/// as a subject at all (the strict `>` gate), so even a well-formed edge to
/// an active target never transmits.
#[test]
fn source_at_threshold_never_transmits() {
    let (graph, _sink, _report) = run();
    assert_eq!(revolutionary(&graph, AT_THRESHOLD_TARGET), 0.25);
}

/// Witness 3c — every other gate passes (source above threshold, strength
/// positive, target active) but `|delta| = 0.0078125 < 0.01` (the
/// negligible-transmission floor) skips the write.
#[test]
fn negligible_delta_is_skipped() {
    let (graph, _sink, _report) = run();
    assert_eq!(revolutionary(&graph, NEGLIGIBLE_TARGET), 0.4375);
}

/// Witness 4 — the multi-inbound divergence (D-record 2), EXECUTED, not
/// merely asserted in prose. Frozen applies each edge sequentially against
/// the previous write (0.1 -> 0.34 -> 0.478); this port collects both
/// subjects' writes against the SAME pre-tick target (0.1) and `set` makes
/// the LAST subject in ascending-node-id order (`multi-source-b`, id 14)
/// win. The write itself is forward-computed via the convex-combination
/// form `solidarity.bsl`'s `p0-transmit` now uses (#491 T1 S1, Director
/// sitting 2026-08-18: repair-now+ceremony) — `(1 - strength) * target +
/// strength * source` — which happens to land bit-identical to the `f64` literal `0.31` here
/// (`0.7 * 0.1 + 0.3 * 0.8`), unlike the frozen `target + delta` form's
/// `0.31000000000000005` (an IEEE-754 rounding-order artifact, not a
/// semantic drift; predicted by
/// reports/kind-straddle-repair-options-2026-08-18.md §2.1's per-witness
/// table, ceremony re-pin).
#[test]
fn multi_inbound_edges_diverge_from_the_frozen_sequential_apply() {
    let (graph, _sink, _report) = run();
    let expected = (1.0_f64 - 0.3_f64) * 0.1_f64 + 0.3_f64 * 0.8_f64;
    assert_eq!(revolutionary(&graph, MULTI_TARGET), expected);
    // Human-readable cross-check that the port really did diverge from the
    // frozen sequential value (0.478) it deliberately does not reproduce.
    assert_ne!(expected, 0.478);
    assert!(
        (expected - 0.31).abs() < 1e-12,
        "expected ~= 0.31, got {expected}"
    );
}

/// Extra — the inactive-SOURCE skip: an otherwise-qualifying source
/// (r=0.9 > threshold) that is itself dead never even reaches its own
/// edges (the subject-level `when` gate).
#[test]
fn inactive_source_never_fires_at_all() {
    let (graph, _sink, _report) = run();
    assert_eq!(revolutionary(&graph, INACTIVE_SOURCE_TARGET), 0.25);
}

/// Extra — the inactive-TARGET skip: a qualifying, live source fires as a
/// subject (it passes its own gate) but its only neighbour is dead, so the
/// per-edge guard produces no write.
#[test]
fn inactive_target_receives_no_write() {
    let (graph, _sink, _report) = run();
    assert_eq!(revolutionary(&graph, INACTIVE_TARGET), 0.25);
}

/// Extra — the clamp witness: `target + delta` (0.875 + 2*(1.0-0.875) =
/// 1.125) overshoots 1.0; the rule's own `if`-expressed clamp (transcribing
/// `solidarity.py:165`'s `max(0.0, min(1.0, …))`) must land it at EXACTLY
/// 1.0, not 1.125 and not any other value — without it this write would be
/// `E-EVAL-020`-fatal (a `probability` field is `[0,1]`-bounded at the
/// store, never silently clamped).
#[test]
fn the_clamp_caps_an_overshooting_transmission_at_exactly_one() {
    let (graph, _sink, _report) = run();
    assert_eq!(revolutionary(&graph, CLAMP_TARGET), 1.0);
}

/// Sanity: the un-targeted sources are untouched (the rule only ever
/// writes `it`, never `self`) — not a witness in its own right, just a
/// guard against a rule that accidentally wrote its own subject.
#[test]
fn sources_are_never_written_by_their_own_rule_firing() {
    let (graph, _sink, _report) = run();
    assert_eq!(revolutionary(&graph, PLAIN_SOURCE), 0.5);
    assert_eq!(revolutionary(&graph, AWAKEN_SOURCE), 0.875);
    assert_eq!(revolutionary(&graph, MASS_AWAKEN_EXACT_SOURCE), 0.6);
    assert_eq!(revolutionary(&graph, ZERO_STRENGTH_SOURCE), 0.75);
    assert_eq!(revolutionary(&graph, AT_THRESHOLD_SOURCE), 0.3);
    assert_eq!(revolutionary(&graph, NEGLIGIBLE_SOURCE), 0.5);
    assert_eq!(revolutionary(&graph, MULTI_SOURCE_A), 0.9);
    assert_eq!(revolutionary(&graph, MULTI_SOURCE_B), 0.8);
    assert_eq!(revolutionary(&graph, INACTIVE_SOURCE), 0.9);
    assert_eq!(revolutionary(&graph, INACTIVE_TARGET_SOURCE), 0.9);
    assert_eq!(revolutionary(&graph, CLAMP_SOURCE), 1.0);
}

// ---------------------------------------------------------------------
// Task 3 — the two event emits (plan §6 Task 3; brief
// `.superpowers/sdd/2026-08-17-solidarity-port/task-3-brief.md`), pinned by
// full ordered payload, in the shape of
// `vitality_conformance.rs::the_reaper_emits_one_entity_death_per_dissolution`.
//
// Nine events total. Order follows the subject firing order (ascending node
// id — `tick.rs`'s own contract) then, within one subject, `neighbors`' own
// ascending-target-id sort (`memory.rs::neighbors`, "a set, not a
// multiset"):
//   1. CT  plain-source(0)          -> plain-target(1)
//   2. CT  awaken-source(2)         -> mass-awaken-cross-target(3)
//   3. MA  mass-awaken-cross-target(3)                 (crosses PAST 0.6)
//   4. CT  awaken-source(2)         -> mass-awaken-stays-target(4)
//                                       (the negative case: rises, no MA)
//   5. CT  mass-awaken-exact-source(5) -> mass-awaken-exact-target(6)
//   6. MA  mass-awaken-exact-target(6)                 (lands AT 0.6, `<=`)
//   7. CT  multi-source-a(13)       -> multi-target(15)
//   8. CT  multi-source-b(14)       -> multi-target(15)
//   9. CT  clamp-source(20)         -> clamp-target(21) (delta is the RAW
//                                       unclamped 0.25, new is the CLAMPED
//                                       1.0 — the two payload fields must
//                                       not be conflated)
// The three skip-gated edges (zero-strength, at-threshold's subject never
// fires, negligible) and the two inactive-node extras never reach an emit;
// the five subjects that fire but own no outbound SOLIDARITY edge (3, 4, 6,
// 12, 21) emit nothing either.

/// The full ordered event list, every payload pinned by exact
/// `Value::NodeRef`/`Value::Real` equality — RED until both emits land in
/// `solidarity.bsl`.
#[test]
fn events_land_in_declared_order_with_full_pinned_payloads() {
    let (_graph, sink, _report) = run();

    // The multi-inbound deltas, forward-computed exactly as
    // `multi_inbound_edges_diverge_from_the_frozen_sequential_apply` already
    // does: 0.9/0.8/0.1/0.3 are not exact dyadic rationals, so a hand-rounded
    // decimal here would risk a transcription error unrelated to the port.
    // `delta` itself is untouched by the kind-coherence repair below (the
    // CONSCIOUSNESS_TRANSMISSION payload still computes it as `strength *
    // (source - target)`, unchanged).
    let delta_a = 0.3_f64 * (0.9_f64 - 0.1_f64);
    let delta_b = 0.3_f64 * (0.8_f64 - 0.1_f64);
    // `new-target-consciousness` (this event's payload AND the final write)
    // is forward-computed via the convex-combination form
    // `solidarity.bsl`'s `p0-transmit` now uses (#491 T1 S1, Director
    // sitting 2026-08-18: repair-now+ceremony) — `(1 - strength) * target +
    // strength * source`, algebraically identical to `target + delta` but
    // NOT bit-identical for a non-power-of-2 strength like 0.3: an
    // IEEE-754 rounding-order artifact, not a semantic drift (dossier
    // reports/kind-straddle-repair-options-2026-08-18.md §2.1's own
    // per-witness table predicted exactly this pair; ceremony re-pin).
    let new_a = (1.0_f64 - 0.3_f64) * 0.1_f64 + 0.3_f64 * 0.9_f64;
    let new_b = (1.0_f64 - 0.3_f64) * 0.1_f64 + 0.3_f64 * 0.8_f64;
    // The exact-0.6 boundary's delta and re-summed new, forward-computed by
    // the SAME Sterbenz-exact subtraction/addition
    // `mass_awakening_exact_case_lands_bit_identical_to_the_threshold_const`
    // already proved bit-identical to the `:const` — reused, not re-derived,
    // so the two tests cannot silently diverge from each other.
    let delta_exact = 0.6_f64 - 0.5_f64;
    let new_exact = 0.5_f64 + delta_exact;

    fn ct(
        source: u64,
        target: u64,
        delta: f64,
        strength: f64,
        source_c: f64,
        old: f64,
        new: f64,
    ) -> (String, Vec<(String, Value)>) {
        (
            "CONSCIOUSNESS_TRANSMISSION".to_owned(),
            vec![
                ("source-id".to_owned(), Value::NodeRef(NodeId(source))),
                ("target-id".to_owned(), Value::NodeRef(NodeId(target))),
                ("delta".to_owned(), Value::Real(delta)),
                ("solidarity-strength".to_owned(), Value::Real(strength)),
                ("source-consciousness".to_owned(), Value::Real(source_c)),
                ("old-target-consciousness".to_owned(), Value::Real(old)),
                ("new-target-consciousness".to_owned(), Value::Real(new)),
            ],
        )
    }
    fn ma(
        target: u64,
        old: f64,
        new: f64,
        triggering_source: u64,
    ) -> (String, Vec<(String, Value)>) {
        (
            "MASS_AWAKENING".to_owned(),
            vec![
                ("target-id".to_owned(), Value::NodeRef(NodeId(target))),
                ("old-consciousness".to_owned(), Value::Real(old)),
                ("new-consciousness".to_owned(), Value::Real(new)),
                (
                    "triggering-source".to_owned(),
                    Value::NodeRef(NodeId(triggering_source)),
                ),
            ],
        )
    }

    let expected: Vec<(String, Vec<(String, Value)>)> = vec![
        ct(PLAIN_SOURCE, PLAIN_TARGET, 0.125, 0.5, 0.5, 0.25, 0.375),
        ct(
            AWAKEN_SOURCE,
            MASS_AWAKEN_CROSS_TARGET,
            0.15625,
            0.5,
            0.875,
            0.5625,
            0.71875,
        ),
        ma(MASS_AWAKEN_CROSS_TARGET, 0.5625, 0.71875, AWAKEN_SOURCE),
        ct(
            AWAKEN_SOURCE,
            MASS_AWAKEN_STAYS_TARGET,
            0.046875,
            0.125,
            0.875,
            0.5,
            0.546875,
        ),
        ct(
            MASS_AWAKEN_EXACT_SOURCE,
            MASS_AWAKEN_EXACT_TARGET,
            delta_exact,
            1.0,
            0.6,
            0.5,
            new_exact,
        ),
        ma(
            MASS_AWAKEN_EXACT_TARGET,
            0.5,
            new_exact,
            MASS_AWAKEN_EXACT_SOURCE,
        ),
        ct(MULTI_SOURCE_A, MULTI_TARGET, delta_a, 0.3, 0.9, 0.1, new_a),
        ct(MULTI_SOURCE_B, MULTI_TARGET, delta_b, 0.3, 0.8, 0.1, new_b),
        ct(CLAMP_SOURCE, CLAMP_TARGET, 0.25, 2.0, 1.0, 0.875, 1.0),
    ];

    assert_eq!(
        sink.events.len(),
        expected.len(),
        "nine events total, got: {:#?}",
        sink.events
    );
    for (i, (actual, expect)) in sink.events.iter().zip(expected.iter()).enumerate() {
        assert_eq!(actual, expect, "event {i}");
    }
}

/// The negative case, isolated (brief step 1's explicit callout): a
/// transmission that raises `revolutionary` but does not cross 0.6 (target
/// 4, `mass-awaken-stays-target`) emits exactly one event —
/// CONSCIOUSNESS_TRANSMISSION only, never MASS_AWAKENING.
#[test]
fn mass_awakening_negative_case_emits_exactly_one_event() {
    let (_graph, sink, _report) = run();
    let for_target: Vec<&(String, Vec<(String, Value)>)> = sink
        .events
        .iter()
        .filter(|(_, payload)| {
            payload.iter().any(|(k, v)| {
                k == "target-id" && *v == Value::NodeRef(NodeId(MASS_AWAKEN_STAYS_TARGET))
            })
        })
        .collect();
    assert_eq!(
        for_target.len(),
        1,
        "exactly one event for the negative case: {for_target:#?}"
    );
    assert_eq!(for_target[0].0, "CONSCIOUSNESS_TRANSMISSION");
}

/// The exact-0.6 boundary target (witness 2c) DOES fire MASS_AWAKENING —
/// the frozen chained comparison's `<=` arm (`solidarity.py:190`), not `<`.
/// Isolated from the full-payload test above so a future edit narrowing
/// `<=` to `<` fails loudly here even if the ordered-list test's shape
/// happened to still line up some other way.
#[test]
fn mass_awakening_fires_on_the_exact_threshold_boundary() {
    let (_graph, sink, _report) = run();
    let fired = sink.events.iter().any(|(ty, payload)| {
        ty == "MASS_AWAKENING"
            && payload.iter().any(|(k, v)| {
                k == "target-id" && *v == Value::NodeRef(NodeId(MASS_AWAKEN_EXACT_TARGET))
            })
    });
    assert!(
        fired,
        "MASS_AWAKENING must fire on the <= boundary (old < 0.6 <= new)"
    );
}

/// Byte-determinism: the same content twice is the same post-state hash.
#[test]
fn the_solidarity_tick_is_deterministic() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after, "two runs, one post-state");
    assert_ne!(a.before, a.after, "the pack must move state");
}
