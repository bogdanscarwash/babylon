//! Exact conserved production, inventory, order, and realization transitions.
//!
//! One transition closes one four-week period. Period ordinals advance by one;
//! content supplies capacities and labor schedules for the entire interval.
//! Inventories, people, order principals and per-batch recipes retain their units.
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

mod inventory;
mod model;
mod production;
mod staffing;
mod transition;
mod wire;

pub use model::*;
pub use staffing::{
    advance_staffing_v2, StaffingErrorV2, StaffingPolicyV1, StaffingPoolBindingV2,
    StaffingPoolIdV1, StaffingPoolStateV2, StaffingReceiptV2, StaffingStateV2,
    StaffingTransitionV2, StaffingWorkRequestV2, StaffingWorkSourceV2,
};

pub use transition::{
    advance_material_circuit_v3, close_material_period_v3, ClosedMaterialPeriodV3,
};
pub use wire::{
    decode_material_circuit_state_v3, encode_material_circuit_state_v3,
    material_circuit_state_v3_digest, MATERIAL_CIRCUIT_STATE_V3_DOMAIN_BYTES,
    MATERIAL_CIRCUIT_V3_SOURCE_SHA256,
};
