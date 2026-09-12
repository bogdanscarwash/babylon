//! Canonical graph, state, event, choice, checkpoint and Archive component rows.

/// Number of typed component families, before material register and receipts.
pub const COMMITTED_TICK_ROW_FAMILY_COUNT: usize = 6;
/// Maximum aggregate typed component rows.
pub const MAX_COMMITTED_TICK_ROWS: usize = 1_048_576;
/// Maximum canonical row-body bytes in one family.
pub const MAX_COMMITTED_TICK_ROW_BATCH_BYTES: usize = 67_108_864;
const ROW_LENGTH_BYTES: usize = 8;

/// Closed, canonical order of durable outputs produced by one tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CommittedTickRowFamily {
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
pub const ALL_COMMITTED_TICK_ROW_FAMILIES: [CommittedTickRowFamily;
    COMMITTED_TICK_ROW_FAMILY_COUNT] = [
    CommittedTickRowFamily::Graph,
    CommittedTickRowFamily::State,
    CommittedTickRowFamily::Event,
    CommittedTickRowFamily::ChoiceReceipt,
    CommittedTickRowFamily::Checkpoint,
    CommittedTickRowFamily::ArchiveDirtyReceipt,
];

impl CommittedTickRowFamily {
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
pub struct CommittedTickRow {
    key: Vec<u8>,
    payload: Vec<u8>,
}

impl CommittedTickRow {
    /// Own exact key and payload bytes without interpreting their material meaning.
    ///
    /// # Errors
    /// Returns [`CommittedTickEnvelopeError::EmptyRowKey`] for an empty key.
    pub fn compose(key: Vec<u8>, payload: Vec<u8>) -> Result<Self, CommittedTickEnvelopeError> {
        if key.is_empty() {
            return Err(CommittedTickEnvelopeError::EmptyRowKey);
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
pub struct CommittedTickRowFamilies {
    /// Stable graph projection rows.
    pub graph: Vec<CommittedTickRow>,
    /// Material runtime-state rows.
    pub state: Vec<CommittedTickRow>,
    /// Governed event rows.
    pub event: Vec<CommittedTickRow>,
    /// Exact material-choice evidence rows.
    pub choice_receipt: Vec<CommittedTickRow>,
    /// Complete or delta checkpoint rows.
    pub checkpoint: Vec<CommittedTickRow>,
    /// Mandatory singular campaign work receipt.
    pub archive_dirty_receipt: CommittedTickRow,
}

/// One checked, strictly key-ordered family batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedTickRowBatch {
    family: CommittedTickRowFamily,
    rows: Vec<CommittedTickRow>,
    body_bytes: usize,
}

impl CommittedTickRowBatch {
    /// Return the closed family discriminator.
    #[must_use]
    pub const fn family(&self) -> CommittedTickRowFamily {
        self.family
    }

    /// Borrow the strict key-ordered exact rows.
    #[must_use]
    pub fn rows(&self) -> &[CommittedTickRow] {
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommittedTickEnvelopeError {
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
        family: CommittedTickRowFamily,
        /// Zero-based row index that broke ordering.
        index: usize,
    },
    /// Two rows in one family carried the same exact key bytes.
    DuplicateRowKey {
        /// Refused row family.
        family: CommittedTickRowFamily,
        /// Zero-based duplicate row index.
        index: usize,
    },
    /// A family body exceeded its governed byte ceiling.
    BatchBytes {
        /// Refused row family.
        family: CommittedTickRowFamily,
        /// Received canonical body length.
        actual: usize,
        /// Governed maximum.
        maximum: usize,
    },
    /// A preflight shape could not describe real canonical rows.
    BatchShape {
        /// Refused row family.
        family: CommittedTickRowFamily,
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
    /// Checked size arithmetic overflowed.
    CapacityOverflow {
        /// Stable capacity name.
        field: &'static str,
    },
    /// Exact canonical allocation failed.
    Allocation {
        /// Stable allocation name.
        field: &'static str,
        /// Requested capacity.
        requested: usize,
    },
}

/// Validate cumulative row and byte ceilings without allocating the payload.
///
/// The body byte count includes each row's two four-byte length fields, exact
/// key bytes, and exact payload bytes. The returned value is the complete
/// component body byte count.
///
/// # Errors
/// Returns the first impossible shape, per-family byte overflow, aggregate-row
/// overflow or arithmetic overflow.
pub fn validate_committed_tick_envelope_bounds(
    row_counts: [usize; COMMITTED_TICK_ROW_FAMILY_COUNT],
    batch_body_bytes: [usize; COMMITTED_TICK_ROW_FAMILY_COUNT],
) -> Result<usize, CommittedTickEnvelopeError> {
    match row_counts[COMMITTED_TICK_ROW_FAMILY_COUNT - 1] {
        0 => return Err(CommittedTickEnvelopeError::MissingArchiveDirtyReceipt),
        1 => {}
        actual => {
            return Err(CommittedTickEnvelopeError::DuplicateArchiveDirtyReceipt { actual });
        }
    }
    for index in 0..COMMITTED_TICK_ROW_FAMILY_COUNT {
        let family = ALL_COMMITTED_TICK_ROW_FAMILIES[index];
        validate_batch_shape(family, row_counts[index], batch_body_bytes[index])?;
    }
    let total_rows = checked_sum(&row_counts, "committed tick aggregate rows")?;
    if total_rows > MAX_COMMITTED_TICK_ROWS {
        return Err(CommittedTickEnvelopeError::AggregateRows {
            actual: total_rows,
            maximum: MAX_COMMITTED_TICK_ROWS,
        });
    }
    let body_bytes = checked_sum(&batch_body_bytes, "committed tick family bytes")?;
    Ok(body_bytes)
}

pub(crate) fn compose_row_families(
    input: CommittedTickRowFamilies,
) -> Result<[CommittedTickRowBatch; COMMITTED_TICK_ROW_FAMILY_COUNT], CommittedTickEnvelopeError> {
    Ok([
        compose_batch(CommittedTickRowFamily::Graph, input.graph)?,
        compose_batch(CommittedTickRowFamily::State, input.state)?,
        compose_batch(CommittedTickRowFamily::Event, input.event)?,
        compose_batch(CommittedTickRowFamily::ChoiceReceipt, input.choice_receipt)?,
        compose_batch(CommittedTickRowFamily::Checkpoint, input.checkpoint)?,
        compose_singular_archive_receipt(input.archive_dirty_receipt)?,
    ])
}

fn compose_singular_archive_receipt(
    row: CommittedTickRow,
) -> Result<CommittedTickRowBatch, CommittedTickEnvelopeError> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(1)
        .map_err(|_| CommittedTickEnvelopeError::Allocation {
            field: "archive dirty receipt rows",
            requested: 1,
        })?;
    rows.push(row);
    compose_batch(CommittedTickRowFamily::ArchiveDirtyReceipt, rows)
}

fn compose_batch(
    family: CommittedTickRowFamily,
    rows: Vec<CommittedTickRow>,
) -> Result<CommittedTickRowBatch, CommittedTickEnvelopeError> {
    if rows.len() > MAX_COMMITTED_TICK_ROWS {
        return Err(CommittedTickEnvelopeError::AggregateRows {
            actual: rows.len(),
            maximum: MAX_COMMITTED_TICK_ROWS,
        });
    }
    let mut body_bytes = 0_usize;
    for index in 0..rows.len() {
        validate_row_order(family, &rows, index)?;
        body_bytes = checked_row_body_bytes(body_bytes, &rows[index])?;
        if body_bytes > MAX_COMMITTED_TICK_ROW_BATCH_BYTES {
            return Err(CommittedTickEnvelopeError::BatchBytes {
                family,
                actual: body_bytes,
                maximum: MAX_COMMITTED_TICK_ROW_BATCH_BYTES,
            });
        }
    }
    Ok(CommittedTickRowBatch {
        family,
        rows,
        body_bytes,
    })
}

fn validate_row_order(
    family: CommittedTickRowFamily,
    rows: &[CommittedTickRow],
    index: usize,
) -> Result<(), CommittedTickEnvelopeError> {
    if index == 0 {
        return Ok(());
    }
    match rows[index - 1].key.cmp(&rows[index].key) {
        std::cmp::Ordering::Less => Ok(()),
        std::cmp::Ordering::Equal => {
            Err(CommittedTickEnvelopeError::DuplicateRowKey { family, index })
        }
        std::cmp::Ordering::Greater => Err(CommittedTickEnvelopeError::RowOrder { family, index }),
    }
}

fn checked_row_body_bytes(
    current: usize,
    row: &CommittedTickRow,
) -> Result<usize, CommittedTickEnvelopeError> {
    current
        .checked_add(ROW_LENGTH_BYTES)
        .and_then(|value| value.checked_add(row.key.len()))
        .and_then(|value| value.checked_add(row.payload.len()))
        .ok_or(CommittedTickEnvelopeError::CapacityOverflow {
            field: "committed tick row body",
        })
}

fn validate_batch_shape(
    family: CommittedTickRowFamily,
    rows: usize,
    body_bytes: usize,
) -> Result<(), CommittedTickEnvelopeError> {
    if body_bytes > MAX_COMMITTED_TICK_ROW_BATCH_BYTES {
        return Err(CommittedTickEnvelopeError::BatchBytes {
            family,
            actual: body_bytes,
            maximum: MAX_COMMITTED_TICK_ROW_BATCH_BYTES,
        });
    }
    let minimum_body_bytes = rows.checked_mul(ROW_LENGTH_BYTES + 1).ok_or(
        CommittedTickEnvelopeError::CapacityOverflow {
            field: "committed tick minimum family bytes",
        },
    )?;
    if body_bytes < minimum_body_bytes || (rows == 0 && body_bytes != 0) {
        return Err(CommittedTickEnvelopeError::BatchShape {
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
) -> Result<usize, CommittedTickEnvelopeError> {
    let mut total = 0_usize;
    for value in values.iter().take(N) {
        total = total
            .checked_add(*value)
            .ok_or(CommittedTickEnvelopeError::CapacityOverflow { field })?;
    }
    Ok(total)
}

impl std::fmt::Display for CommittedTickEnvelopeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "committed tick envelope refused: {self:?}")
    }
}

impl std::error::Error for CommittedTickEnvelopeError {}

#[cfg(test)]
mod tests;
