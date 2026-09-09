use babylon_practice_contract::{
    validate_authority_pair, validate_quote_context, validate_resolve_batch, PolicyAuthorityPairV1,
    PracticeAuthorityContextV1, PracticeAuthorityKindV1, PracticeBudgetTermsV1,
    PracticeContractError, PracticeIdV1, PracticeInputAuthorityV1, PracticeIntentV1,
    PracticeQuoteContextV1, PracticeTargetDomainV1,
};

fn intent() -> PracticeIntentV1 {
    PracticeIntentV1 {
        schema_version: 1,
        submit_after_tick: 10,
        resolve_tick: 11,
        actor_org_id: 7,
        practice_id: PracticeIdV1::Organize,
        target_domain: PracticeTargetDomainV1::SocialClass,
        target_node_id: 101,
        quoted_content_digest: [0x22; 32],
        quoted_action_budget_cost: 1,
        parameters: Vec::new(),
        evidence_digests: Vec::new(),
    }
}

#[test]
fn player_quote_and_empty_batch_validate_without_side_effects() {
    let authority = PracticeInputAuthorityV1 {
        schema_version: 1,
        authority_kind: PracticeAuthorityKindV1::PlayerSeat,
        actor_org_id: 7,
        producer_content_digest: [0x11; 32],
    };
    let context = PracticeAuthorityContextV1 {
        player_org_id: 7,
        player_gateway_content_digest: [0x11; 32],
        policy_authorities: vec![PolicyAuthorityPairV1 {
            producer_content_digest: [0x33; 32],
            actor_org_id: 8,
        }],
    };
    let intent = intent();
    validate_authority_pair(&authority, &intent, &context).unwrap();
    let quote = PracticeQuoteContextV1 {
        last_committed_tick: 10,
        content_digest: [0x22; 32],
        budget_terms: PracticeBudgetTermsV1 {
            initial: 1,
            weekly_credit_cap: 1,
            storage_ceiling: 4,
            organize_cost: 1,
            agitate_cost: 1,
            mutual_aid_cost: 1,
        },
    };
    validate_quote_context(&intent, &quote).unwrap();
    validate_resolve_batch(&[], 11).unwrap();
}

fn authority(
    kind: PracticeAuthorityKindV1,
    actor_org_id: u64,
    producer_content_digest: [u8; 32],
) -> PracticeInputAuthorityV1 {
    PracticeInputAuthorityV1 {
        schema_version: 1,
        authority_kind: kind,
        actor_org_id,
        producer_content_digest,
    }
}

fn context(pairs: Vec<PolicyAuthorityPairV1>) -> PracticeAuthorityContextV1 {
    PracticeAuthorityContextV1 {
        player_org_id: 7,
        player_gateway_content_digest: [0x11; 32],
        policy_authorities: pairs,
    }
}

fn quote() -> PracticeQuoteContextV1 {
    PracticeQuoteContextV1 {
        last_committed_tick: 10,
        content_digest: [0x22; 32],
        budget_terms: PracticeBudgetTermsV1 {
            initial: 1,
            weekly_credit_cap: 1,
            storage_ceiling: 4,
            organize_cost: 1,
            agitate_cost: 2,
            mutual_aid_cost: 3,
        },
    }
}

#[test]
fn player_and_policy_authority_pin_every_refusal() {
    let player = authority(PracticeAuthorityKindV1::PlayerSeat, 7, [0x11; 32]);
    assert_eq!(
        validate_authority_pair(&player, &intent(), &context(Vec::new())),
        Ok(())
    );
    let wrong_actor = authority(PracticeAuthorityKindV1::PlayerSeat, 8, [0x11; 32]);
    assert_eq!(
        validate_authority_pair(&wrong_actor, &intent(), &context(Vec::new())),
        Err(PracticeContractError::PracticeActorMismatch)
    );
    let wrong_digest = authority(PracticeAuthorityKindV1::PlayerSeat, 7, [0x12; 32]);
    assert_eq!(
        validate_authority_pair(&wrong_digest, &intent(), &context(Vec::new())),
        Err(PracticeContractError::PracticeAuthorityContentMismatch)
    );
    let pair = PolicyAuthorityPairV1 {
        producer_content_digest: [0x33; 32],
        actor_org_id: 8,
    };
    let policy = authority(PracticeAuthorityKindV1::DeterministicPolicy, 8, [0x33; 32]);
    let mut policy_intent = intent();
    policy_intent.actor_org_id = 8;
    assert_eq!(
        validate_authority_pair(&policy, &policy_intent, &context(vec![pair.clone()])),
        Ok(())
    );
    assert_eq!(
        validate_authority_pair(&policy, &policy_intent, &context(Vec::new())),
        Err(PracticeContractError::PracticeAuthorityUnregistered)
    );
    assert_eq!(
        validate_authority_pair(
            &policy,
            &policy_intent,
            &context(vec![pair.clone(), pair.clone()])
        ),
        Err(PracticeContractError::PracticeAuthorityRegistryDuplicate)
    );
    let low = PolicyAuthorityPairV1 {
        producer_content_digest: [0x22; 32],
        actor_org_id: 8,
    };
    assert_eq!(
        validate_authority_pair(&policy, &policy_intent, &context(vec![pair.clone(), low])),
        Err(PracticeContractError::PracticeAuthorityRegistryOrder)
    );
    assert_eq!(
        validate_authority_pair(&policy, &intent(), &context(vec![pair.clone()])),
        Err(PracticeContractError::PracticeActorMismatch)
    );
    assert_eq!(
        validate_authority_pair(&policy, &policy_intent, &context(vec![pair; 4_097])),
        Err(PracticeContractError::PracticeAuthorityRegistryLimit)
    );
}

#[test]
fn quote_validation_is_exhaustive_and_checked() {
    for (practice, cost) in [
        (PracticeIdV1::Organize, 1),
        (PracticeIdV1::Agitate, 2),
        (PracticeIdV1::MutualAid, 3),
    ] {
        let mut value = intent();
        value.practice_id = practice;
        value.quoted_action_budget_cost = cost;
        assert_eq!(validate_quote_context(&value, &quote()), Ok(()));
        value.quoted_action_budget_cost = cost + 1;
        assert_eq!(
            validate_quote_context(&value, &quote()),
            Err(PracticeContractError::PracticeQuoteCostMismatch)
        );
    }
    let mut stale_tick = intent();
    stale_tick.submit_after_tick = 9;
    assert_eq!(
        validate_quote_context(&stale_tick, &quote()),
        Err(PracticeContractError::PracticeTickMismatch)
    );
    let mut wrong_resolve = intent();
    wrong_resolve.resolve_tick = 12;
    assert_eq!(
        validate_quote_context(&wrong_resolve, &quote()),
        Err(PracticeContractError::PracticeTickMismatch)
    );
    let mut overflow = intent();
    overflow.submit_after_tick = u64::MAX;
    overflow.resolve_tick = 0;
    let mut overflow_quote = quote();
    overflow_quote.last_committed_tick = u64::MAX;
    assert_eq!(
        validate_quote_context(&overflow, &overflow_quote),
        Err(PracticeContractError::PracticeTickOverflow)
    );
    let mut stale_content = quote();
    stale_content.content_digest = [0x23; 32];
    assert_eq!(
        validate_quote_context(&intent(), &stale_content),
        Err(PracticeContractError::PracticeQuoteContentMismatch)
    );
}

#[test]
fn batch_zero_max_plus_one_tick_and_duplicate_are_exact() {
    assert_eq!(validate_resolve_batch(&[], 11), Ok(()));
    let mut intents = Vec::with_capacity(4_097);
    for actor in 0..4_096_u64 {
        let mut value = intent();
        value.actor_org_id = actor;
        intents.push(value);
    }
    assert_eq!(validate_resolve_batch(&intents, 11), Ok(()));
    let mut extra = intent();
    extra.actor_org_id = 4_096;
    intents.push(extra);
    assert_eq!(
        validate_resolve_batch(&intents, 11),
        Err(PracticeContractError::PracticeBatchLimit)
    );
    intents.pop();
    intents[4_095].actor_org_id = 0;
    assert_eq!(
        validate_resolve_batch(&intents, 11),
        Err(PracticeContractError::PracticeDuplicateActor)
    );
    let mut mismatched = intent();
    mismatched.resolve_tick = 12;
    assert_eq!(
        validate_resolve_batch(&[intent(), mismatched], 11),
        Err(PracticeContractError::PracticeTickMismatch)
    );
}
