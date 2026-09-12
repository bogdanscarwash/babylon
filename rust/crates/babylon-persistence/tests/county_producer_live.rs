//! Live PER-22 county dossier producer proofs against the task-owned
//! disposable `PostgreSQL` runtime.
//!
//! Each test clones the validated Rust-active runtime template, commits real
//! ticks through `DurableMaterialRuntimeV3` from a scenario that declares the
//! governed `territory/county-fips` mapping (`wayne` = 26163, `oakland` =
//! 26125) with committed `territory/median-wage` and `territory/phi-hour`
//! seeds, and then proves one county dossier acceptance property against the
//! committed dirty receipts.

#[path = "support/current_material.rs"]
mod current_material;

use std::str::FromStr;

#[path = "support/archive_reader.rs"]
mod archive_reader;
use archive_reader::{scope_at, with_reader};
use babylon_persistence::archive_revision::{ArchiveDossierBounds, ArchiveDossierState};

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_persistence::material_runtime::DurableMaterialRuntime;
use babylon_persistence::{
    identity::CampaignId, postgres_catalog::validate_connection_target, ArchiveCitation,
    ArchiveKnowledgeGrant, ArchivePageRef, ArchiveReceiptDisposition, ArchiveSubjectKind,
    ArchiveWorker, CountyDossierProducer, SemanticArchiveStore, COUNTY_DECISION_QUESTION,
};
use babylon_practice_contract::OrderedPracticeActionBatch;
use postgres::{Config, NoTls};
use uuid::Uuid;

const DSN_ENV: &str = "BABYLON_POSTGRES_TEST_DSN";
const ACK_ENV: &str = "BABYLON_POSTGRES_DISPOSABLE_ACK";
const ACK: &str = "I_UNDERSTAND_THIS_DISPOSABLE_RUNTIME_DROPS_ITS_SCRATCH_DATABASES_AND_ROLES";
const CANARY_ENV: &str = "BABYLON_POSTGRES_DISPOSABLE_CANARY";
const TEMPLATE_DB_ENV: &str = "BABYLON_RUNTIME_TEMPLATE_DB";

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
        babylon_persistence::preflight_current_schema(&database.config(base))
            .expect("runtime clone has the exact current catalog and role grants");
        let expected_schema_digest = babylon_persistence::current_schema_sha256();
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

struct LiveCountyTarget {
    database: TestDatabase,
    config: Config,
    campaign_id: CampaignId,
}

impl LiveCountyTarget {
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

/// Grant one knowledge grant row through the durable store API.
fn grant(
    store: &SemanticArchiveStore,
    campaign_id: CampaignId,
    kind: ArchiveSubjectKind,
    id: &str,
    grant_key: &str,
    granted_tick: u64,
) {
    store
        .grant_knowledge(
            campaign_id,
            &ArchiveKnowledgeGrant::try_new(
                ArchivePageRef::try_new(kind, id.to_owned()).expect("page ref"),
                grant_key.to_owned(),
                granted_tick,
                ArchiveCitation::try_new(
                    "live-county-grant".to_owned(),
                    format!("{}/{id}@{grant_key}", kind.as_str()),
                )
                .expect("live grant citation"),
            )
            .expect("live knowledge grant"),
        )
        .expect("knowledge grant persists");
}

/// Grant the committed field keys every county page needs. Foundation
/// seeding already granted both counties' subject/identity/containment rows
/// at tick zero, so re-granting `subject` would refuse `GrantConflict`.
fn grant_county_fields(store: &SemanticArchiveStore, campaign_id: CampaignId) {
    for geoid in ["26125", "26163"] {
        for grant_key in ["median-wage", "phi-hour"] {
            grant(
                store,
                campaign_id,
                ArchiveSubjectKind::County,
                geoid,
                grant_key,
                1,
            );
        }
    }
}

fn sweep_dispositions(
    report: &babylon_persistence::ArchiveWorkerSweepReport,
) -> Vec<(u64, ArchiveReceiptDisposition)> {
    report
        .dispositions()
        .iter()
        .map(|(tick, disposition)| (*tick, *disposition))
        .collect()
}

fn county_page_count(config: &Config, campaign_id: CampaignId) -> i64 {
    config
        .connect(NoTls)
        .expect("page count connection")
        .query_one(
            "SELECT pg_catalog.count(DISTINCT subject_id) FROM babylon_meta.archive_page_revision_v2 \
             WHERE campaign_id = $1::uuid AND subject_kind = 'county'",
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

fn county_page_markdown(config: &Config, campaign_id: CampaignId, geoid: &str) -> String {
    config
        .connect(NoTls)
        .expect("county page connection")
        .query_one(
            "SELECT markdown FROM babylon_meta.archive_page_revision_v2 \
             WHERE campaign_id = $1::uuid AND subject_kind = 'county' AND subject_id = $2 ORDER BY effective_tick DESC,origin DESC LIMIT 1",
            &[campaign_id.as_uuid(), &geoid],
        )
        .expect("county page query")
        .try_get(0)
        .expect("county page decodes")
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_county_producer_publishes_committed_signals_then_verifies_quiet_receipts() {
    let target = LiveCountyTarget::create(
        "countyproducerdrain",
        0x2200_0000_0000_0000_0000_0000_0000_00c1,
        3,
    );

    let producer = CountyDossierProducer::try_new(&target.config).expect("pinned products load");
    let store = SemanticArchiveStore::new(&target.config);
    grant_county_fields(&store, target.campaign_id);

    let mut worker = ArchiveWorker::new(&target.config);
    let report = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("county sweep consumes the bootstrap receipt");
    let dispositions = report
        .dispositions()
        .iter()
        .map(|(tick, disposition)| (*tick, *disposition))
        .collect::<Vec<_>>();
    assert_eq!(
        dispositions,
        vec![
            (1, ArchiveReceiptDisposition::Applied),
            (2, ArchiveReceiptDisposition::Applied),
            (3, ArchiveReceiptDisposition::Applied),
        ],
        "receipt 1 publishes both county pages; unchanged later receipts consume empty"
    );
    assert_eq!(
        report.verified_tick(),
        3,
        "quiet ticks advance verification without changing page content"
    );
    assert_eq!(county_page_count(&target.config, target.campaign_id), 2);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        3
    );

    let wayne = county_page_markdown(&target.config, target.campaign_id, "26163");
    assert!(wayne.contains("# Wayne County"));
    assert!(wayne.contains(COUNTY_DECISION_QUESTION));
    assert!(
        wayne.contains("- **Median wage:** 21.000000 — committed-tick-v1; campaign/1/wayne"),
        "the committed median-wage signal pins the exact tick provenance"
    );
    assert!(
        wayne.contains("- **Imperial rent Φ:** 1.000000 — committed-tick-v1; campaign/1/wayne"),
        "the committed phi-hour signal pins the exact tick provenance"
    );
    assert!(
        wayne.contains("[Detroit city](subject:place/2622000)"),
        "the foundation-seeded place subject renders the known link label"
    );
    let oakland = county_page_markdown(&target.config, target.campaign_id, "26125");
    assert!(
        oakland.contains("- **Median wage:** 25.000000 — committed-tick-v1; campaign/1/oakland")
    );
    assert!(
        oakland.contains("- **Imperial rent Φ:** 2.000000 — committed-tick-v1; campaign/1/oakland")
    );

    with_reader(&target.config, |reader| {
        let scope = scope_at(&target.config, target.campaign_id, 3);
        let hits = reader
            .search_as_of(&scope, "21.000000", 10)
            .expect("known-only search");
        assert_eq!(hits.hits.len(), 1);
        assert_eq!(hits.hits[0].subject.id(), "26163");
        let dossier = reader
            .dossier_as_of(
                &scope,
                &hits.hits[0].subject,
                &ArchiveDossierBounds::default(),
            )
            .expect("exact cited county dossier");
        let ArchiveDossierState::Ready { page, .. } = dossier.state else {
            panic!("settled county");
        };
        assert_eq!(
            page.citations.len(),
            2,
            "subject grant plus one shared committed-tick citation remain deduplicated"
        );
    });
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_county_producer_rerun_reconciles_without_duplicate_pages() {
    let target = LiveCountyTarget::create(
        "countyproducerrerun",
        0x2200_0000_0000_0000_0000_0000_0000_00c2,
        2,
    );

    let producer = CountyDossierProducer::try_new(&target.config).expect("pinned products load");
    let store = SemanticArchiveStore::new(&target.config);
    grant_county_fields(&store, target.campaign_id);

    let mut worker = ArchiveWorker::new(&target.config);
    let first = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("first sweep applies the bootstrap receipt");
    assert_eq!(first.applied_count(), 2);
    assert_eq!(first.paged_count(), 0);
    assert_eq!(county_page_count(&target.config, target.campaign_id), 2);

    let second = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("rerun sweep reconciles");
    let dispositions = second
        .dispositions()
        .iter()
        .map(|(tick, disposition)| (*tick, *disposition))
        .collect::<Vec<_>>();
    assert_eq!(
        dispositions,
        vec![],
        "settled receipts need no further work"
    );
    assert_eq!(second.verified_tick(), 2);
    assert_eq!(county_page_count(&target.config, target.campaign_id), 2);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        2,
        "no duplicate consumption rows appear on the rerun"
    );

    let rows: Vec<(String, i64)> = target
        .config
        .connect(NoTls)
        .expect("page rows connection")
        .query(
            "SELECT subject_id, pg_catalog.count(*) FROM babylon_meta.archive_page_revision_v2 \
             WHERE campaign_id = $1::uuid AND subject_kind = 'county' \
             GROUP BY subject_id ORDER BY subject_id",
            &[target.campaign_id.as_uuid()],
        )
        .expect("page rows query")
        .iter()
        .map(|row| {
            (
                row.try_get(0).expect("subject id decodes"),
                row.try_get(1).expect("row count decodes"),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![("26125".to_owned(), 1), ("26163".to_owned(), 1)],
        "quiet receipts preserve exactly one immutable publication per county"
    );
    target.finish();
}

/// Fail after the first durable receipt so the test can add a later field grant.
struct StopAfterFirst<'a>(&'a CountyDossierProducer);

impl babylon_persistence::ArchiveDossierProducer for StopAfterFirst<'_> {
    fn produce(
        &self,
        campaign: Uuid,
        receipt: &babylon_persistence::PendingArchiveReceipt,
        knowledge: &babylon_persistence::ArchiveKnowledge,
        budget: usize,
    ) -> Result<
        babylon_persistence::ArchiveProducerOutcome,
        babylon_persistence::SemanticArchiveError,
    > {
        if receipt.resolve_tick() > 1 {
            return Err(babylon_persistence::SemanticArchiveError::InvalidText);
        }
        babylon_persistence::ArchiveDossierProducer::produce(
            self.0, campaign, receipt, knowledge, budget,
        )
    }
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_county_producer_grant_refresh_republicates_revealed_page() {
    let target = LiveCountyTarget::create(
        "countyproducerrefresh",
        0x2200_0000_0000_0000_0000_0000_0000_00c3,
        3,
    );

    let producer = CountyDossierProducer::try_new(&target.config).expect("pinned products load");
    let store = SemanticArchiveStore::new(&target.config);

    // Publish with seeded foundation knowledge only: county
    // subject/identity/containment and every place subject were granted at
    // tick zero, so the pages render with known place links but no signal
    // section — the earned field keys stay ungranted until the refresh below.
    let mut worker = ArchiveWorker::new(&target.config);
    assert_eq!(
        worker.sweep_once(target.campaign_id, &StopAfterFirst(&producer)),
        Err(babylon_persistence::SemanticArchiveError::InvalidText)
    );
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        1
    );
    let wayne_redacted = county_page_markdown(&target.config, target.campaign_id, "26163");
    assert!(wayne_redacted.contains("# Wayne County"));
    assert!(
        !wayne_redacted.contains("## Signals"),
        "the earned field keys stay ungranted at foundation, so the page publishes no signal"
    );
    assert!(
        wayne_redacted.contains("[Detroit city](subject:place/2622000)"),
        "the seeded place subject reveals the link label"
    );

    // A later field grant arrives, visible from tick two: the wayne page
    // re-dirties and the next pending receipt republishes it with the median
    // wage revealed; phi-hour stays hidden and oakland settles untouched.
    grant(
        &store,
        target.campaign_id,
        ArchiveSubjectKind::County,
        "26163",
        "median-wage",
        2,
    );
    let second = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("grant-refresh sweep republishes");
    assert_eq!(
        sweep_dispositions(&second),
        vec![
            (2, ArchiveReceiptDisposition::Applied),
            (3, ArchiveReceiptDisposition::Applied),
        ],
        "receipt two republishes; receipt three verifies unchanged content"
    );
    let wayne = county_page_markdown(&target.config, target.campaign_id, "26163");
    assert!(
        wayne.contains("- **Median wage:** 21.000000 — committed-tick-v1; campaign/2/wayne"),
        "the signal grant reveals the committed median wage with its provenance"
    );
    assert!(
        wayne.contains("[Detroit city](subject:place/2622000)"),
        "the seeded place subject keeps the link label"
    );
    assert!(
        !wayne.contains("Imperial rent"),
        "phi-hour stays hidden without its own field grant"
    );
    let oakland = county_page_markdown(&target.config, target.campaign_id, "26125");
    assert!(
        !oakland.contains("## Signals"),
        "oakland stays published without signals and untouched"
    );
    assert_eq!(county_page_count(&target.config, target.campaign_id), 2);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        3
    );

    // The revealed page settles: reruns reconcile without further writes.
    let settled = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("settled sweep reconciles");
    assert_eq!(
        sweep_dispositions(&settled),
        vec![],
        "settled receipts stay consumed"
    );
    assert_eq!(county_page_count(&target.config, target.campaign_id), 2);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        3
    );
    target.finish();
}
