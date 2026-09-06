use super::{
    advance_staffing_v1, StaffingErrorV1, StaffingPolicyV1, StaffingPoolBindingV1,
    StaffingPoolIdV1, StaffingPoolStateV1, StaffingReceiptV1, StaffingStateV1,
    StaffingWorkRequestV1,
};
use crate::{ProcessIdV1, SiteIdV1, UnitIdV1, MAX_MATERIAL_CIRCUIT_ROWS_V1};

fn binding(key: u8, labor_force: u64, schedule: u64, process_keys: &[u8]) -> StaffingPoolBindingV1 {
    StaffingPoolBindingV1::try_new(
        StaffingPoolIdV1::from_bytes([key; 32]),
        SiteIdV1::from_bytes([key; 32]),
        UnitIdV1::from_bytes([9; 32]),
        labor_force,
        StaffingPolicyV1::one_period(schedule).expect("explicit positive schedule"),
        process_keys
            .iter()
            .map(|key| ProcessIdV1::from_bytes([*key; 32]))
            .collect(),
    )
    .expect("explicit disjoint binding")
}

fn pool(binding: StaffingPoolBindingV1, employed: u64, previous: u64) -> StaffingPoolStateV1 {
    let reserve = binding.labor_force() - employed;
    StaffingPoolStateV1::try_new(binding, employed, reserve, previous).expect("conserved pool")
}

fn request(binding: &StaffingPoolBindingV1, process: u8, hours: u64) -> StaffingWorkRequestV1 {
    request_at(1, binding, process, hours)
}

fn request_at(
    period: u64,
    binding: &StaffingPoolBindingV1,
    process: u8,
    hours: u64,
) -> StaffingWorkRequestV1 {
    StaffingWorkRequestV1::new(
        period,
        binding.pool_id(),
        ProcessIdV1::from_bytes([process; 32]),
        binding.site_id(),
        binding.unit_id(),
        hours,
    )
}

fn state(pools: Vec<StaffingPoolStateV1>) -> StaffingStateV1 {
    StaffingStateV1::try_new(1, pools).expect("complete opening state")
}

fn conservation(receipt: &StaffingReceiptV1) {
    let labor_force = u128::from(receipt.binding().labor_force());
    assert_eq!(
        u128::from(receipt.opening_employed()) + u128::from(receipt.opening_reserve()),
        labor_force
    );
    assert_eq!(
        u128::from(receipt.closing_employed()) + u128::from(receipt.closing_reserve()),
        labor_force
    );
    assert_eq!(
        u128::from(receipt.opening_employed()) + u128::from(receipt.hires()),
        u128::from(receipt.closing_employed()) + u128::from(receipt.separations()),
    );
    assert_eq!(
        u128::from(receipt.opening_reserve()) + u128::from(receipt.separations()),
        u128::from(receipt.closing_reserve()) + u128::from(receipt.hires()),
    );
    assert!(receipt.hires() == 0 || receipt.separations() == 0);
    assert_eq!(
        u128::from(receipt.next_opening_hours()),
        u128::from(receipt.closing_employed())
            * u128::from(receipt.binding().policy().hours_per_person()),
    );
}

#[test]
fn one_empty_period_is_retained_but_a_second_releases_then_recovery_rehires() {
    let owned = binding(1, 7, 3, &[1]);
    let opening = state(vec![pool(owned.clone(), 5, 15)]);
    let brief = advance_staffing_v1(&opening, &[request(&owned, 1, 0)]).expect("brief shortage");
    assert_eq!(brief.state().pools()[0].employed(), 5);
    assert_eq!(brief.state().pools()[0].previous_unretained_hours(), 0);
    assert_eq!(brief.receipts()[0].retained_hours(), 15);
    assert_eq!(brief.receipts()[0].separations(), 0);
    assert_eq!(brief.next_labor()[0].period, 2);
    assert_eq!(brief.next_labor()[0].available, 15);
    conservation(&brief.receipts()[0]);

    let sustained = advance_staffing_v1(brief.state(), &[request_at(2, &owned, 1, 0)])
        .expect("sustained shortage");
    assert_eq!(sustained.state().pools()[0].employed(), 0);
    assert_eq!(sustained.state().pools()[0].reserve(), 7);
    assert_eq!(sustained.receipts()[0].retained_hours(), 0);
    assert_eq!(sustained.receipts()[0].separations(), 5);
    assert_eq!(sustained.next_labor()[0].available, 0);
    conservation(&sustained.receipts()[0]);

    let recovered = advance_staffing_v1(sustained.state(), &[request_at(3, &owned, 1, 10)])
        .expect("rehire independently of prior labor cap");
    assert_eq!(recovered.state().pools()[0].employed(), 4);
    assert_eq!(recovered.state().pools()[0].reserve(), 3);
    assert_eq!(recovered.receipts()[0].hires(), 4);
    assert_eq!(recovered.next_labor()[0].available, 12);
    assert_eq!(recovered.next_labor()[0].period, 4);
    assert_eq!(opening.pools()[0].employed(), 5, "original owner untouched");
    conservation(&recovered.receipts()[0]);
}

#[test]
fn forty_hour_period_pools_before_rounding_and_remembers_only_current_work() {
    let owned = binding(1, 4, 40, &[1, 2]);
    let opening = state(vec![pool(owned.clone(), 3, 120)]);
    let first = advance_staffing_v1(&opening, &[request(&owned, 1, 20), request(&owned, 2, 20)])
        .expect("one period of retained staffing under the explicit forty-hour schedule");
    assert_eq!(first.receipts()[0].current_unretained_hours(), 40);
    assert_eq!(first.receipts()[0].retained_hours(), 120);
    assert_eq!(first.state().pools()[0].previous_unretained_hours(), 40);
    assert_eq!(first.state().pools()[0].employed(), 3);
    assert_eq!(first.next_labor()[0].available, 120);

    let second = advance_staffing_v1(
        first.state(),
        &[request_at(2, &owned, 1, 20), request_at(2, &owned, 2, 20)],
    )
    .expect("only the preceding unretained forty hours remain");
    assert_eq!(second.receipts()[0].retained_hours(), 40);
    assert_eq!(second.receipts()[0].separations(), 2);
    assert_eq!(second.state().pools()[0].employed(), 1);
    assert_eq!(second.state().pools()[0].reserve(), 3);
    assert_eq!(second.next_labor()[0].available, 40);

    let third = advance_staffing_v1(
        second.state(),
        &[request_at(3, &owned, 1, 20), request_at(3, &owned, 2, 21)],
    )
    .expect("forty-one pooled hours require two people for the following period");
    assert_eq!(third.receipts()[0].current_unretained_hours(), 41);
    assert_eq!(third.receipts()[0].retained_hours(), 41);
    assert_eq!(third.state().pools()[0].previous_unretained_hours(), 41);
    assert_eq!(third.receipts()[0].hires(), 1);
    assert_eq!(third.state().pools()[0].employed(), 2);
    assert_eq!(third.state().pools()[0].reserve(), 2);
    assert_eq!(third.next_labor()[0].available, 80);
    assert_eq!(third.next_labor()[0].period, 4);
    for receipt in [
        &first.receipts()[0],
        &second.receipts()[0],
        &third.receipts()[0],
    ] {
        conservation(receipt);
    }
}

#[test]
fn declining_request_does_not_retain_a_historical_peak_forever() {
    let owned = binding(1, 10, 2, &[1]);
    let opening = state(vec![pool(owned.clone(), 8, 16)]);
    let first = advance_staffing_v1(&opening, &[request(&owned, 1, 6)]).expect("first decline");
    let second =
        advance_staffing_v1(first.state(), &[request_at(2, &owned, 1, 6)]).expect("second decline");
    assert_eq!(first.state().pools()[0].employed(), 8);
    assert_eq!(first.state().pools()[0].previous_unretained_hours(), 6);
    assert_eq!(second.state().pools()[0].employed(), 3);
    assert_eq!(second.receipts()[0].separations(), 5);
    conservation(&second.receipts()[0]);
}

#[test]
fn shared_processes_pool_hours_before_rounding_and_leave_unrelated_work_unchanged() {
    let shared = binding(1, 10, 3, &[2, 1]);
    let food = binding(2, 5, 7, &[3]);
    let opening = state(vec![pool(food.clone(), 2, 14), pool(shared.clone(), 0, 0)]);
    let requests = [
        request(&shared, 2, 1),
        request(&food, 3, 14),
        request(&shared, 1, 1),
    ];
    let result = advance_staffing_v1(&opening, &requests).expect("pooled request");
    assert_eq!(result.receipts().len(), 2);
    let pooled = &result.receipts()[0];
    assert_eq!(pooled.current_unretained_hours(), 2);
    assert_eq!(
        pooled.closing_employed(),
        1,
        "ceil after pooling, not one person per process"
    );
    assert_eq!(
        result.next_labor().len(),
        2,
        "one budget per pool, not per process"
    );
    assert_eq!(result.next_labor()[0].available, 3);
    let unaffected = &result.receipts()[1];
    assert_eq!(
        (unaffected.closing_employed(), unaffected.closing_reserve()),
        (2, 3)
    );
    assert_eq!((unaffected.hires(), unaffected.separations()), (0, 0));
    assert_eq!(unaffected.next_opening_hours(), 14);
    conservation(pooled);
    conservation(unaffected);
    let mut reordered = requests;
    reordered.reverse();
    assert_eq!(advance_staffing_v1(&opening, &reordered), Ok(result));
}

#[test]
fn large_request_caps_at_available_people_without_ceiling_overflow() {
    let owned = binding(1, 2, 2, &[1]);
    let opening = state(vec![pool(owned.clone(), 0, 0)]);
    let result =
        advance_staffing_v1(&opening, &[request(&owned, 1, u64::MAX)]).expect("bounded pool");
    assert_eq!(result.receipts()[0].retained_hours(), u64::MAX);
    assert_eq!(result.receipts()[0].target_employed(), 2);
    assert_eq!(result.receipts()[0].next_opening_hours(), 4);
    conservation(&result.receipts()[0]);
}

#[test]
fn zero_population_and_empty_world_are_explicit_valid_states() {
    let owned = binding(1, 0, 3, &[1]);
    let empty_pool = state(vec![pool(owned.clone(), 0, 0)]);
    let result =
        advance_staffing_v1(&empty_pool, &[request(&owned, 1, 8)]).expect("no available people");
    assert_eq!(result.receipts()[0].hires(), 0);
    assert_eq!(result.receipts()[0].closing_reserve(), 0);
    assert_eq!(result.next_labor()[0].available, 0);
    conservation(&result.receipts()[0]);
    let empty_world = advance_staffing_v1(&state(vec![]), &[]).expect("declared empty state");
    assert!(empty_world.state().pools().is_empty());
    assert!(empty_world.receipts().is_empty());
    assert!(empty_world.next_labor().is_empty());
    assert_eq!(empty_world.state().period(), 2);
}

fn refusal(
    opening: &StaffingStateV1,
    requests: &[StaffingWorkRequestV1],
    expected: StaffingErrorV1,
) {
    let before = opening.clone();
    assert_eq!(advance_staffing_v1(opening, requests), Err(expected));
    assert_eq!(opening, &before);
}

#[test]
fn incomplete_duplicate_foreign_and_cross_unit_requests_refuse() {
    let owned = binding(1, 10, 2, &[1, 2]);
    let opening = state(vec![pool(owned.clone(), 3, 6)]);
    let first = request(&owned, 1, 0);
    let second = request(&owned, 2, 0);
    refusal(&opening, &[first], StaffingErrorV1::MissingRequest);
    refusal(
        &opening,
        &[first, second, first],
        StaffingErrorV1::DuplicateRequest,
    );
    refusal(
        &opening,
        &[first, request(&owned, 9, 0)],
        StaffingErrorV1::UnknownRequest,
    );
    let wrong_unit = StaffingWorkRequestV1::new(
        1,
        owned.pool_id(),
        second.process_id(),
        owned.site_id(),
        UnitIdV1::from_bytes([8; 32]),
        0,
    );
    refusal(
        &opening,
        &[first, wrong_unit],
        StaffingErrorV1::RequestBinding,
    );
    let wrong_site = StaffingWorkRequestV1::new(
        1,
        owned.pool_id(),
        second.process_id(),
        SiteIdV1::from_bytes([8; 32]),
        owned.unit_id(),
        0,
    );
    refusal(
        &opening,
        &[first, wrong_site],
        StaffingErrorV1::RequestBinding,
    );
    let wrong_pool = StaffingWorkRequestV1::new(
        1,
        StaffingPoolIdV1::from_bytes([8; 32]),
        second.process_id(),
        owned.site_id(),
        owned.unit_id(),
        0,
    );
    refusal(
        &opening,
        &[first, wrong_pool],
        StaffingErrorV1::RequestBinding,
    );
}

#[test]
fn overflow_at_request_sum_or_late_pool_publishes_no_partial_transition() {
    let shared = binding(1, 2, 1, &[1, 2]);
    let opening = state(vec![pool(shared.clone(), 0, 0)]);
    refusal(
        &opening,
        &[request(&shared, 1, u64::MAX), request(&shared, 2, 1)],
        StaffingErrorV1::Arithmetic,
    );

    let ordinary = binding(1, 3, 3, &[1]);
    let overflowing = binding(2, 2, u64::MAX / 2 + 1, &[2]);
    let mixed = state(vec![
        pool(ordinary.clone(), 0, 0),
        pool(overflowing.clone(), 0, 0),
    ]);
    // Pool 1 can prepare a hire; pool 2 then refuses its overflowing schedule.
    refusal(
        &mixed,
        &[request(&ordinary, 1, 3), request(&overflowing, 2, u64::MAX)],
        StaffingErrorV1::Arithmetic,
    );
    assert_eq!(
        StaffingPoolStateV1::try_new(overflowing, 2, 0, 0),
        Err(StaffingErrorV1::Arithmetic),
    );
}

#[test]
fn next_schedule_overflow_and_period_overflow_refuse_without_mutation() {
    let owned = binding(1, 2, u64::MAX / 2 + 1, &[1]);
    let opening = state(vec![pool(owned.clone(), 0, 0)]);
    refusal(
        &opening,
        &[request(&owned, 1, u64::MAX)],
        StaffingErrorV1::Arithmetic,
    );
    let last_period = StaffingStateV1::try_new(u64::MAX, vec![pool(owned.clone(), 0, 0)])
        .expect("representable current period");
    refusal(
        &last_period,
        &[request_at(u64::MAX, &owned, 1, 0)],
        StaffingErrorV1::Arithmetic,
    );
}

#[test]
fn missing_schedule_or_nonconserved_person_inputs_refuse() {
    assert_eq!(
        StaffingPolicyV1::one_period(0),
        Err(StaffingErrorV1::ZeroSchedule)
    );
    let owned = binding(1, 5, 2, &[1]);
    assert_eq!(
        StaffingPoolStateV1::try_new(owned, 3, 3, 0),
        Err(StaffingErrorV1::PopulationInvariant)
    );
    let maximum = binding(1, u64::MAX, 1, &[1]);
    assert_eq!(
        StaffingPoolStateV1::try_new(maximum, u64::MAX, 1, 0),
        Err(StaffingErrorV1::Arithmetic)
    );
    assert_eq!(
        StaffingStateV1::try_new(0, vec![]),
        Err(StaffingErrorV1::PeriodInvariant)
    );
}

#[test]
fn shared_pool_ownership_refuses_duplicate_process_pool_and_site_unit() {
    let original = binding(1, 5, 2, &[1]);
    let first = pool(original.clone(), 2, 4);
    assert_eq!(
        StaffingStateV1::try_new(1, vec![first.clone(), first.clone()]),
        Err(StaffingErrorV1::DuplicatePool)
    );
    let other = pool(binding(2, 5, 2, &[1]), 2, 4);
    assert_eq!(
        StaffingStateV1::try_new(1, vec![first.clone(), other]),
        Err(StaffingErrorV1::DuplicateProcess)
    );
    let alias = StaffingPoolBindingV1::try_new(
        StaffingPoolIdV1::from_bytes([2; 32]),
        original.site_id(),
        original.unit_id(),
        5,
        original.policy(),
        vec![ProcessIdV1::from_bytes([2; 32])],
    )
    .expect("binding alone has unique processes");
    assert_eq!(
        StaffingStateV1::try_new(1, vec![first, pool(alias, 2, 4)]),
        Err(StaffingErrorV1::DuplicateSiteUnit)
    );
    assert_eq!(
        StaffingPoolBindingV1::try_new(
            original.pool_id(),
            original.site_id(),
            original.unit_id(),
            5,
            original.policy(),
            vec![ProcessIdV1::from_bytes([1; 32]); 2]
        ),
        Err(StaffingErrorV1::DuplicateProcess),
    );
}

#[test]
fn empty_or_overbound_process_membership_and_requests_refuse() {
    let owned = binding(1, 5, 2, &[1]);
    assert_eq!(
        StaffingPoolBindingV1::try_new(
            owned.pool_id(),
            owned.site_id(),
            owned.unit_id(),
            5,
            owned.policy(),
            vec![]
        ),
        Err(StaffingErrorV1::EmptyProcesses)
    );
    assert_eq!(
        StaffingPoolBindingV1::try_new(
            owned.pool_id(),
            owned.site_id(),
            owned.unit_id(),
            5,
            owned.policy(),
            vec![ProcessIdV1::from_bytes([1; 32]); MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1]
        ),
        Err(StaffingErrorV1::RowLimit)
    );
    let opening = state(vec![pool(owned.clone(), 2, 4)]);
    refusal(
        &opening,
        &vec![request(&owned, 1, 0); MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1],
        StaffingErrorV1::RowLimit,
    );
}

#[test]
fn a_request_from_another_period_is_not_current_work() {
    let owned = binding(1, 5, 3, &[1]);
    let opening = state(vec![pool(owned.clone(), 0, 0)]);
    refusal(
        &opening,
        &[request_at(0, &owned, 1, 3)],
        StaffingErrorV1::PeriodInvariant,
    );
    refusal(
        &opening,
        &[request_at(2, &owned, 1, 3)],
        StaffingErrorV1::PeriodInvariant,
    );
    let second = advance_staffing_v1(&opening, &[request(&owned, 1, 3)]).expect("current request");
    refusal(
        second.state(),
        &[request(&owned, 1, 3)],
        StaffingErrorV1::PeriodInvariant,
    );
}
