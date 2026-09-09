//! Conformance vectors for the `class-dynamics/*` rule pack (Feature-016,
//! issue #669 — the TickDynamics class-dynamics port train,
//! `docs/superpowers/plans/2026-08-18-tickdynamics-port.md`).
//!
//! # Task 2 scope
//!
//! This file lands the registration refusal, the world-1 declaration surface,
//! the frozen-mirror corroboration, and the permanent anti-pattern guards.
//! Per-task rule assertions follow in later tasks.
//!
//! # Provenance
//!
//! The frozen mirror is `content/scenarios/class_dynamics_conformance.py`.
//! It is the structure/ordering oracle, not a byte oracle (ADR183). Every
//! numeric value pinned below is measured from the BSL engine's own output.
//!
//! # Mirror verbatim stdout (2026-08-22, `uv run python`)
//!
//! The mirror's term-for-term transcription and `DefaultClassTransitionEngine`
//! corroboration pass agree exactly (the `_assert_agreement` STOP-first check
//! exits 0). The F11 double run shows the frozen `wage·s²` accumulation versus
//! the repaired `wage·s`:
//!
//! ```text
//! class-dynamics-conformance — frozen mirror (world 1)
//!
//! == wayne (frozen) ==
//!   county_fips = 26163
//!   wage_hourly = 21.0
//!   wage_annual = 43680.0
//!   phi_per_hour = 0.0
//!   phi_adjustment = 0.0
//!   effective_savings_rate = 0.03
//!   annual_accumulation_dollars = 39.31200000000004
//!   rate_accumulation_per_year = 0.0002768450704225355
//!   rate_dispossession_per_year = 0.0117
//!   rate_precaritization_per_year = 0.0565
//!   rate_stabilization_per_year = 0.1425
//!   la_before = 0.4
//!   prol_before = 0.35
//!   lumpen_before = 0.15
//!   la_after = 0.3954168957746479
//!   prol_after = 0.3561831042253521
//!   lumpen_after = 0.1484
//!   total_share_check = 0.9999999999999999
//!
//! == wayne (repaired) ==
//!   county_fips = 26163
//!   wage_hourly = 21.0
//!   wage_annual = 43680.0
//!   phi_per_hour = 0.0
//!   phi_adjustment = 0.0
//!   effective_savings_rate = 0.03
//!   annual_accumulation_dollars = 1310.3999999999999
//!   rate_accumulation_per_year = 0.009228169014084506
//!   rate_dispossession_per_year = 0.0117
//!   rate_precaritization_per_year = 0.0565
//!   rate_stabilization_per_year = 0.1425
//!   la_before = 0.4
//!   prol_before = 0.35
//!   lumpen_before = 0.15
//!   la_after = 0.3985498591549296
//!   prol_after = 0.3530501408450704
//!   lumpen_after = 0.1484
//!   total_share_check = 0.9999999999999999
//!
//! == oakland (frozen) ==
//!   county_fips = 06001
//!   wage_hourly = 25.0
//!   wage_annual = 52000.0
//!   phi_per_hour = 0.0
//!   phi_adjustment = 0.0
//!   effective_savings_rate = 0.03
//!   annual_accumulation_dollars = 46.8
//!   rate_accumulation_per_year = 0.00032957746478873236
//!   rate_dispossession_per_year = 0.0117
//!   rate_precaritization_per_year = 0.0815
//!   rate_stabilization_per_year = 0.135
//!   la_before = 0.35
//!   prol_before = 0.4
//!   lumpen_before = 0.15
//!   la_after = 0.34603683098591553
//!   prol_after = 0.3916131690140846
//!   lumpen_after = 0.16235000000000002
//!   total_share_check = 1.0000000000000002
//!
//! == oakland (repaired) ==
//!   county_fips = 06001
//!   wage_hourly = 25.0
//!   wage_annual = 52000.0
//!   phi_per_hour = 0.0
//!   phi_adjustment = 0.0
//!   effective_savings_rate = 0.03
//!   annual_accumulation_dollars = 1560.0
//!   rate_accumulation_per_year = 0.010985915492957746
//!   rate_dispossession_per_year = 0.0117
//!   rate_precaritization_per_year = 0.0815
//!   rate_stabilization_per_year = 0.135
//!   la_before = 0.35
//!   prol_before = 0.4
//!   lumpen_before = 0.15
//!   la_after = 0.3502993661971831
//!   prol_after = 0.38735063380281703
//!   lumpen_after = 0.16235000000000002
//!   total_share_check = 1.0000000000000002
//!
//! F11 headline (wayne, phi=0, savings=0.03):
//!   frozen annual accumulation (wage·s²)   = 39.31200000000004
//!   repaired annual accumulation (wage·s)  = 1310.3999999999999
//!   ratio repaired/frozen                  = 33.33333333333329
//! ```
//!
//! # Red-phase evidence
//!
//! Before `class-dynamics` was registered in `lib.rs`, the probe rule below
//! refused with:
//! `rule class-dynamics/probe rejected: E-LOAD-002: rule class-dynamics/probe
//! carries no anchor and its first id segment names no registered system —
//! a rule cannot land nowhere (§2.3)`.

use babylon_bsl::scenario::load_scenario;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::run_once_into;

const SCENARIO: &str = include_str!("../content/scenarios/class-dynamics-conformance.bscn");
const PACK: &str = include_str!("../content/rules/class-dynamics.bsl");

// Node ids, fixed by the scenario's own declaration order.
const WAYNE: NodeId = NodeId(0);
const OAKLAND: NodeId = NodeId(1);
const WAYNE_PROLE: NodeId = NodeId(2);
const WAYNE_LA: NodeId = NodeId(3);
const SHARED_CLASS: NodeId = NodeId(4);
const ORPHAN_CLASS: NodeId = NodeId(5);

/// Step 2 green load-smoke: the scenario hydrates and ticks clean under a
/// minimal `class-dynamics/` probe rule. The probe never fires (`crisis-phase`
/// is `NORMAL`, not `ONSET`), so the pre/post hashes are identical — the hash
/// brackets the tick, not the hydration.
#[test]
fn scenario_loads_with_a_class_dynamics_probe() {
    const PROBE_RULE: &str = r#"
(rule class-dynamics/probe
  :role mechanic :evidence derived :material-basis "Task 2 Step 2 green load-smoke: prove registration + scenario load"
  :fuel 8
  (bindings (binding phase :field territory/crisis-phase))
  (when (= phase CrisisPhase/ONSET))
  (effects
    (update-node self territory/bifurcation-score (set 0))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, PROBE_RULE, &mut graph, &mut sink)
        .expect("the scenario must load and run against a registered class-dynamics probe");
    assert_eq!(report.fired, 0, "the probe never fires");
    assert_eq!(
        report.before, report.after,
        "a never-firing probe writes nothing"
    );
}

/// The scenario's own node/edge census, independent of any rule pack.
#[test]
fn the_scenario_loads_clean_with_the_declared_census() {
    let mut graph = HypergraphStore::new();
    let loaded = load_scenario(SCENARIO, &mut graph).expect("the scenario must load clean");
    assert_eq!(loaded.node_count, 6, "2 territories + 4 social classes");
    assert_eq!(loaded.edge_count, 4, "four TENANCY edges");
    assert_eq!(loaded.node_types.get("TERRITORY").copied(), Some(2));
    assert_eq!(loaded.node_types.get("SOCIAL_CLASS").copied(), Some(4));
    assert_eq!(loaded.edge_types.get("TENANCY").copied(), Some(4));
}

/// Every field the pack will read is present on every node of its namespace
/// before any rule exists to touch it (no-defaults contract).
#[test]
fn every_node_seeds_all_its_declared_fields() {
    let mut graph = HypergraphStore::new();
    load_scenario(SCENARIO, &mut graph).expect("the scenario must load clean");
    for id in [WAYNE, OAKLAND] {
        for field in [
            "territory/share-bourgeoisie",
            "territory/share-petit-bourgeoisie",
            "territory/share-labor-aristocracy",
            "territory/share-proletariat",
            "territory/share-lumpenproletariat",
            "territory/dist-year",
            "territory/baseline-la-share",
            "territory/baseline-la-known",
            "territory/unemployment-rate",
            "territory/median-wage",
            "territory/phi-hour",
            "territory/foreclosure-rate",
            "territory/bankruptcy-rate",
            "territory/eviction-rate",
            "territory/crisis-phase",
            "territory/phi-savings-adjustment",
            "territory/bifurcation-score",
            "territory/rate-accumulation",
            "territory/rate-dispossession",
            "territory/rate-precaritization",
            "territory/rate-stabilization",
            "territory/raw-share-labor-aristocracy",
            "territory/raw-share-proletariat",
            "territory/raw-share-lumpenproletariat",
        ] {
            graph
                .node_attribute(id, field)
                .unwrap_or_else(|e| panic!("node {id:?} field {field}: {}", e.message));
        }
    }
    for id in [WAYNE_PROLE, WAYNE_LA, SHARED_CLASS, ORPHAN_CLASS] {
        for field in [
            "social-class/ternary-net-fascist",
            "social-class/revolutionary",
            "social-class/fascist",
            "social-class/population",
        ] {
            graph
                .node_attribute(id, field)
                .unwrap_or_else(|e| panic!("node {id:?} field {field}: {}", e.message));
        }
    }
}

/// ADR195 ordinal parity: `CrisisPhase` order is the frozen Python declaration
/// order (`src/babylon/domain/economics/tick/types.py`). A transposition here
/// silently re-reads every seeded `territory/crisis-phase`.
#[test]
fn defenum_ordinal_parity_with_the_frozen_order() {
    let mut graph = HypergraphStore::new();
    let loaded = load_scenario(SCENARIO, &mut graph).expect("world 1 loads");
    let phase = loaded
        .enums
        .resolve("CrisisPhase")
        .expect("CrisisPhase declared");
    for (expected, member) in ["NORMAL", "ONSET", "EARLY", "DEEP", "RECOVERY"]
        .iter()
        .enumerate()
    {
        assert_eq!(
            loaded.enums.ordinal(phase, member),
            Some(expected as u32),
            "CrisisPhase/{member} must be ordinal {expected}"
        );
    }
}

/// Canonical-table parity (Task 2 lands world 1 only, so this asserts
/// world 1 against the canonical block; when later worlds land, this
/// harness extends to a loop over a `scenarios` array — every constant a
/// world does not deliberately vary must be byte-identical across worlds,
/// so a typo in one world's canonical block cannot pass).
#[test]
fn world1_constants_match_the_canonical_table_bit_exactly() {
    let expected: [(&str, f64); 46] = [
        // Engine parameters — transition_engine.py:51-54.
        ("class-dynamics/wealth-threshold", 142_000.0),
        ("class-dynamics/precaritization-unemployment-weight", 0.5),
        ("class-dynamics/base-stabilization", 0.15),
        ("class-dynamics/max-accumulation-rate", 0.08),
        // Phased amplification table — crisis.py:24-55 (20 constants).
        ("class-dynamics/amp-normal-dispossession", 1.0),
        ("class-dynamics/amp-normal-precaritization", 1.0),
        ("class-dynamics/amp-normal-accumulation", 1.0),
        ("class-dynamics/amp-normal-stabilization", 1.0),
        ("class-dynamics/amp-onset-dispossession-x1e6", 1_200_000.0),
        ("class-dynamics/amp-onset-precaritization-x1e6", 1_500_000.0),
        ("class-dynamics/amp-onset-accumulation", 0.8),
        ("class-dynamics/amp-onset-stabilization", 0.7),
        ("class-dynamics/amp-early-dispossession-x1e6", 1_800_000.0),
        ("class-dynamics/amp-early-precaritization-x1e6", 2_500_000.0),
        ("class-dynamics/amp-early-accumulation", 0.4),
        ("class-dynamics/amp-early-stabilization", 0.4),
        ("class-dynamics/amp-deep-dispossession-x1e6", 3_000_000.0),
        ("class-dynamics/amp-deep-precaritization-x1e6", 3_500_000.0),
        ("class-dynamics/amp-deep-accumulation", 0.1),
        ("class-dynamics/amp-deep-stabilization", 0.2),
        (
            "class-dynamics/amp-recovery-dispossession-x1e6",
            1_300_000.0,
        ),
        (
            "class-dynamics/amp-recovery-precaritization-x1e6",
            1_200_000.0,
        ),
        ("class-dynamics/amp-recovery-accumulation", 0.6),
        ("class-dynamics/amp-recovery-stabilization", 0.5),
        // Dispossession composite weights (LA→P) — dispossession.py:30-36.
        ("class-dynamics/foreclosure-weight-la-to-p", 0.6),
        ("class-dynamics/bankruptcy-weight-la-to-p", 0.3),
        ("class-dynamics/eviction-weight-la-to-p", 0.1),
        // Savings schedule — savings_schedule.py:21-31.
        ("class-dynamics/savings-proletariat", 0.03),
        ("class-dynamics/phi-cap", 0.05),
        // Dispossession-rate defaults — system/__init__.py:2383-2385.
        ("class-dynamics/default-foreclosure-rate", 0.006),
        ("class-dynamics/default-bankruptcy-rate", 0.006),
        ("class-dynamics/default-eviction-rate", 0.063),
        // Wage / subsistence.
        ("class-dynamics/hours-per-year", 2_080.0),
        ("class-dynamics/v-reproduction", 12.0),
        ("class-dynamics/accumulation-halt-floor-ratio", 0.8),
        // Bootstrap five-class shares — types.py:44-48 / R5.
        ("class-dynamics/bootstrap-bourgeoisie", 0.01),
        ("class-dynamics/bootstrap-petit-bourgeoisie", 0.09),
        ("class-dynamics/bootstrap-labor-aristocracy", 0.40),
        ("class-dynamics/bootstrap-proletariat", 0.35),
        ("class-dynamics/bootstrap-lumpenproletariat", 0.15),
        // Cascade milestones + bifurcation threshold — R5 / R6.
        ("class-dynamics/cascade-milestone-1", 0.05),
        ("class-dynamics/cascade-milestone-2", 0.10),
        ("class-dynamics/cascade-milestone-3", 0.15),
        ("class-dynamics/bifurcation-event-threshold", 0.5),
        // Year clamp window.
        ("class-dynamics/year-min", 2_007.0),
        ("class-dynamics/year-max", 2_030.0),
    ];

    let mut graph = HypergraphStore::new();
    let loaded = load_scenario(SCENARIO, &mut graph).expect("world 1 loads");
    for (name, value) in &expected {
        let actual = loaded
            .consts
            .get(*name)
            .unwrap_or_else(|| panic!("{name} missing from world 1"));
        let bits = match actual {
            babylon_bsl::evaluator::Value::Real(r) => *r,
            babylon_bsl::evaluator::Value::Int(i) => *i as f64,
            other => panic!("{name} must be a real or int literal, got {other:?}"),
        };
        assert_eq!(
            bits.to_bits(),
            value.to_bits(),
            "{name} diverges from the canonical value"
        );
    }
}

// ---- §7c's three permanent anti-pattern guards (landed Task 2, NEVER
// deleted). ----

/// §7c row 1: no `(binding … :field class-dynamics/…)` in the pack source —
/// `subject_type_of` would demand a `NodeType/CLASS_DYNAMICS` that does not
/// exist.
#[test]
fn no_rule_binds_a_field_in_the_pack_namespace() {
    let mut rest = PACK;
    while let Some(idx) = rest.find("(binding") {
        rest = &rest[idx..];
        let end = rest.find(')').expect("a binding form closes");
        let form = &rest[..=end];
        assert!(
            !(form.contains(":field") && form.contains("class-dynamics/")),
            "a :field binding owns off the class-dynamics namespace: {form}"
        );
        rest = &rest[1..];
    }
}

/// §7c row 2: the pack declares no intrinsic — protects the duplicate-`floor`
/// hazard (D-NF+22) and §6's no-transcendental verdict in one line.
#[test]
fn the_pack_declares_no_intrinsic() {
    assert!(
        !PACK.contains("(intrinsic "),
        "class-dynamics.bsl must declare no intrinsic (D-NF+22 / §6)"
    );
}

/// §7c row 3: the pack writes only `territory/*` fields and the one allowed
/// class field `social-class/ternary-net-fascist` — pins the §2.3 cross-train
/// disjointness argument mechanically.
#[test]
fn the_pack_writes_only_territory_fields_and_one_class_field() {
    let allowed: std::collections::HashSet<&str> = [
        "territory/rate-accumulation",
        "territory/rate-dispossession",
        "territory/rate-precaritization",
        "territory/rate-stabilization",
        "territory/raw-share-labor-aristocracy",
        "territory/raw-share-proletariat",
        "territory/raw-share-lumpenproletariat",
        "territory/share-labor-aristocracy",
        "territory/share-proletariat",
        "territory/share-lumpenproletariat",
        "territory/dist-year",
        "territory/baseline-la-share",
        "territory/baseline-la-known",
        "territory/bifurcation-score",
        "social-class/ternary-net-fascist",
    ]
    .into_iter()
    .collect();
    // Strip comment lines so verbatim spike-result examples are not parsed
    // as live update-node forms.
    let live_source: String = PACK
        .lines()
        .filter(|line| !line.trim_start().starts_with(';'))
        .collect::<Vec<_>>()
        .join("\n");
    let mut rest = live_source.as_str();
    while let Some(idx) = rest.find("(update-node") {
        rest = &rest[idx..];
        let end = rest.find(')').expect("an update-node form closes");
        let form = &rest[..=end];
        let target = form
            .split_whitespace()
            .nth(2)
            .expect("update-node has a target qname");
        assert!(
            allowed.contains(target),
            "update-node target {target} is not in the allowed set (§7c row 3)"
        );
        rest = &rest[1..];
    }
}
