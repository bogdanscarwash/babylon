#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

#[allow(clippy::unreadable_literal)]
mod schema;

pub use schema::*;

pub mod actor_v2;
pub mod admission;
pub mod authority_v2;
pub mod batch_v2;
pub mod budget;
pub mod codec;
pub mod intent_v2;
pub mod ordered_action_v1;
pub mod resource_v2;
pub mod strike_v2;
pub mod topology;

pub use admission::{validate_authority_pair, validate_quote_context, validate_resolve_batch};
pub use authority_v2::*;
pub use batch_v2::*;
pub use budget::{compute_budget_delta, read_action_budget, write_action_budget};
pub use codec::{
    budget_delta_digest, decode_budget_delta, decode_input_authority, decode_intent,
    decode_rejection, encode_budget_delta, encode_input_authority, encode_intent,
    encode_intent_parameters, encode_rejection, input_authority_digest, intent_digest,
    parameter_bytes_digest, rejection_for, submission_rejection_alias,
    target_selection_policy_digest,
};
pub use intent_v2::*;
pub use resource_v2::*;
pub use strike_v2::*;
pub use topology::{validate_topology, PracticeTopologyLoadCounter};

/// SHA-256 of the exact language-neutral V1 practice schema bytes.
pub const PRACTICE_CONTRACT_SOURCE_SHA256: [u8; 32] = [
    0xe9, 0xed, 0x6d, 0xba, 0xf0, 0x1f, 0x89, 0xf1, 0x29, 0x4f, 0x2e, 0x6d, 0x28, 0x94, 0x6e, 0x73,
    0xb0, 0x5d, 0x9a, 0x4d, 0x75, 0x47, 0x2d, 0x5b, 0x2d, 0xd3, 0x52, 0x35, 0x0d, 0x33, 0x2f, 0x79,
];

/// Designed V1 practice-budget terms declared by the language-neutral contract.
pub const DEFAULT_PRACTICE_BUDGET_TERMS_V1: PracticeBudgetTermsV1 = PracticeBudgetTermsV1 {
    initial: 1,
    weekly_credit_cap: 1,
    storage_ceiling: 4,
    organize_cost: 1,
    agitate_cost: 1,
    mutual_aid_cost: 1,
};

const SHARED_ACTIVATION_BLOCKERS: &[PracticeActivationBlockerV1] = &[
    PracticeActivationBlockerV1::Gate3CommittedEnvelope,
    PracticeActivationBlockerV1::Gate5PendingInput,
];
const MUTUAL_AID_ACTIVATION_BLOCKERS: &[PracticeActivationBlockerV1] = &[
    PracticeActivationBlockerV1::Gate3CommittedEnvelope,
    PracticeActivationBlockerV1::Gate5PendingInput,
    PracticeActivationBlockerV1::Per30OrdersInventory,
    PracticeActivationBlockerV1::Per31FreightRealization,
];

/// Return the stable non-live refusal for one closed practice.
#[must_use]
pub const fn unwired_reason(practice: PracticeIdV1) -> PracticeRejectionCodeV1 {
    match practice {
        PracticeIdV1::Organize | PracticeIdV1::Agitate | PracticeIdV1::MutualAid => {
            PracticeRejectionCodeV1::PracticeUnwired
        }
    }
}

/// Return immutable non-admission dependency metadata.
#[must_use]
pub const fn activation_blockers(practice: PracticeIdV1) -> &'static [PracticeActivationBlockerV1] {
    match practice {
        PracticeIdV1::Organize | PracticeIdV1::Agitate => SHARED_ACTIVATION_BLOCKERS,
        PracticeIdV1::MutualAid => MUTUAL_AID_ACTIVATION_BLOCKERS,
    }
}
