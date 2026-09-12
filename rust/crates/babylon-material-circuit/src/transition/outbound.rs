//! One shared native-unit resource allocator for routed and local outbound work.

use super::merchant_admission::{hours_per_unit, merchant};
use super::{
    capacity_index, credit_inventory, debit_inventory, freight_lot_id, grams_per_unit,
    route_stages, stage_capacities, supplier_routes, BTreeMap, CapacityKey, InventoryKey,
    InventoryLedger, MaterialCircuitError, MaterialCircuitState, RouteId, RoutedDispatchReceipt,
    RoutedFreightLot, SiteId, SupplierKey, SupplyPath, UnitId, MAX_MATERIAL_CIRCUIT_ROWS,
};
use crate::production::proportional_floor;
use crate::{
    LocalRetailFulfillmentReceipt, LocalTransferReceipt, MerchantHandlingReceipt, OutboundOrderId,
    SupplierTransport, MAX_FREIGHT_RESOURCE_REQUESTS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FreightResourceKey {
    Inventory(InventoryKey),
    Corridor(CapacityKey),
    Labor(SiteId, UnitId),
}

#[derive(Debug, Clone, Copy)]
struct FreightRequest {
    order_index: usize,
    requested: u64,
    resource_per_unit: u64,
}

fn add_request(
    groups: &mut BTreeMap<FreightResourceKey, Vec<FreightRequest>>,
    count: &mut usize,
    key: FreightResourceKey,
    order_index: usize,
    requested: u64,
    resource_per_unit: u64,
) -> Result<(), MaterialCircuitError> {
    *count = count
        .checked_add(1)
        .ok_or(MaterialCircuitError::Arithmetic)?;
    ensure_resource_group_count(*count)?;
    groups.entry(key).or_default().push(FreightRequest {
        order_index,
        requested,
        resource_per_unit,
    });
    Ok(())
}

#[derive(Debug)]
struct OutboundOrder {
    id: OutboundOrderId,
    stock: InventoryKey,
    requested: u64,
    route: Option<RouteId>,
    eligible: bool,
}

fn outbound_orders(
    state: &MaterialCircuitState,
    routes: &BTreeMap<SupplierKey, SupplyPath>,
) -> Vec<OutboundOrder> {
    let shipments = state.orders.iter().map(|row| {
        let route = routes
            .get(&(
                row.buyer_site_id,
                row.supplier_site_id,
                row.good_id,
                row.unit_id,
            ))
            .map(|(route, _)| *route);
        OutboundOrder {
            id: OutboundOrderId::Delivery(row.order_id),
            stock: (row.supplier_site_id, row.good_id, row.unit_id),
            requested: row.ordered - row.shipped,
            eligible: route.is_some(),
            route,
        }
    });
    let local = state.final_demand_orders.iter().map(|row| OutboundOrder {
        id: OutboundOrderId::LocalFinalDemand(row.order_id),
        stock: (row.retailer_site_id, row.good_id, row.unit_id),
        requested: row.ordered - row.fulfilled,
        route: None,
        eligible: true,
    });
    shipments.chain(local).collect()
}

fn add_route_requests(
    state: &MaterialCircuitState,
    route: RouteId,
    request: FreightRequest,
    groups: &mut BTreeMap<FreightResourceKey, Vec<FreightRequest>>,
    count: &mut usize,
) -> Result<(), MaterialCircuitError> {
    let mut departure_period = state.period;
    for stage in route_stages(state, route) {
        for capacity in stage_capacities(state, route, stage.stage_index) {
            add_request(
                groups,
                count,
                FreightResourceKey::Corridor((departure_period, capacity.corridor_id)),
                request.order_index,
                request.requested,
                request.resource_per_unit,
            )?;
        }
        departure_period = departure_period
            .checked_add(u64::from(stage.travel_periods))
            .ok_or(MaterialCircuitError::Arithmetic)?;
    }
    Ok(())
}

fn resource_groups(
    state: &MaterialCircuitState,
    orders: &[OutboundOrder],
) -> Result<BTreeMap<FreightResourceKey, Vec<FreightRequest>>, MaterialCircuitError> {
    let mut groups = BTreeMap::new();
    let mut count = 0;
    for (index, order) in orders.iter().enumerate() {
        if !order.eligible || order.requested == 0 {
            continue;
        }
        add_request(
            &mut groups,
            &mut count,
            FreightResourceKey::Inventory(order.stock),
            index,
            order.requested,
            1,
        )?;
        let grams = grams_per_unit(state, order.stock.1, order.stock.2)?;
        if let Some(route) = order.route {
            add_route_requests(
                state,
                route,
                FreightRequest {
                    order_index: index,
                    requested: order.requested,
                    resource_per_unit: grams,
                },
                &mut groups,
                &mut count,
            )?;
        }
        if let Some(merchant) = merchant(state, order.stock.0) {
            add_request(
                &mut groups,
                &mut count,
                FreightResourceKey::Corridor((state.period, merchant.capacity_id)),
                index,
                order.requested,
                grams,
            )?;
        }
    }
    Ok(groups)
}

pub(super) fn ensure_resource_group_count(count: usize) -> Result<(), MaterialCircuitError> {
    if count > MAX_FREIGHT_RESOURCE_REQUESTS {
        Err(MaterialCircuitError::RowLimit)
    } else {
        Ok(())
    }
}

fn resource_available(
    state: &MaterialCircuitState,
    inventory: &InventoryLedger,
    key: FreightResourceKey,
) -> u64 {
    match key {
        FreightResourceKey::Inventory(inventory_key) => {
            inventory.get(&inventory_key).copied().unwrap_or(0)
        }
        FreightResourceKey::Corridor(capacity_key) => capacity_index(state, capacity_key)
            .map_or(0, |index| state.corridor_capacities[index].available_grams),
        FreightResourceKey::Labor(site, unit) => {
            labor_index(state, site, unit).map_or(0, |index| state.labor[index].available)
        }
    }
}

fn order_allocations(
    state: &MaterialCircuitState,
    inventory: &InventoryLedger,
    groups: &BTreeMap<FreightResourceKey, Vec<FreightRequest>>,
    order_count: usize,
) -> Result<Vec<u64>, MaterialCircuitError> {
    let mut allocations = vec![0_u64; order_count];
    // Each active order has exactly one inventory request, regardless of the
    // number of capacity principals on its preselected route.
    for (key, requests) in groups {
        if matches!(key, FreightResourceKey::Inventory(_)) {
            for request in requests {
                allocations[request.order_index] = request.requested;
            }
        }
    }
    limit_allocations(state, inventory, groups, &mut allocations)?;
    Ok(allocations)
}

fn limit_allocations(
    state: &MaterialCircuitState,
    inventory: &InventoryLedger,
    groups: &BTreeMap<FreightResourceKey, Vec<FreightRequest>>,
    allocations: &mut [u64],
) -> Result<(), MaterialCircuitError> {
    for (key, requests) in groups {
        let total = requests.iter().try_fold(0_u128, |sum, request| {
            let requested = u128::from(request.requested) * u128::from(request.resource_per_unit);
            sum.checked_add(requested)
                .ok_or(MaterialCircuitError::Arithmetic)
        })?;
        let available = resource_available(state, inventory, *key);
        for request in requests {
            let resource_request =
                u128::from(request.requested) * u128::from(request.resource_per_unit);
            let grant = if u128::from(available) >= total {
                request.requested
            } else {
                proportional_floor(available, resource_request, total)? / request.resource_per_unit
            };
            allocations[request.order_index] = allocations[request.order_index].min(grant);
        }
    }
    Ok(())
}

fn reserve_route_capacity(
    state: &mut MaterialCircuitState,
    route: RouteId,
    grams: u64,
) -> Result<u64, MaterialCircuitError> {
    let mut departure_period = state.period;
    for stage in route_stages(state, route).to_vec() {
        let capacities = stage_capacities(state, route, stage.stage_index).to_vec();
        for capacity in capacities {
            let index = capacity_index(state, (departure_period, capacity.corridor_id))
                .ok_or(MaterialCircuitError::CapacityInvariant)?;
            state.corridor_capacities[index].available_grams = state.corridor_capacities[index]
                .available_grams
                .checked_sub(grams)
                .ok_or(MaterialCircuitError::Arithmetic)?;
        }
        departure_period = departure_period
            .checked_add(u64::from(stage.travel_periods))
            .ok_or(MaterialCircuitError::Arithmetic)?;
    }
    Ok(departure_period)
}

fn apply_dispatches(
    state: &mut MaterialCircuitState,
    inventory: &mut InventoryLedger,
    routes: &BTreeMap<SupplierKey, SupplyPath>,
    allocations: &[u64],
    receipts: &mut Vec<RoutedDispatchReceipt>,
) -> Result<Vec<LocalTransferReceipt>, MaterialCircuitError> {
    let mut local_transfers = Vec::new();
    for (index, quantity) in allocations
        .iter()
        .copied()
        .enumerate()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
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
        let (route, mode) = routes[&supplier_key];
        let local = mode == SupplierTransport::Local;
        if local {
            debit_inventory(
                inventory,
                (order.supplier_site_id, order.good_id, order.unit_id),
                quantity,
                MaterialCircuitError::FreightInvariant,
            )?;
            let updated = &mut state.orders[index];
            updated.shipped = updated
                .shipped
                .checked_add(quantity)
                .ok_or(MaterialCircuitError::Arithmetic)?;
            updated.delivered = updated
                .delivered
                .checked_add(quantity)
                .ok_or(MaterialCircuitError::Arithmetic)?;
            updated.realized = updated
                .realized
                .checked_add(quantity)
                .ok_or(MaterialCircuitError::Arithmetic)?;
            local_transfers.push(LocalTransferReceipt {
                order_id: order.order_id,
                supplier_site_id: order.supplier_site_id,
                buyer_site_id: order.buyer_site_id,
                good_id: order.good_id,
                unit_id: order.unit_id,
                quantity,
            });
            continue;
        }
        let legs = route_stages(state, route);
        let first_arrival_period = state
            .period
            .checked_add(u64::from(legs[0].travel_periods))
            .ok_or(MaterialCircuitError::Arithmetic)?;
        debit_inventory(
            inventory,
            (order.supplier_site_id, order.good_id, order.unit_id),
            quantity,
            MaterialCircuitError::FreightInvariant,
        )?;
        let grams = quantity
            .checked_mul(grams_per_unit(state, order.good_id, order.unit_id)?)
            .ok_or(MaterialCircuitError::Arithmetic)?;
        let final_arrival_period = reserve_route_capacity(state, route, grams)?;
        state.orders[index].shipped = state.orders[index]
            .shipped
            .checked_add(quantity)
            .ok_or(MaterialCircuitError::Arithmetic)?;
        let lot_id = freight_lot_id(order.order_id, state.period);
        state.freight.push(RoutedFreightLot {
            lot_id,
            order_id: order.order_id,
            route_id: route,
            dispatch_period: state.period,
            current_stage_index: 0,
            stage_arrival_period: first_arrival_period,
            source_site_id: order.supplier_site_id,
            destination_site_id: order.buyer_site_id,
            good_id: order.good_id,
            unit_id: order.unit_id,
            quantity,
        });
        receipts.push(RoutedDispatchReceipt {
            lot_id,
            order_id: order.order_id,
            route_id: route,
            quantity,
            final_arrival_period,
        });
    }
    Ok(local_transfers)
}

fn labor_index(state: &MaterialCircuitState, site: SiteId, unit: UnitId) -> Option<usize> {
    state
        .labor
        .binary_search_by_key(&(state.period, site, unit), |row| {
            (row.period, row.site_id, row.unit_id)
        })
        .ok()
}

fn labor_groups(
    state: &MaterialCircuitState,
    orders: &[OutboundOrder],
    feasible: &[u64],
    existing_requests: usize,
) -> Result<BTreeMap<FreightResourceKey, Vec<FreightRequest>>, MaterialCircuitError> {
    let mut groups = BTreeMap::new();
    let mut count = existing_requests;
    for (index, (order, quantity)) in orders.iter().zip(feasible).enumerate() {
        if let Some(merchant) = merchant(state, order.stock.0) {
            let hours = hours_per_unit(state, order.stock.0, order.stock.1, order.stock.2)?;
            add_request(
                &mut groups,
                &mut count,
                FreightResourceKey::Labor(merchant.site_id, merchant.labor_unit_id),
                index,
                *quantity,
                hours,
            )?;
        }
    }
    Ok(groups)
}

fn apply_handling(
    state: &mut MaterialCircuitState,
    orders: &[OutboundOrder],
    feasible: &[u64],
    actual: &[u64],
) -> Result<Vec<MerchantHandlingReceipt>, MaterialCircuitError> {
    let mut receipts = Vec::new();
    for ((order, feasible_quantity), handled_quantity) in orders.iter().zip(feasible).zip(actual) {
        let Some(merchant) = merchant(state, order.stock.0).cloned() else {
            continue;
        };
        let hours = hours_per_unit(state, order.stock.0, order.stock.1, order.stock.2)?;
        let needed_hours = feasible_quantity
            .checked_mul(hours)
            .ok_or(MaterialCircuitError::Arithmetic)?;
        let used_hours = handled_quantity
            .checked_mul(hours)
            .ok_or(MaterialCircuitError::Arithmetic)?;
        if *handled_quantity > 0 {
            let labor = labor_index(state, merchant.site_id, merchant.labor_unit_id)
                .ok_or(MaterialCircuitError::CapacityInvariant)?;
            state.labor[labor].available = state.labor[labor]
                .available
                .checked_sub(used_hours)
                .ok_or(MaterialCircuitError::Arithmetic)?;
            let grams = handled_quantity
                .checked_mul(grams_per_unit(state, order.stock.1, order.stock.2)?)
                .ok_or(MaterialCircuitError::Arithmetic)?;
            let capacity = capacity_index(state, (state.period, merchant.capacity_id))
                .ok_or(MaterialCircuitError::CapacityInvariant)?;
            state.corridor_capacities[capacity].available_grams = state.corridor_capacities
                [capacity]
                .available_grams
                .checked_sub(grams)
                .ok_or(MaterialCircuitError::Arithmetic)?;
        }
        receipts.push(MerchantHandlingReceipt {
            site_id: merchant.site_id,
            order: order.id,
            feasible_quantity: *feasible_quantity,
            handled_quantity: *handled_quantity,
            needed_hours,
            used_hours,
        });
    }
    receipts.sort_by_key(|row| (row.site_id, row.order));
    Ok(receipts)
}

fn apply_local_fulfillments(
    state: &mut MaterialCircuitState,
    inventory: &mut InventoryLedger,
    allocations: &[u64],
) -> Result<Vec<LocalRetailFulfillmentReceipt>, MaterialCircuitError> {
    let mut receipts = Vec::new();
    for (row, quantity) in state.final_demand_orders.iter_mut().zip(allocations) {
        if *quantity == 0 {
            continue;
        }
        debit_inventory(
            inventory,
            (row.retailer_site_id, row.good_id, row.unit_id),
            *quantity,
            MaterialCircuitError::FreightInvariant,
        )?;
        row.fulfilled = row
            .fulfilled
            .checked_add(*quantity)
            .ok_or(MaterialCircuitError::Arithmetic)?;
        receipts.push(LocalRetailFulfillmentReceipt {
            order_id: row.order_id,
            retailer_site_id: row.retailer_site_id,
            demand_principal_id: row.demand_principal_id,
            good_id: row.good_id,
            unit_id: row.unit_id,
            quantity: *quantity,
        });
    }
    Ok(receipts)
}

pub(super) struct OutboundReceipts {
    pub handling: Vec<MerchantHandlingReceipt>,
    pub local_fulfillments: Vec<LocalRetailFulfillmentReceipt>,
    pub local_transfers: Vec<LocalTransferReceipt>,
}

pub(super) fn dispatch_orders(
    state: &mut MaterialCircuitState,
    inventory: &mut InventoryLedger,
    receipts: &mut Vec<RoutedDispatchReceipt>,
) -> Result<OutboundReceipts, MaterialCircuitError> {
    let routes = supplier_routes(state);
    let orders = outbound_orders(state, &routes);
    let groups = resource_groups(state, &orders)?;
    let feasible = order_allocations(state, inventory, &groups, orders.len())?;
    let mut allocations = feasible.clone();
    let request_count = groups.values().try_fold(0_usize, |count, rows| {
        count
            .checked_add(rows.len())
            .ok_or(MaterialCircuitError::Arithmetic)
    })?;
    let labor = labor_groups(state, &orders, &feasible, request_count)?;
    limit_allocations(state, inventory, &labor, &mut allocations)?;
    let handling = apply_handling(state, &orders, &feasible, &allocations)?;
    let routed_count = state.orders.len();
    let local_transfers = apply_dispatches(
        state,
        inventory,
        &routes,
        &allocations[..routed_count],
        receipts,
    )?;
    let local = apply_local_fulfillments(state, inventory, &allocations[routed_count..])?;
    // All outbound debits and grants were fixed before any local buyer credit.
    // A transfer cannot recursively supply another order within this close.
    for transfer in &local_transfers {
        credit_inventory(
            inventory,
            (transfer.buyer_site_id, transfer.good_id, transfer.unit_id),
            transfer.quantity,
        )?;
    }
    Ok(OutboundReceipts {
        handling,
        local_fulfillments: local,
        local_transfers,
    })
}
