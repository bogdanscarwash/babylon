//! Read-only presentation rows derived from a committed material envelope.
//! These rows report exact stocks and receipts; they never adjudicate a tick.

use serde::{Deserialize, Serialize};

/// One complete role-scoped view of the committed circuit.
/// Row collections are unordered multisets; duplicate rows remain significant.
/// Event sequence is meaningful, while each event's subject list is unordered.
/// The enclosing observation supplies the scope for
/// [`crate::ObserverEconomySnapshotV1::production_evidence_digest`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionSnapshotV2 {
    pub scenario_label: String,
    pub horizon_period: u64,
    pub content_authority_sha256: String,
    pub physical_edges: Vec<ProductionPhysicalEdgeV2>,
    pub road_source: Option<ProductionRoadSourceV2>,
    pub sites: Vec<ProductionSiteV2>,
    pub routes: Vec<ProductionRouteV2>,
    pub freight: Vec<ProductionFreightV2>,
    /// Each mass-capacity principal is disclosed once, with distinct reservation periods.
    pub freight_capacity_accounts: Vec<ProductionFreightCapacityAccountV2>,
    pub events: Vec<ProductionEventV1>,
    pub merchant_handling_accounts: Vec<ProductionMerchantHandlingAccountV2>,
    pub final_demand_accounts: Vec<ProductionFinalDemandAccountV2>,
    /// Each exact site/unit labor principal occurs once, across all its processes.
    pub labor_accounts: Vec<ProductionLaborAccountV2>,
    /// Exact graph-owned modeled people and retained work requests at this scope.
    /// A missing account is not an observed zero; foundation has no completed event.
    pub staffing_accounts: Vec<ProductionStaffingAccountV1>,
    /// Exact completed-period stock accounting; absent at foundation.
    pub material_balance: Option<crate::CompletedMaterialBalanceV2>,
    /// Deduplicated public 2024 source cells, never current modeled employment.
    pub observed_contexts: Vec<ObservedSectorContextV2>,
    /// Designed attribution only; these are not supplier or employment relations.
    pub process_attributions: Vec<DesignedProcessAttributionV1>,
    /// Declared assumptions and source artifact identifiers.
    pub provenance: Vec<String>,
}

/// The exact authored BUSINESS node in an admitted cohort foundation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionBusinessSubjectV1 {
    pub scenario: String,
    pub local_name: String,
}

/// Observed private-industry county manufacturing totals from one source cell.
/// Metrics have their QCEW units: establishments, annual-average jobs, USD annual
/// payroll, and USD weekly mean wage. No metric allocates people to a process.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedSectorContextV2 {
    pub subject: ProductionBusinessSubjectV1,
    pub county_geoid: String,
    pub sector_code: String,
    pub sector_title: String,
    pub vintage: u16,
    pub annual_avg_estabs_count: u64,
    pub annual_avg_emplvl: Option<u64>,
    pub total_annual_wages: Option<u64>,
    pub annual_avg_wkly_wage: Option<u64>,
    pub source_url: String,
    pub source_file: String,
    pub source_sha256: String,
    pub artifact_sha256: String,
    pub evidence_class: crate::ArchiveEvidenceClassV1,
}

/// A Designed process is set in this observed sector context. The link assigns
/// no workers, ownership, factory coordinates, market share, or physical output.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesignedProcessAttributionV1 {
    pub process_id: String,
    pub site_id: String,
    pub industry_code: String,
    pub cohort_subject: ProductionBusinessSubjectV1,
    pub scenario_artifact_sha256: String,
    pub industry_artifact_sha256: String,
    pub evidence_class: crate::ArchiveEvidenceClassV1,
}

/// One aggregate county-sector owner, never a factory coordinate.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionSiteV2 {
    pub id: String,
    pub county_geoid: String,
    pub name: String,
    pub industry_code: String,
    pub observed_employment: Option<u64>,
    pub role: ProductionSiteRoleV2,
    pub sector_code: String,
    pub processes: Vec<ProductionProcessV2>,
    pub inventory: Vec<ProductionStockV1>,
}

/// An owner role does not imply a fabricated productive process.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProductionSiteRoleV2 {
    Production,
    Wholesale,
    Retail,
}

/// One process within an owner; inventory and workforce belong to its site.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionProcessV2 {
    pub id: String,
    pub name: String,
    /// Exact material identity; labels never serve as aggregation keys.
    pub output_good_id: String,
    pub output_unit_id: String,
    pub output_good: String,
    pub output_unit: String,
    pub output_per_batch: u64,
    /// Capacity at the next opening period, in exact process batches.
    pub available_batches: u64,
    /// Last committed production-family reading; absent at foundation.
    /// Omitted zero commitments in a complete family read as zero without
    /// inventing a producer receipt or event.
    pub planned_batches: Option<u64>,
    pub produced_batches: Option<u64>,
    pub inputs: Vec<ProductionInputV1>,
    pub labor: Vec<ProductionLaborV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionStockV1 {
    pub good_id: String,
    pub unit_id: String,
    pub good: String,
    pub unit: String,
    pub quantity: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionInputV1 {
    pub good_id: String,
    pub unit_id: String,
    pub good: String,
    pub unit: String,
    pub quantity_per_batch: u64,
    pub on_hand: u64,
    pub supplier_site_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionLaborV1 {
    pub unit: String,
    pub available: u64,
    pub quantity_per_batch: u64,
}

/// Exact time accounting, distinct from employment, headcount, or paid wages.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionLaborAccountV2 {
    pub site_id: String,
    pub unit_id: String,
    pub unit: String,
    pub next_opening_period: u64,
    pub next_opening_available: u64,
    /// Absent at foundation; unused time expires within its completed period.
    pub completed: Option<CompletedProductionLaborV2>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletedProductionLaborV2 {
    pub period: u64,
    pub opening: u64,
    pub planned: u64,
    pub used: u64,
    pub unused: u64,
    pub handling_needed: u64,
    pub handling_used: u64,
}

/// Stable `SOCIAL_CLASS` subject of an admitted Designed workforce pool.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionStaffingSubjectV1 {
    pub scenario: String,
    pub local_name: String,
}

/// Modeled population stocks at the selected committed period, separate from QCEW jobs.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionStaffingAccountV1 {
    pub pool_id: String,
    pub site_id: String,
    pub unit_id: String,
    pub subject: ProductionStaffingSubjectV1,
    pub hours_per_person: u64,
    pub labor_force: u64,
    pub employed: u64,
    pub reserve: u64,
    pub previous_unretained_hours: u64,
    pub next_opening_period: u64,
    pub next_opening_hours: u64,
    /// Absent at foundation; present only with exact committed staffing evidence.
    pub completed: Option<CompletedProductionStaffingV1>,
}

/// Completed staffing decision. Closing E/R stocks belong to the enclosing account.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletedProductionStaffingV1 {
    pub period: u64,
    pub opening_employed: u64,
    pub opening_reserve: u64,
    pub previous_unretained_hours: u64,
    pub current_unretained_hours: u64,
    pub retained_hours: u64,
    pub target_employed: u64,
    pub hires: u64,
    pub separations: u64,
}

/// A real supplier relation with its declared physical route and order account.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionRouteV2 {
    pub id: String,
    pub supplier_site_id: String,
    pub buyer_site_id: String,
    pub good_id: String,
    pub unit_id: String,
    pub good: String,
    pub unit: String,
    pub travel_periods: u64,
    /// Timed stages identify shared capacities; geometry edges do not add time.
    pub stages: Vec<ProductionRouteStageV2>,
    pub transport_kind: ProductionRouteTransportV2,
    pub physical_edge_ids: Vec<String>,
    pub distance_mm: Option<u64>,
    pub grams_per_unit: u64,
    pub ordered: u64,
    pub shipped: u64,
    pub delivered: u64,
    pub lost: u64,
    pub realized: u64,
    pub backlog: u64,
}

/// One packet on screen corresponds to one actual in-transit freight lot.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionFreightV2 {
    pub id: String,
    pub route_id: String,
    pub source_site_id: String,
    pub destination_site_id: String,
    pub good_id: String,
    pub unit_id: String,
    pub good: String,
    pub unit: String,
    pub quantity: u64,
    pub dispatch_period: u64,
    pub arrival_period: u64,
    pub current_stage_index: u16,
    pub grams_per_unit: u64,
    pub mass_grams: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionEventV1 {
    pub id: String,
    pub period: u64,
    pub subject_site_ids: Vec<String>,
    pub kind: String,
    pub description: String,
    pub receipt_digest: String,
    /// Typed receipt metadata, never inferred from the event's description.
    pub delivery_evidence: Option<ProductionDeliveryEvidenceV1>,
}

/// Three distinct receipt stages for delivered material, never payment evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProductionDeliveryStageV1 {
    Arrival,
    Delivery,
    QuantityRealization,
}

/// Exact order, route and material identities for one original receipt row.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionDeliveryEvidenceV1 {
    pub stage: ProductionDeliveryStageV1,
    pub order_id: String,
    pub route_id: String,
    pub good_id: String,
    pub unit_id: String,
    pub quantity: u64,
}

/// One timed stage with nonduplicated capacity memberships.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionRouteStageV2 {
    pub stage_index: u16,
    pub capacity_ids: Vec<String>,
    pub travel_periods: u64,
}

/// Unreserved grams, shared across participating routes or merchant outbound work.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionFreightCapacityAccountV2 {
    pub corridor_id: String,
    pub corridor_label: String,
    pub kind: ProductionCapacityKindV2,
    pub merchant_site_ids: Vec<String>,
    pub route_ids: Vec<String>,
    pub next_opening_period: u64,
    pub next_opening_available_grams: u64,
    /// Absent at foundation. Completed zero reservations remain explicit.
    pub completed: Option<CompletedProductionFreightCapacityV2>,
}

/// Reservations made by the latest completed dispatch family, not arrivals.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletedProductionFreightCapacityV2 {
    pub period: u64,
    /// Each reservation departure period occurs once within its mass principal.
    pub reservations: Vec<ProductionFreightReservationV2>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionFreightReservationV2 {
    /// A later route leg reserves future capacity during the completed tick.
    pub reservation_period: u64,
    pub opening_available_grams: u64,
    pub newly_reserved_grams: u64,
    pub remaining_available_grams: u64,
    pub orders: Vec<ProductionFreightCapacityOrderV2>,
}

/// One order's opening request and actual committed dispatch for a reservation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionFreightCapacityOrderV2 {
    pub order_id: String,
    pub route_id: Option<String>,
    pub kind: ProductionOutboundKindV2,
    pub supplier_site_id: String,
    pub good_id: String,
    pub unit_id: String,
    pub requested: u64,
    pub dispatched: u64,
    /// Ordered minus shipped, distinct from the route's undelivered backlog.
    pub remaining_unshipped: u64,
    pub grams_per_unit: u64,
    pub requested_grams: u128,
    pub reserved_grams: u64,
}

/// Shared resource accounting distinguishes transport from merchant handling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProductionCapacityKindV2 {
    Transport,
    MerchantHandling,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProductionOutboundKindV2 {
    Delivery,
    LocalFinalDemand,
}

/// Coefficients and last completed handling work for one merchant owner.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionMerchantHandlingAccountV2 {
    pub site_id: String,
    pub capacity_id: String,
    pub labor_unit_id: String,
    pub coefficients: Vec<ProductionHandlingCoefficientV2>,
    pub completed: Option<CompletedProductionMerchantHandlingV2>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionHandlingCoefficientV2 {
    pub good_id: String,
    pub unit_id: String,
    pub grams_per_unit: u64,
    pub hours_per_unit: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletedProductionMerchantHandlingV2 {
    pub period: u64,
    pub needed_hours: u64,
    pub used_hours: u64,
    pub handled_grams: u64,
    pub orders: Vec<ProductionMerchantHandlingOrderV2>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionMerchantHandlingOrderV2 {
    pub order_id: String,
    pub kind: ProductionOutboundKindV2,
    pub good_id: String,
    pub unit_id: String,
    pub requested: u64,
    pub feasible_quantity: u64,
    pub handled_quantity: u64,
    pub needed_hours: u64,
    pub used_hours: u64,
    pub remaining_unshipped: u64,
}

/// One county and native-good account; delivery to end buyers is not consumption.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionFinalDemandAccountV2 {
    pub demand_principal_id: String,
    pub county_geoid: String,
    pub good_id: String,
    pub unit_id: String,
    pub good: String,
    pub unit: String,
    pub ordered: u64,
    pub fulfilled: u64,
    pub outstanding: u64,
    pub retail_stock_on_hand: u64,
    pub retailer_site_ids: Vec<String>,
    pub orders: Vec<ProductionFinalDemandOrderV2>,
    pub completed: Option<CompletedProductionFinalDemandV2>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionFinalDemandOrderV2 {
    pub order_id: String,
    pub retailer_site_id: String,
    pub ordered: u64,
    pub fulfilled: u64,
    pub outstanding: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletedProductionFinalDemandV2 {
    pub period: u64,
    pub opening_fulfilled: u64,
    pub newly_fulfilled: u64,
    pub closing_fulfilled: u64,
}

/// Local internal transfers have no timed stage or physical journey.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProductionRouteTransportV2 {
    Local,
    Staged,
}

/// Captured road geometry, disclosed once and never fetched by the observer.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionPhysicalEdgeV2 {
    pub id: String,
    pub shape_e7: Vec<[i64; 2]>,
    pub distance_mm: u64,
}

/// Exact captured source identity of the qualified physical network.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionRoadSourceV2 {
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
