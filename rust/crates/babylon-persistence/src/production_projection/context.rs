//! Captured observed county-sector context for active material owners.
//! Attribution reads saved source cells and never reopens a current artifact.

use super::ProductionProjectionError;
use crate::{
    michigan_cohorts::michigan_business_subject_for_owner,
    michigan_content::MichiganContentAdmission,
    michigan_economy::digest_hex,
    michigan_material::{MichiganMaterialCatalog, MichiganOwnerSource},
    observer_reader::ObserverVisibility,
    production_observation::DesignedProcessAttribution,
    production_observation::ObservedSectorContext,
    production_observation::ProductionBusinessSubject,
    production_observation::ProductionSnapshot,
    ArchiveEvidenceClass,
};
use babylon_graph::stable_element::StableElementKey;
use std::collections::{BTreeMap, BTreeSet};

type ContextRows = (Vec<ObservedSectorContext>, Vec<DesignedProcessAttribution>);

pub(crate) fn attach_observed_context(
    admitted: &MichiganContentAdmission,
    visibility: ObserverVisibility,
    snapshot: &mut ProductionSnapshot,
) -> Result<(), ProductionProjectionError> {
    if visibility != ObserverVisibility::FullObserver {
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
    catalog: &MichiganMaterialCatalog,
    snapshot: &ProductionSnapshot,
) -> Result<ContextRows, ProductionProjectionError> {
    let mut contexts = BTreeMap::new();
    let mut links = Vec::new();
    let mut site_ids = BTreeSet::new();
    for site in catalog.sites() {
        let site_id = digest_hex(&site.id().as_bytes());
        let visible = snapshot
            .sites
            .iter()
            .find(|row| row.id == site_id)
            .ok_or(ProductionProjectionError::State)?;
        if visible.county_geoid != site.county_geoid
            || visible.sector_code != site.sector_code
            || visible.industry_code != site.naics
            || !site_ids.insert(site_id.clone())
        {
            return Err(ProductionProjectionError::Content);
        }
        let source = catalog
            .owner_source(&site.county_geoid, &site.sector_code)
            .ok_or(ProductionProjectionError::Content)?;
        let context = checked_context(source, catalog.source_url())?;
        let subject = context.subject.clone();
        if contexts
            .insert(subject.clone(), context.clone())
            .is_some_and(|prior| prior != context)
        {
            return Err(ProductionProjectionError::Content);
        }
        for process in catalog
            .processes()
            .iter()
            .filter(|row| row.site_key == site.key)
        {
            let process_id = digest_hex(&process.id().as_bytes());
            if !visible.processes.iter().any(|row| row.id == process_id) {
                return Err(ProductionProjectionError::State);
            }
            links.push(DesignedProcessAttribution {
                process_id,
                site_id: site_id.clone(),
                industry_code: process.industry_code.clone(),
                cohort_subject: subject.clone(),
                scenario_artifact_sha256: digest_hex(&catalog.defines_hash()),
                industry_artifact_sha256: source.industry_artifact_sha256.clone(),
                evidence_class: ArchiveEvidenceClass::Designed,
            });
        }
    }
    if snapshot.sites.len() != site_ids.len() || links.len() != catalog.processes().len() {
        return Err(ProductionProjectionError::Content);
    }
    links.sort_unstable();
    Ok((contexts.into_values().collect(), links))
}

fn checked_context(
    source: &MichiganOwnerSource,
    source_url: &str,
) -> Result<ObservedSectorContext, ProductionProjectionError> {
    let StableElementKey::Node {
        scenario,
        local_name,
    } = michigan_business_subject_for_owner(&source.county_geoid, &source.sector_code)
    else {
        return Err(ProductionProjectionError::Content);
    };
    Ok(ObservedSectorContext {
        subject: ProductionBusinessSubject {
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
        evidence_class: ArchiveEvidenceClass::Observed,
    })
}

#[cfg(test)]
mod tests;
