//! One canonical, scoped Archive observation composed into a native card.
//!
//! Async reads bind the exact installed week, commit identity, capability and
//! geographic selection. The same typed read feeds headless JSONL. Retained
//! publications stay explicitly pending until the reader verifies them; no
//! current-page search or global progress counter fills missing historical data.
//! Keyboard and pointer controls share admission and are checked again on apply.

use babylon_persistence::{
    ArchivePageRefV1, ArchiveSubjectKindV1, CampaignId, SemanticArchiveReaderV1,
};
use bevy::ecs::system::SystemParam;
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};

use crate::atlas::CountyAtlas;
use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::dossier::{availability_label, observation_scope, retained_page, verified_tick};
use crate::map::SelectedCounty;
use crate::observer::{ObservationContext, ObserverSession};
use crate::observer_focus::{ObserverFocusSystems, ObserverFocusTarget, ObserverKeyboardActivate};
use crate::observer_ui::{ObserverFeedback, ObserverFontRole, ObserverFrame, ObserverUiState};
use crate::palette;
use crate::ui::dossier_compose::{
    chronicle_header, chronicle_row_segments, retained_signal_segments, DossierSegment,
    DossierTone, DOSSIER_DECISION_QUESTION, INVESTIGATE_SEALED_CHIP,
};
use babylon_persistence::archive_revision::{
    ArchiveChangeCursorV2, ArchiveDossierBoundsV2, ArchiveDossierLinkV2, ArchiveDossierReadV2,
    ArchiveLinkedPageStateV2, ArchiveReadScopeV2,
};

/// `babylon-runtime`'s default campaign identity (`babylon-runtime.rs`), the
/// fallback when `BABYLON_CAMPAIGN_ID` is unset — the client reads the same
/// campaign the runtime writes by default.
const DEFAULT_CAMPAIGN_UUID: u128 = 0x2810_0000_0000_0000_0000_0000_0000_0001;
const CAMPAIGN_ENV: &str = "BABYLON_CAMPAIGN_ID";

/// The canonical campaign identity the dossier card reads under:
/// `BABYLON_CAMPAIGN_ID` when it is a canonical UUID, else the runtime's
/// pinned default campaign.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DossierCampaignId(pub CampaignId);

impl Default for DossierCampaignId {
    fn default() -> Self {
        let from_env = std::env::var(CAMPAIGN_ENV).ok().and_then(|raw| {
            if let Ok(uuid) = uuid::Uuid::parse_str(&raw) {
                Some(uuid)
            } else {
                log::warn!("{CAMPAIGN_ENV} is not a canonical UUID; using the default campaign");
                None
            }
        });
        Self(CampaignId::from_uuid(from_env.unwrap_or_else(|| {
            uuid::Uuid::from_u128(DEFAULT_CAMPAIGN_UUID)
        })))
    }
}

/// Exact request identity retained beside the single canonical Archive read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DossierRequestScope {
    pub campaign: CampaignId,
    pub county_geoid: String,
    pub refresh_generation: u64,
    pub observer: Option<ObservationContext>,
    pub read_scope: ArchiveReadScopeV2,
    pub subject: ArchivePageRefV1,
}

/// A retained, disclosed link, captured at the page that offered it.
#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct SubjectPageRequest {
    pub scope: DossierRequestScope,
    pub kind: String,
    pub id: String,
    pub label: Option<String>,
}

/// Geographic selection remains independent of the inspected Archive subject.
#[derive(Resource, Clone, Debug, PartialEq, Eq, Default)]
pub enum DossierPageView {
    #[default]
    Card,
    Subject(Box<SubjectPageRequest>),
}

/// Request identity and its one canonical response; no duplicate page projection.
#[derive(Clone, Debug, PartialEq)]
pub struct InstalledDossier {
    pub scope: DossierRequestScope,
    pub read: ArchiveDossierReadV2,
}

/// The installed observation. Consumers must admit its captured scope before reading.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct ActiveCountyDossier(pub Option<InstalledDossier>);

impl ActiveCountyDossier {
    /// Admit one exact observed week, capability, selection and refresh generation.
    #[must_use]
    pub fn for_observer(
        &self,
        session: &ObserverSession,
        frame: &ObserverFrame,
        refresh_generation: u64,
        selected_county_geoid: &str,
    ) -> Option<&ArchiveDossierReadV2> {
        self.0
            .as_ref()?
            .for_observer(session, frame, refresh_generation, selected_county_geoid)
    }
}

impl InstalledDossier {
    fn for_observer(
        &self,
        session: &ObserverSession,
        frame: &ObserverFrame,
        refresh_generation: u64,
        selected_county_geoid: &str,
    ) -> Option<&ArchiveDossierReadV2> {
        let installed = self;
        let snapshot = frame.for_session(session)?;
        let read_scope = observation_scope(
            session.campaign,
            snapshot.resolve_tick,
            snapshot.tick_content_hash.as_deref(),
        )
        .ok()?;
        (installed.scope.campaign == session.campaign
            && installed.scope.county_geoid == selected_county_geoid
            && installed.scope.refresh_generation == refresh_generation
            && installed
                .scope
                .observer
                .as_ref()
                .is_some_and(|scope| session.accepts(scope))
            && installed.scope.read_scope == read_scope
            && installed.read.scope == read_scope
            && installed.read.subject == installed.scope.subject)
            .then_some(&installed.read)
    }
}

/// Acknowledged runtime/Archive progress refreshes a held selection.
#[derive(Resource, Debug, Default)]
pub struct DossierRefresh(pub u64);
impl DossierRefresh {
    /// Advances the held-selection refresh generation.
    ///
    /// # Panics
    /// Panics if the refresh generation exhausts `u64`.
    pub fn bump(&mut self) {
        self.0 = self
            .0
            .checked_add(1)
            .expect("dossier refresh generation exhausted");
    }
}

/// Why one dossier read failed; database errors never become content absence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DossierFetchError {
    ReaderAbsent(String),
    ReadFailed(String),
}
impl std::fmt::Display for DossierFetchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReaderAbsent(detail) => {
                write!(formatter, "Archive reader not configured ({detail})")
            }
            Self::ReadFailed(detail) => write!(formatter, "Archive fetch failed — {detail}"),
        }
    }
}

/// One read task, explicit waiting, or a read failure.
#[derive(Resource, Debug, Default)]
pub enum DossierFetchState {
    #[default]
    Idle,
    WaitingForObservation,
    InFlight {
        scope: DossierRequestScope,
        task: Task<Result<InstalledDossier, DossierFetchError>>,
    },
    Failed(DossierFetchError),
}

#[derive(Resource, Default, Debug, PartialEq, Eq)]
struct DossierPresentation {
    cursor: Option<ArchiveChangeCursorV2>,
    details_open: bool,
}

fn fetch_dossier(
    mut scope: DossierRequestScope,
    cursor: Option<ArchiveChangeCursorV2>,
) -> Result<InstalledDossier, DossierFetchError> {
    let read_error = |error: babylon_persistence::SemanticArchiveReaderErrorV1| {
        DossierFetchError::ReadFailed(error.to_string())
    };
    let reader = SemanticArchiveReaderV1::from_env()
        .map_err(|error| DossierFetchError::ReaderAbsent(error.to_string()))?;
    if scope.observer.is_none() {
        // The standalone conformance card pins one marker before its canonical read.
        scope.read_scope = crate::dossier::pinned_scope(&reader, scope.campaign)
            .map_err(DossierFetchError::ReadFailed)?;
    }
    let bounds = ArchiveDossierBoundsV2::try_new(32, cursor)
        .map_err(|error| DossierFetchError::ReadFailed(error.to_string()))?;
    let read = reader
        .dossier_as_of(&scope.read_scope, &scope.subject, &bounds)
        .map_err(read_error)?;
    if read.scope != scope.read_scope || read.subject != scope.subject {
        return Err(DossierFetchError::ReadFailed(
            "Archive returned a different observation".into(),
        ));
    }
    Ok(InstalledDossier { scope, read })
}

// ---- Card chrome ----

/// Which zone of the card one entity renders. One marker per zone so the
/// repaint finds its targets and headless tests can assert the whole zone
/// tree exists and renders the expected text (the same pub-marker pattern
/// `ui::countdown::CountdownPaneText` and friends use).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DossierZone {
    /// The county title line.
    Title,
    /// The ADR249 decision question, one per display.
    Question,
    /// The durable/verified dual-tick honesty header (R9).
    DualTick,
    /// The signal atom rows.
    Signals,
    /// The place-link chips.
    Places,
    /// The supersession feed.
    Chronicle,
    /// The actions footer (the sealed Investigate chip).
    Actions,
}

/// The card root marker.
#[derive(Component, Debug)]
pub struct DossierCardRoot;

/// A place-link chip entity; also carries the base border color so the
/// hover-out observer can restore it without re-deriving chip state.
#[derive(Component, Clone, Debug)]
struct PlaceChipNode {
    control: DossierControl,
    base_border: Color,
}

const CARD_SURFACE: SurfaceId = SurfaceId::CountyDossier;

fn tone_color(tone: DossierTone) -> Color {
    match tone {
        DossierTone::Bone => palette::BONE,
        DossierTone::BoneDim => palette::BONE.with_alpha(0.7),
        DossierTone::Gold => palette::GOLD,
        DossierTone::Crimson => palette::CRIMSON,
        DossierTone::Dim => palette::DIM,
    }
}

fn section_header(text: &str) -> impl Bundle {
    (
        Text::new(text),
        TextColor(palette::GOLD),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        ObserverFontRole::Body,
        DeclaredSurface::new(CARD_SURFACE),
    )
}

fn gold_rule() -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: px(1),
            ..default()
        },
        BackgroundColor(palette::GOLD.with_alpha(0.25)),
    )
}

/// `Startup` system: spawns the right-docked dossier card chrome — the
/// client's first styled card (FIELD panel, gold hairline border, soft
/// shadow, gold rules between zones), with every zone present but empty;
/// the repaint fills zones from resources.
fn spawn_dossier_card(mut commands: Commands) {
    let zone = |kind: DossierZone| (kind, DeclaredSurface::new(CARD_SURFACE));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(24),
                top: px(24),
                bottom: px(24),
                width: px(384),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                padding: UiRect::all(px(16)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            BackgroundColor(palette::FIELD.with_alpha(0.93)),
            BorderColor::all(palette::GOLD.with_alpha(0.35)),
            BoxShadow::new(Color::BLACK.with_alpha(0.5), px(0), px(4), px(0), px(8)),
            ZIndex(5),
            Visibility::Hidden,
            DeclaredSurface::new(CARD_SURFACE),
            DossierCardRoot,
            TabGroup::new(20),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new(""),
                TextColor(palette::BONE),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                ObserverFontRole::Display,
                zone(DossierZone::Title),
                ObserverFocusTarget::reading(None),
            ));
            card.spawn((
                Text::new(""),
                TextColor(palette::GOLD),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                ObserverFontRole::Body,
                zone(DossierZone::Question),
            ));
            card.spawn((
                Text::new(""),
                TextFont::default(),
                ObserverFontRole::Body,
                zone(DossierZone::DualTick),
            ));
            card.spawn(gold_rule());
            card.spawn(section_header("S I G N A L S"));
            card.spawn((
                Text::new(""),
                TextFont::default(),
                ObserverFontRole::Exact,
                zone(DossierZone::Signals),
            ));
            card.spawn(gold_rule());
            card.spawn(section_header("P L A C E S"));
            card.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(6),
                    row_gap: px(6),
                    ..default()
                },
                zone(DossierZone::Places),
            ));
            card.spawn(gold_rule());
            card.spawn(section_header("C H R O N I C L E"));
            card.spawn((
                Text::new(""),
                TextFont::default(),
                ObserverFontRole::Exact,
                zone(DossierZone::Chronicle),
            ));
            card.spawn(gold_rule());
            spawn_sealed_actions(card);
        });
}

fn spawn_sealed_actions(card: &mut ChildSpawnerCommands) {
    // Actions footer: the visibly-unavailable Investigate chip.
    card.spawn((
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            padding: UiRect::axes(px(8), px(4)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(4)),
            ..default()
        },
        BorderColor::all(palette::DIM),
        DossierZone::Actions,
        DeclaredSurface::new(CARD_SURFACE),
    ))
    .with_child((
        Text::new(INVESTIGATE_SEALED_CHIP),
        TextColor(palette::DIM),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        ObserverFontRole::Body,
        DeclaredSurface::new(CARD_SURFACE),
    ));
}

#[derive(SystemParam)]
struct DossierReadIdentity<'w> {
    campaign: Res<'w, DossierCampaignId>,
    refresh: Res<'w, DossierRefresh>,
    selected: Res<'w, SelectedCounty>,
    atlas: Option<Res<'w, CountyAtlas>>,
    observer: Option<Res<'w, ObserverSession>>,
    frame: Option<Res<'w, ObserverFrame>>,
}

impl DossierReadIdentity<'_> {
    fn county(&self) -> Option<&str> {
        self.atlas
            .as_ref()?
            .county(self.selected.0?)
            .map(|county| county.fips)
    }

    fn scope(&self, view: &DossierPageView) -> Option<DossierRequestScope> {
        let county = self.county()?;
        let observer = self.observer.as_ref().map(|session| session.context());
        let read_scope = if let Some(session) = &self.observer {
            let snapshot = self.frame.as_ref()?.for_session(session)?;
            if session.campaign != self.campaign.0 {
                return None;
            }
            observation_scope(
                session.campaign,
                snapshot.resolve_tick,
                snapshot.tick_content_hash.as_deref(),
            )
            .ok()?
        } else {
            ArchiveReadScopeV2::foundation(self.campaign.0)
        };
        let subject = match view {
            DossierPageView::Card => {
                ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, county.into()).ok()?
            }
            DossierPageView::Subject(request) => request.target()?,
        };
        Some(DossierRequestScope {
            campaign: self.campaign.0,
            county_geoid: county.into(),
            refresh_generation: self.refresh.0,
            observer,
            read_scope,
            subject,
        })
    }

    fn admits(&self, installed: &InstalledDossier, view: &DossierPageView) -> bool {
        let Some(expected) = self.scope(view) else {
            return false;
        };
        if expected.campaign != installed.scope.campaign
            || expected.county_geoid != installed.scope.county_geoid
            || expected.refresh_generation != installed.scope.refresh_generation
            || expected.observer != installed.scope.observer
            || expected.subject != installed.scope.subject
            || installed.read.scope != installed.scope.read_scope
            || installed.read.subject != installed.scope.subject
        {
            return false;
        }
        match (&self.observer, &self.frame) {
            (Some(session), Some(frame)) => installed
                .for_observer(session, frame, self.refresh.0, &expected.county_geoid)
                .is_some(),
            (None, _) => true,
            _ => false,
        }
    }
}

impl SubjectPageRequest {
    fn target(&self) -> Option<ArchivePageRefV1> {
        let kind = match self.kind.as_str() {
            "county" => ArchiveSubjectKindV1::County,
            "place" => ArchiveSubjectKindV1::Place,
            _ => return None,
        };
        ArchivePageRefV1::try_new(kind, self.id.clone()).ok()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FetchKey {
    scope: DossierRequestScope,
    cursor: Option<ArchiveChangeCursorV2>,
}

fn same_navigation_scope(left: &DossierRequestScope, right: &DossierRequestScope) -> bool {
    left.campaign == right.campaign
        && left.county_geoid == right.county_geoid
        && left.observer == right.observer
        && left.read_scope == right.read_scope
}

fn drive_dossier_fetch(
    identity: DossierReadIdentity,
    mut last: Local<Option<FetchKey>>,
    mut state: ResMut<DossierFetchState>,
    mut projection: ResMut<ActiveCountyDossier>,
    mut view: ResMut<DossierPageView>,
    mut presentation: ResMut<DossierPresentation>,
) {
    if last.is_some()
        && !identity.campaign.is_changed()
        && !identity.refresh.is_changed()
        && !identity.selected.is_changed()
        && !view.is_changed()
        && !presentation.is_changed()
        && !identity
            .observer
            .as_ref()
            .is_some_and(DetectChanges::is_changed)
        && !identity
            .frame
            .as_ref()
            .is_some_and(DetectChanges::is_changed)
    {
        return;
    }
    let Some(mut scope) = identity.scope(&view) else {
        if last.take().is_some() || projection.0.is_some() {
            projection.0 = None;
            *view = DossierPageView::Card;
            *presentation = DossierPresentation::default();
        }
        let next = if identity.selected.0.is_some() {
            DossierFetchState::WaitingForObservation
        } else {
            DossierFetchState::Idle
        };
        if !matches!(
            (&*state, &next),
            (DossierFetchState::Idle, DossierFetchState::Idle)
                | (
                    DossierFetchState::WaitingForObservation,
                    DossierFetchState::WaitingForObservation
                )
        ) {
            *state = next;
        }
        return;
    };
    if last
        .as_ref()
        .is_some_and(|old| !same_navigation_scope(&old.scope, &scope))
    {
        *view = DossierPageView::Card;
        *presentation = DossierPresentation::default();
        scope = identity
            .scope(&view)
            .expect("the selected county scope was admitted");
    }
    let key = FetchKey {
        scope: scope.clone(),
        cursor: presentation.cursor.clone(),
    };
    if last.as_ref() == Some(&key) {
        return;
    }
    *last = Some(key);
    projection.0 = None;
    let cursor = presentation.cursor.clone();
    let request = scope.clone();
    let task = AsyncComputeTaskPool::get().spawn(async move { fetch_dossier(request, cursor) });
    *state = DossierFetchState::InFlight { scope, task };
}

fn collect_dossier_fetch(
    mut state: ResMut<DossierFetchState>,
    mut projection: ResMut<ActiveCountyDossier>,
    identity: DossierReadIdentity,
    view: Res<DossierPageView>,
) {
    let DossierFetchState::InFlight { scope, .. } = &*state else {
        return;
    };
    if identity.scope(&view).as_ref() != Some(scope) {
        *state = DossierFetchState::Idle;
        projection.0 = None;
        return;
    }
    let expected = scope.clone();
    let DossierFetchState::InFlight { task, .. } = state.bypass_change_detection() else {
        unreachable!()
    };
    let Some(result) = block_on(bevy::tasks::futures_lite::future::poll_once(task)) else {
        return;
    };
    *state = match result {
        Ok(installed)
            if identity.admits(&installed, &view)
                && (expected.observer.is_none() || installed.scope == expected) =>
        {
            projection.0 = Some(installed);
            DossierFetchState::Idle
        }
        Ok(_) => DossierFetchState::Failed(DossierFetchError::ReadFailed(
            "Archive returned a different observation than requested".into(),
        )),
        Err(error) => DossierFetchState::Failed(error),
    };
}

/// Rebuilds one zone entity's content from toned segments: despawn the old
/// children, spawn one `TextSpan` per segment (the client's first span
/// children — per-span color and size).
fn set_segments(commands: &mut Commands, zone: Entity, segments: &[DossierSegment]) {
    commands.entity(zone).despawn_related::<Children>();
    for segment in segments {
        commands.entity(zone).with_child((
            TextSpan::new(segment.text.clone()),
            TextColor(tone_color(segment.tone)),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            ObserverFontRole::Body,
            DeclaredSurface::new(CARD_SURFACE),
        ));
    }
}

/// Rebuilds one zone entity as a single styled line.
fn set_line(
    commands: &mut Commands,
    zone: Entity,
    text: String,
    color: Color,
    font_size: f32,
    font_role: ObserverFontRole,
) {
    commands.entity(zone).despawn_related::<Children>();
    commands.entity(zone).with_child((
        TextSpan::new(text),
        TextColor(color),
        TextFont {
            font_size,
            ..default()
        },
        font_role,
        DeclaredSurface::new(CARD_SURFACE),
    ));
}

/// Rebuilds one zone entity's children as one multi-segment row per entry
/// (signal rows, chronicle rows) — each row a `Text` with `TextSpan`
/// children.
fn set_segment_rows(commands: &mut Commands, zone: Entity, rows: &[Vec<DossierSegment>]) {
    commands.entity(zone).despawn_related::<Children>().insert((
        Node {
            width: percent(100),
            min_width: px(0),
            max_width: percent(100),
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            row_gap: px(12),
            ..default()
        },
        TextLayout::new_with_linebreak(bevy::text::LineBreak::WordOrCharacter),
    ));
    for row in rows {
        commands.entity(zone).with_children(|parent| {
            parent
                .spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextLayout::new_with_linebreak(bevy::text::LineBreak::WordOrCharacter),
                    Node {
                        width: percent(100),
                        min_width: px(0),
                        max_width: percent(100),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    ObserverFontRole::Exact,
                    DeclaredSurface::new(CARD_SURFACE),
                ))
                .with_children(|text| {
                    for segment in row {
                        text.spawn((
                            TextSpan::new(segment.text.clone()),
                            TextColor(tone_color(segment.tone)),
                            TextFont {
                                font_size: if segment.tone == DossierTone::Dim {
                                    11.0
                                } else {
                                    14.0
                                },
                                ..default()
                            },
                            ObserverFontRole::Exact,
                            DeclaredSurface::new(CARD_SURFACE),
                        ));
                    }
                });
        });
    }
}

#[derive(Message, Clone, Debug, PartialEq, Eq)]
enum DossierControl {
    Link(SubjectPageRequest),
    Back(DossierRequestScope),
    More(DossierRequestScope, ArchiveChangeCursorV2),
    First(DossierRequestScope),
    Details(DossierRequestScope),
}
impl DossierControl {
    fn scope(&self) -> &DossierRequestScope {
        match self {
            Self::Link(request) => &request.scope,
            Self::Back(scope)
            | Self::More(scope, _)
            | Self::First(scope)
            | Self::Details(scope) => scope,
        }
    }
}

fn chip_request(link: &ArchiveDossierLinkV2, scope: &DossierRequestScope) -> SubjectPageRequest {
    SubjectPageRequest {
        scope: scope.clone(),
        kind: link.target.kind().as_str().into(),
        id: link.target.id().into(),
        label: link.retained_label.clone(),
    }
}

fn navigable_link(link: &ArchiveDossierLinkV2) -> bool {
    matches!(
        link.target_state,
        ArchiveLinkedPageStateV2::KnownReady | ArchiveLinkedPageStateV2::KnownPending
    )
}

fn set_chips(
    commands: &mut Commands,
    zone: Entity,
    links: &[ArchiveDossierLinkV2],
    scope: &DossierRequestScope,
) {
    commands.entity(zone).despawn_related::<Children>();
    for link in links {
        let label = if link.target_state == ArchiveLinkedPageStateV2::Unknown {
            babylon_persistence::fog_chip_v1(link.target.kind().as_str(), link.target.id())
        } else {
            link.retained_label.clone().unwrap_or_else(|| {
                format!("{} · {}", link.target.kind().as_str(), link.target.id())
            })
        };
        let text = match link.target_state {
            ArchiveLinkedPageStateV2::KnownReady | ArchiveLinkedPageStateV2::Unknown => label,
            state => format!("{label} · {}", crate::dossier::link_state_label(state)),
        };
        spawn_control(
            commands,
            zone,
            text,
            DossierControl::Link(chip_request(link, scope)),
        );
    }
}

fn spawn_control(commands: &mut Commands, zone: Entity, label: String, control: DossierControl) {
    commands.entity(zone).with_children(|parent| {
        parent
            .spawn((
                Node {
                    padding: UiRect::axes(px(8), px(4)),
                    border: UiRect::bottom(px(1)),
                    ..default()
                },
                BackgroundColor(palette::MUTED_DARK.with_alpha(0.85)),
                BorderColor::all(palette::DIM),
                ObserverFocusTarget::action(control.scope().observer.clone()),
                PlaceChipNode {
                    control,
                    base_border: palette::DIM,
                },
                DeclaredSurface::new(CARD_SURFACE),
            ))
            .observe(on_place_chip_click)
            .observe(on_place_chip_over)
            .observe(on_place_chip_out)
            .with_child((
                Text::new(label),
                TextColor(palette::BONE),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                ObserverFontRole::Body,
                DeclaredSurface::new(CARD_SURFACE),
            ));
    });
}

#[derive(SystemParam)]
struct DossierReadContext<'w> {
    identity: DossierReadIdentity<'w>,
    projection: Res<'w, ActiveCountyDossier>,
    fetch: Res<'w, DossierFetchState>,
    ui: Option<Res<'w, ObserverUiState>>,
}
impl DossierReadContext<'_> {
    fn visible(&self) -> bool {
        self.identity.selected.0.is_some()
            && self.ui.as_ref().is_none_or(|ui| {
                ui.archive_open && !ui.menu_open && !ui.splash_visible && !ui.comparison_open
            })
    }
    fn admits(&self, control: &DossierControl, view: &DossierPageView) -> bool {
        admitted_control(
            control,
            &self.identity,
            &self.projection,
            &self.fetch,
            self.ui.as_deref(),
            view,
        )
    }
}

fn admitted_control(
    control: &DossierControl,
    identity: &DossierReadIdentity,
    projection: &ActiveCountyDossier,
    fetch: &DossierFetchState,
    ui: Option<&ObserverUiState>,
    view: &DossierPageView,
) -> bool {
    if identity.selected.0.is_none()
        || ui.is_some_and(|ui| {
            !ui.archive_open || ui.menu_open || ui.splash_visible || ui.comparison_open
        })
    {
        return false;
    }
    if let DossierControl::Back(scope) = control {
        return matches!(view, DossierPageView::Subject(_))
            && identity.scope(view).as_ref() == Some(scope);
    }
    if !matches!(fetch, DossierFetchState::Idle) {
        return false;
    }
    let Some(installed) = projection
        .0
        .as_ref()
        .filter(|installed| identity.admits(installed, view))
    else {
        return false;
    };
    if control.scope() != &installed.scope {
        return false;
    }
    let page = retained_page(&installed.read);
    match control {
        DossierControl::Link(request) => page.is_some_and(|page| {
            page.links.iter().any(|link| {
                navigable_link(link) && chip_request(link, &installed.scope) == *request
            })
        }),
        DossierControl::Back(_) => matches!(view, DossierPageView::Subject(_)),
        DossierControl::More(_, cursor) => {
            page.is_some_and(|page| page.changes.next_cursor.as_ref() == Some(cursor))
        }
        DossierControl::First(_) | DossierControl::Details(_) => page.is_some(),
    }
}

#[derive(SystemParam)]
struct DossierRefusal<'w> {
    feedback: Option<ResMut<'w, ObserverFeedback>>,
    time: Option<Res<'w, Time>>,
}
impl DossierRefusal<'_> {
    fn reject(&mut self) {
        if let Some(feedback) = &mut self.feedback {
            feedback.reject(
                "This Archive control belongs to an unavailable observation.",
                self.time
                    .as_ref()
                    .map_or(0.0, |time| time.elapsed_secs_f64()),
            );
        }
    }
}

fn dispatch_control(
    control: &DossierControl,
    context: &DossierReadContext,
    view: &DossierPageView,
    refusal: &mut DossierRefusal,
    requests: &mut MessageWriter<DossierControl>,
    subjects: &mut MessageWriter<SubjectPageRequest>,
) {
    if context.admits(control, view) {
        requests.write(control.clone());
        if let DossierControl::Link(request) = control {
            subjects.write(request.clone());
        }
    } else {
        refusal.reject();
    }
}

fn on_place_chip_click(
    click: On<Pointer<Click>>,
    chips: Query<&PlaceChipNode>,
    context: DossierReadContext,
    view: Res<DossierPageView>,
    mut refusal: DossierRefusal,
    mut requests: MessageWriter<DossierControl>,
    mut subjects: MessageWriter<SubjectPageRequest>,
) {
    if let Ok(chip) = chips.get(click.entity) {
        dispatch_control(
            &chip.control,
            &context,
            &view,
            &mut refusal,
            &mut requests,
            &mut subjects,
        );
    }
}
fn keyboard_activate(
    event: On<ObserverKeyboardActivate>,
    chips: Query<&PlaceChipNode>,
    context: DossierReadContext,
    view: Res<DossierPageView>,
    mut refusal: DossierRefusal,
    mut requests: MessageWriter<DossierControl>,
    mut subjects: MessageWriter<SubjectPageRequest>,
) {
    let Ok(chip) = chips.get(event.entity) else {
        return;
    };
    if event.context != chip.control.scope().observer {
        refusal.reject();
        return;
    }
    dispatch_control(
        &chip.control,
        &context,
        &view,
        &mut refusal,
        &mut requests,
        &mut subjects,
    );
}

type DossierFocusOwners = Or<(With<PlaceChipNode>, With<DossierZone>)>;
fn focus_eligibility(
    context: DossierReadContext,
    view: Res<DossierPageView>,
    mut targets: Query<(&mut ObserverFocusTarget, Option<&PlaceChipNode>), DossierFocusOwners>,
) {
    let identity = &context.identity;
    if !(identity.campaign.is_changed()
        || identity.refresh.is_changed()
        || identity.selected.is_changed()
        || context.projection.is_changed()
        || context.fetch.is_changed()
        || view.is_changed()
        || targets.iter_mut().any(|(target, _)| target.is_changed())
        || identity
            .observer
            .as_ref()
            .is_some_and(DetectChanges::is_changed)
        || identity
            .frame
            .as_ref()
            .is_some_and(DetectChanges::is_changed)
        || context.ui.as_ref().is_some_and(DetectChanges::is_changed))
    {
        return;
    }
    for (mut target, chip) in &mut targets {
        let mut next = target.clone();
        if let Some(chip) = chip {
            next.context.clone_from(&chip.control.scope().observer);
            next.available = context.admits(&chip.control, &view);
        } else {
            next.available = context.visible()
                && match (&identity.observer, &next.context) {
                    (Some(session), Some(scope)) => session.accepts(scope),
                    (None, None) => true,
                    _ => false,
                };
        }
        target.set_if_neq(next);
    }
}
fn on_place_chip_over(
    over: On<Pointer<Over>>,
    mut chips: Query<&mut BorderColor, With<PlaceChipNode>>,
) {
    if let Ok(mut border) = chips.get_mut(over.entity) {
        *border = BorderColor::all(palette::GOLD);
    }
}
fn on_place_chip_out(out: On<Pointer<Out>>, mut chips: Query<(&mut BorderColor, &PlaceChipNode)>) {
    if let Ok((mut border, chip)) = chips.get_mut(out.entity) {
        *border = BorderColor::all(chip.base_border);
    }
}

#[derive(SystemParam)]
struct DossierWriteState<'w> {
    view: ResMut<'w, DossierPageView>,
    projection: ResMut<'w, ActiveCountyDossier>,
    fetch: ResMut<'w, DossierFetchState>,
    presentation: ResMut<'w, DossierPresentation>,
}

fn apply_page_requests(
    mut requests: MessageReader<DossierControl>,
    identity: DossierReadIdentity,
    outputs: DossierWriteState,
    ui: Option<Res<ObserverUiState>>,
    mut refusal: DossierRefusal,
) {
    let DossierWriteState {
        mut view,
        mut projection,
        fetch: mut state,
        mut presentation,
    } = outputs;
    for control in requests.read() {
        // Reuse the same admission function without overlapping mutable resource borrows.
        let admitted = admitted_control(
            control,
            &identity,
            &projection,
            &state,
            ui.as_deref(),
            &view,
        );
        if !admitted {
            refusal.reject();
            continue;
        }
        match control {
            DossierControl::Link(request) => {
                *view = DossierPageView::Subject(Box::new(request.clone()));
                *presentation = DossierPresentation::default();
            }
            DossierControl::Back(_) => {
                *view = DossierPageView::Card;
                *presentation = DossierPresentation::default();
            }
            DossierControl::More(_, cursor) => presentation.cursor = Some(cursor.clone()),
            DossierControl::First(_) => presentation.cursor = None,
            DossierControl::Details(_) => {
                presentation.details_open = !presentation.details_open;
                continue;
            }
        }
        projection.0 = None;
        *state = DossierFetchState::Idle;
    }
}

type DossierRoots<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Visibility,
        &'static mut Node,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
        &'static mut BoxShadow,
    ),
    With<DossierCardRoot>,
>;

fn repaint_dossier_card(
    mut commands: Commands,
    context: DossierReadContext,
    view: Res<DossierPageView>,
    presentation: Res<DossierPresentation>,
    zones: Query<(Entity, &DossierZone)>,
    mut roots: DossierRoots,
) {
    let Ok((root, mut visibility, mut node, mut background, mut border, mut shadow)) =
        roots.single_mut()
    else {
        return;
    };
    if context.ui.as_ref().is_some_and(DetectChanges::is_added) {
        commands
            .entity(root)
            .insert(crate::observer_layout::ObserverRegion::Log);
        node.right = Val::Auto;
        node.bottom = Val::Auto;
        node.min_width = Val::Auto;
        node.max_width = Val::Auto;
        node.overflow = Overflow::scroll_y();
        node.border = UiRect::left(px(2));
        node.border_radius = BorderRadius::ZERO;
        *background = BackgroundColor(crate::observer_theme::PANEL);
        *border = BorderColor::all(crate::observer_theme::PAPER);
        *shadow = BoxShadow::default();
    }
    if !context.visible() {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    }
    let became_visible = *visibility != Visibility::Visible;
    visibility.set_if_neq(Visibility::Visible);
    let identity = &context.identity;
    if !(became_visible
        || context.projection.is_changed()
        || context.fetch.is_changed()
        || view.is_changed()
        || presentation.is_changed()
        || identity.selected.is_changed()
        || identity.refresh.is_changed()
        || identity.campaign.is_changed()
        || identity
            .observer
            .as_ref()
            .is_some_and(DetectChanges::is_changed)
        || identity
            .frame
            .as_ref()
            .is_some_and(DetectChanges::is_changed))
    {
        return;
    }
    paint_zones(&mut commands, &context, &view, &presentation, &zones);
}

fn paint_zones(
    commands: &mut Commands,
    context: &DossierReadContext,
    view: &DossierPageView,
    presentation: &DossierPresentation,
    zones: &Query<(Entity, &DossierZone)>,
) {
    let identity = &context.identity;
    let installed = context
        .projection
        .0
        .as_ref()
        .filter(|installed| identity.admits(installed, view));
    let read = installed.map(|installed| &installed.read);
    let page = read.and_then(retained_page);
    for (entity, zone) in zones {
        match zone {
            DossierZone::Title => {
                commands.entity(entity).insert(ObserverFocusTarget::reading(
                    identity.observer.as_ref().map(|session| session.context()),
                ));
                let fallback = match *view {
                    DossierPageView::Card => identity
                        .selected
                        .0
                        .and_then(|index| identity.atlas.as_ref()?.county(index))
                        .map_or("County Archive", |county| county.name),
                    DossierPageView::Subject(_) => "Archive subject",
                };
                set_line(
                    commands,
                    entity,
                    page.map_or(fallback, |page| &page.title).into(),
                    palette::BONE,
                    24.0,
                    ObserverFontRole::Display,
                );
            }
            DossierZone::Question => set_line(
                commands,
                entity,
                page.map_or_else(
                    || {
                        if context.ui.is_some() {
                            "Which cited observations are available at this week?".into()
                        } else {
                            DOSSIER_DECISION_QUESTION.into()
                        }
                    },
                    |page| page.question.clone(),
                ),
                palette::GOLD,
                13.0,
                ObserverFontRole::Body,
            ),
            DossierZone::DualTick => {
                set_segments(commands, entity, &status_segments(&context.fetch, read));
            }
            DossierZone::Signals => {
                let rows = page.map_or_else(Vec::new, |page| {
                    page.signals
                        .iter()
                        .map(|signal| retained_signal_segments(signal, presentation.details_open))
                        .collect()
                });
                set_segment_rows(commands, entity, &rows);
            }
            DossierZone::Places => {
                if let (Some(installed), Some(page)) = (installed, page) {
                    set_chips(commands, entity, &page.links, &installed.scope);
                } else {
                    commands.entity(entity).despawn_related::<Children>();
                }
            }
            DossierZone::Chronicle => {
                let rows = read
                    .filter(|_| presentation.details_open)
                    .map_or_else(Vec::new, evidence_rows);
                set_segment_rows(commands, entity, &rows);
            }
            DossierZone::Actions => paint_actions(
                commands,
                entity,
                installed,
                matches!(view, DossierPageView::Subject(_))
                    .then(|| identity.scope(view))
                    .flatten()
                    .as_ref(),
                presentation,
                context.ui.is_some(),
            ),
        }
    }
}

fn evidence_rows(read: &ArchiveDossierReadV2) -> Vec<Vec<DossierSegment>> {
    let mut rows = vec![vec![chronicle_header(&read.state)]];
    let Some(page) = retained_page(read) else {
        return rows;
    };
    rows.extend(page.changes.changes.iter().map(chronicle_row_segments));
    rows.push(vec![DossierSegment {
        text: format!(
            "Content observed at week {}; published at week {}.\nRevision {}\nContent SHA256 {}",
            page.content_source.tick(),
            page.effective_tick,
            crate::dossier::hex_bytes(page.revision_id),
            crate::dossier::hex_bytes(page.content_sha256)
        ),
        tone: DossierTone::Dim,
    }]);
    rows.push(vec![DossierSegment {
        text: format!("Original publication\n{}", page.markdown),
        tone: DossierTone::BoneDim,
    }]);
    for citation in &page.citations {
        rows.push(vec![DossierSegment {
            text: format!("{}; {}", citation.source_id(), citation.locator()),
            tone: DossierTone::Dim,
        }]);
    }
    rows
}

fn status_segments(
    state: &DossierFetchState,
    read: Option<&ArchiveDossierReadV2>,
) -> Vec<DossierSegment> {
    let Some(read) = read else {
        let (text, tone) = match state {
            DossierFetchState::Idle => (
                "No cited observation is installed.".into(),
                DossierTone::Dim,
            ),
            DossierFetchState::WaitingForObservation => (
                "Waiting for the selected week's committed observation.".into(),
                DossierTone::Dim,
            ),
            DossierFetchState::InFlight { .. } => {
                ("Reading cited observations...".into(), DossierTone::Dim)
            }
            DossierFetchState::Failed(DossierFetchError::ReaderAbsent(_)) => {
                ("Archive reader not configured".into(), DossierTone::Dim)
            }
            DossierFetchState::Failed(error) => (error.to_string(), DossierTone::Crimson),
        };
        return vec![DossierSegment { text, tone }];
    };
    let verified = verified_tick(read);
    let mut segments = vec![DossierSegment {
        text: format!(
            "Viewing week {} · durable week {}\n{}",
            read.scope.tick(),
            read.durable_tick,
            availability_label(read)
        ),
        tone: if verified.is_some() {
            DossierTone::Gold
        } else {
            DossierTone::Crimson
        },
    }];
    if let Some(tick) = verified {
        segments.push(DossierSegment {
            text: format!(" · verified through {tick}"),
            tone: DossierTone::Gold,
        });
    }
    if let Some(page) = retained_page(read) {
        segments.push(DossierSegment {
            text: format!("\nContent last published at week {}", page.effective_tick),
            tone: DossierTone::Dim,
        });
    }
    segments
}

fn paint_actions(
    commands: &mut Commands,
    entity: Entity,
    installed: Option<&InstalledDossier>,
    back_scope: Option<&DossierRequestScope>,
    presentation: &DossierPresentation,
    observer: bool,
) {
    commands.entity(entity).despawn_related::<Children>();
    if let Some(scope) = back_scope {
        spawn_control(
            commands,
            entity,
            "Back to county".into(),
            DossierControl::Back(scope.clone()),
        );
    }
    if let Some(installed) = installed {
        let scope = &installed.scope;
        if let Some(page) = retained_page(&installed.read) {
            spawn_control(
                commands,
                entity,
                if presentation.details_open {
                    "Hide evidence"
                } else {
                    "Evidence and changes"
                }
                .into(),
                DossierControl::Details(scope.clone()),
            );
            if presentation.details_open {
                if let Some(cursor) = &page.changes.next_cursor {
                    spawn_control(
                        commands,
                        entity,
                        "More changes".into(),
                        DossierControl::More(scope.clone(), cursor.clone()),
                    );
                }
                if presentation.cursor.is_some() {
                    spawn_control(
                        commands,
                        entity,
                        "First changes".into(),
                        DossierControl::First(scope.clone()),
                    );
                }
            }
        }
    }
    commands.entity(entity).with_child((
        Text::new(if observer {
            "READ-ONLY ARCHIVE"
        } else {
            INVESTIGATE_SEALED_CHIP
        }),
        TextColor(palette::DIM),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        ObserverFontRole::Body,
        DeclaredSurface::new(CARD_SURFACE),
    ));
}

type AddedCardNodes<'w, 's> = Query<
    'w,
    's,
    (&'static DeclaredSurface, &'static mut Node),
    (Added<Node>, Without<DossierCardRoot>),
>;

/// Apply observer typography to rendered card entities without changing the
/// shared Archive composition or the player-action conformance contract.
fn polish_observer_card(
    mut texts: Query<(&DeclaredSurface, &mut TextColor), Changed<TextColor>>,
    mut nodes: AddedCardNodes,
    mut lines: Query<(&DeclaredSurface, &mut Text), Changed<Text>>,
    mut spans: Query<(&DeclaredSurface, &mut TextSpan), Changed<TextSpan>>,
) {
    use crate::observer_theme as theme;
    fn supported_glyphs(value: &str) -> String {
        value
            .replace('·', "|")
            .replace('…', "...")
            .replace(['—', '–'], "-")
            .replace('→', ">")
            .replace('←', "<")
            .replace('φ', "phi")
    }
    for (surface, mut text) in &mut lines {
        if surface.id == CARD_SURFACE && !text.0.is_ascii() {
            let next = supported_glyphs(&text.0);
            if text.0 != next {
                text.0 = next;
            }
        }
    }
    for (surface, mut span) in &mut spans {
        if surface.id == CARD_SURFACE && !span.0.is_ascii() {
            let next = supported_glyphs(&span.0);
            if span.0 != next {
                span.0 = next;
            }
        }
    }
    for (surface, mut color) in &mut texts {
        if surface.id != CARD_SURFACE {
            continue;
        }
        let next = if color.0 == palette::DIM || color.0 == palette::BONE.with_alpha(0.7) {
            theme::GRAY
        } else if color.0 == palette::BONE {
            theme::PAPER
        } else if color.0 == palette::GOLD {
            theme::YELLOW
        } else if color.0 == palette::CRIMSON {
            theme::RED
        } else {
            color.0
        };
        if color.0 != next {
            color.0 = next;
        }
    }
    for (surface, mut node) in &mut nodes {
        if surface.id == CARD_SURFACE {
            node.flex_shrink = 0.0;
            node.min_width = px(0);
            node.max_width = percent(100);
        }
    }
}

/// Wires the dossier card's resource family and systems into an `App`.
pub struct DossierCardPlugin;

impl Plugin for DossierCardPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SubjectPageRequest>()
            .add_message::<DossierControl>()
            .init_resource::<DossierPresentation>()
            .init_resource::<DossierCampaignId>()
            .init_resource::<ActiveCountyDossier>()
            .init_resource::<DossierPageView>()
            .init_resource::<DossierFetchState>()
            .init_resource::<DossierRefresh>()
            .add_observer(keyboard_activate)
            .add_systems(
                PreUpdate,
                focus_eligibility.in_set(ObserverFocusSystems::Eligibility),
            )
            .add_systems(Startup, spawn_dossier_card)
            .add_systems(
                Update,
                (
                    // `.after(restart_on_n_key)`: the N-key restart clears
                    // the card THROUGH this system — its `SelectedCounty =
                    // None` write is the change this system reacts to, and
                    // the ordering makes the clear land in the same frame
                    // instead of one frame late. Vacuous when TickLoopPlugin
                    // is absent (headless unit compositions).
                    drive_dossier_fetch.after(crate::ui::story_card::restart_on_n_key),
                    collect_dossier_fetch,
                    apply_page_requests,
                    repaint_dossier_card,
                    polish_observer_card
                        .run_if(resource_exists::<crate::observer_ui::ObserverUiState>),
                )
                    .chain()
                    .after(crate::observer_io::ObserverSet::Install)
                    .before(crate::observer_io::ObserverSet::Paint),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use babylon_persistence::archive_revision::{
        ArchiveChangePageV2, ArchiveDossierPageV2, ArchiveDossierPendingV2, ArchiveDossierStateV2,
        ArchivePublicationOriginV2,
    };
    use babylon_persistence::{
        ArchiveCitationV1, ArchiveSignalV1, ObserverEconomySnapshotV1, ObserverVisibilityV1,
    };

    fn subject(kind: ArchiveSubjectKindV1, id: &str) -> ArchivePageRefV1 {
        ArchivePageRefV1::try_new(kind, id.into()).unwrap()
    }
    fn frame(session: &ObserverSession, hash: &str) -> ObserverFrame {
        ObserverFrame(Some(ObserverEconomySnapshotV1 {
            campaign_id: session.campaign.as_uuid().to_string(),
            resolve_tick: session.viewed_tick,
            foundation_digest: "foundation".into(),
            nominal_world_hash: None,
            tick_content_hash: Some(hash.into()),
            envelope_digest: None,
            visibility: ObserverVisibilityV1::FullObserver,
            counties: vec![],
            production: None,
        }))
    }
    fn installed(session: &ObserverSession, hash: &str) -> InstalledDossier {
        let read_scope =
            observation_scope(session.campaign, session.viewed_tick, Some(hash)).unwrap();
        let subject = subject(ArchiveSubjectKindV1::County, "26163");
        let page = ArchiveDossierPageV2 {
            revision_id: [1; 32],
            effective_tick: 1,
            origin: ArchivePublicationOriginV2::Materialized,
            content_source: read_scope.clone(),
            title: "Wayne County".into(),
            question: "Retained question?".into(),
            signals: vec![signal()],
            markdown: "Exact retained narrative".into(),
            content_sha256: [2; 32],
            citations: vec![],
            atoms: vec![],
            links: vec![ArchiveDossierLinkV2 {
                target: self::subject(ArchiveSubjectKindV1::Place, "2622000"),
                retained_label: Some("Retained Detroit".into()),
                target_state: ArchiveLinkedPageStateV2::KnownReady,
            }],
            changes: ArchiveChangePageV2 {
                coverage_from_tick: 1,
                changes: vec![],
                next_cursor: None,
            },
        };
        InstalledDossier {
            scope: DossierRequestScope {
                campaign: session.campaign,
                county_geoid: "26163".into(),
                refresh_generation: 0,
                observer: Some(session.context()),
                read_scope: read_scope.clone(),
                subject: subject.clone(),
            },
            read: ArchiveDossierReadV2 {
                scope: read_scope,
                subject,
                durable_tick: session.durable_tick,
                processed_tick: session.durable_tick,
                history_floor_tick: 1,
                state: ArchiveDossierStateV2::Ready {
                    page,
                    verified_through_tick: session.viewed_tick,
                },
            },
        }
    }
    fn signal() -> ArchiveSignalV1 {
        ArchiveSignalV1::try_new(
            "jobs".into(),
            "Original jobs label".into(),
            "1469 annual average jobs".into(),
            ArchiveCitationV1::try_new("observed-source".into(), "exact/locator".into()).unwrap(),
        )
        .unwrap()
    }
    fn page_mut(installed: &mut InstalledDossier) -> &mut ArchiveDossierPageV2 {
        match &mut installed.read.state {
            ArchiveDossierStateV2::Ready { page, .. } => page,
            _ => panic!("ready fixture"),
        }
    }
    fn chip_app() -> (App, Entity, SubjectPageRequest) {
        let campaign = CampaignId::from_uuid(uuid::Uuid::nil());
        let mut session = ObserverSession::new(campaign);
        session.ready(1, Some("a".repeat(64)));
        assert!(session.installed(&session.context()));
        let frame = frame(&session, &"a".repeat(64));
        let installed = installed(&session, &"a".repeat(64));
        let scope = installed.scope.clone();
        let links = retained_page(&installed.read).unwrap().links.clone();
        let request = chip_request(&links[0], &scope);
        let atlas = CountyAtlas::parse(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/map/county_atlas.bin"
        )))
        .unwrap();
        let selected = SelectedCounty(atlas.index_of_fips("26163"));
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(atlas)
            .insert_resource(selected)
            .insert_resource(session)
            .insert_resource(frame)
            .insert_resource(DossierCampaignId(campaign))
            .insert_resource(ObserverUiState {
                archive_open: true,
                menu_open: false,
                splash_visible: false,
                ..default()
            })
            .insert_resource(ActiveCountyDossier(Some(installed)))
            .init_resource::<DossierRefresh>()
            .init_resource::<DossierFetchState>()
            .init_resource::<DossierPageView>()
            .init_resource::<DossierPresentation>()
            .init_resource::<ObserverFeedback>()
            .add_message::<SubjectPageRequest>()
            .add_message::<DossierControl>()
            .add_observer(keyboard_activate)
            .add_systems(Startup, move |mut commands: Commands| {
                let zone = commands.spawn((Node::default(), TabGroup::new(20))).id();
                set_chips(&mut commands, zone, &links, &scope);
            })
            .add_systems(PreUpdate, focus_eligibility)
            .add_systems(Update, apply_page_requests);
        app.update();
        app.update();
        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<PlaceChipNode>>()
            .single(app.world())
            .unwrap();
        (app, entity, request)
    }

    #[test]
    fn keyboard_archive_chip_opens_the_exact_current_link() {
        let (mut app, entity, request) = chip_app();
        assert!(
            app.world()
                .get::<ObserverFocusTarget>(entity)
                .unwrap()
                .available
        );
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity,
            context: request.scope.observer.clone(),
        });
        assert_eq!(
            *app.world().resource::<DossierPageView>(),
            DossierPageView::Card
        );
        app.update();
        assert_eq!(
            *app.world().resource::<DossierPageView>(),
            DossierPageView::Subject(Box::new(request))
        );
        assert!(app.world().resource::<ActiveCountyDossier>().0.is_none());
        assert_eq!(app.world().resource::<ObserverSession>().viewed_tick, 1);
    }

    #[test]
    fn queued_archive_links_refuse_changed_campaign_county_generation_perspective_and_hash() {
        for mutation in 0..6 {
            let (mut app, entity, request) = chip_app();
            app.world_mut().trigger(ObserverKeyboardActivate {
                entity,
                context: request.scope.observer.clone(),
            });
            match mutation {
                0 => {
                    app.world_mut().resource_mut::<DossierCampaignId>().0 =
                        CampaignId::from_uuid(uuid::Uuid::from_u128(9));
                }
                1 => app.world_mut().resource_mut::<SelectedCounty>().0 = None,
                2 => app.world_mut().resource_mut::<DossierRefresh>().bump(),
                3 => app
                    .world_mut()
                    .resource_mut::<ObserverSession>()
                    .set_perspective(crate::observer::Perspective::PlayerKnowledge),
                4 => {
                    page_mut(
                        app.world_mut()
                            .resource_mut::<ActiveCountyDossier>()
                            .0
                            .as_mut()
                            .unwrap(),
                    )
                    .links[0]
                        .target_state = ArchiveLinkedPageStateV2::Unknown;
                }
                5 => {
                    app.world_mut()
                        .resource_mut::<ObserverFrame>()
                        .0
                        .as_mut()
                        .unwrap()
                        .tick_content_hash = Some("b".repeat(64));
                }
                _ => unreachable!(),
            }
            app.update();
            assert_eq!(
                *app.world().resource::<DossierPageView>(),
                DossierPageView::Card,
                "scope mutation {mutation}"
            );
            assert!(app.world().resource::<ObserverFeedback>().message.is_some());
            assert!(
                !app.world()
                    .get::<ObserverFocusTarget>(entity)
                    .unwrap()
                    .available
            );
        }
    }

    #[test]
    fn archive_chip_rejects_forged_activation_context_and_idle_focus_does_not_change() {
        #[derive(Resource, Default)]
        struct ChangedTargets(usize);
        let (mut app, entity, request) = chip_app();
        app.init_resource::<ChangedTargets>().add_systems(
            PostUpdate,
            |changed: Query<(), Changed<ObserverFocusTarget>>,
             mut count: ResMut<ChangedTargets>| {
                count.0 = changed.iter().count();
            },
        );
        app.update();
        app.update();
        assert_eq!(app.world().resource::<ChangedTargets>().0, 0);
        let mut wrong = request.scope.observer.unwrap();
        wrong.tick += 1;
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity,
            context: Some(wrong),
        });
        app.update();
        assert_eq!(
            *app.world().resource::<DossierPageView>(),
            DossierPageView::Card
        );
        assert!(app.world().resource::<ObserverFeedback>().message.is_some());
    }

    #[test]
    fn back_recovers_from_a_failed_link_read_without_changing_the_week() {
        let (mut app, entity, request) = chip_app();
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity,
            context: request.scope.observer.clone(),
        });
        app.update();
        assert!(app.world().resource::<ActiveCountyDossier>().0.is_none());
        let mut scope = request.scope.clone();
        scope.subject = request.target().unwrap();
        *app.world_mut().resource_mut::<DossierFetchState>() =
            DossierFetchState::Failed(DossierFetchError::ReadFailed("offline".into()));
        app.world_mut().write_message(DossierControl::Back(scope));
        app.update();
        assert_eq!(
            *app.world().resource::<DossierPageView>(),
            DossierPageView::Card
        );
        assert_eq!(app.world().resource::<ObserverSession>().viewed_tick, 1);
        assert!(app.world().resource::<ActiveCountyDossier>().0.is_none());
    }

    #[test]
    fn disclosed_unlabeled_link_uses_only_its_public_identity() {
        let (mut app, entity, request) = chip_app();
        let mut unlabeled = request;
        unlabeled.label = None;
        page_mut(
            app.world_mut()
                .resource_mut::<ActiveCountyDossier>()
                .0
                .as_mut()
                .unwrap(),
        )
        .links[0]
            .retained_label = None;
        app.world_mut()
            .get_mut::<PlaceChipNode>(entity)
            .unwrap()
            .control = DossierControl::Link(unlabeled.clone());
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity,
            context: unlabeled.scope.observer.clone(),
        });
        app.update();
        assert_eq!(
            *app.world().resource::<DossierPageView>(),
            DossierPageView::Subject(Box::new(unlabeled))
        );
    }

    #[test]
    fn historical_admission_uses_the_installed_week_hash_and_preserves_pending() {
        let mut session = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::nil()));
        session.ready(2, Some("b".repeat(64)));
        session.inspect_tick(1);
        assert!(session.installed(&session.context()));
        let frame = frame(&session, &"a".repeat(64));
        let mut active = ActiveCountyDossier(Some(installed(&session, &"a".repeat(64))));
        let page = retained_page(&active.0.as_ref().unwrap().read)
            .unwrap()
            .clone();
        active.0.as_mut().unwrap().read.state = ArchiveDossierStateV2::Pending {
            page: Some(page),
            reason: ArchiveDossierPendingV2::KnowledgeRefresh,
        };
        let read = active.for_observer(&session, &frame, 0, "26163").unwrap();
        assert_eq!(read.scope.tick_content_hash(), Some([0xaa; 32]));
        assert_eq!(
            verified_tick(read),
            None,
            "global progress cannot verify a pending historical page"
        );
        for (refresh, county) in [(1, "26163"), (0, "26099")] {
            assert!(active
                .for_observer(&session, &frame, refresh, county)
                .is_none());
        }
        active.0.as_mut().unwrap().read.scope =
            ArchiveReadScopeV2::committed(session.campaign, 1, [0xbb; 32]).unwrap();
        assert!(active.for_observer(&session, &frame, 0, "26163").is_none());
    }

    #[test]
    fn unknown_and_unavailable_links_never_dispatch_a_guessed_subject() {
        for state in [
            ArchiveLinkedPageStateV2::Unknown,
            ArchiveLinkedPageStateV2::KnownUnavailable,
        ] {
            let (mut app, entity, request) = chip_app();
            page_mut(
                app.world_mut()
                    .resource_mut::<ActiveCountyDossier>()
                    .0
                    .as_mut()
                    .unwrap(),
            )
            .links[0]
                .target_state = state;
            app.world_mut().trigger(ObserverKeyboardActivate {
                entity,
                context: request.scope.observer,
            });
            app.update();
            assert_eq!(
                *app.world().resource::<DossierPageView>(),
                DossierPageView::Card
            );
        }
    }

    #[test]
    fn repainted_archive_title_reenters_tab_order_without_recreated_link_chips() {
        use crate::observer_focus::{ObserverFocusPlugin, ObserverFocusPolicy};
        use bevy::ecs::system::RunSystemOnce;
        use bevy::input::InputPlugin;
        use bevy::input_focus::tab_navigation::TabIndex;
        let (mut app, chip, _) = chip_app();
        app.world_mut().despawn(chip);
        page_mut(
            app.world_mut()
                .resource_mut::<ActiveCountyDossier>()
                .0
                .as_mut()
                .unwrap(),
        )
        .links
        .clear();
        app.add_plugins((InputPlugin, ObserverFocusPlugin))
            .configure_sets(
                PreUpdate,
                ObserverFocusSystems::Registration.after(focus_eligibility),
            )
            .add_systems(Update, repaint_dossier_card.after(apply_page_requests));
        app.world_mut()
            .spawn((Window::default(), bevy::window::PrimaryWindow));
        let context = app.world().resource::<ObserverSession>().context();
        app.world_mut()
            .resource_mut::<ObserverFocusPolicy>()
            .context = Some(context.clone());
        app.world_mut().run_system_once(spawn_dossier_card).unwrap();
        app.update();
        app.update();
        let title = {
            let world = app.world_mut();
            world
                .query::<(Entity, &DossierZone)>()
                .iter(world)
                .find_map(|(entity, zone)| (*zone == DossierZone::Title).then_some(entity))
                .unwrap()
        };
        assert!(app.world().get::<TabIndex>(title).is_some());
        let (frame, mut next) = {
            let mut session = app.world_mut().resource_mut::<ObserverSession>();
            session.ready(2, Some("b".repeat(64)));
            let context = session.context();
            assert!(session.installed(&context));
            (
                frame(&session, &"b".repeat(64)),
                installed(&session, &"b".repeat(64)),
            )
        };
        page_mut(&mut next).links.clear();
        app.world_mut().insert_resource(frame);
        app.world_mut()
            .insert_resource(ActiveCountyDossier(Some(next)));
        let context = app.world().resource::<ObserverSession>().context();
        app.world_mut()
            .resource_mut::<ObserverFocusPolicy>()
            .context = Some(context.clone());
        app.update();
        assert!(app.world().get::<TabIndex>(title).is_none());
        app.update();
        let target = app.world().get::<ObserverFocusTarget>(title).unwrap();
        assert_eq!(target.context.as_ref(), Some(&context));
        assert!(target.available);
        assert!(app.world().get::<TabIndex>(title).is_some());
    }

    #[test]
    fn pending_evidence_expansion_explicitly_waits_instead_of_claiming_empty_history() {
        let mut session = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::nil()));
        session.ready(1, Some("a".repeat(64)));
        let mut installed = installed(&session, &"a".repeat(64));
        let page = retained_page(&installed.read).unwrap().clone();
        installed.read.processed_tick = 99;
        installed.read.state = ArchiveDossierStateV2::Pending {
            page: Some(page),
            reason: ArchiveDossierPendingV2::ReceiptProcessing,
        };
        let rows = evidence_rows(&installed.read);
        assert_eq!(rows[0][0].text, "Changes await Archive completion.");
        let text: String = rows
            .iter()
            .flatten()
            .map(|segment| segment.text.as_str())
            .collect();
        assert!(!text.to_lowercase().contains("no changes"));
        assert!(text.contains("Exact retained narrative"));
    }

    #[test]
    fn retained_signal_labels_and_citations_are_not_reconstructed_from_atom_keys() {
        let signal = signal();
        let collapsed = retained_signal_segments(&signal, false);
        assert_eq!(collapsed[0].text, "Original jobs label: ");
        assert_eq!(collapsed[1].text, "1469 annual average jobs");
        let expanded = retained_signal_segments(&signal, true);
        assert!(expanded[2].text.contains("observed-source"));
        assert!(expanded[2].text.contains("exact/locator"));
    }

    #[test]
    fn rendered_signal_rows_stack_and_wrap_without_losing_citation_bytes() {
        let rows = vec![retained_signal_segments(&signal(), true)];
        let expected: String = rows[0].iter().map(|part| part.text.as_str()).collect();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Startup, move |mut commands: Commands| {
                let zone = commands.spawn((Text::new(""), DossierZone::Signals)).id();
                set_segment_rows(&mut commands, zone, &rows);
            });
        app.update();
        let world = app.world_mut();
        let (_, container, children) = world
            .query::<(&DossierZone, &Node, &Children)>()
            .single(world)
            .unwrap();
        assert_eq!(container.flex_direction, FlexDirection::Column);
        assert_eq!(container.width, percent(100));
        let row = children[0];
        let node = world.get::<Node>(row).unwrap();
        assert_eq!(node.min_width, px(0));
        assert_eq!(
            world.get::<TextLayout>(row).unwrap().linebreak,
            bevy::text::LineBreak::WordOrCharacter
        );
        let actual: String = world
            .get::<Children>(row)
            .unwrap()
            .iter()
            .map(|span| world.get::<TextSpan>(span).unwrap().0.as_str())
            .collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn campaign_default_uses_the_runtime_default_uuid_without_env() {
        let env = crate::test_support::EnvVarGuard::lock(CAMPAIGN_ENV);
        env.remove();
        assert_eq!(
            DossierCampaignId::default().0,
            CampaignId::from_uuid(uuid::Uuid::from_u128(DEFAULT_CAMPAIGN_UUID))
        );
    }
    #[test]
    fn campaign_default_reads_a_canonical_env_uuid() {
        let env = crate::test_support::EnvVarGuard::lock(CAMPAIGN_ENV);
        env.set("28100000-0000-0000-0000-000000000002");
        assert_eq!(
            DossierCampaignId::default().0,
            CampaignId::from_uuid(uuid::Uuid::from_u128(
                0x2810_0000_0000_0000_0000_0000_0000_0002
            ))
        );
    }
    #[test]
    fn fetch_error_display_is_honest() {
        assert_eq!(
            DossierFetchError::ReaderAbsent("missing env".into()).to_string(),
            "Archive reader not configured (missing env)"
        );
        assert!(DossierFetchError::ReadFailed("connection refused".into())
            .to_string()
            .starts_with("Archive fetch failed — "));
    }
}
