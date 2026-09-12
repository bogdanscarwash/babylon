//! Pure current resolved-practice batch identity and authority validation.

use std::collections::BTreeMap;

use babylon_kernel::content_digest::sha256_of;

use crate::actor::ActorOrganizationId;
use crate::authority::validate_input_authority_row;
use crate::{
    decode_input_authority, decode_practice_intent, encode_input_authority, encode_practice_intent,
    input_authority_ledger_digest, practice_proposal_key, validate_practice_intent, CampaignId,
    InputAuthorityId, PracticeAuthorityError, PracticeInputAuthority, PracticeInputAuthorityLedger,
    PracticeIntent, PracticeIntentError, PracticeProposalKey, MAX_PRACTICE_INPUT_AUTHORITY_ROWS,
    MAX_PRACTICE_INTENT_CANONICAL_BYTES, MIN_PRACTICE_INTENT_CANONICAL_BYTES,
    PRACTICE_INPUT_AUTHORITY_CANONICAL_BYTES,
};

const SCHEMA_VERSION: u16 = 2;
const BATCH_HEADER_CANONICAL_BYTES: usize =
    RESOLVED_PRACTICE_BATCH_DOMAIN_BYTES.len() + 1 + 2 + 16 + 8 + 32 + 32 + 32 + 2;
const MIN_BATCH_ITEM_CANONICAL_BYTES: usize =
    2 + PRACTICE_INPUT_AUTHORITY_CANONICAL_BYTES + 2 + MIN_PRACTICE_INTENT_CANONICAL_BYTES;

/// Canonical domain for current resolved-practice batches.
pub const RESOLVED_PRACTICE_BATCH_DOMAIN_BYTES: &[u8] = b"babylon.resolved-practice-batch.v2";
/// SHA-256 of the exact language-neutral current resolved-batch schema bytes.
pub const RESOLVED_PRACTICE_BATCH_SOURCE_SHA256: [u8; 32] = [
    0x67, 0xa5, 0x3f, 0x90, 0xde, 0x17, 0x45, 0x88, 0xe9, 0xee, 0x3f, 0x8f, 0x01, 0x48, 0xa2, 0x15,
    0xa6, 0xe2, 0x05, 0x75, 0x3c, 0xe4, 0xb3, 0x6b, 0x2e, 0xe6, 0x70, 0xd9, 0xa3, 0xf9, 0x99, 0xdb,
];
/// Designed serialization and validation-fuel ceiling, not a political quota.
pub const MAX_RESOLVED_PRACTICE_BATCH_ITEMS: usize = 4_096;
/// Designed canonical-byte and decode-fuel ceiling for one complete batch.
pub const MAX_RESOLVED_PRACTICE_BATCH_CANONICAL_BYTES: usize = BATCH_HEADER_CANONICAL_BYTES
    + MAX_RESOLVED_PRACTICE_BATCH_ITEMS
        * (2 + PRACTICE_INPUT_AUTHORITY_CANONICAL_BYTES + 2 + MAX_PRACTICE_INTENT_CANONICAL_BYTES);

/// Exact current resolved-batch refusals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PracticeBatchError {
    BatchDomain = 1,
    BatchSchemaVersion = 2,
    BatchTruncated = 3,
    BatchTrailingBytes = 4,
    BatchLength = 5,
    BatchItemLimit = 6,
    BatchItemLength = 7,
    BatchItemOrder = 8,
    BatchItemDuplicate = 9,
    BatchResolveTick = 10,
    BatchLedgerDigest = 11,
    BatchCampaign = 12,
    BatchAuthorityMismatch = 13,
    BatchContentDigest = 14,
    BatchResourceContractDigest = 15,
}

/// Unknown current resolved-batch error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownPracticeBatchErrorCode(pub u16);

impl TryFrom<u16> for PracticeBatchError {
    type Error = UnknownPracticeBatchErrorCode;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::BatchDomain),
            2 => Ok(Self::BatchSchemaVersion),
            3 => Ok(Self::BatchTruncated),
            4 => Ok(Self::BatchTrailingBytes),
            5 => Ok(Self::BatchLength),
            6 => Ok(Self::BatchItemLimit),
            7 => Ok(Self::BatchItemLength),
            8 => Ok(Self::BatchItemOrder),
            9 => Ok(Self::BatchItemDuplicate),
            10 => Ok(Self::BatchResolveTick),
            11 => Ok(Self::BatchLedgerDigest),
            12 => Ok(Self::BatchCampaign),
            13 => Ok(Self::BatchAuthorityMismatch),
            14 => Ok(Self::BatchContentDigest),
            15 => Ok(Self::BatchResourceContractDigest),
            _ => Err(UnknownPracticeBatchErrorCode(value)),
        }
    }
}

impl From<PracticeBatchError> for u16 {
    fn from(value: PracticeBatchError) -> Self {
        value as Self
    }
}

/// Lossless batch, nested-authority, or nested-intent refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedPracticeBatchError {
    Batch(PracticeBatchError),
    Authority(PracticeAuthorityError),
    Intent(PracticeIntentError),
}

impl From<PracticeBatchError> for ResolvedPracticeBatchError {
    fn from(value: PracticeBatchError) -> Self {
        Self::Batch(value)
    }
}

impl From<PracticeAuthorityError> for ResolvedPracticeBatchError {
    fn from(value: PracticeAuthorityError) -> Self {
        Self::Authority(value)
    }
}

impl From<PracticeIntentError> for ResolvedPracticeBatchError {
    fn from(value: PracticeIntentError) -> Self {
        Self::Intent(value)
    }
}

/// One exact accepted authority-row and intent pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPracticeBatchItem {
    pub authority: PracticeInputAuthority,
    pub intent: PracticeIntent,
}

/// One immutable canonical current input batch for a detached tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPracticeBatch {
    pub schema_version: u16,
    pub campaign_id: CampaignId,
    pub resolve_tick: u64,
    pub authority_ledger_digest: [u8; 32],
    pub resource_allocation_contract_digest: [u8; 32],
    pub content_digest: [u8; 32],
    pub items: Vec<ResolvedPracticeBatchItem>,
}

/// Fixed top-level field order for language-neutral implementations.
pub const RESOLVED_PRACTICE_BATCH_FIELD_ORDER: [&str; 7] = [
    "schema_version",
    "campaign_id",
    "resolve_tick",
    "authority_ledger_digest",
    "resource_allocation_contract_digest",
    "content_digest",
    "items",
];

/// Fixed nested-item field order for language-neutral implementations.
pub const RESOLVED_PRACTICE_BATCH_ITEM_FIELD_ORDER: [&str; 2] =
    ["authority_row_canonical_bytes", "intent_canonical_bytes"];

fn validate_schema(value: u16) -> Result<(), PracticeBatchError> {
    if value == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(PracticeBatchError::BatchSchemaVersion)
    }
}

fn validate_ledger_digest(
    expected: [u8; 32],
    ledger: &PracticeInputAuthorityLedger,
) -> Result<(), ResolvedPracticeBatchError> {
    let actual = input_authority_ledger_digest(ledger)?;
    if expected == actual {
        Ok(())
    } else {
        Err(PracticeBatchError::BatchLedgerDigest.into())
    }
}

fn validate_top_level(
    batch: &ResolvedPracticeBatch,
    ledger: &PracticeInputAuthorityLedger,
) -> Result<(), ResolvedPracticeBatchError> {
    validate_schema(batch.schema_version)?;
    if batch.items.len() > MAX_RESOLVED_PRACTICE_BATCH_ITEMS {
        return Err(PracticeBatchError::BatchItemLimit.into());
    }
    validate_ledger_digest(batch.authority_ledger_digest, ledger)
}

fn validate_item_identity(
    batch: &ResolvedPracticeBatch,
    item: &ResolvedPracticeBatchItem,
) -> Result<(), ResolvedPracticeBatchError> {
    validate_input_authority_row(&item.authority)?;
    validate_practice_intent(&item.intent)?;
    if item.intent.resolve_tick != batch.resolve_tick {
        return Err(PracticeBatchError::BatchResolveTick.into());
    }
    if item.intent.quoted_content_digest != batch.content_digest {
        return Err(PracticeBatchError::BatchContentDigest.into());
    }
    if item.intent.quoted_resource_contract_digest != batch.resource_allocation_contract_digest {
        return Err(PracticeBatchError::BatchResourceContractDigest.into());
    }
    if item.authority.campaign_id != batch.campaign_id {
        return Err(PracticeBatchError::BatchCampaign.into());
    }
    Ok(())
}

struct ActiveAuthorityIndex<'a> {
    rows_by_id: BTreeMap<InputAuthorityId, Option<&'a PracticeInputAuthority>>,
}

impl<'a> ActiveAuthorityIndex<'a> {
    fn new(
        ledger: &'a PracticeInputAuthorityLedger,
        campaign_id: CampaignId,
        resolve_tick: u64,
    ) -> Self {
        let mut rows_by_id = BTreeMap::new();
        for row in ledger
            .rows
            .iter()
            .take(MAX_PRACTICE_INPUT_AUTHORITY_ROWS + 1)
        {
            if row.campaign_id != campaign_id {
                continue;
            }
            let selected = rows_by_id.entry(row.input_authority_id).or_insert(None);
            if resolve_tick >= row.effective_from_tick
                && resolve_tick < row.effective_through_tick_exclusive
            {
                *selected = Some(row);
            }
        }
        Self { rows_by_id }
    }

    fn resolve(
        &self,
        input_authority_id: InputAuthorityId,
        actor_org_id: ActorOrganizationId,
    ) -> Result<&'a PracticeInputAuthority, PracticeAuthorityError> {
        let selected = self
            .rows_by_id
            .get(&input_authority_id)
            .ok_or(PracticeAuthorityError::AuthorityNotFound)?
            .ok_or(PracticeAuthorityError::AuthorityInactive)?;
        if selected.actor_org_id != actor_org_id {
            return Err(PracticeAuthorityError::AuthorityActorMismatch);
        }
        Ok(selected)
    }
}

fn validate_item_authority(
    item: &ResolvedPracticeBatchItem,
    authority_index: &ActiveAuthorityIndex<'_>,
) -> Result<(), ResolvedPracticeBatchError> {
    let selected =
        authority_index.resolve(item.intent.input_authority_id, item.intent.actor_org_id)?;
    if selected == &item.authority {
        Ok(())
    } else {
        Err(PracticeBatchError::BatchAuthorityMismatch.into())
    }
}

fn validate_batch_items_against_validated_ledger(
    batch: &ResolvedPracticeBatch,
    ledger: &PracticeInputAuthorityLedger,
) -> Result<(), ResolvedPracticeBatchError> {
    let authority_index = ActiveAuthorityIndex::new(ledger, batch.campaign_id, batch.resolve_tick);
    let mut previous: Option<PracticeProposalKey> = None;
    for item in batch
        .items
        .iter()
        .take(MAX_RESOLVED_PRACTICE_BATCH_ITEMS + 1)
    {
        validate_item_identity(batch, item)?;
        validate_item_authority(item, &authority_index)?;
        let current = practice_proposal_key(&item.intent);
        validate_key_order(previous, current)?;
        previous = Some(current);
    }
    Ok(())
}

fn validate_key_order(
    previous: Option<PracticeProposalKey>,
    current: PracticeProposalKey,
) -> Result<(), ResolvedPracticeBatchError> {
    if previous == Some(current) {
        return Err(PracticeBatchError::BatchItemDuplicate.into());
    }
    if previous.is_some_and(|prior| current < prior) {
        return Err(PracticeBatchError::BatchItemOrder.into());
    }
    Ok(())
}

/// Validate one complete current batch against its authoritative committed ledger.
///
/// # Errors
/// Returns the first exact batch, nested authority, or nested intent refusal.
pub fn validate_resolved_practice_batch(
    batch: &ResolvedPracticeBatch,
    ledger: &PracticeInputAuthorityLedger,
) -> Result<(), ResolvedPracticeBatchError> {
    validate_top_level(batch, ledger)?;
    validate_batch_items_against_validated_ledger(batch, ledger)
}

fn append_domain(output: &mut Vec<u8>) {
    output.extend_from_slice(RESOLVED_PRACTICE_BATCH_DOMAIN_BYTES);
    output.push(0);
}

fn append_item(
    output: &mut Vec<u8>,
    item: &ResolvedPracticeBatchItem,
) -> Result<(), ResolvedPracticeBatchError> {
    let authority = encode_input_authority(&item.authority)?;
    let intent = encode_practice_intent(&item.intent)?;
    if authority.len() != PRACTICE_INPUT_AUTHORITY_CANONICAL_BYTES
        || intent.len() > MAX_PRACTICE_INTENT_CANONICAL_BYTES
    {
        return Err(PracticeBatchError::BatchItemLength.into());
    }
    let authority_length =
        u16::try_from(authority.len()).map_err(|_| PracticeBatchError::BatchItemLength)?;
    let intent_length =
        u16::try_from(intent.len()).map_err(|_| PracticeBatchError::BatchItemLength)?;
    output.extend_from_slice(&authority_length.to_be_bytes());
    output.extend_from_slice(&authority);
    output.extend_from_slice(&intent_length.to_be_bytes());
    output.extend_from_slice(&intent);
    Ok(())
}

fn minimum_batch_capacity(item_count: usize) -> Result<usize, PracticeBatchError> {
    let item_bytes = item_count
        .checked_mul(MIN_BATCH_ITEM_CANONICAL_BYTES)
        .ok_or(PracticeBatchError::BatchLength)?;
    BATCH_HEADER_CANONICAL_BYTES
        .checked_add(item_bytes)
        .ok_or(PracticeBatchError::BatchLength)
}

/// Encode one validated current resolved-practice batch.
///
/// # Errors
/// Returns the first exact validation, nested encoding, or canonical-size refusal.
pub fn encode_resolved_practice_batch(
    batch: &ResolvedPracticeBatch,
    ledger: &PracticeInputAuthorityLedger,
) -> Result<Vec<u8>, ResolvedPracticeBatchError> {
    validate_resolved_practice_batch(batch, ledger)?;
    let mut output = Vec::with_capacity(minimum_batch_capacity(batch.items.len())?);
    append_domain(&mut output);
    output.extend_from_slice(&batch.schema_version.to_be_bytes());
    output.extend_from_slice(&batch.campaign_id.as_bytes());
    output.extend_from_slice(&batch.resolve_tick.to_be_bytes());
    output.extend_from_slice(&batch.authority_ledger_digest);
    output.extend_from_slice(&batch.resource_allocation_contract_digest);
    output.extend_from_slice(&batch.content_digest);
    let count = u16::try_from(batch.items.len()).map_err(|_| PracticeBatchError::BatchItemLimit)?;
    output.extend_from_slice(&count.to_be_bytes());
    for item in batch
        .items
        .iter()
        .take(MAX_RESOLVED_PRACTICE_BATCH_ITEMS + 1)
    {
        append_item(&mut output, item)?;
    }
    if output.len() > MAX_RESOLVED_PRACTICE_BATCH_CANONICAL_BYTES {
        return Err(PracticeBatchError::BatchLength.into());
    }
    Ok(output)
}

struct Cursor<'a> {
    payload: &'a [u8],
    index: usize,
}

impl<'a> Cursor<'a> {
    const fn new(payload: &'a [u8]) -> Self {
        Self { payload, index: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], PracticeBatchError> {
        let end = self
            .index
            .checked_add(count)
            .ok_or(PracticeBatchError::BatchTruncated)?;
        let value = self
            .payload
            .get(self.index..end)
            .ok_or(PracticeBatchError::BatchTruncated)?;
        self.index = end;
        Ok(value)
    }

    fn domain(&mut self) -> Result<(), PracticeBatchError> {
        if self.take(RESOLVED_PRACTICE_BATCH_DOMAIN_BYTES.len())?
            == RESOLVED_PRACTICE_BATCH_DOMAIN_BYTES
            && self.take(1)? == [0]
        {
            Ok(())
        } else {
            Err(PracticeBatchError::BatchDomain)
        }
    }

    fn u16(&mut self) -> Result<u16, PracticeBatchError> {
        Ok(u16::from_be_bytes(
            self.take(2)?
                .try_into()
                .map_err(|_| PracticeBatchError::BatchTruncated)?,
        ))
    }

    fn u64(&mut self) -> Result<u64, PracticeBatchError> {
        Ok(u64::from_be_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| PracticeBatchError::BatchTruncated)?,
        ))
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], PracticeBatchError> {
        self.take(N)?
            .try_into()
            .map_err(|_| PracticeBatchError::BatchTruncated)
    }

    fn finish(&self) -> Result<(), PracticeBatchError> {
        if self.index == self.payload.len() {
            Ok(())
        } else {
            Err(PracticeBatchError::BatchTrailingBytes)
        }
    }
}

fn decode_item(
    cursor: &mut Cursor<'_>,
) -> Result<ResolvedPracticeBatchItem, ResolvedPracticeBatchError> {
    let authority_length = usize::from(cursor.u16()?);
    if authority_length != PRACTICE_INPUT_AUTHORITY_CANONICAL_BYTES {
        return Err(PracticeBatchError::BatchItemLength.into());
    }
    let authority = decode_input_authority(cursor.take(authority_length)?)?;
    let intent_length = usize::from(cursor.u16()?);
    if intent_length > MAX_PRACTICE_INTENT_CANONICAL_BYTES {
        return Err(PracticeBatchError::BatchItemLength.into());
    }
    let intent = decode_practice_intent(cursor.take(intent_length)?)?;
    Ok(ResolvedPracticeBatchItem { authority, intent })
}

/// Decode and validate one complete current resolved-practice batch.
///
/// # Errors
/// Returns the first exact size, wire, nested, ledger, identity, or ordering refusal.
pub fn decode_resolved_practice_batch(
    payload: &[u8],
    ledger: &PracticeInputAuthorityLedger,
) -> Result<ResolvedPracticeBatch, ResolvedPracticeBatchError> {
    if payload.len() > MAX_RESOLVED_PRACTICE_BATCH_CANONICAL_BYTES {
        return Err(PracticeBatchError::BatchLength.into());
    }
    let mut cursor = Cursor::new(payload);
    cursor.domain()?;
    let schema_version = cursor.u16()?;
    validate_schema(schema_version)?;
    let campaign_id = CampaignId::from_bytes(cursor.array()?);
    let resolve_tick = cursor.u64()?;
    let authority_ledger_digest = cursor.array()?;
    let resource_allocation_contract_digest = cursor.array()?;
    let content_digest = cursor.array()?;
    let count = usize::from(cursor.u16()?);
    if count > MAX_RESOLVED_PRACTICE_BATCH_ITEMS {
        return Err(PracticeBatchError::BatchItemLimit.into());
    }
    validate_ledger_digest(authority_ledger_digest, ledger)?;
    let mut items = Vec::with_capacity(count);
    for index in 0..=MAX_RESOLVED_PRACTICE_BATCH_ITEMS {
        if index == count {
            break;
        }
        items.push(decode_item(&mut cursor)?);
    }
    cursor.finish()?;
    let batch = ResolvedPracticeBatch {
        schema_version,
        campaign_id,
        resolve_tick,
        authority_ledger_digest,
        resource_allocation_contract_digest,
        content_digest,
        items,
    };
    validate_batch_items_against_validated_ledger(&batch, ledger)?;
    Ok(batch)
}

/// Hash one successfully validated and encoded current resolved-practice batch.
///
/// The digest is not embedded in its own preimage.
///
/// # Errors
/// Returns the exact validation or encoding refusal without publishing a digest.
pub fn resolved_practice_batch_digest(
    batch: &ResolvedPracticeBatch,
    ledger: &PracticeInputAuthorityLedger,
) -> Result<[u8; 32], ResolvedPracticeBatchError> {
    Ok(sha256_of(&encode_resolved_practice_batch(batch, ledger)?))
}
