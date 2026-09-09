use babylon_bsl::probability::{ProbabilityError, TICKET_DENOMINATOR};
use babylon_graph::stable_element::StableElementKeyV1;
use babylon_tick::{
    analyze_content_set_sources, forecast_event_likelihoods,
    forecast_scenario_determined_event_likelihoods,
    forecast_scenario_determined_event_likelihoods_with_kernel_slots,
    kernel_slot::KernelSlotReservationV1, ContentRuleSourceV1, ForecastErrorV1, PrepareError,
};

const SCENARIO: &str = r#"
(scenario struggle/spark-projection-contract
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defenum StruggleSparkOutcome (EXCESSIVE_FORCE NO_INCIDENT))
  (deffield social-class/last-incident int extensive)
  (deffield social-class/earlier-choice int extensive)
  (node worker NodeType/SOCIAL_CLASS
    (social-class/last-incident 0)
    (social-class/earlier-choice 0)))
"#;

const MULTI_CARRIER_SCENARIO: &str = r#"
(scenario struggle/spark-projection-contract
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defenum StruggleSparkOutcome (EXCESSIVE_FORCE NO_INCIDENT))
  (deffield social-class/last-incident int extensive)
  (node worker NodeType/SOCIAL_CLASS
    (social-class/last-incident 0))
  (node second NodeType/SOCIAL_CLASS
    (social-class/last-incident 0)))
"#;

const WRONG_SUBJECT_TYPE_SCENARIO: &str = r#"
(scenario struggle/spark-projection-contract
  (defvocabulary NodeType (SOCIAL_CLASS INSTITUTION))
  (defenum StruggleSparkOutcome (EXCESSIVE_FORCE NO_INCIDENT))
  (deffield social-class/last-incident int extensive)
  (node worker NodeType/SOCIAL_CLASS
    (social-class/last-incident 0))
  (node outsider NodeType/INSTITUTION))
"#;

const PREFIX_DEPENDENT_SCENARIO: &str = r#"
(scenario struggle/spark-projection-contract
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defenum StruggleSparkOutcome (EXCESSIVE_FORCE NO_INCIDENT))
  (deffield social-class/last-incident int extensive)
  (node worker NodeType/SOCIAL_CLASS
    (social-class/last-incident 9)))
"#;

const DETERMINISTIC_PREFIX: &str = r#"
(rule struggle/a-prefix
  :role mechanic
  :evidence derived
  :material-basis "deterministic pre-kernel material state"
  :fuel 64
  (bindings
    (binding last-incident :field social-class/last-incident))
  (effects
    (update-node self social-class/last-incident (set 0))))
"#;

const CALENDAR_DEPENDENT_PREFIX: &str = r#"
(rule struggle/a-calendar-prefix
  :role mechanic
  :evidence derived
  :material-basis "calendar-dependent pre-kernel material state"
  :fuel 64
  (bindings
    (binding now :tick))
  (effects
    (update-node self social-class/last-incident (set now))))
"#;

const EARLIER_KERNEL: &str = r#"
(rule struggle/earlier-a-kernel
  :role mechanic
  :evidence designed
  :material-basis "an earlier finite material alternative"
  :fuel 128
  (bindings
    (binding last-incident :field social-class/last-incident))
  (when (= last-incident 0))
  (effects
    (choose :sample struggle/earlier :slot 0
      (branch StruggleSparkOutcome/EXCESSIVE_FORCE
        :mass 1m
        (effects
          (update-node self social-class/earlier-choice (set 1))))
      (branch StruggleSparkOutcome/NO_INCIDENT
        :mass 1m
        (effects)))))
"#;

const MECHANIC: &str = r#"
(rule struggle/spark-mechanic
  :role mechanic
  :evidence designed
  :material-basis "finite material alternatives over the incident state"
  :fuel 256
  (bindings
    (binding last-incident :field social-class/last-incident))
  (when (= last-incident 0))
  (effects
    (choose :sample struggle/spark :slot 0
      (branch StruggleSparkOutcome/EXCESSIVE_FORCE
        :mass 1m
        (effects
          (update-node self social-class/last-incident (set 1))))
      (branch StruggleSparkOutcome/NO_INCIDENT
        :mass 3m
        (effects)))))
"#;

const PROJECTION: &str = r#"
(rule struggle/spark-recognizer
  :role recognizer
  :evidence derived
  :projects-kernel struggle/spark
  :material-basis "deterministically observes the realized incident state"
  :fuel 128
  (bindings
    (binding last-incident :field social-class/last-incident))
  (when (= last-incident 1))
  (effects
    (emit EventType/EXCESSIVE_FORCE
      (incident-tick last-incident))))
"#;

const CROSS_SAMPLE_SLOTS: [KernelSlotReservationV1<'static>; 2] = [
    KernelSlotReservationV1 {
        ordinal: 0,
        rule: "struggle/spark-mechanic",
        sample: "struggle/spark",
        slot: 0,
    },
    KernelSlotReservationV1 {
        ordinal: 1,
        rule: "struggle/earlier-a-kernel",
        sample: "struggle/earlier",
        slot: 0,
    },
];

fn sources() -> [ContentRuleSourceV1<'static>; 2] {
    [
        ContentRuleSourceV1 {
            source_id: "rules/struggle-spark-mechanic.bsl",
            source: MECHANIC,
        },
        ContentRuleSourceV1 {
            source_id: "rules/struggle-spark-recognizer.bsl",
            source: PROJECTION,
        },
    ]
}

fn scenario(source: &'static str) -> ContentRuleSourceV1<'static> {
    ContentRuleSourceV1 {
        source_id: "scenarios/struggle-spark-projection-contract.bscn",
        source,
    }
}

fn source<'a>(source_id: &'a str, source: &'a str) -> ContentRuleSourceV1<'a> {
    ContentRuleSourceV1 { source_id, source }
}

#[test]
fn named_source_analysis_retains_paths_and_resolved_kernel_projection_linkage() {
    let analysis = analyze_content_set_sources(scenario(SCENARIO), &[], &sources()).unwrap();
    assert_eq!(analysis.rules.len(), 2);
    assert_eq!(
        analysis.rules[0].source_id,
        "rules/struggle-spark-mechanic.bsl"
    );
    assert_eq!(
        analysis.rules[1].source_id,
        "rules/struggle-spark-recognizer.bsl"
    );
    assert_eq!(analysis.links.len(), 1);
    assert_eq!(analysis.links[0].sample, "struggle/spark");
    assert_eq!(analysis.links[0].kernel_rule_id, "struggle/spark-mechanic");
    assert_eq!(
        analysis.links[0].projection_rule_id,
        "struggle/spark-recognizer"
    );
}

#[test]
fn named_source_analysis_enforces_the_permanent_kernel_slot_ledger() {
    let moved_mechanic = MECHANIC.replacen(":slot 0", ":slot 1", 1);
    let moved_sources = [
        source("rules/struggle-spark-mechanic.bsl", &moved_mechanic),
        source("rules/struggle-spark-recognizer.bsl", PROJECTION),
    ];
    let error = analyze_content_set_sources(scenario(SCENARIO), &[], &moved_sources)
        .expect_err("read-only analysis must enforce the executable slot ledger");

    assert!(matches!(error, PrepareError::KernelSlot(_)), "{error:?}");
    assert!(error.to_string().contains("must retain slot 0"), "{error}");
}

#[test]
fn paired_single_carrier_scenario_has_an_exact_recognizer_preimage_measure() {
    let likelihoods = forecast_scenario_determined_event_likelihoods(
        SCENARIO,
        None,
        &sources(),
        "struggle/spark",
    )
    .unwrap();
    assert_eq!(likelihoods.len(), 1);
    assert_eq!(likelihoods[0].event_type, "EXCESSIVE_FORCE");
    assert_eq!(likelihoods[0].favorable_outcomes, ["EXCESSIVE_FORCE"]);
    assert_eq!(likelihoods[0].numerator, TICKET_DENOMINATOR / 4);
    assert_eq!(likelihoods[0].denominator, TICKET_DENOMINATOR);
}

#[test]
fn paired_scenario_forecast_refuses_to_guess_between_multiple_carriers() {
    let error = forecast_scenario_determined_event_likelihoods(
        MULTI_CARRIER_SCENARIO,
        None,
        &sources(),
        "struggle/spark",
    )
    .unwrap_err();
    assert!(
        matches!(error, ForecastErrorV1::NotExactlyEnumerable { .. }),
        "{error:?}"
    );
    assert!(error.to_string().contains("not exactly one"), "{error}");
}

#[test]
fn named_forecast_refuses_a_stable_node_outside_the_mechanic_subject_population() {
    let error = forecast_event_likelihoods(
        WRONG_SUBJECT_TYPE_SCENARIO,
        None,
        &sources(),
        "struggle/spark",
        &StableElementKeyV1::Node {
            scenario: "struggle/spark-projection-contract".to_owned(),
            local_name: "outsider".to_owned(),
        },
        1,
    )
    .expect_err("an institution is not a social-class mechanic carrier");

    assert!(
        matches!(error, ForecastErrorV1::NotExactlyEnumerable { .. }),
        "{error:?}"
    );
    assert!(error.to_string().contains("INSTITUTION"), "{error}");
    assert!(error.to_string().contains("SOCIAL_CLASS"), "{error}");
    assert!(error.to_string().contains("struggle/spark"), "{error}");
}

#[test]
fn forecast_executes_the_resolved_deterministic_prefix_before_enumerating_the_kernel() {
    let mut prefixed = vec![source("rules/struggle-a-prefix.bsl", DETERMINISTIC_PREFIX)];
    prefixed.extend(sources());
    let likelihoods = forecast_scenario_determined_event_likelihoods(
        PREFIX_DEPENDENT_SCENARIO,
        None,
        &prefixed,
        "struggle/spark",
    )
    .unwrap();
    assert_eq!(likelihoods.len(), 1);
    assert_eq!(likelihoods[0].event_type, "EXCESSIVE_FORCE");
    assert_eq!(likelihoods[0].numerator, TICKET_DENOMINATOR / 4);
    assert_eq!(likelihoods[0].denominator, TICKET_DENOMINATOR);
}

#[test]
fn scenario_determined_forecast_refuses_a_calendar_dependent_prefix() {
    let mut prefixed = vec![source(
        "rules/struggle-a-calendar-prefix.bsl",
        CALENDAR_DEPENDENT_PREFIX,
    )];
    prefixed.extend(sources());
    let error =
        forecast_scenario_determined_event_likelihoods(SCENARIO, None, &prefixed, "struggle/spark")
            .unwrap_err();
    assert!(matches!(
        error,
        ForecastErrorV1::NotExactlyEnumerable { .. }
    ));
    assert!(error.to_string().contains("forecast prefix"), "{error}");
    assert!(error.to_string().contains("forecast tick"), "{error}");
}

#[test]
fn forecast_refuses_cross_sample_enumeration_after_an_earlier_finite_kernel() {
    let mut multi_sample = vec![source(
        "rules/struggle-earlier-a-kernel.bsl",
        EARLIER_KERNEL,
    )];
    multi_sample.extend(sources());
    let error = forecast_scenario_determined_event_likelihoods_with_kernel_slots(
        SCENARIO,
        None,
        &multi_sample,
        &CROSS_SAMPLE_SLOTS,
        "struggle/spark",
    )
    .unwrap_err();
    assert!(
        matches!(error, ForecastErrorV1::NotExactlyEnumerable { .. }),
        "{error:?}"
    );
    assert!(error.to_string().contains("cross-sample"), "{error}");
    assert!(error.to_string().contains("struggle/earlier"), "{error}");
}

#[test]
fn resolved_nonadjacency_is_a_typed_rule_owned_probability_refusal() {
    let obstruction = ContentRuleSourceV1 {
        source_id: "rules/struggle-spark-obstruction.bsl",
        source: r#"
(rule struggle/spark-obstruction
  :role mechanic
  :evidence derived
  :material-basis "schedule adjacency refusal fixture"
  :fuel 16
  (bindings
    (binding last-incident :field social-class/last-incident))
  (effects
    (emit EventType/OBSTRUCTION (value last-incident))))
"#,
    };
    let mut separated = sources().to_vec();
    separated.push(obstruction);
    let error = analyze_content_set_sources(scenario(SCENARIO), &[], &separated).unwrap_err();
    let PrepareError::Probability {
        rule_id: Some(rule_id),
        error: ProbabilityError::ProjectionNotAdjacent {
            form_path, sample, ..
        },
    } = error
    else {
        panic!("expected a located projection refusal, got {error:?}");
    };
    assert_eq!(rule_id, "struggle/spark-recognizer");
    assert_eq!(sample, "struggle/spark");
    assert!(!form_path.is_empty());
}
