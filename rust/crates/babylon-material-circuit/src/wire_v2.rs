//! Canonical V2 routed-material state bytes for restart and replay.

use babylon_kernel::sha256_of;

use crate::transition_v2::canonical_state_v2;
use crate::wire_common::{Cursor, CursorError};
use crate::{
    BacklogRowV1, CapacityRowV1, CorridorCapacityV2, CorridorIdV2, FreightLotIdV2, GoodIdV1,
    InputOutputCoefficientV1, InventoryRowV1, LaborCapacityRowV1, LaborCoefficientV1,
    LogisticsNodeIdV2, MaterialCircuitErrorV2, MaterialCircuitStateV2, OrderAccessModeV1,
    OrderIdV1, OrderRowV2, ProcessIdV1, ProcessOutputV1, ProductionCommitmentV1, RouteIdV2,
    RouteLegV2, RoutedFreightLotV2, SiteIdV1, SiteLogisticsNodeV2, SupplierRouteV2, UnitIdV1,
    MAX_MATERIAL_CIRCUIT_ROWS_V1,
};

/// Canonical domain for one complete routed material-circuit opening state.
pub const MATERIAL_CIRCUIT_STATE_V2_DOMAIN_BYTES: &[u8] = b"babylon.material-circuit-state.v2";
/// SHA-256 of the complete language-neutral Material Circuit V2 contract source.
pub const MATERIAL_CIRCUIT_V2_SOURCE_SHA256: [u8; 32] = [
    0x42, 0x49, 0xc6, 0xc2, 0xd2, 0x38, 0xb7, 0xc5, 0xdb, 0x1c, 0x55, 0x2e, 0xc1, 0xa7, 0x20, 0xa5,
    0x83, 0x0c, 0x27, 0xe2, 0x42, 0x9b, 0x3c, 0x4a, 0x8a, 0x3d, 0x29, 0x27, 0x7d, 0xd3, 0x1c, 0x72,
];
const SCHEMA_VERSION: u16 = 2;

impl From<CursorError> for MaterialCircuitErrorV2 {
    fn from(value: CursorError) -> Self {
        match value {
            CursorError::Truncated => Self::WireTruncated,
            CursorError::Trailing => Self::WireTrailing,
        }
    }
}

fn row_count(cursor: &mut Cursor<'_>) -> Result<usize, MaterialCircuitErrorV2> {
    let count = usize::try_from(cursor.u32()?).map_err(|_| MaterialCircuitErrorV2::WireLimit)?;
    if count > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        return Err(MaterialCircuitErrorV2::WireLimit);
    }
    Ok(count)
}

fn append_rows<T>(
    output: &mut Vec<u8>,
    rows: &[T],
    mut append: impl FnMut(&mut Vec<u8>, &T),
) -> Result<(), MaterialCircuitErrorV2> {
    let count = u32::try_from(rows.len()).map_err(|_| MaterialCircuitErrorV2::WireLimit)?;
    output.extend_from_slice(&count.to_be_bytes());
    for row in rows.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        append(output, row);
    }
    Ok(())
}

fn decode_rows<T>(
    cursor: &mut Cursor<'_>,
    mut decode: impl FnMut(&mut Cursor<'_>) -> Result<T, MaterialCircuitErrorV2>,
) -> Result<Vec<T>, MaterialCircuitErrorV2> {
    let count = row_count(cursor)?;
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        if index == count {
            break;
        }
        rows.push(decode(cursor)?);
    }
    Ok(rows)
}

fn append_site_nodes(
    output: &mut Vec<u8>,
    rows: &[SiteLogisticsNodeV2],
) -> Result<(), MaterialCircuitErrorV2> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.site_id.as_bytes());
        bytes.extend_from_slice(&row.node_id.as_bytes());
    })
}

fn append_process_outputs(
    output: &mut Vec<u8>,
    rows: &[ProcessOutputV1],
) -> Result<(), MaterialCircuitErrorV2> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.process_id.as_bytes());
        bytes.extend_from_slice(&row.site_id.as_bytes());
        bytes.extend_from_slice(&row.good_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.quantity_per_batch.to_be_bytes());
    })
}

fn append_input_coefficients(
    output: &mut Vec<u8>,
    rows: &[InputOutputCoefficientV1],
) -> Result<(), MaterialCircuitErrorV2> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.process_id.as_bytes());
        bytes.extend_from_slice(&row.good_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.quantity_per_batch.to_be_bytes());
    })
}

fn append_labor_coefficients(
    output: &mut Vec<u8>,
    rows: &[LaborCoefficientV1],
) -> Result<(), MaterialCircuitErrorV2> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.process_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.quantity_per_batch.to_be_bytes());
    })
}

fn append_supplier_routes(
    output: &mut Vec<u8>,
    rows: &[SupplierRouteV2],
) -> Result<(), MaterialCircuitErrorV2> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.buyer_site_id.as_bytes());
        bytes.extend_from_slice(&row.supplier_site_id.as_bytes());
        bytes.extend_from_slice(&row.good_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.route_id.as_bytes());
    })
}

fn append_route_legs(
    output: &mut Vec<u8>,
    rows: &[RouteLegV2],
) -> Result<(), MaterialCircuitErrorV2> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.route_id.as_bytes());
        bytes.extend_from_slice(&row.leg_index.to_be_bytes());
        bytes.extend_from_slice(&row.corridor_id.as_bytes());
        bytes.extend_from_slice(&row.from_node_id.as_bytes());
        bytes.extend_from_slice(&row.to_node_id.as_bytes());
        bytes.extend_from_slice(&row.travel_periods.to_be_bytes());
        bytes.extend_from_slice(&row.loss_ppm.to_be_bytes());
    })
}

fn append_inventory(
    output: &mut Vec<u8>,
    rows: &[InventoryRowV1],
) -> Result<(), MaterialCircuitErrorV2> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.site_id.as_bytes());
        bytes.extend_from_slice(&row.good_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.quantity.to_be_bytes());
    })
}

fn append_orders(output: &mut Vec<u8>, rows: &[OrderRowV2]) -> Result<(), MaterialCircuitErrorV2> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.order_id.as_bytes());
        bytes.push(row.access_mode as u8);
        bytes.extend_from_slice(&row.buyer_site_id.as_bytes());
        bytes.extend_from_slice(&row.supplier_site_id.as_bytes());
        bytes.extend_from_slice(&row.good_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.ordered.to_be_bytes());
        bytes.extend_from_slice(&row.shipped.to_be_bytes());
        bytes.extend_from_slice(&row.lost.to_be_bytes());
        bytes.extend_from_slice(&row.delivered.to_be_bytes());
        bytes.extend_from_slice(&row.realized.to_be_bytes());
    })
}

fn append_backlog(
    output: &mut Vec<u8>,
    rows: &[BacklogRowV1],
) -> Result<(), MaterialCircuitErrorV2> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.order_id.as_bytes());
        bytes.extend_from_slice(&row.quantity.to_be_bytes());
    })
}

fn append_freight(
    output: &mut Vec<u8>,
    rows: &[RoutedFreightLotV2],
) -> Result<(), MaterialCircuitErrorV2> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.lot_id.as_bytes());
        bytes.extend_from_slice(&row.order_id.as_bytes());
        bytes.extend_from_slice(&row.route_id.as_bytes());
        bytes.extend_from_slice(&row.dispatch_period.to_be_bytes());
        bytes.extend_from_slice(&row.current_leg_index.to_be_bytes());
        bytes.extend_from_slice(&row.leg_arrival_period.to_be_bytes());
        bytes.extend_from_slice(&row.source_site_id.as_bytes());
        bytes.extend_from_slice(&row.destination_site_id.as_bytes());
        bytes.extend_from_slice(&row.good_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.quantity.to_be_bytes());
    })
}

fn append_corridor_capacities(
    output: &mut Vec<u8>,
    rows: &[CorridorCapacityV2],
) -> Result<(), MaterialCircuitErrorV2> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.corridor_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.period.to_be_bytes());
        bytes.extend_from_slice(&row.available.to_be_bytes());
    })
}

fn append_capacities(
    output: &mut Vec<u8>,
    rows: &[CapacityRowV1],
) -> Result<(), MaterialCircuitErrorV2> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.process_id.as_bytes());
        bytes.extend_from_slice(&row.site_id.as_bytes());
        bytes.extend_from_slice(&row.period.to_be_bytes());
        bytes.extend_from_slice(&row.available_batches.to_be_bytes());
    })
}

fn append_labor(
    output: &mut Vec<u8>,
    rows: &[LaborCapacityRowV1],
) -> Result<(), MaterialCircuitErrorV2> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.site_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.period.to_be_bytes());
        bytes.extend_from_slice(&row.available.to_be_bytes());
    })
}

fn append_commitments(
    output: &mut Vec<u8>,
    rows: &[ProductionCommitmentV1],
) -> Result<(), MaterialCircuitErrorV2> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.process_id.as_bytes());
        bytes.extend_from_slice(&row.site_id.as_bytes());
        bytes.extend_from_slice(&row.period.to_be_bytes());
        bytes.extend_from_slice(&row.planned_batches.to_be_bytes());
    })
}

fn decode_site_nodes(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<SiteLogisticsNodeV2>, MaterialCircuitErrorV2> {
    decode_rows(cursor, |bytes| {
        Ok(SiteLogisticsNodeV2 {
            site_id: SiteIdV1::from_bytes(bytes.array()?),
            node_id: LogisticsNodeIdV2::from_bytes(bytes.array()?),
        })
    })
}

fn decode_process_outputs(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<ProcessOutputV1>, MaterialCircuitErrorV2> {
    decode_rows(cursor, |bytes| {
        Ok(ProcessOutputV1 {
            process_id: ProcessIdV1::from_bytes(bytes.array()?),
            site_id: SiteIdV1::from_bytes(bytes.array()?),
            good_id: GoodIdV1::from_bytes(bytes.array()?),
            unit_id: UnitIdV1::from_bytes(bytes.array()?),
            quantity_per_batch: bytes.u64()?,
        })
    })
}

fn decode_input_coefficients(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<InputOutputCoefficientV1>, MaterialCircuitErrorV2> {
    decode_rows(cursor, |bytes| {
        Ok(InputOutputCoefficientV1 {
            process_id: ProcessIdV1::from_bytes(bytes.array()?),
            good_id: GoodIdV1::from_bytes(bytes.array()?),
            unit_id: UnitIdV1::from_bytes(bytes.array()?),
            quantity_per_batch: bytes.u64()?,
        })
    })
}

fn decode_labor_coefficients(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<LaborCoefficientV1>, MaterialCircuitErrorV2> {
    decode_rows(cursor, |bytes| {
        Ok(LaborCoefficientV1 {
            process_id: ProcessIdV1::from_bytes(bytes.array()?),
            unit_id: UnitIdV1::from_bytes(bytes.array()?),
            quantity_per_batch: bytes.u64()?,
        })
    })
}

fn decode_supplier_routes(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<SupplierRouteV2>, MaterialCircuitErrorV2> {
    decode_rows(cursor, |bytes| {
        Ok(SupplierRouteV2 {
            buyer_site_id: SiteIdV1::from_bytes(bytes.array()?),
            supplier_site_id: SiteIdV1::from_bytes(bytes.array()?),
            good_id: GoodIdV1::from_bytes(bytes.array()?),
            unit_id: UnitIdV1::from_bytes(bytes.array()?),
            route_id: RouteIdV2::from_bytes(bytes.array()?),
        })
    })
}

fn decode_route_legs(cursor: &mut Cursor<'_>) -> Result<Vec<RouteLegV2>, MaterialCircuitErrorV2> {
    decode_rows(cursor, |bytes| {
        Ok(RouteLegV2 {
            route_id: RouteIdV2::from_bytes(bytes.array()?),
            leg_index: bytes.u16()?,
            corridor_id: CorridorIdV2::from_bytes(bytes.array()?),
            from_node_id: LogisticsNodeIdV2::from_bytes(bytes.array()?),
            to_node_id: LogisticsNodeIdV2::from_bytes(bytes.array()?),
            travel_periods: bytes.u16()?,
            loss_ppm: bytes.u32()?,
        })
    })
}

fn decode_inventory(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<InventoryRowV1>, MaterialCircuitErrorV2> {
    decode_rows(cursor, |bytes| {
        Ok(InventoryRowV1 {
            site_id: SiteIdV1::from_bytes(bytes.array()?),
            good_id: GoodIdV1::from_bytes(bytes.array()?),
            unit_id: UnitIdV1::from_bytes(bytes.array()?),
            quantity: bytes.u64()?,
        })
    })
}

fn decode_orders(cursor: &mut Cursor<'_>) -> Result<Vec<OrderRowV2>, MaterialCircuitErrorV2> {
    decode_rows(cursor, |bytes| {
        let order_id = OrderIdV1::from_bytes(bytes.array()?);
        let access_mode = match bytes.u8()? {
            1 => OrderAccessModeV1::CommoditySale,
            _ => return Err(MaterialCircuitErrorV2::WireEnum),
        };
        Ok(OrderRowV2 {
            order_id,
            access_mode,
            buyer_site_id: SiteIdV1::from_bytes(bytes.array()?),
            supplier_site_id: SiteIdV1::from_bytes(bytes.array()?),
            good_id: GoodIdV1::from_bytes(bytes.array()?),
            unit_id: UnitIdV1::from_bytes(bytes.array()?),
            ordered: bytes.u64()?,
            shipped: bytes.u64()?,
            lost: bytes.u64()?,
            delivered: bytes.u64()?,
            realized: bytes.u64()?,
        })
    })
}

fn decode_backlog(cursor: &mut Cursor<'_>) -> Result<Vec<BacklogRowV1>, MaterialCircuitErrorV2> {
    decode_rows(cursor, |bytes| {
        Ok(BacklogRowV1 {
            order_id: OrderIdV1::from_bytes(bytes.array()?),
            quantity: bytes.u64()?,
        })
    })
}

fn decode_freight(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<RoutedFreightLotV2>, MaterialCircuitErrorV2> {
    decode_rows(cursor, |bytes| {
        Ok(RoutedFreightLotV2 {
            lot_id: FreightLotIdV2::from_bytes(bytes.array()?),
            order_id: OrderIdV1::from_bytes(bytes.array()?),
            route_id: RouteIdV2::from_bytes(bytes.array()?),
            dispatch_period: bytes.u64()?,
            current_leg_index: bytes.u16()?,
            leg_arrival_period: bytes.u64()?,
            source_site_id: SiteIdV1::from_bytes(bytes.array()?),
            destination_site_id: SiteIdV1::from_bytes(bytes.array()?),
            good_id: GoodIdV1::from_bytes(bytes.array()?),
            unit_id: UnitIdV1::from_bytes(bytes.array()?),
            quantity: bytes.u64()?,
        })
    })
}

fn decode_corridor_capacities(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<CorridorCapacityV2>, MaterialCircuitErrorV2> {
    decode_rows(cursor, |bytes| {
        Ok(CorridorCapacityV2 {
            corridor_id: CorridorIdV2::from_bytes(bytes.array()?),
            unit_id: UnitIdV1::from_bytes(bytes.array()?),
            period: bytes.u64()?,
            available: bytes.u64()?,
        })
    })
}

fn decode_capacities(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<CapacityRowV1>, MaterialCircuitErrorV2> {
    decode_rows(cursor, |bytes| {
        Ok(CapacityRowV1 {
            process_id: ProcessIdV1::from_bytes(bytes.array()?),
            site_id: SiteIdV1::from_bytes(bytes.array()?),
            period: bytes.u64()?,
            available_batches: bytes.u64()?,
        })
    })
}

fn decode_labor(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<LaborCapacityRowV1>, MaterialCircuitErrorV2> {
    decode_rows(cursor, |bytes| {
        Ok(LaborCapacityRowV1 {
            site_id: SiteIdV1::from_bytes(bytes.array()?),
            unit_id: UnitIdV1::from_bytes(bytes.array()?),
            period: bytes.u64()?,
            available: bytes.u64()?,
        })
    })
}

fn decode_commitments(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<ProductionCommitmentV1>, MaterialCircuitErrorV2> {
    decode_rows(cursor, |bytes| {
        Ok(ProductionCommitmentV1 {
            process_id: ProcessIdV1::from_bytes(bytes.array()?),
            site_id: SiteIdV1::from_bytes(bytes.array()?),
            period: bytes.u64()?,
            planned_batches: bytes.u64()?,
        })
    })
}

/// Encode one complete validated V2 state in canonical big-endian order.
///
/// # Errors
/// Returns the first exact state, route, row-bound, or wire-bound refusal.
pub fn encode_material_circuit_state_v2(
    state: &MaterialCircuitStateV2,
) -> Result<Vec<u8>, MaterialCircuitErrorV2> {
    let canonical = canonical_state_v2(state)?;
    let mut output = Vec::new();
    output.extend_from_slice(MATERIAL_CIRCUIT_STATE_V2_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&SCHEMA_VERSION.to_be_bytes());
    output.extend_from_slice(&canonical.period.to_be_bytes());
    append_site_nodes(&mut output, &canonical.site_logistics_nodes)?;
    append_process_outputs(&mut output, &canonical.process_outputs)?;
    append_input_coefficients(&mut output, &canonical.input_coefficients)?;
    append_labor_coefficients(&mut output, &canonical.labor_coefficients)?;
    append_supplier_routes(&mut output, &canonical.supplier_routes)?;
    append_route_legs(&mut output, &canonical.route_legs)?;
    append_inventory(&mut output, &canonical.inventory)?;
    append_orders(&mut output, &canonical.orders)?;
    append_backlog(&mut output, &canonical.backlog)?;
    append_freight(&mut output, &canonical.freight)?;
    append_corridor_capacities(&mut output, &canonical.corridor_capacities)?;
    append_capacities(&mut output, &canonical.capacities)?;
    append_labor(&mut output, &canonical.labor)?;
    append_commitments(&mut output, &canonical.production_commitments)?;
    Ok(output)
}

/// Decode one complete canonical V2 state.
///
/// # Errors
/// Returns the first domain, version, enum, wire, order, or state refusal.
pub fn decode_material_circuit_state_v2(
    payload: &[u8],
) -> Result<MaterialCircuitStateV2, MaterialCircuitErrorV2> {
    let mut cursor = Cursor::new(payload);
    if cursor.take(MATERIAL_CIRCUIT_STATE_V2_DOMAIN_BYTES.len())?
        != MATERIAL_CIRCUIT_STATE_V2_DOMAIN_BYTES
        || cursor.u8()? != 0
    {
        return Err(MaterialCircuitErrorV2::WireDomain);
    }
    if cursor.u16()? != SCHEMA_VERSION {
        return Err(MaterialCircuitErrorV2::WireVersion);
    }
    let state = MaterialCircuitStateV2 {
        period: cursor.u64()?,
        site_logistics_nodes: decode_site_nodes(&mut cursor)?,
        process_outputs: decode_process_outputs(&mut cursor)?,
        input_coefficients: decode_input_coefficients(&mut cursor)?,
        labor_coefficients: decode_labor_coefficients(&mut cursor)?,
        supplier_routes: decode_supplier_routes(&mut cursor)?,
        route_legs: decode_route_legs(&mut cursor)?,
        inventory: decode_inventory(&mut cursor)?,
        orders: decode_orders(&mut cursor)?,
        backlog: decode_backlog(&mut cursor)?,
        freight: decode_freight(&mut cursor)?,
        corridor_capacities: decode_corridor_capacities(&mut cursor)?,
        capacities: decode_capacities(&mut cursor)?,
        labor: decode_labor(&mut cursor)?,
        production_commitments: decode_commitments(&mut cursor)?,
    };
    cursor.finish()?;
    let canonical = canonical_state_v2(&state)?;
    if canonical != state {
        return Err(MaterialCircuitErrorV2::WireNoncanonical);
    }
    Ok(state)
}

/// Hash one complete validated canonical V2 state.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn material_circuit_state_v2_digest(
    state: &MaterialCircuitStateV2,
) -> Result<[u8; 32], MaterialCircuitErrorV2> {
    Ok(sha256_of(&encode_material_circuit_state_v2(state)?))
}
