//! Executable phase-anchor acceptance vectors (PER-17).

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::state_hash::CanonicalState;
use babylon_tick::{diagnose_content_set, run_once_into};

const SCENARIO: &str = include_str!("../content/scenarios/two-classes.bscn");

const VITALITY_RULE: &str = r#"
(rule vitality/z-default-home
  :role mechanic :evidence derived :material-basis "biological reproduction is resolved in the vitality phase"
  :fuel 16
  (bindings (binding wages :field social-class/wages))
  (effects (emit EventType/RUPTURE (probe 1))))
"#;

const BEFORE_SURVIVAL_RULE: &str = r#"
(rule mods/a-before-survival
  :role mechanic :evidence derived :material-basis "a consequence-phase probe runs at its declared governed boundary"
  :fuel 16
  (anchor :before survival)
  (bindings (binding wages :field social-class/wages))
  (effects (emit EventType/RUPTURE (probe 2))))
"#;

fn run(rule_src: &str) -> babylon_tick::TickReport {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, rule_src, &mut graph, &mut sink).expect("phase-ordered tick")
}

#[test]
fn an_explicit_anchor_uses_the_governed_system_spine_not_global_rule_id_order() {
    let forward = format!("{BEFORE_SURVIVAL_RULE}\n{VITALITY_RULE}");
    let reversed = format!("{VITALITY_RULE}\n{BEFORE_SURVIVAL_RULE}");
    let report_a = run(&forward);
    let report_b = run(&reversed);

    let expected = vec![
        ("vitality/z-default-home".to_owned(), 2),
        ("mods/a-before-survival".to_owned(), 2),
    ];
    assert_eq!(report_a.per_rule_fired, expected);
    assert_eq!(report_b.per_rule_fired, expected);
    assert_eq!(report_a.before, report_b.before);
    assert_eq!(report_a.after, report_b.after);
    assert_eq!(report_a.audit_receipts, report_b.audit_receipts);
    assert_eq!(
        report_a
            .audit_receipts
            .iter()
            .map(|receipt| receipt.rule_id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "vitality/z-default-home",
            "vitality/z-default-home",
            "mods/a-before-survival",
            "mods/a-before-survival",
        ]
    );
}

fn assert_rejected_before_hydration(rule_src: &str, code: Option<&str>, text: &str) {
    let error = rejection_before_hydration(rule_src);

    if let Some(code) = code {
        assert!(error.contains(code), "expected {code}, got {error}");
    }
    assert!(error.contains(text), "expected {text:?}, got {error}");
}

fn rejection_before_hydration(rule_src: &str) -> String {
    let mut graph = HypergraphStore::new();
    let before = graph.state_hash().expect("empty hash");
    let mut sink = CollectingSink::default();
    let error = run_once_into(SCENARIO, rule_src, &mut graph, &mut sink)
        .expect_err("invalid phase composition must fail");
    let after = graph.state_hash().expect("post-refusal hash");

    assert_eq!(after, before, "composition failure must precede hydration");
    assert!(sink.events.is_empty(), "composition failure emits nothing");
    error
}

#[test]
fn an_anchor_that_cuts_through_material_base_is_e_load_003_before_hydration() {
    let rule = r#"
(rule mods/illegal-interleave
  :role mechanic :evidence derived :material-basis "a mod cannot splice incidental work into the material causal spine"
  :fuel 16
  (domain :graph)
  (anchor :after vitality)
  (bindings)
  (effects (emit EventType/RUPTURE)))
"#;
    assert_rejected_before_hydration(rule, Some("E-LOAD-003"), "Material Base");
}

#[test]
fn diagnostic_loading_reports_the_same_phase_composition_refusal() {
    let rule = r#"
(rule mods/illegal-diagnostic-interleave
  :role mechanic :evidence derived :material-basis "diagnostics and execution share one causal-placement compiler"
  :fuel 16
  (domain :graph)
  (anchor :after vitality)
  (bindings)
  (effects (emit EventType/RUPTURE)))
"#;
    let errors = diagnose_content_set(SCENARIO, None, &[rule]);

    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].spec_code(), Some("E-LOAD-003"));
    assert!(errors[0].to_string().contains("Material Base"));
}

#[test]
fn rule_surface_failure_precedes_phase_composition_without_hydration() {
    let rule = r#"
(rule mods/invalid-fuel-and-placement
  :role mechanic :evidence derived :material-basis "surface validation remains earlier than causal composition"
  :fuel 0
  (domain :graph)
  (anchor :after vitality)
  (bindings)
  (effects (emit EventType/RUPTURE)))
"#;
    assert_rejected_before_hydration(rule, Some("E-PARSE-012"), ":fuel");
}

#[test]
fn scenario_failure_precedes_phase_composition_without_hydration() {
    let rule = r#"
(rule mods/invalid-placement-after-bad-scenario
  :role mechanic :evidence derived :material-basis "scenario validation remains earlier than causal composition"
  :fuel 16
  (domain :graph)
  (anchor :after vitality)
  (bindings)
  (effects (emit EventType/RUPTURE)))
"#;
    let mut graph = HypergraphStore::new();
    let before = graph.state_hash().expect("empty hash");
    let mut sink = CollectingSink::default();
    let error = run_once_into("(", rule, &mut graph, &mut sink)
        .expect_err("the malformed scenario must fail first");

    assert!(!error.contains("E-LOAD-003"), "{error}");
    assert!(error.contains("scenario"), "{error}");
    assert_eq!(graph.state_hash().expect("refused hash"), before);
    assert!(sink.events.is_empty());
}

#[test]
fn an_anchorless_rule_with_no_system_home_is_e_load_002_before_hydration() {
    let rule = r#"
(rule nowhere/no-home
  :role mechanic :evidence derived :material-basis "an unplaced rule has no causal meaning"
  :fuel 16
  (domain :graph)
  (bindings)
  (effects (emit EventType/RUPTURE)))
"#;
    assert_rejected_before_hydration(rule, Some("E-LOAD-002"), "cannot land nowhere");
}

#[test]
fn duplicate_anchor_forms_fail_before_hydration() {
    let rule = r#"
(rule mods/duplicate-anchor
  :role mechanic :evidence derived :material-basis "one rule has one causal placement"
  :fuel 16
  (domain :graph)
  (anchor :before survival)
  (anchor :after doctrine)
  (bindings)
  (effects (emit EventType/RUPTURE)))
"#;
    assert_rejected_before_hydration(rule, None, "at most one");
}

#[test]
fn invalid_rule_permutations_name_the_same_byte_least_identity() {
    let byte_least = r#"
(rule aaa/no-home
  :role mechanic :evidence derived :material-basis "an unplaced rule must fail deterministically"
  :fuel 16
  (domain :graph)
  (bindings)
  (effects (emit EventType/RUPTURE)))
"#;
    let byte_greater = r#"
(rule zzz/no-home
  :role mechanic :evidence derived :material-basis "another unplaced rule must not win by source order"
  :fuel 16
  (domain :graph)
  (bindings)
  (effects (emit EventType/RUPTURE)))
"#;
    let forward = rejection_before_hydration(&format!("{byte_least}\n{byte_greater}"));
    let reversed = rejection_before_hydration(&format!("{byte_greater}\n{byte_least}"));

    assert_eq!(forward, reversed);
    assert!(forward.contains("E-LOAD-002"), "{forward}");
    assert!(forward.contains("rule aaa/no-home"), "{forward}");
}
