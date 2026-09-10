//! Captured observed county-sector context for active material owners.
//! Attribution reads saved source cells and never reopens a current artifact.

use super::ProductionProjectionErrorV1;
use crate::{
    michigan_cohorts::michigan_business_subject_for_owner_v2,
    michigan_content::MichiganContentAdmissionV1,
    michigan_economy::digest_hex,
    michigan_material::{MichiganMaterialCatalogV1, MichiganOwnerSourceV2},
    ArchiveEvidenceClassV1, DesignedProcessAttributionV1, ObservedSectorContextV2,
    ObserverVisibilityV1, ProductionBusinessSubjectV1, ProductionSnapshotV2,
};
use babylon_graph::stable_element::StableElementKeyV1;
use std::collections::{BTreeMap, BTreeSet};

type ContextRows = (
    Vec<ObservedSectorContextV2>,
    Vec<DesignedProcessAttributionV1>,
);

pub(crate) fn attach_observed_context_v1(
    admitted: &MichiganContentAdmissionV1,
    visibility: ObserverVisibilityV1,
    snapshot: &mut ProductionSnapshotV2,
) -> Result<(), ProductionProjectionErrorV1> {
    if visibility != ObserverVisibilityV1::FullObserver {
        snapshot.observed_contexts.clear();
        snapshot.process_attributions.clear();
        return Ok(());
    }
    let (contexts, links) = context_rows(&admitted.catalog, snapshot)?;
    snapshot.observed_contexts = contexts;
    snapshot.process_attributions = links;
    Ok(())
}

fn context_rows(
    catalog: &MichiganMaterialCatalogV1,
    snapshot: &ProductionSnapshotV2,
) -> Result<ContextRows, ProductionProjectionErrorV1> {
    let mut contexts = BTreeMap::new();
    let mut links = Vec::new();
    let mut site_ids = BTreeSet::new();
    for site in catalog.sites() {
        let site_id = digest_hex(&site.id().as_bytes());
        let visible = snapshot
            .sites
            .iter()
            .find(|row| row.id == site_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        if visible.county_geoid != site.county_geoid
            || visible.sector_code != site.sector_code
            || visible.industry_code != site.naics
            || !site_ids.insert(site_id.clone())
        {
            return Err(ProductionProjectionErrorV1::Content);
        }
        let source = catalog
            .owner_source(&site.county_geoid, &site.sector_code)
            .ok_or(ProductionProjectionErrorV1::Content)?;
        let context = checked_context(source, catalog.source_url())?;
        let subject = context.subject.clone();
        if contexts
            .insert(subject.clone(), context.clone())
            .is_some_and(|prior| prior != context)
        {
            return Err(ProductionProjectionErrorV1::Content);
        }
        for process in catalog
            .processes()
            .iter()
            .filter(|row| row.site_key == site.key)
        {
            let process_id = digest_hex(&process.id().as_bytes());
            if !visible.processes.iter().any(|row| row.id == process_id) {
                return Err(ProductionProjectionErrorV1::State);
            }
            links.push(DesignedProcessAttributionV1 {
                process_id,
                site_id: site_id.clone(),
                industry_code: process.industry_code.clone(),
                cohort_subject: subject.clone(),
                scenario_artifact_sha256: digest_hex(&catalog.defines_hash()),
                industry_artifact_sha256: source.industry_artifact_sha256.clone(),
                evidence_class: ArchiveEvidenceClassV1::Designed,
            });
        }
    }
    if snapshot.sites.len() != site_ids.len() || links.len() != catalog.processes().len() {
        return Err(ProductionProjectionErrorV1::Content);
    }
    links.sort_unstable();
    Ok((contexts.into_values().collect(), links))
}

fn checked_context(
    source: &MichiganOwnerSourceV2,
    source_url: &str,
) -> Result<ObservedSectorContextV2, ProductionProjectionErrorV1> {
    let StableElementKeyV1::Node {
        scenario,
        local_name,
    } = michigan_business_subject_for_owner_v2(&source.county_geoid, &source.sector_code)
    else {
        return Err(ProductionProjectionErrorV1::Content);
    };
    Ok(ObservedSectorContextV2 {
        subject: ProductionBusinessSubjectV1 {
            scenario,
            local_name,
        },
        county_geoid: source.county_geoid.clone(),
        sector_code: source.sector_code.clone(),
        sector_title: source.sector_title.clone(),
        vintage: 2024,
        annual_avg_estabs_count: source.annual_avg_estabs_count,
        annual_avg_emplvl: source.annual_avg_emplvl,
        total_annual_wages: source.total_annual_wages,
        annual_avg_wkly_wage: source.annual_avg_wkly_wage,
        source_url: source_url.to_owned(),
        source_file: source.county_source_file.clone(),
        source_sha256: source.county_source_sha256.clone(),
        artifact_sha256: source.sector_artifact_sha256.clone(),
        evidence_class: ArchiveEvidenceClassV1::Observed,
    })
}

#[cfg(test)]
mod tests;
