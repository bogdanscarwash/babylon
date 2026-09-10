use babylon_material_circuit::SupplierTransportV3;
use babylon_material_circuit::{
    advance_material_circuit_v3, decode_material_circuit_state_v3,
    encode_material_circuit_state_v3, material_circuit_state_v3_digest, BacklogRowV1,
    CorridorCapacityV3, CorridorIdV2, FreightMassCoefficientV3, GoodIdV1, InventoryRowV1,
    LogisticsNodeIdV2, MaterialCircuitErrorV3, MaterialCircuitStateV3, OrderAccessModeV1,
    OrderIdV1, OrderRowV2, RouteIdV2, RouteStageCapacityV3, RouteStageV3, SiteIdV1,
    SiteLogisticsNodeV2, SupplierRouteV3, UnitIdV1,
};

fn competition() -> MaterialCircuitStateV3 {
    let mut state = MaterialCircuitStateV3 {
        merchants: vec![],
        handling_coefficients: vec![],
        final_demand_principals: vec![],
        final_demand_orders: vec![],
        period: 1,
        site_logistics_nodes: Vec::new(),
        process_outputs: Vec::new(),
        input_coefficients: Vec::new(),
        labor_coefficients: Vec::new(),
        supplier_routes: Vec::new(),
        freight_mass_coefficients: Vec::new(),
        route_stage_capacities: Vec::new(),
        route_stages: Vec::new(),
        inventory: Vec::new(),
        orders: Vec::new(),
        backlog: Vec::new(),
        freight: Vec::new(),
        corridor_capacities: Vec::new(),
        capacities: Vec::new(),
        labor: Vec::new(),
        production_commitments: Vec::new(),
    };
    for (identity, quantity) in [(1_u8, 10), (2_u8, 100)] {
        let source = SiteIdV1::from_bytes([identity; 32]);
        let buyer = SiteIdV1::from_bytes([identity + 10; 32]);
        let source_node = LogisticsNodeIdV2::from_bytes([identity; 32]);
        let buyer_node = LogisticsNodeIdV2::from_bytes([identity + 10; 32]);
        let good = GoodIdV1::from_bytes([identity; 32]);
        let unit = UnitIdV1::from_bytes([identity; 32]);
        let route = RouteIdV2::from_bytes([identity; 32]);
        let order = OrderIdV1::from_bytes([identity; 32]);
        state.site_logistics_nodes.extend([
            SiteLogisticsNodeV2 {
                site_id: source,
                node_id: source_node,
            },
            SiteLogisticsNodeV2 {
                site_id: buyer,
                node_id: buyer_node,
            },
        ]);
        state.supplier_routes.push(SupplierRouteV3 {
            transport_kind: SupplierTransportV3::Staged,
            buyer_site_id: buyer,
            supplier_site_id: source,
            good_id: good,
            unit_id: unit,
            route_id: route,
        });
        state.route_stages.push(RouteStageV3 {
            route_id: route,
            stage_index: 0,
            from_node_id: source_node,
            to_node_id: buyer_node,
            travel_periods: 1,
            loss_ppm: 0,
        });
        state.inventory.push(InventoryRowV1 {
            site_id: source,
            good_id: good,
            unit_id: unit,
            quantity,
        });
        state.orders.push(OrderRowV2 {
            order_id: order,
            access_mode: OrderAccessModeV1::CommoditySale,
            buyer_site_id: buyer,
            supplier_site_id: source,
            good_id: good,
            unit_id: unit,
            ordered: quantity,
            shipped: 0,
            lost: 0,
            delivered: 0,
            realized: 0,
        });
        state.backlog.push(BacklogRowV1 {
            order_id: order,
            quantity,
        });
        state.route_stage_capacities.push(RouteStageCapacityV3 {
            route_id: route,
            stage_index: 0,
            corridor_id: CorridorIdV2::from_bytes([90; 32]),
        });
        state
            .freight_mass_coefficients
            .push(FreightMassCoefficientV3 {
                good_id: good,
                unit_id: unit,
                grams_per_unit: if identity == 1 { 10 } else { 1 },
            });
    }
    state.corridor_capacities.push(CorridorCapacityV3 {
        corridor_id: CorridorIdV2::from_bytes([90; 32]),
        period: 1,
        available_grams: 60,
    });
    state
}

#[test]
fn native_panel_and_mass_orders_compete_for_one_sixty_gram_budget() {
    // Ten 10-gram panels and one hundred 1-gram units each request 100 grams.
    let opening = competition();
    let result = advance_material_circuit_v3(&opening).unwrap();
    let quantities: Vec<_> = result.dispatches.iter().map(|row| row.quantity).collect();
    assert_eq!(quantities, [3, 30]);
}

fn dispatched(state: &MaterialCircuitStateV3) -> Vec<u64> {
    advance_material_circuit_v3(state)
        .unwrap()
        .dispatches
        .iter()
        .map(|row| row.quantity)
        .collect()
}

#[test]
fn simultaneous_bottleneck_debits_only_actual_dispatch_and_keeps_residual() {
    let mut state = competition();
    state.route_stage_capacities.push(RouteStageCapacityV3 {
        route_id: RouteIdV2::from_bytes([1; 32]),
        stage_index: 0,
        corridor_id: CorridorIdV2::from_bytes([91; 32]),
    });
    state.corridor_capacities.push(CorridorCapacityV3 {
        corridor_id: CorridorIdV2::from_bytes([91; 32]),
        period: 1,
        available_grams: 25,
    });
    let result = advance_material_circuit_v3(&state).unwrap();
    let quantities: Vec<_> = result.dispatches.iter().map(|row| row.quantity).collect();
    assert_eq!(quantities, [2, 30]);
    assert_eq!(60 - quantities[0] * 10 - quantities[1], 10);
    assert_eq!(25 - quantities[0] * 10, 5);
    assert_eq!(
        result
            .state
            .inventory
            .iter()
            .map(|row| row.quantity)
            .collect::<Vec<_>>(),
        [8, 70]
    );
    assert_eq!(
        result
            .state
            .orders
            .iter()
            .map(|row| row.shipped)
            .collect::<Vec<_>>(),
        quantities
    );
    assert_eq!(result.state.freight.len(), 2);
}

#[test]
fn reducing_competing_order_increases_panel_dispatch_without_reallocating_rounding() {
    let mut state = competition();
    state.orders[1].ordered = 40;
    state.backlog[1].quantity = 40;
    assert_eq!(dispatched(&state), [4, 17]);
    assert_eq!(60 - 4 * 10 - 17, 3);
    state = competition();
    state.corridor_capacities[0].available_grams = 61;
    assert_eq!(dispatched(&state), [3, 30]);
}

#[test]
fn native_inventory_shortage_does_not_redistribute_another_orders_mass_grant() {
    let mut state = competition();
    state.inventory[0].quantity = 1;
    assert_eq!(dispatched(&state), [1, 30]);
}

#[test]
fn many_capacity_memberships_share_one_stage_and_one_arrival() {
    let mut state = competition();
    for key in 100..200 {
        let corridor_id = CorridorIdV2::from_bytes([key; 32]);
        state.corridor_capacities.push(CorridorCapacityV3 {
            corridor_id,
            period: 1,
            available_grams: 60,
        });
        for key in [1, 2] {
            state.route_stage_capacities.push(RouteStageCapacityV3 {
                route_id: RouteIdV2::from_bytes([key; 32]),
                stage_index: 0,
                corridor_id,
            });
        }
    }
    let first = advance_material_circuit_v3(&state).unwrap();
    assert_eq!(first.state.route_stages.len(), 2);
    assert_eq!(first.dispatches.len(), 2);
    assert!(first
        .dispatches
        .iter()
        .all(|row| row.final_arrival_period == 2));
    let second = advance_material_circuit_v3(&first.state).unwrap();
    assert_eq!(
        second
            .arrivals
            .iter()
            .map(|row| row.quantity)
            .collect::<Vec<_>>(),
        [3, 30]
    );
    assert_eq!(second.arrivals.len(), second.deliveries.len());
    assert_eq!(second.arrivals.len(), second.realizations.len());
    assert!(second.state.freight.is_empty());
}

#[test]
fn future_stage_capacity_is_reserved_once_and_current_stage_loss_is_attributed() {
    let mut state = competition();
    let intermediate = LogisticsNodeIdV2::from_bytes([70; 32]);
    let destination = state.route_stages[0].to_node_id;
    state.route_stages[0].to_node_id = intermediate;
    state.route_stages[0].loss_ppm = 500_000;
    state.route_stages.push(RouteStageV3 {
        route_id: RouteIdV2::from_bytes([1; 32]),
        stage_index: 1,
        from_node_id: intermediate,
        to_node_id: destination,
        travel_periods: 1,
        loss_ppm: 0,
    });
    state.route_stage_capacities.push(RouteStageCapacityV3 {
        route_id: RouteIdV2::from_bytes([1; 32]),
        stage_index: 1,
        corridor_id: CorridorIdV2::from_bytes([90; 32]),
    });
    state.corridor_capacities.push(CorridorCapacityV3 {
        corridor_id: CorridorIdV2::from_bytes([90; 32]),
        period: 2,
        available_grams: 60,
    });
    let first = advance_material_circuit_v3(&state).unwrap();
    assert_eq!(first.state.corridor_capacities[0].available_grams, 30);
    let second = advance_material_circuit_v3(&first.state).unwrap();
    assert_eq!(second.losses.len(), 1);
    assert_eq!(second.losses[0].route_id, RouteIdV2::from_bytes([1; 32]));
    assert_eq!(second.losses[0].stage_index, 0);
    assert_eq!(second.losses[0].quantity, 1);
    assert_eq!(second.arrivals.len(), 1);
    let third = advance_material_circuit_v3(&second.state).unwrap();
    assert!(third
        .arrivals
        .iter()
        .any(|row| row.order_id == OrderIdV1::from_bytes([1; 32]) && row.quantity == 2));
}

#[test]
fn duplicate_or_missing_mass_and_stage_capacity_identity_is_refused() {
    let original = competition();
    let mut changed = original.clone();
    changed
        .route_stage_capacities
        .push(changed.route_stage_capacities[0].clone());
    assert_eq!(
        advance_material_circuit_v3(&changed),
        Err(MaterialCircuitErrorV3::DuplicateRow)
    );
    changed = original.clone();
    changed
        .corridor_capacities
        .push(changed.corridor_capacities[0].clone());
    assert_eq!(
        advance_material_circuit_v3(&changed),
        Err(MaterialCircuitErrorV3::DuplicateRow)
    );
    changed = original.clone();
    changed.freight_mass_coefficients[0].grams_per_unit = 0;
    assert_eq!(
        advance_material_circuit_v3(&changed),
        Err(MaterialCircuitErrorV3::MassInvariant)
    );
    changed = original.clone();
    changed.freight_mass_coefficients.remove(0);
    assert_eq!(
        advance_material_circuit_v3(&changed),
        Err(MaterialCircuitErrorV3::MassInvariant)
    );
    changed = original.clone();
    changed.route_stage_capacities.remove(0);
    assert_eq!(
        advance_material_circuit_v3(&changed),
        Err(MaterialCircuitErrorV3::RouteInvariant)
    );
    changed = original;
    changed.corridor_capacities.clear();
    assert!(advance_material_circuit_v3(&changed)
        .unwrap()
        .dispatches
        .is_empty());
}

#[test]
fn row_permutations_and_wire_roundtrip_preserve_complete_continuation() {
    let state = competition();
    let bytes = encode_material_circuit_state_v3(&state).unwrap();
    let restored = decode_material_circuit_state_v3(&bytes).unwrap();
    let mut reversed = state.clone();
    reversed.site_logistics_nodes.reverse();
    reversed.supplier_routes.reverse();
    reversed.route_stages.reverse();
    reversed.route_stage_capacities.reverse();
    reversed.freight_mass_coefficients.reverse();
    reversed.inventory.reverse();
    reversed.orders.reverse();
    reversed.backlog.reverse();
    reversed.corridor_capacities.reverse();
    assert_eq!(encode_material_circuit_state_v3(&reversed).unwrap(), bytes);
    assert_eq!(
        advance_material_circuit_v3(&state),
        advance_material_circuit_v3(&reversed)
    );
    assert_eq!(
        advance_material_circuit_v3(&state),
        advance_material_circuit_v3(&restored)
    );
    let contract = include_bytes!("../../../../contracts/material_circuit_v3.yaml");
    assert_eq!(
        babylon_kernel::sha256_of(contract),
        babylon_material_circuit::MATERIAL_CIRCUIT_V3_SOURCE_SHA256
    );
    assert_eq!(
        material_circuit_state_v3_digest(&state).unwrap(),
        babylon_kernel::sha256_of(&bytes)
    );
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_eq!(
        decode_material_circuit_state_v3(&trailing),
        Err(MaterialCircuitErrorV3::WireTrailing)
    );
    assert_eq!(
        decode_material_circuit_state_v3(&bytes[..bytes.len() - 1]),
        Err(MaterialCircuitErrorV3::WireTruncated)
    );
}

#[test]
fn overflowing_mass_request_sum_and_period_refuse_without_mutating_opening() {
    let mut state = competition();
    for order in &mut state.orders {
        order.ordered = u64::MAX;
    }
    for backlog in &mut state.backlog {
        backlog.quantity = u64::MAX;
    }
    for inventory in &mut state.inventory {
        inventory.quantity = u64::MAX;
    }
    for coefficient in &mut state.freight_mass_coefficients {
        coefficient.grams_per_unit = u64::MAX;
    }
    let before = state.clone();
    assert_eq!(
        advance_material_circuit_v3(&state),
        Err(MaterialCircuitErrorV3::Arithmetic)
    );
    assert_eq!(state, before);
    let mut state = competition();
    state.period = u64::MAX;
    state.corridor_capacities[0].period = u64::MAX;
    let before = state.clone();
    assert_eq!(
        advance_material_circuit_v3(&state),
        Err(MaterialCircuitErrorV3::Arithmetic)
    );
    assert_eq!(state, before);
}

#[test]
fn successor_refusal_registry_includes_mass_and_rejects_unknown_codes() {
    for value in 1_u16..=19 {
        assert_eq!(
            u16::from(MaterialCircuitErrorV3::try_from(value).unwrap()),
            value
        );
    }
    assert!(MaterialCircuitErrorV3::try_from(0).is_err());
    assert!(MaterialCircuitErrorV3::try_from(22).is_err());
}
