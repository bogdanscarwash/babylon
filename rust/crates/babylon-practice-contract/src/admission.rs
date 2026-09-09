//! Pure detached authority, quote, and resolve-batch validation.

use crate::{
    PolicyAuthorityPairV1, PracticeAuthorityContextV1, PracticeAuthorityKindV1,
    PracticeContractError, PracticeIdV1, PracticeInputAuthorityV1, PracticeIntentV1,
    PracticeQuoteContextV1, MAX_INTENTS_PER_RESOLVE_TICK, MAX_POLICY_AUTHORITY_PAIRS,
};

fn validate_policy_registry(pairs: &[PolicyAuthorityPairV1]) -> Result<(), PracticeContractError> {
    if pairs.len() > MAX_POLICY_AUTHORITY_PAIRS {
        return Err(PracticeContractError::PracticeAuthorityRegistryLimit);
    }
    let mut previous: Option<&PolicyAuthorityPairV1> = None;
    for pair in pairs.iter().take(MAX_POLICY_AUTHORITY_PAIRS + 1) {
        if previous.is_some_and(|item| {
            item.producer_content_digest == pair.producer_content_digest
                && item.actor_org_id == pair.actor_org_id
        }) {
            return Err(PracticeContractError::PracticeAuthorityRegistryDuplicate);
        }
        if previous.is_some_and(|item| {
            (pair.producer_content_digest, pair.actor_org_id)
                < (item.producer_content_digest, item.actor_org_id)
        }) {
            return Err(PracticeContractError::PracticeAuthorityRegistryOrder);
        }
        previous = Some(pair);
    }
    Ok(())
}

/// Validates one detached authority-intent pair against immutable context.
///
/// # Errors
/// Returns the first exact actor, registry, registration, or content refusal.
pub fn validate_authority_pair(
    authority: &PracticeInputAuthorityV1,
    intent: &PracticeIntentV1,
    context: &PracticeAuthorityContextV1,
) -> Result<(), PracticeContractError> {
    if authority.actor_org_id != intent.actor_org_id {
        return Err(PracticeContractError::PracticeActorMismatch);
    }
    if authority.authority_kind == PracticeAuthorityKindV1::PlayerSeat {
        if authority.actor_org_id != context.player_org_id {
            return Err(PracticeContractError::PracticeActorMismatch);
        }
        if authority.producer_content_digest != context.player_gateway_content_digest {
            return Err(PracticeContractError::PracticeAuthorityContentMismatch);
        }
        return Ok(());
    }
    validate_policy_registry(&context.policy_authorities)?;
    for pair in context
        .policy_authorities
        .iter()
        .take(MAX_POLICY_AUTHORITY_PAIRS + 1)
    {
        if pair.producer_content_digest == authority.producer_content_digest
            && pair.actor_org_id == authority.actor_org_id
        {
            return Ok(());
        }
    }
    Err(PracticeContractError::PracticeAuthorityUnregistered)
}

fn quoted_cost(intent: &PracticeIntentV1, context: &PracticeQuoteContextV1) -> u32 {
    match intent.practice_id {
        PracticeIdV1::Organize => context.budget_terms.organize_cost,
        PracticeIdV1::Agitate => context.budget_terms.agitate_cost,
        PracticeIdV1::MutualAid => context.budget_terms.mutual_aid_cost,
    }
}

/// Validates a detached next-tick quote against immutable content and terms.
///
/// # Errors
/// Returns the first exact tick, content, or cost refusal.
pub fn validate_quote_context(
    intent: &PracticeIntentV1,
    context: &PracticeQuoteContextV1,
) -> Result<(), PracticeContractError> {
    if intent.submit_after_tick != context.last_committed_tick {
        return Err(PracticeContractError::PracticeTickMismatch);
    }
    let expected_resolve_tick = context
        .last_committed_tick
        .checked_add(1)
        .ok_or(PracticeContractError::PracticeTickOverflow)?;
    if intent.resolve_tick != expected_resolve_tick {
        return Err(PracticeContractError::PracticeTickMismatch);
    }
    if intent.quoted_content_digest != context.content_digest {
        return Err(PracticeContractError::PracticeQuoteContentMismatch);
    }
    if intent.quoted_action_budget_cost != quoted_cost(intent, context) {
        return Err(PracticeContractError::PracticeQuoteCostMismatch);
    }
    Ok(())
}

/// Validates one detached bounded resolve batch without mutation.
///
/// # Errors
/// Returns the first exact limit, tick, or duplicate-actor refusal.
pub fn validate_resolve_batch(
    intents: &[PracticeIntentV1],
    expected_resolve_tick: u64,
) -> Result<(), PracticeContractError> {
    if intents.len() > MAX_INTENTS_PER_RESOLVE_TICK {
        return Err(PracticeContractError::PracticeBatchLimit);
    }
    let mut actors = Vec::with_capacity(intents.len());
    for intent in intents.iter().take(MAX_INTENTS_PER_RESOLVE_TICK + 1) {
        if intent.resolve_tick != expected_resolve_tick {
            return Err(PracticeContractError::PracticeTickMismatch);
        }
        if actors
            .iter()
            .take(MAX_INTENTS_PER_RESOLVE_TICK + 1)
            .any(|actor| *actor == intent.actor_org_id)
        {
            return Err(PracticeContractError::PracticeDuplicateActor);
        }
        actors.push(intent.actor_org_id);
    }
    Ok(())
}
