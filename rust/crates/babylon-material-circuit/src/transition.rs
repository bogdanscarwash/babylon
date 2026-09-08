//! Pure per-period transition for the exact local material circuit.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ArrivalReceiptV1, BacklogRowV1, DeliveryReceiptV1, DispatchReceiptV1, GoodIdV1, InventoryRowV1,
    MaterialCircuitErrorV1, MaterialCircuitStateV1, MaterialCircuitTransitionV1, OrderIdV1,
    ProcessIdV1, ProductionReceiptV1, RealizationReceiptV1, SiteIdV1, TransitLotV1, UnitIdV1,
    MAX_MATERIAL_CIRCUIT_ROWS_V1, MAX_PRODUCTION_RESOURCE_GROUPS_V1,
};

type InventoryKey = (SiteIdV1, GoodIdV1, UnitIdV1);
type InventoryLedger = BTreeMap<InventoryKey, u64>;
type SupplierKey = (SiteIdV1, SiteIdV1, GoodIdV1, UnitIdV1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ProductionResourceKey {
    Input(InventoryKey),
    Labor(SiteIdV1, UnitIdV1),
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
    pub(crate) process_id: ProcessIdV1,
    pub(crate) site_id: SiteIdV1,
    pub(crate) unit_id: UnitIdV1,
    pub(crate) hours: u64,
}

fn process_output(
    state: &MaterialCircuitStateV1,
    process: ProcessIdV1,
) -> Option<&crate::ProcessOutputV1> {
    state
        .process_outputs
        .binary_search_by_key(&process, |row| row.process_id)
        .ok()
        .map(|index| &state.process_outputs[index])
}

fn labor_coefficient(
    state: &MaterialCircuitStateV1,
    process: ProcessIdV1,
) -> Option<&crate::LaborCoefficientV1> {
    state
        .labor_coefficients
        .binary_search_by_key(&process, |row| row.process_id)
        .ok()
        .map(|index| &state.labor_coefficients[index])
}

fn input_coefficients(
    state: &MaterialCircuitStateV1,
    process: ProcessIdV1,
) -> &[crate::InputOutputCoefficientV1] {
    let start = state
        .input_coefficients
        .partition_point(|row| row.process_id < process);
    let end = state
        .input_coefficients
        .partition_point(|row| row.process_id <= process);
    &state.input_coefficients[start..end]
}

fn order_index(state: &MaterialCircuitStateV1, order: OrderIdV1) -> Option<usize> {
    state
        .orders
        .binary_search_by_key(&order, |row| row.order_id)
        .ok()
}

fn check_row_limits(state: &MaterialCircuitStateV1) -> Result<(), MaterialCircuitErrorV1> {
    let lengths = [
        state.process_outputs.len(),
        state.input_coefficients.len(),
        state.labor_coefficients.len(),
        state.supplier_candidates.len(),
        state.inventory.len(),
        state.orders.len(),
        state.backlog.len(),
        state.transit.len(),
        state.capacities.len(),
        state.labor.len(),
        state.production_commitments.len(),
    ];
    if lengths
        .into_iter()
        .any(|length| length > MAX_MATERIAL_CIRCUIT_ROWS_V1)
    {
        return Err(MaterialCircuitErrorV1::RowLimit);
    }
    Ok(())
}

fn has_duplicate<T, K: PartialEq>(rows: &[T], key: impl Fn(&T) -> K) -> bool {
    rows.windows(2)
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1)
        .any(|pair| key(&pair[0]) == key(&pair[1]))
}

fn canonicalize_rows(state: &mut MaterialCircuitStateV1) {
    state.process_outputs.sort();
    state.input_coefficients.sort();
    state.labor_coefficients.sort();
    state.supplier_candidates.sort();
    state.inventory.sort();
    state.orders.sort_by_key(|row| row.order_id);
    state.backlog.sort_by_key(|row| row.order_id);
    state
        .transit
        .sort_by_key(|row| (row.arrival_period, row.order_id, row.dispatch_period));
    state
        .capacities
        .sort_by_key(|row| (row.period, row.site_id, row.process_id));
    state
        .labor
        .sort_by_key(|row| (row.period, row.site_id, row.unit_id));
    state
        .production_commitments
        .sort_by_key(|row| (row.period, row.site_id, row.process_id));
}

fn validate_unique_rows(state: &MaterialCircuitStateV1) -> Result<(), MaterialCircuitErrorV1> {
    let transit_keys: BTreeSet<_> = state
        .transit
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|row| (row.order_id, row.dispatch_period))
        .collect();
    let duplicate = has_duplicate(&state.process_outputs, |row| row.process_id)
        || has_duplicate(&state.input_coefficients, |row| {
            (row.process_id, row.good_id, row.unit_id)
        })
        || has_duplicate(&state.labor_coefficients, |row| row.process_id)
        || has_duplicate(&state.supplier_candidates, |row| {
            (
                row.buyer_site_id,
                row.supplier_site_id,
                row.good_id,
                row.unit_id,
            )
        })
        || has_duplicate(&state.inventory, |row| {
            (row.site_id, row.good_id, row.unit_id)
        })
        || has_duplicate(&state.orders, |row| row.order_id)
        || has_duplicate(&state.backlog, |row| row.order_id)
        || transit_keys.len() != state.transit.len()
        || has_duplicate(&state.capacities, |row| {
            (row.period, row.site_id, row.process_id)
        })
        || has_duplicate(&state.labor, |row| (row.period, row.site_id, row.unit_id))
        || has_duplicate(&state.production_commitments, |row| {
            (row.period, row.site_id, row.process_id)
        });
    if duplicate {
        return Err(MaterialCircuitErrorV1::DuplicateRow);
    }
    Ok(())
}

fn validate_processes(state: &MaterialCircuitStateV1) -> Result<(), MaterialCircuitErrorV1> {
    let process_ids: BTreeSet<_> = state
        .process_outputs
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|row| row.process_id)
        .collect();
    for row in state
        .process_outputs
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        if row.quantity_per_batch == 0 {
            return Err(MaterialCircuitErrorV1::ZeroQuantity);
        }
    }
    for row in state
        .input_coefficients
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        if row.quantity_per_batch == 0 || !process_ids.contains(&row.process_id) {
            return Err(MaterialCircuitErrorV1::ProcessInvariant);
        }
    }
    for row in state
        .labor_coefficients
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        if row.quantity_per_batch == 0 || !process_ids.contains(&row.process_id) {
            return Err(MaterialCircuitErrorV1::ProcessInvariant);
        }
    }
    if state.process_outputs.len() != state.labor_coefficients.len() {
        return Err(MaterialCircuitErrorV1::ProcessInvariant);
    }
    for row in state
        .capacities
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        let output = process_output(state, row.process_id);
        if output.is_none_or(|output| output.site_id != row.site_id) {
            return Err(MaterialCircuitErrorV1::ProcessInvariant);
        }
    }
    for row in state
        .production_commitments
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        let output = process_output(state, row.process_id);
        if output.is_none_or(|output| output.site_id != row.site_id) {
            return Err(MaterialCircuitErrorV1::ProcessInvariant);
        }
    }
    Ok(())
}

fn validate_orders(state: &MaterialCircuitStateV1) -> Result<(), MaterialCircuitErrorV1> {
    if state.orders.len() != state.backlog.len() {
        return Err(MaterialCircuitErrorV1::BacklogInvariant);
    }
    for (order, backlog) in state
        .orders
        .iter()
        .zip(&state.backlog)
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        if order.ordered == 0 {
            return Err(MaterialCircuitErrorV1::ZeroQuantity);
        }
        if order.realized > order.delivered
            || order.delivered > order.shipped
            || order.shipped > order.ordered
        {
            return Err(MaterialCircuitErrorV1::OrderInvariant);
        }
        if backlog.order_id != order.order_id || backlog.quantity != order.ordered - order.shipped {
            return Err(MaterialCircuitErrorV1::BacklogInvariant);
        }
    }
    Ok(())
}

fn validate_transit(state: &MaterialCircuitStateV1) -> Result<(), MaterialCircuitErrorV1> {
    let mut in_transit = BTreeMap::<OrderIdV1, u128>::new();
    for lot in state.transit.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        let Some(index) = order_index(state, lot.order_id) else {
            return Err(MaterialCircuitErrorV1::TransitInvariant);
        };
        let order = &state.orders[index];
        if lot.quantity == 0
            || lot.dispatch_period >= lot.arrival_period
            || lot.dispatch_period >= state.period
            || lot.arrival_period < state.period
            || lot.source_site_id != order.supplier_site_id
            || lot.destination_site_id != order.buyer_site_id
            || lot.good_id != order.good_id
            || lot.unit_id != order.unit_id
        {
            return Err(MaterialCircuitErrorV1::TransitInvariant);
        }
        let total = in_transit.entry(lot.order_id).or_default();
        *total = total
            .checked_add(u128::from(lot.quantity))
            .ok_or(MaterialCircuitErrorV1::Arithmetic)?;
    }
    for order in state.orders.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        let expected = u128::from(order.shipped - order.delivered);
        if in_transit.get(&order.order_id).copied().unwrap_or(0) != expected {
            return Err(MaterialCircuitErrorV1::TransitInvariant);
        }
    }
    Ok(())
}

fn validate_periods(state: &MaterialCircuitStateV1) -> Result<(), MaterialCircuitErrorV1> {
    if state.period == 0
        || state
            .supplier_candidates
            .iter()
            .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
            .any(|row| row.transit_delay_periods == 0)
        || state
            .production_commitments
            .iter()
            .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
            .any(|row| row.period != state.period || row.planned_batches == 0)
        || state
            .capacities
            .iter()
            .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
            .any(|row| row.period < state.period)
        || state
            .labor
            .iter()
            .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
            .any(|row| row.period < state.period)
    {
        return Err(MaterialCircuitErrorV1::PeriodInvariant);
    }
    Ok(())
}

pub(crate) fn canonical_state_v1(
    state: &MaterialCircuitStateV1,
) -> Result<MaterialCircuitStateV1, MaterialCircuitErrorV1> {
    check_row_limits(state)?;
    let mut canonical = state.clone();
    canonicalize_rows(&mut canonical);
    validate_unique_rows(&canonical)?;
    validate_processes(&canonical)?;
    validate_orders(&canonical)?;
    validate_transit(&canonical)?;
    validate_periods(&canonical)?;
    Ok(canonical)
}

fn take_inventory(state: &mut MaterialCircuitStateV1) -> InventoryLedger {
    std::mem::take(&mut state.inventory)
        .into_iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|row| ((row.site_id, row.good_id, row.unit_id), row.quantity))
        .collect()
}

fn publish_inventory(state: &mut MaterialCircuitStateV1, inventory: InventoryLedger) {
    state.inventory = inventory
        .into_iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|((site_id, good_id, unit_id), quantity)| InventoryRowV1 {
            site_id,
            good_id,
            unit_id,
            quantity,
        })
        .collect();
}

fn credit_inventory(
    inventory: &mut InventoryLedger,
    key: InventoryKey,
    quantity: u64,
) -> Result<(), MaterialCircuitErrorV1> {
    if quantity == 0 {
        return Ok(());
    }
    if let Some(current) = inventory.get_mut(&key) {
        *current = current
            .checked_add(quantity)
            .ok_or(MaterialCircuitErrorV1::Arithmetic)?;
        return Ok(());
    }
    if inventory.len() == MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        return Err(MaterialCircuitErrorV1::RowLimit);
    }
    inventory.insert(key, quantity);
    Ok(())
}

fn debit_inventory(
    inventory: &mut InventoryLedger,
    key: InventoryKey,
    quantity: u64,
) -> Result<(), MaterialCircuitErrorV1> {
    if quantity == 0 {
        return Ok(());
    }
    let current = inventory
        .get_mut(&key)
        .ok_or(MaterialCircuitErrorV1::ProcessInvariant)?;
    *current = current
        .checked_sub(quantity)
        .ok_or(MaterialCircuitErrorV1::Arithmetic)?;
    Ok(())
}

fn process_arrivals(
    state: &mut MaterialCircuitStateV1,
    inventory: &mut InventoryLedger,
    arrivals: &mut Vec<ArrivalReceiptV1>,
    deliveries: &mut Vec<DeliveryReceiptV1>,
    realizations: &mut Vec<RealizationReceiptV1>,
) -> Result<(), MaterialCircuitErrorV1> {
    let opening = std::mem::take(&mut state.transit);
    let mut remaining = Vec::with_capacity(opening.len());
    for lot in opening.into_iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        if lot.arrival_period != state.period {
            remaining.push(lot);
            continue;
        }
        credit_inventory(
            inventory,
            (lot.destination_site_id, lot.good_id, lot.unit_id),
            lot.quantity,
        )?;
        let order_index =
            order_index(state, lot.order_id).ok_or(MaterialCircuitErrorV1::TransitInvariant)?;
        let order = &mut state.orders[order_index];
        order.delivered = order
            .delivered
            .checked_add(lot.quantity)
            .ok_or(MaterialCircuitErrorV1::Arithmetic)?;
        order.realized = order
            .realized
            .checked_add(lot.quantity)
            .ok_or(MaterialCircuitErrorV1::Arithmetic)?;
        if order.delivered > order.shipped || order.realized > order.delivered {
            return Err(MaterialCircuitErrorV1::OrderInvariant);
        }
        arrivals.push(ArrivalReceiptV1 {
            order_id: lot.order_id,
            quantity: lot.quantity,
        });
        deliveries.push(DeliveryReceiptV1 {
            order_id: lot.order_id,
            quantity: lot.quantity,
        });
        realizations.push(RealizationReceiptV1 {
            order_id: lot.order_id,
            quantity: lot.quantity,
        });
    }
    state.transit = remaining;
    Ok(())
}

fn process_capacity(
    state: &MaterialCircuitStateV1,
    process: ProcessIdV1,
    site: SiteIdV1,
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
    state: &MaterialCircuitStateV1,
    site: SiteIdV1,
    unit: UnitIdV1,
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
    state: &MaterialCircuitStateV1,
    commitments: &[crate::ProductionCommitmentV1],
    period: u64,
) -> Result<Vec<u64>, MaterialCircuitErrorV1> {
    let mut allocations = Vec::with_capacity(commitments.len());
    for commitment in commitments.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        let output = process_output(state, commitment.process_id)
            .ok_or(MaterialCircuitErrorV1::ProcessInvariant)?;
        if output.site_id != commitment.site_id || commitment.period != period {
            return Err(MaterialCircuitErrorV1::ProcessInvariant);
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
) -> Result<(), MaterialCircuitErrorV1> {
    let requested = u128::from(quantity_per_batch)
        .checked_mul(u128::from(batches))
        .ok_or(MaterialCircuitErrorV1::Arithmetic)?;
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
    state: &MaterialCircuitStateV1,
    commitments: &[crate::ProductionCommitmentV1],
    allocations: &[u64],
    resources: ProductionResources,
) -> Result<BTreeMap<ProductionResourceKey, Vec<ProductionResourceRequest>>, MaterialCircuitErrorV1>
{
    let mut groups = BTreeMap::new();
    for (index, commitment) in commitments
        .iter()
        .enumerate()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        let labor = labor_coefficient(state, commitment.process_id)
            .ok_or(MaterialCircuitErrorV1::ProcessInvariant)?;
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
            .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
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
    state: &MaterialCircuitStateV1,
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
) -> Result<u64, MaterialCircuitErrorV1> {
    if total == 0 || requested > total {
        return Err(MaterialCircuitErrorV1::Arithmetic);
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
    state: &MaterialCircuitStateV1,
    inventory: &InventoryLedger,
    period: u64,
    groups: &BTreeMap<ProductionResourceKey, Vec<ProductionResourceRequest>>,
    allocations: &mut [u64],
) -> Result<(), MaterialCircuitErrorV1> {
    if groups.len() > MAX_PRODUCTION_RESOURCE_GROUPS_V1 {
        return Err(MaterialCircuitErrorV1::RowLimit);
    }
    for (key, requests) in groups.iter().take(MAX_PRODUCTION_RESOURCE_GROUPS_V1) {
        let total = requests
            .iter()
            .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
            .try_fold(0_u128, |sum, request| {
                sum.checked_add(request.requested)
                    .ok_or(MaterialCircuitErrorV1::Arithmetic)
            })?;
        let available = production_resource_available(state, inventory, *key, period);
        for request in requests.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
            let granted_units = if u128::from(available) >= total {
                request.requested
            } else {
                u128::from(proportional_floor(available, request.requested, total)?)
            };
            let granted_batches = granted_units / u128::from(request.quantity_per_batch);
            let granted_batches =
                u64::try_from(granted_batches).map_err(|_| MaterialCircuitErrorV1::Arithmetic)?;
            allocations[request.commitment_index] =
                allocations[request.commitment_index].min(granted_batches);
        }
    }
    Ok(())
}

fn allocate_production_batches(
    state: &MaterialCircuitStateV1,
    inventory: &InventoryLedger,
    commitments: &[crate::ProductionCommitmentV1],
    period: u64,
    resources: ProductionResources,
) -> Result<Vec<u64>, MaterialCircuitErrorV1> {
    let mut allocations = initial_production_allocations(state, commitments, period)?;
    let groups = production_resource_groups(state, commitments, &allocations, resources)?;
    apply_production_resource_limits(state, inventory, period, &groups, &mut allocations)?;
    Ok(allocations)
}

fn execute_production(
    state: &mut MaterialCircuitStateV1,
    inventory: &mut InventoryLedger,
    receipts: &mut Vec<ProductionReceiptV1>,
) -> Result<(), MaterialCircuitErrorV1> {
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

pub(crate) fn execute_shared_production_v1(
    state: &mut MaterialCircuitStateV1,
) -> Result<Vec<ProductionReceiptV1>, MaterialCircuitErrorV1> {
    let mut inventory = take_inventory(state);
    let mut receipts = Vec::new();
    execute_production(state, &mut inventory, &mut receipts)?;
    publish_inventory(state, inventory);
    Ok(receipts)
}

fn debit_production_allocations(
    state: &mut MaterialCircuitStateV1,
    inventory: &mut InventoryLedger,
    commitments: &[crate::ProductionCommitmentV1],
    allocations: &[u64],
) -> Result<(), MaterialCircuitErrorV1> {
    for (index, commitment) in commitments
        .iter()
        .enumerate()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
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
    state: &mut MaterialCircuitStateV1,
    inventory: &mut InventoryLedger,
    commitments: Vec<crate::ProductionCommitmentV1>,
    allocations: &[u64],
    receipts: &mut Vec<ProductionReceiptV1>,
) -> Result<(), MaterialCircuitErrorV1> {
    for (index, commitment) in commitments
        .into_iter()
        .enumerate()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        let output = process_output(state, commitment.process_id)
            .cloned()
            .ok_or(MaterialCircuitErrorV1::ProcessInvariant)?;
        let batches = allocations[index];
        let produced = output
            .quantity_per_batch
            .checked_mul(batches)
            .ok_or(MaterialCircuitErrorV1::Arithmetic)?;
        credit_inventory(
            inventory,
            (output.site_id, output.good_id, output.unit_id),
            produced,
        )?;
        receipts.push(ProductionReceiptV1 {
            process_id: commitment.process_id,
            site_id: commitment.site_id,
            planned_batches: commitment.planned_batches,
            produced_batches: batches,
        });
    }
    Ok(())
}

fn consume_production_inputs(
    state: &mut MaterialCircuitStateV1,
    inventory: &mut InventoryLedger,
    process: ProcessIdV1,
    site: SiteIdV1,
    batches: u64,
) -> Result<(), MaterialCircuitErrorV1> {
    if batches == 0 {
        return Ok(());
    }
    let inputs: Vec<_> = input_coefficients(state, process)
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|row| (row.good_id, row.unit_id, row.quantity_per_batch))
        .collect();
    for (good, unit, quantity_per_batch) in
        inputs.into_iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        let quantity = quantity_per_batch
            .checked_mul(batches)
            .ok_or(MaterialCircuitErrorV1::Arithmetic)?;
        debit_inventory(inventory, (site, good, unit), quantity)?;
    }
    let labor = labor_coefficient(state, process)
        .cloned()
        .ok_or(MaterialCircuitErrorV1::ProcessInvariant)?;
    let labor_used = labor
        .quantity_per_batch
        .checked_mul(batches)
        .ok_or(MaterialCircuitErrorV1::Arithmetic)?;
    let index = labor_capacity_index(state, site, labor.unit_id, state.period)
        .ok_or(MaterialCircuitErrorV1::ProcessInvariant)?;
    state.labor[index].available = state.labor[index]
        .available
        .checked_sub(labor_used)
        .ok_or(MaterialCircuitErrorV1::Arithmetic)?;
    Ok(())
}

fn supplier_delays(state: &MaterialCircuitStateV1) -> BTreeMap<SupplierKey, u16> {
    state
        .supplier_candidates
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|row| {
            (
                (
                    row.buyer_site_id,
                    row.supplier_site_id,
                    row.good_id,
                    row.unit_id,
                ),
                row.transit_delay_periods,
            )
        })
        .collect()
}

fn request_groups(
    state: &MaterialCircuitStateV1,
    delays: &BTreeMap<SupplierKey, u16>,
) -> BTreeMap<InventoryKey, Vec<usize>> {
    let mut groups = BTreeMap::<InventoryKey, Vec<usize>>::new();
    for (index, order) in state
        .orders
        .iter()
        .enumerate()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        let candidate = (
            order.buyer_site_id,
            order.supplier_site_id,
            order.good_id,
            order.unit_id,
        );
        if order.shipped < order.ordered && delays.contains_key(&candidate) {
            groups
                .entry((order.supplier_site_id, order.good_id, order.unit_id))
                .or_default()
                .push(index);
        }
    }
    groups
}

fn group_allocations(
    state: &MaterialCircuitStateV1,
    indices: &[usize],
    available: u64,
) -> Result<Vec<(usize, u64)>, MaterialCircuitErrorV1> {
    let mut total = 0_u128;
    for index in indices
        .iter()
        .copied()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        total = total
            .checked_add(u128::from(
                state.orders[index].ordered - state.orders[index].shipped,
            ))
            .ok_or(MaterialCircuitErrorV1::Arithmetic)?;
    }
    let mut allocations = Vec::with_capacity(indices.len());
    for index in indices
        .iter()
        .copied()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        let requested = state.orders[index].ordered - state.orders[index].shipped;
        let allocated = if u128::from(available) >= total {
            requested
        } else {
            let share = u128::from(available) * u128::from(requested) / total;
            u64::try_from(share).map_err(|_| MaterialCircuitErrorV1::Arithmetic)?
        };
        allocations.push((index, allocated));
    }
    Ok(allocations)
}

fn dispatch_orders(
    state: &mut MaterialCircuitStateV1,
    inventory: &mut InventoryLedger,
    receipts: &mut Vec<DispatchReceiptV1>,
) -> Result<(), MaterialCircuitErrorV1> {
    let delays = supplier_delays(state);
    let groups = request_groups(state, &delays);
    for (inventory_key, indices) in groups.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        let available = inventory.get(inventory_key).copied().unwrap_or(0);
        let allocations = group_allocations(state, indices, available)?;
        let allocated = allocations.iter().try_fold(0_u64, |total, (_, quantity)| {
            total
                .checked_add(*quantity)
                .ok_or(MaterialCircuitErrorV1::Arithmetic)
        })?;
        debit_inventory(inventory, *inventory_key, allocated)?;
        apply_dispatches(state, &delays, allocations, receipts)?;
    }
    Ok(())
}

fn apply_dispatches(
    state: &mut MaterialCircuitStateV1,
    delays: &BTreeMap<SupplierKey, u16>,
    allocations: Vec<(usize, u64)>,
    receipts: &mut Vec<DispatchReceiptV1>,
) -> Result<(), MaterialCircuitErrorV1> {
    for (index, quantity) in allocations
        .into_iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        if quantity == 0 {
            continue;
        }
        let order = state.orders[index].clone();
        let delay = delays[&(
            order.buyer_site_id,
            order.supplier_site_id,
            order.good_id,
            order.unit_id,
        )];
        let arrival_period = state
            .period
            .checked_add(u64::from(delay))
            .ok_or(MaterialCircuitErrorV1::Arithmetic)?;
        state.orders[index].shipped = state.orders[index]
            .shipped
            .checked_add(quantity)
            .ok_or(MaterialCircuitErrorV1::Arithmetic)?;
        state.transit.push(TransitLotV1 {
            order_id: order.order_id,
            dispatch_period: state.period,
            arrival_period,
            source_site_id: order.supplier_site_id,
            destination_site_id: order.buyer_site_id,
            good_id: order.good_id,
            unit_id: order.unit_id,
            quantity,
        });
        receipts.push(DispatchReceiptV1 {
            order_id: order.order_id,
            quantity,
            arrival_period,
        });
    }
    Ok(())
}

fn rebuild_backlog(state: &mut MaterialCircuitStateV1) {
    state.backlog = state
        .orders
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|order| BacklogRowV1 {
            order_id: order.order_id,
            quantity: order.ordered - order.shipped,
        })
        .collect();
}

fn next_period_candidates(
    state: &MaterialCircuitStateV1,
    next_period: u64,
) -> Vec<crate::ProductionCommitmentV1> {
    state
        .process_outputs
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|output| crate::ProductionCommitmentV1 {
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
    state: &mut MaterialCircuitStateV1,
    inventory: &InventoryLedger,
    next_period: u64,
) -> Result<(), MaterialCircuitErrorV1> {
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
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        let batches = allocations[index];
        if batches > 0 {
            state
                .production_commitments
                .push(crate::ProductionCommitmentV1 {
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
pub(crate) fn derive_shared_labor_requests_v1(
    state: &MaterialCircuitStateV1,
    next_period: u64,
) -> Result<Vec<ProcessLaborRequest>, MaterialCircuitErrorV1> {
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
                .ok_or(MaterialCircuitErrorV1::ProcessInvariant)?;
            Ok(ProcessLaborRequest {
                process_id: candidate.process_id,
                site_id: candidate.site_id,
                unit_id: coefficient.unit_id,
                hours: batches
                    .checked_mul(coefficient.quantity_per_batch)
                    .ok_or(MaterialCircuitErrorV1::Arithmetic)?,
            })
        })
        .collect()
}

pub(crate) fn derive_shared_production_v1(
    state: &mut MaterialCircuitStateV1,
    next_period: u64,
) -> Result<(), MaterialCircuitErrorV1> {
    let inventory = take_inventory(state);
    derive_next_period_production(state, &inventory, next_period)?;
    prune_consumed_capacity(state, next_period);
    publish_inventory(state, inventory);
    Ok(())
}

fn prune_consumed_capacity(state: &mut MaterialCircuitStateV1, next_period: u64) {
    state.capacities = std::mem::take(&mut state.capacities)
        .into_iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .filter(|row| row.period >= next_period)
        .collect();
    state.labor = std::mem::take(&mut state.labor)
        .into_iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .filter(|row| row.period >= next_period)
        .collect();
}

/// Close one period atomically and return its canonical successor state.
///
/// # Errors
/// Returns the first exact schema, invariant, bound, or arithmetic refusal.
pub fn advance_material_circuit_v1(
    opening: &MaterialCircuitStateV1,
) -> Result<MaterialCircuitTransitionV1, MaterialCircuitErrorV1> {
    let mut state = canonical_state_v1(opening)?;
    let mut inventory = take_inventory(&mut state);
    let mut arrivals = Vec::new();
    let mut deliveries = Vec::new();
    let mut realizations = Vec::new();
    let mut production = Vec::new();
    let mut dispatches = Vec::new();
    process_arrivals(
        &mut state,
        &mut inventory,
        &mut arrivals,
        &mut deliveries,
        &mut realizations,
    )?;
    execute_production(&mut state, &mut inventory, &mut production)?;
    dispatch_orders(&mut state, &mut inventory, &mut dispatches)?;
    rebuild_backlog(&mut state);
    let next_period = state
        .period
        .checked_add(1)
        .ok_or(MaterialCircuitErrorV1::Arithmetic)?;
    derive_next_period_production(&mut state, &inventory, next_period)?;
    prune_consumed_capacity(&mut state, next_period);
    publish_inventory(&mut state, inventory);
    state.period = next_period;
    state = canonical_state_v1(&state)?;
    Ok(MaterialCircuitTransitionV1 {
        state,
        production,
        dispatches,
        arrivals,
        deliveries,
        realizations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn numbered_identity(index: usize) -> [u8; 32] {
        let mut bytes = [0_u8; 32];
        let number = u64::try_from(index).expect("designed group bound fits u64");
        bytes[24..].copy_from_slice(&number.to_be_bytes());
        bytes
    }

    fn empty_state() -> MaterialCircuitStateV1 {
        MaterialCircuitStateV1 {
            period: 1,
            process_outputs: Vec::new(),
            input_coefficients: Vec::new(),
            labor_coefficients: Vec::new(),
            supplier_candidates: Vec::new(),
            inventory: Vec::new(),
            orders: Vec::new(),
            backlog: Vec::new(),
            transit: Vec::new(),
            capacities: Vec::new(),
            labor: Vec::new(),
            production_commitments: Vec::new(),
        }
    }

    #[test]
    fn resource_group_bound_covers_both_families_and_refuses_plus_one() {
        let site = SiteIdV1::from_bytes([1; 32]);
        let unit = UnitIdV1::from_bytes([2; 32]);
        let mut groups = BTreeMap::new();
        for index in 0..MAX_PRODUCTION_RESOURCE_GROUPS_V1 {
            groups.insert(
                ProductionResourceKey::Input((
                    site,
                    GoodIdV1::from_bytes(numbered_identity(index)),
                    unit,
                )),
                Vec::new(),
            );
        }
        let last_key = ProductionResourceKey::Input((
            site,
            GoodIdV1::from_bytes(numbered_identity(MAX_PRODUCTION_RESOURCE_GROUPS_V1 - 1)),
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
                GoodIdV1::from_bytes(numbered_identity(MAX_PRODUCTION_RESOURCE_GROUPS_V1)),
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
            Err(MaterialCircuitErrorV1::RowLimit)
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
