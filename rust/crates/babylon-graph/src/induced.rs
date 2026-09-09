//! Induced co-projection adjacency (Lane A brick 2; heat dossier §5.2).
//!
//! Two organizations that organize the same class or hold presence in the
//! same territory share an exposure — Serge's penetrability argument and
//! Kalyvas's "chances that were largely geographical" in one construction.
//! The state's targeting practice (Sparrow) needs an org↔org adjacency, and
//! no production writer creates org↔org SOLIDARITY edges; the materially
//! correct adjacency is *induced* from existing base edges
//! (`org --MEMBERSHIP--> social_class`, `org --PRESENCE--> territory`)
//! without minting any new edge type.
//!
//! This is pure composition over the §2.6 query surface — no new
//! mathematics (ADR172: BSL/engine constructs mint none). The shared-base
//! *count* is returned because it is a measure the caller may consume
//! (co-exposure mass), never a chosen curve; ignoring it recovers the plain
//! adjacency.

use crate::substrate::{Direction, GraphError, GraphSubstrate, NodeId};
use std::collections::BTreeMap;

/// The peers of `node` under the co-projection through `base_edge_type`:
/// every other node that shares at least one out-neighbor with `node`
/// across that edge type, in ascending [`NodeId`] order, each with the
/// count of shared bases.
///
/// # Errors
/// Returns [`GraphError`] if `node` does not exist (the same dangling-ref
/// loudness as [`GraphSubstrate::neighbors`]).
pub fn co_projected_peers(
    graph: &impl GraphSubstrate,
    node: NodeId,
    base_edge_type: &str,
) -> Result<Vec<(NodeId, usize)>, GraphError> {
    let bases = graph.neighbors(node, base_edge_type, Direction::Out)?;
    // BTreeMap: O(log p) increments and ascending-NodeId iteration for free.
    let mut counts: BTreeMap<NodeId, usize> = BTreeMap::new();
    for base in bases {
        for peer in graph.neighbors(base, base_edge_type, Direction::In)? {
            if peer != node {
                *counts.entry(peer).or_insert(0) += 1;
            }
        }
    }
    Ok(counts.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::co_projected_peers;
    use crate::memory::MemoryGraph;
    use crate::substrate::{GraphSubstrate, NodeId};

    #[test]
    fn shared_class_base_induces_adjacency_with_multiplicity() {
        // dossier §5.2: org_a and org_b organize the same two classes
        // (count 2); org_c shares one territory-shaped base under a
        // DIFFERENT edge type and must be invisible to this projection.
        let mut graph = MemoryGraph::new();
        let org_a = graph.add_node("organization").unwrap();
        let org_b = graph.add_node("organization").unwrap();
        let org_c = graph.add_node("organization").unwrap();
        let class_x = graph.add_node("social_class").unwrap();
        let class_y = graph.add_node("social_class").unwrap();
        let territory = graph.add_node("territory").unwrap();
        graph.add_edge("membership", org_a, class_x, 1.0).unwrap();
        graph.add_edge("membership", org_a, class_y, 1.0).unwrap();
        graph.add_edge("membership", org_b, class_x, 1.0).unwrap();
        graph.add_edge("membership", org_b, class_y, 1.0).unwrap();
        graph.add_edge("presence", org_a, territory, 1.0).unwrap();
        graph.add_edge("presence", org_c, territory, 1.0).unwrap();

        assert_eq!(
            co_projected_peers(&graph, org_a, "membership").unwrap(),
            vec![(org_b, 2)]
        );
        assert_eq!(
            co_projected_peers(&graph, org_a, "presence").unwrap(),
            vec![(org_c, 1)]
        );
        // org_b holds no presence edges: an empty projection, not an error.
        assert_eq!(
            co_projected_peers(&graph, org_b, "presence").unwrap(),
            vec![]
        );
    }

    #[test]
    fn peers_come_back_in_ascending_id_order() {
        let mut graph = MemoryGraph::new();
        // peer `b` shares MORE bases than peer `a` but has the higher id —
        // the result must order by id, never by count.
        let subject = graph.add_node("organization").unwrap();
        let a = graph.add_node("organization").unwrap();
        let b = graph.add_node("organization").unwrap();
        let class_x = graph.add_node("social_class").unwrap();
        let class_y = graph.add_node("social_class").unwrap();
        graph.add_edge("membership", subject, class_x, 1.0).unwrap();
        graph.add_edge("membership", subject, class_y, 1.0).unwrap();
        graph.add_edge("membership", a, class_x, 1.0).unwrap();
        graph.add_edge("membership", b, class_x, 1.0).unwrap();
        graph.add_edge("membership", b, class_y, 1.0).unwrap();

        assert_eq!(
            co_projected_peers(&graph, subject, "membership").unwrap(),
            vec![(a, 1), (b, 2)]
        );
    }

    #[test]
    fn a_dangling_ref_is_a_loud_error() {
        let graph = MemoryGraph::new();
        assert!(co_projected_peers(&graph, NodeId(404), "membership").is_err());
    }
}
