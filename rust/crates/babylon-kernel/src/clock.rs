//! Deterministic sim clock + per-tick correlation id (spec §6.5 — replaces
//! Python's `uuid4()` per-tick id, which was log-only and non-deterministic
//! by construction; this replacement is strictly better for the same job).
//!
//! No wall-clock reads, no randomness: the correlation id is a pure
//! function of `(session_id, tick)`, so two replays of the same session
//! produce identical log correlation — which is the point.

/// Designed simulation interval in weeks: one adjudication and one committed tick.
pub const WEEKS_PER_TICK: u64 = 4;
/// Designed simulation interval in days, with no internal weekly substeps.
pub const DAYS_PER_TICK: u64 = 28;
/// Four-week periods in one 52-week campaign year; this is not a civil calendar.
pub const TICKS_PER_YEAR: u64 = 13;

/// Opaque session identifier — a validated non-empty string, not a raw
/// `String`, so an empty session id is a construction-time error (III.11).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

/// The construction-time rejection of an empty session id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmptySessionId;

impl SessionId {
    /// Validate and wrap a session identifier.
    ///
    /// # Errors
    /// Returns [`EmptySessionId`] if `id` is the empty string — a loud
    /// III.11 construction failure, because an empty id would silently
    /// collapse every session's correlation ids into one namespace.
    pub fn new(id: impl Into<String>) -> Result<Self, EmptySessionId> {
        let id = id.into();
        if id.is_empty() {
            return Err(EmptySessionId);
        }
        Ok(Self(id))
    }

    /// The validated identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The identifier's UTF-8 bytes — the RNG seeding derivation's input
    /// form (`rng::seed_for`).
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// The four-week tick clock: monotonic, no wall-clock reads, no randomness.
#[derive(Debug, Clone)]
pub struct SimClock {
    session_id: SessionId,
    tick: u64,
}

impl SimClock {
    /// A fresh clock at tick 0 for the given session.
    #[must_use]
    pub fn new(session_id: SessionId) -> Self {
        Self {
            session_id,
            tick: 0,
        }
    }

    /// Advance one four-week tick; returns the new period ordinal.
    ///
    /// # Panics
    /// Panics on u64 overflow — unreachable for any campaign (a 10-year
    /// campaign is 130 ticks; u64 holds 1.8e19), but checked rather than
    /// wrapped because a silently restarted tick counter would corrupt
    /// every downstream correlation (III.11).
    pub fn advance(&mut self) -> u64 {
        self.tick = self
            .tick
            .checked_add(1)
            .expect("SimClock::advance: tick counter overflow");
        self.tick
    }

    /// The current tick number.
    #[must_use]
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// The clock's session.
    #[must_use]
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Deterministic per-tick correlation id — a pure function of
    /// `(session_id, tick)`, never a UUID. Zero-padded to ten digits so
    /// ids sort lexicographically in tick order in any log viewer.
    #[must_use]
    pub fn correlation_id(&self) -> String {
        format!("{}-{:010}", self.session_id.0, self.tick)
    }
}

#[cfg(test)]
mod tests {
    use super::{EmptySessionId, SessionId, SimClock};

    #[test]
    fn correlation_id_is_a_pure_function_of_session_and_tick() {
        let clock_a = SimClock {
            session_id: SessionId::new("abc").unwrap(),
            tick: 3,
        };
        let clock_b = SimClock {
            session_id: SessionId::new("abc").unwrap(),
            tick: 3,
        };
        assert_eq!(clock_a.correlation_id(), clock_b.correlation_id());
    }

    #[test]
    fn correlation_id_sorts_lexicographically_in_tick_order() {
        let mut clock = SimClock::new(SessionId::new("s").unwrap());
        let mut previous = clock.correlation_id();
        for _ in 0..12 {
            clock.advance();
            let current = clock.correlation_id();
            assert!(previous < current, "{previous} !< {current}");
            previous = current;
        }
    }

    #[test]
    fn advance_is_monotonic_and_never_resets() {
        let mut clock = SimClock::new(SessionId::new("s").unwrap());
        assert_eq!(clock.advance(), 1);
        assert_eq!(clock.advance(), 2);
        assert_eq!(clock.tick(), 2);
    }

    #[test]
    fn empty_session_id_is_a_loud_construction_error() {
        assert_eq!(SessionId::new(""), Err(EmptySessionId));
    }

    #[test]
    fn different_sessions_never_share_a_correlation_id() {
        let a = SimClock::new(SessionId::new("a").unwrap());
        let b = SimClock::new(SessionId::new("b").unwrap());
        assert_ne!(a.correlation_id(), b.correlation_id());
    }
}
