//! Database-free content-identity claim for one future `tick_commit` marker.

use babylon_kernel::tick_content_hash::TickContentHash;

use crate::identity::CampaignId;

/// Canonical `TickCommitClaimV1` layout version.
pub const TICK_COMMIT_CLAIM_LAYOUT_VERSION: u32 = 1;
/// Exact byte length of every canonical V1 claim.
pub const TICK_COMMIT_CLAIM_BYTES: usize = 93;

const TICK_CONTENT_HASH_LAYOUT_VERSION: u32 = 1;
const DOMAIN: &[u8; 26] = b"babylon.tick-commit-claim\0";

/// One typed future-marker key bound to its authoritative tick-content identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickCommitClaim {
    campaign_id: CampaignId,
    resolve_tick: u64,
    tick_content_hash: TickContentHash,
    canonical_bytes: [u8; TICK_COMMIT_CLAIM_BYTES],
}

/// Successful classification of a retry against an existing claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickCommitClaimRetry {
    /// The requested key and content identity equal the existing claim.
    Idempotent,
}

/// Closed conflict classes for a requested claim compared with an existing one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickCommitClaimConflict {
    /// The comparison crossed campaign or tick keys.
    KeyMismatch {
        /// Campaign in the existing claim.
        existing_campaign_id: CampaignId,
        /// Tick in the existing claim.
        existing_resolve_tick: u64,
        /// Campaign in the requested claim.
        requested_campaign_id: CampaignId,
        /// Tick in the requested claim.
        requested_resolve_tick: u64,
    },
    /// One campaign tick already has a different authoritative content identity.
    ContentIdentityMismatch {
        /// Shared durable campaign key.
        campaign_id: CampaignId,
        /// Shared resolve tick.
        resolve_tick: u64,
        /// Content identity already claimed.
        existing: TickContentHash,
        /// Different content identity requested by the retry.
        requested: TickContentHash,
    },
}

impl TickCommitClaim {
    /// Compose the exact fixed V1 bytes from the three owning typed values.
    #[must_use]
    pub fn compose(
        campaign_id: CampaignId,
        resolve_tick: u64,
        tick_content_hash: TickContentHash,
    ) -> Self {
        let mut canonical_bytes = [0_u8; TICK_COMMIT_CLAIM_BYTES];
        canonical_bytes[0..26].copy_from_slice(DOMAIN);
        canonical_bytes[26..30].copy_from_slice(&TICK_COMMIT_CLAIM_LAYOUT_VERSION.to_be_bytes());
        canonical_bytes[30] = 0x01;
        canonical_bytes[31..47].copy_from_slice(campaign_id.canonical_bytes());
        canonical_bytes[47] = 0x02;
        canonical_bytes[48..56].copy_from_slice(&resolve_tick.to_be_bytes());
        canonical_bytes[56] = 0x03;
        canonical_bytes[57..61].copy_from_slice(&TICK_CONTENT_HASH_LAYOUT_VERSION.to_be_bytes());
        canonical_bytes[61..93].copy_from_slice(tick_content_hash.as_bytes());
        Self {
            campaign_id,
            resolve_tick,
            tick_content_hash,
            canonical_bytes,
        }
    }

    /// Return the durable campaign key.
    #[must_use]
    pub const fn campaign_id(&self) -> CampaignId {
        self.campaign_id
    }

    /// Return the unsigned resolve tick.
    #[must_use]
    pub const fn resolve_tick(&self) -> u64 {
        self.resolve_tick
    }

    /// Return the direct kernel-owned tick-content identity.
    #[must_use]
    pub const fn tick_content_hash(&self) -> TickContentHash {
        self.tick_content_hash
    }

    /// Borrow the exact 93 canonical bytes.
    #[must_use]
    pub const fn canonical_bytes(&self) -> &[u8; TICK_COMMIT_CLAIM_BYTES] {
        &self.canonical_bytes
    }

    /// Classify this requested claim against one existing claim.
    ///
    /// # Errors
    /// Returns a typed key or content-identity conflict. This method does not
    /// claim whole-envelope payload equality.
    pub fn classify_retry_against(
        &self,
        existing: &Self,
    ) -> Result<TickCommitClaimRetry, TickCommitClaimConflict> {
        if self.campaign_id != existing.campaign_id || self.resolve_tick != existing.resolve_tick {
            return Err(TickCommitClaimConflict::KeyMismatch {
                existing_campaign_id: existing.campaign_id,
                existing_resolve_tick: existing.resolve_tick,
                requested_campaign_id: self.campaign_id,
                requested_resolve_tick: self.resolve_tick,
            });
        }
        if self.tick_content_hash != existing.tick_content_hash {
            return Err(TickCommitClaimConflict::ContentIdentityMismatch {
                campaign_id: self.campaign_id,
                resolve_tick: self.resolve_tick,
                existing: existing.tick_content_hash,
                requested: self.tick_content_hash,
            });
        }
        Ok(TickCommitClaimRetry::Idempotent)
    }
}
