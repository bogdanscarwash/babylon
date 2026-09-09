//! Shared helpers for `babylon-tick`'s conformance test suite (theme 6,
//! `docs/superpowers/plans/2026-08-18-bsl-refactor-program.md` §4 Task
//! R2.1).
//!
//! Lives at `tests/support/mod.rs` — Rust's own convention for an
//! integration-test helper module that is NOT itself compiled as a
//! separate test binary (unlike every sibling `tests/*.rs` file, a
//! `mod.rs` under a subdirectory is visible only to files that declare
//! `mod support;` themselves).
//!
//! **Genuinely-found shapes only.** `rust/crates/babylon-tick/tests/`
//! comprises 31 files (12,158 lines) sharing a `run_once_into`/
//! `CollectingSink`/`node_attribute` trio 315+ times, but NOT one
//! identical shape — direct reads found three distinct `attribute()`
//! signatures (a `MemoryGraph`-keyed-by-`u64` majority, e.g.
//! `dispossession_conformance.rs:74`; a `HypergraphStore`-keyed-by-`NodeId`
//! variant, e.g. `territory_conformance.rs:169`,
//! `control_ratio_conformance.rs:460`; and a `HypergraphStore`-keyed-by-
//! `u64` variant, `multi_rule_conformance.rs:146`), three distinct `run_*`
//! return shapes (below), and at least one independently-renamed pin-table
//! struct (`ExpectedTerritory` in `multi_rule_conformance.rs:71`, diffed
//! field-for-field against `lifecycle_conformance.rs:110`'s and
//! `lifecycle_crisis_conformance.rs:84`'s own `Expected` and confirmed
//! identical). Every item below generalizes from a genuinely repeated
//! shape, cited at its definition — nothing here forces a merge the source
//! files didn't already share.
//!
//! **Pure addition (R2.1.3).** Landing this module changes NOTHING about
//! the 31 existing files; sweeping them onto it is a named, explicitly
//! deferred follow-up past Checkpoint A — several are under active edit by
//! concurrent port trains this week, and touching them now would maximize
//! rebase conflict for a cosmetic win. `support_smoke_conformance.rs` is
//! this module's own adopter; `metabolism_entropy_high_conformance.rs` is
//! migrated onto it as the proof-of-adoption (mechanical, behavior-
//! identical — same fixture, same assertions, same pass/fail contract).

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::allocator_state::AllocatorState;
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_graph::working_copy::DetachedCopy;
use babylon_tick::{run_once, run_once_into, TickReport};

/// Accepts either a raw `u64` (the majority shape across existing
/// conformance files, which wrap `NodeId(id)` at the call site) or a
/// [`NodeId`] directly (the `HypergraphStore`-keyed shape, e.g.
/// `territory_conformance.rs`'s own `heat(graph, id: NodeId)`), so
/// [`attribute`] below subsumes both without forcing every existing call
/// site to change its id literals.
///
/// A local trait, not `std::convert::Into<NodeId>`: the orphan rule
/// forbids implementing a foreign trait (`Into`) for a foreign type
/// (`NodeId`, defined in `babylon-graph`) from this crate's test code —
/// this trait is local to `babylon-tick`, so the same two impls are legal
/// here where they would not be as a bare `Into<NodeId>` bound.
pub trait IntoNodeId {
    fn into_node_id(self) -> NodeId;
}

impl IntoNodeId for u64 {
    fn into_node_id(self) -> NodeId {
        NodeId(self)
    }
}

impl IntoNodeId for NodeId {
    fn into_node_id(self) -> NodeId {
        self
    }
}

/// Read one `f64` attribute, panicking with the node id and field name on
/// a missing node or an honest-null unread field — the exact
/// `graph.node_attribute(id, field).unwrap_or_else(|e| panic!(...))` idiom
/// duplicated verbatim across every one of the 31 conformance/e2e files
/// (e.g. `dispossession_conformance.rs:74`, `lifecycle_conformance.rs:181`,
/// `metabolism_entropy_high_conformance.rs:37`). Generic over
/// [`GraphSubstrate`] (the trait `node_attribute` is declared on, taking a
/// [`NodeId`] regardless of the concrete store) so it subsumes both the
/// `MemoryGraph`-keyed and `HypergraphStore`-keyed call sites, and, via
/// [`IntoNodeId`], both id shapes genuinely found.
pub fn attribute<G: GraphSubstrate>(graph: &G, id: impl IntoNodeId, field: &str) -> f64 {
    let id = id.into_node_id();
    graph
        .node_attribute(id, field)
        .unwrap_or_else(|e| panic!("node {id:?} field {field}: {}", e.message))
}

/// Run one scenario+rule pair once, discarding the sink and the
/// [`TickReport`], returning only the substrate — the `fn run() ->
/// MemoryGraph` shape shared byte-for-byte across seven files
/// (`metabolism_conformance.rs`, `metabolism_ceiling_conformance.rs`,
/// `metabolism_ceiling_suppression_conformance.rs`,
/// `metabolism_ratcheted_ceiling_conformance.rs`,
/// `metabolism_rounding_divergence_conformance.rs`,
/// `metabolism_entropy_high_conformance.rs`,
/// `metabolism_entropy_low_conformance.rs`) — the majority shape among the
/// three `run_*` variants genuinely found (the other two below).
pub fn run_conformance<
    G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy + Default,
>(
    scenario: &str,
    rule: &str,
) -> G {
    let mut graph = G::default();
    let mut sink = CollectingSink::default();
    run_once_into(scenario, rule, &mut graph, &mut sink).expect("the pack must run");
    graph
}

/// Run once, keeping the [`CollectingSink`] and discarding the
/// [`TickReport`] — the `fn run() -> (MemoryGraph, CollectingSink)` /
/// `(HypergraphStore, CollectingSink)` shape shared across
/// `dispossession_conformance.rs` and its five sibling
/// `dispossession_*_conformance.rs` files, `lifecycle_conformance.rs`,
/// `lifecycle_crisis_conformance.rs`, `vitality_conformance.rs`, and
/// `decomposition_conformance.rs`'s own `HypergraphStore` variant.
pub fn run_conformance_with_sink<
    G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy + Default,
>(
    scenario: &str,
    rule: &str,
) -> (G, CollectingSink) {
    let mut graph = G::default();
    let mut sink = CollectingSink::default();
    run_once_into(scenario, rule, &mut graph, &mut sink).expect("the pack must run");
    (graph, sink)
}

/// Run once, keeping the [`TickReport`] and discarding the
/// [`CollectingSink`] — the `fn run_production() -> (HypergraphStore,
/// TickReport)` / `fn run_territory() -> (HypergraphStore, TickReport)`
/// shape shared by `production_conformance.rs` and
/// `territory_conformance.rs`, the third `run_*` variant genuinely found.
pub fn run_conformance_with_report<
    G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy + Default,
>(
    scenario: &str,
    rule: &str,
) -> (G, TickReport) {
    let mut graph = G::default();
    let mut sink = CollectingSink::default();
    let report = run_once_into(scenario, rule, &mut graph, &mut sink).expect("the pack must run");
    (graph, report)
}

/// One subject's expected value for one field — the pin-table row shape,
/// pivoted from the byte-identical `struct Expected { name, id, pop_d,
/// pop_p, ... }` (10 fields) duplicated verbatim across
/// `lifecycle_conformance.rs` and `lifecycle_crisis_conformance.rs`
/// (confirmed by a direct diff of the struct bodies) and independently
/// renamed to `ExpectedTerritory` in `multi_rule_conformance.rs` (same
/// fields, same order, confirmed field-identical by the same diff).
///
/// Kept LONG (one row per subject-field pair) rather than reproducing that
/// domain-specific 8-field struct verbatim: the fixed field list (`pop_d`,
/// `wealth_d_prime`, ...) is lifecycle/territory-specific, not a shape
/// every future adopter shares — forcing every domain through that exact
/// struct would be the "merge that papers over a real divergence" R2.1.2
/// warns against. A future adopter with its own field set (metabolism's
/// `biocapacity`/`max-biocapacity`, say) uses this same row shape with its
/// own field-path strings; nothing about `ExpectedField` itself is
/// lifecycle-specific.
pub struct ExpectedField {
    pub name: &'static str,
    pub id: u64,
    pub field: &'static str,
    pub value: f64,
}

/// Assert every row of a pin table against a live substrate — the
/// `for e in &EXPECTED { assert_eq!(attribute(&graph, e.id, "..."), e.pop_d,
/// "{}: pop-d", e.name); ... }` idiom's body (`lifecycle_conformance.rs`
/// lines 191-236), generalized over the field-name/value pair carried in
/// each [`ExpectedField`] row instead of one hardcoded field per line.
///
/// **Tolerance policy (CLAUDE.md rule 4): exact, not epsilon.** Every
/// existing pin-table test in this crate asserts `f64` equality with NO
/// tolerance (`dispossession_conformance.rs`'s own doc comment: "against
/// the frozen engine's own numbers, exactly (no tolerance)"), and that is
/// correct rather than sloppy: these are the frozen Python engine's own
/// literals, reproduced through basic IEEE-754 ops only (`+`/`-`/`*`/`/`,
/// no libm transcendentals). `dispossession_conformance.rs`'s header
/// comment states the derivation — a Rust float literal parses to the same
/// nearest-representable `f64` a Python `repr()` printed, so the two
/// languages agree on the LITERAL, and basic arithmetic on agreed literals
/// reproduces the same bit pattern in both languages
/// (`metabolism_rounding_divergence_conformance.rs` asserts this down to
/// the raw bit pattern, `0x3ff6666666666666` vs `...668`). An epsilon here
/// would MASK a real divergence rather than tolerate a benign one, so
/// `assert_eq!` stays exact — matching every call site this generalizes
/// from.
pub fn assert_expected<G: GraphSubstrate>(graph: &G, expected: &[ExpectedField]) {
    for e in expected {
        assert_eq!(
            attribute(graph, e.id, e.field),
            e.value,
            "{}: {}",
            e.name,
            e.field
        );
    }
}

/// The double-run-compare-hash idiom — `let a = run_once(scenario, rule)
/// .expect("first run"); let b = run_once(scenario, rule).expect("second
/// run"); assert_eq!(a.after, b.after);` — duplicated ~6 lines at a time
/// 19+ times across this crate's conformance suites (e.g.
/// `metabolism_entropy_high_conformance.rs:73-77`,
/// `dispossession_conformance.rs:264-267`,
/// `lifecycle_conformance.rs:332-335`).
///
/// Returns the first run's [`TickReport`] so callers keep their own
/// trailing assertion — `assert_eq!(report.fired, N)` at some call sites,
/// `assert_ne!(report.before, report.after)` at others, sometimes both with
/// call-site-specific messages. Those diverge genuinely (different rule
/// counts, different "the pack must move state" wording), so this helper
/// extracts only the part that is IDENTICAL everywhere — the double run and
/// the hash comparison — and does not force the rest into one shape.
pub fn assert_deterministic(scenario: &str, rule: &str) -> TickReport {
    let a = run_once(scenario, rule).expect("first run");
    let b = run_once(scenario, rule).expect("second run");
    assert_eq!(
        a.after, b.after,
        "two runs of the same scenario+rule must hash to the same post-tick state"
    );
    a
}
