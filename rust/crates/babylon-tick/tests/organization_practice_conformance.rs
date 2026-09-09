//! Behavioral proof for situated organizational practice over relational territory.

use babylon_bsl::compose_declaration_preludes;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_kernel::SessionId;
use babylon_tick::TickSession;

const SCENARIO: &str = include_str!("../content/scenarios/organization-foundation.bscn");
const PACK: &str = include_str!("../content/rules/organization.bsl");
const ORGANIZATION_PRACTICE_PRELUDE: &str =
    include_str!("../content/declarations/organization-practice.bscn");

fn practice_prelude() -> String {
    compose_declaration_preludes(&[ORGANIZATION_PRACTICE_PRELUDE])
        .expect("the organization practice prelude composes")
}

const COUNTY_A: NodeId = NodeId(1);
const READING_GROUP: NodeId = NodeId(2);
const PRECINCT: NodeId = NodeId(3);
const COUNTY_B: NodeId = NodeId(6);
const COUNTY_C: NodeId = NodeId(7);
const COUNTY_D: NodeId = NodeId(9);

fn attribute(session: &TickSession<HypergraphStore>, node: NodeId, field: &str) -> f64 {
    session
        .graph()
        .node_attribute(node, field)
        .unwrap_or_else(|error| panic!("node {node:?} declares {field}: {error:?}"))
}

fn capacity(session: &TickSession<HypergraphStore>, territory: NodeId) -> f64 {
    attribute(session, territory, "territory/rooted-capacity")
}

fn rooted_work(session: &TickSession<HypergraphStore>, territory: NodeId) -> f64 {
    attribute(session, territory, "territory/rooted-work-inbox")
}

fn membership(session: &TickSession<HypergraphStore>) -> f64 {
    attribute(session, READING_GROUP, "organization/membership-share")
}

fn local_base_population(session: &TickSession<HypergraphStore>, territory: NodeId) -> f64 {
    session
        .graph()
        .edge_attribute(
            "PRESENCE",
            READING_GROUP,
            territory,
            "presence/local-base-population",
        )
        .unwrap_or_else(|error| panic!("PRESENCE local base must exist: {error:?}"))
}

#[test]
fn rooted_practice_requires_a_shared_presence_and_tenancy_territory() {
    let remote_base_scenario = SCENARIO.replace(
        "(edge EdgeType/TENANCY workers county 1)",
        "(edge EdgeType/TENANCY workers county-b 1)",
    );
    assert_ne!(
        remote_base_scenario, SCENARIO,
        "the social base must move outside the organization's presence"
    );
    let mut local = TickSession::new_with_prelude(
        SCENARIO,
        &practice_prelude(),
        PACK,
        HypergraphStore::new(),
        SessionId::new("organization-local-social-base").expect("literal is non-empty"),
    )
    .expect("the local-base world loads");
    let mut remote = TickSession::new_with_prelude(
        &remote_base_scenario,
        &practice_prelude(),
        PACK,
        HypergraphStore::new(),
        SessionId::new("organization-remote-social-base").expect("literal is non-empty"),
    )
    .expect("the remote-base world loads");
    let mut local_sink = CollectingSink::default();
    let mut remote_sink = CollectingSink::default();

    local.advance(&mut local_sink).expect("local-base tick");
    remote.advance(&mut remote_sink).expect("remote-base tick");

    assert!(rooted_work(&local, COUNTY_A) > 0.0);
    assert_eq!(rooted_work(&remote, COUNTY_A).to_bits(), 0.0_f64.to_bits());
}

#[test]
fn a_nonparticipating_presence_does_not_dilute_rooted_work() {
    let remote_branch_scenario = SCENARIO.replace(
        "  (edge EdgeType/SOLIDARITY reading-group precinct 1))",
        "  (edge EdgeType/SOLIDARITY reading-group precinct 1)\n  \
         (edge EdgeType/PRESENCE reading-group county-b 1)\n  \
         (edge-attr EdgeType/PRESENCE reading-group county-b presence/embedding PracticeEmbedding/NONE))",
    );
    assert_ne!(
        remote_branch_scenario, SCENARIO,
        "the organization must gain a remote branch"
    );
    let mut local_only = TickSession::new_with_prelude(
        SCENARIO,
        &practice_prelude(),
        PACK,
        HypergraphStore::new(),
        SessionId::new("organization-local-mean").expect("literal is non-empty"),
    )
    .expect("the local-only world loads");
    let mut with_remote_branch = TickSession::new_with_prelude(
        &remote_branch_scenario,
        &practice_prelude(),
        PACK,
        HypergraphStore::new(),
        SessionId::new("organization-local-mean-with-remote").expect("literal is non-empty"),
    )
    .expect("the remote-branch world loads");
    let mut local_sink = CollectingSink::default();
    let mut remote_sink = CollectingSink::default();

    local_only
        .advance(&mut local_sink)
        .expect("local-only tick");
    with_remote_branch
        .advance(&mut remote_sink)
        .expect("remote-branch tick");

    assert_eq!(
        local_base_population(&with_remote_branch, COUNTY_A).to_bits(),
        1000.0_f64.to_bits()
    );
    assert_eq!(
        local_base_population(&with_remote_branch, COUNTY_B).to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(
        rooted_work(&with_remote_branch, COUNTY_A).to_bits(),
        rooted_work(&local_only, COUNTY_A).to_bits()
    );
    assert_eq!(
        capacity(&with_remote_branch, COUNTY_A).to_bits(),
        capacity(&local_only, COUNTY_A).to_bits()
    );
    assert_eq!(
        membership(&with_remote_branch).to_bits(),
        membership(&local_only).to_bits(),
        "situated practice cannot write membership"
    );
}

#[test]
fn rooted_capacity_does_not_self_replicate_without_fresh_practice() {
    let mut session = TickSession::new_with_prelude(
        SCENARIO,
        &practice_prelude(),
        PACK,
        HypergraphStore::new(),
        SessionId::new("organization-practice-conformance").expect("literal is non-empty"),
    )
    .expect("the organization practice world loads");
    let mut sink = CollectingSink::default();

    session.advance(&mut sink).expect("tick 1");
    assert!(capacity(&session, COUNTY_A) > 0.0);
    assert!(capacity(&session, COUNTY_B) > 0.0);
    assert_eq!(capacity(&session, COUNTY_C).to_bits(), 0.0_f64.to_bits());
    let county_b_after_practice = capacity(&session, COUNTY_B);
    assert_eq!(
        attribute(&session, READING_GROUP, "organization/action-budget").to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(
        attribute(&session, PRECINCT, "organization/action-budget").to_bits(),
        1.0_f64.to_bits()
    );

    session.advance(&mut sink).expect("tick 2");
    assert_eq!(rooted_work(&session, COUNTY_A).to_bits(), 0.0_f64.to_bits());
    assert_eq!(
        capacity(&session, COUNTY_B).to_bits(),
        county_b_after_practice.to_bits()
    );
    assert_eq!(capacity(&session, COUNTY_C).to_bits(), 0.0_f64.to_bits());
}

#[test]
fn unique_low_buffer_corridor_relays_more_capacity_than_reroutable_corridor() {
    let mut session = TickSession::new_with_prelude(
        SCENARIO,
        &practice_prelude(),
        PACK,
        HypergraphStore::new(),
        SessionId::new("organization-circulation-bottleneck").expect("literal is non-empty"),
    )
    .expect("the organization practice world loads");
    let mut sink = CollectingSink::default();

    session
        .advance(&mut sink)
        .expect("circulation comparison tick");

    let unique_corridor_capacity = capacity(&session, COUNTY_B);
    let reroutable_corridor_capacity = capacity(&session, COUNTY_D);
    assert!(reroutable_corridor_capacity > 0.0);
    assert!(
        unique_corridor_capacity > reroutable_corridor_capacity,
        "unique corridor capacity {unique_corridor_capacity} did not exceed reroutable corridor capacity {reroutable_corridor_capacity}"
    );
}

#[test]
fn one_seeded_action_is_divided_across_the_organizations_branches() {
    let mut one_branch = TickSession::new_with_prelude(
        SCENARIO,
        &practice_prelude(),
        PACK,
        HypergraphStore::new(),
        SessionId::new("organization-one-branch").expect("literal is non-empty"),
    )
    .expect("the one-branch organization world loads");
    let two_branch_scenario = SCENARIO.replace(
        "  (edge EdgeType/SOLIDARITY reading-group precinct 1))",
        "  (edge EdgeType/SOLIDARITY reading-group precinct 1)\n  \
         (edge EdgeType/MEMBERSHIP reading-group workers-b 1)\n  \
         (edge EdgeType/PRESENCE reading-group county-b 1)\n  \
         (edge-attr EdgeType/PRESENCE reading-group county-b presence/embedding PracticeEmbedding/NEIGHBORHOOD))",
    );
    assert_ne!(
        two_branch_scenario, SCENARIO,
        "the second branch must be inserted"
    );
    let mut two_branches = TickSession::new_with_prelude(
        &two_branch_scenario,
        &practice_prelude(),
        PACK,
        HypergraphStore::new(),
        SessionId::new("organization-two-branches").expect("literal is non-empty"),
    )
    .expect("the two-branch organization world loads");
    let mut one_branch_sink = CollectingSink::default();
    let mut two_branch_sink = CollectingSink::default();

    one_branch
        .advance(&mut one_branch_sink)
        .expect("one-branch tick");
    two_branches
        .advance(&mut two_branch_sink)
        .expect("two-branch tick");

    let one_branch_work = rooted_work(&one_branch, COUNTY_A);
    let first_branch_work = rooted_work(&two_branches, COUNTY_A);
    let second_branch_work = rooted_work(&two_branches, COUNTY_B);
    assert!(first_branch_work > 0.0);
    assert!(second_branch_work > 0.0);
    assert!(first_branch_work < one_branch_work);
    assert!(first_branch_work + second_branch_work <= one_branch_work);
}

#[test]
fn practice_requires_presence_with_the_matching_material_embedding() {
    let workplace_practice = SCENARIO.replace(
        "(organization/practice-embedding PracticeEmbedding/NEIGHBORHOOD)",
        "(organization/practice-embedding PracticeEmbedding/WORKPLACE)",
    );
    assert_ne!(
        workplace_practice, SCENARIO,
        "the organization must select workplace practice"
    );
    let matched_workplace = workplace_practice.replace(
        "county presence/embedding PracticeEmbedding/NEIGHBORHOOD)",
        "county presence/embedding PracticeEmbedding/WORKPLACE)",
    );
    assert_ne!(
        matched_workplace, workplace_practice,
        "the presence relation must become a workplace embedding"
    );
    let mut mismatched = TickSession::new_with_prelude(
        &workplace_practice,
        &practice_prelude(),
        PACK,
        HypergraphStore::new(),
        SessionId::new("organization-mismatched-embedding").expect("literal is non-empty"),
    )
    .expect("the mismatched-embedding world loads");
    let mut matched = TickSession::new_with_prelude(
        &matched_workplace,
        &practice_prelude(),
        PACK,
        HypergraphStore::new(),
        SessionId::new("organization-matched-embedding").expect("literal is non-empty"),
    )
    .expect("the matched-embedding world loads");
    let mut mismatched_sink = CollectingSink::default();
    let mut matched_sink = CollectingSink::default();

    mismatched
        .advance(&mut mismatched_sink)
        .expect("mismatched-embedding tick");
    matched
        .advance(&mut matched_sink)
        .expect("matched-embedding tick");

    assert_eq!(
        rooted_work(&mismatched, COUNTY_A).to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(
        attribute(&mismatched, COUNTY_A, "territory/reproduction-pressure").to_bits(),
        0.9_f64.to_bits()
    );
    assert!(rooted_work(&matched, COUNTY_A) > 0.0);
    assert_eq!(
        attribute(&matched, COUNTY_A, "territory/reproduction-pressure").to_bits(),
        0.9_f64.to_bits(),
        "rooted work cannot fabricate care without stock, labor, and routing",
    );
}

#[test]
fn situated_practice_builds_capacity_without_direct_membership_or_care() {
    let mut session = TickSession::new_with_prelude(
        SCENARIO,
        &practice_prelude(),
        PACK,
        HypergraphStore::new(),
        SessionId::new("organization-situated-practice").expect("literal is non-empty"),
    )
    .expect("the organization practice world loads");
    let mut sink = CollectingSink::default();
    session.advance(&mut sink).expect("situated practice tick");

    assert!(capacity(&session, COUNTY_A) > 0.0);
    assert_eq!(membership(&session).to_bits(), 0.01_f64.to_bits());
    assert!(
        attribute(&session, COUNTY_A, "territory/command-pressure") > 0.0,
        "rooted organization should provoke a territorial command response"
    );
    assert_eq!(
        attribute(&session, COUNTY_A, "territory/reproduction-pressure").to_bits(),
        0.9_f64.to_bits(),
        "the organization circuit cannot fabricate care relief",
    );
}
