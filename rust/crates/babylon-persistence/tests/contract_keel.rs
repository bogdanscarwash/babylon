//! Public contracts that prevent persistence identity and hash-name collapse.

use babylon_kernel::tick_content_hash::{RefDigestV1, TickContentHashV1};
use babylon_kernel::{seed_for, ContentDigest, SessionId};
use babylon_persistence::{
    CampaignId, GraphStateHash, MigrationSetDigest, PersistenceError, PersistenceFailureKind,
    ReplayIdentityHash,
};
use std::any::TypeId;
use uuid::Uuid;

#[test]
fn persistence_exposes_no_short_identity_aliases() {
    let hashes = include_str!("../src/hashes.rs");
    let exports = include_str!("../src/lib.rs");

    assert!(!hashes.contains(" as RefDigest"));
    assert!(!hashes.contains(" as TickContentHash"));
    assert!(!exports.contains("RefDigest,"));
    assert!(!exports.contains("TickContentHash,"));
}

#[test]
fn campaign_uuid_is_a_storage_wrapper_and_rng_requires_a_session() {
    let first = CampaignId::from_uuid(Uuid::from_u128(1));
    let second = CampaignId::from_uuid(Uuid::from_u128(2));
    let _: fn(&SessionId, u64, &str, &str) -> [u8; 32] = seed_for;
    assert_ne!(first, second);
    assert_eq!(first.as_uuid(), &Uuid::from_u128(1));
}

#[test]
fn honest_hashes_are_nominally_distinct() {
    assert_ne!(
        TypeId::of::<ReplayIdentityHash>(),
        TypeId::of::<GraphStateHash>()
    );
    assert_ne!(
        TypeId::of::<ReplayIdentityHash>(),
        TypeId::of::<TickContentHashV1>()
    );
    assert_ne!(
        TypeId::of::<ReplayIdentityHash>(),
        TypeId::of::<RefDigestV1>()
    );
    assert_ne!(
        TypeId::of::<ReplayIdentityHash>(),
        TypeId::of::<MigrationSetDigest>()
    );
    assert_ne!(
        TypeId::of::<GraphStateHash>(),
        TypeId::of::<TickContentHashV1>()
    );
    assert_ne!(TypeId::of::<GraphStateHash>(), TypeId::of::<RefDigestV1>());
    assert_ne!(
        TypeId::of::<GraphStateHash>(),
        TypeId::of::<MigrationSetDigest>()
    );
    assert_ne!(
        TypeId::of::<TickContentHashV1>(),
        TypeId::of::<RefDigestV1>()
    );
    assert_ne!(
        TypeId::of::<TickContentHashV1>(),
        TypeId::of::<MigrationSetDigest>()
    );
    assert_ne!(
        TypeId::of::<RefDigestV1>(),
        TypeId::of::<MigrationSetDigest>()
    );
    let bytes = [0x07; 32];
    assert_eq!(GraphStateHash::from_bytes(bytes).as_bytes(), &bytes);
    assert_eq!(RefDigestV1::from_bytes(bytes).to_hex(), "07".repeat(32));
}

#[test]
fn content_digest_remains_the_kernel_pair() {
    let digest = ContentDigest {
        defines_hash: [1; 32],
        rules_hash: [2; 32],
    };
    assert_ne!(digest.defines_hash, digest.rules_hash);
}

#[test]
fn failures_keep_five_distinct_stages() {
    let cases = [
        (
            PersistenceError::connection("connect"),
            PersistenceFailureKind::Connection,
        ),
        (
            PersistenceError::migration("adopt"),
            PersistenceFailureKind::Migration,
        ),
        (
            PersistenceError::serialization("encode"),
            PersistenceFailureKind::Serialization,
        ),
        (
            PersistenceError::constraint("foreign key"),
            PersistenceFailureKind::Constraint,
        ),
        (
            PersistenceError::commit("commit"),
            PersistenceFailureKind::Commit,
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(error.kind(), expected);
        assert!(!error.to_string().is_empty());
    }
}
