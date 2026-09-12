//! Database-free durable tick, checkpoint, and Archive work identities.

use std::collections::TryReserveError;

use babylon_kernel::content_digest::sha256_of;
use babylon_kernel::tick_content_hash::TickContentHash;
use babylon_tick::replay_session::IdentifiedTickReport;

use crate::committed_tick_envelope::CommittedTickRow;
use crate::identity::CampaignId;
use crate::runtime::RustPersistenceRuntimeError;
use crate::semantic_codec;

const FULL_CHECKPOINT_SECTION_COUNT: usize = 9;
const ROW_LENGTH_BYTES: usize = 8;

/// A durable one-based tick that fits `PostgreSQL` `BIGINT` exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommittedResolveTick(u64);

impl CommittedResolveTick {
    /// Return the positive PostgreSQL-compatible tick.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A durable tick-number domain refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommittedResolveTickError {
    /// Tick zero is foundation state, never a committed tick marker.
    SyntheticTickZero,
    /// The unsigned value cannot fit `PostgreSQL` `BIGINT`.
    OutOfPostgresRange,
}

impl TryFrom<u64> for CommittedResolveTick {
    type Error = CommittedResolveTickError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value == 0 {
            Err(CommittedResolveTickError::SyntheticTickZero)
        } else if value > i64::MAX as u64 {
            Err(CommittedResolveTickError::OutOfPostgresRange)
        } else {
            Ok(Self(value))
        }
    }
}

/// Closed checkpoint completeness marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointCompleteness {
    /// All nine required reconstruction sections are present.
    Full,
    /// A sparse continuation that is never a restart root.
    Delta,
}

impl CheckpointCompleteness {
    const fn tag(self) -> u8 {
        match self {
            Self::Full => 1,
            Self::Delta => 2,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Delta => "delta",
        }
    }
}

/// Closed order of the nine required full-checkpoint sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FullCheckpointSectionTag {
    StableGraph,
    WorldRegisters,
    ResolverManifest,
    PreparedEnvironment,
    ReplaySessionIdentity,
    RngSeed,
    ContentDigest,
    ReferenceDigest,
    SemanticState,
}

impl FullCheckpointSectionTag {
    /// Return the exact closed section tag.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::StableGraph => 1,
            Self::WorldRegisters => 2,
            Self::ResolverManifest => 3,
            Self::PreparedEnvironment => 4,
            Self::ReplaySessionIdentity => 5,
            Self::RngSeed => 6,
            Self::ContentDigest => 7,
            Self::ReferenceDigest => 8,
            Self::SemanticState => 9,
        }
    }
}

/// Digest-bound summary of one exact full-checkpoint source section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedCheckpointSection {
    tag: FullCheckpointSectionTag,
    row_count: u32,
    sha256: [u8; 32],
}

impl CommittedCheckpointSection {
    /// Return the closed section tag.
    #[must_use]
    pub const fn tag(&self) -> FullCheckpointSectionTag {
        self.tag
    }

    /// Return the exact source-owned row count.
    #[must_use]
    pub const fn row_count(&self) -> u32 {
        self.row_count
    }

    /// Return SHA-256 of the exact section bytes.
    #[must_use]
    pub const fn sha256(&self) -> [u8; 32] {
        self.sha256
    }
}

/// One exact nine-section restart-root manifest.
#[derive(Debug, PartialEq, Eq)]
pub struct CommittedFullCheckpoint {
    completeness: CheckpointCompleteness,
    sections: Vec<CommittedCheckpointSection>,
    exact_section_bytes: Vec<Vec<u8>>,
    rows: Vec<CommittedTickRow>,
    manifest_bytes: Vec<u8>,
    manifest_sha256: [u8; 32],
}

impl CommittedFullCheckpoint {
    /// Capture the nine exact report-owned reconstruction sections.
    ///
    /// # Errors
    /// Returns the first tick, section count, byte-bound, integer, capacity,
    /// or allocation refusal before exposing a partial checkpoint.
    pub fn capture(
        campaign_id: CampaignId,
        resolve_tick: CommittedResolveTick,
        report: &IdentifiedTickReport,
    ) -> Result<Self, RustPersistenceRuntimeError> {
        if u64::try_from(report.result_registers().completed_tick()).ok()
            != Some(resolve_tick.get())
        {
            return Err(RustPersistenceRuntimeError::ReplaySource);
        }
        let source_sections = checkpoint_source_sections(report)?;
        let sections = summarize_sections(&source_sections)?;
        let exact_section_bytes = source_sections
            .iter()
            .map(|source| copy_bytes("checkpoint retained section bytes", &source.bytes))
            .collect::<Result<Vec<_>, _>>()?;
        let rows = encode_checkpoint_rows_from_sources(&source_sections)?;
        Self::validate_restart_root(CheckpointCompleteness::Full, &sections)?;
        let encoded_sections = encoded_section_summaries(&sections)?;
        let manifest_bytes = semantic_codec::encode_full_checkpoint(
            campaign_id,
            resolve_tick.get(),
            &encoded_sections,
        )?;
        let manifest_sha256 = sha256_of(&manifest_bytes);
        Ok(Self {
            completeness: CheckpointCompleteness::Full,
            sections,
            exact_section_bytes,
            rows,
            manifest_bytes,
            manifest_sha256,
        })
    }

    /// Validate whether one declared section set is an exact restart root.
    ///
    /// # Errors
    /// Refuses deltas and every missing, duplicate, or out-of-order section.
    pub fn validate_restart_root(
        completeness: CheckpointCompleteness,
        sections: &[CommittedCheckpointSection],
    ) -> Result<(), RustPersistenceRuntimeError> {
        let mut tags = reserve_vec("checkpoint restart-root tags", sections.len())?;
        tags.extend(sections.iter().map(|section| section.tag.tag()));
        semantic_codec::validate_restart_root(completeness.name(), &tags).map_err(|error| {
            if completeness == CheckpointCompleteness::Delta {
                RustPersistenceRuntimeError::DeltaCheckpointNotRestartRoot
            } else {
                RustPersistenceRuntimeError::from(error)
            }
        })
    }

    /// Return the fixed full completeness marker.
    #[must_use]
    pub const fn completeness(&self) -> CheckpointCompleteness {
        self.completeness
    }

    /// Borrow the canonical full-checkpoint manifest bytes.
    #[must_use]
    pub fn manifest_bytes(&self) -> &[u8] {
        &self.manifest_bytes
    }

    /// Borrow the exact ordered nine section summaries.
    #[must_use]
    pub fn sections(&self) -> &[CommittedCheckpointSection] {
        &self.sections
    }

    /// Borrow the exact nine checkpoint rows in section-tag order.
    #[must_use]
    pub fn rows(&self) -> &[CommittedTickRow] {
        &self.rows
    }

    pub(crate) fn exact_section_bytes(&self) -> &[Vec<u8>] {
        &self.exact_section_bytes
    }

    /// Return SHA-256 of the canonical complete manifest.
    #[must_use]
    pub const fn manifest_sha256(&self) -> [u8; 32] {
        self.manifest_sha256
    }
}

/// Exact checkpoint rows composed from one identified report.
#[derive(Debug, PartialEq, Eq)]
pub struct CheckpointRows {
    source_tick: CommittedResolveTick,
    rows: Vec<CommittedTickRow>,
}

impl CheckpointRows {
    /// Return the report-owned durable tick.
    #[must_use]
    pub const fn source_tick(&self) -> CommittedResolveTick {
        self.source_tick
    }

    /// Return the exact row count.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Borrow exact checkpoint rows in section-tag order.
    #[must_use]
    pub fn rows(&self) -> &[CommittedTickRow] {
        &self.rows
    }

    pub(crate) fn into_rows(self) -> Vec<CommittedTickRow> {
        self.rows
    }
}

/// Exact singular Archive outbox receipt for one identified tick.
#[derive(Debug, PartialEq, Eq)]
pub struct ArchiveDirtyReceipt {
    tick_content_hash: TickContentHash,
    row: CommittedTickRow,
}

impl ArchiveDirtyReceipt {
    /// Return the exact constitutional tick identity carried by the receipt.
    #[must_use]
    pub const fn tick_content_hash(&self) -> TickContentHash {
        self.tick_content_hash
    }

    /// Borrow the exact singular semantic row.
    #[must_use]
    pub const fn row(&self) -> &CommittedTickRow {
        &self.row
    }
}

pub(crate) fn compose_checkpoint_rows(
    report: &IdentifiedTickReport,
    resolve_tick: CommittedResolveTick,
) -> Result<CheckpointRows, RustPersistenceRuntimeError> {
    let source_sections = checkpoint_source_sections(report)?;
    let rows = encode_checkpoint_rows_from_sources(&source_sections)?;
    Ok(CheckpointRows {
        source_tick: resolve_tick,
        rows,
    })
}

fn encode_checkpoint_rows_from_sources(
    source_sections: &[CheckpointSourceSection],
) -> Result<Vec<CommittedTickRow>, RustPersistenceRuntimeError> {
    let mut rows = reserve_vec("checkpoint semantic rows", source_sections.len())?;
    let mut body_bytes = 0_usize;
    for source in source_sections {
        let row = semantic_codec::encode_checkpoint_row(
            source.tag.tag(),
            0,
            CheckpointCompleteness::Full.tag(),
            &source.bytes,
        )?;
        body_bytes = checked_row_body_sum(body_bytes, &row)?;
        rows.push(row);
    }
    Ok(rows)
}

pub(crate) fn compose_archive_dirty_receipt(
    report: &IdentifiedTickReport,
) -> Result<ArchiveDirtyReceipt, RustPersistenceRuntimeError> {
    let tick_content_hash = report.tick_content_hash();
    let row = semantic_codec::encode_archive_dirty_receipt(tick_content_hash.as_bytes())?;
    checked_row_body_sum(0, &row)?;
    Ok(ArchiveDirtyReceipt {
        tick_content_hash,
        row,
    })
}

struct CheckpointSourceSection {
    tag: FullCheckpointSectionTag,
    row_count: u32,
    bytes: Vec<u8>,
}

fn checkpoint_source_sections(
    report: &IdentifiedTickReport,
) -> Result<Vec<CheckpointSourceSection>, RustPersistenceRuntimeError> {
    let stable_graph_count = stable_graph_row_count(report)?;
    let material_count =
        u32::try_from(report.material_state_rows().source_count()).map_err(|_| {
            RustPersistenceRuntimeError::IntegerConversion {
                field: "checkpoint semantic state row count",
                value: report.material_state_rows().source_count(),
            }
        })?;
    let mut content_digest = reserve_bytes("checkpoint content digest", 64)?;
    content_digest.extend_from_slice(&report.content_digest().defines_hash);
    content_digest.extend_from_slice(&report.content_digest().rules_hash);
    let rng_seed = report.rng_seed().to_be_bytes();
    let reference_digest = report.reference_digest();
    let sources = [
        (
            FullCheckpointSectionTag::StableGraph,
            stable_graph_count,
            report.result_stable_graph().canonical_bytes(),
        ),
        (
            FullCheckpointSectionTag::WorldRegisters,
            1,
            report.result_registers().canonical_bytes(),
        ),
        (
            FullCheckpointSectionTag::ResolverManifest,
            1,
            report.resolver_manifest_bytes(),
        ),
        (
            FullCheckpointSectionTag::PreparedEnvironment,
            1,
            report.prepared_environment_bytes(),
        ),
        (
            FullCheckpointSectionTag::ReplaySessionIdentity,
            1,
            report.replay_session_identity().as_bytes(),
        ),
        (FullCheckpointSectionTag::RngSeed, 1, rng_seed.as_slice()),
        (
            FullCheckpointSectionTag::ContentDigest,
            1,
            content_digest.as_slice(),
        ),
        (
            FullCheckpointSectionTag::ReferenceDigest,
            1,
            reference_digest.as_bytes().as_slice(),
        ),
        (
            FullCheckpointSectionTag::SemanticState,
            material_count,
            report.material_state_rows().canonical_bytes(),
        ),
    ];
    debug_assert_eq!(sources.len(), FULL_CHECKPOINT_SECTION_COUNT);
    let mut sections = reserve_vec("checkpoint source sections", sources.len())?;
    for (tag, row_count, bytes) in sources {
        sections.push(CheckpointSourceSection {
            tag,
            row_count,
            bytes: copy_bytes("checkpoint source section bytes", bytes)?,
        });
    }
    Ok(sections)
}

fn summarize_sections(
    sources: &[CheckpointSourceSection],
) -> Result<Vec<CommittedCheckpointSection>, RustPersistenceRuntimeError> {
    let mut sections = reserve_vec("checkpoint section summaries", sources.len())?;
    for source in sources {
        sections.push(CommittedCheckpointSection {
            tag: source.tag,
            row_count: source.row_count,
            sha256: sha256_of(&source.bytes),
        });
    }
    Ok(sections)
}

fn encoded_section_summaries(
    sections: &[CommittedCheckpointSection],
) -> Result<Vec<(u8, u32, [u8; 32])>, RustPersistenceRuntimeError> {
    let mut encoded = reserve_vec("checkpoint encoded section summaries", sections.len())?;
    encoded.extend(
        sections
            .iter()
            .map(|section| (section.tag.tag(), section.row_count, section.sha256)),
    );
    Ok(encoded)
}

fn stable_graph_row_count(
    report: &IdentifiedTickReport,
) -> Result<u32, RustPersistenceRuntimeError> {
    let rows = report.result_stable_graph().rows();
    let count = [
        rows.nodes().len(),
        rows.node_f64().len(),
        rows.edges().len(),
        rows.hyperedges().len(),
        rows.edge_f64().len(),
        rows.node_currency().len(),
        rows.hyperedge_f64().len(),
    ]
    .into_iter()
    .try_fold(0_usize, usize::checked_add)
    .ok_or(RustPersistenceRuntimeError::CapacityOverflow {
        field: "checkpoint stable graph row count",
    })?;
    u32::try_from(count).map_err(|_| RustPersistenceRuntimeError::IntegerConversion {
        field: "checkpoint stable graph row count",
        value: count,
    })
}

fn checked_row_body_sum(
    current: usize,
    row: &CommittedTickRow,
) -> Result<usize, RustPersistenceRuntimeError> {
    current
        .checked_add(ROW_LENGTH_BYTES)
        .and_then(|value| value.checked_add(row.key().len()))
        .and_then(|value| value.checked_add(row.payload().len()))
        .ok_or(RustPersistenceRuntimeError::CapacityOverflow {
            field: "checkpoint row body bytes",
        })
}

fn reserve_vec<T>(
    field: &'static str,
    capacity: usize,
) -> Result<Vec<T>, RustPersistenceRuntimeError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(
            |_: TryReserveError| RustPersistenceRuntimeError::Allocation {
                field,
                requested: capacity,
            },
        )?;
    Ok(values)
}

fn reserve_bytes(
    field: &'static str,
    capacity: usize,
) -> Result<Vec<u8>, RustPersistenceRuntimeError> {
    reserve_vec(field, capacity)
}

fn copy_bytes(field: &'static str, source: &[u8]) -> Result<Vec<u8>, RustPersistenceRuntimeError> {
    let mut bytes = reserve_bytes(field, source.len())?;
    bytes.extend_from_slice(source);
    Ok(bytes)
}
