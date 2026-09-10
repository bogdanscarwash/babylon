//! Bounded ordered receipt draining with coherent committed progress.

use super::{publication, tick_knowledge, ArchiveReadScopeV2};
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
        let Some(receipt) = publication::next_receipt(&mut tx, campaign)? else {
            break;
        };
        let scope = ArchiveReadScopeV2::committed(
            campaign,
            receipt.resolve_tick(),
            *receipt.tick_content_hash(),
        )?;
        let known = tick_knowledge::pin(&mut tx, &scope)?;
        let outcome = producer.produce(
            *campaign.as_uuid(),
            &receipt,
            &known,
            crate::ArchiveDirtyBatchV1::MAX_PAGES,
        )?;
        let mode = if outcome.remaining() == 0 {
            ArchiveMaterializeModeV1::Consume
        } else {
            ArchiveMaterializeModeV1::Stage
        };
        cancellation.check()?;
        let report =
            publication::publish(&mut tx, campaign, &receipt, outcome.batch(), mode, &known)?;
        cancellation.check()?;
        tx.commit()
            .map_err(|error| database("commit ordered Archive producer transaction", &error))?;
        let disposition = match (mode, report.disposition()) {
            (_, ArchiveMaterializeDispositionV1::AlreadyConsumed) => {
                ArchiveReceiptDispositionV1::AlreadyConsumed
            }
            (ArchiveMaterializeModeV1::Stage, _) => ArchiveReceiptDispositionV1::Paged,
            (ArchiveMaterializeModeV1::Consume, _) => ArchiveReceiptDispositionV1::Applied,
        };
        dispositions.push((receipt.resolve_tick(), disposition));
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
    // Admit pending receipt identities in this same committed snapshot.
    let pending = publication::next_receipt(&mut tx, campaign)?.is_some();
    let row = tx
        .query_one(
            "SELECT durable_tick,processed_tick \
        FROM public.v_archive_verification_v1 WHERE campaign_id=$1",
            &[campaign.as_uuid()],
        )
        .map_err(|error| database("read ordered Archive maintenance progress", &error))?;
    let durable = super::storage::unsigned(decode(&row, 0)?)?;
    let processed = super::storage::unsigned(decode(&row, 1)?)?;
    if processed > durable {
        return Err(SemanticArchiveErrorV1::StoredPageMismatch);
    }
    let report = ArchiveWorkerSweepReportV1::new(dispositions, durable, processed, pending);
    tx.commit()
        .map_err(|error| database("finish coherent Archive progress", &error))?;
    Ok(report)
}
