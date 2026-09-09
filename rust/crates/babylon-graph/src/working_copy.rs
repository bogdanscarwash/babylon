//! Explicit deep-copy contract for transactional graph work.
//!
//! A bare [`Clone`] bound cannot say whether a backend shares mutable
//! internals. Tick adjudication depends on a stronger behavioral promise:
//! mutations and allocator advances on the returned value never affect the
//! source. Implementations are therefore explicit and intentionally have no
//! blanket `T: Clone` implementation.

/// Construct an independently mutable graph working copy.
pub trait DetachedCopy: Sized {
    /// Copy every mutable graph lane and identity cursor into a detached
    /// value suitable for transactional mutation.
    #[must_use]
    fn detached_copy(&self) -> Self;
}
