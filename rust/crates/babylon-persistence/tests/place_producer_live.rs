//! Live PER-22 place dossier producer proofs against the task-owned disposable
//! `PostgreSQL` runtime.
//!
//! Each test clones the validated Rust-active runtime template, commits real
//! ticks through `DurableReplayRuntimeV2`, and proves one place dossier
//! acceptance property against the committed dirty receipts: paged bootstrap
//! drain with a pending-until-drained receipt, bounded allowlist drain with
//! clean rerun, and foundation-seeded grants publishing the revealed page
//! without any explicit grant insert.

use std::str::FromStr;

#[path = "support/legacy_archive.rs"]
mod legacy_archive;

use babylon_bsl::rule_pipeline::split_content;
use babylon_bsl::rules_hash_of;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
use babylon_kernel::sha256_of;
use babylon_kernel::tick_content_hash::RefDigestV1;
use babylon_kernel::ContentDigest;
use babylon_persistence::{
    michigan_dynamic_hex_foundation_v1, validate_legacy_connection_target,
    ArchiveDossierProducerV1, ArchiveMaterializeDispositionV1, ArchiveMaterializeModeV1,
    ArchiveReceiptDispositionV1, ArchiveSchemaDispositionV1, ArchiveSubjectKindV1, ArchiveWorkerV1,
    CampaignId, CompositeArchiveDossierProducerV1, CountyDossierProducerV1, DurableReplayRuntimeV2,
    FoundationContentBundleV1, PendingArchiveReceiptV1, PlaceDossierProducerV1,
    SemanticArchiveErrorV1, SemanticArchiveStoreV1,
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
const PLACE_COUNT: usize = 745;
const MAX_PAGES_PER_RECEIPT: usize = 256;

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
        ReplaySessionIdV1::try_from("per22/place-producer-live").expect("session id"),
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

struct LivePlaceTarget {
    database: TestDatabase,
    config: Config,
    campaign_id: CampaignId,
}

impl LivePlaceTarget {
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

fn archive_page_count(config: &Config, campaign_id: CampaignId, subject_kind: &str) -> i64 {
    config
        .connect(NoTls)
        .expect("page count connection")
        .query_one(
            "SELECT pg_catalog.count(DISTINCT subject_id) FROM babylon_meta.archive_page_revision_v2 \
             WHERE campaign_id = $1::uuid AND subject_kind = $2::text",
            &[campaign_id.as_uuid(), &subject_kind],
        )
        .expect("page count query")
        .try_get(0)
        .expect("page count decodes")
}

fn place_page_count(config: &Config, campaign_id: CampaignId) -> i64 {
    archive_page_count(config, campaign_id, "place")
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

fn place_page_rows(config: &Config, campaign_id: CampaignId) -> Vec<(String, i64, String)> {
    config
        .connect(NoTls)
        .expect("place page rows connection")
        .query(
            "SELECT DISTINCT ON(subject_id) subject_id, source_tick, markdown FROM babylon_meta.archive_page_revision_v2 \
             WHERE campaign_id = $1::uuid AND subject_kind = 'place' ORDER BY subject_id,effective_tick DESC,origin DESC",
            &[campaign_id.as_uuid()],
        )
        .expect("place page rows query")
        .iter()
        .map(|row| {
            (
                row.try_get(0).expect("subject id decodes"),
                row.try_get(1).expect("verified tick decodes"),
                row.try_get(2).expect("markdown decodes"),
            )
        })
        .collect()
}

fn detroit_row(rows: &[(String, i64, String)]) -> &(String, i64, String) {
    rows.iter()
        .find(|row| row.0 == "2622000")
        .expect("Detroit page published")
}

fn dispositions(
    report: &babylon_persistence::ArchiveWorkerSweepReportV1,
) -> Vec<(u64, ArchiveReceiptDispositionV1)> {
    report
        .dispositions()
        .iter()
        .map(|(tick, disposition)| (*tick, *disposition))
        .collect()
}

fn assert_revealed_detroit(row: &(String, i64, String), verified_tick: i64) {
    assert_eq!(row.1, verified_tick, "the page carries its receipt tick");
    assert!(
        row.2.contains(
            "census-place-authority-v1; census_place_identity_mi_2023.csv.gz#place_geoid=2622000"
        ),
        "the identity grant reveals the signal citation"
    );
    assert!(
        row.2.contains("[Wayne County](subject:county/26163)"),
        "the county grant reveals the link label"
    );
}

/// Read the staged place-page GEOIDs, requiring the exact count and geoid order.
fn staged_head_geoids(
    config: &Config,
    campaign_id: CampaignId,
    expected_len: usize,
) -> Vec<String> {
    let geoids: Vec<String> = place_page_rows(config, campaign_id)
        .iter()
        .map(|row| row.0.clone())
        .collect();
    assert_eq!(geoids.len(), expected_len);
    let mut sorted = geoids.clone();
    sorted.sort_unstable();
    assert_eq!(geoids, sorted, "the staged head keeps geoid order");
    geoids
}

/// Require every newly staged GEOID to sort strictly after the prior head.
fn assert_new_geoids_sort_after_head(
    config: &Config,
    campaign_id: CampaignId,
    head_geoids: &[String],
) {
    let head_max = head_geoids.last().expect("head is nonempty").clone();
    for (geoid, ..) in place_page_rows(config, campaign_id) {
        if !head_geoids.iter().any(|stored| stored == &geoid) {
            assert!(
                geoid > head_max,
                "the second prefix sorts strictly after the staged head: {geoid}"
            );
        }
    }
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_place_producer_pages_the_bootstrap_drain_across_sweeps() {
    let target = LivePlaceTarget::create(
        "placepageddrain",
        0x2200_0000_0000_0000_0000_0000_0000_00c1,
        1,
    );

    let producer = PlaceDossierProducerV1::try_new(&target.config).expect("pinned products load");
    assert_eq!(
        producer.desired_pages().expect("desired pages").len(),
        PLACE_COUNT
    );

    let mut worker = ArchiveWorkerV1::new(&target.config);
    // Sweep one stages the leading 256-page head; the receipt stays pending
    // and the watermark honestly stalls behind it.
    let first = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("first sweep stages the head batch");
    assert_eq!(
        dispositions(&first),
        vec![(1, ArchiveReceiptDispositionV1::Paged)]
    );
    assert_eq!(first.paged_count(), 1);
    assert_eq!(first.applied_count(), 0);
    assert_eq!(
        first.verified_tick(),
        0,
        "a staged receipt never advances the watermark"
    );
    assert_eq!(place_page_count(&target.config, target.campaign_id), 256);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        0,
        "staging claims nothing"
    );
    let head_geoids = staged_head_geoids(&target.config, target.campaign_id, 256);

    // Sweep two stores the next 256-page prefix: every new geoid sorts after
    // the stored head, which is what advances the drain without dropping pages.
    let second = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("second sweep stages the next prefix");
    assert_eq!(
        dispositions(&second),
        vec![(1, ArchiveReceiptDispositionV1::Paged)]
    );
    assert_eq!(place_page_count(&target.config, target.campaign_id), 512);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        0
    );
    assert_new_geoids_sort_after_head(&target.config, target.campaign_id, &head_geoids);

    // Sweep three drains the 233-page tail whole and consumes the receipt
    // exactly once, so the watermark converges.
    let third = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("third sweep drains the tail");
    assert_eq!(
        dispositions(&third),
        vec![(1, ArchiveReceiptDispositionV1::Applied)]
    );
    assert_eq!(third.paged_count(), 0);
    assert_eq!(third.verified_tick(), 1);
    assert_eq!(
        place_page_count(&target.config, target.campaign_id),
        i64::try_from(PLACE_COUNT).expect("place count fits i64")
    );
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        1
    );

    // A rerun reconciles clean: no pending receipts, no republished pages.
    let rerun = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("rerun sweep reconciles");
    assert!(dispositions(&rerun).is_empty());
    assert_eq!(rerun.verified_tick(), 1);
    assert_eq!(
        place_page_count(&target.config, target.campaign_id),
        i64::try_from(PLACE_COUNT).expect("place count fits i64")
    );
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        1
    );
    for (_, verified_tick, _) in place_page_rows(&target.config, target.campaign_id) {
        assert_eq!(verified_tick, 1, "rerun never republishes a clean page");
    }
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_composite_producer_drains_the_backlog_county_first() {
    let target = LivePlaceTarget::create("compdrain", 0x2200_0000_0000_0000_0000_0000_0000_00c2, 1);

    // The place conformance scenario declares no `territory/county-fips`
    // geography, so scenario reconciliation leaves the declared county mapping
    // empty. Seed the two mapping rows directly — the exact rows a declaring
    // scenario would extract — so the county dossier has a deterministic dirty
    // set to thread ahead of the place head.
    target
        .config
        .connect(NoTls)
        .expect("county map seed connection")
        .execute(
            "INSERT INTO babylon_meta.territory_county_map_v1 \
             (campaign_id, territory_local_name, county_geoid) \
             VALUES ($1::uuid, 'wayne', '26163'), ($1::uuid, 'oakland', '26125')",
            &[target.campaign_id.as_uuid()],
        )
        .expect("county map rows seed");

    let county = CountyDossierProducerV1::try_new(&target.config).expect("county products load");
    let place = PlaceDossierProducerV1::try_new(&target.config).expect("place products load");
    let producer = CompositeArchiveDossierProducerV1::new(vec![Box::new(county), Box::new(place)]);

    let mut worker = ArchiveWorkerV1::new(&target.config);
    // Sweep one proves county-first threading: the shared 256-page budget
    // publishes the county head plus the remaining place head (256 - county
    // exactly), stages them without claiming, and leaves the head receipt
    // pending.
    let first = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("first sweep stages the county pages plus the place head");
    let first_dispositions = dispositions(&first);
    assert_eq!(
        first_dispositions.first(),
        Some(&(1, ArchiveReceiptDispositionV1::Paged)),
        "the head receipt stages its first page batch"
    );
    let staged_county = archive_page_count(&target.config, target.campaign_id, "county");
    let staged_place = place_page_count(&target.config, target.campaign_id);
    assert_eq!(
        staged_county, 2,
        "both declared counties publish in the first batch"
    );
    assert_eq!(
        staged_county + staged_place,
        i64::try_from(MAX_PAGES_PER_RECEIPT).expect("page budget fits i64"),
        "the merged staged batch never exceeds the shared page budget"
    );
    assert_eq!(
        staged_place,
        i64::try_from(MAX_PAGES_PER_RECEIPT).expect("page budget fits i64") - staged_county,
        "the composite threads the budget county-first: the place head is exactly the remainder"
    );
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        0,
        "staging claims nothing"
    );

    // The drain converges in a bounded loop: each pending receipt keeps its
    // pages staged until its own dirty set drains, then consumes exactly once.
    let mut sweeps = 1;
    let mut latest = first;
    while latest.paged_count() > 0 {
        assert!(
            sweeps < 8,
            "the paged drain converges within the bounded loop"
        );
        latest = worker
            .sweep_once(target.campaign_id, &producer)
            .expect("sweep drains the backlog");
        sweeps += 1;
    }
    assert_eq!(latest.paged_count(), 0, "nothing remains undrained");
    assert_eq!(
        place_page_count(&target.config, target.campaign_id),
        i64::try_from(PLACE_COUNT).expect("place count fits i64"),
        "every place page lands exactly once; nothing drops"
    );
    let settled_consumption = receipt_consumption_count(&target.config, target.campaign_id);
    assert!(
        settled_consumption >= 1,
        "the head receipt settles exactly once"
    );

    // A rerun reconciles clean: no paged, no applied, no growth.
    let rerun = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("rerun sweep reconciles");
    assert_eq!(rerun.paged_count(), 0);
    assert!(
        dispositions(&rerun).is_empty(),
        "settled receipts never republish"
    );
    assert_eq!(
        place_page_count(&target.config, target.campaign_id),
        i64::try_from(PLACE_COUNT).expect("place count fits i64")
    );
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        settled_consumption,
        "rerun never re-consumes"
    );
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_staged_batch_restages_without_double_writes() {
    let target =
        LivePlaceTarget::create("stagerestage", 0x2200_0000_0000_0000_0000_0000_0000_00c3, 1);

    let allowlist = vec!["2622000".to_owned()];
    let producer = PlaceDossierProducerV1::with_place_allowlist(&target.config, &allowlist)
        .expect("sorted unique allowlist binds");
    let hash: Vec<u8> = target
        .config
        .connect(NoTls)
        .expect("receipt hash connection")
        .query_one(
            "SELECT tick_content_hash FROM babylon_state.archive_dirty_receipt_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = 1",
            &[target.campaign_id.as_uuid()],
        )
        .expect("one committed dirty receipt")
        .try_get(0)
        .expect("dirty receipt digest");
    let receipt = PendingArchiveReceiptV1::try_new(1, hash.try_into().expect("exact digest width"))
        .expect("pending receipt");
    let outcome = producer
        .produce(
            *target.campaign_id.as_uuid(),
            &receipt,
            &knowledge_at(&target.config, target.campaign_id, receipt.resolve_tick()),
            MAX_PAGES_PER_RECEIPT,
        )
        .expect("allowlisted produce drains whole");
    assert_eq!(outcome.remaining(), 0);
    assert_eq!(outcome.batch().pages().len(), 1);

    let store = SemanticArchiveStoreV1::new(&target.config);
    let first = store
        .materialize_receipt(
            target.campaign_id,
            outcome.batch(),
            ArchiveMaterializeModeV1::Stage,
        )
        .expect("first stage applies");
    assert_eq!(
        first.disposition(),
        ArchiveMaterializeDispositionV1::Applied
    );
    assert_eq!(place_page_count(&target.config, target.campaign_id), 1);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        0,
        "staging writes pages without claiming the receipt"
    );

    // An exact restage — the same sweep crashing between stage and consume —
    // is a no-op through the monotonic page guard and claims nothing.
    let restage = store
        .materialize_receipt(
            target.campaign_id,
            outcome.batch(),
            ArchiveMaterializeModeV1::Stage,
        )
        .expect("restage reconciles");
    assert_eq!(
        restage.disposition(),
        ArchiveMaterializeDispositionV1::Applied
    );
    assert_eq!(place_page_count(&target.config, target.campaign_id), 1);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        0
    );
    let markdown = place_page_rows(&target.config, target.campaign_id)[0]
        .2
        .clone();

    // A later sweep finishes the drain in Consume mode and claims exactly once.
    let consumed = store
        .materialize_receipt(
            target.campaign_id,
            outcome.batch(),
            ArchiveMaterializeModeV1::Consume,
        )
        .expect("consume mode settles the drained receipt");
    assert_eq!(
        consumed.disposition(),
        ArchiveMaterializeDispositionV1::Applied
    );
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        1
    );
    assert_eq!(place_page_count(&target.config, target.campaign_id), 1);
    assert_eq!(
        place_page_rows(&target.config, target.campaign_id)[0].2,
        markdown,
        "settling never rewrites page bytes"
    );

    // After the claim, a stage-mode retry reconciles as AlreadyConsumed.
    let settled = store
        .materialize_receipt(
            target.campaign_id,
            outcome.batch(),
            ArchiveMaterializeModeV1::Stage,
        )
        .expect("settled stage retry reconciles");
    assert_eq!(
        settled.disposition(),
        ArchiveMaterializeDispositionV1::AlreadyConsumed
    );
    assert_eq!(place_page_count(&target.config, target.campaign_id), 1);
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_staged_batch_refuses_tampered_consumption_claim() {
    let target =
        LivePlaceTarget::create("stagetamper", 0x2200_0000_0000_0000_0000_0000_0000_00c4, 1);

    let allowlist = vec!["2622000".to_owned()];
    let producer = PlaceDossierProducerV1::with_place_allowlist(&target.config, &allowlist)
        .expect("sorted unique allowlist binds");
    let hash: Vec<u8> = target
        .config
        .connect(NoTls)
        .expect("receipt hash connection")
        .query_one(
            "SELECT tick_content_hash FROM babylon_state.archive_dirty_receipt_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = 1",
            &[target.campaign_id.as_uuid()],
        )
        .expect("one committed dirty receipt")
        .try_get(0)
        .expect("dirty receipt digest");
    let receipt = PendingArchiveReceiptV1::try_new(1, hash.try_into().expect("exact digest width"))
        .expect("pending receipt");
    let outcome = producer
        .produce(
            *target.campaign_id.as_uuid(),
            &receipt,
            &knowledge_at(&target.config, target.campaign_id, receipt.resolve_tick()),
            MAX_PAGES_PER_RECEIPT,
        )
        .expect("allowlisted produce drains whole");

    let store = SemanticArchiveStoreV1::new(&target.config);
    store
        .materialize_receipt(
            target.campaign_id,
            outcome.batch(),
            ArchiveMaterializeModeV1::Stage,
        )
        .expect("stage applies the drained batch");
    store
        .materialize_receipt(
            target.campaign_id,
            outcome.batch(),
            ArchiveMaterializeModeV1::Consume,
        )
        .expect("consume mode claims the receipt");

    // Tamper with the stored claim. A stage retry must reconcile the stored
    // claim digests and refuse the mismatch exactly like Consume mode,
    // never masking it as an idempotent AlreadyConsumed.
    let tampered = target
        .config
        .connect(NoTls)
        .expect("tamper connection")
        .execute(
            "UPDATE babylon_meta.archive_receipt_consumption_v1 \
             SET batch_sha256 = tick_content_hash \
             WHERE campaign_id = $1::uuid AND resolve_tick = 1",
            &[target.campaign_id.as_uuid()],
        )
        .expect("tamper applies");
    assert_eq!(tampered, 1, "exactly one claim row exists");

    let refused = store.materialize_receipt(
        target.campaign_id,
        outcome.batch(),
        ArchiveMaterializeModeV1::Stage,
    );
    assert_eq!(
        refused,
        Err(SemanticArchiveErrorV1::ReceiptConflict),
        "a stage retry reconciles the stored claim digests and refuses a mismatch"
    );
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_installer_upgrades_the_legacy_page_provenance_anchor() {
    let target = LivePlaceTarget::create(
        "legacyfkupgrade",
        0x2200_0000_0000_0000_0000_0000_0000_00c4,
        1,
    );

    legacy_archive::restore_legacy_heads(&target.config);

    // Re-anchor the page provenance at the consumption marker, exactly the
    // pre-PER-318 shape, to prove the installer upgrades an installed schema.
    let legacy_fk_targets_consumption: bool = target
        .config
        .connect(NoTls)
        .expect("legacy anchor connection")
        .query_one(
            "SELECT pg_catalog.pg_get_constraintdef(oid) LIKE '%archive_receipt_consumption_v1%' \
             FROM pg_catalog.pg_constraint \
             WHERE conname = 'archive_page_v1_campaign_id_source_resolve_tick_fkey' \
               AND conrelid = 'babylon_meta.archive_page_v1'::pg_catalog.regclass",
            &[],
        )
        .expect("legacy anchor lookup")
        .try_get(0)
        .expect("legacy anchor decodes");
    assert!(
        !legacy_fk_targets_consumption,
        "a fresh template already anchors pages at the durable dirty receipt"
    );
    target
        .config
        .connect(NoTls)
        .expect("legacy anchor connection")
        .batch_execute(
            "ALTER TABLE babylon_meta.archive_page_v1 \
             DROP CONSTRAINT archive_page_v1_campaign_id_source_resolve_tick_fkey; \
             ALTER TABLE babylon_meta.archive_page_v1 \
             ADD CONSTRAINT archive_page_v1_campaign_id_source_resolve_tick_fkey \
             FOREIGN KEY (campaign_id, source_resolve_tick) \
             REFERENCES babylon_meta.archive_receipt_consumption_v1(campaign_id, resolve_tick) \
             ON DELETE CASCADE",
        )
        .expect("legacy consumption anchor applies");

    let store = SemanticArchiveStoreV1::new(&target.config);
    assert_eq!(
        store
            .install_schema()
            .expect("installer upgrades the legacy anchor"),
        ArchiveSchemaDispositionV1::Installed,
        "re-anchoring the page provenance reports an installed change"
    );
    let upgraded: bool = target
        .config
        .connect(NoTls)
        .expect("upgrade check connection")
        .query_one(
            "SELECT confrelid = 'babylon_state.archive_dirty_receipt_v1'::pg_catalog.regclass \
             FROM pg_catalog.pg_constraint \
             WHERE conname = 'archive_page_v1_campaign_id_source_resolve_tick_fkey' \
               AND conrelid = 'babylon_meta.archive_page_retired_v1'::pg_catalog.regclass",
            &[],
        )
        .expect("upgrade check lookup")
        .try_get(0)
        .expect("upgrade check decodes");
    assert!(
        upgraded,
        "the upgraded schema anchors pages at the durable dirty receipt"
    );

    // The upgraded schema stages a paged drain batch the legacy anchor refused.
    let producer = PlaceDossierProducerV1::try_new(&target.config).expect("pinned products load");
    let mut worker = ArchiveWorkerV1::new(&target.config);
    let report = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("upgraded schema stages the bootstrap head");
    assert_eq!(
        dispositions(&report),
        vec![(1, ArchiveReceiptDispositionV1::Paged)]
    );
    assert_eq!(place_page_count(&target.config, target.campaign_id), 256);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        0,
        "staging still claims nothing after the upgrade"
    );
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_place_producer_drains_allowlisted_pages_and_reruns_clean() {
    let target = LivePlaceTarget::create(
        "placeproducerdrain",
        0x2200_0000_0000_0000_0000_0000_0000_00b1,
        2,
    );

    let allowlist = vec![
        "2600380".to_owned(),
        "2622000".to_owned(),
        "2627760".to_owned(),
        "2684000".to_owned(),
        "2689320".to_owned(),
    ];
    let producer = PlaceDossierProducerV1::with_place_allowlist(&target.config, &allowlist)
        .expect("sorted unique allowlist binds");

    // No explicit grants: foundation seeding granted every allowlisted place
    // subject and identity plus every overlapping county subject at tick zero.
    let mut worker = ArchiveWorkerV1::new(&target.config);
    let report = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("allowlisted sweep drains the small backlog");
    assert_eq!(
        report
            .dispositions()
            .iter()
            .map(|(tick, disposition)| (*tick, *disposition))
            .collect::<Vec<_>>(),
        vec![
            (1, ArchiveReceiptDispositionV1::Applied),
            (2, ArchiveReceiptDispositionV1::Applied),
        ],
        "one receipt publishes every allowlisted place; the next verifies unchanged content"
    );
    assert_eq!(report.verified_tick(), 2);
    assert_eq!(place_page_count(&target.config, target.campaign_id), 5);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        2
    );

    let rows = place_page_rows(&target.config, target.campaign_id);
    assert_eq!(rows.len(), 5);
    let detroit = detroit_row(&rows);
    assert_eq!(detroit.1, 1);
    assert!(detroit.2.contains("# Detroit city"));
    assert!(
        detroit.2.contains(
            "census-place-authority-v1; census_place_identity_mi_2023.csv.gz#place_geoid=2622000"
        ),
        "a granted identity signal pins the exact artifact row"
    );
    assert!(
        detroit.2.contains("[Wayne County](subject:county/26163)"),
        "a granted county subject renders its known label"
    );

    let fenton = rows
        .iter()
        .find(|row| row.0 == "2627760")
        .expect("Fenton city published");
    for county in ["26049", "26093", "26125"] {
        assert!(
            fenton.2.contains(&format!("](subject:county/{county})")),
            "cross-county place keeps every county slice, including {county}"
        );
    }

    // A rerun reconciles without duplicate or republished pages: the
    // settled receipts remain consumed and no content is rewritten.
    let rerun = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("rerun sweep reconciles");
    assert_eq!(
        dispositions(&rerun),
        vec![],
        "the settled receipts need no further work"
    );
    assert_eq!(rerun.verified_tick(), 2);
    assert_eq!(place_page_count(&target.config, target.campaign_id), 5);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        2
    );
    for (_, verified_tick, _) in place_page_rows(&target.config, target.campaign_id) {
        assert_eq!(verified_tick, 1, "rerun never republishes a clean page");
    }
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_place_producer_foundation_grants_publish_revealed_page_and_rerun_is_idle() {
    let target = LivePlaceTarget::create(
        "placeproducerrefresh",
        0x2200_0000_0000_0000_0000_0000_0000_00b4,
        3,
    );

    let allowlist = vec!["2622000".to_owned()];
    let producer = PlaceDossierProducerV1::with_place_allowlist(&target.config, &allowlist)
        .expect("sorted unique allowlist binds");

    // No explicit grants: foundation seeding granted the place subject and
    // identity and the overlapping county subject at tick zero, so the first
    // receipt already publishes the fully revealed page.
    let mut worker = ArchiveWorkerV1::new(&target.config);
    let first = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("foundation-knowledge sweep publishes the revealed page");
    assert_eq!(
        dispositions(&first),
        vec![
            (1, ArchiveReceiptDispositionV1::Applied),
            (2, ArchiveReceiptDispositionV1::Applied),
            (3, ArchiveReceiptDispositionV1::Applied),
        ]
    );
    let rows = place_page_rows(&target.config, target.campaign_id);
    assert_revealed_detroit(detroit_row(&rows), 1);
    assert_eq!(place_page_count(&target.config, target.campaign_id), 1);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        3
    );

    // The revealed page settles: reruns reconcile without further writes.
    let settled = worker
        .sweep_once(target.campaign_id, &producer)
        .expect("settled sweep reconciles");
    assert_eq!(
        dispositions(&settled),
        vec![],
        "the revealed page and quiet receipt prefix stay settled"
    );
    assert_eq!(settled.verified_tick(), 3);
    assert_eq!(place_page_count(&target.config, target.campaign_id), 1);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        3
    );
    target.finish();
}

fn knowledge_at(
    config: &Config,
    campaign: CampaignId,
    tick: u64,
) -> babylon_persistence::ArchiveKnowledgeV1 {
    let rows=config.connect(NoTls).expect("knowledge fixture connection").query(
        "SELECT subject_kind,subject_id,grant_key,granted_tick,provenance_source_id,provenance_locator FROM babylon_meta.archive_knowledge_grant_v1 WHERE campaign_id=$1 AND granted_tick<=$2 AND subject_kind IN ('county','place') ORDER BY subject_kind,subject_id,grant_key", &[campaign.as_uuid(), &i64::try_from(tick).expect("fixture tick")]).expect("fixture exact knowledge");
    let grants = rows
        .iter()
        .map(|row| {
            let kind = match row.get::<_, &str>(0) {
                "county" => ArchiveSubjectKindV1::County,
                "place" => ArchiveSubjectKindV1::Place,
                _ => panic!("closed fixture page kind"),
            };
            babylon_persistence::ArchiveKnowledgeGrantV1::try_new(
                babylon_persistence::ArchivePageRefV1::try_new(kind, row.get(1))
                    .expect("fixture page"),
                row.get(2),
                u64::try_from(row.get::<_, i64>(3)).expect("fixture grant tick"),
                babylon_persistence::ArchiveCitationV1::try_new(row.get(4), row.get(5))
                    .expect("fixture citation"),
            )
            .expect("fixture grant")
        })
        .collect();
    babylon_persistence::ArchiveKnowledgeV1::try_new(grants).expect("fixture knowledge")
}
