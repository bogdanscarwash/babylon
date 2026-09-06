//! Pure conserved staffing with one preceding unretained work request.
//!
//! Requests are explicit exact labor-time, not inferred headcount or wages.
//! This module does not derive requests, allocate production, or publish a tick.

mod model;
mod transition;

pub use model::{
    StaffingErrorV1, StaffingPolicyV1, StaffingPoolBindingV1, StaffingPoolIdV1,
    StaffingPoolStateV1, StaffingReceiptV1, StaffingStateV1, StaffingTransitionV1,
    StaffingWorkRequestV1,
};
pub use transition::advance_staffing_v1;

#[cfg(test)]
mod tests;
