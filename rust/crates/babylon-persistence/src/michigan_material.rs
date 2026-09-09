//! Bounded Designed physical content with separate observed county-industry evidence.
//!
//! The authoritative session owns advancement, publication and the finite horizon.
//! This module constructs an exact initial state; it does not run mechanics.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::OnceLock;

use crate::michigan_defines::{MichiganDefinesErrorV1, MichiganDefinesV2};

use babylon_bsl::causal_contract::EvidenceClass;
use babylon_kernel::sha256_of;
use babylon_material_circuit::{
    CorridorIdV2, GoodIdV1, LogisticsNodeIdV2, MaterialCircuitErrorV2, OrderIdV1, ProcessIdV1,
    RouteIdV2, SiteIdV1, UnitIdV1,
};
use serde::{Deserialize, Serialize};

/// Exact observed five-row public industry source artifact.
pub const MICHIGAN_INDUSTRY_BASELINE_SHA256_V1: &str =
    "eb486d7e11b8b63fc58c53ab918eff84b341b293a66faf422ddb9304fb2b553e";
const INDUSTRY_BYTES: &[u8] =
    include_bytes!("../../../../contracts/fixtures/michigan_industry_baseline_v1.json");
const TOPOLOGY_BYTES: &[u8] =
    include_bytes!("../../../../content/scenarios/michigan/topology.json");
const ID_DOMAIN: &str = "babylon.michigan-material.v1";
const SOURCE_URL: &str = "https://data.bls.gov/cew/data/files/2024/csv/2024_annual_by_area.zip";

pub const MICHIGAN_MAX_HORIZON_PERIODS_V1: u64 = 16;

/// Separately committed comparisons; no player intervention is implied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganDeliveryPresetV1 {
    Standard,
    Delayed,
    SharedFreightAmple,
    SharedFreightConstrained,
}
impl MichiganDeliveryPresetV1 {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Standard => "michigan-material-standard-v6",
            Self::Delayed => "michigan-material-delayed-v6",
            Self::SharedFreightAmple => "michigan-material-shared-freight-ample-v6",
            Self::SharedFreightConstrained => "michigan-material-shared-freight-constrained-v6",
        }
    }
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "michigan-material-standard-v6" => Some(Self::Standard),
            "michigan-material-delayed-v6" => Some(Self::Delayed),
            "michigan-material-shared-freight-ample-v6" => Some(Self::SharedFreightAmple),
            "michigan-material-shared-freight-constrained-v6" => {
                Some(Self::SharedFreightConstrained)
            }
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
    pub capacity_batches_per_period: u64,
    pub labor_hours_per_batch: u64,
    pub labor_capacity_hours_per_period: u64,
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

/// Explicit Designed workforce seeds; these are not QCEW employment.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganWorkforceSeedV1 {
    pub process_key: String,
    pub employed: u64,
    pub reserve: u64,
    pub previous_unretained_hours: u64,
}
impl MichiganWorkforceSeedV1 {
    #[must_use]
    pub fn local_name(&self) -> String {
        format!("workforce-{}", self.process_key)
    }
}

/// Closed policy and placement of the admitted native staffing composition.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganStaffingDesignV1 {
    pub composition_id: String,
    pub role: String,
    pub evidence_class: String,
    pub placement: String,
    pub hours_per_worker_period: u64,
    pub retention_periods: u8,
    pub pools: Vec<MichiganWorkforceSeedV1>,
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
    pub corridor_key: String,
    pub shared_corridor_key: String,
    pub travel_periods: u16,
    pub delayed_travel_periods: u16,
}
impl MichiganMaterialRouteV1 {
    #[must_use]
    pub fn id(&self) -> RouteIdV2 {
        RouteIdV2::from_bytes(identity("route", &self.key))
    }
    #[must_use]
    pub fn order_id(&self) -> OrderIdV1 {
        OrderIdV1::from_bytes(identity("order", &self.key))
    }
}

/// One capacity principal; a shared allotment is not a named physical road.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialCorridorV1 {
    pub key: String,
    pub label: String,
    pub unit_key: String,
    capacity: MichiganCorridorCapacityV1,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
enum MichiganCorridorCapacityV1 {
    Independent(u64),
    SharedFreight { ample: u64, constrained: u64 },
}
impl MichiganMaterialCorridorV1 {
    #[must_use]
    pub fn id(&self) -> CorridorIdV2 {
        CorridorIdV2::from_bytes(identity("corridor", &self.key))
    }
    #[must_use]
    pub fn capacity_per_period(&self, preset: MichiganDeliveryPresetV1) -> u64 {
        match self.capacity {
            MichiganCorridorCapacityV1::Independent(value) => value,
            MichiganCorridorCapacityV1::SharedFreight { constrained, .. }
                if preset == MichiganDeliveryPresetV1::SharedFreightConstrained =>
            {
                constrained
            }
            MichiganCorridorCapacityV1::SharedFreight { ample, .. } => ample,
        }
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
    staffing: MichiganStaffingDesignV1,
    schema: String,
    evidence_class: String,
    horizon_ticks: u64,
    tick_duration_days: u64,
    geographic_scale: String,
    terminal_output_disposition: String,
    sites: Vec<MichiganMaterialSiteV1>,
    goods: Vec<MichiganMaterialGoodV1>,
    processes: Vec<MichiganMaterialProcessV1>,
    routes: Vec<MichiganMaterialRouteV1>,
    corridors: Vec<MichiganMaterialCorridorV1>,
}

/// Checked metadata shared with read-only projections; observations stay separate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MichiganMaterialCatalogV1 {
    industry: IndustryArtifact,
    scenario: ScenarioArtifact,
    defines_bytes: Vec<u8>,
}
impl MichiganMaterialCatalogV1 {
    /// Load fresh parameters for a new campaign. No missing-file fallback exists.
    /// # Errors
    /// Refuses missing, oversized, malformed, unknown or invalid parameter values.
    pub fn load_defines(path: &Path) -> Result<Self, MichiganDefinesErrorV1> {
        Self::from_defines(&MichiganDefinesV2::load(path)?)
    }
    /// Parse explicit values; useful for authored content and independent tests.
    /// # Errors
    /// Refuses malformed or materially inconsistent values.
    pub fn from_defines_toml(text: &str) -> Result<Self, MichiganDefinesErrorV1> {
        Self::from_defines(&MichiganDefinesV2::parse(text)?)
    }
    pub(crate) fn from_stored_defines(bytes: &[u8]) -> Result<Self, MichiganDefinesErrorV1> {
        Self::from_defines(&MichiganDefinesV2::decode(bytes)?)
    }
    fn from_defines(defines: &MichiganDefinesV2) -> Result<Self, MichiganDefinesErrorV1> {
        compile_catalog(INDUSTRY_BYTES, defines)
    }
    #[must_use]
    pub fn defines_bytes(&self) -> &[u8] {
        &self.defines_bytes
    }
    #[must_use]
    pub fn defines_hash(&self) -> [u8; 32] {
        sha256_of(&self.defines_bytes)
    }
    #[must_use]
    pub const fn horizon_ticks(&self) -> u64 {
        self.scenario.horizon_ticks
    }

    #[must_use]
    pub fn staffing(&self) -> &MichiganStaffingDesignV1 {
        &self.scenario.staffing
    }
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
    pub fn travel_periods(
        &self,
        route: &MichiganMaterialRouteV1,
        preset: MichiganDeliveryPresetV1,
    ) -> u16 {
        match preset {
            MichiganDeliveryPresetV1::Delayed => route.delayed_travel_periods,
            MichiganDeliveryPresetV1::Standard
            | MichiganDeliveryPresetV1::SharedFreightAmple
            | MichiganDeliveryPresetV1::SharedFreightConstrained => route.travel_periods,
        }
    }
    #[must_use]
    pub fn corridor_for_route(
        &self,
        route: &MichiganMaterialRouteV1,
        preset: MichiganDeliveryPresetV1,
    ) -> Option<&MichiganMaterialCorridorV1> {
        let key = match preset {
            MichiganDeliveryPresetV1::Standard | MichiganDeliveryPresetV1::Delayed => {
                &route.corridor_key
            }
            MichiganDeliveryPresetV1::SharedFreightAmple
            | MichiganDeliveryPresetV1::SharedFreightConstrained => &route.shared_corridor_key,
        };
        self.scenario.corridors.iter().find(|c| &c.key == key)
    }
    #[must_use]
    pub fn corridor_label(&self, id: CorridorIdV2) -> Option<&str> {
        self.scenario
            .corridors
            .iter()
            .find(|c| c.id() == id)
            .map(|c| c.label.as_str())
    }
}

fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 64
        && key
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MaterialTopology {
    pub sites: Vec<MichiganMaterialSiteV1>,
    pub goods: Vec<MichiganMaterialGoodV1>,
    pub processes: Vec<ProcessTopology>,
    pub routes: Vec<RouteTopology>,
    corridors: Vec<CorridorTopology>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProcessTopology {
    pub key: String,
    pub site_key: String,
    pub input_good_key: String,
    pub output_good_key: String,
}
impl ProcessTopology {
    pub fn id(&self) -> ProcessIdV1 {
        ProcessIdV1::from_bytes(identity("process", &self.key))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RouteTopology {
    key: String,
    corridor_key: String,
    shared_corridor_key: String,
    supplier_site_key: String,
    buyer_site_key: String,
    good_key: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CorridorTopology {
    key: String,
    label: String,
    unit_key: String,
}
/// Only immutable topology is cached; numeric campaign values never enter this cache.
pub(crate) fn material_topology() -> Result<&'static MaterialTopology, MichiganMaterialErrorV1> {
    static TOPOLOGY: OnceLock<Result<MaterialTopology, MichiganMaterialErrorV1>> = OnceLock::new();
    TOPOLOGY
        .get_or_init(|| {
            serde_json::from_slice(TOPOLOGY_BYTES)
                .map_err(|_| MichiganMaterialErrorV1::ArtifactDecode)
        })
        .as_ref()
        .map_err(|e| *e)
}

impl MaterialTopology {
    pub fn goods(&self) -> &[MichiganMaterialGoodV1] {
        &self.goods
    }
    pub fn processes(&self) -> &[ProcessTopology] {
        &self.processes
    }
    pub fn site(&self, key: &str) -> Option<&MichiganMaterialSiteV1> {
        self.sites.iter().find(|s| s.key == key)
    }
    pub fn industry_for_site(
        site: &MichiganMaterialSiteV1,
    ) -> Option<&'static MichiganIndustryBaselineRowV1> {
        static SOURCE: OnceLock<Result<IndustryArtifact, MichiganMaterialErrorV1>> =
            OnceLock::new();
        SOURCE
            .get_or_init(|| {
                if crate::michigan_economy::digest_hex(&sha256_of(INDUSTRY_BYTES))
                    != MICHIGAN_INDUSTRY_BASELINE_SHA256_V1
                {
                    return Err(MichiganMaterialErrorV1::ArtifactDigest);
                }
                let source: IndustryArtifact = serde_json::from_slice(INDUSTRY_BYTES)
                    .map_err(|_| MichiganMaterialErrorV1::ArtifactDecode)?;
                validate_industry_rows(&source)?;
                Ok(source)
            })
            .as_ref()
            .ok()?
            .rows
            .iter()
            .find(|r| r.area_fips == site.county_geoid && r.industry_code == site.naics)
    }
}

fn compile_catalog(
    industry_bytes: &[u8],
    defines: &MichiganDefinesV2,
) -> Result<MichiganMaterialCatalogV1, MichiganDefinesErrorV1> {
    use MichiganDefinesErrorV1::Material;
    if crate::michigan_economy::digest_hex(&sha256_of(industry_bytes))
        != MICHIGAN_INDUSTRY_BASELINE_SHA256_V1
    {
        return Err(Material(MichiganMaterialErrorV1::ArtifactDigest));
    }
    let industry = serde_json::from_slice(industry_bytes)
        .map_err(|_| Material(MichiganMaterialErrorV1::ArtifactDecode))?;
    let topology = material_topology().map_err(Material)?;
    let mut processes = Vec::new();
    let mut pools = Vec::new();
    let hours = defines.hours_per_period();
    for binding in &topology.processes {
        let values = defines
            .process
            .get(&binding.key.replace('-', "_"))
            .ok_or(Material(MichiganMaterialErrorV1::ContentReference))?;
        let labor = values
            .employed_people
            .checked_mul(hours)
            .ok_or(Material(MichiganMaterialErrorV1::ContentValue))?;
        processes.push(MichiganMaterialProcessV1 {
            key: binding.key.clone(),
            site_key: binding.site_key.clone(),
            input_good_key: binding.input_good_key.clone(),
            output_good_key: binding.output_good_key.clone(),
            input_quantity_per_batch: values.input_units_per_batch,
            output_quantity_per_batch: values.output_units_per_batch,
            capacity_batches_per_period: values.batches_per_week
                * babylon_kernel::clock::WEEKS_PER_TICK,
            labor_hours_per_batch: values.labor_hours_per_batch,
            labor_capacity_hours_per_period: labor,
            opening_input_quantity: values.opening_input_units,
            opening_planned_batches: values.opening_planned_batches,
        });
        pools.push(MichiganWorkforceSeedV1 {
            process_key: binding.key.clone(),
            employed: values.employed_people,
            reserve: values.reserve_people,
            previous_unretained_hours: labor,
        });
    }
    pools.sort_unstable_by(|a, b| a.process_key.cmp(&b.process_key));
    let mut routes = Vec::new();
    for binding in &topology.routes {
        let values = defines
            .route
            .get(&binding.key.replace('-', "_"))
            .ok_or(Material(MichiganMaterialErrorV1::ContentReference))?;
        routes.push(MichiganMaterialRouteV1 {
            key: binding.key.clone(),
            supplier_site_key: binding.supplier_site_key.clone(),
            buyer_site_key: binding.buyer_site_key.clone(),
            good_key: binding.good_key.clone(),
            ordered_quantity: values.ordered_units,
            corridor_key: binding.corridor_key.clone(),
            shared_corridor_key: binding.shared_corridor_key.clone(),
            travel_periods: values.travel_periods,
            delayed_travel_periods: values.delayed_travel_periods,
        });
    }
    let corridors = compile_corridors(topology, defines)?;
    let catalog = MichiganMaterialCatalogV1 {
        industry,
        defines_bytes: defines.encode()?,
        scenario: ScenarioArtifact {
            schema: "MichiganMaterialScenarioV2".to_owned(),
            evidence_class: "Designed".to_owned(),
            horizon_ticks: defines.horizon_periods,
            tick_duration_days: defines.tick_duration_days,
            geographic_scale: "county_industry_aggregate".to_owned(),
            terminal_output_disposition: "on_hand_unsold".to_owned(),
            sites: topology.sites.clone(),
            goods: topology.goods.clone(),
            processes,
            routes,
            corridors,
            staffing: MichiganStaffingDesignV1 {
                composition_id: "g4-workforce-staffing".to_owned(),
                role: "Mechanic".to_owned(),
                evidence_class: "Designed".to_owned(),
                placement: "after-metabolism-material-base".to_owned(),
                hours_per_worker_period: hours,
                retention_periods: 1,
                pools,
            },
        },
    };
    validate_inventory_bounds(&catalog)?;
    validate_catalog(&catalog).map_err(Material)?;
    Ok(catalog)
}

fn compile_corridors(
    topology: &MaterialTopology,
    defines: &MichiganDefinesV2,
) -> Result<Vec<MichiganMaterialCorridorV1>, MichiganDefinesErrorV1> {
    use MichiganDefinesErrorV1::Material;
    let mut corridors = Vec::new();
    for binding in &topology.corridors {
        let capacity = if binding.key == "shared-freight" {
            MichiganCorridorCapacityV1::SharedFreight {
                ample: defines.shared_freight.ample_units_per_week
                    * babylon_kernel::clock::WEEKS_PER_TICK,
                constrained: defines.shared_freight.constrained_units_per_week
                    * babylon_kernel::clock::WEEKS_PER_TICK,
            }
        } else {
            let value = defines
                .corridor
                .get(&binding.key.replace('-', "_"))
                .ok_or(Material(MichiganMaterialErrorV1::ContentReference))?;
            MichiganCorridorCapacityV1::Independent(
                value.units_per_week * babylon_kernel::clock::WEEKS_PER_TICK,
            )
        };
        corridors.push(MichiganMaterialCorridorV1 {
            key: binding.key.clone(),
            label: binding.label.clone(),
            unit_key: binding.unit_key.clone(),
            capacity,
        });
    }
    Ok(corridors)
}

/// Bound every possible local inventory addition over the admitted campaign.
/// This is machine representability, independent of which supplies actually arrive.
fn validate_inventory_bounds(
    catalog: &MichiganMaterialCatalogV1,
) -> Result<(), MichiganDefinesErrorV1> {
    use MichiganDefinesErrorV1::Value;
    const REFUSAL: &str =
        "opening stock, horizon production, and incoming orders exceed inventory integer bounds";
    let mut ceilings: BTreeMap<(String, String), u64> = BTreeMap::new();
    let mut add = |site: &str, good: &str, quantity: u64| -> Result<(), MichiganDefinesErrorV1> {
        let ceiling = ceilings
            .entry((site.to_owned(), good.to_owned()))
            .or_default();
        *ceiling = ceiling.checked_add(quantity).ok_or(Value(REFUSAL))?;
        Ok(())
    };
    for process in catalog.processes() {
        add(
            &process.site_key,
            &process.input_good_key,
            process.opening_input_quantity,
        )?;
        let output = process
            .capacity_batches_per_period
            .checked_mul(process.output_quantity_per_batch)
            .and_then(|quantity| quantity.checked_mul(catalog.horizon_ticks()))
            .ok_or(Value(REFUSAL))?;
        add(&process.site_key, &process.output_good_key, output)?;
    }
    for route in catalog.routes() {
        add(
            &route.buyer_site_key,
            &route.good_key,
            route.ordered_quantity,
        )?;
    }
    Ok(())
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
        || design.schema != "MichiganMaterialScenarioV2"
        || design.evidence_class != "Designed"
        || design.tick_duration_days != babylon_kernel::clock::DAYS_PER_TICK
        || !(1..=MICHIGAN_MAX_HORIZON_PERIODS_V1).contains(&design.horizon_ticks)
        || design.geographic_scale != "county_industry_aggregate"
        || design.terminal_output_disposition != "on_hand_unsold"
        || design.sites.len() != 5
        || design.goods.len() != 7
        || design.processes.len() != 5
        || design.routes.len() != 3
        || design.corridors.len() != 4
    {
        return Err(MichiganMaterialErrorV1::ArtifactShape);
    }
    validate_industry_rows(source)?;
    validate_sites_and_goods(catalog)?;
    validate_processes(catalog)?;
    validate_staffing(catalog)?;
    validate_routes(catalog)
}

fn validate_staffing(catalog: &MichiganMaterialCatalogV1) -> Result<(), MichiganMaterialErrorV1> {
    let design = catalog.staffing();
    if design.composition_id != "g4-workforce-staffing"
        || design.role != "Mechanic"
        || design.evidence_class != "Designed"
        || design.placement != "after-metabolism-material-base"
        || design.hours_per_worker_period == 0
        || design.retention_periods != 1
        || design.pools.len() != catalog.processes().len()
        || design
            .pools
            .windows(2)
            .any(|pair| pair[0].process_key >= pair[1].process_key)
    {
        return Err(MichiganMaterialErrorV1::ContentValue);
    }
    for seed in &design.pools {
        let process = catalog
            .processes()
            .iter()
            .find(|p| p.key == seed.process_key)
            .ok_or(MichiganMaterialErrorV1::ContentReference)?;
        if seed.employed == 0
            || seed.employed.checked_mul(design.hours_per_worker_period)
                != Some(process.labor_capacity_hours_per_period)
            || seed.previous_unretained_hours != process.labor_capacity_hours_per_period
            || seed.previous_unretained_hours > (1_u64 << 53)
        {
            return Err(MichiganMaterialErrorV1::ContentValue);
        }
    }
    Ok(())
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
            || process.capacity_batches_per_period == 0
            || process.labor_capacity_hours_per_period == 0
            || process.opening_planned_batches > process.capacity_batches_per_period
            || input_needed > process.opening_input_quantity
            || labor_needed > process.labor_capacity_hours_per_period
        {
            return Err(MichiganMaterialErrorV1::ContentValue);
        }
    }
    Ok(())
}

fn validate_routes(catalog: &MichiganMaterialCatalogV1) -> Result<(), MichiganMaterialErrorV1> {
    let mut corridor_keys = BTreeSet::new();
    for corridor in &catalog.scenario.corridors {
        if !valid_key(&corridor.key)
            || corridor.label.is_empty()
            || !corridor_keys.insert(&corridor.key)
        {
            return Err(MichiganMaterialErrorV1::ContentValue);
        }
    }
    let mut used_corridors = BTreeSet::new();
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
        for preset in [
            MichiganDeliveryPresetV1::Standard,
            MichiganDeliveryPresetV1::SharedFreightAmple,
            MichiganDeliveryPresetV1::SharedFreightConstrained,
        ] {
            let corridor = catalog
                .corridor_for_route(route, preset)
                .ok_or(MichiganMaterialErrorV1::ContentReference)?;
            let good = catalog
                .good(&route.good_key)
                .ok_or(MichiganMaterialErrorV1::ContentReference)?;
            if good.unit_key != corridor.unit_key || corridor.capacity_per_period(preset) == 0 {
                return Err(MichiganMaterialErrorV1::ContentValue);
            }
            used_corridors.insert(&corridor.key);
        }
        if route.ordered_quantity == 0
            || route.travel_periods == 0
            || route.delayed_travel_periods < route.travel_periods
        {
            return Err(MichiganMaterialErrorV1::ContentValue);
        }
    }
    if used_corridors != corridor_keys {
        return Err(MichiganMaterialErrorV1::ContentReference);
    }
    Ok(())
}

#[cfg(test)]
mod parameter_bounds_tests {
    use super::*;
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../content/scenarios/michigan/defines.toml"
    ));
    #[test]
    fn horizon_output_accumulation_and_future_arrivals_must_fit_inventory() {
        let huge_output = SOURCE.replace(
            "OUTPUT_UNITS_PER_BATCH = 10",
            "OUTPUT_UNITS_PER_BATCH = 500000000000000000",
        );
        assert!(matches!(
            MichiganMaterialCatalogV1::from_defines_toml(&huge_output),
            Err(MichiganDefinesErrorV1::Value(_))
        ));
        let mut stored = MichiganDefinesV2::parse(SOURCE).unwrap();
        stored
            .process
            .get_mut("panel_forming")
            .unwrap()
            .opening_input_units = u64::MAX;
        assert!(matches!(
            MichiganMaterialCatalogV1::from_stored_defines(&stored.encode().unwrap()),
            Err(MichiganDefinesErrorV1::Value(_))
        ));
    }
}
