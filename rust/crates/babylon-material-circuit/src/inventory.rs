//! One checked inventory ledger for production and freight.

use crate::{
    GoodIdV1, InventoryRowV1, MaterialCircuitErrorV3, MaterialCircuitStateV3, SiteIdV1, UnitIdV1,
    MAX_MATERIAL_CIRCUIT_ROWS_V1,
};
use std::collections::BTreeMap;
pub(crate) type InventoryKey = (SiteIdV1, GoodIdV1, UnitIdV1);
pub(crate) type InventoryLedger = BTreeMap<InventoryKey, u64>;

pub(crate) fn take_inventory(state: &mut MaterialCircuitStateV3) -> InventoryLedger {
    std::mem::take(&mut state.inventory)
        .into_iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .map(|row| ((row.site_id, row.good_id, row.unit_id), row.quantity))
        .collect()
}

pub(crate) fn publish_inventory(state: &mut MaterialCircuitStateV3, inventory: InventoryLedger) {
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

pub(crate) fn credit_inventory(
    inventory: &mut InventoryLedger,
    key: InventoryKey,
    quantity: u64,
) -> Result<(), MaterialCircuitErrorV3> {
    if quantity == 0 {
        return Ok(());
    }
    if let Some(current) = inventory.get_mut(&key) {
        *current = current
            .checked_add(quantity)
            .ok_or(MaterialCircuitErrorV3::Arithmetic)?;
        return Ok(());
    }
    if inventory.len() == MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        return Err(MaterialCircuitErrorV3::RowLimit);
    }
    inventory.insert(key, quantity);
    Ok(())
}

pub(crate) fn debit_inventory(
    inventory: &mut InventoryLedger,
    key: InventoryKey,
    quantity: u64,
    missing: MaterialCircuitErrorV3,
) -> Result<(), MaterialCircuitErrorV3> {
    if quantity == 0 {
        return Ok(());
    }
    let current = inventory.get_mut(&key).ok_or(missing)?;
    *current = current
        .checked_sub(quantity)
        .ok_or(MaterialCircuitErrorV3::Arithmetic)?;
    Ok(())
}
