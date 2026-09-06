use super::*;
use crate::{
    michigan_material::MichiganDeliveryPresetV1,
    production_projection::project_material_observation_v1,
};
use babylon_tick::material_world::MaterialWorldRegisterV2;

fn opening() -> ProductionSnapshotV1 {
    let preset = MichiganDeliveryPresetV1::Standard;
    let state = crate::michigan_material::michigan_material_foundation_v1(preset).unwrap();
    project_material_observation_v1(
        preset,
        &MaterialWorldRegisterV2::try_new(0, state).unwrap(),
        None,
        &[],
    )
    .unwrap()
}

#[test]
fn five_designed_processes_share_four_cited_observed_contexts_without_allocating_labor() {
    let mut snapshot = opening();
    let before = snapshot.clone();
    attach_observed_context_v1(
        MichiganContentPresetV1::CohortsStandardV2
            .admitted()
            .unwrap(),
        ObserverVisibilityV1::FullObserver,
        &mut snapshot,
    )
    .unwrap();
    assert_eq!(snapshot.observed_contexts.len(), 4);
    assert_eq!(snapshot.process_attributions.len(), 5);
    let wayne = snapshot
        .observed_contexts
        .iter()
        .find(|row| row.county_geoid == "26163")
        .unwrap();
    assert_eq!(wayne.subject.local_name, "business-26163-31-33");
    assert_eq!(
        wayne.subject.scenario,
        crate::michigan_cohorts::MICHIGAN_COHORT_SCENARIO_V2
    );
    assert_eq!(wayne.annual_avg_emplvl, Some(89_659));
    assert_eq!(wayne.annual_avg_estabs_count, 1_710);
    let links = snapshot
        .process_attributions
        .iter()
        .filter(|link| link.cohort_subject == wayne.subject)
        .collect::<Vec<_>>();
    assert_eq!(links.len(), 2);
    assert_ne!(links[0].process_id, links[1].process_id);
    assert_ne!(links[0].site_id, links[1].site_id);
    for context in &snapshot.observed_contexts {
        assert_eq!(context.evidence_class, ArchiveEvidenceClassV1::Observed);
        assert_eq!(context.vintage, 2024);
        assert_eq!(context.sector_code, "31-33");
        assert_eq!(context.artifact_sha256, QCEW_SECTORS_ARTIFACT_SHA256_V1);
        assert_eq!(context.source_sha256.len(), 64);
        assert!(context
            .source_file
            .starts_with(&format!("2024.annual {} ", context.county_geoid)));
        assert!(context.source_url.starts_with("https://data.bls.gov/"));
    }
    for link in &snapshot.process_attributions {
        assert_eq!(link.evidence_class, ArchiveEvidenceClassV1::Designed);
        assert!(snapshot
            .observed_contexts
            .iter()
            .any(|context| context.subject == link.cohort_subject));
        assert!(snapshot.sites.iter().any(|site| site.id == link.site_id));
    }
    snapshot.observed_contexts.clear();
    snapshot.process_attributions.clear();
    assert_eq!(snapshot, before, "context does not change output, recipes, stock, observed narrow-industry jobs or labor hours");
}

#[test]
fn source_identity_mismatch_or_missing_subject_refuses_without_partial_publication() {
    let catalog = michigan_material_catalog_v1().unwrap();
    let sectors = michigan_county_sectors_v1().unwrap();
    let sector = sectors
        .rows()
        .iter()
        .find(|row| row.county_geoid() == "26163" && row.sector_code().as_str() == "31-33")
        .unwrap();
    let site = catalog.site("wayne-primary-metal").unwrap();
    let industry = catalog.industry_for_site(site).unwrap();
    for field in 0..3 {
        let mut changed = industry.clone();
        match field {
            0 => changed.source_sha256.replace_range(..1, "x"),
            1 => changed.source_file.push_str(" changed"),
            _ => changed.area_fips = "26099".to_owned(),
        }
        assert_eq!(
            checked_context(sector, &changed, catalog.source_url()),
            Err(ProductionProjectionErrorV1::Content)
        );
    }
    let snapshot = opening();
    let missing = sectors
        .rows()
        .iter()
        .filter(|row| row.county_geoid() != "26163")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        context_rows(catalog, &missing, &snapshot),
        Err(ProductionProjectionErrorV1::Content)
    );
    let mut absent = snapshot.clone();
    absent.sites.pop();
    let unchanged = absent.clone();
    assert!(attach_observed_context_v1(
        MichiganContentPresetV1::CohortsStandardV2
            .admitted()
            .unwrap(),
        ObserverVisibilityV1::FullObserver,
        &mut absent
    )
    .is_err());
    assert_eq!(absent, unchanged);
}

#[test]
fn v1_and_preview_clear_context_and_cannot_borrow_v2_subjects() {
    let mut disclosed = opening();
    let v2 = MichiganContentPresetV1::CohortsStandardV2
        .admitted()
        .unwrap();
    attach_observed_context_v1(v2, ObserverVisibilityV1::FullObserver, &mut disclosed).unwrap();
    for preset in crate::michigan_content::MICHIGAN_CONTENT_PRESETS_V1 {
        for visibility in [
            ObserverVisibilityV1::FullObserver,
            ObserverVisibilityV1::KnownPreview,
        ] {
            let mut candidate = disclosed.clone();
            attach_observed_context_v1(preset.admitted().unwrap(), visibility, &mut candidate)
                .unwrap();
            let allowed = visibility == ObserverVisibilityV1::FullObserver
                && matches!(
                    preset,
                    MichiganContentPresetV1::CohortsStandardV2
                        | MichiganContentPresetV1::CohortsDelayedV2
                        | MichiganContentPresetV1::BundlesStandardV3
                        | MichiganContentPresetV1::BundlesDelayedV3
                );
            assert_eq!(
                candidate.observed_contexts.len(),
                if allowed { 4 } else { 0 }
            );
            assert_eq!(
                candidate.process_attributions.len(),
                if allowed { 5 } else { 0 }
            );
        }
    }
}

#[test]
fn executable_bundles_retain_the_exact_observed_context_without_allocating_jobs() {
    for (previous, current) in [
        (
            MichiganContentPresetV1::CohortsStandardV2,
            MichiganContentPresetV1::BundlesStandardV3,
        ),
        (
            MichiganContentPresetV1::CohortsDelayedV2,
            MichiganContentPresetV1::BundlesDelayedV3,
        ),
    ] {
        let mut old = opening();
        let mut new = old.clone();
        attach_observed_context_v1(
            previous.admitted().unwrap(),
            ObserverVisibilityV1::FullObserver,
            &mut old,
        )
        .unwrap();
        attach_observed_context_v1(
            current.admitted().unwrap(),
            ObserverVisibilityV1::FullObserver,
            &mut new,
        )
        .unwrap();
        assert_eq!(new, old);
    }
}

#[test]
fn source_and_visible_site_order_does_not_change_context_or_duplicate_wayne_jobs() {
    let catalog = michigan_material_catalog_v1().unwrap();
    let sectors = michigan_county_sectors_v1().unwrap();
    let mut snapshot = opening();
    let expected = context_rows(catalog, sectors.rows(), &snapshot).unwrap();
    snapshot.sites.reverse();
    let mut reversed = sectors.rows().to_vec();
    reversed.reverse();
    assert_eq!(
        context_rows(catalog, &reversed, &snapshot).unwrap(),
        expected
    );
    assert_eq!(
        expected
            .0
            .iter()
            .filter(|context| context.county_geoid == "26163")
            .count(),
        1
    );
}
