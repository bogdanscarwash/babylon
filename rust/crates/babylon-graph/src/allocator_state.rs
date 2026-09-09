//! Allocator state that is real world state but not graph content.
//!
//! [`crate::state_hash::CanonicalState`] deliberately hashes live graph
//! facts only. Monotonic identity cursors are different: removing an object
//! does not rewind them, and the next minted identity affects every later
//! reference. Tick-level world hashing therefore consumes this sibling
//! contract without changing the established graph-state byte layout.

/// The cursor state of each monotonic graph identity allocator.
///
/// `u64::MAX` is the reserved exhausted-cursor sentinel for either lane. It
/// is never a mintable identity. The last mintable identity is therefore
/// `u64::MAX - 1`; a successful mint there advances its cursor to the
/// sentinel, and every later mint refuses without mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocatorCursors {
    /// The next [`crate::substrate::NodeId`] value, or `u64::MAX` when the
    /// node allocator is exhausted.
    pub next_node: u64,
    /// The next [`crate::substrate::HyperedgeId`] value, or `u64::MAX` when
    /// the hyperedge allocator is exhausted.
    pub next_hyperedge: u64,
}

/// Read-only access to identity allocator state for transaction and hash
/// boundaries.
pub trait AllocatorState {
    /// Return both next-id cursors without advancing either allocator.
    fn allocator_cursors(&self) -> AllocatorCursors;
}

/// Narrow test-only control for proving allocator exhaustion behavior. No
/// production caller may set monotonic identity cursors.
#[cfg(test)]
pub(crate) trait AllocatorTestControl {
    /// Set both cursors to an otherwise unreachable boundary fixture.
    fn set_allocator_cursors_for_test(&mut self, cursors: AllocatorCursors);
}
