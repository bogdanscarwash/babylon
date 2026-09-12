use super::*;
use crate::michigan_content::MichiganContentPreset;
use crate::{
    michigan_material::MichiganDeliveryPreset, production_projection::project_material_observation,
};

fn opening() -> ProductionSnapshot {
    let preset = MichiganDeliveryPreset::Standard;
    let foundation = MichiganContentPreset::new_campaign(preset)
        .create_foundation(&crate::test_support::catalog())
        .unwrap();
    project_material_observation(
        &crate::test_support::catalog(),
        preset,
        foundation.initial_register(),
        None,
        &[],
    )
    .unwrap()
}

#[test]
fn five_designed_processes_share_four_cited_observed_contexts_without_allocating_labor() {
    let mut snapshot = opening();
    let before = snapshot.clone();
    attach_observed_context(
        &MichiganContentPreset::FourWeekStandard
            .admitted(&crate::test_support::catalog())
            .unwrap(),
        ObserverVisibility::FullObserver,
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
        crate::michigan_cohorts::MICHIGAN_COHORT_SCENARIO
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
        assert_eq!(context.evidence_class, ArchiveEvidenceClass::Observed);
        assert_eq!(context.vintage, 2024);
        assert_eq!(context.sector_code, "31-33");
        assert_eq!(
            context.artifact_sha256,
            crate::test_support::catalog()
                .owner_source(&context.county_geoid, &context.sector_code)
                .unwrap()
                .sector_artifact_sha256
        );
        assert_eq!(context.source_sha256.len(), 64);
        assert!(context
            .source_file
            .starts_with(&format!("2024.annual {} ", context.county_geoid)));
        assert!(context.source_url.starts_with("https://data.bls.gov/"));
    }
    for link in &snapshot.process_attributions {
        assert_eq!(link.evidence_class, ArchiveEvidenceClass::Designed);
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
fn missing_or_misidentified_visible_owner_refuses_without_partial_publication() {
    let catalog = crate::test_support::catalog();
    let admitted = MichiganContentPreset::FourWeekStandard
        .admitted(&catalog)
        .unwrap();
    for mutate in [
        |snapshot: &mut ProductionSnapshot| {
            snapshot.sites.pop();
        },
        |snapshot: &mut ProductionSnapshot| {
            snapshot.sites[0].sector_code = "11".to_owned();
        },
        |snapshot: &mut ProductionSnapshot| {
            snapshot.sites[0].processes.clear();
        },
    ] {
        let mut snapshot = opening();
        mutate(&mut snapshot);
        let unchanged = snapshot.clone();
        assert!(attach_observed_context(
            &admitted,
            ObserverVisibility::FullObserver,
            &mut snapshot
        )
        .is_err());
        assert_eq!(snapshot, unchanged);
    }
}

#[test]
fn captured_disclosure_retains_absent_source_cells_instead_of_inventing_zero() {
    let catalog = crate::test_support::catalog();
    let mut source = catalog.owners()[0].clone();
    source.annual_avg_emplvl = None;
    source.total_annual_wages = None;
    source.annual_avg_wkly_wage = None;
    let absent = checked_context(&source, catalog.source_url()).unwrap();
    assert_eq!(absent.annual_avg_emplvl, None);
    source.annual_avg_emplvl = Some(0);
    let zero = checked_context(&source, catalog.source_url()).unwrap();
    assert_eq!(zero.annual_avg_emplvl, Some(0));
    assert_ne!(absent, zero);
}

#[test]
fn preview_clears_context_for_both_current_staffed_presets() {
    let mut disclosed = opening();
    let admitted = MichiganContentPreset::FourWeekStandard
        .admitted(&crate::test_support::catalog())
        .unwrap();
    attach_observed_context(&admitted, ObserverVisibility::FullObserver, &mut disclosed).unwrap();
    for preset in crate::michigan_content::MICHIGAN_CONTENT_PRESETS
        .into_iter()
        .filter(|preset| !preset.delivery().is_statewide())
    {
        for visibility in [
            ObserverVisibility::FullObserver,
            ObserverVisibility::KnownPreview,
        ] {
            let mut candidate = disclosed.clone();
            attach_observed_context(
                &preset.admitted(&crate::test_support::catalog()).unwrap(),
                visibility,
                &mut candidate,
            )
            .unwrap();
            let allowed = visibility == ObserverVisibility::FullObserver;
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
fn delivery_presets_share_observed_context_without_assigning_jobs() {
    let mut standard = opening();
    let mut delayed = standard.clone();
    for (preset, snapshot) in [
        (MichiganContentPreset::FourWeekStandard, &mut standard),
        (MichiganContentPreset::FourWeekDelayed, &mut delayed),
    ] {
        attach_observed_context(
            &preset.admitted(&crate::test_support::catalog()).unwrap(),
            ObserverVisibility::FullObserver,
            snapshot,
        )
        .unwrap();
    }
    assert_eq!(standard.observed_contexts, delayed.observed_contexts);
    assert_eq!(
        standard.process_attributions.len(),
        delayed.process_attributions.len()
    );
    assert_ne!(
        standard.process_attributions[0].scenario_artifact_sha256,
        delayed.process_attributions[0].scenario_artifact_sha256
    );
}

#[test]
fn source_and_visible_site_order_does_not_change_context_or_duplicate_wayne_jobs() {
    let catalog = crate::test_support::catalog();
    let mut snapshot = opening();
    let expected = context_rows(&catalog, &snapshot).unwrap();
    snapshot.sites.reverse();
    assert_eq!(context_rows(&catalog, &snapshot).unwrap(), expected);
    assert_eq!(
        expected
            .0
            .iter()
            .filter(|context| context.county_geoid == "26163")
            .count(),
        1
    );
}
