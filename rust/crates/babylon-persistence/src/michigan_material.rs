//! Captured, normalized Designed physical content with separate observed evidence.
//! Both regional and statewide authoring feed the same material compiler.

mod model;
#[cfg(test)]
mod normalized_tests;
mod regional;
mod source;
mod statewide;
mod validate;

use crate::michigan_defines::{MichiganDefinesErrorV1, MichiganDefinesV3};
use babylon_bsl::causal_contract::EvidenceClass;
use babylon_kernel::sha256_of;
use babylon_material_circuit::{CorridorIdV2, MaterialCircuitErrorV3};
pub use model::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const MICHIGAN_INDUSTRY_BASELINE_SHA256_V1: &str =
    "eb486d7e11b8b63fc58c53ab918eff84b341b293a66faf422ddb9304fb2b553e";
pub const MICHIGAN_MAX_HORIZON_PERIODS_V1: u64 = 16;
pub const MAX_MICHIGAN_CAPTURED_CONTENT_BYTES_V2: usize = 64 * 1024 * 1024;
const SOURCE_URL: &str = "https://data.bls.gov/cew/data/files/2024/csv/2024_annual_by_area.zip";
const ID_DOMAIN: &str = "babylon.michigan-material.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MichiganDeliveryPresetV1 {
    Standard,
    Delayed,
    SharedFreightAmple,
    SharedFreightConstrained,
    StatewideBaseline,
    StatewideFreightConstraint,
    StatewidePackagingShortage,
    StatewideBoth,
}
impl MichiganDeliveryPresetV1 {
    #[must_use]
    pub const fn is_statewide(self) -> bool {
        matches!(
            self,
            Self::StatewideBaseline
                | Self::StatewideFreightConstraint
                | Self::StatewidePackagingShortage
                | Self::StatewideBoth
        )
    }
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Standard => "michigan-material-standard-v7",
            Self::Delayed => "michigan-material-delayed-v7",
            Self::SharedFreightAmple => "michigan-material-shared-freight-ample-v7",
            Self::SharedFreightConstrained => "michigan-material-shared-freight-constrained-v7",
            Self::StatewideBaseline => "michigan-material-statewide-baseline-v7",
            Self::StatewideFreightConstraint => "michigan-material-statewide-freight-constraint-v7",
            Self::StatewidePackagingShortage => "michigan-material-statewide-packaging-shortage-v7",
            Self::StatewideBoth => "michigan-material-statewide-both-v7",
        }
    }
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        [
            Self::Standard,
            Self::Delayed,
            Self::SharedFreightAmple,
            Self::SharedFreightConstrained,
            Self::StatewideBaseline,
            Self::StatewideFreightConstraint,
            Self::StatewidePackagingShortage,
            Self::StatewideBoth,
        ]
        .into_iter()
        .find(|preset| preset.id() == id)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganMaterialErrorV1 {
    ArtifactDigest,
    ArtifactDecode,
    ArtifactShape,
    SourceSuppressed,
    SourceValue,
    ContentReference,
    ContentValue,
    PhysicalPath,
    Preset,
    Bound,
    Circuit(MaterialCircuitErrorV3),
}
impl std::fmt::Display for MichiganMaterialErrorV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Michigan material content refused: {self:?}")
    }
}
impl std::error::Error for MichiganMaterialErrorV1 {}
fn identity(kind: &str, key: &str) -> [u8; 32] {
    sha256_of(format!("{ID_DOMAIN}\0{kind}\0{key}").as_bytes())
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MichiganCapturedContentV2 {
    schema: String,
    graph_scenario_source: String,
    observed_defines: Vec<u8>,
    defines: MichiganDefinesV3,
    base_preset: MichiganDeliveryPresetV1,
    selected_preset: MichiganDeliveryPresetV1,
    normalized: MichiganNormalizedContentV2,
    interventions: Vec<MichiganInterventionV2>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MichiganMaterialCatalogV1 {
    capture: MichiganCapturedContentV2,
    scenario: MichiganNormalizedContentV2,
    defines_bytes: Vec<u8>,
    defines_digest: [u8; 32],
}
impl MichiganMaterialCatalogV1 {
    /// Load the authored sources required by a fresh campaign's geographic scope.
    /// # Errors
    /// Refuses missing or changed qualification artifacts and unfrozen interventions.
    pub fn load_for_preset(
        path: &Path,
        preset: MichiganDeliveryPresetV1,
    ) -> Result<Self, MichiganDefinesErrorV1> {
        if preset.is_statewide() {
            source::load_statewide(path)
        } else {
            Self::load_defines(path)
        }
    }
    /// Read parameters once for a fresh regional campaign.
    /// # Errors
    /// Refuses unknown, missing, malformed or out-of-bound authored values.
    pub fn load_defines(path: &Path) -> Result<Self, MichiganDefinesErrorV1> {
        regional::compile(MichiganDefinesV3::load(path)?)
    }
    /// # Errors
    /// Refuses malformed authored content.
    pub fn from_defines_toml(text: &str) -> Result<Self, MichiganDefinesErrorV1> {
        regional::compile(MichiganDefinesV3::parse(text)?)
    }
    /// Capture prequalified physical paths and source-supported statewide relationships.
    /// # Errors
    /// Refuses disconnected, unqualified, mismatched or malformed authority.
    pub fn from_statewide_qualification(
        defines_toml: &str,
        qualification: &[u8],
        physical: MichiganPhysicalNetworkV2,
        interventions: Vec<MichiganInterventionV2>,
    ) -> Result<Self, MichiganDefinesErrorV1> {
        statewide::compile(defines_toml, qualification, physical, interventions)
    }
    pub(crate) fn from_stored_defines(bytes: &[u8]) -> Result<Self, MichiganDefinesErrorV1> {
        if bytes.len() > MAX_MICHIGAN_CAPTURED_CONTENT_BYTES_V2 {
            return Err(MichiganDefinesErrorV1::Material(
                MichiganMaterialErrorV1::Bound,
            ));
        }
        let capture: MichiganCapturedContentV2 =
            serde_json::from_slice(bytes).map_err(|_| MichiganDefinesErrorV1::Canonical)?;
        // Numeric constraints remain separately bounded and checked; no source file is reopened.
        MichiganDefinesV3::decode(&capture.defines.encode()?)?;
        let result = Self::capture(capture)?;
        if result.defines_bytes != bytes {
            return Err(MichiganDefinesErrorV1::Canonical);
        }
        Ok(result)
    }
    pub(super) fn from_normalized(
        defines: MichiganDefinesV3,
        mut normalized: MichiganNormalizedContentV2,
        base_preset: MichiganDeliveryPresetV1,
        mut interventions: Vec<MichiganInterventionV2>,
    ) -> Result<Self, MichiganDefinesErrorV1> {
        validate::canonicalize(&mut normalized, &mut interventions);
        let observed = crate::michigan_cohorts::michigan_cohorts_v2()
            .map_err(|_| MichiganDefinesErrorV1::Material(MichiganMaterialErrorV1::SourceValue))?;
        let graph_scenario_source =
            crate::michigan_cohorts::michigan_staffed_scenario_v1(&normalized.staffing.pools)
                .map_err(|_| {
                    MichiganDefinesErrorV1::Material(MichiganMaterialErrorV1::ContentValue)
                })?;
        Self::capture(MichiganCapturedContentV2 {
            schema: "MichiganCapturedContentV2".to_owned(),
            graph_scenario_source,
            observed_defines: observed.defines_bytes().to_vec(),
            defines,
            normalized,
            base_preset,
            selected_preset: base_preset,
            interventions,
        })
    }
    fn capture(mut capture: MichiganCapturedContentV2) -> Result<Self, MichiganDefinesErrorV1> {
        use MichiganDefinesErrorV1::Material;
        if capture.graph_scenario_source.is_empty()
            || capture.graph_scenario_source.len() > 1_048_576
            || capture.observed_defines.is_empty()
            || capture.observed_defines.len() > 65_536
        {
            return Err(Material(MichiganMaterialErrorV1::Bound));
        }
        if capture.schema != "MichiganCapturedContentV2" {
            return Err(MichiganDefinesErrorV1::Canonical);
        }
        validate::canonicalize(&mut capture.normalized, &mut capture.interventions);
        validate::content(&capture.normalized).map_err(Material)?;
        validate::interventions(
            &capture.normalized,
            capture.base_preset,
            &capture.interventions,
        )
        .map_err(Material)?;
        let mut scenario = capture.normalized.clone();
        if capture.selected_preset != capture.base_preset {
            let intervention = capture
                .interventions
                .iter()
                .find(|row| row.preset == capture.selected_preset)
                .ok_or(Material(MichiganMaterialErrorV1::Preset))?;
            validate::apply(&mut scenario, intervention).map_err(Material)?;
            validate::content(&scenario).map_err(Material)?;
        }
        let defines_bytes =
            serde_json::to_vec(&capture).map_err(|_| MichiganDefinesErrorV1::Canonical)?;
        if defines_bytes.len() > MAX_MICHIGAN_CAPTURED_CONTENT_BYTES_V2 {
            return Err(Material(MichiganMaterialErrorV1::Bound));
        }
        Ok(Self {
            capture,
            scenario,
            defines_digest: sha256_of(&defines_bytes),
            defines_bytes,
        })
    }
    /// Resolve only the explicit overrides captured with this campaign.
    /// # Errors
    /// Refuses a preset absent from the captured authority.
    pub fn with_preset(
        &self,
        preset: MichiganDeliveryPresetV1,
    ) -> Result<Self, MichiganDefinesErrorV1> {
        if self.preset() == preset {
            return Ok(self.clone());
        }
        let mut capture = self.capture.clone();
        capture.selected_preset = preset;
        Self::capture(capture)
    }
    #[must_use]
    pub const fn preset(&self) -> MichiganDeliveryPresetV1 {
        self.capture.selected_preset
    }
    #[must_use]
    pub fn graph_scenario_source(&self) -> &str {
        &self.capture.graph_scenario_source
    }
    #[must_use]
    pub fn observed_defines(&self) -> &[u8] {
        &self.capture.observed_defines
    }
    #[must_use]
    pub fn defines_bytes(&self) -> &[u8] {
        &self.defines_bytes
    }
    #[must_use]
    pub const fn defines_hash(&self) -> [u8; 32] {
        self.defines_digest
    }
    #[must_use]
    pub const fn horizon_ticks(&self) -> u64 {
        self.scenario.horizon_ticks
    }
    #[must_use]
    pub fn staffing(&self) -> &MichiganStaffingDesignV1 {
        &self.scenario.staffing
    }
    #[must_use]
    pub fn sites(&self) -> &[MichiganMaterialSiteV1] {
        &self.scenario.sites
    }
    #[must_use]
    pub fn goods(&self) -> &[MichiganMaterialGoodV1] {
        &self.scenario.goods
    }
    #[must_use]
    pub fn processes(&self) -> &[MichiganMaterialProcessV1] {
        &self.scenario.processes
    }
    #[must_use]
    pub fn routes(&self) -> &[MichiganMaterialRouteV1] {
        &self.scenario.routes
    }
    #[must_use]
    pub fn corridors(&self) -> &[MichiganMaterialCorridorV1] {
        &self.scenario.corridors
    }
    #[must_use]
    pub fn merchants(&self) -> &[MichiganMerchantV2] {
        &self.scenario.merchants
    }
    #[must_use]
    pub fn final_demands(&self) -> &[MichiganFinalDemandV2] {
        &self.scenario.final_demands
    }
    #[must_use]
    pub fn owners(&self) -> &[MichiganOwnerSourceV2] {
        &self.scenario.owners
    }
    #[must_use]
    pub fn owner_source(&self, county: &str, sector: &str) -> Option<&MichiganOwnerSourceV2> {
        self.owners()
            .iter()
            .find(|o| o.county_geoid == county && o.sector_code == sector)
    }
    #[must_use]
    pub fn physical_network(&self) -> Option<&MichiganPhysicalNetworkV2> {
        self.scenario.physical_network.as_ref()
    }
    #[must_use]
    pub const fn source_url(&self) -> &'static str {
        SOURCE_URL
    }
    #[must_use]
    pub const fn source_evidence_class(&self) -> EvidenceClass {
        EvidenceClass::Observed
    }
    #[must_use]
    pub const fn physical_evidence_class(&self) -> EvidenceClass {
        EvidenceClass::Designed
    }
    #[must_use]
    pub const fn source_vintage(&self) -> u16 {
        2024
    }
    #[must_use]
    pub fn geographic_scale(&self) -> &str {
        &self.scenario.geographic_scale
    }
    #[must_use]
    pub fn terminal_output_disposition(&self) -> &str {
        &self.scenario.terminal_output_disposition
    }
    #[must_use]
    pub fn industry_for_site(
        &self,
        site: &MichiganMaterialSiteV1,
    ) -> Option<&MichiganIndustryBaselineRowV1> {
        self.scenario
            .industry
            .iter()
            .find(|row| row.area_fips == site.county_geoid && row.industry_code == site.naics)
    }
    #[must_use]
    pub fn site(&self, key: &str) -> Option<&MichiganMaterialSiteV1> {
        self.sites().iter().find(|row| row.key == key)
    }
    #[must_use]
    pub fn good(&self, key: &str) -> Option<&MichiganMaterialGoodV1> {
        self.goods().iter().find(|row| row.key == key)
    }
    #[must_use]
    pub fn corridor_label(&self, id: CorridorIdV2) -> Option<&str> {
        self.corridors()
            .iter()
            .find(|row| row.id() == id)
            .map(|row| row.label.as_str())
    }
}
