//! Closed admission of durable Michigan content revisions.
//!
//! A stored revision selects its own immutable identity. Creation uses the newest
//! admitted graph revision; reopening reconstructs stored bytes after admission.

use std::sync::OnceLock;

use babylon_kernel::sha256_of;
use babylon_tick::material_world::MaterialWorldRegisterV2;

use crate::{
    material_runtime::MaterialRuntimeFoundationV2, michigan_cohorts::MICHIGAN_COHORT_SCENARIO_V2,
    michigan_material::MichiganDeliveryPresetV1,
};

/// Graph content revisions are separate from the logical delivery choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganContentPresetV1 {
    StaffedStandardV4,
    StaffedDelayedV4,
}

/// All admitted content revisions retain this exact bounded physical projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganPhysicalProjectionV1 {
    FiveProcessV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganContentErrorV1 {
    UnknownPreset,
    ObservedSource,
    MaterialSource,
    Foundation,
    IdentityMismatch,
}
impl std::fmt::Display for MichiganContentErrorV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Michigan content admission refused: {self:?}")
    }
}
impl std::error::Error for MichiganContentErrorV1 {}

pub const MICHIGAN_CONTENT_PRESETS_V1: [MichiganContentPresetV1; 2] = [
    MichiganContentPresetV1::StaffedStandardV4,
    MichiganContentPresetV1::StaffedDelayedV4,
];

impl MichiganContentPresetV1 {
    #[must_use]
    pub const fn new_campaign(delivery: MichiganDeliveryPresetV1) -> Self {
        match delivery {
            MichiganDeliveryPresetV1::Standard => Self::StaffedStandardV4,
            MichiganDeliveryPresetV1::Delayed => Self::StaffedDelayedV4,
        }
    }
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::StaffedStandardV4 => "michigan-material-standard-v4",
            Self::StaffedDelayedV4 => "michigan-material-delayed-v4",
        }
    }
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        MICHIGAN_CONTENT_PRESETS_V1
            .into_iter()
            .find(|preset| preset.id() == id)
    }
    #[must_use]
    pub const fn delivery(self) -> MichiganDeliveryPresetV1 {
        match self {
            Self::StaffedStandardV4 => MichiganDeliveryPresetV1::Standard,
            Self::StaffedDelayedV4 => MichiganDeliveryPresetV1::Delayed,
        }
    }
    #[must_use]
    pub const fn scenario(self) -> &'static str {
        MICHIGAN_COHORT_SCENARIO_V2
    }
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::StaffedStandardV4 => "Michigan: standard delivery (four active cohorts)",
            Self::StaffedDelayedV4 => "Michigan: delayed sheet delivery (four active cohorts)",
        }
    }
    /// # Errors
    /// Refuses any changed source or foundation construction failure.
    pub fn admitted(self) -> Result<&'static MichiganContentAdmissionV1, MichiganContentErrorV1> {
        static ENTRIES: [OnceLock<Result<MichiganContentAdmissionV1, MichiganContentErrorV1>>; 2] =
            [const { OnceLock::new() }; 2];
        let index = match self {
            Self::StaffedStandardV4 => 0,
            Self::StaffedDelayedV4 => 1,
        };
        ENTRIES[index]
            .get_or_init(|| self.capture_admission())
            .as_ref()
            .map_err(|error| *error)
    }
    /// Construct the exact selected revision for a new campaign only.
    /// # Errors
    /// Refuses source or identity drift against the admission catalog.
    pub fn create_foundation(self) -> Result<MaterialRuntimeFoundationV2, MichiganContentErrorV1> {
        let foundation = self.build_foundation()?;
        let admitted = self.admitted()?;
        if foundation.canonical_bytes() != admitted.canonical_bytes {
            return Err(MichiganContentErrorV1::IdentityMismatch);
        }
        Ok(foundation)
    }
    fn build_foundation(self) -> Result<MaterialRuntimeFoundationV2, MichiganContentErrorV1> {
        crate::sector_bundle::foundation::create_bundle_foundation_v4(self.id(), self.delivery())
            .map_err(|_| MichiganContentErrorV1::Foundation)
    }
    fn capture_admission(self) -> Result<MichiganContentAdmissionV1, MichiganContentErrorV1> {
        let foundation = self.build_foundation()?;
        let graph = foundation.graph_foundation();
        Ok(MichiganContentAdmissionV1 {
            preset: self,
            horizon_ticks: foundation.spec().horizon_ticks,
            content_digest: foundation.spec().content_digest,
            digest: foundation.digest(),
            graph_digest: sha256_of(graph.canonical_bytes()),
            scenario_digest: sha256_of(graph.content_bundle().scenario_source_bytes()),
            canonical_bytes: foundation.canonical_bytes().to_vec(),
            register: foundation.initial_register().clone(),
            physical_projection: MichiganPhysicalProjectionV1::FiveProcessV1,
        })
    }
}

/// Immutable admission evidence, shared by the writer and both read capabilities.
pub struct MichiganContentAdmissionV1 {
    pub(crate) preset: MichiganContentPresetV1,
    pub(crate) horizon_ticks: u64,
    pub(crate) content_digest: [u8; 32],
    pub(crate) digest: [u8; 32],
    pub(crate) graph_digest: [u8; 32],
    pub(crate) scenario_digest: [u8; 32],
    pub(crate) canonical_bytes: Vec<u8>,
    pub(crate) register: MaterialWorldRegisterV2,
    pub(crate) physical_projection: MichiganPhysicalProjectionV1,
}
impl MichiganContentAdmissionV1 {
    #[must_use]
    pub const fn preset(&self) -> MichiganContentPresetV1 {
        self.preset
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    /// Validate the complete safe header, never just its self-reported digest.
    /// # Errors
    /// Refuses mixed revisions, different clocks, or changed content identities.
    pub fn validate_header(
        &self,
        horizon: i64,
        content: &[u8],
        foundation: &[u8],
        tick: u64,
    ) -> Result<(), MichiganContentErrorV1> {
        if u64::try_from(horizon).ok() != Some(self.horizon_ticks)
            || tick > self.horizon_ticks
            || content != self.content_digest
            || foundation != self.digest
        {
            return Err(MichiganContentErrorV1::IdentityMismatch);
        }
        Ok(())
    }
    /// # Errors
    /// Refuses a graph or scenario from any other content revision.
    pub fn validate_graph(
        &self,
        foundation: &[u8],
        scenario: &[u8],
    ) -> Result<(), MichiganContentErrorV1> {
        if foundation != self.graph_digest || scenario != self.scenario_digest {
            return Err(MichiganContentErrorV1::IdentityMismatch);
        }
        Ok(())
    }
}

/// Admit only an exact versioned identity from the closed catalog.
/// # Errors
/// Refuses unknown presets, source failure or mismatched stored metadata.
pub fn admit_michigan_content_v1(
    preset_id: &str,
    horizon: i64,
    content: &[u8],
    foundation: &[u8],
    tick: u64,
) -> Result<&'static MichiganContentAdmissionV1, MichiganContentErrorV1> {
    let expected = MichiganContentPresetV1::from_id(preset_id)
        .ok_or(MichiganContentErrorV1::UnknownPreset)?
        .admitted()?;
    expected.validate_header(horizon, content, foundation, tick)?;
    Ok(expected)
}

#[cfg(test)]
mod tests;
