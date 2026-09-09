use babylon_practice_contract::{
    validate_topology, OrganizationPracticeTopologyEdgeV1, OrganizationPracticeTopologyRowV1,
    OrganizationPracticeTopologyV1, PracticeContractError, PracticeTargetDomainV1,
    PracticeTopologyLoadCounter,
};

fn edge(target: u64) -> OrganizationPracticeTopologyEdgeV1 {
    OrganizationPracticeTopologyEdgeV1 {
        target_domain: PracticeTargetDomainV1::SocialClass,
        target_class_node_id_u64: target,
    }
}

fn row(
    node_id: u64,
    active: bool,
    storage: Option<f64>,
    edges: Vec<OrganizationPracticeTopologyEdgeV1>,
) -> OrganizationPracticeTopologyRowV1 {
    OrganizationPracticeTopologyRowV1 {
        node_id_u64: node_id,
        active_bool: active,
        action_budget_storage_f64_bits_u64: storage.map(f64::to_bits),
        edges,
    }
}

fn topology(rows: Vec<OrganizationPracticeTopologyRowV1>) -> OrganizationPracticeTopologyV1 {
    OrganizationPracticeTopologyV1 {
        organizations: rows,
    }
}

#[test]
fn topology_accepts_zero_and_exact_organization_maximum() {
    assert_eq!(validate_topology(&topology(vec![])), Ok(()));
    let rows = (0_u64..4_096)
        .map(|node_id| row(node_id, false, None, vec![]))
        .collect();
    assert_eq!(validate_topology(&topology(rows)), Ok(()));
}

#[test]
fn topology_organization_bound_order_and_duplicate_codes_are_exact() {
    let too_many = (0_u64..4_097)
        .map(|node_id| row(node_id, false, None, vec![]))
        .collect();
    for (rows, expected) in [
        (
            too_many,
            PracticeContractError::PracticeTopologyOrganizationLimit,
        ),
        (
            vec![row(2, false, None, vec![]), row(1, false, None, vec![])],
            PracticeContractError::PracticeTopologyOrganizationOrder,
        ),
        (
            vec![row(1, false, None, vec![]), row(1, false, None, vec![])],
            PracticeContractError::PracticeTopologyOrganizationDuplicate,
        ),
    ] {
        assert_eq!(validate_topology(&topology(rows)), Err(expected));
    }
}

#[test]
fn active_budget_presence_and_even_inactive_storage_are_validated() {
    assert_eq!(
        validate_topology(&topology(vec![row(1, true, None, vec![])])),
        Err(PracticeContractError::PracticeTopologyBudgetMissing)
    );
    assert_eq!(
        validate_topology(&topology(vec![row(1, false, Some(-1.0), vec![])])),
        Err(PracticeContractError::PracticeBudgetNegative)
    );
    assert_eq!(
        validate_topology(&topology(vec![row(1, false, None, vec![])])),
        Ok(())
    );
    assert_eq!(
        validate_topology(&topology(vec![row(1, true, Some(1.0), vec![])])),
        Ok(())
    );
}

#[test]
fn topology_edge_bound_order_and_duplicate_codes_are_exact() {
    let maximum = (0_u64..256).map(edge).collect();
    assert_eq!(
        validate_topology(&topology(vec![row(1, false, None, maximum)])),
        Ok(())
    );
    let too_many = (0_u64..257).map(edge).collect();
    for (edges, expected) in [
        (too_many, PracticeContractError::PracticeFootprintLimit),
        (
            vec![edge(2), edge(1)],
            PracticeContractError::PracticeTopologyEdgeOrder,
        ),
        (
            vec![edge(1), edge(1)],
            PracticeContractError::PracticeTopologyEdgeDuplicate,
        ),
    ] {
        assert_eq!(
            validate_topology(&topology(vec![row(1, false, None, edges)])),
            Err(expected)
        );
    }
}

#[test]
fn load_counter_is_detached_and_uses_only_validation_local_keys() {
    let mut counter = PracticeTopologyLoadCounter::new();
    counter.observe_organization(10, true, Some(1.0)).unwrap();
    counter
        .observe_solidarity_edge(10, PracticeTargetDomainV1::SocialClass, 20)
        .unwrap();
    assert_eq!(counter.finish(), Ok(()));
}
