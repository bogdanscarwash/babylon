//! `HypergraphStore`: the [`GraphSubstrate`] adapter over
//! `hypergraph-rs` (ADR179 T3) — the storage-swap plan's Phase C.
//!
//! **Two data structures, not one** (delta document §8 covenant 7,
//! `docs/reference/graph-storage-capability-delta.md`). The dyadic half
//! (`nodes`, `attributes`, `edges`, and — since T3, ADR198 R1 —
//! `edge_attributes`) is native Rust maps, identical in shape to
//! [`crate::memory::MemoryGraph`]'s. The hyperedge half delegates to one
//! `hypergraph_rs::Hypergraph<(), String, MembershipPayload>` — the
//! Levi/incidence encoding Amendment D permits as an INTERNAL storage
//! strategy and forbids exposing (D-1). Splitting the halves into separate
//! structures makes Anti-Pattern VIII.9 (no method may expand a hyperedge
//! into pairwise edges) a structural fact rather than a covenant to police:
//! `neighbors()` reads only `edges`, so there is no code path through which
//! a many-member hyperedge could read as a pairwise expansion.
//!
//! **The identity map.** `NodeId`/`HyperedgeId` mint exactly as
//! `MemoryGraph` mints them — monotonic `u64` counters, never a function of
//! declared order (open question 2, ADR191 R2: identity stays a mint
//! counter). The library's own ids are `String`; this adapter's key is the
//! 16-character lowercase zero-padded hex of the id's big-endian `u64`
//! (`node_key`/`hyperedge_key` below), which makes byte-lexicographic and
//! numeric order the same order (delta §4 CD7's recommended resolution,
//! adopted here as a mechanical property rather than a spec claim). Reverse
//! maps (`node_keys`, `hyperedge_keys`) resolve library results back to
//! typed ids without re-parsing hex.
//!
//! **Node universes coincide by construction** (delta §8 covenant 4):
//! [`GraphSubstrate::add_node`] mints into the library's `add_node` in the
//! same call that mints into this adapter's own `nodes` map, so the
//! existence check `add_hyperedge`'s preamble runs against `self.nodes`
//! actually proves something about the library's universe too — the
//! library's silent auto-create of an unknown member never becomes
//! reachable, because no member this adapter would validate can be unknown
//! to the library.

use crate::allocator_state::{AllocatorCursors, AllocatorState};
use crate::stable_element::{StableElementResolverV1, StableIdentityError};
use crate::stable_state::{encode_stable_graph_state_v1, StableGraphStateV1};
use crate::state_hash::CanonicalState;
use crate::substrate::{Direction, GraphError, GraphSubstrate, HyperedgeId, NodeId};
use crate::working_copy::DetachedCopy;
use babylon_kernel::Currency;
use hypergraph_rs::Hypergraph;
use std::collections::HashMap;

/// The 16-character lowercase zero-padded hex of `id`'s big-endian `u64` —
/// the library key a [`NodeId`] mints under.
fn node_key(id: NodeId) -> String {
    format!("{:016x}", id.0)
}

/// The same encoding for a [`HyperedgeId`].
fn hyperedge_key(id: HyperedgeId) -> String {
    format!("{:016x}", id.0)
}

/// The membership payload slot `hypergraph-rs`'s `MembershipEdge<M>` carries
/// on every (member, hyperedge) incidence edge — the `petgraph` edge
/// weight, structurally the right home for Amendment AG (i) / ADR189's
/// attributed membership, because the (member, hyperedge) pair it names is
/// exactly the key this slot hangs on.
///
/// **Carried, empty, unhashed — nothing in this train writes it.** The
/// library exposes the slot with zero accessors (six construction sites
/// hard-code `M::default()`, zero reads —
/// `percy-raskova/hypergraph-rs#2`, filed Phase B Task 4 Step 4). This type
/// exists so the slot already sits in the adapter's type when the accessor
/// lands upstream; the moment an AG task adds a write through it, this
/// comment is the first thing it must revisit.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct MembershipPayload;

/// The `hypergraph-rs`-backed [`GraphSubstrate`]/[`CanonicalState`] adapter.
/// See the module documentation. Nothing but [`Self::new`] is public — a
/// caller depends on the traits; the store's internal shape is its own
/// business.
#[derive(Debug, Default)]
pub struct HypergraphStore {
    /// Dyadic half — identical in shape to `MemoryGraph`'s.
    nodes: HashMap<NodeId, String>,
    node_keys: HashMap<String, NodeId>,
    attributes: HashMap<(NodeId, String), f64>,
    /// `(node, name)` -> Currency — the sixth-section store (T3 #491,
    /// OQ-J), adapter-native like the rest of the dyadic half: Currency
    /// attributes never touch the library. Mirrors `MemoryGraph`'s own
    /// field exactly — see its doc for the "never the same key as
    /// `attributes`" partition.
    currency_attributes: HashMap<(NodeId, String), Currency>,
    edges: HashMap<(String, NodeId, NodeId), f64>,
    /// `(edge_type, from, to, qname)` -> value — the fifth-section store
    /// (T3, ADR198 R1, issue #560), adapter-native like the rest of the
    /// dyadic half: edge attributes never touch the library, exactly as
    /// strength never does. Strength is NOT here — it lives in `edges`
    /// above (section `0x03`'s datum; the double-storage ruling, D143).
    edge_attributes: HashMap<(String, NodeId, NodeId, String), f64>,
    /// `(hyperedge, qname)` -> value — the seventh-section store (Community
    /// port train, Task 5, E2a), adapter-native like the rest of the typed
    /// half: hyperedge attributes never touch the library, exactly as edge
    /// attributes never do. Swept by `remove_hyperedge` (ADR185 R2,
    /// extended to the own-field lane).
    hyperedge_attributes: HashMap<(HyperedgeId, String), f64>,
    /// Hyperedge half — the library key -> `HyperedgeId` reverse map, and the
    /// `(hyperedge_type -> ids)` index the library carries no type
    /// dimension for (delta §4: "the library has no type-keyed query, so
    /// the adapter must build the index itself regardless").
    hyperedge_keys: HashMap<String, HyperedgeId>,
    hyperedge_type_index: HashMap<String, Vec<HyperedgeId>>,
    /// The Levi/incidence store — internal only (D-1). `E = String` carries
    /// the hyperedge type (delta §4 CD1: no type-keyed query on the
    /// library side, so the type rides in `attrs` AND the side index).
    inner: Hypergraph<(), String, MembershipPayload>,
    next_id: u64,
    next_hyperedge_id: u64,
}

impl Clone for HypergraphStore {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            node_keys: self.node_keys.clone(),
            attributes: self.attributes.clone(),
            currency_attributes: self.currency_attributes.clone(),
            edges: self.edges.clone(),
            edge_attributes: self.edge_attributes.clone(),
            hyperedge_attributes: self.hyperedge_attributes.clone(),
            hyperedge_keys: self.hyperedge_keys.clone(),
            hyperedge_type_index: self.hyperedge_type_index.clone(),
            inner: self.inner.copy(),
            next_id: self.next_id,
            next_hyperedge_id: self.next_hyperedge_id,
        }
    }
}

impl AllocatorState for HypergraphStore {
    fn allocator_cursors(&self) -> AllocatorCursors {
        AllocatorCursors {
            next_node: self.next_id,
            next_hyperedge: self.next_hyperedge_id,
        }
    }
}

impl DetachedCopy for HypergraphStore {
    fn detached_copy(&self) -> Self {
        self.clone()
    }
}

#[cfg(test)]
impl crate::allocator_state::AllocatorTestControl for HypergraphStore {
    fn set_allocator_cursors_for_test(&mut self, cursors: AllocatorCursors) {
        self.next_id = cursors.next_node;
        self.next_hyperedge_id = cursors.next_hyperedge;
    }
}

impl HypergraphStore {
    /// An empty substrate.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace every mutable numeric lane from one checkpoint-bound stable state.
    ///
    /// The scenario loader remains the sole topology constructor. This method
    /// first proves that the checkpoint has exactly the sealed node, edge, and
    /// hyperedge topology, then builds all replacement maps off to the side and
    /// swaps them in together.
    ///
    /// # Errors
    /// Returns the first topology, stable-identity, or allocation refusal
    /// without changing the store.
    pub fn restore_stable_state_v1(
        &mut self,
        resolver: &StableElementResolverV1,
        state: &StableGraphStateV1,
    ) -> Result<(), StableIdentityError> {
        resolver.validate_topology(self)?;
        let current = encode_stable_graph_state_v1(self, resolver)?;
        let source = state.rows();
        let live = current.rows();
        let same_nodes = source.nodes() == live.nodes();
        let same_edges = source.edges().len() == live.edges().len()
            && source
                .edges()
                .iter()
                .zip(live.edges())
                .all(|(left, right)| left.0 == right.0 && left.1 == right.1 && left.2 == right.2);
        let same_hyperedges = source.hyperedges() == live.hyperedges();
        if !(same_nodes && same_edges && same_hyperedges) {
            return Err(StableIdentityError::TopologyChanged);
        }

        let mut attributes = try_hash_map("restored node f64 attributes", source.node_f64().len())?;
        for (local_name, qname, bits) in source.node_f64() {
            let node = resolver.node_handle_by_local_name(local_name)?;
            attributes.insert((node, qname.clone()), f64::from_bits(*bits));
        }
        let mut currency_attributes = try_hash_map(
            "restored node Currency attributes",
            source.node_currency().len(),
        )?;
        for (local_name, qname, micro_units) in source.node_currency() {
            let node = resolver.node_handle_by_local_name(local_name)?;
            currency_attributes.insert(
                (node, qname.clone()),
                Currency::from_micro_units(*micro_units),
            );
        }
        let mut edges = try_hash_map("restored edges", source.edges().len())?;
        for (edge_type, source_name, target_name, strength_bits) in source.edges() {
            let from = resolver.node_handle_by_local_name(source_name)?;
            let to = resolver.node_handle_by_local_name(target_name)?;
            edges.insert(
                (edge_type.clone(), from, to),
                f64::from_bits(*strength_bits),
            );
        }
        let mut edge_attributes =
            try_hash_map("restored edge f64 attributes", source.edge_f64().len())?;
        for (edge_type, source_name, target_name, qname, bits) in source.edge_f64() {
            let from = resolver.node_handle_by_local_name(source_name)?;
            let to = resolver.node_handle_by_local_name(target_name)?;
            edge_attributes.insert(
                (edge_type.clone(), from, to, qname.clone()),
                f64::from_bits(*bits),
            );
        }
        let mut hyperedge_attributes = try_hash_map(
            "restored hyperedge f64 attributes",
            source.hyperedge_f64().len(),
        )?;
        for (local_name, qname, bits) in source.hyperedge_f64() {
            let hyperedge = resolver.hyperedge_handle_by_local_name(local_name)?;
            hyperedge_attributes.insert((hyperedge, qname.clone()), f64::from_bits(*bits));
        }

        self.attributes = attributes;
        self.currency_attributes = currency_attributes;
        self.edges = edges;
        self.edge_attributes = edge_attributes;
        self.hyperedge_attributes = hyperedge_attributes;
        Ok(())
    }

    /// The frozen pre-check (delta §8 covenant 6, CD4) at the head of every
    /// one of the 8 mutating [`GraphSubstrate`] methods — `add_node`,
    /// `remove_node`, `add_edge`, `remove_edge`, `update_node`,
    /// `update_edge` (T3, ADR198 R1/R3), `add_hyperedge`,
    /// `remove_hyperedge` — via the library's public
    /// `is_frozen()`. Nothing in this crate ever freezes `self.inner` (no
    /// `GraphSubstrate` method exposes a freeze verb — that would be
    /// amendment territory, delta §4 CD4), so this is defense against a
    /// FUTURE reachable path (`hypergraph_rs::subhypergraph` freezes its
    /// return and there is no unfreeze anywhere in the library) rather than
    /// a condition this train can trigger.
    fn check_not_frozen(&self) -> Result<(), GraphError> {
        if self.inner.is_frozen() {
            return Err(GraphError {
                message: "substrate is frozen — no mutation is possible".to_owned(),
            });
        }
        Ok(())
    }

    /// Drop `id` from the `hyperedge_type` bucket of the type index. A
    /// no-op if `id` was never in it — callers pass the type an id was
    /// minted or found under, so this never needs to be loud.
    fn drop_from_type_index(&mut self, hyperedge_type: &str, id: HyperedgeId) {
        if let Some(ids) = self.hyperedge_type_index.get_mut(hyperedge_type) {
            ids.retain(|existing| *existing != id);
        }
    }
}

fn try_hash_map<K: Eq + std::hash::Hash, V>(
    field: &'static str,
    capacity: usize,
) -> Result<HashMap<K, V>, StableIdentityError> {
    let mut values = HashMap::new();
    values
        .try_reserve(capacity)
        .map_err(|_| StableIdentityError::Allocation {
            field,
            requested: capacity,
        })?;
    Ok(values)
}

impl GraphSubstrate for HypergraphStore {
    fn add_node(&mut self, node_type: &str) -> Result<NodeId, GraphError> {
        self.check_not_frozen()?;
        let next_id = self.next_id.checked_add(1).ok_or_else(|| GraphError {
            message: "node identity allocator exhausted".to_owned(),
        })?;
        let id = NodeId(self.next_id);
        let key = node_key(id);
        self.nodes.insert(id, node_type.to_owned());
        self.node_keys.insert(key.clone(), id);
        // Covenant 4: mint through the library too, or the existence check
        // below proves nothing about the library's universe.
        self.inner.add_node(&key, ());
        self.next_id = next_id;
        Ok(id)
    }

    fn remove_node(&mut self, id: NodeId) -> Result<(), GraphError> {
        self.check_not_frozen()?;
        if self.nodes.remove(&id).is_none() {
            return Err(GraphError {
                message: format!("no such node: {id:?}"),
            });
        }
        let key = node_key(id);
        self.node_keys.remove(&key);
        self.attributes.retain(|(node, _), _| *node != id);
        self.currency_attributes.retain(|(node, _), _| *node != id);
        self.edges
            .retain(|(_, from, to), _| *from != id && *to != id);
        // The cascade sweeps the fifth section too (T3, ADR198 R1): an
        // incident edge's attribute rows go with the edge, exactly as the
        // edge itself goes — no key naming a corpse, in any map.
        self.edge_attributes
            .retain(|(_, from, to, _), _| *from != id && *to != id);

        // ADR185 R2 cascade, hyperedge half. Capture (edge key, type) for
        // every hyperedge this node belongs to BEFORE the library call —
        // once the library has run its own weak-removal-with-cleanup, a
        // hyperedge that lost its LAST member is gone from `inner` and its
        // attrs are unreachable, so the type must be read first.
        let prior_memberships: Vec<(String, String)> = self
            .inner
            .memberships(&key)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|edge_key| {
                self.inner
                    .edge_attrs(&edge_key)
                    .cloned()
                    .map(|ty| (edge_key, ty))
            })
            .collect();

        // strong=false, remove_empty=true: detach this node from every
        // membership, then drop any hyperedge left with none — exactly
        // ADR185 R2 (H2 SHRINK), verified by the conformance suite rather
        // than assumed from the signature (Task 7 Step 5).
        self.inner
            .remove_node(&key, false, true)
            .map_err(|e| GraphError {
                message: format!("hyperedge-half removal of {id:?}: {e}"),
            })?;

        let mut dropped: Vec<HyperedgeId> = Vec::new();
        for (edge_key, hyperedge_type) in prior_memberships {
            if !self.inner.has_edge(&edge_key) {
                if let Some(hid) = self.hyperedge_keys.remove(&edge_key) {
                    self.drop_from_type_index(&hyperedge_type, hid);
                    // The implicitly-dropped hyperedge's own-field rows die
                    // with it (PR #682 harvest — this path missed the sweep
                    // before): no corpse rows into section 0x07.
                    dropped.push(hid);
                }
            }
        }
        // One sweep for the whole drop set (PR #683's own review): retain
        // per dropped id would make the cascade O(dropped × rows).
        if !dropped.is_empty() {
            self.hyperedge_attributes
                .retain(|(hyperedge, _), _| !dropped.contains(hyperedge));
        }
        Ok(())
    }

    fn add_edge(
        &mut self,
        edge_type: &str,
        from: NodeId,
        to: NodeId,
        strength: f64,
    ) -> Result<(), GraphError> {
        self.check_not_frozen()?;
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
        self.check_not_frozen()?;
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
        self.check_not_frozen()?;
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
        self.check_not_frozen()?;
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
        self.check_not_frozen()?;
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
        self.check_not_frozen()?;
        // The loud preamble (delta §8 covenant 3), CD2/CD3 in one place:
        // empty, duplicate, and unknown members are all loud errors here,
        // before delegation — the sort happens anyway (members come back
        // ascending), so the duplicate check is free. The floor is exactly
        // ONE member; "hardening" to two would smuggle in an unruled
        // cardinality constant.
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
        let key = hyperedge_key(id);
        let member_keys: Vec<String> = sorted.iter().map(|n| node_key(*n)).collect();
        self.inner
            .add_edge(member_keys, Some(key.clone()), hyperedge_type.to_owned())
            .map_err(|e| GraphError {
                message: format!("hyperedge-half mint: {e}"),
            })?;
        self.hyperedge_keys.insert(key, id);
        self.hyperedge_type_index
            .entry(hyperedge_type.to_owned())
            .or_default()
            .push(id);
        self.next_hyperedge_id = next_id;
        Ok(id)
    }

    fn remove_hyperedge(&mut self, id: HyperedgeId) -> Result<(), GraphError> {
        self.check_not_frozen()?;
        let key = hyperedge_key(id);
        let hyperedge_type = self.inner.edge_attrs(&key).cloned();
        self.inner.remove_edge(&key).map_err(|_| GraphError {
            message: format!("no such hyperedge: {id:?}"),
        })?;
        self.hyperedge_keys.remove(&key);
        if let Some(ty) = hyperedge_type {
            self.drop_from_type_index(&ty, id);
        }
        // ADR185 R2, extended to the own-field lane (Task 5): the
        // hyperedge's attribute rows die with it, never orphaned.
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
        self.check_not_frozen()?;
        if !value.is_finite() {
            return Err(GraphError {
                message: format!("non-finite hyperedge attribute {attribute} on {id:?}"),
            });
        }
        let key = hyperedge_key(id);
        if self.inner.edge_attrs(&key).is_none() {
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
        let key = hyperedge_key(id);
        if self.inner.edge_attrs(&key).is_none() {
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
        let key = hyperedge_key(id);
        let members = self.inner.members(&key).ok_or_else(|| GraphError {
            message: format!("no such hyperedge: {id:?}"),
        })?;
        let mut result: Vec<NodeId> = members
            .into_iter()
            .map(|member_key| {
                *self.node_keys.get(&member_key).unwrap_or_else(|| {
                    panic!(
                        "internal invariant violated: library member {member_key} \
                         has no NodeId mapping — node/library universe desync"
                    )
                })
            })
            .collect();
        result.sort_unstable(); // BSL D25: declared member order is never observable
        Ok(result)
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
        let Some(type_ids) = self.hyperedge_type_index.get(hyperedge_type) else {
            return Ok(Vec::new());
        };
        let key = node_key(node);
        let membership_keys = self.inner.memberships(&key).unwrap_or_default();
        let mut found: Vec<HyperedgeId> = membership_keys
            .into_iter()
            .filter_map(|member_key| self.hyperedge_keys.get(&member_key).copied())
            .filter(|hid| type_ids.contains(hid))
            .collect();
        found.sort_unstable();
        Ok(found)
    }

    fn hyperedges(&self, hyperedge_type: &str) -> Vec<HyperedgeId> {
        // The already-present type index (built by add_hyperedge/
        // remove_hyperedge) — no second index for one datum. Its per-type
        // Vec is populated by push-in-mint-order and never reordered by
        // remove's retain, so it is ascending by construction; the explicit
        // sort here is the CONTRACT (never assume a storage detail), not a
        // correction of one.
        let mut found: Vec<HyperedgeId> = self
            .hyperedge_type_index
            .get(hyperedge_type)
            .cloned()
            .unwrap_or_default();
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
        self.inner
            .edge_attrs(&hyperedge_key(id))
            .map(String::as_str)
            .ok_or_else(|| GraphError {
                message: format!(
                    "no such hyperedge: {id:?} — a dangling ref never reads as untyped"
                ),
            })
    }
}

impl CanonicalState for HypergraphStore {
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

    /// Walks the library's `edge_ids()`, maps each through the identity
    /// map, reads members, maps those back, and reads the type off `attrs`
    /// — no sorting here (the provided `encode_state` sorts; a store that
    /// needed to sort to come out right here would have a bug this method
    /// should not paper over).
    fn all_hyperedges(&self) -> Vec<(HyperedgeId, String, Vec<NodeId>)> {
        self.inner
            .edge_ids()
            .into_iter()
            .map(|key| {
                let id = *self.hyperedge_keys.get(&key).unwrap_or_else(|| {
                    panic!(
                        "internal invariant violated: library edge {key} \
                         has no HyperedgeId mapping"
                    )
                });
                let hyperedge_type = self.inner.edge_attrs(&key).cloned().unwrap_or_else(|| {
                    panic!("internal invariant violated: library edge {key} has no type attrs")
                });
                let members = self
                    .inner
                    .members(&key)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|member_key| {
                        *self.node_keys.get(&member_key).unwrap_or_else(|| {
                            panic!(
                                "internal invariant violated: library member {member_key} \
                                 has no NodeId mapping"
                            )
                        })
                    })
                    .collect();
                (id, hyperedge_type, members)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::HypergraphStore;
    use crate::allocator_state::{AllocatorCursors, AllocatorState, AllocatorTestControl};
    use crate::conformance::{
        run_allocator_exhaustion_conformance, run_detached_working_copy_conformance,
        run_substrate_conformance,
    };
    use crate::state_hash::CanonicalState;
    use crate::substrate::GraphSubstrate;
    use crate::working_copy::DetachedCopy;
    use std::fmt::Write as _;

    #[test]
    fn hypergraph_store_passes_the_conformance_suite() {
        run_substrate_conformance(HypergraphStore::new);
    }

    #[test]
    fn detached_working_copy_owns_every_mutable_lane() {
        run_detached_working_copy_conformance(HypergraphStore::new);
    }

    #[test]
    fn exhausted_allocators_leave_hypergraph_store_unchanged() {
        run_allocator_exhaustion_conformance(HypergraphStore::new);
    }

    #[test]
    fn failed_library_hyperedge_insert_does_not_advance_the_cursor() {
        let mut graph = HypergraphStore::new();
        let member = graph.add_node("SOCIAL_CLASS").unwrap();
        graph.add_hyperedge("COALITION", &[member]).unwrap();
        graph.set_allocator_cursors_for_test(AllocatorCursors {
            next_node: 1,
            next_hyperedge: 0,
        });
        let before = graph.encode_state().unwrap().as_bytes().to_vec();
        let cursors = graph.allocator_cursors();
        let indexed = graph.hyperedges("COALITION");
        let memberships = graph.hyperedges_of(member, "COALITION").unwrap();
        let members = graph.members_of(indexed[0]).unwrap();

        let error = graph
            .add_hyperedge("COALITION", &[member])
            .expect_err("the library must reject the duplicate internal edge key");

        assert!(error.message.contains("hyperedge-half mint"));
        assert_eq!(graph.encode_state().unwrap().as_bytes(), before);
        assert_eq!(graph.allocator_cursors(), cursors);
        assert_eq!(graph.hyperedges("COALITION"), indexed);
        assert_eq!(
            graph.hyperedges_of(member, "COALITION").unwrap(),
            memberships
        );
        assert_eq!(graph.members_of(indexed[0]).unwrap(), members);

        graph.set_allocator_cursors_for_test(AllocatorCursors {
            next_node: 1,
            next_hyperedge: 1,
        });
        let next = graph
            .add_hyperedge("COALITION", &[member])
            .expect("a valid mint after the rejected library insert succeeds");
        assert_eq!(next, crate::substrate::HyperedgeId(1));
        assert_eq!(
            graph.hyperedges("COALITION"),
            vec![indexed[0], crate::substrate::HyperedgeId(1)]
        );
        assert_eq!(
            graph.hyperedges_of(member, "COALITION").unwrap(),
            vec![indexed[0], crate::substrate::HyperedgeId(1)]
        );
        assert_eq!(graph.members_of(next).unwrap(), vec![member]);
        let detached = graph.detached_copy();
        assert_eq!(detached.allocator_cursors(), graph.allocator_cursors());
    }

    #[test]
    fn allocator_cursors_report_the_next_ids_even_after_removal() {
        let mut graph = HypergraphStore::new();
        let first = graph.add_node("SOCIAL_CLASS").unwrap();
        let second = graph.add_node("SOCIAL_CLASS").unwrap();
        graph.add_hyperedge("CLASS", &[first, second]).unwrap();
        graph.remove_node(second).unwrap();

        assert_eq!(
            graph.allocator_cursors(),
            AllocatorCursors {
                next_node: 2,
                next_hyperedge: 1,
            }
        );
    }

    /// Task 3's III.7 evidence, not a claim (P27 Phase 2 Slice 1): the
    /// literal below was captured from `dev` BEFORE `node_type_of` was
    /// added — this fixture (3 nodes, 2 edges, 1 hyperedge, mixed
    /// attributes) is the III.7 baseline, and this test is run again AFTER
    /// the trait widens to prove the hash did not move. T2's `edge_attribute`
    /// (issue #559) is the SECOND widening this test proves clean, alongside
    /// `node_type_of` — the same fixture and hex digest still apply, since
    /// `edge_attribute` is read-only and adds no `CanonicalState` byte.
    #[test]
    fn adding_a_read_only_query_method_does_not_move_the_state_hash() {
        let mut graph = HypergraphStore::new();
        let a = graph.add_node("SOCIAL_CLASS").unwrap();
        let b = graph.add_node("SOCIAL_CLASS").unwrap();
        let c = graph.add_node("TERRITORY").unwrap();
        graph.update_node(a, "social-class/wages", 120.0).unwrap();
        graph.update_node(a, "social-class/agitation", 0.3).unwrap();
        graph.update_node(c, "territory/heat", 0.5).unwrap();
        graph.add_edge("SOLIDARITY", a, b, 0.75).unwrap();
        graph.add_edge("ADJACENCY", b, c, 1.0).unwrap();
        graph.add_hyperedge("CELL", &[a, b, c]).unwrap();

        let hash = graph.state_hash().unwrap();
        let hex = hash.iter().fold(String::new(), |mut acc, byte| {
            let _ = write!(acc, "{byte:02x}");
            acc
        });
        assert_eq!(
            hex, "9577d95124a7c4ed6faad2c4aca5980b435fb73e7b58813413500a5fdef798ed",
            "state_hash moved — a read-only GraphSubstrate method addition (node_type_of \
             or edge_attribute) must be III.7-clean (read-only, no new CanonicalState \
             section, no byte moved)"
        );
    }
}
