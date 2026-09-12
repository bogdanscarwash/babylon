use babylon_material_circuit::{LogisticsNodeId, RouteStage, RouteStageCapacity};
use babylon_tick::material_world::{decode_material_receipts, MaterialWorldRegister};

use super::*;
use crate::michigan_content::MichiganContentPreset;
use crate::michigan_material::MichiganDeliveryPreset;

fn shared_opening(meal_order: u64, capacity: u64) -> MaterialCircuitState {
    let catalog = crate::test_support::catalog();
    let foundation = MichiganContentPreset::new_campaign(MichiganDeliveryPreset::Standard)
        .create_foundation(&catalog)
        .unwrap();
    let mut state = foundation.initial_register().state().clone();
    let sheet = catalog
        .routes()
        .iter()
        .find(|route| route.good_key == "sheet")
        .unwrap();
    let meal = catalog
        .routes()
        .iter()
        .find(|route| route.good_key == "meal")
        .unwrap();
    let shared = state
        .route_stage_capacities
        .iter()
        .find(|leg| leg.route_id == sheet.id())
        .unwrap()
        .corridor_id;
    let obsolete = state
        .route_stage_capacities
        .iter()
        .find(|leg| leg.route_id == meal.id())
        .unwrap()
        .corridor_id;
    for leg in &mut state.route_stage_capacities {
        if leg.route_id == meal.id() {
            leg.corridor_id = shared;
        }
    }
    state
        .corridor_capacities
        .retain(|row| row.corridor_id != obsolete);
    for row in &mut state.corridor_capacities {
        if row.corridor_id == shared {
            row.available_grams = capacity * 1_000;
        }
    }
    state
        .orders
        .iter_mut()
        .find(|order| order.order_id == meal.order_id())
        .unwrap()
        .ordered = meal_order;
    state
        .backlog
        .iter_mut()
        .find(|row| row.order_id == meal.order_id())
        .unwrap()
        .quantity = meal_order;
    state
}

fn committed_pair(
    state: MaterialCircuitState,
) -> (
    MaterialCircuitState,
    MaterialCircuitState,
    MaterialTickReceipts,
) {
    let opening = MaterialWorldRegister::try_new(0, state).unwrap();
    let next = opening.prepare_next().unwrap();
    (
        opening.state().clone(),
        next.register().state().clone(),
        decode_material_receipts(next.receipt_bytes()).unwrap(),
    )
}

#[test]
fn shared_capacity_is_counted_once_with_exact_competing_dispatches_and_residual() {
    for (meal_order, capacity, expected) in [
        (200, 800, (320, 80, 400)),
        (200, 160, (120, 40, 0)),
        (80, 160, (141, 18, 1)),
    ] {
        let (opening, next, receipt) = committed_pair(shared_opening(meal_order, capacity));
        let accounts = project_freight_capacity_accounts(
            &crate::test_support::catalog(),
            &next,
            Some(&opening),
            Some(&receipt),
        )
        .unwrap();
        assert_eq!(
            accounts.len(),
            2,
            "one shared kg principal and the independent panel principal"
        );
        let account = accounts
            .iter()
            .find(|row| row.route_ids.len() == 2)
            .unwrap();
        let completed = account.completed.as_ref().unwrap();
        assert_eq!(completed.period, 1);
        assert_eq!(completed.reservations.len(), 1);
        let reservation = &completed.reservations[0];
        assert_eq!(
            (
                reservation.reservation_period,
                reservation.opening_available_grams,
                reservation.remaining_available_grams
            ),
            (1, capacity * 1_000, expected.2 * 1_000)
        );
        assert_eq!(
            reservation.newly_reserved_grams,
            (expected.0 + expected.1) * 1_000
        );
        assert_eq!(reservation.orders.len(), 2);
        let quantities: std::collections::BTreeSet<_> = reservation
            .orders
            .iter()
            .map(|row| row.dispatched)
            .collect();
        assert_eq!(quantities, [expected.0, expected.1].into_iter().collect());
        assert!(reservation
            .orders
            .iter()
            .all(|row| row.requested.checked_sub(row.dispatched) == Some(row.remaining_unshipped)));
        let mut permuted_opening = opening.clone();
        permuted_opening.orders.reverse();
        permuted_opening.route_stages.reverse();
        permuted_opening.route_stage_capacities.reverse();
        permuted_opening.corridor_capacities.reverse();
        let mut permuted_receipt = receipt.clone();
        permuted_receipt.dispatches.reverse();
        assert_eq!(
            accounts,
            project_freight_capacity_accounts(
                &crate::test_support::catalog(),
                &next,
                Some(&permuted_opening),
                Some(&permuted_receipt)
            )
            .unwrap()
        );
    }
}

#[test]
fn foundation_sharing_is_known_but_completed_zero_is_not_invented() {
    let state = shared_opening(200, 160);
    let accounts =
        project_freight_capacity_accounts(&crate::test_support::catalog(), &state, None, None)
            .unwrap();
    assert!(accounts.iter().all(|row| row.completed.is_none()));
    assert_eq!(
        accounts
            .iter()
            .find(|row| row.route_ids.len() == 2)
            .unwrap()
            .next_opening_available_grams,
        160_000
    );
    let (opening, next, receipt) = committed_pair(shared_opening(200, 0));
    let accounts = project_freight_capacity_accounts(
        &crate::test_support::catalog(),
        &next,
        Some(&opening),
        Some(&receipt),
    )
    .unwrap();
    let completed = accounts
        .iter()
        .find(|row| row.route_ids.len() == 2)
        .unwrap()
        .completed
        .as_ref()
        .unwrap();
    assert_eq!(completed.reservations[0].newly_reserved_grams, 0);
    assert!(completed.reservations[0]
        .orders
        .iter()
        .all(|row| row.dispatched == 0));
}

#[test]
fn missing_duplicate_and_excess_receipts_or_capacity_refuse() {
    let (opening, next, receipt) = committed_pair(shared_opening(200, 160));
    let catalog = crate::test_support::catalog();
    let mut missing = receipt.clone();
    missing.dispatches.pop();
    assert_eq!(
        project_freight_capacity_accounts(&catalog, &next, Some(&opening), Some(&missing)),
        Err(ProductionProjectionError::State)
    );
    let mut duplicate = receipt.clone();
    duplicate.dispatches.push(receipt.dispatches[0].clone());
    assert_eq!(
        project_freight_capacity_accounts(&catalog, &next, Some(&opening), Some(&duplicate)),
        Err(ProductionProjectionError::State)
    );
    let mut duplicated_capacity = opening.clone();
    duplicated_capacity
        .corridor_capacities
        .push(opening.corridor_capacities[0].clone());
    assert_eq!(
        project_freight_capacity_accounts(
            &catalog,
            &next,
            Some(&duplicated_capacity),
            Some(&receipt)
        ),
        Err(ProductionProjectionError::State)
    );
    let mut wrong_next = next.clone();
    wrong_next.corridor_capacities[0].available_grams += 1;
    assert_eq!(
        project_freight_capacity_accounts(&catalog, &wrong_next, Some(&opening), Some(&receipt)),
        Err(ProductionProjectionError::State)
    );
}

#[test]
fn reservations_for_later_legs_debit_the_future_period_without_claiming_arrival() {
    let mut state = shared_opening(200, 160);
    let catalog = crate::test_support::catalog();
    let sheet = catalog
        .routes()
        .iter()
        .find(|route| route.good_key == "sheet")
        .unwrap();
    let first = state
        .route_stages
        .iter_mut()
        .find(|leg| leg.route_id == sheet.id())
        .unwrap();
    let destination = first.to_node_id;
    let intermediate = LogisticsNodeId::from_bytes([73; 32]);
    first.to_node_id = intermediate;
    let second = RouteStage {
        route_id: first.route_id,
        stage_index: 1,
        from_node_id: intermediate,
        to_node_id: destination,
        travel_periods: 1,
        loss_ppm: 0,
    };
    let shared_id = state
        .route_stage_capacities
        .iter()
        .find(|row| row.route_id == sheet.id())
        .unwrap()
        .corridor_id;
    state.route_stage_capacities.push(RouteStageCapacity {
        route_id: sheet.id(),
        stage_index: 1,
        corridor_id: shared_id,
    });
    state.route_stages.push(second);
    let (opening, next, receipt) = committed_pair(state);
    let accounts =
        project_freight_capacity_accounts(&catalog, &next, Some(&opening), Some(&receipt)).unwrap();
    let shared = accounts
        .iter()
        .find(|row| row.route_ids.len() == 2)
        .unwrap();
    assert_eq!(
        (
            shared.next_opening_period,
            shared.next_opening_available_grams
        ),
        (2, 40_000)
    );
    let completed = shared.completed.as_ref().unwrap();
    assert_eq!(completed.period, 1);
    assert_eq!(
        completed
            .reservations
            .iter()
            .map(|row| (
                row.reservation_period,
                row.newly_reserved_grams,
                row.remaining_available_grams
            ))
            .collect::<Vec<_>>(),
        vec![(1, 160_000, 0), (2, 120_000, 40_000)]
    );
    assert!(receipt.arrivals.is_empty());
    let legs = project_route_stages(&next, sheet.id()).unwrap();
    assert_eq!(
        legs.iter().map(|leg| leg.stage_index).collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(legs[0].capacity_ids, legs[1].capacity_ids);
    let mut mismatched = next.clone();
    let capacity = mismatched
        .corridor_capacities
        .iter_mut()
        .find(|row| {
            row.period == 2 && digest_hex(&row.corridor_id.as_bytes()) == shared.corridor_id
        })
        .unwrap();
    capacity.available_grams += 1;
    assert_eq!(
        project_freight_capacity_accounts(&catalog, &mismatched, Some(&opening), Some(&receipt)),
        Err(ProductionProjectionError::State)
    );
}
