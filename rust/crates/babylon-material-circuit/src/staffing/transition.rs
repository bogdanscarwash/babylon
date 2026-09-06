//! Complete request admission and paired employment/reserve transfers.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    StaffingErrorV1, StaffingPoolStateV1, StaffingReceiptV1, StaffingStateV1, StaffingTransitionV1,
    StaffingWorkRequestV1,
};
use crate::{LaborCapacityRowV1, MAX_MATERIAL_CIRCUIT_ROWS_V1};

fn reserved_vec<T>(count: usize) -> Result<Vec<T>, StaffingErrorV1> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(count)
        .map_err(|_| StaffingErrorV1::Allocation)?;
    Ok(rows)
}

fn pool_requests(
    opening: &StaffingStateV1,
    requests: &[StaffingWorkRequestV1],
) -> Result<Vec<u64>, StaffingErrorV1> {
    if requests.len() > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
        return Err(StaffingErrorV1::RowLimit);
    }
    let mut owners = BTreeMap::new();
    for (index, pool) in opening.pools().iter().enumerate() {
        for process in pool.binding().processes() {
            owners.insert(*process, (index, pool.binding()));
        }
    }
    let mut totals = reserved_vec(opening.pools().len())?;
    totals.resize(opening.pools().len(), 0_u64);
    let mut seen = BTreeSet::new();
    for request in requests {
        if request.period() != opening.period() {
            return Err(StaffingErrorV1::PeriodInvariant);
        }
        let (index, binding) = owners
            .get(&request.process_id())
            .ok_or(StaffingErrorV1::UnknownRequest)?;
        if request.pool_id() != binding.pool_id()
            || request.site_id() != binding.site_id()
            || request.unit_id() != binding.unit_id()
        {
            return Err(StaffingErrorV1::RequestBinding);
        }
        if !seen.insert(request.process_id()) {
            return Err(StaffingErrorV1::DuplicateRequest);
        }
        totals[*index] = totals[*index]
            .checked_add(request.hours())
            .ok_or(StaffingErrorV1::Arithmetic)?;
    }
    if seen.len() != owners.len() {
        return Err(StaffingErrorV1::MissingRequest);
    }
    Ok(totals)
}

fn advance_pool(
    opening: &StaffingPoolStateV1,
    current_request: u64,
    period: u64,
    next_period: u64,
) -> Result<(StaffingPoolStateV1, StaffingReceiptV1, LaborCapacityRowV1), StaffingErrorV1> {
    let binding = opening.binding();
    let schedule = binding.policy().hours_per_person();
    let retained = current_request.max(opening.previous_unretained_hours());
    let requested_people = (retained / schedule)
        .checked_add(u64::from(!retained.is_multiple_of(schedule)))
        .ok_or(StaffingErrorV1::Arithmetic)?;
    let target = requested_people.min(binding.labor_force());
    let (employed, reserve) = if target >= opening.employed() {
        let hires = target - opening.employed();
        (
            opening
                .employed()
                .checked_add(hires)
                .ok_or(StaffingErrorV1::Arithmetic)?,
            opening
                .reserve()
                .checked_sub(hires)
                .ok_or(StaffingErrorV1::Arithmetic)?,
        )
    } else {
        let separations = opening.employed() - target;
        (
            opening
                .employed()
                .checked_sub(separations)
                .ok_or(StaffingErrorV1::Arithmetic)?,
            opening
                .reserve()
                .checked_add(separations)
                .ok_or(StaffingErrorV1::Arithmetic)?,
        )
    };
    let hours = employed
        .checked_mul(schedule)
        .ok_or(StaffingErrorV1::Arithmetic)?;
    let closing =
        StaffingPoolStateV1::try_new(binding.clone(), employed, reserve, current_request)?;
    let receipt = StaffingReceiptV1::from_transition(
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

/// Resolve one period from exact, labor-unconstrained requests for every process.
///
/// A zero request must be explicit. One prior unretained request supplies one
/// period of retention; the retained maximum is never stored as new memory.
/// The caller remains responsible for deriving material-feasible requests and
/// installing these next-period budgets before planning production.
///
/// # Errors
/// Refuses incomplete/foreign/duplicate requests, bounds or checked arithmetic.
/// Every error leaves the opening state unchanged and returns no partial result.
pub fn advance_staffing_v1(
    opening: &StaffingStateV1,
    requests: &[StaffingWorkRequestV1],
) -> Result<StaffingTransitionV1, StaffingErrorV1> {
    let next_period = opening
        .period()
        .checked_add(1)
        .ok_or(StaffingErrorV1::Arithmetic)?;
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
    Ok(StaffingTransitionV1::new(
        StaffingStateV1::try_new(next_period, pools)?,
        receipts,
        labor,
    ))
}
