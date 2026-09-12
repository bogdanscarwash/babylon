use babylon_material_circuit::SupplierTransport;
use babylon_material_circuit::{
    advance_material_circuit, BacklogRow, CorridorCapacity, CorridorId, FreightMassCoefficient,
    GoodId, InventoryRow, LaborCapacityRow, LogisticsNodeId, MaterialCircuitState,
    MerchantHandling, MerchantHandlingCoefficient, MerchantRole, OrderAccessMode, OrderId,
    OrderRow, RouteId, RouteStage, RouteStageCapacity, SiteId, SiteLogisticsNode, SupplierRoute,
    UnitId,
};

fn merchant() -> MaterialCircuitState {
    let source = SiteId::from_bytes([1; 32]);
    let buyer = SiteId::from_bytes([2; 32]);
    let good_id = GoodId::from_bytes([3; 32]);
    let unit_id = UnitId::from_bytes([4; 32]);
    let route_id = RouteId::from_bytes([5; 32]);
    let corridor_id = CorridorId::from_bytes([6; 32]);
    let order_id = OrderId::from_bytes([7; 32]);
    let source_node = LogisticsNodeId::from_bytes([1; 32]);
    let buyer_node = LogisticsNodeId::from_bytes([2; 32]);
    MaterialCircuitState {
        merchants: vec![MerchantHandling {
            site_id: source,
            county_geoid: *b"26163",
            role: MerchantRole::Retail,
            capacity_id: CorridorId::from_bytes([9; 32]),
            labor_unit_id: UnitId::from_bytes([8; 32]),
        }],
        handling_coefficients: vec![MerchantHandlingCoefficient {
            site_id: source,
            good_id,
            unit_id,
            hours_per_unit: 2,
        }],
        final_demand_principals: vec![],
        final_demand_orders: vec![],
        period: 1,
        site_logistics_nodes: vec![
            SiteLogisticsNode {
                site_id: source,
                node_id: source_node,
            },
            SiteLogisticsNode {
                site_id: buyer,
                node_id: buyer_node,
            },
        ],
        process_outputs: vec![],
        input_coefficients: vec![],
        labor_coefficients: vec![],
        freight_mass_coefficients: vec![FreightMassCoefficient {
            good_id,
            unit_id,
            grams_per_unit: 10,
        }],
        supplier_routes: vec![SupplierRoute {
            transport_kind: SupplierTransport::Staged,
            buyer_site_id: buyer,
            supplier_site_id: source,
            good_id,
            unit_id,
            route_id,
        }],
        route_stages: vec![RouteStage {
            route_id,
            stage_index: 0,
            from_node_id: source_node,
            to_node_id: buyer_node,
            travel_periods: 1,
            loss_ppm: 0,
        }],
        route_stage_capacities: vec![RouteStageCapacity {
            route_id,
            stage_index: 0,
            corridor_id,
        }],
        inventory: vec![InventoryRow {
            site_id: source,
            good_id,
            unit_id,
            quantity: 10,
        }],
        orders: vec![OrderRow {
            order_id,
            access_mode: OrderAccessMode::CommoditySale,
            buyer_site_id: buyer,
            supplier_site_id: source,
            good_id,
            unit_id,
            ordered: 10,
            shipped: 0,
            lost: 0,
            delivered: 0,
            realized: 0,
        }],
        backlog: vec![BacklogRow {
            order_id,
            quantity: 10,
        }],
        freight: vec![],
        corridor_capacities: vec![
            CorridorCapacity {
                corridor_id: CorridorId::from_bytes([9; 32]),
                period: 1,
                available_grams: 100,
            },
            CorridorCapacity {
                corridor_id,
                period: 1,
                available_grams: 100,
            },
        ],
        capacities: vec![],
        labor: vec![LaborCapacityRow {
            site_id: source,
            unit_id: UnitId::from_bytes([8; 32]),
            period: 1,
            available: 4,
        }],
        production_commitments: vec![],
    }
}

#[test]
fn outbound_merchant_handling_requires_two_hours_per_native_unit() {
    let opening = merchant();
    let result = advance_material_circuit(&opening).unwrap();
    assert_eq!(result.dispatches[0].quantity, 2);
}

use babylon_material_circuit::{
    advance_staffing, close_material_period, decode_material_circuit_state,
    encode_material_circuit_state, material_circuit_state_digest, FinalDemandOrder,
    FinalDemandPrincipal, FinalDemandPrincipalId, MaterialCircuitError, OutboundOrderId,
    StaffingPolicy, StaffingPoolBinding, StaffingPoolId, StaffingPoolState, StaffingState,
    StaffingWorkSource,
};

fn add_local_order(state: &mut MaterialCircuitState) {
    let store = &state.merchants[0];
    let coefficient = &state.handling_coefficients[0];
    let principal = FinalDemandPrincipalId::from_bytes([10; 32]);
    state.final_demand_principals.push(FinalDemandPrincipal {
        id: principal,
        county_geoid: store.county_geoid,
    });
    state.final_demand_orders.push(FinalDemandOrder {
        order_id: OrderId::from_bytes([11; 32]),
        retailer_site_id: store.site_id,
        demand_principal_id: principal,
        good_id: coefficient.good_id,
        unit_id: coefficient.unit_id,
        ordered: 10,
        fulfilled: 0,
    });
}

fn local_store() -> MaterialCircuitState {
    let mut state = merchant();
    add_local_order(&mut state);
    state.orders.clear();
    state.backlog.clear();
    state.supplier_routes.clear();
    state.route_stages.clear();
    state.route_stage_capacities.clear();
    let capacity = state.merchants[0].capacity_id;
    state
        .corridor_capacities
        .retain(|row| row.corridor_id == capacity);
    state
}

fn schedule(state: &mut MaterialCircuitState, periods: u64, hours: u64) {
    let merchant = state.merchants[0].clone();
    state.labor = (1..=periods)
        .map(|period| LaborCapacityRow {
            site_id: merchant.site_id,
            unit_id: merchant.labor_unit_id,
            period,
            available: hours,
        })
        .collect();
    state
        .corridor_capacities
        .extend((2..=periods).map(|period| CorridorCapacity {
            corridor_id: merchant.capacity_id,
            period,
            available_grams: 100,
        }));
}

#[test]
fn routed_and_local_orders_share_stock_handling_mass_and_opening_labor_once() {
    let mut state = merchant();
    add_local_order(&mut state);
    let completed = advance_material_circuit(&state).unwrap();
    assert_eq!(completed.dispatches[0].quantity, 1);
    assert_eq!(completed.local_fulfillments[0].quantity, 1);
    assert_eq!(completed.handling.len(), 2);
    for receipt in &completed.handling {
        assert_eq!(
            (receipt.feasible_quantity, receipt.handled_quantity),
            (5, 1)
        );
        assert_eq!((receipt.needed_hours, receipt.used_hours), (10, 2));
    }
    assert_eq!(completed.state.inventory[0].quantity, 8);
    assert_eq!(completed.state.freight[0].quantity, 1);
    assert_eq!(completed.state.orders[0].shipped, 1);
    assert_eq!(completed.state.backlog[0].quantity, 9);
    assert_eq!(completed.state.final_demand_orders[0].fulfilled, 1);
    assert!(completed.arrivals.is_empty());
    assert!(completed.deliveries.is_empty());
    assert!(completed.realizations.is_empty());
}

#[test]
fn road_restriction_changes_feasible_need_and_labor_floor_leaves_residual_hours() {
    let mut state = merchant();
    add_local_order(&mut state);
    let road = state.route_stage_capacities[0].corridor_id;
    state
        .corridor_capacities
        .iter_mut()
        .find(|row| row.corridor_id == road)
        .unwrap()
        .available_grams = 20;
    let completed = advance_material_circuit(&state).unwrap();
    assert!(completed.dispatches.is_empty());
    assert_eq!(completed.local_fulfillments[0].quantity, 1);
    assert_eq!(
        completed
            .handling
            .iter()
            .map(|row| (
                row.feasible_quantity,
                row.handled_quantity,
                row.needed_hours,
                row.used_hours
            ))
            .collect::<Vec<_>>(),
        vec![(2, 0, 4, 0), (5, 1, 10, 2)]
    );
    assert_eq!(completed.state.inventory[0].quantity, 9);
}

#[test]
fn local_handoff_consumes_handling_mass_without_a_road_or_freight_lot() {
    let mut state = local_store();
    state.labor[0].available = 100;
    state.corridor_capacities[0].available_grams = 30;
    let completed = advance_material_circuit(&state).unwrap();
    assert_eq!(completed.local_fulfillments[0].quantity, 3);
    assert_eq!(completed.handling[0].needed_hours, 6);
    assert_eq!(completed.handling[0].used_hours, 6);
    assert_eq!(completed.state.inventory[0].quantity, 7);
    assert!(completed.state.freight.is_empty());
    assert!(completed.dispatches.is_empty());
    assert!(completed.production.is_empty());
    assert_eq!(
        completed.state.inventory[0].good_id,
        state.inventory[0].good_id
    );
}

#[test]
fn finite_local_orders_credit_once_and_finished_orders_report_completed_zero() {
    let mut state = local_store();
    schedule(&mut state, 6, 4);
    let mut handed_off = 0;
    for period in 1..=6 {
        assert_eq!(state.period, period);
        let completed = advance_material_circuit(&state).unwrap();
        handed_off += completed
            .local_fulfillments
            .iter()
            .map(|row| row.quantity)
            .sum::<u64>();
        let stock = completed
            .state
            .inventory
            .iter()
            .map(|row| row.quantity)
            .sum::<u64>();
        assert_eq!(stock + handed_off, 10);
        assert_eq!(completed.state.final_demand_orders[0].fulfilled, handed_off);
        if period == 6 {
            assert!(completed.local_fulfillments.is_empty());
            assert_eq!(completed.handling.len(), 1);
            assert_eq!(
                (
                    completed.handling[0].feasible_quantity,
                    completed.handling[0].handled_quantity,
                    completed.handling[0].needed_hours,
                    completed.handling[0].used_hours
                ),
                (0, 0, 0, 0)
            );
        }
        let bytes = encode_material_circuit_state(&completed.state).unwrap();
        state = decode_material_circuit_state(&bytes).unwrap();
    }
    assert_eq!(handed_off, 10);
}

fn staffing_binding(state: &MaterialCircuitState) -> StaffingPoolBinding {
    let merchant = &state.merchants[0];
    StaffingPoolBinding::try_new(
        StaffingPoolId::from_bytes([12; 32]),
        merchant.site_id,
        merchant.labor_unit_id,
        10,
        StaffingPolicy::one_period(2).unwrap(),
        vec![StaffingWorkSource::MerchantHandling(merchant.site_id)],
    )
    .unwrap()
}

#[test]
fn zero_employment_can_rehire_from_feasible_handling_need_without_current_dispatch() {
    let mut state = local_store();
    schedule(&mut state, 2, 0);
    let binding = staffing_binding(&state);
    let opening_staffing = StaffingState::try_new(
        1,
        vec![StaffingPoolState::try_new(binding.clone(), 0, 10, 0).unwrap()],
    )
    .unwrap();
    let closed = close_material_period(&state).unwrap();
    let requests = closed
        .staffing_requests(std::slice::from_ref(&binding))
        .unwrap();
    assert_eq!(
        requests[0].work_source(),
        StaffingWorkSource::MerchantHandling(binding.site_id())
    );
    assert_eq!(requests[0].hours(), 20);
    let staffing = advance_staffing(&opening_staffing, &requests).unwrap();
    assert_eq!(
        (
            staffing.receipts()[0].hires(),
            staffing.receipts()[0].closing_employed(),
            staffing.receipts()[0].closing_reserve()
        ),
        (10, 10, 0)
    );
    let completed = closed
        .finish_with_labor(staffing.next_labor().to_vec())
        .unwrap();
    assert!(completed.local_fulfillments.is_empty());
    assert_eq!(
        (
            completed.handling[0].needed_hours,
            completed.handling[0].used_hours
        ),
        (20, 0)
    );
    let next = advance_material_circuit(&completed.state).unwrap();
    assert_eq!(next.local_fulfillments[0].quantity, 10);
}

#[test]
fn empty_inventory_reports_zero_feasible_handling_and_staffing_need() {
    let mut state = local_store();
    state.inventory.clear();
    let closed = close_material_period(&state).unwrap();
    assert_eq!(
        closed
            .staffing_requests(&[staffing_binding(&state)])
            .unwrap()[0]
            .hours(),
        0
    );
    let result = advance_material_circuit(&state).unwrap();
    assert_eq!(
        (
            result.handling[0].needed_hours,
            result.handling[0].used_hours
        ),
        (0, 0)
    );
    assert!(result.local_fulfillments.is_empty());
}

#[test]
fn arrived_goods_are_available_to_retail_handling_in_the_same_close() {
    let mut state = merchant();
    add_local_order(&mut state);
    schedule(&mut state, 2, 4);
    let incoming_source = state.orders[0].buyer_site_id;
    let retailer = state.orders[0].supplier_site_id;
    state.inventory[0].site_id = incoming_source;
    state.orders[0].buyer_site_id = retailer;
    state.orders[0].supplier_site_id = incoming_source;
    state.supplier_routes[0].buyer_site_id = retailer;
    state.supplier_routes[0].supplier_site_id = incoming_source;
    let stage = &mut state.route_stages[0];
    std::mem::swap(&mut stage.from_node_id, &mut stage.to_node_id);
    let first = advance_material_circuit(&state).unwrap();
    assert_eq!(first.dispatches[0].quantity, 10);
    assert_eq!(first.handling[0].needed_hours, 0);
    let arrived = advance_material_circuit(&first.state).unwrap();
    assert_eq!(arrived.arrivals[0].quantity, 10);
    assert_eq!(arrived.local_fulfillments[0].quantity, 2);
    assert_eq!(
        (
            arrived.handling[0].needed_hours,
            arrived.handling[0].used_hours
        ),
        (20, 4)
    );
    assert_eq!(
        arrived
            .state
            .inventory
            .iter()
            .map(|row| row.quantity)
            .sum::<u64>(),
        8
    );
}

#[test]
fn merchant_admission_refuses_invalid_coefficients_county_role_and_capacity_ownership() {
    let base = local_store();
    let mutations: &[fn(&mut MaterialCircuitState)] = &[
        |s| s.handling_coefficients[0].hours_per_unit = 0,
        |s| s.handling_coefficients.clear(),
        |s| s.final_demand_orders[0].fulfilled = 11,
        |s| s.final_demand_principals[0].county_geoid = *b"26001",
        |s| s.merchants[0].role = MerchantRole::Wholesale,
        |s| s.merchants.push(s.merchants[0].clone()),
        |s| s.final_demand_orders.push(s.final_demand_orders[0].clone()),
        |s| s.final_demand_principals[0].county_geoid = *b"26bad",
    ];
    for mutate in mutations {
        let mut malformed = base.clone();
        mutate(&mut malformed);
        let preserved = malformed.clone();
        assert!(advance_material_circuit(&malformed).is_err());
        assert_eq!(malformed, preserved);
    }
    let mut overlap = merchant();
    overlap.merchants[0].capacity_id = overlap.route_stage_capacities[0].corridor_id;
    assert_eq!(
        advance_material_circuit(&overlap),
        Err(MaterialCircuitError::MerchantInvariant)
    );
}

#[test]
fn handling_hour_overflow_refuses_before_any_successor_or_local_receipt_escapes() {
    let mut state = local_store();
    state.handling_coefficients[0].hours_per_unit = u64::MAX;
    let preserved = state.clone();
    assert_eq!(
        advance_material_circuit(&state),
        Err(MaterialCircuitError::Arithmetic)
    );
    assert_eq!(state, preserved);
    for code in 20..=21 {
        assert_eq!(
            u16::from(MaterialCircuitError::try_from(code).unwrap()),
            code
        );
    }
}

#[test]
fn new_families_are_canonical_hash_bound_and_local_routed_ids_are_disjoint() {
    let mut state = merchant();
    add_local_order(&mut state);
    state.final_demand_orders[0].order_id = state.orders[0].order_id;
    let mut duplicate_good = state.handling_coefficients[0].clone();
    duplicate_good.good_id = GoodId::from_bytes([99; 32]);
    state.handling_coefficients.push(duplicate_good.clone());
    state
        .freight_mass_coefficients
        .push(FreightMassCoefficient {
            good_id: duplicate_good.good_id,
            unit_id: duplicate_good.unit_id,
            grams_per_unit: 10,
        });
    let original = encode_material_circuit_state(&state).unwrap();
    let completed = advance_material_circuit(&state).unwrap();
    assert_eq!(
        completed.handling[0].order,
        OutboundOrderId::Delivery(state.orders[0].order_id)
    );
    assert_eq!(
        completed.handling[1].order,
        OutboundOrderId::LocalFinalDemand(state.orders[0].order_id)
    );
    let mut permuted = state.clone();
    permuted.merchants.reverse();
    permuted.handling_coefficients.reverse();
    permuted.final_demand_principals.reverse();
    permuted.final_demand_orders.reverse();
    permuted.freight_mass_coefficients.reverse();
    permuted.corridor_capacities.reverse();
    permuted.site_logistics_nodes.reverse();
    assert_eq!(encode_material_circuit_state(&permuted).unwrap(), original);
    assert_eq!(advance_material_circuit(&permuted).unwrap(), completed);
    let baseline = material_circuit_state_digest(&state).unwrap();
    let mutations: &[fn(&mut MaterialCircuitState)] = &[
        |s| {
            s.merchants[0].county_geoid = *b"26001";
            s.final_demand_principals[0].county_geoid = *b"26001";
        },
        |s| s.handling_coefficients[0].hours_per_unit += 1,
        |s| {
            let id = FinalDemandPrincipalId::from_bytes([90; 32]);
            s.final_demand_principals[0].id = id;
            s.final_demand_orders[0].demand_principal_id = id;
        },
        |s| s.final_demand_orders[0].ordered += 1,
        |s| s.final_demand_orders[0].fulfilled += 1,
    ];
    for mutate in mutations {
        let mut changed = state.clone();
        mutate(&mut changed);
        assert_ne!(material_circuit_state_digest(&changed).unwrap(), baseline);
    }
}

#[test]
fn multiple_merchants_and_county_accounts_preserve_identity_under_all_new_row_permutations() {
    let mut state = local_store();
    let second_site = state.site_logistics_nodes[1].site_id;
    let mut second_store = state.merchants[0].clone();
    second_store.site_id = second_site;
    second_store.county_geoid = *b"26001";
    second_store.capacity_id = CorridorId::from_bytes([20; 32]);
    let mut coefficient = state.handling_coefficients[0].clone();
    coefficient.site_id = second_site;
    coefficient.hours_per_unit = 3;
    let mut stock = state.inventory[0].clone();
    stock.site_id = second_site;
    stock.quantity = 6;
    let mut labor = state.labor[0].clone();
    labor.site_id = second_site;
    labor.available = 9;
    let mut demand = state.final_demand_principals[0].clone();
    demand.id = FinalDemandPrincipalId::from_bytes([21; 32]);
    demand.county_geoid = second_store.county_geoid;
    let mut order = state.final_demand_orders[0].clone();
    order.order_id = OrderId::from_bytes([22; 32]);
    order.retailer_site_id = second_site;
    order.demand_principal_id = demand.id;
    order.ordered = 4;
    state.corridor_capacities.push(CorridorCapacity {
        corridor_id: second_store.capacity_id,
        period: 1,
        available_grams: 100,
    });
    state.merchants.push(second_store);
    state.handling_coefficients.push(coefficient);
    state.inventory.push(stock);
    state.labor.push(labor);
    state.final_demand_principals.push(demand);
    state.final_demand_orders.push(order);
    let bytes = encode_material_circuit_state(&state).unwrap();
    let expected = advance_material_circuit(&state).unwrap();
    assert_eq!(
        expected
            .local_fulfillments
            .iter()
            .map(|row| row.quantity)
            .collect::<Vec<_>>(),
        vec![2, 3]
    );
    state.merchants.reverse();
    state.handling_coefficients.reverse();
    state.final_demand_principals.reverse();
    state.final_demand_orders.reverse();
    state.inventory.reverse();
    state.labor.reverse();
    state.corridor_capacities.reverse();
    assert_eq!(encode_material_circuit_state(&state).unwrap(), bytes);
    assert_eq!(advance_material_circuit(&state).unwrap(), expected);
}

#[test]
fn merchant_role_encoding_rejects_unknown_tags() {
    let state = local_store();
    let mut bytes = encode_material_circuit_state(&state).unwrap();
    // Four appended families: counts plus one merchant(102), coefficient(104),
    // county principal(37), and local order(176). Role follows site and county.
    let role_offset = bytes.len() - (4 * 4 + 102 + 104 + 37 + 176) + 4 + 32 + 5;
    assert_eq!(bytes[role_offset], 2);
    bytes[role_offset] = 3;
    assert_eq!(
        decode_material_circuit_state(&bytes),
        Err(MaterialCircuitError::WireEnum)
    );
}

#[test]
fn local_inter_owner_transfer_needs_no_road_or_transit() {
    let mut state = merchant();
    state.supplier_routes[0].transport_kind = SupplierTransport::Local;
    state.route_stages.clear();
    state.route_stage_capacities.clear();
    let handling = state.merchants[0].capacity_id;
    state
        .corridor_capacities
        .retain(|row| row.corridor_id == handling);
    let completed = advance_material_circuit(&state).expect("explicit local circulation");
    assert_eq!(completed.state.orders[0].delivered, 2);
    assert!(completed.state.freight.is_empty());
    assert!(completed.arrivals.is_empty());
}

fn local_supplier() -> MaterialCircuitState {
    let mut state = merchant();
    state.supplier_routes[0].transport_kind = SupplierTransport::Local;
    state.route_stages.clear();
    state.route_stage_capacities.clear();
    let capacity = state.merchants[0].capacity_id;
    state
        .corridor_capacities
        .retain(|row| row.corridor_id == capacity);
    state
}

#[test]
fn local_transfer_has_explicit_receipt_and_still_requires_merchant_labor() {
    let mut state = local_supplier();
    let completed = advance_material_circuit(&state).unwrap();
    assert_eq!(completed.local_transfers.len(), 1);
    assert_eq!(completed.local_transfers[0].quantity, 2);
    assert_eq!(
        completed.local_transfers[0].buyer_site_id,
        state.orders[0].buyer_site_id
    );
    assert!(completed.dispatches.is_empty());
    assert!(completed.arrivals.is_empty());
    assert!(completed.deliveries.is_empty());
    assert!(completed.realizations.is_empty());
    assert_eq!(
        (
            completed.state.orders[0].shipped,
            completed.state.orders[0].delivered,
            completed.state.orders[0].realized,
            completed.state.orders[0].lost
        ),
        (2, 2, 2, 0)
    );
    assert_eq!(
        completed
            .state
            .inventory
            .iter()
            .map(|row| row.quantity)
            .sum::<u64>(),
        10
    );
    state.labor[0].available = 0;
    let stopped = advance_material_circuit(&state).unwrap();
    assert!(stopped.local_transfers.is_empty());
    assert_eq!(stopped.handling[0].needed_hours, 20);
    assert_eq!(stopped.handling[0].used_hours, 0);
    assert_eq!(stopped.state.orders[0].shipped, 0);
}

#[test]
fn local_buyer_credits_wait_until_all_outbound_grants_are_fixed() {
    let mut state = local_supplier();
    schedule(&mut state, 2, 4);
    let next_buyer = SiteId::from_bytes([40; 32]);
    state.site_logistics_nodes.push(SiteLogisticsNode {
        site_id: next_buyer,
        node_id: LogisticsNodeId::from_bytes([40; 32]),
    });
    let mut onward = state.orders[0].clone();
    onward.order_id = OrderId::from_bytes([41; 32]);
    onward.supplier_site_id = state.orders[0].buyer_site_id;
    onward.buyer_site_id = next_buyer;
    state.backlog.push(BacklogRow {
        order_id: onward.order_id,
        quantity: onward.ordered,
    });
    let mut relation = state.supplier_routes[0].clone();
    relation.route_id = RouteId::from_bytes([41; 32]);
    relation.supplier_site_id = onward.supplier_site_id;
    relation.buyer_site_id = next_buyer;
    state.supplier_routes.push(relation);
    state.orders.push(onward);
    let first = advance_material_circuit(&state).unwrap();
    assert_eq!(first.local_transfers.len(), 1);
    assert_eq!(first.state.orders[1].shipped, 0);
    assert!(first
        .state
        .inventory
        .iter()
        .all(|row| row.site_id != next_buyer || row.quantity == 0));
    let second = advance_material_circuit(&first.state).unwrap();
    assert_eq!(second.state.orders[1].shipped, 2);
    assert_eq!(
        second
            .state
            .inventory
            .iter()
            .find(|row| row.site_id == next_buyer)
            .unwrap()
            .quantity,
        2
    );
    assert_eq!(
        second
            .state
            .inventory
            .iter()
            .map(|row| row.quantity)
            .sum::<u64>(),
        10
    );
    state.orders.reverse();
    state.backlog.reverse();
    state.supplier_routes.reverse();
    assert_eq!(advance_material_circuit(&state).unwrap(), first);
}

#[test]
fn local_transfer_mode_and_wire_are_checked_and_buyer_overflow_is_atomic() {
    let state = local_supplier();
    let bytes = encode_material_circuit_state(&state).unwrap();
    assert_eq!(
        advance_material_circuit(&decode_material_circuit_state(&bytes).unwrap()).unwrap(),
        advance_material_circuit(&state).unwrap()
    );
    let mut wrong_mode = state.clone();
    wrong_mode.supplier_routes[0].transport_kind = SupplierTransport::Staged;
    assert_eq!(
        advance_material_circuit(&wrong_mode),
        Err(MaterialCircuitError::RouteInvariant)
    );
    let mut wrongly_local = merchant();
    wrongly_local.supplier_routes[0].transport_kind = SupplierTransport::Local;
    assert_eq!(
        advance_material_circuit(&wrongly_local),
        Err(MaterialCircuitError::RouteInvariant)
    );
    let mut overflowing = state.clone();
    let mut buyer_stock = overflowing.inventory[0].clone();
    buyer_stock.site_id = overflowing.orders[0].buyer_site_id;
    buyer_stock.quantity = u64::MAX;
    overflowing.inventory.push(buyer_stock);
    let preserved = overflowing.clone();
    assert_eq!(
        advance_material_circuit(&overflowing),
        Err(MaterialCircuitError::Arithmetic)
    );
    assert_eq!(overflowing, preserved);
}
