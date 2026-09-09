use babylon_kernel::tick_content_hash::TickContentHashV1;
use babylon_persistence::committed_tick_envelope::{
    validate_committed_tick_envelope_bounds_v2, CommittedTickEnvelopeConflictV2,
    CommittedTickEnvelopeErrorV2, CommittedTickEnvelopeRetryV2, CommittedTickEnvelopeV2,
    CommittedTickRowFamiliesV2, CommittedTickRowFamilyV2, CommittedTickRowV2,
    ALL_COMMITTED_TICK_ROW_FAMILIES_V2, COMMITTED_TICK_ENVELOPE_LAYOUT_VERSION_V2,
    COMMITTED_TICK_ROW_FAMILY_COUNT_V2, MAX_COMMITTED_TICK_ENVELOPE_BYTES_V2,
    MAX_COMMITTED_TICK_ROWS_V2, MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2,
};
use babylon_persistence::identity::CampaignId;
use babylon_persistence::tick_commit_claim::{TickCommitClaimConflictV1, TickCommitClaimV1};
use uuid::Uuid;

const DOMAIN_V2: &[u8] = b"babylon.committed-tick-envelope.v2\0";

fn claim(campaign: u128, tick: u64, content: u8) -> TickCommitClaimV1 {
    TickCommitClaimV1::compose(
        CampaignId::from_uuid(Uuid::from_u128(campaign)),
        tick,
        TickContentHashV1::from_bytes([content; 32]),
    )
}

fn row(key: u8, payload: u8) -> CommittedTickRowV2 {
    CommittedTickRowV2::compose(vec![key], vec![payload]).expect("bounded canonical row")
}

fn singleton_families(payload: u8) -> CommittedTickRowFamiliesV2 {
    CommittedTickRowFamiliesV2 {
        graph: vec![row(0x01, payload)],
        state: vec![row(0x02, payload)],
        event: vec![row(0x03, payload)],
        choice_receipt: vec![row(0x04, payload)],
        checkpoint: vec![row(0x05, payload)],
        archive_dirty_receipt: row(0x06, payload),
    }
}

fn sparse_families(payload: u8) -> CommittedTickRowFamiliesV2 {
    CommittedTickRowFamiliesV2 {
        graph: Vec::new(),
        state: Vec::new(),
        event: Vec::new(),
        choice_receipt: Vec::new(),
        checkpoint: Vec::new(),
        archive_dirty_receipt: row(0x17, payload),
    }
}

fn mutate_family(families: &mut CommittedTickRowFamiliesV2, family: CommittedTickRowFamilyV2) {
    let replacement = vec![row(0x01, 0xff)];
    match family {
        CommittedTickRowFamilyV2::Graph => families.graph = replacement,
        CommittedTickRowFamilyV2::State => families.state = replacement,
        CommittedTickRowFamilyV2::Event => families.event = replacement,
        CommittedTickRowFamilyV2::ChoiceReceipt => families.choice_receipt = replacement,
        CommittedTickRowFamilyV2::Checkpoint => families.checkpoint = replacement,
        CommittedTickRowFamilyV2::ArchiveDirtyReceipt => {
            families.archive_dirty_receipt = row(0x17, 0xff);
        }
    }
}

fn append_u32(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(
        &u32::try_from(value)
            .expect("test length fits u32")
            .to_be_bytes(),
    );
}

fn expected_singleton_bytes(claim: TickCommitClaimV1, payload: u8) -> Vec<u8> {
    let families = [
        (0x10, 0x01),
        (0x11, 0x02),
        (0x12, 0x03),
        (0x18, 0x04),
        (0x16, 0x05),
        (0x17, 0x06),
    ];
    let mut expected = Vec::new();
    expected.extend_from_slice(DOMAIN_V2);
    expected.extend_from_slice(&COMMITTED_TICK_ENVELOPE_LAYOUT_VERSION_V2.to_be_bytes());
    expected.push(0x01);
    append_u32(&mut expected, claim.canonical_bytes().len());
    expected.extend_from_slice(claim.canonical_bytes());
    for (family_tag, key) in families {
        expected.push(family_tag);
        append_u32(&mut expected, 1);
        append_u32(&mut expected, 10);
        append_u32(&mut expected, 1);
        expected.push(key);
        append_u32(&mut expected, 1);
        expected.push(payload);
    }
    expected
}

#[test]
fn six_families_have_one_closed_order_with_choice_before_checkpoint() {
    assert_eq!(COMMITTED_TICK_ROW_FAMILY_COUNT_V2, 6);
    assert_eq!(
        ALL_COMMITTED_TICK_ROW_FAMILIES_V2,
        [
            CommittedTickRowFamilyV2::Graph,
            CommittedTickRowFamilyV2::State,
            CommittedTickRowFamilyV2::Event,
            CommittedTickRowFamilyV2::ChoiceReceipt,
            CommittedTickRowFamilyV2::Checkpoint,
            CommittedTickRowFamilyV2::ArchiveDirtyReceipt,
        ]
    );
    assert_eq!(
        ALL_COMMITTED_TICK_ROW_FAMILIES_V2.map(CommittedTickRowFamilyV2::tag),
        [0x10, 0x11, 0x12, 0x18, 0x16, 0x17]
    );
    assert_eq!(
        ALL_COMMITTED_TICK_ROW_FAMILIES_V2.map(CommittedTickRowFamilyV2::name),
        [
            "graph",
            "state",
            "event",
            "choice_receipt",
            "checkpoint",
            "archive_dirty_receipt",
        ]
    );
}

#[test]
fn sparse_tick_keeps_all_six_sections_and_one_archive_receipt() {
    let envelope = CommittedTickEnvelopeV2::compose(
        claim(0x0011_2233_4455_6677_8899_aabb_ccdd_eeff, 42, 0x11),
        sparse_families(0x11),
    )
    .expect("sparse V2 envelope");

    assert_eq!(envelope.total_rows(), 1);
    assert_eq!(envelope.row_families().len(), 6);
    assert_eq!(
        envelope.row_families()[3].family(),
        CommittedTickRowFamilyV2::ChoiceReceipt
    );
    assert!(envelope.canonical_bytes().len() < MAX_COMMITTED_TICK_ENVELOPE_BYTES_V2);
}

#[test]
fn canonical_bytes_pin_v2_domain_layout_and_non_numeric_family_order() {
    let claim = claim(0x0011_2233_4455_6677_8899_aabb_ccdd_eeff, 42, 0x11);
    let envelope =
        CommittedTickEnvelopeV2::compose(claim, singleton_families(0xaa)).expect("V2 envelope");

    assert_eq!(COMMITTED_TICK_ENVELOPE_LAYOUT_VERSION_V2, 2);
    assert_eq!(
        envelope.canonical_bytes(),
        expected_singleton_bytes(claim, 0xaa)
    );
}

#[test]
fn every_mandatory_family_moves_whole_payload_identity() {
    let base = CommittedTickEnvelopeV2::compose(claim(1, 42, 0x11), singleton_families(0xaa))
        .expect("base envelope");

    for family in ALL_COMMITTED_TICK_ROW_FAMILIES_V2 {
        let mut mutated_rows = singleton_families(0xaa);
        mutate_family(&mut mutated_rows, family);
        let mutated = CommittedTickEnvelopeV2::compose(claim(1, 42, 0x11), mutated_rows)
            .expect("mutated envelope");
        assert_ne!(
            base.canonical_bytes(),
            mutated.canonical_bytes(),
            "{family:?}"
        );
        assert_ne!(base.digest(), mutated.digest(), "{family:?}");
    }
}

#[test]
fn retry_requires_exact_v2_whole_payload_bytes() {
    let existing = CommittedTickEnvelopeV2::compose(claim(1, 42, 0x11), singleton_families(0xaa))
        .expect("existing envelope");
    let identical = CommittedTickEnvelopeV2::compose(claim(1, 42, 0x11), singleton_families(0xaa))
        .expect("identical retry");
    let payload_conflict =
        CommittedTickEnvelopeV2::compose(claim(1, 42, 0x11), singleton_families(0xbb))
            .expect("payload conflict");
    let content_conflict =
        CommittedTickEnvelopeV2::compose(claim(1, 42, 0x22), singleton_families(0xaa))
            .expect("content conflict");
    let key_conflict =
        CommittedTickEnvelopeV2::compose(claim(1, 43, 0x11), singleton_families(0xaa))
            .expect("key conflict");

    assert_eq!(
        identical.classify_retry_against(&existing),
        Ok(CommittedTickEnvelopeRetryV2::Idempotent)
    );
    assert!(matches!(
        payload_conflict.classify_retry_against(&existing),
        Err(CommittedTickEnvelopeConflictV2::WholePayloadMismatch { .. })
    ));
    assert!(matches!(
        content_conflict.classify_retry_against(&existing),
        Err(CommittedTickEnvelopeConflictV2::Claim(
            TickCommitClaimConflictV1::ContentIdentityMismatch { .. }
        ))
    ));
    assert!(matches!(
        key_conflict.classify_retry_against(&existing),
        Err(CommittedTickEnvelopeConflictV2::Claim(
            TickCommitClaimConflictV1::KeyMismatch { .. }
        ))
    ));
}

#[test]
fn row_keys_are_nonempty_unique_and_strictly_ordered_in_choice_family() {
    assert!(matches!(
        CommittedTickRowV2::compose(Vec::new(), vec![1]),
        Err(CommittedTickEnvelopeErrorV2::EmptyRowKey)
    ));

    let mut duplicate = singleton_families(1);
    duplicate.choice_receipt = vec![row(1, 1), row(1, 2)];
    assert!(matches!(
        CommittedTickEnvelopeV2::compose(claim(1, 1, 1), duplicate),
        Err(CommittedTickEnvelopeErrorV2::DuplicateRowKey {
            family: CommittedTickRowFamilyV2::ChoiceReceipt,
            ..
        })
    ));

    let mut descending = singleton_families(1);
    descending.choice_receipt = vec![row(2, 1), row(1, 2)];
    assert!(matches!(
        CommittedTickEnvelopeV2::compose(claim(1, 1, 1), descending),
        Err(CommittedTickEnvelopeErrorV2::RowOrder {
            family: CommittedTickRowFamilyV2::ChoiceReceipt,
            ..
        })
    ));
}

#[test]
fn cumulative_bounds_cover_six_families_and_singular_archive() {
    assert_eq!(
        validate_committed_tick_envelope_bounds_v2(
            [1; 6],
            [MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2; 6],
        )
        .expect("exact byte maximum"),
        MAX_COMMITTED_TICK_ENVELOPE_BYTES_V2
    );
    assert!(matches!(
        validate_committed_tick_envelope_bounds_v2(
            [1; 6],
            [MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2 + 1; 6],
        ),
        Err(CommittedTickEnvelopeErrorV2::BatchBytes { .. })
    ));

    let mut maximum_rows = [0_usize; 6];
    maximum_rows[0] = MAX_COMMITTED_TICK_ROWS_V2 - 1;
    maximum_rows[5] = 1;
    let mut minimum_bodies = [0_usize; 6];
    minimum_bodies[0] = (MAX_COMMITTED_TICK_ROWS_V2 - 1) * 9;
    minimum_bodies[5] = 9;
    assert!(validate_committed_tick_envelope_bounds_v2(maximum_rows, minimum_bodies).is_ok());
    maximum_rows[3] = 1;
    minimum_bodies[3] = 9;
    assert!(matches!(
        validate_committed_tick_envelope_bounds_v2(maximum_rows, minimum_bodies),
        Err(CommittedTickEnvelopeErrorV2::AggregateRows { .. })
    ));

    assert!(matches!(
        validate_committed_tick_envelope_bounds_v2([2, 0, 0, 0, 0, 1], [9, 0, 0, 0, 0, 9]),
        Err(CommittedTickEnvelopeErrorV2::BatchShape {
            family: CommittedTickRowFamilyV2::Graph,
            ..
        })
    ));
    assert!(matches!(
        validate_committed_tick_envelope_bounds_v2([0; 6], [0; 6]),
        Err(CommittedTickEnvelopeErrorV2::MissingArchiveDirtyReceipt)
    ));
    assert!(matches!(
        validate_committed_tick_envelope_bounds_v2([0, 0, 0, 0, 0, 2], [0, 0, 0, 0, 0, 18]),
        Err(CommittedTickEnvelopeErrorV2::DuplicateArchiveDirtyReceipt { actual: 2 })
    ));
}

#[test]
fn envelope_module_contains_no_live_v1_surface() {
    let source = include_str!("../src/committed_tick_envelope.rs");
    for forbidden in [
        "CommittedTickEnvelopeV1",
        "CommittedTickRowFamiliesV1",
        "CommittedTickRowFamilyV1",
        "CommittedTickRowV1",
        "validate_committed_tick_envelope_bounds_v1",
        "babylon.committed-tick-envelope.v1",
    ] {
        assert!(
            !source.contains(forbidden),
            "live V1 surface remains: {forbidden}"
        );
    }
}
