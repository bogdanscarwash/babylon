use std::str::FromStr;

use babylon_persistence::{
    archive_batch_matches_receipt_v1, archive_contiguous_watermark_v1, classify_archive_receipt_v1,
    classify_archive_sweep_v1, model_archive_sweep_pages_v1,
    model_archive_sweep_pages_with_bounds_v1, ArchiveCitationV1, ArchiveDirtyBatchV1,
    ArchiveDossierProducerV1, ArchiveLinkV1, ArchivePageInputV1, ArchivePageRefV1,
    ArchiveProducerOutcomeV1, ArchiveReceiptDispositionV1, ArchiveReceiptPlanV1, ArchiveSignalV1,
    ArchiveSubjectKindV1, ArchiveSubjectV1, ArchiveWorkerSweepReportV1,
    CompositeArchiveDossierProducerV1, NullArchiveDossierProducerV1, PendingArchiveReceiptV1,
    SemanticArchiveErrorV1, SemanticArchiveStoreV1, ARCHIVE_PENDING_RECEIPTS_SQL_V1,
    ARCHIVE_SWEEP_MAX_RECEIPTS_V1, ARCHIVE_SWEEP_MAX_SCAN_V1, ARCHIVE_SWEEP_WATERMARK_SQL_V1,
};
use postgres::{Config, NoTls};
use uuid::Uuid;

const LIVE_DSN_ENV: &str = "BABYLON_LEGACY_ADOPTER_TEST_DSN";
const LIVE_CANARY_ENV: &str = "BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY";

fn county_subject() -> ArchiveSubjectV1 {
    ArchiveSubjectV1::try_new(
        ArchiveSubjectKindV1::County,
        "26163".to_owned(),
        "Wayne County".to_owned(),
    )
    .expect("county identity")
}

fn county_page_input(resolve_tick: u64, tick_content_hash: [u8; 32]) -> ArchivePageInputV1 {
    ArchivePageInputV1::try_new(
        county_subject(),
        resolve_tick,
        tick_content_hash,
        "Which neighboring place should organizers investigate next?".to_owned(),
        vec![ArchiveSignalV1::try_new(
            "employment".to_owned(),
            "Employment".to_owned(),
            "728576 jobs".to_owned(),
            ArchiveCitationV1::try_new(
                "qcew-2024".to_owned(),
                "fact_qcew_county_rollup county_fips=26163".to_owned(),
            )
            .expect("citation"),
        )
        .expect("signal")],
        vec![ArchiveLinkV1::try_new(
            ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, "2622000".to_owned())
                .expect("Detroit ref"),
            "Detroit".to_owned(),
        )
        .expect("Detroit link")],
    )
    .expect("page input")
}

fn empty_batch(resolve_tick: u64, tick_content_hash: [u8; 32]) -> ArchiveDirtyBatchV1 {
    ArchiveDirtyBatchV1::try_new(resolve_tick, tick_content_hash, Vec::new()).expect("empty batch")
}

fn non_empty_batch(resolve_tick: u64, tick_content_hash: [u8; 32]) -> ArchiveDirtyBatchV1 {
    ArchiveDirtyBatchV1::try_new(
        resolve_tick,
        tick_content_hash,
        vec![county_page_input(resolve_tick, tick_content_hash)],
    )
    .expect("non-empty batch")
}

fn outcome(batch: ArchiveDirtyBatchV1, remaining: usize) -> ArchiveProducerOutcomeV1 {
    ArchiveProducerOutcomeV1::new(batch, remaining)
}

fn full_outcome(resolve_tick: u64, tick_content_hash: [u8; 32]) -> ArchiveProducerOutcomeV1 {
    outcome(non_empty_batch(resolve_tick, tick_content_hash), 0)
}

#[test]
fn pending_receipts_sql_finds_unconsumed_receipts_in_keyset_order_without_locking() {
    assert!(ARCHIVE_PENDING_RECEIPTS_SQL_V1
        .contains("LEFT JOIN babylon_meta.archive_receipt_consumption_v1"));
    assert!(ARCHIVE_PENDING_RECEIPTS_SQL_V1.contains("JOIN babylon_state.tick_commit AS marker"));
    assert!(ARCHIVE_PENDING_RECEIPTS_SQL_V1.contains("marker.campaign_id = d.campaign_id"));
    assert!(ARCHIVE_PENDING_RECEIPTS_SQL_V1.contains("marker.resolve_tick = d.resolve_tick"));
    assert!(ARCHIVE_PENDING_RECEIPTS_SQL_V1.contains("IS NULL"));
    assert!(ARCHIVE_PENDING_RECEIPTS_SQL_V1.contains("d.campaign_id = $1"));
    assert!(
        ARCHIVE_PENDING_RECEIPTS_SQL_V1.contains("d.resolve_tick > $3"),
        "the sweep pages through pending receipts by keyset cursor, never OFFSET"
    );
    assert!(!ARCHIVE_PENDING_RECEIPTS_SQL_V1.contains("OFFSET"));
    assert!(ARCHIVE_PENDING_RECEIPTS_SQL_V1.contains("ORDER BY d.resolve_tick ASC"));
    assert!(
        ARCHIVE_PENDING_RECEIPTS_SQL_V1.contains("LIMIT $2"),
        "each pending page is a bounded chunk, not unbounded history"
    );
    assert!(!ARCHIVE_PENDING_RECEIPTS_SQL_V1.contains("FOR UPDATE"));
    assert!(!ARCHIVE_PENDING_RECEIPTS_SQL_V1.contains("SKIP LOCKED"));
    assert!(!ARCHIVE_PENDING_RECEIPTS_SQL_V1.contains("NOWAIT"));
}

#[test]
fn pending_receipts_sweep_chunk_and_scan_bound_are_positive_bounded_constants() {
    assert_eq!(ARCHIVE_SWEEP_MAX_RECEIPTS_V1, 256);
    assert_eq!(ARCHIVE_SWEEP_MAX_SCAN_V1, 4096);
}

#[test]
fn watermark_sql_derives_the_contiguous_consumed_prefix_from_durable_state() {
    assert!(ARCHIVE_SWEEP_WATERMARK_SQL_V1.contains("JOIN babylon_state.tick_commit AS marker"));
    assert!(ARCHIVE_SWEEP_WATERMARK_SQL_V1.contains("MIN(d.resolve_tick)"));
    assert!(ARCHIVE_SWEEP_WATERMARK_SQL_V1.contains("MAX(d.resolve_tick)"));
    assert!(ARCHIVE_SWEEP_WATERMARK_SQL_V1.contains("c.campaign_id IS NULL"));
    assert!(ARCHIVE_SWEEP_WATERMARK_SQL_V1.contains("d.campaign_id = $1::uuid"));
}

#[test]
fn worker_identity_is_the_store_contract_not_a_local_claim() {
    let source = std::include_str!("../src/archive_worker.rs");
    assert!(
        !source.contains("INSERT INTO babylon_meta.archive_receipt_consumption_v1"),
        "the worker must not re-implement the store's claim-by-insert"
    );
    assert!(
        source.contains("materialize_receipt"),
        "the worker must delegate consumption to the store"
    );
    assert!(
        source.contains("archive_worker_contract_sha256_v1"),
        "the worker must bind to the store's worker identity"
    );
}

#[test]
fn null_producer_returns_empty_but_valid_outcome() {
    let producer = NullArchiveDossierProducerV1::new();
    let receipt = PendingArchiveReceiptV1::try_new(1, [0x11; 32]).expect("valid receipt");
    let outcome = producer
        .produce(
            Uuid::nil(),
            &receipt,
            &babylon_persistence::ArchiveKnowledgeV1::try_new(Vec::new())
                .expect("empty scripted knowledge"),
            ArchiveDirtyBatchV1::MAX_PAGES,
        )
        .expect("null outcome is valid");
    let batch = outcome.batch();

    assert!(batch.pages().is_empty());
    assert_eq!(outcome.remaining(), 0, "nothing dirty remains");
    assert_eq!(empty_batch(1, [0x11; 32]).sha256(), batch.sha256());
}

#[test]
fn pending_receipt_refuses_tick_zero_and_bigint_overflow() {
    assert_eq!(
        PendingArchiveReceiptV1::try_new(0, [0x11; 32]),
        Err(SemanticArchiveErrorV1::InvalidVerifiedTick)
    );
    assert_eq!(
        PendingArchiveReceiptV1::try_new(i64::MAX as u64 + 1, [0x11; 32]),
        Err(SemanticArchiveErrorV1::InvalidVerifiedTick)
    );
}

#[test]
fn empty_sweep_report_has_zero_counts_and_verified_tick() {
    let report = ArchiveWorkerSweepReportV1::default();

    assert!(report.dispositions().is_empty());
    assert_eq!(report.applied_count(), 0);
    assert_eq!(report.paged_count(), 0);
    assert_eq!(report.already_consumed_count(), 0);
    assert_eq!(report.verified_tick(), 0);
}

#[test]
fn sweep_report_aggregates_dispositions_and_carries_the_persisted_watermark() {
    let report = ArchiveWorkerSweepReportV1::new(
        vec![
            (1, ArchiveReceiptDispositionV1::Paged),
            (2, ArchiveReceiptDispositionV1::Applied),
            (3, ArchiveReceiptDispositionV1::AlreadyConsumed),
            (4, ArchiveReceiptDispositionV1::Paged),
            (5, ArchiveReceiptDispositionV1::Paged),
        ],
        9,
        7,
        true,
        true,
    );

    assert_eq!(report.applied_count(), 1);
    assert_eq!(report.paged_count(), 3);
    assert_eq!(report.already_consumed_count(), 1);
    assert_eq!(report.verified_tick(), 7);
    assert_eq!(report.dispositions().len(), 5);
}

#[test]
fn batch_identity_must_match_the_receipt_exactly() {
    let receipt = PendingArchiveReceiptV1::try_new(2, [0x22; 32]).expect("valid receipt");
    let matching = non_empty_batch(2, [0x22; 32]);
    let wrong_tick = non_empty_batch(3, [0x22; 32]);
    let wrong_hash = non_empty_batch(2, [0x33; 32]);

    assert_eq!(
        archive_batch_matches_receipt_v1(&matching, &receipt),
        Ok(())
    );
    assert_eq!(
        archive_batch_matches_receipt_v1(&wrong_tick, &receipt),
        Err(SemanticArchiveErrorV1::ReceiptMismatch)
    );
    assert_eq!(
        archive_batch_matches_receipt_v1(&wrong_hash, &receipt),
        Err(SemanticArchiveErrorV1::ReceiptMismatch)
    );
}

#[test]
fn contiguous_watermark_never_advances_past_a_pending_tick() {
    // No receipts at all: the watermark stays at zero.
    assert_eq!(archive_contiguous_watermark_v1(None, 0), 0);
    // Everything consumed: the watermark is the highest committed receipt.
    assert_eq!(archive_contiguous_watermark_v1(None, 7), 7);
    // Nothing consumed yet: an empty sweep still reports zero, not the backlog max.
    assert_eq!(archive_contiguous_watermark_v1(Some(1), 5), 0);
    // An undrained earlier tick caps the watermark even though later ticks applied.
    assert_eq!(archive_contiguous_watermark_v1(Some(3), 5), 2);
    // A single pending receipt at the backlog tail leaves the prefix before it.
    assert_eq!(archive_contiguous_watermark_v1(Some(5), 5), 4);
}

#[test]
fn classify_receipt_consumes_quiet_ticks_and_stages_undrained_pages() {
    let empty = outcome(empty_batch(1, [0x11; 32]), 0);
    let non_empty = full_outcome(2, [0x22; 32]);
    let empty_but_undrained = outcome(empty_batch(3, [0x33; 32]), 4);
    let paged = outcome(non_empty_batch(4, [0x44; 32]), 316);

    assert_eq!(
        classify_archive_receipt_v1(&empty),
        ArchiveReceiptPlanV1::Consume
    );
    assert_eq!(
        classify_archive_receipt_v1(&non_empty),
        ArchiveReceiptPlanV1::Consume
    );
    assert_eq!(
        classify_archive_receipt_v1(&empty_but_undrained),
        ArchiveReceiptPlanV1::Stage,
        "an exhausted page budget with dirty pages left still materializes (a no-op stage) \
         instead of deferring forever"
    );
    assert_eq!(
        classify_archive_receipt_v1(&paged),
        ArchiveReceiptPlanV1::Stage,
        "a bounded head batch with an undrained tail materializes without consuming"
    );
    assert_eq!(paged.remaining(), 316);
}

#[test]
fn classify_sweep_preserves_order_and_consumes_complete_receipts() {
    let plans = classify_archive_sweep_v1(vec![
        Ok(outcome(empty_batch(1, [0x11; 32]), 0)),
        Ok(full_outcome(2, [0x22; 32])),
        Ok(outcome(empty_batch(3, [0x33; 32]), 0)),
    ])
    .expect("ordered plans");

    assert_eq!(
        plans,
        vec![
            ArchiveReceiptPlanV1::Consume,
            ArchiveReceiptPlanV1::Consume,
            ArchiveReceiptPlanV1::Consume,
        ]
    );
}

#[test]
fn classify_sweep_stops_at_first_producer_error() {
    let result = classify_archive_sweep_v1(vec![
        Ok(outcome(empty_batch(1, [0x11; 32]), 0)),
        Err(SemanticArchiveErrorV1::InvalidText),
        Ok(outcome(empty_batch(3, [0x33; 32]), 0)),
    ]);

    assert_eq!(result, Err(SemanticArchiveErrorV1::InvalidText));
}

#[test]
fn paged_sweep_model_enforces_the_consume_cap_across_keyset_pages() {
    // The reviewer's composition: one full 256-row page of 255 materializable
    // receipts plus a quiet tail, then another page. The quiet receipt also
    // consumes; the cap must stop this pass at tick 256.
    let mut page_one: Vec<Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1>> = (1..=255)
        .map(|tick| Ok(full_outcome(tick, [0x11; 32])))
        .collect();
    page_one.push(Ok(outcome(empty_batch(256, [0x11; 32]), 0)));
    let page_two: Vec<Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1>> = (257..=258)
        .map(|tick| Ok(full_outcome(tick, [0x22; 32])))
        .collect();

    let model = model_archive_sweep_pages_v1(vec![page_one, page_two]).expect("paged model");

    assert_eq!(
        model.consumed(),
        ARCHIVE_SWEEP_MAX_RECEIPTS_V1,
        "one sweep never consumes past the declared cap, whatever the page composition"
    );
    assert_eq!(
        model.scanned(),
        ARCHIVE_SWEEP_MAX_RECEIPTS_V1,
        "the quiet tail consumes the final slot in this sweep"
    );
    assert_eq!(
        model.plans().len(),
        usize::try_from(model.scanned()).unwrap()
    );
    assert_eq!(model.plans()[255], ArchiveReceiptPlanV1::Consume);
    assert_eq!(model.plans().len(), 256, "the remainder stays pending");
}

#[test]
fn paged_sweep_model_keeps_the_scan_bound_as_the_outer_bound() {
    // Even a fully quiet campaign stops at the independent scan bound.
    let page: Vec<Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1>> = (1..=10)
        .map(|tick| Ok(outcome(empty_batch(tick, [0x11; 32]), 0)))
        .collect();

    let model =
        model_archive_sweep_pages_with_bounds_v1(vec![page], 256, 4).expect("bounded model");

    assert_eq!(model.scanned(), 4);
    assert_eq!(model.consumed(), 4);
    assert!(
        model
            .plans()
            .iter()
            .all(|plan| *plan == ArchiveReceiptPlanV1::Consume),
        "every evaluated quiet receipt settles"
    );
}

#[test]
fn foundation_receipt_pages_its_drain_until_the_tail_converges() {
    // The PER-318 acceptance composition: one foundation receipt dirties 745
    // place pages and 83 county pages. The shared 256-page budget drains the
    // county head plus the leading place prefix first; each following sweep
    // takes the next place prefix; the tail reaches zero only after the last
    // page materializes, and zero remaining is what permits consumption.
    const PLACE_DIRTY: usize = 745;
    const COUNTY_DIRTY: usize = 83;
    const BUDGET: usize = ArchiveDirtyBatchV1::MAX_PAGES;

    let first_place_head = BUDGET - COUNTY_DIRTY;
    assert_eq!(PLACE_DIRTY - first_place_head, 572);

    // Sweep 1: 83 county pages + 173 place pages = 256, 572 places left.
    let sweep_one = outcome(non_empty_batch(42, [0x11; 32]), 572);
    assert_eq!(
        classify_archive_receipt_v1(&sweep_one),
        ArchiveReceiptPlanV1::Stage
    );
    assert_eq!(
        sweep_one.remaining(),
        PLACE_DIRTY - first_place_head,
        "745 places - 173 published = 572 undrained after the first sweep"
    );

    // Sweep 2 takes the next 256 places; sweep 3 the next; sweep 4 drains.
    let sweep_two = outcome(non_empty_batch(42, [0x11; 32]), 316);
    let sweep_three = outcome(non_empty_batch(42, [0x11; 32]), 60);
    let sweep_four = outcome(non_empty_batch(42, [0x11; 32]), 0);
    assert_eq!(sweep_two.remaining(), sweep_one.remaining() - BUDGET);
    assert_eq!(sweep_three.remaining(), sweep_two.remaining() - BUDGET);
    assert_eq!(
        sweep_four.remaining(),
        sweep_three.remaining() - (PLACE_DIRTY - first_place_head - 2 * BUDGET),
        "the final sweep publishes the last 60 places"
    );
    for (label, step) in [
        ("sweep two", &sweep_two),
        ("sweep three", &sweep_three),
        ("sweep four", &sweep_four),
    ] {
        assert_eq!(
            classify_archive_receipt_v1(step),
            if step.remaining() == 0 {
                ArchiveReceiptPlanV1::Consume
            } else {
                ArchiveReceiptPlanV1::Stage
            },
            "{label} keeps materializing the pending receipt"
        );
    }
    assert_eq!(
        sweep_four.remaining(),
        0,
        "the drained tail is what lets the final sweep consume the receipt"
    );
}

#[test]
fn ordered_worker_bounds_every_receipt_evaluation_and_stops_at_stage() {
    let source = include_str!("../src/archive_revision/worker.rs");
    let loop_at = source
        .find("for _ in 0..crate::ARCHIVE_SWEEP_MAX_RECEIPTS_V1")
        .expect("the sole ordered worker caps the entire evaluation loop");
    let next_at = source
        .find("publication::next_work(&mut tx, campaign)")
        .expect("each iteration selects only the earliest unsettled receipt");
    let produce_at = source
        .find("producer.produce(")
        .expect("bounded producer evaluation");
    let publish_at = source
        .find("publication::publish(")
        .expect("atomic publication");
    assert!(
        loop_at < next_at && next_at < produce_at && produce_at < publish_at,
        "the cap encloses selection, evaluation and publication, including quiet receipts"
    );
    assert!(
        source.contains("if mode == ArchiveMaterializeModeV1::Stage {\n            break;"),
        "no later receipt is evaluated against an incomplete earlier publication"
    );
    assert_eq!(ARCHIVE_SWEEP_MAX_RECEIPTS_V1, 256);
}

/// Stub producer returning one scripted outcome per receipt, honoring the
/// page budget like a production producer.
struct ScriptedProducer(ArchiveProducerOutcomeV1);

impl ArchiveDossierProducerV1 for ScriptedProducer {
    fn produce(
        &self,
        _campaign_id: Uuid,
        _receipt: &PendingArchiveReceiptV1,
        _knowledge: &babylon_persistence::ArchiveKnowledgeV1,
        page_budget: usize,
    ) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
        let batch = self.0.batch();
        let pages = batch.pages().len().min(page_budget);
        let head = ArchiveDirtyBatchV1::try_new(
            batch.resolve_tick(),
            *batch.tick_content_hash(),
            batch.pages().iter().take(pages).cloned().collect(),
        )
        .expect("budget-respecting head batch");
        Ok(ArchiveProducerOutcomeV1::new(
            head,
            self.0.remaining() + batch.pages().len() - pages,
        ))
    }
}

fn scripted(batch: ArchiveDirtyBatchV1, remaining: usize) -> ScriptedProducer {
    ScriptedProducer(ArchiveProducerOutcomeV1::new(batch, remaining))
}

fn place_page_input(resolve_tick: u64, tick_content_hash: [u8; 32]) -> ArchivePageInputV1 {
    ArchivePageInputV1::try_new(
        ArchiveSubjectV1::try_new(
            ArchiveSubjectKindV1::Place,
            "2622000".to_owned(),
            "Detroit city".to_owned(),
        )
        .expect("place identity"),
        resolve_tick,
        tick_content_hash,
        "Which overlapping county should organizers investigate next?".to_owned(),
        Vec::new(),
        Vec::new(),
    )
    .expect("place page input")
}

#[test]
fn composite_merges_producer_pages_sorted_and_refuses_duplicate_subjects() {
    let receipt = PendingArchiveReceiptV1::try_new(1, [0x11; 32]).expect("receipt");
    let county_first = CompositeArchiveDossierProducerV1::new(vec![
        Box::new(scripted(non_empty_batch(1, [0x11; 32]), 0)),
        Box::new(scripted(
            ArchiveDirtyBatchV1::try_new(1, [0x11; 32], vec![place_page_input(1, [0x11; 32])])
                .expect("place batch"),
            0,
        )),
    ]);
    let produced = county_first
        .produce(
            Uuid::nil(),
            &receipt,
            &babylon_persistence::ArchiveKnowledgeV1::try_new(Vec::new())
                .expect("empty scripted knowledge"),
            ArchiveDirtyBatchV1::MAX_PAGES,
        )
        .expect("composite merge");
    let order = produced
        .batch()
        .pages()
        .iter()
        .map(|page| page.subject().page_ref().clone())
        .collect::<Vec<_>>();
    assert_eq!(
        order,
        vec![
            ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, "26163".to_owned())
                .expect("county ref"),
            ArchivePageRefV1::try_new(ArchiveSubjectKindV1::Place, "2622000".to_owned())
                .expect("place ref"),
        ],
        "merged pages follow deterministic page-reference order"
    );
    assert_eq!(
        produced.remaining(),
        0,
        "both producers drained: nothing keeps the receipt pending"
    );

    let duplicate = CompositeArchiveDossierProducerV1::new(vec![
        Box::new(scripted(non_empty_batch(1, [0x11; 32]), 0)),
        Box::new(scripted(non_empty_batch(1, [0x11; 32]), 0)),
    ]);
    assert_eq!(
        duplicate.produce(
            Uuid::nil(),
            &receipt,
            &babylon_persistence::ArchiveKnowledgeV1::try_new(Vec::new())
                .expect("empty scripted knowledge"),
            ArchiveDirtyBatchV1::MAX_PAGES
        ),
        Err(SemanticArchiveErrorV1::DuplicateKey),
        "two producers may not claim the same page subject"
    );
}

#[test]
fn composite_threads_the_page_budget_and_sums_the_undrained_remainder() {
    let receipt = PendingArchiveReceiptV1::try_new(1, [0x11; 32]).expect("receipt");
    let county_pages = (0..200)
        .map(|index| {
            ArchivePageInputV1::try_new(
                ArchiveSubjectV1::try_new(
                    ArchiveSubjectKindV1::County,
                    format!("26{index:03}"),
                    "County".to_owned(),
                )
                .expect("county identity"),
                1,
                [0x11; 32],
                "Which neighboring place should organizers investigate next?".to_owned(),
                Vec::new(),
                Vec::new(),
            )
            .expect("county page")
        })
        .collect::<Vec<_>>();
    let composite = CompositeArchiveDossierProducerV1::new(vec![
        Box::new(scripted(
            ArchiveDirtyBatchV1::try_new(1, [0x11; 32], county_pages).expect("county batch"),
            100,
        )),
        Box::new(scripted(
            ArchiveDirtyBatchV1::try_new(1, [0x11; 32], vec![place_page_input(1, [0x11; 32])])
                .expect("place batch"),
            316,
        )),
    ]);
    let produced = composite
        .produce(
            Uuid::nil(),
            &receipt,
            &babylon_persistence::ArchiveKnowledgeV1::try_new(Vec::new())
                .expect("empty scripted knowledge"),
            ArchiveDirtyBatchV1::MAX_PAGES,
        )
        .expect("paged composite merge");

    assert_eq!(
        produced.batch().pages().len(),
        201,
        "both producer heads fit inside one 256-page budget"
    );
    assert_eq!(
        produced.remaining(),
        416,
        "the composite remainder is the exact sum of every producer's undrained tail"
    );
    assert_composite_preserves_unfunded_pages(&receipt);
}

fn assert_composite_preserves_unfunded_pages(receipt: &PendingArchiveReceiptV1) {
    // A producer arriving after the budget is exhausted sees no room and
    // reports its whole dirty set as remainder, exactly like the county side
    // of a foundation receipt that the place head already filled.
    let exhausted = CompositeArchiveDossierProducerV1::new(vec![
        Box::new(scripted(
            ArchiveDirtyBatchV1::try_new(
                1,
                [0x11; 32],
                (0..ArchiveDirtyBatchV1::MAX_PAGES)
                    .map(|index| {
                        ArchivePageInputV1::try_new(
                            ArchiveSubjectV1::try_new(
                                ArchiveSubjectKindV1::County,
                                format!("26{index:03}"),
                                "County".to_owned(),
                            )
                            .expect("full-bound identity"),
                            1,
                            [0x11; 32],
                            "Which neighboring place should organizers investigate next?"
                                .to_owned(),
                            Vec::new(),
                            Vec::new(),
                        )
                        .expect("full-bound page")
                    })
                    .collect(),
            )
            .expect("full-bound batch"),
            0,
        )),
        Box::new(scripted(
            ArchiveDirtyBatchV1::try_new(1, [0x11; 32], vec![place_page_input(1, [0x11; 32])])
                .expect("overflow batch"),
            0,
        )),
    ]);
    let paged = exhausted
        .produce(
            Uuid::nil(),
            receipt,
            &babylon_persistence::ArchiveKnowledgeV1::try_new(Vec::new())
                .expect("empty scripted knowledge"),
            ArchiveDirtyBatchV1::MAX_PAGES,
        )
        .expect("the budget-exhausted composite pages instead of overflowing");
    assert_eq!(paged.batch().pages().len(), ArchiveDirtyBatchV1::MAX_PAGES);
    assert_eq!(
        paged.remaining(),
        1,
        "the unfunded place page stays dirty for the next sweep: nothing truncates, \
         nothing refuses"
    );
    for page in paged.batch().pages() {
        assert_eq!(
            page.subject().page_ref().kind(),
            ArchiveSubjectKindV1::County,
            "the funded head drains before any unfunded producer contributes"
        );
    }
}

#[test]
fn composite_still_refuses_a_producer_that_ignores_the_page_budget() {
    // Paging makes the drain-overflow refusals unreachable in normal
    // operation; the per-batch bound stays as the typed defense behind the
    // budget, so a misbehaving producer refuses loudly instead of writing an
    // over-bound batch.
    struct OverBoundProducer;

    impl ArchiveDossierProducerV1 for OverBoundProducer {
        fn produce(
            &self,
            _campaign_id: Uuid,
            receipt: &PendingArchiveReceiptV1,
            _knowledge: &babylon_persistence::ArchiveKnowledgeV1,
            _page_budget: usize,
        ) -> Result<ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
            let pages = (0..=ArchiveDirtyBatchV1::MAX_PAGES)
                .map(|index| {
                    ArchivePageInputV1::try_new(
                        ArchiveSubjectV1::try_new(
                            ArchiveSubjectKindV1::County,
                            format!("26{index:03}"),
                            "County".to_owned(),
                        )
                        .expect("over-bound identity"),
                        receipt.resolve_tick(),
                        *receipt.tick_content_hash(),
                        "Which neighboring place should organizers investigate next?".to_owned(),
                        Vec::new(),
                        Vec::new(),
                    )
                    .expect("over-bound page")
                })
                .collect::<Vec<_>>();
            let batch = ArchiveDirtyBatchV1::try_new(
                receipt.resolve_tick(),
                *receipt.tick_content_hash(),
                pages,
            )?;
            Ok(ArchiveProducerOutcomeV1::new(batch, 0))
        }
    }

    let receipt = PendingArchiveReceiptV1::try_new(1, [0x11; 32]).expect("receipt");
    let composite = CompositeArchiveDossierProducerV1::new(vec![Box::new(OverBoundProducer)]);
    assert_eq!(
        composite.produce(
            Uuid::nil(),
            &receipt,
            &babylon_persistence::ArchiveKnowledgeV1::try_new(Vec::new())
                .expect("empty scripted knowledge"),
            ArchiveDirtyBatchV1::MAX_PAGES
        ),
        Err(SemanticArchiveErrorV1::CollectionBound),
        "the batch bound refuses a budget-ignoring producer; paging never truncates"
    );
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and one committed tick"]
fn live_worker_consumes_every_evaluated_empty_receipt() {
    let (config, campaign_id) = live_contract_target();
    SemanticArchiveStoreV1::new(&config)
        .install_schema()
        .expect("schema install");
    let mut worker = babylon_persistence::ArchiveWorkerV1::new(&config);
    let report = worker
        .sweep_once(campaign_id, &NullArchiveDossierProducerV1::new())
        .expect("null sweep succeeds");

    assert_eq!(report.applied_count(), report.dispositions().len());
    assert_eq!(report.already_consumed_count(), 0);
    for (_, disposition) in report.dispositions() {
        assert_eq!(*disposition, ArchiveReceiptDispositionV1::Applied);
    }
}

fn live_contract_target() -> (Config, babylon_persistence::CampaignId) {
    let dsn = std::env::var(LIVE_DSN_ENV).expect("disposable live DSN");
    let expected_canary = std::env::var(LIVE_CANARY_ENV).expect("disposable canary");
    let config = Config::from_str(&dsn).expect("live DSN parses");
    let mut client = config
        .clone()
        .connect(NoTls)
        .expect("live preflight connects");
    let actual_canary: String = client
        .query_one(
            "SELECT pg_catalog.current_setting('babylon.per20_disposable', true)",
            &[],
        )
        .expect("live canary query")
        .try_get(0)
        .expect("live canary decode");
    assert_eq!(actual_canary, expected_canary);

    let campaign_uuid =
        Uuid::parse_str(&std::env::var("BABYLON_ARCHIVE_TEST_CAMPAIGN_ID").expect("live campaign"))
            .expect("live campaign UUID");
    (
        config,
        babylon_persistence::CampaignId::from_uuid(campaign_uuid),
    )
}
