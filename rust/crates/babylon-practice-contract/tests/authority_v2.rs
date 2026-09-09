use babylon_practice_contract::actor_v2::ActorOrganizationIdV2;
use babylon_practice_contract::{
    active_player_authority_v2, decode_input_authority_ledger_v2, decode_input_authority_v2,
    encode_input_authority_ledger_v2, encode_input_authority_v2, input_authority_ledger_v2_digest,
    input_authority_v2_digest, resolve_input_authority_v2, validate_input_authority_ledger_v2,
    CampaignIdV2, InputAuthorityIdV2, PracticeAuthorityKindV2, PracticeAuthorityV2Error,
    PracticeInputAuthorityLedgerV2, PracticeInputAuthorityV2,
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

fn player_authority() -> PracticeInputAuthorityV2 {
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

fn policy_authority(
    input_authority_byte: u8,
    actor_org_id: u64,
    from: u64,
    through: u64,
) -> PracticeInputAuthorityV2 {
    PracticeInputAuthorityV2 {
        schema_version: 2,
        campaign_id: CampaignIdV2::from_bytes([0x10; 16]),
        authority_kind: PracticeAuthorityKindV2::DeterministicPolicy,
        input_authority_id: InputAuthorityIdV2::from_bytes([input_authority_byte; 16]),
        actor_org_id: actor_id(actor_org_id),
        effective_from_tick: from,
        effective_through_tick_exclusive: through,
        decision_content_digest: [0x40; 32],
    }
}

#[test]
fn authority_v2_row_round_trips_the_literal_language_neutral_bytes() {
    let expected = hex_bytes(concat!(
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
    ));
    let value = player_authority();

    assert_eq!(encode_input_authority_v2(&value).unwrap(), expected);
    assert_eq!(decode_input_authority_v2(&expected).unwrap(), value);
}

#[test]
fn authority_v2_row_refuses_each_malformed_boundary() {
    let canonical = encode_input_authority_v2(&player_authority()).unwrap();
    let mut wrong_schema = player_authority();
    wrong_schema.schema_version = 1;
    assert_eq!(
        encode_input_authority_v2(&wrong_schema),
        Err(PracticeAuthorityV2Error::AuthoritySchemaVersion)
    );
    let mut empty_interval = player_authority();
    empty_interval.effective_through_tick_exclusive = empty_interval.effective_from_tick;
    assert_eq!(
        encode_input_authority_v2(&empty_interval),
        Err(PracticeAuthorityV2Error::AuthorityEmptyInterval)
    );

    let mut unknown_kind = canonical.clone();
    unknown_kind[54] = 3;
    assert_eq!(
        decode_input_authority_v2(&unknown_kind),
        Err(PracticeAuthorityV2Error::AuthorityEnumCode)
    );
    assert_eq!(
        decode_input_authority_v2(&canonical[..canonical.len() - 1]),
        Err(PracticeAuthorityV2Error::AuthorityTruncated)
    );
    let mut trailing = canonical;
    trailing.push(0);
    assert_eq!(
        decode_input_authority_v2(&trailing),
        Err(PracticeAuthorityV2Error::AuthorityTrailingBytes)
    );
}

#[test]
fn authority_v2_ledger_round_trips_one_literal_row() {
    let row_hex = concat!(
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
    );
    let expected = hex_bytes(&format!(
        "{}{}{}{}{}",
        "626162796c6f6e2e70726163746963652d696e7075742d617574686f726974792d6c65646765722e7632",
        "00",
        "0002",
        "00000001",
        row_hex,
    ));
    let ledger = PracticeInputAuthorityLedgerV2 {
        schema_version: 2,
        rows: vec![player_authority()],
    };

    assert_eq!(encode_input_authority_ledger_v2(&ledger).unwrap(), expected);
    assert_eq!(decode_input_authority_ledger_v2(&expected).unwrap(), ledger);
}

#[test]
fn authority_v2_ledger_refuses_order_duplicate_overlap_and_limit() {
    let first = policy_authority(0x21, 8, 10, 20);
    let next = policy_authority(0x21, 8, 20, 30);
    let ledger = PracticeInputAuthorityLedgerV2 {
        schema_version: 2,
        rows: vec![first.clone(), next.clone()],
    };
    assert_eq!(validate_input_authority_ledger_v2(&ledger), Ok(()));

    for (rows, expected) in [
        (
            vec![next.clone(), first.clone()],
            PracticeAuthorityV2Error::AuthorityLedgerOrder,
        ),
        (
            vec![first.clone(), first.clone()],
            PracticeAuthorityV2Error::AuthorityLedgerDuplicate,
        ),
        (
            vec![first.clone(), policy_authority(0x21, 9, 19, 30)],
            PracticeAuthorityV2Error::AuthorityIntervalOverlap,
        ),
    ] {
        assert_eq!(
            validate_input_authority_ledger_v2(&PracticeInputAuthorityLedgerV2 {
                schema_version: 2,
                rows,
            }),
            Err(expected)
        );
    }
}

#[test]
fn authority_v2_lookup_requires_campaign_authority_actor_and_active_tick() {
    let player = player_authority();
    let ledger = PracticeInputAuthorityLedgerV2 {
        schema_version: 2,
        rows: vec![player.clone()],
    };
    assert_eq!(
        resolve_input_authority_v2(
            &ledger,
            player.campaign_id,
            player.input_authority_id,
            actor_id(7),
            10,
        ),
        Ok(&player)
    );
    assert_eq!(
        resolve_input_authority_v2(
            &ledger,
            player.campaign_id,
            player.input_authority_id,
            actor_id(8),
            10,
        ),
        Err(PracticeAuthorityV2Error::AuthorityActorMismatch)
    );
    assert_eq!(
        resolve_input_authority_v2(
            &ledger,
            player.campaign_id,
            player.input_authority_id,
            actor_id(7),
            20,
        ),
        Err(PracticeAuthorityV2Error::AuthorityInactive)
    );
    assert_eq!(
        resolve_input_authority_v2(
            &ledger,
            player.campaign_id,
            InputAuthorityIdV2::from_bytes([0x99; 16]),
            actor_id(7),
            10,
        ),
        Err(PracticeAuthorityV2Error::AuthorityNotFound)
    );
    assert_eq!(
        active_player_authority_v2(&ledger, player.campaign_id, 10),
        Ok(&player)
    );
    assert_eq!(
        active_player_authority_v2(&ledger, player.campaign_id, 20),
        Err(PracticeAuthorityV2Error::AuthorityPlayerSeatMissing)
    );
}

#[test]
fn authority_v2_ledger_refuses_two_active_player_seats_for_one_campaign() {
    let first = player_authority();
    let mut second = player_authority();
    second.input_authority_id = InputAuthorityIdV2::from_bytes([0x21; 16]);
    second.actor_org_id = actor_id(8);
    assert_eq!(
        validate_input_authority_ledger_v2(&PracticeInputAuthorityLedgerV2 {
            schema_version: 2,
            rows: vec![first, second],
        }),
        Err(PracticeAuthorityV2Error::AuthorityPlayerSeatOverlap)
    );
}

#[test]
fn authority_v2_ledger_refuses_midcampaign_player_seat_reassignment() {
    let first = player_authority();
    let mut reassigned = player_authority();
    reassigned.input_authority_id = InputAuthorityIdV2::from_bytes([0x21; 16]);
    reassigned.actor_org_id = actor_id(8);
    reassigned.effective_from_tick = first.effective_through_tick_exclusive;
    reassigned.effective_through_tick_exclusive = 30;

    assert_eq!(
        validate_input_authority_ledger_v2(&PracticeInputAuthorityLedgerV2 {
            schema_version: 2,
            rows: vec![first, reassigned],
        }),
        Err(PracticeAuthorityV2Error::AuthorityPlayerSeatReassignment)
    );
}

#[test]
fn authority_v2_digests_are_pinned_to_independent_literals() {
    let row = player_authority();
    let ledger = PracticeInputAuthorityLedgerV2 {
        schema_version: 2,
        rows: vec![row.clone()],
    };
    assert_eq!(
        input_authority_v2_digest(&row).unwrap(),
        hex_digest("e7ef0883ac5adbf5f6a7424e820676327c34c280341bb8ab4b4ae015308e7d85")
    );
    assert_eq!(
        input_authority_ledger_v2_digest(&ledger).unwrap(),
        hex_digest("3415c8298f3a78e53fe3660ac453544b43f8be32dc12071928bb2b8c3782908a")
    );
}

#[test]
fn authority_v2_error_codes_are_closed_and_exact() {
    let expected = [
        (PracticeAuthorityV2Error::AuthorityDomain, 1_u16),
        (PracticeAuthorityV2Error::AuthoritySchemaVersion, 2),
        (PracticeAuthorityV2Error::AuthorityEnumCode, 3),
        (PracticeAuthorityV2Error::AuthorityTruncated, 4),
        (PracticeAuthorityV2Error::AuthorityTrailingBytes, 5),
        (PracticeAuthorityV2Error::AuthorityEmptyInterval, 6),
        (PracticeAuthorityV2Error::AuthorityLedgerLimit, 7),
        (PracticeAuthorityV2Error::AuthorityLedgerOrder, 8),
        (PracticeAuthorityV2Error::AuthorityLedgerDuplicate, 9),
        (PracticeAuthorityV2Error::AuthorityIntervalOverlap, 10),
        (PracticeAuthorityV2Error::AuthorityPlayerSeatOverlap, 11),
        (PracticeAuthorityV2Error::AuthorityNotFound, 12),
        (PracticeAuthorityV2Error::AuthorityInactive, 13),
        (PracticeAuthorityV2Error::AuthorityActorMismatch, 14),
        (PracticeAuthorityV2Error::AuthorityPlayerSeatMissing, 15),
        (
            PracticeAuthorityV2Error::AuthorityPlayerSeatReassignment,
            16,
        ),
    ];
    for (error, code) in expected {
        assert_eq!(u16::from(error), code);
        assert_eq!(PracticeAuthorityV2Error::try_from(code), Ok(error));
    }
    assert!(PracticeAuthorityV2Error::try_from(0_u16).is_err());
    assert!(PracticeAuthorityV2Error::try_from(17_u16).is_err());
}

#[test]
fn authority_v2_ledger_accepts_16384_rows_and_refuses_16385_before_sorting() {
    let rows: Vec<_> = (0_u128..16_384)
        .map(|index| PracticeInputAuthorityV2 {
            input_authority_id: InputAuthorityIdV2::from_bytes(index.to_be_bytes()),
            ..policy_authority(0x21, 8, 10, 20)
        })
        .collect();
    assert_eq!(
        validate_input_authority_ledger_v2(&PracticeInputAuthorityLedgerV2 {
            schema_version: 2,
            rows: rows.clone(),
        }),
        Ok(())
    );
    let mut too_many = rows;
    too_many.push(policy_authority(0xff, 8, 10, 20));
    assert_eq!(
        validate_input_authority_ledger_v2(&PracticeInputAuthorityLedgerV2 {
            schema_version: 2,
            rows: too_many,
        }),
        Err(PracticeAuthorityV2Error::AuthorityLedgerLimit)
    );
}
