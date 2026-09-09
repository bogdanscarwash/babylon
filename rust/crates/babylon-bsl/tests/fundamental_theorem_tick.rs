//! **P27 Phase 2 Slice 1: the vertical slice, end to end.**
//!
//! Scenario file → world → loaded rule → tick → deterministic state hash.
//! Every stage is the real one; nothing here is a stub.
//!
//! This exists because until it passed, the Rust engine could parse a rule
//! and hold a graph but could not *run* anything: there was no world-load
//! path and no loop to drive rules over a population. "We have a language"
//! becomes "we have an engine" at the point this file goes green.
//!
//! # The rule under test
//!
//! The Fundamental Theorem: revolution in the core is impossible while wages
//! exceed the value produced, and the gap is imperial rent (Φ). Chosen as the
//! first rule because it is the theoretical spine every other mechanic hangs
//! off, and because it has the minimum shape a real rule needs — read
//! attributes, compare, write a derived value back.
//!
//! The scenario carries two classes drawn to make the theorem's *two* cases
//! visible in one tick, which is the point of the test:
//!
//! - a **core** class with wages 120 against value produced 80 — extracting
//!   Φ = 40, the labour-aristocracy position;
//! - a **periphery** class with wages 20 against value produced 90 — the
//!   guard is false, nothing is written, and the absence of an
//!   `imperial-rent` attribute on that node is itself the assertion.
//!
//! A rule that fired on everything would pass a weaker test. The periphery
//! class is here so the guard has to actually discriminate.

use babylon_bsl::fuel::{CardinalityCeilings, IntrinsicCosts};
use babylon_bsl::intrinsic_host::EmptyIntrinsicHost;
use babylon_bsl::rule_pipeline::{load_rule, LoadContext};
use babylon_bsl::scenario::load_scenario;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_bsl::tick::{run_tick, DefinesEnv};
use babylon_bsl::typecheck::TypeEnv;
use babylon_bsl::types::{BslType, EnumRegistry, FieldDecl, FieldKind};
use babylon_bsl::BindingVocabulary;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_kernel::SessionId;
use std::collections::{HashMap, HashSet};

/// Fixed deterministic V1 namespace for these one-shot conformance drivers.
/// They contain no finite kernel, so the byte-identical state-hash theorem
/// remains independent of governed replay realization.
fn fixture_session() -> SessionId {
    SessionId::new("fundamental-theorem-tick-fixture").expect("literal is non-empty")
}

const SCENARIO: &str = r"
(scenario ft/two-classes
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

const FUNDAMENTAL_THEOREM: &str = r#"
(rule economics/fundamental-theorem
  :role mechanic :evidence derived :material-basis "core wages above the value core labour produces is imperial rent; while the gap holds, revolution in the core is materially foreclosed"
  :fuel 64
  (bindings
    (binding wages :field social-class/wages)
    (binding value-produced :field social-class/value-produced))
  (when (> wages value-produced))
  (effects
    (update-node self social-class/imperial-rent (set (- wages value-produced)))))
"#;

fn field(ty: BslType, kind: FieldKind) -> FieldDecl {
    FieldDecl { ty, kind }
}

fn types() -> TypeEnv {
    TypeEnv {
        fields: HashMap::from([
            (
                "social-class/wages".to_owned(),
                field(BslType::Int, FieldKind::Extensive),
            ),
            (
                "social-class/value-produced".to_owned(),
                field(BslType::Int, FieldKind::Extensive),
            ),
            (
                "social-class/imperial-rent".to_owned(),
                field(BslType::Int, FieldKind::Extensive),
            ),
        ]),
        exemptions: &[],
    }
}

struct Registries {
    vocabulary: BindingVocabulary,
    types: TypeEnv,
    ceilings: CardinalityCeilings,
    intrinsics: IntrinsicCosts,
    systems: HashSet<String>,
    /// This file's fixture declares no enum-typed field — an empty
    /// registry is the honest "no `defenum`s in scope" input.
    enums: EnumRegistry,
}

fn registries() -> Registries {
    let declared = types();
    Registries {
        vocabulary: BindingVocabulary {
            fields: declared.fields.keys().cloned().collect(),
            consts: HashSet::new(),
            probability_consts: HashSet::new(),
            metrics: HashSet::new(),
        },
        types: declared,
        ceilings: CardinalityCeilings::new(
            HashMap::from([("NodeType/SOCIAL_CLASS".to_owned(), 100)]),
            HashMap::new(),
        ),
        intrinsics: IntrinsicCosts::default(),
        systems: HashSet::from(["economics".to_owned()]),
        enums: EnumRegistry::default(),
    }
}

/// The whole slice: load a world, load a rule, run one tick.
fn run_one_tick() -> (MemoryGraph, usize, usize) {
    let r = registries();
    let mut graph = MemoryGraph::new();
    let loaded_scenario = load_scenario(SCENARIO, &mut graph).expect("the scenario must load");

    let ctx = LoadContext {
        vocabulary: &r.vocabulary,
        types: &r.types,
        enums: &r.enums,
        const_values: &loaded_scenario.consts,
        ceilings: &r.ceilings,
        intrinsics: &r.intrinsics,
        systems: &r.systems,
        vocabulary_registry: None,
        rule_file: "economics/fundamental-theorem.bsl",
    };
    let loaded = load_rule(FUNDAMENTAL_THEOREM, &ctx).expect("the rule must pass every load gate");

    let mut sink = CollectingSink::default();
    let outcome = run_tick(
        &loaded,
        &r.types,
        &r.enums,
        &EmptyIntrinsicHost,
        &mut graph,
        &mut sink,
        &r.intrinsics,
        &DefinesEnv::new(),
        1,
        Some(&loaded_scenario.node_content_ids),
        &fixture_session(),
        None,
    )
    .expect("the tick must run");

    (graph, outcome.considered, outcome.fired)
}

#[test]
fn the_fundamental_theorem_runs_end_to_end() {
    let (graph, considered, fired) = run_one_tick();

    assert_eq!(considered, 2, "both classes were subjects");
    assert_eq!(fired, 1, "the guard discriminated — only the core extracts");

    // The core class: Φ = 120 − 80 = 40, written back to the graph.
    let rent = graph
        .node_attribute(NodeId(0), "social-class/imperial-rent")
        .expect("the core class must carry its imperial rent");
    assert!(
        (rent - 40.0).abs() < 1e-12,
        "Φ = wages − value produced = 40, got {rent}"
    );
}

#[test]
fn the_periphery_carries_no_rent_and_that_absence_is_the_assertion() {
    // III.11: a class that extracts nothing has NO imperial-rent attribute —
    // not a zero. A stored 0.0 would be a claim ("measured, and it is none");
    // the absence is the honest state ("this class does not extract").
    let (graph, _, _) = run_one_tick();
    let err = graph
        .node_attribute(NodeId(1), "social-class/imperial-rent")
        .expect_err("the periphery class must have no rent attribute at all");
    assert!(err.message.contains("never a default"), "{}", err.message);
}

#[test]
fn the_same_content_over_the_same_world_hashes_identically() {
    // Constitution III.7. This is the slice's definition of done: run the
    // whole path twice and the state hashes agree byte for byte.
    let (first, _, _) = run_one_tick();
    let (second, _, _) = run_one_tick();
    assert_eq!(
        first.state_hash().unwrap(),
        second.state_hash().unwrap(),
        "two identical runs must produce identical state"
    );
}

#[test]
fn the_tick_actually_changed_the_world() {
    // The dual, and the one that catches a tick that silently did nothing:
    // the post-tick hash must differ from the pre-tick hash. Without this,
    // a run_tick that returned Ok and executed no effect would pass every
    // other test in this file.
    let mut before = MemoryGraph::new();
    load_scenario(SCENARIO, &mut before).unwrap();
    let untouched = before.state_hash().unwrap();

    let (after, _, fired) = run_one_tick();
    assert_eq!(fired, 1);
    assert_ne!(
        untouched,
        after.state_hash().unwrap(),
        "a tick that fired an effect must move the state hash"
    );
}

#[test]
fn a_changed_scenario_changes_the_hash() {
    // The world is part of the fingerprint, not just the rules.
    let r = registries();
    let mut richer = MemoryGraph::new();
    let loaded_scenario = load_scenario(
        r"
(scenario ft/two-classes
  (deffield social-class/wages int extensive)
  (deffield social-class/value-produced int extensive)
  (deffield social-class/imperial-rent int extensive)
  (node core NodeType/SOCIAL_CLASS
    (social-class/wages 121)
    (social-class/value-produced 80))
  (node periphery NodeType/SOCIAL_CLASS
    (social-class/wages 20)
    (social-class/value-produced 90)))
",
        &mut richer,
    )
    .unwrap();

    let ctx = LoadContext {
        vocabulary: &r.vocabulary,
        types: &r.types,
        enums: &r.enums,
        const_values: &loaded_scenario.consts,
        ceilings: &r.ceilings,
        intrinsics: &r.intrinsics,
        systems: &r.systems,
        vocabulary_registry: None,
        rule_file: "economics/fundamental-theorem.bsl",
    };
    let loaded = load_rule(FUNDAMENTAL_THEOREM, &ctx).unwrap();
    let mut sink = CollectingSink::default();
    run_tick(
        &loaded,
        &r.types,
        &r.enums,
        &EmptyIntrinsicHost,
        &mut richer,
        &mut sink,
        &r.intrinsics,
        &DefinesEnv::new(),
        1,
        Some(&loaded_scenario.node_content_ids),
        &fixture_session(),
        None,
    )
    .unwrap();

    let (baseline, _, _) = run_one_tick();
    assert_ne!(
        baseline.state_hash().unwrap(),
        richer.state_hash().unwrap(),
        "one more unit of wages is a different world and must hash differently"
    );
}

// ================================================ the `<bind-src>` estate
//
// `bind_subject` used to read `let BindSource::Field(qname) = … else
// { continue };`, so every OTHER source was silently skipped and
// `resolve_expr_bindings` had no caller at all: a rule with a `:expr`
// binding passed every load gate and then died mid-guard with a generic
// unbound-variable error. These vectors drive `run_tick` — the layer where
// the gap lived — and cover the whole `<bind-src>` set, not the one arm.

/// A world with a wealth field the `:expr` rule below reads and writes.
const EXPR_SCENARIO: &str = r"
(scenario expr/one-class
  (deffield social-class/wealth int extensive)
  (deffield social-class/agitation int intensive)
  (node core NodeType/SOCIAL_CLASS
    (social-class/wealth 900)
    (social-class/agitation 0))
  (node periphery NodeType/SOCIAL_CLASS
    (social-class/wealth 20)
    (social-class/agitation 0)))
";

/// §2.5's own worked shape: a rule that names an intermediate value and
/// then reads it in BOTH the guard and an effect.
const EXPR_RULE: &str = r#"
(rule economics/drained
  :role mechanic :evidence derived :material-basis "the gap between a class's wealth and its subsistence cost is what its reproduction must close"
  :fuel 256
  (bindings
    (binding wealth  :field social-class/wealth)
    (binding drained :expr (- wealth 100)))
  (when (< drained 0))
  (effects
    (update-node self social-class/agitation (add 1))))
"#;

fn expr_registries() -> Registries {
    let declared = TypeEnv {
        fields: HashMap::from([
            (
                "social-class/wealth".to_owned(),
                FieldDecl {
                    ty: BslType::Int,
                    kind: FieldKind::Extensive,
                },
            ),
            (
                "social-class/agitation".to_owned(),
                FieldDecl {
                    ty: BslType::Int,
                    kind: FieldKind::Intensive,
                },
            ),
        ]),
        exemptions: &[],
    };
    Registries {
        vocabulary: BindingVocabulary {
            fields: declared.fields.keys().cloned().collect(),
            consts: HashSet::from(["vitality/subsistence-cost".to_owned()]),
            probability_consts: HashSet::new(),
            metrics: HashSet::from(["solidarity-density".to_owned()]),
        },
        types: declared,
        ceilings: CardinalityCeilings::new(
            HashMap::from([("NodeType/SOCIAL_CLASS".to_owned(), 100)]),
            HashMap::new(),
        ),
        intrinsics: IntrinsicCosts::default(),
        systems: HashSet::from(["economics".to_owned()]),
        enums: EnumRegistry::default(),
    }
}

/// Load `rule` against the `:expr` world and run one tick over it.
fn run_expr_tick(rule: &str) -> Result<(MemoryGraph, usize), String> {
    let r = expr_registries();
    let mut graph = MemoryGraph::new();
    let loaded_scenario = load_scenario(EXPR_SCENARIO, &mut graph).expect("the scenario must load");
    let ctx = LoadContext {
        vocabulary: &r.vocabulary,
        types: &r.types,
        enums: &r.enums,
        const_values: &loaded_scenario.consts,
        ceilings: &r.ceilings,
        intrinsics: &r.intrinsics,
        systems: &r.systems,
        vocabulary_registry: None,
        rule_file: "economics/drained.bsl",
    };
    let loaded = load_rule(rule, &ctx).map_err(|e| format!("load: {e}"))?;
    let mut sink = CollectingSink::default();
    let outcome = run_tick(
        &loaded,
        &r.types,
        &r.enums,
        &EmptyIntrinsicHost,
        &mut graph,
        &mut sink,
        &r.intrinsics,
        &DefinesEnv::new(),
        1,
        Some(&loaded_scenario.node_content_ids),
        &fixture_session(),
        None,
    )
    .map_err(|e| format!("tick: {e}"))?;
    Ok((graph, outcome.fired))
}

/// The load-passes/execute-dies gap, closed: a `:expr` binding read by the
/// guard AND by an effect must make the rule FIRE and move state.
#[test]
fn a_expr_binding_drives_a_real_tick() {
    let before = {
        let mut g = MemoryGraph::new();
        load_scenario(EXPR_SCENARIO, &mut g).unwrap();
        g.state_hash().unwrap()
    };
    let (graph, fired) = run_expr_tick(EXPR_RULE).expect("the :expr rule must run");
    assert_eq!(
        fired, 1,
        "only `periphery` (wealth 20) has `drained < 0`; `core` (900) does not"
    );
    let after = graph.state_hash().unwrap();
    assert_ne!(
        before, after,
        "a tick that fired must move state — a rule that returns Ok and \
         executes nothing would pass a fired-count assertion alone"
    );
    // The effect landed on the subject the guard selected, and nowhere else.
    let subjects = graph.nodes("SOCIAL_CLASS");
    let agitations: Vec<f64> = subjects
        .iter()
        .map(|id| graph.node_attribute(*id, "social-class/agitation").unwrap())
        .collect();
    assert_eq!(agitations, vec![0.0, 1.0]);
}

/// Determinism (§III.7): the same content over the same world twice.
#[test]
fn a_expr_driven_tick_is_deterministic() {
    let (a, fired_a) = run_expr_tick(EXPR_RULE).unwrap();
    let (b, fired_b) = run_expr_tick(EXPR_RULE).unwrap();
    assert_eq!(fired_a, fired_b);
    assert_eq!(a.state_hash().unwrap(), b.state_hash().unwrap());
}

/// §2.5's calendar seam, served rather than refused: the driver knows its
/// own tick number, so `:tick` and `:tick-in-cycle` are exact.
#[test]
fn the_servable_calendar_sources_run() {
    let rule = r#"
(rule economics/clocked
  :role mechanic :evidence derived :material-basis "a reproduction cycle is counted in ticks, and the tick is the kernel's"
  :fuel 256
  (bindings
    (binding wealth :field social-class/wealth)
    (binding now    :tick)
    (binding phase  :tick-in-cycle 4))
  (when (and (= now 1) (= phase 1)))
  (effects
    (update-node self social-class/agitation (add 1))))
"#;
    let (graph, fired) = run_expr_tick(rule).expect("the calendar rule must run");
    assert_eq!(
        fired, 2,
        "tick 1 is phase 1 in a 4-cycle, for every subject"
    );
    // Exact equality is the right comparison here: the field is declared
    // `int` and `add 1` is the §4.3 basic operation, so the stored value is
    // exactly representable and reproduces bit-exactly (§6.1: conformance
    // is not tolerance-bounded).
    let agitations: Vec<f64> = graph
        .nodes("SOCIAL_CLASS")
        .iter()
        .map(|id| graph.node_attribute(*id, "social-class/agitation").unwrap())
        .collect();
    assert_eq!(agitations, vec![1.0, 1.0]);
}

/// …and the sources slice 1 cannot honestly serve are refused **by name
/// and at entry**, never left to surface as an unbound variable mid-guard.
#[test]
fn the_unservable_sources_are_refused_loudly_by_name() {
    let cases = [
        ("(binding c :const vitality/subsistence-cost)", ":const"),
        ("(binding m :metric solidarity-density)", ":metric"),
        ("(binding y :year)", ":year"),
        ("(binding t :tick-of-year)", ":tick-of-year"),
    ];
    for (decl, source) in cases {
        let rule = format!(
            r#"(rule economics/unservable
  :role mechanic :evidence derived :material-basis "the wage relation"
  :fuel 256
  (bindings (binding wealth :field social-class/wealth) {decl})
  (when (< wealth 1000))
  (effects (update-node self social-class/agitation (add 1))))"#
        );
        let err = run_expr_tick(&rule).expect_err(source);
        assert!(
            err.starts_with("tick: "),
            "{source}: refused at load, not at the tick layer: {err}"
        );
        assert!(
            err.contains(source) && err.contains("not servable in slice 1"),
            "{source}: the refusal must name the source and the reason, got: {err}"
        );
        assert!(
            !err.contains("unbound variable"),
            "{source}: refused as an accidental unbound variable: {err}"
        );
    }
}
