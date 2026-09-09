//! The shared BACKFIRE term — why repression sometimes recruits (Lane A;
//! heat dossier §5.3 mode 7 and §6's emergence check).
//!
//! Indiscriminate force does not have a fixed sign. Santiago de María was
//! suppressed by it; other populations were radicalized by the same act. The
//! record's explanation is structural rather than psychological: what differs
//! is **how many of the affected people stood inside a structure** when it
//! happened.
//!
//! Two channels, each a **count of people in a structural position**:
//!
//! - **Protected mass** (Kalyvas) — those inside a rival's protective reach.
//! - **Interpreted mass** (Wood's catechists) — those inside a community or
//!   organizational structure that frames the event for them.
//!
//! **These are measures, not curves.** Both are fractions of a real
//! population, combined as a sum. There is no multiplier, no threshold, and
//! no fitted shape anywhere in this module — which is the ADR172 ruling-5
//! discipline (no imposed functional forms) applied to the one term most
//! likely to attract one. If a coefficient ever appears here, the finding it
//! encodes has been replaced by a stipulation.
//!
//! **The partition is a declared modelling choice, not a finding.** The
//! dossier flags it honestly (§6): the record does not say how the two
//! channels interact when they overlap — Wood's case had interpretation
//! without protection, Kalyvas's had protection without much interpretation.
//! Treating them as a partition, with **protection taking precedence**, is
//! the dossier's construction; it is defensible because a person is either
//! inside a protective reach or not, but the record does not settle it. It is
//! recorded here so it stays visible and reversible instead of hardening into
//! an assumed truth.
//!
//! Because the two cells are disjoint by construction, the sum is a measure
//! over the affected population and never exceeds 1.

use crate::substrate::{Direction, GraphError, GraphSubstrate, NodeId};
use std::collections::BTreeSet;

/// The two masses, as fractions of the affected population.
///
/// Kept apart rather than pre-summed: they are different social facts, and a
/// narrator or rule may want to speak about them separately. [`Self::total`]
/// is the composition the dossier specifies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Backfire {
    /// Fraction of the affected inside a rival's protective reach.
    pub protected_fraction: f64,
    /// Fraction of the affected who are NOT protected but stand inside a
    /// structure that frames the event.
    pub interpreted_fraction: f64,
}

impl Backfire {
    /// The sum of the two measures over the partition.
    ///
    /// In `[0, 1]` by construction — the cells are disjoint and both are
    /// fractions of the same denominator. Where both are ~0 the repression
    /// suppresses; where either is substantial it recruits. That reversal is
    /// the whole finding, and it is arithmetic on counted people rather than
    /// a sign flip anybody wrote down.
    #[must_use]
    pub fn total(&self) -> f64 {
        self.protected_fraction + self.interpreted_fraction
    }
}

/// Whether `node` holds at least one edge of `edge_type`, either direction.
///
/// Undirected because standing inside a structure is not a claim about who
/// authored the tie.
fn stands_inside(
    graph: &impl GraphSubstrate,
    node: NodeId,
    edge_type: &str,
) -> Result<bool, GraphError> {
    Ok(!graph.neighbors(node, edge_type, Direction::Any)?.is_empty())
}

/// Measure the backfire channels over an affected population.
///
/// `protective_edge_type` is the tie that puts someone inside a rival's
/// reach; `interpreting_edge_type` is the tie that puts them inside a
/// framing structure. Both are supplied by the caller because *which* edge
/// types carry those meanings is content, not arithmetic.
///
/// Protection takes precedence: someone inside both is counted once, as
/// protected (the declared partition — see the module doc).
///
/// # Errors
/// Returns [`GraphError`] if `affected` is empty — a backfire fraction over
/// nobody is not zero, it is undefined, and returning `0.0` would read as
/// "this repression was safe". It also propagates a dangling-id error rather
/// than reading an absent node as unaffiliated, which would silently deflate
/// both channels.
pub fn measure(
    graph: &impl GraphSubstrate,
    affected: &[NodeId],
    protective_edge_type: &str,
    interpreting_edge_type: &str,
) -> Result<Backfire, GraphError> {
    let population: BTreeSet<NodeId> = affected.iter().copied().collect();
    if population.is_empty() {
        return Err(GraphError {
            message: "backfire over an empty affected population is undefined, \
                      not zero — an empty sweep is not a safe one"
                .to_owned(),
        });
    }
    for node in &population {
        if !graph.node_exists(*node) {
            return Err(GraphError {
                message: format!(
                    "no such node: {node:?} — a dangling ref never reads as unaffiliated"
                ),
            });
        }
    }

    let mut protected = 0_usize;
    let mut interpreted = 0_usize;
    for node in &population {
        if stands_inside(graph, *node, protective_edge_type)? {
            protected += 1;
        } else if stands_inside(graph, *node, interpreting_edge_type)? {
            interpreted += 1;
        }
    }

    let total = count_as_f64(population.len())?;
    Ok(Backfire {
        protected_fraction: count_as_f64(protected)? / total,
        interpreted_fraction: count_as_f64(interpreted)? / total,
    })
}

/// Widen a count to `f64` loudly, matching [`crate::exposure`]'s discipline.
fn count_as_f64(count: usize) -> Result<f64, GraphError> {
    u32::try_from(count).map(f64::from).map_err(|_| GraphError {
        message: format!("count {count} exceeds the exactly-representable range"),
    })
}

#[cfg(test)]
mod tests {
    use super::{measure, Backfire};
    use crate::memory::MemoryGraph;
    use crate::substrate::{GraphSubstrate, NodeId};

    /// `size` affected people, plus one rival org and one community body they
    /// may or may not be tied to.
    fn population(size: usize) -> (MemoryGraph, Vec<NodeId>, NodeId, NodeId) {
        let mut graph = MemoryGraph::new();
        let people: Vec<NodeId> = (0..size)
            .map(|_| graph.add_node("social_class").unwrap())
            .collect();
        let rival = graph.add_node("organization").unwrap();
        let commune = graph.add_node("community").unwrap();
        (graph, people, rival, commune)
    }

    #[test]
    fn an_unorganized_population_does_not_backfire() {
        // Santiago de María: nobody stands inside anything, and the
        // repression suppresses. Not a coded exception — an empty count.
        let (graph, people, ..) = population(10);
        let measured = measure(&graph, &people, "protection", "membership").unwrap();
        assert!(measured.protected_fraction.abs() < 1e-12);
        assert!(measured.interpreted_fraction.abs() < 1e-12);
        assert!(measured.total().abs() < 1e-12);
    }

    #[test]
    fn interpretation_alone_backfires() {
        // Wood's catechists: no protective reach at all, but the event is
        // framed for those inside a community structure.
        let (mut graph, people, _rival, commune) = population(10);
        for person in people.iter().take(6) {
            graph.add_edge("membership", commune, *person, 1.0).unwrap();
        }
        let measured = measure(&graph, &people, "protection", "membership").unwrap();
        assert!(measured.protected_fraction.abs() < 1e-12);
        assert!((measured.interpreted_fraction - 0.6).abs() < 1e-12);
        assert!((measured.total() - 0.6).abs() < 1e-12);
    }

    #[test]
    fn protection_alone_backfires() {
        let (mut graph, people, rival, _commune) = population(10);
        for person in people.iter().take(3) {
            graph.add_edge("protection", rival, *person, 1.0).unwrap();
        }
        let measured = measure(&graph, &people, "protection", "membership").unwrap();
        assert!((measured.protected_fraction - 0.3).abs() < 1e-12);
        assert!(measured.interpreted_fraction.abs() < 1e-12);
    }

    #[test]
    fn the_partition_never_double_counts_and_never_exceeds_one() {
        // The declared modelling choice: protection takes precedence. Someone
        // inside BOTH structures is one person, counted once — otherwise the
        // "sum of measures" would stop being a measure and could exceed the
        // population it is a fraction of.
        let (mut graph, people, rival, commune) = population(4);
        for person in &people {
            graph.add_edge("protection", rival, *person, 1.0).unwrap();
            graph.add_edge("membership", commune, *person, 1.0).unwrap();
        }
        let measured = measure(&graph, &people, "protection", "membership").unwrap();
        assert!(
            (measured.protected_fraction - 1.0).abs() < 1e-12,
            "all four are protected"
        );
        assert!(
            measured.interpreted_fraction.abs() < 1e-12,
            "and none is counted a second time as interpreted"
        );
        assert!(
            (measured.total() - 1.0).abs() < 1e-12,
            "a measure, never above 1"
        );
    }

    #[test]
    fn the_two_channels_sum_over_a_partition() {
        // Disjoint halves: 2 protected, 2 interpreted, 1 neither.
        let (mut graph, people, rival, commune) = population(5);
        for person in people.iter().take(2) {
            graph.add_edge("protection", rival, *person, 1.0).unwrap();
        }
        for person in people.iter().skip(2).take(2) {
            graph.add_edge("membership", commune, *person, 1.0).unwrap();
        }
        let measured = measure(&graph, &people, "protection", "membership").unwrap();
        assert!((measured.protected_fraction - 0.4).abs() < 1e-12);
        assert!((measured.interpreted_fraction - 0.4).abs() < 1e-12);
        assert!((measured.total() - 0.8).abs() < 1e-12);
    }

    #[test]
    fn an_empty_sweep_is_undefined_not_safe() {
        // Returning 0.0 here would read as "this repression was safe", which
        // is the opposite of what an empty population means.
        let (graph, ..) = population(3);
        let err = measure(&graph, &[], "protection", "membership").unwrap_err();
        assert!(err.message.contains("undefined"), "{}", err.message);
    }

    #[test]
    fn a_dangling_member_is_loud_never_read_as_unaffiliated() {
        // Silently treating an absent id as "stands inside nothing" would
        // deflate both channels and make repression look safer than it is.
        let (graph, people, ..) = population(3);
        let mut affected = people;
        affected.push(NodeId(999));
        let err = measure(&graph, &affected, "protection", "membership").unwrap_err();
        assert!(
            err.message.contains("never reads as unaffiliated"),
            "{}",
            err.message
        );
    }

    #[test]
    fn the_channels_stay_separable() {
        // They are different social facts; a narrator may speak about either.
        // total() is the dossier's composition, not the only readable form.
        let measured = Backfire {
            protected_fraction: 0.25,
            interpreted_fraction: 0.5,
        };
        assert!((measured.total() - 0.75).abs() < 1e-12);
    }
}
