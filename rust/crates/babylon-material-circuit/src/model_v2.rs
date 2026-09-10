//! Current stable logistics identities, order accounts and dispatch receipts.
use crate::model::identity_type;
use crate::{GoodIdV1, OrderAccessModeV1, OrderIdV1, SiteIdV1, UnitIdV1};
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
