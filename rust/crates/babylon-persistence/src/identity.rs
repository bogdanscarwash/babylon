//! Durable database identities that never enter deterministic engine physics.

use uuid::Uuid;

/// Durable `PostgreSQL` campaign key, distinct from the deterministic session namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CampaignId(Uuid);

impl CampaignId {
    /// Wrap an already-minted UUID as a campaign storage identity.
    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Return the UUID used by `PostgreSQL` foreign keys and partitions.
    #[must_use]
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Borrow the exact 16 UUID bytes in network order.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }
}
