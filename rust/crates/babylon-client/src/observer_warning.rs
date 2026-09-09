//! An explicit, untimed opening warning for the native observer.

use bevy::input_focus::{tab_navigation::TabGroup, InputFocus};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::observer_focus::{ObserverFocusSystems, ObserverFocusTarget, ObserverKeyboardActivate};
use crate::observer_io::ObserverSet;
use crate::observer_theme as theme;
use crate::observer_ui::{ObserverCommand, ObserverFontRole, ObserverUiState};

const FICTION: &str = "Babylon is a work of fiction built from real places, records and material relationships. Some of it may feel uncomfortably familiar.";
const EMERGENCE: &str = "PROCEDURALLY EMERGENT. Most of your experience comes from game mechanics. It may behave in unexpected ways.";
const THEORY: &str = "The game draws on Marxist theory. It may hit close to home. It makes no claim about what you should or should not do. This is not an oracle. There is no computational scrying behind the curtain.";
const OBSERVER: &str = "THIS BUILD: OBSERVER ONLY. Watch the simulation. Trace its dependencies. Player interventions are unavailable.";
const AI: &str = "The planned full game will offer optional AI narration: local models first, or a service you configure. Playing without it will reduce narrative detail; the simulation will still run. AI narration is not connected in this observer build.";

#[derive(Component)]
pub(crate) struct ObserverWarningRoot;
#[derive(Component)]
struct WarningHeadline;
#[derive(Component)]
struct WarningReading;
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum WarningAction {
    Continue,
    Quit,
}
#[derive(Message)]
struct ContinueWarning;

fn text(value: &str, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
        ObserverFontRole::Body,
        TextLayout::new_with_linebreak(bevy::text::LineBreak::WordOrCharacter),
        Node {
            max_width: percent(100),
            min_width: px(0),
            flex_shrink: 0.0,
            ..default()
        },
        DeclaredSurface::new(SurfaceId::TitleLockup),
    )
}

fn spawn_warning(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                padding: UiRect::all(px(28)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(14),
                ..default()
            },
            BackgroundColor(theme::INK),
            ZIndex(100),
            TabGroup::modal(),
            ObserverWarningRoot,
            DeclaredSurface::new(SurfaceId::TitleLockup),
        ))
        .with_children(|screen| {
            screen
                .spawn((text("WARNING", 104.0, theme::PAPER), WarningHeadline))
                .insert(ObserverFontRole::Display);
            screen.spawn(text(
                "THIS FICTION IS BUILT FROM REALITY.",
                20.0,
                theme::YELLOW,
            ));
            screen.spawn((
                Node {
                    width: percent(100),
                    max_width: px(1040),
                    height: px(3),
                    flex_shrink: 0.0,
                    ..default()
                },
                BackgroundColor(theme::RED),
                DeclaredSurface::new(SurfaceId::TitleLockup),
            ));
            screen
                .spawn((
                    Node {
                        width: percent(100),
                        max_width: px(1040),
                        min_height: px(0),
                        flex_grow: 1.0,
                        overflow: Overflow::scroll_y(),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(15),
                        padding: UiRect::axes(px(4), px(8)),
                        ..default()
                    },
                    ScrollPosition::default(),
                    DeclaredSurface::new(SurfaceId::TitleLockup),
                ))
                .with_children(|body| {
                    body.spawn((
                        text(FICTION, 18.0, theme::PAPER),
                        WarningReading,
                        ObserverFocusTarget::reading(None),
                    ));
                    body.spawn(text(EMERGENCE, 18.0, theme::PAPER));
                    body.spawn(text(THEORY, 18.0, theme::PAPER));
                    body.spawn(text(OBSERVER, 16.0, theme::YELLOW));
                    body.spawn(text(AI, 16.0, theme::PAPER));
                });
            screen
                .spawn((
                    Node {
                        width: percent(100),
                        max_width: px(1040),
                        flex_shrink: 0.0,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        column_gap: px(20),
                        ..default()
                    },
                    DeclaredSurface::new(SurfaceId::TitleLockup),
                ))
                .with_children(|footer| {
                    action_button(footer, WarningAction::Quit, "Quit [Q]");
                    action_button(footer, WarningAction::Continue, "CONTINUE [ENTER]");
                });
        });
}

fn action_button(parent: &mut ChildSpawnerCommands, action: WarningAction, caption: &str) {
    let (background, foreground) = if action == WarningAction::Continue {
        (theme::PAPER, theme::INK)
    } else {
        (theme::INK, theme::PAPER)
    };
    parent
        .spawn((
            Button,
            action,
            Node {
                min_height: px(48),
                padding: UiRect::axes(px(24), px(12)),
                border: UiRect::bottom(px(2)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(background),
            BorderColor::all(theme::PAPER),
            ObserverFocusTarget::action(None),
            DeclaredSurface::new(SurfaceId::TitleLockup),
        ))
        .observe(pointer_action)
        .with_child(text(caption, 18.0, foreground));
}

fn request_action(
    action: WarningAction,
    ui: &ObserverUiState,
    continues: &mut MessageWriter<ContinueWarning>,
    commands: &mut MessageWriter<ObserverCommand>,
) {
    if !ui.splash_visible {
        return;
    }
    match action {
        WarningAction::Continue => {
            continues.write(ContinueWarning);
        }
        WarningAction::Quit => {
            commands.write(ObserverCommand::Quit);
        }
    }
}
fn pointer_action(
    event: On<Pointer<Click>>,
    actions: Query<&WarningAction>,
    ui: Res<ObserverUiState>,
    mut continues: MessageWriter<ContinueWarning>,
    mut commands: MessageWriter<ObserverCommand>,
) {
    if event.button != bevy::picking::pointer::PointerButton::Primary {
        return;
    }
    if let Ok(action) = actions.get(event.entity) {
        request_action(*action, &ui, &mut continues, &mut commands);
    }
}
fn focused_action(
    event: On<ObserverKeyboardActivate>,
    actions: Query<&WarningAction>,
    ui: Res<ObserverUiState>,
    mut continues: MessageWriter<ContinueWarning>,
    mut commands: MessageWriter<ObserverCommand>,
) {
    if event.context.is_some() {
        return;
    }
    if let Ok(action) = actions.get(event.entity) {
        request_action(*action, &ui, &mut continues, &mut commands);
    }
}
fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    focus: Option<Res<InputFocus>>,
    actions: Query<&WarningAction>,
    ui: Res<ObserverUiState>,
    mut continues: MessageWriter<ContinueWarning>,
    mut commands: MessageWriter<ObserverCommand>,
) {
    if !ui.splash_visible {
        return;
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        request_action(WarningAction::Quit, &ui, &mut continues, &mut commands);
    } else if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        // A focused Quit button owns Enter through the normal focus dispatcher.
        let quit_focused = focus
            .as_ref()
            .and_then(|focus| focus.get())
            .and_then(|entity| actions.get(entity).ok())
            == Some(&WarningAction::Quit);
        if !quit_focused {
            request_action(WarningAction::Continue, &ui, &mut continues, &mut commands);
        }
    }
}

fn apply_continue(mut continues: MessageReader<ContinueWarning>, mut ui: ResMut<ObserverUiState>) {
    if continues.read().count() > 0 && ui.splash_visible {
        ui.splash_visible = false;
        ui.menu_open = true;
    }
}

type WarningFocusTargets<'w, 's> = Query<
    'w,
    's,
    &'static mut ObserverFocusTarget,
    Or<(With<WarningAction>, With<WarningReading>)>,
>;

fn eligibility(ui: Res<ObserverUiState>, mut targets: WarningFocusTargets) {
    for mut target in &mut targets {
        if target.available != ui.splash_visible {
            target.available = ui.splash_visible;
        }
    }
}
fn visibility(
    ui: Res<ObserverUiState>,
    mut roots: Query<&mut Visibility, With<ObserverWarningRoot>>,
) {
    let next = if ui.splash_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut root in &mut roots {
        root.set_if_neq(next);
    }
}
fn layout(
    windows: Query<&Window, With<PrimaryWindow>>,
    scale: Res<UiScale>,
    mut roots: Query<&mut Node, With<ObserverWarningRoot>>,
    mut titles: Query<&mut TextFont, With<WarningHeadline>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let logical_height = window.height() / scale.0;
    let headline_size = (logical_height * 0.15).clamp(64.0, 120.0);
    let padding = (logical_height * 0.035).clamp(16.0, 32.0);
    for mut title in &mut titles {
        if title.font_size.to_bits() != headline_size.to_bits() {
            title.font_size = headline_size;
        }
    }
    for mut root in &mut roots {
        let next = UiRect::all(px(padding));
        if root.padding != next {
            root.padding = next;
        }
    }
}
fn paint_buttons(
    mut buttons: Query<(&Interaction, &WarningAction, &mut BorderColor), Changed<Interaction>>,
) {
    for (interaction, _, mut border) in &mut buttons {
        border.set_if_neq(BorderColor::all(match interaction {
            Interaction::Pressed => theme::RED,
            Interaction::Hovered => theme::YELLOW,
            Interaction::None => theme::PAPER,
        }));
    }
}

pub(crate) struct ObserverWarningPlugin;
impl Plugin for ObserverWarningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObserverUiState>()
            .init_resource::<UiScale>()
            .add_message::<ContinueWarning>()
            .add_message::<ObserverCommand>()
            .add_observer(focused_action)
            .add_systems(Startup, spawn_warning)
            .add_systems(
                PreUpdate,
                eligibility.in_set(ObserverFocusSystems::Eligibility),
            )
            .add_systems(Update, keyboard.in_set(ObserverSet::Input))
            // Keep the warning closed to gameplay for this entire input/commit frame.
            .add_systems(
                Update,
                (apply_continue, visibility, layout, paint_buttons)
                    .chain()
                    .after(ObserverSet::Install)
                    .before(ObserverSet::Paint),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::{
        keyboard::{Key, KeyboardInput, NativeKey},
        ButtonState, InputPlugin,
    };
    use bevy::picking::{
        backend::HitData,
        pointer::{Location, PointerButton, PointerId},
    };
    use std::time::Duration;

    #[derive(Resource, Default)]
    struct InstallTrace(Vec<bool>);

    fn app(width: u32, height: u32) -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, InputPlugin, ObserverWarningPlugin))
            .init_resource::<InstallTrace>()
            .configure_sets(
                Update,
                (
                    ObserverSet::Input,
                    ObserverSet::Receive,
                    ObserverSet::Install,
                    ObserverSet::Paint,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (|ui: Res<ObserverUiState>, mut trace: ResMut<InstallTrace>| {
                    trace.0.push(ui.splash_visible);
                })
                .in_set(ObserverSet::Install),
            );
        let window = app
            .world_mut()
            .spawn((
                Window {
                    resolution: (width, height).into(),
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        app.update();
        (app, window)
    }
    fn key(app: &mut App, window: Entity, key_code: KeyCode) {
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
    fn button(app: &mut App, action: WarningAction) -> Entity {
        let world = app.world_mut();
        world
            .query::<(Entity, &WarningAction)>()
            .iter(world)
            .find_map(|(entity, candidate)| (*candidate == action).then_some(entity))
            .unwrap()
    }

    #[test]
    fn warning_has_no_timer_and_only_explicit_continue_dismisses_it() {
        let (mut app, window) = app(1366, 768);
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs(60),
        ));
        app.update();
        assert!(app.world().resource::<ObserverUiState>().splash_visible);
        key(&mut app, window, KeyCode::Space);
        assert!(app.world().resource::<ObserverUiState>().splash_visible);
        key(&mut app, window, KeyCode::Enter);
        assert!(!app.world().resource::<ObserverUiState>().splash_visible);
        assert!(app.world().resource::<ObserverUiState>().menu_open);
        assert_eq!(
            app.world().resource::<InstallTrace>().0.last(),
            Some(&true),
            "the dismissal frame must stay blocked through playback/install"
        );
        assert!(app
            .world()
            .resource::<Messages<ObserverCommand>>()
            .is_empty());
    }

    #[test]
    fn real_continue_click_queues_dismissal_without_leaking_a_gameplay_command() {
        let (mut app, _) = app(1366, 768);
        let entity = button(&mut app, WarningAction::Continue);
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
            entity,
        ));
        assert!(app.world().resource::<ObserverUiState>().splash_visible);
        app.update();
        assert!(!app.world().resource::<ObserverUiState>().splash_visible);
        assert_eq!(app.world().resource::<InstallTrace>().0.last(), Some(&true));
        assert!(app
            .world()
            .resource::<Messages<ObserverCommand>>()
            .is_empty());
    }

    #[test]
    fn quit_remains_available_and_does_not_acknowledge_the_warning() {
        let (mut app, window) = app(1366, 768);
        key(&mut app, window, KeyCode::KeyQ);
        assert!(app.world().resource::<ObserverUiState>().splash_visible);
        let mut reader = app
            .world()
            .resource::<Messages<ObserverCommand>>()
            .get_cursor();
        let commands: Vec<_> = reader
            .read(app.world().resource::<Messages<ObserverCommand>>())
            .copied()
            .collect();
        assert_eq!(commands, [ObserverCommand::Quit]);
    }

    #[test]
    fn focused_continue_uses_the_same_deferred_action_and_rejects_dynamic_scope() {
        let (mut app, _) = app(1366, 768);
        let entity = button(&mut app, WarningAction::Continue);
        let session = crate::observer::ObserverSession::new(
            babylon_persistence::CampaignId::from_uuid(uuid::Uuid::nil()),
        );
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity,
            context: Some(session.context()),
        });
        app.update();
        assert!(app.world().resource::<ObserverUiState>().splash_visible);
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity,
            context: None,
        });
        assert!(app.world().resource::<ObserverUiState>().splash_visible);
        app.update();
        assert!(!app.world().resource::<ObserverUiState>().splash_visible);
    }

    #[test]
    fn supported_window_sizes_keep_text_scrollable_and_controls_outside_the_scroll_body() {
        for (width, height) in [(1366, 768), (1920, 1080)] {
            let (mut app, _) = app(width, height);
            app.world_mut().resource_mut::<UiScale>().0 = 1.15;
            app.update();
            let world = app.world_mut();
            let title = world
                .query_filtered::<&TextFont, With<WarningHeadline>>()
                .single(world)
                .unwrap();
            assert!((64.0..=120.0).contains(&title.font_size));
            let reading = world
                .query_filtered::<&ChildOf, With<WarningReading>>()
                .single(world)
                .unwrap();
            let body = world.get::<Node>(reading.parent()).unwrap();
            assert_eq!(body.overflow, Overflow::scroll_y());
            assert_eq!(body.min_height, px(0));
            for (_, parent) in world.query::<(&WarningAction, &ChildOf)>().iter(world) {
                let footer = world.get::<Node>(parent.parent()).unwrap();
                assert_eq!(footer.flex_shrink.to_bits(), 0.0_f32.to_bits());
                assert_ne!(footer.overflow, Overflow::scroll_y());
            }
        }
    }
}
