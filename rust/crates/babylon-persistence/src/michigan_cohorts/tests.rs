use std::collections::{BTreeMap, BTreeSet};

use babylon_bsl::{scenario::load_scenario, structural_verbs::CollectingSink};
use babylon_graph::{stable_state::StableGraphStateV1, substrate::GraphSubstrate};
use babylon_kernel::{
    replay::{ReplaySeed, ReplaySessionIdV1},
    sha256_of,
    tick_content_hash::RefDigestV1,
    ContentDigest,
};
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use babylon_tick::{material_state::MaterialStateV1, replay_session::IdentifiedTickReportV2};

use super::*;
use crate::michigan_economy::{digest_hex, michigan_observer_foundation_v1};
use crate::michigan_sectors::MichiganSectorDisclosureV1;
use crate::{michigan_dynamic_hex_foundation_v1, CampaignFoundationV1, FoundationContentBundleV1};

fn stable_graph() -> StableGraphStateV1 {
    michigan_cohort_foundation_v2()
        .unwrap()
        .0
        .stable_graph_state()
        .unwrap()
}

#[test]
fn full_cohort_source_preserves_the_historical_v1_content_byte_refusal() {
    use crate::semantic_codec::{
        encode_foundation_content, SemanticCodecErrorV1, SemanticRefusalCodeV1,
    };

    let cohorts = michigan_cohorts_v2().unwrap();
    assert_eq!(cohorts.scenario_source().len(), 493_177);
    assert_eq!(
        encode_foundation_content(
            cohorts.scenario_source(),
            None,
            "",
            cohorts.defines_bytes(),
            b"reference manifest",
        ),
        Err(SemanticCodecErrorV1::Refusal(
            SemanticRefusalCodeV1::FieldByteBound
        ))
    );
}

#[test]
fn generated_source_loads_only_observed_cohorts_and_native_sector_hyperedges() {
    let cohorts = michigan_cohorts_v2().unwrap();
    assert_eq!(&build_cohorts().unwrap(), cohorts);
    let mut graph = HypergraphStore::new();
    let loaded = load_scenario(cohorts.scenario_source(), &mut graph).unwrap();
    assert_eq!(loaded.id, MICHIGAN_COHORT_SCENARIO_V2);
    assert_eq!(loaded.node_count, 83 + 1_603);
    assert_eq!(graph.nodes("TERRITORY").len(), 83);
    assert_eq!(graph.nodes("ORGANIZATION").len(), 1_603);
    assert_eq!(
        loaded.edge_count, 0,
        "no inferred employment, ownership, or supply edges"
    );
    assert_eq!(loaded.hyperedge_types.get("ECONOMIC_SECTOR"), Some(&19));
    assert_eq!(loaded.hyperedge_types.len(), 1);
    let actual = stable_graph();
    assert_eq!(actual.scenario_scope(), MICHIGAN_COHORT_SCENARIO_V2);
    assert_eq!(
        actual
            .rows()
            .hyperedges()
            .iter()
            .map(|row| row.2.len())
            .sum::<usize>(),
        1_522
    );
    assert!(actual.rows().edges().is_empty());
}

#[test]
fn classified_memberships_preserve_composite_codes_and_exclude_unclassified_cells() {
    let graph = stable_graph();
    let actual: BTreeMap<_, _> = graph
        .rows()
        .hyperedges()
        .iter()
        .map(|(name, kind, members)| {
            assert_eq!(kind, "ECONOMIC_SECTOR");
            (
                name.as_str(),
                members.iter().map(String::as_str).collect::<BTreeSet<_>>(),
            )
        })
        .collect();
    let sectors = michigan_county_sectors_v1().unwrap();
    let mut memberships = 0;
    for row in sectors.rows() {
        let local_name = michigan_business_local_name_v1(row);
        let count = actual
            .values()
            .filter(|members| members.contains(local_name.as_str()))
            .count();
        if row.sector_code().disposition() == MichiganSectorDispositionV1::Unclassified {
            assert_eq!(count, 0);
            assert_eq!(michigan_sector_subject_v2(row.sector_code()), None);
        } else {
            assert_eq!(count, 1);
            assert!(
                actual[format!("sector-{}", row.sector_code().as_str()).as_str()]
                    .contains(local_name.as_str())
            );
            memberships += count;
        }
        assert_eq!(
            michigan_business_subject_v2(row),
            StableElementKeyV1::Node {
                scenario: MICHIGAN_COHORT_SCENARIO_V2.to_owned(),
                local_name,
            }
        );
    }
    assert_eq!(memberships, 1_522);
    for code in ["31-33", "44-45", "48-49"] {
        assert!(actual.contains_key(format!("sector-{code}").as_str()));
        assert!(graph
            .rows()
            .nodes()
            .iter()
            .any(|(name, _)| name == &format!("business-26163-{code}")));
    }
    assert!(!actual.contains_key("sector-31"));
    assert!(!actual.contains_key("sector-99"));
}

fn integer_bits(value: u64) -> u64 {
    value.to_string().parse::<f64>().unwrap().to_bits()
}

#[test]
fn exact_source_cells_are_not_duplicated_and_suppression_is_absence() {
    let graph = stable_graph();
    let attributes: BTreeMap<_, _> = graph
        .rows()
        .node_f64()
        .iter()
        .map(|(name, field, bits)| ((name.as_str(), field.as_str()), *bits))
        .collect();
    let sectors = michigan_county_sectors_v1().unwrap();
    let mut expected = BTreeMap::new();
    let mut suppressed = 0;
    for row in sectors.rows() {
        let name = michigan_business_local_name_v1(row);
        assert!(row.annual_avg_estabs_count() > 0);
        assert!(expected
            .insert(name.clone(), integer_bits(row.annual_avg_estabs_count()))
            .is_none());
        assert_eq!(
            attributes[&(name.as_str(), "organization/kind")],
            1.0_f64.to_bits()
        );
        let values = [
            Some(row.annual_avg_estabs_count()),
            row.annual_avg_emplvl(),
            row.total_annual_wages(),
            row.annual_avg_wkly_wage(),
        ];
        for ((key, _), value) in BUSINESS_FIELDS.iter().zip(values) {
            assert_eq!(
                attributes
                    .get(&(name.as_str(), format!("organization/{key}").as_str()))
                    .copied(),
                value.map(integer_bits)
            );
        }
        if row.disclosure() == MichiganSectorDisclosureV1::Suppressed {
            suppressed += 1;
        }
    }
    let actual: BTreeMap<_, _> = graph
        .rows()
        .node_f64()
        .iter()
        .filter(|(_, field, _)| field == "organization/qcew-establishments")
        .map(|(name, _, bits)| (name.clone(), *bits))
        .collect();
    assert_eq!(
        actual, expected,
        "one organization observation per source cell; county totals remain separate"
    );
    assert_eq!(suppressed, 416);
    assert_eq!(graph.rows().nodes().len(), 83 + expected.len());
}

fn advance(session: &mut ReplayTickSession<HypergraphStore>) -> IdentifiedTickReportV2 {
    let actions = OrderedPracticeActionBatchV1::empty(
        session.session_identity().clone(),
        u64::try_from(session.completed_tick() + 1).unwrap(),
    )
    .unwrap();
    let mut sink = CollectingSink::default();
    let report = session.advance(&mut sink, &actions).unwrap();
    assert_eq!(report.report().fired, 0);
    assert!(sink.events.is_empty());
    report
}

#[test]
fn full_foundation_and_checkpoint_roundtrip_preserve_memberships_and_identity() {
    let (mut continued, bundle) = michigan_cohort_foundation_v2().unwrap();
    let (mut reopened, twin_bundle) = michigan_cohort_foundation_v2().unwrap();
    let (old, old_bundle) = michigan_observer_foundation_v1().unwrap();
    assert_eq!(
        bundle.reference_bundle_manifest_bytes(),
        old_bundle.reference_bundle_manifest_bytes()
    );
    assert_eq!(continued.material_state(), old.material_state());
    let graph = continued.stable_graph_state().unwrap();
    let county_attributes: Vec<_> = graph
        .rows()
        .node_f64()
        .iter()
        .filter(|(name, _, _)| name.starts_with("county-"))
        .cloned()
        .collect();
    assert_eq!(
        county_attributes,
        old.stable_graph_state().unwrap().rows().node_f64()
    );
    assert_ne!(continued.session_identity(), old.session_identity());
    assert!(bundle.rule_source_bytes().is_empty());
    assert_eq!(
        CampaignFoundationV1::capture_v2(&continued, bundle)
            .unwrap()
            .canonical_bytes(),
        CampaignFoundationV1::capture_v2(&reopened, twin_bundle)
            .unwrap()
            .canonical_bytes()
    );
    let first = advance(&mut continued);
    reopened
        .restore_full_checkpoint(
            1,
            first.result_stable_graph(),
            first.material_state_rows(),
            first.result_registers().canonical_bytes(),
        )
        .unwrap();
    let next = advance(&mut continued);
    let resumed = advance(&mut reopened);
    assert_eq!(next.tick_content_hash(), resumed.tick_content_hash());
    assert_eq!(next.result_stable_graph(), resumed.result_stable_graph());
    assert_eq!(next.result_stable_graph().rows().hyperedges().len(), 19);
    assert_eq!(next.material_state_rows().territories().rows().len(), 83);
}

/// Reproduce the pre-extraction constructor independently to protect V1 saves.
fn original_v1_constructor() -> (
    ReplayTickSession<HypergraphStore>,
    FoundationContentBundleV1,
) {
    let source = michigan_economy_v1().unwrap().scenario_source();
    assert_eq!(source.len(), 21_279);
    assert_eq!(
        digest_hex(&sha256_of(source.as_bytes())),
        "84650f5a5ab3a6913eb125def04b606653217a50eaf67f515f148fb37283d1fa"
    );
    let defines = br#"{"qcew_vintage":2024}"#;
    let content = ContentDigest {
        defines_hash: sha256_of(defines),
        rules_hash: babylon_bsl::rules_hash_of(&[]).unwrap(),
    };
    let material = michigan_dynamic_hex_foundation_v1().unwrap();
    let mut manifest = b"babylon.h3.reference-bundle-composite.v1\0".to_vec();
    manifest.extend_from_slice(&material.base_reference_cohort_digest());
    manifest.extend_from_slice(&material.r8_section_digest());
    assert_eq!(sha256_of(&manifest), material.reference_bundle_digest());
    let session = ReplayTickSession::new(
        source,
        None,
        "",
        HypergraphStore::new(),
        ReplaySessionIdV1::try_from("g4/michigan-observer-v1").unwrap(),
        ReplaySeed::new(319),
        content,
        RefDigestV1::from_bytes(material.reference_bundle_digest()),
        MaterialStateV1::try_new(material).unwrap(),
    )
    .unwrap();
    let bundle = FoundationContentBundleV1::try_new(source, None, "", defines, &manifest).unwrap();
    (session, bundle)
}

#[test]
fn shared_constructor_keeps_existing_v1_foundation_bytes_identical() {
    let (expected, expected_bundle) = original_v1_constructor();
    let (actual, actual_bundle) = michigan_observer_foundation_v1().unwrap();
    assert_eq!(
        CampaignFoundationV1::capture(&actual, actual_bundle)
            .unwrap()
            .canonical_bytes(),
        CampaignFoundationV1::capture(&expected, expected_bundle)
            .unwrap()
            .canonical_bytes()
    );
}
