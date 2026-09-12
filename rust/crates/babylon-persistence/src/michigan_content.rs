//! Closed admission of durable Michigan content revisions.
//!
//! A stored revision selects its own immutable identity. Creation uses the newest
//! admitted graph revision; reopening reconstructs stored bytes after admission.

use babylon_graph::stable_state::StableGraphState;
use babylon_kernel::content_digest::sha256_of;
use babylon_tick::material_staffing::StaffingComposition;
use babylon_tick::material_world::MaterialWorldRegister;

use crate::{
    material_runtime::{MaterialComponentIdentity, MaterialRuntimeFoundation},
    michigan_cohorts::MICHIGAN_COHORT_SCENARIO,
    michigan_material::{MichiganDeliveryPreset, MichiganMaterialCatalog},
};

/// Graph content revisions are separate from the logical delivery choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganContentPreset {
    FourWeekStandard,
    FourWeekDelayed,
    SharedFreightAmple,
    SharedFreightConstrained,
    StatewideBaseline,
    StatewideFreightConstraint,
    StatewidePackagingShortage,
    StatewideBoth,
}

/// All admitted presets use the same normalized physical projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganPhysicalProjection {
    Normalized,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganContentError {
    UnknownPreset,
    ObservedSource,
    MaterialSource,
    Foundation,
    IdentityMismatch,
}
impl std::fmt::Display for MichiganContentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Michigan content admission refused: {self:?}")
    }
}
impl std::error::Error for MichiganContentError {}

pub const MICHIGAN_CONTENT_PRESETS: [MichiganContentPreset; 8] = [
    MichiganContentPreset::FourWeekStandard,
    MichiganContentPreset::FourWeekDelayed,
    MichiganContentPreset::SharedFreightAmple,
    MichiganContentPreset::SharedFreightConstrained,
    MichiganContentPreset::StatewideBaseline,
    MichiganContentPreset::StatewideFreightConstraint,
    MichiganContentPreset::StatewidePackagingShortage,
    MichiganContentPreset::StatewideBoth,
];

impl MichiganContentPreset {
    #[must_use]
    pub const fn new_campaign(delivery: MichiganDeliveryPreset) -> Self {
        match delivery {
            MichiganDeliveryPreset::Standard => Self::FourWeekStandard,
            MichiganDeliveryPreset::Delayed => Self::FourWeekDelayed,
            MichiganDeliveryPreset::SharedFreightAmple => Self::SharedFreightAmple,
            MichiganDeliveryPreset::SharedFreightConstrained => Self::SharedFreightConstrained,
            MichiganDeliveryPreset::StatewideBaseline => Self::StatewideBaseline,
            MichiganDeliveryPreset::StatewideFreightConstraint => Self::StatewideFreightConstraint,
            MichiganDeliveryPreset::StatewidePackagingShortage => Self::StatewidePackagingShortage,
            MichiganDeliveryPreset::StatewideBoth => Self::StatewideBoth,
        }
    }
    #[must_use]
    pub const fn id(self) -> &'static str {
        self.delivery().id()
    }
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        MICHIGAN_CONTENT_PRESETS
            .into_iter()
            .find(|preset| preset.id() == id)
    }
    #[must_use]
    pub const fn delivery(self) -> MichiganDeliveryPreset {
        match self {
            Self::FourWeekStandard => MichiganDeliveryPreset::Standard,
            Self::FourWeekDelayed => MichiganDeliveryPreset::Delayed,
            Self::SharedFreightAmple => MichiganDeliveryPreset::SharedFreightAmple,
            Self::SharedFreightConstrained => MichiganDeliveryPreset::SharedFreightConstrained,
            Self::StatewideBaseline => MichiganDeliveryPreset::StatewideBaseline,
            Self::StatewideFreightConstraint => MichiganDeliveryPreset::StatewideFreightConstraint,
            Self::StatewidePackagingShortage => MichiganDeliveryPreset::StatewidePackagingShortage,
            Self::StatewideBoth => MichiganDeliveryPreset::StatewideBoth,
        }
    }
    #[must_use]
    pub const fn scenario(self) -> &'static str {
        MICHIGAN_COHORT_SCENARIO
    }
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::FourWeekStandard => "Michigan: standard delivery (four active cohorts)",
            Self::FourWeekDelayed => "Michigan: delayed delivery (four active cohorts)",
            Self::SharedFreightAmple => "Shared freight — ample",
            Self::SharedFreightConstrained => "Shared freight — constrained",
            Self::StatewideBaseline => "Statewide Michigan — baseline",
            Self::StatewideFreightConstraint => "Statewide Michigan — freight constraint",
            Self::StatewidePackagingShortage => "Statewide Michigan — packaging shortage",
            Self::StatewideBoth => "Statewide Michigan — both constraints",
        }
    }
    /// # Errors
    /// Refuses any changed source or foundation construction failure.
    pub fn admitted(
        self,
        catalog: &MichiganMaterialCatalog,
    ) -> Result<MichiganContentAdmission, MichiganContentError> {
        self.capture_admission(catalog)
    }
    /// Create a campaign from explicit, already validated numeric parameters.
    /// # Errors
    /// Refuses invalid material composition or observed source drift.
    pub fn create_foundation(
        self,
        catalog: &MichiganMaterialCatalog,
    ) -> Result<MaterialRuntimeFoundation, MichiganContentError> {
        self.build_foundation(catalog)
    }
    fn build_foundation(
        self,
        catalog: &MichiganMaterialCatalog,
    ) -> Result<MaterialRuntimeFoundation, MichiganContentError> {
        crate::sector_bundle::foundation::create_bundle_foundation(
            self.id(),
            self.delivery(),
            catalog,
        )
        .map_err(|_| MichiganContentError::Foundation)
    }
    fn capture_admission(
        self,
        catalog: &MichiganMaterialCatalog,
    ) -> Result<MichiganContentAdmission, MichiganContentError> {
        let catalog = catalog
            .with_preset(self.delivery())
            .map_err(|_| MichiganContentError::MaterialSource)?;
        let foundation = self.build_foundation(&catalog)?;
        let graph = foundation.graph_foundation();
        let component_identity = MaterialComponentIdentity::from_foundation(graph);
        let graph_digest = sha256_of(graph.canonical_bytes());
        let scenario_digest = sha256_of(graph.content_bundle().scenario_source_bytes());
        let staffing = foundation.labor().clone();
        let horizon_ticks = foundation.spec().horizon_ticks;
        let content_digest = foundation.spec().content_digest;
        let digest = foundation.digest();
        let canonical_bytes = foundation.canonical_bytes().to_vec();
        let register = foundation.initial_register().clone();
        let foundation_graph = foundation
            .into_session()
            .map_err(|_| MichiganContentError::Foundation)?
            .graph_session()
            .stable_graph_state()
            .map_err(|_| MichiganContentError::Foundation)?;
        Ok(MichiganContentAdmission {
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
            physical_projection: MichiganPhysicalProjection::Normalized,
        })
    }
}

/// Immutable admission evidence, shared by the writer and both read capabilities.
pub struct MichiganContentAdmission {
    pub(crate) preset: MichiganContentPreset,
    pub(crate) catalog: MichiganMaterialCatalog,
    pub(crate) horizon_ticks: u64,
    pub(crate) content_digest: [u8; 32],
    pub(crate) digest: [u8; 32],
    pub(crate) graph_digest: [u8; 32],
    pub(crate) scenario_digest: [u8; 32],
    pub(crate) canonical_bytes: Vec<u8>,
    pub(crate) register: MaterialWorldRegister,
    pub(crate) foundation_graph: StableGraphState,
    pub(crate) staffing: StaffingComposition,
    pub(crate) component_identity: MaterialComponentIdentity,
    pub(crate) physical_projection: MichiganPhysicalProjection,
}
impl MichiganContentAdmission {
    #[must_use]
    pub const fn preset(&self) -> MichiganContentPreset {
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
    ) -> Result<(), MichiganContentError> {
        if u64::try_from(horizon).ok() != Some(self.horizon_ticks)
            || tick > self.horizon_ticks
            || content != self.content_digest
            || foundation != self.digest
        {
            return Err(MichiganContentError::IdentityMismatch);
        }
        Ok(())
    }
    /// # Errors
    /// Refuses a graph or scenario from any other content revision.
    pub fn validate_graph(
        &self,
        foundation: &[u8],
        scenario: &[u8],
    ) -> Result<(), MichiganContentError> {
        if foundation != self.graph_digest || scenario != self.scenario_digest {
            return Err(MichiganContentError::IdentityMismatch);
        }
        Ok(())
    }
}

/// Admit only an exact versioned identity from the closed catalog.
/// # Errors
/// Refuses unknown presets, source failure or mismatched stored metadata.
pub fn admit_michigan_content(
    preset_id: &str,
    horizon: i64,
    content: &[u8],
    foundation: &[u8],
    tick: u64,
    foundation_bytes: &[u8],
) -> Result<MichiganContentAdmission, MichiganContentError> {
    let preset = validate_michigan_header(preset_id, horizon, content, foundation, tick)?;
    let wrapped = stored_defines_from_material_foundation(foundation_bytes)?;
    let decoded =
        crate::sector_bundle::foundation::decode_stored_bundle_defines(wrapped, sha256_of(wrapped))
            .map_err(|_| MichiganContentError::MaterialSource)?;
    let expected = preset.admitted(decoded.catalog())?;
    expected.validate_header(horizon, content, foundation, tick)?;
    if expected.canonical_bytes != foundation_bytes {
        return Err(MichiganContentError::IdentityMismatch);
    }
    Ok(expected)
}

/// Check only public header shape. This does not authenticate opaque material values.
/// `KnownPreview` reads grants and observed fields without material-read capability.
pub(crate) fn validate_michigan_header(
    preset_id: &str,
    horizon: i64,
    content: &[u8],
    foundation: &[u8],
    tick: u64,
) -> Result<MichiganContentPreset, MichiganContentError> {
    let preset =
        MichiganContentPreset::from_id(preset_id).ok_or(MichiganContentError::UnknownPreset)?;
    if !(1..=crate::michigan_material::MICHIGAN_MAX_HORIZON_PERIODS)
        .contains(&u64::try_from(horizon).unwrap_or(0))
        || tick > u64::try_from(horizon).unwrap_or(0)
        || content.len() != 32
        || foundation.len() != 32
        || content.iter().all(|b| *b == 0)
        || foundation.iter().all(|b| *b == 0)
    {
        return Err(MichiganContentError::IdentityMismatch);
    }
    Ok(preset)
}

/// Locate numeric authority inside the current canonical material/graph/content
/// nesting. Full reconstruction above subsequently compares every byte, including
/// all fields skipped here; locating a self-reported digest never admits content.
fn stored_defines_from_material_foundation(bytes: &[u8]) -> Result<&[u8], MichiganContentError> {
    use MichiganContentError::Foundation;
    fn take<'a>(input: &mut &'a [u8], n: usize) -> Result<&'a [u8], MichiganContentError> {
        let value = input.get(..n).ok_or(Foundation)?;
        *input = &input[n..];
        Ok(value)
    }
    fn field32<'a>(input: &mut &'a [u8]) -> Result<&'a [u8], MichiganContentError> {
        let length = u32::from_be_bytes(take(input, 4)?.try_into().map_err(|_| Foundation)?);
        take(input, usize::try_from(length).map_err(|_| Foundation)?)
    }
    fn field64<'a>(input: &mut &'a [u8]) -> Result<&'a [u8], MichiganContentError> {
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
