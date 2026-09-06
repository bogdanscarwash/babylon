//! Exact conserved production, inventory, order, and realization transitions.
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

mod model;
mod model_v2;
mod staffing;
mod transition;
mod transition_v2;
mod wire;
mod wire_common;
mod wire_v2;

pub use model::*;
pub use model_v2::*;
pub use staffing::{
    advance_staffing_v1, StaffingErrorV1, StaffingPolicyV1, StaffingPoolBindingV1,
    StaffingPoolIdV1, StaffingPoolStateV1, StaffingReceiptV1, StaffingStateV1,
    StaffingTransitionV1, StaffingWorkRequestV1,
};
pub use transition::advance_material_circuit_v1;
pub use transition_v2::{
    advance_material_circuit_v2, close_material_week_v2, ClosedMaterialWeekV2,
};
pub use wire::{
    decode_material_circuit_state_v1, encode_material_circuit_state_v1,
    material_circuit_state_v1_digest, MATERIAL_CIRCUIT_STATE_V1_DOMAIN_BYTES,
};
pub use wire_v2::{
    decode_material_circuit_state_v2, encode_material_circuit_state_v2,
    material_circuit_state_v2_digest, MATERIAL_CIRCUIT_STATE_V2_DOMAIN_BYTES,
    MATERIAL_CIRCUIT_V2_SOURCE_SHA256,
};
