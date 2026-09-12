//! Completed physical accounts from verified adjacent registers and receipts.
//!
//! This projection never allocates batches, routes freight, or advances a world.
//! It reports actual receipt quantities using the recipe that governed the period.

use std::collections::{BTreeMap, BTreeSet};

use babylon_material_circuit::{
    GoodId, MaterialCircuitState, OrderId, OrderRow, ProcessId, SiteId, UnitId,
};
use babylon_tick::material_world::MaterialTickReceipts;
use serde::{Deserialize, Serialize};

use super::ProductionProjectionError;
use crate::{michigan_economy::digest_hex, michigan_material::MichiganMaterialCatalog};

/// One complete committed period's local inventory accounts. Absent at foundation.
/// The enclosing authorized observation binds campaign, perspective and evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletedMaterialBalance {
    pub period: u64,
    pub rows: Vec<ProductionMaterialBalanceRow>,
}

/// Exact local stock account for one site/good/unit, never a sum across units.
/// Arrivals already exclude in-transit loss. Delivery and realization are not
/// additional stock credits. All quantities are Derived from committed evidence.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionMaterialBalanceRow {
    pub site_id: String,
    pub good_id: String,
    pub unit_id: String,
    pub good: String,
    pub unit: String,
    pub opening: u64,
    pub arrivals: u64,
    pub local_received: u64,
    pub local_transferred: u64,
    pub final_demand_fulfilled: u64,
    pub produced: u64,
    pub consumed: u64,
    pub dispatched: u64,
    pub closing: u64,
}

type Principal = (SiteId, GoodId, UnitId);
type Ledger = BTreeMap<Principal, Amounts>;
type Processes = BTreeMap<ProcessId, Process>;
type Orders = BTreeMap<OrderId, OrderPrincipal>;
type Movements = BTreeMap<OrderId, Movement>;

#[derive(Default)]
struct Amounts {
    opening: u64,
    arrivals: u64,
    local_received: u64,
    local_transferred: u64,
    final_demand_fulfilled: u64,
    produced: u64,
    consumed: u64,
    dispatched: u64,
    closing: u64,
}

#[derive(PartialEq, Eq)]
struct Process {
    output: Principal,
    output_per_batch: u64,
    inputs: BTreeMap<(GoodId, UnitId), u64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct OrderPrincipal {
    supplier: SiteId,
    buyer: SiteId,
    good: GoodId,
    unit: UnitId,
}

#[derive(Default)]
struct Movement {
    dispatched: u64,
    local_transferred: u64,
    arrived: u64,
    lost: u64,
    delivered: u64,
    realized: u64,
}

/// Inputs have already passed the enclosing material identity verification.
/// A missing completed family is legitimate only at the true foundation.
pub(super) fn project_material_balance(
    catalog: &MichiganMaterialCatalog,
    current: &MaterialCircuitState,
    prior: Option<&MaterialCircuitState>,
    receipt: Option<&MaterialTickReceipts>,
) -> Result<Option<CompletedMaterialBalance>, ProductionProjectionError> {
    project_with_labels(current, prior, receipt, |good, unit| {
        catalog
            .goods()
            .iter()
            .find(|row| row.id() == good && row.unit_id() == unit)
            .map(|row| (row.label.clone(), row.unit_key.clone()))
    })
}

fn project_with_labels(
    current: &MaterialCircuitState,
    prior: Option<&MaterialCircuitState>,
    receipt: Option<&MaterialTickReceipts>,
    labels: impl Fn(GoodId, UnitId) -> Option<(String, String)>,
) -> Result<Option<CompletedMaterialBalance>, ProductionProjectionError> {
    let (prior, receipt) = match (prior, receipt) {
        (None, None) if current.period == 1 => return Ok(None),
        (Some(prior), Some(receipt))
            if prior.period > 0
                && prior.period.checked_add(1) == Some(current.period)
                && receipt.resolve_tick == prior.period =>
        {
            (prior, receipt)
        }
        _ => return Err(ProductionProjectionError::History),
    };
    super::outbound::completed_facts(prior, current, receipt)?;
    let processes = process_map(prior)?;
    let orders = order_map(prior)?;
    if processes != process_map(current)? || orders != order_map(current)? {
        return Err(ProductionProjectionError::State);
    }
    let mut ledger = inventory_ledger(prior, current)?;
    add_production(prior, receipt, &processes, &mut ledger)?;
    add_transport(prior, current, receipt, &orders, &mut ledger)?;
    add_final_demand(prior, receipt, &mut ledger)?;
    let rows = ledger
        .into_iter()
        .map(|(key, amounts)| finish_row(key, &amounts, &labels))
        .collect::<Result<_, _>>()?;
    Ok(Some(CompletedMaterialBalance {
        period: receipt.resolve_tick,
        rows,
    }))
}

fn process_map(state: &MaterialCircuitState) -> Result<Processes, ProductionProjectionError> {
    let mut processes = Processes::new();
    for row in &state.process_outputs {
        if row.quantity_per_batch == 0
            || processes
                .insert(
                    row.process_id,
                    Process {
                        output: (row.site_id, row.good_id, row.unit_id),
                        output_per_batch: row.quantity_per_batch,
                        inputs: BTreeMap::new(),
                    },
                )
                .is_some()
        {
            return Err(ProductionProjectionError::State);
        }
    }
    for row in &state.input_coefficients {
        let process = processes
            .get_mut(&row.process_id)
            .ok_or(ProductionProjectionError::State)?;
        if row.quantity_per_batch == 0
            || process
                .inputs
                .insert((row.good_id, row.unit_id), row.quantity_per_batch)
                .is_some()
        {
            return Err(ProductionProjectionError::State);
        }
    }
    Ok(processes)
}

fn order_map(state: &MaterialCircuitState) -> Result<Orders, ProductionProjectionError> {
    let mut orders = Orders::new();
    for row in &state.orders {
        if row.shipped > row.ordered
            || row.realized > row.delivered
            || row
                .lost
                .checked_add(row.delivered)
                .is_none_or(|n| n > row.shipped)
            || orders
                .insert(
                    row.order_id,
                    OrderPrincipal {
                        supplier: row.supplier_site_id,
                        buyer: row.buyer_site_id,
                        good: row.good_id,
                        unit: row.unit_id,
                    },
                )
                .is_some()
        {
            return Err(ProductionProjectionError::State);
        }
    }
    Ok(orders)
}

fn inventory_ledger(
    prior: &MaterialCircuitState,
    current: &MaterialCircuitState,
) -> Result<Ledger, ProductionProjectionError> {
    let mut ledger = Ledger::new();
    for (state, opening) in [(prior, true), (current, false)] {
        let mut seen = BTreeSet::new();
        for row in &state.inventory {
            let key = (row.site_id, row.good_id, row.unit_id);
            if !seen.insert(key) {
                return Err(ProductionProjectionError::State);
            }
            let amounts = ledger.entry(key).or_default();
            if opening {
                amounts.opening = row.quantity;
            } else {
                amounts.closing = row.quantity;
            }
        }
    }
    Ok(ledger)
}

fn add_production(
    prior: &MaterialCircuitState,
    receipt: &MaterialTickReceipts,
    processes: &Processes,
    ledger: &mut Ledger,
) -> Result<(), ProductionProjectionError> {
    let mut plans = BTreeMap::new();
    for row in &prior.production_commitments {
        let process = processes
            .get(&row.process_id)
            .ok_or(ProductionProjectionError::State)?;
        if row.period != prior.period
            || row.site_id != process.output.0
            || plans.insert(row.process_id, row.planned_batches).is_some()
        {
            return Err(ProductionProjectionError::State);
        }
    }
    // Declared but idle input/output principals have honest zero-flow rows.
    for process in processes.values() {
        ledger.entry(process.output).or_default();
        for &(good, unit) in process.inputs.keys() {
            ledger.entry((process.output.0, good, unit)).or_default();
        }
    }
    for row in &receipt.production {
        let process = processes
            .get(&row.process_id)
            .ok_or(ProductionProjectionError::State)?;
        if plans.remove(&row.process_id) != Some(row.planned_batches)
            || row.site_id != process.output.0
            || row.produced_batches > row.planned_batches
        {
            return Err(ProductionProjectionError::State);
        }
        let output = multiply(process.output_per_batch, row.produced_batches)?;
        add(
            &mut ledger.entry(process.output).or_default().produced,
            output,
        )?;
        for (&(good, unit), &coefficient) in &process.inputs {
            let input = multiply(coefficient, row.produced_batches)?;
            add(
                &mut ledger
                    .entry((row.site_id, good, unit))
                    .or_default()
                    .consumed,
                input,
            )?;
        }
    }
    if !plans.is_empty() {
        return Err(ProductionProjectionError::State);
    }
    Ok(())
}

fn add_transport(
    prior: &MaterialCircuitState,
    current: &MaterialCircuitState,
    receipt: &MaterialTickReceipts,
    orders: &Orders,
    ledger: &mut Ledger,
) -> Result<(), ProductionProjectionError> {
    let mut movements = Movements::new();
    for order in orders.values() {
        ledger
            .entry((order.supplier, order.good, order.unit))
            .or_default();
        ledger
            .entry((order.buyer, order.good, order.unit))
            .or_default();
    }
    add_dispatches(current, receipt, orders, &mut movements, ledger)?;
    add_losses(prior, receipt, orders, &mut movements)?;
    add_local_transfers(receipt, orders, &mut movements, ledger)?;
    for row in &receipt.arrivals {
        let principal = orders
            .get(&row.order_id)
            .ok_or(ProductionProjectionError::State)?;
        if row.quantity == 0 {
            return Err(ProductionProjectionError::State);
        }
        add(
            &mut ledger
                .entry((principal.buyer, principal.good, principal.unit))
                .or_default()
                .arrivals,
            row.quantity,
        )?;
        add(
            &mut movements.entry(row.order_id).or_default().arrived,
            row.quantity,
        )?;
    }
    for row in &receipt.deliveries {
        if row.quantity == 0 || !orders.contains_key(&row.order_id) {
            return Err(ProductionProjectionError::State);
        }
        add(
            &mut movements.entry(row.order_id).or_default().delivered,
            row.quantity,
        )?;
    }
    for row in &receipt.realizations {
        if row.quantity == 0 || !orders.contains_key(&row.order_id) {
            return Err(ProductionProjectionError::State);
        }
        add(
            &mut movements.entry(row.order_id).or_default().realized,
            row.quantity,
        )?;
    }
    let previous: BTreeMap<_, _> = prior.orders.iter().map(|row| (row.order_id, row)).collect();
    for row in &current.orders {
        let before = previous
            .get(&row.order_id)
            .ok_or(ProductionProjectionError::State)?;
        check_movement(
            before,
            row,
            &movements.remove(&row.order_id).unwrap_or_default(),
        )?;
    }
    if !movements.is_empty() {
        return Err(ProductionProjectionError::State);
    }
    Ok(())
}

fn add_dispatches(
    current: &MaterialCircuitState,
    receipt: &MaterialTickReceipts,
    orders: &Orders,
    movements: &mut Movements,
    ledger: &mut Ledger,
) -> Result<(), ProductionProjectionError> {
    let mut seen = BTreeSet::new();
    for row in &receipt.dispatches {
        let principal = orders
            .get(&row.order_id)
            .ok_or(ProductionProjectionError::State)?;
        let lot = current
            .freight
            .iter()
            .find(|lot| lot.lot_id == row.lot_id)
            .ok_or(ProductionProjectionError::State)?;
        if row.quantity == 0
            || !seen.insert(row.lot_id)
            || row.final_arrival_period <= receipt.resolve_tick
            || lot.order_id != row.order_id
            || lot.route_id != row.route_id
            || lot.quantity != row.quantity
            || lot.dispatch_period != receipt.resolve_tick
            || lot.current_stage_index != 0
            || lot.stage_arrival_period <= receipt.resolve_tick
            || (
                lot.source_site_id,
                lot.destination_site_id,
                lot.good_id,
                lot.unit_id,
            ) != (
                principal.supplier,
                principal.buyer,
                principal.good,
                principal.unit,
            )
        {
            return Err(ProductionProjectionError::State);
        }
        add(
            &mut ledger
                .entry((principal.supplier, principal.good, principal.unit))
                .or_default()
                .dispatched,
            row.quantity,
        )?;
        add(
            &mut movements.entry(row.order_id).or_default().dispatched,
            row.quantity,
        )?;
    }
    Ok(())
}

fn add_losses(
    prior: &MaterialCircuitState,
    receipt: &MaterialTickReceipts,
    orders: &Orders,
    movements: &mut Movements,
) -> Result<(), ProductionProjectionError> {
    let mut seen = BTreeSet::new();
    for row in &receipt.losses {
        let lot = prior
            .freight
            .iter()
            .find(|lot| lot.lot_id == row.lot_id)
            .ok_or(ProductionProjectionError::State)?;
        let leg = prior
            .route_stages
            .iter()
            .find(|leg| leg.route_id == lot.route_id && leg.stage_index == lot.current_stage_index)
            .ok_or(ProductionProjectionError::State)?;
        if row.quantity == 0
            || row.quantity > lot.quantity
            || !seen.insert(row.lot_id)
            || !orders.contains_key(&row.order_id)
            || lot.order_id != row.order_id
            || lot.stage_arrival_period != receipt.resolve_tick
            || leg.route_id != row.route_id
            || leg.stage_index != row.stage_index
        {
            return Err(ProductionProjectionError::State);
        }
        // Loss is a transit sink, not another debit of already-dispatched stock.
        add(
            &mut movements.entry(row.order_id).or_default().lost,
            row.quantity,
        )?;
    }
    Ok(())
}

/// A local internal handoff is one debit and one credit of the same native good.
/// The caller has authenticated receipt identity and order deltas through the
/// common outbound facts; no freight lot, arrival or new allocator is involved.
fn add_local_transfers(
    receipt: &MaterialTickReceipts,
    orders: &Orders,
    movements: &mut Movements,
    ledger: &mut Ledger,
) -> Result<(), ProductionProjectionError> {
    for row in &receipt.local_transfers {
        let principal = orders
            .get(&row.order_id)
            .ok_or(ProductionProjectionError::State)?;
        add(
            &mut ledger
                .entry((principal.supplier, principal.good, principal.unit))
                .or_default()
                .local_transferred,
            row.quantity,
        )?;
        add(
            &mut ledger
                .entry((principal.buyer, principal.good, principal.unit))
                .or_default()
                .local_received,
            row.quantity,
        )?;
        add(
            &mut movements.entry(row.order_id).or_default().local_transferred,
            row.quantity,
        )?;
    }
    Ok(())
}

/// Finite end-buyer fulfillment is a terminal stock sink, not consumption.
fn add_final_demand(
    prior: &MaterialCircuitState,
    receipt: &MaterialTickReceipts,
    ledger: &mut Ledger,
) -> Result<(), ProductionProjectionError> {
    for order in &prior.final_demand_orders {
        ledger
            .entry((order.retailer_site_id, order.good_id, order.unit_id))
            .or_default();
    }
    for row in &receipt.local_fulfillments {
        add(
            &mut ledger
                .entry((row.retailer_site_id, row.good_id, row.unit_id))
                .or_default()
                .final_demand_fulfilled,
            row.quantity,
        )?;
    }
    Ok(())
}

fn check_movement(
    prior: &OrderRow,
    current: &OrderRow,
    movement: &Movement,
) -> Result<(), ProductionProjectionError> {
    let shipped = movement
        .dispatched
        .checked_add(movement.local_transferred)
        .ok_or(ProductionProjectionError::Arithmetic)?;
    let delivered = movement
        .arrived
        .checked_add(movement.local_transferred)
        .ok_or(ProductionProjectionError::Arithmetic)?;
    let realized = movement
        .realized
        .checked_add(movement.local_transferred)
        .ok_or(ProductionProjectionError::Arithmetic)?;
    if prior.ordered != current.ordered
        || prior.access_mode != current.access_mode
        || current.shipped.checked_sub(prior.shipped) != Some(shipped)
        || current.delivered.checked_sub(prior.delivered) != Some(delivered)
        || current.lost.checked_sub(prior.lost) != Some(movement.lost)
        || current.realized.checked_sub(prior.realized) != Some(realized)
        || movement.delivered != movement.arrived
        || movement.realized != movement.arrived
    {
        return Err(ProductionProjectionError::State);
    }
    Ok(())
}

fn multiply(left: u64, right: u64) -> Result<u64, ProductionProjectionError> {
    left.checked_mul(right)
        .ok_or(ProductionProjectionError::Arithmetic)
}

fn add(total: &mut u64, value: u64) -> Result<(), ProductionProjectionError> {
    *total = total
        .checked_add(value)
        .ok_or(ProductionProjectionError::Arithmetic)?;
    Ok(())
}

fn finish_row(
    key: Principal,
    amounts: &Amounts,
    labels: &impl Fn(GoodId, UnitId) -> Option<(String, String)>,
) -> Result<ProductionMaterialBalanceRow, ProductionProjectionError> {
    let total = |values: &[u64]| {
        values
            .iter()
            .try_fold(0_u128, |sum, value| sum.checked_add(u128::from(*value)))
            .ok_or(ProductionProjectionError::Arithmetic)
    };
    if total(&[
        amounts.opening,
        amounts.arrivals,
        amounts.local_received,
        amounts.produced,
    ])? != total(&[
        amounts.consumed,
        amounts.dispatched,
        amounts.local_transferred,
        amounts.final_demand_fulfilled,
        amounts.closing,
    ])? {
        return Err(ProductionProjectionError::State);
    }
    let (good, unit) = labels(key.1, key.2).ok_or(ProductionProjectionError::Content)?;
    Ok(ProductionMaterialBalanceRow {
        site_id: digest_hex(&key.0.as_bytes()),
        good_id: digest_hex(&key.1.as_bytes()),
        unit_id: digest_hex(&key.2.as_bytes()),
        good,
        unit,
        opening: amounts.opening,
        arrivals: amounts.arrivals,
        local_received: amounts.local_received,
        local_transferred: amounts.local_transferred,
        final_demand_fulfilled: amounts.final_demand_fulfilled,
        produced: amounts.produced,
        consumed: amounts.consumed,
        dispatched: amounts.dispatched,
        closing: amounts.closing,
    })
}

#[cfg(test)]
mod tests;
