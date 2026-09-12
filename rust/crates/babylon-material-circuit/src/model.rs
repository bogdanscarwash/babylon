//! Current production, inventory, freight and order state.

/// Designed serialization and validation ceiling, not material abundance.
pub const MAX_MATERIAL_CIRCUIT_ROWS: usize = 65_536;
/// Derived transition ceiling for disjoint input and labor resource groups.
pub const MAX_PRODUCTION_RESOURCE_GROUPS: usize = MAX_MATERIAL_CIRCUIT_ROWS * 2;

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

identity_type!(SiteId);
identity_type!(GoodId);
identity_type!(UnitId);
identity_type!(ProcessId);
identity_type!(OrderId);

/// One process output coefficient in exact units per batch.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessOutput {
    pub process_id: ProcessId,
    pub site_id: SiteId,
    pub good_id: GoodId,
    pub unit_id: UnitId,
    pub quantity_per_batch: u64,
}

/// One Leontief material-input coefficient in exact units per batch.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InputOutputCoefficient {
    pub process_id: ProcessId,
    pub good_id: GoodId,
    pub unit_id: UnitId,
    pub quantity_per_batch: u64,
}

/// Exact labor-time required for one process batch.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LaborCoefficient {
    pub process_id: ProcessId,
    pub unit_id: UnitId,
    pub quantity_per_batch: u64,
}

/// Exact on-hand inventory at one site.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InventoryRow {
    pub site_id: SiteId,
    pub good_id: GoodId,
    pub unit_id: UnitId,
    pub quantity: u64,
}

/// Closed V1 access mode for orders that can realize after delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum OrderAccessMode {
    CommoditySale = 1,
}

/// Materialized unshipped demand for one order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BacklogRow {
    pub order_id: OrderId,
    pub quantity: u64,
}

/// Available process batches at one site and period.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CapacityRow {
    pub process_id: ProcessId,
    pub site_id: SiteId,
    pub period: u64,
    pub available_batches: u64,
}

/// Available attributed labor-time at one site and period.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LaborCapacityRow {
    pub site_id: SiteId,
    pub unit_id: UnitId,
    pub period: u64,
    pub available: u64,
}

/// A plan derived at the prior close and bounded again when executed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProductionCommitment {
    pub process_id: ProcessId,
    pub site_id: SiteId,
    pub period: u64,
    pub planned_batches: u64,
}

/// Actual production and its planned upper bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionReceipt {
    pub process_id: ProcessId,
    pub site_id: SiteId,
    pub planned_batches: u64,
    pub produced_batches: u64,
}

/// One lot credited to destination inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrivalReceipt {
    pub order_id: OrderId,
    pub quantity: u64,
}

/// Accepted commodity-sale delivery after destination inventory is credited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryReceipt {
    pub order_id: OrderId,
    pub quantity: u64,
}

/// Commodity quantity realized only after its accepted arrival.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizationReceipt {
    pub order_id: OrderId,
    pub quantity: u64,
}

/// Parts-per-million denominator for exact freight loss.
pub const FREIGHT_LOSS_PARTS_PER_MILLION: u32 = 1_000_000;
identity_type!(LogisticsNodeId);
identity_type!(CorridorId);
identity_type!(RouteId);
identity_type!(FreightLotId);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SiteLogisticsNode {
    pub site_id: SiteId,
    pub node_id: LogisticsNodeId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderRow {
    pub order_id: OrderId,
    pub access_mode: OrderAccessMode,
    pub buyer_site_id: SiteId,
    pub supplier_site_id: SiteId,
    pub good_id: GoodId,
    pub unit_id: UnitId,
    pub ordered: u64,
    pub shipped: u64,
    pub lost: u64,
    pub delivered: u64,
    pub realized: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutedDispatchReceipt {
    pub lot_id: FreightLotId,
    pub order_id: OrderId,
    pub route_id: RouteId,
    pub quantity: u64,
    pub final_arrival_period: u64,
}

/// Bound on timed stages, independently of physical path length.
pub const MAX_ROUTE_STAGES_PER_ROUTE: usize = 16;
/// Designed bound on total order/resource requests during one freight close.
pub const MAX_FREIGHT_RESOURCE_REQUESTS: usize = 1_114_112;

/// Transport attached to an internal supply relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum SupplierTransport {
    Local = 1,
    Staged = 2,
}

/// One exact supplier relationship; local circulation creates no freight lot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SupplierRoute {
    pub buyer_site_id: SiteId,
    pub supplier_site_id: SiteId,
    pub good_id: GoodId,
    pub unit_id: UnitId,
    pub route_id: RouteId,
    pub transport_kind: SupplierTransport,
}

/// Positive inter-owner local transfer; distinct from physical freight arrival.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTransferReceipt {
    pub order_id: OrderId,
    pub supplier_site_id: SiteId,
    pub buyer_site_id: SiteId,
    pub good_id: GoodId,
    pub unit_id: UnitId,
    pub quantity: u64,
}

/// Positive exact grams represented by one native shipment unit.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FreightMassCoefficient {
    pub good_id: GoodId,
    pub unit_id: UnitId,
    pub grams_per_unit: u64,
}
/// One timed transport stage; physical geometry does not advance this clock.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RouteStage {
    pub route_id: RouteId,
    pub stage_index: u16,
    pub from_node_id: LogisticsNodeId,
    pub to_node_id: LogisticsNodeId,
    pub travel_periods: u16,
    pub loss_ppm: u32,
}
/// One nonduplicated capacity membership of a transport stage.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RouteStageCapacity {
    pub route_id: RouteId,
    pub stage_index: u16,
    pub corridor_id: CorridorId,
}
/// Unreserved grams for one shared principal and departure period.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CorridorCapacity {
    pub corridor_id: CorridorId,
    pub period: u64,
    pub available_grams: u64,
}
/// Native goods in transit through one timed stage.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RoutedFreightLot {
    pub lot_id: FreightLotId,
    pub order_id: OrderId,
    pub route_id: RouteId,
    pub dispatch_period: u64,
    pub current_stage_index: u16,
    pub stage_arrival_period: u64,
    pub source_site_id: SiteId,
    pub destination_site_id: SiteId,
    pub good_id: GoodId,
    pub unit_id: UnitId,
    pub quantity: u64,
}
identity_type!(FinalDemandPrincipalId);

/// Staffed circulation role; neither role transforms its goods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum MerchantRole {
    Wholesale = 1,
    Retail = 2,
}

/// One outbound handling and labor principal for a merchant site.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MerchantHandling {
    pub site_id: SiteId,
    pub county_geoid: [u8; 5],
    pub role: MerchantRole,
    pub capacity_id: CorridorId,
    pub labor_unit_id: UnitId,
}

/// Exact positive outbound handling hours for one native unit.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MerchantHandlingCoefficient {
    pub site_id: SiteId,
    pub good_id: GoodId,
    pub unit_id: UnitId,
    pub hours_per_unit: u64,
}

/// County-local end-buyer account identity, without a household or inventory.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FinalDemandPrincipal {
    pub id: FinalDemandPrincipalId,
    pub county_geoid: [u8; 5],
}

/// Finite native-good handoff order; fulfillment does not assert consumption.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FinalDemandOrder {
    pub order_id: OrderId,
    pub retailer_site_id: SiteId,
    pub demand_principal_id: FinalDemandPrincipalId,
    pub good_id: GoodId,
    pub unit_id: UnitId,
    pub ordered: u64,
    pub fulfilled: u64,
}

/// Disjoint identities for routed shipment and local retail handoff requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutboundOrderId {
    Delivery(OrderId),
    LocalFinalDemand(OrderId),
}

/// Completed handling need before labor and actual outbound handling after labor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerchantHandlingReceipt {
    pub site_id: SiteId,
    pub order: OutboundOrderId,
    pub feasible_quantity: u64,
    pub handled_quantity: u64,
    pub needed_hours: u64,
    pub used_hours: u64,
}

/// Positive local handoff, credited once without freight or a journey.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalRetailFulfillmentReceipt {
    pub order_id: OrderId,
    pub retailer_site_id: SiteId,
    pub demand_principal_id: FinalDemandPrincipalId,
    pub good_id: GoodId,
    pub unit_id: UnitId,
    pub quantity: u64,
}

/// Complete opening state; stock and recipe quantities retain their native units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialCircuitState {
    pub period: u64,
    pub site_logistics_nodes: Vec<SiteLogisticsNode>,
    pub process_outputs: Vec<ProcessOutput>,
    pub input_coefficients: Vec<InputOutputCoefficient>,
    pub labor_coefficients: Vec<LaborCoefficient>,
    pub freight_mass_coefficients: Vec<FreightMassCoefficient>,
    pub supplier_routes: Vec<SupplierRoute>,
    pub route_stages: Vec<RouteStage>,
    pub route_stage_capacities: Vec<RouteStageCapacity>,
    pub inventory: Vec<InventoryRow>,
    pub orders: Vec<OrderRow>,
    pub backlog: Vec<BacklogRow>,
    pub freight: Vec<RoutedFreightLot>,
    pub corridor_capacities: Vec<CorridorCapacity>,
    pub capacities: Vec<CapacityRow>,
    pub labor: Vec<LaborCapacityRow>,
    pub production_commitments: Vec<ProductionCommitment>,
    pub merchants: Vec<MerchantHandling>,
    pub handling_coefficients: Vec<MerchantHandlingCoefficient>,
    pub final_demand_principals: Vec<FinalDemandPrincipal>,
    pub final_demand_orders: Vec<FinalDemandOrder>,
}
/// Exact V3 mass-weighted freight refusal classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum MaterialCircuitError {
    RowLimit = 1,
    ZeroQuantity = 2,
    DuplicateRow = 3,
    OrderInvariant = 4,
    BacklogInvariant = 5,
    FreightInvariant = 6,
    ProcessInvariant = 7,
    PeriodInvariant = 8,
    Arithmetic = 9,
    RouteInvariant = 10,
    CapacityInvariant = 11,
    WireLimit = 12,
    WireDomain = 13,
    WireVersion = 14,
    WireTruncated = 15,
    WireTrailing = 16,
    WireEnum = 17,
    WireNoncanonical = 18,
    MassInvariant = 19,
    MerchantInvariant = 20,
    FinalDemandInvariant = 21,
}

/// Unknown language-neutral routed-material refusal code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownMaterialCircuitErrorCode(pub u16);

impl TryFrom<u16> for MaterialCircuitError {
    type Error = UnknownMaterialCircuitErrorCode;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::RowLimit),
            2 => Ok(Self::ZeroQuantity),
            3 => Ok(Self::DuplicateRow),
            4 => Ok(Self::OrderInvariant),
            5 => Ok(Self::BacklogInvariant),
            6 => Ok(Self::FreightInvariant),
            7 => Ok(Self::ProcessInvariant),
            8 => Ok(Self::PeriodInvariant),
            9 => Ok(Self::Arithmetic),
            10 => Ok(Self::RouteInvariant),
            11 => Ok(Self::CapacityInvariant),
            12 => Ok(Self::WireLimit),
            13 => Ok(Self::WireDomain),
            14 => Ok(Self::WireVersion),
            15 => Ok(Self::WireTruncated),
            16 => Ok(Self::WireTrailing),
            17 => Ok(Self::WireEnum),
            18 => Ok(Self::WireNoncanonical),
            19 => Ok(Self::MassInvariant),
            20 => Ok(Self::MerchantInvariant),
            21 => Ok(Self::FinalDemandInvariant),
            _ => Err(UnknownMaterialCircuitErrorCode(value)),
        }
    }
}

impl From<MaterialCircuitError> for u16 {
    fn from(value: MaterialCircuitError) -> Self {
        value as Self
    }
}

/// Loss attributed to a timed stage, independently of its capacity memberships.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreightLossReceipt {
    pub lot_id: FreightLotId,
    pub order_id: OrderId,
    pub route_id: RouteId,
    pub stage_index: u16,
    pub quantity: u64,
}
/// Atomic successor with native quantity receipts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialCircuitTransition {
    pub state: MaterialCircuitState,
    pub production: Vec<ProductionReceipt>,
    pub dispatches: Vec<RoutedDispatchReceipt>,
    pub losses: Vec<FreightLossReceipt>,
    pub arrivals: Vec<crate::ArrivalReceipt>,
    pub deliveries: Vec<crate::DeliveryReceipt>,
    pub realizations: Vec<crate::RealizationReceipt>,
    pub handling: Vec<MerchantHandlingReceipt>,
    pub local_fulfillments: Vec<LocalRetailFulfillmentReceipt>,
    pub local_transfers: Vec<LocalTransferReceipt>,
}
