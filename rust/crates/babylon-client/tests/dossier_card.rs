//! PER-23 Slice 4 harness (ADR249 R5/R6/R9): the county dossier card's real
//! headless proofs. Legacy conformance uses `MapPlugin`, `TickLoopPlugin`,
//! and `DossierCardPlugin`. The observer proof uses the shared shell, history,
//! and dossier plugins. Both run on `MinimalPlugins` with real resources,
//! messages, and pointer events — the composition discipline `tests/projection.rs`
//! established (direct `ButtonInput::press` is wiped by `InputPlugin`'s
//! `PreUpdate`, so key presses ride real `KeyboardInput` messages).
//!
//! The parity thesis under test: the card's repaint derives ONLY from
//! `ActiveCountyDossier`, `DossierFetchState`, and `DossierPageView`, so a
//! seeded projection renders byte-identical zone text headless that the
//! windowed viewer renders from a real fetch — one resource family, one
//! paint path (the rationale `ui::dossier_card`'s module doc commits to).

use babylon_client::atlas::CountyAtlas;
use babylon_client::decision_surface::{DeclaredSurface, SurfaceId};
use babylon_client::map::SelectedCounty;
use babylon_client::palette;
use babylon_client::story;
use babylon_client::ui::dossier_card::{
    ActiveCountyDossier, DossierCampaignId, DossierCardPlugin, DossierCardRoot, DossierFetchError,
    DossierFetchState, DossierPageView, DossierRefresh, DossierRequestScope, DossierZone,
    InstalledDossier,
};
use babylon_client::ui::dossier_compose::{DOSSIER_DECISION_QUESTION, INVESTIGATE_SEALED_CHIP};
use babylon_persistence::archive_revision::{
    ArchiveAtomChangeV2, ArchiveChangePageV2, ArchiveDossierLinkV2, ArchiveDossierPageV2,
    ArchiveDossierPendingV2, ArchiveDossierReadV2, ArchiveDossierStateV2,
    ArchiveDossierUnavailableV2, ArchiveLinkedPageStateV2, ArchivePublicationOriginV2,
    ArchiveReadScopeV2,
};
use babylon_persistence::{
    ArchiveAtomSubjectKindV1, ArchiveAtomSubjectV1, ArchiveAtomV1, ArchiveAtomValueV1,
    ArchiveCitationV1, ArchiveEvidenceClassV1, ArchivePageRefV1, ArchiveSignalV1,
    ArchiveSubjectKindV1, CampaignId,
};
use bevy::asset::AssetPlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::ButtonState;
use bevy::picking::backend::HitData;
use bevy::picking::events::{Click, Pointer};
use bevy::picking::pointer::{Location, PointerButton, PointerId};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::ffi::OsString;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;
use uuid::Uuid;

// ---- app composition (the production wiring, headless) ----

/// Serializes this binary's `BABYLON_READER_DSN` mutations. Rust test
/// threads share one process, so removing the variable without a lock lets
/// a parallel test's fetch task observe the wrong environment.
static READER_DSN_LOCK: Mutex<()> = Mutex::new(());

/// Holds [`READER_DSN_LOCK`] and restores the ambient `BABYLON_READER_DSN`
/// on drop. The fixture returns this guard before the App so callers bind it
/// first: the App and its fetch tasks drop before the guard restores the env.
/// Keeping the guard outside Bevy also releases it when an assertion unwinds.
struct ReaderDsnGuard {
    _lock: MutexGuard<'static, ()>,
    prior: Option<OsString>,
}

impl ReaderDsnGuard {
    fn lock_and_remove() -> Self {
        let lock = READER_DSN_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let prior = std::env::var_os("BABYLON_READER_DSN");
        std::env::remove_var("BABYLON_READER_DSN");
        Self { _lock: lock, prior }
    }
}

impl Drop for ReaderDsnGuard {
    fn drop(&mut self) {
        match self.prior.take() {
            Some(value) => std::env::set_var("BABYLON_READER_DSN", value),
            None => std::env::remove_var("BABYLON_READER_DSN"),
        }
    }
}

/// Builds the legacy conformance app. `SelectedStory(counties())` mirrors
/// `tests/projection.rs::new_app`; the durable observer uses its own fixture.
fn new_app() -> (ReaderDsnGuard, App) {
    // Determinism contract: this harness never has an Archive reader. A
    // developer's shell may export `BABYLON_READER_DSN` for the live foci;
    // left set, the fetch tasks this file spawns would race a real Postgres.
    // The guard serializes the removal against parallel tests and restores
    // the ambient value only when the App (and its fetch tasks) drops.
    let dsn_guard = ReaderDsnGuard::lock_and_remove();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    app.add_plugins(DossierCardPlugin);
    app.insert_resource(DossierCampaignId(CampaignId::from_uuid(Uuid::nil())));
    app.insert_resource(story::SelectedStory(story::counties()));
    // I4 (tests/projection.rs): pin zero injected sim time before the first
    // update so `RunState.running`'s wall-clock batch can never advance the
    // engine mid-assertion.
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app.update(); // Startup
    (dsn_guard, app)
}

/// The observer uses the shared shell's real layout, without a transport or
/// the legacy in-process tick engine. A headless window supplies its size.
fn new_observer_app(window_size: (u32, u32)) -> (ReaderDsnGuard, App) {
    use babylon_client::observer::ObserverSession;
    use babylon_client::observer_io::ObserverSet;
    use babylon_client::observer_ui::{ObserverShellPlugin, ObserverUiState};
    use babylon_client::ui::dossier_card::DossierCampaignId;

    let dsn_guard = ReaderDsnGuard::lock_and_remove();
    let campaign = CampaignId::from_uuid(Uuid::nil());
    let mut session = ObserverSession::new(campaign);
    session.ready(12, Some("0c".repeat(32)));
    let mut app = App::new();
    // This structural fixture runs no text renderer. Distinct reserved handles
    // satisfy the shell's explicit font roles without a second asset loader.
    let font_handles = Assets::<Font>::default();
    app.insert_resource(babylon_client::visual_assets::ObserverFonts {
        body: font_handles.reserve_handle(),
        display: font_handles.reserve_handle(),
    });
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .insert_resource(session)
        .insert_resource(DossierCampaignId(campaign))
        .insert_resource(ObserverUiState {
            menu_open: false,
            splash_visible: false,
            ..default()
        })
        .init_resource::<UiScale>()
        .init_resource::<bevy::picking::hover::HoverMap>()
        .init_resource::<babylon_client::observer_audio::ObserverAudioSettings>()
        .init_resource::<babylon_client::production::PrimaryView>()
        .init_resource::<babylon_client::production::ProductionNavigation>()
        .add_message::<babylon_client::production::ProductionCommand>()
        .add_plugins((
            babylon_client::map::MapPlugin,
            ObserverShellPlugin,
            babylon_client::observer_history::ObserverHistoryPlugin,
            DossierCardPlugin,
        ))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
        .configure_sets(
            Update,
            (
                ObserverSet::Input,
                ObserverSet::Receive,
                ObserverSet::Install,
                ObserverSet::Paint,
            )
                .chain(),
        );
    app.world_mut().spawn((
        Window {
            resolution: bevy::window::WindowResolution::from(window_size)
                .with_scale_factor_override(1.0),
            ..default()
        },
        bevy::window::PrimaryWindow,
    ));
    install_observer_frame(&mut app);
    app.update();
    (dsn_guard, app)
}

/// Presses `key` through the REAL `KeyboardInput` message pipeline — the
/// house pattern (`tests/projection.rs::press_key_via_real_event`): with
/// `MapPlugin`'s `InputPlugin` present, a direct `ButtonInput::press()` is
/// cleared before any `Update` system observes it.
fn press_key_via_real_event(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<Messages<KeyboardInput>>()
        .write(KeyboardInput {
            key_code: key,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
}

fn release_key(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(key);
}

// ---- fixtures ----

/// One Archive atom on county 01001 (atlas index 0), the dossier.rs test
/// fixture shape.
fn atom(signal_key: &str, value: &str, valid_tick: u64) -> ArchiveAtomV1 {
    ArchiveAtomV1::try_new(
        CampaignId::from_uuid(Uuid::nil()),
        ArchiveAtomSubjectV1::try_new(ArchiveAtomSubjectKindV1::County, "01001".to_owned())
            .expect("subject admits"),
        signal_key.to_owned(),
        "employment".to_owned(),
        ArchiveEvidenceClassV1::Observed,
        &ArchiveAtomValueV1::Text(value.to_owned()),
        ArchiveCitationV1::try_new("src".to_owned(), "loc".to_owned()).expect("citation admits"),
        valid_tick,
    )
    .expect("atom admits")
}

/// Requested/durable week 12 with retained week-11 content still awaiting
/// processing. Links preserve three disclosure states and one exact change.
fn fixture_projection() -> ArchiveDossierReadV2 {
    let campaign = CampaignId::from_uuid(Uuid::nil());
    let source = ArchiveReadScopeV2::committed(campaign, 11, [11; 32]).unwrap();
    let signal = ArchiveSignalV1::try_new(
        "employment".into(),
        "Employment".into(),
        "728576 jobs".into(),
        ArchiveCitationV1::try_new("src".into(), "loc".into()).unwrap(),
    )
    .unwrap();
    let page = ArchiveDossierPageV2 {
        revision_id: [7; 32],
        effective_tick: 11,
        origin: ArchivePublicationOriginV2::Materialized,
        content_source: source,
        title: "Autauga County".into(),
        question: DOSSIER_DECISION_QUESTION.into(),
        signals: vec![signal],
        markdown: "Retained fixture narrative".into(),
        content_sha256: [8; 32],
        citations: vec![ArchiveCitationV1::try_new("src".into(), "loc".into()).unwrap()],
        atoms: vec![
            atom("subject", "Autauga County", 1),
            atom("employment", "728576 jobs", 11),
        ],
        links: [
            (
                "0101076",
                Some("Prattville"),
                ArchiveLinkedPageStateV2::KnownReady,
            ),
            (
                "0101128",
                Some("Millbrook"),
                ArchiveLinkedPageStateV2::KnownPending,
            ),
            ("0199999", None, ArchiveLinkedPageStateV2::Unknown),
        ]
        .into_iter()
        .map(|(id, label, target_state)| ArchiveDossierLinkV2 {
            target: ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, id.into()).unwrap(),
            retained_label: label.map(str::to_owned),
            target_state,
        })
        .collect(),
        changes: ArchiveChangePageV2 {
            coverage_from_tick: 1,
            changes: vec![ArchiveAtomChangeV2 {
                publication_tick: 11,
                signal_key: "employment".into(),
                before: Some(atom("employment", "710000 jobs", 10)),
                after: Some(atom("employment", "728576 jobs", 11)),
            }],
            next_cursor: None,
        },
    };
    ArchiveDossierReadV2 {
        scope: ArchiveReadScopeV2::committed(campaign, 12, [12; 32]).unwrap(),
        subject: ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, "01001".into()).unwrap(),
        durable_tick: 12,
        processed_tick: 11,
        history_floor_tick: 0,
        state: ArchiveDossierStateV2::Pending {
            page: Some(page),
            reason: ArchiveDossierPendingV2::ReceiptProcessing,
        },
    }
}

fn install_observer_frame(app: &mut App) {
    use babylon_client::observer::ObserverSession;
    use babylon_client::observer_ui::ObserverFrame;
    use babylon_persistence::{ObserverEconomySnapshotV1, ObserverVisibilityV1};
    let session = app.world().resource::<ObserverSession>();
    let hash = format!("{:02x}", session.viewed_tick).repeat(32);
    let frame = ObserverEconomySnapshotV1 {
        campaign_id: session.campaign.as_uuid().to_string(),
        resolve_tick: session.viewed_tick,
        foundation_digest: String::new(),
        nominal_world_hash: None,
        tick_content_hash: Some(hash),
        envelope_digest: None,
        visibility: ObserverVisibilityV1::FullObserver,
        counties: Vec::new(),
        production: None,
    };
    app.insert_resource(ObserverFrame(Some(frame)));
    let mut session = app.world_mut().resource_mut::<ObserverSession>();
    let context = session.context();
    assert!(session.installed(&context));
}

fn installed_fixture(app: &App, read: ArchiveDossierReadV2) -> InstalledDossier {
    let selected = app.world().resource::<SelectedCounty>().0.unwrap();
    let county = app
        .world()
        .resource::<CountyAtlas>()
        .county(selected)
        .unwrap()
        .fips;
    InstalledDossier {
        scope: DossierRequestScope {
            campaign: app.world().resource::<DossierCampaignId>().0,
            county_geoid: county.into(),
            refresh_generation: app.world().resource::<DossierRefresh>().0,
            observer: app
                .world()
                .get_resource::<babylon_client::observer::ObserverSession>()
                .map(babylon_client::observer::ObserverSession::context),
            read_scope: read.scope.clone(),
            subject: read.subject.clone(),
        },
        read,
    }
}

// ---- tree readers ----

fn card_root(app: &mut App) -> Entity {
    let world = app.world_mut();
    let mut query = world.query_filtered::<Entity, With<DossierCardRoot>>();
    query.single(world).expect("exactly one dossier card root")
}

fn zone_entity(app: &mut App, zone: DossierZone) -> Entity {
    let world = app.world_mut();
    let mut query = world.query::<(Entity, &DossierZone)>();
    query
        .iter(world)
        .find_map(|(entity, found)| (*found == zone).then_some(entity))
        .unwrap_or_else(|| panic!("zone {zone:?} must exist"))
}

/// Concatenated `Text`/`TextSpan` content of one subtree, one component per
/// line — every `contains` assertion below is against text a player would
/// actually read.
fn subtree_text(app: &mut App, entity: Entity) -> String {
    fn collect(world: &World, entity: Entity, out: &mut String) {
        let piece = world
            .get::<Text>(entity)
            .map(|text| text.0.as_str())
            .or_else(|| world.get::<TextSpan>(entity).map(|span| span.0.as_str()));
        if let Some(piece) = piece {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(piece);
        }
        if let Some(children) = world.get::<Children>(entity) {
            for child in children.iter() {
                collect(world, child, out);
            }
        }
    }
    let world = app.world_mut();
    let mut out = String::new();
    collect(world, entity, &mut out);
    out
}

fn zone_text(app: &mut App, zone: DossierZone) -> String {
    let entity = zone_entity(app, zone);
    subtree_text(app, entity)
}

fn subtree_text_colors(app: &mut App, entity: Entity) -> Vec<Color> {
    fn collect(world: &World, entity: Entity, out: &mut Vec<Color>) {
        if let Some(color) = world.get::<TextColor>(entity) {
            out.push(color.0);
        }
        if let Some(children) = world.get::<Children>(entity) {
            for child in children.iter() {
                collect(world, child, out);
            }
        }
    }
    let world = app.world_mut();
    let mut out = Vec::new();
    collect(world, entity, &mut out);
    out
}

/// Finds the chip NODE whose child `Text` equals `wanted` under `root` —
/// chips are identified by their rendered label (exactly what a player
/// reads), but the node is what carries `PlaceChipNode`, the observers, and
/// the state border (the text child has its own default-required
/// `BorderColor(NONE)`, so tests must address the node, not the text).
fn chip_node_with_text(app: &mut App, root: Entity, wanted: &str) -> Entity {
    let text_entity = find_entity_with_text(app, root, wanted)
        .unwrap_or_else(|| panic!("a chip rendered {wanted:?}"));
    app.world()
        .get::<ChildOf>(text_entity)
        .unwrap_or_else(|| panic!("a chip text lives under a chip node"))
        .parent()
}
/// Finds the first entity whose own `Text`/`TextSpan` equals `wanted` under
/// `root`.
fn find_entity_with_text(app: &mut App, root: Entity, wanted: &str) -> Option<Entity> {
    fn walk(world: &World, entity: Entity, wanted: &str) -> Option<Entity> {
        let matches = world
            .get::<Text>(entity)
            .is_some_and(|text| text.0 == wanted)
            || world
                .get::<TextSpan>(entity)
                .is_some_and(|span| span.0 == wanted);
        if matches {
            return Some(entity);
        }
        world
            .get::<Children>(entity)?
            .iter()
            .find_map(|child| walk(world, child, wanted))
    }
    let world = app.world_mut();
    walk(world, root, wanted)
}

// ---- card driving ----

/// Drives a REAL county selection (spawning the fetch task through
/// `drive_dossier_fetch`), then seizes the card with the deterministic
/// fixture: `Idle` makes `collect_dossier_fetch` early-return forever after
/// (its destructuring match needs `InFlight`), so the orphaned task can
/// never overwrite the seeded projection, and one update repaints every
/// zone from it.
fn seize_card(app: &mut App, projection: ArchiveDossierReadV2) {
    let county = app
        .world()
        .resource::<CountyAtlas>()
        .index_of_fips(projection.subject.id())
        .expect("the fixture county belongs to the committed atlas");
    app.world_mut().resource_mut::<SelectedCounty>().0 = Some(county);
    app.update();
    assert!(
        matches!(
            app.world().resource::<DossierFetchState>(),
            DossierFetchState::InFlight { .. } | DossierFetchState::Failed(_)
        ),
        "a real selection change must start a fetch"
    );
    *app.world_mut().resource_mut::<DossierFetchState>() = DossierFetchState::Idle;
    let installed = installed_fixture(app, projection);
    app.world_mut().resource_mut::<ActiveCountyDossier>().0 = Some(installed);
    *app.world_mut().resource_mut::<DossierPageView>() = DossierPageView::Card;
    app.update(); // repaint renders the fixture
}

/// Clicks one place chip by its rendered label through a REAL
/// `Pointer<Click>` trigger on the chip NODE — the observer path a player's
/// click takes, not a direct message write.
fn click_chip(app: &mut App, label: &str) {
    let places = zone_entity(app, DossierZone::Places);
    let chip = chip_node_with_text(app, places, label);
    click_entity(app, chip);
}

fn click_entity(app: &mut App, chip: Entity) {
    app.world_mut().trigger(Pointer::new(
        PointerId::Mouse,
        Location {
            target: bevy::camera::NormalizedRenderTarget::None {
                width: 0,
                height: 0,
            },
            position: Vec2::ZERO,
        },
        Click {
            button: PointerButton::Primary,
            hit: HitData {
                camera: Entity::PLACEHOLDER,
                depth: 0.0,
                position: None,
                normal: None,
            },
            duration: Duration::ZERO,
        },
        chip,
    ));
    app.update(); // observer writes the request; repaint swaps the page
}

// ---- the proofs ----

/// Structural golden: the card spawns at Startup hidden, declares the
/// `CountyDossier` surface on the root and every zone, carries all seven
/// zones exactly once, and seals its actions footer with the pinned R9
/// chip text.
#[test]
fn card_spawns_hidden_with_every_zone_and_the_surface_declaration() {
    let (_dsn_guard, mut app) = new_app();
    let root = card_root(&mut app);

    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Hidden),
        "no county selected — the card must start hidden"
    );
    assert_eq!(
        app.world().get::<DeclaredSurface>(root),
        Some(&DeclaredSurface::new(SurfaceId::CountyDossier)),
        "the root declares the gameplay-role manifest row (ADR249 R9)"
    );

    for zone in [
        DossierZone::Title,
        DossierZone::Question,
        DossierZone::DualTick,
        DossierZone::Signals,
        DossierZone::Places,
        DossierZone::Chronicle,
        DossierZone::Actions,
    ] {
        let world = app.world_mut();
        let mut query = world.query::<&DossierZone>();
        let count = query.iter(world).filter(|found| **found == zone).count();
        assert_eq!(count, 1, "zone {zone:?} must exist exactly once");
    }

    let actions = zone_entity(&mut app, DossierZone::Actions);
    let footer = subtree_text(&mut app, actions);
    assert_eq!(footer, INVESTIGATE_SEALED_CHIP);
    assert!(
        subtree_text_colors(&mut app, actions).contains(&palette::DIM),
        "the sealed chip renders DIM — visibly unavailable, never hidden"
    );
}

/// A selection change reveals the card and starts exactly one fetch. The
/// first frame paints whichever honest line the task has reached: the
/// in-flight "Reading cited observations..." while the task runs, or the
/// reader-absent line if the pool already failed it — never stale data,
/// never a panic, and the projection stays cleared.
#[test]
fn selection_reveals_the_card_and_paints_an_honest_fetch_line() {
    let (_dsn_guard, mut app) = new_app();
    let root = card_root(&mut app);
    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Hidden)
    );

    app.world_mut().resource_mut::<SelectedCounty>().0 = Some(0);
    app.update();

    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Visible),
        "a selection must reveal the card"
    );
    assert!(
        app.world().resource::<ActiveCountyDossier>().0.is_none(),
        "an in-flight card never shows stale data"
    );
    let state = app.world().resource::<DossierFetchState>();
    let expected = match state {
        DossierFetchState::InFlight { .. } => "Reading cited observations...",
        DossierFetchState::Failed(DossierFetchError::ReaderAbsent(_)) => {
            "Archive reader not configured"
        }
        other => panic!("first frame must be in-flight or reader-absent, got {other:?}"),
    };
    let dual_tick = zone_text(&mut app, DossierZone::DualTick);
    assert!(
        dual_tick.contains(expected),
        "the dual-tick zone must paint {expected:?}, got {dual_tick:?}"
    );
}

/// The full task path against an absent reader: the fetch FAILS to the
/// honest `ReaderAbsent` state (the CI headless reality) and the card
/// renders the reader-absent line — never a panic, projection stays empty.
#[test]
fn reader_absent_is_the_terminal_honest_line_never_a_panic() {
    let (_dsn_guard, mut app) = new_app();
    app.world_mut().resource_mut::<SelectedCounty>().0 = Some(0);
    for _ in 0..100 {
        app.update();
        if !matches!(
            app.world().resource::<DossierFetchState>(),
            DossierFetchState::InFlight { .. }
        ) {
            break;
        }
    }
    assert!(
        matches!(
            app.world().resource::<DossierFetchState>(),
            DossierFetchState::Failed(DossierFetchError::ReaderAbsent(_))
        ),
        "with no DSN the fetch must terminate ReaderAbsent"
    );
    assert!(app.world().resource::<ActiveCountyDossier>().0.is_none());
    let dual_tick = zone_text(&mut app, DossierZone::DualTick);
    assert!(
        dual_tick.contains("Archive reader not configured"),
        "got {dual_tick:?}"
    );
}

/// The typed retained response drives the complete card. Evidence starts collapsed, then the real control exposes exact citations and publication changes.
#[test]
fn seeded_projection_renders_the_whole_card_after_one_update() {
    let (_dsn_guard, mut app) = new_app();
    let (pending, ready) = pending_and_verified_fixture();
    seize_card(&mut app, pending);
    assert_eq!(zone_text(&mut app, DossierZone::Title), "Autauga County");
    assert_eq!(
        zone_text(&mut app, DossierZone::Question),
        DOSSIER_DECISION_QUESTION
    );
    let status = zone_text(&mut app, DossierZone::DualTick);
    assert!(
        status.contains("Viewing week 12 · durable week 12"),
        "{status}"
    );
    assert!(
        status.contains("Archive is still processing this observation"),
        "{status}"
    );
    assert!(
        status.contains("Content last published at week 11"),
        "{status}"
    );
    assert!(
        !status.contains("verified through"),
        "pending is not verified"
    );
    let zone = zone_entity(&mut app, DossierZone::DualTick);
    assert!(subtree_text_colors(&mut app, zone).contains(&palette::CRIMSON));
    let signals = zone_text(&mut app, DossierZone::Signals);
    assert!(
        signals.contains("Employment: ") && signals.contains("728576 jobs"),
        "{signals}"
    );
    assert!(!signals.contains("subject"));
    assert!(!signals.contains("Source:"), "citations start collapsed");
    assert_eq!(zone_text(&mut app, DossierZone::Chronicle), "");
    let actions = zone_entity(&mut app, DossierZone::Actions);
    let evidence = chip_node_with_text(&mut app, actions, "Evidence and changes");
    click_entity(&mut app, evidence);
    let signals = zone_text(&mut app, DossierZone::Signals);
    for exact in ["Source: src", "Locator: loc", "Field: employment"] {
        assert!(signals.contains(exact), "{signals}");
    }
    let pending_changes = zone_text(&mut app, DossierZone::Chronicle);
    assert!(pending_changes.contains("Changes await Archive completion."));
    assert!(!pending_changes.contains("coverage starts"));
    assert!(!pending_changes.contains("Published week 11"));
    assert!(!pending_changes.to_lowercase().contains("no changes"));

    let installed = installed_fixture(&app, ready);
    app.world_mut().resource_mut::<ActiveCountyDossier>().0 = Some(installed);
    app.update();
    let status = zone_text(&mut app, DossierZone::DualTick);
    assert!(status.contains("verified through 12"), "{status}");
    assert!(
        status.contains("Content last published at week 11"),
        "{status}"
    );
    let changes = zone_text(&mut app, DossierZone::Chronicle);
    for exact in [
        "Published week 11",
        "employment",
        "710000 jobs",
        "728576 jobs",
        " → ",
        "coverage starts at week 1",
        "Content observed at week 11; published at week 11.",
    ] {
        assert!(changes.contains(exact), "{exact}: {changes}");
    }
    assert!(
        !changes.contains("verified 10"),
        "publication changes never imply verification"
    );
    assert!(zone_text(&mut app, DossierZone::Actions).ends_with(INVESTIGATE_SEALED_CHIP));
}

fn pending_and_verified_fixture() -> (ArchiveDossierReadV2, ArchiveDossierReadV2) {
    let mut pending = fixture_projection();
    pending.history_floor_tick = 1;
    let mut ready = pending.clone();
    let ArchiveDossierStateV2::Pending {
        page: Some(page), ..
    } = &mut pending.state
    else {
        unreachable!()
    };
    ready.state = ArchiveDossierStateV2::Ready {
        page: page.clone(),
        verified_through_tick: 12,
    };
    ready.processed_tick = 12;
    // The confined reader leaves history unenumerated until this scope is ready.
    page.changes.changes.clear();
    (pending, ready)
}

/// Retained links distinguish ready, pending and unknown without borrowing endpoint titles.
#[test]
fn place_chips_render_granted_pending_and_fog_states() {
    let (_dsn_guard, mut app) = new_app();
    seize_card(&mut app, fixture_projection());
    let places = zone_entity(&mut app, DossierZone::Places);
    for label in [
        "Prattville",
        "Millbrook · pending",
        "unknown place · 0199999",
    ] {
        let chip = chip_node_with_text(&mut app, places, label);
        let colors = subtree_text_colors(&mut app, chip);
        assert!(!colors.is_empty(), "{label} must be readable");
    }
    let text = zone_text(&mut app, DossierZone::Places);
    assert!(
        text.contains("unknown place · 0199999"),
        "only the public kind and ID identify the unknown endpoint"
    );
    assert_eq!(app.world().get::<Children>(places).unwrap().len(), 3);
}

/// A granted link requests its exact scoped subject. Only that response supplies place prose; Back restores county navigation.
#[test]
fn granted_chip_reads_the_exact_linked_subject_and_can_return_to_the_county() {
    let (_dsn_guard, mut app) = new_app();
    seize_card(&mut app, fixture_projection());
    let source_scope = app
        .world()
        .resource::<ActiveCountyDossier>()
        .0
        .as_ref()
        .unwrap()
        .scope
        .clone();
    click_chip(&mut app, "Prattville");
    let DossierPageView::Subject(request) = app.world().resource::<DossierPageView>() else {
        panic!("link must navigate to a scoped subject");
    };
    assert_eq!(request.scope, source_scope);
    assert_eq!(request.id, "0101076");
    assert_eq!(request.label.as_deref(), Some("Prattville"));
    assert!(app.world().resource::<ActiveCountyDossier>().0.is_none());
    assert!(!zone_text(&mut app, DossierZone::Signals).contains("728576 jobs"));
    app.update(); // Starts the real target read; no county result may answer it.
    let mut read = fixture_projection();
    read.subject =
        ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, "0101076".into()).unwrap();
    let ArchiveDossierStateV2::Pending {
        page: Some(page), ..
    } = &mut read.state
    else {
        unreachable!()
    };
    page.title = "Prattville retained page".into();
    page.question = "Which observed place facts are available?".into();
    page.signals.clear();
    page.atoms.clear();
    page.links.clear();
    page.changes.changes.clear();
    let installed = installed_fixture(&app, read);
    *app.world_mut().resource_mut::<DossierFetchState>() = DossierFetchState::Idle;
    app.world_mut().resource_mut::<ActiveCountyDossier>().0 = Some(installed);
    app.update();
    assert_eq!(
        zone_text(&mut app, DossierZone::Title),
        "Prattville retained page"
    );
    assert_eq!(
        zone_text(&mut app, DossierZone::Question),
        "Which observed place facts are available?"
    );
    assert_eq!(zone_text(&mut app, DossierZone::Signals), "");
    let actions = zone_entity(&mut app, DossierZone::Actions);
    let back = chip_node_with_text(&mut app, actions, "Back to county");
    click_entity(&mut app, back);
    assert_eq!(
        app.world().resource::<DossierPageView>(),
        &DossierPageView::Card
    );
    assert!(app.world().resource::<ActiveCountyDossier>().0.is_none());
    assert!(!zone_text(&mut app, DossierZone::Title).contains("retained page"));
}

/// An undisclosed endpoint remains inert and cannot request or invent a named place page.
#[test]
fn undisclosed_link_cannot_navigate_or_synthesize_a_place_page() {
    let (_dsn_guard, mut app) = new_app();
    seize_card(&mut app, fixture_projection());
    let before = app.world().resource::<ActiveCountyDossier>().clone();
    click_chip(&mut app, "unknown place · 0199999");
    assert_eq!(
        app.world().resource::<DossierPageView>(),
        &DossierPageView::Card
    );
    assert_eq!(app.world().resource::<ActiveCountyDossier>(), &before);
    assert_eq!(zone_text(&mut app, DossierZone::Title), "Autauga County");
    assert!(!zone_text(&mut app, DossierZone::Signals).contains("0199999"));
}

/// The N-key restart clears the card THROUGH the selection signal:
/// `restart_on_n_key`'s `SelectedCounty = None` write is what
/// `drive_dossier_fetch` reacts to — the projection, the page view, and
/// the root visibility all return to the resting state in the same frame.
#[test]
fn n_key_restart_clears_the_card_through_the_selection_signal() {
    let (_dsn_guard, mut app) = new_app();
    seize_card(&mut app, fixture_projection());
    assert_eq!(zone_text(&mut app, DossierZone::Title), "Autauga County");

    press_key_via_real_event(&mut app, KeyCode::KeyN);
    app.update();
    release_key(&mut app, KeyCode::KeyN);

    assert_eq!(app.world().resource::<SelectedCounty>().0, None);
    assert!(
        app.world().resource::<ActiveCountyDossier>().0.is_none(),
        "the restart must drop the projection"
    );
    assert_eq!(
        app.world().resource::<DossierPageView>(),
        &DossierPageView::Card,
        "the page view returns to the card (nothing selected)"
    );
    let root = card_root(&mut app);
    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Hidden),
        "with no selection the card renders nothing"
    );
}

/// The sealed player-action footer remains visible while a linked Archive read is pending.
#[test]
fn sealed_actions_footer_survives_every_page_view() {
    let (_dsn_guard, mut app) = new_app();
    seize_card(&mut app, fixture_projection());
    click_chip(&mut app, "Prattville");
    let actions = zone_text(&mut app, DossierZone::Actions);
    assert!(actions.ends_with(INVESTIGATE_SEALED_CHIP));
}

#[test]
fn held_selection_refresh_clears_previous_card_and_starts_a_new_read() {
    let (_dsn_guard, mut app) = new_app();
    seize_card(&mut app, fixture_projection());
    let selected = app.world().resource::<SelectedCounty>().0;
    app.world_mut()
        .resource_mut::<babylon_client::ui::dossier_card::DossierRefresh>()
        .bump();
    app.update();
    assert_eq!(app.world().resource::<SelectedCounty>().0, selected);
    assert!(app.world().resource::<ActiveCountyDossier>().0.is_none());
    assert!(matches!(
        app.world().resource::<DossierFetchState>(),
        DossierFetchState::InFlight { .. } | DossierFetchState::Failed(_)
    ));
}

#[test]
fn obsolete_campaign_generation_and_county_results_cannot_install_or_report_errors() {
    use babylon_client::ui::dossier_card::{DossierCampaignId, DossierRefresh};
    use bevy::tasks::AsyncComputeTaskPool;

    let (_dsn_guard, mut app) = new_app();
    seize_card(&mut app, fixture_projection());
    let campaign = app.world().resource::<DossierCampaignId>().0;
    let generation = app.world().resource::<DossierRefresh>().0;
    for (requested_campaign, requested_generation, fips, error) in [
        (
            CampaignId::from_uuid(Uuid::from_u128(1)),
            generation,
            "01001",
            false,
        ),
        (campaign, generation + 1, "01001", false),
        (campaign, generation, "26163", false),
        (campaign, generation + 1, "01001", true),
    ] {
        let result = if error {
            Err(DossierFetchError::ReadFailed("obsolete failure".to_owned()))
        } else {
            Ok(installed_fixture(&app, fixture_projection()))
        };
        let task = AsyncComputeTaskPool::get().spawn(async move { result });
        *app.world_mut().resource_mut::<DossierFetchState>() = DossierFetchState::InFlight {
            scope: DossierRequestScope {
                campaign: requested_campaign,
                county_geoid: fips.into(),
                refresh_generation: requested_generation,
                observer: None,
                read_scope: ArchiveReadScopeV2::foundation(requested_campaign),
                subject: ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, fips.into())
                    .unwrap(),
            },
            task,
        };
        app.update();
        assert!(matches!(
            app.world().resource::<DossierFetchState>(),
            DossierFetchState::Idle
        ));
        assert!(app.world().resource::<ActiveCountyDossier>().0.is_none());
        assert!(!zone_text(&mut app, DossierZone::DualTick).contains("obsolete failure"));
    }
}

#[test]
fn unchanged_card_and_unfinished_task_preserve_the_rendered_subtree() {
    use babylon_client::ui::dossier_card::{DossierCampaignId, DossierRefresh};
    use bevy::tasks::AsyncComputeTaskPool;

    let (_dsn_guard, mut app) = new_app();
    seize_card(&mut app, fixture_projection());
    let title = zone_entity(&mut app, DossierZone::Title);
    let initial = app
        .world()
        .get::<Children>(title)
        .expect("painted title")
        .iter()
        .collect::<Vec<_>>();
    app.update();
    assert_eq!(
        app.world()
            .get::<Children>(title)
            .expect("title remains")
            .iter()
            .collect::<Vec<_>>(),
        initial
    );

    let campaign = app.world().resource::<DossierCampaignId>().0;
    let generation = app.world().resource::<DossierRefresh>().0;
    let task = AsyncComputeTaskPool::get().spawn(std::future::pending());
    *app.world_mut().resource_mut::<DossierFetchState>() = DossierFetchState::InFlight {
        scope: DossierRequestScope {
            campaign,
            county_geoid: "01001".into(),
            refresh_generation: generation,
            observer: None,
            read_scope: ArchiveReadScopeV2::foundation(campaign),
            subject: ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, "01001".into())
                .unwrap(),
        },
        task,
    };
    app.update();
    let pending = app
        .world()
        .get::<Children>(title)
        .expect("pending title")
        .iter()
        .collect::<Vec<_>>();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(
        app.world()
            .get::<Children>(title)
            .expect("pending title remains")
            .iter()
            .collect::<Vec<_>>(),
        pending
    );
}

fn root_containing_text(app: &mut App, text: &str) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, (With<Node>, Without<ChildOf>)>();
    let roots = query.iter(app.world()).collect::<Vec<_>>();
    let matching = roots
        .into_iter()
        .filter(|root| find_entity_with_text(app, *root, text).is_some())
        .collect::<Vec<_>>();
    assert_eq!(matching.len(), 1, "one panel must contain {text:?}");
    matching[0]
}

fn laid_out_rect(app: &App, entity: Entity) -> Rect {
    let node = app.world().get::<Node>(entity).expect("panel Node");
    let (Val::Px(left), Val::Px(top), Val::Px(width), Val::Px(height)) =
        (node.left, node.top, node.width, node.height)
    else {
        panic!("the shared observer layout must assign logical pixel bounds")
    };
    assert!(width > 0.0 && height > 0.0, "panel must have positive area");
    Rect::new(left, top, left + width, top + height)
}

fn assert_observer_archive_geometry(app: &mut App, root: Entity) {
    use babylon_client::observer_ui::ObserverViewport;

    let mut windows = app
        .world_mut()
        .query_filtered::<&Window, With<bevy::window::PrimaryWindow>>();
    let window = windows.single(app.world()).expect("primary window");
    let size = Vec2::new(window.width(), window.height());
    let scale = app.world().resource::<UiScale>().0;
    let world = app
        .world()
        .resource::<ObserverViewport>()
        .0
        .expect("the real shell must calculate a world viewport");
    let world = Rect::from_corners(world.min / scale, world.max / scale);
    let context_entity = root_containing_text(app, "Read the cited Archive [I]");
    let footer = root_containing_text(app, "Return Live");
    let archive = laid_out_rect(app, root);
    let context = laid_out_rect(app, context_entity);
    let footer = laid_out_rect(app, footer);
    assert_eq!(
        archive, context,
        "Archive replaces the selected-subject rail"
    );
    let ui = app
        .world()
        .resource::<babylon_client::observer_ui::ObserverUiState>();
    assert_eq!(
        *app.world()
            .get::<Visibility>(context_entity)
            .expect("context visibility"),
        if ui.archive_open || ui.history_open {
            Visibility::Hidden
        } else {
            Visibility::Visible
        },
        "the shared rail must expose only its admitted panel",
    );
    assert!(world.width() > 0.0 && world.height() > 0.0);
    assert!(
        world.max.y < footer.min.y,
        "world and transport footer stay separate"
    );
    for rect in [world, footer] {
        assert!(rect.min.cmpge(Vec2::ZERO).all());
        assert!(rect.max.cmple(size / scale).all());
        assert!(
            rect.max.x < archive.min.x,
            "the subject rail must not cover the world or its footer"
        );
    }
    assert!(archive.min.cmpge(Vec2::ZERO).all());
    assert!(archive.max.cmple(size / scale).all());
    let node = app.world().get::<Node>(root).expect("Archive Node");
    assert_eq!((node.right, node.bottom), (Val::Auto, Val::Auto));
    assert_eq!((node.min_width, node.max_width), (Val::Auto, Val::Auto));
    assert_eq!(node.overflow.y, OverflowAxis::Scroll);
}

fn assert_historical_observer_archive(app: &mut App, root: Entity) {
    use babylon_client::observer::ObserverSession;
    use babylon_client::observer_ui::ObserverUiState;
    app.world_mut()
        .resource_mut::<ObserverSession>()
        .inspect_tick(10);
    assert_eq!(app.world().resource::<ObserverSession>().viewed_tick, 10);
    app.update();
    assert!(app.world().resource::<ActiveCountyDossier>().0.is_none());
    assert!(matches!(
        app.world().resource::<DossierFetchState>(),
        DossierFetchState::WaitingForObservation
    ));
    assert!(zone_text(app, DossierZone::DualTick)
        .contains("Waiting for the selected week's committed observation"));
    assert!(!zone_text(app, DossierZone::Signals).contains("728576 jobs"));
    install_observer_frame(app);
    app.update();
    assert!(matches!(
        app.world().resource::<DossierFetchState>(),
        DossierFetchState::InFlight { .. } | DossierFetchState::Failed(_)
    ));
    let mut read = fixture_projection();
    read.scope = ArchiveReadScopeV2::committed(read.scope.campaign_id(), 10, [10; 32]).unwrap();
    read.history_floor_tick = 11;
    read.state =
        ArchiveDossierStateV2::Unavailable(ArchiveDossierUnavailableV2::HistoryNotRetained);
    let installed = installed_fixture(app, read);
    app.world_mut().resource_mut::<ActiveCountyDossier>().0 = Some(installed);
    *app.world_mut().resource_mut::<DossierFetchState>() = DossierFetchState::Idle;
    app.update();
    let text = zone_text(app, DossierZone::DualTick);
    assert!(
        text.contains("Viewing week 10") && text.contains("predates retained Archive history"),
        "{text}"
    );
    assert!(!zone_text(app, DossierZone::Signals).contains("728576 jobs"));
    assert_eq!(zone_text(app, DossierZone::Title), "Autauga County, AL");
    assert_eq!(zone_text(app, DossierZone::Actions), "READ-ONLY ARCHIVE");
    assert_eq!(
        zone_text(app, DossierZone::Question),
        "Which cited observations are available at this week?"
    );
    let mut read = fixture_projection();
    read.scope = ArchiveReadScopeV2::committed(read.scope.campaign_id(), 10, [10; 32]).unwrap();
    let ArchiveDossierStateV2::Pending {
        page: Some(mut page),
        ..
    } = read.state
    else {
        unreachable!()
    };
    page.content_source = read.scope.clone();
    page.effective_tick = 10;
    page.atoms = vec![atom("employment", "710000 jobs", 10)];
    page.signals = vec![ArchiveSignalV1::try_new(
        "employment".into(),
        "Employment".into(),
        "710000 jobs".into(),
        ArchiveCitationV1::try_new("src".into(), "loc".into()).unwrap(),
    )
    .unwrap()];
    page.changes.changes.clear();
    read.state = ArchiveDossierStateV2::Ready {
        page,
        verified_through_tick: 10,
    };
    let installed = installed_fixture(app, read);
    app.world_mut().resource_mut::<ActiveCountyDossier>().0 = Some(installed);
    app.update();
    let status = zone_text(app, DossierZone::DualTick);
    assert!(
        status.contains("Viewing week 10")
            && status.contains("durable week 12")
            && status.contains("verified through 10"),
        "{status}"
    );
    assert!(zone_text(app, DossierZone::Signals).contains("710000 jobs"));
    assert!(!zone_text(app, DossierZone::Signals).contains("728576 jobs"));
    for modal in [0, 1, 2] {
        let mut ui = app.world_mut().resource_mut::<ObserverUiState>();
        ui.menu_open = modal == 0;
        ui.splash_visible = modal == 1;
        ui.comparison_open = modal == 2;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(root).unwrap(),
            Visibility::Hidden
        );
    }
}

#[test]
fn observer_archive_layout_and_exact_historical_unavailability_are_explicit() {
    use babylon_client::observer_ui::ObserverUiState;

    for size in [(1366, 768), (1920, 1080)] {
        let (_dsn_guard, mut app) = new_observer_app(size);
        seize_card(&mut app, fixture_projection());
        let root = card_root(&mut app);
        let context = root_containing_text(&mut app, "Read the cited Archive [I]");
        assert_eq!(
            *app.world()
                .get::<Visibility>(root)
                .expect("Archive visibility"),
            Visibility::Hidden
        );
        assert_eq!(
            *app.world()
                .get::<Visibility>(context)
                .expect("context visibility"),
            Visibility::Visible
        );
        assert_observer_archive_geometry(&mut app, root);
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .history_open = true;
        app.update();
        let log = root_containing_text(&mut app, "COMMITTED DEVELOPMENTS");
        assert_eq!(laid_out_rect(&app, log), laid_out_rect(&app, root));
        assert_eq!(
            *app.world().get::<Visibility>(log).unwrap(),
            Visibility::Visible
        );
        assert_observer_archive_geometry(&mut app, root);
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .history_open = false;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(log).unwrap(),
            Visibility::Hidden
        );
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .archive_open = true;
        app.update();
        assert_eq!(
            *app.world()
                .get::<Visibility>(root)
                .expect("Archive visibility"),
            Visibility::Visible
        );
        assert_eq!(
            *app.world().get::<Visibility>(log).expect("log visibility"),
            Visibility::Hidden
        );
        assert_observer_archive_geometry(&mut app, root);
        assert!(zone_text(&mut app, DossierZone::DualTick)
            .contains("Content last published at week 11"));
        assert_historical_observer_archive(&mut app, root);
    }
}
