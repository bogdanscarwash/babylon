use babylon_practice_contract::ActorOrganizationId;
use babylon_practice_contract::{
    active_player_authority, decode_input_authority, decode_input_authority_ledger,
    encode_input_authority, encode_input_authority_ledger, input_authority_digest,
    input_authority_ledger_digest, resolve_input_authority, validate_input_authority_ledger,
    CampaignId, InputAuthorityId, PracticeAuthorityError, PracticeAuthorityKind,
    PracticeInputAuthority, PracticeInputAuthorityLedger,
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

fn player_authority() -> PracticeInputAuthority {
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

fn policy_authority(
    input_authority_byte: u8,
    actor_org_id: u64,
    from: u64,
    through: u64,
) -> PracticeInputAuthority {
    PracticeInputAuthority {
        schema_version: 2,
        campaign_id: CampaignId::from_bytes([0x10; 16]),
        authority_kind: PracticeAuthorityKind::DeterministicPolicy,
        input_authority_id: InputAuthorityId::from_bytes([input_authority_byte; 16]),
        actor_org_id: actor_id(actor_org_id),
        effective_from_tick: from,
        effective_through_tick_exclusive: through,
        decision_content_digest: [0x40; 32],
    }
}

#[test]
fn authority_row_round_trips_the_literal_language_neutral_bytes() {
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

    assert_eq!(encode_input_authority(&value).unwrap(), expected);
    assert_eq!(decode_input_authority(&expected).unwrap(), value);
}

#[test]
fn authority_row_refuses_each_malformed_boundary() {
    let canonical = encode_input_authority(&player_authority()).unwrap();
    let mut wrong_schema = player_authority();
    wrong_schema.schema_version = 1;
    assert_eq!(
        encode_input_authority(&wrong_schema),
        Err(PracticeAuthorityError::AuthoritySchemaVersion)
    );
    let mut empty_interval = player_authority();
    empty_interval.effective_through_tick_exclusive = empty_interval.effective_from_tick;
    assert_eq!(
        encode_input_authority(&empty_interval),
        Err(PracticeAuthorityError::AuthorityEmptyInterval)
    );

    let mut unknown_kind = canonical.clone();
    unknown_kind[54] = 3;
    assert_eq!(
        decode_input_authority(&unknown_kind),
        Err(PracticeAuthorityError::AuthorityEnumCode)
    );
    assert_eq!(
        decode_input_authority(&canonical[..canonical.len() - 1]),
        Err(PracticeAuthorityError::AuthorityTruncated)
    );
    let mut trailing = canonical;
    trailing.push(0);
    assert_eq!(
        decode_input_authority(&trailing),
        Err(PracticeAuthorityError::AuthorityTrailingBytes)
    );
}

#[test]
fn authority_ledger_round_trips_one_literal_row() {
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
    let ledger = PracticeInputAuthorityLedger {
        schema_version: 2,
        rows: vec![player_authority()],
    };

    assert_eq!(encode_input_authority_ledger(&ledger).unwrap(), expected);
    assert_eq!(decode_input_authority_ledger(&expected).unwrap(), ledger);
}

#[test]
fn authority_ledger_refuses_order_duplicate_overlap_and_limit() {
    let first = policy_authority(0x21, 8, 10, 20);
    let next = policy_authority(0x21, 8, 20, 30);
    let ledger = PracticeInputAuthorityLedger {
        schema_version: 2,
        rows: vec![first.clone(), next.clone()],
    };
    assert_eq!(validate_input_authority_ledger(&ledger), Ok(()));

    for (rows, expected) in [
        (
            vec![next.clone(), first.clone()],
            PracticeAuthorityError::AuthorityLedgerOrder,
        ),
        (
            vec![first.clone(), first.clone()],
            PracticeAuthorityError::AuthorityLedgerDuplicate,
        ),
        (
            vec![first.clone(), policy_authority(0x21, 9, 19, 30)],
            PracticeAuthorityError::AuthorityIntervalOverlap,
        ),
    ] {
        assert_eq!(
            validate_input_authority_ledger(&PracticeInputAuthorityLedger {
                schema_version: 2,
                rows,
            }),
            Err(expected)
        );
    }
}

#[test]
fn authority_lookup_requires_campaign_authority_actor_and_active_tick() {
    let player = player_authority();
    let ledger = PracticeInputAuthorityLedger {
        schema_version: 2,
        rows: vec![player.clone()],
    };
    assert_eq!(
        resolve_input_authority(
            &ledger,
            player.campaign_id,
            player.input_authority_id,
            actor_id(7),
            10,
        ),
        Ok(&player)
    );
    assert_eq!(
        resolve_input_authority(
            &ledger,
            player.campaign_id,
            player.input_authority_id,
            actor_id(8),
            10,
        ),
        Err(PracticeAuthorityError::AuthorityActorMismatch)
    );
    assert_eq!(
        resolve_input_authority(
            &ledger,
            player.campaign_id,
            player.input_authority_id,
            actor_id(7),
            20,
        ),
        Err(PracticeAuthorityError::AuthorityInactive)
    );
    assert_eq!(
        resolve_input_authority(
            &ledger,
            player.campaign_id,
            InputAuthorityId::from_bytes([0x99; 16]),
            actor_id(7),
            10,
        ),
        Err(PracticeAuthorityError::AuthorityNotFound)
    );
    assert_eq!(
        active_player_authority(&ledger, player.campaign_id, 10),
        Ok(&player)
    );
    assert_eq!(
        active_player_authority(&ledger, player.campaign_id, 20),
        Err(PracticeAuthorityError::AuthorityPlayerSeatMissing)
    );
}

#[test]
fn authority_ledger_refuses_two_active_player_seats_for_one_campaign() {
    let first = player_authority();
    let mut second = player_authority();
    second.input_authority_id = InputAuthorityId::from_bytes([0x21; 16]);
    second.actor_org_id = actor_id(8);
    assert_eq!(
        validate_input_authority_ledger(&PracticeInputAuthorityLedger {
            schema_version: 2,
            rows: vec![first, second],
        }),
        Err(PracticeAuthorityError::AuthorityPlayerSeatOverlap)
    );
}

#[test]
fn authority_ledger_refuses_midcampaign_player_seat_reassignment() {
    let first = player_authority();
    let mut reassigned = player_authority();
    reassigned.input_authority_id = InputAuthorityId::from_bytes([0x21; 16]);
    reassigned.actor_org_id = actor_id(8);
    reassigned.effective_from_tick = first.effective_through_tick_exclusive;
    reassigned.effective_through_tick_exclusive = 30;

    assert_eq!(
        validate_input_authority_ledger(&PracticeInputAuthorityLedger {
            schema_version: 2,
            rows: vec![first, reassigned],
        }),
        Err(PracticeAuthorityError::AuthorityPlayerSeatReassignment)
    );
}

#[test]
fn authority_digests_are_pinned_to_independent_literals() {
    let row = player_authority();
    let ledger = PracticeInputAuthorityLedger {
        schema_version: 2,
        rows: vec![row.clone()],
    };
    assert_eq!(
        input_authority_digest(&row).unwrap(),
        hex_digest("e7ef0883ac5adbf5f6a7424e820676327c34c280341bb8ab4b4ae015308e7d85")
    );
    assert_eq!(
        input_authority_ledger_digest(&ledger).unwrap(),
        hex_digest("3415c8298f3a78e53fe3660ac453544b43f8be32dc12071928bb2b8c3782908a")
    );
}

#[test]
fn authority_error_codes_are_closed_and_exact() {
    let expected = [
        (PracticeAuthorityError::AuthorityDomain, 1_u16),
        (PracticeAuthorityError::AuthoritySchemaVersion, 2),
        (PracticeAuthorityError::AuthorityEnumCode, 3),
        (PracticeAuthorityError::AuthorityTruncated, 4),
        (PracticeAuthorityError::AuthorityTrailingBytes, 5),
        (PracticeAuthorityError::AuthorityEmptyInterval, 6),
        (PracticeAuthorityError::AuthorityLedgerLimit, 7),
        (PracticeAuthorityError::AuthorityLedgerOrder, 8),
        (PracticeAuthorityError::AuthorityLedgerDuplicate, 9),
        (PracticeAuthorityError::AuthorityIntervalOverlap, 10),
        (PracticeAuthorityError::AuthorityPlayerSeatOverlap, 11),
        (PracticeAuthorityError::AuthorityNotFound, 12),
        (PracticeAuthorityError::AuthorityInactive, 13),
        (PracticeAuthorityError::AuthorityActorMismatch, 14),
        (PracticeAuthorityError::AuthorityPlayerSeatMissing, 15),
        (PracticeAuthorityError::AuthorityPlayerSeatReassignment, 16),
    ];
    for (error, code) in expected {
        assert_eq!(u16::from(error), code);
        assert_eq!(PracticeAuthorityError::try_from(code), Ok(error));
    }
    assert!(PracticeAuthorityError::try_from(0_u16).is_err());
    assert!(PracticeAuthorityError::try_from(17_u16).is_err());
}

#[test]
fn authority_ledger_accepts_16384_rows_and_refuses_16385_before_sorting() {
    let rows: Vec<_> = (0_u128..16_384)
        .map(|index| PracticeInputAuthority {
            input_authority_id: InputAuthorityId::from_bytes(index.to_be_bytes()),
            ..policy_authority(0x21, 8, 10, 20)
        })
        .collect();
    assert_eq!(
        validate_input_authority_ledger(&PracticeInputAuthorityLedger {
            schema_version: 2,
            rows: rows.clone(),
        }),
        Ok(())
    );
    let mut too_many = rows;
    too_many.push(policy_authority(0xff, 8, 10, 20));
    assert_eq!(
        validate_input_authority_ledger(&PracticeInputAuthorityLedger {
            schema_version: 2,
            rows: too_many,
        }),
        Err(PracticeAuthorityError::AuthorityLedgerLimit)
    );
}
