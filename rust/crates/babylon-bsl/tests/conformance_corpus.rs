//! The transcribed conformance corpus (spec §5, §8.1): the 271-line
//! doctrine trap-DSL corpus (`tests/unit/domain/doctrine/test_mechanics.py`)
//! and the 628-line event-evaluator corpus
//! (`tests/unit/engine/test_event_evaluator.py`), ported with the
//! documented 4-point M8 delta at exactly the sites spec §5
//! "Grammar-superset honesty" names — each correction test cites the
//! Python line it replaces and the old vs. new behavior. The full
//! per-function disposition table is
//! `reports/p27-conformance-corpus-transcription.md`.
//!
//! **PR 4, Task 14 (2026-08-11): the Phase-2 scope note retires for the
//! node-set-shaped estate.** The former note read: "fold/query EXECUTION
//! needs the Phase-2 query evaluator, so aggregation vectors pin load-time
//! verdicts (parse, resolve, §3.4 typecheck, §3.7 bound) here and their
//! runtime values ride the Phase-2 vector re-run." That evaluator has now
//! landed (the BSL query-evaluation plan's slice 1). Every aggregation
//! vector whose query is `(nodes …)` executes for real below —
//! `event_node_condition.bsl` (`exists`), `event_forall.bsl` (`forall`)
//! and `event_wealth_aggregates.bsl` (`sum`/`max`/`min`/weighted `mean`) —
//! against a real `MemoryGraph`, asserting the RAISED/returned value and a
//! `:fuel-used` figure (§6.1), not merely that the fixture loads and
//! bounds. `event_edge_count.bsl` ALSO executes for real now (T2, issue
//! #559, slice 2's dyadic edge lane) —
//! `edge_count_evaluates_for_real_on_an_empty_graph`/
//! `..._on_a_non_empty_graph` below, promoted from the load-only pin
//! `edge_count_stays_pinned_and_names_slice_2` used to be.
//! `event_bifurcation.bsl` and `event_metric_conditions.bsl` needed
//! no query at all and already executed for real before this task (see
//! `bifurcation_routes_by_solidarity_density` and
//! `metric_conditions_load_and_evaluate` below) — unchanged.
#![allow(clippy::doc_markdown)] // doc comments cite Python test names and file paths verbatim

use babylon_bsl::evaluator::{evaluate, EvalEnv, Value};
use babylon_bsl::fuel::{CardinalityCeilings, IntrinsicCosts};
use babylon_bsl::intrinsic_host::EmptyIntrinsicHost;
use babylon_bsl::reader::read;
use babylon_bsl::rule_pipeline::{bind_environment, load_rule, LoadContext, LoadError, LoadedRule};
use babylon_bsl::structural_verbs::{CollectingSink, EffectExecutor};
use babylon_bsl::typecheck::TypeEnv;
use babylon_bsl::types::{BslType, EnumRegistry, FieldDecl, FieldKind};
use babylon_bsl::BindingVocabulary;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::GraphSubstrate;
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------- context

fn field(ty: BslType, kind: FieldKind) -> FieldDecl {
    FieldDecl { ty, kind }
}

fn types() -> TypeEnv {
    TypeEnv {
        fields: HashMap::from([
            // Doctrine tag totals: accumulated counts, extensive.
            (
                "organization/mass-link".to_owned(),
                field(BslType::Int, FieldKind::Extensive),
            ),
            (
                "organization/class-analysis".to_owned(),
                field(BslType::Int, FieldKind::Extensive),
            ),
            (
                "organization/militancy".to_owned(),
                field(BslType::Int, FieldKind::Extensive),
            ),
            // Measured-practice variables: unit-interval shares, intensive.
            (
                "organization/solidarity-mass".to_owned(),
                field(BslType::Coefficient, FieldKind::Intensive),
            ),
            (
                "organization/co-optive-share".to_owned(),
                field(BslType::Coefficient, FieldKind::Intensive),
            ),
            (
                "organization/petty-bourgeois-drift".to_owned(),
                field(BslType::Coefficient, FieldKind::Intensive),
            ),
            // Event-evaluator estate.
            (
                "social-class/agitation".to_owned(),
                field(BslType::Intensity, FieldKind::Intensive),
            ),
            (
                "social-class/national-identity".to_owned(),
                field(BslType::Intensity, FieldKind::Intensive),
            ),
            (
                "social-class/class-consciousness".to_owned(),
                field(BslType::Intensity, FieldKind::Intensive),
            ),
            (
                "social-class/wealth".to_owned(),
                field(BslType::Currency, FieldKind::Extensive),
            ),
            (
                "social-class/population".to_owned(),
                field(BslType::Int, FieldKind::Extensive),
            ),
            (
                "solidarity/strength".to_owned(),
                field(BslType::Intensity, FieldKind::Intensive),
            ),
        ]),
        exemptions: &[],
    }
}

fn vocabulary() -> BindingVocabulary {
    let types = types();
    BindingVocabulary {
        fields: types.fields.keys().cloned().collect(),
        consts: HashSet::from([
            "doctrine/solidarity-liquidation-floor".to_owned(),
            "doctrine/co-optive-liquidation-threshold".to_owned(),
            "doctrine/petty-bourgeois-liquidation-threshold".to_owned(),
        ]),
        probability_consts: HashSet::new(),
        metrics: HashSet::from([
            // The six Python metrics (event_evaluator.py:301-310), as the
            // registered metric set.
            "solidarity-density".to_owned(),
            "exploitation-density".to_owned(),
            "average-agitation".to_owned(),
            "average-consciousness".to_owned(),
            "total-wealth".to_owned(),
            "gini-coefficient".to_owned(),
        ]),
    }
}

fn ceilings() -> CardinalityCeilings {
    CardinalityCeilings::new(
        HashMap::from([
            ("NodeType/SOCIAL_CLASS".to_owned(), 100),
            ("EdgeType/SOLIDARITY".to_owned(), 40),
        ]),
        HashMap::new(),
    )
}

struct Registries {
    vocabulary: BindingVocabulary,
    types: TypeEnv,
    ceilings: CardinalityCeilings,
    intrinsics: IntrinsicCosts,
    systems: HashSet<String>,
    /// No fixture in this corpus declares an enum-typed field — an empty
    /// registry is the honest "no `defenum`s in scope" input.
    enums: EnumRegistry,
}

fn registries() -> Registries {
    Registries {
        vocabulary: vocabulary(),
        types: types(),
        ceilings: ceilings(),
        intrinsics: IntrinsicCosts::default(),
        systems: HashSet::from(["doctrine".to_owned(), "event".to_owned()]),
        enums: EnumRegistry::default(),
    }
}

fn load(source: &str, rule_file: &str) -> Result<LoadedRule, LoadError> {
    let r = registries();
    let ctx = LoadContext {
        vocabulary: &r.vocabulary,
        types: &r.types,
        enums: &r.enums,
        const_values: Box::leak(Box::new(HashMap::new())),
        ceilings: &r.ceilings,
        intrinsics: &r.intrinsics,
        systems: &r.systems,
        vocabulary_registry: None,
        rule_file,
    };
    load_rule(source, &ctx)
}

/// The `<when>` clause of a loaded rule — shared by `eval_when` and its
/// graph-bearing twin `eval_when_over_graph` (PR 4, Task 14), plus the
/// slice-2-refusal vector for `event_edge_count.bsl`.
fn when_clause(rule: &LoadedRule) -> &babylon_bsl::SExpr {
    let babylon_bsl::SExpr::List(items) = &rule.rule else {
        unreachable!()
    };
    items
        .iter()
        .find_map(|child| match child {
            babylon_bsl::SExpr::List(inner)
                if matches!(inner.first(), Some(babylon_bsl::SExpr::Atom(babylon_bsl::Atom::Symbol(h))) if h == "when") =>
            {
                inner.get(1)
            }
            _ => None,
        })
        .expect("fixture has a when clause")
}

/// Load a fixture and evaluate its `<when>` condition against supplied
/// binding values — the trap-DSL calling convention
/// (`evaluate_trap_condition(expr, env, coeffs)`) reconstructed from the
/// composed pipeline.
fn eval_when(rule: &LoadedRule, supplied: &HashMap<String, Value>) -> bool {
    let env_map = bind_environment(&rule.bindings, supplied).expect("environment must bind");
    let costs = IntrinsicCosts::default();
    let env = EvalEnv {
        bindings: env_map,
        intrinsic_costs: &costs,
        graph: None,
        types: None,
        enums: None,
        elements: Vec::new(),
        draw_context: None,
    };
    let mut fuel = 10_000;
    match evaluate(when_clause(rule), &env, &EmptyIntrinsicHost, &mut fuel)
        .expect("condition must evaluate")
    {
        Value::Bool(b) => b,
        other => panic!("a <cond> must be Bool, got {other:?}"),
    }
}

/// `eval_when`'s graph-bearing twin (PR 4, Task 14): the same calling
/// convention, but `env.graph = Some(graph)` — for fixtures whose
/// `<when>` materializes a `<query>` (§2.6) rather than reading only
/// rule-scope bindings. Takes the fuel meter BY REFERENCE, unlike
/// `eval_when`, so a caller can assert `:fuel-used` (§6.1) on the vector
/// that matters.
fn eval_when_over_graph(
    rule: &LoadedRule,
    supplied: &HashMap<String, Value>,
    graph: &dyn GraphSubstrate,
    fuel: &mut u64,
) -> bool {
    let env_map = bind_environment(&rule.bindings, supplied).expect("environment must bind");
    let costs = IntrinsicCosts::default();
    let env = EvalEnv {
        bindings: env_map,
        intrinsic_costs: &costs,
        graph: Some(graph),
        types: None,
        enums: None,
        elements: Vec::new(),
        draw_context: None,
    };
    match evaluate(when_clause(rule), &env, &EmptyIntrinsicHost, fuel)
        .expect("condition must evaluate")
    {
        Value::Bool(b) => b,
        other => panic!("a <cond> must be Bool, got {other:?}"),
    }
}

fn int(n: i64) -> Value {
    Value::Int(n)
}
fn real(r: f64) -> Value {
    Value::Real(r)
}

// ================================================================ doctrine
// tests/unit/domain/doctrine/test_mechanics.py (271 lines)

const ADVENTURISM: &str = include_str!("conformance/doctrine_adventurism.bsl");
const LIQUIDATIONISM: &str = include_str!("conformance/doctrine_liquidationism.bsl");
const ABSORBING: &str = include_str!("conformance/doctrine_liquidation_absorbing.bsl");

/// test_mechanics.py:45-64 (TestRealMvpConditions) — the two shipped MVP
/// trap conditions, behavior preserved exactly.
#[test]
fn real_mvp_conditions() {
    let adventurism = load(ADVENTURISM, "tests/conformance/doctrine_adventurism.bsl").unwrap();
    assert!(eval_when(&adventurism, &owned(vec![("mass-link", int(0))])));
    assert!(!eval_when(
        &adventurism,
        &owned(vec![("mass-link", int(3))])
    ));

    let liquidationism = load(
        LIQUIDATIONISM,
        "tests/conformance/doctrine_liquidationism.bsl",
    )
    .unwrap();
    assert!(eval_when(
        &liquidationism,
        &owned(vec![("class-analysis", int(0)), ("militancy", int(0))])
    ));
    assert!(!eval_when(
        &liquidationism,
        &owned(vec![("class-analysis", int(0)), ("militancy", int(5))])
    ));
}

/// test_mechanics.py:67-75 (TestMissingTagIsZero) — absent = no
/// accumulated strength. In BSL the honest-null site is DECLARED:
/// `:optional :default 0`, carried on the DEFAULT_ALLOWLIST.
#[test]
fn missing_tag_reads_the_declared_default() {
    let adventurism = load(ADVENTURISM, "tests/conformance/doctrine_adventurism.bsl").unwrap();
    // No value supplied at all: the declared default 0 applies -> fires.
    assert!(eval_when(&adventurism, &HashMap::new()));
    // The allowlist covers every :default this corpus declares — zero
    // sign-off findings.
    assert!(adventurism.default_findings.is_empty());
    let absorbing = load(
        ABSORBING,
        "tests/conformance/doctrine_liquidation_absorbing.bsl",
    )
    .unwrap();
    assert!(absorbing.default_findings.is_empty());
}

/// test_mechanics.py:85-134 (TestMeasuredPracticeVocabulary) — the U11
/// absorbing state over practice variables + coefficient thresholds.
#[test]
fn liquidation_absorbing_state() {
    let absorbing = load(
        ABSORBING,
        "tests/conformance/doctrine_liquidation_absorbing.bsl",
    )
    .unwrap();
    let coeffs = [
        ("solidarity-liquidation-floor", real(0.05)),
        ("co-optive-liquidation-threshold", real(0.6)),
        ("petty-bourgeois-liquidation-threshold", real(0.6)),
    ];
    // test_mechanics.py:114-117 — collapsed solidarity, high co-optation,
    // embourgeoised base: liquidated.
    let mut fires: Vec<(&str, Value)> = coeffs.to_vec();
    fires.extend([
        ("solidarity-mass", real(0.0)),
        ("co-optive-share", real(0.8)),
        ("petty-bourgeois-drift", real(0.7)),
    ]);
    assert!(eval_when(&absorbing, &owned(fires)));
    // test_mechanics.py:119-122 — a live SOLIDARITY mass base defeats the
    // absorbing state, whatever else.
    let mut dormant: Vec<(&str, Value)> = coeffs.to_vec();
    dormant.extend([
        ("solidarity-mass", real(0.5)),
        ("co-optive-share", real(0.8)),
        ("petty-bourgeois-drift", real(0.7)),
    ]);
    assert!(!eval_when(&absorbing, &owned(dormant)));
}

/// test_mechanics.py:124-138 — an unknown `@coeff` / unknown variable
/// fails LOUD. In BSL both are load-time resolution failures, not
/// evaluation-time surprises.
#[test]
fn unknown_coefficient_and_variable_fail_loud_at_load() {
    // Unknown coefficient: a :const outside the defines vocabulary.
    let unknown_coeff = ABSORBING.replace(
        ":const doctrine/solidarity-liquidation-floor",
        ":const doctrine/no-such-threshold",
    );
    let err = load(&unknown_coeff, "x.bsl").unwrap_err();
    assert_eq!(err.spec_code(), Some("E-LOAD-010"), "{err}");

    // Unknown variable: a free symbol no binding declares.
    let unknown_var = ADVENTURISM.replace("(<= mass-link 0)", "(<= not-a-variable 0)");
    let err = load(&unknown_var, "x.bsl").unwrap_err();
    assert_eq!(err.spec_code(), Some("E-LOAD-010"), "{err}");
}

/// test_mechanics.py:147-181 (TestFullGrammar) — all six comparisons,
/// AND/OR/NOT and grouping. Python's infix precedence ("AND binds tighter
/// than OR") is STRUCTURAL in BSL — s-expressions have no precedence to
/// get wrong, so the precedence test transcribes as explicit nesting.
#[test]
fn full_condition_grammar() {
    type Vector<'a> = (&'a str, &'a [(&'a str, Value)], bool);
    let vectors: &[Vector<'_>] = &[
        ("(>= ca 3)", &[("ca", Value::Int(3))], true),
        ("(> ca 3)", &[("ca", Value::Int(3))], false),
        ("(< ml 2)", &[("ml", Value::Int(1))], true),
        ("(= mi 4)", &[("mi", Value::Int(4))], true),
        ("(!= mi 4)", &[("mi", Value::Int(4))], false),
        (
            "(or (<= ml 0) (<= mi 0))",
            &[("ml", Value::Int(5)), ("mi", Value::Int(0))],
            true,
        ),
        (
            "(or (<= ml 0) (<= mi 0))",
            &[("ml", Value::Int(5)), ("mi", Value::Int(5))],
            false,
        ),
        ("(not (<= ml 0))", &[("ml", Value::Int(3))], true),
        ("(not (<= ml 0))", &[("ml", Value::Int(0))], false),
        (
            "(and (or (<= ca 0) (<= mi 0)) (<= ml 0))",
            &[
                ("ca", Value::Int(0)),
                ("mi", Value::Int(5)),
                ("ml", Value::Int(0)),
            ],
            true,
        ),
        (
            "(and (or (<= ca 0) (<= mi 0)) (<= ml 0))",
            &[
                ("ca", Value::Int(5)),
                ("mi", Value::Int(5)),
                ("ml", Value::Int(0)),
            ],
            false,
        ),
        // test_mechanics.py:177-181 — "A OR (B AND C)", nesting explicit.
        (
            "(or (<= ca 0) (and (<= ml 0) (<= mi 0)))",
            &[
                ("ca", Value::Int(0)),
                ("ml", Value::Int(0)),
                ("mi", Value::Int(5)),
            ],
            true,
        ),
    ];
    for (source, env_pairs, expected) in vectors {
        assert_eq!(eval_cond(source, env_pairs), *expected, "vector: {source}");
    }
}

/// test_mechanics.py:184-202 (TestMalformedRaises) — every malformed
/// condition fails LOUDLY, each through the §4.6 class its defect belongs
/// to; a trap that silently never fires is a correctness hole.
#[test]
fn malformed_conditions_fail_loud() {
    // "" — nothing to read.
    assert!(read("").is_err());
    // "MASS_LINK <=" — dangling operator: strictly binary arity.
    let err = eval_cond_err("(<= ml)", &[("ml", Value::Int(0))]);
    assert!(err.message.contains("E-PARSE-040"), "{err}");
    // "MASS_LINK 0" — missing comparison: the head is an undeclared call.
    let err = eval_cond_err("(ml 0)", &[("ml", Value::Int(0))]);
    assert!(err.message.contains("E-LOAD-021"), "{err}");
    // "UNKNOWN_TAG <= 0" — SCREAMING_SNAKE now lexes fine (#528 fix round:
    // a bare uppercase-with-underscore run is a legal <enum-member>-shaped
    // atom, §2.13's own EBNF for a defenum/defvocabulary member list), but
    // it is not a value in comparison/expression position — the refusal
    // moves from the reader (E-LEX-003) to evaluation.
    let err = eval_cond_err("(<= UNKNOWN_TAG 0)", &[]);
    assert!(
        err.message.contains("not a value in expression position"),
        "{err}"
    );
    // "MASS_LINK <= zero" — a non-literal RHS is a free variable: load error.
    let with_free = ADVENTURISM.replace("(<= mass-link 0)", "(<= mass-link zero)");
    assert_eq!(
        load(&with_free, "x.bsl").unwrap_err().spec_code(),
        Some("E-LOAD-010")
    );
    // "MASS_LINK <= 0 AND" / "(MASS_LINK <= 0" — unterminated forms.
    assert!(read("(and (<= ml 0)").is_err());
}

/// test_mechanics.py:249-258 (TestTheoreticalLabourAccrual) — surplus ×
/// allocation with the negative floor and the unit-interval clamp WRITTEN
/// AS CONTENT (an explicit `if` is honest math; only SILENT clamping is
/// banned, §3.3).
#[test]
fn theoretical_labor_accrual_vectors() {
    let accrual = "(if (< surplus 0) \
                     0 \
                     (* surplus (if (> alloc 1.0c) 1.0c (if (< alloc 0c) 0c alloc))))";
    let cases: &[(f64, f64, f64)] = &[
        (1000.0, 0.2, 200.0),
        (-500.0, 0.2, 0.0),
        (100.0, 1.5, 100.0),
        (100.0, -0.5, 0.0),
    ];
    for (surplus, alloc, expected) in cases {
        let value = eval_value(
            accrual,
            &[("surplus", real(*surplus)), ("alloc", real(*alloc))],
        );
        match value {
            Value::Real(r) => assert!(
                (r - expected).abs() < 1e-9,
                "accrue({surplus}, {alloc}) = {r}, expected {expected}"
            ),
            Value::Int(n) => {
                // The zero arms return the Int literal 0.
                #[allow(clippy::cast_precision_loss)]
                let as_real = n as f64;
                assert!((as_real - expected).abs() < 1e-9);
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}

/// test_mechanics.py:261-271 (TestTagDecay) — multiplicative decay as an
/// expression: identical IEEE-754 arithmetic on both sides, so the
/// expected value is computed by the SAME formula, not an approx literal.
#[test]
fn tag_decay_vectors() {
    let decay = "(* total (- 1 rate))";
    for (total, rate) in [(100.0_f64, 0.0055_f64), (50.0, 0.0055), (7.0, 0.0)] {
        let value = eval_value(decay, &[("total", real(total)), ("rate", real(rate))]);
        assert_eq!(
            value,
            Value::Real(total * (1.0 - rate)),
            "decay({total}, {rate})"
        );
    }
}

// ============================================================== event corpus
// tests/unit/engine/test_event_evaluator.py (628 lines)

const BIFURCATION: &str = include_str!("conformance/event_bifurcation.bsl");
const NODE_CONDITION: &str = include_str!("conformance/event_node_condition.bsl");
const FORALL: &str = include_str!("conformance/event_forall.bsl");
const EDGE_COUNT: &str = include_str!("conformance/event_edge_count.bsl");
const WEALTH_AGGREGATES: &str = include_str!("conformance/event_wealth_aggregates.bsl");
const METRIC_CONDITIONS: &str = include_str!("conformance/event_metric_conditions.bsl");
const UNKNOWN_METRIC: &str = include_str!("conformance/unknown_metric.bsl");
const EMPTY_WHEN: &str = include_str!("conformance/empty_when.bsl");
const UNCONDITIONAL: &str = include_str!("conformance/unconditional.bsl");

/// test_event_evaluator.py:124-151 (TestCompare) — the six comparison
/// operators, exact vectors.
#[test]
fn compare_vectors() {
    let vectors: &[(&str, bool)] = &[
        ("(>= 5 5)", true),
        ("(>= 6 5)", true),
        ("(>= 4 5)", false),
        ("(<= 5 5)", true),
        ("(<= 4 5)", true),
        ("(<= 6 5)", false),
        ("(> 6 5)", true),
        ("(> 5 5)", false),
        ("(< 4 5)", true),
        ("(< 5 5)", false),
        ("(= 5 5)", true),
        ("(!= 5 5)", false),
    ];
    for (source, expected) in vectors {
        assert_eq!(eval_cond(source, &[]), *expected, "vector: {source}");
    }
    // 5.1 == 5.0 / != — the binary64 lane.
    assert!(!eval_cond("(= x 5)", &[("x", real(5.1))]));
    assert!(eval_cond("(!= x 5)", &[("x", real(5.1))]));
}

/// test_event_evaluator.py:154-179 (TestGetNestedValue) — dotted paths
/// become qname field paths; the missing-key-returns-None behavior does
/// NOT transcribe: absence is a load decision (`E-LOAD-010` or a declared
/// `:optional :default`), never a silently skipped None (§3.5).
#[test]
fn nested_paths_are_qnames_and_absence_is_declared() {
    // A declared field resolves...
    assert!(load(NODE_CONDITION, "x.bsl").is_ok());
    // ...an undeclared one is E-LOAD-010 at load, never a skipped None.
    let missing = NODE_CONDITION.replace("social-class/agitation", "social-class/missing");
    assert_eq!(
        load(&missing, "x.bsl").unwrap_err().spec_code(),
        Some("E-LOAD-010")
    );
    // A required binding with no supplied value is loud at bind time.
    let loaded = load(NODE_CONDITION, "x.bsl").unwrap();
    assert!(bind_environment(&loaded.bindings, &HashMap::new()).is_err());
}

/// test_event_evaluator.py:182-214 + 244-323 — aggregations become folds:
/// any -> exists, all -> forall, count/sum/max/min/weighted-mean load and
/// bound against declared ceilings. The load verdict pins here for all
/// four; three of the four ALSO execute for real below
/// (`node_condition_exists_executes_over_a_real_graph`,
/// `forall_executes_over_a_real_graph`,
/// `wealth_aggregates_execute_over_a_real_graph`) — `EDGE_COUNT`'s LOAD
/// verdict is pinned here too, but it ALSO executes for real now (T2,
/// issue #559) — see `edge_count_evaluates_for_real_on_an_empty_graph`/
/// `..._on_a_non_empty_graph` below.
#[test]
fn aggregation_fixtures_load_and_bound() {
    for (fixture, name) in [
        (NODE_CONDITION, "exists/any"),
        (FORALL, "forall/all"),
        (EDGE_COUNT, "count"),
        (WEALTH_AGGREGATES, "sum/max/min/weighted-mean"),
    ] {
        let loaded = load(fixture, "x.bsl").unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(loaded.static_bound > 0, "{name} has a real bound");
    }
}

/// **PR 4, Task 14: EXECUTES.** `event_node_condition.bsl`'s `<when>` —
/// `(exists (nodes NodeType/SOCIAL_CLASS) (>= agitation 0.6p))` —
/// materializes `nodes` and runs `exists` over it, both served by slice 1.
/// The predicate reads the RULE-SCOPE `agitation` binding (not a
/// per-element field, exactly as the fixture is written), so a non-empty
/// population's verdict is gated by `self`'s own agitation, and an EMPTY
/// population is `#f` regardless of it — §4.4's exists-over-empty-is-false.
#[test]
fn node_condition_exists_executes_over_a_real_graph() {
    let loaded = load(NODE_CONDITION, "x.bsl").unwrap();
    let mut populated = MemoryGraph::new();
    populated.add_node("SOCIAL_CLASS").unwrap();

    let mut fuel = 10_000;
    assert!(eval_when_over_graph(
        &loaded,
        &owned(vec![("agitation", real(0.7))]),
        &populated,
        &mut fuel,
    ));
    assert_eq!(
        10_000 - fuel,
        5,
        ":fuel-used is a conformance-vector quantity (§6.1)"
    );

    let mut under = 10_000;
    assert!(!eval_when_over_graph(
        &loaded,
        &owned(vec![("agitation", real(0.3))]),
        &populated,
        &mut under,
    ));

    let empty = MemoryGraph::new();
    let mut on_empty = 10_000;
    assert!(!eval_when_over_graph(
        &loaded,
        &owned(vec![("agitation", real(0.9))]),
        &empty,
        &mut on_empty,
    ));
}

/// **PR 4, Task 14: EXECUTES.** `event_forall.bsl`'s `<when>` — `(forall
/// (nodes NodeType/SOCIAL_CLASS) (>= agitation 0.5p))` — materializes
/// `nodes` and runs `forall` over it. §4.4's forall-over-empty-is-TRUE is
/// the dual of `exists`' empty case above: an EMPTY population fires
/// regardless of agitation, because the condition holds VACUOUSLY.
#[test]
fn forall_executes_over_a_real_graph() {
    let loaded = load(FORALL, "x.bsl").unwrap();
    let mut populated = MemoryGraph::new();
    populated.add_node("SOCIAL_CLASS").unwrap();

    let mut fuel = 10_000;
    assert!(eval_when_over_graph(
        &loaded,
        &owned(vec![("agitation", real(0.6))]),
        &populated,
        &mut fuel,
    ));
    assert_eq!(
        10_000 - fuel,
        5,
        ":fuel-used is a conformance-vector quantity (§6.1)"
    );

    let mut under = 10_000;
    assert!(!eval_when_over_graph(
        &loaded,
        &owned(vec![("agitation", real(0.2))]),
        &populated,
        &mut under,
    ));

    let empty = MemoryGraph::new();
    let mut on_empty = 10_000;
    assert!(eval_when_over_graph(
        &loaded,
        &owned(vec![("agitation", real(0.0))]),
        &empty,
        &mut on_empty,
    ));
}

/// **PR 4, Task 14: EXECUTES.** `event_wealth_aggregates.bsl`'s `<when>`
/// folds `sum`/`max`/`min` over `wealth` and a weighted `mean` over
/// `agitation`/`population` — all four over `(nodes NodeType/SOCIAL_CLASS)`,
/// served by slice 1. Every body is a RULE-SCOPE binding — constant across
/// the fold's elements, exactly as the fixture is written (it is a
/// transcription of an aggregation SHAPE, not a per-node reader) — so with
/// N nodes: `sum = N × wealth`, `max = min = wealth`, and a weighted
/// `mean` of a constant is that constant regardless of the weight.
///
/// **A real finding this executed vector surfaces that the load-only pin
/// could not, recorded rather than silently patched:** the fixture's
/// `wealth` binding is declared `Currency` (this file's own `types()`),
/// but its three threshold literals — `550`, `500`, `50` — are bare `Int`
/// atoms (no `$` suffix). §3.1 orders within ONE numeric lane only, so
/// `(>= <Currency> 550)` is a loud, UNCODED lane-mismatch failure at
/// EVALUATION, for every possible `wealth` value — not `#f`, and not the
/// `E-TYPE-030` arithmetic-mix code (that family covers `+`/`-`/`*`/`/`,
/// not `<`/`<=`/`>`/`>=`). This is exactly the class of defect Task 14's
/// re-run exists to catch: invisible to the Phase-1 load-only pin (parse,
/// resolve, §3.4 typecheck and §3.7 bound all accept the fixture as
/// written — `aggregation_fixtures_load_and_bound` above still passes),
/// visible the moment real evaluation runs. Repairing the fixture itself
/// (`550` → `550$` etc.) is outside this task's file list — conformance
/// corpus DATA, not `conformance_corpus.rs` — so it is filed here as a
/// finding, not fixed blind. Each fold is verified on its own below,
/// against its own exact value, and the compound clause's REAL raised
/// error closes the vector.
#[test]
fn wealth_aggregates_execute_over_a_real_graph() {
    let loaded = load(WEALTH_AGGREGATES, "x.bsl").unwrap();
    let mut graph = MemoryGraph::new();
    graph.add_node("SOCIAL_CLASS").unwrap();
    graph.add_node("SOCIAL_CLASS").unwrap();

    let bindings = owned(vec![
        (
            "wealth",
            Value::Currency(babylon_kernel::Currency::from_micro_units(300_000_000)),
        ),
        ("agitation", real(0.4)),
        ("population", int(10)),
    ]);

    // sum = 2 x 300 = 600.
    let mut fuel = 10_000;
    assert_eq!(
        eval_value_over_graph(
            "(fold sum (nodes NodeType/SOCIAL_CLASS) wealth)",
            &bindings,
            &graph,
            &mut fuel,
        ),
        Value::Currency(babylon_kernel::Currency::from_micro_units(600_000_000)),
    );
    assert_eq!(
        10_000 - fuel,
        5,
        ":fuel-used is a conformance-vector quantity (§6.1)"
    );

    // max = min = 300 — the constant-body finding stated in the doc above.
    let mut fuel2 = 10_000;
    assert_eq!(
        eval_value_over_graph(
            "(fold max (nodes NodeType/SOCIAL_CLASS) wealth)",
            &bindings,
            &graph,
            &mut fuel2,
        ),
        Value::Currency(babylon_kernel::Currency::from_micro_units(300_000_000)),
    );
    let mut fuel3 = 10_000;
    assert_eq!(
        eval_value_over_graph(
            "(fold min (nodes NodeType/SOCIAL_CLASS) wealth)",
            &bindings,
            &graph,
            &mut fuel3,
        ),
        Value::Currency(babylon_kernel::Currency::from_micro_units(300_000_000)),
    );

    // The weighted mean of a constant is that constant, whatever N and
    // whatever the (also-constant) weight — population cancels out of
    // Σ(w·x)/Σw exactly.
    let mut fuel4 = 10_000;
    assert_eq!(
        eval_value_over_graph(
            "(fold mean (nodes NodeType/SOCIAL_CLASS) agitation :weight population)",
            &bindings,
            &graph,
            &mut fuel4,
        ),
        Value::Real(0.4),
    );

    // The compound `<when>` raises a loud, UNCODED lane-mismatch error —
    // the currency-vs-bare-Int defect stated in the doc above — built
    // directly (not through `eval_when_over_graph`, which `.expect()`s
    // success) so the failure is asserted, not panicked past.
    let costs = IntrinsicCosts::default();
    let env_map = bind_environment(&loaded.bindings, &bindings).unwrap();
    let env = EvalEnv {
        bindings: env_map,
        intrinsic_costs: &costs,
        graph: Some(&graph as &dyn GraphSubstrate),
        types: None,
        enums: None,
        elements: Vec::new(),
        draw_context: None,
    };
    let mut fuel5 = 10_000;
    let err = evaluate(when_clause(&loaded), &env, &EmptyIntrinsicHost, &mut fuel5).unwrap_err();
    assert_eq!(err.code, None, "{err}");
    assert!(
        err.message.contains("one numeric lane"),
        "expected the §3.1 lane-mismatch refusal, got: {err}"
    );
}

/// **T2 (issue #559, Task 3): event_edge_count.bsl now evaluates for real**, promoted from the
/// load-only pin `edge_count_stays_pinned_and_names_slice_2` used to be — `edges` is served as
/// of T2. An empty graph has zero SOLIDARITY edges: `(fold count (edges EdgeType/SOLIDARITY)
/// it)` is `0`, and `(>= 0 1)` is `false` — the exact vector T2 exists to unblock.
#[test]
fn edge_count_evaluates_for_real_on_an_empty_graph() {
    let loaded = load(EDGE_COUNT, "x.bsl").unwrap();
    let graph = MemoryGraph::new();
    let costs = IntrinsicCosts::default();
    let env_map = bind_environment(&loaded.bindings, &HashMap::new())
        .expect("EDGE_COUNT's bindings are empty — nothing to resolve");
    let env = EvalEnv {
        bindings: env_map,
        intrinsic_costs: &costs,
        graph: Some(&graph as &dyn GraphSubstrate),
        types: None,
        enums: None,
        elements: Vec::new(),
        draw_context: None,
    };
    let mut fuel = 10_000;
    let result = evaluate(when_clause(&loaded), &env, &EmptyIntrinsicHost, &mut fuel).unwrap();
    assert_eq!(
        result,
        Value::Bool(false),
        "an empty graph has zero SOLIDARITY edges"
    );
}

/// The non-empty companion: one SOLIDARITY edge is enough to cross the `>= 1` threshold.
#[test]
fn edge_count_evaluates_for_real_on_a_non_empty_graph() {
    let loaded = load(EDGE_COUNT, "x.bsl").unwrap();
    let mut graph = MemoryGraph::new();
    let a = graph.add_node("SOCIAL_CLASS").unwrap();
    let b = graph.add_node("SOCIAL_CLASS").unwrap();
    graph.add_edge("SOLIDARITY", a, b, 0.5).unwrap();
    let costs = IntrinsicCosts::default();
    let env_map = bind_environment(&loaded.bindings, &HashMap::new())
        .expect("EDGE_COUNT's bindings are empty — nothing to resolve");
    let env = EvalEnv {
        bindings: env_map,
        intrinsic_costs: &costs,
        graph: Some(&graph as &dyn GraphSubstrate),
        types: None,
        enums: None,
        elements: Vec::new(),
        draw_context: None,
    };
    let mut fuel = 10_000;
    let result = evaluate(when_clause(&loaded), &env, &EmptyIntrinsicHost, &mut fuel).unwrap();
    assert_eq!(
        result,
        Value::Bool(true),
        "one SOLIDARITY edge crosses the >= 1 threshold"
    );
}

/// §3.4 catches what Python's aggregate_and_compare silently permitted:
/// summing / unweighted-averaging an INTENSIVE field is the recorded
/// variance error. Python's `sum_strength`/`avg_strength`
/// (test_event_evaluator.py:307-323) and unweighted `avg` over agitation
/// are E-TYPE-041/042 here — a documented tightening, not a correction
/// row (the four M8 corrections are separate).
#[test]
fn intensive_aggregation_is_rejected_where_python_allowed_it() {
    let sum_strength = EDGE_COUNT.replace(
        "(fold count (edges EdgeType/SOLIDARITY) it)",
        "(fold sum (edges EdgeType/SOLIDARITY) solidarity/strength)",
    );
    assert_eq!(
        load(&sum_strength, "x.bsl").unwrap_err().spec_code(),
        Some("E-TYPE-041")
    );
    let avg_strength = EDGE_COUNT.replace(
        "(fold count (edges EdgeType/SOLIDARITY) it)",
        "(fold mean (edges EdgeType/SOLIDARITY) solidarity/strength)",
    );
    assert_eq!(
        load(&avg_strength, "x.bsl").unwrap_err().spec_code(),
        Some("E-TYPE-042")
    );
}

/// test_event_evaluator.py:326-369 (graph conditions + metrics) — the six
/// Python metrics are the registered :metric set; conditions over them
/// load, and their conjunction evaluates in the expression core.
#[test]
fn metric_conditions_load_and_evaluate() {
    let loaded = load(METRIC_CONDITIONS, "x.bsl").unwrap();
    // solidarity_graph analogue: density 2/6, wealth 550, agitation 0.4.
    let fires = [
        ("solidarity-density", real(2.0 / 6.0)),
        ("total-wealth", real(550.0)),
        ("average-agitation", real(0.4)),
    ];
    assert!(eval_when(&loaded, &owned(fires.to_vec())));
    // simple_graph analogue for density: no solidarity edges -> 0.0.
    let dormant = [
        ("solidarity-density", real(0.0)),
        ("total-wealth", real(550.0)),
        ("average-agitation", real(0.4)),
    ];
    assert!(!eval_when(&loaded, &owned(dormant.to_vec())));
}

/// test_event_evaluator.py:372-417 (PreconditionSet logic) — "all" is
/// `and`, "any" is `or`; both short-circuit per §4.1.
#[test]
fn precondition_logic_vectors() {
    let all = "(and (>= agitation 0.6p) (>= wealth 1000))";
    assert!(!eval_cond(
        all,
        &[("agitation", real(0.7)), ("wealth", real(550.0))]
    ));
    let any = "(or (>= agitation 0.6p) (>= wealth 1000))";
    assert!(eval_cond(
        any,
        &[("agitation", real(0.7)), ("wealth", real(550.0))]
    ));
}

/// test_event_evaluator.py:526-590 (resolution selection) — the
/// bifurcation rule EXECUTES: guards route the same effect list two ways
/// by solidarity density, against a real substrate.
#[test]
fn bifurcation_routes_by_solidarity_density() {
    let loaded = load(BIFURCATION, "x.bsl").unwrap();
    for (density, touched_field) in [
        (0.05, "social-class/national-identity"),
        (2.0 / 6.0, "social-class/class-consciousness"),
    ] {
        let mut graph = MemoryGraph::new();
        let self_id = graph.add_node("SOCIAL_CLASS").unwrap();
        graph
            .update_node(self_id, "social-class/national-identity", 0.2)
            .unwrap();
        graph
            .update_node(self_id, "social-class/class-consciousness", 0.4)
            .unwrap();
        let before = graph.node_attribute(self_id, touched_field).unwrap();

        let supplied = owned(vec![
            ("agitation", real(0.8)),
            ("solidarity-density", real(density)),
            ("self", Value::NodeRef(self_id)),
        ]);
        let mut env_map = bind_environment(&loaded.bindings, &supplied).unwrap();
        env_map.insert("self".to_owned(), Value::NodeRef(self_id));
        let costs = IntrinsicCosts::default();
        let env = EvalEnv {
            bindings: env_map,
            intrinsic_costs: &costs,
            graph: None,
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let babylon_bsl::SExpr::List(items) = &loaded.rule else {
            unreachable!()
        };
        let effects = items
            .iter()
            .find_map(|child| match child {
                babylon_bsl::SExpr::List(inner)
                    if matches!(inner.first(), Some(babylon_bsl::SExpr::Atom(babylon_bsl::Atom::Symbol(h))) if h == "effects") =>
                {
                    Some(&inner[1..])
                }
                _ => None,
            })
            .unwrap();
        let registries = registries();
        let mut executor = EffectExecutor::new(&registries.types, &registries.enums, None);
        let mut sink = CollectingSink::default();
        let mut fuel = 512;
        executor
            .execute_effects(
                effects,
                &env,
                &EmptyIntrinsicHost,
                &mut graph,
                &mut sink,
                &mut fuel,
            )
            .unwrap();
        let after = graph.node_attribute(self_id, touched_field).unwrap();
        assert!(
            (after - (before + 0.15)).abs() < 1e-12,
            "density {density} must route +0.15 to {touched_field}"
        );
    }
}

// ------------------------------------------------- the four M8 corrections

/// CORRECTION 1 of 4 — event_evaluator.py:313: `calculator() if
/// calculator else 0.0`. Old: an unknown graph metric silently reads 0.0.
/// New: an unregistered :metric is E-LOAD-011 at content load (§2.5) —
/// the rule never loads, nothing reads a phantom zero.
#[test]
fn correction_1_unknown_metric_is_e_load_011_not_zero() {
    let err = load(UNKNOWN_METRIC, "x.bsl").unwrap_err();
    assert_eq!(err.spec_code(), Some("E-LOAD-011"), "{err}");
}

/// CORRECTION 2 of 4 — event_evaluator.py:439: the aggregation dispatch
/// falls through to `return False`. Old: an unknown aggregation silently
/// never fires. New: an off-set fold operator is `E-PARSE-015` at parse.
///
/// **Code sharpened by R9 chapter C8 (D75).** This vector previously
/// asserted only the message text, because §6.3's disposition table named
/// the behaviour ("unknown aggregation → `E-PARSE-015` at parse") while no
/// numbered code existed to assert. D75 supplies it, and the grammar pass
/// now rejects the off-set head before the §3.4 checker sees it — an
/// earlier, better-classified rejection of the same content.
#[test]
fn correction_2_unknown_aggregation_is_e_parse_015_not_false() {
    let median = EDGE_COUNT.replace(
        "(fold count (edges EdgeType/SOLIDARITY) it)",
        "(fold median (edges EdgeType/SOLIDARITY) solidarity/strength)",
    );
    let err = load(&median, "x.bsl").unwrap_err();
    assert_eq!(err.spec_code(), Some("E-PARSE-015"), "{err}");
}

/// CORRECTION 3 of 4 — event_evaluator.py:405: the comparison dispatch
/// falls through to `return False`. Old: an unknown operator silently
/// never fires. New: a token outside the closed operator set is not even
/// an atom — E-LEX-003 at read time.
#[test]
fn correction_3_unknown_comparison_operator_is_a_lex_error_not_false() {
    let err = read("(~= agitation 0.5p)").unwrap_err();
    assert!(matches!(
        err.kind,
        babylon_bsl::ReadErrorKind::Lex(babylon_bsl::LexCode::UnclassifiableToken)
    ));
}

/// CORRECTION 4 of 4 — event_evaluator.py:103: `if not results: return
/// True`. Old: an empty precondition set silently always passes. New:
/// `(when)` is a loud load rejection (E-PARSE-020); "always" is written by
/// omitting the clause — explicit intent, never an accident.
#[test]
fn correction_4_empty_when_is_rejected_and_omission_is_the_legal_always() {
    let err = load(EMPTY_WHEN, "x.bsl").unwrap_err();
    assert!(err.to_string().contains("E-PARSE-020"), "{err}");
    let unconditional = load(UNCONDITIONAL, "x.bsl").unwrap();
    assert_eq!(
        unconditional.static_bound, 3,
        "an omitted <when> contributes 0; the emit costs 3"
    );
}

// ---------------------------------------------------------------- helpers

fn owned(pairs: Vec<(&str, Value)>) -> HashMap<String, Value> {
    pairs
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect()
}

fn eval_value(source: &str, env_pairs: &[(&str, Value)]) -> Value {
    let costs = IntrinsicCosts::default();
    let env = EvalEnv {
        bindings: owned(env_pairs.to_vec()),
        intrinsic_costs: &costs,
        graph: None,
        types: None,
        enums: None,
        elements: Vec::new(),
        draw_context: None,
    };
    let (expr, _) = read(source).expect("vector source must parse");
    let mut fuel = 10_000;
    evaluate(&expr, &env, &EmptyIntrinsicHost, &mut fuel).expect("vector must evaluate")
}

/// `eval_value`'s graph-bearing twin (PR 4, Task 14): `env.graph =
/// Some(graph)`, and the fuel meter is BY REFERENCE so a caller can
/// assert `:fuel-used` (§6.1).
fn eval_value_over_graph(
    source: &str,
    bindings: &HashMap<String, Value>,
    graph: &dyn GraphSubstrate,
    fuel: &mut u64,
) -> Value {
    let costs = IntrinsicCosts::default();
    let env = EvalEnv {
        bindings: bindings.clone(),
        intrinsic_costs: &costs,
        graph: Some(graph),
        types: None,
        enums: None,
        elements: Vec::new(),
        draw_context: None,
    };
    let (expr, _) = read(source).expect("vector source must parse");
    evaluate(&expr, &env, &EmptyIntrinsicHost, fuel).expect("vector must evaluate")
}

fn eval_cond(source: &str, env_pairs: &[(&str, Value)]) -> bool {
    match eval_value(source, env_pairs) {
        Value::Bool(b) => b,
        other => panic!("a <cond> must be Bool, got {other:?}"),
    }
}

fn eval_cond_err(source: &str, env_pairs: &[(&str, Value)]) -> babylon_bsl::EvalError {
    let costs = IntrinsicCosts::default();
    let env = EvalEnv {
        bindings: owned(env_pairs.to_vec()),
        intrinsic_costs: &costs,
        graph: None,
        types: None,
        enums: None,
        elements: Vec::new(),
        draw_context: None,
    };
    let (expr, _) = read(source).expect("vector source must parse");
    let mut fuel = 10_000;
    evaluate(&expr, &env, &EmptyIntrinsicHost, &mut fuel).expect_err("vector must fail")
}
