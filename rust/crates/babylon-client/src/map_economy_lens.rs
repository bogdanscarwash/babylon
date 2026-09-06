//! Exact county readings from one already scoped, committed observation.
//! Goods with the same unit remain separate; this module performs no mechanics.

use std::collections::BTreeMap;

use babylon_persistence::{
    ObserverCountyEconomyV1, ObserverEconomySnapshotV1, ProductionSnapshotV1,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EconomyMetric {
    #[default]
    Employment,
    Payroll,
    WeeklyWage,
    Establishments,
}

impl EconomyMetric {
    pub const ALL: [Self; 4] = [
        Self::Employment,
        Self::Payroll,
        Self::WeeklyWage,
        Self::Establishments,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Employment => "Employment",
            Self::Payroll => "Annual payroll",
            Self::WeeklyWage => "Mean weekly wage",
            Self::Establishments => "Establishments",
        }
    }

    #[must_use]
    pub const fn unit(self) -> &'static str {
        match self {
            Self::Employment => "annual average jobs",
            Self::Payroll => "USD / year",
            Self::WeeklyWage => "USD / employee / week",
            Self::Establishments => "annual average establishments",
        }
    }

    #[must_use]
    pub const fn value(self, county: &ObserverCountyEconomyV1) -> Option<u64> {
        match self {
            Self::Employment => county.annual_avg_emplvl,
            Self::Payroll => county.total_annual_wages,
            Self::WeeklyWage => county.annual_avg_wkly_wage,
            Self::Establishments => county.annual_avg_estabs_count,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialLensKind {
    ProducedThisWeek,
    OnHand,
    InboundInTransit,
}

impl MaterialLensKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ProducedThisWeek => "Production this week",
            Self::OnHand => "Inventory on hand",
            Self::InboundInTransit => "Inbound in transit",
        }
    }
}

/// Material and unit identities are authoritative; neither display string is a key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MaterialGoodKey {
    pub good_id: String,
    pub unit_id: String,
}

impl MaterialGoodKey {
    fn matches(&self, good_id: &str, unit_id: &str) -> bool {
        self.good_id == good_id && self.unit_id == unit_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialGoodChoice {
    pub key: MaterialGoodKey,
    pub label: String,
    pub unit: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum MapLens {
    /// Schematic relationships, without a numeric county encoding.
    #[default]
    Relationships,
    Qcew(EconomyMetric),
    Material {
        kind: MaterialLensKind,
        good: Option<MaterialGoodKey>,
    },
}

impl MapLens {
    /// The caller supplies only `ObserverFrame::for_session`; no retained label
    /// or raw identity can enter logs after a capability or context invalidation.
    #[must_use]
    pub fn label_for_log(&self, snapshot: Option<&ObserverEconomySnapshotV1>) -> String {
        match self {
            Self::Relationships => "Supply relationships".to_owned(),
            Self::Qcew(metric) => metric.label().to_owned(),
            Self::Material { kind, good } => snapshot
                .and_then(|snapshot| material_choices(snapshot, *kind).ok())
                .and_then(|choices| {
                    choices
                        .into_iter()
                        .find(|row| Some(&row.key) == good.as_ref())
                })
                .map_or_else(
                    || format!("{} / unavailable", kind.label()),
                    |row| format!("{} / {} / {}", kind.label(), row.label, row.unit),
                ),
        }
    }

    /// Preserve the selected identity across weeks. A different campaign or
    /// capability clears it, including while its new observation is pending.
    pub fn reconcile(&mut self, snapshot: Option<&ObserverEconomySnapshotV1>, scope_changed: bool) {
        let Self::Material { kind, good } = self else {
            return;
        };
        if scope_changed {
            *good = None;
        }
        let Some(snapshot) = snapshot else {
            return;
        };
        let choices = material_choices(snapshot, *kind).unwrap_or_default();
        if !choices.iter().any(|row| Some(&row.key) == good.as_ref()) {
            *good = choices.first().map(|row| row.key.clone());
        }
    }

    pub fn cycle_good(&mut self, snapshot: Option<&ObserverEconomySnapshotV1>, backwards: bool) {
        let Self::Material { kind, good } = self else {
            return;
        };
        let Some(choices) = snapshot.and_then(|snapshot| material_choices(snapshot, *kind).ok())
        else {
            return;
        };
        if choices.is_empty() {
            *good = None;
            return;
        }
        let index = choices
            .iter()
            .position(|row| Some(&row.key) == good.as_ref());
        let next = index.map_or(0, |index| {
            if backwards {
                (index + choices.len() - 1) % choices.len()
            } else {
                (index + 1) % choices.len()
            }
        });
        *good = Some(choices[next].key.clone());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LensUnavailable {
    Loading,
    NotKnown,
    CapabilityUnavailable,
    NoGoodSelected,
    NotModeled,
    NoProductionWeek,
    InvalidObservation,
    Arithmetic,
}

impl LensUnavailable {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Loading => "Loading committed observation",
            Self::NotKnown => "Not known in this observation",
            Self::CapabilityUnavailable => "Material evidence unavailable in this perspective",
            Self::NoGoodSelected => "Choose an available good",
            Self::NotModeled => "Not modeled for this good and lens",
            Self::NoProductionWeek => "Foundation: no committed production week",
            Self::InvalidObservation => "Observation refused: inconsistent material identity",
            Self::Arithmetic => "Observation refused: quantity overflow",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountyLensReading {
    Available(u64),
    Unavailable(LensUnavailable),
}

impl CountyLensReading {
    #[must_use]
    pub const fn value(self) -> Option<u64> {
        match self {
            Self::Available(value) => Some(value),
            Self::Unavailable(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapLensProjection {
    pub label: String,
    pub good_label: Option<String>,
    pub unit: String,
    pub evidence: &'static str,
    pub counties: BTreeMap<String, CountyLensReading>,
    pub unavailable: LensUnavailable,
}

impl MapLensProjection {
    #[must_use]
    pub fn county(&self, geoid: &str) -> CountyLensReading {
        self.counties
            .get(geoid)
            .copied()
            .unwrap_or(CountyLensReading::Unavailable(self.unavailable))
    }

    #[must_use]
    pub fn maximum(&self) -> Option<u64> {
        self.counties.values().filter_map(|row| row.value()).max()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapLensError {
    Identity,
    Arithmetic,
}

fn add_choice(
    choices: &mut BTreeMap<MaterialGoodKey, MaterialGoodChoice>,
    good_id: &str,
    unit_id: &str,
    label: &str,
    unit: &str,
) -> Result<(), MapLensError> {
    // Presentation refuses missing identities rather than falling back to labels.
    if [good_id, unit_id]
        .iter()
        .any(|id| id.len() != 64 || !id.bytes().all(|byte| byte.is_ascii_hexdigit()))
    {
        return Err(MapLensError::Identity);
    }
    let key = MaterialGoodKey {
        good_id: good_id.to_owned(),
        unit_id: unit_id.to_owned(),
    };
    let row = MaterialGoodChoice {
        key: key.clone(),
        label: label.to_owned(),
        unit: unit.to_owned(),
    };
    if choices.get(&key).is_some_and(|previous| previous != &row) {
        return Err(MapLensError::Identity);
    }
    choices.insert(key, row);
    Ok(())
}

/// Goods admitted by this observation and lens, including exact zero accounts.
///
/// # Errors
/// Refuses missing material identities or conflicting labels for one identity.
pub fn material_choices(
    snapshot: &ObserverEconomySnapshotV1,
    kind: MaterialLensKind,
) -> Result<Vec<MaterialGoodChoice>, MapLensError> {
    // The role boundary already withholds this projection, and checking the
    // capability here also prevents accidental reuse of a mismatched fixture.
    if snapshot.visibility != babylon_persistence::ObserverVisibilityV1::FullObserver {
        return Ok(Vec::new());
    }
    let Some(production) = &snapshot.production else {
        return Ok(Vec::new());
    };
    let mut choices = BTreeMap::new();
    match kind {
        MaterialLensKind::ProducedThisWeek => {
            for site in &production.sites {
                add_choice(
                    &mut choices,
                    &site.output_good_id,
                    &site.output_unit_id,
                    &site.output_good,
                    &site.output_unit,
                )?;
            }
        }
        MaterialLensKind::OnHand => {
            for stock in production.sites.iter().flat_map(|site| &site.inventory) {
                add_choice(
                    &mut choices,
                    &stock.good_id,
                    &stock.unit_id,
                    &stock.good,
                    &stock.unit,
                )?;
            }
        }
        MaterialLensKind::InboundInTransit => {
            for route in &production.routes {
                add_choice(
                    &mut choices,
                    &route.good_id,
                    &route.unit_id,
                    &route.good,
                    &route.unit,
                )?;
            }
        }
    }
    let mut choices: Vec<_> = choices.into_values().collect();
    choices.sort_by(|a, b| (&a.label, &a.unit, &a.key).cmp(&(&b.label, &b.unit, &b.key)));
    Ok(choices)
}

/// Every consumer uses this same reading, including refused or absent data.
#[must_use]
pub fn project_map_lens(
    snapshot: Option<&ObserverEconomySnapshotV1>,
    lens: &MapLens,
) -> MapLensProjection {
    let mut result = MapLensProjection {
        label: lens.label_for_log(snapshot),
        good_label: None,
        unit: String::new(),
        evidence: "DESIGNED | county industry cohorts; no factory locations",
        counties: BTreeMap::new(),
        unavailable: LensUnavailable::Loading,
    };
    if let MapLens::Qcew(metric) = lens {
        metric.unit().clone_into(&mut result.unit);
        result.evidence = "OBSERVED | BLS QCEW | 2024 annual baseline";
    }
    if matches!(lens, MapLens::Relationships) {
        result.evidence = "SCHEMATIC | county aggregates; no physical route geometry";
        result.unavailable = if snapshot.is_some() {
            LensUnavailable::NotModeled
        } else {
            LensUnavailable::Loading
        };
        return result;
    }
    let Some(snapshot) = snapshot else {
        return result;
    };
    if let MapLens::Qcew(metric) = lens {
        result.unavailable = LensUnavailable::NotKnown;
        result.counties = snapshot
            .counties
            .iter()
            .map(|county| {
                (
                    county.county_geoid.clone(),
                    metric.value(county).map_or(
                        CountyLensReading::Unavailable(LensUnavailable::NotKnown),
                        CountyLensReading::Available,
                    ),
                )
            })
            .collect();
        return result;
    }
    let MapLens::Material { kind, good } = lens else {
        return result;
    };
    let Some(production) = snapshot
        .production
        .as_ref()
        .filter(|_| snapshot.visibility == babylon_persistence::ObserverVisibilityV1::FullObserver)
    else {
        result.unavailable = LensUnavailable::CapabilityUnavailable;
        return result;
    };
    let Ok(choices) = material_choices(snapshot, *kind) else {
        result.unavailable = LensUnavailable::InvalidObservation;
        return result;
    };
    let Some(choice) = choices
        .iter()
        .find(|choice| Some(&choice.key) == good.as_ref())
    else {
        result.unavailable = LensUnavailable::NoGoodSelected;
        return result;
    };
    result.label = format!("{} / {}", kind.label(), choice.label);
    result.unit.clone_from(&choice.unit);
    result.good_label = Some(choice.label.clone());
    result.unavailable = LensUnavailable::NotModeled;
    match project_material_counties(production, *kind, &choice.key) {
        Ok(counties) => result.counties = counties,
        Err(error) => {
            result.unavailable = match error {
                MapLensError::Identity => LensUnavailable::InvalidObservation,
                MapLensError::Arithmetic => LensUnavailable::Arithmetic,
            }
        }
    }
    result
}

fn add_quantity(
    counties: &mut BTreeMap<String, CountyLensReading>,
    county: &str,
    quantity: Option<u64>,
) -> Result<(), MapLensError> {
    let row = counties
        .entry(county.to_owned())
        .or_insert(CountyLensReading::Available(0));
    *row = match (*row, quantity) {
        (CountyLensReading::Available(before), Some(quantity)) => CountyLensReading::Available(
            before
                .checked_add(quantity)
                .ok_or(MapLensError::Arithmetic)?,
        ),
        _ => CountyLensReading::Unavailable(LensUnavailable::NoProductionWeek),
    };
    Ok(())
}

fn project_material_counties(
    production: &ProductionSnapshotV1,
    kind: MaterialLensKind,
    good: &MaterialGoodKey,
) -> Result<BTreeMap<String, CountyLensReading>, MapLensError> {
    let mut counties = BTreeMap::new();
    match kind {
        MaterialLensKind::ProducedThisWeek => {
            for site in &production.sites {
                if good.matches(&site.output_good_id, &site.output_unit_id) {
                    let produced = site
                        .produced_batches
                        .map(|batches| {
                            batches
                                .checked_mul(site.output_per_batch)
                                .ok_or(MapLensError::Arithmetic)
                        })
                        .transpose()?;
                    add_quantity(&mut counties, &site.county_geoid, produced)?;
                }
            }
        }
        MaterialLensKind::OnHand => {
            for site in &production.sites {
                for stock in &site.inventory {
                    if good.matches(&stock.good_id, &stock.unit_id) {
                        add_quantity(&mut counties, &site.county_geoid, Some(stock.quantity))?;
                    }
                }
            }
        }
        MaterialLensKind::InboundInTransit => {
            let routes: BTreeMap<_, _> = production
                .routes
                .iter()
                .filter(|route| good.matches(&route.good_id, &route.unit_id))
                .map(|route| (route.id.as_str(), route))
                .collect();
            let sites: BTreeMap<_, _> = production
                .sites
                .iter()
                .map(|site| (site.id.as_str(), site))
                .collect();
            for route in routes.values() {
                let buyer = sites
                    .get(route.buyer_site_id.as_str())
                    .ok_or(MapLensError::Identity)?;
                add_quantity(&mut counties, &buyer.county_geoid, Some(0))?;
            }
            for lot in &production.freight {
                if !good.matches(&lot.good_id, &lot.unit_id) {
                    continue;
                }
                let route = routes
                    .get(lot.route_id.as_str())
                    .ok_or(MapLensError::Identity)?;
                if route.buyer_site_id != lot.destination_site_id
                    || route.supplier_site_id != lot.source_site_id
                {
                    return Err(MapLensError::Identity);
                }
                let buyer = sites
                    .get(lot.destination_site_id.as_str())
                    .ok_or(MapLensError::Identity)?;
                add_quantity(&mut counties, &buyer.county_geoid, Some(lot.quantity))?;
            }
        }
    }
    Ok(counties)
}

#[cfg(test)]
mod tests {
    #[test]
    fn relationships_are_the_default_without_fabricating_numeric_county_readings() {
        let lens = super::MapLens::default();
        assert_eq!(lens, super::MapLens::Relationships);
        let observation = snapshot();
        for observation in [None, Some(&observation)] {
            let projection = super::project_map_lens(observation, &lens);
            assert!(projection.counties.is_empty());
            assert_eq!(projection.maximum(), None);
            assert!(projection.unit.is_empty());
        }
        assert_eq!(lens.label_for_log(None), "Supply relationships");
    }

    use super::*;
    use babylon_persistence::{
        ObserverVisibilityV1, ProductionFreightV1, ProductionRouteV1, ProductionSiteV1,
        ProductionStockV1,
    };

    fn key(letter: char) -> MaterialGoodKey {
        MaterialGoodKey {
            good_id: letter.to_string().repeat(64),
            unit_id: "c".repeat(64),
        }
    }

    fn site(id: &str, county: &str, good: char, quantity: u64) -> ProductionSiteV1 {
        ProductionSiteV1 {
            id: id.into(),
            county_geoid: county.into(),
            name: id.into(),
            industry_code: "331".into(),
            observed_employment: None,
            output_good_id: key(good).good_id,
            output_unit_id: key(good).unit_id,
            output_good: format!("Good {good}"),
            output_unit: "kg".into(),
            output_per_batch: 5,
            available_batches: 10,
            planned_batches: Some(2),
            produced_batches: Some(2),
            inventory: vec![ProductionStockV1 {
                good_id: key(good).good_id,
                unit_id: key(good).unit_id,
                good: format!("Good {good}"),
                unit: "kg".into(),
                quantity,
            }],
            inputs: vec![],
            labor: vec![],
        }
    }

    fn snapshot() -> ObserverEconomySnapshotV1 {
        ObserverEconomySnapshotV1 {
            campaign_id: "4ae8c232-9b98-4a24-8a89-23821373da99".into(),
            resolve_tick: 1,
            foundation_digest: "f".repeat(64),
            nominal_world_hash: Some("d".repeat(64)),
            tick_content_hash: Some("e".repeat(64)),
            envelope_digest: Some("b".repeat(64)),
            visibility: ObserverVisibilityV1::FullObserver,
            counties: vec![],
            production: Some(ProductionSnapshotV1 {
                material_balance: None,
                labor_accounts: Vec::new(),
                scenario_label: "Designed lens fixture".into(),
                horizon_week: 16,
                sites: vec![
                    site("source", "26163", 'a', 0),
                    site("other", "26163", 'b', 500),
                    site("buyer", "26099", 'b', 0),
                ],
                routes: vec![ProductionRouteV1 {
                    id: "route".into(),
                    supplier_site_id: "source".into(),
                    buyer_site_id: "buyer".into(),
                    good_id: key('a').good_id,
                    unit_id: key('a').unit_id,
                    good: "Good a".into(),
                    unit: "kg".into(),
                    travel_weeks: 3,
                    ordered: 100,
                    shipped: 90,
                    delivered: 60,
                    lost: 0,
                    realized: 60,
                    backlog: 10,
                }],
                freight: vec![ProductionFreightV1 {
                    id: "lot".into(),
                    route_id: "route".into(),
                    source_site_id: "source".into(),
                    destination_site_id: "buyer".into(),
                    good_id: key('a').good_id,
                    unit_id: key('a').unit_id,
                    good: "Good a".into(),
                    unit: "kg".into(),
                    quantity: 30,
                    dispatch_week: 1,
                    arrival_week: 4,
                }],
                events: vec![],
                observed_contexts: Vec::new(),
                process_attributions: Vec::new(),
                provenance: vec![],
            }),
        }
    }

    fn lens(kind: MaterialLensKind) -> MapLens {
        MapLens::Material {
            kind,
            good: Some(key('a')),
        }
    }

    #[test]
    fn same_unit_and_county_never_merge_different_goods() {
        let snapshot = snapshot();
        let produced = project_map_lens(Some(&snapshot), &lens(MaterialLensKind::ProducedThisWeek));
        assert_eq!(produced.county("26163"), CountyLensReading::Available(10));
        assert_eq!(
            produced.county("26099"),
            CountyLensReading::Unavailable(LensUnavailable::NotModeled)
        );
        let stock = project_map_lens(Some(&snapshot), &lens(MaterialLensKind::OnHand));
        assert_eq!(stock.county("26163"), CountyLensReading::Available(0));
        assert_eq!(stock.maximum(), Some(0));
        assert_eq!(
            stock.county("26001"),
            CountyLensReading::Unavailable(LensUnavailable::NotModeled)
        );
    }

    #[test]
    fn foundation_unavailable_and_committed_zero_are_distinct() {
        let mut snapshot = snapshot();
        let site = &mut snapshot.production.as_mut().unwrap().sites[0];
        site.produced_batches = None;
        snapshot.resolve_tick = 0;
        let projected =
            project_map_lens(Some(&snapshot), &lens(MaterialLensKind::ProducedThisWeek));
        assert_eq!(
            projected.county("26163"),
            CountyLensReading::Unavailable(LensUnavailable::NoProductionWeek)
        );
        snapshot.production.as_mut().unwrap().sites[0].produced_batches = Some(0);
        snapshot.resolve_tick = 1;
        assert_eq!(
            project_map_lens(Some(&snapshot), &lens(MaterialLensKind::ProducedThisWeek))
                .county("26163"),
            CountyLensReading::Available(0)
        );
    }

    #[test]
    fn inbound_counts_actual_lots_once_and_empty_routes_remain_zero() {
        let mut snapshot = snapshot();
        let selection = lens(MaterialLensKind::InboundInTransit);
        let projected = project_map_lens(Some(&snapshot), &selection);
        assert_eq!(projected.county("26099"), CountyLensReading::Available(30));
        assert_eq!(
            projected.county("26163"),
            CountyLensReading::Unavailable(LensUnavailable::NotModeled)
        );
        snapshot.production.as_mut().unwrap().freight.clear();
        let projected = project_map_lens(Some(&snapshot), &selection);
        assert_eq!(projected.county("26099"), CountyLensReading::Available(0));
        assert_eq!(
            material_choices(&snapshot, MaterialLensKind::InboundInTransit)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn multiplication_and_county_sum_overflow_refuse_the_whole_reading() {
        let mut snapshot = snapshot();
        snapshot.production.as_mut().unwrap().sites[0].output_per_batch = u64::MAX;
        assert_eq!(
            project_map_lens(Some(&snapshot), &lens(MaterialLensKind::ProducedThisWeek))
                .unavailable,
            LensUnavailable::Arithmetic
        );
        let mut snapshot = self::snapshot();
        snapshot
            .production
            .as_mut()
            .unwrap()
            .sites
            .push(site("overflow", "26163", 'a', u64::MAX));
        snapshot.production.as_mut().unwrap().sites[0].inventory[0].quantity = 1;
        let result = project_map_lens(Some(&snapshot), &lens(MaterialLensKind::OnHand));
        assert_eq!(result.unavailable, LensUnavailable::Arithmetic);
        assert!(result.counties.is_empty());
    }

    #[test]
    fn identities_are_required_even_when_labels_match() {
        let mut snapshot = snapshot();
        snapshot.production.as_mut().unwrap().sites[0]
            .output_good_id
            .clear();
        let result = project_map_lens(Some(&snapshot), &lens(MaterialLensKind::ProducedThisWeek));
        assert_eq!(result.unavailable, LensUnavailable::InvalidObservation);
        assert!(result.counties.is_empty());
    }

    #[test]
    fn dangling_freight_destination_is_refused() {
        let mut snapshot = snapshot();
        snapshot.production.as_mut().unwrap().freight[0].destination_site_id = "missing".into();
        assert_eq!(
            project_map_lens(Some(&snapshot), &lens(MaterialLensKind::InboundInTransit))
                .unavailable,
            LensUnavailable::InvalidObservation
        );
    }

    #[test]
    fn historical_reading_never_uses_a_later_snapshot() {
        let historical = snapshot();
        let mut live = historical.clone();
        live.resolve_tick = 2;
        live.production.as_mut().unwrap().sites[0].produced_batches = Some(7);
        let selection = lens(MaterialLensKind::ProducedThisWeek);
        assert_eq!(
            project_map_lens(Some(&historical), &selection).county("26163"),
            CountyLensReading::Available(10)
        );
        assert_eq!(
            project_map_lens(Some(&live), &selection).county("26163"),
            CountyLensReading::Available(35)
        );
    }

    #[test]
    fn capability_change_clears_choices_and_never_logs_stale_identity() {
        let mut snapshot = snapshot();
        let mut selection = lens(MaterialLensKind::ProducedThisWeek);
        assert!(selection.label_for_log(Some(&snapshot)).contains("Good a"));
        assert!(!selection.label_for_log(None).contains("Good a"));
        selection.reconcile(None, true);
        assert_eq!(
            selection,
            MapLens::Material {
                kind: MaterialLensKind::ProducedThisWeek,
                good: None
            }
        );
        // Even an accidentally attached full payload cannot open preview choices.
        snapshot.visibility = ObserverVisibilityV1::KnownPreview;
        selection.reconcile(Some(&snapshot), false);
        assert!(
            material_choices(&snapshot, MaterialLensKind::ProducedThisWeek)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            project_map_lens(Some(&snapshot), &selection).unavailable,
            LensUnavailable::CapabilityUnavailable
        );
        assert_eq!(
            selection.label_for_log(Some(&snapshot)),
            "Production this week / unavailable"
        );
    }

    #[test]
    fn selected_good_survives_same_scope_history_loading_and_cycles_by_identity() {
        let snapshot = snapshot();
        let mut selection = lens(MaterialLensKind::ProducedThisWeek);
        selection.cycle_good(Some(&snapshot), false);
        assert_eq!(
            selection,
            MapLens::Material {
                kind: MaterialLensKind::ProducedThisWeek,
                good: Some(key('b'))
            }
        );
        selection.reconcile(None, false);
        selection.reconcile(Some(&snapshot), false);
        assert!(selection.label_for_log(Some(&snapshot)).contains("Good b"));
        selection.cycle_good(Some(&snapshot), true);
        assert_eq!(selection, lens(MaterialLensKind::ProducedThisWeek));
    }
}
