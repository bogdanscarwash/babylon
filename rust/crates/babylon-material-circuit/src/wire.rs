//! Canonical V1 state bytes for restart, replay, and language-neutral vectors.

use babylon_kernel::sha256_of;

use crate::transition::canonical_state_v1;
use crate::wire_common::{Cursor, CursorError};
use crate::{
    BacklogRowV1, CapacityRowV1, GoodIdV1, InputOutputCoefficientV1, InventoryRowV1,
    LaborCapacityRowV1, LaborCoefficientV1, MaterialCircuitErrorV1, MaterialCircuitStateV1,
    OrderAccessModeV1, OrderIdV1, OrderRowV1, ProcessIdV1, ProcessOutputV1, ProductionCommitmentV1,
    SiteIdV1, SupplierCandidateV1, TransitLotV1, UnitIdV1, MAX_MATERIAL_CIRCUIT_ROWS_V1,
};

/// Canonical domain for one complete material-circuit opening state.
pub const MATERIAL_CIRCUIT_STATE_V1_DOMAIN_BYTES: &[u8] = b"babylon.material-circuit-state.v1";
const SCHEMA_VERSION: u16 = 1;

impl From<CursorError> for MaterialCircuitErrorV1 {
    fn from(value: CursorError) -> Self {
        match value {
            CursorError::Truncated => Self::WireTruncated,
            CursorError::Trailing => Self::WireTrailing,
        }
    }
}

fn row_count(cursor: &mut Cursor<'_>) -> Result<usize, MaterialCircuitErrorV1> {
    let count = usize::try_from(cursor.u32()?).map_err(|_| MaterialCircuitErrorV1::WireLimit)?;
    if count > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        return Err(MaterialCircuitErrorV1::WireLimit);
    }
    Ok(count)
}

fn append_count(output: &mut Vec<u8>, length: usize) -> Result<(), MaterialCircuitErrorV1> {
    let count = u32::try_from(length).map_err(|_| MaterialCircuitErrorV1::WireLimit)?;
    output.extend_from_slice(&count.to_be_bytes());
    Ok(())
}

fn append_process_outputs(
    output: &mut Vec<u8>,
    state: &MaterialCircuitStateV1,
) -> Result<(), MaterialCircuitErrorV1> {
    append_count(output, state.process_outputs.len())?;
    for row in state
        .process_outputs
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        output.extend_from_slice(&row.process_id.as_bytes());
        output.extend_from_slice(&row.site_id.as_bytes());
        output.extend_from_slice(&row.good_id.as_bytes());
        output.extend_from_slice(&row.unit_id.as_bytes());
        output.extend_from_slice(&row.quantity_per_batch.to_be_bytes());
    }
    Ok(())
}

fn append_input_coefficients(
    output: &mut Vec<u8>,
    state: &MaterialCircuitStateV1,
) -> Result<(), MaterialCircuitErrorV1> {
    append_count(output, state.input_coefficients.len())?;
    for row in state
        .input_coefficients
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        output.extend_from_slice(&row.process_id.as_bytes());
        output.extend_from_slice(&row.good_id.as_bytes());
        output.extend_from_slice(&row.unit_id.as_bytes());
        output.extend_from_slice(&row.quantity_per_batch.to_be_bytes());
    }
    Ok(())
}

fn append_labor_coefficients(
    output: &mut Vec<u8>,
    state: &MaterialCircuitStateV1,
) -> Result<(), MaterialCircuitErrorV1> {
    append_count(output, state.labor_coefficients.len())?;
    for row in state
        .labor_coefficients
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        output.extend_from_slice(&row.process_id.as_bytes());
        output.extend_from_slice(&row.unit_id.as_bytes());
        output.extend_from_slice(&row.quantity_per_batch.to_be_bytes());
    }
    Ok(())
}

fn append_supplier_candidates(
    output: &mut Vec<u8>,
    state: &MaterialCircuitStateV1,
) -> Result<(), MaterialCircuitErrorV1> {
    append_count(output, state.supplier_candidates.len())?;
    for row in state
        .supplier_candidates
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        output.extend_from_slice(&row.buyer_site_id.as_bytes());
        output.extend_from_slice(&row.supplier_site_id.as_bytes());
        output.extend_from_slice(&row.good_id.as_bytes());
        output.extend_from_slice(&row.unit_id.as_bytes());
        output.extend_from_slice(&row.transit_delay_periods.to_be_bytes());
    }
    Ok(())
}

fn append_inventory(
    output: &mut Vec<u8>,
    state: &MaterialCircuitStateV1,
) -> Result<(), MaterialCircuitErrorV1> {
    append_count(output, state.inventory.len())?;
    for row in state
        .inventory
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        output.extend_from_slice(&row.site_id.as_bytes());
        output.extend_from_slice(&row.good_id.as_bytes());
        output.extend_from_slice(&row.unit_id.as_bytes());
        output.extend_from_slice(&row.quantity.to_be_bytes());
    }
    Ok(())
}

fn append_orders(
    output: &mut Vec<u8>,
    state: &MaterialCircuitStateV1,
) -> Result<(), MaterialCircuitErrorV1> {
    append_count(output, state.orders.len())?;
    for row in state.orders.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        output.extend_from_slice(&row.order_id.as_bytes());
        output.push(row.access_mode as u8);
        output.extend_from_slice(&row.buyer_site_id.as_bytes());
        output.extend_from_slice(&row.supplier_site_id.as_bytes());
        output.extend_from_slice(&row.good_id.as_bytes());
        output.extend_from_slice(&row.unit_id.as_bytes());
        output.extend_from_slice(&row.ordered.to_be_bytes());
        output.extend_from_slice(&row.shipped.to_be_bytes());
        output.extend_from_slice(&row.delivered.to_be_bytes());
        output.extend_from_slice(&row.realized.to_be_bytes());
    }
    Ok(())
}

fn append_backlog(
    output: &mut Vec<u8>,
    state: &MaterialCircuitStateV1,
) -> Result<(), MaterialCircuitErrorV1> {
    append_count(output, state.backlog.len())?;
    for row in state.backlog.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        output.extend_from_slice(&row.order_id.as_bytes());
        output.extend_from_slice(&row.quantity.to_be_bytes());
    }
    Ok(())
}

fn append_transit(
    output: &mut Vec<u8>,
    state: &MaterialCircuitStateV1,
) -> Result<(), MaterialCircuitErrorV1> {
    append_count(output, state.transit.len())?;
    for row in state.transit.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        output.extend_from_slice(&row.order_id.as_bytes());
        output.extend_from_slice(&row.dispatch_period.to_be_bytes());
        output.extend_from_slice(&row.arrival_period.to_be_bytes());
        output.extend_from_slice(&row.source_site_id.as_bytes());
        output.extend_from_slice(&row.destination_site_id.as_bytes());
        output.extend_from_slice(&row.good_id.as_bytes());
        output.extend_from_slice(&row.unit_id.as_bytes());
        output.extend_from_slice(&row.quantity.to_be_bytes());
    }
    Ok(())
}

fn append_capacities(
    output: &mut Vec<u8>,
    state: &MaterialCircuitStateV1,
) -> Result<(), MaterialCircuitErrorV1> {
    append_count(output, state.capacities.len())?;
    for row in state
        .capacities
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        output.extend_from_slice(&row.process_id.as_bytes());
        output.extend_from_slice(&row.site_id.as_bytes());
        output.extend_from_slice(&row.period.to_be_bytes());
        output.extend_from_slice(&row.available_batches.to_be_bytes());
    }
    Ok(())
}

fn append_labor(
    output: &mut Vec<u8>,
    state: &MaterialCircuitStateV1,
) -> Result<(), MaterialCircuitErrorV1> {
    append_count(output, state.labor.len())?;
    for row in state.labor.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        output.extend_from_slice(&row.site_id.as_bytes());
        output.extend_from_slice(&row.unit_id.as_bytes());
        output.extend_from_slice(&row.period.to_be_bytes());
        output.extend_from_slice(&row.available.to_be_bytes());
    }
    Ok(())
}

fn append_production_commitments(
    output: &mut Vec<u8>,
    state: &MaterialCircuitStateV1,
) -> Result<(), MaterialCircuitErrorV1> {
    append_count(output, state.production_commitments.len())?;
    for row in state
        .production_commitments
        .iter()
        .take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
    {
        output.extend_from_slice(&row.process_id.as_bytes());
        output.extend_from_slice(&row.site_id.as_bytes());
        output.extend_from_slice(&row.period.to_be_bytes());
        output.extend_from_slice(&row.planned_batches.to_be_bytes());
    }
    Ok(())
}

fn decode_process_outputs(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<ProcessOutputV1>, MaterialCircuitErrorV1> {
    let count = row_count(cursor)?;
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        if index == count {
            break;
        }
        rows.push(ProcessOutputV1 {
            process_id: ProcessIdV1::from_bytes(cursor.array()?),
            site_id: SiteIdV1::from_bytes(cursor.array()?),
            good_id: GoodIdV1::from_bytes(cursor.array()?),
            unit_id: UnitIdV1::from_bytes(cursor.array()?),
            quantity_per_batch: cursor.u64()?,
        });
    }
    Ok(rows)
}

fn decode_input_coefficients(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<InputOutputCoefficientV1>, MaterialCircuitErrorV1> {
    let count = row_count(cursor)?;
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        if index == count {
            break;
        }
        rows.push(InputOutputCoefficientV1 {
            process_id: ProcessIdV1::from_bytes(cursor.array()?),
            good_id: GoodIdV1::from_bytes(cursor.array()?),
            unit_id: UnitIdV1::from_bytes(cursor.array()?),
            quantity_per_batch: cursor.u64()?,
        });
    }
    Ok(rows)
}

fn decode_labor_coefficients(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<LaborCoefficientV1>, MaterialCircuitErrorV1> {
    let count = row_count(cursor)?;
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        if index == count {
            break;
        }
        rows.push(LaborCoefficientV1 {
            process_id: ProcessIdV1::from_bytes(cursor.array()?),
            unit_id: UnitIdV1::from_bytes(cursor.array()?),
            quantity_per_batch: cursor.u64()?,
        });
    }
    Ok(rows)
}

fn decode_supplier_candidates(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<SupplierCandidateV1>, MaterialCircuitErrorV1> {
    let count = row_count(cursor)?;
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        if index == count {
            break;
        }
        rows.push(SupplierCandidateV1 {
            buyer_site_id: SiteIdV1::from_bytes(cursor.array()?),
            supplier_site_id: SiteIdV1::from_bytes(cursor.array()?),
            good_id: GoodIdV1::from_bytes(cursor.array()?),
            unit_id: UnitIdV1::from_bytes(cursor.array()?),
            transit_delay_periods: cursor.u16()?,
        });
    }
    Ok(rows)
}

fn decode_inventory(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<InventoryRowV1>, MaterialCircuitErrorV1> {
    let count = row_count(cursor)?;
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        if index == count {
            break;
        }
        rows.push(InventoryRowV1 {
            site_id: SiteIdV1::from_bytes(cursor.array()?),
            good_id: GoodIdV1::from_bytes(cursor.array()?),
            unit_id: UnitIdV1::from_bytes(cursor.array()?),
            quantity: cursor.u64()?,
        });
    }
    Ok(rows)
}

fn decode_orders(cursor: &mut Cursor<'_>) -> Result<Vec<OrderRowV1>, MaterialCircuitErrorV1> {
    let count = row_count(cursor)?;
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        if index == count {
            break;
        }
        let order_id = OrderIdV1::from_bytes(cursor.array()?);
        let access_mode = match cursor.u8()? {
            1 => OrderAccessModeV1::CommoditySale,
            _ => return Err(MaterialCircuitErrorV1::WireEnum),
        };
        rows.push(OrderRowV1 {
            order_id,
            access_mode,
            buyer_site_id: SiteIdV1::from_bytes(cursor.array()?),
            supplier_site_id: SiteIdV1::from_bytes(cursor.array()?),
            good_id: GoodIdV1::from_bytes(cursor.array()?),
            unit_id: UnitIdV1::from_bytes(cursor.array()?),
            ordered: cursor.u64()?,
            shipped: cursor.u64()?,
            delivered: cursor.u64()?,
            realized: cursor.u64()?,
        });
    }
    Ok(rows)
}

fn decode_backlog(cursor: &mut Cursor<'_>) -> Result<Vec<BacklogRowV1>, MaterialCircuitErrorV1> {
    let count = row_count(cursor)?;
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        if index == count {
            break;
        }
        rows.push(BacklogRowV1 {
            order_id: OrderIdV1::from_bytes(cursor.array()?),
            quantity: cursor.u64()?,
        });
    }
    Ok(rows)
}

fn decode_transit(cursor: &mut Cursor<'_>) -> Result<Vec<TransitLotV1>, MaterialCircuitErrorV1> {
    let count = row_count(cursor)?;
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        if index == count {
            break;
        }
        rows.push(TransitLotV1 {
            order_id: OrderIdV1::from_bytes(cursor.array()?),
            dispatch_period: cursor.u64()?,
            arrival_period: cursor.u64()?,
            source_site_id: SiteIdV1::from_bytes(cursor.array()?),
            destination_site_id: SiteIdV1::from_bytes(cursor.array()?),
            good_id: GoodIdV1::from_bytes(cursor.array()?),
            unit_id: UnitIdV1::from_bytes(cursor.array()?),
            quantity: cursor.u64()?,
        });
    }
    Ok(rows)
}

fn decode_capacities(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<CapacityRowV1>, MaterialCircuitErrorV1> {
    let count = row_count(cursor)?;
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        if index == count {
            break;
        }
        rows.push(CapacityRowV1 {
            process_id: ProcessIdV1::from_bytes(cursor.array()?),
            site_id: SiteIdV1::from_bytes(cursor.array()?),
            period: cursor.u64()?,
            available_batches: cursor.u64()?,
        });
    }
    Ok(rows)
}

fn decode_labor(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<LaborCapacityRowV1>, MaterialCircuitErrorV1> {
    let count = row_count(cursor)?;
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        if index == count {
            break;
        }
        rows.push(LaborCapacityRowV1 {
            site_id: SiteIdV1::from_bytes(cursor.array()?),
            unit_id: UnitIdV1::from_bytes(cursor.array()?),
            period: cursor.u64()?,
            available: cursor.u64()?,
        });
    }
    Ok(rows)
}

fn decode_production_commitments(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<ProductionCommitmentV1>, MaterialCircuitErrorV1> {
    let count = row_count(cursor)?;
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        if index == count {
            break;
        }
        rows.push(ProductionCommitmentV1 {
            process_id: ProcessIdV1::from_bytes(cursor.array()?),
            site_id: SiteIdV1::from_bytes(cursor.array()?),
            period: cursor.u64()?,
            planned_batches: cursor.u64()?,
        });
    }
    Ok(rows)
}

/// Encode one complete validated state in canonical big-endian order.
///
/// # Errors
/// Returns the first exact state, row-bound, or wire-bound refusal.
pub fn encode_material_circuit_state_v1(
    state: &MaterialCircuitStateV1,
) -> Result<Vec<u8>, MaterialCircuitErrorV1> {
    let canonical = canonical_state_v1(state)?;
    let mut output = Vec::new();
    output.extend_from_slice(MATERIAL_CIRCUIT_STATE_V1_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&SCHEMA_VERSION.to_be_bytes());
    output.extend_from_slice(&canonical.period.to_be_bytes());
    append_process_outputs(&mut output, &canonical)?;
    append_input_coefficients(&mut output, &canonical)?;
    append_labor_coefficients(&mut output, &canonical)?;
    append_supplier_candidates(&mut output, &canonical)?;
    append_inventory(&mut output, &canonical)?;
    append_orders(&mut output, &canonical)?;
    append_backlog(&mut output, &canonical)?;
    append_transit(&mut output, &canonical)?;
    append_capacities(&mut output, &canonical)?;
    append_labor(&mut output, &canonical)?;
    append_production_commitments(&mut output, &canonical)?;
    Ok(output)
}

/// Decode one complete canonical V1 state.
///
/// # Errors
/// Returns the first domain, version, enum, wire, canonical-order, or state refusal.
pub fn decode_material_circuit_state_v1(
    payload: &[u8],
) -> Result<MaterialCircuitStateV1, MaterialCircuitErrorV1> {
    let mut cursor = Cursor::new(payload);
    if cursor.take(MATERIAL_CIRCUIT_STATE_V1_DOMAIN_BYTES.len())?
        != MATERIAL_CIRCUIT_STATE_V1_DOMAIN_BYTES
        || cursor.u8()? != 0
    {
        return Err(MaterialCircuitErrorV1::WireDomain);
    }
    if cursor.u16()? != SCHEMA_VERSION {
        return Err(MaterialCircuitErrorV1::WireVersion);
    }
    let state = MaterialCircuitStateV1 {
        period: cursor.u64()?,
        process_outputs: decode_process_outputs(&mut cursor)?,
        input_coefficients: decode_input_coefficients(&mut cursor)?,
        labor_coefficients: decode_labor_coefficients(&mut cursor)?,
        supplier_candidates: decode_supplier_candidates(&mut cursor)?,
        inventory: decode_inventory(&mut cursor)?,
        orders: decode_orders(&mut cursor)?,
        backlog: decode_backlog(&mut cursor)?,
        transit: decode_transit(&mut cursor)?,
        capacities: decode_capacities(&mut cursor)?,
        labor: decode_labor(&mut cursor)?,
        production_commitments: decode_production_commitments(&mut cursor)?,
    };
    cursor.finish()?;
    let canonical = canonical_state_v1(&state)?;
    if canonical != state {
        return Err(MaterialCircuitErrorV1::WireNoncanonical);
    }
    Ok(state)
}

/// Hash one complete validated canonical state.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn material_circuit_state_v1_digest(
    state: &MaterialCircuitStateV1,
) -> Result<[u8; 32], MaterialCircuitErrorV1> {
    Ok(sha256_of(&encode_material_circuit_state_v1(state)?))
}
