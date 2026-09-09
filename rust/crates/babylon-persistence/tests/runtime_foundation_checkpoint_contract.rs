//! Database-free foundation and full-checkpoint composition contract.

use babylon_bsl::rule_pipeline::split_content;
use babylon_bsl::rules_hash_of;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
use babylon_kernel::sha256_of;
use babylon_kernel::tick_content_hash::RefDigestV1;
use babylon_kernel::ContentDigest;
use babylon_persistence::{
    michigan_dynamic_hex_foundation_v1, prepare_committed_tick_v2, ArchiveDirtyReceiptV1,
    CampaignFoundationV1, CampaignId, CheckpointCompletenessV1, CheckpointRowsV1,
    CommittedCheckpointSectionV1, CommittedFullCheckpointV1, CommittedResolveTickErrorV1,
    CommittedResolveTickV1, FoundationContentBundleV1, FullCheckpointSectionTagV1,
    RustPersistenceRuntimeErrorV2,
};
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use babylon_tick::material_state::MaterialStateV1;
use babylon_tick::replay_session::{IdentifiedTickReportV2, ReplayTickSession};
use uuid::Uuid;

const SCENARIO: &str = r"
(scenario demo/foundation-checkpoint
  (defvocabulary NodeType (SOCIAL_CLASS))
  (deffield social-class/draw coefficient extensive)
  (node class-a NodeType/SOCIAL_CLASS (social-class/draw 0.0c)))
";
const RULE: &str = r#"
(rule production/foundation-checkpoint
  :role mechanic
  :evidence derived
  :material-basis "foundation checkpoint composition law"
  :fuel 32
  (bindings (binding draw :field social-class/draw))
  (when #t)
  (effects
    (update-node self social-class/draw (set 0.25c))
    (emit EventType/CHECKPOINTED (subject self))))
"#;
const DEFINES: &[u8] = br#"{"alpha":1}"#;
const ALTERNATE_DEFINES: &[u8] = br#"{"alpha":2}"#;
const REFERENCE_BUNDLE_DOMAIN: &[u8] = b"babylon.h3.reference-bundle-composite.v1\0";

fn fixture() -> (
    ReplayTickSession<HypergraphStore>,
    ReplaySessionIdV1,
    ReplaySeed,
    ContentDigest,
    RefDigestV1,
    Vec<u8>,
) {
    let (_, rules) = split_content(RULE).expect("rule parses");
    let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
    let content = ContentDigest {
        defines_hash: sha256_of(DEFINES),
        rules_hash: rules_hash_of(&forms).expect("rules hash"),
    };
    let session_id =
        ReplaySessionIdV1::try_from("per281/foundation-checkpoint").expect("session identity");
    let seed = ReplaySeed::new(281);
    let foundation = michigan_dynamic_hex_foundation_v1().expect("foundation decodes");
    let mut reference_manifest = REFERENCE_BUNDLE_DOMAIN.to_vec();
    reference_manifest.extend_from_slice(&foundation.base_reference_cohort_digest());
    reference_manifest.extend_from_slice(&foundation.r8_section_digest());
    assert_eq!(
        sha256_of(&reference_manifest),
        foundation.reference_bundle_digest()
    );
    let reference = RefDigestV1::from_bytes(foundation.reference_bundle_digest());
    let session = ReplayTickSession::new(
        SCENARIO,
        None,
        RULE,
        HypergraphStore::new(),
        session_id.clone(),
        seed,
        content.clone(),
        reference,
        MaterialStateV1::try_new(foundation).expect("material source"),
    )
    .expect("tick-zero session prepares");
    (
        session,
        session_id,
        seed,
        content,
        reference,
        reference_manifest,
    )
}

fn first_report(
    session: &mut ReplayTickSession<HypergraphStore>,
    session_id: &ReplaySessionIdV1,
) -> IdentifiedTickReportV2 {
    let actions =
        OrderedPracticeActionBatchV1::empty(session_id.clone(), 1).expect("empty actions");
    session
        .advance(&mut CollectingSink::default(), &actions)
        .expect("tick succeeds")
}

#[test]
fn committed_resolve_ticks_are_positive_postgresql_bigints() {
    assert_eq!(
        CommittedResolveTickV1::try_from(0_u64),
        Err(CommittedResolveTickErrorV1::SyntheticTickZero)
    );
    assert_eq!(
        CommittedResolveTickV1::try_from(1_u64)
            .expect("first durable tick")
            .get(),
        1
    );
    assert_eq!(
        CommittedResolveTickV1::try_from(i64::MAX as u64)
            .expect("PostgreSQL BIGINT ceiling")
            .get(),
        i64::MAX as u64
    );
    assert_eq!(
        CommittedResolveTickV1::try_from((i64::MAX as u64) + 1),
        Err(CommittedResolveTickErrorV1::OutOfPostgresRange)
    );
}

#[test]
fn tick_zero_foundation_retains_every_exact_replay_source() {
    let (session, session_id, seed, content, reference, reference_manifest) = fixture();
    let stable_graph = session
        .stable_graph_state()
        .expect("tick-zero graph has stable identity");
    let world_registers = session
        .world_registers()
        .expect("synthetic tick zero is foundation-only");
    let bundle =
        FoundationContentBundleV1::try_new(SCENARIO, None, RULE, DEFINES, &reference_manifest)
            .expect("bounded exact content bundle");
    let foundation = CampaignFoundationV1::capture(&session, bundle).expect("tick-zero capture");

    assert_eq!(
        foundation.stable_graph_bytes(),
        stable_graph.canonical_bytes()
    );
    assert_eq!(
        foundation.world_register_bytes(),
        world_registers.canonical_bytes()
    );
    assert_eq!(
        foundation.resolver_manifest_bytes(),
        session.resolver_manifest_bytes()
    );
    assert_eq!(
        foundation.prepared_environment_bytes(),
        session.prepared_environment_bytes()
    );
    assert_eq!(foundation.replay_session_identity(), &session_id);
    assert_eq!(foundation.rng_seed(), seed);
    assert_eq!(foundation.content_digest(), &content);
    assert_eq!(foundation.reference_digest(), reference);
    assert_eq!(
        foundation.content_bundle().scenario_source_bytes(),
        SCENARIO.as_bytes()
    );
    assert_eq!(foundation.content_bundle().prelude_source_bytes(), None);
    assert_eq!(
        foundation.content_bundle().rule_source_bytes(),
        RULE.as_bytes()
    );
    assert_eq!(foundation.content_bundle().defines_bytes(), DEFINES);
    assert_eq!(
        foundation
            .content_bundle()
            .reference_bundle_manifest_bytes(),
        reference_manifest
    );
    assert_eq!(foundation.content_bundle().content_digest(), &content);
    assert_eq!(foundation.content_bundle().reference_digest(), reference);

    let source = include_str!("../src/foundation.rs");
    assert!(!source.contains("prepare_rules("));
    assert!(!source.contains("run_prepared_replay_tick("));
    assert!(!source.contains(".advance("));
}

#[test]
fn foundation_bundle_is_cryptographically_bound_to_session_identities() {
    let (session, _, _, content, reference, reference_manifest) = fixture();
    let alternate_rule = RULE.replace("0.25c", "0.50c");

    for bundle in [
        FoundationContentBundleV1::try_new(
            SCENARIO,
            None,
            RULE,
            ALTERNATE_DEFINES,
            &reference_manifest,
        )
        .expect("alternate defines still form a bounded bundle"),
        FoundationContentBundleV1::try_new(
            SCENARIO,
            None,
            &alternate_rule,
            DEFINES,
            &reference_manifest,
        )
        .expect("alternate rule still forms a bounded bundle"),
    ] {
        assert_eq!(
            CampaignFoundationV1::capture(&session, bundle),
            Err(RustPersistenceRuntimeErrorV2::ReplaySource)
        );
    }

    let mut alternate_reference = reference_manifest.clone();
    *alternate_reference.last_mut().expect("nonempty manifest") ^= 1;
    let bundle =
        FoundationContentBundleV1::try_new(SCENARIO, None, RULE, DEFINES, &alternate_reference)
            .expect("alternate reference still forms a bounded bundle");
    assert_eq!(
        CampaignFoundationV1::capture(&session, bundle),
        Err(RustPersistenceRuntimeErrorV2::ReplaySource)
    );

    let bundle =
        FoundationContentBundleV1::try_new(SCENARIO, None, RULE, DEFINES, &reference_manifest)
            .expect("exact bundle");
    assert_eq!(bundle.content_digest(), &content);
    assert_eq!(bundle.reference_digest(), reference);
    CampaignFoundationV1::capture(&session, bundle).expect("bound bundle captures");
}

#[test]
fn foundation_refuses_a_bundle_whose_scenario_cannot_reproduce_the_session_graph() {
    // The content digest binds defines + rules only, so a caller can pair a
    // session built from scenario A with a bundle carrying scenario B. The
    // declared county mapping would then persist B's geography while the
    // runtime runs A's graph — capture must bind the bundle's scenario to
    // the session's captured graph and refuse the mismatch.
    let (session, _, _, content, reference, reference_manifest) = fixture();
    let foreign_scenario = SCENARIO.replace("0.0c", "0.5c");
    assert_ne!(foreign_scenario, SCENARIO);
    let bundle = FoundationContentBundleV1::try_new(
        &foreign_scenario,
        None,
        RULE,
        DEFINES,
        &reference_manifest,
    )
    .expect("foreign scenario still forms a bounded bundle");
    // The legacy digest bind cannot see a scenario-only difference.
    assert_eq!(bundle.content_digest(), &content);
    assert_eq!(bundle.reference_digest(), reference);
    assert_eq!(
        CampaignFoundationV1::capture(&session, bundle),
        Err(RustPersistenceRuntimeErrorV2::FoundationScenarioMismatch)
    );
}

#[test]
fn foundation_refuses_nonzero_sessions_and_unbounded_content() {
    let (mut session, session_id, _, _, _, reference_manifest) = fixture();
    let report = first_report(&mut session, &session_id);
    drop(report);
    let bundle =
        FoundationContentBundleV1::try_new(SCENARIO, None, RULE, DEFINES, &reference_manifest)
            .expect("bounded bundle");
    assert_eq!(
        CampaignFoundationV1::capture(&session, bundle),
        Err(RustPersistenceRuntimeErrorV2::FoundationAfterTickZero { actual: 1 })
    );

    let embedded_nul = FoundationContentBundleV1::try_new(
        "(scenario bad\0source)",
        None,
        RULE,
        DEFINES,
        &reference_manifest,
    );
    assert_eq!(
        embedded_nul,
        Err(RustPersistenceRuntimeErrorV2::SemanticCodec)
    );
    let oversized = "x".repeat(65_536);
    assert_eq!(
        FoundationContentBundleV1::try_new(&oversized, None, RULE, DEFINES, &reference_manifest,),
        Err(RustPersistenceRuntimeErrorV2::SemanticCodec)
    );
}

#[test]
fn prepared_tick_owns_full_checkpoint_rows_and_exact_archive_receipt() {
    let (mut session, session_id, seed, content, reference, _) = fixture();
    let report = first_report(&mut session, &session_id);
    let prepared = prepare_committed_tick_v2(&report).expect("single report composes");

    let checkpoint: &CheckpointRowsV1 = prepared.checkpoint_rows();
    assert_eq!(checkpoint.row_count(), 9);
    assert_eq!(
        checkpoint.source_tick(),
        CommittedResolveTickV1::try_from(1).unwrap()
    );
    let archive: &ArchiveDirtyReceiptV1 = prepared.archive_dirty_receipt();
    assert_eq!(archive.tick_content_hash(), report.tick_content_hash());
    assert_eq!(
        archive.row().payload(),
        report.tick_content_hash().as_bytes()
    );

    assert_eq!(report.replay_session_identity(), &session_id);
    assert_eq!(report.rng_seed(), seed);
    assert_eq!(report.content_digest(), &content);
    assert_eq!(report.reference_digest(), reference);
    assert_eq!(
        report.resolver_manifest_bytes(),
        session.resolver_manifest_bytes()
    );
    assert_eq!(
        report.prepared_environment_bytes(),
        session.prepared_environment_bytes()
    );
}

#[test]
fn restart_root_is_one_exact_nine_section_full_checkpoint() {
    let (mut session, session_id, _, _, _, _) = fixture();
    let report = first_report(&mut session, &session_id);
    let campaign = CampaignId::from_uuid(
        Uuid::parse_str("00112233-4455-6677-8899-aabbccddeeff").expect("campaign uuid"),
    );
    let tick = CommittedResolveTickV1::try_from(1).expect("tick one");
    let wrong_tick = CommittedResolveTickV1::try_from(2).expect("tick two");
    assert_eq!(
        CommittedFullCheckpointV1::capture(campaign, wrong_tick, &report),
        Err(RustPersistenceRuntimeErrorV2::ReplaySource)
    );
    let checkpoint =
        CommittedFullCheckpointV1::capture(campaign, tick, &report).expect("full checkpoint");

    assert_eq!(checkpoint.completeness(), CheckpointCompletenessV1::Full);
    assert_eq!(checkpoint.sections().len(), 9);
    assert_eq!(
        checkpoint
            .sections()
            .iter()
            .map(CommittedCheckpointSectionV1::tag)
            .collect::<Vec<_>>(),
        vec![
            FullCheckpointSectionTagV1::StableGraph,
            FullCheckpointSectionTagV1::WorldRegisters,
            FullCheckpointSectionTagV1::ResolverManifest,
            FullCheckpointSectionTagV1::PreparedEnvironment,
            FullCheckpointSectionTagV1::ReplaySessionIdentity,
            FullCheckpointSectionTagV1::RngSeed,
            FullCheckpointSectionTagV1::ContentDigest,
            FullCheckpointSectionTagV1::ReferenceDigest,
            FullCheckpointSectionTagV1::SemanticState,
        ]
    );
    assert_eq!(checkpoint.rows().len(), 9);
    assert!(checkpoint
        .rows()
        .windows(2)
        .all(|pair| pair[0].key() < pair[1].key()));
    let rng_seed_bytes = report.rng_seed().to_be_bytes();
    let content_digest_bytes = [
        report.content_digest().defines_hash.as_slice(),
        report.content_digest().rules_hash.as_slice(),
    ]
    .concat();
    let reference_digest = report.reference_digest();
    let exact_section_bytes = [
        report.result_stable_graph().canonical_bytes(),
        report.result_registers().canonical_bytes(),
        report.resolver_manifest_bytes(),
        report.prepared_environment_bytes(),
        report.replay_session_identity().as_bytes(),
        rng_seed_bytes.as_slice(),
        content_digest_bytes.as_slice(),
        reference_digest.as_bytes(),
        report.material_state_rows().canonical_bytes(),
    ];
    for ((section, row), exact_bytes) in checkpoint
        .sections()
        .iter()
        .zip(checkpoint.rows())
        .zip(exact_section_bytes)
    {
        assert_eq!(section.sha256(), sha256_of(exact_bytes));
        let exact_start = row.payload().len() - exact_bytes.len();
        assert_eq!(row.payload()[exact_start - 5], 1);
        let encoded_len = u32::from_be_bytes(
            row.payload()[exact_start - 4..exact_start]
                .try_into()
                .unwrap(),
        );
        assert_eq!(usize::try_from(encoded_len).unwrap(), exact_bytes.len());
        assert_eq!(&row.payload()[exact_start..], exact_bytes);
        assert_eq!(section.sha256(), sha256_of(&row.payload()[exact_start..]));
    }
    assert_eq!(
        checkpoint.manifest_sha256(),
        sha256_of(checkpoint.manifest_bytes())
    );

    assert_eq!(
        CommittedFullCheckpointV1::validate_restart_root(
            CheckpointCompletenessV1::Full,
            &checkpoint.sections()[..8],
        ),
        Err(RustPersistenceRuntimeErrorV2::SemanticCodec)
    );

    assert_eq!(
        CommittedFullCheckpointV1::validate_restart_root(
            CheckpointCompletenessV1::Delta,
            checkpoint.sections(),
        ),
        Err(RustPersistenceRuntimeErrorV2::DeltaCheckpointNotRestartRoot)
    );

    let source = include_str!("../src/checkpoint.rs");
    assert!(!source.contains("TickZeroMarker"));
    assert!(!source.contains("tick_zero_marker"));
    assert!(!source.contains("compatibility"));
    assert!(!source.contains(" alias "));
}

#[test]
fn checkpoint_rows_are_always_the_exact_nine_required_sections() {
    fn assert_send<T: Send>() {}
    assert_send::<CheckpointRowsV1>();
    assert_send::<ArchiveDirtyReceiptV1>();

    let (mut session, session_id, _, _, _, _) = fixture();
    let report = first_report(&mut session, &session_id);
    let prepared = prepare_committed_tick_v2(&report).expect("report composes");
    assert_eq!(prepared.checkpoint_rows().row_count(), 9);

    let source = include_str!("../src/checkpoint.rs");
    assert!(!source.contains("CheckpointRowsEmptyProofV1"));
}
