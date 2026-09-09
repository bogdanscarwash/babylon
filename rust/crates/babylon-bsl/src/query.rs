//! Query element materialization (`bsl-language.rst` §2.6).
//!
//! **Slice boundary, recorded honestly.** Task 2 of the BSL
//! query-evaluation plan
//! (`docs/superpowers/plans/2026-08-11-bsl-query-evaluation-plan.md`) added
//! this module a task early, so [`crate::evaluator::EvalEnv`]'s §2.6
//! chapter C8 element stack had a type to hold before any query head
//! actually produced one. Task 4 is the one that fills in [`materialize`]:
//! `nodes` and typed `neighbors` — the two heads slice 1 serves; `edges`
//! joined them at T2 (slice 2, issue #559). The remaining three §2.6 heads
//! (`hyperedges`, `members-of`, `hyperedges-of`) are MATERIALIZED as of
//! slice 3's landing (Community port train, Task 3, 2026-08-21) — the
//! refusal arm they used to route through is deleted with the unserved
//! table (Task 3 Step 4's at-the-byte decision, recorded there).
//!
//! [`Element::Node`] and [`Element::Edge`] exist ([`EdgeKey`] was minted at
//! T2, slice 2). `Element::Hyperedge(HyperedgeId)` landed with slice 3
//! (Community port train, Task 3, 2026-08-21): the variant is constructed
//! by this module's own materialization of the three §2.6 hyperedge heads,
//! served below — never a constructed-by-nothing placeholder.
//!
//! **Determinism (Constraint 2).** `nodes`/`neighbors` on `GraphSubstrate`
//! already return a canonically sorted, deduplicated `Vec` (§2.6's total
//! order is the substrate's own contract, not something this module must
//! re-establish). `neighbors`' `NodeType` filter therefore runs as a `Vec`
//! filter **after** that sort — never by re-collecting into a `HashMap`/
//! `HashSet`, which would launder the order through an unspecified one.

use crate::evaluator::{charge, evaluate, require_graph, EvalEnv, EvalError, Value};
use crate::fuel::cost;
use crate::intrinsic_host::IntrinsicHost;
use crate::reader::{Atom, SExpr};
use babylon_graph::substrate::{Direction, HyperedgeId, NodeId};

/// A materialized edge's identity — the `(source, target, edge_type)` triple IS the identity
/// (§2.10's own "well-defined because the triple is a key" ruling, `bsl-language.rst:1896-1904`);
/// `GraphSubstrate` mints no separate `EdgeId` (only `NodeId`/`HyperedgeId` exist,
/// `substrate.rs:33,41`). Field order is `(source, target, edge_type)` DELIBERATELY: declaring the
/// fields in §2.6's own `(source-id, target-id, edge-type)` total-order sequence makes the derived
/// `Ord`'s field-by-field comparison agree with the spec's order by construction (T2 plan Task 3
/// design decision 1, `docs/superpowers/plans/2026-08-12-t2-slice2-edge-reads-plan.md`). That
/// property is exercised by this crate's own direct Ord test
/// (`tests::edge_key_ord_prioritizes_source_over_edge_type`) — NOT by `materialize_edges`, which
/// never invokes this derive.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdgeKey {
    /// The edge's source node.
    pub source: NodeId,
    /// The edge's target node.
    pub target: NodeId,
    /// The edge's declared `EdgeType` member — owned, matching both
    /// backends' `HashMap<(String, NodeId, NodeId), f64>` key shape.
    pub edge_type: String,
}

/// One materialized graph element (§2.6).
///
/// **T2's cross-kind Ord ruling (register row D140, CT4P A5 / issue #525, T2 issue #559).** §2.6
/// defines a total order WITHIN each query kind's own result set only — it is silent on comparing a
/// `Node` to an `Edge`. No production `materialize()` call ever mixes kinds (`edges` returns only
/// `Edge`; `nodes`/`neighbors` only `Node`), so this ordering is UNREACHABLE in practice — pinned
/// anyway, per this enum's own standing instruction, rather than left to whatever `#[derive(Ord)]`
/// happens to produce from declaration order. RULED: `Node` sorts before `Edge`, by declaration
/// order below — arbitrary, deliberate, tested (`tests::node_sorts_before_edge_regardless_of_id`).
///
/// **Slice-3 ruling (Community port train, Task 3, rider 4 clause 1, C5 /
/// D-NF+24, 2026-08-21): `Node` sorts before `Edge` sorts before
/// `Hyperedge`, by declaration order — arbitrary, deliberate, tested** (the
/// companions `edge_sorts_before_hyperedge_regardless_of_id` and
/// `the_three_kind_order_holds_regardless_of_ids`). Exactly the reasoning
/// the T2 paragraph above gives: pinned, never left to whatever
/// `#[derive(Ord)]` happens to produce.
///
/// **No longer `Copy` (T2, issue #559): `EdgeKey` owns a `String`.** Every call site that relied on
/// `Copy` is fixed at this variant's landing (see evaluator.rs's own Task-3 call-site fixes) —
/// `Clone` is unaffected and remains the currency for every place that needs an owned `Element`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Element {
    /// A materialized node — see the module doc.
    Node(NodeId),
    /// A materialized dyadic edge (slice 2, T2). Declared SECOND: see this enum's own cross-kind
    /// Ord ruling above.
    Edge(EdgeKey),
    /// A materialized hyperedge (slice 3, Community Task 3). Declared
    /// THIRD: the cross-kind ruling above.
    Hyperedge(HyperedgeId),
}

impl Element {
    /// This element's runtime value. Takes `&self` (not `self`) now that `Element` is no longer
    /// `Copy` — every existing caller already holds a `&Element` at the point it calls this
    /// (`env.elements`'s own `(Option<String>, Element)` tuples are read by reference throughout),
    /// so no caller needs to change; only the signature does, and the `Edge` arm clones its key.
    #[must_use]
    pub fn to_value(&self) -> Value {
        match self {
            Self::Node(id) => Value::NodeRef(*id),
            Self::Edge(key) => Value::EdgeRef(key.clone()),
            Self::Hyperedge(id) => Value::HyperedgeRef(*id),
        }
    }
}

/// All six §2.6 query heads are served (`hyperedges`/`members-of`/
/// `hyperedges-of` joined at slice 3's landing, Community port train Task
/// 3, 2026-08-21). The unserved table this comment once introduced is
/// DELETED with this module's refusal arm — a zero-row table would be the
/// plan's own M4 failure mode (a duplicated table named once), recorded as
/// Task 3 Step 4's at-the-byte decision. `metric-of` is not a §2.6 query
/// head at all (it lives in `evaluator::UNSERVED_EXPRESSION_HEADS`).
/// Materialize a `<query>` form in §2.6's total order, charging §3.7's
/// `cost(query)` base (the querying form's own base — `fold`/`exists`/…
/// charge their own separately) plus this query's operand expression(s).
///
/// # Errors
///
/// A query head slice 1 does not serve (named, with its slice); a dangling
/// element operand (the substrate's own loud error, or — for `neighbors`'
/// source operand — an UNCODED loud error per D-row Q3, since the reference
/// names no `E-EVAL` code for a dangling query operand); a missing graph
/// (`require_graph`'s driver error); a malformed shape.
pub fn materialize(
    query: &SExpr,
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Vec<Element>, EvalError> {
    let SExpr::List(items) = query else {
        return Err(EvalError::plain(format!(
            "expected a §2.6 query form, found {query:?}"
        )));
    };
    let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() else {
        return Err(EvalError::plain(
            "a query form must be headed by a symbol (§2.6)",
        ));
    };
    match head.as_str() {
        "nodes" => materialize_nodes(items, env, fuel),
        "neighbors" => materialize_neighbors(items, env, host, fuel),
        "edges" => materialize_edges(items, env, fuel),
        "hyperedges" => materialize_hyperedges(items, env, fuel),
        "hyperedges-of" => materialize_hyperedges_of(items, env, host, fuel),
        "members-of" => materialize_members_of(items, env, host, fuel),
        h => Err(EvalError::plain(format!(
            "{h} is not one of §2.6's six query heads"
        ))),
    }
}

/// `(nodes <enum-ref> <node-pred>?)`. The `<node-pred>` operand is a real
/// §2.6 grammar production, but zero conformance vectors and zero content
/// rules exercise it (verified 2026-08-11 the same way D51's `neighbors`
/// finding was) — slice 1's task list names no predicate test, so it stays a
/// loud, named gap rather than a silent one (Constraint 4): a predicated
/// `nodes` refuses by name instead of silently ignoring the predicate or
/// applying an unreviewed reading of it.
fn materialize_nodes(
    items: &[SExpr],
    env: &EvalEnv<'_>,
    fuel: &mut u64,
) -> Result<Vec<Element>, EvalError> {
    charge(fuel, cost::QUERY_BASE)?;
    let [_, type_ref, extra @ ..] = items else {
        return Err(EvalError::plain(
            "(nodes <enum-ref> <node-pred>?) — missing the NodeType operand",
        ));
    };
    if !extra.is_empty() {
        return Err(EvalError::plain(
            "(nodes <enum-ref> <node-pred>) — the element-predicate operand \
             is a real §2.6 production this evaluator does not yet serve \
             (no exercised vector or content rule needs it); slice 1 serves \
             the unpredicated (nodes <enum-ref>) form only",
        ));
    }
    let node_type = enum_member(type_ref)?;
    let graph = require_graph(env, "nodes")?;
    Ok(graph
        .nodes(node_type)
        .into_iter()
        .map(Element::Node)
        .collect())
}

/// `(neighbors <expr> <EdgeType> <direction> <NodeType>)` — D51's mandatory
/// four-operand form. The `NodeType` operand **filters** (D24): a neighbour
/// reached across the named edge type that is not of the annotated type is
/// simply not in the result (§2.6). `GraphSubstrate::neighbors` already
/// returns a sorted, deduplicated `Vec` (D72's set semantics), so filtering
/// it preserves both properties — no re-collection into a hash container.
fn materialize_neighbors(
    items: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Vec<Element>, EvalError> {
    charge(fuel, cost::QUERY_BASE)?;
    let [_, source_expr, edge_ref, direction_kw, node_ref] = items else {
        return Err(EvalError::plain(
            "(neighbors <expr> <EdgeType> <direction> <NodeType>) — the \
             NodeType fourth operand is MANDATORY (D51; a three-operand form \
             is E-PARSE-042 at load, not a silent bound)",
        ));
    };
    let source = match evaluate(source_expr, env, host, fuel)? {
        Value::NodeRef(id) => id,
        other => {
            return Err(EvalError::plain(format!(
                "(neighbors …)'s first operand must evaluate to a NodeRef, \
                 got {other:?}"
            )))
        }
    };
    let edge_type = enum_member(edge_ref)?;
    let direction = direction_of(direction_kw)?;
    let result_type = enum_member(node_ref)?;
    let graph = require_graph(env, "neighbors")?;
    let neighbor_ids = graph.neighbors(source, edge_type, direction).map_err(|e| {
        // D-row Q3: a query OPERAND naming no live element has no E-EVAL
        // code — §2.6 codes the annotation mismatch (E-EVAL-032) and §2.10
        // codes accessor referents (E-EVAL-033), but the reference codes no
        // query-operand case. Uncoded loud, per the crate's standing
        // "no invented codes" precedent, until the D-row lands.
        EvalError::plain(format!(
            "(neighbors …): {} — a dangling NodeRef never reads as an empty \
             neighborhood (§2.6 honest-null discipline); D-row Q3 (query-\
             evaluation plan) — no E-EVAL code is minted for this case yet",
            e.message
        ))
    })?;
    let mut out = Vec::with_capacity(neighbor_ids.len());
    for id in neighbor_ids {
        // Defense in depth: `neighbor_ids` came from the substrate's own
        // `neighbors()`, whose contract only ever returns live node ids, so
        // `node_type_of` should never fail here. Reaching the error arm
        // would mean the substrate's own contract broke, not a content bug.
        let node_type = graph.node_type_of(id).map_err(|e| {
            EvalError::plain(format!(
                "(neighbors …): a neighbour returned by the substrate is not \
                 itself readable as a live node ({}) — a substrate contract \
                 violation, not a content error",
                e.message
            ))
        })?;
        if node_type == result_type {
            out.push(Element::Node(id));
        }
    }
    Ok(out)
}

/// `(edges <enum-ref> <edge-pred>?)`. Like `nodes`' `<node-pred>`, the `<edge-pred>` operand is a
/// real §2.6 grammar production with zero exercised conformance vectors and zero content rules
/// (T2 scout dossier §1.1 point 4) — refused loudly by name, mirroring `materialize_nodes` exactly,
/// rather than served on an unreviewed reading.
///
/// **Performs no sort of its own.** `GraphSubstrate::edges` already returns a canonically sorted
/// `Vec<(NodeId, NodeId)>` (both backends' `sort_unstable()`, before this function ever runs) — this
/// maps that ALREADY-ordered output element-for-element; `EdgeKey`'s own `Ord` derive is never
/// consulted here — its field-order choice serves the derive's spec-agreement alone (see
/// [`EdgeKey`]'s own doc), and `tests::edges_materializes_in_exactly_graph_edges_own_order` proves
/// the delegation directly.
fn materialize_edges(
    items: &[SExpr],
    env: &EvalEnv<'_>,
    fuel: &mut u64,
) -> Result<Vec<Element>, EvalError> {
    charge(fuel, cost::QUERY_BASE)?;
    let [_, type_ref, extra @ ..] = items else {
        return Err(EvalError::plain(
            "(edges <enum-ref> <edge-pred>?) — missing the EdgeType operand",
        ));
    };
    if !extra.is_empty() {
        return Err(EvalError::plain(
            "(edges <enum-ref> <edge-pred>) — the element-predicate operand is a real §2.6 \
             production this evaluator does not yet serve (no exercised vector or content rule \
             needs it); T2 serves the unpredicated (edges <enum-ref>) form only",
        ));
    }
    let edge_type = enum_member(type_ref)?;
    let graph = require_graph(env, "edges")?;
    Ok(graph
        .edges(edge_type)
        .into_iter()
        .map(|(source, target)| {
            Element::Edge(EdgeKey {
                source,
                target,
                edge_type: edge_type.to_owned(),
            })
        })
        .collect())
}

/// `(hyperedges <enum-ref>)` — every hyperedge of the type, as
/// `Element::Hyperedge` in ascending `HyperedgeId` order (the substrate's
/// own contract — `hyperedges()` is ascending-and-typed by the conformance
/// suite — §2.6's total order is the substrate's, not re-established here).
/// No predicate variant exists for this head today (same deliberate
/// loud-gap posture as `nodes`/`edges` above).
fn materialize_hyperedges(
    items: &[SExpr],
    env: &EvalEnv<'_>,
    fuel: &mut u64,
) -> Result<Vec<Element>, EvalError> {
    charge(fuel, cost::QUERY_BASE)?;
    let [_, type_ref, extra @ ..] = items else {
        return Err(EvalError::plain(
            "(hyperedges <enum-ref>) — missing the HyperedgeType operand",
        ));
    };
    if !extra.is_empty() {
        return Err(EvalError::plain(
            "(hyperedges <enum-ref>) takes exactly one operand — no predicate variant exists \
             for this head today (the deliberate loud-gap posture `nodes`/`edges` set)",
        ));
    }
    let hyperedge_type = enum_member(type_ref)?;
    let graph = require_graph(env, "hyperedges")?;
    Ok(graph
        .hyperedges(hyperedge_type)
        .into_iter()
        .map(Element::Hyperedge)
        .collect())
}

/// `(hyperedges-of <elem-expr> <enum-ref>)` — the hyperedges of the type a
/// NODE belongs to, as `Element::Hyperedge` in ascending `HyperedgeId`
/// order (the substrate's contract, same as above).
fn materialize_hyperedges_of(
    items: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Vec<Element>, EvalError> {
    charge(fuel, cost::QUERY_BASE)?;
    let [_, elem_expr, type_ref] = items else {
        return Err(EvalError::plain(
            "(hyperedges-of <elem-expr> <enum-ref>) — exactly two operands",
        ));
    };
    let node = match evaluate(elem_expr, env, host, fuel)? {
        Value::NodeRef(id) => id,
        other => {
            return Err(EvalError::plain(format!(
                "(hyperedges-of …)'s first operand must evaluate to a NodeRef, got \
                 {other:?} (§3.1)"
            )))
        }
    };
    let hyperedge_type = enum_member(type_ref)?;
    let graph = require_graph(env, "hyperedges-of")?;
    let ids = graph.hyperedges_of(node, hyperedge_type).map_err(|e| {
        // Same posture as `neighbors`' own operand error (D-row Q3, this
        // module): the reference codes no query-operand case, so this is
        // uncoded loud, never a default.
        EvalError::plain(format!(
            "(hyperedges-of …) over a dangling node operand: {}",
            e.message
        ))
    })?;
    Ok(ids.into_iter().map(Element::Hyperedge).collect())
}

/// `(members-of <elem-expr> <enum-ref>)` — the members of the hyperedge the
/// first operand names, as plain `Element::Node` in ascending `NodeId`
/// order (D25 — `members_of` is already ascending by substrate contract;
/// declared order is never observable).
fn materialize_members_of(
    items: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Vec<Element>, EvalError> {
    charge(fuel, cost::QUERY_BASE)?;
    let [_, elem_expr, type_ref] = items else {
        return Err(EvalError::plain(
            "(members-of <elem-expr> <enum-ref>) — exactly two operands",
        ));
    };
    let hyperedge = match evaluate(elem_expr, env, host, fuel)? {
        Value::HyperedgeRef(id) => id,
        other => {
            return Err(EvalError::plain(format!(
                "(members-of …)'s first operand must evaluate to a HyperedgeRef, got \
                 {other:?} (§3.1)"
            )))
        }
    };
    let hyperedge_type = enum_member(type_ref)?;
    let graph = require_graph(env, "members-of")?;
    // E-EVAL-032 (D24) — the annotated type must match the referent's
    // ACTUAL type: the load-time bound checker bounds this fold by the
    // annotated type's census-fed max-members, and a referent of a
    // different type would iterate past that bound unchecked (PR #684's
    // review found the gap). Never a silently empty set.
    let actual_type = graph.hyperedge_type_of(hyperedge).map_err(|e| {
        // D-row Q3's posture, same module: uncoded loud.
        EvalError::plain(format!(
            "(members-of …) over a dangling hyperedge operand: {}",
            e.message
        ))
    })?;
    if actual_type != hyperedge_type {
        return Err(EvalError::coded(
            crate::evaluator::EvalCode::HyperedgeTypeMismatch,
            format!(
                "(members-of …) annotated {hyperedge_type} but the referent is {actual_type} — \
                 the annotated type bounds the fold's fuel (D24), never a silently empty set"
            ),
        ));
    }
    Ok(graph
        .members_of(hyperedge)
        .map_err(|e| {
            // D-row Q3's posture, same module: uncoded loud.
            EvalError::plain(format!(
                "(members-of …) over a dangling hyperedge operand: {}",
                e.message
            ))
        })?
        .into_iter()
        .map(Element::Node)
        .collect())
}

/// An enum-ref operand's member name — read directly, exactly as
/// `structural_verbs::EffectExecutor::enum_member` reads one, and NOT
/// through `evaluate()`: an enum-ref atom is a static type annotation here,
/// not a value the query computes with (`bound_checker`'s `atom_cost` charges
/// it 0 for the same reason). `pub(crate)` (T2, issue #559):
/// `evaluator::eval_edge_between` reuses this exact reading rather than
/// re-implementing it.
pub(crate) fn enum_member(expr: &SExpr) -> Result<&str, EvalError> {
    match expr {
        SExpr::Atom(Atom::EnumRef { member, .. }) => Ok(member),
        other => Err(EvalError::plain(format!(
            "expected an enum-ref where the grammar requires one, found {other:?}"
        ))),
    }
}

/// `<direction> ::= ":out" | ":in" | ":any"` (§2.6).
fn direction_of(expr: &SExpr) -> Result<Direction, EvalError> {
    match expr {
        SExpr::Atom(Atom::Keyword(kw)) if kw == "out" => Ok(Direction::Out),
        SExpr::Atom(Atom::Keyword(kw)) if kw == "in" => Ok(Direction::In),
        SExpr::Atom(Atom::Keyword(kw)) if kw == "any" => Ok(Direction::Any),
        other => Err(EvalError::plain(format!(
            "expected a direction (:out | :in | :any), found {other:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fuel::IntrinsicCosts;
    use crate::intrinsic_host::EmptyIntrinsicHost;
    use crate::reader::read;
    use babylon_graph::memory::MemoryGraph;
    use babylon_graph::substrate::GraphSubstrate;
    use std::collections::HashMap;

    fn costs() -> IntrinsicCosts {
        IntrinsicCosts::default()
    }

    fn env<'a>(graph: &'a dyn GraphSubstrate, costs: &'a IntrinsicCosts) -> EvalEnv<'a> {
        EvalEnv {
            bindings: HashMap::new(),
            intrinsic_costs: costs,
            graph: Some(graph),
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        }
    }

    fn materialize_src(
        source: &str,
        graph: &dyn GraphSubstrate,
        costs: &IntrinsicCosts,
        fuel: &mut u64,
    ) -> Result<Vec<Element>, EvalError> {
        let (expr, _) = read(source).expect("test source must parse");
        materialize(&expr, &env(graph, costs), &EmptyIntrinsicHost, fuel)
    }

    /// M2 (PR #514 fix-round finding): the ORIGINAL 3-node version of this
    /// test declared its nodes as `c, b, a` and claimed that exercised
    /// "descending declared order" — but `MemoryGraph::add_node` assigns
    /// `NodeId`s monotonically in CALL order (`next_id` only ever
    /// increments, and `remove_node`'s own doc records that ids are never
    /// reused), so `c` (added first) always gets the LOWEST id and `a`
    /// (added last) always gets the HIGHEST — insertion call order and
    /// ascending-id order can NEVER diverge for this store, no matter what
    /// symbolic name order the test author chooses. The old comment's claim
    /// ("the LAST-declared node gets the LOWEST id") was the opposite of
    /// what the fixture actually does, and a materializer that merely
    /// echoed insertion order would have passed the old test.
    ///
    /// What genuinely varies — and is worth guarding — is the underlying
    /// `HashMap`'s iteration order (`MemoryGraph::nodes` sorts it before
    /// returning, but nothing upstream of that sort is contractually
    /// ordered). Fifty nodes makes an accidental sorted match astronomically
    /// unlikely, so a `sort_unstable()` regression will show up here with
    /// overwhelming probability where a 3-element fixture would not — see
    /// this fix's commit body for the mutation evidence.
    #[test]
    fn nodes_materializes_in_ascending_id_order() {
        const N: usize = 50;
        let mut graph = MemoryGraph::new();
        for _ in 0..N {
            graph.add_node("SOCIAL_CLASS").unwrap();
        }
        let costs = costs();
        let mut fuel = 10_000;
        let result =
            materialize_src("(nodes NodeType/SOCIAL_CLASS)", &graph, &costs, &mut fuel).unwrap();
        let ids: Vec<NodeId> = result
            .iter()
            .map(|element| match element {
                Element::Node(id) => *id,
                Element::Edge(_) | Element::Hyperedge(_) => {
                    panic!("nodes must materialize only Node elements")
                }
            })
            .collect();
        assert_eq!(ids.len(), N);
        assert!(
            ids.windows(2).all(|w| w[0] < w[1]),
            "materialized nodes must be strictly ascending by id: {ids:?}"
        );
    }

    #[test]
    fn nodes_filters_by_the_annotated_type() {
        let mut graph = MemoryGraph::new();
        let sc = graph.add_node("SOCIAL_CLASS").unwrap();
        graph.add_node("ORGANIZATION").unwrap();
        let costs = costs();
        let mut fuel = 1_000;
        let result =
            materialize_src("(nodes NodeType/SOCIAL_CLASS)", &graph, &costs, &mut fuel).unwrap();
        assert_eq!(result, vec![Element::Node(sc)]);
    }

    /// D24: the `neighbors` result `NodeType` FILTERS. A node reachable via
    /// `TENANCY` that is `ORGANIZATION`-typed must not appear when the
    /// annotation reads `NodeType/SOCIAL_CLASS`.
    #[test]
    fn neighbors_filters_by_the_annotated_node_type() {
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        let tenant = graph.add_node("SOCIAL_CLASS").unwrap();
        let org = graph.add_node("ORGANIZATION").unwrap();
        graph.add_edge("TENANCY", tenant, subject, 1.0).unwrap();
        graph.add_edge("TENANCY", org, subject, 1.0).unwrap();
        let costs = costs();
        let mut fuel = 1_000;
        let bindings = HashMap::from([("self".to_owned(), Value::NodeRef(subject))]);
        let env = EvalEnv {
            bindings,
            intrinsic_costs: &costs,
            graph: Some(&graph as &dyn GraphSubstrate),
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let (expr, _) =
            read("(neighbors self EdgeType/TENANCY :in NodeType/SOCIAL_CLASS)").unwrap();
        let result = materialize(&expr, &env, &EmptyIntrinsicHost, &mut fuel).unwrap();
        assert_eq!(result, vec![Element::Node(tenant)]);
    }

    /// §6.2 family-17's multiplicity vector (D72): two qualifying edges (one
    /// `:out`, one `:in`) reaching one node under `:any` yield it ONCE —
    /// `neighbors` is a set, not a multiset.
    #[test]
    fn neighbors_is_a_set_not_a_multiset() {
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        let other = graph.add_node("SOCIAL_CLASS").unwrap();
        graph.add_edge("SOLIDARITY", subject, other, 1.0).unwrap();
        graph.add_edge("SOLIDARITY", other, subject, 1.0).unwrap();
        let costs = costs();
        let mut fuel = 1_000;
        let bindings = HashMap::from([("self".to_owned(), Value::NodeRef(subject))]);
        let env = EvalEnv {
            bindings,
            intrinsic_costs: &costs,
            graph: Some(&graph as &dyn GraphSubstrate),
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let (expr, _) =
            read("(neighbors self EdgeType/SOLIDARITY :any NodeType/SOCIAL_CLASS)").unwrap();
        let result = materialize(&expr, &env, &EmptyIntrinsicHost, &mut fuel).unwrap();
        assert_eq!(
            result,
            vec![Element::Node(other)],
            "one node, once, not twice"
        );
    }

    #[test]
    fn neighbors_of_a_dangling_node_is_loud_not_empty() {
        let graph = MemoryGraph::new();
        let costs = costs();
        let mut fuel = 1_000;
        let bindings = HashMap::from([("self".to_owned(), Value::NodeRef(NodeId(999)))]);
        let env = EvalEnv {
            bindings,
            intrinsic_costs: &costs,
            graph: Some(&graph as &dyn GraphSubstrate),
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let (expr, _) =
            read("(neighbors self EdgeType/SOLIDARITY :any NodeType/SOCIAL_CLASS)").unwrap();
        let err = materialize(&expr, &env, &EmptyIntrinsicHost, &mut fuel).unwrap_err();
        assert_eq!(err.code, None, "D-row Q3: uncoded loud, not E-EVAL-031");
        assert!(err.message.contains("D-row Q3"), "{err}");
        assert!(err.message.contains("never reads as an empty"), "{err}");
    }

    #[test]
    fn query_materialization_charges_the_3_7_query_base() {
        let mut graph = MemoryGraph::new();
        graph.add_node("SOCIAL_CLASS").unwrap();
        let costs = costs();
        // (nodes <enum-ref>): QUERY_BASE(1) + enum-ref(0) = 1.
        let mut fuel = 10;
        materialize_src("(nodes NodeType/SOCIAL_CLASS)", &graph, &costs, &mut fuel).unwrap();
        assert_eq!(fuel, 9);

        // (neighbors self <EdgeType> <dir> <NodeType>):
        // QUERY_BASE(1) + self variable-ref(1) = 2.
        let subject = NodeId(0);
        let bindings = HashMap::from([("self".to_owned(), Value::NodeRef(subject))]);
        let env = EvalEnv {
            bindings,
            intrinsic_costs: &costs,
            graph: Some(&graph as &dyn GraphSubstrate),
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let (expr, _) =
            read("(neighbors self EdgeType/SOLIDARITY :any NodeType/SOCIAL_CLASS)").unwrap();
        let mut fuel2 = 10;
        materialize(&expr, &env, &EmptyIntrinsicHost, &mut fuel2).unwrap();
        assert_eq!(fuel2, 8);
    }

    #[test]
    fn edges_materializes_in_ascending_source_target_order_and_charges_query_base() {
        let mut graph = MemoryGraph::new();
        let a = graph.add_node("SOCIAL_CLASS").unwrap();
        let b = graph.add_node("SOCIAL_CLASS").unwrap();
        let c = graph.add_node("SOCIAL_CLASS").unwrap();
        // Insertion order deliberately SHUFFLED against ascending (source, target).
        graph.add_edge("SOLIDARITY", c, a, 0.9).unwrap();
        graph.add_edge("SOLIDARITY", a, b, 0.1).unwrap();
        graph.add_edge("SOLIDARITY", a, c, 0.5).unwrap();
        let costs = costs();
        let mut fuel = 10;
        let result =
            materialize_src("(edges EdgeType/SOLIDARITY)", &graph, &costs, &mut fuel).unwrap();
        let keys: Vec<EdgeKey> = result
            .iter()
            .map(|element| match element {
                Element::Edge(key) => key.clone(),
                Element::Node(_) | Element::Hyperedge(_) => {
                    panic!("edges must materialize only Edge elements")
                }
            })
            .collect();
        assert_eq!(
            keys,
            vec![
                EdgeKey {
                    source: a,
                    target: b,
                    edge_type: "SOLIDARITY".to_owned()
                },
                EdgeKey {
                    source: a,
                    target: c,
                    edge_type: "SOLIDARITY".to_owned()
                },
                EdgeKey {
                    source: c,
                    target: a,
                    edge_type: "SOLIDARITY".to_owned()
                },
            ],
            "ascending (source-id, target-id) — §2.6; materialize_edges performs no sort of its \
             own, it maps graph.edges()'s ALREADY-sorted output (see design decision 1 above)"
        );
        // QUERY_BASE(1) + enum-ref(0) = 1.
        assert_eq!(fuel, 9);
    }

    #[test]
    fn edges_materializes_in_ascending_source_target_order_at_scale() {
        const N: usize = 50;
        let mut graph = MemoryGraph::new();
        let mut ids = Vec::with_capacity(N);
        for _ in 0..N {
            ids.push(graph.add_node("SOCIAL_CLASS").unwrap());
        }
        // Add edges over a SHUFFLED pairing of the id space (every id i -> id (i*17+3) % N,
        // a fixed permutation with no monotonic relationship to insertion/id order) so the
        // resulting edge set's ascending order cannot coincide with any single simple pattern
        // the loop itself might accidentally produce.
        for (i, &from) in ids.iter().enumerate() {
            let to = ids[(i * 17 + 3) % N];
            if from != to {
                graph.add_edge("SOLIDARITY", from, to, 0.1).unwrap();
            }
        }
        let costs = costs();
        let mut fuel = 10_000;
        let result =
            materialize_src("(edges EdgeType/SOLIDARITY)", &graph, &costs, &mut fuel).unwrap();
        let pairs: Vec<(NodeId, NodeId)> = result
            .iter()
            .map(|element| match element {
                Element::Edge(key) => (key.source, key.target),
                Element::Node(_) | Element::Hyperedge(_) => {
                    panic!("edges must materialize only Edge elements")
                }
            })
            .collect();
        assert!(
            pairs.windows(2).all(|w| w[0] < w[1]),
            "materialized edges must be strictly ascending by (source, target): {pairs:?}"
        );
    }

    /// `EdgeKey`'s own law, directly: field order is `(source, target, edge_type)`, so `source`
    /// dominates `edge_type` in comparison — proven with a pair CONSTRUCTED so the two possible
    /// field orderings DISAGREE (a lower source but alphabetically LATER `edge_type` vs. a higher
    /// source but alphabetically EARLIER `edge_type`), making the field-declaration choice
    /// mutation-provable. This is independent of `materialize_edges` (design decision 1) — the
    /// direct test this crate's own trip-wire has been asking for since `Element`'s derive doc was
    /// written.
    #[test]
    fn edge_key_ord_prioritizes_source_over_edge_type() {
        let lower_source_higher_type = EdgeKey {
            source: NodeId(1),
            target: NodeId(2),
            edge_type: "ZZZZ".to_owned(),
        };
        let higher_source_lower_type = EdgeKey {
            source: NodeId(2),
            target: NodeId(1),
            edge_type: "AAAA".to_owned(),
        };
        assert!(
            lower_source_higher_type < higher_source_lower_type,
            "source must dominate edge_type — §2.6's (source-id, target-id, edge-type) order"
        );
    }

    /// `materialize_edges`' own ordering guarantee comes from `GraphSubstrate::edges`' own
    /// contract, NOT from `EdgeKey`'s derived Ord (which this function never invokes) — proven by
    /// direct equality against the substrate's own output, not merely by eyeballing one
    /// hand-picked fixture (the small exact-triple test above).
    #[test]
    fn edges_materializes_in_exactly_graph_edges_own_order() {
        let mut graph = MemoryGraph::new();
        let a = graph.add_node("SOCIAL_CLASS").unwrap();
        let b = graph.add_node("SOCIAL_CLASS").unwrap();
        let c = graph.add_node("SOCIAL_CLASS").unwrap();
        graph.add_edge("SOLIDARITY", c, a, 0.9).unwrap();
        graph.add_edge("SOLIDARITY", a, b, 0.1).unwrap();
        graph.add_edge("SOLIDARITY", a, c, 0.5).unwrap();
        let costs = costs();
        let mut fuel = 10;
        let result =
            materialize_src("(edges EdgeType/SOLIDARITY)", &graph, &costs, &mut fuel).unwrap();
        let materialized: Vec<(NodeId, NodeId)> = result
            .iter()
            .map(|element| match element {
                Element::Edge(key) => (key.source, key.target),
                Element::Node(_) | Element::Hyperedge(_) => {
                    panic!("edges must materialize only Edge elements")
                }
            })
            .collect();
        assert_eq!(
            materialized,
            graph.edges("SOLIDARITY"),
            "materialize_edges must reproduce GraphSubstrate::edges' own order exactly, unchanged"
        );
    }

    #[test]
    fn a_predicated_nodes_query_is_a_loud_named_gap() {
        let graph = MemoryGraph::new();
        let costs = costs();
        let mut fuel = 1_000;
        let err = materialize_src(
            "(nodes NodeType/SOCIAL_CLASS #t)",
            &graph,
            &costs,
            &mut fuel,
        )
        .unwrap_err();
        assert!(err.message.contains("element-predicate"), "{err}");
    }

    /// The `edges` twin of the predicated-`nodes` pin above (full-PR
    /// verification, MINOR-1): the `<edge-pred>` refusal branch had ZERO
    /// coverage — deleting it left the whole workspace green — and the
    /// load gate does NOT save it (`grammar.rs`'s `ARITIES` admits arity 2
    /// for `edges`), so the branch is reachable and was unpinned, against
    /// the sentinel-every-error-class rule.
    #[test]
    fn a_predicated_edges_query_is_a_loud_named_gap() {
        let graph = MemoryGraph::new();
        let costs = costs();
        let mut fuel = 1_000;
        let err = materialize_src("(edges EdgeType/SOLIDARITY #t)", &graph, &costs, &mut fuel)
            .unwrap_err();
        assert!(err.message.contains("element-predicate"), "{err}");
    }

    /// RED→GREEN record (slice 3, Community port train Task 3, 2026-08-21):
    /// this test pinned the three heads' "lands with slice 3" refusal text
    /// while they were unserved; the heads are SERVED now (the
    /// materializers above), so the test inverts — the refusal text it
    /// pinned survives in this comment as the historical record.
    #[test]
    fn hyperedges_and_members_of_are_served_in_query_position() {
        let mut graph = MemoryGraph::new();
        let alpha = graph.add_node("social_class").unwrap();
        let beta = graph.add_node("social_class").unwrap();
        let cell = graph.add_hyperedge("CELL", &[alpha, beta]).unwrap();
        let costs = costs();
        let mut fuel = 10_000;

        let result = materialize_src("(hyperedges HyperedgeType/CELL)", &graph, &costs, &mut fuel)
            .expect("the type-wide head is served");
        assert_eq!(result, vec![Element::Hyperedge(cell)]);
        // PR #684 review: independent assertions get independent fuel — a
        // shared meter would couple the ordering pins to prior calls'
        // consumption.

        // Iteration order is the ruled order — ascending HyperedgeId (D25),
        // pinned against a deliberately multi-hyperedge world; the Step-7
        // mutation (inverting the materializer's order) reds this.
        let cell2 = graph.add_hyperedge("CELL", &[beta]).unwrap();
        let cell0 = graph.add_hyperedge("CELL", &[alpha]).unwrap();
        let mut fuel = 10_000;
        let result = materialize_src("(hyperedges HyperedgeType/CELL)", &graph, &costs, &mut fuel)
            .expect("the type-wide head is served");
        assert_eq!(
            result,
            vec![
                Element::Hyperedge(cell),
                Element::Hyperedge(cell2),
                Element::Hyperedge(cell0)
            ],
            "ascending HyperedgeId, never creation/insertion order"
        );

        let result = materialize_src(
            "(members-of self HyperedgeType/CELL)",
            &graph,
            &costs,
            &mut fuel,
        )
        .expect_err("a bare `self` outside a rule body has no referent — loud");
        assert!(result.message.contains("self"), "{result}");

        let mut fuel = 10_000;
        let members = materialize_src(
            "(members-of it HyperedgeType/CELL)",
            &graph,
            &costs,
            &mut fuel,
        )
        .expect_err("a bare `it` outside an iterating body has no referent — loud");
        assert!(members.message.contains("it"), "{members}");

        // E-EVAL-032 (D24; PR #684 review): the annotated type must match
        // the referent's ACTUAL type — the load-time bound is the annotated
        // type's census-fed max-members, so a mismatch would iterate past
        // the declared bound unchecked.
        let mut env = env(&graph, &costs);
        env.elements
            .push((Some("c".to_owned()), Element::Hyperedge(cell)));
        let mut fuel = 10_000;
        let err = materialize(
            &read("(members-of c HyperedgeType/OTHER)")
                .expect("parses")
                .0,
            &env,
            &EmptyIntrinsicHost,
            &mut fuel,
        )
        .expect_err("a COMMUNITY referent annotated OTHER must refuse");
        assert!(
            err.message
                .contains("annotated OTHER but the referent is CELL"),
            "{err}"
        );
        assert_eq!(
            err.code,
            Some(crate::evaluator::EvalCode::HyperedgeTypeMismatch)
        );
    }

    /// Compile-time trap (verifier fix round, MINOR-5 on issue #525): an
    /// EXHAUSTIVE match over `Element`, no wildcard. The prose warning on
    /// `Element`'s own derive is not, by itself, a mechanical guarantee
    /// that anyone reads it before adding a new variant (`Hyperedge`,
    /// slice 3); this function breaks COMPILATION, at this exact test, the
    /// moment a new variant lands — forcing the cross-kind order to be
    /// pinned against the spec before the code can even build, not merely
    /// before a reviewer happens to notice. (T2, issue #559: `Edge` landed
    /// exactly this way — the cross-kind ruling is on the enum's own doc,
    /// pinned by `node_sorts_before_edge_regardless_of_id` below.) Takes
    /// `&Element` now that `Element` is no longer `Copy`.
    fn element_kind_name(element: &Element) -> &'static str {
        match element {
            Element::Node(_) => "node",
            Element::Edge(_) => "edge",
            Element::Hyperedge(_) => "hyperedge",
        }
    }

    /// CT4P A5 (issue #525): within one kind, `Element` ordering reduces
    /// exactly to `NodeId`'s own order — see the derive's own doc for the
    /// ruling this pins against. [`element_kind_name`] above is the
    /// mechanical trip-wire; this test is the value-level pin.
    #[test]
    fn element_ordering_matches_ascending_node_id() {
        let a = Element::Node(NodeId(1));
        let b = Element::Node(NodeId(2));
        assert_eq!(element_kind_name(&a), "node");
        assert!(a < b);
        assert!(b >= a);
        assert_eq!(Element::Node(NodeId(7)), Element::Node(NodeId(7)));
    }

    /// T2's cross-kind Ord ruling, value-pinned: Node < Edge regardless of id/key magnitude.
    #[test]
    fn node_sorts_before_edge_regardless_of_id() {
        let node = Element::Node(NodeId(u64::MAX));
        let edge = Element::Edge(EdgeKey {
            source: NodeId(0),
            target: NodeId(0),
            edge_type: String::new(),
        });
        assert_eq!(element_kind_name(&node), "node");
        assert_eq!(element_kind_name(&edge), "edge");
        assert!(node < edge, "T2's ruling: kind dominates value");
    }

    /// Slice 3's cross-kind Ord ruling, value-pinned: Edge < Hyperedge
    /// regardless of id/key magnitude (the enum's own standing instruction,
    /// extended — arbitrary, deliberate, tested).
    #[test]
    fn edge_sorts_before_hyperedge_regardless_of_id() {
        let edge = Element::Edge(EdgeKey {
            source: NodeId(u64::MAX),
            target: NodeId(u64::MAX),
            edge_type: String::from("z"),
        });
        let hyperedge = Element::Hyperedge(HyperedgeId(0));
        assert!(edge < hyperedge, "slice-3 ruling: kind dominates value");
    }

    /// The three-kind chain: Node < Edge < Hyperedge, all by declaration
    /// order, regardless of the carried ids' magnitudes.
    #[test]
    fn the_three_kind_order_holds_regardless_of_ids() {
        let node = Element::Node(NodeId(u64::MAX));
        let edge = Element::Edge(EdgeKey {
            source: NodeId(u64::MAX),
            target: NodeId(u64::MAX),
            edge_type: String::from("z"),
        });
        let hyperedge = Element::Hyperedge(HyperedgeId(0));
        assert!(node < edge && edge < hyperedge, "Node < Edge < Hyperedge");
    }
}
