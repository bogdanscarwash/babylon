//! Executable causal-role and audit-receipt contracts for built-in BSL rules
//! and native compositions (PER-19 / ADR224 / ADR255).

use babylon_bsl::causal_contract::{
    effect_footprint, parse_rule_contract, validate_governed_attribution, AllowedEffect,
    EffectSignature, EvidenceClass, RuleRole, GOVERNED_EFFECT_ALLOWANCES,
    GOVERNED_RULE_ATTRIBUTIONS,
};
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_bsl::{reader::read_all, reader::Atom, reader::SExpr};
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::material_staffing::STAFFING_COMPOSITION_ID;
use babylon_tick::{diagnose_content_set_sources, run_once_into, ContentRuleSource};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const SHOCK_PRESENT: &str = r"
(scenario causal/shock-present
  (deffield social-class/wages int extensive)
  (deffield social-class/value-produced int extensive)
  (deffield social-class/imperial-rent int extensive)
  (node core NodeType/SOCIAL_CLASS
    (social-class/wages 120)
    (social-class/value-produced 80))
  (node periphery NodeType/SOCIAL_CLASS
    (social-class/wages 20)
    (social-class/value-produced 90)))
";

const SHOCK_ABSENT: &str = r"
(scenario causal/shock-absent
  (deffield social-class/wages int extensive)
  (deffield social-class/value-produced int extensive)
  (deffield social-class/imperial-rent int extensive)
  (node core NodeType/SOCIAL_CLASS
    (social-class/wages 70)
    (social-class/value-produced 80))
  (node periphery NodeType/SOCIAL_CLASS
    (social-class/wages 20)
    (social-class/value-produced 90)))
";

const MECHANIC: &str = r#"
(rule vitality/causal-probe
  :role mechanic
  :evidence derived
  :material-basis "a wage-value gap is an input that can cause derived rent"
  :fuel 64
  (bindings
    (binding wages :field social-class/wages)
    (binding value-produced :field social-class/value-produced))
  (when (> wages value-produced))
  (effects
    (emit EventType/RUPTURE)
    (update-node self social-class/imperial-rent
      (set (- wages value-produced)))))
"#;

const FAILING_SCENARIO: &str = r"
(scenario causal/failing-receipt
  (deffield social-class/probability probability intensive)
  (node first NodeType/SOCIAL_CLASS (social-class/probability 0.1p))
  (node second NodeType/SOCIAL_CLASS (social-class/probability 0.9p)))
";

const FAILING_RULE: &str = r#"
(rule vitality/failing-receipt
  :role mechanic
  :evidence designed
  :material-basis "a refused store-boundary write aborts the whole working tick"
  :fuel 64
  (bindings (binding probability :field social-class/probability))
  (effects
    (emit EventType/RUPTURE)
    (update-node self social-class/probability (add 0.4i))))
"#;

const RECOGNIZER_WITH_VERB_LIKE_PAYLOAD_LABELS: &str = r#"
(rule control-ratio/c03-crisis
  :role recognizer
  :evidence derived
  :material-basis "payload labels describe evidence rather than invoke effects"
  :fuel 32
  (bindings (binding wages :field social-class/wages))
  (when (> wages 0))
  (effects
    (emit EventType/CONTROL_RATIO_CRISIS (add-node wages) (emit 2))))
"#;

const ORDERING_SCENARIO: &str = r"
(scenario causal/ordering
  (deffield social-class/source int extensive)
  (deffield social-class/lagged int extensive)
  (deffield social-class/output int extensive)
  (node core NodeType/SOCIAL_CLASS (social-class/source 7)))
";

const RANK_HAZARD: &str = r#"
(rule solidarity/z-reader
  :role mechanic :evidence derived :material-basis "an early phase reads a later write" :fuel 32
  (bindings (binding lagged :field social-class/lagged :optional :default 0))
  (effects (update-node self social-class/output (set lagged))))
(rule consciousness/a-writer
  :role mechanic :evidence derived :material-basis "the later phase supplies the field" :fuel 32
  (bindings (binding source :field social-class/source))
  (effects (update-node self social-class/lagged (set source))))
"#;

const RANK_CLEAN_INVERSE: &str = r#"
(rule solidarity/z-writer
  :role mechanic :evidence derived :material-basis "the early phase supplies the field" :fuel 32
  (bindings (binding source :field social-class/source))
  (effects (update-node self social-class/lagged (set source))))
(rule consciousness/a-reader
  :role mechanic :evidence derived :material-basis "the later phase reads the committed write" :fuel 32
  (bindings (binding lagged :field social-class/lagged :optional :default 0))
  (effects (update-node self social-class/output (set lagged))))
"#;

const CONDITIONAL_FAN_IN: &str = r#"
(rule solidarity/a-flow
  :role mechanic :evidence derived :material-basis "one conditional flow" :fuel 32
  (bindings (binding source :field social-class/source))
  (when (> source 0))
  (effects (update-node self social-class/lagged (add source))))
(rule consciousness/b-flow
  :role mechanic :evidence derived :material-basis "a second conditional flow" :fuel 32
  (bindings (binding source :field social-class/source))
  (when (> source 0))
  (effects (update-node self social-class/lagged (add source))))
"#;

const NESTED_GUARDED_RESET_FAN_IN: &str = r#"
(rule solidarity/a-reset
  :role mechanic :evidence derived :material-basis "a skipped nested set is not a reset" :fuel 32
  (bindings)
  (effects (guard #f (update-node self social-class/lagged (set 0)))))
(rule consciousness/b-flow
  :role mechanic :evidence derived :material-basis "a later flow still requires a reset" :fuel 32
  (bindings (binding source :field social-class/source))
  (effects (update-node self social-class/lagged (add source))))
"#;

const MAX_PRODUCTION_RULE_PACK_FILES: usize = 64;
const MAX_PRODUCTION_RULE_DIR_ENTRIES: usize = 128;
const MAX_FORMS_PER_RULE_PACK: usize = 256;

fn bounded_rule_pack_paths(paths: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for (entry_count, path) in paths.into_iter().enumerate() {
        assert!(
            entry_count < MAX_PRODUCTION_RULE_DIR_ENTRIES,
            "production rule directory exceeds its {MAX_PRODUCTION_RULE_DIR_ENTRIES}-entry bound"
        );
        if path.extension().is_none_or(|extension| extension != "bsl") {
            continue;
        }
        assert!(
            files.len() < MAX_PRODUCTION_RULE_PACK_FILES,
            "production rule packs exceed their {MAX_PRODUCTION_RULE_PACK_FILES}-file bound"
        );
        files.push(path);
    }
    files.sort();
    files
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum StaticEffect {
    NodeField(String),
    EdgeField(String),
    HyperedgeField(String),
    Event(String),
    Shape(String),
}

fn role_name(role: RuleRole) -> &'static str {
    match role {
        RuleRole::Mechanic => "mechanic",
        RuleRole::Recognizer => "recognizer",
        RuleRole::ExternalEvent => "external-event",
        RuleRole::Intent => "intent",
    }
}

fn footprint_effect(effect: &EffectSignature) -> StaticEffect {
    match effect {
        EffectSignature::NodeField(field) => StaticEffect::NodeField(field.clone()),
        EffectSignature::EdgeField(field) => StaticEffect::EdgeField(field.clone()),
        EffectSignature::HyperedgeField(field) => StaticEffect::HyperedgeField(field.clone()),
        EffectSignature::Event(event) => StaticEffect::Event(event.clone()),
        EffectSignature::Shape(verb) => StaticEffect::Shape(format!("{verb:?}")),
    }
}

fn allowed_effect(effect: AllowedEffect) -> StaticEffect {
    match effect {
        AllowedEffect::NodeField(field) => StaticEffect::NodeField(field.to_owned()),
        AllowedEffect::EdgeField(field) => StaticEffect::EdgeField(field.to_owned()),
        AllowedEffect::HyperedgeField(field) => StaticEffect::HyperedgeField(field.to_owned()),
        AllowedEffect::Event(event) => StaticEffect::Event(event.to_owned()),
    }
}

fn governed_restricted_effects() -> BTreeMap<(String, &'static str), BTreeSet<StaticEffect>> {
    let mut exact_keys = BTreeSet::new();
    let mut by_rule: BTreeMap<(String, &'static str), BTreeSet<StaticEffect>> = BTreeMap::new();
    for row in GOVERNED_EFFECT_ALLOWANCES {
        let effect = allowed_effect(row.effect);
        let key = (row.rule_id.to_owned(), role_name(row.role));
        assert!(
            exact_keys.insert((key.clone(), effect.clone())),
            "duplicate governed effect allowance for {key:?}: {effect:?}"
        );
        by_rule.entry(key).or_default().insert(effect);
    }
    by_rule
}

fn run(scenario: &str, rule: &str) -> (babylon_tick::TickReport, HypergraphStore) {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(scenario, rule, &mut graph, &mut sink)
        .expect("the causal probe must adjudicate");
    (report, graph)
}

fn assert_aggregate_refusal(rule_source: &str, code: &str) {
    let mut graph = HypergraphStore::new();
    let before = graph.state_hash().expect("empty graph hashes");
    let mut sink = CollectingSink::default();
    let error = run_once_into(ORDERING_SCENARIO, rule_source, &mut graph, &mut sink)
        .expect_err("the aggregate composition gate must refuse");
    assert!(error.contains(code), "{error}");
    assert_eq!(graph.state_hash().expect("refused graph hashes"), before);
    assert!(sink.events.is_empty());

    let diagnostics = diagnose_content_set_sources(
        ORDERING_SCENARIO,
        None,
        &[ContentRuleSource {
            source_id: "rules/aggregate-refusal.bsl",
            source: rule_source,
        }],
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.error.to_string().contains(code)),
        "diagnostic path did not report {code}: {diagnostics:?}"
    );
}

#[test]
fn a_mechanic_derives_an_outcome_and_receipts_actual_event_then_write() {
    let (report, graph) = run(SHOCK_PRESENT, MECHANIC);

    assert_eq!(report.fired, 1);
    assert_eq!(
        graph
            .node_attribute(NodeId(0), "social-class/imperial-rent")
            .unwrap()
            .to_bits(),
        40.0_f64.to_bits()
    );
    assert_eq!(report.audit_receipts.len(), 2);
    assert_eq!(report.audit_receipts[0].role, RuleRole::Mechanic);
    assert_eq!(report.audit_receipts[0].evidence, EvidenceClass::Derived);
    assert_eq!(report.audit_receipts[0].ordinal, 0);
    assert_eq!(
        report.audit_receipts[0].effect,
        EffectSignature::Event("EventType/RUPTURE".to_owned())
    );
    assert_eq!(report.audit_receipts[1].ordinal, 1);
    assert_eq!(
        report.audit_receipts[1].effect,
        EffectSignature::NodeField("social-class/imperial-rent".to_owned())
    );
}

#[test]
fn severing_the_input_condition_removes_the_downstream_state_and_receipts() {
    let (connected, connected_graph) = run(SHOCK_PRESENT, MECHANIC);
    let (severed, severed_graph) = run(SHOCK_ABSENT, MECHANIC);

    assert_ne!(connected.after, severed.after);
    assert_eq!(connected.audit_receipts.len(), 2);
    assert!(severed.audit_receipts.is_empty());
    assert!(connected_graph
        .node_attribute(NodeId(0), "social-class/imperial-rent")
        .is_ok());
    assert!(severed_graph
        .node_attribute(NodeId(0), "social-class/imperial-rent")
        .is_err());
}

#[test]
fn restricted_roles_cannot_author_an_outcome_field() {
    for role in ["recognizer", "external-event", "intent"] {
        let rule = format!(
            r#"(rule vitality/restricted-probe
  :role {role}
  :evidence derived
  :material-basis "a restricted producer cannot author derived rent"
  :fuel 32
  (bindings (binding wages :field social-class/wages))
  (effects (update-node self social-class/imperial-rent (set wages))))"#
        );
        let mut graph = HypergraphStore::new();
        let before = graph.state_hash().unwrap();
        let mut sink = CollectingSink::default();
        let error = run_once_into(SHOCK_PRESENT, &rule, &mut graph, &mut sink)
            .expect_err("the default-deny contract must refuse the outcome write");

        assert!(error.contains("E-LOAD-060"), "{role}: {error}");
        assert!(
            error.contains("social-class/imperial-rent"),
            "{role}: {error}"
        );
        assert_eq!(graph.state_hash().unwrap(), before);
        assert!(sink.events.is_empty());
    }
}

#[test]
fn verb_like_emit_payload_labels_survive_the_composed_load_and_tick_path() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(
        SHOCK_PRESENT,
        RECOGNIZER_WITH_VERB_LIKE_PAYLOAD_LABELS,
        &mut graph,
        &mut sink,
    )
    .expect("payload labels must not fabricate restricted-role effects");

    assert_eq!(report.fired, 2);
    assert_eq!(sink.events.len(), 2);
    assert_eq!(sink.events[0].0, "CONTROL_RATIO_CRISIS");
    assert_eq!(sink.events[0].1.len(), 2);
    assert_eq!(sink.events[0].1[0].0, "add-node");
    assert_eq!(sink.events[0].1[1].0, "emit");
}

#[test]
fn a_failed_working_tick_publishes_neither_effects_nor_a_receipt_report() {
    let mut graph = HypergraphStore::new();
    let before = graph.state_hash().unwrap();
    let mut sink = CollectingSink::default();

    let result = run_once_into(FAILING_SCENARIO, FAILING_RULE, &mut graph, &mut sink);

    assert!(result.is_err());
    assert_eq!(graph.state_hash().unwrap(), before);
    assert!(sink.events.is_empty());
}

#[test]
fn aggregate_loader_refuses_a_rank_aware_stale_default_before_publication() {
    assert!("consciousness/a-writer" < "solidarity/z-reader");
    assert_aggregate_refusal(RANK_HAZARD, "E-LOAD-058");
}

#[test]
fn aggregate_loader_accepts_the_phase_rank_clean_inverse() {
    assert!("consciousness/a-reader" < "solidarity/z-writer");
    assert!(diagnose_content_set_sources(
        ORDERING_SCENARIO,
        None,
        &[ContentRuleSource {
            source_id: "rules/rank-clean-inverse.bsl",
            source: RANK_CLEAN_INVERSE
        }]
    )
    .is_empty());
    let (report, graph) = run(ORDERING_SCENARIO, RANK_CLEAN_INVERSE);
    assert_eq!(report.fired, 2);
    assert_eq!(
        graph
            .node_attribute(NodeId(0), "social-class/output")
            .expect("the later reader sees the earlier phase write"),
        7.0
    );
}

#[test]
fn aggregate_loader_refuses_unreset_conditional_fan_in_before_publication() {
    assert_aggregate_refusal(CONDITIONAL_FAN_IN, "E-LOAD-059");
}

#[test]
fn aggregate_loader_does_not_treat_a_skipped_nested_set_as_a_reset() {
    assert_aggregate_refusal(NESTED_GUARDED_RESET_FAN_IN, "E-LOAD-059");
}

#[test]
fn governed_effect_allowance_keys_are_unique() {
    let governed = governed_restricted_effects();
    let unique_effect_count = governed.values().map(BTreeSet::len).sum::<usize>();
    assert_eq!(unique_effect_count, GOVERNED_EFFECT_ALLOWANCES.len());
}

#[test]
fn production_rule_pack_selection_filters_before_applying_the_bsl_bound() {
    let mut paths = (0..65)
        .map(|index| PathBuf::from(format!("mixed-{index}.txt")))
        .collect::<Vec<_>>();
    let late_rule = PathBuf::from("zz-late-rule.bsl");
    paths.push(late_rule.clone());
    assert_eq!(bounded_rule_pack_paths(paths), vec![late_rule]);
}

#[test]
fn every_production_rule_identity_is_governed_independently_of_its_content() {
    let rules_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/content/rules");
    let paths = std::fs::read_dir(rules_dir)
        .unwrap_or_else(|error| panic!("cannot read {rules_dir}: {error}"))
        .map(|entry| entry.expect("production rule directory entry").path());
    let files = bounded_rule_pack_paths(paths);

    let mut parsed_rule_count = 0_usize;
    let mut production_ids = BTreeSet::new();
    let mut production_restricted_effects = BTreeMap::new();
    for path in files {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let forms = read_all(source.as_bytes())
            .unwrap_or_else(|error| panic!("cannot parse {}: {error:?}", path.display()));
        assert!(forms.len() <= MAX_FORMS_PER_RULE_PACK);
        for form in forms.into_iter().take(MAX_FORMS_PER_RULE_PACK) {
            let is_rule = matches!(
                &form,
                SExpr::List(items)
                    if matches!(items.first(), Some(SExpr::Atom(Atom::Symbol(head))) if head == "rule")
            );
            if !is_rule {
                continue;
            }
            let contract = parse_rule_contract(&form)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            validate_governed_attribution(&contract)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            parsed_rule_count += 1;
            if contract.role != RuleRole::Mechanic {
                let effects = effect_footprint(&form)
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
                    .iter()
                    .map(footprint_effect)
                    .collect::<BTreeSet<_>>();
                let key = (contract.rule_id.clone(), role_name(contract.role));
                assert!(
                    production_restricted_effects
                        .insert(key.clone(), effects)
                        .is_none(),
                    "duplicate restricted production rule identity: {key:?}"
                );
            }
            production_ids.insert(contract.rule_id);
        }
    }

    let governed_ids = GOVERNED_RULE_ATTRIBUTIONS
        .iter()
        .map(|row| row.rule_id.to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        parsed_rule_count,
        production_ids.len(),
        "duplicate production rule ID"
    );
    assert!(
        production_ids.insert(STAFFING_COMPOSITION_ID.to_owned()),
        "native staffing composition duplicates a production BSL rule ID"
    );
    assert_eq!(production_ids, governed_ids);
    let mut governed_effect_sets = governed_restricted_effects();
    for key in production_restricted_effects.keys() {
        governed_effect_sets.entry(key.clone()).or_default();
    }
    assert_eq!(production_restricted_effects, governed_effect_sets);
}
