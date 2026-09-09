//! Executable, immutable bundles for the admitted Michigan production and merchant owners.
//!
//! Each bundle owns exact material rows, separate from observed jobs or an
//! inferred factory. Staffing closes its labor account at the material boundary.
//! The existing V3 transition remains the sole production adjudicator.

mod codec;
pub(crate) mod foundation;
mod michigan;
mod staffing;
mod validate;

use babylon_graph::stable_element::StableElementKeyV1;
use babylon_kernel::sha256_of;
use babylon_material_circuit::{
    decode_material_circuit_state_v3, encode_material_circuit_state_v3, MaterialCircuitErrorV3,
    MaterialCircuitStateV3, ProcessIdV1, UnitIdV1,
};

pub use michigan::{compile_sector_bundles_v2, michigan_sector_bundles_v2};

const BUNDLE_DOMAIN: &[u8] = b"babylon.sector-bundle.v2\0";
const BUNDLE_VERSION: u16 = 2;
const MAX_BUNDLE_BYTES: usize = 1_048_576;
const MAX_BUNDLE_TEXT_BYTES: usize = 4_096;
const MAX_BUNDLE_GOODS: usize = 64;
const MAX_BUNDLE_PROCESSES: usize = 64;
use crate::michigan_material::MICHIGAN_MAX_HORIZON_PERIODS_V1;

/// Closed content refusals; an absent productive bundle never means zero output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectorBundleErrorV2 {
    Bound,
    Source,
    Owner,
    ProcessOwnership,
    GoodUnit,
    Resource,
    Coverage,
    Foundation,
    Preset,
    Arithmetic,
    Digest,
    WireDomain,
    WireVersion,
    WireTruncated,
    WireTrailing,
    WireNoncanonical,
    Circuit(MaterialCircuitErrorV3),
}
impl std::fmt::Display for SectorBundleErrorV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sector bundle refused: {self:?}")
    }
}
impl std::error::Error for SectorBundleErrorV2 {}
impl From<MaterialCircuitErrorV3> for SectorBundleErrorV2 {
    fn from(error: MaterialCircuitErrorV3) -> Self {
        Self::Circuit(error)
    }
}

/// Observed ownership context. No employee or financial measure is allocated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectorBundleOwnerV2 {
    subject: StableElementKeyV1,
    county_geoid: String,
    sector_code: String,
}
impl SectorBundleOwnerV2 {
    #[must_use]
    pub const fn subject(&self) -> &StableElementKeyV1 {
        &self.subject
    }
    #[must_use]
    pub fn county_geoid(&self) -> &str {
        &self.county_geoid
    }
    #[must_use]
    pub fn sector_code(&self) -> &str {
        &self.sector_code
    }
}

/// Exact sources of the observed binding and the separately Designed coefficients.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectorBundleSourcesV2 {
    county_source_file: String,
    county_source_sha256: [u8; 32],
    sector_artifact_sha256: [u8; 32],
    sector_semantic_sha256: [u8; 32],
    industry_artifact_sha256: [u8; 32],
    designed_scenario_sha256: [u8; 32],
}
impl SectorBundleSourcesV2 {
    #[must_use]
    pub fn county_source_file(&self) -> &str {
        &self.county_source_file
    }
    #[must_use]
    pub const fn county_source_sha256(&self) -> [u8; 32] {
        self.county_source_sha256
    }
    #[must_use]
    pub const fn designed_scenario_sha256(&self) -> [u8; 32] {
        self.designed_scenario_sha256
    }
}

/// A physical good has one exact unit inside and across the compiled bundles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SectorBundleGoodV2 {
    good_id: babylon_material_circuit::GoodIdV1,
    unit_id: UnitIdV1,
}
impl SectorBundleGoodV2 {
    #[must_use]
    pub const fn good_id(self) -> babylon_material_circuit::GoodIdV1 {
        self.good_id
    }
    #[must_use]
    pub const fn unit_id(self) -> UnitIdV1 {
        self.unit_id
    }
}

/// A process belongs to one bundle; its site remains a separate resource account.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SectorBundleProcessV2 {
    process_id: ProcessIdV1,
    industry_code: String,
}
impl SectorBundleProcessV2 {
    #[must_use]
    pub const fn process_id(&self) -> ProcessIdV1 {
        self.process_id
    }
    #[must_use]
    pub fn industry_code(&self) -> &str {
        &self.industry_code
    }
}

/// Canonical executable content. Borrowed rows cannot mutate the bundle.
///
/// Rows contain production, inventory, labor and logistics-node ownership only.
/// Cross-bundle routes and orders belong to the circuit composition. Keeping the
/// V3 row codec avoids a second interpretation of recipe coefficients.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectorBundleV2 {
    owner: SectorBundleOwnerV2,
    sources: SectorBundleSourcesV2,
    goods: Vec<SectorBundleGoodV2>,
    processes: Vec<SectorBundleProcessV2>,
    labor_unit: UnitIdV1,
    rows: MaterialCircuitStateV3,
    bytes: Vec<u8>,
    digest: [u8; 32],
}
impl SectorBundleV2 {
    fn from_parts(
        owner: SectorBundleOwnerV2,
        sources: SectorBundleSourcesV2,
        mut goods: Vec<SectorBundleGoodV2>,
        mut processes: Vec<SectorBundleProcessV2>,
        labor_unit: UnitIdV1,
        rows: &MaterialCircuitStateV3,
    ) -> Result<Self, SectorBundleErrorV2> {
        goods.sort_unstable();
        processes.sort_unstable();
        let rows = decode_material_circuit_state_v3(&encode_material_circuit_state_v3(rows)?)?;
        let mut bundle = Self {
            owner,
            sources,
            goods,
            processes,
            labor_unit,
            rows,
            bytes: Vec::new(),
            digest: [0; 32],
        };
        validate::bundle(&bundle)?;
        bundle.bytes = codec::encode(&bundle)?;
        bundle.digest = sha256_of(&bundle.bytes);
        Ok(bundle)
    }

    /// Decode canonical bytes against a caller's independently admitted digest.
    /// # Errors
    /// Refuses changed identity, malformed content and noncanonical encodings.
    pub fn decode(bytes: &[u8], expected: [u8; 32]) -> Result<Self, SectorBundleErrorV2> {
        if bytes.len() > MAX_BUNDLE_BYTES {
            return Err(SectorBundleErrorV2::Bound);
        }
        if sha256_of(bytes) != expected {
            return Err(SectorBundleErrorV2::Digest);
        }
        codec::decode(bytes)
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    #[must_use]
    pub const fn sha256(&self) -> [u8; 32] {
        self.digest
    }
    #[must_use]
    pub const fn owner(&self) -> &SectorBundleOwnerV2 {
        &self.owner
    }
    #[must_use]
    pub const fn sources(&self) -> &SectorBundleSourcesV2 {
        &self.sources
    }
    #[must_use]
    pub const fn horizon_ticks(&self) -> u64 {
        MICHIGAN_MAX_HORIZON_PERIODS_V1
    }
    #[must_use]
    pub fn goods(&self) -> &[SectorBundleGoodV2] {
        &self.goods
    }
    #[must_use]
    pub fn processes(&self) -> &[SectorBundleProcessV2] {
        &self.processes
    }
    #[must_use]
    pub const fn material_rows(&self) -> &MaterialCircuitStateV3 {
        &self.rows
    }
    #[must_use]
    pub const fn production_evidence_class(&self) -> crate::ArchiveEvidenceClassV1 {
        crate::ArchiveEvidenceClassV1::Designed
    }
}

#[cfg(test)]
mod tests;
