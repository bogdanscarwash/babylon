//! Live PER-22 county dossier producer proofs against the task-owned
//! disposable `PostgreSQL` runtime.
//!
//! Each test clones the validated current runtime template and commits real
//! ticks through `DurableMaterialRuntime`. The current Michigan foundation
//! declares all 83 counties and their exact public QCEW baseline fields.
//! These tests prove committed signals, source provenance, quiet receipts,
//! and idempotent publication through the production county dossier path.

#[path = "support/current_material.rs"]
mod current_material;
use babylon_persistence::{material_runtime, michigan_content, michigan_material};

use std::str::FromStr;

#[path = "support/archive_reader.rs"]
mod archive_reader;
use archive_reader::{scope_at, with_reader};
use babylon_persistence::archive_revision::ArchiveReadScope;
use babylon_persistence::archive_revision::{ArchiveDossierBounds, ArchiveDossierState};
use babylon_persistence::{install_reader_role, SemanticArchiveReader};

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_persistence::material_runtime::DurableMaterialRuntime;
use babylon_persistence::{
    identity::CampaignId, postgres_catalog::validate_connection_target, ArchiveReceiptDisposition,
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
const COUNTY_COUNT: i64 = 83;

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
             WHERE campaign_id = $1::uuid AND subject_kind = 'county' AND subject_id = $2 ORDER BY effective_tick DESC LIMIT 1",
            &[campaign_id.as_uuid(), &geoid],
        )
        .expect("county page query")
        .try_get(0)
        .expect("county page decodes")
}

fn assert_public_county_signals(markdown: &str, geoid: &str, values: [i64; 4]) {
    for (label, value) in [
        "QCEW 2024 annual-average establishments",
        "QCEW 2024 annual-average employment (jobs)",
        "QCEW 2024 total annual wages (USD)",
        "QCEW 2024 average weekly wage (USD/week)",
    ]
    .into_iter()
    .zip(values)
    {
        let expected = format!(
            "- **{label}:** {value} — qcew-county-economics-v1; qcew_county_economics_mi_2024.csv.gz#county_geoid={geoid}&sha256=116affb2998c6c0259d5bf14840f99f835d7e0733aa0b4f4c60a257b2723cd16"
        );
        assert!(
            markdown.contains(&expected),
            "the committed {label} signal pins its exact integer, unit, and public artifact row"
        );
    }
    assert!(!markdown.contains("Median wage"));
    assert!(!markdown.contains("Imperial rent"));
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
        "receipt 1 publishes all 83 county pages; unchanged later receipts consume empty"
    );
    assert_eq!(
        report.verified_tick(),
        3,
        "quiet ticks advance verification without changing page content"
    );
    assert_eq!(
        county_page_count(&target.config, target.campaign_id),
        COUNTY_COUNT
    );
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        3
    );

    let wayne = county_page_markdown(&target.config, target.campaign_id, "26163");
    assert!(wayne.contains("# Wayne County"));
    assert!(wayne.contains(COUNTY_DECISION_QUESTION));
    assert!(
        wayne.contains("[Detroit city](subject:place/2622000)"),
        "the foundation-seeded place subject renders the known link label"
    );
    let oakland = county_page_markdown(&target.config, target.campaign_id, "26125");
    assert_public_county_signals(&wayne, "26163", [36_727, 725_504, 55_436_615_328, 1_469]);
    assert_public_county_signals(&oakland, "26125", [43_047, 723_862, 56_401_482_100, 1_498]);

    with_reader(&target.config, |reader| {
        let scope = scope_at(&target.config, target.campaign_id, 3);
        let hits = reader
            .search_as_of(&scope, "55436615328", 10)
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
            page.content_source.tick(),
            1,
            "quiet receipts retain the first publication"
        );
        assert_eq!(
            page.citations.len(),
            2,
            "subject grant plus one shared QCEW artifact-row citation remain deduplicated"
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

    let mut worker = ArchiveWorker::new(&target.config);
    let first = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("first sweep applies the bootstrap receipt");
    assert_eq!(first.applied_count(), 2);
    assert_eq!(first.paged_count(), 0);
    assert_eq!(
        county_page_count(&target.config, target.campaign_id),
        COUNTY_COUNT
    );

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
    assert_eq!(
        county_page_count(&target.config, target.campaign_id),
        COUNTY_COUNT
    );
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
        (0..COUNTY_COUNT)
            .map(|index| ((26_001 + index * 2).to_string(), 1))
            .collect::<Vec<_>>(),
        "quiet receipts preserve exactly one immutable publication per county"
    );
    target.finish();
}
