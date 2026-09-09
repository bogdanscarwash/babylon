use babylon_bsl::bindings::parse_bindings;
use babylon_bsl::causal_contract::{EvidenceClass, RuleContract, RuleRole};
use babylon_bsl::evaluator::{evaluate, EvalEnv, Value};
use babylon_bsl::fuel::IntrinsicCosts;
use babylon_bsl::intrinsic_host::EmptyIntrinsicHost;
use babylon_bsl::metrics::MetricRegistry;
use babylon_bsl::probability::{
    allocate_tickets, analyze_content_set,
    compile_rule_probability as compile_rule_probability_with_types, forecast_event_likelihoods,
    validate_probability_content_set, BranchProjectionV1, FiniteKernelV1, FiniteProjectionV1, Mass,
    ProbabilityError, FINITE_KERNEL_DRAW_BASE, TICKET_DENOMINATOR,
};
use babylon_bsl::reader::{read, read_all_spanned, Atom, SExpr};
use babylon_bsl::scenario::{
    load_scenario, load_scenario_with_named_preludes, NamedDeclarationPreludeV1,
};
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_bsl::tick::{
    forecast_event_likelihoods as forecast_detached, run_tick_observed, ForecastContextV1,
};
use babylon_bsl::types::{BslType, FieldDecl, FieldKind};
use babylon_bsl::write_log::CollectingWriteLog;
use babylon_bsl::{
    check_rule_with_kernel, expr_cost, load_rule, load_rule_form, split_content, BindSource,
    BindingDecl, BindingVocabulary, CardinalityCeilings, EnumRegistry, LoadContext, LoadError,
    TypeEnv,
};
use babylon_graph::memory::MemoryGraph;
use babylon_graph::stable_element::StableElementResolverV1;
use babylon_graph::substrate::GraphSubstrate;
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1, RngSeedContext};
use std::collections::{HashMap, HashSet};

#[test]
fn mass_literals_are_exact_unsigned_nanounits() {
    let (value, _) = read("1.250000000m").expect("mass literal parses");
    assert_eq!(
        value,
        SExpr::Atom(Atom::Mass(Mass::from_nanounits(1_250_000_000)))
    );
    assert!(
        read("-0m").is_err(),
        "lexically negative Mass refuses, including -0m"
    );
    assert!(
        read("0.0000000001m").is_err(),
        "Mass never rounds at lex time"
    );
}

#[test]
fn quantize_mass_is_the_only_dynamic_numeric_crossing() {
    let (expr, _) = read("(quantize-mass 0.5c)").unwrap();
    let costs = IntrinsicCosts::default();
    let env = EvalEnv {
        bindings: HashMap::new(),
        intrinsic_costs: &costs,
        graph: None,
        types: None,
        enums: None,
        elements: Vec::new(),
        draw_context: None,
    };
    let mut fuel = 100;
    assert_eq!(
        evaluate(&expr, &env, &EmptyIntrinsicHost, &mut fuel).unwrap(),
        Value::Mass(Mass::from_nanounits(500_000_000))
    );
}

#[test]
fn mass_addition_and_subtraction_are_exact_and_checked() {
    assert_eq!(
        Mass::from_nanounits(2)
            .checked_add(Mass::from_nanounits(3))
            .unwrap(),
        Mass::from_nanounits(5)
    );
    assert_eq!(
        Mass::from_nanounits(5)
            .checked_sub(Mass::from_nanounits(3))
            .unwrap(),
        Mass::from_nanounits(2)
    );
    assert_eq!(
        Mass::from_nanounits(u64::MAX).checked_add(Mass::from_nanounits(1)),
        Err(ProbabilityError::MassOverflow)
    );
    assert_eq!(
        Mass::from_nanounits(0).checked_sub(Mass::from_nanounits(1)),
        Err(ProbabilityError::MassUnderflow)
    );
}

#[test]
fn quantize_mass_refuses_negative_nonfinite_and_overflowing_inputs() {
    assert_eq!(
        Mass::quantize(-0.0),
        Ok(Mass::from_nanounits(0)),
        "signed zero is numerically zero and canonicalizes to Mass zero"
    );
    assert_eq!(
        Mass::quantize(-0.000_000_001),
        Err(ProbabilityError::NegativeMassInput)
    );
    for nonfinite in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            Mass::quantize(nonfinite),
            Err(ProbabilityError::NonFiniteMassInput)
        );
    }
    assert_eq!(
        Mass::quantize(f64::MAX),
        Err(ProbabilityError::MassOverflow)
    );
}

#[test]
fn largest_remainder_allocation_uses_enum_order_for_ties() {
    let equal = Mass::from_nanounits(1);
    let allocation = allocate_tickets(&[equal, equal, equal]).unwrap();
    let q = TICKET_DENOMINATOR / 3;
    assert_eq!(allocation[0].count, q + 1);
    assert_eq!(allocation[1].count, q);
    assert_eq!(allocation[2].count, q);
    assert_eq!(allocation.last().unwrap().end, TICKET_DENOMINATOR);
}

#[test]
fn an_all_zero_kernel_is_not_a_probability_distribution() {
    assert_eq!(
        allocate_tickets(&[Mass::from_nanounits(0), Mass::from_nanounits(0)]),
        Err(ProbabilityError::ZeroTotalMass)
    );
}

fn spark_enums() -> EnumRegistry {
    let mut enums = EnumRegistry::default();
    enums
        .declare(
            "SparkOutcome",
            &["EXCESSIVE_FORCE".to_owned(), "NO_INCIDENT".to_owned()],
        )
        .unwrap();
    enums
}

fn contract(role: RuleRole) -> RuleContract {
    RuleContract {
        rule_id: "demo/spark".to_owned(),
        role,
        evidence: EvidenceClass::Designed,
    }
}

fn kernel_rule(branches: &str) -> SExpr {
    read(&format!(
        "(rule demo/spark :role mechanic :evidence designed \
         :material-basis \"bounded material spark\" :fuel 64 \
         (bindings) (effects \
           (choose :sample struggle/spark :slot 0 {branches})))"
    ))
    .unwrap()
    .0
}

fn compile_rule_probability(
    rule: &SExpr,
    root_path: &[u32],
    contract: &RuleContract,
    enums: &EnumRegistry,
    bindings: &[BindingDecl],
    consts: &HashMap<String, Value>,
) -> Result<(Option<FiniteKernelV1>, Option<FiniteProjectionV1>), ProbabilityError> {
    let types = TypeEnv {
        fields: HashMap::new(),
        exemptions: &[],
    };
    let compiled = compile_rule_probability_with_types(
        rule,
        root_path,
        contract,
        enums,
        &types,
        bindings,
        consts,
        &HashSet::new(),
    )?;
    Ok((compiled.kernel, compiled.projection))
}

#[test]
fn canonical_choose_compiles_once_to_enum_ordered_ir() {
    let rule = kernel_rule(
        "(branch SparkOutcome/EXCESSIVE_FORCE :mass 1m (effects)) \
         (branch SparkOutcome/NO_INCIDENT :mass 3m (effects))",
    );
    let types = TypeEnv {
        fields: HashMap::new(),
        exemptions: &[],
    };
    let compiled = compile_rule_probability_with_types(
        &rule,
        &[0],
        &contract(RuleRole::Mechanic),
        &spark_enums(),
        &types,
        &[],
        &HashMap::new(),
        &HashSet::new(),
    )
    .unwrap();
    assert_eq!(compiled.facts.mass_literals.len(), 2);
    let kernel = compiled.kernel.expect("choose compiled");
    assert!(compiled.projection.is_none());
    assert_eq!(kernel.sample, "struggle/spark");
    assert_eq!(kernel.slot, 0);
    assert_eq!(kernel.branches[0].member, "EXCESSIVE_FORCE");
    assert_eq!(kernel.branches[1].member, "NO_INCIDENT");
    assert_eq!(kernel.branches[0].mass_literals.len(), 1);
    assert_eq!(kernel.branches[1].mass_literals.len(), 1);
    assert_eq!(
        check_rule_with_kernel(
            &rule,
            Some(&kernel),
            &CardinalityCeilings::default(),
            &IntrinsicCosts::default(),
        )
        .unwrap(),
        FINITE_KERNEL_DRAW_BASE
    );
}

#[test]
fn multi_top_form_kernel_paths_retain_the_original_source_coordinate_space() {
    let source = r#"(intrinsic floor :params (real) :returns int :cost 5)
(rule demo/first :role mechanic :evidence derived
  :material-basis "first rule occupies top-form one" :fuel 8
  (bindings) (effects))
(rule demo/spark :role mechanic :evidence designed
  :material-basis "second rule owns the kernel paths" :fuel 64
  (bindings (binding current :field social-class/value))
  (effects
    (choose :sample struggle/spark :slot 0
      (branch SparkOutcome/EXCESSIVE_FORCE :mass 1m (effects))
      (branch SparkOutcome/NO_INCIDENT :mass 3m (effects)))))"#;
    let (_, mut split) = split_content(source).unwrap();
    let second = split.pop().expect("second rule is retained");
    assert_eq!(second.rule_id, "demo/spark");
    assert_eq!(second.root_path, vec![2]);

    let fields = HashMap::from([(
        "social-class/value".to_owned(),
        FieldDecl {
            ty: BslType::Int,
            kind: FieldKind::Extensive,
        },
    )]);
    let vocabulary = BindingVocabulary {
        fields: fields.keys().cloned().collect(),
        consts: HashSet::new(),
        probability_consts: HashSet::new(),
        metrics: HashSet::new(),
    };
    let types = TypeEnv {
        fields,
        exemptions: &[],
    };
    let enums = spark_enums();
    let const_values = HashMap::new();
    let ceilings = CardinalityCeilings::default();
    let intrinsics = IntrinsicCosts::default();
    let systems = HashSet::from(["demo".to_owned(), "struggle".to_owned()]);
    let context = LoadContext {
        vocabulary: &vocabulary,
        types: &types,
        enums: &enums,
        const_values: &const_values,
        ceilings: &ceilings,
        intrinsics: &intrinsics,
        systems: &systems,
        vocabulary_registry: None,
        rule_file: "demo/multi.bsl",
    };
    let loaded = load_rule_form(second.form, second.root_path, &context).unwrap();
    assert_eq!(loaded.root_path, vec![2]);
    let kernel = loaded.kernel.expect("second rule choose compiles");
    assert!(kernel.form_path.starts_with(&[2]));
    let (_, spans) = read_all_spanned(source.as_bytes()).unwrap();
    let choose_head = spans.span_of(&kernel.head_path).unwrap();
    assert_eq!(&source[choose_head.start..choose_head.end], "choose");
    let slot = spans.span_of(&kernel.slot_path).unwrap();
    assert_eq!(&source[slot.start..slot.end], "0");

    let invalid_source = source.replacen(":mass 1m", ":mass 0.5c", 1);
    let (_, mut split) = split_content(&invalid_source).unwrap();
    let invalid_second = split.pop().unwrap();
    let error = load_rule_form(invalid_second.form, invalid_second.root_path, &context)
        .expect_err("ordinary Real cannot enter a Mass slot");
    let LoadError::Probability(ProbabilityError::InvalidForm { form_path, .. }) = error else {
        panic!("expected typed probability path, got {error:?}");
    };
    assert!(form_path.starts_with(&[2]));
    let (_, spans) = read_all_spanned(invalid_source.as_bytes()).unwrap();
    let mass_span = spans.span_of(&form_path).unwrap();
    assert_eq!(&invalid_source[mass_span.start..mass_span.end], "0.5c");
}

#[test]
fn choose_requires_exact_enum_exhaustiveness_and_direct_placement() {
    let reversed = kernel_rule(
        "(branch SparkOutcome/NO_INCIDENT :mass 3m (effects)) \
         (branch SparkOutcome/EXCESSIVE_FORCE :mass 1m (effects))",
    );
    let error = compile_rule_probability(
        &reversed,
        &[0],
        &contract(RuleRole::Mechanic),
        &spark_enums(),
        &[],
        &HashMap::new(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("declaration order"));

    let missing = kernel_rule("(branch SparkOutcome/EXCESSIVE_FORCE :mass 1m (effects))");
    assert!(compile_rule_probability(
        &missing,
        &[0],
        &contract(RuleRole::Mechanic),
        &spark_enums(),
        &[],
        &HashMap::new(),
    )
    .unwrap_err()
    .to_string()
    .contains("exhaust"));

    assert!(compile_rule_probability(
        &kernel_rule(
            "(branch SparkOutcome/EXCESSIVE_FORCE :mass 1m (effects)) \
             (branch SparkOutcome/NO_INCIDENT :mass 1m (effects))",
        ),
        &[0],
        &contract(RuleRole::Recognizer),
        &spark_enums(),
        &[],
        &HashMap::new(),
    )
    .unwrap_err()
    .to_string()
    .contains("only a Mechanic"));

    let nested = read(
        "(rule demo/spark :role mechanic :evidence designed \
         :material-basis \"nested choice is not the AJ boundary\" :fuel 64 \
         (bindings) (effects (guard #t \
           (choose :sample struggle/spark :slot 0 \
             (branch SparkOutcome/EXCESSIVE_FORCE :mass 1m (effects)) \
             (branch SparkOutcome/NO_INCIDENT :mass 1m (effects))))))",
    )
    .unwrap()
    .0;
    assert!(compile_rule_probability(
        &nested,
        &[0],
        &contract(RuleRole::Mechanic),
        &spark_enums(),
        &[],
        &HashMap::new(),
    )
    .unwrap_err()
    .to_string()
    .contains("direct child"));

    for (forbidden_body, expected) in [
        ("(emit EventType/X)", "branch body cannot contain"),
        (
            "(guard #t (choose :sample struggle/nested :slot 1 \
               (branch SparkOutcome/EXCESSIVE_FORCE :mass 1m (effects)) \
               (branch SparkOutcome/NO_INCIDENT :mass 1m (effects))))",
            "at most one choose",
        ),
    ] {
        let rule = kernel_rule(&format!(
            "(branch SparkOutcome/EXCESSIVE_FORCE :mass 1m (effects {forbidden_body})) \
             (branch SparkOutcome/NO_INCIDENT :mass 1m (effects))"
        ));
        let error = compile_rule_probability(
            &rule,
            &[0],
            &contract(RuleRole::Mechanic),
            &spark_enums(),
            &[],
            &HashMap::new(),
        )
        .unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn standalone_kernels_accept_otherwise_governed_cross_carrier_material_writes() {
    let cross_node_branch = kernel_rule(
        "(branch SparkOutcome/EXCESSIVE_FORCE :mass 1m
           (effects (update-node other demo/x (set 1))))
         (branch SparkOutcome/NO_INCIDENT :mass 1m (effects))",
    );
    let (kernel, projection) = compile_rule_probability(
        &cross_node_branch,
        &[0],
        &contract(RuleRole::Mechanic),
        &spark_enums(),
        &[],
        &HashMap::new(),
    )
    .expect("carrier locality applies only when an exact projection is linked");
    assert!(kernel.is_some());
    assert!(projection.is_none());

    let shared_edge_branch = kernel_rule(
        "(branch SparkOutcome/EXCESSIVE_FORCE :mass 1m
           (effects (update-edge shared-edge demo/x (set 1))))
         (branch SparkOutcome/NO_INCIDENT :mass 1m (effects))",
    );
    let (kernel, projection) = compile_rule_probability(
        &shared_edge_branch,
        &[0],
        &contract(RuleRole::Mechanic),
        &spark_enums(),
        &[],
        &HashMap::new(),
    )
    .expect("a standalone joint kernel may govern a shared carrier");
    assert!(kernel.is_some());
    assert!(projection.is_none());
}

#[test]
fn kernel_fuel_charges_every_mass_and_only_the_maximum_branch_body() {
    let rule = kernel_rule(
        "(branch SparkOutcome/EXCESSIVE_FORCE :mass (+ 1m 2m)
           (effects (update-node self demo/x (set 1))))
         (branch SparkOutcome/NO_INCIDENT :mass (quantize-mass (+ 0.5c 0.25c))
           (effects
             (update-node self demo/x (set (+ 1 2)))
             (update-node self demo/x (add 1))))",
    );
    let (kernel, _) = compile_rule_probability(
        &rule,
        &[0],
        &contract(RuleRole::Mechanic),
        &spark_enums(),
        &[],
        &HashMap::new(),
    )
    .unwrap();
    let kernel = kernel.unwrap();
    assert_eq!(
        kernel.branches[1].static_mass,
        Some(Mass::from_nanounits(750_000_000)),
        "source-independent binary64 arithmetic must fold before LSP allocation analysis"
    );
    let ceilings = CardinalityCeilings::default();
    let intrinsics = IntrinsicCosts::default();
    let mass_cost: u64 = kernel
        .branches
        .iter()
        .map(|branch| expr_cost(&branch.mass, &ceilings, &intrinsics).unwrap())
        .sum();
    let body_costs: Vec<u64> = kernel
        .branches
        .iter()
        .map(|branch| {
            branch
                .effects
                .iter()
                .map(|effect| expr_cost(effect, &ceilings, &intrinsics).unwrap())
                .sum()
        })
        .collect();
    assert_ne!(body_costs[0], body_costs[1]);
    let expected = FINITE_KERNEL_DRAW_BASE + mass_cost + body_costs[0].max(body_costs[1]);
    assert_eq!(
        check_rule_with_kernel(&rule, Some(&kernel), &ceilings, &intrinsics).unwrap(),
        expected
    );
    assert_ne!(
        expected,
        FINITE_KERNEL_DRAW_BASE + mass_cost + body_costs.iter().sum::<u64>()
    );
}

#[test]
fn a_non_mass_constant_or_expr_binding_cannot_enter_branch_mass() {
    let rule = kernel_rule(
        "(branch SparkOutcome/EXCESSIVE_FORCE :mass weight (effects)) \
         (branch SparkOutcome/NO_INCIDENT :mass 1m (effects))",
    );
    let binding = BindingDecl {
        name: "weight".to_owned(),
        source: BindSource::Const("demo/ordinary-probability".to_owned()),
        optional: false,
        default: None,
    };
    let ordinary = HashMap::from([("demo/ordinary-probability".to_owned(), Value::Real(0.5))]);
    let error = compile_rule_probability(
        &rule,
        &[0],
        &contract(RuleRole::Mechanic),
        &spark_enums(),
        &[binding],
        &ordinary,
    )
    .unwrap_err();
    assert!(error.to_string().contains("static Mass type"));

    let ordinary_expr = BindingDecl {
        name: "weight".to_owned(),
        source: BindSource::Expr(read("0.5c").unwrap().0),
        optional: false,
        default: None,
    };
    let error = compile_rule_probability(
        &rule,
        &[0],
        &contract(RuleRole::Mechanic),
        &spark_enums(),
        &[ordinary_expr],
        &HashMap::new(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("static Mass type"));
}

#[test]
fn quantize_mass_requires_an_ordinary_numeric_operand_at_load() {
    let rule_with_bindings = |bindings: &str| {
        format!(
            "(rule demo/spark :role mechanic :evidence designed \
             :material-basis \"quantize cannot consume Mass\" :fuel 64 \
             (bindings {bindings}) \
             (effects (choose :sample struggle/spark :slot 0 \
               (branch SparkOutcome/EXCESSIVE_FORCE :mass spark-mass (effects)) \
               (branch SparkOutcome/NO_INCIDENT :mass 1m (effects)))))"
        )
    };
    for invalid in [
        rule_with_bindings("(binding spark-mass :expr (quantize-mass (+ 1m 2m)))"),
        rule_with_bindings(
            "(binding base-mass :expr 1m) \
             (binding spark-mass :expr (quantize-mass base-mass))",
        ),
        rule_with_bindings("(binding spark-mass :expr (quantize-mass (quantize-mass 0.5c)))"),
    ] {
        let rule = read(&invalid).unwrap().0;
        let bindings = parse_bindings(&rule).unwrap();
        let error = compile_rule_probability(
            &rule,
            &[0],
            &contract(RuleRole::Mechanic),
            &spark_enums(),
            &bindings,
            &HashMap::new(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("statically Int/Real-lane"));
    }

    let valid = rule_with_bindings("(binding spark-mass :expr (quantize-mass (* 0.5c 2)))");
    let rule = read(&valid).unwrap().0;
    let bindings = parse_bindings(&rule).unwrap();
    assert!(compile_rule_probability(
        &rule,
        &[0],
        &contract(RuleRole::Mechanic),
        &spark_enums(),
        &bindings,
        &HashMap::new(),
    )
    .is_ok());
}

#[test]
fn quantize_mass_refuses_int_division_at_load() {
    for (bindings_source, division) in [
        (
            "(binding spark-mass :expr (quantize-mass (/ 4 2)))",
            "(/ 4 2)",
        ),
        (
            "(binding numerator :expr 4) \
             (binding denominator :expr 2) \
             (binding spark-mass :expr (quantize-mass (/ numerator denominator)))",
            "(/ numerator denominator)",
        ),
    ] {
        let source = format!(
            "(rule demo/spark :role mechanic :evidence designed \
             :material-basis \"Int division has no runtime semantics\" :fuel 64 \
             (bindings {bindings_source}) \
             (effects (choose :sample struggle/spark :slot 0 \
               (branch SparkOutcome/EXCESSIVE_FORCE :mass spark-mass (effects)) \
               (branch SparkOutcome/NO_INCIDENT :mass 1m (effects)))))"
        );
        let rule = read(&source).unwrap().0;
        let bindings = parse_bindings(&rule).unwrap();
        let error = compile_rule_probability(
            &rule,
            &[0],
            &contract(RuleRole::Mechanic),
            &spark_enums(),
            &bindings,
            &HashMap::new(),
        )
        .expect_err("quantize-mass must not load an Int / Int expression the evaluator refuses");
        let ProbabilityError::InvalidForm { message, form_path } = error else {
            panic!("expected a located quantize-mass refusal, got {error:?}");
        };
        assert!(message.contains("statically Int/Real-lane"), "{message}");
        let (_, spans) = read_all_spanned(source.as_bytes()).unwrap();
        let span = spans.span_of(&form_path).unwrap();
        assert_eq!(&source[span.start..span.end], division);
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn quantize_mass_uses_declared_types_for_fields_metrics_and_accessors() {
    let mut enums = spark_enums();
    let flag_type = enums
        .declare("Flag", &["OFF".to_owned(), "ON".to_owned()])
        .unwrap();
    let types = TypeEnv {
        fields: HashMap::from([
            (
                "demo/bool".to_owned(),
                FieldDecl {
                    ty: BslType::Bool,
                    kind: FieldKind::NotApplicable,
                },
            ),
            (
                "demo/enum".to_owned(),
                FieldDecl {
                    ty: BslType::Enum(flag_type),
                    kind: FieldKind::NotApplicable,
                },
            ),
            (
                "demo/currency".to_owned(),
                FieldDecl {
                    ty: BslType::Currency,
                    kind: FieldKind::Extensive,
                },
            ),
            (
                "bool-metric".to_owned(),
                FieldDecl {
                    ty: BslType::Bool,
                    kind: FieldKind::NotApplicable,
                },
            ),
        ]),
        exemptions: &[],
    };
    let cases = [
        ("flag", BindSource::Field("demo/bool".to_owned())),
        ("flag", BindSource::Field("demo/enum".to_owned())),
        ("flag", BindSource::Field("demo/currency".to_owned())),
        ("flag", BindSource::Metric("bool-metric".to_owned())),
    ];
    for (name, source) in cases {
        let rule = read(&format!(
            "(rule demo/spark :role mechanic :evidence designed \
             :material-basis \"typed quantization\" :fuel 64 \
             (bindings (binding spark-mass :expr (quantize-mass {name}))) \
             (effects (choose :sample struggle/spark :slot 0 \
               (branch SparkOutcome/EXCESSIVE_FORCE :mass spark-mass (effects)) \
               (branch SparkOutcome/NO_INCIDENT :mass 1m (effects)))))"
        ))
        .unwrap()
        .0;
        let bindings = [
            BindingDecl {
                name: name.to_owned(),
                source,
                optional: false,
                default: None,
            },
            BindingDecl {
                name: "spark-mass".to_owned(),
                source: BindSource::Expr(read(&format!("(quantize-mass {name})")).unwrap().0),
                optional: false,
                default: None,
            },
        ];
        let error = compile_rule_probability_with_types(
            &rule,
            &[0],
            &contract(RuleRole::Mechanic),
            &enums,
            &types,
            &bindings,
            &HashMap::new(),
            &HashSet::new(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("statically Int/Real-lane"));
    }

    for operand in [
        "#t",
        "Flag/ON",
        "(field-of self demo/bool)",
        "(field-of self demo/enum)",
        "(field-of self demo/currency)",
        "(metric-of self bool-metric)",
    ] {
        let source = format!(
            "(rule demo/spark :role mechanic :evidence designed \
             :material-basis \"typed accessor quantization\" :fuel 64 \
             (bindings (binding spark-mass :expr (quantize-mass {operand}))) \
             (effects (choose :sample struggle/spark :slot 0 \
               (branch SparkOutcome/EXCESSIVE_FORCE :mass spark-mass (effects)) \
               (branch SparkOutcome/NO_INCIDENT :mass 1m (effects)))))"
        );
        let rule = read(&source).unwrap().0;
        let bindings = parse_bindings(&rule).unwrap();
        let error = compile_rule_probability_with_types(
            &rule,
            &[0],
            &contract(RuleRole::Mechanic),
            &enums,
            &types,
            &bindings,
            &HashMap::new(),
            &HashSet::new(),
        )
        .unwrap_err();
        let ProbabilityError::InvalidForm { form_path, .. } = error else {
            panic!("expected a located type refusal, got {error:?}");
        };
        let (_, spans) = read_all_spanned(source.as_bytes()).unwrap();
        let span = spans.span_of(&form_path).unwrap();
        assert_eq!(&source[span.start..span.end], operand);
    }
}

#[test]
fn a_mass_binding_declaration_name_is_not_mistaken_for_a_value_use() {
    let source = "(rule demo/spark :role mechanic :evidence designed \
        :material-basis \"bound Mass\" :fuel 64 \
        (bindings (binding spark-mass :expr 1m)) \
        (effects (choose :sample struggle/spark :slot 0 \
          (branch SparkOutcome/EXCESSIVE_FORCE :mass spark-mass (effects)) \
          (branch SparkOutcome/NO_INCIDENT :mass 3m (effects)))))";
    let rule = read(source).unwrap().0;
    let bindings = parse_bindings(&rule).unwrap();
    assert!(compile_rule_probability(
        &rule,
        &[0],
        &contract(RuleRole::Mechanic),
        &spark_enums(),
        &bindings,
        &HashMap::new(),
    )
    .is_ok());

    let illegal_use = source.replace("(effects (choose", "(when spark-mass) (effects (choose");
    let rule = read(&illegal_use).unwrap().0;
    let bindings = parse_bindings(&rule).unwrap();
    let error = compile_rule_probability(
        &rule,
        &[0],
        &contract(RuleRole::Mechanic),
        &spark_enums(),
        &bindings,
        &HashMap::new(),
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("legal only in another Mass binding or branch :mass expression"));
}

#[test]
fn content_set_linking_refuses_duplicate_samples_and_nonadjacent_projections() {
    let fields = HashMap::from([(
        "social-class/value".to_owned(),
        FieldDecl {
            ty: BslType::Int,
            kind: FieldKind::Extensive,
        },
    )]);
    let vocabulary = BindingVocabulary {
        fields: fields.keys().cloned().collect(),
        consts: HashSet::new(),
        probability_consts: HashSet::new(),
        metrics: HashSet::new(),
    };
    let types = TypeEnv {
        fields,
        exemptions: &[],
    };
    let enums = spark_enums();
    let const_values = HashMap::new();
    let ceilings = CardinalityCeilings::default();
    let intrinsics = IntrinsicCosts::default();
    let systems = HashSet::from(["demo".to_owned(), "struggle".to_owned()]);
    let context = LoadContext {
        vocabulary: &vocabulary,
        types: &types,
        enums: &enums,
        const_values: &const_values,
        ceilings: &ceilings,
        intrinsics: &intrinsics,
        systems: &systems,
        vocabulary_registry: None,
        rule_file: "demo/kernel.bsl",
    };
    let source = "(rule demo/ID :role mechanic :evidence designed \
        :material-basis \"duplicate sample\" :fuel 64 \
        (bindings (binding current :field social-class/value)) \
        (effects (choose :sample struggle/spark :slot 0 \
          (branch SparkOutcome/EXCESSIVE_FORCE :mass 1m (effects)) \
          (branch SparkOutcome/NO_INCIDENT :mass 1m (effects)))))";
    let first = load_rule(&source.replace("demo/ID", "demo/first"), &context).unwrap();
    let second = load_rule(&source.replace("demo/ID", "demo/second"), &context).unwrap();
    assert!(matches!(
        validate_probability_content_set(&[first, second]),
        Err(ProbabilityError::DuplicateSample { .. })
    ));
}

#[test]
fn kernels_and_projections_require_a_stable_subject_carrier_at_load() {
    let fields = HashMap::from([
        (
            "social-class/value".to_owned(),
            FieldDecl {
                ty: BslType::Int,
                kind: FieldKind::Extensive,
            },
        ),
        (
            "organization/value".to_owned(),
            FieldDecl {
                ty: BslType::Int,
                kind: FieldKind::Extensive,
            },
        ),
    ]);
    let vocabulary = BindingVocabulary {
        fields: fields.keys().cloned().collect(),
        consts: HashSet::new(),
        probability_consts: HashSet::new(),
        metrics: HashSet::new(),
    };
    let types = TypeEnv {
        fields,
        exemptions: &[],
    };
    let enums = spark_enums();
    let const_values = HashMap::new();
    let ceilings = CardinalityCeilings::default();
    let intrinsics = IntrinsicCosts::default();
    let systems = HashSet::from(["demo".to_owned(), "struggle".to_owned()]);
    let context = LoadContext {
        vocabulary: &vocabulary,
        types: &types,
        enums: &enums,
        const_values: &const_values,
        ceilings: &ceilings,
        intrinsics: &intrinsics,
        systems: &systems,
        vocabulary_registry: None,
        rule_file: "demo/probability.bsl",
    };
    let cases = [
        (
            "(rule demo/kernel :role mechanic :evidence designed \
             :material-basis \"bounded material alternatives\" :fuel 64 \
             (bindings) (effects \
               (choose :sample struggle/spark :slot 0 \
                 (branch SparkOutcome/EXCESSIVE_FORCE :mass 1m (effects)) \
                 (branch SparkOutcome/NO_INCIDENT :mass 1m (effects)))))",
            "finite kernel requires a stable subject carrier",
            "no :field binding",
        ),
        (
            "(rule struggle/spark-recognizer :role recognizer :evidence derived \
             :projects-kernel struggle/spark \
             :material-basis \"deterministic observation\" :fuel 64 \
             (bindings) (effects (emit EventType/EXCESSIVE_FORCE)))",
            "finite projection requires a stable subject carrier",
            "no :field binding",
        ),
        (
            "(rule demo/kernel :role mechanic :evidence designed \
             :material-basis \"ambiguous material alternatives\" :fuel 64 \
             (bindings \
               (binding current :field social-class/value) \
               (binding organization :field organization/value)) \
             (effects (choose :sample struggle/spark :slot 0 \
               (branch SparkOutcome/EXCESSIVE_FORCE :mass 1m (effects)) \
               (branch SparkOutcome/NO_INCIDENT :mass 1m (effects)))))",
            "finite kernel requires a stable subject carrier",
            "span 2 namespaces",
        ),
        (
            "(rule struggle/spark-recognizer :role recognizer :evidence derived \
             :projects-kernel struggle/spark \
             :material-basis \"ambiguous deterministic observation\" :fuel 64 \
             (bindings \
               (binding current :field social-class/value) \
               (binding organization :field organization/value)) \
             (effects (emit EventType/EXCESSIVE_FORCE)))",
            "finite projection requires a stable subject carrier",
            "span 2 namespaces",
        ),
    ];
    for (source, expected, carrier_detail) in cases {
        let error = load_rule(source, &context)
            .expect_err("finite probability rules without carriers must fail at load");
        let LoadError::Probability(ProbabilityError::InvalidForm { message, form_path }) = error
        else {
            panic!("expected a located probability refusal, got {error:?}");
        };
        assert!(message.contains(expected), "{message}");
        assert!(message.contains(carrier_detail), "{message}");
        let (_, spans) = read_all_spanned(source.as_bytes()).unwrap();
        let span = spans.span_of(&form_path).unwrap();
        assert_eq!(&source[span.start..span.end], "struggle/spark");
    }
}

fn assert_carrier_locality_refusal(error: ProbabilityError, source: &str) {
    let ProbabilityError::InvalidForm { message, form_path } = error else {
        panic!("expected a located carrier-locality refusal, got {error:?}");
    };
    assert!(message.contains("carrier-local"), "{message}");
    let (_, spans) = read_all_spanned(source.as_bytes()).unwrap();
    let span = spans.span_of(&form_path).unwrap();
    assert_eq!(&source[span.start..span.end], "(the NodeType/SOCIAL_CLASS)");
}

fn assert_compiled_locality_survives_raw_rule_mutation(
    mut cross_carrier_kernel: babylon_bsl::LoadedRule,
    replacement_rule: SExpr,
    matching_projection: babylon_bsl::LoadedRule,
    cross_carrier_source: &str,
) {
    cross_carrier_kernel.rule = replacement_rule;
    let error = validate_probability_content_set(&[cross_carrier_kernel, matching_projection])
        .expect_err("compiled carrier-locality evidence must survive raw-rule mutation");
    assert_carrier_locality_refusal(error, cross_carrier_source);
}

#[test]
#[allow(clippy::too_many_lines)]
fn adjacent_projection_requires_the_kernel_subject_carrier_at_its_sample_path() {
    let fields = HashMap::from([
        (
            "social-class/value".to_owned(),
            FieldDecl {
                ty: BslType::Int,
                kind: FieldKind::Extensive,
            },
        ),
        (
            "organization/value".to_owned(),
            FieldDecl {
                ty: BslType::Int,
                kind: FieldKind::Extensive,
            },
        ),
    ]);
    let vocabulary = BindingVocabulary {
        fields: fields.keys().cloned().collect(),
        consts: HashSet::new(),
        probability_consts: HashSet::new(),
        metrics: HashSet::new(),
    };
    let types = TypeEnv {
        fields,
        exemptions: &[],
    };
    let enums = spark_enums();
    let const_values = HashMap::new();
    let ceilings = CardinalityCeilings::new(
        HashMap::from([("NodeType/SOCIAL_CLASS".to_owned(), 1)]),
        HashMap::new(),
    );
    let intrinsics = IntrinsicCosts::default();
    let systems = HashSet::from(["demo".to_owned(), "struggle".to_owned()]);
    let context = LoadContext {
        vocabulary: &vocabulary,
        types: &types,
        enums: &enums,
        const_values: &const_values,
        ceilings: &ceilings,
        intrinsics: &intrinsics,
        systems: &systems,
        vocabulary_registry: None,
        rule_file: "demo/probability.bsl",
    };
    let kernel_source = "(rule struggle/spark-mechanic :role mechanic :evidence designed \
        :material-basis \"social-class spark\" :fuel 64 \
        (bindings (binding current :field social-class/value)) \
        (effects (choose :sample struggle/spark :slot 0 \
          (branch SparkOutcome/EXCESSIVE_FORCE :mass 1m (effects)) \
          (branch SparkOutcome/NO_INCIDENT :mass 1m (effects)))))";
    let projection_source = "(rule struggle/spark-recognizer :role recognizer :evidence derived \
        :projects-kernel struggle/spark \
        :material-basis \"organization observation\" :fuel 64 \
        (bindings (binding current :field organization/value)) \
        (effects (emit EventType/EXCESSIVE_FORCE)))";
    let kernel = load_rule(kernel_source, &context).unwrap();
    let projection = load_rule(projection_source, &context).unwrap();
    assert_eq!(kernel.probability_carrier.as_deref(), Some("SOCIAL_CLASS"));
    assert_eq!(
        projection.probability_carrier.as_deref(),
        Some("ORGANIZATION")
    );
    let error = validate_probability_content_set(&[kernel.clone(), projection]).unwrap_err();
    let ProbabilityError::SubjectCarrierMismatch(details) = error else {
        panic!("expected a typed carrier mismatch, got {error:?}");
    };
    assert_eq!(details.sample, "struggle/spark");
    assert_eq!(details.kernel_carrier, "SOCIAL_CLASS");
    assert_eq!(details.projection_carrier, "ORGANIZATION");
    let (_, spans) = read_all_spanned(projection_source.as_bytes()).unwrap();
    let span = spans.span_of(&details.form_path).unwrap();
    assert_eq!(&projection_source[span.start..span.end], "struggle/spark");

    let matching_source = projection_source.replace("organization/value", "social-class/value");
    let matching = load_rule(&matching_source, &context).unwrap();
    validate_probability_content_set(&[kernel.clone(), matching.clone()]).unwrap();

    let cross_carrier_source = "(rule struggle/spark-mechanic :role mechanic \
        :evidence designed :material-basis \"joint singleton carrier\" :fuel 64 \
        (bindings (binding current :field social-class/value)) \
        (effects (choose :sample struggle/spark :slot 0 \
          (branch SparkOutcome/EXCESSIVE_FORCE :mass 1m \
            (effects (update-node (the NodeType/SOCIAL_CLASS) \
              social-class/value (set 1)))) \
          (branch SparkOutcome/NO_INCIDENT :mass 1m (effects)))))";
    let cross_carrier_kernel = load_rule(cross_carrier_source, &context)
        .expect("an otherwise-governed standalone joint-carrier kernel loads");
    validate_probability_content_set(std::slice::from_ref(&cross_carrier_kernel))
        .expect("standalone kernels are not constrained by projection enumerability");
    let error = validate_probability_content_set(&[cross_carrier_kernel.clone(), matching.clone()])
        .expect_err("an established finite projection requires carrier-local kernel effects");
    assert_carrier_locality_refusal(error, cross_carrier_source);

    let membership_source = "(rule struggle/spark-mechanic :role mechanic \
        :evidence designed :material-basis \"shared membership carrier\" :fuel 64 \
        (bindings (binding current :field social-class/value)) \
        (effects (choose :sample struggle/spark :slot 0 \
          (branch SparkOutcome/EXCESSIVE_FORCE :mass 1m \
            (effects (update-membership self self (set social-class/value 1)))) \
          (branch SparkOutcome/NO_INCIDENT :mass 1m (effects)))))";
    let membership_rule = read(membership_source).unwrap().0;
    let membership_bindings = parse_bindings(&membership_rule).unwrap();
    let membership_probability = compile_rule_probability_with_types(
        &membership_rule,
        &[0],
        &kernel.contract,
        &enums,
        &types,
        &membership_bindings,
        &const_values,
        &HashSet::new(),
    )
    .expect("a standalone membership kernel may compile before projection linking");
    let mut membership_kernel = kernel.clone();
    membership_kernel.rule = membership_rule;
    membership_kernel.bindings = membership_bindings;
    membership_kernel.kernel = membership_probability.kernel;
    membership_kernel.probability_facts = membership_probability.facts;
    let error = validate_probability_content_set(&[membership_kernel, matching.clone()])
        .expect_err("a paired finite projection cannot enumerate a shared membership write");
    let ProbabilityError::InvalidForm { message, form_path } = error else {
        panic!("expected a located membership-locality refusal, got {error:?}");
    };
    assert!(
        message.contains("shared or graph-shape writes"),
        "{message}"
    );
    let (_, spans) = read_all_spanned(membership_source.as_bytes()).unwrap();
    let span = spans.span_of(&form_path).unwrap();
    assert_eq!(
        &membership_source[span.start..span.end],
        "update-membership"
    );

    // Replacing the raw rule after load cannot erase or relocate the compiled
    // whole-rule locality result used by projection linking.
    assert_compiled_locality_survives_raw_rule_mutation(
        cross_carrier_kernel,
        kernel.rule,
        matching,
        cross_carrier_source,
    );
}

#[test]
fn a_finite_projection_refuses_graph_global_metric_bindings_at_load() {
    let mut graph = MemoryGraph::new();
    let scenario = load_scenario(
        "(scenario demo/projection \
          (deffield social-class/value int extensive) \
          (node subject NodeType/SOCIAL_CLASS (social-class/value 0)))",
        &mut graph,
    )
    .unwrap();
    let types = TypeEnv {
        fields: scenario.fields.clone(),
        exemptions: &[],
    };
    let vocabulary = BindingVocabulary {
        fields: scenario.fields.keys().cloned().collect(),
        consts: HashSet::new(),
        probability_consts: scenario.probability_consts.clone(),
        metrics: HashSet::from(["global-score".to_owned()]),
    };
    let ceilings = CardinalityCeilings::new(scenario.node_types, HashMap::new());
    let costs = IntrinsicCosts::default();
    let systems = HashSet::from(["demo".to_owned(), "struggle".to_owned()]);
    let error = load_rule(
        "(rule struggle/spark-recognizer :role recognizer :evidence derived \
          :projects-kernel demo/kernel \
          :material-basis \"subject-local projection\" :fuel 64 \
          (bindings \
            (binding current :field social-class/value) \
            (binding global :metric global-score)) \
          (effects (emit EventType/EXCESSIVE_FORCE)))",
        &LoadContext {
            vocabulary: &vocabulary,
            types: &types,
            enums: &scenario.enums,
            const_values: &scenario.consts,
            ceilings: &ceilings,
            intrinsics: &costs,
            systems: &systems,
            vocabulary_registry: None,
            rule_file: "struggle/spark-recognizer.bsl",
        },
    )
    .expect_err("a graph-global metric cannot enter a subject-local projection");
    assert!(
        error.to_string().contains("graph-global :metric"),
        "{error:?}"
    );
}

fn load_mass_probe(source: &str) -> Result<babylon_bsl::LoadedRule, LoadError> {
    let fields = HashMap::from([(
        "social-class/value".to_owned(),
        FieldDecl {
            ty: BslType::Int,
            kind: FieldKind::Extensive,
        },
    )]);
    let vocabulary = BindingVocabulary {
        fields: fields.keys().cloned().collect(),
        consts: HashSet::new(),
        probability_consts: HashSet::new(),
        metrics: HashSet::new(),
    };
    let types = TypeEnv {
        fields,
        exemptions: &[],
    };
    let enums = EnumRegistry::default();
    let const_values = HashMap::new();
    let ceilings = CardinalityCeilings::default();
    let intrinsics = IntrinsicCosts::default();
    let systems = HashSet::from(["demo".to_owned(), "struggle".to_owned()]);
    load_rule(
        source,
        &LoadContext {
            vocabulary: &vocabulary,
            types: &types,
            enums: &enums,
            const_values: &const_values,
            ceilings: &ceilings,
            intrinsics: &intrinsics,
            systems: &systems,
            vocabulary_registry: None,
            rule_file: "tests/mass-probe.bsl",
        },
    )
}

#[test]
fn loader_analysis_retains_mass_literals_in_bindings_scenarios_and_named_preludes() {
    let rule_source = "(rule demo/mass-binding :role mechanic :evidence designed \
        :material-basis \"transient exact mass\" :fuel 64 \
        (bindings (binding weight :expr 1.25m)) \
        (effects (guard #f (emit EventType/X))))";
    let mut loaded_rule = load_mass_probe(rule_source).unwrap();
    let retained_facts = loaded_rule.probability_facts.clone();
    loaded_rule.rule = SExpr::Atom(Atom::Bool(false));
    let rule_analysis = analyze_content_set(&[loaded_rule]).unwrap();
    assert_eq!(rule_analysis.rules[0].nodes, retained_facts.nodes);
    assert_eq!(
        rule_analysis.rules[0].mass_literals,
        retained_facts.mass_literals
    );
    assert_eq!(rule_analysis.rules[0].mass_literals.len(), 1);
    assert_eq!(
        rule_analysis.rules[0].mass_literals[0].mass,
        Mass::from_nanounits(1_250_000_000)
    );
    let (_, rule_spans) = read_all_spanned(rule_source.as_bytes()).unwrap();
    let span = rule_spans
        .span_of(&rule_analysis.rules[0].mass_literals[0].form_path)
        .unwrap();
    assert_eq!(&rule_source[span.start..span.end], "1.25m");

    let prelude_source = "(defconst demo/prelude-mass 2m)\n";
    let scenario_source = "(scenario demo/mass-declarations (defconst demo/scenario-mass 3m))";
    let mut graph = MemoryGraph::new();
    let loaded = load_scenario_with_named_preludes(
        "content/scenario.bscn",
        scenario_source,
        &[NamedDeclarationPreludeV1 {
            source_id: "content/prelude.bsl",
            source: prelude_source,
        }],
        &mut graph,
    )
    .unwrap();
    let [scenario, prelude] = loaded.mass_declarations.as_slice() else {
        panic!("one scenario and one prelude Mass declaration must be retained")
    };
    assert_eq!(prelude.source_id, "content/prelude.bsl");
    assert_eq!(prelude.qname, "demo/prelude-mass");
    assert_eq!(prelude.mass, Mass::from_nanounits(2_000_000_000));
    assert_eq!(scenario.source_id, "content/scenario.bscn");
    assert_eq!(scenario.qname, "demo/scenario-mass");
    assert_eq!(scenario.mass, Mass::from_nanounits(3_000_000_000));
    for (source, declaration) in [(prelude_source, prelude), (scenario_source, scenario)] {
        let (_, spans) = read_all_spanned(source.as_bytes()).unwrap();
        let span = spans.span_of(&declaration.form_path).unwrap();
        assert!(source[span.start..span.end].ends_with('m'));
    }
}

#[test]
fn loader_refuses_mass_in_payload_write_comparison_fold_and_scalar_contexts_at_its_path() {
    let cases = [
        "(rule demo/payload :role recognizer :evidence derived :material-basis \"observation\" :fuel 64 (bindings) (effects (emit EventType/X (x 1m))))",
        "(rule demo/write :role mechanic :evidence derived :material-basis \"write\" :fuel 64 (bindings) (effects (update-node self demo/x (set 1m))))",
        "(rule demo/comparison :role mechanic :evidence derived :material-basis \"comparison\" :fuel 64 (bindings) (when (= 1m 1m)) (effects))",
        "(rule demo/fold :role mechanic :evidence derived :material-basis \"fold\" :fuel 64 (bindings) (when (= (fold sum (nodes NodeType/X) 1m) 0)) (effects))",
        "(rule demo/scalar :role mechanic :evidence derived :material-basis \"scalar\" :fuel 64 (bindings) (when (> (* 1m 1m) 0)) (effects))",
    ];
    for source in cases {
        let error = load_mass_probe(source).expect_err("Mass must be transient");
        let LoadError::Probability(ProbabilityError::InvalidForm { form_path, .. }) = error else {
            panic!("expected a probability-path refusal, got {error:?}");
        };
        let (_, spans) = read_all_spanned(source.as_bytes()).unwrap();
        let span = spans
            .span_of(&form_path)
            .expect("diagnostic path maps to source");
        assert_eq!(&source[span.start..span.end], "1m");
    }
}

#[test]
fn projection_probability_analysis_treats_payload_labels_as_data_and_values_as_expressions() {
    let labels_are_data = "(rule struggle/spark-recognizer :role recognizer :evidence derived \
        :projects-kernel demo/kernel :material-basis \"payload labels are observations\" \
        :fuel 64 (bindings (binding current :field social-class/value)) (effects \
          (emit EventType/EXCESSIVE_FORCE \
            (choose 1) (nodes 2) (quantize-mass 3))))";
    let loaded = load_mass_probe(labels_are_data)
        .expect("verb-shaped payload labels must not become probability forms");
    assert_eq!(
        loaded
            .projection
            .as_ref()
            .map(|projection| projection.sample.as_str()),
        Some("demo/kernel")
    );

    let nonlocal_value = "(rule struggle/spark-recognizer :role recognizer :evidence derived \
        :projects-kernel demo/kernel :material-basis \"payload values are expressions\" \
        :fuel 64 (bindings (binding current :field social-class/value)) (effects \
          (emit EventType/EXCESSIVE_FORCE \
            (choose (fold count (nodes NodeType/X) 1)))))";
    let error = load_mass_probe(nonlocal_value)
        .expect_err("a graph-global query in the payload value is still nonlocal");
    assert!(error.to_string().contains("subject-local"), "{error:?}");
}

#[test]
#[allow(clippy::too_many_lines)]
fn authored_probability_event_payloads_refuse_literals_fields_exprs_and_consts() {
    let literal_source = "(rule demo/literal :role recognizer :evidence derived \
        :material-basis \"event likelihood is derived\" :fuel 64 \
        (bindings) (effects (emit EventType/X (probability 0.5p))))";
    let literal_error = load_mass_probe(literal_source)
        .expect_err("a Probability literal cannot be authored into an event payload");
    let LoadError::Probability(ProbabilityError::InvalidForm { message, form_path }) =
        literal_error
    else {
        panic!("expected a located Probability refusal, got {literal_error:?}");
    };
    assert!(message.contains("event likelihood is derived"));
    let (_, spans) = read_all_spanned(literal_source.as_bytes()).unwrap();
    let span = spans.span_of(&form_path).unwrap();
    assert_eq!(&literal_source[span.start..span.end], "0.5p");

    let mut graph = MemoryGraph::new();
    let scenario = load_scenario(
        "(scenario demo/payload-types \
          (defconst demo/likelihood 0.5p) \
          (defconst demo/intensity 0.5i) \
          (deffield social-class/likelihood probability intensive) \
          (node subject NodeType/SOCIAL_CLASS (social-class/likelihood 0.5p)))",
        &mut graph,
    )
    .unwrap();
    let types = TypeEnv {
        fields: scenario.fields.clone(),
        exemptions: &[],
    };
    let vocabulary = BindingVocabulary {
        fields: scenario.fields.keys().cloned().collect(),
        consts: scenario.consts.keys().cloned().collect(),
        probability_consts: scenario.probability_consts.clone(),
        metrics: HashSet::new(),
    };
    let ceilings = CardinalityCeilings::new(scenario.node_types, HashMap::new());
    let intrinsics = IntrinsicCosts::default();
    let systems = HashSet::from(["demo".to_owned(), "struggle".to_owned()]);
    let context = LoadContext {
        vocabulary: &vocabulary,
        types: &types,
        enums: &scenario.enums,
        const_values: &scenario.consts,
        ceilings: &ceilings,
        intrinsics: &intrinsics,
        systems: &systems,
        vocabulary_registry: None,
        rule_file: "tests/probability-payload.bsl",
    };

    for (binding, value) in [
        (
            "(binding observed :field social-class/likelihood)",
            "observed",
        ),
        ("(binding authored :expr 0.25p)", "authored"),
        ("(binding authored :const demo/likelihood)", "authored"),
    ] {
        let source = format!(
            "(rule demo/projection :role recognizer :evidence derived \
             :projects-kernel demo/kernel :material-basis \"typed probability refusal\" \
             :fuel 64 (bindings {binding}) \
             (effects (emit EventType/X (probability {value}))))"
        );
        let error = load_rule(&source, &context)
            .expect_err("every Probability-typed payload value must refuse");
        assert!(
            error.to_string().contains("event likelihood is derived"),
            "{error:?}"
        );
        let LoadError::Probability(ProbabilityError::InvalidForm { form_path, .. }) = error else {
            panic!("expected a located Probability refusal, got {error:?}");
        };
        let (_, spans) = read_all_spanned(source.as_bytes()).unwrap();
        let span = spans.span_of(&form_path).unwrap();
        assert_eq!(&source[span.start..span.end], value);
    }

    load_rule(
        "(rule struggle/spark-recognizer :role recognizer :evidence derived \
         :material-basis \"same carrier is not the same type\" :fuel 64 \
         (bindings (binding observed :const demo/intensity)) \
         (effects (emit EventType/EXCESSIVE_FORCE (probability observed))))",
        &context,
    )
    .expect("an identically valued Intensity const must not be inferred as Probability");
}

#[test]
fn authored_probability_event_payloads_refuse_probability_in_either_if_branch() {
    for value in ["(if #t 0.5p 0c)", "(if #f 0c 0.5p)"] {
        let source = format!(
            "(rule demo/conditional :role recognizer :evidence derived \
             :material-basis \"event likelihood is derived\" :fuel 64 \
             (bindings) (effects (emit EventType/X (probability {value}))))"
        );
        let error = load_mass_probe(&source)
            .expect_err("either branch capable of yielding Probability must refuse");
        assert!(
            error.to_string().contains("event likelihood is derived"),
            "{error:?}"
        );
    }
}

#[test]
fn mass_cannot_be_declared_as_a_stored_field_or_metric() {
    let mut graph = MemoryGraph::new();
    let field_error = load_scenario(
        "(scenario demo/mass-field (deffield social-class/x mass extensive))",
        &mut graph,
    )
    .expect_err("Mass must not be a stored field type");
    assert!(field_error.to_string().contains("unknown type `mass`"));

    let (metric, _) =
        read("(metric mass-score :type mass :kind intensive (domain :graph) :provider demo)")
            .unwrap();
    let metric_error = MetricRegistry::default()
        .declare(&metric)
        .expect_err("Mass must not be a metric type");
    assert!(metric_error
        .to_string()
        .contains("'mass' is not one of §3.1's type names"));
}

#[test]
fn exact_pushforward_retains_zero_one_and_multiple_favorable_preimages() {
    let rule = kernel_rule(
        "(branch SparkOutcome/EXCESSIVE_FORCE :mass 1m (effects)) \
         (branch SparkOutcome/NO_INCIDENT :mass 3m (effects))",
    );
    let (kernel, _) = compile_rule_probability(
        &rule,
        &[0],
        &contract(RuleRole::Mechanic),
        &spark_enums(),
        &[],
        &HashMap::new(),
    )
    .unwrap();
    let kernel = kernel.unwrap();
    let projection_rule = read(
        r#"(rule demo/projection :role recognizer :evidence derived
  :projects-kernel struggle/spark
  :material-basis "declared events retain empty exact preimages" :fuel 64
  (bindings)
  (effects
    (guard #f (emit EventType/ZERO))
    (emit EventType/ONE)
    (emit EventType/MULTIPLE)))"#,
    )
    .unwrap()
    .0;
    let (_, projection) = compile_rule_probability(
        &projection_rule,
        &[0],
        &contract(RuleRole::Recognizer),
        &spark_enums(),
        &[],
        &HashMap::new(),
    )
    .unwrap();
    let projection = projection.unwrap();
    let likelihoods = forecast_event_likelihoods(
        &kernel,
        &projection,
        &[Mass::from_nanounits(1), Mass::from_nanounits(3)],
        &[
            BranchProjectionV1 {
                outcome: "EXCESSIVE_FORCE".to_owned(),
                event_types: vec!["ONE".to_owned(), "MULTIPLE".to_owned()],
            },
            BranchProjectionV1 {
                outcome: "NO_INCIDENT".to_owned(),
                event_types: vec!["MULTIPLE".to_owned()],
            },
        ],
    )
    .unwrap();
    let by_event: HashMap<_, _> = likelihoods
        .into_iter()
        .map(|likelihood| (likelihood.event_type.clone(), likelihood))
        .collect();
    assert_eq!(by_event.len(), 3);
    assert_eq!(by_event["ZERO"].favorable_outcomes, Vec::<String>::new());
    assert_eq!(by_event["ZERO"].numerator, 0);
    assert_eq!(by_event["ONE"].favorable_outcomes, ["EXCESSIVE_FORCE"]);
    assert_eq!(by_event["ONE"].numerator, TICKET_DENOMINATOR / 4);
    assert_eq!(
        by_event["MULTIPLE"].favorable_outcomes,
        ["EXCESSIVE_FORCE", "NO_INCIDENT"]
    );
    assert_eq!(by_event["MULTIPLE"].numerator, TICKET_DENOMINATOR);
    assert!(by_event
        .values()
        .all(|likelihood| likelihood.denominator == TICKET_DENOMINATOR));
}

#[test]
fn detached_forecast_applies_each_real_branch_and_runs_the_adjacent_recognizer() {
    let scenario_source = r"
(scenario demo/forecast
  (defenum SparkOutcome (EXCESSIVE_FORCE NO_INCIDENT))
  (deffield social-class/value int extensive)
  (node subject NodeType/SOCIAL_CLASS (social-class/value 0)))
";
    let mut graph = MemoryGraph::new();
    let scenario = load_scenario(scenario_source, &mut graph).unwrap();
    let types = TypeEnv {
        fields: scenario.fields.clone(),
        exemptions: &[],
    };
    let vocabulary = BindingVocabulary {
        fields: scenario.fields.keys().cloned().collect(),
        consts: scenario.consts.keys().cloned().collect(),
        probability_consts: scenario.probability_consts.clone(),
        metrics: HashSet::new(),
    };
    let ceilings = CardinalityCeilings::new(scenario.node_types.clone(), HashMap::new());
    let costs = IntrinsicCosts::default();
    let systems = HashSet::from(["control-ratio".to_owned(), "demo".to_owned()]);
    let load = |source: &str, file: &str| {
        load_rule(
            source,
            &LoadContext {
                vocabulary: &vocabulary,
                types: &types,
                enums: &scenario.enums,
                const_values: &scenario.consts,
                ceilings: &ceilings,
                intrinsics: &costs,
                systems: &systems,
                vocabulary_registry: None,
                rule_file: file,
            },
        )
        .unwrap()
    };
    let mechanic = load(
        r#"(rule demo/spark :role mechanic :evidence designed
  :material-basis "bounded material spark" :fuel 64
  (bindings (binding current :field social-class/value))
  (when (= current 0))
  (effects
    (choose :sample struggle/spark :slot 0
      (branch SparkOutcome/EXCESSIVE_FORCE :mass 1m
        (effects (update-node self social-class/value (set 1))))
      (branch SparkOutcome/NO_INCIDENT :mass 3m
        (effects (update-node self social-class/value (set 0)))))
    (emit EventType/MECHANIC_OBSERVATION (value current))))"#,
        "demo/spark.bsl",
    );
    let projection = load(
        r#"(rule control-ratio/c03-crisis :role recognizer :evidence derived
  :projects-kernel struggle/spark
  :material-basis "observes the realized material state" :fuel 64
  (bindings (binding current :field social-class/value))
  (when (= current 1))
  (effects (emit EventType/CONTROL_RATIO_CRISIS (value current))))"#,
        "demo/project.bsl",
    );
    let rules = vec![mechanic, projection];
    let subject = graph.nodes("SOCIAL_CLASS")[0];
    let likelihoods = forecast_detached(
        &rules,
        0,
        &graph,
        subject,
        &ForecastContextV1 {
            types: &types,
            enums: &scenario.enums,
            host: &EmptyIntrinsicHost,
            costs: &costs,
            defines: &scenario.consts,
            tick: 1,
            vocabulary: None,
        },
    )
    .unwrap();
    assert_eq!(likelihoods.len(), 1);
    assert_eq!(likelihoods[0].event_type, "CONTROL_RATIO_CRISIS");
    assert_eq!(likelihoods[0].favorable_outcomes, ["EXCESSIVE_FORCE"]);
    assert_eq!(likelihoods[0].numerator, TICKET_DENOMINATOR / 4);
    assert_eq!(
        graph
            .node_attribute(subject, "social-class/value")
            .unwrap()
            .to_bits(),
        0.0_f64.to_bits()
    );
}

#[test]
fn runtime_never_evaluates_an_unselected_branch_body() {
    let scenario_source = r"
(scenario demo/lazy
  (defenum LazyOutcome (SAFE WOULD_FAIL))
  (deffield social-class/value int extensive)
  (node subject NodeType/SOCIAL_CLASS (social-class/value 0)))
";
    let mut graph = MemoryGraph::new();
    let scenario = load_scenario(scenario_source, &mut graph).unwrap();
    let types = TypeEnv {
        fields: scenario.fields.clone(),
        exemptions: &[],
    };
    let vocabulary = BindingVocabulary {
        fields: scenario.fields.keys().cloned().collect(),
        consts: scenario.consts.keys().cloned().collect(),
        probability_consts: scenario.probability_consts.clone(),
        metrics: HashSet::new(),
    };
    let ceilings = CardinalityCeilings::new(scenario.node_types.clone(), HashMap::new());
    let costs = IntrinsicCosts::default();
    let systems = HashSet::from(["demo".to_owned()]);
    let loaded = load_rule(
        r#"(rule demo/lazy-choice :role mechanic :evidence designed
  :material-basis "only the selected material alternative executes" :fuel 128
  (bindings (binding current :field social-class/value))
  (effects
    (choose :sample demo/lazy-choice :slot 0
      (branch LazyOutcome/SAFE :mass 1m
        (effects (update-node self social-class/value (set 1))))
      (branch LazyOutcome/WOULD_FAIL :mass 0m
        (effects (update-node self social-class/value (set (/ 1 0))))))))"#,
        &LoadContext {
            vocabulary: &vocabulary,
            types: &types,
            enums: &scenario.enums,
            const_values: &scenario.consts,
            ceilings: &ceilings,
            intrinsics: &costs,
            systems: &systems,
            vocabulary_registry: None,
            rule_file: "demo/lazy-choice.bsl",
        },
    )
    .unwrap();
    let resolver = StableElementResolverV1::seal(
        &graph,
        "demo/lazy",
        &scenario.node_content_ids,
        &scenario.hyperedge_content_ids,
    )
    .unwrap();
    let session = ReplaySessionIdV1::try_from("demo/lazy-runtime").unwrap();
    let mut sink = CollectingSink::default();
    let mut writes = CollectingWriteLog::new();
    let outcome = run_tick_observed(
        &loaded,
        &types,
        &scenario.enums,
        &EmptyIntrinsicHost,
        &mut graph,
        &mut sink,
        &costs,
        &scenario.consts,
        1,
        Some(&scenario.node_content_ids),
        RngSeedContext::V2 {
            session: &session,
            seed: ReplaySeed::new(313),
        },
        Some(&resolver),
        None,
        &mut writes,
    )
    .expect("the zero-ticket failing branch must remain lazy");
    assert_eq!(outcome.kernel_realizations.len(), 1);
    assert_eq!(outcome.kernel_realizations[0].selected_outcome, "SAFE");
    let subject = graph.nodes("SOCIAL_CLASS")[0];
    assert_eq!(
        graph
            .node_attribute(subject, "social-class/value")
            .unwrap()
            .to_bits(),
        1.0_f64.to_bits()
    );
}
