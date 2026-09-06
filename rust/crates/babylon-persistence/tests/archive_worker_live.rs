//! Live Archive worker proofs against the task-owned disposable `PostgreSQL` runtime.
//!
//! Each test clones the validated Rust-active runtime template, commits real
//! ticks through `DurableReplayRuntimeV2`, and then proves one worker
//! acceptance property against the committed dirty receipts.

use std::str::FromStr;

#[path = "support/archive_reader.rs"]
mod archive_reader;
#[path = "archive_worker_live/revisions.rs"]
mod revisions;
#[path = "archive_worker_live/wakeup.rs"]
mod wakeup;
use archive_reader::{scope_at, with_reader};
use babylon_persistence::archive_revision::{ArchiveDossierBoundsV2, ArchiveDossierStateV2};

use babylon_bsl::rule_pipeline::split_content;
use babylon_bsl::rules_hash_of;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
use babylon_kernel::sha256_of;
use babylon_kernel::tick_content_hash::RefDigestV1;
use babylon_kernel::ContentDigest;
use babylon_persistence::{
    michigan_dynamic_hex_foundation_v1, seed_foundation_grants_v1,
    validate_legacy_connection_target, ArchiveCitationV1, ArchiveDirtyBatchV1,
    ArchiveDossierProducerV1, ArchiveKnowledgeGrantV1, ArchivePageInputV1, ArchivePageRefV1,
    ArchiveProducerOutcomeV1, ArchiveReceiptDispositionV1, ArchiveSchemaDispositionV1,
    ArchiveSignalV1, ArchiveSubjectKindV1, ArchiveSubjectV1, ArchiveWorkerV1, CampaignId,
    DurableReplayRuntimeV2, FoundationContentBundleV1, FoundationGrantsErrorV1,
    NullArchiveDossierProducerV1, PendingArchiveReceiptV1, SemanticArchiveErrorV1,
    SemanticArchiveStoreV1,
};
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use babylon_tick::material_state::MaterialStateV1;
use babylon_tick::replay_session::ReplayTickSession;
use postgres::{Config, NoTls};
use uuid::Uuid;

const DSN_ENV: &str = "BABYLON_LEGACY_ADOPTER_TEST_DSN";
const ACK_ENV: &str = "BABYLON_LEGACY_ADOPTER_DISPOSABLE_ACK";
const ACK: &str = "I_UNDERSTAND_PER20_DROPS_SCRATCH_DATABASES_ROLES_AND_CREATED_BABYLON_INTEL";
const CANARY_ENV: &str = "BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY";
const TEMPLATE_DB_ENV: &str = "BABYLON_RUNTIME_TEMPLATE_DB";
const DEFINES: &[u8] = br#"{"alpha":1}"#;
const REFERENCE_BUNDLE_DOMAIN: &[u8] = b"babylon.h3.reference-bundle-composite.v1\0";
const SCENARIO: &str =
    include_str!("../../babylon-tick/content/scenarios/struggle-spark-conformance.bscn");
const RULE: &str = include_str!("../../babylon-tick/content/rules/struggle-spark.bsl");
const WORKER_SEED: i64 = 2;

/// One distinct stub subject per receipt tick, because the Archive keeps one
/// latest page per subject while preserving each immutable revision. The ids
/// are synthetic: foundation grant seeding already covers every real Michigan
/// county and place, and an explicit grant for a seeded subject refuses
/// `GrantConflict` instead of shadowing the seeded row.
struct StubSubjectSpec {
    page_ref: ArchivePageRefV1,
    title: &'static str,
}

fn stub_subject_spec(tick: u64) -> StubSubjectSpec {
    let (kind, id, title) = match tick % 3 {
        1 => (ArchiveSubjectKindV1::County, "99963", "Stub County One"),
        2 => (ArchiveSubjectKindV1::Place, "9990001", "Stub Place"),
        _ => (ArchiveSubjectKindV1::County, "99925", "Stub County Two"),
    };
    StubSubjectSpec {
        page_ref: ArchivePageRefV1::try_new(kind, id.to_owned()).expect("stub subject ref"),
        title,
    }
}

fn stub_place_page_ref() -> ArchivePageRefV1 {
    ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, "9990001".to_owned())
        .expect("stub place ref")
}

fn stub_page_input(receipt: &PendingArchiveReceiptV1) -> ArchivePageInputV1 {
    let spec = stub_subject_spec(receipt.resolve_tick());
    let subject = ArchiveSubjectV1::try_new(
        spec.page_ref.kind(),
        spec.page_ref.id().to_owned(),
        spec.title.to_owned(),
    )
    .expect("stub subject");
    ArchivePageInputV1::try_new(
        subject,
        receipt.resolve_tick(),
        *receipt.tick_content_hash(),
        format!(
            "Which neighboring place should organizers investigate at tick {}?",
            receipt.resolve_tick()
        ),
        vec![ArchiveSignalV1::try_new(
            "employment".to_owned(),
            "Employment".to_owned(),
            "728576 jobs".to_owned(),
            ArchiveCitationV1::try_new(
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

impl ArchiveDossierProducerV1 for StubPageProducer {
    fn produce(
        &self,
        _campaign_id: Uuid,
        receipt: &PendingArchiveReceiptV1,
        _knowledge: &babylon_persistence::ArchiveKnowledgeV1,
        _page_budget: usize,
    ) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
        let batch = ArchiveDirtyBatchV1::try_new(
            receipt.resolve_tick(),
            *receipt.tick_content_hash(),
            vec![stub_page_input(receipt)],
        )?;
        Ok(ArchiveProducerOutcomeV1::new(batch, 0))
    }
}

/// Stub producer that refuses one scripted tick to prove exact resume.
struct FailAtTickProducer {
    fail_at_tick: u64,
}

impl ArchiveDossierProducerV1 for FailAtTickProducer {
    fn produce(
        &self,
        campaign_id: Uuid,
        receipt: &PendingArchiveReceiptV1,
        knowledge: &babylon_persistence::ArchiveKnowledgeV1,
        page_budget: usize,
    ) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
        if receipt.resolve_tick() == self.fail_at_tick {
            return Err(SemanticArchiveErrorV1::InvalidText);
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

impl ArchiveDossierProducerV1 for QuietExceptProducer {
    fn produce(
        &self,
        campaign_id: Uuid,
        receipt: &PendingArchiveReceiptV1,
        knowledge: &babylon_persistence::ArchiveKnowledgeV1,
        page_budget: usize,
    ) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
        if receipt.resolve_tick() == self.materialize_tick {
            StubPageProducer.produce(campaign_id, receipt, knowledge, page_budget)
        } else {
            let batch = ArchiveDirtyBatchV1::try_new(
                receipt.resolve_tick(),
                *receipt.tick_content_hash(),
                Vec::new(),
            )?;
            Ok(ArchiveProducerOutcomeV1::new(batch, 0))
        }
    }
}

/// Changed-content receipts surrounding one successful quiet receipt.
struct ChangedExceptProducer {
    quiet_tick: u64,
}

impl ArchiveDossierProducerV1 for ChangedExceptProducer {
    fn produce(
        &self,
        campaign_id: Uuid,
        receipt: &PendingArchiveReceiptV1,
        knowledge: &babylon_persistence::ArchiveKnowledgeV1,
        page_budget: usize,
    ) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
        if receipt.resolve_tick() == self.quiet_tick {
            let batch = ArchiveDirtyBatchV1::try_new(
                receipt.resolve_tick(),
                *receipt.tick_content_hash(),
                Vec::new(),
            )?;
            Ok(ArchiveProducerOutcomeV1::new(batch, 0))
        } else {
            StubPageProducer.produce(campaign_id, receipt, knowledge, page_budget)
        }
    }
}

impl ArchiveDossierProducerV1 for WrongTickProducer {
    fn produce(
        &self,
        _campaign_id: Uuid,
        receipt: &PendingArchiveReceiptV1,
        _knowledge: &babylon_persistence::ArchiveKnowledgeV1,
        _page_budget: usize,
    ) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
        let wrong = PendingArchiveReceiptV1::try_new(
            receipt.resolve_tick() + 1,
            *receipt.tick_content_hash(),
        )
        .expect("wrong-tick receipt boundary");
        let batch = ArchiveDirtyBatchV1::try_new(
            wrong.resolve_tick(),
            *wrong.tick_content_hash(),
            vec![stub_page_input(&wrong)],
        )?;
        Ok(ArchiveProducerOutcomeV1::new(batch, 0))
    }
}

struct UndrainedProducer;

impl ArchiveDossierProducerV1 for UndrainedProducer {
    fn produce(
        &self,
        _campaign_id: Uuid,
        receipt: &PendingArchiveReceiptV1,
        _knowledge: &babylon_persistence::ArchiveKnowledgeV1,
        _page_budget: usize,
    ) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
        Ok(ArchiveProducerOutcomeV1::new(
            ArchiveDirtyBatchV1::try_new(
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

/// Insert marker-backed dirty receipts directly for a tick range, mirroring
/// what a committed tick writes without paying the per-tick runtime cost.
fn insert_marker_backed_dirty_receipts(
    config: &Config,
    campaign_id: CampaignId,
    ticks: std::ops::RangeInclusive<u64>,
) {
    let mut client = config.connect(NoTls).expect("marker insert connection");
    for tick in ticks {
        let resolve_tick = i64::try_from(tick).expect("bounded test tick");
        let mut tick_content_hash = [0u8; 32];
        tick_content_hash[..8].copy_from_slice(&tick.to_be_bytes());
        let envelope_digest = [0xEEu8; 32];
        client
            .execute(
                "INSERT INTO babylon_state.tick_commit (\
                     campaign_id, resolve_tick, envelope_layout_version, \
                     tick_content_hash, envelope_digest\
                 ) VALUES ($1::uuid, $2, 2, $3, $4)",
                &[
                    campaign_id.as_uuid(),
                    &resolve_tick,
                    &&tick_content_hash[..],
                    &&envelope_digest[..],
                ],
            )
            .expect("marker row inserts");
        client
            .execute(
                "INSERT INTO babylon_state.archive_dirty_receipt_v1 \
                 (campaign_id, resolve_tick, tick_content_hash) VALUES ($1::uuid, $2, $3)",
                &[
                    campaign_id.as_uuid(),
                    &resolve_tick,
                    &&tick_content_hash[..],
                ],
            )
            .expect("dirty receipt row inserts");
    }
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
    validate_legacy_connection_target(&config).expect("loopback target");
    assert_eq!(config.get_user(), Some("test"));
    assert_eq!(config.get_dbname(), Some("postgres"));
    let actual: Option<String> = config
        .connect(NoTls)
        .expect("canary connection")
        .query_one(
            "SELECT pg_catalog.current_setting('babylon.per20_disposable', true)",
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
        let observation = database
            .config(base)
            .connect(NoTls)
            .expect("runtime clone connection")
            .query_one(
                "SELECT \
                   (SELECT pg_catalog.string_agg(ordinal::pg_catalog.text || ':' || \
                            state_tag::pg_catalog.text || ':' || schema_epoch::pg_catalog.text, \
                            ',' ORDER BY ordinal) \
                    FROM babylon_meta.persistence_authority_ledger), \
                   (SELECT pg_catalog.count(*) FROM babylon_meta.campaign)",
                &[],
            )
            .expect("runtime clone observation");
        assert_eq!(
            observation
                .try_get::<_, String>(0)
                .expect("authority ledger decodes"),
            "1:1:8,2:2:9"
        );
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

fn runtime_fixture_with_seed(
    seed: i64,
) -> (
    ReplayTickSession<HypergraphStore>,
    FoundationContentBundleV1,
) {
    let (_, rules) = split_content(RULE).expect("live rule parses");
    let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
    let content = ContentDigest {
        defines_hash: sha256_of(DEFINES),
        rules_hash: rules_hash_of(&forms).expect("live rule hashes"),
    };
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
        ReplaySessionIdV1::try_from("per22/archive-worker-live").expect("session id"),
        ReplaySeed::new(seed),
        content,
        reference,
        MaterialStateV1::try_new(foundation).expect("material state"),
    )
    .expect("tick-zero session prepares");
    let bundle =
        FoundationContentBundleV1::try_new(SCENARIO, None, RULE, DEFINES, &reference_manifest)
            .expect("content bundle");
    (session, bundle)
}

fn commit_ticks(runtime: &mut DurableReplayRuntimeV2<HypergraphStore>, count: u64) {
    for tick in 1..=count {
        let actions = OrderedPracticeActionBatchV1::empty(
            runtime.foundation().replay_session_identity().clone(),
            tick,
        )
        .expect("empty action batch");
        let receipt = runtime
            .advance_and_commit(&mut CollectingSink::default(), &actions)
            .expect("tick commits");
        assert_eq!(receipt.resolve_tick().get(), tick);
    }
}

fn grant_stub_knowledge(store: &SemanticArchiveStoreV1, campaign_id: CampaignId, ticks: &[u64]) {
    for tick in ticks {
        let spec = stub_subject_spec(*tick);
        for (grant_key, source_id) in [
            ("subject", "live-worker-subject"),
            ("employment", "live-worker-employment"),
        ] {
            store
                .grant_knowledge(
                    campaign_id,
                    &ArchiveKnowledgeGrantV1::try_new(
                        spec.page_ref.clone(),
                        grant_key.to_owned(),
                        1,
                        ArchiveCitationV1::try_new(
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
        let store = SemanticArchiveStoreV1::new(&config);
        match store.install_schema().expect("Archive schema installs") {
            ArchiveSchemaDispositionV1::Installed | ArchiveSchemaDispositionV1::AlreadyCurrent => {}
        }
        let (session, bundle) = runtime_fixture_with_seed(WORKER_SEED);
        let mut runtime = DurableReplayRuntimeV2::create(&config, campaign_id, session, bundle)
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
        let store = SemanticArchiveStoreV1::new(&config);
        match store.install_schema().expect("Archive schema installs") {
            ArchiveSchemaDispositionV1::Installed | ArchiveSchemaDispositionV1::AlreadyCurrent => {}
        }
        let (session, bundle) = runtime_fixture_with_seed(WORKER_SEED);
        let mut runtime = DurableReplayRuntimeV2::create(&config, campaign_id, session, bundle)
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

    let mut worker = ArchiveWorkerV1::new(&target.config);
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
            (1, ArchiveReceiptDispositionV1::Applied),
            (2, ArchiveReceiptDispositionV1::Applied),
            (3, ArchiveReceiptDispositionV1::Applied),
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
                &ArchiveDossierBoundsV2::default(),
            )
            .expect("exact retained dossier");
        let ArchiveDossierStateV2::Ready {
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

    let mut worker = ArchiveWorkerV1::new(&target.config);
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

    let mut failing = ArchiveWorkerV1::new(&target.config);
    let failure = failing.sweep_once(target.campaign_id, &FailAtTickProducer { fail_at_tick: 2 });
    assert_eq!(failure, Err(SemanticArchiveErrorV1::InvalidText));
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        1
    );
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 1);

    let mut probe = ArchiveWorkerV1::new(&target.config);
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
        vec![(2, ArchiveReceiptDispositionV1::Paged)],
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

    let mut resumed = ArchiveWorkerV1::new(&target.config);
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
            (2, ArchiveReceiptDispositionV1::Applied),
            (3, ArchiveReceiptDispositionV1::Applied),
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
fn live_worker_quiet_backlog_respects_the_bound_and_reaches_later_changed_content() {
    const TICKS: u64 = 300;
    let target = LiveWorkerTarget::create_with_grants(
        "archiveworkersweeppage",
        0x2200_0000_0000_0000_0000_0000_0000_00a7,
        1,
        &[1, TICKS],
    );
    insert_marker_backed_dirty_receipts(&target.config, target.campaign_id, 2..=TICKS);
    let producer = QuietExceptProducer {
        materialize_tick: TICKS,
    };
    let mut worker = ArchiveWorkerV1::new(&target.config);
    let first = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("bounded quiet prefix");
    assert_eq!(first.applied_count(), 256);
    assert_eq!(first.verified_tick(), 256);
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 0);
    let second = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("remaining prefix");
    assert_eq!(second.applied_count(), 44);
    assert_eq!(second.verified_tick(), TICKS);
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 1);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        300
    );
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_worker_stops_at_the_consume_cap_and_leaves_the_remainder_pending() {
    // One real committed tick establishes the campaign; the remaining
    // marker-backed dirty receipts insert directly, as the quiet-backlog
    // proof already does. Tick 256 consumes empty and fills the sweep cap.
    const TICKS: u64 = 300;
    let target = LiveWorkerTarget::create_with_grants(
        "archiveworkercap",
        0x2200_0000_0000_0000_0000_0000_0000_00a9,
        1,
        &[1, 2, 3],
    );
    insert_marker_backed_dirty_receipts(&target.config, target.campaign_id, 2..=TICKS);

    let mut worker = ArchiveWorkerV1::new(&target.config);
    let report = worker
        .sweep_once(
            target.campaign_id,
            &ChangedExceptProducer { quiet_tick: 256 },
        )
        .expect("one sweep stops at the consume cap");

    assert_eq!(
        report.applied_count(),
        256,
        "255 changed receipts plus the quiet receipt at tick 256"
    );
    assert_eq!(report.paged_count(), 0);
    assert_eq!(report.already_consumed_count(), 0);
    assert_eq!(report.dispositions().len(), 256);
    let last = report
        .dispositions()
        .last()
        .expect("the final receipt in the bounded prefix settles");
    assert_eq!(*last, (256, ArchiveReceiptDispositionV1::Applied));
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        256,
        "the consume cap is a hard per-sweep stop, whatever the page composition"
    );
    assert_eq!(
        report.verified_tick(),
        256,
        "the quiet tick 256 settles within the same bounded prefix"
    );

    let mut resumed = ArchiveWorkerV1::new(&target.config);
    let second = resumed
        .sweep_once(target.campaign_id, &StubPageProducer)
        .expect("the remainder stays pending for the next invocation");
    assert_eq!(second.applied_count(), 44, "ticks 257..=300");
    assert_eq!(second.paged_count(), 0);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        i64::try_from(TICKS).expect("bounded tick count")
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
    let mut worker = ArchiveWorkerV1::new(&target.config);
    let settled = worker
        .sweep_once(target.campaign_id, &NullArchiveDossierProducerV1::new())
        .expect("evaluated quiet receipts settle");
    assert_eq!(settled.applied_count(), 2);
    assert_eq!(settled.verified_tick(), 2);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        2
    );
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 0);
    let mut restarted = ArchiveWorkerV1::new(&target.config);
    let rerun = restarted
        .sweep_once(target.campaign_id, &NullArchiveDossierProducerV1::new())
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

    let mut worker = ArchiveWorkerV1::new(&target.config);
    let failure = worker.sweep_once(target.campaign_id, &WrongTickProducer);
    assert_eq!(
        failure,
        Err(SemanticArchiveErrorV1::ReceiptMismatch),
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

    let mut worker = ArchiveWorkerV1::new(&target.config);
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
            (1, ArchiveReceiptDispositionV1::Applied),
            (2, ArchiveReceiptDispositionV1::Applied),
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

    let mut worker = ArchiveWorkerV1::new(&target.config);
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
            Err(babylon_persistence::SemanticArchiveReaderErrorV1::Archive(
                SemanticArchiveErrorV1::StoredPageMismatch
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

    let mut worker = ArchiveWorkerV1::new(&target.config);
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
    let report = seed_foundation_grants_v1(&mut client, target.campaign_id)
        .expect("the exact foundation retry reconciles");
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
    let refusal = seed_foundation_grants_v1(&mut client, target.campaign_id)
        .expect_err("a drifted grant row must refuse the foundation retry");
    assert_eq!(
        refusal,
        FoundationGrantsErrorV1::Archive(SemanticArchiveErrorV1::GrantConflict)
    );
    target.finish();
}
