use babylon_material_circuit::{
    CapacityRowV1, GoodIdV1, LaborCapacityRowV1, LaborCoefficientV1, LogisticsNodeIdV2,
    ProcessOutputV1, ProductionCommitmentV1, SiteLogisticsNodeV2,
};
use babylon_tick::material_world::{decode_material_receipts_v3, MaterialWorldRegisterV2};

use super::*;

fn shared_opening() -> MaterialCircuitStateV2 {
    let site = SiteIdV1::from_bytes([1; 32]);
    let labor_unit = UnitIdV1::from_bytes([2; 32]);
    let mut state = MaterialCircuitStateV2 {
        week: 1,
        site_logistics_nodes: vec![SiteLogisticsNodeV2 {
            site_id: site,
            node_id: LogisticsNodeIdV2::from_bytes([3; 32]),
        }],
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
    };
    for (id, quantity) in [(4, 2), (5, 3)] {
        let process = ProcessIdV1::from_bytes([id; 32]);
        state.process_outputs.push(ProcessOutputV1 {
            process_id: process,
            site_id: site,
            good_id: GoodIdV1::from_bytes([id; 32]),
            unit_id: UnitIdV1::from_bytes([6; 32]),
            quantity_per_batch: 1,
        });
        state.labor_coefficients.push(LaborCoefficientV1 {
            process_id: process,
            unit_id: labor_unit,
            quantity_per_batch: quantity,
        });
        state.capacities.push(CapacityRowV1 {
            process_id: process,
            site_id: site,
            week: 1,
            available_batches: quantity,
        });
        state.production_commitments.push(ProductionCommitmentV1 {
            process_id: process,
            site_id: site,
            week: 1,
            planned_batches: quantity,
        });
    }
    for (week, available) in [(1, 12), (2, 30)] {
        state.labor.push(LaborCapacityRowV1 {
            site_id: site,
            unit_id: labor_unit,
            week,
            available,
        });
    }
    state
}

fn committed_pair(
    state: MaterialCircuitStateV2,
) -> (
    MaterialCircuitStateV2,
    MaterialCircuitStateV2,
    MaterialTickReceiptsV3,
) {
    let opening = MaterialWorldRegisterV2::try_new(0, state).unwrap();
    let next = opening.prepare_next().unwrap();
    (
        opening.state().clone(),
        next.register().state().clone(),
        decode_material_receipts_v3(next.receipt_bytes()).unwrap(),
    )
}

#[test]
fn shared_principal_is_counted_once_and_time_closes_from_actual_receipts() {
    let (opening, next, receipt) = committed_pair(shared_opening());
    let rows = project_labor_accounts(&next, Some(&opening), Some(&receipt)).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].next_opening_week, 2);
    assert_eq!(rows[0].next_opening_available, 30);
    assert_eq!(
        rows[0].completed,
        Some(CompletedProductionLaborV1 {
            week: 1,
            opening: 12,
            planned: 13,
            used: 8,
            unused: 4,
        })
    );
    let mut reversed = receipt.clone();
    reversed.production.reverse();
    assert_eq!(
        rows,
        project_labor_accounts(&next, Some(&opening), Some(&reversed)).unwrap()
    );
    assert_eq!(
        opening.labor[0].available, 12,
        "projection never debits authoritative time"
    );
}

#[test]
fn exact_unit_principals_remain_separate_at_the_same_site() {
    let mut state = shared_opening();
    let other_unit = UnitIdV1::from_bytes([7; 32]);
    state.labor_coefficients[1].unit_id = other_unit;
    let mut budget = state.labor[0].clone();
    budget.unit_id = other_unit;
    budget.available = 6;
    state.labor.push(budget);
    let (opening, next, receipt) = committed_pair(state);
    let rows = project_labor_accounts(&next, Some(&opening), Some(&receipt)).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].site_id, rows[1].site_id);
    assert_ne!(rows[0].unit_id, rows[1].unit_id);
    let accounts: Vec<_> = rows
        .iter()
        .map(|row| row.completed.as_ref().unwrap())
        .collect();
    assert_eq!(
        (accounts[0].opening, accounts[0].used, accounts[0].unused),
        (12, 4, 8)
    );
    assert_eq!(
        (accounts[1].opening, accounts[1].used, accounts[1].unused),
        (6, 6, 0)
    );
    assert_eq!(
        rows[1].next_opening_available, 0,
        "no carried unused time or fabricated future budget"
    );
}

#[test]
fn multiplication_and_shared_sum_overflow_refuse_without_mutating_inputs() {
    let (mut opening, _, mut receipt) = committed_pair(shared_opening());
    opening.labor_coefficients[0].quantity_per_batch = u64::MAX;
    let before = opening.clone();
    assert_eq!(
        completed_totals(&opening, &receipt),
        Err(ProductionProjectionErrorV1::Arithmetic)
    );
    assert_eq!(opening, before);
    for coefficient in &mut opening.labor_coefficients {
        coefficient.quantity_per_batch = u64::MAX;
    }
    for plan in &mut opening.production_commitments {
        plan.planned_batches = 1;
    }
    for row in &mut receipt.production {
        row.planned_batches = 1;
        row.produced_batches = 0;
    }
    assert_eq!(
        completed_totals(&opening, &receipt),
        Err(ProductionProjectionErrorV1::Arithmetic)
    );
}

#[test]
fn inconsistent_accounts_refuse_instead_of_publishing_negative_or_unattributed_time() {
    let (opening, next, receipt) = committed_pair(shared_opening());
    let mut insufficient = opening.clone();
    insufficient.labor[0].available = 7;
    assert_eq!(
        project_labor_accounts(&next, Some(&insufficient), Some(&receipt)),
        Err(ProductionProjectionErrorV1::State)
    );
    let mut duplicate = receipt.clone();
    duplicate.production.push(receipt.production[0].clone());
    assert_eq!(
        project_labor_accounts(&next, Some(&opening), Some(&duplicate)),
        Err(ProductionProjectionErrorV1::State)
    );
    let mut missing = receipt.clone();
    missing.production.pop();
    assert_eq!(
        project_labor_accounts(&next, Some(&opening), Some(&missing)),
        Err(ProductionProjectionErrorV1::State)
    );
    assert_eq!(
        project_labor_accounts(&next, None, Some(&receipt)),
        Err(ProductionProjectionErrorV1::History)
    );
    let mut duplicate_budget = opening.clone();
    duplicate_budget.labor.push(opening.labor[0].clone());
    assert_eq!(
        budgets(&duplicate_budget),
        Err(ProductionProjectionErrorV1::State)
    );
}
