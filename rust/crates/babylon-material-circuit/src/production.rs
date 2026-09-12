//! The single private production reducer shared by current material transitions.

use std::collections::{BTreeMap, BTreeSet};

use crate::inventory::{
    credit_inventory, debit_inventory, publish_inventory, take_inventory, InventoryKey,
    InventoryLedger,
};
use crate::transition::has_duplicate;
use crate::MaterialCircuitState;
use crate::{
    MaterialCircuitError, ProcessId, ProductionReceipt, SiteId, UnitId, MAX_MATERIAL_CIRCUIT_ROWS,
    MAX_PRODUCTION_RESOURCE_GROUPS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ProductionResourceKey {
    Input(InventoryKey),
    Labor(SiteId, UnitId),
}

#[derive(Debug, Clone, Copy)]
struct ProductionResourceRequest {
    commitment_index: usize,
    quantity_per_batch: u64,
    requested: u128,
}

#[derive(Clone, Copy)]
enum ProductionResources {
    InputsOnly,
    InputsAndLabor,
}

/// Complete process demand before labor capacity constrains next-period planning.
pub(crate) struct ProcessLaborRequest {
    pub(crate) process_id: ProcessId,
    pub(crate) site_id: SiteId,
    pub(crate) unit_id: UnitId,
    pub(crate) hours: u64,
}

fn process_output(
    state: &MaterialCircuitState,
    process: ProcessId,
) -> Option<&crate::ProcessOutput> {
    state
        .process_outputs
        .binary_search_by_key(&process, |row| row.process_id)
        .ok()
        .map(|index| &state.process_outputs[index])
}

fn labor_coefficient(
    state: &MaterialCircuitState,
    process: ProcessId,
) -> Option<&crate::LaborCoefficient> {
    state
        .labor_coefficients
        .binary_search_by_key(&process, |row| row.process_id)
        .ok()
        .map(|index| &state.labor_coefficients[index])
}

fn input_coefficients(
    state: &MaterialCircuitState,
    process: ProcessId,
) -> &[crate::InputOutputCoefficient] {
    let start = state
        .input_coefficients
        .partition_point(|row| row.process_id < process);
    let end = state
        .input_coefficients
        .partition_point(|row| row.process_id <= process);
    &state.input_coefficients[start..end]
}

pub(crate) fn validate_unique_rows(
    state: &MaterialCircuitState,
) -> Result<(), MaterialCircuitError> {
    let duplicate = has_duplicate(&state.process_outputs, |row| row.process_id)
        || has_duplicate(&state.input_coefficients, |row| {
            (row.process_id, row.good_id, row.unit_id)
        })
        || has_duplicate(&state.labor_coefficients, |row| row.process_id)
        || has_duplicate(&state.inventory, |row| {
            (row.site_id, row.good_id, row.unit_id)
        })
        || has_duplicate(&state.capacities, |row| {
            (row.period, row.site_id, row.process_id)
        })
        || has_duplicate(&state.labor, |row| (row.period, row.site_id, row.unit_id))
        || has_duplicate(&state.production_commitments, |row| {
            (row.period, row.site_id, row.process_id)
        });
    if duplicate {
        return Err(MaterialCircuitError::DuplicateRow);
    }
    Ok(())
}

pub(crate) fn validate_processes(state: &MaterialCircuitState) -> Result<(), MaterialCircuitError> {
    let process_ids: BTreeSet<_> = state
        .process_outputs
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
        .map(|row| row.process_id)
        .collect();
    for row in state
        .process_outputs
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
    {
        if row.quantity_per_batch == 0 {
            return Err(MaterialCircuitError::ZeroQuantity);
        }
    }
    for row in state
        .input_coefficients
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
    {
        if row.quantity_per_batch == 0 || !process_ids.contains(&row.process_id) {
            return Err(MaterialCircuitError::ProcessInvariant);
        }
    }
    for row in state
        .labor_coefficients
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
    {
        if row.quantity_per_batch == 0 || !process_ids.contains(&row.process_id) {
            return Err(MaterialCircuitError::ProcessInvariant);
        }
    }
    if state.process_outputs.len() != state.labor_coefficients.len() {
        return Err(MaterialCircuitError::ProcessInvariant);
    }
    for row in state.capacities.iter().take(MAX_MATERIAL_CIRCUIT_ROWS + 1) {
        let output = process_output(state, row.process_id);
        if output.is_none_or(|output| output.site_id != row.site_id) {
            return Err(MaterialCircuitError::ProcessInvariant);
        }
    }
    for row in state
        .production_commitments
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
    {
        let output = process_output(state, row.process_id);
        if output.is_none_or(|output| output.site_id != row.site_id) {
            return Err(MaterialCircuitError::ProcessInvariant);
        }
    }
    Ok(())
}

pub(crate) fn validate_periods(state: &MaterialCircuitState) -> Result<(), MaterialCircuitError> {
    if state.period == 0
        || state
            .production_commitments
            .iter()
            .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
            .any(|row| row.period != state.period || row.planned_batches == 0)
        || state
            .capacities
            .iter()
            .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
            .any(|row| row.period < state.period)
        || state
            .labor
            .iter()
            .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
            .any(|row| row.period < state.period)
    {
        return Err(MaterialCircuitError::PeriodInvariant);
    }
    Ok(())
}

fn process_capacity(
    state: &MaterialCircuitState,
    process: ProcessId,
    site: SiteId,
    period: u64,
) -> u64 {
    state
        .capacities
        .binary_search_by_key(&(period, site, process), |row| {
            (row.period, row.site_id, row.process_id)
        })
        .ok()
        .map_or(0, |index| state.capacities[index].available_batches)
}

fn labor_capacity_index(
    state: &MaterialCircuitState,
    site: SiteId,
    unit: UnitId,
    period: u64,
) -> Option<usize> {
    state
        .labor
        .binary_search_by_key(&(period, site, unit), |row| {
            (row.period, row.site_id, row.unit_id)
        })
        .ok()
}

fn initial_production_allocations(
    state: &MaterialCircuitState,
    commitments: &[crate::ProductionCommitment],
    period: u64,
) -> Result<Vec<u64>, MaterialCircuitError> {
    let mut allocations = Vec::with_capacity(commitments.len());
    for commitment in commitments.iter().take(MAX_MATERIAL_CIRCUIT_ROWS + 1) {
        let output = process_output(state, commitment.process_id)
            .ok_or(MaterialCircuitError::ProcessInvariant)?;
        if output.site_id != commitment.site_id || commitment.period != period {
            return Err(MaterialCircuitError::ProcessInvariant);
        }
        allocations.push(commitment.planned_batches.min(process_capacity(
            state,
            commitment.process_id,
            commitment.site_id,
            period,
        )));
    }
    Ok(allocations)
}

fn add_production_request(
    groups: &mut BTreeMap<ProductionResourceKey, Vec<ProductionResourceRequest>>,
    key: ProductionResourceKey,
    commitment_index: usize,
    quantity_per_batch: u64,
    batches: u64,
) -> Result<(), MaterialCircuitError> {
    let requested = u128::from(quantity_per_batch)
        .checked_mul(u128::from(batches))
        .ok_or(MaterialCircuitError::Arithmetic)?;
    groups
        .entry(key)
        .or_default()
        .push(ProductionResourceRequest {
            commitment_index,
            quantity_per_batch,
            requested,
        });
    Ok(())
}

fn production_resource_groups(
    state: &MaterialCircuitState,
    commitments: &[crate::ProductionCommitment],
    allocations: &[u64],
    resources: ProductionResources,
) -> Result<BTreeMap<ProductionResourceKey, Vec<ProductionResourceRequest>>, MaterialCircuitError> {
    let mut groups = BTreeMap::new();
    for (index, commitment) in commitments
        .iter()
        .enumerate()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
    {
        let labor = labor_coefficient(state, commitment.process_id)
            .ok_or(MaterialCircuitError::ProcessInvariant)?;
        if matches!(resources, ProductionResources::InputsAndLabor) {
            add_production_request(
                &mut groups,
                ProductionResourceKey::Labor(commitment.site_id, labor.unit_id),
                index,
                labor.quantity_per_batch,
                allocations[index],
            )?;
        }
        for input in input_coefficients(state, commitment.process_id)
            .iter()
            .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
        {
            add_production_request(
                &mut groups,
                ProductionResourceKey::Input((commitment.site_id, input.good_id, input.unit_id)),
                index,
                input.quantity_per_batch,
                allocations[index],
            )?;
        }
    }
    Ok(groups)
}

fn production_resource_available(
    state: &MaterialCircuitState,
    inventory: &InventoryLedger,
    key: ProductionResourceKey,
    period: u64,
) -> u64 {
    match key {
        ProductionResourceKey::Input(inventory_key) => {
            inventory.get(&inventory_key).copied().unwrap_or(0)
        }
        ProductionResourceKey::Labor(site, unit) => labor_capacity_index(state, site, unit, period)
            .map_or(0, |index| state.labor[index].available),
    }
}

fn wide_product(multiplier: u64, multiplicand: u128) -> [u64; 3] {
    let low_mask = u128::from(u64::MAX);
    let low_multiplicand =
        u64::try_from(multiplicand & low_mask).expect("masking a u128 to 64 low bits must fit u64");
    let low_product = u128::from(multiplier) * u128::from(low_multiplicand);
    let high_product = u128::from(multiplier) * (multiplicand >> 64) + (low_product >> 64);
    let high_limb =
        u64::try_from(high_product >> 64).expect("shifting a u128 right by 64 bits must fit u64");
    let middle_limb =
        u64::try_from(high_product & low_mask).expect("masking a u128 to 64 low bits must fit u64");
    let low_limb =
        u64::try_from(low_product & low_mask).expect("masking a u128 to 64 low bits must fit u64");
    [high_limb, middle_limb, low_limb]
}

pub(crate) fn proportional_floor(
    available: u64,
    requested: u128,
    total: u128,
) -> Result<u64, MaterialCircuitError> {
    if total == 0 || requested > total {
        return Err(MaterialCircuitError::Arithmetic);
    }
    let target = wide_product(available, requested);
    let mut quotient = 0_u64;
    for bit in (0_u32..64).rev() {
        let candidate = quotient | (1_u64 << bit);
        if candidate <= available && wide_product(candidate, total) <= target {
            quotient = candidate;
        }
    }
    Ok(quotient)
}

fn apply_production_resource_limits(
    state: &MaterialCircuitState,
    inventory: &InventoryLedger,
    period: u64,
    groups: &BTreeMap<ProductionResourceKey, Vec<ProductionResourceRequest>>,
    allocations: &mut [u64],
) -> Result<(), MaterialCircuitError> {
    if groups.len() > MAX_PRODUCTION_RESOURCE_GROUPS {
        return Err(MaterialCircuitError::RowLimit);
    }
    for (key, requests) in groups.iter().take(MAX_PRODUCTION_RESOURCE_GROUPS) {
        let total = requests
            .iter()
            .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
            .try_fold(0_u128, |sum, request| {
                sum.checked_add(request.requested)
                    .ok_or(MaterialCircuitError::Arithmetic)
            })?;
        let available = production_resource_available(state, inventory, *key, period);
        for request in requests.iter().take(MAX_MATERIAL_CIRCUIT_ROWS + 1) {
            let granted_units = if u128::from(available) >= total {
                request.requested
            } else {
                u128::from(proportional_floor(available, request.requested, total)?)
            };
            let granted_batches = granted_units / u128::from(request.quantity_per_batch);
            let granted_batches =
                u64::try_from(granted_batches).map_err(|_| MaterialCircuitError::Arithmetic)?;
            allocations[request.commitment_index] =
                allocations[request.commitment_index].min(granted_batches);
        }
    }
    Ok(())
}

fn allocate_production_batches(
    state: &MaterialCircuitState,
    inventory: &InventoryLedger,
    commitments: &[crate::ProductionCommitment],
    period: u64,
    resources: ProductionResources,
) -> Result<Vec<u64>, MaterialCircuitError> {
    let mut allocations = initial_production_allocations(state, commitments, period)?;
    let groups = production_resource_groups(state, commitments, &allocations, resources)?;
    apply_production_resource_limits(state, inventory, period, &groups, &mut allocations)?;
    Ok(allocations)
}

fn execute_production(
    state: &mut MaterialCircuitState,
    inventory: &mut InventoryLedger,
    receipts: &mut Vec<ProductionReceipt>,
) -> Result<(), MaterialCircuitError> {
    let commitments = std::mem::take(&mut state.production_commitments);
    let allocations = allocate_production_batches(
        state,
        inventory,
        &commitments,
        state.period,
        ProductionResources::InputsAndLabor,
    )?;
    debit_production_allocations(state, inventory, &commitments, &allocations)?;
    credit_production_allocations(state, inventory, commitments, &allocations, receipts)
}

pub(crate) fn execute_shared_production(
    state: &mut MaterialCircuitState,
) -> Result<Vec<ProductionReceipt>, MaterialCircuitError> {
    let mut inventory = take_inventory(state);
    let mut receipts = Vec::new();
    execute_production(state, &mut inventory, &mut receipts)?;
    publish_inventory(state, inventory);
    Ok(receipts)
}

fn debit_production_allocations(
    state: &mut MaterialCircuitState,
    inventory: &mut InventoryLedger,
    commitments: &[crate::ProductionCommitment],
    allocations: &[u64],
) -> Result<(), MaterialCircuitError> {
    for (index, commitment) in commitments
        .iter()
        .enumerate()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
    {
        consume_production_inputs(
            state,
            inventory,
            commitment.process_id,
            commitment.site_id,
            allocations[index],
        )?;
    }
    Ok(())
}

fn credit_production_allocations(
    state: &mut MaterialCircuitState,
    inventory: &mut InventoryLedger,
    commitments: Vec<crate::ProductionCommitment>,
    allocations: &[u64],
    receipts: &mut Vec<ProductionReceipt>,
) -> Result<(), MaterialCircuitError> {
    for (index, commitment) in commitments
        .into_iter()
        .enumerate()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
    {
        let output = process_output(state, commitment.process_id)
            .cloned()
            .ok_or(MaterialCircuitError::ProcessInvariant)?;
        let batches = allocations[index];
        let produced = output
            .quantity_per_batch
            .checked_mul(batches)
            .ok_or(MaterialCircuitError::Arithmetic)?;
        credit_inventory(
            inventory,
            (output.site_id, output.good_id, output.unit_id),
            produced,
        )?;
        receipts.push(ProductionReceipt {
            process_id: commitment.process_id,
            site_id: commitment.site_id,
            planned_batches: commitment.planned_batches,
            produced_batches: batches,
        });
    }
    Ok(())
}

fn consume_production_inputs(
    state: &mut MaterialCircuitState,
    inventory: &mut InventoryLedger,
    process: ProcessId,
    site: SiteId,
    batches: u64,
) -> Result<(), MaterialCircuitError> {
    if batches == 0 {
        return Ok(());
    }
    let inputs: Vec<_> = input_coefficients(state, process)
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
        .map(|row| (row.good_id, row.unit_id, row.quantity_per_batch))
        .collect();
    for (good, unit, quantity_per_batch) in inputs.into_iter().take(MAX_MATERIAL_CIRCUIT_ROWS + 1) {
        let quantity = quantity_per_batch
            .checked_mul(batches)
            .ok_or(MaterialCircuitError::Arithmetic)?;
        debit_inventory(
            inventory,
            (site, good, unit),
            quantity,
            MaterialCircuitError::ProcessInvariant,
        )?;
    }
    let labor = labor_coefficient(state, process)
        .cloned()
        .ok_or(MaterialCircuitError::ProcessInvariant)?;
    let labor_used = labor
        .quantity_per_batch
        .checked_mul(batches)
        .ok_or(MaterialCircuitError::Arithmetic)?;
    let index = labor_capacity_index(state, site, labor.unit_id, state.period)
        .ok_or(MaterialCircuitError::ProcessInvariant)?;
    state.labor[index].available = state.labor[index]
        .available
        .checked_sub(labor_used)
        .ok_or(MaterialCircuitError::Arithmetic)?;
    Ok(())
}

fn next_period_candidates(
    state: &MaterialCircuitState,
    next_period: u64,
) -> Vec<crate::ProductionCommitment> {
    state
        .process_outputs
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
        .map(|output| crate::ProductionCommitment {
            process_id: output.process_id,
            site_id: output.site_id,
            period: next_period,
            planned_batches: process_capacity(
                state,
                output.process_id,
                output.site_id,
                next_period,
            ),
        })
        .collect()
}

fn derive_next_period_production(
    state: &mut MaterialCircuitState,
    inventory: &InventoryLedger,
    next_period: u64,
) -> Result<(), MaterialCircuitError> {
    let candidates = next_period_candidates(state, next_period);
    let allocations = allocate_production_batches(
        state,
        inventory,
        &candidates,
        next_period,
        ProductionResources::InputsAndLabor,
    )?;
    for (index, candidate) in candidates
        .into_iter()
        .enumerate()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
    {
        let batches = allocations[index];
        if batches > 0 {
            state
                .production_commitments
                .push(crate::ProductionCommitment {
                    process_id: candidate.process_id,
                    site_id: candidate.site_id,
                    period: next_period,
                    planned_batches: batches,
                });
        }
    }
    Ok(())
}

/// Uses the same simultaneous shared-input allocation without a labor resource group.
/// This reads the closed inventory and returns zeros too; it does not publish plans.
pub(crate) fn derive_shared_labor_requests(
    state: &MaterialCircuitState,
    next_period: u64,
) -> Result<Vec<ProcessLaborRequest>, MaterialCircuitError> {
    let inventory = state
        .inventory
        .iter()
        .map(|row| ((row.site_id, row.good_id, row.unit_id), row.quantity))
        .collect();
    let candidates = next_period_candidates(state, next_period);
    let allocations = allocate_production_batches(
        state,
        &inventory,
        &candidates,
        next_period,
        ProductionResources::InputsOnly,
    )?;
    candidates
        .iter()
        .zip(allocations)
        .map(|(candidate, batches)| {
            let coefficient = labor_coefficient(state, candidate.process_id)
                .ok_or(MaterialCircuitError::ProcessInvariant)?;
            Ok(ProcessLaborRequest {
                process_id: candidate.process_id,
                site_id: candidate.site_id,
                unit_id: coefficient.unit_id,
                hours: batches
                    .checked_mul(coefficient.quantity_per_batch)
                    .ok_or(MaterialCircuitError::Arithmetic)?,
            })
        })
        .collect()
}

pub(crate) fn derive_shared_production(
    state: &mut MaterialCircuitState,
    next_period: u64,
) -> Result<(), MaterialCircuitError> {
    let inventory = take_inventory(state);
    derive_next_period_production(state, &inventory, next_period)?;
    prune_consumed_capacity(state, next_period);
    publish_inventory(state, inventory);
    Ok(())
}

fn prune_consumed_capacity(state: &mut MaterialCircuitState, next_period: u64) {
    state.capacities = std::mem::take(&mut state.capacities)
        .into_iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
        .filter(|row| row.period >= next_period)
        .collect();
    state.labor = std::mem::take(&mut state.labor)
        .into_iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
        .filter(|row| row.period >= next_period)
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GoodId;

    fn numbered_identity(index: usize) -> [u8; 32] {
        let mut bytes = [0_u8; 32];
        let number = u64::try_from(index).expect("designed group bound fits u64");
        bytes[24..].copy_from_slice(&number.to_be_bytes());
        bytes
    }

    fn empty_state() -> MaterialCircuitState {
        MaterialCircuitState {
            period: 1,
            site_logistics_nodes: Vec::new(),
            process_outputs: Vec::new(),
            input_coefficients: Vec::new(),
            labor_coefficients: Vec::new(),
            freight_mass_coefficients: Vec::new(),
            supplier_routes: Vec::new(),
            route_stages: Vec::new(),
            route_stage_capacities: Vec::new(),
            inventory: Vec::new(),
            orders: Vec::new(),
            backlog: Vec::new(),
            freight: Vec::new(),
            corridor_capacities: Vec::new(),
            capacities: Vec::new(),
            labor: Vec::new(),
            production_commitments: Vec::new(),
            merchants: Vec::new(),
            handling_coefficients: Vec::new(),
            final_demand_principals: Vec::new(),
            final_demand_orders: Vec::new(),
        }
    }

    #[test]
    fn resource_group_bound_covers_both_families_and_refuses_plus_one() {
        let site = SiteId::from_bytes([1; 32]);
        let unit = UnitId::from_bytes([2; 32]);
        let mut groups = BTreeMap::new();
        for index in 0..MAX_PRODUCTION_RESOURCE_GROUPS {
            groups.insert(
                ProductionResourceKey::Input((
                    site,
                    GoodId::from_bytes(numbered_identity(index)),
                    unit,
                )),
                Vec::new(),
            );
        }
        let last_key = ProductionResourceKey::Input((
            site,
            GoodId::from_bytes(numbered_identity(MAX_PRODUCTION_RESOURCE_GROUPS - 1)),
            unit,
        ));
        groups.insert(
            last_key,
            vec![ProductionResourceRequest {
                commitment_index: 0,
                quantity_per_batch: 1,
                requested: 1,
            }],
        );
        let mut allocations = [1];
        assert_eq!(
            apply_production_resource_limits(
                &empty_state(),
                &BTreeMap::new(),
                1,
                &groups,
                &mut allocations,
            ),
            Ok(())
        );
        assert_eq!(allocations, [0]);

        groups.insert(
            ProductionResourceKey::Input((
                site,
                GoodId::from_bytes(numbered_identity(MAX_PRODUCTION_RESOURCE_GROUPS)),
                unit,
            )),
            Vec::new(),
        );
        assert_eq!(
            apply_production_resource_limits(
                &empty_state(),
                &BTreeMap::new(),
                1,
                &groups,
                &mut allocations,
            ),
            Err(MaterialCircuitError::RowLimit)
        );
    }

    #[test]
    fn wide_proportional_floor_is_exact_without_overflow() {
        for available in 0_u64..=20 {
            for total in 1_u128..=20 {
                for requested in 0_u128..=total {
                    assert_eq!(
                        proportional_floor(available, requested, total),
                        Ok(
                            available * u64::try_from(requested).expect("small request fits u64")
                                / u64::try_from(total).expect("small total fits u64")
                        )
                    );
                }
            }
        }
        assert_eq!(
            proportional_floor(u64::MAX, u128::MAX - 1, u128::MAX),
            Ok(u64::MAX - 1)
        );
    }
}
