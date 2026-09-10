//! Current production, inventory, freight and order state.

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

/// Parts-per-million denominator for exact freight loss.
pub const FREIGHT_LOSS_PARTS_PER_MILLION_V2: u32 = 1_000_000;
identity_type!(LogisticsNodeIdV2);
identity_type!(CorridorIdV2);
identity_type!(RouteIdV2);
identity_type!(FreightLotIdV2);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SiteLogisticsNodeV2 {
    pub site_id: SiteIdV1,
    pub node_id: LogisticsNodeIdV2,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderRowV2 {
    pub order_id: OrderIdV1,
    pub access_mode: OrderAccessModeV1,
    pub buyer_site_id: SiteIdV1,
    pub supplier_site_id: SiteIdV1,
    pub good_id: GoodIdV1,
    pub unit_id: UnitIdV1,
    pub ordered: u64,
    pub shipped: u64,
    pub lost: u64,
    pub delivered: u64,
    pub realized: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutedDispatchReceiptV2 {
    pub lot_id: FreightLotIdV2,
    pub order_id: OrderIdV1,
    pub route_id: RouteIdV2,
    pub quantity: u64,
    pub final_arrival_period: u64,
}

/// Bound on timed stages, independently of physical path length.
pub const MAX_ROUTE_STAGES_PER_ROUTE_V3: usize = 16;
/// Designed bound on total order/resource requests during one freight close.
pub const MAX_FREIGHT_RESOURCE_REQUESTS_V3: usize = 1_114_112;

/// Transport attached to an internal supply relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum SupplierTransportV3 {
    Local = 1,
    Staged = 2,
}

/// One exact supplier relationship; local circulation creates no freight lot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SupplierRouteV3 {
    pub buyer_site_id: SiteIdV1,
    pub supplier_site_id: SiteIdV1,
    pub good_id: GoodIdV1,
    pub unit_id: UnitIdV1,
    pub route_id: RouteIdV2,
    pub transport_kind: SupplierTransportV3,
}

/// Positive inter-owner local transfer; distinct from physical freight arrival.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTransferReceiptV3 {
    pub order_id: OrderIdV1,
    pub supplier_site_id: SiteIdV1,
    pub buyer_site_id: SiteIdV1,
    pub good_id: GoodIdV1,
    pub unit_id: UnitIdV1,
    pub quantity: u64,
}

/// Positive exact grams represented by one native shipment unit.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FreightMassCoefficientV3 {
    pub good_id: GoodIdV1,
    pub unit_id: UnitIdV1,
    pub grams_per_unit: u64,
}
/// One timed transport stage; physical geometry does not advance this clock.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RouteStageV3 {
    pub route_id: RouteIdV2,
    pub stage_index: u16,
    pub from_node_id: LogisticsNodeIdV2,
    pub to_node_id: LogisticsNodeIdV2,
    pub travel_periods: u16,
    pub loss_ppm: u32,
}
/// One nonduplicated capacity membership of a transport stage.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RouteStageCapacityV3 {
    pub route_id: RouteIdV2,
    pub stage_index: u16,
    pub corridor_id: CorridorIdV2,
}
/// Unreserved grams for one shared principal and departure period.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CorridorCapacityV3 {
    pub corridor_id: CorridorIdV2,
    pub period: u64,
    pub available_grams: u64,
}
/// Native goods in transit through one timed stage.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RoutedFreightLotV3 {
    pub lot_id: FreightLotIdV2,
    pub order_id: OrderIdV1,
    pub route_id: RouteIdV2,
    pub dispatch_period: u64,
    pub current_stage_index: u16,
    pub stage_arrival_period: u64,
    pub source_site_id: SiteIdV1,
    pub destination_site_id: SiteIdV1,
    pub good_id: GoodIdV1,
    pub unit_id: UnitIdV1,
    pub quantity: u64,
}
identity_type!(FinalDemandPrincipalIdV3);

/// Staffed circulation role; neither role transforms its goods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum MerchantRoleV3 {
    Wholesale = 1,
    Retail = 2,
}

/// One outbound handling and labor principal for a merchant site.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MerchantHandlingV3 {
    pub site_id: SiteIdV1,
    pub county_geoid: [u8; 5],
    pub role: MerchantRoleV3,
    pub capacity_id: CorridorIdV2,
    pub labor_unit_id: UnitIdV1,
}

/// Exact positive outbound handling hours for one native unit.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MerchantHandlingCoefficientV3 {
    pub site_id: SiteIdV1,
    pub good_id: GoodIdV1,
    pub unit_id: UnitIdV1,
    pub hours_per_unit: u64,
}

/// County-local end-buyer account identity, without a household or inventory.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FinalDemandPrincipalV3 {
    pub id: FinalDemandPrincipalIdV3,
    pub county_geoid: [u8; 5],
}

/// Finite native-good handoff order; fulfillment does not assert consumption.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FinalDemandOrderV3 {
    pub order_id: OrderIdV1,
    pub retailer_site_id: SiteIdV1,
    pub demand_principal_id: FinalDemandPrincipalIdV3,
    pub good_id: GoodIdV1,
    pub unit_id: UnitIdV1,
    pub ordered: u64,
    pub fulfilled: u64,
}

/// Disjoint identities for routed shipment and local retail handoff requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutboundOrderIdV3 {
    Delivery(OrderIdV1),
    LocalFinalDemand(OrderIdV1),
}

/// Completed handling need before labor and actual outbound handling after labor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerchantHandlingReceiptV3 {
    pub site_id: SiteIdV1,
    pub order: OutboundOrderIdV3,
    pub feasible_quantity: u64,
    pub handled_quantity: u64,
    pub needed_hours: u64,
    pub used_hours: u64,
}

/// Positive local handoff, credited once without freight or a journey.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalRetailFulfillmentReceiptV3 {
    pub order_id: OrderIdV1,
    pub retailer_site_id: SiteIdV1,
    pub demand_principal_id: FinalDemandPrincipalIdV3,
    pub good_id: GoodIdV1,
    pub unit_id: UnitIdV1,
    pub quantity: u64,
}

/// Complete opening state; stock and recipe quantities retain their native units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialCircuitStateV3 {
    pub period: u64,
    pub site_logistics_nodes: Vec<SiteLogisticsNodeV2>,
    pub process_outputs: Vec<ProcessOutputV1>,
    pub input_coefficients: Vec<InputOutputCoefficientV1>,
    pub labor_coefficients: Vec<LaborCoefficientV1>,
    pub freight_mass_coefficients: Vec<FreightMassCoefficientV3>,
    pub supplier_routes: Vec<SupplierRouteV3>,
    pub route_stages: Vec<RouteStageV3>,
    pub route_stage_capacities: Vec<RouteStageCapacityV3>,
    pub inventory: Vec<InventoryRowV1>,
    pub orders: Vec<OrderRowV2>,
    pub backlog: Vec<BacklogRowV1>,
    pub freight: Vec<RoutedFreightLotV3>,
    pub corridor_capacities: Vec<CorridorCapacityV3>,
    pub capacities: Vec<CapacityRowV1>,
    pub labor: Vec<LaborCapacityRowV1>,
    pub production_commitments: Vec<ProductionCommitmentV1>,
    pub merchants: Vec<MerchantHandlingV3>,
    pub handling_coefficients: Vec<MerchantHandlingCoefficientV3>,
    pub final_demand_principals: Vec<FinalDemandPrincipalV3>,
    pub final_demand_orders: Vec<FinalDemandOrderV3>,
}
/// Exact V3 mass-weighted freight refusal classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum MaterialCircuitErrorV3 {
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
pub struct UnknownMaterialCircuitErrorCodeV3(pub u16);

impl TryFrom<u16> for MaterialCircuitErrorV3 {
    type Error = UnknownMaterialCircuitErrorCodeV3;

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
            _ => Err(UnknownMaterialCircuitErrorCodeV3(value)),
        }
    }
}

impl From<MaterialCircuitErrorV3> for u16 {
    fn from(value: MaterialCircuitErrorV3) -> Self {
        value as Self
    }
}

/// Loss attributed to a timed stage, independently of its capacity memberships.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreightLossReceiptV3 {
    pub lot_id: FreightLotIdV2,
    pub order_id: OrderIdV1,
    pub route_id: RouteIdV2,
    pub stage_index: u16,
    pub quantity: u64,
}
/// Atomic successor with native quantity receipts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialCircuitTransitionV3 {
    pub state: MaterialCircuitStateV3,
    pub production: Vec<ProductionReceiptV1>,
    pub dispatches: Vec<RoutedDispatchReceiptV2>,
    pub losses: Vec<FreightLossReceiptV3>,
    pub arrivals: Vec<crate::ArrivalReceiptV1>,
    pub deliveries: Vec<crate::DeliveryReceiptV1>,
    pub realizations: Vec<crate::RealizationReceiptV1>,
    pub handling: Vec<MerchantHandlingReceiptV3>,
    pub local_fulfillments: Vec<LocalRetailFulfillmentReceiptV3>,
    pub local_transfers: Vec<LocalTransferReceiptV3>,
}
