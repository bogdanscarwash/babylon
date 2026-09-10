//! Canonical V3 routed-material state bytes for restart and replay.

use crate::SupplierTransportV3;
use crate::{
    FinalDemandOrderV3, FinalDemandPrincipalIdV3, FinalDemandPrincipalV3,
    MerchantHandlingCoefficientV3, MerchantHandlingV3, MerchantRoleV3,
};
use babylon_kernel::sha256_of;

use crate::transition::canonical_state_v3;
use crate::{
    BacklogRowV1, CapacityRowV1, CorridorCapacityV3, CorridorIdV2, FreightLotIdV2,
    FreightMassCoefficientV3, GoodIdV1, InputOutputCoefficientV1, InventoryRowV1,
    LaborCapacityRowV1, LaborCoefficientV1, LogisticsNodeIdV2, MaterialCircuitErrorV3,
    MaterialCircuitStateV3, OrderAccessModeV1, OrderIdV1, OrderRowV2, ProcessIdV1, ProcessOutputV1,
    ProductionCommitmentV1, RouteIdV2, RouteStageCapacityV3, RouteStageV3, RoutedFreightLotV3,
    SiteIdV1, SiteLogisticsNodeV2, SupplierRouteV3, UnitIdV1, MAX_MATERIAL_CIRCUIT_ROWS_V1,
};

/// Canonical domain for one complete routed material-circuit opening state.
pub const MATERIAL_CIRCUIT_STATE_V3_DOMAIN_BYTES: &[u8] = b"babylon.material-circuit-state.v3";
/// SHA-256 of the complete language-neutral Material Circuit V3 contract source.
pub const MATERIAL_CIRCUIT_V3_SOURCE_SHA256: [u8; 32] = [
    197, 18, 133, 60, 241, 31, 36, 212, 29, 50, 104, 0, 70, 24, 192, 228, 56, 57, 91, 7, 64, 212,
    220, 252, 147, 189, 223, 80, 190, 160, 19, 75,
];
const SCHEMA_VERSION: u16 = 3;

impl From<CursorError> for MaterialCircuitErrorV3 {
    fn from(value: CursorError) -> Self {
        match value {
            CursorError::Truncated => Self::WireTruncated,
            CursorError::Trailing => Self::WireTrailing,
        }
    }
}

fn row_count(cursor: &mut Cursor<'_>) -> Result<usize, MaterialCircuitErrorV3> {
    let count = usize::try_from(cursor.u32()?).map_err(|_| MaterialCircuitErrorV3::WireLimit)?;
    if count > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        return Err(MaterialCircuitErrorV3::WireLimit);
    }
    Ok(count)
}

fn append_rows<T>(
    output: &mut Vec<u8>,
    rows: &[T],
    mut append: impl FnMut(&mut Vec<u8>, &T),
) -> Result<(), MaterialCircuitErrorV3> {
    let count = u32::try_from(rows.len()).map_err(|_| MaterialCircuitErrorV3::WireLimit)?;
    output.extend_from_slice(&count.to_be_bytes());
    for row in rows.iter().take(MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1) {
        append(output, row);
    }
    Ok(())
}

fn decode_rows<T>(
    cursor: &mut Cursor<'_>,
    mut decode: impl FnMut(&mut Cursor<'_>) -> Result<T, MaterialCircuitErrorV3>,
) -> Result<Vec<T>, MaterialCircuitErrorV3> {
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
) -> Result<(), MaterialCircuitErrorV3> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.site_id.as_bytes());
        bytes.extend_from_slice(&row.node_id.as_bytes());
    })
}

fn append_process_outputs(
    output: &mut Vec<u8>,
    rows: &[ProcessOutputV1],
) -> Result<(), MaterialCircuitErrorV3> {
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
) -> Result<(), MaterialCircuitErrorV3> {
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
) -> Result<(), MaterialCircuitErrorV3> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.process_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.quantity_per_batch.to_be_bytes());
    })
}

fn append_supplier_routes(
    output: &mut Vec<u8>,
    rows: &[SupplierRouteV3],
) -> Result<(), MaterialCircuitErrorV3> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.buyer_site_id.as_bytes());
        bytes.extend_from_slice(&row.supplier_site_id.as_bytes());
        bytes.extend_from_slice(&row.good_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.route_id.as_bytes());
        bytes.push(row.transport_kind as u8);
    })
}

fn append_route_stages(
    output: &mut Vec<u8>,
    rows: &[RouteStageV3],
) -> Result<(), MaterialCircuitErrorV3> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.route_id.as_bytes());
        bytes.extend_from_slice(&row.stage_index.to_be_bytes());
        bytes.extend_from_slice(&row.from_node_id.as_bytes());
        bytes.extend_from_slice(&row.to_node_id.as_bytes());
        bytes.extend_from_slice(&row.travel_periods.to_be_bytes());
        bytes.extend_from_slice(&row.loss_ppm.to_be_bytes());
    })
}
fn append_stage_capacities(
    output: &mut Vec<u8>,
    rows: &[RouteStageCapacityV3],
) -> Result<(), MaterialCircuitErrorV3> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.route_id.as_bytes());
        bytes.extend_from_slice(&row.stage_index.to_be_bytes());
        bytes.extend_from_slice(&row.corridor_id.as_bytes());
    })
}
fn append_mass(
    output: &mut Vec<u8>,
    rows: &[FreightMassCoefficientV3],
) -> Result<(), MaterialCircuitErrorV3> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.good_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.grams_per_unit.to_be_bytes());
    })
}

fn append_inventory(
    output: &mut Vec<u8>,
    rows: &[InventoryRowV1],
) -> Result<(), MaterialCircuitErrorV3> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.site_id.as_bytes());
        bytes.extend_from_slice(&row.good_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.quantity.to_be_bytes());
    })
}

fn append_orders(output: &mut Vec<u8>, rows: &[OrderRowV2]) -> Result<(), MaterialCircuitErrorV3> {
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
) -> Result<(), MaterialCircuitErrorV3> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.order_id.as_bytes());
        bytes.extend_from_slice(&row.quantity.to_be_bytes());
    })
}

fn append_freight(
    output: &mut Vec<u8>,
    rows: &[RoutedFreightLotV3],
) -> Result<(), MaterialCircuitErrorV3> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.lot_id.as_bytes());
        bytes.extend_from_slice(&row.order_id.as_bytes());
        bytes.extend_from_slice(&row.route_id.as_bytes());
        bytes.extend_from_slice(&row.dispatch_period.to_be_bytes());
        bytes.extend_from_slice(&row.current_stage_index.to_be_bytes());
        bytes.extend_from_slice(&row.stage_arrival_period.to_be_bytes());
        bytes.extend_from_slice(&row.source_site_id.as_bytes());
        bytes.extend_from_slice(&row.destination_site_id.as_bytes());
        bytes.extend_from_slice(&row.good_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.quantity.to_be_bytes());
    })
}

fn append_corridor_capacities(
    output: &mut Vec<u8>,
    rows: &[CorridorCapacityV3],
) -> Result<(), MaterialCircuitErrorV3> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.corridor_id.as_bytes());
        bytes.extend_from_slice(&row.period.to_be_bytes());
        bytes.extend_from_slice(&row.available_grams.to_be_bytes());
    })
}

fn append_capacities(
    output: &mut Vec<u8>,
    rows: &[CapacityRowV1],
) -> Result<(), MaterialCircuitErrorV3> {
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
) -> Result<(), MaterialCircuitErrorV3> {
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
) -> Result<(), MaterialCircuitErrorV3> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.process_id.as_bytes());
        bytes.extend_from_slice(&row.site_id.as_bytes());
        bytes.extend_from_slice(&row.period.to_be_bytes());
        bytes.extend_from_slice(&row.planned_batches.to_be_bytes());
    })
}

fn decode_site_nodes(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<SiteLogisticsNodeV2>, MaterialCircuitErrorV3> {
    decode_rows(cursor, |bytes| {
        Ok(SiteLogisticsNodeV2 {
            site_id: SiteIdV1::from_bytes(bytes.array()?),
            node_id: LogisticsNodeIdV2::from_bytes(bytes.array()?),
        })
    })
}

fn decode_process_outputs(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<ProcessOutputV1>, MaterialCircuitErrorV3> {
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
) -> Result<Vec<InputOutputCoefficientV1>, MaterialCircuitErrorV3> {
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
) -> Result<Vec<LaborCoefficientV1>, MaterialCircuitErrorV3> {
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
) -> Result<Vec<SupplierRouteV3>, MaterialCircuitErrorV3> {
    decode_rows(cursor, |bytes| {
        Ok(SupplierRouteV3 {
            buyer_site_id: SiteIdV1::from_bytes(bytes.array()?),
            supplier_site_id: SiteIdV1::from_bytes(bytes.array()?),
            good_id: GoodIdV1::from_bytes(bytes.array()?),
            unit_id: UnitIdV1::from_bytes(bytes.array()?),
            route_id: RouteIdV2::from_bytes(bytes.array()?),
            transport_kind: match bytes.u8()? {
                1 => SupplierTransportV3::Local,
                2 => SupplierTransportV3::Staged,
                _ => return Err(MaterialCircuitErrorV3::WireEnum),
            },
        })
    })
}

fn decode_route_stages(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<RouteStageV3>, MaterialCircuitErrorV3> {
    decode_rows(cursor, |bytes| {
        Ok(RouteStageV3 {
            route_id: RouteIdV2::from_bytes(bytes.array()?),
            stage_index: bytes.u16()?,
            from_node_id: LogisticsNodeIdV2::from_bytes(bytes.array()?),
            to_node_id: LogisticsNodeIdV2::from_bytes(bytes.array()?),
            travel_periods: bytes.u16()?,
            loss_ppm: bytes.u32()?,
        })
    })
}
fn decode_stage_capacities(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<RouteStageCapacityV3>, MaterialCircuitErrorV3> {
    decode_rows(cursor, |bytes| {
        Ok(RouteStageCapacityV3 {
            route_id: RouteIdV2::from_bytes(bytes.array()?),
            stage_index: bytes.u16()?,
            corridor_id: CorridorIdV2::from_bytes(bytes.array()?),
        })
    })
}
fn decode_mass(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<FreightMassCoefficientV3>, MaterialCircuitErrorV3> {
    decode_rows(cursor, |bytes| {
        Ok(FreightMassCoefficientV3 {
            good_id: GoodIdV1::from_bytes(bytes.array()?),
            unit_id: UnitIdV1::from_bytes(bytes.array()?),
            grams_per_unit: bytes.u64()?,
        })
    })
}

fn decode_inventory(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<InventoryRowV1>, MaterialCircuitErrorV3> {
    decode_rows(cursor, |bytes| {
        Ok(InventoryRowV1 {
            site_id: SiteIdV1::from_bytes(bytes.array()?),
            good_id: GoodIdV1::from_bytes(bytes.array()?),
            unit_id: UnitIdV1::from_bytes(bytes.array()?),
            quantity: bytes.u64()?,
        })
    })
}

fn decode_orders(cursor: &mut Cursor<'_>) -> Result<Vec<OrderRowV2>, MaterialCircuitErrorV3> {
    decode_rows(cursor, |bytes| {
        let order_id = OrderIdV1::from_bytes(bytes.array()?);
        let access_mode = match bytes.u8()? {
            1 => OrderAccessModeV1::CommoditySale,
            _ => return Err(MaterialCircuitErrorV3::WireEnum),
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

fn decode_backlog(cursor: &mut Cursor<'_>) -> Result<Vec<BacklogRowV1>, MaterialCircuitErrorV3> {
    decode_rows(cursor, |bytes| {
        Ok(BacklogRowV1 {
            order_id: OrderIdV1::from_bytes(bytes.array()?),
            quantity: bytes.u64()?,
        })
    })
}

fn decode_freight(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<RoutedFreightLotV3>, MaterialCircuitErrorV3> {
    decode_rows(cursor, |bytes| {
        Ok(RoutedFreightLotV3 {
            lot_id: FreightLotIdV2::from_bytes(bytes.array()?),
            order_id: OrderIdV1::from_bytes(bytes.array()?),
            route_id: RouteIdV2::from_bytes(bytes.array()?),
            dispatch_period: bytes.u64()?,
            current_stage_index: bytes.u16()?,
            stage_arrival_period: bytes.u64()?,
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
) -> Result<Vec<CorridorCapacityV3>, MaterialCircuitErrorV3> {
    decode_rows(cursor, |bytes| {
        Ok(CorridorCapacityV3 {
            corridor_id: CorridorIdV2::from_bytes(bytes.array()?),
            period: bytes.u64()?,
            available_grams: bytes.u64()?,
        })
    })
}

fn decode_capacities(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<CapacityRowV1>, MaterialCircuitErrorV3> {
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
) -> Result<Vec<LaborCapacityRowV1>, MaterialCircuitErrorV3> {
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
) -> Result<Vec<ProductionCommitmentV1>, MaterialCircuitErrorV3> {
    decode_rows(cursor, |bytes| {
        Ok(ProductionCommitmentV1 {
            process_id: ProcessIdV1::from_bytes(bytes.array()?),
            site_id: SiteIdV1::from_bytes(bytes.array()?),
            period: bytes.u64()?,
            planned_batches: bytes.u64()?,
        })
    })
}

fn append_merchants(
    output: &mut Vec<u8>,
    rows: &[MerchantHandlingV3],
) -> Result<(), MaterialCircuitErrorV3> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.site_id.as_bytes());
        bytes.extend_from_slice(&row.county_geoid);
        bytes.push(row.role as u8);
        bytes.extend_from_slice(&row.capacity_id.as_bytes());
        bytes.extend_from_slice(&row.labor_unit_id.as_bytes());
    })
}

fn decode_merchants(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<MerchantHandlingV3>, MaterialCircuitErrorV3> {
    decode_rows(cursor, |bytes| {
        Ok(MerchantHandlingV3 {
            site_id: SiteIdV1::from_bytes(bytes.array()?),
            county_geoid: bytes.array()?,
            role: match bytes.u8()? {
                1 => MerchantRoleV3::Wholesale,
                2 => MerchantRoleV3::Retail,
                _ => return Err(MaterialCircuitErrorV3::WireEnum),
            },
            capacity_id: CorridorIdV2::from_bytes(bytes.array()?),
            labor_unit_id: UnitIdV1::from_bytes(bytes.array()?),
        })
    })
}

fn append_handling_coefficients(
    output: &mut Vec<u8>,
    rows: &[MerchantHandlingCoefficientV3],
) -> Result<(), MaterialCircuitErrorV3> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.site_id.as_bytes());
        bytes.extend_from_slice(&row.good_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.hours_per_unit.to_be_bytes());
    })
}

fn decode_handling_coefficients(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<MerchantHandlingCoefficientV3>, MaterialCircuitErrorV3> {
    decode_rows(cursor, |bytes| {
        Ok(MerchantHandlingCoefficientV3 {
            site_id: SiteIdV1::from_bytes(bytes.array()?),
            good_id: GoodIdV1::from_bytes(bytes.array()?),
            unit_id: UnitIdV1::from_bytes(bytes.array()?),
            hours_per_unit: bytes.u64()?,
        })
    })
}

fn append_final_demand_principals(
    output: &mut Vec<u8>,
    rows: &[FinalDemandPrincipalV3],
) -> Result<(), MaterialCircuitErrorV3> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.id.as_bytes());
        bytes.extend_from_slice(&row.county_geoid);
    })
}

fn decode_final_demand_principals(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<FinalDemandPrincipalV3>, MaterialCircuitErrorV3> {
    decode_rows(cursor, |bytes| {
        Ok(FinalDemandPrincipalV3 {
            id: FinalDemandPrincipalIdV3::from_bytes(bytes.array()?),
            county_geoid: bytes.array()?,
        })
    })
}

fn append_final_demand_orders(
    output: &mut Vec<u8>,
    rows: &[FinalDemandOrderV3],
) -> Result<(), MaterialCircuitErrorV3> {
    append_rows(output, rows, |bytes, row| {
        bytes.extend_from_slice(&row.order_id.as_bytes());
        bytes.extend_from_slice(&row.retailer_site_id.as_bytes());
        bytes.extend_from_slice(&row.demand_principal_id.as_bytes());
        bytes.extend_from_slice(&row.good_id.as_bytes());
        bytes.extend_from_slice(&row.unit_id.as_bytes());
        bytes.extend_from_slice(&row.ordered.to_be_bytes());
        bytes.extend_from_slice(&row.fulfilled.to_be_bytes());
    })
}

fn decode_final_demand_orders(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<FinalDemandOrderV3>, MaterialCircuitErrorV3> {
    decode_rows(cursor, |bytes| {
        Ok(FinalDemandOrderV3 {
            order_id: OrderIdV1::from_bytes(bytes.array()?),
            retailer_site_id: SiteIdV1::from_bytes(bytes.array()?),
            demand_principal_id: FinalDemandPrincipalIdV3::from_bytes(bytes.array()?),
            good_id: GoodIdV1::from_bytes(bytes.array()?),
            unit_id: UnitIdV1::from_bytes(bytes.array()?),
            ordered: bytes.u64()?,
            fulfilled: bytes.u64()?,
        })
    })
}

/// Encode one complete validated V3 state in canonical big-endian order.
///
/// # Errors
/// Returns the first exact state, route, row-bound, or wire-bound refusal.
pub fn encode_material_circuit_state_v3(
    state: &MaterialCircuitStateV3,
) -> Result<Vec<u8>, MaterialCircuitErrorV3> {
    let canonical = canonical_state_v3(state)?;
    let mut output = Vec::new();
    output.extend_from_slice(MATERIAL_CIRCUIT_STATE_V3_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&SCHEMA_VERSION.to_be_bytes());
    output.extend_from_slice(&canonical.period.to_be_bytes());
    append_site_nodes(&mut output, &canonical.site_logistics_nodes)?;
    append_process_outputs(&mut output, &canonical.process_outputs)?;
    append_input_coefficients(&mut output, &canonical.input_coefficients)?;
    append_labor_coefficients(&mut output, &canonical.labor_coefficients)?;
    append_mass(&mut output, &canonical.freight_mass_coefficients)?;
    append_supplier_routes(&mut output, &canonical.supplier_routes)?;
    append_route_stages(&mut output, &canonical.route_stages)?;
    append_stage_capacities(&mut output, &canonical.route_stage_capacities)?;
    append_inventory(&mut output, &canonical.inventory)?;
    append_orders(&mut output, &canonical.orders)?;
    append_backlog(&mut output, &canonical.backlog)?;
    append_freight(&mut output, &canonical.freight)?;
    append_corridor_capacities(&mut output, &canonical.corridor_capacities)?;
    append_capacities(&mut output, &canonical.capacities)?;
    append_labor(&mut output, &canonical.labor)?;
    append_commitments(&mut output, &canonical.production_commitments)?;
    append_merchants(&mut output, &canonical.merchants)?;
    append_handling_coefficients(&mut output, &canonical.handling_coefficients)?;
    append_final_demand_principals(&mut output, &canonical.final_demand_principals)?;
    append_final_demand_orders(&mut output, &canonical.final_demand_orders)?;

    Ok(output)
}

/// Decode one complete canonical V3 state.
///
/// # Errors
/// Returns the first domain, version, enum, wire, order, or state refusal.
pub fn decode_material_circuit_state_v3(
    payload: &[u8],
) -> Result<MaterialCircuitStateV3, MaterialCircuitErrorV3> {
    let mut cursor = Cursor::new(payload);
    if cursor.take(MATERIAL_CIRCUIT_STATE_V3_DOMAIN_BYTES.len())?
        != MATERIAL_CIRCUIT_STATE_V3_DOMAIN_BYTES
        || cursor.u8()? != 0
    {
        return Err(MaterialCircuitErrorV3::WireDomain);
    }
    if cursor.u16()? != SCHEMA_VERSION {
        return Err(MaterialCircuitErrorV3::WireVersion);
    }
    let state = MaterialCircuitStateV3 {
        period: cursor.u64()?,
        site_logistics_nodes: decode_site_nodes(&mut cursor)?,
        process_outputs: decode_process_outputs(&mut cursor)?,
        input_coefficients: decode_input_coefficients(&mut cursor)?,
        labor_coefficients: decode_labor_coefficients(&mut cursor)?,
        freight_mass_coefficients: decode_mass(&mut cursor)?,
        supplier_routes: decode_supplier_routes(&mut cursor)?,
        route_stages: decode_route_stages(&mut cursor)?,
        route_stage_capacities: decode_stage_capacities(&mut cursor)?,
        inventory: decode_inventory(&mut cursor)?,
        orders: decode_orders(&mut cursor)?,
        backlog: decode_backlog(&mut cursor)?,
        freight: decode_freight(&mut cursor)?,
        corridor_capacities: decode_corridor_capacities(&mut cursor)?,
        capacities: decode_capacities(&mut cursor)?,
        labor: decode_labor(&mut cursor)?,
        production_commitments: decode_commitments(&mut cursor)?,
        merchants: decode_merchants(&mut cursor)?,
        handling_coefficients: decode_handling_coefficients(&mut cursor)?,
        final_demand_principals: decode_final_demand_principals(&mut cursor)?,
        final_demand_orders: decode_final_demand_orders(&mut cursor)?,
    };
    cursor.finish()?;
    let canonical = canonical_state_v3(&state)?;
    if canonical != state {
        return Err(MaterialCircuitErrorV3::WireNoncanonical);
    }
    Ok(state)
}

/// Hash one complete validated canonical V3 state.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn material_circuit_state_v3_digest(
    state: &MaterialCircuitStateV3,
) -> Result<[u8; 32], MaterialCircuitErrorV3> {
    Ok(sha256_of(&encode_material_circuit_state_v3(state)?))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CursorError {
    Truncated,
    Trailing,
}

pub(crate) struct Cursor<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0 }
    }

    pub(crate) fn take(&mut self, length: usize) -> Result<&'a [u8], CursorError> {
        let end = self
            .index
            .checked_add(length)
            .ok_or(CursorError::Truncated)?;
        let output = self
            .bytes
            .get(self.index..end)
            .ok_or(CursorError::Truncated)?;
        self.index = end;
        Ok(output)
    }

    pub(crate) fn array<const N: usize>(&mut self) -> Result<[u8; N], CursorError> {
        self.take(N)?.try_into().map_err(|_| CursorError::Truncated)
    }

    pub(crate) fn u8(&mut self) -> Result<u8, CursorError> {
        Ok(self.array::<1>()?[0])
    }

    pub(crate) fn u16(&mut self) -> Result<u16, CursorError> {
        Ok(u16::from_be_bytes(self.array()?))
    }

    pub(crate) fn u32(&mut self) -> Result<u32, CursorError> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    pub(crate) fn u64(&mut self) -> Result<u64, CursorError> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    pub(crate) fn finish(self) -> Result<(), CursorError> {
        if self.index == self.bytes.len() {
            Ok(())
        } else {
            Err(CursorError::Trailing)
        }
    }
}
