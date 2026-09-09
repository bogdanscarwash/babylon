use std::fmt::Write as _;

use babylon_bsl::scenario::{load_scenario, load_scenario_with_prelude, ScenarioError};
use babylon_graph::memory::MemoryGraph;
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::GraphSubstrate;

const PRACTICE_PRELUDE: &str = "\
(deffield organization/action-budget int intensive)\n\
(deffield organization/active int intensive)\n";

fn scenario(body: &str) -> String {
    format!(
        "(scenario test/practice-topology\n\
         (deffield organization/action-budget int intensive)\n\
         (deffield organization/active int intensive)\n{body})"
    )
}

fn prelude_scenario(body: &str) -> String {
    format!("(scenario test/practice-topology\n{body})")
}

fn assert_direct_refusal_preserves_graph(source: &str, code: &'static str) {
    let mut subject = MemoryGraph::new();
    let mut control = MemoryGraph::new();
    assert_eq!(
        subject.add_node("SEED").unwrap(),
        control.add_node("SEED").unwrap()
    );
    let before = control.state_hash().unwrap();
    let error = load_scenario(source, &mut subject).unwrap_err();
    assert_eq!(error.code, Some(code), "{error}");
    assert_eq!(subject.state_hash().unwrap(), before);
    assert_eq!(
        subject.add_node("AFTER").unwrap(),
        control.add_node("AFTER").unwrap()
    );
    assert_eq!(subject.state_hash().unwrap(), control.state_hash().unwrap());
}

fn assert_prelude_refusal_preserves_graph(prelude: &str, source: &str, code: &'static str) {
    let mut subject = MemoryGraph::new();
    let mut control = MemoryGraph::new();
    assert_eq!(
        subject.add_node("SEED").unwrap(),
        control.add_node("SEED").unwrap()
    );
    let before = control.state_hash().unwrap();
    let error = load_scenario_with_prelude(prelude, source, &mut subject).unwrap_err();
    assert_eq!(error.code, Some(code), "{error}");
    assert_eq!(subject.state_hash().unwrap(), before);
    assert_eq!(
        subject.add_node("AFTER").unwrap(),
        control.add_node("AFTER").unwrap()
    );
    assert_eq!(subject.state_hash().unwrap(), control.state_hash().unwrap());
}

fn organization_source(count: usize) -> String {
    let mut body = String::new();
    for index in 0..4_097 {
        if index == count {
            break;
        }
        writeln!(&mut body, "  (node org-{index} NodeType/ORGANIZATION)")
            .expect("writing to a String cannot fail");
    }
    scenario(&body)
}

fn solidarity_source(edge_count: usize) -> String {
    let mut body = String::from(
        "  (node org NodeType/ORGANIZATION\n\
         (organization/active 1)\n\
         (organization/action-budget 1))\n",
    );
    for index in 0..257 {
        if index == edge_count {
            break;
        }
        writeln!(&mut body, "  (node class-{index} NodeType/SOCIAL_CLASS)")
            .expect("writing to a String cannot fail");
    }
    for index in 0..257 {
        if index == edge_count {
            break;
        }
        writeln!(
            &mut body,
            "  (edge EdgeType/SOLIDARITY org class-{index} 1)"
        )
        .expect("writing to a String cannot fail");
    }
    scenario(&body)
}

fn prelude_organization_source(count: usize) -> String {
    let mut body = String::new();
    for index in 0..4_097 {
        if index == count {
            break;
        }
        writeln!(&mut body, "  (node org-{index} NodeType/ORGANIZATION)")
            .expect("writing to a String cannot fail");
    }
    prelude_scenario(&body)
}

fn prelude_solidarity_source(edge_count: usize) -> String {
    let mut body = String::from(
        "  (node org NodeType/ORGANIZATION\n\
         (organization/active 1)\n\
         (organization/action-budget 1))\n",
    );
    for index in 0..257 {
        if index == edge_count {
            break;
        }
        writeln!(&mut body, "  (node class-{index} NodeType/SOCIAL_CLASS)")
            .expect("writing to a String cannot fail");
    }
    for index in 0..257 {
        if index == edge_count {
            break;
        }
        writeln!(
            &mut body,
            "  (edge EdgeType/SOLIDARITY org class-{index} 1)"
        )
        .expect("writing to a String cannot fail");
    }
    prelude_scenario(&body)
}

#[test]
fn exact_organization_maximum_loads_and_plus_one_is_e_load_061() {
    let mut graph = MemoryGraph::new();
    let loaded = load_scenario(&organization_source(4_096), &mut graph).unwrap();
    assert_eq!(loaded.node_count, 4_096);
    assert_direct_refusal_preserves_graph(&organization_source(4_097), "E-LOAD-061");
}

#[test]
fn exact_solidarity_maximum_loads_and_plus_one_is_e_load_062() {
    let mut graph = MemoryGraph::new();
    let loaded = load_scenario(&solidarity_source(256), &mut graph).unwrap();
    assert_eq!(loaded.edge_count, 256);
    assert_direct_refusal_preserves_graph(&solidarity_source(257), "E-LOAD-062");
}

#[test]
fn prelude_loader_pins_exact_organization_and_edge_boundaries() {
    let mut organization_graph = MemoryGraph::new();
    load_scenario_with_prelude(
        PRACTICE_PRELUDE,
        &prelude_organization_source(4_096),
        &mut organization_graph,
    )
    .unwrap();
    assert_prelude_refusal_preserves_graph(
        PRACTICE_PRELUDE,
        &prelude_organization_source(4_097),
        "E-LOAD-061",
    );

    let mut edge_graph = MemoryGraph::new();
    load_scenario_with_prelude(
        PRACTICE_PRELUDE,
        &prelude_solidarity_source(256),
        &mut edge_graph,
    )
    .unwrap();
    assert_prelude_refusal_preserves_graph(
        PRACTICE_PRELUDE,
        &prelude_solidarity_source(257),
        "E-LOAD-062",
    );
}

#[test]
fn active_missing_budget_and_invalid_budget_are_pre_mutation_refusals() {
    let missing = scenario("  (node org NodeType/ORGANIZATION (organization/active 1))\n");
    assert_direct_refusal_preserves_graph(&missing, "E-LOAD-063");
    let invalid = scenario(
        "  (node org NodeType/ORGANIZATION\n\
         (organization/active 1)\n\
         (organization/action-budget -1))\n",
    );
    assert_direct_refusal_preserves_graph(&invalid, "E-LOAD-064");
}

#[test]
fn final_registry_signature_refusals_cover_both_loader_entry_points() {
    let direct = "\
(scenario test/practice-topology\n\
  (deffield organization/action-budget int intensive))";
    assert_direct_refusal_preserves_graph(direct, "E-LOAD-065");

    let prelude = "(deffield organization/action-budget int intensive)\n";
    assert_prelude_refusal_preserves_graph(
        prelude,
        "(scenario test/practice-topology)",
        "E-LOAD-065",
    );
}

#[test]
fn every_wrong_final_signature_is_e_load_065() {
    for (budget_decl, active_decl) in [
        (
            "(deffield organization/action-budget real intensive)",
            "(deffield organization/active int intensive)",
        ),
        (
            "(deffield organization/action-budget int extensive)",
            "(deffield organization/active int intensive)",
        ),
        (
            "(deffield organization/action-budget int intensive)",
            "(deffield organization/active real intensive)",
        ),
        (
            "(deffield organization/action-budget int intensive)",
            "(deffield organization/active int extensive)",
        ),
    ] {
        let source = format!("(scenario test/practice-topology\n  {budget_decl}\n  {active_decl})");
        assert_direct_refusal_preserves_graph(&source, "E-LOAD-065");
    }
}

#[test]
fn effective_last_active_and_budget_attributes_match_loader_order() {
    let inactive = scenario(
        "  (node org NodeType/ORGANIZATION\n\
         (organization/active 1)\n\
         (organization/active 0))\n",
    );
    load_scenario(&inactive, &mut MemoryGraph::new()).unwrap();

    let active = scenario(
        "  (node org NodeType/ORGANIZATION\n\
         (organization/active 0)\n\
         (organization/active 1))\n",
    );
    assert_direct_refusal_preserves_graph(&active, "E-LOAD-063");

    let valid_budget = scenario(
        "  (node org NodeType/ORGANIZATION\n\
         (organization/active 1)\n\
         (organization/action-budget -1)\n\
         (organization/action-budget 1))\n",
    );
    load_scenario(&valid_budget, &mut MemoryGraph::new()).unwrap();

    let invalid_budget = scenario(
        "  (node org NodeType/ORGANIZATION\n\
         (organization/active 1)\n\
         (organization/action-budget 1)\n\
         (organization/action-budget -1))\n",
    );
    assert_direct_refusal_preserves_graph(&invalid_budget, "E-LOAD-064");
}

#[test]
fn only_exact_organization_rows_count_toward_the_limit() {
    let mut body = String::new();
    for index in 0..4_097 {
        writeln!(&mut body, "  (node class-{index} NodeType/SOCIAL_CLASS)")
            .expect("writing to a String cannot fail");
    }
    body.push_str("  (node org NodeType/ORGANIZATION)\n");
    let mut graph = MemoryGraph::new();
    let loaded = load_scenario(&scenario(&body), &mut graph).unwrap();
    assert_eq!(loaded.node_count, 4_098);
}

#[test]
fn duplicate_canonical_solidarity_triple_stays_e_load_044_before_mutation() {
    let source = scenario(
        "  (node org NodeType/ORGANIZATION)\n\
         (node class NodeType/SOCIAL_CLASS)\n\
         (edge EdgeType/SOLIDARITY org class 1)\n\
         (edge EdgeType/SOLIDARITY org class 1)\n",
    );
    assert_direct_refusal_preserves_graph(&source, "E-LOAD-044");
}

#[test]
fn organization_to_organization_solidarity_does_not_count_toward_class_footprint() {
    let mut body = String::from(
        "  (node org NodeType/ORGANIZATION)\n\
         (node other-org NodeType/ORGANIZATION)\n",
    );
    for index in 0..256 {
        writeln!(&mut body, "  (node class-{index} NodeType/SOCIAL_CLASS)")
            .expect("writing to a String cannot fail");
    }
    for index in 0..256 {
        writeln!(
            &mut body,
            "  (edge EdgeType/SOLIDARITY org class-{index} 1)"
        )
        .expect("writing to a String cannot fail");
    }
    body.push_str("  (edge EdgeType/SOLIDARITY org other-org 1)\n");
    let loaded = load_scenario(&scenario(&body), &mut MemoryGraph::new()).unwrap();
    assert_eq!(loaded.edge_count, 257);
}

#[test]
fn prelude_entry_point_enforces_topology_before_mutation() {
    let source = prelude_scenario("  (node org NodeType/ORGANIZATION (organization/active 1))\n");
    assert_prelude_refusal_preserves_graph(PRACTICE_PRELUDE, &source, "E-LOAD-063");
}

#[test]
fn source_byte_maximum_loads_and_plus_one_refuses_before_reading() {
    const MAX_SOURCE_BYTES: usize = 4_194_304;
    let base = "(scenario test/practice-topology)";
    let exact = format!("{base}{}", " ".repeat(MAX_SOURCE_BYTES - base.len()));
    assert_eq!(exact.len(), MAX_SOURCE_BYTES);
    load_scenario(&exact, &mut MemoryGraph::new()).unwrap();
    let plus_one = format!("{exact} ");
    let error = load_scenario(&plus_one, &mut MemoryGraph::new()).unwrap_err();
    assert!(error.message.contains("4,194,304"), "{error}");
}

fn scenario_with_body_form_count(count: usize) -> String {
    let mut source = String::from(
        "(scenario test/practice-topology\n\
         (deffield organization/action-budget int intensive)\n\
         (deffield organization/active int intensive)\n",
    );
    for index in 0..65_535 {
        if index + 2 == count {
            break;
        }
        writeln!(&mut source, "  (defconst limit/v{index} 0)")
            .expect("writing to a String cannot fail");
    }
    source.push(')');
    source
}

#[test]
fn body_form_maximum_loads_and_plus_one_refuses() {
    let exact = scenario_with_body_form_count(65_536);
    load_scenario(&exact, &mut MemoryGraph::new()).unwrap();
    let plus_one = scenario_with_body_form_count(65_537);
    let error = load_scenario(&plus_one, &mut MemoryGraph::new()).unwrap_err();
    assert!(error.message.contains("65,536-form"), "{error}");
}

fn nested_body_form(list_depth: usize) -> String {
    let mut nested = "(".repeat(list_depth);
    nested.push_str(&")".repeat(list_depth));
    scenario(&format!("  {nested}\n"))
}

#[test]
fn walker_depth_maximum_reaches_semantic_loading_and_plus_one_refuses() {
    let exact_error = load_scenario(&nested_body_form(255), &mut MemoryGraph::new()).unwrap_err();
    assert!(
        !exact_error.message.contains("walker depth"),
        "{exact_error}"
    );
    let error = load_scenario(&nested_body_form(256), &mut MemoryGraph::new()).unwrap_err();
    assert!(
        error.message.contains("walker depth bound of 256"),
        "{error}"
    );
}

fn wide_body_form(child_count: usize) -> String {
    scenario(&format!("  ({})\n", "()".repeat(child_count)))
}

#[test]
fn walker_stack_maximum_reaches_semantic_loading_and_plus_one_refuses() {
    let exact_error = load_scenario(&wide_body_form(65_536), &mut MemoryGraph::new()).unwrap_err();
    assert!(
        !exact_error.message.contains("walker stack"),
        "{exact_error}"
    );
    let error = load_scenario(&wide_body_form(65_537), &mut MemoryGraph::new()).unwrap_err();
    assert!(
        error.message.contains("65,536-entry walker stack"),
        "{error}"
    );
}

fn ast_body_form(empty_list_count: usize) -> String {
    let mut remaining = empty_list_count;
    let mut body = String::from("  (");
    for _chunk_index in 0..16 {
        let chunk = remaining.min(65_536);
        body.push('(');
        body.push_str(&"()".repeat(chunk));
        body.push(')');
        remaining -= chunk;
    }
    assert_eq!(remaining, 0);
    body.push_str(")\n");
    scenario(&body)
}

#[test]
fn ast_node_maximum_reaches_semantic_loading_and_plus_one_refuses() {
    const NON_EMPTY_LIST_AST_NODES: usize = 30;
    let exact_children = 1_048_576 - NON_EMPTY_LIST_AST_NODES;
    let exact_error =
        load_scenario(&ast_body_form(exact_children), &mut MemoryGraph::new()).unwrap_err();
    assert!(
        !exact_error.message.contains("AST exceeds"),
        "{exact_error}"
    );
    let error =
        load_scenario(&ast_body_form(exact_children + 1), &mut MemoryGraph::new()).unwrap_err();
    assert!(error.message.contains("1,048,576-node"), "{error}");
}

#[test]
fn error_type_remains_structured() {
    fn require_scenario_error(error: ScenarioError) -> ScenarioError {
        error
    }
    let error = load_scenario("", &mut MemoryGraph::new()).unwrap_err();
    assert!(require_scenario_error(error).code.is_none());
}
