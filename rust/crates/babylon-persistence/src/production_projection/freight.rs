//! Shared gram capacity from authenticated opening budgets and committed movements.

use super::{
    outbound::{completed_facts, identity, same_rows, OutboundFact},
    ProductionProjectionError,
};
use crate::{
    michigan_economy::digest_hex, michigan_material::MichiganMaterialCatalog,
    production_observation::CompletedProductionFreightCapacity,
    production_observation::ProductionCapacityKind,
    production_observation::ProductionFreightCapacityAccount,
    production_observation::ProductionFreightCapacityOrder,
    production_observation::ProductionFreightReservation,
    production_observation::ProductionRouteStage,
};
use babylon_material_circuit::{
    CorridorId, MaterialCircuitState, RouteId, RouteStage, SiteId, SupplierTransport,
};
use babylon_tick::material_world::MaterialTickReceipts;
use std::collections::{BTreeMap, BTreeSet};

type Result<T> = std::result::Result<T, ProductionProjectionError>;
type CapacityKey = (CorridorId, u64);
type Budgets = BTreeMap<CapacityKey, u64>;
type Reservations = BTreeMap<CapacityKey, Vec<ProductionFreightCapacityOrder>>;

#[derive(Default)]
struct Participants {
    routes: BTreeSet<RouteId>,
    merchants: BTreeSet<SiteId>,
}

pub(super) fn project_route_stages(
    state: &MaterialCircuitState,
    route: RouteId,
) -> Result<Vec<ProductionRouteStage>> {
    stages(state, route)?
        .into_iter()
        .map(|stage| {
            let ids = memberships(state, route, stage.stage_index)?;
            Ok(ProductionRouteStage {
                stage_index: stage.stage_index,
                travel_periods: u64::from(stage.travel_periods),
                capacity_ids: ids
                    .into_iter()
                    .map(|id| digest_hex(&id.as_bytes()))
                    .collect(),
            })
        })
        .collect()
}

fn stages(state: &MaterialCircuitState, route: RouteId) -> Result<Vec<&RouteStage>> {
    let mut rows: Vec<_> = state
        .route_stages
        .iter()
        .filter(|row| row.route_id == route)
        .collect();
    rows.sort_unstable_by_key(|row| row.stage_index);
    if rows
        .iter()
        .enumerate()
        .any(|(index, row)| usize::from(row.stage_index) != index || row.travel_periods == 0)
    {
        return Err(ProductionProjectionError::State);
    }
    Ok(rows)
}

fn memberships(
    state: &MaterialCircuitState,
    route: RouteId,
    stage_index: u16,
) -> Result<BTreeSet<CorridorId>> {
    let mut ids = BTreeSet::new();
    for row in state
        .route_stage_capacities
        .iter()
        .filter(|row| row.route_id == route && row.stage_index == stage_index)
    {
        if !ids.insert(row.corridor_id) {
            return Err(ProductionProjectionError::State);
        }
    }
    if ids.is_empty() {
        return Err(ProductionProjectionError::State);
    }
    Ok(ids)
}

fn budgets(state: &MaterialCircuitState) -> Result<Budgets> {
    let mut rows = BTreeMap::new();
    for row in &state.corridor_capacities {
        if row.period < state.period
            || rows
                .insert((row.corridor_id, row.period), row.available_grams)
                .is_some()
        {
            return Err(ProductionProjectionError::State);
        }
    }
    Ok(rows)
}

fn participants(state: &MaterialCircuitState) -> Result<BTreeMap<CorridorId, Participants>> {
    let mut result = BTreeMap::<CorridorId, Participants>::new();
    for stage in &state.route_stages {
        for id in memberships(state, stage.route_id, stage.stage_index)? {
            result.entry(id).or_default().routes.insert(stage.route_id);
        }
    }
    for merchant in &state.merchants {
        let row = result.entry(merchant.capacity_id).or_default();
        if !row.routes.is_empty()
            || !row.merchants.insert(merchant.site_id)
            || row.merchants.len() != 1
        {
            return Err(ProductionProjectionError::State);
        }
    }
    if state
        .corridor_capacities
        .iter()
        .any(|row| !result.contains_key(&row.corridor_id))
    {
        return Err(ProductionProjectionError::State);
    }
    Ok(result)
}

pub(super) fn project_freight_capacity_accounts(
    catalog: &MichiganMaterialCatalog,
    state: &MaterialCircuitState,
    opening: Option<&MaterialCircuitState>,
    receipt: Option<&MaterialTickReceipts>,
) -> Result<Vec<ProductionFreightCapacityAccount>> {
    let current = budgets(state)?;
    let principals = participants(state)?;
    let completed = match (opening, receipt) {
        (None, None) if state.period == 1 => None,
        (Some(prior), Some(receipt)) => Some(completed_reservations(
            prior,
            state,
            receipt,
            &current,
            &principals,
        )?),
        _ => return Err(ProductionProjectionError::History),
    };
    principals
        .into_iter()
        .map(|(id, participating)| {
            let complete = completed
                .as_ref()
                .map(|rows| CompletedProductionFreightCapacity {
                    period: state.period - 1,
                    reservations: rows
                        .iter()
                        .filter(|((principal, _), _)| *principal == id)
                        .map(|(_, row)| row.clone())
                        .collect(),
                });
            if complete
                .as_ref()
                .is_some_and(|row| row.reservations.is_empty())
            {
                return Err(ProductionProjectionError::State);
            }
            Ok(ProductionFreightCapacityAccount {
                corridor_id: digest_hex(&id.as_bytes()),
                corridor_label: catalog
                    .corridor_label(id)
                    .ok_or(ProductionProjectionError::Content)?
                    .to_owned(),
                kind: if participating.merchants.is_empty() {
                    ProductionCapacityKind::Transport
                } else {
                    ProductionCapacityKind::MerchantHandling
                },
                merchant_site_ids: participating
                    .merchants
                    .iter()
                    .map(|id| digest_hex(&id.as_bytes()))
                    .collect(),
                route_ids: participating
                    .routes
                    .iter()
                    .map(|id| digest_hex(&id.as_bytes()))
                    .collect(),
                next_opening_period: state.period,
                next_opening_available_grams: current
                    .get(&(id, state.period))
                    .copied()
                    .unwrap_or(0),
                completed: complete,
            })
        })
        .collect()
}

fn capacity_order(fact: &OutboundFact) -> Result<ProductionFreightCapacityOrder> {
    let (id, kind) = identity(fact.id);
    Ok(ProductionFreightCapacityOrder {
        order_id: digest_hex(&id.as_bytes()),
        kind,
        supplier_site_id: digest_hex(&fact.site.as_bytes()),
        route_id: fact.route.map(|id| digest_hex(&id.as_bytes())),
        good_id: digest_hex(&fact.good.as_bytes()),
        unit_id: digest_hex(&fact.unit.as_bytes()),
        requested: fact.requested,
        dispatched: fact.quantity,
        remaining_unshipped: fact.remaining,
        grams_per_unit: fact.grams_per_unit,
        requested_grams: u128::from(fact.requested) * u128::from(fact.grams_per_unit),
        reserved_grams: fact
            .quantity
            .checked_mul(fact.grams_per_unit)
            .ok_or(ProductionProjectionError::Arithmetic)?,
    })
}

fn completed_reservations(
    prior: &MaterialCircuitState,
    current: &MaterialCircuitState,
    receipt: &MaterialTickReceipts,
    current_budgets: &Budgets,
    principals: &BTreeMap<CorridorId, Participants>,
) -> Result<BTreeMap<CapacityKey, ProductionFreightReservation>> {
    if !same_rows(&prior.route_stages, &current.route_stages)
        || !same_rows(
            &prior.route_stage_capacities,
            &current.route_stage_capacities,
        )
    {
        return Err(ProductionProjectionError::State);
    }
    let facts = completed_facts(prior, current, receipt)?;
    let mut reservations = Reservations::new();
    // A completed zero principal remains visible even after all finite orders finish.
    for id in principals.keys() {
        reservations.entry((*id, prior.period)).or_default();
    }
    let prior_budgets = budgets(prior)?;
    for fact in facts {
        let order = capacity_order(&fact)?;
        if fact.transport == Some(SupplierTransport::Staged) {
            let route = fact.route.ok_or(ProductionProjectionError::State)?;
            let mut departure = prior.period;
            let stages = stages(prior, route)?;
            if stages.is_empty() {
                return Err(ProductionProjectionError::State);
            }
            for stage in stages {
                for capacity in memberships(prior, route, stage.stage_index)? {
                    reservations
                        .entry((capacity, departure))
                        .or_default()
                        .push(order.clone());
                }
                departure = departure
                    .checked_add(u64::from(stage.travel_periods))
                    .ok_or(ProductionProjectionError::Arithmetic)?;
            }
            if receipt.dispatches.iter().any(|row| {
                row.order_id == identity(fact.id).0 && row.final_arrival_period != departure
            }) {
                return Err(ProductionProjectionError::State);
            }
        }
        if let Some(merchant) = prior.merchants.iter().find(|row| row.site_id == fact.site) {
            reservations
                .entry((merchant.capacity_id, prior.period))
                .or_default()
                .push(order);
        }
    }
    reconcile_reservation_budgets(
        &prior_budgets,
        current_budgets,
        current.period,
        reservations,
    )
}

fn reconcile_reservation_budgets(
    prior: &Budgets,
    current: &Budgets,
    next_period: u64,
    reservations: Reservations,
) -> Result<BTreeMap<CapacityKey, ProductionFreightReservation>> {
    let mut expected = prior.clone();
    let mut result = BTreeMap::new();
    for (key, mut orders) in reservations {
        orders.sort_unstable();
        let opening_available_grams = prior.get(&key).copied().unwrap_or(0);
        let newly_reserved_grams = orders
            .iter()
            .try_fold(0_u64, |sum, row| sum.checked_add(row.reserved_grams))
            .ok_or(ProductionProjectionError::Arithmetic)?;
        let remaining_available_grams = opening_available_grams
            .checked_sub(newly_reserved_grams)
            .ok_or(ProductionProjectionError::State)?;
        if let Some(value) = expected.get_mut(&key) {
            *value = remaining_available_grams;
        }
        result.insert(
            key,
            ProductionFreightReservation {
                reservation_period: key.1,
                opening_available_grams,
                newly_reserved_grams,
                remaining_available_grams,
                orders,
            },
        );
    }
    expected.retain(|(_, period), _| *period >= next_period);
    if expected != *current {
        return Err(ProductionProjectionError::State);
    }
    Ok(result)
}

#[cfg(test)]
mod tests;
