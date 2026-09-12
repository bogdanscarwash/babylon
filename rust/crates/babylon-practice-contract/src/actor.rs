//! Opaque eight-byte organization identity for frozen Practice current contracts.

/// Stable organization identity carried as exact canonical bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ActorOrganizationId([u8; 8]);

impl ActorOrganizationId {
    /// Construct from the exact canonical identity bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 8]) -> Self {
        Self(bytes)
    }

    /// Return the exact canonical identity bytes.
    #[must_use]
    pub const fn to_bytes(self) -> [u8; 8] {
        self.0
    }
}
