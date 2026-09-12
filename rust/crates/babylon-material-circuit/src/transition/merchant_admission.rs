//! Complete merchant and local final-demand admission before any allocation.

use super::{grams_per_unit, has_duplicate, site_node, BTreeSet};
use crate::{
    GoodId, MaterialCircuitError, MaterialCircuitState, MerchantHandling, MerchantRole, SiteId,
    UnitId,
};

pub(super) fn merchant(state: &MaterialCircuitState, site: SiteId) -> Option<&MerchantHandling> {
    state
        .merchants
        .binary_search_by_key(&site, |row| row.site_id)
        .ok()
        .map(|index| &state.merchants[index])
}

pub(super) fn hours_per_unit(
    state: &MaterialCircuitState,
    site: SiteId,
    good: GoodId,
    unit: UnitId,
) -> Result<u64, MaterialCircuitError> {
    state
        .handling_coefficients
        .binary_search_by_key(&(site, good, unit), |row| {
            (row.site_id, row.good_id, row.unit_id)
        })
        .ok()
        .map(|index| state.handling_coefficients[index].hours_per_unit)
        .filter(|hours| *hours > 0)
        .ok_or(MaterialCircuitError::MerchantInvariant)
}

pub(super) fn validate_merchants(state: &MaterialCircuitState) -> Result<(), MaterialCircuitError> {
    if has_duplicate(&state.merchants, |row| row.site_id)
        || has_duplicate(&state.handling_coefficients, |row| {
            (row.site_id, row.good_id, row.unit_id)
        })
        || has_duplicate(&state.final_demand_principals, |row| row.id)
        || has_duplicate(&state.final_demand_orders, |row| row.order_id)
    {
        return Err(MaterialCircuitError::DuplicateRow);
    }
    let production_sites: BTreeSet<_> = state
        .process_outputs
        .iter()
        .map(|row| row.site_id)
        .collect();
    let road_principals: BTreeSet<_> = state
        .route_stage_capacities
        .iter()
        .map(|row| row.corridor_id)
        .collect();
    let mut handling_principals = BTreeSet::new();
    for row in &state.merchants {
        if !row.county_geoid.iter().all(u8::is_ascii_digit)
            || site_node(state, row.site_id).is_none()
            || production_sites.contains(&row.site_id)
            || road_principals.contains(&row.capacity_id)
        {
            return Err(MaterialCircuitError::MerchantInvariant);
        }
        if !handling_principals.insert(row.capacity_id) {
            return Err(MaterialCircuitError::DuplicateRow);
        }
    }
    for row in &state.handling_coefficients {
        if merchant(state, row.site_id).is_none() || row.hours_per_unit == 0 {
            return Err(MaterialCircuitError::MerchantInvariant);
        }
        grams_per_unit(state, row.good_id, row.unit_id)?;
    }
    for row in &state.orders {
        if merchant(state, row.supplier_site_id).is_some() {
            hours_per_unit(state, row.supplier_site_id, row.good_id, row.unit_id)?;
        }
    }
    let mut counties = BTreeSet::new();
    for row in &state.final_demand_principals {
        if !row.county_geoid.iter().all(u8::is_ascii_digit) {
            return Err(MaterialCircuitError::FinalDemandInvariant);
        }
        if !counties.insert(row.county_geoid) {
            return Err(MaterialCircuitError::DuplicateRow);
        }
    }
    for row in &state.final_demand_orders {
        let retailer = merchant(state, row.retailer_site_id)
            .ok_or(MaterialCircuitError::FinalDemandInvariant)?;
        let principal = state
            .final_demand_principals
            .binary_search_by_key(&row.demand_principal_id, |row| row.id)
            .ok()
            .map(|index| &state.final_demand_principals[index])
            .ok_or(MaterialCircuitError::FinalDemandInvariant)?;
        if retailer.role != MerchantRole::Retail
            || retailer.county_geoid != principal.county_geoid
            || row.ordered == 0
            || row.fulfilled > row.ordered
        {
            return Err(MaterialCircuitError::FinalDemandInvariant);
        }
        hours_per_unit(state, row.retailer_site_id, row.good_id, row.unit_id)?;
        grams_per_unit(state, row.good_id, row.unit_id)?;
    }
    Ok(())
}
