//! Pure conserved staffing with one preceding unretained work request.
//!
//! Requests are explicit exact labor-time, not inferred headcount or wages.
//! This module does not derive requests, allocate production, or publish a tick.

mod model;
mod transition;

pub use model::{
    StaffingErrorV2, StaffingPolicyV1, StaffingPoolBindingV2, StaffingPoolIdV1,
    StaffingPoolStateV2, StaffingReceiptV2, StaffingStateV2, StaffingTransitionV2,
    StaffingWorkRequestV2, StaffingWorkSourceV2,
};
pub use transition::advance_staffing_v2;

#[cfg(test)]
mod tests;
