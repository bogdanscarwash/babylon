//! One normalized physical-content model for regional and statewide campaigns.

use super::{identity, MichiganDeliveryPresetV1};
use babylon_material_circuit::{
    CorridorIdV2, FinalDemandPrincipalIdV3, GoodIdV1, LogisticsNodeIdV2, OrderIdV1, ProcessIdV1,
    RouteIdV2, SiteIdV1, UnitIdV1,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MichiganSiteRoleV2 {
    Production,
    Wholesale,
    Retail,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialSiteV1 {
    pub key: String,
    pub label: String,
    pub county_geoid: String,
    pub naics: String,
    pub sector_code: String,
    pub role: MichiganSiteRoleV2,
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
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialGoodV1 {
    pub key: String,
    pub label: String,
    pub unit_key: String,
    pub grams_per_unit: u64,
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
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialInputV2 {
    pub good_key: String,
    pub quantity_per_batch: u64,
    pub opening_quantity: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialProcessV1 {
    pub key: String,
    pub site_key: String,
    pub industry_code: String,
    pub inputs: Vec<MichiganMaterialInputV2>,
    pub output_good_key: String,
    pub output_quantity_per_batch: u64,
    pub capacity_batches_per_period: u64,
    pub labor_hours_per_batch: u64,
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
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganWorkforceSeedV1 {
    pub key: String,
    pub site_key: String,
    pub process_keys: Vec<String>,
    pub merchant_handling: bool,
    pub employed: u64,
    pub reserve: u64,
    pub previous_unretained_hours: u64,
}
impl MichiganWorkforceSeedV1 {
    #[must_use]
    pub fn local_name(&self) -> String {
        format!("workforce-{}", self.key)
    }
}
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
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MichiganMaterialPathV2 {
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
pub struct MichiganMaterialRouteV1 {
    pub key: String,
    pub supplier_site_key: String,
    pub buyer_site_key: String,
    pub good_key: String,
    pub ordered_quantity: u64,
    pub path: MichiganMaterialPathV2,
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
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMaterialCorridorV1 {
    pub key: String,
    pub label: String,
    pub capacity_grams_per_period: u64,
}
impl MichiganMaterialCorridorV1 {
    #[must_use]
    pub fn id(&self) -> CorridorIdV2 {
        CorridorIdV2::from_bytes(identity("corridor", &self.key))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganMerchantV2 {
    pub site_key: String,
    pub capacity_key: String,
    pub handling_hours_per_unit: BTreeMap<String, u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganFinalDemandV2 {
    pub key: String,
    pub retailer_site_key: String,
    pub county_geoid: String,
    pub good_key: String,
    pub ordered_quantity: u64,
}
impl MichiganFinalDemandV2 {
    #[must_use]
    pub fn order_id(&self) -> OrderIdV1 {
        OrderIdV1::from_bytes(identity("final-demand-order", &self.key))
    }
    #[must_use]
    pub fn principal_id(&self) -> FinalDemandPrincipalIdV3 {
        FinalDemandPrincipalIdV3::from_bytes(identity("final-demand-principal", &self.county_geoid))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganOwnerSourceV2 {
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
pub struct MichiganIndustryBaselineRowV1 {
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
pub struct MichiganRoadSourceV2 {
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
pub struct MichiganVehicleProfileV2 {
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
pub struct MichiganCountyTerminalV2 {
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
pub struct MichiganPhysicalEdgeV2 {
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
pub struct MichiganPhysicalCapacityGroupV2 {
    pub key: String,
    pub label: String,
    pub edge_keys: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganPhysicalNetworkV2 {
    pub source: MichiganRoadSourceV2,
    pub profile: MichiganVehicleProfileV2,
    pub terminal_source_pins: BTreeMap<String, String>,
    pub terminal_policy: MichiganTerminalPolicyV2,
    pub terminal_attachment_limit_meters: u64,
    pub terminals: Vec<MichiganCountyTerminalV2>,
    pub edges: Vec<MichiganPhysicalEdgeV2>,
    pub capacity_groups: Vec<MichiganPhysicalCapacityGroupV2>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganCapacityOverrideV2 {
    pub capacity_key: String,
    pub grams_per_period: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganOpeningStockOverrideV2 {
    pub process_key: String,
    pub good_key: String,
    pub quantity: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganRouteOverrideV2 {
    pub route_key: String,
    pub path: MichiganMaterialPathV2,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganInterventionV2 {
    pub preset: MichiganDeliveryPresetV1,
    pub capacities: Vec<MichiganCapacityOverrideV2>,
    pub opening_stocks: Vec<MichiganOpeningStockOverrideV2>,
    pub routes: Vec<MichiganRouteOverrideV2>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganNormalizedContentV2 {
    pub schema: String,
    pub evidence_class: String,
    pub horizon_ticks: u64,
    pub tick_duration_days: u64,
    pub geographic_scale: String,
    pub terminal_output_disposition: String,
    pub sites: Vec<MichiganMaterialSiteV1>,
    pub goods: Vec<MichiganMaterialGoodV1>,
    pub processes: Vec<MichiganMaterialProcessV1>,
    pub routes: Vec<MichiganMaterialRouteV1>,
    pub corridors: Vec<MichiganMaterialCorridorV1>,
    pub staffing: MichiganStaffingDesignV1,
    pub merchants: Vec<MichiganMerchantV2>,
    pub final_demands: Vec<MichiganFinalDemandV2>,
    pub owners: Vec<MichiganOwnerSourceV2>,
    pub industry: Vec<MichiganIndustryBaselineRowV1>,
    pub physical_network: Option<MichiganPhysicalNetworkV2>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MichiganTerminalPolicyV2 {
    pub terminal_evidence_class: String,
    pub anchor: String,
    pub attachment_limit_mm: u64,
    pub attachment_distance: String,
    pub usable_node: String,
    pub physical_path_distance: String,
    pub projection: String,
}
