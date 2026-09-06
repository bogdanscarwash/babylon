//! Ownership, compatible units and finite resource schedules for executable content.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    michigan, SectorBundleErrorV1, SectorBundleV1, UnitIdV1, HORIZON_TICKS, MAX_BUNDLE_GOODS,
    MAX_BUNDLE_PROCESSES,
};

pub(super) fn bundle(value: &SectorBundleV1) -> Result<(), SectorBundleErrorV1> {
    michigan::validate_sources(&value.owner, &value.sources)?;
    let rows = &value.rows;
    if rows.week != 1
        || !rows.orders.is_empty()
        || !rows.backlog.is_empty()
        || !rows.freight.is_empty()
        || !rows.supplier_routes.is_empty()
        || !rows.route_legs.is_empty()
        || !rows.corridor_capacities.is_empty()
        || value.processes.is_empty()
        || value.processes.len() > MAX_BUNDLE_PROCESSES
        || value.goods.is_empty()
        || value.goods.len() > MAX_BUNDLE_GOODS
    {
        return Err(SectorBundleErrorV1::Bound);
    }
    let mut goods = BTreeMap::new();
    for good in &value.goods {
        if good.unit_id == value.labor_unit || goods.insert(good.good_id, good.unit_id).is_some() {
            return Err(SectorBundleErrorV1::GoodUnit);
        }
    }
    let processes: BTreeSet<_> = value.processes.iter().map(|row| row.process_id).collect();
    let outputs: BTreeSet<_> = rows
        .process_outputs
        .iter()
        .map(|row| row.process_id)
        .collect();
    if processes.len() != value.processes.len() || processes != outputs {
        return Err(SectorBundleErrorV1::ProcessOwnership);
    }
    michigan::validate_process_bindings(value)?;
    validate_goods(value, &goods)?;
    validate_resources(value)
}

fn validate_goods(
    value: &SectorBundleV1,
    goods: &BTreeMap<babylon_material_circuit::GoodIdV1, UnitIdV1>,
) -> Result<(), SectorBundleErrorV1> {
    let rows = &value.rows;
    let mut used = BTreeSet::new();
    let mut inventory_keys = BTreeSet::new();
    for output in &rows.process_outputs {
        if goods.get(&output.good_id) != Some(&output.unit_id) {
            return Err(SectorBundleErrorV1::GoodUnit);
        }
        used.insert(output.good_id);
        inventory_keys.insert((output.site_id, output.good_id, output.unit_id));
        for input in rows
            .input_coefficients
            .iter()
            .filter(|row| row.process_id == output.process_id)
        {
            if goods.get(&input.good_id) != Some(&input.unit_id) {
                return Err(SectorBundleErrorV1::GoodUnit);
            }
            used.insert(input.good_id);
            inventory_keys.insert((output.site_id, input.good_id, input.unit_id));
        }
    }
    let actual: BTreeSet<_> = rows
        .inventory
        .iter()
        .map(|row| (row.site_id, row.good_id, row.unit_id))
        .collect();
    if inventory_keys != actual || used != goods.keys().copied().collect() {
        return Err(SectorBundleErrorV1::GoodUnit);
    }
    Ok(())
}

fn validate_resources(value: &SectorBundleV1) -> Result<(), SectorBundleErrorV1> {
    let rows = &value.rows;
    let sites: BTreeSet<_> = rows.process_outputs.iter().map(|row| row.site_id).collect();
    let nodes: BTreeSet<_> = rows
        .site_logistics_nodes
        .iter()
        .map(|row| row.site_id)
        .collect();
    if sites != nodes || sites.len() != rows.site_logistics_nodes.len() {
        return Err(SectorBundleErrorV1::ProcessOwnership);
    }
    let mut expected_capacities = BTreeSet::new();
    let mut expected_labor = BTreeSet::new();
    for output in &rows.process_outputs {
        let labor: Vec<_> = rows
            .labor_coefficients
            .iter()
            .filter(|row| row.process_id == output.process_id)
            .collect();
        if labor.len() != 1 || labor[0].unit_id != value.labor_unit {
            return Err(SectorBundleErrorV1::Resource);
        }
        for week in 1..=HORIZON_TICKS {
            expected_capacities.insert((output.process_id, output.site_id, week));
            expected_labor.insert((output.site_id, value.labor_unit, week));
        }
    }
    let capacities: BTreeSet<_> = rows
        .capacities
        .iter()
        .map(|row| (row.process_id, row.site_id, row.week))
        .collect();
    let labor: BTreeSet<_> = rows
        .labor
        .iter()
        .map(|row| (row.site_id, row.unit_id, row.week))
        .collect();
    if expected_capacities != capacities
        || capacities.len() != rows.capacities.len()
        || expected_labor != labor
        || labor.len() != rows.labor.len()
    {
        return Err(SectorBundleErrorV1::Resource);
    }
    for commitment in &rows.production_commitments {
        let capacity = rows
            .capacities
            .iter()
            .find(|row| row.process_id == commitment.process_id && row.week == 1)
            .ok_or(SectorBundleErrorV1::Resource)?;
        if commitment.planned_batches > capacity.available_batches {
            return Err(SectorBundleErrorV1::Resource);
        }
        for input in rows
            .input_coefficients
            .iter()
            .filter(|row| row.process_id == commitment.process_id)
        {
            let required = input
                .quantity_per_batch
                .checked_mul(commitment.planned_batches)
                .ok_or(SectorBundleErrorV1::Arithmetic)?;
            let stock = rows
                .inventory
                .iter()
                .find(|row| {
                    row.site_id == commitment.site_id
                        && row.good_id == input.good_id
                        && row.unit_id == input.unit_id
                })
                .ok_or(SectorBundleErrorV1::Resource)?;
            if stock.quantity < required {
                return Err(SectorBundleErrorV1::Resource);
            }
        }
        let coefficient = rows
            .labor_coefficients
            .iter()
            .find(|row| row.process_id == commitment.process_id)
            .ok_or(SectorBundleErrorV1::Resource)?;
        let required = coefficient
            .quantity_per_batch
            .checked_mul(commitment.planned_batches)
            .ok_or(SectorBundleErrorV1::Arithmetic)?;
        let labor = rows
            .labor
            .iter()
            .find(|row| row.site_id == commitment.site_id && row.week == 1)
            .ok_or(SectorBundleErrorV1::Resource)?;
        if labor.available < required {
            return Err(SectorBundleErrorV1::Resource);
        }
    }
    Ok(())
}
