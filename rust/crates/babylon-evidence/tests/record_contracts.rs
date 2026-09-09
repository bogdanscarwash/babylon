//! Language-neutral contracts for the six frozen T3 run-record envelopes.

use babylon_evidence::{
    canonical_envelope, decode_envelope, practice_attempt_row_id, record_digest, Digest32,
    PracticeAttemptLedgerV1, PracticeAttemptRowV1, PracticeCandidateRowV1,
    PracticeCandidateScheduleV1, PracticeDispositionV1, RunIdentityField, RunIdentityV1,
    SfsPreregistrationV1, SfsRecordError, SfsSampleV1, SfsTraceV1, SfsWireError, T3Record,
};
use babylon_kernel::{sha256_of, SessionId};
use babylon_practice_contract::PracticeIdV1;

fn digest(tag: u8) -> Digest32 {
    let mut bytes = [0_u8; 32];
    bytes[0] = tag;
    bytes[31] = tag ^ 0xff;
    Digest32::from_bytes(bytes)
}

fn literal_envelope(domain: &[u8], payload: &[u8]) -> Vec<u8> {
    let length = u32::try_from(payload.len()).expect("test payload length fits u32");
    let mut bytes = Vec::new();
    bytes.extend_from_slice(domain);
    bytes.push(0);
    bytes.extend_from_slice(&1_u16.to_be_bytes());
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes.extend_from_slice(payload);
    bytes
}

fn payload_start<T: T3Record>() -> usize {
    T::DOMAIN.len() + 7
}

fn payload_length<T: T3Record>(envelope: &[u8]) -> usize {
    let start = T::DOMAIN.len() + 3;
    usize::try_from(u32::from_be_bytes(
        envelope[start..start + 4]
            .try_into()
            .expect("envelope length field is four bytes"),
    ))
    .expect("u32 fits usize")
}

fn changed_digest(
    mutation: Option<RunIdentityField>,
    field: RunIdentityField,
    tag: u8,
) -> Digest32 {
    if mutation == Some(field) {
        digest(tag + 64)
    } else {
        digest(tag)
    }
}

fn run_identity(mutation: Option<RunIdentityField>) -> RunIdentityV1 {
    let session = if mutation == Some(RunIdentityField::Session) {
        SessionId::new("mutated-session").unwrap()
    } else {
        SessionId::new("r").unwrap()
    };
    let rng = if mutation == Some(RunIdentityField::RngAlgorithmId) {
        "rng-mutated"
    } else {
        "x"
    };
    let graph = if mutation == Some(RunIdentityField::GraphContractId) {
        "graph-mutated"
    } else {
        "y"
    };
    RunIdentityV1::new(
        session,
        changed_digest(mutation, RunIdentityField::Scenario, 1),
        changed_digest(mutation, RunIdentityField::PreludeDeclarations, 2),
        changed_digest(mutation, RunIdentityField::Vocabulary, 3),
        changed_digest(mutation, RunIdentityField::RuleAst, 4),
        changed_digest(mutation, RunIdentityField::HostComponentManifest, 5),
        changed_digest(mutation, RunIdentityField::Defines, 6),
        changed_digest(mutation, RunIdentityField::IntrinsicCostCap, 7),
        changed_digest(mutation, RunIdentityField::ReferenceManifest, 8),
        changed_digest(mutation, RunIdentityField::GovernedFootprintManifest, 9),
        changed_digest(mutation, RunIdentityField::SfsProofProfile, 10),
        changed_digest(mutation, RunIdentityField::SfsPreregistration, 11),
        changed_digest(mutation, RunIdentityField::InitialCommittedEnvelope, 12),
        changed_digest(mutation, RunIdentityField::InitialNominalWorld, 13),
        changed_digest(mutation, RunIdentityField::ExogenousInputLedger, 14),
        changed_digest(mutation, RunIdentityField::PracticeAttemptLedger, 15),
        rng,
        graph,
    )
    .unwrap()
}

fn run_with_strings(
    session: SessionId,
    rng: &str,
    graph: &str,
) -> Result<RunIdentityV1, SfsRecordError> {
    RunIdentityV1::new(
        session,
        digest(1),
        digest(2),
        digest(3),
        digest(4),
        digest(5),
        digest(6),
        digest(7),
        digest(8),
        digest(9),
        digest(10),
        digest(11),
        digest(12),
        digest(13),
        digest(14),
        digest(15),
        rng,
        graph,
    )
}

fn expected_run_payload() -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&1_u16.to_be_bytes());
    payload.push(b'r');
    for tag in 1..=15 {
        payload.extend_from_slice(digest(tag).as_bytes());
    }
    payload.extend_from_slice(&1_u16.to_be_bytes());
    payload.push(b'x');
    payload.extend_from_slice(&1_u16.to_be_bytes());
    payload.push(b'y');
    payload
}

fn literal_sample_envelope(tick: u64, aggregate: f64) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&tick.to_be_bytes());
    payload.extend_from_slice(digest(20).as_bytes());
    payload.extend_from_slice(digest(21).as_bytes());
    payload.extend_from_slice(digest(22).as_bytes());
    payload.extend_from_slice(&aggregate.to_bits().to_be_bytes());
    literal_envelope(SfsSampleV1::DOMAIN, &payload)
}

fn literal_trace_envelope() -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(digest(40).as_bytes());
    payload.extend_from_slice(digest(41).as_bytes());
    payload.extend_from_slice(&42_u64.to_be_bytes());
    payload.extend_from_slice(&10_u64.to_be_bytes());
    payload.extend_from_slice(&1_u16.to_be_bytes());
    payload.extend_from_slice(&2_u16.to_be_bytes());
    payload.extend_from_slice(&7_u16.to_be_bytes());
    payload.extend_from_slice(&literal_sample_envelope(10, 0.0));
    payload.extend_from_slice(&literal_sample_envelope(11, 1.0));
    payload.extend_from_slice(&literal_sample_envelope(12, 2.0));
    payload.extend_from_slice(&literal_sample_envelope(13, 5.0));
    payload.extend_from_slice(&literal_sample_envelope(14, 8.0));
    payload.extend_from_slice(&literal_sample_envelope(15, 10.0));
    payload.extend_from_slice(&literal_sample_envelope(16, 11.0));
    payload.push(2);
    literal_envelope(SfsTraceV1::DOMAIN, &payload)
}

fn sample(tick: u64, aggregate: f64) -> SfsSampleV1 {
    SfsSampleV1::new(tick, digest(20), digest(21), digest(22), aggregate).unwrap()
}

fn continuing_samples(start_tick: u64) -> Vec<SfsSampleV1> {
    let masses = [0.0, 1.0, 2.0, 5.0, 8.0, 10.0, 11.0];
    let mut samples = Vec::with_capacity(7);
    #[allow(clippy::needless_range_loop)] // The literal ceiling is statically auditable.
    for offset in 0..7 {
        let tick_offset = u64::try_from(offset).expect("seven offsets fit u64");
        samples.push(sample(start_tick + tick_offset, masses[offset]));
    }
    samples
}

fn candidate(tick: u64, authority_tag: u8, intent_tag: u8) -> PracticeCandidateRowV1 {
    PracticeCandidateRowV1::new(tick, digest(authority_tag), digest(intent_tag))
}

fn preregistration(
    preregistered_at_tick: u64,
    stride: u16,
    practice: PracticeIdV1,
) -> Result<SfsPreregistrationV1, SfsRecordError> {
    SfsPreregistrationV1::new(
        preregistered_at_tick,
        digest(30),
        digest(31),
        digest(32),
        digest(33),
        digest(34),
        digest(35),
        20,
        stride,
        3,
        practice,
        digest(36),
        40,
        digest(37),
    )
}

#[test]
fn record_domains_and_payload_maxima_are_exact() {
    assert_eq!(RunIdentityV1::DOMAIN, b"babylon.run-identity.v1");
    assert_eq!(RunIdentityV1::MAX_PAYLOAD_BYTES, 870);
    assert_eq!(SfsSampleV1::DOMAIN, b"babylon.sfs-sample.v1");
    assert_eq!(SfsSampleV1::MAX_PAYLOAD_BYTES, 112);
    assert_eq!(SfsTraceV1::DOMAIN, b"babylon.sfs-trace.v1");
    assert_eq!(SfsTraceV1::MAX_PAYLOAD_BYTES, 22_067);
    assert_eq!(
        SfsPreregistrationV1::DOMAIN,
        b"babylon.sfs-preregistration.v1"
    );
    assert_eq!(SfsPreregistrationV1::MAX_PAYLOAD_BYTES, 291);
    assert_eq!(
        PracticeCandidateScheduleV1::DOMAIN,
        b"babylon.practice-candidate-schedule.v1"
    );
    assert_eq!(PracticeCandidateScheduleV1::MAX_PAYLOAD_BYTES, 6_815_644);
    assert_eq!(
        PracticeAttemptLedgerV1::DOMAIN,
        b"babylon.practice-attempt-ledger.v1"
    );
    assert_eq!(PracticeAttemptLedgerV1::MAX_PAYLOAD_BYTES, 8_978_331);
}

#[test]
fn run_identity_encodes_all_eighteen_fields_in_exact_order() {
    let run = run_identity(None);
    let expected = literal_envelope(RunIdentityV1::DOMAIN, &expected_run_payload());
    assert_eq!(canonical_envelope(&run).unwrap(), expected);
    assert_eq!(decode_envelope::<RunIdentityV1>(&expected).unwrap(), run);
    assert_eq!(run.host_component_manifest_digest(), digest(5));
    assert_eq!(run.governed_footprint_manifest_digest(), digest(9));
    assert_eq!(run.sfs_proof_profile_digest(), digest(10));
    assert_eq!(run.sfs_preregistration_digest(), digest(11));
    assert_eq!(run.exogenous_input_ledger_digest(), digest(14));
    assert_eq!(run.practice_attempt_ledger_digest(), digest(15));

    let fields = [
        RunIdentityField::Session,
        RunIdentityField::Scenario,
        RunIdentityField::PreludeDeclarations,
        RunIdentityField::Vocabulary,
        RunIdentityField::RuleAst,
        RunIdentityField::HostComponentManifest,
        RunIdentityField::Defines,
        RunIdentityField::IntrinsicCostCap,
        RunIdentityField::ReferenceManifest,
        RunIdentityField::GovernedFootprintManifest,
        RunIdentityField::SfsProofProfile,
        RunIdentityField::SfsPreregistration,
        RunIdentityField::InitialCommittedEnvelope,
        RunIdentityField::InitialNominalWorld,
        RunIdentityField::ExogenousInputLedger,
        RunIdentityField::PracticeAttemptLedger,
        RunIdentityField::RngAlgorithmId,
        RunIdentityField::GraphContractId,
    ];
    let baseline_bytes = canonical_envelope(&run).unwrap();
    let baseline_digest = record_digest(&run).unwrap();
    for field in fields {
        let mutated = run_identity(Some(field));
        assert_eq!(run.differing_fields(&mutated), vec![field]);
        assert_ne!(canonical_envelope(&mutated).unwrap(), baseline_bytes);
        assert_ne!(record_digest(&mutated).unwrap(), baseline_digest);
    }
    let all_different = RunIdentityV1::new(
        SessionId::new("all-mutated").unwrap(),
        digest(65),
        digest(66),
        digest(67),
        digest(68),
        digest(69),
        digest(70),
        digest(71),
        digest(72),
        digest(73),
        digest(74),
        digest(75),
        digest(76),
        digest(77),
        digest(78),
        digest(79),
        "rng-all-mutated",
        "graph-all-mutated",
    )
    .unwrap();
    assert_eq!(run.differing_fields(&all_different), fields.to_vec());
}

#[test]
fn run_identity_session_and_ascii_id_bounds_are_closed() {
    let one = run_with_strings(SessionId::new("s").unwrap(), "r", "g").unwrap();
    assert_eq!(
        decode_envelope::<RunIdentityV1>(&canonical_envelope(&one).unwrap()).unwrap(),
        one
    );
    let maximum = run_with_strings(
        SessionId::new("s".repeat(256)).unwrap(),
        &"r".repeat(64),
        &"g".repeat(64),
    )
    .unwrap();
    let maximum_bytes = canonical_envelope(&maximum).unwrap();
    assert_eq!(payload_length::<RunIdentityV1>(&maximum_bytes), 870);
    assert_eq!(
        decode_envelope::<RunIdentityV1>(&maximum_bytes).unwrap(),
        maximum
    );
    assert_eq!(
        run_with_strings(SessionId::new("s".repeat(257)).unwrap(), "r", "g"),
        Err(SfsRecordError::InvalidSessionLength { actual: 257 })
    );
    assert_eq!(
        run_with_strings(SessionId::new("cafe\u{301}").unwrap(), "r", "g"),
        Err(SfsRecordError::Wire(SfsWireError::NonNfc {
            field: "session"
        }))
    );
    assert_eq!(
        run_with_strings(SessionId::new("s").unwrap(), "", "g"),
        Err(SfsRecordError::Wire(SfsWireError::StringEmpty {
            field: "rng_algorithm_id"
        }))
    );
    assert_eq!(
        run_with_strings(SessionId::new("s").unwrap(), &"r".repeat(65), "g"),
        Err(SfsRecordError::Wire(SfsWireError::StringTooLong {
            field: "rng_algorithm_id",
            limit: 64,
            actual: 65,
        }))
    );
    assert_eq!(
        run_with_strings(SessionId::new("s").unwrap(), "r", "gráph"),
        Err(SfsRecordError::Wire(SfsWireError::NonAscii {
            field: "graph_contract_id"
        }))
    );
}

#[test]
fn sample_wire_is_fixed_and_aggregate_validation_is_exact() {
    let value = sample(7, -0.0);
    let mut payload = Vec::new();
    payload.extend_from_slice(&7_u64.to_be_bytes());
    payload.extend_from_slice(digest(20).as_bytes());
    payload.extend_from_slice(digest(21).as_bytes());
    payload.extend_from_slice(digest(22).as_bytes());
    payload.extend_from_slice(&0_u64.to_be_bytes());
    let expected = literal_envelope(SfsSampleV1::DOMAIN, &payload);
    assert_eq!(canonical_envelope(&value).unwrap(), expected);
    assert_eq!(decode_envelope::<SfsSampleV1>(&expected).unwrap(), value);
    assert_eq!(
        SfsSampleV1::new(0, digest(1), digest(2), digest(3), -1.0),
        Err(SfsRecordError::Wire(SfsWireError::Negative {
            field: "aggregate"
        }))
    );
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            SfsSampleV1::new(0, digest(1), digest(2), digest(3), value),
            Err(SfsRecordError::Wire(SfsWireError::NonFinite {
                field: "aggregate"
            }))
        );
    }
}

#[test]
fn trace_requires_exact_window_count_start_and_consecutive_ticks() {
    let valid = SfsTraceV1::new(digest(40), digest(41), 42, 10, 2, continuing_samples(10)).unwrap();
    let encoded = canonical_envelope(&valid).unwrap();
    assert_eq!(encoded.last(), Some(&(2_u8)));
    assert_eq!(decode_envelope::<SfsTraceV1>(&encoded).unwrap(), valid);
    assert_eq!(valid.run_identity_digest(), digest(40));
    assert_eq!(
        SfsTraceV1::new(digest(1), digest(2), 3, 10, 1, continuing_samples(10)),
        Err(SfsRecordError::InvalidWindowWidth { found: 1 })
    );
    assert_eq!(
        SfsTraceV1::new(
            digest(1),
            digest(2),
            3,
            10,
            2,
            continuing_samples(10)[..6].to_vec()
        ),
        Err(SfsRecordError::WrongSampleCount {
            expected: 7,
            actual: 6,
        })
    );
    let mut wrong_start = continuing_samples(11);
    assert_eq!(
        SfsTraceV1::new(digest(1), digest(2), 3, 10, 2, wrong_start.clone()),
        Err(SfsRecordError::TickDiscontinuity {
            expected: 10,
            actual: 11,
        })
    );
    wrong_start[3] = sample(99, 5.0);
    assert_eq!(
        SfsTraceV1::new(digest(1), digest(2), 3, 11, 2, wrong_start),
        Err(SfsRecordError::TickDiscontinuity {
            expected: 14,
            actual: 99,
        })
    );
    let mut overflow = continuing_samples(0);
    overflow[0] = sample(u64::MAX, 0.0);
    assert_eq!(
        SfsTraceV1::new(digest(1), digest(2), 3, u64::MAX, 2, overflow),
        Err(SfsRecordError::ArithmeticOverflow {
            field: "sample_tick"
        })
    );
}

#[test]
fn trace_wire_is_one_complete_literal_envelope_with_nested_sample_envelopes() {
    let trace = SfsTraceV1::new(digest(40), digest(41), 42, 10, 2, continuing_samples(10)).unwrap();
    let expected = literal_trace_envelope();
    assert_eq!(canonical_envelope(&trace).unwrap(), expected);
    assert_eq!(decode_envelope::<SfsTraceV1>(&expected).unwrap(), trace);
}

#[test]
fn trace_decode_refusal_precedence_pins_interval_and_classification() {
    let trace = SfsTraceV1::new(digest(40), digest(41), 42, 10, 2, continuing_samples(10)).unwrap();
    let encoded = canonical_envelope(&trace).unwrap();
    let mut invalid_code = encoded.clone();
    *invalid_code.last_mut().unwrap() = 6;
    assert_eq!(
        decode_envelope::<SfsTraceV1>(&invalid_code),
        Err(SfsRecordError::Wire(SfsWireError::InvalidCode {
            field: "classification",
            value: 6,
        }))
    );
    let mut wrong_valid_class = encoded.clone();
    *wrong_valid_class.last_mut().unwrap() = 0;
    assert_eq!(
        decode_envelope::<SfsTraceV1>(&wrong_valid_class),
        Err(SfsRecordError::ClassificationMismatch {
            stored: 0,
            computed: 2,
        })
    );
    let mut wrong_interval = encoded;
    let interval = payload_start::<SfsTraceV1>() + 80;
    wrong_interval[interval..interval + 2].copy_from_slice(&2_u16.to_be_bytes());
    assert_eq!(
        decode_envelope::<SfsTraceV1>(&wrong_interval),
        Err(SfsRecordError::InvalidSampleInterval { found: 2 })
    );
}

#[test]
fn trace_maximum_of_157_complete_sample_envelopes_round_trips() {
    let mut samples = Vec::with_capacity(157);
    for tick in 0..157 {
        samples.push(sample(tick, 0.0));
    }
    let trace = SfsTraceV1::new(digest(1), digest(2), 3, 0, 52, samples).unwrap();
    let envelope = canonical_envelope(&trace).unwrap();
    assert_eq!(payload_length::<SfsTraceV1>(&envelope), 22_067);
    assert_eq!(decode_envelope::<SfsTraceV1>(&envelope).unwrap(), trace);
}

#[test]
fn candidate_row_id_uses_the_exact_preimage_and_changes_with_every_field() {
    let authority = digest(50);
    let intent = digest(51);
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"babylon.practice-attempt-row.v1");
    preimage.push(0);
    preimage.extend_from_slice(&7_u64.to_be_bytes());
    preimage.extend_from_slice(authority.as_bytes());
    preimage.extend_from_slice(intent.as_bytes());
    let expected = Digest32::from_bytes(sha256_of(&preimage));
    assert_eq!(practice_attempt_row_id(7, authority, intent), expected);
    assert_ne!(practice_attempt_row_id(8, authority, intent), expected);
    assert_ne!(practice_attempt_row_id(7, digest(52), intent), expected);
    assert_ne!(practice_attempt_row_id(7, authority, digest(53)), expected);
}

#[test]
fn candidate_schedule_sorts_exact_rows_and_rejects_duplicates_or_bad_ids() {
    let later = candidate(2, 1, 2);
    let same_tick_a = candidate(1, 3, 4);
    let same_tick_b = candidate(1, 3, 5);
    let expected_first_intent = if practice_attempt_row_id(1, digest(3), digest(4))
        < practice_attempt_row_id(1, digest(3), digest(5))
    {
        digest(4)
    } else {
        digest(5)
    };
    let schedule = PracticeCandidateScheduleV1::new(vec![later, same_tick_b, same_tick_a]).unwrap();
    assert_eq!(schedule.rows()[0].attempt_tick(), 1);
    assert_eq!(
        schedule.rows()[0].practice_intent_digest(),
        expected_first_intent
    );
    assert_eq!(schedule.rows()[2].attempt_tick(), 2);
    let round_trip = canonical_envelope(&schedule).unwrap();
    assert_eq!(
        decode_envelope::<PracticeCandidateScheduleV1>(&round_trip).unwrap(),
        schedule
    );
    let duplicate = candidate(1, 3, 4);
    assert_eq!(
        PracticeCandidateScheduleV1::new(vec![duplicate.clone(), duplicate]),
        Err(SfsRecordError::Wire(SfsWireError::DuplicateEntry {
            field: "candidate_rows"
        }))
    );

    let one = candidate(7, 8, 9);
    let one_schedule = PracticeCandidateScheduleV1::new(vec![one.clone()]).unwrap();
    let mut expected_payload = Vec::new();
    expected_payload.extend_from_slice(&1_u32.to_be_bytes());
    expected_payload.extend_from_slice(practice_attempt_row_id(7, digest(8), digest(9)).as_bytes());
    expected_payload.extend_from_slice(&7_u64.to_be_bytes());
    expected_payload.extend_from_slice(digest(8).as_bytes());
    expected_payload.extend_from_slice(digest(9).as_bytes());
    let expected = literal_envelope(PracticeCandidateScheduleV1::DOMAIN, &expected_payload);
    assert_eq!(canonical_envelope(&one_schedule).unwrap(), expected);
    let mut bad_id = expected;
    bad_id[payload_start::<PracticeCandidateScheduleV1>() + 4] ^= 0xff;
    assert_eq!(
        decode_envelope::<PracticeCandidateScheduleV1>(&bad_id),
        Err(SfsRecordError::StableRowDigestMismatch)
    );
}

#[test]
fn candidate_schedule_literal_malformed_payloads_refuse_before_rows_or_trailing_bytes() {
    let too_many = literal_envelope(
        PracticeCandidateScheduleV1::DOMAIN,
        &65_536_u32.to_be_bytes(),
    );
    assert_eq!(
        decode_envelope::<PracticeCandidateScheduleV1>(&too_many),
        Err(SfsRecordError::Wire(SfsWireError::CountTooLarge {
            field: "candidate_rows",
            limit: 65_535,
            actual: 65_536,
        }))
    );

    let truncated = literal_envelope(PracticeCandidateScheduleV1::DOMAIN, &1_u32.to_be_bytes());
    assert_eq!(
        decode_envelope::<PracticeCandidateScheduleV1>(&truncated),
        Err(SfsRecordError::Wire(SfsWireError::TruncatedEnvelope))
    );

    let mut trailing_payload = 0_u32.to_be_bytes().to_vec();
    trailing_payload.push(0xaa);
    let trailing = literal_envelope(PracticeCandidateScheduleV1::DOMAIN, &trailing_payload);
    assert_eq!(
        decode_envelope::<PracticeCandidateScheduleV1>(&trailing),
        Err(SfsRecordError::Wire(SfsWireError::TrailingBytes {
            count: 1,
        }))
    );
}

#[test]
fn preregistration_derives_start_and_closes_codes_and_accessors() {
    for practice in [
        PracticeIdV1::Organize,
        PracticeIdV1::Agitate,
        PracticeIdV1::MutualAid,
    ] {
        let value = preregistration(10, 2, practice).unwrap();
        let bytes = canonical_envelope(&value).unwrap();
        assert_eq!(
            decode_envelope::<SfsPreregistrationV1>(&bytes).unwrap(),
            value
        );
        assert_eq!(value.practice_candidate_schedule_digest(), digest(31));
        assert_eq!(value.sfs_proof_profile_digest(), digest(32));
        assert_eq!(value.driver_contract_digest(), digest(33));
        assert_eq!(value.mutation_manifest_digest(), digest(34));
        assert_eq!(value.expected_exogenous_ledger_digest(), digest(35));
        assert_eq!(value.first_attempt_tick(), 20);
        assert_eq!(value.attempt_stride(), 2);
        assert_eq!(value.attempt_count(), 3);
        assert_eq!(value.practice_code(), practice);
        assert_eq!(value.target_selection_policy_digest(), digest(36));
        assert_eq!(value.governed_cost(), 40);
        assert_eq!(value.parameter_bytes_digest(), digest(37));
    }
    let exact = preregistration(10, 2, PracticeIdV1::Organize).unwrap();
    let mut payload = Vec::new();
    payload.extend_from_slice(&10_u64.to_be_bytes());
    payload.extend_from_slice(&11_u64.to_be_bytes());
    for tag in 30..=35 {
        payload.extend_from_slice(digest(tag).as_bytes());
    }
    payload.extend_from_slice(&[0, 0]);
    payload.extend_from_slice(&20_u64.to_be_bytes());
    payload.extend_from_slice(&2_u16.to_be_bytes());
    payload.extend_from_slice(&3_u16.to_be_bytes());
    payload.push(PracticeIdV1::Organize as u8);
    payload.extend_from_slice(digest(36).as_bytes());
    payload.extend_from_slice(&40_u32.to_be_bytes());
    payload.extend_from_slice(digest(37).as_bytes());
    assert_eq!(
        canonical_envelope(&exact).unwrap(),
        literal_envelope(SfsPreregistrationV1::DOMAIN, &payload)
    );
    assert_eq!(
        preregistration(10, 0, PracticeIdV1::Organize),
        Err(SfsRecordError::InvalidCadence)
    );
    assert_eq!(
        preregistration(u64::MAX, 1, PracticeIdV1::Organize),
        Err(SfsRecordError::ArithmeticOverflow {
            field: "start_tick"
        })
    );
}

#[test]
fn preregistration_decode_rejects_stored_start_policy_cadence_and_practice_code() {
    let value = preregistration(10, 2, PracticeIdV1::Organize).unwrap();
    let encoded = canonical_envelope(&value).unwrap();
    let payload = payload_start::<SfsPreregistrationV1>();
    let mut wrong_start = encoded.clone();
    wrong_start[payload + 8..payload + 16].copy_from_slice(&99_u64.to_be_bytes());
    assert_eq!(
        decode_envelope::<SfsPreregistrationV1>(&wrong_start),
        Err(SfsRecordError::TickDiscontinuity {
            expected: 11,
            actual: 99,
        })
    );
    let mut policy = encoded.clone();
    policy[payload + 208] = 1;
    assert_eq!(
        decode_envelope::<SfsPreregistrationV1>(&policy),
        Err(SfsRecordError::InvalidExogenousPolicy)
    );
    let mut cadence = encoded.clone();
    cadence[payload + 209] = 1;
    assert_eq!(
        decode_envelope::<SfsPreregistrationV1>(&cadence),
        Err(SfsRecordError::InvalidCadence)
    );
    let mut stride = encoded.clone();
    stride[payload + 218..payload + 220].copy_from_slice(&0_u16.to_be_bytes());
    assert_eq!(
        decode_envelope::<SfsPreregistrationV1>(&stride),
        Err(SfsRecordError::InvalidCadence)
    );
    let mut practice = encoded;
    practice[payload + 222] = 0;
    assert_eq!(
        decode_envelope::<SfsPreregistrationV1>(&practice),
        Err(SfsRecordError::InvalidPracticeCode { value: 0 })
    );
}

#[test]
fn attempt_ledger_sorts_keeps_rejections_and_projects_exact_candidates() {
    let accepted_candidate = candidate(2, 60, 61);
    let rejected_candidate = candidate(1, 62, 63);
    let accepted = PracticeAttemptRowV1::new(
        accepted_candidate.clone(),
        PracticeDispositionV1::Accepted,
        digest(64),
    )
    .unwrap();
    let rejected = PracticeAttemptRowV1::new(
        rejected_candidate.clone(),
        PracticeDispositionV1::Rejected,
        digest(65),
    )
    .unwrap();
    let ledger = PracticeAttemptLedgerV1::new(digest(66), vec![accepted, rejected]).unwrap();
    let projected = ledger.project_candidates().unwrap();
    let expected =
        PracticeCandidateScheduleV1::new(vec![accepted_candidate, rejected_candidate]).unwrap();
    assert_eq!(
        canonical_envelope(&projected).unwrap(),
        canonical_envelope(&expected).unwrap()
    );
    assert_eq!(projected.rows().len(), 2);
    let bytes = canonical_envelope(&ledger).unwrap();
    assert_eq!(
        decode_envelope::<PracticeAttemptLedgerV1>(&bytes).unwrap(),
        ledger
    );
    assert_eq!(PracticeDispositionV1::Accepted.code(), 0);
    assert_eq!(PracticeDispositionV1::Rejected.code(), 1);
    assert_eq!(PracticeDispositionV1::from_code(2), None);
}

#[test]
fn attempt_rows_reject_zero_duplicate_unknown_and_mismatched_identity() {
    let candidate_row = candidate(1, 2, 3);
    assert_eq!(
        PracticeAttemptRowV1::new(
            candidate_row.clone(),
            PracticeDispositionV1::Accepted,
            Digest32::from_bytes([0; 32]),
        ),
        Err(SfsRecordError::ZeroDispositionDigest)
    );
    let row = PracticeAttemptRowV1::new(candidate_row, PracticeDispositionV1::Accepted, digest(4))
        .unwrap();
    assert_eq!(
        PracticeAttemptLedgerV1::new(digest(5), vec![row.clone(), row]),
        Err(SfsRecordError::Wire(SfsWireError::DuplicateEntry {
            field: "attempt_rows"
        }))
    );

    let row = PracticeAttemptRowV1::new(
        candidate(1, 2, 3),
        PracticeDispositionV1::Accepted,
        digest(4),
    )
    .unwrap();
    let ledger = PracticeAttemptLedgerV1::new(digest(5), vec![row]).unwrap();
    let encoded = canonical_envelope(&ledger).unwrap();
    let mut expected_payload = Vec::new();
    expected_payload.extend_from_slice(digest(5).as_bytes());
    expected_payload.extend_from_slice(&1_u32.to_be_bytes());
    expected_payload.extend_from_slice(practice_attempt_row_id(1, digest(2), digest(3)).as_bytes());
    expected_payload.extend_from_slice(&1_u64.to_be_bytes());
    expected_payload.extend_from_slice(digest(2).as_bytes());
    expected_payload.extend_from_slice(digest(3).as_bytes());
    expected_payload.push(0);
    expected_payload.extend_from_slice(digest(4).as_bytes());
    assert_eq!(
        encoded,
        literal_envelope(PracticeAttemptLedgerV1::DOMAIN, &expected_payload)
    );
    let payload = payload_start::<PracticeAttemptLedgerV1>();
    let mut unknown = encoded.clone();
    unknown[payload + 36 + 104] = 2;
    assert_eq!(
        decode_envelope::<PracticeAttemptLedgerV1>(&unknown),
        Err(SfsRecordError::InvalidDisposition { value: 2 })
    );
    let mut bad_id = encoded;
    bad_id[payload + 36] ^= 0xff;
    assert_eq!(
        decode_envelope::<PracticeAttemptLedgerV1>(&bad_id),
        Err(SfsRecordError::StableRowDigestMismatch)
    );
}

#[test]
fn attempt_ledger_literal_malformed_payloads_refuse_before_rows_or_trailing_bytes() {
    let mut too_many_payload = Vec::new();
    too_many_payload.extend_from_slice(digest(1).as_bytes());
    too_many_payload.extend_from_slice(&65_536_u32.to_be_bytes());
    let too_many = literal_envelope(PracticeAttemptLedgerV1::DOMAIN, &too_many_payload);
    assert_eq!(
        decode_envelope::<PracticeAttemptLedgerV1>(&too_many),
        Err(SfsRecordError::Wire(SfsWireError::CountTooLarge {
            field: "attempt_rows",
            limit: 65_535,
            actual: 65_536,
        }))
    );

    let mut truncated_payload = Vec::new();
    truncated_payload.extend_from_slice(digest(1).as_bytes());
    truncated_payload.extend_from_slice(&1_u32.to_be_bytes());
    let truncated = literal_envelope(PracticeAttemptLedgerV1::DOMAIN, &truncated_payload);
    assert_eq!(
        decode_envelope::<PracticeAttemptLedgerV1>(&truncated),
        Err(SfsRecordError::Wire(SfsWireError::TruncatedEnvelope))
    );

    let mut trailing_payload = Vec::new();
    trailing_payload.extend_from_slice(digest(1).as_bytes());
    trailing_payload.extend_from_slice(&0_u32.to_be_bytes());
    trailing_payload.push(0xaa);
    let trailing = literal_envelope(PracticeAttemptLedgerV1::DOMAIN, &trailing_payload);
    assert_eq!(
        decode_envelope::<PracticeAttemptLedgerV1>(&trailing),
        Err(SfsRecordError::Wire(SfsWireError::TrailingBytes {
            count: 1,
        }))
    );
}

#[test]
fn row_count_maximum_succeeds_and_plus_one_refuses_before_sorting() {
    let mut candidates = Vec::with_capacity(65_536);
    let mut attempts = Vec::with_capacity(65_536);
    for index in 0..65_536 {
        let row = candidate(index, 70, 71);
        attempts.push(
            PracticeAttemptRowV1::new(row.clone(), PracticeDispositionV1::Accepted, digest(72))
                .unwrap(),
        );
        candidates.push(row);
    }
    assert_eq!(
        PracticeCandidateScheduleV1::new(candidates.clone()),
        Err(SfsRecordError::Wire(SfsWireError::CountTooLarge {
            field: "candidate_rows",
            limit: 65_535,
            actual: 65_536,
        }))
    );
    assert_eq!(
        PracticeAttemptLedgerV1::new(digest(73), attempts.clone()),
        Err(SfsRecordError::Wire(SfsWireError::CountTooLarge {
            field: "attempt_rows",
            limit: 65_535,
            actual: 65_536,
        }))
    );
    candidates.pop();
    attempts.pop();
    let schedule = PracticeCandidateScheduleV1::new(candidates).unwrap();
    let schedule_bytes = canonical_envelope(&schedule).unwrap();
    assert_eq!(
        payload_length::<PracticeCandidateScheduleV1>(&schedule_bytes),
        6_815_644
    );
    let ledger = PracticeAttemptLedgerV1::new(digest(73), attempts).unwrap();
    let ledger_bytes = canonical_envelope(&ledger).unwrap();
    assert_eq!(
        payload_length::<PracticeAttemptLedgerV1>(&ledger_bytes),
        8_978_331
    );
}
