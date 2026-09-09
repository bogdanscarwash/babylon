use babylon_practice_contract::*;

const PRACTICE_SCHEMA: &[u8] = include_bytes!("../../../../contracts/practice_contract_v1.yaml");

const EXPECTED_ERRORS: [(PracticeContractError, u16); 44] = [
    (PracticeContractError::PracticeDomain, 1),
    (PracticeContractError::PracticeSchemaVersion, 2),
    (PracticeContractError::PracticeEnumCode, 3),
    (PracticeContractError::PracticeLength, 5),
    (PracticeContractError::PracticeTruncated, 6),
    (PracticeContractError::PracticeTrailingBytes, 7),
    (PracticeContractError::PracticeBoolean, 9),
    (PracticeContractError::PracticeParameter, 10),
    (PracticeContractError::PracticeParameterLimit, 11),
    (PracticeContractError::PracticeParameterLength, 12),
    (PracticeContractError::PracticeEvidenceLimit, 13),
    (PracticeContractError::PracticeEvidenceOrder, 14),
    (PracticeContractError::PracticeEvidenceDuplicate, 15),
    (PracticeContractError::PracticeTickOverflow, 16),
    (PracticeContractError::PracticeTickMismatch, 17),
    (PracticeContractError::PracticeAuthorityRegistryLimit, 18),
    (PracticeContractError::PracticeAuthorityRegistryOrder, 19),
    (
        PracticeContractError::PracticeAuthorityRegistryDuplicate,
        20,
    ),
    (PracticeContractError::PracticeAuthorityUnregistered, 21),
    (PracticeContractError::PracticeActorMismatch, 22),
    (PracticeContractError::PracticeAuthorityContentMismatch, 23),
    (PracticeContractError::PracticeQuoteContentMismatch, 24),
    (PracticeContractError::PracticeQuoteCostMismatch, 25),
    (PracticeContractError::PracticeBatchLimit, 26),
    (PracticeContractError::PracticeDuplicateActor, 27),
    (PracticeContractError::PracticeBudgetNonfinite, 28),
    (PracticeContractError::PracticeBudgetNegative, 29),
    (PracticeContractError::PracticeBudgetFractional, 30),
    (PracticeContractError::PracticeBudgetRange, 31),
    (PracticeContractError::PracticeBudgetRoundtrip, 32),
    (PracticeContractError::PracticeBudgetInsufficient, 33),
    (PracticeContractError::PracticeBudgetArithmetic, 34),
    (PracticeContractError::PracticeFootprintLimit, 35),
    (PracticeContractError::PracticeFootprintOrder, 36),
    (PracticeContractError::PracticeFootprintDuplicate, 37),
    (PracticeContractError::PracticeFootprintSource, 38),
    (
        PracticeContractError::PracticeFootprintStrengthNonfinite,
        39,
    ),
    (
        PracticeContractError::PracticeFootprintStrengthNonpositive,
        40,
    ),
    (PracticeContractError::PracticeTopologyOrganizationLimit, 41),
    (PracticeContractError::PracticeTopologyOrganizationOrder, 42),
    (
        PracticeContractError::PracticeTopologyOrganizationDuplicate,
        43,
    ),
    (PracticeContractError::PracticeTopologyBudgetMissing, 44),
    (PracticeContractError::PracticeTopologyEdgeOrder, 45),
    (PracticeContractError::PracticeTopologyEdgeDuplicate, 46),
];

#[test]
fn closed_codes_and_machine_mappings_are_exact() {
    assert_eq!(
        PracticeIdV1::try_from(0_u8),
        Err(PracticeContractError::PracticeEnumCode)
    );
    assert_eq!(
        PracticeIdV1::try_from(4_u8),
        Err(PracticeContractError::PracticeEnumCode)
    );
    assert_eq!(
        practice_machine_verb(PracticeIdV1::Organize),
        MachineVerbV1 {
            stem: VerbStemV1::Mobilize,
            mode: Some(VerbModeV1::Canvass),
        }
    );
    assert_eq!(
        practice_machine_verb(PracticeIdV1::Agitate),
        MachineVerbV1 {
            stem: VerbStemV1::Mobilize,
            mode: Some(VerbModeV1::Agitate),
        }
    );
    assert_eq!(
        practice_machine_verb(PracticeIdV1::MutualAid),
        MachineVerbV1 {
            stem: VerbStemV1::Aid,
            mode: None,
        }
    );
}

#[test]
fn contract_error_codes_are_exact_and_holes_refuse() {
    for (error, code) in EXPECTED_ERRORS {
        assert_eq!(u16::from(error), code);
        assert_eq!(PracticeContractError::try_from(code), Ok(error));
    }
    assert!(PracticeContractError::try_from(0_u16).is_err());
    assert!(PracticeContractError::try_from(4_u16).is_err());
    assert!(PracticeContractError::try_from(8_u16).is_err());
    assert!(PracticeContractError::try_from(47_u16).is_err());
}

#[test]
fn schema_field_order_is_the_contract_order() {
    assert_eq!(
        SOLIDARITY_FOOTPRINT_EDGE_V1_FIELD_ORDER,
        [
            "source_org_node_id_u64",
            "target_domain_u8",
            "target_class_node_id_u64",
            "strength_f64_bits_u64",
        ]
    );
    assert_eq!(
        ORGANIZATION_PRACTICE_TOPOLOGY_EDGE_V1_FIELD_ORDER,
        ["target_domain", "target_class_node_id_u64"]
    );
    assert_eq!(
        ORGANIZATION_PRACTICE_TOPOLOGY_ROW_V1_FIELD_ORDER,
        [
            "node_id_u64",
            "active_bool",
            "action_budget_storage_f64_bits_u64",
            "edges",
        ]
    );
    assert_eq!(
        ORGANIZATION_PRACTICE_TOPOLOGY_V1_FIELD_ORDER,
        ["organizations"]
    );
}

#[test]
fn schema_wire_domain_bytes_and_terminator_are_exact() {
    assert_eq!(
        PRACTICE_INPUT_AUTHORITY_V1_DOMAIN_BYTES,
        b"babylon.practice-input-authority.v1"
    );
    assert_eq!(
        PRACTICE_INTENT_V1_DOMAIN_BYTES,
        b"babylon.practice-intent.v1"
    );
    assert_eq!(
        ORGANIZATION_BUDGET_DELTA_V1_DOMAIN_BYTES,
        b"babylon.organization-budget-delta.v1"
    );
    assert_eq!(PRACTICE_WIRE_DOMAIN_TERMINATOR_BYTES, b"\x00");
}

#[test]
fn schema_record_shapes_use_typed_fixed_width_values() {
    let digest = [0_u8; 32];
    let parameter = PracticeParameterV1 {
        key_u8: 0,
        value_kind_u8: 0,
        value_length_u16: 0,
        value_bytes: Vec::new(),
    };
    let intent = PracticeIntentV1 {
        schema_version: 1,
        submit_after_tick: 0,
        resolve_tick: 1,
        actor_org_id: 1,
        practice_id: PracticeIdV1::Organize,
        target_domain: PracticeTargetDomainV1::SocialClass,
        target_node_id: 2,
        quoted_content_digest: digest,
        quoted_action_budget_cost: 1,
        parameters: vec![parameter],
        evidence_digests: vec![digest],
    };
    assert_eq!(intent.parameters.len(), 1);
    let row = OrganizationPracticeTopologyRowV1 {
        node_id_u64: 1,
        active_bool: true,
        action_budget_storage_f64_bits_u64: Some(0),
        edges: vec![OrganizationPracticeTopologyEdgeV1 {
            target_domain: PracticeTargetDomainV1::SocialClass,
            target_class_node_id_u64: 2,
        }],
    };
    assert_eq!(row.action_budget_storage_f64_bits_u64, Some(0));
}

#[test]
fn schema_sequence_validators_refuse_plus_one() {
    let digest = [0_u8; 32];
    let parameter = PracticeParameterV1 {
        key_u8: 0,
        value_kind_u8: 0,
        value_length_u16: 0,
        value_bytes: Vec::new(),
    };
    let intent = PracticeIntentV1 {
        schema_version: 1,
        submit_after_tick: 0,
        resolve_tick: 1,
        actor_org_id: 1,
        practice_id: PracticeIdV1::Organize,
        target_domain: PracticeTargetDomainV1::SocialClass,
        target_node_id: 2,
        quoted_content_digest: digest,
        quoted_action_budget_cost: 1,
        parameters: vec![parameter; MAX_PARAMETERS + 1],
        evidence_digests: Vec::new(),
    };
    assert_eq!(
        validate_intent_collection_bounds(&intent),
        Err(PracticeContractError::PracticeParameterLimit)
    );
    let policy = PolicyAuthorityPairV1 {
        producer_content_digest: digest,
        actor_org_id: 1,
    };
    let authority = PracticeAuthorityContextV1 {
        player_org_id: 1,
        player_gateway_content_digest: digest,
        policy_authorities: vec![policy; MAX_POLICY_AUTHORITY_PAIRS + 1],
    };
    assert_eq!(
        validate_authority_context_collection_bounds(&authority),
        Err(PracticeContractError::PracticeAuthorityRegistryLimit)
    );
    let edge = OrganizationPracticeTopologyEdgeV1 {
        target_domain: PracticeTargetDomainV1::SocialClass,
        target_class_node_id_u64: 2,
    };
    let topology = OrganizationPracticeTopologyV1 {
        organizations: vec![OrganizationPracticeTopologyRowV1 {
            node_id_u64: 1,
            active_bool: true,
            action_budget_storage_f64_bits_u64: Some(0),
            edges: vec![edge; MAX_ORG_SOLIDARITY_EDGES_PER_ORG + 1],
        }],
    };
    assert_eq!(
        validate_topology_collection_bounds(&topology),
        Err(PracticeContractError::PracticeFootprintLimit)
    );
}

#[test]
fn crate_manifest_has_one_production_dependency() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("[dependencies]\nbabylon-kernel"));
    assert!(manifest.contains("[dev-dependencies]\nserde_json"));
    for forbidden in [
        "babylon-graph",
        "babylon-bsl",
        "babylon-tick",
        "babylon-persistence",
        "babylon-client",
    ] {
        assert!(!manifest.contains(forbidden));
    }
}

#[test]
fn language_neutral_schema_bytes_are_bound_to_rust() {
    assert_eq!(
        babylon_kernel::sha256_of(PRACTICE_SCHEMA),
        PRACTICE_CONTRACT_SOURCE_SHA256
    );
}

#[test]
fn designed_budget_defaults_are_typed_and_exact() {
    assert_eq!(
        DEFAULT_PRACTICE_BUDGET_TERMS_V1,
        PracticeBudgetTermsV1 {
            initial: 1,
            weekly_credit_cap: 1,
            storage_ceiling: 4,
            organize_cost: 1,
            agitate_cost: 1,
            mutual_aid_cost: 1,
        }
    );
}
