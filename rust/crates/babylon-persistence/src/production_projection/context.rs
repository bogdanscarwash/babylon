//! Designed process attribution to immutable observed manufacturing context.
//! These links neither allocate labor nor alter a material relationship.

use std::collections::{BTreeMap, BTreeSet};

use babylon_graph::stable_element::StableElementKeyV1;

use super::ProductionProjectionErrorV1;
use crate::{
    michigan_cohorts::michigan_business_subject_v2,
    michigan_content::{MichiganContentAdmissionV1, MichiganContentPresetV1},
    michigan_economy::digest_hex,
    michigan_material::{
        michigan_material_catalog_v1, MichiganIndustryBaselineRowV1, MichiganMaterialCatalogV1,
        MICHIGAN_INDUSTRY_BASELINE_SHA256_V1, MICHIGAN_MATERIAL_SCENARIO_SHA256_V1,
    },
    michigan_sectors::{
        michigan_county_sectors_v1, MichiganCountySectorV1, QCEW_SECTORS_ARTIFACT_SHA256_V1,
        QCEW_SECTORS_VINTAGE_V1,
    },
    ArchiveEvidenceClassV1, DesignedProcessAttributionV1, ObservedManufacturingContextV1,
    ObserverVisibilityV1, ProductionBusinessSubjectV1, ProductionSnapshotV1,
};

struct AttributionBinding {
    process: &'static str,
    site: &'static str,
    county: &'static str,
    industry: &'static str,
}
const BINDINGS: [AttributionBinding; 5] = [
    AttributionBinding {
        process: "sheet-rolling",
        site: "wayne-primary-metal",
        county: "26163",
        industry: "331",
    },
    AttributionBinding {
        process: "panel-forming",
        site: "macomb-fabricated-metal",
        county: "26099",
        industry: "332",
    },
    AttributionBinding {
        process: "subassembly-making",
        site: "wayne-vehicle-parts",
        county: "26163",
        industry: "3363",
    },
    AttributionBinding {
        process: "meal-milling",
        site: "washtenaw-food",
        county: "26161",
        industry: "311",
    },
    AttributionBinding {
        process: "meal-packaging",
        site: "oakland-food",
        county: "26125",
        industry: "311",
    },
];

type ContextRows = (
    Vec<ObservedManufacturingContextV1>,
    Vec<DesignedProcessAttributionV1>,
);

/// Called only after the snapshot's shared content/graph admission. Older graphs
/// have no cohort subjects; preview grants do not authorize organization facts.
pub(crate) fn attach_observed_context_v1(
    admitted: &MichiganContentAdmissionV1,
    visibility: ObserverVisibilityV1,
    snapshot: &mut ProductionSnapshotV1,
) -> Result<(), ProductionProjectionErrorV1> {
    if visibility != ObserverVisibilityV1::FullObserver
        || !matches!(
            admitted.preset(),
            MichiganContentPresetV1::CohortsStandardV2
                | MichiganContentPresetV1::CohortsDelayedV2
                | MichiganContentPresetV1::BundlesStandardV3
                | MichiganContentPresetV1::BundlesDelayedV3
        )
    {
        snapshot.observed_contexts.clear();
        snapshot.process_attributions.clear();
        return Ok(());
    }
    let catalog =
        michigan_material_catalog_v1().map_err(|_| ProductionProjectionErrorV1::Content)?;
    let sectors = michigan_county_sectors_v1().map_err(|_| ProductionProjectionErrorV1::Content)?;
    let (contexts, links) = context_rows(catalog, sectors.rows(), snapshot)?;
    snapshot.observed_contexts = contexts;
    snapshot.process_attributions = links;
    Ok(())
}

fn context_rows(
    catalog: &MichiganMaterialCatalogV1,
    sectors: &[MichiganCountySectorV1],
    snapshot: &ProductionSnapshotV1,
) -> Result<ContextRows, ProductionProjectionErrorV1> {
    let mut contexts = BTreeMap::new();
    let mut links = Vec::new();
    let mut site_ids = BTreeSet::new();
    for binding in &BINDINGS {
        let site = catalog
            .site(binding.site)
            .ok_or(ProductionProjectionErrorV1::Content)?;
        let process = catalog
            .processes()
            .iter()
            .find(|row| row.key == binding.process)
            .ok_or(ProductionProjectionErrorV1::Content)?;
        let site_id = digest_hex(&site.id().as_bytes());
        let visible = snapshot
            .sites
            .iter()
            .find(|row| row.id == site_id)
            .ok_or(ProductionProjectionErrorV1::State)?;
        if process.site_id() != site.id()
            || site.county_geoid != binding.county
            || site.naics != binding.industry
            || visible.county_geoid != binding.county
            || visible.industry_code != binding.industry
            || !site_ids.insert(site_id.clone())
        {
            return Err(ProductionProjectionErrorV1::Content);
        }
        let sector = sectors
            .iter()
            .find(|row| {
                row.county_geoid() == binding.county && row.sector_code().as_str() == "31-33"
            })
            .ok_or(ProductionProjectionErrorV1::Content)?;
        let industry = catalog
            .industry_for_site(site)
            .ok_or(ProductionProjectionErrorV1::Content)?;
        let context = checked_context(sector, industry, catalog.source_url())?;
        let subject = context.subject.clone();
        if let Some(previous) = contexts.insert(subject.clone(), context.clone()) {
            if previous != context {
                return Err(ProductionProjectionErrorV1::Content);
            }
        }
        links.push(DesignedProcessAttributionV1 {
            process_id: digest_hex(&process.id().as_bytes()),
            site_id,
            industry_code: binding.industry.to_owned(),
            cohort_subject: subject,
            scenario_artifact_sha256: MICHIGAN_MATERIAL_SCENARIO_SHA256_V1.to_owned(),
            industry_artifact_sha256: MICHIGAN_INDUSTRY_BASELINE_SHA256_V1.to_owned(),
            evidence_class: ArchiveEvidenceClassV1::Designed,
        });
    }
    if contexts.len() != 4 || links.len() != 5 || snapshot.sites.len() != site_ids.len() {
        return Err(ProductionProjectionErrorV1::Content);
    }
    links.sort_unstable();
    Ok((contexts.into_values().collect(), links))
}

fn checked_context(
    sector: &MichiganCountySectorV1,
    industry: &MichiganIndustryBaselineRowV1,
    source_url: &str,
) -> Result<ObservedManufacturingContextV1, ProductionProjectionErrorV1> {
    if sector.county_geoid() != industry.area_fips
        || sector.sector_code().as_str() != "31-33"
        || sector.source_file() != industry.source_file
        || sector.source_sha256() != industry.source_sha256
    {
        return Err(ProductionProjectionErrorV1::Content);
    }
    let StableElementKeyV1::Node {
        scenario,
        local_name,
    } = michigan_business_subject_v2(sector)
    else {
        return Err(ProductionProjectionErrorV1::Content);
    };
    Ok(ObservedManufacturingContextV1 {
        subject: ProductionBusinessSubjectV1 {
            scenario,
            local_name,
        },
        county_geoid: sector.county_geoid().to_owned(),
        sector_code: sector.sector_code().as_str().to_owned(),
        sector_title: sector.sector_title().to_owned(),
        vintage: QCEW_SECTORS_VINTAGE_V1,
        annual_avg_estabs_count: sector.annual_avg_estabs_count(),
        annual_avg_emplvl: sector.annual_avg_emplvl(),
        total_annual_wages: sector.total_annual_wages(),
        annual_avg_wkly_wage: sector.annual_avg_wkly_wage(),
        source_url: source_url.to_owned(),
        source_file: sector.source_file().to_owned(),
        source_sha256: sector.source_sha256().to_owned(),
        artifact_sha256: QCEW_SECTORS_ARTIFACT_SHA256_V1.to_owned(),
        evidence_class: ArchiveEvidenceClassV1::Observed,
    })
}

#[cfg(test)]
mod tests;
