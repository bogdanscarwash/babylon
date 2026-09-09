//! Conformance vectors for the `territory/*` rule pack (P27 Phase 2, the
//! Territory port train — `docs/superpowers/plans/2026-08-12-territory-
//! port-plan.md`, PR B), taken from the frozen Python engine's live
//! behaviour.
//!
//! # Provenance
//!
//! Every STRUCTURAL claim below (which fields moved, in which direction,
//! the latch set, the sink chosen, the suppression set) was checked against
//! the frozen `TerritorySystem` running one `step()` over a fixture that
//! mirrors `content/scenarios/territory-conformance.bscn` node for node.
//! The command, from the repository root:
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/territory_conformance.py
//! ```
//!
//! The frozen system is the contract source for STRUCTURE and ORDERING,
//! not a correctness oracle (ADR183) — the port train's own D-records
//! (`territory.bsl`'s header) name every place this pack's arithmetic
//! diverges from the frozen engine's operation SEQUENCE for the same
//! real-valued function (the scaled-Int rent lane, the pull-side spillover
//! fold, the directed-vs-any sink/spillover walks). Every NUMERIC value
//! pinned below is measured from the BSL engine's own output, never copied
//! from the frozen mirror's printed floats.
//!
//! # Scenario census (Task 3)
//!
//! Twelve territories, three social classes, eight edges (six ADJACENCY,
//! two TENANCY) — see `territory-conformance.bscn`'s own header for the
//! full per-node conformance-case map.

use babylon_bsl::scenario::load_scenario;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_kernel::SessionId;
use babylon_tick::run_once_into;

const SCENARIO: &str = include_str!("../content/scenarios/territory-conformance.bscn");

// Node ids, fixed by the scenario's own declaration order
// (`territory-conformance.bscn`'s own header names the same map). Several
// are unused until later tasks (p2's sink/latch nodes, p3's chain, p4's
// camp) accrete their own tests — named now, for documentation symmetry
// with the scenario's own header, per `query_lane_e2e.rs`'s precedent.
//
// SINK_RESERVATION (id 3) precedes SINK_PENAL (id 4) -- fix round NIT-10:
// declaring the tiebreak LOSER first means the sink-selection test
// discriminates real score-based selection from an accidental
// declaration-id-order coincidence (previously PENAL_COLONY, the
// higher-scoring sink, ALSO happened to have the lower id, so a mutated
// select-max that fell back to plain id-order would still have passed).
const SUB_THRESHOLD_HIGH: NodeId = NodeId(0);
const SUB_THRESHOLD_LOW: NodeId = NodeId(1);
#[allow(dead_code)]
const LATCH_TICK_SOURCE: NodeId = NodeId(2);
#[allow(dead_code)]
const SINK_RESERVATION: NodeId = NodeId(3);
#[allow(dead_code)]
const SINK_PENAL: NodeId = NodeId(4);
#[allow(dead_code)]
const LATCH_NO_SINK: NodeId = NodeId(5);
#[allow(dead_code)]
const ALREADY_LATCHED_TO_CAMP: NodeId = NodeId(6);
#[allow(dead_code)]
const CONCENTRATION_CAMP: NodeId = NodeId(7);
const CHAIN_1: NodeId = NodeId(8);
const CHAIN_2: NodeId = NodeId(9);
const CHAIN_3: NodeId = NodeId(10);
const ISOLATED_FALLBACK: NodeId = NodeId(11);
#[allow(dead_code)] // named for documentation symmetry with the id map above
const TENANT_1: NodeId = NodeId(12);
#[allow(dead_code)]
const TENANT_2: NodeId = NodeId(13);
#[allow(dead_code)]
const NON_TENANT: NodeId = NodeId(14);

/// Task 3, Step 2: the scenario loads clean and the node/edge census
/// matches the header's own count — no rule pack yet, load-only.
#[test]
fn the_scenario_loads_clean_with_the_declared_census() {
    let mut graph = HypergraphStore::new();
    let loaded = load_scenario(SCENARIO, &mut graph).expect("the scenario must load clean");
    assert_eq!(loaded.node_count, 15, "12 territories + 3 social classes");
    assert_eq!(loaded.edge_count, 8, "6 ADJACENCY + 2 TENANCY");
    assert_eq!(
        loaded.node_types.get("TERRITORY").copied(),
        Some(12),
        "twelve territory nodes"
    );
    assert_eq!(
        loaded.node_types.get("SOCIAL_CLASS").copied(),
        Some(3),
        "three social-class nodes"
    );
    assert_eq!(loaded.edge_types.get("ADJACENCY").copied(), Some(6));
    assert_eq!(loaded.edge_types.get("TENANCY").copied(), Some(2));
}

/// Every field the pack's four phases read must be present on every
/// territory (No-defaults contract) — a smoke read of all six declared
/// territory fields on every one of the twelve nodes, before any rule
/// exists to touch them.
#[test]
fn every_territory_seeds_all_six_declared_fields() {
    let mut graph = HypergraphStore::new();
    load_scenario(SCENARIO, &mut graph).expect("the scenario must load clean");
    for id in 0..12u64 {
        let node = NodeId(id);
        for field in [
            "territory/profile",
            "territory/territory-type",
            "territory/heat",
            "territory/rent-level-x1e6",
            "territory/under-eviction",
            "territory/population",
        ] {
            graph
                .node_attribute(node, field)
                .unwrap_or_else(|e| panic!("node {id} field {field}: {}", e.message));
        }
    }
}

/// A no-op tick (a rule that never fires) still hashes deterministically —
/// the load-only byte-determinism check this pack's later goldens build on.
#[test]
fn a_no_op_rule_is_deterministic_across_two_independent_loads() {
    const NO_OP_RULE: &str = r#"
(rule territory/noop-probe
  :role mechanic :evidence derived :material-basis "load-only smoke: prove the scenario alone hashes deterministically"
  :fuel 8
  (bindings (binding heat :field territory/heat))
  (when (< heat 0))
  (effects
    (update-node self territory/heat (set heat))))
"#;
    let a = run_once_into(
        SCENARIO,
        NO_OP_RULE,
        &mut HypergraphStore::new(),
        &mut babylon_bsl::structural_verbs::CollectingSink::default(),
    )
    .expect("the scenario plus a never-firing rule must still run");
    let b = run_once_into(
        SCENARIO,
        NO_OP_RULE,
        &mut HypergraphStore::new(),
        &mut babylon_bsl::structural_verbs::CollectingSink::default(),
    )
    .expect("second run");
    assert_eq!(a.before, b.before, "pre-tick hash reproducible");
    assert_eq!(a.after, b.after, "post-tick hash reproducible");
    assert_eq!(a.fired, 0, "no territory has heat < 0");
}

// ============================================================ Task 4: p1

const TERRITORY_RULES: &str = include_str!("../content/rules/territory.bsl");

fn run_territory() -> (HypergraphStore, babylon_tick::TickReport) {
    let mut graph = HypergraphStore::new();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let report = run_once_into(SCENARIO, TERRITORY_RULES, &mut graph, &mut sink)
        .expect("the territory pack must load and run");
    (graph, report)
}

fn heat(graph: &HypergraphStore, id: NodeId) -> f64 {
    graph
        .node_attribute(id, "territory/heat")
        .unwrap_or_else(|e| panic!("node {id:?} territory/heat: {}", e.message))
}

/// `territory/p1-heat-dynamics`: HIGH_PROFILE gains exactly the defined
/// `high_profile_heat_gain` (0.15) — `sub-threshold-high`'s seed 0.3 -> 0.45.
#[test]
fn p1_high_profile_gains_exactly_the_defined_gain() {
    let (graph, _report) = run_territory();
    // Measured (ADR183), not derived: IEEE-754 `0.3 + 0.15` does NOT land
    // on the correctly-rounded literal `0.45` — it is one ULP below
    // (`0.44999999999999996`). The frozen Python mirror's own printed
    // output (`territory_conformance.py`) agrees bit-for-bit
    // (`heat=0.44999999999999996`): both engines perform the identical
    // `heat + gain` binary64 add.
    assert_eq!(
        heat(&graph, SUB_THRESHOLD_HIGH).to_bits(),
        0.44999999999999996_f64.to_bits(),
        "0.3 + 0.15, measured — matches the frozen mirror's own printed float bit for bit"
    );
}

/// LOW_PROFILE decays by `(1 - heat_decay_rate)` = x0.9 —
/// `sub-threshold-low`'s seed 0.4 -> ~0.36.
#[test]
fn p1_low_profile_decays_by_the_defined_rate() {
    let (graph, _report) = run_territory();
    // Measured (ADR183): `0.4 * 0.9` is one ULP above the correctly-rounded
    // `0.36` literal (`0.36000000000000004`) — matches the frozen mirror's
    // own printed output bit for bit.
    assert_eq!(
        heat(&graph, SUB_THRESHOLD_LOW).to_bits(),
        0.36000000000000004_f64.to_bits(),
        "0.4 * 0.9, measured — matches the frozen mirror's own printed float bit for bit"
    );
}

/// `chain-1`'s seed 0.9 + 0.15 = 1.05, over the [0,1] ceiling — the
/// `system_base.py::_write_clamped` double-clamp idiom (nested-if, floor
/// then ceiling) must land it at EXACTLY 1.0, not 1.05.
#[test]
fn p1_clamps_the_ceiling_at_exactly_one() {
    let (graph, _report) = run_territory();
    assert_eq!(
        heat(&graph, CHAIN_1),
        1.0,
        "0.9 + 0.15 = 1.05 must clamp to exactly 1.0"
    );
}

/// `:fuel`/guard shape: `(when #t)` fires unconditionally — every one of
/// the twelve territories, and nothing else (the three social classes
/// carry no `territory/*` fields, so they are not this rule's subject
/// type at all).
#[test]
fn p1_fires_on_every_seeded_territory() {
    let (_graph, report) = run_territory();
    let p1_fired = report
        .per_rule_fired
        .iter()
        .find(|(id, _)| id == "territory/p1-heat-dynamics")
        .map(|(_, n)| *n);
    assert_eq!(p1_fired, Some(12), "all twelve territories, unconditional");
}

// ============================================================ Task 5: p2

fn under_eviction(graph: &HypergraphStore, id: NodeId) -> f64 {
    graph
        .node_attribute(id, "territory/under-eviction")
        .unwrap_or_else(|e| panic!("node {id:?} territory/under-eviction: {}", e.message))
}

fn rent(graph: &HypergraphStore, id: NodeId) -> f64 {
    graph
        .node_attribute(id, "territory/rent-level-x1e6")
        .unwrap_or_else(|e| panic!("node {id:?} territory/rent-level-x1e6: {}", e.message))
}

fn population(graph: &HypergraphStore, id: NodeId) -> f64 {
    graph
        .node_attribute(id, "territory/population")
        .unwrap_or_else(|e| panic!("node {id:?} territory/population: {}", e.message))
}

/// `latch-tick-source` crosses the 0.8 threshold THIS tick (p1 pushes heat
/// 0.68 -> 0.83): flag latches, rent spikes x1.5, population displaces
/// `floor(1000 * 0.1) = 100`, and the EXTRACTION-priority tiebreak routes
/// the transfer to `sink-penal` (PENAL_COLONY, priority 3) over
/// `sink-reservation` (RESERVATION, priority 2, declared FIRST as of the
/// fix round's NIT-10 reorder — this test would flip if `select-max`
/// silently fell back to declaration-id order instead of the score).
/// `sink-penal`'s heat pin is JC-D's value-pin batch: it also receives
/// spillover from BOTH `latch-tick-source` and (via the MINOR-7 witness
/// edge) `latch-no-sink`, `0.09 (post-p1 decay) + (0.83 + 0.83) * 0.05 =
/// 0.17300000000000004` — measured, matching the frozen mirror bit for bit.
#[test]
fn p2_latch_tick_spikes_rent_displaces_population_and_picks_the_penal_sink() {
    let (graph, _report) = run_territory();
    assert_eq!(under_eviction(&graph, LATCH_TICK_SOURCE), 1.0);
    assert_eq!(rent(&graph, LATCH_TICK_SOURCE), 1_500_000.0);
    assert_eq!(population(&graph, LATCH_TICK_SOURCE), 900.0);
    assert_eq!(
        heat(&graph, LATCH_TICK_SOURCE).to_bits(),
        0.8390000000000001_f64.to_bits(),
        "0.68 + 0.15 (p1) + (0.09 + 0.09) * 0.05 (p3, both sinks' post-p1 heat) = 0.839..."
    );
    assert_eq!(
        population(&graph, SINK_PENAL),
        100.0,
        "PENAL_COLONY (priority 3) wins the EXTRACTION tiebreak"
    );
    assert_eq!(
        under_eviction(&graph, SINK_PENAL),
        0.0,
        "never itself latches"
    );
    assert_eq!(rent(&graph, SINK_PENAL), 1_000_000.0, "rent unspiked");
    assert_eq!(
        heat(&graph, SINK_PENAL).to_bits(),
        0.17300000000000004_f64.to_bits(),
        "sink-penal's own heat: measured, matching the frozen mirror bit for bit"
    );
    assert_eq!(
        population(&graph, SINK_RESERVATION),
        200.0,
        "RESERVATION (priority 2) loses the tiebreak — seed value untouched (fix round \
         MINOR-4: seeded nonzero so this is a real assertion, not floor(0*x)=0 vacuity)"
    );
    assert_eq!(under_eviction(&graph, SINK_RESERVATION), 0.0);
    assert_eq!(rent(&graph, SINK_RESERVATION), 1_000_000.0);
    assert_eq!(
        heat(&graph, SINK_RESERVATION).to_bits(),
        0.1315_f64.to_bits(),
        "sink-reservation's own heat: measured, matching the frozen mirror bit for bit"
    );
}

/// A latched territory with NO OUTGOING adjacency loses population with
/// nothing gaining it — the frozen `sink_id is None` case, transcribed:
/// the population "disappears" rather than being conserved. `latch-no-sink`
/// heat pin is JC-D's value-pin batch: `0.68 + 0.15 (p1) + 0.09 * 0.05 (p3,
/// spillover FROM sink-penal via the MINOR-7 witness edge) = 0.8345`.
#[test]
fn p2_no_sink_latched_territory_loses_population_with_nothing_gaining_it() {
    let (graph, _report) = run_territory();
    assert_eq!(under_eviction(&graph, LATCH_NO_SINK), 1.0);
    assert_eq!(rent(&graph, LATCH_NO_SINK), 1_500_000.0);
    assert_eq!(
        population(&graph, LATCH_NO_SINK),
        900.0,
        "1000 - floor(1000*0.1) = 900, with no sink to receive the 100 displaced"
    );
    assert_eq!(
        heat(&graph, LATCH_NO_SINK).to_bits(),
        0.8345_f64.to_bits(),
        "heat still moves via p3's :any spillover walk even though p2's :out sink \
         walk finds nothing — measured, matching the frozen mirror bit for bit"
    );
}

/// D123's frozen defect, witnessed directly (fix round MINOR-7): the
/// `(edge EdgeType/ADJACENCY sink-penal latch-no-sink 1)` edge makes a
/// PENAL_COLONY topologically ADJACENT to `latch-no-sink` — via
/// `:any`, p3-spillover sees it (the heat pin above proves this) — but
/// p2's DIRECTED `:out` sink walk requires `latch-no-sink` to be the
/// edge's SOURCE, which it is not (the edge points INTO it), so the
/// walk finds no candidate and population still disappears. This is the
/// exact frozen shape the register's D123 row names: "a canonical pair
/// pointing INTO the evicting territory yields 'no sink, population
/// disappears'" — previously asserted only in prose, now a real vector:
/// a genuinely-adjacent sink exists and is still invisible to the walk.
#[test]
fn p2_d123_a_topologically_adjacent_sink_is_invisible_to_the_directed_walk() {
    let (graph, _report) = run_territory();
    // The witness precondition: sink-penal really is ADJACENT (p3's :any
    // spillover reached it — see the heat pin in the no-sink test above).
    // The D123 claim: population STILL disappears despite that adjacency.
    assert_eq!(
        population(&graph, LATCH_NO_SINK),
        900.0,
        "a PENAL_COLONY is topologically adjacent via the reversed edge, \
         yet the directed :out sink walk cannot see it — 100 still disappears"
    );
    assert_eq!(
        population(&graph, SINK_PENAL),
        100.0,
        "sink-penal's population came ONLY from latch-tick-source's (correctly \
         directed) transfer — nothing arrived from latch-no-sink via the \
         reversed edge, because that edge is invisible to latch-no-sink's OWN walk"
    );
}

/// A sub-threshold, never-latched territory is fully untouched by p2 —
/// every field p2 could have written stays at its seed value.
#[test]
fn p2_sub_threshold_territory_is_untouched() {
    let (graph, _report) = run_territory();
    assert_eq!(under_eviction(&graph, SUB_THRESHOLD_LOW), 0.0);
    assert_eq!(
        rent(&graph, SUB_THRESHOLD_LOW),
        1_000_000.0,
        "rent unspiked"
    );
    assert_eq!(
        population(&graph, SUB_THRESHOLD_LOW),
        100.0,
        "population unmoved — the seed value"
    );
}

/// An already-latched territory keeps compounding across ticks:
/// rent 1.0 -> 1.5e6 (tick 1) -> 2.25e6 (tick 2), via TWO
/// `TickSession::advance` calls. `TERRITORY_RULES` is the WHOLE pack (all
/// five rules, p1 through p4) even at this point in the file — the
/// section markers below are for reading order, not content accretion;
/// p3/p4 fire on every tick this test runs too (harmlessly: p3 moves
/// this node's `heat`, p4's guards don't match its CORE territory-type),
/// they just do not change the rent value this test pins.
#[test]
fn p2_already_latched_territory_compounds_rent_across_two_ticks() {
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let mut session = babylon_tick::TickSession::new(
        SCENARIO,
        TERRITORY_RULES,
        HypergraphStore::new(),
        SessionId::new("territory-conformance-test").expect("literal is non-empty"),
    )
    .expect("the pack must load into a session");
    session.advance(&mut sink).expect("tick 1");
    session.advance(&mut sink).expect("tick 2");
    let rent_after_two_ticks = session
        .graph()
        .node_attribute(ALREADY_LATCHED_TO_CAMP, "territory/rent-level-x1e6")
        .expect("rent must be written");
    assert_eq!(
        rent_after_two_ticks, 2_250_000.0,
        "1,000,000 * 1.5 * 1.5 = 2,250,000 — compounding across two ticks"
    );
}

/// Conservation, asserted explicitly (Task 5's own requirement): total
/// population lost by every source this tick equals total population
/// gained by every sink, EXCEPT the no-sink disappearance
/// (`latch-no-sink`'s 100, which is the ONE deliberate non-conserving
/// term — D-record #8, hash-neutral by construction, population-neutral
/// by frozen design).
#[test]
fn p2_population_conserves_except_the_declared_no_sink_disappearance() {
    let (graph, _report) = run_territory();
    // Sources that actually latch and displace this tick:
    //   latch-tick-source: -100 (all routed to sink-penal)
    //   latch-no-sink:     -100 (no sink — disappears)
    //   already-latched-to-camp: -100 (routed to concentration-camp)
    //   chain-1: -100 (no qualifying sink among its CORE neighbours — disappears)
    let latch_tick_source_loss = 1000.0 - population(&graph, LATCH_TICK_SOURCE);
    let latch_no_sink_loss = 1000.0 - population(&graph, LATCH_NO_SINK);
    let already_latched_loss = 1000.0 - population(&graph, ALREADY_LATCHED_TO_CAMP);
    let chain_1_loss = 1000.0 - population(&graph, CHAIN_1);
    assert_eq!(latch_tick_source_loss, 100.0);
    assert_eq!(latch_no_sink_loss, 100.0);
    assert_eq!(already_latched_loss, 100.0);
    assert_eq!(chain_1_loss, 100.0);

    let sink_penal_gain = population(&graph, SINK_PENAL); // seeded 0
                                                          // concentration-camp's own seed is 500, gain measured separately by p4.
    assert_eq!(
        sink_penal_gain, 100.0,
        "sink-penal gained exactly latch-tick-source's transfer"
    );
    // The conservation law, stated: total displaced (400) = total
    // conserved-and-received (100 to sink-penal + 100 to concentration-camp)
    // + total disappeared (100 latch-no-sink + 100 chain-1, no qualifying sink).
    let total_displaced =
        latch_tick_source_loss + latch_no_sink_loss + already_latched_loss + chain_1_loss;
    let total_received_by_sink_penal = sink_penal_gain;
    let total_received_by_camp_this_tick = 100.0; // asserted directly by the p4 camp-decay test
    let total_disappeared = latch_no_sink_loss + chain_1_loss;
    assert_eq!(
        total_displaced,
        total_received_by_sink_penal + total_received_by_camp_this_tick + total_disappeared,
        "400 displaced = 100 (sink-penal) + 100 (camp) + 200 (disappeared, no sink)"
    );
}

// ============================================================ Task 6: p3

/// The middle of the 3-chain (`chain-2`) gains spillover from BOTH
/// neighbours, reading their PRE-PHASE-3 heat (post-p1, since p2 never
/// writes `heat`) — the section-4.2 chapter-C4 pre-state law, end to end.
/// Measured (ADR183): `0.329` exactly — matches the frozen mirror's own
/// printed `chain-2 heat=0.329` bit for bit (this fixture's inputs happen
/// to make `Σ(heatᵢ) * rate` and `Σ(heatᵢ * rate)` agree; D-9 names this
/// as a property of these specific inputs, not a general guarantee).
#[test]
fn p3_middle_chain_node_gains_from_both_neighbours_pre_phase() {
    let (graph, _report) = run_territory();
    assert_eq!(
        heat(&graph, CHAIN_2).to_bits(),
        0.329_f64.to_bits(),
        "chain-2: 0.27 (post-p1 decay) + (1.0 + 0.18) * 0.05 = 0.329"
    );
}

/// An end of the chain (`chain-3`) gains spillover from its one neighbour
/// only. Measured: `0.19350000000000003` — one ULP above the
/// correctly-rounded `0.1935`, matching the frozen mirror bit for bit.
#[test]
fn p3_end_chain_node_gains_one_term() {
    let (graph, _report) = run_territory();
    assert_eq!(
        heat(&graph, CHAIN_3).to_bits(),
        0.19350000000000003_f64.to_bits(),
        "chain-3: 0.18 (post-p1 decay) + 0.27 * 0.05, measured"
    );
}

/// `chain-1` starts phase 3 already AT the ceiling (p1's own clamp, from
/// heat 0.9 + 0.15 = 1.05). Adding any positive spillover inflow must
/// still land at EXACTLY 1.0 — the upper-only clamp (`min(1.0, …)`,
/// territory.py:315), not the two-sided p1 clamp.
#[test]
fn p3_near_ceiling_chain_node_clamps_at_exactly_one() {
    let (graph, _report) = run_territory();
    assert_eq!(heat(&graph, CHAIN_1), 1.0);
}

/// `isolated-fallback` carries ZERO ADJACENCY edges — the `exists` guard's
/// fallback branch takes `(- 0 0c)` (Real zero), so its heat ends the tick
/// BYTE-UNMOVED from whatever p1 already wrote (0.25 * 0.9 =
/// 0.225, LOW_PROFILE decay) — the write still happens (`set clamped`),
/// but the value is identical to what was already there.
#[test]
fn p3_isolated_territory_heat_is_byte_unmoved_by_spillover() {
    let (graph, _report) = run_territory();
    assert_eq!(
        heat(&graph, ISOLATED_FALLBACK).to_bits(),
        0.225_f64.to_bits(),
        "post-p1 decay only (0.25*0.9=0.225); p3 adds exactly zero"
    );
}

// ============================================================ Task 7: p4

fn organization(graph: &HypergraphStore, id: NodeId) -> f64 {
    graph
        .node_attribute(id, "social-class/organization")
        .unwrap_or_else(|e| panic!("node {id:?} social-class/organization: {}", e.message))
}

/// `concentration-camp` seeds population 500 and receives +100 THIS TICK
/// from `already-latched-to-camp`'s p2 eviction (its only qualifying
/// sink) — `floor(600 * (1 - 0.2)) = floor(480.0) = 480` proves the
/// p2 -> p4 same-tick sequencing (D116's relied-upon divergence): camp
/// decay eats this-tick displaced arrivals, not just the seed. Also pins
/// JC-D's remaining two value pins (`already-latched-to-camp`,
/// `concentration-camp`) in the SAME single-tick `run_territory()` world
/// the 2-tick rent-compounding test above deliberately does not use.
#[test]
fn p4_camp_decay_eats_this_ticks_displaced_arrivals() {
    let (graph, _report) = run_territory();
    assert_eq!(under_eviction(&graph, ALREADY_LATCHED_TO_CAMP), 1.0);
    assert_eq!(rent(&graph, ALREADY_LATCHED_TO_CAMP), 1_500_000.0);
    assert_eq!(population(&graph, ALREADY_LATCHED_TO_CAMP), 900.0);
    assert_eq!(
        heat(&graph, ALREADY_LATCHED_TO_CAMP).to_bits(),
        0.4545_f64.to_bits(),
        "0.5 * 0.9 (p1 decay) + 0.09 * 0.05 (p3, concentration-camp's post-p1 heat) \
         = 0.4545 — measured, matching the frozen mirror bit for bit"
    );
    assert_eq!(
        population(&graph, CONCENTRATION_CAMP),
        480.0,
        "500 seed + 100 same-tick arrival = 600; floor(600 * 0.8) = 480"
    );
    assert_eq!(
        under_eviction(&graph, CONCENTRATION_CAMP),
        0.0,
        "never itself latches"
    );
    assert_eq!(
        rent(&graph, CONCENTRATION_CAMP),
        1_000_000.0,
        "rent unspiked"
    );
    assert_eq!(
        heat(&graph, CONCENTRATION_CAMP).to_bits(),
        0.11250000000000002_f64.to_bits(),
        "concentration-camp's own heat: measured, matching the frozen mirror bit for bit"
    );
}

/// Both TENANCY-connected classes (`tenant-1`, `tenant-2`) have their
/// organization zeroed by `sink-penal`'s PENAL_COLONY suppression.
#[test]
fn p4_penal_suppression_zeroes_both_tenant_classes() {
    let (graph, _report) = run_territory();
    assert_eq!(organization(&graph, TENANT_1), 0.0);
    assert_eq!(organization(&graph, TENANT_2), 0.0);
}

/// The frozen law `test_social_class_without_tenancy_edge_is_untouched`,
/// transcribed: a class with NO TENANCY edge to the penal colony keeps
/// its seed value exactly.
#[test]
fn p4_the_unconnected_class_is_untouched() {
    let (graph, _report) = run_territory();
    assert_eq!(
        organization(&graph, NON_TENANT),
        0.6,
        "no TENANCY edge to sink-penal — left exactly as seeded"
    );
}

/// A RESERVATION territory is untouched by EITHER p4 rule — neither
/// `p4-camp-decay` (guards on CONCENTRATION_CAMP) nor
/// `p4-penal-suppression` (guards on PENAL_COLONY) fires for it.
/// `sink-reservation` lost the p2 tiebreak (population still its seed,
/// 200 — fix round MINOR-4: seeded NONZERO specifically so this
/// assertion discriminates; the previous 0 seed could not distinguish
/// "untouched" from "touched by a mutated guard that computes
/// `floor(0 * 0.8) = 0`" — a guard-flip mutation to CONCENTRATION_CAMP
/// or PENAL_COLONY would leave a zero-seeded node looking identical
/// either way), so this test isolates the p4-specific claim: nothing
/// ELSE wrote to it either.
#[test]
fn p4_reservation_territory_is_untouched() {
    let (graph, _report) = run_territory();
    assert_eq!(
        population(&graph, SINK_RESERVATION),
        200.0,
        "RESERVATION has no p4 rule of its own, and never won the p2 tiebreak — \
         floor(200 * 0.8) = 160 would be visible if a guard ever misrouted here"
    );
}

// ============================================================ Task 8

/// Full-pack e2e (Task 8, Step 1): load the scenario and ALL FIVE rules
/// through one `run_once_into` and assert STRUCTURAL agreement with the
/// frozen mirror's own printed post-tick state
/// (`territory_conformance.py`) — same nodes moved, same latch set, same
/// sink chosen, same suppression set — in ONE place, over the WHOLE
/// twelve-territory/three-class world, rather than spread across the
/// per-phase tests above. Every value here was independently confirmed
/// against `territory_conformance.py`'s own printed output before this
/// test was written (ADR183: STRUCTURE oracle, not a byte oracle) —
/// `per_rule_fired` additionally proves ALL FIVE rules ran, not just
/// `territory/p1-heat-dynamics`.
#[test]
fn full_pack_agrees_with_the_frozen_mirrors_structure() {
    let (graph, report) = run_territory();

    // Every rule fired at least once, and p1/p3 fired on all 12.
    let fired = |id: &str| -> Option<usize> {
        report
            .per_rule_fired
            .iter()
            .find(|(rid, _)| rid == id)
            .map(|(_, n)| *n)
    };
    assert_eq!(fired("territory/p1-heat-dynamics"), Some(12));
    assert_eq!(fired("territory/p2-eviction-pipeline"), Some(4)); // latch-tick-source, latch-no-sink, already-latched-to-camp, chain-1
    assert_eq!(fired("territory/p3-spillover"), Some(12));
    assert_eq!(fired("territory/p4-camp-decay"), Some(1)); // concentration-camp
    assert_eq!(fired("territory/p4-penal-suppression"), Some(1)); // sink-penal

    // The latch set (under-eviction == 1) is exactly the four sources p2
    // fired on above — no more, no less.
    for (id, expect_latched) in [
        (SUB_THRESHOLD_HIGH, false),
        (SUB_THRESHOLD_LOW, false),
        (LATCH_TICK_SOURCE, true),
        (SINK_PENAL, false),
        (SINK_RESERVATION, false),
        (LATCH_NO_SINK, true),
        (ALREADY_LATCHED_TO_CAMP, true),
        (CONCENTRATION_CAMP, false),
        (CHAIN_1, true),
        (CHAIN_2, false),
        (CHAIN_3, false),
        (ISOLATED_FALLBACK, false),
    ] {
        assert_eq!(
            under_eviction(&graph, id),
            f64::from(expect_latched),
            "node {id:?}: latch state"
        );
    }

    // The sink chosen: sink-penal (PENAL_COLONY, priority 3) received the
    // transfer, sink-reservation (priority 2, declared first — NIT-10) did
    // not; its seed (200, MINOR-4) stays untouched.
    assert_eq!(population(&graph, SINK_PENAL), 100.0);
    assert_eq!(population(&graph, SINK_RESERVATION), 200.0);

    // The suppression set: exactly the two TENANCY tenants of sink-penal.
    assert_eq!(organization(&graph, TENANT_1), 0.0);
    assert_eq!(organization(&graph, TENANT_2), 0.0);
    assert_eq!(organization(&graph, NON_TENANT), 0.6);

    // The camp decay total.
    assert_eq!(population(&graph, CONCENTRATION_CAMP), 480.0);
}
