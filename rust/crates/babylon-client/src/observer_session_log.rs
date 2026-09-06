//! Scoped observer telemetry through the existing Bevy tracing subscriber.
//! Requests are intentions; applied state and durable progress are separate
//! records. No raw transport errors, credentials, economic rows or stale IDs
//! enter this stream. Camera checkpoints are bounded, never raw input events.

use bevy::prelude::*;

use crate::atlas::CountyAtlas;
use crate::campaign_browser::CampaignBrowserCommand;
use crate::map::SelectedCounty;
use crate::observer::{ObservationContext, ObserverSession, Perspective, SessionPhase};
use crate::observer_audio::ObserverAudioSettings;
use crate::observer_io::ObserverSet;
use crate::observer_map3d::ObserverMapCamera;
use crate::observer_ui::{
    ObserverCommand, ObserverDisclosure, ObserverFeedback, ObserverFrame, ObserverUiState,
};
use crate::production::{PrimaryView, ProductionCamera, ProductionCommand, ProductionNavigation};
use crate::ui::dossier_card::{
    ActiveCountyDossier, DossierCampaignId, DossierFetchState, DossierPageView, DossierRefresh,
    SubjectPageRequest,
};

macro_rules! scoped_info {
    ($session:expr, $($fields:tt)*) => {
        bevy::log::info!(target: "session",
            campaign = %$session.campaign.as_uuid(),
            viewed_week = $session.viewed_tick,
            durable_week = $session.durable_tick,
            perspective = $session.perspective.label(),
            generation = $session.generation,
            $($fields)*)
    };
}

pub(crate) fn register(app: &mut App) {
    app.add_message::<ObserverCommand>()
        .add_message::<ProductionCommand>()
        .add_message::<CampaignBrowserCommand>()
        .add_systems(
            Update,
            log_requests
                .after(ObserverSet::Input)
                .before(ObserverSet::Receive)
                .run_if(resource_exists::<ObserverSession>),
        )
        .add_systems(
            Update,
            (
                log_session,
                log_feedback,
                log_presentation,
                log_dossier,
                log_camera,
            )
                .chain()
                .after(ObserverSet::Paint)
                .run_if(resource_exists::<ObserverSession>),
        );
}

fn observer_command_name(command: ObserverCommand) -> &'static str {
    match command {
        ObserverCommand::TogglePlay => "toggle_play",
        ObserverCommand::Step => "step",
        ObserverCommand::Speed => "speed",
        ObserverCommand::Perspective => "perspective",
        ObserverCommand::PreviousWeek => "previous_week",
        ObserverCommand::NextWeek => "next_week",
        ObserverCommand::Live => "live",
        ObserverCommand::Lens(_) => "lens",
        ObserverCommand::MaterialLens(_) => "material_lens",
        ObserverCommand::CycleGood(_) => "cycle_good",
        ObserverCommand::Archive => "archive",
        ObserverCommand::Menu => "menu",
        ObserverCommand::NewCampaign => "new_campaign",
        ObserverCommand::NewDelayedCampaign => "new_delayed_campaign",
        ObserverCommand::ReopenCampaign => "reopen_campaign",
        ObserverCommand::Quit => "quit",
        ObserverCommand::UiScale => "ui_scale",
        ObserverCommand::ReducedMotion => "reduced_motion",
        ObserverCommand::MusicVolume => "music_volume",
        ObserverCommand::EffectsVolume => "effects_volume",
        ObserverCommand::MusicTrack => "music_track",
        ObserverCommand::History => "history",
        ObserverCommand::StopOnDelivery => "stop_on_delivery",
        ObserverCommand::Disclosure(ObserverDisclosure::Time) => "time_controls",
        ObserverCommand::Disclosure(ObserverDisclosure::Lens) => "lens_controls",
        ObserverCommand::Evidence => "evidence",
        ObserverCommand::EconomicDetails => "economic_details",
        ObserverCommand::Relationships => "relationships",
    }
}

fn log_requests(
    session: Res<ObserverSession>,
    mut observer: MessageReader<ObserverCommand>,
    mut production: MessageReader<ProductionCommand>,
    mut browser: MessageReader<CampaignBrowserCommand>,
    mut subjects: MessageReader<SubjectPageRequest>,
) {
    for _ in subjects.read() {
        scoped_info!(
            session,
            command = "archive_subject",
            "observer command requested"
        );
    }
    for command in observer.read() {
        scoped_info!(
            session,
            command = observer_command_name(*command),
            "observer command requested"
        );
    }
    for command in production.read() {
        // Select can contain a stale or undisclosed site. Only the applied
        // projection below is permitted to log a validated subject identity.
        let name = match command {
            ProductionCommand::Open => "production_open",
            ProductionCommand::Map => "map_open",
            ProductionCommand::Flat => "production_flat",
            ProductionCommand::Back => "production_back",
            ProductionCommand::Details => "production_details",
            ProductionCommand::Select { .. } => "production_select",
        };
        scoped_info!(session, command = name, "observer command requested");
    }
    for command in browser.read() {
        let name = match command {
            CampaignBrowserCommand::Previous => "campaign_previous",
            CampaignBrowserCommand::Next => "campaign_next",
            CampaignBrowserCommand::Open => "campaign_open",
            CampaignBrowserCommand::Compare => "campaign_compare",
            CampaignBrowserCommand::CloseComparison => "comparison_close",
            CampaignBrowserCommand::Refresh => "campaign_refresh",
        };
        scoped_info!(session, command = name, "observer command requested");
    }
}

#[derive(PartialEq)]
struct SessionSnapshot {
    context: ObservationContext,
    durable: u64,
    archive: u64,
    phase: SessionPhase,
    playing: bool,
    month_target: Option<u64>,
    speed: f64,
    failed: bool,
}

fn log_session(session: Res<ObserverSession>, mut last: Local<Option<SessionSnapshot>>) {
    let next = SessionSnapshot {
        context: session.context(),
        durable: session.durable_tick,
        archive: session.archive_verified_tick,
        phase: session.phase,
        playing: session.playing,
        month_target: session.month_target_tick(),
        speed: session.weeks_per_second,
        failed: session.error.is_some(),
    };
    if last.as_ref() == Some(&next) {
        return;
    }
    if let Some(previous) = last.as_ref() {
        if previous.context.campaign == next.context.campaign && next.durable > previous.durable {
            scoped_info!(
                session,
                previous_week = previous.durable,
                "observer durable progress acknowledged"
            );
        }
    }
    scoped_info!(session, phase = ?next.phase, playing = next.playing,
        month_target_week = ?next.month_target, speed = next.speed, archive_processed_week = next.archive, failed = next.failed,
        "observer session applied");
    *last = Some(next);
}

fn log_feedback(
    session: Res<ObserverSession>,
    feedback: Option<Res<ObserverFeedback>>,
    mut last_revision: Local<u64>,
) {
    let Some(feedback) = feedback else {
        return;
    };
    if feedback.revision == *last_revision {
        return;
    }
    *last_revision = feedback.revision;
    // Feedback reasons are static control-availability text, never row labels
    // or transport errors. Revisions preserve distinct repeated denied clicks.
    if let Some(reason) = feedback.message {
        scoped_info!(session, reason, "observer command rejected");
    }
}

#[derive(PartialEq)]
// These are independent applied controls, not mutually exclusive session states.
#[allow(clippy::struct_excessive_bools)]
struct PresentationSnapshot {
    perspective: Perspective,
    lens: String,
    view: PrimaryView,
    flat: bool,
    details: bool,
    disclosure: &'static str,
    evidence: bool,
    economic_details: bool,
    site: Option<String>,
    county: Option<String>,
    archive: bool,
    menu: bool,
    splash: bool,
    history: bool,
    comparison: bool,
    reduced_motion: bool,
    stop_on_delivery: bool,
    ui_scale: f32,
    audio: Option<(f32, f32, usize)>,
}

#[allow(clippy::too_many_arguments)]
fn log_presentation(
    session: Res<ObserverSession>,
    frame: Option<Res<ObserverFrame>>,
    ui: Option<Res<ObserverUiState>>,
    view: Option<Res<PrimaryView>>,
    navigation: Option<Res<ProductionNavigation>>,
    audio: Option<Res<ObserverAudioSettings>>,
    scale: Option<Res<UiScale>>,
    selected: Option<Res<SelectedCounty>>,
    atlas: Option<Res<CountyAtlas>>,
    mut last: Local<Option<PresentationSnapshot>>,
) {
    let Some(ui) = ui else {
        return;
    };
    if last.is_some()
        && !session.is_changed()
        && !ui.is_changed()
        && !frame.as_ref().is_some_and(DetectChanges::is_changed)
        && !view.as_ref().is_some_and(DetectChanges::is_changed)
        && !navigation.as_ref().is_some_and(DetectChanges::is_changed)
        && !audio.as_ref().is_some_and(DetectChanges::is_changed)
        && !scale.as_ref().is_some_and(DetectChanges::is_changed)
        && !selected.as_ref().is_some_and(DetectChanges::is_changed)
    {
        return;
    }
    let snapshot = frame.as_ref().and_then(|frame| frame.for_session(&session));
    let site = if session.perspective == Perspective::FullObserver {
        snapshot
            .and_then(|snapshot| snapshot.production.as_ref())
            .and_then(|production| {
                let selected = navigation.as_ref()?.selected_site.as_ref()?;
                production
                    .sites
                    .iter()
                    .find(|site| &site.id == selected)
                    .map(|site| site.id.clone())
            })
    } else {
        None
    };
    let county = selected.as_ref().and_then(|selected| {
        atlas
            .as_ref()?
            .county(selected.0?)
            .map(|county| county.fips.to_owned())
    });
    let next = PresentationSnapshot {
        perspective: session.perspective,
        lens: ui.lens.label_for_log(snapshot),
        view: view.as_deref().copied().unwrap_or_default(),
        flat: navigation
            .as_ref()
            .is_some_and(|navigation| navigation.flat),
        details: navigation
            .as_ref()
            .is_some_and(|navigation| navigation.details_open),
        disclosure: match ui.disclosure {
            Some(ObserverDisclosure::Time) => "time",
            Some(ObserverDisclosure::Lens) => "lens",
            None => "none",
        },
        evidence: ui.evidence_open,
        economic_details: ui.economic_details_open,
        site,
        county,
        archive: ui.archive_open,
        menu: ui.menu_open,
        splash: ui.splash_visible,
        history: ui.history_open,
        comparison: ui.comparison_open,
        reduced_motion: ui.reduced_motion,
        stop_on_delivery: ui.stop_on_delivery,
        ui_scale: scale.as_ref().map_or(1.0, |scale| scale.0),
        audio: audio
            .as_ref()
            .map(|audio| (audio.music_volume, audio.effects_volume, audio.track)),
    };
    if last.as_ref() == Some(&next) {
        return;
    }
    scoped_info!(session, lens = %next.lens, view = ?next.view, flat = next.flat,
        details = next.details, disclosure = next.disclosure, evidence = next.evidence, economic_details = next.economic_details,
        selected_site = next.site.as_deref().unwrap_or("none_or_undisclosed"),
        county = next.county.as_deref().unwrap_or("none"), archive = next.archive,
        menu = next.menu, splash = next.splash, history = next.history,
        comparison = next.comparison, reduced_motion = next.reduced_motion,
        stop_on_delivery = next.stop_on_delivery, ui_scale = %next.ui_scale,
        music_volume = next.audio.map(|audio| audio.0),
        effects_volume = next.audio.map(|audio| audio.1),
        track = next.audio.map(|audio| audio.2), "observer presentation applied");
    *last = Some(next);
}

#[derive(PartialEq)]
struct DossierSnapshot {
    context: ObservationContext,
    county: Option<String>,
    subject: Option<String>,
    content_tick: Option<u64>,
    verified_tick: Option<u64>,
    history_floor_tick: Option<u64>,
    status: &'static str,
    page: &'static str,
}
#[allow(clippy::too_many_arguments)]
fn log_dossier(
    session: Res<ObserverSession>,
    projection: Option<Res<ActiveCountyDossier>>,
    fetch: Option<Res<DossierFetchState>>,
    campaign: Option<Res<DossierCampaignId>>,
    selected: Option<Res<SelectedCounty>>,
    atlas: Option<Res<CountyAtlas>>,
    view: Option<Res<DossierPageView>>,
    frame: Option<Res<ObserverFrame>>,
    refresh: Option<Res<DossierRefresh>>,
    mut last: Local<Option<DossierSnapshot>>,
) {
    let Some(fetch) = fetch else {
        return;
    };
    if last.is_some()
        && !session.is_changed()
        && !fetch.is_changed()
        && !selected.as_ref().is_some_and(DetectChanges::is_changed)
        && !campaign.as_ref().is_some_and(DetectChanges::is_changed)
        && !view.as_ref().is_some_and(DetectChanges::is_changed)
        && !projection.as_ref().is_some_and(DetectChanges::is_changed)
        && !frame.as_ref().is_some_and(DetectChanges::is_changed)
        && !refresh.as_ref().is_some_and(DetectChanges::is_changed)
    {
        return;
    }
    let county = selected
        .as_ref()
        .and_then(|selected| atlas.as_ref()?.county(selected.0?));
    let card = projection.as_ref().and_then(|projection| {
        if !matches!(*fetch, DossierFetchState::Idle) || campaign.as_ref()?.0 != session.campaign {
            return None;
        }
        projection.for_observer(
            &session,
            frame.as_ref()?,
            refresh.as_ref()?.0,
            county.as_ref()?.fips,
        )
    });
    let status = match (&*fetch, card) {
        (DossierFetchState::Idle, Some(read)) => match &read.state {
            babylon_persistence::archive_revision::ArchiveDossierStateV2::Ready { .. } => "ready",
            babylon_persistence::archive_revision::ArchiveDossierStateV2::Pending { .. } => {
                "pending"
            }
            babylon_persistence::archive_revision::ArchiveDossierStateV2::Unavailable(_) => {
                "unavailable"
            }
        },
        (DossierFetchState::Idle, None) => "empty",
        (DossierFetchState::InFlight { .. }, _) => "reading",
        (DossierFetchState::WaitingForObservation, _) => "waiting_for_observation",
        (
            DossierFetchState::Failed(crate::ui::dossier_card::DossierFetchError::ReaderAbsent(_)),
            _,
        ) => "reader_unavailable",
        (DossierFetchState::Failed(_), _) => "read_failed",
    };
    let next = DossierSnapshot {
        context: session.context(),
        county: card
            .and(county.as_ref())
            .map(|county| county.fips.to_owned()),
        // A requested identity is not itself a disclosure. Only a witnessed
        // retained page authorizes a subject identity in the applied log.
        subject: card
            .filter(|read| crate::dossier::retained_page(read).is_some())
            .map(|read| format!("{}:{}", read.subject.kind().as_str(), read.subject.id())),
        content_tick: card
            .and_then(crate::dossier::retained_page)
            .map(|page| page.content_source.tick()),
        verified_tick: card.and_then(crate::dossier::verified_tick),
        history_floor_tick: card.map(|read| read.history_floor_tick),
        status,
        page: if view
            .as_deref()
            .is_some_and(|view| matches!(view, DossierPageView::Subject(_)))
        {
            "subject"
        } else {
            "county"
        },
    };
    if last.as_ref() == Some(&next) {
        return;
    }
    scoped_info!(
        session,
        county = next.county.as_deref().unwrap_or("none"),
        subject = next.subject.as_deref().unwrap_or("none"),
        content_tick = next.content_tick,
        verified_tick = next.verified_tick,
        history_floor_tick = next.history_floor_tick,
        status = next.status,
        page = next.page,
        "observer archive applied"
    );
    *last = Some(next);
}

#[derive(Clone, Copy, PartialEq)]
struct CameraPose {
    position: Vec3,
    rotation: Quat,
    kind: &'static str,
    lens: f32,
    aspect_or_width: f32,
}
impl CameraPose {
    fn from_components(transform: &Transform, projection: &Projection) -> Self {
        let (kind, lens, aspect_or_width) = match projection {
            Projection::Perspective(perspective) => {
                ("perspective", perspective.fov, perspective.aspect_ratio)
            }
            Projection::Orthographic(orthographic) => (
                "orthographic",
                orthographic.scale,
                orthographic.area.width(),
            ),
            Projection::Custom(_) => ("custom", 0.0, 0.0),
        };
        Self {
            position: transform.translation,
            rotation: transform.rotation,
            kind,
            lens,
            aspect_or_width,
        }
    }
}

#[derive(Default)]
struct CameraCheckpoint {
    observed: Option<CameraPose>,
    emitted: Option<CameraPose>,
    emitted_at: f64,
    moved_at: f64,
    settled: bool,
}
impl CameraCheckpoint {
    fn sample(&mut self, pose: CameraPose, now: f64) -> Option<bool> {
        if self.observed != Some(pose) {
            self.observed = Some(pose);
            self.moved_at = now;
            self.settled = false;
        }
        let settled = now - self.moved_at >= 0.15;
        if self.emitted.is_some() && now - self.emitted_at < 0.5 {
            return None;
        }
        if self.emitted == Some(pose) && (self.settled || !settled) {
            return None;
        }
        self.emitted = Some(pose);
        self.emitted_at = now;
        self.settled = settled;
        Some(settled)
    }
}

type CameraReadings<'w, 's> = Query<
    'w,
    's,
    (
        &'static Camera,
        &'static Transform,
        &'static Projection,
        Option<&'static ObserverMapCamera>,
        Option<&'static ProductionCamera>,
    ),
>;

fn log_camera(
    session: Res<ObserverSession>,
    view: Option<Res<PrimaryView>>,
    ui: Option<Res<ObserverUiState>>,
    time: Option<Res<Time<Real>>>,
    cameras: CameraReadings,
    mut checkpoint: Local<CameraCheckpoint>,
) {
    let Some(time) = time else {
        return;
    };
    if ui
        .as_ref()
        .is_some_and(|ui| ui.menu_open || ui.splash_visible || ui.comparison_open)
    {
        return;
    }
    let view = view.as_deref().copied().unwrap_or_default();
    for (camera, transform, projection, map, production) in &cameras {
        if !camera.is_active
            || !matches!(
                (view, map.is_some(), production.is_some()),
                (PrimaryView::Map, true, _) | (PrimaryView::Production, _, true)
            )
        {
            continue;
        }
        let pose = CameraPose::from_components(transform, projection);
        let Some(settled) = checkpoint.sample(pose, time.elapsed_secs_f64()) else {
            continue;
        };
        bevy::log::debug!(target: "session", campaign = %session.campaign.as_uuid(),
            viewed_week = session.viewed_tick, durable_week = session.durable_tick,
            perspective = session.perspective.label(), generation = session.generation,
            view = ?view, projection = pose.kind, lens = pose.lens, aspect_or_width = pose.aspect_or_width,
            position = ?pose.position.to_array(), rotation = ?pose.rotation.to_array(),
            settled, "observer camera checkpoint");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use babylon_persistence::{
        CampaignId, ObserverEconomySnapshotV1, ObserverVisibilityV1, ProductionSiteV1,
        ProductionSnapshotV1,
    };
    use bevy::log::tracing_subscriber::layer::SubscriberExt as _;
    use std::time::Duration;

    const HIDDEN_SITE: &str = "undisclosed-production-id";
    const HIDDEN_LABEL: &str = "Undisclosed factory label";

    fn snapshot(session: &ObserverSession) -> ObserverEconomySnapshotV1 {
        ObserverEconomySnapshotV1 {
            campaign_id: session.campaign.as_uuid().to_string(),
            resolve_tick: session.viewed_tick,
            foundation_digest: "foundation".into(),
            nominal_world_hash: None,
            tick_content_hash: session.content_hash.clone(),
            envelope_digest: None,
            visibility: ObserverVisibilityV1::FullObserver,
            counties: Vec::new(),
            production: Some(ProductionSnapshotV1 {
                material_balance: None,
                labor_accounts: Vec::new(),
                scenario_label: "Designed telemetry fixture".into(),
                horizon_week: 16,
                sites: vec![ProductionSiteV1 {
                    id: HIDDEN_SITE.into(),
                    name: HIDDEN_LABEL.into(),
                    county_geoid: "26163".into(),
                    industry_code: "331".into(),
                    observed_employment: None,
                    output_good_id: "hidden-good-id".into(),
                    output_unit_id: "hidden-unit-id".into(),
                    output_good: "hidden-good-name".into(),
                    output_unit: "kg".into(),
                    output_per_batch: 1,
                    available_batches: 1,
                    planned_batches: None,
                    produced_batches: None,
                    inventory: Vec::new(),
                    inputs: Vec::new(),
                    labor: Vec::new(),
                }],
                routes: Vec::new(),
                freight: Vec::new(),
                events: Vec::new(),
                observed_contexts: Vec::new(),
                process_attributions: Vec::new(),
                provenance: Vec::new(),
            }),
        }
    }

    fn captured(drive: impl FnOnce(&mut App)) -> String {
        let path = std::env::temp_dir().join(format!(
            "babylon-observer-telemetry-{}",
            uuid::Uuid::new_v4()
        ));
        let sink =
            crate::logging::RotatingSink::open(&path, 1024 * 1024, 2).expect("rotating sink");
        let subscriber = bevy::log::tracing_subscriber::registry().with(
            bevy::log::tracing_subscriber::fmt::Layer::default()
                .with_ansi(false)
                .with_writer(sink),
        );
        let mut app = App::new();
        let mut session = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::nil()));
        session.foundation_digest = Some("foundation".into());
        session.ready(0, None);
        assert!(session.installed(&session.context()));
        app.insert_resource(ObserverFrame(Some(snapshot(&session))))
            .insert_resource(session)
            .insert_resource(ObserverUiState {
                menu_open: false,
                splash_visible: false,
                ..default()
            })
            .init_resource::<PrimaryView>()
            .init_resource::<ProductionNavigation>()
            .init_resource::<ObserverAudioSettings>()
            .init_resource::<UiScale>()
            .init_resource::<Time<Real>>()
            .add_plugins(crate::session_log::SessionLogPlugin);
        app.edit_schedule(Startup, |schedule| {
            schedule.set_executor_kind(bevy::ecs::schedule::ExecutorKind::SingleThreaded);
        });
        app.edit_schedule(Update, |schedule| {
            schedule.set_executor_kind(bevy::ecs::schedule::ExecutorKind::SingleThreaded);
        });
        bevy::log::tracing::subscriber::with_default(subscriber, || {
            app.update();
            drive(&mut app);
            app.update();
        });
        let log = std::fs::read_to_string(path.join("babylon-client.log")).expect("captured log");
        std::fs::remove_dir_all(path).expect("remove exact owned test log");
        log
    }

    #[test]
    fn a_scoped_undisclosed_archive_response_does_not_log_the_requested_identity() {
        use crate::ui::dossier_card::{DossierRequestScope, InstalledDossier};
        use babylon_persistence::archive_revision::{
            ArchiveDossierReadV2, ArchiveDossierStateV2, ArchiveDossierUnavailableV2,
            ArchiveReadScopeV2,
        };
        use babylon_persistence::{ArchivePageRefV1, ArchiveSubjectKindV1};

        let log = captured(|app| {
            {
                let mut session = app.world_mut().resource_mut::<ObserverSession>();
                session.ready(1, Some("01".repeat(32)));
                let context = session.context();
                assert!(session.installed(&context));
            }
            let session = app.world().resource::<ObserverSession>();
            let campaign = session.campaign;
            let observer = session.context();
            let frame = snapshot(session);
            let atlas = CountyAtlas::parse(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../assets/map/county_atlas.bin"
            )))
            .unwrap();
            let wayne = atlas.index_of_fips("26163").unwrap();
            let scope = ArchiveReadScopeV2::committed(campaign, 1, [1; 32]).unwrap();
            let subject =
                ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, "2674900".into()).unwrap();
            app.insert_resource(atlas)
                .insert_resource(SelectedCounty(Some(wayne)))
                .insert_resource(DossierCampaignId(campaign))
                .insert_resource(ObserverFrame(Some(frame)))
                .init_resource::<DossierRefresh>()
                .insert_resource(ActiveCountyDossier(Some(InstalledDossier {
                    scope: DossierRequestScope {
                        campaign,
                        county_geoid: "26163".into(),
                        refresh_generation: 0,
                        observer: Some(observer),
                        read_scope: scope.clone(),
                        subject: subject.clone(),
                    },
                    read: ArchiveDossierReadV2 {
                        scope,
                        subject,
                        durable_tick: 1,
                        processed_tick: 1,
                        history_floor_tick: 0,
                        state: ArchiveDossierStateV2::Unavailable(
                            ArchiveDossierUnavailableV2::SubjectNotDisclosed,
                        ),
                    },
                })));
        });
        assert!(log.contains("status=\"unavailable\""), "{log}");
        assert!(log.contains("subject=\"none\""), "{log}");
        assert!(!log.contains("2674900"), "{log}");
    }

    #[test]
    fn requested_controls_and_acknowledged_state_use_distinct_records() {
        let log = captured(|app| {
            app.world_mut()
                .resource_mut::<Messages<ObserverCommand>>()
                .write(ObserverCommand::Step);
            app.update();
            // The request alone cannot advance durability. The real session
            // state machine still requires a matching acknowledgement.
            assert_eq!(app.world().resource::<ObserverSession>().durable_tick, 0);
            let mut session = app.world_mut().resource_mut::<ObserverSession>();
            let request = session.begin_advance().expect("ready advance");
            assert!(session.acknowledge(request, 1, Some("committed".into())));
            session.archive_verified_tick = 1;
            session.playing = true;
            session.weeks_per_second = 2.0;
            app.update();
            let mut ui = app.world_mut().resource_mut::<ObserverUiState>();
            ui.history_open = true;
            ui.reduced_motion = true;
            ui.stop_on_delivery = true;
            ui.menu_open = true;
            ui.comparison_open = true;
            ui.evidence_open = true;
            ui.disclosure = Some(ObserverDisclosure::Time);
            app.world_mut()
                .resource_mut::<ProductionNavigation>()
                .details_open = true;
            app.world_mut().resource_mut::<UiScale>().0 = 1.15;
            let mut audio = app.world_mut().resource_mut::<ObserverAudioSettings>();
            audio.track = 1;
            audio.music_volume = 0.5;
            audio.effects_volume = 0.0;
        });
        let requested = log
            .find("observer command requested")
            .expect("request record");
        let acknowledged = log
            .find("observer durable progress acknowledged")
            .expect("ack record");
        assert!(requested < acknowledged, "{log}");
        assert!(log.contains("command=\"step\""), "{log}");
        for field in [
            "durable_week=1",
            "archive_processed_week=1",
            "playing=true",
            "speed=2",
            "history=true",
            "reduced_motion=true",
            "comparison=true",
            "details=true",
            "disclosure=\"time\"",
            "evidence=true",
            "ui_scale=1.15",
            "track=1",
        ] {
            assert!(log.contains(field), "missing {field}: {log}");
        }
        assert!(
            log.contains("campaign=00000000-0000-0000-0000-000000000000"),
            "{log}"
        );
        assert!(log.contains("generation=2"), "{log}");
    }

    #[test]
    fn denied_clicks_log_each_revision_without_repeating_visible_feedback() {
        let log = captured(|app| {
            app.insert_resource(ObserverFeedback {
                message: Some("Wait for the current week to finish."),
                revision: 1,
                expires_at: 10.0,
            });
            app.update();
            for _ in 0..5 {
                app.update();
            }
            app.world_mut().resource_mut::<ObserverFeedback>().revision = 2;
            app.update();
            app.world_mut().resource_mut::<ObserverFeedback>().message = None;
            app.update();
        });
        assert_eq!(log.matches("observer command rejected").count(), 2, "{log}");
        assert_eq!(
            log.matches("Wait for the current week to finish.").count(),
            2,
            "{log}"
        );
        assert!(
            !log.contains("observer durable progress acknowledged"),
            "{log}"
        );
    }

    #[test]
    fn preview_clears_logged_subjects_and_never_serializes_stale_requests_or_errors() {
        let log = captured(|app| {
            app.world_mut()
                .resource_mut::<ProductionNavigation>()
                .selected_site = Some(HIDDEN_SITE.into());
            app.update(); // Valid FullObserver selection can be identified.
            app.world_mut()
                .resource_mut::<ObserverSession>()
                .set_perspective(Perspective::PlayerKnowledge);
            app.world_mut().resource_mut::<ObserverUiState>().lens =
                crate::map_economy_lens::MapLens::Material {
                    kind: crate::map_economy_lens::MaterialLensKind::ProducedThisWeek,
                    good: Some(crate::map_economy_lens::MaterialGoodKey {
                        good_id: "hidden-good-id".into(),
                        unit_id: "hidden-unit-id".into(),
                    }),
                };
            let stale_context = ObservationContext {
                perspective: Perspective::FullObserver,
                ..app.world().resource::<ObserverSession>().context()
            };
            app.world_mut()
                .resource_mut::<Messages<ProductionCommand>>()
                .write(ProductionCommand::Select {
                    site_id: "never-disclose-this-request".into(),
                    context: stale_context,
                });
            // Both navigation and the old full-observer snapshot remain in
            // memory. The telemetry capability check must reject them itself.
            app.update();
            app.world_mut()
                .resource_mut::<ObserverSession>()
                .fail("postgres://user:secret@private".into());
            *app.world_mut().resource_mut::<DossierFetchState>() =
                DossierFetchState::Failed(crate::ui::dossier_card::DossierFetchError::ReadFailed(
                    "password=never-log-this".into(),
                ));
        });
        assert!(log.contains(HIDDEN_SITE), "valid observer identity: {log}");
        let preview = log
            .split_once("perspective=\"PLAYER KNOWLEDGE\"")
            .expect("preview event")
            .1;
        assert!(
            !preview.contains(HIDDEN_SITE),
            "stale selected identity: {preview}"
        );
        assert!(
            preview.contains("selected_site=\"none_or_undisclosed\""),
            "clear event: {preview}"
        );
        for hidden in [
            HIDDEN_LABEL,
            "never-disclose-this-request",
            "password=",
            "postgres://",
            "hidden-good",
        ] {
            assert!(!log.contains(hidden), "sensitive {hidden}: {log}");
        }
        assert!(
            log.contains("command=\"production_select\""),
            "request without ID: {log}"
        );
        assert!(
            log.contains("status=\"read_failed\""),
            "bounded failure classification: {log}"
        );
    }

    #[test]
    fn idle_change_marks_do_not_repeat_applied_events() {
        let log = captured(|app| {
            for _ in 0..20 {
                app.world_mut()
                    .resource_mut::<ObserverSession>()
                    .set_changed();
                app.world_mut()
                    .resource_mut::<ObserverUiState>()
                    .set_changed();
                app.world_mut()
                    .resource_mut::<ProductionNavigation>()
                    .set_changed();
                app.world_mut()
                    .resource_mut::<ObserverAudioSettings>()
                    .set_changed();
                app.update();
            }
        });
        assert_eq!(log.matches("observer session applied").count(), 1, "{log}");
        assert_eq!(
            log.matches("observer presentation applied").count(),
            1,
            "{log}"
        );
        assert_eq!(log.matches("observer archive applied").count(), 1, "{log}");
        assert!(!log.contains("observer command requested"), "{log}");
    }

    #[test]
    fn camera_logs_bounded_checkpoints_and_the_final_settled_pose() {
        let log = captured(|app| {
            let camera = app
                .world_mut()
                .spawn((
                    Camera::default(),
                    Transform::IDENTITY,
                    Projection::Perspective(PerspectiveProjection::default()),
                    ObserverMapCamera,
                ))
                .id();
            for position in 0_u16..100 {
                app.world_mut()
                    .resource_mut::<Time<Real>>()
                    .advance_by(Duration::from_millis(10));
                app.world_mut()
                    .get_mut::<Transform>(camera)
                    .expect("camera")
                    .translation
                    .x = f32::from(position);
                app.update();
            }
            for _ in 0..100 {
                app.world_mut()
                    .resource_mut::<Time<Real>>()
                    .advance_by(Duration::from_millis(10));
                app.update();
            }
        });
        assert_eq!(
            log.matches("observer camera checkpoint").count(),
            4,
            "{log}"
        );
        let last = log
            .lines()
            .filter(|line| line.contains("observer camera checkpoint"))
            .next_back()
            .expect("pose");
        assert!(
            last.contains("position=[99.0, 0.0, 0.0]") && last.contains("settled=true"),
            "{last}"
        );
    }
}
