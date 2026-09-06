//! Exact successor world-register ownership for the routed material circuit.
//!
//! An active register contains the complete V2 opening state, never an economic
//! summary or a second inventory ledger. The graph-only digest stays unchanged.

use babylon_kernel::sha256_of;
use babylon_material_circuit::{
    advance_material_circuit_v2, decode_material_circuit_state_v2,
    encode_material_circuit_state_v2, MaterialCircuitErrorV2, MaterialCircuitStateV2,
    MaterialCircuitTransitionV2,
};

const REGISTER_DOMAIN: &[u8] = b"babylon.material-world-register.v2\0";
const NOMINAL_DOMAIN: &[u8] = b"babylon.nominal-material-world.v2\0";
const RECEIPT_DOMAIN: &[u8] = b"babylon.material-tick-receipts.v3\0";
/// Shared identity ceiling inherited by the aggregate replay envelope.
pub const MAX_MATERIAL_WORLD_REGISTER_BYTES_V2: usize = 67_108_864;

/// One checked complete material register at a completed weekly boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialWorldRegisterV2 {
    completed_tick: u64,
    state: MaterialCircuitStateV2,
    canonical_bytes: Vec<u8>,
    digest: [u8; 32],
}

/// Failure never publishes any part of a proposed material transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialWorldErrorV2 {
    Circuit(MaterialCircuitErrorV2),
    WeekMismatch,
    Arithmetic,
    ByteLimit,
    Allocation,
    Wire,
}
impl std::fmt::Display for MaterialWorldErrorV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "material world refused: {self:?}")
    }
}
impl std::error::Error for MaterialWorldErrorV2 {}
impl From<MaterialCircuitErrorV2> for MaterialWorldErrorV2 {
    fn from(error: MaterialCircuitErrorV2) -> Self {
        Self::Circuit(error)
    }
}

impl MaterialWorldRegisterV2 {
    /// Own and validate the full opening state for the next weekly interval.
    /// # Errors
    /// Refuses invalid circuit state, week mismatch, overflow or aggregate byte bound.
    pub fn try_new(
        completed_tick: u64,
        state: MaterialCircuitStateV2,
    ) -> Result<Self, MaterialWorldErrorV2> {
        if completed_tick
            .checked_add(1)
            .ok_or(MaterialWorldErrorV2::Arithmetic)?
            != state.week
        {
            return Err(MaterialWorldErrorV2::WeekMismatch);
        }
        let state_bytes = encode_material_circuit_state_v2(&state)?;
        let state = decode_material_circuit_state_v2(&state_bytes)?;
        let length = REGISTER_DOMAIN
            .len()
            .checked_add(20)
            .and_then(|count| count.checked_add(state_bytes.len()))
            .ok_or(MaterialWorldErrorV2::Arithmetic)?;
        let mut bytes = bounded_bytes(length)?;
        bytes.extend_from_slice(REGISTER_DOMAIN);
        bytes.extend_from_slice(&2_u32.to_be_bytes());
        bytes.extend_from_slice(&completed_tick.to_be_bytes());
        bytes.extend_from_slice(
            &u64::try_from(state_bytes.len())
                .map_err(|_| MaterialWorldErrorV2::Arithmetic)?
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
    pub fn state(&self) -> &MaterialCircuitStateV2 {
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
    pub fn decode(bytes: &[u8]) -> Result<Self, MaterialWorldErrorV2> {
        let header = REGISTER_DOMAIN.len() + 20;
        if bytes.len() < header
            || bytes.len() > MAX_MATERIAL_WORLD_REGISTER_BYTES_V2
            || !bytes.starts_with(REGISTER_DOMAIN)
        {
            return Err(MaterialWorldErrorV2::Wire);
        }
        let start = REGISTER_DOMAIN.len();
        if bytes[start..start + 4] != 2_u32.to_be_bytes() {
            return Err(MaterialWorldErrorV2::Wire);
        }
        let tick = u64::from_be_bytes(
            bytes[start + 4..start + 12]
                .try_into()
                .map_err(|_| MaterialWorldErrorV2::Wire)?,
        );
        let length = usize::try_from(u64::from_be_bytes(
            bytes[start + 12..header]
                .try_into()
                .map_err(|_| MaterialWorldErrorV2::Wire)?,
        ))
        .map_err(|_| MaterialWorldErrorV2::ByteLimit)?;
        if header.checked_add(length) != Some(bytes.len()) {
            return Err(MaterialWorldErrorV2::Wire);
        }
        let register = Self::try_new(tick, decode_material_circuit_state_v2(&bytes[header..])?)?;
        if register.canonical_bytes != bytes {
            return Err(MaterialWorldErrorV2::Wire);
        }
        Ok(register)
    }

    /// Prepare one next-week successor without mutating this register.
    /// The existing material transition executes arrivals, prior commitments,
    /// dispatch and following-week commitments in its governed order.
    /// # Errors
    /// Refuses any circuit or receipt encoding failure without changing this owner.
    pub fn prepare_next(&self) -> Result<PreparedMaterialWorldV3, MaterialWorldErrorV2> {
        let transition = advance_material_circuit_v2(&self.state)?;
        let receipts = encode_material_receipts_v3(self.state.week, &transition)?;
        let next = self
            .completed_tick
            .checked_add(1)
            .ok_or(MaterialWorldErrorV2::Arithmetic)?;
        let register = Self::try_new(next, transition.state)?;
        Ok(PreparedMaterialWorldV3 {
            prior_digest: self.digest,
            register,
            receipt_bytes: receipts,
        })
    }
}

/// Detached exact successor and its immutable material event evidence.
#[derive(Debug, PartialEq, Eq)]
pub struct PreparedMaterialWorldV3 {
    prior_digest: [u8; 32],
    register: MaterialWorldRegisterV2,
    receipt_bytes: Vec<u8>,
}
impl PreparedMaterialWorldV3 {
    #[must_use]
    pub const fn prior_digest(&self) -> [u8; 32] {
        self.prior_digest
    }
    #[must_use]
    pub const fn register(&self) -> &MaterialWorldRegisterV2 {
        &self.register
    }
    #[must_use]
    pub fn receipt_bytes(&self) -> &[u8] {
        &self.receipt_bytes
    }
    #[must_use]
    pub fn into_register(self) -> MaterialWorldRegisterV2 {
        self.register
    }
}

/// Successor nominal identity, binding both the graph world's identity and material state.
#[must_use]
pub fn nominal_material_world_hash_v2(
    graph_world_hash: [u8; 32],
    register: &MaterialWorldRegisterV2,
) -> [u8; 32] {
    let mut bytes = [0_u8; NOMINAL_DOMAIN.len() + 68];
    let domain = NOMINAL_DOMAIN.len();
    bytes[..domain].copy_from_slice(NOMINAL_DOMAIN);
    bytes[domain..domain + 4].copy_from_slice(&2_u32.to_be_bytes());
    bytes[domain + 4..domain + 36].copy_from_slice(&graph_world_hash);
    bytes[domain + 36..domain + 68].copy_from_slice(&register.digest);
    sha256_of(&bytes[..domain + 68])
}

fn bounded_bytes(length: usize) -> Result<Vec<u8>, MaterialWorldErrorV2> {
    if length > MAX_MATERIAL_WORLD_REGISTER_BYTES_V2 {
        return Err(MaterialWorldErrorV2::ByteLimit);
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| MaterialWorldErrorV2::Allocation)?;
    Ok(bytes)
}
fn encode_material_receipts_v3(
    tick: u64,
    transition: &MaterialCircuitTransitionV2,
) -> Result<Vec<u8>, MaterialWorldErrorV2> {
    let families = [
        (transition.production.len(), 80_usize),
        (transition.dispatches.len(), 112),
        (transition.losses.len(), 104),
        (transition.arrivals.len(), 40),
        (transition.deliveries.len(), 40),
        (transition.realizations.len(), 40),
    ];
    let length = families.iter().try_fold(
        RECEIPT_DOMAIN.len() + 12 + 6 * 9,
        |total, (count, width)| {
            total
                .checked_add(
                    count
                        .checked_mul(*width)
                        .ok_or(MaterialWorldErrorV2::Arithmetic)?,
                )
                .ok_or(MaterialWorldErrorV2::Arithmetic)
        },
    )?;
    let mut bytes = bounded_bytes(length)?;
    bytes.extend_from_slice(RECEIPT_DOMAIN);
    bytes.extend_from_slice(&3_u32.to_be_bytes());
    bytes.extend_from_slice(&tick.to_be_bytes());
    for (tag, (count, _)) in families.iter().enumerate() {
        bytes.push(u8::try_from(tag + 1).map_err(|_| MaterialWorldErrorV2::Arithmetic)?);
        bytes.extend_from_slice(
            &u64::try_from(*count)
                .map_err(|_| MaterialWorldErrorV2::Arithmetic)?
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
                    bytes.extend_from_slice(&row.final_arrival_week.to_be_bytes());
                }
            }
            2 => {
                for row in &transition.losses {
                    bytes.extend_from_slice(&row.lot_id.as_bytes());
                    bytes.extend_from_slice(&row.order_id.as_bytes());
                    bytes.extend_from_slice(&row.corridor_id.as_bytes());
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
            _ => unreachable!("the six material receipt families are closed"),
        }
    }
    debug_assert_eq!(bytes.len(), length);
    Ok(bytes)
}

/// Typed material evidence decoded only from an exact committed V3 receipt family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialTickReceiptsV3 {
    pub resolve_tick: u64,
    pub production: Vec<babylon_material_circuit::ProductionReceiptV1>,
    pub dispatches: Vec<babylon_material_circuit::RoutedDispatchReceiptV2>,
    pub losses: Vec<babylon_material_circuit::FreightLossReceiptV2>,
    pub arrivals: Vec<babylon_material_circuit::ArrivalReceiptV1>,
    pub deliveries: Vec<babylon_material_circuit::DeliveryReceiptV1>,
    pub realizations: Vec<babylon_material_circuit::RealizationReceiptV1>,
}
/// Decode a bounded receipt family. Hash/campaign binding belongs to its V3 envelope.
/// # Errors
/// Refuses versions, tags, counts, truncation, trailing bytes and invalid quantity relations.
pub fn decode_material_receipts_v3(
    bytes: &[u8],
) -> Result<MaterialTickReceiptsV3, MaterialWorldErrorV2> {
    use babylon_material_circuit::*;
    if bytes.len() > MAX_MATERIAL_WORLD_REGISTER_BYTES_V2 || !bytes.starts_with(RECEIPT_DOMAIN) {
        return Err(MaterialWorldErrorV2::Wire);
    }
    let mut cursor = ReceiptCursorV3 {
        bytes,
        position: RECEIPT_DOMAIN.len(),
    };
    if cursor.take::<4>()? != 3_u32.to_be_bytes() {
        return Err(MaterialWorldErrorV2::Wire);
    }
    let resolve_tick = cursor.u64()?;
    if resolve_tick == 0 {
        return Err(MaterialWorldErrorV2::Wire);
    }
    let mut result = MaterialTickReceiptsV3 {
        resolve_tick,
        production: Vec::new(),
        dispatches: Vec::new(),
        losses: Vec::new(),
        arrivals: Vec::new(),
        deliveries: Vec::new(),
        realizations: Vec::new(),
    };
    for tag in 1..=6 {
        if cursor.take::<1>()? != [tag] {
            return Err(MaterialWorldErrorV2::Wire);
        }
        let count = usize::try_from(cursor.u64()?).map_err(|_| MaterialWorldErrorV2::ByteLimit)?;
        if count > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
            return Err(MaterialWorldErrorV2::ByteLimit);
        }
        let width = [80_usize, 112, 104, 40, 40, 40][usize::from(tag - 1)];
        if count
            .checked_mul(width)
            .is_none_or(|length| length > bytes.len() - cursor.position)
        {
            return Err(MaterialWorldErrorV2::Wire);
        }
        match tag {
            1 => result.production.try_reserve_exact(count),
            2 => result.dispatches.try_reserve_exact(count),
            3 => result.losses.try_reserve_exact(count),
            4 => result.arrivals.try_reserve_exact(count),
            5 => result.deliveries.try_reserve_exact(count),
            6 => result.realizations.try_reserve_exact(count),
            _ => return Err(MaterialWorldErrorV2::Wire),
        }
        .map_err(|_| MaterialWorldErrorV2::Allocation)?;
        for _ in 0..count {
            match tag {
                1 => {
                    let process_id = ProcessIdV1::from_bytes(cursor.take()?);
                    let site_id = SiteIdV1::from_bytes(cursor.take()?);
                    let planned_batches = cursor.u64()?;
                    let produced_batches = cursor.u64()?;
                    if produced_batches > planned_batches {
                        return Err(MaterialWorldErrorV2::Wire);
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
                    let final_arrival_week = cursor.u64()?;
                    if final_arrival_week <= resolve_tick {
                        return Err(MaterialWorldErrorV2::Wire);
                    }
                    result.dispatches.push(RoutedDispatchReceiptV2 {
                        lot_id,
                        order_id,
                        route_id,
                        quantity,
                        final_arrival_week,
                    });
                }
                3 => {
                    let lot_id = FreightLotIdV2::from_bytes(cursor.take()?);
                    let order_id = OrderIdV1::from_bytes(cursor.take()?);
                    let corridor_id = CorridorIdV2::from_bytes(cursor.take()?);
                    let quantity = cursor.positive()?;
                    result.losses.push(FreightLossReceiptV2 {
                        lot_id,
                        order_id,
                        corridor_id,
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
                _ => return Err(MaterialWorldErrorV2::Wire),
            }
        }
    }
    if cursor.position != bytes.len() {
        return Err(MaterialWorldErrorV2::Wire);
    }
    Ok(result)
}
struct ReceiptCursorV3<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl ReceiptCursorV3<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], MaterialWorldErrorV2> {
        let end = self
            .position
            .checked_add(N)
            .ok_or(MaterialWorldErrorV2::Wire)?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or(MaterialWorldErrorV2::Wire)?;
        self.position = end;
        bytes.try_into().map_err(|_| MaterialWorldErrorV2::Wire)
    }
    fn u64(&mut self) -> Result<u64, MaterialWorldErrorV2> {
        Ok(u64::from_be_bytes(self.take()?))
    }
    fn positive(&mut self) -> Result<u64, MaterialWorldErrorV2> {
        let value = self.u64()?;
        if value == 0 {
            Err(MaterialWorldErrorV2::Wire)
        } else {
            Ok(value)
        }
    }
}
