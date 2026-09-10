//! Exact conserved production, inventory, order, and realization transitions.
//!
//! One transition closes one four-week period. Period ordinals advance by one;
//! content supplies capacities and labor schedules for the entire interval.
//! Inventories, people, order principals and per-batch recipes retain their units.
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

mod model;
mod model_v2;
mod model_v3;
mod staffing;
mod transition;
mod transition_v3;
mod wire_common;
mod wire_v3;

pub use model::*;
pub use model_v2::*;
pub use staffing::{
    advance_staffing_v2, StaffingErrorV2, StaffingPolicyV1, StaffingPoolBindingV2,
    StaffingPoolIdV1, StaffingPoolStateV2, StaffingReceiptV2, StaffingStateV2,
    StaffingTransitionV2, StaffingWorkRequestV2, StaffingWorkSourceV2,
};

pub use model_v3::*;
pub use transition_v3::{
    advance_material_circuit_v3, close_material_period_v3, ClosedMaterialPeriodV3,
};
pub use wire_v3::{
    decode_material_circuit_state_v3, encode_material_circuit_state_v3,
    material_circuit_state_v3_digest, MATERIAL_CIRCUIT_STATE_V3_DOMAIN_BYTES,
    MATERIAL_CIRCUIT_V3_SOURCE_SHA256,
};
