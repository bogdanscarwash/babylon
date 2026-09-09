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

/// A materially possible buyer-supplier relation with a local transit delay.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SupplierCandidateV1 {
    pub buyer_site_id: SiteIdV1,
    pub supplier_site_id: SiteIdV1,
    pub good_id: GoodIdV1,
    pub unit_id: UnitIdV1,
    pub transit_delay_periods: u16,
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

/// Cumulative commodity-sale order quantities.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderRowV1 {
    pub order_id: OrderIdV1,
    pub access_mode: OrderAccessModeV1,
    pub buyer_site_id: SiteIdV1,
    pub supplier_site_id: SiteIdV1,
    pub good_id: GoodIdV1,
    pub unit_id: UnitIdV1,
    pub ordered: u64,
    pub shipped: u64,
    pub delivered: u64,
    pub realized: u64,
}

/// Materialized unshipped demand for one order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BacklogRowV1 {
    pub order_id: OrderIdV1,
    pub quantity: u64,
}

/// One sparse local in-transit lot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransitLotV1 {
    pub order_id: OrderIdV1,
    pub dispatch_period: u64,
    pub arrival_period: u64,
    pub source_site_id: SiteIdV1,
    pub destination_site_id: SiteIdV1,
    pub good_id: GoodIdV1,
    pub unit_id: UnitIdV1,
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

/// Complete V1 auxiliary register state at the opening of `period`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialCircuitStateV1 {
    pub period: u64,
    pub process_outputs: Vec<ProcessOutputV1>,
    pub input_coefficients: Vec<InputOutputCoefficientV1>,
    pub labor_coefficients: Vec<LaborCoefficientV1>,
    pub supplier_candidates: Vec<SupplierCandidateV1>,
    pub inventory: Vec<InventoryRowV1>,
    pub orders: Vec<OrderRowV1>,
    pub backlog: Vec<BacklogRowV1>,
    pub transit: Vec<TransitLotV1>,
    pub capacities: Vec<CapacityRowV1>,
    pub labor: Vec<LaborCapacityRowV1>,
    pub production_commitments: Vec<ProductionCommitmentV1>,
}

/// Exact V1 material-circuit refusal classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum MaterialCircuitErrorV1 {
    RowLimit = 1,
    ZeroQuantity = 2,
    DuplicateRow = 3,
    OrderInvariant = 4,
    BacklogInvariant = 5,
    TransitInvariant = 6,
    ProcessInvariant = 7,
    PeriodInvariant = 8,
    Arithmetic = 9,
    WireLimit = 10,
    WireDomain = 11,
    WireVersion = 12,
    WireTruncated = 13,
    WireTrailing = 14,
    WireEnum = 15,
    WireNoncanonical = 16,
}

/// Unknown language-neutral material-circuit refusal code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownMaterialCircuitErrorCodeV1(pub u16);

impl TryFrom<u16> for MaterialCircuitErrorV1 {
    type Error = UnknownMaterialCircuitErrorCodeV1;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::RowLimit),
            2 => Ok(Self::ZeroQuantity),
            3 => Ok(Self::DuplicateRow),
            4 => Ok(Self::OrderInvariant),
            5 => Ok(Self::BacklogInvariant),
            6 => Ok(Self::TransitInvariant),
            7 => Ok(Self::ProcessInvariant),
            8 => Ok(Self::PeriodInvariant),
            9 => Ok(Self::Arithmetic),
            10 => Ok(Self::WireLimit),
            11 => Ok(Self::WireDomain),
            12 => Ok(Self::WireVersion),
            13 => Ok(Self::WireTruncated),
            14 => Ok(Self::WireTrailing),
            15 => Ok(Self::WireEnum),
            16 => Ok(Self::WireNoncanonical),
            _ => Err(UnknownMaterialCircuitErrorCodeV1(value)),
        }
    }
}

impl From<MaterialCircuitErrorV1> for u16 {
    fn from(value: MaterialCircuitErrorV1) -> Self {
        value as Self
    }
}

/// Actual production and its planned upper bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionReceiptV1 {
    pub process_id: ProcessIdV1,
    pub site_id: SiteIdV1,
    pub planned_batches: u64,
    pub produced_batches: u64,
}

/// Inventory moved into one sparse transit lot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchReceiptV1 {
    pub order_id: OrderIdV1,
    pub quantity: u64,
    pub arrival_period: u64,
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

/// Atomic result of closing one material-circuit period.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialCircuitTransitionV1 {
    pub state: MaterialCircuitStateV1,
    pub production: Vec<ProductionReceiptV1>,
    pub dispatches: Vec<DispatchReceiptV1>,
    pub arrivals: Vec<ArrivalReceiptV1>,
    pub deliveries: Vec<DeliveryReceiptV1>,
    pub realizations: Vec<RealizationReceiptV1>,
}
