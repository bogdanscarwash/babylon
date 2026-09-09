use std::collections::HashMap;
use std::process::Command;

#[allow(
    dead_code,
    reason = "the tick integration reuses only the checked loader"
)]
#[path = "../../babylon-persistence/src/michigan_dynamic_hex_foundation.rs"]
mod michigan_dynamic_hex_foundation;

use babylon_bsl::causal_contract::{AuditReceipt, EffectSignature, EvidenceClass, RuleRole};
use babylon_bsl::evaluator::Value;
use babylon_bsl::identity_codec::StableBslValueV1;
use babylon_bsl::rule_pipeline::split_content;
use babylon_bsl::rules_hash_of;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::stable_element::{StableElementKeyV1, StableElementResolverV1};
use babylon_graph::stable_state::encode_stable_graph_state_v1;
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_graph::{allocator_state::AllocatorState, working_copy::DetachedCopy};
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
use babylon_kernel::tick_content_hash::{
    PreparedEnvironmentDigestV1, RefDigestV1, TickContentPartsV1, TickContentPreimageV1,
};
use babylon_kernel::{sha256_of, ContentDigest};
use babylon_practice_contract::actor_v2::ActorOrganizationIdV2;
use babylon_practice_contract::ordered_action_v1::{
    OrderedPracticeActionBatchV1, ORDERED_PRACTICE_ACTION_BATCH_V1_LAYOUT_VERSION,
};
use babylon_practice_contract::{
    input_authority_ledger_v2_digest, CampaignIdV2, InputAuthorityIdV2, PracticeAuthorityKindV2,
    PracticeIdV2, PracticeInputAuthorityLedgerV2, PracticeInputAuthorityV2, PracticeIntentV2,
    PracticeTargetIdentityV2, PracticeTargetTagV2, ProposalNonceV2, ResolvedPracticeBatchItemV2,
    ResolvedPracticeBatchV2, TaggedPracticeTargetV2,
};
use babylon_tick::committed_event::CommittedEventV2;
use babylon_tick::material_state::{
    MaterialStateErrorV1, MaterialStateRowRefV1, MaterialStateRowsV1, MaterialStateV1,
    WorldRegisterRowV1,
};
use babylon_tick::replay_identity::{
    encode_stable_world_v1, encode_tick_payload_v2, encode_world_register_set_v1,
    world_register_manifest_v1, ReplayTickIdentityError, STABLE_WORLD_LAYOUT_VERSION_V1,
    TICK_PAYLOAD_LAYOUT_VERSION_V2, WORLD_REGISTER_MANIFEST_LAYOUT_VERSION_V1,
    WORLD_REGISTER_SET_LAYOUT_VERSION_V1,
};
use babylon_tick::replay_session::{
    ReplayCommitAcknowledgementV1, ReplayCommitDispositionV1, ReplayTickError, ReplayTickSession,
};
use michigan_dynamic_hex_foundation::michigan_dynamic_hex_foundation_v1;

const REPLAY_SCENARIO: &str = r"
(scenario demo/finite-choice-two-classes
  (defenum ReplayOutcome (LOW HIGH))
  (deffield social-class/needs-roll int extensive)
  (deffield social-class/draw coefficient extensive)
  (node class-a NodeType/SOCIAL_CLASS
    (social-class/needs-roll 0)
    (social-class/draw 0.25c))
  (node class-b NodeType/SOCIAL_CLASS
    (social-class/needs-roll 1)
    (social-class/draw 0.25c)))
";
const MATERIAL_SCENARIO: &str = r"
(scenario demo/material-state
  (defenum OrgKind (POLITICAL_FACTION))
  (deffield social-class/needs-roll int extensive)
  (deffield social-class/draw coefficient extensive)
  (deffield organization/kind enum OrgKind)
  (deffield organization/members int extensive)
  (deffield organization/ratio real extensive)
  (node class-a NodeType/SOCIAL_CLASS
    (social-class/needs-roll 0))
  (node class-b NodeType/SOCIAL_CLASS
    (social-class/needs-roll 1))
  (node territory-a NodeType/TERRITORY)
  (node org-a NodeType/ORGANIZATION
    (organization/kind OrgKind/POLITICAL_FACTION)
    (organization/members 31)
    (organization/ratio 0))
  (edge EdgeType/PRESENCE org-a territory-a 1))
";
const DERIVED_TERRITORY_SCENARIO: &str = r"
(scenario demo/territory-derived
  (defvocabulary NodeType (TERRITORY))
  (defenum TerritoryType (CORE PERIPHERY))
  (deffield territory/population int extensive)
  (deffield territory/production-total real extensive)
  (deffield territory/heat intensity extensive)
  (deffield territory/territory-type enum TerritoryType)
  (deffield territory/treasury currency extensive)
  (node z NodeType/TERRITORY
    (territory/population 20)
    (territory/production-total 2.5r)
    (territory/heat 0.5i)
    (territory/territory-type TerritoryType/PERIPHERY)
    (territory/treasury 11$))
  (node aa NodeType/TERRITORY
    (territory/population 10)
    (territory/production-total 1.25r)
    (territory/heat 0.25i)
    (territory/territory-type TerritoryType/CORE)
    (territory/treasury 7$)))
";
const DERIVED_TERRITORY_RULE: &str = r#"
(rule territory/material-projection-witness
  :role mechanic
  :evidence derived
  :material-basis "post-tick territory material projection contract"
  :fuel 32
  (bindings
    (binding population :field territory/population)
    (binding heat :field territory/heat))
  (when #t)
  (effects
    (update-node self territory/population (add 3))
    (update-node self territory/heat (add 0.125i))))
"#;
const DERIVED_ORGANIZATION_SCENARIO: &str = r"
(scenario demo/organization-derived
  (defvocabulary NodeType (TERRITORY ORGANIZATION))
  (defenum OrgKind (BUSINESS POLITICAL_FACTION))
  (defenum OrgStatus (ACTIVE DORMANT))
  (deffield organization/kind enum OrgKind)
  (deffield organization/members int extensive)
  (deffield organization/productivity real extensive)
  (deffield organization/status enum OrgStatus)
  (deffield organization/treasury currency extensive)
  (node z NodeType/TERRITORY)
  (node aa NodeType/TERRITORY)
  (node f NodeType/ORGANIZATION
    (organization/kind OrgKind/POLITICAL_FACTION)
    (organization/members 10)
    (organization/productivity 1.25r)
    (organization/status OrgStatus/ACTIVE)
    (organization/treasury 7$))
  (node council NodeType/ORGANIZATION
    (organization/kind OrgKind/BUSINESS)
    (organization/members 20)
    (organization/productivity 2.5r)
    (organization/status OrgStatus/DORMANT)
    (organization/treasury 11$))
  (edge EdgeType/PRESENCE f z 1)
  (edge EdgeType/PRESENCE f aa 1))
";
const DERIVED_ORGANIZATION_RULE: &str = r#"
(rule organization/material-projection-witness
  :role mechanic
  :evidence derived
  :material-basis "post-tick organization material projection contract"
  :fuel 32
  (bindings
    (binding members :field organization/members))
  (when #t)
  (effects
    (update-node self organization/members (add 5))))
"#;
const REPLAY_RULE: &str = r#"
(rule struggle/spark-mechanic
  :role mechanic
  :evidence designed
  :material-basis "replay identity test exercises bounded finite material alternatives"
  :fuel 128
  (bindings
    (binding needs-roll :field social-class/needs-roll))
  (when #t)
  (effects
    (choose :sample struggle/spark :slot 0
      (branch ReplayOutcome/LOW
        :mass 1m
        (effects
          (update-node self social-class/draw (set 0.25c))))
      (branch ReplayOutcome/HIGH
        :mass 1m
        (effects
          (update-node self social-class/draw (set 0.75c)))))))

(rule struggle/spark-recognizer
  :role recognizer
  :evidence derived
  :projects-kernel struggle/spark
  :material-basis "adjacent observation of the realized replay alternative"
  :fuel 64
  (bindings
    (binding draw :field social-class/draw))
  (when (= draw 0.75c))
  (effects
    (emit EventType/EXCESSIVE_FORCE
      (subject self))))
"#;
const RETAINED_OUTPUT_RULE: &str = r#"
(rule vitality/retained-replay-output
  :role mechanic
  :evidence derived
  :material-basis "completed replay output retention contract"
  :fuel 32
  (bindings
    (binding needs-roll :field social-class/needs-roll))
  (when #t)
  (effects
    (update-node self social-class/draw (set 0.25c))
    (emit EventType/REPLAY_RETAINED
      (subject self)
      (needs-roll needs-roll))))
"#;
const DUPLICATE_RETAINED_OUTPUT_RULE: &str = r#"
(rule vitality/duplicate-retained-replay-output
  :role mechanic
  :evidence derived
  :material-basis "retained event fields must have unique canonical names"
  :fuel 32
  (bindings
    (binding needs-roll :field social-class/needs-roll))
  (when #t)
  (effects
    (update-node self social-class/draw (set 0.75c))
    (emit EventType/REPLAY_DUPLICATE
      (subject self)
      (subject self))))
"#;
const PROCESS_CHILD_ENV: &str = "BABYLON_PER60_REPLAY_CHILD";
const PROCESS_MARKER: &str = "PER60_REPLAY=";
const REPLAY_VECTOR_SEED: i64 = 811;
const CHANGED_REPLAY_VECTOR_SEED: i64 = 812;
const FAILURE_SCENARIO: &str = r"
(scenario tick/replay-failure
  (defvocabulary NodeType (SOCIAL_CLASS))
  (deffield social-class/probability probability intensive)
  (node first NodeType/SOCIAL_CLASS (social-class/probability 0.1p))
  (node second NodeType/SOCIAL_CLASS (social-class/probability 0.9p)))
";
const FAILURE_RULE: &str = r#"
(rule vitality/replay-failure
  :role mechanic
  :evidence derived
  :material-basis "one valid event precedes a domain-invalid write in the detached graph"
  :fuel 64
  (bindings (binding probability :field social-class/probability))
  (when (> probability 0.0p))
  (effects
    (emit EventType/REPLAY_FAILURE)
    (update-node self social-class/probability (add 0.4i))))
"#;

fn str32(value: &str) -> Vec<u8> {
    [
        u32::try_from(value.len()).unwrap().to_be_bytes().as_slice(),
        value.as_bytes(),
    ]
    .concat()
}

fn content_for(rule_src: &str) -> ContentDigest {
    let (_, rules) = split_content(rule_src).unwrap();
    let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
    ContentDigest {
        defines_hash: [0x2d; 32],
        rules_hash: rules_hash_of(&forms).unwrap(),
    }
}

fn michigan_foundation() -> &'static babylon_tick::h3_runtime::MichiganDynamicHexFoundationV1 {
    michigan_dynamic_hex_foundation_v1().expect("the governed foundation must decode once")
}

fn foundation_reference() -> RefDigestV1 {
    RefDigestV1::from_bytes(michigan_foundation().reference_bundle_digest())
}

fn stable_material_node(local_name: &str) -> StableElementKeyV1 {
    StableElementKeyV1::Node {
        scenario: "demo/material-state".to_owned(),
        local_name: local_name.to_owned(),
    }
}

fn foundation_material_state() -> MaterialStateV1 {
    MaterialStateV1::try_new(michigan_foundation()).unwrap()
}

#[test]
fn material_state_is_foundation_only_with_exact_graph_derived_families() {
    let replay_id = ReplaySessionIdV1::try_from("per281/foundation-only-material").unwrap();
    let mut session = ReplayTickSession::new(
        MATERIAL_SCENARIO,
        None,
        RETAINED_OUTPUT_RULE,
        HypergraphStore::new(),
        replay_id.clone(),
        ReplaySeed::new(37),
        content_for(RETAINED_OUTPUT_RULE),
        foundation_reference(),
        MaterialStateV1::try_new(michigan_foundation()).unwrap(),
    )
    .unwrap();
    let actions = OrderedPracticeActionBatchV1::empty(replay_id, 1).unwrap();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let report = session.advance(&mut sink, &actions).unwrap();
    let material = report.material_state_rows();
    let dynamic = material.dynamic_hexes();

    assert_eq!(dynamic.source_count(), 45_572);
    for index in [0, 22_786, 45_571] {
        assert_eq!(
            dynamic.rows()[index].cell_id(),
            michigan_foundation().rows()[index].cell_id()
        );
        assert_eq!(
            dynamic.rows()[index].value_bits(),
            michigan_foundation().rows()[index].value_bits()
        );
    }
    assert_eq!(
        dynamic.source_digest(),
        [
            0x81, 0x38, 0xea, 0x07, 0x1e, 0xdc, 0x58, 0x23, 0xbd, 0x7b, 0x7b, 0xab, 0x89, 0xe4,
            0xa5, 0x13, 0x87, 0x91, 0x7f, 0x56, 0x46, 0xee, 0x8f, 0x82, 0xeb, 0x14, 0xb2, 0x33,
            0xd2, 0x61, 0x3e, 0x70,
        ]
    );
    assert_eq!(material.territories().source_count(), 1);
    assert_eq!(material.organizations().source_count(), 1);
}

#[test]
fn dynamic_h3_rows_are_activated_from_the_exact_foundation_and_reference() {
    let foundation =
        michigan_dynamic_hex_foundation_v1().expect("the governed foundation must decode once");
    let expected_reference = RefDigestV1::from_bytes(foundation.reference_bundle_digest());
    let foreign_reference = RefDigestV1::from_bytes([0x55; 32]);
    let foreign_material = MaterialStateV1::try_new(foundation).unwrap();
    let foreign_error = ReplayTickSession::new(
        "this is deliberately not a scenario",
        None,
        RETAINED_OUTPUT_RULE,
        HypergraphStore::new(),
        ReplaySessionIdV1::try_from("per281/foundation-reference-refusal").unwrap(),
        ReplaySeed::new(37),
        content_for(RETAINED_OUTPUT_RULE),
        foreign_reference,
        foreign_material,
    )
    .err()
    .expect("foreign reference must refuse before scenario preparation");
    assert_eq!(
        foreign_error,
        ReplayTickError::MaterialState(MaterialStateErrorV1::ReferenceBundleMismatch {
            expected: foundation.reference_bundle_digest(),
            actual: *foreign_reference.as_bytes(),
        })
    );

    let material = MaterialStateV1::try_new(foundation).unwrap();
    let replay_id = ReplaySessionIdV1::try_from("per281/foundation-activation").unwrap();
    let mut session = ReplayTickSession::new(
        MATERIAL_SCENARIO,
        None,
        RETAINED_OUTPUT_RULE,
        HypergraphStore::new(),
        replay_id.clone(),
        ReplaySeed::new(37),
        content_for(RETAINED_OUTPUT_RULE),
        expected_reference,
        material,
    )
    .unwrap();
    let actions = OrderedPracticeActionBatchV1::empty(replay_id, 1).unwrap();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let report = session.advance(&mut sink, &actions).unwrap();
    let dynamic = report.material_state_rows().dynamic_hexes();

    assert_eq!(dynamic.source_count(), 45_572);
    assert!(dynamic
        .rows()
        .windows(2)
        .all(|rows| rows[0].cell_id().as_u64() < rows[1].cell_id().as_u64()));
    for index in [0, 22_786, 45_571] {
        assert_eq!(
            dynamic.rows()[index].cell_id(),
            foundation.rows()[index].cell_id()
        );
        assert_eq!(
            dynamic.rows()[index].value_bits(),
            foundation.rows()[index].value_bits()
        );
    }
    assert_eq!(
        dynamic.source_digest(),
        sha256_of(dynamic.canonical_bytes())
    );
}

fn nonempty_action_batch(session: ReplaySessionIdV1) -> OrderedPracticeActionBatchV1 {
    let authority = PracticeInputAuthorityV2 {
        schema_version: 2,
        campaign_id: CampaignIdV2::from_bytes([0x10; 16]),
        authority_kind: PracticeAuthorityKindV2::PlayerSeat,
        input_authority_id: InputAuthorityIdV2::from_bytes([0x20; 16]),
        actor_org_id: ActorOrganizationIdV2::from_bytes(7_u64.to_be_bytes()),
        effective_from_tick: 10,
        effective_through_tick_exclusive: 20,
        decision_content_digest: [0x30; 32],
    };
    let ledger = PracticeInputAuthorityLedgerV2 {
        schema_version: 2,
        rows: vec![authority.clone()],
    };
    let intent = PracticeIntentV2 {
        schema_version: 2,
        submit_after_tick: 10,
        resolve_tick: 11,
        input_authority_id: InputAuthorityIdV2::from_bytes([0x20; 16]),
        actor_org_id: ActorOrganizationIdV2::from_bytes(7_u64.to_be_bytes()),
        practice_id: PracticeIdV2::Strike,
        target: TaggedPracticeTargetV2 {
            tag: PracticeTargetTagV2::LaborProcess,
            identity: PracticeTargetIdentityV2::from_bytes([0x50; 32]),
        },
        proposal_nonce: ProposalNonceV2::from_bytes([0x60; 16]),
        quoted_content_digest: [0x30; 32],
        quoted_resource_contract_digest: [0x40; 32],
        parameters: Vec::new(),
        evidence_digests: vec![[0x70; 32]],
    };
    let source = ResolvedPracticeBatchV2 {
        schema_version: 2,
        campaign_id: CampaignIdV2::from_bytes([0x10; 16]),
        resolve_tick: 11,
        authority_ledger_digest: input_authority_ledger_v2_digest(&ledger).unwrap(),
        resource_allocation_contract_digest: [0x40; 32],
        content_digest: [0x30; 32],
        items: vec![ResolvedPracticeBatchItemV2 { authority, intent }],
    };
    OrderedPracticeActionBatchV1::project(session, &source, &ledger).unwrap()
}

#[derive(Debug, PartialEq, Eq)]
struct ReplayRun {
    outer: String,
    prepared: String,
    prior: String,
    result: String,
    payload: String,
    actions: String,
    choice_receipt_digest: [u8; 32],
    selected_outcomes: Vec<String>,
    draw_tickets: Vec<u64>,
    result_bits: Vec<u64>,
}

fn run_replay<G>(seed: i64) -> ReplayRun
where
    G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy + Default,
{
    let replay_id = ReplaySessionIdV1::try_from("per60/cross-substrate").unwrap();
    let mut session = ReplayTickSession::new(
        REPLAY_SCENARIO,
        None,
        REPLAY_RULE,
        G::default(),
        replay_id.clone(),
        ReplaySeed::new(seed),
        content_for(REPLAY_RULE),
        foundation_reference(),
        foundation_material_state(),
    )
    .unwrap();
    let actions = OrderedPracticeActionBatchV1::empty(replay_id, 1).unwrap();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let report = session.advance(&mut sink, &actions).unwrap();
    let choice_receipts = &report.report().choice_receipts;
    ReplayRun {
        outer: report.tick_content_hash().to_hex(),
        prepared: report.prepared_environment_digest().to_hex(),
        prior: report.prior_world().digest().to_hex(),
        result: report.result_world().digest().to_hex(),
        payload: report.payload().digest().to_hex(),
        actions: report.action_batch_digest().to_hex(),
        choice_receipt_digest: report.choice_receipt_source_digest(),
        selected_outcomes: choice_receipts
            .iter()
            .map(|receipt| receipt.selected_outcome().to_owned())
            .collect(),
        draw_tickets: choice_receipts
            .iter()
            .map(|receipt| receipt.draw_ticket())
            .collect(),
        result_bits: [NodeId(0), NodeId(1)]
            .into_iter()
            .map(|node| {
                session
                    .graph()
                    .node_attribute(node, "social-class/draw")
                    .unwrap()
                    .to_bits()
            })
            .collect(),
    }
}

#[test]
fn replay_session_constructor_is_typed_to_v2_identity() {
    let _constructor = ReplayTickSession::<MemoryGraph>::new;
}

#[test]
fn replay_session_publishes_exact_identity_and_retains_static_bytes_once() {
    let replay_id = ReplaySessionIdV1::try_from("per60/session-a").unwrap();
    let seed = ReplaySeed::new(-71);
    let content = content_for(REPLAY_RULE);
    let reference = foundation_reference();
    let mut session = ReplayTickSession::new(
        REPLAY_SCENARIO,
        None,
        REPLAY_RULE,
        HypergraphStore::new(),
        replay_id.clone(),
        seed,
        content.clone(),
        reference,
        foundation_material_state(),
    )
    .unwrap();
    let resolver_storage = session.resolver_manifest_bytes().as_ptr();
    let register_storage = session.register_manifest_bytes().as_ptr();
    let prepared_storage = session.prepared_environment_bytes().as_ptr();
    let prepared_digest =
        PreparedEnvironmentDigestV1::from_bytes(sha256_of(session.prepared_environment_bytes()));
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let actions = OrderedPracticeActionBatchV1::empty(replay_id.clone(), 1).unwrap();
    let identified = session.advance(&mut sink, &actions).unwrap();

    assert_eq!(session.completed_tick(), 1);
    assert_eq!(identified.action_batch_bytes(), actions.canonical_bytes());
    assert_eq!(
        identified.action_batch_layout_version(),
        ORDERED_PRACTICE_ACTION_BATCH_V1_LAYOUT_VERSION
    );
    assert_eq!(identified.action_batch_digest(), actions.digest());
    assert_eq!(identified.prior_registers().completed_tick(), 0);
    assert_eq!(identified.result_registers().completed_tick(), 1);
    assert_eq!(identified.prepared_environment_digest(), prepared_digest);
    assert_eq!(
        identified.tick_content_hash(),
        identified.outer_preimage().digest()
    );
    assert_eq!(identified.report().choice_receipts.len(), 2);
    let selected_high_count = identified
        .report()
        .choice_receipts
        .iter()
        .filter(|receipt| receipt.selected_outcome() == "HIGH")
        .count();
    assert_eq!(identified.report().fired, 2 + selected_high_count);
    assert_eq!(
        identified.report().committed_events.len(),
        selected_high_count
    );
    for event in &identified.report().committed_events {
        assert_eq!(event.emitting_rule(), "struggle/spark-recognizer");
        assert_eq!(event.event_type(), "EXCESSIVE_FORCE");
        let receipt_ordinal = event
            .choice_receipt()
            .expect("projected replay event must retain its choice provenance")
            .encounter_ordinal();
        assert_eq!(
            identified.report().choice_receipts[receipt_ordinal as usize].selected_outcome(),
            "HIGH"
        );
    }

    let independently_composed = TickContentPreimageV1::compose(&TickContentPartsV1 {
        session: &replay_id,
        resolve_tick: 1,
        seed,
        content: &content,
        reference,
        prepared: prepared_digest,
        prior_world: identified.prior_world().digest(),
        actions: actions.digest(),
        result_world: identified.result_world().digest(),
        payload: identified.payload().digest(),
    })
    .unwrap();
    assert_eq!(
        identified.outer_preimage().as_bytes(),
        independently_composed.as_bytes()
    );

    let next_actions = OrderedPracticeActionBatchV1::empty(replay_id, 2).unwrap();
    let second = session.advance(&mut sink, &next_actions).unwrap();
    assert_eq!(second.prior_registers().completed_tick(), 1);
    assert_eq!(second.result_registers().completed_tick(), 2);
    assert_eq!(session.resolver_manifest_bytes().as_ptr(), resolver_storage);
    assert_eq!(session.register_manifest_bytes().as_ptr(), register_storage);
    assert_eq!(
        session.prepared_environment_bytes().as_ptr(),
        prepared_storage
    );
}

#[test]
fn completed_replay_retains_typed_graph_rows_and_successful_event_batch() {
    let replay_id = ReplaySessionIdV1::try_from("per281/retained-output").unwrap();
    let mut session = ReplayTickSession::new(
        REPLAY_SCENARIO,
        None,
        RETAINED_OUTPUT_RULE,
        HypergraphStore::new(),
        replay_id.clone(),
        ReplaySeed::new(17),
        content_for(RETAINED_OUTPUT_RULE),
        foundation_reference(),
        foundation_material_state(),
    )
    .unwrap();
    let actions = OrderedPracticeActionBatchV1::empty(replay_id, 1).unwrap();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();

    let identified = session.advance(&mut sink, &actions).unwrap();

    let stable_graph = identified.result_stable_graph();
    assert_eq!(
        stable_graph.digest(),
        identified.result_stable_graph_digest()
    );
    let draw_rows = stable_graph
        .rows()
        .node_f64()
        .iter()
        .filter(|(_, qname, _)| qname == "social-class/draw")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        draw_rows,
        vec![
            (
                "class-a".to_owned(),
                "social-class/draw".to_owned(),
                0.25_f64.to_bits(),
            ),
            (
                "class-b".to_owned(),
                "social-class/draw".to_owned(),
                0.25_f64.to_bits(),
            ),
        ]
    );

    let events = identified.successful_event_batch().events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_type(), "REPLAY_RETAINED");
    assert_eq!(
        events[0].fields(),
        &[
            (
                "needs-roll".to_owned(),
                StableBslValueV1::RealBits(0.0_f64.to_bits()),
            ),
            (
                "subject".to_owned(),
                StableBslValueV1::Node(StableElementKeyV1::Node {
                    scenario: "demo/finite-choice-two-classes".to_owned(),
                    local_name: "class-a".to_owned(),
                },),
            ),
        ]
    );
    assert_eq!(events[1].event_type(), "REPLAY_RETAINED");
    assert_eq!(
        events[1].fields(),
        &[
            (
                "needs-roll".to_owned(),
                StableBslValueV1::RealBits(1.0_f64.to_bits()),
            ),
            (
                "subject".to_owned(),
                StableBslValueV1::Node(StableElementKeyV1::Node {
                    scenario: "demo/finite-choice-two-classes".to_owned(),
                    local_name: "class-b".to_owned(),
                },),
            ),
        ]
    );
    assert_eq!(
        sink.events,
        vec![
            (
                "REPLAY_RETAINED".to_owned(),
                vec![
                    ("subject".to_owned(), Value::NodeRef(NodeId(0))),
                    ("needs-roll".to_owned(), Value::Real(0.0)),
                ],
            ),
            (
                "REPLAY_RETAINED".to_owned(),
                vec![
                    ("subject".to_owned(), Value::NodeRef(NodeId(1))),
                    ("needs-roll".to_owned(), Value::Real(1.0)),
                ],
            ),
        ]
    );
}

#[test]
fn prepared_replay_publishes_once_only_after_exact_commit_acknowledgement() {
    let replay_id = ReplaySessionIdV1::try_from("per281/prepared-commit-ack").unwrap();
    let mut session = ReplayTickSession::new(
        REPLAY_SCENARIO,
        None,
        RETAINED_OUTPUT_RULE,
        HypergraphStore::new(),
        replay_id.clone(),
        ReplaySeed::new(43),
        content_for(RETAINED_OUTPUT_RULE),
        foundation_reference(),
        foundation_material_state(),
    )
    .unwrap();
    let actions = OrderedPracticeActionBatchV1::empty(replay_id, 1).unwrap();
    let before_graph = session.graph().encode_state().unwrap().as_bytes().to_vec();
    let before_cursors = session.graph().allocator_cursors();
    let before_material = foundation_material_state();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink {
        events: vec![("EventType/PRIOR".to_owned(), Vec::new())],
    };
    let before_events = sink.events.clone();

    let abandoned = session.prepare_advance(&actions).unwrap();
    assert_eq!(abandoned.report().result_registers().completed_tick(), 1);
    drop(abandoned);
    assert_eq!(session.completed_tick(), 0);
    assert_eq!(
        session.graph().encode_state().unwrap().as_bytes(),
        before_graph
    );
    assert_eq!(session.graph().allocator_cursors(), before_cursors);
    assert_eq!(session.material_state(), &before_material);
    assert_eq!(sink.events, before_events);

    let prepared = session.prepare_advance(&actions).unwrap();
    let stale_after_first_publication = session.prepare_advance(&actions).unwrap();
    let tick_content_hash = prepared.report().tick_content_hash();
    let acknowledgement = ReplayCommitAcknowledgementV1::new(
        ReplayCommitDispositionV1::Committed,
        1,
        tick_content_hash,
    );
    let report = session
        .acknowledge_prepared(&mut sink, prepared, acknowledgement)
        .unwrap();
    assert_eq!(session.completed_tick(), 1);
    assert_eq!(report.tick_content_hash(), tick_content_hash);
    assert_eq!(session.graph().state_hash().unwrap(), report.report().after);
    assert_eq!(session.graph().allocator_cursors(), before_cursors);
    assert_eq!(sink.events.first(), before_events.first());
    assert_eq!(sink.events.len(), 3);

    let events_after_first_ack = sink.events.clone();
    let ambiguous_acknowledgement = ReplayCommitAcknowledgementV1::new(
        ReplayCommitDispositionV1::ReconciledAfterAmbiguousCommit,
        1,
        stale_after_first_publication.report().tick_content_hash(),
    );
    assert_eq!(
        session
            .acknowledge_prepared(
                &mut sink,
                stale_after_first_publication,
                ambiguous_acknowledgement,
            )
            .unwrap_err(),
        ReplayTickError::StalePreparedTick {
            prepared_after: 0,
            live_completed: 1,
        }
    );
    assert_eq!(session.completed_tick(), 1);
    assert_eq!(sink.events, events_after_first_ack);
}

fn assert_populated_material_rows(material: &MaterialStateRowsV1) {
    assert_eq!(material.source_count(), 45_575);
    assert_eq!(material.rows().len(), 45_575);
    assert_eq!(
        material.source_digest(),
        sha256_of(material.canonical_bytes())
    );
    assert_eq!(material.world_registers().source_count(), 1);
    assert_eq!(material.territories().source_count(), 1);
    assert_eq!(material.dynamic_hexes().source_count(), 45_572);
    assert_eq!(material.organizations().source_count(), 1);
    let mut material_rows = material.rows();
    assert!(matches!(
        material_rows.next(),
        Some(MaterialStateRowRefV1::WorldRegister(world))
            if world == &WorldRegisterRowV1::try_new(
                "world/completed-tick".to_owned(),
                StableBslValueV1::Int(1),
            ).unwrap()
    ));
    let Some(MaterialStateRowRefV1::Territory(territory)) = material_rows.next() else {
        panic!("material rows lost the derived territory family")
    };
    assert_eq!(
        territory.territory_id(),
        &stable_material_node("territory-a")
    );
    assert!(territory.ordered_fields().is_empty());
    for foundation_row in michigan_foundation().rows() {
        let Some(MaterialStateRowRefV1::DynamicHex(dynamic_hex)) = material_rows.next() else {
            panic!("material rows lost the exact dynamic-H3 family")
        };
        assert_eq!(dynamic_hex.cell_id(), foundation_row.cell_id());
        assert_eq!(dynamic_hex.value_bits(), foundation_row.value_bits());
    }
    let Some(MaterialStateRowRefV1::Organization(organization)) = material_rows.next() else {
        panic!("material rows lost the derived organization family")
    };
    assert!(material_rows.next().is_none());
    assert_eq!(
        organization.organization_id(),
        &stable_material_node("org-a")
    );
    assert_eq!(
        organization.organization_kind(),
        &StableBslValueV1::Enum {
            enum_type: "OrgKind".to_owned(),
            member: "POLITICAL_FACTION".to_owned(),
        }
    );
    assert_eq!(
        organization.ordered_territory_ids(),
        &[stable_material_node("territory-a")]
    );
    assert_eq!(
        organization.ordered_fields(),
        &[
            ("members".to_owned(), StableBslValueV1::Int(31)),
            (
                "ratio".to_owned(),
                StableBslValueV1::RealBits(0.0_f64.to_bits()),
            ),
        ]
    );
}

fn assert_empty_material_rows(empty: &MaterialStateRowsV1) {
    assert_eq!(empty.source_count(), 45_573);
    assert_eq!(empty.rows().len(), 45_573);
    assert_eq!(empty.world_registers().source_count(), 1);
    assert_eq!(empty.territories().source_count(), 0);
    assert_eq!(empty.dynamic_hexes().source_count(), 45_572);
    assert_eq!(empty.organizations().source_count(), 0);
    for (digest, bytes) in [
        (
            empty.territories().source_digest(),
            empty.territories().canonical_bytes(),
        ),
        (
            empty.organizations().source_digest(),
            empty.organizations().canonical_bytes(),
        ),
    ] {
        assert_eq!(digest, sha256_of(bytes));
    }
    let mut empty_rows = empty.rows();
    assert!(matches!(
        empty_rows.next(),
        Some(MaterialStateRowRefV1::WorldRegister(world))
            if world == &WorldRegisterRowV1::try_new(
                "world/completed-tick".to_owned(),
                StableBslValueV1::Int(1),
            ).unwrap()
    ));
    for foundation_row in michigan_foundation().rows() {
        assert!(matches!(
            empty_rows.next(),
            Some(MaterialStateRowRefV1::DynamicHex(dynamic))
                if dynamic.cell_id() == foundation_row.cell_id()
                    && dynamic.value_bits() == foundation_row.value_bits()
        ));
    }
    assert!(empty_rows.next().is_none());
}

#[test]
fn completed_replay_retains_every_source_owned_material_family() {
    assert_eq!(
        WorldRegisterRowV1::try_new("world/completed-tick".to_owned(), StableBslValueV1::Int(-1),)
            .unwrap_err(),
        MaterialStateErrorV1::WorldRegister
    );
    let replay_id = ReplaySessionIdV1::try_from("per281/material-complete").unwrap();
    let material_state = foundation_material_state();
    let mut session = ReplayTickSession::new(
        MATERIAL_SCENARIO,
        None,
        RETAINED_OUTPUT_RULE,
        HypergraphStore::new(),
        replay_id.clone(),
        ReplaySeed::new(37),
        content_for(RETAINED_OUTPUT_RULE),
        foundation_reference(),
        material_state,
    )
    .unwrap();
    let actions = OrderedPracticeActionBatchV1::empty(replay_id, 1).unwrap();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();

    let identified = session.advance(&mut sink, &actions).unwrap();
    let material = identified.material_state_rows();
    assert_populated_material_rows(material);

    let empty_replay_id = ReplaySessionIdV1::try_from("per281/material-empty").unwrap();
    let mut empty_session = ReplayTickSession::new(
        REPLAY_SCENARIO,
        None,
        RETAINED_OUTPUT_RULE,
        HypergraphStore::new(),
        empty_replay_id.clone(),
        ReplaySeed::new(43),
        content_for(RETAINED_OUTPUT_RULE),
        foundation_reference(),
        foundation_material_state(),
    )
    .unwrap();
    let empty_actions = OrderedPracticeActionBatchV1::empty(empty_replay_id, 1).unwrap();
    let mut empty_sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let empty_identified = empty_session
        .advance(&mut empty_sink, &empty_actions)
        .unwrap();
    let empty = empty_identified.material_state_rows();
    assert_empty_material_rows(empty);
}

#[test]
fn territory_rows_are_derived_from_the_post_tick_graph() {
    let short_id = StableElementKeyV1::Node {
        scenario: "demo/territory-derived".to_owned(),
        local_name: "z".to_owned(),
    };
    let long_id = StableElementKeyV1::Node {
        scenario: "demo/territory-derived".to_owned(),
        local_name: "aa".to_owned(),
    };
    assert!(
        b"aa".as_slice() < b"z".as_slice(),
        "stable graph local-name order is lexical"
    );
    assert!(
        short_id.canonical_bytes().unwrap() < long_id.canonical_bytes().unwrap(),
        "length-framed material primary-key order must put z before aa"
    );
    let replay_id = ReplaySessionIdV1::try_from("per281/territory-derived").unwrap();
    let mut session = ReplayTickSession::new(
        DERIVED_TERRITORY_SCENARIO,
        None,
        DERIVED_TERRITORY_RULE,
        HypergraphStore::new(),
        replay_id.clone(),
        ReplaySeed::new(53),
        content_for(DERIVED_TERRITORY_RULE),
        foundation_reference(),
        foundation_material_state(),
    )
    .unwrap();
    let actions = OrderedPracticeActionBatchV1::empty(replay_id, 1).unwrap();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let identified = session.advance(&mut sink, &actions).unwrap();
    let territory_rows = identified.material_state_rows().territories();

    assert_eq!(territory_rows.source_count(), 2);
    let [short, long] = territory_rows.rows() else {
        panic!("territory projection lost its exact two-row fixture");
    };
    assert_eq!(short.territory_id(), &short_id);
    assert_eq!(long.territory_id(), &long_id);
    assert_eq!(
        short.ordered_fields(),
        &[
            (
                "heat".to_owned(),
                StableBslValueV1::RealBits(0.625_f64.to_bits()),
            ),
            ("population".to_owned(), StableBslValueV1::Int(23)),
            (
                "production-total".to_owned(),
                StableBslValueV1::RealBits(2.5_f64.to_bits()),
            ),
            (
                "territory-type".to_owned(),
                StableBslValueV1::Enum {
                    enum_type: "TerritoryType".to_owned(),
                    member: "PERIPHERY".to_owned(),
                },
            ),
            (
                "treasury".to_owned(),
                StableBslValueV1::CurrencyMicroUnits(11_000_000),
            ),
        ]
    );
    assert_eq!(
        long.ordered_fields(),
        &[
            (
                "heat".to_owned(),
                StableBslValueV1::RealBits(0.375_f64.to_bits()),
            ),
            ("population".to_owned(), StableBslValueV1::Int(13)),
            (
                "production-total".to_owned(),
                StableBslValueV1::RealBits(1.25_f64.to_bits()),
            ),
            (
                "territory-type".to_owned(),
                StableBslValueV1::Enum {
                    enum_type: "TerritoryType".to_owned(),
                    member: "CORE".to_owned(),
                },
            ),
            (
                "treasury".to_owned(),
                StableBslValueV1::CurrencyMicroUnits(7_000_000),
            ),
        ]
    );
    let frozen_alpha_hex = long
        .canonical_bytes()
        .iter()
        .fold(String::new(), |mut output, byte| {
            use std::fmt::Write as _;
            write!(output, "{byte:02x}").unwrap();
            output
        });
    assert_eq!(
        frozen_alpha_hex,
        "626162796c6f6e2e6d6174657269616c2d73746174652d726f770000000001020000003c626162796c6f6e2e737461626c652d656c656d656e740000000001010000001664656d6f2f7465727269746f72792d646572697665640000000261610000000c7465727269746f72795f69640000003c626162796c6f6e2e737461626c652d656c656d656e740000000001010000001664656d6f2f7465727269746f72792d646572697665640000000261610000000e6f7264657265645f6669656c647300000005000000046865617400000009033fd80000000000000000000a706f70756c6174696f6e0000000901000000000000000d0000001070726f64756374696f6e2d746f74616c00000009033ff40000000000000000000e7465727269746f72792d747970650000001a060000000d5465727269746f72795479706500000004434f52450000000874726561737572790000001102000000000000000000000000006acfc0"
    );
    assert_eq!(
        babylon_tick::hex(&sha256_of(long.canonical_bytes())),
        "04f10197f8919500872d9d9238450b7d549af03b21df504821c46fd589eb27af"
    );
}

#[test]
fn organization_rows_are_derived_from_the_post_tick_graph_and_presence_topology() {
    let organization_id = |local_name: &str| StableElementKeyV1::Node {
        scenario: "demo/organization-derived".to_owned(),
        local_name: local_name.to_owned(),
    };
    let territory_id = |local_name: &str| StableElementKeyV1::Node {
        scenario: "demo/organization-derived".to_owned(),
        local_name: local_name.to_owned(),
    };
    assert!(
        b"council".as_slice() < b"f".as_slice(),
        "stable graph organization order is lexical"
    );
    assert!(
        organization_id("f").canonical_bytes().unwrap()
            < organization_id("council").canonical_bytes().unwrap(),
        "length-framed organization primary-key order must put f before council"
    );
    assert!(
        b"aa".as_slice() < b"z".as_slice(),
        "stable graph territory order is lexical"
    );
    assert!(
        territory_id("z").canonical_bytes().unwrap()
            < territory_id("aa").canonical_bytes().unwrap(),
        "length-framed territory primary-key order must put z before aa"
    );

    let replay_id = ReplaySessionIdV1::try_from("per281/organization-derived").unwrap();
    let mut session = ReplayTickSession::new(
        DERIVED_ORGANIZATION_SCENARIO,
        None,
        DERIVED_ORGANIZATION_RULE,
        HypergraphStore::new(),
        replay_id.clone(),
        ReplaySeed::new(59),
        content_for(DERIVED_ORGANIZATION_RULE),
        foundation_reference(),
        foundation_material_state(),
    )
    .unwrap();
    let actions = OrderedPracticeActionBatchV1::empty(replay_id, 1).unwrap();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
    let identified = session.advance(&mut sink, &actions).unwrap();
    let organizations = identified.material_state_rows().organizations();

    assert_eq!(organizations.source_count(), 2);
    let [derived_political, derived_business] = organizations.rows() else {
        panic!("organization projection lost its exact two-row fixture");
    };
    assert_eq!(derived_political.organization_id(), &organization_id("f"));
    assert_eq!(
        derived_political.organization_kind(),
        &StableBslValueV1::Enum {
            enum_type: "OrgKind".to_owned(),
            member: "POLITICAL_FACTION".to_owned(),
        }
    );
    assert_eq!(
        derived_political.ordered_territory_ids(),
        &[territory_id("z"), territory_id("aa")]
    );
    assert_eq!(
        derived_political.ordered_fields(),
        &[
            ("members".to_owned(), StableBslValueV1::Int(15)),
            (
                "productivity".to_owned(),
                StableBslValueV1::RealBits(1.25_f64.to_bits()),
            ),
            (
                "status".to_owned(),
                StableBslValueV1::Enum {
                    enum_type: "OrgStatus".to_owned(),
                    member: "ACTIVE".to_owned(),
                },
            ),
            (
                "treasury".to_owned(),
                StableBslValueV1::CurrencyMicroUnits(7_000_000),
            ),
        ]
    );
    let frozen_hex =
        derived_political
            .canonical_bytes()
            .iter()
            .fold(String::new(), |mut output, byte| {
                use std::fmt::Write as _;
                write!(output, "{byte:02x}").unwrap();
                output
            });
    assert_eq!(
        frozen_hex,
        "626162796c6f6e2e6d6174657269616c2d73746174652d726f770000000001080000003e626162796c6f6e2e737461626c652d656c656d656e740000000001010000001964656d6f2f6f7267616e697a6174696f6e2d6465726976656400000001660000000f6f7267616e697a6174696f6e5f69640000003e626162796c6f6e2e737461626c652d656c656d656e740000000001010000001964656d6f2f6f7267616e697a6174696f6e2d646572697665640000000166000000116f7267616e697a6174696f6e5f6b696e640000002106000000074f72674b696e6400000011504f4c49544943414c5f46414354494f4e000000156f7264657265645f7465727269746f72795f696473000000020000003e626162796c6f6e2e737461626c652d656c656d656e740000000001010000001964656d6f2f6f7267616e697a6174696f6e2d64657269766564000000017a0000003f626162796c6f6e2e737461626c652d656c656d656e740000000001010000001964656d6f2f6f7267616e697a6174696f6e2d646572697665640000000261610000000e6f7264657265645f6669656c647300000004000000076d656d626572730000000901000000000000000f0000000c70726f64756374697669747900000009033ff4000000000000000000067374617475730000001806000000094f7267537461747573000000064143544956450000000874726561737572790000001102000000000000000000000000006acfc0"
    );
    assert_eq!(
        babylon_tick::hex(&sha256_of(derived_political.canonical_bytes())),
        "f0b0c41b248b13a0ed0e6f656b7dd9ed11036f9be0b73691961fb4dc3cc34708"
    );
    assert_eq!(
        derived_business.organization_id(),
        &organization_id("council")
    );
    assert_eq!(
        derived_business.organization_kind(),
        &StableBslValueV1::Enum {
            enum_type: "OrgKind".to_owned(),
            member: "BUSINESS".to_owned(),
        }
    );
    assert!(derived_business.ordered_territory_ids().is_empty());
    assert_eq!(
        derived_business.ordered_fields(),
        &[
            ("members".to_owned(), StableBslValueV1::Int(25)),
            (
                "productivity".to_owned(),
                StableBslValueV1::RealBits(2.5_f64.to_bits()),
            ),
            (
                "status".to_owned(),
                StableBslValueV1::Enum {
                    enum_type: "OrgStatus".to_owned(),
                    member: "DORMANT".to_owned(),
                },
            ),
            (
                "treasury".to_owned(),
                StableBslValueV1::CurrencyMicroUnits(11_000_000),
            ),
        ]
    );
}

#[test]
fn duplicate_retained_event_field_refuses_without_publication() {
    let replay_id = ReplaySessionIdV1::try_from("per281/duplicate-retained-field").unwrap();
    let mut session = ReplayTickSession::new(
        REPLAY_SCENARIO,
        None,
        DUPLICATE_RETAINED_OUTPUT_RULE,
        HypergraphStore::new(),
        replay_id.clone(),
        ReplaySeed::new(23),
        content_for(DUPLICATE_RETAINED_OUTPUT_RULE),
        foundation_reference(),
        foundation_material_state(),
    )
    .unwrap();
    let before = session.graph().encode_state().unwrap().as_bytes().to_vec();
    let before_cursors = session.graph().allocator_cursors();
    let before_completed_tick = session.completed_tick();
    let actions = OrderedPracticeActionBatchV1::empty(replay_id, 1).unwrap();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink {
        events: vec![("EventType/PRIOR".to_owned(), Vec::new())],
    };
    let before_events = sink.events.clone();

    assert_eq!(
        session.advance(&mut sink, &actions).unwrap_err(),
        ReplayTickError::DuplicateSuccessfulEventField {
            event_type: "REPLAY_DUPLICATE".to_owned(),
            field: "subject".to_owned(),
        }
    );
    assert_eq!(session.completed_tick(), before_completed_tick);
    assert_eq!(sink.events, before_events);
    assert_eq!(session.graph().encode_state().unwrap().as_bytes(), before);
    assert_eq!(session.graph().allocator_cursors(), before_cursors);
}

#[test]
fn replay_identity_agrees_across_substrates_and_seed_moves_finite_choices() {
    let memory = run_replay::<MemoryGraph>(REPLAY_VECTOR_SEED);
    let hypergraph = run_replay::<HypergraphStore>(REPLAY_VECTOR_SEED);
    assert_eq!(memory, hypergraph);
    assert_eq!(hypergraph.selected_outcomes, ["LOW", "LOW"]);
    assert_eq!(
        hypergraph.draw_tickets,
        [6_210_590_038_642_615_346, 4_297_492_488_788_804_740]
    );

    let changed_seed = run_replay::<HypergraphStore>(CHANGED_REPLAY_VECTOR_SEED);
    assert_eq!(changed_seed.selected_outcomes, ["HIGH", "LOW"]);
    assert_eq!(
        changed_seed.draw_tickets,
        [12_032_564_276_218_344_752, 2_494_123_929_480_873_784]
    );
    assert_eq!(hypergraph.prepared, changed_seed.prepared);
    assert_eq!(hypergraph.prior, changed_seed.prior);
    assert_eq!(hypergraph.actions, changed_seed.actions);
    assert_ne!(
        hypergraph.choice_receipt_digest,
        changed_seed.choice_receipt_digest
    );
    assert_ne!(hypergraph.selected_outcomes, changed_seed.selected_outcomes);
    assert_ne!(hypergraph.draw_tickets, changed_seed.draw_tickets);
    assert_ne!(hypergraph.result_bits, changed_seed.result_bits);
    assert_ne!(hypergraph.payload, changed_seed.payload);
    assert_ne!(hypergraph.result, changed_seed.result);
    assert_ne!(hypergraph.outer, changed_seed.outer);
}

#[test]
fn replay_identity_is_exact_across_fresh_processes() {
    if std::env::var_os(PROCESS_CHILD_ENV).is_some() {
        println!(
            "{PROCESS_MARKER}{:?}",
            run_replay::<HypergraphStore>(REPLAY_VECTOR_SEED)
        );
        return;
    }
    let executable = std::env::current_exe().unwrap();
    let run_child = || {
        let output = Command::new(&executable)
            .args([
                "--exact",
                "replay_identity_is_exact_across_fresh_processes",
                "--nocapture",
            ])
            .env(PROCESS_CHILD_ENV, "1")
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .find_map(|line| line.strip_prefix(PROCESS_MARKER).map(str::to_owned))
            .unwrap()
    };
    assert_eq!(run_child(), run_child());
}

#[test]
fn replay_action_guards_and_rules_hash_refuse_without_publication() {
    let replay_id = ReplaySessionIdV1::try_from("per60/guarded").unwrap();
    let mut bad_content = content_for(REPLAY_RULE);
    bad_content.rules_hash = [0xff; 32];
    assert!(matches!(
        ReplayTickSession::new(
            REPLAY_SCENARIO,
            None,
            REPLAY_RULE,
            MemoryGraph::new(),
            replay_id.clone(),
            ReplaySeed::new(9),
            bad_content,
            foundation_reference(),
            foundation_material_state(),
        ),
        Err(ReplayTickError::Identity(
            ReplayTickIdentityError::RulesHashMismatch { .. }
        ))
    ));

    let mut session = ReplayTickSession::new(
        REPLAY_SCENARIO,
        None,
        REPLAY_RULE,
        MemoryGraph::new(),
        replay_id.clone(),
        ReplaySeed::new(9),
        content_for(REPLAY_RULE),
        foundation_reference(),
        foundation_material_state(),
    )
    .unwrap();
    let before = session.graph().encode_state().unwrap().as_bytes().to_vec();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();

    let structural = nonempty_action_batch(replay_id.clone());
    assert!(matches!(
        session.advance(&mut sink, &structural),
        Err(ReplayTickError::NonEmptyActionBatch { count: 1 })
    ));

    let other_session = ReplaySessionIdV1::try_from("per60/other").unwrap();
    let wrong_session = OrderedPracticeActionBatchV1::empty(other_session, 1).unwrap();
    assert!(matches!(
        session.advance(&mut sink, &wrong_session),
        Err(ReplayTickError::ActionSessionMismatch)
    ));
    let wrong_tick = OrderedPracticeActionBatchV1::empty(replay_id, 2).unwrap();
    assert!(matches!(
        session.advance(&mut sink, &wrong_tick),
        Err(ReplayTickError::ActionTickMismatch {
            expected: 1,
            actual: 2,
        })
    ));
    assert_eq!(session.completed_tick(), 0);
    assert!(sink.events.is_empty());
    assert_eq!(session.graph().encode_state().unwrap().as_bytes(), before);
}

#[test]
fn replay_rule_failure_discards_detached_writes_events_and_identity() {
    let replay_id = ReplaySessionIdV1::try_from("per60/rule-failure").unwrap();
    let mut session = ReplayTickSession::new(
        FAILURE_SCENARIO,
        None,
        FAILURE_RULE,
        MemoryGraph::new(),
        replay_id.clone(),
        ReplaySeed::new(13),
        content_for(FAILURE_RULE),
        foundation_reference(),
        foundation_material_state(),
    )
    .unwrap();
    let before = session.graph().encode_state().unwrap().as_bytes().to_vec();
    let actions = OrderedPracticeActionBatchV1::empty(replay_id, 1).unwrap();
    let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();

    assert!(matches!(
        session.advance(&mut sink, &actions),
        Err(ReplayTickError::Execution { .. })
    ));
    assert_eq!(session.completed_tick(), 0);
    assert!(sink.events.is_empty());
    assert_eq!(session.graph().encode_state().unwrap().as_bytes(), before);
}

#[test]
fn register_manifest_set_and_stable_world_bytes_are_exact() {
    let manifest = world_register_manifest_v1().unwrap();
    assert_eq!(
        manifest.canonical_bytes(),
        [
            b"babylon.world-register-manifest\0".as_slice(),
            &WORLD_REGISTER_MANIFEST_LAYOUT_VERSION_V1.to_be_bytes(),
            &1_u32.to_be_bytes(),
            &str32("world/completed-tick"),
            &1_u32.to_be_bytes(),
        ]
        .concat()
    );
    let registers = encode_world_register_set_v1(&manifest, 0).unwrap();
    assert_eq!(
        registers.canonical_bytes(),
        [
            b"babylon.world-register-set\0".as_slice(),
            &WORLD_REGISTER_SET_LAYOUT_VERSION_V1.to_be_bytes(),
            &[0x01],
            &WORLD_REGISTER_MANIFEST_LAYOUT_VERSION_V1.to_be_bytes(),
            manifest.digest().as_slice(),
            &[0x02],
            &1_u32.to_be_bytes(),
            &str32("world/completed-tick"),
            &1_u32.to_be_bytes(),
            &8_u32.to_be_bytes(),
            &0_i64.to_be_bytes(),
        ]
        .concat()
    );

    let mut graph = MemoryGraph::new();
    let node = graph.add_node("class").unwrap();
    let resolver = StableElementResolverV1::seal(
        &graph,
        "demo/world",
        &HashMap::from([(node, "workers".to_owned())]),
        &HashMap::new(),
    )
    .unwrap();
    let stable_graph = encode_stable_graph_state_v1(&graph, &resolver).unwrap();
    let world = encode_stable_world_v1(&stable_graph, &registers).unwrap();
    assert_eq!(
        world.canonical_bytes(),
        [
            b"babylon.stable-world\0".as_slice(),
            &STABLE_WORLD_LAYOUT_VERSION_V1.to_be_bytes(),
            &[0x01],
            &1_u32.to_be_bytes(),
            stable_graph.digest().as_bytes(),
            &[0x02],
            &WORLD_REGISTER_SET_LAYOUT_VERSION_V1.to_be_bytes(),
            registers.digest().as_slice(),
        ]
        .concat()
    );
}

#[test]
fn register_tick_domain_is_checked() {
    let manifest = world_register_manifest_v1().unwrap();
    assert!(encode_world_register_set_v1(&manifest, i64::MAX).is_ok());
    assert_eq!(
        encode_world_register_set_v1(&manifest, -1),
        Err(ReplayTickIdentityError::NegativeCompletedTick { value: -1 })
    );
}

#[test]
fn tick_payload_v2_is_exact_and_order_sensitive_without_reencoding_fired() {
    let mut graph = MemoryGraph::new();
    let node = graph.add_node("class").unwrap();
    let resolver = StableElementResolverV1::seal(
        &graph,
        "demo/world",
        &HashMap::from([(node, "workers".to_owned())]),
        &HashMap::new(),
    )
    .unwrap();
    let order = vec!["demo/a".to_owned(), "demo/b".to_owned()];
    let outcomes = vec![("demo/a".to_owned(), 1), ("demo/b".to_owned(), 2)];
    let events = vec![
        CommittedEventV2::new(
            "demo/a".to_owned(),
            None,
            "FIRST".to_owned(),
            vec![
                ("value".to_owned(), Value::Int(1)),
                ("value".to_owned(), Value::Int(2)),
            ],
        ),
        CommittedEventV2::new(
            "demo/b".to_owned(),
            None,
            "SECOND".to_owned(),
            vec![("value".to_owned(), Value::Int(3))],
        ),
    ];
    let receipts = vec![
        AuditReceipt {
            rule_id: "demo/a".to_owned(),
            role: RuleRole::Mechanic,
            evidence: EvidenceClass::Derived,
            ordinal: 0,
            effect: EffectSignature::NodeField("class/value".to_owned()),
        },
        AuditReceipt {
            rule_id: "demo/b".to_owned(),
            role: RuleRole::Mechanic,
            evidence: EvidenceClass::Derived,
            ordinal: 1,
            effect: EffectSignature::Event("SECOND".to_owned()),
        },
    ];
    let payload =
        encode_tick_payload_v2(&order, &outcomes, 3, &events, &[], &receipts, &resolver).unwrap();
    assert!(payload.canonical_bytes().starts_with(
        &[
            b"babylon.tick-payload.v2\0".as_slice(),
            &TICK_PAYLOAD_LAYOUT_VERSION_V2.to_be_bytes(),
            &[0x01],
        ]
        .concat()
    ));
    assert_eq!(payload.canonical_bytes().last(), Some(&0));
    let reversed = encode_tick_payload_v2(
        &order,
        &outcomes,
        3,
        &events.iter().cloned().rev().collect::<Vec<_>>(),
        &[],
        &receipts,
        &resolver,
    )
    .unwrap();
    assert_ne!(payload.digest(), reversed.digest());
    let reversed_pairs = vec![
        ("value".to_owned(), Value::Int(2)),
        ("value".to_owned(), Value::Int(1)),
    ];
    let pair_reordered_events = vec![
        CommittedEventV2::new(
            "demo/a".to_owned(),
            None,
            "FIRST".to_owned(),
            reversed_pairs,
        ),
        events[1].clone(),
    ];
    let pair_reordered = encode_tick_payload_v2(
        &order,
        &outcomes,
        3,
        &pair_reordered_events,
        &[],
        &receipts,
        &resolver,
    )
    .unwrap();
    assert_ne!(payload.digest(), pair_reordered.digest());
    let provenance_changed_events = vec![
        CommittedEventV2::new(
            "demo/b".to_owned(),
            None,
            "FIRST".to_owned(),
            vec![
                ("value".to_owned(), Value::Int(1)),
                ("value".to_owned(), Value::Int(2)),
            ],
        ),
        events[1].clone(),
    ];
    let provenance_changed = encode_tick_payload_v2(
        &order,
        &outcomes,
        3,
        &provenance_changed_events,
        &[],
        &receipts,
        &resolver,
    )
    .unwrap();
    assert_ne!(payload.digest(), provenance_changed.digest());
    let receipt_reordered = encode_tick_payload_v2(
        &order,
        &outcomes,
        3,
        &events,
        &[],
        &receipts.iter().cloned().rev().collect::<Vec<_>>(),
        &resolver,
    )
    .unwrap();
    assert_ne!(payload.digest(), receipt_reordered.digest());
    assert!(matches!(
        encode_tick_payload_v2(&order, &outcomes, 4, &events, &[], &receipts, &resolver),
        Err(ReplayTickIdentityError::FiredTotalMismatch { .. })
    ));
    assert!(matches!(
        encode_tick_payload_v2(
            &["demo/b".to_owned(), "demo/a".to_owned()],
            &outcomes,
            3,
            &events,
            &[],
            &receipts,
            &resolver,
        ),
        Err(ReplayTickIdentityError::RuleOutcomeOrder)
    ));
}
