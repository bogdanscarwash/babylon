//! Language-neutral T3 contract vectors consumed by Rust.

use babylon_evidence::{
    canonical_envelope, classify_persistence, classify_sfs, record_digest, CanonicalProfileSet,
    CausalConeV1, ComponentKindV1, DifferingLedgerKindV1, Digest32, InterventionDeltaRowV1,
    InterventionDeltaV1, InterventionOperationV1, PersistenceComparisonV1, PracticeAttemptLedgerV1,
    PracticeAttemptRowV1, PracticeCandidateRowV1, PracticeCandidateScheduleV1,
    PracticeDispositionV1, RunIdentityV1, SfsComponentProofProfileV1, SfsPreregistrationV1,
    SfsProofProfileV1, SfsSampleV1, SfsTraceV1, T3Record,
};
use babylon_kernel::SessionId;
use babylon_practice_contract::PracticeIdV1;

const MAX_VECTOR_BYTES: usize = 16_777_216;
const WIRE_VECTORS: &str = include_str!("fixtures/sfs_wire_vectors_v1.txt");
const CLASSIFIER_VECTORS: &str = include_str!("fixtures/sfs_classifier_vectors_v1.txt");
const IDENTITY_MUTATIONS: &str = include_str!("fixtures/sfs_identity_mutations_v1.txt");

fn digest(tag: u8) -> Digest32 {
    let mut bytes = [0_u8; 32];
    bytes[0] = tag;
    bytes[31] = tag ^ 0xff;
    Digest32::from_bytes(bytes)
}

fn fixture_lines(input: &str, expected: usize) -> Vec<&str> {
    assert!(input.len() <= MAX_VECTOR_BYTES);
    assert!(input.is_ascii());
    assert!(input.ends_with('\n'));
    let bytes = input.as_bytes();
    let mut lines = Vec::with_capacity(expected);
    let mut start = 0_usize;
    for index in 0..MAX_VECTOR_BYTES {
        if index >= bytes.len() {
            break;
        }
        if bytes[index] == b'\n' {
            lines.push(&input[start..index]);
            start = index + 1;
        }
    }
    assert_eq!(start, bytes.len());
    assert_eq!(lines.len(), expected);
    lines
}

fn five_fields(line: &str) -> [&str; 5] {
    let (first, remainder) = line.split_once('|').expect("field one separator");
    let (second, remainder) = remainder.split_once('|').expect("field two separator");
    let (third, remainder) = remainder.split_once('|').expect("field three separator");
    let (fourth, fifth) = remainder.split_once('|').expect("field four separator");
    assert!(!fifth.contains('|'));
    [first, second, third, fourth, fifth]
}

fn nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => panic!("fixture hex must be lowercase"),
    }
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    assert!(value.len() / 2 <= MAX_VECTOR_BYTES);
    let bytes = value.as_bytes();
    let count = bytes.len() / 2;
    let mut output = Vec::with_capacity(count);
    for index in 0..MAX_VECTOR_BYTES {
        if index >= count {
            break;
        }
        output.push((nibble(bytes[index * 2]) << 4) | nibble(bytes[index * 2 + 1]));
    }
    output
}

fn assert_record<T: T3Record>(record: &T, domain: &str, envelope_hex: &str, digest_hex: &str) {
    assert_eq!(domain.as_bytes(), T::DOMAIN);
    let fixture_envelope = decode_hex(envelope_hex);
    let fixture_digest = decode_hex(digest_hex);
    assert_eq!(fixture_digest.len(), 32);
    assert_eq!(canonical_envelope(record).unwrap(), fixture_envelope);
    assert_eq!(
        record_digest(record).unwrap().as_bytes(),
        fixture_digest.as_slice()
    );
}

fn run_identity_with_mutation(field: &str) -> RunIdentityV1 {
    let changed = |name: &str, tag: u8| {
        if field == name {
            digest(tag + 128)
        } else {
            digest(tag)
        }
    };
    let session = if field == "session" {
        SessionId::new("run-互助").unwrap()
    } else {
        SessionId::new("run-é").unwrap()
    };
    let rng = if field == "rng-algorithm-id" {
        "rng-v2"
    } else {
        "rng-v1"
    };
    let graph = if field == "graph-contract-id" {
        "graph-v2"
    } else {
        "graph-v1"
    };
    RunIdentityV1::new(
        session,
        changed("scenario", 1),
        changed("prelude-declarations", 2),
        changed("vocabulary", 3),
        changed("rule-ast", 4),
        changed("host-component-manifest", 5),
        changed("defines", 6),
        changed("intrinsic-cost-cap", 7),
        changed("reference-manifest", 8),
        changed("governed-footprint-manifest", 9),
        changed("sfs-proof-profile", 10),
        changed("sfs-preregistration", 11),
        changed("initial-committed-envelope", 12),
        changed("initial-nominal-world", 13),
        changed("exogenous-input-ledger", 14),
        changed("practice-attempt-ledger", 15),
        rng,
        graph,
    )
    .unwrap()
}

fn sample(tick: u64, first_tag: u8, aggregate: f64) -> SfsSampleV1 {
    SfsSampleV1::new(
        tick,
        digest(first_tag),
        digest(first_tag + 1),
        digest(first_tag + 2),
        aggregate,
    )
    .unwrap()
}

#[allow(clippy::needless_range_loop)] // The seven-sample ceiling is part of the vector contract.
fn trace() -> SfsTraceV1 {
    let masses = [0.0, 1.0, 2.0, 5.0, 8.0, 10.0, 11.0];
    let mut samples = Vec::with_capacity(7);
    for index in 0..7 {
        samples.push(sample(
            100 + u64::try_from(index).unwrap(),
            40 + u8::try_from(index * 3).unwrap(),
            masses[index],
        ));
    }
    SfsTraceV1::new(digest(24), digest(25), 26, 100, 2, samples).unwrap()
}

fn candidates() -> Vec<PracticeCandidateRowV1> {
    vec![
        PracticeCandidateRowV1::new(201, digest(70), digest(71)),
        PracticeCandidateRowV1::new(200, digest(72), digest(73)),
    ]
}

fn candidate_schedule() -> PracticeCandidateScheduleV1 {
    PracticeCandidateScheduleV1::new(candidates()).unwrap()
}

fn preregistration() -> SfsPreregistrationV1 {
    SfsPreregistrationV1::new(
        299,
        digest(80),
        digest(81),
        digest(82),
        digest(83),
        digest(84),
        digest(85),
        310,
        3,
        4,
        PracticeIdV1::Agitate,
        digest(86),
        87,
        digest(88),
    )
    .unwrap()
}

fn attempt_ledger() -> PracticeAttemptLedgerV1 {
    let schedule = candidate_schedule();
    let rows = schedule.rows();
    let attempts = vec![
        PracticeAttemptRowV1::new(rows[0].clone(), PracticeDispositionV1::Accepted, digest(91))
            .unwrap(),
        PracticeAttemptRowV1::new(rows[1].clone(), PracticeDispositionV1::Rejected, digest(92))
            .unwrap(),
    ];
    PracticeAttemptLedgerV1::new(digest(90), attempts).unwrap()
}

fn profile_set(field: &'static str, values: &[&str]) -> CanonicalProfileSet {
    assert!(values.len() <= 64);
    let mut entries = Vec::with_capacity(values.len());
    for index in 0..64 {
        if index >= values.len() {
            break;
        }
        entries.push(values[index].to_owned());
    }
    CanonicalProfileSet::new(field, entries).unwrap()
}

fn component_profile() -> SfsComponentProofProfileV1 {
    SfsComponentProofProfileV1::new(
        "component-Ā",
        ComponentKindV1::RustBoundary,
        digest(100),
        profile_set("field_reads", &["field-a", "مساعدة"]),
        profile_set("edge_reads", &["edge-a"]),
        profile_set("constant_reads", &[&"x".repeat(96)]),
        profile_set("queries", &["q"]),
        profile_set("operators", &["op"]),
        profile_set("intrinsics", &["intrinsic"]),
        profile_set("comparison_clamp_contexts", &["clamp"]),
        profile_set("effects", &["effect", "互助"]),
    )
    .unwrap()
}

fn proof_profile() -> SfsProofProfileV1 {
    SfsProofProfileV1::new(
        digest(110),
        digest(111),
        "babylon.sfs.audit.v1",
        digest(112),
        digest(113),
        vec![component_profile()],
    )
    .unwrap()
}

fn causal_cone() -> CausalConeV1 {
    CausalConeV1::new(
        vec!["z".to_owned(), "aa".to_owned(), "\u{10000}".to_owned()],
        vec!["café".to_owned()],
        vec!["Ā".to_owned(), "互助".to_owned()],
    )
    .unwrap()
}

fn intervention_delta() -> InterventionDeltaV1 {
    let zero = Digest32::from_bytes([0; 32]);
    let rows = vec![
        InterventionDeltaRowV1::new(InterventionOperationV1::Add, digest(120), zero, digest(121))
            .unwrap(),
        InterventionDeltaRowV1::new(
            InterventionOperationV1::Remove,
            digest(122),
            digest(123),
            zero,
        )
        .unwrap(),
        InterventionDeltaRowV1::new(
            InterventionOperationV1::Replace,
            digest(124),
            digest(125),
            digest(126),
        )
        .unwrap(),
    ];
    InterventionDeltaV1::new(DifferingLedgerKindV1::PracticeAttempt, rows).unwrap()
}

fn persistence_comparison() -> PersistenceComparisonV1 {
    PersistenceComparisonV1::new(
        digest(130),
        digest(131),
        DifferingLedgerKindV1::PracticeAttempt,
        digest(132),
        digest(133),
        digest(134),
        135,
        2,
        vec![2.0, 1.0, 0.5],
    )
    .unwrap()
}

fn assert_wire(parts: [&str; 5]) {
    assert_eq!(parts[0], "wire");
    match parts[1] {
        "run-identity" => assert_record(
            &run_identity_with_mutation(""),
            parts[2],
            parts[3],
            parts[4],
        ),
        "sfs-sample" => assert_record(&sample(7, 21, 3.5), parts[2], parts[3], parts[4]),
        "sfs-trace" => assert_record(&trace(), parts[2], parts[3], parts[4]),
        "sfs-preregistration" => {
            assert_record(&preregistration(), parts[2], parts[3], parts[4]);
        }
        "practice-candidate-schedule" => {
            assert_record(&candidate_schedule(), parts[2], parts[3], parts[4]);
        }
        "practice-attempt-ledger" => {
            assert_record(&attempt_ledger(), parts[2], parts[3], parts[4]);
        }
        "component-proof-profile" => {
            assert_record(&component_profile(), parts[2], parts[3], parts[4]);
        }
        "proof-profile" => assert_record(&proof_profile(), parts[2], parts[3], parts[4]),
        "causal-cone" => assert_record(&causal_cone(), parts[2], parts[3], parts[4]),
        "intervention-delta" => {
            assert_record(&intervention_delta(), parts[2], parts[3], parts[4]);
        }
        "persistence-comparison" => {
            assert_record(&persistence_comparison(), parts[2], parts[3], parts[4]);
        }
        label => panic!("unknown wire-vector label {label}"),
    }
}

fn parse_bits(value: &str, expected: usize) -> Vec<f64> {
    let mut output = Vec::with_capacity(expected);
    let mut remainder = value;
    for index in 0..157 {
        if index >= expected {
            break;
        }
        let (part, next) = if index + 1 == expected {
            (remainder, "")
        } else {
            remainder.split_once(',').expect("classifier bit separator")
        };
        let bits = u64::from_str_radix(part, 16).expect("literal binary64 bits");
        output.push(f64::from_bits(bits));
        remainder = next;
    }
    assert!(remainder.is_empty());
    output
}

#[test]
#[allow(clippy::needless_range_loop)] // The eleven-row ceiling is part of the fixture contract.
fn every_wire_vector_reconstructs_exact_rust_envelope_and_digest() {
    let lines = fixture_lines(WIRE_VECTORS, 11);
    for index in 0..11 {
        assert_wire(five_fields(lines[index]));
    }
}

#[test]
#[allow(clippy::needless_range_loop)] // The twelve-row ceiling is part of the fixture contract.
fn all_classifier_vectors_match_the_independent_literal_bits() {
    let lines = fixture_lines(CLASSIFIER_VECTORS, 12);
    for index in 0..12 {
        let parts = five_fields(lines[index]);
        let width = parts[2].parse::<u16>().expect("literal width");
        let class = parts[4].parse::<u8>().expect("literal class");
        match parts[0] {
            "classifier" => {
                assert_eq!(
                    classify_sfs(width, &parse_bits(parts[3], 7))
                        .unwrap()
                        .code(),
                    class
                );
            }
            "persistence" => {
                assert_eq!(
                    classify_persistence(width, &parse_bits(parts[3], 3))
                        .unwrap()
                        .code(),
                    class
                );
            }
            kind => panic!("unknown classifier-vector kind {kind}"),
        }
    }
}

#[test]
#[allow(clippy::needless_range_loop)] // The eighteen-row ceiling is part of the fixture contract.
fn every_run_identity_mutation_changes_bytes_and_digest_and_reconstructs() {
    let base = run_identity_with_mutation("");
    let base_bytes = canonical_envelope(&base).unwrap();
    let base_digest = record_digest(&base).unwrap();
    let lines = fixture_lines(IDENTITY_MUTATIONS, 18);
    for index in 0..18 {
        let parts = five_fields(lines[index]);
        assert_eq!(parts[0], "mutation");
        assert_eq!(parts[1], "run-identity");
        let mutated = run_identity_with_mutation(parts[2]);
        let mutated_bytes = canonical_envelope(&mutated).unwrap();
        let mutated_digest = record_digest(&mutated).unwrap();
        assert_eq!(mutated_bytes, decode_hex(parts[3]));
        assert_eq!(mutated_digest.to_hex(), parts[4]);
        assert_ne!(mutated_bytes, base_bytes);
        assert_ne!(mutated_digest.as_bytes(), base_digest.as_bytes());
    }
}
