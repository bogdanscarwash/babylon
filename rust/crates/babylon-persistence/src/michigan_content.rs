//! Closed admission of durable Michigan content revisions.
//!
//! A stored revision selects its own immutable identity. Creation uses the newest
//! admitted graph revision; reopening reconstructs stored bytes after admission.

use babylon_graph::stable_state::StableGraphStateV1;
use babylon_kernel::sha256_of;
use babylon_tick::material_world::MaterialWorldRegisterV2;
use babylon_tick::{material_replay::MaterialLaborV1, material_staffing::StaffingCompositionV1};

use crate::{
    material_runtime::{MaterialComponentIdentityV1, MaterialRuntimeFoundationV2},
    michigan_cohorts::MICHIGAN_COHORT_SCENARIO_V2,
    michigan_material::{MichiganDeliveryPresetV1, MichiganMaterialCatalogV1},
};

/// Graph content revisions are separate from the logical delivery choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganContentPresetV1 {
    FourWeekStandardV6,
    FourWeekDelayedV6,
    SharedFreightAmpleV6,
    SharedFreightConstrainedV6,
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

pub const MICHIGAN_CONTENT_PRESETS_V1: [MichiganContentPresetV1; 4] = [
    MichiganContentPresetV1::FourWeekStandardV6,
    MichiganContentPresetV1::FourWeekDelayedV6,
    MichiganContentPresetV1::SharedFreightAmpleV6,
    MichiganContentPresetV1::SharedFreightConstrainedV6,
];

impl MichiganContentPresetV1 {
    #[must_use]
    pub const fn new_campaign(delivery: MichiganDeliveryPresetV1) -> Self {
        match delivery {
            MichiganDeliveryPresetV1::Standard => Self::FourWeekStandardV6,
            MichiganDeliveryPresetV1::Delayed => Self::FourWeekDelayedV6,
            MichiganDeliveryPresetV1::SharedFreightAmple => Self::SharedFreightAmpleV6,
            MichiganDeliveryPresetV1::SharedFreightConstrained => Self::SharedFreightConstrainedV6,
        }
    }
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::FourWeekStandardV6 => "michigan-material-standard-v6",
            Self::FourWeekDelayedV6 => "michigan-material-delayed-v6",
            Self::SharedFreightAmpleV6 => "michigan-material-shared-freight-ample-v6",
            Self::SharedFreightConstrainedV6 => "michigan-material-shared-freight-constrained-v6",
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
            Self::FourWeekStandardV6 => MichiganDeliveryPresetV1::Standard,
            Self::FourWeekDelayedV6 => MichiganDeliveryPresetV1::Delayed,
            Self::SharedFreightAmpleV6 => MichiganDeliveryPresetV1::SharedFreightAmple,
            Self::SharedFreightConstrainedV6 => MichiganDeliveryPresetV1::SharedFreightConstrained,
        }
    }
    #[must_use]
    pub const fn scenario(self) -> &'static str {
        MICHIGAN_COHORT_SCENARIO_V2
    }
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::FourWeekStandardV6 => "Michigan: standard delivery (four active cohorts)",
            Self::FourWeekDelayedV6 => "Michigan: delayed delivery (four active cohorts)",
            Self::SharedFreightAmpleV6 => "Shared freight — ample",
            Self::SharedFreightConstrainedV6 => "Shared freight — constrained",
        }
    }
    /// # Errors
    /// Refuses any changed source or foundation construction failure.
    pub fn admitted(
        self,
        catalog: &MichiganMaterialCatalogV1,
    ) -> Result<MichiganContentAdmissionV1, MichiganContentErrorV1> {
        self.capture_admission(catalog)
    }
    /// Create a campaign from explicit, already validated numeric parameters.
    /// # Errors
    /// Refuses invalid material composition or observed source drift.
    pub fn create_foundation(
        self,
        catalog: &MichiganMaterialCatalogV1,
    ) -> Result<MaterialRuntimeFoundationV2, MichiganContentErrorV1> {
        self.build_foundation(catalog)
    }
    fn build_foundation(
        self,
        catalog: &MichiganMaterialCatalogV1,
    ) -> Result<MaterialRuntimeFoundationV2, MichiganContentErrorV1> {
        crate::sector_bundle::foundation::create_bundle_foundation_v6(
            self.id(),
            self.delivery(),
            catalog,
        )
        .map_err(|_| MichiganContentErrorV1::Foundation)
    }
    fn capture_admission(
        self,
        catalog: &MichiganMaterialCatalogV1,
    ) -> Result<MichiganContentAdmissionV1, MichiganContentErrorV1> {
        let foundation = self.build_foundation(catalog)?;
        let graph = foundation.graph_foundation();
        let component_identity = MaterialComponentIdentityV1::from_foundation(graph);
        let graph_digest = sha256_of(graph.canonical_bytes());
        let scenario_digest = sha256_of(graph.content_bundle().scenario_source_bytes());
        let MaterialLaborV1::Staffed(staffing) = foundation.labor().clone() else {
            return Err(MichiganContentErrorV1::Foundation);
        };
        let horizon_ticks = foundation.spec().horizon_ticks;
        let content_digest = foundation.spec().content_digest;
        let digest = foundation.digest();
        let canonical_bytes = foundation.canonical_bytes().to_vec();
        let register = foundation.initial_register().clone();
        let foundation_graph = foundation
            .into_session()
            .map_err(|_| MichiganContentErrorV1::Foundation)?
            .graph_session()
            .stable_graph_state()
            .map_err(|_| MichiganContentErrorV1::Foundation)?;
        Ok(MichiganContentAdmissionV1 {
            preset: self,
            catalog: catalog.clone(),
            horizon_ticks,
            content_digest,
            digest,
            graph_digest,
            scenario_digest,
            canonical_bytes,
            register,
            foundation_graph,
            staffing,
            component_identity,
            physical_projection: MichiganPhysicalProjectionV1::FiveProcessV1,
        })
    }
}

/// Immutable admission evidence, shared by the writer and both read capabilities.
pub struct MichiganContentAdmissionV1 {
    pub(crate) preset: MichiganContentPresetV1,
    pub(crate) catalog: MichiganMaterialCatalogV1,
    pub(crate) horizon_ticks: u64,
    pub(crate) content_digest: [u8; 32],
    pub(crate) digest: [u8; 32],
    pub(crate) graph_digest: [u8; 32],
    pub(crate) scenario_digest: [u8; 32],
    pub(crate) canonical_bytes: Vec<u8>,
    pub(crate) register: MaterialWorldRegisterV2,
    pub(crate) foundation_graph: StableGraphStateV1,
    pub(crate) staffing: StaffingCompositionV1,
    pub(crate) component_identity: MaterialComponentIdentityV1,
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
    foundation_bytes: &[u8],
) -> Result<MichiganContentAdmissionV1, MichiganContentErrorV1> {
    let preset = validate_michigan_header_v1(preset_id, horizon, content, foundation, tick)?;
    let wrapped = stored_defines_from_material_foundation(foundation_bytes)?;
    let decoded = crate::sector_bundle::foundation::decode_stored_bundle_defines_v3(
        wrapped,
        sha256_of(wrapped),
    )
    .map_err(|_| MichiganContentErrorV1::MaterialSource)?;
    let expected = preset.admitted(decoded.catalog())?;
    expected.validate_header(horizon, content, foundation, tick)?;
    if expected.canonical_bytes != foundation_bytes {
        return Err(MichiganContentErrorV1::IdentityMismatch);
    }
    Ok(expected)
}

/// Check only public header shape. This does not authenticate opaque material values.
/// `KnownPreview` reads grants and observed fields without material-read capability.
pub(crate) fn validate_michigan_header_v1(
    preset_id: &str,
    horizon: i64,
    content: &[u8],
    foundation: &[u8],
    tick: u64,
) -> Result<MichiganContentPresetV1, MichiganContentErrorV1> {
    let preset =
        MichiganContentPresetV1::from_id(preset_id).ok_or(MichiganContentErrorV1::UnknownPreset)?;
    if !(1..=crate::michigan_material::MICHIGAN_MAX_HORIZON_PERIODS_V1)
        .contains(&u64::try_from(horizon).unwrap_or(0))
        || tick > u64::try_from(horizon).unwrap_or(0)
        || content.len() != 32
        || foundation.len() != 32
        || content.iter().all(|b| *b == 0)
        || foundation.iter().all(|b| *b == 0)
    {
        return Err(MichiganContentErrorV1::IdentityMismatch);
    }
    Ok(preset)
}

/// Locate numeric authority inside the current canonical material/graph/content
/// nesting. Full reconstruction above subsequently compares every byte, including
/// all fields skipped here; locating a self-reported digest never admits content.
fn stored_defines_from_material_foundation(bytes: &[u8]) -> Result<&[u8], MichiganContentErrorV1> {
    use MichiganContentErrorV1::Foundation;
    fn take<'a>(input: &mut &'a [u8], n: usize) -> Result<&'a [u8], MichiganContentErrorV1> {
        let value = input.get(..n).ok_or(Foundation)?;
        *input = &input[n..];
        Ok(value)
    }
    fn field32<'a>(input: &mut &'a [u8]) -> Result<&'a [u8], MichiganContentErrorV1> {
        let length = u32::from_be_bytes(take(input, 4)?.try_into().map_err(|_| Foundation)?);
        take(input, usize::try_from(length).map_err(|_| Foundation)?)
    }
    fn field64<'a>(input: &mut &'a [u8]) -> Result<&'a [u8], MichiganContentErrorV1> {
        let length = u64::from_be_bytes(take(input, 8)?.try_into().map_err(|_| Foundation)?);
        take(input, usize::try_from(length).map_err(|_| Foundation)?)
    }
    if bytes.len() > 67_108_864 {
        return Err(Foundation);
    }
    let mut input = bytes;
    let domain = b"babylon.material-campaign-foundation.v2\0";
    if take(&mut input, domain.len())? != domain || take(&mut input, 4)? != 2_u32.to_be_bytes() {
        return Err(Foundation);
    }
    take(&mut input, 8 + 32)?;
    field64(&mut input)?; // Preset identity is compared against the reconstructed bytes.
    let mut graph = field64(&mut input)?;
    field64(&mut input)?;
    if !input.is_empty() {
        return Err(Foundation);
    }
    for _ in 0..5 {
        field32(&mut graph)?;
    }
    take(&mut graph, 8 + 3 * 32)?;
    let domain = b"babylon.campaign-foundation-content.v2\0";
    if take(&mut graph, domain.len())? != domain || take(&mut graph, 4)? != 2_u32.to_be_bytes() {
        return Err(Foundation);
    }
    if take(&mut graph, 1)? != [1] {
        return Err(Foundation);
    }
    field32(&mut graph)?;
    if take(&mut graph, 1)? != [2] {
        return Err(Foundation);
    }
    match take(&mut graph, 1)? {
        [0] => {}
        [1] => {
            field32(&mut graph)?;
        }
        _ => return Err(Foundation),
    }
    if take(&mut graph, 1)? != [3] {
        return Err(Foundation);
    }
    field32(&mut graph)?;
    if take(&mut graph, 1)? != [4] {
        return Err(Foundation);
    }
    let defines = field32(&mut graph)?;
    if take(&mut graph, 1)? != [5] {
        return Err(Foundation);
    }
    field32(&mut graph)?;
    if !graph.is_empty() {
        return Err(Foundation);
    }
    Ok(defines)
}

#[cfg(test)]
mod tests;
