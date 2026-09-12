//! The production receipt loop stops at its budget and resumes through real commits.
//! The public worker always uses `ARCHIVE_SWEEP_MAX_RECEIPTS` (256). These live
//! boundaries use four of six genuinely committed ticks, within the current
//! campaign horizon; `archive_worker_contract` retains the literal 256 proof.

use super::*;

fn bounded_sweep(
    config: &Config,
    campaign: CampaignId,
    producer: &dyn ArchiveDossierProducer,
) -> Result<crate::ArchiveWorkerSweepReport, SemanticArchiveError> {
    let store = SemanticArchiveStore::new(config);
    let mut client = store.connect("connect bounded live Archive worker")?;
    let cancellation = crate::ArchiveWorkerCancellation::default();
    cancellation.check()?;
    super::super::publication::with_campaign_lock(&mut client, campaign, |client| {
        super::super::sweep_locked(client, campaign, producer, &cancellation, 4)
    })
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_worker_quiet_backlog_respects_the_bound_and_reaches_later_changed_content() {
    const TICKS: u64 = 6;
    let target = LiveWorkerTarget::create_with_grants(
        "archiveworkersweeppage",
        0x2200_0000_0000_0000_0000_0000_0000_00a7,
        TICKS,
        &[1, TICKS],
    );
    let producer = QuietExceptProducer {
        materialize_tick: TICKS,
    };
    let first =
        bounded_sweep(&target.config, target.campaign_id, &producer).expect("bounded quiet prefix");
    assert_eq!(first.applied_count(), 4);
    assert_eq!(first.verified_tick(), 4);
    assert!(first.has_pending_work());
    assert_eq!(first.durable_tick(), TICKS);
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 0);
    let second =
        bounded_sweep(&target.config, target.campaign_id, &producer).expect("remaining prefix");
    assert_eq!(second.applied_count(), 2);
    assert_eq!(second.verified_tick(), TICKS);
    assert!(!second.has_pending_work());
    assert_eq!(second.durable_tick(), TICKS);
    assert_eq!(archive_page_count(&target.config, target.campaign_id), 1);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        6
    );
    target.finish();
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and committed ticks"]
fn live_worker_stops_at_the_consume_cap_and_leaves_the_remainder_pending() {
    // Every pending receipt comes from the current material runtime.
    // Tick four is quiet and still fills the fourth slot in this sweep.
    const TICKS: u64 = 6;
    let target = LiveWorkerTarget::create_with_grants(
        "archiveworkercap",
        0x2200_0000_0000_0000_0000_0000_0000_00a9,
        TICKS,
        &[1, 2, 3],
    );

    let report = bounded_sweep(
        &target.config,
        target.campaign_id,
        &ChangedExceptProducer { quiet_tick: 4 },
    )
    .expect("one sweep stops at the consume cap");

    assert_eq!(
        report.applied_count(),
        4,
        "three changed receipts plus the quiet receipt at tick four"
    );
    assert!(report.has_pending_work());
    assert_eq!(report.durable_tick(), TICKS);
    assert_eq!(report.paged_count(), 0);
    assert_eq!(report.already_consumed_count(), 0);
    assert_eq!(report.dispositions().len(), 4);
    let last = report
        .dispositions()
        .last()
        .expect("the final receipt in the bounded prefix settles");
    assert_eq!(*last, (4, ArchiveReceiptDisposition::Applied));
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        4,
        "the consume cap is a hard per-sweep stop, whatever the page composition"
    );
    assert_eq!(
        report.verified_tick(),
        4,
        "the quiet tick four settles within the same bounded prefix"
    );

    // A fresh worker connection must resume the durable pending suffix.
    let second = bounded_sweep(&target.config, target.campaign_id, &StubPageProducer)
        .expect("the remainder stays pending for the next invocation");
    assert_eq!(second.applied_count(), 2, "ticks 5..=6");
    assert_eq!(second.paged_count(), 0);
    assert!(!second.has_pending_work());
    assert_eq!(second.verified_tick(), TICKS);
    assert_eq!(second.durable_tick(), TICKS);
    assert_eq!(
        receipt_consumption_count(&target.config, target.campaign_id),
        i64::try_from(TICKS).expect("bounded tick count")
    );
    target.finish();
}
