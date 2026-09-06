//! Bounded ordered receipt draining followed by exact adoption validation.

use super::{
    publication::{self, Work},
    tick_knowledge,
};
use crate::archive::{database, decode};
use crate::{
    ArchiveDossierProducerV1, ArchiveMaterializeDispositionV1, ArchiveMaterializeModeV1,
    ArchiveReceiptDispositionV1, ArchiveWorkerCancellationV1, ArchiveWorkerSweepReportV1,
    CampaignId, SemanticArchiveErrorV1, SemanticArchiveStoreV1,
};
use postgres::{Client, IsolationLevel};

pub(crate) fn sweep(
    store: &SemanticArchiveStoreV1,
    campaign: CampaignId,
    producer: &dyn ArchiveDossierProducerV1,
    cancellation: &ArchiveWorkerCancellationV1,
) -> Result<ArchiveWorkerSweepReportV1, SemanticArchiveErrorV1> {
    cancellation.check()?;
    let mut client = store.connect("connect ordered Archive worker")?;
    publication::with_campaign_lock(&mut client, campaign, |client| {
        sweep_locked(client, campaign, producer, cancellation)
    })
}

fn sweep_locked(
    client: &mut Client,
    campaign: CampaignId,
    producer: &dyn ArchiveDossierProducerV1,
    cancellation: &ArchiveWorkerCancellationV1,
) -> Result<ArchiveWorkerSweepReportV1, SemanticArchiveErrorV1> {
    let mut dispositions = Vec::new();
    for _ in 0..crate::ARCHIVE_SWEEP_MAX_RECEIPTS_V1 {
        cancellation.check()?;
        let mut tx = client
            .build_transaction()
            .isolation_level(IsolationLevel::Serializable)
            .start()
            .map_err(|error| database("begin ordered Archive producer transaction", &error))?;
        let Some(work) = publication::next_work(&mut tx, campaign)? else {
            break;
        };
        let known = tick_knowledge::pin(&mut tx, &work.scope(campaign)?)?;
        let outcome = producer.produce(
            *campaign.as_uuid(),
            work.receipt(),
            &known,
            crate::ArchiveDirtyBatchV1::MAX_PAGES,
        )?;
        let coverage = if matches!(work, Work::Cutover(_)) {
            Some(producer.cutover_subjects(*campaign.as_uuid(), work.receipt(), &known)?)
        } else {
            None
        };
        let mode = if outcome.remaining() == 0 {
            ArchiveMaterializeModeV1::Consume
        } else {
            ArchiveMaterializeModeV1::Stage
        };
        cancellation.check()?;
        let report = publication::publish(
            &mut tx,
            campaign,
            &work,
            outcome.batch(),
            mode,
            &known,
            coverage.as_deref(),
        )?;
        cancellation.check()?;
        tx.commit()
            .map_err(|error| database("commit ordered Archive producer transaction", &error))?;
        if let Work::Receipt(receipt) = work {
            let disposition = match (mode, report.disposition()) {
                (_, ArchiveMaterializeDispositionV1::AlreadyConsumed) => {
                    ArchiveReceiptDispositionV1::AlreadyConsumed
                }
                (ArchiveMaterializeModeV1::Stage, _) => ArchiveReceiptDispositionV1::Paged,
                (ArchiveMaterializeModeV1::Consume, _) => ArchiveReceiptDispositionV1::Applied,
            };
            dispositions.push((receipt.resolve_tick(), disposition));
        }
        // Never evaluate a later quiet receipt against an incomplete earlier head.
        if mode == ArchiveMaterializeModeV1::Stage {
            break;
        }
    }
    read_progress(client, campaign, dispositions)
}

fn read_progress(
    client: &mut Client,
    campaign: CampaignId,
    dispositions: Vec<(u64, ArchiveReceiptDispositionV1)>,
) -> Result<ArchiveWorkerSweepReportV1, SemanticArchiveErrorV1> {
    let mut tx = client
        .build_transaction()
        .isolation_level(IsolationLevel::RepeatableRead)
        .start()
        .map_err(|error| database("begin coherent Archive progress", &error))?;
    // Admit the exact seal and pending receipt identities in this same snapshot.
    let pending = publication::next_work(&mut tx, campaign)?.is_some();
    let row=tx.query_one("SELECT COALESCE(v.durable_tick,0),COALESCE(v.processed_tick,0),r.sealed \
        FROM public.v_archive_retention_v2 r LEFT JOIN public.v_archive_verification_v1 v USING(campaign_id) \
        WHERE r.campaign_id=$1",&[campaign.as_uuid()])
        .map_err(|error|database("read ordered Archive maintenance progress",&error))?;
    let durable = super::storage::unsigned(decode(&row, 0)?)?;
    let processed = super::storage::unsigned(decode(&row, 1)?)?;
    if processed > durable {
        return Err(SemanticArchiveErrorV1::StoredPageMismatch);
    }
    let report = ArchiveWorkerSweepReportV1::new(
        dispositions,
        durable,
        processed,
        decode(&row, 2)?,
        pending,
    );
    tx.commit()
        .map_err(|error| database("finish coherent Archive progress", &error))?;
    Ok(report)
}
