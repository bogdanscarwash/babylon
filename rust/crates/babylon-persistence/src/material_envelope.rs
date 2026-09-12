//! Eight-family committed-material successor. Component row codecs remain exact.

use crate::{
    committed_tick_envelope::{
        compose_row_families, validate_committed_tick_envelope_bounds, CommittedTickRowBatch,
        CommittedTickRowFamilies,
    },
    identity::CampaignId,
    runtime::RustPersistenceRuntimeError,
};
use babylon_kernel::content_digest::sha256_of;
use babylon_tick::material_replay::IdentifiedMaterialTick;

const DOMAIN: &[u8] = b"babylon.committed-material-tick.v3\0";
pub const MAX_COMMITTED_MATERIAL_TICK_BYTES: usize = 67_108_864;

/// Exact closed envelope, with material register and receipts inseparable from its claim.
#[derive(Debug, PartialEq, Eq)]
pub struct CommittedMaterialTickEnvelope {
    bytes: Vec<u8>,
    digest: [u8; 32],
}
impl CommittedMaterialTickEnvelope {
    /// Frame six typed component families followed by register and material receipt families.
    /// # Errors
    /// Refuses component ordering/shape, aggregate bounds, hash mismatch and allocation failure.
    pub fn compose(
        campaign: CampaignId,
        identity: &IdentifiedMaterialTick,
        families: CommittedTickRowFamilies,
        register: &[u8],
        receipts: &[u8],
    ) -> Result<Self, RustPersistenceRuntimeError> {
        if sha256_of(receipts) != identity.receipt_digest() {
            return Err(RustPersistenceRuntimeError::CampaignConflict);
        }
        let component = compose_row_families(families)
            .map_err(RustPersistenceRuntimeError::SemanticEnvelope)?;
        validate_committed_tick_envelope_bounds(
            component.each_ref().map(|batch| batch.rows().len()),
            component.each_ref().map(CommittedTickRowBatch::body_bytes),
        )
        .map_err(RustPersistenceRuntimeError::SemanticEnvelope)?;
        let capacity = component
            .iter()
            .try_fold(DOMAIN.len() + 4 + 16 + 8 + 32 + 8 * 9, |total, batch| {
                batch.rows().iter().try_fold(total, |total, row| {
                    total
                        .checked_add(8)
                        .and_then(|n| n.checked_add(row.key().len()))
                        .and_then(|n| n.checked_add(row.payload().len()))
                        .ok_or(RustPersistenceRuntimeError::CampaignConflict)
                })
            })?
            .checked_add(16)
            .and_then(|n| n.checked_add(register.len()))
            .and_then(|n| n.checked_add(receipts.len()))
            .ok_or(RustPersistenceRuntimeError::CampaignConflict)?;
        if capacity > MAX_COMMITTED_MATERIAL_TICK_BYTES {
            return Err(RustPersistenceRuntimeError::CampaignConflict);
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(capacity)
            .map_err(|_| RustPersistenceRuntimeError::Allocation {
                field: "material envelope",
                requested: capacity,
            })?;
        bytes.extend_from_slice(DOMAIN);
        bytes.extend_from_slice(&3_u32.to_be_bytes());
        bytes.extend_from_slice(campaign.canonical_bytes());
        bytes.extend_from_slice(&identity.resolve_tick().to_be_bytes());
        bytes.extend_from_slice(identity.tick_content_hash().as_bytes());
        for (index, batch) in component.iter().enumerate() {
            bytes.push(
                u8::try_from(index + 1)
                    .map_err(|_| RustPersistenceRuntimeError::CampaignConflict)?,
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
            return Err(RustPersistenceRuntimeError::CampaignConflict);
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
fn append_u64(bytes: &mut Vec<u8>, value: usize) -> Result<(), RustPersistenceRuntimeError> {
    bytes.extend_from_slice(
        &u64::try_from(value)
            .map_err(|_| RustPersistenceRuntimeError::CampaignConflict)?
            .to_be_bytes(),
    );
    Ok(())
}
fn append_row(
    bytes: &mut Vec<u8>,
    key: &[u8],
    payload: &[u8],
) -> Result<(), RustPersistenceRuntimeError> {
    for value in [key, payload] {
        bytes.extend_from_slice(
            &u32::try_from(value.len())
                .map_err(|_| RustPersistenceRuntimeError::CampaignConflict)?
                .to_be_bytes(),
        );
        bytes.extend_from_slice(value);
    }
    Ok(())
}
