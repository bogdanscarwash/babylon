use babylon_practice_contract::actor_v2::ActorOrganizationIdV2;
use babylon_practice_contract::{
    decode_practice_intent_v2, encode_practice_intent_v2, practice_intent_v2_digest,
    practice_proposal_key_v2, validate_practice_intent_authority_v2, validate_practice_intent_v2,
    CampaignIdV2, InputAuthorityIdV2, PracticeAuthorityKindV2, PracticeAuthorityV2Error,
    PracticeIdV2, PracticeInputAuthorityLedgerV2, PracticeInputAuthorityV2,
    PracticeIntentAuthorityV2Error, PracticeIntentV2, PracticeIntentV2Error, PracticeParameterV2,
    PracticeTargetIdentityV2, PracticeTargetTagV2, ProposalNonceV2, TaggedPracticeTargetV2,
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

fn strike_intent() -> PracticeIntentV2 {
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
        proposal_nonce: ProposalNonceV2::from_bytes([0x60; 16]),
        quoted_content_digest: [0x30; 32],
        quoted_resource_contract_digest: [0x40; 32],
        parameters: Vec::new(),
        evidence_digests: vec![[0x70; 32], [0x80; 32]],
    }
}

fn authority_ledger() -> PracticeInputAuthorityLedgerV2 {
    PracticeInputAuthorityLedgerV2 {
        schema_version: 2,
        rows: vec![PracticeInputAuthorityV2 {
            schema_version: 2,
            campaign_id: CampaignIdV2::from_bytes([0x10; 16]),
            authority_kind: PracticeAuthorityKindV2::PlayerSeat,
            input_authority_id: InputAuthorityIdV2::from_bytes([0x20; 16]),
            actor_org_id: actor_id(7),
            effective_from_tick: 0,
            effective_through_tick_exclusive: 20,
            decision_content_digest: [0x90; 32],
        }],
    }
}

#[test]
fn intent_v2_round_trips_independent_literal_bytes() {
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

    assert_eq!(encode_practice_intent_v2(&intent).unwrap(), expected);
    assert_eq!(decode_practice_intent_v2(&expected).unwrap(), intent);
    assert_eq!(
        practice_intent_v2_digest(&intent).unwrap(),
        hex_digest("0537e9106faaa91dd9b54e3dc68fcb6a45e7154d9086a835085ce85e9479f80b")
    );
}

#[test]
fn intent_v2_practice_target_table_is_closed() {
    let accepted = [
        (PracticeIdV2::Organize, PracticeTargetTagV2::SocialClass),
        (PracticeIdV2::Agitate, PracticeTargetTagV2::SocialClass),
        (PracticeIdV2::MutualAid, PracticeTargetTagV2::SocialClass),
        (PracticeIdV2::Strike, PracticeTargetTagV2::LaborProcess),
        (PracticeIdV2::Blockade, PracticeTargetTagV2::Route),
        (PracticeIdV2::Blockade, PracticeTargetTagV2::ShipmentClass),
        (PracticeIdV2::Blockade, PracticeTargetTagV2::AccessPoint),
        (PracticeIdV2::Occupation, PracticeTargetTagV2::Facility),
        (PracticeIdV2::Occupation, PracticeTargetTagV2::Territory),
        (PracticeIdV2::Occupation, PracticeTargetTagV2::AccessPoint),
        (PracticeIdV2::Damage, PracticeTargetTagV2::Facility),
        (PracticeIdV2::Damage, PracticeTargetTagV2::Stock),
        (
            PracticeIdV2::CapitalStrike,
            PracticeTargetTagV2::InvestmentCommitment,
        ),
        (
            PracticeIdV2::CapitalStrike,
            PracticeTargetTagV2::CreditCommitment,
        ),
        (
            PracticeIdV2::CapitalStrike,
            PracticeTargetTagV2::ProcurementCommitment,
        ),
        (
            PracticeIdV2::CapitalStrike,
            PracticeTargetTagV2::ProductionCommitment,
        ),
    ];
    for (practice_id, tag) in accepted {
        let mut intent = strike_intent();
        intent.practice_id = practice_id;
        intent.target.tag = tag;
        assert_eq!(validate_practice_intent_v2(&intent), Ok(()));
    }

    let mut mismatch = strike_intent();
    mismatch.target.tag = PracticeTargetTagV2::SocialClass;
    assert_eq!(
        validate_practice_intent_v2(&mismatch),
        Err(PracticeIntentV2Error::IntentTargetMismatch)
    );
}

#[test]
fn intent_v2_refuses_parameters_and_noncanonical_evidence() {
    let mut parameterized = strike_intent();
    parameterized.parameters.push(PracticeParameterV2 {
        key_u8: 1,
        value_kind_u8: 1,
        value_length_u16: 1,
        value_bytes: vec![1],
    });
    assert_eq!(
        validate_practice_intent_v2(&parameterized),
        Err(PracticeIntentV2Error::IntentParameterUnsupported)
    );

    let mut duplicate = strike_intent();
    duplicate.evidence_digests = vec![[0x70; 32], [0x70; 32]];
    assert_eq!(
        validate_practice_intent_v2(&duplicate),
        Err(PracticeIntentV2Error::IntentEvidenceDuplicate)
    );
    let mut unordered = strike_intent();
    unordered.evidence_digests = vec![[0x80; 32], [0x70; 32]];
    assert_eq!(
        validate_practice_intent_v2(&unordered),
        Err(PracticeIntentV2Error::IntentEvidenceOrder)
    );
}

#[test]
fn intent_v2_authority_validation_consumes_the_authoritative_ledger() {
    let intent = strike_intent();
    let ledger = authority_ledger();
    let campaign = CampaignIdV2::from_bytes([0x10; 16]);
    assert_eq!(
        validate_practice_intent_authority_v2(&ledger, campaign, &intent),
        Ok(&ledger.rows[0])
    );

    let mut wrong_actor = intent;
    wrong_actor.actor_org_id = actor_id(8);
    assert_eq!(
        validate_practice_intent_authority_v2(&ledger, campaign, &wrong_actor),
        Err(PracticeIntentAuthorityV2Error::Authority(
            PracticeAuthorityV2Error::AuthorityActorMismatch
        ))
    );

    assert_eq!(
        validate_practice_intent_authority_v2(
            &ledger,
            CampaignIdV2::from_bytes([0x11; 16]),
            &strike_intent(),
        ),
        Err(PracticeIntentAuthorityV2Error::Authority(
            PracticeAuthorityV2Error::AuthorityNotFound
        ))
    );

    let mut malformed = strike_intent();
    malformed.target.tag = PracticeTargetTagV2::SocialClass;
    assert_eq!(
        validate_practice_intent_authority_v2(&ledger, campaign, &malformed),
        Err(PracticeIntentAuthorityV2Error::Intent(
            PracticeIntentV2Error::IntentTargetMismatch
        ))
    );
}

#[test]
fn intent_v2_proposal_key_distinguishes_nonce_without_granting_priority() {
    let first = strike_intent();
    let mut second = first.clone();
    second.proposal_nonce = ProposalNonceV2::from_bytes([0x61; 16]);

    assert_ne!(
        practice_proposal_key_v2(&first),
        practice_proposal_key_v2(&second)
    );
    assert_eq!(first.actor_org_id, second.actor_org_id);
}

#[test]
fn intent_v2_error_and_discriminant_tables_are_exact() {
    let errors = [
        (PracticeIntentV2Error::IntentDomain, 1_u16),
        (PracticeIntentV2Error::IntentSchemaVersion, 2),
        (PracticeIntentV2Error::IntentEnumCode, 3),
        (PracticeIntentV2Error::IntentTruncated, 4),
        (PracticeIntentV2Error::IntentTrailingBytes, 5),
        (PracticeIntentV2Error::IntentLength, 6),
        (PracticeIntentV2Error::IntentTickOverflow, 7),
        (PracticeIntentV2Error::IntentTickMismatch, 8),
        (PracticeIntentV2Error::IntentParameterLimit, 9),
        (PracticeIntentV2Error::IntentParameterLength, 10),
        (PracticeIntentV2Error::IntentParameterUnsupported, 11),
        (PracticeIntentV2Error::IntentEvidenceLimit, 12),
        (PracticeIntentV2Error::IntentEvidenceOrder, 13),
        (PracticeIntentV2Error::IntentEvidenceDuplicate, 14),
        (PracticeIntentV2Error::IntentTargetMismatch, 15),
    ];
    for (error, code) in errors {
        assert_eq!(u16::from(error), code);
        assert_eq!(PracticeIntentV2Error::try_from(code), Ok(error));
    }
    assert!(PracticeIntentV2Error::try_from(0_u16).is_err());
    assert!(PracticeIntentV2Error::try_from(16_u16).is_err());

    for (code, practice) in [
        (1_u8, PracticeIdV2::Organize),
        (2, PracticeIdV2::Agitate),
        (3, PracticeIdV2::MutualAid),
        (4, PracticeIdV2::Strike),
        (5, PracticeIdV2::Blockade),
        (6, PracticeIdV2::Occupation),
        (7, PracticeIdV2::Damage),
        (8, PracticeIdV2::CapitalStrike),
    ] {
        assert_eq!(PracticeIdV2::try_from(code), Ok(practice));
        assert_eq!(practice as u8, code);
    }
    assert!(PracticeIdV2::try_from(0_u8).is_err());
    assert!(PracticeIdV2::try_from(9_u8).is_err());

    for (code, tag) in [
        (1_u8, PracticeTargetTagV2::SocialClass),
        (2, PracticeTargetTagV2::LaborProcess),
        (3, PracticeTargetTagV2::Route),
        (4, PracticeTargetTagV2::ShipmentClass),
        (5, PracticeTargetTagV2::AccessPoint),
        (6, PracticeTargetTagV2::Facility),
        (7, PracticeTargetTagV2::Territory),
        (8, PracticeTargetTagV2::Stock),
        (9, PracticeTargetTagV2::InvestmentCommitment),
        (10, PracticeTargetTagV2::CreditCommitment),
        (11, PracticeTargetTagV2::ProcurementCommitment),
        (12, PracticeTargetTagV2::ProductionCommitment),
    ] {
        assert_eq!(PracticeTargetTagV2::try_from(code), Ok(tag));
        assert_eq!(tag as u8, code);
    }
    assert!(PracticeTargetTagV2::try_from(0_u8).is_err());
    assert!(PracticeTargetTagV2::try_from(13_u8).is_err());
}

#[test]
fn intent_v2_bounds_refuse_maximum_plus_one_before_unbounded_work() {
    let mut maximum = strike_intent();
    maximum.evidence_digests = (0_u8..64)
        .map(|index| {
            let mut digest = [0_u8; 32];
            digest[31] = index;
            digest
        })
        .collect();
    assert_eq!(validate_practice_intent_v2(&maximum), Ok(()));

    let mut too_many = maximum;
    let mut final_digest = [0_u8; 32];
    final_digest[31] = 64;
    too_many.evidence_digests.push(final_digest);
    assert_eq!(
        validate_practice_intent_v2(&too_many),
        Err(PracticeIntentV2Error::IntentEvidenceLimit)
    );
    assert_eq!(
        decode_practice_intent_v2(&vec![0_u8; 16_385]),
        Err(PracticeIntentV2Error::IntentLength)
    );
}

#[test]
fn intent_v2_digest_binds_every_valid_scalar_and_collection_identity() {
    let base = strike_intent();
    let expected = practice_intent_v2_digest(&base).unwrap();
    let mut variants = Vec::with_capacity(9);

    let mut timing = base.clone();
    timing.submit_after_tick = 11;
    timing.resolve_tick = 12;
    variants.push(timing);
    let mut authority = base.clone();
    authority.input_authority_id = InputAuthorityIdV2::from_bytes([0x21; 16]);
    variants.push(authority);
    let mut actor = base.clone();
    actor.actor_org_id = actor_id(8);
    variants.push(actor);
    let mut practice = base.clone();
    practice.practice_id = PracticeIdV2::Blockade;
    practice.target.tag = PracticeTargetTagV2::Route;
    variants.push(practice);
    let mut target = base.clone();
    target.target.identity = PracticeTargetIdentityV2::from_bytes([0x51; 32]);
    variants.push(target);
    let mut nonce = base.clone();
    nonce.proposal_nonce = ProposalNonceV2::from_bytes([0x61; 16]);
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
        assert_ne!(practice_intent_v2_digest(variant).unwrap(), expected);
    }
}
