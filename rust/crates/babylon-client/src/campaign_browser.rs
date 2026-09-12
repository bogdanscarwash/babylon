//! Native campaign catalog and read-only comparison of separately committed worlds.

mod aggregate;

use crate::workforce::{validate_staffing_period, StaffingIdentity};

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use babylon_persistence::{
    identity::CampaignId, observer_reader::CampaignSummary, observer_reader::ObserverEconomyReader,
    observer_reader::ObserverEconomySnapshot, observer_reader::ObserverVisibility,
    production_observation::ProductionStaffingAccount,
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

type CatalogTask = (BrowserScope, Task<Result<Vec<CampaignSummary>, String>>);
type ComparisonTask = (BrowserScope, Task<Result<ObserverEconomySnapshot, String>>);

#[derive(Resource, Default)]
struct CampaignBrowserState {
    context: Option<ObservationContext>,
    generation: u64,
    catalog_task: Option<CatalogTask>,
    comparison_task: Option<ComparisonTask>,
    catalog: Vec<CampaignSummary>,
    selected: usize,
    comparison: Option<ObserverEconomySnapshot>,
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

fn reader(perspective: Perspective) -> Result<ObserverEconomyReader, String> {
    match perspective {
        Perspective::FullObserver => ObserverEconomyReader::from_observer_env(),
        Perspective::PlayerKnowledge => ObserverEconomyReader::from_known_env(),
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
    pipe: Option<Res<RuntimePipe>>,
) {
    for command in messages.read() {
        if session.quit_requested {
            "Closing the campaign; committed periods are saved automatically."
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
                open_selected_campaign(&mut browser, &mut session, pipe.as_deref());
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
                    browser.status = format!(
                        "That campaign is committed only through period {}. Inspect that period or an earlier period first.",
                        selected.durable_tick
                    );
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
                browser.status = "Loading the other campaign's committed period...".into();
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
    match session.queue_campaign(
        babylon_persistence::runtime_session::RuntimeSessionTarget::Open {
            campaign_id: campaign.as_uuid().to_string(),
        },
    ) {
        Ok(()) => browser.status = "Opening the selected campaign...".into(),
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
                    "Comparison campaign, perspective or committed period did not match the request."
                        .into(),
                Err(error) => browser.status = error,
            }
        }
    }
}

fn matches_comparison(snapshot: &ObserverEconomySnapshot, scope: &BrowserScope) -> bool {
    let visibility = match scope.active.perspective {
        Perspective::FullObserver => ObserverVisibility::FullObserver,
        Perspective::PlayerKnowledge => ObserverVisibility::KnownPreview,
    };
    scope
        .target
        .is_some_and(|target| snapshot.campaign_id == target.as_uuid().to_string())
        && snapshot.resolve_tick == scope.active.tick
        && snapshot.visibility == visibility
        && snapshot.foundation_digest.len() == 64
        && (snapshot.resolve_tick == 0 || snapshot.tick_content_hash.is_some())
}

fn receipt_text(
    site: &babylon_persistence::production_observation::ProductionProcess,
    tick: u64,
) -> String {
    match (site.produced_batches, site.planned_batches) {
        (Some(produced), Some(planned)) => format!("{produced}/{planned} batches produced/planned"),
        (None, None) if tick == 0 => "no production receipt at foundation".into(),
        (None, None) => "no production receipt this period".into(),
        _ => "production receipt unavailable".into(),
    }
}

fn staffing_difference(output: &mut String, label: &str, current: u64, compared: u64) {
    let difference = i128::from(current) - i128::from(compared);
    writeln!(
        output,
        "{label}: {current} / {compared} people | difference {difference:+}"
    )
    .expect("writing to a String cannot fail");
}

fn compare_staffing(
    output: &mut String,
    site_id: &str,
    tick: u64,
    current: &[ProductionStaffingAccount],
    compared: &[ProductionStaffingAccount],
) {
    let identities: BTreeSet<_> = current
        .iter()
        .chain(compared)
        .filter(|account| account.site_id == site_id)
        .map(StaffingIdentity::from)
        .collect();
    if identities.is_empty() {
        output.push_str(
            "Modeled workforce unavailable: no staffing account disclosed for this cohort.\n",
        );
        return;
    }
    for identity in identities {
        let mut current_accounts = current
            .iter()
            .filter(|account| StaffingIdentity::from(*account) == identity);
        let mut compared_accounts = compared
            .iter()
            .filter(|account| StaffingIdentity::from(*account) == identity);
        let (Some(current), Some(compared)) = (current_accounts.next(), compared_accounts.next())
        else {
            output.push_str("Modeled workforce unavailable: no matching pool, site and labor unit in both campaigns.\n");
            continue;
        };
        if current_accounts.next().is_some() || compared_accounts.next().is_some() {
            output
                .push_str("Modeled workforce unavailable: duplicate pool, site and labor unit.\n");
            continue;
        }
        writeln!(output, "Modeled workforce: {}", current.subject.local_name)
            .expect("writing to a String cannot fail");
        if let Err(error) = validate_staffing_period(current, tick) {
            writeln!(output, "CURRENT workforce unavailable: {error}.")
                .expect("writing to a String cannot fail");
            continue;
        }
        if let Err(error) = validate_staffing_period(compared, tick) {
            writeln!(output, "COMPARED workforce unavailable: {error}.")
                .expect("writing to a String cannot fail");
            continue;
        }
        let (employed, reserve) = if tick == 0 {
            ("Foundation employed", "Foundation reserve")
        } else {
            ("Closing employed", "Closing reserve")
        };
        staffing_difference(output, employed, current.employed, compared.employed);
        staffing_difference(output, reserve, current.reserve, compared.reserve);
        if let (Some(current), Some(compared)) = (&current.completed, &compared.completed) {
            staffing_difference(output, "Hires this period", current.hires, compared.hires);
            staffing_difference(
                output,
                "Separations this period",
                current.separations,
                compared.separations,
            );
        } else {
            output.push_str("No completed staffing receipt at foundation; hires and separations are unavailable.\n");
        }
    }
}

fn comparison_cohort_ids<'a>(
    current: &'a babylon_persistence::production_observation::ProductionSnapshot,
    selected_site: Option<&str>,
) -> BTreeSet<&'a str> {
    let mut cohort_ids = BTreeSet::new();
    if let Some(site) = selected_site.and_then(|id| current.sites.iter().find(|site| site.id == id))
    {
        cohort_ids.insert(site.id.as_str());
        for (_, neighbor) in crate::production_brief::dependency_sites(site, current)
            .into_iter()
            .take(6)
        {
            cohort_ids.insert(neighbor.id.as_str());
        }
    } else {
        for id in current
            .sites
            .iter()
            .map(|site| site.id.as_str())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .take(6)
        {
            cohort_ids.insert(id);
        }
    }
    cohort_ids
}

fn comparison_text(
    active: &ObserverEconomySnapshot,
    other: &ObserverEconomySnapshot,
    selected_site: Option<&str>,
    lens: &crate::map_economy_lens::MapLens,
) -> String {
    let mut output = format!(
        "Period {} | {}\nCurrent {}\nCompared {}\n\n",
        active.resolve_tick,
        match active.visibility {
            ObserverVisibility::FullObserver => "full observer",
            ObserverVisibility::KnownPreview => "player knowledge",
        },
        active.campaign_id,
        other.campaign_id
    );
    if active.visibility != ObserverVisibility::FullObserver
        || other.visibility != ObserverVisibility::FullObserver
        || active.resolve_tick != other.resolve_tick
    {
        output.push_str(
            "Material comparison unavailable: matching full-observer periods are required.",
        );
        return output;
    }
    let (Some(current), Some(compared)) = (&active.production, &other.production) else {
        output.push_str("Material observations are unavailable in this perspective. Missing knowledge is not zero production.");
        return output;
    };
    writeln!(output, "{}\n{}\nRead the same committed period in both campaigns. No world is advanced by this comparison.\nCounts read current / compared. Signed difference = current minus compared.\n", current.scenario_label, compared.scenario_label).expect("writing to a String cannot fail");
    aggregate::write(&mut output, active, other, lens);
    let cohort_ids = comparison_cohort_ids(current, selected_site);
    writeln!(output, "Comparing {} disclosed cohorts; select a cohort in Circuit to compare its neighborhood. Materials retain their exact good and unit identities.", cohort_ids.len()).expect("String write");
    output.push_str(&crate::production_freight::comparison_reading(
        active.resolve_tick,
        current,
        compared,
        selected_site,
    ));
    for site in current
        .sites
        .iter()
        .filter(|site| cohort_ids.contains(site.id.as_str()))
    {
        writeln!(output, "{} | NAICS {}", site.name, site.industry_code)
            .expect("writing to a String cannot fail");
        let Some(other_site) = compared.sites.iter().find(|other| other.id == site.id) else {
            output.push_str("Comparable cohort unavailable.\n\n");
            continue;
        };
        for process in &site.processes {
            let Some(other_process) = other_site.processes.iter().find(|other| {
                other.id == process.id
                    && other.output_good_id == process.output_good_id
                    && other.output_unit_id == process.output_unit_id
            }) else {
                writeln!(output, "{}: compatible process unavailable.", process.name)
                    .expect("String write");
                continue;
            };
            writeln!(
                output,
                "{} / {} ({})\nCURRENT  {}\nCOMPARED  {}\nNext-period capacity: {} / {} batches.",
                process.name,
                process.output_good,
                process.output_unit,
                receipt_text(process, active.resolve_tick),
                receipt_text(other_process, other.resolve_tick),
                process.available_batches,
                other_process.available_batches
            )
            .expect("String write");
        }
        if site.processes.is_empty() {
            output.push_str("Merchant owner / handling and distribution; no productive output.\n");
        }
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
        compare_staffing(
            &mut output,
            &site.id,
            active.resolve_tick,
            &current.staffing_accounts,
            &compared.staffing_accounts,
        );
        output.push('\n');
    }
    output.push_str("Modeled workforce counts are people, separate from observed QCEW jobs and labor-hours. Staffing receipts do not record wage payments or class migration. Retail fulfillment records delivery to final demand; remaining inventory stays on hand.");
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
    navigation: Option<Res<'w, crate::production::ProductionNavigation>>,
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
        navigation,
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
                "{} / {} | {}\n{}\nCommitted period {}\n{}",
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
        || ui.is_changed()
        || navigation.as_ref().is_some_and(DetectChanges::is_changed)
        || context_changed
        || previous.comparison_visible != comparison_visible
    {
        let value = if !comparison_visible {
            String::new()
        } else if let (Some(active), Some(compared)) = (&frame.0, &browser.comparison) {
            let scope = BrowserScope {
                active: context.clone(),
                generation: browser.generation,
                target: browser.comparison_target,
            };
            if active.campaign_id == context.campaign.as_uuid().to_string()
                && active.resolve_tick == context.tick
                && active.visibility == compared.visibility
                && browser.comparison_target != Some(context.campaign)
                && matches_comparison(compared, &scope)
            {
                comparison_text(
                    active,
                    compared,
                    navigation
                        .as_ref()
                        .and_then(|navigation| navigation.selected_site.as_deref()),
                    &ui.lens,
                )
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

pub(crate) fn preference_path() -> Result<PathBuf, String> {
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

pub(crate) fn write_preference(
    path: &Path,
    campaign: CampaignId,
    generation: u64,
) -> Result<(), String> {
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
    use babylon_persistence::production_observation::ProductionSite;

    fn staffing_snapshot(
        campaign: CampaignId,
        tick: u64,
        employed: u64,
        hires: u64,
        separations: u64,
    ) -> ObserverEconomySnapshot {
        use babylon_persistence::{
            production_observation::CompletedProductionStaffing,
            production_observation::ProductionSnapshot,
            production_observation::ProductionStaffingSubject,
        };

        let site_id = "1".repeat(64);
        let opening_employed = employed + separations - hires;
        ObserverEconomySnapshot {
            campaign_id: campaign.as_uuid().to_string(),
            resolve_tick: tick,
            foundation_digest: "a".repeat(64),
            nominal_world_hash: Some("b".repeat(64)),
            tick_content_hash: (tick > 0).then(|| "c".repeat(64)),
            envelope_digest: (tick > 0).then(|| "d".repeat(64)),
            visibility: ObserverVisibility::FullObserver,
            counties: Vec::new(),
            production: Some(ProductionSnapshot {
                content_authority_sha256: "a".repeat(64),
                road_source: None,
                physical_edges: Vec::new(),
                merchant_handling_accounts: Vec::new(),
                final_demand_accounts: Vec::new(),
                freight_capacity_accounts: Vec::new(),
                scenario_label: "Staffing comparison fixture".into(),
                horizon_period: 520,
                sites: vec![ProductionSite {
                    id: site_id.clone(),
                    county_geoid: "26163".into(),
                    name: "Wayne manufacturing cohort".into(),
                    industry_code: "331".into(),
                    observed_employment: Some(20),
                    inventory: Vec::new(),
                    role:
                        babylon_persistence::production_observation::ProductionSiteRole::Production,
                    sector_code: "31-33".into(),
                    processes: vec![
                        babylon_persistence::production_observation::ProductionProcess {
                            id: "fixture-process".into(),
                            name: "Fixture process".into(),
                            output_good_id: "4".repeat(64),
                            output_unit_id: "5".repeat(64),
                            output_good: "steel".into(),
                            output_unit: "kg".into(),
                            output_per_batch: 10,
                            available_batches: 8,
                            planned_batches: (tick > 0).then_some(8),
                            produced_batches: (tick > 0).then_some(7),
                            inputs: Vec::new(),
                            labor: Vec::new(),
                        },
                    ],
                }],
                staffing_accounts: vec![ProductionStaffingAccount {
                    pool_id: "2".repeat(64),
                    site_id,
                    unit_id: "3".repeat(64),
                    subject: ProductionStaffingSubject {
                        scenario: "fixture".into(),
                        local_name: "Cohort workforce".into(),
                    },
                    hours_per_person: 40,
                    labor_force: 10,
                    employed,
                    reserve: 10 - employed,
                    previous_unretained_hours: 0,
                    next_opening_period: tick + 1,
                    next_opening_hours: employed * 40,
                    completed: (tick > 0).then_some(CompletedProductionStaffing {
                        period: tick,
                        opening_employed,
                        opening_reserve: 10 - opening_employed,
                        previous_unretained_hours: 0,
                        current_unretained_hours: 0,
                        retained_hours: employed * 40,
                        target_employed: employed,
                        hires,
                        separations,
                    }),
                }],
                routes: Vec::new(),
                freight: Vec::new(),
                events: Vec::new(),
                labor_accounts: Vec::new(),
                material_balance: None,
                observed_contexts: Vec::new(),
                process_attributions: Vec::new(),
                provenance: Vec::new(),
            }),
        }
    }

    fn staffing_comparison_app(tick: u64) -> (App, Entity) {
        let campaign = CampaignId::from_uuid(uuid::Uuid::from_u128(1));
        let other = CampaignId::from_uuid(uuid::Uuid::from_u128(2));
        let mut session = ObserverSession::new(campaign);
        session.ready(tick, None);
        let context = session.context();
        let mut app = App::new();
        app.insert_resource(session)
            .insert_resource(CampaignBrowserState {
                context: Some(context),
                comparison_target: Some(other),
                comparison: Some(staffing_snapshot(other, tick, 4, 0, 1)),
                ..default()
            })
            .insert_resource(ObserverFrame(Some(staffing_snapshot(
                campaign, tick, 6, 2, 0,
            ))))
            .insert_resource(ObserverUiState {
                menu_open: false,
                splash_visible: false,
                comparison_open: true,
                ..default()
            })
            .add_systems(Update, paint);
        let text = app.world_mut().spawn((Text::new(""), ComparisonText)).id();
        (app, text)
    }

    fn painted_comparison(app: &mut App, text: Entity) -> String {
        app.update();
        app.world().get::<Text>(text).unwrap().0.clone()
    }

    fn edit_comparison_production(
        app: &mut App,
        mut edit: impl FnMut(&mut babylon_persistence::production_observation::ProductionSnapshot),
    ) {
        edit(
            app.world_mut()
                .resource_mut::<ObserverFrame>()
                .0
                .as_mut()
                .unwrap()
                .production
                .as_mut()
                .unwrap(),
        );
        edit(
            app.world_mut()
                .resource_mut::<CampaignBrowserState>()
                .comparison
                .as_mut()
                .unwrap()
                .production
                .as_mut()
                .unwrap(),
        );
    }

    fn retail_comparison_app(tick: u64) -> (App, Entity) {
        use babylon_persistence::{
            production_observation::CompletedProductionFinalDemand,
            production_observation::ProductionFinalDemandAccount,
            production_observation::ProductionFinalDemandOrder,
            production_observation::ProductionSiteRole, production_observation::ProductionStock,
        };
        let (mut app, text) = staffing_comparison_app(tick);
        edit_comparison_production(&mut app, |snapshot| {
            let mut retailer = snapshot.sites[0].clone();
            retailer.id = "retailer".into();
            retailer.role = ProductionSiteRole::Retail;
            retailer.processes.clear();
            let mut workforce = snapshot.staffing_accounts[0].clone();
            workforce.pool_id = "retail-workforce".into();
            workforce.site_id.clone_from(&retailer.id);
            snapshot.staffing_accounts.push(workforce);
            for (unit_id, unit, stock, fulfilled) in [("5", "kg", 10, 3), ("6", "tonne", 900, 11)] {
                let fulfilled = if tick == 0 { 0 } else { fulfilled };
                retailer.inventory.push(ProductionStock {
                    good_id: "4".repeat(64),
                    unit_id: unit_id.repeat(64),
                    good: "Steel".into(),
                    unit: unit.into(),
                    quantity: stock,
                });
                snapshot
                    .final_demand_accounts
                    .push(ProductionFinalDemandAccount {
                        demand_principal_id: format!("demand-{unit}"),
                        county_geoid: "26163".into(),
                        good_id: "4".repeat(64),
                        unit_id: unit_id.repeat(64),
                        good: "Steel".into(),
                        unit: unit.into(),
                        ordered: 20,
                        fulfilled,
                        outstanding: 20 - fulfilled,
                        retail_stock_on_hand: stock,
                        retailer_site_ids: vec![retailer.id.clone()],
                        orders: vec![ProductionFinalDemandOrder {
                            order_id: format!("retail-order-{unit}"),
                            retailer_site_id: retailer.id.clone(),
                            ordered: 20,
                            fulfilled,
                            outstanding: 20 - fulfilled,
                        }],
                        completed: (tick > 0).then_some(CompletedProductionFinalDemand {
                            period: tick,
                            opening_fulfilled: 0,
                            newly_fulfilled: fulfilled,
                            closing_fulfilled: fulfilled,
                        }),
                    });
            }
            snapshot.sites.push(retailer);
        });
        app.world_mut().resource_mut::<ObserverUiState>().lens =
            crate::map_economy_lens::MapLens::Material {
                kind: crate::map_economy_lens::MaterialLensKind::OnHand,
                good: Some(crate::map_economy_lens::MaterialGoodKey {
                    good_id: "4".repeat(64),
                    unit_id: "5".repeat(64),
                }),
            };
        (app, text)
    }

    #[test]
    fn comparison_aggregate_covers_every_owner_once_beyond_neighborhood_paging() {
        let (mut app, text) = staffing_comparison_app(2);
        edit_comparison_production(&mut app, |snapshot| {
            for index in 0..7 {
                let mut site = snapshot.sites[0].clone();
                site.id = format!("owner-{index}");
                site.processes[0].id = format!("process-{index}");
                let mut workforce = snapshot.staffing_accounts[0].clone();
                workforce.site_id.clone_from(&site.id);
                workforce.pool_id = format!("workforce-{index}");
                snapshot.sites.push(site);
                snapshot.staffing_accounts.push(workforce);
            }
        });
        app.world_mut()
            .resource_mut::<CampaignBrowserState>()
            .comparison
            .as_mut()
            .unwrap()
            .production
            .as_mut()
            .unwrap()
            .staffing_accounts
            .reverse();
        let reading = painted_comparison(&mut app, text);
        assert!(reading.contains("MODELED CAMPAIGN TOTALS / 8 owners / 1 counties"));
        assert_eq!(
            reading
                .matches("All modeled employed: 48 / 32 people")
                .count(),
            1
        );
        assert!(reading.contains("All modeled reserve: 32 / 48 people"));
        assert!(reading.contains("Comparing 6 disclosed cohorts"));
    }

    #[test]
    fn comparison_aggregate_refuses_duplicate_principals_and_incompatible_owner_coverage() {
        for mismatch in [
            "duplicate workforce",
            "duplicate owner",
            "owner coverage",
            "missing workforce",
            "bad balance",
            "overflow",
        ] {
            let (mut app, text) = staffing_comparison_app(2);
            {
                let mut browser = app.world_mut().resource_mut::<CampaignBrowserState>();
                let snapshot = browser
                    .comparison
                    .as_mut()
                    .unwrap()
                    .production
                    .as_mut()
                    .unwrap();
                match mismatch {
                    "duplicate workforce" => snapshot
                        .staffing_accounts
                        .push(snapshot.staffing_accounts[0].clone()),
                    "duplicate owner" => snapshot.sites.push(snapshot.sites[0].clone()),
                    "owner coverage" => snapshot.sites[0].county_geoid = "26001".into(),
                    "missing workforce" => snapshot.staffing_accounts.clear(),
                    "bad balance" => snapshot.staffing_accounts[0].reserve = 100,
                    "overflow" => snapshot.staffing_accounts[0].employed = u64::MAX,
                    _ => unreachable!(),
                }
            }
            let reading = painted_comparison(&mut app, text);
            assert!(
                reading.contains("Aggregate workforce unavailable:"),
                "{mismatch}: {reading}"
            );
            assert!(
                !reading.contains("All modeled employed:"),
                "{mismatch}: {reading}"
            );
        }
    }

    #[test]
    fn comparison_aggregate_uses_the_exact_world_good_and_unit_and_repaints_selection() {
        let (mut app, text) = retail_comparison_app(2);
        let reading = painted_comparison(&mut app, text);
        assert!(reading.contains("Selected material / Steel / kg"));
        assert!(reading.contains("Inventory on hand: 10 / 10 kg"));
        assert!(reading.contains("Delivered to end buyers to date: 3 / 3 kg"));
        assert!(reading.contains("Unsold retail stock: 10 / 10 kg"));
        if let crate::map_economy_lens::MapLens::Material {
            good: Some(good), ..
        } = &mut app.world_mut().resource_mut::<ObserverUiState>().lens
        {
            good.unit_id = "6".repeat(64);
        }
        let reading = painted_comparison(&mut app, text);
        assert!(reading.contains("Inventory on hand: 900 / 900 tonne"));
        assert!(reading.contains("Delivered to end buyers to date: 11 / 11 tonne"));
        assert!(reading.contains("Unsold retail stock: 900 / 900 tonne"));
        assert!(!reading.contains("Inventory on hand: 910"));
        app.world_mut().resource_mut::<ObserverUiState>().lens =
            crate::map_economy_lens::MapLens::default();
        let reading = painted_comparison(&mut app, text);
        assert!(reading.contains("Select an exact good and unit in World's material lens"));
        assert!(!reading.contains("Delivered to end buyers to date:"));
    }

    #[test]
    fn comparison_aggregate_refuses_missing_or_duplicate_selected_material_principals() {
        for mismatch in [
            "missing stock",
            "duplicate stock",
            "wrong unit",
            "duplicate process",
        ] {
            let (mut app, text) = retail_comparison_app(2);
            if mismatch == "duplicate process" {
                if let crate::map_economy_lens::MapLens::Material { kind, .. } =
                    &mut app.world_mut().resource_mut::<ObserverUiState>().lens
                {
                    *kind = crate::map_economy_lens::MaterialLensKind::ProducedThisPeriod;
                }
            }
            {
                let mut browser = app.world_mut().resource_mut::<CampaignBrowserState>();
                let snapshot = browser
                    .comparison
                    .as_mut()
                    .unwrap()
                    .production
                    .as_mut()
                    .unwrap();
                match mismatch {
                    "missing stock" => {
                        snapshot.sites[1].inventory.remove(0);
                    }
                    "duplicate stock" => {
                        let duplicate = snapshot.sites[1].inventory[0].clone();
                        snapshot.sites[1].inventory.push(duplicate);
                    }
                    "wrong unit" => snapshot.sites[1].inventory[0].unit_id = "7".repeat(64),
                    "duplicate process" => {
                        let duplicate = snapshot.sites[0].processes[0].clone();
                        snapshot.sites[0].processes.push(duplicate);
                    }
                    _ => unreachable!(),
                }
            }
            let reading = painted_comparison(&mut app, text);
            assert!(
                reading.contains("Selected material unavailable:"),
                "{mismatch}: {reading}"
            );
            assert!(
                !reading.contains("Inventory on hand:"),
                "{mismatch}: {reading}"
            );
            assert!(
                !reading.contains("Production this period:"),
                "{mismatch}: {reading}"
            );
        }
    }

    #[test]
    fn comparison_aggregate_distinguishes_zero_production_from_foundation() {
        for tick in [0, 2] {
            let (mut app, text) = retail_comparison_app(tick);
            if let crate::map_economy_lens::MapLens::Material { kind, .. } =
                &mut app.world_mut().resource_mut::<ObserverUiState>().lens
            {
                *kind = crate::map_economy_lens::MaterialLensKind::ProducedThisPeriod;
            }
            edit_comparison_production(&mut app, |snapshot| {
                snapshot.sites[0].processes[0].produced_batches = (tick > 0).then_some(0);
            });
            let reading = painted_comparison(&mut app, text);
            if tick == 0 {
                assert!(reading.contains(
                    "Selected material unavailable: foundation; no completed production period"
                ));
                assert!(!reading.contains("Production this period:"));
            } else {
                assert!(reading.contains("Production this period: 0 / 0 kg"));
            }
        }
    }

    #[test]
    fn comparison_aggregate_refuses_unmatched_retail_principals_and_missing_periods() {
        for mismatch in [
            "order",
            "county",
            "duplicate",
            "missing",
            "receipt period",
            "retailer",
        ] {
            let (mut app, text) = retail_comparison_app(2);
            {
                let mut browser = app.world_mut().resource_mut::<CampaignBrowserState>();
                let rows = &mut browser
                    .comparison
                    .as_mut()
                    .unwrap()
                    .production
                    .as_mut()
                    .unwrap()
                    .final_demand_accounts;
                match mismatch {
                    "order" => rows[0].orders[0].order_id = "different-order".into(),
                    "county" => rows[0].county_geoid = "26001".into(),
                    "duplicate" => rows.push(rows[0].clone()),
                    "missing" => rows[0].completed = None,
                    "receipt period" => rows[0].completed.as_mut().unwrap().period = 1,
                    "retailer" => rows[0].retailer_site_ids = vec!["withheld".into()],
                    _ => unreachable!(),
                }
            }
            let reading = painted_comparison(&mut app, text);
            assert!(
                reading.contains("Retail totals unavailable:"),
                "{mismatch}: {reading}"
            );
            assert!(
                !reading.contains("Delivered to end buyers this period:"),
                "{mismatch}: {reading}"
            );
            assert!(
                !reading.contains("Unsold retail stock:"),
                "{mismatch}: {reading}"
            );
        }
    }

    #[test]
    fn comparison_aggregate_distinguishes_completed_zero_from_foundation() {
        for tick in [0, 2] {
            let (mut app, text) = retail_comparison_app(tick);
            edit_comparison_production(&mut app, |snapshot| {
                let row = &mut snapshot.final_demand_accounts[0];
                row.fulfilled = 0;
                row.outstanding = 20;
                row.retail_stock_on_hand = 0;
                row.orders[0].fulfilled = 0;
                row.orders[0].outstanding = 20;
                if let Some(completed) = &mut row.completed {
                    completed.newly_fulfilled = 0;
                    completed.closing_fulfilled = 0;
                }
                snapshot.sites[1].inventory[0].quantity = 0;
            });
            let reading = painted_comparison(&mut app, text);
            assert!(reading.contains("Unsold retail stock: 0 / 0 kg"));
            if tick == 0 {
                assert!(reading.contains("Foundation; no completed retail deliveries"));
                assert!(!reading.contains("Delivered to end buyers this period:"));
            } else {
                assert!(reading.contains("Delivered to end buyers this period: 0 / 0 kg"));
                assert!(!reading.contains("Foundation; no completed retail deliveries"));
            }
        }
    }

    #[test]
    fn comparison_renders_signed_staffing_counts_for_the_selected_completed_period() {
        let (mut app, text) = staffing_comparison_app(2);
        let value = painted_comparison(&mut app, text);
        assert!(value.starts_with("Period 2 | full observer\nCurrent 00000000-0000-0000-0000-000000000001\nCompared 00000000-0000-0000-0000-000000000002"));
        assert!(value.contains("Signed difference = current minus compared"));
        assert!(value.contains("Closing employed: 6 / 4 people | difference +2"));
        assert!(value.contains("Closing reserve: 4 / 6 people | difference -2"));
        assert!(value.contains("Hires this period: 2 / 0 people | difference +2"));
        assert!(value.contains("Separations this period: 0 / 1 people | difference -1"));
        assert!(value.contains("separate from observed QCEW jobs and labor-hours"));
        assert!(value.contains("do not record wage payments or class migration"));
    }

    #[test]
    fn comparison_joins_canonical_staffing_identity_independent_of_names_and_row_order() {
        let (mut app, text) = staffing_comparison_app(2);
        let mut extra = app
            .world()
            .resource::<ObserverFrame>()
            .0
            .as_ref()
            .unwrap()
            .production
            .as_ref()
            .unwrap()
            .staffing_accounts[0]
            .clone();
        extra.pool_id = "6".repeat(64);
        extra.subject.local_name = "Second workforce".into();
        app.world_mut()
            .resource_mut::<ObserverFrame>()
            .0
            .as_mut()
            .unwrap()
            .production
            .as_mut()
            .unwrap()
            .staffing_accounts
            .insert(0, extra.clone());
        {
            let mut browser = app.world_mut().resource_mut::<CampaignBrowserState>();
            let compared = browser
                .comparison
                .as_mut()
                .unwrap()
                .production
                .as_mut()
                .unwrap();
            compared.sites[0].name = "A renamed cohort".into();
            compared.staffing_accounts[0].subject.local_name = "A renamed workforce".into();
            compared.staffing_accounts.push(extra);
        }
        let value = painted_comparison(&mut app, text);
        assert!(value.contains("Closing employed: 6 / 4 people | difference +2"));
        assert!(value.contains("Closing employed: 6 / 6 people | difference +0"));
        assert!(!value.contains("workforce unavailable"));
    }

    #[test]
    fn comparison_does_not_join_missing_mismatched_or_duplicate_staffing_rows() {
        for mismatch in ["pool", "site", "unit", "duplicate", "missing"] {
            let (mut app, text) = staffing_comparison_app(2);
            {
                let mut browser = app.world_mut().resource_mut::<CampaignBrowserState>();
                let accounts = &mut browser
                    .comparison
                    .as_mut()
                    .unwrap()
                    .production
                    .as_mut()
                    .unwrap()
                    .staffing_accounts;
                match mismatch {
                    "pool" => accounts[0].pool_id = "9".repeat(64),
                    "site" => accounts[0].site_id = "9".repeat(64),
                    "unit" => accounts[0].unit_id = "9".repeat(64),
                    "duplicate" => accounts.push(accounts[0].clone()),
                    "missing" => accounts.clear(),
                    _ => unreachable!(),
                }
            }
            let value = painted_comparison(&mut app, text);
            assert!(
                value.contains("Modeled workforce unavailable:"),
                "{mismatch}: {value}"
            );
            assert!(!value.contains("Closing employed:"), "{mismatch}: {value}");
            assert!(!value.contains("Hires this period:"), "{mismatch}: {value}");
        }
    }

    #[test]
    fn comparison_distinguishes_foundation_real_zero_and_missing_or_wrong_period_receipts() {
        let (mut app, text) = staffing_comparison_app(0);
        let foundation = painted_comparison(&mut app, text);
        assert!(foundation.contains("Foundation employed: 6 / 4 people | difference +2"));
        assert!(foundation.contains("No completed staffing receipt at foundation"));
        assert!(!foundation.contains("Hires this period:"));
        assert!(!foundation.contains("Closing employed:"));

        let (mut app, text) = staffing_comparison_app(2);
        {
            let mut frame = app.world_mut().resource_mut::<ObserverFrame>();
            let receipt = frame
                .0
                .as_mut()
                .unwrap()
                .production
                .as_mut()
                .unwrap()
                .staffing_accounts[0]
                .completed
                .as_mut()
                .unwrap();
            receipt.hires = 0;
            receipt.opening_employed = 6;
            receipt.opening_reserve = 4;
        }
        {
            let mut browser = app.world_mut().resource_mut::<CampaignBrowserState>();
            let receipt = browser
                .comparison
                .as_mut()
                .unwrap()
                .production
                .as_mut()
                .unwrap()
                .staffing_accounts[0]
                .completed
                .as_mut()
                .unwrap();
            receipt.separations = 0;
            receipt.opening_employed = 4;
            receipt.opening_reserve = 6;
        }
        let zero = painted_comparison(&mut app, text);
        assert!(zero.contains("Hires this period: 0 / 0 people | difference +0"));
        assert!(zero.contains("Separations this period: 0 / 0 people | difference +0"));

        for mismatch in ["missing receipt", "receipt period", "account period"] {
            let (mut app, text) = staffing_comparison_app(2);
            {
                let mut browser = app.world_mut().resource_mut::<CampaignBrowserState>();
                let account = &mut browser
                    .comparison
                    .as_mut()
                    .unwrap()
                    .production
                    .as_mut()
                    .unwrap()
                    .staffing_accounts[0];
                match mismatch {
                    "missing receipt" => account.completed = None,
                    "receipt period" => account.completed.as_mut().unwrap().period = 1,
                    "account period" => account.next_opening_period = 2,
                    _ => unreachable!(),
                }
            }
            let value = painted_comparison(&mut app, text);
            assert!(
                value.contains("COMPARED workforce unavailable:"),
                "{mismatch}: {value}"
            );
            assert!(!value.contains("Hires this period:"), "{mismatch}: {value}");
            assert!(!value.contains("Closing employed:"), "{mismatch}: {value}");
        }
    }

    #[test]
    fn comparison_repaint_removes_staffing_counts_outside_its_campaign_period_or_perspective() {
        for mismatch in [
            "campaign",
            "same campaign",
            "period",
            "perspective",
            "active period",
            "session perspective",
        ] {
            let (mut app, text) = staffing_comparison_app(2);
            assert!(painted_comparison(&mut app, text).contains("Closing employed:"));
            match mismatch {
                "active period" => {
                    app.world_mut()
                        .resource_mut::<ObserverFrame>()
                        .0
                        .as_mut()
                        .unwrap()
                        .resolve_tick = 1;
                }
                "session perspective" => {
                    app.world_mut()
                        .resource_mut::<ObserverSession>()
                        .set_perspective(Perspective::PlayerKnowledge);
                }
                _ => {
                    let mut browser = app.world_mut().resource_mut::<CampaignBrowserState>();
                    if mismatch == "same campaign" {
                        browser.comparison_target =
                            Some(CampaignId::from_uuid(uuid::Uuid::from_u128(1)));
                    }
                    let compared = browser.comparison.as_mut().unwrap();
                    match mismatch {
                        "campaign" => compared.campaign_id = uuid::Uuid::from_u128(3).to_string(),
                        "same campaign" => {
                            compared.campaign_id = uuid::Uuid::from_u128(1).to_string();
                        }
                        "period" => compared.resolve_tick = 1,
                        "perspective" => compared.visibility = ObserverVisibility::KnownPreview,
                        _ => unreachable!(),
                    }
                }
            }
            let value = painted_comparison(&mut app, text);
            assert!(!value.contains("Closing employed:"), "{mismatch}: {value}");
            assert!(!value.contains("Hires this period:"), "{mismatch}: {value}");
        }
    }

    #[test]
    fn comparison_paints_freight_with_output_and_staffing_and_clears_on_scope_change() {
        let (mut app, text) = staffing_comparison_app(1);
        let freight = crate::production_freight::tests::fixture();
        {
            let mut frame = app.world_mut().resource_mut::<ObserverFrame>();
            let production = frame.0.as_mut().unwrap().production.as_mut().unwrap();
            production.routes = freight.routes.clone();
            production.freight_capacity_accounts = freight.freight_capacity_accounts.clone();
            production.sites.extend(freight.sites.clone());
        }
        {
            let mut browser = app.world_mut().resource_mut::<CampaignBrowserState>();
            let production = browser
                .comparison
                .as_mut()
                .unwrap()
                .production
                .as_mut()
                .unwrap();
            production.routes = freight.routes;
            production.freight_capacity_accounts = freight.freight_capacity_accounts;
            production.sites.extend(freight.sites);
        }
        let value = painted_comparison(&mut app, text);
        assert_eq!(value.matches("Designed regional freight pool").count(), 1);
        assert!(value.contains("Dispatched: 120 / 120 kg"));
        assert!(value.contains("Closing employed:"));
        assert!(value.contains("Next-period capacity:"));
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(Perspective::PlayerKnowledge);
        let value = painted_comparison(&mut app, text);
        assert!(!value.contains("Dispatched:"));
        assert!(!value.contains("Designed regional freight pool"));
    }

    #[test]
    fn staffing_difference_preserves_the_full_unsigned_count_range() {
        let mut output = String::new();
        staffing_difference(&mut output, "Employed", u64::MAX, 0);
        staffing_difference(&mut output, "Reserve", 0, u64::MAX);
        assert!(output.contains("difference +18446744073709551615"));
        assert!(output.contains("difference -18446744073709551615"));
    }

    fn catalog_handoff_app(with_pipe: bool) -> (App, CampaignId) {
        let campaign = CampaignId::from_uuid(uuid::Uuid::from_u128(1));
        let selected = CampaignId::from_uuid(uuid::Uuid::from_u128(2));
        let mut session = ObserverSession::new(campaign);
        session.fail("Campaign admission refused".into());
        session.connected_fixture();
        let mut app = App::new();
        app.insert_resource(session)
            .insert_resource(CampaignBrowserState {
                catalog: vec![CampaignSummary {
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
    fn catalog_after_failed_admission_queues_open_without_saving_or_exiting() {
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
        assert!(app.world().resource::<Messages<AppExit>>().is_empty());
        let request = app
            .world_mut()
            .resource_mut::<ObserverSession>()
            .pending_switch_request()
            .unwrap();
        assert!(
            matches!(request, babylon_persistence::runtime_session::RuntimeSessionRequest::Switch {
            target: babylon_persistence::runtime_session::RuntimeSessionTarget::Open { campaign_id }, ..
        } if campaign_id == selected.as_uuid().to_string())
        );
        assert!(
            !preference_path().unwrap().exists(),
            "An unadmitted target became the saved continuation"
        );
    }

    #[test]
    fn open_before_ready_keeps_transport_and_failed_ui_recoverable() {
        let environment = crate::test_support::EnvVarGuard::lock("XDG_STATE_HOME");
        let directory = std::env::temp_dir().join(format!(
            "babylon-connecting-handoff-{}",
            uuid::Uuid::new_v4()
        ));
        environment.set(directory.to_str().unwrap());
        let (mut catalog, _) = catalog_handoff_app(false);
        let browser = catalog
            .world_mut()
            .remove_resource::<CampaignBrowserState>()
            .unwrap();
        let (mut app, requests, responses) = crate::observer_io::tests::quit_app();
        let campaign = app.world().resource::<ObserverSession>().campaign;
        app.insert_resource(ObserverSession::new(campaign))
            .insert_resource(browser)
            .add_message::<CampaignBrowserCommand>()
            .add_systems(PreUpdate, commands);
        app.world_mut()
            .resource_mut::<Messages<CampaignBrowserCommand>>()
            .write(CampaignBrowserCommand::Open);
        app.update();
        let exited_before_ready = !app.world().resource::<Messages<AppExit>>().is_empty();
        assert!(app.world().contains_resource::<RuntimePipe>());

        let admitted_target =
            crate::observer_io::tests::refuse_initial_switch(&mut app, &requests, &responses);
        if directory.exists() {
            fs::remove_dir_all(directory).unwrap();
        }

        assert!(
            !exited_before_ready,
            "Open closed the runtime response pipe before Ready"
        );
        assert!(
            app.world().resource::<Messages<AppExit>>().is_empty(),
            "Startup failure closed the recovery UI"
        );
        assert!(app.world().contains_resource::<RuntimePipe>());
        assert!(app.world().resource::<ObserverUiState>().menu_open);
        let state = app.world().resource::<ObserverSession>();
        assert_eq!(state.phase, crate::observer::SessionPhase::Failed);
        assert_eq!(state.campaign, admitted_target);
        assert_eq!(state.durable_tick, 0);
        assert_eq!(
            state.error.as_deref(),
            Some(
                babylon_persistence::runtime_session::RuntimeSessionErrorCode::StorageRefused
                    .to_string()
                    .as_str()
            )
        );
        assert_eq!(
            availability(ObserverCommand::ReopenCampaign, state),
            ControlAvailability::Enabled
        );
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
    fn catalog_handoff_pending_commit_queues_without_abandoning_ack_or_saving_target() {
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
                // The rendered control was enabled before this period began.
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
                "Opening the selected campaign..."
            );
            assert!(app
                .world_mut()
                .resource_mut::<ObserverSession>()
                .pending_switch_request()
                .is_none());
            assert!(app
                .world_mut()
                .resource_mut::<ObserverSession>()
                .acknowledge(request, 4, None));
            app.world_mut()
                .resource_mut::<Messages<CampaignBrowserCommand>>()
                .write(CampaignBrowserCommand::Open);
            app.update();
            assert!(app.world().resource::<Messages<AppExit>>().is_empty());
            let request = app
                .world_mut()
                .resource_mut::<ObserverSession>()
                .pending_switch_request()
                .unwrap();
            assert!(
                matches!(request, babylon_persistence::runtime_session::RuntimeSessionRequest::Switch {
                target: babylon_persistence::runtime_session::RuntimeSessionTarget::Open { campaign_id }, ..
            } if campaign_id == selected.as_uuid().to_string())
            );
            assert_eq!(
                fs::read_to_string(&path).unwrap(),
                format!("{}\n", active.as_uuid())
            );
        }
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn catalog_open_remains_queueable_during_pending_commit_without_pointer_motion() {
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
            app.world()
                .get::<ObserverFocusTarget>(button)
                .unwrap()
                .available
        );
        assert_eq!(
            *app.world().get::<BorderColor>(button).unwrap(),
            BorderColor::all(theme::YELLOW)
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
                .map(|id| CampaignSummary {
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
    fn comparison_scope_rejects_changed_perspective_period_campaign_and_generation() {
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
