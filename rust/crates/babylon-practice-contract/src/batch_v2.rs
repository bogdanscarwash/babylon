//! Pure V2 resolved-practice batch identity and authority validation.

use std::collections::BTreeMap;

use babylon_kernel::sha256_of;

use crate::actor_v2::ActorOrganizationIdV2;
use crate::authority_v2::validate_input_authority_row_v2;
use crate::{
    decode_input_authority_v2, decode_practice_intent_v2, encode_input_authority_v2,
    encode_practice_intent_v2, input_authority_ledger_v2_digest, practice_proposal_key_v2,
    validate_practice_intent_v2, CampaignIdV2, InputAuthorityIdV2, PracticeAuthorityV2Error,
    PracticeInputAuthorityLedgerV2, PracticeInputAuthorityV2, PracticeIntentV2,
    PracticeIntentV2Error, PracticeProposalKeyV2, MAX_PRACTICE_INPUT_AUTHORITY_ROWS_V2,
    MAX_PRACTICE_INTENT_CANONICAL_BYTES_V2, MIN_PRACTICE_INTENT_CANONICAL_BYTES_V2,
    PRACTICE_INPUT_AUTHORITY_V2_CANONICAL_BYTES,
};

const SCHEMA_VERSION: u16 = 2;
const BATCH_HEADER_CANONICAL_BYTES: usize =
    RESOLVED_PRACTICE_BATCH_V2_DOMAIN_BYTES.len() + 1 + 2 + 16 + 8 + 32 + 32 + 32 + 2;
const MIN_BATCH_ITEM_CANONICAL_BYTES: usize =
    2 + PRACTICE_INPUT_AUTHORITY_V2_CANONICAL_BYTES + 2 + MIN_PRACTICE_INTENT_CANONICAL_BYTES_V2;

/// Canonical domain for V2 resolved-practice batches.
pub const RESOLVED_PRACTICE_BATCH_V2_DOMAIN_BYTES: &[u8] = b"babylon.resolved-practice-batch.v2";
/// SHA-256 of the exact language-neutral V2 resolved-batch schema bytes.
pub const RESOLVED_PRACTICE_BATCH_V2_SOURCE_SHA256: [u8; 32] = [
    0xbd, 0x3d, 0x6d, 0x2e, 0x8c, 0x3f, 0x24, 0x90, 0x55, 0x5d, 0xbb, 0xce, 0xc8, 0x79, 0x54, 0x54,
    0xfa, 0x06, 0xb2, 0x25, 0xf8, 0xb7, 0xf3, 0x92, 0x6e, 0xbf, 0x30, 0xb3, 0x59, 0xb4, 0xf1, 0x2f,
];
/// Designed serialization and validation-fuel ceiling, not a political quota.
pub const MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2: usize = 4_096;
/// Designed canonical-byte and decode-fuel ceiling for one complete batch.
pub const MAX_RESOLVED_PRACTICE_BATCH_CANONICAL_BYTES_V2: usize = BATCH_HEADER_CANONICAL_BYTES
    + MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2
        * (2 + PRACTICE_INPUT_AUTHORITY_V2_CANONICAL_BYTES
            + 2
            + MAX_PRACTICE_INTENT_CANONICAL_BYTES_V2);

/// Exact V2 resolved-batch refusals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PracticeBatchV2Error {
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

/// Unknown V2 resolved-batch error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownPracticeBatchV2ErrorCode(pub u16);

impl TryFrom<u16> for PracticeBatchV2Error {
    type Error = UnknownPracticeBatchV2ErrorCode;

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
            _ => Err(UnknownPracticeBatchV2ErrorCode(value)),
        }
    }
}

impl From<PracticeBatchV2Error> for u16 {
    fn from(value: PracticeBatchV2Error) -> Self {
        value as Self
    }
}

/// Lossless batch, nested-authority, or nested-intent refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedPracticeBatchV2Error {
    Batch(PracticeBatchV2Error),
    Authority(PracticeAuthorityV2Error),
    Intent(PracticeIntentV2Error),
}

impl From<PracticeBatchV2Error> for ResolvedPracticeBatchV2Error {
    fn from(value: PracticeBatchV2Error) -> Self {
        Self::Batch(value)
    }
}

impl From<PracticeAuthorityV2Error> for ResolvedPracticeBatchV2Error {
    fn from(value: PracticeAuthorityV2Error) -> Self {
        Self::Authority(value)
    }
}

impl From<PracticeIntentV2Error> for ResolvedPracticeBatchV2Error {
    fn from(value: PracticeIntentV2Error) -> Self {
        Self::Intent(value)
    }
}

/// One exact accepted authority-row and intent pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPracticeBatchItemV2 {
    pub authority: PracticeInputAuthorityV2,
    pub intent: PracticeIntentV2,
}

/// One immutable canonical V2 input batch for a detached tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPracticeBatchV2 {
    pub schema_version: u16,
    pub campaign_id: CampaignIdV2,
    pub resolve_tick: u64,
    pub authority_ledger_digest: [u8; 32],
    pub resource_allocation_contract_digest: [u8; 32],
    pub content_digest: [u8; 32],
    pub items: Vec<ResolvedPracticeBatchItemV2>,
}

/// Fixed top-level field order for language-neutral implementations.
pub const RESOLVED_PRACTICE_BATCH_V2_FIELD_ORDER: [&str; 7] = [
    "schema_version",
    "campaign_id",
    "resolve_tick",
    "authority_ledger_digest",
    "resource_allocation_contract_digest",
    "content_digest",
    "items",
];

/// Fixed nested-item field order for language-neutral implementations.
pub const RESOLVED_PRACTICE_BATCH_ITEM_V2_FIELD_ORDER: [&str; 2] =
    ["authority_row_canonical_bytes", "intent_canonical_bytes"];

fn validate_schema(value: u16) -> Result<(), PracticeBatchV2Error> {
    if value == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(PracticeBatchV2Error::BatchSchemaVersion)
    }
}

fn validate_ledger_digest(
    expected: [u8; 32],
    ledger: &PracticeInputAuthorityLedgerV2,
) -> Result<(), ResolvedPracticeBatchV2Error> {
    let actual = input_authority_ledger_v2_digest(ledger)?;
    if expected == actual {
        Ok(())
    } else {
        Err(PracticeBatchV2Error::BatchLedgerDigest.into())
    }
}

fn validate_top_level(
    batch: &ResolvedPracticeBatchV2,
    ledger: &PracticeInputAuthorityLedgerV2,
) -> Result<(), ResolvedPracticeBatchV2Error> {
    validate_schema(batch.schema_version)?;
    if batch.items.len() > MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2 {
        return Err(PracticeBatchV2Error::BatchItemLimit.into());
    }
    validate_ledger_digest(batch.authority_ledger_digest, ledger)
}

fn validate_item_identity(
    batch: &ResolvedPracticeBatchV2,
    item: &ResolvedPracticeBatchItemV2,
) -> Result<(), ResolvedPracticeBatchV2Error> {
    validate_input_authority_row_v2(&item.authority)?;
    validate_practice_intent_v2(&item.intent)?;
    if item.intent.resolve_tick != batch.resolve_tick {
        return Err(PracticeBatchV2Error::BatchResolveTick.into());
    }
    if item.intent.quoted_content_digest != batch.content_digest {
        return Err(PracticeBatchV2Error::BatchContentDigest.into());
    }
    if item.intent.quoted_resource_contract_digest != batch.resource_allocation_contract_digest {
        return Err(PracticeBatchV2Error::BatchResourceContractDigest.into());
    }
    if item.authority.campaign_id != batch.campaign_id {
        return Err(PracticeBatchV2Error::BatchCampaign.into());
    }
    Ok(())
}

struct ActiveAuthorityIndexV2<'a> {
    rows_by_id: BTreeMap<InputAuthorityIdV2, Option<&'a PracticeInputAuthorityV2>>,
}

impl<'a> ActiveAuthorityIndexV2<'a> {
    fn new(
        ledger: &'a PracticeInputAuthorityLedgerV2,
        campaign_id: CampaignIdV2,
        resolve_tick: u64,
    ) -> Self {
        let mut rows_by_id = BTreeMap::new();
        for row in ledger
            .rows
            .iter()
            .take(MAX_PRACTICE_INPUT_AUTHORITY_ROWS_V2 + 1)
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
        input_authority_id: InputAuthorityIdV2,
        actor_org_id: ActorOrganizationIdV2,
    ) -> Result<&'a PracticeInputAuthorityV2, PracticeAuthorityV2Error> {
        let selected = self
            .rows_by_id
            .get(&input_authority_id)
            .ok_or(PracticeAuthorityV2Error::AuthorityNotFound)?
            .ok_or(PracticeAuthorityV2Error::AuthorityInactive)?;
        if selected.actor_org_id != actor_org_id {
            return Err(PracticeAuthorityV2Error::AuthorityActorMismatch);
        }
        Ok(selected)
    }
}

fn validate_item_authority(
    item: &ResolvedPracticeBatchItemV2,
    authority_index: &ActiveAuthorityIndexV2<'_>,
) -> Result<(), ResolvedPracticeBatchV2Error> {
    let selected =
        authority_index.resolve(item.intent.input_authority_id, item.intent.actor_org_id)?;
    if selected == &item.authority {
        Ok(())
    } else {
        Err(PracticeBatchV2Error::BatchAuthorityMismatch.into())
    }
}

fn validate_batch_items_against_validated_ledger(
    batch: &ResolvedPracticeBatchV2,
    ledger: &PracticeInputAuthorityLedgerV2,
) -> Result<(), ResolvedPracticeBatchV2Error> {
    let authority_index =
        ActiveAuthorityIndexV2::new(ledger, batch.campaign_id, batch.resolve_tick);
    let mut previous: Option<PracticeProposalKeyV2> = None;
    for item in batch
        .items
        .iter()
        .take(MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2 + 1)
    {
        validate_item_identity(batch, item)?;
        validate_item_authority(item, &authority_index)?;
        let current = practice_proposal_key_v2(&item.intent);
        validate_key_order(previous, current)?;
        previous = Some(current);
    }
    Ok(())
}

fn validate_key_order(
    previous: Option<PracticeProposalKeyV2>,
    current: PracticeProposalKeyV2,
) -> Result<(), ResolvedPracticeBatchV2Error> {
    if previous == Some(current) {
        return Err(PracticeBatchV2Error::BatchItemDuplicate.into());
    }
    if previous.is_some_and(|prior| current < prior) {
        return Err(PracticeBatchV2Error::BatchItemOrder.into());
    }
    Ok(())
}

/// Validate one complete V2 batch against its authoritative committed ledger.
///
/// # Errors
/// Returns the first exact batch, nested authority, or nested intent refusal.
pub fn validate_resolved_practice_batch_v2(
    batch: &ResolvedPracticeBatchV2,
    ledger: &PracticeInputAuthorityLedgerV2,
) -> Result<(), ResolvedPracticeBatchV2Error> {
    validate_top_level(batch, ledger)?;
    validate_batch_items_against_validated_ledger(batch, ledger)
}

fn append_domain(output: &mut Vec<u8>) {
    output.extend_from_slice(RESOLVED_PRACTICE_BATCH_V2_DOMAIN_BYTES);
    output.push(0);
}

fn append_item(
    output: &mut Vec<u8>,
    item: &ResolvedPracticeBatchItemV2,
) -> Result<(), ResolvedPracticeBatchV2Error> {
    let authority = encode_input_authority_v2(&item.authority)?;
    let intent = encode_practice_intent_v2(&item.intent)?;
    if authority.len() != PRACTICE_INPUT_AUTHORITY_V2_CANONICAL_BYTES
        || intent.len() > MAX_PRACTICE_INTENT_CANONICAL_BYTES_V2
    {
        return Err(PracticeBatchV2Error::BatchItemLength.into());
    }
    let authority_length =
        u16::try_from(authority.len()).map_err(|_| PracticeBatchV2Error::BatchItemLength)?;
    let intent_length =
        u16::try_from(intent.len()).map_err(|_| PracticeBatchV2Error::BatchItemLength)?;
    output.extend_from_slice(&authority_length.to_be_bytes());
    output.extend_from_slice(&authority);
    output.extend_from_slice(&intent_length.to_be_bytes());
    output.extend_from_slice(&intent);
    Ok(())
}

fn minimum_batch_capacity(item_count: usize) -> Result<usize, PracticeBatchV2Error> {
    let item_bytes = item_count
        .checked_mul(MIN_BATCH_ITEM_CANONICAL_BYTES)
        .ok_or(PracticeBatchV2Error::BatchLength)?;
    BATCH_HEADER_CANONICAL_BYTES
        .checked_add(item_bytes)
        .ok_or(PracticeBatchV2Error::BatchLength)
}

/// Encode one validated V2 resolved-practice batch.
///
/// # Errors
/// Returns the first exact validation, nested encoding, or canonical-size refusal.
pub fn encode_resolved_practice_batch_v2(
    batch: &ResolvedPracticeBatchV2,
    ledger: &PracticeInputAuthorityLedgerV2,
) -> Result<Vec<u8>, ResolvedPracticeBatchV2Error> {
    validate_resolved_practice_batch_v2(batch, ledger)?;
    let mut output = Vec::with_capacity(minimum_batch_capacity(batch.items.len())?);
    append_domain(&mut output);
    output.extend_from_slice(&batch.schema_version.to_be_bytes());
    output.extend_from_slice(&batch.campaign_id.as_bytes());
    output.extend_from_slice(&batch.resolve_tick.to_be_bytes());
    output.extend_from_slice(&batch.authority_ledger_digest);
    output.extend_from_slice(&batch.resource_allocation_contract_digest);
    output.extend_from_slice(&batch.content_digest);
    let count =
        u16::try_from(batch.items.len()).map_err(|_| PracticeBatchV2Error::BatchItemLimit)?;
    output.extend_from_slice(&count.to_be_bytes());
    for item in batch
        .items
        .iter()
        .take(MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2 + 1)
    {
        append_item(&mut output, item)?;
    }
    if output.len() > MAX_RESOLVED_PRACTICE_BATCH_CANONICAL_BYTES_V2 {
        return Err(PracticeBatchV2Error::BatchLength.into());
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

    fn take(&mut self, count: usize) -> Result<&'a [u8], PracticeBatchV2Error> {
        let end = self
            .index
            .checked_add(count)
            .ok_or(PracticeBatchV2Error::BatchTruncated)?;
        let value = self
            .payload
            .get(self.index..end)
            .ok_or(PracticeBatchV2Error::BatchTruncated)?;
        self.index = end;
        Ok(value)
    }

    fn domain(&mut self) -> Result<(), PracticeBatchV2Error> {
        if self.take(RESOLVED_PRACTICE_BATCH_V2_DOMAIN_BYTES.len())?
            == RESOLVED_PRACTICE_BATCH_V2_DOMAIN_BYTES
            && self.take(1)? == [0]
        {
            Ok(())
        } else {
            Err(PracticeBatchV2Error::BatchDomain)
        }
    }

    fn u16(&mut self) -> Result<u16, PracticeBatchV2Error> {
        Ok(u16::from_be_bytes(
            self.take(2)?
                .try_into()
                .map_err(|_| PracticeBatchV2Error::BatchTruncated)?,
        ))
    }

    fn u64(&mut self) -> Result<u64, PracticeBatchV2Error> {
        Ok(u64::from_be_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| PracticeBatchV2Error::BatchTruncated)?,
        ))
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], PracticeBatchV2Error> {
        self.take(N)?
            .try_into()
            .map_err(|_| PracticeBatchV2Error::BatchTruncated)
    }

    fn finish(&self) -> Result<(), PracticeBatchV2Error> {
        if self.index == self.payload.len() {
            Ok(())
        } else {
            Err(PracticeBatchV2Error::BatchTrailingBytes)
        }
    }
}

fn decode_item(
    cursor: &mut Cursor<'_>,
) -> Result<ResolvedPracticeBatchItemV2, ResolvedPracticeBatchV2Error> {
    let authority_length = usize::from(cursor.u16()?);
    if authority_length != PRACTICE_INPUT_AUTHORITY_V2_CANONICAL_BYTES {
        return Err(PracticeBatchV2Error::BatchItemLength.into());
    }
    let authority = decode_input_authority_v2(cursor.take(authority_length)?)?;
    let intent_length = usize::from(cursor.u16()?);
    if intent_length > MAX_PRACTICE_INTENT_CANONICAL_BYTES_V2 {
        return Err(PracticeBatchV2Error::BatchItemLength.into());
    }
    let intent = decode_practice_intent_v2(cursor.take(intent_length)?)?;
    Ok(ResolvedPracticeBatchItemV2 { authority, intent })
}

/// Decode and validate one complete V2 resolved-practice batch.
///
/// # Errors
/// Returns the first exact size, wire, nested, ledger, identity, or ordering refusal.
pub fn decode_resolved_practice_batch_v2(
    payload: &[u8],
    ledger: &PracticeInputAuthorityLedgerV2,
) -> Result<ResolvedPracticeBatchV2, ResolvedPracticeBatchV2Error> {
    if payload.len() > MAX_RESOLVED_PRACTICE_BATCH_CANONICAL_BYTES_V2 {
        return Err(PracticeBatchV2Error::BatchLength.into());
    }
    let mut cursor = Cursor::new(payload);
    cursor.domain()?;
    let schema_version = cursor.u16()?;
    validate_schema(schema_version)?;
    let campaign_id = CampaignIdV2::from_bytes(cursor.array()?);
    let resolve_tick = cursor.u64()?;
    let authority_ledger_digest = cursor.array()?;
    let resource_allocation_contract_digest = cursor.array()?;
    let content_digest = cursor.array()?;
    let count = usize::from(cursor.u16()?);
    if count > MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2 {
        return Err(PracticeBatchV2Error::BatchItemLimit.into());
    }
    validate_ledger_digest(authority_ledger_digest, ledger)?;
    let mut items = Vec::with_capacity(count);
    for index in 0..=MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2 {
        if index == count {
            break;
        }
        items.push(decode_item(&mut cursor)?);
    }
    cursor.finish()?;
    let batch = ResolvedPracticeBatchV2 {
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

/// Hash one successfully validated and encoded V2 resolved-practice batch.
///
/// The digest is not embedded in its own preimage.
///
/// # Errors
/// Returns the exact validation or encoding refusal without publishing a digest.
pub fn resolved_practice_batch_v2_digest(
    batch: &ResolvedPracticeBatchV2,
    ledger: &PracticeInputAuthorityLedgerV2,
) -> Result<[u8; 32], ResolvedPracticeBatchV2Error> {
    Ok(sha256_of(&encode_resolved_practice_batch_v2(
        batch, ledger,
    )?))
}
