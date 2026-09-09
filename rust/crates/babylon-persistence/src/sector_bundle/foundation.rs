//! One current stored authority for both regional and statewide foundations.
use super::staffing::StoredStaffingV2;
use super::{
    codec::Cursor, compile_sector_bundles_v2, michigan_sector_bundles_v2, sha256_of,
    SectorBundleErrorV2, SectorBundleV2, MAX_BUNDLE_BYTES, MICHIGAN_MAX_HORIZON_PERIODS_V1,
};
use crate::{
    material_runtime::{MaterialFoundationSpecV2, MaterialRuntimeFoundationV2},
    michigan_cohorts::MICHIGAN_COHORT_SESSION_V2,
    michigan_economy::observer_foundation_from_source,
    michigan_material::{
        MichiganDeliveryPresetV1, MichiganMaterialCatalogV1, MAX_MICHIGAN_CAPTURED_CONTENT_BYTES_V2,
    },
    FoundationContentBundleV2,
};
use babylon_tick::material_replay::MaterialLaborV1;
const DEFINES_DOMAIN: &[u8] = b"babylon.sector-bundle-defines.v4\0";
const CONTENT_DOMAIN: &[u8] = b"babylon.michigan-material-content.v7\0";
const MAX_DEFINES_BYTES: usize = 64 * 1024 * 1024;
const MAX_BUNDLES: usize = 1024;
const MAX_STAFFING_BYTES: usize = 1_048_576;
pub(crate) struct StoredSectorBundleDefinesV4 {
    bundles: Vec<SectorBundleV2>,
    staffing: StoredStaffingV2,
    catalog: MichiganMaterialCatalogV1,
}
impl StoredSectorBundleDefinesV4 {
    pub(crate) fn catalog(&self) -> &MichiganMaterialCatalogV1 {
        &self.catalog
    }
    pub(crate) fn labor(&self) -> Result<MaterialLaborV1, SectorBundleErrorV2> {
        Ok(MaterialLaborV1::Staffed(self.staffing.composition()?))
    }
    pub(crate) fn scenario(&self) -> &str {
        self.catalog.graph_scenario_source()
    }
    pub(crate) fn bundles(&self) -> &[SectorBundleV2] {
        &self.bundles
    }
}
fn append_blob(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), SectorBundleErrorV2> {
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .map_err(|_| SectorBundleErrorV2::Bound)?
            .to_be_bytes(),
    );
    bytes.extend_from_slice(value);
    Ok(())
}
fn take_blob<'a>(cursor: &mut Cursor<'a>, bound: usize) -> Result<&'a [u8], SectorBundleErrorV2> {
    let count = usize::try_from(u32::from_be_bytes(cursor.array()?))
        .map_err(|_| SectorBundleErrorV2::Bound)?;
    if count == 0 || count > bound {
        return Err(SectorBundleErrorV2::Bound);
    }
    cursor.take(count)
}
fn encode_stored_defines(
    catalog: &MichiganMaterialCatalogV1,
    bundles: &[SectorBundleV2],
    staffing: &StoredStaffingV2,
) -> Result<Vec<u8>, SectorBundleErrorV2> {
    if bundles.is_empty() || bundles.len() > MAX_BUNDLES || bundles.len() != catalog.owners().len()
    {
        return Err(SectorBundleErrorV2::Coverage);
    }
    let mut ordered: Vec<_> = bundles.iter().collect();
    ordered.sort_by(|a, b| a.owner.subject.cmp(&b.owner.subject));
    if ordered
        .windows(2)
        .any(|pair| pair[0].owner.subject == pair[1].owner.subject)
    {
        return Err(SectorBundleErrorV2::ProcessOwnership);
    }
    let mut bytes = DEFINES_DOMAIN.to_vec();
    bytes.extend_from_slice(&4_u16.to_be_bytes());
    bytes.extend_from_slice(&babylon_kernel::clock::DAYS_PER_TICK.to_be_bytes());
    append_blob(&mut bytes, catalog.defines_bytes())?;
    bytes.extend_from_slice(
        &u16::try_from(ordered.len())
            .map_err(|_| SectorBundleErrorV2::Bound)?
            .to_be_bytes(),
    );
    for bundle in ordered {
        bytes.extend_from_slice(&bundle.sha256());
        append_blob(&mut bytes, bundle.canonical_bytes())?;
    }
    append_blob(&mut bytes, &staffing.encode()?)?;
    if bytes.len() > MAX_DEFINES_BYTES {
        return Err(SectorBundleErrorV2::Bound);
    }
    Ok(bytes)
}
pub(crate) fn decode_stored_bundle_defines_v4(
    bytes: &[u8],
    expected_digest: [u8; 32],
) -> Result<StoredSectorBundleDefinesV4, SectorBundleErrorV2> {
    if bytes.len() > MAX_DEFINES_BYTES {
        return Err(SectorBundleErrorV2::Bound);
    }
    if sha256_of(bytes) != expected_digest {
        return Err(SectorBundleErrorV2::Digest);
    }
    let mut cursor = Cursor::new(bytes);
    if cursor.take(DEFINES_DOMAIN.len())? != DEFINES_DOMAIN {
        return Err(SectorBundleErrorV2::WireDomain);
    }
    if u16::from_be_bytes(cursor.array()?) != 4 {
        return Err(SectorBundleErrorV2::WireVersion);
    }
    if u64::from_be_bytes(cursor.array()?) != babylon_kernel::clock::DAYS_PER_TICK {
        return Err(SectorBundleErrorV2::Preset);
    }
    let catalog = MichiganMaterialCatalogV1::from_stored_defines(take_blob(
        &mut cursor,
        MAX_MICHIGAN_CAPTURED_CONTENT_BYTES_V2,
    )?)
    .map_err(|_| SectorBundleErrorV2::Source)?;
    let count = cursor.count(MAX_BUNDLES)?;
    let mut bundles = Vec::with_capacity(count);
    for _ in 0..count {
        let expected = cursor.array()?;
        bundles.push(SectorBundleV2::decode(
            take_blob(&mut cursor, MAX_BUNDLE_BYTES)?,
            expected,
        )?);
    }
    let staffing = StoredStaffingV2::decode(take_blob(&mut cursor, MAX_STAFFING_BYTES)?, &catalog)?;
    if !cursor.finished() {
        return Err(SectorBundleErrorV2::WireTrailing);
    }
    if bundles != michigan_sector_bundles_v2(&catalog)? {
        return Err(SectorBundleErrorV2::Source);
    }
    if encode_stored_defines(&catalog, &bundles, &staffing)? != bytes {
        return Err(SectorBundleErrorV2::WireNoncanonical);
    }
    Ok(StoredSectorBundleDefinesV4 {
        bundles,
        staffing,
        catalog,
    })
}
/// No current source artifact is read while reconstructing saved authority.
pub(crate) fn validate_stored_material_authority(
    graph: &crate::CampaignFoundationV1,
    register: &babylon_tick::material_world::MaterialWorldRegisterV3,
    spec: &MaterialFoundationSpecV2,
) -> Result<MaterialLaborV1, SectorBundleErrorV2> {
    let delivery =
        MichiganDeliveryPresetV1::from_id(&spec.preset_id).ok_or(SectorBundleErrorV2::Preset)?;
    if !(1..=MICHIGAN_MAX_HORIZON_PERIODS_V1).contains(&spec.horizon_ticks) {
        return Err(SectorBundleErrorV2::Preset);
    }
    let decoded = decode_stored_bundle_defines_v4(
        graph.content_bundle().defines_bytes(),
        graph.content_digest().defines_hash,
    )?;
    if spec.horizon_ticks != decoded.catalog().horizon_ticks()
        || decoded.catalog().preset() != delivery
        || decoded.scenario().as_bytes() != graph.content_bundle().scenario_source_bytes()
        || &compile_sector_bundles_v2(decoded.bundles(), delivery, decoded.catalog())?
            != register.state()
    {
        return Err(SectorBundleErrorV2::Foundation);
    }
    let mut identity = CONTENT_DOMAIN.to_vec();
    identity.extend_from_slice(&graph.content_digest().defines_hash);
    identity.extend_from_slice(&sha256_of(graph.content_bundle().scenario_source_bytes()));
    if sha256_of(&identity) != spec.content_digest {
        return Err(SectorBundleErrorV2::Digest);
    }
    decoded.labor()
}
pub(crate) fn create_bundle_foundation_v7(
    preset_id: &str,
    delivery: MichiganDeliveryPresetV1,
    catalog: &MichiganMaterialCatalogV1,
) -> Result<MaterialRuntimeFoundationV2, SectorBundleErrorV2> {
    if preset_id != delivery.id() {
        return Err(SectorBundleErrorV2::Preset);
    }
    let catalog = catalog
        .with_preset(delivery)
        .map_err(|_| SectorBundleErrorV2::Preset)?;
    let defines = encode_stored_defines(
        &catalog,
        &michigan_sector_bundles_v2(&catalog)?,
        &StoredStaffingV2::authored(&catalog)?,
    )?;
    let decoded = decode_stored_bundle_defines_v4(&defines, sha256_of(&defines))?;
    let state = compile_sector_bundles_v2(decoded.bundles(), delivery, decoded.catalog())?;
    let scenario = decoded.scenario();
    let (graph, bundle) = observer_foundation_from_source(
        scenario,
        MICHIGAN_COHORT_SESSION_V2,
        &defines,
        FoundationContentBundleV2::try_new,
    )
    .map_err(|_| SectorBundleErrorV2::Foundation)?;
    let mut identity = CONTENT_DOMAIN.to_vec();
    identity.extend_from_slice(&sha256_of(&defines));
    identity.extend_from_slice(&sha256_of(scenario.as_bytes()));
    MaterialRuntimeFoundationV2::capture_v2(
        graph,
        bundle,
        state,
        MaterialFoundationSpecV2 {
            preset_id: preset_id.to_owned(),
            horizon_ticks: catalog.horizon_ticks(),
            content_digest: sha256_of(&identity),
        },
    )
    .map_err(|_| SectorBundleErrorV2::Foundation)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn bytes() -> Vec<u8> {
        let c = crate::test_support::catalog();
        encode_stored_defines(
            &c,
            &michigan_sector_bundles_v2(&c).unwrap(),
            &StoredStaffingV2::authored(&c).unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn stored_authority_binds_capture_children_timebase_and_canonical_ownership() {
        let bytes = bytes();
        let decoded = decode_stored_bundle_defines_v4(&bytes, sha256_of(&bytes)).unwrap();
        assert_eq!(decoded.bundles().len(), 4);
        assert_eq!(
            decoded.scenario(),
            decoded.catalog().graph_scenario_source()
        );
        let mut changed = bytes.clone();
        changed[DEFINES_DOMAIN.len() + 2 + 7] ^= 1;
        assert_eq!(
            decode_stored_bundle_defines_v4(&changed, sha256_of(&bytes)).err(),
            Some(SectorBundleErrorV2::Digest)
        );
        assert_eq!(
            decode_stored_bundle_defines_v4(&changed, sha256_of(&changed)).err(),
            Some(SectorBundleErrorV2::Preset)
        );
        let offset = DEFINES_DOMAIN.len() + 2 + 8 + 4 + decoded.catalog().defines_bytes().len() + 2;
        let mut changed = bytes.clone();
        changed[offset] ^= 1;
        assert_eq!(
            decode_stored_bundle_defines_v4(&changed, sha256_of(&changed)).err(),
            Some(SectorBundleErrorV2::Digest)
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
            decode_stored_bundle_defines_v4(&changed, sha256_of(&changed)).err(),
            Some(SectorBundleErrorV2::WireVersion)
        );
        for length in [0, DEFINES_DOMAIN.len(), bytes.len() - 1] {
            let part = &bytes[..length];
            assert_eq!(
                decode_stored_bundle_defines_v4(part, sha256_of(part)).err(),
                Some(SectorBundleErrorV2::WireTruncated)
            );
        }
        let mut changed = bytes;
        changed.push(0);
        assert_eq!(
            decode_stored_bundle_defines_v4(&changed, sha256_of(&changed)).err(),
            Some(SectorBundleErrorV2::WireTrailing)
        );
    }
}
