//! Committed merchant handling and finite county demand, without allocation.

use super::{
    outbound::{completed_facts, identity, mass, OutboundFact},
    ProductionProjectionErrorV1,
};
use crate::{
    michigan_economy::digest_hex, michigan_material::MichiganMaterialCatalogV1,
    CompletedProductionFinalDemandV2, CompletedProductionMerchantHandlingV2,
    ProductionFinalDemandAccountV2, ProductionFinalDemandOrderV2, ProductionHandlingCoefficientV2,
    ProductionMerchantHandlingAccountV2, ProductionMerchantHandlingOrderV2,
};
use babylon_material_circuit::{
    FinalDemandPrincipalIdV3, GoodIdV1, MaterialCircuitStateV3, OutboundOrderIdV3, SiteIdV1,
    UnitIdV1,
};
use babylon_tick::material_world::MaterialTickReceiptsV4;
use std::collections::{BTreeMap, BTreeSet};

type Result<T> = std::result::Result<T, ProductionProjectionErrorV1>;
type DemandKey = (FinalDemandPrincipalIdV3, GoodIdV1, UnitIdV1);

pub(super) fn project_merchants(
    catalog: &MichiganMaterialCatalogV1,
    current: &MaterialCircuitStateV3,
    prior: Option<&MaterialCircuitStateV3>,
    receipt: Option<&MaterialTickReceiptsV4>,
) -> Result<(
    Vec<ProductionMerchantHandlingAccountV2>,
    Vec<ProductionFinalDemandAccountV2>,
)> {
    let completed = match (prior, receipt) {
        (None, None) if current.period == 1 => None,
        (Some(prior), Some(receipt)) => {
            Some((prior, receipt, completed_facts(prior, current, receipt)?))
        }
        _ => return Err(ProductionProjectionErrorV1::History),
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
    current: &MaterialCircuitStateV3,
    completed: Option<(
        &MaterialCircuitStateV3,
        &MaterialTickReceiptsV4,
        &[OutboundFact],
    )>,
) -> Result<Vec<ProductionMerchantHandlingAccountV2>> {
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
                        return Err(ProductionProjectionErrorV1::State);
                    }
                    Ok(ProductionHandlingCoefficientV2 {
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
                                .ok_or(ProductionProjectionErrorV1::State)?;
                            row.handled_quantity
                                .checked_mul(coefficient.grams_per_unit)
                                .ok_or(ProductionProjectionErrorV1::Arithmetic)
                        })
                        .collect::<Result<Vec<_>>>()?)?;
                    Ok(CompletedProductionMerchantHandlingV2 {
                        period: current.period - 1,
                        needed_hours,
                        used_hours,
                        handled_grams,
                        orders,
                    })
                })
                .transpose()?;
            Ok(ProductionMerchantHandlingAccountV2 {
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
    prior: &MaterialCircuitStateV3,
    receipt: &MaterialTickReceiptsV4,
    facts: &[OutboundFact],
) -> Result<BTreeMap<SiteIdV1, Vec<ProductionMerchantHandlingOrderV2>>> {
    let merchants: BTreeSet<_> = prior.merchants.iter().map(|row| row.site_id).collect();
    let mut expected = BTreeMap::new();
    for fact in facts.iter().filter(|fact| merchants.contains(&fact.site)) {
        if expected.insert((fact.site, fact.id), fact).is_some() {
            return Err(ProductionProjectionErrorV1::State);
        }
    }
    let mut result = BTreeMap::<SiteIdV1, Vec<ProductionMerchantHandlingOrderV2>>::new();
    for row in &receipt.handling {
        let fact = expected
            .remove(&(row.site_id, row.order))
            .ok_or(ProductionProjectionErrorV1::State)?;
        let mut coefficients = prior.handling_coefficients.iter().filter(|coefficient| {
            coefficient.site_id == row.site_id
                && coefficient.good_id == fact.good
                && coefficient.unit_id == fact.unit
        });
        let coefficient = coefficients
            .next()
            .ok_or(ProductionProjectionErrorV1::State)?;
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
            return Err(ProductionProjectionErrorV1::State);
        }
        let (id, kind) = identity(row.order);
        result
            .entry(row.site_id)
            .or_default()
            .push(ProductionMerchantHandlingOrderV2 {
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
        return Err(ProductionProjectionErrorV1::State);
    }
    for rows in result.values_mut() {
        rows.sort_unstable();
    }
    Ok(result)
}

fn final_demand_accounts(
    catalog: &MichiganMaterialCatalogV1,
    current: &MaterialCircuitStateV3,
    completed: Option<(&MaterialCircuitStateV3, &[OutboundFact])>,
) -> Result<Vec<ProductionFinalDemandAccountV2>> {
    let mut groups = BTreeMap::<DemandKey, Vec<_>>::new();
    for order in &current.final_demand_orders {
        groups
            .entry((order.demand_principal_id, order.good_id, order.unit_id))
            .or_default()
            .push(order);
    }
    groups.into_iter().map(|((principal, good, unit), orders)| {
        let county = current.final_demand_principals.iter().find(|row| row.id == principal).ok_or(ProductionProjectionErrorV1::State)?;
        let material = catalog.goods().iter().find(|row| row.id() == good && row.unit_id() == unit).ok_or(ProductionProjectionErrorV1::Content)?;
        let retailers: BTreeSet<_> = orders.iter().map(|row| row.retailer_site_id).collect();
        let ordered = sum(orders.iter().map(|row| row.ordered))?;
        let fulfilled = sum(orders.iter().map(|row| row.fulfilled))?;
        let outstanding = ordered.checked_sub(fulfilled).ok_or(ProductionProjectionErrorV1::State)?;
        let retail_stock_on_hand = sum(current.inventory.iter().filter(|row| retailers.contains(&row.site_id) && row.good_id == good && row.unit_id == unit).map(|row| row.quantity))?;
        let completed = completed.map(|(prior, facts)| {
            let opening_fulfilled = sum(prior.final_demand_orders.iter().filter(|row| row.demand_principal_id == principal && row.good_id == good && row.unit_id == unit).map(|row| row.fulfilled))?;
            let ids: BTreeSet<_> = orders.iter().map(|row| row.order_id).collect();
            let newly_fulfilled = sum(facts.iter().filter(|fact| matches!(fact.id, OutboundOrderIdV3::LocalFinalDemand(id) if ids.contains(&id))).map(|fact| fact.quantity))?;
            if opening_fulfilled.checked_add(newly_fulfilled) != Some(fulfilled) { return Err(ProductionProjectionErrorV1::State); }
            Ok(CompletedProductionFinalDemandV2 { period: prior.period, opening_fulfilled, newly_fulfilled, closing_fulfilled: fulfilled })
        }).transpose()?;
        Ok(ProductionFinalDemandAccountV2 { demand_principal_id: digest_hex(&principal.as_bytes()),
            county_geoid: String::from_utf8(county.county_geoid.to_vec()).map_err(|_| ProductionProjectionErrorV1::State)?,
            good_id: digest_hex(&good.as_bytes()), unit_id: digest_hex(&unit.as_bytes()), good: material.label.clone(), unit: material.unit_key.clone(),
            ordered, fulfilled, outstanding, retail_stock_on_hand,
            retailer_site_ids: retailers.iter().map(|id| digest_hex(&id.as_bytes())).collect(),
            orders: orders.iter().map(|row| Ok(ProductionFinalDemandOrderV2 {order_id: digest_hex(&row.order_id.as_bytes()),
                retailer_site_id: digest_hex(&row.retailer_site_id.as_bytes()), ordered: row.ordered, fulfilled: row.fulfilled,
                outstanding: row.ordered.checked_sub(row.fulfilled).ok_or(ProductionProjectionErrorV1::State)? })).collect::<Result<_>>()?, completed })
    }).collect()
}

fn sum(values: impl IntoIterator<Item = u64>) -> Result<u64> {
    values.into_iter().try_fold(0_u64, |sum, value| {
        sum.checked_add(value)
            .ok_or(ProductionProjectionErrorV1::Arithmetic)
    })
}
