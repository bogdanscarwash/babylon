//! One current stored authority for both regional and statewide foundations.
use super::staffing::StoredStaffing;
use super::{
    codec::Cursor, compile_sector_bundles, michigan_sector_bundles, sha256_of, SectorBundle,
    SectorBundleError, MAX_BUNDLE_BYTES, MICHIGAN_MAX_HORIZON_PERIODS,
};
use crate::{
    material_runtime::{MaterialFoundationSpec, MaterialRuntimeFoundation},
    michigan_cohorts::MICHIGAN_COHORT_SESSION,
    michigan_economy::observer_foundation_from_source,
    michigan_material::{
        MichiganDeliveryPreset, MichiganMaterialCatalog, MAX_MICHIGAN_CAPTURED_CONTENT_BYTES,
    },
    FoundationContentBundle,
};
use babylon_tick::material_staffing::StaffingComposition;
const DEFINES_DOMAIN: &[u8] = b"babylon.sector-bundle-defines.v4\0";
const CONTENT_DOMAIN: &[u8] = b"babylon.michigan-material-content.v7\0";
const MAX_DEFINES_BYTES: usize = 64 * 1024 * 1024;
const MAX_BUNDLES: usize = 1024;
const MAX_STAFFING_BYTES: usize = 1_048_576;
pub(crate) struct StoredSectorBundleDefines {
    bundles: Vec<SectorBundle>,
    staffing: StoredStaffing,
    catalog: MichiganMaterialCatalog,
}
impl StoredSectorBundleDefines {
    pub(crate) fn catalog(&self) -> &MichiganMaterialCatalog {
        &self.catalog
    }
    pub(crate) fn labor(&self) -> Result<StaffingComposition, SectorBundleError> {
        self.staffing.composition()
    }
    pub(crate) fn scenario(&self) -> &str {
        self.catalog.graph_scenario_source()
    }
    pub(crate) fn bundles(&self) -> &[SectorBundle] {
        &self.bundles
    }
}
fn append_blob(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), SectorBundleError> {
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .map_err(|_| SectorBundleError::Bound)?
            .to_be_bytes(),
    );
    bytes.extend_from_slice(value);
    Ok(())
}
fn take_blob<'a>(cursor: &mut Cursor<'a>, bound: usize) -> Result<&'a [u8], SectorBundleError> {
    let count = usize::try_from(u32::from_be_bytes(cursor.array()?))
        .map_err(|_| SectorBundleError::Bound)?;
    if count == 0 || count > bound {
        return Err(SectorBundleError::Bound);
    }
    cursor.take(count)
}
fn encode_stored_defines(
    catalog: &MichiganMaterialCatalog,
    bundles: &[SectorBundle],
    staffing: &StoredStaffing,
) -> Result<Vec<u8>, SectorBundleError> {
    if bundles.is_empty() || bundles.len() > MAX_BUNDLES || bundles.len() != catalog.owners().len()
    {
        return Err(SectorBundleError::Coverage);
    }
    let mut ordered: Vec<_> = bundles.iter().collect();
    ordered.sort_by(|a, b| a.owner.subject.cmp(&b.owner.subject));
    if ordered
        .windows(2)
        .any(|pair| pair[0].owner.subject == pair[1].owner.subject)
    {
        return Err(SectorBundleError::ProcessOwnership);
    }
    let mut bytes = DEFINES_DOMAIN.to_vec();
    bytes.extend_from_slice(&4_u16.to_be_bytes());
    bytes.extend_from_slice(&babylon_kernel::clock::DAYS_PER_TICK.to_be_bytes());
    append_blob(&mut bytes, catalog.defines_bytes())?;
    bytes.extend_from_slice(
        &u16::try_from(ordered.len())
            .map_err(|_| SectorBundleError::Bound)?
            .to_be_bytes(),
    );
    for bundle in ordered {
        bytes.extend_from_slice(&bundle.sha256());
        append_blob(&mut bytes, bundle.canonical_bytes())?;
    }
    append_blob(&mut bytes, &staffing.encode()?)?;
    if bytes.len() > MAX_DEFINES_BYTES {
        return Err(SectorBundleError::Bound);
    }
    Ok(bytes)
}
pub(crate) fn decode_stored_bundle_defines(
    bytes: &[u8],
    expected_digest: [u8; 32],
) -> Result<StoredSectorBundleDefines, SectorBundleError> {
    if bytes.len() > MAX_DEFINES_BYTES {
        return Err(SectorBundleError::Bound);
    }
    if sha256_of(bytes) != expected_digest {
        return Err(SectorBundleError::Digest);
    }
    let mut cursor = Cursor::new(bytes);
    if cursor.take(DEFINES_DOMAIN.len())? != DEFINES_DOMAIN {
        return Err(SectorBundleError::WireDomain);
    }
    if u16::from_be_bytes(cursor.array()?) != 4 {
        return Err(SectorBundleError::WireVersion);
    }
    if u64::from_be_bytes(cursor.array()?) != babylon_kernel::clock::DAYS_PER_TICK {
        return Err(SectorBundleError::Preset);
    }
    let catalog = MichiganMaterialCatalog::from_stored_defines(take_blob(
        &mut cursor,
        MAX_MICHIGAN_CAPTURED_CONTENT_BYTES,
    )?)
    .map_err(|_| SectorBundleError::Source)?;
    let count = cursor.count(MAX_BUNDLES)?;
    let mut bundles = Vec::with_capacity(count);
    for _ in 0..count {
        let expected = cursor.array()?;
        bundles.push(SectorBundle::decode(
            take_blob(&mut cursor, MAX_BUNDLE_BYTES)?,
            expected,
        )?);
    }
    let staffing = StoredStaffing::decode(take_blob(&mut cursor, MAX_STAFFING_BYTES)?, &catalog)?;
    if !cursor.finished() {
        return Err(SectorBundleError::WireTrailing);
    }
    if bundles != michigan_sector_bundles(&catalog)? {
        return Err(SectorBundleError::Source);
    }
    if encode_stored_defines(&catalog, &bundles, &staffing)? != bytes {
        return Err(SectorBundleError::WireNoncanonical);
    }
    Ok(StoredSectorBundleDefines {
        bundles,
        staffing,
        catalog,
    })
}
/// No current source artifact is read while reconstructing saved authority.
pub(crate) fn validate_stored_material_authority(
    graph: &crate::CampaignFoundation,
    register: &babylon_tick::material_world::MaterialWorldRegister,
    spec: &MaterialFoundationSpec,
) -> Result<StaffingComposition, SectorBundleError> {
    let delivery =
        MichiganDeliveryPreset::from_id(&spec.preset_id).ok_or(SectorBundleError::Preset)?;
    if !(1..=MICHIGAN_MAX_HORIZON_PERIODS).contains(&spec.horizon_ticks) {
        return Err(SectorBundleError::Preset);
    }
    let decoded = decode_stored_bundle_defines(
        graph.content_bundle().defines_bytes(),
        graph.content_digest().defines_hash,
    )?;
    if spec.horizon_ticks != decoded.catalog().horizon_ticks()
        || decoded.catalog().preset() != delivery
        || decoded.scenario().as_bytes() != graph.content_bundle().scenario_source_bytes()
        || &compile_sector_bundles(decoded.bundles(), delivery, decoded.catalog())?
            != register.state()
    {
        return Err(SectorBundleError::Foundation);
    }
    let mut identity = CONTENT_DOMAIN.to_vec();
    identity.extend_from_slice(&graph.content_digest().defines_hash);
    identity.extend_from_slice(&sha256_of(graph.content_bundle().scenario_source_bytes()));
    if sha256_of(&identity) != spec.content_digest {
        return Err(SectorBundleError::Digest);
    }
    decoded.labor()
}
pub(crate) fn create_bundle_foundation(
    preset_id: &str,
    delivery: MichiganDeliveryPreset,
    catalog: &MichiganMaterialCatalog,
) -> Result<MaterialRuntimeFoundation, SectorBundleError> {
    if preset_id != delivery.id() {
        return Err(SectorBundleError::Preset);
    }
    let catalog = catalog
        .with_preset(delivery)
        .map_err(|_| SectorBundleError::Preset)?;
    let defines = encode_stored_defines(
        &catalog,
        &michigan_sector_bundles(&catalog)?,
        &StoredStaffing::authored(&catalog)?,
    )?;
    let decoded = decode_stored_bundle_defines(&defines, sha256_of(&defines))?;
    let state = compile_sector_bundles(decoded.bundles(), delivery, decoded.catalog())?;
    let scenario = decoded.scenario();
    let (graph, bundle) = observer_foundation_from_source(
        scenario,
        MICHIGAN_COHORT_SESSION,
        &defines,
        FoundationContentBundle::try_new,
    )
    .map_err(|_| SectorBundleError::Foundation)?;
    let mut identity = CONTENT_DOMAIN.to_vec();
    identity.extend_from_slice(&sha256_of(&defines));
    identity.extend_from_slice(&sha256_of(scenario.as_bytes()));
    MaterialRuntimeFoundation::capture(
        graph,
        bundle,
        state,
        MaterialFoundationSpec {
            preset_id: preset_id.to_owned(),
            horizon_ticks: catalog.horizon_ticks(),
            content_digest: sha256_of(&identity),
        },
    )
    .map_err(|_| SectorBundleError::Foundation)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn bytes() -> Vec<u8> {
        let c = crate::test_support::catalog();
        encode_stored_defines(
            &c,
            &michigan_sector_bundles(&c).unwrap(),
            &StoredStaffing::authored(&c).unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn stored_authority_binds_capture_children_timebase_and_canonical_ownership() {
        let bytes = bytes();
        let decoded = decode_stored_bundle_defines(&bytes, sha256_of(&bytes)).unwrap();
        assert_eq!(decoded.bundles().len(), 4);
        assert_eq!(
            decoded.scenario(),
            decoded.catalog().graph_scenario_source()
        );
        let mut changed = bytes.clone();
        changed[DEFINES_DOMAIN.len() + 2 + 7] ^= 1;
        assert_eq!(
            decode_stored_bundle_defines(&changed, sha256_of(&bytes)).err(),
            Some(SectorBundleError::Digest)
        );
        assert_eq!(
            decode_stored_bundle_defines(&changed, sha256_of(&changed)).err(),
            Some(SectorBundleError::Preset)
        );
        let offset = DEFINES_DOMAIN.len() + 2 + 8 + 4 + decoded.catalog().defines_bytes().len() + 2;
        let mut changed = bytes.clone();
        changed[offset] ^= 1;
        assert_eq!(
            decode_stored_bundle_defines(&changed, sha256_of(&changed)).err(),
            Some(SectorBundleError::Digest)
        );
        let mut reversed = decoded.bundles().to_vec();
        reversed.reverse();
        assert_eq!(
            encode_stored_defines(decoded.catalog(), &reversed, &decoded.staffing).unwrap(),
            bytes
        );
        assert!(
            encode_stored_defines(decoded.catalog(), &reversed[..3], &decoded.staffing).is_err()
        );
    }
    #[test]
    fn stored_authority_refuses_other_versions_truncation_and_trailing_bytes() {
        let bytes = bytes();
        let mut changed = bytes.clone();
        changed[DEFINES_DOMAIN.len() + 1] = 3;
        assert_eq!(
            decode_stored_bundle_defines(&changed, sha256_of(&changed)).err(),
            Some(SectorBundleError::WireVersion)
        );
        for length in [0, DEFINES_DOMAIN.len(), bytes.len() - 1] {
            let part = &bytes[..length];
            assert_eq!(
                decode_stored_bundle_defines(part, sha256_of(part)).err(),
                Some(SectorBundleError::WireTruncated)
            );
        }
        let mut changed = bytes;
        changed.push(0);
        assert_eq!(
            decode_stored_bundle_defines(&changed, sha256_of(&changed)).err(),
            Some(SectorBundleError::WireTrailing)
        );
    }
}
