//! The Community @6.0 port train's conformance suite (issue #667, plan
//! `docs/superpowers/plans/2026-08-18-community-port.md`, Task 7) — world 1
//! (`content/scenarios/community-conformance.bscn`), the frozen mirror
//! (`content/scenarios/community_conformance.py`), THE SPIKE's seven-shape
//! verdicts, and §8c's four permanent anti-pattern guards (landed here,
//! never deleted).
//!
//! # Task 7 deviations from the plan's letter (both recorded, both forced)
//!
//! 1. **Step 1's "expected FAIL (unregistered system)" never red-fired**:
//!    the `community` namespace was registered in `lib.rs`'s systems
//!    `HashSet` at **Task 4** (#681), one task earlier than the plan's
//!    Step 2 — the ceiling supply chain's RED tests needed the two refusal
//!    codes specifically, and an unregistered namespace fails E-LOAD-004
//!    one stage earlier (the registration comment in `lib.rs` says all of
//!    this). So this suite's load-smoke was green from birth; Step 2's
//!    "confirming the entry is genuinely new" discharges by READING that
//!    comment, not by re-adding the entry.
//! 2. **The "empty rule source" deviation, inherited**: `run_once_into`'s
//!    `split_content` refuses a content set with zero `(rule …)` top-forms
//!    outright (`decomposition_conformance.rs`'s Step-1 note and
//!    `production_conformance.rs:75-94` record the same), so the smoke
//!    test drives ONE minimal, never-firing probe rule.
//!
//! # The mirror's verbatim stdout (2026-08-22, python3 3.13)
//!
//! `community_conformance.py` is THE ORACLE (§9: standalone, no `babylon`
//! import, transcribing the planned rules c00→c11 over the literal WORLD).
//! Tasks 8-11 pin bit-exact against these values via the `.to_bits()`
//! idiom — **exact equality, no tolerance**, because every value here is
//! the same IEEE-754 operation chain the rules transcribe (the mirror's
//! header states the operation order per accumulator; shortest-repr
//! decimals round-trip through Rust's correctly-rounded parser to the same
//! binary64):
//!
//! ```text
//! community-conformance — mirror output (the oracle)
//! == world 1 (community-conformance.bscn) ==
//! h0 new-afrikan kind=NEW_AFRIKAN
//!   member-count = 2.0
//!   r-raw = 0.4
//!   l-raw = 0.0625
//!   f-raw = 0.0
//!   density-sum  = 1.5
//!   substrate-floor = 0.136
//!   r = 0.8648648648648649
//!   l = 0.13513513513513511
//!   f = 0.0
//!   contestation = 0.3604848532176559
//!   dominant     = REVOLUTIONARY
//!   decayed heat               = 0.475
//!   decayed cohesion           = 0.7275
//!   decayed education-pressure = 0.225
//! h1 settler kind=SETTLER
//!   member-count = 1.0
//!   r-raw = 0.0
//!   l-raw = 0.125
//!   f-raw = 0.125
//!   density-sum  = 2.0
//!   substrate-floor = 0.0
//!   r = 0.0
//!   l = 0.5
//!   f = 0.5
//!   contestation = 0.6309297535714575
//!   dominant     = LIBERAL
//!   decayed heat               = 0.2375
//!   decayed cohesion           = 0.485
//!   decayed education-pressure = 0.1125
//! h2 queer kind=QUEER
//!   member-count = 1.0
//!   r-raw = 0.4
//!   l-raw = 0.125
//!   f-raw = 0.0
//!   density-sum  = 2.0
//!   substrate-floor = 0.04
//!   r = 0.7619047619047619
//!   l = 0.23809523809523808
//!   f = 0.0
//!   contestation = 0.49960699524890273
//!   dominant     = REVOLUTIONARY
//!   decayed heat               = 0.7124999999999999
//!   decayed cohesion           = 0.60625
//!   decayed education-pressure = 0.45
//! n1 na-worker: org-r-weight = 0.4, org-l-weight = 0.125, org-f-weight = 0.0, org-count = 2.0
//! n1 na-worker: community-cost-modifier = 1.09375
//! n2 na-organizer: org-r-weight = 0.4, org-l-weight = 0.0, org-f-weight = 0.0, org-count = 1.0
//! n2 na-organizer: community-cost-modifier = 0.875
//! n3 settler-la: org-r-weight = 0.0, org-l-weight = 0.125, org-f-weight = 0.125, org-count = 2.0
//! n3 settler-la: community-cost-modifier = 1.0
//! n4 unaffiliated: org-r-weight = 0.0, org-l-weight = 0.0, org-f-weight = 0.0, org-count = 0.0
//! n4 unaffiliated: community-cost-modifier = 1.0
//! n5 inactive-member: org-r-weight = 0.0, org-l-weight = 0.0, org-f-weight = 0.0, org-count = 0.0
//! n5 inactive-member: community-cost-modifier = ABSENT (honest null)
//! == world 2 (community-floor-conformance.bscn) ==
//! h0 na-comm kind=NEW_AFRIKAN
//!   member-count = 2.0
//!   r-raw = 0.03125
//!   l-raw = 1.0
//!   f-raw = 0.0625
//!   density-sum  = 2.5
//!   substrate-floor = 0.136
//!   r = 0.136
//!   l = 0.8131764705882352
//!   f = 0.0508235294117647
//!   contestation = 0.53788562725128
//!   dominant     = LIBERAL
//!   decayed heat               = 0.475
//!   decayed cohesion           = 0.7275
//!   decayed education-pressure = 0.225
//! h1 fn-comm kind=FIRST_NATIONS
//!   member-count = 1.0
//!   r-raw = 0.03125
//!   l-raw = 1.0
//!   f-raw = 0.0
//!   density-sum  = 2.0
//!   substrate-floor = 0.155
//!   r = 0.155
//!   l = 0.845
//!   f = 0.0
//!   contestation = 0.3925724663663698
//!   dominant     = LIBERAL
//!   decayed heat               = 0.2375
//!   decayed cohesion           = 0.485
//!   decayed education-pressure = 0.1125
//! h2 settler-comm kind=SETTLER
//!   member-count = 1.0
//!   r-raw = 0.0
//!   l-raw = 1.0
//!   f-raw = 0.0
//!   density-sum  = 1.0
//!   substrate-floor = 0.0
//!   r = 0.0
//!   l = 1.0
//!   f = 0.0
//!   contestation = 0.0
//!   dominant     = LIBERAL
//!   decayed heat               = 0.7124999999999999
//!   decayed cohesion           = 0.60625
//!   decayed education-pressure = 0.45
//! h3 low-density-comm kind=SETTLER
//!   member-count = 2.0
//!   r-raw = 0.015625
//!   l-raw = 0.0
//!   f-raw = 0.0
//!   density-sum  = 0.5
//!   substrate-floor = 0.0
//!   r = 0.030303030303030304
//!   l = 0.9696969696969697
//!   f = 0.0
//!   contestation = 0.12360498799464747
//!   dominant     = LIBERAL
//!   decayed heat               = 0.475
//!   decayed cohesion           = 0.2425
//!   decayed education-pressure = 0.3375
//! n1 w2-a: org-r-weight = 0.03125, org-l-weight = 1.0, org-f-weight = 0.0, org-count = 2.0
//! n1 w2-a: community-cost-modifier = 1.09375
//! n2 w2-b: org-r-weight = 0.03125, org-l-weight = 1.0, org-f-weight = 0.125, org-count = 3.0
//! n2 w2-b: community-cost-modifier = 0.875
//! n3 w2-s: org-r-weight = 0.0, org-l-weight = 1.0, org-f-weight = 0.0, org-count = 1.0
//! n3 w2-s: community-cost-modifier = 1.0
//! n4 w2-c: org-r-weight = 0.0, org-l-weight = 0.0, org-f-weight = 0.0, org-count = 0.0
//! n4 w2-c: community-cost-modifier = 1.0
//! n5 w2-d: org-r-weight = 0.03125, org-l-weight = 0.0, org-f-weight = 0.0, org-count = 1.0
//! n5 w2-d: community-cost-modifier = 1.0
//! == world 3 (community-degenerate-conformance.bscn) ==
//! h0 deg-comm kind=CHICANO
//!   member-count = 1.0
//!   r-raw = 0.0
//!   l-raw = 0.0
//!   f-raw = 0.0
//!   density-sum  = 1.0
//!   substrate-floor = 0.113
//!   r = 0.113
//!   l = 0.887
//!   f = 0.0
//!   contestation = 0.32107956537304744
//!   dominant     = LIBERAL
//!   decayed heat               = 0.475
//!   decayed cohesion           = 0.7275
//!   decayed education-pressure = 0.225
//! h1 no-org-comm kind=QUEER
//!   member-count = 1.0
//!   r-raw = 0.0
//!   l-raw = 0.0
//!   f-raw = 0.0
//!   density-sum  = 0.0
//!   substrate-floor = ABSENT (no-org skip gate)
//!   r = 0.125
//!   l = 0.75
//!   f = 0.125
//!   contestation = PRESERVED (no-org skip gate)
//!   dominant     = PRESERVED (no-org skip gate)
//!   decayed heat               = 0.2375
//!   decayed cohesion           = 0.485
//!   decayed education-pressure = 0.1125
//! n1 d-a: org-r-weight = 0.0, org-l-weight = 0.0, org-f-weight = 0.0, org-count = 1.0
//! n1 d-a: community-cost-modifier = 0.75
//! n2 d-b: org-r-weight = 0.0, org-l-weight = 0.0, org-f-weight = 0.0, org-count = 0.0
//! n2 d-b: community-cost-modifier = 1.125
//! == world 4 (community-cost-modifier-conformance.bscn) ==
//! h0 low-cost kind=QUEER
//!   member-count = 2.0
//!   r-raw = 0.0
//!   l-raw = 0.125
//!   f-raw = 0.0
//!   density-sum  = 0.5
//!   substrate-floor = 0.04
//!   r = 0.04
//!   l = 0.96
//!   f = 0.0
//!   contestation = 0.15286935115005695
//!   dominant     = LIBERAL
//!   decayed heat               = 0.475
//!   decayed cohesion           = 0.7275
//!   decayed education-pressure = 0.225
//! h1 high-cost kind=NEW_AFRIKAN
//!   member-count = 1.0
//!   r-raw = 0.0
//!   l-raw = 0.0
//!   f-raw = 0.0
//!   density-sum  = 0.0
//!   substrate-floor = ABSENT (no-org skip gate)
//!   r = 0.5
//!   l = 0.25
//!   f = 0.25
//!   contestation = PRESERVED (no-org skip gate)
//!   dominant     = PRESERVED (no-org skip gate)
//!   decayed heat               = 0.7124999999999999
//!   decayed cohesion           = 0.60625
//!   decayed education-pressure = 0.45
//! n1 w4-both: org-r-weight = 0.0, org-l-weight = 0.0, org-f-weight = 0.0, org-count = 0.0
//! n1 w4-both: community-cost-modifier = 0.9375
//! n2 w4-one: org-r-weight = 0.0, org-l-weight = 0.25, org-f-weight = 0.0, org-count = 1.0
//! n2 w4-one: community-cost-modifier = 0.75
//! n3 w4-none: org-r-weight = 0.0, org-l-weight = 0.0, org-f-weight = 0.0, org-count = 0.0
//! n3 w4-none: community-cost-modifier = 1.0
//! n4 w4-inactive: org-r-weight = 0.0, org-l-weight = 0.0, org-f-weight = 0.0, org-count = 0.0
//! n4 w4-inactive: community-cost-modifier = ABSENT (honest null)
//! ```
//!
//! # THE SPIKE (Task 7 Step 5) — the seven-shape verdicts
//!
//! Each shape below was proven by a THROWAWAY rule against the real
//! `run_once_into` driver and THIS scenario, then deleted (the
//! solidarity-conformance.bscn:9-20 precedent — the scenario header
//! carries the same record). Verdicts:
//!
//! - **(a)** `for-each` over `(hyperedges HyperedgeType/COMMUNITY)` from a
//!   carrier-subject rule — **FIRES** (3 communities × 1 carrier = 3
//!   body-evaluations, verified by a write landing on all three).
//! - **(b)** `(update-hyperedge it community/heat (set …))` inside that
//!   body — **WRITES** (Task 6's own surface; read back bit-exact).
//! - **(c)** `(for-each (hyperedges-of self HyperedgeType/COMMUNITY) …)`
//!   from a SOCIAL_CLASS-subject rule + `(field-of it …)` read inside —
//!   **FIRES and READS** (na-worker sees exactly its two memberships, h0
//!   then h2, and the reads returned the seeded values).
//! - **(d)** repeated `(scale …)` accumulates multiplicatively across a
//!   `for-each` — **PROVEN**: two scales of 0.5c over na-worker's two
//!   memberships landed 1 → 0.25 on the probe field (the operand is
//!   pre-computed per collected write; the COMBINE reads the current value
//!   at APPLY, so the two scales compose as a product — §9 item 2's
//!   pre-state law governs operand evaluation, never the combine).
//! - **(e)** a 14-arm `if`-chain over the enum-field equality (`community/
//!   kind` against each CommunityType member) — **LOADS and EVALUATES**:
//!   each hyperedge's dispatch returned its own floor constant
//!   (0.136/0.0/0.04), the shape §6.2's c06 dispatch needs. The only
//!   landed precedent was 3-arm (`territory.bsl:130-137`); the 14-arm is
//!   now demonstrated, not merely plausible.
//! - **(f)** a same-tick cross-rule read of a hyperedge field written by
//!   an earlier rule — **SEES the new value** (§8b's fatal rows rely on
//!   exactly this; rule-id byte order `community/spike-a` <
//!   `community/spike-b` carried the write → read).
//! - **(g)** the world LOADS with a measured `E-LOAD-040`-bounded
//!   hyperedge-querying rule — Task 4's ceiling supply chain reaches this
//!   scenario: the probe's static bound passed with the census-fed
//!   `HyperedgeType/COMMUNITY` ceiling 3 and max-members 3.
//!
//! No shape refused; §8's rule split stands as planned.
//!
//! # The mirror pin (Task 8+)
//!
//! Tasks 8-11 extend this file with per-rule assertions against the
//! mirror transcript above; Task 7 lands the world, the load-smoke, the
//! ordinal parity, and §8c's four guards only.

use babylon_bsl::scenario::{
    compose_declaration_preludes, load_scenario_with_prelude, LoadedScenario,
};
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, HyperedgeId, NodeId};
use babylon_tick::run_once_into_with_prelude;

/// The world-1 scenario source, single-homed here — Tasks 8-11's rule
/// packs load against exactly this text.
const SCENARIO: &str = include_str!("../content/scenarios/community-conformance.bscn");
const ORGANIZATION_PRACTICE_PRELUDE: &str =
    include_str!("../content/declarations/organization-practice.bscn");

fn practice_prelude() -> String {
    compose_declaration_preludes(&[ORGANIZATION_PRACTICE_PRELUDE])
        .expect("the organization practice prelude composes")
}

fn load_practice_scenario(scenario: &str, graph: &mut HypergraphStore) -> LoadedScenario {
    load_scenario_with_prelude(&practice_prelude(), scenario, graph)
        .expect("the promoted scenario loads")
}

/// The pack source this suite drives — the c00-c04 census/decomposition
/// rules (Task 8); §8c guard 2 reads it, which is what gives the guard
/// teeth (it was vacuous against the Task-7 empty string by design).
const PACK: &str = include_str!("../content/rules/community.bsl");

/// Step 1/7's load-smoke: the world hydrates and ticks clean under one
/// minimal, never-firing probe rule (`active` is 0/1, never 2). The
/// pre/post hashes must be IDENTICAL — a silent probe writes nothing, and
/// the hash brackets the tick, not the hydration.
#[test]
fn scenario_and_empty_pack_load() {
    let probe = r#"
(rule community/load-smoke
  :role mechanic :evidence derived :material-basis "Task 7 Step 1: the load-smoke probe — never fires, proves the world + registration"
  :fuel 64
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active))
  (when (= active 2))
  (effects
    (emit EventType/ORGANIZATION_SEEDED (probe 1))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report =
        run_once_into_with_prelude(SCENARIO, &practice_prelude(), probe, &mut graph, &mut sink)
            .expect("world 1 loads and ticks clean");
    assert_eq!(report.fired, 0, "the probe never fires");
    assert_eq!(
        report.before, report.after,
        "a never-firing probe writes nothing — the hash brackets the tick"
    );
}

/// ADR195 ordinal parity: both enum registries in FROZEN DECLARATION
/// ORDER — CommunityType from `models/enums/community.py:38-55`,
/// ConsciousnessTendency from `models/enums/consciousness.py:82-85`. A
/// transposition here silently re-reads every seeded `community/kind`.
#[test]
fn defenum_ordinal_parity_with_the_frozen_order() {
    let mut graph = HypergraphStore::new();
    let loaded = load_practice_scenario(SCENARIO, &mut graph);
    let community = loaded
        .enums
        .resolve("CommunityType")
        .expect("CommunityType declared");
    let frozen_order = [
        "SETTLER",
        "PATRIARCHAL",
        "NEW_AFRIKAN",
        "FIRST_NATIONS",
        "CHICANO",
        "WOMEN",
        "TRANS",
        "DISABLED",
        "QUEER",
        "UNDOCUMENTED",
        "INCARCERATED",
        "YOUTH",
        "ADULT",
        "ELDER",
    ];
    for (expected, member) in frozen_order.iter().enumerate() {
        assert_eq!(
            loaded.enums.ordinal(community, member),
            Some(expected as u32),
            "CommunityType/{member} must be ordinal {expected} (frozen declaration order)"
        );
    }
    let tendency = loaded
        .enums
        .resolve("ConsciousnessTendency")
        .expect("ConsciousnessTendency declared");
    for (expected, member) in ["LIBERAL", "FASCIST", "REVOLUTIONARY"].iter().enumerate() {
        assert_eq!(
            loaded.enums.ordinal(tendency, member),
            Some(expected as u32),
            "ConsciousnessTendency/{member} must be ordinal {expected} (frozen declaration order)"
        );
    }
    // …and the seeded kinds read back as those ordinals through the
    // substrate: NEW_AFRIKAN=2, SETTLER=0, QUEER=8.
    for (hid, member, ordinal) in [
        (HyperedgeId(0), "NEW_AFRIKAN", 2.0_f64),
        (HyperedgeId(1), "SETTLER", 0.0_f64),
        (HyperedgeId(2), "QUEER", 8.0_f64),
    ] {
        let stored = graph
            .hyperedge_attribute(hid, "community/kind")
            .expect("kind seeded");
        assert_eq!(
            stored.to_bits(),
            (ordinal).to_bits(),
            "h{}'s kind must store {member}'s ordinal",
            hid.0
        );
    }
}

// ---- §8c's four permanent anti-pattern guards (landed Task 7, NEVER
// deleted — the INV-010 estate's Rust half) ----

/// §8c row 1: communities appear ONLY as hyperedges — no node whose type
/// name contains `COMMUNITY` may exist in the census. Every world this
/// pack loads into (plan §8c: the guards outlive the frozen linter).
#[test]
fn no_community_typed_node_exists() {
    for scenario in [SCENARIO, SCENARIO_W2, SCENARIO_W3, SCENARIO_W4] {
        let mut graph = HypergraphStore::new();
        let loaded = load_practice_scenario(scenario, &mut graph);
        for (member, count) in &loaded.node_types {
            assert!(
                !member.contains("COMMUNITY"),
                "node type {member} (×{count}) — communities are hyperedges, never nodes (§8c row 1)"
            );
        }
    }
}

/// §8c row 2: no `(binding … :field community/…)` in the pack source —
/// a `:field` binding is node-scoped and stays node-scoped (D29; the
/// mechanism is `tick.rs::subject_type_of`'s owner-kind filter, D202, whose
/// error this guard never wants to see fire on real content). Reads the
/// pack string THIS suite drives — vacuous at Task 7 (PACK is empty),
/// toothed from Task 8 on.
#[test]
fn no_field_binding_uses_the_community_namespace() {
    let mut rest = PACK;
    while let Some(idx) = rest.find("(binding") {
        rest = &rest[idx..];
        let end = rest.find(')').expect("a binding form closes");
        let form = &rest[..=end];
        assert!(
            !(form.contains(":field") && form.contains("community/")),
            "a :field binding owns off the community namespace: {form} — \
             a hyperedge's field reads through field-of / writes through \
             update-hyperedge, never a binding (D29, D202)"
        );
        rest = &rest[1..];
    }
}

/// §8c row 3 (VIII.9 verbatim): a seeded community records ONE hyperedge
/// with N members, never C(n,2) dyadic edges — the world's total edge
/// census is exactly the five org→class MEMBERSHIP edges, and h0 (three
/// members) minted no dyadic clique.
#[test]
fn membership_crosses_whole() {
    let mut graph = HypergraphStore::new();
    let loaded = load_practice_scenario(SCENARIO, &mut graph);
    assert_eq!(loaded.edge_count, 5, "the five MEMBERSHIP edges, no more");
    assert_eq!(
        loaded.edge_types.get("MEMBERSHIP"),
        Some(&5),
        "every edge is an org→class MEMBERSHIP edge"
    );
    assert_eq!(
        loaded.hyperedge_types.get("COMMUNITY"),
        Some(&3),
        "three communities, as hyperedges"
    );
    // h0's three members are ONE hyperedge — if a clique expansion existed,
    // three more dyadic edges would have appeared above.
    let members = graph
        .hyperedges_of(NodeId(1), "COMMUNITY")
        .expect("na-worker exists");
    assert_eq!(
        members.len(),
        2,
        "na-worker crosses two communities whole (h0, h2)"
    );
}

/// §8c row 4 (§3.7a / C3): every world this pack loads into contains
/// EXACTLY ONE NodeType/INSTITUTION node — never two (each carrier-subject
/// rule would fire twice, double-applying every hyperedge write) and never
/// zero (six rules would iterate an empty population — silent inertness
/// dressed as a passing test).
#[test]
fn exactly_one_institution_carrier() {
    // Every world this pack loads into (§8c row 4's "never zero either" is
    // the loaded half — a carrier-free world runs the carrier rules over
    // an empty population, silent inertness dressed as a passing test).
    for scenario in [SCENARIO, SCENARIO_W2, SCENARIO_W3, SCENARIO_W4] {
        let mut graph = HypergraphStore::new();
        let loaded = load_practice_scenario(scenario, &mut graph);
        assert_eq!(
            loaded.node_types.get("INSTITUTION"),
            Some(&1),
            "exactly one INSTITUTION carrier per world — tick.rs's subject \
             derivation iterates EVERY node of the type (D202's filter \
             note), so zero is silent inertness and two is double-applied \
             hyperedge writes"
        );
        let carrier = graph.nodes("INSTITUTION");
        assert_eq!(carrier.len(), 1);
        let anchor = graph
            .node_attribute(carrier[0], "institution/community-carrier")
            .expect("the carrier carries the anchor field");
        assert_eq!(anchor.to_bits(), (1.0_f64).to_bits());
    }
}

// ---- Task 8: c00-c04 — the census and the org-weight decomposition ----
// Every assertion is bit-exact against the mirror (`.to_bits()`, the
// doc-comment transcript above) — no tolerance, per §9.

/// Run the pack over world 1, returning the post-tick graph.
fn tick_world_1() -> HypergraphStore {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report =
        run_once_into_with_prelude(SCENARIO, &practice_prelude(), PACK, &mut graph, &mut sink)
            .expect("world 1 + the c00-c04 pack ticks clean");
    assert_eq!(
        report.fired, 32,
        "c00:1 + c01:4 (active classes) + c02:5 (all classes) + c03f:1 + \
         c03l:1 + c03r:2 (rev-org AND n9 — n9 FIRES with an empty for-each, \
         frozen's :421 skip is structural) + c04:4 (active classes) + \
         c05:1 + c06a:1 + c06b:1 (Task 9) + c09:4 + c10:4 (active classes) \
         + c11:1 (the carrier — Task 10) + c07:1 + c08:1 (the DG-2 readout)"
    );
    graph
}

fn he(graph: &HypergraphStore, hid: u64, field: &str) -> f64 {
    graph
        .hyperedge_attribute(HyperedgeId(hid), field)
        .unwrap_or_else(|e| panic!("h{hid} {field}: {}", e.message))
}

fn node(graph: &HypergraphStore, nid: u64, field: &str) -> f64 {
    graph
        .node_attribute(NodeId(nid), field)
        .unwrap_or_else(|e| panic!("n{nid} {field}: {}", e.message))
}

/// Task 8 Step 1: the census counts ACTIVE members only — n5's membership
/// in h0 is real but inactive (frozen community.py:472-474).
#[test]
fn census_counts_only_active_members() {
    let graph = tick_world_1();
    assert_eq!(
        he(&graph, 0, "community/member-count").to_bits(),
        (2.0_f64).to_bits(),
        "h0: n1+n2, n5 excluded"
    );
    assert_eq!(
        he(&graph, 1, "community/member-count").to_bits(),
        (1.0_f64).to_bits(),
        "h1: n3"
    );
    assert_eq!(
        he(&graph, 2, "community/member-count").to_bits(),
        (1.0_f64).to_bits(),
        "h2: n1"
    );
}

/// Task 8 Step 1: n9 (`no-member-org`) has zero MEMBERSHIP edges, so its
/// cadre x cohesion = 1.0 x 1.0 push lands NOWHERE (frozen's :421 skip).
/// The strongest pin: the r-weight total across every class is exactly
/// rev-org's two pushes — 0.8, not 1.8.
#[test]
fn orgs_with_no_members_contribute_nothing() {
    let graph = tick_world_1();
    let r_total: f64 = (1..=5u64) // the five social classes (n0 is the carrier)
        .map(|nid| node(&graph, nid, "social-class/org-r-weight"))
        .sum();
    assert_eq!(
        r_total.to_bits(),
        (0.8_f64).to_bits(),
        "n9's 1.0 push never landed"
    );
    let count_total: f64 = (1..=5u64)
        .map(|nid| node(&graph, nid, "social-class/org-count"))
        .sum();
    assert_eq!(
        count_total.to_bits(),
        (5.0_f64).to_bits(),
        "exactly the five real MEMBERSHIP edges"
    );
}

/// Task 8 Step 1: the pushed weight is cadre x cohesion — n1's l-weight is
/// lib-org's 0.25 x 0.5 = 0.125, never the 0.75 the `+` mutation yields.
#[test]
fn org_weight_is_cadre_times_cohesion() {
    let graph = tick_world_1();
    assert_eq!(
        node(&graph, 1, "social-class/org-l-weight").to_bits(),
        (0.125_f64).to_bits()
    );
    assert_eq!(
        node(&graph, 1, "social-class/org-r-weight").to_bits(),
        (0.4_f64).to_bits(),
        "0.5 x 0.8"
    );
    assert_eq!(
        node(&graph, 3, "social-class/org-f-weight").to_bits(),
        (0.125_f64).to_bits(),
        "0.5 x 0.25"
    );
}

/// Task 8 Step 1: c04's divisor is the SAME-TICK census count (§8b row 1),
/// not a literal and not a stale tick — h0's r-raw is (0.4 + 0.4)/2 = 0.4;
/// h2's is 0.4/1 = 0.4.
#[test]
fn contribution_divides_by_the_census_count() {
    let graph = tick_world_1();
    assert_eq!(
        he(&graph, 0, "community/r-raw").to_bits(),
        (0.4_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 0, "community/l-raw").to_bits(),
        (0.0625_f64).to_bits(),
        "0.125/2, n1 only"
    );
    assert_eq!(
        he(&graph, 2, "community/r-raw").to_bits(),
        (0.4_f64).to_bits(),
        "0.4/1"
    );
    assert_eq!(
        he(&graph, 1, "community/f-raw").to_bits(),
        (0.125_f64).to_bits(),
        "0.125/1"
    );
}

/// Task 8 Step 1: density-sum counts org MEMBERSHIPS (org-count per
/// class), not orgs — h0 reads (2 + 1)/2 = 1.5, not 2/2 = 1.0.
#[test]
fn density_sum_counts_org_memberships_not_orgs() {
    let graph = tick_world_1();
    assert_eq!(
        he(&graph, 0, "community/density-sum").to_bits(),
        (1.5_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 1, "community/density-sum").to_bits(),
        (2.0_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 2, "community/density-sum").to_bits(),
        (2.0_f64).to_bits(),
        "n1's 2 orgs over 1 member"
    );
}

// ---- Task 9: c05 + c06a/c06b — normalization and the ADR214 floor ----
// (c07/c08 are DG-2-gated and do not exist until the Director's ruling.)
// Bit-exact against the mirror transcript (`.to_bits()`), per §9.

/// World 2 and world 3, single-homed beside world 1.
const SCENARIO_W2: &str = include_str!("../content/scenarios/community-floor-conformance.bscn");
const SCENARIO_W3: &str =
    include_str!("../content/scenarios/community-degenerate-conformance.bscn");

fn tick(scenario: &str, pack: &str) -> HypergraphStore {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into_with_prelude(scenario, &practice_prelude(), pack, &mut graph, &mut sink)
        .expect("the world + pack ticks clean");
    graph
}

/// Step 1: world 1's normalized ternaries — the pack's c05/c06 output,
/// bit-exact against the mirror. (No floor binds in world 1.)
#[test]
fn world_1_normalizes_to_the_mirror() {
    let graph = tick(SCENARIO, PACK);
    // h0: (0.4, 0.0625, 0.0) / 0.4625 — no bind (0.8648… > 0.136).
    assert_eq!(
        he(&graph, 0, "community/revolutionary").to_bits(),
        (0.8648648648648649_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 0, "community/liberal").to_bits(),
        (0.13513513513513511_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 0, "community/fascist").to_bits(),
        (0.0_f64).to_bits()
    );
    // h1: (0, 0.125, 0.125) / 0.25 — the (0.0, 0.5, 0.5) tie.
    assert_eq!(
        he(&graph, 1, "community/liberal").to_bits(),
        (0.5_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 1, "community/fascist").to_bits(),
        (0.5_f64).to_bits()
    );
    // h2: (0.4, 0.125, 0.0) / 0.525 — no bind (0.7619… > 0.04).
    assert_eq!(
        he(&graph, 2, "community/revolutionary").to_bits(),
        (0.7619047619047619_f64).to_bits()
    );
}

/// Step 1: the unorganized fraction defaults to liberal — world 2's h3 has
/// density-sum 0.5, so unorganized = 0.5 folds into l (Jackson: passive
/// acceptance is liberal hegemony). The 0.0 SETTLER floor never binds, so
/// the pure c05 shape is what this reads.
#[test]
fn unorganized_fraction_defaults_to_liberal() {
    let graph = tick(SCENARIO_W2, PACK);
    assert_eq!(
        he(&graph, 3, "community/density-sum").to_bits(),
        (0.5_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 3, "community/liberal").to_bits(),
        (0.9696969696969697_f64).to_bits(),
        "(0.0 + 0.5) / 0.515625 — the unorganized 0.5 IS the numerator"
    );
    assert_eq!(
        he(&graph, 3, "community/revolutionary").to_bits(),
        (0.030303030303030304_f64).to_bits(),
        "0.015625 / 0.515625"
    );
}

/// Step 1: the degenerate branch emits (0, 1, 0) and NOTHING else — the
/// floor routes through c06, bit-identically (§6.2 I5): world 3's h0 has a
/// zero-cadre only org, so every weight is 0, total collapses, and c06's
/// CHICANO arm (0.113) lands r and 1 − 0.113 on l through the lf main arm.
#[test]
fn degenerate_total_yields_floor_and_remainder() {
    let graph = tick(SCENARIO_W3, PACK);
    assert_eq!(
        he(&graph, 0, "community/revolutionary").to_bits(),
        (0.113_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 0, "community/liberal").to_bits(),
        (0.887_f64).to_bits(),
        "1.0 x 0.887 / 1.0 — exact"
    );
    assert_eq!(
        he(&graph, 0, "community/fascist").to_bits(),
        (0.0_f64).to_bits()
    );
}

/// Step 1: world 2's h0 — the NEW_AFRIKAN floor binds (normalized r ≈
/// 0.037 < 0.136) and the remaining 0.864 redistributes to l and f
/// PROPORTIONALLY (h0's weak-fash edge gives a nonzero f arm).
#[test]
fn floor_binds_and_redistributes_proportionally() {
    let graph = tick(SCENARIO_W2, PACK);
    assert_eq!(
        he(&graph, 0, "community/revolutionary").to_bits(),
        (0.136_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 0, "community/liberal").to_bits(),
        (0.8131764705882352_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 0, "community/fascist").to_bits(),
        (0.0508235294117647_f64).to_bits()
    );
}

/// Step 1: the lf <= 1e-10 arm — reachable ONLY synthetically (c05's
/// normalized outputs never produce it: r < floor <= 0.18 forces
/// l+f > 0.82). A test-scoped rig rule (byte-ordered c05 < c05z < c06a)
/// overwrites world 3's h0 to (0.05, 0, 0) after c05's normalize; c06b
/// then takes the else arm: l = remaining, f = 0. Dropping the else arm
/// reds this test (l stays 0).
#[test]
fn floor_redistribution_handles_zero_lf() {
    let rig = r#"
(rule community/c05z-rig-zero-lf
  :role mechanic :evidence derived :material-basis "Task 9 test rig: write the pathological (0.05, 0, 0) ternary onto world 3's h0 between c05 and c06a — the ONLY way lf <= 1e-10 with r < floor exists (c05's normalized outputs never produce it)"
  :fuel 64
  (domain NodeType/INSTITUTION)
  (bindings (binding carrier :field institution/community-carrier))
  (when #t)
  (effects
    (for-each (hyperedges HyperedgeType/COMMUNITY)
      (guard (= (field-of it community/kind) CommunityType/CHICANO)
        (update-hyperedge it community/revolutionary (set 0.05p))
        (update-hyperedge it community/liberal (set 0.0p))
        (update-hyperedge it community/fascist (set 0.0p))))))
"#;
    let mut pack_with_rig = PACK.to_owned();
    pack_with_rig.push_str(rig);
    let graph = tick(SCENARIO_W3, &pack_with_rig);
    assert_eq!(
        he(&graph, 0, "community/revolutionary").to_bits(),
        (0.113_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 0, "community/liberal").to_bits(),
        (0.887_f64).to_bits(),
        "the else arm: l = 1 - floor outright (no lf to scale against)"
    );
    assert_eq!(
        he(&graph, 0, "community/fascist").to_bits(),
        (0.0_f64).to_bits()
    );
}

/// Step 1: the SETTLER floor is identically 0.0 — the guard `r < floor`
/// is false at floor 0, so world 2's h2 keeps its normalized (0, 1, 0)
/// untouched (§6.2: the settler pole IS the norm).
#[test]
fn settler_floor_is_identically_zero() {
    let graph = tick(SCENARIO_W2, PACK);
    assert_eq!(
        he(&graph, 2, "community/substrate-floor").to_bits(),
        (0.0_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 2, "community/revolutionary").to_bits(),
        (0.0_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 2, "community/liberal").to_bits(),
        (1.0_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 2, "community/fascist").to_bits(),
        (0.0_f64).to_bits()
    );
}

/// Step 1: ADR214 Ruling 3 EXECUTED — world 2's FIRST_NATIONS community
/// lands r = 0.155 ABOVE its NEW_AFRIKAN community's 0.136, the first time
/// the frozen table's exact tie breaks (an emergent output of the chosen
/// measure, not a fresh stipulation).
#[test]
fn first_nations_floor_exceeds_new_afrikan() {
    let graph = tick(SCENARIO_W2, PACK);
    let na = he(&graph, 0, "community/revolutionary");
    let fn_ = he(&graph, 1, "community/revolutionary");
    assert_eq!(na.to_bits(), (0.136_f64).to_bits());
    assert_eq!(fn_.to_bits(), (0.155_f64).to_bits());
    assert!(
        fn_ > na,
        "Ruling 3: FIRST_NATIONS strictly exceeds NEW_AFRIKAN"
    );
    assert_eq!(
        he(&graph, 1, "community/liberal").to_bits(),
        (0.845_f64).to_bits()
    );
}

/// Step 1: frozen's `:452` skip gate — world 3's h1 (no org edges on its
/// member) keeps its seeded prior ternary BYTE-EXACTLY, and c06a's cache
/// write is skipped too (the field stays ABSENT — §9 item 5's law one
/// field over).
#[test]
fn community_without_orgs_keeps_its_prior_consciousness() {
    let graph = tick(SCENARIO_W3, PACK);
    assert_eq!(
        he(&graph, 1, "community/revolutionary").to_bits(),
        (0.125_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 1, "community/liberal").to_bits(),
        (0.75_f64).to_bits()
    );
    assert_eq!(
        he(&graph, 1, "community/fascist").to_bits(),
        (0.125_f64).to_bits()
    );
    assert!(
        graph
            .hyperedge_attribute(HyperedgeId(1), "community/substrate-floor")
            .is_err(),
        "a skipped community's floor cache is never written — honest null, never 0.0"
    );
}

/// Step 3's cross-world parity (DG-3's mechanism): every world's 14 floor
/// constants (and the three decay alphas) are equal to each other and to
/// the ADR214 values — a typo in one world cannot pass.
#[test]
fn the_floor_table_is_byte_identical_across_all_three_worlds() {
    let expected: [(&str, f64); 14] = [
        ("community/floor-settler", 0.0),
        ("community/floor-patriarchal", 0.0),
        ("community/floor-new-afrikan", 0.136),
        ("community/floor-first-nations", 0.155),
        ("community/floor-chicano", 0.113),
        ("community/floor-women", 0.04),
        ("community/floor-trans", 0.06),
        ("community/floor-disabled", 0.03),
        ("community/floor-queer", 0.04),
        ("community/floor-undocumented", 0.1),
        ("community/floor-incarcerated", 0.18),
        ("community/floor-youth", 0.0),
        ("community/floor-adult", 0.0),
        ("community/floor-elder", 0.02),
    ];
    for scenario in [SCENARIO, SCENARIO_W2, SCENARIO_W3] {
        let mut graph = HypergraphStore::new();
        let loaded = load_practice_scenario(scenario, &mut graph);
        for (name, value) in &expected {
            let actual = loaded
                .consts
                .get(*name)
                .unwrap_or_else(|| panic!("{name} missing from a world"));
            let babylon_bsl::evaluator::Value::Real(bits) = actual else {
                panic!("{name} must be a probability literal")
            };
            assert_eq!(
                bits.to_bits(),
                value.to_bits(),
                "{name} diverges from the ADR214 value in a world"
            );
        }
        for (name, value) in [
            ("community/heat-decay-alpha", 0.05_f64),
            ("community/cohesion-decay-alpha", 0.03_f64),
            ("community/education-pressure-decay", 0.1_f64),
        ] {
            let actual = loaded
                .consts
                .get(name)
                .unwrap_or_else(|| panic!("{name} missing from a world"));
            let babylon_bsl::evaluator::Value::Real(bits) = actual else {
                panic!("{name} must be a coefficient literal")
            };
            assert_eq!(
                bits.to_bits(),
                value.to_bits(),
                "{name} diverges from the frozen defines value in a world"
            );
        }
    }
}

// ---- Task 10: c09-c11 — cost modifiers and the state decay ----

/// World 4 (the cost-modifier world), single-homed beside worlds 1-3.
const SCENARIO_W4: &str =
    include_str!("../content/scenarios/community-cost-modifier-conformance.bscn");

/// Step 1: the modifier is the PRODUCT over the class's communities —
/// w4-both reads 0.75 x 1.25 = 0.9375, in ascending HyperedgeId order
/// (D25; D-NF+13's product-order note).
#[test]
fn cost_modifier_is_the_product_over_communities() {
    let graph = tick(SCENARIO_W4, PACK);
    assert_eq!(
        node(&graph, 1, "social-class/community-cost-modifier").to_bits(),
        (0.9375_f64).to_bits(),
        "w4-both: 0.75 x 1.25"
    );
    assert_eq!(
        node(&graph, 2, "social-class/community-cost-modifier").to_bits(),
        (0.75_f64).to_bits(),
        "w4-one: the single factor"
    );
}

/// Step 1: no memberships → c09's reset writes 1 and c10's empty for-each
/// leaves it — exactly 1.0, never a defaulted absence.
#[test]
fn cost_modifier_without_membership_is_exactly_one() {
    let graph = tick(SCENARIO_W4, PACK);
    assert_eq!(
        node(&graph, 3, "social-class/community-cost-modifier").to_bits(),
        (1.0_f64).to_bits(),
        "w4-none: the reset's value, untouched"
    );
}

/// Step 1 (C4's detector, the SECOND world): an inactive class is written
/// NOTHING — the field reads the substrate's honest-null ERROR, never 1.0
/// (the landed `.is_err()` shape of consciousness_ternary_conformance.rs:
/// 244-245). Mutation vector: deleting c09's active guard reds this.
#[test]
fn inactive_class_receives_no_cost_modifier_write() {
    // World 4's w4-inactive (n4) …
    let graph = tick(SCENARIO_W4, PACK);
    assert!(
        graph
            .node_attribute(NodeId(4), "social-class/community-cost-modifier")
            .is_err(),
        "w4-inactive: honest null, never 1.0"
    );
    // … and world 1's n5, the first home (one tick of the full pack).
    let graph = tick(SCENARIO, PACK);
    assert!(
        graph
            .node_attribute(NodeId(5), "social-class/community-cost-modifier")
            .is_err(),
        "world 1's n5: honest null, never 1.0"
    );
}

// ---- c07/c08 — the DG-2 readout (PUBLISH, the 2026-08-18 sitting) ----

/// The tie world (the §8a row-1 copies-agree home), single-homed here.
const SCENARIO_TIE: &str = include_str!("../content/scenarios/community-tie-conformance.bscn");
const CONSCIOUSNESS_PACK: &str = include_str!("../content/rules/consciousness.bsl");
/// The tie world consumes the shared WorldView registry through the
/// declarations prelude (the consciousness suite's own Train-B seam,
/// #591/D157).
const WORLDVIEW_PRELUDE: &str = include_str!("../content/declarations/worldview.bscn");

fn practice_worldview_prelude() -> String {
    compose_declaration_preludes(&[ORGANIZATION_PRACTICE_PRELUDE, WORLDVIEW_PRELUDE])
        .expect("the ordered organization and worldview preludes compose")
}

/// Step 4 (DG-2 landed): c07's contestation is the normalized Shannon
/// entropy, bit-exact against the mirror — the divisor is the ENGINE's
/// pinned soft-float `log(3)` (libm, 1.0986122886681096), one ulp below
/// CPython's glibc value; the mirror carries the engine's value, so these
/// pins are bit-exact against the PORT's arithmetic (the 1-ulp note is
/// D225's; a future world whose vector does NOT cancel the ulps re-opens
/// with a written tolerance policy, per the transcendental contract).
#[test]
fn contestation_is_normalized_shannon_entropy() {
    let graph = tick(SCENARIO, PACK);
    // World 1: h0 (0.8648…, 0.1351…, 0), h1 the (0, 0.5, 0.5) split, h2.
    assert_eq!(
        he(&graph, 0, "community/contestation").to_bits(),
        (0.3604848532176559_f64).to_bits(),
        "h0: -(0.8648…ln + 0.1351…ln) / log(3), the libm divisor"
    );
    assert_eq!(
        he(&graph, 1, "community/contestation").to_bits(),
        (0.6309297535714575_f64).to_bits(),
        "h1: the two-component split"
    );
    assert_eq!(
        he(&graph, 2, "community/contestation").to_bits(),
        (0.49960699524890273_f64).to_bits(),
        "h2: (0.7619…, 0.238…, 0)"
    );
    // World 2's fn-comm: the (0.155, 0.845, 0) post-floor shape.
    let graph = tick(SCENARIO_W2, PACK);
    assert_eq!(
        he(&graph, 1, "community/contestation").to_bits(),
        (0.3925724663663698_f64).to_bits(),
        "fn-comm: the post-floor two-component split"
    );
    // The skip gate: world 3's no-org community is written NOTHING.
    let graph = tick(SCENARIO_W3, PACK);
    assert!(
        graph
            .hyperedge_attribute(HyperedgeId(1), "community/contestation")
            .is_err(),
        "a skipped community's contestation is never written — honest null"
    );
}

/// Step 5 (DG-2 landed): c08's dominant tendency with the ruled tie order
/// — world 1's h1 is the (0.0, 0.5, 0.5) tie, and LIBERAL wins it.
#[test]
fn dominant_tendency_breaks_ties_liberal_first() {
    let graph = tick(SCENARIO, PACK);
    // ConsciousnessTendency frozen order: LIBERAL=0, FASCIST=1,
    // REVOLUTIONARY=2 (ADR195).
    assert_eq!(
        he(&graph, 0, "community/dominant-tendency").to_bits(),
        (2.0_f64).to_bits(),
        "h0: REVOLUTIONARY outright (0.8648…)"
    );
    assert_eq!(
        he(&graph, 1, "community/dominant-tendency").to_bits(),
        (0.0_f64).to_bits(),
        "h1: the (0, 0.5, 0.5) tie breaks LIBERAL-first"
    );
    assert_eq!(
        he(&graph, 2, "community/dominant-tendency").to_bits(),
        (2.0_f64).to_bits(),
        "h2: REVOLUTIONARY (0.7619…)"
    );
}

/// §8a row 1's copies-agree test: ONE world holding a class AND a
/// community at a three-way tie — BOTH readouts pick LIBERAL. The class
/// side is the landed `tv-tie-all-true` witness (NodeId(13) by this
/// file's declaration order — (0.333333p, 0.333333p, 0.333333p) seeded,
/// its pack-era writes move l by one ulp, and the ruled 1e-6 tie order
/// still picks LIBERAL); the community side's tie is COMPUTED exact (three
/// equal-weight orgs, one per tendency: 0.5/1.5 per arm, bit-identical
/// components). The mutation (swap the tie order in ONE pack) reds exactly
/// this.
#[test]
fn dominant_tendency_ties_match_the_class_surface_rule() {
    let co_loaded = format!("{CONSCIOUSNESS_PACK}\n{PACK}");
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    babylon_tick::run_once_into_with_prelude(
        SCENARIO_TIE,
        &practice_worldview_prelude(),
        &co_loaded,
        &mut graph,
        &mut sink,
    )
    .expect("the tie world ticks under both packs");
    // The community's readout: ConsciousnessTendency/LIBERAL is ordinal 0.
    let comm = graph
        .hyperedge_attribute(HyperedgeId(0), "community/dominant-tendency")
        .expect("c08 wrote the tied community's readout");
    assert_eq!(
        comm.to_bits(),
        (0.0_f64).to_bits(),
        "c08: LIBERAL (ordinal 0)"
    );
    // The class surface's readout on the landed tie witness
    // (tv-tie-all-true, NodeId(13) by declaration order): WorldView/LIBERAL
    // is ordinal 1 (REVOLUTIONARY, LIBERAL, FASCIST — the ruled order).
    let worldview = graph
        .node_attribute(NodeId(13), "social-class/dominant-worldview")
        .expect("p8 wrote the tied class's readout");
    assert_eq!(
        worldview.to_bits(),
        (1.0_f64).to_bits(),
        "p8: LIBERAL (WorldView ordinal 1) — the two readouts agree"
    );
}
