//! Database-free whole-payload contract for one live V2 committed tick.

use std::collections::TryReserveError;

use babylon_kernel::sha256_of;

use crate::identity::CampaignId;
use crate::tick_commit_claim::{
    TickCommitClaimConflictV1, TickCommitClaimV1, TICK_COMMIT_CLAIM_BYTES_V1,
};

/// Canonical `CommittedTickEnvelopeV2` layout version.
pub const COMMITTED_TICK_ENVELOPE_LAYOUT_VERSION_V2: u32 = 2;
/// Number of mandatory row families in every V2 envelope.
pub const COMMITTED_TICK_ROW_FAMILY_COUNT_V2: usize = 6;
/// Maximum aggregate rows across all mandatory families.
pub const MAX_COMMITTED_TICK_ROWS_V2: usize = 1_048_576;
/// Maximum canonical row-body bytes in one family.
pub const MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2: usize = 67_108_864;

const DOMAIN: &[u8; 35] = b"babylon.committed-tick-envelope.v2\0";
const CLAIM_TAG: u8 = 0x01;
const CLAIM_HEADER_BYTES: usize = 5;
const ROW_BATCH_HEADER_BYTES: usize = 9;
const ROW_LENGTH_BYTES: usize = 8;
const FIXED_ENVELOPE_BYTES: usize = DOMAIN.len()
    + size_of::<u32>()
    + CLAIM_HEADER_BYTES
    + TICK_COMMIT_CLAIM_BYTES_V1
    + COMMITTED_TICK_ROW_FAMILY_COUNT_V2 * ROW_BATCH_HEADER_BYTES;

/// Maximum complete canonical V2 envelope length.
pub const MAX_COMMITTED_TICK_ENVELOPE_BYTES_V2: usize = FIXED_ENVELOPE_BYTES
    + COMMITTED_TICK_ROW_FAMILY_COUNT_V2 * MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2;

/// Closed, canonical order of durable outputs produced by one tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CommittedTickRowFamilyV2 {
    /// Stable graph projection rows.
    Graph,
    /// Material runtime-state rows.
    State,
    /// Governed event rows.
    Event,
    /// Exact material-choice evidence rows.
    ChoiceReceipt,
    /// Complete or delta checkpoint rows.
    Checkpoint,
    /// Archive dirty-receipt outbox rows.
    ArchiveDirtyReceipt,
}

/// Exact mandatory V2 family order.
pub const ALL_COMMITTED_TICK_ROW_FAMILIES_V2: [CommittedTickRowFamilyV2;
    COMMITTED_TICK_ROW_FAMILY_COUNT_V2] = [
    CommittedTickRowFamilyV2::Graph,
    CommittedTickRowFamilyV2::State,
    CommittedTickRowFamilyV2::Event,
    CommittedTickRowFamilyV2::ChoiceReceipt,
    CommittedTickRowFamilyV2::Checkpoint,
    CommittedTickRowFamilyV2::ArchiveDirtyReceipt,
];

impl CommittedTickRowFamilyV2 {
    /// Return the exact V2 section tag.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::Graph => 0x10,
            Self::State => 0x11,
            Self::Event => 0x12,
            Self::ChoiceReceipt => 0x18,
            Self::Checkpoint => 0x16,
            Self::ArchiveDirtyReceipt => 0x17,
        }
    }

    /// Return the stable contract name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Graph => "graph",
            Self::State => "state",
            Self::Event => "event",
            Self::ChoiceReceipt => "choice_receipt",
            Self::Checkpoint => "checkpoint",
            Self::ArchiveDirtyReceipt => "archive_dirty_receipt",
        }
    }
}

/// One exact, immutable row supplied by its owning canonical row codec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedTickRowV2 {
    key: Vec<u8>,
    payload: Vec<u8>,
}

impl CommittedTickRowV2 {
    /// Own exact key and payload bytes without interpreting their material meaning.
    ///
    /// # Errors
    /// Returns [`CommittedTickEnvelopeErrorV2::EmptyRowKey`] for an empty key.
    pub fn compose(key: Vec<u8>, payload: Vec<u8>) -> Result<Self, CommittedTickEnvelopeErrorV2> {
        if key.is_empty() {
            return Err(CommittedTickEnvelopeErrorV2::EmptyRowKey);
        }
        Ok(Self { key, payload })
    }

    /// Borrow the exact canonical logical row-key bytes.
    #[must_use]
    pub fn key(&self) -> &[u8] {
        &self.key
    }

    /// Borrow the exact canonical row-payload bytes.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

/// Raw family-owned rows consumed by the envelope composer.
#[derive(Debug)]
pub struct CommittedTickRowFamiliesV2 {
    /// Stable graph projection rows.
    pub graph: Vec<CommittedTickRowV2>,
    /// Material runtime-state rows.
    pub state: Vec<CommittedTickRowV2>,
    /// Governed event rows.
    pub event: Vec<CommittedTickRowV2>,
    /// Exact material-choice evidence rows.
    pub choice_receipt: Vec<CommittedTickRowV2>,
    /// Complete or delta checkpoint rows.
    pub checkpoint: Vec<CommittedTickRowV2>,
    /// Mandatory singular campaign work receipt.
    pub archive_dirty_receipt: CommittedTickRowV2,
}

/// One checked, strictly key-ordered family batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedTickRowBatchV2 {
    family: CommittedTickRowFamilyV2,
    rows: Vec<CommittedTickRowV2>,
    body_bytes: usize,
}

impl CommittedTickRowBatchV2 {
    /// Return the closed family discriminator.
    #[must_use]
    pub const fn family(&self) -> CommittedTickRowFamilyV2 {
        self.family
    }

    /// Borrow the strict key-ordered exact rows.
    #[must_use]
    pub fn rows(&self) -> &[CommittedTickRowV2] {
        &self.rows
    }

    /// Return canonical row-body bytes, excluding the family section header.
    #[must_use]
    pub const fn body_bytes(&self) -> usize {
        self.body_bytes
    }
}

/// SHA-256 diagnostic for the exact complete envelope bytes.
///
/// This digest cannot substitute for the kernel-owned constitutional
/// `TickContentHashV1`; retry classification compares exact envelope bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommittedTickEnvelopeDigestV2([u8; 32]);

impl CommittedTickEnvelopeDigestV2 {
    /// Borrow the exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Complete database-free payload for one live V2 marker-last transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedTickEnvelopeV2 {
    claim: TickCommitClaimV1,
    row_families: [CommittedTickRowBatchV2; COMMITTED_TICK_ROW_FAMILY_COUNT_V2],
    total_rows: usize,
    canonical_bytes: Vec<u8>,
    digest: CommittedTickEnvelopeDigestV2,
}

/// Exact whole-payload retry success.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommittedTickEnvelopeRetryV2 {
    /// Claim and every canonical envelope byte are identical.
    Idempotent,
}

/// Closed conflict classes for a requested envelope and an existing envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommittedTickEnvelopeConflictV2 {
    /// Campaign, tick, or kernel content identity differs.
    Claim(TickCommitClaimConflictV1),
    /// The claim matches but at least one exact payload byte differs.
    WholePayloadMismatch {
        /// Shared durable campaign key.
        campaign_id: CampaignId,
        /// Shared resolve tick.
        resolve_tick: u64,
        /// Existing complete-envelope diagnostic.
        existing: CommittedTickEnvelopeDigestV2,
        /// Requested complete-envelope diagnostic.
        requested: CommittedTickEnvelopeDigestV2,
    },
}

/// Checked envelope construction refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommittedTickEnvelopeErrorV2 {
    /// The mandatory campaign work receipt was absent.
    MissingArchiveDirtyReceipt,
    /// More than one campaign work receipt was supplied.
    DuplicateArchiveDirtyReceipt {
        /// Received receipt count.
        actual: usize,
    },
    /// A canonical row key was empty.
    EmptyRowKey,
    /// Rows in one family were not strictly ascending by exact key bytes.
    RowOrder {
        /// Refused row family.
        family: CommittedTickRowFamilyV2,
        /// Zero-based row index that broke ordering.
        index: usize,
    },
    /// Two rows in one family carried the same exact key bytes.
    DuplicateRowKey {
        /// Refused row family.
        family: CommittedTickRowFamilyV2,
        /// Zero-based duplicate row index.
        index: usize,
    },
    /// A family body exceeded its governed byte ceiling.
    BatchBytes {
        /// Refused row family.
        family: CommittedTickRowFamilyV2,
        /// Received canonical body length.
        actual: usize,
        /// Governed maximum.
        maximum: usize,
    },
    /// A preflight shape could not describe real canonical rows.
    BatchShape {
        /// Refused row family.
        family: CommittedTickRowFamilyV2,
        /// Declared row count.
        rows: usize,
        /// Declared canonical body bytes.
        body_bytes: usize,
    },
    /// Aggregate row count exceeded its governed ceiling.
    AggregateRows {
        /// Received row count.
        actual: usize,
        /// Governed maximum.
        maximum: usize,
    },
    /// Complete canonical envelope bytes exceeded their derived ceiling.
    EnvelopeBytes {
        /// Received or calculated byte length.
        actual: usize,
        /// Governed maximum.
        maximum: usize,
    },
    /// Checked size arithmetic overflowed.
    CapacityOverflow {
        /// Stable capacity name.
        field: &'static str,
    },
    /// A bounded length could not enter its canonical unsigned field.
    IntegerConversion {
        /// Stable converted field name.
        field: &'static str,
        /// Received value.
        value: usize,
    },
    /// Exact canonical allocation failed.
    Allocation {
        /// Stable allocation name.
        field: &'static str,
        /// Requested capacity.
        requested: usize,
    },
    /// Encoder output did not equal its checked preflight length.
    CanonicalLength {
        /// Checked expected length.
        expected: usize,
        /// Produced length.
        actual: usize,
    },
}

impl CommittedTickEnvelopeV2 {
    /// Validate, frame, and own all mandatory row families.
    ///
    /// # Errors
    /// Returns the first exact key, ordering, bound, conversion, arithmetic,
    /// allocation, or canonical-length refusal before exposing an envelope.
    pub fn compose(
        claim: TickCommitClaimV1,
        input: CommittedTickRowFamiliesV2,
    ) -> Result<Self, CommittedTickEnvelopeErrorV2> {
        let row_families = compose_row_families(input)?;
        let row_counts = row_families.each_ref().map(|batch| batch.rows.len());
        let body_bytes = row_families.each_ref().map(|batch| batch.body_bytes);
        let capacity = validate_committed_tick_envelope_bounds_v2(row_counts, body_bytes)?;
        let total_rows = checked_sum(&row_counts, "committed tick aggregate rows")?;
        let mut canonical_bytes = reserve_bytes(capacity)?;
        append_claim(&mut canonical_bytes, &claim)?;
        append_row_families(&mut canonical_bytes, &row_families)?;
        if canonical_bytes.len() != capacity {
            return Err(CommittedTickEnvelopeErrorV2::CanonicalLength {
                expected: capacity,
                actual: canonical_bytes.len(),
            });
        }
        let digest = CommittedTickEnvelopeDigestV2(sha256_of(&canonical_bytes));
        Ok(Self {
            claim,
            row_families,
            total_rows,
            canonical_bytes,
            digest,
        })
    }

    /// Return the exact live V2 marker claim.
    #[must_use]
    pub const fn claim(&self) -> TickCommitClaimV1 {
        self.claim
    }

    /// Borrow all six batches in their governed order.
    #[must_use]
    pub const fn row_families(
        &self,
    ) -> &[CommittedTickRowBatchV2; COMMITTED_TICK_ROW_FAMILY_COUNT_V2] {
        &self.row_families
    }

    /// Return aggregate row count across every family.
    #[must_use]
    pub const fn total_rows(&self) -> usize {
        self.total_rows
    }

    /// Borrow exact complete-envelope bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Return the exact complete-envelope diagnostic.
    #[must_use]
    pub const fn digest(&self) -> CommittedTickEnvelopeDigestV2 {
        self.digest
    }

    /// Classify this requested envelope against one existing envelope.
    ///
    /// # Errors
    /// Returns a typed claim conflict before payload comparison, or a whole-
    /// payload mismatch when the claim matches but any canonical byte differs.
    pub fn classify_retry_against(
        &self,
        existing: &Self,
    ) -> Result<CommittedTickEnvelopeRetryV2, CommittedTickEnvelopeConflictV2> {
        self.claim
            .classify_retry_against(&existing.claim)
            .map_err(CommittedTickEnvelopeConflictV2::Claim)?;
        if self.canonical_bytes == existing.canonical_bytes {
            return Ok(CommittedTickEnvelopeRetryV2::Idempotent);
        }
        Err(CommittedTickEnvelopeConflictV2::WholePayloadMismatch {
            campaign_id: self.claim.campaign_id(),
            resolve_tick: self.claim.resolve_tick(),
            existing: existing.digest,
            requested: self.digest,
        })
    }
}

/// Validate cumulative row and byte ceilings without allocating the payload.
///
/// The body byte count includes each row's two four-byte length fields, exact
/// key bytes, and exact payload bytes. The returned value is the complete
/// canonical envelope capacity.
///
/// # Errors
/// Returns the first impossible shape, per-family byte overflow, aggregate-row
/// overflow, arithmetic overflow, or complete-envelope overflow.
pub fn validate_committed_tick_envelope_bounds_v2(
    row_counts: [usize; COMMITTED_TICK_ROW_FAMILY_COUNT_V2],
    batch_body_bytes: [usize; COMMITTED_TICK_ROW_FAMILY_COUNT_V2],
) -> Result<usize, CommittedTickEnvelopeErrorV2> {
    match row_counts[COMMITTED_TICK_ROW_FAMILY_COUNT_V2 - 1] {
        0 => return Err(CommittedTickEnvelopeErrorV2::MissingArchiveDirtyReceipt),
        1 => {}
        actual => {
            return Err(CommittedTickEnvelopeErrorV2::DuplicateArchiveDirtyReceipt { actual });
        }
    }
    for index in 0..COMMITTED_TICK_ROW_FAMILY_COUNT_V2 {
        let family = ALL_COMMITTED_TICK_ROW_FAMILIES_V2[index];
        validate_batch_shape(family, row_counts[index], batch_body_bytes[index])?;
    }
    let total_rows = checked_sum(&row_counts, "committed tick aggregate rows")?;
    if total_rows > MAX_COMMITTED_TICK_ROWS_V2 {
        return Err(CommittedTickEnvelopeErrorV2::AggregateRows {
            actual: total_rows,
            maximum: MAX_COMMITTED_TICK_ROWS_V2,
        });
    }
    let body_bytes = checked_sum(&batch_body_bytes, "committed tick family bytes")?;
    let envelope_bytes = FIXED_ENVELOPE_BYTES.checked_add(body_bytes).ok_or(
        CommittedTickEnvelopeErrorV2::CapacityOverflow {
            field: "committed tick envelope bytes",
        },
    )?;
    if envelope_bytes > MAX_COMMITTED_TICK_ENVELOPE_BYTES_V2 {
        return Err(CommittedTickEnvelopeErrorV2::EnvelopeBytes {
            actual: envelope_bytes,
            maximum: MAX_COMMITTED_TICK_ENVELOPE_BYTES_V2,
        });
    }
    Ok(envelope_bytes)
}

fn compose_row_families(
    input: CommittedTickRowFamiliesV2,
) -> Result<
    [CommittedTickRowBatchV2; COMMITTED_TICK_ROW_FAMILY_COUNT_V2],
    CommittedTickEnvelopeErrorV2,
> {
    Ok([
        compose_batch(CommittedTickRowFamilyV2::Graph, input.graph)?,
        compose_batch(CommittedTickRowFamilyV2::State, input.state)?,
        compose_batch(CommittedTickRowFamilyV2::Event, input.event)?,
        compose_batch(
            CommittedTickRowFamilyV2::ChoiceReceipt,
            input.choice_receipt,
        )?,
        compose_batch(CommittedTickRowFamilyV2::Checkpoint, input.checkpoint)?,
        compose_singular_archive_receipt(input.archive_dirty_receipt)?,
    ])
}

fn compose_singular_archive_receipt(
    row: CommittedTickRowV2,
) -> Result<CommittedTickRowBatchV2, CommittedTickEnvelopeErrorV2> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(1)
        .map_err(|_| CommittedTickEnvelopeErrorV2::Allocation {
            field: "archive dirty receipt rows",
            requested: 1,
        })?;
    rows.push(row);
    compose_batch(CommittedTickRowFamilyV2::ArchiveDirtyReceipt, rows)
}

fn compose_batch(
    family: CommittedTickRowFamilyV2,
    rows: Vec<CommittedTickRowV2>,
) -> Result<CommittedTickRowBatchV2, CommittedTickEnvelopeErrorV2> {
    if rows.len() > MAX_COMMITTED_TICK_ROWS_V2 {
        return Err(CommittedTickEnvelopeErrorV2::AggregateRows {
            actual: rows.len(),
            maximum: MAX_COMMITTED_TICK_ROWS_V2,
        });
    }
    let mut body_bytes = 0_usize;
    for index in 0..rows.len() {
        validate_row_order(family, &rows, index)?;
        body_bytes = checked_row_body_bytes(body_bytes, &rows[index])?;
        if body_bytes > MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2 {
            return Err(CommittedTickEnvelopeErrorV2::BatchBytes {
                family,
                actual: body_bytes,
                maximum: MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2,
            });
        }
    }
    Ok(CommittedTickRowBatchV2 {
        family,
        rows,
        body_bytes,
    })
}

fn validate_row_order(
    family: CommittedTickRowFamilyV2,
    rows: &[CommittedTickRowV2],
    index: usize,
) -> Result<(), CommittedTickEnvelopeErrorV2> {
    if index == 0 {
        return Ok(());
    }
    match rows[index - 1].key.cmp(&rows[index].key) {
        std::cmp::Ordering::Less => Ok(()),
        std::cmp::Ordering::Equal => {
            Err(CommittedTickEnvelopeErrorV2::DuplicateRowKey { family, index })
        }
        std::cmp::Ordering::Greater => {
            Err(CommittedTickEnvelopeErrorV2::RowOrder { family, index })
        }
    }
}

fn checked_row_body_bytes(
    current: usize,
    row: &CommittedTickRowV2,
) -> Result<usize, CommittedTickEnvelopeErrorV2> {
    current
        .checked_add(ROW_LENGTH_BYTES)
        .and_then(|value| value.checked_add(row.key.len()))
        .and_then(|value| value.checked_add(row.payload.len()))
        .ok_or(CommittedTickEnvelopeErrorV2::CapacityOverflow {
            field: "committed tick row body",
        })
}

fn validate_batch_shape(
    family: CommittedTickRowFamilyV2,
    rows: usize,
    body_bytes: usize,
) -> Result<(), CommittedTickEnvelopeErrorV2> {
    if body_bytes > MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2 {
        return Err(CommittedTickEnvelopeErrorV2::BatchBytes {
            family,
            actual: body_bytes,
            maximum: MAX_COMMITTED_TICK_ROW_BATCH_BYTES_V2,
        });
    }
    let minimum_body_bytes = rows.checked_mul(ROW_LENGTH_BYTES + 1).ok_or(
        CommittedTickEnvelopeErrorV2::CapacityOverflow {
            field: "committed tick minimum family bytes",
        },
    )?;
    if body_bytes < minimum_body_bytes || (rows == 0 && body_bytes != 0) {
        return Err(CommittedTickEnvelopeErrorV2::BatchShape {
            family,
            rows,
            body_bytes,
        });
    }
    Ok(())
}

fn checked_sum<const N: usize>(
    values: &[usize; N],
    field: &'static str,
) -> Result<usize, CommittedTickEnvelopeErrorV2> {
    let mut total = 0_usize;
    for value in values.iter().take(N) {
        total = total
            .checked_add(*value)
            .ok_or(CommittedTickEnvelopeErrorV2::CapacityOverflow { field })?;
    }
    Ok(total)
}

fn reserve_bytes(capacity: usize) -> Result<Vec<u8>, CommittedTickEnvelopeErrorV2> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(
            |_: TryReserveError| CommittedTickEnvelopeErrorV2::Allocation {
                field: "committed tick envelope",
                requested: capacity,
            },
        )?;
    Ok(bytes)
}

fn append_claim(
    output: &mut Vec<u8>,
    claim: &TickCommitClaimV1,
) -> Result<(), CommittedTickEnvelopeErrorV2> {
    output.extend_from_slice(DOMAIN);
    output.extend_from_slice(&COMMITTED_TICK_ENVELOPE_LAYOUT_VERSION_V2.to_be_bytes());
    output.push(CLAIM_TAG);
    append_u32(
        output,
        TICK_COMMIT_CLAIM_BYTES_V1,
        "tick commit claim bytes",
    )?;
    output.extend_from_slice(claim.canonical_bytes());
    Ok(())
}

fn append_row_families(
    output: &mut Vec<u8>,
    families: &[CommittedTickRowBatchV2; COMMITTED_TICK_ROW_FAMILY_COUNT_V2],
) -> Result<(), CommittedTickEnvelopeErrorV2> {
    for family in families.iter().take(COMMITTED_TICK_ROW_FAMILY_COUNT_V2) {
        output.push(family.family.tag());
        append_u32(output, family.rows.len(), "committed tick family row count")?;
        append_u32(
            output,
            family.body_bytes,
            "committed tick family body bytes",
        )?;
        for row in family.rows.iter().take(MAX_COMMITTED_TICK_ROWS_V2) {
            append_u32(output, row.key.len(), "committed tick row key bytes")?;
            output.extend_from_slice(&row.key);
            append_u32(
                output,
                row.payload.len(),
                "committed tick row payload bytes",
            )?;
            output.extend_from_slice(&row.payload);
        }
    }
    Ok(())
}

fn append_u32(
    output: &mut Vec<u8>,
    value: usize,
    field: &'static str,
) -> Result<(), CommittedTickEnvelopeErrorV2> {
    let converted = u32::try_from(value)
        .map_err(|_| CommittedTickEnvelopeErrorV2::IntegerConversion { field, value })?;
    output.extend_from_slice(&converted.to_be_bytes());
    Ok(())
}

impl std::fmt::Display for CommittedTickEnvelopeErrorV2 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "committed tick envelope refused: {self:?}")
    }
}

impl std::error::Error for CommittedTickEnvelopeErrorV2 {}

#[cfg(test)]
mod allocation_store_tests {
    use super::{CommittedTickEnvelopeV2, CommittedTickRowBatchV2, CommittedTickRowV2};

    #[test]
    fn private_owned_stores_remain_vec_backed_without_shrink_allocation() {
        fn assert_row_store(row: &CommittedTickRowV2) {
            let _: &Vec<u8> = &row.key;
            let _: &Vec<u8> = &row.payload;
        }
        fn assert_batch_store(batch: &CommittedTickRowBatchV2) {
            let _: &Vec<CommittedTickRowV2> = &batch.rows;
        }
        fn assert_envelope_store(envelope: &CommittedTickEnvelopeV2) {
            let _: &Vec<u8> = &envelope.canonical_bytes;
        }

        let _ = (
            assert_row_store as fn(&CommittedTickRowV2),
            assert_batch_store as fn(&CommittedTickRowBatchV2),
            assert_envelope_store as fn(&CommittedTickEnvelopeV2),
        );
    }
}
