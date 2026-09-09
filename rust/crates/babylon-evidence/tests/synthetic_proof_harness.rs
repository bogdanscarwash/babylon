use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;

use babylon_bsl::{
    canonical_bytes, read, validate_sfs_rule_profile, CardinalityCeilings, ClosedVocabulary,
    EnumKind, GovernedComparisonSite, IntrinsicCosts, SExpr, SfsAuditPolicy, SfsComparisonContext,
    SfsRuleAuditResult,
};
use babylon_evidence::{
    bind_synthetic_driver, canonical_envelope, component_profile_from_bsl, decode_envelope,
    parse_synthetic_driver_contract, parse_synthetic_governed_manifest, record_digest,
    validate_synthetic_cone, validate_synthetic_mutation_manifest,
    validate_synthetic_profile_identity, CanonicalProfileSet, CausalConeV1, ComponentKindV1,
    DifferingLedgerKindV1, Digest32, InterventionDeltaRowV1, InterventionDeltaV1,
    InterventionOperationV1, PersistenceComparisonV1, PracticeAttemptLedgerV1,
    PracticeAttemptRowV1, PracticeCandidateRowV1, PracticeCandidateScheduleV1,
    PracticeDispositionV1, RunIdentityField, RunIdentityV1, SfsComponentProofProfileV1,
    SfsPreregistrationV1, SfsProofProfileV1, SfsSampleV1, SfsTraceV1, SfsValidationError,
    SyntheticDriverError, T3Record,
};
use babylon_kernel::{sha256_of, SessionId};
use babylon_practice_contract::{
    intent_digest, parameter_bytes_digest, target_selection_policy_digest, PracticeIdV1,
    PracticeIntentV1, PracticeParameterV1, PracticeTargetDomainV1,
};

const GOVERNED: &[u8] = include_bytes!("fixtures/sfs_synthetic_governed_manifest_v1.txt");
const PROFILE: &str = include_str!("fixtures/sfs_synthetic_profile_v1.txt");
const DRIVER_CONTRACT: &[u8] = include_bytes!("fixtures/sfs_synthetic_driver_contract_v1.txt");
const MUTATIONS: &[u8] = include_bytes!("fixtures/sfs_mutation_manifest_v1.txt");
const FORBIDDEN: &[u8] =
    include_bytes!("../../babylon-bsl/tests/fixtures/sfs_profile/sfs_forbidden_manifest_v1.txt");
const AUDIT_SOURCES: &[u8] =
    include_bytes!("../../babylon-bsl/tests/fixtures/sfs_profile/sfs_audit_source_manifest_v1.txt");
const WIRE_VECTORS: &str = include_str!("fixtures/sfs_wire_vectors_v1.txt");
const IDENTITY_MUTATIONS: &str = include_str!("fixtures/sfs_identity_mutations_v1.txt");
const SYNTHETIC_EMPTY_EXOGENOUS_DIGEST: Digest32 = Digest32::from_bytes([0xE0; 32]);
const MEMBERSHIP_DESCRIPTOR: &[u8] =
    b"membership-reducer maps one synthetic field value to one reducer output";
const PRODUCER_DESCRIPTOR: &[u8] =
    b"post-commit-producer emits one synthetic sample after a sealed envelope";
const AUDIT_SEMANTICS_ID: &str = "babylon.sfs.audit.v1";

fn digest(tag: u8) -> Digest32 {
    let mut bytes = [0_u8; 32];
    bytes[0] = tag;
    bytes[31] = !tag;
    Digest32::from_bytes(bytes)
}

fn hex_bytes(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).unwrap())
        .collect()
}

fn lower_hex(value: &[u8]) -> String {
    let mut output = String::with_capacity(value.len() * 2);
    for byte in value {
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}

fn sorted_manifest(mut rows: Vec<String>) -> Vec<u8> {
    rows.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    format!("{}\n", rows.join("\n")).into_bytes()
}

fn base36_4(mut value: usize) -> String {
    const ALPHABET: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut output = [b'0'; 4];
    for offset in 0..4 {
        let index = 3 - offset;
        output[index] = ALPHABET[value % 36];
        value /= 36;
    }
    String::from_utf8(output.to_vec()).unwrap()
}

fn rule() -> SExpr {
    let source =
        include_str!("../../babylon-bsl/tests/fixtures/sfs_profile/allowed/scoped_mechanic.bsl");
    read(source).unwrap().0
}

fn audit() -> SfsRuleAuditResult {
    let rule = rule();
    let vocabulary = ClosedVocabulary::new([
        (
            EnumKind::NodeType,
            vec!["SYNTHETIC_SOURCE".to_owned(), "ORGANIZATION".to_owned()],
        ),
        (EnumKind::EdgeType, vec!["SYNTHETIC_LINK".to_owned()]),
    ])
    .unwrap();
    let ceilings = CardinalityCeilings::new(
        HashMap::from([("EdgeType/SYNTHETIC_LINK".to_owned(), 8)]),
        HashMap::new(),
    );
    let sites = vec![
        GovernedComparisonSite::from_rule_path(
            &rule,
            &[0, 11, 1, 1],
            SfsComparisonContext::ConservationRefusal,
        )
        .unwrap(),
        GovernedComparisonSite::from_rule_path(
            &rule,
            &[0, 11, 1, 2],
            SfsComparisonContext::EligibilityNoEffect,
        )
        .unwrap(),
    ];
    let policy = SfsAuditPolicy::new(
        "synthetic-source/scoped-mechanic",
        sha256_of(&canonical_bytes(&rule).unwrap()),
        31,
        ["synthetic-source/quanta"],
        ["synthetic-link/strength"],
        [
            "synthetic/minimum-link-strength",
            "synthetic/transfer-quantum",
        ],
        ["edges"],
        [">"],
        [],
        sites,
        ["node:synthetic-source/quanta"],
    )
    .unwrap();
    validate_sfs_rule_profile(
        &rule,
        &vocabulary,
        &ceilings,
        &IntrinsicCosts::default(),
        &policy,
    )
    .unwrap()
}

fn proof_profile() -> SfsProofProfileV1 {
    let governed = parse_synthetic_governed_manifest(GOVERNED, &rule(), &audit()).unwrap();
    proof_with(governed.manifest_digest(), &cone(), source_profiles())
}

fn domain_digest(domain: &[u8], payload: &[u8]) -> Digest32 {
    Digest32::from_bytes(sha256_of(&[domain, b"\0", payload].concat()))
}

fn proof_with(
    governed_manifest_digest: Digest32,
    selected_cone: &CausalConeV1,
    components: Vec<SfsComponentProofProfileV1>,
) -> SfsProofProfileV1 {
    SfsProofProfileV1::new(
        governed_manifest_digest,
        domain_digest(b"babylon.sfs-forbidden-corpus-manifest.v1", FORBIDDEN),
        AUDIT_SEMANTICS_ID,
        domain_digest(b"babylon.sfs-audit-source-manifest.v1", AUDIT_SOURCES),
        Digest32::from_bytes(*record_digest(selected_cone).unwrap().as_bytes()),
        components,
    )
    .unwrap()
}

fn profile_set(values: &[&str]) -> CanonicalProfileSet {
    let mut entries = Vec::with_capacity(values.len());
    for index in 0..64 {
        if index >= values.len() {
            break;
        }
        entries.push(values[index].to_owned());
    }
    CanonicalProfileSet::new("synthetic-test", entries).unwrap()
}

fn host_profile(
    component_id: &str,
    kind: ComponentKindV1,
    descriptor: &[u8],
    field_reads: &[&str],
    effects: &[&str],
) -> SfsComponentProofProfileV1 {
    SfsComponentProofProfileV1::new(
        component_id,
        kind,
        domain_digest(b"babylon.sfs-synthetic-component-source.v1", descriptor),
        profile_set(field_reads),
        profile_set(&[]),
        profile_set(&[]),
        profile_set(&[]),
        profile_set(&[]),
        profile_set(&[]),
        profile_set(&[]),
        profile_set(effects),
    )
    .unwrap()
}

fn membership_profile(field_read: &str) -> SfsComponentProofProfileV1 {
    host_profile(
        "membership-reducer",
        ComponentKindV1::Reducer,
        MEMBERSHIP_DESCRIPTOR,
        &[field_read],
        &["reducer-output:synthetic/membership-reducer-output"],
    )
}

fn producer_profile() -> SfsComponentProofProfileV1 {
    producer_profile_with(
        "reducer-output:synthetic/membership-reducer-output",
        "receipt:synthetic/sfs-sample",
    )
}

fn producer_profile_with(field_read: &str, effect: &str) -> SfsComponentProofProfileV1 {
    host_profile(
        "post-commit-producer",
        ComponentKindV1::PostCommitProducer,
        PRODUCER_DESCRIPTOR,
        &[field_read],
        &[effect],
    )
}

fn source_profiles() -> Vec<SfsComponentProofProfileV1> {
    vec![
        membership_profile("synthetic-source/quanta"),
        producer_profile(),
        component_profile_from_bsl("scoped-bsl-rule", &audit()).unwrap(),
    ]
}

fn profile_set_from_audit(values: &BTreeSet<String>) -> CanonicalProfileSet {
    CanonicalProfileSet::new("synthetic-test", values.iter().cloned().collect()).unwrap()
}

fn changed_scoped_profile(operator: &str) -> SfsComponentProofProfileV1 {
    let sealed = audit();
    let footprint = sealed.footprint();
    SfsComponentProofProfileV1::new(
        "scoped-bsl-rule",
        ComponentKindV1::BslRule,
        Digest32::from_bytes(*footprint.source_digest()),
        profile_set_from_audit(footprint.field_reads()),
        profile_set_from_audit(footprint.edge_reads()),
        profile_set_from_audit(footprint.constant_reads()),
        profile_set_from_audit(footprint.queries()),
        profile_set(&[operator]),
        profile_set_from_audit(footprint.intrinsics()),
        profile_set_from_audit(footprint.comparison_clamp_contexts()),
        profile_set_from_audit(footprint.effects()),
    )
    .unwrap()
}

fn cone() -> CausalConeV1 {
    CausalConeV1::new(
        vec!["scoped-bsl-rule".to_owned()],
        vec!["post-commit-producer".to_owned()],
        vec![
            "scoped-bsl-rule".to_owned(),
            "membership-reducer".to_owned(),
            "post-commit-producer".to_owned(),
        ],
    )
    .unwrap()
}

fn assert_profile_fixture<T: T3Record>(label: &str, name: &str, actual: &T) {
    let prefix = format!("{label}|{name}|");
    let row = PROFILE
        .lines()
        .find(|candidate| candidate.starts_with(&prefix))
        .unwrap();
    let fields = row.split('|').collect::<Vec<_>>();
    assert_eq!(fields.len(), 5);
    assert_eq!(fields[2].as_bytes(), T::DOMAIN);
    assert_eq!(hex_bytes(fields[3]), canonical_envelope(actual).unwrap());
    assert_eq!(fields[4], record_digest(actual).unwrap().to_hex());
}

#[test]
fn five_profile_fixtures_derive_from_independent_source_contracts() {
    let governed = parse_synthetic_governed_manifest(GOVERNED, &rule(), &audit()).unwrap();
    let selected_cone = cone();
    let components = source_profiles();
    let proof = proof_with(
        governed.manifest_digest(),
        &selected_cone,
        components.clone(),
    );
    assert_profile_fixture("component", "membership-reducer", &components[0]);
    assert_profile_fixture("component", "post-commit-producer", &components[1]);
    assert_profile_fixture("component", "scoped-bsl-rule", &components[2]);
    assert_profile_fixture("cone", "synthetic-chain", &selected_cone);
    assert_profile_fixture("proof-profile", "synthetic-chain", &proof);
}

#[test]
fn proof_header_pins_forbidden_audit_source_and_semantics_independently() {
    let forbidden = domain_digest(b"babylon.sfs-forbidden-corpus-manifest.v1", FORBIDDEN);
    let audit_source = domain_digest(b"babylon.sfs-audit-source-manifest.v1", AUDIT_SOURCES);
    assert_eq!(
        forbidden.to_hex(),
        "e3e7d0c90b7302c441005a4cb482a1aff86c2e9178b06a514b2f9c6304aeca74"
    );
    assert_eq!(
        audit_source.to_hex(),
        "91cfca31b605e3297db7e440db4007b0d15f228ce24461afdde6cb3859ce8487"
    );
    let envelope = canonical_envelope(&proof_profile()).unwrap();
    let payload = SfsProofProfileV1::DOMAIN.len() + 7;
    assert_eq!(&envelope[payload + 32..payload + 64], forbidden.as_bytes());
    assert_eq!(
        u16::from_be_bytes([envelope[payload + 64], envelope[payload + 65]]),
        u16::try_from(AUDIT_SEMANTICS_ID.len()).unwrap()
    );
    assert_eq!(
        &envelope[payload + 66..payload + 66 + AUDIT_SEMANTICS_ID.len()],
        AUDIT_SEMANTICS_ID.as_bytes()
    );
    let audit_start = payload + 66 + AUDIT_SEMANTICS_ID.len();
    assert_eq!(
        &envelope[audit_start..audit_start + 32],
        audit_source.as_bytes()
    );
}

#[test]
fn exact_three_component_cone_is_required() {
    let governed = parse_synthetic_governed_manifest(GOVERNED, &rule(), &audit()).unwrap();
    let profile = proof_profile();
    assert_eq!(
        validate_synthetic_cone(&cone(), &profile, &governed),
        Ok(())
    );
    let missing_middle = CausalConeV1::new(
        vec!["scoped-bsl-rule".to_owned()],
        vec!["post-commit-producer".to_owned()],
        vec![
            "scoped-bsl-rule".to_owned(),
            "post-commit-producer".to_owned(),
        ],
    )
    .unwrap();
    let missing_profile = proof_with(
        governed.manifest_digest(),
        &missing_middle,
        profile.components().to_vec(),
    );
    assert_eq!(
        validate_synthetic_cone(&missing_middle, &missing_profile, &governed),
        Err(SfsValidationError::GovernedComponentSetMismatch)
    );

    let extra = CausalConeV1::new(
        vec!["scoped-bsl-rule".to_owned()],
        vec!["post-commit-producer".to_owned()],
        [
            cone().components().to_vec(),
            vec!["unprofiled-extra".to_owned()],
        ]
        .concat(),
    )
    .unwrap();
    let extra_profile = proof_with(
        governed.manifest_digest(),
        &extra,
        profile.components().to_vec(),
    );
    assert_eq!(
        validate_synthetic_cone(&extra, &extra_profile, &governed),
        Err(SfsValidationError::GovernedComponentSetMismatch)
    );
}

#[test]
fn reachability_profile_and_path_boundaries_are_distinct() {
    let governed = parse_synthetic_governed_manifest(GOVERNED, &rule(), &audit()).unwrap();
    let original = proof_profile();
    let unreachable = CausalConeV1::new(
        vec!["scoped-bsl-rule".to_owned()],
        vec!["membership-reducer".to_owned()],
        cone().components().to_vec(),
    )
    .unwrap();
    let unreachable_profile = proof_with(
        governed.manifest_digest(),
        &unreachable,
        original.components().to_vec(),
    );
    assert_eq!(
        validate_synthetic_cone(&unreachable, &unreachable_profile, &governed),
        Err(SfsValidationError::GovernedComponentSetMismatch)
    );

    let without_middle = vec![
        component_profile_from_bsl("scoped-bsl-rule", &audit()).unwrap(),
        producer_profile(),
    ];
    let unprofiled = proof_with(governed.manifest_digest(), &cone(), without_middle);
    assert_eq!(
        validate_synthetic_cone(&cone(), &unprofiled, &governed),
        Err(SfsValidationError::ConeProfileMismatch)
    );

    let reversed = CausalConeV1::new(
        vec!["post-commit-producer".to_owned()],
        vec!["scoped-bsl-rule".to_owned()],
        cone().components().to_vec(),
    )
    .unwrap();
    let reversed_profile = proof_with(
        governed.manifest_digest(),
        &reversed,
        original.components().to_vec(),
    );
    assert_eq!(
        validate_synthetic_cone(&reversed, &reversed_profile, &governed),
        Err(SfsValidationError::NoRootToSinkPath)
    );
}

#[test]
fn recomputed_host_profile_identity_cannot_hide_changed_profile_bytes() {
    let governed = parse_synthetic_governed_manifest(GOVERNED, &rule(), &audit()).unwrap();
    let changed = proof_with(
        governed.manifest_digest(),
        &cone(),
        vec![
            membership_profile("synthetic-source/quantb"),
            producer_profile(),
            component_profile_from_bsl("scoped-bsl-rule", &audit()).unwrap(),
        ],
    );
    assert_ne!(
        record_digest(&changed).unwrap(),
        record_digest(&proof_profile()).unwrap()
    );
    assert_eq!(
        validate_synthetic_cone(&cone(), &changed, &governed),
        Err(SfsValidationError::ConeProfileMismatch)
    );
}

fn assert_changed_host_manifest_profile_refuses(
    changed_manifest_bytes: &[u8],
    changed_components: Vec<SfsComponentProofProfileV1>,
) {
    let governed =
        parse_synthetic_governed_manifest(changed_manifest_bytes, &rule(), &audit()).unwrap();
    let changed = proof_with(governed.manifest_digest(), &cone(), changed_components);
    assert_eq!(
        validate_synthetic_cone(&cone(), &changed, &governed),
        Err(SfsValidationError::ConeProfileMismatch)
    );
}

#[test]
fn both_host_manifest_profiles_are_source_bound_to_exact_descriptor_contracts() {
    let membership = String::from_utf8(GOVERNED.to_vec())
        .unwrap()
        .replace(
            "profile|6d656d626572736869702d72656475636572|field_reads|73796e7468657469632d736f757263652f7175616e7461",
            "profile|6d656d626572736869702d72656475636572|field_reads|73796e7468657469632d736f757263652f7175616e7462",
        );
    assert_changed_host_manifest_profile_refuses(
        membership.as_bytes(),
        vec![
            membership_profile("synthetic-source/quantb"),
            producer_profile(),
            component_profile_from_bsl("scoped-bsl-rule", &audit()).unwrap(),
        ],
    );
    let producer = String::from_utf8(GOVERNED.to_vec())
        .unwrap()
        .replace(
            "profile|706f73742d636f6d6d69742d70726f6475636572|effects|726563656970743a73796e7468657469632f7366732d73616d706c65",
            "profile|706f73742d636f6d6d69742d70726f6475636572|effects|726563656970743a73796e7468657469632f7366732d73616d706c66",
        );
    assert_changed_host_manifest_profile_refuses(
        producer.as_bytes(),
        vec![
            membership_profile("synthetic-source/quanta"),
            producer_profile_with(
                "reducer-output:synthetic/membership-reducer-output",
                "receipt:synthetic/sfs-samplf",
            ),
            component_profile_from_bsl("scoped-bsl-rule", &audit()).unwrap(),
        ],
    );
}

#[test]
fn typed_edges_are_identity_bound_beyond_reachability() {
    let original_manifest = parse_synthetic_governed_manifest(GOVERNED, &rule(), &audit()).unwrap();
    let original_profile = proof_profile();
    let original_cone_bytes = canonical_envelope(&cone()).unwrap();
    let changed = String::from_utf8(GOVERNED.to_vec())
        .unwrap()
        .replace("|5|73796e7468657469632f", "|4|73796e7468657469632f");
    let changed_manifest =
        parse_synthetic_governed_manifest(changed.as_bytes(), &rule(), &audit()).unwrap();
    assert_eq!(canonical_envelope(&cone()).unwrap(), original_cone_bytes);
    assert_ne!(
        changed_manifest.manifest_digest(),
        original_manifest.manifest_digest()
    );
    assert_eq!(
        validate_synthetic_cone(&cone(), &original_profile, &changed_manifest),
        Err(SfsValidationError::GovernedManifestDigestMismatch)
    );

    let rebound_profile = proof_with(
        changed_manifest.manifest_digest(),
        &cone(),
        original_profile.components().to_vec(),
    );
    assert_ne!(
        record_digest(&rebound_profile).unwrap(),
        record_digest(&original_profile).unwrap()
    );
    assert!(matches!(
        validate_synthetic_cone(&cone(), &rebound_profile, &changed_manifest),
        Err(SfsValidationError::EdgeProducerEffectMismatch { .. })
    ));
}

#[test]
fn recomputed_bsl_source_cannot_replace_the_sealed_audit_source() {
    let source =
        include_str!("../../babylon-bsl/tests/fixtures/sfs_profile/allowed/scoped_mechanic.bsl");
    let changed_rule = read(&source.replace("synthetic-source/quanta", "synthetic-source/quantb"))
        .unwrap()
        .0;
    let changed_payload = canonical_bytes(&changed_rule).unwrap();
    let mut rows = String::from_utf8(GOVERNED.to_vec())
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let bsl_index = rows
        .iter()
        .position(|row| row.starts_with("component|73636f7065642d62736c2d72756c65|"))
        .unwrap();
    let fields = rows[bsl_index].split('|').collect::<Vec<_>>();
    rows[bsl_index] = format!(
        "{}|{}|{}|{}|{}|{}",
        fields[0],
        fields[1],
        fields[2],
        fields[3],
        lower_hex(&changed_payload),
        lower_hex(&sha256_of(&changed_payload)),
    );
    assert_eq!(
        parse_synthetic_governed_manifest(&sorted_manifest(rows), &changed_rule, &audit(),),
        Err(SfsValidationError::ComponentSourceDigestMismatch {
            component_id: "scoped-bsl-rule".to_owned(),
        })
    );
}

#[test]
fn recomputed_outer_identities_cannot_hide_changed_bsl_profile_rows() {
    let original_manifest = parse_synthetic_governed_manifest(GOVERNED, &rule(), &audit()).unwrap();
    let original_profile = proof_profile();
    let changed_bytes = String::from_utf8(GOVERNED.to_vec()).unwrap().replace(
        "profile|73636f7065642d62736c2d72756c65|operators|3e",
        "profile|73636f7065642d62736c2d72756c65|operators|3d",
    );
    let changed_manifest =
        parse_synthetic_governed_manifest(changed_bytes.as_bytes(), &rule(), &audit()).unwrap();
    let changed_profile = proof_with(
        changed_manifest.manifest_digest(),
        &cone(),
        vec![
            membership_profile("synthetic-source/quanta"),
            producer_profile(),
            changed_scoped_profile("="),
        ],
    );
    let (_, schedule, attempts) = candidate_bundle();
    let changed_prereg = preregistration(&schedule, &changed_profile, digest(99));
    let changed_run = run_identity(
        changed_manifest.host_component_manifest_digest(),
        changed_manifest.manifest_digest(),
        Digest32::from_bytes(*record_digest(&changed_profile).unwrap().as_bytes()),
        Digest32::from_bytes(*record_digest(&changed_prereg).unwrap().as_bytes()),
        Digest32::from_bytes(*record_digest(&attempts).unwrap().as_bytes()),
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
    );
    assert_ne!(
        original_manifest.manifest_digest(),
        changed_manifest.manifest_digest()
    );
    assert_ne!(
        record_digest(&original_profile).unwrap(),
        record_digest(&changed_profile).unwrap()
    );
    assert_eq!(
        validate_synthetic_profile_identity(
            &changed_run,
            &changed_profile,
            &changed_prereg,
            &changed_manifest,
            changed_prereg.mutation_manifest_digest(),
        ),
        Ok(())
    );
    assert_eq!(
        validate_synthetic_cone(&cone(), &changed_profile, &changed_manifest),
        Err(SfsValidationError::ConeProfileMismatch)
    );
}

#[test]
fn duplicate_component_and_typed_edge_identities_refuse_specifically() {
    let rows = String::from_utf8(GOVERNED.to_vec())
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let membership = rows
        .iter()
        .find(|row| row.starts_with("component|6d656d626572736869702d72656475636572|"))
        .unwrap()
        .replacen("|2|synthetic-descriptor|", "|1|synthetic-descriptor|", 1);
    let mut duplicate_component = rows.clone();
    duplicate_component.push(membership);
    assert_eq!(
        parse_synthetic_governed_manifest(&sorted_manifest(duplicate_component), &rule(), &audit(),),
        Err(SfsValidationError::DuplicateComponentId {
            component_id: "membership-reducer".to_owned(),
        })
    );

    let reducer_edge = rows
        .iter()
        .find(|row| row.starts_with("edge|6d656d626572736869702d72656475636572|"))
        .unwrap()
        .replacen("|5|", "|4|", 1);
    let mut duplicate_edge = rows;
    duplicate_edge.push(reducer_edge);
    assert_eq!(
        parse_synthetic_governed_manifest(&sorted_manifest(duplicate_edge), &rule(), &audit()),
        Err(SfsValidationError::DuplicateTypedEdge {
            producer_id: "membership-reducer".to_owned(),
            consumer_id: "post-commit-producer".to_owned(),
            channel_id: "synthetic/membership-reducer-output".to_owned(),
        })
    );
}

fn intent(tick: u64) -> PracticeIntentV1 {
    PracticeIntentV1 {
        schema_version: 1,
        submit_after_tick: tick - 1,
        resolve_tick: tick,
        actor_org_id: 7,
        practice_id: PracticeIdV1::Organize,
        target_domain: PracticeTargetDomainV1::SocialClass,
        target_node_id: 99,
        quoted_content_digest: [7; 32],
        quoted_action_budget_cost: 3,
        parameters: vec![],
        evidence_digests: vec![],
    }
}

fn candidate_bundle() -> (
    Vec<PracticeIntentV1>,
    PracticeCandidateScheduleV1,
    PracticeAttemptLedgerV1,
) {
    let intents = vec![intent(100), intent(102), intent(104)];
    let rows = intents
        .iter()
        .enumerate()
        .map(|(index, value)| {
            PracticeCandidateRowV1::new(
                value.resolve_tick,
                digest(20 + u8::try_from(index).unwrap()),
                Digest32::from_bytes(intent_digest(value).unwrap()),
            )
        })
        .collect::<Vec<_>>();
    let schedule = PracticeCandidateScheduleV1::new(rows.clone()).unwrap();
    let attempts = rows
        .into_iter()
        .enumerate()
        .map(|(index, row)| {
            PracticeAttemptRowV1::new(
                row,
                PracticeDispositionV1::Rejected,
                digest(40 + u8::try_from(index).unwrap()),
            )
            .unwrap()
        })
        .collect();
    let ledger = PracticeAttemptLedgerV1::new(digest(50), attempts).unwrap();
    (intents, schedule, ledger)
}

fn bundle_from_intents(
    intents: Vec<PracticeIntentV1>,
) -> (
    Vec<PracticeIntentV1>,
    PracticeCandidateScheduleV1,
    PracticeAttemptLedgerV1,
) {
    let rows = intents
        .iter()
        .enumerate()
        .map(|(index, value)| {
            PracticeCandidateRowV1::new(
                value.resolve_tick,
                digest(150 + u8::try_from(index).unwrap()),
                Digest32::from_bytes(intent_digest(value).unwrap()),
            )
        })
        .collect::<Vec<_>>();
    let schedule = PracticeCandidateScheduleV1::new(rows.clone()).unwrap();
    let ledger = ledger_from_rows(rows);
    (intents, schedule, ledger)
}

fn ledger_from_rows(rows: Vec<PracticeCandidateRowV1>) -> PracticeAttemptLedgerV1 {
    let attempts = rows
        .into_iter()
        .enumerate()
        .map(|(index, row)| {
            PracticeAttemptRowV1::new(
                row,
                PracticeDispositionV1::Rejected,
                digest(160 + u8::try_from(index).unwrap()),
            )
            .unwrap()
        })
        .collect();
    PracticeAttemptLedgerV1::new(digest(170), attempts).unwrap()
}

#[allow(clippy::too_many_arguments)]
fn preregistration_custom(
    schedule: &PracticeCandidateScheduleV1,
    driver_digest: Digest32,
    first: u64,
    stride: u16,
    count: u16,
    practice: PracticeIdV1,
    target: u64,
    cost: u32,
    parameter_digest: Digest32,
) -> SfsPreregistrationV1 {
    SfsPreregistrationV1::new(
        90,
        digest(2),
        Digest32::from_bytes(*record_digest(schedule).unwrap().as_bytes()),
        Digest32::from_bytes(*record_digest(&proof_profile()).unwrap().as_bytes()),
        driver_digest,
        digest(3),
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        first,
        stride,
        count,
        practice,
        Digest32::from_bytes(target_selection_policy_digest(
            PracticeTargetDomainV1::SocialClass,
            target,
        )),
        cost,
        parameter_digest,
    )
    .unwrap()
}

fn run_for_candidate(
    preregistration: &SfsPreregistrationV1,
    attempts: &PracticeAttemptLedgerV1,
    exogenous: Digest32,
) -> RunIdentityV1 {
    run_identity(
        digest(1),
        digest(2),
        digest(3),
        Digest32::from_bytes(*record_digest(preregistration).unwrap().as_bytes()),
        Digest32::from_bytes(*record_digest(attempts).unwrap().as_bytes()),
        exogenous,
    )
}

fn preregistration(
    schedule: &PracticeCandidateScheduleV1,
    proof: &SfsProofProfileV1,
    driver_digest: Digest32,
) -> SfsPreregistrationV1 {
    SfsPreregistrationV1::new(
        90,
        digest(2),
        Digest32::from_bytes(*record_digest(schedule).unwrap().as_bytes()),
        Digest32::from_bytes(*record_digest(proof).unwrap().as_bytes()),
        driver_digest,
        Digest32::from_bytes(sha256_of(
            &[b"babylon.sfs-mutation-manifest.v1\0".as_slice(), MUTATIONS].concat(),
        )),
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        100,
        2,
        3,
        PracticeIdV1::Organize,
        Digest32::from_bytes(target_selection_policy_digest(
            PracticeTargetDomainV1::SocialClass,
            99,
        )),
        3,
        Digest32::from_bytes(parameter_bytes_digest(&intent(100)).unwrap()),
    )
    .unwrap()
}

fn identity_preregistration(
    schedule: &PracticeCandidateScheduleV1,
    proof_digest: Digest32,
    driver_digest: Digest32,
    mutation_digest: Digest32,
) -> SfsPreregistrationV1 {
    SfsPreregistrationV1::new(
        90,
        digest(2),
        Digest32::from_bytes(*record_digest(schedule).unwrap().as_bytes()),
        proof_digest,
        driver_digest,
        mutation_digest,
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        100,
        2,
        3,
        PracticeIdV1::Organize,
        Digest32::from_bytes(target_selection_policy_digest(
            PracticeTargetDomainV1::SocialClass,
            99,
        )),
        3,
        Digest32::from_bytes(parameter_bytes_digest(&intent(100)).unwrap()),
    )
    .unwrap()
}

fn run_identity(
    host: Digest32,
    governed: Digest32,
    proof: Digest32,
    preregistration: Digest32,
    attempts: Digest32,
    exogenous: Digest32,
) -> RunIdentityV1 {
    RunIdentityV1::new(
        SessionId::new("synthetic-run").unwrap(),
        digest(60),
        digest(61),
        digest(62),
        digest(63),
        host,
        digest(64),
        digest(65),
        digest(66),
        governed,
        proof,
        preregistration,
        digest(67),
        digest(68),
        exogenous,
        attempts,
        "rng-v1",
        "graph-v1",
    )
    .unwrap()
}

#[test]
fn candidate_projection_cadence_and_intent_realization_are_exact() {
    let (intents, schedule, attempts) = candidate_bundle();
    assert_eq!(
        canonical_envelope(&attempts.project_candidates().unwrap()).unwrap(),
        canonical_envelope(&schedule).unwrap()
    );
    let contract = parse_synthetic_driver_contract(DRIVER_CONTRACT).unwrap();
    let proof = proof_profile();
    let governed = parse_synthetic_governed_manifest(GOVERNED, &rule(), &audit()).unwrap();
    let prereg = preregistration(&schedule, &proof, contract.manifest_digest());
    let run = run_identity(
        governed.host_component_manifest_digest(),
        governed.manifest_digest(),
        Digest32::from_bytes(*record_digest(&proof).unwrap().as_bytes()),
        Digest32::from_bytes(*record_digest(&prereg).unwrap().as_bytes()),
        Digest32::from_bytes(*record_digest(&attempts).unwrap().as_bytes()),
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
    );
    validate_synthetic_profile_identity(
        &run,
        &proof,
        &prereg,
        &governed,
        prereg.mutation_manifest_digest(),
    )
    .unwrap();
    let driver = bind_synthetic_driver(&prereg, &contract).unwrap();
    assert_eq!(
        driver.validate_candidate_projection(
            &run,
            &prereg,
            &schedule,
            &attempts,
            &intents,
            SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        ),
        Ok(())
    );
}

#[test]
fn synthetic_profile_identity_fields_refuse_independently() {
    let governed = parse_synthetic_governed_manifest(GOVERNED, &rule(), &audit()).unwrap();
    assert_ne!(
        governed.host_component_manifest_digest(),
        governed.manifest_digest()
    );
    let proof = proof_profile();
    let proof_digest = Digest32::from_bytes(*record_digest(&proof).unwrap().as_bytes());
    let mutation_digest = domain_digest(b"babylon.sfs-mutation-manifest.v1", MUTATIONS);
    let contract = parse_synthetic_driver_contract(DRIVER_CONTRACT).unwrap();
    let (_, schedule, attempts) = candidate_bundle();
    let prereg = identity_preregistration(
        &schedule,
        proof_digest,
        contract.manifest_digest(),
        mutation_digest,
    );
    let prereg_digest = Digest32::from_bytes(*record_digest(&prereg).unwrap().as_bytes());
    let valid = run_identity(
        governed.host_component_manifest_digest(),
        governed.manifest_digest(),
        proof_digest,
        prereg_digest,
        Digest32::from_bytes(*record_digest(&attempts).unwrap().as_bytes()),
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
    );
    assert_eq!(
        validate_synthetic_profile_identity(&valid, &proof, &prereg, &governed, mutation_digest,),
        Ok(())
    );

    let cases = [
        (
            run_identity(
                digest(210),
                governed.manifest_digest(),
                proof_digest,
                prereg_digest,
                valid.practice_attempt_ledger_digest(),
                SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
            ),
            SfsValidationError::HostManifestDigestMismatch,
        ),
        (
            run_identity(
                governed.host_component_manifest_digest(),
                digest(211),
                proof_digest,
                prereg_digest,
                valid.practice_attempt_ledger_digest(),
                SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
            ),
            SfsValidationError::GovernedFootprintDigestMismatch,
        ),
        (
            run_identity(
                governed.host_component_manifest_digest(),
                governed.manifest_digest(),
                digest(212),
                prereg_digest,
                valid.practice_attempt_ledger_digest(),
                SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
            ),
            SfsValidationError::ProofProfileDigestMismatch,
        ),
        (
            run_identity(
                governed.host_component_manifest_digest(),
                governed.manifest_digest(),
                proof_digest,
                digest(213),
                valid.practice_attempt_ledger_digest(),
                SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
            ),
            SfsValidationError::PreregistrationDigestMismatch,
        ),
    ];
    for (changed_run, expected) in cases {
        assert_eq!(
            validate_synthetic_profile_identity(
                &changed_run,
                &proof,
                &prereg,
                &governed,
                mutation_digest,
            ),
            Err(expected)
        );
    }
}

#[test]
fn proof_header_preregistration_and_mutation_identities_are_closed() {
    let governed = parse_synthetic_governed_manifest(GOVERNED, &rule(), &audit()).unwrap();
    let proof = proof_profile();
    let proof_digest = Digest32::from_bytes(*record_digest(&proof).unwrap().as_bytes());
    let mutation_digest = domain_digest(b"babylon.sfs-mutation-manifest.v1", MUTATIONS);
    let contract = parse_synthetic_driver_contract(DRIVER_CONTRACT).unwrap();
    let (_, schedule, attempts) = candidate_bundle();
    let changed_proof = proof_with(digest(214), &cone(), proof.components().to_vec());
    let changed_proof_digest =
        Digest32::from_bytes(*record_digest(&changed_proof).unwrap().as_bytes());
    let changed_prereg = identity_preregistration(
        &schedule,
        changed_proof_digest,
        contract.manifest_digest(),
        mutation_digest,
    );
    let changed_run = run_identity(
        governed.host_component_manifest_digest(),
        governed.manifest_digest(),
        changed_proof_digest,
        Digest32::from_bytes(*record_digest(&changed_prereg).unwrap().as_bytes()),
        Digest32::from_bytes(*record_digest(&attempts).unwrap().as_bytes()),
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
    );
    assert_eq!(
        validate_synthetic_profile_identity(
            &changed_run,
            &changed_proof,
            &changed_prereg,
            &governed,
            mutation_digest,
        ),
        Err(SfsValidationError::GovernedManifestDigestMismatch)
    );

    let bad_prereg = identity_preregistration(
        &schedule,
        digest(215),
        contract.manifest_digest(),
        mutation_digest,
    );
    let bad_prereg_run = run_identity(
        governed.host_component_manifest_digest(),
        governed.manifest_digest(),
        proof_digest,
        Digest32::from_bytes(*record_digest(&bad_prereg).unwrap().as_bytes()),
        Digest32::from_bytes(*record_digest(&attempts).unwrap().as_bytes()),
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
    );
    assert_eq!(
        validate_synthetic_profile_identity(
            &bad_prereg_run,
            &proof,
            &bad_prereg,
            &governed,
            mutation_digest,
        ),
        Err(SfsValidationError::ProofProfileDigestMismatch)
    );

    let prereg = identity_preregistration(
        &schedule,
        proof_digest,
        contract.manifest_digest(),
        mutation_digest,
    );
    let valid_run = run_identity(
        governed.host_component_manifest_digest(),
        governed.manifest_digest(),
        proof_digest,
        Digest32::from_bytes(*record_digest(&prereg).unwrap().as_bytes()),
        Digest32::from_bytes(*record_digest(&attempts).unwrap().as_bytes()),
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
    );
    assert_eq!(
        validate_synthetic_profile_identity(&valid_run, &proof, &prereg, &governed, digest(216),),
        Err(SfsValidationError::MutationManifestDigestMismatch)
    );
}

#[test]
fn candidate_ledger_schedule_exogenous_and_cadence_precedence_is_exact() {
    let (intents, schedule, attempts) = candidate_bundle();
    let contract = parse_synthetic_driver_contract(DRIVER_CONTRACT).unwrap();
    let parameter = Digest32::from_bytes(parameter_bytes_digest(&intents[0]).unwrap());
    let prereg = preregistration_custom(
        &schedule,
        contract.manifest_digest(),
        100,
        2,
        3,
        PracticeIdV1::Organize,
        99,
        3,
        parameter,
    );
    let driver = bind_synthetic_driver(&prereg, &contract).unwrap();
    let valid_run = run_for_candidate(&prereg, &attempts, SYNTHETIC_EMPTY_EXOGENOUS_DIGEST);
    let wrong_attempt = run_identity(
        digest(1),
        digest(2),
        digest(3),
        digest(4),
        digest(99),
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
    );
    assert_eq!(
        driver.validate_candidate_projection(
            &wrong_attempt,
            &prereg,
            &schedule,
            &attempts,
            &intents,
            SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        ),
        Err(SyntheticDriverError::AttemptLedgerDigestMismatch)
    );
    assert_eq!(
        driver.validate_candidate_projection(
            &valid_run,
            &prereg,
            &schedule,
            &attempts,
            &intents,
            digest(98),
        ),
        Err(SyntheticDriverError::ExogenousLedgerDigestMismatch)
    );
    let wrong_exogenous_run = run_for_candidate(&prereg, &attempts, digest(97));
    assert_eq!(
        driver.validate_candidate_projection(
            &wrong_exogenous_run,
            &prereg,
            &schedule,
            &attempts,
            &intents,
            SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        ),
        Err(SyntheticDriverError::ExogenousLedgerDigestMismatch)
    );

    let wrong_schedule_prereg = SfsPreregistrationV1::new(
        90,
        digest(2),
        digest(95),
        Digest32::from_bytes(*record_digest(&proof_profile()).unwrap().as_bytes()),
        contract.manifest_digest(),
        digest(3),
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        100,
        2,
        3,
        PracticeIdV1::Organize,
        Digest32::from_bytes(target_selection_policy_digest(
            PracticeTargetDomainV1::SocialClass,
            99,
        )),
        3,
        parameter,
    )
    .unwrap();
    let wrong_schedule_run = run_for_candidate(
        &wrong_schedule_prereg,
        &attempts,
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
    );
    let wrong_schedule_driver = bind_synthetic_driver(&wrong_schedule_prereg, &contract).unwrap();
    assert_eq!(
        wrong_schedule_driver.validate_candidate_projection(
            &wrong_schedule_run,
            &wrong_schedule_prereg,
            &schedule,
            &attempts,
            &intents,
            SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        ),
        Err(SyntheticDriverError::CandidateScheduleDigestMismatch)
    );
}

#[test]
fn candidate_count_tick_and_intent_count_refusals_are_specific() {
    let (intents, schedule, attempts) = candidate_bundle();
    let contract = parse_synthetic_driver_contract(DRIVER_CONTRACT).unwrap();
    let parameter = Digest32::from_bytes(parameter_bytes_digest(&intents[0]).unwrap());
    let count_prereg = preregistration_custom(
        &schedule,
        contract.manifest_digest(),
        100,
        2,
        4,
        PracticeIdV1::Organize,
        99,
        3,
        parameter,
    );
    let count_run = run_for_candidate(&count_prereg, &attempts, SYNTHETIC_EMPTY_EXOGENOUS_DIGEST);
    let count_driver = bind_synthetic_driver(&count_prereg, &contract).unwrap();
    assert!(matches!(
        count_driver.validate_candidate_projection(
            &count_run,
            &count_prereg,
            &schedule,
            &attempts,
            &intents,
            SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        ),
        Err(SyntheticDriverError::CandidateCadenceCountMismatch {
            declared: 4,
            actual: 3
        })
    ));
    let tick_prereg = preregistration_custom(
        &schedule,
        contract.manifest_digest(),
        101,
        2,
        3,
        PracticeIdV1::Organize,
        99,
        3,
        parameter,
    );
    let tick_run = run_for_candidate(&tick_prereg, &attempts, SYNTHETIC_EMPTY_EXOGENOUS_DIGEST);
    let tick_driver = bind_synthetic_driver(&tick_prereg, &contract).unwrap();
    assert!(matches!(
        tick_driver.validate_candidate_projection(
            &tick_run,
            &tick_prereg,
            &schedule,
            &attempts,
            &intents,
            SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        ),
        Err(SyntheticDriverError::CandidateCadenceTickMismatch { index: 0, .. })
    ));
    assert!(matches!(
        tick_driver.validate_candidate_projection(
            &tick_run,
            &tick_prereg,
            &schedule,
            &attempts,
            &intents[..2],
            SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        ),
        Err(SyntheticDriverError::CandidateIntentCountMismatch {
            expected: 3,
            actual: 2
        })
    ));
    let mut extra = intents.clone();
    extra.push(intent(106));
    assert_eq!(
        tick_driver.validate_candidate_projection(
            &tick_run,
            &tick_prereg,
            &schedule,
            &attempts,
            &extra,
            SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        ),
        Err(SyntheticDriverError::CandidateIntentCountMismatch {
            expected: 3,
            actual: 4,
        })
    );
}

#[test]
fn candidate_intent_field_and_parameter_identities_refuse_specifically() {
    let contract = parse_synthetic_driver_contract(DRIVER_CONTRACT).unwrap();
    let base_parameter = Digest32::from_bytes(parameter_bytes_digest(&intent(100)).unwrap());
    let cases = [
        (PracticeIdV1::Agitate, 99_u64, 3_u32),
        (PracticeIdV1::Organize, 100_u64, 3_u32),
        (PracticeIdV1::Organize, 99_u64, 4_u32),
    ];
    for (index, (practice, target, cost)) in cases.into_iter().enumerate() {
        let mut changed = vec![intent(100), intent(102), intent(104)];
        changed[0].practice_id = practice;
        changed[0].target_node_id = target;
        changed[0].quoted_action_budget_cost = cost;
        let (intents, schedule, attempts) = bundle_from_intents(changed);
        let prereg = preregistration_custom(
            &schedule,
            contract.manifest_digest(),
            100,
            2,
            3,
            PracticeIdV1::Organize,
            99,
            3,
            base_parameter,
        );
        let run = run_for_candidate(&prereg, &attempts, SYNTHETIC_EMPTY_EXOGENOUS_DIGEST);
        let driver = bind_synthetic_driver(&prereg, &contract).unwrap();
        let error = driver
            .validate_candidate_projection(
                &run,
                &prereg,
                &schedule,
                &attempts,
                &intents,
                SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
            )
            .unwrap_err();
        assert_eq!(
            error,
            [
                SyntheticDriverError::CandidatePracticeMismatch { index: 0 },
                SyntheticDriverError::CandidateTargetPolicyMismatch { index: 0 },
                SyntheticDriverError::CandidateGovernedCostMismatch { index: 0 },
            ][index]
        );
    }
    let (intents, schedule, attempts) = candidate_bundle();
    let prereg = preregistration_custom(
        &schedule,
        contract.manifest_digest(),
        100,
        2,
        3,
        PracticeIdV1::Organize,
        99,
        3,
        digest(96),
    );
    let run = run_for_candidate(&prereg, &attempts, SYNTHETIC_EMPTY_EXOGENOUS_DIGEST);
    let driver = bind_synthetic_driver(&prereg, &contract).unwrap();
    assert_eq!(
        driver.validate_candidate_projection(
            &run,
            &prereg,
            &schedule,
            &attempts,
            &intents,
            SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        ),
        Err(SyntheticDriverError::CandidateParameterBytesMismatch { index: 0 })
    );
}

#[test]
fn candidate_projection_and_complete_intent_order_are_closed() {
    let (intents, schedule, attempts) = candidate_bundle();
    let contract = parse_synthetic_driver_contract(DRIVER_CONTRACT).unwrap();
    let prereg = preregistration(&schedule, &proof_profile(), contract.manifest_digest());
    let driver = bind_synthetic_driver(&prereg, &contract).unwrap();
    let mut reordered = intents.clone();
    reordered.swap(0, 1);
    let run = run_for_candidate(&prereg, &attempts, SYNTHETIC_EMPTY_EXOGENOUS_DIGEST);
    assert_eq!(
        driver.validate_candidate_projection(
            &run,
            &prereg,
            &schedule,
            &attempts,
            &reordered,
            SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        ),
        Err(SyntheticDriverError::CandidateIntentDigestMismatch { index: 0 })
    );
    let alternate_rows = schedule
        .rows()
        .iter()
        .cloned()
        .map(|row| {
            PracticeCandidateRowV1::new(
                row.attempt_tick(),
                digest(95),
                row.practice_intent_digest(),
            )
        })
        .collect();
    let alternate_attempts = ledger_from_rows(alternate_rows);
    let alternate_run = run_for_candidate(
        &prereg,
        &alternate_attempts,
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
    );
    assert_eq!(
        driver.validate_candidate_projection(
            &alternate_run,
            &prereg,
            &schedule,
            &alternate_attempts,
            &intents,
            SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        ),
        Err(SyntheticDriverError::CandidateProjectionMismatch)
    );

    let mut moved = intents.clone();
    moved[0].submit_after_tick += 1;
    moved[0].resolve_tick += 1;
    let (moved_intents, moved_schedule, moved_attempts) = bundle_from_intents(moved);
    let moved_prereg = preregistration_custom(
        &moved_schedule,
        contract.manifest_digest(),
        100,
        2,
        3,
        PracticeIdV1::Organize,
        99,
        3,
        Digest32::from_bytes(parameter_bytes_digest(&moved_intents[0]).unwrap()),
    );
    let moved_run = run_for_candidate(
        &moved_prereg,
        &moved_attempts,
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
    );
    let moved_driver = bind_synthetic_driver(&moved_prereg, &contract).unwrap();
    assert_eq!(
        moved_driver.validate_candidate_projection(
            &moved_run,
            &moved_prereg,
            &moved_schedule,
            &moved_attempts,
            &moved_intents,
            SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        ),
        Err(SyntheticDriverError::CandidateCadenceTickMismatch {
            index: 0,
            expected: 100,
            actual: 101,
        })
    );
}

#[test]
fn malformed_adapter_intents_map_before_any_synthetic_run_membership() {
    let (intents, schedule, attempts) = candidate_bundle();
    let contract = parse_synthetic_driver_contract(DRIVER_CONTRACT).unwrap();
    let prereg = preregistration(&schedule, &proof_profile(), contract.manifest_digest());
    let run = run_for_candidate(&prereg, &attempts, SYNTHETIC_EMPTY_EXOGENOUS_DIGEST);
    let driver = bind_synthetic_driver(&prereg, &contract).unwrap();

    let mut invalid_parameter = intents.clone();
    invalid_parameter[0].parameters.push(PracticeParameterV1 {
        key_u8: 1,
        value_kind_u8: 1,
        value_length_u16: 2,
        value_bytes: vec![1],
    });
    assert_eq!(
        driver.validate_candidate_projection(
            &run,
            &prereg,
            &schedule,
            &attempts,
            &invalid_parameter,
            SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        ),
        Err(SyntheticDriverError::CandidateParameterBytesMismatch { index: 0 })
    );

    let mut unsorted_evidence = intents;
    unsorted_evidence[0].evidence_digests = vec![[2; 32], [1; 32]];
    assert_eq!(
        driver.validate_candidate_projection(
            &run,
            &prereg,
            &schedule,
            &attempts,
            &unsorted_evidence,
            SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        ),
        Err(SyntheticDriverError::CandidateIntentDigestMismatch { index: 0 })
    );
}

#[test]
fn every_run_field_moves_identity() {
    let base_row = WIRE_VECTORS
        .lines()
        .find(|row| row.starts_with("wire|run-identity|"))
        .unwrap();
    let base: RunIdentityV1 =
        decode_envelope(&hex_bytes(base_row.split('|').nth(3).unwrap())).unwrap();
    let base_envelope = canonical_envelope(&base).unwrap();
    let mut fields = Vec::new();
    for row in IDENTITY_MUTATIONS.lines() {
        let parts = row.split('|').collect::<Vec<_>>();
        let changed: RunIdentityV1 = decode_envelope(&hex_bytes(parts[3])).unwrap();
        let differences = base.differing_fields(&changed);
        assert_eq!(differences.len(), 1, "{}", parts[2]);
        assert_ne!(canonical_envelope(&changed).unwrap(), base_envelope);
        assert_ne!(
            record_digest(&changed).unwrap(),
            record_digest(&base).unwrap()
        );
        fields.push(differences[0]);
    }
    fields.sort_by_key(|field| *field as u8);
    fields.dedup();
    assert_eq!(fields.len(), 18);

    let base = run_identity(
        digest(1),
        digest(2),
        digest(3),
        digest(4),
        digest(5),
        digest(6),
    );
    let changed_attempt = run_identity(
        digest(1),
        digest(2),
        digest(3),
        digest(4),
        digest(7),
        digest(6),
    );
    let changed_both = run_identity(
        digest(1),
        digest(2),
        digest(3),
        digest(4),
        digest(7),
        digest(8),
    );
    assert_ne!(
        record_digest(&base).unwrap(),
        record_digest(&changed_attempt).unwrap()
    );
    let contract = parse_synthetic_driver_contract(DRIVER_CONTRACT).unwrap();
    let (_, schedule, _) = candidate_bundle();
    let prereg = preregistration(&schedule, &proof_profile(), contract.manifest_digest());
    let driver = bind_synthetic_driver(&prereg, &contract).unwrap();
    assert_eq!(
        driver.validate_twin_identity_difference(
            &base,
            &changed_attempt,
            babylon_evidence::DifferingLedgerKindV1::PracticeAttempt,
        ),
        Ok(())
    );
    assert!(driver
        .validate_twin_identity_difference(
            &base,
            &changed_both,
            babylon_evidence::DifferingLedgerKindV1::PracticeAttempt,
        )
        .is_err());
    assert_eq!(
        driver.validate_twin_identity_difference(
            &base,
            &changed_attempt,
            DifferingLedgerKindV1::ExogenousInput,
        ),
        Err(SyntheticDriverError::TwinChangedWrongLedger)
    );
    let changed_host = run_identity(
        digest(9),
        digest(2),
        digest(3),
        digest(4),
        digest(5),
        digest(6),
    );
    assert_eq!(
        driver.validate_twin_identity_difference(
            &base,
            &changed_host,
            DifferingLedgerKindV1::PracticeAttempt,
        ),
        Err(SyntheticDriverError::TwinChangedNonLedgerField {
            field: RunIdentityField::HostComponentManifest,
        })
    );
}

fn trace(run: &RunIdentityV1, tag: u8) -> SfsTraceV1 {
    let run_digest = Digest32::from_bytes(*record_digest(run).unwrap().as_bytes());
    let masses = [0.0, 1.0, 2.0, 5.0, 8.0, 10.0, 11.0];
    let samples = masses
        .iter()
        .enumerate()
        .map(|(index, mass)| {
            SfsSampleV1::new(
                200 + u64::try_from(index).unwrap(),
                digest(tag),
                digest(tag + 1),
                digest(tag + 2),
                *mass,
            )
            .unwrap()
        })
        .collect();
    SfsTraceV1::new(run_digest, digest(90), 7, 200, 2, samples).unwrap()
}

fn make_comparison(
    control_trace_digest: Digest32,
    intervention_trace_digest: Digest32,
    kind: DifferingLedgerKindV1,
    control_ledger_digest: Digest32,
    intervention_ledger_digest: Digest32,
    delta_digest: Digest32,
) -> PersistenceComparisonV1 {
    PersistenceComparisonV1::new(
        control_trace_digest,
        intervention_trace_digest,
        kind,
        control_ledger_digest,
        intervention_ledger_digest,
        delta_digest,
        199,
        2,
        vec![2.0, 1.0, 0.5],
    )
    .unwrap()
}

struct PersistenceFixture {
    control: RunIdentityV1,
    intervention: RunIdentityV1,
    control_trace: SfsTraceV1,
    intervention_trace: SfsTraceV1,
    delta: InterventionDeltaV1,
    comparison: PersistenceComparisonV1,
    control_trace_digest: Digest32,
    intervention_trace_digest: Digest32,
    delta_digest: Digest32,
}

fn persistence_fixture() -> PersistenceFixture {
    let control = run_identity(
        digest(1),
        digest(2),
        digest(3),
        digest(4),
        digest(5),
        digest(6),
    );
    let intervention = run_identity(
        digest(1),
        digest(2),
        digest(3),
        digest(4),
        digest(7),
        digest(6),
    );
    let control_trace = trace(&control, 100);
    let intervention_trace = trace(&intervention, 110);
    let delta = InterventionDeltaV1::new(
        DifferingLedgerKindV1::PracticeAttempt,
        vec![InterventionDeltaRowV1::new(
            InterventionOperationV1::Replace,
            digest(120),
            digest(121),
            digest(122),
        )
        .unwrap()],
    )
    .unwrap();
    let control_trace_digest =
        Digest32::from_bytes(*record_digest(&control_trace).unwrap().as_bytes());
    let intervention_trace_digest =
        Digest32::from_bytes(*record_digest(&intervention_trace).unwrap().as_bytes());
    let delta_digest = Digest32::from_bytes(*record_digest(&delta).unwrap().as_bytes());
    let comparison = make_comparison(
        control_trace_digest,
        intervention_trace_digest,
        DifferingLedgerKindV1::PracticeAttempt,
        control.practice_attempt_ledger_digest(),
        intervention.practice_attempt_ledger_digest(),
        delta_digest,
    );
    PersistenceFixture {
        control,
        intervention,
        control_trace,
        intervention_trace,
        delta,
        comparison,
        control_trace_digest,
        intervention_trace_digest,
        delta_digest,
    }
}

fn bound_driver_contract() -> (
    babylon_evidence::SyntheticDriverContractV1,
    SfsPreregistrationV1,
) {
    let contract = parse_synthetic_driver_contract(DRIVER_CONTRACT).unwrap();
    let (_, schedule, _) = candidate_bundle();
    let prereg = preregistration(&schedule, &proof_profile(), contract.manifest_digest());
    (contract, prereg)
}

#[test]
fn persistence_trace_run_identity_bindings_are_exact() {
    let fixture = persistence_fixture();
    let (contract, prereg) = bound_driver_contract();
    let driver = bind_synthetic_driver(&prereg, &contract).unwrap();
    assert_eq!(
        driver.validate_persistence_comparison_identity(
            &fixture.control,
            &fixture.intervention,
            &fixture.control_trace,
            &fixture.intervention_trace,
            &fixture.comparison,
            &fixture.delta,
        ),
        Ok(())
    );
    let wrong_trace = trace(&fixture.intervention, 100);
    assert_eq!(
        driver.validate_persistence_comparison_identity(
            &fixture.control,
            &fixture.intervention,
            &wrong_trace,
            &fixture.intervention_trace,
            &fixture.comparison,
            &fixture.delta,
        ),
        Err(SyntheticDriverError::ControlTraceRunIdentityMismatch)
    );
    let wrong_intervention_trace = trace(&fixture.control, 110);
    assert_eq!(
        driver.validate_persistence_comparison_identity(
            &fixture.control,
            &fixture.intervention,
            &fixture.control_trace,
            &wrong_intervention_trace,
            &fixture.comparison,
            &fixture.delta,
        ),
        Err(SyntheticDriverError::InterventionTraceRunIdentityMismatch)
    );
}

#[test]
fn persistence_comparison_stored_digests_are_exact() {
    let fixture = persistence_fixture();
    let (contract, prereg) = bound_driver_contract();
    let driver = bind_synthetic_driver(&prereg, &contract).unwrap();
    let cases = [
        (
            make_comparison(
                digest(200),
                fixture.intervention_trace_digest,
                DifferingLedgerKindV1::PracticeAttempt,
                fixture.control.practice_attempt_ledger_digest(),
                fixture.intervention.practice_attempt_ledger_digest(),
                fixture.delta_digest,
            ),
            SyntheticDriverError::ComparisonControlTraceDigestMismatch,
        ),
        (
            make_comparison(
                fixture.control_trace_digest,
                digest(201),
                DifferingLedgerKindV1::PracticeAttempt,
                fixture.control.practice_attempt_ledger_digest(),
                fixture.intervention.practice_attempt_ledger_digest(),
                fixture.delta_digest,
            ),
            SyntheticDriverError::ComparisonInterventionTraceDigestMismatch,
        ),
        (
            make_comparison(
                fixture.control_trace_digest,
                fixture.intervention_trace_digest,
                DifferingLedgerKindV1::PracticeAttempt,
                digest(202),
                fixture.intervention.practice_attempt_ledger_digest(),
                fixture.delta_digest,
            ),
            SyntheticDriverError::ComparisonControlLedgerDigestMismatch,
        ),
        (
            make_comparison(
                fixture.control_trace_digest,
                fixture.intervention_trace_digest,
                DifferingLedgerKindV1::PracticeAttempt,
                fixture.control.practice_attempt_ledger_digest(),
                digest(203),
                fixture.delta_digest,
            ),
            SyntheticDriverError::ComparisonInterventionLedgerDigestMismatch,
        ),
        (
            make_comparison(
                fixture.control_trace_digest,
                fixture.intervention_trace_digest,
                DifferingLedgerKindV1::PracticeAttempt,
                fixture.control.practice_attempt_ledger_digest(),
                fixture.intervention.practice_attempt_ledger_digest(),
                digest(204),
            ),
            SyntheticDriverError::ComparisonInterventionDeltaDigestMismatch,
        ),
    ];
    for (changed, expected) in cases {
        assert_eq!(
            driver.validate_persistence_comparison_identity(
                &fixture.control,
                &fixture.intervention,
                &fixture.control_trace,
                &fixture.intervention_trace,
                &changed,
                &fixture.delta,
            ),
            Err(expected)
        );
    }
}

#[test]
fn persistence_selected_kind_and_delta_kind_are_exact() {
    let fixture = persistence_fixture();
    let (contract, prereg) = bound_driver_contract();
    let driver = bind_synthetic_driver(&prereg, &contract).unwrap();
    let wrong_kind = make_comparison(
        fixture.control_trace_digest,
        fixture.intervention_trace_digest,
        DifferingLedgerKindV1::ExogenousInput,
        fixture.control.exogenous_input_ledger_digest(),
        fixture.intervention.exogenous_input_ledger_digest(),
        fixture.delta_digest,
    );
    assert_eq!(
        driver.validate_persistence_comparison_identity(
            &fixture.control,
            &fixture.intervention,
            &fixture.control_trace,
            &fixture.intervention_trace,
            &wrong_kind,
            &fixture.delta,
        ),
        Err(SyntheticDriverError::TwinChangedWrongLedger)
    );

    let wrong_delta = InterventionDeltaV1::new(
        DifferingLedgerKindV1::ExogenousInput,
        vec![InterventionDeltaRowV1::new(
            InterventionOperationV1::Replace,
            digest(120),
            digest(121),
            digest(122),
        )
        .unwrap()],
    )
    .unwrap();
    assert_eq!(
        driver.validate_persistence_comparison_identity(
            &fixture.control,
            &fixture.intervention,
            &fixture.control_trace,
            &fixture.intervention_trace,
            &fixture.comparison,
            &wrong_delta,
        ),
        Err(SyntheticDriverError::ComparisonLedgerKindMismatch)
    );
}

#[test]
fn cadence_overflow_precedes_tick_comparison() {
    let first = intent(u64::MAX - 1);
    let second = intent(u64::MAX);
    let rows = [&first, &second]
        .iter()
        .enumerate()
        .map(|(index, value)| {
            PracticeCandidateRowV1::new(
                value.resolve_tick,
                digest(130 + u8::try_from(index).unwrap()),
                Digest32::from_bytes(intent_digest(value).unwrap()),
            )
        })
        .collect::<Vec<_>>();
    let schedule = PracticeCandidateScheduleV1::new(rows.clone()).unwrap();
    let attempts = rows
        .into_iter()
        .map(|row| {
            PracticeAttemptRowV1::new(row, PracticeDispositionV1::Rejected, digest(140)).unwrap()
        })
        .collect();
    let attempts = PracticeAttemptLedgerV1::new(digest(141), attempts).unwrap();
    let contract = parse_synthetic_driver_contract(DRIVER_CONTRACT).unwrap();
    let prereg = SfsPreregistrationV1::new(
        80,
        digest(2),
        Digest32::from_bytes(*record_digest(&schedule).unwrap().as_bytes()),
        Digest32::from_bytes(*record_digest(&proof_profile()).unwrap().as_bytes()),
        contract.manifest_digest(),
        digest(3),
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        u64::MAX - 1,
        2,
        2,
        PracticeIdV1::Organize,
        Digest32::from_bytes(target_selection_policy_digest(
            PracticeTargetDomainV1::SocialClass,
            99,
        )),
        3,
        Digest32::from_bytes(parameter_bytes_digest(&first).unwrap()),
    )
    .unwrap();
    let run = run_identity(
        digest(4),
        digest(5),
        digest(6),
        Digest32::from_bytes(*record_digest(&prereg).unwrap().as_bytes()),
        Digest32::from_bytes(*record_digest(&attempts).unwrap().as_bytes()),
        SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
    );
    let driver = bind_synthetic_driver(&prereg, &contract).unwrap();
    assert_eq!(
        driver.validate_candidate_projection(
            &run,
            &prereg,
            &schedule,
            &attempts,
            &[first, second],
            SYNTHETIC_EMPTY_EXOGENOUS_DIGEST,
        ),
        Err(SyntheticDriverError::CandidateCadenceOverflow { index: 1 })
    );
}

#[test]
fn mutation_manifest_is_exact_dependency_labelled_specification() {
    let contract = parse_synthetic_driver_contract(DRIVER_CONTRACT).unwrap();
    let (_, schedule, _) = candidate_bundle();
    let prereg = preregistration(&schedule, &proof_profile(), contract.manifest_digest());
    assert_eq!(
        validate_synthetic_mutation_manifest(MUTATIONS, &prereg),
        Ok(prereg.mutation_manifest_digest())
    );
    let changed_activation = String::from_utf8(MUTATIONS.to_vec())
        .unwrap()
        .replace("|GATE5|-", "|SYNTHETIC|-");
    assert_eq!(
        validate_synthetic_mutation_manifest(changed_activation.as_bytes(), &prereg),
        Err(SfsValidationError::MutationManifestByteLimit {
            actual: changed_activation.len(),
        })
    );
    let crlf = String::from_utf8(MUTATIONS.to_vec())
        .unwrap()
        .replacen('\n', "\r\n", 1);
    assert_eq!(
        validate_synthetic_mutation_manifest(crlf.as_bytes(), &prereg),
        Err(SfsValidationError::MutationManifestByteLimit { actual: 4_400 })
    );

    let rows = String::from_utf8(MUTATIONS.to_vec())
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert!(matches!(
        validate_synthetic_mutation_manifest(
            format!("{}\n", rows[..40].join("\n")).as_bytes(),
            &prereg,
        ),
        Err(SfsValidationError::MutationCoverageMismatch { .. })
    ));
    let mut extra = rows.clone();
    extra.push("Z99_EXTRA|STATIC|x|x|GATE3|-".to_owned());
    let extra_manifest = sorted_manifest(extra);
    assert_eq!(
        validate_synthetic_mutation_manifest(&extra_manifest, &prereg),
        Err(SfsValidationError::MutationManifestByteLimit {
            actual: extra_manifest.len(),
        })
    );
    let mut duplicate = rows.clone();
    duplicate[1] = duplicate[0].clone();
    assert!(matches!(
        validate_synthetic_mutation_manifest(&sorted_manifest(duplicate), &prereg),
        Err(SfsValidationError::MutationManifestMalformed { .. })
    ));
    let unknown_phase = String::from_utf8(MUTATIONS.to_vec())
        .unwrap()
        .replacen("|DRIVER|", "|BROKEN|", 1);
    assert!(matches!(
        validate_synthetic_mutation_manifest(unknown_phase.as_bytes(), &prereg),
        Err(SfsValidationError::MutationManifestMalformed { .. })
    ));
    let unknown_activation = String::from_utf8(MUTATIONS.to_vec())
        .unwrap()
        .replacen("|GATE5|-", "|WRONG|-", 1);
    assert!(matches!(
        validate_synthetic_mutation_manifest(unknown_activation.as_bytes(), &prereg),
        Err(SfsValidationError::MutationManifestMalformed { .. })
    ));
    let changed_test = String::from_utf8(MUTATIONS.to_vec()).unwrap().replacen(
        "|aligned_material_bits_match",
        "|wrong_executable_test",
        1,
    );
    assert_eq!(
        validate_synthetic_mutation_manifest(changed_test.as_bytes(), &prereg),
        Err(SfsValidationError::MutationManifestDigestMismatch)
    );
    assert_eq!(
        validate_synthetic_mutation_manifest(&MUTATIONS[..MUTATIONS.len() - 1], &prereg),
        Err(SfsValidationError::MutationManifestMalformed { row: 0 })
    );
}

#[test]
fn synthetic_manifest_test_names_resolve_to_genuine_exact_tests() {
    let sources = [
        include_str!("../src/driver.rs"),
        include_str!("classifier_goldens.rs"),
        include_str!("synthetic_proof_harness.rs"),
        include_str!("../../babylon-bsl/tests/sfs_profile_contract.rs"),
        include_str!("../../bsl-lint/tests/sfs_non_authorability.rs"),
    ];
    for row in String::from_utf8(MUTATIONS.to_vec()).unwrap().lines() {
        let fields = row.split('|').collect::<Vec<_>>();
        if fields[4] != "SYNTHETIC" {
            continue;
        }
        assert_ne!(fields[5], "-");
        let exact = format!("fn {}(", fields[5]);
        assert!(
            sources.iter().any(|source| source.contains(&exact)),
            "missing exact executable test {}",
            fields[5]
        );
    }
}

#[test]
fn mutation_manifest_byte_preflight_precedes_framing_and_allocation() {
    assert_eq!(MUTATIONS.len(), 4_399);
    let contract = parse_synthetic_driver_contract(DRIVER_CONTRACT).unwrap();
    let (_, schedule, _) = candidate_bundle();
    let prereg = preregistration(&schedule, &proof_profile(), contract.manifest_digest());
    assert_eq!(
        validate_synthetic_mutation_manifest(MUTATIONS, &prereg),
        Ok(prereg.mutation_manifest_digest())
    );
    let mut plus_one = vec![0xff; 4_400];
    plus_one[0] = b'\r';
    assert_eq!(
        validate_synthetic_mutation_manifest(&plus_one, &prereg),
        Err(SfsValidationError::MutationManifestByteLimit { actual: 4_400 })
    );
}

#[test]
fn validation_source_uses_only_literal_indexed_bounded_traversals() {
    let source = include_str!("../src/validation.rs");
    for forbidden in [
        ".iter()",
        ".into_iter()",
        ".split(",
        ".filter(",
        ".map(",
        ".find(",
        ".position(",
        ".zip(",
        ".take(",
        ".bytes()",
        ".chars()",
        ".nfc()",
        ".contains(",
        ".ends_with(",
        "for start in",
        "for id in",
        "for (set_name, ids) in",
    ] {
        assert!(
            !source.contains(forbidden),
            "forbidden traversal: {forbidden}"
        );
    }
    for required in [
        "const MAX_MUTATION_MANIFEST_BYTES: usize = 4_399;",
        "for index in 0..MAX_MUTATION_MANIFEST_BYTES",
        "for index in 0..MAX_GOVERNED_MANIFEST_BYTES",
        "for index in 0..MAX_COMPONENTS",
        "for index in 0..MAX_PROFILE_ROWS",
        "for index in 0..MAX_EDGES",
        "for _pass in 0..MAX_COMPONENTS",
        "for index in 0..MAX_PROFILE_ENTRIES",
    ] {
        assert!(
            source.contains(required),
            "missing literal ceiling: {required}"
        );
    }
}

#[test]
fn governed_manifest_global_byte_row_and_line_bounds_precede_parsing() {
    let exact_bytes = [vec![b'a'; 1_048_575], vec![b'\n']].concat();
    assert_ne!(
        parse_synthetic_governed_manifest(&exact_bytes, &rule(), &audit()),
        Err(SfsValidationError::GovernedManifestByteLimit { actual: 1_048_576 })
    );
    let oversized = vec![b'a'; 1_048_577];
    assert_eq!(
        parse_synthetic_governed_manifest(&oversized, &rule(), &audit()),
        Err(SfsValidationError::GovernedManifestByteLimit { actual: 1_048_577 })
    );
    let exact_line = [vec![b'a'; 131_681], vec![b'\n']].concat();
    assert_ne!(
        parse_synthetic_governed_manifest(&exact_line, &rule(), &audit()),
        Err(SfsValidationError::GovernedManifestLineLimit {
            row: 1,
            actual: 131_682,
        })
    );
    let long_line = [vec![b'a'; 131_682], vec![b'\n']].concat();
    assert_eq!(
        parse_synthetic_governed_manifest(&long_line, &rule(), &audit()),
        Err(SfsValidationError::GovernedManifestLineLimit {
            row: 1,
            actual: 131_683
        })
    );
    let mut exact_rows = Vec::new();
    for index in 0..36_992 {
        exact_rows.extend_from_slice(format!("{index:05}\n").as_bytes());
    }
    assert_ne!(
        parse_synthetic_governed_manifest(&exact_rows, &rule(), &audit()),
        Err(SfsValidationError::GovernedManifestTotalRowLimit { actual: 36_992 })
    );
    let mut rows = exact_rows;
    rows.extend_from_slice(b"36992\n");
    assert_eq!(
        parse_synthetic_governed_manifest(&rows, &rule(), &audit()),
        Err(SfsValidationError::GovernedManifestTotalRowLimit { actual: 36_993 })
    );
}

#[test]
fn governed_manifest_family_limits_accept_maximum_then_refuse_plus_one() {
    assert_family_limit(
        "component",
        64,
        65,
        SfsValidationError::ComponentLimit { actual: 65 },
    );
    assert_family_limit(
        "bound",
        64,
        65,
        SfsValidationError::BoundRowLimit { actual: 65 },
    );
    assert_family_limit(
        "edge",
        4_096,
        4_097,
        SfsValidationError::EdgeLimit { actual: 4_097 },
    );
    assert_family_limit(
        "profile",
        32_768,
        32_769,
        SfsValidationError::ProfileRowLimit { actual: 32_769 },
    );
}

fn assert_family_limit(kind: &str, maximum: usize, plus_one: usize, expected: SfsValidationError) {
    let maximum_bytes = family_rows(kind, maximum);
    assert_ne!(
        parse_synthetic_governed_manifest(&maximum_bytes, &rule(), &audit()),
        Err(expected.clone())
    );
    assert_eq!(
        parse_synthetic_governed_manifest(&family_rows(kind, plus_one), &rule(), &audit()),
        Err(expected)
    );
}

fn family_rows(kind: &str, count: usize) -> Vec<u8> {
    let digest_hex = lower_hex(&sha256_of(b"babylon.sfs-synthetic-component-source.v1\0x"));
    let mut rows = Vec::new();
    for index in 0..32_769 {
        if index >= count {
            break;
        }
        let id = base36_4(index);
        let id_hex = lower_hex(id.as_bytes());
        let row = match kind {
            "component" => format!("component|{id_hex}|2|synthetic-descriptor|78|{digest_hex}\n"),
            "bound" => format!(
                "bound|{id_hex}|1|1|{}|{}\n",
                "01".repeat(32),
                "02".repeat(32)
            ),
            "edge" => format!("edge|{id_hex}|{id_hex}|0|{id_hex}\n"),
            "profile" => format!("profile|61|operators|{id_hex}\n"),
            _ => panic!("unknown family"),
        };
        rows.extend_from_slice(row.as_bytes());
    }
    rows
}

#[test]
fn source_payload_and_complete_row_order_boundaries_are_exact() {
    let maximum_payload = vec![b'x'; 65_535];
    let maximum_id = vec![b'a'; 256];
    let maximum_row = format!(
        "component|{}|2|synthetic-descriptor|{}|{}\n",
        lower_hex(&maximum_id),
        lower_hex(&maximum_payload),
        domain_digest(
            b"babylon.sfs-synthetic-component-source.v1",
            &maximum_payload
        )
        .to_hex(),
    );
    assert_ne!(
        parse_synthetic_governed_manifest(maximum_row.as_bytes(), &rule(), &audit()),
        Err(SfsValidationError::SourcePayloadLimit {
            component_id: "a".repeat(256),
            actual: 65_535,
        })
    );
    assert_ne!(
        parse_synthetic_governed_manifest(maximum_row.as_bytes(), &rule(), &audit()),
        Err(SfsValidationError::GovernedManifestLineLimit {
            row: 1,
            actual: maximum_row.len(),
        })
    );
    let payload = "00".repeat(65_536);
    let row = format!(
        "component|61|2|synthetic-descriptor|{payload}|{}\n",
        "00".repeat(32)
    );
    assert!(matches!(
        parse_synthetic_governed_manifest(row.as_bytes(), &rule(), &audit()),
        Err(SfsValidationError::SourcePayloadLimit { actual: 65_536, .. })
    ));
    let text = String::from_utf8(GOVERNED.to_vec()).unwrap();
    let mut lines = text.lines().collect::<Vec<_>>();
    lines.swap(5, 6);
    let reordered = format!("{}\n", lines.join("\n"));
    assert!(matches!(
        parse_synthetic_governed_manifest(reordered.as_bytes(), &rule(), &audit()),
        Err(SfsValidationError::GovernedManifestMalformed { .. })
    ));
}
