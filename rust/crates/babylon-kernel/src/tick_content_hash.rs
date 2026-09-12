//! Nominal replay digests and the fixed `TickContentHashV1` outer codec.

use crate::replay::{ReplaySeed, ReplaySessionId};
use crate::{content_digest::sha256_of, content_digest::ContentDigest};
use std::collections::TryReserveError;

const TICK_CONTENT_DOMAIN: &[u8] = b"babylon.tick-content\0";
const TICK_CONTENT_FIXED_BYTES: usize = 349;
const LAYOUT: u32 = 1;
const RNG_LAYOUT: u32 = 2;

macro_rules! digest_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name([u8; 32]);

        impl $name {
            /// Wrap one already-computed SHA-256 value.
            #[must_use]
            pub const fn from_bytes(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }

            /// Borrow the exact 32 digest bytes.
            #[must_use]
            pub const fn as_bytes(&self) -> &[u8; 32] {
                &self.0
            }

            /// Render lowercase, two-digit hexadecimal without a prefix.
            #[must_use]
            pub fn to_hex(&self) -> String {
                use std::fmt::Write as _;
                self.0.iter().take(32).fold(
                    String::with_capacity(64),
                    |mut output, byte| {
                        let _ = write!(output, "{byte:02x}");
                        output
                    },
                )
            }
        }
    };
}

digest_type!(
    /// Identity of one immutable reference-data cohort.
    RefDigest
);
digest_type!(
    /// SHA-256 identity of one checked prepared mechanics environment.
    PreparedEnvironmentDigest
);
digest_type!(
    /// SHA-256 identity of one stable graph plus governed world registers.
    StableWorldDigest
);
digest_type!(
    /// SHA-256 identity of one ordered accepted Practice action batch.
    OrderedPracticeActionBatchDigest
);
digest_type!(
    /// SHA-256 identity of one exact governed tick payload.
    TickPayloadDigest
);
digest_type!(
    /// SHA-256 identity of the fixed replay tick-content preimage.
    TickContentHash
);

/// Inputs to the fixed ten-section tick-content outer codec.
#[derive(Debug)]
pub struct TickContentParts<'a> {
    /// Checked material replay-session identity.
    pub session: &'a ReplaySessionId,
    /// One-based tick being resolved.
    pub resolve_tick: u64,
    /// Explicit replay seed.
    pub seed: ReplaySeed,
    /// Immutable mechanics content identity.
    pub content: &'a ContentDigest,
    /// Immutable reference-data identity.
    pub reference: RefDigest,
    /// Prepared mechanics environment identity.
    pub prepared: PreparedEnvironmentDigest,
    /// Stable world before adjudication.
    pub prior_world: StableWorldDigest,
    /// Ordered accepted Practice action identity.
    pub actions: OrderedPracticeActionBatchDigest,
    /// Stable world after adjudication.
    pub result_world: StableWorldDigest,
    /// Exact governed output payload identity.
    pub payload: TickPayloadDigest,
}

/// A checked outer-codec failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TickContentHashError {
    /// A checked integer conversion failed.
    IntegerConversion {
        /// Stable name of the field being converted.
        field: &'static str,
        /// Value that could not be represented.
        value: usize,
    },
    /// Canonical capacity arithmetic overflowed.
    CapacityOverflow {
        /// Stable name of the capacity being computed.
        field: &'static str,
    },
    /// The bounded canonical allocation failed.
    Allocation {
        /// Stable name of the requested allocation.
        field: &'static str,
        /// Exact requested capacity.
        requested: usize,
    },
}

/// Exact canonical outer bytes before SHA-256.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickContentPreimage(Vec<u8>);

impl TickContentPreimage {
    /// Compose the mandatory ten sections in their governed order.
    ///
    /// # Errors
    /// Returns [`TickContentHashError`] if checked capacity arithmetic,
    /// session-length conversion, or bounded allocation fails.
    pub fn compose(parts: &TickContentParts<'_>) -> Result<Self, TickContentHashError> {
        let session_length = u16::try_from(parts.session.as_bytes().len()).map_err(|_| {
            TickContentHashError::IntegerConversion {
                field: "tick-content replay session length",
                value: parts.session.as_bytes().len(),
            }
        })?;
        let capacity = TICK_CONTENT_FIXED_BYTES
            .checked_add(parts.session.as_bytes().len())
            .ok_or(TickContentHashError::CapacityOverflow {
                field: "tick-content preimage capacity",
            })?;
        let mut bytes = reserve_preimage(capacity)?;

        append_prefix_and_session(&mut bytes, parts, session_length);
        append_tick_and_seed(&mut bytes, parts);
        append_content(&mut bytes, parts.content);
        append_digest_section(&mut bytes, 0x05, parts.reference.as_bytes());
        append_digest_section(&mut bytes, 0x06, parts.prepared.as_bytes());
        append_digest_section(&mut bytes, 0x07, parts.prior_world.as_bytes());
        append_digest_section(&mut bytes, 0x08, parts.actions.as_bytes());
        append_digest_section(&mut bytes, 0x09, parts.result_world.as_bytes());
        append_digest_section(&mut bytes, 0x0a, parts.payload.as_bytes());
        debug_assert_eq!(bytes.len(), capacity);
        Ok(Self(bytes))
    }

    /// Borrow the exact canonical preimage bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Hash the exact canonical preimage.
    #[must_use]
    pub fn digest(&self) -> TickContentHash {
        TickContentHash::from_bytes(sha256_of(&self.0))
    }
}

fn reserve_preimage(capacity: usize) -> Result<Vec<u8>, TickContentHashError> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_: TryReserveError| TickContentHashError::Allocation {
            field: "tick-content preimage",
            requested: capacity,
        })?;
    Ok(bytes)
}

fn append_prefix_and_session(
    output: &mut Vec<u8>,
    parts: &TickContentParts<'_>,
    session_length: u16,
) {
    output.extend_from_slice(TICK_CONTENT_DOMAIN);
    output.extend_from_slice(&LAYOUT.to_be_bytes());
    output.push(0x01);
    output.extend_from_slice(&LAYOUT.to_be_bytes());
    output.extend_from_slice(&session_length.to_be_bytes());
    output.extend_from_slice(parts.session.as_bytes());
}

fn append_tick_and_seed(output: &mut Vec<u8>, parts: &TickContentParts<'_>) {
    output.push(0x02);
    output.extend_from_slice(&parts.resolve_tick.to_be_bytes());
    output.push(0x03);
    output.extend_from_slice(&LAYOUT.to_be_bytes());
    output.extend_from_slice(&RNG_LAYOUT.to_be_bytes());
    output.extend_from_slice(&parts.seed.to_be_bytes());
}

fn append_content(output: &mut Vec<u8>, content: &ContentDigest) {
    output.push(0x04);
    output.extend_from_slice(&LAYOUT.to_be_bytes());
    output.extend_from_slice(&content.defines_hash);
    output.extend_from_slice(&content.rules_hash);
}

fn append_digest_section(output: &mut Vec<u8>, tag: u8, digest: &[u8; 32]) {
    output.push(tag);
    output.extend_from_slice(&LAYOUT.to_be_bytes());
    output.extend_from_slice(digest);
}
