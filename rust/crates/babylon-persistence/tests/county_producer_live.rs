//! Live PER-22 county dossier producer proofs against the task-owned
//! disposable `PostgreSQL` runtime.
//!
//! Each test clones the validated Rust-active runtime template, commits real
//! ticks through `DurableReplayRuntimeV2` from a scenario that declares the
//! governed `territory/county-fips` mapping (`wayne` = 26163, `oakland` =
//! 26125) with committed `territory/median-wage` and `territory/phi-hour`
//! seeds, and then proves one county dossier acceptance property against the
//! committed dirty receipts.

use std::str::FromStr;

#[path = "support/archive_reader.rs"]
mod archive_reader;
#[path = "support/legacy_archive.rs"]
mod legacy_archive;
use archive_reader::{scope_at, with_reader};
use babylon_persistence::archive_revision::{
    ArchiveDossierBoundsV2, ArchiveDossierPageV2, ArchiveDossierPendingV2, ArchiveDossierStateV2,
    ArchiveDossierUnavailableV2, ArchiveReadScopeV2,
};

use babylon_bsl::rule_pipeline::split_content;
use babylon_bsl::rules_hash_of;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
use babylon_kernel::sha256_of;
use babylon_kernel::tick_content_hash::RefDigestV1;
use babylon_kernel::ContentDigest;
use babylon_persistence::{
    michigan_dynamic_hex_foundation_v1, validate_legacy_connection_target, ArchiveCitationV1,
    ArchiveKnowledgeGrantV1, ArchivePageRefV1, ArchiveReceiptDispositionV1,
    ArchiveSchemaDispositionV1, ArchiveSubjectKindV1, ArchiveWorkerV1, CampaignId,
    CountyDossierProducerV1, DurableReplayRuntimeV2, FoundationContentBundleV1,
    SemanticArchiveStoreV1, COUNTY_DECISION_QUESTION_V1,
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
/// Disposable conformance world: the struggle-spark carrier plus the two
/// governed territory fields and the declared county mapping.
const SCENARIO: &str = r"
; PER-22 county dossier producer disposable conformance world. The
; territory/county-fips mapping is governed geography identity (declared
; int + extensive, write-prohibited to rules) and median-wage/phi-hour are
; the only D2-committed county signal sources.
(scenario county-dossier/conformance
  (defvocabulary NodeType (SOCIAL_CLASS TERRITORY))
  (defenum StruggleSparkOutcome (EXCESSIVE_FORCE NO_INCIDENT))

  (deffield social-class/repression-faced intensity intensive)
  (deffield social-class/agitation-backfire intensity intensive)
  (deffield social-class/last-incident-known int intensive)
  (deffield social-class/last-incident-tick int extensive)

  (deffield territory/county-fips int extensive)
  (deffield territory/median-wage real intensive)
  (deffield territory/phi-hour real intensive)

  (defconst struggle/spark-scale 0.5c)
  (defconst struggle/backfire-step 0.2c)

  (node wayne NodeType/TERRITORY
    (territory/county-fips 26163)
    (territory/median-wage 21.0r)
    (territory/phi-hour 1.0r))

  (node oakland NodeType/TERRITORY
    (territory/county-fips 26125)
    (territory/median-wage 25.0r)
    (territory/phi-hour 2.0r))

  (node workers NodeType/SOCIAL_CLASS
    (social-class/repression-faced 0.5i)
    (social-class/agitation-backfire 0.1i)
    (social-class/last-incident-known 0)
    (social-class/last-incident-tick -1)))
";
const RULE: &str = include_str!("../../babylon-tick/content/rules/struggle-spark.bsl");
const WORKER_SEED: i64 = 2;

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
        ReplaySessionIdV1::try_from("per22/county-producer-live").expect("session id"),
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
        let store = SemanticArchiveStoreV1::new(&config);
        match store.install_schema().expect("Archive schema installs") {
            ArchiveSchemaDispositionV1::Installed | ArchiveSchemaDispositionV1::AlreadyCurrent => {}
        }
        let (session, bundle) = runtime_fixture_with_seed(WORKER_SEED);
        let mut runtime = DurableReplayRuntimeV2::create(&config, campaign_id, session, bundle)
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
    store: &SemanticArchiveStoreV1,
    campaign_id: CampaignId,
    kind: ArchiveSubjectKindV1,
    id: &str,
    grant_key: &str,
    granted_tick: u64,
) {
    store
        .grant_knowledge(
            campaign_id,
            &ArchiveKnowledgeGrantV1::try_new(
                ArchivePageRefV1::try_new(kind, id.to_owned()).expect("page ref"),
                grant_key.to_owned(),
                granted_tick,
                ArchiveCitationV1::try_new(
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
fn grant_county_fields(store: &SemanticArchiveStoreV1, campaign_id: CampaignId) {
    for geoid in ["26125", "26163"] {
        for grant_key in ["median-wage", "phi-hour"] {
            grant(
                store,
                campaign_id,
                ArchiveSubjectKindV1::County,
                geoid,
                grant_key,
                1,
            );
        }
    }
}

fn sweep_dispositions(
    report: &babylon_persistence::ArchiveWorkerSweepReportV1,
) -> Vec<(u64, ArchiveReceiptDispositionV1)> {
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

    let producer = CountyDossierProducerV1::try_new(&target.config).expect("pinned products load");
    let store = SemanticArchiveStoreV1::new(&target.config);
    grant_county_fields(&store, target.campaign_id);

    let mut worker = ArchiveWorkerV1::new(&target.config);
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
            (1, ArchiveReceiptDispositionV1::Applied),
            (2, ArchiveReceiptDispositionV1::Applied),
            (3, ArchiveReceiptDispositionV1::Applied),
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
    assert!(wayne.contains(COUNTY_DECISION_QUESTION_V1));
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
                &ArchiveDossierBoundsV2::default(),
            )
            .expect("exact cited county dossier");
        let ArchiveDossierStateV2::Ready { page, .. } = dossier.state else {
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

    let producer = CountyDossierProducerV1::try_new(&target.config).expect("pinned products load");
    let store = SemanticArchiveStoreV1::new(&target.config);
    grant_county_fields(&store, target.campaign_id);

    let mut worker = ArchiveWorkerV1::new(&target.config);
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
struct StopAfterFirst<'a>(&'a CountyDossierProducerV1);

impl babylon_persistence::ArchiveDossierProducerV1 for StopAfterFirst<'_> {
    fn produce(
        &self,
        campaign: Uuid,
        receipt: &babylon_persistence::PendingArchiveReceiptV1,
        knowledge: &babylon_persistence::ArchiveKnowledgeV1,
        budget: usize,
    ) -> Result<
        babylon_persistence::ArchiveProducerOutcomeV1,
        babylon_persistence::SemanticArchiveErrorV1,
    > {
        if receipt.resolve_tick() > 1 {
            return Err(babylon_persistence::SemanticArchiveErrorV1::InvalidText);
        }
        babylon_persistence::ArchiveDossierProducerV1::produce(
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

    let producer = CountyDossierProducerV1::try_new(&target.config).expect("pinned products load");
    let store = SemanticArchiveStoreV1::new(&target.config);

    // Publish with seeded foundation knowledge only: county
    // subject/identity/containment and every place subject were granted at
    // tick zero, so the pages render with known place links but no signal
    // section — the earned field keys stay ungranted until the refresh below.
    let mut worker = ArchiveWorkerV1::new(&target.config);
    assert_eq!(
        worker.sweep_once(target.campaign_id, &StopAfterFirst(&producer)),
        Err(babylon_persistence::SemanticArchiveErrorV1::InvalidText)
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
        ArchiveSubjectKindV1::County,
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
            (2, ArchiveReceiptDispositionV1::Applied),
            (3, ArchiveReceiptDispositionV1::Applied),
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

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_adoption_retains_exact_current_head_and_validates_quiet_tail_without_a_tick() {
    let target = LiveCountyTarget::create(
        "countyadoption",
        0x2200_0000_0000_0000_0000_0000_0000_00ca,
        3,
    );
    let producer = CountyDossierProducerV1::try_new(&target.config).expect("county producer");
    let store = SemanticArchiveStoreV1::new(&target.config);
    grant_county_fields(&store, target.campaign_id);
    let report = ArchiveWorkerV1::new(&target.config)
        .sweep_once(target.campaign_id, &producer)
        .expect("publish then quiet drain");
    assert_eq!(report.verified_tick(), 3);
    let subject = ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, "26163".to_owned())
        .expect("subject");
    let scope = scope_at(&target.config, target.campaign_id, 3);
    let old = with_reader(&target.config, |reader| {
        let read = reader
            .dossier_as_of(&scope, &subject, &ArchiveDossierBoundsV2::default())
            .expect("original exact page");
        let ArchiveDossierStateV2::Ready { page, .. } = read.state else {
            panic!("original ready");
        };
        page
    });
    assert_eq!(old.content_source.tick(), 1);
    legacy_archive::restore_legacy_heads(&target.config);
    assert_eq!(
        store
            .install_schema()
            .expect("adopt original retained bytes"),
        ArchiveSchemaDispositionV1::Installed
    );
    assert_pending_adoption(&target, &scope, &subject, &old);
    let report = ArchiveWorkerV1::new(&target.config)
        .sweep_once(target.campaign_id, &producer)
        .expect("validate adopted complete desired set");
    assert!(
        report.dispositions().is_empty(),
        "maintenance creates no receipt or game tick"
    );
    assert!(report.retention_ready());
    assert_eq!(report.verified_tick(), 3);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        3
    );
    assert_eq!(scope_at(&target.config, target.campaign_id, 3), scope);
    with_reader(&target.config, |reader| {
        let read = reader
            .dossier_as_of(&scope, &subject, &ArchiveDossierBoundsV2::default())
            .expect("verified adopted head");
        let ArchiveDossierStateV2::Ready {
            page,
            verified_through_tick: 3,
        } = read.state
        else {
            panic!("cutover ready");
        };
        assert_eq!(page.markdown, old.markdown);
        assert_eq!(page.atoms, old.atoms);
        assert_eq!(page.content_source, old.content_source);
        assert_eq!(page.content_sha256, old.content_sha256);
        assert_eq!(page.effective_tick, 3);
        assert!(
            page.changes.changes.is_empty(),
            "adoption is the baseline, not invented older change history"
        );
    });
    assert_eq!(
        store.install_schema().expect("strict immutable reinstall"),
        ArchiveSchemaDispositionV1::AlreadyCurrent
    );
    target.config.connect(NoTls).expect("corruption connection").execute(
        "DELETE FROM babylon_meta.archive_revision_atom_v2 WHERE campaign_id=$1 AND subject_kind='county' AND subject_id='26163' AND origin=0 AND position=0",
        &[target.campaign_id.as_uuid()]
    ).expect("remove one original adopted membership");
    assert_eq!(
        store.install_schema(),
        Err(babylon_persistence::SemanticArchiveErrorV1::StoredPageMismatch),
        "reinstall must still validate the entire original adoption"
    );
    target.finish();
}

fn assert_pending_adoption(
    target: &LiveCountyTarget,
    scope: &ArchiveReadScopeV2,
    subject: &ArchivePageRefV1,
    old: &ArchiveDossierPageV2,
) {
    with_reader(&target.config, |reader| {
        let read = reader
            .dossier_as_of(scope, subject, &ArchiveDossierBoundsV2::default())
            .expect("pending adopted head");
        assert_eq!(read.history_floor_tick, 3);
        assert_eq!(
            read.processed_tick, 3,
            "an old consumed prefix does not prove cutover composition"
        );
        let ArchiveDossierStateV2::Pending {
            page: Some(page),
            reason: ArchiveDossierPendingV2::CutoverValidation,
        } = read.state
        else {
            panic!("adoption requires validation even P=D");
        };
        assert_eq!(page.markdown, old.markdown);
        assert_eq!(page.atoms, old.atoms);
        assert_eq!(page.content_source, old.content_source);
        assert_eq!(page.content_sha256, old.content_sha256);
        let earlier = reader
            .dossier_as_of(
                &scope_at(&target.config, target.campaign_id, 2),
                subject,
                &ArchiveDossierBoundsV2::default(),
            )
            .expect("honest earlier absence");
        assert_eq!(
            earlier.state,
            ArchiveDossierStateV2::Unavailable(ArchiveDossierUnavailableV2::HistoryNotRetained)
        );
    });
}
