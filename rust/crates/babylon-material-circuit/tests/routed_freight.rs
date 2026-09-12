use babylon_material_circuit::{
    advance_material_circuit, decode_material_circuit_state, encode_material_circuit_state,
    material_circuit_state_digest, BacklogRow, CapacityRow, CorridorCapacity, CorridorId,
    FreightMassCoefficient, GoodId, InputOutputCoefficient, InventoryRow, LaborCapacityRow,
    LaborCoefficient, LogisticsNodeId, MaterialCircuitState, OrderAccessMode, OrderId, OrderRow,
    ProcessId, ProcessOutput, RouteId, RouteStage, RouteStageCapacity, SiteId, SiteLogisticsNode,
    SupplierRoute, SupplierTransport, UnitId, MATERIAL_CIRCUIT_STATE_DOMAIN_BYTES,
    MAX_ROUTE_STAGES_PER_ROUTE,
};

fn site(byte: u8) -> SiteId {
    SiteId::from_bytes([byte; 32])
}

fn good(byte: u8) -> GoodId {
    GoodId::from_bytes([byte; 32])
}

fn unit(byte: u8) -> UnitId {
    UnitId::from_bytes([byte; 32])
}

fn order(byte: u8) -> OrderId {
    OrderId::from_bytes([byte; 32])
}

fn node(byte: u8) -> LogisticsNodeId {
    LogisticsNodeId::from_bytes([byte; 32])
}

fn corridor(byte: u8) -> CorridorId {
    CorridorId::from_bytes([byte; 32])
}

fn route(byte: u8) -> RouteId {
    RouteId::from_bytes([byte; 32])
}

fn process(byte: u8) -> ProcessId {
    ProcessId::from_bytes([byte; 32])
}

const SUPPLIER: u8 = 1;
const BUYER: u8 = 2;
const GOODS: u8 = 3;
const GOODS_UNIT: u8 = 4;
const ORDER: u8 = 5;
const SUPPLIER_NODE: u8 = 6;
const BUYER_NODE: u8 = 7;
const CORRIDOR: u8 = 8;
const ROUTE: u8 = 9;

fn base_state() -> MaterialCircuitState {
    MaterialCircuitState {
        period: 1,
        merchants: vec![],
        handling_coefficients: vec![],
        final_demand_principals: vec![],
        final_demand_orders: vec![],
        freight_mass_coefficients: vec![FreightMassCoefficient {
            good_id: good(GOODS),
            unit_id: unit(GOODS_UNIT),
            grams_per_unit: 1,
        }],
        route_stage_capacities: vec![RouteStageCapacity {
            route_id: route(ROUTE),
            stage_index: 0,
            corridor_id: corridor(CORRIDOR),
        }],
        site_logistics_nodes: vec![
            SiteLogisticsNode {
                site_id: site(SUPPLIER),
                node_id: node(SUPPLIER_NODE),
            },
            SiteLogisticsNode {
                site_id: site(BUYER),
                node_id: node(BUYER_NODE),
            },
        ],
        process_outputs: Vec::new(),
        input_coefficients: Vec::new(),
        labor_coefficients: Vec::new(),
        supplier_routes: vec![SupplierRoute {
            transport_kind: SupplierTransport::Staged,
            buyer_site_id: site(BUYER),
            supplier_site_id: site(SUPPLIER),
            good_id: good(GOODS),
            unit_id: unit(GOODS_UNIT),
            route_id: route(ROUTE),
        }],
        route_stages: vec![RouteStage {
            route_id: route(ROUTE),
            stage_index: 0,
            from_node_id: node(SUPPLIER_NODE),
            to_node_id: node(BUYER_NODE),
            travel_periods: 1,
            loss_ppm: 0,
        }],
        inventory: vec![
            InventoryRow {
                site_id: site(SUPPLIER),
                good_id: good(GOODS),
                unit_id: unit(GOODS_UNIT),
                quantity: 10,
            },
            InventoryRow {
                site_id: site(BUYER),
                good_id: good(GOODS),
                unit_id: unit(GOODS_UNIT),
                quantity: 0,
            },
        ],
        orders: vec![OrderRow {
            order_id: order(ORDER),
            access_mode: OrderAccessMode::CommoditySale,
            buyer_site_id: site(BUYER),
            supplier_site_id: site(SUPPLIER),
            good_id: good(GOODS),
            unit_id: unit(GOODS_UNIT),
            ordered: 6,
            shipped: 0,
            lost: 0,
            delivered: 0,
            realized: 0,
        }],
        backlog: vec![BacklogRow {
            order_id: order(ORDER),
            quantity: 6,
        }],
        freight: Vec::new(),
        corridor_capacities: vec![CorridorCapacity {
            corridor_id: corridor(CORRIDOR),
            period: 1,
            available_grams: 4,
        }],
        capacities: Vec::new(),
        labor: Vec::new(),
        production_commitments: Vec::new(),
    }
}

fn inventory_quantity(state: &MaterialCircuitState, site_id: SiteId) -> u64 {
    state
        .inventory
        .iter()
        .find(|row| row.site_id == site_id && row.good_id == good(GOODS))
        .map_or(0, |row| row.quantity)
}

fn two_leg_state(second_leg_capacity: u64) -> MaterialCircuitState {
    let mut state = base_state();
    let middle = node(10);
    state.route_stages = vec![
        RouteStage {
            to_node_id: middle,
            loss_ppm: 250_000,
            ..state.route_stages[0].clone()
        },
        RouteStage {
            route_id: route(ROUTE),
            stage_index: 1,
            from_node_id: middle,
            to_node_id: node(BUYER_NODE),
            travel_periods: 1,
            loss_ppm: 0,
        },
    ];
    state.route_stage_capacities.push(RouteStageCapacity {
        route_id: route(ROUTE),
        stage_index: 1,
        corridor_id: corridor(11),
    });
    state.corridor_capacities = vec![
        CorridorCapacity {
            available_grams: 4,
            ..state.corridor_capacities[0].clone()
        },
        CorridorCapacity {
            corridor_id: corridor(11),
            period: 2,
            available_grams: second_leg_capacity,
        },
    ];
    state
}

fn two_route_state() -> MaterialCircuitState {
    let mut state = base_state();
    let second_buyer = site(12);
    let second_node = node(13);
    let second_order = order(14);
    let second_route = route(15);
    state.inventory[0].quantity = 12;
    state.site_logistics_nodes.push(SiteLogisticsNode {
        site_id: second_buyer,
        node_id: second_node,
    });
    state.supplier_routes.push(SupplierRoute {
        transport_kind: SupplierTransport::Staged,
        buyer_site_id: second_buyer,
        supplier_site_id: site(SUPPLIER),
        good_id: good(GOODS),
        unit_id: unit(GOODS_UNIT),
        route_id: second_route,
    });
    state.route_stage_capacities.push(RouteStageCapacity {
        route_id: second_route,
        stage_index: 0,
        corridor_id: corridor(16),
    });
    state.route_stages.push(RouteStage {
        route_id: second_route,
        stage_index: 0,
        from_node_id: node(SUPPLIER_NODE),
        to_node_id: second_node,
        travel_periods: 1,
        loss_ppm: 0,
    });
    state.inventory.push(InventoryRow {
        site_id: second_buyer,
        good_id: good(GOODS),
        unit_id: unit(GOODS_UNIT),
        quantity: 0,
    });
    state.orders.push(OrderRow {
        order_id: second_order,
        access_mode: OrderAccessMode::CommoditySale,
        buyer_site_id: second_buyer,
        supplier_site_id: site(SUPPLIER),
        good_id: good(GOODS),
        unit_id: unit(GOODS_UNIT),
        ordered: 4,
        shipped: 0,
        lost: 0,
        delivered: 0,
        realized: 0,
    });
    state.backlog.push(BacklogRow {
        order_id: second_order,
        quantity: 4,
    });
    state.corridor_capacities.push(CorridorCapacity {
        corridor_id: corridor(16),
        period: 1,
        available_grams: 4,
    });
    state
}

fn route_depth_state(leg_count: usize) -> MaterialCircuitState {
    let mut state = base_state();
    state.route_stages = (0..leg_count)
        .map(|index| RouteStage {
            route_id: route(ROUTE),
            stage_index: u16::try_from(index).expect("test route index must fit"),
            from_node_id: if index == 0 {
                node(SUPPLIER_NODE)
            } else {
                node(19 + u8::try_from(index).expect("test node must fit"))
            },
            to_node_id: if index + 1 == leg_count {
                node(BUYER_NODE)
            } else {
                node(20 + u8::try_from(index).expect("test node must fit"))
            },
            travel_periods: 1,
            loss_ppm: 0,
        })
        .collect();
    state.route_stage_capacities = (0..leg_count)
        .map(|index| RouteStageCapacity {
            route_id: route(ROUTE),
            stage_index: u16::try_from(index).unwrap(),
            corridor_id: corridor(60 + u8::try_from(index).unwrap()),
        })
        .collect();
    state.corridor_capacities = state
        .route_stage_capacities
        .iter()
        .map(|membership| CorridorCapacity {
            corridor_id: membership.corridor_id,
            period: 1 + u64::from(membership.stage_index),
            available_grams: 4,
        })
        .collect();
    state
}

fn shipped_for(state: &MaterialCircuitState, order_id: OrderId) -> u64 {
    state
        .orders
        .iter()
        .find(|row| row.order_id == order_id)
        .map_or(0, |row| row.shipped)
}

#[test]
fn corridor_capacity_bounds_routed_dispatch() {
    let outcome = advance_material_circuit(&base_state()).expect("period one must close");

    assert_eq!(outcome.state.period, 2);
    assert_eq!(outcome.state.orders[0].shipped, 4);
    assert_eq!(outcome.state.orders[0].delivered, 0);
    assert_eq!(outcome.state.orders[0].realized, 0);
    assert_eq!(outcome.state.backlog[0].quantity, 2);
    assert_eq!(inventory_quantity(&outcome.state, site(SUPPLIER)), 6);
    assert_eq!(inventory_quantity(&outcome.state, site(BUYER)), 0);
    assert_eq!(outcome.dispatches.len(), 1);
    assert_eq!(outcome.dispatches[0].quantity, 4);
    assert_eq!(outcome.state.freight.len(), 1);
    assert_eq!(outcome.state.freight[0].quantity, 4);
    assert_eq!(outcome.state.freight[0].current_stage_index, 0);
    assert_eq!(outcome.state.freight[0].stage_arrival_period, 2);
}

#[test]
fn final_route_arrival_credits_inventory_before_realization() {
    let first = advance_material_circuit(&base_state()).expect("period one must close");
    let second = advance_material_circuit(&first.state).expect("period two must close");

    assert!(second.state.freight.is_empty());
    assert_eq!(inventory_quantity(&second.state, site(BUYER)), 4);
    assert_eq!(second.state.orders[0].delivered, 4);
    assert_eq!(second.state.orders[0].realized, 4);
    assert_eq!(second.arrivals.len(), 1);
    assert_eq!(second.deliveries.len(), 1);
    assert_eq!(second.realizations.len(), 1);
}

#[test]
fn missing_supplier_route_remains_backlog() {
    let mut state = base_state();
    state.supplier_routes.clear();

    let outcome = advance_material_circuit(&state)
        .expect("a missing routed supplier relation is a material shortage");

    assert!(outcome.dispatches.is_empty());
    assert!(outcome.state.freight.is_empty());
    assert_eq!(outcome.state.orders[0].shipped, 0);
    assert_eq!(outcome.state.backlog[0].quantity, 6);
    assert_eq!(inventory_quantity(&outcome.state, site(SUPPLIER)), 10);
}

#[test]
fn capacity_for_an_unknown_corridor_refuses() {
    let mut state = base_state();
    state.corridor_capacities.push(CorridorCapacity {
        corridor_id: corridor(99),
        period: 1,
        available_grams: 1,
    });

    assert_eq!(
        advance_material_circuit(&state),
        Err(babylon_material_circuit::MaterialCircuitError::CapacityInvariant)
    );
}

#[test]
fn freight_lot_identity_must_bind_order_and_dispatch_period() {
    let first = advance_material_circuit(&base_state()).expect("period one must close");
    let mut state = first.state;
    state.freight[0].lot_id = babylon_material_circuit::FreightLotId::from_bytes([77; 32]);

    assert_eq!(
        advance_material_circuit(&state),
        Err(babylon_material_circuit::MaterialCircuitError::FreightInvariant)
    );
}

#[test]
fn freight_leg_arrival_must_match_the_reserved_route_schedule() {
    let first = advance_material_circuit(&base_state()).expect("period one must close");
    let mut state = first.state;
    state.freight[0].stage_arrival_period = 3;

    assert_eq!(
        advance_material_circuit(&state),
        Err(babylon_material_circuit::MaterialCircuitError::FreightInvariant)
    );
}

#[test]
fn future_leg_capacity_limits_origin_dispatch() {
    let outcome = advance_material_circuit(&two_leg_state(2)).expect("two-leg route must close");

    assert_eq!(outcome.state.orders[0].shipped, 2);
    assert_eq!(outcome.state.backlog[0].quantity, 4);
    assert_eq!(outcome.state.freight[0].quantity, 2);
    assert_eq!(outcome.dispatches[0].final_arrival_period, 3);
    assert_eq!(inventory_quantity(&outcome.state, site(SUPPLIER)), 8);
    assert_eq!(outcome.state.corridor_capacities[0].period, 2);
    assert_eq!(outcome.state.corridor_capacities[0].available_grams, 0);
}

#[test]
fn completed_leg_loss_remains_attributed_before_final_delivery() {
    let first = advance_material_circuit(&two_leg_state(4)).expect("period one must close");
    let second = advance_material_circuit(&first.state).expect("period two must close");

    assert_eq!(second.losses.len(), 1);
    assert_eq!(second.losses[0].quantity, 1);
    assert_eq!(second.state.orders[0].lost, 1);
    assert_eq!(second.state.orders[0].delivered, 0);
    assert_eq!(second.state.freight[0].quantity, 3);
    assert_eq!(second.state.freight[0].current_stage_index, 1);
    assert_eq!(second.state.freight[0].stage_arrival_period, 3);

    let third = advance_material_circuit(&second.state).expect("period three must close");
    assert!(third.state.freight.is_empty());
    assert_eq!(third.state.orders[0].shipped, 4);
    assert_eq!(third.state.orders[0].lost, 1);
    assert_eq!(third.state.orders[0].delivered, 3);
    assert_eq!(third.state.orders[0].realized, 3);
    assert_eq!(inventory_quantity(&third.state, site(BUYER)), 3);
}

#[test]
fn final_arrival_can_form_and_execute_following_period_production() {
    let mut state = base_state();
    state.process_outputs.push(ProcessOutput {
        process_id: process(18),
        site_id: site(BUYER),
        good_id: good(19),
        unit_id: unit(GOODS_UNIT),
        quantity_per_batch: 5,
    });
    state.input_coefficients.push(InputOutputCoefficient {
        process_id: process(18),
        good_id: good(GOODS),
        unit_id: unit(GOODS_UNIT),
        quantity_per_batch: 2,
    });
    state.labor_coefficients.push(LaborCoefficient {
        process_id: process(18),
        unit_id: unit(20),
        quantity_per_batch: 1,
    });
    state.capacities.push(CapacityRow {
        process_id: process(18),
        site_id: site(BUYER),
        period: 3,
        available_batches: 2,
    });
    state.labor.push(LaborCapacityRow {
        site_id: site(BUYER),
        unit_id: unit(20),
        period: 3,
        available: 2,
    });

    let dispatch = advance_material_circuit(&state).expect("dispatch period must close");
    assert!(dispatch.state.production_commitments.is_empty());
    let arrival = advance_material_circuit(&dispatch.state).expect("arrival period must close");
    assert_eq!(arrival.state.production_commitments.len(), 1);
    assert_eq!(arrival.state.production_commitments[0].planned_batches, 2);
    let production =
        advance_material_circuit(&arrival.state).expect("production period must close");

    assert_eq!(production.production.len(), 1);
    assert_eq!(production.production[0].produced_batches, 2);
    assert_eq!(inventory_quantity(&production.state, site(BUYER)), 0);
    assert_eq!(
        production
            .state
            .inventory
            .iter()
            .find(|row| row.site_id == site(BUYER) && row.good_id == good(19))
            .expect("produced output inventory must exist")
            .quantity,
        10
    );
}

#[test]
fn route_depth_accepts_the_designed_maximum_and_refuses_plus_one() {
    assert!(advance_material_circuit(&route_depth_state(MAX_ROUTE_STAGES_PER_ROUTE)).is_ok());
    assert_eq!(
        advance_material_circuit(&route_depth_state(MAX_ROUTE_STAGES_PER_ROUTE + 1)),
        Err(babylon_material_circuit::MaterialCircuitError::RouteInvariant)
    );
}

#[test]
fn arrival_overflow_refuses_atomically_without_mutating_the_opening_state() {
    let dispatch = advance_material_circuit(&base_state()).expect("dispatch period must close");
    let mut opening = dispatch.state;
    let buyer_inventory = opening
        .inventory
        .iter_mut()
        .find(|row| row.site_id == site(BUYER))
        .expect("buyer inventory must exist");
    buyer_inventory.quantity = u64::MAX;
    let opening_digest = material_circuit_state_digest(&opening).expect("opening must hash");

    assert_eq!(
        advance_material_circuit(&opening),
        Err(babylon_material_circuit::MaterialCircuitError::Arithmetic)
    );
    assert_eq!(
        material_circuit_state_digest(&opening).expect("opening must remain valid"),
        opening_digest
    );
}

#[test]
fn current_decoder_refuses_domain_version_truncation_and_trailing_bytes() {
    let bytes = encode_material_circuit_state(&base_state()).expect("base state must encode");
    let mut wrong_domain = bytes.clone();
    wrong_domain[0] ^= 1;
    assert_eq!(
        decode_material_circuit_state(&wrong_domain),
        Err(babylon_material_circuit::MaterialCircuitError::WireDomain)
    );

    let version_index = MATERIAL_CIRCUIT_STATE_DOMAIN_BYTES.len() + 1;
    let mut wrong_version = bytes.clone();
    wrong_version[version_index + 1] = 4;
    assert_eq!(
        decode_material_circuit_state(&wrong_version),
        Err(babylon_material_circuit::MaterialCircuitError::WireVersion)
    );
    assert_eq!(
        decode_material_circuit_state(&bytes[..bytes.len() - 1]),
        Err(babylon_material_circuit::MaterialCircuitError::WireTruncated)
    );
    let mut trailing = bytes;
    trailing.push(0);
    assert_eq!(
        decode_material_circuit_state(&trailing),
        Err(babylon_material_circuit::MaterialCircuitError::WireTrailing)
    );
}

#[test]
fn severed_corridor_changes_only_its_routed_inventory_and_realization() {
    let full = advance_material_circuit(&two_route_state()).expect("both routes must close");
    let mut severed_state = two_route_state();
    severed_state
        .corridor_capacities
        .retain(|row| row.corridor_id != corridor(CORRIDOR));
    let severed = advance_material_circuit(&severed_state).expect("severed route must close");

    assert_eq!(shipped_for(&full.state, order(ORDER)), 4);
    assert_eq!(shipped_for(&severed.state, order(ORDER)), 0);
    assert_eq!(shipped_for(&full.state, order(14)), 4);
    assert_eq!(shipped_for(&severed.state, order(14)), 4);

    let full_arrival = advance_material_circuit(&full.state).expect("full arrival must close");
    let severed_arrival =
        advance_material_circuit(&severed.state).expect("severed arrival must close");
    assert_eq!(shipped_for(&full_arrival.state, order(ORDER)), 4);
    assert_eq!(
        full_arrival
            .state
            .orders
            .iter()
            .find(|row| row.order_id == order(ORDER))
            .unwrap()
            .realized,
        4
    );
    assert_eq!(
        severed_arrival
            .state
            .orders
            .iter()
            .find(|row| row.order_id == order(ORDER))
            .unwrap()
            .realized,
        0
    );
    assert_eq!(
        full_arrival
            .state
            .orders
            .iter()
            .find(|row| row.order_id == order(14))
            .unwrap()
            .realized,
        4
    );
    assert_eq!(
        severed_arrival
            .state
            .orders
            .iter()
            .find(|row| row.order_id == order(14))
            .unwrap()
            .realized,
        4
    );
}

#[test]
fn shared_corridor_allocation_exhaustively_conserves_permutations() {
    for available in 0_u64..=10 {
        for corridor_available in 0_u64..=10 {
            for first_requested in 1_u64..=5 {
                for second_requested in 1_u64..=5 {
                    let mut state = base_state();
                    state.inventory[0].quantity = available;
                    state.corridor_capacities[0].available_grams = corridor_available;
                    state.orders[0].ordered = first_requested;
                    state.backlog[0].quantity = first_requested;
                    state.orders.push(OrderRow {
                        order_id: order(17),
                        ordered: second_requested,
                        ..state.orders[0].clone()
                    });
                    state.backlog.push(BacklogRow {
                        order_id: order(17),
                        quantity: second_requested,
                    });
                    let mut reversed = state.clone();
                    reversed.orders.reverse();
                    reversed.backlog.reverse();

                    let outcome = advance_material_circuit(&state).expect("allocation must close");
                    let twin = advance_material_circuit(&reversed)
                        .expect("permuted allocation must close");
                    let effective = available.min(corridor_available);
                    let total_requested = first_requested + second_requested;
                    let expected_first = if effective >= total_requested {
                        first_requested
                    } else {
                        effective * first_requested / total_requested
                    };
                    let expected_second = if effective >= total_requested {
                        second_requested
                    } else {
                        effective * second_requested / total_requested
                    };

                    assert_eq!(shipped_for(&outcome.state, order(ORDER)), expected_first);
                    assert_eq!(shipped_for(&outcome.state, order(17)), expected_second);
                    assert_eq!(
                        inventory_quantity(&outcome.state, site(SUPPLIER))
                            + expected_first
                            + expected_second,
                        available
                    );
                    assert_eq!(
                        material_circuit_state_digest(&outcome.state),
                        material_circuit_state_digest(&twin.state)
                    );
                }
            }
        }
    }
}
