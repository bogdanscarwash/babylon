//! Exact successor content encoding; historical V1 content stays immutable.

use babylon_kernel::sha256_of;
use babylon_persistence::{
    michigan_cohorts::{michigan_cohort_foundation_v2, michigan_cohorts_v2},
    michigan_economy::michigan_observer_foundation_v1,
    CampaignFoundationV1, FoundationContentBundle, FoundationContentBundleV1,
    FoundationContentBundleV2, FoundationContentLayout, RustPersistenceRuntimeErrorV2,
};

const SOURCE_BOUND: usize = 1_048_576;
const SMALL_SOURCE: &str = "(scenario fixture/content)";
const SMALL_HEX: &str = "626162796c6f6e2e63616d706169676e2d666f756e646174696f6e2d636f6e74656e742e76320000000002010000001a287363656e6172696f20666978747572652f636f6e74656e74290200030000000004000000027b7d05000000097265666572656e6365";
const SMALL_SHA256: &str = "2215ae1508a00a06bb9f826617165dbcf1ede1ca12d746028a8e70f45d9c0894";

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut result, "{byte:02x}").unwrap();
    }
    result
}

#[test]
fn exact_v2_wire_binds_explicit_version_and_all_five_fields() {
    let v2 =
        FoundationContentBundleV2::try_new(SMALL_SOURCE, None, "", b"{}", b"reference").unwrap();
    assert_eq!(hex(v2.canonical_bytes()), SMALL_HEX);
    assert_eq!(hex(&sha256_of(v2.canonical_bytes())), SMALL_SHA256);
    assert_eq!(v2.scenario_source_bytes(), SMALL_SOURCE.as_bytes());
    assert_eq!(v2.prelude_source_bytes(), None);
    assert!(v2.rule_source_bytes().is_empty());
    assert_eq!(v2.defines_bytes(), b"{}");
    assert_eq!(v2.reference_bundle_manifest_bytes(), b"reference");

    let v1 =
        FoundationContentBundleV1::try_new(SMALL_SOURCE, None, "", b"{}", b"reference").unwrap();
    assert_ne!(v1.canonical_bytes(), v2.canonical_bytes());
    assert_eq!(v1.content_digest(), v2.content_digest());
    assert_eq!(v1.reference_digest(), v2.reference_digest());
    assert_eq!(
        FoundationContentBundle::V1(v1).layout(),
        FoundationContentLayout::V1
    );
    assert_eq!(
        FoundationContentBundle::V2(v2).layout(),
        FoundationContentLayout::V2
    );
}

#[derive(Clone, Copy, Debug)]
enum SourceField {
    Scenario,
    Prelude,
    Rules,
}

fn with_source(
    field: SourceField,
    value: &str,
) -> Result<FoundationContentBundleV2, RustPersistenceRuntimeErrorV2> {
    let (scenario, prelude, rules) = match field {
        SourceField::Scenario => (value, None, ""),
        SourceField::Prelude => (SMALL_SOURCE, Some(value), ""),
        SourceField::Rules => (SMALL_SOURCE, None, value),
    };
    FoundationContentBundleV2::try_new(scenario, prelude, rules, b"{}", b"reference")
}

#[test]
fn all_source_fields_accept_the_exact_bound_and_refuse_the_next_byte() {
    // A comment is valid empty rule content, so this exercises byte bounds
    // independently of rule grammar for all three authored-source fields.
    let exact = format!(";{}", "x".repeat(SOURCE_BOUND - 1));
    let over = format!("{exact}x");
    for field in [
        SourceField::Scenario,
        SourceField::Prelude,
        SourceField::Rules,
    ] {
        let bundle = with_source(field, &exact).unwrap();
        let stored = match field {
            SourceField::Scenario => bundle.scenario_source_bytes(),
            SourceField::Prelude => bundle.prelude_source_bytes().unwrap(),
            SourceField::Rules => bundle.rule_source_bytes(),
        };
        assert_eq!(stored, exact.as_bytes(), "{field:?}");
        assert_eq!(
            with_source(field, &over),
            Err(RustPersistenceRuntimeErrorV2::SemanticCodec),
            "{field:?}"
        );
    }
}

#[test]
fn every_source_field_refuses_nul_without_rewriting_the_source() {
    let source = ";source\0suffix";
    for field in [
        SourceField::Scenario,
        SourceField::Prelude,
        SourceField::Rules,
    ] {
        assert_eq!(
            with_source(field, source),
            Err(RustPersistenceRuntimeErrorV2::SemanticCodec),
            "{field:?}"
        );
    }
    assert_eq!(source.as_bytes(), b";source\0suffix");
}

#[test]
fn absent_empty_and_exact_prelude_bytes_have_distinct_content_identities() {
    let inputs = [None, Some(""), Some("; reference prelude\n")];
    let bundles: Vec<_> = inputs
        .into_iter()
        .map(|prelude| {
            FoundationContentBundleV2::try_new(SMALL_SOURCE, prelude, "", b"{}", b"reference")
                .unwrap()
        })
        .collect();
    for (input, bundle) in inputs.into_iter().zip(&bundles) {
        assert_eq!(bundle.prelude_source_bytes(), input.map(str::as_bytes));
    }
    for left in 0..bundles.len() {
        for right in left + 1..bundles.len() {
            assert_ne!(
                sha256_of(bundles[left].canonical_bytes()),
                sha256_of(bundles[right].canonical_bytes())
            );
        }
    }
}

#[test]
fn full_cohort_source_requires_explicit_v2_and_is_retained_in_the_foundation() {
    let cohorts = michigan_cohorts_v2().unwrap();
    let (session, bundle) = michigan_cohort_foundation_v2().unwrap();
    assert_eq!(
        bundle.scenario_source_bytes(),
        cohorts.scenario_source().as_bytes()
    );
    assert!(bundle.scenario_source_bytes().len() > 65_535);
    assert_eq!(
        FoundationContentBundleV1::try_new(
            cohorts.scenario_source(),
            None,
            "",
            cohorts.defines_bytes(),
            bundle.reference_bundle_manifest_bytes(),
        ),
        Err(RustPersistenceRuntimeErrorV2::SemanticCodec)
    );
    let captured = CampaignFoundationV1::capture_v2(&session, bundle).unwrap();
    assert_eq!(
        captured.content_bundle().layout(),
        FoundationContentLayout::V2
    );
    assert_eq!(
        captured.content_bundle().scenario_source_bytes(),
        cohorts.scenario_source().as_bytes()
    );

    let (old_session, old_bundle) = michigan_observer_foundation_v1().unwrap();
    let old = CampaignFoundationV1::capture(&old_session, old_bundle).unwrap();
    assert_eq!(old.content_bundle().layout(), FoundationContentLayout::V1);
    assert_ne!(old.canonical_bytes(), captured.canonical_bytes());
}

#[test]
fn v2_capture_still_refuses_a_source_that_does_not_reproduce_the_live_graph() {
    let (session, original) = michigan_observer_foundation_v1().unwrap();
    let source = std::str::from_utf8(original.scenario_source_bytes()).unwrap();
    let changed = source.replacen("county-26001", "county-26000", 1);
    assert_ne!(changed, source);
    let wrong = FoundationContentBundleV2::try_new(
        &changed,
        None,
        "",
        original.defines_bytes(),
        original.reference_bundle_manifest_bytes(),
    )
    .unwrap();
    assert_eq!(
        CampaignFoundationV1::capture_v2(&session, wrong),
        Err(RustPersistenceRuntimeErrorV2::FoundationScenarioMismatch)
    );
}
