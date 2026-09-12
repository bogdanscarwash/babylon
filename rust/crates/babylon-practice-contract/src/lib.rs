//! Pure current practice contracts; no graph mutation or gameplay activation.
mod actor;
mod authority;
mod batch;
mod budget;
mod codec;
mod intent;
mod ordered_action;
mod resource;
#[allow(clippy::unreadable_literal)]
mod schema;
mod strike;
mod topology;

pub use actor::*;
pub use authority::*;
pub use batch::*;
pub use budget::{compute_budget_delta, read_action_budget, write_action_budget};
pub use codec::{budget_delta_digest, decode_budget_delta, encode_budget_delta};
pub use intent::*;
pub use ordered_action::*;
pub use resource::*;
pub use schema::*;
pub use strike::*;
pub use topology::{validate_topology, PracticeTopologyLoadCounter};

/// SHA-256 of the exact language-neutral budget and topology contract.
pub const PRACTICE_BUDGET_TOPOLOGY_SOURCE_SHA256: [u8; 32] = [
    0xae, 0x26, 0xe0, 0xe9, 0x34, 0x5f, 0x26, 0xc2, 0x59, 0x50, 0xf8, 0xf5, 0xe1, 0xf7, 0x46, 0xb5,
    0xad, 0x46, 0xcf, 0x1b, 0x04, 0x0d, 0x58, 0xd2, 0x09, 0xa1, 0xdf, 0xf3, 0x22, 0x7b, 0x64, 0xc1,
];
/// Designed terms for the declared three-practice ActionBudget contract.
pub const DEFAULT_PRACTICE_BUDGET_TERMS: PracticeBudgetTerms = PracticeBudgetTerms {
    initial: 1,
    period_credit_cap: 1,
    storage_ceiling: 4,
    organize_cost: 1,
    agitate_cost: 1,
    mutual_aid_cost: 1,
};
