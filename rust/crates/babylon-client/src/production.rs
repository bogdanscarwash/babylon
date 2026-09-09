//! A read-only production scene: exact receipts, cohort columns and actual lots.
//! County aggregates have no invented geographic placement.

use std::fmt::Write as _;

use babylon_persistence::{ProductionSiteV2, ProductionSnapshotV2};
use bevy::camera::{visibility::RenderLayers, ScalingMode, Viewport};
use bevy::ecs::system::SystemParam;
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::light::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::atlas::CountyAtlas;
use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::map::SelectedCounty;
use crate::observer::{ObservationContext, ObserverSession};
use crate::observer_focus::{
    ObserverFocusSystems, ObserverFocusTarget, ObserverFocusWorld, ObserverKeyboardActivate,
    ObserverKeyboardClaim,
};
use crate::observer_io::ObserverSet;
use crate::observer_theme as theme;
use crate::observer_ui::{
    grouped, ObserverFeedback, ObserverFrame, ObserverUiState, ObserverViewport,
};
use crate::production_brief::{
    committed_plan_status, dependency_flow_summary, dependency_sites, describe_brief,
    describe_overview, opening_site, DependencyDirection,
};
use crate::production_freight::{
    account_brief, account_reading, competitor_sites, format_freight_mass, shared_accounts,
};
use crate::production_layout::{path_point, place_label, relation_path, ProductionLayout};

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PrimaryView {
    #[default]
    Map,
    Production,
}

#[derive(Resource, Default)]
pub struct ProductionNavigation {
    pub selected_site: Option<String>,
    pub selected_process: Option<String>,
    pub(crate) county_geoid: Option<String>,
    pub(crate) county_open: bool,
    cohort_page: usize,
    relationship_page: usize,
    competitor_page: usize,
    pub flat: bool,
    pub details_open: bool,
    pub(crate) reading_section: ProductionReadingSection,
    history: Vec<String>,
}

impl ProductionNavigation {
    fn open_county(&mut self, county_geoid: &str, snapshot: Option<&ProductionSnapshotV2>) {
        self.county_geoid = Some(county_geoid.to_owned());
        let resume = snapshot.is_some_and(|snapshot| {
            snapshot.sites.iter().any(|site| {
                self.selected_site.as_ref() == Some(&site.id) && site.county_geoid == county_geoid
            })
        });
        if !resume {
            self.county_open = true;
            self.cohort_page = 0;
            self.selected_site = None;
            self.history.clear();
        }
    }

    fn select_site(&mut self, id: &str) {
        if let Some(previous) = self.selected_site.take() {
            if previous != id {
                if self.history.len() == 128 {
                    self.history.remove(0);
                }
                self.history.push(previous);
            }
        }
        self.selected_site = Some(id.to_owned());
        self.county_open = false;
        self.relationship_page = 0;
        self.competitor_page = 0;
        self.selected_process = None;
    }

    pub(crate) fn process<'a>(
        &self,
        site: &'a ProductionSiteV2,
    ) -> Option<&'a babylon_persistence::ProductionProcessV2> {
        site.processes
            .iter()
            .find(|process| self.selected_process.as_ref() == Some(&process.id))
            .or_else(|| site.processes.iter().min_by(|a, b| a.id.cmp(&b.id)))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ProductionPage {
    Cohorts,
    Relationships,
    Competitors,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProductionReadingSection {
    #[default]
    Flow,
    Freight,
    Work,
    Sources,
}

impl ProductionReadingSection {
    fn label(self) -> &'static str {
        match self {
            Self::Flow => "Flow",
            Self::Freight => "Freight",
            Self::Work => "Work",
            Self::Sources => "Sources",
        }
    }
}

#[derive(Resource)]
struct ProductionOrbit {
    yaw: f32,
    pitch: f32,
    distance: f32,
}
impl Default for ProductionOrbit {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.8,
            distance: 1000.0,
        }
    }
}

#[derive(Component)]
pub struct ProductionCamera;
#[derive(Component)]
struct ProductionGeometry;
#[derive(Component)]
struct ProductionLabel {
    anchor: Vec3,
    site_id: String,
    selected: bool,
    leader: Entity,
}
#[derive(Component)]
struct ProductionLeader;
#[derive(Component)]
struct ProductionPanel;
#[derive(Component)]
struct ProductionDetailGroup;
#[derive(Component)]
struct ProductionReadingBody;
#[derive(Component)]
struct ProductionDisclosureLabel;
#[derive(Component)]
struct ProductionDetails;
#[derive(Component)]
struct ProductionReadingSubject;
#[derive(Component)]
struct ProductionReadingHeadline;
#[derive(Component)]
struct ProductionBrief;
#[derive(Component)]
struct ProductionDependencies;
#[derive(Component)]
pub(crate) struct ProductionCountyCohorts;
#[derive(Component)]
struct ProductionFreightReading;
#[derive(Component, Clone)]
struct ProductionButton(ProductionCommand);

#[derive(Message, Clone)]
pub enum ProductionCommand {
    Open,
    Map,
    Flat,
    Details,
    Reading(ProductionReadingSection),
    Page {
        kind: ProductionPage,
        page: usize,
        context: ObservationContext,
    },
    Process {
        process_id: String,
        context: ObservationContext,
    },
    Back,
    Select {
        site_id: String,
        context: ObservationContext,
    },
}

#[derive(SystemParam)]
struct ProductionObservation<'w> {
    frame: Res<'w, ObserverFrame>,
    state: Res<'w, ObserverSession>,
}

#[derive(SystemParam)]
struct ProductionUi<'w> {
    state: ResMut<'w, ObserverUiState>,
    feedback: ResMut<'w, ObserverFeedback>,
    time: Res<'w, Time>,
}

struct ProductionControlAvailability {
    scene: bool,
    previous_index: Option<usize>,
    county_back: bool,
}

impl ProductionControlAvailability {
    fn for_snapshot(
        snapshot: Option<&ProductionSnapshotV2>,
        navigation: &ProductionNavigation,
    ) -> Self {
        Self {
            scene: snapshot.is_some_and(|snapshot| !snapshot.sites.is_empty()),
            county_back: navigation.selected_site.is_some()
                && navigation.county_geoid.is_some()
                && !navigation.county_open,
            previous_index: snapshot.and_then(|snapshot| {
                navigation.history.iter().rposition(|id| {
                    navigation.selected_site.as_ref() != Some(id)
                        && snapshot.sites.iter().any(|site| site.id == *id)
                })
            }),
        }
    }

    fn display(&self, command: &ProductionCommand) -> Option<Display> {
        let available = match command {
            ProductionCommand::Back => self.previous_index.is_some() || self.county_back,
            ProductionCommand::Details
            | ProductionCommand::Flat
            | ProductionCommand::Reading(_) => self.scene,
            _ => return None,
        };
        Some(if available {
            Display::Flex
        } else {
            Display::None
        })
    }

    fn refusal(
        &self,
        command: &ProductionCommand,
        snapshot: Option<&ProductionSnapshotV2>,
        navigation: &ProductionNavigation,
        state: &ObserverSession,
    ) -> Option<&'static str> {
        match command {
            ProductionCommand::Back if self.previous_index.is_none() && !self.county_back => {
                Some("There is no previous work view in this observation.")
            }
            ProductionCommand::Flat if !self.scene => {
                Some("Display controls need disclosed production relationships.")
            }
            ProductionCommand::Details if !self.scene && !navigation.details_open => {
                Some("Exact readings need disclosed production relationships.")
            }
            ProductionCommand::Reading(_) if !self.scene => {
                Some("Exact readings need disclosed production relationships.")
            }
            ProductionCommand::Page { context, .. }
            | ProductionCommand::Process { context, .. }
                if !state.accepts(context) || !self.scene =>
            {
                Some("This circuit control belongs to another observation.")
            }
            ProductionCommand::Process { process_id, .. }
                if !snapshot.is_some_and(|snapshot| {
                    snapshot.sites.iter().any(|site| {
                        navigation.selected_site.as_ref() == Some(&site.id)
                            && site
                                .processes
                                .iter()
                                .any(|process| process.id == *process_id)
                    })
                }) =>
            {
                Some("This process is not disclosed for the selected owner.")
            }
            ProductionCommand::Page { kind, page, .. }
                if !snapshot.is_some_and(|snapshot| {
                    let count = match kind {
                        ProductionPage::Cohorts => snapshot
                            .sites
                            .iter()
                            .filter(|site| {
                                navigation.county_geoid.as_ref() == Some(&site.county_geoid)
                            })
                            .count(),
                        ProductionPage::Relationships => navigation
                            .selected_site
                            .as_ref()
                            .and_then(|id| snapshot.sites.iter().find(|site| site.id == *id))
                            .map_or(0, |site| {
                                dependency_sites(site, snapshot)
                                    .iter()
                                    .map(|(_, site)| &site.id)
                                    .collect::<std::collections::BTreeSet<_>>()
                                    .len()
                            }),
                        ProductionPage::Competitors => navigation
                            .selected_site
                            .as_deref()
                            .map_or(0, |id| competitor_sites(id, snapshot).len()),
                    };
                    *page < count.div_ceil(6).max(1)
                }) =>
            {
                Some("This page is unavailable in the current observation.")
            }
            ProductionCommand::Select { site_id, context }
                if !state.accepts(context)
                    || !snapshot.is_some_and(|snapshot| {
                        snapshot.sites.iter().any(|site| site.id == *site_id)
                    }) =>
            {
                Some("This work relationship is unavailable in the current observation.")
            }
            _ => None,
        }
    }
}

/// A disclosed inspector temporarily occupies the log's shared side panel.
pub(crate) fn readings_panel_visible(
    view: PrimaryView,
    navigation: &ProductionNavigation,
    ui: &ObserverUiState,
    snapshot: Option<&ProductionSnapshotV2>,
) -> bool {
    view == PrimaryView::Production
        && navigation.details_open
        && !ui.history_open
        && ProductionControlAvailability::for_snapshot(snapshot, navigation).scene
        && !ui.archive_open
        && !ui.menu_open
        && !ui.comparison_open
        && !ui.splash_visible
}

#[derive(SystemParam)]
struct ProductionPointer<'w, 's> {
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    buttons: Res<'w, ButtonInput<MouseButton>>,
    motion: MessageReader<'w, 's, bevy::input::mouse::MouseMotion>,
    wheel: MessageReader<'w, 's, bevy::input::mouse::MouseWheel>,
    interactions: Query<'w, 's, &'static Interaction, With<Button>>,
}

type SceneGeometry = Or<(
    With<ProductionGeometry>,
    With<ProductionLabel>,
    With<ProductionLeader>,
)>;
type ReadingMarkers = Or<(
    With<ProductionDetails>,
    With<ProductionBrief>,
    With<ProductionReadingSubject>,
    With<ProductionReadingHeadline>,
)>;
type ReadingText = (
    &'static mut Text,
    Option<&'static ProductionBrief>,
    Option<&'static ProductionReadingSubject>,
    Option<&'static ProductionReadingHeadline>,
);
type PanelParts = (&'static mut Visibility, &'static mut Node);
type CameraParts = (
    &'static mut Camera,
    &'static mut Transform,
    &'static mut Projection,
);
type LabelParts = (
    Entity,
    &'static ProductionLabel,
    &'static ComputedNode,
    &'static mut Node,
    &'static mut Visibility,
);
type LeaderParts = (
    &'static mut Node,
    &'static mut UiTransform,
    &'static mut Visibility,
);
type LabelFilter = (Without<ProductionPanel>, Without<ProductionLeader>);
type LeaderFilter = (
    With<ProductionLeader>,
    Without<ProductionPanel>,
    Without<ProductionLabel>,
);
type ButtonVisuals = (
    &'static ProductionButton,
    &'static Interaction,
    &'static mut BackgroundColor,
    &'static mut BorderColor,
);

#[derive(SystemParam)]
struct ProductionScene<'w, 's> {
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    camera: Query<'w, 's, CameraParts, With<ProductionCamera>>,
    panels: Query<'w, 's, PanelParts, With<ProductionPanel>>,
}

#[derive(SystemParam)]
struct ProductionLabels<'w, 's> {
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    camera: Query<'w, 's, (&'static Camera, &'static Transform), With<ProductionCamera>>,
    labels: Query<'w, 's, LabelParts, LabelFilter>,
    leaders: Query<'w, 's, LeaderParts, LeaderFilter>,
}

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
        crate::observer_ui::ObserverFontRole::Body,
        DeclaredSurface::new(SurfaceId::ObserverProduction),
    )
}

fn button_node(command: ProductionCommand) -> impl Bundle {
    let context = production_command_context(&command).cloned();
    (
        Button,
        ProductionButton(command),
        ObserverFocusTarget::action(context),
        Node {
            padding: UiRect::axes(px(10), px(8)),
            border: UiRect::bottom(px(2)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(theme::PANEL),
        BorderColor::all(theme::PAPER),
        DeclaredSurface::new(SurfaceId::ObserverProduction),
    )
}

pub(crate) fn button(parent: &mut ChildSpawnerCommands, value: &str, command: ProductionCommand) {
    parent
        .spawn(button_node(command))
        .with_child(text(value, 15.0, theme::PAPER));
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            is_active: false,
            ..default()
        },
        Projection::Perspective(PerspectiveProjection::default()),
        Transform::from_xyz(0.0, 850.0, 1100.0).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::layer(1),
        ProductionCamera,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 9000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-500.0, 900.0, 400.0).looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 900.0,
            maximum_distance: 3300.0,
            ..default()
        }
        .build(),
        RenderLayers::layer(1),
    ));
    commands.spawn((
        PointLight {
            intensity: 4_000_000.0,
            color: theme::BLUE,
            range: 1800.0,
            ..default()
        },
        Transform::from_xyz(450.0, 400.0, -350.0),
        RenderLayers::layer(1),
    ));
    setup_panel(&mut commands);
}

fn setup_panel(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                padding: UiRect::all(px(16)),
                flex_direction: FlexDirection::Column,
                row_gap: px(20),
                border: UiRect::left(px(2)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(theme::PANEL),
            BorderColor::all(theme::PAPER),
            ZIndex(7),
            Visibility::Hidden,
            ProductionPanel,
            TabGroup::new(10),
            crate::observer_layout::ObserverRegion::Context,
            DeclaredSurface::new(SurfaceId::ObserverProduction),
        ))
        .with_children(panel_contents);
    setup_readings_panel(commands);
}

fn panel_contents(panel: &mut ChildSpawnerCommands) {
    panel
        .spawn(crate::observer_ui::context_column())
        .with_children(|panel| {
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    row_gap: px(8),
                    flex_shrink: 0.0,
                    min_width: px(0),
                    ..default()
                })
                .with_children(|header| {
                    header
                        .spawn(text("WORK & DEPENDENCE", 23.0, theme::PAPER))
                        .insert(crate::observer_ui::ObserverFontRole::Display);
                    header
                        .spawn(button_node(ProductionCommand::Details))
                        .with_child((
                            text("READINGS +", 13.0, theme::PAPER),
                            ProductionDisclosureLabel,
                        ));
                });
            panel.spawn((
                text(
                    "Whose work makes this possible? Who relies on its output?",
                    15.0,
                    theme::PAPER,
                ),
                Node {
                    flex_shrink: 0.0,
                    min_width: px(0),
                    ..default()
                },
            ));
            panel.spawn((
                text("", 15.0, theme::PAPER),
                ProductionBrief,
                ObserverFocusTarget::reading(None),
                Node {
                    flex_shrink: 0.0,
                    min_width: px(0),
                    max_width: percent(100),
                    ..default()
                },
            ));
        });
    panel
        .spawn(crate::observer_ui::context_column())
        .with_children(|panel| {
            button(panel, "BACK  [Backspace]", ProductionCommand::Back);
            panel.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    flex_shrink: 0.0,
                    min_width: px(0),
                    ..default()
                },
                ProductionDependencies,
            ));
        });
}

fn setup_readings_panel(commands: &mut Commands) {
    commands
        .spawn((
            ProductionDetailGroup,
            TabGroup::new(20),
            crate::observer_layout::ObserverRegion::Log,
            DeclaredSurface::new(SurfaceId::ObserverProduction),
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(16)),
                border: UiRect::left(px(2)),
                row_gap: px(12),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(theme::INK),
            BorderColor::all(theme::PAPER),
            ZIndex(8),
        ))
        .with_children(readings_contents);
}

fn readings_contents(panel: &mut ChildSpawnerCommands) {
    panel
        .spawn(Node {
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: px(8),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|header| {
            header.spawn(text("READINGS", 13.0, theme::GRAY));
            button(header, "CLOSE", ProductionCommand::Details);
        });
    panel
        .spawn((
            text("", 23.0, theme::PAPER),
            ProductionReadingSubject,
            Node {
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .insert(crate::observer_ui::ObserverFontRole::Display);
    panel.spawn((
        text("", 15.0, theme::PAPER),
        ProductionReadingHeadline,
        Node {
            flex_shrink: 0.0,
            ..default()
        },
    ));
    panel
        .spawn(Node {
            column_gap: px(4),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|tabs| {
            for section in [
                ProductionReadingSection::Flow,
                ProductionReadingSection::Freight,
                ProductionReadingSection::Work,
                ProductionReadingSection::Sources,
            ] {
                button(tabs, section.label(), ProductionCommand::Reading(section));
            }
        });
    panel
        .spawn((
            ProductionReadingBody,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                flex_grow: 1.0,
                min_height: px(0),
                min_width: px(0),
                max_width: percent(100),
                overflow: Overflow::scroll_y(),
                ..default()
            },
        ))
        .with_children(|details| {
            details.spawn((
                text("", 15.0, theme::PAPER),
                ProductionDetails,
                ObserverFocusTarget::reading(None),
                Node {
                    flex_shrink: 0.0,
                    min_width: px(0),
                    max_width: percent(100),
                    ..default()
                },
            ));
        });
    button(panel, "3D / 2D  [V]", ProductionCommand::Flat);
}

fn orbit_input(
    view: Res<PrimaryView>,
    ui: Res<ObserverUiState>,
    viewport: Res<ObserverViewport>,
    mut pointer: ProductionPointer,
    mut orbit: ResMut<ProductionOrbit>,
) {
    let delta: Vec2 = pointer.motion.read().map(|event| event.delta).sum();
    let scroll: f32 = pointer.wheel.read().map(|event| event.y).sum();
    if *view != PrimaryView::Production
        || ui.menu_open
        || ui.splash_visible
        || ui.comparison_open
        || ui.disclosure.is_some()
    {
        return;
    }
    if !pointer
        .windows
        .single()
        .ok()
        .and_then(Window::cursor_position)
        .is_some_and(|point| viewport.0.is_some_and(|rect| rect.contains(point)))
    {
        return;
    }
    if pointer
        .interactions
        .iter()
        .any(|interaction| *interaction != Interaction::None)
    {
        return;
    }
    if pointer.buttons.pressed(MouseButton::Right) && delta != Vec2::ZERO {
        orbit.yaw -= delta.x * 0.006;
        orbit.pitch = (orbit.pitch + delta.y * 0.004).clamp(0.25, 1.35);
    }
    if scroll != 0.0 {
        orbit.distance = (orbit.distance - scroll * 65.0).clamp(550.0, 2200.0);
    }
}

#[derive(SystemParam)]
struct ProductionInputContext<'w> {
    observation: ProductionObservation<'w>,
    navigation: Res<'w, ProductionNavigation>,
    claim: Option<Res<'w, ObserverKeyboardClaim>>,
}

fn inputs(
    keys: Res<ButtonInput<KeyCode>>,
    mut ui: ProductionUi,
    view: Res<PrimaryView>,
    context: ProductionInputContext,
    buttons: Query<(&Interaction, &ProductionButton), Changed<Interaction>>,
    mut events: MessageWriter<ProductionCommand>,
) {
    if ui.state.menu_open || ui.state.splash_visible || ui.state.comparison_open {
        return;
    }
    let ProductionInputContext {
        observation,
        navigation,
        claim,
    } = context;
    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed {
            queue_production_button(&button.0, &observation, &navigation, &mut ui, &mut events);
        }
    }
    if claim
        .as_ref()
        .is_some_and(|claim| claim.blocks_world_shortcuts())
    {
        return;
    }
    if [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
    ]
    .iter()
    .any(|key| keys.just_pressed(*key))
    {
        events.write(ProductionCommand::Map);
    }
    for (key, command) in [
        (KeyCode::KeyP, ProductionCommand::Open),
        (KeyCode::KeyM, ProductionCommand::Map),
        (KeyCode::Backspace, ProductionCommand::Back),
    ] {
        if keys.just_pressed(key) {
            events.write(command);
        }
    }
    if *view == PrimaryView::Production && keys.just_pressed(KeyCode::KeyV) {
        events.write(ProductionCommand::Flat);
    }
}

fn production_command_context(command: &ProductionCommand) -> Option<&ObservationContext> {
    match command {
        ProductionCommand::Select { context, .. }
        | ProductionCommand::Page { context, .. }
        | ProductionCommand::Process { context, .. } => Some(context),
        _ => None,
    }
}

fn queue_production_button(
    command: &ProductionCommand,
    observation: &ProductionObservation,
    navigation: &ProductionNavigation,
    ui: &mut ProductionUi,
    events: &mut MessageWriter<ProductionCommand>,
) {
    if ui.state.menu_open || ui.state.splash_visible || ui.state.comparison_open {
        return;
    }
    let snapshot = observation
        .frame
        .for_session(&observation.state)
        .and_then(|frame| frame.production.as_ref());
    let available = ProductionControlAvailability::for_snapshot(snapshot, navigation);
    if let Some(reason) = available.refusal(command, snapshot, navigation, &observation.state) {
        ui.feedback.reject(reason, ui.time.elapsed_secs_f64());
    } else {
        events.write(command.clone());
    }
}

fn keyboard_activate(
    event: On<ObserverKeyboardActivate>,
    buttons: Query<&ProductionButton>,
    observation: ProductionObservation,
    navigation: Res<ProductionNavigation>,
    mut ui: ProductionUi,
    mut events: MessageWriter<ProductionCommand>,
) {
    let Ok(button) = buttons.get(event.entity) else {
        return;
    };
    if event.context.as_ref() != production_command_context(&button.0) {
        ui.feedback.reject(
            "This work control belongs to an older observation.",
            ui.time.elapsed_secs_f64(),
        );
        return;
    }
    queue_production_button(&button.0, &observation, &navigation, &mut ui, &mut events);
}

type ProductionFocusOwners = Or<(
    With<ProductionButton>,
    With<ProductionDetails>,
    With<ProductionBrief>,
    With<ProductionFreightReading>,
)>;

fn focus_eligibility(
    observation: ProductionObservation,
    navigation: Res<ProductionNavigation>,
    ui: Res<ObserverUiState>,
    mut targets: Query<
        (&mut ObserverFocusTarget, Option<&ProductionButton>),
        ProductionFocusOwners,
    >,
) {
    if !(observation.frame.is_changed()
        || observation.state.is_changed()
        || navigation.is_changed()
        || ui.is_changed()
        // Paint installs the reading's new scope after this PreUpdate pass.
        // Admit that changed target on the following frame even if no control moved.
        || targets.iter_mut().any(|(target, _)| target.is_changed()))
    {
        return;
    }
    let snapshot = observation
        .frame
        .for_session(&observation.state)
        .and_then(|frame| frame.production.as_ref());
    let available = ProductionControlAvailability::for_snapshot(snapshot, &navigation);
    for (mut target, button) in &mut targets {
        let (context, admitted) = match button {
            Some(button) => (
                production_command_context(&button.0).cloned(),
                available
                    .refusal(&button.0, snapshot, &navigation, &observation.state)
                    .is_none(),
            ),
            None => (
                target.context.clone(),
                target
                    .context
                    .as_ref()
                    .is_some_and(|context| observation.state.accepts(context)),
            ),
        };
        let mut next = target.clone();
        next.context = context;
        next.available = admitted && !ui.menu_open && !ui.splash_visible && !ui.comparison_open;
        target.set_if_neq(next);
    }
}

#[derive(SystemParam)]
struct ProductionLocation<'w> {
    atlas: Res<'w, CountyAtlas>,
    selected: ResMut<'w, SelectedCounty>,
}

fn navigate(
    mut commands: Commands,
    mut events: MessageReader<ProductionCommand>,
    mut view: ResMut<PrimaryView>,
    mut navigation: ResMut<ProductionNavigation>,
    observation: ProductionObservation,
    location: ProductionLocation,
    ui: ProductionUi,
) {
    let ProductionLocation {
        atlas,
        mut selected,
    } = location;
    let ProductionUi {
        state: mut ui,
        mut feedback,
        time,
    } = ui;
    let ProductionObservation { frame, state } = observation;
    let snapshot = frame
        .for_session(&state)
        .and_then(|frame| frame.production.as_ref());
    for event in events.read() {
        if ui.menu_open || ui.splash_visible || ui.comparison_open {
            continue;
        }
        let available = ProductionControlAvailability::for_snapshot(snapshot, &navigation);
        if let Some(reason) = available.refusal(event, snapshot, &navigation, &state) {
            feedback.reject(reason, time.elapsed_secs_f64());
            continue;
        }
        let mut sync_county = false;
        match event {
            ProductionCommand::Open => {
                sync_county = true;
                *view = PrimaryView::Production;
                ui.archive_open = false;
                ui.disclosure = None;
                if let Some(county) = selected.0.and_then(|index| atlas.county(index)) {
                    navigation.open_county(county.fips, snapshot);
                }
            }
            ProductionCommand::Map => {
                *view = PrimaryView::Map;
                ui.disclosure = None;
            }
            ProductionCommand::Flat => {
                navigation.flat = !navigation.flat;
            }
            ProductionCommand::Details => {
                navigation.details_open = !navigation.details_open;
            }
            ProductionCommand::Reading(section) => {
                navigation.reading_section = *section;
                navigation.details_open = true;
            }
            ProductionCommand::Page { kind, page, .. } => {
                let target = match kind {
                    ProductionPage::Cohorts => &mut navigation.cohort_page,
                    ProductionPage::Relationships => &mut navigation.relationship_page,
                    ProductionPage::Competitors => &mut navigation.competitor_page,
                };
                *target = *page;
            }
            ProductionCommand::Process { process_id, .. } => {
                navigation.selected_process = Some(process_id.clone());
                ui.history_open = true;
                navigation.details_open = false;
            }
            ProductionCommand::Back => {
                if let Some(index) = available.previous_index {
                    navigation.selected_site = Some(navigation.history[index].clone());
                    navigation.history.truncate(index);
                    sync_county = true;
                    *view = PrimaryView::Production;
                    ui.archive_open = false;
                    ui.disclosure = None;
                } else if available.county_back {
                    navigation.county_open = true;
                    navigation.selected_site = None;
                    navigation.details_open = false;
                }
                navigation.relationship_page = 0;
                navigation.competitor_page = 0;
                navigation.selected_process = None;
            }
            ProductionCommand::Select { site_id: id, .. } => {
                navigation.select_site(id);
                sync_county = true;
                *view = PrimaryView::Production;
                ui.archive_open = false;
                ui.disclosure = None;
            }
        }
        if matches!(event, ProductionCommand::Open | ProductionCommand::Map) {
            commands.trigger(ObserverFocusWorld);
        }
        if !sync_county {
            continue;
        }
        sync_selected_county(&navigation, snapshot, &atlas, &mut selected);
    }
}

fn sync_selected_county(
    navigation: &ProductionNavigation,
    snapshot: Option<&ProductionSnapshotV2>,
    atlas: &CountyAtlas,
    selected: &mut SelectedCounty,
) {
    if let Some(site) = snapshot.and_then(|snapshot| {
        snapshot
            .sites
            .iter()
            .find(|site| navigation.selected_site.as_ref() == Some(&site.id))
    }) {
        selected.0 = (0..atlas.len()).find(|index| {
            atlas
                .county(*index)
                .is_some_and(|county| county.fips == site.county_geoid)
        });
    }
}

fn sync_world_county(
    observation: ProductionObservation,
    view: Res<PrimaryView>,
    atlas: Res<CountyAtlas>,
    selected: Res<SelectedCounty>,
    mut navigation: ResMut<ProductionNavigation>,
) {
    if *view != PrimaryView::Map {
        return;
    }
    let snapshot = observation
        .frame
        .for_session(&observation.state)
        .and_then(|frame| frame.production.as_ref());
    let county = selected.0.and_then(|index| atlas.county(index));
    if let (Some(snapshot), Some(county)) = (snapshot, county) {
        if navigation.county_geoid.as_deref() != Some(county.fips) {
            navigation.open_county(county.fips, Some(snapshot));
        }
    }
}

fn invalidate_navigation(
    state: Res<ObserverSession>,
    frame: Res<ObserverFrame>,
    mut navigation: ResMut<ProductionNavigation>,
    mut scope: Local<
        Option<(
            babylon_persistence::CampaignId,
            crate::observer::Perspective,
            Option<u64>,
        )>,
    >,
) {
    let current = (state.campaign, state.perspective, state.lifecycle_epoch());
    if scope.as_ref() != Some(&current) {
        navigation.selected_site = None;
        navigation.selected_process = None;
        navigation.county_open = false;
        navigation.county_geoid = None;
        navigation.relationship_page = 0;
        navigation.competitor_page = 0;
        navigation.cohort_page = 0;
        navigation.details_open = false;
        navigation.history.clear();
        *scope = Some(current);
    }
    if frame.is_changed() {
        if let Some(snapshot) = frame
            .for_session(&state)
            .and_then(|frame| frame.production.as_ref())
        {
            navigation
                .history
                .retain(|id| snapshot.sites.iter().any(|site| site.id == *id));
            if navigation
                .selected_site
                .as_ref()
                .is_some_and(|id| !snapshot.sites.iter().any(|site| site.id == *id))
            {
                navigation.selected_site = None;
            }
        }
    }
}

/// Start with a visible dependency, then preserve the person's selection.
/// No subject can be chosen from an observation outside the current capability.
fn focus_opening(
    state: Res<ObserverSession>,
    frame: Res<ObserverFrame>,
    view: Res<PrimaryView>,
    mut navigation: ResMut<ProductionNavigation>,
    atlas: Res<CountyAtlas>,
    mut selected: ResMut<SelectedCounty>,
) {
    if *view != PrimaryView::Production
        || navigation.selected_site.is_some()
        || navigation.county_open
    {
        return;
    }
    let Some(site) = frame
        .for_session(&state)
        .and_then(|frame| frame.production.as_ref())
        .and_then(opening_site)
    else {
        return;
    };
    navigation.selected_site = Some(site.id.clone());
    selected.0 = (0..atlas.len()).find(|index| {
        atlas
            .county(*index)
            .is_some_and(|county| county.fips == site.county_geoid)
    });
}

type InspectorScrolls<'w, 's> = Query<
    'w,
    's,
    &'static mut ScrollPosition,
    Or<(With<ProductionPanel>, With<ProductionReadingBody>)>,
>;

#[derive(PartialEq, Eq)]
struct InspectorScrollScope {
    campaign: babylon_persistence::CampaignId,
    perspective: crate::observer::Perspective,
    site: Option<String>,
    section: ProductionReadingSection,
}

/// New subjects start at their brief; a new period preserves the reader's place.
fn reset_inspector_scroll(
    state: Res<ObserverSession>,
    navigation: Res<ProductionNavigation>,
    mut panels: InspectorScrolls,
    mut previous: Local<Option<InspectorScrollScope>>,
) {
    let scope = InspectorScrollScope {
        campaign: state.campaign,
        perspective: state.perspective,
        site: navigation.selected_site.clone(),
        section: navigation.reading_section,
    };
    if previous.as_ref() == Some(&scope) {
        return;
    }
    *previous = Some(scope);
    for mut position in &mut panels {
        if position.0 != Vec2::ZERO {
            position.0 = Vec2::ZERO;
        }
    }
}

fn block(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    size: Vec3,
    transform: Transform,
    color: Color,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.7,
            metallic: 0.15,
            ..default()
        })),
        transform,
        RenderLayers::layer(1),
        ProductionGeometry,
        DeclaredSurface::new(SurfaceId::ObserverProduction),
    ));
}

#[derive(PartialEq)]
struct ProductionGeometryContext {
    observation: ObservationContext,
    flat: bool,
    selected_site: Option<String>,
    relationship_page: usize,
}

fn rebuild(
    mut commands: Commands,
    observation: ProductionObservation,
    navigation: Res<ProductionNavigation>,
    old: Query<Entity, (SceneGeometry, Without<ChildOf>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last_context: Local<Option<ProductionGeometryContext>>,
) {
    let ProductionObservation { frame, state } = observation;
    let context = state.context();
    let geometry_key = ProductionGeometryContext {
        observation: context.clone(),
        flat: navigation.flat,
        selected_site: navigation.selected_site.clone(),
        relationship_page: navigation.relationship_page,
    };
    if !frame.is_changed() && last_context.as_ref() == Some(&geometry_key) {
        return;
    }
    *last_context = Some(geometry_key);
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let Some(snapshot) = frame
        .for_session(&state)
        .and_then(|frame| frame.production.as_ref())
    else {
        return;
    };
    let layout = ProductionLayout::focused(
        snapshot,
        navigation.selected_site.as_deref(),
        navigation.relationship_page,
    );
    for (center, size) in &layout.platforms {
        block(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(size.x, 10.0, size.y),
            Transform::from_translation(*center),
            theme::PANEL,
        );
    }
    spawn_sites(
        &mut commands,
        &mut meshes,
        &mut materials,
        snapshot,
        &navigation,
        &context,
        &layout,
    );
    spawn_routes(
        &mut commands,
        &mut meshes,
        &mut materials,
        &layout,
        &navigation,
    );
    spawn_freight(
        &mut commands,
        &mut meshes,
        &mut materials,
        snapshot,
        &layout,
        state.viewed_tick,
    );
}

fn spawn_sites(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    snapshot: &ProductionSnapshotV2,
    navigation: &ProductionNavigation,
    context: &ObservationContext,
    layout: &ProductionLayout,
) {
    let labels = commands
        .spawn((
            ProductionGeometry,
            TabGroup::new(10),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Pickable::IGNORE,
            DeclaredSurface::new(SurfaceId::ObserverProduction),
        ))
        .id();
    for site in &snapshot.sites {
        let Some(&origin) = layout.positions.get(&site.id) else {
            continue;
        };
        let height = if navigation.flat { 12.0 } else { 86.0 };
        let selected = navigation.selected_site.as_ref() == Some(&site.id);
        let color = if selected { theme::PAPER } else { theme::LAND };
        block(
            commands,
            meshes,
            materials,
            Vec3::new(142.0, 8.0, 106.0),
            Transform::from_translation(origin),
            if selected { theme::PAPER } else { theme::GRAY },
        );
        block(
            commands,
            meshes,
            materials,
            Vec3::new(116.0, height, 82.0),
            Transform::from_translation(origin + Vec3::Y * (height * 0.5 + 5.0)),
            color,
        );
        spawn_site_label(
            commands,
            site,
            context,
            origin + Vec3::Y * (height + 14.0),
            selected,
            color,
            labels,
        );
    }
}

fn spawn_site_label(
    commands: &mut Commands,
    site: &ProductionSiteV2,
    context: &ObservationContext,
    anchor: Vec3,
    selected: bool,
    color: Color,
    parent: Entity,
) {
    let leader = commands
        .spawn((
            ProductionLeader,
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            UiTransform::IDENTITY,
            Visibility::Hidden,
            BackgroundColor(if selected { theme::PAPER } else { theme::GRAY }),
            ZIndex(3),
            Pickable::IGNORE,
            ChildOf(parent),
            DeclaredSurface::new(SurfaceId::ObserverProduction),
        ))
        .id();
    commands
        .spawn((
            Button,
            ProductionButton(ProductionCommand::Select {
                site_id: site.id.clone(),
                context: context.clone(),
            }),
            ObserverFocusTarget::action(Some(context.clone())),
            ChildOf(parent),
            ProductionLabel {
                anchor,
                site_id: site.id.clone(),
                selected,
                leader,
            },
            Node {
                position_type: PositionType::Absolute,
                padding: UiRect::axes(px(7), px(4)),
                max_width: px(174),
                border: UiRect::left(px(if selected { 3 } else { 1 })),
                ..default()
            },
            BackgroundColor(theme::INK.with_alpha(0.93)),
            BorderColor::all(color),
            ZIndex(4),
            Visibility::Hidden,
            DeclaredSurface::new(SurfaceId::ObserverProduction),
        ))
        .with_child(text(
            if selected {
                format!(
                    "{}\n{}",
                    site.name.trim_end_matches(" cohort"),
                    committed_plan_status(site)
                        .strip_prefix("Committed ")
                        .unwrap_or(committed_plan_status(site))
                )
            } else {
                site.name.trim_end_matches(" cohort").to_owned()
            },
            12.0,
            theme::PAPER,
        ));
}

fn spawn_routes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    layout: &ProductionLayout,
    navigation: &ProductionNavigation,
) {
    for (supplier, buyer) in &layout.links {
        let (Some(from), Some(to)) = (layout.positions.get(supplier), layout.positions.get(buyer))
        else {
            continue;
        };
        let selected = navigation.selected_site.as_ref();
        let incident = selected.is_some_and(|site| site == supplier || site == buyer);
        let color = if selected == Some(supplier) {
            theme::COPPER
        } else if selected == Some(buyer) {
            theme::BLUE
        } else {
            theme::GRAY
        };
        let width = if incident { 7.0 } else { 4.0 };
        let path = relation_path(*from, *to);
        for segment in path.windows(2) {
            rail(
                commands, meshes, materials, segment[0], segment[1], width, color,
            );
        }
        if let Some(last) = path.windows(2).last() {
            let direction = (last[1] - last[0]).normalize();
            let tip = last[1] - direction * 20.0;
            let side = Vec3::new(-direction.z, 0.0, direction.x) * 14.0;
            for wing in [-side, side] {
                rail(
                    commands,
                    meshes,
                    materials,
                    tip - direction * 22.0 + wing,
                    tip,
                    width,
                    color,
                );
            }
        }
    }
}

fn rail(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    from: Vec3,
    to: Vec3,
    width: f32,
    color: Color,
) {
    let delta = to - from;
    block(
        commands,
        meshes,
        materials,
        Vec3::new(width, 5.0, delta.length()),
        Transform::from_translation((from + to) * 0.5)
            .with_rotation(Quat::from_rotation_y(delta.x.atan2(delta.z))),
        color,
    );
}

// Actual lots get static, evenly separated schematic positions, not invented
// travel motion or geographic progress between committed observations.
#[allow(clippy::cast_precision_loss)]
fn freight_markers(
    snapshot: &ProductionSnapshotV2,
    layout: &ProductionLayout,
    viewed_tick: u64,
) -> Vec<Vec3> {
    let mut groups = std::collections::BTreeMap::<(&str, &str), Vec<_>>::new();
    for lot in &snapshot.freight {
        if lot.quantity == 0
            || lot.dispatch_period > viewed_tick
            || lot.arrival_period <= viewed_tick
        {
            continue;
        }
        if !snapshot.routes.iter().any(|route| {
            route.id == lot.route_id
                && route.supplier_site_id == lot.source_site_id
                && route.buyer_site_id == lot.destination_site_id
                && route.good_id == lot.good_id
                && route.unit_id == lot.unit_id
        }) {
            continue;
        }
        groups
            .entry((&lot.source_site_id, &lot.destination_site_id))
            .or_default()
            .push(lot);
    }
    let mut markers = Vec::new();
    for ((supplier, buyer), mut lots) in groups {
        let (Some(&from), Some(&to)) =
            (layout.positions.get(supplier), layout.positions.get(buyer))
        else {
            continue;
        };
        lots.sort_by(|left, right| left.id.cmp(&right.id));
        let path = relation_path(from, to);
        for index in 0..lots.len() {
            if let Some(position) = path_point(&path, (index + 1) as f32 / (lots.len() + 1) as f32)
            {
                markers.push(position + Vec3::Y * 14.0);
            }
        }
    }
    markers
}

fn spawn_freight(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    snapshot: &ProductionSnapshotV2,
    layout: &ProductionLayout,
    viewed_tick: u64,
) {
    for position in freight_markers(snapshot, layout, viewed_tick) {
        block(
            commands,
            meshes,
            materials,
            Vec3::splat(17.0),
            Transform::from_translation(position),
            theme::PAPER,
        );
    }
}

fn describe(
    site: &ProductionSiteV2,
    snapshot: &ProductionSnapshotV2,
    section: ProductionReadingSection,
) -> String {
    match section {
        ProductionReadingSection::Flow => describe_flow(site, snapshot),
        ProductionReadingSection::Freight => describe_freight(site, snapshot),
        ProductionReadingSection::Work => describe_work(site, snapshot),
        ProductionReadingSection::Sources => describe_sources(site, snapshot),
    }
}

fn reading_headline(
    site: &ProductionSiteV2,
    snapshot: &ProductionSnapshotV2,
    period: u64,
) -> String {
    let mut value = if period == 0 {
        "Foundation / Designed\n".to_owned()
    } else {
        format!("Period {period} / committed reading\n")
    };
    let mut outputs = std::collections::BTreeSet::new();
    for process in &site.processes {
        if !outputs.insert((&process.output_good_id, &process.output_unit_id)) {
            continue;
        }
        let output = snapshot.material_balance.as_ref().and_then(|balance| {
            balance.rows.iter().find(|row| {
                row.site_id == site.id
                    && row.good_id == process.output_good_id
                    && row.unit_id == process.output_unit_id
            })
        });
        if let Some(output) = output {
            writeln!(
                value,
                "{} {} produced / Derived · {}",
                grouped(output.produced),
                output.unit,
                process.output_good
            )
            .expect("String write");
        } else {
            writeln!(
                value,
                "{}: {}",
                process.output_good,
                crate::production_brief::process_plan_status(process)
            )
            .expect("String write");
        }
    }
    if site.processes.is_empty() {
        writeln!(value, "{}", committed_plan_status(site)).expect("String write");
    }
    let accounts: Vec<_> = snapshot
        .staffing_accounts
        .iter()
        .filter(|account| account.site_id == site.id)
        .collect();
    if accounts.is_empty() {
        value.push_str("Modeled workforce not disclosed");
    }
    for account in accounts {
        writeln!(
            value,
            "{} employed · {} reserve / {}",
            grouped(account.employed),
            grouped(account.reserve),
            if account.completed.is_some() {
                "Derived"
            } else {
                "Designed"
            }
        )
        .expect("String write");
    }
    value.trim_end().to_owned()
}

fn describe_flow(site: &ProductionSiteV2, snapshot: &ProductionSnapshotV2) -> String {
    let mut value = String::new();
    for process in &site.processes {
        writeln!(
            value,
            "{} / {}\n{}",
            process.name,
            process.output_good,
            crate::production_brief::process_plan_status(process)
        )
        .expect("String write");
        if let (Some(done), Some(plan)) = (process.produced_batches, process.planned_batches) {
            writeln!(
                value,
                "COMMITTED PRODUCTION\n{done} of {plan} planned batches"
            )
            .expect("String write");
        }
        writeln!(
            value,
            "{} {} / batch (Designed)\nNext-period capacity: {} batches\n\nINPUTS / ON HAND",
            grouped(process.output_per_batch),
            process.output_unit,
            grouped(process.available_batches)
        )
        .expect("String write");
        for input in &process.inputs {
            writeln!(
                value,
                "{}: {} {}",
                input.good,
                grouped(input.on_hand),
                input.unit
            )
            .expect("String write");
        }
        if process.inputs.is_empty() {
            value.push_str("No material inputs in this recipe.\n");
        }
        value.push('\n');
    }
    if site.processes.is_empty() {
        writeln!(value, "{}", committed_plan_status(site)).expect("String write");
    }
    for account in snapshot
        .final_demand_accounts
        .iter()
        .filter(|account| account.retailer_site_ids.contains(&site.id))
    {
        writeln!(value, "COUNTY FINAL DEMAND / {}\n{} {} ordered · {} fulfilled · {} outstanding\nDelivery to end buyers; consumption is not recorded.", account.good, grouped(account.ordered), account.unit, grouped(account.fulfilled), grouped(account.outstanding)).expect("String write");
    }
    describe_material_balance(&mut value, site, snapshot);
    value.push_str("\nINVENTORY\n");
    for stock in &site.inventory {
        writeln!(
            value,
            "{}: {} {}",
            stock.good,
            grouped(stock.quantity),
            stock.unit
        )
        .expect("String write");
    }
    value
}

fn describe_freight(site: &ProductionSiteV2, snapshot: &ProductionSnapshotV2) -> String {
    let mut value = String::new();
    let accounts = shared_accounts(snapshot, Some(&site.id));
    if accounts.is_empty() {
        value.push_str("No shared freight pool disclosed for this subject.\n");
    }
    if accounts.len() > 3 {
        writeln!(value, "{} shared capacity accounts; showing the three with least next-opening availability.\n", accounts.len()).expect("String write");
    }
    for account in accounts.into_iter().take(3) {
        value.push_str(&account_reading(account, snapshot));
        value.push('\n');
    }
    value.push_str("\nPHYSICAL DELIVERIES / TO DATE\n");
    for route in snapshot
        .routes
        .iter()
        .filter(|route| route.buyer_site_id == site.id || route.supplier_site_id == site.id)
    {
        let other = if route.buyer_site_id == site.id {
            &route.supplier_site_id
        } else {
            &route.buyer_site_id
        };
        let name = snapshot
            .sites
            .iter()
            .find(|site| site.id == *other)
            .map_or(other.as_str(), |site| site.name.as_str());
        writeln!(
            value,
            "{} | {}\n{} / {} {} delivered | {} unshipped\n",
            name,
            match route.transport_kind {
                babylon_persistence::ProductionRouteTransportV2::Local =>
                    "Local inter-owner transfer".into(),
                babylon_persistence::ProductionRouteTransportV2::Staged =>
                    format!("{} periods travel", route.travel_periods),
            },
            grouped(route.delivered),
            grouped(route.ordered),
            route.unit,
            route
                .ordered
                .checked_sub(route.shipped)
                .map_or_else(|| "unavailable".into(), grouped)
        )
        .expect("String write");
    }
    value.push_str("Deliveries record quantities, not payments.\n");
    value
}

fn describe_work(site: &ProductionSiteV2, snapshot: &ProductionSnapshotV2) -> String {
    let mut value = String::new();
    describe_staffing_accounts(&mut value, site, snapshot);
    describe_labor_accounts(&mut value, site, snapshot);
    value.push_str("\nLABOR BUDGET / DERIVED\n");
    for labor in site.processes.iter().flat_map(|process| &process.labor) {
        writeln!(
            value,
            "{} {} available | {} / batch (Designed)",
            grouped(labor.available),
            labor.unit,
            grouped(labor.quantity_per_batch)
        )
        .expect("String write");
    }
    for handling in snapshot
        .merchant_handling_accounts
        .iter()
        .filter(|account| account.site_id == site.id)
    {
        value.push_str("\nMERCHANT HANDLING\n");
        if let Some(done) = &handling.completed {
            writeln!(
                value,
                "Period {}: {} handled · {} / {} labor-hours used / needed",
                done.period,
                format_freight_mass(done.handled_grams),
                grouped(done.used_hours),
                grouped(done.needed_hours)
            )
            .expect("String write");
        } else {
            value.push_str("Foundation; no completed handling work.\n");
        }
        value.push_str("Handling moves existing goods; it does not create productive output.\n");
    }
    value.trim_start().to_owned()
}

fn describe_sources(site: &ProductionSiteV2, snapshot: &ProductionSnapshotV2) -> String {
    let mut value = format!(
        "COUNTY-SECTOR OWNER / NAICS {}\nSector {} · {:?}\n",
        site.industry_code, site.sector_code, site.role
    );
    for process in &site.processes {
        writeln!(
            value,
            "\nRECIPE / DESIGNED / {}\n{} {} / batch",
            process.name,
            grouped(process.output_per_batch),
            process.output_unit
        )
        .expect("String write");
        for input in &process.inputs {
            writeln!(
                value,
                "{}: {} {} / batch",
                input.good,
                grouped(input.quantity_per_batch),
                input.unit
            )
            .expect("String write");
        }
    }
    if let Some(jobs) = site.observed_employment {
        writeln!(value, "\nINDUSTRY EMPLOYMENT / OBSERVED 2024\n{} annual-average jobs (QCEW; separate from modeled people and hours)", grouped(jobs)).expect("String write");
    }
    describe_sector_context(&mut value, site, snapshot);
    writeln!(
        value,
        "\nCAPTURED AUTHORITY\n{}",
        snapshot.content_authority_sha256
    )
    .expect("String write");
    if let Some(source) = &snapshot.road_source {
        writeln!(
            value,
            "Road extract: {}\nReplication: {}\nRouting profile: {}\nGraph: {}",
            source.pbf_url,
            source.replication_timestamp,
            source.routing_profile_version,
            source.graph_sha256
        )
        .expect("String write");
        value.push_str("Road data © OpenStreetMap contributors, available under the Open Database License (ODbL).\nhttps://www.openstreetmap.org/copyright\n");
    }
    value.push_str("\nSCENE KEY\nEqual-height structures identify county cohorts; height and spacing carry no quantity or geography. Arrows point from disclosed suppliers to buyers. Cyan links enter the selection; copper links leave it. Packets are actual in-transit lots at static schematic positions.\n");
    value
}

fn describe_material_balance(
    value: &mut String,
    site: &ProductionSiteV2,
    snapshot: &ProductionSnapshotV2,
) {
    let Some(balance) = &snapshot.material_balance else {
        value.push_str("\nNo completed stock-movement account at this point.\n");
        return;
    };
    let mut rows = balance
        .rows
        .iter()
        .filter(|row| row.site_id == site.id)
        .peekable();
    if rows.peek().is_none() {
        value.push_str("\nNo stock-movement account disclosed for this subject.\n");
        return;
    }
    writeln!(value, "\nSTOCK MOVEMENT / PERIOD {}", balance.period).expect("String write");
    for row in rows {
        if row.local_received != 0 || row.local_transferred != 0 || row.final_demand_fulfilled != 0
        {
            writeln!(value, "{} / {}\nOpened {} + arrived {} + received locally {} + produced {}\n= consumed {} + dispatched {} + transferred locally {} + final demand {} + closed {}", row.good, row.unit, grouped(row.opening), grouped(row.arrivals), grouped(row.local_received), grouped(row.produced), grouped(row.consumed), grouped(row.dispatched), grouped(row.local_transferred), grouped(row.final_demand_fulfilled), grouped(row.closing)).expect("String write");
            continue;
        }
        writeln!(
            value,
            "{} / {}\nOpened {} + arrived {} + produced {}\n= consumed {} + dispatched {} + closed {}",
            row.good,
            row.unit,
            grouped(row.opening),
            grouped(row.arrivals),
            grouped(row.produced),
            grouped(row.consumed),
            grouped(row.dispatched),
            grouped(row.closing),
        )
        .expect("String write");
    }
}

fn describe_sector_context(
    value: &mut String,
    site: &ProductionSiteV2,
    snapshot: &ProductionSnapshotV2,
) {
    let subjects: std::collections::BTreeSet<_> = snapshot
        .process_attributions
        .iter()
        .filter(|link| link.site_id == site.id)
        .map(|link| &link.cohort_subject)
        .collect();
    for context in snapshot.observed_contexts.iter().filter(|context| {
        context.county_geoid == site.county_geoid && subjects.contains(&context.subject)
    }) {
        writeln!(
            value,
            "\nSECTOR CONTEXT / OBSERVED {}\n{} | NAICS {}\n{} establishments",
            context.vintage,
            context.sector_title,
            context.sector_code,
            grouped(context.annual_avg_estabs_count),
        )
        .expect("String write");
        for (metric, prefix, suffix, undisclosed) in [
            (
                context.annual_avg_emplvl,
                "",
                " annual-average jobs",
                "Annual-average jobs: not disclosed",
            ),
            (
                context.total_annual_wages,
                "USD ",
                " annual payroll",
                "Annual payroll: not disclosed",
            ),
            (
                context.annual_avg_wkly_wage,
                "USD ",
                " mean weekly wage",
                "Mean weekly wage: not disclosed",
            ),
        ] {
            match metric {
                Some(metric) => writeln!(value, "{prefix}{}{suffix}", grouped(metric)),
                None => writeln!(value, "{undisclosed}"),
            }
            .expect("String write");
        }
        let shared_ids: std::collections::BTreeSet<_> = snapshot
            .process_attributions
            .iter()
            .filter(|link| link.cohort_subject == context.subject)
            .map(|link| link.site_id.as_str())
            .collect();
        let mut names: Vec<_> = snapshot
            .sites
            .iter()
            .filter(|other| shared_ids.contains(other.id.as_str()))
            .map(|other| other.name.as_str())
            .collect();
        names.sort_unstable();
        writeln!(
            value,
            "Modeled processes sharing this context: {}\nThis county-sector total does not assign workers to a process.\nSource: BLS QCEW / {}\n{}",
            names.join("; "),
            context.source_file,
            context.source_url,
        )
        .expect("String write");
    }
}

fn describe_staffing_accounts(
    value: &mut String,
    site: &ProductionSiteV2,
    snapshot: &ProductionSnapshotV2,
) {
    let mut disclosed = false;
    for account in snapshot
        .staffing_accounts
        .iter()
        .filter(|account| account.site_id == site.id)
    {
        disclosed = true;
        value.push_str(if account.completed.is_some() {
            "\nMODELED WORKFORCE / DERIVED\n"
        } else {
            "\nMODELED WORKFORCE / DESIGNED\n"
        });
        writeln!(
            value,
            "{} employed + {} reserve = {} people\n{} hours per person / period (Designed)",
            grouped(account.employed),
            grouped(account.reserve),
            grouped(account.labor_force),
            grouped(account.hours_per_person),
        )
        .expect("String write");
        if let Some(completed) = &account.completed {
            writeln!(
                value,
                "\nSTAFFING / PERIOD {}\nOpening: {} employed, {} reserve\nHires: {} | separations: {} | target: {} employed\nWork request: {} hours | prior period: {} hours\nOne-period retention: {} hours\n",
                completed.period,
                grouped(completed.opening_employed),
                grouped(completed.opening_reserve),
                grouped(completed.hires),
                grouped(completed.separations),
                grouped(completed.target_employed),
                grouped(completed.current_unretained_hours),
                grouped(completed.previous_unretained_hours),
                grouped(completed.retained_hours),
            )
            .expect("String write");
        } else {
            value.push_str("Opening workforce; no completed staffing period.\n");
        }
    }
    if disclosed {
        value.push_str("Observed QCEW jobs are separate; these accounts record no payments.\n");
    } else {
        value.push_str("\nMODELED WORKFORCE\nNo workforce account disclosed for this subject.\n");
    }
}

fn describe_labor_accounts(
    value: &mut String,
    site: &ProductionSiteV2,
    snapshot: &ProductionSnapshotV2,
) {
    for account in snapshot
        .labor_accounts
        .iter()
        .filter(|account| account.site_id == site.id)
    {
        if let Some(completed) = &account.completed {
            writeln!(
                value,
                "\nCOMMITTED WORK TIME / PERIOD {} / DERIVED\n{} used + {} unused = {} available\nProduction planned: {} {}",
                completed.period,
                grouped(completed.used),
                grouped(completed.unused),
                grouped(completed.opening),
                grouped(completed.planned),
                account.unit,
            )
            .expect("String write");
            if site.processes.is_empty()
                || completed.handling_needed != 0
                || completed.handling_used != 0
            {
                writeln!(
                    value,
                    "Handling: {} needed · {} used {}",
                    grouped(completed.handling_needed),
                    grouped(completed.handling_used),
                    account.unit
                )
                .expect("String write");
            }
            value.push_str("Time accounts do not measure job losses.\n");
        }
        writeln!(
            value,
            "Next opening (period {}): {} {} (Derived)",
            account.next_opening_period,
            grouped(account.next_opening_available),
            account.unit,
        )
        .expect("String write");
    }
}

fn page_controls(
    panel: &mut ChildSpawnerCommands,
    kind: ProductionPage,
    page: usize,
    total: usize,
    context: &ObservationContext,
) {
    if total <= 6 {
        return;
    }
    panel.spawn(text(
        format!("Page {} of {} · six per page", page + 1, total.div_ceil(6)),
        13.0,
        theme::GRAY,
    ));
    if page > 0 {
        button(
            panel,
            "Previous page",
            ProductionCommand::Page {
                kind,
                page: page - 1,
                context: context.clone(),
            },
        );
    }
    if page + 1 < total.div_ceil(6) {
        button(
            panel,
            "Next page",
            ProductionCommand::Page {
                kind,
                page: page + 1,
                context: context.clone(),
            },
        );
    }
}

fn spawn_county_cohorts(
    panel: &mut ChildSpawnerCommands,
    snapshot: &ProductionSnapshotV2,
    navigation: &ProductionNavigation,
    context: &ObservationContext,
) {
    let mut sites: Vec<_> = snapshot
        .sites
        .iter()
        .filter(|site| navigation.county_geoid.as_ref() == Some(&site.county_geoid))
        .collect();
    sites.sort_by(|a, b| (&a.sector_code, &a.id).cmp(&(&b.sector_code, &b.id)));
    panel.spawn(text(
        format!("COUNTY COHORTS / {} disclosed", sites.len()),
        15.0,
        theme::YELLOW,
    ));
    panel.spawn(text(
        "Commodity production and circulation are active. Other sectors remain reference context.",
        13.0,
        theme::GRAY,
    ));
    page_controls(
        panel,
        ProductionPage::Cohorts,
        navigation.cohort_page,
        sites.len(),
        context,
    );
    for site in sites
        .into_iter()
        .skip(navigation.cohort_page.saturating_mul(6))
        .take(6)
    {
        let accounts: Vec<_> = snapshot
            .staffing_accounts
            .iter()
            .filter(|account| account.site_id == site.id)
            .collect();
        let workforce = if accounts.len() == 1 {
            format!(
                "{} employed · {} reserve",
                grouped(accounts[0].employed),
                grouped(accounts[0].reserve)
            )
        } else {
            "Workforce accounts in Work".into()
        };
        button(
            panel,
            &format!(
                "{} / {:?}\n{} · {}",
                site.name, site.role, site.sector_code, workforce
            ),
            ProductionCommand::Select {
                site_id: site.id.clone(),
                context: context.clone(),
            },
        );
    }
}

fn spawn_freight_participants(
    panel: &mut ChildSpawnerCommands,
    site: &ProductionSiteV2,
    snapshot: &ProductionSnapshotV2,
    navigation: &ProductionNavigation,
    context: &ObservationContext,
) {
    for account in shared_accounts(snapshot, Some(&site.id))
        .into_iter()
        .take(1)
    {
        panel.spawn((
            text(account_brief(account), 15.0, theme::PAPER),
            ProductionFreightReading,
            Node {
                flex_shrink: 0.0,
                min_width: px(0),
                max_width: percent(100),
                ..default()
            },
            ObserverFocusTarget::reading(Some(context.clone())),
        ));
    }
    let competitors = competitor_sites(&site.id, snapshot);
    if !competitors.is_empty() {
        panel.spawn(text(
            "OTHER PARTICIPANTS / SHARED FREIGHT",
            13.0,
            theme::YELLOW,
        ));
        page_controls(
            panel,
            ProductionPage::Competitors,
            navigation.competitor_page,
            competitors.len(),
            context,
        );
        for competitor in competitors
            .into_iter()
            .skip(navigation.competitor_page.saturating_mul(6))
            .take(6)
        {
            button(
                panel,
                &format!(
                    "{}\nInspect shared-freight participant",
                    competitor.name.trim_end_matches(" cohort")
                ),
                ProductionCommand::Select {
                    site_id: competitor.id.clone(),
                    context: context.clone(),
                },
            );
        }
    }
}

fn rebuild_dependencies(
    mut commands: Commands,
    roots: Query<Entity, With<ProductionDependencies>>,
    state: Res<ObserverSession>,
    frame: Res<ObserverFrame>,
    navigation: Res<ProductionNavigation>,
    mut last_context: Local<Option<ObservationContext>>,
) {
    let context = state.context();
    if !frame.is_changed() && !navigation.is_changed() && last_context.as_ref() == Some(&context) {
        return;
    }
    *last_context = Some(context.clone());
    let snapshot = frame
        .for_session(&state)
        .and_then(|frame| frame.production.as_ref());
    for root in &roots {
        commands.entity(root).despawn_related::<Children>();
        let Some(snapshot) = snapshot else {
            continue;
        };
        if navigation.county_open {
            commands.entity(root).with_children(|panel| {
                spawn_county_cohorts(panel, snapshot, &navigation, &context);
            });
            continue;
        }
        let Some(site) = snapshot
            .sites
            .iter()
            .find(|site| navigation.selected_site.as_ref() == Some(&site.id))
        else {
            continue;
        };
        let layout =
            ProductionLayout::focused(snapshot, Some(&site.id), navigation.relationship_page);
        let links = dependency_sites(site, snapshot);
        let group_count = links
            .iter()
            .map(|(_, site)| &site.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        commands.entity(root).with_children(|panel| {
            for process in &site.processes {
                if site.processes.len() > 1 {
                    button(
                        panel,
                        &format!("Chart {} / {}", process.name, process.output_unit),
                        ProductionCommand::Process {
                            process_id: process.id.clone(),
                            context: context.clone(),
                        },
                    );
                }
            }
            spawn_freight_participants(panel, site, snapshot, &navigation, &context);
            page_controls(
                panel,
                ProductionPage::Relationships,
                navigation.relationship_page,
                group_count,
                &context,
            );
            for direction in [
                DependencyDirection::Upstream,
                DependencyDirection::Downstream,
            ] {
                panel.spawn(text(direction.label(), 13.0, theme::YELLOW));
                let mut count = 0;
                for (_, neighbor) in links.iter().filter(|(candidate, neighbor)| {
                    *candidate == direction && layout.positions.contains_key(&neighbor.id)
                }) {
                    count += 1;
                    button(
                        panel,
                        &format!(
                            "{}\n{}",
                            neighbor.name.trim_end_matches(" cohort"),
                            dependency_flow_summary(site, neighbor, direction, snapshot)
                        ),
                        ProductionCommand::Select {
                            site_id: neighbor.id.clone(),
                            context: context.clone(),
                        },
                    );
                }
                if count == 0 {
                    panel.spawn(text("No relation on this page.", 13.0, theme::GRAY));
                }
            }
        });
    }
}

fn rebuild_county_cohorts(
    mut commands: Commands,
    roots: Query<Entity, With<ProductionCountyCohorts>>,
    observation: ProductionObservation,
    navigation: Res<ProductionNavigation>,
    mut last_context: Local<Option<ObservationContext>>,
) {
    let context = observation.state.context();
    if !observation.frame.is_changed()
        && !navigation.is_changed()
        && last_context.as_ref() == Some(&context)
    {
        return;
    }
    *last_context = Some(context.clone());
    let snapshot = observation
        .frame
        .for_session(&observation.state)
        .and_then(|frame| frame.production.as_ref());
    for root in &roots {
        commands.entity(root).despawn_related::<Children>();
        if let Some(snapshot) = snapshot {
            commands.entity(root).with_children(|panel| {
                spawn_county_cohorts(panel, snapshot, &navigation, &context);
            });
        }
    }
}

fn paint_disclosure(
    navigation: Res<ProductionNavigation>,
    observation: ProductionObservation,
    view: Res<PrimaryView>,
    ui: Res<ObserverUiState>,
    mut groups: Query<&mut Node, (With<ProductionDetailGroup>, Without<ProductionButton>)>,
    mut controls: Query<(&ProductionButton, &mut Node), Without<ProductionDetailGroup>>,
    mut labels: Query<&mut Text, With<ProductionDisclosureLabel>>,
) {
    if !navigation.is_changed()
        && !observation.frame.is_changed()
        && !observation.state.is_changed()
        && !view.is_changed()
        && !ui.is_changed()
    {
        return;
    }
    let snapshot = observation
        .frame
        .for_session(&observation.state)
        .and_then(|frame| frame.production.as_ref());
    let available = ProductionControlAvailability::for_snapshot(snapshot, &navigation);
    for (button, mut node) in &mut controls {
        if let Some(next) = available.display(&button.0) {
            if node.display != next {
                node.display = next;
            }
        }
    }
    for mut node in &mut groups {
        let next = if readings_panel_visible(*view, &navigation, &ui, snapshot) {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != next {
            node.display = next;
        }
    }
    for mut text in &mut labels {
        let next = if navigation.details_open {
            "READINGS -"
        } else {
            "READINGS +"
        };
        if text.0 != next {
            next.clone_into(&mut text.0);
        }
    }
}

fn paint_buttons(
    navigation: Res<ProductionNavigation>,
    view: Res<PrimaryView>,
    mut buttons: Query<ButtonVisuals>,
) {
    for (button, interaction, mut background, mut border) in &mut buttons {
        let selected = match &button.0 {
            ProductionCommand::Open => *view == PrimaryView::Production,
            ProductionCommand::Map => *view == PrimaryView::Map,
            ProductionCommand::Flat => navigation.flat,
            ProductionCommand::Details => navigation.details_open,
            ProductionCommand::Reading(section) => navigation.reading_section == *section,
            ProductionCommand::Select { site_id, .. } => {
                navigation.selected_site.as_ref() == Some(site_id)
            }
            ProductionCommand::Process { process_id, .. } => {
                navigation.selected_process.as_ref() == Some(process_id)
            }
            ProductionCommand::Back | ProductionCommand::Page { .. } => false,
        };
        let next = match interaction {
            Interaction::Pressed => theme::RED.with_alpha(0.5),
            Interaction::Hovered => theme::YELLOW.with_alpha(0.25),
            Interaction::None if selected => theme::YELLOW.with_alpha(0.2),
            Interaction::None => theme::PANEL,
        };
        if background.0 != next {
            background.0 = next;
        }
        border.set_if_neq(BorderColor::all(if selected {
            theme::YELLOW
        } else {
            theme::PAPER
        }));
    }
}

fn paint_scene(
    view: Res<PrimaryView>,
    navigation: Res<ProductionNavigation>,
    ui: Res<ObserverUiState>,
    orbit: Res<ProductionOrbit>,
    viewport: Res<ObserverViewport>,
    scene: ProductionScene,
    observation: ProductionObservation,
) {
    let ProductionScene {
        windows,
        mut camera,
        mut panels,
    } = scene;
    let visible = *view == PrimaryView::Production;
    let snapshot = observation
        .frame
        .for_session(&observation.state)
        .and_then(|frame| frame.production.as_ref());
    let readings = readings_panel_visible(*view, &navigation, &ui, snapshot);
    let blocked = ui.menu_open || ui.splash_visible || ui.comparison_open;
    for (mut panel, _) in &mut panels {
        panel.set_if_neq(
            if visible && !readings && !ui.archive_open && !ui.history_open && !blocked {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        );
    }
    let Ok((mut camera, mut transform, mut projection)) = camera.single_mut() else {
        return;
    };
    if camera.is_active != visible {
        camera.is_active = visible;
    }
    if let (Some(rect), Ok(window)) = (viewport.0, windows.single()) {
        let next_viewport = Viewport {
            physical_position: (rect.min * window.scale_factor()).as_uvec2(),
            physical_size: (rect.size() * window.scale_factor()).as_uvec2(),
            ..default()
        };
        if camera.viewport.as_ref().is_none_or(|current| {
            current.physical_position != next_viewport.physical_position
                || current.physical_size != next_viewport.physical_size
        }) {
            camera.viewport = Some(next_viewport);
        }
    }
    if navigation.flat != matches!(&*projection, Projection::Orthographic(_)) {
        *projection = if navigation.flat {
            Projection::Orthographic(OrthographicProjection {
                // The camera sits 1300 units above the scene; include the
                // plinths below the origin as well as the raised structures.
                near: 0.1,
                far: 2000.0,
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: 760.0,
                },
                ..OrthographicProjection::default_3d()
            })
        } else {
            Projection::Perspective(PerspectiveProjection::default())
        };
    }
    let direction = Vec3::new(
        orbit.yaw.sin() * orbit.pitch.cos(),
        orbit.pitch.sin(),
        orbit.yaw.cos() * orbit.pitch.cos(),
    );
    transform.set_if_neq(if navigation.flat {
        Transform::from_xyz(0.0, 1300.0, 0.1).looking_at(Vec3::ZERO, Vec3::Y)
    } else {
        Transform::from_translation(direction * orbit.distance).looking_at(Vec3::ZERO, Vec3::Y)
    });
}

fn paint_labels(
    view: Res<PrimaryView>,
    ui: Res<ObserverUiState>,
    viewport: Res<ObserverViewport>,
    scale: Res<UiScale>,
    scene: ProductionLabels,
) {
    let ProductionLabels {
        windows,
        camera,
        mut labels,
        mut leaders,
    } = scene;
    let visible = *view == PrimaryView::Production
        && !ui.menu_open
        && !ui.splash_visible
        && !ui.comparison_open;
    let mut order: Vec<_> = labels
        .iter()
        .map(|(entity, label, ..)| (entity, !label.selected, label.site_id.clone()))
        .collect();
    order.sort_by(|left, right| (left.1, &left.2).cmp(&(right.1, &right.2)));
    let mut occupied = Vec::new();
    for (entity, _, _) in order {
        let Ok((_, label, computed, mut node, mut visibility)) = labels.get_mut(entity) else {
            continue;
        };
        let placement = if visible {
            match (camera.single(), windows.single(), viewport.0) {
                (Ok((camera, transform)), Ok(window), Some(bounds)) => {
                    let size = computed.size() / window.scale_factor();
                    camera
                        .world_to_viewport(&GlobalTransform::from(*transform), label.anchor)
                        .ok()
                        .filter(|_| size.x > 0.0 && size.y > 0.0)
                        .and_then(|anchor| {
                            place_label(anchor, bounds, size, &occupied).map(|rect| (anchor, rect))
                        })
                }
                _ => None,
            }
        } else {
            None
        };
        visibility.set_if_neq(if placement.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
        if let Some((_, rect)) = placement {
            let left = px(rect.min.x / scale.0);
            let top = px(rect.min.y / scale.0);
            if node.left != left || node.top != top {
                node.left = left;
                node.top = top;
            }
            occupied.push(rect);
        }
        if let Ok((mut line, mut transform, mut visibility)) = leaders.get_mut(label.leader) {
            visibility.set_if_neq(if placement.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            });
            if let Some((anchor, rect)) = placement {
                let end = anchor.clamp(rect.min, rect.max);
                let delta = (end - anchor) / scale.0;
                let center = (anchor + end) * 0.5 / scale.0;
                let left = px(center.x - delta.length() * 0.5);
                let top = px(center.y - 0.5);
                let width = px(delta.length());
                if line.left != left || line.top != top || line.width != width {
                    line.left = left;
                    line.top = top;
                    line.width = width;
                    line.height = px(1);
                }
                transform.set_if_neq(UiTransform::from_rotation(Rot2::radians(
                    delta.y.atan2(delta.x),
                )));
            }
        }
    }
}

type ReadingFocusTargets<'w, 's> = Query<
    'w,
    's,
    &'static mut ObserverFocusTarget,
    Or<(With<ProductionBrief>, With<ProductionDetails>)>,
>;

fn paint_readings(
    observation: ProductionObservation,
    navigation: Res<ProductionNavigation>,
    mut details: Query<ReadingText, ReadingMarkers>,
    mut reading_targets: ReadingFocusTargets,
    mut last_context: Local<Option<ObservationContext>>,
) {
    let ProductionObservation { frame, state } = observation;
    let context = state.context();
    if navigation.is_changed() || frame.is_changed() || last_context.as_ref() != Some(&context) {
        *last_context = Some(context.clone());
        for mut target in &mut reading_targets {
            let mut next = target.clone();
            next.context = Some(context.clone());
            target.set_if_neq(next);
        }
        let snapshot = frame
            .for_session(&state)
            .and_then(|frame| frame.production.as_ref());
        let site = snapshot.and_then(|snapshot| {
            navigation
                .selected_site
                .as_ref()
                .and_then(|id| snapshot.sites.iter().find(|site| site.id == *id))
        });
        for (mut text, brief, subject, headline) in &mut details {
            if subject.is_some() || headline.is_some() {
                text.0 = match (snapshot, site) {
                    (Some(snapshot), Some(site)) if navigation.details_open => {
                        if subject.is_some() {
                            site.name.trim_end_matches(" cohort").into()
                        } else {
                            reading_headline(site, snapshot, state.viewed_tick)
                        }
                    }
                    _ => String::new(),
                };
                continue;
            }
            text.0 = match (snapshot, site, brief.is_some()) {
                (Some(snapshot), Some(site), true) => describe_brief(site, snapshot),
                (Some(snapshot), Some(site), false) if navigation.details_open => describe(site, snapshot, navigation.reading_section),
                (Some(snapshot), None, true) if navigation.county_open => format!("COUNTY {}\nChoose a commodity cohort to follow its circuit.\n{}", navigation.county_geoid.as_deref().unwrap_or("unavailable"), snapshot.scenario_label),
                (Some(snapshot), None, true) => describe_overview(snapshot),
                (None, _, true) => "No production relationships are disclosed at this period and perspective. Open Geography to explore the information available to you.".into(),
                _ => String::new(),
            };
        }
    }
}

pub struct ProductionPlugin;
impl Plugin for ProductionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PrimaryView>()
            .init_resource::<ProductionNavigation>()
            .init_resource::<ProductionOrbit>()
            .init_resource::<ObserverFeedback>()
            .add_message::<ProductionCommand>()
            .add_observer(keyboard_activate)
            .add_systems(
                PreUpdate,
                focus_eligibility.in_set(ObserverFocusSystems::Eligibility),
            )
            .add_systems(Startup, setup)
            .add_systems(Update, (inputs, orbit_input).in_set(ObserverSet::Input))
            .add_systems(
                Update,
                (
                    invalidate_navigation,
                    navigate,
                    sync_world_county,
                    focus_opening,
                )
                    .chain()
                    .after(ObserverSet::Install)
                    .before(ObserverSet::Paint),
            )
            .add_systems(
                PostUpdate,
                reset_inspector_scroll.before(bevy::ui::UiSystems::Layout),
            )
            .add_systems(
                Update,
                (
                    rebuild,
                    rebuild_dependencies,
                    rebuild_county_cohorts,
                    paint_scene,
                    paint_labels,
                    paint_readings,
                    paint_disclosure,
                    paint_buttons,
                )
                    .chain()
                    .in_set(ObserverSet::Paint),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use babylon_persistence::{
        CampaignId, ObserverEconomySnapshotV1, ObserverVisibilityV1, ProductionInputV1,
        ProductionRouteV2,
    };

    fn site(id: &str, suppliers: &[&str]) -> ProductionSiteV2 {
        ProductionSiteV2 {
            id: id.into(),
            county_geoid: "26163".into(),
            name: format!("Cohort {id}"),
            industry_code: "331".into(),
            observed_employment: Some(20),
            inventory: Vec::new(),
            role: babylon_persistence::ProductionSiteRoleV2::Production,
            sector_code: "31-33".into(),
            processes: vec![babylon_persistence::ProductionProcessV2 {
                id: "fixture-process".into(),
                name: "Fixture process".into(),
                output_good_id: "a".repeat(64),
                output_unit_id: "b".repeat(64),
                output_good: "steel".into(),
                output_unit: "kg".into(),
                output_per_batch: 10,
                available_batches: 8,
                planned_batches: Some(8),
                produced_batches: Some(7),
                labor: Vec::new(),
                inputs: vec![ProductionInputV1 {
                    good_id: "a".repeat(64),
                    unit_id: "b".repeat(64),
                    good: "input".into(),
                    unit: "kg".into(),
                    quantity_per_batch: 1,
                    on_hand: 20,
                    supplier_site_ids: suppliers.iter().map(|id| (*id).into()).collect(),
                }],
            }],
        }
    }

    fn snapshot() -> ProductionSnapshotV2 {
        ProductionSnapshotV2 {
            content_authority_sha256: "a".repeat(64),
            road_source: None,
            physical_edges: Vec::new(),
            merchant_handling_accounts: Vec::new(),
            final_demand_accounts: Vec::new(),
            freight_capacity_accounts: Vec::new(),
            material_balance: None,
            labor_accounts: Vec::new(),
            staffing_accounts: Vec::new(),
            observed_contexts: Vec::new(),
            process_attributions: Vec::new(),
            scenario_label: "Navigation fixture".into(),
            horizon_period: 8,
            sites: vec![
                site("a", &[]),
                site("b", &["a", "withheld"]),
                site("c", &["b"]),
            ],
            routes: vec![ProductionRouteV2 {
                physical_edge_ids: Vec::new(),
                distance_mm: None,
                transport_kind: babylon_persistence::ProductionRouteTransportV2::Staged,
                grams_per_unit: 1000,
                stages: Vec::new(),
                id: "a-b".into(),
                supplier_site_id: "a".into(),
                buyer_site_id: "b".into(),
                good_id: "a".repeat(64),
                unit_id: "b".repeat(64),
                good: "steel".into(),
                unit: "kg".into(),
                travel_periods: 1,
                ordered: 20,
                shipped: 10,
                delivered: 10,
                lost: 0,
                realized: 10,
                backlog: 10,
            }],
            freight: Vec::new(),
            events: Vec::new(),
            provenance: Vec::new(),
        }
    }

    #[test]
    fn focused_circuit_pages_six_incident_groups_and_keeps_unrelated_owners_out() {
        let mut snapshot = snapshot();
        snapshot.sites = vec![site("center", &[])];
        for index in 0..14 {
            snapshot
                .sites
                .push(site(&format!("buyer-{index:02}"), &["center"]));
        }
        snapshot.sites.push(site("unrelated", &[]));
        snapshot.routes.clear();
        let first = ProductionLayout::focused(&snapshot, Some("center"), 0);
        let second = ProductionLayout::focused(&snapshot, Some("center"), 1);
        assert_eq!(first.positions.len(), 7);
        assert_eq!(second.positions.len(), 7);
        assert_eq!(first.links.len(), 6);
        assert_eq!(second.links.len(), 6);
        assert!(first.positions.contains_key("center"));
        assert!(!first.positions.contains_key("unrelated"));
        assert!(first
            .positions
            .keys()
            .filter(|id| id.as_str() != "center")
            .all(|id| !second.positions.contains_key(id)));
        snapshot.sites.reverse();
        let permuted = ProductionLayout::focused(&snapshot, Some("center"), 0);
        assert_eq!(first.positions, permuted.positions);
        assert_eq!(first.links, permuted.links);
        assert!(ProductionLayout::focused(&snapshot, Some("withheld"), 0)
            .positions
            .is_empty());
    }

    #[test]
    fn reading_headline_uses_exact_output_identity_and_keeps_absence_distinct_from_zero() {
        use babylon_persistence::{CompletedMaterialBalanceV2, ProductionMaterialBalanceRowV2};
        let mut snapshot = snapshot();
        let mut selected = snapshot.sites[0].clone();
        selected.processes[0].planned_batches = None;
        selected.processes[0].produced_batches = None;
        let foundation = reading_headline(&selected, &snapshot, 0);
        assert!(foundation.contains("Foundation / Designed"));
        assert!(foundation.contains("no committed production"));
        assert!(foundation.contains("Modeled workforce not disclosed"));
        assert!(!foundation.contains("0 employed"));
        snapshot.material_balance = Some(CompletedMaterialBalanceV2 {
            period: 5,
            rows: vec![ProductionMaterialBalanceRowV2 {
                local_received: 0,
                local_transferred: 0,
                final_demand_fulfilled: 0,
                site_id: selected.id.clone(),
                good_id: selected.processes[0].output_good_id.clone(),
                unit_id: "another-unit".into(),
                good: selected.processes[0].output_good.clone(),
                unit: "other unit".into(),
                opening: 0,
                arrivals: 0,
                produced: 999,
                consumed: 0,
                dispatched: 0,
                closing: 999,
            }],
        });
        assert!(!reading_headline(&selected, &snapshot, 5).contains("999"));
        let row = &mut snapshot.material_balance.as_mut().unwrap().rows[0];
        row.unit_id
            .clone_from(&selected.processes[0].output_unit_id);
        row.unit.clone_from(&selected.processes[0].output_unit);
        row.produced = 0;
        row.closing = 0;
        snapshot
            .staffing_accounts
            .push(staffing_account(&selected.id));
        let completed = reading_headline(&selected, &snapshot, 5);
        assert!(completed.contains(&format!(
            "0 {} produced / Derived",
            selected.processes[0].output_unit
        )));
        assert!(completed.contains("2 employed · 2 reserve / Derived"));
        assert!(!completed.contains("Foundation"));
    }

    #[test]
    fn stock_readings_keep_units_and_subjects_separate_and_do_not_invent_foundation_flows() {
        use babylon_persistence::{CompletedMaterialBalanceV2, ProductionMaterialBalanceRowV2};

        let mut snapshot = snapshot();
        let selected = snapshot.sites[0].clone();
        let mut value = String::new();
        describe_material_balance(&mut value, &selected, &snapshot);
        assert!(value.contains("No completed stock-movement account"));
        assert!(!value.contains("Opened 0"));
        let kilograms = ProductionMaterialBalanceRowV2 {
            local_received: 0,
            local_transferred: 0,
            final_demand_fulfilled: 0,
            site_id: selected.id.clone(),
            good_id: "ore".into(),
            unit_id: "kg".into(),
            good: "Ore".into(),
            unit: "kg".into(),
            opening: 10,
            arrivals: 5,
            produced: 4,
            consumed: 3,
            dispatched: 6,
            closing: 10,
        };
        let tonnes = ProductionMaterialBalanceRowV2 {
            unit_id: "tonne".into(),
            unit: "tonne".into(),
            opening: 1,
            arrivals: 2,
            produced: 0,
            consumed: 0,
            dispatched: 0,
            closing: 3,
            ..kilograms.clone()
        };
        let unrelated = ProductionMaterialBalanceRowV2 {
            local_received: 0,
            local_transferred: 0,
            final_demand_fulfilled: 0,
            site_id: "b".into(),
            good: "Unrelated stock".into(),
            ..kilograms.clone()
        };
        snapshot.material_balance = Some(CompletedMaterialBalanceV2 {
            period: 5,
            rows: vec![kilograms, tonnes, unrelated],
        });
        value.clear();
        describe_material_balance(&mut value, &selected, &snapshot);
        assert!(value.contains("STOCK MOVEMENT / PERIOD 5"));
        assert!(value.contains(
            "Ore / kg\nOpened 10 + arrived 5 + produced 4\n= consumed 3 + dispatched 6 + closed 10"
        ));
        assert!(value.contains(
            "Ore / tonne\nOpened 1 + arrived 2 + produced 0\n= consumed 0 + dispatched 0 + closed 3"
        ));
        assert!(!value.contains("Unrelated stock"));
        value.clear();
        describe_material_balance(&mut value, &snapshot.sites[2], &snapshot);
        assert!(value.contains("No stock-movement account disclosed for this subject"));
        assert!(!value.contains("Opened"));
    }

    #[test]
    fn merchant_reading_has_no_fake_production_and_separates_local_goods_from_arrivals() {
        use babylon_persistence::{
            CompletedMaterialBalanceV2, ProductionMaterialBalanceRowV2, ProductionRouteTransportV2,
            ProductionSiteRoleV2,
        };
        let mut snapshot = snapshot();
        let merchant = &mut snapshot.sites[1];
        merchant.role = ProductionSiteRoleV2::Retail;
        merchant.processes.clear();
        snapshot.routes[0].transport_kind = ProductionRouteTransportV2::Local;
        snapshot.routes[0].travel_periods = 0;
        snapshot.material_balance = Some(CompletedMaterialBalanceV2 {
            period: 1,
            rows: vec![ProductionMaterialBalanceRowV2 {
                site_id: "b".into(),
                good_id: "meal".into(),
                unit_id: "kg".into(),
                good: "Meal".into(),
                unit: "kg".into(),
                opening: 10,
                arrivals: 0,
                local_received: 2,
                produced: 0,
                consumed: 0,
                dispatched: 0,
                local_transferred: 3,
                final_demand_fulfilled: 4,
                closing: 5,
            }],
        });
        let flow = describe_flow(&snapshot.sites[1], &snapshot);
        assert!(flow.contains("Retail / delivery to final demand"));
        assert!(!flow.contains("batch"));
        assert!(flow.contains("arrived 0 + received locally 2"));
        assert!(flow.contains("transferred locally 3 + final demand 4 + closed 5"));
        let freight = describe_freight(&snapshot.sites[1], &snapshot);
        assert!(freight.contains("Local inter-owner transfer"));
        assert!(!freight.contains("periods travel"));
    }

    #[test]
    fn merchant_handling_reading_uses_exact_kilograms_without_changing_work_hours() {
        use babylon_persistence::{
            CompletedProductionMerchantHandlingV2, ProductionMerchantHandlingAccountV2,
            ProductionSiteRoleV2,
        };
        let mut snapshot = snapshot();
        snapshot.sites[1].role = ProductionSiteRoleV2::Retail;
        snapshot.sites[1].processes.clear();
        snapshot.merchant_handling_accounts = vec![ProductionMerchantHandlingAccountV2 {
            site_id: "b".into(),
            capacity_id: "merchant-handling".into(),
            labor_unit_id: "hours".into(),
            coefficients: Vec::new(),
            completed: Some(CompletedProductionMerchantHandlingV2 {
                period: 1,
                needed_hours: 8,
                used_hours: 3,
                handled_grams: 160_001,
                orders: Vec::new(),
            }),
        }];
        let reading = describe_work(&snapshot.sites[1], &snapshot);
        assert!(reading.contains("Period 1: 160.001 kg handled · 3 / 8 labor-hours used / needed"));
        assert!(reading
            .contains("Handling moves existing goods; it does not create productive output."));
        assert!(!describe_work(&snapshot.sites[0], &snapshot).contains("MERCHANT HANDLING"));
    }

    #[test]
    fn inspector_separates_committed_work_time_from_next_opening_and_other_sites() {
        use babylon_persistence::{CompletedProductionLaborV2, ProductionLaborAccountV2};

        let mut snapshot = snapshot();
        snapshot.labor_accounts = vec![
            ProductionLaborAccountV2 {
                site_id: "a".into(),
                unit_id: "hours".into(),
                unit: "labor-hours".into(),
                next_opening_period: 6,
                next_opening_available: 160,
                completed: Some(CompletedProductionLaborV2 {
                    handling_needed: 0,
                    handling_used: 0,
                    period: 5,
                    opening: 120,
                    planned: 100,
                    used: 80,
                    unused: 40,
                }),
            },
            ProductionLaborAccountV2 {
                site_id: "b".into(),
                unit_id: "other-hours".into(),
                unit: "other site's private work time".into(),
                next_opening_period: 6,
                next_opening_available: 987,
                completed: None,
            },
        ];
        let text = describe(
            &snapshot.sites[0],
            &snapshot,
            ProductionReadingSection::Work,
        );
        assert!(text.contains("COMMITTED WORK TIME / PERIOD 5 / DERIVED"));
        assert!(text.contains("80 used + 40 unused = 120 available"));
        assert!(text.contains("Production planned: 100 labor-hours"));
        assert!(text.contains("Next opening (period 6): 160 labor-hours (Derived)"));
        assert!(!text.contains("private work time"));
        assert!(!text.contains("987"));
        assert!(text.contains("Time accounts do not measure job losses."));
    }

    #[test]
    fn foundation_labor_account_does_not_invent_a_completed_work_period() {
        use babylon_persistence::ProductionLaborAccountV2;

        let mut snapshot = snapshot();
        snapshot.labor_accounts = vec![ProductionLaborAccountV2 {
            site_id: "a".into(),
            unit_id: "hours".into(),
            unit: "labor-hours".into(),
            next_opening_period: 1,
            next_opening_available: 120,
            completed: None,
        }];
        let text = describe(
            &snapshot.sites[0],
            &snapshot,
            ProductionReadingSection::Work,
        );
        assert!(!text.contains("COMMITTED WORK TIME"));
        assert!(text.contains("Next opening (period 1): 120 labor-hours (Derived)"));
    }

    fn staffing_account(site_id: &str) -> babylon_persistence::ProductionStaffingAccountV1 {
        use babylon_persistence::{
            CompletedProductionStaffingV1, ProductionStaffingAccountV1, ProductionStaffingSubjectV1,
        };
        ProductionStaffingAccountV1 {
            pool_id: format!("pool-{site_id}"),
            site_id: site_id.into(),
            unit_id: "labor-hours".into(),
            subject: ProductionStaffingSubjectV1 {
                scenario: "fixture".into(),
                local_name: format!("workers-{site_id}"),
            },
            hours_per_person: 40,
            labor_force: 4,
            employed: 2,
            reserve: 2,
            previous_unretained_hours: 40,
            next_opening_period: 6,
            next_opening_hours: 80,
            completed: Some(CompletedProductionStaffingV1 {
                period: 5,
                opening_employed: 4,
                opening_reserve: 0,
                previous_unretained_hours: 80,
                current_unretained_hours: 40,
                retained_hours: 80,
                target_employed: 2,
                hires: 0,
                separations: 2,
            }),
        }
    }

    #[test]
    fn workforce_readings_use_exact_people_and_retention_for_only_the_selected_site() {
        let mut snapshot = snapshot();
        let mut unrelated = staffing_account("b");
        unrelated.employed = 987;
        snapshot.staffing_accounts = vec![staffing_account("a"), unrelated];
        snapshot
            .labor_accounts
            .push(babylon_persistence::ProductionLaborAccountV2 {
                site_id: "a".into(),
                unit_id: "labor-hours".into(),
                unit: "labor-hours".into(),
                next_opening_period: 6,
                next_opening_available: 80,
                completed: Some(babylon_persistence::CompletedProductionLaborV2 {
                    handling_needed: 0,
                    handling_used: 0,
                    period: 5,
                    opening: 160,
                    planned: 40,
                    used: 40,
                    unused: 120,
                }),
            });
        let text = describe(
            &snapshot.sites[0],
            &snapshot,
            ProductionReadingSection::Work,
        );
        assert!(text.contains("MODELED WORKFORCE / DERIVED"));
        assert!(text.contains("2 employed + 2 reserve = 4 people"));
        assert!(text.contains("40 hours per person / period (Designed)"));
        assert!(text.contains("STAFFING / PERIOD 5"));
        assert!(text.contains("Opening: 4 employed, 0 reserve"));
        assert!(text.contains("Hires: 0 | separations: 2 | target: 2 employed"));
        assert!(text.contains("Work request: 40 hours | prior period: 80 hours"));
        assert!(text.contains("One-period retention: 80 hours"));
        assert!(text.contains("Next opening (period 6): 80 labor-hours (Derived)"));
        assert_eq!(text.matches("Next opening").count(), 1);
        assert!(
            text.contains("Observed QCEW jobs are separate; these accounts record no payments.")
        );
        assert!(!text.contains("987"));
        assert!(!text.contains("workers-b"));
    }

    #[test]
    fn workforce_foundation_absence_and_zero_completed_flows_remain_distinct() {
        let mut snapshot = snapshot();
        let mut account = staffing_account("a");
        account.next_opening_period = 1;
        account.completed = None;
        snapshot.staffing_accounts.push(account);
        snapshot
            .labor_accounts
            .push(babylon_persistence::ProductionLaborAccountV2 {
                site_id: "a".into(),
                unit_id: "labor-hours".into(),
                unit: "labor-hours".into(),
                next_opening_period: 1,
                next_opening_available: 80,
                completed: None,
            });
        let foundation = describe(
            &snapshot.sites[0],
            &snapshot,
            ProductionReadingSection::Work,
        );
        assert!(foundation.contains("Opening workforce; no completed staffing period."));
        assert!(foundation.contains("MODELED WORKFORCE / DESIGNED"));
        assert!(!foundation.contains("MODELED WORKFORCE / DERIVED"));
        assert!(!foundation.contains("Hires:"));
        assert!(!foundation.contains("STAFFING / PERIOD"));
        assert!(foundation.contains("Next opening (period 1): 80 labor-hours (Derived)"));
        assert_eq!(foundation.matches("Next opening").count(), 1);
        let missing = describe(
            &snapshot.sites[2],
            &snapshot,
            ProductionReadingSection::Work,
        );
        assert!(missing.contains("No workforce account disclosed for this subject."));
        assert!(!missing.contains("0 employed"));
        let completed = staffing_account("a").completed.unwrap();
        snapshot.staffing_accounts[0].completed =
            Some(babylon_persistence::CompletedProductionStaffingV1 {
                opening_employed: 2,
                opening_reserve: 2,
                hires: 0,
                separations: 0,
                ..completed
            });
        snapshot.staffing_accounts[0].next_opening_period = 6;
        snapshot.labor_accounts[0].next_opening_period = 6;
        snapshot.labor_accounts[0].completed =
            Some(babylon_persistence::CompletedProductionLaborV2 {
                handling_needed: 0,
                handling_used: 0,
                period: 5,
                opening: 80,
                planned: 40,
                used: 40,
                unused: 40,
            });
        let quiet = describe(
            &snapshot.sites[0],
            &snapshot,
            ProductionReadingSection::Work,
        );
        assert!(quiet.contains("Hires: 0 | separations: 0"));
        assert!(!quiet.contains("no completed staffing period"));
        assert!(quiet.contains("Next opening (period 6): 80 labor-hours (Derived)"));
        assert_eq!(quiet.matches("Next opening").count(), 1);
    }

    fn attributed_snapshot() -> ProductionSnapshotV2 {
        use babylon_persistence::{
            ArchiveEvidenceClassV1, DesignedProcessAttributionV1, ObservedSectorContextV2,
            ProductionBusinessSubjectV1,
        };
        let mut snapshot = snapshot();
        let subject = ProductionBusinessSubjectV1 {
            scenario: "observed-fixture".into(),
            local_name: "business-26163-31-33".into(),
        };
        snapshot.observed_contexts.push(ObservedSectorContextV2 {
            subject: subject.clone(),
            county_geoid: "26163".into(),
            sector_code: "31-33".into(),
            sector_title: "Manufacturing".into(),
            vintage: 2024,
            annual_avg_estabs_count: 11,
            annual_avg_emplvl: Some(1_234),
            total_annual_wages: Some(12_345_678),
            annual_avg_wkly_wage: Some(987),
            source_url: "https://www.bls.gov/cew/".into(),
            source_file: "county-source.csv".into(),
            source_sha256: "a".repeat(64),
            artifact_sha256: "b".repeat(64),
            evidence_class: ArchiveEvidenceClassV1::Observed,
        });
        for site in &snapshot.sites[..2] {
            snapshot
                .process_attributions
                .push(DesignedProcessAttributionV1 {
                    process_id: format!("process-{}", site.id),
                    site_id: site.id.clone(),
                    industry_code: site.industry_code.clone(),
                    cohort_subject: subject.clone(),
                    scenario_artifact_sha256: "c".repeat(64),
                    industry_artifact_sha256: "d".repeat(64),
                    evidence_class: ArchiveEvidenceClassV1::Designed,
                });
        }
        snapshot
    }

    #[test]
    fn inspector_distinguishes_shared_sector_context_from_process_workers() {
        let snapshot = attributed_snapshot();
        let text = describe(
            &snapshot.sites[0],
            &snapshot,
            ProductionReadingSection::Sources,
        );
        assert!(text.contains("SECTOR CONTEXT / OBSERVED 2024"));
        assert!(text.contains("Manufacturing | NAICS 31-33"));
        assert_eq!(text.matches("1,234 annual-average jobs").count(), 1);
        assert!(text.contains("USD 12,345,678 annual payroll"));
        assert!(text.contains("USD 987 mean weekly wage"));
        assert!(text.contains("Modeled processes sharing this context: Cohort a; Cohort b"));
        assert!(text.contains("This county-sector total does not assign workers to a process."));
        assert!(!text.contains("2,468"));
        assert!(!describe(
            &snapshot.sites[2],
            &snapshot,
            ProductionReadingSection::Sources
        )
        .contains("SECTOR CONTEXT"));
    }

    #[test]
    fn inspector_keeps_undisclosed_sector_metrics_distinct_from_zero() {
        let mut snapshot = attributed_snapshot();
        let context = &mut snapshot.observed_contexts[0];
        context.annual_avg_emplvl = None;
        context.total_annual_wages = Some(0);
        context.annual_avg_wkly_wage = None;
        let text = describe(
            &snapshot.sites[0],
            &snapshot,
            ProductionReadingSection::Sources,
        );
        assert!(text.contains("Annual-average jobs: not disclosed"));
        assert!(text.contains("USD 0 annual payroll"));
        assert!(text.contains("Mean weekly wage: not disclosed"));
    }

    #[test]
    fn dependency_buttons_deduplicate_real_relations_and_exclude_withheld_endpoints() {
        let snapshot = snapshot();
        let links = dependency_sites(&snapshot.sites[1], &snapshot);
        assert_eq!(
            links
                .into_iter()
                .map(|(direction, site)| (direction, site.id.as_str()))
                .collect::<Vec<_>>(),
            [
                (DependencyDirection::Upstream, "a"),
                (DependencyDirection::Downstream, "c")
            ],
        );
    }

    #[test]
    fn flat_camera_projects_the_scene_and_plinths_inside_its_clip_volume() {
        use bevy::camera::CameraProjection;

        let mut app = App::new();
        app.insert_resource(PrimaryView::Production)
            .insert_resource(ProductionNavigation {
                flat: true,
                ..default()
            })
            .init_resource::<ObserverUiState>()
            .init_resource::<ProductionOrbit>()
            .init_resource::<ObserverViewport>()
            .init_resource::<UiScale>()
            .init_resource::<ObserverFrame>()
            .insert_resource(ObserverSession::new(CampaignId::from_uuid(
                uuid::Uuid::nil(),
            )))
            .add_systems(Update, paint_scene);
        let camera = app
            .world_mut()
            .spawn((
                Camera::default(),
                Transform::default(),
                Projection::Perspective(PerspectiveProjection::default()),
                ProductionCamera,
            ))
            .id();
        app.update();
        let transform = *app.world().get::<Transform>(camera).unwrap();
        let Projection::Orthographic(mut projection) =
            app.world().get::<Projection>(camera).unwrap().clone()
        else {
            panic!("flat view must install an orthographic projection");
        };
        projection.update(934.0, 552.0);
        let clip_from_world = projection.get_clip_from_view() * transform.to_matrix().inverse();
        let mut production = snapshot();
        production.sites.extend([site("d", &[]), site("e", &["d"])]);
        let layout = ProductionLayout::focused(&production, Some("b"), 0);
        let mut points = vec![Vec3::ZERO];
        for position in layout.positions.values() {
            points.extend([*position, *position + Vec3::Y * 110.0]);
        }
        for (center, size) in &layout.platforms {
            for x in [-size.x * 0.5, size.x * 0.5] {
                for z in [-size.y * 0.5, size.y * 0.5] {
                    points.push(*center + Vec3::new(x, -5.0, z));
                }
            }
        }
        for point in points {
            let ndc = clip_from_world.project_point3(point);
            assert!(
                ndc.is_finite()
                    && ndc.x.abs() <= 1.0
                    && ndc.y.abs() <= 1.0
                    && ndc.z > 0.0
                    && ndc.z < 1.0,
                "scene point {point:?} is clipped at {ndc:?}"
            );
        }
    }

    #[test]
    fn labels_remain_inside_the_scene_when_history_reduces_a_small_window() {
        let scene = Rect::new(16.0, 96.0, 965.0, 378.0);
        let size = Vec2::new(180.0, 64.0);
        let position =
            place_label(Vec2::new(955.0, 375.0), scene, size, &[]).expect("visible anchor");
        assert!(scene.contains(position.min));
        assert!(scene.contains(position.max));
        assert!(place_label(Vec2::new(500.0, 420.0), scene, size, &[]).is_none());
        assert!(place_label(Vec2::new(20.0, 100.0), scene, Vec2::splat(1_000.0), &[]).is_none());
    }

    #[test]
    fn open_drawers_block_scene_gestures_without_replaying_them_on_close() {
        use crate::observer_ui::ObserverDisclosure;
        use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};

        let mut app = App::new();
        let mut window = Window::default();
        window.set_cursor_position(Some(Vec2::splat(50.0)));
        let window = app.world_mut().spawn((window, PrimaryWindow)).id();
        app.add_plugins(MinimalPlugins)
            .insert_resource(PrimaryView::Production)
            .insert_resource(ObserverUiState {
                menu_open: false,
                splash_visible: false,
                ..default()
            })
            .insert_resource(ObserverViewport(Some(Rect::new(0.0, 0.0, 200.0, 200.0))))
            .init_resource::<ProductionOrbit>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_message::<MouseMotion>()
            .add_message::<MouseWheel>()
            .add_systems(Update, orbit_input);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        for disclosure in [ObserverDisclosure::Time, ObserverDisclosure::Lens] {
            app.world_mut().resource_mut::<ObserverUiState>().disclosure = Some(disclosure);
            app.world_mut()
                .resource_mut::<Messages<MouseMotion>>()
                .write(MouseMotion {
                    delta: Vec2::new(10.0, 5.0),
                });
            app.world_mut()
                .resource_mut::<Messages<MouseWheel>>()
                .write(MouseWheel {
                    unit: MouseScrollUnit::Line,
                    x: 0.0,
                    y: 1.0,
                    window,
                });
            app.update();
            let orbit = app.world().resource::<ProductionOrbit>();
            assert_eq!(orbit.yaw.to_bits(), 0.0_f32.to_bits());
            assert_eq!(
                orbit.distance.to_bits(),
                ProductionOrbit::default().distance.to_bits()
            );
        }
        app.world_mut().resource_mut::<ObserverUiState>().disclosure = None;
        app.update();
        assert_eq!(
            app.world().resource::<ProductionOrbit>().distance.to_bits(),
            ProductionOrbit::default().distance.to_bits()
        );
        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(MouseWheel {
                unit: MouseScrollUnit::Line,
                x: 0.0,
                y: 1.0,
                window,
            });
        app.update();
        assert_eq!(
            app.world().resource::<ProductionOrbit>().distance.to_bits(),
            (ProductionOrbit::default().distance - 65.0).to_bits()
        );
    }

    #[test]
    fn inspector_scroll_resets_for_subject_or_capability_but_survives_tick_refresh() {
        let mut app = App::new();
        app.insert_resource(ObserverSession::new(CampaignId::from_uuid(
            uuid::Uuid::nil(),
        )))
        .insert_resource(ProductionNavigation {
            selected_site: Some("a".into()),
            ..default()
        })
        .add_systems(Update, reset_inspector_scroll);
        let panel = app
            .world_mut()
            .spawn((ProductionPanel, ScrollPosition::default()))
            .id();
        let other = app
            .world_mut()
            .spawn(ScrollPosition(Vec2::new(0.0, 77.0)))
            .id();
        app.update();
        app.world_mut()
            .entity_mut(panel)
            .get_mut::<ScrollPosition>()
            .unwrap()
            .y = 240.0;
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .ready(1, Some("a".repeat(64)));
        app.world_mut().resource_mut::<ProductionNavigation>().flat = true;
        app.update();
        assert_eq!(
            app.world()
                .get::<ScrollPosition>(panel)
                .unwrap()
                .y
                .to_bits(),
            240.0_f32.to_bits()
        );
        for change in 0..4 {
            app.world_mut()
                .entity_mut(panel)
                .get_mut::<ScrollPosition>()
                .unwrap()
                .y = 180.0;
            match change {
                0 => {
                    app.world_mut()
                        .resource_mut::<ProductionNavigation>()
                        .selected_site = Some("b".into());
                }
                1 => {
                    app.world_mut()
                        .resource_mut::<ObserverSession>()
                        .perspective = crate::observer::Perspective::PlayerKnowledge;
                }
                2 => {
                    app.world_mut().resource_mut::<ObserverSession>().campaign =
                        CampaignId::from_uuid(uuid::Uuid::from_u128(1));
                }
                _ => {
                    app.world_mut()
                        .resource_mut::<ProductionNavigation>()
                        .reading_section = ProductionReadingSection::Work;
                }
            }
            app.update();
            assert_eq!(
                app.world()
                    .get::<ScrollPosition>(panel)
                    .unwrap()
                    .y
                    .to_bits(),
                0.0_f32.to_bits()
            );
            assert_eq!(
                app.world()
                    .get::<ScrollPosition>(other)
                    .unwrap()
                    .y
                    .to_bits(),
                77.0_f32.to_bits()
            );
        }
        app.world_mut()
            .entity_mut(panel)
            .get_mut::<ScrollPosition>()
            .unwrap()
            .y = 120.0;
        app.update();
        assert_eq!(
            app.world()
                .get::<ScrollPosition>(panel)
                .unwrap()
                .y
                .to_bits(),
            120.0_f32.to_bits()
        );
    }

    #[test]
    fn map_and_flat_controls_preserve_the_county_selected_on_geography() {
        let campaign = CampaignId::from_uuid(uuid::Uuid::nil());
        let mut state = ObserverSession::new(campaign);
        state.ready(1, Some("a".repeat(64)));
        let context = state.context();
        let atlas = CountyAtlas::parse(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/map/county_atlas.bin"
        )))
        .expect("atlas");
        let index = |fips: &str| {
            (0..atlas.len())
                .find(|index| atlas.county(*index).is_some_and(|row| row.fips == fips))
                .expect("Michigan county")
        };
        let macomb = index("26099");
        let wayne = index("26163");
        let mut production = snapshot();
        production.sites[1].industry_code = "332".into();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(state)
            .insert_resource(ObserverFrame(Some(ObserverEconomySnapshotV1 {
                campaign_id: campaign.as_uuid().to_string(),
                resolve_tick: 1,
                foundation_digest: "f".repeat(64),
                tick_content_hash: Some("a".repeat(64)),
                nominal_world_hash: None,
                envelope_digest: None,
                visibility: ObserverVisibilityV1::FullObserver,
                counties: Vec::new(),
                production: Some(production),
            })))
            .insert_resource(atlas)
            .insert_resource(PrimaryView::Production)
            .insert_resource(ProductionNavigation {
                selected_site: Some("a".into()),
                ..default()
            })
            .insert_resource(SelectedCounty(Some(macomb)))
            .init_resource::<ObserverFeedback>()
            .insert_resource(ObserverUiState {
                menu_open: false,
                splash_visible: false,
                ..default()
            })
            .add_message::<ProductionCommand>()
            .add_systems(Update, navigate);
        for command in [ProductionCommand::Map, ProductionCommand::Flat] {
            app.world_mut()
                .resource_mut::<Messages<ProductionCommand>>()
                .write(command);
            app.update();
            assert_eq!(app.world().resource::<SelectedCounty>().0, Some(macomb));
            assert_eq!(*app.world().resource::<PrimaryView>(), PrimaryView::Map);
        }
        assert!(app.world().resource::<ProductionNavigation>().flat);
        app.world_mut()
            .resource_mut::<Messages<ProductionCommand>>()
            .write(ProductionCommand::Select {
                site_id: "b".into(),
                context,
            });
        app.update();
        assert_eq!(app.world().resource::<SelectedCounty>().0, Some(wayne));
        assert_eq!(
            *app.world().resource::<PrimaryView>(),
            PrimaryView::Production
        );
        for _ in 0..2 {
            app.world_mut()
                .resource_mut::<Messages<ProductionCommand>>()
                .write(ProductionCommand::Open);
            app.update();
            assert_eq!(
                app.world()
                    .resource::<ProductionNavigation>()
                    .selected_site
                    .as_deref(),
                Some("b"),
                "reopening preserves the chosen industry rather than the county's first site"
            );
            assert_eq!(app.world().resource::<SelectedCounty>().0, Some(wayne));
        }
    }

    #[test]
    fn opening_focus_uses_current_capability_and_preserves_selection() {
        let campaign = CampaignId::from_uuid(uuid::Uuid::nil());
        let mut state = ObserverSession::new(campaign);
        state.ready(1, Some("a".repeat(64)));
        let frame = ObserverFrame(Some(ObserverEconomySnapshotV1 {
            campaign_id: campaign.as_uuid().to_string(),
            resolve_tick: 1,
            foundation_digest: "b".repeat(64),
            nominal_world_hash: None,
            tick_content_hash: Some("a".repeat(64)),
            envelope_digest: None,
            visibility: ObserverVisibilityV1::FullObserver,
            counties: Vec::new(),
            production: Some(snapshot()),
        }));
        let mut app = App::new();
        app.insert_resource(state)
            .insert_resource(frame)
            .insert_resource(PrimaryView::Production)
            .insert_resource(
                CountyAtlas::parse(include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../../assets/map/county_atlas.bin"
                )))
                .expect("atlas"),
            )
            .init_resource::<SelectedCounty>()
            .init_resource::<ProductionNavigation>()
            .add_systems(Update, (invalidate_navigation, focus_opening).chain());
        app.update();
        assert!(app
            .world()
            .resource::<ProductionNavigation>()
            .selected_site
            .is_some());
        app.world_mut()
            .resource_mut::<ProductionNavigation>()
            .selected_site = Some("c".into());
        app.update();
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("c")
        );
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(crate::observer::Perspective::PlayerKnowledge);
        app.update();
        assert!(app
            .world()
            .resource::<ProductionNavigation>()
            .selected_site
            .is_none());
        // Even an installed frame with the old observer payload cannot select
        // a site while the player capability is still loading.
        app.update();
        assert!(app
            .world()
            .resource::<ProductionNavigation>()
            .selected_site
            .is_none());
    }

    #[test]
    fn unchanged_scene_does_not_dirty_camera_or_label_components() {
        #[derive(Resource, Default)]
        struct ChangedCounts([usize; 4]);
        type ChangedSurfaceVisibility = (
            Or<(With<ProductionLabel>, With<ProductionPanel>)>,
            Changed<Visibility>,
        );
        fn count_changes(
            cameras: Query<Entity, (With<ProductionCamera>, Changed<Camera>)>,
            transforms: Query<Entity, (With<ProductionCamera>, Changed<Transform>)>,
            nodes: Query<Entity, (With<ProductionLabel>, Changed<Node>)>,
            visibility: Query<Entity, ChangedSurfaceVisibility>,
            mut counts: ResMut<ChangedCounts>,
        ) {
            counts.0 = [
                cameras.iter().count(),
                transforms.iter().count(),
                nodes.iter().count(),
                visibility.iter().count(),
            ];
        }
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ObserverSession::new(CampaignId::from_uuid(
                uuid::Uuid::nil(),
            )))
            .insert_resource(PrimaryView::Production)
            .init_resource::<ProductionNavigation>()
            .init_resource::<ObserverFrame>()
            .init_resource::<ObserverUiState>()
            .init_resource::<ProductionOrbit>()
            .init_resource::<ObserverViewport>()
            .init_resource::<UiScale>()
            .init_resource::<ChangedCounts>()
            .add_systems(Update, (paint_scene, paint_labels, count_changes).chain());
        app.world_mut().spawn((
            Camera::default(),
            Transform::default(),
            Projection::default(),
            ProductionCamera,
        ));
        app.world_mut()
            .spawn((ProductionPanel, Visibility::Visible, Node::default()));
        let leader = app
            .world_mut()
            .spawn((
                ProductionLeader,
                Node::default(),
                UiTransform::IDENTITY,
                Visibility::Hidden,
            ))
            .id();
        app.world_mut().spawn((
            ProductionLabel {
                anchor: Vec3::ZERO,
                site_id: "a".into(),
                selected: false,
                leader,
            },
            Node::default(),
            Visibility::Visible,
        ));
        app.update();
        app.update();
        assert_eq!(app.world().resource::<ChangedCounts>().0, [0; 4]);
    }

    fn press_site(app: &mut App, id: &str) {
        let world = app.world_mut();
        let entity = world
            .query::<(Entity, &ProductionButton)>()
            .iter(world)
            .find_map(|(entity, button)| match &button.0 {
                ProductionCommand::Select { site_id, .. } if site_id == id => Some(entity),
                _ => None,
            })
            .expect("a visible dependency button");
        world.entity_mut(entity).insert(Interaction::Pressed);
    }

    fn dependency_navigation_app() -> App {
        let mut app = unstarted_dependency_navigation_app();
        app.update();
        app
    }

    fn unstarted_dependency_navigation_app() -> App {
        let campaign = CampaignId::from_uuid(uuid::Uuid::nil());
        let mut state = ObserverSession::new(campaign);
        state.ready(1, Some("a".repeat(64)));
        let frame = ObserverFrame(Some(ObserverEconomySnapshotV1 {
            campaign_id: campaign.as_uuid().to_string(),
            resolve_tick: 1,
            foundation_digest: "f".repeat(64),
            tick_content_hash: Some("a".repeat(64)),
            nominal_world_hash: None,
            envelope_digest: None,
            visibility: ObserverVisibilityV1::FullObserver,
            counties: Vec::new(),
            production: Some(snapshot()),
        }));
        let atlas = CountyAtlas::parse(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/map/county_atlas.bin"
        )))
        .expect("committed atlas");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(state)
            .insert_resource(frame)
            .insert_resource(atlas)
            .init_resource::<PrimaryView>()
            .init_resource::<ProductionNavigation>()
            .init_resource::<SelectedCounty>()
            .init_resource::<ObserverUiState>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ObserverFeedback>()
            .add_message::<ProductionCommand>()
            .add_systems(
                Update,
                (
                    inputs,
                    invalidate_navigation,
                    navigate,
                    sync_world_county,
                    rebuild_dependencies,
                    rebuild_county_cohorts,
                )
                    .chain(),
            );
        app.world_mut().resource_mut::<ObserverUiState>().menu_open = false;
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .splash_visible = false;
        app.world_mut()
            .spawn((Node::default(), ProductionDependencies));
        app
    }

    fn panel_text<T: Component>(app: &mut App) -> String {
        let world = app.world_mut();
        world
            .query_filtered::<&Text, With<T>>()
            .single(world)
            .unwrap()
            .0
            .clone()
    }

    fn production_panel_app() -> App {
        use bevy::ecs::system::RunSystemOnce;

        let mut app = dependency_navigation_app();
        app.insert_resource(PrimaryView::Production);
        app.world_mut()
            .run_system_once(setup)
            .expect("production panel");
        app.add_systems(Update, paint_disclosure.after(navigate));
        app.update();
        app
    }

    fn control_display(app: &mut App, command: &ProductionCommand) -> Display {
        let world = app.world_mut();
        world
            .query::<(&ProductionButton, &Node)>()
            .iter(world)
            .find_map(|(button, node)| {
                (std::mem::discriminant(&button.0) == std::mem::discriminant(command))
                    .then_some(node.display)
            })
            .expect("production control")
    }

    fn send_command(app: &mut App, command: ProductionCommand) {
        app.world_mut()
            .resource_mut::<Messages<ProductionCommand>>()
            .write(command);
        app.update();
    }

    #[test]
    fn production_context_yields_its_rail_to_readings_and_returns_after_close() {
        let mut app = production_panel_app();
        app.init_resource::<ProductionOrbit>()
            .init_resource::<ObserverViewport>()
            .add_systems(Update, paint_scene.after(navigate));
        send_command(&mut app, ProductionCommand::Open);
        let world = app.world_mut();
        let subject = world
            .query_filtered::<Entity, With<ProductionPanel>>()
            .single(world)
            .unwrap();
        assert_eq!(
            world.get::<Node>(subject).unwrap().flex_direction,
            FlexDirection::Column
        );
        assert_eq!(
            *world.get::<Visibility>(subject).unwrap(),
            Visibility::Visible
        );
        send_command(&mut app, ProductionCommand::Details);
        assert_eq!(
            *app.world().get::<Visibility>(subject).unwrap(),
            Visibility::Hidden
        );
        send_command(&mut app, ProductionCommand::Details);
        assert_eq!(
            *app.world().get::<Visibility>(subject).unwrap(),
            Visibility::Visible
        );
    }

    #[test]
    fn expanded_readings_use_the_side_panel_and_yield_to_modal_views() {
        use crate::observer_layout::{ObserverLayout, ObserverRegion};

        let mut app = production_panel_app();
        send_command(&mut app, ProductionCommand::Open);
        send_command(&mut app, ProductionCommand::Details);
        let world = app.world_mut();
        let detail = world
            .query_filtered::<Entity, With<ProductionDetailGroup>>()
            .single(world)
            .expect("one inspector");
        assert!(
            matches!(
                world.get::<ObserverRegion>(detail),
                Some(ObserverRegion::Log)
            ),
            "expanded readings must use the full-height side panel"
        );
        assert!(world.get::<ChildOf>(detail).is_none());
        for size in [Vec2::new(1366.0, 768.0), Vec2::new(1920.0, 1080.0)] {
            let layout = ObserverLayout::new(size, 1.0, false);
            let reading = layout.region(ObserverRegion::Log);
            assert!(reading.height() > 600.0);
            assert!(reading.min.x > layout.world.max.x);
        }
        assert_eq!(world.get::<Node>(detail).unwrap().display, Display::Flex);
        // History owns this same rail while open; closing it restores the
        // current subject's existing reading preference without recreating it.
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .history_open = true;
        app.update();
        assert_eq!(
            app.world().get::<Node>(detail).unwrap().display,
            Display::None
        );
        assert!(app.world().resource::<ProductionNavigation>().details_open);
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .history_open = false;
        app.update();
        assert_eq!(
            app.world().get::<Node>(detail).unwrap().display,
            Display::Flex
        );
        for modal in 0..4 {
            {
                let mut ui = app.world_mut().resource_mut::<ObserverUiState>();
                ui.menu_open = modal == 0;
                ui.archive_open = modal == 1;
                ui.comparison_open = modal == 2;
                ui.splash_visible = modal == 3;
            }
            app.update();
            assert_eq!(
                app.world().get::<Node>(detail).unwrap().display,
                Display::None
            );
        }
        *app.world_mut().resource_mut::<ObserverUiState>() = ObserverUiState {
            menu_open: false,
            splash_visible: false,
            ..default()
        };
        app.update();
        assert_eq!(
            app.world().get::<Node>(detail).unwrap().display,
            Display::Flex
        );
        send_command(&mut app, ProductionCommand::Map);
        assert_eq!(
            app.world().get::<Node>(detail).unwrap().display,
            Display::None
        );
    }

    #[test]
    fn undisclosed_scene_hides_controls_and_explains_keyboard_refusals() {
        let mut app = production_panel_app();
        let full = app.world().resource::<ObserverFrame>().0.clone().unwrap();
        assert_eq!(
            control_display(&mut app, &ProductionCommand::Back),
            Display::None
        );
        assert_eq!(
            control_display(&mut app, &ProductionCommand::Details),
            Display::Flex
        );
        for case in 0..4 {
            let mut frame = full.clone();
            let mut perspective = crate::observer::Perspective::FullObserver;
            match case {
                0 => {}
                1 => {
                    perspective = crate::observer::Perspective::PlayerKnowledge;
                    frame.visibility = ObserverVisibilityV1::KnownPreview;
                    frame.production = None;
                }
                2 => frame.production.as_mut().unwrap().sites.clear(),
                _ => frame.resolve_tick += 1,
            }
            app.world_mut()
                .resource_mut::<ObserverSession>()
                .set_perspective(perspective);
            app.world_mut().resource_mut::<ObserverFrame>().0 = (case != 0).then_some(frame);
            app.update();
            for command in [
                ProductionCommand::Back,
                ProductionCommand::Details,
                ProductionCommand::Flat,
            ] {
                assert_eq!(control_display(&mut app, &command), Display::None);
            }
            for (key, reason) in [
                (
                    KeyCode::Backspace,
                    "There is no previous work view in this observation.",
                ),
                (
                    KeyCode::KeyV,
                    "Display controls need disclosed production relationships.",
                ),
            ] {
                *app.world_mut().resource_mut::<PrimaryView>() = PrimaryView::Production;
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(key);
                app.update();
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .reset_all();
                assert_eq!(
                    app.world()
                        .resource::<crate::observer_ui::ObserverFeedback>()
                        .message,
                    Some(reason)
                );
            }
            assert!(!app.world().resource::<ProductionNavigation>().flat);
            assert!(!app.world().resource::<ProductionNavigation>().details_open);
        }
    }

    #[test]
    fn back_ignores_history_without_a_different_disclosed_destination() {
        let mut app = production_panel_app();
        {
            let mut navigation = app.world_mut().resource_mut::<ProductionNavigation>();
            navigation.selected_site = Some("b".into());
            navigation.history = vec!["b".into(), "undisclosed-site".into()];
        }
        app.update();
        assert_eq!(
            control_display(&mut app, &ProductionCommand::Back),
            Display::None
        );
        send_command(&mut app, ProductionCommand::Back);
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("b")
        );
        assert_eq!(
            app.world().resource::<ObserverFeedback>().message,
            Some("There is no previous work view in this observation.")
        );
    }

    #[test]
    fn scoped_back_and_display_preferences_survive_refresh_without_blocking_recovery() {
        let mut app = production_panel_app();
        let full = app.world().resource::<ObserverFrame>().0.clone();
        let context = app.world().resource::<ObserverSession>().context();
        {
            let mut navigation = app.world_mut().resource_mut::<ProductionNavigation>();
            navigation.selected_site = Some("b".into());
            navigation.flat = true;
            navigation.details_open = true;
        }
        send_command(
            &mut app,
            ProductionCommand::Select {
                site_id: "a".into(),
                context,
            },
        );
        assert_eq!(
            control_display(&mut app, &ProductionCommand::Back),
            Display::Flex
        );
        app.world_mut().resource_mut::<ObserverFrame>().0 = None;
        app.update();
        assert_eq!(
            control_display(&mut app, &ProductionCommand::Details),
            Display::None
        );
        assert!(app.world().resource::<ProductionNavigation>().details_open);
        send_command(&mut app, ProductionCommand::Details);
        assert!(!app.world().resource::<ProductionNavigation>().details_open);
        send_command(&mut app, ProductionCommand::Map);
        assert_eq!(*app.world().resource::<PrimaryView>(), PrimaryView::Map);
        app.world_mut().resource_mut::<ObserverFrame>().0 = full;
        app.update();
        assert_eq!(
            control_display(&mut app, &ProductionCommand::Back),
            Display::Flex
        );
        send_command(&mut app, ProductionCommand::Back);
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("b")
        );
        // Returning through World also supplies the county as a final Back destination.
        assert_eq!(
            control_display(&mut app, &ProductionCommand::Back),
            Display::Flex
        );
        send_command(&mut app, ProductionCommand::Back);
        assert!(app.world().resource::<ProductionNavigation>().county_open);
        assert_eq!(
            control_display(&mut app, &ProductionCommand::Back),
            Display::None
        );
        assert!(app.world().resource::<ProductionNavigation>().flat);
        send_command(&mut app, ProductionCommand::Details);
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(crate::observer::Perspective::PlayerKnowledge);
        app.update();
        let navigation = app.world().resource::<ProductionNavigation>();
        assert!(navigation.flat);
        assert!(!navigation.details_open);
        assert!(navigation.history.is_empty());
        assert!(navigation.selected_site.is_none());
        assert_eq!(
            control_display(&mut app, &ProductionCommand::Back),
            Display::None
        );
    }

    #[test]
    fn accepted_navigation_closes_drawers_but_display_toggles_keep_them() {
        use crate::observer_ui::ObserverDisclosure;

        let mut app = dependency_navigation_app();
        app.world_mut()
            .resource_mut::<ProductionNavigation>()
            .selected_site = Some("b".into());
        let context = app.world().resource::<ObserverSession>().context();
        for command in [
            ProductionCommand::Open,
            ProductionCommand::Map,
            ProductionCommand::Select {
                site_id: "a".into(),
                context,
            },
            ProductionCommand::Back,
        ] {
            app.world_mut().resource_mut::<ObserverUiState>().disclosure =
                Some(ObserverDisclosure::Lens);
            app.world_mut()
                .resource_mut::<Messages<ProductionCommand>>()
                .write(command);
            app.update();
            assert!(app
                .world()
                .resource::<ObserverUiState>()
                .disclosure
                .is_none());
        }
        for command in [ProductionCommand::Flat, ProductionCommand::Details] {
            app.world_mut().resource_mut::<ObserverUiState>().disclosure =
                Some(ObserverDisclosure::Time);
            app.world_mut()
                .resource_mut::<Messages<ProductionCommand>>()
                .write(command);
            app.update();
            assert_eq!(
                app.world().resource::<ObserverUiState>().disclosure,
                Some(ObserverDisclosure::Time)
            );
        }
        assert!(app.world().resource::<ProductionNavigation>().details_open);
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("b")
        );
    }

    #[test]
    fn exact_readings_are_collapsed_until_requested_and_clear_with_capability() {
        use bevy::ecs::system::RunSystemOnce;

        let mut app = dependency_navigation_app();
        *app.world_mut().resource_mut::<PrimaryView>() = PrimaryView::Production;
        app.world_mut()
            .run_system_once(setup)
            .expect("production panel");
        app.add_systems(
            Update,
            (paint_readings, paint_disclosure).chain().after(navigate),
        );
        app.world_mut()
            .resource_mut::<ProductionNavigation>()
            .selected_site = Some("b".into());
        app.update();
        let group = {
            let world = app.world_mut();
            world
                .query_filtered::<Entity, With<ProductionDetailGroup>>()
                .single(world)
                .unwrap()
        };
        assert_eq!(
            app.world().get::<Node>(group).unwrap().display,
            Display::None
        );
        assert!(panel_text::<ProductionBrief>(&mut app).contains("Committed plan partly completed"));
        assert!(panel_text::<ProductionDetails>(&mut app).is_empty());
        assert_eq!(
            panel_text::<ProductionDisclosureLabel>(&mut app),
            "READINGS +"
        );
        app.world_mut()
            .resource_mut::<Messages<ProductionCommand>>()
            .write(ProductionCommand::Details);
        app.update();
        assert!(app.world().resource::<ProductionNavigation>().details_open);
        assert_eq!(
            app.world().get::<Node>(group).unwrap().display,
            Display::Flex
        );
        assert!(panel_text::<ProductionDetails>(&mut app).contains("INPUTS / ON HAND"));
        press_site(&mut app, "a");
        app.update();
        assert!(app.world().resource::<ProductionNavigation>().details_open);
        assert!(panel_text::<ProductionBrief>(&mut app).starts_with("Cohort a"));
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(crate::observer::Perspective::PlayerKnowledge);
        app.update();
        assert!(!app.world().resource::<ProductionNavigation>().details_open);
        assert_eq!(
            app.world().get::<Node>(group).unwrap().display,
            Display::None
        );
        assert!(panel_text::<ProductionDetails>(&mut app).is_empty());
        assert!(!panel_text::<ProductionBrief>(&mut app).contains("Cohort"));
        assert!(panel_text::<ProductionReadingSubject>(&mut app).is_empty());
        assert!(panel_text::<ProductionReadingHeadline>(&mut app).is_empty());
    }

    #[test]
    fn shared_freight_participant_buttons_navigate_and_expire_with_observation_scope() {
        use bevy::ecs::system::RunSystemOnce;
        let mut app = dependency_navigation_app();
        app.world_mut()
            .run_system_once(|mut commands: Commands| setup_readings_panel(&mut commands))
            .unwrap();
        app.add_systems(Update, paint_readings.after(navigate));
        app.world_mut()
            .resource_mut::<ObserverFrame>()
            .0
            .as_mut()
            .unwrap()
            .production = Some(crate::production_freight::tests::fixture());
        let context = app.world().resource::<ObserverSession>().context();
        send_command(
            &mut app,
            ProductionCommand::Select {
                site_id: "panels".into(),
                context: context.clone(),
            },
        );
        let world = app.world_mut();
        let text = world
            .query::<&Text>()
            .iter(world)
            .map(|text| text.0.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(text.matches("Designed regional freight pool").count(), 1);
        assert!(text.contains("OTHER PARTICIPANTS / SHARED FREIGHT"));
        assert!(text.contains("160 kg opening · 160 kg reserved · 0 kg remaining"));
        send_command(
            &mut app,
            ProductionCommand::Reading(ProductionReadingSection::Freight),
        );
        assert!(panel_text::<ProductionDetails>(&mut app)
            .contains("Requested 200 kg | dispatched 40 kg"));
        send_command(&mut app, ProductionCommand::Details);
        press_site(&mut app, "mill");
        app.update();
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("mill")
        );
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(crate::observer::Perspective::PlayerKnowledge);
        send_command(
            &mut app,
            ProductionCommand::Select {
                site_id: "panels".into(),
                context,
            },
        );
        assert!(app
            .world()
            .resource::<ProductionNavigation>()
            .selected_site
            .is_none());
        let world = app.world_mut();
        assert!(!world
            .query::<&Text>()
            .iter(world)
            .any(|text| text.0.contains("Designed regional freight pool")));
    }

    #[test]
    fn keyboard_dependency_activation_uses_the_pointer_queue_and_rejects_changed_scope() {
        let mut app = dependency_navigation_app();
        app.add_observer(keyboard_activate);
        app.world_mut()
            .resource_mut::<ProductionNavigation>()
            .selected_site = Some("b".into());
        app.update();
        let context = app.world().resource::<ObserverSession>().context();
        let button = app
            .world_mut()
            .spawn(button_node(ProductionCommand::Select {
                site_id: "a".into(),
                context: context.clone(),
            }))
            .id();
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity: button,
            context: Some(context.clone()),
        });
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("b"),
            "keyboard activation queues the existing command; it does not mutate navigation in PreUpdate"
        );
        app.update();
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("a")
        );
        assert_eq!(
            app.world().resource::<ProductionNavigation>().history,
            ["b"]
        );
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity: button,
            context: Some(context),
        });
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(crate::observer::Perspective::PlayerKnowledge);
        app.update();
        assert!(app
            .world()
            .resource::<ProductionNavigation>()
            .selected_site
            .is_none());
        assert!(app.world().resource::<ObserverFeedback>().message.is_some());
    }

    #[test]
    fn world_county_list_enters_the_selected_circuit_and_rejects_old_observations() {
        let mut app = dependency_navigation_app();
        app.add_observer(keyboard_activate);
        let county_index = |app: &App, fips: &str| {
            let atlas = app.world().resource::<CountyAtlas>();
            (0..atlas.len())
                .find(|index| atlas.county(*index).unwrap().fips == fips)
                .unwrap()
        };
        let wayne = county_index(&app, "26163");
        app.world_mut().resource_mut::<SelectedCounty>().0 = Some(wayne);
        let root = app
            .world_mut()
            .spawn((Node::default(), ProductionCountyCohorts))
            .id();
        app.update();
        assert_eq!(*app.world().resource::<PrimaryView>(), PrimaryView::Map);
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .county_geoid
                .as_deref(),
            Some("26163")
        );
        let button = {
            let world = app.world_mut();
            world.query::<(Entity, &ProductionButton, &ChildOf)>().iter(world)
                .find_map(|(entity, button, parent)| {
                    (parent.parent() == root && matches!(&button.0, ProductionCommand::Select { site_id, .. } if site_id == "b"))
                        .then_some(entity)
                }).expect("World exposes the real cohort selection control")
        };
        let context = app.world().resource::<ObserverSession>().context();
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity: button,
            context: Some(context.clone()),
        });
        app.update();
        assert_eq!(
            *app.world().resource::<PrimaryView>(),
            PrimaryView::Production
        );
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("b")
        );
        send_command(&mut app, ProductionCommand::Map);
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("b")
        );
        let macomb = county_index(&app, "26099");
        app.world_mut().resource_mut::<SelectedCounty>().0 = Some(macomb);
        app.update();
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .county_geoid
                .as_deref(),
            Some("26099")
        );
        assert!(app
            .world()
            .resource::<ProductionNavigation>()
            .selected_site
            .is_none());
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(crate::observer::Perspective::PlayerKnowledge);
        send_command(
            &mut app,
            ProductionCommand::Select {
                site_id: "b".into(),
                context,
            },
        );
        assert!(app
            .world()
            .resource::<ProductionNavigation>()
            .selected_site
            .is_none());
        assert!(app
            .world()
            .get::<Children>(root)
            .is_none_or(RelationshipTarget::is_empty));
    }

    #[test]
    fn keyboard_pages_the_disclosed_county_then_enters_a_circuit_and_back() {
        let mut app = dependency_navigation_app();
        app.add_observer(keyboard_activate);
        {
            let mut frame = app.world_mut().resource_mut::<ObserverFrame>();
            let snapshot = frame.0.as_mut().unwrap().production.as_mut().unwrap();
            snapshot.sites = (0..14)
                .map(|index| site(&format!("owner-{index:02}"), &[]))
                .collect();
            snapshot.routes.clear();
        }
        {
            let mut navigation = app.world_mut().resource_mut::<ProductionNavigation>();
            navigation.county_geoid = Some("26163".into());
            navigation.county_open = true;
        }
        app.update();
        let context = app.world().resource::<ObserverSession>().context();
        let page_button = {
            let world = app.world_mut();
            world
                .query::<(Entity, &ProductionButton)>()
                .iter(world)
                .find_map(|(entity, button)| {
                    matches!(
                        button.0,
                        ProductionCommand::Page {
                            kind: ProductionPage::Cohorts,
                            page: 1,
                            ..
                        }
                    )
                    .then_some(entity)
                })
                .expect("a real focusable next-page control")
        };
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity: page_button,
            context: Some(context.clone()),
        });
        app.update();
        assert_eq!(
            app.world().resource::<ProductionNavigation>().cohort_page,
            1
        );
        press_site(&mut app, "owner-06");
        app.update();
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("owner-06")
        );
        assert!(!app.world().resource::<ProductionNavigation>().county_open);
        send_command(&mut app, ProductionCommand::Back);
        assert!(app.world().resource::<ProductionNavigation>().county_open);
        assert_eq!(
            app.world().resource::<ProductionNavigation>().cohort_page,
            1
        );
        send_command(
            &mut app,
            ProductionCommand::Page {
                kind: ProductionPage::Cohorts,
                page: 2,
                context,
            },
        );
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(crate::observer::Perspective::PlayerKnowledge);
        app.update();
        assert!(app
            .world()
            .resource::<ProductionNavigation>()
            .county_geoid
            .is_none());
    }

    #[test]
    fn accepted_world_buttons_release_focus_and_focused_keys_do_not_run_world_shortcuts() {
        use crate::observer_focus::{ObserverFocusPlugin, ObserverFocusPolicy};
        use bevy::input::{
            keyboard::{Key, KeyboardInput, NativeKey},
            ButtonState, InputPlugin,
        };
        use bevy::input_focus::InputFocus;
        let mut app = dependency_navigation_app();
        app.add_plugins((InputPlugin, ObserverFocusPlugin))
            .add_observer(keyboard_activate)
            .add_systems(
                PreUpdate,
                focus_eligibility.in_set(ObserverFocusSystems::Eligibility),
            );
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        let context = app.world().resource::<ObserverSession>().context();
        app.world_mut()
            .resource_mut::<ObserverFocusPolicy>()
            .context = Some(context);
        let group = app
            .world_mut()
            .spawn((Node::default(), TabGroup::new(10)))
            .id();
        let button = app
            .world_mut()
            .spawn((button_node(ProductionCommand::Open), ChildOf(group)))
            .id();
        app.update();
        app.world_mut().resource_mut::<InputFocus>().set(button);
        let key = |app: &mut App, key_code, state| {
            app.world_mut().write_message(KeyboardInput {
                key_code,
                logical_key: Key::Unidentified(NativeKey::Unidentified),
                state,
                text: None,
                repeat: false,
                window,
            });
            app.update();
        };
        key(&mut app, KeyCode::KeyP, ButtonState::Pressed);
        assert_eq!(
            *app.world().resource::<PrimaryView>(),
            PrimaryView::Map,
            "focused controls own raw P, so it cannot also open the world"
        );
        key(&mut app, KeyCode::KeyP, ButtonState::Released);
        key(&mut app, KeyCode::Enter, ButtonState::Pressed);
        assert_eq!(
            *app.world().resource::<PrimaryView>(),
            PrimaryView::Production
        );
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(window));
        assert!(app
            .world()
            .resource::<ObserverKeyboardClaim>()
            .claimed(KeyCode::Enter));
        key(&mut app, KeyCode::Enter, ButtonState::Released);
        app.world_mut().resource_mut::<InputFocus>().set(button);
        key(&mut app, KeyCode::Enter, ButtonState::Pressed);
        assert_eq!(
            app.world().resource::<InputFocus>().get(),
            Some(window),
            "WORK releases focus even when work is already the active view"
        );
    }

    struct ReadingsFocusFixture {
        app: App,
        window: Entity,
        body: Entity,
        reading: Entity,
        close: Entity,
        flat: Entity,
        footer: Entity,
    }

    fn readings_focus_app() -> ReadingsFocusFixture {
        use crate::observer_focus::{ObserverFocusPlugin, ObserverFocusPolicy};
        use bevy::ecs::system::RunSystemOnce;
        use bevy::input::InputPlugin;
        let mut app = unstarted_dependency_navigation_app();
        app.add_plugins((InputPlugin, ObserverFocusPlugin))
            .add_observer(keyboard_activate)
            .add_systems(
                PreUpdate,
                focus_eligibility.in_set(ObserverFocusSystems::Eligibility),
            )
            .add_systems(
                Update,
                (paint_readings, paint_disclosure).chain().after(navigate),
            );
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        let context = app.world().resource::<ObserverSession>().context();
        app.world_mut()
            .resource_mut::<ObserverFocusPolicy>()
            .context = Some(context);
        // The real TabNavigationPlugin attaches its window observer in Startup.
        // Install it before the fixture's first update, as the windowed app does.
        app.update();
        *app.world_mut().resource_mut::<PrimaryView>() = PrimaryView::Production;
        {
            let mut navigation = app.world_mut().resource_mut::<ProductionNavigation>();
            navigation.selected_site = Some("b".into());
            navigation.details_open = true;
        }
        app.world_mut()
            .run_system_once(|mut commands: Commands| setup_readings_panel(&mut commands))
            .unwrap();
        let body = app
            .world_mut()
            .query_filtered::<Entity, With<ProductionReadingBody>>()
            .single(app.world())
            .unwrap();
        let reading = app
            .world_mut()
            .query_filtered::<Entity, With<ProductionDetails>>()
            .single(app.world())
            .unwrap();
        let find_button = |world: &mut World, command: fn(&ProductionCommand) -> bool| {
            world
                .query::<(Entity, &ProductionButton)>()
                .iter(world)
                .find_map(|(entity, button)| command(&button.0).then_some(entity))
                .unwrap()
        };
        let close = find_button(app.world_mut(), |command| {
            matches!(command, ProductionCommand::Details)
        });
        let flat = find_button(app.world_mut(), |command| {
            matches!(command, ProductionCommand::Flat)
        });
        let group = app
            .world_mut()
            .spawn((Node::default(), TabGroup::new(40)))
            .id();
        let mut target = ObserverFocusTarget::action(None);
        target.available = true;
        let footer = app
            .world_mut()
            .spawn((Node::default(), target, ChildOf(group)))
            .id();
        // Exercise the actual paging system with explicit overflow geometry;
        // rendering/layout itself belongs to the native-window acceptance check.
        app.world_mut().entity_mut(body).insert((
            ComputedNode {
                size: Vec2::new(300.0, 160.0),
                content_size: Vec2::new(300.0, 1200.0),
                inverse_scale_factor: 1.0,
                ..default()
            },
            ScrollPosition::default(),
        ));
        app.world_mut().entity_mut(reading).insert((
            ComputedNode {
                size: Vec2::new(300.0, 1100.0),
                content_size: Vec2::new(300.0, 1100.0),
                inverse_scale_factor: 1.0,
                ..default()
            },
            UiGlobalTransform::from_translation(Vec2::new(0.0, 470.0)),
        ));
        app.update();
        app.update();
        ReadingsFocusFixture {
            app,
            window,
            body,
            reading,
            close,
            flat,
            footer,
        }
    }

    fn readings_key(app: &mut App, window: Entity, key_code: KeyCode) {
        use bevy::input::{
            keyboard::{Key, KeyboardInput, NativeKey},
            ButtonState,
        };
        for state in [ButtonState::Pressed, ButtonState::Released] {
            app.world_mut().write_message(KeyboardInput {
                key_code,
                logical_key: Key::Unidentified(NativeKey::Unidentified),
                state,
                text: None,
                repeat: false,
                window,
            });
            app.update();
        }
    }

    #[test]
    fn reading_sections_switch_by_keyboard_without_changing_the_observed_period() {
        use bevy::input_focus::InputFocus;
        let ReadingsFocusFixture {
            mut app, window, ..
        } = readings_focus_app();
        let period = app.world().resource::<ObserverSession>().viewed_tick;
        let work = {
            let world = app.world_mut();
            world
                .query::<(&Text, &ChildOf)>()
                .iter(world)
                .find_map(|(text, parent)| (text.0 == "Work").then_some(parent.parent()))
                .expect("a visible Work section control")
        };
        assert!(panel_text::<ProductionDetails>(&mut app).contains("INPUTS / ON HAND"));
        assert!(!panel_text::<ProductionDetails>(&mut app).contains("LABOR BUDGET"));
        app.world_mut().resource_mut::<InputFocus>().set(work);
        readings_key(&mut app, window, KeyCode::Enter);
        let reading = panel_text::<ProductionDetails>(&mut app);
        assert!(reading.contains("LABOR BUDGET / DERIVED"));
        assert!(!reading.contains("INPUTS / ON HAND"));
        assert!(!reading.contains("SECTOR CONTEXT"));
        assert_eq!(
            app.world().resource::<ObserverSession>().viewed_tick,
            period
        );
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("b")
        );
    }

    #[test]
    fn readings_tab_order_reaches_the_text_after_controls_and_pages_its_scroll_ancestor() {
        use bevy::input_focus::InputFocus;
        let ReadingsFocusFixture {
            mut app,
            window,
            body,
            reading,
            close,
            flat,
            footer,
        } = readings_focus_app();
        app.world_mut().resource_mut::<InputFocus>().set(close);
        for section in [
            ProductionReadingSection::Flow,
            ProductionReadingSection::Freight,
            ProductionReadingSection::Work,
            ProductionReadingSection::Sources,
        ] {
            readings_key(&mut app, window, KeyCode::Tab);
            let focused = app.world().resource::<InputFocus>().get().unwrap();
            assert!(
                matches!(&app.world().get::<ProductionButton>(focused).unwrap().0,
                ProductionCommand::Reading(current) if *current == section)
            );
        }
        readings_key(&mut app, window, KeyCode::Tab);
        assert_eq!(
            app.world().resource::<InputFocus>().get(),
            Some(reading),
            "the reading follows its section controls"
        );
        let period = app.world().resource::<ObserverSession>().viewed_tick;
        readings_key(&mut app, window, KeyCode::PageDown);
        assert!(app.world().get::<ScrollPosition>(body).unwrap().0.y > 0.0);
        readings_key(&mut app, window, KeyCode::End);
        assert_eq!(
            app.world()
                .get::<ScrollPosition>(body)
                .unwrap()
                .0
                .y
                .to_bits(),
            1040.0_f32.to_bits()
        );
        readings_key(&mut app, window, KeyCode::Home);
        assert_eq!(
            app.world().get::<ScrollPosition>(body).unwrap().0,
            Vec2::ZERO
        );
        readings_key(&mut app, window, KeyCode::Enter);
        assert_eq!(
            app.world().resource::<ObserverSession>().viewed_tick,
            period
        );
        assert!(app.world().resource::<ProductionNavigation>().details_open);
        readings_key(&mut app, window, KeyCode::Tab);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(flat));
        readings_key(&mut app, window, KeyCode::Tab);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(footer));
    }

    #[test]
    fn shared_freight_reading_enters_tab_order_and_pages_its_scroll_ancestor() {
        use bevy::input_focus::{tab_navigation::TabIndex, InputFocus};
        let ReadingsFocusFixture {
            mut app,
            window,
            footer,
            ..
        } = readings_focus_app();
        app.world_mut()
            .resource_mut::<ObserverFrame>()
            .0
            .as_mut()
            .unwrap()
            .production = Some(crate::production_freight::tests::fixture());
        {
            let mut navigation = app.world_mut().resource_mut::<ProductionNavigation>();
            navigation.selected_site = Some("panels".into());
            navigation.details_open = false;
        }
        let body = app
            .world_mut()
            .query_filtered::<Entity, With<ProductionDependencies>>()
            .single(app.world())
            .unwrap();
        app.world_mut().entity_mut(body).insert((
            Node {
                overflow: Overflow::scroll_y(),
                ..default()
            },
            TabGroup::new(10),
            ComputedNode {
                size: Vec2::new(300.0, 160.0),
                content_size: Vec2::new(300.0, 1200.0),
                inverse_scale_factor: 1.0,
                ..default()
            },
            ScrollPosition::default(),
        ));
        app.update(); // Rebuild the actual shared-pool reading and competitor buttons.
        app.update(); // Ownership must admit the initially unavailable reading.
        let reading = app
            .world_mut()
            .query_filtered::<Entity, With<ProductionFreightReading>>()
            .single(app.world())
            .unwrap();
        let target = app.world().get::<ObserverFocusTarget>(reading).unwrap();
        assert!(target.available);
        assert!(app.world().get::<TabIndex>(reading).is_some());
        app.world_mut().resource_mut::<InputFocus>().set(footer);
        readings_key(&mut app, window, KeyCode::Tab);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(reading));
        readings_key(&mut app, window, KeyCode::PageDown);
        assert!(app.world().get::<ScrollPosition>(body).unwrap().0.y > 0.0);
        let period = app.world().resource::<ObserverSession>().viewed_tick;
        readings_key(&mut app, window, KeyCode::Enter);
        assert_eq!(
            app.world().resource::<ObserverSession>().viewed_tick,
            period
        );
        app.world_mut().resource_mut::<ObserverUiState>().menu_open = true;
        app.update();
        assert!(
            !app.world()
                .get::<ObserverFocusTarget>(reading)
                .unwrap()
                .available
        );
        assert!(app.world().get::<TabIndex>(reading).is_none());
    }

    #[test]
    fn repainted_reading_scope_reenters_tab_order_after_a_committed_period_refresh() {
        use crate::observer_focus::ObserverFocusPolicy;
        use bevy::input_focus::tab_navigation::TabIndex;
        let ReadingsFocusFixture {
            mut app, reading, ..
        } = readings_focus_app();
        let hash = "b".repeat(64);
        {
            let mut session = app.world_mut().resource_mut::<ObserverSession>();
            session.ready(2, Some(hash.clone()));
            let context = session.context();
            assert!(session.installed(&context));
        }
        {
            let mut frame = app.world_mut().resource_mut::<ObserverFrame>();
            let snapshot = frame.0.as_mut().unwrap();
            snapshot.resolve_tick = 2;
            snapshot.tick_content_hash = Some(hash);
        }
        let context = app.world().resource::<ObserverSession>().context();
        app.world_mut()
            .resource_mut::<ObserverFocusPolicy>()
            .context = Some(context.clone());
        app.update(); // old scope is removed, then the reading is repainted
        app.update(); // the repainted target must be admitted without another UI action
        let target = app.world().get::<ObserverFocusTarget>(reading).unwrap();
        assert_eq!(target.context.as_ref(), Some(&context));
        assert!(target.available);
        assert!(app.world().get::<TabIndex>(reading).is_some());
    }

    #[test]
    fn native_dependency_navigation_preserves_back_and_rejects_late_contexts() {
        let mut app = dependency_navigation_app();
        app.world_mut()
            .resource_mut::<ProductionNavigation>()
            .selected_site = Some("b".into());
        app.update();

        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .comparison_open = true;
        press_site(&mut app, "a");
        app.update();
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("b")
        );
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .comparison_open = false;
        app.update();
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("b"),
            "closing the modal does not replay blocked input"
        );
        press_site(&mut app, "a");
        app.update();
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("a")
        );
        assert_eq!(
            app.world().resource::<ProductionNavigation>().history,
            ["b"]
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Backspace);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("b")
        );
        assert!(app
            .world()
            .resource::<ProductionNavigation>()
            .history
            .is_empty());

        press_site(&mut app, "a");
        app.world_mut().resource_mut::<ObserverSession>().generation += 1;
        app.update();
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("b")
        );
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .perspective = crate::observer::Perspective::PlayerKnowledge;
        app.update();
        assert!(app
            .world()
            .resource::<ProductionNavigation>()
            .selected_site
            .is_none());
        let world = app.world_mut();
        assert_eq!(world.query::<&ProductionButton>().iter(world).count(), 0);
    }
    #[test]
    fn topology_uses_disclosed_endpoints_and_survives_input_reordering() {
        let mut snapshot = snapshot();
        let mut hidden = snapshot.routes[0].clone();
        hidden.id = "hidden-route".into();
        hidden.buyer_site_id = "withheld".into();
        snapshot.routes.push(hidden);
        let original = ProductionLayout::focused(&snapshot, Some("b"), 0);
        assert_eq!(original.positions.len(), 3);
        assert_eq!(
            original.links,
            [("a".into(), "b".into()), ("b".into(), "c".into())]
        );
        assert!(original.positions["a"].x < original.positions["b"].x);
        assert!(original.positions["b"].x < original.positions["c"].x);
        snapshot.sites.reverse();
        snapshot.routes.reverse();
        for site in &mut snapshot.sites {
            for input in &mut site.processes[0].inputs {
                input.supplier_site_ids.reverse();
            }
        }
        let reordered = ProductionLayout::focused(&snapshot, Some("b"), 0);
        assert_eq!(original.positions, reordered.positions);
        assert_eq!(original.links, reordered.links);
        snapshot.sites.retain(|site| site.id != "a");
        let scoped = ProductionLayout::focused(&snapshot, Some("b"), 0);
        assert_eq!(scoped.links, [("b".into(), "c".into())]);
        assert!(!scoped.positions.contains_key("a"));
        assert!(!scoped.positions.contains_key("withheld"));
    }

    #[test]
    fn only_actual_visible_in_transit_lots_get_static_markers() {
        use babylon_persistence::ProductionFreightV2;

        let mut snapshot = snapshot();
        let layout = ProductionLayout::focused(&snapshot, Some("b"), 0);
        assert!(
            freight_markers(&snapshot, &layout, 1).is_empty(),
            "orders and deliveries alone must not generate freight"
        );
        let lot = ProductionFreightV2 {
            current_stage_index: 0,
            grams_per_unit: 1000,
            mass_grams: 1000,
            id: "actual-lot".into(),
            route_id: "a-b".into(),
            source_site_id: "a".into(),
            destination_site_id: "b".into(),
            good_id: "a".repeat(64),
            unit_id: "b".repeat(64),
            good: "steel".into(),
            unit: "kg".into(),
            quantity: 10,
            dispatch_period: 1,
            arrival_period: 4,
        };
        snapshot.freight.push(lot.clone());
        let first = freight_markers(&snapshot, &layout, 1);
        assert_eq!(first.len(), 1);
        assert_eq!(
            first,
            freight_markers(&snapshot, &layout, 2),
            "schematic markers do not invent continuous travel motion"
        );
        assert!(freight_markers(&snapshot, &layout, 0).is_empty());
        assert!(freight_markers(&snapshot, &layout, 4).is_empty());
        for case in 0..4 {
            let mut withheld = lot.clone();
            match case {
                0 => withheld.quantity = 0,
                1 => withheld.route_id = "unavailable-route".into(),
                2 => withheld.destination_site_id = "withheld".into(),
                _ => withheld.good_id = "another-good".into(),
            }
            snapshot.freight = vec![withheld];
            assert!(freight_markers(&snapshot, &layout, 1).is_empty());
        }
    }
}
