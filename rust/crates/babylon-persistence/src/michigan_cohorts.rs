//! Observed county-sector BUSINESS aggregates in a separate immutable foundation.
//!
//! A node represents one source cell, not a named enterprise or allocated workers.
//! Sector hyperedges classify those aggregates; they imply no trade, employment,
//! organization membership, physical production, or ownership relation.

use std::{collections::BTreeMap, fmt::Write as _, sync::OnceLock};

use babylon_graph::{hypergraph_store::HypergraphStore, stable_element::StableElementKey};
use babylon_tick::replay_session::ReplayTickSession;

use crate::{
    michigan_economy::{
        append_county_observations, michigan_economy, observer_foundation_from_source,
        MichiganEconomyError, QCEW_ECONOMICS_ARTIFACT_SHA256,
    },
    michigan_sectors::{
        michigan_county_sectors, MichiganCountySector, MichiganCountySectors, MichiganSectorCode,
        MichiganSectorDisposition, MichiganSectorsError, QCEW_SECTORS_ARTIFACT_SHA256,
        QCEW_SECTORS_SEMANTIC_SHA256,
    },
    FoundationContentBundle,
};

pub const MICHIGAN_COHORT_SCENARIO: &str = "production/michigan-observer-v2";
pub const MICHIGAN_COHORT_SESSION: &str = "g4/michigan-observer-v2";

const BUSINESS_FIELDS: [(&str, &str); 4] = [
    ("qcew-establishments", "extensive"),
    ("qcew-employment", "extensive"),
    ("qcew-total-annual-wages", "extensive"),
    ("qcew-average-weekly-wage", "intensive"),
];

/// Deterministic source composition with immutable source identities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MichiganCohorts {
    scenario_source: String,
    defines: Vec<u8>,
}
impl MichiganCohorts {
    #[must_use]
    pub fn scenario_source(&self) -> &str {
        &self.scenario_source
    }
    #[must_use]
    pub fn defines_bytes(&self) -> &[u8] {
        &self.defines
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganCohortsError {
    Economy(MichiganEconomyError),
    Sectors(MichiganSectorsError),
    NumericRepresentation,
    Coverage,
}
impl std::fmt::Display for MichiganCohortsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Michigan cohort foundation refused: {self:?}")
    }
}
impl std::error::Error for MichiganCohortsError {}

/// Exact composite NAICS code remains part of the stable local subject name.
#[must_use]
pub fn michigan_business_local_name(row: &MichiganCountySector) -> String {
    format!(
        "business-{}-{}",
        row.county_geoid(),
        row.sector_code().as_str()
    )
}

#[must_use]
pub fn michigan_business_subject(row: &MichiganCountySector) -> StableElementKey {
    michigan_business_subject_for_owner(row.county_geoid(), row.sector_code().as_str())
}

/// Construct the same subject from captured owner identity without reading current sources.
#[must_use]
pub fn michigan_business_subject_for_owner(
    county_geoid: &str,
    sector_code: &str,
) -> StableElementKey {
    StableElementKey::Node {
        scenario: MICHIGAN_COHORT_SCENARIO.to_owned(),
        local_name: format!("business-{county_geoid}-{sector_code}"),
    }
}

/// Code 99 has no classified sector subject or membership.
#[must_use]
pub fn michigan_sector_subject(code: MichiganSectorCode) -> Option<StableElementKey> {
    (code.disposition() == MichiganSectorDisposition::Classified).then(|| {
        StableElementKey::Hyperedge {
            scenario: MICHIGAN_COHORT_SCENARIO.to_owned(),
            local_name: format!("sector-{}", code.as_str()),
        }
    })
}

fn append_business(
    source: &mut String,
    row: &MichiganCountySector,
) -> Result<(), MichiganCohortsError> {
    writeln!(source, "  (node {} NodeType/ORGANIZATION\n    (organization/kind OrgKind/BUSINESS)\n    (organization/county-fips {})", michigan_business_local_name(row), row.county_geoid()).expect("String write");
    let values = [
        Some(row.annual_avg_estabs_count()),
        row.annual_avg_emplvl(),
        row.total_annual_wages(),
        row.annual_avg_wkly_wage(),
    ];
    for ((field, _), value) in BUSINESS_FIELDS.iter().zip(values) {
        if let Some(value) = value {
            // The graph stores these int fields through binary64. Refuse a
            // value that could lose its exact public-record integer identity.
            if value > 9_007_199_254_740_992 {
                return Err(MichiganCohortsError::NumericRepresentation);
            }
            writeln!(source, "    (organization/{field} {value})").expect("String write");
        }
    }
    source.push_str("  )\n");
    Ok(())
}

fn append_sectors(
    source: &mut String,
    sectors: &MichiganCountySectors,
) -> Result<(), MichiganCohortsError> {
    let mut memberships = BTreeMap::<MichiganSectorCode, Vec<String>>::new();
    for row in sectors.rows() {
        if row.sector_code().disposition() == MichiganSectorDisposition::Classified {
            memberships
                .entry(row.sector_code())
                .or_default()
                .push(michigan_business_local_name(row));
        }
    }
    if memberships.len() != 19 || memberships.values().map(Vec::len).sum::<usize>() != 1_522 {
        return Err(MichiganCohortsError::Coverage);
    }
    for (code, members) in memberships {
        write!(
            source,
            "  (hyperedge sector-{} HyperedgeType/ECONOMIC_SECTOR (members",
            code.as_str()
        )
        .expect("String write");
        for member in members {
            write!(source, " {member}").expect("String write");
        }
        source.push_str("))\n");
    }
    Ok(())
}

fn build_cohorts() -> Result<MichiganCohorts, MichiganCohortsError> {
    build_cohorts_with_workforce(&[])
}

fn build_cohorts_with_workforce(
    workforce: &[crate::michigan_material::MichiganWorkforceSeed],
) -> Result<MichiganCohorts, MichiganCohortsError> {
    let economy = michigan_economy().map_err(MichiganCohortsError::Economy)?;
    let sectors = michigan_county_sectors().map_err(MichiganCohortsError::Sectors)?;
    if sectors.rows().len() != 1_603 {
        return Err(MichiganCohortsError::Coverage);
    }
    let workforce_type = if workforce.is_empty() {
        ""
    } else {
        " SOCIAL_CLASS"
    };
    let mut source = format!("(scenario {MICHIGAN_COHORT_SCENARIO}\n  (defvocabulary NodeType (TERRITORY ORGANIZATION{workforce_type}))\n  (defvocabulary HyperedgeType (ECONOMIC_SECTOR))\n  (deffield territory/county-fips int extensive)\n");
    append_county_observations(&mut source, economy.counties());
    source.push_str("  (defenum OrgKind (STATE_APPARATUS BUSINESS POLITICAL_FACTION CIVIL_SOCIETY))\n  (deffield organization/kind enum OrgKind)\n  (deffield organization/county-fips int intensive)\n");
    for (field, quantity) in BUSINESS_FIELDS {
        writeln!(
            &mut source,
            "  (deffield organization/{field} int {quantity})"
        )
        .expect("String write");
    }
    for row in sectors.rows() {
        append_business(&mut source, row)?;
    }
    append_sectors(&mut source, sectors)?;
    if !workforce.is_empty() {
        for field in babylon_tick::material_staffing::STAFFING_FIELDS {
            writeln!(&mut source, "  (deffield {field} int extensive)").expect("String write");
        }
        for seed in workforce {
            writeln!(&mut source, "  (node {} NodeType/SOCIAL_CLASS (social-class/employed-population {}) (social-class/reserve-population {}) (social-class/previous-unretained-labor-hours {}))", seed.local_name(), seed.employed, seed.reserve, seed.previous_unretained_hours).expect("String write");
        }
    }
    source.push_str(")\n");
    let defines = format!("{{\"qcew_vintage\":2024,\"county_artifact_sha256\":\"{QCEW_ECONOMICS_ARTIFACT_SHA256}\",\"sector_artifact_sha256\":\"{QCEW_SECTORS_ARTIFACT_SHA256}\",\"sector_semantic_sha256\":\"{QCEW_SECTORS_SEMANTIC_SHA256}\",\"cohort_composition_version\":2}}").into_bytes();
    Ok(MichiganCohorts {
        scenario_source: source,
        defines,
    })
}

/// Material-only graph composition. The observed foundation has no workforce seeds.
pub(crate) fn michigan_staffed_scenario(
    workforce: &[crate::michigan_material::MichiganWorkforceSeed],
) -> Result<String, MichiganCohortsError> {
    Ok(build_cohorts_with_workforce(workforce)?.scenario_source)
}

/// Construct only from the two admitted, digest-pinned observed artifacts.
/// # Errors
/// Refuses source, exact numeric representation, or coverage failures.
pub fn michigan_cohorts() -> Result<&'static MichiganCohorts, MichiganCohortsError> {
    static COHORTS: OnceLock<Result<MichiganCohorts, MichiganCohortsError>> = OnceLock::new();
    COHORTS
        .get_or_init(build_cohorts)
        .as_ref()
        .map_err(|error| *error)
}

/// Prepare the new content revision without admitting it to the runtime catalog.
/// # Errors
/// Refuses source, graph, or foundation construction errors.
pub fn michigan_cohort_foundation(
) -> Result<(ReplayTickSession<HypergraphStore>, FoundationContentBundle), MichiganCohortsError> {
    let cohorts = michigan_cohorts()?;
    observer_foundation_from_source(
        cohorts.scenario_source(),
        MICHIGAN_COHORT_SESSION,
        cohorts.defines_bytes(),
        FoundationContentBundle::try_new,
    )
    .map_err(MichiganCohortsError::Economy)
}

#[cfg(test)]
mod tests;
