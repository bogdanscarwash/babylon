//! Session-scoped identity for canonically ordered accepted Practice actions.

use std::collections::TryReserveError;

use babylon_kernel::content_digest::sha256_of;
use babylon_kernel::replay::{ReplayIdentityError, ReplaySessionId};
use babylon_kernel::tick_content_hash::OrderedPracticeActionBatchDigest;

use crate::{
    encode_practice_intent, practice_intent_digest, validate_resolved_practice_batch,
    PracticeInputAuthorityLedger, PracticeIntent, PracticeIntentError, ResolvedPracticeBatch,
    ResolvedPracticeBatchError, MAX_PRACTICE_INTENT_CANONICAL_BYTES,
    MAX_RESOLVED_PRACTICE_BATCH_ITEMS,
};

/// Exact `ActionId` preimage domain without its mandatory NUL terminator.
pub const PRACTICE_ACTION_ID_DOMAIN_BYTES: &[u8] = b"babylon.practice-action-id.v1";
/// Exact ordered accepted-action batch domain without its mandatory NUL terminator.
pub const ORDERED_PRACTICE_ACTION_BATCH_DOMAIN_BYTES: &[u8] =
    b"babylon.ordered-practice-action-batch.v1";
/// Governed `ActionId` schema version.
pub const PRACTICE_ACTION_ID_SCHEMA_VERSION: u16 = 1;
/// Governed ordered accepted-action batch layout version.
pub const ORDERED_PRACTICE_ACTION_BATCH_LAYOUT_VERSION: u32 = 1;

const PRACTICE_INTENT_SCHEMA_VERSION: u16 = 2;
const ORDERED_BATCH_SCHEMA_VERSION: u16 = 1;
const ACTION_ID_FIXED_BYTES: usize = 68;
const ORDERED_BATCH_FIXED_BYTES: usize = 55;
const ORDERED_ACTION_FIXED_BYTES: usize = 36;
const MAX_ORDERED_PRACTICE_ACTION_BATCH_BYTES: usize = 9_302_326;

/// Exact identity of one accepted Practice current intent in a replay session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PracticeActionId([u8; 32]);

impl PracticeActionId {
    /// Borrow the exact 32 digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// One privately constructed action in canonical proposal order.
///
/// These public types can be imported and named:
///
/// ```
/// use babylon_practice_contract::{OrderedPracticeAction, PracticeActionId, PracticeIntent};
/// fn inspect(action: &OrderedPracticeAction) -> (&PracticeActionId, &PracticeIntent) {
///     (action.action_id(), action.intent())
/// }
/// ```
///
/// Callers cannot supply an ordinal or identity:
///
/// ```compile_fail,E0451
/// use babylon_practice_contract::{
///     OrderedPracticeAction, PracticeActionId, PracticeIntent,
/// };
///
/// fn forge(intent: PracticeIntent, action_id: PracticeActionId) {
///     let _ = OrderedPracticeAction {
///         canonical_input_ordinal: 0,
///         action_id,
///         intent,
///     };
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedPracticeAction {
    canonical_input_ordinal: u16,
    action_id: PracticeActionId,
    intent: PracticeIntent,
}

impl OrderedPracticeAction {
    /// Return the zero-based canonical input ordinal.
    #[must_use]
    pub const fn canonical_input_ordinal(&self) -> u16 {
        self.canonical_input_ordinal
    }

    /// Borrow the recomputed session-scoped action identity.
    #[must_use]
    pub const fn action_id(&self) -> &PracticeActionId {
        &self.action_id
    }

    /// Borrow the exact accepted Practice current intent.
    #[must_use]
    pub const fn intent(&self) -> &PracticeIntent {
        &self.intent
    }
}

/// One checked ordered accepted-action batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedPracticeActionBatch {
    session: ReplaySessionId,
    resolve_tick: u64,
    items: Vec<OrderedPracticeAction>,
    canonical_bytes: Vec<u8>,
    digest: OrderedPracticeActionBatchDigest,
}

/// Checked ordered-action projection or encoding failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderedPracticeActionError {
    /// Replay-session canonical encoding failed.
    Replay(ReplayIdentityError),
    /// The trusted resolved-batch source refused validation.
    Source(ResolvedPracticeBatchError),
    /// Nested intent encoding failed after source validation.
    Intent(PracticeIntentError),
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

impl From<ResolvedPracticeBatchError> for OrderedPracticeActionError {
    fn from(value: ResolvedPracticeBatchError) -> Self {
        Self::Source(value)
    }
}

impl From<PracticeIntentError> for OrderedPracticeActionError {
    fn from(value: PracticeIntentError) -> Self {
        Self::Intent(value)
    }
}

/// Encode the exact bytes hashed for one session-scoped Practice `ActionId`.
///
/// # Errors
/// Returns a checked replay-session, intent, arithmetic, or allocation error.
pub fn encode_practice_action_id_preimage(
    session: &ReplaySessionId,
    intent: &PracticeIntent,
) -> Result<Vec<u8>, OrderedPracticeActionError> {
    let session_bytes = session.canonical_bytes()?;
    let capacity = ACTION_ID_FIXED_BYTES
        .checked_add(session.as_bytes().len())
        .ok_or(OrderedPracticeActionError::CapacityOverflow {
            field: "practice action id preimage",
        })?;
    let mut output = reserve_bytes("practice action id preimage", capacity)?;
    output.extend_from_slice(PRACTICE_ACTION_ID_DOMAIN_BYTES);
    output.push(0);
    output.extend_from_slice(&PRACTICE_ACTION_ID_SCHEMA_VERSION.to_be_bytes());
    output.extend_from_slice(&session_bytes);
    output.extend_from_slice(&PRACTICE_INTENT_SCHEMA_VERSION.to_be_bytes());
    output.extend_from_slice(&practice_intent_digest(intent)?);
    debug_assert_eq!(output.len(), capacity);
    Ok(output)
}

/// Derive one session-scoped `ActionId` from an exact Practice current intent.
///
/// # Errors
/// Returns the first checked preimage encoding error.
pub fn practice_action_id(
    session: &ReplaySessionId,
    intent: &PracticeIntent,
) -> Result<PracticeActionId, OrderedPracticeActionError> {
    let preimage = encode_practice_action_id_preimage(session, intent)?;
    Ok(PracticeActionId(sha256_of(&preimage)))
}

impl OrderedPracticeActionBatch {
    /// Construct the exact Gate 3 empty action batch.
    ///
    /// # Errors
    /// Returns a checked replay-session, arithmetic, or allocation error.
    pub fn empty(
        session: ReplaySessionId,
        resolve_tick: u64,
    ) -> Result<Self, OrderedPracticeActionError> {
        build_batch(session, resolve_tick, &[])
    }

    /// Project a fully validated Practice current source batch into private actions.
    ///
    /// This proves structural consistency against the supplied trusted ledger;
    /// it does not confer accepted-input or persistence provenance.
    ///
    /// # Errors
    /// Returns the first source-validation, nested encoding, bound, or
    /// allocation error.
    pub fn project(
        session: ReplaySessionId,
        source: &ResolvedPracticeBatch,
        trusted_ledger: &PracticeInputAuthorityLedger,
    ) -> Result<Self, OrderedPracticeActionError> {
        validate_resolved_practice_batch(source, trusted_ledger)?;
        build_batch(session, source.resolve_tick, &source.items)
    }

    /// Borrow the checked replay-session identity.
    #[must_use]
    pub const fn session(&self) -> &ReplaySessionId {
        &self.session
    }

    /// Return the one resolve tick bound by this batch.
    #[must_use]
    pub const fn resolve_tick(&self) -> u64 {
        self.resolve_tick
    }

    /// Borrow the actions in canonical proposal order.
    #[must_use]
    pub fn items(&self) -> &[OrderedPracticeAction] {
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
    pub const fn digest(&self) -> OrderedPracticeActionBatchDigest {
        self.digest
    }
}

fn build_batch(
    session: ReplaySessionId,
    resolve_tick: u64,
    source_items: &[crate::ResolvedPracticeBatchItem],
) -> Result<OrderedPracticeActionBatch, OrderedPracticeActionError> {
    let mut items = reserve_items(source_items.len())?;
    let mut capacity = ORDERED_BATCH_FIXED_BYTES
        .checked_add(session.as_bytes().len())
        .ok_or(OrderedPracticeActionError::CapacityOverflow {
            field: "ordered practice action batch",
        })?;
    for (index, source) in source_items
        .iter()
        .take(MAX_RESOLVED_PRACTICE_BATCH_ITEMS + 1)
        .enumerate()
    {
        let ordinal =
            u16::try_from(index).map_err(|_| OrderedPracticeActionError::IntegerConversion {
                field: "canonical input ordinal",
                value: index,
            })?;
        let intent_bytes = encode_practice_intent(&source.intent)?;
        validate_intent_length(index, intent_bytes.len())?;
        capacity = checked_batch_capacity(capacity, intent_bytes.len())?;
        items.push(OrderedPracticeAction {
            canonical_input_ordinal: ordinal,
            action_id: practice_action_id(&session, &source.intent)?,
            intent: source.intent.clone(),
        });
    }
    encode_batch(session, resolve_tick, items, capacity)
}

fn encode_batch(
    session: ReplaySessionId,
    resolve_tick: u64,
    items: Vec<OrderedPracticeAction>,
    capacity: usize,
) -> Result<OrderedPracticeActionBatch, OrderedPracticeActionError> {
    let session_bytes = session.canonical_bytes()?;
    let item_count =
        u16::try_from(items.len()).map_err(|_| OrderedPracticeActionError::IntegerConversion {
            field: "ordered practice action count",
            value: items.len(),
        })?;
    let mut canonical_bytes = reserve_bytes("ordered practice action batch", capacity)?;
    canonical_bytes.extend_from_slice(ORDERED_PRACTICE_ACTION_BATCH_DOMAIN_BYTES);
    canonical_bytes.push(0);
    canonical_bytes.extend_from_slice(&ORDERED_BATCH_SCHEMA_VERSION.to_be_bytes());
    canonical_bytes.extend_from_slice(&session_bytes);
    canonical_bytes.extend_from_slice(&resolve_tick.to_be_bytes());
    canonical_bytes.extend_from_slice(&item_count.to_be_bytes());
    append_ordered_items(&mut canonical_bytes, &items)?;
    debug_assert_eq!(canonical_bytes.len(), capacity);
    let digest = OrderedPracticeActionBatchDigest::from_bytes(sha256_of(&canonical_bytes));
    Ok(OrderedPracticeActionBatch {
        session,
        resolve_tick,
        items,
        canonical_bytes,
        digest,
    })
}

fn append_ordered_items(
    output: &mut Vec<u8>,
    items: &[OrderedPracticeAction],
) -> Result<(), OrderedPracticeActionError> {
    for item in items.iter().take(MAX_RESOLVED_PRACTICE_BATCH_ITEMS + 1) {
        let intent_bytes = encode_practice_intent(&item.intent)?;
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
    if length <= MAX_PRACTICE_INTENT_CANONICAL_BYTES {
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
    if updated <= MAX_ORDERED_PRACTICE_ACTION_BATCH_BYTES {
        Ok(updated)
    } else {
        Err(OrderedPracticeActionError::BatchLength { actual: updated })
    }
}

fn reserve_items(count: usize) -> Result<Vec<OrderedPracticeAction>, OrderedPracticeActionError> {
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
        MAX_ORDERED_PRACTICE_ACTION_BATCH_BYTES, MAX_PRACTICE_INTENT_CANONICAL_BYTES,
    };

    #[test]
    fn ordered_action_encoders_accept_each_maximum_and_refuse_plus_one() {
        assert_eq!(
            validate_intent_length(0, MAX_PRACTICE_INTENT_CANONICAL_BYTES),
            Ok(())
        );
        assert_eq!(
            validate_intent_length(0, MAX_PRACTICE_INTENT_CANONICAL_BYTES + 1),
            Err(OrderedPracticeActionError::IntentLength {
                index: 0,
                actual: MAX_PRACTICE_INTENT_CANONICAL_BYTES + 1,
            })
        );
        let accepted_capacity = MAX_ORDERED_PRACTICE_ACTION_BATCH_BYTES - 37;
        assert_eq!(
            checked_batch_capacity(accepted_capacity, 1),
            Ok(MAX_ORDERED_PRACTICE_ACTION_BATCH_BYTES)
        );
        assert_eq!(
            checked_batch_capacity(accepted_capacity + 1, 1),
            Err(OrderedPracticeActionError::BatchLength {
                actual: MAX_ORDERED_PRACTICE_ACTION_BATCH_BYTES + 1,
            })
        );
    }
}
