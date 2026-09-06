//! Completed physical accounts from verified adjacent registers and receipts.
//!
//! This projection never allocates batches, routes freight, or advances a world.
//! It reports actual receipt quantities using the recipe that governed the week.

use std::collections::{BTreeMap, BTreeSet};

use babylon_material_circuit::{
    GoodIdV1, MaterialCircuitStateV2, OrderIdV1, OrderRowV2, ProcessIdV1, SiteIdV1, UnitIdV1,
};
use babylon_tick::material_world::MaterialTickReceiptsV3;
use serde::{Deserialize, Serialize};

use super::ProductionProjectionErrorV1;
use crate::{michigan_economy::digest_hex, michigan_material::michigan_material_catalog_v1};

/// One complete committed week's local inventory accounts. Absent at foundation.
/// The enclosing authorized observation binds campaign, perspective and evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletedMaterialBalanceV1 {
    pub week: u64,
    pub rows: Vec<ProductionMaterialBalanceRowV1>,
}

/// Exact local stock account for one site/good/unit, never a sum across units.
/// Arrivals already exclude in-transit loss. Delivery and realization are not
/// additional stock credits. All quantities are Derived from committed evidence.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionMaterialBalanceRowV1 {
    pub site_id: String,
    pub good_id: String,
    pub unit_id: String,
    pub good: String,
    pub unit: String,
    pub opening: u64,
    pub arrivals: u64,
    pub produced: u64,
    pub consumed: u64,
    pub dispatched: u64,
    pub closing: u64,
}

type Principal = (SiteIdV1, GoodIdV1, UnitIdV1);
type Ledger = BTreeMap<Principal, Amounts>;
type Processes = BTreeMap<ProcessIdV1, Process>;
type Orders = BTreeMap<OrderIdV1, OrderPrincipal>;
type Movements = BTreeMap<OrderIdV1, Movement>;

#[derive(Default)]
struct Amounts {
    opening: u64,
    arrivals: u64,
    produced: u64,
    consumed: u64,
    dispatched: u64,
    closing: u64,
}

#[derive(PartialEq, Eq)]
struct Process {
    output: Principal,
    output_per_batch: u64,
    inputs: BTreeMap<(GoodIdV1, UnitIdV1), u64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct OrderPrincipal {
    supplier: SiteIdV1,
    buyer: SiteIdV1,
    good: GoodIdV1,
    unit: UnitIdV1,
}

#[derive(Default)]
struct Movement {
    dispatched: u64,
    arrived: u64,
    lost: u64,
    delivered: u64,
    realized: u64,
}

/// Inputs have already passed the enclosing material identity verification.
/// A missing completed family is legitimate only at the true foundation.
pub(super) fn project_material_balance(
    current: &MaterialCircuitStateV2,
    prior: Option<&MaterialCircuitStateV2>,
    receipt: Option<&MaterialTickReceiptsV3>,
) -> Result<Option<CompletedMaterialBalanceV1>, ProductionProjectionErrorV1> {
    let catalog =
        michigan_material_catalog_v1().map_err(|_| ProductionProjectionErrorV1::Content)?;
    project_with_labels(current, prior, receipt, |good, unit| {
        catalog
            .goods()
            .iter()
            .find(|row| row.id() == good && row.unit_id() == unit)
            .map(|row| (row.label.clone(), row.unit_key.clone()))
    })
}

fn project_with_labels(
    current: &MaterialCircuitStateV2,
    prior: Option<&MaterialCircuitStateV2>,
    receipt: Option<&MaterialTickReceiptsV3>,
    labels: impl Fn(GoodIdV1, UnitIdV1) -> Option<(String, String)>,
) -> Result<Option<CompletedMaterialBalanceV1>, ProductionProjectionErrorV1> {
    let (prior, receipt) = match (prior, receipt) {
        (None, None) if current.week == 1 => return Ok(None),
        (Some(prior), Some(receipt))
            if prior.week > 0
                && prior.week.checked_add(1) == Some(current.week)
                && receipt.resolve_tick == prior.week =>
        {
            (prior, receipt)
        }
        _ => return Err(ProductionProjectionErrorV1::History),
    };
    let processes = process_map(prior)?;
    let orders = order_map(prior)?;
    if processes != process_map(current)? || orders != order_map(current)? {
        return Err(ProductionProjectionErrorV1::State);
    }
    let mut ledger = inventory_ledger(prior, current)?;
    add_production(prior, receipt, &processes, &mut ledger)?;
    add_transport(prior, current, receipt, &orders, &mut ledger)?;
    let rows = ledger
        .into_iter()
        .map(|(key, amounts)| finish_row(key, &amounts, &labels))
        .collect::<Result<_, _>>()?;
    Ok(Some(CompletedMaterialBalanceV1 {
        week: receipt.resolve_tick,
        rows,
    }))
}

fn process_map(state: &MaterialCircuitStateV2) -> Result<Processes, ProductionProjectionErrorV1> {
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
            return Err(ProductionProjectionErrorV1::State);
        }
    }
    for row in &state.input_coefficients {
        let process = processes
            .get_mut(&row.process_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        if row.quantity_per_batch == 0
            || process
                .inputs
                .insert((row.good_id, row.unit_id), row.quantity_per_batch)
                .is_some()
        {
            return Err(ProductionProjectionErrorV1::State);
        }
    }
    Ok(processes)
}

fn order_map(state: &MaterialCircuitStateV2) -> Result<Orders, ProductionProjectionErrorV1> {
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
            return Err(ProductionProjectionErrorV1::State);
        }
    }
    Ok(orders)
}

fn inventory_ledger(
    prior: &MaterialCircuitStateV2,
    current: &MaterialCircuitStateV2,
) -> Result<Ledger, ProductionProjectionErrorV1> {
    let mut ledger = Ledger::new();
    for (state, opening) in [(prior, true), (current, false)] {
        let mut seen = BTreeSet::new();
        for row in &state.inventory {
            let key = (row.site_id, row.good_id, row.unit_id);
            if !seen.insert(key) {
                return Err(ProductionProjectionErrorV1::State);
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
    prior: &MaterialCircuitStateV2,
    receipt: &MaterialTickReceiptsV3,
    processes: &Processes,
    ledger: &mut Ledger,
) -> Result<(), ProductionProjectionErrorV1> {
    let mut plans = BTreeMap::new();
    for row in &prior.production_commitments {
        let process = processes
            .get(&row.process_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        if row.week != prior.week
            || row.site_id != process.output.0
            || plans.insert(row.process_id, row.planned_batches).is_some()
        {
            return Err(ProductionProjectionErrorV1::State);
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
            .ok_or(ProductionProjectionErrorV1::State)?;
        if plans.remove(&row.process_id) != Some(row.planned_batches)
            || row.site_id != process.output.0
            || row.produced_batches > row.planned_batches
        {
            return Err(ProductionProjectionErrorV1::State);
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
        return Err(ProductionProjectionErrorV1::State);
    }
    Ok(())
}

fn add_transport(
    prior: &MaterialCircuitStateV2,
    current: &MaterialCircuitStateV2,
    receipt: &MaterialTickReceiptsV3,
    orders: &Orders,
    ledger: &mut Ledger,
) -> Result<(), ProductionProjectionErrorV1> {
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
    for row in &receipt.arrivals {
        let principal = orders
            .get(&row.order_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        if row.quantity == 0 {
            return Err(ProductionProjectionErrorV1::State);
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
            return Err(ProductionProjectionErrorV1::State);
        }
        add(
            &mut movements.entry(row.order_id).or_default().delivered,
            row.quantity,
        )?;
    }
    for row in &receipt.realizations {
        if row.quantity == 0 || !orders.contains_key(&row.order_id) {
            return Err(ProductionProjectionErrorV1::State);
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
            .ok_or(ProductionProjectionErrorV1::State)?;
        check_movement(
            before,
            row,
            &movements.remove(&row.order_id).unwrap_or_default(),
        )?;
    }
    if !movements.is_empty() {
        return Err(ProductionProjectionErrorV1::State);
    }
    Ok(())
}

fn add_dispatches(
    current: &MaterialCircuitStateV2,
    receipt: &MaterialTickReceiptsV3,
    orders: &Orders,
    movements: &mut Movements,
    ledger: &mut Ledger,
) -> Result<(), ProductionProjectionErrorV1> {
    let mut seen = BTreeSet::new();
    for row in &receipt.dispatches {
        let principal = orders
            .get(&row.order_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        let lot = current
            .freight
            .iter()
            .find(|lot| lot.lot_id == row.lot_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        if row.quantity == 0
            || !seen.insert(row.lot_id)
            || row.final_arrival_week <= receipt.resolve_tick
            || lot.order_id != row.order_id
            || lot.route_id != row.route_id
            || lot.quantity != row.quantity
            || lot.dispatch_week != receipt.resolve_tick
            || lot.current_leg_index != 0
            || lot.leg_arrival_week <= receipt.resolve_tick
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
            return Err(ProductionProjectionErrorV1::State);
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
    prior: &MaterialCircuitStateV2,
    receipt: &MaterialTickReceiptsV3,
    orders: &Orders,
    movements: &mut Movements,
) -> Result<(), ProductionProjectionErrorV1> {
    let mut seen = BTreeSet::new();
    for row in &receipt.losses {
        let lot = prior
            .freight
            .iter()
            .find(|lot| lot.lot_id == row.lot_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        let leg = prior
            .route_legs
            .iter()
            .find(|leg| leg.route_id == lot.route_id && leg.leg_index == lot.current_leg_index)
            .ok_or(ProductionProjectionErrorV1::State)?;
        if row.quantity == 0
            || row.quantity > lot.quantity
            || !seen.insert(row.lot_id)
            || !orders.contains_key(&row.order_id)
            || lot.order_id != row.order_id
            || lot.leg_arrival_week != receipt.resolve_tick
            || leg.corridor_id != row.corridor_id
        {
            return Err(ProductionProjectionErrorV1::State);
        }
        // Loss is a transit sink, not another debit of already-dispatched stock.
        add(
            &mut movements.entry(row.order_id).or_default().lost,
            row.quantity,
        )?;
    }
    Ok(())
}

fn check_movement(
    prior: &OrderRowV2,
    current: &OrderRowV2,
    movement: &Movement,
) -> Result<(), ProductionProjectionErrorV1> {
    if prior.ordered != current.ordered
        || prior.access_mode != current.access_mode
        || current.shipped.checked_sub(prior.shipped) != Some(movement.dispatched)
        || current.delivered.checked_sub(prior.delivered) != Some(movement.arrived)
        || current.lost.checked_sub(prior.lost) != Some(movement.lost)
        || current.realized.checked_sub(prior.realized) != Some(movement.realized)
        || movement.delivered != movement.arrived
        || movement.realized != movement.arrived
    {
        return Err(ProductionProjectionErrorV1::State);
    }
    Ok(())
}

fn multiply(left: u64, right: u64) -> Result<u64, ProductionProjectionErrorV1> {
    left.checked_mul(right)
        .ok_or(ProductionProjectionErrorV1::Arithmetic)
}

fn add(total: &mut u64, value: u64) -> Result<(), ProductionProjectionErrorV1> {
    *total = total
        .checked_add(value)
        .ok_or(ProductionProjectionErrorV1::Arithmetic)?;
    Ok(())
}

fn finish_row(
    key: Principal,
    amounts: &Amounts,
    labels: &impl Fn(GoodIdV1, UnitIdV1) -> Option<(String, String)>,
) -> Result<ProductionMaterialBalanceRowV1, ProductionProjectionErrorV1> {
    let total = |a: u64, b: u64, c: u64| {
        u128::from(a)
            .checked_add(u128::from(b))
            .and_then(|sum| sum.checked_add(u128::from(c)))
            .ok_or(ProductionProjectionErrorV1::Arithmetic)
    };
    if total(amounts.opening, amounts.arrivals, amounts.produced)?
        != total(amounts.consumed, amounts.dispatched, amounts.closing)?
    {
        return Err(ProductionProjectionErrorV1::State);
    }
    let (good, unit) = labels(key.1, key.2).ok_or(ProductionProjectionErrorV1::Content)?;
    Ok(ProductionMaterialBalanceRowV1 {
        site_id: digest_hex(&key.0.as_bytes()),
        good_id: digest_hex(&key.1.as_bytes()),
        unit_id: digest_hex(&key.2.as_bytes()),
        good,
        unit,
        opening: amounts.opening,
        arrivals: amounts.arrivals,
        produced: amounts.produced,
        consumed: amounts.consumed,
        dispatched: amounts.dispatched,
        closing: amounts.closing,
    })
}

#[cfg(test)]
mod tests;
