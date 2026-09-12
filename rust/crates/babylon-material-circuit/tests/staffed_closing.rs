//! The staffing seam reads one real close and plans against authoritative new hours.

use babylon_material_circuit::{
    advance_material_circuit, advance_staffing, close_material_period, BacklogRow, CapacityRow,
    CorridorCapacity, CorridorId, FreightMassCoefficient, GoodId, InputOutputCoefficient,
    InventoryRow, LaborCapacityRow, LaborCoefficient, LogisticsNodeId, MaterialCircuitError,
    MaterialCircuitState, OrderAccessMode, OrderId, OrderRow, ProcessId, ProcessOutput,
    ProductionCommitment, RouteId, RouteStage, RouteStageCapacity, SiteId, SiteLogisticsNode,
    StaffingPolicy, StaffingPoolBinding, StaffingPoolId, StaffingPoolState, StaffingState,
    StaffingWorkSource, SupplierRoute, SupplierTransport, UnitId,
};

fn site(value: u8) -> SiteId {
    SiteId::from_bytes([value; 32])
}

fn process(value: u8) -> ProcessId {
    ProcessId::from_bytes([value; 32])
}

fn good(value: u8) -> GoodId {
    GoodId::from_bytes([value; 32])
}

fn unit(value: u8) -> UnitId {
    UnitId::from_bytes([value; 32])
}

fn labor(period: u64, available: u64) -> LaborCapacityRow {
    LaborCapacityRow {
        site_id: site(1),
        unit_id: unit(1),
        period,
        available,
    }
}

fn binding(processes: &[u8]) -> StaffingPoolBinding {
    StaffingPoolBinding::try_new(
        StaffingPoolId::from_bytes([1; 32]),
        site(1),
        unit(1),
        2,
        StaffingPolicy::one_period(40).unwrap(),
        processes
            .iter()
            .copied()
            .map(|id| StaffingWorkSource::Production(process(id)))
            .collect(),
    )
    .unwrap()
}

fn opening() -> MaterialCircuitState {
    MaterialCircuitState {
        merchants: vec![],
        handling_coefficients: vec![],
        final_demand_principals: vec![],
        final_demand_orders: vec![],
        period: 1,
        site_logistics_nodes: Vec::new(),
        process_outputs: vec![ProcessOutput {
            process_id: process(1),
            site_id: site(1),
            good_id: good(2),
            unit_id: unit(2),
            quantity_per_batch: 5,
        }],
        input_coefficients: vec![InputOutputCoefficient {
            process_id: process(1),
            good_id: good(1),
            unit_id: unit(2),
            quantity_per_batch: 2,
        }],
        labor_coefficients: vec![LaborCoefficient {
            process_id: process(1),
            unit_id: unit(1),
            quantity_per_batch: 10,
        }],
        supplier_routes: Vec::new(),
        freight_mass_coefficients: Vec::new(),
        route_stage_capacities: Vec::new(),
        route_stages: Vec::new(),
        inventory: vec![InventoryRow {
            site_id: site(1),
            good_id: good(1),
            unit_id: unit(2),
            quantity: 12,
        }],
        orders: Vec::new(),
        backlog: Vec::new(),
        freight: Vec::new(),
        corridor_capacities: Vec::new(),
        capacities: vec![CapacityRow {
            process_id: process(1),
            site_id: site(1),
            period: 2,
            available_batches: 4,
        }],
        labor: vec![labor(1, 0)],
        production_commitments: Vec::new(),
    }
}

fn arrival_opening() -> MaterialCircuitState {
    let mut state = opening();
    state.inventory[0].site_id = site(2);
    state.inventory[0].quantity = 4;
    state.capacities[0].period = 3;
    state.capacities[0].available_batches = 2;
    state.labor_coefficients[0].quantity_per_batch = 20;
    state.site_logistics_nodes = [1, 2]
        .map(|value| SiteLogisticsNode {
            site_id: site(value),
            node_id: LogisticsNodeId::from_bytes([value; 32]),
        })
        .to_vec();
    state.supplier_routes.push(SupplierRoute {
        transport_kind: SupplierTransport::Staged,
        buyer_site_id: site(1),
        supplier_site_id: site(2),
        good_id: good(1),
        unit_id: unit(2),
        route_id: RouteId::from_bytes([1; 32]),
    });
    state.route_stages.push(RouteStage {
        route_id: RouteId::from_bytes([1; 32]),
        stage_index: 0,
        from_node_id: LogisticsNodeId::from_bytes([2; 32]),
        to_node_id: LogisticsNodeId::from_bytes([1; 32]),
        travel_periods: 1,
        loss_ppm: 0,
    });
    state.orders.push(OrderRow {
        order_id: OrderId::from_bytes([1; 32]),
        access_mode: OrderAccessMode::CommoditySale,
        buyer_site_id: site(1),
        supplier_site_id: site(2),
        good_id: good(1),
        unit_id: unit(2),
        ordered: 4,
        shipped: 0,
        lost: 0,
        delivered: 0,
        realized: 0,
    });
    state.backlog.push(BacklogRow {
        order_id: OrderId::from_bytes([1; 32]),
        quantity: 4,
    });
    state.corridor_capacities.push(CorridorCapacity {
        corridor_id: CorridorId::from_bytes([1; 32]),
        period: 1,
        available_grams: 4,
    });
    state.route_stage_capacities.push(RouteStageCapacity {
        route_id: RouteId::from_bytes([1; 32]),
        stage_index: 0,
        corridor_id: CorridorId::from_bytes([1; 32]),
    });
    state
        .freight_mass_coefficients
        .push(FreightMassCoefficient {
            good_id: good(1),
            unit_id: unit(2),
            grams_per_unit: 1,
        });
    state
}

#[test]
fn real_arrival_requests_work_with_zero_employment_then_plans_next_period() {
    let opening = arrival_opening();
    let original = opening.clone();
    let binding = binding(&[1]);
    let dispatched = close_material_period(&opening)
        .unwrap()
        .finish_with_labor(vec![labor(2, 0)])
        .unwrap();
    assert_eq!(dispatched.dispatches.len(), 1);
    assert!(dispatched.state.production_commitments.is_empty());
    assert_eq!(opening, original);

    let closed = close_material_period(&dispatched.state).unwrap();
    assert_eq!((closed.closing_period(), closed.next_period()), (2, 3));
    assert_eq!(closed.inventory()[0].quantity, 4);
    let requests = closed
        .staffing_requests(std::slice::from_ref(&binding))
        .unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].period(), 2);
    assert_eq!(requests[0].hours(), 40);
    let people = StaffingState::try_new(
        2,
        vec![StaffingPoolState::try_new(binding, 0, 2, 0).unwrap()],
    )
    .unwrap();
    let staffing = advance_staffing(&people, &requests).unwrap();
    assert_eq!(staffing.receipts()[0].hires(), 1);
    let arrival = closed
        .finish_with_labor(staffing.next_labor().to_vec())
        .unwrap();
    assert_eq!(arrival.arrivals.len(), 1);
    assert!(arrival.production.is_empty());
    assert_eq!(arrival.state.production_commitments[0].planned_batches, 2);
    assert_eq!(arrival.state.labor, vec![labor(3, 40)]);
    let production = close_material_period(&arrival.state)
        .unwrap()
        .finish_with_labor(vec![labor(4, 0)])
        .unwrap();
    assert_eq!(production.production[0].produced_batches, 2);
    // Debiting preserves existing zero rows, including the emptied supplier.
    assert_eq!(
        production.state.inventory,
        vec![
            InventoryRow {
                site_id: site(1),
                good_id: good(1),
                unit_id: unit(2),
                quantity: 0,
            },
            InventoryRow {
                site_id: site(1),
                good_id: good(2),
                unit_id: unit(2),
                quantity: 10,
            },
            InventoryRow {
                site_id: site(2),
                good_id: good(1),
                unit_id: unit(2),
                quantity: 0,
            },
        ],
    );
}

fn shared_opening() -> MaterialCircuitState {
    let mut state = opening();
    for id in [2, 3] {
        let mut output = state.process_outputs[0].clone();
        output.process_id = process(id);
        output.good_id = good(id + 1);
        state.process_outputs.push(output);
        let mut input = state.input_coefficients[0].clone();
        input.process_id = process(id);
        state.input_coefficients.push(input);
        let mut coefficient = state.labor_coefficients[0].clone();
        coefficient.process_id = process(id);
        state.labor_coefficients.push(coefficient);
        let mut capacity = state.capacities[0].clone();
        capacity.process_id = process(id);
        capacity.available_batches = if id == 2 { 4 } else { 0 };
        state.capacities.push(capacity);
    }
    state
}

#[test]
fn requests_share_inputs_preserve_zero_processes_and_ignore_labor_under_permutations() {
    let mut state = shared_opening();
    state.labor.extend([labor(2, u64::MAX), labor(3, u64::MAX)]);
    let mut twin = state.clone();
    twin.process_outputs.reverse();
    twin.input_coefficients.reverse();
    twin.labor_coefficients.reverse();
    twin.capacities.reverse();
    twin.labor = vec![labor(1, 0)];
    let bindings = [binding(&[3, 1, 2])];
    let closed = close_material_period(&state).unwrap();
    let requests = closed.staffing_requests(&bindings).unwrap();
    let permuted = close_material_period(&twin).unwrap();
    assert_eq!(requests, permuted.staffing_requests(&bindings).unwrap());
    assert_eq!(
        requests.iter().map(|row| row.hours()).collect::<Vec<_>>(),
        [30, 30, 0]
    );
    assert_eq!(closed.inventory(), state.inventory);
    // The final planner must use the real small replacement, not request-mode grants
    // or either preseeded future schedule. Shared labor allows one batch each.
    let actual = closed.finish_with_labor(vec![labor(2, 20)]).unwrap();
    let twin = permuted.finish_with_labor(vec![labor(2, 20)]).unwrap();
    assert_eq!(actual, twin);
    assert_eq!(actual.state.labor, vec![labor(2, 20)]);
    assert_eq!(actual.state.production_commitments.len(), 2);
    assert!(actual
        .state
        .production_commitments
        .iter()
        .all(|row| row.planned_batches == 1));
    assert_eq!(actual.state.inventory, state.inventory);
    assert!(actual.production.is_empty());
}

#[test]
fn staffing_requests_refuse_incomplete_duplicate_and_foreign_bindings() {
    let state = shared_opening();
    let closed = close_material_period(&state).unwrap();
    for bindings in [vec![], vec![binding(&[1, 2])], vec![binding(&[1, 2, 4])]] {
        assert_eq!(
            closed.staffing_requests(&bindings),
            Err(MaterialCircuitError::ProcessInvariant)
        );
    }
    assert_eq!(
        closed.staffing_requests(&[binding(&[1, 2, 3]), binding(&[1])]),
        Err(MaterialCircuitError::DuplicateRow),
    );
    for (owner, labor_unit) in [(site(9), unit(1)), (site(1), unit(9))] {
        let foreign = StaffingPoolBinding::try_new(
            StaffingPoolId::from_bytes([1; 32]),
            owner,
            labor_unit,
            2,
            StaffingPolicy::one_period(40).unwrap(),
            vec![1, 2, 3]
                .into_iter()
                .map(|id| StaffingWorkSource::Production(process(id)))
                .collect(),
        )
        .unwrap();
        assert_eq!(
            closed.staffing_requests(&[foreign]),
            Err(MaterialCircuitError::ProcessInvariant)
        );
    }
    assert_eq!(
        closed
            .staffing_requests(&[binding(&[1, 2, 3])])
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn invalid_next_labor_refuses_without_mutating_opening_or_publishing_a_partial_close() {
    let state = opening();
    let original = state.clone();
    let mut unknown = labor(2, 40);
    unknown.unit_id = unit(9);
    for (rows, expected) in [
        (vec![], MaterialCircuitError::CapacityInvariant),
        (vec![labor(1, 40)], MaterialCircuitError::PeriodInvariant),
        (vec![labor(3, 40)], MaterialCircuitError::PeriodInvariant),
        (vec![unknown], MaterialCircuitError::CapacityInvariant),
        (
            vec![labor(2, 20), labor(2, 20)],
            MaterialCircuitError::DuplicateRow,
        ),
    ] {
        assert_eq!(
            close_material_period(&state)
                .unwrap()
                .finish_with_labor(rows),
            Err(expected)
        );
        assert_eq!(state, original);
    }
    let finished = close_material_period(&state)
        .unwrap()
        .finish_with_labor(vec![labor(2, 0)])
        .unwrap();
    assert!(finished.state.production_commitments.is_empty());
    assert_eq!(finished.state.labor, vec![labor(2, 0)]);
}

#[test]
fn request_hour_overflow_and_closing_period_overflow_refuse_atomically() {
    let mut state = opening();
    state.input_coefficients.clear();
    state.capacities[0].available_batches = 2;
    state.labor_coefficients[0].quantity_per_batch = u64::MAX;
    let original = state.clone();
    let closed = close_material_period(&state).unwrap();
    assert_eq!(
        closed.staffing_requests(&[binding(&[1])]),
        Err(MaterialCircuitError::Arithmetic)
    );
    assert_eq!(state, original);
    assert_eq!(closed.inventory(), original.inventory);

    state.period = u64::MAX;
    state.capacities[0].period = u64::MAX;
    state.labor[0].period = u64::MAX;
    let original = state.clone();
    assert!(matches!(
        close_material_period(&state),
        Err(MaterialCircuitError::Arithmetic)
    ));
    assert_eq!(state, original);
}

#[test]
fn supplied_schedule_and_one_shot_use_identical_execution_and_planning() {
    let mut state = opening();
    state.labor.push(labor(2, 20));
    let one_shot = advance_material_circuit(&state).unwrap();
    let split = close_material_period(&state)
        .unwrap()
        .finish_with_labor(vec![labor(2, 20)])
        .unwrap();
    assert_eq!(split, one_shot);
}

#[test]
fn zero_current_labor_still_blocks_execution_before_unconstrained_next_period_requests() {
    let mut state = opening();
    let mut current = state.capacities[0].clone();
    current.period = 1;
    state.capacities.push(current);
    state.production_commitments.push(ProductionCommitment {
        process_id: process(1),
        site_id: site(1),
        period: 1,
        planned_batches: 4,
    });
    let closed = close_material_period(&state).unwrap();
    assert_eq!(closed.inventory(), state.inventory);
    assert_eq!(
        closed.staffing_requests(&[binding(&[1])]).unwrap()[0].hours(),
        40
    );
    let next = closed.finish_with_labor(vec![labor(2, 40)]).unwrap();
    assert_eq!(next.production[0].produced_batches, 0);
    assert_eq!(next.state.production_commitments[0].planned_batches, 4);
    assert_eq!(next.state.inventory, state.inventory);
}

#[test]
fn replacement_labor_is_sorted_before_the_planner_searches_multiple_principals() {
    let mut state = opening();
    let mut output = state.process_outputs[0].clone();
    output.process_id = process(2);
    output.site_id = site(2);
    state.process_outputs.push(output);
    let mut input = state.input_coefficients[0].clone();
    input.process_id = process(2);
    state.input_coefficients.push(input);
    let mut coefficient = state.labor_coefficients[0].clone();
    coefficient.process_id = process(2);
    state.labor_coefficients.push(coefficient);
    let mut inventory = state.inventory[0].clone();
    inventory.site_id = site(2);
    state.inventory.push(inventory);
    let mut capacity = state.capacities[0].clone();
    capacity.process_id = process(2);
    capacity.site_id = site(2);
    state.capacities.push(capacity);
    let mut second = labor(2, 20);
    second.site_id = site(2);
    let next = close_material_period(&state)
        .unwrap()
        .finish_with_labor(vec![second.clone(), labor(2, 10)])
        .unwrap();
    assert_eq!(next.state.labor, [labor(2, 10), second]);
    assert_eq!(next.state.production_commitments[0].planned_batches, 1);
    assert_eq!(next.state.production_commitments[1].planned_batches, 2);
}
