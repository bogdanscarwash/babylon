//! Stored executable bundle content for the separately admitted V3 campaigns.

use super::{
    codec::Cursor, compile_sector_bundles_v1, michigan_sector_bundles_v1, sha256_of,
    SectorBundleErrorV1, SectorBundleV1, HORIZON_TICKS, MAX_BUNDLE_BYTES,
};
use crate::{
    material_runtime::{MaterialFoundationSpecV2, MaterialRuntimeFoundationV2},
    michigan_cohorts::{michigan_cohorts_v2, MICHIGAN_COHORT_SESSION_V2},
    michigan_economy::observer_foundation_from_source,
    michigan_material::MichiganDeliveryPresetV1,
    FoundationContentBundleV2,
};

const DEFINES_DOMAIN: &[u8] = b"babylon.sector-bundle-defines.v1\0";
const CONTENT_DOMAIN: &[u8] = b"babylon.michigan-material-content.v3\0";
const MAX_OBSERVED_DEFINES: usize = 65_536;
const MAX_DEFINES_BYTES: usize = 4 * MAX_BUNDLE_BYTES + MAX_OBSERVED_DEFINES + 256;

/// Exact original observed definitions plus four independently checked bundles.
pub(crate) struct StoredSectorBundleDefinesV1 {
    observed_defines: Vec<u8>,
    bundles: Vec<SectorBundleV1>,
}
impl StoredSectorBundleDefinesV1 {
    pub(crate) fn observed_defines(&self) -> &[u8] {
        &self.observed_defines
    }
    pub(crate) fn bundles(&self) -> &[SectorBundleV1] {
        &self.bundles
    }
}

fn append_blob(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), SectorBundleErrorV1> {
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .map_err(|_| SectorBundleErrorV1::Bound)?
            .to_be_bytes(),
    );
    bytes.extend_from_slice(value);
    Ok(())
}

fn take_blob<'a>(cursor: &mut Cursor<'a>, bound: usize) -> Result<&'a [u8], SectorBundleErrorV1> {
    let count = usize::try_from(u32::from_be_bytes(cursor.array()?))
        .map_err(|_| SectorBundleErrorV1::Bound)?;
    if count == 0 || count > bound {
        return Err(SectorBundleErrorV1::Bound);
    }
    cursor.take(count)
}

fn encode_stored_defines(
    observed: &[u8],
    bundles: &[SectorBundleV1],
) -> Result<Vec<u8>, SectorBundleErrorV1> {
    if observed.is_empty() || observed.len() > MAX_OBSERVED_DEFINES || bundles.len() != 4 {
        return Err(SectorBundleErrorV1::Bound);
    }
    let mut ordered: Vec<_> = bundles.iter().collect();
    ordered.sort_unstable_by(|left, right| left.owner.subject.cmp(&right.owner.subject));
    if ordered
        .windows(2)
        .any(|pair| pair[0].owner.subject == pair[1].owner.subject)
    {
        return Err(SectorBundleErrorV1::ProcessOwnership);
    }
    let mut bytes = DEFINES_DOMAIN.to_vec();
    bytes.extend_from_slice(&1_u16.to_be_bytes());
    append_blob(&mut bytes, observed)?;
    bytes.extend_from_slice(&4_u16.to_be_bytes());
    for bundle in ordered {
        bytes.extend_from_slice(&bundle.sha256());
        append_blob(&mut bytes, bundle.canonical_bytes())?;
    }
    if bytes.len() > MAX_DEFINES_BYTES {
        return Err(SectorBundleErrorV1::Bound);
    }
    Ok(bytes)
}

/// Decode only after the caller admits the complete definitions digest.
/// Child hashes are part of that admitted envelope, never substitutes for it.
pub(crate) fn decode_stored_bundle_defines_v1(
    bytes: &[u8],
    expected_digest: [u8; 32],
) -> Result<StoredSectorBundleDefinesV1, SectorBundleErrorV1> {
    if bytes.len() > MAX_DEFINES_BYTES {
        return Err(SectorBundleErrorV1::Bound);
    }
    if sha256_of(bytes) != expected_digest {
        return Err(SectorBundleErrorV1::Digest);
    }
    let mut cursor = Cursor::new(bytes);
    if cursor.take(DEFINES_DOMAIN.len())? != DEFINES_DOMAIN {
        return Err(SectorBundleErrorV1::WireDomain);
    }
    if u16::from_be_bytes(cursor.array()?) != 1 {
        return Err(SectorBundleErrorV1::WireVersion);
    }
    let observed_defines = take_blob(&mut cursor, MAX_OBSERVED_DEFINES)?.to_vec();
    if cursor.count(4)? != 4 {
        return Err(SectorBundleErrorV1::Coverage);
    }
    let mut bundles = Vec::new();
    for _ in 0..4 {
        let expected = cursor.array()?;
        let bytes = take_blob(&mut cursor, MAX_BUNDLE_BYTES)?;
        bundles.push(SectorBundleV1::decode(bytes, expected)?);
    }
    if !cursor.finished() {
        return Err(SectorBundleErrorV1::WireTrailing);
    }
    if encode_stored_defines(&observed_defines, &bundles)? != bytes {
        return Err(SectorBundleErrorV1::WireNoncanonical);
    }
    Ok(StoredSectorBundleDefinesV1 {
        observed_defines,
        bundles,
    })
}

/// Only new V3 creation uses this compiler. V1/V2 factories remain unchanged.
pub(crate) fn create_bundle_foundation_v3(
    preset_id: &str,
    delivery: MichiganDeliveryPresetV1,
) -> Result<MaterialRuntimeFoundationV2, SectorBundleErrorV1> {
    let expected = match delivery {
        MichiganDeliveryPresetV1::Standard => "michigan-material-standard-v3",
        MichiganDeliveryPresetV1::Delayed => "michigan-material-delayed-v3",
    };
    if preset_id != expected {
        return Err(SectorBundleErrorV1::Preset);
    }
    let observed = michigan_cohorts_v2().map_err(|_| SectorBundleErrorV1::Source)?;
    let defines = encode_stored_defines(observed.defines_bytes(), michigan_sector_bundles_v1()?)?;
    let decoded = decode_stored_bundle_defines_v1(&defines, sha256_of(&defines))?;
    if decoded.observed_defines() != observed.defines_bytes() {
        return Err(SectorBundleErrorV1::Source);
    }
    // The material state is compiled from the exact decoded content retained in
    // this foundation. The independent predecessor material factory is not used.
    let state = compile_sector_bundles_v1(decoded.bundles(), delivery)?;
    let (graph, bundle) = observer_foundation_from_source(
        observed.scenario_source(),
        MICHIGAN_COHORT_SESSION_V2,
        &defines,
        FoundationContentBundleV2::try_new,
    )
    .map_err(|_| SectorBundleErrorV1::Foundation)?;
    let mut identity = CONTENT_DOMAIN.to_vec();
    identity.extend_from_slice(&sha256_of(&defines));
    identity.extend_from_slice(&sha256_of(observed.scenario_source().as_bytes()));
    MaterialRuntimeFoundationV2::capture_v2(
        graph,
        bundle,
        state,
        MaterialFoundationSpecV2 {
            preset_id: preset_id.to_owned(),
            horizon_ticks: HORIZON_TICKS,
            content_digest: sha256_of(&identity),
        },
    )
    .map_err(|_| SectorBundleErrorV1::Foundation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_bundle_set_is_canonical_complete_and_bound_to_its_admitted_digest() {
        let observed = michigan_cohorts_v2().unwrap();
        let bundles = michigan_sector_bundles_v1().unwrap();
        let bytes = encode_stored_defines(observed.defines_bytes(), bundles).unwrap();
        let mut reversed = bundles.to_vec();
        reversed.reverse();
        assert_eq!(
            encode_stored_defines(observed.defines_bytes(), &reversed).unwrap(),
            bytes
        );
        let decoded = decode_stored_bundle_defines_v1(&bytes, sha256_of(&bytes)).unwrap();
        assert_eq!(decoded.observed_defines(), observed.defines_bytes());
        assert_eq!(decoded.bundles(), bundles);
        let mut changed = bytes.clone();
        let child_hash = DEFINES_DOMAIN.len() + 2 + 4 + observed.defines_bytes().len() + 2;
        changed[child_hash] ^= 1;
        assert!(matches!(
            decode_stored_bundle_defines_v1(&changed, sha256_of(&bytes)),
            Err(SectorBundleErrorV1::Digest)
        ));
        // Even an independently allowed outer envelope cannot bless a wrong child hash.
        assert!(matches!(
            decode_stored_bundle_defines_v1(&changed, sha256_of(&changed)),
            Err(SectorBundleErrorV1::Digest)
        ));
        assert!(encode_stored_defines(observed.defines_bytes(), &bundles[..3]).is_err());
        let mut duplicate = bundles.to_vec();
        duplicate[1] = duplicate[0].clone();
        assert!(matches!(
            encode_stored_defines(observed.defines_bytes(), &duplicate),
            Err(SectorBundleErrorV1::ProcessOwnership)
        ));
    }

    #[test]
    fn actual_v3_initial_register_is_compiled_from_its_exact_stored_bundle_rows() {
        for (id, delivery) in [
            (
                "michigan-material-standard-v3",
                MichiganDeliveryPresetV1::Standard,
            ),
            (
                "michigan-material-delayed-v3",
                MichiganDeliveryPresetV1::Delayed,
            ),
        ] {
            let foundation = create_bundle_foundation_v3(id, delivery).unwrap();
            let graph = foundation.graph_foundation();
            let decoded = decode_stored_bundle_defines_v1(
                graph.content_bundle().defines_bytes(),
                graph.content_digest().defines_hash,
            )
            .unwrap();
            let material = compile_sector_bundles_v1(decoded.bundles(), delivery).unwrap();
            assert_eq!(&material, foundation.initial_register().state());
            assert_eq!(foundation.spec().preset_id, id);
            assert_eq!(foundation.spec().horizon_ticks, 16);
        }
        assert!(matches!(
            create_bundle_foundation_v3(
                "michigan-material-standard-v2",
                MichiganDeliveryPresetV1::Standard
            ),
            Err(SectorBundleErrorV1::Preset)
        ));
    }

    #[test]
    fn stored_defines_reject_unknown_versions_truncation_trailing_and_noncanonical_order() {
        let observed = michigan_cohorts_v2().unwrap();
        let bundles = michigan_sector_bundles_v1().unwrap();
        let bytes = encode_stored_defines(observed.defines_bytes(), bundles).unwrap();
        let mut changed = bytes.clone();
        changed[DEFINES_DOMAIN.len() + 1] = 2;
        assert!(matches!(
            decode_stored_bundle_defines_v1(&changed, sha256_of(&changed)),
            Err(SectorBundleErrorV1::WireVersion)
        ));
        changed[0] ^= 1;
        assert!(matches!(
            decode_stored_bundle_defines_v1(&changed, sha256_of(&changed)),
            Err(SectorBundleErrorV1::WireDomain)
        ));
        for length in [0, DEFINES_DOMAIN.len(), bytes.len() - 1] {
            let truncated = &bytes[..length];
            assert!(matches!(
                decode_stored_bundle_defines_v1(truncated, sha256_of(truncated)),
                Err(SectorBundleErrorV1::WireTruncated)
            ));
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(matches!(
            decode_stored_bundle_defines_v1(&trailing, sha256_of(&trailing)),
            Err(SectorBundleErrorV1::WireTrailing)
        ));
        let entries_offset = DEFINES_DOMAIN.len() + 2 + 4 + observed.defines_bytes().len() + 2;
        let mut noncanonical = bytes[..entries_offset].to_vec();
        for bundle in bundles.iter().rev() {
            noncanonical.extend_from_slice(&bundle.sha256());
            append_blob(&mut noncanonical, bundle.canonical_bytes()).unwrap();
        }
        assert!(matches!(
            decode_stored_bundle_defines_v1(&noncanonical, sha256_of(&noncanonical)),
            Err(SectorBundleErrorV1::WireNoncanonical)
        ));
    }
}
