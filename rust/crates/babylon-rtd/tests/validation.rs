use babylon_rtd::*;

const ZERO_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const QCEW_DIGEST: &str = "34c2bbb935f79b3c8076a97092b004b14cca120e8272b93c35b3ac9dc2721d13";
const LODES_DIGEST: &str = "d3745f8def09cd8c7a38e1870e6ec2c1853e210b777d8e8358cfce36665bd64d";

fn identity(local_id: &str, domain: &str, authority: &str) -> TypedIdentityV1 {
    TypedIdentityV1 {
        domain: domain.to_owned(),
        authority: authority.to_owned(),
        local_id: local_id.to_owned(),
    }
}

fn test_identity(local_id: &str) -> TypedIdentityV1 {
    identity(local_id, "entity", "test")
}

fn registry_identity(domain: &str, local_id: &str) -> TypedIdentityV1 {
    identity(local_id, domain, "babylon.rtd.v1")
}

fn surface() -> DecisionSurfaceV1 {
    DecisionSurfaceV1 {
        question_id: identity("administrative-question", "question", "test"),
        signal_refs: Vec::new(),
        action_refs: Vec::new(),
        receipt_refs: Vec::new(),
        archive_subject_refs: Vec::new(),
    }
}

fn base_draft() -> RtdDossierDraftV1 {
    RtdDossierDraftV1 {
        schema: RTD_V1_SCHEMA_ID.to_owned(),
        schema_version: 1,
        projection_version: 1,
        audience: AudienceV1::AdminMaterial,
        durability: DurabilityV1::InMemory,
        verified_tick: 7,
        graph_state_hash: ZERO_DIGEST.to_owned(),
        nominal_world_hash: ZERO_DIGEST.to_owned(),
        reference_digests: Vec::new(),
        definitions_digest: ZERO_DIGEST.to_owned(),
        template_digest: ZERO_DIGEST.to_owned(),
        fog_policy_digest: None,
        knowledge_context_digest: None,
        actor: None,
        focus: Vec::new(),
        scale_memberships: Vec::new(),
        facets: Vec::new(),
        dyads: Vec::new(),
        hyperedges: Vec::new(),
        flows: Vec::new(),
        gaps: Vec::new(),
        provenance: Vec::new(),
        decision_surface: surface(),
    }
}

fn reference(local_id: &str, digest: &str) -> ReferenceDigestV1 {
    ReferenceDigestV1 {
        reference_id: identity(local_id, "reference-artifact", "babylon.data.v7"),
        sha256_hex: digest.to_owned(),
        artifact_schema_id_or_null: None,
        vintage: "2023".to_owned(),
        evidence_class: EvidenceClassV1::Observed,
    }
}

fn county_facet() -> FacetV1 {
    FacetV1 {
        facet_id: identity("facet-1", "facet", "test"),
        family: FacetFamilyV1::ProductionCirculation,
        subject_ref: identity("26163", "county", "census"),
        metric_id: registry_identity("metric", "production/qcew-county-employment"),
        unit_id: registry_identity("unit", "jobs"),
        native_scale: registry_identity("native-scale", "county-ownership-year"),
        coordinates: vec![
            DimensionCoordinateV1 {
                dimension_ref: registry_identity("dimension", "county"),
                member_ref: identity("26163", "county", "census"),
            },
            DimensionCoordinateV1 {
                dimension_ref: registry_identity("dimension", "ownership"),
                member_ref: identity("0", "ownership", "bls"),
            },
        ],
        vintage: "2023".to_owned(),
        status: StatusV1::Present,
        value_kind: ValueKindV1::Uint64Bits,
        value_bits_or_null: Some("0000000000000000".to_owned()),
        coverage: CoverageV1::Complete,
        evidence_class: EvidenceClassV1::Observed,
        provenance_refs: Vec::new(),
    }
}

fn with_county_facet(facet: FacetV1) -> RtdDossierDraftV1 {
    let mut draft = base_draft();
    draft.facets.push(facet);
    draft
        .reference_digests
        .push(reference("fact_qcew_county_rollup", QCEW_DIGEST));
    draft
}

fn dyad(kind: DyadKindV1) -> DyadV1 {
    DyadV1 {
        relation_id: identity("dyad", "dyad", "test"),
        relation_kind: kind,
        from_ref: identity("actor", "organization", "test"),
        to_ref: identity("node", "county", "test"),
        native_scale: registry_identity("native-scale", "actor-node-verified-tick"),
        status: StatusV1::Present,
        coverage: CoverageV1::Complete,
        payload_facets: Vec::new(),
        evidence_class: EvidenceClassV1::Derived,
        provenance_refs: Vec::new(),
    }
}

fn lodes_facet(flow_id: TypedIdentityV1) -> FacetV1 {
    FacetV1 {
        facet_id: identity("lodes-facet", "facet", "test"),
        family: FacetFamilyV1::ProductionCirculation,
        subject_ref: flow_id,
        metric_id: registry_identity("metric", "circulation/lodes-county-commuter-total-jobs"),
        unit_id: registry_identity("unit", "jobs"),
        native_scale: registry_identity("native-scale", "home-county-work-county-year"),
        coordinates: vec![
            DimensionCoordinateV1 {
                dimension_ref: registry_identity("dimension", "home-county"),
                member_ref: identity("26163", "county", "census"),
            },
            DimensionCoordinateV1 {
                dimension_ref: registry_identity("dimension", "work-county"),
                member_ref: identity("26125", "county", "census"),
            },
        ],
        vintage: "2022".to_owned(),
        status: StatusV1::Present,
        value_kind: ValueKindV1::Uint64Bits,
        value_bits_or_null: Some("0000000000000001".to_owned()),
        coverage: CoverageV1::Complete,
        evidence_class: EvidenceClassV1::Derived,
        provenance_refs: Vec::new(),
    }
}

fn commuter_flow(flow_id: TypedIdentityV1, payload: Vec<TypedIdentityV1>) -> ReferenceFlowV1 {
    ReferenceFlowV1 {
        flow_id,
        flow_kind: FlowKindV1::CommuterJobs,
        origin_ref: identity("26163", "county", "census"),
        destination_ref: identity("26125", "county", "census"),
        payload_facets: payload,
        native_scale: registry_identity("native-scale", "home-county-work-county-year"),
        status: StatusV1::Present,
        coverage: CoverageV1::Complete,
        evidence_class: EvidenceClassV1::Derived,
        provenance_refs: Vec::new(),
    }
}

fn assert_error(expected: RtdError, draft: &RtdDossierDraftV1) {
    assert_eq!(validate_draft(draft), Err(expected));
}

#[test]
fn empty_administrative_draft_and_focus_permutations_validate() {
    assert_eq!(validate_draft(&base_draft()), Ok(()));
    let mut first = base_draft();
    first.focus = vec![test_identity("a"), test_identity("b")];
    let mut second = base_draft();
    second.focus = vec![test_identity("b"), test_identity("a")];
    assert_eq!(validate_draft(&first), Ok(()));
    assert_eq!(validate_draft(&second), Ok(()));
    let mut ordered_surface = base_draft();
    ordered_surface.focus.push(test_identity("a"));
    ordered_surface.decision_surface.signal_refs = vec![test_identity("a"), test_identity("a")];
    assert_eq!(validate_draft(&ordered_surface), Ok(()));
}

#[test]
fn untrusted_json_boundary_classifies_shape_and_normalizes_float_negative_zero() {
    assert_eq!(parse_draft_json(b"{"), Err(RtdError::Json));
    assert_eq!(parse_draft_json(b"{}"), Err(RtdError::Json));
    assert_eq!(parse_draft_json(b"{}{}"), Err(RtdError::Json));
    let deep = format!("{}{}", "[".repeat(33), "]".repeat(33));
    assert_eq!(parse_draft_json(deep.as_bytes()), Err(RtdError::JsonDepth));
    let duplicate = br#"{"schema":"x","schema":"y"}"#;
    assert_eq!(parse_draft_json(duplicate), Err(RtdError::DuplicateKey));
    let unknown = valid_json().replace(
        "\"decision_surface\"",
        "\"unexpected\":true,\"decision_surface\"",
    );
    assert_eq!(
        parse_draft_json(unknown.as_bytes()),
        Err(RtdError::UnknownField)
    );
    let bad_enum = valid_json().replace("\"ADMIN_MATERIAL\"", "\"SECRET\"");
    assert_eq!(parse_draft_json(bad_enum.as_bytes()), Err(RtdError::Enum));
    let bad_version = valid_json().replace("\"schema_version\":1", "\"schema_version\":2");
    assert_eq!(
        parse_draft_json(bad_version.as_bytes()),
        Err(RtdError::SchemaVersion)
    );
    let mut facet = county_facet();
    facet.metric_id = registry_identity("metric", "production/qcew-county-total-wages-usd");
    facet.unit_id = registry_identity("unit", "usd-current");
    facet.value_kind = ValueKindV1::Float64Bits;
    facet.value_bits_or_null = Some("8000000000000000".to_owned());
    let payload = facet_json(&facet);
    let parsed = parse_draft_json(payload.as_bytes()).expect("valid negative-zero draft");
    assert_eq!(
        parsed.facets[0].value_bits_or_null.as_deref(),
        Some("0000000000000000")
    );
}

#[test]
fn structural_refusal_taxonomy_matches_contract_for_numeric_enums_and_missing_versions() {
    let numeric_audience = valid_json().replace("\"ADMIN_MATERIAL\"", "7");
    assert_eq!(
        parse_draft_json(numeric_audience.as_bytes()),
        Err(RtdError::Enum)
    );

    let mut facet = county_facet();
    facet.metric_id = registry_identity("metric", "production/qcew-county-total-wages-usd");
    facet.unit_id = registry_identity("unit", "usd-current");
    facet.value_kind = ValueKindV1::Float64Bits;
    let nested = facet_json(&facet).replace("\"coverage\":\"COMPLETE\"", "\"coverage\":7");
    assert_eq!(parse_draft_json(nested.as_bytes()), Err(RtdError::Enum));

    let missing_schema_version = valid_json().replace("\"schema_version\":1,", "");
    assert_eq!(
        parse_draft_json(missing_schema_version.as_bytes()),
        Err(RtdError::SchemaVersion)
    );
    let missing_projection_version = valid_json().replace("\"projection_version\":1,", "");
    assert_eq!(
        parse_draft_json(missing_projection_version.as_bytes()),
        Err(RtdError::SchemaVersion)
    );
}

#[test]
fn identity_digest_status_and_reference_closure_refuse_exactly() {
    let mut draft = base_draft();
    draft.focus.push(test_identity(""));
    assert_error(RtdError::Identity, &draft);
    let mut draft = base_draft();
    draft.focus.push(test_identity("e\u{301}"));
    assert_error(RtdError::NonNfc, &draft);
    let mut draft = base_draft();
    draft.graph_state_hash = "A".repeat(64);
    assert_error(RtdError::Digest, &draft);
    let mut draft = base_draft();
    let duplicate = test_identity("duplicate");
    draft.focus = vec![duplicate.clone(), duplicate];
    assert_error(RtdError::DuplicateKey, &draft);
    let mut facet = county_facet();
    facet.status = StatusV1::Unknown;
    assert_error(RtdError::StatusValue, &with_county_facet(facet));
    let mut facet = county_facet();
    facet.value_bits_or_null = None;
    assert_error(RtdError::StatusValue, &with_county_facet(facet));
    let mut facet = county_facet();
    facet.metric_id = registry_identity("metric", "production/qcew-county-total-wages-usd");
    facet.unit_id = registry_identity("unit", "usd-current");
    facet.value_kind = ValueKindV1::Float64Bits;
    facet.value_bits_or_null = Some("7ff0000000000000".to_owned());
    assert_error(RtdError::StatusValue, &with_county_facet(facet));
    let mut facet = county_facet();
    facet
        .provenance_refs
        .push(identity("missing", "provenance", "test"));
    assert_error(RtdError::DanglingReference, &with_county_facet(facet));
}

#[test]
fn metric_registry_and_required_digest_semantics_refuse_mutations() {
    let mut draft = with_county_facet(county_facet());
    draft.facets[0].metric_id = identity("unknown", "metric", "test");
    assert_error(RtdError::NativeGrain, &draft);
    let mut draft = with_county_facet(county_facet());
    draft.facets[0].unit_id = registry_identity("unit", "persons");
    assert_error(RtdError::NativeGrain, &draft);
    let mut draft = with_county_facet(county_facet());
    draft.facets[0].coordinates.pop();
    assert_error(RtdError::NativeGrain, &draft);
    let mut draft = with_county_facet(county_facet());
    draft.facets[0].coordinates.push(DimensionCoordinateV1 {
        dimension_ref: registry_identity("dimension", "race"),
        member_ref: identity("all", "race", "test"),
    });
    assert_error(RtdError::NativeGrain, &draft);
    let mut draft = with_county_facet(county_facet());
    draft.facets[0].native_scale = registry_identity("native-scale", "county-source-time-race");
    assert_error(RtdError::NativeGrain, &draft);
    let mut draft = with_county_facet(county_facet());
    draft.facets[0].evidence_class = EvidenceClassV1::Designed;
    assert_error(RtdError::NativeGrain, &draft);
    let mut draft = with_county_facet(county_facet());
    let duplicate = draft.facets[0].coordinates[0].clone();
    draft.facets[0].coordinates.push(duplicate);
    assert_error(RtdError::DuplicateKey, &draft);
    let mut draft = with_county_facet(county_facet());
    draft.reference_digests[0].sha256_hex = ZERO_DIGEST.to_owned();
    assert_error(RtdError::Digest, &draft);
    let mut draft = base_draft();
    draft.facets.push(county_facet());
    assert_error(RtdError::Digest, &draft);
}

#[test]
fn every_relation_binding_and_payload_mutation_is_checked() {
    for kind in [
        DyadKindV1::Presence,
        DyadKindV1::Membership,
        DyadKindV1::Solidarity,
    ] {
        let mut draft = base_draft();
        draft.dyads.push(dyad(kind));
        assert_eq!(validate_draft(&draft), Ok(()));
    }
    let mut command_draft = base_draft();
    command_draft.dyads.push(dyad(DyadKindV1::Command));
    let mut border = commuter_flow(identity("border", "flow", "test"), Vec::new());
    border.flow_kind = FlowKindV1::BorderSynthesis;
    border.destination_ref = identity("canada", "external", "iso3166-1");
    border.native_scale = identity("border", "native-scale", "test");
    command_draft.flows.push(border);
    assert_eq!(validate_draft(&command_draft), Ok(()));
    let mut wrong = dyad(DyadKindV1::Presence);
    wrong.native_scale = identity("wrong", "native-scale", "test");
    let mut draft = base_draft();
    draft.dyads.push(wrong);
    assert_error(RtdError::NativeGrain, &draft);
    let flow_id = identity("flow-1", "flow", "test");
    let facet = lodes_facet(flow_id.clone());
    let mut draft = base_draft();
    draft
        .reference_digests
        .push(reference("fact_lodes_commuter_flow", LODES_DIGEST));
    draft
        .flows
        .push(commuter_flow(flow_id, vec![facet.facet_id.clone()]));
    draft.facets.push(facet);
    assert_eq!(validate_draft(&draft), Ok(()));
    draft.flows[0].payload_facets.clear();
    assert_error(RtdError::NativeGrain, &draft);
    let flow_id = identity("flow-2", "flow", "test");
    let facet = lodes_facet(flow_id.clone());
    let mut duplicate_payload = base_draft();
    duplicate_payload
        .reference_digests
        .push(reference("fact_lodes_commuter_flow", LODES_DIGEST));
    duplicate_payload.flows.push(commuter_flow(
        flow_id,
        vec![facet.facet_id.clone(), facet.facet_id.clone()],
    ));
    duplicate_payload.facets.push(facet);
    assert_error(RtdError::DuplicateKey, &duplicate_payload);
    let mut command = dyad(DyadKindV1::Command);
    command.payload_facets.push(test_identity("payload"));
    let mut dangling_command_draft = base_draft();
    dangling_command_draft.dyads.push(command);
    assert_error(RtdError::DanglingReference, &dangling_command_draft);
    let mut wrong_evidence = dyad(DyadKindV1::Solidarity);
    wrong_evidence.evidence_class = EvidenceClassV1::Observed;
    let mut evidence_draft = base_draft();
    evidence_draft.dyads.push(wrong_evidence);
    assert_error(RtdError::NativeGrain, &evidence_draft);
}

#[test]
fn geography_and_administration_refusals_are_exact() {
    let mut draft = base_draft();
    draft.focus.push(identity("8928308280fffff", "h3", "test"));
    assert_error(RtdError::H3BeforePer21, &draft);
    let mut draft = base_draft();
    draft.focus.push(identity("19820", "msa", "omb"));
    assert_error(RtdError::MsaEvidence, &draft);
    let mut draft = base_draft();
    draft.focus.push(identity("windsor", "county", "canada"));
    assert_error(RtdError::CanadaControl, &draft);
    let mut draft = base_draft();
    draft.audience = AudienceV1::PlayerKnowledge;
    assert_error(RtdError::ForbiddenReduction, &draft);
    let mut draft = base_draft();
    draft.durability = DurabilityV1::Committed;
    assert_error(RtdError::ForbiddenReduction, &draft);
    let mut draft = base_draft();
    draft.fog_policy_digest = Some(ZERO_DIGEST.to_owned());
    assert_error(RtdError::ForbiddenReduction, &draft);
    let mut draft = base_draft();
    draft.actor = Some(identity("player", "organization", "test"));
    assert_error(RtdError::ForbiddenReduction, &draft);
    let mut draft = base_draft();
    draft
        .decision_surface
        .action_refs
        .push(test_identity("action"));
    assert_error(RtdError::ForbiddenReduction, &draft);
    let mut flattened = dyad(DyadKindV1::Command);
    flattened.relation_id = identity("flattened", "hyperedge", "test");
    let mut draft = base_draft();
    draft.dyads.push(flattened);
    assert_error(RtdError::ForbiddenReduction, &draft);
    let mut canada_flow = commuter_flow(
        identity("canada-flow", "flow", "test"),
        vec![test_identity("missing")],
    );
    canada_flow.destination_ref = identity("canada", "external", "iso3166-1");
    let mut draft = base_draft();
    draft.flows.push(canada_flow);
    assert_error(RtdError::CanadaControl, &draft);
    let edge = HyperedgeV1 {
        hyperedge_id: identity("public", "hyperedge", "test"),
        hyperedge_kind: HyperedgeKindV1::PublicRelation,
        member_refs: vec![test_identity("a"), test_identity("b"), test_identity("c")],
        native_scale: identity("public", "native-scale", "test"),
        status: StatusV1::Present,
        coverage: CoverageV1::Complete,
        payload_facets: Vec::new(),
        evidence_class: EvidenceClassV1::Derived,
        provenance_refs: Vec::new(),
    };
    let mut draft = base_draft();
    draft.hyperedges.push(edge);
    assert_eq!(validate_draft(&draft), Ok(()));
}

#[test]
fn direct_validation_refuses_top_level_and_nested_limit_plus_one() {
    let mut draft = base_draft();
    draft.facets = vec![county_facet(); 65_536];
    assert_error(RtdError::LimitExceeded, &draft);
    let mut facet = county_facet();
    facet.coordinates = vec![facet.coordinates[0].clone(); 33];
    assert_error(RtdError::LimitExceeded, &with_county_facet(facet));
}

#[test]
fn membership_and_h3_gap_refusals_are_exact() {
    let membership = ScaleMembershipV1 {
        membership_id: identity("msa", "membership", "test"),
        member_ref: identity("26163", "county", "census"),
        scale_ref: identity("region", "scale", "test"),
        membership_kind: MembershipKindV1::WeightedOverlap,
        status: StatusV1::Present,
        weight_status: StatusV1::Absent,
        weight_bits_or_null: None,
        coverage: CoverageV1::Complete,
        evidence_class: EvidenceClassV1::Observed,
        provenance_refs: Vec::new(),
    };
    let mut draft = base_draft();
    draft.scale_memberships.push(membership.clone());
    assert_error(RtdError::UnsupportedDownscale, &draft);
    draft.scale_memberships[0].membership_kind = MembershipKindV1::Metropolitan;
    assert_error(RtdError::MsaEvidence, &draft);
    let gap = GapV1 {
        gap_id: identity("h3-gap", "gap", "test"),
        requested_metric_or_relation: registry_identity(
            "metric",
            "reproduction/h3-population-persons",
        ),
        status: StatusV1::NotComputed,
        reason_code: GapReasonV1::IdentityContractPending,
        required_producer_or_null: Some("PER-21".to_owned()),
        provenance_refs: Vec::new(),
    };
    let mut draft = base_draft();
    draft.gaps.push(gap.clone());
    assert_eq!(validate_draft(&draft), Ok(()));
    draft.gaps[0].required_producer_or_null = Some("PER-99".to_owned());
    assert_error(RtdError::H3BeforePer21, &draft);
}

#[test]
#[allow(clippy::needless_range_loop)] // Every closed kind uses a fixed audit row.
fn append_uses_every_closed_collection_limit_without_mutating_failure_input() {
    let kinds = [
        (RtdCollectionKindV1::Focus, 64),
        (RtdCollectionKindV1::ReferenceDigests, 4_096),
        (RtdCollectionKindV1::ScaleMemberships, 65_535),
        (RtdCollectionKindV1::Facets, 65_535),
        (RtdCollectionKindV1::Dyads, 65_535),
        (RtdCollectionKindV1::Hyperedges, 65_535),
        (RtdCollectionKindV1::Flows, 65_535),
        (RtdCollectionKindV1::Gaps, 65_535),
        (RtdCollectionKindV1::Provenance, 65_535),
        (RtdCollectionKindV1::Coordinates, 32),
        (RtdCollectionKindV1::MemberRefs, 1_024),
        (RtdCollectionKindV1::PayloadFacets, 256),
        (RtdCollectionKindV1::DisplayRefs, 256),
        (RtdCollectionKindV1::ProvenanceRefs, 8_192),
    ];
    for index in 0..14 {
        let (kind, limit) = kinds[index];
        let original = vec![0_u8; limit];
        let before = original.clone();
        assert_eq!(
            append_bounded(&original, 1, kind),
            Err(RtdError::LimitExceeded)
        );
        assert_eq!(original, before);
    }
}

fn valid_json() -> String {
    format!(
        r#"{{"schema":"{schema}","schema_version":1,"projection_version":1,"audience":"ADMIN_MATERIAL","durability":"IN_MEMORY","verified_tick":7,"graph_state_hash":"{zero}","nominal_world_hash":"{zero}","reference_digests":[],"definitions_digest":"{zero}","template_digest":"{zero}","fog_policy_digest":null,"knowledge_context_digest":null,"actor":null,"focus":[],"scale_memberships":[],"facets":[],"dyads":[],"hyperedges":[],"flows":[],"gaps":[],"provenance":[],"decision_surface":{{"question_id":{{"domain":"question","authority":"test","local_id":"administrative-question"}},"signal_refs":[],"action_refs":[],"receipt_refs":[],"archive_subject_refs":[]}}}}"#,
        schema = RTD_V1_SCHEMA_ID,
        zero = ZERO_DIGEST,
    )
}

fn facet_json(facet: &FacetV1) -> String {
    let facet_json = format!(
        r#"{{"facet_id":{{"domain":"facet","authority":"test","local_id":"facet-1"}},"family":"PRODUCTION_CIRCULATION","subject_ref":{{"domain":"county","authority":"census","local_id":"26163"}},"metric_id":{{"domain":"metric","authority":"babylon.rtd.v1","local_id":"{}"}},"unit_id":{{"domain":"unit","authority":"babylon.rtd.v1","local_id":"{}"}},"native_scale":{{"domain":"native-scale","authority":"babylon.rtd.v1","local_id":"county-ownership-year"}},"coordinates":[{{"dimension_ref":{{"domain":"dimension","authority":"babylon.rtd.v1","local_id":"county"}},"member_ref":{{"domain":"county","authority":"census","local_id":"26163"}}}},{{"dimension_ref":{{"domain":"dimension","authority":"babylon.rtd.v1","local_id":"ownership"}},"member_ref":{{"domain":"ownership","authority":"bls","local_id":"0"}}}}],"vintage":"2023","status":"PRESENT","value_kind":"FLOAT64_BITS","value_bits_or_null":"8000000000000000","coverage":"COMPLETE","evidence_class":"Observed","provenance_refs":[]}}"#,
        facet.metric_id.local_id, facet.unit_id.local_id
    );
    let reference_json = format!(
        r#"{{"reference_id":{{"domain":"reference-artifact","authority":"babylon.data.v7","local_id":"fact_qcew_county_rollup"}},"sha256_hex":"{QCEW_DIGEST}","artifact_schema_id_or_null":null,"vintage":"2023","evidence_class":"Observed"}}"#
    );
    valid_json()
        .replace(
            "\"reference_digests\":[]",
            &format!("\"reference_digests\":[{reference_json}]"),
        )
        .replace("\"facets\":[]", &format!("\"facets\":[{facet_json}]"))
}
