//! Pure conserved staffing with one preceding unretained work request.
//!
//! Requests are explicit exact labor-time, not inferred headcount or wages.
//! This module does not derive requests, allocate production, or publish a tick.

mod model;
mod transition;

pub use model::{
    StaffingError, StaffingPolicy, StaffingPoolBinding, StaffingPoolId, StaffingPoolState,
    StaffingReceipt, StaffingState, StaffingTransition, StaffingWorkRequest, StaffingWorkSource,
};
pub use transition::advance_staffing;

#[cfg(test)]
mod tests;
