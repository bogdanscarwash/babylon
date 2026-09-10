//! Exact successor world-register ownership for the routed material circuit.
//!
//! An active register contains the complete V3 opening state, never an economic
//! summary or a second inventory ledger. The graph-only digest stays unchanged.

use babylon_kernel::sha256_of;
use babylon_material_circuit::{
    advance_material_circuit_v3, decode_material_circuit_state_v3,
    encode_material_circuit_state_v3, MaterialCircuitErrorV3, MaterialCircuitStateV3,
    MaterialCircuitTransitionV3,
};

const REGISTER_DOMAIN: &[u8] = b"babylon.material-world-register.v3\0";
const NOMINAL_DOMAIN: &[u8] = b"babylon.nominal-material-world.v3\0";
const RECEIPT_DOMAIN: &[u8] = b"babylon.material-tick-receipts.v4\0";
/// Shared identity ceiling inherited by the aggregate replay envelope.
pub const MAX_MATERIAL_WORLD_REGISTER_BYTES_V3: usize = 67_108_864;

/// One checked complete material register at a completed four-week boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialWorldRegisterV3 {
    completed_tick: u64,
    state: MaterialCircuitStateV3,
    canonical_bytes: Vec<u8>,
    digest: [u8; 32],
}

/// Failure never publishes any part of a proposed material transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialWorldErrorV3 {
    Circuit(MaterialCircuitErrorV3),
    PeriodMismatch,
    Arithmetic,
    ByteLimit,
    Allocation,
    Wire,
}
impl std::fmt::Display for MaterialWorldErrorV3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "material world refused: {self:?}")
    }
}
impl std::error::Error for MaterialWorldErrorV3 {}
impl From<MaterialCircuitErrorV3> for MaterialWorldErrorV3 {
    fn from(error: MaterialCircuitErrorV3) -> Self {
        Self::Circuit(error)
    }
}

impl MaterialWorldRegisterV3 {
    /// Own and validate the full opening state for the next four-week interval.
    /// # Errors
    /// Refuses invalid circuit state, period mismatch, overflow or aggregate byte bound.
    pub fn try_new(
        completed_tick: u64,
        state: MaterialCircuitStateV3,
    ) -> Result<Self, MaterialWorldErrorV3> {
        if completed_tick
            .checked_add(1)
            .ok_or(MaterialWorldErrorV3::Arithmetic)?
            != state.period
        {
            return Err(MaterialWorldErrorV3::PeriodMismatch);
        }
        let state_bytes = encode_material_circuit_state_v3(&state)?;
        let state = decode_material_circuit_state_v3(&state_bytes)?;
        let length = REGISTER_DOMAIN
            .len()
            .checked_add(20)
            .and_then(|count| count.checked_add(state_bytes.len()))
            .ok_or(MaterialWorldErrorV3::Arithmetic)?;
        let mut bytes = bounded_bytes(length)?;
        bytes.extend_from_slice(REGISTER_DOMAIN);
        bytes.extend_from_slice(&3_u32.to_be_bytes());
        bytes.extend_from_slice(&completed_tick.to_be_bytes());
        bytes.extend_from_slice(
            &u64::try_from(state_bytes.len())
                .map_err(|_| MaterialWorldErrorV3::Arithmetic)?
                .to_be_bytes(),
        );
        bytes.extend_from_slice(&state_bytes);
        let digest = sha256_of(&bytes);
        Ok(Self {
            completed_tick,
            state,
            canonical_bytes: bytes,
            digest,
        })
    }
    #[must_use]
    pub const fn completed_tick(&self) -> u64 {
        self.completed_tick
    }
    #[must_use]
    pub fn state(&self) -> &MaterialCircuitStateV3 {
        &self.state
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    /// Decode the exact complete register, including canonical state revalidation.
    /// # Errors
    /// Refuses unsupported/truncated/trailing/noncanonical bytes and invalid state.
    pub fn decode(bytes: &[u8]) -> Result<Self, MaterialWorldErrorV3> {
        let header = REGISTER_DOMAIN.len() + 20;
        if bytes.len() < header
            || bytes.len() > MAX_MATERIAL_WORLD_REGISTER_BYTES_V3
            || !bytes.starts_with(REGISTER_DOMAIN)
        {
            return Err(MaterialWorldErrorV3::Wire);
        }
        let start = REGISTER_DOMAIN.len();
        if bytes[start..start + 4] != 3_u32.to_be_bytes() {
            return Err(MaterialWorldErrorV3::Wire);
        }
        let tick = u64::from_be_bytes(
            bytes[start + 4..start + 12]
                .try_into()
                .map_err(|_| MaterialWorldErrorV3::Wire)?,
        );
        let length = usize::try_from(u64::from_be_bytes(
            bytes[start + 12..header]
                .try_into()
                .map_err(|_| MaterialWorldErrorV3::Wire)?,
        ))
        .map_err(|_| MaterialWorldErrorV3::ByteLimit)?;
        if header.checked_add(length) != Some(bytes.len()) {
            return Err(MaterialWorldErrorV3::Wire);
        }
        let register = Self::try_new(tick, decode_material_circuit_state_v3(&bytes[header..])?)?;
        if register.canonical_bytes != bytes {
            return Err(MaterialWorldErrorV3::Wire);
        }
        Ok(register)
    }

    /// Prepare one next-period successor without mutating this register.
    /// The existing material transition executes arrivals, prior commitments,
    /// dispatch and following-period commitments in its governed order.
    /// # Errors
    /// Refuses any circuit or receipt encoding failure without changing this owner.
    pub fn prepare_next(&self) -> Result<PreparedMaterialWorldV4, MaterialWorldErrorV3> {
        let transition = advance_material_circuit_v3(&self.state)?;
        self.prepare_transition(transition)
    }

    /// Seal the result of the shared closed-period planner on this exact opening.
    pub(crate) fn prepare_transition(
        &self,
        transition: MaterialCircuitTransitionV3,
    ) -> Result<PreparedMaterialWorldV4, MaterialWorldErrorV3> {
        if self.state.period.checked_add(1) != Some(transition.state.period) {
            return Err(MaterialWorldErrorV3::PeriodMismatch);
        }
        let receipts = encode_material_receipts_v4(self.state.period, &transition)?;
        let next = self
            .completed_tick
            .checked_add(1)
            .ok_or(MaterialWorldErrorV3::Arithmetic)?;
        let register = Self::try_new(next, transition.state)?;
        Ok(PreparedMaterialWorldV4 {
            prior_digest: self.digest,
            register,
            receipt_bytes: receipts,
        })
    }
}

/// Detached exact successor and its immutable material event evidence.
#[derive(Debug, PartialEq, Eq)]
pub struct PreparedMaterialWorldV4 {
    prior_digest: [u8; 32],
    register: MaterialWorldRegisterV3,
    receipt_bytes: Vec<u8>,
}
impl PreparedMaterialWorldV4 {
    #[must_use]
    pub const fn prior_digest(&self) -> [u8; 32] {
        self.prior_digest
    }
    #[must_use]
    pub const fn register(&self) -> &MaterialWorldRegisterV3 {
        &self.register
    }
    #[must_use]
    pub fn receipt_bytes(&self) -> &[u8] {
        &self.receipt_bytes
    }
    #[must_use]
    pub fn into_register(self) -> MaterialWorldRegisterV3 {
        self.register
    }
}

/// Successor nominal identity, binding both the graph world's identity and material state.
#[must_use]
pub fn nominal_material_world_hash_v3(
    graph_world_hash: [u8; 32],
    register: &MaterialWorldRegisterV3,
) -> [u8; 32] {
    let mut bytes = [0_u8; NOMINAL_DOMAIN.len() + 68];
    let domain = NOMINAL_DOMAIN.len();
    bytes[..domain].copy_from_slice(NOMINAL_DOMAIN);
    bytes[domain..domain + 4].copy_from_slice(&3_u32.to_be_bytes());
    bytes[domain + 4..domain + 36].copy_from_slice(&graph_world_hash);
    bytes[domain + 36..domain + 68].copy_from_slice(&register.digest);
    sha256_of(&bytes[..domain + 68])
}

fn bounded_bytes(length: usize) -> Result<Vec<u8>, MaterialWorldErrorV3> {
    if length > MAX_MATERIAL_WORLD_REGISTER_BYTES_V3 {
        return Err(MaterialWorldErrorV3::ByteLimit);
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| MaterialWorldErrorV3::Allocation)?;
    Ok(bytes)
}
fn encode_material_receipts_v4(
    tick: u64,
    transition: &MaterialCircuitTransitionV3,
) -> Result<Vec<u8>, MaterialWorldErrorV3> {
    let families = [
        (transition.production.len(), 80_usize),
        (transition.dispatches.len(), 112),
        (transition.losses.len(), 106),
        (transition.arrivals.len(), 40),
        (transition.deliveries.len(), 40),
        (transition.realizations.len(), 40),
        (transition.handling.len(), 97),
        (transition.local_fulfillments.len(), 168),
        (transition.local_transfers.len(), 168),
    ];
    let length = families.iter().try_fold(
        RECEIPT_DOMAIN.len() + 12 + 9 * 9,
        |total, (count, width)| {
            total
                .checked_add(
                    count
                        .checked_mul(*width)
                        .ok_or(MaterialWorldErrorV3::Arithmetic)?,
                )
                .ok_or(MaterialWorldErrorV3::Arithmetic)
        },
    )?;
    let mut bytes = bounded_bytes(length)?;
    bytes.extend_from_slice(RECEIPT_DOMAIN);
    bytes.extend_from_slice(&4_u32.to_be_bytes());
    bytes.extend_from_slice(&tick.to_be_bytes());
    for (tag, (count, _)) in families.iter().enumerate() {
        bytes.push(u8::try_from(tag + 1).map_err(|_| MaterialWorldErrorV3::Arithmetic)?);
        bytes.extend_from_slice(
            &u64::try_from(*count)
                .map_err(|_| MaterialWorldErrorV3::Arithmetic)?
                .to_be_bytes(),
        );
        match tag {
            0 => {
                for row in &transition.production {
                    bytes.extend_from_slice(&row.process_id.as_bytes());
                    bytes.extend_from_slice(&row.site_id.as_bytes());
                    bytes.extend_from_slice(&row.planned_batches.to_be_bytes());
                    bytes.extend_from_slice(&row.produced_batches.to_be_bytes());
                }
            }
            1 => {
                for row in &transition.dispatches {
                    bytes.extend_from_slice(&row.lot_id.as_bytes());
                    bytes.extend_from_slice(&row.order_id.as_bytes());
                    bytes.extend_from_slice(&row.route_id.as_bytes());
                    bytes.extend_from_slice(&row.quantity.to_be_bytes());
                    bytes.extend_from_slice(&row.final_arrival_period.to_be_bytes());
                }
            }
            2 => {
                for row in &transition.losses {
                    bytes.extend_from_slice(&row.lot_id.as_bytes());
                    bytes.extend_from_slice(&row.order_id.as_bytes());
                    bytes.extend_from_slice(&row.route_id.as_bytes());
                    bytes.extend_from_slice(&row.stage_index.to_be_bytes());
                    bytes.extend_from_slice(&row.quantity.to_be_bytes());
                }
            }
            3 => {
                for row in &transition.arrivals {
                    bytes.extend_from_slice(&row.order_id.as_bytes());
                    bytes.extend_from_slice(&row.quantity.to_be_bytes());
                }
            }
            4 => {
                for row in &transition.deliveries {
                    bytes.extend_from_slice(&row.order_id.as_bytes());
                    bytes.extend_from_slice(&row.quantity.to_be_bytes());
                }
            }
            5 => {
                for row in &transition.realizations {
                    bytes.extend_from_slice(&row.order_id.as_bytes());
                    bytes.extend_from_slice(&row.quantity.to_be_bytes());
                }
            }
            6 => {
                use babylon_material_circuit::OutboundOrderIdV3;
                for row in &transition.handling {
                    bytes.extend_from_slice(&row.site_id.as_bytes());
                    let (tag, id) = match row.order {
                        OutboundOrderIdV3::Delivery(id) => (1, id),
                        OutboundOrderIdV3::LocalFinalDemand(id) => (2, id),
                    };
                    bytes.push(tag);
                    bytes.extend_from_slice(&id.as_bytes());
                    for quantity in [
                        row.feasible_quantity,
                        row.handled_quantity,
                        row.needed_hours,
                        row.used_hours,
                    ] {
                        bytes.extend_from_slice(&quantity.to_be_bytes());
                    }
                }
            }
            7 => {
                for row in &transition.local_fulfillments {
                    bytes.extend_from_slice(&row.order_id.as_bytes());
                    bytes.extend_from_slice(&row.retailer_site_id.as_bytes());
                    bytes.extend_from_slice(&row.demand_principal_id.as_bytes());
                    bytes.extend_from_slice(&row.good_id.as_bytes());
                    bytes.extend_from_slice(&row.unit_id.as_bytes());
                    bytes.extend_from_slice(&row.quantity.to_be_bytes());
                }
            }
            8 => {
                for row in &transition.local_transfers {
                    bytes.extend_from_slice(&row.order_id.as_bytes());
                    bytes.extend_from_slice(&row.supplier_site_id.as_bytes());
                    bytes.extend_from_slice(&row.buyer_site_id.as_bytes());
                    bytes.extend_from_slice(&row.good_id.as_bytes());
                    bytes.extend_from_slice(&row.unit_id.as_bytes());
                    bytes.extend_from_slice(&row.quantity.to_be_bytes());
                }
            }
            _ => unreachable!("the nine material receipt families are closed"),
        }
    }
    debug_assert_eq!(bytes.len(), length);
    Ok(bytes)
}

/// Typed material evidence decoded only from an exact committed V4 receipt family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialTickReceiptsV4 {
    pub resolve_tick: u64,
    pub production: Vec<babylon_material_circuit::ProductionReceiptV1>,
    pub dispatches: Vec<babylon_material_circuit::RoutedDispatchReceiptV2>,
    pub losses: Vec<babylon_material_circuit::FreightLossReceiptV3>,
    pub arrivals: Vec<babylon_material_circuit::ArrivalReceiptV1>,
    pub deliveries: Vec<babylon_material_circuit::DeliveryReceiptV1>,
    pub realizations: Vec<babylon_material_circuit::RealizationReceiptV1>,
    pub handling: Vec<babylon_material_circuit::MerchantHandlingReceiptV3>,
    pub local_fulfillments: Vec<babylon_material_circuit::LocalRetailFulfillmentReceiptV3>,
    pub local_transfers: Vec<babylon_material_circuit::LocalTransferReceiptV3>,
}
/// Decode a bounded receipt family. Hash/campaign binding belongs to its V3 envelope.
/// # Errors
/// Refuses versions, tags, counts, truncation, trailing bytes and invalid quantity relations.
pub fn decode_material_receipts_v4(
    bytes: &[u8],
) -> Result<MaterialTickReceiptsV4, MaterialWorldErrorV3> {
    use babylon_material_circuit::*;
    if bytes.len() > MAX_MATERIAL_WORLD_REGISTER_BYTES_V3 || !bytes.starts_with(RECEIPT_DOMAIN) {
        return Err(MaterialWorldErrorV3::Wire);
    }
    let mut cursor = ReceiptCursorV4 {
        bytes,
        position: RECEIPT_DOMAIN.len(),
    };
    if cursor.take::<4>()? != 4_u32.to_be_bytes() {
        return Err(MaterialWorldErrorV3::Wire);
    }
    let resolve_tick = cursor.u64()?;
    if resolve_tick == 0 {
        return Err(MaterialWorldErrorV3::Wire);
    }
    let mut result = MaterialTickReceiptsV4 {
        resolve_tick,
        production: Vec::new(),
        dispatches: Vec::new(),
        losses: Vec::new(),
        arrivals: Vec::new(),
        deliveries: Vec::new(),
        realizations: Vec::new(),
        handling: Vec::new(),
        local_fulfillments: Vec::new(),
        local_transfers: Vec::new(),
    };
    for tag in 1..=9 {
        if cursor.take::<1>()? != [tag] {
            return Err(MaterialWorldErrorV3::Wire);
        }
        let count = usize::try_from(cursor.u64()?).map_err(|_| MaterialWorldErrorV3::ByteLimit)?;
        if count > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
            return Err(MaterialWorldErrorV3::ByteLimit);
        }
        let width = [80_usize, 112, 106, 40, 40, 40, 97, 168, 168][usize::from(tag - 1)];
        if count
            .checked_mul(width)
            .is_none_or(|length| length > bytes.len() - cursor.position)
        {
            return Err(MaterialWorldErrorV3::Wire);
        }
        match tag {
            1 => result.production.try_reserve_exact(count),
            2 => result.dispatches.try_reserve_exact(count),
            3 => result.losses.try_reserve_exact(count),
            4 => result.arrivals.try_reserve_exact(count),
            5 => result.deliveries.try_reserve_exact(count),
            6 => result.realizations.try_reserve_exact(count),
            7 => result.handling.try_reserve_exact(count),
            8 => result.local_fulfillments.try_reserve_exact(count),
            9 => result.local_transfers.try_reserve_exact(count),
            _ => return Err(MaterialWorldErrorV3::Wire),
        }
        .map_err(|_| MaterialWorldErrorV3::Allocation)?;
        for _ in 0..count {
            match tag {
                1 => {
                    let process_id = ProcessIdV1::from_bytes(cursor.take()?);
                    let site_id = SiteIdV1::from_bytes(cursor.take()?);
                    let planned_batches = cursor.u64()?;
                    let produced_batches = cursor.u64()?;
                    if produced_batches > planned_batches {
                        return Err(MaterialWorldErrorV3::Wire);
                    }
                    result.production.push(ProductionReceiptV1 {
                        process_id,
                        site_id,
                        planned_batches,
                        produced_batches,
                    });
                }
                2 => {
                    let lot_id = FreightLotIdV2::from_bytes(cursor.take()?);
                    let order_id = OrderIdV1::from_bytes(cursor.take()?);
                    let route_id = RouteIdV2::from_bytes(cursor.take()?);
                    let quantity = cursor.positive()?;
                    let final_arrival_period = cursor.u64()?;
                    if final_arrival_period <= resolve_tick {
                        return Err(MaterialWorldErrorV3::Wire);
                    }
                    result.dispatches.push(RoutedDispatchReceiptV2 {
                        lot_id,
                        order_id,
                        route_id,
                        quantity,
                        final_arrival_period,
                    });
                }
                3 => {
                    let lot_id = FreightLotIdV2::from_bytes(cursor.take()?);
                    let order_id = OrderIdV1::from_bytes(cursor.take()?);
                    let route_id = RouteIdV2::from_bytes(cursor.take()?);
                    let stage_index = u16::from_be_bytes(cursor.take()?);
                    if usize::from(stage_index) >= MAX_ROUTE_STAGES_PER_ROUTE_V3 {
                        return Err(MaterialWorldErrorV3::Wire);
                    }
                    let quantity = cursor.positive()?;
                    result.losses.push(FreightLossReceiptV3 {
                        lot_id,
                        order_id,
                        route_id,
                        stage_index,
                        quantity,
                    });
                }
                4 => result.arrivals.push(ArrivalReceiptV1 {
                    order_id: OrderIdV1::from_bytes(cursor.take()?),
                    quantity: cursor.positive()?,
                }),
                5 => result.deliveries.push(DeliveryReceiptV1 {
                    order_id: OrderIdV1::from_bytes(cursor.take()?),
                    quantity: cursor.positive()?,
                }),
                6 => result.realizations.push(RealizationReceiptV1 {
                    order_id: OrderIdV1::from_bytes(cursor.take()?),
                    quantity: cursor.positive()?,
                }),
                7 => {
                    let site_id = SiteIdV1::from_bytes(cursor.take()?);
                    let [order_tag] = cursor.take()?;
                    let id = OrderIdV1::from_bytes(cursor.take()?);
                    let order = match order_tag {
                        1 => OutboundOrderIdV3::Delivery(id),
                        2 => OutboundOrderIdV3::LocalFinalDemand(id),
                        _ => return Err(MaterialWorldErrorV3::Wire),
                    };
                    let feasible_quantity = cursor.u64()?;
                    let handled_quantity = cursor.u64()?;
                    let needed_hours = cursor.u64()?;
                    let used_hours = cursor.u64()?;
                    if handled_quantity > feasible_quantity
                        || used_hours > needed_hours
                        || (feasible_quantity == 0) != (needed_hours == 0)
                        || (handled_quantity == 0) != (used_hours == 0)
                        || u128::from(feasible_quantity) * u128::from(used_hours)
                            != u128::from(handled_quantity) * u128::from(needed_hours)
                        || result
                            .handling
                            .last()
                            .is_some_and(|row| (row.site_id, row.order) >= (site_id, order))
                    {
                        return Err(MaterialWorldErrorV3::Wire);
                    }
                    result.handling.push(MerchantHandlingReceiptV3 {
                        site_id,
                        order,
                        feasible_quantity,
                        handled_quantity,
                        needed_hours,
                        used_hours,
                    });
                }
                8 => {
                    let row = LocalRetailFulfillmentReceiptV3 {
                        order_id: OrderIdV1::from_bytes(cursor.take()?),
                        retailer_site_id: SiteIdV1::from_bytes(cursor.take()?),
                        demand_principal_id: FinalDemandPrincipalIdV3::from_bytes(cursor.take()?),
                        good_id: GoodIdV1::from_bytes(cursor.take()?),
                        unit_id: UnitIdV1::from_bytes(cursor.take()?),
                        quantity: cursor.positive()?,
                    };
                    if result
                        .local_fulfillments
                        .last()
                        .is_some_and(|prior| prior.order_id >= row.order_id)
                    {
                        return Err(MaterialWorldErrorV3::Wire);
                    }
                    result.local_fulfillments.push(row);
                }
                9 => {
                    let row = LocalTransferReceiptV3 {
                        order_id: OrderIdV1::from_bytes(cursor.take()?),
                        supplier_site_id: SiteIdV1::from_bytes(cursor.take()?),
                        buyer_site_id: SiteIdV1::from_bytes(cursor.take()?),
                        good_id: GoodIdV1::from_bytes(cursor.take()?),
                        unit_id: UnitIdV1::from_bytes(cursor.take()?),
                        quantity: cursor.positive()?,
                    };
                    if row.supplier_site_id == row.buyer_site_id
                        || result
                            .local_transfers
                            .last()
                            .is_some_and(|prior| prior.order_id >= row.order_id)
                    {
                        return Err(MaterialWorldErrorV3::Wire);
                    }
                    result.local_transfers.push(row);
                }
                _ => return Err(MaterialWorldErrorV3::Wire),
            }
        }
    }
    if cursor.position != bytes.len() {
        return Err(MaterialWorldErrorV3::Wire);
    }
    Ok(result)
}
struct ReceiptCursorV4<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl ReceiptCursorV4<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], MaterialWorldErrorV3> {
        let end = self
            .position
            .checked_add(N)
            .ok_or(MaterialWorldErrorV3::Wire)?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or(MaterialWorldErrorV3::Wire)?;
        self.position = end;
        bytes.try_into().map_err(|_| MaterialWorldErrorV3::Wire)
    }
    fn u64(&mut self) -> Result<u64, MaterialWorldErrorV3> {
        Ok(u64::from_be_bytes(self.take()?))
    }
    fn positive(&mut self) -> Result<u64, MaterialWorldErrorV3> {
        let value = self.u64()?;
        if value == 0 {
            Err(MaterialWorldErrorV3::Wire)
        } else {
            Ok(value)
        }
    }
}

#[cfg(test)]
mod tests;
