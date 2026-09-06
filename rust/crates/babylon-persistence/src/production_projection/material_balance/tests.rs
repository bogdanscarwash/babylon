use babylon_material_circuit::{
    BacklogRowV1, CapacityRowV1, CorridorCapacityV2, CorridorIdV2, InputOutputCoefficientV1,
    InventoryRowV1, LaborCapacityRowV1, LaborCoefficientV1, LogisticsNodeIdV2, OrderAccessModeV1,
    ProcessOutputV1, ProductionCommitmentV1, RouteIdV2, RouteLegV2, SiteLogisticsNodeV2,
    SupplierRouteV2,
};
use babylon_tick::material_world::{decode_material_receipts_v3, MaterialWorldRegisterV2};

use super::*;
use crate::michigan_material::{michigan_material_foundation_v1, MichiganDeliveryPresetV1};

type Pair = (
    MaterialCircuitStateV2,
    MaterialCircuitStateV2,
    MaterialTickReceiptsV3,
);

fn empty_state() -> MaterialCircuitStateV2 {
    MaterialCircuitStateV2 {
        week: 1,
        site_logistics_nodes: vec![],
        process_outputs: vec![],
        input_coefficients: vec![],
        labor_coefficients: vec![],
        supplier_routes: vec![],
        route_legs: vec![],
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

fn pair(state: MaterialCircuitStateV2) -> Pair {
    let opening = MaterialWorldRegisterV2::try_new(state.week - 1, state).unwrap();
    let next = opening.prepare_next().unwrap();
    (
        opening.state().clone(),
        next.register().state().clone(),
        decode_material_receipts_v3(next.receipt_bytes()).unwrap(),
    )
}

fn project(pair: &Pair) -> Result<Option<CompletedMaterialBalanceV1>, ProductionProjectionErrorV1> {
    project_with_labels(&pair.1, Some(&pair.0), Some(&pair.2), |good, unit| {
        Some((digest_hex(&good.as_bytes()), digest_hex(&unit.as_bytes())))
    })
}

fn complete(pair: &Pair) -> CompletedMaterialBalanceV1 {
    project(pair).unwrap().unwrap()
}

fn conserved(balance: &CompletedMaterialBalanceV1) {
    for row in &balance.rows {
        assert_eq!(
            u128::from(row.opening) + u128::from(row.arrivals) + u128::from(row.produced),
            u128::from(row.consumed) + u128::from(row.dispatched) + u128::from(row.closing),
            "{row:?}",
        );
    }
}

fn stock(site: u8, good: u8, unit: u8, quantity: u64) -> InventoryRowV1 {
    InventoryRowV1 {
        site_id: SiteIdV1::from_bytes([site; 32]),
        good_id: GoodIdV1::from_bytes([good; 32]),
        unit_id: UnitIdV1::from_bytes([unit; 32]),
        quantity,
    }
}

fn production_state(specs: &[(u8, u64, u64, u64)], opening: u64) -> MaterialCircuitStateV2 {
    let mut state = empty_state();
    let inventory = stock(1, 2, 3, opening);
    let mut labor = 0_u64;
    for &(id, input, output, batches) in specs {
        let process_id = ProcessIdV1::from_bytes([id; 32]);
        state.process_outputs.push(ProcessOutputV1 {
            process_id,
            site_id: inventory.site_id,
            good_id: inventory.good_id,
            unit_id: inventory.unit_id,
            quantity_per_batch: output,
        });
        state.input_coefficients.push(InputOutputCoefficientV1 {
            process_id,
            good_id: inventory.good_id,
            unit_id: inventory.unit_id,
            quantity_per_batch: input,
        });
        state.labor_coefficients.push(LaborCoefficientV1 {
            process_id,
            unit_id: UnitIdV1::from_bytes([4; 32]),
            quantity_per_batch: 1,
        });
        state.capacities.push(CapacityRowV1 {
            process_id,
            site_id: inventory.site_id,
            week: 1,
            available_batches: batches,
        });
        state.production_commitments.push(ProductionCommitmentV1 {
            process_id,
            site_id: inventory.site_id,
            week: 1,
            planned_batches: batches,
        });
        labor = labor.checked_add(batches).unwrap();
    }
    state.labor.push(LaborCapacityRowV1 {
        site_id: inventory.site_id,
        unit_id: UnitIdV1::from_bytes([4; 32]),
        week: 1,
        available: labor,
    });
    state.inventory.push(inventory);
    state
}

#[test]
fn shared_process_principal_records_production_and_consumption_separately_once() {
    let pair = pair(production_state(&[(10, 2, 3, 2), (11, 1, 2, 3)], 10));
    let balance = complete(&pair);
    assert_eq!(balance.week, 1);
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

fn freight_state(loss_ppm: u32) -> MaterialCircuitStateV2 {
    let mut state = empty_state();
    let inventory = stock(1, 2, 3, 100);
    let buyer = SiteIdV1::from_bytes([4; 32]);
    let source = LogisticsNodeIdV2::from_bytes([5; 32]);
    let destination = LogisticsNodeIdV2::from_bytes([6; 32]);
    let route = RouteIdV2::from_bytes([7; 32]);
    let corridor = CorridorIdV2::from_bytes([8; 32]);
    let order = OrderIdV1::from_bytes([9; 32]);
    state.site_logistics_nodes = vec![
        SiteLogisticsNodeV2 {
            site_id: inventory.site_id,
            node_id: source,
        },
        SiteLogisticsNodeV2 {
            site_id: buyer,
            node_id: destination,
        },
    ];
    state.supplier_routes.push(SupplierRouteV2 {
        buyer_site_id: buyer,
        supplier_site_id: inventory.site_id,
        good_id: inventory.good_id,
        unit_id: inventory.unit_id,
        route_id: route,
    });
    state.route_legs.push(RouteLegV2 {
        route_id: route,
        leg_index: 0,
        corridor_id: corridor,
        from_node_id: source,
        to_node_id: destination,
        travel_weeks: 1,
        loss_ppm,
    });
    state.corridor_capacities.push(CorridorCapacityV2 {
        corridor_id: corridor,
        unit_id: inventory.unit_id,
        week: 1,
        available: 100,
    });
    state.orders.push(OrderRowV2 {
        order_id: order,
        access_mode: OrderAccessModeV1::CommoditySale,
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
    state.backlog.push(BacklogRowV1 {
        order_id: order,
        quantity: 100,
    });
    state.inventory.push(inventory);
    state
}

fn site_row(balance: &CompletedMaterialBalanceV1, site: u8) -> &ProductionMaterialBalanceRowV1 {
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

fn two_leg_work_state() -> MaterialCircuitStateV2 {
    let mut state = freight_state(250_000);
    let destination = state.route_legs[0].to_node_id;
    let intermediate = LogisticsNodeIdV2::from_bytes([10; 32]);
    state.route_legs[0].to_node_id = intermediate;
    state.route_legs.push(RouteLegV2 {
        route_id: state.route_legs[0].route_id,
        leg_index: 1,
        corridor_id: CorridorIdV2::from_bytes([11; 32]),
        from_node_id: intermediate,
        to_node_id: destination,
        travel_weeks: 1,
        loss_ppm: 0,
    });
    // The actual V2 dispatcher reserves the full dispatched quantity on every
    // leg, without anticipating the loss that will later occur in transit.
    state.corridor_capacities.push(CorridorCapacityV2 {
        corridor_id: state.route_legs[1].corridor_id,
        unit_id: state.inventory[0].unit_id,
        week: 2,
        available: 100,
    });
    add_receiving_work(&mut state);
    add_receiving_dispatch(&mut state);
    state
}

fn add_receiving_work(state: &mut MaterialCircuitStateV2) {
    let buyer = state.orders[0].buyer_site_id;
    let mut work = production_state(&[(17, 1, 2, 1)], 5);
    work.process_outputs[0].site_id = buyer;
    work.inventory[0].site_id = buyer;
    work.capacities[0].site_id = buyer;
    work.labor[0].site_id = buyer;
    work.production_commitments[0].site_id = buyer;
    for week in [2, 3] {
        work.capacities.push(CapacityRowV1 {
            week,
            ..work.capacities[0].clone()
        });
        work.labor.push(LaborCapacityRowV1 {
            week,
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

fn add_receiving_dispatch(state: &mut MaterialCircuitStateV2) {
    let supplier = state.orders[0].buyer_site_id;
    let buyer = SiteIdV1::from_bytes([15; 32]);
    let destination = LogisticsNodeIdV2::from_bytes([14; 32]);
    let route = RouteIdV2::from_bytes([12; 32]);
    let corridor = CorridorIdV2::from_bytes([13; 32]);
    let order = OrderIdV1::from_bytes([16; 32]);
    state.site_logistics_nodes.push(SiteLogisticsNodeV2 {
        site_id: buyer,
        node_id: destination,
    });
    state.route_legs.push(RouteLegV2 {
        route_id: route,
        leg_index: 0,
        corridor_id: corridor,
        from_node_id: state.route_legs[1].to_node_id,
        to_node_id: destination,
        travel_weeks: 1,
        loss_ppm: 0,
    });
    state.supplier_routes.push(SupplierRouteV2 {
        supplier_site_id: supplier,
        buyer_site_id: buyer,
        route_id: route,
        ..state.supplier_routes[0].clone()
    });
    state.orders.push(OrderRowV2 {
        order_id: order,
        supplier_site_id: supplier,
        buyer_site_id: buyer,
        ordered: 4,
        ..state.orders[0].clone()
    });
    state.backlog.push(BacklogRowV1 {
        order_id: order,
        quantity: 4,
    });
    state.corridor_capacities.push(CorridorCapacityV2 {
        corridor_id: corridor,
        unit_id: state.orders[0].unit_id,
        week: 3,
        available: 4,
    });
}

fn assert_intermediate_transit_loss(intermediate: &Pair) {
    assert_eq!(intermediate.2.losses.len(), 1);
    assert_eq!(intermediate.2.losses[0].quantity, 25);
    assert!(intermediate.2.arrivals.is_empty());
    assert!(intermediate.2.deliveries.is_empty());
    assert!(intermediate.2.realizations.is_empty());
    assert_eq!(intermediate.1.freight.len(), 1);
    assert_eq!(intermediate.1.freight[0].current_leg_index, 1);
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
    assert_eq!(dispatched.2.dispatches[0].final_arrival_week, 3);
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
        digest_hex(&UnitIdV1::from_bytes([3; 32]).as_bytes())
    );
    assert_eq!(arrival.1.freight.len(), 1);
    assert_eq!(
        arrival.1.freight[0].source_site_id,
        SiteIdV1::from_bytes([4; 32])
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

fn michigan_week(preset: MichiganDeliveryPresetV1, week: u64) -> Pair {
    let mut state = michigan_material_foundation_v1(preset).unwrap();
    for _ in 1..week {
        state = pair(state).1;
    }
    pair(state)
}

#[test]
fn delivery_twins_explain_downstream_input_use_and_preserve_unrelated_food() {
    let standard = michigan_week(MichiganDeliveryPresetV1::Standard, 3);
    let delayed = michigan_week(MichiganDeliveryPresetV1::Delayed, 3);
    let a = project_material_balance(&standard.1, Some(&standard.0), Some(&standard.2))
        .unwrap()
        .unwrap();
    let b = project_material_balance(&delayed.1, Some(&delayed.0), Some(&delayed.2))
        .unwrap()
        .unwrap();
    let catalog = michigan_material_catalog_v1().unwrap();
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
    let later = pair(standard.1.clone());
    conserved(&complete(&later));
    assert_eq!(standard, original);
    assert_eq!(
        project_material_balance(&standard.1, Some(&standard.0), Some(&standard.2)).unwrap(),
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

fn refuses_unchanged(pair: &Pair, error: ProductionProjectionErrorV1) {
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
    refuses_unchanged(&multiplication, ProductionProjectionErrorV1::Arithmetic);
    let mut addition = pair(production_state(&[(10, 1, 1, 1), (11, 1, 1, 1)], 10));
    for state in [&mut addition.0, &mut addition.1] {
        for output in &mut state.process_outputs {
            output.quantity_per_batch = u64::MAX / 2 + 1;
        }
    }
    refuses_unchanged(&addition, ProductionProjectionErrorV1::Arithmetic);
    let mut arrivals = pair(pair(freight_state(0)).1);
    arrivals.2.arrivals[0].quantity = u64::MAX;
    arrivals.2.arrivals.push(arrivals.2.arrivals[0].clone());
    refuses_unchanged(&arrivals, ProductionProjectionErrorV1::Arithmetic);
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
            Err(ProductionProjectionErrorV1::History)
        );
    }
    let mut wrong = actual.clone();
    wrong.2.resolve_tick += 1;
    refuses_unchanged(&wrong, ProductionProjectionErrorV1::History);
    let mut wrong = actual;
    wrong.1.week += 1;
    refuses_unchanged(&wrong, ProductionProjectionErrorV1::History);
}

#[test]
fn production_family_and_recipe_or_inventory_mismatches_refuse() {
    let actual = pair(production_state(&[(10, 1, 1, 2)], 10));
    for case in 0..9 {
        let mut changed = actual.clone();
        match case {
            0 => changed.2.production.clear(),
            1 => changed.2.production.push(changed.2.production[0].clone()),
            2 => changed.2.production[0].site_id = SiteIdV1::from_bytes([99; 32]),
            3 => changed.2.production[0].process_id = ProcessIdV1::from_bytes([99; 32]),
            4 => changed.2.production[0].planned_batches += 1,
            5 => changed.1.process_outputs[0].quantity_per_batch += 1,
            6 => changed.1.input_coefficients[0].quantity_per_batch += 1,
            7 => changed.1.inventory[0].quantity += 1,
            _ => changed.0.inventory.push(changed.0.inventory[0].clone()),
        }
        refuses_unchanged(&changed, ProductionProjectionErrorV1::State);
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
            2 => changed.2.dispatches[0].route_id = RouteIdV2::from_bytes([99; 32]),
            3 => changed.2.dispatches[0].quantity -= 1,
            _ => changed.1.orders[0].unit_id = UnitIdV1::from_bytes([99; 32]),
        }
        refuses_unchanged(&changed, ProductionProjectionErrorV1::State);
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
            4 => changed.2.losses[0].corridor_id = CorridorIdV2::from_bytes([99; 32]),
            5 => changed.2.losses.push(changed.2.losses[0].clone()),
            6 => changed.2.arrivals[0].order_id = OrderIdV1::from_bytes([99; 32]),
            7 => changed.2.deliveries[0].quantity += 1,
            _ => changed.2.arrivals[0].quantity = 0,
        }
        refuses_unchanged(&changed, ProductionProjectionErrorV1::State);
    }
}

#[test]
fn unknown_unit_metadata_refuses_instead_of_inventing_a_label() {
    let mut state = empty_state();
    state.inventory.push(stock(1, 2, 3, 9));
    let (prior, current, receipt) = pair(state);
    assert_eq!(
        project_with_labels(&current, Some(&prior), Some(&receipt), |_, _| None),
        Err(ProductionProjectionErrorV1::Content)
    );
    assert_eq!(
        project_material_balance(&current, Some(&prior), Some(&receipt)),
        Err(ProductionProjectionErrorV1::Content)
    );
}
