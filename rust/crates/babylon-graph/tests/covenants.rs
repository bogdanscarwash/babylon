//! The delta document's §8 adapter covenants
//! (`docs/reference/graph-storage-capability-delta.md`), turned into a gate
//! — Phase C Task 9. Covenant 10 observes that the trait cannot enforce its
//! own ordering contract at compile time, so only tests can; this file is
//! where that observation becomes tests rather than discipline.
//!
//! **Coverage note.** Covenants 3 (loud preamble) and 9 (sort on the ruled
//! key, at every ranged accessor) are already exercised, exhaustively, by
//! `run_substrate_conformance(HypergraphStore::new)`
//! (`hypergraph_store.rs`'s own `#[cfg(test)]` module) — duplicating them
//! here would be the same assertions under a different name. This file adds
//! what that suite does not: source-level checks (covenants 1, 2, 6) and
//! HypergraphStore-specific behavioural checks the generic suite has no
//! vocabulary for (4, 5, 8). Covenant 7 (two stores, not one) has no test —
//! see the doc comment at the top of `hypergraph_store.rs`
//! ("Two data structures, not one").

use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::{GraphSubstrate, NodeId};

/// The adapter's own source — read once, checked several ways below. A
/// source-level check rather than a runtime one on purpose: these covenants
/// are about which METHODS the adapter calls, not about behaviour a black
/// box could exhibit either way.
const HYPERGRAPH_STORE_SOURCE: &str = include_str!("../src/hypergraph_store.rs");

/// Covenant 1: sole writer. The adapter must be the only entry point to the
/// library's ingest surface — `add_edges_from`, `add_node_to_edge`,
/// `SimplicialComplex`, `DiHypergraph` (the directed variant this adapter
/// never needs) must never appear, and the one call to the library's
/// `add_edge` must sit inside `add_hyperedge`, after the validation
/// preamble the same function begins with.
#[test]
fn covenant_1_sole_writer_no_excluded_library_surface_is_reached() {
    for forbidden in [
        "add_edges_from",
        "add_node_to_edge",
        "SimplicialComplex",
        "DiHypergraph",
        "convert::",
    ] {
        assert!(
            !HYPERGRAPH_STORE_SOURCE.contains(forbidden),
            "the adapter must never reach the excluded library surface `{forbidden}` \
             (delta §8 covenant 1)"
        );
    }

    // The library's add_edge is called exactly once, and it sits inside
    // add_hyperedge — the function whose body opens with the loud preamble.
    let add_hyperedge_start = HYPERGRAPH_STORE_SOURCE
        .find("fn add_hyperedge(")
        .expect("add_hyperedge must exist");
    let remove_hyperedge_start = HYPERGRAPH_STORE_SOURCE
        .find("fn remove_hyperedge(")
        .expect("remove_hyperedge must exist");
    let add_hyperedge_body = &HYPERGRAPH_STORE_SOURCE[add_hyperedge_start..remove_hyperedge_start];
    assert!(
        add_hyperedge_body.contains(".inner\n            .add_edge(")
            || add_hyperedge_body.contains(".inner.add_edge("),
        "the library's add_edge must be called from inside add_hyperedge"
    );
    // The dyadic add_edge (GraphSubstrate::add_edge, our own trait method
    // signature) and the library's add_edge share the substring "add_edge(" —
    // count occurrences of the LIBRARY call specifically via its receiver.
    let library_add_edge_calls = HYPERGRAPH_STORE_SOURCE.matches(".inner.add_edge(").count()
        + HYPERGRAPH_STORE_SOURCE
            .matches(".inner\n            .add_edge(")
            .count();
    assert_eq!(
        library_add_edge_calls, 1,
        "the library's add_edge must be called from exactly one place"
    );
}

/// Covenant 2: feature declaration. `default-features = false`, and
/// `generators` (which compiles a second, unguarded permissive ingest
/// surface into the build) must never be enabled.
#[test]
fn covenant_2_default_features_false_and_generators_never_enabled() {
    const CARGO_TOML: &str = include_str!("../Cargo.toml");
    let dep_line = CARGO_TOML
        .lines()
        .find(|line| line.trim_start().starts_with("hypergraph-rs ="))
        .expect("the hypergraph-rs dependency line must exist");
    assert!(
        dep_line.contains("default-features = false"),
        "hypergraph-rs must be pinned default-features = false: {dep_line}"
    );
    assert!(
        !dep_line.contains("generators"),
        "the generators feature must never be enabled: {dep_line}"
    );
}

/// Covenants 3 + 4 together, HypergraphStore-specific: the loud preamble
/// rejects an unknown member BEFORE the library ever sees it, which is what
/// makes the library's silent auto-create unreachable — "node universes
/// coincide" is exactly the claim that this rejection is sufficient, since
/// every member this adapter would accept was already minted into the
/// library at `add_node` time (covenant 4).
#[test]
fn covenant_3_and_4_unknown_member_is_loud_before_the_library_ever_sees_it() {
    let mut graph = HypergraphStore::new();
    let known = graph.add_node("social_class").unwrap();
    // NodeId(999_999) was never minted by add_node, so it exists in NEITHER
    // this adapter's nodes map NOR the library's agent registry (the two
    // are always minted together) — the preamble must catch it.
    let err = graph
        .add_hyperedge("economic_sector", &[known, NodeId(999_999)])
        .unwrap_err();
    assert!(!err.message.is_empty());
    // And the hyperedge must not have been partially minted — a second,
    // now-valid attempt gets a FRESH id, proving nothing leaked from the
    // rejected attempt.
    let id = graph.add_hyperedge("economic_sector", &[known]).unwrap();
    assert_eq!(id.0, 0, "the rejected attempt must not have consumed an id");
}

/// Covenant 5: a strength or attribute must never reach `M::default()` /
/// `N::default()` silently. The dyadic half never touches the library at
/// all (strength lives only in this adapter's own `edges` map), so the only
/// way this could fail is a strength or attribute write not round-tripping
/// exactly.
///
/// **Copilot review, PR #494 (D6-1): the original version of this test
/// asserted only that `edges("solidarity")` returned the right
/// `(NodeId, NodeId)` pair — that type carries no strength at all
/// (`GraphSubstrate::edges` returns endpoints only), so a strength silently
/// defaulting to `0.0` or `M::default()` would have passed. Fixed to read
/// the actual strength back through `CanonicalState::all_edges`, the same
/// listing the differential harness compares byte-for-byte, and assert
/// exact equality against the written value — the covenant this test is
/// named for.**
#[test]
fn covenant_5_strength_and_attributes_round_trip_exactly_never_a_default() {
    let mut graph = HypergraphStore::new();
    let a = graph.add_node("social_class").unwrap();
    let b = graph.add_node("social_class").unwrap();
    let written_strength = 0.123_456_789;
    graph
        .add_edge("solidarity", a, b, written_strength)
        .unwrap();
    graph.update_node(a, "wealth", 987.654_321).unwrap();

    // edges() only returns (NodeId, NodeId) — no strength — so existence
    // alone proves nothing about the covenant. Read the strength back
    // through the SAME listing the differential harness trusts.
    let read_strength = graph
        .all_edges()
        .into_iter()
        .find(|(edge_type, from, to, _)| edge_type == "solidarity" && *from == a && *to == b)
        .map(|(_, _, _, strength)| strength);
    assert_eq!(
        read_strength,
        Some(written_strength),
        "the strength must round-trip EXACTLY — a silent default or a \
         corrupted write would both pass a bare existence check"
    );
    assert!(
        (graph.node_attribute(a, "wealth").unwrap() - 987.654_321).abs() < f64::EPSILON,
        "the attribute must round-trip exactly, never default"
    );

    // T3 (ADR198 R1, issue #560): the same covenant on the fifth section —
    // a deffield-declared edge attribute lives in the adapter's own
    // `edge_attributes` map (the dyadic half never touches the library), so
    // the only failure shape is again a write that does not round-trip
    // exactly. Read back through `all_edge_attributes`, the listing the
    // differential harness compares byte-for-byte.
    let written_tension = 0.333_333_333_333_333_3;
    graph
        .update_edge("solidarity", a, b, "solidarity/tension", written_tension)
        .unwrap();
    let read_tension = graph
        .all_edge_attributes()
        .into_iter()
        .find(|(edge_type, from, to, name, _)| {
            edge_type == "solidarity" && *from == a && *to == b && name == "solidarity/tension"
        })
        .map(|(_, _, _, _, value)| value);
    assert_eq!(
        read_tension,
        Some(written_tension),
        "the edge attribute must round-trip EXACTLY through the fifth-section listing"
    );
}

/// Covenant 6: the frozen pre-check sits at the head of all 8 mutating
/// methods (the original seven plus T3's `update_edge`, ADR198 R1/R3 — the
/// list is hard-coded here by design, so a new mutating method must be
/// added BY HAND and its omission is loud at review, not silent).
/// Source-level, because nothing in this train ever sets the flag
/// (the check is defense against a future reachable path,
/// `hypergraph_store.rs`'s own doc comment on `check_not_frozen`), so a
/// runtime test would only ever exercise the flag being false.
#[test]
fn covenant_6_frozen_precheck_guards_all_eight_mutating_methods() {
    let mutating_methods = [
        "fn add_node(",
        "fn remove_node(",
        "fn add_edge(",
        "fn remove_edge(",
        "fn update_node(",
        "fn update_edge(",
        "fn add_hyperedge(",
        "fn remove_hyperedge(",
    ];
    for method in mutating_methods {
        let start = HYPERGRAPH_STORE_SOURCE
            .find(method)
            .unwrap_or_else(|| panic!("{method} must exist in hypergraph_store.rs"));
        // The check must appear within the first ~200 bytes of the method
        // body — "at the head", not merely somewhere in a large function.
        let window =
            &HYPERGRAPH_STORE_SOURCE[start..(start + 300).min(HYPERGRAPH_STORE_SOURCE.len())];
        assert!(
            window.contains("check_not_frozen()?"),
            "{method} must open with the frozen pre-check (delta §8 covenant 6)"
        );
    }
}

/// Covenant 8: deterministic bimap assignment. Two stores minting the same
/// sequence of node types must assign the identical `NodeId` sequence —
/// ascending-`NodeId` order is an observable contract, so the assignment
/// itself must be deterministic, not merely the read order.
#[test]
fn covenant_8_identical_mint_sequences_assign_identical_ids() {
    let make = || {
        let mut g = HypergraphStore::new();
        let ids: Vec<NodeId> = ["social_class", "territory", "social_class", "organization"]
            .iter()
            .map(|ty| g.add_node(ty).unwrap())
            .collect();
        ids
    };
    assert_eq!(make(), make());
}
