//! Conformance session and Archive interaction telemetry through the shared
//! Bevy tracing sink. These legacy observers cover the in-process viewer's
//! resources; they do not describe the durable observer clock or camera.
//!
//! Observer composition uses the internal `observer_session_log` module: scoped
//! requests, applied state, acknowledgements and bounded camera checkpoints.
//! Neither stream records every input or proves that a person understood it.
//! Value snapshots suppress repeated events from spurious change marks.

use bevy::prelude::*;

use crate::atlas::CountyAtlas;
use crate::loop_ui::TickCounter;
use crate::map::SelectedCounty;
use crate::story::SelectedStory;
use crate::ui::beats::BeatLog;
use crate::ui::dossier_card::{
    ActiveCountyDossier, DossierCampaignId, DossierFetchState, DossierPageView, InstalledDossier,
    SubjectPageRequest,
};
use crate::ui::time::{AutopauseMode, RunState, SPEEDS_PER_SECOND};

/// The snapshot state every value-diffing observer keeps: nothing observed
/// yet (the baseline pass), or the last logged value — which may itself be an
/// absent selection or an empty projection slot.
#[derive(Clone, PartialEq, Default)]
enum Snapshot<T> {
    #[default]
    Unseen,
    Seen(T),
}

/// Wires the session observers into an `App`. Added to the windowed build
/// after [`crate::ui::dossier_card::DossierCardPlugin`] so its message
/// registration exists; every observer tolerates a missing resource family so
/// headless test compositions can add this plugin alone.
pub struct SessionLogPlugin;

impl Plugin for SessionLogPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SubjectPageRequest>()
            .init_resource::<SelectedCounty>()
            .init_resource::<ActiveCountyDossier>()
            .init_resource::<DossierPageView>()
            .init_resource::<DossierFetchState>()
            .add_systems(
                Startup,
                log_session_start.run_if(not(resource_exists::<crate::observer::ObserverSession>)),
            )
            .add_systems(
                Update,
                (
                    log_selection_changes,
                    log_subject_page_requests,
                    log_page_view_changes,
                    log_dossier_projection_changes,
                    log_fetch_state_changes,
                    log_control_changes,
                    log_story_changes,
                    log_tick_and_beats,
                )
                    .chain()
                    .run_if(not(resource_exists::<crate::observer::ObserverSession>)),
            );
        crate::observer_session_log::register(app);
    }
}

/// `Startup`: the session's frame of reference — which campaign the dossier
/// surfaces read under and which story the in-process world runs.
fn log_session_start(campaign: Option<Res<DossierCampaignId>>, story: Option<Res<SelectedStory>>) {
    match campaign {
        Some(campaign) => {
            bevy::log::info!(target: "session", "session start campaign={}", campaign.0.as_uuid());
        }
        None => {
            bevy::log::info!(target: "session", "session start campaign=<dossier surfaces absent>");
        }
    }
    if let Some(story) = story {
        bevy::log::info!(
            target: "session",
            "session start story id={} title={:?}",
            story.0.id,
            story.0.title
        );
    }
}

/// `Update`: county selection changes — the player's map clicks. The initial
/// empty selection is the baseline, not an event.
fn log_selection_changes(
    selected: Res<SelectedCounty>,
    atlas: Option<Res<CountyAtlas>>,
    mut last: Local<Snapshot<Option<usize>>>,
) {
    let current = selected.0;
    if matches!(&*last, Snapshot::Seen(previous) if previous == &current) {
        return;
    }
    let first = matches!(*last, Snapshot::Unseen);
    *last = Snapshot::Seen(current);
    if first {
        return;
    }
    let Some(index) = current else {
        bevy::log::info!(target: "session", "county selection cleared");
        return;
    };
    if let Some(county) = atlas.as_deref().and_then(|atlas| atlas.county(index)) {
        bevy::log::info!(
            target: "session",
            "county selected fips={} name={:?}",
            county.fips,
            county.name
        );
    } else {
        bevy::log::info!(
            target: "session",
            "county selected index={index} (outside the atlas)"
        );
    }
}

/// A pending request is not an admitted observation. Record only its static
/// action kind; the scope-checked page installation has its separate record.
fn log_subject_page_requests(mut requests: MessageReader<SubjectPageRequest>) {
    for request in requests.read() {
        let kind = if request.kind == "place" {
            "place"
        } else {
            "unrecognized"
        };
        bevy::log::info!(target: "session", "subject page requested kind={kind}");
    }
}

/// `Update`: county or linked-subject navigation within the same Archive panel.
fn log_page_view_changes(view: Res<DossierPageView>, mut last: Local<Snapshot<DossierPageView>>) {
    let current = &*view;
    if matches!(&*last, Snapshot::Seen(previous) if previous == current) {
        return;
    }
    let first = matches!(*last, Snapshot::Unseen);
    *last = Snapshot::Seen(current.clone());
    if first {
        return;
    }
    if let DossierPageView::Subject(request) = current {
        bevy::log::info!(
            target: "session",
            "page view: subject kind={} id={}",
            request.kind,
            request.id
        );
    } else {
        bevy::log::info!(target: "session", "page view: county card");
    }
}

/// `Update`: dossier projection installs and clears — what the card actually
/// composed from, at field-count resolution (the atoms themselves stay in the
/// Archive; the log records that they arrived).
fn log_dossier_projection_changes(
    projection: Res<ActiveCountyDossier>,
    mut last: Local<Snapshot<Option<InstalledDossier>>>,
) {
    let current = &projection.0;
    if matches!(&*last, Snapshot::Seen(previous) if previous == current) {
        return;
    }
    let first = matches!(*last, Snapshot::Unseen);
    *last = Snapshot::Seen(current.clone());
    if first {
        return;
    }
    if let Some(installed) = current {
        let read = &installed.read;
        let page = crate::dossier::retained_page(read);
        bevy::log::info!(
            target: "session",
            "dossier installed geoid={} title={:?} atoms={} links={} changes={} requested={} durable={} processed={} verified={:?} availability={}",
            installed.scope.county_geoid,
            page.map(|page| page.title.as_str()),
            page.map_or(0, |page| page.atoms.len()),
            page.map_or(0, |page| page.links.len()),
            page.map_or(0, |page| page.changes.changes.len()),
            read.scope.tick(),
            read.durable_tick,
            read.processed_tick,
            crate::dossier::verified_tick(read),
            crate::dossier::availability_label(read)
        );
    } else {
        bevy::log::info!(target: "session", "dossier cleared");
    }
}

/// `Update`: the fetch lifecycle, so a card that shows "Archive reader not
/// configured" or a hard failure is explained by the log line that precedes
/// it. Snapshots a descriptor string: `DossierFetchState` holds the in-flight
/// `Task` (no `PartialEq`); the descriptor diff suppresses unchanged lifecycle
/// descriptions.
fn log_fetch_state_changes(state: Res<DossierFetchState>, mut last: Local<Snapshot<String>>) {
    let current = match &*state {
        DossierFetchState::Idle => "idle".to_owned(),
        DossierFetchState::WaitingForObservation => "waiting-for-observation".to_owned(),
        DossierFetchState::InFlight { scope, .. } => format!("in-flight:{}", scope.county_geoid),
        DossierFetchState::Failed(crate::ui::dossier_card::DossierFetchError::ReaderAbsent(_)) => {
            "failed:ReaderAbsent".to_owned()
        }
        DossierFetchState::Failed(crate::ui::dossier_card::DossierFetchError::ReadFailed(_)) => {
            "failed:ReadFailed".to_owned()
        }
    };
    if matches!(&*last, Snapshot::Seen(previous) if previous == &current) {
        return;
    }
    let first = matches!(*last, Snapshot::Unseen);
    *last = Snapshot::Seen(current.clone());
    if first {
        return;
    }
    if let Some(failure) = current.strip_prefix("failed:") {
        bevy::log::info!(target: "session", "dossier fetch failed: {failure}");
    } else if let Some(fips) = current.strip_prefix("in-flight:") {
        bevy::log::info!(target: "session", "dossier fetch started fips={fips}");
    } else {
        bevy::log::info!(target: "session", "dossier fetch: {current}");
    }
}

/// `Update`: the sim clock's control plane — pause/resume, speed steps,
/// autopause flips. `accumulator` churns every frame by design, so the
/// snapshot is the three fields a keypress can move. Unlike the interaction
/// plane, the baseline IS logged: a session record starts from the controls
/// it started with.
fn log_control_changes(
    run_state: Option<Res<RunState>>,
    mut last: Local<Snapshot<(bool, usize, AutopauseMode)>>,
) {
    let Some(run_state) = run_state else {
        return;
    };
    let current = (
        run_state.running,
        run_state.speed_index,
        run_state.autopause,
    );
    if matches!(&*last, Snapshot::Seen(previous) if previous == &current) {
        return;
    }
    let first = matches!(*last, Snapshot::Unseen);
    *last = Snapshot::Seen(current);
    let (running, speed_index, autopause) = current;
    let speed = SPEEDS_PER_SECOND.get(speed_index).copied().unwrap_or(0.0);
    if first {
        bevy::log::info!(
            target: "session",
            "controls at start: running={running} speed={speed}t/s autopause={autopause:?}"
        );
    } else {
        bevy::log::info!(
            target: "session",
            "controls changed: running={running} speed={speed}t/s autopause={autopause:?}"
        );
    }
}

/// `Update`: story switches — the N-key restart replaces the whole engine
/// session, so the story identity is part of the session's frame of
/// reference. The baseline is logged for the same reason as the controls.
fn log_story_changes(story: Option<Res<SelectedStory>>, mut last: Local<Snapshot<&'static str>>) {
    let Some(story) = story else {
        return;
    };
    let current = story.0.id;
    if matches!(&*last, Snapshot::Seen(previous) if *previous == current) {
        return;
    }
    let first = matches!(*last, Snapshot::Unseen);
    *last = Snapshot::Seen(current);
    if first {
        bevy::log::info!(target: "session", "story at start: id={current}");
    } else {
        bevy::log::info!(target: "session", "story restarted id={current}");
    }
}

/// `Update`: the tick spine. The heartbeat is `DEBUG` (file-only); beats
/// drained during the tick are `INFO` — they are the "what happened" stream
/// the on-screen beat feed renders. Beats are matched by tick rather than
/// tracked by cursor, so the 512-cap eviction can never desync the log; the
/// baseline tick logs no beats (they are history, not this session's events).
fn log_tick_and_beats(
    counter: Option<Res<TickCounter>>,
    beats: Option<Res<BeatLog>>,
    mut last: Local<Snapshot<i64>>,
) {
    let Some(counter) = counter else {
        return;
    };
    let current = counter.0;
    if matches!(&*last, Snapshot::Seen(previous) if *previous == current) {
        return;
    }
    let first = matches!(*last, Snapshot::Unseen);
    *last = Snapshot::Seen(current);
    if first {
        bevy::log::info!(target: "session", "tick heartbeat starts at tick={current}");
        return;
    }
    bevy::log::debug!(target: "session", "tick {current}");
    if let Some(beats) = beats {
        for beat in beats.beats.iter().filter(|beat| beat.tick == current) {
            bevy::log::info!(
                target: "session",
                "beat tick={} type={} tier={:?} delta={:?}",
                beat.tick,
                beat.event_type,
                beat.tier,
                beat.magnitude_delta
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::RotatingSink;
    use crate::severity::SeverityTier;
    use crate::ui::beats::Beat;
    use crate::ui::dossier_card::DossierFetchError;
    use babylon_persistence::archive_revision::{
        ArchiveChangePageV2, ArchiveDossierPageV2, ArchiveDossierReadV2, ArchiveDossierStateV2,
        ArchivePublicationOriginV2, ArchiveReadScopeV2,
    };
    use babylon_persistence::{ArchivePageRefV1, ArchiveSubjectKindV1};
    use bevy::log::tracing_subscriber::layer::SubscriberExt as _;
    use std::collections::VecDeque;
    use std::path::PathBuf;

    const ATLAS_BYTES: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../assets/map/county_atlas.bin"
    ));

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "babylon-session-logtest-{tag}-{}",
            std::process::id()
        ));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).expect("test log dir");
        dir
    }

    /// Boot the session app with the REAL rotating sink + fmt layer as the
    /// subscriber (never a copied pipeline), run one baseline update (Startup
    /// plus the observers' first-observation pass), then `drive` the
    /// interaction and one more update, and return the live file's contents.
    /// The schedules run single-threaded so the thread-local subscriber
    /// captures every system — the default multi-threaded executor would run
    /// them on the global `ComputeTaskPool`, where `with_default` does not
    /// reach.
    fn run_session_app(dir: &std::path::Path, drive: impl Fn(&mut App)) -> String {
        let sink = RotatingSink::open(dir, 1024 * 1024, 2).expect("sink opens");
        let layer = bevy::log::tracing_subscriber::fmt::Layer::default()
            .with_ansi(false)
            .with_writer(sink);
        let subscriber = bevy::log::tracing_subscriber::registry().with(layer);
        let mut app = App::new();
        app.add_plugins(SessionLogPlugin)
            .insert_resource(CountyAtlas::parse(ATLAS_BYTES).expect("committed atlas parses"));
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
        std::fs::read_to_string(dir.join("babylon-client.log")).expect("live log")
    }

    #[test]
    fn a_full_interaction_sequence_lands_in_the_file_log() {
        let dir = temp_dir("sequence");
        let log = run_session_app(&dir, |app| {
            let wayne = app
                .world()
                .resource::<CountyAtlas>()
                .index_of_fips("26163")
                .expect("the committed atlas carries Wayne County");
            app.world_mut().resource_mut::<SelectedCounty>().0 = Some(wayne);
            let scope = crate::ui::dossier_card::DossierRequestScope {
                campaign: DossierCampaignId::default().0,
                county_geoid: "26163".into(),
                refresh_generation: 0,
                observer: None,
                read_scope: ArchiveReadScopeV2::committed(
                    DossierCampaignId::default().0,
                    2,
                    [2; 32],
                )
                .unwrap(),
                subject: ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, "26163".into())
                    .unwrap(),
            };
            app.world_mut()
                .resource_mut::<Messages<SubjectPageRequest>>()
                .write(SubjectPageRequest {
                    scope: scope.clone(),
                    kind: "place".to_owned(),
                    id: "2674900".to_owned(),
                    label: None,
                });
            *app.world_mut().resource_mut::<DossierPageView>() =
                DossierPageView::Subject(Box::new(SubjectPageRequest {
                    scope: scope.clone(),
                    kind: "place".to_owned(),
                    id: "2674900".to_owned(),
                    label: None,
                }));
            app.world_mut().resource_mut::<ActiveCountyDossier>().0 = Some(InstalledDossier {
                read: ArchiveDossierReadV2 {
                    scope: scope.read_scope.clone(),
                    subject: scope.subject.clone(),
                    durable_tick: 2,
                    processed_tick: 2,
                    history_floor_tick: 0,
                    state: ArchiveDossierStateV2::Ready {
                        verified_through_tick: 2,
                        page: ArchiveDossierPageV2 {
                            revision_id: [1; 32],
                            effective_tick: 1,
                            origin: ArchivePublicationOriginV2::Materialized,
                            content_source: ArchiveReadScopeV2::committed(
                                scope.campaign,
                                1,
                                [1; 32],
                            )
                            .unwrap(),
                            title: "Wayne County".into(),
                            question: "What changed?".into(),
                            signals: Vec::new(),
                            markdown: String::new(),
                            content_sha256: [0; 32],
                            citations: Vec::new(),
                            atoms: Vec::new(),
                            links: Vec::new(),
                            changes: ArchiveChangePageV2 {
                                coverage_from_tick: 0,
                                changes: Vec::new(),
                                next_cursor: None,
                            },
                        },
                    },
                },
                scope,
            });
        });
        assert!(log.contains("session start campaign="), "startup: {log}");
        assert!(
            log.contains("county selected fips=26163"),
            "selection: {log}"
        );
        assert!(
            log.contains("subject page requested kind=place"),
            "chip: {log}"
        );
        assert!(
            log.contains("page view: subject kind=place id=2674900"),
            "view: {log}"
        );
        assert!(
            log.contains("dossier installed geoid=26163 title=Some(\"Wayne County\") atoms=0 links=0 changes=0 requested=2 durable=2 processed=2 verified=Some(2)"),
            "install: {log}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn pending_subject_log_never_emits_a_label_after_scope_changes() {
        use crate::observer::{ObserverSession, Perspective};
        use crate::ui::dossier_card::{DossierRefresh, DossierRequestScope};
        let dir = temp_dir("pending-scope");
        let sink = RotatingSink::open(&dir, 1024 * 1024, 2).unwrap();
        let layer = bevy::log::tracing_subscriber::fmt::Layer::default()
            .with_ansi(false)
            .with_writer(sink);
        let subscriber = bevy::log::tracing_subscriber::registry().with(layer);
        let mut app = App::new();
        app.add_message::<SubjectPageRequest>()
            .init_resource::<DossierRefresh>()
            .add_systems(Update, log_subject_page_requests);
        let campaign = DossierCampaignId::default().0;
        app.insert_resource(ObserverSession::new(campaign));
        app.edit_schedule(Update, |schedule| {
            schedule.set_executor_kind(bevy::ecs::schedule::ExecutorKind::SingleThreaded);
        });
        bevy::log::tracing::subscriber::with_default(subscriber, || {
            for perspective_change in [false, true] {
                let scope = DossierRequestScope {
                    campaign,
                    county_geoid: "26163".into(),
                    refresh_generation: app.world().resource::<DossierRefresh>().0,
                    observer: Some(app.world().resource::<ObserverSession>().context()),
                    read_scope: ArchiveReadScopeV2::foundation(campaign),
                    subject: ArchivePageRefV1::try_new(
                        ArchiveSubjectKindV1::County,
                        "26163".into(),
                    )
                    .unwrap(),
                };
                app.world_mut().write_message(SubjectPageRequest {
                    scope,
                    kind: "place".into(),
                    id: "private-place-id".into(),
                    label: Some("withheld-place-label".into()),
                });
                if perspective_change {
                    app.world_mut()
                        .resource_mut::<ObserverSession>()
                        .set_perspective(Perspective::PlayerKnowledge);
                } else {
                    app.world_mut().resource_mut::<DossierRefresh>().bump();
                }
                app.update();
            }
        });
        let log = std::fs::read_to_string(dir.join("babylon-client.log")).unwrap();
        assert_eq!(log.matches("subject page requested kind=place").count(), 2);
        assert!(!log.contains("private-place-id"));
        assert!(!log.contains("withheld-place-label"));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn clearing_the_selection_and_failing_a_fetch_are_logged_honestly() {
        let dir = temp_dir("clear-fail");
        let log = run_session_app(&dir, |app| {
            app.world_mut().resource_mut::<SelectedCounty>().0 = Some(usize::MAX);
            app.update();
            app.world_mut().resource_mut::<SelectedCounty>().0 = None;
            *app.world_mut().resource_mut::<DossierFetchState>() = DossierFetchState::Failed(
                DossierFetchError::ReaderAbsent("BABYLON_READER_DSN unset".to_owned()),
            );
        });
        let expected = format!("county selected index={} (outside the atlas)", usize::MAX);
        assert!(log.contains(&expected), "select: {log}");
        assert!(log.contains("county selection cleared"), "clear: {log}");
        assert!(
            log.contains("dossier fetch failed: ReaderAbsent"),
            "fail: {log}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn spurious_change_marks_do_not_flood_the_log() {
        // The Codex review case: `collect_dossier_fetch` marks the fetch state
        // changed on every poll, so change-detection alone would re-log every
        // frame. The snapshot diff must emit one line for one real transition
        // no matter how many times the resource is touched unchanged.
        let dir = temp_dir("flood");
        let log = run_session_app(&dir, |app| {
            *app.world_mut().resource_mut::<DossierFetchState>() = DossierFetchState::Failed(
                DossierFetchError::ReaderAbsent("BABYLON_READER_DSN unset".to_owned()),
            );
            app.update();
            for _ in 0..3 {
                // A producer-side `ResMut` deref with no value change.
                app.world_mut()
                    .resource_mut::<DossierFetchState>()
                    .set_changed();
                app.update();
            }
        });
        assert_eq!(
            log.matches("dossier fetch failed").count(),
            1,
            "one transition, one line: {log}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn control_transitions_story_and_the_tick_spine_are_logged() {
        let dir = temp_dir("controls");
        let log = run_session_app(&dir, |app| {
            app.insert_resource(RunState::default());
            app.insert_resource(TickCounter(41));
            let mut beats = BeatLog::default();
            beats.beats = VecDeque::from([Beat {
                tick: 42,
                event_type: "LIFECYCLE_TRANSITION".to_owned(),
                payload: Vec::new(),
                tier: SeverityTier::Informational,
                magnitude_delta: Some(1.0),
            }]);
            app.insert_resource(beats);
            app.update();
            app.world_mut().resource_mut::<RunState>().running = false;
            app.world_mut().resource_mut::<TickCounter>().0 = 42;
            app.update();
        });
        assert!(
            log.contains("controls at start: running=true speed=5t/s autopause=OnCritical"),
            "baseline controls: {log}"
        );
        assert!(
            log.contains("controls changed: running=false"),
            "pause: {log}"
        );
        assert!(
            log.contains("tick heartbeat starts at tick=41"),
            "spine: {log}"
        );
        assert!(log.contains("tick 42"), "heartbeat: {log}");
        assert!(
            log.contains("beat tick=42 type=LIFECYCLE_TRANSITION"),
            "beat: {log}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
