//! The kernel RNG service (spec §9, R8 + the ADR176 ruling-20 rider):
//! one pinned algorithm, **per-carrier counter-based streams**.
//!
//! **Algorithm choice (Phase-1 engineering call, not amendment-gated — R8
//! already authorizes the stream divergence from Python's MT19937):**
//! `ChaCha8Rng` (`rand_chacha`). Rationale: (1) it takes an exact 32-byte
//! seed, which is exactly a SHA-256 digest's width — the seeding derivation
//! below needs no truncation/expansion step, unlike a generator wanting a
//! `u64` or `[u8; 16]` seed; (2) it is a pure-Rust, no-`unsafe`,
//! platform-independent stream-cipher construction with strong statistical
//! properties and no OS-entropy dependency (fully deterministic from its
//! seed, required for III.7); (3) 8 rounds is the documented "fast, still
//! no known practical distinguisher" configuration — this is not a
//! cryptographic-security use case, so `ChaCha8` is preferred over
//! `ChaCha20` purely for speed with no correctness cost; (4) `ChaCha` is
//! itself a counter-mode construction, so a carrier's stream position IS
//! the rider's per-draw counter — no extra bookkeeping.
//!
//! **Why per-carrier streams and not one stream per tick (ADR176 r20;
//! `reports/design-inputs-dossier-2026-07-29.md` §6.3):** with one stream
//! consumed in iteration order, adding a single carrier shifts every later
//! draw that tick — LOD refinement becomes a butterfly generator and every
//! stochastic family grain-couples. Deriving each stream from the
//! carrier's OWN identity `(domain, stable_key)` makes draws depend only
//! on that identity: grain-invariant by construction, and refinement needs
//! no RNG state migration because children derive streams from their own
//! ids. The API deliberately offers NO tick-global stream, so the
//! butterfly shape cannot be reached by accident (III.11 posture).
//!
//! **Streams differ from Python by design (R8):** this is the pinned
//! Rust-side replacement, not a port. Python's MT19937 streams are a
//! closed epoch; stochastic baselines re-bless at cutover under
//! ensemble-envelope comparison, not byte replay.
use crate::clock::SessionId;
use crate::replay::{ReplayIdentityError, ReplaySeed, ReplaySessionIdV1, RngDomainV2};
use rand_chacha::ChaCha8Rng;
use rand_core::{Rng, SeedableRng};
use sha2::{Digest, Sha256};

/// Mirrors `kernel/system_base.py::_SYSTEM_RNG_SEED_SALT` structurally
/// (same salt constant in the mixing) — NOT the same stream, per R8.
pub const SEED_SALT: u64 = 0x0BA1_AC1A;

/// Derive the 32-byte seed for one carrier's stream:
/// `SHA256(session_utf8 ‖ tick_le8 ‖ salt_le8 ‖ len_le8(domain) ‖
/// domain_utf8 ‖ len_le8(stable_key) ‖ stable_key_utf8)`.
///
/// `domain` names the stochastic family (e.g. `"bifurcation"`); `stable_key`
/// names the carrier within it (e.g. a class id, a hex index). Both are
/// **length-prefixed** (8-byte little-endian) so `("ab", "c")` and
/// `("a", "bc")` can never collide — concatenation without framing would
/// make stream identity depend on where the strings split.
#[must_use]
pub fn seed_for(session_id: &SessionId, tick: u64, domain: &str, stable_key: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(session_id.as_bytes());
    hasher.update(tick.to_le_bytes());
    hasher.update(SEED_SALT.to_le_bytes());
    hasher.update((domain.len() as u64).to_le_bytes());
    hasher.update(domain.as_bytes());
    hasher.update((stable_key.len() as u64).to_le_bytes());
    hasher.update(stable_key.as_bytes());
    hasher.finalize().into()
}

/// Derive the V2 `ChaCha8` key from one checked replay identity and validated
/// carrier bytes.
///
/// The raw carrier bytes are a low-level adapter and contract-test entry
/// point. Graph owns the authoritative provenance and validation of that key.
///
/// # Errors
/// Returns [`ReplayIdentityError`] if a carrier length cannot be represented
/// in this version's required big-endian `u32` field.
pub fn seed_for_v2(
    session: &ReplaySessionIdV1,
    seed: ReplaySeed,
    tick: u64,
    domain: &RngDomainV2,
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
/// `(session_id, tick, domain, stable_key)` (III.7 + the r20 rider).
pub struct KernelRng(ChaCha8Rng);

impl KernelRng {
    /// The stream for one carrier at one tick.
    #[must_use]
    pub fn for_carrier(session_id: &SessionId, tick: u64, domain: &str, stable_key: &str) -> Self {
        Self(ChaCha8Rng::from_seed(seed_for(
            session_id, tick, domain, stable_key,
        )))
    }

    /// The V2 stream for one graph-validated carrier at one replay tick.
    ///
    /// # Errors
    /// Returns [`ReplayIdentityError`] when the exact V2 key preimage cannot
    /// encode one of its checked fields.
    pub fn for_carrier_v2(
        session: &ReplaySessionIdV1,
        seed: ReplaySeed,
        tick: u64,
        domain: &RngDomainV2,
        validated_carrier_key: &[u8],
    ) -> Result<Self, ReplayIdentityError> {
        let key = seed_for_v2(session, seed, tick, domain, validated_carrier_key)?;
        Ok(Self(ChaCha8Rng::from_seed(key)))
    }

    /// The next 64 uniformly distributed bits. The stream position behind
    /// this call is the rider's per-draw counter — `ChaCha` is counter-mode
    /// by construction.
    pub fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    /// A uniform draw on `[0, 1)`: the top 53 bits of one `u64` scaled by
    /// 2⁻⁵³ — every representable value is an exact multiple of 2⁻⁵³, so
    /// the mapping is bit-deterministic across platforms (no libm, no
    /// rounding-mode dependence).
    #[allow(clippy::cast_precision_loss)] // 53-bit value: exact in f64 by construction
    pub fn next_f64(&mut self) -> f64 {
        let bits53 = self.0.next_u64() >> 11;
        (bits53 as f64) * (1.0 / (1u64 << 53) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::{seed_for, KernelRng, SEED_SALT};
    use crate::clock::SessionId;

    #[test]
    fn same_carrier_reproduces_the_same_stream() {
        let sid = SessionId::new("s1").unwrap();
        let mut a = KernelRng::for_carrier(&sid, 7, "bifurcation", "C001");
        let mut b = KernelRng::for_carrier(&sid, 7, "bifurcation", "C001");
        for _ in 0..8 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_ticks_diverge() {
        let sid = SessionId::new("s1").unwrap();
        let mut a = KernelRng::for_carrier(&sid, 7, "bifurcation", "C001");
        let mut b = KernelRng::for_carrier(&sid, 8, "bifurcation", "C001");
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn different_sessions_diverge() {
        let a_id = SessionId::new("s1").unwrap();
        let b_id = SessionId::new("s2").unwrap();
        assert_ne!(seed_for(&a_id, 7, "d", "k"), seed_for(&b_id, 7, "d", "k"));
    }

    #[test]
    fn different_carriers_in_one_domain_diverge() {
        let sid = SessionId::new("s1").unwrap();
        assert_ne!(
            seed_for(&sid, 7, "bifurcation", "C001"),
            seed_for(&sid, 7, "bifurcation", "C002")
        );
    }

    #[test]
    fn domain_and_key_are_framed_not_concatenated() {
        // ("ab","c") vs ("a","bc"): unframed concatenation would collide.
        let sid = SessionId::new("s").unwrap();
        assert_ne!(seed_for(&sid, 1, "ab", "c"), seed_for(&sid, 1, "a", "bc"));
    }

    #[test]
    fn adding_a_carrier_cannot_shift_another_carriers_draws() {
        // The r20 rider's whole point (grain invariance): C001's stream is
        // identical whether or not C002 ever draws.
        let sid = SessionId::new("s").unwrap();
        let mut alone = KernelRng::for_carrier(&sid, 3, "metabolism", "C001");
        let lone_draws = [alone.next_u64(), alone.next_u64()];

        let mut c001 = KernelRng::for_carrier(&sid, 3, "metabolism", "C001");
        let mut c002 = KernelRng::for_carrier(&sid, 3, "metabolism", "C002");
        let _ = c002.next_u64(); // another carrier draws in between
        let first = c001.next_u64();
        let _ = c002.next_u64();
        let second = c001.next_u64();
        assert_eq!([first, second], lone_draws);
    }

    #[test]
    fn the_salt_is_the_python_constant() {
        // Structural mirror of _SYSTEM_RNG_SEED_SALT = 0xBA1AC1A.
        assert_eq!(SEED_SALT, 0xBA1_AC1A);
    }

    #[test]
    fn next_f64_is_in_the_half_open_unit_interval() {
        let sid = SessionId::new("s").unwrap();
        let mut rng = KernelRng::for_carrier(&sid, 1, "d", "k");
        for _ in 0..1000 {
            let v = rng.next_f64();
            assert!((0.0..1.0).contains(&v), "{v}");
        }
    }

    /// The within-implementation replay conformance vector (plan Task 5
    /// Step 2): generated ONCE from this implementation and pinned — any
    /// future divergence is a determinism regression, never "the RNG got
    /// better". Mirrored in the determinism contract's RNG chapter.
    #[test]
    fn conformance_vector_first_four_u64s() {
        let sid = SessionId::new("conformance").unwrap();
        let mut rng = KernelRng::for_carrier(&sid, 1, "conformance-domain", "carrier-0");
        let observed = [
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
        ];
        // Filled from the first green run (2026-07-30, rider derivation)
        // and byte-pinned thereafter.
        let pinned: [u64; 4] = [
            0x6774_721d_2209_092f,
            0x6d42_2bc9_af84_28f1,
            0x0ce2_91ab_fcb1_1e7a,
            0xdd11_9629_7249_5117,
        ];
        assert_eq!(observed, pinned);
    }
}
