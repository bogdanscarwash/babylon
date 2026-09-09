use babylon_bsl::probability::TICKET_DENOMINATOR;
use babylon_graph::stable_element::StableElementKeyV1;
use babylon_tick::{analyze_content_set_sources, forecast_event_likelihoods, ContentRuleSourceV1};

const SCENARIO: &str = include_str!("../content/scenarios/struggle-spark-conformance.bscn");
const RULES: &str = include_str!("../content/rules/struggle-spark.bsl");

fn sources() -> [ContentRuleSourceV1<'static>; 1] {
    [ContentRuleSourceV1 {
        source_id: "rules/struggle-spark.bsl",
        source: RULES,
    }]
}

fn scenario() -> ContentRuleSourceV1<'static> {
    ContentRuleSourceV1 {
        source_id: "scenarios/struggle-spark-conformance.bscn",
        source: SCENARIO,
    }
}

#[test]
fn pilot_loads_as_one_adjacent_typed_kernel_projection_pair() {
    let analysis = analyze_content_set_sources(scenario(), &[], &sources()).unwrap();
    assert_eq!(analysis.rules.len(), 2);
    assert_eq!(analysis.rules[0].rule_id, "struggle/spark-mechanic");
    assert_eq!(analysis.rules[1].rule_id, "struggle/spark-recognizer");
    assert_eq!(analysis.links.len(), 1);
    assert_eq!(analysis.links[0].sample, "struggle/spark");
}

#[test]
fn pilot_likelihood_is_the_exact_recognizer_preimage_not_authored_payload() {
    let likelihoods = forecast_event_likelihoods(
        SCENARIO,
        None,
        &sources(),
        "struggle/spark",
        &StableElementKeyV1::Node {
            scenario: "struggle/spark-conformance".to_owned(),
            local_name: "workers".to_owned(),
        },
        1,
    )
    .unwrap();
    assert_eq!(likelihoods.len(), 1);
    assert_eq!(likelihoods[0].event_type, "EXCESSIVE_FORCE");
    assert_eq!(likelihoods[0].favorable_outcomes, ["EXCESSIVE_FORCE"]);
    assert_eq!(likelihoods[0].numerator, TICKET_DENOMINATOR / 4);
    assert_eq!(likelihoods[0].denominator, TICKET_DENOMINATOR);
    assert!(!RULES.contains("(probability "));
}
