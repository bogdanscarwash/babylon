//! Shared language-neutral golden-parity vectors for the PER-22 county dossier.
//!
//! The Python oracle (`babylon.projection.county.project_county`) generated
//! the checked expectations in `contracts/county_dossier_parity_v1_vectors.jsonl`;
//! this test rebuilds the producer's pure page-planning path from each
//! vector's committed binary64 inputs and asserts the Rust side reproduces the
//! oracle-pinned display values, grant filtering, and place-link visibility
//! exactly. The contract (`contracts/county_dossier_parity_v1.yaml`) scopes the
//! claim: grid-exact committed values, D2 absence-maximal nulls, canonical
//! `%.6f` statblock formatting with -0.0 -> 0.0.

use babylon_persistence::{
    county_committed_signals_v1, desired_county_projection_v1, filter_granted_county_plans_v1,
    format_county_statblock_value_v1, ArchiveSubjectKindV1, CommittedTerritoryFieldsV1,
    CountyGrantIndexV1, CountyPagePlanV1, CountyPlaceLinkV1, CountySignalV1,
    COMMITTED_TICK_SOURCE_ID_V1, COUNTY_DECISION_QUESTION_V1, COUNTY_MEDIAN_WAGE_GRANT_KEY_V1,
    COUNTY_MEDIAN_WAGE_LABEL_V1, COUNTY_PHI_HOUR_LABEL_V1,
};
use babylon_persistence::{
    michigan_spatial_reference_products_v1, representative_h3_reference_cohort_v1,
};
use serde_json::Value;

const VECTORS: &str = include_str!("../../../../contracts/county_dossier_parity_v1_vectors.jsonl");
const MAX_ROWS: usize = 16;
const MAX_LINE_BYTES: usize = 16_384;
const NEGATIVE_ZERO_BITS: u64 = 0x8000_0000_0000_0000;

fn hex_decode_bits(text: &str) -> u64 {
    assert_eq!(text.len(), 16, "exact 16-char bit hex");
    assert!(
        text.bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "lowercase bit hex"
    );
    u64::from_str_radix(text, 16).expect("lowercase bit hex")
}

fn decode_committed(value: &Value, field: &str) -> Option<f64> {
    match value {
        Value::Null => None,
        Value::String(text) => {
            let bits = hex_decode_bits(text);
            let decoded = f64::from_bits(bits);
            assert!(
                decoded.is_finite(),
                "committed value must be finite: {field}"
            );
            Some(decoded)
        }
        _ => panic!("invalid committed bits: {field}"),
    }
}

fn expected_signal(signal: &CountySignalV1) -> Value {
    serde_json::json!({
        "grant_key": signal.grant_key(),
        "label": signal.label(),
        "value": signal.value(),
    })
}

fn rows() -> Vec<Value> {
    let input = VECTORS.strip_suffix('\n').unwrap_or(VECTORS);
    let mut rows = Vec::with_capacity(MAX_ROWS);
    for (index, line) in input.split('\n').take(MAX_ROWS + 1).enumerate() {
        assert!(index < MAX_ROWS, "bounded vector row count");
        assert!(!line.is_empty() && line.len() <= MAX_LINE_BYTES);
        let row: Value = serde_json::from_str(line).expect("valid bounded vector row");
        assert!(
            row["id"].is_string() && row["kind"] == "parity" && row["data"].is_object(),
            "vector row shape"
        );
        rows.push(row);
    }
    rows
}

fn parse_links(data: &Value) -> Vec<CountyPlaceLinkV1> {
    data["links"]
        .as_array()
        .expect("links array")
        .iter()
        .map(|link| {
            CountyPlaceLinkV1::try_new(
                link["place_geoid"]
                    .as_str()
                    .expect("place geoid text")
                    .to_owned(),
                link["place_name"]
                    .as_str()
                    .expect("place name text")
                    .to_owned(),
            )
            .expect("valid place link")
        })
        .collect()
}

fn parse_grants(data: &Value) -> CountyGrantIndexV1 {
    let county_ref = data["county_geoid"].as_str().expect("county geoid text");
    let mut rows = Vec::new();
    if data["grants"]["county_subject"].as_bool() == Some(true) {
        rows.push((
            ArchiveSubjectKindV1::County,
            county_ref.to_owned(),
            "subject".to_owned(),
        ));
    }
    for key in data["grants"]["field_keys"]
        .as_array()
        .expect("field keys array")
    {
        rows.push((
            ArchiveSubjectKindV1::County,
            county_ref.to_owned(),
            key.as_str().expect("grant key text").to_owned(),
        ));
    }
    for geoid in data["grants"]["place_subjects"]
        .as_array()
        .expect("place subjects array")
    {
        rows.push((
            ArchiveSubjectKindV1::Place,
            geoid.as_str().expect("place geoid text").to_owned(),
            "subject".to_owned(),
        ));
    }
    CountyGrantIndexV1::try_from_rows(rows).expect("valid grant index")
}

fn products_links_for(county_geoid: &str) -> Vec<(String, String)> {
    let cohort = representative_h3_reference_cohort_v1().expect("reference cohort");
    let products = michigan_spatial_reference_products_v1(cohort).expect("reference products");
    let names: std::collections::BTreeMap<&str, &str> = products
        .places()
        .iter()
        .map(|place| (place.place_geoid(), place.name_lsad()))
        .collect();
    let mut links: Vec<(String, String)> = products
        .county_place_land_areas()
        .iter()
        .filter(|row| row.county_geoid() == county_geoid)
        .map(|row| {
            (
                row.place_geoid().to_owned(),
                names[row.place_geoid()].to_owned(),
            )
        })
        .collect();
    links.sort();
    links.dedup();
    links
}

fn assert_plan_signals(row: &Value, fields: &CommittedTerritoryFieldsV1) -> CountyPagePlanV1 {
    let data = &row["data"];
    // The plan carries every committed field; each formatted value must
    // equal the oracle-generated statblock text byte for byte.
    let plan_signals = county_committed_signals_v1(fields).expect("committed signals");
    let expected_plan: Vec<Value> = data["expected"]["plan_signals"]
        .as_array()
        .expect("plan signals array")
        .clone();
    let actual_plan: Vec<Value> = plan_signals.iter().map(expected_signal).collect();
    assert_eq!(actual_plan, expected_plan, "{} plan signals", row["id"]);
    CountyPagePlanV1::try_new(
        data["county_geoid"]
            .as_str()
            .expect("county geoid")
            .to_owned(),
        data["territory_local_name"]
            .as_str()
            .expect("territory local name")
            .to_owned(),
        data["title"].as_str().expect("title").to_owned(),
        plan_signals,
        parse_links(data),
    )
    .expect("valid county page plan")
}

fn assert_visible_signals(row: &Value, plan: &CountyPagePlanV1, grants: &CountyGrantIndexV1) {
    let data = &row["data"];
    // Grant filtering decides which plan signals are visible.
    let county_ref = babylon_persistence::ArchivePageRefV1::try_new(
        ArchiveSubjectKindV1::County,
        data["county_geoid"]
            .as_str()
            .expect("county geoid")
            .to_owned(),
    )
    .expect("valid county ref");
    let visible: Vec<Value> = plan
        .signals()
        .iter()
        .filter(|signal| grants.knows_field(&county_ref, signal.grant_key()))
        .map(expected_signal)
        .collect();
    let expected_visible: Vec<Value> = data["expected"]["signals"]
        .as_array()
        .expect("visible signals array")
        .clone();
    assert_eq!(visible, expected_visible, "{} visible signals", row["id"]);
}

fn assert_projection(row: &Value, plan: &CountyPagePlanV1, grants: &CountyGrantIndexV1) {
    let data = &row["data"];
    // The semantic projection pins labels, values, and the committed
    // provenance identity the renderer cites.
    let projection = desired_county_projection_v1(plan, grants).expect("desired projection");
    assert_eq!(
        projection.title(),
        data["title"].as_str().expect("title"),
        "{}",
        row["id"]
    );
    assert_eq!(
        projection.question(),
        COUNTY_DECISION_QUESTION_V1,
        "{}",
        row["id"]
    );
    let expected_visible: Vec<Value> = data["expected"]["signals"]
        .as_array()
        .expect("visible signals array")
        .clone();
    assert_eq!(
        projection.signals().len(),
        expected_visible.len(),
        "{}",
        row["id"]
    );
    for (signal, expected) in projection.signals().iter().zip(&expected_visible) {
        assert_eq!(
            signal.label(),
            expected["label"].as_str().unwrap(),
            "{}",
            row["id"]
        );
        assert_eq!(
            signal.value(),
            expected["value"].as_str().unwrap(),
            "{}",
            row["id"]
        );
        assert_eq!(
            signal.source_id(),
            COMMITTED_TICK_SOURCE_ID_V1,
            "{}",
            row["id"]
        );
        assert_eq!(
            signal.provenance_name(),
            data["territory_local_name"].as_str().unwrap(),
            "{}",
            row["id"]
        );
    }
    let expected_places: Vec<Value> = data["expected"]["places"]
        .as_array()
        .expect("places array")
        .clone();
    assert_eq!(
        projection.places().len(),
        expected_places.len(),
        "{}",
        row["id"]
    );
    for ((geoid, known_name), expected) in projection.places().iter().zip(&expected_places) {
        assert_eq!(
            geoid,
            expected["place_geoid"].as_str().unwrap(),
            "{}",
            row["id"]
        );
        let expected_name = expected["known_name"].as_str();
        assert_eq!(known_name.as_deref(), expected_name, "{}", row["id"]);
    }
}

#[test]
fn shared_vectors_reproduce_the_oracle_pinned_display_values() {
    let rows = rows();
    let ids: Vec<&str> = rows
        .iter()
        .map(|row| row["id"].as_str().expect("row id"))
        .collect();
    assert_eq!(
        ids,
        [
            "parity-wayne-normal",
            "parity-oakland-zero-wage",
            "parity-wayne-negative-zero",
            "parity-oakland-absent-phi",
            "parity-wayne-field-grant-redacted",
            "parity-wayne-place-redlink",
        ],
        "pinned parity scenario order"
    );
    for row in &rows {
        let data = &row["data"];
        let fields = CommittedTerritoryFieldsV1::try_new(
            decode_committed(&data["committed"]["median_wage_bits"], "median_wage_bits"),
            decode_committed(&data["committed"]["phi_hour_bits"], "phi_hour_bits"),
        )
        .expect("valid committed fields");
        let plan = assert_plan_signals(row, &fields);
        let grants = parse_grants(data);
        assert_visible_signals(row, &plan, &grants);
        assert_projection(row, &plan, &grants);
        // The county subject grant is present in every parity vector, so the
        // fog-safe page filter must keep the plan.
        let granted = filter_granted_county_plans_v1(std::slice::from_ref(&plan), &grants);
        assert_eq!(
            granted.len(),
            1,
            "{} subject grant keeps the page",
            row["id"]
        );
    }
}

#[test]
fn negative_zero_committed_bits_canonicalize_like_the_oracle_boundary() {
    let rows = rows();
    let row = rows
        .iter()
        .find(|row| row["id"] == "parity-wayne-negative-zero")
        .expect("negative-zero vector");
    let bits = row["data"]["committed"]["median_wage_bits"]
        .as_str()
        .expect("committed bits");
    assert_eq!(hex_decode_bits(bits), NEGATIVE_ZERO_BITS);
    // The oracle's SnapToGrid CountyView boundary and the producer both
    // canonicalize -0.0 to +0.0, so the display value is "0.000000" and the
    // vector pins the canonicalized view bits.
    assert_eq!(
        format_county_statblock_value_v1(f64::from_bits(NEGATIVE_ZERO_BITS)),
        Ok("0.000000".to_owned())
    );
    assert_eq!(
        row["data"]["expected"]["county_view"]["median_wage_bits"],
        "0000000000000000"
    );
    let median_signal = row["data"]["expected"]["plan_signals"]
        .as_array()
        .expect("plan signals")
        .iter()
        .find(|signal| signal["grant_key"] == COUNTY_MEDIAN_WAGE_GRANT_KEY_V1)
        .expect("median-wage signal");
    assert_eq!(median_signal["value"], "0.000000");
    assert_eq!(COUNTY_MEDIAN_WAGE_LABEL_V1, "Median wage");
    assert_eq!(COUNTY_PHI_HOUR_LABEL_V1, "Imperial rent Φ");
}

#[test]
fn vector_links_and_titles_match_the_pinned_reference_products() {
    let rows = rows();
    for row in &rows {
        let data = &row["data"];
        let geoid = data["county_geoid"].as_str().expect("county geoid");
        let cohort = representative_h3_reference_cohort_v1().expect("reference cohort");
        let products = michigan_spatial_reference_products_v1(cohort).expect("reference products");
        let county = products
            .counties()
            .iter()
            .find(|county| county.county_geoid() == geoid)
            .expect("county identity in pinned products");
        assert_eq!(
            county.county_name(),
            data["title"].as_str().expect("title"),
            "{} title must be the governed census name",
            row["id"]
        );
        let vector_links: Vec<(String, String)> = data["links"]
            .as_array()
            .expect("links array")
            .iter()
            .map(|link| {
                (
                    link["place_geoid"]
                        .as_str()
                        .expect("place geoid")
                        .to_owned(),
                    link["place_name"].as_str().expect("place name").to_owned(),
                )
            })
            .collect();
        assert_eq!(
            vector_links,
            products_links_for(geoid),
            "{geoid} links must be exactly the pinned overlap set"
        );
    }
}
