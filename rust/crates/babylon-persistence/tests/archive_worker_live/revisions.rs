//! Ordered immutable publication and adoption proofs over real committed ticks.
use super::*;
use babylon_persistence::archive_revision::{
    ArchiveDossierPendingV2, ArchiveDossierUnavailableV2, ArchiveSearchStateV2,
};
use babylon_persistence::{ArchiveKnowledgeV1, ArchiveMaterializeModeV1};

#[path = "../support/legacy_archive.rs"]
mod legacy_archive;

fn stable_input(receipt: &PendingArchiveReceiptV1, question: &str) -> ArchivePageInputV1 {
    let original = stub_page_input(
        &PendingArchiveReceiptV1::try_new(1, *receipt.tick_content_hash()).expect("stub identity"),
    );
    ArchivePageInputV1::try_new(
        original.subject().clone(),
        receipt.resolve_tick(),
        *receipt.tick_content_hash(),
        question.to_owned(),
        original.signals().to_vec(),
        Vec::new(),
    )
    .expect("exact stable subject emission")
}
fn batch_at(target: &LiveWorkerTarget, tick: u64, question: &str) -> ArchiveDirtyBatchV1 {
    let scope = scope_at(&target.config, target.campaign_id, tick);
    let receipt =
        PendingArchiveReceiptV1::try_new(tick, scope.tick_content_hash().expect("committed hash"))
            .expect("receipt");
    ArchiveDirtyBatchV1::try_new(
        tick,
        *receipt.tick_content_hash(),
        vec![stable_input(&receipt, question)],
    )
    .expect("batch")
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_revision_refuses_later_tick_and_conflicting_stage_without_partial_publication() {
    let target = LiveWorkerTarget::create(
        "revisionorder",
        0x2200_0000_0000_0000_0000_0000_0000_00d1,
        2,
    );
    let store = SemanticArchiveStoreV1::new(&target.config);
    let second = batch_at(&target, 2, "A");
    assert_eq!(
        store.materialize_receipt(
            target.campaign_id,
            &second,
            ArchiveMaterializeModeV1::Consume
        ),
        Err(SemanticArchiveErrorV1::ArchiveOrderViolation)
    );
    let wrong =
        ArchiveDirtyBatchV1::try_new(1, [0x71; 32], Vec::new()).expect("well formed wrong hash");
    assert_eq!(
        store.materialize_receipt(
            target.campaign_id,
            &wrong,
            ArchiveMaterializeModeV1::Consume
        ),
        Err(SemanticArchiveErrorV1::ReceiptMismatch)
    );
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 0);
    let pins: i64 = target
        .config
        .connect(NoTls)
        .expect("pin count")
        .query_one(
            "SELECT count(*) FROM babylon_meta.archive_tick_knowledge_v2 WHERE campaign_id=$1",
            &[target.campaign_id.as_uuid()],
        )
        .expect("pin query")
        .get(0);
    assert_eq!(
        pins, 0,
        "refused requests roll back their attempted knowledge pin"
    );
    let first = batch_at(&target, 1, "A");
    store
        .materialize_receipt(target.campaign_id, &first, ArchiveMaterializeModeV1::Stage)
        .expect("stage first page");
    assert_eq!(
        store.materialize_receipt(
            target.campaign_id,
            &batch_at(&target, 1, "B"),
            ArchiveMaterializeModeV1::Stage
        ),
        Err(SemanticArchiveErrorV1::ReceiptConflict)
    );
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 1);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        0
    );
    store
        .materialize_receipt(
            target.campaign_id,
            &first,
            ArchiveMaterializeModeV1::Consume,
        )
        .expect("exact retry consumes");
    store
        .materialize_receipt(
            target.campaign_id,
            &second,
            ArchiveMaterializeModeV1::Consume,
        )
        .expect("later publication now eligible");
    assert_eq!(
        archive_page_count(&target.config, target.campaign_id),
        2,
        "both revisions remain immutable"
    );
    target.finish();
}

fn grant_subject_only(target: &LiveWorkerTarget) {
    SemanticArchiveStoreV1::new(&target.config)
        .grant_knowledge(
            target.campaign_id,
            &ArchiveKnowledgeGrantV1::try_new(
                stub_subject_spec(1).page_ref,
                "subject".to_owned(),
                1,
                ArchiveCitationV1::try_new("late-grant-proof".to_owned(), "subject".to_owned())
                    .expect("citation"),
            )
            .expect("grant"),
        )
        .expect("subject only");
}
fn assert_late_grant_pending(target: &LiveWorkerTarget) {
    with_reader(&target.config, |reader| {
        let scope = scope_at(&target.config, target.campaign_id, 1);
        let read = reader
            .dossier_as_of(
                &scope,
                &stub_subject_spec(1).page_ref,
                &ArchiveDossierBoundsV2::default(),
            )
            .expect("late grant scoped read");
        let ArchiveDossierStateV2::Pending {
            page: Some(page),
            reason: ArchiveDossierPendingV2::KnowledgeRefresh,
        } = read.state
        else {
            panic!("tail knowledge refresh stays pending");
        };
        assert!(page.signals.is_empty());
        assert!(!page.markdown.contains("728576"));
        let search = reader
            .search_as_of(&scope, "728576", 10)
            .expect("late grant search");
        assert_eq!(
            search.state,
            ArchiveSearchStateV2::Pending(ArchiveDossierPendingV2::KnowledgeRefresh)
        );
        assert!(search.hits.is_empty());
    });
}
#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_late_grant_stays_pending_at_tail_and_never_rewrites_old_tick() {
    let target = LiveWorkerTarget::create_with_grants(
        "revisionlategrant",
        0x2200_0000_0000_0000_0000_0000_0000_00d2,
        1,
        &[],
    );
    grant_subject_only(&target);
    let store = SemanticArchiveStoreV1::new(&target.config);
    let first = batch_at(&target, 1, "Which work should be examined?");
    store
        .materialize_receipt(target.campaign_id, &first, ArchiveMaterializeModeV1::Stage)
        .expect("first pin with no field grant");
    store
        .grant_knowledge(
            target.campaign_id,
            &ArchiveKnowledgeGrantV1::try_new(
                stub_subject_spec(1).page_ref,
                "employment".to_owned(),
                1,
                ArchiveCitationV1::try_new("late-grant-proof".to_owned(), "field".to_owned())
                    .expect("citation"),
            )
            .expect("grant"),
        )
        .expect("late field arrives");
    store
        .materialize_receipt(
            target.campaign_id,
            &first,
            ArchiveMaterializeModeV1::Consume,
        )
        .expect("same pinned emission consumes");
    assert_late_grant_pending(&target);
    let mut runtime =
        DurableReplayRuntimeV2::<HypergraphStore>::open(&target.config, target.campaign_id)
            .expect("resume actual runtime");
    let actions = OrderedPracticeActionBatchV1::empty(
        runtime.foundation().replay_session_identity().clone(),
        2,
    )
    .expect("next exact action batch");
    runtime
        .advance_and_commit(&mut CollectingSink::default(), &actions)
        .expect("real next tick");
    drop(runtime);
    store
        .materialize_receipt(
            target.campaign_id,
            &batch_at(&target, 2, "Which work should be examined?"),
            ArchiveMaterializeModeV1::Consume,
        )
        .expect("next eligible receipt admits field");
    with_reader(&target.config, |reader| {
        let subject = stub_subject_spec(1).page_ref;
        let older = reader
            .dossier_as_of(
                &scope_at(&target.config, target.campaign_id, 1),
                &subject,
                &ArchiveDossierBoundsV2::default(),
            )
            .expect("historical pinned observation");
        let ArchiveDossierStateV2::Ready { page: old, .. } = older.state else {
            panic!("old tick does not remain invalidated by later grant");
        };
        assert!(old.signals.is_empty());
        assert!(!old.markdown.contains("728576"));
        let current = reader
            .dossier_as_of(
                &scope_at(&target.config, target.campaign_id, 2),
                &subject,
                &ArchiveDossierBoundsV2::default(),
            )
            .expect("new pinned observation");
        let ArchiveDossierStateV2::Ready { page: new, .. } = current.state else {
            panic!("new tick ready");
        };
        assert_eq!(new.signals.len(), 1);
        assert!(new.markdown.contains("728576"));
        assert_eq!(old.content_source.tick(), 1);
        assert_eq!(new.content_source.tick(), 2);
    });
    target.finish();
}

struct CurrentDesiredProducer {
    question: &'static str,
}
impl ArchiveDossierProducerV1 for CurrentDesiredProducer {
    fn produce(
        &self,
        _campaign: Uuid,
        receipt: &PendingArchiveReceiptV1,
        _known: &ArchiveKnowledgeV1,
        _budget: usize,
    ) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
        Ok(ArchiveProducerOutcomeV1::new(
            ArchiveDirtyBatchV1::try_new(
                receipt.resolve_tick(),
                *receipt.tick_content_hash(),
                vec![stable_input(receipt, self.question)],
            )?,
            0,
        ))
    }
    fn cutover_subjects(
        &self,
        _campaign: Uuid,
        _receipt: &PendingArchiveReceiptV1,
        _known: &ArchiveKnowledgeV1,
    ) -> Result<Vec<ArchivePageRefV1>, SemanticArchiveErrorV1> {
        Ok(vec![stub_subject_spec(1).page_ref])
    }
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_adoption_revalidates_stale_head_even_when_old_consumed_prefix_equals_tail() {
    let target = LiveWorkerTarget::create(
        "revisionstalecutover",
        0x2200_0000_0000_0000_0000_0000_0000_00d3,
        3,
    );
    let store = SemanticArchiveStoreV1::new(&target.config);
    for (tick, question) in [(1, "Desired A"), (2, "Staged B")] {
        store
            .materialize_receipt(
                target.campaign_id,
                &batch_at(&target, tick, question),
                ArchiveMaterializeModeV1::Consume,
            )
            .expect("real source identity and rendered old page");
    }
    let scope = scope_at(&target.config, target.campaign_id, 3);
    store
        .materialize_receipt(
            target.campaign_id,
            &ArchiveDirtyBatchV1::try_new(
                3,
                scope.tick_content_hash().expect("marker"),
                Vec::new(),
            )
            .expect("old quiet receipt"),
            ArchiveMaterializeModeV1::Consume,
        )
        .expect("construct old quiet consumption pattern");
    legacy_archive::restore_legacy_heads(&target.config);
    store
        .install_schema()
        .expect("adopt exact stale retained B bytes");
    assert_pending_adopted_question(&target, &scope, "Staged B");
    let report = ArchiveWorkerV1::new(&target.config)
        .sweep_once(
            target.campaign_id,
            &CurrentDesiredProducer {
                question: "Desired A",
            },
        )
        .expect("existing producer validates actual desired tail");
    assert!(report.retention_ready());
    assert!(report.dispositions().is_empty());
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        3
    );
    with_reader(&target.config, |reader| {
        let read = reader
            .dossier_as_of(
                &scope,
                &stub_subject_spec(1).page_ref,
                &ArchiveDossierBoundsV2::default(),
            )
            .expect("validated replacement");
        let ArchiveDossierStateV2::Ready { page, .. } = read.state else {
            panic!("validated ready");
        };
        assert_eq!(page.question, "Desired A");
        assert_eq!(page.content_source.tick(), 3);
    });
    assert_cutover_corruption_refuses(&target, &scope);
    target.finish();
}
fn assert_pending_adopted_question(
    target: &LiveWorkerTarget,
    scope: &babylon_persistence::archive_revision::ArchiveReadScopeV2,
    question: &str,
) {
    with_reader(&target.config, |reader| {
        let subject = stub_subject_spec(1).page_ref;
        let read = reader
            .dossier_as_of(scope, &subject, &ArchiveDossierBoundsV2::default())
            .expect("adopted pending read");
        assert_eq!(read.processed_tick, 3);
        let ArchiveDossierStateV2::Pending {
            page: Some(page),
            reason: ArchiveDossierPendingV2::CutoverValidation,
        } = read.state
        else {
            panic!("consumption alone does not verify composition");
        };
        assert_eq!(page.question, question);
        assert_eq!(page.content_source.tick(), 2);
        let older = reader
            .dossier_as_of(
                &scope_at(&target.config, target.campaign_id, 2),
                &subject,
                &ArchiveDossierBoundsV2::default(),
            )
            .expect("earlier retained coverage refusal");
        assert_eq!(
            older.state,
            ArchiveDossierStateV2::Unavailable(ArchiveDossierUnavailableV2::HistoryNotRetained)
        );
    });
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_opaque_adoption_preserves_original_bytes_privately_until_typed_validation() {
    let target = LiveWorkerTarget::create(
        "revisionopaque",
        0x2200_0000_0000_0000_0000_0000_0000_00d4,
        1,
    );
    let store = SemanticArchiveStoreV1::new(&target.config);
    let original = batch_at(
        &target,
        1,
        "Which work?\n## Related\nA lawful question with section delimiters",
    );
    let emitted = store
        .materialize_receipt(
            target.campaign_id,
            &original,
            ArchiveMaterializeModeV1::Consume,
        )
        .expect("actual V1 renderer accepts the lawful question");
    let bytes = emitted.pages()[0].page().markdown().to_owned();
    legacy_archive::restore_legacy_heads(&target.config);
    store
        .install_schema()
        .expect("opaque adoption keeps original evidence");
    let scope = scope_at(&target.config, target.campaign_id, 1);
    with_reader(&target.config, |reader| {
        let read = reader
            .dossier_as_of(
                &scope,
                &stub_subject_spec(1).page_ref,
                &ArchiveDossierBoundsV2::default(),
            )
            .expect("honest pending opaque head");
        assert!(matches!(
            read.state,
            ArchiveDossierStateV2::Pending {
                page: None,
                reason: ArchiveDossierPendingV2::EmissionWitnessRequired
            }
        ));
        let search = reader
            .search_as_of(&scope, "lawful question", 10)
            .expect("opaque search bounded safely");
        assert!(search.hits.is_empty());
        assert_eq!(
            search.state,
            ArchiveSearchStateV2::Pending(ArchiveDossierPendingV2::EmissionWitnessRequired)
        );
    });
    let retained:String=target.config.connect(NoTls).expect("private retention inspection").query_one(
        "SELECT markdown FROM babylon_meta.archive_page_revision_v2 WHERE campaign_id=$1 AND origin=0",
        &[target.campaign_id.as_uuid()]).expect("original adopted bytes").get(0);
    assert_eq!(retained, bytes);
    let report = ArchiveWorkerV1::new(&target.config)
        .sweep_once(
            target.campaign_id,
            &CurrentDesiredProducer {
                question: "Which work should be examined?",
            },
        )
        .expect("existing typed producer validates cutover");
    assert!(report.retention_ready());
    with_reader(&target.config, |reader| {
        let read = reader
            .dossier_as_of(
                &scope,
                &stub_subject_spec(1).page_ref,
                &ArchiveDossierBoundsV2::default(),
            )
            .expect("new witnessed publication");
        let ArchiveDossierStateV2::Ready { page, .. } = read.state else {
            panic!("typed validated page ready");
        };
        assert_eq!(page.question, "Which work should be examined?");
        assert!(!page.markdown.contains("lawful question"));
    });
    let retained_after:String=target.config.connect(NoTls).expect("private retained evidence").query_one(
        "SELECT markdown FROM babylon_meta.archive_page_revision_v2 WHERE campaign_id=$1 AND origin=0",
        &[target.campaign_id.as_uuid()]).expect("immutable original adopted bytes").get(0);
    assert_eq!(
        retained_after, bytes,
        "validation never discards or rewrites opaque evidence"
    );
    target.finish();
}

fn wait_for_install_table_lock(config: &Config, application: &str) {
    let mut client = config.connect(NoTls).expect("lock census connection");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let waiting:bool=client.query_one(
            "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_locks locks JOIN pg_catalog.pg_stat_activity activity ON locks.pid=activity.pid \
             WHERE activity.application_name=$1 AND NOT locks.granted AND locks.locktype='relation')", &[&application]
        ).expect("exact installer lock census").get(0);
        if waiting {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "installer reaches the writer-excluding table lock"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_adoption_snapshot_includes_writer_commit_that_precedes_its_table_lock() {
    let target =
        LiveWorkerTarget::create("revisionmvcc", 0x2200_0000_0000_0000_0000_0000_0000_00d5, 1);
    let store = SemanticArchiveStoreV1::new(&target.config);
    store
        .materialize_receipt(
            target.campaign_id,
            &batch_at(&target, 1, "Original witnessed page"),
            ArchiveMaterializeModeV1::Consume,
        )
        .expect("actual source publication");
    legacy_archive::restore_legacy_heads(&target.config);
    let mut writer = target
        .config
        .connect(NoTls)
        .expect("legacy writer connection");
    let mut tx = writer.transaction().expect("legacy writer transaction");
    tx.execute("UPDATE babylon_meta.archive_page_v1 SET title='Committed invalid head' WHERE campaign_id=$1",&[target.campaign_id.as_uuid()]).expect("legacy writer holds row and relation lock");
    let application = format!(
        "archive_cutover_{}",
        format_args!(
            "{}_{:x}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos()
        )
    );
    let mut installer = target.config.clone();
    installer.application_name(&application);
    let handle =
        std::thread::spawn(move || SemanticArchiveStoreV1::new(&installer).install_schema());
    wait_for_install_table_lock(&target.config, &application);
    tx.commit()
        .expect("legacy writer commits before installer obtains table lock");
    assert_eq!(handle.join().expect("installer returns"),Err(SemanticArchiveErrorV1::StoredPageMismatch),
        "adoption must see the latest committed head corruption; an earlier MVCC snapshot would falsely accept the old bytes");
    let marker_present: bool = target
        .config
        .connect(NoTls)
        .expect("failed cutover check")
        .query_one(
            "SELECT pg_catalog.to_regclass('babylon_meta.archive_revision_schema_v2') IS NOT NULL",
            &[],
        )
        .expect("marker check")
        .get(0);
    assert!(
        !marker_present,
        "a refused adoption leaves no successor marker"
    );
    target.finish();
}

fn assert_cutover_corruption_refuses(
    target: &LiveWorkerTarget,
    scope: &babylon_persistence::archive_revision::ArchiveReadScopeV2,
) {
    for (table, column) in [
        ("archive_retention_seal_v2", "worker_contract_sha256"),
        ("archive_retention_seal_v2", "knowledge_sha256"),
        ("archive_retention_seal_v2", "composition_sha256"),
        ("archive_tick_knowledge_v2", "worker_contract_sha256"),
        ("archive_tick_knowledge_v2", "knowledge_sha256"),
    ] {
        let mut admin = target
            .config
            .connect(NoTls)
            .expect("bounded seal corruption connection");
        let original: Vec<u8> = admin
            .query_one(
                &format!("SELECT {column} FROM babylon_meta.{table} WHERE campaign_id=$1"),
                &[target.campaign_id.as_uuid()],
            )
            .expect("original exact header value")
            .get(0);
        admin
            .execute(
                &format!("UPDATE babylon_meta.{table} SET {column}=$2 WHERE campaign_id=$1"),
                &[target.campaign_id.as_uuid(), &&[0x73_u8; 32][..]],
            )
            .expect("mutate one exact seal or pin component");
        with_reader(&target.config, |reader| {
            let error = babylon_persistence::SemanticArchiveReaderErrorV1::Archive(
                SemanticArchiveErrorV1::StoredPageMismatch,
            );
            assert_eq!(
                reader.dossier_as_of(
                    scope,
                    &stub_subject_spec(1).page_ref,
                    &ArchiveDossierBoundsV2::default()
                ),
                Err(error.clone()),
                "{table}.{column} cannot certify Ready"
            );
            assert_eq!(
                reader.search_as_of(scope, "Desired A", 10),
                Err(error),
                "corrupt cutover cannot certify search"
            );
        });
        assert_eq!(
            ArchiveWorkerV1::new(&target.config).sweep_once(
                target.campaign_id,
                &CurrentDesiredProducer {
                    question: "Desired A"
                }
            ),
            Err(SemanticArchiveErrorV1::StoredPageMismatch),
            "corrupt sealed work must not silently reenter cutover"
        );
        admin
            .execute(
                &format!("UPDATE babylon_meta.{table} SET {column}=$2 WHERE campaign_id=$1"),
                &[target.campaign_id.as_uuid(), &original],
            )
            .expect("restore exact original component for the independent next case");
    }
}
