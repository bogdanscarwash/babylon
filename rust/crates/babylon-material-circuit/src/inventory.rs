//! One checked inventory ledger for production and freight.

use crate::{
    GoodId, InventoryRow, MaterialCircuitError, MaterialCircuitState, SiteId, UnitId,
    MAX_MATERIAL_CIRCUIT_ROWS,
};
use std::collections::BTreeMap;
pub(crate) type InventoryKey = (SiteId, GoodId, UnitId);
pub(crate) type InventoryLedger = BTreeMap<InventoryKey, u64>;

pub(crate) fn take_inventory(state: &mut MaterialCircuitState) -> InventoryLedger {
    std::mem::take(&mut state.inventory)
        .into_iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
        .map(|row| ((row.site_id, row.good_id, row.unit_id), row.quantity))
        .collect()
}

pub(crate) fn publish_inventory(state: &mut MaterialCircuitState, inventory: InventoryLedger) {
    state.inventory = inventory
        .into_iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS + 1)
        .map(|((site_id, good_id, unit_id), quantity)| InventoryRow {
            site_id,
            good_id,
            unit_id,
            quantity,
        })
        .collect();
}

pub(crate) fn credit_inventory(
    inventory: &mut InventoryLedger,
    key: InventoryKey,
    quantity: u64,
) -> Result<(), MaterialCircuitError> {
    if quantity == 0 {
        return Ok(());
    }
    if let Some(current) = inventory.get_mut(&key) {
        *current = current
            .checked_add(quantity)
            .ok_or(MaterialCircuitError::Arithmetic)?;
        return Ok(());
    }
    if inventory.len() == MAX_MATERIAL_CIRCUIT_ROWS {
        return Err(MaterialCircuitError::RowLimit);
    }
    inventory.insert(key, quantity);
    Ok(())
}

pub(crate) fn debit_inventory(
    inventory: &mut InventoryLedger,
    key: InventoryKey,
    quantity: u64,
    missing: MaterialCircuitError,
) -> Result<(), MaterialCircuitError> {
    if quantity == 0 {
        return Ok(());
    }
    let current = inventory.get_mut(&key).ok_or(missing)?;
    *current = current
        .checked_sub(quantity)
        .ok_or(MaterialCircuitError::Arithmetic)?;
    Ok(())
}
