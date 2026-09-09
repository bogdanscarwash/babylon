//! Shared gram capacity from authenticated opening budgets and committed movements.

use super::{
    outbound::{completed_facts, identity, same_rows, OutboundFact},
    ProductionProjectionErrorV1,
};
use crate::{
    michigan_economy::digest_hex, michigan_material::MichiganMaterialCatalogV1,
    CompletedProductionFreightCapacityV2, ProductionCapacityKindV2,
    ProductionFreightCapacityAccountV2, ProductionFreightCapacityOrderV2,
    ProductionFreightReservationV2, ProductionRouteStageV2,
};
use babylon_material_circuit::{
    CorridorIdV2, MaterialCircuitStateV3, RouteIdV2, RouteStageV3, SiteIdV1, SupplierTransportV3,
};
use babylon_tick::material_world::MaterialTickReceiptsV4;
use std::collections::{BTreeMap, BTreeSet};

type Result<T> = std::result::Result<T, ProductionProjectionErrorV1>;
type CapacityKey = (CorridorIdV2, u64);
type Budgets = BTreeMap<CapacityKey, u64>;
type Reservations = BTreeMap<CapacityKey, Vec<ProductionFreightCapacityOrderV2>>;

#[derive(Default)]
struct Participants {
    routes: BTreeSet<RouteIdV2>,
    merchants: BTreeSet<SiteIdV1>,
}

pub(super) fn project_route_stages(
    state: &MaterialCircuitStateV3,
    route: RouteIdV2,
) -> Result<Vec<ProductionRouteStageV2>> {
    stages(state, route)?
        .into_iter()
        .map(|stage| {
            let ids = memberships(state, route, stage.stage_index)?;
            Ok(ProductionRouteStageV2 {
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

fn stages(state: &MaterialCircuitStateV3, route: RouteIdV2) -> Result<Vec<&RouteStageV3>> {
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
        return Err(ProductionProjectionErrorV1::State);
    }
    Ok(rows)
}

fn memberships(
    state: &MaterialCircuitStateV3,
    route: RouteIdV2,
    stage_index: u16,
) -> Result<BTreeSet<CorridorIdV2>> {
    let mut ids = BTreeSet::new();
    for row in state
        .route_stage_capacities
        .iter()
        .filter(|row| row.route_id == route && row.stage_index == stage_index)
    {
        if !ids.insert(row.corridor_id) {
            return Err(ProductionProjectionErrorV1::State);
        }
    }
    if ids.is_empty() {
        return Err(ProductionProjectionErrorV1::State);
    }
    Ok(ids)
}

fn budgets(state: &MaterialCircuitStateV3) -> Result<Budgets> {
    let mut rows = BTreeMap::new();
    for row in &state.corridor_capacities {
        if row.period < state.period
            || rows
                .insert((row.corridor_id, row.period), row.available_grams)
                .is_some()
        {
            return Err(ProductionProjectionErrorV1::State);
        }
    }
    Ok(rows)
}

fn participants(state: &MaterialCircuitStateV3) -> Result<BTreeMap<CorridorIdV2, Participants>> {
    let mut result = BTreeMap::<CorridorIdV2, Participants>::new();
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
            return Err(ProductionProjectionErrorV1::State);
        }
    }
    if state
        .corridor_capacities
        .iter()
        .any(|row| !result.contains_key(&row.corridor_id))
    {
        return Err(ProductionProjectionErrorV1::State);
    }
    Ok(result)
}

pub(super) fn project_freight_capacity_accounts(
    catalog: &MichiganMaterialCatalogV1,
    state: &MaterialCircuitStateV3,
    opening: Option<&MaterialCircuitStateV3>,
    receipt: Option<&MaterialTickReceiptsV4>,
) -> Result<Vec<ProductionFreightCapacityAccountV2>> {
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
        _ => return Err(ProductionProjectionErrorV1::History),
    };
    principals
        .into_iter()
        .map(|(id, participating)| {
            let complete = completed
                .as_ref()
                .map(|rows| CompletedProductionFreightCapacityV2 {
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
                return Err(ProductionProjectionErrorV1::State);
            }
            Ok(ProductionFreightCapacityAccountV2 {
                corridor_id: digest_hex(&id.as_bytes()),
                corridor_label: catalog
                    .corridor_label(id)
                    .ok_or(ProductionProjectionErrorV1::Content)?
                    .to_owned(),
                kind: if participating.merchants.is_empty() {
                    ProductionCapacityKindV2::Transport
                } else {
                    ProductionCapacityKindV2::MerchantHandling
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

fn capacity_order(fact: &OutboundFact) -> Result<ProductionFreightCapacityOrderV2> {
    let (id, kind) = identity(fact.id);
    Ok(ProductionFreightCapacityOrderV2 {
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
            .ok_or(ProductionProjectionErrorV1::Arithmetic)?,
    })
}

fn completed_reservations(
    prior: &MaterialCircuitStateV3,
    current: &MaterialCircuitStateV3,
    receipt: &MaterialTickReceiptsV4,
    current_budgets: &Budgets,
    principals: &BTreeMap<CorridorIdV2, Participants>,
) -> Result<BTreeMap<CapacityKey, ProductionFreightReservationV2>> {
    if !same_rows(&prior.route_stages, &current.route_stages)
        || !same_rows(
            &prior.route_stage_capacities,
            &current.route_stage_capacities,
        )
    {
        return Err(ProductionProjectionErrorV1::State);
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
        if fact.transport == Some(SupplierTransportV3::Staged) {
            let route = fact.route.ok_or(ProductionProjectionErrorV1::State)?;
            let mut departure = prior.period;
            let stages = stages(prior, route)?;
            if stages.is_empty() {
                return Err(ProductionProjectionErrorV1::State);
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
                    .ok_or(ProductionProjectionErrorV1::Arithmetic)?;
            }
            if receipt.dispatches.iter().any(|row| {
                row.order_id == identity(fact.id).0 && row.final_arrival_period != departure
            }) {
                return Err(ProductionProjectionErrorV1::State);
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
) -> Result<BTreeMap<CapacityKey, ProductionFreightReservationV2>> {
    let mut expected = prior.clone();
    let mut result = BTreeMap::new();
    for (key, mut orders) in reservations {
        orders.sort_unstable();
        let opening_available_grams = prior.get(&key).copied().unwrap_or(0);
        let newly_reserved_grams = orders
            .iter()
            .try_fold(0_u64, |sum, row| sum.checked_add(row.reserved_grams))
            .ok_or(ProductionProjectionErrorV1::Arithmetic)?;
        let remaining_available_grams = opening_available_grams
            .checked_sub(newly_reserved_grams)
            .ok_or(ProductionProjectionErrorV1::State)?;
        if let Some(value) = expected.get_mut(&key) {
            *value = remaining_available_grams;
        }
        result.insert(
            key,
            ProductionFreightReservationV2 {
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
        return Err(ProductionProjectionErrorV1::State);
    }
    Ok(result)
}

#[cfg(test)]
mod tests;
