//! Pure per-period transition for the exact routed material circuit.

use std::collections::{BTreeMap, BTreeSet};

use babylon_kernel::sha256_of;

use crate::transition::{
    canonical_state_v1, derive_shared_labor_requests_v1, derive_shared_production_v1,
    execute_shared_production_v1, proportional_floor,
};
use crate::{
    ArrivalReceiptV1, BacklogRowV1, CorridorIdV2, DeliveryReceiptV1, FreightLossReceiptV2,
    FreightLotIdV2, GoodIdV1, InventoryRowV1, LaborCapacityRowV1, MaterialCircuitErrorV2,
    MaterialCircuitStateV1, MaterialCircuitStateV2, MaterialCircuitTransitionV2, OrderIdV1,
    RealizationReceiptV1, RouteIdV2, RouteLegV2, RoutedDispatchReceiptV2, RoutedFreightLotV2,
    SiteIdV1, StaffingPoolBindingV1, StaffingWorkRequestV1, UnitIdV1,
    FREIGHT_LOSS_PARTS_PER_MILLION_V2, MAX_FREIGHT_RESOURCE_GROUPS_V2,
    MAX_MATERIAL_CIRCUIT_ROWS_V1, MAX_ROUTE_LEGS_PER_ROUTE_V2,
};

type InventoryKey = (SiteIdV1, GoodIdV1, UnitIdV1);
type InventoryLedger = BTreeMap<InventoryKey, u64>;
type SupplierKey = (SiteIdV1, SiteIdV1, GoodIdV1, UnitIdV1);
type CapacityKey = (u64, CorridorIdV2, UnitIdV1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FreightResourceKey {
    Inventory(InventoryKey),
    Corridor(CapacityKey),
}

#[derive(Debug, Clone, Copy)]
struct FreightRequest {
    order_index: usize,
    requested: u64,
}

fn check_row_limits(state: &MaterialCircuitStateV2) -> Result<(), MaterialCircuitErrorV2> {
    let lengths = [
        state.site_logistics_nodes.len(),
        state.process_outputs.len(),
        state.input_coefficients.len(),
        state.labor_coefficients.len(),
        state.supplier_routes.len(),
        state.route_legs.len(),
        state.inventory.len(),
        state.orders.len(),
        state.backlog.len(),
        state.freight.len(),
        state.corridor_capacities.len(),
        state.capacities.len(),
        state.labor.len(),
        state.production_commitments.len(),
    ];
    if lengths
        .into_iter()
        .any(|length| length > MAX_MATERIAL_CIRCUIT_ROWS_V1)
    {
        return Err(MaterialCircuitErrorV2::RowLimit);
    }
    Ok(())
}

fn canonicalize_rows(state: &mut MaterialCircuitStateV2) {
    state.site_logistics_nodes.sort();
    state.process_outputs.sort();
    state.input_coefficients.sort();
    state.labor_coefficients.sort();
    state.supplier_routes.sort();
    state.route_legs.sort();
    state.inventory.sort();
    state.orders.sort_by_key(|row| row.order_id);
    state.backlog.sort_by_key(|row| row.order_id);
    state
        .freight
        .sort_by_key(|row| (row.leg_arrival_period, row.lot_id));
    state
        .corridor_capacities
        .sort_by_key(|row| (row.period, row.corridor_id, row.unit_id));
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

fn validate_unique_rows(state: &MaterialCircuitStateV2) -> Result<(), MaterialCircuitErrorV2> {
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
    let duplicate = has_duplicate(&state.site_logistics_nodes, |row| row.site_id)
        || node_ids.len() != state.site_logistics_nodes.len()
        || has_duplicate(&state.supplier_routes, |row| {
            (
                row.buyer_site_id,
                row.supplier_site_id,
                row.good_id,
                row.unit_id,
            )
        })
        || has_duplicate(&state.route_legs, |row| (row.route_id, row.leg_index))
        || has_duplicate(&state.inventory, |row| {
            (row.site_id, row.good_id, row.unit_id)
        })
        || has_duplicate(&state.orders, |row| row.order_id)
        || has_duplicate(&state.backlog, |row| row.order_id)
        || has_duplicate(&state.freight, |row| row.lot_id)
        || dispatch_ids.len() != state.freight.len()
        || has_duplicate(&state.corridor_capacities, |row| {
            (row.period, row.corridor_id, row.unit_id)
        });
    if duplicate {
        return Err(MaterialCircuitErrorV2::DuplicateRow);
    }
    Ok(())
}

fn route_legs(state: &MaterialCircuitStateV2, route: RouteIdV2) -> &[RouteLegV2] {
    let start = state.route_legs.partition_point(|row| row.route_id < route);
    let end = state
        .route_legs
        .partition_point(|row| row.route_id <= route);
    &state.route_legs[start..end]
}

fn site_node(state: &MaterialCircuitStateV2, site: SiteIdV1) -> Option<crate::LogisticsNodeIdV2> {
    state
        .site_logistics_nodes
        .binary_search_by_key(&site, |row| row.site_id)
        .ok()
        .map(|index| state.site_logistics_nodes[index].node_id)
}

fn validate_route_legs(legs: &[RouteLegV2]) -> Result<(), MaterialCircuitErrorV2> {
    if legs.is_empty() || legs.len() > MAX_ROUTE_LEGS_PER_ROUTE_V2 {
        return Err(MaterialCircuitErrorV2::RouteInvariant);
    }
    for (index, leg) in legs
        .iter()
        .enumerate()
        .take(MAX_ROUTE_LEGS_PER_ROUTE_V2 + 1)
    {
        if usize::from(leg.leg_index) != index
            || leg.travel_periods == 0
            || leg.loss_ppm > FREIGHT_LOSS_PARTS_PER_MILLION_V2
        {
            return Err(MaterialCircuitErrorV2::RouteInvariant);
        }
        if index > 0 && legs[index - 1].to_node_id != leg.from_node_id {
            return Err(MaterialCircuitErrorV2::RouteInvariant);
        }
    }
    Ok(())
}

fn validate_routes(state: &MaterialCircuitStateV2) -> Result<(), MaterialCircuitErrorV2> {
    let route_ids: BTreeSet<_> = state
        .route_legs
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|row| row.route_id)
        .collect();
    let corridor_ids: BTreeSet<_> = state
        .route_legs
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|row| row.corridor_id)
        .collect();
    for route in route_ids.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        validate_route_legs(route_legs(state, *route))?;
    }
    for supplier in state
        .supplier_routes
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        let legs = route_legs(state, supplier.route_id);
        validate_route_legs(legs)?;
        if site_node(state, supplier.supplier_site_id) != Some(legs[0].from_node_id)
            || site_node(state, supplier.buyer_site_id) != Some(legs[legs.len() - 1].to_node_id)
        {
            return Err(MaterialCircuitErrorV2::RouteInvariant);
        }
    }
    if state
        .corridor_capacities
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .any(|row| !corridor_ids.contains(&row.corridor_id))
    {
        return Err(MaterialCircuitErrorV2::CapacityInvariant);
    }
    Ok(())
}

fn order_index(state: &MaterialCircuitStateV2, order: OrderIdV1) -> Option<usize> {
    state
        .orders
        .binary_search_by_key(&order, |row| row.order_id)
        .ok()
}

fn supplier_routes(state: &MaterialCircuitStateV2) -> BTreeMap<SupplierKey, RouteIdV2> {
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
                row.route_id,
            )
        })
        .collect()
}

fn expected_leg_arrival(
    lot: &RoutedFreightLotV2,
    legs: &[RouteLegV2],
) -> Result<u64, MaterialCircuitErrorV2> {
    legs.iter()
        .take(usize::from(lot.current_leg_index) + 1)
        .try_fold(lot.dispatch_period, |period, leg| {
            period
                .checked_add(u64::from(leg.travel_periods))
                .ok_or(MaterialCircuitErrorV2::Arithmetic)
        })
}

fn validate_orders_and_freight(
    state: &MaterialCircuitStateV2,
) -> Result<(), MaterialCircuitErrorV2> {
    if state.orders.len() != state.backlog.len() {
        return Err(MaterialCircuitErrorV2::BacklogInvariant);
    }
    let routes = supplier_routes(state);
    let mut in_transit = BTreeMap::<OrderIdV1, u128>::new();
    for lot in state.freight.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        let Some(index) = order_index(state, lot.order_id) else {
            return Err(MaterialCircuitErrorV2::FreightInvariant);
        };
        let order = &state.orders[index];
        let supplier_key = (
            order.buyer_site_id,
            order.supplier_site_id,
            order.good_id,
            order.unit_id,
        );
        let legs = route_legs(state, lot.route_id);
        if lot.quantity == 0
            || lot.lot_id != freight_lot_id(lot.order_id, lot.dispatch_period)
            || lot.dispatch_period >= state.period
            || lot.leg_arrival_period < state.period
            || usize::from(lot.current_leg_index) >= legs.len()
            || routes.get(&supplier_key) != Some(&lot.route_id)
            || lot.source_site_id != order.supplier_site_id
            || lot.destination_site_id != order.buyer_site_id
            || lot.good_id != order.good_id
            || lot.unit_id != order.unit_id
        {
            return Err(MaterialCircuitErrorV2::FreightInvariant);
        }
        if expected_leg_arrival(lot, legs)? != lot.leg_arrival_period {
            return Err(MaterialCircuitErrorV2::FreightInvariant);
        }
        let total = in_transit.entry(lot.order_id).or_default();
        *total = total
            .checked_add(u128::from(lot.quantity))
            .ok_or(MaterialCircuitErrorV2::Arithmetic)?;
    }
    for (order, backlog) in state
        .orders
        .iter()
        .zip(&state.backlog)
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        if order.ordered == 0 {
            return Err(MaterialCircuitErrorV2::ZeroQuantity);
        }
        let accounted = order
            .delivered
            .checked_add(order.lost)
            .ok_or(MaterialCircuitErrorV2::Arithmetic)?;
        if order.realized > order.delivered
            || accounted > order.shipped
            || order.shipped > order.ordered
        {
            return Err(MaterialCircuitErrorV2::OrderInvariant);
        }
        if backlog.order_id != order.order_id || backlog.quantity != order.ordered - order.shipped {
            return Err(MaterialCircuitErrorV2::BacklogInvariant);
        }
        if in_transit.get(&order.order_id).copied().unwrap_or(0)
            != u128::from(order.shipped - accounted)
        {
            return Err(MaterialCircuitErrorV2::FreightInvariant);
        }
    }
    Ok(())
}

fn production_state(state: &MaterialCircuitStateV2) -> MaterialCircuitStateV1 {
    MaterialCircuitStateV1 {
        period: state.period,
        process_outputs: state.process_outputs.clone(),
        input_coefficients: state.input_coefficients.clone(),
        labor_coefficients: state.labor_coefficients.clone(),
        supplier_candidates: Vec::new(),
        inventory: state.inventory.clone(),
        orders: Vec::new(),
        backlog: Vec::new(),
        transit: Vec::new(),
        capacities: state.capacities.clone(),
        labor: state.labor.clone(),
        production_commitments: state.production_commitments.clone(),
    }
}

fn merge_production_state(state: &mut MaterialCircuitStateV2, production: MaterialCircuitStateV1) {
    state.inventory = production.inventory;
    state.capacities = production.capacities;
    state.labor = production.labor;
    state.production_commitments = production.production_commitments;
}

pub(crate) fn canonical_state_v2(
    state: &MaterialCircuitStateV2,
) -> Result<MaterialCircuitStateV2, MaterialCircuitErrorV2> {
    check_row_limits(state)?;
    let mut canonical = state.clone();
    canonicalize_rows(&mut canonical);
    validate_unique_rows(&canonical)?;
    validate_routes(&canonical)?;
    validate_orders_and_freight(&canonical)?;
    canonical_state_v1(&production_state(&canonical)).map_err(MaterialCircuitErrorV2::from)?;
    if canonical.period == 0
        || canonical
            .corridor_capacities
            .iter()
            .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
            .any(|row| row.period < canonical.period)
    {
        return Err(MaterialCircuitErrorV2::PeriodInvariant);
    }
    Ok(canonical)
}

fn take_inventory(state: &mut MaterialCircuitStateV2) -> InventoryLedger {
    std::mem::take(&mut state.inventory)
        .into_iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|row| ((row.site_id, row.good_id, row.unit_id), row.quantity))
        .collect()
}

fn publish_inventory(state: &mut MaterialCircuitStateV2, inventory: InventoryLedger) {
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
) -> Result<(), MaterialCircuitErrorV2> {
    if quantity == 0 {
        return Ok(());
    }
    if let Some(current) = inventory.get_mut(&key) {
        *current = current
            .checked_add(quantity)
            .ok_or(MaterialCircuitErrorV2::Arithmetic)?;
        return Ok(());
    }
    if inventory.len() == MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        return Err(MaterialCircuitErrorV2::RowLimit);
    }
    inventory.insert(key, quantity);
    Ok(())
}

fn debit_inventory(
    inventory: &mut InventoryLedger,
    key: InventoryKey,
    quantity: u64,
) -> Result<(), MaterialCircuitErrorV2> {
    if quantity == 0 {
        return Ok(());
    }
    let current = inventory
        .get_mut(&key)
        .ok_or(MaterialCircuitErrorV2::FreightInvariant)?;
    *current = current
        .checked_sub(quantity)
        .ok_or(MaterialCircuitErrorV2::Arithmetic)?;
    Ok(())
}

fn loss_quantity(quantity: u64, loss_ppm: u32) -> Result<u64, MaterialCircuitErrorV2> {
    let loss = u128::from(quantity)
        .checked_mul(u128::from(loss_ppm))
        .ok_or(MaterialCircuitErrorV2::Arithmetic)?
        / u128::from(FREIGHT_LOSS_PARTS_PER_MILLION_V2);
    u64::try_from(loss).map_err(|_| MaterialCircuitErrorV2::Arithmetic)
}

fn process_due_freight(
    state: &mut MaterialCircuitStateV2,
    inventory: &mut InventoryLedger,
    losses: &mut Vec<FreightLossReceiptV2>,
    arrivals: &mut Vec<ArrivalReceiptV1>,
    deliveries: &mut Vec<DeliveryReceiptV1>,
    realizations: &mut Vec<RealizationReceiptV1>,
) -> Result<(), MaterialCircuitErrorV2> {
    let opening = std::mem::take(&mut state.freight);
    let mut remaining = Vec::with_capacity(opening.len());
    for mut lot in opening.into_iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        if lot.leg_arrival_period != state.period {
            remaining.push(lot);
            continue;
        }
        let index = usize::from(lot.current_leg_index);
        let (corridor_id, loss_ppm, next_leg) = {
            let legs = route_legs(state, lot.route_id);
            let leg = &legs[index];
            let next_leg = legs
                .get(index + 1)
                .map(|next| (next.leg_index, next.travel_periods));
            (leg.corridor_id, leg.loss_ppm, next_leg)
        };
        let lost = loss_quantity(lot.quantity, loss_ppm)?;
        let retained = lot
            .quantity
            .checked_sub(lost)
            .ok_or(MaterialCircuitErrorV2::Arithmetic)?;
        let order_index =
            order_index(state, lot.order_id).ok_or(MaterialCircuitErrorV2::FreightInvariant)?;
        state.orders[order_index].lost = state.orders[order_index]
            .lost
            .checked_add(lost)
            .ok_or(MaterialCircuitErrorV2::Arithmetic)?;
        if lost > 0 {
            losses.push(FreightLossReceiptV2 {
                lot_id: lot.lot_id,
                order_id: lot.order_id,
                corridor_id,
                quantity: lost,
            });
        }
        if let Some((next_leg_index, next_travel_periods)) = next_leg.filter(|_| retained > 0) {
            lot.current_leg_index = next_leg_index;
            lot.leg_arrival_period = state
                .period
                .checked_add(u64::from(next_travel_periods))
                .ok_or(MaterialCircuitErrorV2::Arithmetic)?;
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
                .ok_or(MaterialCircuitErrorV2::Arithmetic)?;
            order.realized = order
                .realized
                .checked_add(retained)
                .ok_or(MaterialCircuitErrorV2::Arithmetic)?;
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

fn capacity_index(state: &MaterialCircuitStateV2, key: CapacityKey) -> Option<usize> {
    state
        .corridor_capacities
        .binary_search_by_key(&key, |row| (row.period, row.corridor_id, row.unit_id))
        .ok()
}

fn add_request(
    groups: &mut BTreeMap<FreightResourceKey, Vec<FreightRequest>>,
    key: FreightResourceKey,
    order_index: usize,
    requested: u64,
) {
    groups.entry(key).or_default().push(FreightRequest {
        order_index,
        requested,
    });
}

fn resource_groups(
    state: &MaterialCircuitStateV2,
    routes: &BTreeMap<SupplierKey, RouteIdV2>,
) -> Result<BTreeMap<FreightResourceKey, Vec<FreightRequest>>, MaterialCircuitErrorV2> {
    let mut groups = BTreeMap::new();
    for (index, order) in state
        .orders
        .iter()
        .enumerate()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        let requested = order.ordered - order.shipped;
        let key = (
            order.buyer_site_id,
            order.supplier_site_id,
            order.good_id,
            order.unit_id,
        );
        let Some(route) = routes.get(&key) else {
            continue;
        };
        if requested == 0 {
            continue;
        }
        add_request(
            &mut groups,
            FreightResourceKey::Inventory((order.supplier_site_id, order.good_id, order.unit_id)),
            index,
            requested,
        );
        let mut departure_period = state.period;
        for leg in route_legs(state, *route)
            .iter()
            .take(MAX_ROUTE_LEGS_PER_ROUTE_V2 + 1)
        {
            add_request(
                &mut groups,
                FreightResourceKey::Corridor((departure_period, leg.corridor_id, order.unit_id)),
                index,
                requested,
            );
            departure_period = departure_period
                .checked_add(u64::from(leg.travel_periods))
                .ok_or(MaterialCircuitErrorV2::Arithmetic)?;
        }
    }
    ensure_resource_group_count(groups.len())?;
    Ok(groups)
}

fn ensure_resource_group_count(count: usize) -> Result<(), MaterialCircuitErrorV2> {
    if count > MAX_FREIGHT_RESOURCE_GROUPS_V2 {
        Err(MaterialCircuitErrorV2::RowLimit)
    } else {
        Ok(())
    }
}

fn resource_available(
    state: &MaterialCircuitStateV2,
    inventory: &InventoryLedger,
    key: FreightResourceKey,
) -> u64 {
    match key {
        FreightResourceKey::Inventory(inventory_key) => {
            inventory.get(&inventory_key).copied().unwrap_or(0)
        }
        FreightResourceKey::Corridor(capacity_key) => capacity_index(state, capacity_key)
            .map_or(0, |index| state.corridor_capacities[index].available),
    }
}

fn order_allocations(
    state: &MaterialCircuitStateV2,
    inventory: &InventoryLedger,
    groups: &BTreeMap<FreightResourceKey, Vec<FreightRequest>>,
) -> Result<Vec<u64>, MaterialCircuitErrorV2> {
    let mut allocations = vec![0_u64; state.orders.len()];
    for requests in groups.values().take(MAX_FREIGHT_RESOURCE_GROUPS_V2) {
        for request in requests.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
            allocations[request.order_index] = request.requested;
        }
    }
    for (key, requests) in groups.iter().take(MAX_FREIGHT_RESOURCE_GROUPS_V2) {
        let total = requests
            .iter()
            .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
            .try_fold(0_u128, |sum, request| {
                sum.checked_add(u128::from(request.requested))
                    .ok_or(MaterialCircuitErrorV2::Arithmetic)
            })?;
        let available = resource_available(state, inventory, *key);
        for request in requests.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
            let grant = if u128::from(available) >= total {
                request.requested
            } else {
                proportional_floor(available, u128::from(request.requested), total)
                    .map_err(MaterialCircuitErrorV2::from)?
            };
            allocations[request.order_index] = allocations[request.order_index].min(grant);
        }
    }
    Ok(allocations)
}

fn freight_lot_id(order: OrderIdV1, period: u64) -> FreightLotIdV2 {
    let mut bytes = b"babylon.freight-lot.v2\0".to_vec();
    bytes.extend_from_slice(&order.as_bytes());
    bytes.extend_from_slice(&period.to_be_bytes());
    FreightLotIdV2::from_bytes(sha256_of(&bytes))
}

fn reserve_route_capacity(
    state: &mut MaterialCircuitStateV2,
    route: RouteIdV2,
    unit: UnitIdV1,
    quantity: u64,
) -> Result<u64, MaterialCircuitErrorV2> {
    let mut departure_period = state.period;
    let mut final_arrival_period = state.period;
    let legs = route_legs(state, route).to_vec();
    for leg in legs.iter().take(MAX_ROUTE_LEGS_PER_ROUTE_V2 + 1) {
        let key = (departure_period, leg.corridor_id, unit);
        let index = capacity_index(state, key).ok_or(MaterialCircuitErrorV2::CapacityInvariant)?;
        state.corridor_capacities[index].available = state.corridor_capacities[index]
            .available
            .checked_sub(quantity)
            .ok_or(MaterialCircuitErrorV2::Arithmetic)?;
        final_arrival_period = departure_period
            .checked_add(u64::from(leg.travel_periods))
            .ok_or(MaterialCircuitErrorV2::Arithmetic)?;
        departure_period = final_arrival_period;
    }
    Ok(final_arrival_period)
}

fn apply_dispatches(
    state: &mut MaterialCircuitStateV2,
    inventory: &mut InventoryLedger,
    routes: &BTreeMap<SupplierKey, RouteIdV2>,
    allocations: &[u64],
    receipts: &mut Vec<RoutedDispatchReceiptV2>,
) -> Result<(), MaterialCircuitErrorV2> {
    for (index, quantity) in allocations
        .iter()
        .copied()
        .enumerate()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        if quantity == 0 {
            continue;
        }
        let order = state.orders[index].clone();
        let supplier_key = (
            order.buyer_site_id,
            order.supplier_site_id,
            order.good_id,
            order.unit_id,
        );
        let route = routes[&supplier_key];
        let legs = route_legs(state, route);
        let first_arrival_period = state
            .period
            .checked_add(u64::from(legs[0].travel_periods))
            .ok_or(MaterialCircuitErrorV2::Arithmetic)?;
        debit_inventory(
            inventory,
            (order.supplier_site_id, order.good_id, order.unit_id),
            quantity,
        )?;
        let final_arrival_period = reserve_route_capacity(state, route, order.unit_id, quantity)?;
        state.orders[index].shipped = state.orders[index]
            .shipped
            .checked_add(quantity)
            .ok_or(MaterialCircuitErrorV2::Arithmetic)?;
        let lot_id = freight_lot_id(order.order_id, state.period);
        state.freight.push(RoutedFreightLotV2 {
            lot_id,
            order_id: order.order_id,
            route_id: route,
            dispatch_period: state.period,
            current_leg_index: 0,
            leg_arrival_period: first_arrival_period,
            source_site_id: order.supplier_site_id,
            destination_site_id: order.buyer_site_id,
            good_id: order.good_id,
            unit_id: order.unit_id,
            quantity,
        });
        receipts.push(RoutedDispatchReceiptV2 {
            lot_id,
            order_id: order.order_id,
            route_id: route,
            quantity,
            final_arrival_period,
        });
    }
    Ok(())
}

fn dispatch_orders(
    state: &mut MaterialCircuitStateV2,
    inventory: &mut InventoryLedger,
    receipts: &mut Vec<RoutedDispatchReceiptV2>,
) -> Result<(), MaterialCircuitErrorV2> {
    let routes = supplier_routes(state);
    let groups = resource_groups(state, &routes)?;
    let allocations = order_allocations(state, inventory, &groups)?;
    apply_dispatches(state, inventory, &routes, &allocations, receipts)
}

fn rebuild_backlog(state: &mut MaterialCircuitStateV2) {
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
    state: &mut MaterialCircuitStateV2,
) -> Result<Vec<crate::ProductionReceiptV1>, MaterialCircuitErrorV2> {
    let mut production = production_state(state);
    let receipts =
        execute_shared_production_v1(&mut production).map_err(MaterialCircuitErrorV2::from)?;
    merge_production_state(state, production);
    Ok(receipts)
}

fn derive_next_production(
    state: &mut MaterialCircuitStateV2,
    next_period: u64,
) -> Result<(), MaterialCircuitErrorV2> {
    let mut production = production_state(state);
    derive_shared_production_v1(&mut production, next_period)
        .map_err(MaterialCircuitErrorV2::from)?;
    merge_production_state(state, production);
    Ok(())
}

fn prune_corridor_capacity(state: &mut MaterialCircuitStateV2, next_period: u64) {
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
pub struct ClosedMaterialPeriodV2 {
    transition: MaterialCircuitTransitionV2,
    next_period: u64,
}

impl ClosedMaterialPeriodV2 {
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

    /// Request next-opening work from shared inputs and process capacity only.
    ///
    /// Every process has one request, including zero. The request's `period` is
    /// this closing interval, as required by staffing; its work is for the next
    /// opening. Neither current employment nor any scheduled hours limit it.
    ///
    /// # Errors
    /// Refuses incomplete, duplicate or foreign pool bindings, row bounds and
    /// hours that cannot be represented exactly as `u64`.
    pub fn staffing_requests(
        &self,
        bindings: &[StaffingPoolBindingV1],
    ) -> Result<Vec<StaffingWorkRequestV1>, MaterialCircuitErrorV2> {
        let owners = staffing_process_owners(bindings)?;
        let requests = derive_shared_labor_requests_v1(
            &production_state(&self.transition.state),
            self.next_period,
        )?;
        if owners.len() != requests.len() {
            return Err(MaterialCircuitErrorV2::ProcessInvariant);
        }
        requests
            .into_iter()
            .map(|request| {
                let binding = owners
                    .get(&request.process_id)
                    .ok_or(MaterialCircuitErrorV2::ProcessInvariant)?;
                if binding.site_id() != request.site_id || binding.unit_id() != request.unit_id {
                    return Err(MaterialCircuitErrorV2::ProcessInvariant);
                }
                Ok(StaffingWorkRequestV1::new(
                    self.closing_period(),
                    binding.pool_id(),
                    request.process_id,
                    request.site_id,
                    request.unit_id,
                    request.hours,
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
    ) -> Result<MaterialCircuitTransitionV2, MaterialCircuitErrorV2> {
        validate_next_labor(&self.transition.state, self.next_period, &next_labor)?;
        // The allocator performs binary searches before final canonicalization.
        next_labor.sort_unstable_by_key(|row| (row.period, row.site_id, row.unit_id));
        self.transition.state.labor = next_labor;
        self.finish()
    }

    fn finish(mut self) -> Result<MaterialCircuitTransitionV2, MaterialCircuitErrorV2> {
        let state = &mut self.transition.state;
        derive_next_production(state, self.next_period)?;
        prune_corridor_capacity(state, self.next_period);
        state.period = self.next_period;
        *state = canonical_state_v2(state)?;
        Ok(self.transition)
    }
}

fn staffing_process_owners(
    bindings: &[StaffingPoolBindingV1],
) -> Result<BTreeMap<crate::ProcessIdV1, &StaffingPoolBindingV1>, MaterialCircuitErrorV2> {
    if bindings.len() > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        return Err(MaterialCircuitErrorV2::RowLimit);
    }
    let mut owners = BTreeMap::new();
    let mut pools = BTreeSet::new();
    let mut principals = BTreeSet::new();
    for binding in bindings {
        if !pools.insert(binding.pool_id())
            || !principals.insert((binding.site_id(), binding.unit_id()))
        {
            return Err(MaterialCircuitErrorV2::DuplicateRow);
        }
        for process in binding.processes() {
            if owners.insert(*process, binding).is_some() {
                return Err(MaterialCircuitErrorV2::DuplicateRow);
            }
            if owners.len() > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
                return Err(MaterialCircuitErrorV2::RowLimit);
            }
        }
    }
    Ok(owners)
}

fn validate_next_labor(
    state: &MaterialCircuitStateV2,
    next_period: u64,
    rows: &[LaborCapacityRowV1],
) -> Result<(), MaterialCircuitErrorV2> {
    if rows.len() > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        return Err(MaterialCircuitErrorV2::RowLimit);
    }
    // The detached close preserves the checked, process-sorted recipe roster.
    let expected: BTreeSet<_> = state
        .process_outputs
        .iter()
        .zip(&state.labor_coefficients)
        .map(|(output, coefficient)| (output.site_id, coefficient.unit_id))
        .collect();
    let mut actual = BTreeSet::new();
    for row in rows {
        if row.period != next_period {
            return Err(MaterialCircuitErrorV2::PeriodInvariant);
        }
        if !actual.insert((row.site_id, row.unit_id)) {
            return Err(MaterialCircuitErrorV2::DuplicateRow);
        }
    }
    if actual != expected {
        return Err(MaterialCircuitErrorV2::CapacityInvariant);
    }
    Ok(())
}

/// Close one routed period atomically and return its canonical successor state.
///
/// # Errors
/// Returns the first exact schema, route, conservation, bound, or arithmetic refusal.
pub fn advance_material_circuit_v2(
    opening: &MaterialCircuitStateV2,
) -> Result<MaterialCircuitTransitionV2, MaterialCircuitErrorV2> {
    close_material_period_v2(opening)?.finish()
}

/// Execute due freight, prior production commitments and dispatch exactly once.
///
/// The result borrows no mutable opening state and cannot become a world
/// register until next-opening labor and normal planning have been resolved.
///
/// # Errors
/// Returns the same schema, route, conservation, bound or arithmetic refusals
/// as the one-shot transition, leaving the opening state unchanged.
pub fn close_material_period_v2(
    opening: &MaterialCircuitStateV2,
) -> Result<ClosedMaterialPeriodV2, MaterialCircuitErrorV2> {
    let mut state = canonical_state_v2(opening)?;
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
    dispatch_orders(&mut state, &mut inventory, &mut dispatches)?;
    rebuild_backlog(&mut state);
    publish_inventory(&mut state, inventory);
    let next_period = state
        .period
        .checked_add(1)
        .ok_or(MaterialCircuitErrorV2::Arithmetic)?;
    Ok(ClosedMaterialPeriodV2 {
        next_period,
        transition: MaterialCircuitTransitionV2 {
            state,
            production,
            dispatches,
            losses,
            arrivals,
            deliveries,
            realizations,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::ensure_resource_group_count;
    use crate::{MaterialCircuitErrorV2, MAX_FREIGHT_RESOURCE_GROUPS_V2};

    #[test]
    fn resource_group_ceiling_accepts_maximum_and_refuses_plus_one() {
        assert_eq!(
            ensure_resource_group_count(MAX_FREIGHT_RESOURCE_GROUPS_V2),
            Ok(())
        );
        assert_eq!(
            ensure_resource_group_count(MAX_FREIGHT_RESOURCE_GROUPS_V2 + 1),
            Err(MaterialCircuitErrorV2::RowLimit)
        );
    }
}
