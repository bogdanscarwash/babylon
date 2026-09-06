//! Executable, immutable bundles for the four active Michigan manufacturing cohorts.
//!
//! This content successor preserves the existing V1/V2 campaign factories. Each
//! bundle owns exact material rows, not observed jobs or an inferred factory.
//! The existing V2 transition remains the sole production adjudicator.

mod codec;
pub(crate) mod foundation;
mod michigan;
mod validate;

use babylon_graph::stable_element::StableElementKeyV1;
use babylon_kernel::sha256_of;
use babylon_material_circuit::{
    decode_material_circuit_state_v2, encode_material_circuit_state_v2, MaterialCircuitErrorV2,
    MaterialCircuitStateV2, ProcessIdV1, UnitIdV1,
};

pub use michigan::{compile_sector_bundles_v1, michigan_sector_bundles_v1};

const BUNDLE_DOMAIN: &[u8] = b"babylon.sector-bundle.v1\0";
const BUNDLE_VERSION: u16 = 1;
const MAX_BUNDLE_BYTES: usize = 1_048_576;
const MAX_BUNDLE_TEXT_BYTES: usize = 4_096;
const MAX_BUNDLE_GOODS: usize = 8;
const MAX_BUNDLE_PROCESSES: usize = 2;
const HORIZON_TICKS: u64 = 16;

/// Closed content refusals; an absent productive bundle never means zero output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectorBundleErrorV1 {
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
    Circuit(MaterialCircuitErrorV2),
}
impl std::fmt::Display for SectorBundleErrorV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sector bundle refused: {self:?}")
    }
}
impl std::error::Error for SectorBundleErrorV1 {}
impl From<MaterialCircuitErrorV2> for SectorBundleErrorV1 {
    fn from(error: MaterialCircuitErrorV2) -> Self {
        Self::Circuit(error)
    }
}

/// Observed ownership context. No employee or financial measure is allocated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectorBundleOwnerV1 {
    subject: StableElementKeyV1,
    county_geoid: String,
    sector_code: String,
}
impl SectorBundleOwnerV1 {
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
pub struct SectorBundleSourcesV1 {
    county_source_file: String,
    county_source_sha256: [u8; 32],
    sector_artifact_sha256: [u8; 32],
    sector_semantic_sha256: [u8; 32],
    industry_artifact_sha256: [u8; 32],
    designed_scenario_sha256: [u8; 32],
}
impl SectorBundleSourcesV1 {
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
pub struct SectorBundleGoodV1 {
    good_id: babylon_material_circuit::GoodIdV1,
    unit_id: UnitIdV1,
}
impl SectorBundleGoodV1 {
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
pub struct SectorBundleProcessV1 {
    process_id: ProcessIdV1,
    industry_code: String,
}
impl SectorBundleProcessV1 {
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
/// V2 row codec avoids a second interpretation of recipe coefficients.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectorBundleV1 {
    owner: SectorBundleOwnerV1,
    sources: SectorBundleSourcesV1,
    goods: Vec<SectorBundleGoodV1>,
    processes: Vec<SectorBundleProcessV1>,
    labor_unit: UnitIdV1,
    rows: MaterialCircuitStateV2,
    bytes: Vec<u8>,
    digest: [u8; 32],
}
impl SectorBundleV1 {
    fn from_parts(
        owner: SectorBundleOwnerV1,
        sources: SectorBundleSourcesV1,
        mut goods: Vec<SectorBundleGoodV1>,
        mut processes: Vec<SectorBundleProcessV1>,
        labor_unit: UnitIdV1,
        rows: &MaterialCircuitStateV2,
    ) -> Result<Self, SectorBundleErrorV1> {
        goods.sort_unstable();
        processes.sort_unstable();
        let rows = decode_material_circuit_state_v2(&encode_material_circuit_state_v2(rows)?)?;
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
    pub fn decode(bytes: &[u8], expected: [u8; 32]) -> Result<Self, SectorBundleErrorV1> {
        if bytes.len() > MAX_BUNDLE_BYTES {
            return Err(SectorBundleErrorV1::Bound);
        }
        if sha256_of(bytes) != expected {
            return Err(SectorBundleErrorV1::Digest);
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
    pub const fn owner(&self) -> &SectorBundleOwnerV1 {
        &self.owner
    }
    #[must_use]
    pub const fn sources(&self) -> &SectorBundleSourcesV1 {
        &self.sources
    }
    #[must_use]
    pub const fn horizon_ticks(&self) -> u64 {
        HORIZON_TICKS
    }
    #[must_use]
    pub fn goods(&self) -> &[SectorBundleGoodV1] {
        &self.goods
    }
    #[must_use]
    pub fn processes(&self) -> &[SectorBundleProcessV1] {
        &self.processes
    }
    #[must_use]
    pub const fn material_rows(&self) -> &MaterialCircuitStateV2 {
        &self.rows
    }
    #[must_use]
    pub const fn production_evidence_class(&self) -> crate::ArchiveEvidenceClassV1 {
        crate::ArchiveEvidenceClassV1::Designed
    }
}

#[cfg(test)]
mod tests;
