//! Live PER-23 Slice 1 proofs for the read-only reader role and the
//! fog-safe committed-tick status view (ADR249 R8).
//!
//! Each test clones the validated Rust-active runtime template, commits real
//! ticks through `DurableMaterialRuntimeV3`, verifies the current Archive and
//! reader-role schemas, and then proves one privilege property against the
//! live `PostgreSQL` privilege layer on the one current schema.

#[path = "support/current_material.rs"]
mod current_material;
use babylon_persistence::{material_runtime, michigan_content, michigan_material};

use std::str::FromStr;

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_kernel::content_digest::sha256_of;
use babylon_persistence::archive_revision::{
    ArchiveDossierBounds, ArchiveDossierPage, ArchiveDossierPending, ArchiveDossierRead,
    ArchiveDossierState, ArchiveDossierUnavailable, ArchiveReadScope, ArchiveSearchState,
};
use babylon_persistence::material_runtime::{DurableMaterialRuntime, MaterialRuntimeError};
use babylon_persistence::michigan_content::MichiganContentPreset;
use babylon_persistence::michigan_material::MichiganDeliveryPreset;
use babylon_persistence::runtime_session::{
    run_runtime_session, RuntimeSessionRequest, RuntimeSessionResponse, RuntimeSessionScope,
    RuntimeSessionTail, RuntimeSessionTarget, RUNTIME_SESSION_PROTOCOL_VERSION,
};
use babylon_persistence::{
    identity::CampaignId, install_reader_role, postgres_catalog::validate_connection_target,
    ArchiveCitation, ArchiveDirtyBatch, ArchiveKnowledgeGrant, ArchiveMaterializeMode,
    ArchivePageInput, ArchivePageRef, ArchiveSignal, ArchiveSubject, ArchiveSubjectKind,
    ReaderRoleDisposition, SemanticArchiveReader, SemanticArchiveStore,
};
use babylon_persistence::{
    observer_reader::provision_observer_role, observer_reader::ObserverEconomyError,
    observer_reader::ObserverEconomyReader, observer_reader::ObserverVisibility,
};
use babylon_practice_contract::OrderedPracticeActionBatch;
use babylon_tick::material_replay::IdentifiedMaterialTick;
use postgres::{Config, NoTls};
use uuid::Uuid;

const DSN_ENV: &str = "BABYLON_POSTGRES_TEST_DSN";
const ACK_ENV: &str = "BABYLON_POSTGRES_DISPOSABLE_ACK";
const ACK: &str = "I_UNDERSTAND_THIS_DISPOSABLE_RUNTIME_DROPS_ITS_SCRATCH_DATABASES_AND_ROLES";
const CANARY_ENV: &str = "BABYLON_POSTGRES_DISPOSABLE_CANARY";
const TEMPLATE_DB_ENV: &str = "BABYLON_RUNTIME_TEMPLATE_DB";
const READER_DSN_ENV: &str = "BABYLON_READER_DSN";

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
        let observation = database
            .config(base)
            .connect(NoTls)
            .expect("runtime clone connection")
            .query_one(
                "SELECT \
                   (SELECT pg_catalog.encode(schema_sha256, 'hex') FROM babylon_meta.current_schema WHERE singleton), \
                   (SELECT pg_catalog.count(*) FROM babylon_meta.campaign)",
                &[],
            )
            .expect("runtime clone observation");
        assert_eq!(
            observation
                .try_get::<_, String>(0)
                .expect("current schema identity decodes"),
            babylon_tick::hex(&babylon_persistence::current_schema_sha256())
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

struct ReaderTarget {
    database: TestDatabase,
    config: Config,
    campaign_id: CampaignId,
}

/// One confined `LOGIN` role as a member of `babylon_reader`: the deployment
/// credential shape the reader handle is designed for (the reader role itself
/// is `NOLOGIN` by design).
struct ConfinedLogin {
    name: String,
    admin: Config,
    active: bool,
}

impl ConfinedLogin {
    const PASSWORD: &'static str = "readerconfined";

    fn create(base: &Config) -> Self {
        Self::create_for_role(base, "babylon_reader")
    }

    fn create_for_role(base: &Config, group: &str) -> Self {
        assert!(matches!(group, "babylon_reader" | "babylon_observer"));
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock is after the epoch")
            .as_nanos();
        let name = format!("per281_reader_login_{}_{unique:024x}", std::process::id());
        let mut admin = base.clone();
        admin.dbname("postgres");
        // The bounded startup options pin event_triggers=off; the runtime
        // hardening makes that parameter grant-only (see ScratchRole).
        admin
            .connect(NoTls)
            .expect("admin connection")
            .batch_execute(&format!(
                "CREATE ROLE {name} LOGIN PASSWORD '{}' \
                 NOSUPERUSER NOCREATEDB NOCREATEROLE; \
                 GRANT {group} TO {name}; \
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
    /// Clone the template and install Archive before creating the campaign.
    /// Then commit real ticks under retained coverage from foundation and install
    /// the confined reader role and tick-status view.
    fn create(label: &str, campaign_uuid: u128, tick_count: u64) -> Self {
        assert!(tick_count > 0);
        let base = validated_base_config();
        let template = validated_template_name();
        let database = TestDatabase::create_from_template(&base, &template, label);
        let config = database.config(&base);
        let store = SemanticArchiveStore::new(&config);
        store
            .verify_schema()
            .expect("Archive schema installs before foundation");
        let campaign_id = CampaignId::from_uuid(Uuid::from_u128(campaign_uuid));
        let foundation = current_material::foundation();
        let mut runtime = DurableMaterialRuntime::create(&config, campaign_id, foundation)
            .expect("runtime constructs after activation");
        for tick in 1..=tick_count {
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
        drop(runtime);

        assert_eq!(
            install_reader_role(&config).expect("reader role installs"),
            ReaderRoleDisposition::Installed
        );
        assert_eq!(
            install_reader_role(&config).expect("reader role reinstall reconciles"),
            ReaderRoleDisposition::AlreadyCurrent,
            "the exact role, view, and grants reinstall idempotently"
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

fn county_subject() -> ArchiveSubject {
    ArchiveSubject::try_new(
        ArchiveSubjectKind::County,
        "26163".to_owned(),
        "Wayne County".to_owned(),
    )
    .expect("county identity")
}

fn county_page_input(tick_content_hash: [u8; 32]) -> ArchivePageInput {
    ArchivePageInput::try_new(
        county_subject(),
        1,
        tick_content_hash,
        "Which neighboring place should organizers investigate next?".to_owned(),
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
    .expect("county page input")
}

/// Grant employment knowledge, then materialize the tick-one receipt so
/// scoped search has one known page to find. The county subject grant needs
/// no explicit insert: foundation seeding granted every real Michigan county
/// subject at tick zero, and a conflicting re-grant would refuse `GrantConflict`.
fn materialize_county_page(config: &Config, campaign_id: CampaignId, tick_content_hash: [u8; 32]) {
    let store = SemanticArchiveStore::new(config);
    let county_ref = ArchivePageRef::try_new(ArchiveSubjectKind::County, "26163".to_owned())
        .expect("county ref");
    store
        .grant_knowledge(
            campaign_id,
            &ArchiveKnowledgeGrant::try_new(
                county_ref,
                "employment".to_owned(),
                1,
                ArchiveCitation::try_new(
                    "reader-live-employment".to_owned(),
                    "employment@tick-1".to_owned(),
                )
                .expect("live grant citation"),
            )
            .expect("live knowledge grant"),
        )
        .expect("knowledge grant persists");
    let batch = ArchiveDirtyBatch::try_new(
        1,
        tick_content_hash,
        vec![county_page_input(tick_content_hash)],
    )
    .expect("live dirty batch");
    store
        .materialize_receipt(campaign_id, &batch, ArchiveMaterializeMode::Consume)
        .expect("live receipt materializes");
}

fn tick_one_content_hash(config: &Config, campaign_id: CampaignId) -> [u8; 32] {
    let receipt: Vec<u8> = config
        .connect(NoTls)
        .expect("receipt connection")
        .query_one(
            "SELECT tick_content_hash FROM babylon_state.archive_dirty_receipt_v1 \
             WHERE campaign_id = $1::uuid AND resolve_tick = 1",
            &[campaign_id.as_uuid()],
        )
        .expect("tick-one receipt query")
        .try_get(0)
        .expect("tick-one receipt digest");
    receipt.try_into().expect("exact receipt digest width")
}

/// Run as the database owner: the role holds SELECT only on the fog-safe
/// views and no table privilege anywhere else.
fn assert_owner_side_privilege_matrix(client: &mut postgres::Client) {
    let held = client
        .query(
            "SELECT relation::pg_catalog.text || ':' || privilege::pg_catalog.text \
             FROM (VALUES \
                 ('babylon_state.tick_commit'::pg_catalog.text), \
                 ('babylon_state.campaign'), \
                 ('babylon_meta.archive_page_revision_v2'), \
                 ('babylon_meta.archive_knowledge_grant_v1'), \
                 ('babylon_meta.archive_receipt_consumption_v1'), \
                 ('babylon_meta.archive_atom_v1'), \
                 ('babylon_meta.archive_revision_atom_v2'), \
                 ('babylon_meta.archive_revision_grant_v2'), \
                 ('babylon_meta.archive_tick_knowledge_v2'), \
                 ('babylon_meta.archive_tick_knowledge_member_v2'), \
                 ('babylon_meta.current_schema')) AS tables(relation) \
             CROSS JOIN (VALUES \
                 ('SELECT'::pg_catalog.text), ('INSERT'), ('UPDATE'), ('DELETE'), \
                 ('TRUNCATE'), ('REFERENCES'), ('TRIGGER')) AS privileges(privilege) \
             WHERE pg_catalog.has_table_privilege('babylon_reader', relation, privilege)",
            &[],
        )
        .expect("owner-side privilege matrix query")
        .iter()
        .map(|row| row.try_get::<_, String>(0).expect("matrix entry decodes"))
        .collect::<Vec<_>>();
    assert!(
        held.is_empty(),
        "babylon_reader must hold no base-table privileges, held={held:?}"
    );
    for view in [
        "public.v_committed_tick_status_v1",
        "public.v_archive_revision_known_v2",
        "public.v_archive_revision_index_v2",
        "public.v_archive_revision_atom_v2",
        "public.v_archive_revision_grant_v2",
        "public.v_archive_subject_grant_v2",
        "public.v_archive_tick_knowledge_v2",
        "public.v_archive_revision_scope_v2",
        "public.v_archive_verification_v1",
    ] {
        let view_select = client
            .query_one(
                "SELECT pg_catalog.has_table_privilege('babylon_reader', $1::text, 'SELECT')",
                &[&view],
            )
            .expect("view SELECT privilege query")
            .try_get::<_, bool>(0)
            .expect("view SELECT privilege decodes");
        assert!(view_select, "babylon_reader holds SELECT on {view}");
        let view_write = client
            .query_one(
                "SELECT pg_catalog.bool_or(priv) FROM (VALUES \
                     (pg_catalog.has_table_privilege('babylon_reader', $1::text, 'INSERT')), \
                     (pg_catalog.has_table_privilege('babylon_reader', $1::text, 'UPDATE')), \
                     (pg_catalog.has_table_privilege('babylon_reader', $1::text, 'DELETE')) \
                 ) AS writes(priv)",
                &[&view],
            )
            .expect("view write privilege query")
            .try_get::<_, bool>(0)
            .expect("view write privilege decodes");
        assert!(
            !view_write,
            "babylon_reader must not hold INSERT, UPDATE, or DELETE on {view}"
        );
    }
}

/// Every read a fog-safe reader must never reach, run under
/// `SET ROLE babylon_reader` on a superuser connection.
fn assert_reader_query_refusals(client: &mut postgres::Client) {
    for relation in [
        "archive_revision_grant_v2",
        "archive_tick_knowledge_v2",
        "archive_tick_knowledge_member_v2",
        "archive_atom_v1",
        "current_schema",
    ] {
        assert!(
            client
                .query(
                    &format!("SELECT count(*) FROM babylon_meta.{relation}"),
                    &[]
                )
                .is_err(),
            "reader must not access {relation}"
        );
    }
    for label_sql in [
        ("read the base tick_commit table", "SELECT pg_catalog.count(*) FROM babylon_state.tick_commit"),
        ("read the base campaign table", "SELECT pg_catalog.count(*) FROM babylon_state.campaign"),
        (
            "read archive pages",
            "SELECT pg_catalog.count(*) FROM babylon_meta.archive_page_revision_v2",
        ),
        (
            "read archive knowledge grants",
            "SELECT pg_catalog.count(*) FROM babylon_meta.archive_knowledge_grant_v1",
        ),
        (
            "read archive receipt consumption",
            "SELECT pg_catalog.count(*) FROM babylon_meta.archive_receipt_consumption_v1",
        ),
        (
            "read archive atoms",
            "SELECT pg_catalog.count(*) FROM babylon_meta.archive_atom_v1",
        ),
        (
            "read archive page atom composition",
            "SELECT pg_catalog.count(*) FROM babylon_meta.archive_revision_atom_v2",
        ),
        (
            "write through the tick-status view",
            "INSERT INTO public.v_committed_tick_status_v1 \
             (campaign_id, resolve_tick, envelope_layout_version, tick_content_hash, envelope_digest) \
             VALUES ('00000000-0000-0000-0000-000000000000', 1, 1, '\\x00'::bytea, '\\x00'::bytea)",
        ),
        (
            "write the base tick table",
            "INSERT INTO babylon_state.tick_commit \
             (campaign_id, resolve_tick, envelope_layout_version, tick_content_hash, envelope_digest) \
             VALUES ('00000000-0000-0000-0000-000000000000', 1, 1, '\\x00'::bytea, '\\x00'::bytea)",
        ),
        (
            "write archive knowledge grants",
            "INSERT INTO babylon_meta.archive_knowledge_grant_v1 \
             (campaign_id, subject_kind, subject_id, grant_key, granted_tick, \
              provenance_source_id, provenance_locator) \
             VALUES ('00000000-0000-0000-0000-000000000000', 'county', '26163', 'subject', 1, 's', 'l')",
        ),
    ] {
        let (label, sql) = label_sql;
        assert!(
            client.execute(sql, &[]).is_err(),
            "babylon_reader must not {label}"
        );
    }
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_reader_role_reads_the_view_and_refuses_every_base_relation() {
    let target = ReaderTarget::create(
        "readerrolepriv",
        0x2300_0000_0000_0000_0000_0000_0000_00a1,
        2,
    );

    let mut client = target
        .config
        .connect(NoTls)
        .expect("privilege probe connection");
    assert_owner_side_privilege_matrix(&mut client);
    client
        .batch_execute("SET ROLE babylon_reader")
        .expect("superuser delegates to the reader role");

    let row = client
        .query_one(
            "SELECT resolve_tick, envelope_layout_version, tick_content_hash, envelope_digest \
             FROM public.v_committed_tick_status_v1 \
             WHERE campaign_id = $1::uuid \
             ORDER BY resolve_tick DESC LIMIT 1",
            &[target.campaign_id.as_uuid()],
        )
        .expect("babylon_reader selects the committed-tick status view");
    let resolve_tick: i64 = row.try_get(0).expect("view resolve tick decodes");
    assert_eq!(
        resolve_tick, 2,
        "the view exposes the acknowledged commit tail"
    );

    let expected = target
        .config
        .connect(NoTls)
        .expect("owner probe connection")
        .query_one(
            "SELECT envelope_layout_version, tick_content_hash, envelope_digest \
             FROM babylon_state.tick_commit \
             WHERE campaign_id = $1::uuid AND resolve_tick = 2",
            &[target.campaign_id.as_uuid()],
        )
        .expect("owner reads the base marker row");
    let layout: i16 = row.try_get(1).expect("view layout decodes");
    assert_eq!(
        layout,
        expected.try_get::<_, i16>(0).expect("base layout decodes"),
        "the view preserves the envelope identity exactly"
    );
    let content_hash: Vec<u8> = row.try_get(2).expect("view content hash decodes");
    assert_eq!(
        content_hash,
        expected
            .try_get::<_, Vec<u8>>(1)
            .expect("base content hash decodes"),
        "the view preserves the tick content hash exactly"
    );
    let envelope_digest: Vec<u8> = row.try_get(3).expect("view envelope digest decodes");
    assert_eq!(
        envelope_digest,
        expected
            .try_get::<_, Vec<u8>>(2)
            .expect("base envelope digest decodes"),
        "the view preserves the envelope digest exactly"
    );

    assert_reader_query_refusals(&mut client);
    target.finish();
}

/// The confined search and dossier agree on the exact retained composition.
/// Its original atoms and changes remain cited; an ungranted county stays dark.
fn assert_confined_reader_search_and_card(
    reader: &SemanticArchiveReader,
    scope: &ArchiveReadScope,
) {
    let search = reader
        .search_as_of(scope, "728576", 10)
        .expect("confined scoped search");
    assert_eq!(search.scope, *scope);
    assert_eq!(search.state, ArchiveSearchState::Ready);
    assert!(!search.truncated);
    assert_eq!(search.hits.len(), 1);
    let hit = &search.hits[0];
    assert_eq!(hit.subject.kind(), ArchiveSubjectKind::County);
    assert_eq!(hit.subject.id(), "26163");
    assert_eq!(hit.title, "Wayne County");
    let card = read_county(reader, scope, "26163");
    assert_eq!(card.scope, *scope);
    let ArchiveDossierState::Ready {
        page,
        verified_through_tick,
    } = &card.state
    else {
        panic!("expected ready tick-one page: {:?}", card.state);
    };
    assert_eq!(*verified_through_tick, 1);
    assert_eq!(page.revision_id, hit.revision_id);
    assert_eq!(page.content_source, hit.content_source);
    assert_eq!(page.content_source, *scope);
    assert_eq!(page.content_sha256, sha256_of(page.markdown.as_bytes()));
    assert_first_county_revision_evidence(page);
    let dark = read_county(reader, scope, "99901");
    assert_eq!(
        dark.state,
        ArchiveDossierState::Unavailable(ArchiveDossierUnavailable::SubjectNotDisclosed)
    );
    assert!(reader
        .search_as_of(scope, "99901", 10)
        .unwrap()
        .hits
        .is_empty());
}

fn assert_first_county_revision_evidence(page: &ArchiveDossierPage) {
    assert!(page.markdown.contains("728576 jobs"));
    assert!(page.atoms.iter().any(|atom| atom.signal_key() == "subject"));
    let employment = page
        .atoms
        .iter()
        .filter(|atom| atom.signal_key() == "employment")
        .collect::<Vec<_>>();
    assert_eq!(employment.len(), 1);
    assert_eq!(employment[0].grant_key(), "employment");
    assert!(matches!(
        employment[0].value(),
        babylon_persistence::ArchiveAtomValue::Text(text) if text == "728576 jobs"
    ));
    assert_eq!(page.signals[0].label(), "Employment");
    assert_eq!(page.signals[0].citation().source_id(), "qcew-2024");
    assert_eq!(page.changes.coverage_from_tick, 0);
    assert!(page.changes.next_cursor.is_none());
    assert_eq!(page.changes.changes.len(), page.atoms.len());
    assert!(page
        .changes
        .changes
        .windows(2)
        .all(|pair| pair[0].signal_key < pair[1].signal_key));
    for change in &page.changes.changes {
        assert_eq!(change.publication_tick, 1);
        assert!(change.before.is_none());
        assert!(page
            .atoms
            .contains(change.after.as_ref().expect("first retained appearance")));
    }
}

fn read_county(
    reader: &SemanticArchiveReader,
    scope: &ArchiveReadScope,
    geoid: &str,
) -> ArchiveDossierRead {
    reader
        .dossier_as_of(
            scope,
            &ArchivePageRef::try_new(ArchiveSubjectKind::County, geoid.into()).unwrap(),
            &ArchiveDossierBounds::default(),
        )
        .expect("confined scoped dossier")
}

fn retained_page(read: &ArchiveDossierRead) -> &ArchiveDossierPage {
    match &read.state {
        ArchiveDossierState::Ready { page, .. }
        | ArchiveDossierState::Pending {
            page: Some(page), ..
        } => page,
        other => panic!("expected exact retained content: {other:?}"),
    }
}

struct Undrained;
impl babylon_persistence::ArchiveDossierProducer for Undrained {
    fn produce(
        &self,
        _campaign: Uuid,
        receipt: &babylon_persistence::PendingArchiveReceipt,
        _knowledge: &babylon_persistence::ArchiveKnowledge,
        _budget: usize,
    ) -> Result<
        babylon_persistence::ArchiveProducerOutcome,
        babylon_persistence::SemanticArchiveError,
    > {
        Ok(babylon_persistence::ArchiveProducerOutcome::new(
            ArchiveDirtyBatch::try_new(
                receipt.resolve_tick(),
                *receipt.tick_content_hash(),
                Vec::new(),
            )?,
            1,
        ))
    }
}

fn assert_quiet_receipt_verification(
    reader: &SemanticArchiveReader,
    target: &ReaderTarget,
    owner_tail: Vec<u8>,
) {
    let first = ArchiveReadScope::committed(
        target.campaign_id,
        1,
        tick_one_content_hash(&target.config, target.campaign_id),
    )
    .unwrap();
    let scope =
        ArchiveReadScope::committed(target.campaign_id, 2, owner_tail.try_into().unwrap()).unwrap();
    let before = read_county(reader, &first, "26163");
    let source = retained_page(&before).clone();
    assert_pending_retained_page(reader, &scope, &source);
    let pending = reader
        .archive_verification_status(target.campaign_id)
        .unwrap()
        .unwrap();
    assert_eq!((pending.durable_tick(), pending.processed_tick()), (2, 1));
    let mut worker = babylon_persistence::ArchiveWorker::new(&target.config);
    let staged = worker.sweep_once(target.campaign_id, &Undrained).unwrap();
    assert_eq!(staged.paged_count(), 1);
    assert_eq!(staged.verified_tick(), 1);
    assert_eq!(
        reader
            .archive_verification_status(target.campaign_id)
            .unwrap(),
        Some(pending)
    );
    assert_pending_retained_page(reader, &scope, &source);
    let settled = worker
        .sweep_once(
            target.campaign_id,
            &babylon_persistence::NullArchiveDossierProducer::new(),
        )
        .unwrap();
    assert_eq!(settled.applied_count(), 1);
    let verified = reader
        .archive_verification_status(target.campaign_id)
        .unwrap()
        .unwrap();
    assert_eq!((verified.durable_tick(), verified.processed_tick()), (2, 2));
    assert_quiet_retry(target, &scope);
    let current = read_county(reader, &scope, "26163");
    assert!(matches!(
        current.state,
        ArchiveDossierState::Ready {
            verified_through_tick: 2,
            ..
        }
    ));
    assert_eq!(
        retained_page(&current),
        &source,
        "quiet verification never changes content, atoms, links, changes or citations"
    );
    let historical = read_county(reader, &first, "26163");
    assert_eq!(historical.scope, before.scope);
    assert_eq!(historical.subject, before.subject);
    assert_eq!(historical.state, before.state);
    assert_eq!((historical.durable_tick, historical.processed_tick), (2, 2));
    assert_eq!(source.content_source.tick(), 1);
    let search = reader.search_as_of(&scope, "728576", 10).unwrap();
    assert_eq!(search.state, ArchiveSearchState::Ready);
    assert_eq!(search.hits[0].content_source, first);
    let mut restarted = babylon_persistence::ArchiveWorker::new(&target.config);
    let idle = restarted
        .sweep_once(
            target.campaign_id,
            &babylon_persistence::NullArchiveDossierProducer::new(),
        )
        .unwrap();
    assert!(idle.dispositions().is_empty());
    assert_eq!(idle.verified_tick(), 2);
    assert_eq!(
        reader
            .archive_verification_status(target.campaign_id)
            .unwrap(),
        Some(verified)
    );
    assert_eq!(read_county(reader, &scope, "26163"), current);
    assert_pin_worker_identity_refused(reader, target, &scope, &current);
}

fn assert_quiet_retry(target: &ReaderTarget, scope: &ArchiveReadScope) {
    let empty =
        ArchiveDirtyBatch::try_new(2, scope.tick_content_hash().unwrap(), Vec::new()).unwrap();
    let retry = SemanticArchiveStore::new(&target.config)
        .materialize_receipt(target.campaign_id, &empty, ArchiveMaterializeMode::Consume)
        .expect("exact quiet receipt retry");
    assert_eq!(
        retry.disposition(),
        babylon_persistence::ArchiveMaterializeDisposition::AlreadyConsumed
    );
    assert!(retry.pages().is_empty());
}

fn assert_pending_retained_page(
    reader: &SemanticArchiveReader,
    scope: &ArchiveReadScope,
    source: &ArchiveDossierPage,
) {
    let pending = read_county(reader, scope, "26163");
    assert_eq!(pending.scope, *scope);
    assert_eq!((pending.durable_tick, pending.processed_tick), (2, 1));
    assert!(matches!(
        pending.state,
        ArchiveDossierState::Pending {
            reason: ArchiveDossierPending::ReceiptProcessing,
            page: Some(_)
        }
    ));
    let page = retained_page(&pending);
    assert_eq!(page.content_source, source.content_source);
    assert_eq!(page.content_sha256, source.content_sha256);
    assert_eq!(page.markdown, source.markdown);
    assert_eq!(page.atoms, source.atoms);
    assert_eq!(page.citations, source.citations);
    assert!(
        page.changes.changes.is_empty(),
        "pending reads do not fabricate a closed historical change set"
    );
    assert!(page.changes.next_cursor.is_none());
    let search = reader.search_as_of(scope, "728576", 10).unwrap();
    assert_eq!(
        search.state,
        ArchiveSearchState::Pending(ArchiveDossierPending::ReceiptProcessing)
    );
    assert_eq!(search.hits.len(), 1);
    assert_eq!(search.hits[0].revision_id, source.revision_id);
    assert_eq!(search.hits[0].content_source, source.content_source);
}

fn assert_pin_worker_identity_refused(
    reader: &SemanticArchiveReader,
    target: &ReaderTarget,
    scope: &ArchiveReadScope,
    unchanged: &ArchiveDossierRead,
) {
    use babylon_persistence::{SemanticArchiveError, SemanticArchiveReaderError};
    let mut client = target.config.connect(NoTls).unwrap();
    let tick = i64::try_from(scope.tick()).unwrap();
    let original: Vec<u8> = client
        .query_one(
            "SELECT worker_contract_sha256 FROM babylon_meta.archive_tick_knowledge_v2 \
             WHERE campaign_id=$1 AND resolve_tick=$2",
            &[target.campaign_id.as_uuid(), &tick],
        )
        .unwrap()
        .get(0);
    let mut corrupt = original.clone();
    corrupt[0] ^= 1;
    let update = "UPDATE babylon_meta.archive_tick_knowledge_v2 SET worker_contract_sha256=$3 \
                  WHERE campaign_id=$1 AND resolve_tick=$2";
    assert_eq!(
        client
            .execute(update, &[target.campaign_id.as_uuid(), &tick, &corrupt])
            .unwrap(),
        1
    );
    let subject = ArchivePageRef::try_new(ArchiveSubjectKind::County, "26163".into()).unwrap();
    let failures = [
        reader
            .dossier_as_of(scope, &subject, &ArchiveDossierBounds::default())
            .map(|_| ()),
        reader.search_as_of(scope, "728576", 10).map(|_| ()),
    ];
    for result in failures {
        assert!(
            matches!(
                result,
                Err(SemanticArchiveReaderError::Archive(
                    SemanticArchiveError::StoredPageMismatch
                        | SemanticArchiveError::ReceiptConflict
                ))
            ),
            "{result:?}"
        );
    }
    assert_eq!(
        client
            .execute(update, &[target.campaign_id.as_uuid(), &tick, &original])
            .unwrap(),
        1
    );
    assert_eq!(&read_county(reader, scope, "26163"), unchanged);
    assert_eq!(
        reader.search_as_of(scope, "728576", 10).unwrap().state,
        ArchiveSearchState::Ready
    );
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_reader_handle_reads_through_confined_login_and_refuses_writer_authority() {
    let target = ReaderTarget::create(
        "readerhandler",
        0x2300_0000_0000_0000_0000_0000_0000_00a2,
        2,
    );
    let tick_one_hash = tick_one_content_hash(&target.config, target.campaign_id);
    materialize_county_page(&target.config, target.campaign_id, tick_one_hash);

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
    std::env::set_var(
        READER_DSN_ENV,
        login.dsn(&host, port, &target.database.name),
    );
    let reader = SemanticArchiveReader::from_env().expect("BABYLON_READER_DSN admits");
    std::env::remove_var(READER_DSN_ENV);

    let status = reader
        .committed_tick_status(target.campaign_id)
        .expect("confined login reads the committed-tick status")
        .expect("one committed tick exists");
    assert_eq!(status.resolve_tick(), 2);
    assert_eq!(status.campaign_id(), &target.campaign_id);
    let owner_tail: Vec<u8> = target
        .config
        .connect(NoTls)
        .expect("owner tail connection")
        .query_one(
            "SELECT tick_content_hash FROM babylon_state.tick_commit \
             WHERE campaign_id = $1::uuid AND resolve_tick = 2",
            &[target.campaign_id.as_uuid()],
        )
        .expect("owner tail query")
        .try_get(0)
        .expect("owner tail digest");
    assert_eq!(
        status.tick_content_hash()[..],
        owner_tail[..],
        "the reader status preserves the acknowledged commit tail hash exactly"
    );

    let first = ArchiveReadScope::committed(target.campaign_id, 1, tick_one_hash).unwrap();
    assert_confined_reader_search_and_card(&reader, &first);

    assert_quiet_receipt_verification(&reader, &target, owner_tail);

    // The owner credential carries writer authority: the handle must refuse
    // to operate, not silently read with owner powers.
    let owner_dsn = format!(
        "postgresql://test:test@{host}:{port}/{}",
        target.database.name
    );
    std::env::set_var(READER_DSN_ENV, &owner_dsn);
    let owner_reader = SemanticArchiveReader::from_env().expect("loopback owner DSN admits");
    std::env::remove_var(READER_DSN_ENV);
    let refused = owner_reader.committed_tick_status(target.campaign_id);
    match refused {
        Err(babylon_persistence::SemanticArchiveReaderError::WriterAuthorityRefused(held)) => {
            assert!(
                !held.is_empty(),
                "the refusal must carry the observed census"
            );
        }
        other => panic!("owner credentials must refuse with writer authority, got {other:?}"),
    }

    login.cleanup();
    target.finish();
}

fn reader_installation_catalog(client: &mut postgres::Client) -> Vec<(String, String)> {
    client
        .query(
            "SELECT namespace.nspname || '.' || relation.relname, \
                    pg_catalog.jsonb_build_object(\
                        'kind', relation.relkind, 'owner', relation.relowner, \
                        'acl', relation.relacl, \
                        'view', CASE WHEN relation.relkind = 'v' \
                            THEN pg_catalog.pg_get_viewdef(relation.oid, true) \
                            ELSE '' END)::text \
             FROM pg_catalog.pg_class AS relation \
             JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace \
             WHERE namespace.nspname IN ('babylon_ref', 'babylon_state', 'babylon_meta', 'public') \
             ORDER BY namespace.nspname, relation.relname",
            &[],
        )
        .expect("reader provisioning catalog snapshot")
        .into_iter()
        .map(|row| (row.get(0), row.get(1)))
        .collect()
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_reader_installer_refuses_privilege_drift_and_view_identity_mismatch() {
    use babylon_persistence::{CurrentSchemaError, SemanticArchiveReaderError};

    let target = ReaderTarget::create(
        "readerroledrift",
        0x2300_0000_0000_0000_0000_0000_0000_00a3,
        1,
    );
    let mut client = target
        .config
        .connect(NoTls)
        .expect("drift probe connection");
    let refusal = Err(SemanticArchiveReaderError::CurrentSchema(
        CurrentSchemaError::CurrentCensusMismatch,
    ));
    for relation in ["archive_page_revision_v2", "archive_atom_v1"] {
        client
            .batch_execute(&format!(
                "GRANT SELECT ON babylon_meta.{relation} TO babylon_reader"
            ))
            .expect("extra base-table grant applies");
        let before = reader_installation_catalog(&mut client);
        assert_eq!(install_reader_role(&target.config).map(|_| ()), refusal);
        assert_eq!(
            reader_installation_catalog(&mut client),
            before,
            "schema refusal must leave the excessive grant and every catalog relation unchanged"
        );
        client
            .batch_execute(&format!(
                "REVOKE SELECT ON babylon_meta.{relation} FROM babylon_reader"
            ))
            .expect("test restores the exact grant footprint");
        assert_eq!(
            install_reader_role(&target.config),
            Ok(ReaderRoleDisposition::AlreadyCurrent),
            "restoring the exact footprint permits idempotent provisioning"
        );
    }

    client
        .batch_execute(
            "DROP VIEW public.v_committed_tick_status_v1; \
             CREATE TABLE public.v_committed_tick_status_v1(id bigint)",
        )
        .expect("impostor table replaces the view");
    let impostor = reader_installation_catalog(&mut client);
    assert_eq!(install_reader_role(&target.config).map(|_| ()), refusal);
    assert_eq!(
        reader_installation_catalog(&mut client),
        impostor,
        "a same-named table is refused without replacement or grant changes"
    );
    client
        .batch_execute("DROP TABLE public.v_committed_tick_status_v1")
        .expect("test removes the impostor table");
    let absent = reader_installation_catalog(&mut client);
    assert_eq!(install_reader_role(&target.config).map(|_| ()), refusal);
    assert_eq!(
        reader_installation_catalog(&mut client),
        absent,
        "a missing mandatory view is refused without schema repair"
    );
    target.finish();
}

fn assert_economic_grant_boundary(
    config: &Config,
    observer_config: &Config,
    observer: &ObserverEconomyReader,
    preview: &ObserverEconomyReader,
    campaign: CampaignId,
) {
    let owner_reader =
        ObserverEconomyReader::connect(config, ObserverVisibility::FullObserver).unwrap();
    assert_eq!(
        owner_reader.snapshot(campaign, 1),
        Err(ObserverEconomyError::Authority)
    );
    config.connect(NoTls).unwrap().execute("DELETE FROM babylon_meta.archive_knowledge_grant_v1 WHERE campaign_id = $1 AND subject_kind = 'county' AND subject_id = '26163' AND grant_key = 'qcew-employment'", &[campaign.as_uuid()]).unwrap();
    let known = preview
        .snapshot(campaign, 1)
        .expect("individually grant-filtered snapshot");
    let hidden = known
        .counties
        .iter()
        .find(|row| row.county_geoid == "26163")
        .unwrap();
    assert_eq!(hidden.annual_avg_emplvl, None);
    assert_eq!(hidden.annual_avg_wkly_wage, Some(1469));
    assert_eq!(
        observer
            .snapshot(campaign, 1)
            .unwrap()
            .counties
            .iter()
            .find(|row| row.county_geoid == "26163")
            .unwrap()
            .annual_avg_emplvl,
        Some(725_504)
    );
    let ungranted = observer_config
        .connect(NoTls)
        .unwrap()
        .query("SELECT * FROM public.v_known_county_economy_v1", &[]);
    assert!(
        ungranted.is_err(),
        "observer credential is not a preview credential"
    );
}

#[test]
#[ignore = "requires task-owned disposable PostgreSQL runtime template"]
fn live_observer_economics_reads_exact_foundation_commit_and_granted_preview() {
    let base = validated_base_config();
    let database =
        TestDatabase::create_from_template(&base, &validated_template_name(), "observereconomics");
    let config = database.config(&base);
    let campaign =
        CampaignId::from_uuid(Uuid::from_u128(0x3190_0000_0000_0000_0000_0000_0000_0001));
    let foundation = current_material::foundation();
    let mut runtime =
        DurableMaterialRuntime::create(&config, campaign, foundation).expect("observer campaign");
    SemanticArchiveStore::new(&config)
        .verify_schema()
        .expect("Archive schema");
    install_reader_role(&config).expect("reader role");
    provision_observer_role(&config).expect("economic views and groups");
    provision_observer_role(&config).expect("idempotent exact observer schema");
    let observer_login = ConfinedLogin::create_for_role(&base, "babylon_observer");
    let known_login = ConfinedLogin::create(&base);
    let mut observer_config = config.clone();
    observer_config
        .user(&observer_login.name)
        .password(ConfinedLogin::PASSWORD);
    let mut known_config = config.clone();
    known_config
        .user(&known_login.name)
        .password(ConfinedLogin::PASSWORD);
    let observer =
        ObserverEconomyReader::connect(&observer_config, ObserverVisibility::FullObserver).unwrap();
    let preview =
        ObserverEconomyReader::connect(&known_config, ObserverVisibility::KnownPreview).unwrap();
    let foundation = observer
        .snapshot(campaign, 0)
        .expect("true foundation, without hidden tick");
    assert_eq!(foundation.counties.len(), 83);
    assert_eq!(foundation.tick_content_hash, None);
    assert_eq!(runtime.tail(), None);
    assert_eq!(
        preview.snapshot(campaign, 0).unwrap().counties,
        foundation.counties
    );
    let actions = OrderedPracticeActionBatch::empty(
        runtime.session().graph_session().session_identity().clone(),
        1,
    )
    .unwrap();
    let receipt = runtime
        .advance_and_commit(&mut CollectingSink::default(), &actions)
        .expect("actual quiet commit");
    let committed = observer
        .snapshot(campaign, 1)
        .expect("exact committed baseline");
    assert_eq!(committed.counties, foundation.counties);
    assert_eq!(committed.resolve_tick, receipt.resolve_tick());
    assert!(committed.tick_content_hash.is_some());
    assert_eq!(
        observer.snapshot(campaign, 2),
        Err(ObserverEconomyError::TickAbsent)
    );
    assert_economic_grant_boundary(&config, &observer_config, &observer, &preview, campaign);
    let other = CampaignId::from_uuid(Uuid::from_u128(0x3190_0000_0000_0000_0000_0000_0000_0002));
    let foundation = current_material::foundation();
    let other_runtime = DurableMaterialRuntime::create(&config, other, foundation)
        .expect("distinct campaign with the same admitted foundation");
    let other_snapshot = observer
        .snapshot(other, 0)
        .expect("second current campaign has its own foundation scope");
    assert_eq!(other_snapshot.campaign_id, other.as_uuid().to_string());
    assert_eq!(other_snapshot.resolve_tick, 0);
    assert_eq!(other_snapshot.tick_content_hash, None);
    assert_eq!(other_runtime.tail(), None);
    assert_eq!(other_snapshot.counties, committed.counties);
    assert_eq!(
        preview.snapshot(other, 0).unwrap().counties,
        other_snapshot.counties,
        "the first campaign's withheld grant cannot hide the second campaign's foundation field"
    );
    assert_eq!(
        observer.snapshot(other, 1),
        Err(ObserverEconomyError::TickAbsent),
        "the first campaign's committed tick cannot satisfy the second campaign's scope"
    );
    assert_eq!(
        observer.snapshot(campaign, 1).unwrap(),
        committed,
        "creating another campaign preserves the first campaign's exact committed observation"
    );
    drop(other_runtime);
    drop(runtime);
    observer_login.cleanup();
    known_login.cleanup();
    database.cleanup();
}

fn assert_material_lock_refusal(
    config: &Config,
    owner: &mut postgres::Client,
    campaign: CampaignId,
    runtime: &mut DurableMaterialRuntime,
    actions: &OrderedPracticeActionBatch,
    sink: &mut CollectingSink,
) {
    let before = runtime.session().current_world_hash().unwrap();
    let opening = runtime.session().material().canonical_bytes().to_vec();
    // Negative control self-releases after ten seconds: an unbounded writer
    // would eventually commit and fail the assertion instead of hanging CI.
    let lock_config = config.clone();
    let (locked_tx, locked_rx) = std::sync::mpsc::sync_channel(1);
    let (release_tx, release_rx) = std::sync::mpsc::sync_channel(1);
    let lock_holder = std::thread::spawn(move || {
        let mut connection = lock_config.connect(NoTls).unwrap();
        let mut transaction = connection.transaction().unwrap();
        transaction
            .query_one(
                "SELECT campaign_id FROM babylon_state.material_campaign_foundation_v2 \
             WHERE campaign_id=$1::uuid FOR UPDATE",
                &[campaign.as_uuid()],
            )
            .unwrap();
        locked_tx.send(()).unwrap();
        let _ = release_rx.recv_timeout(std::time::Duration::from_secs(10));
        transaction.rollback().unwrap();
    });
    locked_rx
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    let locked_result = runtime.advance_and_commit(sink, actions);
    let _ = release_tx.send(());
    lock_holder.join().unwrap();
    assert!(matches!(
        locked_result,
        Err(MaterialRuntimeError::DatabaseLockRefused(_))
    ));
    assert_eq!(runtime.session().completed_tick(), 0);
    assert_eq!(runtime.session().graph_session().completed_tick(), 0);
    assert_eq!(runtime.session().current_world_hash().unwrap(), before);
    assert_eq!(runtime.session().material().canonical_bytes(), opening);
    assert!(runtime.tail().is_none());
    assert!(sink.events.is_empty());
    let durable: i64 = owner
        .query_one(
            "SELECT count(*) FROM babylon_state.tick_commit WHERE campaign_id=$1::uuid",
            &[campaign.as_uuid()],
        )
        .unwrap()
        .get(0);
    assert_eq!(durable, 0);
}

fn assert_material_marker_rollback(
    owner: &mut postgres::Client,
    campaign: CampaignId,
    runtime: &mut DurableMaterialRuntime,
    actions: &OrderedPracticeActionBatch,
    sink: &mut CollectingSink,
) {
    let before = runtime.session().current_world_hash().unwrap();
    let opening = runtime.session().material().canonical_bytes().to_vec();
    owner.batch_execute("CREATE FUNCTION public.refuse_material_marker_v3() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'injected pre-marker refusal'; END $$; CREATE TRIGGER refuse_material_marker_v3 BEFORE INSERT ON babylon_state.tick_commit FOR EACH ROW EXECUTE FUNCTION public.refuse_material_marker_v3()").unwrap();
    assert!(runtime.advance_and_commit(sink, actions).is_err());
    assert_eq!(runtime.session().completed_tick(), 0);
    assert_eq!(runtime.session().graph_session().completed_tick(), 0);
    assert_eq!(runtime.session().current_world_hash().unwrap(), before);
    assert_eq!(runtime.session().material().canonical_bytes(), opening);
    assert!(sink.events.is_empty());
    for table in [
        "tick_commit",
        "material_tick_v3",
        "world_register_v1",
        "archive_dirty_receipt_v1",
    ] {
        let count: i64 = owner
            .query_one(
                &format!("SELECT count(*) FROM babylon_state.{table} WHERE campaign_id=$1::uuid"),
                &[campaign.as_uuid()],
            )
            .unwrap()
            .get(0);
        assert_eq!(count, 0, "rollback {table}");
    }
    owner.batch_execute("DROP TRIGGER refuse_material_marker_v3 ON babylon_state.tick_commit; DROP FUNCTION public.refuse_material_marker_v3()").unwrap();
}

fn assert_committed_material_visibility(
    observer: &ObserverEconomyReader,
    known: &ObserverEconomyReader,
    campaign: CampaignId,
    first: &IdentifiedMaterialTick,
) {
    let snapshot = observer.snapshot(campaign, 1).unwrap();
    assert_eq!(
        snapshot.nominal_world_hash,
        Some(babylon_tick::hex(&first.result_world_hash()))
    );
    assert_ne!(snapshot.nominal_world_hash, snapshot.tick_content_hash);
    assert!(known
        .snapshot(campaign, 1)
        .unwrap()
        .nominal_world_hash
        .is_none());
    assert!(snapshot.production.is_some());
    assert!(known.snapshot(campaign, 1).unwrap().production.is_none());
}

fn assert_material_restart_reconciliation(
    config: &Config,
    campaign: CampaignId,
    runtime: &mut DurableMaterialRuntime,
    sink: &mut CollectingSink,
) -> (DurableMaterialRuntime, IdentifiedMaterialTick) {
    let mut reopened = DurableMaterialRuntime::open(
        config,
        campaign,
        MichiganContentPreset::new_campaign(MichiganDeliveryPreset::Standard)
            .create_foundation(&crate::test_support::catalog())
            .unwrap()
            .digest(),
    )
    .unwrap();
    assert_eq!(
        reopened.session().material().canonical_bytes(),
        runtime.session().material().canonical_bytes()
    );
    assert_eq!(
        reopened.session().current_world_hash().unwrap(),
        runtime.session().current_world_hash().unwrap()
    );
    let second_actions = OrderedPracticeActionBatch::empty(
        runtime.session().graph_session().session_identity().clone(),
        2,
    )
    .unwrap();
    let uninterrupted = runtime.advance_and_commit(sink, &second_actions).unwrap();
    let reconciled = reopened
        .advance_and_commit(&mut CollectingSink::default(), &second_actions)
        .unwrap();
    assert_eq!(uninterrupted, reconciled);
    assert!(matches!(
        DurableMaterialRuntime::open(
            config,
            campaign,
            MichiganContentPreset::new_campaign(MichiganDeliveryPreset::Delayed)
                .create_foundation(&crate::test_support::catalog())
                .unwrap()
                .digest()
        ),
        Err(MaterialRuntimeError::FoundationMismatch)
    ));
    (reopened, uninterrupted)
}

fn assert_material_corruption_refused(
    owner: &mut postgres::Client,
    config: &Config,
    campaign: CampaignId,
    observer: &ObserverEconomyReader,
) {
    let exact:Vec<u8>=owner.query_one("SELECT register_bytes FROM babylon_state.material_tick_v3 WHERE campaign_id=$1::uuid AND resolve_tick=2",&[campaign.as_uuid()]).unwrap().get(0);
    let mut corrupted = exact.clone();
    let last = corrupted.len() - 1;
    corrupted[last] ^= 1;
    owner.execute("UPDATE babylon_state.material_tick_v3 SET register_bytes=$2 WHERE campaign_id=$1::uuid AND resolve_tick=2",&[campaign.as_uuid(),&corrupted]).unwrap();
    assert!(observer.snapshot(campaign, 2).is_err());
    assert!(DurableMaterialRuntime::open(
        config,
        campaign,
        MichiganContentPreset::new_campaign(MichiganDeliveryPreset::Standard)
            .create_foundation(&crate::test_support::catalog())
            .unwrap()
            .digest()
    )
    .is_err());
    owner.execute("UPDATE babylon_state.material_tick_v3 SET register_bytes=$2 WHERE campaign_id=$1::uuid AND resolve_tick=2",&[campaign.as_uuid(),&exact]).unwrap();
}

fn assert_material_stdio_advance(
    config: &Config,
    campaign: CampaignId,
    observer: &ObserverEconomyReader,
    uninterrupted: &IdentifiedMaterialTick,
) {
    let scope = RuntimeSessionScope {
        epoch: 1,
        campaign_id: Some(campaign.as_uuid().to_string()),
    };
    let open = RuntimeSessionRequest::Switch {
        protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION,
        request_id: 6,
        scope: RuntimeSessionScope {
            epoch: 0,
            campaign_id: None,
        },
        target: RuntimeSessionTarget::Open {
            campaign_id: campaign.as_uuid().to_string(),
        },
    };
    let request = RuntimeSessionRequest::Advance {
        protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION,
        scope: scope.clone(),
        request_id: 7,
        expected_tail: RuntimeSessionTail {
            resolve_tick: 2,
            tick_content_hash: Some(babylon_tick::hex(
                uninterrupted.tick_content_hash().as_bytes(),
            )),
        },
    };
    let stop = RuntimeSessionRequest::Stop {
        protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION,
        scope,
        request_id: 8,
    };
    let mut input = serde_json::to_vec(&open).unwrap();
    input.push(b'\n');
    input.extend(serde_json::to_vec(&request).unwrap());
    input.push(b'\n');
    input.extend(serde_json::to_vec(&stop).unwrap());
    input.push(b'\n');
    let mut output = Vec::new();
    run_runtime_session(
        config,
        std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../content/scenarios/michigan/defines.toml"
        )),
        std::io::Cursor::new(input),
        &mut output,
    )
    .unwrap();
    let responses: Vec<RuntimeSessionResponse> = output
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).unwrap())
        .collect();
    assert!(
        matches!(&responses[0],RuntimeSessionResponse::Hello{scope,..} if scope.epoch==0 && scope.campaign_id.is_none())
    );
    assert!(matches!(&responses[2],RuntimeSessionResponse::Ready{tail,..} if tail.resolve_tick==2));
    assert!(responses.iter().any(|response| matches!(
        response,
        RuntimeSessionResponse::Committed { request_id: 7, tail, .. }
            if tail.resolve_tick == 3
    )));
    let mut acknowledged_tick = 2;
    for response in &responses {
        match response {
            RuntimeSessionResponse::Ready { tail, .. }
            | RuntimeSessionResponse::Committed { tail, .. } => {
                acknowledged_tick = tail.resolve_tick;
            }
            RuntimeSessionResponse::ArchiveProgress { durable_tick, .. } => {
                assert_eq!(*durable_tick, acknowledged_tick);
            }
            RuntimeSessionResponse::Hello { .. }
            | RuntimeSessionResponse::Switching { .. }
            | RuntimeSessionResponse::Error { .. }
            | RuntimeSessionResponse::Stopped { .. } => {}
        }
    }
    assert_eq!(observer.snapshot(campaign, 3).unwrap().resolve_tick, 3);
}

#[test]
#[ignore = "requires task-owned disposable PostgreSQL runtime template"]
fn live_material_runtime_atomic_restart_identity_and_observer_projection() {
    let base = validated_base_config();
    let database =
        TestDatabase::create_from_template(&base, &validated_template_name(), "materialruntime");
    let config = database.config(&base);
    let campaign =
        CampaignId::from_uuid(Uuid::from_u128(0x3190_0000_0000_0000_0000_0000_0000_0003));
    let preset = MichiganDeliveryPreset::Standard;
    let foundation = MichiganContentPreset::new_campaign(preset)
        .create_foundation(&crate::test_support::catalog())
        .unwrap();
    let digest = foundation.digest();
    let mut runtime = DurableMaterialRuntime::create(&config, campaign, foundation).unwrap();
    install_reader_role(&config).unwrap();
    provision_observer_role(&config).unwrap();
    let observer_login = ConfinedLogin::create_for_role(&base, "babylon_observer");
    let known_login = ConfinedLogin::create(&base);
    let mut observer_config = config.clone();
    observer_config
        .user(&observer_login.name)
        .password(ConfinedLogin::PASSWORD);
    let mut known_config = config.clone();
    known_config
        .user(&known_login.name)
        .password(ConfinedLogin::PASSWORD);
    let observer =
        ObserverEconomyReader::connect(&observer_config, ObserverVisibility::FullObserver).unwrap();
    let known =
        ObserverEconomyReader::connect(&known_config, ObserverVisibility::KnownPreview).unwrap();
    let zero = observer.snapshot(campaign, 0).unwrap();
    assert!(zero.production.is_some());
    assert!(known.snapshot(campaign, 0).unwrap().production.is_none());
    assert_eq!(zero.foundation_digest, babylon_tick::hex(&digest));
    let mut owner = config.connect(NoTls).unwrap();
    let actions = OrderedPracticeActionBatch::empty(
        runtime.session().graph_session().session_identity().clone(),
        1,
    )
    .unwrap();
    let mut sink = CollectingSink::default();
    assert_material_lock_refusal(
        &config,
        &mut owner,
        campaign,
        &mut runtime,
        &actions,
        &mut sink,
    );
    assert_material_marker_rollback(&mut owner, campaign, &mut runtime, &actions, &mut sink);
    let first = runtime.advance_and_commit(&mut sink, &actions).unwrap();
    assert_eq!(first.resolve_tick(), 1);
    assert_committed_material_visibility(&observer, &known, campaign, &first);
    let (reopened, uninterrupted) =
        assert_material_restart_reconciliation(&config, campaign, &mut runtime, &mut sink);
    assert_material_corruption_refused(&mut owner, &config, campaign, &observer);
    assert_material_stdio_advance(&config, campaign, &observer, &uninterrupted);
    drop(owner);
    drop(runtime);
    drop(reopened);
    observer_login.cleanup();
    known_login.cleanup();
    database.cleanup();
}

#[path = "support/material_config.rs"]
mod test_support;
