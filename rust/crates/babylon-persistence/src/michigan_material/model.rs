//! One normalized physical-content model for regional and statewide campaigns.

use super::{identity, MichiganDeliveryPreset};
use babylon_material_circuit::{
    CorridorId, FinalDemandPrincipalId, GoodId, LogisticsNodeId, OrderId, ProcessId, RouteId,
    SiteId, UnitId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MichiganSiteRole {
    Production,
    Wholesale,
    Retail,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialSite {
    pub key: String,
    pub label: String,
    pub county_geoid: String,
    pub naics: String,
    pub sector_code: String,
    pub role: MichiganSiteRole,
}
impl MichiganMaterialSite {
    #[must_use]
    pub fn id(&self) -> SiteId {
        SiteId::from_bytes(identity("site", &self.key))
    }
    #[must_use]
    pub fn node_id(&self) -> LogisticsNodeId {
        LogisticsNodeId::from_bytes(identity("node", &self.key))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialGood {
    pub key: String,
    pub label: String,
    pub unit_key: String,
    pub grams_per_unit: u64,
}
impl MichiganMaterialGood {
    #[must_use]
    pub fn id(&self) -> GoodId {
        GoodId::from_bytes(identity("good", &self.key))
    }
    #[must_use]
    pub fn unit_id(&self) -> UnitId {
        UnitId::from_bytes(identity("unit", &self.unit_key))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialInput {
    pub good_key: String,
    pub quantity_per_batch: u64,
    pub opening_quantity: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialProcess {
    pub key: String,
    pub site_key: String,
    pub industry_code: String,
    pub inputs: Vec<MichiganMaterialInput>,
    pub output_good_key: String,
    pub output_quantity_per_batch: u64,
    pub capacity_batches_per_period: u64,
    pub labor_hours_per_batch: u64,
    pub opening_planned_batches: u64,
}
impl MichiganMaterialProcess {
    #[must_use]
    pub fn id(&self) -> ProcessId {
        ProcessId::from_bytes(identity("process", &self.key))
    }
    #[must_use]
    pub fn site_id(&self) -> SiteId {
        SiteId::from_bytes(identity("site", &self.site_key))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganWorkforceSeed {
    pub key: String,
    pub site_key: String,
    pub process_keys: Vec<String>,
    pub merchant_handling: bool,
    pub employed: u64,
    pub reserve: u64,
    pub previous_unretained_hours: u64,
}
impl MichiganWorkforceSeed {
    #[must_use]
    pub fn local_name(&self) -> String {
        format!("workforce-{}", self.key)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganStaffingDesign {
    pub composition_id: String,
    pub role: String,
    pub evidence_class: String,
    pub placement: String,
    pub hours_per_worker_period: u64,
    pub retention_periods: u8,
    pub pools: Vec<MichiganWorkforceSeed>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MichiganMaterialPath {
    Local,
    Routed {
        travel_periods: u16,
        capacity_keys: Vec<String>,
        physical_edge_keys: Vec<String>,
        distance_mm: Option<u64>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialRoute {
    pub key: String,
    pub supplier_site_key: String,
    pub buyer_site_key: String,
    pub good_key: String,
    pub ordered_quantity: u64,
    pub path: MichiganMaterialPath,
}
impl MichiganMaterialRoute {
    #[must_use]
    pub fn id(&self) -> RouteId {
        RouteId::from_bytes(identity("route", &self.key))
    }
    #[must_use]
    pub fn order_id(&self) -> OrderId {
        OrderId::from_bytes(identity("order", &self.key))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialCorridor {
    pub key: String,
    pub label: String,
    pub capacity_grams_per_period: u64,
}
impl MichiganMaterialCorridor {
    #[must_use]
    pub fn id(&self) -> CorridorId {
        CorridorId::from_bytes(identity("corridor", &self.key))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMerchant {
    pub site_key: String,
    pub capacity_key: String,
    pub handling_hours_per_unit: BTreeMap<String, u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganFinalDemand {
    pub key: String,
    pub retailer_site_key: String,
    pub county_geoid: String,
    pub good_key: String,
    pub ordered_quantity: u64,
}
impl MichiganFinalDemand {
    #[must_use]
    pub fn order_id(&self) -> OrderId {
        OrderId::from_bytes(identity("final-demand-order", &self.key))
    }
    #[must_use]
    pub fn principal_id(&self) -> FinalDemandPrincipalId {
        FinalDemandPrincipalId::from_bytes(identity("final-demand-principal", &self.county_geoid))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganOwnerSource {
    pub county_geoid: String,
    pub sector_code: String,
    pub county_source_file: String,
    pub sector_title: String,
    pub sector_disposition: String,
    pub disclosure_code: String,
    pub annual_avg_estabs_count: u64,
    pub annual_avg_emplvl: Option<u64>,
    pub total_annual_wages: Option<u64>,
    pub annual_avg_wkly_wage: Option<u64>,
    pub county_source_sha256: String,
    pub sector_artifact_sha256: String,
    pub sector_semantic_sha256: String,
    pub industry_artifact_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganIndustryBaselineRow {
    pub area_fips: String,
    pub area_title: String,
    pub industry_code: String,
    pub industry_title: String,
    pub own_code: String,
    pub agglvl_code: String,
    pub disclosure_code: String,
    pub annual_avg_estabs_count: u64,
    pub annual_avg_emplvl: Option<u64>,
    pub total_annual_wages: Option<u64>,
    pub annual_avg_wkly_wage: Option<u64>,
    pub source_file: String,
    pub source_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganRoadSource {
    pub pbf_sha256: String,
    pub pbf_bytes: u64,
    pub pbf_url: String,
    pub replication_timestamp: String,
    pub footprint_sha256: String,
    pub buffer_degrees_e7: u64,
    pub extraction_version: String,
    pub distance_version: String,
    pub routing_profile_version: String,
    pub graph_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganVehicleProfile {
    pub gross_weight_kg: u64,
    pub height_mm: u64,
    pub width_mm: u64,
    pub length_mm: u64,
    pub default_maxheight_mm: u64,
    pub axle_load_kg: Option<u64>,
    pub evidence_class: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganCountyTerminal {
    pub county_geoid: String,
    pub node_id: i64,
    pub county_name: String,
    pub anchor_lon_e7: i64,
    pub anchor_lat_e7: i64,
    pub atlas_grid_x: i64,
    pub atlas_grid_y: i64,
    pub node_lon_e7: i64,
    pub node_lat_e7: i64,
    pub attachment_distance_mm: u64,
    pub status: String,
    pub evidence_class: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganPhysicalEdge {
    pub id: String,
    pub way_id: i64,
    pub from_node: i64,
    pub to_node: i64,
    pub distance_mm: u64,
    pub shape_e7: Vec<[i64; 2]>,
    pub tags: BTreeMap<String, String>,
    pub way_version: u64,
    pub way_timestamp: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganPhysicalCapacityGroup {
    pub key: String,
    pub label: String,
    pub edge_keys: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganPhysicalNetwork {
    pub source: MichiganRoadSource,
    pub profile: MichiganVehicleProfile,
    pub terminal_source_pins: BTreeMap<String, String>,
    pub terminal_policy: MichiganTerminalPolicy,
    pub terminal_attachment_limit_meters: u64,
    pub terminals: Vec<MichiganCountyTerminal>,
    pub edges: Vec<MichiganPhysicalEdge>,
    pub capacity_groups: Vec<MichiganPhysicalCapacityGroup>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganCapacityOverride {
    pub capacity_key: String,
    pub grams_per_period: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganOpeningStockOverride {
    pub process_key: String,
    pub good_key: String,
    pub quantity: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganRouteOverride {
    pub route_key: String,
    pub path: MichiganMaterialPath,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganIntervention {
    pub preset: MichiganDeliveryPreset,
    pub capacities: Vec<MichiganCapacityOverride>,
    pub opening_stocks: Vec<MichiganOpeningStockOverride>,
    pub routes: Vec<MichiganRouteOverride>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganNormalizedContent {
    pub schema: String,
    pub evidence_class: String,
    pub horizon_ticks: u64,
    pub tick_duration_days: u64,
    pub geographic_scale: String,
    pub terminal_output_disposition: String,
    pub sites: Vec<MichiganMaterialSite>,
    pub goods: Vec<MichiganMaterialGood>,
    pub processes: Vec<MichiganMaterialProcess>,
    pub routes: Vec<MichiganMaterialRoute>,
    pub corridors: Vec<MichiganMaterialCorridor>,
    pub staffing: MichiganStaffingDesign,
    pub merchants: Vec<MichiganMerchant>,
    pub final_demands: Vec<MichiganFinalDemand>,
    pub owners: Vec<MichiganOwnerSource>,
    pub industry: Vec<MichiganIndustryBaselineRow>,
    pub physical_network: Option<MichiganPhysicalNetwork>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganTerminalPolicy {
    pub terminal_evidence_class: String,
    pub anchor: String,
    pub attachment_limit_mm: u64,
    pub attachment_distance: String,
    pub usable_node: String,
    pub physical_path_distance: String,
    pub projection: String,
}
