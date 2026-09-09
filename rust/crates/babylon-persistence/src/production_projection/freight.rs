//! Capacity accounting from adjacent authenticated registers and actual dispatch receipts.

use std::collections::{BTreeMap, BTreeSet};

use babylon_material_circuit::{
    CorridorIdV2, MaterialCircuitStateV2, OrderIdV1, OrderRowV2, RouteIdV2, RouteLegV2, UnitIdV1,
};
use babylon_tick::material_world::MaterialTickReceiptsV3;

use super::ProductionProjectionErrorV1;
use crate::{
    michigan_economy::digest_hex, michigan_material::MichiganMaterialCatalogV1,
    CompletedProductionFreightCapacityV1, ProductionFreightCapacityAccountV1,
    ProductionFreightCapacityOrderV1, ProductionFreightReservationV1, ProductionRouteCorridorLegV1,
};

type Principal = (CorridorIdV2, UnitIdV1);
type CapacityKey = (Principal, u64);
type Budgets = BTreeMap<CapacityKey, u64>;
type Reservations = BTreeMap<CapacityKey, Vec<ProductionFreightCapacityOrderV1>>;

pub(super) fn project_route_legs(
    state: &MaterialCircuitStateV2,
    route: RouteIdV2,
) -> Result<Vec<ProductionRouteCorridorLegV1>, ProductionProjectionErrorV1> {
    Ok(route_legs(state, route)?
        .into_iter()
        .map(|leg| ProductionRouteCorridorLegV1 {
            leg_index: leg.leg_index,
            corridor_id: digest_hex(&leg.corridor_id.as_bytes()),
            travel_periods: u64::from(leg.travel_periods),
        })
        .collect())
}

fn route_legs(
    state: &MaterialCircuitStateV2,
    route: RouteIdV2,
) -> Result<Vec<&RouteLegV2>, ProductionProjectionErrorV1> {
    let mut legs: Vec<_> = state
        .route_legs
        .iter()
        .filter(|leg| leg.route_id == route)
        .collect();
    legs.sort_unstable_by_key(|leg| leg.leg_index);
    if legs.is_empty()
        || legs
            .iter()
            .enumerate()
            .any(|(index, leg)| usize::from(leg.leg_index) != index || leg.travel_periods == 0)
    {
        return Err(ProductionProjectionErrorV1::State);
    }
    Ok(legs)
}

fn order_route(
    state: &MaterialCircuitStateV2,
    order: &OrderRowV2,
) -> Result<RouteIdV2, ProductionProjectionErrorV1> {
    let mut routes = state.supplier_routes.iter().filter(|route| {
        route.supplier_site_id == order.supplier_site_id
            && route.buyer_site_id == order.buyer_site_id
            && route.good_id == order.good_id
            && route.unit_id == order.unit_id
    });
    let route = routes.next().ok_or(ProductionProjectionErrorV1::State)?;
    if routes.next().is_some() {
        return Err(ProductionProjectionErrorV1::State);
    }
    Ok(route.route_id)
}

fn budgets(state: &MaterialCircuitStateV2) -> Result<Budgets, ProductionProjectionErrorV1> {
    let mut result = Budgets::new();
    for row in &state.corridor_capacities {
        if row.period < state.period
            || result
                .insert(((row.corridor_id, row.unit_id), row.period), row.available)
                .is_some()
        {
            return Err(ProductionProjectionErrorV1::State);
        }
    }
    Ok(result)
}

pub(super) fn project_freight_capacity_accounts(
    catalog: &MichiganMaterialCatalogV1,
    state: &MaterialCircuitStateV2,
    opening: Option<&MaterialCircuitStateV2>,
    receipt: Option<&MaterialTickReceiptsV3>,
) -> Result<Vec<ProductionFreightCapacityAccountV1>, ProductionProjectionErrorV1> {
    let next = budgets(state)?;
    let mut principals = BTreeMap::<Principal, BTreeSet<RouteIdV2>>::new();
    for order in &state.orders {
        let route = order_route(state, order)?;
        for leg in route_legs(state, route)? {
            principals
                .entry((leg.corridor_id, order.unit_id))
                .or_default()
                .insert(route);
        }
    }
    if next
        .keys()
        .any(|(principal, _)| !principals.contains_key(principal))
    {
        return Err(ProductionProjectionErrorV1::State);
    }
    let completed = match (opening, receipt) {
        (None, None) if state.period == 1 => None,
        (Some(prior), Some(receipt))
            if prior.period.checked_add(1) == Some(state.period)
                && receipt.resolve_tick == prior.period =>
        {
            Some(completed_reservations(prior, state, receipt, &next)?)
        }
        _ => return Err(ProductionProjectionErrorV1::History),
    };
    principals
        .into_iter()
        .map(|(principal, routes)| {
            let unit = catalog
                .goods()
                .iter()
                .find(|good| good.unit_id() == principal.1)
                .ok_or(ProductionProjectionErrorV1::Content)?;
            let corridor_label = catalog
                .corridor_label(principal.0)
                .ok_or(ProductionProjectionErrorV1::Content)?;
            let completed = completed
                .as_ref()
                .map(|rows| {
                    let reservations = rows
                        .iter()
                        .filter(|((key, _), _)| key == &principal)
                        .map(|(_, row)| row.clone())
                        .collect::<Vec<_>>();
                    if reservations.is_empty() {
                        return Err(ProductionProjectionErrorV1::State);
                    }
                    Ok(CompletedProductionFreightCapacityV1 {
                        period: state.period - 1,
                        reservations,
                    })
                })
                .transpose()?;
            Ok(ProductionFreightCapacityAccountV1 {
                corridor_id: digest_hex(&principal.0.as_bytes()),
                corridor_label: corridor_label.to_owned(),
                unit_id: digest_hex(&principal.1.as_bytes()),
                unit: unit.unit_key.clone(),
                route_ids: routes
                    .iter()
                    .map(|route| digest_hex(&route.as_bytes()))
                    .collect(),
                next_opening_period: state.period,
                next_opening_available: next.get(&(principal, state.period)).copied().unwrap_or(0),
                completed,
            })
        })
        .collect()
}

fn same_topology(opening: &MaterialCircuitStateV2, next: &MaterialCircuitStateV2) -> bool {
    let mut before_legs = opening.route_legs.clone();
    let mut after_legs = next.route_legs.clone();
    before_legs.sort_unstable();
    after_legs.sort_unstable();
    let mut before_routes = opening.supplier_routes.clone();
    let mut after_routes = next.supplier_routes.clone();
    before_routes.sort_unstable();
    after_routes.sort_unstable();
    before_legs == after_legs && before_routes == after_routes
}

fn completed_reservations(
    opening: &MaterialCircuitStateV2,
    next: &MaterialCircuitStateV2,
    receipt: &MaterialTickReceiptsV3,
    next_budgets: &Budgets,
) -> Result<BTreeMap<CapacityKey, ProductionFreightReservationV1>, ProductionProjectionErrorV1> {
    if !same_topology(opening, next) || opening.orders.len() != next.orders.len() {
        return Err(ProductionProjectionErrorV1::State);
    }
    let prior_budgets = budgets(opening)?;
    let mut dispatches = BTreeMap::new();
    let mut lots = BTreeSet::new();
    for dispatch in &receipt.dispatches {
        if dispatch.quantity == 0
            || !lots.insert(dispatch.lot_id)
            || dispatches.insert(dispatch.order_id, dispatch).is_some()
        {
            return Err(ProductionProjectionErrorV1::State);
        }
    }
    let mut closing_orders = BTreeMap::<OrderIdV1, &OrderRowV2>::new();
    for order in &next.orders {
        if closing_orders.insert(order.order_id, order).is_some() {
            return Err(ProductionProjectionErrorV1::State);
        }
    }
    let mut reservations = Reservations::new();
    for order in &opening.orders {
        let closing = closing_orders
            .remove(&order.order_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        let route = order_route(opening, order)?;
        let dispatch = dispatches.remove(&order.order_id);
        let dispatched = dispatch.map_or(0, |row| row.quantity);
        if order.ordered != closing.ordered
            || order.access_mode != closing.access_mode
            || order.supplier_site_id != closing.supplier_site_id
            || order.buyer_site_id != closing.buyer_site_id
            || order.good_id != closing.good_id
            || order.unit_id != closing.unit_id
            || order.shipped.checked_add(dispatched) != Some(closing.shipped)
            || dispatch.is_some_and(|row| row.route_id != route)
        {
            return Err(ProductionProjectionErrorV1::State);
        }
        let requested = order
            .ordered
            .checked_sub(order.shipped)
            .ok_or(ProductionProjectionErrorV1::State)?;
        let remaining_unshipped = closing
            .ordered
            .checked_sub(closing.shipped)
            .ok_or(ProductionProjectionErrorV1::State)?;
        let mut departure = opening.period;
        for leg in route_legs(opening, route)? {
            reservations
                .entry(((leg.corridor_id, order.unit_id), departure))
                .or_default()
                .push(ProductionFreightCapacityOrderV1 {
                    order_id: digest_hex(&order.order_id.as_bytes()),
                    route_id: digest_hex(&route.as_bytes()),
                    good_id: digest_hex(&order.good_id.as_bytes()),
                    unit_id: digest_hex(&order.unit_id.as_bytes()),
                    requested,
                    dispatched,
                    remaining_unshipped,
                });
            departure = departure
                .checked_add(u64::from(leg.travel_periods))
                .ok_or(ProductionProjectionErrorV1::Arithmetic)?;
        }
        if dispatch.is_some_and(|row| row.final_arrival_period != departure) {
            return Err(ProductionProjectionErrorV1::State);
        }
    }
    if !dispatches.is_empty() || !closing_orders.is_empty() {
        return Err(ProductionProjectionErrorV1::State);
    }
    let mut expected_next = prior_budgets.clone();
    let mut result = BTreeMap::new();
    for (key, mut orders) in reservations {
        orders.sort_unstable();
        let opening_available = prior_budgets.get(&key).copied().unwrap_or(0);
        let newly_reserved = orders
            .iter()
            .try_fold(0_u64, |sum, order| sum.checked_add(order.dispatched))
            .ok_or(ProductionProjectionErrorV1::Arithmetic)?;
        let remaining_available = opening_available
            .checked_sub(newly_reserved)
            .ok_or(ProductionProjectionErrorV1::State)?;
        if let Some(value) = expected_next.get_mut(&key) {
            *value = remaining_available;
        }
        result.insert(
            key,
            ProductionFreightReservationV1 {
                reservation_period: key.1,
                opening_available,
                newly_reserved,
                remaining_available,
                orders,
            },
        );
    }
    expected_next.retain(|(_, period), _| *period >= next.period);
    if expected_next != *next_budgets {
        return Err(ProductionProjectionErrorV1::State);
    }
    Ok(result)
}

#[cfg(test)]
mod tests;
