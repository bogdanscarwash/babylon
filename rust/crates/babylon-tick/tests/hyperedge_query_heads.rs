//! Hyperedge-lane Task 3 (E1c, Community port train, plan
//! `docs/superpowers/plans/2026-08-18-community-port.md`): the three §2.6
//! hyperedge query heads — `hyperedges`, `members-of`, `hyperedges-of` —
//! served by the query evaluator, plus the `Element::Hyperedge` cross-kind
//! Ord ruling.
//!
//! RED→GREEN record: each head was first pinned refusing through BOTH
//! tables — `query::materialize`'s unserved-head refusal (query position,
//! "…is a §2.6 query head the query evaluator does not yet serve — it
//! lands with slice 3, never as a default here") and `evaluator`'s
//! expression-position classifier (bare `<expr>` position, "…is a
//! query/selection/accessor form the evaluator does not yet serve
//! (§2.6/§2.7/§2.10) — it lands with slice 3…"). Those pins ran green
//! pre-landing and were then replaced by the served proofs below; the texts
//! survive in this comment as the historical record (D201).
//!
//! NOTE on file placement (deviation, recorded): the plan's Task 3 names
//! `tests/hyperedge_lane_e2e.rs` (babylon-bsl); that file's harness only
//! loads scenarios, while these pins must drive the tick — they live here
//! beside Task 4's own tick-driver file, the seam the pins actually
//! exercise.

use babylon_bsl::evaluator::Value;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_tick::run_once_into;

/// The Task-4 probe world: two COMMUNITY hyperedges over three classes.
const SCENARIO: &str = r"
(scenario ft/hyperedge-query-heads
  (defvocabulary HyperedgeType (COMMUNITY))
  (deffield social-class/active int extensive)
  (node alpha NodeType/SOCIAL_CLASS (social-class/active 1))
  (node beta NodeType/SOCIAL_CLASS (social-class/active 1))
  (node gamma NodeType/SOCIAL_CLASS (social-class/active 1))
  (hyperedge new-afrikan HyperedgeType/COMMUNITY (members alpha beta gamma))
  (hyperedge queer HyperedgeType/COMMUNITY (members alpha)))
";

/// Build the probe rule: `<head-form>` in a fold body (query position) or
/// a bare `:expr` binding (expression position).
fn probe_rule(head_form: &str, position: &str) -> String {
    match position {
        "query" => format!(
            r#"
(rule community/probe-query
  :role mechanic :evidence derived :material-basis "probe for the query-position refusal (Task 3 Step 1)"
  :fuel 512
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (emit EventType/ORGANIZATION_SEEDED (probe (fold sum {head_form} (field-of it social-class/active))))))
"#
        ),
        "expression" => format!(
            r#"
(rule community/probe-expression
  :role mechanic :evidence derived :material-basis "probe for the expression-position refusal (Task 3 Step 1)"
  :fuel 512
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active)
    (binding x :expr {head_form}))
  (when (= active 1))
  (effects
    (emit EventType/ORGANIZATION_SEEDED (probe 1))))
"#
        ),
        other => panic!("unknown position {other}"),
    }
}

// ---------------------------------------------------------------------------
// Slice-3 GREEN proofs (Task 3 Steps 3-4): the three heads served, exercised
// through the real driver. The Step-1 refusal pins above are the historical
// record of the texts these proofs replaced.
// ---------------------------------------------------------------------------

/// The two-hop census: every active member of every COMMUNITY hyperedge,
/// counted — `(fold sum (hyperedges …) :as c (fold sum (members-of c …)
/// (field-of it social-class/active)))` is §2.6's own two-hop worked
/// example's shape. The world's census is 3 (new-afrikan) + 1 (queer) = 4.
/// The type-wide head serves the census. (domain :graph) fires once per
/// tick in principle, but the driver's subject-type derivation reads the
/// rule's :field bindings first (E-LOAD-004's domain surface), so this
/// probe stays NodeType/SOCIAL_CLASS-scoped and fires once per class —
/// the same census count each time, the point being the heads serve.
#[test]
fn the_type_wide_head_serves_the_census() {
    let rule = r#"
(rule community/probe-census-all
  :role mechanic :evidence derived :material-basis "served-proof for the type-wide head (Task 3)"
  :fuel 2048
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (emit EventType/ORGANIZATION_SEEDED
      (probe (fold sum (hyperedges HyperedgeType/COMMUNITY) :as c (fold sum (members-of c HyperedgeType/COMMUNITY) (field-of it social-class/active)))))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, rule, &mut graph, &mut sink).expect("the census must run");
    let sums: Vec<f64> = sink
        .events
        .iter()
        .filter_map(|(_, payload)| {
            payload.iter().find_map(|(k, v)| {
                (k == "probe").then_some(v).and_then(|v| match v {
                    Value::Real(x) => Some(*x),
                    _ => None,
                })
            })
        })
        .collect();
    assert_eq!(
        sums,
        vec![4.0, 4.0, 4.0],
        "one firing per class, each computing the world's census 3 + 1"
    );
}

/// The subject-relative head: each class's own memberships counted.
/// alpha (id 0) belongs to BOTH hyperedges (members 3 + 1 = 4); beta and
/// gamma to new-afrikan only (3 each).
#[test]
fn the_subject_relative_head_serves_per_class_membership_sums() {
    let rule = r#"
(rule community/probe-census-per-class
  :role mechanic :evidence derived :material-basis "served-proof for the subject-relative head (Task 3)"
  :fuel 2048
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (emit EventType/ORGANIZATION_SEEDED
      (probe (fold sum (hyperedges-of self HyperedgeType/COMMUNITY) :as h (fold sum (members-of h HyperedgeType/COMMUNITY) (field-of it social-class/active)))))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, rule, &mut graph, &mut sink).expect("the per-class census must run");
    let sums: Vec<f64> = sink
        .events
        .iter()
        .filter_map(|(_, payload)| {
            payload.iter().find_map(|(k, v)| {
                (k == "probe").then_some(v).and_then(|v| match v {
                    Value::Real(x) => Some(*x),
                    _ => None,
                })
            })
        })
        .collect();
    assert_eq!(
        sums,
        vec![4.0, 3.0, 3.0],
        "alpha belongs to both; beta and gamma to new-afrikan only"
    );
}

/// The Step-1 pins, inverted at the landing: a SERVED head in bare
/// `<expr>` position still refuses — the query-operand-only law (§2.7: no
/// bare `<query>` production) is grammatical, not a TODO. The
/// query-position refusal texts these pins replaced are recorded in this
/// file's header as the historical record.
#[test]
fn a_served_head_in_bare_expression_position_still_refuses_with_the_2_7_law() {
    for head_form in [
        "(hyperedges HyperedgeType/COMMUNITY)",
        "(members-of self HyperedgeType/COMMUNITY)",
        "(hyperedges-of self HyperedgeType/COMMUNITY)",
    ] {
        let mut graph = HypergraphStore::new();
        let mut sink = CollectingSink::default();
        let err = run_once_into(
            SCENARIO,
            &probe_rule(head_form, "expression"),
            &mut graph,
            &mut sink,
        )
        .unwrap_err();
        assert!(
            err.contains("no <expr> production of its own (§2.7)"),
            "{head_form}: {err}"
        );
    }
}

/// Step 5, the fuel axis VERIFIED (not rebuilt): a hyperedge-folding rule
/// with a deliberate `:fuel 1` refuses E-LOAD-040 with the bound the
/// already-landed ceiling machinery computes — read from the refusal,
/// never hand-computed.
#[test]
fn the_fuel_axis_bounds_a_hyperedge_fold_as_landed() {
    let rule = r#"
(rule community/probe-fuel
  :role mechanic :evidence derived :material-basis "fuel-axis readback (Task 3 Step 5)"
  :fuel 1
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (emit EventType/ORGANIZATION_SEEDED
      (probe (fold count (hyperedges HyperedgeType/COMMUNITY) :as c (fold count (members-of c HyperedgeType/COMMUNITY) (field-of it social-class/active)))))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let err = run_once_into(SCENARIO, rule, &mut graph, &mut sink).unwrap_err();
    assert!(err.contains("E-LOAD-040"), "{err}");
    let bound = err
        .split("static bound ")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|n| n.parse::<u64>().ok())
        .unwrap_or_else(|| panic!("no measured bound in the refusal: {err}"));
    // Measured 2026-08-22: 28, read from the refusal, never hand-computed
    // (the plan's own law). The pin guards that the number moves iff the
    // machinery changes.
    assert_eq!(bound, 28, "the E-LOAD-040 refusal's own number: {err}");
}

/// Step 5, the negative half: the same fold in a world seeding NO
/// COMMUNITY hyperedge (an empty census) refuses E-LOAD-045 — the TYPE
/// ceiling is checked before the max-members axis, and with nothing seeded
/// both maps lack the entry, so 045 fires first. (The 042-only state — a
/// type ceiling with no max-members entry — is not constructible from
/// content: the two maps fill together at the Task-4 seam. That path stays
/// pinned by bound_checker.rs's own unit test plus Task 4's mutation
/// vector, recorded here so the absence is explained, not silent.)
#[test]
fn a_members_of_fold_in_an_empty_census_world_is_e_load_045() {
    let empty_world = r"
(scenario ft/no-communities
  (defvocabulary HyperedgeType (COMMUNITY))
  (deffield social-class/active int extensive)
  (node alpha NodeType/SOCIAL_CLASS (social-class/active 1)))
";
    let rule = r#"
(rule community/probe-fuel-empty
  :role mechanic :evidence derived :material-basis "fuel-axis negative readback (Task 3 Step 5)"
  :fuel 512
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (emit EventType/ORGANIZATION_SEEDED
      (probe (fold count (hyperedges HyperedgeType/COMMUNITY) :as c (fold count (members-of c HyperedgeType/COMMUNITY) (field-of it social-class/active)))))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let err = run_once_into(empty_world, rule, &mut graph, &mut sink).unwrap_err();
    assert!(err.contains("E-LOAD-045"), "{err}");
    assert!(err.contains("COMMUNITY"), "{err}");
}

/// Step 6: a hyperedge element in a numeric position refuses at LOAD,
/// through the same Phase-1 kind-propagation gate that covers nodes and
/// edges today — no new machinery, pinned so the shared gate's coverage of
/// this lane is on record.
#[test]
fn a_hyperedge_element_in_a_numeric_position_refuses_at_load() {
    let rule = r#"
(rule community/probe-numeric
  :role mechanic :evidence derived :material-basis "Step-6 pin: hyperedge element in a numeric position"
  :fuel 2048
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (emit EventType/ORGANIZATION_SEEDED
      (probe (fold sum (hyperedges HyperedgeType/COMMUNITY) (+ it 1))))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let err = run_once_into(SCENARIO, rule, &mut graph, &mut sink).unwrap_err();
    assert!(
        err.contains("kind-propagation over compound expressions is not implemented in Phase 1"),
        "{err}"
    );
}

/// Step 6, the enum-ref operand position: a NodeType member where the head
/// demands a HyperedgeType member refuses E-TYPE-011 (grammar.rs's own
/// WrongEnumKind — the HyperedgeType positions pre-registered in
/// ENUM_REF_POSITIONS are live).
#[test]
fn a_node_type_member_in_a_hyperedge_position_is_e_type_011() {
    let rule = r#"
(rule community/probe-wrong-kind
  :role mechanic :evidence derived :material-basis "Step-6 pin: enum-ref kind check on a hyperedge position"
  :fuel 2048
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (emit EventType/ORGANIZATION_SEEDED
      (probe (fold sum (hyperedges NodeType/SOCIAL_CLASS) :as c (fold count (members-of c NodeType/SOCIAL_CLASS) (field-of it social-class/active)))))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let err = run_once_into(SCENARIO, rule, &mut graph, &mut sink).unwrap_err();
    assert!(err.contains("E-TYPE-011"), "{err}");
}

/// PR #684 review (a REAL soundness gap): `members-of`'s annotated type
/// must match the referent's actual type — the load-time bound checker
/// bounds the fold by the ANNOTATED type's census-fed max-members, and a
/// referent of a different type would iterate past that bound unchecked.
/// Now refused with E-EVAL-032 (D24), driven content-level through the
/// two-hop form: the outer fold binds a COMMUNITY referent, the inner
/// annotates SECTOR.
#[test]
fn a_members_of_fold_with_a_mismatched_annotation_is_e_eval_032() {
    let world = r"
(scenario ft/type-mismatch
  (defvocabulary HyperedgeType (COMMUNITY SECTOR))
  (deffield social-class/active int extensive)
  (node alpha NodeType/SOCIAL_CLASS (social-class/active 1))
  (hyperedge cell HyperedgeType/COMMUNITY (members alpha))
  (hyperedge sector HyperedgeType/SECTOR (members alpha)))
";
    let rule = r#"
(rule community/probe-type-mismatch
  :role mechanic :evidence derived :material-basis "E-EVAL-032 pin (PR #684 harvest)"
  :fuel 2048
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (emit EventType/ORGANIZATION_SEEDED
      (probe (fold sum (hyperedges HyperedgeType/COMMUNITY) :as c (fold sum (members-of c HyperedgeType/SECTOR) (field-of it social-class/active)))))))
"#;
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let err = run_once_into(world, rule, &mut graph, &mut sink).unwrap_err();
    assert!(err.contains("E-EVAL-032"), "{err}");
    assert!(err.contains("SECTOR"), "{err}");
    assert!(err.contains("COMMUNITY"), "{err}");
}
