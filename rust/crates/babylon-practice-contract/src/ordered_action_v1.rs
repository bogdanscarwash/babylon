//! Session-scoped identity for canonically ordered accepted Practice actions.

use std::collections::TryReserveError;

use babylon_kernel::replay::{ReplayIdentityError, ReplaySessionIdV1};
use babylon_kernel::sha256_of;
use babylon_kernel::tick_content_hash::OrderedPracticeActionBatchDigestV1;

use crate::{
    encode_practice_intent_v2, practice_intent_v2_digest, validate_resolved_practice_batch_v2,
    PracticeInputAuthorityLedgerV2, PracticeIntentV2, PracticeIntentV2Error,
    ResolvedPracticeBatchV2, ResolvedPracticeBatchV2Error, MAX_PRACTICE_INTENT_CANONICAL_BYTES_V2,
    MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2,
};

/// Exact `ActionId` preimage domain without its mandatory NUL terminator.
pub const PRACTICE_ACTION_ID_V1_DOMAIN_BYTES: &[u8] = b"babylon.practice-action-id.v1";
/// Exact ordered accepted-action batch domain without its mandatory NUL terminator.
pub const ORDERED_PRACTICE_ACTION_BATCH_V1_DOMAIN_BYTES: &[u8] =
    b"babylon.ordered-practice-action-batch.v1";
/// Governed `ActionId` schema version.
pub const PRACTICE_ACTION_ID_V1_SCHEMA_VERSION: u16 = 1;
/// Governed ordered accepted-action batch layout version.
pub const ORDERED_PRACTICE_ACTION_BATCH_V1_LAYOUT_VERSION: u32 = 1;

const PRACTICE_INTENT_SCHEMA_VERSION: u16 = 2;
const ORDERED_BATCH_SCHEMA_VERSION: u16 = 1;
const ACTION_ID_FIXED_BYTES: usize = 68;
const ORDERED_BATCH_FIXED_BYTES: usize = 55;
const ORDERED_ACTION_FIXED_BYTES: usize = 36;
const MAX_ORDERED_PRACTICE_ACTION_BATCH_BYTES_V1: usize = 9_302_326;

/// Exact identity of one accepted Practice V2 intent in a replay session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PracticeActionIdV1([u8; 32]);

impl PracticeActionIdV1 {
    /// Borrow the exact 32 digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// One privately constructed action in canonical proposal order.
///
/// Callers cannot supply an ordinal or identity:
///
/// ```compile_fail
/// use babylon_practice_contract::{
///     OrderedPracticeActionV1, PracticeActionIdV1, PracticeIntentV2,
/// };
///
/// fn forge(intent: PracticeIntentV2, action_id: PracticeActionIdV1) {
///     let _ = OrderedPracticeActionV1 {
///         canonical_input_ordinal: 0,
///         action_id,
///         intent,
///     };
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedPracticeActionV1 {
    canonical_input_ordinal: u16,
    action_id: PracticeActionIdV1,
    intent: PracticeIntentV2,
}

impl OrderedPracticeActionV1 {
    /// Return the zero-based canonical input ordinal.
    #[must_use]
    pub const fn canonical_input_ordinal(&self) -> u16 {
        self.canonical_input_ordinal
    }

    /// Borrow the recomputed session-scoped action identity.
    #[must_use]
    pub const fn action_id(&self) -> &PracticeActionIdV1 {
        &self.action_id
    }

    /// Borrow the exact accepted Practice V2 intent.
    #[must_use]
    pub const fn intent(&self) -> &PracticeIntentV2 {
        &self.intent
    }
}

/// One checked ordered accepted-action batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedPracticeActionBatchV1 {
    session: ReplaySessionIdV1,
    resolve_tick: u64,
    items: Vec<OrderedPracticeActionV1>,
    canonical_bytes: Vec<u8>,
    digest: OrderedPracticeActionBatchDigestV1,
}

/// Checked ordered-action projection or encoding failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderedPracticeActionError {
    /// Replay-session canonical encoding failed.
    Replay(ReplayIdentityError),
    /// The trusted resolved-batch source refused validation.
    Source(ResolvedPracticeBatchV2Error),
    /// Nested intent encoding failed after source validation.
    Intent(PracticeIntentV2Error),
    /// A bounded integer conversion failed.
    IntegerConversion {
        /// Stable name of the converted field.
        field: &'static str,
        /// Value that could not be represented.
        value: usize,
    },
    /// Checked canonical-size arithmetic overflowed.
    CapacityOverflow {
        /// Stable name of the capacity.
        field: &'static str,
    },
    /// One nested intent exceeded its governed canonical bound.
    IntentLength {
        /// Zero-based canonical item index.
        index: usize,
        /// Received intent byte length.
        actual: usize,
    },
    /// The complete batch exceeded its governed canonical bound.
    BatchLength {
        /// Received canonical byte length.
        actual: usize,
    },
    /// A bounded canonical allocation could not be reserved.
    Allocation {
        /// Stable name of the requested allocation.
        field: &'static str,
        /// Exact requested capacity or item count.
        requested: usize,
    },
}

impl From<ReplayIdentityError> for OrderedPracticeActionError {
    fn from(value: ReplayIdentityError) -> Self {
        Self::Replay(value)
    }
}

impl From<ResolvedPracticeBatchV2Error> for OrderedPracticeActionError {
    fn from(value: ResolvedPracticeBatchV2Error) -> Self {
        Self::Source(value)
    }
}

impl From<PracticeIntentV2Error> for OrderedPracticeActionError {
    fn from(value: PracticeIntentV2Error) -> Self {
        Self::Intent(value)
    }
}

/// Encode the exact bytes hashed for one session-scoped Practice `ActionId`.
///
/// # Errors
/// Returns a checked replay-session, intent, arithmetic, or allocation error.
pub fn encode_practice_action_id_preimage_v1(
    session: &ReplaySessionIdV1,
    intent: &PracticeIntentV2,
) -> Result<Vec<u8>, OrderedPracticeActionError> {
    let session_bytes = session.canonical_bytes()?;
    let capacity = ACTION_ID_FIXED_BYTES
        .checked_add(session.as_bytes().len())
        .ok_or(OrderedPracticeActionError::CapacityOverflow {
            field: "practice action id preimage",
        })?;
    let mut output = reserve_bytes("practice action id preimage", capacity)?;
    output.extend_from_slice(PRACTICE_ACTION_ID_V1_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&PRACTICE_ACTION_ID_V1_SCHEMA_VERSION.to_be_bytes());
    output.extend_from_slice(&session_bytes);
    output.extend_from_slice(&PRACTICE_INTENT_SCHEMA_VERSION.to_be_bytes());
    output.extend_from_slice(&practice_intent_v2_digest(intent)?);
    debug_assert_eq!(output.len(), capacity);
    Ok(output)
}

/// Derive one session-scoped `ActionId` from an exact Practice V2 intent.
///
/// # Errors
/// Returns the first checked preimage encoding error.
pub fn practice_action_id_v1(
    session: &ReplaySessionIdV1,
    intent: &PracticeIntentV2,
) -> Result<PracticeActionIdV1, OrderedPracticeActionError> {
    let preimage = encode_practice_action_id_preimage_v1(session, intent)?;
    Ok(PracticeActionIdV1(sha256_of(&preimage)))
}

impl OrderedPracticeActionBatchV1 {
    /// Construct the exact Gate 3 empty action batch.
    ///
    /// # Errors
    /// Returns a checked replay-session, arithmetic, or allocation error.
    pub fn empty(
        session: ReplaySessionIdV1,
        resolve_tick: u64,
    ) -> Result<Self, OrderedPracticeActionError> {
        build_batch(session, resolve_tick, &[])
    }

    /// Project a fully validated Practice V2 source batch into private actions.
    ///
    /// This proves structural consistency against the supplied trusted ledger;
    /// it does not confer accepted-input or persistence provenance.
    ///
    /// # Errors
    /// Returns the first source-validation, nested encoding, bound, or
    /// allocation error.
    pub fn project(
        session: ReplaySessionIdV1,
        source: &ResolvedPracticeBatchV2,
        trusted_ledger: &PracticeInputAuthorityLedgerV2,
    ) -> Result<Self, OrderedPracticeActionError> {
        validate_resolved_practice_batch_v2(source, trusted_ledger)?;
        build_batch(session, source.resolve_tick, &source.items)
    }

    /// Borrow the checked replay-session identity.
    #[must_use]
    pub const fn session(&self) -> &ReplaySessionIdV1 {
        &self.session
    }

    /// Return the one resolve tick bound by this batch.
    #[must_use]
    pub const fn resolve_tick(&self) -> u64 {
        self.resolve_tick
    }

    /// Borrow the actions in canonical proposal order.
    #[must_use]
    pub fn items(&self) -> &[OrderedPracticeActionV1] {
        &self.items
    }

    /// Return whether this is the exact live Gate 3 empty form.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Borrow the exact canonical batch bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Return the SHA-256 identity of the exact canonical bytes.
    #[must_use]
    pub const fn digest(&self) -> OrderedPracticeActionBatchDigestV1 {
        self.digest
    }
}

fn build_batch(
    session: ReplaySessionIdV1,
    resolve_tick: u64,
    source_items: &[crate::ResolvedPracticeBatchItemV2],
) -> Result<OrderedPracticeActionBatchV1, OrderedPracticeActionError> {
    let mut items = reserve_items(source_items.len())?;
    let mut capacity = ORDERED_BATCH_FIXED_BYTES
        .checked_add(session.as_bytes().len())
        .ok_or(OrderedPracticeActionError::CapacityOverflow {
            field: "ordered practice action batch",
        })?;
    for (index, source) in source_items
        .iter()
        .take(MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2 + 1)
        .enumerate()
    {
        let ordinal =
            u16::try_from(index).map_err(|_| OrderedPracticeActionError::IntegerConversion {
                field: "canonical input ordinal",
                value: index,
            })?;
        let intent_bytes = encode_practice_intent_v2(&source.intent)?;
        validate_intent_length(index, intent_bytes.len())?;
        capacity = checked_batch_capacity(capacity, intent_bytes.len())?;
        items.push(OrderedPracticeActionV1 {
            canonical_input_ordinal: ordinal,
            action_id: practice_action_id_v1(&session, &source.intent)?,
            intent: source.intent.clone(),
        });
    }
    encode_batch(session, resolve_tick, items, capacity)
}

fn encode_batch(
    session: ReplaySessionIdV1,
    resolve_tick: u64,
    items: Vec<OrderedPracticeActionV1>,
    capacity: usize,
) -> Result<OrderedPracticeActionBatchV1, OrderedPracticeActionError> {
    let session_bytes = session.canonical_bytes()?;
    let item_count =
        u16::try_from(items.len()).map_err(|_| OrderedPracticeActionError::IntegerConversion {
            field: "ordered practice action count",
            value: items.len(),
        })?;
    let mut canonical_bytes = reserve_bytes("ordered practice action batch", capacity)?;
    canonical_bytes.extend_from_slice(ORDERED_PRACTICE_ACTION_BATCH_V1_DOMAIN_BYTES);
    canonical_bytes.push(0);
    canonical_bytes.extend_from_slice(&ORDERED_BATCH_SCHEMA_VERSION.to_be_bytes());
    canonical_bytes.extend_from_slice(&session_bytes);
    canonical_bytes.extend_from_slice(&resolve_tick.to_be_bytes());
    canonical_bytes.extend_from_slice(&item_count.to_be_bytes());
    append_ordered_items(&mut canonical_bytes, &items)?;
    debug_assert_eq!(canonical_bytes.len(), capacity);
    let digest = OrderedPracticeActionBatchDigestV1::from_bytes(sha256_of(&canonical_bytes));
    Ok(OrderedPracticeActionBatchV1 {
        session,
        resolve_tick,
        items,
        canonical_bytes,
        digest,
    })
}

fn append_ordered_items(
    output: &mut Vec<u8>,
    items: &[OrderedPracticeActionV1],
) -> Result<(), OrderedPracticeActionError> {
    for item in items.iter().take(MAX_RESOLVED_PRACTICE_BATCH_ITEMS_V2 + 1) {
        let intent_bytes = encode_practice_intent_v2(&item.intent)?;
        let intent_length = u16::try_from(intent_bytes.len()).map_err(|_| {
            OrderedPracticeActionError::IntegerConversion {
                field: "ordered practice action intent length",
                value: intent_bytes.len(),
            }
        })?;
        output.extend_from_slice(&item.canonical_input_ordinal.to_be_bytes());
        output.extend_from_slice(item.action_id.as_bytes());
        output.extend_from_slice(&intent_length.to_be_bytes());
        output.extend_from_slice(&intent_bytes);
    }
    Ok(())
}

fn validate_intent_length(index: usize, length: usize) -> Result<(), OrderedPracticeActionError> {
    if length <= MAX_PRACTICE_INTENT_CANONICAL_BYTES_V2 {
        Ok(())
    } else {
        Err(OrderedPracticeActionError::IntentLength {
            index,
            actual: length,
        })
    }
}

fn checked_batch_capacity(
    capacity: usize,
    intent_length: usize,
) -> Result<usize, OrderedPracticeActionError> {
    let item_length = ORDERED_ACTION_FIXED_BYTES
        .checked_add(intent_length)
        .ok_or(OrderedPracticeActionError::CapacityOverflow {
            field: "ordered practice action item",
        })?;
    let updated =
        capacity
            .checked_add(item_length)
            .ok_or(OrderedPracticeActionError::CapacityOverflow {
                field: "ordered practice action batch",
            })?;
    if updated <= MAX_ORDERED_PRACTICE_ACTION_BATCH_BYTES_V1 {
        Ok(updated)
    } else {
        Err(OrderedPracticeActionError::BatchLength { actual: updated })
    }
}

fn reserve_items(count: usize) -> Result<Vec<OrderedPracticeActionV1>, OrderedPracticeActionError> {
    let mut items = Vec::new();
    items
        .try_reserve_exact(count)
        .map_err(
            |_: TryReserveError| OrderedPracticeActionError::Allocation {
                field: "ordered practice actions",
                requested: count,
            },
        )?;
    Ok(items)
}

fn reserve_bytes(
    field: &'static str,
    capacity: usize,
) -> Result<Vec<u8>, OrderedPracticeActionError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(capacity)
        .map_err(
            |_: TryReserveError| OrderedPracticeActionError::Allocation {
                field,
                requested: capacity,
            },
        )?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{
        checked_batch_capacity, validate_intent_length, OrderedPracticeActionError,
        MAX_ORDERED_PRACTICE_ACTION_BATCH_BYTES_V1, MAX_PRACTICE_INTENT_CANONICAL_BYTES_V2,
    };

    #[test]
    fn ordered_action_encoders_accept_each_maximum_and_refuse_plus_one() {
        assert_eq!(
            validate_intent_length(0, MAX_PRACTICE_INTENT_CANONICAL_BYTES_V2),
            Ok(())
        );
        assert_eq!(
            validate_intent_length(0, MAX_PRACTICE_INTENT_CANONICAL_BYTES_V2 + 1),
            Err(OrderedPracticeActionError::IntentLength {
                index: 0,
                actual: MAX_PRACTICE_INTENT_CANONICAL_BYTES_V2 + 1,
            })
        );
        let accepted_capacity = MAX_ORDERED_PRACTICE_ACTION_BATCH_BYTES_V1 - 37;
        assert_eq!(
            checked_batch_capacity(accepted_capacity, 1),
            Ok(MAX_ORDERED_PRACTICE_ACTION_BATCH_BYTES_V1)
        );
        assert_eq!(
            checked_batch_capacity(accepted_capacity + 1, 1),
            Err(OrderedPracticeActionError::BatchLength {
                actual: MAX_ORDERED_PRACTICE_ACTION_BATCH_BYTES_V1 + 1,
            })
        );
    }
}
