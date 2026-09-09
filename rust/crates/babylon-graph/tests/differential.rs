//! The differential harness — Phase C Task 8. A fixed operation script
//! applied to a [`MemoryGraph`] and a [`HypergraphStore`] IN LOCKSTEP,
//! asserting `encode_state().as_bytes()` equality after EVERY SINGLE
//! operation, not only at the end.
//!
//! Bytes rather than hashes on purpose (`StateEncoder::as_bytes` exists for
//! exactly this): a hash says the two states differ; the bytes say WHERE —
//! which section, which entry. A one-encoder design (`CanonicalState`,
//! Phase A Task 1) means a divergence here can only be the two stores
//! REPORTING different facts through `all_nodes`/`all_attributes`/
//! `all_edges`/`all_hyperedges`/`all_edge_attributes` — never a difference
//! in how the bytes get written, because both stores share the identical
//! `encode_state` body.

use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::{GraphSubstrate, HyperedgeId, NodeId};

/// Drives one `GraphSubstrate + CanonicalState` operation against both
/// stores at once and asserts their canonical bytes agree immediately
/// after — a divergence fails at the FIRST operation that caused it, never
/// only at the end.
struct Twin {
    memory: MemoryGraph,
    hyper: HypergraphStore,
    step: usize,
}

impl Twin {
    fn new() -> Self {
        Self {
            memory: MemoryGraph::new(),
            hyper: HypergraphStore::new(),
            step: 0,
        }
    }

    fn assert_synced(&mut self, label: &str) {
        self.step += 1;
        let memory_bytes = self.memory.encode_state().unwrap();
        let hyper_bytes = self.hyper.encode_state().unwrap();
        assert_eq!(
            memory_bytes.as_bytes(),
            hyper_bytes.as_bytes(),
            "canonical bytes diverged at step {} ({label}) — MemoryGraph and \
             HypergraphStore reported different facts",
            self.step
        );
    }

    fn add_node(&mut self, node_type: &str) -> NodeId {
        let m = self.memory.add_node(node_type).unwrap();
        let h = self.hyper.add_node(node_type).unwrap();
        assert_eq!(m, h, "identity minting diverged at step {}", self.step + 1);
        self.assert_synced("add_node");
        m
    }

    fn update_node(&mut self, id: NodeId, attribute: &str, value: f64) {
        self.memory.update_node(id, attribute, value).unwrap();
        self.hyper.update_node(id, attribute, value).unwrap();
        self.assert_synced("update_node");
    }

    fn add_edge(&mut self, edge_type: &str, from: NodeId, to: NodeId, strength: f64) {
        self.memory.add_edge(edge_type, from, to, strength).unwrap();
        self.hyper.add_edge(edge_type, from, to, strength).unwrap();
        self.assert_synced("add_edge");
    }

    /// T3 (ADR198 R1-R3, issue #560): the edge-attribute write op. Riding the
    /// same per-operation byte-equality assertion as every other op is what
    /// proves the two stores' FIFTH-section facts agree — including the
    /// strength fork (a `/strength` write must move section `0x03` on both,
    /// never mint a `0x05` row on either).
    fn update_edge(
        &mut self,
        edge_type: &str,
        from: NodeId,
        to: NodeId,
        attribute: &str,
        value: f64,
    ) {
        self.memory
            .update_edge(edge_type, from, to, attribute, value)
            .unwrap();
        self.hyper
            .update_edge(edge_type, from, to, attribute, value)
            .unwrap();
        self.assert_synced("update_edge");
    }

    fn remove_edge(&mut self, edge_type: &str, from: NodeId, to: NodeId) {
        self.memory.remove_edge(edge_type, from, to).unwrap();
        self.hyper.remove_edge(edge_type, from, to).unwrap();
        self.assert_synced("remove_edge");
    }

    fn add_hyperedge(&mut self, hyperedge_type: &str, members: &[NodeId]) -> HyperedgeId {
        let m = self.memory.add_hyperedge(hyperedge_type, members).unwrap();
        let h = self.hyper.add_hyperedge(hyperedge_type, members).unwrap();
        assert_eq!(
            m,
            h,
            "hyperedge identity minting diverged at step {}",
            self.step + 1
        );
        self.assert_synced("add_hyperedge");
        m
    }

    /// D2 (PR #494 adversarial review): the ONE mutating `GraphSubstrate`
    /// method the original script never drove — and the one touching all
    /// three structures `add_hyperedge` also touches but `remove_node`'s
    /// cascade exercises only indirectly (the Levi store, the
    /// `hyperedge_keys` reverse map, and the `hyperedge_type_index`).
    fn remove_hyperedge(&mut self, id: HyperedgeId) {
        self.memory.remove_hyperedge(id).unwrap();
        self.hyper.remove_hyperedge(id).unwrap();
        self.assert_synced("remove_hyperedge");
    }

    fn remove_node(&mut self, id: NodeId) {
        self.memory.remove_node(id).unwrap();
        self.hyper.remove_node(id).unwrap();
        self.assert_synced("remove_node");
    }
}

#[test]
fn canonical_bytes_agree_after_every_operation_in_a_mixed_script() {
    let mut twin = Twin::new();

    // Node adds across a decade boundary (delta §4 CD5's numeric-vs-
    // lexicographic hazard) — 13 nodes, so NodeId(10) exists.
    let mut nodes = Vec::with_capacity(13);
    for i in 0..13 {
        let ty = if i % 3 == 0 {
            "territory"
        } else {
            "social_class"
        };
        nodes.push(twin.add_node(ty));
    }

    // Attribute writes including -0.0 and a value that ROUNDS to it
    // (upstream arithmetic producing a signed zero), plus ordinary values.
    twin.update_node(nodes[0], "wealth", -0.0);
    twin.update_node(nodes[1], "wealth", -(0.0_f64)); // computed negation, rounds to -0.0
    twin.update_node(nodes[2], "wealth", 0.0);
    twin.update_node(nodes[10], "wealth", 42.5);
    twin.update_node(nodes[10], "value_produced", 1_000_000.25);

    // Typed edges across THREE types, declared in an order that fights
    // ascending (source, target) — high source id first, mixed types.
    twin.add_edge("wages", nodes[12], nodes[0], 1.0);
    twin.add_edge("solidarity", nodes[5], nodes[1], 0.5);
    twin.add_edge("tribute", nodes[9], nodes[2], 0.25);
    twin.add_edge("wages", nodes[3], nodes[0], 0.75);
    twin.add_edge("solidarity", nodes[1], nodes[5], 0.9); // reverse of the pair above

    // T3 (ADR198 R1-R3): edge-attribute writes — a deffield-declared field
    // on two different edge types, a `-0.0` (the fifth section inherits the
    // sign-bit canonicalization), and a STRENGTH-fork write proving both
    // stores route it to section 0x03, never a shadow 0x05 row. The
    // removals below then exercise the cascade: remove_edge and remove_node
    // must shrink the two stores' fifth sections in lockstep.
    twin.update_edge("solidarity", nodes[5], nodes[1], "solidarity/tension", 0.7);
    twin.update_edge("wages", nodes[12], nodes[0], "wages/value-flow", 1_234.5);
    twin.update_edge("tribute", nodes[9], nodes[2], "tribute/rate", -0.0);
    twin.update_edge(
        "solidarity",
        nodes[1],
        nodes[5],
        "solidarity/strength",
        0.95,
    );

    // Hyperedges: one of a SINGLE member, one of MANY with declared order
    // reversed against id order.
    twin.add_hyperedge("household", &[nodes[7]]);
    twin.add_hyperedge(
        "economic_sector",
        &[nodes[12], nodes[9], nodes[5], nodes[1]],
    );
    twin.add_hyperedge("economic_sector", &[nodes[2], nodes[0]]);
    // D2: a hyperedge over members NONE of which the removals below touch,
    // so its own explicit removal (not a remove_node cascade) is what
    // exercises the direct path.
    let directly_removed = twin.add_hyperedge("economic_sector", &[nodes[3], nodes[4], nodes[6]]);

    // Removals exercising the CASCADE (a multi-member hyperedge shrinks,
    // dyadic edges and attributes go with the node), the LAST-MEMBER case
    // (the single-member "household" hyperedge disappears whole), and the
    // DIRECT removal path (remove_hyperedge itself, not a remove_node
    // side effect) — the only mutating GraphSubstrate method touching all
    // three novel structures (the Levi store, the hyperedge_keys reverse
    // map, the hyperedge_type_index) that a cascade-only script would
    // never drive.
    twin.remove_edge("wages", nodes[12], nodes[0]);
    twin.remove_node(nodes[1]); // in "economic_sector" (multi) + two edges + an attribute
    twin.remove_node(nodes[7]); // sole member of "household" — last-member removal
    twin.remove_hyperedge(directly_removed);

    // One more add after the cascade, to prove minting still agrees post-removal.
    let extra = twin.add_node("organization");
    twin.add_edge("membership", extra, nodes[10], 1.0);
    // …and an edge-attribute write on the freshly minted edge, so the fifth
    // section's post-cascade growth is compared too (the cascade legs above
    // already shrank it: nodes[1]'s removal took solidarity/tension, the
    // wages removal took wages/value-flow).
    twin.update_edge("membership", extra, nodes[10], "membership/accrual", 0.1);
}
