//! Real commit-bound hints, cancellable publication, and listener catch-up.
use super::*;
use babylon_persistence::archive_driver::{ArchiveDriverEventV1, ArchiveDriverV1};
use babylon_persistence::{
    material_runtime::{DurableMaterialRuntimeV3, MaterialRuntimeErrorV3},
    michigan_content::MichiganContentPresetV1,
    ArchiveWorkerCancellationV1, ARCHIVE_WAKEUP_CHANNEL_V1,
};
use postgres::fallible_iterator::FallibleIterator as _;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

fn listen(config: &Config) -> postgres::Client {
    let mut client = config.connect(NoTls).expect("listener connection");
    client
        .batch_execute("LISTEN babylon_archive_wakeup_v1")
        .expect("autocommit LISTEN");
    client
}
fn next_hint(client: &mut postgres::Client, wait: Duration) -> Option<postgres::Notification> {
    client
        .notifications()
        .timeout_iter(wait)
        .next()
        .expect("notification connection")
}
fn assert_installer_refuses_wakeup_drift(
    store: &SemanticArchiveStoreV1,
    writer: &mut postgres::Client,
) {
    writer
        .batch_execute(
            "ALTER TABLE babylon_state.tick_commit DISABLE TRIGGER archive_wakeup_tick_v1",
        )
        .expect("isolated trigger corruption");
    assert_eq!(
        store.install_schema(),
        Err(SemanticArchiveErrorV1::SchemaMismatch)
    );
    writer
        .batch_execute(
            "ALTER TABLE babylon_state.tick_commit ENABLE TRIGGER archive_wakeup_tick_v1",
        )
        .expect("restore trigger");
    writer
        .batch_execute("GRANT EXECUTE ON FUNCTION babylon_meta.archive_wakeup_v1() TO PUBLIC")
        .expect("isolated function exposure");
    assert_eq!(
        store.install_schema(),
        Err(SemanticArchiveErrorV1::SchemaMismatch)
    );
    writer
        .batch_execute("REVOKE ALL ON FUNCTION babylon_meta.archive_wakeup_v1() FROM PUBLIC")
        .expect("restore owner-only function");
    assert_eq!(
        store.install_schema().expect("exact restored install"),
        ArchiveSchemaDispositionV1::AlreadyCurrent
    );
}
fn assert_empty_hint(listener: &mut postgres::Client) {
    let hint = next_hint(listener, Duration::from_secs(2)).expect("committed hint");
    assert_eq!(hint.channel(), ARCHIVE_WAKEUP_CHANNEL_V1);
    assert_eq!(hint.payload(), "");
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime"]
fn live_wakeup_is_commit_bound_empty_and_installer_refuses_trigger_drift() {
    let target =
        LiveWorkerTarget::create("wakeupcommit", 0x2200_0000_0000_0000_0000_0000_0000_00f1, 1);
    let store = SemanticArchiveStoreV1::new(&target.config);
    assert_eq!(
        store.install_schema().expect("idempotent install"),
        ArchiveSchemaDispositionV1::AlreadyCurrent
    );
    let mut listener = listen(&target.config);
    let mut writer = target.config.connect(NoTls).expect("probe writer");
    let mut tx = writer
        .transaction()
        .expect("rolled back enrollment statement");
    // Statement triggers also fire for zero rows; use rollback without inventing
    // a campaign or an invalid retained enrollment merely to test the transport.
    tx.execute("INSERT INTO babylon_meta.archive_retention_v2 SELECT * FROM babylon_meta.archive_retention_v2 WHERE FALSE", &[])
        .expect("transactional enrollment hint");
    assert!(next_hint(&mut listener, Duration::from_millis(150)).is_none());
    tx.rollback().expect("rollback enrollment hint");
    assert!(next_hint(&mut listener, Duration::from_millis(150)).is_none());

    let campaign =
        CampaignId::from_uuid(Uuid::from_u128(0x2200_0000_0000_0000_0000_0000_0000_00f4));
    let (session, bundle) = runtime_fixture_with_seed(WORKER_SEED);
    let foundation = DurableReplayRuntimeV2::create(&target.config, campaign, session, bundle)
        .expect("real zero-tick campaign enrollment");
    assert_empty_hint(&mut listener);
    drop(foundation);
    commit_next(&target.config, target.campaign_id, 2);
    assert_empty_hint(&mut listener);

    // The worker's own pages, consumption, and seal have typed progress results;
    // those writes must not feed another database wake back into this driver.
    let report = ArchiveWorkerV1::new(&target.config)
        .sweep_once(target.campaign_id, &StubPageProducer)
        .expect("canonical Archive publication");
    assert_eq!(report.verified_tick(), 2);
    assert!(next_hint(&mut listener, Duration::from_millis(200)).is_none());
    assert_installer_refuses_wakeup_drift(&store, &mut writer);
    drop(listener);
    drop(writer);
    target.finish();
}

struct CancelAfterProduce(ArchiveWorkerCancellationV1);
impl ArchiveDossierProducerV1 for CancelAfterProduce {
    fn produce(
        &self,
        campaign: Uuid,
        receipt: &PendingArchiveReceiptV1,
        knowledge: &babylon_persistence::ArchiveKnowledgeV1,
        page_budget: usize,
    ) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
        let result = StubPageProducer.produce(campaign, receipt, knowledge, page_budget);
        self.0.request_stop();
        result
    }
}
#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime"]
fn live_worker_stop_rolls_back_uncommitted_pin_and_page_then_retry_drains() {
    let target =
        LiveWorkerTarget::create("wakeupcancel", 0x2200_0000_0000_0000_0000_0000_0000_00f2, 1);
    let cancellation = ArchiveWorkerCancellationV1::default();
    let mut worker = ArchiveWorkerV1::new(&target.config);
    assert_eq!(
        worker.sweep_cancellable(
            target.campaign_id,
            &CancelAfterProduce(cancellation.clone()),
            &cancellation
        ),
        Err(SemanticArchiveErrorV1::WorkerCanceled)
    );
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 0);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        0
    );
    let count: i64 = target
        .config
        .connect(NoTls)
        .expect("pin observer")
        .query_one(
            "SELECT count(*) FROM babylon_meta.archive_tick_knowledge_v2 WHERE campaign_id=$1",
            &[target.campaign_id.as_uuid()],
        )
        .expect("pin count")
        .get(0);
    assert_eq!(
        count, 0,
        "canceled unpublished receipt cannot leave its knowledge pin"
    );
    let report = worker
        .sweep_once(target.campaign_id, &StubPageProducer)
        .expect("retry real canonical worker");
    assert_eq!(
        (
            report.durable_tick(),
            report.verified_tick(),
            report.retention_ready(),
            report.has_pending_work()
        ),
        (1, 1, true, false)
    );
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 1);
    target.finish();
}

fn wait_progress(receiver: &Receiver<ArchiveDriverEventV1>, tick: u64, request: Option<u64>) {
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        let event = receiver
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .expect("bounded driver progress");
        match event {
            ArchiveDriverEventV1::Progress {
                request_id,
                durable_tick,
                verified_tick,
                retention_ready,
            } => {
                assert!(
                    verified_tick <= durable_tick,
                    "coherent progress cannot exceed durable tail"
                );
                if durable_tick == tick
                    && verified_tick == tick
                    && retention_ready
                    && (request.is_none() || request_id == request)
                {
                    return;
                }
            }
            ArchiveDriverEventV1::Failure { retrying: true, .. } => {}
            other => panic!("unexpected driver result: {other:?}"),
        }
    }
}
fn stop(driver: &mut ArchiveDriverV1) {
    driver.request_stop();
    let deadline = Instant::now() + Duration::from_secs(15);
    while !driver.is_finished() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(driver
        .join_if_finished()
        .expect("actual driver completion before deadline")
        .expect("driver did not panic")
        .is_ok());
}
fn listener_pid(client: &mut postgres::Client, database: &str) -> i32 {
    client.query_one("SELECT pid FROM pg_catalog.pg_stat_activity WHERE datname=$1 AND application_name='babylon-archive-listener-v1'", &[&database])
        .expect("one dedicated listener").get(0)
}
fn commit_next(config: &Config, campaign: CampaignId, tick: u64) {
    let mut runtime =
        DurableReplayRuntimeV2::open(config, campaign).expect("reopen same authoritative campaign");
    let actions = OrderedPracticeActionBatchV1::empty(
        runtime.foundation().replay_session_identity().clone(),
        tick,
    )
    .expect("empty next actions");
    assert_eq!(
        runtime
            .advance_and_commit(&mut CollectingSink::default(), &actions)
            .expect("real next commit")
            .resolve_tick()
            .get(),
        tick
    );
}

fn assert_offline_gap_catches_up(target: &LiveWorkerTarget, observer: &mut postgres::Client) {
    let listeners: i64 = observer.query_one(
        "SELECT count(*) FROM pg_catalog.pg_stat_activity WHERE datname=$1 AND application_name='babylon-archive-listener-v1'",
        &[&target.database.name],
    ).expect("stopped listener census").get(0);
    assert_eq!(listeners, 0);
    commit_next(&target.config, target.campaign_id, 4);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        3,
        "offline commit has no live worker or replayed notification"
    );
    let (sender, receiver) = mpsc::sync_channel(16);
    let mut restarted = ArchiveDriverV1::start(&target.config, target.campaign_id, move |event| {
        sender.try_send(event).is_ok()
    })
    .expect("restart after offline commit");
    wait_progress(&receiver, 4, None);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        4,
        "startup catch-up consumes the offline gap without another game commit"
    );
    stop(&mut restarted);
    drop(restarted);
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime"]
fn live_driver_catches_startup_backlog_and_reconnect_then_stops_under_backpressure() {
    let target =
        LiveWorkerTarget::create("wakeupdriver", 0x2200_0000_0000_0000_0000_0000_0000_00f3, 2);
    let (sender, receiver) = mpsc::sync_channel(16);
    let mut driver = ArchiveDriverV1::start(&target.config, target.campaign_id, move |event| {
        sender.try_send(event).is_ok()
    })
    .expect("driver starts");
    // Both commits predate LISTEN; startup must discover and drain them anyway.
    wait_progress(&receiver, 2, None);
    driver
        .request_refresh(41)
        .expect("explicit refresh accepted");
    wait_progress(&receiver, 2, Some(41));
    let mut observer = target.config.connect(NoTls).expect("lifecycle observer");
    let old_pid = listener_pid(&mut observer, &target.database.name);
    observer
        .query_one("SELECT pg_catalog.pg_terminate_backend($1)", &[&old_pid])
        .expect("disconnect exact task listener");
    commit_next(&target.config, target.campaign_id, 3);
    wait_progress(&receiver, 3, None);
    assert_ne!(
        listener_pid(&mut observer, &target.database.name),
        old_pid,
        "reconnect establishes a new LISTEN session"
    );
    stop(&mut driver);
    drop(driver);
    assert_offline_gap_catches_up(&target, &mut observer);
    drop(observer);
    let backpressure_seen = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let sink_signal = backpressure_seen.clone();
    let mut blocked = ArchiveDriverV1::start(&target.config, target.campaign_id, move |_| {
        sink_signal.store(true, std::sync::atomic::Ordering::Release);
        false
    })
    .expect("saturated sink driver");
    let deadline = Instant::now() + Duration::from_secs(90);
    while !backpressure_seen.load(std::sync::atomic::Ordering::Acquire) && Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        backpressure_seen.load(std::sync::atomic::Ordering::Acquire),
        "the sink actually refused an event before stop"
    );
    blocked
        .request_refresh(99)
        .expect("bounded refresh accepted");
    stop(&mut blocked);
    drop(blocked);
    target.finish();
}

/// Census every persisted family keyed by this campaign and resolve tick, rather
/// than checking only the final marker while leaving earlier writes unexamined.
fn assert_no_candidate_rows(client: &mut postgres::Client, campaign: CampaignId, tick: i64) {
    let tables = client.query(
        "SELECT namespace.nspname,relation.relname FROM pg_catalog.pg_class relation \
        JOIN pg_catalog.pg_namespace namespace ON namespace.oid=relation.relnamespace \
        WHERE namespace.nspname IN ('babylon_state','babylon_meta') AND relation.relkind IN ('r','p') \
        AND EXISTS (SELECT 1 FROM pg_catalog.pg_attribute a WHERE a.attrelid=relation.oid AND a.attname='campaign_id' AND NOT a.attisdropped) \
        AND EXISTS (SELECT 1 FROM pg_catalog.pg_attribute a WHERE a.attrelid=relation.oid AND a.attname='resolve_tick' AND NOT a.attisdropped) \
        ORDER BY namespace.nspname,relation.relname", &[],
    ).expect("typed persisted family census");
    assert!(!tables.is_empty());
    for table in tables {
        let schema: String = table.get(0);
        let name: String = table.get(1);
        assert!([&schema, &name].into_iter().all(|value| value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')));
        let count: i64 = client
            .query_one(
                &format!(
                    "SELECT count(*) FROM {schema}.{name} WHERE campaign_id=$1 AND resolve_tick=$2"
                ),
                &[campaign.as_uuid(), &tick],
            )
            .expect("candidate family count")
            .get(0);
        assert_eq!(
            count, 0,
            "rolled-back marker must roll back {schema}.{name}"
        );
    }
    let last_tick: i64 = client
        .query_one(
            "SELECT last_tick FROM babylon_meta.campaign WHERE campaign_id=$1",
            &[campaign.as_uuid()],
        )
        .expect("campaign catalog tail")
        .get(0);
    assert_eq!(
        last_tick,
        tick - 1,
        "campaign catalog advances in the same transaction"
    );
}

fn material_actions(runtime: &DurableMaterialRuntimeV3, tick: u64) -> OrderedPracticeActionBatchV1 {
    OrderedPracticeActionBatchV1::empty(
        runtime.session().graph_session().session_identity().clone(),
        tick,
    )
    .expect("empty authoritative action batch")
}

fn assert_marker_fault_rolls_back(
    runtime: &mut DurableMaterialRuntimeV3,
    writer: &mut postgres::Client,
) {
    let prior_tail = runtime.tail().copied().expect("committed opening tail");
    let prior_world = runtime
        .session()
        .current_world_hash()
        .expect("combined world identity");
    let prior_material = runtime.session().material().canonical_bytes().to_vec();
    let exact_function: String = writer.query_one(
        "SELECT pg_catalog.pg_get_functiondef('babylon_meta.archive_wakeup_v1()'::regprocedure)", &[],
    ).expect("retain exact wakeup function").get(0);
    writer.batch_execute("CREATE OR REPLACE FUNCTION babylon_meta.archive_wakeup_v1() RETURNS trigger \
        LANGUAGE plpgsql SET search_path = pg_catalog AS $fault$ BEGIN \
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='test-owned Archive wakeup failure'; END $fault$")
        .expect("install test-owned marker trigger failure");
    let mut sink = CollectingSink::default();
    let actions = material_actions(runtime, 2);
    let refused = runtime.advance_and_commit(&mut sink, &actions);
    // Restore before assertions, so even a failed assertion leaves the exact
    // installed function rather than a persistent fault in this scratch target.
    writer
        .batch_execute(&exact_function)
        .expect("restore exact wakeup function");
    let Err(MaterialRuntimeErrorV3::Database(error)) = refused else {
        panic!("notification failure must refuse the commit acknowledgement");
    };
    assert_eq!(
        error.code(),
        Some(&postgres::error::SqlState::RAISE_EXCEPTION)
    );
    assert_eq!(runtime.session().completed_tick(), 1);
    assert_eq!(runtime.tail(), Some(&prior_tail));
    assert_eq!(
        runtime
            .session()
            .current_world_hash()
            .expect("unchanged world"),
        prior_world
    );
    assert_eq!(
        runtime.session().material().canonical_bytes(),
        prior_material
    );
    assert!(
        sink.events.is_empty(),
        "failed commit publishes no graph events"
    );
    assert_no_candidate_rows(writer, runtime.campaign_id(), 2);
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime"]
fn live_notification_failure_rolls_back_material_commit_and_preserves_memory_before_retry() {
    let base = validated_base_config();
    let database =
        TestDatabase::create_from_template(&base, &validated_template_name(), "wakeupatomic");
    let config = database.config(&base);
    let campaign =
        CampaignId::from_uuid(Uuid::from_u128(0x2200_0000_0000_0000_0000_0000_0000_00f5));
    let foundation = MichiganContentPresetV1::BundlesStandardV3
        .create_foundation()
        .expect("admitted material foundation");
    let foundation_digest = foundation.digest();
    let mut runtime = DurableMaterialRuntimeV3::create(&config, campaign, foundation)
        .expect("runtime opens before trigger fault");
    let opening_actions = material_actions(&runtime, 1);
    assert_eq!(
        runtime
            .advance_and_commit(&mut CollectingSink::default(), &opening_actions)
            .expect("first real commit")
            .resolve_tick(),
        1
    );
    let mut writer = config
        .connect(NoTls)
        .expect("test-owned trigger fault connection");
    assert_marker_fault_rolls_back(&mut runtime, &mut writer);
    SemanticArchiveStoreV1::new(&config)
        .install_schema()
        .expect("restored trigger identity is exact");
    let next_actions = material_actions(&runtime, 2);
    let second = runtime
        .advance_and_commit(&mut CollectingSink::default(), &next_actions)
        .expect("same live runtime retries exact next tick");
    assert_eq!(second.resolve_tick(), 2);
    let reopened = DurableMaterialRuntimeV3::open(&config, campaign, foundation_digest)
        .expect("durable material reopen");
    assert_eq!(reopened.tail(), Some(&second));
    assert_eq!(
        reopened
            .session()
            .current_world_hash()
            .expect("reopened world"),
        runtime.session().current_world_hash().expect("live world")
    );
    assert_eq!(
        reopened.session().material().canonical_bytes(),
        runtime.session().material().canonical_bytes()
    );
    drop(reopened);
    drop(runtime);
    drop(writer);
    database.cleanup();
}
