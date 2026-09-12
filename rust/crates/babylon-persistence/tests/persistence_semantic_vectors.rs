//! DB-free executable laws for the current persistence semantic corpus.

use babylon_persistence::{
    verify_persistence_semantic_vector_row, verify_persistence_semantic_vectors,
    RustPersistenceVectorError, RustPersistenceVectorOutcome, RustPersistenceVectorReport,
};

const VECTORS: &[u8] = include_bytes!("../../../../contracts/persistence_semantic_vectors.jsonl");
fn assert_send<T: Send>() {}

#[test]
fn semantic_vector_types_are_send_and_executors_are_typed() {
    assert_send::<RustPersistenceVectorError>();
    assert_send::<RustPersistenceVectorOutcome>();
    assert_send::<RustPersistenceVectorReport>();

    let _: fn(&[u8]) -> Result<RustPersistenceVectorOutcome, RustPersistenceVectorError> =
        verify_persistence_semantic_vector_row;
    let _: fn(&[u8]) -> Result<RustPersistenceVectorReport, RustPersistenceVectorError> =
        verify_persistence_semantic_vectors;
}

#[test]
fn shared_language_neutral_vectors_execute_every_codec_and_refusal() {
    const AD_HOC_VALID: &[u8] = br#"{"id":"ad-hoc-f64-two-point-five","kind":"valid_scalar","codec":"f64_be_canonical","input":"2.5","expected_hex":"4004000000000000"}"#;
    const AD_HOC_REFUSAL: &[u8] = br#"{"id":"ad-hoc-f64-nan","kind":"refusal","operation":"encode_scalar","codec":"f64_be_canonical","input":"nan","expected_code":"nonfinite_f64"}"#;
    const LYING_REFUSAL: &[u8] = br#"{"id":"changed-id-input-and-code","kind":"refusal","operation":"encode_scalar","codec":"f64_be_canonical","input":"2.5","expected_code":"field_byte_bound"}"#;

    let report = verify_persistence_semantic_vectors(VECTORS).expect("governed vector corpus");

    assert_eq!(report.row_count(), 54);
    for kind in [
        "valid_scalar",
        "valid_row",
        "valid_foundation",
        "valid_checkpoint",
        "valid_empty_family",
        "refusal",
    ] {
        assert!(
            report.kind_count(kind) > 0,
            "missing executed vector kind {kind}"
        );
    }
    for codec in [
        "stable_graph_node_v1",
        "stable_graph_node_f64_v1",
        "stable_graph_edge_v1",
        "stable_graph_hyperedge_v1",
        "stable_graph_edge_f64_v1",
        "stable_graph_node_currency_v1",
        "stable_graph_hyperedge_f64_v1",
        "world_register_v1",
        "territory_state_v1",
        "dynamic_hex_state_v1",
        "organization_state_v1",
        "successful_event_v1",
        "checkpoint_v1",
        "archive_dirty_receipt_v1",
    ] {
        assert_eq!(report.valid_row_codec_count(codec), 1, "row codec: {codec}");
    }

    let rows = VECTORS
        .split(|byte| *byte == b'\n')
        .filter(|row| !row.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 54);
    for row in rows {
        let outcome = verify_persistence_semantic_vector_row(row)
            .expect("every row executes independently of corpus identity");
        assert!(!outcome.id().is_empty());
        assert!(!outcome.kind().is_empty());
    }

    let ad_hoc = verify_persistence_semantic_vector_row(AD_HOC_VALID)
        .expect("non-corpus input executes the actual scalar codec");
    assert_eq!(ad_hoc.id(), "ad-hoc-f64-two-point-five");

    verify_persistence_semantic_vector_row(AD_HOC_REFUSAL)
        .expect("non-corpus refusal executes the actual scalar rule");

    assert!(
        verify_persistence_semantic_vector_row(LYING_REFUSAL).is_err(),
        "an expected code cannot manufacture a refusal after semantic input succeeds"
    );

    let mut mutated_corpus = VECTORS.to_vec();
    let witness = b"3ff8000000000000";
    let offset = mutated_corpus
        .windows(witness.len())
        .position(|window| window == witness)
        .expect("governed byte witness");
    mutated_corpus[offset] = b'2';
    assert!(verify_persistence_semantic_vectors(&mutated_corpus).is_err());

    let mut mutated_input = AD_HOC_VALID.to_vec();
    let witness = b"\"input\":\"2.5\"";
    let offset = mutated_input
        .windows(witness.len())
        .position(|window| window == witness)
        .expect("ad-hoc input witness");
    mutated_input[offset + witness.len() - 4] = b'3';
    assert!(
        verify_persistence_semantic_vector_row(&mutated_input).is_err(),
        "semantic input cannot change while exact expected bytes stay fixed"
    );
}

#[test]
fn individual_rows_require_a_nonempty_diagnostic_id() {
    const EMPTY_ID: &[u8] =
        br#"{"id":"","kind":"valid_scalar","codec":"bool_u8","input":true,"expected_hex":"01"}"#;
    assert!(verify_persistence_semantic_vector_row(EMPTY_ID).is_err());
}

#[test]
fn vector_executor_consumes_foundation_checkpoint_and_empty_proof_semantics() {
    for (id, witness, replacement) in [
        (
            "foundation-full-nine-fields",
            b"\"layout\":2".as_slice(),
            b"\"layout\":1".as_slice(),
        ),
        (
            "checkpoint-full-nine-sections",
            b"\"layout\":1".as_slice(),
            b"\"layout\":2".as_slice(),
        ),
        (
            "checkpoint-full-nine-sections",
            b"\"completeness\":\"full\"".as_slice(),
            b"\"completeness\":\"fake\"".as_slice(),
        ),
        (
            "empty-successful-event-source-proof",
            b"\"family\":\"event\"".as_slice(),
            b"\"family\":\"state\"".as_slice(),
        ),
        (
            "empty-successful-event-source-proof",
            b"\"producer\":\"successful_event_batch_v1\"".as_slice(),
            b"\"producer\":\"successful_event_batch_v2\"".as_slice(),
        ),
    ] {
        let row = VECTORS
            .split(|byte| *byte == b'\n')
            .find(|row| row.windows(id.len()).any(|window| window == id.as_bytes()))
            .expect("governed semantic row");
        assert_eq!(witness.len(), replacement.len());
        let mut mutated = row.to_vec();
        let offset = mutated
            .windows(witness.len())
            .position(|window| window == witness)
            .expect("semantic witness");
        mutated[offset..offset + witness.len()].copy_from_slice(replacement);
        assert!(
            verify_persistence_semantic_vector_row(&mutated).is_err(),
            "unused semantic field: {id}"
        );
    }
}
