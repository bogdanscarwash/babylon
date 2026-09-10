//! Complete request admission and paired employment/reserve transfers.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    StaffingErrorV2, StaffingPoolStateV2, StaffingReceiptV2, StaffingStateV2, StaffingTransitionV2,
    StaffingWorkRequestV2,
};
use crate::{LaborCapacityRowV1, MAX_MATERIAL_CIRCUIT_ROWS_V1};

fn reserved_vec<T>(count: usize) -> Result<Vec<T>, StaffingErrorV2> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(count)
        .map_err(|_| StaffingErrorV2::Allocation)?;
    Ok(rows)
}

fn pool_requests(
    opening: &StaffingStateV2,
    requests: &[StaffingWorkRequestV2],
) -> Result<Vec<u64>, StaffingErrorV2> {
    if requests.len() > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        return Err(StaffingErrorV2::RowLimit);
    }
    let mut owners = BTreeMap::new();
    for (index, pool) in opening.pools().iter().enumerate() {
        for source in pool.binding().work_sources() {
            owners.insert(*source, (index, pool.binding()));
        }
    }
    let mut totals = reserved_vec(opening.pools().len())?;
    totals.resize(opening.pools().len(), 0_u64);
    let mut seen = BTreeSet::new();
    for request in requests {
        if request.period() != opening.period() {
            return Err(StaffingErrorV2::PeriodInvariant);
        }
        let (index, binding) = owners
            .get(&request.work_source())
            .ok_or(StaffingErrorV2::UnknownRequest)?;
        if request.pool_id() != binding.pool_id()
            || request.site_id() != binding.site_id()
            || request.unit_id() != binding.unit_id()
        {
            return Err(StaffingErrorV2::RequestBinding);
        }
        if !seen.insert(request.work_source()) {
            return Err(StaffingErrorV2::DuplicateRequest);
        }
        totals[*index] = totals[*index]
            .checked_add(request.hours())
            .ok_or(StaffingErrorV2::Arithmetic)?;
    }
    if seen.len() != owners.len() {
        return Err(StaffingErrorV2::MissingRequest);
    }
    Ok(totals)
}

fn advance_pool(
    opening: &StaffingPoolStateV2,
    current_request: u64,
    period: u64,
    next_period: u64,
) -> Result<(StaffingPoolStateV2, StaffingReceiptV2, LaborCapacityRowV1), StaffingErrorV2> {
    let binding = opening.binding();
    let schedule = binding.policy().hours_per_person();
    let retained = current_request.max(opening.previous_unretained_hours());
    let requested_people = (retained / schedule)
        .checked_add(u64::from(!retained.is_multiple_of(schedule)))
        .ok_or(StaffingErrorV2::Arithmetic)?;
    let target = requested_people.min(binding.labor_force());
    let (employed, reserve) = if target >= opening.employed() {
        let hires = target - opening.employed();
        (
            opening
                .employed()
                .checked_add(hires)
                .ok_or(StaffingErrorV2::Arithmetic)?,
            opening
                .reserve()
                .checked_sub(hires)
                .ok_or(StaffingErrorV2::Arithmetic)?,
        )
    } else {
        let separations = opening.employed() - target;
        (
            opening
                .employed()
                .checked_sub(separations)
                .ok_or(StaffingErrorV2::Arithmetic)?,
            opening
                .reserve()
                .checked_add(separations)
                .ok_or(StaffingErrorV2::Arithmetic)?,
        )
    };
    let hours = employed
        .checked_mul(schedule)
        .ok_or(StaffingErrorV2::Arithmetic)?;
    let closing =
        StaffingPoolStateV2::try_new(binding.clone(), employed, reserve, current_request)?;
    let receipt = StaffingReceiptV2::from_transition(
        period,
        opening,
        &closing,
        current_request,
        retained,
        target,
        hours,
    );
    let labor = LaborCapacityRowV1 {
        site_id: binding.site_id(),
        unit_id: binding.unit_id(),
        period: next_period,
        available: hours,
    };
    Ok((closing, receipt, labor))
}

/// Resolve one period from exact labor-unconstrained requests for every work source.
///
/// A zero request must be explicit. One prior unretained request supplies one
/// period of retention; the retained maximum is never stored as new memory.
/// The caller remains responsible for deriving material-feasible requests and
/// installing these next-period budgets before planning production.
///
/// # Errors
/// Refuses incomplete/foreign/duplicate requests, bounds or checked arithmetic.
/// Every error leaves the opening state unchanged and returns no partial result.
pub fn advance_staffing_v2(
    opening: &StaffingStateV2,
    requests: &[StaffingWorkRequestV2],
) -> Result<StaffingTransitionV2, StaffingErrorV2> {
    let next_period = opening
        .period()
        .checked_add(1)
        .ok_or(StaffingErrorV2::Arithmetic)?;
    let requests = pool_requests(opening, requests)?;
    let count = opening.pools().len();
    let mut pools = reserved_vec(count)?;
    let mut receipts = reserved_vec(count)?;
    let mut labor = reserved_vec(count)?;
    for (opening_pool, request) in opening.pools().iter().zip(requests) {
        let (pool, receipt, capacity) =
            advance_pool(opening_pool, request, opening.period(), next_period)?;
        pools.push(pool);
        receipts.push(receipt);
        labor.push(capacity);
    }
    labor.sort_unstable_by_key(|row| (row.site_id, row.unit_id));
    Ok(StaffingTransitionV2::new(
        StaffingStateV2::try_new(next_period, pools)?,
        receipts,
        labor,
    ))
}
