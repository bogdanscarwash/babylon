use babylon_material_circuit::{
    advance_material_circuit_v3, material_circuit_state_v3_digest, BacklogRowV1, CapacityRowV1,
    CorridorCapacityV3, CorridorIdV2, FreightMassCoefficientV3, GoodIdV1, InputOutputCoefficientV1,
    InventoryRowV1, LaborCapacityRowV1, LaborCoefficientV1, LogisticsNodeIdV2,
    MaterialCircuitErrorV3, MaterialCircuitStateV3, OrderAccessModeV1, OrderIdV1, OrderRowV2,
    ProcessIdV1, ProcessOutputV1, ProductionCommitmentV1, RouteIdV2, RouteStageCapacityV3,
    RouteStageV3, SiteIdV1, SiteLogisticsNodeV2, SupplierRouteV3, SupplierTransportV3, UnitIdV1,
    MAX_MATERIAL_CIRCUIT_ROWS_V1,
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

fn process(byte: u8) -> ProcessIdV1 {
    ProcessIdV1::from_bytes([byte; 32])
}

fn order(byte: u8) -> OrderIdV1 {
    OrderIdV1::from_bytes([byte; 32])
}

fn numbered_good(index: usize) -> GoodIdV1 {
    let mut bytes = [0_u8; 32];
    let number = u64::try_from(index).expect("material row ceiling fits u64");
    bytes[24..].copy_from_slice(&number.to_be_bytes());
    GoodIdV1::from_bytes(bytes)
}

const SUPPLIER: u8 = 1;
const FACTORY: u8 = 2;
const GRAIN: u8 = 3;
const BREAD: u8 = 4;
const GOODS_UNIT: u8 = 5;
const LABOR_UNIT: u8 = 6;
const BAKERY: u8 = 7;
const GRAIN_ORDER: u8 = 8;
fn capacity_rows() -> Vec<CapacityRowV1> {
    [1, 2, 3]
        .into_iter()
        .map(|period| CapacityRowV1 {
            process_id: process(BAKERY),
            site_id: site(FACTORY),
            period,
            available_batches: 3,
        })
        .collect()
}

fn labor_rows() -> Vec<LaborCapacityRowV1> {
    [1, 2, 3]
        .into_iter()
        .map(|period| LaborCapacityRowV1 {
            site_id: site(FACTORY),
            unit_id: unit(LABOR_UNIT),
            period,
            available: 12,
        })
        .collect()
}

fn base_state() -> MaterialCircuitStateV3 {
    MaterialCircuitStateV3 {
        period: 1,
        merchants: vec![],
        handling_coefficients: vec![],
        final_demand_principals: vec![],
        final_demand_orders: vec![],
        site_logistics_nodes: vec![
            SiteLogisticsNodeV2 {
                site_id: site(SUPPLIER),
                node_id: LogisticsNodeIdV2::from_bytes([SUPPLIER; 32]),
            },
            SiteLogisticsNodeV2 {
                site_id: site(FACTORY),
                node_id: LogisticsNodeIdV2::from_bytes([FACTORY; 32]),
            },
        ],
        freight_mass_coefficients: vec![FreightMassCoefficientV3 {
            good_id: good(GRAIN),
            unit_id: unit(GOODS_UNIT),
            grams_per_unit: 1,
        }],
        route_stages: vec![RouteStageV3 {
            route_id: RouteIdV2::from_bytes([GRAIN_ORDER; 32]),
            stage_index: 0,
            from_node_id: LogisticsNodeIdV2::from_bytes([SUPPLIER; 32]),
            to_node_id: LogisticsNodeIdV2::from_bytes([FACTORY; 32]),
            travel_periods: 1,
            loss_ppm: 0,
        }],
        route_stage_capacities: vec![RouteStageCapacityV3 {
            route_id: RouteIdV2::from_bytes([GRAIN_ORDER; 32]),
            stage_index: 0,
            corridor_id: CorridorIdV2::from_bytes([9; 32]),
        }],
        corridor_capacities: (1..=3)
            .map(|period| CorridorCapacityV3 {
                corridor_id: CorridorIdV2::from_bytes([9; 32]),
                period,
                available_grams: u64::MAX,
            })
            .collect(),
        process_outputs: vec![ProcessOutputV1 {
            process_id: process(BAKERY),
            site_id: site(FACTORY),
            good_id: good(BREAD),
            unit_id: unit(GOODS_UNIT),
            quantity_per_batch: 2,
        }],
        input_coefficients: vec![InputOutputCoefficientV1 {
            process_id: process(BAKERY),
            good_id: good(GRAIN),
            unit_id: unit(GOODS_UNIT),
            quantity_per_batch: 3,
        }],
        labor_coefficients: vec![LaborCoefficientV1 {
            process_id: process(BAKERY),
            unit_id: unit(LABOR_UNIT),
            quantity_per_batch: 4,
        }],
        supplier_routes: vec![SupplierRouteV3 {
            buyer_site_id: site(FACTORY),
            supplier_site_id: site(SUPPLIER),
            good_id: good(GRAIN),
            unit_id: unit(GOODS_UNIT),
            transport_kind: SupplierTransportV3::Staged,
            route_id: RouteIdV2::from_bytes([GRAIN_ORDER; 32]),
        }],
        inventory: vec![
            InventoryRowV1 {
                site_id: site(SUPPLIER),
                good_id: good(GRAIN),
                unit_id: unit(GOODS_UNIT),
                quantity: 10,
            },
            InventoryRowV1 {
                site_id: site(FACTORY),
                good_id: good(GRAIN),
                unit_id: unit(GOODS_UNIT),
                quantity: 0,
            },
            InventoryRowV1 {
                site_id: site(FACTORY),
                good_id: good(BREAD),
                unit_id: unit(GOODS_UNIT),
                quantity: 0,
            },
        ],
        orders: vec![OrderRowV2 {
            order_id: order(GRAIN_ORDER),
            access_mode: OrderAccessModeV1::CommoditySale,
            buyer_site_id: site(FACTORY),
            supplier_site_id: site(SUPPLIER),
            good_id: good(GRAIN),
            unit_id: unit(GOODS_UNIT),
            ordered: 6,
            shipped: 0,
            lost: 0,
            delivered: 0,
            realized: 0,
        }],
        backlog: vec![BacklogRowV1 {
            order_id: order(GRAIN_ORDER),
            quantity: 6,
        }],
        freight: Vec::new(),
        capacities: capacity_rows(),
        labor: labor_rows(),
        production_commitments: Vec::new(),
    }
}

fn inventory_quantity(state: &MaterialCircuitStateV3, site_byte: u8, good_byte: u8) -> u64 {
    state
        .inventory
        .iter()
        .take(babylon_material_circuit::MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .find(|row| row.site_id == site(site_byte) && row.good_id == good(good_byte))
        .map_or(0, |row| row.quantity)
}

#[test]
fn shipment_precedes_arrival_and_realization() {
    let first = advance_material_circuit_v3(&base_state()).expect("period one must close");
    assert_eq!(first.state.period, 2);
    assert_eq!(first.dispatches.len(), 1);
    assert!(first.arrivals.is_empty());
    assert!(first.realizations.is_empty());
    assert_eq!(first.state.orders[0].shipped, 6);
    assert_eq!(first.state.orders[0].delivered, 0);
    assert_eq!(first.state.orders[0].realized, 0);
    assert_eq!(first.state.backlog[0].quantity, 0);
    assert_eq!(inventory_quantity(&first.state, SUPPLIER, GRAIN), 4);
    assert_eq!(inventory_quantity(&first.state, FACTORY, GRAIN), 0);

    let second = advance_material_circuit_v3(&first.state).expect("period two must close");
    assert_eq!(second.state.period, 3);
    assert_eq!(second.arrivals.len(), 1);
    assert_eq!(second.deliveries.len(), 1);
    assert_eq!(second.realizations.len(), 1);
    assert_eq!(second.state.orders[0].delivered, 6);
    assert_eq!(second.state.orders[0].realized, 6);
    assert_eq!(inventory_quantity(&second.state, FACTORY, GRAIN), 6);
    assert_eq!(second.state.production_commitments[0].period, 3);
    assert_eq!(second.state.production_commitments[0].planned_batches, 2);
}

#[test]
fn delivered_inputs_feed_the_following_period_not_the_arrival_period() {
    let first = advance_material_circuit_v3(&base_state()).expect("period one must close");
    assert!(first.state.production_commitments.is_empty());

    let second = advance_material_circuit_v3(&first.state).expect("period two must close");
    assert_eq!(second.state.production_commitments[0].planned_batches, 2);
    assert!(second.production.is_empty());

    let third = advance_material_circuit_v3(&second.state).expect("period three must close");
    assert_eq!(third.production[0].planned_batches, 2);
    assert_eq!(third.production[0].produced_batches, 2);
    assert_eq!(inventory_quantity(&third.state, FACTORY, GRAIN), 0);
    assert_eq!(inventory_quantity(&third.state, FACTORY, BREAD), 4);
}

#[test]
fn severed_supplier_relation_causes_backlog_without_creating_goods() {
    let mut state = base_state();
    state.supplier_routes.clear();

    let first = advance_material_circuit_v3(&state).expect("missing supply is a material outcome");
    assert!(first.dispatches.is_empty());
    assert_eq!(first.state.orders[0].shipped, 0);
    assert_eq!(first.state.backlog[0].quantity, 6);
    assert_eq!(inventory_quantity(&first.state, SUPPLIER, GRAIN), 10);

    let second = advance_material_circuit_v3(&first.state).expect("period two must close");
    assert!(second.arrivals.is_empty());
    assert!(second.state.production_commitments.is_empty());
}

#[test]
fn missing_stock_and_labor_are_material_shortages_not_engine_errors() {
    let mut state = base_state();
    state.inventory.remove(0);
    state.labor = state
        .labor
        .into_iter()
        .take(babylon_material_circuit::MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .filter(|row| row.period != 1)
        .collect();
    state.production_commitments = vec![ProductionCommitmentV1 {
        process_id: process(BAKERY),
        site_id: site(FACTORY),
        period: 1,
        planned_batches: 3,
    }];

    let outcome = advance_material_circuit_v3(&state).expect("zero supply must still close");
    assert_eq!(outcome.production[0].produced_batches, 0);
    assert!(outcome.dispatches.is_empty());
    assert_eq!(outcome.state.backlog[0].quantity, 6);
    assert_eq!(inventory_quantity(&outcome.state, FACTORY, BREAD), 0);
}

#[test]
fn leontief_output_is_bounded_by_labor_capacity_and_inputs() {
    let mut state = base_state();
    state.orders.clear();
    state.backlog.clear();
    state.inventory[1].quantity = 30;
    state.capacities[0].available_batches = 4;
    state.labor[0].available = 10;
    state.production_commitments = vec![ProductionCommitmentV1 {
        process_id: process(BAKERY),
        site_id: site(FACTORY),
        period: 1,
        planned_batches: 10,
    }];

    let outcome = advance_material_circuit_v3(&state).expect("bounded production must close");
    assert_eq!(outcome.production[0].planned_batches, 10);
    assert_eq!(outcome.production[0].produced_batches, 2);
    assert_eq!(inventory_quantity(&outcome.state, FACTORY, GRAIN), 24);
    assert_eq!(inventory_quantity(&outcome.state, FACTORY, BREAD), 4);
}

#[test]
fn production_debits_all_inputs_before_crediting_any_output() {
    let producer = process(1);
    let consumer = process(2);
    let shared_good = good(3);
    let consumer_output = good(4);
    let production_site = site(5);
    let goods_unit = unit(6);
    let labor_unit = unit(7);
    let state = MaterialCircuitStateV3 {
        period: 1,
        merchants: vec![],
        handling_coefficients: vec![],
        final_demand_principals: vec![],
        final_demand_orders: vec![],
        site_logistics_nodes: vec![],
        freight_mass_coefficients: vec![],
        route_stages: vec![],
        route_stage_capacities: vec![],
        corridor_capacities: vec![],
        process_outputs: vec![
            ProcessOutputV1 {
                process_id: producer,
                site_id: production_site,
                good_id: shared_good,
                unit_id: goods_unit,
                quantity_per_batch: 1,
            },
            ProcessOutputV1 {
                process_id: consumer,
                site_id: production_site,
                good_id: consumer_output,
                unit_id: goods_unit,
                quantity_per_batch: 1,
            },
        ],
        input_coefficients: vec![InputOutputCoefficientV1 {
            process_id: consumer,
            good_id: shared_good,
            unit_id: goods_unit,
            quantity_per_batch: 1,
        }],
        labor_coefficients: vec![
            LaborCoefficientV1 {
                process_id: producer,
                unit_id: labor_unit,
                quantity_per_batch: 1,
            },
            LaborCoefficientV1 {
                process_id: consumer,
                unit_id: labor_unit,
                quantity_per_batch: 1,
            },
        ],
        supplier_routes: Vec::new(),
        inventory: vec![InventoryRowV1 {
            site_id: production_site,
            good_id: shared_good,
            unit_id: goods_unit,
            quantity: u64::MAX,
        }],
        orders: Vec::new(),
        backlog: Vec::new(),
        freight: Vec::new(),
        capacities: vec![
            CapacityRowV1 {
                process_id: producer,
                site_id: production_site,
                period: 1,
                available_batches: 1,
            },
            CapacityRowV1 {
                process_id: consumer,
                site_id: production_site,
                period: 1,
                available_batches: 1,
            },
        ],
        labor: vec![LaborCapacityRowV1 {
            site_id: production_site,
            unit_id: labor_unit,
            period: 1,
            available: 2,
        }],
        production_commitments: vec![
            ProductionCommitmentV1 {
                process_id: producer,
                site_id: production_site,
                period: 1,
                planned_batches: 1,
            },
            ProductionCommitmentV1 {
                process_id: consumer,
                site_id: production_site,
                period: 1,
                planned_batches: 1,
            },
        ],
    };

    let outcome = advance_material_circuit_v3(&state).expect("net-conserved production must close");
    assert_eq!(
        inventory_quantity(&outcome.state, 5, 3),
        u64::MAX,
        "the shared inventory must be debited before its output is credited"
    );
    assert_eq!(inventory_quantity(&outcome.state, 5, 4), 1);
}

#[test]
fn proportional_production_uses_u128_for_unbounded_requested_units() {
    let mut state = production_state_for_numeric_boundary();
    state.production_commitments[0].planned_batches = u64::MAX;
    state.capacities[0].available_batches = u64::MAX;
    state.labor[0].available = u64::MAX;

    let outcome = advance_material_circuit_v3(&state)
        .expect("scarcity must bound an intermediate request larger than u64");
    assert_eq!(outcome.production[0].produced_batches, 1);
    assert_eq!(inventory_quantity(&outcome.state, 1, 2), 0);
    assert_eq!(inventory_quantity(&outcome.state, 1, 3), 1);
}

fn production_state_for_numeric_boundary() -> MaterialCircuitStateV3 {
    let production_site = site(1);
    let input_good = good(2);
    let output_good = good(3);
    let goods_unit = unit(4);
    let labor_unit = unit(5);
    let process_id = process(6);
    MaterialCircuitStateV3 {
        period: 1,
        merchants: vec![],
        handling_coefficients: vec![],
        final_demand_principals: vec![],
        final_demand_orders: vec![],
        site_logistics_nodes: vec![],
        freight_mass_coefficients: vec![],
        route_stages: vec![],
        route_stage_capacities: vec![],
        corridor_capacities: vec![],
        process_outputs: vec![ProcessOutputV1 {
            process_id,
            site_id: production_site,
            good_id: output_good,
            unit_id: goods_unit,
            quantity_per_batch: 1,
        }],
        input_coefficients: vec![InputOutputCoefficientV1 {
            process_id,
            good_id: input_good,
            unit_id: goods_unit,
            quantity_per_batch: 2,
        }],
        labor_coefficients: vec![LaborCoefficientV1 {
            process_id,
            unit_id: labor_unit,
            quantity_per_batch: 1,
        }],
        supplier_routes: Vec::new(),
        inventory: vec![
            InventoryRowV1 {
                site_id: production_site,
                good_id: input_good,
                unit_id: goods_unit,
                quantity: 2,
            },
            InventoryRowV1 {
                site_id: production_site,
                good_id: output_good,
                unit_id: goods_unit,
                quantity: 0,
            },
        ],
        orders: Vec::new(),
        backlog: Vec::new(),
        freight: Vec::new(),
        capacities: vec![CapacityRowV1 {
            process_id,
            site_id: production_site,
            period: 1,
            available_batches: 1,
        }],
        labor: vec![LaborCapacityRowV1 {
            site_id: production_site,
            unit_id: labor_unit,
            period: 1,
            available: 1,
        }],
        production_commitments: vec![ProductionCommitmentV1 {
            process_id,
            site_id: production_site,
            period: 1,
            planned_batches: 1,
        }],
    }
}

#[test]
fn zero_production_does_not_create_an_empty_inventory_row() {
    let production_site = site(1);
    let output_good = GoodIdV1::from_bytes([0xff; 32]);
    let goods_unit = unit(2);
    let labor_unit = unit(3);
    let process_id = process(4);
    let inventory = (0..MAX_MATERIAL_CIRCUIT_ROWS_V1)
        .map(|index| InventoryRowV1 {
            site_id: production_site,
            good_id: numbered_good(index),
            unit_id: goods_unit,
            quantity: 1,
        })
        .collect();
    let state = MaterialCircuitStateV3 {
        period: 1,
        merchants: vec![],
        handling_coefficients: vec![],
        final_demand_principals: vec![],
        final_demand_orders: vec![],
        site_logistics_nodes: vec![],
        freight_mass_coefficients: vec![],
        route_stages: vec![],
        route_stage_capacities: vec![],
        corridor_capacities: vec![],
        process_outputs: vec![ProcessOutputV1 {
            process_id,
            site_id: production_site,
            good_id: output_good,
            unit_id: goods_unit,
            quantity_per_batch: 1,
        }],
        input_coefficients: Vec::new(),
        labor_coefficients: vec![LaborCoefficientV1 {
            process_id,
            unit_id: labor_unit,
            quantity_per_batch: 1,
        }],
        supplier_routes: Vec::new(),
        inventory,
        orders: Vec::new(),
        backlog: Vec::new(),
        freight: Vec::new(),
        capacities: Vec::new(),
        labor: Vec::new(),
        production_commitments: vec![ProductionCommitmentV1 {
            process_id,
            site_id: production_site,
            period: 1,
            planned_batches: 1,
        }],
    };

    let outcome = advance_material_circuit_v3(&state).expect("zero production is a valid outcome");
    assert_eq!(outcome.production[0].produced_batches, 0);
    assert_eq!(outcome.state.inventory.len(), MAX_MATERIAL_CIRCUIT_ROWS_V1);
    assert!(outcome
        .state
        .inventory
        .iter()
        .all(|row| row.good_id != output_good));
}

#[test]
fn proportional_stock_allocation_has_no_order_priority() {
    let mut state = base_state();
    state.process_outputs.clear();
    state.input_coefficients.clear();
    state.labor_coefficients.clear();
    state.capacities.clear();
    state.labor.clear();
    state.inventory[0].quantity = 4;
    state.orders = vec![
        OrderRowV2 {
            ordered: 4,
            ..state.orders[0].clone()
        },
        OrderRowV2 {
            order_id: order(9),
            ordered: 6,
            ..state.orders[0].clone()
        },
    ];
    state.backlog = vec![
        BacklogRowV1 {
            order_id: order(GRAIN_ORDER),
            quantity: 4,
        },
        BacklogRowV1 {
            order_id: order(9),
            quantity: 6,
        },
    ];

    let mut reversed = state.clone();
    reversed.orders.reverse();
    reversed.backlog.reverse();

    let a = advance_material_circuit_v3(&state).expect("allocation must close");
    let b = advance_material_circuit_v3(&reversed).expect("permuted allocation must close");
    assert_eq!(a.state.orders[0].shipped, 1);
    assert_eq!(a.state.orders[1].shipped, 2);
    assert_eq!(inventory_quantity(&a.state, SUPPLIER, GRAIN), 1);
    assert_eq!(
        material_circuit_state_v3_digest(&a.state),
        material_circuit_state_v3_digest(&b.state)
    );
}

#[test]
fn arithmetic_refusal_does_not_publish_a_partial_state() {
    let first = advance_material_circuit_v3(&base_state()).unwrap();
    let mut state = first.state;
    state
        .inventory
        .iter_mut()
        .find(|row| row.site_id == site(FACTORY) && row.good_id == good(GRAIN))
        .unwrap()
        .quantity = u64::MAX;
    let before = material_circuit_state_v3_digest(&state).unwrap();
    assert_eq!(
        advance_material_circuit_v3(&state),
        Err(MaterialCircuitErrorV3::Arithmetic)
    );
    assert_eq!(material_circuit_state_v3_digest(&state).unwrap(), before);
}

#[test]
fn duplicate_dispatch_identity_is_rejected_even_when_arrival_periods_differ() {
    let first = advance_material_circuit_v3(&base_state()).unwrap();
    let mut state = first.state;
    let mut duplicate = state.freight[0].clone();
    duplicate.stage_arrival_period += 1;
    state.freight.push(duplicate);
    assert_eq!(
        advance_material_circuit_v3(&state),
        Err(MaterialCircuitErrorV3::DuplicateRow)
    );
}
