//! DB-free executable laws for the frozen Rust persistence cutover corpus.

use babylon_persistence::{
    verify_rust_persistence_cutover_vector_row_v1, verify_rust_persistence_cutover_vectors_v1,
    RustPersistenceVectorErrorV1, RustPersistenceVectorOutcomeV1, RustPersistenceVectorReportV1,
};

const VECTORS: &[u8] =
    include_bytes!("../../../../contracts/rust_persistence_cutover_v1_vectors.jsonl");
const MATERIAL_STATE_SOURCE: &str = include_str!("../../babylon-tick/src/material_state.rs");
const SEMANTIC_CODEC_SOURCE: &str = include_str!("../src/semantic_codec.rs");
const CUTOVER_VECTOR_SOURCE: &str = include_str!("../src/cutover_vectors.rs");
const ENVELOPE_SOURCE: &str = include_str!("../src/committed_tick_envelope.rs");

fn assert_send<T: Send>() {}

#[test]
fn false_sidecar_types_codecs_and_executors_do_not_survive() {
    for symbol in [
        "HexActivityRowV1",
        "HexMaterialStateRowV1",
        "HexTerrainStateRowV1",
        "InfrastructureLinkStateRowV1",
        "HexActivityRowsV1",
        "HexMaterialStateRowsV1",
        "HexTerrainStateRowsV1",
        "InfrastructureLinkStateRowsV1",
        "HexActivity(",
        "HexMaterial(",
        "HexTerrain(",
        "InfrastructureLink(",
        "hex_activities",
        "hex_materials",
        "hex_terrains",
        "infrastructure_links",
        "encode_hex_activity",
        "encode_hex_material",
        "encode_hex_terrain",
        "encode_infrastructure",
    ] {
        assert!(
            !MATERIAL_STATE_SOURCE.contains(symbol),
            "false material authority survived: {symbol}"
        );
    }
    for codec in [
        "encode_hex_activity",
        "encode_hex_material_state",
        "encode_hex_terrain_state",
        "encode_infrastructure_link_state",
    ] {
        assert!(
            !SEMANTIC_CODEC_SOURCE.contains(codec),
            "false semantic codec survived: {codec}"
        );
    }
    for executor in [
        "execute_hex_activity",
        "execute_hex_material",
        "execute_hex_terrain",
        "execute_infrastructure",
        "hex_activity_v1",
        "hex_material_state_v1",
        "hex_terrain_state_v1",
        "infrastructure_link_state_v1",
    ] {
        assert!(
            !CUTOVER_VECTOR_SOURCE.contains(executor),
            "false vector executor survived: {executor}"
        );
    }

    for governed in [
        "material_batch!(WorldRegisterRowsV1, WorldRegisterRowV1, 0x01)",
        "material_batch!(TerritoryStateRowsV1, TerritoryStateRowV1, 0x02)",
        "material_batch!(DynamicHexStateRowsV1, DynamicHexStateRowV1, 0x03)",
        "material_batch!(OrganizationStateRowsV1, OrganizationStateRowV1, 0x08)",
    ] {
        assert!(MATERIAL_STATE_SOURCE.contains(governed));
    }
    let batch_declarations = MATERIAL_STATE_SOURCE
        .split("material_batch!")
        .skip(1)
        .map(|suffix| {
            suffix
                .split(';')
                .next()
                .expect("terminated batch declaration")
        })
        .collect::<String>();
    for retired_tag in ["0x04", "0x05", "0x06", "0x07"] {
        assert!(
            !batch_declarations.contains(retired_tag),
            "retired tag must not be rebound: {retired_tag}"
        );
    }
}

#[test]
fn unproduced_envelope_families_codecs_and_executors_do_not_survive() {
    for symbol in [
        "Subsystem",
        "Conservation",
        "BoundaryFlow",
        "subsystem_v1",
        "conservation_v1",
        "boundary_flow_v1",
        "encode_subsystem",
        "encode_conservation",
        "encode_boundary_flow",
    ] {
        assert!(!ENVELOPE_SOURCE.contains(symbol), "false family: {symbol}");
        assert!(
            !SEMANTIC_CODEC_SOURCE.contains(symbol),
            "false codec: {symbol}"
        );
        assert!(
            !CUTOVER_VECTOR_SOURCE.contains(symbol),
            "false executor: {symbol}"
        );
    }
}

#[test]
fn cutover_vector_executor_exports_only_the_five_frozen_symbols() {
    assert_send::<RustPersistenceVectorErrorV1>();
    assert_send::<RustPersistenceVectorOutcomeV1>();
    assert_send::<RustPersistenceVectorReportV1>();

    let _: fn(&[u8]) -> Result<RustPersistenceVectorOutcomeV1, RustPersistenceVectorErrorV1> =
        verify_rust_persistence_cutover_vector_row_v1;
    let _: fn(&[u8]) -> Result<RustPersistenceVectorReportV1, RustPersistenceVectorErrorV1> =
        verify_rust_persistence_cutover_vectors_v1;
}

#[test]
fn shared_language_neutral_vectors_execute_every_codec_and_refusal() {
    const AD_HOC_VALID: &[u8] = br#"{"id":"ad-hoc-f64-two-point-five","kind":"valid_scalar","codec":"f64_be_canonical","input":"2.5","expected_hex":"4004000000000000"}"#;
    const AD_HOC_REFUSAL: &[u8] = br#"{"id":"ad-hoc-f64-nan","kind":"refusal","operation":"encode_scalar","codec":"f64_be_canonical","input":"nan","expected_code":"nonfinite_f64"}"#;
    const LYING_REFUSAL: &[u8] = br#"{"id":"changed-id-input-and-code","kind":"refusal","operation":"encode_scalar","codec":"f64_be_canonical","input":"2.5","expected_code":"field_byte_bound"}"#;

    let report =
        verify_rust_persistence_cutover_vectors_v1(VECTORS).expect("governed vector corpus");

    assert_eq!(report.row_count(), 56);
    for kind in [
        "valid_scalar",
        "valid_row",
        "valid_foundation",
        "valid_checkpoint",
        "valid_empty_family",
        "valid_authority_ledger",
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
    assert_eq!(rows.len(), 56);
    for row in rows {
        let outcome = verify_rust_persistence_cutover_vector_row_v1(row)
            .expect("every row executes independently of corpus identity");
        assert!(!outcome.id().is_empty());
        assert!(!outcome.kind().is_empty());
    }

    let ad_hoc = verify_rust_persistence_cutover_vector_row_v1(AD_HOC_VALID)
        .expect("non-corpus input executes the actual scalar codec");
    assert_eq!(ad_hoc.id(), "ad-hoc-f64-two-point-five");

    verify_rust_persistence_cutover_vector_row_v1(AD_HOC_REFUSAL)
        .expect("non-corpus refusal executes the actual scalar rule");

    assert!(
        verify_rust_persistence_cutover_vector_row_v1(LYING_REFUSAL).is_err(),
        "an expected code cannot manufacture a refusal after semantic input succeeds"
    );

    let mut mutated_corpus = VECTORS.to_vec();
    let witness = b"3ff8000000000000";
    let offset = mutated_corpus
        .windows(witness.len())
        .position(|window| window == witness)
        .expect("governed byte witness");
    mutated_corpus[offset] = b'2';
    assert!(verify_rust_persistence_cutover_vectors_v1(&mutated_corpus).is_err());

    let mut mutated_input = AD_HOC_VALID.to_vec();
    let witness = b"\"input\":\"2.5\"";
    let offset = mutated_input
        .windows(witness.len())
        .position(|window| window == witness)
        .expect("ad-hoc input witness");
    mutated_input[offset + witness.len() - 4] = b'3';
    assert!(
        verify_rust_persistence_cutover_vector_row_v1(&mutated_input).is_err(),
        "semantic input cannot change while exact expected bytes stay fixed"
    );
}

#[test]
fn authority_ledger_vectors_execute_exact_bytes_and_predecessor_identity() {
    let rows = VECTORS
        .split(|byte| *byte == b'\n')
        .filter(|row| !row.is_empty())
        .collect::<Vec<_>>();
    let prepared = rows
        .iter()
        .copied()
        .find(|row| {
            row.windows(b"authority-ledger-prepared".len())
                .any(|window| window == b"authority-ledger-prepared")
        })
        .expect("prepared authority-ledger vector");
    let active = rows
        .iter()
        .copied()
        .find(|row| {
            row.windows(b"authority-ledger-rust-active".len())
                .any(|window| window == b"authority-ledger-rust-active")
        })
        .expect("active authority-ledger vector");

    verify_rust_persistence_cutover_vector_row_v1(prepared)
        .expect("prepared authority row exact bytes");
    verify_rust_persistence_cutover_vector_row_v1(active)
        .expect("active authority row exact bytes and predecessor");

    let mut wrong_predecessor = active.to_vec();
    let witness = b"7d9d13782b603486b3c03f1a90d73a7da05243d75818ce32fc00bf568a232440";
    let offset = wrong_predecessor
        .windows(witness.len())
        .position(|window| window == witness)
        .expect("prepared-row SHA predecessor witness");
    wrong_predecessor[offset] = b'0';
    assert!(
        verify_rust_persistence_cutover_vector_row_v1(&wrong_predecessor).is_err(),
        "the active authority row must bind the exact prepared-row SHA"
    );
}

#[test]
fn individual_rows_require_a_nonempty_diagnostic_id() {
    const EMPTY_ID: &[u8] =
        br#"{"id":"","kind":"valid_scalar","codec":"bool_u8","input":true,"expected_hex":"01"}"#;
    assert!(verify_rust_persistence_cutover_vector_row_v1(EMPTY_ID).is_err());
}

#[test]
fn vector_executor_consumes_foundation_checkpoint_and_empty_proof_semantics() {
    for (id, witness, replacement) in [
        (
            "foundation-full-nine-fields",
            b"\"layout\":1".as_slice(),
            b"\"layout\":2".as_slice(),
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
            verify_rust_persistence_cutover_vector_row_v1(&mutated).is_err(),
            "unused semantic field: {id}"
        );
    }
}
