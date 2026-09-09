//! Compile-time RED surface for the sole production Rust persistence runtime.

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::stable_state::StableGraphStateV1;
use babylon_kernel::content_digest::ContentDigest;
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
use babylon_kernel::tick_content_hash::{RefDigestV1, TickContentHashV1};
use babylon_persistence::{
    activate_rust_persistence_v2, hydrate_campaign_foundation_v1, prepare_committed_tick_v2,
    ActivationReportV2, ArchiveDirtyReceiptV1, BreadcrumbRowV1, CampaignCatalogRowV1,
    CampaignFoundationV1, CampaignId, CheckpointCompletenessV1, CheckpointRowsV1,
    CommittedCheckpointSectionV1, CommittedFullCheckpointV1, CommittedResolveTickErrorV1,
    CommittedResolveTickV1, CommittedTickAuthorityLedgerRowV2, CommittedTickAuthorityStateV2,
    CommittedTickReceiptV2, DurableReplayRuntimeV2, FoundationContentBundle,
    FoundationContentBundleV1, FullCheckpointSectionTagV1, JumplistRowV1, PreparedCommittedTickV2,
    RetainedMetadataStoreV1, RustPersistenceActivationErrorV2, RustPersistenceRuntimeErrorV2,
    StableGraphRowsEmptyProofV1, SuccessfulEventBatchEmptyProofV2, WatchlistRowV1,
};
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use babylon_tick::material_state::MaterialStateRowsV1;
use babylon_tick::replay_session::{
    IdentifiedTickReportV2, ReplayCommitDispositionV1, ReplayTickSession, SuccessfulEventBatchV2,
};
use postgres::Config;

fn assert_send<T: Send>() {}

fn assert_prepare_signature(
    value: fn(
        &IdentifiedTickReportV2,
    ) -> Result<PreparedCommittedTickV2, RustPersistenceRuntimeErrorV2>,
) {
    let _ = value;
}

fn assert_activation_signature(
    value: fn(&Config) -> Result<ActivationReportV2, RustPersistenceActivationErrorV2>,
) {
    let _ = value;
}

fn assert_foundation_hydration_signature(
    value: fn(&Config, CampaignId) -> Result<CampaignFoundationV1, RustPersistenceRuntimeErrorV2>,
) {
    let _ = value;
}

type ContentBundleConstructor =
    fn(
        &str,
        Option<&str>,
        &str,
        &[u8],
        &[u8],
    ) -> Result<FoundationContentBundleV1, RustPersistenceRuntimeErrorV2>;

fn assert_content_bundle_constructor(value: ContentBundleConstructor) {
    let _ = value;
}

fn assert_foundation_capture_signature(
    value: fn(
        &ReplayTickSession<HypergraphStore>,
        FoundationContentBundleV1,
    ) -> Result<CampaignFoundationV1, RustPersistenceRuntimeErrorV2>,
) {
    let _ = value;
}

type ProductionRuntime = DurableReplayRuntimeV2<HypergraphStore>;

fn assert_runtime_create_signature(
    value: fn(
        &Config,
        CampaignId,
        ReplayTickSession<HypergraphStore>,
        FoundationContentBundleV1,
    ) -> Result<ProductionRuntime, RustPersistenceRuntimeErrorV2>,
) {
    let _ = value;
}

fn assert_runtime_open_signature(
    value: fn(&Config, CampaignId) -> Result<ProductionRuntime, RustPersistenceRuntimeErrorV2>,
) {
    let _ = value;
}

fn assert_runtime_advance_signature(
    value: fn(
        &mut ProductionRuntime,
        &mut CollectingSink,
        &OrderedPracticeActionBatchV1,
    ) -> Result<CommittedTickReceiptV2, RustPersistenceRuntimeErrorV2>,
) {
    let _ = value;
}

#[test]
fn cutover_exports_one_typed_runtime_and_prepared_tick_boundary() {
    assert_send::<DurableReplayRuntimeV2<HypergraphStore>>();
    assert_send::<PreparedCommittedTickV2>();
    assert_send::<ActivationReportV2>();
    assert_send::<CampaignFoundationV1>();
    assert_send::<CommittedFullCheckpointV1>();
    assert_send::<RustPersistenceActivationErrorV2>();
    assert_send::<RustPersistenceRuntimeErrorV2>();
    assert_send::<StableGraphRowsEmptyProofV1>();
    assert_send::<SuccessfulEventBatchEmptyProofV2>();
    assert_send::<ArchiveDirtyReceiptV1>();

    assert_activation_signature(activate_rust_persistence_v2);
    assert_foundation_hydration_signature(hydrate_campaign_foundation_v1);
    assert_prepare_signature(prepare_committed_tick_v2);
    assert_content_bundle_constructor(FoundationContentBundleV1::try_new);
    assert_foundation_capture_signature(CampaignFoundationV1::capture);
    assert_runtime_create_signature(ProductionRuntime::create);
    assert_runtime_open_signature(ProductionRuntime::open);
    assert_runtime_advance_signature(ProductionRuntime::advance_and_commit);

    let checkpoint_source = include_str!("../src/checkpoint.rs");
    let export_source = include_str!("../src/lib.rs");
    assert!(!checkpoint_source.contains("CheckpointRowsEmptyProofV1"));
    assert!(!export_source.contains("CheckpointRowsEmptyProofV1"));
}

#[test]
fn activation_ledger_and_runtime_receipts_are_typed_and_observable() {
    fn assert_ledger_row(row: &CommittedTickAuthorityLedgerRowV2) {
        let _: u16 = row.ordinal();
        let _: CommittedTickAuthorityStateV2 = row.state();
        let _: u16 = row.activation_epoch();
        let _: [u8; 32] = row.contract_sha256();
        let _: [u8; 32] = row.reader_contract_sha256();
        let _: [u8; 32] = row.predecessor_sha256();
    }

    fn assert_activation_report(report: &ActivationReportV2) {
        let _: &CommittedTickAuthorityLedgerRowV2 = report.prepared_row();
        let _: &CommittedTickAuthorityLedgerRowV2 = report.active_row();
    }

    fn assert_runtime(runtime: &ProductionRuntime) {
        let _: &CampaignFoundationV1 = runtime.foundation();
        let _: &CommittedTickAuthorityLedgerRowV2 = runtime.activation_row();
        let _: Option<CommittedResolveTickV1> = runtime.last_committed_tick();
    }

    fn assert_receipt(receipt: &CommittedTickReceiptV2) {
        let _: CommittedResolveTickV1 = receipt.resolve_tick();
        let _: ReplayCommitDispositionV1 = receipt.commit_disposition();
        let _: [u8; 32] = receipt.graph_before();
        let _: [u8; 32] = receipt.graph_after();
        let _: [u8; 32] = receipt.world_before();
        let _: [u8; 32] = receipt.world_after();
        let _: usize = receipt.considered();
        let _: usize = receipt.fired();
        let _: &[(String, usize)] = receipt.per_rule_considered();
        let _: &[(String, usize)] = receipt.per_rule_fired();
        let _: usize = receipt.event_count();
        let _: [u8; 32] = receipt.event_digest();
        let _: usize = receipt.audit_receipt_count();
        let _: usize = receipt.material_row_count();
        let _: [u8; 32] = receipt.material_row_digest();
        let _: TickContentHashV1 = receipt.tick_content_hash();
    }

    let _: [CommittedTickAuthorityStateV2; 2] = [
        CommittedTickAuthorityStateV2::Prepared,
        CommittedTickAuthorityStateV2::Active,
    ];
    let _ = assert_ledger_row;
    let _ = assert_activation_report;
    let _ = assert_runtime;
    let _ = assert_receipt;
}

#[test]
fn only_an_identified_replay_report_can_prepare_semantic_rows() {
    assert_prepare_signature(prepare_committed_tick_v2);

    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime.rs"),
    )
    .expect("sole production persistence composition module");
    assert!(source.contains("IdentifiedTickReportV2"));
    assert!(source.contains("prepare_committed_tick_v2"));
    assert!(!source.contains("prepare_rules("));
    assert!(!source.contains("run_prepared_replay_tick("));
}

#[test]
fn campaign_foundation_retains_every_exact_replay_source() {
    fn assert_foundation_accessors(foundation: &CampaignFoundationV1) {
        let FoundationContentBundle::V1(content): &FoundationContentBundle =
            foundation.content_bundle()
        else {
            panic!("this retained accessor fixture uses explicit V1 content")
        };
        let content: &FoundationContentBundleV1 = content;
        let _: &[u8] = content.scenario_source_bytes();
        let _: Option<&[u8]> = content.prelude_source_bytes();
        let _: &[u8] = content.rule_source_bytes();
        let _: &[u8] = content.defines_bytes();
        let _: &[u8] = content.reference_bundle_manifest_bytes();
        let _: &[u8] = foundation.stable_graph_bytes();
        let _: &[u8] = foundation.world_register_bytes();
        let _: &[u8] = foundation.resolver_manifest_bytes();
        let _: &[u8] = foundation.prepared_environment_bytes();
        let _: &ReplaySessionIdV1 = foundation.replay_session_identity();
        let _: ReplaySeed = foundation.rng_seed();
        let _: &ContentDigest = foundation.content_digest();
        let _: RefDigestV1 = foundation.reference_digest();
    }

    let _ = assert_foundation_accessors;
}

#[test]
fn durable_tick_numbers_refuse_a_synthetic_tick_zero() {
    assert_eq!(
        CommittedResolveTickV1::try_from(0_u64),
        Err(CommittedResolveTickErrorV1::SyntheticTickZero)
    );
    assert_eq!(
        CommittedResolveTickV1::try_from(1_u64)
            .expect("the first real executed tick")
            .get(),
        1
    );
    assert_eq!(
        CommittedResolveTickV1::try_from((i64::MAX as u64) + 1),
        Err(CommittedResolveTickErrorV1::OutOfPostgresRange)
    );
}

#[test]
fn reports_and_prepared_ticks_expose_every_named_semantic_producer() {
    fn assert_report_sources(report: &IdentifiedTickReportV2) {
        let _: &StableGraphStateV1 = report.result_stable_graph();
        let _: &MaterialStateRowsV1 = report.material_state_rows();
        let _: &SuccessfulEventBatchV2 = report.successful_event_batch();
    }

    fn assert_prepared_sources(prepared: &PreparedCommittedTickV2) {
        let _: &CheckpointRowsV1 = prepared.checkpoint_rows();
        let _: &ArchiveDirtyReceiptV1 = prepared.archive_dirty_receipt();
    }

    let _ = assert_report_sources;
    let _ = assert_prepared_sources;
}

#[test]
fn restart_roots_are_explicitly_full_checkpoints() {
    fn assert_full_checkpoint(checkpoint: &CommittedFullCheckpointV1) {
        assert_eq!(checkpoint.completeness(), CheckpointCompletenessV1::Full);
        let _: &[u8] = checkpoint.manifest_bytes();
        let _: &[CommittedCheckpointSectionV1] = checkpoint.sections();
    }

    fn assert_section(section: &CommittedCheckpointSectionV1) {
        let _: FullCheckpointSectionTagV1 = section.tag();
        let _: u32 = section.row_count();
        let _: [u8; 32] = section.sha256();
    }

    let _: [FullCheckpointSectionTagV1; 9] = [
        FullCheckpointSectionTagV1::StableGraph,
        FullCheckpointSectionTagV1::WorldRegisters,
        FullCheckpointSectionTagV1::ResolverManifest,
        FullCheckpointSectionTagV1::PreparedEnvironment,
        FullCheckpointSectionTagV1::ReplaySessionIdentity,
        FullCheckpointSectionTagV1::RngSeed,
        FullCheckpointSectionTagV1::ContentDigest,
        FullCheckpointSectionTagV1::ReferenceDigest,
        FullCheckpointSectionTagV1::SemanticState,
    ];
    let _ = assert_full_checkpoint;
    let _ = assert_section;
}

#[test]
fn production_no_longer_hydrates_or_commits_a_tick_zero_marker() {
    let writer = include_str!("../src/runtime.rs");
    let old_writer =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/committed_tick_writer.rs");
    let old_contract = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/committed_tick_writer_v1_contract.rs");
    let activation = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("migrations/0009_rust_persistence_activation.sql"),
    )
    .expect("epoch-9 activation migration");

    assert!(writer.contains("babylon_state.campaign_foundation"));
    assert!(!writer.contains("marker.resolve_tick = 0"));
    assert!(!writer.contains("committed tick-zero"));
    assert!(!old_writer.exists());
    assert!(!old_contract.exists());
    assert!(activation.contains("CHECK (resolve_tick >= 1)"));
}

#[test]
fn epoch_nine_locks_each_present_legacy_table_before_empty_inventory_and_drop() {
    let activation = include_str!("../migrations/0009_rust_persistence_activation.sql");
    let lock = activation
        .find("'LOCK TABLE %s IN ACCESS EXCLUSIVE MODE'")
        .expect("epoch 9 takes a dynamic access-exclusive table lock");
    let count = activation
        .find("'SELECT pg_catalog.count(*) FROM %s'")
        .expect("epoch 9 inventories the locked relation");
    let disposition = activation
        .find("INSERT INTO babylon_meta.python_relation_disposition_v1")
        .expect("epoch 9 records the empty locked relation");
    let drop_tables = activation
        .find("DROP TABLE IF EXISTS\n    public.action_result")
        .expect("epoch 9 drops the inventoried legacy relations");

    assert_eq!(
        activation
            .matches("'LOCK TABLE %s IN ACCESS EXCLUSIVE MODE'")
            .count(),
        1
    );
    assert!(lock < count);
    assert!(count < disposition);
    assert!(disposition < drop_tables);

    let opaque_lock = activation
        .find("LOCK TABLE\n    babylon_state.tick_graph_row")
        .expect("epoch 9 locks the closed opaque predecessor set");
    let opaque_count = activation
        .find("DO $rust_persistence_opaque_preflight$")
        .expect("epoch 9 inventories the locked opaque predecessor set");
    let opaque_drop = activation
        .find("DROP TABLE babylon_state.tick_archive_dirty_receipt_row")
        .expect("epoch 9 drops the inventoried opaque predecessor set");
    assert!(opaque_lock < opaque_count);
    assert!(opaque_count < opaque_drop);
}

#[test]
fn typed_tick_retry_rechecks_the_exact_envelope_after_the_campaign_lock() {
    let writer = include_str!("../src/runtime.rs");
    let commit = writer
        .split_once("fn commit_typed_tick_v2(")
        .expect("typed tick writer exists")
        .1
        .split_once("fn commit_marker_last_v2(")
        .expect("typed tick writer has a bounded body")
        .0;
    let lock = commit
        .find("FOR UPDATE")
        .expect("campaign row is locked before mutation");
    let locked_retry = commit[lock..]
        .find("marker_matches_envelope_v2(&mut transaction, report, envelope)")
        .map(|offset| lock + offset)
        .expect("exact retry is reconciled under the campaign lock");
    let tail = commit
        .find("read campaign marker tail")
        .expect("ordinary predecessor validation remains present");

    assert!(lock < locked_retry);
    assert!(locked_retry < tail);
}

#[test]
fn production_activation_checks_exact_pg17_before_the_inventory_transaction() {
    let writer = include_str!("../src/runtime.rs");
    let preflight = writer
        .split_once("fn preflight_v2_activation_before_mutation(")
        .expect("production activation preflight exists")
        .1
        .split_once("fn require_postgresql_server_major_v2(")
        .expect("server-major helper follows the preflight")
        .0;
    let connect = preflight
        .find("connect for pre-activation inventory")
        .expect("preflight opens its read-only connection");
    let version = preflight
        .find("require_postgresql_server_major_v2(&mut client)")
        .expect("preflight enforces the exact server major");
    let transaction = preflight
        .find("begin pre-activation inventory")
        .expect("inventory transaction begins after version validation");

    assert!(connect < version);
    assert!(version < transaction);
    assert!(writer.contains("const REQUIRED_POSTGRESQL_SERVER_MAJOR_V2: u32 = 17;"));
    assert!(writer.contains("PostgreSqlServerMajorMismatch"));
}

#[test]
fn durable_commit_uses_preflighted_infallible_session_publication() {
    let runtime = include_str!("../src/runtime.rs");
    let advance = runtime
        .split_once("pub fn advance_and_commit(")
        .expect("durable advance exists")
        .1
        .split_once("pub fn observe_committed_graph_state_v1(")
        .expect("durable advance has a bounded body")
        .0;
    assert!(advance.contains(".commit_prepared_and_publish(sink, candidate"));
    assert!(!advance.contains(".acknowledge_prepared("));

    let replay = include_str!("../../babylon-tick/src/replay_session.rs");
    let publish = replay
        .split_once("fn commit_prepared_and_publish_with_allocation")
        .expect("preflighted commit helper exists")
        .1
        .split_once("fn validate_prepared_publication")
        .expect("preflighted commit helper has a bounded body")
        .0;
    let validation = publish
        .find("validate_prepared_publication")
        .expect("candidate validation runs first");
    let reservation = publish
        .find(".reserve(sink, prepared.events.len())")
        .expect("sink capacity is reserved before commit");
    let commit = publish
        .find("let disposition = commit(&prepared.report)")
        .expect("caller-owned durable commit runs after preflight");
    let infallible_publish = publish
        .find("publish_prepared_infallibly")
        .expect("successful commit is followed by infallible publication");

    assert!(validation < reservation);
    assert!(reservation < commit);
    assert!(commit < infallible_publish);
    assert!(!publish[infallible_publish..].contains('?'));
}

#[test]
fn the_separate_schema_epoch_binary_has_been_absorbed() {
    let old_binary =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/bin/babylon-schema-epoch.rs");
    assert!(
        !old_binary.exists(),
        "the production runtime must own the sole migration command"
    );
}

#[test]
fn retained_client_metadata_has_rust_types_and_is_not_dropped_at_activation() {
    fn assert_metadata_store(store: &RetainedMetadataStoreV1, campaign_id: CampaignId) {
        let _: Result<Option<CampaignCatalogRowV1>, RustPersistenceRuntimeErrorV2> =
            store.campaign(campaign_id);
        let _: Result<Vec<WatchlistRowV1>, RustPersistenceRuntimeErrorV2> =
            store.watchlist(campaign_id);
        let _: Result<Vec<JumplistRowV1>, RustPersistenceRuntimeErrorV2> =
            store.jumplist(campaign_id);
        let _: Result<Vec<BreadcrumbRowV1>, RustPersistenceRuntimeErrorV2> =
            store.breadcrumbs(campaign_id);
    }

    assert_send::<CampaignCatalogRowV1>();
    assert_send::<WatchlistRowV1>();
    assert_send::<JumplistRowV1>();
    assert_send::<BreadcrumbRowV1>();
    assert_send::<RetainedMetadataStoreV1>();

    let _ = assert_metadata_store;

    let activation = include_str!("../migrations/0009_rust_persistence_activation.sql");
    let drop_block = activation
        .split_once("DROP TABLE IF EXISTS")
        .expect("activation has one retired-table block")
        .1;
    for retained in [
        "babylon_meta.campaign",
        "babylon_meta.watchlist",
        "babylon_meta.jumplist",
        "babylon_meta.breadcrumb",
    ] {
        assert!(
            !drop_block.contains(retained),
            "{retained} must survive cutover"
        );
    }
    for retained in ["campaign", "watchlist", "jumplist", "breadcrumb"] {
        assert!(
            activation.contains(&format!(
                "CREATE TABLE IF NOT EXISTS babylon_meta.{retained}"
            )),
            "fresh Rust activation must create babylon_meta.{retained}"
        );
    }
    assert!(activation.contains("ALTER TABLE babylon_meta.campaign"));
    assert!(activation.contains("ADD COLUMN IF NOT EXISTS rng_seed BIGINT"));
    assert!(activation.contains("ADD COLUMN IF NOT EXISTS content_digest TEXT"));
    assert!(activation.contains("ALTER COLUMN last_tick TYPE BIGINT"));
}
