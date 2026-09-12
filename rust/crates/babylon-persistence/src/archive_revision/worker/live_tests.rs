//! Live Archive worker proofs against the task-owned disposable `PostgreSQL` runtime.
//!
//! Each test clones the validated current runtime template, commits real
//! ticks through `DurableMaterialRuntime`, and then proves one worker
//! acceptance property against the committed dirty receipts.

#[path = "../../../tests/support/current_material.rs"]
mod current_material;

use crate::archive_revision::ArchiveReadScope;
use crate::{install_reader_role, SemanticArchiveReader};
use crate::{material_runtime, michigan_content, michigan_material};
use std::str::FromStr;

#[path = "live_tests/bounds.rs"]
mod bounds;

#[path = "../../../tests/support/archive_reader.rs"]
mod archive_reader;
#[path = "live_tests/revisions.rs"]
mod revisions;
#[path = "live_tests/wakeup.rs"]
mod wakeup;
use crate::archive_revision::{ArchiveDossierBounds, ArchiveDossierState};
use archive_reader::{scope_at, with_reader};

use crate::material_runtime::DurableMaterialRuntime;
use crate::{
    identity::CampaignId, postgres_catalog::validate_connection_target, seed_foundation_grants,
    ArchiveCitation, ArchiveDirtyBatch, ArchiveDossierProducer, ArchiveKnowledgeGrant,
    ArchivePageInput, ArchivePageRef, ArchiveProducerOutcome, ArchiveReceiptDisposition,
    ArchiveSignal, ArchiveSubject, ArchiveSubjectKind, ArchiveWorker, FoundationGrantsError,
    NullArchiveDossierProducer, PendingArchiveReceipt, SemanticArchiveError, SemanticArchiveStore,
};
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_practice_contract::OrderedPracticeActionBatch;
use postgres::{Config, NoTls};
use uuid::Uuid;

const DSN_ENV: &str = "BABYLON_POSTGRES_TEST_DSN";
const ACK_ENV: &str = "BABYLON_POSTGRES_DISPOSABLE_ACK";
const ACK: &str = "I_UNDERSTAND_THIS_DISPOSABLE_RUNTIME_DROPS_ITS_SCRATCH_DATABASES_AND_ROLES";
const CANARY_ENV: &str = "BABYLON_POSTGRES_DISPOSABLE_CANARY";
const TEMPLATE_DB_ENV: &str = "BABYLON_RUNTIME_TEMPLATE_DB";

/// One distinct stub subject per receipt tick, because the Archive keeps one
/// latest page per subject while preserving each immutable revision. The ids
/// are synthetic: foundation grant seeding already covers every real Michigan
/// county and place, and an explicit grant for a seeded subject refuses
/// `GrantConflict` instead of shadowing the seeded row.
struct StubSubjectSpec {
    page_ref: ArchivePageRef,
    title: &'static str,
}

fn stub_subject_spec(tick: u64) -> StubSubjectSpec {
    let (kind, id, title) = match tick % 3 {
        1 => (ArchiveSubjectKind::County, "99963", "Stub County One"),
        2 => (ArchiveSubjectKind::Place, "9990001", "Stub Place"),
        _ => (ArchiveSubjectKind::County, "99925", "Stub County Two"),
    };
    StubSubjectSpec {
        page_ref: ArchivePageRef::try_new(kind, id.to_owned()).expect("stub subject ref"),
        title,
    }
}

fn stub_place_page_ref() -> ArchivePageRef {
    ArchivePageRef::try_new(ArchiveSubjectKind::Place, "9990001".to_owned())
        .expect("stub place ref")
}

fn stub_page_input(receipt: &PendingArchiveReceipt) -> ArchivePageInput {
    let spec = stub_subject_spec(receipt.resolve_tick());
    let subject = ArchiveSubject::try_new(
        spec.page_ref.kind(),
        spec.page_ref.id().to_owned(),
        spec.title.to_owned(),
    )
    .expect("stub subject");
    ArchivePageInput::try_new(
        subject,
        receipt.resolve_tick(),
        *receipt.tick_content_hash(),
        format!(
            "Which neighboring place should organizers investigate at tick {}?",
            receipt.resolve_tick()
        ),
        vec![ArchiveSignal::try_new(
            "employment".to_owned(),
            "Employment".to_owned(),
            "728576 jobs".to_owned(),
            ArchiveCitation::try_new(
                "qcew-2024".to_owned(),
                "fact_qcew_county_rollup county_fips=26163".to_owned(),
            )
            .expect("citation"),
        )
        .expect("signal")],
        Vec::new(),
    )
    .expect("stub page")
}

/// Stub producer that materializes one valid page per receipt.
struct StubPageProducer;

impl ArchiveDossierProducer for StubPageProducer {
    fn produce(
        &self,
        _campaign_id: Uuid,
        receipt: &PendingArchiveReceipt,
        _knowledge: &crate::ArchiveKnowledge,
        _page_budget: usize,
    ) -> Result<ArchiveProducerOutcome, SemanticArchiveError> {
        let batch = ArchiveDirtyBatch::try_new(
            receipt.resolve_tick(),
            *receipt.tick_content_hash(),
            vec![stub_page_input(receipt)],
        )?;
        Ok(ArchiveProducerOutcome::new(batch, 0))
    }
}

/// Stub producer that refuses one scripted tick to prove exact resume.
struct FailAtTickProducer {
    fail_at_tick: u64,
}

impl ArchiveDossierProducer for FailAtTickProducer {
    fn produce(
        &self,
        campaign_id: Uuid,
        receipt: &PendingArchiveReceipt,
        knowledge: &crate::ArchiveKnowledge,
        page_budget: usize,
    ) -> Result<ArchiveProducerOutcome, SemanticArchiveError> {
        if receipt.resolve_tick() == self.fail_at_tick {
            return Err(SemanticArchiveError::InvalidText);
        }
        StubPageProducer.produce(campaign_id, receipt, knowledge, page_budget)
    }
}

/// Stub producer that returns a well-formed batch bound to the wrong tick
/// identity, proving the worker refuses identity drift before the store.
struct WrongTickProducer;

/// Successful quiet receipts surrounding one changed-content receipt.
struct QuietExceptProducer {
    materialize_tick: u64,
}

impl ArchiveDossierProducer for QuietExceptProducer {
    fn produce(
        &self,
        campaign_id: Uuid,
        receipt: &PendingArchiveReceipt,
        knowledge: &crate::ArchiveKnowledge,
        page_budget: usize,
    ) -> Result<ArchiveProducerOutcome, SemanticArchiveError> {
        if receipt.resolve_tick() == self.materialize_tick {
            StubPageProducer.produce(campaign_id, receipt, knowledge, page_budget)
        } else {
            let batch = ArchiveDirtyBatch::try_new(
                receipt.resolve_tick(),
                *receipt.tick_content_hash(),
                Vec::new(),
            )?;
            Ok(ArchiveProducerOutcome::new(batch, 0))
        }
    }
}

/// Changed-content receipts surrounding one successful quiet receipt.
struct ChangedExceptProducer {
    quiet_tick: u64,
}

impl ArchiveDossierProducer for ChangedExceptProducer {
    fn produce(
        &self,
        campaign_id: Uuid,
        receipt: &PendingArchiveReceipt,
        knowledge: &crate::ArchiveKnowledge,
        page_budget: usize,
    ) -> Result<ArchiveProducerOutcome, SemanticArchiveError> {
        if receipt.resolve_tick() == self.quiet_tick {
            let batch = ArchiveDirtyBatch::try_new(
                receipt.resolve_tick(),
                *receipt.tick_content_hash(),
                Vec::new(),
            )?;
            Ok(ArchiveProducerOutcome::new(batch, 0))
        } else {
            StubPageProducer.produce(campaign_id, receipt, knowledge, page_budget)
        }
    }
}

impl ArchiveDossierProducer for WrongTickProducer {
    fn produce(
        &self,
        _campaign_id: Uuid,
        receipt: &PendingArchiveReceipt,
        _knowledge: &crate::ArchiveKnowledge,
        _page_budget: usize,
    ) -> Result<ArchiveProducerOutcome, SemanticArchiveError> {
        let wrong = PendingArchiveReceipt::try_new(
            receipt.resolve_tick() + 1,
            *receipt.tick_content_hash(),
        )
        .expect("wrong-tick receipt boundary");
        let batch = ArchiveDirtyBatch::try_new(
            wrong.resolve_tick(),
            *wrong.tick_content_hash(),
            vec![stub_page_input(&wrong)],
        )?;
        Ok(ArchiveProducerOutcome::new(batch, 0))
    }
}

struct UndrainedProducer;

impl ArchiveDossierProducer for UndrainedProducer {
    fn produce(
        &self,
        _campaign_id: Uuid,
        receipt: &PendingArchiveReceipt,
        _knowledge: &crate::ArchiveKnowledge,
        _page_budget: usize,
    ) -> Result<ArchiveProducerOutcome, SemanticArchiveError> {
        Ok(ArchiveProducerOutcome::new(
            ArchiveDirtyBatch::try_new(
                receipt.resolve_tick(),
                *receipt.tick_content_hash(),
                Vec::new(),
            )?,
            1,
        ))
    }
}

/// Insert one dirty receipt row with no `tick_commit` marker, as a crash
/// residue or partial rollback would leave behind.
fn insert_orphan_dirty_receipt(
    config: &Config,
    campaign_id: CampaignId,
    resolve_tick: i64,
    tick_content_hash: [u8; 32],
) {
    config
        .connect(NoTls)
        .expect("orphan insert connection")
        .execute(
            "INSERT INTO babylon_state.archive_dirty_receipt_v1 \
             (campaign_id, resolve_tick, tick_content_hash) VALUES ($1::uuid, $2, $3)",
            &[
                campaign_id.as_uuid(),
                &resolve_tick,
                &&tick_content_hash[..],
            ],
        )
        .expect("orphan dirty receipt inserts without a marker");
}

fn dirty_receipt_count(config: &Config, campaign_id: CampaignId) -> i64 {
    config
        .connect(NoTls)
        .expect("dirty receipt count connection")
        .query_one(
            "SELECT pg_catalog.count(*) FROM babylon_state.archive_dirty_receipt_v1 \
             WHERE campaign_id = $1::uuid",
            &[campaign_id.as_uuid()],
        )
        .expect("dirty receipt count query")
        .try_get(0)
        .expect("dirty receipt count decodes")
}

fn validated_base_config() -> Config {
    assert_eq!(std::env::var(ACK_ENV).as_deref(), Ok(ACK));
    let canary = std::env::var(CANARY_ENV).expect("runner supplies the disposable canary");
    assert_eq!(canary.len(), 32);
    let dsn = std::env::var(DSN_ENV).expect("runner supplies the disposable DSN");
    let config = Config::from_str(&dsn).expect("runner DSN parses");
    validate_connection_target(&config).expect("loopback target");
    assert_eq!(config.get_user(), Some("test"));
    assert_eq!(config.get_dbname(), Some("postgres"));
    let actual: Option<String> = config
        .connect(NoTls)
        .expect("canary connection")
        .query_one(
            "SELECT pg_catalog.current_setting('babylon.disposable_runtime', true)",
            &[],
        )
        .expect("canary query")
        .try_get(0)
        .expect("canary decode");
    assert_eq!(actual.as_deref(), Some(canary.as_str()));
    config
}

fn validated_template_name() -> String {
    let template = std::env::var(TEMPLATE_DB_ENV)
        .expect("runner supplies the validated Rust-active template database");
    let suffix = template
        .strip_prefix("per281_runtime_template_")
        .expect("runtime template uses the task-owned prefix");
    assert_eq!(suffix.len(), 12);
    assert!(suffix
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    assert!(template
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'));
    template
}

struct TestDatabase {
    name: String,
    admin: Config,
    active: bool,
}

impl TestDatabase {
    fn create_from_template(base: &Config, template: &str, label: &str) -> Self {
        assert!(label.bytes().all(|byte| byte.is_ascii_lowercase()));
        assert!(template
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'));
        let name = format!("per281_runtime_{label}_{}", std::process::id());
        let mut admin = base.clone();
        admin.dbname("postgres");
        let sql = format!("CREATE DATABASE \"{name}\" OWNER test TEMPLATE \"{template}\"");
        admin
            .connect(NoTls)
            .expect("admin connection")
            .batch_execute(&sql)
            .expect("runtime clone creation");
        let database = Self {
            name,
            admin,
            active: true,
        };
        crate::preflight_current_schema(&database.config(base))
            .expect("runtime clone has the exact current catalog and role grants");
        let expected_schema_digest = crate::current_schema_sha256();
        let observation = database
            .config(base)
            .connect(NoTls)
            .expect("runtime clone connection")
            .query_one(
                "SELECT \
                   (SELECT pg_catalog.count(*) = 1 AND \
                           pg_catalog.bool_and(singleton AND schema_sha256 = $1) \
                    FROM babylon_meta.current_schema), \
                   (SELECT pg_catalog.count(*) FROM babylon_meta.campaign)",
                &[&expected_schema_digest.as_slice()],
            )
            .expect("runtime clone observation");
        assert!(observation
            .try_get::<_, bool>(0)
            .expect("current schema identity decodes"));
        assert_eq!(
            observation
                .try_get::<_, i64>(1)
                .expect("campaign count decodes"),
            0
        );
        database
    }

    fn config(&self, base: &Config) -> Config {
        let mut config = base.clone();
        config.dbname(&self.name);
        config
    }

    fn cleanup(mut self) {
        self.try_drop_database()
            .expect("runtime test database cleanup");
        self.active = false;
    }

    fn try_drop_database(&self) -> Result<(), ()> {
        let sql = format!("DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)", self.name);
        self.admin
            .connect(NoTls)
            .map_err(|_| ())?
            .batch_execute(&sql)
            .map_err(|_| ())
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        if std::thread::panicking() {
            let _cleanup = self.try_drop_database();
            return;
        }
        self.try_drop_database()
            .expect("runtime test database cleanup");
        self.active = false;
    }
}

fn commit_ticks(runtime: &mut DurableMaterialRuntime, count: u64) {
    for tick in 1..=count {
        let actions = OrderedPracticeActionBatch::empty(
            runtime.session().graph_session().session_identity().clone(),
            tick,
        )
        .expect("empty action batch");
        let receipt = runtime
            .advance_and_commit(&mut CollectingSink::default(), &actions)
            .expect("tick commits");
        assert_eq!(receipt.resolve_tick(), tick);
    }
}

fn grant_stub_knowledge(store: &SemanticArchiveStore, campaign_id: CampaignId, ticks: &[u64]) {
    for tick in ticks {
        let spec = stub_subject_spec(*tick);
        for (grant_key, source_id) in [
            ("subject", "live-worker-subject"),
            ("employment", "live-worker-employment"),
        ] {
            store
                .grant_knowledge(
                    campaign_id,
                    &ArchiveKnowledgeGrant::try_new(
                        spec.page_ref.clone(),
                        grant_key.to_owned(),
                        1,
                        ArchiveCitation::try_new(
                            source_id.to_owned(),
                            format!("{grant_key}@tick-1"),
                        )
                        .expect("live grant citation"),
                    )
                    .expect("live knowledge grant"),
                )
                .expect("knowledge grant persists");
        }
    }
}

fn archive_page_count(config: &Config, campaign_id: CampaignId) -> i64 {
    config
        .connect(NoTls)
        .expect("page count connection")
        .query_one(
            "SELECT pg_catalog.count(*) FROM babylon_meta.archive_page_revision_v2 \
             WHERE campaign_id = $1::uuid",
            &[campaign_id.as_uuid()],
        )
        .expect("page count query")
        .try_get(0)
        .expect("page count decodes")
}

fn receipt_consumption_count(config: &Config, campaign_id: CampaignId) -> i64 {
    config
        .connect(NoTls)
        .expect("consumption count connection")
        .query_one(
            "SELECT pg_catalog.count(*) FROM babylon_meta.archive_receipt_consumption_v1 \
             WHERE campaign_id = $1::uuid",
            &[campaign_id.as_uuid()],
        )
        .expect("consumption count query")
        .try_get(0)
        .expect("consumption count decodes")
}

struct LiveWorkerTarget {
    database: TestDatabase,
    config: Config,
    campaign_id: CampaignId,
}

impl LiveWorkerTarget {
    fn create(label: &str, campaign_uuid: u128, tick_count: u64) -> Self {
        assert!(tick_count > 0);
        let base = validated_base_config();
        let template = validated_template_name();
        let database = TestDatabase::create_from_template(&base, &template, label);
        let config = database.config(&base);
        let campaign_id = CampaignId::from_uuid(Uuid::from_u128(campaign_uuid));
        let store = SemanticArchiveStore::new(&config);
        store.verify_schema().expect("Archive schema installs");
        let foundation = current_material::foundation();
        let mut runtime = DurableMaterialRuntime::create(&config, campaign_id, foundation)
            .expect("runtime constructs after activation");
        commit_ticks(&mut runtime, tick_count);
        drop(runtime);
        grant_stub_knowledge(&store, campaign_id, &[1, 2, 3]);
        Self {
            database,
            config,
            campaign_id,
        }
    }

    /// Create one campaign with `tick_count` committed receipts but grant
    /// knowledge only for `granted_ticks`, for quiet-backlog sweep proofs.
    fn create_with_grants(
        label: &str,
        campaign_uuid: u128,
        tick_count: u64,
        granted_ticks: &[u64],
    ) -> Self {
        assert!(tick_count > 0);
        let base = validated_base_config();
        let template = validated_template_name();
        let database = TestDatabase::create_from_template(&base, &template, label);
        let config = database.config(&base);
        let campaign_id = CampaignId::from_uuid(Uuid::from_u128(campaign_uuid));
        let store = SemanticArchiveStore::new(&config);
        store.verify_schema().expect("Archive schema installs");
        let foundation = current_material::foundation();
        let mut runtime = DurableMaterialRuntime::create(&config, campaign_id, foundation)
            .expect("runtime constructs after activation");
        commit_ticks(&mut runtime, tick_count);
        drop(runtime);
        grant_stub_knowledge(&store, campaign_id, granted_ticks);
        Self {
            database,
            config,
            campaign_id,
        }
    }

    fn finish(self) {
        self.database.cleanup();
    }
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_worker_consumes_pending_receipts_in_tick_order() {
    let target = LiveWorkerTarget::create(
        "archiveworkerorder",
        0x2200_0000_0000_0000_0000_0000_0000_00a1,
        3,
    );

    let mut worker = ArchiveWorker::new(&target.config);
    let report = worker
        .sweep_once(target.campaign_id, &StubPageProducer)
        .expect("sweep applies every pending receipt");

    let dispositions = report.dispositions();
    let applied = dispositions
        .iter()
        .map(|(tick, disposition)| (*tick, *disposition))
        .collect::<Vec<_>>();
    assert_eq!(
        applied,
        vec![
            (1, ArchiveReceiptDisposition::Applied),
            (2, ArchiveReceiptDisposition::Applied),
            (3, ArchiveReceiptDisposition::Applied),
        ]
    );
    assert!(dispositions.windows(2).all(|pair| pair[0].0 < pair[1].0));
    assert_eq!(report.applied_count(), 3);
    assert_eq!(report.already_consumed_count(), 0);
    assert_eq!(report.paged_count(), 0);
    assert_eq!(report.verified_tick(), 3);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        3
    );
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 3);

    with_reader(&target.config, |reader| {
        let scope = scope_at(&target.config, target.campaign_id, 3);
        let hits = reader
            .search_as_of(&scope, "investigate at tick 2", 10)
            .expect("known-only scoped search");
        assert_eq!(hits.hits.len(), 1);
        assert_eq!(hits.hits[0].subject, stub_place_page_ref());
        assert_eq!(hits.hits[0].content_source.tick(), 2);
        let dossier = reader
            .dossier_as_of(
                &scope,
                &stub_place_page_ref(),
                &ArchiveDossierBounds::default(),
            )
            .expect("exact retained dossier");
        let ArchiveDossierState::Ready {
            page,
            verified_through_tick: 3,
        } = dossier.state
        else {
            panic!("settled scope must be ready");
        };
        assert!(page.markdown.contains("728576 jobs"));
        assert_eq!(page.citations.len(), 2);
        assert_eq!(page.citations[0].source_id(), "live-worker-subject");
        assert_eq!(page.citations[1].source_id(), "qcew-2024");
        assert_eq!(page.citations[0].locator(), "subject@tick-1");
        assert_eq!(
            page.citations[1].locator(),
            "fact_qcew_county_rollup county_fips=26163"
        );
    });
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_worker_rerun_reconciles_without_duplicate_publication() {
    let target = LiveWorkerTarget::create(
        "archiveworkerrerun",
        0x2200_0000_0000_0000_0000_0000_0000_00a2,
        2,
    );

    let mut worker = ArchiveWorker::new(&target.config);
    let first = worker
        .sweep_once(target.campaign_id, &StubPageProducer)
        .expect("first sweep applies");
    assert_eq!(first.applied_count(), 2);
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 2);

    let second = worker
        .sweep_once(target.campaign_id, &StubPageProducer)
        .expect("rerun sweep reconciles");
    assert!(second.dispositions().is_empty());
    assert_eq!(second.applied_count(), 0);
    assert_eq!(second.already_consumed_count(), 0);
    assert_eq!(second.paged_count(), 0);
    assert_eq!(
        second.verified_tick(),
        2,
        "an empty sweep reports the persisted contiguous watermark, not zero"
    );
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 2);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        2
    );
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_worker_crash_between_receipts_resumes_exactly() {
    let target = LiveWorkerTarget::create(
        "archiveworkerresume",
        0x2200_0000_0000_0000_0000_0000_0000_00a3,
        3,
    );

    let mut failing = ArchiveWorker::new(&target.config);
    let failure = failing.sweep_once(target.campaign_id, &FailAtTickProducer { fail_at_tick: 2 });
    assert_eq!(failure, Err(SemanticArchiveError::InvalidText));
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        1
    );
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 1);

    let mut probe = ArchiveWorker::new(&target.config);
    let pending = probe
        .sweep_once(target.campaign_id, &UndrainedProducer)
        .expect("probe sweep stages the surviving receipts");
    let pending_dispositions = pending
        .dispositions()
        .iter()
        .map(|(tick, disposition)| (*tick, *disposition))
        .collect::<Vec<_>>();
    assert_eq!(
        pending_dispositions,
        vec![(2, ArchiveReceiptDisposition::Paged)],
        "an undrained receipt prevents every later producer evaluation"
    );
    assert_eq!(
        pending.verified_tick(),
        1,
        "the undrained tick 2 caps the watermark at the contiguous prefix even though tick 1 applied"
    );
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        1
    );

    let mut resumed = ArchiveWorker::new(&target.config);
    let resume = resumed
        .sweep_once(target.campaign_id, &StubPageProducer)
        .expect("resumed sweep completes");
    let resumed_dispositions = resume
        .dispositions()
        .iter()
        .map(|(tick, disposition)| (*tick, *disposition))
        .collect::<Vec<_>>();
    assert_eq!(
        resumed_dispositions,
        vec![
            (2, ArchiveReceiptDisposition::Applied),
            (3, ArchiveReceiptDisposition::Applied),
        ]
    );
    assert_eq!(resume.verified_tick(), 3);
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 3);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        3
    );
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_worker_consumes_empty_batches_once_without_publishing_content() {
    let target = LiveWorkerTarget::create(
        "archiveworkerquiet",
        0x2200_0000_0000_0000_0000_0000_0000_00a4,
        2,
    );
    let mut worker = ArchiveWorker::new(&target.config);
    let settled = worker
        .sweep_once(target.campaign_id, &NullArchiveDossierProducer::new())
        .expect("evaluated quiet receipts settle");
    assert_eq!(settled.applied_count(), 2);
    assert_eq!(settled.verified_tick(), 2);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        2
    );
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 0);
    let mut restarted = ArchiveWorker::new(&target.config);
    let rerun = restarted
        .sweep_once(target.campaign_id, &NullArchiveDossierProducer::new())
        .expect("restarted worker reads settled prefix");
    assert!(rerun.dispositions().is_empty());
    assert_eq!(rerun.verified_tick(), 2);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        2
    );
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 0);
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_worker_refuses_batch_identity_mismatch_without_consuming() {
    let target = LiveWorkerTarget::create(
        "archiveworkeridentity",
        0x2200_0000_0000_0000_0000_0000_0000_00a5,
        2,
    );

    let mut worker = ArchiveWorker::new(&target.config);
    let failure = worker.sweep_once(target.campaign_id, &WrongTickProducer);
    assert_eq!(
        failure,
        Err(SemanticArchiveError::ReceiptMismatch),
        "a batch bound to another tick must stop the sweep before any consumption"
    );
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        0
    );
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 0);
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_worker_skips_orphan_dirty_receipt_without_marker() {
    let target = LiveWorkerTarget::create(
        "archiveworkerorphan",
        0x2200_0000_0000_0000_0000_0000_0000_00a6,
        2,
    );
    insert_orphan_dirty_receipt(&target.config, target.campaign_id, 3, [0xee; 32]);

    let mut worker = ArchiveWorker::new(&target.config);
    let report = worker
        .sweep_once(target.campaign_id, &StubPageProducer)
        .expect("orphan rows never reach the producer or stop the ordered sweep");
    let applied = report
        .dispositions()
        .iter()
        .map(|(tick, disposition)| (*tick, *disposition))
        .collect::<Vec<_>>();
    assert_eq!(
        applied,
        vec![
            (1, ArchiveReceiptDisposition::Applied),
            (2, ArchiveReceiptDisposition::Applied),
        ]
    );
    assert_eq!(report.verified_tick(), 2);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        2
    );
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 2);
    assert_eq!(
        dirty_receipt_count(&target.config, target.campaign_id),
        3,
        "the orphan row stays dirty, unconsumed, and out of the sweep's view"
    );
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_search_refuses_tampered_page_content() {
    let target = LiveWorkerTarget::create(
        "archivetamper",
        0x2200_0000_0000_0000_0000_0000_0000_00a7,
        1,
    );

    let mut worker = ArchiveWorker::new(&target.config);
    worker
        .sweep_once(target.campaign_id, &StubPageProducer)
        .expect("sweep materializes the page");

    with_reader(&target.config, |reader| {
        let scope = scope_at(&target.config, target.campaign_id, 1);
        let hits = reader
            .search_as_of(&scope, "728576", 10)
            .expect("untampered search returns the known page");
        assert_eq!(hits.hits.len(), 1);
        target
            .config
            .connect(NoTls)
            .expect("tamper connection")
            .execute(
                "UPDATE babylon_meta.archive_page_revision_v2 SET markdown = \
             pg_catalog.concat(markdown, ' tampered') WHERE campaign_id = $1::uuid",
                &[target.campaign_id.as_uuid()],
            )
            .expect("stored markdown tampers");
        assert_eq!(
            reader.search_as_of(&scope, "728576", 10),
            Err(crate::SemanticArchiveReaderError::Archive(
                SemanticArchiveError::StoredPageMismatch
            )),
            "bytes that disagree with their digest refuse the canonical read"
        );
    });
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_search_bounds_results_to_the_requested_limit() {
    let target =
        LiveWorkerTarget::create("archivelimit", 0x2200_0000_0000_0000_0000_0000_0000_00a8, 2);

    let mut worker = ArchiveWorker::new(&target.config);
    worker
        .sweep_once(target.campaign_id, &StubPageProducer)
        .expect("sweep materializes both pages");
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 2);

    with_reader(&target.config, |reader| {
        let scope = scope_at(&target.config, target.campaign_id, 2);
        let bounded = reader
            .search_as_of(&scope, "728576", 1)
            .expect("bounded search");
        assert_eq!(
            bounded.hits.len(),
            1,
            "the requested limit bounds matching subjects"
        );
        assert!(bounded.truncated, "the second match is reported honestly");
        let complete = reader
            .search_as_of(&scope, "728576", 10)
            .expect("complete bounded search");
        assert_eq!(complete.hits.len(), 2);
        assert!(!complete.truncated);
    });
    target.finish();
}

/// Assert the seeded foundation grant census is exactly the digest-pinned
/// grant-row set: 83 counties + 745 places + 8 concepts at tick 0 only, with
/// no earned magnitude keys.
fn assert_foundation_grant_census(client: &mut postgres::Client, campaign_id: CampaignId) {
    let census = client
        .query(
            "SELECT subject_kind, grant_key, pg_catalog.count(*) \
             FROM babylon_meta.archive_knowledge_grant_v1 \
             WHERE campaign_id = $1::uuid \
             GROUP BY subject_kind, grant_key ORDER BY subject_kind, grant_key",
            &[campaign_id.as_uuid()],
        )
        .expect("foundation grant census query")
        .iter()
        .map(|row| {
            (
                row.try_get::<_, String>(0).expect("kind decodes"),
                row.try_get::<_, String>(1).expect("key decodes"),
                row.try_get::<_, i64>(2).expect("count decodes"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        census,
        [
            ("concept".to_owned(), "identity".to_owned(), 8),
            ("concept".to_owned(), "subject".to_owned(), 8),
            ("county".to_owned(), "containment".to_owned(), 83),
            ("county".to_owned(), "identity".to_owned(), 83),
            (
                "county".to_owned(),
                "qcew-average-weekly-wage".to_owned(),
                83
            ),
            ("county".to_owned(), "qcew-employment".to_owned(), 83),
            ("county".to_owned(), "qcew-establishments".to_owned(), 83),
            (
                "county".to_owned(),
                "qcew-total-annual-wages".to_owned(),
                83
            ),
            ("county".to_owned(), "subject".to_owned(), 83),
            ("place".to_owned(), "containment".to_owned(), 745),
            ("place".to_owned(), "identity".to_owned(), 745),
            ("place".to_owned(), "subject".to_owned(), 745),
        ],
        "the foundation census is exactly the digest-pinned grant-row set"
    );
    let total: i64 = client
        .query_one(
            "SELECT pg_catalog.count(*) FROM babylon_meta.archive_knowledge_grant_v1 \
             WHERE campaign_id = $1::uuid",
            &[campaign_id.as_uuid()],
        )
        .expect("total grant row query")
        .try_get(0)
        .expect("total grant row count decodes");
    assert_eq!(total, 2_832);
    let grant_ticks = client
        .query(
            "SELECT DISTINCT granted_tick FROM babylon_meta.archive_knowledge_grant_v1 \
             WHERE campaign_id = $1::uuid",
            &[campaign_id.as_uuid()],
        )
        .expect("grant tick census query")
        .iter()
        .map(|row| row.try_get::<_, i64>(0).expect("tick decodes"))
        .collect::<Vec<_>>();
    assert_eq!(grant_ticks, [0], "foundation knowledge predates every tick");
    let earned: i64 = client
        .query_one(
            "SELECT pg_catalog.count(*) FROM babylon_meta.archive_knowledge_grant_v1 \
             WHERE campaign_id = $1::uuid AND grant_key IN \
             ('median-wage', 'phi-hour', 'class-composition', 'employment')",
            &[campaign_id.as_uuid()],
        )
        .expect("earned-key census query")
        .try_get(0)
        .expect("earned-key count decodes");
    assert_eq!(
        earned, 0,
        "magnitude keys are earned in play, never seeded at foundation"
    );
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_foundation_grants_seed_at_campaign_foundation_and_reconcile_exactly() {
    let target = LiveWorkerTarget::create_with_grants(
        "foundationgrants",
        0x2200_0000_0000_0000_0000_0000_0000_00b1,
        1,
        &[],
    );
    let mut client = target
        .config
        .connect(NoTls)
        .expect("foundation grant census connection");
    assert_foundation_grant_census(&mut client, target.campaign_id);

    // Exact retry: insert-if-absent reconciles every row to the same census.
    let report = {
        let mut transaction = client.transaction().unwrap();
        transaction
            .batch_execute("SET LOCAL search_path=pg_catalog; SET LOCAL quote_all_identifiers=off")
            .unwrap();
        let report = seed_foundation_grants(&mut transaction, target.campaign_id)
            .expect("the exact foundation retry reconciles");
        transaction.commit().unwrap();
        report
    };
    assert_eq!(report.counties(), 83);
    assert_eq!(report.places(), 745);
    assert_eq!(report.concepts(), 8);
    assert_eq!(report.grant_rows(), 2_832);
    let retried_total: i64 = client
        .query_one(
            "SELECT pg_catalog.count(*) FROM babylon_meta.archive_knowledge_grant_v1 \
             WHERE campaign_id = $1::uuid",
            &[target.campaign_id.as_uuid()],
        )
        .expect("retried total query")
        .try_get(0)
        .expect("retried total decodes");
    assert_eq!(retried_total, 2_832, "the retry mints no duplicate rows");

    // Divergence: one drifted durable row refuses loudly, never rewrites.
    client
        .execute(
            "UPDATE babylon_meta.archive_knowledge_grant_v1 \
             SET provenance_locator = 'drifted' \
             WHERE campaign_id = $1::uuid AND subject_kind = 'county' \
               AND subject_id = '26001' AND grant_key = 'subject'",
            &[target.campaign_id.as_uuid()],
        )
        .expect("divergence update applies");
    let refusal = {
        let mut transaction = client.transaction().unwrap();
        transaction
            .batch_execute("SET LOCAL search_path=pg_catalog; SET LOCAL quote_all_identifiers=off")
            .unwrap();
        let refusal = seed_foundation_grants(&mut transaction, target.campaign_id)
            .expect_err("a drifted grant row must refuse the foundation retry");
        transaction.commit().unwrap();
        refusal
    };
    assert_eq!(
        refusal,
        FoundationGrantsError::Archive(SemanticArchiveError::GrantConflict)
    );
    target.finish();
}
