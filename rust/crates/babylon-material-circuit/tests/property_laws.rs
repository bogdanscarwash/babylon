use babylon_material_circuit::{
    advance_material_circuit_v1, material_circuit_state_v1_digest, BacklogRowV1, CapacityRowV1,
    GoodIdV1, InputOutputCoefficientV1, InventoryRowV1, LaborCapacityRowV1, LaborCoefficientV1,
    MaterialCircuitStateV1, OrderAccessModeV1, OrderIdV1, OrderRowV1, ProcessIdV1, ProcessOutputV1,
    ProductionCommitmentV1, SiteIdV1, SupplierCandidateV1, UnitIdV1,
};

fn id<const BYTE: u8>() -> [u8; 32] {
    [BYTE; 32]
}

fn allocation_state(available: u64, first: u64, second: u64) -> MaterialCircuitStateV1 {
    let supplier = SiteIdV1::from_bytes(id::<1>());
    let buyer = SiteIdV1::from_bytes(id::<2>());
    let good = GoodIdV1::from_bytes(id::<3>());
    let unit = UnitIdV1::from_bytes(id::<4>());
    let orders = [(5, first), (6, second)]
        .into_iter()
        .map(|(identity, ordered)| OrderRowV1 {
            order_id: OrderIdV1::from_bytes([identity; 32]),
            access_mode: OrderAccessModeV1::CommoditySale,
            buyer_site_id: buyer,
            supplier_site_id: supplier,
            good_id: good,
            unit_id: unit,
            ordered,
            shipped: 0,
            delivered: 0,
            realized: 0,
        })
        .collect::<Vec<_>>();
    MaterialCircuitStateV1 {
        period: 1,
        process_outputs: Vec::new(),
        input_coefficients: Vec::new(),
        labor_coefficients: Vec::new(),
        supplier_candidates: vec![SupplierCandidateV1 {
            buyer_site_id: buyer,
            supplier_site_id: supplier,
            good_id: good,
            unit_id: unit,
            transit_delay_periods: 1,
        }],
        inventory: vec![InventoryRowV1 {
            site_id: supplier,
            good_id: good,
            unit_id: unit,
            quantity: available,
        }],
        backlog: orders
            .iter()
            .map(|order| BacklogRowV1 {
                order_id: order.order_id,
                quantity: order.ordered,
            })
            .collect(),
        orders,
        transit: Vec::new(),
        capacities: Vec::new(),
        labor: Vec::new(),
        production_commitments: Vec::new(),
    }
}

#[test]
fn proportional_allocation_exhaustively_conserves_small_stocks() {
    for available in 0_u64..=20 {
        for first in 1_u64..=10 {
            for second in 1_u64..=10 {
                let state = allocation_state(available, first, second);
                let outcome = advance_material_circuit_v1(&state).expect("allocation must close");
                let total_requested = first + second;
                let shipped: u64 = outcome
                    .state
                    .orders
                    .iter()
                    .take(babylon_material_circuit::MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
                    .map(|row| row.shipped)
                    .sum();
                let closing = outcome.state.inventory[0].quantity;
                assert_eq!(closing + shipped, available);
                assert!(shipped <= available.min(total_requested));
                assert!(outcome
                    .state
                    .orders
                    .iter()
                    .take(babylon_material_circuit::MAX_MATERIAL_CIRCUIT_ROWS_V1 + 1)
                    .all(|row| row.shipped <= row.ordered));
                if available >= total_requested {
                    assert_eq!(shipped, total_requested);
                } else {
                    assert_eq!(
                        outcome.state.orders[0].shipped,
                        available * first / total_requested
                    );
                    assert_eq!(
                        outcome.state.orders[1].shipped,
                        available * second / total_requested
                    );
                }
                let mut reversed = state;
                reversed.orders.reverse();
                reversed.backlog.reverse();
                let twin = advance_material_circuit_v1(&reversed).expect("twin must close");
                assert_eq!(
                    material_circuit_state_v1_digest(&outcome.state),
                    material_circuit_state_v1_digest(&twin.state)
                );
            }
        }
    }
}

fn production_state(input: u64, labor: u64, capacity: u64) -> MaterialCircuitStateV1 {
    let site = SiteIdV1::from_bytes(id::<1>());
    let input_good = GoodIdV1::from_bytes(id::<2>());
    let output_good = GoodIdV1::from_bytes(id::<3>());
    let goods_unit = UnitIdV1::from_bytes(id::<4>());
    let labor_unit = UnitIdV1::from_bytes(id::<5>());
    let process = ProcessIdV1::from_bytes(id::<6>());
    MaterialCircuitStateV1 {
        period: 1,
        process_outputs: vec![ProcessOutputV1 {
            process_id: process,
            site_id: site,
            good_id: output_good,
            unit_id: goods_unit,
            quantity_per_batch: 5,
        }],
        input_coefficients: vec![InputOutputCoefficientV1 {
            process_id: process,
            good_id: input_good,
            unit_id: goods_unit,
            quantity_per_batch: 2,
        }],
        labor_coefficients: vec![LaborCoefficientV1 {
            process_id: process,
            unit_id: labor_unit,
            quantity_per_batch: 3,
        }],
        supplier_candidates: Vec::new(),
        inventory: vec![
            InventoryRowV1 {
                site_id: site,
                good_id: input_good,
                unit_id: goods_unit,
                quantity: input,
            },
            InventoryRowV1 {
                site_id: site,
                good_id: output_good,
                unit_id: goods_unit,
                quantity: 0,
            },
        ],
        orders: Vec::new(),
        backlog: Vec::new(),
        transit: Vec::new(),
        capacities: vec![CapacityRowV1 {
            process_id: process,
            site_id: site,
            period: 1,
            available_batches: capacity,
        }],
        labor: vec![LaborCapacityRowV1 {
            site_id: site,
            unit_id: labor_unit,
            period: 1,
            available: labor,
        }],
        production_commitments: vec![ProductionCommitmentV1 {
            process_id: process,
            site_id: site,
            period: 1,
            planned_batches: 10,
        }],
    }
}

#[test]
fn leontief_minimum_exhaustively_bounds_small_production() {
    for input in 0_u64..=12 {
        for labor in 0_u64..=12 {
            for capacity in 0_u64..=6 {
                let outcome =
                    advance_material_circuit_v1(&production_state(input, labor, capacity))
                        .expect("bounded production must close");
                let expected = 10_u64.min(input / 2).min(labor / 3).min(capacity);
                assert_eq!(outcome.production[0].produced_batches, expected);
                assert_eq!(outcome.state.inventory[0].quantity, input - expected * 2);
                assert_eq!(outcome.state.inventory[1].quantity, expected * 5);
            }
        }
    }
}

#[test]
fn shared_inputs_and_labor_allocate_without_process_order_priority() {
    let mut state = production_state(12, 18, 4);
    let second_process = ProcessIdV1::from_bytes(id::<7>());
    state.process_outputs.push(ProcessOutputV1 {
        process_id: second_process,
        site_id: SiteIdV1::from_bytes(id::<1>()),
        good_id: GoodIdV1::from_bytes(id::<8>()),
        unit_id: UnitIdV1::from_bytes(id::<4>()),
        quantity_per_batch: 5,
    });
    state.input_coefficients.push(InputOutputCoefficientV1 {
        process_id: second_process,
        good_id: GoodIdV1::from_bytes(id::<2>()),
        unit_id: UnitIdV1::from_bytes(id::<4>()),
        quantity_per_batch: 2,
    });
    state.labor_coefficients.push(LaborCoefficientV1 {
        process_id: second_process,
        unit_id: UnitIdV1::from_bytes(id::<5>()),
        quantity_per_batch: 3,
    });
    state.capacities.push(CapacityRowV1 {
        process_id: second_process,
        site_id: SiteIdV1::from_bytes(id::<1>()),
        period: 1,
        available_batches: 4,
    });
    state.production_commitments[0].planned_batches = 4;
    state.production_commitments.push(ProductionCommitmentV1 {
        process_id: second_process,
        site_id: SiteIdV1::from_bytes(id::<1>()),
        period: 1,
        planned_batches: 4,
    });
    let mut reversed = state.clone();
    reversed.production_commitments.reverse();

    let outcome = advance_material_circuit_v1(&state).expect("shared allocation must close");
    let twin = advance_material_circuit_v1(&reversed).expect("permuted allocation must close");
    assert_eq!(outcome.production[0].produced_batches, 3);
    assert_eq!(outcome.production[1].produced_batches, 3);
    assert_eq!(outcome.state.inventory[0].quantity, 0);
    assert_eq!(
        material_circuit_state_v1_digest(&outcome.state),
        material_circuit_state_v1_digest(&twin.state)
    );
}
