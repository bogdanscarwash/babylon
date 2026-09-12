//! Immutable Archive publications and exact scoped dossier observations.

mod changes;
pub(crate) mod emission;
mod knowledge;
pub(crate) mod publication;
mod read;
mod read_history;
mod record;
mod storage;
mod tick_knowledge;
pub(crate) mod worker;

use crate::{
    identity::CampaignId, ArchiveAtom, ArchiveCitation, ArchivePageRef, ArchiveSignal,
    SemanticArchiveError,
};

/// One exact acknowledged Archive observation. Fields cannot bypass validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveReadScope {
    campaign_id: CampaignId,
    tick: u64,
    tick_content_hash: Option<[u8; 32]>,
}

impl ArchiveReadScope {
    /// Foundation has no fabricated commit hash and cannot contain a rendered page.
    #[must_use]
    pub const fn foundation(campaign_id: CampaignId) -> Self {
        Self {
            campaign_id,
            tick: 0,
            tick_content_hash: None,
        }
    }

    /// Bind one positive, representable committed tick and its expected identity.
    ///
    /// # Errors
    /// Refuses zero or ticks beyond the persistence integer domain.
    pub fn committed(
        campaign_id: CampaignId,
        tick: u64,
        hash: [u8; 32],
    ) -> Result<Self, SemanticArchiveError> {
        if tick == 0 || tick > i64::MAX as u64 {
            return Err(SemanticArchiveError::InvalidVerifiedTick);
        }
        Ok(Self {
            campaign_id,
            tick,
            tick_content_hash: Some(hash),
        })
    }

    /// Campaign whose marker and grants govern the whole observation.
    #[must_use]
    pub const fn campaign_id(&self) -> CampaignId {
        self.campaign_id
    }

    /// Requested committed tick, or zero for foundation.
    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    /// Exact expected marker identity; absent only at foundation.
    #[must_use]
    pub const fn tick_content_hash(&self) -> Option<[u8; 32]> {
        self.tick_content_hash
    }
}

/// Why a retained page cannot yet certify the requested observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveDossierPending {
    /// An earlier committed receipt has not completed its bounded page drain.
    ReceiptProcessing,
    /// A grant arrived after this tick's immutable knowledge snapshot was pinned.
    KnowledgeRefresh,
}

/// Honest absence, distinct from corrupt data, wrong scope, or database failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveDossierUnavailable {
    /// Foundation is not a rendered committed Archive page.
    FoundationHasNoPage,
    /// No subject grant covers this observation.
    SubjectNotDisclosed,
    /// Subject identity is disclosed, but no page has been retained for this scope.
    PageNotMaterialized,
}

/// Availability of the exact link target, independent from the retained label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveLinkedPageState {
    /// Only the public structural target identity may be shown.
    Unknown,
    /// A known target has no retained page at the requested scope.
    KnownUnavailable,
    /// A retained target awaits Archive processing.
    KnownPending,
    /// The target has a verified page at the requested scope.
    KnownReady,
}

/// One retained ordered link, with no title borrowed from a later page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveDossierLink {
    /// Already-public target structure.
    pub target: ArchivePageRef,
    /// Original label; an empty-text link remains absent even after a later grant.
    pub retained_label: Option<String>,
    /// Exact scoped target availability.
    pub target_state: ArchiveLinkedPageState,
}

/// A change in retained asserted atoms, including removal without synthetic zero.
#[derive(Clone, Debug, PartialEq)]
pub struct ArchiveAtomChange {
    /// Effective tick of the later publication.
    pub publication_tick: u64,
    /// Exact typed signal identity.
    pub signal_key: String,
    /// Earlier retained value, when known within coverage.
    pub before: Option<ArchiveAtom>,
    /// Later retained value; absent for a removal.
    pub after: Option<ArchiveAtom>,
}

/// Opaque deterministic continuation bound to one scope and history identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveChangeCursor {
    pub(crate) scope: ArchiveReadScope,
    pub(crate) subject: ArchivePageRef,
    pub(crate) history_digest: [u8; 32],
    pub(crate) publication_tick: u64,
    pub(crate) change_offset: u32,
}

/// Explicit bounded changelog query; page, atom, and link bounds remain independent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveDossierBounds {
    pub(crate) change_limit: u32,
    pub(crate) change_cursor: Option<ArchiveChangeCursor>,
}

impl Default for ArchiveDossierBounds {
    fn default() -> Self {
        Self {
            change_limit: 32,
            change_cursor: None,
        }
    }
}

impl ArchiveDossierBounds {
    /// Admit one bounded history page and an optional continuation.
    ///
    /// # Errors
    /// Refuses a zero or over-100 result bound.
    pub fn try_new(
        change_limit: u32,
        change_cursor: Option<ArchiveChangeCursor>,
    ) -> Result<Self, SemanticArchiveError> {
        if !(1..=100).contains(&change_limit) {
            return Err(SemanticArchiveError::CollectionBound);
        }
        Ok(Self {
            change_limit,
            change_cursor,
        })
    }
}

/// One bounded page of actual retained composition changes.
#[derive(Clone, Debug, PartialEq)]
pub struct ArchiveChangePage {
    /// Current campaigns retain their complete publication history from foundation.
    pub coverage_from_tick: u64,
    /// Ordered exact atom changes.
    pub changes: Vec<ArchiveAtomChange>,
    /// Explicit continuation; truncation never implies absence.
    pub next_cursor: Option<ArchiveChangeCursor>,
}

/// Complete immutable page observation, always bound to its original content source.
#[derive(Clone, Debug, PartialEq)]
pub struct ArchiveDossierPage {
    /// Complete publication identity, including exact ordered membership.
    pub revision_id: [u8; 32],
    /// Tick where this publication enters retained coverage.
    pub effective_tick: u64,
    /// Original committed content tick and hash, preserved through quiet validation.
    pub content_source: ArchiveReadScope,
    /// Exact retained title.
    pub title: String,
    /// Original decision question from the validated emission witness.
    pub question: String,
    /// Original ordered disclosed labels, values and citations.
    pub signals: Vec<ArchiveSignal>,
    /// Exact retained narrative, question, signals, and known/unknown links.
    pub markdown: String,
    /// Unchanged Markdown-only V1 digest.
    pub content_sha256: [u8; 32],
    /// Exact original citations.
    pub citations: Vec<ArchiveCitation>,
    /// Exact retained ordered membership; never all atoms minted for the subject.
    pub atoms: Vec<ArchiveAtom>,
    /// Exact retained links, in original profile order.
    pub links: Vec<ArchiveDossierLink>,
    /// Bounded retained changes with explicit coverage.
    pub changes: ArchiveChangePage,
}

/// A scoped dossier is either verified, retained but pending, or honestly absent.
#[derive(Clone, Debug, PartialEq)]
pub enum ArchiveDossierState {
    /// Both retained coverage and contiguous processing cover the requested tick.
    Ready {
        /// Complete scoped page.
        page: ArchiveDossierPage,
        /// Exactly the requested tick, separate from the content source.
        verified_through_tick: u64,
    },
    /// Preserve a readable staged page without claiming verification.
    Pending {
        /// Eligible retained content, when available.
        page: Option<ArchiveDossierPage>,
        /// Required remaining Archive work.
        reason: ArchiveDossierPending,
    },
    /// No eligible retained page can answer the requested observation.
    Unavailable(ArchiveDossierUnavailable),
}

/// One role-confined MVCC observation; progress alone never certifies its state.
#[derive(Clone, Debug, PartialEq)]
pub struct ArchiveDossierRead {
    /// Exact requested campaign and commit identity.
    pub scope: ArchiveReadScope,
    /// Exact requested subject.
    pub subject: ArchivePageRef,
    /// Global marker-backed tail observed in the same read transaction.
    pub durable_tick: u64,
    /// Global contiguous receipt progress; distinct from selected-page verification.
    pub processed_tick: u64,
    /// The only authority for selected dossier freshness.
    pub state: ArchiveDossierState,
}

/// One retained, scoped search match; opening it is a fresh exact-subject read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveSearchHit {
    pub subject: ArchivePageRef,
    pub revision_id: [u8; 32],
    pub title: String,
    pub content_source: ArchiveReadScope,
}

/// Search completeness is separate from whether any matching bytes were retained.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveSearchState {
    Ready,
    Pending(ArchiveDossierPending),
    Unavailable(ArchiveDossierUnavailable),
}

/// Bounded search over the exact retained composition at one committed scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveSearchRead {
    pub scope: ArchiveReadScope,
    pub durable_tick: u64,
    pub processed_tick: u64,
    pub state: ArchiveSearchState,
    pub hits: Vec<ArchiveSearchHit>,
    /// More matching retained pages exist than the explicit result bound.
    pub truncated: bool,
}
