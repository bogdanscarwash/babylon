//! `diagnose_content_set` — the structured, multi-error diagnostics seam
//! (#652 Task 3, plan §3.4). Unlike [`babylon_tick::run_once`]'s own
//! `prepare_rules`, which stops at the FIRST failing stage, this function
//! collects every INDEPENDENT failure a content set has — the `bsl-ls`
//! diagnostics seam (wave 1) needs a full report, not just the first
//! problem.
//!
//! Four rows (plan §3.1):
//!
//! 1. A clean content set diagnoses empty.
//! 2. A scenario whose `deffield`s collide with D32's implicit
//!    `<edge-type>/strength` field yields exactly one entry, coded
//!    `E-LOAD-001`.
//! 3. Two independently-broken `(rule …)` forms yield two entries — one bad
//!    rule cannot hide the other.
//! 4. **The row revision 1 missed:** the SAME D32 collision (this time with
//!    two colliding fields, proving the byte-least-name determinism reaches
//!    this public seam too) yields a `spec_code()` that is structured DATA
//!    — `Some("E-LOAD-001")`, never `None` — not merely a substring of the
//!    formatted message. Revision 1's four-variant `PrepareError` design
//!    would have passed a `.to_string().contains("E-LOAD-001")` check while
//!    silently returning `None` from `spec_code()`, because its
//!    `Composition` variant carried no `code`/`identity` fields at all.

use babylon_tick::diagnose_content_set;

const SCENARIO: &str = include_str!("../content/scenarios/two-classes.bscn");
const RULE: &str = include_str!("../content/rules/fundamental-theorem.bsl");

/// One explicit `deffield` re-declaring D32's implicit `<edge-type>/
/// strength` field — the same shape `babylon_tick::lib`'s own internal
/// `D32_WIRING_PROBE_SCENARIO_WITH_EXPLICIT_REDECLARATION` fixture uses,
/// reproduced here because `tests/*.rs` files compile as separate crates
/// and cannot reach `lib.rs`'s `#[cfg(test)]`-only constants.
const D32_SINGLE_COLLISION_SCENARIO: &str = r"
(scenario diagnose-seam/d32-single-collision
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defvocabulary EdgeType (SOLIDARITY))
  (deffield social-class/shape int extensive)
  (deffield solidarity/strength int extensive)
  (node core NodeType/SOCIAL_CLASS (social-class/shape 1))
  (node other NodeType/SOCIAL_CLASS (social-class/shape 1))
  (edge EdgeType/SOLIDARITY core other 1))
";

/// TWO explicit re-declarations colliding against the implicit seed set —
/// the byte-least-qname-naming determinism proof, reproduced from
/// `lib.rs`'s own internal `D32_TWO_COLLISION_SCENARIO` fixture for the
/// same reason as [`D32_SINGLE_COLLISION_SCENARIO`].
const D32_TWO_COLLISION_SCENARIO: &str = r"
(scenario diagnose-seam/d32-two-collision
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defvocabulary EdgeType (SOLIDARITY TENANCY))
  (deffield social-class/shape int extensive)
  (deffield solidarity/strength int extensive)
  (deffield tenancy/strength int extensive)
  (node core NodeType/SOCIAL_CLASS (social-class/shape 1))
  (node other NodeType/SOCIAL_CLASS (social-class/shape 1))
  (edge EdgeType/SOLIDARITY core other 1))
";

/// The probe rule both D32 scenarios above load against — reads the
/// implicit `solidarity/strength` field through a `neighbors` fold, the
/// same shape `lib.rs`'s own `D32_WIRING_PROBE_RULE` uses. The composition
/// check runs BEFORE any rule loads, so this rule's own body is never
/// reached when the scenario collides — its only job is to give
/// `split_content` one legal `(rule …)` form to split out.
const D32_RULE: &str = r#"(rule vitality/d32-diagnose-seam-probe
  :role mechanic :evidence derived :material-basis "diagnose_content_set rows 2/4 (#652 Task 3): the D32 implicit-strength collision must survive as structured spec_code() data, not just message text"
  :fuel 128
  (bindings (binding shape :field social-class/shape))
  (when (= shape 1))
  (effects (emit EventType/PROBE
    (s (fold sum (neighbors self EdgeType/SOLIDARITY :any NodeType/SOCIAL_CLASS)
             (field-of it solidarity/strength))))))"#;

/// Row 3's first independently-broken rule: `:fuel 0` is out of range
/// (`E-PARSE-012`) — a pure surface-stage rejection, no scenario field
/// dependency.
const BROKEN_RULE_FUEL: &str = r#"(rule vitality/broken-fuel
  :role mechanic :evidence derived :material-basis "diagnose_content_set row 3 (#652 Task 3): an out-of-range :fuel is its own independent entry"
  :fuel 0
  (bindings (binding wages :field social-class/wages))
  (when (> wages 0))
  (effects (emit EventType/PROBE)))"#;

/// Row 3's second independently-broken rule: an empty `:material-basis` is
/// `E-PARSE-011` — a DIFFERENT surface-stage rejection than
/// [`BROKEN_RULE_FUEL`]'s, so the row proves two DISTINCT failures each get
/// their own entry, not that the same failure is merely counted twice.
const BROKEN_RULE_MATERIAL_BASIS: &str = r#"(rule vitality/broken-material-basis
  :role mechanic :evidence derived :material-basis ""
  :fuel 64
  (bindings (binding wages :field social-class/wages))
  (when (> wages 0))
  (effects (emit EventType/PROBE)))"#;

const ILLEGAL_PHASE_RULE: &str = r#"(rule mods/illegal-material-base-interleave
  :role mechanic :evidence derived :material-basis "an independent causal-composition refusal"
  :fuel 64
  (domain :graph)
  (anchor :after vitality)
  (bindings)
  (effects (emit EventType/PROBE)))"#;

const DUPLICATE_ID_WITH_ILLEGAL_PHASE: &str = r#"(rule economics/fundamental-theorem
  :role mechanic :evidence derived :material-basis "duplicate identity makes phase placement undefined"
  :fuel 64
  (domain :graph)
  (anchor :after vitality)
  (bindings)
  (effects (emit EventType/PROBE)))"#;

#[test]
fn a_clean_content_set_diagnoses_empty() {
    let errors = diagnose_content_set(SCENARIO, None, &[RULE]);
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn a_scenario_with_a_duplicate_deffield_yields_one_e_load_001_entry() {
    let errors = diagnose_content_set(D32_SINGLE_COLLISION_SCENARIO, None, &[D32_RULE]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].spec_code(), Some("E-LOAD-001"));
}

#[test]
fn two_independently_broken_rule_forms_yield_two_entries() {
    let errors = diagnose_content_set(
        SCENARIO,
        None,
        &[BROKEN_RULE_FUEL, BROKEN_RULE_MATERIAL_BASIS],
    );
    assert_eq!(errors.len(), 2, "{errors:?}");
}

#[test]
fn a_scenario_that_redeclares_an_implicit_strength_field_yields_a_structured_code_not_none() {
    let errors = diagnose_content_set(D32_TWO_COLLISION_SCENARIO, None, &[D32_RULE]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_ne!(errors[0].spec_code(), None, "{errors:?}");
    assert_eq!(errors[0].spec_code(), Some("E-LOAD-001"));
}

#[test]
fn a_bad_rule_does_not_hide_an_independent_phase_composition_failure() {
    let errors = diagnose_content_set(SCENARIO, None, &[BROKEN_RULE_FUEL, ILLEGAL_PHASE_RULE]);

    assert_eq!(errors.len(), 2, "{errors:?}");
    assert_eq!(errors[0].spec_code(), Some("E-PARSE-012"));
    assert_eq!(errors[1].spec_code(), Some("E-LOAD-003"));
}

#[test]
fn duplicate_rule_ids_across_sources_are_one_structured_e_load_001() {
    let errors = diagnose_content_set(SCENARIO, None, &[RULE, RULE]);

    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].spec_code(), Some("E-LOAD-001"));
    assert!(errors[0].to_string().contains("fundamental-theorem"));
}

#[test]
fn duplicate_rule_ids_suppress_phase_diagnostics_in_both_source_orders() {
    for sources in [
        [RULE, DUPLICATE_ID_WITH_ILLEGAL_PHASE],
        [DUPLICATE_ID_WITH_ILLEGAL_PHASE, RULE],
    ] {
        let errors = diagnose_content_set(SCENARIO, None, &sources);

        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].spec_code(), Some("E-LOAD-001"));
    }
}
