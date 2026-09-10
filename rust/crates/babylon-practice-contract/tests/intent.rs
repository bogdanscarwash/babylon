use babylon_practice_contract::ActorOrganizationId;
use babylon_practice_contract::{
    decode_practice_intent, encode_practice_intent, practice_intent_digest, practice_proposal_key,
    validate_practice_intent, validate_practice_intent_authority, CampaignId, InputAuthorityId,
    PracticeAuthorityError, PracticeAuthorityKind, PracticeId, PracticeInputAuthority,
    PracticeInputAuthorityLedger, PracticeIntent, PracticeIntentAuthorityError,
    PracticeIntentError, PracticeParameter, PracticeTargetIdentity, PracticeTargetTag,
    ProposalNonce, TaggedPracticeTarget,
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

fn strike_intent() -> PracticeIntent {
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
        proposal_nonce: ProposalNonce::from_bytes([0x60; 16]),
        quoted_content_digest: [0x30; 32],
        quoted_resource_contract_digest: [0x40; 32],
        parameters: Vec::new(),
        evidence_digests: vec![[0x70; 32], [0x80; 32]],
    }
}

fn authority_ledger() -> PracticeInputAuthorityLedger {
    PracticeInputAuthorityLedger {
        schema_version: 2,
        rows: vec![PracticeInputAuthority {
            schema_version: 2,
            campaign_id: CampaignId::from_bytes([0x10; 16]),
            authority_kind: PracticeAuthorityKind::PlayerSeat,
            input_authority_id: InputAuthorityId::from_bytes([0x20; 16]),
            actor_org_id: actor_id(7),
            effective_from_tick: 0,
            effective_through_tick_exclusive: 20,
            decision_content_digest: [0x90; 32],
        }],
    }
}

#[test]
fn intent_round_trips_independent_literal_bytes() {
    let expected = hex_bytes(concat!(
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
    let intent = strike_intent();

    assert_eq!(encode_practice_intent(&intent).unwrap(), expected);
    assert_eq!(decode_practice_intent(&expected).unwrap(), intent);
    assert_eq!(
        practice_intent_digest(&intent).unwrap(),
        hex_digest("0537e9106faaa91dd9b54e3dc68fcb6a45e7154d9086a835085ce85e9479f80b")
    );
}

#[test]
fn intent_practice_target_table_is_closed() {
    let accepted = [
        (PracticeId::Organize, PracticeTargetTag::SocialClass),
        (PracticeId::Agitate, PracticeTargetTag::SocialClass),
        (PracticeId::MutualAid, PracticeTargetTag::SocialClass),
        (PracticeId::Strike, PracticeTargetTag::LaborProcess),
        (PracticeId::Blockade, PracticeTargetTag::Route),
        (PracticeId::Blockade, PracticeTargetTag::ShipmentClass),
        (PracticeId::Blockade, PracticeTargetTag::AccessPoint),
        (PracticeId::Occupation, PracticeTargetTag::Facility),
        (PracticeId::Occupation, PracticeTargetTag::Territory),
        (PracticeId::Occupation, PracticeTargetTag::AccessPoint),
        (PracticeId::Damage, PracticeTargetTag::Facility),
        (PracticeId::Damage, PracticeTargetTag::Stock),
        (
            PracticeId::CapitalStrike,
            PracticeTargetTag::InvestmentCommitment,
        ),
        (
            PracticeId::CapitalStrike,
            PracticeTargetTag::CreditCommitment,
        ),
        (
            PracticeId::CapitalStrike,
            PracticeTargetTag::ProcurementCommitment,
        ),
        (
            PracticeId::CapitalStrike,
            PracticeTargetTag::ProductionCommitment,
        ),
    ];
    for (practice_id, tag) in accepted {
        let mut intent = strike_intent();
        intent.practice_id = practice_id;
        intent.target.tag = tag;
        assert_eq!(validate_practice_intent(&intent), Ok(()));
    }

    let mut mismatch = strike_intent();
    mismatch.target.tag = PracticeTargetTag::SocialClass;
    assert_eq!(
        validate_practice_intent(&mismatch),
        Err(PracticeIntentError::IntentTargetMismatch)
    );
}

#[test]
fn intent_refuses_parameters_and_noncanonical_evidence() {
    let mut parameterized = strike_intent();
    parameterized.parameters.push(PracticeParameter {
        key_u8: 1,
        value_kind_u8: 1,
        value_length_u16: 1,
        value_bytes: vec![1],
    });
    assert_eq!(
        validate_practice_intent(&parameterized),
        Err(PracticeIntentError::IntentParameterUnsupported)
    );

    let mut duplicate = strike_intent();
    duplicate.evidence_digests = vec![[0x70; 32], [0x70; 32]];
    assert_eq!(
        validate_practice_intent(&duplicate),
        Err(PracticeIntentError::IntentEvidenceDuplicate)
    );
    let mut unordered = strike_intent();
    unordered.evidence_digests = vec![[0x80; 32], [0x70; 32]];
    assert_eq!(
        validate_practice_intent(&unordered),
        Err(PracticeIntentError::IntentEvidenceOrder)
    );
}

#[test]
fn intent_authority_validation_consumes_the_authoritative_ledger() {
    let intent = strike_intent();
    let ledger = authority_ledger();
    let campaign = CampaignId::from_bytes([0x10; 16]);
    assert_eq!(
        validate_practice_intent_authority(&ledger, campaign, &intent),
        Ok(&ledger.rows[0])
    );

    let mut wrong_actor = intent;
    wrong_actor.actor_org_id = actor_id(8);
    assert_eq!(
        validate_practice_intent_authority(&ledger, campaign, &wrong_actor),
        Err(PracticeIntentAuthorityError::Authority(
            PracticeAuthorityError::AuthorityActorMismatch
        ))
    );

    assert_eq!(
        validate_practice_intent_authority(
            &ledger,
            CampaignId::from_bytes([0x11; 16]),
            &strike_intent(),
        ),
        Err(PracticeIntentAuthorityError::Authority(
            PracticeAuthorityError::AuthorityNotFound
        ))
    );

    let mut malformed = strike_intent();
    malformed.target.tag = PracticeTargetTag::SocialClass;
    assert_eq!(
        validate_practice_intent_authority(&ledger, campaign, &malformed),
        Err(PracticeIntentAuthorityError::Intent(
            PracticeIntentError::IntentTargetMismatch
        ))
    );
}

#[test]
fn intent_proposal_key_distinguishes_nonce_without_granting_priority() {
    let first = strike_intent();
    let mut second = first.clone();
    second.proposal_nonce = ProposalNonce::from_bytes([0x61; 16]);

    assert_ne!(
        practice_proposal_key(&first),
        practice_proposal_key(&second)
    );
    assert_eq!(first.actor_org_id, second.actor_org_id);
}

#[test]
fn intent_error_and_discriminant_tables_are_exact() {
    let errors = [
        (PracticeIntentError::IntentDomain, 1_u16),
        (PracticeIntentError::IntentSchemaVersion, 2),
        (PracticeIntentError::IntentEnumCode, 3),
        (PracticeIntentError::IntentTruncated, 4),
        (PracticeIntentError::IntentTrailingBytes, 5),
        (PracticeIntentError::IntentLength, 6),
        (PracticeIntentError::IntentTickOverflow, 7),
        (PracticeIntentError::IntentTickMismatch, 8),
        (PracticeIntentError::IntentParameterLimit, 9),
        (PracticeIntentError::IntentParameterLength, 10),
        (PracticeIntentError::IntentParameterUnsupported, 11),
        (PracticeIntentError::IntentEvidenceLimit, 12),
        (PracticeIntentError::IntentEvidenceOrder, 13),
        (PracticeIntentError::IntentEvidenceDuplicate, 14),
        (PracticeIntentError::IntentTargetMismatch, 15),
    ];
    for (error, code) in errors {
        assert_eq!(u16::from(error), code);
        assert_eq!(PracticeIntentError::try_from(code), Ok(error));
    }
    assert!(PracticeIntentError::try_from(0_u16).is_err());
    assert!(PracticeIntentError::try_from(16_u16).is_err());

    for (code, practice) in [
        (1_u8, PracticeId::Organize),
        (2, PracticeId::Agitate),
        (3, PracticeId::MutualAid),
        (4, PracticeId::Strike),
        (5, PracticeId::Blockade),
        (6, PracticeId::Occupation),
        (7, PracticeId::Damage),
        (8, PracticeId::CapitalStrike),
    ] {
        assert_eq!(PracticeId::try_from(code), Ok(practice));
        assert_eq!(practice as u8, code);
    }
    assert!(PracticeId::try_from(0_u8).is_err());
    assert!(PracticeId::try_from(9_u8).is_err());

    for (code, tag) in [
        (1_u8, PracticeTargetTag::SocialClass),
        (2, PracticeTargetTag::LaborProcess),
        (3, PracticeTargetTag::Route),
        (4, PracticeTargetTag::ShipmentClass),
        (5, PracticeTargetTag::AccessPoint),
        (6, PracticeTargetTag::Facility),
        (7, PracticeTargetTag::Territory),
        (8, PracticeTargetTag::Stock),
        (9, PracticeTargetTag::InvestmentCommitment),
        (10, PracticeTargetTag::CreditCommitment),
        (11, PracticeTargetTag::ProcurementCommitment),
        (12, PracticeTargetTag::ProductionCommitment),
    ] {
        assert_eq!(PracticeTargetTag::try_from(code), Ok(tag));
        assert_eq!(tag as u8, code);
    }
    assert!(PracticeTargetTag::try_from(0_u8).is_err());
    assert!(PracticeTargetTag::try_from(13_u8).is_err());
}

#[test]
fn intent_bounds_refuse_maximum_plus_one_before_unbounded_work() {
    let mut maximum = strike_intent();
    maximum.evidence_digests = (0_u8..64)
        .map(|index| {
            let mut digest = [0_u8; 32];
            digest[31] = index;
            digest
        })
        .collect();
    assert_eq!(validate_practice_intent(&maximum), Ok(()));

    let mut too_many = maximum;
    let mut final_digest = [0_u8; 32];
    final_digest[31] = 64;
    too_many.evidence_digests.push(final_digest);
    assert_eq!(
        validate_practice_intent(&too_many),
        Err(PracticeIntentError::IntentEvidenceLimit)
    );
    assert_eq!(
        decode_practice_intent(&vec![0_u8; 16_385]),
        Err(PracticeIntentError::IntentLength)
    );
}

#[test]
fn intent_digest_binds_every_valid_scalar_and_collection_identity() {
    let base = strike_intent();
    let expected = practice_intent_digest(&base).unwrap();
    let mut variants = Vec::with_capacity(9);

    let mut timing = base.clone();
    timing.submit_after_tick = 11;
    timing.resolve_tick = 12;
    variants.push(timing);
    let mut authority = base.clone();
    authority.input_authority_id = InputAuthorityId::from_bytes([0x21; 16]);
    variants.push(authority);
    let mut actor = base.clone();
    actor.actor_org_id = actor_id(8);
    variants.push(actor);
    let mut practice = base.clone();
    practice.practice_id = PracticeId::Blockade;
    practice.target.tag = PracticeTargetTag::Route;
    variants.push(practice);
    let mut target = base.clone();
    target.target.identity = PracticeTargetIdentity::from_bytes([0x51; 32]);
    variants.push(target);
    let mut nonce = base.clone();
    nonce.proposal_nonce = ProposalNonce::from_bytes([0x61; 16]);
    variants.push(nonce);
    let mut content = base.clone();
    content.quoted_content_digest = [0x31; 32];
    variants.push(content);
    let mut resources = base.clone();
    resources.quoted_resource_contract_digest = [0x41; 32];
    variants.push(resources);
    let mut evidence = base;
    evidence.evidence_digests = vec![[0x70; 32], [0x81; 32]];
    variants.push(evidence);

    assert_eq!(variants.len(), 9);
    for variant in variants.iter().take(9) {
        assert_ne!(practice_intent_digest(variant).unwrap(), expected);
    }
}
