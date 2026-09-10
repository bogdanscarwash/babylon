//! Production dirty-receipt Archive worker composition.

use postgres::Config;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use uuid::Uuid;

use crate::{
    ArchiveDirtyBatchV1, ArchivePageInputV1, CampaignId, SemanticArchiveErrorV1,
    SemanticArchiveStoreV1,
};

/// Exact pending-receipt page query used by the production Archive worker.
///
/// Claiming happens inside `SemanticArchiveStoreV1::materialize_receipt` under
/// `SERIALIZABLE`; this query deliberately avoids row locking because a
/// single-worker assumption holds and a double-run reconciles as
/// `AlreadyConsumed`.
///
/// Only marker-backed receipts are selected: an inner join to
/// `babylon_state.tick_commit` (not `MAX(tick)`) marks durability, so orphan
/// dirty rows left by a partial rollback never reach a producer and never
/// block later valid receipts. Each invocation returns one keyset page of at
/// most [`ARCHIVE_SWEEP_MAX_RECEIPTS_V1`] unconsumed receipts strictly after
/// the `$3` resolve-tick cursor, in ascending tick order; `sweep_once` pages
/// forward through the bounded pending backlog.
pub const ARCHIVE_PENDING_RECEIPTS_SQL_V1: &str = "SELECT \
    d.resolve_tick, d.tick_content_hash \
    FROM babylon_state.archive_dirty_receipt_v1 d \
    JOIN babylon_state.tick_commit AS marker \
      ON marker.campaign_id = d.campaign_id \
     AND marker.resolve_tick = d.resolve_tick \
    LEFT JOIN babylon_meta.archive_receipt_consumption_v1 c \
      ON c.campaign_id = d.campaign_id \
     AND c.resolve_tick = d.resolve_tick \
    WHERE d.campaign_id = $1::uuid \
      AND c.campaign_id IS NULL \
      AND d.resolve_tick > $3::bigint \
    ORDER BY d.resolve_tick ASC \
    LIMIT $2";

/// Maximum number of receipts one sweep consumes and retains.
///
/// One `--once` invocation claims at most this many ordered receipts; a larger
/// materializable backlog waits for subsequent invocations instead of
/// exhausting memory or the operational timeout.
pub const ARCHIVE_SWEEP_MAX_RECEIPTS_V1: i64 = 256;

/// Maximum number of pending receipts one sweep scans in total.
///
/// This independent scan bound limits how much pending history one invocation
/// observes. Quiet evaluated receipts settle; undrained page sets stay pending.
pub const ARCHIVE_SWEEP_MAX_SCAN_V1: i64 = 4096;

/// Read-only contiguous-watermark query over durable Archive state.
///
/// The first column is the lowest marker-backed unconsumed receipt tick and
/// the second is the highest marker-backed receipt tick (zero when the
/// campaign has no durable receipts). [`archive_contiguous_watermark_v1`]
/// turns that pair into the largest tick whose every receipt is consumed.
pub const ARCHIVE_SWEEP_WATERMARK_SQL_V1: &str = "SELECT \
    (SELECT MIN(d.resolve_tick) \
     FROM babylon_state.archive_dirty_receipt_v1 d \
     JOIN babylon_state.tick_commit AS marker \
       ON marker.campaign_id = d.campaign_id \
      AND marker.resolve_tick = d.resolve_tick \
     LEFT JOIN babylon_meta.archive_receipt_consumption_v1 c \
       ON c.campaign_id = d.campaign_id \
      AND c.resolve_tick = d.resolve_tick \
     WHERE d.campaign_id = $1::uuid \
       AND c.campaign_id IS NULL), \
    COALESCE((SELECT MAX(d.resolve_tick) \
     FROM babylon_state.archive_dirty_receipt_v1 d \
     JOIN babylon_state.tick_commit AS marker \
       ON marker.campaign_id = d.campaign_id \
      AND marker.resolve_tick = d.resolve_tick \
     WHERE d.campaign_id = $1::uuid), 0)";

/// One committed dirty receipt waiting for a content producer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingArchiveReceiptV1 {
    resolve_tick: u64,
    tick_content_hash: [u8; 32],
}

impl PendingArchiveReceiptV1 {
    /// Validate a committed dirty receipt boundary.
    ///
    /// # Errors
    /// Refuses tick zero or a value outside `PostgreSQL` `BIGINT`.
    pub fn try_new(
        resolve_tick: u64,
        tick_content_hash: [u8; 32],
    ) -> Result<Self, SemanticArchiveErrorV1> {
        if resolve_tick == 0 || resolve_tick > i64::MAX as u64 {
            return Err(SemanticArchiveErrorV1::InvalidVerifiedTick);
        }
        Ok(Self {
            resolve_tick,
            tick_content_hash,
        })
    }

    /// Return the honest committed source tick.
    #[must_use]
    pub const fn resolve_tick(&self) -> u64 {
        self.resolve_tick
    }

    /// Return the exact tick content hash bound to the receipt.
    #[must_use]
    pub const fn tick_content_hash(&self) -> &[u8; 32] {
        &self.tick_content_hash
    }
}

/// Pure decision made for one pending receipt before any database work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveReceiptPlanV1 {
    /// The producer proved that no dirty pages remain, including a quiet tick.
    Consume,
    /// Dirty pages remain; stage the bounded head without consuming the receipt.
    Stage,
}

/// Observed outcome for one processed receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveReceiptDispositionV1 {
    /// `materialize_receipt` consumed the receipt now.
    Applied,
    /// `materialize_receipt` observed an exact prior consumption.
    AlreadyConsumed,
    /// `materialize_receipt` staged one bounded page batch; the receipt stays
    /// pending with dirty pages remaining for the next sweep (PER-318).
    Paged,
}

/// One bounded head of a dirty page set plus its exact undrained tail.
///
/// Producers select at most `limit` dirty pages in deterministic subject
/// order and report how many dirty pages remain, so a receipt whose dirty
/// set exceeds the per-sweep bound drains across successive sweeps instead
/// of refusing. Nothing is ever dropped: stored-current pages fall out of
/// the dirty set, so the head advances sweep over sweep until the tail
/// reaches zero.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveDirtySelectionV1<T> {
    head: Vec<T>,
    remaining: usize,
}

impl<T> ArchiveDirtySelectionV1<T> {
    /// Construct one bounded head selection with its undrained tail count.
    #[must_use]
    pub const fn new(head: Vec<T>, remaining: usize) -> Self {
        Self { head, remaining }
    }

    /// Borrow the bounded head in deterministic subject order.
    #[must_use]
    pub fn head(&self) -> &[T] {
        &self.head
    }

    /// Count the dirty pages left undrained for this receipt.
    #[must_use]
    pub const fn remaining(&self) -> usize {
        self.remaining
    }
}

/// One producer outcome: the bounded page batch plus the exact undrained
/// dirty remainder for the same receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveProducerOutcomeV1 {
    batch: ArchiveDirtyBatchV1,
    remaining: usize,
}

impl ArchiveProducerOutcomeV1 {
    /// Bind one bounded batch to its undrained dirty remainder.
    #[must_use]
    pub const fn new(batch: ArchiveDirtyBatchV1, remaining: usize) -> Self {
        Self { batch, remaining }
    }

    /// Borrow the bounded page batch bound to the receipt.
    #[must_use]
    pub const fn batch(&self) -> &ArchiveDirtyBatchV1 {
        &self.batch
    }

    /// Count the dirty pages left undrained after this batch.
    #[must_use]
    pub const fn remaining(&self) -> usize {
        self.remaining
    }
}

/// Content producer that turns one pending receipt into a bounded dirty batch.
pub trait ArchiveDossierProducerV1 {
    /// Produce the exact head batch for one committed dirty receipt.
    ///
    /// `page_budget` is the number of pages this producer may contribute to
    /// the current sweep; the composite threads one shared budget so the
    /// merged batch never exceeds [`ArchiveDirtyBatchV1::MAX_PAGES`]. The
    /// outcome reports the exact undrained dirty remainder: a non-zero
    /// remainder keeps the receipt pending for the next sweep.
    ///
    /// # Errors
    /// Returns any producer-side refusal as a `SemanticArchiveErrorV1`.
    fn produce(
        &self,
        campaign_id: Uuid,
        receipt: &PendingArchiveReceiptV1,
        knowledge: &crate::ArchiveKnowledgeV1,
        page_budget: usize,
    ) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1>;
}

/// Producer for a scope with no pages: every successful receipt settles empty.
pub struct NullArchiveDossierProducerV1;

impl NullArchiveDossierProducerV1 {
    /// Construct the null producer.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for NullArchiveDossierProducerV1 {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchiveDossierProducerV1 for NullArchiveDossierProducerV1 {
    fn produce(
        &self,
        _campaign_id: Uuid,
        receipt: &PendingArchiveReceiptV1,
        _knowledge: &crate::ArchiveKnowledgeV1,
        _page_budget: usize,
    ) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
        let batch = ArchiveDirtyBatchV1::try_new(
            receipt.resolve_tick,
            receipt.tick_content_hash,
            Vec::new(),
        )?;
        Ok(ArchiveProducerOutcomeV1::new(batch, 0))
    }
}

/// Ordered production composition over several dossier producers.
///
/// Every producer sees the same pending receipt and the same shared page
/// budget; the composite queries producers in registration order, shrinks
/// the budget by each funded head, merges the pages into one deterministic
/// batch sorted by page reference, and refuses duplicate subjects across
/// producers. Each producer receives only the budget the earlier producers
/// left, so the merged batch never exceeds
/// [`ArchiveDirtyBatchV1::MAX_PAGES`] and the per-batch bound stays a typed
/// defense behind the budget instead of a refusal. The composite remainder
/// is the exact sum of every producer's undrained tail: the receipt stays
/// pending until the whole merged dirty set drains across successive
/// sweeps, and nothing is ever dropped or truncated.
///
/// The composite registers the county dossier producer first and the place
/// dossier producer second, so a foundation receipt drains every county
/// page before the place head takes the remaining budget.
pub struct CompositeArchiveDossierProducerV1 {
    producers: Vec<Box<dyn ArchiveDossierProducerV1>>,
}

impl CompositeArchiveDossierProducerV1 {
    /// Construct one composite from the exact producer order it will query.
    #[must_use]
    pub fn new(producers: Vec<Box<dyn ArchiveDossierProducerV1>>) -> Self {
        Self { producers }
    }

    /// Borrow the registered producers in query order.
    #[must_use]
    pub fn producers(&self) -> &[Box<dyn ArchiveDossierProducerV1>] {
        &self.producers
    }
}

impl ArchiveDossierProducerV1 for CompositeArchiveDossierProducerV1 {
    fn produce(
        &self,
        campaign_id: Uuid,
        receipt: &PendingArchiveReceiptV1,
        knowledge: &crate::ArchiveKnowledgeV1,
        page_budget: usize,
    ) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
        let mut budget = page_budget;
        let mut remaining = 0usize;
        let mut merged: std::collections::BTreeMap<_, ArchivePageInputV1> =
            std::collections::BTreeMap::new();
        for producer in &self.producers {
            let produced = producer.produce(campaign_id, receipt, knowledge, budget)?;
            archive_batch_matches_receipt_v1(produced.batch(), receipt)?;
            remaining = remaining
                .checked_add(produced.remaining())
                .ok_or(SemanticArchiveErrorV1::CollectionBound)?;
            for page in produced.batch().pages() {
                let key = page.subject().page_ref().clone();
                if merged.insert(key, page.clone()).is_some() {
                    return Err(SemanticArchiveErrorV1::DuplicateKey);
                }
            }
            budget = budget.saturating_sub(produced.batch().pages().len());
        }
        let pages: Vec<ArchivePageInputV1> = merged.into_values().collect();
        let batch =
            ArchiveDirtyBatchV1::try_new(receipt.resolve_tick, receipt.tick_content_hash, pages)?;
        Ok(ArchiveProducerOutcomeV1::new(batch, remaining))
    }
}

/// Per-sweep worker report with ordered dispositions and derived aggregates.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ArchiveWorkerSweepReportV1 {
    dispositions: Vec<(u64, ArchiveReceiptDispositionV1)>,
    durable_tick: u64,
    verified_tick: u64,
    pending_work: bool,
}

impl ArchiveWorkerSweepReportV1 {
    /// Construct one report from ordered per-receipt dispositions and the
    /// campaign's persisted contiguous watermark observed after the sweep.
    #[must_use]
    pub fn new(
        dispositions: Vec<(u64, ArchiveReceiptDispositionV1)>,
        durable_tick: u64,
        verified_tick: u64,
        pending_work: bool,
    ) -> Self {
        Self {
            dispositions,
            durable_tick,
            verified_tick,
            pending_work,
        }
    }

    /// Durable marker tail from the same snapshot as the verified prefix.
    #[must_use]
    pub const fn durable_tick(&self) -> u64 {
        self.durable_tick
    }

    /// Whether another canonical ordered publication remains at that snapshot.
    #[must_use]
    pub const fn has_pending_work(&self) -> bool {
        self.pending_work
    }

    /// Borrow the ordered per-receipt outcomes.
    #[must_use]
    pub fn dispositions(&self) -> &[(u64, ArchiveReceiptDispositionV1)] {
        &self.dispositions
    }

    /// Count receipts consumed by this sweep.
    #[must_use]
    pub fn applied_count(&self) -> usize {
        self.dispositions
            .iter()
            .filter(|(_, disposition)| *disposition == ArchiveReceiptDispositionV1::Applied)
            .count()
    }

    /// Count receipts staged with dirty pages remaining for the next sweep.
    #[must_use]
    pub fn paged_count(&self) -> usize {
        self.dispositions
            .iter()
            .filter(|(_, disposition)| *disposition == ArchiveReceiptDispositionV1::Paged)
            .count()
    }

    /// Count receipts observed as exactly consumed by a prior run.
    #[must_use]
    pub fn already_consumed_count(&self) -> usize {
        self.dispositions
            .iter()
            .filter(|(_, disposition)| *disposition == ArchiveReceiptDispositionV1::AlreadyConsumed)
            .count()
    }

    /// The campaign's contiguous persisted watermark observed after the sweep.
    ///
    /// This is the largest tick whose every marker-backed dirty receipt is
    /// consumed in durable state, never the sweep-local maximum: an undrained
    /// earlier tick caps it, and an empty sweep still reports the persisted
    /// watermark instead of zero.
    #[must_use]
    pub const fn verified_tick(&self) -> u64 {
        self.verified_tick
    }
}

/// Pure per-receipt decision helper over one producer outcome.
///
/// A successful producer proves the exact dirty remainder. Zero remaining
/// settles the receipt even when no content changed. A nonzero remainder
/// always stages, including an empty head after its page budget ran out.
#[must_use]
pub fn classify_archive_receipt_v1(outcome: &ArchiveProducerOutcomeV1) -> ArchiveReceiptPlanV1 {
    if outcome.remaining() == 0 {
        ArchiveReceiptPlanV1::Consume
    } else {
        ArchiveReceiptPlanV1::Stage
    }
}

/// Pure batch-identity refusal: a producer's batch must be bound to the exact
/// pending receipt the worker asked about.
///
/// # Errors
/// Returns `SemanticArchiveErrorV1::ReceiptMismatch` when the batch targets a
/// different resolve tick or tick content hash than the receipt.
pub fn archive_batch_matches_receipt_v1(
    batch: &ArchiveDirtyBatchV1,
    receipt: &PendingArchiveReceiptV1,
) -> Result<(), SemanticArchiveErrorV1> {
    if batch.resolve_tick() != receipt.resolve_tick()
        || batch.tick_content_hash() != receipt.tick_content_hash()
    {
        return Err(SemanticArchiveErrorV1::ReceiptMismatch);
    }
    Ok(())
}

/// Pure contiguous-watermark derivation from durable state observations.
///
/// `first_pending_tick` is the lowest marker-backed unconsumed receipt tick
/// (`None` when nothing is pending) and `max_receipt_tick` is the highest
/// marker-backed receipt tick. The result is the largest tick whose every
/// receipt is consumed: a pending tick caps the watermark at its predecessor,
/// while a fully consumed backlog reports its highest receipt. An empty
/// campaign reports zero.
#[must_use]
pub const fn archive_contiguous_watermark_v1(
    first_pending_tick: Option<u64>,
    max_receipt_tick: u64,
) -> u64 {
    match first_pending_tick {
        Some(first_pending) => first_pending - 1,
        None => max_receipt_tick,
    }
}

/// Pure sweep planner over a scripted producer-outcome sequence.
///
/// Returns `Err` at the first producer failure and never skips past it.
///
/// # Errors
/// Propagates the first producer error unchanged.
pub fn classify_archive_sweep_v1(
    outcomes: Vec<Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1>>,
) -> Result<Vec<ArchiveReceiptPlanV1>, SemanticArchiveErrorV1> {
    outcomes
        .into_iter()
        .map(|outcome| Ok(classify_archive_receipt_v1(&outcome?)))
        .collect()
}

/// Pure paged-sweep outcome: the ordered per-receipt plans plus the scan and
/// consume counts the production sweep reaches under the same bounds.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ArchiveSweepPageModelV1 {
    plans: Vec<ArchiveReceiptPlanV1>,
    scanned: i64,
    consumed: i64,
}

impl ArchiveSweepPageModelV1 {
    /// Construct one model outcome from ordered plans and derived counts.
    #[must_use]
    pub fn new(plans: Vec<ArchiveReceiptPlanV1>, scanned: i64, consumed: i64) -> Self {
        Self {
            plans,
            scanned,
            consumed,
        }
    }

    /// Borrow the ordered per-receipt plans across every scanned page.
    #[must_use]
    pub fn plans(&self) -> &[ArchiveReceiptPlanV1] {
        &self.plans
    }

    /// Count receipts the sweep scanned, including staged ones.
    #[must_use]
    pub const fn scanned(&self) -> i64 {
        self.scanned
    }

    /// Count receipts the sweep materialized (consumed or paged), capped by
    /// the consume bound.
    #[must_use]
    pub const fn consumed(&self) -> i64 {
        self.consumed
    }
}

/// Pure paged-sweep model over scripted producer outcome pages under the
/// production bounds ([`ARCHIVE_SWEEP_MAX_RECEIPTS_V1`] and
/// [`ARCHIVE_SWEEP_MAX_SCAN_V1`]).
///
/// The model mirrors [`ArchiveWorkerV1::sweep_once`]: pages arrive in keyset
/// order, each scanned receipt consumes or stages exactly as classified,
/// and the sweep stops as soon as the consume cap or the scan cap is
/// reached, leaving the remainder pending for the next invocation.
///
/// # Errors
/// Propagates the first producer error unchanged.
pub fn model_archive_sweep_pages_v1(
    pages: Vec<Vec<Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1>>>,
) -> Result<ArchiveSweepPageModelV1, SemanticArchiveErrorV1> {
    model_archive_sweep_pages_with_bounds_v1(
        pages,
        ARCHIVE_SWEEP_MAX_RECEIPTS_V1,
        ARCHIVE_SWEEP_MAX_SCAN_V1,
    )
}

/// Pure paged-sweep model with explicit bounds for contract regression tests.
///
/// # Errors
/// Propagates the first producer error unchanged.
pub fn model_archive_sweep_pages_with_bounds_v1(
    pages: Vec<Vec<Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1>>>,
    max_receipts: i64,
    max_scan: i64,
) -> Result<ArchiveSweepPageModelV1, SemanticArchiveErrorV1> {
    let mut model = ArchiveSweepPageModelV1::default();
    'pages: for page in pages {
        if model.consumed >= max_receipts || model.scanned >= max_scan {
            break;
        }
        for step in page {
            if model.scanned >= max_scan || model.consumed >= max_receipts {
                break 'pages;
            }
            model.scanned += 1;
            let outcome = step?;
            let plan = classify_archive_receipt_v1(&outcome);
            model.consumed += 1;
            model.plans.push(plan);
            if plan == ArchiveReceiptPlanV1::Stage {
                break 'pages;
            }
        }
    }
    Ok(model)
}

/// Shared cooperative stop token. It never cancels an acknowledged game tick.
#[derive(Clone, Debug, Default)]
pub struct ArchiveWorkerCancellationV1(Arc<AtomicBool>);

impl ArchiveWorkerCancellationV1 {
    /// Stop before the next publication; any uncommitted work rolls back.
    pub fn request_stop(&self) {
        self.0.store(true, Ordering::Release);
    }

    /// Whether the owner has requested stop.
    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    pub(crate) fn check(&self) -> Result<(), SemanticArchiveErrorV1> {
        if self.is_stopped() {
            Err(SemanticArchiveErrorV1::WorkerCanceled)
        } else {
            Ok(())
        }
    }
}

/// Production Archive worker that composes a content producer with the
/// semantic Archive store.
pub struct ArchiveWorkerV1 {
    store: SemanticArchiveStoreV1,
}

impl ArchiveWorkerV1 {
    /// Bind the worker to one Rust-authoritative `PostgreSQL` target.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        Self {
            store: SemanticArchiveStoreV1::new(config),
        }
    }

    /// Run one ordered sweep over the pending dirty receipts.
    ///
    /// The sweep pages through the marker-backed pending set by keyset cursor
    /// ([`ARCHIVE_PENDING_RECEIPTS_SQL_V1`]). Every successful producer result
    /// either settles or stages its receipt. It stops as soon as
    /// it has claimed [`ARCHIVE_SWEEP_MAX_RECEIPTS_V1`] receipts, scanned
    /// [`ARCHIVE_SWEEP_MAX_SCAN_V1`] receipts in total, or exhausted the
    /// pending set. Each claimed receipt delegates to
    /// [`SemanticArchiveStoreV1::materialize_receipt`], which binds the worker
    /// identity via [`crate::archive_worker_contract_sha256_v1`]: a receipt
    /// whose producer reports an undrained remainder is staged in
    /// [`crate::ArchiveMaterializeModeV1::Stage`] mode — its pages write, its
    /// consumption row stays absent, and the disposition reports
    /// [`ArchiveReceiptDispositionV1::Paged`] — so the receipt stays pending
    /// and `verified_tick` honestly stalls behind the draining backlog. The
    /// pure [`model_archive_sweep_pages_v1`] mirrors this loop for contract
    /// regression tests.
    ///
    /// # Errors
    /// Returns any producer refusal, batch-identity mismatch, or database
    /// failure immediately, leaving the sweep incomplete.
    pub fn sweep_once(
        &mut self,
        campaign_id: CampaignId,
        producer: &dyn ArchiveDossierProducerV1,
    ) -> Result<ArchiveWorkerSweepReportV1, SemanticArchiveErrorV1> {
        self.sweep_cancellable(
            campaign_id,
            producer,
            &ArchiveWorkerCancellationV1::default(),
        )
    }

    /// Run the same canonical sweep with cooperative publication-boundary stop.
    ///
    /// # Errors
    /// Preserves producer/database refusals; returns `WorkerCanceled` on stop.
    pub fn sweep_cancellable(
        &mut self,
        campaign_id: CampaignId,
        producer: &dyn ArchiveDossierProducerV1,
        cancellation: &ArchiveWorkerCancellationV1,
    ) -> Result<ArchiveWorkerSweepReportV1, SemanticArchiveErrorV1> {
        crate::archive_revision::worker::sweep(&self.store, campaign_id, producer, cancellation)
    }
}
