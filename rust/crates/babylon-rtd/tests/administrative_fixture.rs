//! Rust authority over the fixed Detroit-Windsor administrative dossier.

use babylon_rtd::{
    canonical_draft_bytes, parse_draft_json, parse_vector_corpus, seal_draft, RtdVectorCaseV1,
};

const CONTROL: &[u8] =
    include_bytes!("../../../../contracts/fixtures/detroit_windsor_rtd_v1_admin_control.json");
const VECTORS: &[u8] =
    include_bytes!("../../../../contracts/relational_territory_dossier_v1_vectors.jsonl");
const EXTRACTION_LEDGER: &[u8] =
    include_bytes!("../../../../contracts/fixtures/detroit_windsor_rtd_v1_extraction.yaml");
const EXTRACTION_LEDGER_SHA256: [u8; 32] = [
    0x89, 0x40, 0x61, 0x47, 0x6e, 0x2a, 0x82, 0xa9, 0x0c, 0x78, 0xf1, 0x47, 0xde, 0xba, 0x0b, 0x4f,
    0x26, 0xb1, 0x8d, 0x72, 0x7c, 0xb6, 0xbf, 0x38, 0x0c, 0xcb, 0xa0, 0x8c, 0xf2, 0xed, 0x33, 0x9c,
];

#[test]
fn administrative_control_is_the_shared_rust_vector() {
    let mut sealed_value: serde_json::Value =
        serde_json::from_slice(CONTROL).expect("closed sealed control JSON");
    let sealed_object = sealed_value
        .as_object_mut()
        .expect("sealed control must be an object");
    let sealed_hash = sealed_object
        .remove("projection_hash")
        .and_then(|value| value.as_str().map(str::to_owned))
        .expect("sealed control projection hash");
    let draft_json = serde_json::to_vec(&sealed_value).expect("control draft JSON");
    let control = parse_draft_json(&draft_json).expect("closed control draft");
    let control_bytes = canonical_draft_bytes(&control).expect("canonical control bytes");
    assert_eq!(
        seal_draft(control.clone())
            .expect("sealed control")
            .projection_hash,
        sealed_hash
    );
    let cases = parse_vector_corpus(VECTORS).expect("closed RTD vector corpus");
    let mut vector = None;
    for index in 0..256 {
        if index == cases.len() {
            break;
        }
        if let RtdVectorCaseV1::Valid {
            case_id,
            draft_json,
            projection_hash,
            ..
        } = &cases[index]
        {
            if case_id == "detroit-windsor-admin-control" {
                vector = Some((
                    canonical_draft_bytes(
                        &parse_draft_json(draft_json).expect("vector control draft"),
                    )
                    .expect("vector canonical bytes"),
                    projection_hash,
                ));
                break;
            }
        }
    }
    let (vector_bytes, vector_hash) = vector.expect("Detroit-Windsor control vector");
    assert_eq!(control_bytes, vector_bytes);
    assert_eq!(&sealed_hash, vector_hash);
}

#[test]
fn extraction_ledger_bytes_are_pinned() {
    assert_eq!(
        babylon_kernel::sha256_of(EXTRACTION_LEDGER),
        EXTRACTION_LEDGER_SHA256
    );
}
