use babylon_material_circuit::{
    BacklogRow, CapacityRow, CorridorCapacity, CorridorId, FreightMassCoefficient,
    InputOutputCoefficient, InventoryRow, LaborCapacityRow, LaborCoefficient, LogisticsNodeId,
    OrderAccessMode, ProcessOutput, ProductionCommitment, RouteId, RouteStage, RouteStageCapacity,
    SiteLogisticsNode, SupplierRoute, SupplierTransport,
};
use babylon_tick::material_world::{decode_material_receipts, MaterialWorldRegister};

use super::*;
use crate::{michigan_content::MichiganContentPreset, michigan_material::MichiganDeliveryPreset};
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_practice_contract::OrderedPracticeActionBatch;
use babylon_tick::replay_session::ReplayCommitDisposition;

type Pair = (
    MaterialCircuitState,
    MaterialCircuitState,
    MaterialTickReceipts,
);

fn empty_state() -> MaterialCircuitState {
    MaterialCircuitState {
        period: 1,
        site_logistics_nodes: vec![],
        process_outputs: vec![],
        input_coefficients: vec![],
        labor_coefficients: vec![],
        supplier_routes: vec![],
        route_stages: vec![],
        route_stage_capacities: vec![],
        freight_mass_coefficients: vec![],
        merchants: vec![],
        handling_coefficients: vec![],
        final_demand_principals: vec![],
        final_demand_orders: vec![],
        inventory: vec![],
        orders: vec![],
        backlog: vec![],
        freight: vec![],
        corridor_capacities: vec![],
        capacities: vec![],
        labor: vec![],
        production_commitments: vec![],
    }
}

fn pair(state: MaterialCircuitState) -> Pair {
    let opening = MaterialWorldRegister::try_new(state.period - 1, state).unwrap();
    let next = opening.prepare_next().unwrap();
    (
        opening.state().clone(),
        next.register().state().clone(),
        decode_material_receipts(next.receipt_bytes()).unwrap(),
    )
}

fn project(pair: &Pair) -> Result<Option<CompletedMaterialBalance>, ProductionProjectionError> {
    project_with_labels(&pair.1, Some(&pair.0), Some(&pair.2), |good, unit| {
        Some((digest_hex(&good.as_bytes()), digest_hex(&unit.as_bytes())))
    })
}

fn complete(pair: &Pair) -> CompletedMaterialBalance {
    project(pair).unwrap().unwrap()
}

fn conserved(balance: &CompletedMaterialBalance) {
    for row in &balance.rows {
        assert_eq!(
            u128::from(row.opening)
                + u128::from(row.arrivals)
                + u128::from(row.local_received)
                + u128::from(row.produced),
            u128::from(row.consumed)
                + u128::from(row.dispatched)
                + u128::from(row.local_transferred)
                + u128::from(row.final_demand_fulfilled)
                + u128::from(row.closing),
            "{row:?}",
        );
    }
}

fn stock(site: u8, good: u8, unit: u8, quantity: u64) -> InventoryRow {
    InventoryRow {
        site_id: SiteId::from_bytes([site; 32]),
        good_id: GoodId::from_bytes([good; 32]),
        unit_id: UnitId::from_bytes([unit; 32]),
        quantity,
    }
}

fn production_state(specs: &[(u8, u64, u64, u64)], opening: u64) -> MaterialCircuitState {
    let mut state = empty_state();
    let inventory = stock(1, 2, 3, opening);
    let mut labor = 0_u64;
    for &(id, input, output, batches) in specs {
        let process_id = ProcessId::from_bytes([id; 32]);
        state.process_outputs.push(ProcessOutput {
            process_id,
            site_id: inventory.site_id,
            good_id: inventory.good_id,
            unit_id: inventory.unit_id,
            quantity_per_batch: output,
        });
        state.input_coefficients.push(InputOutputCoefficient {
            process_id,
            good_id: inventory.good_id,
            unit_id: inventory.unit_id,
            quantity_per_batch: input,
        });
        state.labor_coefficients.push(LaborCoefficient {
            process_id,
            unit_id: UnitId::from_bytes([4; 32]),
            quantity_per_batch: 1,
        });
        state.capacities.push(CapacityRow {
            process_id,
            site_id: inventory.site_id,
            period: 1,
            available_batches: batches,
        });
        state.production_commitments.push(ProductionCommitment {
            process_id,
            site_id: inventory.site_id,
            period: 1,
            planned_batches: batches,
        });
        labor = labor.checked_add(batches).unwrap();
    }
    state.labor.push(LaborCapacityRow {
        site_id: inventory.site_id,
        unit_id: UnitId::from_bytes([4; 32]),
        period: 1,
        available: labor,
    });
    state.inventory.push(inventory);
    state
}

#[test]
fn shared_process_principal_records_production_and_consumption_separately_once() {
    let pair = pair(production_state(&[(10, 2, 3, 2), (11, 1, 2, 3)], 10));
    let balance = complete(&pair);
    assert_eq!(balance.period, 1);
    assert_eq!(balance.rows.len(), 1);
    let row = &balance.rows[0];
    assert_eq!(
        (row.opening, row.produced, row.consumed, row.closing),
        (10, 12, 7, 15)
    );
    assert_eq!((row.arrivals, row.dispatched), (0, 0));
    conserved(&balance);
    let mut reversed = pair.clone();
    for state in [&mut reversed.0, &mut reversed.1] {
        state.process_outputs.reverse();
        state.input_coefficients.reverse();
        state.production_commitments.reverse();
        state.inventory.reverse();
    }
    reversed.2.production.reverse();
    assert_eq!(complete(&reversed), balance);
}

#[test]
fn foundation_is_absent_but_committed_quiet_and_empty_accounts_are_present() {
    let mut state = empty_state();
    state.inventory.push(stock(1, 2, 3, 9));
    assert_eq!(
        project_with_labels(&state, None, None, |_, _| None),
        Ok(None)
    );
    let balance = complete(&pair(state));
    let row = &balance.rows[0];
    assert_eq!((row.opening, row.closing), (9, 9));
    assert_eq!(
        (row.arrivals, row.produced, row.consumed, row.dispatched),
        (0, 0, 0, 0)
    );
    assert_eq!(complete(&pair(empty_state())).rows.len(), 0);
}

#[test]
fn exact_units_and_sites_never_merge_even_when_labels_match() {
    let mut state = empty_state();
    state.inventory = vec![stock(1, 2, 3, 7), stock(1, 2, 4, 9), stock(5, 2, 3, 11)];
    let (prior, current, receipt) = pair(state);
    let balance = project_with_labels(&current, Some(&prior), Some(&receipt), |_, _| {
        Some(("material".to_owned(), "unit".to_owned()))
    })
    .unwrap()
    .unwrap();
    assert_eq!(balance.rows.len(), 3);
    let keys: BTreeSet<_> = balance
        .rows
        .iter()
        .map(|row| (&row.site_id, &row.good_id, &row.unit_id))
        .collect();
    assert_eq!(keys.len(), 3);
    assert_eq!(balance.rows.iter().map(|row| row.opening).sum::<u64>(), 27);
    conserved(&balance);
}

fn freight_state(loss_ppm: u32) -> MaterialCircuitState {
    let mut state = empty_state();
    let inventory = stock(1, 2, 3, 100);
    let buyer = SiteId::from_bytes([4; 32]);
    let source = LogisticsNodeId::from_bytes([5; 32]);
    let destination = LogisticsNodeId::from_bytes([6; 32]);
    let route = RouteId::from_bytes([7; 32]);
    let corridor = CorridorId::from_bytes([8; 32]);
    let order = OrderId::from_bytes([9; 32]);
    state.site_logistics_nodes = vec![
        SiteLogisticsNode {
            site_id: inventory.site_id,
            node_id: source,
        },
        SiteLogisticsNode {
            site_id: buyer,
            node_id: destination,
        },
    ];
    state.supplier_routes.push(SupplierRoute {
        transport_kind: SupplierTransport::Staged,
        buyer_site_id: buyer,
        supplier_site_id: inventory.site_id,
        good_id: inventory.good_id,
        unit_id: inventory.unit_id,
        route_id: route,
    });
    state.route_stage_capacities.push(RouteStageCapacity {
        route_id: route,
        stage_index: 0,
        corridor_id: corridor,
    });
    state.route_stages.push(RouteStage {
        route_id: route,
        stage_index: 0,
        from_node_id: source,
        to_node_id: destination,
        travel_periods: 1,
        loss_ppm,
    });
    state.corridor_capacities.push(CorridorCapacity {
        corridor_id: corridor,
        period: 1,
        available_grams: 100,
    });
    state.orders.push(OrderRow {
        order_id: order,
        access_mode: OrderAccessMode::CommoditySale,
        buyer_site_id: buyer,
        supplier_site_id: inventory.site_id,
        good_id: inventory.good_id,
        unit_id: inventory.unit_id,
        ordered: 100,
        shipped: 0,
        lost: 0,
        delivered: 0,
        realized: 0,
    });
    state.backlog.push(BacklogRow {
        order_id: order,
        quantity: 100,
    });
    state
        .freight_mass_coefficients
        .push(FreightMassCoefficient {
            good_id: inventory.good_id,
            unit_id: inventory.unit_id,
            grams_per_unit: 1,
        });
    state.inventory.push(inventory);
    state
}

fn site_row(balance: &CompletedMaterialBalance, site: u8) -> &ProductionMaterialBalanceRow {
    let id = digest_hex(&[site; 32]);
    balance.rows.iter().find(|row| row.site_id == id).unwrap()
}

#[test]
fn dispatch_and_partial_or_total_loss_do_not_charge_local_stock_twice() {
    for (loss, arrived) in [(0, 100), (250_000, 75), (1_000_000, 0)] {
        let dispatched = pair(freight_state(loss));
        let first = complete(&dispatched);
        assert_eq!(site_row(&first, 1).dispatched, 100);
        assert_eq!(site_row(&first, 1).closing, 0);
        let arrived_pair = pair(dispatched.1);
        let second = complete(&arrived_pair);
        let supplier = site_row(&second, 1);
        assert_eq!(
            (supplier.opening, supplier.dispatched, supplier.closing),
            (0, 0, 0)
        );
        let buyer = site_row(&second, 4);
        assert_eq!(
            (buyer.opening, buyer.arrivals, buyer.closing),
            (0, arrived, arrived)
        );
        assert_eq!(
            arrived_pair
                .2
                .losses
                .iter()
                .map(|row| row.quantity)
                .sum::<u64>(),
            100 - arrived
        );
        conserved(&first);
        conserved(&second);
    }
}

fn two_leg_work_state() -> MaterialCircuitState {
    let mut state = freight_state(250_000);
    let destination = state.route_stages[0].to_node_id;
    let intermediate = LogisticsNodeId::from_bytes([10; 32]);
    state.route_stages[0].to_node_id = intermediate;
    state.route_stage_capacities.push(RouteStageCapacity {
        route_id: state.route_stages[0].route_id,
        stage_index: 1,
        corridor_id: CorridorId::from_bytes([11; 32]),
    });
    state.route_stages.push(RouteStage {
        route_id: state.route_stages[0].route_id,
        stage_index: 1,
        from_node_id: intermediate,
        to_node_id: destination,
        travel_periods: 1,
        loss_ppm: 0,
    });
    // The actual V3 dispatcher reserves the full dispatched quantity on every
    // leg, without anticipating the loss that will later occur in transit.
    state.corridor_capacities.push(CorridorCapacity {
        corridor_id: state.route_stage_capacities[1].corridor_id,
        period: 2,
        available_grams: 100,
    });
    add_receiving_work(&mut state);
    add_receiving_dispatch(&mut state);
    state
}

fn add_receiving_work(state: &mut MaterialCircuitState) {
    let buyer = state.orders[0].buyer_site_id;
    let mut work = production_state(&[(17, 1, 2, 1)], 5);
    work.process_outputs[0].site_id = buyer;
    work.inventory[0].site_id = buyer;
    work.capacities[0].site_id = buyer;
    work.labor[0].site_id = buyer;
    work.production_commitments[0].site_id = buyer;
    for period in [2, 3] {
        work.capacities.push(CapacityRow {
            period,
            ..work.capacities[0].clone()
        });
        work.labor.push(LaborCapacityRow {
            period,
            ..work.labor[0].clone()
        });
    }
    state.process_outputs.extend(work.process_outputs);
    state.input_coefficients.extend(work.input_coefficients);
    state.labor_coefficients.extend(work.labor_coefficients);
    state.inventory.extend(work.inventory);
    state.capacities.extend(work.capacities);
    state.labor.extend(work.labor);
    state
        .production_commitments
        .extend(work.production_commitments);
}

fn add_receiving_dispatch(state: &mut MaterialCircuitState) {
    let supplier = state.orders[0].buyer_site_id;
    let buyer = SiteId::from_bytes([15; 32]);
    let destination = LogisticsNodeId::from_bytes([14; 32]);
    let route = RouteId::from_bytes([12; 32]);
    let corridor = CorridorId::from_bytes([13; 32]);
    let order = OrderId::from_bytes([16; 32]);
    state.site_logistics_nodes.push(SiteLogisticsNode {
        site_id: buyer,
        node_id: destination,
    });
    state.route_stage_capacities.push(RouteStageCapacity {
        route_id: route,
        stage_index: 0,
        corridor_id: corridor,
    });
    state.route_stages.push(RouteStage {
        route_id: route,
        stage_index: 0,
        from_node_id: state.route_stages[1].to_node_id,
        to_node_id: destination,
        travel_periods: 1,
        loss_ppm: 0,
    });
    state.supplier_routes.push(SupplierRoute {
        transport_kind: SupplierTransport::Staged,
        supplier_site_id: supplier,
        buyer_site_id: buyer,
        route_id: route,
        ..state.supplier_routes[0].clone()
    });
    state.orders.push(OrderRow {
        order_id: order,
        supplier_site_id: supplier,
        buyer_site_id: buyer,
        ordered: 4,
        ..state.orders[0].clone()
    });
    state.backlog.push(BacklogRow {
        order_id: order,
        quantity: 4,
    });
    state.corridor_capacities.push(CorridorCapacity {
        corridor_id: corridor,
        period: 3,
        available_grams: 4,
    });
}

fn assert_intermediate_transit_loss(intermediate: &Pair) {
    assert_eq!(intermediate.2.losses.len(), 1);
    assert_eq!(intermediate.2.losses[0].quantity, 25);
    assert!(intermediate.2.arrivals.is_empty());
    assert!(intermediate.2.deliveries.is_empty());
    assert!(intermediate.2.realizations.is_empty());
    assert_eq!(intermediate.1.freight.len(), 1);
    assert_eq!(intermediate.1.freight[0].current_stage_index, 1);
    assert_eq!(intermediate.1.freight[0].quantity, 75);
    let intermediate_balance = complete(intermediate);
    let supplier = site_row(&intermediate_balance, 1);
    assert_eq!(
        (supplier.opening, supplier.dispatched, supplier.closing),
        (0, 0, 0)
    );
    assert_eq!(site_row(&intermediate_balance, 4).arrivals, 0);
    conserved(&intermediate_balance);
}

#[test]
fn intermediate_loss_then_final_arrival_preserves_same_principal_work_and_dispatch() {
    let dispatched = pair(two_leg_work_state());
    assert_eq!(dispatched.2.dispatches.len(), 1);
    assert_eq!(dispatched.2.dispatches[0].quantity, 100);
    assert_eq!(dispatched.2.dispatches[0].final_arrival_period, 3);
    conserved(&complete(&dispatched));

    let intermediate = pair(dispatched.1);
    assert_intermediate_transit_loss(&intermediate);

    let arrival = pair(intermediate.1);
    assert!(arrival.2.losses.is_empty());
    assert_eq!(arrival.2.arrivals.len(), 1);
    assert_eq!(arrival.2.arrivals[0].quantity, 75);
    assert_eq!(arrival.2.production.len(), 1);
    assert_eq!(arrival.2.production[0].produced_batches, 1);
    assert_eq!(arrival.2.dispatches.len(), 1);
    assert_eq!(arrival.2.dispatches[0].quantity, 4);
    let balance = complete(&arrival);
    let receiver = site_row(&balance, 4);
    assert_eq!(
        (
            receiver.opening,
            receiver.arrivals,
            receiver.produced,
            receiver.consumed,
            receiver.dispatched,
            receiver.closing
        ),
        (7, 75, 2, 1, 4, 79),
    );
    assert_eq!(
        receiver.unit_id,
        digest_hex(&UnitId::from_bytes([3; 32]).as_bytes())
    );
    assert_eq!(arrival.1.freight.len(), 1);
    assert_eq!(
        arrival.1.freight[0].source_site_id,
        SiteId::from_bytes([4; 32])
    );
    assert_eq!(arrival.1.freight[0].quantity, 4);
    conserved(&balance);
}

#[test]
fn arrival_family_retains_multiplicity_without_counting_delivery_or_realization_again() {
    let mut received = pair(pair(freight_state(0)).1);
    let baseline = complete(&received);
    let mut extra = received.2.arrivals[0].clone();
    extra.quantity = 40;
    received.2.arrivals[0].quantity = 60;
    received.2.arrivals.push(extra);
    // Arrival rows do not carry lot IDs. Their per-order quantity is additive;
    // receipt-family identity is verified before this accounting boundary.
    assert_eq!(complete(&received), baseline);
    let buyer = site_row(&baseline, 4);
    assert_eq!((buyer.arrivals, buyer.closing), (100, 100));
}

fn michigan_period(preset: MichiganDeliveryPreset, period: u64) -> Pair {
    let mut session = MichiganContentPreset::new_campaign(preset)
        .create_foundation(&crate::test_support::catalog())
        .unwrap()
        .into_session()
        .unwrap();
    for tick in 1..=period {
        let actions = OrderedPracticeActionBatch::empty(
            session.graph_session().session_identity().clone(),
            tick,
        )
        .unwrap();
        let next = session.prepare_advance(&actions).unwrap();
        if tick == period {
            return (
                session.material().state().clone(),
                next.material().register().state().clone(),
                decode_material_receipts(next.material().receipt_bytes()).unwrap(),
            );
        }
        session
            .commit_prepared_and_publish(&mut CollectingSink::default(), next, |_| {
                Ok::<_, ()>(ReplayCommitDisposition::Committed)
            })
            .unwrap();
    }
    panic!("fixture requires a completed period");
}

#[test]
fn delivery_twins_explain_downstream_input_use_and_preserve_unrelated_food() {
    let standard = michigan_period(MichiganDeliveryPreset::Standard, 3);
    let delayed = michigan_period(MichiganDeliveryPreset::Delayed, 3);
    let a = project_material_balance(
        &crate::test_support::catalog(),
        &standard.1,
        Some(&standard.0),
        Some(&standard.2),
    )
    .unwrap()
    .unwrap();
    let b = project_material_balance(
        &crate::test_support::catalog(),
        &delayed.1,
        Some(&delayed.0),
        Some(&delayed.2),
    )
    .unwrap()
    .unwrap();
    let catalog = crate::test_support::catalog();
    let macomb = digest_hex(
        &catalog
            .site("macomb-fabricated-metal")
            .unwrap()
            .id()
            .as_bytes(),
    );
    assert!(a
        .rows
        .iter()
        .any(|row| row.site_id == macomb && row.consumed > 0));
    assert!(b
        .rows
        .iter()
        .filter(|row| row.site_id == macomb)
        .all(|row| row.consumed == 0));
    for site in catalog.sites().iter().filter(|site| site.naics == "311") {
        let id = digest_hex(&site.id().as_bytes());
        assert_eq!(
            a.rows
                .iter()
                .filter(|row| row.site_id == id)
                .collect::<Vec<_>>(),
            b.rows
                .iter()
                .filter(|row| row.site_id == id)
                .collect::<Vec<_>>(),
        );
    }
    conserved(&a);
    conserved(&b);
    // Reading a later pair does not alter a historical account or its inputs.
    let original = standard.clone();
    let later = michigan_period(MichiganDeliveryPreset::Standard, 4);
    conserved(&complete(&later));
    assert_eq!(standard, original);
    assert_eq!(
        project_material_balance(
            &crate::test_support::catalog(),
            &standard.1,
            Some(&standard.0),
            Some(&standard.2)
        )
        .unwrap(),
        Some(a)
    );
}

#[test]
fn widened_equality_accepts_valid_turnover_larger_than_u64() {
    let actual = pair(production_state(&[(10, 1, 1, u64::MAX)], u64::MAX));
    let balance = complete(&actual);
    let row = &balance.rows[0];
    assert_eq!(
        (row.opening, row.produced, row.consumed, row.closing),
        (u64::MAX, u64::MAX, u64::MAX, u64::MAX)
    );
    assert!(row.opening.checked_add(row.produced).is_none());
    conserved(&balance);
}

fn refuses_unchanged(pair: &Pair, error: ProductionProjectionError) {
    let before = pair.clone();
    assert_eq!(project(pair), Err(error));
    assert_eq!(
        pair, &before,
        "projection refusal never mutates committed inputs"
    );
}

#[test]
fn products_and_shared_component_sums_refuse_overflow() {
    let mut multiplication = pair(production_state(&[(10, 1, 1, 2)], 10));
    for state in [&mut multiplication.0, &mut multiplication.1] {
        state.process_outputs[0].quantity_per_batch = u64::MAX;
    }
    refuses_unchanged(&multiplication, ProductionProjectionError::Arithmetic);
    let mut addition = pair(production_state(&[(10, 1, 1, 1), (11, 1, 1, 1)], 10));
    for state in [&mut addition.0, &mut addition.1] {
        for output in &mut state.process_outputs {
            output.quantity_per_batch = u64::MAX / 2 + 1;
        }
    }
    refuses_unchanged(&addition, ProductionProjectionError::Arithmetic);
    let mut arrivals = pair(pair(freight_state(0)).1);
    arrivals.2.arrivals[0].quantity = u64::MAX;
    arrivals.2.arrivals.push(arrivals.2.arrivals[0].clone());
    refuses_unchanged(&arrivals, ProductionProjectionError::Arithmetic);
}

#[test]
fn incomplete_or_nonadjacent_history_is_never_a_zero_account() {
    let actual = pair(production_state(&[(10, 1, 1, 2)], 10));
    for (prior, receipt) in [
        (None, None),
        (Some(&actual.0), None),
        (None, Some(&actual.2)),
    ] {
        assert_eq!(
            project_with_labels(&actual.1, prior, receipt, |_, _| None),
            Err(ProductionProjectionError::History)
        );
    }
    let mut wrong = actual.clone();
    wrong.2.resolve_tick += 1;
    refuses_unchanged(&wrong, ProductionProjectionError::History);
    let mut wrong = actual;
    wrong.1.period += 1;
    refuses_unchanged(&wrong, ProductionProjectionError::History);
}

#[test]
fn production_family_and_recipe_or_inventory_mismatches_refuse() {
    let actual = pair(production_state(&[(10, 1, 1, 2)], 10));
    for case in 0..9 {
        let mut changed = actual.clone();
        match case {
            0 => changed.2.production.clear(),
            1 => changed.2.production.push(changed.2.production[0].clone()),
            2 => changed.2.production[0].site_id = SiteId::from_bytes([99; 32]),
            3 => changed.2.production[0].process_id = ProcessId::from_bytes([99; 32]),
            4 => changed.2.production[0].planned_batches += 1,
            5 => changed.1.process_outputs[0].quantity_per_batch += 1,
            6 => changed.1.input_coefficients[0].quantity_per_batch += 1,
            7 => changed.1.inventory[0].quantity += 1,
            _ => changed.0.inventory.push(changed.0.inventory[0].clone()),
        }
        refuses_unchanged(&changed, ProductionProjectionError::State);
    }
}

#[test]
fn every_transport_family_must_match_exact_order_and_lot_evidence() {
    let dispatch = pair(freight_state(250_000));
    for case in 0..5 {
        let mut changed = dispatch.clone();
        match case {
            0 => changed.2.dispatches.clear(),
            1 => changed.2.dispatches.push(changed.2.dispatches[0].clone()),
            2 => changed.2.dispatches[0].route_id = RouteId::from_bytes([99; 32]),
            3 => changed.2.dispatches[0].quantity -= 1,
            _ => changed.1.orders[0].unit_id = UnitId::from_bytes([99; 32]),
        }
        refuses_unchanged(&changed, ProductionProjectionError::State);
    }
    check_arrival_refusals(&pair(dispatch.1));
}

fn check_arrival_refusals(arrival: &Pair) {
    for case in 0..9 {
        let mut changed = arrival.clone();
        match case {
            0 => changed.2.arrivals.clear(),
            1 => changed.2.deliveries.clear(),
            2 => changed.2.realizations.clear(),
            3 => changed.2.losses.clear(),
            4 => changed.2.losses[0].route_id = RouteId::from_bytes([99; 32]),
            5 => changed.2.losses.push(changed.2.losses[0].clone()),
            6 => changed.2.arrivals[0].order_id = OrderId::from_bytes([99; 32]),
            7 => changed.2.deliveries[0].quantity += 1,
            _ => changed.2.arrivals[0].quantity = 0,
        }
        refuses_unchanged(&changed, ProductionProjectionError::State);
    }
}

#[test]
fn unknown_unit_metadata_refuses_instead_of_inventing_a_label() {
    let mut state = empty_state();
    state.inventory.push(stock(1, 2, 3, 9));
    let (prior, current, receipt) = pair(state);
    assert_eq!(
        project_with_labels(&current, Some(&prior), Some(&receipt), |_, _| None),
        Err(ProductionProjectionError::Content)
    );
    assert_eq!(
        project_material_balance(
            &crate::test_support::catalog(),
            &current,
            Some(&prior),
            Some(&receipt)
        ),
        Err(ProductionProjectionError::Content)
    );
}

fn retail_state() -> MaterialCircuitState {
    use babylon_material_circuit::{
        CorridorCapacity, FinalDemandOrder, FinalDemandPrincipal, FinalDemandPrincipalId,
        FreightMassCoefficient, MerchantHandling, MerchantHandlingCoefficient, MerchantRole,
    };
    let mut state = empty_state();
    let catalog = crate::test_support::catalog();
    let good = catalog.good("meal").unwrap();
    let inventory = InventoryRow {
        good_id: good.id(),
        unit_id: good.unit_id(),
        ..stock(1, 2, 3, 10)
    };
    let labor_unit_id = UnitId::from_bytes([4; 32]);
    let capacity_id = catalog.corridors()[0].id();
    let demand_principal_id = FinalDemandPrincipalId::from_bytes([6; 32]);
    state.site_logistics_nodes.push(SiteLogisticsNode {
        site_id: inventory.site_id,
        node_id: LogisticsNodeId::from_bytes([7; 32]),
    });
    state
        .freight_mass_coefficients
        .push(FreightMassCoefficient {
            good_id: inventory.good_id,
            unit_id: inventory.unit_id,
            grams_per_unit: 10,
        });
    state.merchants.push(MerchantHandling {
        site_id: inventory.site_id,
        county_geoid: *b"26163",
        role: MerchantRole::Retail,
        capacity_id,
        labor_unit_id,
    });
    state
        .handling_coefficients
        .push(MerchantHandlingCoefficient {
            site_id: inventory.site_id,
            good_id: inventory.good_id,
            unit_id: inventory.unit_id,
            hours_per_unit: 2,
        });
    state.final_demand_principals.push(FinalDemandPrincipal {
        id: demand_principal_id,
        county_geoid: *b"26163",
    });
    state.final_demand_orders.push(FinalDemandOrder {
        order_id: OrderId::from_bytes([8; 32]),
        retailer_site_id: inventory.site_id,
        demand_principal_id,
        good_id: inventory.good_id,
        unit_id: inventory.unit_id,
        ordered: 10,
        fulfilled: 0,
    });
    state.labor.push(LaborCapacityRow {
        site_id: inventory.site_id,
        unit_id: labor_unit_id,
        period: 1,
        available: 6,
    });
    state.corridor_capacities.push(CorridorCapacity {
        corridor_id: capacity_id,
        period: 1,
        available_grams: 100,
    });
    state.inventory.push(inventory);
    state
}

#[test]
fn retail_final_handoff_has_a_distinct_committed_stock_sink() {
    let committed = pair(retail_state());
    assert_eq!(committed.2.local_fulfillments[0].quantity, 3);
    assert!(committed.2.dispatches.is_empty());
    assert!(committed.2.arrivals.is_empty());
    let balance = complete(&committed);
    let retailer = site_row(&balance, 1);
    assert_eq!(
        (
            retailer.opening,
            retailer.final_demand_fulfilled,
            retailer.closing
        ),
        (10, 3, 7)
    );
    assert_eq!(
        (
            retailer.arrivals,
            retailer.dispatched,
            retailer.local_received,
            retailer.local_transferred
        ),
        (0, 0, 0, 0)
    );
}

#[test]
fn local_internal_transfer_moves_same_native_stock_without_physical_arrival() {
    let mut opening = freight_state(0);
    opening.supplier_routes[0].transport_kind = SupplierTransport::Local;
    opening.route_stages.clear();
    opening.route_stage_capacities.clear();
    opening.corridor_capacities.clear();
    let committed = pair(opening);
    assert_eq!(committed.2.local_transfers[0].quantity, 100);
    assert!(committed.2.dispatches.is_empty());
    assert!(committed.2.arrivals.is_empty());
    assert!(committed.1.freight.is_empty());
    let balance = complete(&committed);
    assert_eq!(
        (
            site_row(&balance, 1).local_transferred,
            site_row(&balance, 1).closing
        ),
        (100, 0)
    );
    assert_eq!(
        (
            site_row(&balance, 4).local_received,
            site_row(&balance, 4).closing
        ),
        (100, 100)
    );
    conserved(&balance);
    for mutate in [
        |pair: &mut Pair| {
            pair.2.local_transfers.clear();
        },
        |pair: &mut Pair| {
            pair.2.local_transfers[0].quantity -= 1;
        },
        |pair: &mut Pair| {
            pair.2.local_transfers[0].buyer_site_id = SiteId::from_bytes([99; 32]);
        },
        |pair: &mut Pair| {
            pair.2
                .local_transfers
                .push(pair.2.local_transfers[0].clone());
        },
        |pair: &mut Pair| {
            pair.1.orders[0].delivered -= 1;
        },
    ] {
        let mut changed = committed.clone();
        mutate(&mut changed);
        refuses_unchanged(&changed, ProductionProjectionError::State);
    }
}

#[test]
fn finite_fulfillment_and_quiet_successor_reconcile_handling_work_stock_and_mass() {
    let catalog = crate::test_support::catalog();
    let mut state = retail_state();
    state.final_demand_orders[0].ordered = 3;
    let foundation =
        super::super::merchants::project_merchants(&catalog, &state, None, None).unwrap();
    assert!(foundation.0[0].completed.is_none());
    assert!(foundation.1[0].completed.is_none());
    let completed = pair(state);
    let balances = complete(&completed);
    conserved(&balances);
    let (handling, demand) = super::super::merchants::project_merchants(
        &catalog,
        &completed.1,
        Some(&completed.0),
        Some(&completed.2),
    )
    .unwrap();
    let work = handling[0].completed.as_ref().unwrap();
    assert_eq!(
        (work.needed_hours, work.used_hours, work.handled_grams),
        (6, 6, 30)
    );
    assert_eq!(
        (
            demand[0].ordered,
            demand[0].fulfilled,
            demand[0].outstanding,
            demand[0].retail_stock_on_hand
        ),
        (3, 3, 0, 7)
    );
    let labor = super::super::labor::project_labor_accounts(
        &completed.1,
        Some(&completed.0),
        Some(&completed.2),
    )
    .unwrap();
    let time = labor[0].completed.as_ref().unwrap();
    assert_eq!(
        (
            time.planned,
            time.handling_needed,
            time.handling_used,
            time.used,
            time.unused
        ),
        (0, 6, 6, 6, 0)
    );
    let capacity = super::super::freight::project_freight_capacity_accounts(
        &catalog,
        &completed.1,
        Some(&completed.0),
        Some(&completed.2),
    )
    .unwrap();
    let reservation = &capacity[0].completed.as_ref().unwrap().reservations[0];
    assert_eq!(
        (
            reservation.opening_available_grams,
            reservation.newly_reserved_grams,
            reservation.remaining_available_grams
        ),
        (100, 30, 70)
    );
    assert!(reservation.orders[0].route_id.is_none());
    let quiet = pair(completed.1);
    let (handling, demand) = super::super::merchants::project_merchants(
        &catalog,
        &quiet.1,
        Some(&quiet.0),
        Some(&quiet.2),
    )
    .unwrap();
    assert_eq!(handling[0].completed.as_ref().unwrap().used_hours, 0);
    assert_eq!(demand[0].completed.as_ref().unwrap().newly_fulfilled, 0);
    assert_eq!(demand[0].retail_stock_on_hand, 7);
    assert_eq!(site_row(&complete(&quiet), 1).final_demand_fulfilled, 0);
    conserved(&complete(&quiet));
}

#[test]
fn completed_local_fulfillment_and_handling_require_exact_receipt_identity() {
    let catalog = crate::test_support::catalog();
    let committed = pair(retail_state());
    for mutate in [
        |pair: &mut Pair| {
            pair.2.local_fulfillments.clear();
        },
        |pair: &mut Pair| {
            pair.2.local_fulfillments[0].quantity -= 1;
        },
        |pair: &mut Pair| {
            pair.2.local_fulfillments[0].unit_id = UnitId::from_bytes([99; 32]);
        },
        |pair: &mut Pair| {
            pair.2
                .local_fulfillments
                .push(pair.2.local_fulfillments[0].clone());
        },
        |pair: &mut Pair| {
            pair.1.final_demand_orders[0].fulfilled += 1;
        },
    ] {
        let mut changed = committed.clone();
        mutate(&mut changed);
        refuses_unchanged(&changed, ProductionProjectionError::State);
    }
    for mutate in [
        |pair: &mut Pair| {
            pair.2.handling.clear();
        },
        |pair: &mut Pair| {
            pair.2.handling[0].needed_hours += 1;
        },
        |pair: &mut Pair| {
            pair.2.handling[0].used_hours += 1;
        },
        |pair: &mut Pair| {
            pair.2.handling[0].handled_quantity += 1;
        },
        |pair: &mut Pair| {
            pair.2.handling[0].site_id = SiteId::from_bytes([99; 32]);
        },
        |pair: &mut Pair| {
            pair.2.handling.push(pair.2.handling[0].clone());
        },
    ] {
        let mut changed = committed.clone();
        mutate(&mut changed);
        assert!(matches!(
            super::super::merchants::project_merchants(
                &catalog,
                &changed.1,
                Some(&changed.0),
                Some(&changed.2)
            ),
            Err(ProductionProjectionError::State)
        ));
    }
}
