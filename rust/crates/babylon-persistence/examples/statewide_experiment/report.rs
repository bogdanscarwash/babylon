use babylon_persistence::michigan_material::{
    MichiganMaterialPath, MichiganRoadSource, MichiganSiteRole,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
pub struct Candidate {
    pub capacity_key: String,
    pub food_process: String,
    pub constrained_grams: u64,
    pub shortage_opening: u64,
}
#[derive(Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub evidence_scope: &'static str,
    pub qualified: bool,
    pub persisted: bool,
    pub parameter_evidence: &'static str,
    pub outcome_evidence: &'static str,
    pub candidate: Candidate,
    pub baseline_capacity_grams: u64,
    pub baseline_packaging_opening: u64,
    pub input_sha256: BTreeMap<&'static str, String>,
    pub qualification_source_pins: QualificationSourcePins,
    pub road_source: MichiganRoadSource,
    pub terminal_source_pins: BTreeMap<String, String>,
    pub owners: BTreeMap<String, OwnerIdentity>,
    pub processes: BTreeMap<String, ProcessIdentity>,
    pub routes: BTreeMap<String, RouteIdentity>,
    pub cases: BTreeMap<&'static str, Case>,
    pub witnesses: Witnesses,
}
#[derive(Serialize)]
pub struct OwnerIdentity {
    pub county_geoid: String,
    pub sector_code: String,
    pub role: MichiganSiteRole,
}
#[derive(Serialize)]
pub struct ProcessIdentity {
    pub owner: String,
    pub output_good: String,
    pub output_unit: String,
}
#[derive(Serialize)]
pub struct RouteIdentity {
    pub supplier: String,
    pub buyer: String,
    pub good: String,
    pub unit: String,
    pub grams_per_unit: u64,
    pub path: MichiganMaterialPath,
}
#[derive(Serialize)]
pub struct Case {
    pub preset: &'static str,
    pub captured_content_sha256: String,
    pub foundation_sha256: String,
    pub captured_bytes: usize,
    pub foundation_bytes: usize,
    pub compile_ms: u128,
    pub advance_ms: u128,
    pub periods: Vec<CompletedPeriod>,
}
#[derive(Serialize)]
pub struct CompletedPeriod {
    pub period: u64,
    pub tick_content_sha256: String,
    pub prior_world_sha256: String,
    pub world_sha256: String,
    pub material_state_sha256: String,
    pub material_receipts_sha256: String,
    pub staffing_events_sha256: String,
    pub register_bytes: usize,
    pub receipt_bytes: usize,
    pub maximum_family_rows: usize,
    pub selected_capacity_opening_grams: u64,
    pub selected_capacity_reserved_grams: u64,
    pub processes: BTreeMap<String, Production>,
    pub owners: BTreeMap<String, Owner>,
    pub routes: BTreeMap<String, Route>,
    pub final_demand: BTreeMap<String, FinalDemand>,
}
#[derive(Serialize)]
pub struct NativeQuantity {
    pub unit: String,
    pub quantity: u64,
}
#[derive(Serialize)]
pub struct Production {
    pub planned_batches: u64,
    pub produced_batches: u64,
    pub output_units: u64,
    pub consumed_inputs: BTreeMap<String, NativeQuantity>,
    pub used_labor_hours: u64,
}
#[derive(Serialize)]
pub struct Owner {
    pub staffing: BTreeMap<String, u64>,
    pub closing_inventory: BTreeMap<String, NativeQuantity>,
    pub handling_needed_hours: u64,
    pub handling_used_hours: u64,
}
#[derive(Serialize)]
pub struct Route {
    pub dispatched: u64,
    pub arrived: u64,
    pub local_transferred: u64,
    pub delivered: u64,
    pub cumulative_shipped: u64,
    pub cumulative_delivered: u64,
    pub cumulative_realized: u64,
    pub cumulative_lost: u64,
    pub outstanding: u64,
    pub in_transit: u64,
}
#[derive(Serialize)]
pub struct FinalDemand {
    pub retailer: String,
    pub county_geoid: String,
    pub good: String,
    pub unit: String,
    pub ordered: u64,
    pub fulfilled_this_period: u64,
    pub cumulative_fulfilled: u64,
    pub outstanding: u64,
}
#[derive(Default, Serialize)]
pub struct Witnesses {
    pub freight: Option<FreightWitness>,
    pub packaging: Option<OutputDifference>,
    pub missing: Vec<&'static str>,
}
#[derive(Serialize)]
pub struct FreightWitness {
    pub dispatch_period: u64,
    pub capacity_key: String,
    pub baseline_reserved_grams: u64,
    pub constrained_reserved_grams: u64,
    pub changed_routes: Vec<RouteDifference>,
    pub downstream_output: OutputDifference,
    pub downstream_workforce: WorkforceDifference,
}
#[derive(Serialize)]
pub struct RouteDifference {
    pub route: String,
    pub baseline_dispatched: u64,
    pub constrained_dispatched: u64,
}
#[derive(Serialize)]
pub struct OutputDifference {
    pub period: u64,
    pub process: String,
    pub owner: String,
    pub baseline_output_units: u64,
    pub constrained_output_units: u64,
    pub supply_chain_routes: Vec<String>,
}
#[derive(Serialize)]
pub struct WorkforceDifference {
    pub period: u64,
    pub owner: String,
    pub baseline_employed: u64,
    pub constrained_employed: u64,
    pub baseline_reserve: u64,
    pub constrained_reserve: u64,
    pub supply_chain_routes: Vec<String>,
}

/// Select only the pins from the compiler-validated full qualification input.
#[derive(Deserialize, Serialize)]
pub struct QualificationSourcePins {
    #[serde(rename = "defines_sha256")]
    pub defines: String,
    #[serde(rename = "roster_sha256")]
    pub roster: String,
    #[serde(rename = "paths_sha256")]
    pub paths: String,
}
