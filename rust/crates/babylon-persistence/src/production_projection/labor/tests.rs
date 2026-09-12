use babylon_material_circuit::{
    CapacityRow, GoodId, LaborCapacityRow, LaborCoefficient, LogisticsNodeId, ProcessOutput,
    ProductionCommitment, SiteLogisticsNode,
};
use babylon_tick::material_world::{decode_material_receipts, MaterialWorldRegister};

use super::*;

fn shared_opening() -> MaterialCircuitState {
    let site = SiteId::from_bytes([1; 32]);
    let labor_unit = UnitId::from_bytes([2; 32]);
    let mut state = MaterialCircuitState {
        period: 1,
        site_logistics_nodes: vec![SiteLogisticsNode {
            site_id: site,
            node_id: LogisticsNodeId::from_bytes([3; 32]),
        }],
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
    };
    for (id, quantity) in [(4, 2), (5, 3)] {
        let process = ProcessId::from_bytes([id; 32]);
        state.process_outputs.push(ProcessOutput {
            process_id: process,
            site_id: site,
            good_id: GoodId::from_bytes([id; 32]),
            unit_id: UnitId::from_bytes([6; 32]),
            quantity_per_batch: 1,
        });
        state.labor_coefficients.push(LaborCoefficient {
            process_id: process,
            unit_id: labor_unit,
            quantity_per_batch: quantity,
        });
        state.capacities.push(CapacityRow {
            process_id: process,
            site_id: site,
            period: 1,
            available_batches: quantity,
        });
        state.production_commitments.push(ProductionCommitment {
            process_id: process,
            site_id: site,
            period: 1,
            planned_batches: quantity,
        });
    }
    for (period, available) in [(1, 12), (2, 30)] {
        state.labor.push(LaborCapacityRow {
            site_id: site,
            unit_id: labor_unit,
            period,
            available,
        });
    }
    state
}

fn committed_pair(
    state: MaterialCircuitState,
) -> (
    MaterialCircuitState,
    MaterialCircuitState,
    MaterialTickReceipts,
) {
    let opening = MaterialWorldRegister::try_new(0, state).unwrap();
    let next = opening.prepare_next().unwrap();
    (
        opening.state().clone(),
        next.register().state().clone(),
        decode_material_receipts(next.receipt_bytes()).unwrap(),
    )
}

#[test]
fn shared_principal_is_counted_once_and_time_closes_from_actual_receipts() {
    let (opening, next, receipt) = committed_pair(shared_opening());
    let rows = project_labor_accounts(&next, Some(&opening), Some(&receipt)).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].next_opening_period, 2);
    assert_eq!(rows[0].next_opening_available, 30);
    assert_eq!(
        rows[0].completed,
        Some(CompletedProductionLabor {
            period: 1,
            opening: 12,
            planned: 13,
            used: 8,
            unused: 4,
            handling_needed: 0,
            handling_used: 0,
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
    let other_unit = UnitId::from_bytes([7; 32]);
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
    assert!(matches!(
        completed_totals(&opening, &receipt),
        Err(ProductionProjectionError::Arithmetic)
    ));
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
    assert!(matches!(
        completed_totals(&opening, &receipt),
        Err(ProductionProjectionError::Arithmetic)
    ));
}

#[test]
fn inconsistent_accounts_refuse_instead_of_publishing_negative_or_unattributed_time() {
    let (opening, next, receipt) = committed_pair(shared_opening());
    let mut insufficient = opening.clone();
    insufficient.labor[0].available = 7;
    assert_eq!(
        project_labor_accounts(&next, Some(&insufficient), Some(&receipt)),
        Err(ProductionProjectionError::State)
    );
    let mut duplicate = receipt.clone();
    duplicate.production.push(receipt.production[0].clone());
    assert_eq!(
        project_labor_accounts(&next, Some(&opening), Some(&duplicate)),
        Err(ProductionProjectionError::State)
    );
    let mut missing = receipt.clone();
    missing.production.pop();
    assert_eq!(
        project_labor_accounts(&next, Some(&opening), Some(&missing)),
        Err(ProductionProjectionError::State)
    );
    assert_eq!(
        project_labor_accounts(&next, None, Some(&receipt)),
        Err(ProductionProjectionError::History)
    );
    let mut duplicate_budget = opening.clone();
    duplicate_budget.labor.push(opening.labor[0].clone());
    assert_eq!(
        budgets(&duplicate_budget),
        Err(ProductionProjectionError::State)
    );
}
