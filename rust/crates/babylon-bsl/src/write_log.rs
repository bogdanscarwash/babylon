//! The BSL write log (ADR182 R1): every mutation the effect executor
//! performs, in source order, attributed to the rule that performed it.
//!
//! This exists because of a *window*, not a feature request. Every field
//! write in the language already funnels through one place —
//! [`crate::structural_verbs::EffectExecutor`] — and the production store
//! swaps in at the Phase 1/2 boundary. Installing the interception point
//! while that boundary is under construction costs a field and a `Vec`;
//! installing it afterwards is a retrofit across every write site. The
//! Director ruled the log ships now and the diff UI that consumes it ships
//! on its own schedule (ADR182 R1).
//!
//! Three disciplines hold here, each pinned by a test:
//!
//! 1. **The log is an observer, never a participant.** An observed run and
//!    an unobserved run of the same effect list leave identical graph state
//!    and consume identical fuel. Observation is not a semantic mode.
//! 2. **A record is emitted only after the substrate call succeeds.** A
//!    write that failed the store-boundary range check, the existence
//!    discipline, or the fuel meter leaves no record — absence is never a
//!    write, the mirror of §2.8's "absence is never success".
//! 3. **`previous` is a probe, not a read-modify-write.** For an update the
//!    executor did not already have to read (`set`), the prior value is
//!    probed with [`babylon_graph::substrate::GraphSubstrate::node_attribute`]
//!    and a failure is recorded as `None`. This is the one place in the
//!    crate that deliberately discards a substrate error, and it is sound
//!    for exactly one reason: `node_attribute` fails when the attribute has
//!    never been written (the §3.5 honest-null discipline), so `None` *is*
//!    the fact being recorded. Propagating that error instead would make an
//!    observed run fail where an unobserved run succeeds — discipline 1
//!    violated, and a determinism divergence with it.

use babylon_graph::substrate::{HyperedgeId, NodeId};
use babylon_kernel::currency::Currency;

/// One mutation, as it crossed the store boundary.
///
/// The variants record supported node, edge, and hyperedge field updates. `emit` is absent
/// on purpose: an event is not a write, and it already has its own seam
/// ([`crate::structural_verbs::EventSink`]).
#[derive(Debug, Clone, PartialEq)]
pub enum Write {
    /// A field write from `update-node`.
    NodeAttribute {
        /// The node written to.
        id: NodeId,
        /// The fully-qualified field name.
        field: String,
        /// The value the field held before this write, or `None` where it
        /// held nothing (see the module doc's discipline 3).
        previous: Option<f64>,
        /// The value now stored.
        value: f64,
    },
    /// A Currency field write — from `update-node` against a
    /// `currency`-declared field (T3 #491, OQ-J: the i128 typed-storage
    /// lane). `set` only — Currency has no read-modify-write here (see
    /// `structural_verbs::EffectExecutor`'s currency write fork), so unlike
    /// [`Self::NodeAttribute`] this variant never represents an `add`/`sub`/
    /// `scale` combine.
    NodeCurrencyAttribute {
        /// The node written to.
        id: NodeId,
        /// The fully-qualified field name.
        field: String,
        /// The value the field held before this write, or `None` where it
        /// held nothing (the same discipline 3 probe, through
        /// [`babylon_graph::substrate::GraphSubstrate::node_attribute_currency`]).
        previous: Option<Currency>,
        /// The value now stored.
        value: Currency,
    },
    /// A field write on a dyadic edge — from `update-edge`, or from an
    /// `add-edge` field-init (T3, ADR198 R1/R3, issue #560). A write to
    /// `<edge-type>/strength` records here too — the record reports WHICH
    /// field moved; the storage routing (0x03 slot vs fifth-section row,
    /// D143) is the substrate's business, not the log's.
    EdgeAttribute {
        /// The edge's declared type.
        edge_type: String,
        /// Source node.
        from: NodeId,
        /// Target node.
        to: NodeId,
        /// The fully-qualified field name.
        field: String,
        /// The value the field held before this write, or `None` where it
        /// held nothing (discipline 3 — the probe reads through
        /// [`babylon_graph::substrate::GraphSubstrate::edge_attribute`], so a
        /// never-written deffield field records `None` and a strength write
        /// records the prior strength).
        previous: Option<f64>,
        /// The value now stored.
        value: f64,
    },

    /// A field write on a hyperedge — from `update-hyperedge`, on either
    /// dispatch site (Community port train, Task 6, E2b). An
    /// `(hyperedge-attr …)` scenario seed does NOT produce one: hydration
    /// writes the substrate directly and carries no write log (the same
    /// convention as `(edge-attr …)`'s direct `update_edge`).
    /// Mirrors [`Self::EdgeAttribute`]'s shape minus the endpoints — a
    /// hyperedge's identity is its id alone.
    HyperedgeAttribute {
        /// The hyperedge written to.
        id: HyperedgeId,
        /// The fully-qualified field name.
        field: String,
        /// The value the field held before this write, or `None` where it
        /// held nothing (the same discipline-3 probe, through
        /// [`babylon_graph::substrate::GraphSubstrate::hyperedge_attribute`]).
        previous: Option<f64>,
        /// The value now stored.
        value: f64,
    },
}

/// A [`Write`] with its attribution: which rule performed it, and where in
/// that rule's effect list it fell.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteRecord {
    /// The rule id (`<system>/<rule-name>`) whose effect list produced this
    /// write. Empty when the executor was constructed without attribution.
    pub rule: String,
    /// Source-order position within this effect list, from 0. Guards do not
    /// reset it and an untaken branch does not advance it: the ordinal
    /// counts writes that happened, not effect items that were considered.
    pub ordinal: u32,
    /// What crossed the boundary.
    pub write: Write,
}

/// Where the write log lands. The engine wires the production implementation
/// at the Phase 1/2 boundary; the inspector's diff pane is one consumer, a
/// replay/audit trail is another.
///
/// There is deliberately no null implementation: the executor holds an
/// `Option<&mut dyn WriteObserver>`, so *not observing* is `None` rather
/// than a do-nothing object, and an unobserved run does no observer work at
/// all.
pub trait WriteObserver {
    /// Record one mutation. Called only after the substrate accepted it.
    fn record(&mut self, record: WriteRecord);
}

/// An observer that simply collects, for tests, the conformance corpus, and
/// any consumer that wants the whole effect list at once.
#[derive(Debug, Default)]
pub struct CollectingWriteLog {
    /// Every write, in the order it crossed the boundary.
    pub records: Vec<WriteRecord>,
}

impl CollectingWriteLog {
    /// A fresh, empty log.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Just the [`Write`]s, discarding attribution — the shape most
    /// assertions want.
    #[must_use]
    pub fn writes(&self) -> Vec<Write> {
        self.records.iter().map(|r| r.write.clone()).collect()
    }
}

impl WriteObserver for CollectingWriteLog {
    fn record(&mut self, record: WriteRecord) {
        self.records.push(record);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_collecting_log_preserves_arrival_order() {
        let mut log = CollectingWriteLog::new();
        for (ordinal, id) in [(0_u32, 7_u64), (1, 3), (2, 9)] {
            log.record(WriteRecord {
                rule: "hunger/agitate".to_owned(),
                ordinal,
                write: Write::NodeAttribute {
                    id: NodeId(id),
                    field: "social-class/agitation".to_owned(),
                    previous: None,
                    value: 0.5,
                },
            });
        }
        assert_eq!(
            log.writes(),
            vec![
                Write::NodeAttribute {
                    id: NodeId(7),
                    field: "social-class/agitation".to_owned(),
                    previous: None,
                    value: 0.5
                },
                Write::NodeAttribute {
                    id: NodeId(3),
                    field: "social-class/agitation".to_owned(),
                    previous: None,
                    value: 0.5
                },
                Write::NodeAttribute {
                    id: NodeId(9),
                    field: "social-class/agitation".to_owned(),
                    previous: None,
                    value: 0.5
                },
            ],
            "arrival order is the record, not id order"
        );
        assert_eq!(
            log.records.iter().map(|r| r.ordinal).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
    }
}
