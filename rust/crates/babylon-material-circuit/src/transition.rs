//! Pure per-period transition for the exact routed material circuit.

mod merchant_admission;
mod outbound;

use std::collections::{BTreeMap, BTreeSet};

use babylon_kernel::content_digest::sha256_of;

use crate::production::{
    derive_shared_labor_requests, derive_shared_production, execute_shared_production,
};
use crate::{
    ArrivalReceipt, BacklogRow, CorridorId, DeliveryReceipt, FreightLossReceipt, FreightLotId,
    GoodId, InventoryRow, LaborCapacityRow, MaterialCircuitError, MaterialCircuitState,
    MaterialCircuitTransition, OrderId, RealizationReceipt, RouteId, RouteStage,
    RoutedDispatchReceipt, RoutedFreightLot, SiteId, StaffingPoolBinding, StaffingWorkRequest,
    UnitId, FREIGHT_LOSS_PARTS_PER_MILLION, MAX_MATERIAL_CIRCUIT_ROWS, MAX_ROUTE_STAGES_PER_ROUTE,
};

use crate::inventory::{
    credit_inventory, debit_inventory, publish_inventory, take_inventory, InventoryKey,
    InventoryLedger,
};
type SupplierKey = (SiteId, SiteId, GoodId, UnitId);
type SupplyPath = (RouteId, crate::SupplierTransport);
type CapacityKey = (u64, CorridorId);

fn check_row_limits(state: &MaterialCircuitState) -> Result<(), MaterialCircuitError> {
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
            .ok_or(MaterialCircuitError::Arithmetic)?,
    ];
    if lengths
        .into_iter()
        .any(|length| length > MAX_MATERIAL_CIRCUIT_ROWS)
    {
        return Err(MaterialCircuitError::RowLimit);
    }
    Ok(())
}

fn canonicalize_rows(state: &mut MaterialCircuitState) {
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

pub(crate) fn has_duplicate<T, K: PartialEq>(rows: &[T], key: impl Fn(&T) -> K) -> bool {
    rows.windows(2)
        .take(MAX_MATERIAL_CIRCUIT_ROWS)
        .any(|pair| key(&pair[0]) == key(&pair[1]))
}

fn validate_unique_rows(state: &MaterialCircuitState) -> Result<(), MaterialCircuitError> {
    let node_ids: BTreeSet<_> = state
        .site_logistics_nodes
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
        .map(|row| row.node_id)
        .collect();
    let dispatch_ids: BTreeSet<_> = state
        .freight
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
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
        return Err(MaterialCircuitError::DuplicateRow);
    }
    Ok(())
}

fn route_stages(state: &MaterialCircuitState, route: RouteId) -> &[RouteStage] {
    let start = state
        .route_stages
        .partition_point(|row| row.route_id < route);
    let end = state
        .route_stages
        .partition_point(|row| row.route_id <= route);
    &state.route_stages[start..end]
}

fn site_node(state: &MaterialCircuitState, site: SiteId) -> Option<crate::LogisticsNodeId> {
    state
        .site_logistics_nodes
        .binary_search_by_key(&site, |row| row.site_id)
        .ok()
        .map(|index| state.site_logistics_nodes[index].node_id)
}

fn validate_route_stages(legs: &[RouteStage]) -> Result<(), MaterialCircuitError> {
    if legs.is_empty() || legs.len() > MAX_ROUTE_STAGES_PER_ROUTE {
        return Err(MaterialCircuitError::RouteInvariant);
    }
    for (index, leg) in legs.iter().enumerate().take(MAX_ROUTE_STAGES_PER_ROUTE + 1) {
        if usize::from(leg.stage_index) != index
            || leg.travel_periods == 0
            || leg.loss_ppm > FREIGHT_LOSS_PARTS_PER_MILLION
        {
            return Err(MaterialCircuitError::RouteInvariant);
        }
        if index > 0 && legs[index - 1].to_node_id != leg.from_node_id {
            return Err(MaterialCircuitError::RouteInvariant);
        }
    }
    Ok(())
}

fn stage_capacities(
    state: &MaterialCircuitState,
    route: RouteId,
    ordinal: u16,
) -> &[crate::RouteStageCapacity] {
    let start = state
        .route_stage_capacities
        .partition_point(|row| (row.route_id, row.stage_index) < (route, ordinal));
    let end = state
        .route_stage_capacities
        .partition_point(|row| (row.route_id, row.stage_index) <= (route, ordinal));
    &state.route_stage_capacities[start..end]
}

fn grams_per_unit(
    state: &MaterialCircuitState,
    good: GoodId,
    unit: UnitId,
) -> Result<u64, MaterialCircuitError> {
    state
        .freight_mass_coefficients
        .binary_search_by_key(&(good, unit), |row| (row.good_id, row.unit_id))
        .ok()
        .map(|index| state.freight_mass_coefficients[index].grams_per_unit)
        .filter(|grams| *grams > 0)
        .ok_or(MaterialCircuitError::MassInvariant)
}

fn validate_routes(state: &MaterialCircuitState) -> Result<(), MaterialCircuitError> {
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
        return Err(MaterialCircuitError::RouteInvariant);
    }
    let mut modes = BTreeMap::new();
    for supplier in &state.supplier_routes {
        if modes
            .insert(supplier.route_id, supplier.transport_kind)
            .is_some_and(|previous| previous != supplier.transport_kind)
        {
            return Err(MaterialCircuitError::RouteInvariant);
        }
        let stages = route_stages(state, supplier.route_id);
        match supplier.transport_kind {
            crate::SupplierTransport::Local => {
                if !stages.is_empty()
                    || supplier.supplier_site_id == supplier.buyer_site_id
                    || site_node(state, supplier.supplier_site_id).is_none()
                    || site_node(state, supplier.buyer_site_id).is_none()
                {
                    return Err(MaterialCircuitError::RouteInvariant);
                }
            }
            crate::SupplierTransport::Staged => {
                validate_route_stages(stages)?;
                if site_node(state, supplier.supplier_site_id) != Some(stages[0].from_node_id)
                    || site_node(state, supplier.buyer_site_id)
                        != Some(stages[stages.len() - 1].to_node_id)
                {
                    return Err(MaterialCircuitError::RouteInvariant);
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
        return Err(MaterialCircuitError::CapacityInvariant);
    }
    if state
        .freight_mass_coefficients
        .iter()
        .any(|row| row.grams_per_unit == 0)
    {
        return Err(MaterialCircuitError::MassInvariant);
    }
    for order in &state.orders {
        grams_per_unit(state, order.good_id, order.unit_id)?;
    }
    Ok(())
}

fn order_index(state: &MaterialCircuitState, order: OrderId) -> Option<usize> {
    state
        .orders
        .binary_search_by_key(&order, |row| row.order_id)
        .ok()
}

fn supplier_routes(state: &MaterialCircuitState) -> BTreeMap<SupplierKey, SupplyPath> {
    state
        .supplier_routes
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
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
    lot: &RoutedFreightLot,
    legs: &[RouteStage],
) -> Result<u64, MaterialCircuitError> {
    legs.iter()
        .take(usize::from(lot.current_stage_index) + 1)
        .try_fold(lot.dispatch_period, |period, leg| {
            period
                .checked_add(u64::from(leg.travel_periods))
                .ok_or(MaterialCircuitError::Arithmetic)
        })
}

fn validate_orders_and_freight(state: &MaterialCircuitState) -> Result<(), MaterialCircuitError> {
    if state.orders.len() != state.backlog.len() {
        return Err(MaterialCircuitError::BacklogInvariant);
    }
    let routes = supplier_routes(state);
    let mut in_transit = BTreeMap::<OrderId, u128>::new();
    for lot in state.freight.iter().take(MAX_MATERIAL_CIRCUIT_ROWS + 1) {
        let Some(index) = order_index(state, lot.order_id) else {
            return Err(MaterialCircuitError::FreightInvariant);
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
            || routes.get(&supplier_key) != Some(&(lot.route_id, crate::SupplierTransport::Staged))
            || lot.source_site_id != order.supplier_site_id
            || lot.destination_site_id != order.buyer_site_id
            || lot.good_id != order.good_id
            || lot.unit_id != order.unit_id
        {
            return Err(MaterialCircuitError::FreightInvariant);
        }
        if expected_stage_arrival(lot, legs)? != lot.stage_arrival_period {
            return Err(MaterialCircuitError::FreightInvariant);
        }
        let total = in_transit.entry(lot.order_id).or_default();
        *total = total
            .checked_add(u128::from(lot.quantity))
            .ok_or(MaterialCircuitError::Arithmetic)?;
    }
    for (order, backlog) in state
        .orders
        .iter()
        .zip(&state.backlog)
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
    {
        if order.ordered == 0 {
            return Err(MaterialCircuitError::ZeroQuantity);
        }
        let accounted = order
            .delivered
            .checked_add(order.lost)
            .ok_or(MaterialCircuitError::Arithmetic)?;
        let local = routes
            .get(&(
                order.buyer_site_id,
                order.supplier_site_id,
                order.good_id,
                order.unit_id,
            ))
            .is_some_and(|(_, mode)| *mode == crate::SupplierTransport::Local);
        if (local
            && (order.lost != 0
                || order.shipped != order.delivered
                || order.realized != order.delivered))
            || order.realized > order.delivered
            || accounted > order.shipped
            || order.shipped > order.ordered
        {
            return Err(MaterialCircuitError::OrderInvariant);
        }
        if backlog.order_id != order.order_id || backlog.quantity != order.ordered - order.shipped {
            return Err(MaterialCircuitError::BacklogInvariant);
        }
        if in_transit.get(&order.order_id).copied().unwrap_or(0)
            != u128::from(order.shipped - accounted)
        {
            return Err(MaterialCircuitError::FreightInvariant);
        }
    }
    Ok(())
}

pub(crate) fn canonical_state(
    state: &MaterialCircuitState,
) -> Result<MaterialCircuitState, MaterialCircuitError> {
    check_row_limits(state)?;
    let mut canonical = state.clone();
    canonicalize_rows(&mut canonical);
    validate_unique_rows(&canonical)?;
    merchant_admission::validate_merchants(&canonical)?;
    validate_routes(&canonical)?;
    validate_orders_and_freight(&canonical)?;
    crate::production::validate_unique_rows(&canonical)?;
    crate::production::validate_processes(&canonical)?;
    crate::production::validate_periods(&canonical)?;
    if canonical.period == 0
        || canonical
            .corridor_capacities
            .iter()
            .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
            .any(|row| row.period < canonical.period)
    {
        return Err(MaterialCircuitError::PeriodInvariant);
    }
    Ok(canonical)
}

fn loss_quantity(quantity: u64, loss_ppm: u32) -> Result<u64, MaterialCircuitError> {
    let loss = u128::from(quantity)
        .checked_mul(u128::from(loss_ppm))
        .ok_or(MaterialCircuitError::Arithmetic)?
        / u128::from(FREIGHT_LOSS_PARTS_PER_MILLION);
    u64::try_from(loss).map_err(|_| MaterialCircuitError::Arithmetic)
}

fn process_due_freight(
    state: &mut MaterialCircuitState,
    inventory: &mut InventoryLedger,
    losses: &mut Vec<FreightLossReceipt>,
    arrivals: &mut Vec<ArrivalReceipt>,
    deliveries: &mut Vec<DeliveryReceipt>,
    realizations: &mut Vec<RealizationReceipt>,
) -> Result<(), MaterialCircuitError> {
    let opening = std::mem::take(&mut state.freight);
    let mut remaining = Vec::with_capacity(opening.len());
    for mut lot in opening.into_iter().take(MAX_MATERIAL_CIRCUIT_ROWS + 1) {
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
            .ok_or(MaterialCircuitError::Arithmetic)?;
        let order_index =
            order_index(state, lot.order_id).ok_or(MaterialCircuitError::FreightInvariant)?;
        state.orders[order_index].lost = state.orders[order_index]
            .lost
            .checked_add(lost)
            .ok_or(MaterialCircuitError::Arithmetic)?;
        if lost > 0 {
            losses.push(FreightLossReceipt {
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
                .ok_or(MaterialCircuitError::Arithmetic)?;
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
                .ok_or(MaterialCircuitError::Arithmetic)?;
            order.realized = order
                .realized
                .checked_add(retained)
                .ok_or(MaterialCircuitError::Arithmetic)?;
            arrivals.push(ArrivalReceipt {
                order_id: lot.order_id,
                quantity: retained,
            });
            deliveries.push(DeliveryReceipt {
                order_id: lot.order_id,
                quantity: retained,
            });
            realizations.push(RealizationReceipt {
                order_id: lot.order_id,
                quantity: retained,
            });
        }
    }
    state.freight = remaining;
    Ok(())
}

fn capacity_index(state: &MaterialCircuitState, key: CapacityKey) -> Option<usize> {
    state
        .corridor_capacities
        .binary_search_by_key(&key, |row| (row.period, row.corridor_id))
        .ok()
}

fn freight_lot_id(order: OrderId, period: u64) -> FreightLotId {
    let mut bytes = b"babylon.freight-lot.v2\0".to_vec();
    bytes.extend_from_slice(&order.as_bytes());
    bytes.extend_from_slice(&period.to_be_bytes());
    FreightLotId::from_bytes(sha256_of(&bytes))
}

fn rebuild_backlog(state: &mut MaterialCircuitState) {
    state.backlog = state
        .orders
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
        .map(|order| BacklogRow {
            order_id: order.order_id,
            quantity: order.ordered - order.shipped,
        })
        .collect();
}

fn prune_corridor_capacity(state: &mut MaterialCircuitState, next_period: u64) {
    state.corridor_capacities = std::mem::take(&mut state.corridor_capacities)
        .into_iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
        .filter(|row| row.period >= next_period)
        .collect();
}

/// Detached physical close before next-opening labor and production planning.
///
/// This is not a canonical opening register: newly dispatched freight still
/// shares its closing period. Only successful final planning yields a successor.
/// Private fields prevent callers from replacing closed inventory or receipts.
#[derive(Debug)]
pub struct ClosedMaterialPeriod {
    transition: MaterialCircuitTransition,
    next_period: u64,
}

impl ClosedMaterialPeriod {
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
    pub fn inventory(&self) -> &[InventoryRow] {
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
        bindings: &[StaffingPoolBinding],
    ) -> Result<Vec<StaffingWorkRequest>, MaterialCircuitError> {
        let state = &self.transition.state;
        let owners = staffing_work_owners(bindings)?;
        let production = derive_shared_labor_requests(state, self.next_period)?;
        let mut requests = Vec::new();
        for request in production {
            requests.push((
                crate::StaffingWorkSource::Production(request.process_id),
                request.site_id,
                request.unit_id,
                request.hours,
            ));
        }
        let mut needed = BTreeMap::<SiteId, u64>::new();
        for receipt in &self.transition.handling {
            let hours = needed.entry(receipt.site_id).or_default();
            *hours = hours
                .checked_add(receipt.needed_hours)
                .ok_or(MaterialCircuitError::Arithmetic)?;
        }
        for merchant in &state.merchants {
            requests.push((
                crate::StaffingWorkSource::MerchantHandling(merchant.site_id),
                merchant.site_id,
                merchant.labor_unit_id,
                needed.get(&merchant.site_id).copied().unwrap_or(0),
            ));
        }
        if owners.len() != requests.len() {
            return Err(MaterialCircuitError::ProcessInvariant);
        }
        requests
            .into_iter()
            .map(|(source, site, unit, hours)| {
                let binding = owners
                    .get(&source)
                    .ok_or(MaterialCircuitError::ProcessInvariant)?;
                if binding.site_id() != site || binding.unit_id() != unit {
                    return Err(MaterialCircuitError::ProcessInvariant);
                }
                Ok(StaffingWorkRequest::new(
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
        mut next_labor: Vec<LaborCapacityRow>,
    ) -> Result<MaterialCircuitTransition, MaterialCircuitError> {
        validate_next_labor(&self.transition.state, self.next_period, &next_labor)?;
        // The allocator performs binary searches before final canonicalization.
        next_labor.sort_unstable_by_key(|row| (row.period, row.site_id, row.unit_id));
        self.transition.state.labor = next_labor;
        self.finish()
    }

    fn finish(mut self) -> Result<MaterialCircuitTransition, MaterialCircuitError> {
        let state = &mut self.transition.state;
        derive_shared_production(state, self.next_period)?;
        prune_corridor_capacity(state, self.next_period);
        state.period = self.next_period;
        *state = canonical_state(state)?;
        Ok(self.transition)
    }
}

fn staffing_work_owners(
    bindings: &[StaffingPoolBinding],
) -> Result<BTreeMap<crate::StaffingWorkSource, &StaffingPoolBinding>, MaterialCircuitError> {
    if bindings.len() > MAX_MATERIAL_CIRCUIT_ROWS {
        return Err(MaterialCircuitError::RowLimit);
    }
    let mut owners = BTreeMap::new();
    let mut pools = BTreeSet::new();
    let mut principals = BTreeSet::new();
    for binding in bindings {
        if !pools.insert(binding.pool_id())
            || !principals.insert((binding.site_id(), binding.unit_id()))
        {
            return Err(MaterialCircuitError::DuplicateRow);
        }
        for process in binding.work_sources() {
            if owners.insert(*process, binding).is_some() {
                return Err(MaterialCircuitError::DuplicateRow);
            }
            if owners.len() > MAX_MATERIAL_CIRCUIT_ROWS {
                return Err(MaterialCircuitError::RowLimit);
            }
        }
    }
    Ok(owners)
}

fn validate_next_labor(
    state: &MaterialCircuitState,
    next_period: u64,
    rows: &[LaborCapacityRow],
) -> Result<(), MaterialCircuitError> {
    if rows.len() > MAX_MATERIAL_CIRCUIT_ROWS {
        return Err(MaterialCircuitError::RowLimit);
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
            return Err(MaterialCircuitError::PeriodInvariant);
        }
        if !actual.insert((row.site_id, row.unit_id)) {
            return Err(MaterialCircuitError::DuplicateRow);
        }
    }
    if actual != expected {
        return Err(MaterialCircuitError::CapacityInvariant);
    }
    Ok(())
}

/// Close one routed period atomically and return its canonical successor state.
///
/// # Errors
/// Returns the first exact schema, route, conservation, bound, or arithmetic refusal.
pub fn advance_material_circuit(
    opening: &MaterialCircuitState,
) -> Result<MaterialCircuitTransition, MaterialCircuitError> {
    close_material_period(opening)?.finish()
}

/// Execute due freight, prior production commitments and dispatch exactly once.
///
/// The result borrows no mutable opening state and cannot become a world
/// register until next-opening labor and normal planning have been resolved.
///
/// # Errors
/// Returns the same schema, route, conservation, bound or arithmetic refusals
/// as the one-shot transition, leaving the opening state unchanged.
pub fn close_material_period(
    opening: &MaterialCircuitState,
) -> Result<ClosedMaterialPeriod, MaterialCircuitError> {
    let mut state = canonical_state(opening)?;
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
    let production = execute_shared_production(&mut state)?;
    let mut inventory = take_inventory(&mut state);
    let outbound = outbound::dispatch_orders(&mut state, &mut inventory, &mut dispatches)?;
    rebuild_backlog(&mut state);
    publish_inventory(&mut state, inventory);
    let next_period = state
        .period
        .checked_add(1)
        .ok_or(MaterialCircuitError::Arithmetic)?;
    Ok(ClosedMaterialPeriod {
        next_period,
        transition: MaterialCircuitTransition {
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
    use crate::{MaterialCircuitError, MAX_FREIGHT_RESOURCE_REQUESTS};

    #[test]
    fn resource_group_ceiling_accepts_maximum_and_refuses_plus_one() {
        assert_eq!(
            ensure_resource_group_count(MAX_FREIGHT_RESOURCE_REQUESTS),
            Ok(())
        );
        assert_eq!(
            ensure_resource_group_count(MAX_FREIGHT_RESOURCE_REQUESTS + 1),
            Err(MaterialCircuitError::RowLimit)
        );
    }
}
