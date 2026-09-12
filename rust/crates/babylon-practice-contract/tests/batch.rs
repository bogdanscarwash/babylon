use babylon_practice_contract::ActorOrganizationId;
use babylon_practice_contract::{
    decode_resolved_practice_batch, encode_practice_intent, encode_resolved_practice_batch,
    input_authority_ledger_digest, resolved_practice_batch_digest,
    validate_resolved_practice_batch, CampaignId, InputAuthorityId, PracticeAuthorityError,
    PracticeAuthorityKind, PracticeBatchError, PracticeId, PracticeInputAuthority,
    PracticeInputAuthorityLedger, PracticeIntent, PracticeIntentError, PracticeTargetIdentity,
    PracticeTargetTag, ProposalNonce, ResolvedPracticeBatch, ResolvedPracticeBatchError,
    ResolvedPracticeBatchItem, TaggedPracticeTarget, MAX_RESOLVED_PRACTICE_BATCH_CANONICAL_BYTES,
    MAX_RESOLVED_PRACTICE_BATCH_ITEMS, MIN_PRACTICE_INTENT_CANONICAL_BYTES,
    RESOLVED_PRACTICE_BATCH_DOMAIN_BYTES,
};

fn actor_id(value: u64) -> ActorOrganizationId {
    ActorOrganizationId::from_bytes(value.to_be_bytes())
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

fn authority() -> PracticeInputAuthority {
    PracticeInputAuthority {
        schema_version: 2,
        campaign_id: CampaignId::from_bytes([0x10; 16]),
        authority_kind: PracticeAuthorityKind::PlayerSeat,
        input_authority_id: InputAuthorityId::from_bytes([0x20; 16]),
        actor_org_id: actor_id(7),
        effective_from_tick: 10,
        effective_through_tick_exclusive: 20,
        decision_content_digest: [0x30; 32],
    }
}

fn ledger() -> PracticeInputAuthorityLedger {
    PracticeInputAuthorityLedger {
        schema_version: 2,
        rows: vec![authority()],
    }
}

fn intent(proposal_marker: u8) -> PracticeIntent {
    PracticeIntent {
        schema_version: 2,
        submit_after_tick: 10,
        resolve_tick: 11,
        input_authority_id: InputAuthorityId::from_bytes([0x20; 16]),
        actor_org_id: actor_id(7),
        practice_id: PracticeId::Strike,
        target: TaggedPracticeTarget {
            tag: PracticeTargetTag::LaborProcess,
            identity: PracticeTargetIdentity::from_bytes([0x50; 32]),
        },
        proposal_nonce: ProposalNonce::from_bytes([proposal_marker; 16]),
        quoted_content_digest: [0x30; 32],
        quoted_resource_contract_digest: [0x40; 32],
        parameters: Vec::new(),
        evidence_digests: vec![[0x70; 32], [0x80; 32]],
    }
}

fn item(proposal_marker: u8) -> ResolvedPracticeBatchItem {
    ResolvedPracticeBatchItem {
        authority: authority(),
        intent: intent(proposal_marker),
    }
}

fn batch(items: Vec<ResolvedPracticeBatchItem>) -> ResolvedPracticeBatch {
    ResolvedPracticeBatch {
        schema_version: 2,
        campaign_id: CampaignId::from_bytes([0x10; 16]),
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
fn resolved_batch_round_trips_independent_literal_bytes() {
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
        encode_resolved_practice_batch(&value, &ledger()).unwrap(),
        expected
    );
    assert_eq!(
        decode_resolved_practice_batch(&expected, &ledger()).unwrap(),
        value
    );
    assert_eq!(
        resolved_practice_batch_digest(&value, &ledger()).unwrap(),
        hex_digest("04292b2d21ec9512d0e5c7c7d184595c1f32904475007736a41f6dfba1f28594")
    );
}

#[test]
fn resolved_batch_binds_ledger_campaign_and_exact_authority_row() {
    let authoritative = ledger();
    let mut wrong_digest = batch(vec![item(0x60)]);
    wrong_digest.authority_ledger_digest = [0x99; 32];
    assert_eq!(
        validate_resolved_practice_batch(&wrong_digest, &authoritative),
        Err(ResolvedPracticeBatchError::Batch(
            PracticeBatchError::BatchLedgerDigest
        ))
    );

    let mut wrong_campaign = batch(vec![item(0x60)]);
    wrong_campaign.campaign_id = CampaignId::from_bytes([0x11; 16]);
    assert_eq!(
        validate_resolved_practice_batch(&wrong_campaign, &authoritative),
        Err(ResolvedPracticeBatchError::Batch(
            PracticeBatchError::BatchCampaign
        ))
    );

    let mut altered = batch(vec![item(0x60)]);
    altered.items[0].authority.decision_content_digest = [0x31; 32];
    assert_eq!(
        validate_resolved_practice_batch(&altered, &authoritative),
        Err(ResolvedPracticeBatchError::Batch(
            PracticeBatchError::BatchAuthorityMismatch
        ))
    );
}

#[test]
fn resolved_batch_preserves_missing_inactive_and_actor_authority_refusals() {
    let mut empty_ledger = PracticeInputAuthorityLedger {
        schema_version: 2,
        rows: Vec::new(),
    };
    let mut missing = batch(vec![item(0x60)]);
    missing.authority_ledger_digest = input_authority_ledger_digest(&empty_ledger).unwrap();
    assert_eq!(
        validate_resolved_practice_batch(&missing, &empty_ledger),
        Err(ResolvedPracticeBatchError::Authority(
            PracticeAuthorityError::AuthorityNotFound
        ))
    );

    empty_ledger.rows.push(PracticeInputAuthority {
        effective_through_tick_exclusive: 11,
        ..authority()
    });
    let mut inactive = batch(vec![item(0x60)]);
    inactive.authority_ledger_digest = input_authority_ledger_digest(&empty_ledger).unwrap();
    assert_eq!(
        validate_resolved_practice_batch(&inactive, &empty_ledger),
        Err(ResolvedPracticeBatchError::Authority(
            PracticeAuthorityError::AuthorityInactive
        ))
    );

    let mut wrong_actor = batch(vec![item(0x60)]);
    wrong_actor.items[0].intent.actor_org_id = actor_id(8);
    assert_eq!(
        validate_resolved_practice_batch(&wrong_actor, &ledger()),
        Err(ResolvedPracticeBatchError::Authority(
            PracticeAuthorityError::AuthorityActorMismatch
        ))
    );
}

#[test]
fn resolved_batch_binds_tick_content_resource_and_nested_intent_errors() {
    for (value, expected) in [
        {
            let mut value = batch(vec![item(0x60)]);
            value.resolve_tick = 12;
            (value, PracticeBatchError::BatchResolveTick)
        },
        {
            let mut value = batch(vec![item(0x60)]);
            value.content_digest = [0x31; 32];
            (value, PracticeBatchError::BatchContentDigest)
        },
        {
            let mut value = batch(vec![item(0x60)]);
            value.resource_allocation_contract_digest = [0x41; 32];
            (value, PracticeBatchError::BatchResourceContractDigest)
        },
    ] {
        assert_eq!(
            validate_resolved_practice_batch(&value, &ledger()),
            Err(ResolvedPracticeBatchError::Batch(expected))
        );
    }

    let mut malformed = batch(vec![item(0x60)]);
    malformed.items[0].intent.target.tag = PracticeTargetTag::SocialClass;
    assert_eq!(
        validate_resolved_practice_batch(&malformed, &ledger()),
        Err(ResolvedPracticeBatchError::Intent(
            PracticeIntentError::IntentTargetMismatch
        ))
    );
}

#[test]
fn resolved_batch_requires_ascending_unique_complete_proposal_keys() {
    let ordered = batch(vec![item(0x60), item(0x61)]);
    assert_eq!(
        validate_resolved_practice_batch(&ordered, &ledger()),
        Ok(())
    );

    let reversed = batch(vec![item(0x61), item(0x60)]);
    assert_eq!(
        validate_resolved_practice_batch(&reversed, &ledger()),
        Err(ResolvedPracticeBatchError::Batch(
            PracticeBatchError::BatchItemOrder
        ))
    );

    let mut distinct_bytes_same_key = item(0x60);
    distinct_bytes_same_key.intent.evidence_digests = vec![[0x70; 32], [0x81; 32]];
    let duplicate = batch(vec![item(0x60), distinct_bytes_same_key]);
    assert_eq!(
        validate_resolved_practice_batch(&duplicate, &ledger()),
        Err(ResolvedPracticeBatchError::Batch(
            PracticeBatchError::BatchItemDuplicate
        ))
    );
}

#[test]
fn resolved_batch_refuses_maximum_plus_one_before_nested_work() {
    let too_many = batch(vec![item(0x60); MAX_RESOLVED_PRACTICE_BATCH_ITEMS + 1]);
    assert_eq!(
        validate_resolved_practice_batch(&too_many, &ledger()),
        Err(ResolvedPracticeBatchError::Batch(
            PracticeBatchError::BatchItemLimit
        ))
    );

    let oversized = vec![0_u8; MAX_RESOLVED_PRACTICE_BATCH_CANONICAL_BYTES + 1];
    assert_eq!(
        decode_resolved_practice_batch(&oversized, &ledger()),
        Err(ResolvedPracticeBatchError::Batch(
            PracticeBatchError::BatchLength
        ))
    );
}

#[test]
fn resolved_batch_refuses_bad_ledger_before_nested_decode() {
    let authoritative = ledger();
    let mut payload =
        encode_resolved_practice_batch(&batch(vec![item(0x60)]), &authoritative).unwrap();
    let ledger_digest_offset = RESOLVED_PRACTICE_BATCH_DOMAIN_BYTES.len() + 1 + 2 + 16 + 8;
    let header_length = ledger_digest_offset + 32 + 32 + 32 + 2;
    payload[ledger_digest_offset] ^= 0x01;
    payload.truncate(header_length);

    assert_eq!(
        decode_resolved_practice_batch(&payload, &authoritative),
        Err(ResolvedPracticeBatchError::Batch(
            PracticeBatchError::BatchLedgerDigest
        ))
    );
}

#[test]
fn resolved_batch_minimum_nested_intent_length_is_exact() {
    let mut minimum = intent(0x60);
    minimum.evidence_digests.clear();

    assert_eq!(
        encode_practice_intent(&minimum).unwrap().len(),
        MIN_PRACTICE_INTENT_CANONICAL_BYTES
    );
}

#[test]
fn resolved_batch_error_table_is_exact() {
    let errors = [
        (PracticeBatchError::BatchDomain, 1_u16),
        (PracticeBatchError::BatchSchemaVersion, 2),
        (PracticeBatchError::BatchTruncated, 3),
        (PracticeBatchError::BatchTrailingBytes, 4),
        (PracticeBatchError::BatchLength, 5),
        (PracticeBatchError::BatchItemLimit, 6),
        (PracticeBatchError::BatchItemLength, 7),
        (PracticeBatchError::BatchItemOrder, 8),
        (PracticeBatchError::BatchItemDuplicate, 9),
        (PracticeBatchError::BatchResolveTick, 10),
        (PracticeBatchError::BatchLedgerDigest, 11),
        (PracticeBatchError::BatchCampaign, 12),
        (PracticeBatchError::BatchAuthorityMismatch, 13),
        (PracticeBatchError::BatchContentDigest, 14),
        (PracticeBatchError::BatchResourceContractDigest, 15),
    ];
    for (error, code) in errors {
        assert_eq!(u16::from(error), code);
        assert_eq!(PracticeBatchError::try_from(code), Ok(error));
    }
    assert!(PracticeBatchError::try_from(0_u16).is_err());
    assert!(PracticeBatchError::try_from(16_u16).is_err());
}

#[test]
fn resolved_batch_digest_binds_top_level_identity_and_items() {
    let authoritative = ledger();
    let empty = batch(Vec::new());
    let expected = resolved_practice_batch_digest(&empty, &authoritative).unwrap();
    let mut variants = Vec::with_capacity(5);

    let mut campaign = empty.clone();
    campaign.campaign_id = CampaignId::from_bytes([0x11; 16]);
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
            resolved_practice_batch_digest(variant, &authoritative).unwrap(),
            expected
        );
    }
}
