use babylon_kernel::replay::ReplaySessionIdV1;
use babylon_practice_contract::actor_v2::ActorOrganizationIdV2;
use babylon_practice_contract::ordered_action_v1::{
    encode_practice_action_id_preimage_v1, practice_action_id_v1, OrderedPracticeActionBatchV1,
    OrderedPracticeActionError,
};
use babylon_practice_contract::{
    input_authority_ledger_v2_digest, CampaignIdV2, InputAuthorityIdV2, PracticeAuthorityKindV2,
    PracticeBatchV2Error, PracticeIdV2, PracticeInputAuthorityLedgerV2, PracticeInputAuthorityV2,
    PracticeIntentV2, PracticeIntentV2Error, PracticeParameterV2, PracticeTargetIdentityV2,
    PracticeTargetTagV2, ProposalNonceV2, ResolvedPracticeBatchItemV2, ResolvedPracticeBatchV2,
    ResolvedPracticeBatchV2Error, TaggedPracticeTargetV2, MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2,
};

fn actor_id(value: u64) -> ActorOrganizationIdV2 {
    ActorOrganizationIdV2::from_bytes(value.to_be_bytes())
}

fn session(value: &str) -> ReplaySessionIdV1 {
    ReplaySessionIdV1::try_from(value).expect("session fixture is valid")
}

fn hex_bytes(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let text = std::str::from_utf8(chunk).expect("hex fixture is ASCII");
            u8::from_str_radix(text, 16).expect("hex fixture is valid")
        })
        .collect()
}

fn hex_digest(value: &str) -> [u8; 32] {
    hex_bytes(value)
        .try_into()
        .expect("digest fixture is 32 bytes")
}

fn authority() -> PracticeInputAuthorityV2 {
    PracticeInputAuthorityV2 {
        schema_version: 2,
        campaign_id: CampaignIdV2::from_bytes([0x10; 16]),
        authority_kind: PracticeAuthorityKindV2::PlayerSeat,
        input_authority_id: InputAuthorityIdV2::from_bytes([0x20; 16]),
        actor_org_id: actor_id(7),
        effective_from_tick: 10,
        effective_through_tick_exclusive: 20,
        decision_content_digest: [0x30; 32],
    }
}

fn ledger() -> PracticeInputAuthorityLedgerV2 {
    PracticeInputAuthorityLedgerV2 {
        schema_version: 2,
        rows: vec![authority()],
    }
}

fn intent(proposal_marker: u8) -> PracticeIntentV2 {
    PracticeIntentV2 {
        schema_version: 2,
        submit_after_tick: 10,
        resolve_tick: 11,
        input_authority_id: InputAuthorityIdV2::from_bytes([0x20; 16]),
        actor_org_id: actor_id(7),
        practice_id: PracticeIdV2::Strike,
        target: TaggedPracticeTargetV2 {
            tag: PracticeTargetTagV2::LaborProcess,
            identity: PracticeTargetIdentityV2::from_bytes([0x50; 32]),
        },
        proposal_nonce: ProposalNonceV2::from_bytes([proposal_marker; 16]),
        quoted_content_digest: [0x30; 32],
        quoted_resource_contract_digest: [0x40; 32],
        parameters: Vec::new(),
        evidence_digests: vec![[0x70; 32], [0x80; 32]],
    }
}

fn item(proposal_marker: u8) -> ResolvedPracticeBatchItemV2 {
    ResolvedPracticeBatchItemV2 {
        authority: authority(),
        intent: intent(proposal_marker),
    }
}

fn batch(items: Vec<ResolvedPracticeBatchItemV2>) -> ResolvedPracticeBatchV2 {
    ResolvedPracticeBatchV2 {
        schema_version: 2,
        campaign_id: CampaignIdV2::from_bytes([0x10; 16]),
        resolve_tick: 11,
        authority_ledger_digest: input_authority_ledger_v2_digest(&ledger()).unwrap(),
        resource_allocation_contract_digest: [0x40; 32],
        content_digest: [0x30; 32],
        items,
    }
}

#[test]
fn action_id_preimage_is_exact_and_excludes_the_ordinal() {
    let replay = session("replay-A");
    let value = intent(0x60);
    let expected = hex_bytes(concat!(
        "626162796c6f6e2e70726163746963652d616374696f6e2d69642e7631",
        "00",
        "0001",
        "0008",
        "7265706c61792d41",
        "0002",
        "0537e9106faaa91dd9b54e3dc68fcb6a45e7154d9086a835085ce85e9479f80b",
    ));

    assert_eq!(expected.len(), 76);
    assert_eq!(
        encode_practice_action_id_preimage_v1(&replay, &value).unwrap(),
        expected
    );
    assert_eq!(
        practice_action_id_v1(&replay, &value).unwrap().as_bytes(),
        &hex_digest("955c45b85e469ccdfcbee405d6899fc00d0c6bb882daa32e6f2bfa072fe181a6")
    );
    assert_eq!(
        encode_practice_action_id_preimage_v1(&session("!"), &value)
            .unwrap()
            .len(),
        69
    );
    let maximum_session = "Z".repeat(256);
    assert_eq!(
        encode_practice_action_id_preimage_v1(&session(&maximum_session), &value)
            .unwrap()
            .len(),
        324
    );

    let original_only = OrderedPracticeActionBatchV1::project(
        session("replay-A"),
        &batch(vec![item(0x70)]),
        &ledger(),
    )
    .unwrap();
    let lower_inserted = OrderedPracticeActionBatchV1::project(
        session("replay-A"),
        &batch(vec![item(0x60), item(0x70)]),
        &ledger(),
    )
    .unwrap();
    assert_eq!(original_only.items()[0].canonical_input_ordinal(), 0);
    assert_eq!(lower_inserted.items()[0].canonical_input_ordinal(), 0);
    assert_eq!(lower_inserted.items()[1].canonical_input_ordinal(), 1);
    assert_eq!(
        original_only.items()[0].action_id(),
        lower_inserted.items()[1].action_id()
    );
}

#[test]
fn exact_empty_batch_binds_session_and_tick() {
    let value = OrderedPracticeActionBatchV1::empty(session("replay-A"), 11).unwrap();
    let expected = hex_bytes(concat!(
        "626162796c6f6e2e6f7264657265642d70726163746963652d616374696f6e2d62617463682e7631",
        "00",
        "0001",
        "0008",
        "7265706c61792d41",
        "000000000000000b",
        "0000",
    ));

    assert_eq!(expected.len(), 63);
    assert_eq!(value.canonical_bytes(), expected);
    assert_eq!(
        value.digest().as_bytes(),
        &hex_digest("736efc660f813604c9b35654958a7b43f0cdf39f23782359fa1ccbd326f87551")
    );
    assert_eq!(value.session().as_bytes(), b"replay-A");
    assert_eq!(value.resolve_tick(), 11);
    assert!(value.items().is_empty());
    assert!(value.is_empty());

    let other_session = OrderedPracticeActionBatchV1::empty(session("replay-B"), 11).unwrap();
    let other_tick = OrderedPracticeActionBatchV1::empty(session("replay-A"), 12).unwrap();
    assert_ne!(value.digest(), other_session.digest());
    assert_ne!(value.digest(), other_tick.digest());
}

#[test]
fn projector_assigns_contiguous_ordinals_and_exact_canonical_bytes() {
    let source = batch(vec![item(0x60)]);
    let value =
        OrderedPracticeActionBatchV1::project(session("replay-A"), &source, &ledger()).unwrap();
    let expected = hex_bytes(concat!(
        "626162796c6f6e2e6f7264657265642d70726163746963652d616374696f6e2d62617463682e7631",
        "00",
        "0001",
        "0008",
        "7265706c61792d41",
        "000000000000000b",
        "0001",
        "0000",
        "955c45b85e469ccdfcbee405d6899fc00d0c6bb882daa32e6f2bfa072fe181a6",
        "00fb",
        "626162796c6f6e2e70726163746963652d696e74656e742e7632",
        "00",
        "0002",
        "000000000000000a",
        "000000000000000b",
        "20202020202020202020202020202020",
        "0000000000000007",
        "04",
        "02",
        "5050505050505050505050505050505050505050505050505050505050505050",
        "60606060606060606060606060606060",
        "3030303030303030303030303030303030303030303030303030303030303030",
        "4040404040404040404040404040404040404040404040404040404040404040",
        "0000",
        "0002",
        "7070707070707070707070707070707070707070707070707070707070707070",
        "8080808080808080808080808080808080808080808080808080808080808080",
    ));

    assert_eq!(value.items().len(), 1);
    assert_eq!(value.items()[0].canonical_input_ordinal(), 0);
    assert_eq!(value.items()[0].intent(), &source.items[0].intent);
    assert_eq!(value.canonical_bytes(), expected);
    assert_eq!(expected.len(), 350);
    assert_eq!(
        value.digest().as_bytes(),
        &hex_digest("7f6573a522a394608de09bac62aa92beccab6d7a73e08462247d1ed8fe2ff41d")
    );
}

#[test]
fn projector_refuses_every_untrusted_source_shape_before_projection() {
    let trusted = ledger();
    let valid = batch(vec![item(0x60)]);
    let mut wrong_ledger = trusted.clone();
    wrong_ledger.rows[0].decision_content_digest = [0x31; 32];
    assert_eq!(
        OrderedPracticeActionBatchV1::project(session("replay-A"), &valid, &wrong_ledger),
        Err(OrderedPracticeActionError::Source(
            ResolvedPracticeBatchV2Error::Batch(PracticeBatchV2Error::BatchLedgerDigest)
        ))
    );

    let unordered = batch(vec![item(0x70), item(0x60)]);
    assert_eq!(
        OrderedPracticeActionBatchV1::project(session("replay-A"), &unordered, &trusted),
        Err(OrderedPracticeActionError::Source(
            ResolvedPracticeBatchV2Error::Batch(PracticeBatchV2Error::BatchItemOrder)
        ))
    );
    let duplicate = batch(vec![item(0x60), item(0x60)]);
    assert_eq!(
        OrderedPracticeActionBatchV1::project(session("replay-A"), &duplicate, &trusted),
        Err(OrderedPracticeActionError::Source(
            ResolvedPracticeBatchV2Error::Batch(PracticeBatchV2Error::BatchItemDuplicate)
        ))
    );

    let mut wrong_tick = valid.clone();
    wrong_tick.resolve_tick = 12;
    assert_eq!(
        OrderedPracticeActionBatchV1::project(session("replay-A"), &wrong_tick, &trusted),
        Err(OrderedPracticeActionError::Source(
            ResolvedPracticeBatchV2Error::Batch(PracticeBatchV2Error::BatchResolveTick)
        ))
    );

    let mut oversized_intent = valid.clone();
    oversized_intent.items[0]
        .intent
        .parameters
        .push(PracticeParameterV2 {
            key_u8: 1,
            value_kind_u8: 1,
            value_length_u16: 257,
            value_bytes: vec![0; 257],
        });
    assert_eq!(
        OrderedPracticeActionBatchV1::project(session("replay-A"), &oversized_intent, &trusted),
        Err(OrderedPracticeActionError::Source(
            ResolvedPracticeBatchV2Error::Intent(PracticeIntentV2Error::IntentParameterLength)
        ))
    );

    let too_many = batch(vec![item(0x60); MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2 + 1]);
    assert_eq!(
        OrderedPracticeActionBatchV1::project(session("replay-A"), &too_many, &trusted),
        Err(OrderedPracticeActionError::Source(
            ResolvedPracticeBatchV2Error::Batch(PracticeBatchV2Error::BatchItemLimit)
        ))
    );
}
