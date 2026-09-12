//! Committed merchant handling and finite county demand, without allocation.

use super::{
    outbound::{completed_facts, identity, mass, OutboundFact},
    ProductionProjectionError,
};
use crate::{
    michigan_economy::digest_hex, michigan_material::MichiganMaterialCatalog,
    production_observation::CompletedProductionFinalDemand,
    production_observation::CompletedProductionMerchantHandling,
    production_observation::ProductionFinalDemandAccount,
    production_observation::ProductionFinalDemandOrder,
    production_observation::ProductionHandlingCoefficient,
    production_observation::ProductionMerchantHandlingAccount,
    production_observation::ProductionMerchantHandlingOrder,
};
use babylon_material_circuit::{
    FinalDemandPrincipalId, GoodId, MaterialCircuitState, OutboundOrderId, SiteId, UnitId,
};
use babylon_tick::material_world::MaterialTickReceipts;
use std::collections::{BTreeMap, BTreeSet};

type Result<T> = std::result::Result<T, ProductionProjectionError>;
type DemandKey = (FinalDemandPrincipalId, GoodId, UnitId);

pub(super) fn project_merchants(
    catalog: &MichiganMaterialCatalog,
    current: &MaterialCircuitState,
    prior: Option<&MaterialCircuitState>,
    receipt: Option<&MaterialTickReceipts>,
) -> Result<(
    Vec<ProductionMerchantHandlingAccount>,
    Vec<ProductionFinalDemandAccount>,
)> {
    let completed = match (prior, receipt) {
        (None, None) if current.period == 1 => None,
        (Some(prior), Some(receipt)) => {
            Some((prior, receipt, completed_facts(prior, current, receipt)?))
        }
        _ => return Err(ProductionProjectionError::History),
    };
    let handling = handling_accounts(
        current,
        completed
            .as_ref()
            .map(|(prior, receipt, facts)| (*prior, *receipt, facts.as_slice())),
    )?;
    let demand = final_demand_accounts(
        catalog,
        current,
        completed
            .as_ref()
            .map(|(prior, _, facts)| (*prior, facts.as_slice())),
    )?;
    Ok((handling, demand))
}

fn handling_accounts(
    current: &MaterialCircuitState,
    completed: Option<(
        &MaterialCircuitState,
        &MaterialTickReceipts,
        &[OutboundFact],
    )>,
) -> Result<Vec<ProductionMerchantHandlingAccount>> {
    let complete_rows = completed
        .map(|(prior, receipt, facts)| handling_rows(prior, receipt, facts))
        .transpose()?;
    current
        .merchants
        .iter()
        .map(|merchant| {
            let coefficients = current
                .handling_coefficients
                .iter()
                .filter(|row| row.site_id == merchant.site_id)
                .map(|row| {
                    if row.hours_per_unit == 0 {
                        return Err(ProductionProjectionError::State);
                    }
                    Ok(ProductionHandlingCoefficient {
                        good_id: digest_hex(&row.good_id.as_bytes()),
                        unit_id: digest_hex(&row.unit_id.as_bytes()),
                        grams_per_unit: mass(current, row.good_id, row.unit_id)?,
                        hours_per_unit: row.hours_per_unit,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let completed = complete_rows
                .as_ref()
                .map(|all| {
                    let orders = all.get(&merchant.site_id).cloned().unwrap_or_default();
                    let needed_hours = sum(orders.iter().map(|row| row.needed_hours))?;
                    let used_hours = sum(orders.iter().map(|row| row.used_hours))?;
                    let handled_grams = sum(orders
                        .iter()
                        .map(|row| {
                            let coefficient = coefficients
                                .iter()
                                .find(|coefficient| {
                                    coefficient.good_id == row.good_id
                                        && coefficient.unit_id == row.unit_id
                                })
                                .ok_or(ProductionProjectionError::State)?;
                            row.handled_quantity
                                .checked_mul(coefficient.grams_per_unit)
                                .ok_or(ProductionProjectionError::Arithmetic)
                        })
                        .collect::<Result<Vec<_>>>()?)?;
                    Ok(CompletedProductionMerchantHandling {
                        period: current.period - 1,
                        needed_hours,
                        used_hours,
                        handled_grams,
                        orders,
                    })
                })
                .transpose()?;
            Ok(ProductionMerchantHandlingAccount {
                site_id: digest_hex(&merchant.site_id.as_bytes()),
                capacity_id: digest_hex(&merchant.capacity_id.as_bytes()),
                labor_unit_id: digest_hex(&merchant.labor_unit_id.as_bytes()),
                coefficients,
                completed,
            })
        })
        .collect()
}

fn handling_rows(
    prior: &MaterialCircuitState,
    receipt: &MaterialTickReceipts,
    facts: &[OutboundFact],
) -> Result<BTreeMap<SiteId, Vec<ProductionMerchantHandlingOrder>>> {
    let merchants: BTreeSet<_> = prior.merchants.iter().map(|row| row.site_id).collect();
    let mut expected = BTreeMap::new();
    for fact in facts.iter().filter(|fact| merchants.contains(&fact.site)) {
        if expected.insert((fact.site, fact.id), fact).is_some() {
            return Err(ProductionProjectionError::State);
        }
    }
    let mut result = BTreeMap::<SiteId, Vec<ProductionMerchantHandlingOrder>>::new();
    for row in &receipt.handling {
        let fact = expected
            .remove(&(row.site_id, row.order))
            .ok_or(ProductionProjectionError::State)?;
        let mut coefficients = prior.handling_coefficients.iter().filter(|coefficient| {
            coefficient.site_id == row.site_id
                && coefficient.good_id == fact.good
                && coefficient.unit_id == fact.unit
        });
        let coefficient = coefficients
            .next()
            .ok_or(ProductionProjectionError::State)?;
        if coefficients.next().is_some()
            || coefficient.hours_per_unit == 0
            || row.handled_quantity != fact.quantity
            || row.handled_quantity > row.feasible_quantity
            || row.feasible_quantity > fact.requested
            || row
                .feasible_quantity
                .checked_mul(coefficient.hours_per_unit)
                != Some(row.needed_hours)
            || row.handled_quantity.checked_mul(coefficient.hours_per_unit) != Some(row.used_hours)
        {
            return Err(ProductionProjectionError::State);
        }
        let (id, kind) = identity(row.order);
        result
            .entry(row.site_id)
            .or_default()
            .push(ProductionMerchantHandlingOrder {
                order_id: digest_hex(&id.as_bytes()),
                kind,
                good_id: digest_hex(&fact.good.as_bytes()),
                unit_id: digest_hex(&fact.unit.as_bytes()),
                requested: fact.requested,
                feasible_quantity: row.feasible_quantity,
                handled_quantity: row.handled_quantity,
                needed_hours: row.needed_hours,
                used_hours: row.used_hours,
                remaining_unshipped: fact.remaining,
            });
    }
    if !expected.is_empty() {
        return Err(ProductionProjectionError::State);
    }
    for rows in result.values_mut() {
        rows.sort_unstable();
    }
    Ok(result)
}

fn final_demand_accounts(
    catalog: &MichiganMaterialCatalog,
    current: &MaterialCircuitState,
    completed: Option<(&MaterialCircuitState, &[OutboundFact])>,
) -> Result<Vec<ProductionFinalDemandAccount>> {
    let mut groups = BTreeMap::<DemandKey, Vec<_>>::new();
    for order in &current.final_demand_orders {
        groups
            .entry((order.demand_principal_id, order.good_id, order.unit_id))
            .or_default()
            .push(order);
    }
    groups.into_iter().map(|((principal, good, unit), orders)| {
        let county = current.final_demand_principals.iter().find(|row| row.id == principal).ok_or(ProductionProjectionError::State)?;
        let material = catalog.goods().iter().find(|row| row.id() == good && row.unit_id() == unit).ok_or(ProductionProjectionError::Content)?;
        let retailers: BTreeSet<_> = orders.iter().map(|row| row.retailer_site_id).collect();
        let ordered = sum(orders.iter().map(|row| row.ordered))?;
        let fulfilled = sum(orders.iter().map(|row| row.fulfilled))?;
        let outstanding = ordered.checked_sub(fulfilled).ok_or(ProductionProjectionError::State)?;
        let retail_stock_on_hand = sum(current.inventory.iter().filter(|row| retailers.contains(&row.site_id) && row.good_id == good && row.unit_id == unit).map(|row| row.quantity))?;
        let completed = completed.map(|(prior, facts)| {
            let opening_fulfilled = sum(prior.final_demand_orders.iter().filter(|row| row.demand_principal_id == principal && row.good_id == good && row.unit_id == unit).map(|row| row.fulfilled))?;
            let ids: BTreeSet<_> = orders.iter().map(|row| row.order_id).collect();
            let newly_fulfilled = sum(facts.iter().filter(|fact| matches!(fact.id, OutboundOrderId::LocalFinalDemand(id) if ids.contains(&id))).map(|fact| fact.quantity))?;
            if opening_fulfilled.checked_add(newly_fulfilled) != Some(fulfilled) { return Err(ProductionProjectionError::State); }
            Ok(CompletedProductionFinalDemand { period: prior.period, opening_fulfilled, newly_fulfilled, closing_fulfilled: fulfilled })
        }).transpose()?;
        Ok(ProductionFinalDemandAccount { demand_principal_id: digest_hex(&principal.as_bytes()),
            county_geoid: String::from_utf8(county.county_geoid.to_vec()).map_err(|_| ProductionProjectionError::State)?,
            good_id: digest_hex(&good.as_bytes()), unit_id: digest_hex(&unit.as_bytes()), good: material.label.clone(), unit: material.unit_key.clone(),
            ordered, fulfilled, outstanding, retail_stock_on_hand,
            retailer_site_ids: retailers.iter().map(|id| digest_hex(&id.as_bytes())).collect(),
            orders: orders.iter().map(|row| Ok(ProductionFinalDemandOrder {order_id: digest_hex(&row.order_id.as_bytes()),
                retailer_site_id: digest_hex(&row.retailer_site_id.as_bytes()), ordered: row.ordered, fulfilled: row.fulfilled,
                outstanding: row.ordered.checked_sub(row.fulfilled).ok_or(ProductionProjectionError::State)? })).collect::<Result<_>>()?, completed })
    }).collect()
}

fn sum(values: impl IntoIterator<Item = u64>) -> Result<u64> {
    values.into_iter().try_fold(0_u64, |sum, value| {
        sum.checked_add(value)
            .ok_or(ProductionProjectionError::Arithmetic)
    })
}
