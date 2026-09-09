use babylon_practice_contract::actor_v2::ActorOrganizationIdV2;
use babylon_practice_contract::{
    decode_resolved_practice_batch_v2, encode_practice_intent_v2,
    encode_resolved_practice_batch_v2, input_authority_ledger_v2_digest,
    resolved_practice_batch_v2_digest, validate_resolved_practice_batch_v2, CampaignIdV2,
    InputAuthorityIdV2, PracticeAuthorityKindV2, PracticeAuthorityV2Error, PracticeBatchV2Error,
    PracticeIdV2, PracticeInputAuthorityLedgerV2, PracticeInputAuthorityV2, PracticeIntentV2,
    PracticeIntentV2Error, PracticeTargetIdentityV2, PracticeTargetTagV2, ProposalNonceV2,
    ResolvedPracticeBatchItemV2, ResolvedPracticeBatchV2, ResolvedPracticeBatchV2Error,
    TaggedPracticeTargetV2, MAX_RESOLVED_PRACTICE_BATCH_CANONICAL_BYTES_V2,
    MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2, MIN_PRACTICE_INTENT_CANONICAL_BYTES_V2,
    RESOLVED_PRACTICE_BATCH_V2_DOMAIN_BYTES,
};

fn actor_id(value: u64) -> ActorOrganizationIdV2 {
    ActorOrganizationIdV2::from_bytes(value.to_be_bytes())
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
        authority_ledger_digest: hex_digest(
            "3415c8298f3a78e53fe3660ac453544b43f8be32dc12071928bb2b8c3782908a",
        ),
        resource_allocation_contract_digest: [0x40; 32],
        content_digest: [0x30; 32],
        items,
    }
}

#[test]
fn resolved_batch_v2_round_trips_independent_literal_bytes() {
    let expected = hex_bytes(concat!(
        "626162796c6f6e2e7265736f6c7665642d70726163746963652d62617463682e7632",
        "00",
        "0002",
        "10101010101010101010101010101010",
        "000000000000000b",
        "3415c8298f3a78e53fe3660ac453544b43f8be32dc12071928bb2b8c3782908a",
        "4040404040404040404040404040404040404040404040404040404040404040",
        "3030303030303030303030303030303030303030303030303030303030303030",
        "0001",
        "007f",
        "626162796c6f6e2e70726163746963652d696e7075742d617574686f726974792e7632",
        "00",
        "0002",
        "10101010101010101010101010101010",
        "01",
        "20202020202020202020202020202020",
        "0000000000000007",
        "000000000000000a",
        "0000000000000014",
        "3030303030303030303030303030303030303030303030303030303030303030",
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
    let value = batch(vec![item(0x60)]);

    assert_eq!(expected.len(), 541);
    assert_eq!(
        encode_resolved_practice_batch_v2(&value, &ledger()).unwrap(),
        expected
    );
    assert_eq!(
        decode_resolved_practice_batch_v2(&expected, &ledger()).unwrap(),
        value
    );
    assert_eq!(
        resolved_practice_batch_v2_digest(&value, &ledger()).unwrap(),
        hex_digest("04292b2d21ec9512d0e5c7c7d184595c1f32904475007736a41f6dfba1f28594")
    );
}

#[test]
fn resolved_batch_v2_binds_ledger_campaign_and_exact_authority_row() {
    let authoritative = ledger();
    let mut wrong_digest = batch(vec![item(0x60)]);
    wrong_digest.authority_ledger_digest = [0x99; 32];
    assert_eq!(
        validate_resolved_practice_batch_v2(&wrong_digest, &authoritative),
        Err(ResolvedPracticeBatchV2Error::Batch(
            PracticeBatchV2Error::BatchLedgerDigest
        ))
    );

    let mut wrong_campaign = batch(vec![item(0x60)]);
    wrong_campaign.campaign_id = CampaignIdV2::from_bytes([0x11; 16]);
    assert_eq!(
        validate_resolved_practice_batch_v2(&wrong_campaign, &authoritative),
        Err(ResolvedPracticeBatchV2Error::Batch(
            PracticeBatchV2Error::BatchCampaign
        ))
    );

    let mut altered = batch(vec![item(0x60)]);
    altered.items[0].authority.decision_content_digest = [0x31; 32];
    assert_eq!(
        validate_resolved_practice_batch_v2(&altered, &authoritative),
        Err(ResolvedPracticeBatchV2Error::Batch(
            PracticeBatchV2Error::BatchAuthorityMismatch
        ))
    );
}

#[test]
fn resolved_batch_v2_preserves_missing_inactive_and_actor_authority_refusals() {
    let mut empty_ledger = PracticeInputAuthorityLedgerV2 {
        schema_version: 2,
        rows: Vec::new(),
    };
    let mut missing = batch(vec![item(0x60)]);
    missing.authority_ledger_digest = input_authority_ledger_v2_digest(&empty_ledger).unwrap();
    assert_eq!(
        validate_resolved_practice_batch_v2(&missing, &empty_ledger),
        Err(ResolvedPracticeBatchV2Error::Authority(
            PracticeAuthorityV2Error::AuthorityNotFound
        ))
    );

    empty_ledger.rows.push(PracticeInputAuthorityV2 {
        effective_through_tick_exclusive: 11,
        ..authority()
    });
    let mut inactive = batch(vec![item(0x60)]);
    inactive.authority_ledger_digest = input_authority_ledger_v2_digest(&empty_ledger).unwrap();
    assert_eq!(
        validate_resolved_practice_batch_v2(&inactive, &empty_ledger),
        Err(ResolvedPracticeBatchV2Error::Authority(
            PracticeAuthorityV2Error::AuthorityInactive
        ))
    );

    let mut wrong_actor = batch(vec![item(0x60)]);
    wrong_actor.items[0].intent.actor_org_id = actor_id(8);
    assert_eq!(
        validate_resolved_practice_batch_v2(&wrong_actor, &ledger()),
        Err(ResolvedPracticeBatchV2Error::Authority(
            PracticeAuthorityV2Error::AuthorityActorMismatch
        ))
    );
}

#[test]
fn resolved_batch_v2_binds_tick_content_resource_and_nested_intent_errors() {
    for (value, expected) in [
        {
            let mut value = batch(vec![item(0x60)]);
            value.resolve_tick = 12;
            (value, PracticeBatchV2Error::BatchResolveTick)
        },
        {
            let mut value = batch(vec![item(0x60)]);
            value.content_digest = [0x31; 32];
            (value, PracticeBatchV2Error::BatchContentDigest)
        },
        {
            let mut value = batch(vec![item(0x60)]);
            value.resource_allocation_contract_digest = [0x41; 32];
            (value, PracticeBatchV2Error::BatchResourceContractDigest)
        },
    ] {
        assert_eq!(
            validate_resolved_practice_batch_v2(&value, &ledger()),
            Err(ResolvedPracticeBatchV2Error::Batch(expected))
        );
    }

    let mut malformed = batch(vec![item(0x60)]);
    malformed.items[0].intent.target.tag = PracticeTargetTagV2::SocialClass;
    assert_eq!(
        validate_resolved_practice_batch_v2(&malformed, &ledger()),
        Err(ResolvedPracticeBatchV2Error::Intent(
            PracticeIntentV2Error::IntentTargetMismatch
        ))
    );
}

#[test]
fn resolved_batch_v2_requires_ascending_unique_complete_proposal_keys() {
    let ordered = batch(vec![item(0x60), item(0x61)]);
    assert_eq!(
        validate_resolved_practice_batch_v2(&ordered, &ledger()),
        Ok(())
    );

    let reversed = batch(vec![item(0x61), item(0x60)]);
    assert_eq!(
        validate_resolved_practice_batch_v2(&reversed, &ledger()),
        Err(ResolvedPracticeBatchV2Error::Batch(
            PracticeBatchV2Error::BatchItemOrder
        ))
    );

    let mut distinct_bytes_same_key = item(0x60);
    distinct_bytes_same_key.intent.evidence_digests = vec![[0x70; 32], [0x81; 32]];
    let duplicate = batch(vec![item(0x60), distinct_bytes_same_key]);
    assert_eq!(
        validate_resolved_practice_batch_v2(&duplicate, &ledger()),
        Err(ResolvedPracticeBatchV2Error::Batch(
            PracticeBatchV2Error::BatchItemDuplicate
        ))
    );
}

#[test]
fn resolved_batch_v2_refuses_maximum_plus_one_before_nested_work() {
    let too_many = batch(vec![item(0x60); MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2 + 1]);
    assert_eq!(
        validate_resolved_practice_batch_v2(&too_many, &ledger()),
        Err(ResolvedPracticeBatchV2Error::Batch(
            PracticeBatchV2Error::BatchItemLimit
        ))
    );

    let oversized = vec![0_u8; MAX_RESOLVED_PRACTICE_BATCH_CANONICAL_BYTES_V2 + 1];
    assert_eq!(
        decode_resolved_practice_batch_v2(&oversized, &ledger()),
        Err(ResolvedPracticeBatchV2Error::Batch(
            PracticeBatchV2Error::BatchLength
        ))
    );
}

#[test]
fn resolved_batch_v2_refuses_bad_ledger_before_nested_decode() {
    let authoritative = ledger();
    let mut payload =
        encode_resolved_practice_batch_v2(&batch(vec![item(0x60)]), &authoritative).unwrap();
    let ledger_digest_offset = RESOLVED_PRACTICE_BATCH_V2_DOMAIN_BYTES.len() + 1 + 2 + 16 + 8;
    let header_length = ledger_digest_offset + 32 + 32 + 32 + 2;
    payload[ledger_digest_offset] ^= 0x01;
    payload.truncate(header_length);

    assert_eq!(
        decode_resolved_practice_batch_v2(&payload, &authoritative),
        Err(ResolvedPracticeBatchV2Error::Batch(
            PracticeBatchV2Error::BatchLedgerDigest
        ))
    );
}

#[test]
fn resolved_batch_v2_minimum_nested_intent_length_is_exact() {
    let mut minimum = intent(0x60);
    minimum.evidence_digests.clear();

    assert_eq!(
        encode_practice_intent_v2(&minimum).unwrap().len(),
        MIN_PRACTICE_INTENT_CANONICAL_BYTES_V2
    );
}

#[test]
fn resolved_batch_v2_error_table_is_exact() {
    let errors = [
        (PracticeBatchV2Error::BatchDomain, 1_u16),
        (PracticeBatchV2Error::BatchSchemaVersion, 2),
        (PracticeBatchV2Error::BatchTruncated, 3),
        (PracticeBatchV2Error::BatchTrailingBytes, 4),
        (PracticeBatchV2Error::BatchLength, 5),
        (PracticeBatchV2Error::BatchItemLimit, 6),
        (PracticeBatchV2Error::BatchItemLength, 7),
        (PracticeBatchV2Error::BatchItemOrder, 8),
        (PracticeBatchV2Error::BatchItemDuplicate, 9),
        (PracticeBatchV2Error::BatchResolveTick, 10),
        (PracticeBatchV2Error::BatchLedgerDigest, 11),
        (PracticeBatchV2Error::BatchCampaign, 12),
        (PracticeBatchV2Error::BatchAuthorityMismatch, 13),
        (PracticeBatchV2Error::BatchContentDigest, 14),
        (PracticeBatchV2Error::BatchResourceContractDigest, 15),
    ];
    for (error, code) in errors {
        assert_eq!(u16::from(error), code);
        assert_eq!(PracticeBatchV2Error::try_from(code), Ok(error));
    }
    assert!(PracticeBatchV2Error::try_from(0_u16).is_err());
    assert!(PracticeBatchV2Error::try_from(16_u16).is_err());
}

#[test]
fn resolved_batch_v2_digest_binds_top_level_identity_and_items() {
    let authoritative = ledger();
    let empty = batch(Vec::new());
    let expected = resolved_practice_batch_v2_digest(&empty, &authoritative).unwrap();
    let mut variants = Vec::with_capacity(5);

    let mut campaign = empty.clone();
    campaign.campaign_id = CampaignIdV2::from_bytes([0x11; 16]);
    variants.push(campaign);
    let mut tick = empty.clone();
    tick.resolve_tick = 12;
    variants.push(tick);
    let mut resource = empty.clone();
    resource.resource_allocation_contract_digest = [0x41; 32];
    variants.push(resource);
    let mut content = empty;
    content.content_digest = [0x31; 32];
    variants.push(content);
    variants.push(batch(vec![item(0x60)]));

    assert_eq!(variants.len(), 5);
    for variant in variants.iter().take(5) {
        assert_ne!(
            resolved_practice_batch_v2_digest(variant, &authoritative).unwrap(),
            expected
        );
    }
}
