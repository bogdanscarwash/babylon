//! Observer navigation, scoped command admission, and production selection.

use babylon_persistence::{
    production_observation::ProductionSite, production_observation::ProductionSnapshot,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::atlas::CountyAtlas;
use crate::map::SelectedCounty;
use crate::observer::ObserverSession;
use crate::observer_focus::ObserverFocusWorld;
use crate::observer_ui::{ObserverFrame, ObserverUiState};
use crate::production_brief::{dependency_sites, opening_site};
use crate::production_freight::competitor_sites;

use super::{
    PrimaryView, ProductionCommand, ProductionNavigation, ProductionObservation, ProductionPage,
    ProductionUi,
};

impl ProductionNavigation {
    pub(super) fn open_county(
        &mut self,
        county_geoid: &str,
        snapshot: Option<&ProductionSnapshot>,
    ) {
        self.county_geoid = Some(county_geoid.to_owned());
        let resume = snapshot.is_some_and(|snapshot| {
            snapshot.sites.iter().any(|site| {
                self.selected_site.as_ref() == Some(&site.id) && site.county_geoid == county_geoid
            })
        });
        if !resume {
            self.county_open = true;
            self.details_open = false;
            self.cohort_page = 0;
            self.selected_site = None;
            self.history.clear();
        }
    }

    pub(super) fn select_site(&mut self, id: &str) {
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
        site: &'a ProductionSite,
    ) -> Option<&'a babylon_persistence::production_observation::ProductionProcess> {
        site.processes
            .iter()
            .find(|process| self.selected_process.as_ref() == Some(&process.id))
            .or_else(|| site.processes.iter().min_by(|a, b| a.id.cmp(&b.id)))
    }
}

pub(super) struct ProductionControlAvailability {
    pub(super) scene: bool,
    pub(super) readings: bool,
    pub(super) previous_index: Option<usize>,
    pub(super) county_back: bool,
}

impl ProductionControlAvailability {
    pub(super) fn for_snapshot(
        snapshot: Option<&ProductionSnapshot>,
        navigation: &ProductionNavigation,
    ) -> Self {
        Self {
            scene: snapshot.is_some_and(|snapshot| !snapshot.sites.is_empty()),
            readings: !navigation.county_open
                && snapshot.is_some_and(|snapshot| !snapshot.sites.is_empty()),
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

    pub(super) fn display(&self, command: &ProductionCommand) -> Option<Display> {
        let available = match command {
            ProductionCommand::Back => self.previous_index.is_some() || self.county_back,
            ProductionCommand::Details | ProductionCommand::Reading(_) => self.readings,
            ProductionCommand::Flat => self.scene,
            _ => return None,
        };
        Some(if available {
            Display::Flex
        } else {
            Display::None
        })
    }

    pub(super) fn refusal(
        &self,
        command: &ProductionCommand,
        snapshot: Option<&ProductionSnapshot>,
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
            ProductionCommand::Details | ProductionCommand::Reading(_)
                if navigation.county_open && !navigation.details_open =>
            {
                Some("Choose a county cohort before opening its readings.")
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
            | ProductionCommand::Focus { site_id, context }
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
    snapshot: Option<&ProductionSnapshot>,
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
pub(super) struct ProductionLocation<'w> {
    atlas: Res<'w, CountyAtlas>,
    selected: ResMut<'w, SelectedCounty>,
}

pub(super) fn navigate(
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
            ProductionCommand::Select { site_id: id, .. }
            | ProductionCommand::Focus { site_id: id, .. } => {
                navigation.select_site(id);
                sync_county = true;
                *view = if matches!(event, ProductionCommand::Focus { .. }) {
                    PrimaryView::Map
                } else {
                    PrimaryView::Production
                };
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

pub(super) fn sync_selected_county(
    navigation: &ProductionNavigation,
    snapshot: Option<&ProductionSnapshot>,
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

pub(super) fn sync_world_county(
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

pub(super) fn invalidate_navigation(
    state: Res<ObserverSession>,
    frame: Res<ObserverFrame>,
    mut navigation: ResMut<ProductionNavigation>,
    mut scope: Local<
        Option<(
            babylon_persistence::identity::CampaignId,
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
pub(super) fn focus_opening(
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
