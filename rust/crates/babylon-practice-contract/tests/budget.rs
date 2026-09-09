use babylon_practice_contract::{
    compute_budget_delta, read_action_budget, write_action_budget, PracticeBudgetTermsV1,
    PracticeContractError, PracticeIdV1, PracticeTargetDomainV1, SolidarityFootprintEdgeV1,
};

const WORLD_HASH: [u8; 32] = [0x5a; 32];

fn terms() -> PracticeBudgetTermsV1 {
    PracticeBudgetTermsV1 {
        initial: 1,
        weekly_credit_cap: 1,
        storage_ceiling: 4,
        organize_cost: 1,
        agitate_cost: 2,
        mutual_aid_cost: 3,
    }
}

fn edge(source: u64, target: u64, strength: f64) -> SolidarityFootprintEdgeV1 {
    SolidarityFootprintEdgeV1 {
        source_org_node_id_u64: source,
        target_domain_u8: PracticeTargetDomainV1::SocialClass,
        target_class_node_id_u64: target,
        strength_f64_bits_u64: strength.to_bits(),
    }
}

#[test]
fn storage_conversion_round_trips_zero_and_u32_max() {
    for value in [0, 1, u32::MAX] {
        assert_eq!(read_action_budget(write_action_budget(value)), Ok(value));
    }
}

#[test]
fn storage_conversion_refusals_pin_codes_28_through_32() {
    for (storage, expected) in [
        (f64::NAN, PracticeContractError::PracticeBudgetNonfinite),
        (
            f64::INFINITY,
            PracticeContractError::PracticeBudgetNonfinite,
        ),
        (-1.0, PracticeContractError::PracticeBudgetNegative),
        (1.5, PracticeContractError::PracticeBudgetFractional),
        (
            f64::from(u32::MAX) + 1.0,
            PracticeContractError::PracticeBudgetRange,
        ),
        (-0.0, PracticeContractError::PracticeBudgetRoundtrip),
    ] {
        assert_eq!(read_action_budget(storage), Err(expected));
        assert_eq!(u16::from(expected), expected as u16);
    }
}

#[test]
fn transition_derives_cost_count_credit_and_snapshot() {
    let delta = compute_budget_delta(
        11,
        7,
        WORLD_HASH,
        2,
        Some(PracticeIdV1::Organize),
        &[edge(7, 101, 1.0), edge(7, 102, 1.0)],
        terms(),
    )
    .unwrap();
    assert_eq!(delta.schema_version, 1);
    assert_eq!(delta.tick, 11);
    assert_eq!(delta.actor_node_id, 7);
    assert_eq!(delta.pre_action_world_hash, WORLD_HASH);
    assert_eq!(delta.budget_before, 2);
    assert_eq!(delta.governed_cost, 1);
    assert_eq!(delta.footprint_count, 2);
    assert_eq!(delta.raw_credit, 2);
    assert_eq!(delta.credited_credit, 1);
    assert!(!delta.ceiling_bound);
    assert_eq!(delta.budget_after, 2);
}

#[test]
fn every_practice_cost_and_none_zero_cost_are_exhaustive() {
    for (practice, expected) in [
        (None, 0),
        (Some(PracticeIdV1::Organize), 1),
        (Some(PracticeIdV1::Agitate), 2),
        (Some(PracticeIdV1::MutualAid), 3),
    ] {
        let delta = compute_budget_delta(11, 7, WORLD_HASH, 3, practice, &[], terms()).unwrap();
        assert_eq!(delta.governed_cost, expected);
        assert_eq!(delta.budget_after, 3 - expected);
    }
}

#[test]
fn insufficient_and_checked_addition_refusals_pin_codes_33_and_34() {
    assert_eq!(
        compute_budget_delta(
            11,
            7,
            WORLD_HASH,
            0,
            Some(PracticeIdV1::Organize),
            &[],
            terms(),
        ),
        Err(PracticeContractError::PracticeBudgetInsufficient)
    );
    let overflow_terms = PracticeBudgetTermsV1 {
        storage_ceiling: u32::MAX,
        ..terms()
    };
    assert_eq!(
        compute_budget_delta(
            11,
            7,
            WORLD_HASH,
            u32::MAX,
            None,
            &[edge(7, 101, 1.0)],
            overflow_terms,
        ),
        Err(PracticeContractError::PracticeBudgetArithmetic)
    );
}

#[test]
fn subtraction_keeps_precedence_guard_and_uses_checked_contract() {
    let source = include_str!("../src/budget.rs");
    let guard = source
        .find("if budget_before < cost")
        .expect("explicit code-33 precedence guard");
    let checked = source
        .find(".checked_sub(cost)")
        .expect("checked subtraction");
    let mapping = checked
        + source[checked..]
            .find("PracticeContractError::PracticeBudgetInsufficient")
            .expect("subtraction maps to the code-33 identity");
    assert!(guard < checked);
    assert!(checked < mapping);
}

#[test]
fn ceiling_binds_only_after_valid_checked_addition() {
    let capped_terms = PracticeBudgetTermsV1 {
        storage_ceiling: 3,
        ..terms()
    };
    let delta = compute_budget_delta(
        11,
        7,
        WORLD_HASH,
        3,
        None,
        &[edge(7, 101, 1.0)],
        capped_terms,
    )
    .unwrap();
    assert!(delta.ceiling_bound);
    assert_eq!(delta.budget_after, 3);
}

#[test]
fn footprint_bound_order_duplicate_and_source_refusals_are_exact() {
    let too_many: Vec<_> = (0_u64..257).map(|target| edge(7, target, 1.0)).collect();
    for (edges, expected) in [
        (too_many, PracticeContractError::PracticeFootprintLimit),
        (
            vec![edge(7, 2, 1.0), edge(7, 1, 1.0)],
            PracticeContractError::PracticeFootprintOrder,
        ),
        (
            vec![edge(7, 1, 1.0), edge(7, 1, 1.0)],
            PracticeContractError::PracticeFootprintDuplicate,
        ),
        (
            vec![edge(8, 1, 1.0)],
            PracticeContractError::PracticeFootprintSource,
        ),
    ] {
        assert_eq!(
            compute_budget_delta(11, 7, WORLD_HASH, 3, None, &edges, terms()),
            Err(expected)
        );
    }
}

#[test]
fn footprint_strength_refusals_and_maximum_are_exact() {
    for (strength, expected) in [
        (
            f64::NAN,
            PracticeContractError::PracticeFootprintStrengthNonfinite,
        ),
        (
            f64::INFINITY,
            PracticeContractError::PracticeFootprintStrengthNonfinite,
        ),
        (
            0.0,
            PracticeContractError::PracticeFootprintStrengthNonpositive,
        ),
        (
            -1.0,
            PracticeContractError::PracticeFootprintStrengthNonpositive,
        ),
    ] {
        assert_eq!(
            compute_budget_delta(11, 7, WORLD_HASH, 3, None, &[edge(7, 1, strength)], terms(),),
            Err(expected)
        );
    }
    let maximum: Vec<_> = (0_u64..256).map(|target| edge(7, target, 1.0)).collect();
    let delta = compute_budget_delta(11, 7, WORLD_HASH, 0, None, &maximum, terms()).unwrap();
    assert_eq!(delta.footprint_count, 256);
    assert_eq!(delta.raw_credit, 256);
}
