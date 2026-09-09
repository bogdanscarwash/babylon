//! Accounting from validated adjacent registers and the completed receipt family.

use std::collections::{BTreeMap, BTreeSet};

use babylon_material_circuit::{MaterialCircuitStateV3, ProcessIdV1, SiteIdV1, UnitIdV1};
use babylon_tick::material_world::MaterialTickReceiptsV4;

use super::ProductionProjectionErrorV1;
use crate::{michigan_economy::digest_hex, CompletedProductionLaborV2, ProductionLaborAccountV2};

type Principal = (SiteIdV1, UnitIdV1);
type Budgets = BTreeMap<Principal, u64>;
type Totals = BTreeMap<Principal, LaborTotals>;

#[derive(Clone, Copy, Default)]
struct LaborTotals {
    planned: u64,
    used: u64,
    handling_needed: u64,
    handling_used: u64,
}

pub(super) fn project_labor_accounts(
    state: &MaterialCircuitStateV3,
    opening: Option<&MaterialCircuitStateV3>,
    receipt: Option<&MaterialTickReceiptsV4>,
) -> Result<Vec<ProductionLaborAccountV2>, ProductionProjectionErrorV1> {
    let next = budgets(state)?;
    let (prior, totals) = match (opening, receipt) {
        (None, None) if state.period == 1 => (None, Totals::new()),
        (Some(prior), Some(receipt))
            if prior.period.checked_add(1) == Some(state.period)
                && receipt.resolve_tick == prior.period =>
        {
            if !super::outbound::same_rows(&prior.labor_coefficients, &state.labor_coefficients)
                || !super::outbound::same_rows(&prior.process_outputs, &state.process_outputs)
                || !super::outbound::same_rows(&prior.merchants, &state.merchants)
                || !super::outbound::same_rows(
                    &prior.handling_coefficients,
                    &state.handling_coefficients,
                )
            {
                return Err(ProductionProjectionErrorV1::State);
            }
            (Some(budgets(prior)?), completed_totals(prior, receipt)?)
        }
        _ => return Err(ProductionProjectionErrorV1::History),
    };
    let mut keys: BTreeSet<_> = next.keys().copied().collect();
    if let Some(prior) = &prior {
        keys.extend(prior.keys().copied());
    }
    keys.extend(totals.keys().copied());
    keys.into_iter()
        .map(|key| {
            let completed = prior
                .as_ref()
                .map(|prior| {
                    let available = prior.get(&key).copied().unwrap_or(0);
                    let account = totals.get(&key).copied().unwrap_or_default();
                    let used = account.used;
                    let unused = available
                        .checked_sub(used)
                        .ok_or(ProductionProjectionErrorV1::State)?;
                    Ok::<_, ProductionProjectionErrorV1>(CompletedProductionLaborV2 {
                        period: state.period - 1,
                        opening: available,
                        planned: account.planned,
                        used,
                        unused,
                        handling_needed: account.handling_needed,
                        handling_used: account.handling_used,
                    })
                })
                .transpose()?;
            Ok(ProductionLaborAccountV2 {
                site_id: digest_hex(&key.0.as_bytes()),
                unit_id: digest_hex(&key.1.as_bytes()),
                unit: "labor-hours".to_owned(),
                next_opening_period: state.period,
                next_opening_available: next.get(&key).copied().unwrap_or(0),
                completed,
            })
        })
        .collect()
}

/// Missing sparse capacity is zero; a duplicated principal is never summed.
fn budgets(state: &MaterialCircuitStateV3) -> Result<Budgets, ProductionProjectionErrorV1> {
    let mut result = Budgets::new();
    for row in state.labor.iter().filter(|row| row.period == state.period) {
        if result
            .insert((row.site_id, row.unit_id), row.available)
            .is_some()
        {
            return Err(ProductionProjectionErrorV1::State);
        }
    }
    for process in &state.process_outputs {
        let coefficient = state
            .labor_coefficients
            .iter()
            .find(|row| row.process_id == process.process_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        result
            .entry((process.site_id, coefficient.unit_id))
            .or_insert(0);
    }
    for merchant in &state.merchants {
        result
            .entry((merchant.site_id, merchant.labor_unit_id))
            .or_insert(0);
    }
    Ok(result)
}

fn completed_totals(
    opening: &MaterialCircuitStateV3,
    receipt: &MaterialTickReceiptsV4,
) -> Result<Totals, ProductionProjectionErrorV1> {
    let mut processes = BTreeMap::<ProcessIdV1, (Principal, u64, u64)>::new();
    for plan in &opening.production_commitments {
        let coefficient = opening
            .labor_coefficients
            .iter()
            .find(|row| row.process_id == plan.process_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        if plan.period != opening.period
            || !opening
                .process_outputs
                .iter()
                .any(|row| row.process_id == plan.process_id && row.site_id == plan.site_id)
            || processes
                .insert(
                    plan.process_id,
                    (
                        (plan.site_id, coefficient.unit_id),
                        coefficient.quantity_per_batch,
                        plan.planned_batches,
                    ),
                )
                .is_some()
        {
            return Err(ProductionProjectionErrorV1::State);
        }
    }
    let mut totals = Totals::new();
    for row in &receipt.production {
        let (key, coefficient, planned) = processes
            .remove(&row.process_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        if row.site_id != key.0 || row.planned_batches != planned || row.produced_batches > planned
        {
            return Err(ProductionProjectionErrorV1::State);
        }
        let account = totals.entry(key).or_default();
        account.planned = add_time(account.planned, planned, coefficient)?;
        account.used = add_time(account.used, row.produced_batches, coefficient)?;
    }
    if !processes.is_empty() {
        return Err(ProductionProjectionErrorV1::State);
    }
    add_handling_time(opening, receipt, &mut totals)?;
    Ok(totals)
}

fn add_handling_time(
    opening: &MaterialCircuitStateV3,
    receipt: &MaterialTickReceiptsV4,
    totals: &mut Totals,
) -> Result<(), ProductionProjectionErrorV1> {
    let mut seen = BTreeSet::new();
    for row in &receipt.handling {
        let merchant = opening
            .merchants
            .iter()
            .find(|merchant| merchant.site_id == row.site_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        if !seen.insert(row.order)
            || row.handled_quantity > row.feasible_quantity
            || row.used_hours > row.needed_hours
        {
            return Err(ProductionProjectionErrorV1::State);
        }
        let account = totals
            .entry((row.site_id, merchant.labor_unit_id))
            .or_default();
        account.handling_needed = add_time(account.handling_needed, row.needed_hours, 1)?;
        account.handling_used = add_time(account.handling_used, row.used_hours, 1)?;
        account.used = add_time(account.used, row.used_hours, 1)?;
    }
    Ok(())
}

fn add_time(
    total: u64,
    batches: u64,
    coefficient: u64,
) -> Result<u64, ProductionProjectionErrorV1> {
    coefficient
        .checked_mul(batches)
        .and_then(|time| total.checked_add(time))
        .ok_or(ProductionProjectionErrorV1::Arithmetic)
}

#[cfg(test)]
mod tests;
