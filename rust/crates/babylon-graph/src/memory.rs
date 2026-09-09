//! `MemoryGraph`: the in-memory [`GraphSubstrate`] implementation.
//!
//! **No longer the production store** (superseded 2026-08-11, ADR179 T3 /
//! ADR193 — the hypergraph-rs storage swap): `babylon-tick::run_once`
//! constructs [`crate::hypergraph_store::HypergraphStore`] now. This type
//! is kept, not deleted — it is the crate's differential oracle
//! (`babylon-graph/tests/differential.rs`, byte-level, operation-by-
//! operation against `HypergraphStore`) and the reference implementation
//! [`crate::conformance::run_substrate_conformance`] is written against;
//! every invariant that suite checks is held here, and held on purpose.
//! It shipped in Phase 1 as `PlaceholderGraph`, a compile-target whose own
//! documentation said not to build on it, then was promoted (Director
//! ruling 2026-07-31, P27 Phase 2 Slice 1) to the production store it
//! remained until the swap.
//!
//! - **Amendment D shape.** Hyperedges are their own objects with their own
//!   id space; members are a sorted set; nothing expands to `C(n,2)` edges.
//! - **§2.8 existence discipline.** Duplicate add and absent remove are loud
//!   errors, never silent no-ops.
//! - **Honest nulls.** An unwritten attribute errors; it never reads 0.0.
//! - **ADR185 R2 cascade.** Removal is whole — edges, memberships and
//!   attributes go with the node, so a member list is always a set of live
//!   nodes and no internal map holds a key naming a corpse.
//! - **Contractual iteration order.** Every ranged accessor sorts before
//!   returning; storage order is never observable.
//!
//! **What it is not.** It is in-memory and unpersisted: there is no
//! snapshot, no journal, and no recovery. A campaign lives in a process.
//! Persistence is a separate estate and does not belong behind this trait.
//!
//! **The swap executed** (ADR193): `docs/reference/graph-storage-capability-delta.md`
//! enumerated seven capability deltas between `GraphSubstrate` and
//! hypergraph-rs, all but one absorbed behind `HypergraphStore`'s adapter
//! covenants. Depend on the TRAIT — every call site that did stayed
//! unchanged by the swap, which is the entire point of the insulation.
use crate::allocator_state::{AllocatorCursors, AllocatorState};
use crate::state_hash::CanonicalState;
use crate::substrate::{Direction, GraphError, GraphSubstrate, HyperedgeId, NodeId};
use crate::working_copy::DetachedCopy;
use babylon_kernel::Currency;
use std::collections::HashMap;

/// The in-memory substrate. See the module documentation.
#[derive(Debug, Default, Clone)]
pub struct MemoryGraph {
    nodes: HashMap<NodeId, String>,
    attributes: HashMap<(NodeId, String), f64>,
    /// `(node, name)` -> Currency — the SIXTH-section store (T3 #491, OQ-J:
    /// Half 2 of the typed-attribute-seeding design), a map PARALLEL to
    /// `attributes` above, never sharing a key: a `currency`-declared field
    /// lives here and ONLY here, an `attributes`-declared field never here.
    /// A row exists only while its node does — `remove_node`'s cascade
    /// sweeps it exactly as it sweeps `attributes` (ADR185 R2, extended).
    currency_attributes: HashMap<(NodeId, String), Currency>,
    /// `(edge_type, from, to)` -> strength. Real storage, so the §2.8
    /// duplicate-add / absent-remove discipline is checkable.
    edges: HashMap<(String, NodeId, NodeId), f64>,
    /// `(edge_type, from, to, qname)` -> value — the fifth-section store
    /// (T3, ADR198 R1, issue #560): every edge-resident field OTHER than
    /// strength, which lives in `edges` above and hashes in section `0x03`.
    /// A row exists only while its edge does — `remove_edge` and the
    /// `remove_node` cascade both sweep it (ADR185 R2, extended).
    edge_attributes: HashMap<(String, NodeId, NodeId, String), f64>,
    /// `(hyperedge, qname)` -> value — the seventh-section store (Community
    /// port train, Task 5, E2a): a hyperedge's own fields, keyed apart from
    /// its member list. Swept by `remove_hyperedge` (a row exists only
    /// while its hyperedge does — ADR185 R2's cascade discipline).
    hyperedge_attributes: HashMap<(HyperedgeId, String), f64>,
    /// Hyperedge id -> (type, sorted member list). Stored as ONE record per
    /// hyperedge — the toy analogue of a first-class object. A production
    /// store may instead keep incidence edges (Levi); callers cannot tell.
    hyperedges: HashMap<HyperedgeId, (String, Vec<NodeId>)>,
    next_id: u64,
    next_hyperedge_id: u64,
}

impl MemoryGraph {
    /// An empty substrate.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Test-visible edge count (the dyadic half only).
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Test-visible count of stored attribute keys, so the ADR185 R2 cascade
    /// can be checked for orphan rows from outside.
    #[must_use]
    pub fn attribute_key_count(&self) -> usize {
        self.attributes.len()
    }

    /// The Currency-lane analogue of [`Self::attribute_key_count`] (T3 #491,
    /// OQ-J), for the same cascade-orphan check.
    #[must_use]
    pub fn currency_attribute_key_count(&self) -> usize {
        self.currency_attributes.len()
    }
}

impl AllocatorState for MemoryGraph {
    fn allocator_cursors(&self) -> AllocatorCursors {
        AllocatorCursors {
            next_node: self.next_id,
            next_hyperedge: self.next_hyperedge_id,
        }
    }
}

impl DetachedCopy for MemoryGraph {
    fn detached_copy(&self) -> Self {
        self.clone()
    }
}

#[cfg(test)]
impl crate::allocator_state::AllocatorTestControl for MemoryGraph {
    fn set_allocator_cursors_for_test(&mut self, cursors: AllocatorCursors) {
        self.next_id = cursors.next_node;
        self.next_hyperedge_id = cursors.next_hyperedge;
    }
}

impl CanonicalState for MemoryGraph {
    /// The sort moved to [`CanonicalState::encode_state`] — this listing
    /// reports storage order, exactly as `HashMap` iteration gives it.
    fn all_nodes(&self) -> Vec<(NodeId, String)> {
        self.nodes
            .iter()
            .map(|(id, ty)| (*id, ty.clone()))
            .collect()
    }

    fn all_attributes(&self) -> Vec<(NodeId, String, f64)> {
        self.attributes
            .iter()
            .map(|((id, name), value)| (*id, name.clone(), *value))
            .collect()
    }

    fn all_edges(&self) -> Vec<(String, NodeId, NodeId, f64)> {
        self.edges
            .iter()
            .map(|((ty, from, to), strength)| (ty.clone(), *from, *to, *strength))
            .collect()
    }

    fn all_edge_attributes(&self) -> Vec<(String, NodeId, NodeId, String, f64)> {
        self.edge_attributes
            .iter()
            .map(|((ty, from, to, name), value)| (ty.clone(), *from, *to, name.clone(), *value))
            .collect()
    }

    fn all_hyperedges(&self) -> Vec<(HyperedgeId, String, Vec<NodeId>)> {
        self.hyperedges
            .iter()
            .map(|(id, (ty, members))| (*id, ty.clone(), members.clone()))
            .collect()
    }

    fn all_currency_attributes(&self) -> Vec<(NodeId, String, Currency)> {
        self.currency_attributes
            .iter()
            .map(|((id, name), value)| (*id, name.clone(), *value))
            .collect()
    }

    fn all_hyperedge_attributes(&self) -> Vec<(HyperedgeId, String, f64)> {
        self.hyperedge_attributes
            .iter()
            .map(|((id, name), value)| (*id, name.clone(), *value))
            .collect()
    }
}

impl GraphSubstrate for MemoryGraph {
    fn add_node(&mut self, node_type: &str) -> Result<NodeId, GraphError> {
        let next_id = self.next_id.checked_add(1).ok_or_else(|| GraphError {
            message: "node identity allocator exhausted".to_owned(),
        })?;
        let id = NodeId(self.next_id);
        self.nodes.insert(id, node_type.to_owned());
        self.next_id = next_id;
        Ok(id)
    }

    fn remove_node(&mut self, id: NodeId) -> Result<(), GraphError> {
        if self.nodes.remove(&id).is_none() {
            return Err(GraphError {
                message: format!("no such node: {id:?}"),
            });
        }
        // ADR185 R2: removal is WHOLE. Incident edges go, the node is dropped
        // from every member list, and its attributes go with it — so a member
        // list stays a set of live nodes, `members_of` means one thing to
        // every reader, and no internal map holds a key naming a dead node.
        //
        // The attribute sweep is not merely hygiene. `next_id` is monotonic
        // here so ids are never reused, but a production store that recycles
        // an id would resurrect a corpse's attributes onto a fresh node —
        // silently, and reading as real data. The invariant is what makes
        // that class of bug unavailable to the next implementor.
        self.attributes.retain(|(node, _), _| *node != id);
        self.currency_attributes.retain(|(node, _), _| *node != id);
        self.edges
            .retain(|(_, from, to), _| *from != id && *to != id);
        // The cascade sweeps the fifth section too (T3, ADR198 R1): an
        // incident edge's attribute rows go with the edge, exactly as the
        // edge itself goes — no key naming a corpse, in any map.
        self.edge_attributes
            .retain(|(_, from, to, _), _| *from != id && *to != id);
        for (_, members) in self.hyperedges.values_mut() {
            members.retain(|member| *member != id);
        }
        // An empty hyperedge is unrepresentable (`add_hyperedge` rejects an
        // empty member list), so leaving one behind would create by deletion
        // a state that cannot be created directly.
        self.hyperedges
            .retain(|_, (_, members)| !members.is_empty());
        // And their own-field rows die with them (PR #682 harvest — the
        // implicit-drop path missed this before): sweep by liveness, never
        // by assumption about which hyperedges emptied.
        self.hyperedge_attributes
            .retain(|(hyperedge, _), _| self.hyperedges.contains_key(hyperedge));
        Ok(())
    }

    fn add_edge(
        &mut self,
        edge_type: &str,
        from: NodeId,
        to: NodeId,
        strength: f64,
    ) -> Result<(), GraphError> {
        if !self.node_exists(from) || !self.node_exists(to) {
            return Err(GraphError {
                message: "edge endpoint does not exist".into(),
            });
        }
        let key = (edge_type.to_owned(), from, to);
        if self.edges.contains_key(&key) {
            return Err(GraphError {
                message: format!("edge already exists: {key:?}"),
            });
        }
        self.edges.insert(key, strength);
        Ok(())
    }

    fn remove_edge(&mut self, edge_type: &str, from: NodeId, to: NodeId) -> Result<(), GraphError> {
        let key = (edge_type.to_owned(), from, to);
        self.edges
            .remove(&key)
            .map(|_| ())
            .ok_or_else(|| GraphError {
                message: format!("no such edge: {key:?} — absence is never success"),
            })?;
        // ADR185 R2's invariant, extended to the fifth section (T3, ADR198
        // R1): the edge's attribute rows go with it, so a re-minted triple
        // never resurrects its predecessor's fields.
        self.edge_attributes
            .retain(|(ty, f, t, _), _| !(ty == &key.0 && f == &key.1 && t == &key.2));
        Ok(())
    }

    fn update_node(&mut self, id: NodeId, attribute: &str, value: f64) -> Result<(), GraphError> {
        if !self.node_exists(id) {
            return Err(GraphError {
                message: format!("no such node: {id:?}"),
            });
        }
        self.attributes.insert((id, attribute.to_owned()), value);
        Ok(())
    }

    fn update_node_currency(
        &mut self,
        id: NodeId,
        attribute: &str,
        value: Currency,
    ) -> Result<(), GraphError> {
        if !self.node_exists(id) {
            return Err(GraphError {
                message: format!("no such node: {id:?}"),
            });
        }
        self.currency_attributes
            .insert((id, attribute.to_owned()), value);
        Ok(())
    }

    fn node_attribute_currency(&self, id: NodeId, attribute: &str) -> Result<Currency, GraphError> {
        if !self.node_exists(id) {
            return Err(GraphError {
                message: format!("no such node: {id:?}"),
            });
        }
        self.currency_attributes
            .get(&(id, attribute.to_owned()))
            .copied()
            .ok_or_else(|| GraphError {
                message: format!(
                    "currency attribute {attribute} was never written on {id:?} — never a default"
                ),
            })
    }

    fn update_edge(
        &mut self,
        edge_type: &str,
        from: NodeId,
        to: NodeId,
        attribute: &str,
        value: f64,
    ) -> Result<(), GraphError> {
        let key = (edge_type.to_owned(), from, to);
        if !self.edges.contains_key(&key) {
            return Err(GraphError {
                message: format!(
                    "no such edge: {key:?} — a write never mints state for an absent edge"
                ),
            });
        }
        // The strength fork (the double-storage ruling, D143): a `/strength`
        // qname writes the edge's EXISTING 0x03-slot strength — the
        // contains_key above makes this insert a replacement, never a mint —
        // and never a fifth-section row. The owner segment is deliberately
        // NOT checked (the trait doc's division: ownership is the caller's
        // obligation).
        if attribute.ends_with("/strength") {
            self.edges.insert(key, value);
        } else {
            self.edge_attributes
                .insert((key.0, key.1, key.2, attribute.to_owned()), value);
        }
        Ok(())
    }

    fn node_attribute(&self, id: NodeId, attribute: &str) -> Result<f64, GraphError> {
        if !self.node_exists(id) {
            return Err(GraphError {
                message: format!("no such node: {id:?}"),
            });
        }
        self.attributes
            .get(&(id, attribute.to_owned()))
            .copied()
            .ok_or_else(|| GraphError {
                message: format!(
                    "attribute {attribute} was never written on {id:?} — never a default 0.0"
                ),
            })
    }

    fn node_exists(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    fn nodes(&self, node_type: &str) -> Vec<NodeId> {
        let mut found: Vec<NodeId> = self
            .nodes
            .iter()
            .filter(|(_, ty)| *ty == node_type)
            .map(|(id, _)| *id)
            .collect();
        found.sort_unstable();
        found
    }

    fn edges(&self, edge_type: &str) -> Vec<(NodeId, NodeId)> {
        let mut found: Vec<(NodeId, NodeId)> = self
            .edges
            .keys()
            .filter(|(ty, _, _)| ty == edge_type)
            .map(|(_, from, to)| (*from, *to))
            .collect();
        found.sort_unstable();
        found
    }

    fn edge_attribute(
        &self,
        edge_type: &str,
        from: NodeId,
        to: NodeId,
        attribute: &str,
    ) -> Result<f64, GraphError> {
        let key = (edge_type.to_owned(), from, to);
        // Suffix routing (T3, ADR198 R1 — the trait doc's contract): strength
        // reads the 0x03 slot; anything else reads the fifth-section store.
        // The owner segment is deliberately NOT checked, here exactly as in
        // update_edge — ownership is the CALLER's obligation (pinned by the
        // conformance row edge_attribute_does_not_check_the_owner_segment).
        if attribute.ends_with("/strength") {
            return self.edges.get(&key).copied().ok_or_else(|| GraphError {
                message: format!(
                    "no such edge: ({edge_type}, {from:?}, {to:?}) — never a default 0.0"
                ),
            });
        }
        if !self.edges.contains_key(&key) {
            return Err(GraphError {
                message: format!(
                    "no such edge: ({edge_type}, {from:?}, {to:?}) — never a default 0.0"
                ),
            });
        }
        self.edge_attributes
            .get(&(key.0, key.1, key.2, attribute.to_owned()))
            .copied()
            .ok_or_else(|| GraphError {
                message: format!(
                    "edge attribute '{attribute}' was never written on ({edge_type}, {from:?}, \
                     {to:?}) — never a default 0.0"
                ),
            })
    }

    fn neighbors(
        &self,
        node: NodeId,
        edge_type: &str,
        direction: Direction,
    ) -> Result<Vec<NodeId>, GraphError> {
        if !self.node_exists(node) {
            return Err(GraphError {
                message: format!("no such node: {node:?} — a dangling ref never reads empty"),
            });
        }
        let mut found: Vec<NodeId> = self
            .edges
            .keys()
            .filter(|(ty, _, _)| ty == edge_type)
            .filter_map(|(_, from, to)| match direction {
                Direction::Out => (*from == node).then_some(*to),
                Direction::In => (*to == node).then_some(*from),
                Direction::Any if *from == node => Some(*to),
                Direction::Any if *to == node => Some(*from),
                Direction::Any => None,
            })
            .collect();
        found.sort_unstable();
        found.dedup();
        Ok(found)
    }

    fn add_hyperedge(
        &mut self,
        hyperedge_type: &str,
        members: &[NodeId],
    ) -> Result<HyperedgeId, GraphError> {
        if members.is_empty() {
            return Err(GraphError {
                message: "hyperedge must have at least one member".into(),
            });
        }
        let mut sorted: Vec<NodeId> = members.to_vec();
        sorted.sort_unstable();
        if sorted.windows(2).any(|w| w[0] == w[1]) {
            return Err(GraphError {
                message: "duplicate member in hyperedge".into(),
            });
        }
        if let Some(missing) = sorted.iter().find(|n| !self.node_exists(**n)) {
            return Err(GraphError {
                message: format!("no such member node: {missing:?}"),
            });
        }
        let next_id = self
            .next_hyperedge_id
            .checked_add(1)
            .ok_or_else(|| GraphError {
                message: "hyperedge identity allocator exhausted".to_owned(),
            })?;
        let id = HyperedgeId(self.next_hyperedge_id);
        self.hyperedges
            .insert(id, (hyperedge_type.to_owned(), sorted));
        self.next_hyperedge_id = next_id;
        Ok(id)
    }

    fn remove_hyperedge(&mut self, id: HyperedgeId) -> Result<(), GraphError> {
        self.hyperedges
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| GraphError {
                message: format!("no such hyperedge: {id:?}"),
            })?;
        // ADR185 R2, extended to the own-field lane (Task 5): a hyperedge's
        // attribute rows die with it, never orphaned.
        self.hyperedge_attributes
            .retain(|(hyperedge, _), _| *hyperedge != id);
        Ok(())
    }

    fn update_hyperedge_attribute(
        &mut self,
        id: HyperedgeId,
        attribute: &str,
        value: f64,
    ) -> Result<(), GraphError> {
        if !value.is_finite() {
            return Err(GraphError {
                message: format!("non-finite hyperedge attribute {attribute} on {id:?}"),
            });
        }
        if !self.hyperedges.contains_key(&id) {
            return Err(GraphError {
                message: format!(
                    "no such hyperedge: {id:?} — a write never mints state for an absent hyperedge"
                ),
            });
        }
        self.hyperedge_attributes
            .insert((id, attribute.to_owned()), value);
        Ok(())
    }

    fn hyperedge_attribute(&self, id: HyperedgeId, attribute: &str) -> Result<f64, GraphError> {
        if !self.hyperedges.contains_key(&id) {
            return Err(GraphError {
                message: format!("no such hyperedge: {id:?}"),
            });
        }
        self.hyperedge_attributes
            .get(&(id, attribute.to_owned()))
            .copied()
            .ok_or_else(|| GraphError {
                message: format!(
                    "attribute {attribute} was never written on {id:?} — never a default 0.0"
                ),
            })
    }

    fn members_of(&self, id: HyperedgeId) -> Result<Vec<NodeId>, GraphError> {
        self.hyperedges
            .get(&id)
            .map(|(_, members)| members.clone()) // already sorted at insert
            .ok_or_else(|| GraphError {
                message: format!("no such hyperedge: {id:?}"),
            })
    }

    fn hyperedges_of(
        &self,
        node: NodeId,
        hyperedge_type: &str,
    ) -> Result<Vec<HyperedgeId>, GraphError> {
        if !self.nodes.contains_key(&node) {
            return Err(GraphError {
                message: format!(
                    "no such node: {node:?} — belonging to nothing and not existing \
                     are different facts"
                ),
            });
        }
        let mut found: Vec<HyperedgeId> = self
            .hyperedges
            .iter()
            .filter(|(_, (ty, members))| ty == hyperedge_type && members.contains(&node))
            .map(|(id, _)| *id)
            .collect();
        found.sort_unstable();
        Ok(found)
    }

    fn hyperedges(&self, hyperedge_type: &str) -> Vec<HyperedgeId> {
        let mut found: Vec<HyperedgeId> = self
            .hyperedges
            .iter()
            .filter(|(_, (ty, _))| ty == hyperedge_type)
            .map(|(id, _)| *id)
            .collect();
        found.sort_unstable();
        found
    }

    fn node_type_of(&self, id: NodeId) -> Result<&str, GraphError> {
        self.nodes
            .get(&id)
            .map(String::as_str)
            .ok_or_else(|| GraphError {
                message: format!("no such node: {id:?} — a dangling ref never reads as untyped"),
            })
    }

    fn hyperedge_type_of(&self, id: HyperedgeId) -> Result<&str, GraphError> {
        self.hyperedges
            .get(&id)
            .map(|(ty, _)| ty.as_str())
            .ok_or_else(|| GraphError {
                message: format!(
                    "no such hyperedge: {id:?} — a dangling ref never reads as untyped"
                ),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{CanonicalState, GraphSubstrate, MemoryGraph, NodeId};
    use crate::allocator_state::{AllocatorCursors, AllocatorState};
    use crate::conformance::{
        run_allocator_exhaustion_conformance, run_detached_working_copy_conformance,
    };
    use crate::substrate::Direction;

    #[test]
    fn removing_a_node_cascades_to_its_edges_and_memberships() {
        // ADR185 R2. Before this ruling remove_node dropped the node and
        // nothing else, leaving `edges` holding dead endpoints and member
        // lists holding dead ids — so `edges()` returned endpoints that
        // failed `node_exists`, and `members_of` stopped being a set of live
        // nodes. Removal is whole.
        let mut graph = MemoryGraph::new();
        let doomed = graph.add_node("social_class").unwrap();
        let survivor = graph.add_node("social_class").unwrap();
        let bystander = graph.add_node("social_class").unwrap();
        graph.add_edge("solidarity", doomed, survivor, 1.0).unwrap();
        graph
            .add_edge("solidarity", bystander, doomed, 1.0)
            .unwrap();
        graph
            .add_edge("solidarity", survivor, bystander, 1.0)
            .unwrap();
        let sector = graph
            .add_hyperedge("economic_sector", &[doomed, survivor, bystander])
            .unwrap();

        graph.remove_node(doomed).unwrap();

        assert_eq!(
            graph.edges("solidarity"),
            vec![(survivor, bystander)],
            "both edges touching the removed node are gone; the third stands"
        );
        for (from, to) in graph.edges("solidarity") {
            assert!(graph.node_exists(from) && graph.node_exists(to));
        }
        assert_eq!(
            graph.members_of(sector).unwrap(),
            vec![survivor, bystander],
            "a member list is a set of LIVE nodes"
        );
        assert!(graph
            .neighbors(doomed, "solidarity", Direction::Any)
            .is_err());
    }

    #[test]
    fn detached_working_copy_owns_every_mutable_lane() {
        run_detached_working_copy_conformance(MemoryGraph::new);
    }

    #[test]
    fn exhausted_allocators_leave_memory_graph_unchanged() {
        run_allocator_exhaustion_conformance(MemoryGraph::new);
    }

    #[test]
    fn allocator_cursors_report_the_next_ids_even_after_removal() {
        let mut graph = MemoryGraph::new();
        let first = graph.add_node("social_class").unwrap();
        let second = graph.add_node("social_class").unwrap();
        graph.add_hyperedge("class", &[first, second]).unwrap();
        graph.remove_node(second).unwrap();

        assert_eq!(
            graph.allocator_cursors(),
            AllocatorCursors {
                next_node: 2,
                next_hyperedge: 1,
            }
        );
    }

    /// Build the same world twice, inserting in opposite orders.
    fn same_world_two_orders() -> (MemoryGraph, MemoryGraph) {
        let mut forward = MemoryGraph::new();
        let a = forward.add_node("social_class").unwrap();
        let b = forward.add_node("social_class").unwrap();
        forward.update_node(a, "wages", 120.0).unwrap();
        forward.update_node(a, "value_produced", 80.0).unwrap();
        forward.update_node(b, "wages", 40.0).unwrap();
        forward.add_edge("solidarity", a, b, 0.5).unwrap();
        forward.add_hyperedge("economic_sector", &[a, b]).unwrap();

        let mut reverse = MemoryGraph::new();
        let a2 = reverse.add_node("social_class").unwrap();
        let b2 = reverse.add_node("social_class").unwrap();
        // Same facts, opposite write order.
        reverse.add_hyperedge("economic_sector", &[b2, a2]).unwrap();
        reverse.add_edge("solidarity", a2, b2, 0.5).unwrap();
        reverse.update_node(b2, "wages", 40.0).unwrap();
        reverse.update_node(a2, "value_produced", 80.0).unwrap();
        reverse.update_node(a2, "wages", 120.0).unwrap();

        (forward, reverse)
    }

    #[test]
    fn the_state_hash_does_not_depend_on_insertion_order() {
        // THE determinism contract for this store (Constitution III.7). It is
        // `HashMap`-backed, so its iteration order varies per process; if the
        // encoding did not sort, the same world would hash differently on
        // every run and replay would be impossible.
        let (forward, reverse) = same_world_two_orders();
        assert_eq!(
            forward.state_hash().unwrap(),
            reverse.state_hash().unwrap(),
            "the same world must hash identically however it was built"
        );
    }

    #[test]
    fn the_state_hash_is_stable_across_repeated_encodings() {
        // Guards against a hash that varies within one process too — the
        // failure a single equality check between two graphs would miss.
        let (graph, _) = same_world_two_orders();
        let first = graph.state_hash().unwrap();
        for _ in 0..8 {
            assert_eq!(graph.state_hash().unwrap(), first);
        }
    }

    #[test]
    fn any_real_change_moves_the_state_hash() {
        // The dual of determinism: a hash nothing changes is worthless.
        let (mut graph, _) = same_world_two_orders();
        let before = graph.state_hash().unwrap();

        graph.update_node(NodeId(0), "wages", 121.0).unwrap();
        let after_attribute = graph.state_hash().unwrap();
        assert_ne!(before, after_attribute, "an attribute write moves it");

        graph
            .remove_edge("solidarity", NodeId(0), NodeId(1))
            .unwrap();
        let after_edge = graph.state_hash().unwrap();
        assert_ne!(after_attribute, after_edge, "an edge removal moves it");

        graph.add_node("organization").unwrap();
        assert_ne!(
            after_edge,
            graph.state_hash().unwrap(),
            "a new node moves it"
        );
    }

    #[test]
    fn removal_takes_the_nodes_attributes_with_it() {
        // No internal map may hold a key naming a dead node. `next_id` is
        // monotonic here so ids are never reused — but a production store
        // that recycles one would resurrect a corpse's attributes onto a
        // fresh node, silently, reading as real data.
        let mut graph = MemoryGraph::new();
        let doomed = graph.add_node("social_class").unwrap();
        let survivor = graph.add_node("social_class").unwrap();
        graph.update_node(doomed, "wealth", 42.0).unwrap();
        graph.update_node(survivor, "wealth", 7.0).unwrap();

        graph.remove_node(doomed).unwrap();

        assert_eq!(
            graph.attribute_key_count(),
            1,
            "the removed node's attribute rows are gone, not orphaned"
        );
        assert!(
            (graph.node_attribute(survivor, "wealth").unwrap() - 7.0).abs() < 1e-12,
            "and the survivor's are untouched"
        );
    }

    #[test]
    fn a_hyperedge_losing_its_last_member_is_removed_not_emptied() {
        // An empty hyperedge is unrepresentable — add_hyperedge rejects an
        // empty member list — so leaving one behind would create BY DELETION
        // a state that cannot be created directly.
        let mut graph = MemoryGraph::new();
        let only = graph.add_node("social_class").unwrap();
        let sector = graph.add_hyperedge("economic_sector", &[only]).unwrap();

        graph.remove_node(only).unwrap();

        let err = graph.members_of(sector).unwrap_err();
        assert!(err.message.contains("no such hyperedge"), "{}", err.message);
    }

    #[test]
    fn belonging_to_nothing_and_not_existing_are_different_facts() {
        // The honest-null discipline at the hyperedge half. An unknown TYPE
        // is an empty range (type validity is BSL's E-TYPE-011, not the
        // substrate's); an unknown NODE is loud.
        //
        // Why the asymmetry is load-bearing rather than pedantic: a target
        // that belongs to no protective structure is the CHEAPEST one an
        // adversary can reach. If a missing node read as "belongs to
        // nothing", every data hole would present as a defenceless target.
        // Pine Ridge (FIPS 46102) has zero census rows at every vintage, so
        // that is a real shape in the data, not a hypothetical.
        let mut graph = MemoryGraph::new();
        let lone = graph.add_node("social_class").unwrap();

        assert_eq!(
            graph.hyperedges_of(lone, "community").unwrap(),
            vec![],
            "a real node in no community of that type is an empty range"
        );
        assert_eq!(
            graph.hyperedges_of(lone, "no_such_type").unwrap(),
            vec![],
            "an unknown TYPE is empty, not an error — E-TYPE-011 is BSL's job"
        );

        let err = graph.hyperedges_of(NodeId(999), "community").unwrap_err();
        assert!(
            err.message.contains("different facts"),
            "an absent NODE must be loud: {}",
            err.message
        );
    }

    #[test]
    fn add_then_update_then_read_back() {
        let mut graph = MemoryGraph::new();
        let node = graph.add_node("social_class").unwrap();
        graph.update_node(node, "wealth", 42.0).unwrap();
        assert_eq!(graph.node_attribute(node, "wealth"), Ok(42.0));
    }

    #[test]
    fn an_unwritten_attribute_reads_loud_never_zero() {
        let mut graph = MemoryGraph::new();
        let node = graph.add_node("social_class").unwrap();
        assert!(graph.node_attribute(node, "wealth").is_err());
    }

    #[test]
    fn add_then_update_currency_then_read_back() {
        // T3 #491, OQ-J: the i128 lane round-trips exactly, PARALLEL to the
        // f64 lane above — the same field name in both maps would be two
        // different rows, but this test only exercises one lane.
        use babylon_kernel::Currency;
        let mut graph = MemoryGraph::new();
        let node = graph.add_node("social_class").unwrap();
        graph
            .update_node_currency(node, "wages", Currency::from_micro_units(120_000_000))
            .unwrap();
        assert_eq!(
            graph.node_attribute_currency(node, "wages"),
            Ok(Currency::from_micro_units(120_000_000))
        );
    }

    #[test]
    fn an_unwritten_currency_attribute_reads_loud_never_zero() {
        let mut graph = MemoryGraph::new();
        let node = graph.add_node("social_class").unwrap();
        assert!(graph.node_attribute_currency(node, "wages").is_err());
    }

    #[test]
    fn removal_takes_the_nodes_currency_attributes_with_it() {
        // ADR185 R2's cascade, extended to the sixth-section map: no
        // internal map may hold a key naming a dead node, the SAME
        // invariant `removal_takes_the_nodes_attributes_with_it` proves for
        // the f64 lane.
        use babylon_kernel::Currency;
        let mut graph = MemoryGraph::new();
        let doomed = graph.add_node("social_class").unwrap();
        let survivor = graph.add_node("social_class").unwrap();
        graph
            .update_node_currency(doomed, "wages", Currency::from_micro_units(1))
            .unwrap();
        graph
            .update_node_currency(survivor, "wages", Currency::from_micro_units(2))
            .unwrap();

        graph.remove_node(doomed).unwrap();

        assert_eq!(
            graph.currency_attribute_key_count(),
            1,
            "the removed node's currency-attribute row is gone, not orphaned"
        );
        assert_eq!(
            graph.node_attribute_currency(survivor, "wages"),
            Ok(Currency::from_micro_units(2)),
            "and the survivor's is untouched"
        );
    }

    #[test]
    fn edge_to_nonexistent_node_is_a_loud_error() {
        let mut graph = MemoryGraph::new();
        let node = graph.add_node("territory").unwrap();
        assert!(graph
            .add_edge("adjacency", node, NodeId(9999), 1.0)
            .is_err());
    }

    #[test]
    fn duplicate_edge_add_and_absent_edge_remove_are_loud() {
        // §2.8: absence is never success, and adding what exists is an
        // error, not an overwrite.
        let mut graph = MemoryGraph::new();
        let a = graph.add_node("social_class").unwrap();
        let b = graph.add_node("social_class").unwrap();
        graph.add_edge("solidarity", a, b, 0.5).unwrap();
        assert!(graph.add_edge("solidarity", a, b, 0.9).is_err());
        graph.remove_edge("solidarity", a, b).unwrap();
        assert!(graph.remove_edge("solidarity", a, b).is_err());
    }

    #[test]
    fn hyperedge_members_come_back_sorted_not_in_declared_order() {
        // Amendment D / BSL D25: declared member order is unobservable.
        let mut graph = MemoryGraph::new();
        let first = graph.add_node("social_class").unwrap();
        let second = graph.add_node("social_class").unwrap();
        let third = graph.add_node("social_class").unwrap();
        let sector = graph
            .add_hyperedge("economic_sector", &[third, first, second])
            .unwrap();
        assert_eq!(
            graph.members_of(sector).unwrap(),
            vec![first, second, third]
        );
    }

    #[test]
    fn duplicate_member_is_a_loud_error() {
        let mut graph = MemoryGraph::new();
        let member = graph.add_node("social_class").unwrap();
        assert!(graph
            .add_hyperedge("economic_sector", &[member, member])
            .is_err());
    }

    #[test]
    fn nodes_query_ranges_over_one_type_in_ascending_id_order() {
        // §2.6 `(nodes <enum-ref>)` + the iteration-order ruling: ascending
        // node-id order, never storage order.
        //
        // babylon-bsl's query::materialize (`nodes_materializes_in_
        // ascending_id_order`) rides THIS test's guarantee directly — it
        // forwards `nodes()`'s result with no sort step of its own, so it
        // has nothing local to mutate. Fifty `social_class` nodes here (up
        // from the original three) is what makes the guard real: `next_id`
        // is monotonic and never reused (`remove_node`'s own doc), so
        // insertion call order and ascending-id order can never diverge for
        // THIS store regardless of element count — what a small fixture
        // does NOT rule out is `nodes()` merely echoing the underlying
        // `HashMap`'s (unspecified) iteration order instead of actually
        // running `sort_unstable()`. Fifty elements makes an accidental
        // sorted match astronomically unlikely, so removing the sort would
        // fail this with overwhelming probability where three elements
        // would not.
        let mut graph = MemoryGraph::new();
        let mut class_ids = Vec::with_capacity(50);
        for _ in 0..50 {
            class_ids.push(graph.add_node("social_class").unwrap());
        }
        let territory = graph.add_node("territory").unwrap();
        let result = graph.nodes("social_class");
        assert_eq!(result, class_ids);
        assert!(
            result.windows(2).all(|w| w[0] < w[1]),
            "must be strictly ascending by id: {result:?}"
        );
        assert_eq!(graph.nodes("territory"), vec![territory]);
        assert_eq!(graph.nodes("organization"), Vec::<NodeId>::new());
    }

    #[test]
    fn edges_query_ranges_over_one_type_in_source_target_order() {
        // §2.6 `(edges <enum-ref>)`: ascending (source-id, target-id) order.
        let mut graph = MemoryGraph::new();
        let a = graph.add_node("social_class").unwrap();
        let b = graph.add_node("social_class").unwrap();
        let c = graph.add_node("social_class").unwrap();
        graph.add_edge("solidarity", b, a, 0.4).unwrap();
        graph.add_edge("solidarity", a, c, 0.6).unwrap();
        graph.add_edge("wages", c, a, 0.9).unwrap();
        assert_eq!(graph.edges("solidarity"), vec![(a, c), (b, a)]);
        assert_eq!(graph.edges("wages"), vec![(c, a)]);
        assert_eq!(graph.edges("tribute"), Vec::<(NodeId, NodeId)>::new());
    }

    #[test]
    fn neighbors_query_honors_direction_and_dedups_as_a_set() {
        // §2.6 `(neighbors <expr> <enum-ref> <direction>)`: :out follows
        // source->target, :in the reverse, :any their union — a NodeSet,
        // so a node reachable both ways appears once.
        let mut graph = MemoryGraph::new();
        let org = graph.add_node("organization").unwrap();
        let class_a = graph.add_node("social_class").unwrap();
        let class_b = graph.add_node("social_class").unwrap();
        graph.add_edge("membership", org, class_a, 1.0).unwrap();
        graph.add_edge("membership", org, class_b, 1.0).unwrap();
        graph.add_edge("membership", class_b, org, 0.2).unwrap();
        assert_eq!(
            graph.neighbors(org, "membership", Direction::Out).unwrap(),
            vec![class_a, class_b]
        );
        assert_eq!(
            graph.neighbors(org, "membership", Direction::In).unwrap(),
            vec![class_b]
        );
        assert_eq!(
            graph.neighbors(org, "membership", Direction::Any).unwrap(),
            vec![class_a, class_b]
        );
        // other edge types are invisible to this query
        assert_eq!(
            graph.neighbors(org, "solidarity", Direction::Any).unwrap(),
            Vec::<NodeId>::new()
        );
    }

    #[test]
    fn neighbors_of_a_nonexistent_node_is_a_loud_error() {
        // A dangling NodeRef must never read as an empty neighborhood.
        let graph = MemoryGraph::new();
        assert!(graph
            .neighbors(NodeId(9999), "membership", Direction::Any)
            .is_err());
    }

    #[test]
    fn co_projection_composes_from_neighbors_alone() {
        // The dossier §5.2 org<->org inducement (shared class base via
        // MEMBERSHIP) expressed as pure composition over the query surface —
        // the shape the heat system's targeting reads. Two orgs organizing
        // the same class are induced-adjacent; a third org organizing a
        // different class is not.
        let mut graph = MemoryGraph::new();
        let org_a = graph.add_node("organization").unwrap();
        let org_b = graph.add_node("organization").unwrap();
        let org_c = graph.add_node("organization").unwrap();
        let shared_class = graph.add_node("social_class").unwrap();
        let other_class = graph.add_node("social_class").unwrap();
        graph
            .add_edge("membership", org_a, shared_class, 1.0)
            .unwrap();
        graph
            .add_edge("membership", org_b, shared_class, 1.0)
            .unwrap();
        graph
            .add_edge("membership", org_c, other_class, 1.0)
            .unwrap();

        let induced: Vec<NodeId> = graph
            .neighbors(org_a, "membership", Direction::Out)
            .unwrap()
            .into_iter()
            .flat_map(|base| graph.neighbors(base, "membership", Direction::In).unwrap())
            .filter(|peer| *peer != org_a)
            .collect();
        assert_eq!(induced, vec![org_b]);
    }

    #[test]
    fn a_hyperedge_mints_no_pairwise_edges() {
        // VIII.9 by construction: n members cost one object, not C(n,2) edges.
        let mut graph = MemoryGraph::new();
        let first = graph.add_node("social_class").unwrap();
        let second = graph.add_node("social_class").unwrap();
        let sector = graph
            .add_hyperedge("economic_sector", &[first, second])
            .unwrap();
        assert_eq!(
            graph.hyperedges_of(first, "economic_sector").unwrap(),
            vec![sector]
        );
        // the dyadic half is untouched by minting a hyperedge
        assert_eq!(graph.edge_count(), 0);
        assert_eq!(graph.hyperedges.len(), 1);
    }
}
