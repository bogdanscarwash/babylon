//! Hyperedge-lane Task 6 (E2b, Community port train, plan
//! `docs/superpowers/plans/2026-08-18-community-port.md`): the BSL
//! read/write surface for hyperedge own-fields, proven through the content
//! driver — `update-hyperedge` on the tick's collect-then-apply path,
//! `field-of` over a `HyperedgeRef`, `(hyperedge-attr …)` seeding, the
//! D29 owner-kind filter's subject-type refusal, and the typecheck
//! coverage in both directions.
//!
//! This file began as the RED half: two refusal texts pinned through the
//! content driver (the III.7 storage refusal and the "not meaningful"
//! refusal). Task 5's `GraphSubstrate::update_hyperedge_attribute`
//! discharged the storage gap, so the pins INVERTED — what is pinned now
//! is the served surface, with the refusals that remain (the §2.10
//! referent check, the D29 subject-type law, the scalar-type guards)
//! asserted on their own terms.
//!
//! NOTE on file placement (deviation, recorded): the plan's Task 6 names
//! `tests/hyperedge_lane_e2e.rs` (babylon-bsl), whose harness only loads
//! scenarios — the same deviation Task 3's pin file records applies here
//! (these proofs drive the tick).

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, HyperedgeId, NodeId};
use babylon_tick::run_once_into;

/// Two classes, one COMMUNITY hyperedge over both, `community/heat`
/// declared. Every proof below seeds from some extension of this shape.
const SCENARIO: &str = r"
(scenario ft/hyperedge-surface
  (defvocabulary HyperedgeType (COMMUNITY))
  (deffield social-class/active int extensive)
  (deffield social-class/observed coefficient intensive)
  (deffield community/heat coefficient intensive)
  (node alpha NodeType/SOCIAL_CLASS (social-class/active 1))
  (node beta NodeType/SOCIAL_CLASS (social-class/active 1))
  (hyperedge cell HyperedgeType/COMMUNITY (members alpha beta)))
";

/// Step 2, the tick half: `update-hyperedge` inside a `for-each` over
/// `(hyperedges …)` WRITES through the collect-then-apply path. Two active
/// subjects each run the body once against the same hyperedge; `set` is
/// idempotent under that, so the stored value is 0.5 exactly (dyadic).
#[test]
fn update_hyperedge_writes_through_the_tick() {
    let rule = r#"
(rule community/probe-update-hyperedge
  :role mechanic :evidence derived :material-basis "Task 6 Step 2 proof: update-hyperedge writes through the tick"
  :fuel 2048
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (for-each (hyperedges HyperedgeType/COMMUNITY)
      (update-hyperedge it community/heat (set 0.5c)))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, rule, &mut graph, &mut sink)
        .expect("update-hyperedge is served on the tick path");
    assert_eq!(report.fired, 2, "both active subjects fire the for-each");
    let stored = graph
        .hyperedge_attribute(HyperedgeId(0), "community/heat")
        .expect("the write landed");
    assert_eq!(stored.to_bits(), (0.5_f64).to_bits());
}

/// Steps 3+5 in one chain: `(hyperedge-attr …)` seeds the field at
/// hydration, `field-of` over the `HyperedgeRef` reads it mid-tick, and
/// the value lands on a node field through `update-node` — the full
/// seed → read → write circuit.
#[test]
fn field_of_reads_a_seeded_hyperedge_field_into_a_node_write() {
    let scenario = SCENARIO.replace(
        "(members alpha beta)))",
        "(members alpha beta))\n  (hyperedge-attr cell community/heat 0.25c))",
    );
    let rule = r#"
(rule community/probe-field-of-hyperedge
  :role mechanic :evidence derived :material-basis "Task 6 Steps 3+5 proof: field-of reads a hyperedge-attr-seeded field"
  :fuel 2048
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (for-each (hyperedges HyperedgeType/COMMUNITY)
      (update-node self social-class/observed (set (field-of it community/heat))))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(&scenario, rule, &mut graph, &mut sink)
        .expect("field-of over a HyperedgeRef is served");
    for id in [NodeId(0), NodeId(1)] {
        let observed = graph
            .node_attribute(id, "social-class/observed")
            .expect("the node write landed");
        assert_eq!(
            observed.to_bits(),
            (0.25_f64).to_bits(),
            "node {id:?} read the seeded 0.25 through field-of"
        );
    }
}

/// §2.10 discipline 1, live: the qname's owning type must match the
/// referent's declared type, on the write side exactly as on the read
/// side. Fires BEFORE the field-registry lookup, so the foreign field
/// needs no declaration.
#[test]
fn update_hyperedge_refuses_a_qname_owned_by_another_type() {
    let rule = r#"
(rule community/probe-referent-check
  :role mechanic :evidence derived :material-basis "Task 6 proof: the §2.10 discipline-1 referent check"
  :fuel 2048
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (for-each (hyperedges HyperedgeType/COMMUNITY)
      (update-hyperedge it economic-sector/output (set 0.5c)))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let err = run_once_into(SCENARIO, rule, &mut graph, &mut sink).unwrap_err();
    assert!(
        err.contains("the referent is a COMMUNITY hyperedge, not ECONOMIC_SECTOR"),
        "the referent check names both types: {err}"
    );
}

/// Step 4's mechanism behind §8c guard 2: a hyperedge-namespace `:field`
/// binding is invisible to subject derivation (D29 — a `:field` binding is
/// node-scoped and stays node-scoped), so a rule whose ONLY field binding
/// owns off a HyperedgeType refuses with the subject-type error rather
/// than silently iterating an empty population.
#[test]
fn a_hyperedge_owned_field_binding_names_no_subject_type() {
    let rule = r#"
(rule community/probe-d29-binding
  :role mechanic :evidence derived :material-basis "Task 6 Step 4 proof: a hyperedge-owned :field binding names no subject type"
  :fuel 2048
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding heat :field community/heat))
  (when (>= heat 0.0c))
  (effects
    (emit EventType/ORGANIZATION_SEEDED (probe 1))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let err = run_once_into(SCENARIO, rule, &mut graph, &mut sink).unwrap_err();
    assert!(
        err.contains("names no subject type"),
        "the subject-type error, not silent inertness: {err}"
    );
}

/// Step 6, write direction: a non-numeric write value refuses at the one
/// funnel every write crosses — `numeric_write_value` — never coerced.
#[test]
fn update_hyperedge_refuses_a_wrong_typed_write_value() {
    let rule = r#"
(rule community/probe-write-type
  :role mechanic :evidence derived :material-basis "Task 6 Step 6 proof: a wrong-typed write value refuses"
  :fuel 2048
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (for-each (hyperedges HyperedgeType/COMMUNITY)
      (update-hyperedge it community/heat (set #t)))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let err = run_once_into(SCENARIO, rule, &mut graph, &mut sink).unwrap_err();
    assert!(
        err.contains("cannot store") && err.contains("community/heat"),
        "the write-value type refusal names the field: {err}"
    );
}

/// Step 6, read direction: an enum-typed hyperedge field renders
/// `Value::Enum` through the same `bind_field_value` a `:field` binding
/// uses (D102's discharge, one element kind over), and arithmetic on it
/// refuses — Enum<T> supports no arithmetic (§2.13).
#[test]
fn field_of_an_enum_typed_hyperedge_field_refuses_arithmetic() {
    let scenario = r"
(scenario ft/hyperedge-enum-arith
  (defvocabulary HyperedgeType (COMMUNITY))
  (defenum CommunityType (REVOLUTIONARY LIBERAL))
  (deffield social-class/active int extensive)
  (deffield social-class/observed coefficient intensive)
  (deffield community/kind enum CommunityType)
  (node alpha NodeType/SOCIAL_CLASS (social-class/active 1))
  (node beta NodeType/SOCIAL_CLASS (social-class/active 1))
  (hyperedge cell HyperedgeType/COMMUNITY (members alpha beta))
  (hyperedge-attr cell community/kind CommunityType/REVOLUTIONARY))
";
    let rule = r#"
(rule community/probe-enum-arith
  :role mechanic :evidence derived :material-basis "Task 6 Step 6 proof: enum field read in arithmetic position refuses"
  :fuel 2048
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (for-each (hyperedges HyperedgeType/COMMUNITY)
      (update-node self social-class/observed (set (+ (field-of it community/kind) 1))))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let err = run_once_into(scenario, rule, &mut graph, &mut sink).unwrap_err();
    assert!(
        err.contains("no arithmetic is defined on"),
        "Enum<T> in arithmetic position refuses: {err}"
    );
}
