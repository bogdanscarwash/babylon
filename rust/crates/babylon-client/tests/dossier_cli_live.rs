//! Live PER-23 Slice 3 proof (ADR249 R10-R11): the headless dossier CLI
//! (`babylon-client --headless …`) driven as a real child process against
//! the task-owned disposable `PostgreSQL` runtime, reading through the
//! confined reader login — never the owner credential.
//!
//! The proof covers the four commands (`tick status`, `dossier show`,
//! `dossier search`, `changelog`), the JSONL stdout contract, the restart
//! proof (two separate processes answer the identical content hash), and
//! the dual-tick pending state (a committed tick 3 the Archive has not
//! materialized leaves the scoped read honestly `pending`).

use std::io::Write;
use std::process::Command;

use babylon_bsl::rule_pipeline::split_content;
use babylon_bsl::rules_hash_of;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_client::dossier::{retained_page, verified_tick};
use babylon_client::ui::dossier_card::{
    ActiveCountyDossier, DossierCampaignId, DossierCardPlugin, DossierFetchState, DossierRefresh,
    InstalledDossier,
};
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
use babylon_kernel::sha256_of;
use babylon_kernel::tick_content_hash::RefDigestV1;
use babylon_kernel::ContentDigest;
use babylon_persistence::{
    install_reader_role_v1, michigan_dynamic_hex_foundation_v1, validate_legacy_connection_target,
    ArchiveCitationV1, ArchiveDirtyBatchV1, ArchiveKnowledgeGrantV1, ArchiveMaterializeModeV1,
    ArchivePageInputV1, ArchivePageRefV1, ArchiveSchemaDispositionV1, ArchiveSignalV1,
    ArchiveSubjectKindV1, ArchiveSubjectV1, ArchiveWorkerV1, CampaignId, DurableReplayRuntimeV2,
    FoundationContentBundleV1, NullArchiveDossierProducerV1, ReaderRoleDispositionV1,
    SemanticArchiveStoreV1,
};
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use babylon_tick::material_state::MaterialStateV1;
use babylon_tick::replay_session::ReplayTickSession;
use bevy::prelude::*;
use postgres::{Config, NoTls};
use serde_json::Value;
use uuid::Uuid;

const DSN_ENV: &str = "BABYLON_LEGACY_ADOPTER_TEST_DSN";
const ACK_ENV: &str = "BABYLON_LEGACY_ADOPTER_DISPOSABLE_ACK";
const ACK: &str = "I_UNDERSTAND_PER20_DROPS_SCRATCH_DATABASES_ROLES_AND_CREATED_BABYLON_INTEL";
const CANARY_ENV: &str = "BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY";
const TEMPLATE_DB_ENV: &str = "BABYLON_RUNTIME_TEMPLATE_DB";
const READER_DSN_ENV: &str = "BABYLON_READER_DSN";
const DEFINES: &[u8] = br#"{"alpha":1}"#;
const REFERENCE_BUNDLE_DOMAIN: &[u8] = b"babylon.h3.reference-bundle-composite.v1\0";
const SCENARIO: &str =
    include_str!("../../babylon-tick/content/scenarios/struggle-spark-conformance.bscn");
const RULE: &str = include_str!("../../babylon-tick/content/rules/struggle-spark.bsl");
const READER_SEED: i64 = 3;

fn validated_base_config() -> Config {
    let dsn =
        std::env::var(DSN_ENV).unwrap_or_else(|_| panic!("{DSN_ENV} must name the runner DSN"));
    assert_eq!(
        std::env::var(ACK_ENV).as_deref(),
        Ok(ACK),
        "the disposable-environment acknowledgement must be exact"
    );
    assert!(std::env::var(CANARY_ENV).is_ok());
    let config: Config = dsn.parse().expect("runner DSN parses");
    validate_legacy_connection_target(&config).expect("runner DSN validates as a loopback target");
    config
}

fn validated_template_name() -> String {
    let template =
        std::env::var(TEMPLATE_DB_ENV).unwrap_or_else(|_| panic!("{TEMPLATE_DB_ENV} must be set"));
    assert!(template
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'));
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

struct ReaderTarget {
    database: TestDatabase,
    config: Config,
    campaign_id: CampaignId,
}

/// One confined `LOGIN` role as a member of `babylon_reader`: the
/// deployment credential shape the reader handle is designed for.
struct ConfinedLogin {
    name: String,
    admin: Config,
    active: bool,
}

impl ConfinedLogin {
    const PASSWORD: &'static str = "readerconfined";

    fn create(base: &Config) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock is after the epoch")
            .as_nanos();
        let name = format!("per281_reader_login_{}_{unique:024x}", std::process::id());
        let mut admin = base.clone();
        admin.dbname("postgres");
        admin
            .connect(NoTls)
            .expect("admin connection")
            .batch_execute(&format!(
                "CREATE ROLE {name} LOGIN PASSWORD '{}' \
                 NOSUPERUSER NOCREATEDB NOCREATEROLE; \
                 GRANT babylon_reader TO {name}; \
                 GRANT SET ON PARAMETER event_triggers TO {name}",
                Self::PASSWORD
            ))
            .expect("confined reader login creates");
        Self {
            name,
            admin,
            active: true,
        }
    }

    fn dsn(&self, host: &str, port: u16, database: &str) -> String {
        format!(
            "postgresql://{}:{}@{host}:{port}/{database}",
            self.name,
            Self::PASSWORD
        )
    }

    fn try_cleanup(&self) -> Result<(), postgres::Error> {
        self.admin.connect(NoTls)?.batch_execute(&format!(
            "REVOKE SET ON PARAMETER event_triggers FROM {role}; DROP ROLE IF EXISTS {role}",
            role = self.name
        ))
    }

    fn cleanup(mut self) {
        self.try_cleanup().expect("confined reader login cleanup");
        self.active = false;
    }
}

impl Drop for ConfinedLogin {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        if std::thread::panicking() {
            let _cleanup = self.try_cleanup();
            return;
        }
        self.try_cleanup().expect("confined reader login cleanup");
        self.active = false;
    }
}

impl ReaderTarget {
    /// Clone the template, commit `tick_count` real ticks for one campaign,
    /// with foundation-enrolled Archive history, then install the reader role.
    fn create(label: &str, campaign_uuid: u128, tick_count: u64) -> Self {
        assert!(tick_count > 0);
        let base = validated_base_config();
        let template = validated_template_name();
        let database = TestDatabase::create_from_template(&base, &template, label);
        let config = database.config(&base);
        // Enroll this new campaign at foundation: history at weeks 1 and 2
        // must be retained, not inferred from a later adopted head.
        let store = SemanticArchiveStoreV1::new(&config);
        match store
            .install_schema()
            .expect("Archive schema installs before campaign creation")
        {
            ArchiveSchemaDispositionV1::Installed | ArchiveSchemaDispositionV1::AlreadyCurrent => {}
        }
        let campaign_id = CampaignId::from_uuid(Uuid::from_u128(campaign_uuid));
        let (session, bundle) = runtime_fixture_with_seed(READER_SEED);
        let mut runtime = DurableReplayRuntimeV2::create(&config, campaign_id, session, bundle)
            .expect("runtime constructs after activation");
        for tick in 1..=tick_count {
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
        drop(runtime);

        assert_eq!(
            install_reader_role_v1(&config).expect("reader role installs"),
            ReaderRoleDispositionV1::Installed
        );
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
        ReplaySessionIdV1::try_from("per23/dossier-cli-live").expect("session id"),
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

fn county_subject() -> ArchiveSubjectV1 {
    ArchiveSubjectV1::try_new(
        ArchiveSubjectKindV1::County,
        "26163".to_owned(),
        "Wayne County".to_owned(),
    )
    .expect("county identity")
}

fn county_page_input(
    verified_tick: u64,
    tick_content_hash: [u8; 32],
    employment_value: &str,
) -> ArchivePageInputV1 {
    ArchivePageInputV1::try_new(
        county_subject(),
        verified_tick,
        tick_content_hash,
        "Which neighboring place should organizers investigate next?".to_owned(),
        vec![ArchiveSignalV1::try_new(
            "employment".to_owned(),
            "Employment".to_owned(),
            employment_value.to_owned(),
            ArchiveCitationV1::try_new(
                "qcew-2024".to_owned(),
                "fact_qcew_county_rollup county_fips=26163".to_owned(),
            )
            .expect("citation"),
        )
        .expect("signal")],
        Vec::new(),
    )
    .expect("county page input")
}

/// Grant employment knowledge (tick one only — a re-grant would refuse
/// `GrantConflict`), then materialize one receipt so the Archive mints the
/// county's atoms at that receipt's tick.
fn materialize_county_page(
    config: &Config,
    campaign_id: CampaignId,
    verified_tick: u64,
    tick_content_hash: [u8; 32],
    employment_value: &str,
) {
    let store = SemanticArchiveStoreV1::new(config);
    let county_ref = ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, "26163".to_owned())
        .expect("county ref");
    if verified_tick == 1 {
        store
            .grant_knowledge(
                campaign_id,
                &ArchiveKnowledgeGrantV1::try_new(
                    county_ref,
                    "employment".to_owned(),
                    1,
                    ArchiveCitationV1::try_new(
                        "reader-live-employment".to_owned(),
                        "employment@tick-1".to_owned(),
                    )
                    .expect("live grant citation"),
                )
                .expect("live knowledge grant"),
            )
            .expect("knowledge grant persists");
    }
    let batch = ArchiveDirtyBatchV1::try_new(
        verified_tick,
        tick_content_hash,
        vec![county_page_input(
            verified_tick,
            tick_content_hash,
            employment_value,
        )],
    )
    .expect("live dirty batch");
    store
        .materialize_receipt(campaign_id, &batch, ArchiveMaterializeModeV1::Consume)
        .expect("live receipt materializes");
}

fn tick_content_hash(config: &Config, campaign_id: CampaignId, resolve_tick: u64) -> [u8; 32] {
    let resolve_tick = i64::try_from(resolve_tick).expect("the receipt tick fits i64");
    let receipt: Vec<u8> = config
        .connect(NoTls)
        .expect("receipt connection")
        .query_one(
            "SELECT tick_content_hash FROM babylon_state.archive_dirty_receipt_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = $2",
            &[campaign_id.as_uuid(), &resolve_tick],
        )
        .expect("receipt query")
        .try_get(0)
        .expect("receipt digest");
    receipt.try_into().expect("exact receipt digest width")
}

struct CliRun {
    code: i32,
    stdout: String,
}

/// Drive the built binary as a real child process, pointed at the
/// confined reader login, with JSONL landing on stdout only.
fn run_cli(reader_dsn: &str, campaign: &str, args: &[&str]) -> CliRun {
    let output = Command::new(env!("CARGO_BIN_EXE_babylon-client"))
        .args([
            babylon_client::cli::HEADLESS_FLAG,
            babylon_client::cli::CAMPAIGN_FLAG,
            campaign,
        ])
        .args(args)
        .env(READER_DSN_ENV, reader_dsn)
        .env_remove(babylon_client::cli::CAMPAIGN_ENV)
        .output()
        .expect("dossier CLI child process spawns");
    CliRun {
        code: output
            .status
            .code()
            .expect("the child exits by status code, never a signal"),
        stdout: String::from_utf8(output.stdout).expect("stdout is UTF-8 JSONL"),
    }
}

fn jsonl(stdout: &str) -> Vec<Value> {
    stdout
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("every stdout line is JSON"))
        .collect()
}

/// `dossier show` against the two-materialized world answers the
/// exact ready card; returns the card for the restart comparison.
fn assert_archive_current_card(reader_dsn: &str, campaign: &str) -> Value {
    let run = run_cli(reader_dsn, campaign, &["dossier", "show", "26163"]);
    assert_eq!(run.code, 0, "dossier show exits 0");
    let rows = jsonl(&run.stdout);
    assert_eq!(rows.len(), 1, "dossier show emits exactly one card");
    let card = &rows[0];
    assert_eq!(card["record"], "county-dossier");
    assert_eq!(card["schema_version"], 2);
    assert_eq!(card["scope"]["tick"], 2);
    assert_eq!(card["scope"]["campaign_id"], campaign);
    assert_eq!(
        card["scope"]["tick_content_hash"].as_str().unwrap().len(),
        64
    );
    assert_eq!(card["page"]["content_sha256"].as_str().unwrap().len(), 64);
    assert_eq!(card["history_floor_tick"], 0);
    assert_eq!(card["geoid"], "26163");
    assert_eq!(card["page"]["title"], "Wayne County");
    assert_eq!(card["durable_tick"], 2);
    assert_eq!(card["page"]["content_source"]["tick"], 2);
    assert_eq!(card["processed_tick"], 2);
    assert_eq!(card["verified_tick"], 2);
    assert_eq!(card["state"], "ready");
    let employment = card["page"]["atoms"]
        .as_array()
        .expect("atoms array")
        .iter()
        .find(|atom| atom["signal_key"] == "employment")
        .expect("the employment atom rides the card");
    assert_eq!(employment["value"], "731000 jobs");
    assert_eq!(employment["valid_tick"], 2);
    card.clone()
}

fn assert_search_finds_the_county(reader_dsn: &str, campaign: &str) {
    let run = run_cli(reader_dsn, campaign, &["dossier", "search", "Wayne County"]);
    assert_eq!(run.code, 0, "dossier search exits 0");
    let rows = jsonl(&run.stdout);
    assert!(
        rows.iter().any(|hit| hit["record"] == "search-hit"
            && hit["subject_kind"] == "county"
            && hit["geoid"] == "26163"
            && hit["title"] == "Wayne County"),
        "the county hit answers the title query, got {rows:?}"
    );
}

fn assert_changelog_feed(reader_dsn: &str, campaign: &str) {
    let run = run_cli(reader_dsn, campaign, &["changelog", "26163"]);
    assert_eq!(run.code, 0, "changelog exits 0");
    let rows = jsonl(&run.stdout);
    // The feed opens with the signal's first appearance (from null), then
    // the identity-change rows; the supersession row is the one that
    // actually crosses the tick boundary.
    let appearance = rows
        .iter()
        .find(|row| {
            row["record"] == "changelog-row"
                && row["signal_key"] == "employment"
                && row["before"].is_null()
        })
        .expect("the employment appearance row exists");
    assert_eq!(appearance["publication_tick"], 1);
    assert_eq!(appearance["after"]["value"], "728576 jobs");
    let employment_row = rows
        .iter()
        .find(|row| {
            row["record"] == "changelog-row"
                && row["signal_key"] == "employment"
                && row["before"]["valid_tick"] == 1
        })
        .expect("the employment supersession row exists");
    assert_eq!(employment_row["publication_tick"], 2);
    assert_eq!(employment_row["before"]["value"], "728576 jobs");
    assert_eq!(employment_row["after"]["value"], "731000 jobs");
    assert!(
        rows.iter()
            .any(|row| row["record"] == "changelog-row" && row["signal_key"] == "subject"),
        "the subject signal's supersession rides the feed too"
    );
}

/// After a tick 3 commits without Archive materialization, the card stays
/// honest: requested/durable 3, retained content 2, verification unavailable.
fn assert_pending_card_after_tick_three(reader_dsn: &str, campaign: &str) {
    let run = run_cli(reader_dsn, campaign, &["dossier", "show", "26163"]);
    assert_eq!(run.code, 0, "dossier show still exits 0");
    let rows = jsonl(&run.stdout);
    assert_eq!(rows.len(), 1);
    let card = &rows[0];
    assert_eq!(card["durable_tick"], 3, "tick 3 is durably committed");
    assert_eq!(card["scope"]["tick"], 3);
    assert_eq!(card["page"]["content_source"]["tick"], 2);
    assert_eq!(
        card["verified_tick"],
        Value::Null,
        "a retained tick-2 page cannot certify the pending tick-3 observation"
    );
    assert_eq!(
        card["state"], "pending",
        "the card is honest about the unmaterialized tick 3"
    );
}

struct ReaderEnvGuard(Option<std::ffi::OsString>);
impl ReaderEnvGuard {
    fn set(dsn: &str) -> Self {
        let prior = std::env::var_os(READER_DSN_ENV);
        std::env::set_var(READER_DSN_ENV, dsn);
        Self(prior)
    }
}
impl Drop for ReaderEnvGuard {
    fn drop(&mut self) {
        match self.0.take() {
            Some(prior) => std::env::set_var(READER_DSN_ENV, prior),
            None => std::env::remove_var(READER_DSN_ENV),
        }
    }
}

fn held_county_app(campaign: CampaignId) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    app.add_plugins(DossierCardPlugin);
    app.insert_resource(DossierCampaignId(campaign));
    app.insert_resource(babylon_client::story::SelectedStory(
        babylon_client::story::counties(),
    ));
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::ZERO,
    ));
    app.update();
    let index = app
        .world()
        .resource::<babylon_client::atlas::CountyAtlas>()
        .index_of_fips("26163")
        .expect("Wayne County is in the atlas");
    app.world_mut()
        .resource_mut::<babylon_client::map::SelectedCounty>()
        .0 = Some(index);
    app
}

fn collect_held_card(app: &mut App) -> InstalledDossier {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        app.update();
        if let Some(card) = &app.world().resource::<ActiveCountyDossier>().0 {
            return card.clone();
        }
        if let DossierFetchState::Failed(error) = app.world().resource::<DossierFetchState>() {
            panic!("held county read failed: {error}");
        }
        assert!(
            std::time::Instant::now() < deadline,
            "held county read timed out"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn assert_restart_preserves_card(reader_dsn: &str, campaign: &str, card: &serde_json::Value) {
    let rerun = run_cli(reader_dsn, campaign, &["dossier", "show", "26163"]);
    assert_eq!(rerun.code, 0, "the second dossier show exits 0");
    let rerun_rows = jsonl(&rerun.stdout);
    assert_eq!(rerun_rows.len(), 1);
    assert_eq!(
        rerun_rows[0]["page"], card["page"],
        "restart preserves the complete retained page and evidence"
    );
    assert_eq!(
        rerun_rows[0]["page"]["content_sha256"], card["page"]["content_sha256"],
        "a separate process answers the identical content hash (restart proof)"
    );
    assert_eq!(
        rerun_rows[0]["verified_tick"], card["verified_tick"],
        "a separate process answers the identical verified tick"
    );
}

fn assert_quiet_sweep_refreshes_held_card(
    target: &ReaderTarget,
    reader_dsn: &str,
    campaign: &str,
    card: &serde_json::Value,
    held_app: &mut App,
    held_before: &InstalledDossier,
) {
    let sweep = ArchiveWorkerV1::new(&target.config)
        .sweep_once(target.campaign_id, &NullArchiveDossierProducerV1)
        .expect("evaluated quiet tick settles");
    assert_eq!(sweep.applied_count(), 1);
    assert_eq!(sweep.verified_tick(), 3);
    let settled = run_cli(reader_dsn, campaign, &["dossier", "show", "26163"]);
    assert_eq!(settled.code, 0);
    let settled = &jsonl(&settled.stdout)[0];
    assert_eq!(settled["page"]["content_source"]["tick"], 2);
    assert_eq!(settled["processed_tick"], 3);
    assert_eq!(settled["verified_tick"], 3);
    assert_eq!(settled["state"], "ready");
    assert_eq!(
        settled["page"]["content_sha256"],
        card["page"]["content_sha256"]
    );
    assert_eq!(settled["page"]["atoms"], card["page"]["atoms"]);
    held_app.world_mut().resource_mut::<DossierRefresh>().bump();
    let held_settled = collect_held_card(held_app);
    assert_eq!(held_settled.read.durable_tick, 3);
    assert_eq!(held_settled.read.scope.tick(), 3);
    assert_eq!(verified_tick(&held_settled.read), Some(3));
    let before_page = retained_page(&held_before.read).unwrap();
    let settled_page = retained_page(&held_settled.read).unwrap();
    assert_eq!(settled_page.content_source.tick(), 2);
    assert_eq!(settled_page.atoms, before_page.atoms);
    assert_eq!(settled_page.changes.changes, before_page.changes.changes);
    assert_eq!(settled_page.content_sha256, before_page.content_sha256);
    let restart = ArchiveWorkerV1::new(&target.config)
        .sweep_once(target.campaign_id, &NullArchiveDossierProducerV1)
        .expect("restarted quiet worker is idle");
    assert!(restart.dispositions().is_empty());
    assert_eq!(restart.verified_tick(), 3);
}

fn commit_third_tick(target: &ReaderTarget) {
    let mut runtime = DurableReplayRuntimeV2::open(&target.config, target.campaign_id)
        .expect("the runtime reopens from its checkpoint");
    let actions = OrderedPracticeActionBatchV1::empty(
        runtime.foundation().replay_session_identity().clone(),
        3,
    )
    .expect("empty action batch");
    let receipt = runtime
        .advance_and_commit(&mut CollectingSink::default(), &actions)
        .expect("tick 3 commits");
    assert_eq!(receipt.resolve_tick().get(), 3);
    drop(runtime);
}

fn assert_pending_held_card(held_app: &mut App) {
    held_app.world_mut().resource_mut::<DossierRefresh>().bump();
    let held_pending = collect_held_card(held_app);
    assert_eq!(held_pending.read.durable_tick, 3);
    assert_eq!(held_pending.read.scope.tick(), 3);
    assert_eq!(
        retained_page(&held_pending.read)
            .unwrap()
            .content_source
            .tick(),
        2
    );
    assert_eq!(verified_tick(&held_pending.read), None);
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_dossier_cli_reads_through_the_confined_reader_and_survives_restart() {
    let mut sink = std::io::stderr().lock();
    let _ = writeln!(
        sink,
        "dossier_cli_live: cloning template and committing ticks"
    );
    let target = ReaderTarget::create("dossiercli", 0x2300_0000_0000_0000_0000_0000_0000_00a3, 2);
    let tick_one_hash = tick_content_hash(&target.config, target.campaign_id, 1);
    let tick_two_hash = tick_content_hash(&target.config, target.campaign_id, 2);
    let _ = writeln!(
        sink,
        "dossier_cli_live: materializing tick-1 and tick-2 pages"
    );
    materialize_county_page(
        &target.config,
        target.campaign_id,
        1,
        tick_one_hash,
        "728576 jobs",
    );
    materialize_county_page(
        &target.config,
        target.campaign_id,
        2,
        tick_two_hash,
        "731000 jobs",
    );

    let base = validated_base_config();
    let host = match base.get_hosts() {
        [postgres::config::Host::Tcp(address)] => address.clone(),
        other => panic!("runner DSN must name one loopback TCP host, got {other:?}"),
    };
    let port = base
        .get_ports()
        .first()
        .copied()
        .expect("runner DSN names a port");
    let login = ConfinedLogin::create(&base);
    let reader_dsn = login.dsn(&host, port, &target.database.name);
    let campaign = target.campaign_id.as_uuid().to_string();

    let _ = writeln!(sink, "dossier_cli_live: running tick status");
    let run = run_cli(&reader_dsn, &campaign, &["tick", "status"]);
    assert_eq!(run.code, 0, "tick status exits 0, stderr above");
    let rows = jsonl(&run.stdout);
    assert_eq!(rows.len(), 1, "tick status emits exactly one row");
    assert_eq!(rows[0]["record"], "tick-status");
    assert_eq!(rows[0]["durable_tick"], 2, "two ticks committed durably");

    let _ = writeln!(sink, "dossier_cli_live: running dossier show");
    let card = assert_archive_current_card(&reader_dsn, &campaign);
    let reader_env = ReaderEnvGuard::set(&reader_dsn);
    let mut held_app = held_county_app(target.campaign_id);
    let held_before = collect_held_card(&mut held_app);
    assert_eq!(
        retained_page(&held_before.read)
            .unwrap()
            .content_source
            .tick(),
        2
    );
    assert_eq!(verified_tick(&held_before.read), Some(2));

    let _ = writeln!(sink, "dossier_cli_live: restart proof (second process)");
    assert_restart_preserves_card(&reader_dsn, &campaign, &card);

    let _ = writeln!(sink, "dossier_cli_live: running dossier search");
    assert_search_finds_the_county(&reader_dsn, &campaign);

    let _ = writeln!(sink, "dossier_cli_live: running changelog");
    assert_changelog_feed(&reader_dsn, &campaign);

    let _ = writeln!(
        sink,
        "dossier_cli_live: committing tick 3 without materializing"
    );
    commit_third_tick(&target);

    let _ = writeln!(
        sink,
        "dossier_cli_live: dossier show under the dual-tick gap"
    );
    assert_pending_card_after_tick_three(&reader_dsn, &campaign);
    assert_pending_held_card(&mut held_app);

    assert_quiet_sweep_refreshes_held_card(
        &target,
        &reader_dsn,
        &campaign,
        &card,
        &mut held_app,
        &held_before,
    );
    drop(held_app);
    drop(reader_env);

    let _ = writeln!(
        sink,
        "dossier_cli_live: tick status under the dual-tick gap"
    );
    let run = run_cli(&reader_dsn, &campaign, &["tick", "status"]);
    assert_eq!(run.code, 0);
    let rows = jsonl(&run.stdout);
    assert_eq!(rows[0]["durable_tick"], 3);

    login.cleanup();
    target.finish();
    let _ = writeln!(sink, "dossier_cli_live: done");
}
