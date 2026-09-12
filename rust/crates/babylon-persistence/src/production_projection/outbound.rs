//! Authenticated native order movements shared by capacity and merchant accounts.
//! These joins check committed facts; they never allocate or predict a shipment.

use super::ProductionProjectionError;
use babylon_material_circuit::{
    GoodId, MaterialCircuitState, OrderId, OutboundOrderId, RouteId, SiteId, SupplierTransport,
    UnitId,
};
use babylon_tick::material_world::MaterialTickReceipts;
use std::collections::BTreeMap;

type Result<T> = std::result::Result<T, ProductionProjectionError>;

#[derive(Clone)]
pub(super) struct OutboundFact {
    pub id: OutboundOrderId,
    pub site: SiteId,
    pub good: GoodId,
    pub unit: UnitId,
    pub route: Option<RouteId>,
    pub transport: Option<SupplierTransport>,
    pub requested: u64,
    pub quantity: u64,
    pub remaining: u64,
    pub grams_per_unit: u64,
}

pub(super) fn mass(state: &MaterialCircuitState, good: GoodId, unit: UnitId) -> Result<u64> {
    let mut rows = state
        .freight_mass_coefficients
        .iter()
        .filter(|row| row.good_id == good && row.unit_id == unit);
    let row = rows.next().ok_or(ProductionProjectionError::State)?;
    if row.grams_per_unit == 0 || rows.next().is_some() {
        return Err(ProductionProjectionError::State);
    }
    Ok(row.grams_per_unit)
}

pub(super) fn completed_facts(
    prior: &MaterialCircuitState,
    current: &MaterialCircuitState,
    receipt: &MaterialTickReceipts,
) -> Result<Vec<OutboundFact>> {
    if prior.period.checked_add(1) != Some(current.period) || receipt.resolve_tick != prior.period {
        return Err(ProductionProjectionError::History);
    }
    if !same_rows(&prior.supplier_routes, &current.supplier_routes)
        || !same_rows(
            &prior.freight_mass_coefficients,
            &current.freight_mass_coefficients,
        )
        || !same_rows(&prior.merchants, &current.merchants)
        || !same_rows(&prior.handling_coefficients, &current.handling_coefficients)
        || !same_rows(
            &prior.final_demand_principals,
            &current.final_demand_principals,
        )
    {
        return Err(ProductionProjectionError::State);
    }
    let mut facts = delivery_facts(prior, current, receipt)?;
    facts.extend(final_facts(prior, current, receipt)?);
    Ok(facts)
}

pub(super) fn same_rows<T: Clone + Ord>(before: &[T], after: &[T]) -> bool {
    let mut before = before.to_vec();
    let mut after = after.to_vec();
    before.sort_unstable();
    after.sort_unstable();
    before == after
}

fn delivery_facts(
    prior: &MaterialCircuitState,
    current: &MaterialCircuitState,
    receipt: &MaterialTickReceipts,
) -> Result<Vec<OutboundFact>> {
    let mut next = BTreeMap::new();
    for row in &current.orders {
        if next.insert(row.order_id, row).is_some() {
            return Err(ProductionProjectionError::State);
        }
    }
    let mut dispatches = BTreeMap::new();
    for row in &receipt.dispatches {
        if row.quantity == 0 || dispatches.insert(row.order_id, row).is_some() {
            return Err(ProductionProjectionError::State);
        }
    }
    let mut local = BTreeMap::new();
    for row in &receipt.local_transfers {
        if row.quantity == 0 || local.insert(row.order_id, row).is_some() {
            return Err(ProductionProjectionError::State);
        }
    }
    let mut facts = Vec::new();
    for order in &prior.orders {
        let closing = next
            .remove(&order.order_id)
            .ok_or(ProductionProjectionError::State)?;
        let mut relation = prior.supplier_routes.iter().filter(|row| {
            row.supplier_site_id == order.supplier_site_id
                && row.buyer_site_id == order.buyer_site_id
                && row.good_id == order.good_id
                && row.unit_id == order.unit_id
        });
        let route = relation.next();
        if relation.next().is_some() {
            return Err(ProductionProjectionError::State);
        }
        let dispatch = dispatches.remove(&order.order_id);
        let transfer = local.remove(&order.order_id);
        let quantity = match route.map(|row| row.transport_kind) {
            Some(SupplierTransport::Staged) if transfer.is_none() => {
                if dispatch.is_some_and(|row| Some(row.route_id) != route.map(|row| row.route_id)) {
                    return Err(ProductionProjectionError::State);
                }
                dispatch.map_or(0, |row| row.quantity)
            }
            Some(SupplierTransport::Local) if dispatch.is_none() => {
                if transfer.is_some_and(|row| {
                    row.supplier_site_id != order.supplier_site_id
                        || row.buyer_site_id != order.buyer_site_id
                        || row.good_id != order.good_id
                        || row.unit_id != order.unit_id
                }) {
                    return Err(ProductionProjectionError::State);
                }
                transfer.map_or(0, |row| row.quantity)
            }
            None if dispatch.is_none() && transfer.is_none() => 0,
            _ => return Err(ProductionProjectionError::State),
        };
        if order.supplier_site_id != closing.supplier_site_id
            || order.buyer_site_id != closing.buyer_site_id
            || order.good_id != closing.good_id
            || order.unit_id != closing.unit_id
            || order.ordered != closing.ordered
            || order.access_mode != closing.access_mode
            || order.shipped.checked_add(quantity) != Some(closing.shipped)
        {
            return Err(ProductionProjectionError::State);
        }
        facts.push(OutboundFact {
            id: OutboundOrderId::Delivery(order.order_id),
            site: order.supplier_site_id,
            good: order.good_id,
            unit: order.unit_id,
            route: route.map(|row| row.route_id),
            transport: route.map(|row| row.transport_kind),
            requested: order
                .ordered
                .checked_sub(order.shipped)
                .ok_or(ProductionProjectionError::State)?,
            quantity,
            remaining: closing
                .ordered
                .checked_sub(closing.shipped)
                .ok_or(ProductionProjectionError::State)?,
            grams_per_unit: mass(prior, order.good_id, order.unit_id)?,
        });
    }
    if !next.is_empty() || !dispatches.is_empty() || !local.is_empty() {
        return Err(ProductionProjectionError::State);
    }
    Ok(facts)
}

fn final_facts(
    prior: &MaterialCircuitState,
    current: &MaterialCircuitState,
    receipt: &MaterialTickReceipts,
) -> Result<Vec<OutboundFact>> {
    let mut next = BTreeMap::new();
    for row in &current.final_demand_orders {
        if next.insert(row.order_id, row).is_some() {
            return Err(ProductionProjectionError::State);
        }
    }
    let mut movements = BTreeMap::new();
    for row in &receipt.local_fulfillments {
        if row.quantity == 0 || movements.insert(row.order_id, row).is_some() {
            return Err(ProductionProjectionError::State);
        }
    }
    let mut facts = Vec::new();
    for order in &prior.final_demand_orders {
        let closing = next
            .remove(&order.order_id)
            .ok_or(ProductionProjectionError::State)?;
        let movement = movements.remove(&order.order_id);
        let quantity = movement.map_or(0, |row| row.quantity);
        if order.retailer_site_id != closing.retailer_site_id
            || order.demand_principal_id != closing.demand_principal_id
            || order.good_id != closing.good_id
            || order.unit_id != closing.unit_id
            || order.ordered != closing.ordered
            || order.fulfilled.checked_add(quantity) != Some(closing.fulfilled)
            || movement.is_some_and(|row| {
                row.retailer_site_id != order.retailer_site_id
                    || row.demand_principal_id != order.demand_principal_id
                    || row.good_id != order.good_id
                    || row.unit_id != order.unit_id
            })
        {
            return Err(ProductionProjectionError::State);
        }
        facts.push(OutboundFact {
            id: OutboundOrderId::LocalFinalDemand(order.order_id),
            site: order.retailer_site_id,
            good: order.good_id,
            unit: order.unit_id,
            route: None,
            transport: None,
            requested: order
                .ordered
                .checked_sub(order.fulfilled)
                .ok_or(ProductionProjectionError::State)?,
            quantity,
            remaining: closing
                .ordered
                .checked_sub(closing.fulfilled)
                .ok_or(ProductionProjectionError::State)?,
            grams_per_unit: mass(prior, order.good_id, order.unit_id)?,
        });
    }
    if !next.is_empty() || !movements.is_empty() {
        return Err(ProductionProjectionError::State);
    }
    Ok(facts)
}

pub(super) fn identity(
    id: OutboundOrderId,
) -> (
    OrderId,
    crate::production_observation::ProductionOutboundKind,
) {
    match id {
        OutboundOrderId::Delivery(id) => (
            id,
            crate::production_observation::ProductionOutboundKind::Delivery,
        ),
        OutboundOrderId::LocalFinalDemand(id) => (
            id,
            crate::production_observation::ProductionOutboundKind::LocalFinalDemand,
        ),
    }
}
