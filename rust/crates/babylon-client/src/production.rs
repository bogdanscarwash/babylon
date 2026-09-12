//! A read-only production scene: exact receipts, cohort columns and actual lots.
//! County aggregates have no invented geographic placement.

use babylon_persistence::{
    production_observation::ProductionSite, production_observation::ProductionSnapshot,
};
use bevy::camera::{visibility::RenderLayers, ScalingMode, Viewport};
use bevy::ecs::system::SystemParam;
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::light::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::observer::{ObservationContext, ObserverSession};
use crate::observer_focus::{
    ObserverFocusSystems, ObserverFocusTarget, ObserverKeyboardActivate, ObserverKeyboardClaim,
};
use crate::observer_io::ObserverSet;
use crate::observer_theme as theme;
use crate::observer_ui::{
    grouped, ObserverFeedback, ObserverFrame, ObserverUiState, ObserverViewport,
};
use crate::production_brief::{
    committed_plan_status, dependency_flow_summary, dependency_sites, describe_brief,
    describe_overview, DependencyDirection,
};
use crate::production_freight::{account_brief, competitor_sites, shared_accounts};
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
    Focus {
        site_id: String,
        context: ObservationContext,
    },
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
        | ProductionCommand::Focus { context, .. }
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

type InspectorScrolls<'w, 's> = Query<
    'w,
    's,
    &'static mut ScrollPosition,
    Or<(With<ProductionPanel>, With<ProductionReadingBody>)>,
>;

#[derive(PartialEq, Eq)]
struct InspectorScrollScope {
    campaign: babylon_persistence::identity::CampaignId,
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
    snapshot: &ProductionSnapshot,
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
    site: &ProductionSite,
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
    snapshot: &ProductionSnapshot,
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
    snapshot: &ProductionSnapshot,
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
    snapshot: &ProductionSnapshot,
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
    site: &ProductionSite,
    snapshot: &ProductionSnapshot,
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
            ProductionCommand::Select { site_id, .. }
            | ProductionCommand::Focus { site_id, .. } => {
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
mod tests;

pub(crate) mod navigation;
mod readings;

use navigation::{
    focus_opening, invalidate_navigation, navigate, readings_panel_visible, sync_world_county,
    ProductionControlAvailability,
};
use readings::{describe, reading_headline};
