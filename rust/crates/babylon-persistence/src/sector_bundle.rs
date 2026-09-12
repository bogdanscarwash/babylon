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

use babylon_graph::stable_element::StableElementKey;
use babylon_kernel::content_digest::sha256_of;
use babylon_material_circuit::{
    decode_material_circuit_state, encode_material_circuit_state, MaterialCircuitError,
    MaterialCircuitState, ProcessId, UnitId,
};

pub use michigan::{compile_sector_bundles, michigan_sector_bundles};

const BUNDLE_DOMAIN: &[u8] = b"babylon.sector-bundle.v2\0";
const BUNDLE_VERSION: u16 = 2;
const MAX_BUNDLE_BYTES: usize = 1_048_576;
const MAX_BUNDLE_TEXT_BYTES: usize = 4_096;
const MAX_BUNDLE_GOODS: usize = 64;
const MAX_BUNDLE_PROCESSES: usize = 64;
use crate::michigan_material::MICHIGAN_MAX_HORIZON_PERIODS;

/// Closed content refusals; an absent productive bundle never means zero output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectorBundleError {
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
    Circuit(MaterialCircuitError),
}
impl std::fmt::Display for SectorBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sector bundle refused: {self:?}")
    }
}
impl std::error::Error for SectorBundleError {}
impl From<MaterialCircuitError> for SectorBundleError {
    fn from(error: MaterialCircuitError) -> Self {
        Self::Circuit(error)
    }
}

/// Observed ownership context. No employee or financial measure is allocated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectorBundleOwner {
    subject: StableElementKey,
    county_geoid: String,
    sector_code: String,
}
impl SectorBundleOwner {
    #[must_use]
    pub const fn subject(&self) -> &StableElementKey {
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
pub struct SectorBundleSources {
    county_source_file: String,
    county_source_sha256: [u8; 32],
    sector_artifact_sha256: [u8; 32],
    sector_semantic_sha256: [u8; 32],
    industry_artifact_sha256: [u8; 32],
    designed_scenario_sha256: [u8; 32],
}
impl SectorBundleSources {
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
pub struct SectorBundleGood {
    good_id: babylon_material_circuit::GoodId,
    unit_id: UnitId,
}
impl SectorBundleGood {
    #[must_use]
    pub const fn good_id(self) -> babylon_material_circuit::GoodId {
        self.good_id
    }
    #[must_use]
    pub const fn unit_id(self) -> UnitId {
        self.unit_id
    }
}

/// A process belongs to one bundle; its site remains a separate resource account.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SectorBundleProcess {
    process_id: ProcessId,
    industry_code: String,
}
impl SectorBundleProcess {
    #[must_use]
    pub const fn process_id(&self) -> ProcessId {
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
pub struct SectorBundle {
    owner: SectorBundleOwner,
    sources: SectorBundleSources,
    goods: Vec<SectorBundleGood>,
    processes: Vec<SectorBundleProcess>,
    labor_unit: UnitId,
    rows: MaterialCircuitState,
    bytes: Vec<u8>,
    digest: [u8; 32],
}
impl SectorBundle {
    fn from_parts(
        owner: SectorBundleOwner,
        sources: SectorBundleSources,
        mut goods: Vec<SectorBundleGood>,
        mut processes: Vec<SectorBundleProcess>,
        labor_unit: UnitId,
        rows: &MaterialCircuitState,
    ) -> Result<Self, SectorBundleError> {
        goods.sort_unstable();
        processes.sort_unstable();
        let rows = decode_material_circuit_state(&encode_material_circuit_state(rows)?)?;
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
    pub fn decode(bytes: &[u8], expected: [u8; 32]) -> Result<Self, SectorBundleError> {
        if bytes.len() > MAX_BUNDLE_BYTES {
            return Err(SectorBundleError::Bound);
        }
        if sha256_of(bytes) != expected {
            return Err(SectorBundleError::Digest);
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
    pub const fn owner(&self) -> &SectorBundleOwner {
        &self.owner
    }
    #[must_use]
    pub const fn sources(&self) -> &SectorBundleSources {
        &self.sources
    }
    #[must_use]
    pub const fn horizon_ticks(&self) -> u64 {
        MICHIGAN_MAX_HORIZON_PERIODS
    }
    #[must_use]
    pub fn goods(&self) -> &[SectorBundleGood] {
        &self.goods
    }
    #[must_use]
    pub fn processes(&self) -> &[SectorBundleProcess] {
        &self.processes
    }
    #[must_use]
    pub const fn material_rows(&self) -> &MaterialCircuitState {
        &self.rows
    }
    #[must_use]
    pub const fn production_evidence_class(&self) -> crate::ArchiveEvidenceClass {
        crate::ArchiveEvidenceClass::Designed
    }
}

#[cfg(test)]
mod tests;
