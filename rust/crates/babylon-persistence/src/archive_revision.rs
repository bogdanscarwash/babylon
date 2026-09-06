//! Immutable Archive publications and exact scoped dossier observations.

mod changes;
pub(crate) mod emission;
mod enrollment;
mod knowledge;
pub(crate) mod publication;
mod read;
mod read_history;
mod record;
mod recovery;
pub(crate) mod schema;
mod storage;
mod tick_knowledge;
pub(crate) mod worker;

use crate::{
    ArchiveAtomV1, ArchiveCitationV1, ArchivePageRefV1, ArchiveSignalV1, CampaignId,
    SemanticArchiveErrorV1,
};

/// One exact acknowledged Archive observation. Fields cannot bypass validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveReadScopeV2 {
    campaign_id: CampaignId,
    tick: u64,
    tick_content_hash: Option<[u8; 32]>,
}

impl ArchiveReadScopeV2 {
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
    ) -> Result<Self, SemanticArchiveErrorV1> {
        if tick == 0 || tick > i64::MAX as u64 {
            return Err(SemanticArchiveErrorV1::InvalidVerifiedTick);
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
pub enum ArchiveDossierPendingV2 {
    /// Original bytes survive privately, but their complete emitted structure is unproved.
    EmissionWitnessRequired,
    /// The adopted head awaits validation against its committed cutover tick.
    CutoverValidation,
    /// An earlier committed receipt has not completed its bounded page drain.
    ReceiptProcessing,
    /// A grant arrived after this tick's immutable knowledge snapshot was pinned.
    KnowledgeRefresh,
}

/// Honest absence, distinct from corrupt data, wrong scope, or database failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveDossierUnavailableV2 {
    /// Foundation is not a rendered committed Archive page.
    FoundationHasNoPage,
    /// The requested tick precedes retained coverage; old prose was overwritten.
    HistoryNotRetained,
    /// No subject grant covers this observation.
    SubjectNotDisclosed,
    /// Subject identity is disclosed, but no page has been retained for this scope.
    PageNotMaterialized,
}

/// Closed publication origin; adoption never impersonates the live renderer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchivePublicationOriginV2 {
    /// Exact existing page retained at the upgrade's durable tail.
    AdoptedHead,
    /// Exact output published by the revision-aware materializer.
    Materialized,
}

impl ArchivePublicationOriginV2 {
    pub(crate) const fn tag(self) -> i16 {
        match self {
            Self::AdoptedHead => 0,
            Self::Materialized => 1,
        }
    }

    pub(crate) fn from_tag(tag: i16) -> Result<Self, SemanticArchiveErrorV1> {
        match tag {
            0 => Ok(Self::AdoptedHead),
            1 => Ok(Self::Materialized),
            _ => Err(SemanticArchiveErrorV1::StoredPageMismatch),
        }
    }
}

/// Availability of the exact link target, independent from the retained label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveLinkedPageStateV2 {
    /// Only the public structural target identity may be shown.
    Unknown,
    /// A known target has no retained page at the requested scope.
    KnownUnavailable,
    /// A retained target awaits Archive processing or cutover validation.
    KnownPending,
    /// The target has a verified page at the requested scope.
    KnownReady,
}

/// One retained ordered link, with no title borrowed from a later page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveDossierLinkV2 {
    /// Already-public target structure.
    pub target: ArchivePageRefV1,
    /// Original label; an empty-text link remains absent even after a later grant.
    pub retained_label: Option<String>,
    /// Exact scoped target availability.
    pub target_state: ArchiveLinkedPageStateV2,
}

/// A change in retained asserted atoms, including removal without synthetic zero.
#[derive(Clone, Debug, PartialEq)]
pub struct ArchiveAtomChangeV2 {
    /// Effective tick of the later publication.
    pub publication_tick: u64,
    /// Exact typed signal identity.
    pub signal_key: String,
    /// Earlier retained value, when known within coverage.
    pub before: Option<ArchiveAtomV1>,
    /// Later retained value; absent for a removal.
    pub after: Option<ArchiveAtomV1>,
}

/// Opaque deterministic continuation bound to one scope and history identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveChangeCursorV2 {
    pub(crate) scope: ArchiveReadScopeV2,
    pub(crate) subject: ArchivePageRefV1,
    pub(crate) history_digest: [u8; 32],
    pub(crate) publication_tick: u64,
    pub(crate) publication_origin: i16,
    pub(crate) change_offset: u32,
}

/// Explicit bounded changelog query; page, atom, and link bounds remain independent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveDossierBoundsV2 {
    pub(crate) change_limit: u32,
    pub(crate) change_cursor: Option<ArchiveChangeCursorV2>,
}

impl Default for ArchiveDossierBoundsV2 {
    fn default() -> Self {
        Self {
            change_limit: 32,
            change_cursor: None,
        }
    }
}

impl ArchiveDossierBoundsV2 {
    /// Admit one bounded history page and an optional continuation.
    ///
    /// # Errors
    /// Refuses a zero or over-100 result bound.
    pub fn try_new(
        change_limit: u32,
        change_cursor: Option<ArchiveChangeCursorV2>,
    ) -> Result<Self, SemanticArchiveErrorV1> {
        if !(1..=100).contains(&change_limit) {
            return Err(SemanticArchiveErrorV1::CollectionBound);
        }
        Ok(Self {
            change_limit,
            change_cursor,
        })
    }
}

/// One bounded page of actual retained composition changes.
#[derive(Clone, Debug, PartialEq)]
pub struct ArchiveChangePageV2 {
    /// Earlier ticks cannot be inferred from an adopted baseline.
    pub coverage_from_tick: u64,
    /// Ordered exact atom changes.
    pub changes: Vec<ArchiveAtomChangeV2>,
    /// Explicit continuation; truncation never implies absence.
    pub next_cursor: Option<ArchiveChangeCursorV2>,
}

/// Complete immutable page observation, always bound to its original content source.
#[derive(Clone, Debug, PartialEq)]
pub struct ArchiveDossierPageV2 {
    /// Complete publication identity, including exact ordered membership.
    pub revision_id: [u8; 32],
    /// Tick where this publication enters retained coverage.
    pub effective_tick: u64,
    /// Adoption or live publication; neither changes substantive evidence classes.
    pub origin: ArchivePublicationOriginV2,
    /// Original committed content tick and hash, preserved through quiet validation.
    pub content_source: ArchiveReadScopeV2,
    /// Exact retained title.
    pub title: String,
    /// Original decision question from the validated emission witness.
    pub question: String,
    /// Original ordered disclosed labels, values and citations.
    pub signals: Vec<ArchiveSignalV1>,
    /// Exact retained narrative, question, signals, and known/unknown links.
    pub markdown: String,
    /// Unchanged Markdown-only V1 digest.
    pub content_sha256: [u8; 32],
    /// Exact original citations.
    pub citations: Vec<ArchiveCitationV1>,
    /// Exact retained ordered membership; never all atoms minted for the subject.
    pub atoms: Vec<ArchiveAtomV1>,
    /// Exact retained links, in original profile order.
    pub links: Vec<ArchiveDossierLinkV2>,
    /// Bounded retained changes with explicit coverage.
    pub changes: ArchiveChangePageV2,
}

/// A scoped dossier is either verified, retained but pending, or honestly absent.
#[derive(Clone, Debug, PartialEq)]
pub enum ArchiveDossierStateV2 {
    /// Both retained coverage and contiguous processing cover the requested tick.
    Ready {
        /// Complete scoped page.
        page: ArchiveDossierPageV2,
        /// Exactly the requested tick, separate from the content source.
        verified_through_tick: u64,
    },
    /// Preserve a readable adopted/staged page without claiming verification.
    Pending {
        /// Eligible retained content, when available.
        page: Option<ArchiveDossierPageV2>,
        /// Required remaining Archive work.
        reason: ArchiveDossierPendingV2,
    },
    /// No eligible retained page can answer the requested observation.
    Unavailable(ArchiveDossierUnavailableV2),
}

/// One role-confined MVCC observation; progress alone never certifies its state.
#[derive(Clone, Debug, PartialEq)]
pub struct ArchiveDossierReadV2 {
    /// Exact requested campaign and commit identity.
    pub scope: ArchiveReadScopeV2,
    /// Exact requested subject.
    pub subject: ArchivePageRefV1,
    /// Global marker-backed tail observed in the same read transaction.
    pub durable_tick: u64,
    /// Global contiguous receipt progress; distinct from selected-page verification.
    pub processed_tick: u64,
    /// Conservative retained history floor.
    pub history_floor_tick: u64,
    /// The only authority for selected dossier freshness.
    pub state: ArchiveDossierStateV2,
}

/// One retained, scoped search match; opening it is a fresh exact-subject read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveSearchHitV2 {
    pub subject: ArchivePageRefV1,
    pub revision_id: [u8; 32],
    pub title: String,
    pub content_source: ArchiveReadScopeV2,
}

/// Search completeness is separate from whether any matching bytes were retained.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveSearchStateV2 {
    Ready,
    Pending(ArchiveDossierPendingV2),
    Unavailable(ArchiveDossierUnavailableV2),
}

/// Bounded search over the exact retained composition at one committed scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveSearchReadV2 {
    pub scope: ArchiveReadScopeV2,
    pub durable_tick: u64,
    pub processed_tick: u64,
    pub history_floor_tick: u64,
    pub state: ArchiveSearchStateV2,
    pub hits: Vec<ArchiveSearchHitV2>,
    /// More matching retained pages exist than the explicit result bound.
    pub truncated: bool,
}
