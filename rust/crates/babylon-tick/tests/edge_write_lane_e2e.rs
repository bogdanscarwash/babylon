//! The T3 edge-WRITE-lane end-to-end vectors (Program 29 train T3 PR B,
//! issue #560; ADR198 R1-R3) — `update-edge` executed through the real
//! production entry point (`babylon_tick::run_once_into` → `run_tick`'s
//! collect-then-apply, the same seam the CLI driver and `babylon-client`'s
//! engine link both call), against T3 PR A's storage.
//!
//! **This ships no Solidarity content** — the same posture `edge_lane_e2e.rs`
//! took for the read half: the fixture (`content/scenarios/edge-write-lane-
//! e2e.bscn`) is a hand-built minimal world and the rules are the §2.10/§6.2
//! worked shapes, not a port. Each shape runs as its OWN single-rule content
//! set, loaded fresh from the one shared scenario file each time
//! (`edge_lane_e2e.rs`'s discipline — no shape's tick can observe another
//! shape's writes).
//!
//! What these two vectors prove that the unit suite cannot: before T3, an
//! `update-edge` rule LOADED CLEAN and died at its first admitted tick in
//! the collect path's refusal arm (the dossier's surprise 6). These run the
//! real load pipeline AND the real tick — the load-time gate
//! (`check_no_deferred_shape_verbs`) never covered `update-edge`, so the
//! full path from content to a moved tick hash is what's actually on trial.
//!
//! # Provenance
//!
//! Every expected value is DERIVED in its test's own comment and exact in
//! binary64 (dyadic rationals only — `edge_lane_e2e.rs`'s discipline): plain
//! `f64` equality is the right assertion throughout.

use babylon_bsl::evaluator::Value;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::run_once_into;

const SCENARIO: &str = include_str!("../content/scenarios/edge-write-lane-e2e.bscn");

// Node ids, fixed by the scenario's own declaration order (see the scenario
// file's header comment for the id map).
const BASE_A: NodeId = NodeId(1);
const BASE_B: NodeId = NodeId(2);
const BASE_C: NodeId = NodeId(3);

// ============================================================ Shape 1

/// §6.2 chapter C6's vector: `for-each` over `edges` applying `update-edge`
/// per element — BOTH write kinds (a `<edge-type>/strength` scale, routed to
/// the 0x03 slot per D143, and a deffield-declared field set, minting
/// fifth-section rows) — plus an `emit` reading `solidarity/strength` inside
/// the same body. The emit is the pre-state proof: it fires during
/// COLLECTION, so it must read the PRE-tick strengths (0.5, 0.25) even
/// though the body's own scale writes are collected around it — §2.8's own
/// worked-example discipline, on the edge lane.
///
/// # Derivation
///
/// §2.6's edge iteration order is ascending (source, target): `base-a →
/// base-b` (1, 2) before `base-b → base-c` (2, 3). Post-tick strengths:
/// `0.5 × 0.5 = 0.25` and `0.25 × 0.5 = 0.125`, exact; both edges carry
/// `tension = 0.75`. The two PROBE payloads read the PRE-tick strengths
/// `0.5` and `0.25`, in that order.
const RULE_FOR_EACH_WRITES: &str = r#"
(rule social-class/edge-write-for-each-e2e
  :role mechanic :evidence derived :material-basis "§6.2 chapter C6's required vector — for-each over edges applying update-edge per element, both write kinds (D143 strength-fork scale + deffield set), with an emit reading the PRE-tick strength inside the same body (the pre-state law on the edge lane)"
  :fuel 512
  (bindings (binding shape :field social-class/shape))
  (when (= shape 1))
  (effects
    (for-each (edges EdgeType/SOLIDARITY)
      (update-edge it solidarity/strength (scale 0.5c))
      (update-edge it solidarity/tension (set 0.75i))
      (emit EventType/PROBE (s (field-of it solidarity/strength))))))
"#;

#[test]
fn shape_1_for_each_over_edges_writes_every_edge_and_the_emit_reads_pre_state() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, RULE_FOR_EACH_WRITES, &mut graph, &mut sink)
        .expect("the for-each write rule must load and run through run_once_into");
    assert_eq!(report.fired, 1, "only writer (shape=1) fires");

    // The writes landed, per element, through the production collect path.
    let strength_ab = graph
        .edge_attribute("SOLIDARITY", BASE_A, BASE_B, "solidarity/strength")
        .unwrap();
    let strength_bc = graph
        .edge_attribute("SOLIDARITY", BASE_B, BASE_C, "solidarity/strength")
        .unwrap();
    assert_eq!(strength_ab, 0.25, "0.5 scaled by 0.5");
    assert_eq!(strength_bc, 0.125, "0.25 scaled by 0.5");
    for (from, to) in [(BASE_A, BASE_B), (BASE_B, BASE_C)] {
        let tension = graph
            .edge_attribute("SOLIDARITY", from, to, "solidarity/tension")
            .unwrap();
        assert_eq!(tension, 0.75, "the deffield field set per element");
    }

    // The emit read PRE-tick strengths, in §2.6 ascending (source, target)
    // order — the pre-state law, pinned on the edge lane.
    let events: Vec<&(String, Vec<(String, Value)>)> = sink.events.iter().collect();
    assert_eq!(events.len(), 2, "one PROBE per edge element");
    let payload_s = |event: &(String, Vec<(String, Value)>)| match &event.1[..] {
        [(name, Value::Real(r))] if name == "s" => *r,
        other => panic!("unexpected PROBE payload: {other:?}"),
    };
    assert_eq!(payload_s(events[0]), 0.5, "base-a → base-b, PRE-scale");
    assert_eq!(payload_s(events[1]), 0.25, "base-b → base-c, PRE-scale");

    // The tick hash MOVED — the first production-path fifth-section writes
    // are hash-visible (III.7's dual), and the pre-tick hash is the T2-era
    // fixture's own (the elision keeps a writeless load byte-identical — the
    // R2 proof at the scenario level: before T3 this scenario hydrated the
    // same bytes, and the fifth section stayed elided until the first write).
    assert_ne!(
        report.before, report.after,
        "edge-attribute writes through the production path must move the tick hash"
    );
    assert!(
        !graph.all_edge_attributes().is_empty(),
        "the fifth section is non-empty after the tick"
    );

    // The same-file determinism leg (query_lane_e2e.rs's discipline): a
    // second, independent run of the same content produces byte-identical
    // hashes.
    let mut graph2 = HypergraphStore::new();
    let mut sink2 = CollectingSink::default();
    let report2 = run_once_into(SCENARIO, RULE_FOR_EACH_WRITES, &mut graph2, &mut sink2)
        .expect("the determinism leg's run must succeed");
    assert_eq!(report.before, report2.before);
    assert_eq!(report.after, report2.after);
}

// ============================================================ Shape 2

/// §2.10's worked write shape (D36): a subject holding only endpoints
/// reaches the edge through `edge-between` and writes it — the `add` op on
/// `<edge-type>/strength` and a `set` on the deffield field, through the
/// production path. Also pinned: the untouched second edge's tension was
/// NEVER written and reads loud (the honest-null discipline end-to-end).
///
/// **Determinism caveat (edge_lane_e2e.rs's own, restated):** the
/// `select-max` scores by the constant `1` — deterministic ONLY because
/// `base-a`'s `:out` SOLIDARITY neighbor set is a singleton (`base-b`).
///
/// # Derivation
///
/// `edge-between(SOLIDARITY, base-a, base-b)` resolves the seeded `0.5c`
/// edge; `(add 0.25c)` takes its strength to `0.75` exactly; tension sets
/// to `0.5`. The `base-b → base-c` edge is untouched: strength `0.25`,
/// tension never written (a loud error, never 0.0).
const RULE_TARGETED_WRITE: &str = r#"
(rule social-class/edge-write-targeted-e2e
  :role mechanic :evidence derived :material-basis "§2.10's worked write shape (D36) through the production path — edge-between resolving the referent, add on <edge-type>/strength (the 0x03 slot, D143), set on the deffield field (the fifth section), and the untouched edge's never-written field reading loud"
  :fuel 512
  (bindings (binding shape :field social-class/shape))
  (when (= shape 2))
  (effects
    (update-edge
      (edge-between EdgeType/SOLIDARITY self
        (select-max (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS) 1))
      solidarity/strength (add 0.25c))
    (update-edge
      (edge-between EdgeType/SOLIDARITY self
        (select-max (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS) 1))
      solidarity/tension (set 0.5i))))
"#;

#[test]
fn shape_2_edge_between_targets_one_edges_write_and_the_other_stays_honest_null() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, RULE_TARGETED_WRITE, &mut graph, &mut sink)
        .expect("the targeted write rule must load and run through run_once_into");
    assert_eq!(report.fired, 1, "only base-a (shape=2) fires");

    let strength_ab = graph
        .edge_attribute("SOLIDARITY", BASE_A, BASE_B, "solidarity/strength")
        .unwrap();
    assert_eq!(strength_ab, 0.75, "0.5 + 0.25, exact");
    let tension_ab = graph
        .edge_attribute("SOLIDARITY", BASE_A, BASE_B, "solidarity/tension")
        .unwrap();
    assert_eq!(tension_ab, 0.5, "the set landed");

    let strength_bc = graph
        .edge_attribute("SOLIDARITY", BASE_B, BASE_C, "solidarity/strength")
        .unwrap();
    assert_eq!(strength_bc, 0.25, "the untouched edge's strength stands");
    assert!(
        graph
            .edge_attribute("SOLIDARITY", BASE_B, BASE_C, "solidarity/tension")
            .is_err(),
        "never written is loud, end-to-end — never a default 0.0 (III.11)"
    );
    assert_ne!(report.before, report.after);
}
