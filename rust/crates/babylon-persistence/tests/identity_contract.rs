//! Storage and deterministic replay identities remain distinct.

use babylon_kernel::replay::ReplaySessionId;
use babylon_kernel::tick_content_hash::{RefDigest, TickContentHash};
use babylon_persistence::identity::CampaignId;
use std::any::TypeId;
use uuid::Uuid;

#[test]
fn campaign_storage_identity_is_distinct_from_replay_identity() {
    let first = CampaignId::from_uuid(Uuid::from_u128(1));
    let second = CampaignId::from_uuid(Uuid::from_u128(2));
    assert_ne!(TypeId::of::<CampaignId>(), TypeId::of::<ReplaySessionId>());
    assert_ne!(first, second);
    assert_eq!(first.as_uuid(), &Uuid::from_u128(1));
    assert_eq!(first.canonical_bytes(), Uuid::from_u128(1).as_bytes());
}

#[test]
fn reference_and_tick_content_hashes_are_nominally_distinct() {
    assert_ne!(TypeId::of::<TickContentHash>(), TypeId::of::<RefDigest>());
    let bytes = [0x07; 32];
    assert_eq!(RefDigest::from_bytes(bytes).as_bytes(), &bytes);
    assert_eq!(RefDigest::from_bytes(bytes).to_hex(), "07".repeat(32));
}
