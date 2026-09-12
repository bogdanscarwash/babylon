//! Structural owner and resource checks; captured catalog equality supplies authority.
use super::{
    SectorBundle, SectorBundleError, MAX_BUNDLE_GOODS, MAX_BUNDLE_PROCESSES,
    MICHIGAN_MAX_HORIZON_PERIODS,
};
use crate::michigan_cohorts::michigan_business_subject_for_owner;
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn bundle(value: &SectorBundle) -> Result<(), SectorBundleError> {
    let expected =
        michigan_business_subject_for_owner(&value.owner.county_geoid, &value.owner.sector_code);
    if value.owner.subject != expected
        || value.owner.county_geoid.len() != 5
        || !value.owner.county_geoid.starts_with("26")
        || !value.owner.county_geoid.bytes().all(|b| b.is_ascii_digit())
        || !matches!(
            value.owner.sector_code.as_str(),
            "11" | "21" | "31-33" | "42" | "44-45"
        )
    {
        return Err(SectorBundleError::Owner);
    }
    if value.sources.county_source_file.is_empty()
        || [
            value.sources.county_source_sha256,
            value.sources.sector_artifact_sha256,
            value.sources.sector_semantic_sha256,
            value.sources.industry_artifact_sha256,
            value.sources.designed_scenario_sha256,
        ]
        .contains(&[0; 32])
    {
        return Err(SectorBundleError::Source);
    }
    let rows = &value.rows;
    if rows.period != 1
        || !rows.orders.is_empty()
        || !rows.backlog.is_empty()
        || !rows.freight.is_empty()
        || !rows.supplier_routes.is_empty()
        || !rows.route_stages.is_empty()
        || !rows.route_stage_capacities.is_empty()
        || !rows.corridor_capacities.is_empty()
        || !rows.final_demand_orders.is_empty()
        || !rows.final_demand_principals.is_empty()
        || value.processes.len() > MAX_BUNDLE_PROCESSES
        || value.goods.is_empty()
        || value.goods.len() > MAX_BUNDLE_GOODS
        || (value.processes.is_empty() && rows.merchants.is_empty())
    {
        return Err(SectorBundleError::Bound);
    }
    goods(value)?;
    ownership_and_resources(value)?;
    Ok(())
}

fn goods(value: &SectorBundle) -> Result<(), SectorBundleError> {
    let rows = &value.rows;
    let goods: BTreeMap<_, _> = value.goods.iter().map(|g| (g.good_id, g.unit_id)).collect();
    if goods.len() != value.goods.len() || goods.values().any(|unit| *unit == value.labor_unit) {
        return Err(SectorBundleError::GoodUnit);
    }
    let used: BTreeSet<_> = rows
        .inventory
        .iter()
        .map(|r| (r.good_id, r.unit_id))
        .collect();
    if used != goods.iter().map(|(g, u)| (*g, *u)).collect() {
        return Err(SectorBundleError::GoodUnit);
    }
    if rows
        .process_outputs
        .iter()
        .any(|r| goods.get(&r.good_id) != Some(&r.unit_id))
        || rows
            .input_coefficients
            .iter()
            .any(|r| goods.get(&r.good_id) != Some(&r.unit_id))
        || rows
            .handling_coefficients
            .iter()
            .any(|r| goods.get(&r.good_id) != Some(&r.unit_id))
    {
        return Err(SectorBundleError::GoodUnit);
    }

    Ok(())
}

fn ownership_and_resources(value: &SectorBundle) -> Result<(), SectorBundleError> {
    let rows = &value.rows;
    let processes: BTreeSet<_> = value.processes.iter().map(|p| p.process_id).collect();
    if processes.len() != value.processes.len()
        || processes != rows.process_outputs.iter().map(|r| r.process_id).collect()
    {
        return Err(SectorBundleError::ProcessOwnership);
    }
    let sites: BTreeSet<_> = rows
        .process_outputs
        .iter()
        .map(|p| p.site_id)
        .chain(rows.merchants.iter().map(|m| m.site_id))
        .collect();
    if sites
        != rows
            .site_logistics_nodes
            .iter()
            .map(|n| n.site_id)
            .collect()
        || sites.len() != rows.site_logistics_nodes.len()
        || rows.inventory.iter().any(|r| !sites.contains(&r.site_id))
    {
        return Err(SectorBundleError::ProcessOwnership);
    }
    let expected_labor: BTreeSet<_> = sites.iter().map(|s| (*s, value.labor_unit, 1)).collect();
    if expected_labor
        != rows
            .labor
            .iter()
            .map(|r| (r.site_id, r.unit_id, r.period))
            .collect()
        || expected_labor.len() != rows.labor.len()
    {
        return Err(SectorBundleError::Resource);
    }
    let mut expected_capacity = BTreeSet::new();
    for output in &rows.process_outputs {
        let labor: Vec<_> = rows
            .labor_coefficients
            .iter()
            .filter(|r| r.process_id == output.process_id)
            .collect();
        if labor.len() != 1 || labor[0].unit_id != value.labor_unit {
            return Err(SectorBundleError::Resource);
        }
        for period in 1..=MICHIGAN_MAX_HORIZON_PERIODS {
            expected_capacity.insert((output.process_id, output.site_id, period));
        }
    }
    if expected_capacity
        != rows
            .capacities
            .iter()
            .map(|r| (r.process_id, r.site_id, r.period))
            .collect()
        || expected_capacity.len() != rows.capacities.len()
    {
        return Err(SectorBundleError::Resource);
    }
    for commitment in &rows.production_commitments {
        let capacity = rows
            .capacities
            .iter()
            .find(|r| r.process_id == commitment.process_id && r.period == 1)
            .ok_or(SectorBundleError::Resource)?;
        if commitment.period != 1 || commitment.planned_batches > capacity.available_batches {
            return Err(SectorBundleError::Resource);
        }
    }

    Ok(())
}
