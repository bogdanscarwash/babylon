//! Pure per-period transition for the exact routed material circuit.

mod merchant_admission;
mod outbound;

use std::collections::{BTreeMap, BTreeSet};

use babylon_kernel::sha256_of;

use crate::model::ProductionState;
use crate::transition::{
    canonical_state_v1, derive_shared_labor_requests_v1, derive_shared_production_v1,
    execute_shared_production_v1,
};
use crate::{
    ArrivalReceiptV1, BacklogRowV1, CorridorIdV2, DeliveryReceiptV1, FreightLossReceiptV3,
    FreightLotIdV2, GoodIdV1, InventoryRowV1, LaborCapacityRowV1, MaterialCircuitErrorV3,
    MaterialCircuitStateV3, MaterialCircuitTransitionV3, OrderIdV1, RealizationReceiptV1,
    RouteIdV2, RouteStageV3, RoutedDispatchReceiptV2, RoutedFreightLotV3, SiteIdV1,
    StaffingPoolBindingV2, StaffingWorkRequestV2, UnitIdV1, FREIGHT_LOSS_PARTS_PER_MILLION_V2,
    MAX_MATERIAL_CIRCUIT_ROWS_V1, MAX_ROUTE_STAGES_PER_ROUTE_V3,
};

type InventoryKey = (SiteIdV1, GoodIdV1, UnitIdV1);
type InventoryLedger = BTreeMap<InventoryKey, u64>;
type SupplierKey = (SiteIdV1, SiteIdV1, GoodIdV1, UnitIdV1);
type SupplyPath = (RouteIdV2, crate::SupplierTransportV3);
type CapacityKey = (u64, CorridorIdV2);

fn check_row_limits(state: &MaterialCircuitStateV3) -> Result<(), MaterialCircuitErrorV3> {
    let lengths = [
        state.site_logistics_nodes.len(),
        state.process_outputs.len(),
        state.input_coefficients.len(),
        state.labor_coefficients.len(),
        state.supplier_routes.len(),
        state.freight_mass_coefficients.len(),
        state.route_stage_capacities.len(),
        state.route_stages.len(),
        state.inventory.len(),
        state.orders.len(),
        state.backlog.len(),
        state.freight.len(),
        state.corridor_capacities.len(),
        state.capacities.len(),
        state.labor.len(),
        state.production_commitments.len(),
        state.merchants.len(),
        state.handling_coefficients.len(),
        state.final_demand_principals.len(),
        state.final_demand_orders.len(),
        state
            .orders
            .len()
            .checked_add(state.final_demand_orders.len())
            .ok_or(MaterialCircuitErrorV3::Arithmetic)?,
    ];
    if lengths
        .into_iter()
        .any(|length| length > MAX_MATERIAL_CIRCUIT_ROWS_V1)
    {
        return Err(MaterialCircuitErrorV3::RowLimit);
    }
    Ok(())
}

fn canonicalize_rows(state: &mut MaterialCircuitStateV3) {
    state.merchants.sort();
    state.handling_coefficients.sort();
    state.final_demand_principals.sort();
    state.final_demand_orders.sort_by_key(|row| row.order_id);
    state.site_logistics_nodes.sort();
    state.process_outputs.sort();
    state.input_coefficients.sort();
    state.labor_coefficients.sort();
    state.supplier_routes.sort();
    state.freight_mass_coefficients.sort();
    state.route_stage_capacities.sort();
    state.route_stages.sort();
    state.inventory.sort();
    state.orders.sort_by_key(|row| row.order_id);
    state.backlog.sort_by_key(|row| row.order_id);
    state
        .freight
        .sort_by_key(|row| (row.stage_arrival_period, row.lot_id));
    state
        .corridor_capacities
        .sort_by_key(|row| (row.period, row.corridor_id));
    state
        .capacities
        .sort_by_key(|row| (row.period, row.site_id, row.process_id));
    state
        .labor
        .sort_by_key(|row| (row.period, row.site_id, row.unit_id));
    state
        .production_commitments
        .sort_by_key(|row| (row.period, row.site_id, row.process_id));
}

fn has_duplicate<T, K: PartialEq>(rows: &[T], key: impl Fn(&T) -> K) -> bool {
    rows.windows(2)
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1)
        .any(|pair| key(&pair[0]) == key(&pair[1]))
}

fn validate_unique_rows(state: &MaterialCircuitStateV3) -> Result<(), MaterialCircuitErrorV3> {
    let node_ids: BTreeSet<_> = state
        .site_logistics_nodes
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|row| row.node_id)
        .collect();
    let dispatch_ids: BTreeSet<_> = state
        .freight
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|row| (row.order_id, row.dispatch_period))
        .collect();
    let duplicate = has_duplicate(&state.freight_mass_coefficients, |row| {
        (row.good_id, row.unit_id)
    }) || has_duplicate(&state.route_stage_capacities, |row| {
        (row.route_id, row.stage_index, row.corridor_id)
    }) || has_duplicate(&state.site_logistics_nodes, |row| row.site_id)
        || node_ids.len() != state.site_logistics_nodes.len()
        || has_duplicate(&state.supplier_routes, |row| {
            (
                row.buyer_site_id,
                row.supplier_site_id,
                row.good_id,
                row.unit_id,
            )
        })
        || has_duplicate(&state.route_stages, |row| (row.route_id, row.stage_index))
        || has_duplicate(&state.inventory, |row| {
            (row.site_id, row.good_id, row.unit_id)
        })
        || has_duplicate(&state.orders, |row| row.order_id)
        || has_duplicate(&state.backlog, |row| row.order_id)
        || has_duplicate(&state.freight, |row| row.lot_id)
        || dispatch_ids.len() != state.freight.len()
        || has_duplicate(&state.corridor_capacities, |row| {
            (row.period, row.corridor_id)
        });
    if duplicate {
        return Err(MaterialCircuitErrorV3::DuplicateRow);
    }
    Ok(())
}

fn route_stages(state: &MaterialCircuitStateV3, route: RouteIdV2) -> &[RouteStageV3] {
    let start = state
        .route_stages
        .partition_point(|row| row.route_id < route);
    let end = state
        .route_stages
        .partition_point(|row| row.route_id <= route);
    &state.route_stages[start..end]
}

fn site_node(state: &MaterialCircuitStateV3, site: SiteIdV1) -> Option<crate::LogisticsNodeIdV2> {
    state
        .site_logistics_nodes
        .binary_search_by_key(&site, |row| row.site_id)
        .ok()
        .map(|index| state.site_logistics_nodes[index].node_id)
}

fn validate_route_stages(legs: &[RouteStageV3]) -> Result<(), MaterialCircuitErrorV3> {
    if legs.is_empty() || legs.len() > MAX_ROUTE_STAGES_PER_ROUTE_V3 {
        return Err(MaterialCircuitErrorV3::RouteInvariant);
    }
    for (index, leg) in legs
        .iter()
        .enumerate()
        .take(MAX_ROUTE_STAGES_PER_ROUTE_V3 + 1)
    {
        if usize::from(leg.stage_index) != index
            || leg.travel_periods == 0
            || leg.loss_ppm > FREIGHT_LOSS_PARTS_PER_MILLION_V2
        {
            return Err(MaterialCircuitErrorV3::RouteInvariant);
        }
        if index > 0 && legs[index - 1].to_node_id != leg.from_node_id {
            return Err(MaterialCircuitErrorV3::RouteInvariant);
        }
    }
    Ok(())
}

fn stage_capacities(
    state: &MaterialCircuitStateV3,
    route: RouteIdV2,
    ordinal: u16,
) -> &[crate::RouteStageCapacityV3] {
    let start = state
        .route_stage_capacities
        .partition_point(|row| (row.route_id, row.stage_index) < (route, ordinal));
    let end = state
        .route_stage_capacities
        .partition_point(|row| (row.route_id, row.stage_index) <= (route, ordinal));
    &state.route_stage_capacities[start..end]
}

fn grams_per_unit(
    state: &MaterialCircuitStateV3,
    good: GoodIdV1,
    unit: UnitIdV1,
) -> Result<u64, MaterialCircuitErrorV3> {
    state
        .freight_mass_coefficients
        .binary_search_by_key(&(good, unit), |row| (row.good_id, row.unit_id))
        .ok()
        .map(|index| state.freight_mass_coefficients[index].grams_per_unit)
        .filter(|grams| *grams > 0)
        .ok_or(MaterialCircuitErrorV3::MassInvariant)
}

fn validate_routes(state: &MaterialCircuitStateV3) -> Result<(), MaterialCircuitErrorV3> {
    let route_ids: BTreeSet<_> = state.route_stages.iter().map(|row| row.route_id).collect();
    let stage_ids: BTreeSet<_> = state
        .route_stages
        .iter()
        .map(|row| (row.route_id, row.stage_index))
        .collect();
    let mut corridors: BTreeSet<_> = state
        .route_stage_capacities
        .iter()
        .map(|row| row.corridor_id)
        .collect();
    corridors.extend(state.merchants.iter().map(|row| row.capacity_id));
    for route in route_ids {
        validate_route_stages(route_stages(state, route))?;
    }
    if state
        .route_stage_capacities
        .iter()
        .any(|row| !stage_ids.contains(&(row.route_id, row.stage_index)))
        || state
            .route_stages
            .iter()
            .any(|row| stage_capacities(state, row.route_id, row.stage_index).is_empty())
    {
        return Err(MaterialCircuitErrorV3::RouteInvariant);
    }
    let mut modes = BTreeMap::new();
    for supplier in &state.supplier_routes {
        if modes
            .insert(supplier.route_id, supplier.transport_kind)
            .is_some_and(|previous| previous != supplier.transport_kind)
        {
            return Err(MaterialCircuitErrorV3::RouteInvariant);
        }
        let stages = route_stages(state, supplier.route_id);
        match supplier.transport_kind {
            crate::SupplierTransportV3::Local => {
                if !stages.is_empty()
                    || supplier.supplier_site_id == supplier.buyer_site_id
                    || site_node(state, supplier.supplier_site_id).is_none()
                    || site_node(state, supplier.buyer_site_id).is_none()
                {
                    return Err(MaterialCircuitErrorV3::RouteInvariant);
                }
            }
            crate::SupplierTransportV3::Staged => {
                validate_route_stages(stages)?;
                if site_node(state, supplier.supplier_site_id) != Some(stages[0].from_node_id)
                    || site_node(state, supplier.buyer_site_id)
                        != Some(stages[stages.len() - 1].to_node_id)
                {
                    return Err(MaterialCircuitErrorV3::RouteInvariant);
                }
            }
        }
        grams_per_unit(state, supplier.good_id, supplier.unit_id)?;
    }
    if state
        .corridor_capacities
        .iter()
        .any(|row| !corridors.contains(&row.corridor_id))
    {
        return Err(MaterialCircuitErrorV3::CapacityInvariant);
    }
    if state
        .freight_mass_coefficients
        .iter()
        .any(|row| row.grams_per_unit == 0)
    {
        return Err(MaterialCircuitErrorV3::MassInvariant);
    }
    for order in &state.orders {
        grams_per_unit(state, order.good_id, order.unit_id)?;
    }
    Ok(())
}

fn order_index(state: &MaterialCircuitStateV3, order: OrderIdV1) -> Option<usize> {
    state
        .orders
        .binary_search_by_key(&order, |row| row.order_id)
        .ok()
}

fn supplier_routes(state: &MaterialCircuitStateV3) -> BTreeMap<SupplierKey, SupplyPath> {
    state
        .supplier_routes
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|row| {
            (
                (
                    row.buyer_site_id,
                    row.supplier_site_id,
                    row.good_id,
                    row.unit_id,
                ),
                (row.route_id, row.transport_kind),
            )
        })
        .collect()
}

fn expected_stage_arrival(
    lot: &RoutedFreightLotV3,
    legs: &[RouteStageV3],
) -> Result<u64, MaterialCircuitErrorV3> {
    legs.iter()
        .take(usize::from(lot.current_stage_index) + 1)
        .try_fold(lot.dispatch_period, |period, leg| {
            period
                .checked_add(u64::from(leg.travel_periods))
                .ok_or(MaterialCircuitErrorV3::Arithmetic)
        })
}

fn validate_orders_and_freight(
    state: &MaterialCircuitStateV3,
) -> Result<(), MaterialCircuitErrorV3> {
    if state.orders.len() != state.backlog.len() {
        return Err(MaterialCircuitErrorV3::BacklogInvariant);
    }
    let routes = supplier_routes(state);
    let mut in_transit = BTreeMap::<OrderIdV1, u128>::new();
    for lot in state.freight.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        let Some(index) = order_index(state, lot.order_id) else {
            return Err(MaterialCircuitErrorV3::FreightInvariant);
        };
        let order = &state.orders[index];
        let supplier_key = (
            order.buyer_site_id,
            order.supplier_site_id,
            order.good_id,
            order.unit_id,
        );
        let legs = route_stages(state, lot.route_id);
        if lot.quantity == 0
            || lot.lot_id != freight_lot_id(lot.order_id, lot.dispatch_period)
            || lot.dispatch_period >= state.period
            || lot.stage_arrival_period < state.period
            || usize::from(lot.current_stage_index) >= legs.len()
            || routes.get(&supplier_key)
                != Some(&(lot.route_id, crate::SupplierTransportV3::Staged))
            || lot.source_site_id != order.supplier_site_id
            || lot.destination_site_id != order.buyer_site_id
            || lot.good_id != order.good_id
            || lot.unit_id != order.unit_id
        {
            return Err(MaterialCircuitErrorV3::FreightInvariant);
        }
        if expected_stage_arrival(lot, legs)? != lot.stage_arrival_period {
            return Err(MaterialCircuitErrorV3::FreightInvariant);
        }
        let total = in_transit.entry(lot.order_id).or_default();
        *total = total
            .checked_add(u128::from(lot.quantity))
            .ok_or(MaterialCircuitErrorV3::Arithmetic)?;
    }
    for (order, backlog) in state
        .orders
        .iter()
        .zip(&state.backlog)
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        if order.ordered == 0 {
            return Err(MaterialCircuitErrorV3::ZeroQuantity);
        }
        let accounted = order
            .delivered
            .checked_add(order.lost)
            .ok_or(MaterialCircuitErrorV3::Arithmetic)?;
        let local = routes
            .get(&(
                order.buyer_site_id,
                order.supplier_site_id,
                order.good_id,
                order.unit_id,
            ))
            .is_some_and(|(_, mode)| *mode == crate::SupplierTransportV3::Local);
        if (local
            && (order.lost != 0
                || order.shipped != order.delivered
                || order.realized != order.delivered))
            || order.realized > order.delivered
            || accounted > order.shipped
            || order.shipped > order.ordered
        {
            return Err(MaterialCircuitErrorV3::OrderInvariant);
        }
        if backlog.order_id != order.order_id || backlog.quantity != order.ordered - order.shipped {
            return Err(MaterialCircuitErrorV3::BacklogInvariant);
        }
        if in_transit.get(&order.order_id).copied().unwrap_or(0)
            != u128::from(order.shipped - accounted)
        {
            return Err(MaterialCircuitErrorV3::FreightInvariant);
        }
    }
    Ok(())
}

fn production_state(state: &MaterialCircuitStateV3) -> ProductionState {
    ProductionState {
        period: state.period,
        process_outputs: state.process_outputs.clone(),
        input_coefficients: state.input_coefficients.clone(),
        labor_coefficients: state.labor_coefficients.clone(),
        inventory: state.inventory.clone(),
        capacities: state.capacities.clone(),
        labor: state.labor.clone(),
        production_commitments: state.production_commitments.clone(),
    }
}

fn merge_production_state(state: &mut MaterialCircuitStateV3, production: ProductionState) {
    state.inventory = production.inventory;
    state.capacities = production.capacities;
    state.labor = production.labor;
    state.production_commitments = production.production_commitments;
}

pub(crate) fn canonical_state_v3(
    state: &MaterialCircuitStateV3,
) -> Result<MaterialCircuitStateV3, MaterialCircuitErrorV3> {
    check_row_limits(state)?;
    let mut canonical = state.clone();
    canonicalize_rows(&mut canonical);
    validate_unique_rows(&canonical)?;
    merchant_admission::validate_merchants(&canonical)?;
    validate_routes(&canonical)?;
    validate_orders_and_freight(&canonical)?;
    canonical_state_v1(&production_state(&canonical))?;
    if canonical.period == 0
        || canonical
            .corridor_capacities
            .iter()
            .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
            .any(|row| row.period < canonical.period)
    {
        return Err(MaterialCircuitErrorV3::PeriodInvariant);
    }
    Ok(canonical)
}

fn take_inventory(state: &mut MaterialCircuitStateV3) -> InventoryLedger {
    std::mem::take(&mut state.inventory)
        .into_iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|row| ((row.site_id, row.good_id, row.unit_id), row.quantity))
        .collect()
}

fn publish_inventory(state: &mut MaterialCircuitStateV3, inventory: InventoryLedger) {
    state.inventory = inventory
        .into_iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|((site_id, good_id, unit_id), quantity)| InventoryRowV1 {
            site_id,
            good_id,
            unit_id,
            quantity,
        })
        .collect();
}

fn credit_inventory(
    inventory: &mut InventoryLedger,
    key: InventoryKey,
    quantity: u64,
) -> Result<(), MaterialCircuitErrorV3> {
    if quantity == 0 {
        return Ok(());
    }
    if let Some(current) = inventory.get_mut(&key) {
        *current = current
            .checked_add(quantity)
            .ok_or(MaterialCircuitErrorV3::Arithmetic)?;
        return Ok(());
    }
    if inventory.len() == MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        return Err(MaterialCircuitErrorV3::RowLimit);
    }
    inventory.insert(key, quantity);
    Ok(())
}

fn debit_inventory(
    inventory: &mut InventoryLedger,
    key: InventoryKey,
    quantity: u64,
) -> Result<(), MaterialCircuitErrorV3> {
    if quantity == 0 {
        return Ok(());
    }
    let current = inventory
        .get_mut(&key)
        .ok_or(MaterialCircuitErrorV3::FreightInvariant)?;
    *current = current
        .checked_sub(quantity)
        .ok_or(MaterialCircuitErrorV3::Arithmetic)?;
    Ok(())
}

fn loss_quantity(quantity: u64, loss_ppm: u32) -> Result<u64, MaterialCircuitErrorV3> {
    let loss = u128::from(quantity)
        .checked_mul(u128::from(loss_ppm))
        .ok_or(MaterialCircuitErrorV3::Arithmetic)?
        / u128::from(FREIGHT_LOSS_PARTS_PER_MILLION_V2);
    u64::try_from(loss).map_err(|_| MaterialCircuitErrorV3::Arithmetic)
}

fn process_due_freight(
    state: &mut MaterialCircuitStateV3,
    inventory: &mut InventoryLedger,
    losses: &mut Vec<FreightLossReceiptV3>,
    arrivals: &mut Vec<ArrivalReceiptV1>,
    deliveries: &mut Vec<DeliveryReceiptV1>,
    realizations: &mut Vec<RealizationReceiptV1>,
) -> Result<(), MaterialCircuitErrorV3> {
    let opening = std::mem::take(&mut state.freight);
    let mut remaining = Vec::with_capacity(opening.len());
    for mut lot in opening.into_iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        if lot.stage_arrival_period != state.period {
            remaining.push(lot);
            continue;
        }
        let index = usize::from(lot.current_stage_index);
        let (stage_index, loss_ppm, next_leg) = {
            let legs = route_stages(state, lot.route_id);
            let leg = &legs[index];
            let next_leg = legs
                .get(index + 1)
                .map(|next| (next.stage_index, next.travel_periods));
            (leg.stage_index, leg.loss_ppm, next_leg)
        };
        let lost = loss_quantity(lot.quantity, loss_ppm)?;
        let retained = lot
            .quantity
            .checked_sub(lost)
            .ok_or(MaterialCircuitErrorV3::Arithmetic)?;
        let order_index =
            order_index(state, lot.order_id).ok_or(MaterialCircuitErrorV3::FreightInvariant)?;
        state.orders[order_index].lost = state.orders[order_index]
            .lost
            .checked_add(lost)
            .ok_or(MaterialCircuitErrorV3::Arithmetic)?;
        if lost > 0 {
            losses.push(FreightLossReceiptV3 {
                lot_id: lot.lot_id,
                order_id: lot.order_id,
                route_id: lot.route_id,
                stage_index,
                quantity: lost,
            });
        }
        if let Some((next_stage_index, next_travel_periods)) = next_leg.filter(|_| retained > 0) {
            lot.current_stage_index = next_stage_index;
            lot.stage_arrival_period = state
                .period
                .checked_add(u64::from(next_travel_periods))
                .ok_or(MaterialCircuitErrorV3::Arithmetic)?;
            lot.quantity = retained;
            remaining.push(lot);
            continue;
        }
        if retained > 0 {
            credit_inventory(
                inventory,
                (lot.destination_site_id, lot.good_id, lot.unit_id),
                retained,
            )?;
            let order = &mut state.orders[order_index];
            order.delivered = order
                .delivered
                .checked_add(retained)
                .ok_or(MaterialCircuitErrorV3::Arithmetic)?;
            order.realized = order
                .realized
                .checked_add(retained)
                .ok_or(MaterialCircuitErrorV3::Arithmetic)?;
            arrivals.push(ArrivalReceiptV1 {
                order_id: lot.order_id,
                quantity: retained,
            });
            deliveries.push(DeliveryReceiptV1 {
                order_id: lot.order_id,
                quantity: retained,
            });
            realizations.push(RealizationReceiptV1 {
                order_id: lot.order_id,
                quantity: retained,
            });
        }
    }
    state.freight = remaining;
    Ok(())
}

fn capacity_index(state: &MaterialCircuitStateV3, key: CapacityKey) -> Option<usize> {
    state
        .corridor_capacities
        .binary_search_by_key(&key, |row| (row.period, row.corridor_id))
        .ok()
}

fn freight_lot_id(order: OrderIdV1, period: u64) -> FreightLotIdV2 {
    let mut bytes = b"babylon.freight-lot.v2\0".to_vec();
    bytes.extend_from_slice(&order.as_bytes());
    bytes.extend_from_slice(&period.to_be_bytes());
    FreightLotIdV2::from_bytes(sha256_of(&bytes))
}

fn rebuild_backlog(state: &mut MaterialCircuitStateV3) {
    state.backlog = state
        .orders
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|order| BacklogRowV1 {
            order_id: order.order_id,
            quantity: order.ordered - order.shipped,
        })
        .collect();
}

fn execute_production(
    state: &mut MaterialCircuitStateV3,
) -> Result<Vec<crate::ProductionReceiptV1>, MaterialCircuitErrorV3> {
    let mut production = production_state(state);
    let receipts = execute_shared_production_v1(&mut production)?;
    merge_production_state(state, production);
    Ok(receipts)
}

fn derive_next_production(
    state: &mut MaterialCircuitStateV3,
    next_period: u64,
) -> Result<(), MaterialCircuitErrorV3> {
    let mut production = production_state(state);
    derive_shared_production_v1(&mut production, next_period)?;
    merge_production_state(state, production);
    Ok(())
}

fn prune_corridor_capacity(state: &mut MaterialCircuitStateV3, next_period: u64) {
    state.corridor_capacities = std::mem::take(&mut state.corridor_capacities)
        .into_iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .filter(|row| row.period >= next_period)
        .collect();
}

/// Detached physical close before next-opening labor and production planning.
///
/// This is not a canonical opening register: newly dispatched freight still
/// shares its closing period. Only successful final planning yields a successor.
/// Private fields prevent callers from replacing closed inventory or receipts.
#[derive(Debug)]
pub struct ClosedMaterialPeriodV3 {
    transition: MaterialCircuitTransitionV3,
    next_period: u64,
}

impl ClosedMaterialPeriodV3 {
    /// The interval whose arrivals, production and dispatch have completed.
    #[must_use]
    pub const fn closing_period(&self) -> u64 {
        self.transition.state.period
    }

    /// The opening interval being requested and planned.
    #[must_use]
    pub const fn next_period(&self) -> u64 {
        self.next_period
    }

    /// Exact closing stock after dispatch, without a second inventory owner.
    #[must_use]
    pub fn inventory(&self) -> &[InventoryRowV1] {
        &self.transition.state.inventory
    }

    /// Request production work from next-opening inputs and capacity, and merchant
    /// work from this close's recorded nonlabor-feasible handling need.
    ///
    /// Every admitted work source has one request, including zero. Its `period` is
    /// this closing interval, as required by staffing. Neither current employment
    /// nor any scheduled hours limit the recorded need.
    ///
    /// # Errors
    /// Refuses incomplete, duplicate or foreign pool bindings, row bounds and
    /// hours that cannot be represented exactly as `u64`.
    pub fn staffing_requests(
        &self,
        bindings: &[StaffingPoolBindingV2],
    ) -> Result<Vec<StaffingWorkRequestV2>, MaterialCircuitErrorV3> {
        let state = &self.transition.state;
        let owners = staffing_work_owners(bindings)?;
        let production =
            derive_shared_labor_requests_v1(&production_state(state), self.next_period)?;
        let mut requests = Vec::new();
        for request in production {
            requests.push((
                crate::StaffingWorkSourceV2::Production(request.process_id),
                request.site_id,
                request.unit_id,
                request.hours,
            ));
        }
        let mut needed = BTreeMap::<SiteIdV1, u64>::new();
        for receipt in &self.transition.handling {
            let hours = needed.entry(receipt.site_id).or_default();
            *hours = hours
                .checked_add(receipt.needed_hours)
                .ok_or(MaterialCircuitErrorV3::Arithmetic)?;
        }
        for merchant in &state.merchants {
            requests.push((
                crate::StaffingWorkSourceV2::MerchantHandling(merchant.site_id),
                merchant.site_id,
                merchant.labor_unit_id,
                needed.get(&merchant.site_id).copied().unwrap_or(0),
            ));
        }
        if owners.len() != requests.len() {
            return Err(MaterialCircuitErrorV3::ProcessInvariant);
        }
        requests
            .into_iter()
            .map(|(source, site, unit, hours)| {
                let binding = owners
                    .get(&source)
                    .ok_or(MaterialCircuitErrorV3::ProcessInvariant)?;
                if binding.site_id() != site || binding.unit_id() != unit {
                    return Err(MaterialCircuitErrorV3::ProcessInvariant);
                }
                Ok(StaffingWorkRequestV2::new(
                    self.closing_period(),
                    binding.pool_id(),
                    source,
                    site,
                    unit,
                    hours,
                ))
            })
            .collect()
    }

    /// Replace the labor schedule with one exact next-opening row per principal.
    ///
    /// Zero hours are explicit. No preseeded future row survives this staffing
    /// ownership transfer. The normal planner still bounds commitments by both
    /// shared inputs and supplied labor; requests do not become commitments.
    ///
    /// # Errors
    /// Refuses missing/foreign/duplicate principals, wrong periods, row bounds,
    /// arithmetic and any invalid final circuit. No partial successor escapes.
    pub fn finish_with_labor(
        mut self,
        mut next_labor: Vec<LaborCapacityRowV1>,
    ) -> Result<MaterialCircuitTransitionV3, MaterialCircuitErrorV3> {
        validate_next_labor(&self.transition.state, self.next_period, &next_labor)?;
        // The allocator performs binary searches before final canonicalization.
        next_labor.sort_unstable_by_key(|row| (row.period, row.site_id, row.unit_id));
        self.transition.state.labor = next_labor;
        self.finish()
    }

    fn finish(mut self) -> Result<MaterialCircuitTransitionV3, MaterialCircuitErrorV3> {
        let state = &mut self.transition.state;
        derive_next_production(state, self.next_period)?;
        prune_corridor_capacity(state, self.next_period);
        state.period = self.next_period;
        *state = canonical_state_v3(state)?;
        Ok(self.transition)
    }
}

fn staffing_work_owners(
    bindings: &[StaffingPoolBindingV2],
) -> Result<BTreeMap<crate::StaffingWorkSourceV2, &StaffingPoolBindingV2>, MaterialCircuitErrorV3> {
    if bindings.len() > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        return Err(MaterialCircuitErrorV3::RowLimit);
    }
    let mut owners = BTreeMap::new();
    let mut pools = BTreeSet::new();
    let mut principals = BTreeSet::new();
    for binding in bindings {
        if !pools.insert(binding.pool_id())
            || !principals.insert((binding.site_id(), binding.unit_id()))
        {
            return Err(MaterialCircuitErrorV3::DuplicateRow);
        }
        for process in binding.work_sources() {
            if owners.insert(*process, binding).is_some() {
                return Err(MaterialCircuitErrorV3::DuplicateRow);
            }
            if owners.len() > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
                return Err(MaterialCircuitErrorV3::RowLimit);
            }
        }
    }
    Ok(owners)
}

fn validate_next_labor(
    state: &MaterialCircuitStateV3,
    next_period: u64,
    rows: &[LaborCapacityRowV1],
) -> Result<(), MaterialCircuitErrorV3> {
    if rows.len() > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        return Err(MaterialCircuitErrorV3::RowLimit);
    }
    // The detached close preserves the checked, process-sorted recipe roster.
    let mut expected: BTreeSet<_> = state
        .process_outputs
        .iter()
        .zip(&state.labor_coefficients)
        .map(|(output, coefficient)| (output.site_id, coefficient.unit_id))
        .collect();
    expected.extend(
        state
            .merchants
            .iter()
            .map(|row| (row.site_id, row.labor_unit_id)),
    );
    let mut actual = BTreeSet::new();
    for row in rows {
        if row.period != next_period {
            return Err(MaterialCircuitErrorV3::PeriodInvariant);
        }
        if !actual.insert((row.site_id, row.unit_id)) {
            return Err(MaterialCircuitErrorV3::DuplicateRow);
        }
    }
    if actual != expected {
        return Err(MaterialCircuitErrorV3::CapacityInvariant);
    }
    Ok(())
}

/// Close one routed period atomically and return its canonical successor state.
///
/// # Errors
/// Returns the first exact schema, route, conservation, bound, or arithmetic refusal.
pub fn advance_material_circuit_v3(
    opening: &MaterialCircuitStateV3,
) -> Result<MaterialCircuitTransitionV3, MaterialCircuitErrorV3> {
    close_material_period_v3(opening)?.finish()
}

/// Execute due freight, prior production commitments and dispatch exactly once.
///
/// The result borrows no mutable opening state and cannot become a world
/// register until next-opening labor and normal planning have been resolved.
///
/// # Errors
/// Returns the same schema, route, conservation, bound or arithmetic refusals
/// as the one-shot transition, leaving the opening state unchanged.
pub fn close_material_period_v3(
    opening: &MaterialCircuitStateV3,
) -> Result<ClosedMaterialPeriodV3, MaterialCircuitErrorV3> {
    let mut state = canonical_state_v3(opening)?;
    let mut inventory = take_inventory(&mut state);
    let mut losses = Vec::new();
    let mut arrivals = Vec::new();
    let mut deliveries = Vec::new();
    let mut realizations = Vec::new();
    let mut dispatches = Vec::new();
    process_due_freight(
        &mut state,
        &mut inventory,
        &mut losses,
        &mut arrivals,
        &mut deliveries,
        &mut realizations,
    )?;
    publish_inventory(&mut state, inventory);
    let production = execute_production(&mut state)?;
    let mut inventory = take_inventory(&mut state);
    let outbound = outbound::dispatch_orders(&mut state, &mut inventory, &mut dispatches)?;
    rebuild_backlog(&mut state);
    publish_inventory(&mut state, inventory);
    let next_period = state
        .period
        .checked_add(1)
        .ok_or(MaterialCircuitErrorV3::Arithmetic)?;
    Ok(ClosedMaterialPeriodV3 {
        next_period,
        transition: MaterialCircuitTransitionV3 {
            state,
            production,
            dispatches,
            losses,
            arrivals,
            deliveries,
            realizations,
            handling: outbound.handling,
            local_fulfillments: outbound.local_fulfillments,
            local_transfers: outbound.local_transfers,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::outbound::ensure_resource_group_count;
    use crate::{MaterialCircuitErrorV3, MAX_FREIGHT_RESOURCE_REQUESTS_V3};

    #[test]
    fn resource_group_ceiling_accepts_maximum_and_refuses_plus_one() {
        assert_eq!(
            ensure_resource_group_count(MAX_FREIGHT_RESOURCE_REQUESTS_V3),
            Ok(())
        );
        assert_eq!(
            ensure_resource_group_count(MAX_FREIGHT_RESOURCE_REQUESTS_V3 + 1),
            Err(MaterialCircuitErrorV3::RowLimit)
        );
    }
}
