//! Native campaign catalog and read-only comparison of separately committed worlds.

use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use babylon_persistence::{
    observer_reader::CampaignSummaryV1, CampaignId, ObserverEconomyReaderV1,
    ObserverEconomySnapshotV1, ObserverVisibilityV1, ProductionSiteV1,
};
use bevy::ecs::{query::QueryData, system::SystemParam};
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};

use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::observer::{ObservationContext, ObserverSession, Perspective};
use crate::observer_controls::{availability, ControlAvailability};
use crate::observer_focus::{
    ObserverFocusPolicy, ObserverFocusSystems, ObserverFocusTarget, ObserverKeyboardActivate,
    ObserverKeyboardClaim,
};
use crate::observer_io::{ObserverSet, RuntimePipe, LAUNCHER_REQUIRED};
use crate::observer_theme as theme;
use crate::observer_ui::{
    ObserverCampaignCatalog, ObserverCommand, ObserverFontRole, ObserverFrame, ObserverUiState,
};

const OPEN_SELECTED_EXIT: u8 = 23;

#[derive(Message, Clone, Copy, Debug)]
pub enum CampaignBrowserCommand {
    Previous,
    Next,
    Open,
    Compare,
    CloseComparison,
    Refresh,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BrowserScope {
    active: ObservationContext,
    generation: u64,
    target: Option<CampaignId>,
}

type CatalogTask = (BrowserScope, Task<Result<Vec<CampaignSummaryV1>, String>>);
type ComparisonTask = (
    BrowserScope,
    Task<Result<ObserverEconomySnapshotV1, String>>,
);

#[derive(Resource, Default)]
struct CampaignBrowserState {
    context: Option<ObservationContext>,
    generation: u64,
    catalog_task: Option<CatalogTask>,
    comparison_task: Option<ComparisonTask>,
    catalog: Vec<CampaignSummaryV1>,
    selected: usize,
    comparison: Option<ObserverEconomySnapshotV1>,
    comparison_target: Option<CampaignId>,
    menu_was_open: bool,
    status: String,
}
impl CampaignBrowserState {
    fn invalidate(&mut self, context: ObservationContext, ui: &mut ObserverUiState) {
        self.context = Some(context);
        self.catalog_task = None;
        self.comparison_task = None;
        self.catalog.clear();
        self.selected = 0;
        self.comparison = None;
        self.comparison_target = None;
        ui.comparison_open = false;
        self.menu_was_open = false;
        self.status.clear();
    }

    fn next_generation(&mut self) -> Option<u64> {
        let Some(generation) = self.generation.checked_add(1) else {
            self.status = "Catalog request counter exhausted; reopen the campaign.".into();
            return None;
        };
        self.generation = generation;
        Some(generation)
    }

    fn accepts(&self, scope: &BrowserScope, session: &ObserverSession) -> bool {
        session.accepts(&scope.active)
            && self.context.as_ref() == Some(&scope.active)
            && self.generation == scope.generation
            && (scope.target.is_none() || scope.target == self.comparison_target)
    }
}

#[derive(Component, Clone, Copy)]
struct BrowserButton(CampaignBrowserCommand);
#[derive(Component)]
struct CatalogText;
#[derive(Component)]
pub(crate) struct ComparisonPanel;
#[derive(Component)]
struct ComparisonText;

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
        ObserverFontRole::Body,
        DeclaredSurface::new(SurfaceId::ObserverShell),
    )
}

fn button(parent: &mut ChildSpawnerCommands, label: &str, command: CampaignBrowserCommand) {
    parent
        .spawn((
            Button,
            BrowserButton(command),
            ObserverFocusTarget::action(None),
            Node {
                padding: UiRect::axes(px(10), px(8)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::ZERO,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(theme::PANEL),
            BorderColor::all(theme::PAPER),
            DeclaredSurface::new(SurfaceId::ObserverShell),
        ))
        .with_child(text(label, 13.0, theme::PAPER));
}

fn setup(mut commands: Commands, menu: Query<Entity, With<ObserverCampaignCatalog>>) {
    if let Ok(menu) = menu.single() {
        commands.entity(menu).with_children(|panel| {
            panel
                .spawn((
                    text("SAVED CAMPAIGNS", 19.0, theme::YELLOW),
                    Node {
                        flex_shrink: 0.0,
                        min_width: px(0),
                        ..default()
                    },
                ))
                .insert(ObserverFontRole::Display);
            panel.spawn((
                text("Loading campaign catalog...", 13.0, theme::PAPER),
                CatalogText,
                ObserverFocusTarget::reading(None),
                Node {
                    flex_shrink: 0.0,
                    min_width: px(0),
                    max_width: percent(100),
                    ..default()
                },
            ));
            panel
                .spawn(Node {
                    column_gap: px(8),
                    row_gap: px(8),
                    flex_wrap: FlexWrap::Wrap,
                    flex_shrink: 0.0,
                    min_width: px(0),
                    ..default()
                })
                .with_children(|row| {
                    button(row, "<  [Left]", CampaignBrowserCommand::Previous);
                    button(row, ">  [Right]", CampaignBrowserCommand::Next);
                    button(row, "Open  [Enter]", CampaignBrowserCommand::Open);
                    button(row, "Compare  [X]", CampaignBrowserCommand::Compare);
                    button(row, "Refresh", CampaignBrowserCommand::Refresh);
                });
        });
    }
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(8),
                right: percent(8),
                top: px(105),
                bottom: px(148),
                padding: UiRect::all(px(22)),
                row_gap: px(14),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::ZERO,
                ..default()
            },
            BackgroundColor(theme::INK),
            BorderColor::all(theme::YELLOW),
            ZIndex(21),
            Visibility::Hidden,
            ComparisonPanel,
            TabGroup::modal(),
            DeclaredSurface::new(SurfaceId::ObserverShell),
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    text("COMMITTED CAMPAIGN COMPARISON", 22.0, theme::YELLOW),
                    Node {
                        flex_shrink: 0.0,
                        min_width: px(0),
                        ..default()
                    },
                ))
                .insert(ObserverFontRole::Display);
            button(
                panel,
                "Close comparison  [Escape]",
                CampaignBrowserCommand::CloseComparison,
            );
            panel
                .spawn((
                    text("", 15.0, theme::PAPER),
                    ComparisonText,
                    ObserverFocusTarget::reading(None),
                    Node {
                        flex_shrink: 0.0,
                        min_width: px(0),
                        max_width: percent(100),
                        ..default()
                    },
                    TextLayout::new_with_linebreak(bevy::text::LineBreak::AnyCharacter),
                ))
                .insert(ObserverFontRole::Exact);
        });
}

fn input(
    buttons: Query<(&Interaction, &BrowserButton), Changed<Interaction>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    claimed: Res<ObserverKeyboardClaim>,
    ui: Res<ObserverUiState>,
    mut messages: MessageWriter<CampaignBrowserCommand>,
) {
    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed {
            dispatch_button(button.0, &ui, &mut messages);
        }
    }
    if ui.menu_open && !ui.splash_visible && !ui.comparison_open {
        for (key, command) in [
            (KeyCode::ArrowLeft, CampaignBrowserCommand::Previous),
            (KeyCode::ArrowRight, CampaignBrowserCommand::Next),
            (KeyCode::Enter, CampaignBrowserCommand::Open),
            (KeyCode::KeyX, CampaignBrowserCommand::Compare),
        ] {
            if keyboard.just_pressed(key) && !claimed.claimed(key) {
                dispatch_button(command, &ui, &mut messages);
            }
        }
    }
    if ui.comparison_open && keyboard.just_pressed(KeyCode::Escape) {
        messages.write(CampaignBrowserCommand::CloseComparison);
    }
}

fn button_visible(command: CampaignBrowserCommand, ui: &ObserverUiState) -> bool {
    !ui.splash_visible
        && match command {
            CampaignBrowserCommand::CloseComparison => ui.comparison_open,
            _ => ui.menu_open && !ui.comparison_open,
        }
}

fn dispatch_button(
    command: CampaignBrowserCommand,
    ui: &ObserverUiState,
    messages: &mut MessageWriter<CampaignBrowserCommand>,
) {
    if button_visible(command, ui) {
        messages.write(command);
    }
}

fn keyboard_button(
    event: On<ObserverKeyboardActivate>,
    buttons: Query<(&BrowserButton, &ObserverFocusTarget)>,
    ui: Res<ObserverUiState>,
    session: Res<ObserverSession>,
    mut messages: MessageWriter<CampaignBrowserCommand>,
) {
    let Ok((button, target)) = buttons.get(event.entity) else {
        return;
    };
    if event.context == target.context && event.context.as_ref() == Some(&session.context()) {
        dispatch_button(button.0, &ui, &mut messages);
    }
}

type BrowserFocusTargets<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut ObserverFocusTarget,
        Option<&'static BrowserButton>,
        Has<CatalogText>,
        Has<ComparisonText>,
    ),
>;

fn sync_focus_targets(
    ui: Res<ObserverUiState>,
    session: Res<ObserverSession>,
    browser: Res<CampaignBrowserState>,
    mut targets: BrowserFocusTargets,
) {
    let context = session.context();
    for (mut target, button, catalog, comparison) in &mut targets {
        if button.is_none() && !catalog && !comparison {
            continue;
        }
        let mut next = target.clone();
        next.context = Some(context.clone());
        next.available = if let Some(button) = button {
            button_visible(button.0, &ui)
                && !session.quit_requested
                && match button.0 {
                    CampaignBrowserCommand::CloseComparison | CampaignBrowserCommand::Refresh => {
                        true
                    }
                    CampaignBrowserCommand::Previous | CampaignBrowserCommand::Next => {
                        browser.context.as_ref() == Some(&context) && browser.catalog.len() > 1
                    }
                    CampaignBrowserCommand::Open => {
                        browser.context.as_ref() == Some(&context)
                            && browser.catalog.get(browser.selected).is_some()
                            && availability(ObserverCommand::NewCampaign, &session)
                                == ControlAvailability::Enabled
                    }
                    CampaignBrowserCommand::Compare => {
                        browser.context.as_ref() == Some(&context)
                            && browser
                                .catalog
                                .get(browser.selected)
                                .is_some_and(|selected| {
                                    selected.id != session.campaign.as_uuid().to_string()
                                        && selected.durable_tick >= session.viewed_tick
                                })
                    }
                }
        } else {
            !ui.splash_visible
                && if comparison {
                    ui.comparison_open
                } else {
                    ui.menu_open && !ui.comparison_open
                }
        };
        target.set_if_neq(next);
    }
}

fn reader(perspective: Perspective) -> Result<ObserverEconomyReaderV1, String> {
    match perspective {
        Perspective::FullObserver => ObserverEconomyReaderV1::from_observer_env(),
        Perspective::PlayerKnowledge => ObserverEconomyReaderV1::from_known_env(),
    }
    .map_err(|error| error.to_string())
}

fn request_catalog(browser: &mut CampaignBrowserState, session: &ObserverSession) {
    let Some(generation) = browser.next_generation() else {
        return;
    };
    let scope = BrowserScope {
        active: session.context(),
        generation,
        target: None,
    };
    let perspective = scope.active.perspective;
    browser.catalog_task = Some((
        scope,
        AsyncComputeTaskPool::get().spawn(async move {
            reader(perspective)?
                .campaigns()
                .map_err(|error| error.to_string())
        }),
    ));
    browser.status = "Loading campaign catalog...".into();
}

fn commands(
    mut messages: MessageReader<CampaignBrowserCommand>,
    mut browser: ResMut<CampaignBrowserState>,
    mut session: ResMut<ObserverSession>,
    mut ui: ResMut<ObserverUiState>,
    mut exits: MessageWriter<AppExit>,
    pipe: Option<Res<RuntimePipe>>,
) {
    for command in messages.read() {
        if session.quit_requested {
            "Closing the campaign; committed weeks are saved automatically."
                .clone_into(&mut browser.status);
            continue;
        }
        if !button_visible(*command, &ui) {
            continue;
        }
        match command {
            CampaignBrowserCommand::Previous | CampaignBrowserCommand::Next => {
                if browser.catalog.is_empty() {
                    continue;
                }
                let length = browser.catalog.len();
                browser.selected = match command {
                    CampaignBrowserCommand::Previous => (browser.selected + length - 1) % length,
                    _ => (browser.selected + 1) % length,
                };
                browser.comparison_task = None;
                browser.comparison = None;
                browser.comparison_target = None;
            }
            CampaignBrowserCommand::Open => {
                open_selected_campaign(&mut browser, &mut session, pipe.as_deref(), &mut exits);
            }
            CampaignBrowserCommand::Compare => {
                let Some(selected) = browser.catalog.get(browser.selected) else {
                    continue;
                };
                if selected.id == session.campaign.as_uuid().to_string() {
                    browser.status = "Select another committed campaign to compare.".into();
                    continue;
                }
                if selected.durable_tick < session.viewed_tick {
                    browser.status = format!("That campaign is committed only through week {}. Inspect that week or an earlier week first.", selected.durable_tick);
                    continue;
                }
                let target = match parse_campaign(&selected.id) {
                    Ok(campaign) => campaign,
                    Err(error) => {
                        browser.status = error;
                        continue;
                    }
                };
                let Some(generation) = browser.next_generation() else {
                    continue;
                };
                session.playing = false;
                let scope = BrowserScope {
                    active: session.context(),
                    generation,
                    target: Some(target),
                };
                let requested = scope.clone();
                browser.comparison_target = Some(target);
                browser.comparison = None;
                ui.comparison_open = true;
                browser.comparison_task = Some((
                    scope,
                    AsyncComputeTaskPool::get().spawn(async move {
                        reader(requested.active.perspective)?
                            .snapshot(target, requested.active.tick)
                            .map_err(|error| error.to_string())
                    }),
                ));
                browser.status = "Loading the other campaign's committed week...".into();
                ui.menu_open = false;
            }
            CampaignBrowserCommand::CloseComparison => {
                ui.comparison_open = false;
                browser.comparison_task = None;
                browser.comparison = None;
                browser.comparison_target = None;
            }
            CampaignBrowserCommand::Refresh => request_catalog(&mut browser, &session),
        }
    }
}

fn open_selected_campaign(
    browser: &mut CampaignBrowserState,
    session: &mut ObserverSession,
    pipe: Option<&RuntimePipe>,
    exits: &mut MessageWriter<AppExit>,
) {
    if let ControlAvailability::Disabled(reason) =
        availability(ObserverCommand::NewCampaign, session)
    {
        reason.clone_into(&mut browser.status);
        return;
    }
    if pipe.is_none() {
        LAUNCHER_REQUIRED.clone_into(&mut browser.status);
        return;
    }
    let Some(selected) = browser.catalog.get(browser.selected) else {
        return;
    };
    let campaign = match parse_campaign(&selected.id) {
        Ok(campaign) => campaign,
        Err(error) => {
            browser.status = error;
            return;
        }
    };
    session.playing = false;
    match preference_path().and_then(|path| write_preference(&path, campaign, browser.generation)) {
        Ok(()) => {
            exits.write(AppExit::Error(
                std::num::NonZeroU8::new(OPEN_SELECTED_EXIT).expect("reserved launcher control"),
            ));
        }
        Err(error) => browser.status = error,
    }
}

fn refresh_scope(
    session: Res<ObserverSession>,
    mut ui: ResMut<ObserverUiState>,
    mut browser: ResMut<CampaignBrowserState>,
) {
    if session.quit_requested {
        return;
    }
    let context = session.context();
    if browser.context.as_ref() != Some(&context) {
        browser.invalidate(context, &mut ui);
    }
    let menu_open = ui.menu_open && !ui.splash_visible && !ui.comparison_open;
    if menu_open && !browser.menu_was_open {
        request_catalog(&mut browser, &session);
    }
    if browser.menu_was_open != menu_open {
        browser.menu_was_open = menu_open;
    }
}

fn collect(session: Res<ObserverSession>, mut browser: ResMut<CampaignBrowserState>) {
    // Polling an unfinished task has no presentation meaning. Only installed
    // results and explicit state changes invalidate the rendered text.
    if let Some((scope, task)) = &mut browser.bypass_change_detection().catalog_task {
        if let Some(result) = block_on(bevy::tasks::futures_lite::future::poll_once(task)) {
            let scope = scope.clone();
            browser.catalog_task = None;
            if browser.accepts(&scope, &session) {
                match result {
                    Ok(catalog) => {
                        browser.selected = catalog
                            .iter()
                            .position(|entry| entry.id != session.campaign.as_uuid().to_string())
                            .unwrap_or(0);
                        browser.catalog = catalog;
                        browser.status = if browser.catalog.is_empty() {
                            "No committed material campaigns are available.".into()
                        } else {
                            String::new()
                        };
                    }
                    Err(error) => browser.status = error,
                }
            }
        }
    }
    if let Some((scope, task)) = &mut browser.bypass_change_detection().comparison_task {
        if let Some(result) = block_on(bevy::tasks::futures_lite::future::poll_once(task)) {
            let scope = scope.clone();
            browser.comparison_task = None;
            if !browser.accepts(&scope, &session) {
                return;
            }
            match result {
                Ok(snapshot) if matches_comparison(&snapshot, &scope) => {
                    browser.comparison = Some(snapshot);
                    browser.status.clear();
                }
                Ok(_) => browser.status =
                    "Comparison campaign, perspective or committed week did not match the request."
                        .into(),
                Err(error) => browser.status = error,
            }
        }
    }
}

fn matches_comparison(snapshot: &ObserverEconomySnapshotV1, scope: &BrowserScope) -> bool {
    let visibility = match scope.active.perspective {
        Perspective::FullObserver => ObserverVisibilityV1::FullObserver,
        Perspective::PlayerKnowledge => ObserverVisibilityV1::KnownPreview,
    };
    scope
        .target
        .is_some_and(|target| snapshot.campaign_id == target.as_uuid().to_string())
        && snapshot.resolve_tick == scope.active.tick
        && snapshot.visibility == visibility
        && snapshot.foundation_digest.len() == 64
        && (snapshot.resolve_tick == 0 || snapshot.tick_content_hash.is_some())
}

fn receipt_text(site: &ProductionSiteV1, tick: u64) -> String {
    match (site.produced_batches, site.planned_batches) {
        (Some(produced), Some(planned)) => format!("{produced}/{planned} batches produced/planned"),
        (None, None) if tick == 0 => "no production receipt at foundation".into(),
        (None, None) => "no production receipt this week".into(),
        _ => "production receipt unavailable".into(),
    }
}

fn comparison_text(
    active: &ObserverEconomySnapshotV1,
    other: &ObserverEconomySnapshotV1,
) -> String {
    let mut output = format!(
        "Week {} | {}\nCurrent {}\nCompared {}\n\n",
        active.resolve_tick,
        match active.visibility {
            ObserverVisibilityV1::FullObserver => "full observer",
            ObserverVisibilityV1::KnownPreview => "player knowledge",
        },
        active.campaign_id,
        other.campaign_id
    );
    let (Some(current), Some(compared)) = (&active.production, &other.production) else {
        output.push_str("Material observations are unavailable in this perspective. Missing knowledge is not zero production.");
        return output;
    };
    writeln!(output, "{}\n{}\nRead the same committed week in both campaigns. No world is advanced by this comparison.\n", current.scenario_label, compared.scenario_label).expect("writing to a String cannot fail");
    for site in &current.sites {
        writeln!(output, "{} | NAICS {}", site.name, site.industry_code)
            .expect("writing to a String cannot fail");
        let Some(other_site) = compared.sites.iter().find(|other| other.id == site.id) else {
            output.push_str("Comparable cohort unavailable.\n\n");
            continue;
        };
        writeln!(
            output,
            "CURRENT  {}\nCOMPARED  {}",
            receipt_text(site, active.resolve_tick),
            receipt_text(other_site, other.resolve_tick)
        )
        .expect("writing to a String cannot fail");
        writeln!(
            output,
            "Next-week capacity: {} / {} batches (current / compared).",
            site.available_batches, other_site.available_batches
        )
        .expect("writing to a String cannot fail");
        for stock in &site.inventory {
            let other_stock = other_site
                .inventory
                .iter()
                .find(|other| other.good_id == stock.good_id && other.unit_id == stock.unit_id);
            let value = other_stock
                .map_or_else(|| "unavailable".into(), |other| other.quantity.to_string());
            writeln!(
                output,
                "{} on hand: {} / {} {}",
                stock.good, stock.quantity, value, stock.unit
            )
            .expect("writing to a String cannot fail");
        }
        output.push('\n');
    }
    output.push_str("Designed labor-hours stay separate from observed QCEW jobs. Terminal goods are unsold on-hand stocks.");
    output
}

#[derive(Default)]
struct BrowserPaintScope {
    context: Option<ObservationContext>,
    catalog_visible: bool,
    comparison_visible: bool,
}

#[derive(SystemParam)]
struct BrowserPaintInput<'w> {
    session: Res<'w, ObserverSession>,
    ui: Res<'w, ObserverUiState>,
    browser: Res<'w, CampaignBrowserState>,
    frame: Res<'w, ObserverFrame>,
}

fn paint(
    input: BrowserPaintInput,
    mut previous: Local<BrowserPaintScope>,
    mut catalog_text: Query<&mut Text, With<CatalogText>>,
    mut comparison_texts: Query<&mut Text, (With<ComparisonText>, Without<CatalogText>)>,
    mut panels: Query<&mut Visibility, With<ComparisonPanel>>,
) {
    let BrowserPaintInput {
        session,
        ui,
        browser,
        frame,
    } = input;
    let valid = browser
        .context
        .as_ref()
        .is_some_and(|context| session.accepts(context));
    let context = session.context();
    let context_changed = previous.context.as_ref() != Some(&context);
    let catalog_visible = valid && ui.menu_open && !ui.splash_visible && !ui.comparison_open;
    let comparison_visible = valid && ui.comparison_open && !ui.menu_open;
    if browser.is_changed() || context_changed || previous.catalog_visible != catalog_visible {
        let value = if !catalog_visible {
            String::new()
        } else if let Some(selected) = browser.catalog.get(browser.selected) {
            format!(
                "{} / {} | {}\n{}\nCommitted week {}\n{}",
                browser.selected + 1,
                browser.catalog.len(),
                selected.label,
                selected.id,
                selected.durable_tick,
                browser.status
            )
        } else {
            browser.status.clone()
        };
        for mut text in &mut catalog_text {
            if text.0 != value {
                text.0.clone_from(&value);
            }
        }
    }
    if previous.comparison_visible != comparison_visible || context_changed {
        let desired = if comparison_visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        for mut visibility in &mut panels {
            visibility.set_if_neq(desired);
        }
    }
    if browser.is_changed()
        || frame.is_changed()
        || context_changed
        || previous.comparison_visible != comparison_visible
    {
        let value = if !comparison_visible {
            String::new()
        } else if let (Some(active), Some(compared)) = (&frame.0, &browser.comparison) {
            if active.campaign_id == context.campaign.as_uuid().to_string()
                && active.resolve_tick == context.tick
                && active.visibility == compared.visibility
            {
                comparison_text(active, compared)
            } else {
                "Waiting for the current campaign's matching observation...".into()
            }
        } else {
            browser.status.clone()
        };
        for mut text in &mut comparison_texts {
            if text.0 != value {
                text.0.clone_from(&value);
            }
        }
    }
    previous.context = Some(context);
    previous.catalog_visible = catalog_visible;
    previous.comparison_visible = comparison_visible;
}

#[derive(QueryData)]
#[query_data(mutable)]
struct BrowserButtonAppearance {
    command: &'static BrowserButton,
    interaction: Ref<'static, Interaction>,
    background: &'static mut BackgroundColor,
    border: &'static mut BorderColor,
}

fn paint_buttons(
    session: Res<ObserverSession>,
    mut buttons: Query<BrowserButtonAppearance, With<BrowserButton>>,
) {
    for mut button in &mut buttons {
        if !session.is_changed() && !button.interaction.is_changed() {
            continue;
        }
        let disabled = matches!(button.command.0, CampaignBrowserCommand::Open)
            && availability(ObserverCommand::NewCampaign, &session) != ControlAvailability::Enabled;
        let background = if !disabled && *button.interaction == Interaction::Pressed {
            theme::BLUE
        } else {
            theme::PANEL
        };
        let border = if disabled {
            theme::GRAY
        } else if *button.interaction == Interaction::None {
            theme::PAPER
        } else {
            theme::YELLOW
        };
        button.background.set_if_neq(BackgroundColor(background));
        button.border.set_if_neq(BorderColor::all(border));
    }
}

fn parse_campaign(value: &str) -> Result<CampaignId, String> {
    let id = uuid::Uuid::parse_str(value)
        .map_err(|_| "Selected campaign identity is invalid.".to_owned())?;
    if id.is_nil() || id.to_string() != value {
        return Err("Selected campaign identity is invalid.".into());
    }
    Ok(CampaignId::from_uuid(id))
}

fn preference_path() -> Result<PathBuf, String> {
    let base =
        if let Some(path) = std::env::var_os("XDG_STATE_HOME").filter(|path| !path.is_empty()) {
            PathBuf::from(path)
        } else {
            PathBuf::from(std::env::var_os("HOME").ok_or_else(|| {
                "Personal campaign preference directory is unavailable.".to_owned()
            })?)
            .join(".local/state")
        };
    if !base.is_absolute() {
        return Err("Personal campaign preference directory must be absolute.".into());
    }
    Ok(base.join("babylon/observer-campaign"))
}

fn write_preference(path: &Path, campaign: CampaignId, generation: u64) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Personal campaign preference directory is unavailable.".to_owned())?;
    fs::create_dir_all(parent)
        .map_err(|_| "Cannot create the personal campaign preference directory.".to_owned())?;
    let mut temporary = None;
    for attempt in 0..8_u8 {
        let candidate = parent.join(format!(
            ".observer-campaign-{}-{generation}-{attempt}.tmp",
            std::process::id()
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&candidate) {
            Ok(file) => {
                temporary = Some((candidate, file));
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return Err("Cannot save the selected campaign preference.".into()),
        }
    }
    let (temporary, mut file) =
        temporary.ok_or_else(|| "Cannot allocate a campaign preference file.".to_owned())?;
    let result = writeln!(file, "{}", campaign.as_uuid())
        .and_then(|()| file.sync_all())
        .and_then(|()| fs::rename(&temporary, path));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|_| "Cannot save the selected campaign preference.".to_owned())
}

pub struct CampaignBrowserPlugin;
impl Plugin for CampaignBrowserPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CampaignBrowserState>()
            .init_resource::<ObserverKeyboardClaim>()
            .add_message::<CampaignBrowserCommand>()
            .add_systems(PostStartup, setup)
            .add_systems(Update, input.in_set(ObserverSet::Input))
            .add_observer(keyboard_button)
            .add_systems(
                PreUpdate,
                sync_focus_targets
                    .in_set(ObserverFocusSystems::Eligibility)
                    .run_if(resource_exists::<ObserverFocusPolicy>),
            )
            .add_systems(
                Update,
                (refresh_scope, commands, collect)
                    .chain()
                    .in_set(ObserverSet::Install),
            )
            .add_systems(Update, (paint, paint_buttons).in_set(ObserverSet::Paint));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog_handoff_app(with_pipe: bool) -> (App, CampaignId) {
        let campaign = CampaignId::from_uuid(uuid::Uuid::from_u128(1));
        let selected = CampaignId::from_uuid(uuid::Uuid::from_u128(2));
        let mut session = ObserverSession::new(campaign);
        session.fail("Runtime disconnected".into());
        let mut app = App::new();
        app.insert_resource(session)
            .insert_resource(CampaignBrowserState {
                catalog: vec![CampaignSummaryV1 {
                    id: selected.as_uuid().to_string(),
                    preset: "standard".into(),
                    label: "Saved campaign".into(),
                    durable_tick: 3,
                }],
                ..default()
            })
            .insert_resource(ObserverUiState {
                menu_open: true,
                splash_visible: false,
                ..default()
            })
            .add_message::<CampaignBrowserCommand>()
            .add_message::<AppExit>()
            .add_systems(Update, commands);
        if with_pipe {
            app.insert_resource(crate::observer_io::RuntimePipe::detached_fixture());
        }
        (app, selected)
    }

    #[test]
    fn launcher_handoff_catalog_without_pipe_preserves_preferences_and_window() {
        let environment = crate::test_support::EnvVarGuard::lock("XDG_STATE_HOME");
        let directory =
            std::env::temp_dir().join(format!("babylon-detached-handoff-{}", uuid::Uuid::new_v4()));
        environment.set(directory.to_str().unwrap());
        let path = preference_path().unwrap();
        let original = CampaignId::from_uuid(uuid::Uuid::from_u128(1));
        write_preference(&path, original, 0).unwrap();
        let (mut app, _) = catalog_handoff_app(false);
        app.world_mut()
            .resource_mut::<Messages<CampaignBrowserCommand>>()
            .write(CampaignBrowserCommand::Open);
        app.update();
        assert!(
            app.world().resource::<Messages<AppExit>>().is_empty(),
            "Opening a saved campaign closed a standalone window"
        );
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            format!("{}\n", original.as_uuid())
        );
        assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
        assert_eq!(
            app.world().resource::<CampaignBrowserState>().status,
            "This window has no launcher connection. Close it and start Babylon through its launcher."
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn launcher_handoff_catalog_with_pipe_writes_selected_campaign_after_failure() {
        let environment = crate::test_support::EnvVarGuard::lock("XDG_STATE_HOME");
        let directory = std::env::temp_dir().join(format!(
            "babylon-connected-handoff-{}",
            uuid::Uuid::new_v4()
        ));
        environment.set(directory.to_str().unwrap());
        let (mut app, selected) = catalog_handoff_app(true);
        app.world_mut()
            .resource_mut::<Messages<CampaignBrowserCommand>>()
            .write(CampaignBrowserCommand::Open);
        app.update();
        assert!(matches!(
            app.world_mut().resource_mut::<Messages<AppExit>>().drain().collect::<Vec<_>>().as_slice(),
            [AppExit::Error(code)] if code.get() == OPEN_SELECTED_EXIT
        ));
        assert_eq!(
            fs::read_to_string(preference_path().unwrap()).unwrap(),
            format!("{}\n", selected.as_uuid())
        );
        fs::remove_dir_all(directory).unwrap();
    }

    fn ready_catalog_handoff_app() -> (App, CampaignId) {
        let (mut app, selected) = catalog_handoff_app(true);
        let context = {
            let mut session = app.world_mut().resource_mut::<ObserverSession>();
            session.ready(3, None);
            let context = session.context();
            assert!(session.installed(&context));
            context
        };
        app.world_mut()
            .resource_mut::<CampaignBrowserState>()
            .context = Some(context);
        (app, selected)
    }

    #[test]
    fn catalog_handoff_pending_commit_refuses_queued_and_stale_enabled_open() {
        let environment = crate::test_support::EnvVarGuard::lock("XDG_STATE_HOME");
        let directory =
            std::env::temp_dir().join(format!("babylon-pending-handoff-{}", uuid::Uuid::new_v4()));
        environment.set(directory.to_str().unwrap());
        for (keyboard, failed) in [(false, false), (true, false), (true, true)] {
            let (mut app, selected) = ready_catalog_handoff_app();
            let active = app.world().resource::<ObserverSession>().campaign;
            let context = app.world().resource::<ObserverSession>().context();
            let path = preference_path().unwrap();
            write_preference(&path, active, 0).unwrap();
            app.add_observer(keyboard_button);
            let mut target = ObserverFocusTarget::action(Some(context.clone()));
            target.available = true;
            let button = app
                .world_mut()
                .spawn((BrowserButton(CampaignBrowserCommand::Open), target))
                .id();
            let request = {
                let mut session = app.world_mut().resource_mut::<ObserverSession>();
                let request = session.begin_advance().unwrap();
                if failed {
                    session.fail("Commit acknowledgement lost".into());
                }
                request
            };
            if keyboard {
                // The rendered control was enabled before this week began.
                app.world_mut().trigger(ObserverKeyboardActivate {
                    entity: button,
                    context: Some(context.clone()),
                });
            } else {
                app.world_mut()
                    .resource_mut::<Messages<CampaignBrowserCommand>>()
                    .write(CampaignBrowserCommand::Open);
            }
            app.update();
            assert!(
                app.world().resource::<Messages<AppExit>>().is_empty(),
                "Saved-campaign handoff abandoned the pending commit"
            );
            let session = app.world().resource::<ObserverSession>();
            assert!(session.advance_pending());
            assert_eq!(session.context(), context);
            assert_eq!(session.campaign, active);
            assert_eq!(session.durable_tick, 3);
            assert_eq!(
                fs::read_to_string(&path).unwrap(),
                format!("{}\n", active.as_uuid())
            );
            assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
            assert_eq!(
                app.world().resource::<CampaignBrowserState>().status,
                if failed {
                    "Reopen the campaign to reconcile committed progress"
                } else {
                    "Wait for the current week to finish committing"
                }
            );
            assert!(app
                .world_mut()
                .resource_mut::<ObserverSession>()
                .acknowledge(request, 4, None));
            app.world_mut()
                .resource_mut::<Messages<CampaignBrowserCommand>>()
                .write(CampaignBrowserCommand::Open);
            app.update();
            assert!(matches!(
                app.world_mut().resource_mut::<Messages<AppExit>>().drain().collect::<Vec<_>>().as_slice(),
                [AppExit::Error(code)] if code.get() == OPEN_SELECTED_EXIT
            ));
            assert_eq!(
                fs::read_to_string(&path).unwrap(),
                format!("{}\n", selected.as_uuid())
            );
        }
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn catalog_open_presentation_disables_pending_commit_without_pointer_motion() {
        let (mut app, _) = ready_catalog_handoff_app();
        app.add_systems(PreUpdate, sync_focus_targets)
            .add_systems(Update, paint_buttons.after(commands));
        let button = app
            .world_mut()
            .spawn((
                BrowserButton(CampaignBrowserCommand::Open),
                ObserverFocusTarget::action(None),
                Interaction::Hovered,
                BackgroundColor(theme::PANEL),
                BorderColor::all(theme::PAPER),
            ))
            .id();
        app.update();
        assert!(
            app.world()
                .get::<ObserverFocusTarget>(button)
                .unwrap()
                .available
        );
        assert_eq!(
            *app.world().get::<BorderColor>(button).unwrap(),
            BorderColor::all(theme::YELLOW)
        );
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .begin_advance()
            .unwrap();
        app.update();
        assert!(
            !app.world()
                .get::<ObserverFocusTarget>(button)
                .unwrap()
                .available
        );
        assert_eq!(
            *app.world().get::<BorderColor>(button).unwrap(),
            BorderColor::all(theme::GRAY)
        );
        assert_eq!(
            *app.world().get::<Interaction>(button).unwrap(),
            Interaction::Hovered
        );
    }

    #[test]
    fn browser_keyboard_request_revalidates_perspective_and_visible_menu() {
        let session = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::from_u128(1)));
        let original = session.context();
        let mut app = App::new();
        app.insert_resource(session)
            .insert_resource(ObserverUiState {
                splash_visible: false,
                ..default()
            })
            .add_message::<CampaignBrowserCommand>()
            .add_observer(keyboard_button);
        let button = app
            .world_mut()
            .spawn((
                BrowserButton(CampaignBrowserCommand::Refresh),
                ObserverFocusTarget::action(Some(original.clone())),
            ))
            .id();
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity: button,
            context: Some(original.clone()),
        });
        assert!(matches!(
            app.world_mut()
                .resource_mut::<Messages<CampaignBrowserCommand>>()
                .drain()
                .collect::<Vec<_>>()
                .as_slice(),
            [CampaignBrowserCommand::Refresh]
        ));
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .perspective = Perspective::PlayerKnowledge;
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity: button,
            context: Some(original),
        });
        assert!(app
            .world()
            .resource::<Messages<CampaignBrowserCommand>>()
            .is_empty());
        let current = app.world().resource::<ObserverSession>().context();
        app.world_mut()
            .get_mut::<ObserverFocusTarget>(button)
            .unwrap()
            .context = Some(current.clone());
        app.world_mut().resource_mut::<ObserverUiState>().menu_open = false;
        app.world_mut().trigger(ObserverKeyboardActivate {
            entity: button,
            context: Some(current),
        });
        assert!(app
            .world()
            .resource::<Messages<CampaignBrowserCommand>>()
            .is_empty());
    }

    #[test]
    fn shutdown_consumes_queued_browser_commands_without_changing_selection_or_exiting_again() {
        let mut session = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::from_u128(1)));
        session.quit_requested = true;
        let browser = CampaignBrowserState {
            // Invalid fixture identities make a regression incapable of writing user preferences.
            catalog: ["unavailable-a", "unavailable-b"]
                .map(|id| CampaignSummaryV1 {
                    id: id.into(),
                    preset: "standard".into(),
                    label: id.into(),
                    durable_tick: 3,
                })
                .into(),
            ..default()
        };
        let mut app = App::new();
        app.insert_resource(session)
            .insert_resource(browser)
            .init_resource::<ObserverUiState>()
            .add_message::<CampaignBrowserCommand>()
            .add_message::<AppExit>()
            .add_systems(Update, commands);
        app.world_mut()
            .resource_mut::<Messages<CampaignBrowserCommand>>()
            .write_batch([CampaignBrowserCommand::Next, CampaignBrowserCommand::Open]);
        app.update();
        let browser = app.world().resource::<CampaignBrowserState>();
        assert_eq!(browser.selected, 0);
        assert!(browser.status.starts_with("Closing the campaign"));
        assert!(app.world().resource::<Messages<AppExit>>().is_empty());
        assert!(browser.catalog_task.is_none() && browser.comparison_task.is_none());
    }

    #[test]
    fn comparison_scope_rejects_changed_perspective_week_campaign_and_generation() {
        let first = parse_campaign("81b979ee-a9c1-48fd-8835-06cbfe594675").unwrap();
        let other = parse_campaign("fc7d28a0-a29a-49ea-bf3b-ef07ee163cd4").unwrap();
        let mut session = ObserverSession::new(first);
        let mut browser = CampaignBrowserState::default();
        browser.invalidate(session.context(), &mut ObserverUiState::default());
        browser.comparison_target = Some(other);
        let scope = BrowserScope {
            active: session.context(),
            generation: 0,
            target: Some(other),
        };
        assert!(browser.accepts(&scope, &session));
        session.set_perspective(Perspective::PlayerKnowledge);
        assert!(!browser.accepts(&scope, &session));
        session = ObserverSession::new(first);
        session.viewed_tick = 1;
        assert!(!browser.accepts(&scope, &session));
        session = ObserverSession::new(other);
        assert!(!browser.accepts(&scope, &session));
        session = ObserverSession::new(first);
        browser.generation = 1;
        assert!(!browser.accepts(&scope, &session));
    }

    #[test]
    fn pending_comparison_stays_modal_without_repainting_until_scope_changes() {
        #[derive(Resource, Default)]
        struct Changes(u32);

        fn record_changes(browser: Res<CampaignBrowserState>, mut changes: ResMut<Changes>) {
            if browser.is_changed() {
                changes.0 += 1;
            }
        }

        let campaign = parse_campaign("81b979ee-a9c1-48fd-8835-06cbfe594675").unwrap();
        let other = parse_campaign("fc7d28a0-a29a-49ea-bf3b-ef07ee163cd4").unwrap();
        let session = ObserverSession::new(campaign);
        let scope = BrowserScope {
            active: session.context(),
            generation: 0,
            target: Some(other),
        };
        let pool = AsyncComputeTaskPool::get_or_init(|| {
            bevy::tasks::TaskPoolBuilder::new().num_threads(1).build()
        });
        let browser = CampaignBrowserState {
            context: Some(session.context()),
            comparison_target: Some(other),
            comparison_task: Some((scope, pool.spawn(std::future::pending()))),
            ..default()
        };
        let mut app = App::new();
        app.insert_resource(session)
            .insert_resource(browser)
            .insert_resource(ObserverUiState {
                menu_open: false,
                splash_visible: false,
                comparison_open: true,
                ..default()
            })
            .init_resource::<Changes>()
            .add_systems(Update, (refresh_scope, collect, record_changes).chain());
        app.update();
        app.update();
        assert!(app.world().resource::<ObserverUiState>().comparison_open);
        let browser = app.world().resource::<CampaignBrowserState>();
        assert!(browser.comparison_task.is_some());
        assert!(browser.comparison.is_none());
        assert_eq!(app.world().resource::<Changes>().0, 1);

        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(Perspective::PlayerKnowledge);
        app.update();
        assert!(!app.world().resource::<ObserverUiState>().comparison_open);
        let browser = app.world().resource::<CampaignBrowserState>();
        assert!(browser.comparison_task.is_none());
        assert!(browser.comparison_target.is_none());
        assert_eq!(app.world().resource::<Changes>().0, 2);
    }

    #[test]
    fn catalog_preference_is_exact_personal_uuid_and_atomic_replacement() {
        let root = std::env::temp_dir().join(format!(
            "babylon-campaign-preference-{}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let path = root.join("observer-campaign");
        let first = parse_campaign("81b979ee-a9c1-48fd-8835-06cbfe594675").unwrap();
        let next = parse_campaign("fc7d28a0-a29a-49ea-bf3b-ef07ee163cd4").unwrap();
        write_preference(&path, first, 0).unwrap();
        write_preference(&path, next, 1).unwrap();
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            format!("{}\n", next.as_uuid())
        );
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_file(path).unwrap();
        fs::remove_dir(root).unwrap();
        assert!(parse_campaign("not-a-campaign").is_err());
        assert!(parse_campaign("00000000-0000-0000-0000-000000000000").is_err());
    }
}
