//! Fixed simulation interval and the evidence record session identifier.

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

    /// The exact UTF-8 bytes used by evidence record encoding.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::{EmptySessionId, SessionId};

    #[test]
    fn empty_session_id_is_a_loud_construction_error() {
        assert_eq!(SessionId::new(""), Err(EmptySessionId));
    }
}
