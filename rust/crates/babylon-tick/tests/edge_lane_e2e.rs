//! The three T2 slice-2 edge-lane end-to-end vectors (Program 29 train T2,
//! issue #559; plan `docs/superpowers/plans/2026-08-12-t2-slice2-edge-reads-
//! plan.md`, Task 6).
//!
//! **This ships no Solidarity content.** It proves the three slice-2 heads —
//! `edges`, `edge-between`, `field-of` over an `EdgeRef` — evaluate
//! correctly through the real production entry point
//! (`babylon_tick::run_once_into`, the same seam the CLI driver and
//! `babylon-client`'s engine link both call), over a hand-built fixture
//! (`content/scenarios/edge-lane-e2e.bscn`) anticipatory of Solidarity's own
//! read shape (`bsl-language.rst` §3.8 item 8's worked example) — the same
//! posture `query_lane_e2e.rs` took for Territory's shapes.
//!
//! Each shape below runs as its OWN single-rule content set, loaded fresh
//! from the ONE shared scenario file each time — no shape's tick can observe
//! another shape's writes, and the cross-RULE pre-state gap (D-row Q14)
//! never applies: nothing in this file ever loads more than one `(rule …)`
//! form in the same `rule_src`.
//!
//! **Why every rule declares `social-class/shape`.** `run_tick`'s subject
//! loop walks EVERY node of a rule's derived subject type; `social-class/
//! shape` is the discriminator that keeps each shape's rule from firing on
//! the eleven nodes it does not own (the `territory/shape` convention,
//! `query_lane_e2e.rs`).
//!
//! # Provenance
//!
//! Every expected value below is DERIVED in its test's own comment from the
//! scenario's seeded strengths. Every seeded strength is an exact dyadic
//! rational (a fraction whose denominator is a power of two — `0.1875 =
//! 3/16`, `0.375 = 3/8` are not powers of two themselves, but terminate
//! exactly in binary64, which is the property that matters), and every fold
//! sum below is a sum of exact dyadic rationals whose intermediate and final
//! values are all exactly representable — so IEEE-754 addition is EXACT at
//! every step, plain `f64` equality is the right assertion, and no
//! `to_bits()` pin is needed (`query_lane_e2e.rs` pins bits only where a
//! multiply introduces real rounding; nothing here multiplies).

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::run_once_into;

const SCENARIO: &str = include_str!("../content/scenarios/edge-lane-e2e.bscn");

// Node ids, fixed by the scenario's own declaration order (scenario.rs's
// "declaration order is the id order" contract) — see the scenario file's
// header comment for the full id map.
const FOLD_REPORTER: NodeId = NodeId(2);
const PAIR_X: NodeId = NodeId(3);
const HUB: NodeId = NodeId(6);

// ============================================================ Shape 1

/// `edges`-fold, graph-scope (no `the` needed): the subject sums the
/// implicit strength (D32) of EVERY dyadic SOLIDARITY edge in the graph —
/// proves `(edges <enum-ref>)` evaluates for real AND that Task 2's D32
/// wiring resolves `<edge-type>/strength` through an UNWEIGHTED aggregation
/// (`typecheck.rs::resolve_field`), not merely the bare accessor's graceful
/// fallback.
///
/// # Derivation
///
/// All seven seeded strengths, in the substrate's ascending (source, target)
/// order (each an exact dyadic rational; the sum is exact at every step):
/// `0.125 + 0.03125 + 0.015625 + 0.0625 + 0.125 + 0.1875 + 0.25
///  = 0.796875` — inside `[0,1]`, `E-EVAL-020`'s own domain for the
/// Coefficient-typed `social-class/fold-total`.
const RULE_EDGES_FOLD: &str = r#"
(rule social-class/edges-fold-e2e
  :role mechanic :evidence derived :material-basis "T2's edges query head materializing every dyadic SOLIDARITY edge in the graph and folding its implicit strength (D32) — proves (edges <enum-ref>) evaluates for real and Task 2's D32 wiring resolves <edge-type>/strength through an UNWEIGHTED aggregation (typecheck.rs::resolve_field), not merely the bare accessor's graceful fallback"
  :fuel 512
  (bindings (binding shape :field social-class/shape))
  (when (= shape 1))
  (effects
    (update-node self social-class/fold-total
      (set (fold sum (edges EdgeType/SOLIDARITY) (field-of it solidarity/strength))))))
"#;

#[test]
fn shape_1_edges_fold_sums_every_solidarity_edge_in_the_graph() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, RULE_EDGES_FOLD, &mut graph, &mut sink)
        .expect("the edges-fold rule must load and run through run_once_into");
    assert_eq!(report.fired, 1, "only fold-reporter (shape=1) fires");

    let fold_total = graph
        .node_attribute(FOLD_REPORTER, "social-class/fold-total")
        .unwrap();
    assert_eq!(
        fold_total, 0.796875,
        "0.125 + 0.03125 + 0.015625 + 0.0625 + 0.125 + 0.1875 + 0.25 = \
         0.796875 exactly (all dyadic rationals; every intermediate sum \
         exact in binary64)"
    );
}

// ============================================================ Shape 2

/// `edge-between` resolving successfully, its `field-of` read agreeing with
/// the strength seeded at the edge's own declaration — the R9 chapter-C2
/// required-vector family, turned into a real evaluation-level vector.
///
/// **Determinism caveat, stated explicitly (adversarial review nit):** the
/// `select-max` scores by the CONSTANT `1` — deterministic ONLY because
/// `pair-x`'s `:out` SOLIDARITY neighbor set is a SINGLETON (`pair-y`). A
/// constant score genuinely exercises D46's ascending-id tiebreak only when
/// two or more candidates tie, which this fixture does not attempt. If a
/// future edit ever gives `pair-x` a second outgoing SOLIDARITY edge, this
/// rule's constant score must become a real field (or the tiebreak must be
/// deliberately exercised and asserted).
///
/// # Derivation
///
/// `edge-between(SOLIDARITY, pair-x, pair-y)` resolves the seeded
/// `0.03125c` edge; `field-of … solidarity/strength` reads back `0.03125`
/// exactly (dyadic; bit-exact through Task 6a's hydration conversion).
const RULE_EDGE_BETWEEN_RESOLVES: &str = r#"
(rule social-class/edge-between-resolves-e2e
  :role mechanic :evidence derived :material-basis "edge-between resolving successfully and its field-of read agreeing with the strength seeded at the edge's own declaration — the R9 chapter-C2 required-vector family, turned into a real evaluation-level vector"
  :fuel 256
  (bindings (binding shape :field social-class/shape))
  (when (= shape 2))
  (effects
    (update-node self social-class/fold-total
      (set (field-of (edge-between EdgeType/SOLIDARITY self
                        (select-max (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS) 1))
                      solidarity/strength)))))
"#;

/// The failing direction: `edge-between` is directional (§2.10's key is the
/// ORDERED triple) — the fixture seeds `pair-z -> pair-x`, and this rule
/// looks up the REVERSE (`pair-x -> pair-z`), which must fail loudly
/// through the real driver (`E-EVAL-034`, a TICK ABORT), never a silent
/// absent reference. Same singleton-neighbor determinism caveat as the
/// resolving rule above (`pair-z`'s `:out` set is `{pair-x}` alone).
const RULE_EDGE_BETWEEN_MISSING: &str = r#"
(rule social-class/edge-between-missing-is-e-eval-034-e2e
  :role mechanic :evidence derived :material-basis "edge-between is directional (§2.10's key is the ORDERED triple) — looking up the reverse of a seeded edge finds nothing and must fail loudly through the real driver, never a silent absent reference"
  :fuel 256
  (bindings (binding shape :field social-class/shape))
  (when (= shape 4))
  (effects
    (update-node self social-class/fold-total
      (set (field-of (edge-between EdgeType/SOLIDARITY
                        (select-max (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS) 1)
                        self)
                      solidarity/strength)))))
"#;

#[test]
fn shape_2a_edge_between_resolves_and_reads_strength() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, RULE_EDGE_BETWEEN_RESOLVES, &mut graph, &mut sink)
        .expect("the edge-between-resolves rule must load and run through run_once_into");
    assert_eq!(report.fired, 1, "only pair-x (shape=2) fires");

    let fold_total = graph
        .node_attribute(PAIR_X, "social-class/fold-total")
        .unwrap();
    assert_eq!(
        fold_total, 0.03125,
        "the strength seeded at (edge … pair-x pair-y 0.03125c), read back \
         through edge-between + field-of, exactly"
    );
}

#[test]
fn shape_2b_edge_between_on_a_missing_pair_is_e_eval_034() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let err = run_once_into(SCENARIO, RULE_EDGE_BETWEEN_MISSING, &mut graph, &mut sink).expect_err(
        "looking up the REVERSE of a seeded directional edge must abort \
             the tick (E-EVAL-034), never yield an absent reference",
    );
    assert!(err.contains("E-EVAL-034"), "{err}");
}

// ============================================================ Shape 3

/// The self-anchored `neighbors`+`edge-between` idiom — §3.8 item 8's worked
/// example, Solidarity's own anticipated read shape: per TARGET node, walk
/// incoming SOLIDARITY neighbours and resolve each edge's strength by key,
/// needing no `(edges …)` iteration and no `source-of`/`target-of` endpoint
/// accessor (the language does not have one — §3.8 item 8's own open item,
/// dossier §8). This is the vector that unblocks Solidarity's own port
/// train.
///
/// # Derivation
///
/// `hub`'s `:in` SOLIDARITY neighbours are `spoke-1`/`spoke-2`/`spoke-3`;
/// their seeded strengths sum `0.0625 + 0.125 + 0.1875 = 0.375` exactly
/// (dyadic at every step; inside `[0,1]`).
const RULE_SELF_ANCHORED: &str = r#"
(rule social-class/self-anchored-solidarity-fold-e2e
  :role mechanic :evidence derived :material-basis "the §3.8 item 8 worked example — Solidarity's own anticipated read shape, self-anchored via neighbors+edge-between rather than iterating (edges ...) and needing an endpoint accessor the language does not have (§3.8 item 8's own open item, dossier §8). T2 proves this idiom evaluates for real, and this is the vector that unblocks Solidarity's own port train without needing source-of/target-of."
  :fuel 512
  (bindings (binding shape :field social-class/shape))
  (when (= shape 3))
  (effects
    (update-node self social-class/fold-total
      (set (fold sum (neighbors self EdgeType/SOLIDARITY :in NodeType/SOCIAL_CLASS)
                 (field-of (edge-between EdgeType/SOLIDARITY it self) solidarity/strength))))))
"#;

#[test]
fn shape_3_self_anchored_neighbors_and_edge_between_sums_incoming_strength() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, RULE_SELF_ANCHORED, &mut graph, &mut sink)
        .expect("the self-anchored rule must load and run through run_once_into");
    assert_eq!(report.fired, 1, "only hub (shape=3) fires");

    let fold_total = graph
        .node_attribute(HUB, "social-class/fold-total")
        .unwrap();
    assert_eq!(
        fold_total, 0.375,
        "0.0625 + 0.125 + 0.1875 = 0.375 exactly — hub's three incoming \
         spoke strengths, resolved per-edge via edge-between"
    );
}

// ============================================================ Determinism

/// §6.2 family 8, split (adversarial review Blocker 3): the three SUCCEEDING
/// shapes run through the unmodified `query_lane_e2e.rs`-style
/// `TickReport`-comparison loop — Shape 2b is DESIGNED to `Err` and gets its
/// own two-run leg below, since this loop's `.unwrap_or_else(panic!)` would
/// panic on it.
#[test]
fn every_succeeding_shape_is_deterministic_across_two_independent_runs() {
    for (rule, label) in [
        (RULE_EDGES_FOLD, "edges-fold"),
        (RULE_EDGE_BETWEEN_RESOLVES, "edge-between-resolves"),
        (RULE_SELF_ANCHORED, "self-anchored-fold"),
    ] {
        let mut graph_a = HypergraphStore::new();
        let mut sink_a = CollectingSink::default();
        let report_a = run_once_into(SCENARIO, rule, &mut graph_a, &mut sink_a)
            .unwrap_or_else(|e| panic!("{label}: first run: {e}"));

        let mut graph_b = HypergraphStore::new();
        let mut sink_b = CollectingSink::default();
        let report_b = run_once_into(SCENARIO, rule, &mut graph_b, &mut sink_b)
            .unwrap_or_else(|e| panic!("{label}: second run: {e}"));

        assert_eq!(report_a.before, report_b.before, "{label}: before hash");
        assert_eq!(report_a.after, report_b.after, "{label}: after hash");
        assert_eq!(report_a.fired, report_b.fired, "{label}: fired count");
        assert_eq!(
            report_a.per_rule_fired, report_b.per_rule_fired,
            "{label}: per_rule_fired"
        );
    }
}

/// Shape 2b's own two-run leg: the same content must fail the same way both
/// times — the error STRING is the comparison, since a tick that aborts
/// yields no `TickReport` to compare.
#[test]
fn shape_2b_error_is_deterministic_across_two_independent_runs() {
    let mut graph_a = HypergraphStore::new();
    let mut sink_a = CollectingSink::default();
    let err_a = run_once_into(
        SCENARIO,
        RULE_EDGE_BETWEEN_MISSING,
        &mut graph_a,
        &mut sink_a,
    )
    .expect_err("shape 2b must abort the tick");

    let mut graph_b = HypergraphStore::new();
    let mut sink_b = CollectingSink::default();
    let err_b = run_once_into(
        SCENARIO,
        RULE_EDGE_BETWEEN_MISSING,
        &mut graph_b,
        &mut sink_b,
    )
    .expect_err("shape 2b must abort the tick");

    assert_eq!(
        err_a, err_b,
        "the same content must fail the same way both times"
    );
    assert!(err_a.contains("E-EVAL-034"), "{err_a}");
}
