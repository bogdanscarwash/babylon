//! Versioned routed-freight successor rows for the material circuit.

use crate::model::identity_type;
use crate::{
    BacklogRowV1, CapacityRowV1, InputOutputCoefficientV1, InventoryRowV1, LaborCapacityRowV1,
    LaborCoefficientV1, OrderAccessModeV1, OrderIdV1, ProcessOutputV1, ProductionCommitmentV1,
    ProductionReceiptV1, SiteIdV1, UnitIdV1, MAX_MATERIAL_CIRCUIT_ROWS_V1,
};

/// Designed route-depth ceiling for bounded reservation work.
pub const MAX_ROUTE_LEGS_PER_ROUTE_V2: usize = 16;
/// Derived ceiling for one inventory request plus every route-leg request per order.
pub const MAX_FREIGHT_RESOURCE_GROUPS_V2: usize =
    MAX_MATERIAL_CIRCUIT_ROWS_V1 * (MAX_ROUTE_LEGS_PER_ROUTE_V2 + 1);
/// Parts-per-million denominator for exact freight loss.
pub const FREIGHT_LOSS_PARTS_PER_MILLION_V2: u32 = 1_000_000;

identity_type!(LogisticsNodeIdV2);
identity_type!(CorridorIdV2);
identity_type!(RouteIdV2);
identity_type!(FreightLotIdV2);

/// Connect one material site to one logistics node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SiteLogisticsNodeV2 {
    pub site_id: SiteIdV1,
    pub node_id: LogisticsNodeIdV2,
}

/// The unique routed supplier relation for one exact order principal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SupplierRouteV2 {
    pub buyer_site_id: SiteIdV1,
    pub supplier_site_id: SiteIdV1,
    pub good_id: crate::GoodIdV1,
    pub unit_id: UnitIdV1,
    pub route_id: RouteIdV2,
}

/// One connected positive-duration route leg.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RouteLegV2 {
    pub route_id: RouteIdV2,
    pub leg_index: u16,
    pub corridor_id: CorridorIdV2,
    pub from_node_id: LogisticsNodeIdV2,
    pub to_node_id: LogisticsNodeIdV2,
    pub travel_periods: u16,
    pub loss_ppm: u32,
}

/// Available, unreserved exact capacity for one corridor departure period.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CorridorCapacityV2 {
    pub corridor_id: CorridorIdV2,
    pub unit_id: UnitIdV1,
    pub period: u64,
    pub available: u64,
}

/// Cumulative order quantities with an explicit freight-loss destination.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderRowV2 {
    pub order_id: OrderIdV1,
    pub access_mode: OrderAccessModeV1,
    pub buyer_site_id: SiteIdV1,
    pub supplier_site_id: SiteIdV1,
    pub good_id: crate::GoodIdV1,
    pub unit_id: UnitIdV1,
    pub ordered: u64,
    pub shipped: u64,
    pub lost: u64,
    pub delivered: u64,
    pub realized: u64,
}

/// One authoritative in-transit state for one exact routed lot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RoutedFreightLotV2 {
    pub lot_id: FreightLotIdV2,
    pub order_id: OrderIdV1,
    pub route_id: RouteIdV2,
    pub dispatch_period: u64,
    pub current_leg_index: u16,
    pub leg_arrival_period: u64,
    pub source_site_id: SiteIdV1,
    pub destination_site_id: SiteIdV1,
    pub good_id: crate::GoodIdV1,
    pub unit_id: UnitIdV1,
    pub quantity: u64,
}

/// Complete V2 auxiliary-register state at the opening of `period`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialCircuitStateV2 {
    pub period: u64,
    pub site_logistics_nodes: Vec<SiteLogisticsNodeV2>,
    pub process_outputs: Vec<ProcessOutputV1>,
    pub input_coefficients: Vec<InputOutputCoefficientV1>,
    pub labor_coefficients: Vec<LaborCoefficientV1>,
    pub supplier_routes: Vec<SupplierRouteV2>,
    pub route_legs: Vec<RouteLegV2>,
    pub inventory: Vec<InventoryRowV1>,
    pub orders: Vec<OrderRowV2>,
    pub backlog: Vec<BacklogRowV1>,
    pub freight: Vec<RoutedFreightLotV2>,
    pub corridor_capacities: Vec<CorridorCapacityV2>,
    pub capacities: Vec<CapacityRowV1>,
    pub labor: Vec<LaborCapacityRowV1>,
    pub production_commitments: Vec<ProductionCommitmentV1>,
}

/// Exact V2 routed-material refusal classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum MaterialCircuitErrorV2 {
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
}

/// Unknown language-neutral routed-material refusal code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownMaterialCircuitErrorCodeV2(pub u16);

impl TryFrom<u16> for MaterialCircuitErrorV2 {
    type Error = UnknownMaterialCircuitErrorCodeV2;

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
            _ => Err(UnknownMaterialCircuitErrorCodeV2(value)),
        }
    }
}

impl From<MaterialCircuitErrorV2> for u16 {
    fn from(value: MaterialCircuitErrorV2) -> Self {
        value as Self
    }
}

impl From<crate::MaterialCircuitErrorV1> for MaterialCircuitErrorV2 {
    fn from(value: crate::MaterialCircuitErrorV1) -> Self {
        match value {
            crate::MaterialCircuitErrorV1::RowLimit => Self::RowLimit,
            crate::MaterialCircuitErrorV1::ZeroQuantity => Self::ZeroQuantity,
            crate::MaterialCircuitErrorV1::DuplicateRow => Self::DuplicateRow,
            crate::MaterialCircuitErrorV1::OrderInvariant => Self::OrderInvariant,
            crate::MaterialCircuitErrorV1::BacklogInvariant => Self::BacklogInvariant,
            crate::MaterialCircuitErrorV1::TransitInvariant => Self::FreightInvariant,
            crate::MaterialCircuitErrorV1::ProcessInvariant => Self::ProcessInvariant,
            crate::MaterialCircuitErrorV1::PeriodInvariant => Self::PeriodInvariant,
            crate::MaterialCircuitErrorV1::Arithmetic => Self::Arithmetic,
            crate::MaterialCircuitErrorV1::WireLimit => Self::WireLimit,
            crate::MaterialCircuitErrorV1::WireDomain => Self::WireDomain,
            crate::MaterialCircuitErrorV1::WireVersion => Self::WireVersion,
            crate::MaterialCircuitErrorV1::WireTruncated => Self::WireTruncated,
            crate::MaterialCircuitErrorV1::WireTrailing => Self::WireTrailing,
            crate::MaterialCircuitErrorV1::WireEnum => Self::WireEnum,
            crate::MaterialCircuitErrorV1::WireNoncanonical => Self::WireNoncanonical,
        }
    }
}

/// Inventory and whole-route capacity reserved for one new freight lot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutedDispatchReceiptV2 {
    pub lot_id: FreightLotIdV2,
    pub order_id: OrderIdV1,
    pub route_id: RouteIdV2,
    pub quantity: u64,
    pub final_arrival_period: u64,
}

/// Exact goods lost when one route leg completes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreightLossReceiptV2 {
    pub lot_id: FreightLotIdV2,
    pub order_id: OrderIdV1,
    pub corridor_id: CorridorIdV2,
    pub quantity: u64,
}

/// Atomic result of closing one routed material-circuit period.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialCircuitTransitionV2 {
    pub state: MaterialCircuitStateV2,
    pub production: Vec<ProductionReceiptV1>,
    pub dispatches: Vec<RoutedDispatchReceiptV2>,
    pub losses: Vec<FreightLossReceiptV2>,
    pub arrivals: Vec<crate::ArrivalReceiptV1>,
    pub deliveries: Vec<crate::DeliveryReceiptV1>,
    pub realizations: Vec<crate::RealizationReceiptV1>,
}
