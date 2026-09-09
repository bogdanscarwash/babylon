//! Checked replay identity primitives and parsed RNG layout selection.
//!
//! These values define the database-free replay boundary. They intentionally
//! have no conversion to campaign, persistence, or graph identities.
use crate::clock::SessionId;
use std::collections::TryReserveError;

const MIN_REPLAY_SESSION_BYTES: usize = 1;
const MAX_REPLAY_SESSION_BYTES: usize = 256;
const MIN_RNG_DOMAIN_BYTES: usize = 1;
const MAX_RNG_DOMAIN_BYTES: usize = 128;
const MIN_RNG_DOMAIN_SEGMENTS: usize = 2;
const MAX_RNG_DOMAIN_SEGMENTS: usize = 4;
const MAX_RNG_DOMAIN_SEGMENT_BYTES: usize = 64;

/// A checked replay identity failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayIdentityError {
    /// A byte was not in the strict ASCII graphic range `0x21..=0x7e`.
    InvalidAsciiGraphic {
        /// The validated field's stable name.
        field: &'static str,
        /// The offending byte's zero-based index.
        index: usize,
        /// The offending byte.
        byte: u8,
    },
    /// A field did not meet its inclusive byte-length bounds.
    LengthOutOfBounds {
        /// The validated field's stable name.
        field: &'static str,
        /// The received byte length.
        actual: usize,
        /// The inclusive lower bound.
        minimum: usize,
        /// The inclusive upper bound.
        maximum: usize,
    },
    /// An RNG domain did not match the governed BSL qname grammar.
    InvalidRngDomainQname {
        /// The zero-based byte index where validation failed.
        index: usize,
    },
    /// A numeric layout value is not governed by this replay boundary.
    UnsupportedRngLayoutVersion {
        /// The unsupported numeric value.
        value: u32,
    },
    /// A checked integer conversion needed for a canonical codec failed.
    IntegerConversion {
        /// The conversion's stable field name.
        field: &'static str,
        /// The value that could not be represented.
        value: usize,
    },
    /// A bounded canonical allocation could not reserve its required bytes.
    Allocation {
        /// The allocation's stable field name.
        field: &'static str,
        /// The requested capacity.
        requested: usize,
    },
}

/// The authoritative logical replay session namespace, encoded as strict
/// graphic ASCII and never normalized.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReplaySessionIdV1(Vec<u8>);

impl ReplaySessionIdV1 {
    /// The exact session bytes after checked construction.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Encode this session as a big-endian `u16` byte length and exact bytes.
    ///
    /// # Errors
    /// Returns [`ReplayIdentityError`] if the bounded output cannot be
    /// allocated or its checked length could not be represented as `u16`.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ReplayIdentityError> {
        let length = checked_u16("replay session length", self.0.len())?;
        let capacity =
            self.0
                .len()
                .checked_add(2)
                .ok_or(ReplayIdentityError::IntegerConversion {
                    field: "replay session canonical capacity",
                    value: self.0.len(),
                })?;
        let mut encoded = reserve_bytes("replay session canonical bytes", capacity)?;
        encoded.extend_from_slice(&length.to_be_bytes());
        encoded.extend_from_slice(&self.0);
        Ok(encoded)
    }
}

impl TryFrom<&[u8]> for ReplaySessionIdV1 {
    type Error = ReplayIdentityError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        validate_ascii_graphic::<MIN_REPLAY_SESSION_BYTES, MAX_REPLAY_SESSION_BYTES>(
            "replay session",
            value,
        )?;
        let mut bytes = reserve_bytes("replay session", value.len())?;
        bytes.extend_from_slice(value);
        Ok(Self(bytes))
    }
}

impl TryFrom<&str> for ReplaySessionIdV1 {
    type Error = ReplayIdentityError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.as_bytes())
    }
}

/// A caller-supplied replay seed with its full signed `i64` domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReplaySeed(i64);

impl ReplaySeed {
    /// Wrap one explicit signed replay seed.
    #[must_use]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    /// Return the seed's canonical signed big-endian bytes.
    #[must_use]
    pub const fn to_be_bytes(self) -> [u8; 8] {
        self.0.to_be_bytes()
    }
}

/// The governed RNG layouts parsed once at the replay boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RngLayoutVersion {
    /// The frozen current `TickSession` layout.
    V1,
    /// The seed-aware replay layout.
    V2,
}

impl TryFrom<u32> for RngLayoutVersion {
    type Error = ReplayIdentityError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::V1),
            2 => Ok(Self::V2),
            _ => Err(ReplayIdentityError::UnsupportedRngLayoutVersion { value }),
        }
    }
}

/// A checked V2 RNG domain: a strict-ASCII firing-rule qname.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RngDomainV2(String);

impl RngDomainV2 {
    /// Borrow the checked UTF-8 domain text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Borrow the checked ASCII domain bytes for a canonical preimage.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl TryFrom<&str> for RngDomainV2 {
    type Error = ReplayIdentityError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_ascii_graphic::<MIN_RNG_DOMAIN_BYTES, MAX_RNG_DOMAIN_BYTES>(
            "RNG domain",
            value.as_bytes(),
        )?;
        validate_rng_domain_qname(value.as_bytes())?;
        let mut bytes = reserve_bytes("RNG domain", value.len())?;
        bytes.extend_from_slice(value.as_bytes());
        let domain =
            String::from_utf8(bytes).map_err(|_| ReplayIdentityError::IntegerConversion {
                field: "RNG domain UTF-8",
                value: value.len(),
            })?;
        Ok(Self(domain))
    }
}

/// The typed context that prevents replay callers from selecting an unparsed
/// numeric RNG layout.
#[derive(Debug, Clone, Copy)]
pub enum RngSeedContext<'a> {
    /// Frozen V1 derivation using the current `TickSession` identity.
    V1 {
        /// The current `TickSession` session identity.
        session: &'a SessionId,
    },
    /// V2 derivation using the replay session and explicit replay seed.
    V2 {
        /// The checked replay session identity.
        session: &'a ReplaySessionIdV1,
        /// The explicit replay seed.
        seed: ReplaySeed,
    },
}

fn validate_rng_domain_qname(value: &[u8]) -> Result<(), ReplayIdentityError> {
    let mut segment_count = 1usize;
    let mut segment_length = 0usize;
    for index in 0..MAX_RNG_DOMAIN_BYTES {
        let Some(byte) = value.get(index).copied() else {
            break;
        };
        if byte == b'/' {
            if segment_length == 0 || segment_count == MAX_RNG_DOMAIN_SEGMENTS {
                return Err(ReplayIdentityError::InvalidRngDomainQname { index });
            }
            segment_count = segment_count.saturating_add(1);
            segment_length = 0;
            continue;
        }
        let valid = if segment_length == 0 {
            byte.is_ascii_lowercase()
        } else {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
        };
        if !valid || segment_length == MAX_RNG_DOMAIN_SEGMENT_BYTES {
            return Err(ReplayIdentityError::InvalidRngDomainQname { index });
        }
        segment_length = segment_length.saturating_add(1);
    }
    if segment_count < MIN_RNG_DOMAIN_SEGMENTS || segment_length == 0 {
        return Err(ReplayIdentityError::InvalidRngDomainQname { index: value.len() });
    }
    Ok(())
}

fn validate_ascii_graphic<const MINIMUM: usize, const MAXIMUM: usize>(
    field: &'static str,
    value: &[u8],
) -> Result<(), ReplayIdentityError> {
    if value.len() < MINIMUM || value.len() > MAXIMUM {
        return Err(ReplayIdentityError::LengthOutOfBounds {
            field,
            actual: value.len(),
            minimum: MINIMUM,
            maximum: MAXIMUM,
        });
    }
    for index in 0..MAXIMUM {
        let Some(byte) = value.get(index).copied() else {
            break;
        };
        if !(0x21..=0x7e).contains(&byte) {
            return Err(ReplayIdentityError::InvalidAsciiGraphic { field, index, byte });
        }
    }
    Ok(())
}

fn checked_u16(field: &'static str, value: usize) -> Result<u16, ReplayIdentityError> {
    u16::try_from(value).map_err(|_| ReplayIdentityError::IntegerConversion { field, value })
}

fn reserve_bytes(field: &'static str, capacity: usize) -> Result<Vec<u8>, ReplayIdentityError> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_: TryReserveError| ReplayIdentityError::Allocation {
            field,
            requested: capacity,
        })?;
    Ok(bytes)
}
