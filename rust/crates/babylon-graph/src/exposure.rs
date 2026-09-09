//! Structural exposure — what a strike against a structure would YIELD
//! (Lane A brick 3; heat dossier §5.2 helpers 1–2 and §5.3 mode 6).
//!
//! **`X` is derived, never stored.** The dossier's L/K/X split forbids an
//! accumulated "threat" or "heat" score on an organization: Sparrow's
//! conscious-opponent argument means target priority must be recomputed from
//! live structure every time it is asked for, not carried forward from past
//! conduct. Every function here is a pure read over the graph; nothing in
//! this module writes.
//!
//! **The graph you pass is the graph the answer is about.** The state
//! targets its own partial, biased dossier (`L`), never ground truth — so
//! these functions take whatever substrate and node set the caller supplies
//! and are silent on whose view it is. Passing the true graph asks "what
//! WOULD a strike yield"; passing `L` asks "what does the state BELIEVE a
//! strike would yield". Those are different questions with the same
//! arithmetic, and keeping them one function is what stops the engine from
//! quietly granting the state omniscience.
//!
//! **No new mathematics** (ADR172): connectivity, a giant-component
//! fraction, a neighborhood-size signature, and a quotient. Everything is a
//! composition over the §2.6 query surface.
//!
//! **Coordination is undirected here.** A raid severs a tie regardless of
//! which endpoint the edge was authored from, so every traversal below uses
//! [`Direction::Any`]. Directionality remains observable through the query
//! surface for callers that need it; it is simply not what fragmentation
//! means.

use crate::substrate::{Direction, GraphError, GraphSubstrate, NodeId};
use std::collections::BTreeSet;

/// Validate every member of `nodes` once and return the scope as a set.
///
/// Every public entry point below funnels through this, so "a dangling ref
/// never reads empty" holds for the WHOLE scope and not merely for the
/// target — a bogus id in the set would otherwise silently participate in
/// signature comparisons.
fn validated_scope(
    graph: &impl GraphSubstrate,
    nodes: &[NodeId],
) -> Result<BTreeSet<NodeId>, GraphError> {
    let scope: BTreeSet<NodeId> = nodes.iter().copied().collect();
    for node in &scope {
        if !graph.node_exists(*node) {
            return Err(GraphError {
                message: format!("no such node: {node:?} — a dangling ref never reads empty"),
            });
        }
    }
    Ok(scope)
}

/// Widen a count to `f64` loudly. Saturating here would silently distort φ
/// and the targeting quotient on an absurdly large graph; the crate's
/// discipline is to fail instead.
fn count_as_f64(count: usize) -> Result<f64, GraphError> {
    u32::try_from(count).map(f64::from).map_err(|_| GraphError {
        message: format!("count {count} exceeds the exactly-representable range"),
    })
}

/// Connected components of the subgraph induced on `nodes` by `edge_type`,
/// traversed undirected.
///
/// Deterministic shape: each component is ascending by [`NodeId`], and the
/// components themselves are ordered by their smallest member. An isolated
/// node is its own one-member component — it is part of the structure, not
/// absent from it.
///
/// # Errors
/// Returns [`GraphError`] if any member of `nodes` does not exist.
pub fn components(
    graph: &impl GraphSubstrate,
    nodes: &[NodeId],
    edge_type: &str,
) -> Result<Vec<Vec<NodeId>>, GraphError> {
    let scope = validated_scope(graph, nodes)?;
    let mut unvisited = scope.clone();
    let mut found: Vec<Vec<NodeId>> = Vec::new();
    // `unvisited` is a BTreeSet, so the seed is always the smallest
    // remaining id — that, plus the sort below, is what makes the output
    // independent of insertion history.
    while let Some(seed) = unvisited.iter().next().copied() {
        unvisited.remove(&seed);
        let mut component: BTreeSet<NodeId> = BTreeSet::from([seed]);
        let mut frontier: Vec<NodeId> = vec![seed];
        while let Some(current) = frontier.pop() {
            for peer in graph.neighbors(current, edge_type, Direction::Any)? {
                if scope.contains(&peer) && component.insert(peer) {
                    unvisited.remove(&peer);
                    frontier.push(peer);
                }
            }
        }
        found.push(component.into_iter().collect());
    }
    found.sort_unstable();
    Ok(found)
}

/// φ — the fraction of `nodes` inside the largest component.
///
/// The coordination-capacity measure the dossier's removal differential is
/// built from: 1.0 is a fully connected structure, and lower means the
/// structure is already fragmented.
///
/// # Errors
/// Returns [`GraphError`] if `nodes` is empty (the fraction of an empty set
/// is undefined — never a silent 0.0), or if a member does not exist.
pub fn giant_component_fraction(
    graph: &impl GraphSubstrate,
    nodes: &[NodeId],
    edge_type: &str,
) -> Result<f64, GraphError> {
    let scope = validated_scope(graph, nodes)?;
    if scope.is_empty() {
        return Err(GraphError {
            message: "giant-component fraction of an empty node set is undefined".into(),
        });
    }
    let largest = components(graph, nodes, edge_type)?
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(0);
    Ok(count_as_f64(largest)? / count_as_f64(scope.len())?)
}

/// Δφ(v) — the removal differential: how much coordination capacity the
/// structure loses if `target` is removed (Sparrow's Point Strength).
///
/// Removing the last remaining node leaves nothing coordinating; the limit
/// φ(∅) = 0 is taken deliberately and documented rather than erroring, so a
/// one-node structure reports maximal loss instead of a special case the
/// caller has to know about.
///
/// # Errors
/// Returns [`GraphError`] if `target` is not in `nodes`, if `nodes` is
/// empty, or if a member does not exist.
pub fn removal_differential(
    graph: &impl GraphSubstrate,
    nodes: &[NodeId],
    edge_type: &str,
    target: NodeId,
) -> Result<f64, GraphError> {
    if nodes.is_empty() {
        return Err(GraphError {
            message: "removal differential over an empty node set is undefined".into(),
        });
    }
    if !nodes.contains(&target) {
        return Err(GraphError {
            message: format!("{target:?} is not in the node set being evaluated"),
        });
    }
    let before = giant_component_fraction(graph, nodes, edge_type)?;
    let remaining: Vec<NodeId> = nodes.iter().copied().filter(|n| *n != target).collect();
    if remaining.is_empty() {
        return Ok(before);
    }
    let after = giant_component_fraction(graph, &remaining, edge_type)?;
    Ok(before - after)
}

/// The k-hop neighborhood-size signature of `target` within `nodes`:
/// `[|N¹(v)|, |N²(v)|, …, |N^hops(v)|]`, where `N^{i+1}(S) = N(N^i(S))`
/// (Everett & Borgatti's layering, per the dossier).
///
/// Two nodes with equal signatures are structurally interchangeable at this
/// resolution — which is what makes the signature a replaceability measure
/// rather than a popularity one.
///
/// # Errors
/// Returns [`GraphError`] if `target` is not in `nodes`, or a member does
/// not exist. `hops` of 0 yields an empty signature.
pub fn degree_signature(
    graph: &impl GraphSubstrate,
    nodes: &[NodeId],
    edge_type: &str,
    target: NodeId,
    hops: usize,
) -> Result<Vec<usize>, GraphError> {
    let scope = validated_scope(graph, nodes)?;
    if !scope.contains(&target) {
        return Err(GraphError {
            message: format!("{target:?} is not in the node set being evaluated"),
        });
    }
    signature_in_scope(graph, &scope, edge_type, target, hops)
}

/// [`degree_signature`]'s body, over an ALREADY-validated scope — so a
/// class-size sweep validates once instead of once per candidate.
fn signature_in_scope(
    graph: &impl GraphSubstrate,
    scope: &BTreeSet<NodeId>,
    edge_type: &str,
    target: NodeId,
    hops: usize,
) -> Result<Vec<usize>, GraphError> {
    let mut signature = Vec::with_capacity(hops);
    let mut layer: BTreeSet<NodeId> = BTreeSet::from([target]);
    for _ in 0..hops {
        let mut next: BTreeSet<NodeId> = BTreeSet::new();
        for node in &layer {
            for peer in graph.neighbors(*node, edge_type, Direction::Any)? {
                if scope.contains(&peer) {
                    next.insert(peer);
                }
            }
        }
        signature.push(next.len());
        layer = next;
    }
    Ok(signature)
}

/// |signature-class(v)| — how many nodes in `nodes` share `target`'s
/// signature, counting `target` itself. **This is replaceability:** a class
/// size of 1 is an irreplaceable specialist; a large class means the
/// structure carries redundant equivalents.
///
/// # Errors
/// Returns [`GraphError`] if `target` is not in `nodes`, or a member does
/// not exist.
pub fn signature_class_size(
    graph: &impl GraphSubstrate,
    nodes: &[NodeId],
    edge_type: &str,
    target: NodeId,
    hops: usize,
) -> Result<usize, GraphError> {
    let scope = validated_scope(graph, nodes)?;
    if !scope.contains(&target) {
        return Err(GraphError {
            message: format!("{target:?} is not in the node set being evaluated"),
        });
    }
    let subject = signature_in_scope(graph, &scope, edge_type, target, hops)?;
    let mut size = 0;
    for candidate in &scope {
        if signature_in_scope(graph, &scope, edge_type, *candidate, hops)? == subject {
            size += 1;
        }
    }
    Ok(size)
}

/// Sparrow's targeting rule, verbatim: `Δφ(v) / |signature-class(v)|` —
/// removal differential divided by replaceability.
///
/// This is the quantity a repression mode's allocation ranks over. Because
/// it is a quotient rather than a threshold, **the failure of decapitation
/// against distributed organizations is not a coded exception** — a
/// redundant structure divides its own yield down and the state's own
/// allocation rule declines the strike. Nothing anywhere says "if the org is
/// distributed, decapitation fails"; it falls out.
///
/// # Errors
/// Returns [`GraphError`] if `target` is not in `nodes`, `nodes` is empty,
/// or a member does not exist.
pub fn decapitation_value(
    graph: &impl GraphSubstrate,
    nodes: &[NodeId],
    edge_type: &str,
    target: NodeId,
    hops: usize,
) -> Result<f64, GraphError> {
    let differential = removal_differential(graph, nodes, edge_type, target)?;
    let class_size = signature_class_size(graph, nodes, edge_type, target, hops)?;
    Ok(differential / count_as_f64(class_size)?)
}

#[cfg(test)]
mod tests {
    use super::{
        components, decapitation_value, degree_signature, giant_component_fraction,
        removal_differential, signature_class_size,
    };
    use crate::memory::MemoryGraph;
    use crate::substrate::{GraphSubstrate, NodeId};

    /// A star: one hub, `spokes` leaves, all joined to the hub only.
    /// The centralized organization — Sparrow's "messiah" shape.
    fn star(spokes: usize) -> (MemoryGraph, NodeId, Vec<NodeId>) {
        let mut graph = MemoryGraph::new();
        let hub = graph.add_node("cadre").unwrap();
        let mut all = vec![hub];
        for _ in 0..spokes {
            let leaf = graph.add_node("cadre").unwrap();
            graph.add_edge("coordination", hub, leaf, 1.0).unwrap();
            all.push(leaf);
        }
        (graph, hub, all)
    }

    /// A ring: every node holds exactly two ties. The distributed
    /// organization — no node's removal disconnects anything.
    fn ring(size: usize) -> (MemoryGraph, Vec<NodeId>) {
        let mut graph = MemoryGraph::new();
        let all: Vec<NodeId> = (0..size)
            .map(|_| graph.add_node("cadre").unwrap())
            .collect();
        for index in 0..size {
            let next = all[(index + 1) % size];
            graph
                .add_edge("coordination", all[index], next, 1.0)
                .unwrap();
        }
        (graph, all)
    }

    #[test]
    fn components_are_deterministic_and_include_isolates() {
        let mut graph = MemoryGraph::new();
        let a = graph.add_node("cadre").unwrap();
        let b = graph.add_node("cadre").unwrap();
        let isolate = graph.add_node("cadre").unwrap();
        graph.add_edge("coordination", b, a, 1.0).unwrap();
        // declared b->a, asked in a different order: output is by id anyway
        assert_eq!(
            components(&graph, &[isolate, b, a], "coordination").unwrap(),
            vec![vec![a, b], vec![isolate]]
        );
    }

    #[test]
    fn traversal_is_undirected_because_a_raid_ignores_edge_authorship() {
        let mut graph = MemoryGraph::new();
        let a = graph.add_node("cadre").unwrap();
        let b = graph.add_node("cadre").unwrap();
        graph.add_edge("coordination", a, b, 1.0).unwrap();
        // one component, though the only edge points a -> b
        assert_eq!(
            components(&graph, &[a, b], "coordination").unwrap(),
            vec![vec![a, b]]
        );
    }

    #[test]
    fn an_empty_scope_is_loud_never_zero() {
        let graph = MemoryGraph::new();
        assert!(giant_component_fraction(&graph, &[], "coordination").is_err());
    }

    #[test]
    fn giant_fraction_measures_fragmentation() {
        let (graph, _hub, all) = star(3);
        assert!(
            (giant_component_fraction(&graph, &all, "coordination").unwrap() - 1.0).abs() < 1e-12
        );
        // the three leaves alone share no ties: three singleton components
        let leaves = &all[1..];
        let fraction = giant_component_fraction(&graph, leaves, "coordination").unwrap();
        assert!((fraction - 1.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn removing_a_hub_costs_more_than_removing_a_leaf() {
        let (graph, hub, all) = star(3);
        let hub_loss = removal_differential(&graph, &all, "coordination", hub).unwrap();
        let leaf_loss = removal_differential(&graph, &all, "coordination", all[1]).unwrap();
        // hub: 1.0 -> 1/3 ; leaf: 1.0 -> 1.0
        assert!((hub_loss - (1.0 - 1.0 / 3.0)).abs() < 1e-12);
        assert!((leaf_loss - 0.0).abs() < 1e-12);
        assert!(hub_loss > leaf_loss);
    }

    #[test]
    fn removing_the_last_node_reports_the_whole_loss() {
        let mut graph = MemoryGraph::new();
        let only = graph.add_node("cadre").unwrap();
        assert!(
            (removal_differential(&graph, &[only], "coordination", only).unwrap() - 1.0).abs()
                < 1e-12
        );
    }

    #[test]
    fn signature_separates_a_hub_from_its_leaves() {
        let (graph, hub, all) = star(3);
        assert_eq!(
            degree_signature(&graph, &all, "coordination", hub, 2).unwrap(),
            vec![3, 1]
        );
        assert_eq!(
            degree_signature(&graph, &all, "coordination", all[1], 2).unwrap(),
            vec![1, 3]
        );
    }

    #[test]
    fn class_size_is_replaceability() {
        let (graph, hub, all) = star(3);
        // the hub is a specialist: nobody shares its signature
        assert_eq!(
            signature_class_size(&graph, &all, "coordination", hub, 2).unwrap(),
            1
        );
        // each leaf is interchangeable with the other two
        assert_eq!(
            signature_class_size(&graph, &all, "coordination", all[1], 2).unwrap(),
            3
        );
        // a ring is wholly redundant: every node shares one class
        let (ring_graph, ring_nodes) = ring(6);
        assert_eq!(
            signature_class_size(&ring_graph, &ring_nodes, "coordination", ring_nodes[0], 2)
                .unwrap(),
            6
        );
    }

    #[test]
    fn decapitation_fails_against_a_distributed_org_without_a_coded_exception() {
        // THE emergence contract (dossier §5.3 mode 6): nothing in the code
        // says "distributed orgs resist decapitation". The state ranks by
        // Δφ/|class| and a redundant structure divides its own yield away.
        let (star_graph, hub, star_nodes) = star(5);
        let (ring_graph, ring_nodes) = ring(6);

        let centralized =
            decapitation_value(&star_graph, &star_nodes, "coordination", hub, 2).unwrap();
        let distributed =
            decapitation_value(&ring_graph, &ring_nodes, "coordination", ring_nodes[0], 2).unwrap();

        assert!(
            centralized > distributed,
            "centralized {centralized} should outrank distributed {distributed}"
        );
        // the ring yields nothing at all: removing one node of a ring leaves
        // a path — still one component — so Δφ is 0 before the quotient.
        assert!((distributed - 0.0).abs() < 1e-12);
        assert!(centralized > 0.5);
    }

    #[test]
    fn replaceability_divides_the_yield_away() {
        // The ring above proves the emergence claim through Δφ alone. This
        // pins the OTHER half — the quotient — with a structure whose hubs
        // genuinely fragment the org (Δφ = 0.4 each) yet are perfectly
        // interchangeable with each other. A caterpillar: two hubs, two
        // leaves apiece, one bridge between the hubs.
        let mut graph = MemoryGraph::new();
        let h1 = graph.add_node("cadre").unwrap();
        let h2 = graph.add_node("cadre").unwrap();
        let leaves: Vec<NodeId> = (0..4).map(|_| graph.add_node("cadre").unwrap()).collect();
        graph.add_edge("coordination", h1, leaves[0], 1.0).unwrap();
        graph.add_edge("coordination", h1, leaves[1], 1.0).unwrap();
        graph.add_edge("coordination", h2, leaves[2], 1.0).unwrap();
        graph.add_edge("coordination", h2, leaves[3], 1.0).unwrap();
        graph.add_edge("coordination", h1, h2, 1.0).unwrap();
        let all: Vec<NodeId> = [h1, h2].into_iter().chain(leaves).collect();

        let differential = removal_differential(&graph, &all, "coordination", h1).unwrap();
        assert!((differential - 0.4).abs() < 1e-12, "Δφ was {differential}");
        assert_eq!(
            signature_class_size(&graph, &all, "coordination", h1, 2).unwrap(),
            2,
            "the two hubs are structurally interchangeable"
        );
        // …so the strike is worth exactly half what its fragmentation alone
        // would suggest. Remove the quotient and this reads 0.4.
        let value = decapitation_value(&graph, &all, "coordination", h1, 2).unwrap();
        assert!(
            (value - 0.2).abs() < 1e-12,
            "decapitation value was {value}"
        );
        // symmetric by construction: neither hub is the better target
        let sibling = decapitation_value(&graph, &all, "coordination", h2, 2).unwrap();
        assert!((value - sibling).abs() < 1e-12);
    }

    #[test]
    fn removing_dead_weight_scores_negative_and_the_argmax_declines_it() {
        // A declared property, not a wart to discover later: φ is a
        // FRACTION, so removing a node that carries no coordination makes
        // the remainder proportionally MORE connected. Δφ goes negative,
        // which under a rank-order allocation means "this strike is
        // counterproductive" — the state's own rule refuses it without any
        // special case.
        let (mut graph, hub, mut all) = star(2);
        let isolate = graph.add_node("cadre").unwrap();
        all.push(isolate);
        let isolate_value = removal_differential(&graph, &all, "coordination", isolate).unwrap();
        assert!(
            isolate_value < 0.0,
            "expected a negative differential, got {isolate_value}"
        );
        assert!(removal_differential(&graph, &all, "coordination", hub).unwrap() > isolate_value);
    }

    #[test]
    fn a_dangling_or_out_of_scope_target_is_loud() {
        let (graph, _hub, all) = star(2);
        assert!(removal_differential(&graph, &all, "coordination", NodeId(404)).is_err());
        assert!(degree_signature(&graph, &all, "coordination", NodeId(404), 2).is_err());
        assert!(components(&graph, &[NodeId(404)], "coordination").is_err());
    }

    #[test]
    fn a_dangling_non_target_member_is_loud_on_every_entry_point() {
        // The scope is validated WHOLE, not just the target: a bogus id
        // among `nodes` would otherwise silently join the comparison set
        // and skew signatures without ever erroring.
        let (graph, hub, all) = star(2);
        let mut poisoned = all.clone();
        poisoned.push(NodeId(404));
        assert!(components(&graph, &poisoned, "coordination").is_err());
        assert!(giant_component_fraction(&graph, &poisoned, "coordination").is_err());
        assert!(removal_differential(&graph, &poisoned, "coordination", hub).is_err());
        assert!(degree_signature(&graph, &poisoned, "coordination", hub, 2).is_err());
        assert!(signature_class_size(&graph, &poisoned, "coordination", hub, 2).is_err());
        assert!(decapitation_value(&graph, &poisoned, "coordination", hub, 2).is_err());
    }

    #[test]
    fn an_empty_scope_reports_itself_not_a_missing_target() {
        // The diagnostic names the real problem: before, an empty set fell
        // through to "target is not in the node set", which is true but
        // misleading.
        let graph = MemoryGraph::new();
        let error = removal_differential(&graph, &[], "coordination", NodeId(0)).unwrap_err();
        assert!(
            error.message.contains("empty node set"),
            "{}",
            error.message
        );
    }
}
