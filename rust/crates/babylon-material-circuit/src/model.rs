//! Versioned exact-quantity rows for the local material circuit.

/// Designed serialization and validation ceiling, not material abundance.
pub const MAX_MATERIAL_CIRCUIT_ROWS_V1: usize = 65_536;
/// Derived transition ceiling for disjoint input and labor resource groups.
pub const MAX_PRODUCTION_RESOURCE_GROUPS_V1: usize = MAX_MATERIAL_CIRCUIT_ROWS_V1 * 2;

macro_rules! identity_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        pub struct $name([u8; 32]);

        impl $name {
            #[must_use]
            pub const fn from_bytes(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }

            #[must_use]
            pub const fn as_bytes(self) -> [u8; 32] {
                self.0
            }
        }
    };
}

pub(crate) use identity_type;

identity_type!(SiteIdV1);
identity_type!(GoodIdV1);
identity_type!(UnitIdV1);
identity_type!(ProcessIdV1);
identity_type!(OrderIdV1);

/// One process output coefficient in exact units per batch.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessOutputV1 {
    pub process_id: ProcessIdV1,
    pub site_id: SiteIdV1,
    pub good_id: GoodIdV1,
    pub unit_id: UnitIdV1,
    pub quantity_per_batch: u64,
}

/// One Leontief material-input coefficient in exact units per batch.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InputOutputCoefficientV1 {
    pub process_id: ProcessIdV1,
    pub good_id: GoodIdV1,
    pub unit_id: UnitIdV1,
    pub quantity_per_batch: u64,
}

/// Exact labor-time required for one process batch.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LaborCoefficientV1 {
    pub process_id: ProcessIdV1,
    pub unit_id: UnitIdV1,
    pub quantity_per_batch: u64,
}

/// Exact on-hand inventory at one site.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InventoryRowV1 {
    pub site_id: SiteIdV1,
    pub good_id: GoodIdV1,
    pub unit_id: UnitIdV1,
    pub quantity: u64,
}

/// Closed V1 access mode for orders that can realize after delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum OrderAccessModeV1 {
    CommoditySale = 1,
}

/// Materialized unshipped demand for one order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BacklogRowV1 {
    pub order_id: OrderIdV1,
    pub quantity: u64,
}

/// Available process batches at one site and period.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CapacityRowV1 {
    pub process_id: ProcessIdV1,
    pub site_id: SiteIdV1,
    pub period: u64,
    pub available_batches: u64,
}

/// Available attributed labor-time at one site and period.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LaborCapacityRowV1 {
    pub site_id: SiteIdV1,
    pub unit_id: UnitIdV1,
    pub period: u64,
    pub available: u64,
}

/// A plan derived at the prior close and bounded again when executed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProductionCommitmentV1 {
    pub process_id: ProcessIdV1,
    pub site_id: SiteIdV1,
    pub period: u64,
    pub planned_batches: u64,
}

/// Private input and output rows of the shared production reducer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProductionState {
    pub period: u64,
    pub process_outputs: Vec<ProcessOutputV1>,
    pub input_coefficients: Vec<InputOutputCoefficientV1>,
    pub labor_coefficients: Vec<LaborCoefficientV1>,
    pub inventory: Vec<InventoryRowV1>,
    pub capacities: Vec<CapacityRowV1>,
    pub labor: Vec<LaborCapacityRowV1>,
    pub production_commitments: Vec<ProductionCommitmentV1>,
}

/// Actual production and its planned upper bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionReceiptV1 {
    pub process_id: ProcessIdV1,
    pub site_id: SiteIdV1,
    pub planned_batches: u64,
    pub produced_batches: u64,
}

/// One lot credited to destination inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrivalReceiptV1 {
    pub order_id: OrderIdV1,
    pub quantity: u64,
}

/// Accepted commodity-sale delivery after destination inventory is credited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryReceiptV1 {
    pub order_id: OrderIdV1,
    pub quantity: u64,
}

/// Commodity quantity realized only after its accepted arrival.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizationReceiptV1 {
    pub order_id: OrderIdV1,
    pub quantity: u64,
}
