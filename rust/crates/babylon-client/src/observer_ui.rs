//! Native observer shell. All economic readings come from one installed frame.

use std::fmt::Write as _;

use babylon_persistence::ObserverEconomySnapshotV1;
use bevy::ecs::system::SystemParam;
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::atlas::CountyAtlas;
use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::map::{HoveredCounty, SelectedCounty};
use crate::map_economy_lens::{
    project_map_lens, CountyLensReading, EconomyMetric, MapLens, MaterialLensKind,
};
use crate::observer::ObserverSession;
use crate::observer_controls::{availability, turn_presentation, ControlAvailability};
use crate::observer_focus::{
    ObserverFocusPolicy, ObserverFocusSystems, ObserverFocusTarget, ObserverKeyboardActivate,
    ObserverKeyboardClaim,
};
use crate::observer_layout::{ObserverLayout, ObserverRegion};
use crate::observer_theme as theme;
use crate::ui::dossier_card::{ActiveCountyDossier, DossierFetchState, DossierRefresh};

pub(crate) const OBSERVER_PANEL_BOTTOM: f32 = 56.0;

#[derive(Resource, Default)]
pub struct ObserverFrame(pub Option<ObserverEconomySnapshotV1>);

impl ObserverFrame {
    /// Returns only the exact installed week and capability for this session.
    /// Async generation is checked before installation by the IO task.
    #[must_use]
    pub fn for_session(&self, session: &ObserverSession) -> Option<&ObserverEconomySnapshotV1> {
        self.0.as_ref().filter(|frame| {
            frame.campaign_id == session.campaign.as_uuid().to_string()
                && frame.resolve_tick == session.viewed_tick
                && matches!(
                    (session.perspective, frame.visibility),
                    (
                        crate::observer::Perspective::FullObserver,
                        babylon_persistence::ObserverVisibilityV1::FullObserver
                    ) | (
                        crate::observer::Perspective::PlayerKnowledge,
                        babylon_persistence::ObserverVisibilityV1::KnownPreview
                    )
                )
                && session
                    .foundation_digest
                    .as_ref()
                    .is_none_or(|digest| *digest == frame.foundation_digest)
                && (session.viewed_tick != session.durable_tick
                    || frame.tick_content_hash == session.content_hash)
        })
    }
}

// These are independent presentation preferences and disclosures, not transport phases.
#[allow(clippy::struct_excessive_bools)]
#[derive(Resource)]
pub struct ObserverUiState {
    pub lens: MapLens,
    pub archive_open: bool,
    pub reduced_motion: bool,
    pub menu_open: bool,
    pub splash_visible: bool,
    pub history_open: bool,
    pub stop_on_delivery: bool,
    pub comparison_open: bool,
    pub disclosure: Option<ObserverDisclosure>,
    pub evidence_open: bool,
    pub economic_details_open: bool,
}
impl Default for ObserverUiState {
    fn default() -> Self {
        Self {
            lens: MapLens::default(),
            archive_open: false,
            reduced_motion: false,
            menu_open: true,
            splash_visible: true,
            history_open: false,
            stop_on_delivery: false,
            comparison_open: false,
            disclosure: None,
            evidence_open: false,
            economic_details_open: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct ObserverViewport(pub Option<Rect>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverDisclosure {
    Time,
    Lens,
}

/// Short, non-sensitive explanations for declined controls.
#[derive(Resource, Default)]
pub(crate) struct ObserverFeedback {
    pub message: Option<&'static str>,
    pub revision: u64,
    pub expires_at: f64,
}
impl ObserverFeedback {
    pub(crate) fn reject(&mut self, reason: &'static str, now: f64) {
        self.message = Some(reason);
        self.revision = self.revision.wrapping_add(1);
        self.expires_at = now + 4.0;
    }
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverCommand {
    TogglePlay,
    Step,
    Speed,
    Perspective,
    PreviousWeek,
    NextWeek,
    Live,
    Lens(EconomyMetric),
    MaterialLens(MaterialLensKind),
    CycleGood(bool),
    Archive,
    Menu,
    NewCampaign,
    NewDelayedCampaign,
    ReopenCampaign,
    Quit,
    UiScale,
    ReducedMotion,
    MusicVolume,
    EffectsVolume,
    MusicTrack,
    History,
    StopOnDelivery,
    Disclosure(ObserverDisclosure),
    Evidence,
    EconomicDetails,
    Relationships,
}

#[derive(Component, Clone, Copy)]
struct ObserverButton {
    command: ObserverCommand,
    in_menu: bool,
}

#[derive(Component)]
struct ObserverButtonCaption(ObserverCommand);
#[derive(Component)]
struct ControlDrawer(ObserverDisclosure);
#[derive(Component)]
struct HistoryControls;
#[derive(Component)]
struct VerificationDetails;
#[derive(Component)]
struct ControlHint;

#[derive(Component, Clone, Copy)]
enum ObserverText {
    Clock,
    Identity,
    Status,
    Legend,
    County,
    Measures,
    Hover,
    Audio,
    Evidence,
    EvidenceDetails,
    Production,
    Developments,
    Source,
}

#[derive(Component)]
struct ObserverInspector;
#[derive(Component)]
struct EconomicReadings;
#[derive(Component)]
struct MapLensControls;
#[derive(Component)]
struct CircuitGuide;

#[derive(Component)]
pub struct ObserverMenu;
/// The saved-campaign browser occupies its own bounded menu column.
#[derive(Component)]
pub struct ObserverCampaignCatalog;

/// Full-window UI camera; the geography camera owns only the map viewport.
#[derive(Component)]
pub struct ObserverUiCamera;

/// Explicit presentation roles; the native font resource is required, not optional.
#[derive(Component, Clone, Copy)]
pub(crate) enum ObserverFontRole {
    Body,
    Display,
    Exact,
}

fn apply_fonts(
    fonts: Res<crate::visual_assets::ObserverFonts>,
    mut labels: Query<(&ObserverFontRole, &mut TextFont), Added<ObserverFontRole>>,
) {
    for (role, mut font) in &mut labels {
        font.font = match role {
            ObserverFontRole::Body => fonts.body.clone(),
            ObserverFontRole::Display => fonts.display.clone(),
            // Bevy's bundled Fira Mono remains the deliberate exact-value face.
            ObserverFontRole::Exact => Handle::default(),
        };
        if matches!(role, ObserverFontRole::Display) {
            font.weight = FontWeight::SEMIBOLD;
        }
    }
}

fn label(text: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    role_label(text, size, color, ObserverFontRole::Body)
}

fn role_label(
    text: impl Into<String>,
    size: f32,
    color: Color,
    role: ObserverFontRole,
) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
        TextLayout::new_with_linebreak(bevy::text::LineBreak::WordOrCharacter),
        role,
        DeclaredSurface::new(SurfaceId::ObserverShell),
    )
}

fn block_label(text: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        label(text, size, color),
        Node {
            min_width: px(0),
            max_width: percent(100),
            flex_shrink: 0.0,
            ..default()
        },
    )
}

/// Preserve every identity character while bounding the inspector's line width.
fn wrapped_identity(value: &str) -> String {
    let mut wrapped = String::with_capacity(value.len() + value.len() / 24);
    for (index, character) in value.chars().enumerate() {
        if index > 0 && index % 24 == 0 {
            wrapped.push('\n');
        }
        wrapped.push(character);
    }
    wrapped
}

fn button(parent: &mut ChildSpawnerCommands, text: &str, command: ObserverCommand) {
    scoped_button(parent, text, command, false);
}

fn scoped_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    command: ObserverCommand,
    in_menu: bool,
) {
    parent
        .spawn((
            Button,
            ObserverFocusTarget::action(None),
            Node {
                width: match command {
                    ObserverCommand::TogglePlay => px(200),
                    ObserverCommand::Step => px(174),
                    _ => Val::Auto,
                },
                justify_content: JustifyContent::Center,
                padding: UiRect::axes(px(12), px(8)),
                border: match command {
                    ObserverCommand::TogglePlay | ObserverCommand::Step => UiRect {
                        left: px(1),
                        right: px(1),
                        top: px(1),
                        bottom: px(3),
                    },
                    _ => UiRect::bottom(px(2)),
                },
                border_radius: BorderRadius::ZERO,
                flex_shrink: 0.0,
                min_width: px(0),
                ..default()
            },
            BackgroundColor(theme::PANEL),
            BorderColor::all(theme::GRAY),
            ObserverButton { command, in_menu },
            DeclaredSurface::new(SurfaceId::ObserverShell),
        ))
        .with_child((
            label(text, 15.0, theme::PAPER),
            ObserverButtonCaption(command),
        ));
}

fn row() -> Node {
    Node {
        column_gap: px(8),
        align_items: AlignItems::Center,
        min_width: px(0),
        ..default()
    }
}

fn spawn_hud(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(16),
                right: px(16),
                top: px(8),
                height: px(68),
                padding: UiRect::axes(px(8), px(4)),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                ..default()
            },
            BackgroundColor(theme::INK),
            ZIndex(8),
            TabGroup::new(0),
            DeclaredSurface::new(SurfaceId::ObserverShell),
        ))
        .with_children(|hud| {
            hud.spawn(row()).with_children(|bar| {
                bar.spawn(role_label(
                    "BABYLON",
                    26.0,
                    theme::PAPER,
                    ObserverFontRole::Display,
                ));
                bar.spawn((
                    label("Opening campaign", 16.0, theme::PAPER),
                    Node {
                        width: px(160),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    ObserverText::Clock,
                    ObserverFocusTarget::reading(None),
                ));
                bar.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
                button(bar, "Run month", ObserverCommand::TogglePlay);
                button(
                    bar,
                    "Time +",
                    ObserverCommand::Disclosure(ObserverDisclosure::Time),
                );
                bar.spawn(Node {
                    width: px(12),
                    ..default()
                });
                crate::production::button(
                    bar,
                    "Circuit [P]",
                    crate::production::ProductionCommand::Open,
                );
                crate::production::button(
                    bar,
                    "World [M]",
                    crate::production::ProductionCommand::Map,
                );
                button(bar, "Menu [Esc]", ObserverCommand::Menu);
            });
            hud.spawn(row()).with_children(|bar| {
                bar.spawn((label("", 14.0, theme::YELLOW), ObserverText::Status));
                bar.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
                bar.spawn((label("", 13.0, theme::GRAY), ObserverText::Identity));
            });
        });
}

fn spawn_inspector(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                padding: UiRect::all(px(18)),
                row_gap: px(18),
                border: UiRect::left(px(2)),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ObserverRegion::Context,
            BackgroundColor(theme::PANEL),
            BorderColor::all(theme::GRAY.with_alpha(0.35)),
            ZIndex(6),
            ObserverInspector,
            TabGroup::new(10),
            DeclaredSurface::new(SurfaceId::ObserverShell),
        ))
        .with_children(|panel| {
            panel.spawn((
                role_label(
                    "Select a county",
                    29.0,
                    theme::PAPER,
                    ObserverFontRole::Display,
                ),
                ObserverText::County,
            ));
            panel.spawn((
                block_label("", 17.0, theme::PAPER),
                ObserverText::Production,
                ObserverFocusTarget::reading(None),
            ));
            crate::production::button(
                panel,
                "Follow its work [P]",
                crate::production::ProductionCommand::Open,
            );
            panel.spawn((
                block_label("", 15.0, theme::GRAY),
                ObserverText::Developments,
            ));
            button(
                panel,
                "Economic readings +",
                ObserverCommand::EconomicDetails,
            );
            panel
                .spawn((context_column(), EconomicReadings))
                .with_children(|detail| {
                    detail.spawn((
                        role_label("", 14.0, theme::PAPER, ObserverFontRole::Exact),
                        ObserverText::Measures,
                        ObserverFocusTarget::reading(None),
                    ));
                    detail.spawn((block_label("", 14.0, theme::GRAY), ObserverText::Source));
                    button(
                        detail,
                        "Choose a map lens",
                        ObserverCommand::Disclosure(ObserverDisclosure::Lens),
                    );
                });
            button(
                panel,
                "Read the cited Archive [I]",
                ObserverCommand::Archive,
            );
            panel.spawn((block_label("", 13.0, theme::GRAY), ObserverText::Evidence));
        });
}

pub(crate) fn context_column() -> Node {
    Node {
        width: percent(100),
        flex_shrink: 0.0,
        min_width: px(0),
        flex_direction: FlexDirection::Column,
        row_gap: px(12),
        ..default()
    }
}

fn paint_economic_readings(
    ui: Res<ObserverUiState>,
    mut sections: Query<&mut Node, With<EconomicReadings>>,
) {
    for mut node in &mut sections {
        let display = if ui.economic_details_open {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
}

fn spawn_footer(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                padding: UiRect::axes(px(8), px(0)),
                column_gap: px(12),
                align_items: AlignItems::Center,
                overflow: Overflow::scroll_x(),
                ..default()
            },
            BackgroundColor(theme::INK),
            ObserverRegion::Footer,
            TabGroup::new(40),
            ZIndex(8),
            DeclaredSurface::new(SurfaceId::ObserverShell),
        ))
        .with_children(|bar| {
            button(bar, "Trends [H]", ObserverCommand::History);
            bar.spawn((row(), HistoryControls))
                .with_children(|controls| {
                    button(controls, "< Week", ObserverCommand::PreviousWeek);
                    button(controls, "Week >", ObserverCommand::NextWeek);
                    button(controls, "Return Live", ObserverCommand::Live);
                });
            bar.spawn((row(), MapLensControls))
                .with_children(|controls| {
                    button(
                        controls,
                        "Map lens +",
                        ObserverCommand::Disclosure(ObserverDisclosure::Lens),
                    );
                    button(controls, "< Good", ObserverCommand::CycleGood(true));
                    button(controls, "Good >", ObserverCommand::CycleGood(false));
                    controls.spawn((label("", 12.0, theme::PAPER), ObserverText::Legend));
                });
            bar.spawn((
                label(
                    "Tab: controls. World / Circuit: return to navigation.",
                    13.0,
                    theme::GRAY,
                ),
                CircuitGuide,
            ));
        });
}

fn spawn_drawers(commands: &mut Commands) {
    for disclosure in [ObserverDisclosure::Time, ObserverDisclosure::Lens] {
        let node = Node {
            position_type: PositionType::Absolute,
            left: px(16),
            width: px(720),
            top: if disclosure == ObserverDisclosure::Time {
                px(112)
            } else {
                Val::Auto
            },
            bottom: if disclosure == ObserverDisclosure::Lens {
                px(OBSERVER_PANEL_BOTTOM)
            } else {
                Val::Auto
            },
            padding: UiRect::all(px(16)),
            row_gap: px(12),
            flex_direction: FlexDirection::Column,
            border: UiRect::top(px(3)),
            display: Display::None,
            ..default()
        };
        commands.spawn((node, BackgroundColor(theme::INK), BorderColor::all(theme::YELLOW),
            ZIndex(12), TabGroup::new(50), ControlDrawer(disclosure), DeclaredSurface::new(SurfaceId::ObserverShell),
        )).with_children(|panel| {
            panel.spawn(row()).with_children(|bar| {
                bar.spawn(label(if disclosure == ObserverDisclosure::Time { "CAMPAIGN TIME" } else { "MAP LENS" }, 17.0, theme::YELLOW));
                bar.spawn(Node { flex_grow: 1.0, ..default() });
                button(bar, "Close", ObserverCommand::Disclosure(disclosure));
            });
            if disclosure == ObserverDisclosure::Time {
                panel.spawn(block_label(crate::observer_controls::MONTH_ADVANCE_HELP, 14.0, theme::PAPER));
                panel.spawn(row()).with_children(|bar| {
                    button(bar, "Advance one week", ObserverCommand::Step);
                    button(bar, "Speed", ObserverCommand::Speed);
                    button(bar, "Stop on delivery", ObserverCommand::StopOnDelivery);
                });
                panel.spawn(block_label("Space: run / pause month     Enter: advance one week     H: inspect trends", 12.0, theme::GRAY));
            } else {
                button(panel, "Relationships", ObserverCommand::Relationships);
                panel.spawn(row()).with_children(|bar| {
                    for (kind, title) in [(MaterialLensKind::ProducedThisWeek,"Production [5]"), (MaterialLensKind::OnHand,"Inventory [6]"), (MaterialLensKind::InboundInTransit,"Inbound [7]")] {
                        button(bar, title, ObserverCommand::MaterialLens(kind));
                    }
                });
                panel.spawn(block_label("Observed county context / BLS QCEW 2024", 12.0, theme::GRAY));
                panel.spawn(row()).with_children(|bar| {
                    for (metric, title) in EconomyMetric::ALL.into_iter().zip(["Jobs [1]","Payroll [2]","Weekly wage [3]","Establishments [4]"]) {
                        button(bar, title, ObserverCommand::Lens(metric));
                    }
                });
                panel.spawn(block_label(crate::observer_map3d::MAP_VIEW_HELP, 12.0, theme::GRAY));
            }
        });
    }
}

fn spawn_menu(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(12),
                right: percent(12),
                top: px(112),
                bottom: px(40),
                padding: UiRect::all(px(20)),
                row_gap: px(14),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                border: UiRect::all(px(2)),
                ..default()
            },
            BackgroundColor(theme::INK),
            BorderColor::all(theme::YELLOW),
            ZIndex(20),
            Visibility::Hidden,
            ObserverMenu,
            TabGroup::modal(),
            DeclaredSurface::new(SurfaceId::ObserverShell),
        ))
        .with_children(|panel| {
            panel.spawn(block_label("C A M P A I G N", 23.0, theme::YELLOW));
            panel
                .spawn(Node {
                    flex_grow: 1.0,
                    min_height: px(0),
                    min_width: px(0),
                    column_gap: px(24),
                    ..default()
                })
                .with_children(|columns| {
                    columns.spawn(menu_column()).with_children(menu_campaign);
                    columns.spawn(menu_column()).with_children(menu_settings);
                });
            panel.spawn(row()).with_children(|bar| {
                bar.spawn(label(
                    "Committed weeks are saved automatically.",
                    12.0,
                    theme::GRAY,
                ));
                bar.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
                scoped_button(bar, "Quit game [Q]", ObserverCommand::Quit, true);
            });
        });
}

fn menu_column() -> Node {
    Node {
        flex_grow: 1.0,
        flex_basis: px(0),
        min_width: px(0),
        min_height: px(0),
        flex_direction: FlexDirection::Column,
        row_gap: px(8),
        overflow: Overflow::scroll_y(),
        ..default()
    }
}

fn menu_campaign(panel: &mut ChildSpawnerCommands) {
    for (title, command) in [
        ("Continue [C]", ObserverCommand::Menu),
        (
            "Reopen committed campaign [R]",
            ObserverCommand::ReopenCampaign,
        ),
        ("New Michigan campaign [N]", ObserverCommand::NewCampaign),
        (
            "New delivery-delay scenario [D]",
            ObserverCommand::NewDelayedCampaign,
        ),
    ] {
        scoped_button(panel, title, command, true);
    }
    panel.spawn(block_label(
        "A new campaign preserves your existing world.",
        12.0,
        theme::GRAY,
    ));
    panel.spawn((
        Node {
            min_width: px(0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            margin: UiRect::top(px(10)),
            ..default()
        },
        ObserverCampaignCatalog,
    ));
}

fn menu_settings(panel: &mut ChildSpawnerCommands) {
    panel.spawn(block_label("PRESENTATION / SOUND", 17.0, theme::YELLOW));
    panel.spawn(block_label(
        "Tab / Shift+Tab: select controls. Enter: activate. Page Up / Down: read. WORLD / WORK: return to navigation.",
        12.0,
        theme::GRAY,
    ));
    for (title, command) in [
        ("Observer perspective [K]", ObserverCommand::Perspective),
        ("Interface size [U]", ObserverCommand::UiScale),
        ("Reduced motion [M]", ObserverCommand::ReducedMotion),
        ("Music volume / mute [B]", ObserverCommand::MusicVolume),
        ("Sound effects / mute [F]", ObserverCommand::EffectsVolume),
        ("Change theme [J]", ObserverCommand::MusicTrack),
    ] {
        scoped_button(panel, title, command, true);
    }
    panel.spawn((block_label("", 13.0, theme::YELLOW), ObserverText::Audio));
    scoped_button(
        panel,
        "Verification details +",
        ObserverCommand::Evidence,
        true,
    );
    panel.spawn((
        block_label("", 11.0, theme::GRAY),
        ObserverText::EvidenceDetails,
        VerificationDetails,
        ObserverFocusTarget::reading(None),
    ));
    panel.spawn(block_label(
        "OBSERVE / TRACE / COMPARE\nPlayer interventions are unavailable in observer mode.",
        12.0,
        theme::GRAY,
    ));
}

fn spawn_shell(
    mut commands: Commands,
    atlas: Res<CountyAtlas>,
    mut selected: ResMut<SelectedCounty>,
) {
    if selected.0.is_none() {
        selected.0 = (0..atlas.len()).find(|index| {
            atlas
                .county(*index)
                .is_some_and(|county| county.fips == "26163")
        });
    }
    commands.spawn((
        Camera2d,
        Camera {
            order: 10,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        bevy::camera::visibility::RenderLayers::layer(31),
        IsDefaultUiCamera,
        ObserverUiCamera,
    ));
    spawn_hud(&mut commands);
    spawn_inspector(&mut commands);
    spawn_footer(&mut commands);
    spawn_drawers(&mut commands);
    spawn_menu(&mut commands);
    commands.spawn((
        label("", 14.0, theme::PAPER),
        Node {
            position_type: PositionType::Absolute,
            left: px(30),
            top: px(120),
            ..default()
        },
        ZIndex(5),
        ObserverText::Hover,
    ));
    commands.spawn((
        label("", 13.0, theme::YELLOW),
        Node {
            position_type: PositionType::Absolute,
            left: px(32),
            bottom: px(84),
            padding: UiRect::all(px(8)),
            ..default()
        },
        BackgroundColor(theme::INK),
        ZIndex(30),
        ControlHint,
    ));
}

#[derive(EntityEvent)]
#[entity_event(propagate, auto_propagate)]
struct ScrollPanel {
    entity: Entity,
    delta: f32,
}

fn scroll_input(
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    hover: Res<bevy::picking::hover::HoverMap>,
    mut commands: Commands,
) {
    for event in wheel.read() {
        let delta = -event.y
            * if event.unit == bevy::input::mouse::MouseScrollUnit::Line {
                28.0
            } else {
                1.0
            };
        for map in hover.values() {
            for entity in map.keys() {
                commands.trigger(ScrollPanel {
                    entity: *entity,
                    delta,
                });
            }
        }
    }
}

fn scroll_panel(
    mut event: On<ScrollPanel>,
    mut nodes: Query<(&Node, &ComputedNode, &mut ScrollPosition)>,
) {
    let Ok((node, computed, mut position)) = nodes.get_mut(event.entity) else {
        return;
    };
    if node.overflow.y != OverflowAxis::Scroll {
        return;
    }
    let maximum = ((computed.content_size().y - computed.size().y)
        * computed.inverse_scale_factor())
    .max(0.0);
    let next = (position.y + event.delta).clamp(0.0, maximum);
    if next.to_bits() != position.y.to_bits() {
        position.y = next;
        event.propagate(false);
    }
}

fn pointer_buttons(
    buttons: Query<(&Interaction, &ObserverButton), Changed<Interaction>>,
    ui: Res<ObserverUiState>,
    session: Res<ObserverSession>,
    view: Res<crate::production::PrimaryView>,
    mut commands: MessageWriter<ObserverCommand>,
) {
    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed {
            dispatch_button(*button, &ui, &session, *view, &mut commands);
        }
    }
}

fn button_visible(
    button: ObserverButton,
    ui: &ObserverUiState,
    session: &ObserverSession,
    view: crate::production::PrimaryView,
) -> bool {
    if ui.splash_visible || ui.comparison_open || button.in_menu != ui.menu_open {
        return false;
    }
    if button.in_menu {
        return true;
    }
    let map = view == crate::production::PrimaryView::Map;
    match button.command {
        ObserverCommand::Step | ObserverCommand::Speed | ObserverCommand::StopOnDelivery => {
            ui.disclosure == Some(ObserverDisclosure::Time)
        }
        ObserverCommand::Relationships
        | ObserverCommand::Lens(_)
        | ObserverCommand::MaterialLens(_) => {
            map && ui.disclosure == Some(ObserverDisclosure::Lens)
        }
        ObserverCommand::CycleGood(_) => {
            map && !ui.history_open
                && session.viewed_tick == session.durable_tick
                && matches!(ui.lens, MapLens::Material { .. })
        }
        ObserverCommand::PreviousWeek | ObserverCommand::NextWeek | ObserverCommand::Live => {
            ui.history_open || session.viewed_tick < session.durable_tick
        }
        ObserverCommand::EconomicDetails | ObserverCommand::Archive => {
            map && !ui.archive_open && !ui.history_open
        }
        ObserverCommand::Disclosure(ObserverDisclosure::Lens) => map,
        _ => true,
    }
}

fn dispatch_button(
    button: ObserverButton,
    ui: &ObserverUiState,
    session: &ObserverSession,
    view: crate::production::PrimaryView,
    commands: &mut MessageWriter<ObserverCommand>,
) {
    if button_visible(button, ui, session, view) {
        // The canonical observer command handler still admits or refuses the request.
        commands.write(button.command);
    }
}

fn keyboard_button(
    event: On<ObserverKeyboardActivate>,
    buttons: Query<(&ObserverButton, &ObserverFocusTarget)>,
    ui: Res<ObserverUiState>,
    session: Res<ObserverSession>,
    view: Res<crate::production::PrimaryView>,
    mut commands: MessageWriter<ObserverCommand>,
) {
    let Ok((button, target)) = buttons.get(event.entity) else {
        return;
    };
    if event.context == target.context {
        dispatch_button(*button, &ui, &session, *view, &mut commands);
    }
}

#[derive(SystemParam)]
struct ShellFocusState<'w> {
    ui: Res<'w, ObserverUiState>,
    session: Res<'w, ObserverSession>,
    view: Res<'w, crate::production::PrimaryView>,
    frame: Res<'w, ObserverFrame>,
}

fn sync_focus_policy(
    state: ShellFocusState,
    menus: Query<Entity, With<ObserverMenu>>,
    warnings: Query<Entity, With<crate::observer_warning::ObserverWarningRoot>>,
    comparisons: Query<Entity, With<crate::campaign_browser::ComparisonPanel>>,
    mut policy: ResMut<ObserverFocusPolicy>,
) {
    policy.set_if_neq(ObserverFocusPolicy {
        context: Some(state.session.context()),
        modal: if state.ui.splash_visible {
            warnings.single().ok()
        } else if state.ui.comparison_open {
            comparisons.single().ok()
        } else if state.ui.menu_open && !state.ui.splash_visible {
            menus.single().ok()
        } else {
            None
        },
    });
}

fn sync_focus_targets(
    state: ShellFocusState,
    mut targets: Query<(
        &mut ObserverFocusTarget,
        Option<&ObserverButton>,
        Option<&ObserverText>,
    )>,
) {
    for (mut target, button, reading) in &mut targets {
        let mut next = target.clone();
        if let Some(button) = button {
            next.available = button_visible(*button, &state.ui, &state.session, *state.view)
                && availability(button.command, &state.session) == ControlAvailability::Enabled;
        } else if let Some(reading) = reading {
            let scope_visible = !state.ui.splash_visible && !state.ui.comparison_open;
            next.available = scope_visible
                && match reading {
                    ObserverText::Clock => !state.ui.menu_open,
                    ObserverText::EvidenceDetails => state.ui.menu_open && state.ui.evidence_open,
                    ObserverText::Production | ObserverText::Measures => {
                        inspector_visibility(&state.ui, *state.view) == Visibility::Visible
                            && (!matches!(reading, ObserverText::Measures)
                                || state.ui.economic_details_open)
                            && state.frame.for_session(&state.session).is_some()
                    }
                    _ => false,
                };
            next.context = Some(state.session.context());
        } else {
            continue;
        }
        target.set_if_neq(next);
    }
}

type PaintedButtons<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static ObserverButton,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
        &'static mut Node,
    ),
    Without<ControlHint>,
>;
type HintText<'w, 's> = Query<
    'w,
    's,
    (&'static mut Text, &'static mut Node),
    (
        With<ControlHint>,
        Without<ObserverButtonCaption>,
        Without<ObserverButton>,
    ),
>;

#[derive(SystemParam)]
struct ButtonPaint<'w, 's> {
    buttons: PaintedButtons<'w, 's>,
    captions: Query<
        'w,
        's,
        (
            &'static ObserverButtonCaption,
            &'static mut Text,
            &'static mut TextColor,
        ),
        Without<ControlHint>,
    >,
    hints: HintText<'w, 's>,
}

fn caption(
    command: ObserverCommand,
    state: &ObserverSession,
    ui: &ObserverUiState,
) -> Option<String> {
    Some(match command {
        ObserverCommand::TogglePlay => format!("{} [Space]", turn_presentation(state).play_label),
        ObserverCommand::Step => turn_presentation(state).step_label,
        ObserverCommand::Quit => if state.quit_requested {
            "Closing game..."
        } else {
            "Quit game [Q]"
        }
        .to_owned(),
        ObserverCommand::Speed => format!("Speed: {} week(s) / sec", state.weeks_per_second),
        ObserverCommand::StopOnDelivery => format!(
            "Stop on delivery: {}",
            if ui.stop_on_delivery { "ON" } else { "OFF" }
        ),
        ObserverCommand::History => {
            format!("History {} [H]", if ui.history_open { "-" } else { "+" })
        }
        ObserverCommand::Perspective => state.perspective.label().to_owned(),
        ObserverCommand::EconomicDetails => format!(
            "Economic readings {}",
            if ui.economic_details_open { "-" } else { "+" }
        ),
        ObserverCommand::Evidence => format!(
            "Verification details {}",
            if ui.evidence_open { "-" } else { "+" }
        ),
        _ => return None,
    })
}

fn paint_buttons(
    ui: Res<ObserverUiState>,
    state: Res<ObserverSession>,
    view: Res<crate::production::PrimaryView>,
    feedback: Res<ObserverFeedback>,
    mut paint: ButtonPaint,
) {
    let mut hint = feedback.message;
    for (interaction, button, mut background, mut border, mut node) in &mut paint.buttons {
        if matches!(button.command, ObserverCommand::CycleGood(_)) {
            let display = if *view == crate::production::PrimaryView::Map
                && matches!(&ui.lens, MapLens::Material { .. })
            {
                Display::Flex
            } else {
                Display::None
            };
            if node.display != display {
                node.display = display;
            }
        }
        let available = availability(button.command, &state);
        let active_scope =
            !ui.splash_visible && !ui.comparison_open && button.in_menu == ui.menu_open;
        if active_scope && *interaction != Interaction::None {
            if let ControlAvailability::Disabled(reason) = available {
                hint = Some(reason);
            }
        }
        let selected = match (button.command, &ui.lens) {
            (ObserverCommand::Relationships, MapLens::Relationships) => true,
            (ObserverCommand::EconomicDetails, _) => ui.economic_details_open,
            (ObserverCommand::Lens(metric), MapLens::Qcew(current)) => metric == *current,
            (ObserverCommand::MaterialLens(kind), MapLens::Material { kind: current, .. }) => {
                kind == *current
            }
            (ObserverCommand::Disclosure(value), _) => ui.disclosure == Some(value),
            (ObserverCommand::History, _) => ui.history_open,
            (ObserverCommand::Archive, _) => ui.archive_open,
            _ => false,
        };
        let enabled = active_scope && available == ControlAvailability::Enabled;
        let next = match interaction {
            _ if !enabled => theme::INK,
            Interaction::Pressed => theme::RED.with_alpha(0.5),
            Interaction::Hovered => theme::YELLOW.with_alpha(0.25),
            Interaction::None if selected => theme::YELLOW.with_alpha(0.2),
            Interaction::None => theme::PANEL,
        };
        background.set_if_neq(BackgroundColor(next));
        border.set_if_neq(BorderColor::all(if !enabled {
            theme::GRAY.with_alpha(0.3)
        } else if selected {
            theme::YELLOW
        } else {
            theme::GRAY
        }));
    }
    for (marker, mut text, mut color) in &mut paint.captions {
        if state.is_changed() || ui.is_changed() || text.is_added() {
            if let Some(value) = caption(marker.0, &state, &ui) {
                text.set_if_neq(Text::new(value));
            }
        }
        color.set_if_neq(TextColor(
            if availability(marker.0, &state) == ControlAvailability::Enabled {
                theme::PAPER
            } else {
                theme::GRAY.with_alpha(0.65)
            },
        ));
    }
    for (mut text, mut node) in &mut paint.hints {
        text.set_if_neq(Text::new(hint.unwrap_or_default()));
        let display = if hint.is_some() && !ui.splash_visible {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
}

type DisclosureRows<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Node,
        Option<&'static MapLensControls>,
        Option<&'static CircuitGuide>,
        Option<&'static ControlDrawer>,
        Option<&'static HistoryControls>,
        Option<&'static VerificationDetails>,
    ),
    Or<(
        With<MapLensControls>,
        With<CircuitGuide>,
        With<ControlDrawer>,
        With<HistoryControls>,
        With<VerificationDetails>,
    )>,
>;

fn paint_view_controls(
    view: Res<crate::production::PrimaryView>,
    ui: Res<ObserverUiState>,
    state: Res<ObserverSession>,
    mut rows: DisclosureRows,
) {
    if !(view.is_changed() || ui.is_changed() || state.is_changed()) {
        return;
    }
    for (mut node, map, circuit, drawer, history, evidence) in &mut rows {
        let visible = if let Some(drawer) = drawer {
            ui.disclosure == Some(drawer.0)
                && !ui.menu_open
                && !ui.splash_visible
                && !ui.comparison_open
                && (drawer.0 != ObserverDisclosure::Lens
                    || *view == crate::production::PrimaryView::Map)
        } else if map.is_some() {
            *view == crate::production::PrimaryView::Map
                && !ui.history_open
                && state.viewed_tick == state.durable_tick
        } else if circuit.is_some() {
            *view == crate::production::PrimaryView::Production
                && !ui.history_open
                && state.viewed_tick == state.durable_tick
        } else if history.is_some() {
            ui.history_open || state.viewed_tick < state.durable_tick
        } else {
            evidence.is_some() && ui.evidence_open
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn expire_feedback(time: Res<Time>, mut feedback: ResMut<ObserverFeedback>) {
    if feedback.message.is_some() && time.elapsed_secs_f64() >= feedback.expires_at {
        feedback.message = None;
    }
}

fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    claimed: Res<ObserverKeyboardClaim>,
    ui: Res<ObserverUiState>,
    view: Res<crate::production::PrimaryView>,
    atlas: Res<CountyAtlas>,
    mut selected: ResMut<SelectedCounty>,
    mut commands: MessageWriter<ObserverCommand>,
) {
    if ui.splash_visible || ui.comparison_open {
        return;
    }
    if !ui.menu_open && keys.just_pressed(KeyCode::Escape) {
        commands.write(
            ui.disclosure
                .map_or(ObserverCommand::Menu, ObserverCommand::Disclosure),
        );
        return;
    }
    if ui.menu_open {
        for (key, command) in [
            (KeyCode::Escape, ObserverCommand::Menu),
            (KeyCode::KeyC, ObserverCommand::Menu),
            (KeyCode::KeyN, ObserverCommand::NewCampaign),
            (KeyCode::KeyR, ObserverCommand::ReopenCampaign),
            (KeyCode::KeyQ, ObserverCommand::Quit),
            (KeyCode::KeyD, ObserverCommand::NewDelayedCampaign),
            (KeyCode::KeyU, ObserverCommand::UiScale),
            (KeyCode::KeyM, ObserverCommand::ReducedMotion),
            (KeyCode::KeyB, ObserverCommand::MusicVolume),
            (KeyCode::KeyF, ObserverCommand::EffectsVolume),
            (KeyCode::KeyJ, ObserverCommand::MusicTrack),
            (KeyCode::KeyE, ObserverCommand::StopOnDelivery),
            (KeyCode::KeyK, ObserverCommand::Perspective),
        ] {
            if keys.just_pressed(key) && !claimed.claimed(key) {
                commands.write(command);
            }
        }
        return;
    }
    if claimed.blocks_world_shortcuts() {
        return;
    }
    for (key, command) in [
        (KeyCode::Space, ObserverCommand::TogglePlay),
        (KeyCode::Enter, ObserverCommand::Step),
        (KeyCode::BracketLeft, ObserverCommand::PreviousWeek),
        (KeyCode::BracketRight, ObserverCommand::NextWeek),
        (KeyCode::KeyK, ObserverCommand::Perspective),
        (KeyCode::KeyI, ObserverCommand::Archive),
        (KeyCode::KeyT, ObserverCommand::Speed),
        (KeyCode::KeyH, ObserverCommand::History),
        (
            KeyCode::Digit5,
            ObserverCommand::MaterialLens(MaterialLensKind::ProducedThisWeek),
        ),
        (
            KeyCode::Digit6,
            ObserverCommand::MaterialLens(MaterialLensKind::OnHand),
        ),
        (
            KeyCode::Digit7,
            ObserverCommand::MaterialLens(MaterialLensKind::InboundInTransit),
        ),
        (KeyCode::Comma, ObserverCommand::CycleGood(true)),
        (KeyCode::Period, ObserverCommand::CycleGood(false)),
        (
            KeyCode::Digit1,
            ObserverCommand::Lens(EconomyMetric::Employment),
        ),
        (
            KeyCode::Digit2,
            ObserverCommand::Lens(EconomyMetric::Payroll),
        ),
        (
            KeyCode::Digit3,
            ObserverCommand::Lens(EconomyMetric::WeeklyWage),
        ),
        (
            KeyCode::Digit4,
            ObserverCommand::Lens(EconomyMetric::Establishments),
        ),
    ] {
        if keys.just_pressed(key) {
            commands.write(command);
        }
    }
    if *view == crate::production::PrimaryView::Map
        && (keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::ArrowRight))
    {
        if let Some(next) =
            adjacent_county(&atlas, selected.0, keys.just_pressed(KeyCode::ArrowRight))
        {
            selected.0 = Some(next);
        }
    }
}

fn adjacent_county(atlas: &CountyAtlas, selected: Option<usize>, forward: bool) -> Option<usize> {
    let counties: Vec<usize> = (0..atlas.len())
        .filter(|index| {
            atlas
                .county(*index)
                .is_some_and(|county| county.fips.starts_with("26"))
        })
        .collect();
    if counties.is_empty() {
        return None;
    }
    let current = counties.iter().position(|index| Some(*index) == selected);
    let next = current.map_or(0, |index| {
        if forward {
            (index + 1) % counties.len()
        } else {
            (index + counties.len() - 1) % counties.len()
        }
    });
    Some(counties[next])
}

#[must_use]
pub fn grouped(value: u64) -> String {
    let raw = value.to_string();
    let mut text = String::with_capacity(raw.len() + raw.len() / 3);
    for (index, byte) in raw.bytes().enumerate() {
        if index > 0 && (raw.len() - index).is_multiple_of(3) {
            text.push(',');
        }
        text.push(char::from(byte));
    }
    text
}

#[must_use]
pub fn format_lens_reading(reading: CountyLensReading, unit: &str) -> String {
    match reading {
        CountyLensReading::Available(value) => format!("{} {unit}", grouped(value)),
        CountyLensReading::Unavailable(reason) => reason.label().to_owned(),
    }
}

fn local_relationships(
    snapshot: &babylon_persistence::ProductionSnapshotV1,
    county: Option<&str>,
) -> String {
    let Some(county) = county else {
        return "Select a county to follow its work and dependencies.".into();
    };
    let mut sites: Vec<_> = snapshot
        .sites
        .iter()
        .filter(|site| site.county_geoid == county)
        .collect();
    sites.sort_by(|a, b| a.id.cmp(&b.id));
    if sites.is_empty() {
        return "No production relationships are modeled here yet. The Archive contains the observed county context.".into();
    }
    let mut lines = Vec::new();
    for site in sites {
        let relations = crate::production_brief::dependency_sites(site, snapshot);
        if relations.is_empty() {
            lines.push(format!("{} / no disclosed supply relationships", site.name));
        }
        for (direction, other) in relations {
            lines.push(match direction {
                crate::production_brief::DependencyDirection::Upstream => {
                    format!("{} depends on {}", site.name, other.name)
                }
                crate::production_brief::DependencyDirection::Downstream => {
                    format!("{} supplies {}", site.name, other.name)
                }
            });
        }
    }
    format!(
        "SUPPLY RELATIONSHIPS\n{}\n\nDesigned county cohorts; schematic links.",
        lines.join("\n\n")
    )
}

fn county_developments(snapshot: Option<&ObserverEconomySnapshotV1>, county: &str) -> String {
    let Some(snapshot) = snapshot else {
        return "Awaiting this week's observation.".into();
    };
    let Some(production) = snapshot.production.as_ref() else {
        return "Material developments are not disclosed in this observation.".into();
    };
    if snapshot.resolve_tick == 0 {
        return "Opening week. No completed production week yet.".into();
    }
    let records = production
        .events
        .iter()
        .filter(|event| {
            event.week == snapshot.resolve_tick
                && event.subject_site_ids.iter().any(|id| {
                    production
                        .sites
                        .iter()
                        .any(|site| site.id == *id && site.county_geoid == county)
                })
        })
        .count();
    format!("Week {}: {records} committed records for this county. Open History to inspect the evidence.", snapshot.resolve_tick)
}

#[derive(SystemParam)]
struct ShellState<'w> {
    state: Res<'w, ObserverSession>,
    frame: Res<'w, ObserverFrame>,
    ui: Res<'w, ObserverUiState>,
    view: Res<'w, crate::production::PrimaryView>,
    audio: Res<'w, crate::observer_audio::ObserverAudioSettings>,
    atlas: Res<'w, CountyAtlas>,
    selected: Res<'w, SelectedCounty>,
    hovered: Res<'w, HoveredCounty>,
    dossier: Option<Res<'w, ActiveCountyDossier>>,
    dossier_refresh: Option<Res<'w, DossierRefresh>>,
    dossier_fetch: Option<Res<'w, DossierFetchState>>,
}

impl ShellState<'_> {
    fn needs_repaint(&self) -> bool {
        self.state.is_changed()
            || self.frame.is_changed()
            || self.ui.is_changed()
            || self.selected.is_changed()
            || self.hovered.is_changed()
            || self.view.is_changed()
            || self.audio.is_changed()
            || self.dossier.as_ref().is_some_and(DetectChanges::is_changed)
            || self
                .dossier_refresh
                .as_ref()
                .is_some_and(DetectChanges::is_changed)
            || self
                .dossier_fetch
                .as_ref()
                .is_some_and(DetectChanges::is_changed)
    }
}

fn archive_page_status(
    read: Option<&babylon_persistence::archive_revision::ArchiveDossierReadV2>,
    selected: bool,
    read_failed: bool,
) -> String {
    use babylon_persistence::archive_revision::ArchiveDossierStateV2;
    match read.map(|read| &read.state) {
        Some(ArchiveDossierStateV2::Ready {
            verified_through_tick,
            ..
        }) => format!("Archive verified for week {verified_through_tick}"),
        Some(ArchiveDossierStateV2::Pending { .. }) => "Archive pending".into(),
        Some(ArchiveDossierStateV2::Unavailable(_)) => "Archive unavailable".into(),
        None if read_failed => "Archive read failed".into(),
        None if selected => "Archive awaiting page".into(),
        None => "Archive: select a county".into(),
    }
}

fn repaint(
    shell: ShellState,
    mut texts: Query<(&ObserverText, &mut Text)>,
    mut menus: Query<&mut Visibility, With<ObserverMenu>>,
    mut inspectors: Query<&mut Visibility, (With<ObserverInspector>, Without<ObserverMenu>)>,
) {
    if !shell.needs_repaint() {
        return;
    }
    let ShellState {
        state,
        frame,
        ui,
        view,
        audio,
        atlas,
        selected,
        hovered,
        dossier,
        dossier_refresh,
        dossier_fetch,
    } = shell;
    let county = selected.0.and_then(|index| atlas.county(index));
    let installed = frame.for_session(&state);
    let archive = dossier.as_ref().and_then(|dossier| {
        dossier.for_observer(
            &state,
            &frame,
            dossier_refresh.as_ref()?.0,
            county.as_ref()?.fips,
        )
    });
    let archive_status = archive_page_status(
        archive,
        county.is_some(),
        dossier_fetch
            .as_deref()
            .is_some_and(|fetch| matches!(fetch, DossierFetchState::Failed(_))),
    );
    let archive_detail: &str = match archive {
        Some(read) => crate::dossier::availability_label(read),
        None => &archive_status,
    };
    let lens = project_map_lens(installed, &ui.lens);
    let turn = turn_presentation(&state);
    for (kind, mut text) in &mut texts {
        if matches!(kind, ObserverText::EvidenceDetails) {
            if !ui.evidence_open {
                text.0.clear();
                continue;
            }
            if !(frame.is_changed() || state.is_changed() || ui.is_changed()) {
                continue;
            }
        }
        let value = match kind {
            ObserverText::Clock => turn.period.clone(),
            ObserverText::Identity => format!("{} | {}", state.perspective.label(), archive_status),
            ObserverText::Status => turn.status.clone(),
            ObserverText::Legend => match &ui.lens {
                MapLens::Relationships => String::new(),
                MapLens::Qcew(_) => format!("0..{} {}", lens.maximum().map_or_else(|| "-".into(), grouped), lens.unit),
                MapLens::Material {..} => lens.good_label.as_ref().map_or_else(|| "Material unavailable".into(), |good|format!("{good} ({})", lens.unit)),
            },
            ObserverText::County => county.as_ref().map_or_else(|| "Select a county".into(), |county| county.name.to_owned()),
            ObserverText::Measures if matches!(ui.lens, MapLens::Relationships) => "Choose an economic lens to inspect exact county readings.".into(),
            ObserverText::Measures => county.as_ref().map_or_else(|| "Select a county to inspect this lens.".into(), |county| format!("{}\n{}", lens.label, format_lens_reading(lens.county(county.fips), &lens.unit))),
            ObserverText::Hover if *view != crate::production::PrimaryView::Map => String::new(),
            ObserverText::Hover if matches!(ui.lens, MapLens::Relationships) => hovered.0.and_then(|index| atlas.county(index)).map_or_else(String::new, |county| county.name.to_owned()),
            ObserverText::Hover => hovered.0.and_then(|index| atlas.county(index)).filter(|county| county.fips.starts_with("26")).map_or_else(String::new, |county| format!("{}\n{}\n{}", county.name, lens.label, format_lens_reading(lens.county(county.fips), &lens.unit))),
            ObserverText::Audio => format!("{} | music {:.0}% | effects {:.0}%\nReduced motion: {} | Stop on delivery: {}", if audio.track==0 {"PHI"}else{"PANOPTICON"},audio.music_volume*100.0,audio.effects_volume*100.0,if ui.reduced_motion {"ON"}else{"OFF"},if ui.stop_on_delivery {"ON"}else{"OFF"}),
            ObserverText::Evidence => format!("Viewing week {} / Archive processed through {}\n{}", state.viewed_tick, state.archive_verified_tick, archive_detail),
            ObserverText::EvidenceDetails => installed.map_or_else(String::new, |snapshot| {
                let mut evidence = format!("CAMPAIGN\n{}\n\nCOMMITTED EVIDENCE / WEEK {}\n{}\n\nWORLD IDENTITY\n{}", wrapped_identity(&snapshot.campaign_id), snapshot.resolve_tick, wrapped_identity(snapshot.tick_content_hash.as_deref().unwrap_or(&snapshot.foundation_digest)), snapshot.nominal_world_hash.as_deref().map_or_else(|| "Unavailable in this observation".to_owned(), wrapped_identity));
                if let Some(digest) = snapshot.production_evidence_digest() {
                    let _ = write!(evidence, "\n\nPRODUCTION OBSERVATION\n{}", wrapped_identity(&digest.to_hex()));
                }
                evidence
            }),
            ObserverText::Source => match &ui.lens {
                MapLens::Relationships => "County geography anchors aggregates. Supply links are schematic; they do not locate factories or physical routes.".into(),
                MapLens::Qcew(_) => "OBSERVED | BLS QCEW | 2024 annual\nJobs are annual averages; weekly wages are means. Monetary values are dollars, not physical output.".into(),
                MapLens::Material {kind, ..} => format!("{}\n{}\nZero is a measured account; unavailable and unmodeled counties have no numeric reading.", lens.evidence, match kind {MaterialLensKind::ProducedThisWeek => "Output in the selected week; foundation has no production receipt.",MaterialLensKind::OnHand => "Stock held at the end of the selected week. Terminal goods remain unsold on hand.",MaterialLensKind::InboundInTransit => "Actual lots destined for this county, counted once. This is not traffic passing through the county."}),
            },
            ObserverText::Developments => county.as_ref().map_or_else(String::new, |county| county_developments(installed, county.fips)),
            ObserverText::Production => installed.and_then(|snapshot| snapshot.production.as_ref())
                .map_or_else(|| "Production relationships are unavailable in this observation.".into(), |production|
                    local_relationships(production, county.as_ref().map(|county| county.fips))),
        };
        text.set_if_neq(Text::new(value));
    }
    for mut visibility in &mut menus {
        visibility.set_if_neq(if ui.menu_open && !ui.comparison_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
    for mut visibility in &mut inspectors {
        visibility.set_if_neq(inspector_visibility(&ui, *view));
    }
}

fn inspector_visibility(ui: &ObserverUiState, view: crate::production::PrimaryView) -> Visibility {
    if ui.archive_open
        || ui.history_open
        || ui.menu_open
        || ui.splash_visible
        || ui.comparison_open
        || view == crate::production::PrimaryView::Production
    {
        Visibility::Hidden
    } else {
        Visibility::Visible
    }
}

fn fit_viewport(
    windows: Query<&Window, With<PrimaryWindow>>,
    scale: Res<UiScale>,
    ui: Res<ObserverUiState>,
    mut viewport: ResMut<ObserverViewport>,
    mut regions: Query<(&ObserverRegion, &mut Node)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let layout = ObserverLayout::new(
        Vec2::new(window.width(), window.height()),
        scale.0,
        ui.history_open,
    );
    for (region, mut node) in &mut regions {
        let rect = layout.region(*region);
        let (left, top, width, height) = (
            px(rect.min.x),
            px(rect.min.y),
            px(rect.width()),
            px(rect.height()),
        );
        if node.left != left || node.top != top || node.width != width || node.height != height {
            node.left = left;
            node.top = top;
            node.width = width;
            node.height = height;
        }
    }
    let rect = Rect::from_corners(layout.world.min * scale.0, layout.world.max * scale.0);
    if viewport.0 != Some(rect) {
        viewport.0 = Some(rect);
    }
}

fn reconcile_lens(
    frame: Res<ObserverFrame>,
    session: Res<ObserverSession>,
    mut ui: ResMut<ObserverUiState>,
    mut previous_scope: Local<
        Option<(
            babylon_persistence::CampaignId,
            crate::observer::Perspective,
        )>,
    >,
) {
    let scope = (session.campaign, session.perspective);
    let changed = previous_scope.as_ref() != Some(&scope);
    if !(changed || frame.is_changed() || session.is_changed() || ui.is_changed()) {
        return;
    }
    let mut lens = ui.lens.clone();
    lens.reconcile(frame.for_session(&session), changed);
    if ui.lens != lens {
        ui.lens = lens;
    }
    if changed {
        *previous_scope = Some(scope);
    }
}

pub struct ObserverShellPlugin;
impl Plugin for ObserverShellPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::observer_warning::ObserverWarningPlugin)
            .init_resource::<ObserverFrame>()
            .init_resource::<ObserverUiState>()
            .init_resource::<ObserverViewport>()
            .init_resource::<ObserverFeedback>()
            .init_resource::<ObserverKeyboardClaim>()
            .add_message::<ObserverCommand>()
            .add_systems(Startup, spawn_shell.after(crate::map::spawn_map_surface))
            .add_systems(PostUpdate, apply_fonts.before(bevy::ui::UiSystems::Content))
            .add_systems(
                Update,
                paint_economic_readings.in_set(crate::observer_io::ObserverSet::Paint),
            )
            .add_systems(
                Update,
                (pointer_buttons, keyboard, scroll_input, expire_feedback)
                    .in_set(crate::observer_io::ObserverSet::Input),
            )
            .add_observer(scroll_panel)
            .add_observer(keyboard_button)
            .add_systems(
                PreUpdate,
                (sync_focus_policy, sync_focus_targets)
                    .in_set(ObserverFocusSystems::Eligibility)
                    .run_if(resource_exists::<ObserverFocusPolicy>),
            )
            .add_systems(
                Update,
                reconcile_lens
                    .after(crate::observer_io::ObserverSet::Install)
                    .before(crate::observer_io::ObserverSet::Paint),
            )
            .add_systems(
                Update,
                (fit_viewport, repaint, paint_buttons, paint_view_controls)
                    .in_set(crate::observer_io::ObserverSet::Paint),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observer_focus::ObserverFocusPlugin;
    use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
    use bevy::input::{ButtonState, InputPlugin};
    use bevy::input_focus::InputFocus;

    #[test]
    fn explicit_font_roles_use_distinct_native_faces_and_preserve_exact_mono() {
        let handles = Assets::<Font>::default();
        let fonts = crate::visual_assets::ObserverFonts {
            body: handles.reserve_handle(),
            display: handles.reserve_handle(),
        };
        let mut app = App::new();
        app.insert_resource(fonts.clone())
            .add_systems(PostUpdate, apply_fonts.before(bevy::ui::UiSystems::Content));
        let body = app
            .world_mut()
            .spawn((TextFont::default(), ObserverFontRole::Body))
            .id();
        let display = app
            .world_mut()
            .spawn((TextFont::default(), ObserverFontRole::Display))
            .id();
        let exact = app
            .world_mut()
            .spawn((
                TextFont {
                    weight: FontWeight::BOLD,
                    ..default()
                },
                ObserverFontRole::Exact,
            ))
            .id();
        app.update();
        assert_eq!(app.world().get::<TextFont>(body).unwrap().font, fonts.body);
        assert_eq!(
            app.world().get::<TextFont>(display).unwrap().font,
            fonts.display
        );
        assert_eq!(
            app.world().get::<TextFont>(display).unwrap().weight,
            FontWeight::SEMIBOLD
        );
        assert_eq!(
            app.world().get::<TextFont>(exact).unwrap().font,
            Handle::<Font>::default()
        );
        assert_eq!(
            app.world().get::<TextFont>(exact).unwrap().weight,
            FontWeight::BOLD,
            "choosing an exact-value face must preserve evidence emphasis"
        );
    }

    #[test]
    fn economic_readings_start_collapsed_and_reveal_without_rebuilding_subject() {
        let mut app = App::new();
        app.init_resource::<ObserverUiState>()
            .add_systems(Startup, |mut commands: Commands| {
                spawn_inspector(&mut commands);
            })
            .add_systems(Update, paint_economic_readings);
        app.update();
        let world = app.world_mut();
        let subject = world
            .query_filtered::<Entity, With<ObserverInspector>>()
            .single(world)
            .unwrap();
        let readings = world
            .query_filtered::<Entity, With<EconomicReadings>>()
            .single(world)
            .unwrap();
        assert!(matches!(
            world.get::<ObserverRegion>(subject),
            Some(ObserverRegion::Context)
        ));
        assert_eq!(
            world.get::<Node>(subject).unwrap().flex_direction,
            FlexDirection::Column
        );
        assert_eq!(world.get::<Node>(readings).unwrap().display, Display::None);
        world
            .resource_mut::<ObserverUiState>()
            .economic_details_open = true;
        app.update();
        assert_eq!(
            app.world().get::<Node>(readings).unwrap().display,
            Display::Flex
        );
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .economic_details_open = false;
        app.update();
        assert_eq!(
            app.world().get::<Node>(readings).unwrap().display,
            Display::None
        );
        assert!(app.world().get::<ObserverInspector>(subject).is_some());
    }

    #[test]
    fn selected_context_controls_yield_to_history_archive_and_warning() {
        let session = ObserverSession::new(babylon_persistence::CampaignId::from_uuid(
            uuid::Uuid::from_u128(1),
        ));
        for hidden in 0..3 {
            let ui = ObserverUiState {
                menu_open: false,
                splash_visible: hidden == 0,
                archive_open: hidden == 1,
                history_open: hidden == 2,
                ..default()
            };
            assert_eq!(
                inspector_visibility(&ui, crate::production::PrimaryView::Map),
                Visibility::Hidden
            );
            assert!(!button_visible(
                ObserverButton {
                    command: ObserverCommand::EconomicDetails,
                    in_menu: false
                },
                &ui,
                &session,
                crate::production::PrimaryView::Map
            ));
            assert!(!button_visible(
                ObserverButton {
                    command: ObserverCommand::Archive,
                    in_menu: false
                },
                &ui,
                &session,
                crate::production::PrimaryView::Map
            ));
        }
    }

    #[test]
    fn archive_hud_certifies_only_the_scoped_ready_page() {
        use babylon_persistence::archive_revision::{
            ArchiveChangePageV2, ArchiveDossierPageV2, ArchiveDossierPendingV2,
            ArchiveDossierReadV2, ArchiveDossierStateV2, ArchiveDossierUnavailableV2,
            ArchivePublicationOriginV2, ArchiveReadScopeV2,
        };
        use babylon_persistence::{ArchivePageRefV1, ArchiveSubjectKindV1, CampaignId};
        let campaign = CampaignId::from_uuid(uuid::Uuid::from_u128(1));
        let scope = ArchiveReadScopeV2::committed(campaign, 3, [3; 32]).unwrap();
        let mut read = ArchiveDossierReadV2 {
            scope,
            subject: ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, "26163".into())
                .unwrap(),
            durable_tick: 16,
            processed_tick: 16,
            history_floor_tick: 0,
            state: ArchiveDossierStateV2::Pending {
                page: None,
                reason: ArchiveDossierPendingV2::CutoverValidation,
            },
        };
        assert_eq!(
            archive_page_status(Some(&read), true, false),
            "Archive pending"
        );
        read.state =
            ArchiveDossierStateV2::Unavailable(ArchiveDossierUnavailableV2::HistoryNotRetained);
        assert_eq!(
            archive_page_status(Some(&read), true, false),
            "Archive unavailable"
        );
        read.state = ArchiveDossierStateV2::Ready {
            verified_through_tick: 3,
            page: ArchiveDossierPageV2 {
                revision_id: [1; 32],
                effective_tick: 1,
                origin: ArchivePublicationOriginV2::Materialized,
                content_source: ArchiveReadScopeV2::committed(campaign, 1, [1; 32]).unwrap(),
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
        };
        assert_eq!(
            archive_page_status(Some(&read), true, false),
            "Archive verified for week 3"
        );
        assert_eq!(
            archive_page_status(None, true, false),
            "Archive awaiting page"
        );
        assert_eq!(archive_page_status(None, true, true), "Archive read failed");
    }

    fn focused_shell(menu_open: bool, command: ObserverCommand) -> (App, Entity, Entity) {
        use crate::observer_io::ObserverSet;
        let mut session = ObserverSession::new(babylon_persistence::CampaignId::from_uuid(
            uuid::Uuid::from_u128(1),
        ));
        session.phase = crate::observer::SessionPhase::Ready;
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            InputPlugin,
            ObserverFocusPlugin,
            crate::campaign_browser::CampaignBrowserPlugin,
        ))
        .insert_resource(session)
        .insert_resource(
            CountyAtlas::parse(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../assets/map/county_atlas.bin"
            )))
            .unwrap(),
        )
        .insert_resource(ObserverUiState {
            menu_open,
            splash_visible: false,
            disclosure: Some(ObserverDisclosure::Time),
            ..default()
        })
        .init_resource::<ObserverFrame>()
        .init_resource::<crate::production::PrimaryView>()
        .init_resource::<SelectedCounty>()
        .add_message::<ObserverCommand>()
        .configure_sets(Update, ObserverSet::Install.run_if(|| false))
        .configure_sets(Update, ObserverSet::Paint.run_if(|| false))
        .add_systems(
            PreUpdate,
            (sync_focus_policy, sync_focus_targets).in_set(ObserverFocusSystems::Eligibility),
        )
        .add_systems(
            Update,
            (keyboard, pointer_buttons).in_set(ObserverSet::Input),
        )
        .add_observer(keyboard_button);
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        let root = app
            .world_mut()
            .spawn((
                Node::default(),
                if menu_open {
                    TabGroup::modal()
                } else {
                    TabGroup::new(0)
                },
            ))
            .id();
        if menu_open {
            app.world_mut().entity_mut(root).insert(ObserverMenu);
        }
        let target = app
            .world_mut()
            .spawn((
                Button,
                Node::default(),
                ObserverButton {
                    command,
                    in_menu: menu_open,
                },
                ObserverFocusTarget::action(None),
            ))
            .id();
        app.world_mut().entity_mut(root).add_child(target);
        app.update();
        app.world_mut().resource_mut::<InputFocus>().0 = Some(target);
        (app, window, target)
    }

    fn focus_key(app: &mut App, window: Entity, key_code: KeyCode) {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window,
        });
        app.update();
    }

    #[test]
    fn focused_menu_enter_does_not_also_open_a_saved_campaign_or_advance() {
        let (mut app, window, target) = focused_shell(true, ObserverCommand::Menu);
        assert!(
            app.world()
                .get::<ObserverFocusTarget>(target)
                .unwrap()
                .available
        );
        focus_key(&mut app, window, KeyCode::Enter);
        assert_eq!(
            app.world_mut()
                .resource_mut::<Messages<ObserverCommand>>()
                .drain()
                .collect::<Vec<_>>(),
            [ObserverCommand::Menu]
        );
        assert!(app
            .world()
            .resource::<Messages<crate::campaign_browser::CampaignBrowserCommand>>()
            .is_empty());
        assert_eq!(app.world().resource::<ObserverSession>().durable_tick, 0);
    }

    #[test]
    fn focused_drawer_space_only_activates_its_control() {
        let command = ObserverCommand::Disclosure(ObserverDisclosure::Time);
        let (mut app, window, _) = focused_shell(false, command);
        focus_key(&mut app, window, KeyCode::Space);
        assert_eq!(
            app.world_mut()
                .resource_mut::<Messages<ObserverCommand>>()
                .drain()
                .collect::<Vec<_>>(),
            [command]
        );
        assert!(!app.world().resource::<ObserverSession>().playing);
    }

    #[test]
    fn dismissed_time_drawer_refuses_both_late_pointer_and_keyboard_requests() {
        let (mut app, _, target) = focused_shell(false, ObserverCommand::Step);
        app.world_mut().resource_mut::<ObserverUiState>().disclosure = None;
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity: target,
            context: None,
        });
        app.world_mut()
            .entity_mut(target)
            .insert(Interaction::Pressed);
        app.update();
        assert!(app
            .world()
            .resource::<Messages<ObserverCommand>>()
            .is_empty());
        assert!(
            !app.world()
                .get::<ObserverFocusTarget>(target)
                .unwrap()
                .available
        );
    }

    #[test]
    fn focused_control_arrows_preserve_selected_county() {
        let (mut app, window, _) = focused_shell(false, ObserverCommand::History);
        app.world_mut().resource_mut::<SelectedCounty>().0 = Some(0);
        focus_key(&mut app, window, KeyCode::ArrowRight);
        assert_eq!(app.world().resource::<SelectedCounty>().0, Some(0));
    }

    #[test]
    fn shell_startup_waits_for_the_map_atlas_before_selecting_detroit() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
            .insert_resource(ObserverSession::new(
                babylon_persistence::CampaignId::from_uuid(uuid::Uuid::nil()),
            ))
            .add_plugins((crate::map::MapPlugin, ObserverShellPlugin));
        app.finish();
        app.cleanup();
        app.world_mut().run_schedule(Startup);
        let atlas = app.world().resource::<CountyAtlas>();
        let selected = app.world().resource::<SelectedCounty>().0;
        assert_eq!(
            selected
                .and_then(|index| atlas.county(index))
                .map(|county| county.fips),
            Some("26163")
        );
    }

    #[derive(Resource, Default)]
    struct ChangedButtonNodes(Vec<Entity>);

    fn button_paint_fixture() -> (App, [Entity; 3]) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ObserverUiState {
                menu_open: false,
                splash_visible: false,
                ..default()
            })
            .insert_resource(ObserverSession::new(
                babylon_persistence::CampaignId::from_uuid(uuid::Uuid::from_u128(1)),
            ))
            .insert_resource(crate::production::PrimaryView::Map)
            .init_resource::<ObserverFeedback>()
            .init_resource::<ChangedButtonNodes>()
            .add_systems(
                Update,
                (
                    paint_buttons,
                    |nodes: Query<Entity, Changed<Node>>,
                     mut changed: ResMut<ChangedButtonNodes>| {
                        changed.0 = nodes.iter().collect();
                    },
                )
                    .chain(),
            );
        let buttons = [false, true].map(|previous| {
            app.world_mut()
                .spawn((
                    Interaction::None,
                    ObserverButton {
                        command: ObserverCommand::CycleGood(previous),
                        in_menu: false,
                    },
                    BackgroundColor::default(),
                    BorderColor::default(),
                    Node::default(),
                ))
                .id()
        });
        let hint = app
            .world_mut()
            .spawn((ControlHint, Text::default(), Node::default()))
            .id();
        (app, [buttons[0], buttons[1], hint])
    }

    #[test]
    fn unchanged_button_and_hint_visibility_does_not_invalidate_layout() {
        let (mut app, nodes) = button_paint_fixture();
        app.update();
        assert_eq!(app.world().resource::<ChangedButtonNodes>().0.len(), 3);
        app.update();
        assert!(app.world().resource::<ChangedButtonNodes>().0.is_empty());

        for (lens, message, display) in [
            (
                MapLens::Material {
                    kind: MaterialLensKind::OnHand,
                    good: None,
                },
                Some("Wait for the current week to finish."),
                Display::Flex,
            ),
            (MapLens::default(), None, Display::None),
        ] {
            app.world_mut().resource_mut::<ObserverUiState>().lens = lens;
            app.world_mut().resource_mut::<ObserverFeedback>().message = message;
            app.update();
            let changes = &app.world().resource::<ChangedButtonNodes>().0;
            assert_eq!(changes.len(), nodes.len());
            for node in nodes {
                assert!(changes.contains(&node));
                assert_eq!(app.world().get::<Node>(node).unwrap().display, display);
            }
            app.update();
            assert!(app.world().resource::<ChangedButtonNodes>().0.is_empty());
        }

        // Updating a visible explanation changes its text without dirtying its layout node.
        app.world_mut().resource_mut::<ObserverFeedback>().message = Some("Loading week.");
        app.update();
        assert_eq!(app.world().resource::<ChangedButtonNodes>().0, [nodes[2]]);
        app.world_mut().resource_mut::<ObserverFeedback>().message = Some("Finishing week.");
        app.update();
        assert!(app.world().resource::<ChangedButtonNodes>().0.is_empty());
        assert_eq!(
            app.world().get::<Text>(nodes[2]).unwrap().0,
            "Finishing week."
        );
    }

    #[test]
    fn menu_clicks_do_not_activate_world_controls_behind_it() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ObserverSession::new(
                babylon_persistence::CampaignId::from_uuid(uuid::Uuid::nil()),
            ))
            .init_resource::<crate::production::PrimaryView>()
            .insert_resource(ObserverUiState {
                splash_visible: false,
                ..default()
            })
            .init_resource::<ObserverKeyboardClaim>()
            .add_message::<ObserverCommand>()
            .add_systems(Update, pointer_buttons);
        app.world_mut().spawn((
            Interaction::Pressed,
            ObserverButton {
                command: ObserverCommand::Step,
                in_menu: false,
            },
        ));
        let menu = app
            .world_mut()
            .spawn((
                Interaction::Pressed,
                ObserverButton {
                    command: ObserverCommand::Menu,
                    in_menu: true,
                },
            ))
            .id();
        app.update();
        let received: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<ObserverCommand>>()
            .drain()
            .collect();
        assert_eq!(received, [ObserverCommand::Menu]);
        app.world_mut().resource_mut::<ObserverUiState>().menu_open = false;
        app.world_mut()
            .entity_mut(menu)
            .insert(Interaction::Hovered);
        app.update();
        app.world_mut()
            .entity_mut(menu)
            .insert(Interaction::Pressed);
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<Messages<ObserverCommand>>()
                .drain()
                .count(),
            0
        );
    }

    #[test]
    fn advanced_controls_start_collapsed_and_only_requested_drawer_is_visible() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ObserverUiState {
                menu_open: false,
                splash_visible: false,
                ..default()
            })
            .insert_resource(crate::production::PrimaryView::Map)
            .insert_resource(ObserverSession::new(
                babylon_persistence::CampaignId::from_uuid(uuid::Uuid::from_u128(1)),
            ))
            .add_systems(Startup, |mut commands: Commands| {
                spawn_drawers(&mut commands);
                spawn_menu(&mut commands);
                spawn_footer(&mut commands);
            })
            .add_systems(Update, paint_view_controls);
        app.update();
        let world = app.world_mut();
        assert!(world
            .query_filtered::<&Node, With<ControlDrawer>>()
            .iter(world)
            .all(|node| node.display == Display::None));
        assert!(world
            .query_filtered::<&Node, With<VerificationDetails>>()
            .iter(world)
            .all(|node| node.display == Display::None));
        world.resource_mut::<ObserverUiState>().disclosure = Some(ObserverDisclosure::Lens);
        app.update();
        let world = app.world_mut();
        for (drawer, node) in world.query::<(&ControlDrawer, &Node)>().iter(world) {
            assert_eq!(
                node.display == Display::Flex,
                drawer.0 == ObserverDisclosure::Lens
            );
        }
        // A modal menu suppresses an open drawer and its buttons as one subtree.
        world.resource_mut::<ObserverUiState>().menu_open = true;
        app.update();
        let world = app.world_mut();
        assert!(world
            .query_filtered::<&Node, With<ControlDrawer>>()
            .iter(world)
            .all(|node| node.display == Display::None));
    }

    #[test]
    fn comparison_and_perspective_shortcuts_dispatch_independently_across_input_systems() {
        use crate::campaign_browser::{CampaignBrowserCommand, CampaignBrowserPlugin};
        use crate::observer_io::ObserverSet;

        let atlas = CountyAtlas::parse(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/map/county_atlas.bin"
        )))
        .expect("atlas");
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, InputPlugin, CampaignBrowserPlugin))
            .insert_resource(atlas)
            .init_resource::<SelectedCounty>()
            .init_resource::<crate::production::PrimaryView>()
            .init_resource::<ObserverUiState>()
            .init_resource::<ObserverKeyboardClaim>()
            .add_message::<ObserverCommand>()
            // Exercise the actual input systems without launching catalog reads or applying
            // either request. In particular, a second consumer must not hide a collision.
            .configure_sets(Update, ObserverSet::Install.run_if(|| false))
            .configure_sets(Update, ObserverSet::Paint.run_if(|| false))
            .add_systems(Update, keyboard.in_set(ObserverSet::Input));
        app.world_mut().spawn(ObserverCampaignCatalog);

        for (menu_open, splash_visible, comparison_open) in [
            (true, false, false),
            (false, false, false),
            (true, true, false),
            (false, false, true),
        ] {
            *app.world_mut().resource_mut::<ObserverUiState>() = ObserverUiState {
                menu_open,
                splash_visible,
                comparison_open,
                ..default()
            };
            for key_code in [KeyCode::KeyK, KeyCode::KeyX] {
                app.world_mut()
                    .resource_mut::<Messages<KeyboardInput>>()
                    .write(KeyboardInput {
                        key_code,
                        logical_key: Key::Unidentified(NativeKey::Unidentified),
                        state: ButtonState::Pressed,
                        text: None,
                        repeat: false,
                        window: Entity::PLACEHOLDER,
                    });
                app.update();
                let observer: Vec<_> = app
                    .world_mut()
                    .resource_mut::<Messages<ObserverCommand>>()
                    .drain()
                    .collect();
                let browser: Vec<_> = app
                    .world_mut()
                    .resource_mut::<Messages<CampaignBrowserCommand>>()
                    .drain()
                    .collect();
                let unobscured = !splash_visible && !comparison_open;
                assert_eq!(
                    observer,
                    if unobscured && key_code == KeyCode::KeyK {
                        vec![ObserverCommand::Perspective]
                    } else {
                        Vec::new()
                    }
                );
                if unobscured && menu_open && key_code == KeyCode::KeyX {
                    assert!(matches!(
                        browser.as_slice(),
                        [CampaignBrowserCommand::Compare]
                    ));
                } else {
                    assert!(
                        browser.is_empty(),
                        "unexpected browser command: {browser:?}"
                    );
                }
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .reset_all();
            }
        }
        let world = app.world_mut();
        let labels: Vec<_> = world
            .query::<&Text>()
            .iter(world)
            .map(|text| &text.0)
            .collect();
        assert!(labels.iter().any(|text| text.as_str() == "Compare  [X]"));
        assert!(!labels.iter().any(|text| text.as_str() == "Compare  [K]"));
    }

    #[test]
    fn quit_shortcut_is_only_available_inside_the_campaign_menu() {
        let atlas = CountyAtlas::parse(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/map/county_atlas.bin"
        )))
        .expect("atlas");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(atlas)
            .init_resource::<SelectedCounty>()
            .init_resource::<crate::production::PrimaryView>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(ObserverUiState {
                splash_visible: false,
                menu_open: false,
                ..default()
            })
            .init_resource::<ObserverKeyboardClaim>()
            .add_message::<ObserverCommand>()
            .add_systems(Update, keyboard);
        for menu_open in [false, true] {
            app.world_mut().resource_mut::<ObserverUiState>().menu_open = menu_open;
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyQ);
            app.update();
            let commands: Vec<_> = app
                .world_mut()
                .resource_mut::<Messages<ObserverCommand>>()
                .drain()
                .collect();
            assert_eq!(
                commands,
                if menu_open {
                    vec![ObserverCommand::Quit]
                } else {
                    Vec::new()
                }
            );
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
        }
    }

    #[test]
    fn county_arrow_shortcuts_only_change_selection_on_geography() {
        let atlas = CountyAtlas::parse(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/map/county_atlas.bin"
        )))
        .expect("atlas");
        let county = (0..atlas.len())
            .find(|index| atlas.county(*index).is_some_and(|row| row.fips == "26099"))
            .expect("Macomb");
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, InputPlugin))
            .insert_resource(atlas)
            .insert_resource(SelectedCounty(Some(county)))
            .insert_resource(crate::production::PrimaryView::Production)
            .insert_resource(ObserverUiState {
                menu_open: false,
                splash_visible: false,
                ..default()
            })
            .init_resource::<ObserverKeyboardClaim>()
            .add_message::<ObserverCommand>()
            .add_systems(Update, keyboard);
        for view in [
            crate::production::PrimaryView::Production,
            crate::production::PrimaryView::Map,
        ] {
            *app.world_mut()
                .resource_mut::<crate::production::PrimaryView>() = view;
            for key in [KeyCode::ArrowLeft, KeyCode::ArrowRight] {
                app.world_mut().resource_mut::<SelectedCounty>().0 = Some(county);
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
                app.update();
                let after = app.world().resource::<SelectedCounty>().0;
                if view == crate::production::PrimaryView::Production {
                    assert_eq!(after, Some(county));
                } else {
                    assert_ne!(after, Some(county));
                }
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .release(key);
            }
        }
    }
}
