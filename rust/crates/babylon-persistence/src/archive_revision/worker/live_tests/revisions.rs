//! Ordered immutable publication proofs over real committed ticks.
use super::*;
use crate::archive_revision::{ArchiveDossierPending, ArchiveSearchState};
use crate::ArchiveMaterializeMode;

fn stable_input(receipt: &PendingArchiveReceipt, question: &str) -> ArchivePageInput {
    let original = stub_page_input(
        &PendingArchiveReceipt::try_new(1, *receipt.tick_content_hash()).expect("stub identity"),
    );
    ArchivePageInput::try_new(
        original.subject().clone(),
        receipt.resolve_tick(),
        *receipt.tick_content_hash(),
        question.to_owned(),
        original.signals().to_vec(),
        Vec::new(),
    )
    .expect("exact stable subject emission")
}
fn batch_at(target: &LiveWorkerTarget, tick: u64, question: &str) -> ArchiveDirtyBatch {
    let scope = scope_at(&target.config, target.campaign_id, tick);
    let receipt =
        PendingArchiveReceipt::try_new(tick, scope.tick_content_hash().expect("committed hash"))
            .expect("receipt");
    ArchiveDirtyBatch::try_new(
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
    let store = SemanticArchiveStore::new(&target.config);
    let second = batch_at(&target, 2, "A");
    assert_eq!(
        store.materialize_receipt(target.campaign_id, &second, ArchiveMaterializeMode::Consume),
        Err(SemanticArchiveError::ArchiveOrderViolation)
    );
    let wrong =
        ArchiveDirtyBatch::try_new(1, [0x71; 32], Vec::new()).expect("well formed wrong hash");
    assert_eq!(
        store.materialize_receipt(target.campaign_id, &wrong, ArchiveMaterializeMode::Consume),
        Err(SemanticArchiveError::ReceiptMismatch)
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
        .materialize_receipt(target.campaign_id, &first, ArchiveMaterializeMode::Stage)
        .expect("stage first page");
    assert_eq!(
        store.materialize_receipt(
            target.campaign_id,
            &batch_at(&target, 1, "B"),
            ArchiveMaterializeMode::Stage
        ),
        Err(SemanticArchiveError::ReceiptConflict)
    );
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 1);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        0
    );
    store
        .materialize_receipt(target.campaign_id, &first, ArchiveMaterializeMode::Consume)
        .expect("exact retry consumes");
    store
        .materialize_receipt(target.campaign_id, &second, ArchiveMaterializeMode::Consume)
        .expect("later publication now eligible");
    assert_eq!(
        archive_page_count(&target.config, target.campaign_id),
        2,
        "both revisions remain immutable"
    );
    target.finish();
}

fn grant_subject_only(target: &LiveWorkerTarget) {
    SemanticArchiveStore::new(&target.config)
        .grant_knowledge(
            target.campaign_id,
            &ArchiveKnowledgeGrant::try_new(
                stub_subject_spec(1).page_ref,
                "subject".to_owned(),
                1,
                ArchiveCitation::try_new("late-grant-proof".to_owned(), "subject".to_owned())
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
                &ArchiveDossierBounds::default(),
            )
            .expect("late grant scoped read");
        let ArchiveDossierState::Pending {
            page: Some(page),
            reason: ArchiveDossierPending::KnowledgeRefresh,
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
            ArchiveSearchState::Pending(ArchiveDossierPending::KnowledgeRefresh)
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
    let store = SemanticArchiveStore::new(&target.config);
    let first = batch_at(&target, 1, "Which work should be examined?");
    store
        .materialize_receipt(target.campaign_id, &first, ArchiveMaterializeMode::Stage)
        .expect("first pin with no field grant");
    store
        .grant_knowledge(
            target.campaign_id,
            &ArchiveKnowledgeGrant::try_new(
                stub_subject_spec(1).page_ref,
                "employment".to_owned(),
                1,
                ArchiveCitation::try_new("late-grant-proof".to_owned(), "field".to_owned())
                    .expect("citation"),
            )
            .expect("grant"),
        )
        .expect("late field arrives");
    store
        .materialize_receipt(target.campaign_id, &first, ArchiveMaterializeMode::Consume)
        .expect("same pinned emission consumes");
    assert_late_grant_pending(&target);
    let mut runtime = DurableMaterialRuntime::open(
        &target.config,
        target.campaign_id,
        current_material::foundation().digest(),
    )
    .expect("resume actual runtime");
    let actions = OrderedPracticeActionBatch::empty(
        runtime.session().graph_session().session_identity().clone(),
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
            ArchiveMaterializeMode::Consume,
        )
        .expect("next eligible receipt admits field");
    with_reader(&target.config, |reader| {
        let subject = stub_subject_spec(1).page_ref;
        let older = reader
            .dossier_as_of(
                &scope_at(&target.config, target.campaign_id, 1),
                &subject,
                &ArchiveDossierBounds::default(),
            )
            .expect("historical pinned observation");
        let ArchiveDossierState::Ready { page: old, .. } = older.state else {
            panic!("old tick does not remain invalidated by later grant");
        };
        assert!(old.signals.is_empty());
        assert!(!old.markdown.contains("728576"));
        let current = reader
            .dossier_as_of(
                &scope_at(&target.config, target.campaign_id, 2),
                &subject,
                &ArchiveDossierBounds::default(),
            )
            .expect("new pinned observation");
        let ArchiveDossierState::Ready { page: new, .. } = current.state else {
            panic!("new tick ready");
        };
        assert_eq!(new.signals.len(), 1);
        assert!(new.markdown.contains("728576"));
        assert_eq!(old.content_source.tick(), 1);
        assert_eq!(new.content_source.tick(), 2);
    });
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_archive_verifier_refuses_missing_current_schema_without_repair() {
    let target = LiveWorkerTarget::create(
        "revisionmissing",
        0x2200_0000_0000_0000_0000_0000_0000_00d3,
        1,
    );
    let scope = scope_at(&target.config, target.campaign_id, 1);
    let mut admin = target
        .config
        .connect(NoTls)
        .expect("owned incomplete-schema fixture");
    admin
        .batch_execute("DROP TABLE babylon_meta.current_schema")
        .expect("remove only the owned scratch current schema marker");
    assert!(matches!(
        SemanticArchiveStore::new(&target.config).verify_schema(),
        Err(SemanticArchiveError::CurrentSchema(_))
    ));
    assert_eq!(scope_at(&target.config, target.campaign_id, 1), scope);
    let absent: bool = admin
        .query_one(
            "SELECT pg_catalog.to_regclass('babylon_meta.current_schema') IS NULL",
            &[],
        )
        .expect("refusal leaves the missing marker absent")
        .get(0);
    assert!(absent);
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime"]
fn live_archive_atom_schema_rejects_every_non_finite_numeric_value() {
    let target = LiveWorkerTarget::create(
        "archivefinite",
        0x2200_0000_0000_0000_0000_0000_0000_00e9,
        1,
    );
    let mut writer = target.config.connect(NoTls).expect("atom constraint probe");
    let insert = "INSERT INTO babylon_meta.archive_atom_v1 \
        (atom_id,campaign_id,subject_kind,subject_id,signal_key,grant_key,evidence_class,\
        value_kind,value_f64,provenance_source_id,provenance_locator,valid_tick) \
        VALUES($1,$2,'county','26163','employment','employment','Observed',\
        'f64',$3,'qcew-2024','county/26163',1)";
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error = writer
            .execute(
                insert,
                &[&&[0x91_u8; 32][..], target.campaign_id.as_uuid(), &value],
            )
            .expect_err("SQL must reject non-finite atoms independently of the Rust encoder");
        assert_eq!(
            error.code(),
            Some(&postgres::error::SqlState::CHECK_VIOLATION)
        );
    }
    assert_eq!(
        writer
            .execute(
                insert,
                &[&&[0x91_u8; 32][..], target.campaign_id.as_uuid(), &0.0_f64]
            )
            .expect("finite zero remains a valid numeric observation"),
        1
    );
    target.finish();
}
