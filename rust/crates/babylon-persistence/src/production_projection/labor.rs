//! Accounting from validated adjacent registers and the completed receipt family.

use std::collections::{BTreeMap, BTreeSet};

use babylon_material_circuit::{MaterialCircuitStateV2, ProcessIdV1, SiteIdV1, UnitIdV1};
use babylon_tick::material_world::MaterialTickReceiptsV3;

use super::ProductionProjectionErrorV1;
use crate::{michigan_economy::digest_hex, CompletedProductionLaborV1, ProductionLaborAccountV1};

type Principal = (SiteIdV1, UnitIdV1);
type Budgets = BTreeMap<Principal, u64>;
type Totals = BTreeMap<Principal, (u64, u64)>;

pub(super) fn project_labor_accounts(
    state: &MaterialCircuitStateV2,
    opening: Option<&MaterialCircuitStateV2>,
    receipt: Option<&MaterialTickReceiptsV3>,
) -> Result<Vec<ProductionLaborAccountV1>, ProductionProjectionErrorV1> {
    let next = budgets(state)?;
    let (prior, totals) = match (opening, receipt) {
        (None, None) if state.week == 1 => (None, Totals::new()),
        (Some(prior), Some(receipt))
            if prior.week.checked_add(1) == Some(state.week)
                && receipt.resolve_tick == prior.week =>
        {
            if prior.labor_coefficients != state.labor_coefficients
                || prior.process_outputs != state.process_outputs
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
                    let (planned, used) = totals.get(&key).copied().unwrap_or((0, 0));
                    let unused = available
                        .checked_sub(used)
                        .ok_or(ProductionProjectionErrorV1::State)?;
                    Ok::<_, ProductionProjectionErrorV1>(CompletedProductionLaborV1 {
                        week: state.week - 1,
                        opening: available,
                        planned,
                        used,
                        unused,
                    })
                })
                .transpose()?;
            Ok(ProductionLaborAccountV1 {
                site_id: digest_hex(&key.0.as_bytes()),
                unit_id: digest_hex(&key.1.as_bytes()),
                unit: "Designed labor-hours".to_owned(),
                next_opening_week: state.week,
                next_opening_available: next.get(&key).copied().unwrap_or(0),
                completed,
            })
        })
        .collect()
}

/// Missing sparse capacity is zero; a duplicated principal is never summed.
fn budgets(state: &MaterialCircuitStateV2) -> Result<Budgets, ProductionProjectionErrorV1> {
    let mut result = Budgets::new();
    for row in state.labor.iter().filter(|row| row.week == state.week) {
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
    Ok(result)
}

fn completed_totals(
    opening: &MaterialCircuitStateV2,
    receipt: &MaterialTickReceiptsV3,
) -> Result<Totals, ProductionProjectionErrorV1> {
    let mut processes = BTreeMap::<ProcessIdV1, (Principal, u64, u64)>::new();
    for plan in &opening.production_commitments {
        let coefficient = opening
            .labor_coefficients
            .iter()
            .find(|row| row.process_id == plan.process_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        if plan.week != opening.week
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
        account.0 = add_time(account.0, planned, coefficient)?;
        account.1 = add_time(account.1, row.produced_batches, coefficient)?;
    }
    if !processes.is_empty() {
        return Err(ProductionProjectionErrorV1::State);
    }
    Ok(totals)
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
