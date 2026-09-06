use babylon_material_circuit::{
    advance_material_circuit_v2, decode_material_circuit_state_v2,
    encode_material_circuit_state_v2, material_circuit_state_v2_digest, BacklogRowV1,
    CapacityRowV1, CorridorCapacityV2, CorridorIdV2, GoodIdV1, InputOutputCoefficientV1,
    InventoryRowV1, LaborCapacityRowV1, LaborCoefficientV1, LogisticsNodeIdV2,
    MaterialCircuitStateV2, OrderAccessModeV1, OrderIdV1, OrderRowV2, ProcessIdV1, ProcessOutputV1,
    RouteIdV2, RouteLegV2, SiteIdV1, SiteLogisticsNodeV2, SupplierRouteV2, UnitIdV1,
    MATERIAL_CIRCUIT_STATE_V2_DOMAIN_BYTES, MATERIAL_CIRCUIT_V2_SOURCE_SHA256,
    MAX_ROUTE_LEGS_PER_ROUTE_V2,
};

fn site(byte: u8) -> SiteIdV1 {
    SiteIdV1::from_bytes([byte; 32])
}

fn good(byte: u8) -> GoodIdV1 {
    GoodIdV1::from_bytes([byte; 32])
}

fn unit(byte: u8) -> UnitIdV1 {
    UnitIdV1::from_bytes([byte; 32])
}

fn order(byte: u8) -> OrderIdV1 {
    OrderIdV1::from_bytes([byte; 32])
}

fn node(byte: u8) -> LogisticsNodeIdV2 {
    LogisticsNodeIdV2::from_bytes([byte; 32])
}

fn corridor(byte: u8) -> CorridorIdV2 {
    CorridorIdV2::from_bytes([byte; 32])
}

fn route(byte: u8) -> RouteIdV2 {
    RouteIdV2::from_bytes([byte; 32])
}

fn process(byte: u8) -> ProcessIdV1 {
    ProcessIdV1::from_bytes([byte; 32])
}

fn hex(bytes: [u8; 32]) -> String {
    use std::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(64), |mut output, byte| {
            let _ = write!(output, "{byte:02x}");
            output
        })
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
const CONTRACT_VECTORS: &str =
    include_str!("../../../../contracts/material_circuit_v2_vectors.jsonl");
const CONTRACT_SCHEMA: &[u8] = include_bytes!("../../../../contracts/material_circuit_v2.yaml");

fn base_state() -> MaterialCircuitStateV2 {
    MaterialCircuitStateV2 {
        period: 1,
        site_logistics_nodes: vec![
            SiteLogisticsNodeV2 {
                site_id: site(SUPPLIER),
                node_id: node(SUPPLIER_NODE),
            },
            SiteLogisticsNodeV2 {
                site_id: site(BUYER),
                node_id: node(BUYER_NODE),
            },
        ],
        process_outputs: Vec::new(),
        input_coefficients: Vec::new(),
        labor_coefficients: Vec::new(),
        supplier_routes: vec![SupplierRouteV2 {
            buyer_site_id: site(BUYER),
            supplier_site_id: site(SUPPLIER),
            good_id: good(GOODS),
            unit_id: unit(GOODS_UNIT),
            route_id: route(ROUTE),
        }],
        route_legs: vec![RouteLegV2 {
            route_id: route(ROUTE),
            leg_index: 0,
            corridor_id: corridor(CORRIDOR),
            from_node_id: node(SUPPLIER_NODE),
            to_node_id: node(BUYER_NODE),
            travel_periods: 1,
            loss_ppm: 0,
        }],
        inventory: vec![
            InventoryRowV1 {
                site_id: site(SUPPLIER),
                good_id: good(GOODS),
                unit_id: unit(GOODS_UNIT),
                quantity: 10,
            },
            InventoryRowV1 {
                site_id: site(BUYER),
                good_id: good(GOODS),
                unit_id: unit(GOODS_UNIT),
                quantity: 0,
            },
        ],
        orders: vec![OrderRowV2 {
            order_id: order(ORDER),
            access_mode: OrderAccessModeV1::CommoditySale,
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
        backlog: vec![BacklogRowV1 {
            order_id: order(ORDER),
            quantity: 6,
        }],
        freight: Vec::new(),
        corridor_capacities: vec![CorridorCapacityV2 {
            corridor_id: corridor(CORRIDOR),
            unit_id: unit(GOODS_UNIT),
            period: 1,
            available: 4,
        }],
        capacities: Vec::new(),
        labor: Vec::new(),
        production_commitments: Vec::new(),
    }
}

fn inventory_quantity(state: &MaterialCircuitStateV2, site_id: SiteIdV1) -> u64 {
    state
        .inventory
        .iter()
        .find(|row| row.site_id == site_id && row.good_id == good(GOODS))
        .map_or(0, |row| row.quantity)
}

fn two_leg_state(second_leg_capacity: u64) -> MaterialCircuitStateV2 {
    let mut state = base_state();
    let middle = node(10);
    state.route_legs = vec![
        RouteLegV2 {
            to_node_id: middle,
            loss_ppm: 250_000,
            ..state.route_legs[0].clone()
        },
        RouteLegV2 {
            route_id: route(ROUTE),
            leg_index: 1,
            corridor_id: corridor(11),
            from_node_id: middle,
            to_node_id: node(BUYER_NODE),
            travel_periods: 1,
            loss_ppm: 0,
        },
    ];
    state.corridor_capacities = vec![
        CorridorCapacityV2 {
            available: 4,
            ..state.corridor_capacities[0].clone()
        },
        CorridorCapacityV2 {
            corridor_id: corridor(11),
            unit_id: unit(GOODS_UNIT),
            period: 2,
            available: second_leg_capacity,
        },
    ];
    state
}

fn two_route_state() -> MaterialCircuitStateV2 {
    let mut state = base_state();
    let second_buyer = site(12);
    let second_node = node(13);
    let second_order = order(14);
    let second_route = route(15);
    state.inventory[0].quantity = 12;
    state.site_logistics_nodes.push(SiteLogisticsNodeV2 {
        site_id: second_buyer,
        node_id: second_node,
    });
    state.supplier_routes.push(SupplierRouteV2 {
        buyer_site_id: second_buyer,
        supplier_site_id: site(SUPPLIER),
        good_id: good(GOODS),
        unit_id: unit(GOODS_UNIT),
        route_id: second_route,
    });
    state.route_legs.push(RouteLegV2 {
        route_id: second_route,
        leg_index: 0,
        corridor_id: corridor(16),
        from_node_id: node(SUPPLIER_NODE),
        to_node_id: second_node,
        travel_periods: 1,
        loss_ppm: 0,
    });
    state.inventory.push(InventoryRowV1 {
        site_id: second_buyer,
        good_id: good(GOODS),
        unit_id: unit(GOODS_UNIT),
        quantity: 0,
    });
    state.orders.push(OrderRowV2 {
        order_id: second_order,
        access_mode: OrderAccessModeV1::CommoditySale,
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
    state.backlog.push(BacklogRowV1 {
        order_id: second_order,
        quantity: 4,
    });
    state.corridor_capacities.push(CorridorCapacityV2 {
        corridor_id: corridor(16),
        unit_id: unit(GOODS_UNIT),
        period: 1,
        available: 4,
    });
    state
}

fn route_depth_state(leg_count: usize) -> MaterialCircuitStateV2 {
    let mut state = base_state();
    state.route_legs = (0..leg_count)
        .map(|index| RouteLegV2 {
            route_id: route(ROUTE),
            leg_index: u16::try_from(index).expect("test route index must fit"),
            corridor_id: corridor(60 + u8::try_from(index).expect("test corridor must fit")),
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
    state.corridor_capacities = state
        .route_legs
        .iter()
        .map(|leg| CorridorCapacityV2 {
            corridor_id: leg.corridor_id,
            unit_id: unit(GOODS_UNIT),
            period: 1 + u64::from(leg.leg_index),
            available: 4,
        })
        .collect();
    state
}

fn shipped_for(state: &MaterialCircuitStateV2, order_id: OrderIdV1) -> u64 {
    state
        .orders
        .iter()
        .find(|row| row.order_id == order_id)
        .map_or(0, |row| row.shipped)
}

#[test]
fn corridor_capacity_bounds_routed_dispatch() {
    let outcome = advance_material_circuit_v2(&base_state()).expect("period one must close");

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
    assert_eq!(outcome.state.freight[0].current_leg_index, 0);
    assert_eq!(outcome.state.freight[0].leg_arrival_period, 2);
}

#[test]
fn final_route_arrival_credits_inventory_before_realization() {
    let first = advance_material_circuit_v2(&base_state()).expect("period one must close");
    let second = advance_material_circuit_v2(&first.state).expect("period two must close");

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

    let outcome = advance_material_circuit_v2(&state)
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
    state.corridor_capacities.push(CorridorCapacityV2 {
        corridor_id: corridor(99),
        unit_id: unit(GOODS_UNIT),
        period: 1,
        available: 1,
    });

    assert_eq!(
        advance_material_circuit_v2(&state),
        Err(babylon_material_circuit::MaterialCircuitErrorV2::CapacityInvariant)
    );
}

#[test]
fn freight_lot_identity_must_bind_order_and_dispatch_period() {
    let first = advance_material_circuit_v2(&base_state()).expect("period one must close");
    let mut state = first.state;
    state.freight[0].lot_id = babylon_material_circuit::FreightLotIdV2::from_bytes([77; 32]);

    assert_eq!(
        advance_material_circuit_v2(&state),
        Err(babylon_material_circuit::MaterialCircuitErrorV2::FreightInvariant)
    );
}

#[test]
fn freight_leg_arrival_must_match_the_reserved_route_schedule() {
    let first = advance_material_circuit_v2(&base_state()).expect("period one must close");
    let mut state = first.state;
    state.freight[0].leg_arrival_period = 3;

    assert_eq!(
        advance_material_circuit_v2(&state),
        Err(babylon_material_circuit::MaterialCircuitErrorV2::FreightInvariant)
    );
}

#[test]
fn future_leg_capacity_limits_origin_dispatch() {
    let outcome = advance_material_circuit_v2(&two_leg_state(2)).expect("two-leg route must close");

    assert_eq!(outcome.state.orders[0].shipped, 2);
    assert_eq!(outcome.state.backlog[0].quantity, 4);
    assert_eq!(outcome.state.freight[0].quantity, 2);
    assert_eq!(outcome.dispatches[0].final_arrival_period, 3);
    assert_eq!(inventory_quantity(&outcome.state, site(SUPPLIER)), 8);
    assert_eq!(outcome.state.corridor_capacities[0].period, 2);
    assert_eq!(outcome.state.corridor_capacities[0].available, 0);
}

#[test]
fn completed_leg_loss_remains_attributed_before_final_delivery() {
    let first = advance_material_circuit_v2(&two_leg_state(4)).expect("period one must close");
    let second = advance_material_circuit_v2(&first.state).expect("period two must close");

    assert_eq!(second.losses.len(), 1);
    assert_eq!(second.losses[0].quantity, 1);
    assert_eq!(second.state.orders[0].lost, 1);
    assert_eq!(second.state.orders[0].delivered, 0);
    assert_eq!(second.state.freight[0].quantity, 3);
    assert_eq!(second.state.freight[0].current_leg_index, 1);
    assert_eq!(second.state.freight[0].leg_arrival_period, 3);

    let third = advance_material_circuit_v2(&second.state).expect("period three must close");
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
    state.process_outputs.push(ProcessOutputV1 {
        process_id: process(18),
        site_id: site(BUYER),
        good_id: good(19),
        unit_id: unit(GOODS_UNIT),
        quantity_per_batch: 5,
    });
    state.input_coefficients.push(InputOutputCoefficientV1 {
        process_id: process(18),
        good_id: good(GOODS),
        unit_id: unit(GOODS_UNIT),
        quantity_per_batch: 2,
    });
    state.labor_coefficients.push(LaborCoefficientV1 {
        process_id: process(18),
        unit_id: unit(20),
        quantity_per_batch: 1,
    });
    state.capacities.push(CapacityRowV1 {
        process_id: process(18),
        site_id: site(BUYER),
        period: 3,
        available_batches: 2,
    });
    state.labor.push(LaborCapacityRowV1 {
        site_id: site(BUYER),
        unit_id: unit(20),
        period: 3,
        available: 2,
    });

    let dispatch = advance_material_circuit_v2(&state).expect("dispatch period must close");
    assert!(dispatch.state.production_commitments.is_empty());
    let arrival = advance_material_circuit_v2(&dispatch.state).expect("arrival period must close");
    assert_eq!(arrival.state.production_commitments.len(), 1);
    assert_eq!(arrival.state.production_commitments[0].planned_batches, 2);
    let production =
        advance_material_circuit_v2(&arrival.state).expect("production period must close");

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
    assert!(advance_material_circuit_v2(&route_depth_state(MAX_ROUTE_LEGS_PER_ROUTE_V2)).is_ok());
    assert_eq!(
        advance_material_circuit_v2(&route_depth_state(MAX_ROUTE_LEGS_PER_ROUTE_V2 + 1)),
        Err(babylon_material_circuit::MaterialCircuitErrorV2::RouteInvariant)
    );
}

#[test]
fn arrival_overflow_refuses_atomically_without_mutating_the_opening_state() {
    let dispatch = advance_material_circuit_v2(&base_state()).expect("dispatch period must close");
    let mut opening = dispatch.state;
    let buyer_inventory = opening
        .inventory
        .iter_mut()
        .find(|row| row.site_id == site(BUYER))
        .expect("buyer inventory must exist");
    buyer_inventory.quantity = u64::MAX;
    let opening_digest = material_circuit_state_v2_digest(&opening).expect("opening must hash");

    assert_eq!(
        advance_material_circuit_v2(&opening),
        Err(babylon_material_circuit::MaterialCircuitErrorV2::Arithmetic)
    );
    assert_eq!(
        material_circuit_state_v2_digest(&opening).expect("opening must remain valid"),
        opening_digest
    );
}

#[test]
fn canonical_v2_state_round_trips_through_exact_bytes() {
    let bytes = encode_material_circuit_state_v2(&base_state()).expect("base state must encode");
    let decoded = decode_material_circuit_state_v2(&bytes).expect("canonical bytes must decode");
    let digest = material_circuit_state_v2_digest(&base_state()).expect("base state must hash");

    assert_eq!(bytes.len(), 1_053);
    assert_eq!(
        hex(digest),
        "a0ca4c774cf74110ffc3611aa1bd7609cbee07e360477e99363f268b93d7cf31"
    );
    assert_eq!(decoded, base_state());
    assert_eq!(
        material_circuit_state_v2_digest(&decoded).expect("decoded state must hash"),
        digest
    );
    let vector: serde_json::Value = serde_json::from_str(
        CONTRACT_VECTORS
            .lines()
            .next()
            .expect("the base vector must be first"),
    )
    .expect("the base vector must be valid JSON");
    assert_eq!(vector["data"]["canonical_bytes"], bytes.len());
    assert_eq!(vector["data"]["digest_hex"], hex(digest));
    let manifest: serde_json::Value = serde_json::from_str(
        CONTRACT_VECTORS
            .lines()
            .nth(1)
            .expect("the manifest vector must be second"),
    )
    .expect("the manifest vector must be valid JSON");
    assert_eq!(
        babylon_kernel::sha256_of(CONTRACT_SCHEMA),
        MATERIAL_CIRCUIT_V2_SOURCE_SHA256
    );
    assert_eq!(
        manifest["data"]["schema_sha256"],
        hex(MATERIAL_CIRCUIT_V2_SOURCE_SHA256)
    );
    assert_eq!(
        manifest["data"]["route_legs_per_route"],
        MAX_ROUTE_LEGS_PER_ROUTE_V2
    );
    assert_eq!(
        manifest["data"]["freight_resource_groups"],
        babylon_material_circuit::MAX_FREIGHT_RESOURCE_GROUPS_V2
    );
}

#[test]
fn v2_decoder_refuses_domain_version_truncation_and_trailing_bytes() {
    let bytes = encode_material_circuit_state_v2(&base_state()).expect("base state must encode");
    let mut wrong_domain = bytes.clone();
    wrong_domain[0] ^= 1;
    assert_eq!(
        decode_material_circuit_state_v2(&wrong_domain),
        Err(babylon_material_circuit::MaterialCircuitErrorV2::WireDomain)
    );

    let version_index = MATERIAL_CIRCUIT_STATE_V2_DOMAIN_BYTES.len() + 1;
    let mut wrong_version = bytes.clone();
    wrong_version[version_index + 1] = 3;
    assert_eq!(
        decode_material_circuit_state_v2(&wrong_version),
        Err(babylon_material_circuit::MaterialCircuitErrorV2::WireVersion)
    );
    assert_eq!(
        decode_material_circuit_state_v2(&bytes[..bytes.len() - 1]),
        Err(babylon_material_circuit::MaterialCircuitErrorV2::WireTruncated)
    );
    let mut trailing = bytes;
    trailing.push(0);
    assert_eq!(
        decode_material_circuit_state_v2(&trailing),
        Err(babylon_material_circuit::MaterialCircuitErrorV2::WireTrailing)
    );
}

#[test]
fn v2_decoder_refuses_unknown_access_mode_and_noncanonical_rows() {
    const HEADER_BYTES: usize = 44;
    const SITE_ROW_BYTES: usize = 64;
    const FIRST_SITE_ROW: usize = HEADER_BYTES + 4;
    const ORDER_ACCESS_MODE_INDEX: usize = 740;

    let bytes = encode_material_circuit_state_v2(&base_state()).expect("base state must encode");
    let mut unknown_mode = bytes.clone();
    unknown_mode[ORDER_ACCESS_MODE_INDEX] = 2;
    assert_eq!(
        decode_material_circuit_state_v2(&unknown_mode),
        Err(babylon_material_circuit::MaterialCircuitErrorV2::WireEnum)
    );

    let mut noncanonical = bytes;
    let first = noncanonical[FIRST_SITE_ROW..FIRST_SITE_ROW + SITE_ROW_BYTES].to_vec();
    let second =
        noncanonical[FIRST_SITE_ROW + SITE_ROW_BYTES..FIRST_SITE_ROW + 2 * SITE_ROW_BYTES].to_vec();
    noncanonical[FIRST_SITE_ROW..FIRST_SITE_ROW + SITE_ROW_BYTES].copy_from_slice(&second);
    noncanonical[FIRST_SITE_ROW + SITE_ROW_BYTES..FIRST_SITE_ROW + 2 * SITE_ROW_BYTES]
        .copy_from_slice(&first);
    assert_eq!(
        decode_material_circuit_state_v2(&noncanonical),
        Err(babylon_material_circuit::MaterialCircuitErrorV2::WireNoncanonical)
    );
}

#[test]
fn every_v2_refusal_code_round_trips_and_registry_is_closed() {
    for code in 1_u16..=18 {
        let error = babylon_material_circuit::MaterialCircuitErrorV2::try_from(code)
            .expect("declared code must decode");
        assert_eq!(u16::from(error), code);
    }
    assert!(babylon_material_circuit::MaterialCircuitErrorV2::try_from(0).is_err());
    assert!(babylon_material_circuit::MaterialCircuitErrorV2::try_from(19).is_err());
}

#[test]
fn severed_corridor_changes_only_its_routed_inventory_and_realization() {
    let full = advance_material_circuit_v2(&two_route_state()).expect("both routes must close");
    let mut severed_state = two_route_state();
    severed_state
        .corridor_capacities
        .retain(|row| row.corridor_id != corridor(CORRIDOR));
    let severed = advance_material_circuit_v2(&severed_state).expect("severed route must close");

    assert_eq!(shipped_for(&full.state, order(ORDER)), 4);
    assert_eq!(shipped_for(&severed.state, order(ORDER)), 0);
    assert_eq!(shipped_for(&full.state, order(14)), 4);
    assert_eq!(shipped_for(&severed.state, order(14)), 4);

    let full_arrival = advance_material_circuit_v2(&full.state).expect("full arrival must close");
    let severed_arrival =
        advance_material_circuit_v2(&severed.state).expect("severed arrival must close");
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
                    state.corridor_capacities[0].available = corridor_available;
                    state.orders[0].ordered = first_requested;
                    state.backlog[0].quantity = first_requested;
                    state.orders.push(OrderRowV2 {
                        order_id: order(17),
                        ordered: second_requested,
                        ..state.orders[0].clone()
                    });
                    state.backlog.push(BacklogRowV1 {
                        order_id: order(17),
                        quantity: second_requested,
                    });
                    let mut reversed = state.clone();
                    reversed.orders.reverse();
                    reversed.backlog.reverse();

                    let outcome =
                        advance_material_circuit_v2(&state).expect("allocation must close");
                    let twin = advance_material_circuit_v2(&reversed)
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
                        material_circuit_state_v2_digest(&outcome.state),
                        material_circuit_state_v2_digest(&twin.state)
                    );
                }
            }
        }
    }
}
