//! Language-neutral contracts for the frozen T3 proof-profile records.

use babylon_evidence::{
    canonical_envelope, decode_envelope, CanonicalProfileSet, CausalConeV1, ComponentKindV1,
    DifferingLedgerKindV1, Digest32, InterventionDeltaRowV1, InterventionDeltaV1,
    InterventionOperationV1, PersistenceClassError, PersistenceComparisonV1,
    SfsComponentProofProfileV1, SfsProfileRecordError, SfsProofProfileV1, SfsWireError, T3Record,
};

fn digest(tag: u8) -> Digest32 {
    let mut bytes = [0_u8; 32];
    bytes[0] = tag;
    bytes[31] = tag ^ 0xff;
    Digest32::from_bytes(bytes)
}

fn indexed_digest(index: u64) -> Digest32 {
    let mut bytes = [0_u8; 32];
    bytes[..8].copy_from_slice(&index.to_be_bytes());
    bytes[8] = 1;
    Digest32::from_bytes(bytes)
}

fn zero_digest() -> Digest32 {
    Digest32::from_bytes([0; 32])
}

fn literal_envelope(domain: &[u8], payload: &[u8]) -> Vec<u8> {
    let length = u32::try_from(payload.len()).expect("test payload fits u32");
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
            .expect("payload length is four bytes"),
    ))
    .expect("u32 fits usize")
}

fn literal_string(value: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u16::try_from(value.len())
            .expect("test string fits u16")
            .to_be_bytes(),
    );
    bytes.extend_from_slice(value.as_bytes());
    bytes
}

fn literal_set(entries: &[&str]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u16::try_from(entries.len())
            .expect("test set count fits u16")
            .to_be_bytes(),
    );
    for index in 0..64 {
        if index >= entries.len() {
            break;
        }
        bytes.extend_from_slice(&literal_string(entries[index]));
    }
    bytes
}

fn literal_component_envelope(component_id: &str, field_reads: &[&str]) -> Vec<u8> {
    let mut payload = literal_string(component_id);
    payload.push(0);
    payload.extend_from_slice(digest(1).as_bytes());
    payload.extend_from_slice(&literal_set(field_reads));
    for _index in 0..7 {
        payload.extend_from_slice(&0_u16.to_be_bytes());
    }
    literal_envelope(SfsComponentProofProfileV1::DOMAIN, &payload)
}

fn profile_set(field: &'static str, values: &[&str]) -> CanonicalProfileSet {
    CanonicalProfileSet::new(
        field,
        values.iter().map(|value| (*value).to_owned()).collect(),
    )
    .unwrap()
}

fn empty_set(field: &'static str) -> CanonicalProfileSet {
    profile_set(field, &[])
}

fn component(component_id: &str, field_reads: &[&str]) -> SfsComponentProofProfileV1 {
    SfsComponentProofProfileV1::new(
        component_id,
        ComponentKindV1::BslRule,
        digest(1),
        profile_set("field_reads", field_reads),
        empty_set("edge_reads"),
        empty_set("constant_reads"),
        empty_set("queries"),
        empty_set("operators"),
        empty_set("intrinsics"),
        empty_set("comparison_clamp_contexts"),
        empty_set("effects"),
    )
    .unwrap()
}

fn proof_profile(components: Vec<SfsComponentProofProfileV1>) -> SfsProofProfileV1 {
    SfsProofProfileV1::new(
        digest(2),
        digest(3),
        "babylon.sfs.audit.v1",
        digest(4),
        digest(5),
        components,
    )
    .unwrap()
}

fn literal_proof_payload(components: &[Vec<u8>]) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(digest(2).as_bytes());
    payload.extend_from_slice(digest(3).as_bytes());
    payload.extend_from_slice(&literal_string("babylon.sfs.audit.v1"));
    payload.extend_from_slice(digest(4).as_bytes());
    payload.extend_from_slice(digest(5).as_bytes());
    payload.extend_from_slice(
        &u16::try_from(components.len())
            .expect("test component count fits u16")
            .to_be_bytes(),
    );
    for index in 0..64 {
        if index >= components.len() {
            break;
        }
        payload.extend_from_slice(&components[index]);
    }
    payload
}

fn maximum_profile_set() -> CanonicalProfileSet {
    let mut entries = Vec::with_capacity(64);
    for index in 0..64 {
        entries.push(format!("{index:02}{}", "x".repeat(94)));
    }
    CanonicalProfileSet::new("maximum_set", entries).unwrap()
}

fn maximum_component(index: usize) -> SfsComponentProofProfileV1 {
    let component_id = format!("{index:02}{}", "c".repeat(254));
    let set = maximum_profile_set();
    SfsComponentProofProfileV1::new(
        &component_id,
        ComponentKindV1::PostCommitProducer,
        digest(1),
        set.clone(),
        set.clone(),
        set.clone(),
        set.clone(),
        set.clone(),
        set.clone(),
        set.clone(),
        set,
    )
    .unwrap()
}

#[test]
fn profile_domains_maxima_and_closed_codes_are_exact() {
    assert_eq!(
        SfsComponentProofProfileV1::DOMAIN,
        b"babylon.sfs-component-proof-profile.v1"
    );
    assert_eq!(SfsComponentProofProfileV1::MAX_PAYLOAD_BYTES, 50_483);
    assert_eq!(SfsProofProfileV1::DOMAIN, b"babylon.sfs-proof-profile.v1");
    assert_eq!(SfsProofProfileV1::MAX_PAYLOAD_BYTES, 3_233_944);
    assert_eq!(CausalConeV1::DOMAIN, b"babylon.sfs-causal-cone.v1");
    assert_eq!(CausalConeV1::MAX_PAYLOAD_BYTES, 49_542);
    assert_eq!(
        InterventionDeltaV1::DOMAIN,
        b"babylon.intervention-delta.v1"
    );
    assert_eq!(InterventionDeltaV1::MAX_PAYLOAD_BYTES, 6_356_900);
    assert_eq!(
        PersistenceComparisonV1::DOMAIN,
        b"babylon.persistence-comparison.v1"
    );
    assert_eq!(PersistenceComparisonV1::MAX_PAYLOAD_BYTES, 598);
    assert_eq!(ComponentKindV1::BslRule as u8, 0);
    assert_eq!(ComponentKindV1::RustBoundary as u8, 1);
    assert_eq!(ComponentKindV1::Reducer as u8, 2);
    assert_eq!(ComponentKindV1::PostCommitProducer as u8, 3);
    assert_eq!(DifferingLedgerKindV1::ExogenousInput as u8, 0);
    assert_eq!(DifferingLedgerKindV1::PracticeAttempt as u8, 1);
    assert_eq!(InterventionOperationV1::Add as u8, 0);
    assert_eq!(InterventionOperationV1::Remove as u8, 1);
    assert_eq!(InterventionOperationV1::Replace as u8, 2);
}

#[test]
fn component_profile_proof_profile_and_cone_maximum_shapes_encode() {
    let component = maximum_component(0);
    let component_bytes = canonical_envelope(&component).unwrap();
    assert_eq!(
        payload_length::<SfsComponentProofProfileV1>(&component_bytes),
        50_483
    );
    let mut components = Vec::with_capacity(64);
    for index in 0..64 {
        components.push(maximum_component(index));
    }
    let profile = proof_profile(components);
    let profile_bytes = canonical_envelope(&profile).unwrap();
    assert_eq!(
        payload_length::<SfsProofProfileV1>(&profile_bytes),
        3_233_944
    );
    assert_eq!(
        decode_envelope::<SfsProofProfileV1>(&profile_bytes).unwrap(),
        profile
    );

    let mut ids = Vec::with_capacity(64);
    for index in 0..64 {
        ids.push(format!("{index:02}{}", "i".repeat(254)));
    }
    let cone = CausalConeV1::new(ids.clone(), ids.clone(), ids).unwrap();
    let cone_bytes = canonical_envelope(&cone).unwrap();
    assert_eq!(payload_length::<CausalConeV1>(&cone_bytes), 49_542);
    assert_eq!(decode_envelope::<CausalConeV1>(&cone_bytes).unwrap(), cone);
}

#[test]
fn component_id_and_profile_set_string_bounds_are_closed() {
    assert_eq!(component("a", &[]).component_id(), "a");
    assert_eq!(component(&"a".repeat(256), &[]).component_id().len(), 256);
    assert_eq!(profile_set("field_reads", &["a"]).entries(), &["a"]);
    assert_eq!(
        profile_set("field_reads", &[&"a".repeat(96)]).entries()[0].len(),
        96
    );

    let empty_component = SfsComponentProofProfileV1::new(
        "",
        ComponentKindV1::BslRule,
        digest(1),
        empty_set("field_reads"),
        empty_set("edge_reads"),
        empty_set("constant_reads"),
        empty_set("queries"),
        empty_set("operators"),
        empty_set("intrinsics"),
        empty_set("comparison_clamp_contexts"),
        empty_set("effects"),
    );
    assert_eq!(
        empty_component,
        Err(SfsProfileRecordError::Wire(SfsWireError::StringEmpty {
            field: "component_id",
        }))
    );
    assert_eq!(
        SfsComponentProofProfileV1::new(
            &"a".repeat(257),
            ComponentKindV1::BslRule,
            digest(1),
            empty_set("field_reads"),
            empty_set("edge_reads"),
            empty_set("constant_reads"),
            empty_set("queries"),
            empty_set("operators"),
            empty_set("intrinsics"),
            empty_set("comparison_clamp_contexts"),
            empty_set("effects"),
        ),
        Err(SfsProfileRecordError::Wire(SfsWireError::StringTooLong {
            field: "component_id",
            limit: 256,
            actual: 257,
        }))
    );
    assert_eq!(
        CanonicalProfileSet::new("field_reads", vec![String::new()]),
        Err(SfsProfileRecordError::Wire(SfsWireError::StringEmpty {
            field: "field_reads",
        }))
    );
    assert_eq!(
        CanonicalProfileSet::new("field_reads", vec!["a".repeat(97)]),
        Err(SfsProfileRecordError::Wire(SfsWireError::StringTooLong {
            field: "field_reads",
            limit: 96,
            actual: 97,
        }))
    );
    let non_nfc = literal_component_envelope("cafe\u{301}", &[]);
    assert_eq!(
        decode_envelope::<SfsComponentProofProfileV1>(&non_nfc),
        Err(SfsProfileRecordError::Wire(SfsWireError::NonNfc {
            field: "component_id",
        }))
    );
}

#[test]
fn profile_sets_sort_and_reject_counts_duplicates_non_nfc_and_wire_order() {
    let sorted = profile_set("field_reads", &["b", "a"]);
    assert_eq!(sorted.entries(), &["a", "b"]);
    assert_eq!(
        CanonicalProfileSet::new("field_reads", vec!["a".to_owned(), "a".to_owned()]),
        Err(SfsProfileRecordError::Wire(SfsWireError::DuplicateEntry {
            field: "field_reads",
        }))
    );
    assert_eq!(
        CanonicalProfileSet::new("field_reads", vec!["cafe\u{301}".to_owned()]),
        Err(SfsProfileRecordError::Wire(SfsWireError::NonNfc {
            field: "field_reads",
        }))
    );
    let mut entries = Vec::new();
    for index in 0..65 {
        entries.push(format!("entry-{index:02}"));
    }
    assert!(CanonicalProfileSet::new("field_reads", entries[..64].to_vec()).is_ok());
    assert_eq!(
        CanonicalProfileSet::new("field_reads", entries),
        Err(SfsProfileRecordError::Wire(SfsWireError::CountTooLarge {
            field: "field_reads",
            limit: 64,
            actual: 65,
        }))
    );
    let out_of_order = literal_component_envelope("a", &["b", "a"]);
    assert_eq!(
        decode_envelope::<SfsComponentProofProfileV1>(&out_of_order),
        Err(SfsProfileRecordError::Wire(SfsWireError::OutOfOrder {
            field: "field_reads",
        }))
    );
    let mut too_many = literal_component_envelope("a", &[]);
    let count = payload_start::<SfsComponentProofProfileV1>() + 36;
    too_many[count..count + 2].copy_from_slice(&65_u16.to_be_bytes());
    assert_eq!(
        decode_envelope::<SfsComponentProofProfileV1>(&too_many),
        Err(SfsProfileRecordError::Wire(SfsWireError::CountTooLarge {
            field: "field_reads",
            limit: 64,
            actual: 65,
        }))
    );
}

#[test]
fn component_profile_literal_wire_and_kind_decoder_are_exact() {
    let value = component("a", &["field"]);
    assert_eq!(value.component_kind(), ComponentKindV1::BslRule);
    assert_eq!(value.component_source_digest(), digest(1));
    let expected = literal_component_envelope("a", &["field"]);
    assert_eq!(canonical_envelope(&value).unwrap(), expected);
    assert_eq!(
        decode_envelope::<SfsComponentProofProfileV1>(&expected).unwrap(),
        value
    );
    let kind_offset = payload_start::<SfsComponentProofProfileV1>() + 3;
    for (code, kind) in [
        (0, ComponentKindV1::BslRule),
        (1, ComponentKindV1::RustBoundary),
        (2, ComponentKindV1::Reducer),
        (3, ComponentKindV1::PostCommitProducer),
    ] {
        let mut encoded = expected.clone();
        encoded[kind_offset] = code;
        let decoded = decode_envelope::<SfsComponentProofProfileV1>(&encoded).unwrap();
        assert_eq!(decoded.component_kind(), kind);
        assert_eq!(canonical_envelope(&decoded).unwrap(), encoded);
    }
    let mut invalid_kind = expected;
    invalid_kind[kind_offset] = 4;
    assert_eq!(
        decode_envelope::<SfsComponentProofProfileV1>(&invalid_kind),
        Err(SfsProfileRecordError::InvalidComponentKind { value: 4 })
    );
}

#[test]
fn proof_profile_sorts_complete_nested_envelopes_and_closes_audit_semantics() {
    let profile = proof_profile(vec![component("b", &[]), component("a", &[])]);
    assert_eq!(profile.components()[0].component_id(), "a");
    assert_eq!(profile.components()[1].component_id(), "b");
    assert_eq!(profile.governed_manifest_digest(), digest(2));
    assert_eq!(profile.causal_cone_digest(), digest(5));
    let expected_payload = literal_proof_payload(&[
        literal_component_envelope("a", &[]),
        literal_component_envelope("b", &[]),
    ]);
    let expected = literal_envelope(SfsProofProfileV1::DOMAIN, &expected_payload);
    assert_eq!(canonical_envelope(&profile).unwrap(), expected);
    assert_eq!(
        decode_envelope::<SfsProofProfileV1>(&expected).unwrap(),
        profile
    );
    assert_eq!(
        SfsProofProfileV1::new(
            digest(2),
            digest(3),
            "babylon.sfs.audit.v2",
            digest(4),
            digest(5),
            vec![]
        ),
        Err(SfsProfileRecordError::InvalidAuditSemanticsId)
    );
    assert_eq!(
        SfsProofProfileV1::new(digest(2), digest(3), "", digest(4), digest(5), vec![]),
        Err(SfsProfileRecordError::Wire(SfsWireError::StringEmpty {
            field: "audit_semantics_id",
        }))
    );
    assert_eq!(
        SfsProofProfileV1::new(
            digest(2),
            digest(3),
            &"a".repeat(65),
            digest(4),
            digest(5),
            vec![]
        ),
        Err(SfsProfileRecordError::Wire(SfsWireError::StringTooLong {
            field: "audit_semantics_id",
            limit: 64,
            actual: 65,
        }))
    );
    assert_eq!(
        SfsProofProfileV1::new(
            digest(2),
            digest(3),
            "audit-é",
            digest(4),
            digest(5),
            vec![]
        ),
        Err(SfsProfileRecordError::Wire(SfsWireError::NonAscii {
            field: "audit_semantics_id",
        }))
    );
}

#[test]
fn proof_profile_empty_and_singleton_envelopes_round_trip() {
    let empty = proof_profile(vec![]);
    let empty_bytes = canonical_envelope(&empty).unwrap();
    assert_eq!(
        decode_envelope::<SfsProofProfileV1>(&empty_bytes).unwrap(),
        empty
    );

    let singleton = proof_profile(vec![component("only-component", &["field"])]);
    let singleton_bytes = canonical_envelope(&singleton).unwrap();
    assert_eq!(
        decode_envelope::<SfsProofProfileV1>(&singleton_bytes).unwrap(),
        singleton
    );
}

#[test]
fn proof_profile_component_count_duplicates_order_and_trailing_are_closed() {
    let mut components = Vec::new();
    for index in 0..65 {
        components.push(component(&format!("component-{index:02}"), &[]));
    }
    assert!(SfsProofProfileV1::new(
        digest(2),
        digest(3),
        "babylon.sfs.audit.v1",
        digest(4),
        digest(5),
        components[..64].to_vec(),
    )
    .is_ok());
    assert_eq!(
        SfsProofProfileV1::new(
            digest(2),
            digest(3),
            "babylon.sfs.audit.v1",
            digest(4),
            digest(5),
            components,
        ),
        Err(SfsProfileRecordError::Wire(SfsWireError::CountTooLarge {
            field: "components",
            limit: 64,
            actual: 65,
        }))
    );
    assert_eq!(
        SfsProofProfileV1::new(
            digest(2),
            digest(3),
            "babylon.sfs.audit.v1",
            digest(4),
            digest(5),
            vec![component("a", &[]), component("a", &[])],
        ),
        Err(SfsProfileRecordError::DuplicateComponentId)
    );
    let reversed_payload = literal_proof_payload(&[
        literal_component_envelope("b", &[]),
        literal_component_envelope("a", &[]),
    ]);
    let reversed = literal_envelope(SfsProofProfileV1::DOMAIN, &reversed_payload);
    assert_eq!(
        decode_envelope::<SfsProofProfileV1>(&reversed),
        Err(SfsProfileRecordError::Wire(SfsWireError::OutOfOrder {
            field: "components",
        }))
    );
    let mut trailing_payload = literal_proof_payload(&[]);
    trailing_payload.push(0xaa);
    let trailing = literal_envelope(SfsProofProfileV1::DOMAIN, &trailing_payload);
    assert_eq!(
        decode_envelope::<SfsProofProfileV1>(&trailing),
        Err(SfsProfileRecordError::Wire(SfsWireError::TrailingBytes {
            count: 1,
        }))
    );
}

#[test]
fn causal_cone_canonicalizes_each_set_and_rejects_named_duplicates() {
    let cone = CausalConeV1::new(
        vec!["root-b".to_owned(), "root-a".to_owned()],
        vec!["sink-b".to_owned(), "sink-a".to_owned()],
        vec!["component-b".to_owned(), "component-a".to_owned()],
    )
    .unwrap();
    assert_eq!(cone.roots(), &["root-a", "root-b"]);
    assert_eq!(cone.sinks(), &["sink-a", "sink-b"]);
    assert_eq!(cone.components(), &["component-a", "component-b"]);
    for (set, roots, sinks, components) in [
        (
            "roots",
            vec!["a".to_owned(), "a".to_owned()],
            vec![],
            vec![],
        ),
        (
            "sinks",
            vec![],
            vec!["a".to_owned(), "a".to_owned()],
            vec![],
        ),
        (
            "components",
            vec![],
            vec![],
            vec!["a".to_owned(), "a".to_owned()],
        ),
    ] {
        assert_eq!(
            CausalConeV1::new(roots, sinks, components),
            Err(SfsProfileRecordError::DuplicateConeId { set })
        );
    }
}

#[test]
fn causal_cone_literal_wire_order_and_64_65_bounds_are_exact() {
    let cone = CausalConeV1::new(
        vec!["root".to_owned()],
        vec!["sink".to_owned()],
        vec!["component".to_owned()],
    )
    .unwrap();
    let mut payload = literal_set(&["root"]);
    payload.extend_from_slice(&literal_set(&["sink"]));
    payload.extend_from_slice(&literal_set(&["component"]));
    let expected = literal_envelope(CausalConeV1::DOMAIN, &payload);
    assert_eq!(canonical_envelope(&cone).unwrap(), expected);
    assert_eq!(decode_envelope::<CausalConeV1>(&expected).unwrap(), cone);

    let mut ids = Vec::new();
    for index in 0..65 {
        ids.push(format!("id-{index:02}"));
    }
    assert!(CausalConeV1::new(ids[..64].to_vec(), vec![], vec![]).is_ok());
    assert_eq!(
        CausalConeV1::new(ids, vec![], vec![]),
        Err(SfsProfileRecordError::Wire(SfsWireError::CountTooLarge {
            field: "roots",
            limit: 64,
            actual: 65,
        }))
    );
    let mut reversed_payload = literal_set(&["b", "a"]);
    reversed_payload.extend_from_slice(&literal_set(&[]));
    reversed_payload.extend_from_slice(&literal_set(&[]));
    let reversed = literal_envelope(CausalConeV1::DOMAIN, &reversed_payload);
    assert_eq!(
        decode_envelope::<CausalConeV1>(&reversed),
        Err(SfsProfileRecordError::Wire(SfsWireError::OutOfOrder {
            field: "roots",
        }))
    );
    assert_eq!(
        CausalConeV1::new(vec![String::new()], vec![], vec![]),
        Err(SfsProfileRecordError::Wire(SfsWireError::StringEmpty {
            field: "roots",
        }))
    );
    assert_eq!(
        CausalConeV1::new(vec!["a".repeat(257)], vec![], vec![]),
        Err(SfsProfileRecordError::Wire(SfsWireError::StringTooLong {
            field: "roots",
            limit: 256,
            actual: 257,
        }))
    );
}

fn intervention_row(
    operation: InterventionOperationV1,
    stable: Digest32,
) -> InterventionDeltaRowV1 {
    let (control, intervention) = match operation {
        InterventionOperationV1::Add => (zero_digest(), digest(10)),
        InterventionOperationV1::Remove => (digest(11), zero_digest()),
        InterventionOperationV1::Replace => (digest(12), digest(13)),
    };
    InterventionDeltaRowV1::new(operation, stable, control, intervention).unwrap()
}

fn literal_delta_payload(
    kind: u8,
    count: u32,
    rows: &[(u8, Digest32, Digest32, Digest32)],
) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.push(kind);
    payload.extend_from_slice(&count.to_be_bytes());
    for index in 0..65_535 {
        if index >= rows.len() {
            break;
        }
        let (operation, stable, control, intervention) = rows[index];
        payload.push(operation);
        payload.extend_from_slice(stable.as_bytes());
        payload.extend_from_slice(control.as_bytes());
        payload.extend_from_slice(intervention.as_bytes());
    }
    payload
}

#[test]
fn intervention_operations_enforce_exact_zero_and_nonzero_sides() {
    assert!(InterventionDeltaRowV1::new(
        InterventionOperationV1::Add,
        digest(1),
        zero_digest(),
        digest(2),
    )
    .is_ok());
    assert!(InterventionDeltaRowV1::new(
        InterventionOperationV1::Remove,
        digest(1),
        digest(2),
        zero_digest(),
    )
    .is_ok());
    assert!(InterventionDeltaRowV1::new(
        InterventionOperationV1::Replace,
        digest(1),
        digest(2),
        digest(3),
    )
    .is_ok());
    for (operation, control, intervention) in [
        (InterventionOperationV1::Add, digest(2), digest(3)),
        (InterventionOperationV1::Add, digest(2), zero_digest()),
        (InterventionOperationV1::Add, zero_digest(), zero_digest()),
        (
            InterventionOperationV1::Remove,
            zero_digest(),
            zero_digest(),
        ),
        (InterventionOperationV1::Remove, zero_digest(), digest(3)),
        (InterventionOperationV1::Remove, digest(2), digest(3)),
        (
            InterventionOperationV1::Replace,
            zero_digest(),
            zero_digest(),
        ),
        (InterventionOperationV1::Replace, zero_digest(), digest(3)),
        (InterventionOperationV1::Replace, digest(2), zero_digest()),
        (InterventionOperationV1::Replace, digest(2), digest(2)),
    ] {
        assert_eq!(
            InterventionDeltaRowV1::new(operation, digest(1), control, intervention),
            Err(SfsProfileRecordError::InvalidInterventionRow)
        );
    }
}

#[test]
fn intervention_delta_literal_wire_sorts_and_rejects_duplicate_stable_ids() {
    let stable_a = digest(20);
    let stable_b = digest(21);
    let value = InterventionDeltaV1::new(
        DifferingLedgerKindV1::PracticeAttempt,
        vec![
            intervention_row(InterventionOperationV1::Replace, stable_b),
            intervention_row(InterventionOperationV1::Add, stable_a),
        ],
    )
    .unwrap();
    assert_eq!(value.ledger_kind(), DifferingLedgerKindV1::PracticeAttempt);
    let payload = literal_delta_payload(
        1,
        2,
        &[
            (0, stable_a, zero_digest(), digest(10)),
            (2, stable_b, digest(12), digest(13)),
        ],
    );
    let expected = literal_envelope(InterventionDeltaV1::DOMAIN, &payload);
    assert_eq!(canonical_envelope(&value).unwrap(), expected);
    assert_eq!(
        decode_envelope::<InterventionDeltaV1>(&expected).unwrap(),
        value
    );
    let duplicate = intervention_row(InterventionOperationV1::Add, stable_a);
    assert_eq!(
        InterventionDeltaV1::new(
            DifferingLedgerKindV1::ExogenousInput,
            vec![duplicate.clone(), duplicate],
        ),
        Err(SfsProfileRecordError::Wire(SfsWireError::DuplicateEntry {
            field: "intervention_rows",
        }))
    );
}

#[test]
fn intervention_delta_decoder_closes_kind_operation_order_and_row_rules() {
    let invalid_kind = literal_envelope(
        InterventionDeltaV1::DOMAIN,
        &literal_delta_payload(2, 0, &[]),
    );
    assert_eq!(
        decode_envelope::<InterventionDeltaV1>(&invalid_kind),
        Err(SfsProfileRecordError::InvalidLedgerKind { value: 2 })
    );
    let invalid_operation = literal_envelope(
        InterventionDeltaV1::DOMAIN,
        &literal_delta_payload(0, 1, &[(3, digest(1), zero_digest(), digest(2))]),
    );
    assert_eq!(
        decode_envelope::<InterventionDeltaV1>(&invalid_operation),
        Err(SfsProfileRecordError::InvalidInterventionOperation { value: 3 })
    );
    let invalid_row = literal_envelope(
        InterventionDeltaV1::DOMAIN,
        &literal_delta_payload(0, 1, &[(0, digest(1), digest(2), digest(3))]),
    );
    assert_eq!(
        decode_envelope::<InterventionDeltaV1>(&invalid_row),
        Err(SfsProfileRecordError::InvalidInterventionRow)
    );
    let reversed = literal_envelope(
        InterventionDeltaV1::DOMAIN,
        &literal_delta_payload(
            0,
            2,
            &[
                (0, digest(2), zero_digest(), digest(3)),
                (0, digest(1), zero_digest(), digest(3)),
            ],
        ),
    );
    assert_eq!(
        decode_envelope::<InterventionDeltaV1>(&reversed),
        Err(SfsProfileRecordError::Wire(SfsWireError::OutOfOrder {
            field: "intervention_rows",
        }))
    );
    let duplicate = literal_envelope(
        InterventionDeltaV1::DOMAIN,
        &literal_delta_payload(
            0,
            2,
            &[
                (0, digest(1), zero_digest(), digest(3)),
                (0, digest(1), zero_digest(), digest(4)),
            ],
        ),
    );
    assert_eq!(
        decode_envelope::<InterventionDeltaV1>(&duplicate),
        Err(SfsProfileRecordError::Wire(SfsWireError::DuplicateEntry {
            field: "intervention_rows",
        }))
    );
}

#[test]
fn intervention_count_maximum_succeeds_and_plus_one_preflights() {
    let mut rows = Vec::with_capacity(65_536);
    for index in 0..65_536 {
        rows.push(intervention_row(
            InterventionOperationV1::Add,
            indexed_digest(index),
        ));
    }
    assert_eq!(
        InterventionDeltaV1::new(DifferingLedgerKindV1::ExogenousInput, rows.clone()),
        Err(SfsProfileRecordError::Wire(SfsWireError::CountTooLarge {
            field: "intervention_rows",
            limit: 65_535,
            actual: 65_536,
        }))
    );
    rows.pop();
    let maximum = InterventionDeltaV1::new(DifferingLedgerKindV1::ExogenousInput, rows).unwrap();
    let encoded = canonical_envelope(&maximum).unwrap();
    assert_eq!(payload_length::<InterventionDeltaV1>(&encoded), 6_356_900);
    assert_eq!(
        decode_envelope::<InterventionDeltaV1>(&encoded).unwrap(),
        maximum
    );
    let empty = InterventionDeltaV1::new(DifferingLedgerKindV1::PracticeAttempt, vec![]).unwrap();
    let empty_bytes = canonical_envelope(&empty).unwrap();
    assert_eq!(payload_length::<InterventionDeltaV1>(&empty_bytes), 5);
    assert_eq!(
        decode_envelope::<InterventionDeltaV1>(&empty_bytes).unwrap(),
        empty
    );
}

#[test]
fn intervention_decoder_preflights_product_count_truncation_and_trailing() {
    let overflow = literal_envelope(
        InterventionDeltaV1::DOMAIN,
        &literal_delta_payload(0, u32::MAX, &[]),
    );
    assert_eq!(
        decode_envelope::<InterventionDeltaV1>(&overflow),
        Err(SfsProfileRecordError::Wire(
            SfsWireError::ArithmeticOverflow {
                field: "intervention_rows",
            }
        ))
    );
    let too_many = literal_envelope(
        InterventionDeltaV1::DOMAIN,
        &literal_delta_payload(0, 65_536, &[]),
    );
    assert_eq!(
        decode_envelope::<InterventionDeltaV1>(&too_many),
        Err(SfsProfileRecordError::Wire(SfsWireError::CountTooLarge {
            field: "intervention_rows",
            limit: 65_535,
            actual: 65_536,
        }))
    );
    let mut truncated_payload = literal_delta_payload(0, 1, &[]);
    truncated_payload.push(3);
    let truncated = literal_envelope(InterventionDeltaV1::DOMAIN, &truncated_payload);
    assert_eq!(
        decode_envelope::<InterventionDeltaV1>(&truncated),
        Err(SfsProfileRecordError::Wire(SfsWireError::TruncatedEnvelope))
    );
    let mut trailing_payload = literal_delta_payload(0, 0, &[]);
    trailing_payload.push(3);
    let trailing = literal_envelope(InterventionDeltaV1::DOMAIN, &trailing_payload);
    assert_eq!(
        decode_envelope::<InterventionDeltaV1>(&trailing),
        Err(SfsProfileRecordError::Wire(SfsWireError::TrailingBytes {
            count: 1,
        }))
    );
}

fn persistence(
    kind: DifferingLedgerKindV1,
    post_width: u16,
    separations: Vec<f64>,
) -> Result<PersistenceComparisonV1, SfsProfileRecordError> {
    PersistenceComparisonV1::new(
        digest(30),
        digest(31),
        kind,
        digest(32),
        digest(33),
        digest(34),
        35,
        post_width,
        separations,
    )
}

fn literal_persistence_payload(kind: u8, separations: &[f64], class: u8) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(digest(30).as_bytes());
    payload.extend_from_slice(digest(31).as_bytes());
    payload.push(kind);
    payload.extend_from_slice(digest(32).as_bytes());
    payload.extend_from_slice(digest(33).as_bytes());
    payload.extend_from_slice(digest(34).as_bytes());
    payload.extend_from_slice(&35_u64.to_be_bytes());
    payload.extend_from_slice(&2_u16.to_be_bytes());
    payload.extend_from_slice(
        &u16::try_from(separations.len())
            .expect("test separation count fits u16")
            .to_be_bytes(),
    );
    for index in 0..53 {
        if index >= separations.len() {
            break;
        }
        payload.extend_from_slice(&separations[index].to_bits().to_be_bytes());
    }
    payload.push(class);
    payload
}

#[test]
fn persistence_comparison_literal_wire_accessors_and_all_classes_are_exact() {
    let value = persistence(
        DifferingLedgerKindV1::PracticeAttempt,
        2,
        vec![2.0, 1.0, -1.0],
    )
    .unwrap();
    assert_eq!(value.control_trace_digest(), digest(30));
    assert_eq!(value.intervention_trace_digest(), digest(31));
    assert_eq!(
        value.differing_ledger_kind(),
        DifferingLedgerKindV1::PracticeAttempt
    );
    assert_eq!(value.control_differing_ledger_digest(), digest(32));
    assert_eq!(value.intervention_differing_ledger_digest(), digest(33));
    assert_eq!(value.intervention_delta_digest(), digest(34));
    let expected_payload = literal_persistence_payload(1, &[2.0, 1.0, -1.0], 1);
    let expected = literal_envelope(PersistenceComparisonV1::DOMAIN, &expected_payload);
    assert_eq!(canonical_envelope(&value).unwrap(), expected);
    assert_eq!(
        decode_envelope::<PersistenceComparisonV1>(&expected).unwrap(),
        value
    );
    for (separations, class) in [
        (vec![2.0, 0.0, 0.0], 0),
        (vec![2.0, 1.0, -1.0], 1),
        (vec![2.0, 1.0, 0.5], 2),
        (vec![2.0, 0.0, 1.0], 3),
    ] {
        let encoded = canonical_envelope(
            &persistence(DifferingLedgerKindV1::ExogenousInput, 2, separations).unwrap(),
        )
        .unwrap();
        assert_eq!(encoded.last(), Some(&class));
        assert!(decode_envelope::<PersistenceComparisonV1>(&encoded).is_ok());
    }
}

#[test]
fn persistence_requires_exact_p_plus_one_and_maximum_width() {
    assert_eq!(
        persistence(DifferingLedgerKindV1::ExogenousInput, 2, vec![1.0, 2.0]),
        Err(SfsProfileRecordError::Classification(
            PersistenceClassError::WrongLength {
                expected: 3,
                actual: 2,
            }
        ))
    );
    assert_eq!(
        persistence(DifferingLedgerKindV1::ExogenousInput, 2, vec![1.0; 4]),
        Err(SfsProfileRecordError::Classification(
            PersistenceClassError::WrongLength {
                expected: 3,
                actual: 4,
            }
        ))
    );
    let maximum = persistence(DifferingLedgerKindV1::ExogenousInput, 52, vec![0.0; 53]).unwrap();
    let maximum_bytes = canonical_envelope(&maximum).unwrap();
    assert_eq!(
        payload_length::<PersistenceComparisonV1>(&maximum_bytes),
        598
    );
    assert_eq!(
        decode_envelope::<PersistenceComparisonV1>(&maximum_bytes).unwrap(),
        maximum
    );
}

#[test]
fn persistence_decoder_rejects_ledger_and_class_codes_before_recompute_mismatch() {
    let valid_payload = literal_persistence_payload(0, &[2.0, 1.0, -1.0], 1);
    let mut invalid_ledger_payload = valid_payload.clone();
    invalid_ledger_payload[64] = 2;
    let invalid_ledger = literal_envelope(PersistenceComparisonV1::DOMAIN, &invalid_ledger_payload);
    assert_eq!(
        decode_envelope::<PersistenceComparisonV1>(&invalid_ledger),
        Err(SfsProfileRecordError::InvalidLedgerKind { value: 2 })
    );
    let mut invalid_class_payload = valid_payload.clone();
    *invalid_class_payload.last_mut().unwrap() = 4;
    let separation_start = 64 + 1 + 96 + 8 + 2 + 2;
    invalid_class_payload[separation_start..separation_start + 8]
        .copy_from_slice(&f64::NAN.to_bits().to_be_bytes());
    let invalid_class = literal_envelope(PersistenceComparisonV1::DOMAIN, &invalid_class_payload);
    assert_eq!(
        decode_envelope::<PersistenceComparisonV1>(&invalid_class),
        Err(SfsProfileRecordError::Wire(SfsWireError::InvalidCode {
            field: "persistence_class",
            value: 4,
        }))
    );
    let mut nonfinite_payload = valid_payload.clone();
    nonfinite_payload[separation_start..separation_start + 8]
        .copy_from_slice(&f64::NAN.to_bits().to_be_bytes());
    let nonfinite = literal_envelope(PersistenceComparisonV1::DOMAIN, &nonfinite_payload);
    assert_eq!(
        decode_envelope::<PersistenceComparisonV1>(&nonfinite),
        Err(SfsProfileRecordError::Classification(
            PersistenceClassError::NonFiniteSeparation { index: 0 }
        ))
    );
    let mut mismatch_payload = valid_payload;
    *mismatch_payload.last_mut().unwrap() = 0;
    let mismatch = literal_envelope(PersistenceComparisonV1::DOMAIN, &mismatch_payload);
    assert_eq!(
        decode_envelope::<PersistenceComparisonV1>(&mismatch),
        Err(SfsProfileRecordError::ClassificationMismatch {
            stored: 0,
            computed: 1,
        })
    );
}
