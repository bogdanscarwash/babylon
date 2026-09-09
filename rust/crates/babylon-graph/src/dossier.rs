//! `L` — an organization's DOSSIER on others (Lane A brick 3; heat dossier
//! §5.1 A).
//!
//! **Symmetric form, asymmetric means** (Director ruling 2026-07-30,
//! ADR184 R3 consequence). Every actor holds a dossier, because
//! [`crate::capacity`]'s ruling makes repression and revolutionary action
//! the same allocation and an allocation runs on *belief* — so a movement
//! misreads the apparatus exactly as the apparatus misreads a movement.
//! What differs is **how each one resolves**: a state resolves by SPENDING
//! a surveillance instrument it must fund; a movement resolves by BEING
//! embedded in the social structures it moves through, which costs it
//! nothing and reaches only where it already stands. That asymmetry lives
//! in the resolution *paths* — which are content (BSL rules) and Phase 2
//! bindings — never in this type. This module supplies the operations; it
//! does not know who is looking or what it cost them.
//!
//! Scott's argument survives that symmetry rather than being flattened by
//! it: legibility is specifically what states do to populations, and the
//! reason is visible here as the *price* and *reach* of resolution, not as
//! a privileged type. The state buys a wide, shallow, standardized view;
//! the movement gets a narrow, deep, untransferable one.
//!
//! The heat reformulation splits one question-begging `[0,1]` scalar into
//! three quantities with different owners: what an actor **knows** (`L`,
//! here), what it **spends** (`K`, [`crate::capacity`]), and what its target
//! **is** (`X`, [`crate::exposure`], derived and never stored).
//! Conflating them is what forced the old mechanic to key repression to
//! *conduct* — the ideological falsehood the dossier indicts. Kept apart,
//! nothing needs to ask whether the player broke a law.
//!
//! **A dossier is a projection, not a fraction of the truth.** Scott's five
//! properties of state legibility — interested, documentary, static,
//! aggregate, standardized — mean `L` keeps only what the state's own
//! categories can hold. So this type stores what the state has *resolved*
//! and nothing else: it holds no pointer back to ground truth, and there is
//! deliberately no method that answers "what is really there". Code that
//! wants the truth must ask the substrate; code that wants the state's
//! belief asks a `Dossier`. Those are different questions, and the type
//! system is what keeps them different.
//!
//! **There is no threat score here, and none may be added.** Sparrow's
//! conscious-opponent argument forbids an accumulated priority: an opponent
//! who knows he is being ranked reorganizes, so the ranking must be
//! recomputed from live structure at decision time. `L` records *resolution*
//! — which nodes and ties the state has managed to see — never a verdict
//! about a target. The verdict is [`crate::exposure`] evaluated over
//! [`Dossier::scope`], every time it is asked for.
//!
//! **Growth and decay are not symmetric, and neither is conduct-keyed.**
//! Resolution grows through named collection channels and — Sparrow's
//! feedback loop — fastest where it is already high, because a resolved node
//! exposes its neighbors. It decays through membership turnover,
//! compartmentalization, and restructuring. Both are rules expressed as
//! content (BSL), not arithmetic baked in here; this module supplies only
//! the operations those rules drive.

use crate::substrate::{GraphError, GraphSubstrate, NodeId};
use std::collections::{BTreeMap, BTreeSet};

/// One actor's partial, biased view of another's structure.
///
/// Symmetric in form: a police force's dossier on a union and a union's
/// dossier on a police force are the same type doing the same job. What
/// differs is the price and reach of resolving anything — see the module
/// doc (ADR184 R10).
///
/// Iteration order everywhere is ascending and deterministic (`BTree*`), so
/// two runs that resolved the same facts produce the same scope in the same
/// order — a tick-hash prerequisite, not a convenience.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dossier {
    /// Whose belief this is.
    ///
    /// Parallel to [`crate::capacity::Capacity::owner`] and for the same
    /// reason (ADR184): a belief with no believer is the unowned-state error
    /// one construct over. It also makes the anti-omniscience contract
    /// checkable — two dossiers over one substrate must be able to disagree,
    /// which is only meaningful once they are distinguishable.
    owner: NodeId,
    resolved_nodes: BTreeSet<NodeId>,
    /// `edge_type -> {(from, to)}`. Stored as authored, because *which way
    /// the state thinks a tie runs* is part of what it believes.
    resolved_edges: BTreeMap<String, BTreeSet<(NodeId, NodeId)>>,
}

impl Dossier {
    /// An empty dossier: `owner` knows nothing until it collects.
    ///
    /// This is the honest starting point and it matters — a dossier that
    /// began populated would grant free omniscience at tick 0, which is the
    /// failure this whole split exists to prevent. It holds for a movement
    /// exactly as for a state.
    #[must_use]
    pub fn new(owner: NodeId) -> Self {
        Self {
            owner,
            resolved_nodes: BTreeSet::new(),
            resolved_edges: BTreeMap::new(),
        }
    }

    /// Whose belief this is.
    #[must_use]
    pub fn owner(&self) -> NodeId {
        self.owner
    }

    /// Record that the state has resolved `node`.
    ///
    /// Idempotent: re-collecting a known node is not an error and does not
    /// double-count. Returns whether this call added something new — the
    /// signal a collection rule needs to decide whether effort paid off.
    pub fn resolve_node(&mut self, node: NodeId) -> bool {
        self.resolved_nodes.insert(node)
    }

    /// Record that the state has resolved a tie.
    ///
    /// Resolving a tie implies resolving BOTH endpoints: the state cannot
    /// coherently know that two people are connected without knowing they
    /// exist. Anything else would let a dossier describe ties among nodes it
    /// has never seen, and [`Self::scope`] would then hand
    /// [`crate::exposure`] a graph the state does not actually hold.
    pub fn resolve_edge(&mut self, edge_type: &str, from: NodeId, to: NodeId) -> bool {
        self.resolved_nodes.insert(from);
        self.resolved_nodes.insert(to);
        self.resolved_edges
            .entry(edge_type.to_owned())
            .or_default()
            .insert((from, to))
    }

    /// Drop a node and every tie touching it — membership turnover, a
    /// member going quiet, a cell restructuring out of the state's picture.
    ///
    /// Returns whether the node had been known. Decay is deliberately
    /// *whole*: a dossier retaining ties to a person it can no longer place
    /// would out-perform a real one.
    pub fn forget_node(&mut self, node: NodeId) -> bool {
        let known = self.resolved_nodes.remove(&node);
        for ties in self.resolved_edges.values_mut() {
            ties.retain(|(from, to)| *from != node && *to != node);
        }
        self.resolved_edges.retain(|_, ties| !ties.is_empty());
        known
    }

    /// Drop one tie, leaving both endpoints known — compartmentalization:
    /// the state still has the people, but no longer the link between them.
    pub fn forget_edge(&mut self, edge_type: &str, from: NodeId, to: NodeId) -> bool {
        let Some(ties) = self.resolved_edges.get_mut(edge_type) else {
            return false;
        };
        let removed = ties.remove(&(from, to));
        if ties.is_empty() {
            self.resolved_edges.remove(edge_type);
        }
        removed
    }

    /// Whether the state has resolved this node.
    #[must_use]
    pub fn knows_node(&self, node: NodeId) -> bool {
        self.resolved_nodes.contains(&node)
    }

    /// Whether the state has resolved this tie, as authored.
    #[must_use]
    pub fn knows_edge(&self, edge_type: &str, from: NodeId, to: NodeId) -> bool {
        self.resolved_edges
            .get(edge_type)
            .is_some_and(|ties| ties.contains(&(from, to)))
    }

    /// How many nodes the state has resolved.
    #[must_use]
    pub fn resolved_node_count(&self) -> usize {
        self.resolved_nodes.len()
    }

    /// How many ties the state has resolved, across every type.
    #[must_use]
    pub fn resolved_edge_count(&self) -> usize {
        self.resolved_edges.values().map(BTreeSet::len).sum()
    }

    /// The scope to hand [`crate::exposure`], ascending.
    ///
    /// This is the whole point of the type. Every exposure function takes a
    /// node set and is silent on whose view it is; passing this scope asks
    /// *"what does the state BELIEVE a strike would yield"*, while passing
    /// the substrate's own node list asks what it WOULD yield. Same
    /// arithmetic, different question — and the state gets the first one.
    #[must_use]
    pub fn scope(&self) -> Vec<NodeId> {
        self.resolved_nodes.iter().copied().collect()
    }

    /// Every resolved node that no longer exists in `graph`.
    ///
    /// A dossier outlives the structure it describes: arrests, deaths and
    /// dissolutions leave the state holding cards for people who are gone.
    /// [`crate::exposure`] rejects a dangling id loudly rather than reading
    /// it as empty, so a caller that has mutated the graph asks this first
    /// and reconciles with [`Self::forget_node`] — the state noticing, which
    /// is an action, not an automatic correction.
    #[must_use]
    pub fn stale_nodes(&self, graph: &impl GraphSubstrate) -> Vec<NodeId> {
        self.resolved_nodes
            .iter()
            .copied()
            .filter(|node| !graph.node_exists(*node))
            .collect()
    }

    /// Drop every resolved node the graph no longer has, and their ties.
    ///
    /// Returns what was dropped, ascending. Deliberately explicit rather
    /// than implicit inside [`Self::scope`]: reconciliation is the state
    /// spending attention, and a silent auto-clean would make the dossier
    /// track reality for free.
    pub fn reconcile(&mut self, graph: &impl GraphSubstrate) -> Vec<NodeId> {
        let stale = self.stale_nodes(graph);
        for node in &stale {
            self.forget_node(*node);
        }
        stale
    }

    /// Resolve `node` and every tie of `edge_type` it holds in `graph`,
    /// returning how many NEW facts that added.
    ///
    /// This is the shape of a successful collection action against a target
    /// the state already had: interrogating a resolved member exposes his
    /// contacts. It is also the mechanism behind Sparrow's feedback loop —
    /// resolution grows fastest where it is already high, because each
    /// resolved node is a doorway to its neighbors. The rule deciding WHEN
    /// this happens is content; this is only the operation it performs.
    ///
    /// # Errors
    /// Returns [`GraphError`] if `node` does not exist in `graph`.
    pub fn resolve_neighborhood(
        &mut self,
        graph: &impl GraphSubstrate,
        node: NodeId,
        edge_type: &str,
    ) -> Result<usize, GraphError> {
        if !graph.node_exists(node) {
            return Err(GraphError {
                message: format!("no such node: {node:?} — a dangling ref never reads empty"),
            });
        }
        let mut added = usize::from(self.resolve_node(node));
        for (from, to) in graph.edges(edge_type) {
            if from == node || to == node {
                // A newly-surfaced CONTACT is a new fact in its own right, and
                // usually the one that mattered — `resolve_edge` reports only
                // whether the TIE was new, so count the endpoints before it
                // silently inserts them. Undercounting here would tell a
                // collection rule that a productive interrogation bought
                // nothing.
                added += usize::from(!self.knows_node(from));
                added += usize::from(!self.knows_node(to));
                added += usize::from(self.resolve_edge(edge_type, from, to));
            }
        }
        Ok(added)
    }
}

#[cfg(test)]
mod tests {
    use super::Dossier;
    use crate::exposure::decapitation_value;
    use crate::memory::MemoryGraph;
    use crate::substrate::{GraphSubstrate, NodeId};

    /// Whoever is doing the looking. Most tests do not care who — that is
    /// the point of the symmetry (ADR184 R3): the operations are the same
    /// for a police force and a union local.
    const OBSERVER: NodeId = NodeId(1_000);

    /// A hub joined to `spokes` leaves — the centralized shape whose hub is
    /// worth striking.
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

    #[test]
    fn a_dossier_starts_empty() {
        let dossier = Dossier::new(OBSERVER);
        assert_eq!(dossier.resolved_node_count(), 0);
        assert_eq!(dossier.resolved_edge_count(), 0);
        assert!(dossier.scope().is_empty(), "the state knows nothing yet");
    }

    #[test]
    fn resolving_a_tie_resolves_both_endpoints() {
        // The state cannot know two people are connected without knowing
        // they exist — otherwise scope() would hand exposure a graph the
        // state does not hold.
        let mut dossier = Dossier::new(OBSERVER);
        assert!(dossier.resolve_edge("coordination", NodeId(4), NodeId(9)));
        assert!(dossier.knows_node(NodeId(4)));
        assert!(dossier.knows_node(NodeId(9)));
        assert_eq!(dossier.scope(), vec![NodeId(4), NodeId(9)]);
    }

    #[test]
    fn resolution_is_idempotent_and_reports_novelty() {
        let mut dossier = Dossier::new(OBSERVER);
        assert!(dossier.resolve_node(NodeId(1)), "first sighting is news");
        assert!(!dossier.resolve_node(NodeId(1)), "the second is not");
        assert_eq!(dossier.resolved_node_count(), 1);
    }

    #[test]
    fn forgetting_a_node_takes_its_ties_with_it() {
        // Decay is whole: a dossier retaining ties to someone it can no
        // longer place would out-perform a real one.
        let mut dossier = Dossier::new(OBSERVER);
        dossier.resolve_edge("coordination", NodeId(1), NodeId(2));
        dossier.resolve_edge("coordination", NodeId(2), NodeId(3));
        assert!(dossier.forget_node(NodeId(2)));
        assert_eq!(dossier.resolved_edge_count(), 0);
        assert!(dossier.knows_node(NodeId(1)), "the others stay known");
        assert!(dossier.knows_node(NodeId(3)));
    }

    #[test]
    fn compartmentalization_drops_the_tie_and_keeps_the_people() {
        let mut dossier = Dossier::new(OBSERVER);
        dossier.resolve_edge("coordination", NodeId(1), NodeId(2));
        assert!(dossier.forget_edge("coordination", NodeId(1), NodeId(2)));
        assert!(dossier.knows_node(NodeId(1)));
        assert!(dossier.knows_node(NodeId(2)));
        assert!(!dossier.knows_edge("coordination", NodeId(1), NodeId(2)));
        assert!(
            !dossier.forget_edge("coordination", NodeId(1), NodeId(2)),
            "forgetting what is already forgotten is not news"
        );
    }

    #[test]
    fn resolving_a_neighborhood_grows_the_dossier_from_a_doorway() {
        // Sparrow's feedback loop: a resolved node exposes its neighbors, so
        // knowledge grows fastest where it is already high.
        let (graph, hub, all) = star(4);
        let mut dossier = Dossier::new(OBSERVER);
        let added = dossier
            .resolve_neighborhood(&graph, hub, "coordination")
            .unwrap();
        assert_eq!(dossier.resolved_node_count(), all.len(), "the whole star");
        assert_eq!(dossier.resolved_edge_count(), 4);
        assert_eq!(
            added, 9,
            "1 hub + 4 contacts newly surfaced + 4 ties = 9 new facts; the \
             contacts are usually the ones that mattered, so they count"
        );
        let again = dossier
            .resolve_neighborhood(&graph, hub, "coordination")
            .unwrap();
        assert_eq!(again, 0, "re-running the same collection adds nothing");
    }

    #[test]
    fn a_dangling_collection_target_is_loud() {
        let (graph, ..) = star(2);
        let mut dossier = Dossier::new(OBSERVER);
        let err = dossier
            .resolve_neighborhood(&graph, NodeId(999), "coordination")
            .unwrap_err();
        assert!(err.message.contains("never reads empty"), "{}", err.message);
    }

    #[test]
    fn reconciliation_is_explicit_never_automatic() {
        // A dossier outlives the structure it describes. Noticing costs the
        // state an action; scope() must not quietly self-clean.
        let (mut graph, hub, all) = star(3);
        let mut dossier = Dossier::new(OBSERVER);
        dossier
            .resolve_neighborhood(&graph, hub, "coordination")
            .unwrap();
        let gone = all[1];
        graph.remove_node(gone).unwrap();

        assert!(
            dossier.knows_node(gone),
            "the card is still in the drawer until the state looks"
        );
        assert_eq!(dossier.stale_nodes(&graph), vec![gone]);
        assert_eq!(dossier.reconcile(&graph), vec![gone]);
        assert!(!dossier.knows_node(gone));
        assert!(dossier.stale_nodes(&graph).is_empty());
    }

    #[test]
    fn the_state_can_be_wrong_about_what_a_strike_would_yield() {
        // THE anti-omniscience contract. Same arithmetic, two scopes: the
        // true graph answers what a strike WOULD yield, the dossier answers
        // what the state BELIEVES it would. A partial dossier that has
        // resolved only part of a hub's reach undervalues that hub — the
        // state under-rates a target it cannot fully see. If these two ever
        // agree by construction, the engine has granted the state free
        // omniscience and this whole split is decoration.
        let (graph, hub, all) = star(6);

        let truth = decapitation_value(&graph, &all, "coordination", hub, 1).unwrap();

        let mut dossier = Dossier::new(OBSERVER);
        dossier.resolve_node(hub);
        for leaf in all.iter().skip(1).take(2) {
            dossier.resolve_edge("coordination", hub, *leaf);
        }
        let believed =
            decapitation_value(&graph, &dossier.scope(), "coordination", hub, 1).unwrap();

        assert!(
            believed < truth,
            "a partially-resolved hub must look less valuable than it is \
             (believed {believed}, true {truth})"
        );
    }

    #[test]
    fn a_fully_resolved_dossier_agrees_with_the_truth() {
        // The other side of the same contract: the divergence above is a
        // consequence of PARTIAL knowledge, not an artefact of scoping. When
        // the state has resolved everything, its belief must converge.
        let (graph, hub, all) = star(6);
        let truth = decapitation_value(&graph, &all, "coordination", hub, 1).unwrap();

        let mut dossier = Dossier::new(OBSERVER);
        dossier
            .resolve_neighborhood(&graph, hub, "coordination")
            .unwrap();
        let believed =
            decapitation_value(&graph, &dossier.scope(), "coordination", hub, 1).unwrap();

        assert!(
            (believed - truth).abs() < 1e-12,
            "full resolution must converge (believed {believed}, true {truth})"
        );
    }

    #[test]
    fn two_actors_can_hold_different_beliefs_about_one_substrate() {
        // ADR184 R3's symmetry, and the reason `owner` exists. A police
        // force and a union local look at the SAME graph and come away with
        // different pictures — not because the arithmetic differs, but
        // because each resolved different parts of it. If dossiers were
        // ownerless there would be one belief for everyone, which is
        // omniscience wearing a partial-knowledge costume.
        let (graph, hub, spokes) = star(5);
        let police = NodeId(900);
        let local = NodeId(901);

        // `star`'s third value leads with the hub, so the leaves are the
        // tail — the police resolved rank-and-file and never the coordinator.
        let mut wide_and_shallow = Dossier::new(police);
        for leaf in spokes.iter().filter(|node| **node != hub) {
            wide_and_shallow.resolve_node(*leaf);
        }

        let mut narrow_and_deep = Dossier::new(local);
        narrow_and_deep
            .resolve_neighborhood(&graph, hub, "coordination")
            .unwrap();

        assert_ne!(
            wide_and_shallow, narrow_and_deep,
            "two actors' beliefs are distinguishable values"
        );
        assert_eq!(wide_and_shallow.owner(), police);
        assert_eq!(narrow_and_deep.owner(), local);
        assert!(
            !wide_and_shallow.knows_node(hub),
            "the police saw the members and missed the coordinator"
        );
        assert!(
            narrow_and_deep.knows_node(hub),
            "the local knows its own coordinator"
        );

        // And the disagreement is not cosmetic: it changes what each one
        // believes a strike on the hub would be worth.
        let police_view =
            decapitation_value(&graph, &wide_and_shallow.scope(), "coordination", hub, 1);
        let local_view =
            decapitation_value(&graph, &narrow_and_deep.scope(), "coordination", hub, 1).unwrap();
        assert!(
            police_view.is_err() || police_view.unwrap() < local_view,
            "a dossier that never resolved the hub cannot value striking it \
             the way one that did does"
        );
    }
}
