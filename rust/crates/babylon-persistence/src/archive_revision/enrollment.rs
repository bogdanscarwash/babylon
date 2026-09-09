//! Original retention identity verification; reinstallation never advances its floor.

use postgres::GenericClient;
use sha2::{Digest as _, Sha256};

use super::storage::{signed, unsigned, ReadAuthority};
use super::{knowledge, schema, storage, ArchiveReadScopeV2};
use crate::archive::{database, decode, decode_digest};
use crate::{CampaignId, SemanticArchiveErrorV1};

const ADOPTION_DOMAIN: &[u8] = b"babylon.archive-retention-adoption.v2\0";

pub(super) struct AdoptionSeed {
    pub floor: ArchiveReadScopeV2,
    pub processed: u64,
    pub count: u64,
    pub heads_digest: [u8; 32],
}

impl AdoptionSeed {
    pub fn digest(&self) -> Result<[u8; 32], SemanticArchiveErrorV1> {
        if self.processed > self.floor.tick()
            || self.count > i64::MAX as u64
            || (self.floor.tick() == 0 && self.count != 0)
        {
            return Err(SemanticArchiveErrorV1::StoredPageMismatch);
        }
        let mut digest = Sha256::new();
        digest.update(ADOPTION_DOMAIN);
        digest.update(self.floor.campaign_id().canonical_bytes());
        digest.update(self.floor.tick().to_be_bytes());
        let hash = self.floor.tick_content_hash();
        digest.update([u8::from(hash.is_some())]);
        if let Some(hash) = hash {
            digest.update(hash);
        }
        digest.update(self.processed.to_be_bytes());
        digest.update(self.count.to_be_bytes());
        digest.update(self.heads_digest);
        Ok(digest.finalize().into())
    }
}

pub(super) fn insert(
    client: &mut impl GenericClient,
    seed: &AdoptionSeed,
) -> Result<(), SemanticArchiveErrorV1> {
    let digest = seed.digest()?;
    let campaign = seed.floor.campaign_id();
    let hash = seed.floor.tick_content_hash().map(|bytes| bytes.to_vec());
    client.execute("INSERT INTO babylon_meta.archive_retention_v2 \
        (campaign_id,floor_tick,floor_content_hash,processed_at_adoption,adopted_page_count,adopted_heads_sha256,adoption_sha256) \
        VALUES($1,$2,$3,$4,$5,$6,$7)",
        &[campaign.as_uuid(), &signed(seed.floor.tick())?, &hash, &signed(seed.processed)?, &signed(seed.count)?, &&seed.heads_digest[..], &&digest[..]])
        .map_err(|error| database("publish exact Archive retention enrollment", &error))?;
    Ok(())
}

pub(super) fn validate_all(client: &mut impl GenericClient) -> Result<(), SemanticArchiveErrorV1> {
    let mut after: Option<uuid::Uuid> = None;
    loop {
        let rows = client
            .query(
                "SELECT campaign_id FROM babylon_meta.campaign \
                WHERE ($1::uuid IS NULL OR campaign_id>$1) ORDER BY campaign_id LIMIT 256",
                &[&after],
            )
            .map_err(|error| database("enumerate existing Archive enrollments", &error))?;
        if rows.is_empty() {
            return Ok(());
        }
        for row in rows {
            let id = decode(&row, 0)?;
            validate(client, CampaignId::from_uuid(id))?;
            after = Some(id);
        }
    }
}

pub(super) fn validate(
    client: &mut impl GenericClient,
    campaign: CampaignId,
) -> Result<(), SemanticArchiveErrorV1> {
    let seed = validate_header(client, campaign)?;
    let (count, heads_digest) = validate_adopted_heads(client, &seed.floor)?;
    if count != seed.count || heads_digest != seed.heads_digest {
        return Err(SemanticArchiveErrorV1::StoredPageMismatch);
    }
    Ok(())
}

/// Validate the original immutable enrollment identity without reopening every
/// adopted payload. Publications hold this row locked and validate each payload
/// they use; installation still verifies the complete original adopted set.
pub(super) fn validate_header(
    client: &mut impl GenericClient,
    campaign: CampaignId,
) -> Result<AdoptionSeed, SemanticArchiveErrorV1> {
    let row = client
        .query_opt(
            "SELECT floor_tick,floor_content_hash,processed_at_adoption, \
        adopted_page_count,adoption_sha256,adopted_heads_sha256 FROM babylon_meta.archive_retention_v2 \
        WHERE campaign_id=$1",
            &[campaign.as_uuid()],
        )
        .map_err(|error| database("read original Archive enrollment", &error))?
        .ok_or(SemanticArchiveErrorV1::StoredPageMismatch)?;
    let floor_tick = unsigned(decode(&row, 0)?)?;
    let stored_hash: Option<Vec<u8>> = decode(&row, 1)?;
    let floor = match (floor_tick, stored_hash) {
        (0, None) => ArchiveReadScopeV2::foundation(campaign),
        (0, Some(_)) | (_, None) => return Err(SemanticArchiveErrorV1::StoredPageMismatch),
        (tick, Some(hash)) => ArchiveReadScopeV2::committed(
            campaign,
            tick,
            hash.try_into()
                .map_err(|_| SemanticArchiveErrorV1::StoredPageMismatch)?,
        )?,
    };
    validate_floor(client, &floor)?;
    let seed = AdoptionSeed {
        floor,
        processed: unsigned(decode(&row, 2)?)?,
        count: unsigned(decode(&row, 3)?)?,
        heads_digest: decode_digest(&row, 5)?,
    };
    if seed.digest()? != decode_digest(&row, 4)? {
        return Err(SemanticArchiveErrorV1::StoredPageMismatch);
    }
    Ok(seed)
}

fn validate_floor(
    client: &mut impl GenericClient,
    floor: &ArchiveReadScopeV2,
) -> Result<(), SemanticArchiveErrorV1> {
    if floor.tick() == 0 {
        return Ok(());
    }
    let campaign = floor.campaign_id();
    let row = client
        .query_opt(
            "SELECT tick_content_hash FROM babylon_state.tick_commit \
        WHERE campaign_id=$1 AND resolve_tick=$2",
            &[campaign.as_uuid(), &signed(floor.tick())?],
        )
        .map_err(|error| database("verify original Archive retention floor", &error))?
        .ok_or(SemanticArchiveErrorV1::MissingCommittedReceipt)?;
    if Some(decode_digest(&row, 0)?) != floor.tick_content_hash() {
        return Err(SemanticArchiveErrorV1::ReceiptMismatch);
    }
    Ok(())
}

fn validate_adopted_heads(
    client: &mut impl GenericClient,
    floor: &ArchiveReadScopeV2,
) -> Result<(u64, [u8; 32]), SemanticArchiveErrorV1> {
    let campaign = floor.campaign_id();
    let mut after_kind = String::new();
    let mut after_id = String::new();
    let mut after_tick = 0_i64;
    let mut count = 0_u64;
    let mut heads = Sha256::new();
    loop {
        let rows = client.query(&format!("SELECT {} FROM babylon_meta.archive_page_revision_v2 \
            WHERE campaign_id=$1 AND origin=0 AND (subject_kind,subject_id,effective_tick)>($2,$3,$4) \
            ORDER BY subject_kind,subject_id,effective_tick LIMIT 16", storage::COLUMNS),
            &[campaign.as_uuid(), &after_kind, &after_id, &after_tick])
            .map_err(|error| database("verify original retained Archive heads", &error))?;
        if rows.is_empty() {
            return Ok((count, heads.finalize().into()));
        }
        for row in rows {
            let record = storage::decode_record(client, &row, ReadAuthority::Writer)?;
            if record.effective_tick != floor.tick()
                || knowledge::capture(client, &record)? != record.grants
            {
                return Err(SemanticArchiveErrorV1::StoredPageMismatch);
            }
            schema::verify_source_marker(client, &record)?;
            heads.update(record.digest()?);
            count = count
                .checked_add(1)
                .ok_or(SemanticArchiveErrorV1::CollectionBound)?;
            record.subject.kind().as_str().clone_into(&mut after_kind);
            record.subject.id().clone_into(&mut after_id);
            after_tick = signed(record.effective_tick)?;
        }
    }
}

#[cfg(test)]
mod tests;
