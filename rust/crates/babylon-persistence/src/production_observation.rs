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
pub struct ProductionSnapshotV1 {
    pub scenario_label: String,
    pub horizon_week: u64,
    pub sites: Vec<ProductionSiteV1>,
    pub routes: Vec<ProductionRouteV1>,
    pub freight: Vec<ProductionFreightV1>,
    pub events: Vec<ProductionEventV1>,
    /// Each exact site/unit labor principal occurs once, across all its processes.
    pub labor_accounts: Vec<ProductionLaborAccountV1>,
    /// Exact completed-week stock accounting; absent at foundation.
    pub material_balance: Option<crate::CompletedMaterialBalanceV1>,
    /// Deduplicated public 2024 source cells, never current modeled employment.
    pub observed_contexts: Vec<ObservedManufacturingContextV1>,
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
pub struct ObservedManufacturingContextV1 {
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

/// An aggregate cohort at county resolution, never a factory coordinate.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionSiteV1 {
    pub id: String,
    pub county_geoid: String,
    pub name: String,
    pub industry_code: String,
    pub observed_employment: Option<u64>,
    /// Exact material identity; labels never serve as aggregation keys.
    pub output_good_id: String,
    pub output_unit_id: String,
    pub output_good: String,
    pub output_unit: String,
    pub output_per_batch: u64,
    /// Capacity at the next opening week, in exact process batches.
    pub available_batches: u64,
    /// Last committed production-family reading; absent at foundation.
    /// Omitted zero commitments in a complete family read as zero without
    /// inventing a producer receipt or event.
    pub planned_batches: Option<u64>,
    pub produced_batches: Option<u64>,
    pub inventory: Vec<ProductionStockV1>,
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
pub struct ProductionLaborAccountV1 {
    pub site_id: String,
    pub unit_id: String,
    pub unit: String,
    pub next_opening_week: u64,
    pub next_opening_available: u64,
    /// Absent at foundation; unused time expires within its completed week.
    pub completed: Option<CompletedProductionLaborV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletedProductionLaborV1 {
    pub week: u64,
    pub opening: u64,
    pub planned: u64,
    pub used: u64,
    pub unused: u64,
}

/// A real supplier relation with its declared physical route and order account.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionRouteV1 {
    pub id: String,
    pub supplier_site_id: String,
    pub buyer_site_id: String,
    pub good_id: String,
    pub unit_id: String,
    pub good: String,
    pub unit: String,
    pub travel_weeks: u64,
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
pub struct ProductionFreightV1 {
    pub id: String,
    pub route_id: String,
    pub source_site_id: String,
    pub destination_site_id: String,
    pub good_id: String,
    pub unit_id: String,
    pub good: String,
    pub unit: String,
    pub quantity: u64,
    pub dispatch_week: u64,
    pub arrival_week: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionEventV1 {
    pub id: String,
    pub week: u64,
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
