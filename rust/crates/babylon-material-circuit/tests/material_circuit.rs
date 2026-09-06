use babylon_material_circuit::{
    advance_material_circuit_v1, decode_material_circuit_state_v1,
    encode_material_circuit_state_v1, material_circuit_state_v1_digest, BacklogRowV1,
    CapacityRowV1, GoodIdV1, InputOutputCoefficientV1, InventoryRowV1, LaborCapacityRowV1,
    LaborCoefficientV1, MaterialCircuitErrorV1, MaterialCircuitStateV1, OrderAccessModeV1,
    OrderIdV1, OrderRowV1, ProcessIdV1, ProcessOutputV1, ProductionCommitmentV1, SiteIdV1,
    SupplierCandidateV1, UnitIdV1, MATERIAL_CIRCUIT_STATE_V1_DOMAIN_BYTES,
    MAX_MATERIAL_CIRCUIT_ROWS_V1, MAX_PRODUCTION_RESOURCE_GROUPS_V1,
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
const CONTRACT_VECTORS: &str =
    include_str!("../../../../contracts/material_circuit_v1_vectors.jsonl");

fn hex(bytes: [u8; 32]) -> String {
    use std::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(64), |mut output, byte| {
            let _ = write!(output, "{byte:02x}");
            output
        })
}

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

fn base_state() -> MaterialCircuitStateV1 {
    MaterialCircuitStateV1 {
        period: 1,
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
        supplier_candidates: vec![SupplierCandidateV1 {
            buyer_site_id: site(FACTORY),
            supplier_site_id: site(SUPPLIER),
            good_id: good(GRAIN),
            unit_id: unit(GOODS_UNIT),
            transit_delay_periods: 1,
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
        orders: vec![OrderRowV1 {
            order_id: order(GRAIN_ORDER),
            access_mode: OrderAccessModeV1::CommoditySale,
            buyer_site_id: site(FACTORY),
            supplier_site_id: site(SUPPLIER),
            good_id: good(GRAIN),
            unit_id: unit(GOODS_UNIT),
            ordered: 6,
            shipped: 0,
            delivered: 0,
            realized: 0,
        }],
        backlog: vec![BacklogRowV1 {
            order_id: order(GRAIN_ORDER),
            quantity: 6,
        }],
        transit: Vec::new(),
        capacities: capacity_rows(),
        labor: labor_rows(),
        production_commitments: Vec::new(),
    }
}

fn inventory_quantity(state: &MaterialCircuitStateV1, site_byte: u8, good_byte: u8) -> u64 {
    state
        .inventory
        .iter()
        .take(babylon_material_circuit::MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .find(|row| row.site_id == site(site_byte) && row.good_id == good(good_byte))
        .map_or(0, |row| row.quantity)
}

#[test]
fn shipment_precedes_arrival_and_realization() {
    let first = advance_material_circuit_v1(&base_state()).expect("period one must close");
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

    let second = advance_material_circuit_v1(&first.state).expect("period two must close");
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
    let first = advance_material_circuit_v1(&base_state()).expect("period one must close");
    assert!(first.state.production_commitments.is_empty());

    let second = advance_material_circuit_v1(&first.state).expect("period two must close");
    assert_eq!(second.state.production_commitments[0].planned_batches, 2);
    assert!(second.production.is_empty());

    let third = advance_material_circuit_v1(&second.state).expect("period three must close");
    assert_eq!(third.production[0].planned_batches, 2);
    assert_eq!(third.production[0].produced_batches, 2);
    assert_eq!(inventory_quantity(&third.state, FACTORY, GRAIN), 0);
    assert_eq!(inventory_quantity(&third.state, FACTORY, BREAD), 4);
}

#[test]
fn severed_supplier_relation_causes_backlog_without_creating_goods() {
    let mut state = base_state();
    state.supplier_candidates.clear();

    let first = advance_material_circuit_v1(&state).expect("missing supply is a material outcome");
    assert!(first.dispatches.is_empty());
    assert_eq!(first.state.orders[0].shipped, 0);
    assert_eq!(first.state.backlog[0].quantity, 6);
    assert_eq!(inventory_quantity(&first.state, SUPPLIER, GRAIN), 10);

    let second = advance_material_circuit_v1(&first.state).expect("period two must close");
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

    let outcome = advance_material_circuit_v1(&state).expect("zero supply must still close");
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

    let outcome = advance_material_circuit_v1(&state).expect("bounded production must close");
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
    let state = MaterialCircuitStateV1 {
        period: 1,
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
        supplier_candidates: Vec::new(),
        inventory: vec![InventoryRowV1 {
            site_id: production_site,
            good_id: shared_good,
            unit_id: goods_unit,
            quantity: u64::MAX,
        }],
        orders: Vec::new(),
        backlog: Vec::new(),
        transit: Vec::new(),
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

    let outcome = advance_material_circuit_v1(&state).expect("net-conserved production must close");
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

    let outcome = advance_material_circuit_v1(&state)
        .expect("scarcity must bound an intermediate request larger than u64");
    assert_eq!(outcome.production[0].produced_batches, 1);
    assert_eq!(inventory_quantity(&outcome.state, 1, 2), 0);
    assert_eq!(inventory_quantity(&outcome.state, 1, 3), 1);
}

fn production_state_for_numeric_boundary() -> MaterialCircuitStateV1 {
    let production_site = site(1);
    let input_good = good(2);
    let output_good = good(3);
    let goods_unit = unit(4);
    let labor_unit = unit(5);
    let process_id = process(6);
    MaterialCircuitStateV1 {
        period: 1,
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
        supplier_candidates: Vec::new(),
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
        transit: Vec::new(),
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
    let state = MaterialCircuitStateV1 {
        period: 1,
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
        supplier_candidates: Vec::new(),
        inventory,
        orders: Vec::new(),
        backlog: Vec::new(),
        transit: Vec::new(),
        capacities: Vec::new(),
        labor: Vec::new(),
        production_commitments: vec![ProductionCommitmentV1 {
            process_id,
            site_id: production_site,
            period: 1,
            planned_batches: 1,
        }],
    };

    let outcome = advance_material_circuit_v1(&state).expect("zero production is a valid outcome");
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
        OrderRowV1 {
            ordered: 4,
            ..state.orders[0].clone()
        },
        OrderRowV1 {
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

    let a = advance_material_circuit_v1(&state).expect("allocation must close");
    let b = advance_material_circuit_v1(&reversed).expect("permuted allocation must close");
    assert_eq!(a.state.orders[0].shipped, 1);
    assert_eq!(a.state.orders[1].shipped, 2);
    assert_eq!(inventory_quantity(&a.state, SUPPLIER, GRAIN), 1);
    assert_eq!(
        material_circuit_state_v1_digest(&a.state),
        material_circuit_state_v1_digest(&b.state)
    );
}

#[test]
fn arithmetic_refusal_does_not_publish_a_partial_state() {
    let mut state = base_state();
    state.period = 2;
    state.capacities = state
        .capacities
        .into_iter()
        .take(babylon_material_circuit::MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .filter(|row| row.period >= 2)
        .collect();
    state.labor = state
        .labor
        .into_iter()
        .take(babylon_material_circuit::MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
        .filter(|row| row.period >= 2)
        .collect();
    state.inventory[1].quantity = u64::MAX;
    state.orders[0].shipped = 6;
    state.backlog[0].quantity = 0;
    state.transit.push(babylon_material_circuit::TransitLotV1 {
        order_id: order(GRAIN_ORDER),
        dispatch_period: 1,
        arrival_period: 2,
        source_site_id: site(SUPPLIER),
        destination_site_id: site(FACTORY),
        good_id: good(GRAIN),
        unit_id: unit(GOODS_UNIT),
        quantity: 6,
    });
    let before = material_circuit_state_v1_digest(&state).expect("opening state must encode");

    assert_eq!(
        advance_material_circuit_v1(&state),
        Err(MaterialCircuitErrorV1::Arithmetic)
    );
    assert_eq!(
        material_circuit_state_v1_digest(&state).expect("refusal must not mutate input"),
        before
    );
}

#[test]
fn canonical_state_bytes_and_digest_are_pinned() {
    let bytes = encode_material_circuit_state_v1(&base_state()).expect("base state must encode");
    let digest = material_circuit_state_v1_digest(&base_state()).expect("base state must hash");
    assert_eq!(bytes.len(), 1_555);
    assert_eq!(
        digest,
        [
            0x35, 0x76, 0xba, 0xa1, 0xaf, 0x2a, 0x38, 0xbe, 0x8a, 0x13, 0x76, 0x25, 0x9d, 0xc4,
            0x42, 0x3a, 0x47, 0x06, 0x07, 0xad, 0x9e, 0xef, 0x59, 0x69, 0x71, 0x59, 0x31, 0xe1,
            0x6b, 0x30, 0x62, 0x0a,
        ]
    );
    assert_eq!(
        decode_material_circuit_state_v1(&bytes).expect("canonical bytes must decode"),
        base_state()
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
        manifest["data"]["production_resource_groups"],
        MAX_PRODUCTION_RESOURCE_GROUPS_V1
    );
}

#[test]
fn decoder_refuses_wrong_domain_version_truncation_and_trailing_bytes() {
    let bytes = encode_material_circuit_state_v1(&base_state()).expect("base state must encode");
    let mut wrong_domain = bytes.clone();
    wrong_domain[0] ^= 1;
    assert_eq!(
        decode_material_circuit_state_v1(&wrong_domain),
        Err(MaterialCircuitErrorV1::WireDomain)
    );
    let version_index = MATERIAL_CIRCUIT_STATE_V1_DOMAIN_BYTES.len() + 1;
    let mut wrong_version = bytes.clone();
    wrong_version[version_index + 1] = 2;
    assert_eq!(
        decode_material_circuit_state_v1(&wrong_version),
        Err(MaterialCircuitErrorV1::WireVersion)
    );
    assert_eq!(
        decode_material_circuit_state_v1(&bytes[..bytes.len() - 1]),
        Err(MaterialCircuitErrorV1::WireTruncated)
    );
    let mut trailing = bytes;
    trailing.push(0);
    assert_eq!(
        decode_material_circuit_state_v1(&trailing),
        Err(MaterialCircuitErrorV1::WireTrailing)
    );
}

#[test]
fn decoder_refuses_unknown_access_mode_and_noncanonical_row_order() {
    const INVENTORY_ROW_BYTES: usize = 104;
    const INVENTORY_ROWS_START: usize = 506;
    const ORDER_ACCESS_MODE_INDEX: usize = 854;
    let bytes = encode_material_circuit_state_v1(&base_state()).expect("base state must encode");
    let mut unknown_mode = bytes.clone();
    unknown_mode[ORDER_ACCESS_MODE_INDEX] = 2;
    assert_eq!(
        decode_material_circuit_state_v1(&unknown_mode),
        Err(MaterialCircuitErrorV1::WireEnum)
    );

    let mut noncanonical = bytes;
    let first =
        noncanonical[INVENTORY_ROWS_START..INVENTORY_ROWS_START + INVENTORY_ROW_BYTES].to_vec();
    let second = noncanonical[INVENTORY_ROWS_START + INVENTORY_ROW_BYTES
        ..INVENTORY_ROWS_START + 2 * INVENTORY_ROW_BYTES]
        .to_vec();
    noncanonical[INVENTORY_ROWS_START..INVENTORY_ROWS_START + INVENTORY_ROW_BYTES]
        .copy_from_slice(&second);
    noncanonical[INVENTORY_ROWS_START + INVENTORY_ROW_BYTES
        ..INVENTORY_ROWS_START + 2 * INVENTORY_ROW_BYTES]
        .copy_from_slice(&first);
    assert_eq!(
        decode_material_circuit_state_v1(&noncanonical),
        Err(MaterialCircuitErrorV1::WireNoncanonical)
    );
}

#[test]
fn every_refusal_code_round_trips_and_the_registry_is_closed() {
    for code in 1_u16..=16 {
        let error = MaterialCircuitErrorV1::try_from(code).expect("declared code must decode");
        assert_eq!(u16::from(error), code);
    }
    assert!(MaterialCircuitErrorV1::try_from(0).is_err());
    assert!(MaterialCircuitErrorV1::try_from(17).is_err());
}

#[test]
fn duplicate_dispatch_identity_is_rejected_even_when_arrival_periods_differ() {
    let mut state = base_state();
    state.period = 2;
    state.orders[0].shipped = 6;
    state.backlog[0].quantity = 0;
    state.transit = [2_u64, 3]
        .into_iter()
        .map(|arrival_period| babylon_material_circuit::TransitLotV1 {
            order_id: order(GRAIN_ORDER),
            dispatch_period: 1,
            arrival_period,
            source_site_id: site(SUPPLIER),
            destination_site_id: site(FACTORY),
            good_id: good(GRAIN),
            unit_id: unit(GOODS_UNIT),
            quantity: 3,
        })
        .collect();
    assert_eq!(
        advance_material_circuit_v1(&state),
        Err(MaterialCircuitErrorV1::DuplicateRow)
    );
}
