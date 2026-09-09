//! Hyperedge-lane Task 1 (E1a): the `(hyperedge …)` scenario top-form.
//!
//! `docs/superpowers/plans/2026-08-18-community-port.md`'s Task 1 — the
//! hyperedge lane's first landing. Before this train, `scenario.rs:63-64`
//! stated plainly: "No hyperedges yet. The grammar has room for them;
//! nothing in slice 1 needs one, and an unused form is an untested form."
//! Every world the Community BSL port depends on needs one, so this is that
//! form's own test file — the first thing to exercise it end to end, and the
//! file every later task's own hyperedge-touching test builds on.
//!
//! Three properties this file pins, one per Task 1 step:
//!
//! - **Step 1/2** — the top-form loads, resolves its `HyperedgeType` through
//!   the closed vocabulary (`E-LOAD-031 UnknownEnumMember` on a typo, never
//!   the field-qname code `E-LOAD-023` revision 1 of the plan wrongly
//!   prescribed), resolves each member by the same local-name table
//!   `node`/`edge` share, refuses an empty member list before ever reaching
//!   the substrate, and canonicalizes member order exactly as
//!   `structural_verbs.rs::EffectExecutor::add_hyperedge` already does.
//! - **Step 3** — `HyperedgeId`s mint on their own counter; inserting a
//!   `hyperedge` form never shifts a `NodeId`.
//! - **Step 4** — `LoadedScenario::hyperedge_types` /
//!   `LoadedScenario::max_members_seen` count the seeded population, the
//!   same way `node_types`/`edge_types` already do.

use babylon_bsl::scenario::load_scenario;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{Direction, GraphError, GraphSubstrate, HyperedgeId, NodeId};
use babylon_kernel::currency::Currency;

// ---- Step 1: the pre-existing catch-all refusal, still live for a form
// that really is unknown (a `hyperedge` tag no longer hits this arm, but a
// bogus one still does) ----

const BOGUS_FORM: &str = r"
(scenario ft/hyperedge-lane-bogus-form
  (bogus-form x))
";

#[test]
fn a_genuinely_unknown_top_form_still_refuses_and_the_message_now_lists_hyperedge() {
    // Before this task, `(hyperedge ...)` fell into this exact arm too —
    // `load_scenario_inner`'s dispatch `_ =>` case, whose message read
    // "a scenario body form must begin with `defenum`, `defvocabulary`,
    // `deffield`, `defconst`, `node`, `edge` or `edge-attr`" (no `hyperedge`
    // at all). This test pins two facts at once: the catch-all mechanism
    // itself is untouched (a truly unknown tag still refuses), and the
    // message's own enumeration now names `hyperedge` — the change of
    // behaviour Step 1 asked to make visible in the diff.
    let mut graph = MemoryGraph::new();
    let err = load_scenario(BOGUS_FORM, &mut graph).unwrap_err();
    assert!(
        err.message.contains("hyperedge"),
        "the catch-all message must now list `hyperedge` among the legal top-forms: {}",
        err.message
    );
    assert!(err.message.contains("defenum"), "{}", err.message);
    assert!(
        err.code.is_none(),
        "the generic catch-all carries no spec code"
    );
}

// ---- Step 2: the top-form itself ----

const NEW_AFRIKAN: &str = r"
(scenario ft/hyperedge-lane-new-afrikan
  (defvocabulary HyperedgeType (COMMUNITY))
  (node alpha NodeType/SOCIAL_CLASS)
  (node beta NodeType/SOCIAL_CLASS)
  (hyperedge new-afrikan HyperedgeType/COMMUNITY (members alpha beta)))
";

#[test]
fn the_hyperedge_top_form_mints_and_resolves_members_by_local_name() {
    let mut graph = MemoryGraph::new();
    let loaded = load_scenario(NEW_AFRIKAN, &mut graph).expect("the hyperedge top-form now loads");

    assert_eq!(loaded.hyperedge_types.get("COMMUNITY"), Some(&1));
    assert_eq!(loaded.max_members_seen.get("COMMUNITY"), Some(&2));

    // `alpha`/`beta` are the first two `node` forms — NodeId(0)/NodeId(1)
    // (declaration order is id order, scenario.rs:41-45), unaffected by the
    // `hyperedge` form that follows them.
    let alpha = NodeId(0);
    let beta = NodeId(1);
    let hyperedges = graph
        .hyperedges_of(alpha, "COMMUNITY")
        .expect("alpha exists");
    assert_eq!(
        hyperedges.len(),
        1,
        "alpha belongs to exactly one hyperedge"
    );
    assert_eq!(
        graph.members_of(hyperedges[0]).unwrap(),
        vec![alpha, beta],
        "members resolved through the same local-name table node/edge share"
    );
}

const UNKNOWN_MEMBER: &str = r"
(scenario ft/hyperedge-lane-unknown-member
  (defvocabulary HyperedgeType (COMMUNITY))
  (node alpha NodeType/SOCIAL_CLASS)
  (hyperedge x HyperedgeType/NOWHERE (members alpha)))
";

#[test]
fn an_unknown_hyperedge_type_member_is_e_load_031_not_e_load_023() {
    // Revision 1 of the plan prescribed E-LOAD-023 (UnknownFieldOwner — a
    // FIELD QNAME's first segment naming no registered type,
    // vocabulary.rs:145-150/163/213) for this case. The corrected brief:
    // this is E-LOAD-031 (UnknownEnumMember — a member the registered TYPE
    // does not carry, vocabulary.rs:118/160/192), the same code
    // `check_enum_ref` raises for `(node x NodeType/NOWHERE)`.
    let mut graph = MemoryGraph::new();
    let err = load_scenario(UNKNOWN_MEMBER, &mut graph).unwrap_err();
    assert_eq!(err.code, Some("E-LOAD-031"), "{}", err.message);
    assert!(err.message.contains("NOWHERE"), "{}", err.message);
}

// Fix round 1 (review I1): a DIFFERENT branch from the one above —
// UNKNOWN_MEMBER exercises an unknown *HyperedgeType member* (the type/enum
// check, `demand_enum_kind`/`check_enum_ref`); this exercises an unknown
// *member NODE name* (`named.get(name)` in the member-resolution loop),
// 35 lines further down in `load_hyperedge`. `beta` is declared but never
// referenced as a hyperedge member, so a mutation collapsing the lookup to
// `unwrap_or(NodeId(0))` would silently substitute `alpha` (NodeId(0)) for
// the typo'd name — two DISTINCT existing nodes, no substrate-level
// duplicate-member refusal to accidentally catch it — which is exactly why
// `beta`, not `alpha`, is the declared-but-unused control here.
const UNKNOWN_MEMBER_NAME: &str = r"
(scenario ft/hyperedge-lane-unknown-member-name
  (defvocabulary HyperedgeType (COMMUNITY))
  (node alpha NodeType/SOCIAL_CLASS)
  (node beta NodeType/SOCIAL_CLASS)
  (hyperedge x HyperedgeType/COMMUNITY (members beta nosuch)))
";

#[test]
fn an_unknown_member_node_name_refuses_distinctly_from_an_unknown_hyperedge_type_member() {
    let mut graph = MemoryGraph::new();
    let err = load_scenario(UNKNOWN_MEMBER_NAME, &mut graph).unwrap_err();
    assert!(
        err.message.contains("names unknown node `nosuch`"),
        "{}",
        err.message
    );
    assert!(
        err.code.is_none(),
        "this refusal carries no spec code — distinct from E-LOAD-031, which the \
         test above exercises for the OTHER unknown-name branch: {}",
        err.message
    );
}

const EMPTY_MEMBERS: &str = r"
(scenario ft/hyperedge-lane-empty-members
  (defvocabulary HyperedgeType (COMMUNITY))
  (hyperedge x HyperedgeType/COMMUNITY (members)))
";

#[test]
fn an_empty_member_list_refuses_before_ever_reaching_the_substrate() {
    // Matches MemoryGraph::add_hyperedge's own refusal text verbatim
    // (babylon-graph/src/memory.rs:357-361) — the loader's own check fires
    // first, so a scenario-authoring mistake is diagnosed at the form that
    // caused it rather than bubbling up as a generic substrate refusal.
    let mut graph = MemoryGraph::new();
    let err = load_scenario(EMPTY_MEMBERS, &mut graph).unwrap_err();
    assert!(
        err.message
            .contains("hyperedge must have at least one member"),
        "{}",
        err.message
    );
    // The differentiator: `GraphSubstrate::add_hyperedge`'s OWN refusal (had
    // the loader's pre-check not fired) would reach here wrapped by
    // `ScenarioError::from(GraphError)` as "substrate refused the scenario:
    // …" — a plain `.contains()` on the target text alone cannot tell the
    // loader's own pre-substrate check apart from that bubble-up, since
    // both contain the identical wording. This assertion is what actually
    // proves the loader's check fires FIRST, never reaching the substrate
    // at all.
    assert!(
        !err.message.starts_with("substrate refused the scenario"),
        "the refusal must be the loader's OWN pre-check, not a bubbled-up \
         GraphError from GraphSubstrate::add_hyperedge: {}",
        err.message
    );
}

// Fix round 1 (review I3): two `(hyperedge ...)` forms declaring the SAME
// local name — `load_node`'s own duplicate check has no hyperedge analogue
// before this fix, so this fixture would have loaded silently.
const DUPLICATE_HYPEREDGE_NAME: &str = r"
(scenario ft/hyperedge-lane-duplicate-name
  (defvocabulary HyperedgeType (COMMUNITY))
  (node alpha NodeType/SOCIAL_CLASS)
  (node beta NodeType/SOCIAL_CLASS)
  (hyperedge h0 HyperedgeType/COMMUNITY (members alpha))
  (hyperedge h0 HyperedgeType/COMMUNITY (members beta)))
";

#[test]
fn a_duplicate_hyperedge_local_name_refuses_mirroring_load_nodes_own_check() {
    let mut graph = MemoryGraph::new();
    let err = load_scenario(DUPLICATE_HYPEREDGE_NAME, &mut graph).unwrap_err();
    assert!(
        err.message.contains("duplicate scenario name `h0`"),
        "{}",
        err.message
    );
    assert!(
        err.message.contains("exactly one hyperedge"),
        "the wording must name the element kind, mirroring load_node's own \
         \"exactly one node\" phrasing one kind over: {}",
        err.message
    );
    // Only ONE hyperedge minted — the second form's mint never runs.
    assert_eq!(
        graph.hyperedges_of(NodeId(0), "COMMUNITY").unwrap().len(),
        1,
        "the duplicate form must refuse before minting a second hyperedge"
    );
}

// ---- Step 3: the id-order law extends to hyperedges, on an independent
// counter ----

const WITH_HYPEREDGE: &str = r"
(scenario ft/hyperedge-lane-id-order-with
  (defvocabulary HyperedgeType (COMMUNITY))
  (node alpha NodeType/SOCIAL_CLASS)
  (node beta NodeType/SOCIAL_CLASS)
  (hyperedge h0 HyperedgeType/COMMUNITY (members alpha beta))
  (node gamma NodeType/SOCIAL_CLASS))
";

const WITHOUT_HYPEREDGE: &str = r"
(scenario ft/hyperedge-lane-id-order-without
  (node alpha NodeType/SOCIAL_CLASS)
  (node beta NodeType/SOCIAL_CLASS)
  (node gamma NodeType/SOCIAL_CLASS))
";

#[test]
fn inserting_a_hyperedge_form_does_not_shift_any_node_id() {
    let mut graph_with = MemoryGraph::new();
    let loaded_with = load_scenario(WITH_HYPEREDGE, &mut graph_with).unwrap();
    let mut graph_without = MemoryGraph::new();
    let loaded_without = load_scenario(WITHOUT_HYPEREDGE, &mut graph_without).unwrap();

    // `gamma` is the THIRD `node` form in both files. A `hyperedge` form
    // sits between `beta` and `gamma` in WITH_HYPEREDGE only — if hyperedge
    // minting consumed a NodeId, gamma would land on NodeId(3) there and
    // NodeId(2) in the other file. It must land on NodeId(2) in BOTH.
    assert_eq!(
        loaded_with.node_content_ids.get(&NodeId(2)),
        Some(&"gamma".to_owned()),
        "the hyperedge form must not shift gamma's NodeId"
    );
    assert_eq!(
        loaded_without.node_content_ids.get(&NodeId(2)),
        Some(&"gamma".to_owned())
    );
    assert_eq!(loaded_with.node_count, 3);
    assert_eq!(loaded_with.hyperedge_types.get("COMMUNITY"), Some(&1));

    // The two counters are independent (substrate.rs:35-41's type-level
    // separation) — the minted hyperedge is HyperedgeId(0), never anything
    // derived from the node counter's current position.
    let hyperedges = graph_with
        .hyperedges_of(NodeId(0), "COMMUNITY")
        .expect("alpha exists");
    assert_eq!(hyperedges, vec![HyperedgeId(0)]);
}

// ---- Step 4: LoadedScenario::hyperedge_types / max_members_seen ----

const THREE_HYPEREDGES: &str = r"
(scenario ft/hyperedge-lane-three-hyperedges
  (defvocabulary HyperedgeType (COMMUNITY ECONOMIC_SECTOR))
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (node c NodeType/SOCIAL_CLASS)
  (node d NodeType/SOCIAL_CLASS)
  (node e NodeType/SOCIAL_CLASS)
  (node f NodeType/SOCIAL_CLASS)
  (hyperedge h0 HyperedgeType/COMMUNITY (members a b c))
  (hyperedge h1 HyperedgeType/COMMUNITY (members d))
  (hyperedge h2 HyperedgeType/ECONOMIC_SECTOR (members e f)))
";

#[test]
fn hyperedge_types_and_max_members_seen_pin_against_a_three_hyperedge_fixture() {
    // h0/h1 are both COMMUNITY (3 members, then 1 — unequal, so the max is
    // genuinely exercised, not just echoing a single observation); h2 is a
    // SEPARATE type (ECONOMIC_SECTOR, 2 members) — proving the maps are keyed per
    // type, not one global count/max.
    let mut graph = MemoryGraph::new();
    let loaded = load_scenario(THREE_HYPEREDGES, &mut graph).unwrap();

    assert_eq!(loaded.hyperedge_types.get("COMMUNITY"), Some(&2));
    assert_eq!(loaded.hyperedge_types.get("ECONOMIC_SECTOR"), Some(&1));
    assert_eq!(loaded.hyperedge_types.len(), 2, "no stray type entries");

    assert_eq!(loaded.max_members_seen.get("COMMUNITY"), Some(&3));
    assert_eq!(loaded.max_members_seen.get("ECONOMIC_SECTOR"), Some(&2));
    assert_eq!(loaded.max_members_seen.len(), 2);
}

// ---- Step 2 continued: member canonicalization agrees with the executor's
// already-landed law, proven against a substrate that does NOT self-correct
// declared order ----

/// A recording [`GraphSubstrate`] wrapper — delegates every method to a real
/// [`MemoryGraph`], except [`GraphSubstrate::add_hyperedge`], which
/// additionally records the exact member slice it was called with (before
/// any substrate-internal re-sort) into `hyperedge_call_args`.
///
/// **Why this wrapper exists rather than just reading `members_of()` back.**
/// Both `MemoryGraph` (`babylon-graph/src/memory.rs:362-363`) and the
/// production `HypergraphStore` (`hypergraph_store.rs:415-416`) ALSO sort
/// members internally on insert — defense in depth, per their own comments.
/// That means `members_of()` alone cannot distinguish "the loader
/// pre-sorted before calling `add_hyperedge`" from "the loader passed
/// declared order and the substrate corrected it after the fact" — both
/// produce the identical stored result against either real substrate. The
/// executor's own regression test for this exact property
/// (`structural_verbs.rs::tests::hyperedge_members_are_canonicalized_never_
/// logged_as_declared`) sidesteps the same problem by inspecting its write
/// log instead of the substrate — the one surface a self-correcting
/// substrate cannot launder. Scenario loading writes no log at all, so this
/// wrapper's recorded call argument is the nearest equivalent vantage
/// point: it observes what the LOADER handed the substrate, independent of
/// what the substrate does with it next.
#[derive(Default)]
struct OrderSpyGraph {
    inner: MemoryGraph,
    hyperedge_call_args: Vec<Vec<NodeId>>,
}

impl GraphSubstrate for OrderSpyGraph {
    fn add_node(&mut self, node_type: &str) -> Result<NodeId, GraphError> {
        self.inner.add_node(node_type)
    }

    fn remove_node(&mut self, id: NodeId) -> Result<(), GraphError> {
        self.inner.remove_node(id)
    }

    fn add_edge(
        &mut self,
        edge_type: &str,
        from: NodeId,
        to: NodeId,
        strength: f64,
    ) -> Result<(), GraphError> {
        self.inner.add_edge(edge_type, from, to, strength)
    }

    fn remove_edge(&mut self, edge_type: &str, from: NodeId, to: NodeId) -> Result<(), GraphError> {
        self.inner.remove_edge(edge_type, from, to)
    }

    fn update_node(&mut self, id: NodeId, attribute: &str, value: f64) -> Result<(), GraphError> {
        self.inner.update_node(id, attribute, value)
    }

    fn update_edge(
        &mut self,
        edge_type: &str,
        from: NodeId,
        to: NodeId,
        attribute: &str,
        value: f64,
    ) -> Result<(), GraphError> {
        self.inner
            .update_edge(edge_type, from, to, attribute, value)
    }

    fn node_attribute(&self, id: NodeId, attribute: &str) -> Result<f64, GraphError> {
        self.inner.node_attribute(id, attribute)
    }

    // The Currency lane (T3, #491/OQ-J): pure delegation, un-spied — this
    // double exists to watch hyperedge membership calls only, and the
    // currency lane never participates in those.
    fn update_node_currency(
        &mut self,
        id: NodeId,
        attribute: &str,
        value: Currency,
    ) -> Result<(), GraphError> {
        self.inner.update_node_currency(id, attribute, value)
    }

    fn node_attribute_currency(&self, id: NodeId, attribute: &str) -> Result<Currency, GraphError> {
        self.inner.node_attribute_currency(id, attribute)
    }

    // The hyperedge own-field lane (Community Task 5, E2a): pure
    // delegation, un-spied — the lane is this double's own watch target,
    // but only its MEMBERSHIP calls (`add_hyperedge`/`remove_hyperedge`)
    // are recorded; attribute reads/writes pass through.
    fn update_hyperedge_attribute(
        &mut self,
        id: HyperedgeId,
        attribute: &str,
        value: f64,
    ) -> Result<(), GraphError> {
        self.inner.update_hyperedge_attribute(id, attribute, value)
    }

    fn hyperedge_attribute(&self, id: HyperedgeId, attribute: &str) -> Result<f64, GraphError> {
        self.inner.hyperedge_attribute(id, attribute)
    }

    fn node_exists(&self, id: NodeId) -> bool {
        self.inner.node_exists(id)
    }

    fn nodes(&self, node_type: &str) -> Vec<NodeId> {
        self.inner.nodes(node_type)
    }

    fn edges(&self, edge_type: &str) -> Vec<(NodeId, NodeId)> {
        self.inner.edges(edge_type)
    }

    fn neighbors(
        &self,
        node: NodeId,
        edge_type: &str,
        direction: Direction,
    ) -> Result<Vec<NodeId>, GraphError> {
        self.inner.neighbors(node, edge_type, direction)
    }

    fn add_hyperedge(
        &mut self,
        hyperedge_type: &str,
        members: &[NodeId],
    ) -> Result<HyperedgeId, GraphError> {
        self.hyperedge_call_args.push(members.to_vec());
        self.inner.add_hyperedge(hyperedge_type, members)
    }

    fn remove_hyperedge(&mut self, id: HyperedgeId) -> Result<(), GraphError> {
        self.inner.remove_hyperedge(id)
    }

    fn members_of(&self, id: HyperedgeId) -> Result<Vec<NodeId>, GraphError> {
        self.inner.members_of(id)
    }

    fn hyperedges_of(
        &self,
        node: NodeId,
        hyperedge_type: &str,
    ) -> Result<Vec<HyperedgeId>, GraphError> {
        self.inner.hyperedges_of(node, hyperedge_type)
    }

    fn hyperedges(&self, hyperedge_type: &str) -> Vec<HyperedgeId> {
        self.inner.hyperedges(hyperedge_type)
    }

    fn node_type_of(&self, id: NodeId) -> Result<&str, GraphError> {
        self.inner.node_type_of(id)
    }

    // Task 3's review-driven addition (PR #684): pure delegation,
    // un-spied, exactly as the own-field lane above.
    fn hyperedge_type_of(&self, id: HyperedgeId) -> Result<&str, GraphError> {
        self.inner.hyperedge_type_of(id)
    }

    fn edge_attribute(
        &self,
        edge_type: &str,
        from: NodeId,
        to: NodeId,
        attribute: &str,
    ) -> Result<f64, GraphError> {
        self.inner.edge_attribute(edge_type, from, to, attribute)
    }
}

const UNSORTED_MEMBERS: &str = r"
(scenario ft/hyperedge-lane-unsorted-members
  (defvocabulary HyperedgeType (COMMUNITY))
  (node alpha NodeType/SOCIAL_CLASS)
  (node beta NodeType/SOCIAL_CLASS)
  (node gamma NodeType/SOCIAL_CLASS)
  (hyperedge h0 HyperedgeType/COMMUNITY (members gamma alpha beta)))
";

#[test]
fn the_loader_sorts_members_ascending_before_minting_matching_the_executors_law() {
    // alpha/beta/gamma resolve to NodeId(0)/NodeId(1)/NodeId(2) — declared
    // as `gamma alpha beta`, i.e. NodeId order [2, 0, 1]: deliberately
    // unsorted, exactly what the brief calls for.
    let mut graph = OrderSpyGraph::default();
    load_scenario(UNSORTED_MEMBERS, &mut graph).unwrap();
    assert_eq!(
        graph.hyperedge_call_args,
        vec![vec![NodeId(0), NodeId(1), NodeId(2)]],
        "the loader must hand GraphSubstrate::add_hyperedge an \
         ALREADY-ascending member slice — structural_verbs.rs's \
         EffectExecutor::add_hyperedge (:1361-1366) sorts before calling \
         the substrate for the identical reason (D25: declared member order \
         is never observable, and the write log — the loader's own \
         analogue of which is this wrapper's recorded call — must not leak \
         it back)"
    );
}
