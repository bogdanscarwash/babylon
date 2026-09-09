//! Full material reads, historical identity, and SQL-denied preview on a disposable clone.

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_persistence::archive_revision::{
    ArchiveDossierBoundsV2, ArchiveDossierPageV2, ArchiveDossierPendingV2, ArchiveDossierReadV2,
    ArchiveDossierStateV2, ArchiveDossierUnavailableV2, ArchiveReadScopeV2, ArchiveSearchStateV2,
};
use babylon_persistence::{
    install_observer_economy_schema_v1, install_reader_role_v1,
    material_runtime::DurableMaterialRuntimeV3,
    michigan_content::MichiganContentPresetV1,
    michigan_economy::{
        michigan_economy_v1, MichiganCountyEconomyV1, QCEW_ECONOMICS_ARTIFACT_SHA256_V1,
        QCEW_ECONOMICS_FIELD_KEYS_V1, QCEW_ECONOMICS_SOURCE_ID_V1,
    },
    michigan_material::MichiganDeliveryPresetV1,
    observer_reader::{ObserverEconomyErrorV1, ObserverEconomyReaderV1, ObserverVisibilityV1},
    validate_connection_target, ArchiveAtomSubjectKindV1, ArchiveAtomValueV1,
    ArchiveEvidenceClassV1, ArchivePageRefV1, ArchiveReceiptDispositionV1, ArchiveSubjectKindV1,
    ArchiveWorkerV1, CampaignId, CompositeArchiveDossierProducerV1, CountyDossierProducerV1,
    PlaceDossierProducerV1, SemanticArchiveReaderV1, SemanticArchiveStoreV1,
};
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use babylon_tick::material_world::MaterialWorldRegisterV2;
use postgres::{Config, NoTls};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use uuid::Uuid;

const ACK: &str = "I_UNDERSTAND_THIS_DISPOSABLE_RUNTIME_DROPS_ITS_SCRATCH_DATABASES_AND_ROLES";

static NEXT_DISPOSABLE_TARGET: AtomicU64 = AtomicU64::new(0);
// PostgreSQL parameter ACLs are cluster-wide even when test databases differ.
// The owned harness runs this test binary alone; only these short shared-row
// mutations serialize, while each clone's material/Archive proof runs freely.
static PARAMETER_ACL_WRITE: Mutex<()> = Mutex::new(());

struct DisposableTarget {
    sequence: u64,
    admin: Config,
    writer: Config,
    database: String,
    roles: Vec<String>,
}

fn michigan_archive_producer(config: &Config) -> CompositeArchiveDossierProducerV1 {
    CompositeArchiveDossierProducerV1::new(vec![
        Box::new(CountyDossierProducerV1::try_new(config).unwrap()),
        Box::new(PlaceDossierProducerV1::try_new(config).unwrap()),
    ])
}

fn advance_material_period(runtime: &mut DurableMaterialRuntimeV3) {
    let tick = runtime.session().completed_tick() + 1;
    let actions = OrderedPracticeActionBatchV1::empty(
        runtime.session().graph_session().session_identity().clone(),
        tick,
    )
    .unwrap();
    runtime
        .advance_and_commit(&mut CollectingSink::default(), &actions)
        .unwrap();
}

fn assert_archive_progress(
    reader: &SemanticArchiveReaderV1,
    campaign: CampaignId,
    durable: u64,
    processed: u64,
) {
    let status = reader
        .archive_verification_status(campaign)
        .unwrap()
        .unwrap();
    assert_eq!(status.durable_tick(), durable);
    assert_eq!(status.processed_tick(), processed);
    let committed = reader.committed_tick_status(campaign).unwrap().unwrap();
    assert_eq!(*committed.campaign_id(), campaign);
    assert_eq!(committed.resolve_tick(), durable);
}

fn assert_public_qcew_card(
    page: &ArchiveDossierPageV2,
    campaign: CampaignId,
    county: &MichiganCountyEconomyV1,
) {
    let expected = [
        (
            "qcew-establishments",
            "QCEW 2024 annual-average establishments",
            county.annual_avg_estabs_count,
        ),
        (
            "qcew-employment",
            "QCEW 2024 annual-average employment (jobs)",
            county.annual_avg_emplvl,
        ),
        (
            "qcew-total-annual-wages",
            "QCEW 2024 total annual wages (USD)",
            county.total_annual_wages,
        ),
        (
            "qcew-average-weekly-wage",
            "QCEW 2024 average weekly wage (USD/week)",
            county.annual_avg_wkly_wage,
        ),
    ];
    let atoms = &page.atoms;
    let signals: Vec<_> = atoms
        .iter()
        .filter(|atom| QCEW_ECONOMICS_FIELD_KEYS_V1.contains(&atom.signal_key()))
        .collect();
    assert_eq!(signals.len(), 4);
    assert_eq!(
        page.content_source.tick(),
        1,
        "content remains sourced from period one"
    );
    assert_eq!(page.content_source.campaign_id(), campaign);
    assert_eq!(
        page.content_sha256,
        babylon_kernel::sha256_of(page.markdown.as_bytes())
    );
    let locator = format!(
        "qcew_county_economics_mi_2024.csv.gz#county_geoid={}&sha256={QCEW_ECONOMICS_ARTIFACT_SHA256_V1}",
        county.county_geoid
    );
    for (key, label, value) in expected {
        let atom = signals
            .iter()
            .find(|atom| atom.signal_key() == key)
            .unwrap();
        assert_eq!(*atom.campaign_id(), campaign);
        assert_eq!(atom.subject().kind(), ArchiveAtomSubjectKindV1::County);
        assert_eq!(atom.subject().id(), county.county_geoid);
        assert_eq!(atom.grant_key(), key);
        assert_eq!(atom.evidence_class(), ArchiveEvidenceClassV1::Observed);
        assert_eq!(atom.value(), &ArchiveAtomValueV1::Text(value.to_string()));
        assert_eq!(atom.valid_tick(), 1);
        assert_eq!(atom.citation().source_id(), QCEW_ECONOMICS_SOURCE_ID_V1);
        assert_eq!(atom.citation().locator(), locator);
        assert!(page.citations.contains(atom.citation()));
        assert!(page.markdown.contains(&format!("- **{label}:** {value} —")));
    }
    assert!(atoms
        .iter()
        .all(|atom| !matches!(atom.signal_key(), "median-wage" | "production" | "phi-hour")));
}

fn current_archive_scope(
    reader: &SemanticArchiveReaderV1,
    campaign: CampaignId,
) -> ArchiveReadScopeV2 {
    let status = reader.committed_tick_status(campaign).unwrap().unwrap();
    assert_eq!(*status.campaign_id(), campaign);
    ArchiveReadScopeV2::committed(campaign, status.resolve_tick(), *status.tick_content_hash())
        .unwrap()
}

fn read_county(
    reader: &SemanticArchiveReaderV1,
    scope: &ArchiveReadScopeV2,
    geoid: &str,
) -> ArchiveDossierReadV2 {
    reader
        .dossier_as_of(
            scope,
            &ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, geoid.to_owned()).unwrap(),
            &ArchiveDossierBoundsV2::try_new(100, None).unwrap(),
        )
        .unwrap()
}

fn assert_scoped_page<'a>(
    read: &'a ArchiveDossierReadV2,
    scope: &ArchiveReadScopeV2,
    expected: ArchiveSearchStateV2,
) -> &'a ArchiveDossierPageV2 {
    assert_eq!(&read.scope, scope);
    assert_eq!(read.history_floor_tick, 0);
    match (&read.state, expected) {
        (
            ArchiveDossierStateV2::Ready {
                page,
                verified_through_tick,
            },
            ArchiveSearchStateV2::Ready,
        ) => {
            assert_eq!(*verified_through_tick, scope.tick());
            page
        }
        (
            ArchiveDossierStateV2::Pending {
                page: Some(page),
                reason,
            },
            ArchiveSearchStateV2::Pending(expected),
        ) => {
            assert_eq!(*reason, expected);
            assert!(page.changes.changes.is_empty());
            assert!(page.changes.next_cursor.is_none());
            page
        }
        other => panic!("unexpected scoped page state: {other:?}"),
    }
}

fn assert_all_county_cards(
    reader: &SemanticArchiveReaderV1,
    scope: &ArchiveReadScopeV2,
    expected: ArchiveSearchStateV2,
) -> Vec<ArchiveDossierPageV2> {
    let search = reader.search_as_of(scope, "QCEW 2024", 100).unwrap();
    assert_eq!(&search.scope, scope);
    assert_eq!(search.state, expected);
    assert!(!search.truncated);
    assert_eq!(search.hits.len(), 83);
    search
        .hits
        .iter()
        .zip(michigan_economy_v1().unwrap().counties())
        .map(|(hit, county)| {
            assert_eq!(hit.subject.kind(), ArchiveSubjectKindV1::County);
            assert_eq!(hit.subject.id(), county.county_geoid);
            let read = read_county(reader, scope, &county.county_geoid);
            assert_eq!(read.subject, hit.subject);
            let page = assert_scoped_page(&read, scope, expected);
            assert_eq!(page.revision_id, hit.revision_id);
            assert_eq!(page.content_source, hit.content_source);
            assert_eq!(page.title, hit.title);
            assert_public_qcew_card(page, scope.campaign_id(), county);
            page.clone()
        })
        .collect()
}

fn assert_retained_content(actual: &ArchiveDossierPageV2, original: &ArchiveDossierPageV2) {
    assert_eq!(actual.revision_id, original.revision_id);
    assert_eq!(actual.effective_tick, original.effective_tick);
    assert_eq!(actual.origin, original.origin);
    assert_eq!(actual.content_source, original.content_source);
    assert_eq!(actual.title, original.title);
    assert_eq!(actual.question, original.question);
    assert_eq!(actual.signals, original.signals);
    assert_eq!(actual.markdown, original.markdown);
    assert_eq!(actual.content_sha256, original.content_sha256);
    assert_eq!(actual.citations, original.citations);
    assert_eq!(actual.atoms, original.atoms);
    // Target readiness and history coverage are scoped observations, not source bytes.
    let labels = |page: &ArchiveDossierPageV2| {
        page.links
            .iter()
            .map(|link| (link.target.clone(), link.retained_label.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(labels(actual), labels(original));
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed Michigan ticks"]
fn live_michigan_all_county_cards_keep_public_source_and_quiet_restart_freshness() {
    let mut target = DisposableTarget::create();
    let campaign =
        CampaignId::from_uuid(Uuid::from_u128(0x0044_0000_0000_0000_0000_0000_0000_0002));
    SemanticArchiveStoreV1::new(&target.writer)
        .install_schema()
        .unwrap();
    // A fresh campaign must remain searchable before automatic statistics
    // maintenance. Keep this disposable fixture cold through the full drain;
    // otherwise autovacuum timing can hide repeated whole-Archive validation.
    target
        .writer
        .connect(NoTls)
        .unwrap()
        .batch_execute(
            "ALTER TABLE babylon_meta.archive_page_revision_v2 SET (autovacuum_enabled = false); \
             ALTER TABLE babylon_meta.archive_revision_grant_v2 SET (autovacuum_enabled = false); \
             ALTER TABLE babylon_meta.archive_revision_atom_v2 SET (autovacuum_enabled = false); \
             ALTER TABLE babylon_meta.archive_knowledge_grant_v1 SET (autovacuum_enabled = false)",
        )
        .unwrap();
    let preset = MichiganDeliveryPresetV1::Standard;
    let mut runtime = DurableMaterialRuntimeV3::create(
        &target.writer,
        campaign,
        MichiganContentPresetV1::new_campaign(preset)
            .create_foundation(&crate::test_support::catalog())
            .unwrap(),
    )
    .unwrap();
    install_reader_role_v1(&target.writer).unwrap();
    install_observer_economy_schema_v1(&target.writer).unwrap();
    let config = target.login("babylon_reader", "countycards");
    let reader = SemanticArchiveReaderV1::new(&config).unwrap();
    let foundation = read_county(&reader, &ArchiveReadScopeV2::foundation(campaign), "26163");
    assert_eq!(
        foundation.state,
        ArchiveDossierStateV2::Unavailable(ArchiveDossierUnavailableV2::FoundationHasNoPage)
    );
    for relation in [
        "babylon_meta.archive_knowledge_grant_v1",
        "babylon_state.territory_state_field_v1",
    ] {
        let error = config
            .connect(NoTls)
            .unwrap()
            .query(&format!("SELECT * FROM {relation} LIMIT 1"), &[])
            .unwrap_err();
        assert_eq!(
            error.code(),
            Some(&postgres::error::SqlState::INSUFFICIENT_PRIVILEGE)
        );
    }
    advance_material_period(&mut runtime);
    assert_archive_progress(&reader, campaign, 1, 0);
    let mut worker = ArchiveWorkerV1::new(&target.writer);
    let producer = michigan_archive_producer(&target.writer);
    let first = worker.sweep_once(campaign, &producer).unwrap();
    assert_eq!(
        first.dispositions(),
        &[(1, ArchiveReceiptDispositionV1::Paged)]
    );
    assert_archive_progress(&reader, campaign, 1, 0);
    let scope = current_archive_scope(&reader, campaign);
    let pages = assert_all_county_cards(
        &reader,
        &scope,
        ArchiveSearchStateV2::Pending(ArchiveDossierPendingV2::ReceiptProcessing),
    );
    drop((runtime, worker, producer));
    assert_restart_drains_and_verifies_quiet_periods(&target, &reader, &scope, &pages);
}

fn assert_restart_drains_and_verifies_quiet_periods(
    target: &DisposableTarget,
    reader: &SemanticArchiveReaderV1,
    first_scope: &ArchiveReadScopeV2,
    pages: &[ArchiveDossierPageV2],
) {
    let campaign = first_scope.campaign_id();
    let mut runtime = DurableMaterialRuntimeV3::open(
        &target.writer,
        campaign,
        MichiganContentPresetV1::new_campaign(MichiganDeliveryPresetV1::Standard)
            .create_foundation(&crate::test_support::catalog())
            .unwrap()
            .digest(),
    )
    .unwrap();
    let mut worker = ArchiveWorkerV1::new(&target.writer);
    let producer = michigan_archive_producer(&target.writer);
    for _ in 0..4 {
        if worker
            .sweep_once(campaign, &producer)
            .unwrap()
            .verified_tick()
            == 1
        {
            break;
        }
    }
    assert_archive_progress(reader, campaign, 1, 1);
    let ready_pages = assert_all_county_cards(reader, first_scope, ArchiveSearchStateV2::Ready);
    assert_eq!(ready_pages.len(), pages.len());
    for (ready, staged) in ready_pages.iter().zip(pages) {
        assert_retained_content(ready, staged);
    }
    let held_read = read_county(reader, first_scope, "26163");
    let held = assert_scoped_page(&held_read, first_scope, ArchiveSearchStateV2::Ready);
    assert!(held.changes.next_cursor.is_none());
    assert_eq!(held.changes.changes.len(), held.atoms.len());
    for change in &held.changes.changes {
        assert_eq!(change.publication_tick, 1);
        assert!(change.before.is_none());
        assert!(held.atoms.contains(change.after.as_ref().unwrap()));
    }
    assert!(worker
        .sweep_once(campaign, &producer)
        .unwrap()
        .dispositions()
        .is_empty());
    for tick in 2..=3 {
        advance_material_period(&mut runtime);
        let scope = current_archive_scope(reader, campaign);
        assert_eq!(scope.tick(), tick);
        assert_archive_progress(reader, campaign, tick, tick - 1);
        let pending = read_county(reader, &scope, "26163");
        assert_retained_content(
            assert_scoped_page(
                &pending,
                &scope,
                ArchiveSearchStateV2::Pending(ArchiveDossierPendingV2::ReceiptProcessing),
            ),
            held,
        );
        let quiet = worker.sweep_once(campaign, &producer).unwrap();
        assert_eq!(
            quiet.dispositions(),
            &[(tick, ArchiveReceiptDispositionV1::Applied)]
        );
        assert_archive_progress(reader, campaign, tick, tick);
        let current_pages = assert_all_county_cards(reader, &scope, ArchiveSearchStateV2::Ready);
        assert_eq!(current_pages.len(), pages.len());
        for (current, original) in current_pages.iter().zip(pages) {
            assert_retained_content(current, original);
        }
        let current = read_county(reader, &scope, "26163");
        assert_eq!(
            assert_scoped_page(&current, &scope, ArchiveSearchStateV2::Ready).changes,
            held.changes
        );
        let historical = read_county(reader, first_scope, "26163");
        assert_eq!(
            assert_scoped_page(&historical, first_scope, ArchiveSearchStateV2::Ready),
            held
        );
        drop(runtime);
        runtime = DurableMaterialRuntimeV3::open(
            &target.writer,
            campaign,
            MichiganContentPresetV1::new_campaign(MichiganDeliveryPresetV1::Standard)
                .create_foundation(&crate::test_support::catalog())
                .unwrap()
                .digest(),
        )
        .unwrap();
        assert_eq!(runtime.session().completed_tick(), tick);
        assert_eq!(read_county(reader, &scope, "26163"), current);
    }
}
impl DisposableTarget {
    fn create() -> Self {
        assert_eq!(
            std::env::var("BABYLON_POSTGRES_DISPOSABLE_ACK").as_deref(),
            Ok(ACK)
        );
        let canary = std::env::var("BABYLON_POSTGRES_DISPOSABLE_CANARY").unwrap();
        assert_eq!(canary.len(), 32);
        let admin: Config = std::env::var("BABYLON_POSTGRES_TEST_DSN")
            .unwrap()
            .parse()
            .unwrap();
        validate_connection_target(&admin).unwrap();
        assert_eq!(admin.get_user(), Some("test"));
        assert_eq!(admin.get_dbname(), Some("postgres"));
        let mut connection = admin.connect(NoTls).unwrap();
        let actual: Option<String> = connection
            .query_one(
                "SELECT pg_catalog.current_setting('babylon.disposable_runtime',true)",
                &[],
            )
            .unwrap()
            .get(0);
        assert_eq!(actual.as_deref(), Some(canary.as_str()));
        let template = std::env::var("BABYLON_RUNTIME_TEMPLATE_DB").unwrap();
        let suffix = template.strip_prefix("per281_runtime_template_").unwrap();
        assert_eq!(suffix.len(), 12);
        assert!(suffix.bytes().all(|byte| byte.is_ascii_hexdigit()));
        let sequence = NEXT_DISPOSABLE_TARGET
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .expect("disposable target sequence remains bounded");
        let database = format!(
            "per281_runtime_materialobserver_{}_{sequence}",
            std::process::id()
        );
        connection
            .batch_execute(&format!(
                "CREATE DATABASE \"{database}\" OWNER test TEMPLATE \"{template}\""
            ))
            .unwrap();
        let mut writer = admin.clone();
        writer.dbname(&database);
        Self {
            sequence,
            admin,
            writer,
            database,
            roles: Vec::new(),
        }
    }

    fn login(&mut self, group: &str, suffix: &str) -> Config {
        assert!(matches!(group, "babylon_observer" | "babylon_reader"));
        assert!(suffix.bytes().all(|byte| byte.is_ascii_lowercase()));
        let role = format!(
            "g4_material_{suffix}_{}_{}",
            std::process::id(),
            self.sequence
        );
        let mut connection = self.writer.connect(NoTls).unwrap();
        let created = {
            let _guard = PARAMETER_ACL_WRITE.lock().unwrap();
            connection.batch_execute(&format!("CREATE ROLE \"{role}\" LOGIN PASSWORD 'reader' NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS; GRANT {group} TO \"{role}\"; GRANT SET ON PARAMETER event_triggers TO \"{role}\""))
        };
        created.unwrap();
        self.roles.push(role.clone());
        let mut config = self.writer.clone();
        config.user(&role).password("reader");
        config
    }
}
impl Drop for DisposableTarget {
    fn drop(&mut self) {
        if let Ok(mut connection) = self.admin.connect(NoTls) {
            let dropped = connection.batch_execute(&format!(
                "DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)",
                self.database
            ));
            if !std::thread::panicking() {
                dropped.expect("owned clone cleanup");
            }
            for role in &self.roles {
                let removed = {
                    let _guard = PARAMETER_ACL_WRITE.lock().unwrap();
                    connection.batch_execute(&format!("REVOKE SET ON PARAMETER event_triggers FROM \"{role}\"; DROP ROLE IF EXISTS \"{role}\""))
                };
                if !std::thread::panicking() {
                    removed.expect("owned role cleanup");
                }
            }
        }
    }
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed material ticks"]
fn live_material_observer_preserves_history_and_denies_preview_blob_authority() {
    let mut target = DisposableTarget::create();
    let preset = MichiganDeliveryPresetV1::Standard;
    let campaign =
        CampaignId::from_uuid(Uuid::from_u128(0x0044_0000_0000_0000_0000_0000_0000_0001));
    let mut runtime = DurableMaterialRuntimeV3::create(
        &target.writer,
        campaign,
        MichiganContentPresetV1::new_campaign(preset)
            .create_foundation(&crate::test_support::catalog())
            .unwrap(),
    )
    .unwrap();
    install_reader_role_v1(&target.writer).unwrap();
    install_observer_economy_schema_v1(&target.writer).unwrap();
    install_observer_economy_schema_v1(&target.writer).unwrap();
    let observer_config = target.login("babylon_observer", "observer");
    let known_config = target.login("babylon_reader", "known");
    let observer =
        ObserverEconomyReaderV1::connect(&observer_config, ObserverVisibilityV1::FullObserver)
            .unwrap();
    let known = ObserverEconomyReaderV1::connect(&known_config, ObserverVisibilityV1::KnownPreview)
        .unwrap();
    let archive = SemanticArchiveReaderV1::new(&known_config).unwrap();
    let zero = observer.snapshot(campaign, 0).unwrap();
    assert_eq!(zero.counties.len(), 83);
    assert_eq!(zero.production.as_ref().unwrap().sites.len(), 5);
    assert_material_accounts(&zero);
    assert_known_material_absence(&known.snapshot(campaign, 0).unwrap());
    assert!(known_config
        .connect(NoTls)
        .unwrap()
        .query(
            "SELECT register_bytes FROM public.v_observer_material_state_v1",
            &[]
        )
        .is_err());
    assert_eq!(observer.campaigns().unwrap(), known.campaigns().unwrap());
    assert_eq!(observer.campaigns().unwrap()[0].durable_tick, 0);
    let mut history_at_two = None;
    for tick in 1..=6 {
        advance_material_period(&mut runtime);
        let snapshot = observer.snapshot(campaign, tick).unwrap();
        assert_eq!(snapshot.foundation_digest, zero.foundation_digest);
        assert_eq!(snapshot.resolve_tick, tick);
        assert_eq!(snapshot.counties.len(), 83);
        assert_material_accounts(&snapshot);
        assert_known_material_absence(&known.snapshot(campaign, tick).unwrap());
        archive.committed_tick_status(campaign).unwrap();
        if tick == 2 {
            history_at_two = Some(snapshot);
        }
        if tick == 3 {
            runtime = DurableMaterialRuntimeV3::open(
                &target.writer,
                campaign,
                MichiganContentPresetV1::new_campaign(preset)
                    .create_foundation(&crate::test_support::catalog())
                    .unwrap()
                    .digest(),
            )
            .unwrap();
            assert_eq!(runtime.session().completed_tick(), 3);
            let fresh = ObserverEconomyReaderV1::connect(
                &observer_config,
                ObserverVisibilityV1::FullObserver,
            )
            .unwrap();
            assert_eq!(
                fresh.snapshot(campaign, 2).unwrap(),
                history_at_two.clone().unwrap()
            );
            assert_material_accounts(&fresh.snapshot(campaign, 3).unwrap());
        }
    }
    assert_eq!(
        observer.snapshot(campaign, 2).unwrap(),
        history_at_two.unwrap()
    );
    assert_eq!(observer.campaigns().unwrap()[0].durable_tick, 6);
    assert_eq!(
        observer.snapshot(campaign, 7),
        Err(ObserverEconomyErrorV1::TickAbsent)
    );

    let mut connection = target.writer.connect(NoTls).unwrap();
    assert_corrupted_register_is_rejected(&mut connection, &observer, campaign);

    let known_role = known_config.get_user().unwrap();
    connection
        .batch_execute(&format!(
            "GRANT SELECT ON public.v_observer_material_state_v1 TO \"{known_role}\""
        ))
        .unwrap();
    assert_eq!(
        known.snapshot(campaign, 6),
        Err(ObserverEconomyErrorV1::Authority)
    );
}

fn identity_hex(bytes: [u8; 32]) -> String {
    use std::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(64), |mut result, byte| {
            write!(result, "{byte:02x}").unwrap();
            result
        })
}

fn assert_known_material_absence(snapshot: &babylon_persistence::ObserverEconomySnapshotV1) {
    assert_eq!(snapshot.visibility, ObserverVisibilityV1::KnownPreview);
    assert!(snapshot.production.is_none());
    assert!(snapshot.production_evidence_digest().is_none());
}

fn assert_material_accounts(snapshot: &babylon_persistence::ObserverEconomySnapshotV1) {
    use babylon_persistence::ProductionDeliveryStageV1;
    use std::collections::{BTreeMap, BTreeSet};

    let rows = snapshot.production.as_ref().unwrap();
    assert!(rows
        .events
        .iter()
        .all(|event| event.period <= snapshot.resolve_tick));
    if snapshot.resolve_tick == 0 {
        assert!(rows.material_balance.is_none());
        assert!(rows.events.is_empty());
        return;
    }
    let balance = rows.material_balance.as_ref().unwrap();
    assert_eq!(balance.period, snapshot.resolve_tick);
    assert!(!balance.rows.is_empty());
    let mut principals = BTreeSet::new();
    let mut arrivals = BTreeMap::new();
    for event in &rows.events {
        let expected_stage = match event.kind.as_str() {
            "arrival" => Some(ProductionDeliveryStageV1::Arrival),
            "delivery" => Some(ProductionDeliveryStageV1::Delivery),
            "quantity realization" => Some(ProductionDeliveryStageV1::QuantityRealization),
            _ => None,
        };
        assert_eq!(
            event.delivery_evidence.as_ref().map(|row| row.stage),
            expected_stage
        );
        let Some(evidence) = &event.delivery_evidence else {
            continue;
        };
        let route = rows
            .routes
            .iter()
            .find(|route| route.id == evidence.route_id)
            .unwrap();
        assert_eq!(evidence.good_id, route.good_id);
        assert_eq!(evidence.unit_id, route.unit_id);
        assert!(evidence.quantity > 0);
        let catalog = crate::test_support::catalog();
        let source = catalog
            .routes()
            .iter()
            .find(|row| identity_hex(row.id().as_bytes()) == route.id)
            .unwrap();
        assert_eq!(
            evidence.order_id,
            identity_hex(source.order_id().as_bytes())
        );
        if event.period == balance.period && evidence.stage == ProductionDeliveryStageV1::Arrival {
            let key = (&route.buyer_site_id, &evidence.good_id, &evidence.unit_id);
            *arrivals.entry(key).or_insert(0_u128) += u128::from(evidence.quantity);
        }
    }
    for row in &balance.rows {
        let principal = (&row.site_id, &row.good_id, &row.unit_id);
        assert!(principals.insert(principal));
        assert_eq!(
            u128::from(row.opening) + u128::from(row.arrivals) + u128::from(row.produced),
            u128::from(row.consumed) + u128::from(row.dispatched) + u128::from(row.closing)
        );
        assert_eq!(
            u128::from(row.arrivals),
            arrivals.remove(&principal).unwrap_or(0)
        );
        let site = rows
            .sites
            .iter()
            .find(|site| site.id == row.site_id)
            .unwrap();
        let closing = site
            .inventory
            .iter()
            .find(|stock| stock.good_id == row.good_id && stock.unit_id == row.unit_id)
            .map_or(0, |stock| stock.quantity);
        assert_eq!(row.closing, closing);
    }
    assert!(arrivals.is_empty());
}

fn assert_corrupted_register_is_rejected(
    connection: &mut postgres::Client,
    observer: &ObserverEconomyReaderV1,
    campaign: CampaignId,
) {
    // A syntactically valid stored register mutation cannot retain its committed identity.
    let original: Vec<u8> = connection.query_one("SELECT register_bytes FROM babylon_state.material_tick_v3 WHERE campaign_id=$1 AND resolve_tick=6", &[campaign.as_uuid()]).unwrap().get(0);
    let register = MaterialWorldRegisterV2::decode(&original).unwrap();
    let mut state = register.state().clone();
    state.inventory[0].quantity += 1;
    let corrupt = MaterialWorldRegisterV2::try_new(6, state).unwrap();
    connection.execute("UPDATE babylon_state.material_tick_v3 SET register_bytes=$2 WHERE campaign_id=$1 AND resolve_tick=6", &[campaign.as_uuid(), &corrupt.canonical_bytes()]).unwrap();
    assert_eq!(
        observer.snapshot(campaign, 6),
        Err(ObserverEconomyErrorV1::InvalidProjection)
    );
    connection.execute("UPDATE babylon_state.material_tick_v3 SET register_bytes=$2 WHERE campaign_id=$1 AND resolve_tick=6", &[campaign.as_uuid(), &original]).unwrap();
}

#[test]
#[ignore = "requires the existing disposable PostgreSQL harness and restricted reader roles"]
fn live_content_revisions_resume_exactly_and_catalog_filters_before_its_limit() {
    use babylon_persistence::michigan_content::MICHIGAN_CONTENT_PRESETS_V1;
    let mut target = DisposableTarget::create();
    let mut campaigns = Vec::new();
    for (index, preset) in MICHIGAN_CONTENT_PRESETS_V1.into_iter().enumerate() {
        let campaign =
            CampaignId::from_uuid(Uuid::from_u128(10_000 + u128::try_from(index).unwrap()));
        let mut runtime = DurableMaterialRuntimeV3::create(
            &target.writer,
            campaign,
            preset
                .create_foundation(&crate::test_support::catalog())
                .unwrap(),
        )
        .unwrap();
        advance_material_period(&mut runtime);
        advance_material_period(&mut runtime);
        campaigns.push((campaign, preset, runtime));
    }
    install_reader_role_v1(&target.writer).unwrap();
    install_observer_economy_schema_v1(&target.writer).unwrap();
    let observer = ObserverEconomyReaderV1::connect(
        &target.login("babylon_observer", "catalogobserver"),
        ObserverVisibilityV1::FullObserver,
    )
    .unwrap();
    let known_config = target.login("babylon_reader", "catalogknown");
    let known = ObserverEconomyReaderV1::connect(&known_config, ObserverVisibilityV1::KnownPreview)
        .unwrap();
    for (campaign, preset, runtime) in &mut campaigns {
        assert_revision_resume(
            &target.writer,
            *campaign,
            *preset,
            runtime,
            &observer,
            &known,
        );
    }
    let before = observer.campaigns().unwrap();
    assert_eq!(before.len(), 4);
    assert_eq!(
        before
            .iter()
            .map(|row| (row.id.clone(), row.preset.clone()))
            .collect::<std::collections::BTreeSet<_>>(),
        campaigns
            .iter()
            .map(|(campaign, preset, _)| (campaign.as_uuid().to_string(), preset.id().to_owned()))
            .collect::<std::collections::BTreeSet<_>>()
    );
    assert!(before.iter().all(|row| row.durable_tick == 4));
    assert_eq!(before, known.campaigns().unwrap());
    let unknown = insert_unadmitted_catalog_rows(&target.writer, campaigns[0].0);
    // A safe header is discoverable without disclosing its opaque Designed values.
    // Only the full capability authenticates stored material content.
    let discovered = observer.campaigns().unwrap();
    assert_eq!(discovered, known.campaigns().unwrap());
    assert_eq!(discovered.len(), before.len() + 1);
    assert_eq!(&discovered[1..], before.as_slice());
    assert_eq!(discovered[0].id, unknown[65].as_uuid().to_string());
    assert_eq!(unknown.len(), 66);
    for reader in [&observer, &known] {
        assert_eq!(
            reader.snapshot(unknown[0], 0),
            Err(ObserverEconomyErrorV1::ScenarioMismatch)
        );
    }
    assert_eq!(
        observer.snapshot(unknown[65], 0),
        Err(ObserverEconomyErrorV1::ScenarioMismatch)
    );
    let opaque = known.snapshot(unknown[65], 0).unwrap();
    assert!(opaque.production.is_none());
    assert!(opaque.nominal_world_hash.is_none());
    assert_eq!(opaque.counties.len(), 83);
    assert!(opaque.counties.iter().all(|county| {
        county.annual_avg_estabs_count.is_none()
            && county.annual_avg_emplvl.is_none()
            && county.total_annual_wages.is_none()
            && county.annual_avg_wkly_wage.is_none()
    }));
    assert!(known_config
        .connect(NoTls)
        .unwrap()
        .query(
            "SELECT register_bytes FROM public.v_observer_material_state_v1",
            &[]
        )
        .is_err());
}

fn assert_revision_resume(
    config: &Config,
    campaign: CampaignId,
    preset: babylon_persistence::michigan_content::MichiganContentPresetV1,
    runtime: &mut DurableMaterialRuntimeV3,
    observer: &ObserverEconomyReaderV1,
    known: &ObserverEconomyReaderV1,
) {
    let history = observer.snapshot(campaign, 1).unwrap();
    assert_eq!(history.counties.len(), 83);
    assert_eq!(history.production.as_ref().unwrap().sites.len(), 5);
    let at_two = observer.snapshot(campaign, 2).unwrap();
    assert_material_accounts(&history);
    assert_material_accounts(&at_two);
    let actions = OrderedPracticeActionBatchV1::empty(
        runtime.session().graph_session().session_identity().clone(),
        3,
    )
    .unwrap();
    let uninterrupted = runtime.session().prepare_advance(&actions).unwrap();
    let mut reopened = DurableMaterialRuntimeV3::open(
        config,
        campaign,
        preset
            .admitted(&crate::test_support::catalog())
            .unwrap()
            .digest(),
    )
    .unwrap();
    assert_eq!(reopened.session().completed_tick(), 2);
    let restored = reopened.session().prepare_advance(&actions).unwrap();
    assert_eq!(uninterrupted.identity(), restored.identity());
    assert_eq!(
        uninterrupted.material().register().canonical_bytes(),
        restored.material().register().canonical_bytes()
    );
    assert_eq!(
        uninterrupted.material().receipt_bytes(),
        restored.material().receipt_bytes()
    );
    assert_eq!(observer.snapshot(campaign, 2).unwrap(), at_two);
    advance_material_period(&mut reopened);
    assert_material_accounts(&observer.snapshot(campaign, 3).unwrap());
    assert_eq!(observer.snapshot(campaign, 1).unwrap(), history);
    assert_known_material_absence(&known.snapshot(campaign, 3).unwrap());
    assert_session_admits_stored_revision(config, campaign, observer);
    assert_eq!(observer.snapshot(campaign, 1).unwrap(), history);
    *runtime = reopened;
}

fn assert_session_admits_stored_revision(
    config: &Config,
    campaign: CampaignId,
    observer: &ObserverEconomyReaderV1,
) {
    use babylon_persistence::runtime_session::{
        run_runtime_session_v3, RuntimeSessionRequestV3, RuntimeSessionResponseV3,
        RuntimeSessionScopeV3, RuntimeSessionTailV3, RuntimeSessionTargetV3,
        RUNTIME_SESSION_PROTOCOL_VERSION_V3,
    };
    let current = observer.snapshot(campaign, 3).unwrap();
    let scope = RuntimeSessionScopeV3 {
        epoch: 1,
        campaign_id: Some(campaign.as_uuid().to_string()),
    };
    let requests = [
        RuntimeSessionRequestV3::Switch {
            protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION_V3,
            request_id: 1,
            scope: RuntimeSessionScopeV3 {
                epoch: 0,
                campaign_id: None,
            },
            target: RuntimeSessionTargetV3::Open {
                campaign_id: campaign.as_uuid().to_string(),
            },
        },
        RuntimeSessionRequestV3::Advance {
            protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION_V3,
            scope: scope.clone(),
            request_id: 2,
            expected_tail: RuntimeSessionTailV3 {
                resolve_tick: 3,
                tick_content_hash: current.tick_content_hash,
            },
        },
        RuntimeSessionRequestV3::Stop {
            protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION_V3,
            scope,
            request_id: 3,
        },
    ];
    let mut lines = Vec::new();
    for request in &requests {
        serde_json::to_writer(&mut lines, request).unwrap();
        lines.push(b'\n');
    }
    let mut output = Vec::new();
    run_runtime_session_v3(
        config,
        std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../content/scenarios/michigan/defines.toml"
        )),
        std::io::Cursor::new(lines),
        &mut output,
    )
    .unwrap();
    let responses = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<RuntimeSessionResponseV3>(line).unwrap())
        .collect::<Vec<_>>();
    assert!(
        matches!(&responses[0], RuntimeSessionResponseV3::Hello { protocol_version: 3, scope }
        if scope.epoch == 0 && scope.campaign_id.is_none())
    );
    assert!(
        matches!(&responses[2], RuntimeSessionResponseV3::Ready { foundation_digest, tail, .. }
        if foundation_digest == &current.foundation_digest && tail.resolve_tick == 3)
    );
    assert!(responses.iter().any(|response| matches!(response, RuntimeSessionResponseV3::Committed { tail, .. } if tail.resolve_tick == 4)));
    assert_eq!(
        observer.snapshot(campaign, 4).unwrap().foundation_digest,
        current.foundation_digest
    );
}

// Deliberately malformed metadata is confined to this test's disposable clone.
// The 65 unknown UUIDs sort before every admitted campaign, exposing LIMIT-before-
// admission bugs. The final row mixes an admitted preset with a changed digest.
fn insert_unadmitted_catalog_rows(config: &Config, source: CampaignId) -> Vec<CampaignId> {
    let mut client = config.connect(NoTls).unwrap();
    let mut tx = client.transaction().unwrap();
    let mut ids = Vec::new();
    for number in 1..=66_u128 {
        let campaign = CampaignId::from_uuid(Uuid::from_u128(number));
        tx.execute("INSERT INTO babylon_state.campaign (campaign_id,replay_layout_version,rng_layout_version,replay_session_id,rng_seed,defines_hash,rules_hash,ref_digest) SELECT $1,replay_layout_version,rng_layout_version,replay_session_id,rng_seed,defines_hash,rules_hash,ref_digest FROM babylon_state.campaign WHERE campaign_id=$2", &[campaign.as_uuid(), source.as_uuid()]).unwrap();
        tx.execute("INSERT INTO babylon_state.campaign_foundation (campaign_id,stable_graph,world_registers,resolver_manifest,prepared_environment,replay_session_id,rng_seed,defines_hash,rules_hash,ref_digest,scenario_source,prelude_source,rule_source,defines_bytes,reference_manifest_bytes,foundation_sha256) SELECT $1,stable_graph,world_registers,resolver_manifest,prepared_environment,replay_session_id,rng_seed,defines_hash,rules_hash,ref_digest,scenario_source,prelude_source,rule_source,defines_bytes,reference_manifest_bytes,foundation_sha256 FROM babylon_state.campaign_foundation WHERE campaign_id=$2", &[campaign.as_uuid(), source.as_uuid()]).unwrap();
        let preset = if number == 66 {
            // Opaque material content still needs a complete public county family.
            // Without this mapping, snapshot refusal would concern missing county
            // rows rather than whether the reader authenticates material bytes.
            assert_eq!(tx.execute(
                "INSERT INTO babylon_meta.campaign (campaign_id,slug,engine_version,defines_hash,last_tick,status,rng_seed,content_digest) \
                 SELECT $1,$1::uuid::text,engine_version,defines_hash,0,'ACTIVE',rng_seed,content_digest \
                 FROM babylon_meta.campaign WHERE campaign_id=$2",
                &[campaign.as_uuid(), source.as_uuid()],
            ).unwrap(), 1);
            assert_eq!(tx.execute(
                "INSERT INTO babylon_meta.territory_county_map_v1 (campaign_id,territory_local_name,county_geoid) \
                 SELECT $1,territory_local_name,county_geoid FROM babylon_meta.territory_county_map_v1 WHERE campaign_id=$2",
                &[campaign.as_uuid(), source.as_uuid()],
            ).unwrap(), 83);
            // No knowledge grants are copied: all observed values must stay hidden.
            MichiganContentPresetV1::new_campaign(MichiganDeliveryPresetV1::Standard).id()
        } else {
            "unadmitted-fixture-v1"
        };
        tx.execute("INSERT INTO babylon_state.material_campaign_foundation_v2 (campaign_id,preset_id,horizon_ticks,content_sha256,initial_register_bytes,foundation_bytes,foundation_sha256) SELECT $1,$3,horizon_ticks,content_sha256,initial_register_bytes,foundation_bytes,pg_catalog.set_byte(foundation_sha256,0,(pg_catalog.get_byte(foundation_sha256,0)+1)%256) FROM babylon_state.material_campaign_foundation_v2 WHERE campaign_id=$2", &[campaign.as_uuid(), source.as_uuid(), &preset]).unwrap();
        ids.push(campaign);
    }
    tx.commit().unwrap();
    ids
}

/// Every deliberate metadata fault in this module is confined to `DisposableTarget`'s
/// freshly created clone. Neither the template nor an existing user campaign is altered.
mod foundation_content_layout {
    use super::*;
    use babylon_persistence::{
        hydrate_campaign_foundation_v1,
        material_runtime::{install_material_runtime_schema_v3, MaterialRuntimeErrorV3},
        michigan_content::MichiganContentPresetV1,
        FoundationContentLayout, RustPersistenceRuntimeErrorV2,
    };

    const LAYOUT_TABLE: &str = "babylon_state.campaign_foundation_content_layout_v2";

    #[test]
    #[ignore = "requires the existing disposable PostgreSQL harness; independent clone ownership"]
    fn live_current_content_layout_refuses_corruption_and_reopens_exactly() {
        let target = DisposableTarget::create();
        let standard = CampaignId::from_uuid(Uuid::from_u128(21_001));
        let delayed = CampaignId::from_uuid(Uuid::from_u128(21_002));
        let standard_preset =
            MichiganContentPresetV1::new_campaign(MichiganDeliveryPresetV1::Standard);
        let delayed_preset =
            MichiganContentPresetV1::new_campaign(MichiganDeliveryPresetV1::Delayed);
        assert_ne!(standard_preset, delayed_preset);
        let mut standard_runtime = DurableMaterialRuntimeV3::create(
            &target.writer,
            standard,
            standard_preset
                .create_foundation(&crate::test_support::catalog())
                .unwrap(),
        )
        .unwrap();
        let mut delayed_runtime = DurableMaterialRuntimeV3::create(
            &target.writer,
            delayed,
            delayed_preset
                .create_foundation(&crate::test_support::catalog())
                .unwrap(),
        )
        .unwrap();
        advance_material_period(&mut standard_runtime);
        advance_material_period(&mut delayed_runtime);
        assert_missing_layout_is_not_healed(
            &target,
            standard,
            standard_preset,
            delayed,
            delayed_preset,
        );
        for (campaign, preset, runtime) in [
            (standard, standard_preset, &standard_runtime),
            (delayed, delayed_preset, &delayed_runtime),
        ] {
            let before = foundation_bytes(&target.writer, campaign);
            assert_unknown_layout_is_refused(&target, campaign, preset);
            assert_valid_but_wrong_layout_is_refused(&target, campaign, preset);
            install_material_runtime_schema_v3(&target.writer).unwrap();
            install_material_runtime_schema_v3(&target.writer).unwrap();
            assert_eq!(
                hydrate_campaign_foundation_v1(&target.writer, campaign)
                    .unwrap()
                    .content_bundle()
                    .layout(),
                FoundationContentLayout::V2
            );
            assert_eq!(foundation_bytes(&target.writer, campaign), before);
            assert_next_commit_survives_reopen(&target.writer, campaign, preset, runtime);
        }
    }

    #[test]
    #[ignore = "requires the existing disposable PostgreSQL harness; independent clone ownership"]
    fn live_open_material_runtime_refuses_missing_or_changed_content_layout_before_ack() {
        let target = DisposableTarget::create();
        for (index, missing) in [true, false].into_iter().enumerate() {
            let campaign =
                CampaignId::from_uuid(Uuid::from_u128(22_001 + u128::try_from(index).unwrap()));
            let mut runtime = DurableMaterialRuntimeV3::create(
                &target.writer,
                campaign,
                MichiganContentPresetV1::new_campaign(MichiganDeliveryPresetV1::Standard)
                    .create_foundation(&crate::test_support::catalog())
                    .unwrap(),
            )
            .unwrap();
            let world = runtime.session().current_world_hash().unwrap();
            corrupt_open_layout(&target.writer, campaign, missing, 1);
            let actions = OrderedPracticeActionBatchV1::empty(
                runtime.session().graph_session().session_identity().clone(),
                1,
            )
            .unwrap();
            let mut sink = CollectingSink::default();
            let result = runtime.advance_and_commit(&mut sink, &actions);
            assert!(result.is_err(), "an open material runtime must refuse missing={missing} layout before acknowledgement");
            assert_eq!(runtime.session().completed_tick(), 0);
            assert_eq!(runtime.session().current_world_hash().unwrap(), world);
            assert!(runtime.tail().is_none());
            assert!(sink.events.is_empty());
            assert_no_committed_tick(&target.writer, campaign);
        }
    }

    #[test]
    #[ignore = "requires the existing disposable PostgreSQL harness; independent clone ownership"]
    fn live_open_graph_runtime_refuses_missing_or_changed_content_layout_before_ack() {
        use babylon_persistence::{
            michigan_economy::michigan_observer_foundation_v1, DurableReplayRuntimeV2,
        };
        let target = DisposableTarget::create();
        for (index, missing) in [true, false].into_iter().enumerate() {
            let campaign =
                CampaignId::from_uuid(Uuid::from_u128(23_001 + u128::try_from(index).unwrap()));
            let (session, bundle) = michigan_observer_foundation_v1().unwrap();
            let actions =
                OrderedPracticeActionBatchV1::empty(session.session_identity().clone(), 1).unwrap();
            let mut runtime =
                DurableReplayRuntimeV2::create(&target.writer, campaign, session, bundle).unwrap();
            let graph = runtime.observe_current_stable_graph_state_v1().unwrap();
            corrupt_open_layout(&target.writer, campaign, missing, 2);
            let mut sink = CollectingSink::default();
            let result = runtime.advance_and_commit(&mut sink, &actions);
            assert!(
                result.is_err(),
                "an open graph runtime must refuse missing={missing} layout before acknowledgement"
            );
            assert!(runtime.last_committed_tick().is_none());
            assert_eq!(
                runtime
                    .observe_current_stable_graph_state_v1()
                    .unwrap()
                    .canonical_bytes(),
                graph.canonical_bytes()
            );
            assert!(sink.events.is_empty());
            assert_no_committed_tick(&target.writer, campaign);
        }
    }

    fn corrupt_open_layout(config: &Config, campaign: CampaignId, missing: bool, wrong: i16) {
        let mut client = config.connect(NoTls).unwrap();
        let affected = if missing {
            client.execute(
                "DELETE FROM babylon_state.campaign_foundation_content_layout_v2 WHERE campaign_id=$1::uuid",
                &[campaign.as_uuid()],
            ).unwrap()
        } else {
            client.execute(
                "UPDATE babylon_state.campaign_foundation_content_layout_v2 SET content_layout_version=$2 WHERE campaign_id=$1::uuid",
                &[campaign.as_uuid(), &wrong],
            ).unwrap()
        };
        assert_eq!(affected, 1);
    }

    fn assert_no_committed_tick(config: &Config, campaign: CampaignId) {
        let count: i64 = config
            .connect(NoTls)
            .unwrap()
            .query_one(
                "SELECT count(*) FROM babylon_state.tick_commit WHERE campaign_id=$1::uuid",
                &[campaign.as_uuid()],
            )
            .unwrap()
            .get(0);
        assert_eq!(count, 0);
    }

    fn foundation_bytes(config: &Config, campaign: CampaignId) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let row = config
            .connect(NoTls)
            .unwrap()
            .query_one(
                "SELECT f.foundation_sha256, m.foundation_bytes, m.foundation_sha256 \
             FROM babylon_state.campaign_foundation f \
             JOIN babylon_state.material_campaign_foundation_v2 m USING (campaign_id) \
             WHERE campaign_id = $1::uuid",
                &[campaign.as_uuid()],
            )
            .unwrap();
        (row.get(0), row.get(1), row.get(2))
    }

    fn assert_missing_layout_is_not_healed(
        target: &DisposableTarget,
        campaign: CampaignId,
        preset: MichiganContentPresetV1,
        sibling: CampaignId,
        sibling_preset: MichiganContentPresetV1,
    ) {
        let before = foundation_bytes(&target.writer, campaign);
        let mut sql = target.writer.connect(NoTls).unwrap();
        assert_eq!(
            sql.execute(
                &format!("DELETE FROM {LAYOUT_TABLE} WHERE campaign_id=$1::uuid"),
                &[campaign.as_uuid()]
            )
            .unwrap(),
            1
        );
        for _ in 0..2 {
            install_material_runtime_schema_v3(&target.writer).unwrap();
        }
        assert!(matches!(
            DurableMaterialRuntimeV3::open(
                &target.writer,
                campaign,
                preset
                    .admitted(&crate::test_support::catalog())
                    .unwrap()
                    .digest()
            ),
            Err(MaterialRuntimeErrorV3::Graph(
                RustPersistenceRuntimeErrorV2::FoundationAbsent
            ))
        ));
        assert!(matches!(
            DurableMaterialRuntimeV3::create(
                &target.writer,
                campaign,
                preset
                    .create_foundation(&crate::test_support::catalog())
                    .unwrap()
            ),
            Err(MaterialRuntimeErrorV3::Graph(
                RustPersistenceRuntimeErrorV2::FoundationAbsent
            ))
        ));
        let missing: i64 = sql
            .query_one(
                &format!("SELECT count(*) FROM {LAYOUT_TABLE} WHERE campaign_id=$1::uuid"),
                &[campaign.as_uuid()],
            )
            .unwrap()
            .get(0);
        assert_eq!(missing, 0);
        assert_eq!(
            DurableMaterialRuntimeV3::open(
                &target.writer,
                sibling,
                sibling_preset
                    .admitted(&crate::test_support::catalog())
                    .unwrap()
                    .digest()
            )
            .unwrap()
            .session()
            .completed_tick(),
            1
        );
        assert_eq!(foundation_bytes(&target.writer, campaign), before);
        sql.execute(
            &format!("INSERT INTO {LAYOUT_TABLE} VALUES ($1::uuid, 2)"),
            &[campaign.as_uuid()],
        )
        .unwrap();
    }

    fn assert_unknown_layout_is_refused(
        target: &DisposableTarget,
        campaign: CampaignId,
        preset: MichiganContentPresetV1,
    ) {
        let mut sql = target.writer.connect(NoTls).unwrap();
        let error = sql
            .execute(
                &format!(
                    "UPDATE {LAYOUT_TABLE} SET content_layout_version=3 WHERE campaign_id=$1::uuid"
                ),
                &[campaign.as_uuid()],
            )
            .unwrap_err();
        assert_eq!(
            error.code(),
            Some(&postgres::error::SqlState::CHECK_VIOLATION)
        );
        let constraint: String = sql
            .query_one(
                "SELECT conname FROM pg_catalog.pg_constraint \
             WHERE conrelid = 'babylon_state.campaign_foundation_content_layout_v2'::regclass \
             AND contype = 'c'",
                &[],
            )
            .unwrap()
            .get(0);
        assert!(constraint
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'));
        // Check decoder default-deny even under deliberate schema corruption. The
        // cloned database is dropped by DisposableTarget on either pass or panic.
        sql.batch_execute(&format!(
            "ALTER TABLE {LAYOUT_TABLE} DROP CONSTRAINT \"{constraint}\""
        ))
        .unwrap();
        sql.execute(
            &format!(
                "UPDATE {LAYOUT_TABLE} SET content_layout_version=3 WHERE campaign_id=$1::uuid"
            ),
            &[campaign.as_uuid()],
        )
        .unwrap();
        assert!(matches!(
            DurableMaterialRuntimeV3::open(
                &target.writer,
                campaign,
                preset
                    .admitted(&crate::test_support::catalog())
                    .unwrap()
                    .digest()
            ),
            Err(MaterialRuntimeErrorV3::Graph(
                RustPersistenceRuntimeErrorV2::ReplaySource
            ))
        ));
        sql.execute(
            &format!(
                "UPDATE {LAYOUT_TABLE} SET content_layout_version=2 WHERE campaign_id=$1::uuid"
            ),
            &[campaign.as_uuid()],
        )
        .unwrap();
        sql.batch_execute(&format!("ALTER TABLE {LAYOUT_TABLE} ADD CONSTRAINT \"{constraint}\" CHECK (content_layout_version IN (1,2))")).unwrap();
    }

    fn assert_valid_but_wrong_layout_is_refused(
        target: &DisposableTarget,
        campaign: CampaignId,
        preset: MichiganContentPresetV1,
    ) {
        let mut sql = target.writer.connect(NoTls).unwrap();
        let before = foundation_bytes(&target.writer, campaign);
        sql.execute(
            &format!(
                "UPDATE {LAYOUT_TABLE} SET content_layout_version=1 WHERE campaign_id=$1::uuid"
            ),
            &[campaign.as_uuid()],
        )
        .unwrap();
        assert!(matches!(
            DurableMaterialRuntimeV3::open(
                &target.writer,
                campaign,
                preset
                    .admitted(&crate::test_support::catalog())
                    .unwrap()
                    .digest(),
            ),
            Err(MaterialRuntimeErrorV3::Graph(
                RustPersistenceRuntimeErrorV2::SemanticCodec
            ))
        ));
        assert_eq!(foundation_bytes(&target.writer, campaign), before);
        sql.execute(
            &format!(
                "UPDATE {LAYOUT_TABLE} SET content_layout_version=2 WHERE campaign_id=$1::uuid"
            ),
            &[campaign.as_uuid()],
        )
        .unwrap();
    }

    fn assert_next_commit_survives_reopen(
        config: &Config,
        campaign: CampaignId,
        preset: MichiganContentPresetV1,
        uninterrupted: &DurableMaterialRuntimeV3,
    ) {
        let mut reopened = DurableMaterialRuntimeV3::open(
            config,
            campaign,
            preset
                .admitted(&crate::test_support::catalog())
                .unwrap()
                .digest(),
        )
        .unwrap();
        assert_eq!(reopened.session().completed_tick(), 1);
        let actions = OrderedPracticeActionBatchV1::empty(
            uninterrupted
                .session()
                .graph_session()
                .session_identity()
                .clone(),
            2,
        )
        .unwrap();
        let expected = uninterrupted.session().prepare_advance(&actions).unwrap();
        let actual = reopened.session().prepare_advance(&actions).unwrap();
        assert_eq!(actual.identity(), expected.identity());
        assert_eq!(
            actual.material().register().canonical_bytes(),
            expected.material().register().canonical_bytes()
        );
        assert_eq!(
            actual.material().receipt_bytes(),
            expected.material().receipt_bytes()
        );
        drop(actual);
        advance_material_period(&mut reopened);
        assert_eq!(reopened.session().completed_tick(), 2);
        assert_eq!(
            DurableMaterialRuntimeV3::open(
                config,
                campaign,
                preset
                    .admitted(&crate::test_support::catalog())
                    .unwrap()
                    .digest()
            )
            .unwrap()
            .session()
            .completed_tick(),
            2
        );
    }
}

mod campaign_writer_ownership {
    use super::*;
    use babylon_persistence::{
        material_runtime::MaterialRuntimeErrorV3,
        michigan_economy::michigan_observer_foundation_v1, DurableReplayRuntimeV2,
    };
    use std::{
        thread,
        time::{Duration, Instant},
    };

    #[test]
    #[ignore = "requires the existing disposable PostgreSQL harness; independent clone ownership"]
    fn live_graph_owner_is_refused_for_an_already_registered_material_campaign() {
        let target = DisposableTarget::create();
        let campaign = CampaignId::from_uuid(Uuid::from_u128(31_001));
        let material = DurableMaterialRuntimeV3::create(
            &target.writer,
            campaign,
            MichiganContentPresetV1::new_campaign(MichiganDeliveryPresetV1::Standard)
                .create_foundation(&crate::test_support::catalog())
                .unwrap(),
        )
        .unwrap();
        let (graph, bundle) = michigan_observer_foundation_v1().unwrap();
        let graph_create = DurableReplayRuntimeV2::create(&target.writer, campaign, graph, bundle);
        let graph_open = DurableReplayRuntimeV2::open(&target.writer, campaign);
        assert_eq!(
            (graph_create.is_err(), graph_open.is_err()),
            (true, true),
            "a registered material campaign must never expose a graph-only owner"
        );
        assert_eq!(material.session().completed_tick(), 0);
        let markers: i64 = target
            .writer
            .connect(NoTls)
            .unwrap()
            .query_one(
                "SELECT count(*) FROM babylon_state.tick_commit WHERE campaign_id=$1::uuid",
                &[campaign.as_uuid()],
            )
            .unwrap()
            .get(0);
        assert_eq!(markers, 0);
    }

    #[test]
    #[ignore = "requires the existing disposable PostgreSQL harness; independent clone ownership"]
    fn live_graph_campaign_before_material_schema_still_creates_reopens_and_commits() {
        let target = DisposableTarget::create();
        let absent: bool = target.writer.connect(NoTls).unwrap().query_one(
            "SELECT pg_catalog.to_regclass('babylon_state.material_campaign_foundation_v2') IS NULL", &[],
        ).unwrap().get(0);
        assert!(
            absent,
            "this predecessor fixture must not install material ownership"
        );
        let campaign = CampaignId::from_uuid(Uuid::from_u128(31_002));
        let (graph, bundle) = michigan_observer_foundation_v1().unwrap();
        let actions =
            OrderedPracticeActionBatchV1::empty(graph.session_identity().clone(), 1).unwrap();
        let original =
            DurableReplayRuntimeV2::create(&target.writer, campaign, graph, bundle).unwrap();
        let mut reopened = DurableReplayRuntimeV2::open(&target.writer, campaign).unwrap();
        assert_eq!(
            reopened.foundation().canonical_bytes(),
            original.foundation().canonical_bytes()
        );
        let receipt = reopened
            .advance_and_commit(&mut CollectingSink::default(), &actions)
            .unwrap();
        assert_eq!(receipt.resolve_tick().get(), 1);
        assert_eq!(
            DurableReplayRuntimeV2::open(&target.writer, campaign)
                .unwrap()
                .last_committed_tick()
                .unwrap()
                .get(),
            1
        );
    }

    #[test]
    #[ignore = "requires the existing disposable PostgreSQL harness; independent clone ownership"]
    fn live_concurrent_creation_cannot_promote_the_winning_graph_campaign_to_material() {
        let target = DisposableTarget::create();
        // Warm every additive schema before the controlled interleaving. No
        // production hooks are used: a relation lock pauses the real catalog insert.
        let warm = CampaignId::from_uuid(Uuid::from_u128(31_900));
        drop(
            DurableMaterialRuntimeV3::create(
                &target.writer,
                warm,
                MichiganContentPresetV1::new_campaign(MichiganDeliveryPresetV1::Standard)
                    .create_foundation(&crate::test_support::catalog())
                    .unwrap(),
            )
            .unwrap(),
        );
        let campaign = CampaignId::from_uuid(Uuid::from_u128(31_003));
        let (graph, bundle) = michigan_observer_foundation_v1().unwrap();
        let material = MichiganContentPresetV1::new_campaign(MichiganDeliveryPresetV1::Standard)
            .create_foundation(&crate::test_support::catalog())
            .unwrap();
        let mut graph_config = target.writer.clone();
        graph_config.application_name("g4-owner-race-graph");
        let mut material_config = target.writer.clone();
        material_config.application_name("g4-owner-race-material");
        let mut holder = target.writer.connect(NoTls).unwrap();
        let mut blocker = holder.transaction().unwrap();
        blocker
            .batch_execute("LOCK TABLE babylon_meta.campaign IN SHARE MODE")
            .unwrap();
        let graph_worker = thread::spawn(move || {
            DurableReplayRuntimeV2::create(&graph_config, campaign, graph, bundle).map(|_| ())
        });
        let graph_blocked = wait_for_writer_lock(&target.writer, "g4-owner-race-graph", true);
        let material_worker = thread::spawn(move || {
            DurableMaterialRuntimeV3::create(&material_config, campaign, material).map(|_| ())
        });
        let material_blocked =
            wait_for_writer_lock(&target.writer, "g4-owner-race-material", false);
        // Release before assertions or joins, including a failed observation, so
        // no worker is stranded behind a test-owned relation lock.
        blocker.rollback().unwrap();
        let graph_result = graph_worker.join().unwrap();
        let material_result = material_worker.join().unwrap();
        assert!(
            graph_blocked && material_blocked,
            "both real writers reached the controlled lock boundary"
        );
        assert!(
            graph_result.is_ok(),
            "the graph campaign won its canonical insertion"
        );
        assert!(matches!(material_result, Err(MaterialRuntimeErrorV3::LegacyCampaign)),
            "material creation must re-observe the winning graph owner instead of adopting it: {material_result:?}");
        let mut sql = target.writer.connect(NoTls).unwrap();
        let material_rows: i64 = sql.query_one(
            "SELECT count(*) FROM babylon_state.material_campaign_foundation_v2 WHERE campaign_id=$1::uuid", &[campaign.as_uuid()],
        ).unwrap().get(0);
        assert_eq!(material_rows, 0);
        assert!(DurableReplayRuntimeV2::open(&target.writer, campaign).is_ok());
    }

    fn wait_for_writer_lock(config: &Config, application: &str, catalog_insert: bool) -> bool {
        let mut observer = config.connect(NoTls).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let waiting: bool = observer
                .query_one(
                    "SELECT EXISTS (SELECT 1 FROM pg_catalog.pg_stat_activity a \
                 WHERE a.datname=current_database() AND a.application_name=$1 \
                 AND a.wait_event_type='Lock' \
                 AND (NOT $2::boolean OR a.query LIKE '%INSERT INTO babylon_meta.campaign%'))",
                    &[&application, &catalog_insert],
                )
                .unwrap()
                .get(0);
            if waiting {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

#[path = "observer_material_live/runtime_process.rs"]
mod runtime_process;

#[path = "observer_material_live/tick_components.rs"]
mod tick_components;

#[path = "observer_material_live/staffing_admission.rs"]
mod staffing_admission;

#[path = "observer_material_live/staffing_history.rs"]
mod staffing_history;

#[path = "observer_material_live/persisted_twins.rs"]
mod persisted_twins;

#[path = "support/material_config.rs"]
mod test_support;
