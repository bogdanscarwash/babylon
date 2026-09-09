//! Conformance vectors for the `production/*` rule pack (P27, issue #565 —
//! the Production port train, `docs/superpowers/plans/2026-08-12-production-
//! port-plan.md`, plus a post-landing adversarial-verification fix round),
//! taken from the frozen Python engine's live behaviour.
//!
//! # Provenance
//!
//! Every STRUCTURAL claim below (which fields moved, in which direction, the
//! accumulation set, the extraction-intensity aggregate) was checked against
//! the frozen `ProductionSystem` running one `step()` over a fixture that
//! mirrors `content/scenarios/production-conformance.bscn` node for node.
//! The command, from the repository root:
//!
//! ```text
//! PYTHONPATH="$PWD/src" uv run python \
//!     rust/crates/babylon-tick/content/scenarios/production_conformance.py
//! ```
//!
//! The frozen system is the contract source for STRUCTURE and ORDERING, not
//! a correctness oracle (ADR183) — the port train's own D-records
//! (`production.bsl`'s header) name every place this pack's arithmetic
//! diverges from the frozen engine's field shape. Every NUMERIC value pinned
//! below is measured from the BSL engine's own output, never copied from the
//! frozen mirror's printed floats.
//!
//! # Fix round (adversarial verification, discharging D136)
//!
//! The ORIGINAL landing used a territory-side PULL fold for
//! `production/p4-extraction-intensity`, which double-counted a multi-
//! tenancy producer (`worker-pp-two-lands`) into every territory it held a
//! TENANCY edge to — a genuine divergence from the frozen engine's single-
//! territory attribution the original register row (D136) claimed had "no
//! `.bsl`-level fix available within port-as-is". That claim was FABRICATED:
//! a producer-side PUSH redesign (a fifth rule, `production/
//! p0-production-total-reset`, plus a third effect on p1/p2/p3) matches the
//! frozen engine EXACTLY, using only already-landed grammar. This file's
//! `T_BETA` pin is the direct evidence: it now agrees with the frozen mirror
//! bit for bit, where the original landing's own pin deliberately recorded a
//! divergence.
//!
//! # Scenario census
//!
//! Nine social classes, five territories, twelve edges (nine TENANCY, three
//! WAGES) — see `production-conformance.bscn`'s own header for the full
//! per-node conformance-case map. `worker-tight`/`t-tight` (MINOR-2, fix
//! round) are a pure addition at the end of the declaration order — every
//! other node's NodeId is unchanged from the original landing.

use babylon_bsl::scenario::load_scenario;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_kernel::SessionId;
use babylon_tick::run_once_into;

const SCENARIO: &str = include_str!("../content/scenarios/production-conformance.bscn");

// Node ids, fixed by the scenario's own declaration order
// (`production-conformance.bscn`'s own header names the same map).
const WORKER_PP: NodeId = NodeId(0);
const WORKER_PP_TWO_LANDS: NodeId = NodeId(1);
const WORKER_LA_ONE: NodeId = NodeId(2);
const WORKER_LA_TWO: NodeId = NodeId(3);
const WORKER_LA_ORPHAN: NodeId = NodeId(4);
const WORKER_LA_IDLE: NodeId = NodeId(5);
const COMPRADOR: NodeId = NodeId(6);
const EMPLOYER: NodeId = NodeId(7);
const T_ALPHA: NodeId = NodeId(8);
const T_BETA: NodeId = NodeId(9);
const T_DEAD: NodeId = NodeId(10);
const T_EMPTY: NodeId = NodeId(11);
// MINOR-2 (fix round): appended at the end, so every id above is unchanged
// from the original landing.
const WORKER_TIGHT: NodeId = NodeId(12);
const T_TIGHT: NodeId = NodeId(13);

/// The load-smoke test, through the REAL `run_once_into` seam — proves BOTH
/// halves the original plan named (`Expected: FAIL (unregistered system /
/// missing scenario)`).
///
/// **Deviation from the plan's literal text (plan line 41):** the plan
/// describes "an empty rule source"; `run_once_into`'s own `split_content`
/// refuses a content set with zero `(rule …)` top-forms outright
/// ("a content set needs at least one (rule …) top-form, found 0") —
/// confirmed by running exactly that against `lib.rs` before this rule
/// existed. A truly empty rule source therefore cannot exercise the
/// system-registration gate at all; it fails for an unrelated, earlier
/// reason. This test uses a minimal, never-firing probe rule instead, the
/// same idiom `territory_conformance.rs::a_no_op_rule_is_deterministic_
/// across_two_independent_loads` uses for the identical purpose — which DOES
/// reach the anchor check (`mod_anchors::check_anchor` against
/// `ctx.systems`, `rule_pipeline.rs:313`); `"production"` was NOT yet in
/// `lib.rs`'s registered-system `HashSet` (`lib.rs:174-205`) at the time
/// this test was first written, and the probe genuinely failed with an
/// unregistered-system anchor error — confirmed by running it before the
/// registration edit landed.
#[test]
fn scenario_loads_with_a_probe_pack() {
    const PROBE_RULE: &str = r#"
(rule production/probe
  :role mechanic :evidence derived :material-basis "load-only smoke: prove the scenario loads against a registered production system"
  :fuel 8
  (bindings (binding wealth :field social-class/wealth))
  (when (< wealth 0))
  (effects
    (update-node self social-class/wealth (set wealth))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    run_once_into(SCENARIO, PROBE_RULE, &mut graph, &mut sink)
        .expect("the scenario must load and run against a registered-system probe rule");
}

/// The scenario's own node/edge census, independent of any rule pack.
#[test]
fn the_scenario_loads_clean_with_the_declared_census() {
    let mut graph = HypergraphStore::new();
    let loaded = load_scenario(SCENARIO, &mut graph).expect("the scenario must load clean");
    assert_eq!(loaded.node_count, 14, "9 social classes + 5 territories");
    assert_eq!(loaded.edge_count, 12, "9 TENANCY + 3 WAGES");
    assert_eq!(
        loaded.node_types.get("SOCIAL_CLASS").copied(),
        Some(9),
        "nine social-class nodes"
    );
    assert_eq!(
        loaded.node_types.get("TERRITORY").copied(),
        Some(5),
        "five territory nodes"
    );
    assert_eq!(loaded.edge_types.get("TENANCY").copied(), Some(9));
    assert_eq!(loaded.edge_types.get("WAGES").copied(), Some(3));
}

/// Every field the pack's five rules read must be present on every node of
/// its own subject type (No-defaults contract) — a smoke read of all five
/// declared social-class fields and all four declared territory fields
/// (`territory/production-total` is new, fix round), before any rule exists
/// to touch them.
#[test]
fn every_node_seeds_all_its_declared_fields() {
    let mut graph = HypergraphStore::new();
    load_scenario(SCENARIO, &mut graph).expect("the scenario must load clean");
    // NOT a contiguous NodeId range for either type: `worker-tight`
    // (MINOR-2, fix round) is declared AFTER the four original territories,
    // so SOCIAL_CLASS is {0..=7, 12} and TERRITORY is {8..=11, 13} — an
    // explicit id list, not a range, is required.
    let social_class_ids: [u64; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 12];
    let territory_ids: [u64; 5] = [8, 9, 10, 11, 13];
    for id in social_class_ids {
        let node = NodeId(id);
        for field in [
            "social-class/role",
            "social-class/active",
            "social-class/population",
            "social-class/wealth",
            "social-class/production-value",
        ] {
            graph
                .node_attribute(node, field)
                .unwrap_or_else(|e| panic!("node {id} field {field}: {}", e.message));
        }
    }
    for id in territory_ids {
        let node = NodeId(id);
        for field in [
            "territory/biocapacity",
            "territory/max-biocapacity",
            "territory/extraction-intensity",
            "territory/production-total",
        ] {
            graph
                .node_attribute(node, field)
                .unwrap_or_else(|e| panic!("node {id} field {field}: {}", e.message));
        }
    }
}

// ============================================================ p1

const PRODUCTION_RULE: &str = include_str!("../content/rules/production.bsl");

fn run_production() -> (HypergraphStore, babylon_tick::TickReport) {
    let mut graph = HypergraphStore::new();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let report = run_once_into(SCENARIO, PRODUCTION_RULE, &mut graph, &mut sink)
        .expect("the production pack must load and run");
    (graph, report)
}

fn wealth(graph: &HypergraphStore, id: NodeId) -> f64 {
    graph
        .node_attribute(id, "social-class/wealth")
        .unwrap_or_else(|e| panic!("node {id:?} social-class/wealth: {}", e.message))
}

fn production_value(graph: &HypergraphStore, id: NodeId) -> f64 {
    graph
        .node_attribute(id, "social-class/production-value")
        .unwrap_or_else(|e| panic!("node {id:?} social-class/production-value: {}", e.message))
}

fn production_total(graph: &HypergraphStore, id: NodeId) -> f64 {
    graph
        .node_attribute(id, "territory/production-total")
        .unwrap_or_else(|e| panic!("node {id:?} territory/production-total: {}", e.message))
}

fn extraction_intensity(graph: &HypergraphStore, id: NodeId) -> f64 {
    graph
        .node_attribute(id, "territory/extraction-intensity")
        .unwrap_or_else(|e| panic!("node {id:?} territory/extraction-intensity: {}", e.message))
}

/// `production/p1-direct-production`: `worker-pp` (PERIPHERY_PROLETARIAT,
/// TENANCY to `t-alpha`, biocapacity 80/100) accumulates its own wealth by
/// exactly `(base_labor_power / weeks_per_year) * population * bio_ratio` =
/// `(1.0/52) * 100 * 0.8` — measured, matching the frozen mirror's own
/// printed `worker-pp wealth=11.538461538461538` bit for bit (this rule's
/// binding chain is a direct, unreordered transcription of production.py's
/// own `(effective_labor_power * population) * bio_ratio`, so bit-identical
/// agreement is the expected, not merely hoped-for, result — ADR183 still
/// requires it be measured, not assumed).
#[test]
fn p1_direct_producer_accumulates_own_wealth() {
    let (graph, _report) = run_production();
    assert_eq!(
        wealth(&graph, WORKER_PP).to_bits(),
        11.538461538461538_f64.to_bits(),
        "10 + (1/52)*100*0.8, measured — matches the frozen mirror's own printed float bit for bit"
    );
    assert_eq!(
        production_value(&graph, WORKER_PP).to_bits(),
        1.5384615384615385_f64.to_bits(),
        "the produced value itself, written to the new per-node ledger field"
    );
}

/// p1's `when` guard is role-scoped to `PERIPHERY_PROLETARIAT` alone —
/// `comprador` (COMPRADOR_BOURGEOISIE, holds a TENANCY edge to `t-alpha`)
/// never matches ANY of the three producer rules' `when` guards (p1/p2/p3
/// are all role-scoped to PERIPHERY_PROLETARIAT/LABOR_ARISTOCRACY), so its
/// seed wealth (10.0) stays exactly as seeded across the WHOLE pack — the
/// p4 filter vector's own precondition, witnessed here at the wealth field
/// too, not just at `production-value` (p4's own test). Also never pushes
/// anything onto `t-alpha`'s `production-total` (the fix round's own
/// producer-side design).
#[test]
fn comprador_wealth_is_never_moved_by_any_producer_rule() {
    let (graph, _report) = run_production();
    assert_eq!(
        wealth(&graph, COMPRADOR),
        10.0,
        "comprador: not a producer role, in any of p1/p2/p3's when guards"
    );
}

// ============================================================ p2/p3

/// `production/p2-employed-routing`: BOTH `worker-la-one` and
/// `worker-la-two` (LABOR_ARISTOCRACY, TENANCY to `t-beta`, WAGES from the
/// SAME `employer`) route their product to `employer`'s wealth — the
/// D103/D104 accumulate-into-a-shared-target shape, this pack's first
/// content-pack instance of it. `employer`'s seed wealth (10.0) plus BOTH
/// contributions, added in subject (ascending NodeId) order — measured,
/// matching the frozen mirror's own printed
/// `employer wealth=10.961538461538462` bit for bit.
#[test]
fn p2_two_la_products_accumulate_into_one_employer() {
    let (graph, _report) = run_production();
    assert_eq!(
        wealth(&graph, EMPLOYER).to_bits(),
        10.961538461538462_f64.to_bits(),
        "10 + worker-la-one's product + worker-la-two's product, both kept — measured, \
         matches the frozen mirror's own printed float bit for bit"
    );
}

/// `worker-la-idle` (LABOR_ARISTOCRACY, active=0, TENANCY to `t-beta`,
/// WAGES from `employer`) — p2's `when` guard is role+employer-existence+
/// tenancy-existence, not `active`, so it FIRES (unlike the frozen engine's
/// own `continue` skip for an inactive worker), but the active-gated
/// `output` binding computes to 0: `employer`'s wealth is unaffected by this
/// worker specifically (isolated from
/// `p2_two_la_products_accumulate_into_one_employer`'s own two
/// contributions by comparing against the SAME pinned total), and this
/// worker's own `production-value` stays at its seeded 0 — the D127
/// hash-neutral idiom.
#[test]
fn p2_idle_la_adds_nothing() {
    let (graph, _report) = run_production();
    assert_eq!(
        production_value(&graph, WORKER_LA_IDLE),
        0.0,
        "inactive: output forced to 0, (set 0) matches the seed — hash-neutral"
    );
    assert_eq!(
        wealth(&graph, EMPLOYER).to_bits(),
        10.961538461538462_f64.to_bits(),
        "employer's total is UNCHANGED by worker-la-idle's own (add 0) — same pin as the \
         two-LA accumulation test, proving the idle worker contributed nothing"
    );
}

/// `production/p3-employed-fallback`: `worker-la-orphan` (LABOR_ARISTOCRACY,
/// TENANCY to `t-alpha`, NO WAGES edge) keeps its own product — the frozen
/// fallback (production.py:196-198). Measured, matching the frozen mirror's
/// own printed `worker-la-orphan wealth=10.461538461538462` bit for bit.
#[test]
fn p3_orphan_la_keeps_own_product() {
    let (graph, _report) = run_production();
    assert_eq!(
        wealth(&graph, WORKER_LA_ORPHAN).to_bits(),
        10.461538461538462_f64.to_bits(),
        "10 + (1/52)*30*0.8, measured — matches the frozen mirror's own printed float bit \
         for bit"
    );
    assert_eq!(
        production_value(&graph, WORKER_LA_ORPHAN).to_bits(),
        0.4615384615384616_f64.to_bits(),
        "the produced value itself"
    );
}

/// The RESERVED-LINE routing structure, asserted directly: an EMPLOYED
/// producer's OWN wealth never moves — the product routes AWAY, to the
/// employer, exactly as production.py:184-194 (Amin/Wallerstein) transcribes.
#[test]
fn la_wealth_unmoved_by_p2() {
    let (graph, _report) = run_production();
    assert_eq!(
        wealth(&graph, WORKER_LA_ONE),
        10.0,
        "worker-la-one: product routed to employer, own wealth untouched"
    );
    assert_eq!(
        wealth(&graph, WORKER_LA_TWO),
        10.0,
        "worker-la-two: product routed to employer, own wealth untouched"
    );
}

// ============================================================ p0/p4

/// `production/p0-production-total-reset` + the three producer rules'
/// pushes: `t-alpha`'s `production-total` is the SUM of `worker-pp` +
/// `worker-pp-two-lands` (its own computed value, via the p1 tiebreak that
/// picked `t-alpha`) + `worker-la-orphan` — but NOT `comprador` (never
/// fires any producer rule, never pushes). Measured, matches the frozen
/// mirror's own printed `t-alpha extraction_intensity=0.027692307692307697`
/// bit for bit once divided by `max-biocapacity` — pinned here at the
/// `production-total` field directly, and again at `extraction-intensity`
/// below.
#[test]
fn p0_p1_p3_push_t_alphas_production_total_correctly() {
    let (graph, _report) = run_production();
    assert_eq!(
        production_total(&graph, T_ALPHA).to_bits(),
        2.7692307692307696_f64.to_bits(),
        "worker-pp + worker-pp-two-lands + worker-la-orphan production-values, pushed onto \
         t-alpha by their OWN select-max tiebreak ref — measured"
    );
    assert_eq!(
        extraction_intensity(&graph, T_ALPHA).to_bits(),
        0.027692307692307697_f64.to_bits(),
        "measured — matches the frozen mirror's own printed float bit for bit"
    );
}

/// `t-beta`'s own `production-total` — THE FIX ROUND'S OWN DISCHARGE
/// VECTOR. `worker-pp-two-lands` holds TWO TENANCY edges (`t-alpha` AND
/// `t-beta`), but its push effect targets ONLY the D45 tiebreak winner
/// (`t-alpha`, the lower NodeId) — the SAME ref its `bio`/`max-bio`
/// bindings already select — so `t-beta`'s own total is `worker-la-one` +
/// `worker-la-two` + `worker-la-idle` (0, inactive) ONLY, excluding
/// `worker-pp-two-lands` entirely. This is the corrected behaviour: the
/// ORIGINAL (pull-fold) landing double-counted `worker-pp-two-lands` into
/// `t-beta` here, diverging from the frozen mirror's own printed `t-beta
/// extraction_intensity=0.009615384615384616`. This pack's measured value
/// now agrees with it BIT FOR BIT — the adversarial-verification finding,
/// discharged.
#[test]
fn p4_extraction_matches_frozen_single_territory_attribution() {
    let (graph, _report) = run_production();
    assert_eq!(
        production_total(&graph, T_BETA).to_bits(),
        0.9615384615384617_f64.to_bits(),
        "worker-la-one + worker-la-two + worker-la-idle(0) — worker-pp-two-lands EXCLUDED, \
         matching the frozen engine's single-territory attribution — measured"
    );
    assert_eq!(
        extraction_intensity(&graph, T_BETA).to_bits(),
        0.009615384615384616_f64.to_bits(),
        "measured — matches the frozen mirror's own printed float bit for bit (the ORIGINAL \
         landing's own pin, 0.01730769230769231, was the double-counted divergence this fix \
         round corrects)"
    );
}

/// `t-dead` (biocapacity 0, max-biocapacity 0, NO TENANCY edges at all) —
/// the zero-guard vector: `production-total` stays at p0's reset value, `0`
/// (no producer's push ever targets it — none has a TENANCY edge there),
/// and `max-bio > 0` is false regardless, so `ratio` is forced to `0`.
#[test]
fn p4_zero_max_biocapacity_yields_zero() {
    let (graph, _report) = run_production();
    assert_eq!(production_total(&graph, T_DEAD), 0.0);
    assert_eq!(
        extraction_intensity(&graph, T_DEAD),
        0.0,
        "max-biocapacity <= 0 forces the ratio to 0, matching production.py:267's own \
         zero-guard (`intensity = ... if max_biocapacity > 0 else 0.0`)"
    );
}

/// `t-empty` (biocapacity 100, max-biocapacity 100, NO TENANTS) — the
/// no-production vector: no producer's push ever targets it, so
/// `production-total` stays at p0's reset value, `0`, and `0 / 100 = 0`.
#[test]
fn p4_no_tenants_yields_zero() {
    let (graph, _report) = run_production();
    assert_eq!(production_total(&graph, T_EMPTY), 0.0);
    assert_eq!(
        extraction_intensity(&graph, T_EMPTY),
        0.0,
        "no TENANCY-incident producers at all — production-total stays at p0's reset value"
    );
}

/// `t-tight` (MINOR-2, fix round: biocapacity 1, max-biocapacity 1,
/// `worker-tight` the sole TENANCY-incident producer, population 100) —
/// the upper-clamp LIVE vector: `worker-tight`'s own push
/// (`(1/52)*100*1.0 = 1.9230769230769231`) exceeds `max-biocapacity` (1), so
/// `ratio > 1` and the clamp's CONSTANT branch (`(- 1 0c)`) is taken, not
/// merely present — the original four territories' own ratios never reach
/// it (`p4_all_original_territories_stay_sub_one` below).
#[test]
fn p4_upper_clamp_is_live_at_t_tight() {
    let (graph, _report) = run_production();
    assert_eq!(
        production_total(&graph, T_TIGHT).to_bits(),
        1.9230769230769231_f64.to_bits(),
        "(1/52)*100*1.0 — worker-tight's sole contribution — measured"
    );
    assert_eq!(
        extraction_intensity(&graph, T_TIGHT),
        1.0,
        "ratio (1.923...) exceeds 1 — the clamp's constant branch is taken, not the \
         pass-through — matches the frozen mirror's own printed \
         t-tight extraction_intensity=1.0"
    );
}

/// The original four territories' own ratios stay comfortably sub-1.0 —
/// `t-tight` above is what makes the clamp's constant branch mutation-live;
/// this test isolates the claim that the ORIGINAL fixture alone never did.
#[test]
fn p4_all_original_territories_stay_sub_one() {
    let (graph, _report) = run_production();
    for id in [T_ALPHA, T_BETA, T_DEAD, T_EMPTY] {
        let intensity = extraction_intensity(&graph, id);
        assert!(
            (0.0..1.0).contains(&intensity),
            "node {id:?}: extraction_intensity {intensity} must be in [0, 1) — none of the \
             original four territories' own seeded producer totals are large enough to reach \
             the clamp's ceiling branch"
        );
    }
}

/// Byte order, verified empirically via per-rule fired counts (fix round):
/// `production/p0-production-total-reset` fires on EVERY territory (five),
/// unconditionally, exactly like `production/p4-extraction-intensity` — both
/// always-fire (`(when #t)`), same idiom `territory/p1-heat-dynamics` uses.
#[test]
fn p0_and_p4_fire_on_every_territory() {
    let (_graph, report) = run_production();
    let fired = |id: &str| -> Option<usize> {
        report
            .per_rule_fired
            .iter()
            .find(|(rid, _)| rid == id)
            .map(|(_, n)| *n)
    };
    assert_eq!(
        fired("production/p0-production-total-reset"),
        Some(5),
        "all five territories, unconditional"
    );
    assert_eq!(
        fired("production/p4-extraction-intensity"),
        Some(5),
        "all five territories, unconditional"
    );
}

// ============================================================ Mutation: p0's reset

/// **Mutation evidence for p0's reset, fix round.** A SINGLE tick cannot
/// distinguish "p0 resets production-total to 0" from "p0 does nothing" —
/// every territory's `production-total` is ALREADY seeded `0`, so a missing
/// reset would be invisible in one tick. This test runs TWO ticks via
/// `TickSession` (same idiom `territory_conformance.rs::
/// p2_already_latched_territory_compounds_rent_across_two_ticks` uses):
/// every input to `t-alpha`'s own computation (`active`, `population`,
/// `biocapacity`, `max-biocapacity`) is tick-invariant in this fixture, so
/// with a WORKING reset, tick 2's `extraction-intensity` must be BIT-
/// IDENTICAL to tick 1's — a fresh `0` each tick, then the SAME
/// contributions re-added. If p0's reset is broken (mutated to a no-op —
/// evidence recorded in `docs/reference/bsl-language.rst` register row
/// D132/D136 and this fix round's own commit body, not reproduced as a
/// standing test here, since a permanently-mutated pack would have to
/// replace this file's single shared pack constant), tick 2's `production-total` carries tick 1's
/// total FORWARD and adds tick 2's contributions on top — MEASURED, not
/// estimated: `t-alpha`'s tick-2 extraction-intensity becomes
/// `0.05538461538461539`, EXACTLY DOUBLE tick 1's `0.027692307692307697`.
#[test]
fn p0_reset_keeps_extraction_intensity_stable_across_two_ticks() {
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let mut session = babylon_tick::TickSession::new(
        SCENARIO,
        PRODUCTION_RULE,
        HypergraphStore::new(),
        SessionId::new("production-conformance-test").expect("literal is non-empty"),
    )
    .expect("the pack must load into a session");
    session.advance(&mut sink).expect("tick 1");
    let after_tick_1 = extraction_intensity(session.graph(), T_ALPHA);
    session.advance(&mut sink).expect("tick 2");
    let after_tick_2 = extraction_intensity(session.graph(), T_ALPHA);
    assert_eq!(
        after_tick_1.to_bits(),
        0.027692307692307697_f64.to_bits(),
        "tick 1: same value the single-tick test pins"
    );
    assert_eq!(
        after_tick_2.to_bits(),
        after_tick_1.to_bits(),
        "tick 2: BIT-IDENTICAL to tick 1 — every input is tick-invariant, and a working reset \
         means the accumulator starts fresh each tick rather than compounding"
    );
}

// ============================================================ Full pack

/// Full-pack e2e: load the scenario and ALL FIVE rules through one
/// `run_once_into` and assert STRUCTURAL agreement with the frozen mirror's
/// own printed post-tick state (`production_conformance.py`) — same nodes
/// moved, same accumulation set, same extraction-intensity set — in ONE
/// place, over the WHOLE fourteen-node world, rather than spread across the
/// per-rule tests above. `per_rule_fired` proves ALL FIVE rules ran with the
/// exact counts this fix round's own arithmetic predicted
/// (p0x5 + p1x3 + p2x3 + p3x1 + p4x5 = 17, verified here rather than
/// trusted) — `territory_conformance.rs`'s own
/// `full_pack_agrees_with_the_frozen_mirrors_structure` is the precedent
/// this test follows.
#[test]
fn full_pack_agrees_with_the_frozen_mirrors_structure() {
    let (graph, report) = run_production();

    let fired = |id: &str| -> Option<usize> {
        report
            .per_rule_fired
            .iter()
            .find(|(rid, _)| rid == id)
            .map(|(_, n)| *n)
    };
    assert_eq!(fired("production/p0-production-total-reset"), Some(5));
    assert_eq!(fired("production/p1-direct-production"), Some(3));
    assert_eq!(fired("production/p2-employed-routing"), Some(3));
    assert_eq!(fired("production/p3-employed-fallback"), Some(1));
    assert_eq!(fired("production/p4-extraction-intensity"), Some(5));
    assert_eq!(
        report.fired, 17,
        "p0x5 + p1x3 + p2x3 + p3x1 + p4x5 = 17 -- fix round arithmetic, verified"
    );

    // The wealth ledger: every producer's own wealth or its employer's,
    // measured against the frozen mirror bit for bit where the two engines
    // agree (every entry here does -- wealth is a per-worker field, never
    // multi-tenancy-affected the way extraction-intensity used to be).
    assert_eq!(
        wealth(&graph, WORKER_PP).to_bits(),
        11.538461538461538_f64.to_bits()
    );
    assert_eq!(
        wealth(&graph, WORKER_PP_TWO_LANDS).to_bits(),
        10.76923076923077_f64.to_bits()
    );
    assert_eq!(wealth(&graph, WORKER_LA_ONE), 10.0);
    assert_eq!(wealth(&graph, WORKER_LA_TWO), 10.0);
    assert_eq!(
        wealth(&graph, WORKER_LA_ORPHAN).to_bits(),
        10.461538461538462_f64.to_bits()
    );
    assert_eq!(
        wealth(&graph, WORKER_LA_IDLE),
        10.0,
        "idle: p2 fires but its output is hash-neutrally zeroed by the active gate"
    );
    assert_eq!(wealth(&graph, COMPRADOR), 10.0);
    assert_eq!(
        wealth(&graph, EMPLOYER).to_bits(),
        10.961538461538462_f64.to_bits()
    );
    assert_eq!(
        wealth(&graph, WORKER_TIGHT).to_bits(),
        11.923076923076923_f64.to_bits(),
        "MINOR-2 (fix round): matches the frozen mirror's own printed float bit for bit"
    );

    // The idle-worker vector, named explicitly: worker-la-idle's firing
    // moved nothing observable anywhere in the graph -- its own wealth, the
    // employer's wealth, and its own production-value are all exactly as
    // seeded.
    assert_eq!(production_value(&graph, WORKER_LA_IDLE), 0.0);

    // The extraction-intensity broadcast: t-alpha and t-beta BOTH agree
    // with the frozen mirror bit for bit now (the fix round's own
    // discharge); both zero-guard territories land at exactly 0.0;
    // t-tight clamps at exactly 1.0 (MINOR-2).
    assert_eq!(
        extraction_intensity(&graph, T_ALPHA).to_bits(),
        0.027692307692307697_f64.to_bits()
    );
    assert_eq!(
        extraction_intensity(&graph, T_BETA).to_bits(),
        0.009615384615384616_f64.to_bits(),
        "fix round: now bit-identical to the frozen mirror, not the double-counted divergence"
    );
    assert_eq!(extraction_intensity(&graph, T_DEAD), 0.0);
    assert_eq!(extraction_intensity(&graph, T_EMPTY), 0.0);
    assert_eq!(extraction_intensity(&graph, T_TIGHT), 1.0);
}
