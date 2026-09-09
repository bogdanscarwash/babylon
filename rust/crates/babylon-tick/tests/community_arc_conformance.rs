//! The decay-arc conformance suite (Community port train, Task 10 — plan
//! `docs/superpowers/plans/2026-08-18-community-port.md`): the three-tick
//! arc over world 5 (`content/scenarios/community-decay-arc-conformance.bscn`)
//! via `TickSession` (the `control_ratio_conformance.rs:1172-1181` idiom,
//! with `SessionId::new`'s deterministic-identity law, D179).
//!
//! # The arc mirror's verbatim stdout (2026-08-22, python3 3.13)
//!
//! `community_decay_arc_conformance.py` is the arc's oracle (standalone,
//! transcribing the SAME rules as the main mirror over three ticks — the
//! multi-tick half, where c00's and c09's resets stop being the identity).
//! Exact equality, no tolerance, per §9:
//!
//! ```text
//! community-decay-arc — mirror output (the three-tick oracle)
//! tick 0 (seed): heat = 0.5 cohesion = 0.75 edu = 0.25
//! tick 1: heat = 0.475 cohesion = 0.7275 edu = 0.225 | r = 1.0 l = 0.0 f = 0.0 | cost-modifier = 0.875 | member-count = 1.0 density-sum = 1.0
//! tick 2: heat = 0.45125 cohesion = 0.705675 edu = 0.2025 | r = 1.0 l = 0.0 f = 0.0 | cost-modifier = 0.875 | member-count = 1.0 density-sum = 1.0
//! tick 3: heat = 0.42868749999999994 cohesion = 0.68450475 edu = 0.18225000000000002 | r = 1.0 l = 0.0 f = 0.0 | cost-modifier = 0.875 | member-count = 1.0 density-sum = 1.0
//! ```

use babylon_bsl::scenario::{compose_declaration_preludes, load_scenario_with_prelude};
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, HyperedgeId, NodeId};
use babylon_kernel::SessionId;
use babylon_tick::TickSession;

const SCENARIO: &str = include_str!("../content/scenarios/community-decay-arc-conformance.bscn");
const PACK: &str = include_str!("../content/rules/community.bsl");
const ORGANIZATION_PRACTICE_PRELUDE: &str =
    include_str!("../content/declarations/organization-practice.bscn");

fn practice_prelude() -> String {
    compose_declaration_preludes(&[ORGANIZATION_PRACTICE_PRELUDE])
        .expect("the organization practice prelude composes")
}

/// The session id — `SessionId::new`'s deterministic-identity law (D179).
fn arc_session() -> SessionId {
    SessionId::new("community-decay-arc").expect("literal is non-empty")
}

/// Drive three ticks, returning the session (the graph lives in it).
fn run_arc() -> TickSession<HypergraphStore> {
    let mut session = TickSession::new_with_prelude(
        SCENARIO,
        &practice_prelude(),
        PACK,
        HypergraphStore::new(),
        arc_session(),
    )
    .expect("world 5 + the pack loads into a session");
    let mut sink = CollectingSink::default();
    session.advance(&mut sink).expect("tick 1");
    session.advance(&mut sink).expect("tick 2");
    session.advance(&mut sink).expect("tick 3");
    session
}

fn heat(session: &TickSession<HypergraphStore>) -> f64 {
    session
        .graph()
        .hyperedge_attribute(HyperedgeId(0), "community/heat")
        .expect("heat written")
}

fn at(session: &TickSession<HypergraphStore>, field: &str) -> f64 {
    session
        .graph()
        .hyperedge_attribute(HyperedgeId(0), field)
        .unwrap_or_else(|e| panic!("{field}: {}", e.message))
}

/// Step 1: the three decay arms are INDEPENDENT (§8a row 2) — tick 3 reads
/// three DISTINCT values, each its own α compounded three times
/// (0.5·0.95³, 0.75·0.97³, 0.25·0.9³), bit-exact against the arc mirror.
#[test]
fn heat_cohesion_education_decay_independently() {
    let session = run_arc();
    assert_eq!(
        at(&session, "community/heat").to_bits(),
        (0.42868749999999994_f64).to_bits()
    );
    assert_eq!(
        at(&session, "community/cohesion").to_bits(),
        (0.68450475_f64).to_bits()
    );
    assert_eq!(
        at(&session, "community/education-pressure").to_bits(),
        (0.18225000000000002_f64).to_bits()
    );
}

/// Step 1: frozen law L4's ported half (`test_law_community_system.py:221`
/// — heat/cohesion never increase per tick): per-tick reads are monotone
/// non-increasing across the arc.
#[test]
fn decay_is_monotone_non_increasing() {
    let mut session = TickSession::new_with_prelude(
        SCENARIO,
        &practice_prelude(),
        PACK,
        HypergraphStore::new(),
        arc_session(),
    )
    .expect("session loads");
    let mut sink = CollectingSink::default();
    let mut prev = f64::INFINITY;
    for _tick in 1..=3 {
        session.advance(&mut sink).expect("tick");
        let current = heat(&session);
        assert!(
            current <= prev,
            "heat must not increase: {prev} -> {current}"
        );
        prev = current;
    }
}

/// Step 1 (the Task-8-deferred vector's proof home): c09's reset makes the
/// cost modifier PER-TICK — tick 3 reads 0.875 again, never 0.875³
/// (0.669…). Deleting c09 reds this at tick 2 (0.765625).
#[test]
fn cost_modifier_does_not_compound_across_ticks() {
    let session = run_arc();
    let value = session
        .graph()
        .node_attribute(NodeId(1), "social-class/community-cost-modifier")
        .expect("the active class is written every tick");
    assert_eq!(
        value.to_bits(),
        (0.875_f64).to_bits(),
        "the reset + the scale, per tick — never compounding"
    );
}

/// The ternary recomputes identically from the static org landscape every
/// tick: (1.0, 0.0, 0.0), r = 1.0 clearing the 0.136 floor.
#[test]
fn the_ternary_is_idempotent_across_the_arc() {
    let session = run_arc();
    assert_eq!(
        at(&session, "community/revolutionary").to_bits(),
        (1.0_f64).to_bits()
    );
    assert_eq!(
        at(&session, "community/liberal").to_bits(),
        (0.0_f64).to_bits()
    );
    assert_eq!(
        at(&session, "community/fascist").to_bits(),
        (0.0_f64).to_bits()
    );
}

// ---- World 5b: the solidarity seam (§2.1) ----

/// World 5b: the landed solidarity world PLUS the community half's
/// additions (carrier, load-law org, one community).
const SEAM_SCENARIO: &str =
    include_str!("../content/scenarios/community-solidarity-seam-conformance.bscn");
const SOLIDARITY_PACK: &str = include_str!("../content/rules/solidarity.bsl");

/// Step 1, first half (§2.1): co-loading the community pack moves NO
/// SOLIDARITY edge's strength — byte-identical against the
/// solidarity-only run of the SAME world. (The amplification half that
/// WOULD move them is #653-blocked and not in this pack.)
#[test]
fn community_tick_leaves_solidarity_strength_byte_identical() {
    let mut solo = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    babylon_tick::run_once_into_with_prelude(
        SEAM_SCENARIO,
        &practice_prelude(),
        SOLIDARITY_PACK,
        &mut solo,
        &mut sink,
    )
    .expect("the solidarity-only run ticks");

    let co_loaded = format!("{SOLIDARITY_PACK}\n{PACK}");
    let mut both = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    babylon_tick::run_once_into_with_prelude(
        SEAM_SCENARIO,
        &practice_prelude(),
        &co_loaded,
        &mut both,
        &mut sink,
    )
    .expect("the co-loaded run ticks");

    for (from, to) in solo.edges("SOLIDARITY") {
        let solo_strength = solo
            .edge_attribute("SOLIDARITY", from, to, "solidarity/strength")
            .expect("the strength slot");
        let both_strength = both
            .edge_attribute("SOLIDARITY", from, to, "solidarity/strength")
            .expect("the strength slot");
        assert_eq!(
            solo_strength.to_bits(),
            both_strength.to_bits(),
            "SOLIDARITY {from:?}->{to:?} moved under the co-load"
        );
    }
    // and the worlds' edge censuses agree (the co-load mints no edges).
    assert_eq!(
        solo.edges("SOLIDARITY").len(),
        both.edges("SOLIDARITY").len()
    );
}

/// Step 1, second half: the seam proof cannot pass VACUOUSLY — the
/// co-loaded run's community half actually RAN (the seam community's heat
/// decayed 0.5 -> 0.475 through c11) AND its carrier rules fired. The
/// mutation vector (remove world 5b's carrier) reds this AND §8c guard 4.
#[test]
fn seam_world_community_half_actually_ran() {
    let co_loaded = format!("{SOLIDARITY_PACK}\n{PACK}");
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = babylon_tick::run_once_into_with_prelude(
        SEAM_SCENARIO,
        &practice_prelude(),
        &co_loaded,
        &mut graph,
        &mut sink,
    )
    .expect("the co-loaded run ticks");
    assert!(
        report.fired > 0,
        "the carrier rules fired (the fired arithmetic is the per-rule-id ledger)"
    );
    let heat = graph
        .hyperedge_attribute(HyperedgeId(0), "community/heat")
        .expect("the seam community's heat");
    assert_eq!(
        heat.to_bits(),
        (0.475_f64).to_bits(),
        "0.5 x (1 - 0.05) — the community half RAN (c11's decay moved it)"
    );
}

// ---- World 5c: the carrier collision (§3.7a) ----

/// World 5c: the landed carceral world with its ONE institution node
/// carrying BOTH packs' anchor fields.
const COLLISION_SCENARIO: &str =
    include_str!("../content/scenarios/community-carrier-collision-conformance.bscn");
const CONTROL_RATIO_PACK: &str = include_str!("../content/rules/control-ratio.bsl");

/// Step 1 (§3.7a): the two packs co-loaded over ONE carrier fire once
/// each — the per-rule-id fired arithmetic (the carceral_arc_conformance
/// style) and the decay-applied-ONCE read (0.475, never the
/// double-application 0.45125). The mutation vector (a SECOND institution
/// node in this world) reds §8c guard 4 first, then this arithmetic.
#[test]
fn co_loaded_packs_fire_once_each_on_one_carrier() {
    let co_loaded = format!("{CONTROL_RATIO_PACK}\n{PACK}");
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = babylon_tick::run_once_into_with_prelude(
        COLLISION_SCENARIO,
        &practice_prelude(),
        &co_loaded,
        &mut graph,
        &mut sink,
    )
    .expect("the co-loaded world ticks");
    let heat = graph
        .hyperedge_attribute(HyperedgeId(0), "community/heat")
        .expect("the collision community's heat");
    assert_eq!(
        heat.to_bits(),
        (0.475_f64).to_bits(),
        "c11's decay applied ONCE (0.5 x 0.95) — a second carrier would \
         double-apply it (0.45125)"
    );
    // The fired arithmetic (breakdown in the failure message).
    assert_eq!(
        report.fired, 39,
        "control-ratio: c01:6 (every SOCIAL_CLASS subject) + c02:1 + c03:1 \
         + c04:1 = 9; community: c00:1 + c01:4 (the four ACTIVE carceral \
         classes — enforcer-inactive and prisoner-inactive are excluded) + \
         c02:6 (all six) + c03f:0 + c03l:1 (collision-org is LIBERAL) + \
         c03r:0 + c04:4 + c05:1 + c06a:1 + c06b:1 + c09:4 + c10:4 + c11:1 \
         + c07:1 + c08:1 (the DG-2 readout) = 30 (per-rule-id, measured \
         via per_rule_fired) — each pack's carrier rules fire ONCE on the \
         shared carrier"
    );
}

// ---- World 6 + the L1 analogue (§1.2) ----

/// World 6: one carrier, one all-inactive community.
const EMPTY_SCENARIO: &str = include_str!("../content/scenarios/community-empty-conformance.bscn");

/// Step 1 (§1.2's L2 analogue, per the plan's OWN repinning in D205): with
/// every member inactive, every MEMBERSHIP lane is skipped — the census
/// reads 0, the seeded simplex is preserved byte-exactly (the c05/c06
/// skip gate), and no cost modifier is ever written (the honest-null
/// read) — while the carrier rules DO fire and c11's ungated decay still
/// moves heat (0.5 -> 0.475). Frozen's global early return
/// (community.py:337-338) suppresses the decay; the port's per-rule
/// structure cannot express a cross-rule early return, so the decay
/// applying is the recorded divergence — NOT the plan's "byte-exact
/// no-op" phrasing, which its own ungated c11 row contradicts.
#[test]
fn all_inactive_members_skip_every_membership_lane() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = babylon_tick::run_once_into_with_prelude(
        EMPTY_SCENARIO,
        &practice_prelude(),
        PACK,
        &mut graph,
        &mut sink,
    )
    .expect("every rule loads and the world ticks");
    // The carrier rules fire; the class rules fire on nobody active.
    let per_rule: std::collections::HashMap<&str, usize> = report
        .per_rule_fired
        .iter()
        .map(|(id, n)| (id.as_str(), *n))
        .collect();
    assert_eq!(per_rule["community/c00-census-reset"], 1);
    assert_eq!(
        per_rule["community/c11-state-decay"], 1,
        "the carrier decay rule fires"
    );
    assert_eq!(
        per_rule["community/c01-member-census"], 0,
        "no active class"
    );
    assert_eq!(
        per_rule["community/c09-cost-modifier-reset"], 0,
        "the active guard"
    );
    assert_eq!(
        per_rule["community/c10-cost-modifier-accumulate"], 0,
        "the active guard"
    );
    // The census is zero and the simplex is preserved byte-exactly.
    assert_eq!(
        graph
            .hyperedge_attribute(HyperedgeId(0), "community/member-count")
            .expect("c00's reset landed")
            .to_bits(),
        (0.0_f64).to_bits()
    );
    assert_eq!(
        graph
            .hyperedge_attribute(HyperedgeId(0), "community/revolutionary")
            .expect("the seeded ternary")
            .to_bits(),
        (0.5_f64).to_bits(),
        "the skip gate preserves the seed"
    );
    // No cost modifier is ever written on either inactive class.
    for nid in [1_u64, 2] {
        assert!(
            graph
                .node_attribute(NodeId(nid), "social-class/community-cost-modifier")
                .is_err(),
            "n{nid}: honest null, never 1.0"
        );
    }
    // The recorded divergence: c11's ungated decay DOES apply.
    assert_eq!(
        graph
            .hyperedge_attribute(HyperedgeId(0), "community/heat")
            .expect("heat")
            .to_bits(),
        (0.475_f64).to_bits(),
        "the decay applies — the port has no cross-rule early return (D205)"
    );
}

/// Step 1 (§1.2's L1 analogue): a world with NO COMMUNITY hyperedge
/// REFUSES the pack at load — MissingCeiling (E-LOAD-045), the driver's
/// documented "fail loudly at load rather than quietly iterate nothing"
/// (lib.rs:250-252) — where frozen silently no-ops. A refusal test, never
/// a golden-pinned world.
#[test]
fn a_world_with_no_community_hyperedge_refuses_at_load() {
    // One carrier, one active class, one org, one MEMBERSHIP edge — every
    // load law satisfied EXCEPT the community ceiling.
    let scenario = r"
(scenario community/no-community-refusal
  (defvocabulary NodeType (INSTITUTION SOCIAL_CLASS ORGANIZATION))
  (defvocabulary EdgeType (MEMBERSHIP))
  (defvocabulary HyperedgeType (COMMUNITY))
  (deffield institution/community-carrier int extensive)
  (deffield social-class/active int extensive)
  (deffield organization/cadre-level probability intensive)
  (deffield organization/cohesion probability intensive)
  (defenum ConsciousnessTendency (LIBERAL FASCIST REVOLUTIONARY))
  (deffield organization/consciousness-tendency enum ConsciousnessTendency)
  (deffield social-class/community-cost-modifier real intensive)
  (deffield social-class/org-r-weight real extensive)
  (deffield social-class/org-l-weight real extensive)
  (deffield social-class/org-f-weight real extensive)
  (deffield social-class/org-count int extensive)
  (node lone-register NodeType/INSTITUTION (institution/community-carrier 1))
  (node lone-class NodeType/SOCIAL_CLASS (social-class/active 1))
  (node lone-org NodeType/ORGANIZATION
    (organization/cadre-level 0.5p) (organization/cohesion 0.5p)
    (organization/consciousness-tendency ConsciousnessTendency/LIBERAL))
  (edge EdgeType/MEMBERSHIP lone-org lone-class 1))
";
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let err = babylon_tick::run_once_into(scenario, PACK, &mut graph, &mut sink)
        .expect_err("a community-free world refuses the pack at load");
    assert!(
        err.contains("E-LOAD-045"),
        "MissingCeiling, named by code: {err}"
    );
}

// ---- §8c guards 1 and 4 over the Task-10 worlds (the same law, the arc
// file's own instances — the guards are never deleted and they cover
// every world this pack loads into) ----

/// §8c row 1 over the arc/seam/collision/empty worlds: no node whose type
/// name contains `COMMUNITY`.
#[test]
fn no_community_typed_node_exists_in_the_arc_worlds() {
    for scenario in [SCENARIO, SEAM_SCENARIO, COLLISION_SCENARIO, EMPTY_SCENARIO] {
        let mut graph = HypergraphStore::new();
        let loaded = load_scenario_with_prelude(&practice_prelude(), scenario, &mut graph)
            .expect("world loads");
        for (member, count) in &loaded.node_types {
            assert!(
                !member.contains("COMMUNITY"),
                "node type {member} (x{count}) — communities are hyperedges, never nodes"
            );
        }
    }
}

/// §8c row 4 over the arc/seam/collision/empty worlds: exactly ONE
/// INSTITUTION carrier each. The 5c collision world is the row's
/// load-bearing case — its one carrier carries BOTH packs' anchors.
#[test]
fn exactly_one_institution_carrier_in_the_arc_worlds() {
    for scenario in [SCENARIO, SEAM_SCENARIO, COLLISION_SCENARIO, EMPTY_SCENARIO] {
        let mut graph = HypergraphStore::new();
        let loaded = load_scenario_with_prelude(&practice_prelude(), scenario, &mut graph)
            .expect("world loads");
        assert_eq!(
            loaded.node_types.get("INSTITUTION"),
            Some(&1),
            "exactly one INSTITUTION carrier — two double-applies every \
             carrier rule's hyperedge writes; zero is silent inertness"
        );
    }
}
