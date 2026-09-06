//! Bounded, capability-scoped history. Charts never recompute a transition.

mod delivery_groups;

use babylon_persistence::{ObserverEconomyReaderV1, ProductionEventV1, ProductionSiteV1};
use bevy::ecs::{query::QueryData, system::SystemParam};
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};

use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::observer::{ObservationContext, ObserverSession, Perspective};
use crate::observer_calendar::CampaignMonth;
use crate::observer_controls::{inspection_availability, ControlAvailability};
use crate::observer_focus::{ObserverFocusSystems, ObserverFocusTarget, ObserverKeyboardActivate};
use crate::observer_io::ObserverSet;
use crate::observer_layout::ObserverRegion;
use crate::observer_theme as theme;
use crate::observer_ui::{grouped, ObserverFeedback, ObserverFrame, ObserverUiState};
use crate::production::{
    readings_panel_visible, PrimaryView, ProductionCommand, ProductionNavigation,
};
use crate::ui::dossier_card::DossierRefresh;
use delivery_groups::{
    delivery_log_entries, DeliveryGroup, DeliveryGroupKey, DeliveryLog, DeliveryLogEntry,
};

const CHART_WEEKS: u64 = 12;
const LOG_ENTRIES: usize = 160;

#[derive(Clone, Debug, PartialEq, Eq)]
struct HistoryScope {
    context: ObservationContext,
    site: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WeeklyOutput {
    week: u64,
    planned: Option<u64>,
    produced: Option<u64>,
}

impl WeeklyOutput {
    fn from_site(week: u64, site: &ProductionSiteV1) -> Result<Self, String> {
        let quantity = |batches: Option<u64>| {
            batches
                .map(|batches| {
                    batches.checked_mul(site.output_per_batch).ok_or_else(|| {
                        "Output quantity exceeds its exact integer range.".to_owned()
                    })
                })
                .transpose()
        };
        Ok(Self {
            week,
            planned: quantity(site.planned_batches)?,
            produced: quantity(site.produced_batches)?,
        })
    }
}

type HistoryTask = Task<Result<Vec<WeeklyOutput>, String>>;

#[derive(Resource, Default)]
struct HistoryState {
    scope: Option<HistoryScope>,
    pending: Option<HistoryTask>,
    points: Vec<WeeklyOutput>,
    error: Option<String>,
    selected_event: Option<(ObservationContext, ProductionEventV1)>,
    focus_selected_event: bool,
    expanded_delivery: Option<(ObservationContext, DeliveryGroupKey)>,
}

#[derive(Component)]
struct HistoryPanel;
#[derive(Component)]
struct LogPanel;
#[derive(Component)]
struct HistoryHint;
#[derive(Resource, Default)]
struct HistoryKeyboardActions(Vec<HistoryButton>);
#[derive(Component, Clone)]
enum HistoryButton {
    Week {
        context: ObservationContext,
        week: u64,
    },
    Event {
        context: ObservationContext,
        event: ProductionEventV1,
    },
    DeliveryEvidence {
        context: ObservationContext,
        group: DeliveryGroupKey,
    },
}

fn label(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
        crate::observer_ui::ObserverFontRole::Body,
        Node {
            flex_shrink: 0.0,
            min_width: px(0),
            max_width: percent(100),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::ObserverProduction),
    )
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            padding: UiRect::all(px(12)),
            row_gap: px(8),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
            border: UiRect::top(px(2)),
            ..default()
        },
        BackgroundColor(theme::INK),
        BorderColor::all(theme::YELLOW),
        ZIndex(9),
        Visibility::Hidden,
        HistoryPanel,
        TabGroup::new(30),
        ObserverRegion::History,
        DeclaredSurface::new(SurfaceId::ObserverProduction),
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            padding: UiRect::all(px(16)),
            row_gap: px(12),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
            border: UiRect::left(px(2)),
            ..default()
        },
        BackgroundColor(theme::INK),
        BorderColor::all(theme::PAPER.with_alpha(0.6)),
        ZIndex(6),
        LogPanel,
        TabGroup::new(20),
        Visibility::Visible,
        ObserverRegion::Log,
        DeclaredSurface::new(SurfaceId::ObserverProduction),
    ));
}

fn button_availability(
    button: &HistoryButton,
    session: &ObserverSession,
    frame: &ObserverFrame,
    delivery_log: Option<&DeliveryLog<'_>>,
) -> ControlAvailability {
    use ControlAvailability::{Disabled, Enabled};
    let (context, week) = match button {
        HistoryButton::Week { context, week } => (context, *week),
        HistoryButton::Event { context, event } => (context, event.week),
        HistoryButton::DeliveryEvidence { context, group } => (context, group.week),
    };
    if !session.accepts(context) {
        return Disabled("History changed; wait for its current observation");
    }
    let available = inspection_availability(session);
    if available != Enabled {
        return available;
    }
    if week > session.durable_tick {
        return Disabled("That week has not committed");
    }
    match button {
        HistoryButton::Week { .. } if week == session.viewed_tick => {
            Disabled("Already viewing this committed week")
        }
        HistoryButton::Event { event, .. } => {
            let valid = frame
                .for_session(session)
                .and_then(|frame| frame.production.as_ref())
                .is_some_and(|snapshot| snapshot.events.iter().any(|current| current == event));
            if valid {
                Enabled
            } else {
                Disabled("This event is unavailable in the current observation")
            }
        }
        HistoryButton::Week { .. } => Enabled,
        HistoryButton::DeliveryEvidence { group, .. } => {
            if delivery_log.is_some_and(|log| log.contains_group(group)) {
                Enabled
            } else {
                Disabled("This delivery group is unavailable in the current observation")
            }
        }
    }
}

#[derive(SystemParam)]
struct HistoryInput<'w> {
    session: ResMut<'w, ObserverSession>,
    frame: Res<'w, ObserverFrame>,
    history: ResMut<'w, HistoryState>,
    ui: Res<'w, ObserverUiState>,
    refresh: ResMut<'w, DossierRefresh>,
    feedback: ResMut<'w, ObserverFeedback>,
    time: Res<'w, Time>,
    navigation: Res<'w, ProductionNavigation>,
    view: Res<'w, PrimaryView>,
}

fn history_button_context(button: &HistoryButton) -> &ObservationContext {
    match button {
        HistoryButton::Week { context, .. }
        | HistoryButton::Event { context, .. }
        | HistoryButton::DeliveryEvidence { context, .. } => context,
    }
}

fn history_control_visibility(
    button: &HistoryButton,
    ui: &ObserverUiState,
    navigation: &ProductionNavigation,
    view: PrimaryView,
    snapshot: Option<&babylon_persistence::ProductionSnapshotV1>,
) -> ControlAvailability {
    use ControlAvailability::{Disabled, Enabled};
    if ui.menu_open || ui.splash_visible || ui.comparison_open {
        return Disabled("Close the menu to inspect history");
    }
    if matches!(button, HistoryButton::Week { .. }) {
        if ui.history_open {
            Enabled
        } else {
            Disabled("Open Trends to inspect a committed week")
        }
    } else if !ui.history_open {
        Disabled("Open History to inspect committed developments")
    } else if ui.archive_open || readings_panel_visible(view, navigation, ui, snapshot) {
        Disabled("Close the current readings to inspect the delivery log")
    } else {
        Enabled
    }
}

fn keyboard_activate(
    event: On<ObserverKeyboardActivate>,
    buttons: Query<&HistoryButton>,
    mut queued: ResMut<HistoryKeyboardActions>,
) {
    let Ok(button) = buttons.get(event.entity) else {
        return;
    };
    if event.context.as_ref() == Some(history_button_context(button)) {
        // Keep the original payload. The Update application checks its exact
        // observation again after other input has had an opportunity to change it.
        queued.0.push(button.clone());
    }
}

fn input(
    buttons: Query<(&Interaction, &HistoryButton), Changed<Interaction>>,
    mut queued: ResMut<HistoryKeyboardActions>,
    mut context: HistoryInput,
) {
    for button in std::mem::take(&mut queued.0) {
        apply_history_action(&button, &mut context);
    }
    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed {
            apply_history_action(button, &mut context);
        }
    }
}

fn apply_history_action(button: &HistoryButton, context: &mut HistoryInput) {
    let HistoryInput {
        session,
        frame,
        history,
        ui,
        refresh,
        feedback,
        time,
        navigation,
        view,
    } = context;
    let snapshot = frame
        .for_session(session)
        .and_then(|frame| frame.production.as_ref());
    let visible = history_control_visibility(button, ui, navigation, **view, snapshot);
    let log = matches!(button, HistoryButton::DeliveryEvidence { .. })
        .then(|| snapshot.and_then(|snapshot| delivery_log_entries(snapshot, LOG_ENTRIES).ok()))
        .flatten();
    let available = if visible == ControlAvailability::Enabled {
        button_availability(button, session, frame, log.as_ref())
    } else {
        visible
    };
    if let ControlAvailability::Disabled(reason) = available {
        feedback.reject(reason, time.elapsed_secs_f64());
        return;
    }
    feedback.message = None;
    if let HistoryButton::DeliveryEvidence { context, group } = button {
        let expanded = (context.clone(), group.clone());
        history.expanded_delivery = if history.expanded_delivery.as_ref() == Some(&expanded) {
            None
        } else {
            Some(expanded)
        };
        return;
    }
    let week = match button {
        HistoryButton::Week { week, .. } => *week,
        HistoryButton::Event { event, .. } => event.week,
        HistoryButton::DeliveryEvidence { .. } => return,
    };
    session.pause_month();
    if session.viewed_tick != week {
        session.inspect_tick(week);
        refresh.bump();
    }
    // The committed evidence is revalidated after this week loads.
    history.selected_event = match button {
        HistoryButton::Event { event, .. } => Some((session.context(), event.clone())),
        HistoryButton::Week { .. } | HistoryButton::DeliveryEvidence { .. } => None,
    };
    history.focus_selected_event = history.selected_event.is_some();
}

type HistoryFocusOwners = Or<(With<HistoryButton>, With<HistoryHint>)>;

fn focus_eligibility(
    presentation: LogPresentation,
    mut targets: Query<(&mut ObserverFocusTarget, Option<&HistoryButton>), HistoryFocusOwners>,
) {
    let LogPresentation {
        session,
        frame,
        history: _,
        ui,
        navigation,
        view,
    } = presentation;
    if !(session.is_changed()
        || frame.is_changed()
        || ui.is_changed()
        || navigation.is_changed()
        || view.is_changed()
        || targets.iter_mut().any(|(target, _)| target.is_added()))
    {
        return;
    }
    let snapshot = frame
        .for_session(&session)
        .and_then(|frame| frame.production.as_ref());
    let log = snapshot.and_then(|snapshot| delivery_log_entries(snapshot, LOG_ENTRIES).ok());
    for (mut target, button) in &mut targets {
        let (context, available) = button.map_or_else(
            || {
                (
                    target.context.clone(),
                    target
                        .context
                        .as_ref()
                        .is_some_and(|context| session.accepts(context))
                        && ui.history_open
                        && !ui.menu_open
                        && !ui.splash_visible
                        && !ui.comparison_open,
                )
            },
            |button| {
                (
                    Some(history_button_context(button).clone()),
                    history_control_visibility(button, &ui, &navigation, *view, snapshot)
                        == ControlAvailability::Enabled
                        && button_availability(button, &session, &frame, log.as_ref())
                            == ControlAvailability::Enabled,
                )
            },
        );
        let mut next = target.clone();
        next.context = context;
        next.available = available;
        target.set_if_neq(next);
    }
}

fn fetch(scope: &HistoryScope) -> Result<Vec<WeeklyOutput>, String> {
    let Some(site_id) = &scope.site else {
        return Ok(Vec::new());
    };
    let reader = match scope.context.perspective {
        Perspective::FullObserver => ObserverEconomyReaderV1::from_observer_env(),
        Perspective::PlayerKnowledge => ObserverEconomyReaderV1::from_known_env(),
    }
    .map_err(|error| error.to_string())?;
    let mut points = Vec::new();
    for week in scope.context.tick.saturating_sub(CHART_WEEKS - 1)..=scope.context.tick {
        let frame = reader
            .snapshot(scope.context.campaign, week)
            .map_err(|error| error.to_string())?;
        let Some(site) = frame
            .production
            .as_ref()
            .and_then(|snapshot| snapshot.sites.iter().find(|site| site.id == *site_id))
        else {
            return Err("Production history is not disclosed by this read capability.".into());
        };
        points.push(WeeklyOutput::from_site(week, site)?);
    }
    Ok(points)
}

fn update(
    session: Res<ObserverSession>,
    frame: Res<ObserverFrame>,
    ui: Res<ObserverUiState>,
    navigation: Res<ProductionNavigation>,
    mut history: ResMut<HistoryState>,
    mut links: MessageWriter<ProductionCommand>,
) {
    if history.expanded_delivery.is_some() && (session.is_changed() || frame.is_changed()) {
        reconcile_delivery_expansion(&session, &frame, &mut history);
    }
    let scope = HistoryScope {
        context: session.context(),
        site: navigation.selected_site.clone(),
    };
    if history.scope.as_ref() != Some(&scope) {
        history.pending = None;
        history.points.clear();
        history.error = None;
        history.scope = Some(scope.clone());
    }
    if history
        .selected_event
        .as_ref()
        .is_some_and(|(context, _)| !session.accepts(context))
    {
        history.selected_event = None;
        history.focus_selected_event = false;
    }
    if let Some((_, event)) = history
        .selected_event
        .clone()
        .filter(|_| history.focus_selected_event)
    {
        if let Some(snapshot) = frame
            .for_session(&session)
            .and_then(|frame| frame.production.as_ref())
        {
            if snapshot.events.contains(&event) {
                if let Some(site) = event.subject_site_ids.first() {
                    if navigation.selected_site.as_ref() != Some(site) {
                        links.write(ProductionCommand::Select {
                            site_id: site.clone(),
                            context: session.context(),
                        });
                    }
                }
            } else {
                history.selected_event = None;
            }
            history.focus_selected_event = false;
        }
    }
    if !ui.history_open || ui.menu_open || ui.comparison_open {
        return;
    }
    if frame.for_session(&session).is_none() {
        return;
    }
    if history.pending.is_none()
        && history.points.is_empty()
        && history.error.is_none()
        && scope.site.is_some()
    {
        history.pending = Some(AsyncComputeTaskPool::get().spawn(async move { fetch(&scope) }));
    }
    if history.pending.is_some() {
        let result = history
            .bypass_change_detection()
            .pending
            .as_mut()
            .and_then(|task| block_on(bevy::tasks::futures_lite::future::poll_once(task)));
        if let Some(result) = result {
            history.pending = None;
            match result {
                Ok(points) => history.points = points,
                Err(error) => history.error = Some(error),
            }
        }
    }
}

fn reconcile_delivery_expansion(
    session: &ObserverSession,
    frame: &ObserverFrame,
    history: &mut HistoryState,
) {
    let Some((context, group)) = &history.expanded_delivery else {
        return;
    };
    let valid = session.accepts(context)
        && frame
            .for_session(session)
            .and_then(|frame| frame.production.as_ref())
            .and_then(|snapshot| delivery_log_entries(snapshot, LOG_ENTRIES).ok())
            .is_some_and(|log| log.contains_group(group));
    if !valid {
        history.expanded_delivery = None;
    }
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn bar_height(value: Option<u64>, maximum: u64) -> f32 {
    value.map_or(0.0, |value| {
        ((value as f64 / maximum.max(1) as f64) * 56.0) as f32
    })
}

#[derive(QueryData)]
#[query_data(mutable)]
struct HistoryButtonAppearance {
    interaction: &'static Interaction,
    button: &'static HistoryButton,
    background: &'static mut BackgroundColor,
    border: &'static mut BorderColor,
}

type ChangedHistoryButtons = (
    With<HistoryButton>,
    Or<(Changed<Interaction>, Changed<HistoryButton>)>,
);

fn history_buttons_need_paint(
    session: Res<ObserverSession>,
    frame: Res<ObserverFrame>,
    ui: Res<ObserverUiState>,
    history: Res<HistoryState>,
    changed_buttons: Query<(), ChangedHistoryButtons>,
    added_hints: Query<(), Added<HistoryHint>>,
) -> bool {
    session.is_changed()
        || frame.is_changed()
        || ui.is_changed()
        || history.is_changed()
        || !changed_buttons.is_empty()
        || !added_hints.is_empty()
}

fn paint_buttons(
    session: Res<ObserverSession>,
    frame: Res<ObserverFrame>,
    ui: Res<ObserverUiState>,
    mut buttons: Query<HistoryButtonAppearance>,
    mut hints: Query<&mut Text, With<HistoryHint>>,
) {
    if ui.menu_open || ui.splash_visible || ui.comparison_open {
        return;
    }
    let log = buttons
        .iter()
        .any(|item| matches!(item.button, HistoryButton::DeliveryEvidence { .. }))
        .then(|| {
            frame
                .for_session(&session)
                .and_then(|frame| frame.production.as_ref())
                .and_then(|snapshot| delivery_log_entries(snapshot, LOG_ENTRIES).ok())
        })
        .flatten();
    let mut hint = match inspection_availability(&session) {
        ControlAvailability::Enabled => "Select a committed week or event to inspect.",
        ControlAvailability::Disabled(reason) => reason,
    };
    for mut item in &mut buttons {
        let available = button_availability(item.button, &session, &frame, log.as_ref());
        if *item.interaction != Interaction::None {
            if let ControlAvailability::Disabled(reason) = available {
                hint = reason;
            }
        }
        let enabled = available == ControlAvailability::Enabled;
        let color = match item.interaction {
            _ if !enabled => theme::INK,
            Interaction::Pressed => theme::RED.with_alpha(0.5),
            Interaction::Hovered => theme::YELLOW.with_alpha(0.25),
            Interaction::None => theme::PANEL,
        };
        item.background.set_if_neq(BackgroundColor(color));
        let border = if !enabled {
            theme::GRAY.with_alpha(0.3)
        } else if *item.interaction != Interaction::None {
            theme::YELLOW
        } else if matches!(item.button, HistoryButton::Event { .. }) {
            theme::BLUE
        } else {
            theme::GRAY
        };
        item.border.set_if_neq(BorderColor::all(border));
    }
    for mut text in &mut hints {
        text.set_if_neq(Text::new(hint));
    }
}

fn paint(
    mut commands: Commands,
    session: Res<ObserverSession>,
    frame: Res<ObserverFrame>,
    history: Res<HistoryState>,
    ui: Res<ObserverUiState>,
    navigation: Res<ProductionNavigation>,
    mut panels: Query<(Entity, &mut Visibility), With<HistoryPanel>>,
) {
    let Ok((panel, mut visibility)) = panels.single_mut() else {
        return;
    };
    visibility.set_if_neq(if ui.history_open && !ui.menu_open && !ui.comparison_open {
        Visibility::Visible
    } else {
        Visibility::Hidden
    });
    if !(history.is_changed()
        || frame.is_changed()
        || ui.is_changed()
        || navigation.is_changed()
        || session.is_changed())
    {
        return;
    }
    commands.entity(panel).despawn_children();
    if !ui.history_open {
        return;
    }
    let context = session.context();
    let snapshot = frame
        .for_session(&session)
        .and_then(|frame| frame.production.as_ref());
    commands.entity(panel).with_children(|panel| {
        panel.spawn((label("", 11.0, theme::GRAY), HistoryHint, ObserverFocusTarget::reading(Some(context.clone()))));
        let Some(snapshot)=snapshot else {
            panel.spawn(label("HISTORY / production evidence unavailable in this observation",14.0,theme::GRAY));
            return;
        };
        if let Some((scope, event)) = &history.selected_event {
            if session.accepts(scope) && snapshot.events.iter().any(|current| current == event) {
                panel.spawn(label(format!("SELECTED / WEEK {} / {}\n{}\nRECEIPT {}", event.week, event.kind, event.description, event.receipt_digest), 12.0, theme::YELLOW));
            }
        }
        let site=navigation.selected_site.as_ref().and_then(|id|snapshot.sites.iter().find(|site|site.id==*id));
        if let Some(site)=site {
            panel.spawn(label(format!("{} / {} {} per week | P planned / D produced",site.name,site.output_unit,site.output_good),13.0,theme::YELLOW));
            if let Some(error)=&history.error {panel.spawn(label(error,13.0,theme::RED));}
            else if history.points.is_empty() {panel.spawn(label("Reading committed weeks...",13.0,theme::GRAY));}
            else {
                let maximum=history.points.iter().flat_map(|point|[point.planned,point.produced]).flatten().max().unwrap_or(1);
                panel.spawn(Node{column_gap:px(5),align_items:AlignItems::End,flex_shrink:0.0,overflow:Overflow::scroll_x(),..default()}).with_children(|chart| {
                    for point in &history.points {
                        chart.spawn((Button, ObserverFocusTarget::action(Some(context.clone())), HistoryButton::Week{context:context.clone(),week:point.week},
                            Node{min_width:px(48),padding:UiRect::all(px(3)),row_gap:px(3),flex_direction:FlexDirection::Column,
                                border:UiRect::bottom(px(2)),..default()},
                            BorderColor::all(if point.week==session.viewed_tick {theme::YELLOW}else{theme::GRAY}),
                            BackgroundColor(theme::PANEL),
                            DeclaredSurface::new(SurfaceId::ObserverProduction)))
                        .with_children(|column| {
                            column.spawn(Node{height:px(56),column_gap:px(4),align_items:AlignItems::End,..default()}).with_children(|bars| {
                                bars.spawn((Node{width:px(16),height:px(bar_height(point.planned,maximum)),..default()},BackgroundColor(theme::GRAY)));
                                bars.spawn((Node{width:px(16),height:px(bar_height(point.produced,maximum)),..default()},BackgroundColor(theme::YELLOW)));
                            });
                            column.spawn(label(format!("P {}\nD {}\nW{:02}",point.planned.map_or_else(||"-".into(),grouped),point.produced.map_or_else(||"-".into(),grouped),point.week),11.0,theme::PAPER));
                        });
                    }
                });
                panel.spawn(label(format!("Scale 0-{} {} | - means no production receipt at foundation. Click a week to inspect.",grouped(maximum),site.output_unit),11.0,theme::GRAY));
            }
        } else {panel.spawn(label("Select a producer to chart its committed output. The log locates affected subjects.",13.0,theme::YELLOW));}

    });
}

/// The log reads the same committed observation as the map. It is independent
/// of chart disclosure and never fabricates communications or player acts.
#[derive(SystemParam)]
struct LogPresentation<'w> {
    session: Res<'w, ObserverSession>,
    frame: Res<'w, ObserverFrame>,
    history: Res<'w, HistoryState>,
    ui: Res<'w, ObserverUiState>,
    navigation: Res<'w, ProductionNavigation>,
    view: Res<'w, PrimaryView>,
}

fn paint_log(
    mut commands: Commands,
    presentation: LogPresentation,
    mut panels: Query<(Entity, &mut Visibility), With<LogPanel>>,
) {
    let LogPresentation {
        session,
        frame,
        history,
        ui,
        navigation,
        view,
    } = presentation;
    let Ok((entity, mut visibility)) = panels.single_mut() else {
        return;
    };
    let snapshot = frame
        .for_session(&session)
        .and_then(|frame| frame.production.as_ref());
    visibility.set_if_neq(
        if !ui.history_open
            || ui.menu_open
            || ui.splash_visible
            || ui.comparison_open
            || ui.archive_open
            || readings_panel_visible(*view, &navigation, &ui, snapshot)
        {
            Visibility::Hidden
        } else {
            Visibility::Visible
        },
    );
    if !ui.history_open {
        if ui.is_changed() {
            commands.entity(entity).despawn_children();
        }
        return;
    }
    if !(session.is_changed() || frame.is_changed() || history.is_changed() || ui.is_changed()) {
        return;
    }
    commands.entity(entity).despawn_children();
    let context = session.context();
    commands.entity(entity).with_children(|panel| {
        panel.spawn(label("COMMITTED DEVELOPMENTS", 24.0, theme::PAPER)).insert(crate::observer_ui::ObserverFontRole::Display);
        panel.spawn((label("", 11.0, theme::GRAY), HistoryHint, ObserverFocusTarget::reading(Some(context.clone()))));
        let Some(snapshot) = snapshot else {
            panel.spawn(label("No production developments are available in this observation.", 13.0, theme::GRAY));
            return;
        };
        if let Some((scope, event)) = &history.selected_event {
            if session.accepts(scope) && snapshot.events.iter().any(|current| current == event) {
                panel.spawn(label(format!("EXAMINING / {}\n{}", event.kind.to_uppercase(), event.description), 13.0, theme::YELLOW));
                panel.spawn(label(format!("Committed week {}. Follow the affected subjects in the world. Receipt details are in Trends [H].", event.week), 11.0, theme::GRAY));
            }
        }
        if snapshot.events.is_empty() {
            panel.spawn(label("No developments are recorded through this point in the campaign. Run a month from the live edge to follow production and deliveries.", 14.0, theme::PAPER));
            return;
        }
        spawn_log_entries(panel, snapshot, &context, &history);
    });
}

fn spawn_log_entries(
    panel: &mut ChildSpawnerCommands,
    snapshot: &babylon_persistence::ProductionSnapshotV1,
    context: &ObservationContext,
    history: &HistoryState,
) {
    let log = match delivery_log_entries(snapshot, LOG_ENTRIES) {
        Ok(log) => log,
        Err(error) => {
            panel.spawn(label(error.to_string(), 13.0, theme::RED));
            return;
        }
    };
    panel.spawn(label(format!(
        "Latest {} of {} developments / {} original evidence entries. Expand a delivery to inspect its committed evidence.",
        log.entries.len(), log.total_entries, log.evidence_entries
    ), 11.0, theme::GRAY));
    let mut previous_month = None;
    for entry in log.entries {
        let month = CampaignMonth::at_week(entry.week()).number;
        if previous_month != Some(month) {
            panel
                .spawn(label(
                    format!("CAMPAIGN MONTH {month}"),
                    12.0,
                    theme::YELLOW,
                ))
                .insert(Node {
                    margin: UiRect::top(px(8)),
                    flex_shrink: 0.0,
                    ..default()
                });
            previous_month = Some(month);
        }
        match entry {
            DeliveryLogEntry::Event(event) => spawn_event_entry(panel, event, context),
            DeliveryLogEntry::Delivery(group) => {
                let expanded = history
                    .expanded_delivery
                    .as_ref()
                    .is_some_and(|(scope, key)| scope == context && *key == group.key);
                spawn_delivery_entry(panel, &group, context, expanded);
            }
        }
    }
}

fn spawn_event_entry(
    panel: &mut ChildSpawnerCommands,
    event: &ProductionEventV1,
    context: &ObservationContext,
) {
    panel
        .spawn((
            Button,
            ObserverFocusTarget::action(Some(context.clone())),
            HistoryButton::Event {
                context: context.clone(),
                event: event.clone(),
            },
            Node {
                padding: UiRect::axes(px(10), px(9)),
                border: UiRect::left(px(2)),
                flex_direction: FlexDirection::Column,
                row_gap: px(5),
                flex_shrink: 0.0,
                min_width: px(0),
                ..default()
            },
            BorderColor::all(theme::BLUE),
            BackgroundColor(theme::PANEL),
            DeclaredSurface::new(SurfaceId::ObserverProduction),
        ))
        .with_children(|entry| {
            entry.spawn(label(event.kind.to_uppercase(), 11.0, theme::BLUE));
            entry.spawn(label(&event.description, 13.0, theme::PAPER));
            entry.spawn(label(
                format!("Week {} / inspect consequence", event.week),
                10.0,
                theme::GRAY,
            ));
        });
}

fn spawn_delivery_entry(
    panel: &mut ChildSpawnerCommands,
    group: &DeliveryGroup<'_>,
    context: &ObservationContext,
    expanded: bool,
) {
    panel
        .spawn((
            Node {
                padding: UiRect::axes(px(10), px(9)),
                border: UiRect::left(px(2)),
                flex_direction: FlexDirection::Column,
                row_gap: px(7),
                flex_shrink: 0.0,
                min_width: px(0),
                ..default()
            },
            BorderColor::all(theme::BLUE),
            BackgroundColor(theme::PANEL),
            DeclaredSurface::new(SurfaceId::ObserverProduction),
        ))
        .with_children(|entry| {
            entry.spawn(label(group.headline(), 14.0, theme::PAPER));
            entry.spawn(label(
                format!(
                    "Week {} / {} -> {}",
                    group.key.week, group.supplier.name, group.buyer.name
                ),
                11.0,
                theme::GRAY,
            ));
            entry
                .spawn((
                    Button,
                    ObserverFocusTarget::action(Some(context.clone())),
                    HistoryButton::DeliveryEvidence {
                        context: context.clone(),
                        group: group.key.clone(),
                    },
                    Node {
                        padding: UiRect::axes(px(7), px(5)),
                        border: UiRect::bottom(px(1)),
                        flex_shrink: 0.0,
                        min_width: px(0),
                        ..default()
                    },
                    BorderColor::all(theme::GRAY),
                    BackgroundColor(theme::PANEL),
                    DeclaredSurface::new(SurfaceId::ObserverProduction),
                ))
                .with_children(|button| {
                    button.spawn(label(
                        if expanded {
                            "Collapse evidence".into()
                        } else {
                            format!("Expand {} evidence entries", group.events.len())
                        },
                        11.0,
                        theme::BLUE,
                    ));
                });
            if expanded {
                entry.spawn(label(group.details(), 11.0, theme::PAPER));
                for event in &group.events {
                    spawn_event_entry(entry, event, context);
                }
            }
        });
}

pub struct ObserverHistoryPlugin;
impl Plugin for ObserverHistoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HistoryState>()
            .init_resource::<HistoryKeyboardActions>()
            .add_observer(keyboard_activate)
            .add_systems(
                PreUpdate,
                focus_eligibility.in_set(ObserverFocusSystems::Eligibility),
            )
            .add_systems(Startup, setup)
            .add_systems(Update, input.in_set(ObserverSet::Input))
            .add_systems(
                Update,
                update
                    .after(ObserverSet::Install)
                    .before(ObserverSet::Paint),
            )
            .add_systems(
                Update,
                (
                    paint,
                    paint_log,
                    paint_buttons.run_if(history_buttons_need_paint),
                )
                    .chain()
                    .in_set(ObserverSet::Paint),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observer::SessionPhase;
    use babylon_persistence::{
        CampaignId, ObserverEconomySnapshotV1, ObserverVisibilityV1, ProductionSnapshotV1,
    };

    fn event(week: u64) -> ProductionEventV1 {
        ProductionEventV1 {
            id: format!("delivery-{week}"),
            week,
            subject_site_ids: vec!["producer".into()],
            kind: "delivery".into(),
            description: "Committed input delivery".into(),
            receipt_digest: "a".repeat(64),
            delivery_evidence: None,
        }
    }

    fn history_app(event: ProductionEventV1) -> (App, Entity) {
        let mut session = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::from_u128(1)));
        session.ready(3, None);
        session.foundation_digest = Some("foundation".into());
        assert!(session.installed(&session.context()));
        let frame = ObserverFrame(Some(ObserverEconomySnapshotV1 {
            campaign_id: session.campaign.as_uuid().to_string(),
            resolve_tick: 3,
            foundation_digest: "foundation".into(),
            nominal_world_hash: None,
            tick_content_hash: None,
            envelope_digest: None,
            visibility: ObserverVisibilityV1::FullObserver,
            counties: vec![],
            production: Some(ProductionSnapshotV1 {
                labor_accounts: Vec::new(),
                scenario_label: "Designed test campaign".into(),
                horizon_week: 16,
                sites: vec![],
                routes: vec![],
                freight: vec![],
                events: vec![event],
                material_balance: None,
                observed_contexts: Vec::new(),
                process_attributions: Vec::new(),
                provenance: vec![],
            }),
        }));
        let mut app = App::new();
        app.insert_resource(session)
            .insert_resource(frame)
            .insert_resource(ObserverUiState {
                history_open: true,
                menu_open: false,
                splash_visible: false,
                ..default()
            })
            .init_resource::<HistoryState>()
            .init_resource::<HistoryKeyboardActions>()
            .init_resource::<ProductionNavigation>()
            .init_resource::<PrimaryView>()
            .init_resource::<DossierRefresh>()
            .init_resource::<ObserverFeedback>()
            .init_resource::<Time>()
            .add_systems(PreUpdate, focus_eligibility)
            .add_systems(
                Update,
                (input, paint_buttons.run_if(history_buttons_need_paint)).chain(),
            );
        let hint = app.world_mut().spawn((Text::new(""), HistoryHint)).id();
        (app, hint)
    }

    fn spawn_button(app: &mut App, button: HistoryButton, interaction: Interaction) -> Entity {
        app.world_mut()
            .spawn((
                button,
                interaction,
                BackgroundColor(theme::PANEL),
                BorderColor::all(theme::BLUE),
            ))
            .id()
    }

    #[test]
    fn keyboard_history_queues_original_evidence_and_refuses_a_later_scope_change() {
        let original = event(2);
        let (mut app, _) = history_app(original.clone());
        app.add_observer(keyboard_activate);
        let context = app.world().resource::<ObserverSession>().context();
        let button = spawn_button(
            &mut app,
            HistoryButton::Event {
                context: context.clone(),
                event: original.clone(),
            },
            Interaction::None,
        );
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity: button,
            context: Some(context.clone()),
        });
        assert_eq!(app.world().resource::<ObserverSession>().viewed_tick, 3);
        assert!(app
            .world()
            .resource::<HistoryState>()
            .selected_event
            .is_none());
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(Perspective::PlayerKnowledge);
        app.update();
        assert_eq!(app.world().resource::<ObserverSession>().viewed_tick, 3);
        assert!(app
            .world()
            .resource::<HistoryState>()
            .selected_event
            .is_none());
        assert_eq!(app.world().resource::<DossierRefresh>().0, 0);
        assert!(app.world().resource::<ObserverFeedback>().message.is_some());

        let (mut app, _) = history_app(original.clone());
        app.add_observer(keyboard_activate);
        let context = app.world().resource::<ObserverSession>().context();
        let button = spawn_button(
            &mut app,
            HistoryButton::Event {
                context: context.clone(),
                event: original.clone(),
            },
            Interaction::None,
        );
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity: button,
            context: Some(context),
        });
        app.update();
        assert_eq!(app.world().resource::<ObserverSession>().viewed_tick, 2);
        assert_eq!(app.world().resource::<DossierRefresh>().0, 1);
        assert_eq!(
            app.world()
                .resource::<HistoryState>()
                .selected_event
                .as_ref()
                .unwrap()
                .1,
            original
        );
    }

    #[test]
    fn keyboard_delivery_expansion_uses_current_group_without_advancing_time() {
        let mut app = delivery_app(2, 3);
        app.add_observer(keyboard_activate);
        app.update();
        let (entity, context) = {
            let world = app.world_mut();
            world
                .query::<(Entity, &HistoryButton)>()
                .iter(world)
                .find_map(|(entity, button)| {
                    matches!(button, HistoryButton::DeliveryEvidence { .. })
                        .then(|| (entity, history_button_context(button).clone()))
                })
                .expect("a grouped delivery has an expansion control")
        };
        let week = app.world().resource::<ObserverSession>().viewed_tick;
        let generation = app.world().resource::<DossierRefresh>().0;
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity,
            context: Some(context),
        });
        app.update();
        assert!(app
            .world()
            .resource::<HistoryState>()
            .expanded_delivery
            .is_some());
        assert_eq!(app.world().resource::<ObserverSession>().viewed_tick, week);
        assert_eq!(app.world().resource::<DossierRefresh>().0, generation);
    }

    fn delivery_site(id: &str, name: &str) -> ProductionSiteV1 {
        ProductionSiteV1 {
            id: id.into(),
            county_geoid: "26163".into(),
            name: name.into(),
            industry_code: "331".into(),
            observed_employment: None,
            output_good_id: "sheet".into(),
            output_unit_id: "tonnes".into(),
            output_good: "Sheet metal".into(),
            output_unit: "tonnes".into(),
            output_per_batch: 1,
            available_batches: 1,
            planned_batches: None,
            produced_batches: None,
            inventory: vec![],
            inputs: vec![],
            labor: vec![],
        }
    }

    fn delivery_app(parts: usize, week: u64) -> App {
        use babylon_persistence::{
            ProductionDeliveryEvidenceV1, ProductionDeliveryStageV1, ProductionRouteV1,
        };
        let (mut app, _) = history_app(event(week));
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .history_open = true;
        let mut frame = app.world_mut().resource_mut::<ObserverFrame>();
        let snapshot = frame.0.as_mut().unwrap().production.as_mut().unwrap();
        snapshot.sites = vec![
            delivery_site("supplier", "Wayne metal"),
            delivery_site("buyer", "Macomb parts"),
        ];
        snapshot.routes = vec![ProductionRouteV1 {
            id: "route".into(),
            supplier_site_id: "supplier".into(),
            buyer_site_id: "buyer".into(),
            good_id: "sheet".into(),
            unit_id: "tonnes".into(),
            good: "Sheet metal".into(),
            unit: "tonnes".into(),
            travel_weeks: 1,
            ordered: 10_000,
            shipped: 999,
            delivered: 333,
            lost: 0,
            realized: 333,
            backlog: 9_001,
        }];
        snapshot.events = (0..parts)
            .flat_map(|part| {
                [
                    ProductionDeliveryStageV1::Arrival,
                    ProductionDeliveryStageV1::Delivery,
                    ProductionDeliveryStageV1::QuantityRealization,
                ]
                .into_iter()
                .enumerate()
                .map(move |(index, stage)| ProductionEventV1 {
                    id: format!("part-{part}-stage-{index}"),
                    week,
                    subject_site_ids: vec!["supplier".into(), "buyer".into()],
                    kind: format!("Original stage {index}"),
                    description: format!("Original part {part} stage {index}"),
                    receipt_digest: "a".repeat(64),
                    delivery_evidence: Some(ProductionDeliveryEvidenceV1 {
                        stage,
                        order_id: "order".into(),
                        route_id: "route".into(),
                        good_id: "sheet".into(),
                        unit_id: "tonnes".into(),
                        quantity: 2,
                    }),
                })
            })
            .collect();
        app.add_message::<ProductionCommand>().add_systems(
            Update,
            (update, paint_log)
                .chain()
                .after(input)
                .before(paint_buttons),
        );
        app.world_mut().spawn((LogPanel, Visibility::Visible));
        app
    }

    fn delivery_toggle(app: &mut App) -> (Entity, HistoryButton) {
        let world = app.world_mut();
        world
            .query::<(Entity, &HistoryButton)>()
            .iter(world)
            .find(|(_, button)| matches!(button, HistoryButton::DeliveryEvidence { .. }))
            .map(|(entity, button)| (entity, button.clone()))
            .expect("delivery expansion button")
    }

    fn log_text(app: &mut App) -> String {
        let world = app.world_mut();
        world
            .query::<&Text>()
            .iter(world)
            .map(|text| text.0.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn delivery_summaries_group_before_display_limit_and_expand_exact_evidence_without_time_change()
    {
        let mut app = delivery_app(61, 3);
        app.world_mut().resource_mut::<ObserverSession>().playing = true;
        app.update();
        assert_eq!(history_buttons(&mut app).len(), 1);
        assert!(log_text(&mut app).contains("Expand 183 evidence entries"));
        assert!(!log_text(&mut app).contains("Original part 0 stage 0"));
        let context = app.world().resource::<ObserverSession>().context();
        let (toggle, _) = delivery_toggle(&mut app);
        app.world_mut()
            .entity_mut(toggle)
            .insert(Interaction::Pressed);
        app.update();
        let session = app.world().resource::<ObserverSession>();
        assert_eq!(session.context(), context);
        assert_eq!((session.durable_tick, session.viewed_tick), (3, 3));
        assert!(session.playing, "expansion must not even pause the month");
        assert_eq!(app.world().resource::<DossierRefresh>().0, 0);
        assert!(app
            .world()
            .resource::<HistoryState>()
            .selected_event
            .is_none());
        let texts = log_text(&mut app);
        assert!(texts
            .contains("Arrived: 122 tonnes\nDelivered: 122 tonnes\nQuantity realized: 122 tonnes"));
        assert!(!texts.contains("366 tonnes"));
        let originals = app
            .world()
            .resource::<ObserverFrame>()
            .0
            .as_ref()
            .unwrap()
            .production
            .as_ref()
            .unwrap()
            .events
            .clone();
        let world = app.world_mut();
        let shown = world
            .query::<&HistoryButton>()
            .iter(world)
            .filter_map(|button| match button {
                HistoryButton::Event {
                    context: scope,
                    event,
                } => {
                    assert_eq!(*scope, context);
                    Some(event.clone())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(shown.len(), originals.len());
        for event in originals {
            assert_eq!(shown.iter().filter(|row| **row == event).count(), 1);
        }
        let (toggle, _) = delivery_toggle(&mut app);
        app.world_mut()
            .entity_mut(toggle)
            .insert(Interaction::Pressed);
        app.update();
        assert_eq!(history_buttons(&mut app).len(), 1);
        assert!(app
            .world()
            .resource::<HistoryState>()
            .expanded_delivery
            .is_none());
    }

    #[test]
    fn expanded_original_event_preserves_exact_navigation_and_receipt_scope() {
        let mut app = delivery_app(1, 2);
        app.update();
        let (toggle, _) = delivery_toggle(&mut app);
        app.world_mut()
            .entity_mut(toggle)
            .insert(Interaction::Pressed);
        app.update();
        let world = app.world_mut();
        let (entity, event) = world
            .query::<(Entity, &HistoryButton)>()
            .iter(world)
            .find_map(|(entity, button)| match button {
                HistoryButton::Event { event, .. } => Some((entity, event.clone())),
                _ => None,
            })
            .unwrap();
        app.world_mut()
            .entity_mut(entity)
            .insert(Interaction::Pressed);
        app.update();
        let session = app.world().resource::<ObserverSession>();
        assert_eq!(session.viewed_tick, 2);
        assert_eq!(session.phase, SessionPhase::Loading);
        assert_eq!(app.world().resource::<DossierRefresh>().0, 1);
        assert_eq!(
            app.world().resource::<HistoryState>().selected_event,
            Some((session.context(), event.clone()))
        );
        assert!(app
            .world()
            .resource::<HistoryState>()
            .expanded_delivery
            .is_none());
        {
            let mut frame = app.world_mut().resource_mut::<ObserverFrame>();
            frame.0.as_mut().unwrap().resolve_tick = 2;
        }
        let context = app.world().resource::<ObserverSession>().context();
        assert!(app
            .world_mut()
            .resource_mut::<ObserverSession>()
            .installed(&context));
        app.update();
        let selected = app
            .world_mut()
            .resource_mut::<Messages<ProductionCommand>>()
            .drain()
            .collect::<Vec<_>>();
        assert!(
            matches!(selected.as_slice(), [ProductionCommand::Select { site_id, context: actual }]
            if site_id == "supplier" && *actual == context)
        );
        assert_eq!(
            app.world()
                .resource::<HistoryState>()
                .selected_event
                .as_ref()
                .unwrap()
                .1
                .receipt_digest,
            event.receipt_digest
        );
    }

    #[test]
    fn delivery_expansion_clears_on_context_or_evidence_loss_and_stale_controls_refuse() {
        for change in ["campaign", "perspective", "tick", "absent", "removed"] {
            let mut app = delivery_app(1, 3);
            app.update();
            let (toggle, old_button) = delivery_toggle(&mut app);
            app.world_mut()
                .entity_mut(toggle)
                .insert(Interaction::Pressed);
            app.update();
            assert!(app
                .world()
                .resource::<HistoryState>()
                .expanded_delivery
                .is_some());
            match change {
                "campaign" => {
                    app.world_mut().resource_mut::<ObserverSession>().campaign =
                        CampaignId::from_uuid(uuid::Uuid::from_u128(2));
                }
                "perspective" => app
                    .world_mut()
                    .resource_mut::<ObserverSession>()
                    .set_perspective(Perspective::PlayerKnowledge),
                "tick" => {
                    app.world_mut()
                        .resource_mut::<ObserverSession>()
                        .inspect_tick(2);
                }
                "removed" => {
                    app.world_mut()
                        .resource_mut::<ObserverFrame>()
                        .0
                        .as_mut()
                        .unwrap()
                        .production
                        .as_mut()
                        .unwrap()
                        .events
                        .clear();
                }
                _ => {
                    app.world_mut().resource_mut::<ObserverFrame>().0 = None;
                }
            }
            app.update();
            assert!(
                app.world()
                    .resource::<HistoryState>()
                    .expanded_delivery
                    .is_none(),
                "{change}"
            );
            let context = app.world().resource::<ObserverSession>().context();
            spawn_button(&mut app, old_button, Interaction::Pressed);
            app.update();
            assert_eq!(app.world().resource::<ObserverSession>().context(), context);
            assert!(app
                .world()
                .resource::<HistoryState>()
                .expanded_delivery
                .is_none());
            assert!(app.world().resource::<ObserverFeedback>().message.is_some());
            assert_eq!(app.world().resource::<DossierRefresh>().0, 0);
        }
    }

    #[test]
    fn readings_occupy_the_log_panel_and_block_its_hidden_controls() {
        let mut app = delivery_app(1, 3);
        app.update();
        let (toggle, _) = delivery_toggle(&mut app);
        let log = {
            let world = app.world_mut();
            world
                .query_filtered::<Entity, With<LogPanel>>()
                .single(world)
                .unwrap()
        };
        *app.world_mut().resource_mut::<PrimaryView>() = PrimaryView::Production;
        app.world_mut()
            .resource_mut::<ProductionNavigation>()
            .details_open = true;
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .history_open = false;
        // The click was queued before the rail changed owners. Application must
        // reject it even though its original event and observation still match.
        app.world_mut()
            .entity_mut(toggle)
            .insert(Interaction::Pressed);
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(log).unwrap(),
            Visibility::Hidden
        );
        assert!(app
            .world()
            .resource::<HistoryState>()
            .expanded_delivery
            .is_none());
        assert_eq!(
            app.world().resource::<ObserverFeedback>().message,
            Some("Open History to inspect committed developments")
        );
        {
            let world = app.world();
            let frame = world.resource::<ObserverFrame>();
            let session = world.resource::<ObserverSession>();
            assert!(readings_panel_visible(
                PrimaryView::Production,
                world.resource::<ProductionNavigation>(),
                world.resource::<ObserverUiState>(),
                frame
                    .for_session(session)
                    .and_then(|frame| frame.production.as_ref()),
            ));
        }
        // Reopening History takes the rail without discarding the subject's
        // reading preference, and only the newly visible control is admitted.
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .history_open = true;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(log).unwrap(),
            Visibility::Visible
        );
        assert!(app.world().resource::<ProductionNavigation>().details_open);
        let (current_toggle, _) = delivery_toggle(&mut app);
        assert_ne!(current_toggle, toggle);
        app.world_mut()
            .entity_mut(current_toggle)
            .insert(Interaction::Pressed);
        app.update();
        assert!(app
            .world()
            .resource::<HistoryState>()
            .expanded_delivery
            .is_some());
        assert_eq!(app.world().resource::<ObserverSession>().viewed_tick, 3);
        assert_eq!(app.world().resource::<DossierRefresh>().0, 0);
        app.world_mut().resource_mut::<ObserverFrame>().0 = None;
        app.update();
        assert_eq!(*app.world().get::<Visibility>(log).unwrap(), Visibility::Visible,
            "a retained details flag without a validated reading must not obscure the unavailable log");
    }

    #[derive(Resource, Default, Debug, PartialEq, Eq)]
    struct HistoryPaintChanges {
        visibility: usize,
        backgrounds: usize,
        borders: usize,
        text: usize,
    }

    type ChangedHistoryVisibility<'w, 's> = Query<
        'w,
        's,
        (),
        (
            Or<(With<HistoryPanel>, With<LogPanel>)>,
            Changed<Visibility>,
        ),
    >;

    fn record_paint_changes(
        panels: ChangedHistoryVisibility,
        backgrounds: Query<(), (With<HistoryButton>, Changed<BackgroundColor>)>,
        borders: Query<(), (With<HistoryButton>, Changed<BorderColor>)>,
        text: Query<(), Changed<Text>>,
        mut changes: ResMut<HistoryPaintChanges>,
    ) {
        *changes = HistoryPaintChanges {
            visibility: panels.iter().count(),
            backgrounds: backgrounds.iter().count(),
            borders: borders.iter().count(),
            text: text.iter().count(),
        };
    }

    #[test]
    fn expanded_delivery_log_keeps_entities_and_components_unchanged_while_idle() {
        let mut app = delivery_app(61, 3);
        app.init_resource::<HistoryPaintChanges>()
            .add_systems(PostUpdate, record_paint_changes);
        app.update();
        let (toggle, _) = delivery_toggle(&mut app);
        app.world_mut()
            .entity_mut(toggle)
            .insert(Interaction::Pressed);
        app.update();
        app.update();
        let buttons = history_buttons(&mut app);
        assert_eq!(buttons.len(), 184);
        for _ in 0..3 {
            app.update();
            assert_eq!(history_buttons(&mut app), buttons);
            assert_eq!(
                *app.world().resource::<HistoryPaintChanges>(),
                HistoryPaintChanges::default()
            );
        }
    }

    #[test]
    fn contradictory_delivery_identity_refuses_rendering_and_clears_expansion() {
        let mut app = delivery_app(1, 3);
        app.update();
        let (toggle, _) = delivery_toggle(&mut app);
        app.world_mut()
            .entity_mut(toggle)
            .insert(Interaction::Pressed);
        app.update();
        app.world_mut()
            .resource_mut::<ObserverFrame>()
            .0
            .as_mut()
            .unwrap()
            .production
            .as_mut()
            .unwrap()
            .events[0]
            .delivery_evidence
            .as_mut()
            .unwrap()
            .good_id = "different-good".into();
        app.update();
        assert!(history_buttons(&mut app).is_empty());
        assert!(app
            .world()
            .resource::<HistoryState>()
            .expanded_delivery
            .is_none());
        let text = log_text(&mut app);
        assert!(text.contains("Delivery evidence identities do not agree."));
        assert!(!text.contains("Sheet metal delivered"));
    }

    fn history_buttons(app: &mut App) -> Vec<Entity> {
        let mut buttons = app
            .world_mut()
            .query_filtered::<Entity, With<HistoryButton>>();
        let mut entities = buttons.iter(app.world()).collect::<Vec<_>>();
        entities.sort_unstable();
        entities
    }

    #[test]
    fn idle_history_with_sixty_events_keeps_entities_and_components_unchanged() {
        let (mut app, _) = history_app(event(3));
        let events = (0..60)
            .map(|index| ProductionEventV1 {
                id: format!("delivery-3-{index}"),
                ..event(3)
            })
            .collect();
        app.world_mut()
            .resource_mut::<ObserverFrame>()
            .0
            .as_mut()
            .unwrap()
            .production
            .as_mut()
            .unwrap()
            .events = events;
        app.init_resource::<ProductionNavigation>()
            .init_resource::<HistoryPaintChanges>()
            .add_systems(Update, (paint, paint_log).chain().before(paint_buttons))
            .add_systems(PostUpdate, record_paint_changes);
        app.world_mut().spawn((HistoryPanel, Visibility::Hidden));
        app.world_mut().spawn((LogPanel, Visibility::Visible));
        app.update();
        let buttons = history_buttons(&mut app);
        assert_eq!(buttons.len(), 60);
        assert!(app.world().resource::<HistoryPaintChanges>().text > 0);

        app.update();
        assert_eq!(history_buttons(&mut app), buttons);
        assert_eq!(
            *app.world().resource::<HistoryPaintChanges>(),
            HistoryPaintChanges::default(),
            "idle history must not mark unchanged presentation components as changed"
        );
    }

    #[derive(Resource, Default)]
    struct ButtonPaintPasses(usize);

    fn record_button_paint(mut passes: ResMut<ButtonPaintPasses>) {
        passes.0 += 1;
    }

    #[test]
    fn button_paint_gate_skips_idle_and_wakes_for_real_ecs_changes() {
        let event = event(3);
        let (mut app, _) = history_app(event.clone());
        app.init_resource::<ButtonPaintPasses>().add_systems(
            Update,
            record_button_paint.run_if(history_buttons_need_paint),
        );
        let context = app.world().resource::<ObserverSession>().context();
        let entity = spawn_button(
            &mut app,
            HistoryButton::Event { context, event },
            Interaction::None,
        );
        app.update();
        app.update();
        assert_eq!(app.world().resource::<ButtonPaintPasses>().0, 1);
        for (index, interaction) in [Interaction::Hovered, Interaction::None]
            .into_iter()
            .enumerate()
        {
            *app.world_mut().get_mut::<Interaction>(entity).unwrap() = interaction;
            app.update();
            assert_eq!(app.world().resource::<ButtonPaintPasses>().0, index + 2);
        }
        let context = app.world().resource::<ObserverSession>().context();
        *app.world_mut().get_mut::<HistoryButton>(entity).unwrap() =
            HistoryButton::Week { context, week: 2 };
        app.update();
        assert_eq!(app.world().resource::<ButtonPaintPasses>().0, 4);
        app.world_mut().resource_mut::<ObserverFrame>().0 = None;
        app.update();
        assert_eq!(app.world().resource::<ButtonPaintPasses>().0, 5);
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(Perspective::PlayerKnowledge);
        app.update();
        assert_eq!(app.world().resource::<ButtonPaintPasses>().0, 6);
        app.world_mut().resource_mut::<ObserverUiState>().menu_open = true;
        app.update();
        assert_eq!(app.world().resource::<ButtonPaintPasses>().0, 7);
        app.world_mut().resource_mut::<HistoryState>().error = Some("Unavailable".into());
        app.update();
        assert_eq!(app.world().resource::<ButtonPaintPasses>().0, 8);
        app.world_mut().spawn((HistoryHint, Text::new("")));
        app.update();
        assert_eq!(app.world().resource::<ButtonPaintPasses>().0, 9);
        app.update();
        assert_eq!(app.world().resource::<ButtonPaintPasses>().0, 9);
        let context = app.world().resource::<ObserverSession>().context();
        spawn_button(
            &mut app,
            HistoryButton::Week { context, week: 1 },
            Interaction::None,
        );
        app.update();
        assert_eq!(app.world().resource::<ButtonPaintPasses>().0, 10);
    }

    #[test]
    fn unchanged_interaction_repaints_when_evidence_or_capability_changes() {
        let event = event(3);
        let (mut app, _) = history_app(event.clone());
        let context = app.world().resource::<ObserverSession>().context();
        let entity = spawn_button(
            &mut app,
            HistoryButton::Event {
                context,
                event: event.clone(),
            },
            Interaction::None,
        );
        app.update();
        assert_eq!(
            app.world().get::<BackgroundColor>(entity).unwrap().0,
            theme::PANEL
        );
        app.world_mut()
            .resource_mut::<ObserverFrame>()
            .0
            .as_mut()
            .unwrap()
            .production
            .as_mut()
            .unwrap()
            .events[0]
            .receipt_digest = "b".repeat(64);
        app.update();
        assert_eq!(
            app.world().get::<BackgroundColor>(entity).unwrap().0,
            theme::INK
        );
        app.world_mut()
            .resource_mut::<ObserverFrame>()
            .0
            .as_mut()
            .unwrap()
            .production
            .as_mut()
            .unwrap()
            .events[0] = event;
        app.update();
        assert_eq!(
            app.world().get::<BackgroundColor>(entity).unwrap().0,
            theme::PANEL
        );
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(Perspective::PlayerKnowledge);
        app.update();
        assert_eq!(
            app.world().get::<BackgroundColor>(entity).unwrap().0,
            theme::INK
        );
        *app.world_mut().get_mut::<Interaction>(entity).unwrap() = Interaction::Pressed;
        app.update();
        assert!(app
            .world()
            .resource::<HistoryState>()
            .selected_event
            .is_none());
        assert_eq!(
            app.world().resource::<ObserverFeedback>().message,
            Some("History changed; wait for its current observation")
        );
    }

    #[test]
    fn pending_week_and_event_buttons_explain_refusal_without_selecting_evidence() {
        for choose_event in [false, true] {
            let event = event(3);
            let (mut app, _) = history_app(event.clone());
            let context = {
                let mut session = app.world_mut().resource_mut::<ObserverSession>();
                session.begin_advance().unwrap();
                session.context()
            };
            let button = if choose_event {
                HistoryButton::Event { context, event }
            } else {
                HistoryButton::Week { context, week: 2 }
            };
            let entity = spawn_button(&mut app, button, Interaction::Pressed);
            app.update();
            let session = app.world().resource::<ObserverSession>();
            assert_eq!(session.viewed_tick, 3);
            assert!(session.advance_pending());
            assert_eq!(session.phase, SessionPhase::Advancing);
            assert!(app
                .world()
                .resource::<HistoryState>()
                .selected_event
                .is_none());
            assert_eq!(app.world().resource::<DossierRefresh>().0, 0);
            assert_eq!(
                app.world().resource::<ObserverFeedback>().message,
                Some("Wait for the current week to finish committing")
            );
            assert_eq!(
                app.world().get::<BackgroundColor>(entity).unwrap().0,
                theme::INK
            );
        }
    }

    #[test]
    fn hovering_disabled_history_explains_without_mutating_feedback() {
        let (mut app, hint) = history_app(event(2));
        let context = {
            let mut session = app.world_mut().resource_mut::<ObserverSession>();
            session.begin_advance().unwrap();
            session.context()
        };
        spawn_button(
            &mut app,
            HistoryButton::Week { context, week: 2 },
            Interaction::Hovered,
        );
        app.update();
        app.update();
        assert_eq!(
            app.world().get::<Text>(hint).unwrap().0,
            "Wait for the current week to finish committing"
        );
        assert_eq!(app.world().resource::<ObserverFeedback>().revision, 0);
        assert!(app.world().resource::<ObserverFeedback>().message.is_none());
    }

    #[test]
    fn valid_event_navigation_refreshes_the_dossier_and_binds_selected_evidence() {
        let event = event(2);
        let (mut app, _) = history_app(event.clone());
        let context = app.world().resource::<ObserverSession>().context();
        app.world_mut().resource_mut::<ObserverSession>().playing = true;
        spawn_button(
            &mut app,
            HistoryButton::Event {
                context,
                event: event.clone(),
            },
            Interaction::Pressed,
        );
        app.update();
        let session = app.world().resource::<ObserverSession>();
        assert_eq!(session.viewed_tick, 2);
        assert_eq!(session.phase, SessionPhase::Loading);
        assert!(!session.playing);
        assert_eq!(app.world().resource::<DossierRefresh>().0, 1);
        assert_eq!(
            app.world().resource::<HistoryState>().selected_event,
            Some((session.context(), event))
        );
    }

    #[test]
    fn log_navigation_requires_open_history_and_clears_on_perspective_change() {
        use bevy::ecs::system::RunSystemOnce;
        let event = event(2);
        let (mut app, _) = history_app(event.clone());
        app.world_mut().spawn((LogPanel, Visibility::Visible));
        app.world_mut().run_system_once(paint_log).unwrap();
        let mut query = app.world_mut().query::<(Entity, &HistoryButton)>();
        let button = query
            .iter(app.world())
            .find_map(|(entity, button)| {
                matches!(button, HistoryButton::Event { .. }).then_some(entity)
            })
            .unwrap();
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .history_open = false;
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();
        assert_eq!(app.world().resource::<ObserverSession>().viewed_tick, 3);
        assert!(app
            .world()
            .resource::<HistoryState>()
            .selected_event
            .is_none());
        assert_eq!(
            app.world().resource::<ObserverFeedback>().message,
            Some("Open History to inspect committed developments")
        );
        app.world_mut().entity_mut(button).insert(Interaction::None);
        app.update();
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .history_open = true;
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();
        assert_eq!(app.world().resource::<ObserverSession>().viewed_tick, 2);
        assert!(app
            .world()
            .resource::<HistoryState>()
            .selected_event
            .is_some());

        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(Perspective::PlayerKnowledge);
        app.world_mut().run_system_once(paint_log).unwrap();
        let mut query = app.world_mut().query::<&HistoryButton>();
        assert_eq!(query.iter(app.world()).count(), 0);
        let mut texts = app.world_mut().query::<&Text>();
        assert!(!texts
            .iter(app.world())
            .any(|text| text.0.contains(&event.description)));
    }

    #[test]
    fn an_examined_event_focuses_once_without_overriding_later_navigation() {
        use bevy::ecs::system::RunSystemOnce;
        let event = event(3);
        let (mut app, _) = history_app(event.clone());
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .history_open = false;
        app.init_resource::<ProductionNavigation>()
            .add_message::<ProductionCommand>();
        let context = app.world().resource::<ObserverSession>().context();
        {
            let mut history = app.world_mut().resource_mut::<HistoryState>();
            history.selected_event = Some((context, event));
            history.focus_selected_event = true;
        }
        app.world_mut().run_system_once(update).unwrap();
        let messages = app
            .world_mut()
            .resource_mut::<Messages<ProductionCommand>>()
            .drain()
            .count();
        assert_eq!(messages, 1);
        app.world_mut()
            .resource_mut::<ProductionNavigation>()
            .selected_site = Some("other".into());
        app.world_mut().run_system_once(update).unwrap();
        assert_eq!(
            app.world_mut()
                .resource_mut::<Messages<ProductionCommand>>()
                .drain()
                .count(),
            0
        );
        assert!(app
            .world()
            .resource::<HistoryState>()
            .selected_event
            .is_some());
    }

    #[test]
    fn event_navigation_rejects_changed_evidence_and_stale_capability_context() {
        let mut event = event(2);
        let (mut app, _) = history_app(event.clone());
        let context = app.world().resource::<ObserverSession>().context();
        event.receipt_digest = "b".repeat(64);
        let button = HistoryButton::Event { context, event };
        assert_eq!(
            button_availability(
                &button,
                app.world().resource::<ObserverSession>(),
                app.world().resource::<ObserverFrame>(),
                None,
            ),
            ControlAvailability::Disabled("This event is unavailable in the current observation")
        );
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(Perspective::PlayerKnowledge);
        assert_eq!(
            button_availability(
                &button,
                app.world().resource::<ObserverSession>(),
                app.world().resource::<ObserverFrame>(),
                None,
            ),
            ControlAvailability::Disabled("History changed; wait for its current observation")
        );
    }

    #[test]
    fn exact_history_distinguishes_foundation_zero_and_overflow() {
        let mut site = ProductionSiteV1 {
            id: "site".into(),
            county_geoid: "26163".into(),
            name: "cohort".into(),
            industry_code: "331".into(),
            observed_employment: None,
            output_good_id: "a".repeat(64),
            output_unit_id: "b".repeat(64),
            output_good: "sheet".into(),
            output_unit: "kg".into(),
            output_per_batch: 10,
            available_batches: 8,
            planned_batches: None,
            produced_batches: None,
            inventory: vec![],
            inputs: vec![],
            labor: vec![],
        };
        assert_eq!(WeeklyOutput::from_site(0, &site).unwrap().produced, None);
        site.planned_batches = Some(8);
        site.produced_batches = Some(0);
        assert_eq!(
            WeeklyOutput::from_site(1, &site).unwrap(),
            WeeklyOutput {
                week: 1,
                planned: Some(80),
                produced: Some(0)
            }
        );
        site.produced_batches = Some(u64::MAX);
        assert!(WeeklyOutput::from_site(2, &site).is_err());
    }
}
