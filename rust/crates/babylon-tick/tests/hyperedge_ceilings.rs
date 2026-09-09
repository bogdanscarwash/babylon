//! Hyperedge-lane Task 4 (E1d, Community port train, plan
//! `docs/superpowers/plans/2026-08-18-community-port.md`): the ceiling
//! supply chain — the driver's `CardinalityCeilings` must be fed from the
//! scenario's own hyperedge census (`LoadedScenario::hyperedge_types` +
//! `max_members_seen`), or EVERY hyperedge-querying rule fails at load.
//!
//! RED→GREEN record: the refusal texts (E-LOAD-045 naming the unceiled
//! type, E-LOAD-042 naming the max-members axis) were pinned first, while
//! the driver passed an empty `max_members` map and minted no
//! `HyperedgeType/*` ceiling; the two refusal pins then INVERTED to the
//! load-clean proofs below when `lib.rs`'s `build_shared_load_inputs` was
//! fed. The measured-bound test is the through-the-driver proof of the
//! `:max-members` axis's arithmetic.
//!
//! The `community/` namespace registration (lib.rs's systems set) landed
//! one task early (Task 7 Step 2's content) precisely so these probes
//! reach the ceiling checks instead of dying at E-LOAD-004.

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_tick::{diagnose_content_set, run_once_into};

/// Two COMMUNITY hyperedges over three classes — the minimal world whose
/// census should feed the ceiling maps (Task 1's own population maps,
/// measured: two hyperedges, longest member list 3).
const SCENARIO: &str = r"
(scenario ft/hyperedge-ceilings
  (defvocabulary HyperedgeType (COMMUNITY))
  (deffield social-class/active int extensive)
  (node alpha NodeType/SOCIAL_CLASS)
  (node beta NodeType/SOCIAL_CLASS)
  (node gamma NodeType/SOCIAL_CLASS)
  (hyperedge new-afrikan HyperedgeType/COMMUNITY (members alpha beta gamma))
  (hyperedge queer HyperedgeType/COMMUNITY (members alpha)))
";

/// A rule folding the type-wide `hyperedges` head — needs a
/// `HyperedgeType/COMMUNITY` ceiling entry.
const RULE_HYPEREDGES_FOLD: &str = r#"
(rule community/probe-hyperedge-count
  :role mechanic :evidence derived :material-basis "probe rule for the hyperedge ceiling supply chain (Task 4 RED) — the community/ namespace was registered one task early (Task 7 Step 2's content) precisely so this probe reaches the ceiling check instead of dying at E-LOAD-004"
  :fuel 512
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active :optional :default 1))
  (when (= active 1))
  (effects
    (emit EventType/ORGANIZATION_SEEDED
      (probe (fold sum (hyperedges HyperedgeType/COMMUNITY) :as c (fold sum (members-of c HyperedgeType/COMMUNITY) (field-of it social-class/active)))))))
"#;

/// A rule folding the subject-relative `members-of` head — needs the
/// `:max-members` axis (E-LOAD-042), not just the type ceiling.
const RULE_MEMBERS_OF_FOLD: &str = r#"
(rule community/probe-membership-count
  :role mechanic :evidence derived :material-basis "probe rule for the :max-members axis (Task 4 RED) — namespace registration as probe-hyperedge-count's"
  :fuel 512
  (domain NodeType/SOCIAL_CLASS)
  (bindings
    (binding active :field social-class/active :optional :default 1))
  (when (= active 1))
  (effects
    (emit EventType/ORGANIZATION_SEEDED
      (probe (fold sum (members-of self HyperedgeType/COMMUNITY) (field-of it social-class/active))))))
"#;

/// Step 4: the two Step-1 probes now LOAD through the real driver. LOAD,
/// not run — the §2.6 query heads are load-registered and bound-checked,
/// but the query EVALUATOR does not serve them until Task 3's slice-3 lane
/// (a tick attempt today errors "lands with slice 3", loudly, never a
/// default); that lane is this train's own next task.
#[test]
fn the_hyperedges_folding_rule_now_loads_through_the_driver() {
    let errors = diagnose_content_set(SCENARIO, None, &[RULE_HYPEREDGES_FOLD]);
    assert!(
        errors.is_empty(),
        "the type-wide fold loads once the census feeds the ceiling map: {errors:?}"
    );
}

#[test]
fn the_members_of_folding_rule_now_loads_through_the_driver() {
    let errors = diagnose_content_set(SCENARIO, None, &[RULE_MEMBERS_OF_FOLD]);
    assert!(
        errors.is_empty(),
        "the members-of fold loads once :max-members is census-fed: {errors:?}"
    );
}

/// Step 4's measured-bound pin: a `members-of` fold with `:fuel 1` refuses
/// E-LOAD-040, and the refusal's computed bound is read back — never
/// hand-computed — matching bound_checker.rs's landed arithmetic shape
/// `2 + query(2) + max_members × body` (`:897-905`). The world's longest
/// member list is 3 (`new-afrikan`), fed from `max_members_seen` — NOT the
/// 0 of an empty map (that variant is the mutation vector below).
#[test]
fn the_measured_e_load_040_bound_matches_the_fold_shape() {
    let underfunded = RULE_MEMBERS_OF_FOLD.replace(":fuel 512", ":fuel 1");
    assert!(underfunded.contains(":fuel 1"));
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let err = run_once_into(SCENARIO, &underfunded, &mut graph, &mut sink).unwrap_err();
    let bound = err
        .split("static bound ")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|n| n.parse::<u64>().ok())
        .unwrap_or_else(|| panic!("no measured bound in the refusal: {err}"));
    // Measured (never hand-computed, per the plan's own law): the refusal
    // reports 15. Decomposed against bound_checker.rs's arithmetic: the
    // fold alone is 2 + query(2) + max_members(3, census-fed from
    // `new-afrikan`'s member list, the world's longest) × body(2, the
    // field-of read) = 10; the rule's full static bound adds the `(when
    // …)` condition and the `emit` + payload overhead, landing at 15. The
    // assertion is against the MEASURED number, so a shape change in any
    // of those pieces reds loudly here.
    assert_eq!(bound, 15, "the E-LOAD-040 refusal's own number: {err}");
    assert!(err.contains("E-LOAD-040"), "{err}");
}
