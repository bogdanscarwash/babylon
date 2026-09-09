//! Private-field digest wrappers for canonical evidence records.

use babylon_kernel::sha256_of;

use crate::wire::{canonical_envelope, SfsWireError, T3Record};

/// One opaque SHA-256 value supplied to or decoded from a T3 record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Digest32([u8; 32]);

impl Digest32 {
    /// Wraps one exact 32-byte digest without assigning record-digest authority.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Reports whether every byte is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == [0; 32]
    }

    /// Returns the lowercase 64-character hexadecimal representation.
    #[must_use]
    #[allow(clippy::needless_range_loop)] // The literal bound is part of the digest contract.
    pub fn to_hex(&self) -> String {
        use std::fmt::Write as _;

        let mut output = String::with_capacity(64);
        for index in 0..32 {
            let byte = self.0[index];
            write!(output, "{byte:02x}").expect("writing to String cannot fail");
        }
        output
    }
}

/// SHA-256 of one complete canonical T3 record envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordDigest(Digest32);

impl RecordDigest {
    /// Returns the exact record-digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    /// Returns the lowercase 64-character hexadecimal representation.
    #[must_use]
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }
}

/// Hashes one complete canonical record envelope.
///
/// # Errors
/// Returns the exact domain, payload, string, or numeric refusal produced while
/// constructing the canonical envelope.
pub fn record_digest<T: T3Record>(record: &T) -> Result<RecordDigest, SfsWireError> {
    let envelope = canonical_envelope(record)?;
    Ok(RecordDigest(Digest32::from_bytes(sha256_of(&envelope))))
}
