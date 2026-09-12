//! Deterministic per-carrier `ChaCha8` streams keyed by explicit replay identity.

use crate::replay::{ReplayIdentityError, ReplaySeed, ReplaySessionId, RngDomain};
use rand_chacha::ChaCha8Rng;
use rand_core::{Rng, SeedableRng};
use sha2::{Digest, Sha256};

/// Derive the V2 `ChaCha8` key from one checked replay identity and validated
/// carrier bytes.
///
/// The raw carrier bytes are a low-level adapter and contract-test entry
/// point. Graph owns the authoritative provenance and validation of that key.
///
/// # Errors
/// Returns [`ReplayIdentityError`] if a carrier length cannot be represented
/// in this version's required big-endian `u32` field.
pub fn seed_for(
    session: &ReplaySessionId,
    seed: ReplaySeed,
    tick: u64,
    domain: &RngDomain,
    validated_carrier_key: &[u8],
) -> Result<[u8; 32], ReplayIdentityError> {
    let carrier_length = u32::try_from(validated_carrier_key.len()).map_err(|_| {
        ReplayIdentityError::IntegerConversion {
            field: "RNG V2 carrier length",
            value: validated_carrier_key.len(),
        }
    })?;
    let domain_length = u32::try_from(domain.as_bytes().len()).map_err(|_| {
        ReplayIdentityError::IntegerConversion {
            field: "RNG V2 domain length",
            value: domain.as_bytes().len(),
        }
    })?;
    let mut hasher = Sha256::new();
    hasher.update(b"babylon.rng-stream\0");
    hasher.update(2u32.to_be_bytes());
    hasher.update([0x01]);
    hasher.update(seed.to_be_bytes());
    hasher.update([0x02]);
    hasher.update(session.canonical_bytes()?);
    hasher.update([0x03]);
    hasher.update(tick.to_be_bytes());
    hasher.update([0x04]);
    hasher.update(domain_length.to_be_bytes());
    hasher.update(domain.as_bytes());
    hasher.update([0x05]);
    hasher.update(carrier_length.to_be_bytes());
    hasher.update(validated_carrier_key);
    Ok(hasher.finalize().into())
}

/// One carrier's pinned stream. Constructed only via
/// [`KernelRng::for_carrier`] — there is deliberately no `from_entropy()`
/// and no tick-global constructor: every stream is a pure function of
/// `(session_id, seed, tick, domain, stable_key)` (III.7 + the r20 rider).
pub struct KernelRng(ChaCha8Rng);

impl KernelRng {
    /// The V2 stream for one graph-validated carrier at one replay tick.
    ///
    /// # Errors
    /// Returns [`ReplayIdentityError`] when the exact V2 key preimage cannot
    /// encode one of its checked fields.
    pub fn for_carrier(
        session: &ReplaySessionId,
        seed: ReplaySeed,
        tick: u64,
        domain: &RngDomain,
        validated_carrier_key: &[u8],
    ) -> Result<Self, ReplayIdentityError> {
        let key = seed_for(session, seed, tick, domain, validated_carrier_key)?;
        Ok(Self(ChaCha8Rng::from_seed(key)))
    }

    /// The next 64 uniformly distributed bits. The stream position behind
    /// this call is the rider's per-draw counter — `ChaCha` is counter-mode
    /// by construction.
    pub fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }
}
