//! Observed county-sector BUSINESS aggregates in a separate immutable foundation.
//!
//! A node represents one source cell, not a named enterprise or allocated workers.
//! Sector hyperedges classify those aggregates; they imply no trade, employment,
//! organization membership, physical production, or ownership relation.

use std::{collections::BTreeMap, fmt::Write as _, sync::OnceLock};

use babylon_graph::{hypergraph_store::HypergraphStore, stable_element::StableElementKeyV1};
use babylon_tick::replay_session::ReplayTickSession;

use crate::{
    michigan_economy::{
        append_county_observations, michigan_economy_v1, observer_foundation_from_source,
        MichiganEconomyErrorV1, QCEW_ECONOMICS_ARTIFACT_SHA256_V1,
    },
    michigan_sectors::{
        michigan_county_sectors_v1, MichiganCountySectorV1, MichiganCountySectorsV1,
        MichiganSectorCodeV1, MichiganSectorDispositionV1, MichiganSectorsErrorV1,
        QCEW_SECTORS_ARTIFACT_SHA256_V1, QCEW_SECTORS_SEMANTIC_SHA256_V1,
    },
    FoundationContentBundleV2,
};

pub const MICHIGAN_COHORT_SCENARIO_V2: &str = "production/michigan-observer-v2";
pub const MICHIGAN_COHORT_SESSION_V2: &str = "g4/michigan-observer-v2";

const BUSINESS_FIELDS: [(&str, &str); 4] = [
    ("qcew-establishments", "extensive"),
    ("qcew-employment", "extensive"),
    ("qcew-total-annual-wages", "extensive"),
    ("qcew-average-weekly-wage", "intensive"),
];

/// Deterministic source composition with immutable source identities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MichiganCohortsV2 {
    scenario_source: String,
    defines: Vec<u8>,
}
impl MichiganCohortsV2 {
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
pub enum MichiganCohortsErrorV1 {
    Economy(MichiganEconomyErrorV1),
    Sectors(MichiganSectorsErrorV1),
    NumericRepresentation,
    Coverage,
}
impl std::fmt::Display for MichiganCohortsErrorV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Michigan cohort foundation refused: {self:?}")
    }
}
impl std::error::Error for MichiganCohortsErrorV1 {}

/// Exact composite NAICS code remains part of the stable local subject name.
#[must_use]
pub fn michigan_business_local_name_v1(row: &MichiganCountySectorV1) -> String {
    format!(
        "business-{}-{}",
        row.county_geoid(),
        row.sector_code().as_str()
    )
}

#[must_use]
pub fn michigan_business_subject_v2(row: &MichiganCountySectorV1) -> StableElementKeyV1 {
    michigan_business_subject_for_owner_v2(row.county_geoid(), row.sector_code().as_str())
}

/// Construct the same subject from captured owner identity without reading current sources.
#[must_use]
pub fn michigan_business_subject_for_owner_v2(
    county_geoid: &str,
    sector_code: &str,
) -> StableElementKeyV1 {
    StableElementKeyV1::Node {
        scenario: MICHIGAN_COHORT_SCENARIO_V2.to_owned(),
        local_name: format!("business-{county_geoid}-{sector_code}"),
    }
}

/// Code 99 has no classified sector subject or membership.
#[must_use]
pub fn michigan_sector_subject_v2(code: MichiganSectorCodeV1) -> Option<StableElementKeyV1> {
    (code.disposition() == MichiganSectorDispositionV1::Classified).then(|| {
        StableElementKeyV1::Hyperedge {
            scenario: MICHIGAN_COHORT_SCENARIO_V2.to_owned(),
            local_name: format!("sector-{}", code.as_str()),
        }
    })
}

fn append_business(
    source: &mut String,
    row: &MichiganCountySectorV1,
) -> Result<(), MichiganCohortsErrorV1> {
    writeln!(source, "  (node {} NodeType/ORGANIZATION\n    (organization/kind OrgKind/BUSINESS)\n    (organization/county-fips {})", michigan_business_local_name_v1(row), row.county_geoid()).expect("String write");
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
                return Err(MichiganCohortsErrorV1::NumericRepresentation);
            }
            writeln!(source, "    (organization/{field} {value})").expect("String write");
        }
    }
    source.push_str("  )\n");
    Ok(())
}

fn append_sectors(
    source: &mut String,
    sectors: &MichiganCountySectorsV1,
) -> Result<(), MichiganCohortsErrorV1> {
    let mut memberships = BTreeMap::<MichiganSectorCodeV1, Vec<String>>::new();
    for row in sectors.rows() {
        if row.sector_code().disposition() == MichiganSectorDispositionV1::Classified {
            memberships
                .entry(row.sector_code())
                .or_default()
                .push(michigan_business_local_name_v1(row));
        }
    }
    if memberships.len() != 19 || memberships.values().map(Vec::len).sum::<usize>() != 1_522 {
        return Err(MichiganCohortsErrorV1::Coverage);
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

fn build_cohorts() -> Result<MichiganCohortsV2, MichiganCohortsErrorV1> {
    build_cohorts_with_workforce(&[])
}

fn build_cohorts_with_workforce(
    workforce: &[crate::michigan_material::MichiganWorkforceSeedV1],
) -> Result<MichiganCohortsV2, MichiganCohortsErrorV1> {
    let economy = michigan_economy_v1().map_err(MichiganCohortsErrorV1::Economy)?;
    let sectors = michigan_county_sectors_v1().map_err(MichiganCohortsErrorV1::Sectors)?;
    if sectors.rows().len() != 1_603 {
        return Err(MichiganCohortsErrorV1::Coverage);
    }
    let workforce_type = if workforce.is_empty() {
        ""
    } else {
        " SOCIAL_CLASS"
    };
    let mut source = format!("(scenario {MICHIGAN_COHORT_SCENARIO_V2}\n  (defvocabulary NodeType (TERRITORY ORGANIZATION{workforce_type}))\n  (defvocabulary HyperedgeType (ECONOMIC_SECTOR))\n  (deffield territory/county-fips int extensive)\n");
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
        for field in babylon_tick::material_staffing::STAFFING_FIELDS_V1 {
            writeln!(&mut source, "  (deffield {field} int extensive)").expect("String write");
        }
        for seed in workforce {
            writeln!(&mut source, "  (node {} NodeType/SOCIAL_CLASS (social-class/employed-population {}) (social-class/reserve-population {}) (social-class/previous-unretained-labor-hours {}))", seed.local_name(), seed.employed, seed.reserve, seed.previous_unretained_hours).expect("String write");
        }
    }
    source.push_str(")\n");
    let defines = format!("{{\"qcew_vintage\":2024,\"county_artifact_sha256\":\"{QCEW_ECONOMICS_ARTIFACT_SHA256_V1}\",\"sector_artifact_sha256\":\"{QCEW_SECTORS_ARTIFACT_SHA256_V1}\",\"sector_semantic_sha256\":\"{QCEW_SECTORS_SEMANTIC_SHA256_V1}\",\"cohort_composition_version\":2}}").into_bytes();
    Ok(MichiganCohortsV2 {
        scenario_source: source,
        defines,
    })
}

/// Material-only graph composition. The observed foundation has no workforce seeds.
pub(crate) fn michigan_staffed_scenario_v1(
    workforce: &[crate::michigan_material::MichiganWorkforceSeedV1],
) -> Result<String, MichiganCohortsErrorV1> {
    Ok(build_cohorts_with_workforce(workforce)?.scenario_source)
}

/// Construct only from the two admitted, digest-pinned observed artifacts.
/// # Errors
/// Refuses source, exact numeric representation, or coverage failures.
pub fn michigan_cohorts_v2() -> Result<&'static MichiganCohortsV2, MichiganCohortsErrorV1> {
    static COHORTS: OnceLock<Result<MichiganCohortsV2, MichiganCohortsErrorV1>> = OnceLock::new();
    COHORTS
        .get_or_init(build_cohorts)
        .as_ref()
        .map_err(|error| *error)
}

/// Prepare the new content revision without admitting it to the runtime catalog.
/// # Errors
/// Refuses source, graph, or foundation construction errors.
pub fn michigan_cohort_foundation_v2() -> Result<
    (
        ReplayTickSession<HypergraphStore>,
        FoundationContentBundleV2,
    ),
    MichiganCohortsErrorV1,
> {
    let cohorts = michigan_cohorts_v2()?;
    observer_foundation_from_source(
        cohorts.scenario_source(),
        MICHIGAN_COHORT_SESSION_V2,
        cohorts.defines_bytes(),
        FoundationContentBundleV2::try_new,
    )
    .map_err(MichiganCohortsErrorV1::Economy)
}

#[cfg(test)]
mod tests;
