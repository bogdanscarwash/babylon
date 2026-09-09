//! Eight-family committed-material successor. Component row codecs remain exact.

use crate::{
    committed_tick_envelope::{CommittedTickEnvelopeV2, CommittedTickRowFamiliesV2},
    identity::CampaignId,
    runtime::RustPersistenceRuntimeErrorV2,
    tick_commit_claim::TickCommitClaimV1,
};
use babylon_kernel::sha256_of;
use babylon_tick::material_replay::IdentifiedMaterialTickV3;

const DOMAIN: &[u8] = b"babylon.committed-material-tick.v3\0";
pub const MAX_COMMITTED_MATERIAL_TICK_BYTES_V3: usize = 67_108_864;

/// Exact closed envelope, with material register and receipts inseparable from its claim.
#[derive(Debug, PartialEq, Eq)]
pub struct CommittedMaterialTickEnvelopeV3 {
    bytes: Vec<u8>,
    digest: [u8; 32],
}
impl CommittedMaterialTickEnvelopeV3 {
    /// Frame six typed component families followed by register and material receipt families.
    /// # Errors
    /// Refuses component ordering/shape, aggregate bounds, hash mismatch and allocation failure.
    pub fn compose(
        campaign: CampaignId,
        identity: &IdentifiedMaterialTickV3,
        families: CommittedTickRowFamiliesV2,
        register: &[u8],
        receipts: &[u8],
    ) -> Result<Self, RustPersistenceRuntimeErrorV2> {
        if sha256_of(receipts) != identity.receipt_digest() {
            return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
        }
        // Reuse the existing six-family shape and key validation. Its temporary
        // container is never persisted or acknowledged as a V2 tick.
        let component = CommittedTickEnvelopeV2::compose(
            TickCommitClaimV1::compose(
                campaign,
                identity.resolve_tick(),
                identity.tick_content_hash(),
            ),
            families,
        )
        .map_err(RustPersistenceRuntimeErrorV2::SemanticEnvelope)?;
        let capacity = component
            .row_families()
            .iter()
            .try_fold(DOMAIN.len() + 4 + 16 + 8 + 32 + 8 * 9, |total, batch| {
                batch.rows().iter().try_fold(total, |total, row| {
                    total
                        .checked_add(8)
                        .and_then(|n| n.checked_add(row.key().len()))
                        .and_then(|n| n.checked_add(row.payload().len()))
                        .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)
                })
            })?
            .checked_add(16)
            .and_then(|n| n.checked_add(register.len()))
            .and_then(|n| n.checked_add(receipts.len()))
            .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?;
        if capacity > MAX_COMMITTED_MATERIAL_TICK_BYTES_V3 {
            return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
        }
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(capacity).map_err(|_| {
            RustPersistenceRuntimeErrorV2::Allocation {
                field: "material envelope",
                requested: capacity,
            }
        })?;
        bytes.extend_from_slice(DOMAIN);
        bytes.extend_from_slice(&3_u32.to_be_bytes());
        bytes.extend_from_slice(campaign.canonical_bytes());
        bytes.extend_from_slice(&identity.resolve_tick().to_be_bytes());
        bytes.extend_from_slice(identity.tick_content_hash().as_bytes());
        for (index, batch) in component.row_families().iter().enumerate() {
            bytes.push(
                u8::try_from(index + 1)
                    .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?,
            );
            append_u64(&mut bytes, batch.rows().len())?;
            for row in batch.rows() {
                append_row(&mut bytes, row.key(), row.payload())?;
            }
        }
        for (tag, payload) in [(7, register), (8, receipts)] {
            bytes.push(tag);
            bytes.extend_from_slice(&1_u64.to_be_bytes());
            append_row(&mut bytes, &[], payload)?;
        }
        if bytes.len() != capacity {
            return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
        }
        let digest = sha256_of(&bytes);
        Ok(Self { bytes, digest })
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}
fn append_u64(bytes: &mut Vec<u8>, value: usize) -> Result<(), RustPersistenceRuntimeErrorV2> {
    bytes.extend_from_slice(
        &u64::try_from(value)
            .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?
            .to_be_bytes(),
    );
    Ok(())
}
fn append_row(
    bytes: &mut Vec<u8>,
    key: &[u8],
    payload: &[u8],
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    for value in [key, payload] {
        bytes.extend_from_slice(
            &u32::try_from(value.len())
                .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?
                .to_be_bytes(),
        );
        bytes.extend_from_slice(value);
    }
    Ok(())
}
