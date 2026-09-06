//! Bounded Designed physical content with separate observed county-industry evidence.
//!
//! The authoritative session owns advancement, publication and the finite horizon.
//! This module constructs an exact initial state; it does not run mechanics.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use babylon_bsl::causal_contract::EvidenceClass;
use babylon_kernel::sha256_of;
use babylon_material_circuit::{
    decode_material_circuit_state_v2, encode_material_circuit_state_v2, BacklogRowV1,
    CapacityRowV1, CorridorCapacityV2, CorridorIdV2, GoodIdV1, InputOutputCoefficientV1,
    InventoryRowV1, LaborCapacityRowV1, LaborCoefficientV1, LogisticsNodeIdV2,
    MaterialCircuitErrorV2, MaterialCircuitStateV2, OrderAccessModeV1, OrderIdV1, OrderRowV2,
    ProcessIdV1, ProcessOutputV1, ProductionCommitmentV1, RouteIdV2, RouteLegV2, SiteIdV1,
    SiteLogisticsNodeV2, SupplierRouteV2, UnitIdV1,
};
use serde::Deserialize;

/// Exact observed five-row public industry source artifact.
pub const MICHIGAN_INDUSTRY_BASELINE_SHA256_V1: &str =
    "eb486d7e11b8b63fc58c53ab918eff84b341b293a66faf422ddb9304fb2b553e";
/// Exact Designed content artifact, shared by both delay presets.
pub const MICHIGAN_MATERIAL_SCENARIO_SHA256_V1: &str =
    "5e09fa210c34cd1033d35bf9cc626bd06b3b536e016b005ed53e27421ff9b5ca";
const INDUSTRY_BYTES: &[u8] =
    include_bytes!("../../../../contracts/fixtures/michigan_industry_baseline_v1.json");
const SCENARIO_BYTES: &[u8] =
    include_bytes!("../../../../contracts/fixtures/michigan_material_scenario_v1.json");
const ID_DOMAIN: &str = "babylon.michigan-material.v1";
const SOURCE_URL: &str = "https://data.bls.gov/cew/data/files/2024/csv/2024_annual_by_area.zip";

/// Separately committed comparisons; no player intervention is implied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganDeliveryPresetV1 {
    Standard,
    Delayed,
}
impl MichiganDeliveryPresetV1 {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Standard => "michigan-material-standard-v1",
            Self::Delayed => "michigan-material-delayed-v1",
        }
    }
    #[must_use]
    pub const fn horizon_ticks(self) -> u64 {
        16
    }
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "michigan-material-standard-v1" => Some(Self::Standard),
            "michigan-material-delayed-v1" => Some(Self::Delayed),
            _ => None,
        }
    }
}

/// Closed construction refusals; source suppression never becomes a numeric zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganMaterialErrorV1 {
    ArtifactDigest,
    ArtifactDecode,
    ArtifactShape,
    SourceSuppressed,
    SourceValue,
    ContentReference,
    ContentValue,
    Circuit(MaterialCircuitErrorV2),
}
impl std::fmt::Display for MichiganMaterialErrorV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Michigan material content refused: {self:?}")
    }
}
impl std::error::Error for MichiganMaterialErrorV1 {}

fn identity(kind: &str, key: &str) -> [u8; 32] {
    sha256_of(format!("{ID_DOMAIN}\0{kind}\0{key}").as_bytes())
}

/// A Designed county-industry cohort, not an observed factory or employer.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialSiteV1 {
    pub key: String,
    pub label: String,
    pub county_geoid: String,
    pub naics: String,
}
impl MichiganMaterialSiteV1 {
    #[must_use]
    pub fn id(&self) -> SiteIdV1 {
        SiteIdV1::from_bytes(identity("site", &self.key))
    }
    #[must_use]
    pub fn node_id(&self) -> LogisticsNodeIdV2 {
        LogisticsNodeIdV2::from_bytes(identity("node", &self.key))
    }
}

/// Compatible exact physical unit; no monetary principal exists here.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialGoodV1 {
    pub key: String,
    pub label: String,
    pub unit_key: String,
}
impl MichiganMaterialGoodV1 {
    #[must_use]
    pub fn id(&self) -> GoodIdV1 {
        GoodIdV1::from_bytes(identity("good", &self.key))
    }
    #[must_use]
    pub fn unit_id(&self) -> UnitIdV1 {
        UnitIdV1::from_bytes(identity("unit", &self.unit_key))
    }
}

/// Entirely Designed physical recipe and finite labor-time budget.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialProcessV1 {
    pub key: String,
    pub site_key: String,
    pub input_good_key: String,
    pub input_quantity_per_batch: u64,
    pub output_good_key: String,
    pub output_quantity_per_batch: u64,
    pub capacity_batches_per_week: u64,
    pub labor_hours_per_batch: u64,
    pub labor_capacity_hours_per_week: u64,
    pub opening_input_quantity: u64,
    pub opening_planned_batches: u64,
}
impl MichiganMaterialProcessV1 {
    #[must_use]
    pub fn id(&self) -> ProcessIdV1 {
        ProcessIdV1::from_bytes(identity("process", &self.key))
    }
    #[must_use]
    pub fn site_id(&self) -> SiteIdV1 {
        SiteIdV1::from_bytes(identity("site", &self.site_key))
    }
}

/// Designed aggregate transfer, without an invented real supplier or road.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialRouteV1 {
    pub key: String,
    pub supplier_site_key: String,
    pub buyer_site_key: String,
    pub good_key: String,
    pub ordered_quantity: u64,
    pub capacity_quantity_per_week: u64,
    pub travel_weeks: u16,
}
impl MichiganMaterialRouteV1 {
    #[must_use]
    pub fn id(&self) -> RouteIdV2 {
        RouteIdV2::from_bytes(identity("route", &self.key))
    }
    #[must_use]
    pub fn corridor_id(&self) -> CorridorIdV2 {
        CorridorIdV2::from_bytes(identity("corridor", &self.key))
    }
    #[must_use]
    pub fn order_id(&self) -> OrderIdV1 {
        OrderIdV1::from_bytes(identity("order", &self.key))
    }
}

/// One source row retained with its exact raw county-file identity.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganIndustryBaselineRowV1 {
    pub area_fips: String,
    pub area_title: String,
    pub industry_code: String,
    pub industry_title: String,
    pub own_code: String,
    pub agglvl_code: String,
    pub disclosure_code: String,
    pub annual_avg_estabs_count: u64,
    pub annual_avg_emplvl: u64,
    pub total_annual_wages: u64,
    pub annual_avg_wkly_wage: u64,
    pub source_file: String,
    pub source_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct IndustryArtifact {
    schema: String,
    vintage: u16,
    evidence_class: String,
    source_url: String,
    documentation_url: String,
    rows: Vec<MichiganIndustryBaselineRowV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioArtifact {
    schema: String,
    evidence_class: String,
    horizon_ticks: u64,
    geographic_scale: String,
    terminal_output_disposition: String,
    delayed_route_key: String,
    standard_travel_weeks: u16,
    delayed_travel_weeks: u16,
    sites: Vec<MichiganMaterialSiteV1>,
    goods: Vec<MichiganMaterialGoodV1>,
    processes: Vec<MichiganMaterialProcessV1>,
    routes: Vec<MichiganMaterialRouteV1>,
}

/// Checked metadata shared with read-only projections; observations stay separate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MichiganMaterialCatalogV1 {
    industry: IndustryArtifact,
    scenario: ScenarioArtifact,
}
impl MichiganMaterialCatalogV1 {
    #[must_use]
    pub fn sites(&self) -> &[MichiganMaterialSiteV1] {
        &self.scenario.sites
    }
    #[must_use]
    pub fn goods(&self) -> &[MichiganMaterialGoodV1] {
        &self.scenario.goods
    }
    #[must_use]
    pub fn processes(&self) -> &[MichiganMaterialProcessV1] {
        &self.scenario.processes
    }
    #[must_use]
    pub fn routes(&self) -> &[MichiganMaterialRouteV1] {
        &self.scenario.routes
    }
    #[must_use]
    pub fn source_url(&self) -> &str {
        &self.industry.source_url
    }
    #[must_use]
    pub const fn source_evidence_class(&self) -> EvidenceClass {
        EvidenceClass::Observed
    }
    #[must_use]
    pub const fn physical_evidence_class(&self) -> EvidenceClass {
        EvidenceClass::Designed
    }
    #[must_use]
    pub fn source_vintage(&self) -> u16 {
        self.industry.vintage
    }
    #[must_use]
    pub fn geographic_scale(&self) -> &str {
        &self.scenario.geographic_scale
    }
    #[must_use]
    pub fn terminal_output_disposition(&self) -> &str {
        &self.scenario.terminal_output_disposition
    }
    #[must_use]
    pub fn industry_for_site(
        &self,
        site: &MichiganMaterialSiteV1,
    ) -> Option<&MichiganIndustryBaselineRowV1> {
        self.industry
            .rows
            .iter()
            .find(|row| row.area_fips == site.county_geoid && row.industry_code == site.naics)
    }
    #[must_use]
    pub fn site(&self, key: &str) -> Option<&MichiganMaterialSiteV1> {
        self.sites().iter().find(|row| row.key == key)
    }
    #[must_use]
    pub fn good(&self, key: &str) -> Option<&MichiganMaterialGoodV1> {
        self.goods().iter().find(|row| row.key == key)
    }
    #[must_use]
    pub fn travel_weeks(
        &self,
        route: &MichiganMaterialRouteV1,
        preset: MichiganDeliveryPresetV1,
    ) -> u16 {
        if preset == MichiganDeliveryPresetV1::Delayed
            && route.key == self.scenario.delayed_route_key
        {
            self.scenario.delayed_travel_weeks
        } else {
            route.travel_weeks
        }
    }
}

fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 64
        && key
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
}

fn parse_catalog(
    industry_bytes: &[u8],
    scenario_bytes: &[u8],
) -> Result<MichiganMaterialCatalogV1, MichiganMaterialErrorV1> {
    let industry: IndustryArtifact = serde_json::from_slice(industry_bytes)
        .map_err(|_| MichiganMaterialErrorV1::ArtifactDecode)?;
    let scenario: ScenarioArtifact = serde_json::from_slice(scenario_bytes)
        .map_err(|_| MichiganMaterialErrorV1::ArtifactDecode)?;
    let catalog = MichiganMaterialCatalogV1 { industry, scenario };
    validate_catalog(&catalog)?;
    Ok(catalog)
}

fn validate_catalog(catalog: &MichiganMaterialCatalogV1) -> Result<(), MichiganMaterialErrorV1> {
    let source = &catalog.industry;
    let design = &catalog.scenario;
    if source.schema != "MichiganIndustryBaselineV1"
        || source.vintage != 2024
        || source.evidence_class != "Observed"
        || source.source_url != SOURCE_URL
        || source.documentation_url != "https://www.bls.gov/cew/downloadable-data-files.htm"
        || source.rows.len() != 5
        || design.schema != "MichiganMaterialScenarioV1"
        || design.evidence_class != "Designed"
        || design.horizon_ticks != MichiganDeliveryPresetV1::Standard.horizon_ticks()
        || design.geographic_scale != "county_industry_aggregate"
        || design.terminal_output_disposition != "on_hand_unsold"
        || design.sites.len() != 5
        || design.goods.len() != 7
        || design.processes.len() != 5
        || design.routes.len() != 3
        || design.standard_travel_weeks != 1
        || design.delayed_travel_weeks != 3
    {
        return Err(MichiganMaterialErrorV1::ArtifactShape);
    }
    validate_industry_rows(source)?;
    validate_sites_and_goods(catalog)?;
    validate_processes(catalog)?;
    validate_routes(catalog)
}

fn validate_industry_rows(source: &IndustryArtifact) -> Result<(), MichiganMaterialErrorV1> {
    let mut observed = BTreeSet::new();
    for row in &source.rows {
        if !row.disclosure_code.is_empty() {
            return Err(MichiganMaterialErrorV1::SourceSuppressed);
        }
        if row.own_code != "5"
            || !matches!(row.industry_code.len(), 3 | 4)
            || row.agglvl_code
                != if row.industry_code.len() == 3 {
                    "75"
                } else {
                    "76"
                }
            || row.area_fips.len() != 5
            || !row.area_fips.starts_with("26")
            || !row.area_fips.bytes().all(|byte| byte.is_ascii_digit())
            || !row.industry_code.bytes().all(|byte| byte.is_ascii_digit())
            || !row
                .industry_title
                .starts_with(&format!("NAICS {} ", row.industry_code))
            || !row
                .source_file
                .starts_with(&format!("2024.annual {} ", row.area_fips))
            || row.source_file.contains(['/', '\\'])
            || row.source_sha256.len() != 64
            || !row
                .source_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
            || row.annual_avg_emplvl == 0
            || row.annual_avg_estabs_count == 0
            || !observed.insert((&row.area_fips, &row.industry_code))
        {
            return Err(MichiganMaterialErrorV1::SourceValue);
        }
    }
    Ok(())
}

fn validate_sites_and_goods(
    catalog: &MichiganMaterialCatalogV1,
) -> Result<(), MichiganMaterialErrorV1> {
    let mut keys = BTreeSet::new();
    for site in catalog.sites() {
        if !valid_key(&site.key)
            || !keys.insert(&site.key)
            || catalog.industry_for_site(site).is_none()
        {
            return Err(MichiganMaterialErrorV1::ContentReference);
        }
    }
    keys.clear();
    for good in catalog.goods() {
        if !valid_key(&good.key)
            || !keys.insert(&good.key)
            || !matches!(good.unit_key.as_str(), "kg" | "panel" | "subassembly")
        {
            return Err(MichiganMaterialErrorV1::ContentReference);
        }
    }
    Ok(())
}

fn validate_processes(catalog: &MichiganMaterialCatalogV1) -> Result<(), MichiganMaterialErrorV1> {
    let mut keys = BTreeSet::new();
    let mut process_sites = BTreeSet::new();
    for process in catalog.processes() {
        if !valid_key(&process.key)
            || !keys.insert(&process.key)
            || !process_sites.insert(&process.site_key)
            || catalog.site(&process.site_key).is_none()
            || catalog.good(&process.input_good_key).is_none()
            || catalog.good(&process.output_good_key).is_none()
            || process.input_good_key == process.output_good_key
        {
            return Err(MichiganMaterialErrorV1::ContentReference);
        }
        let input_needed = process
            .input_quantity_per_batch
            .checked_mul(process.opening_planned_batches)
            .ok_or(MichiganMaterialErrorV1::ContentValue)?;
        let labor_needed = process
            .labor_hours_per_batch
            .checked_mul(process.opening_planned_batches)
            .ok_or(MichiganMaterialErrorV1::ContentValue)?;
        if process.input_quantity_per_batch == 0
            || process.output_quantity_per_batch == 0
            || process.labor_hours_per_batch == 0
            || process.capacity_batches_per_week == 0
            || process.labor_capacity_hours_per_week == 0
            || process.opening_planned_batches > process.capacity_batches_per_week
            || input_needed > process.opening_input_quantity
            || labor_needed > process.labor_capacity_hours_per_week
        {
            return Err(MichiganMaterialErrorV1::ContentValue);
        }
    }
    Ok(())
}

fn validate_routes(catalog: &MichiganMaterialCatalogV1) -> Result<(), MichiganMaterialErrorV1> {
    let design = &catalog.scenario;
    let mut keys = BTreeSet::new();
    for route in catalog.routes() {
        if !valid_key(&route.key)
            || !keys.insert(&route.key)
            || catalog.site(&route.supplier_site_key).is_none()
            || catalog.site(&route.buyer_site_key).is_none()
            || catalog.good(&route.good_key).is_none()
            || !catalog.processes().iter().any(|process| {
                process.site_key == route.supplier_site_key
                    && process.output_good_key == route.good_key
            })
            || !catalog.processes().iter().any(|process| {
                process.site_key == route.buyer_site_key && process.input_good_key == route.good_key
            })
        {
            return Err(MichiganMaterialErrorV1::ContentReference);
        }
        if route.ordered_quantity == 0
            || route.capacity_quantity_per_week == 0
            || route.travel_weeks != design.standard_travel_weeks
        {
            return Err(MichiganMaterialErrorV1::ContentValue);
        }
    }
    if !catalog
        .routes()
        .iter()
        .any(|route| route.key == design.delayed_route_key)
    {
        return Err(MichiganMaterialErrorV1::ContentReference);
    }
    Ok(())
}

/// Read the exact bounded artifacts once, independent of the acquisition host.
/// # Errors
/// Refuses changed bytes, malformed fields, suppressed sources or invalid content.
pub fn michigan_material_catalog_v1(
) -> Result<&'static MichiganMaterialCatalogV1, MichiganMaterialErrorV1> {
    static CATALOG: OnceLock<Result<MichiganMaterialCatalogV1, MichiganMaterialErrorV1>> =
        OnceLock::new();
    CATALOG
        .get_or_init(|| {
            if crate::michigan_economy::digest_hex(&sha256_of(INDUSTRY_BYTES))
                != MICHIGAN_INDUSTRY_BASELINE_SHA256_V1
                || crate::michigan_economy::digest_hex(&sha256_of(SCENARIO_BYTES))
                    != MICHIGAN_MATERIAL_SCENARIO_SHA256_V1
            {
                return Err(MichiganMaterialErrorV1::ArtifactDigest);
            }
            parse_catalog(INDUSTRY_BYTES, SCENARIO_BYTES)
        })
        .as_ref()
        .map_err(|error| *error)
}

/// Construct one canonical opening state. The session enforces the 16-week horizon.
/// # Errors
/// Refuses source/content defects or the exact V2 circuit validation failure.
pub fn michigan_material_foundation_v1(
    preset: MichiganDeliveryPresetV1,
) -> Result<MaterialCircuitStateV2, MichiganMaterialErrorV1> {
    let catalog = michigan_material_catalog_v1()?;
    let mut state = MaterialCircuitStateV2 {
        week: 1,
        site_logistics_nodes: Vec::new(),
        process_outputs: Vec::new(),
        input_coefficients: Vec::new(),
        labor_coefficients: Vec::new(),
        supplier_routes: Vec::new(),
        route_legs: Vec::new(),
        inventory: Vec::new(),
        orders: Vec::new(),
        backlog: Vec::new(),
        freight: Vec::new(),
        corridor_capacities: Vec::new(),
        capacities: Vec::new(),
        labor: Vec::new(),
        production_commitments: Vec::new(),
    };
    for site in catalog.sites() {
        state.site_logistics_nodes.push(SiteLogisticsNodeV2 {
            site_id: site.id(),
            node_id: site.node_id(),
        });
    }
    for process in catalog.processes() {
        add_process(&mut state, catalog, process, preset.horizon_ticks())?;
    }
    for route in catalog.routes() {
        add_route(&mut state, catalog, route, preset)?;
    }
    let bytes =
        encode_material_circuit_state_v2(&state).map_err(MichiganMaterialErrorV1::Circuit)?;
    decode_material_circuit_state_v2(&bytes).map_err(MichiganMaterialErrorV1::Circuit)
}

fn add_process(
    state: &mut MaterialCircuitStateV2,
    catalog: &MichiganMaterialCatalogV1,
    process: &MichiganMaterialProcessV1,
    horizon: u64,
) -> Result<(), MichiganMaterialErrorV1> {
    let input = catalog
        .good(&process.input_good_key)
        .ok_or(MichiganMaterialErrorV1::ContentReference)?;
    let output = catalog
        .good(&process.output_good_key)
        .ok_or(MichiganMaterialErrorV1::ContentReference)?;
    let site_id = process.site_id();
    let process_id = process.id();
    let labor_unit = UnitIdV1::from_bytes(identity("unit", "labor-hour"));
    state.process_outputs.push(ProcessOutputV1 {
        process_id,
        site_id,
        good_id: output.id(),
        unit_id: output.unit_id(),
        quantity_per_batch: process.output_quantity_per_batch,
    });
    state.input_coefficients.push(InputOutputCoefficientV1 {
        process_id,
        good_id: input.id(),
        unit_id: input.unit_id(),
        quantity_per_batch: process.input_quantity_per_batch,
    });
    state.labor_coefficients.push(LaborCoefficientV1 {
        process_id,
        unit_id: labor_unit,
        quantity_per_batch: process.labor_hours_per_batch,
    });
    state.inventory.extend([
        InventoryRowV1 {
            site_id,
            good_id: input.id(),
            unit_id: input.unit_id(),
            quantity: process.opening_input_quantity,
        },
        InventoryRowV1 {
            site_id,
            good_id: output.id(),
            unit_id: output.unit_id(),
            quantity: 0,
        },
    ]);
    for week in 1..=horizon {
        state.capacities.push(CapacityRowV1 {
            process_id,
            site_id,
            week,
            available_batches: process.capacity_batches_per_week,
        });
        state.labor.push(LaborCapacityRowV1 {
            site_id,
            unit_id: labor_unit,
            week,
            available: process.labor_capacity_hours_per_week,
        });
    }
    if process.opening_planned_batches > 0 {
        state.production_commitments.push(ProductionCommitmentV1 {
            process_id,
            site_id,
            week: 1,
            planned_batches: process.opening_planned_batches,
        });
    }
    Ok(())
}

fn add_route(
    state: &mut MaterialCircuitStateV2,
    catalog: &MichiganMaterialCatalogV1,
    route: &MichiganMaterialRouteV1,
    preset: MichiganDeliveryPresetV1,
) -> Result<(), MichiganMaterialErrorV1> {
    let supplier = catalog
        .site(&route.supplier_site_key)
        .ok_or(MichiganMaterialErrorV1::ContentReference)?;
    let buyer = catalog
        .site(&route.buyer_site_key)
        .ok_or(MichiganMaterialErrorV1::ContentReference)?;
    let good = catalog
        .good(&route.good_key)
        .ok_or(MichiganMaterialErrorV1::ContentReference)?;
    state.supplier_routes.push(SupplierRouteV2 {
        buyer_site_id: buyer.id(),
        supplier_site_id: supplier.id(),
        good_id: good.id(),
        unit_id: good.unit_id(),
        route_id: route.id(),
    });
    state.route_legs.push(RouteLegV2 {
        route_id: route.id(),
        leg_index: 0,
        corridor_id: route.corridor_id(),
        from_node_id: supplier.node_id(),
        to_node_id: buyer.node_id(),
        travel_weeks: catalog.travel_weeks(route, preset),
        loss_ppm: 0,
    });
    state.orders.push(OrderRowV2 {
        order_id: route.order_id(),
        access_mode: OrderAccessModeV1::CommoditySale,
        buyer_site_id: buyer.id(),
        supplier_site_id: supplier.id(),
        good_id: good.id(),
        unit_id: good.unit_id(),
        ordered: route.ordered_quantity,
        shipped: 0,
        lost: 0,
        delivered: 0,
        realized: 0,
    });
    state.backlog.push(BacklogRowV1 {
        order_id: route.order_id(),
        quantity: route.ordered_quantity,
    });
    for week in 1..=preset.horizon_ticks() {
        state.corridor_capacities.push(CorridorCapacityV2 {
            corridor_id: route.corridor_id(),
            unit_id: good.unit_id(),
            week,
            available: route.capacity_quantity_per_week,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suppressed_source_zeros_are_refused() {
        let mut source: serde_json::Value = serde_json::from_slice(INDUSTRY_BYTES).unwrap();
        source["rows"][0]["disclosure_code"] = "N".into();
        source["rows"][0]["annual_avg_emplvl"] = 0.into();
        assert_eq!(
            parse_catalog(&serde_json::to_vec(&source).unwrap(), SCENARIO_BYTES),
            Err(MichiganMaterialErrorV1::SourceSuppressed)
        );
    }

    #[test]
    fn absent_source_binding_and_extra_factory_coordinates_refuse() {
        let mut design: serde_json::Value = serde_json::from_slice(SCENARIO_BYTES).unwrap();
        design["sites"][0]["county_geoid"] = "01001".into();
        assert_eq!(
            parse_catalog(INDUSTRY_BYTES, &serde_json::to_vec(&design).unwrap()),
            Err(MichiganMaterialErrorV1::ContentReference)
        );
        design["sites"][0]["latitude"] = 42.into();
        assert_eq!(
            parse_catalog(INDUSTRY_BYTES, &serde_json::to_vec(&design).unwrap()),
            Err(MichiganMaterialErrorV1::ArtifactDecode)
        );
    }
}
