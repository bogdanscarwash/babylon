//! Bounded ordered receipt draining with coherent committed progress.

use super::{publication, tick_knowledge, ArchiveReadScope};
use crate::archive::{database, decode};
use crate::{
    identity::CampaignId, ArchiveDossierProducer, ArchiveMaterializeDisposition,
    ArchiveMaterializeMode, ArchiveReceiptDisposition, ArchiveWorkerCancellation,
    ArchiveWorkerSweepReport, SemanticArchiveError, SemanticArchiveStore,
};
use postgres::{Client, IsolationLevel};

pub(crate) fn sweep(
    store: &SemanticArchiveStore,
    campaign: CampaignId,
    producer: &dyn ArchiveDossierProducer,
    cancellation: &ArchiveWorkerCancellation,
) -> Result<ArchiveWorkerSweepReport, SemanticArchiveError> {
    cancellation.check()?;
    let mut client = store.connect("connect ordered Archive worker")?;
    publication::with_campaign_lock(&mut client, campaign, |client| {
        sweep_locked(
            client,
            campaign,
            producer,
            cancellation,
            crate::ARCHIVE_SWEEP_MAX_RECEIPTS,
        )
    })
}

fn sweep_locked(
    client: &mut Client,
    campaign: CampaignId,
    producer: &dyn ArchiveDossierProducer,
    cancellation: &ArchiveWorkerCancellation,
    receipt_budget: i64,
) -> Result<ArchiveWorkerSweepReport, SemanticArchiveError> {
    let mut dispositions = Vec::new();
    for _ in 0..receipt_budget {
        cancellation.check()?;
        let mut tx = client
            .build_transaction()
            .isolation_level(IsolationLevel::Serializable)
            .read_only(false)
            .start()
            .map_err(|error| database("begin ordered Archive producer transaction", &error))?;
        crate::current_schema::require_current_schema(&mut tx)
            .map_err(SemanticArchiveError::CurrentSchema)?;
        let Some(receipt) = publication::next_receipt(&mut tx, campaign)? else {
            break;
        };
        let scope = ArchiveReadScope::committed(
            campaign,
            receipt.resolve_tick(),
            *receipt.tick_content_hash(),
        )?;
        let known = tick_knowledge::pin(&mut tx, &scope)?;
        let outcome = producer.produce(
            *campaign.as_uuid(),
            &receipt,
            &known,
            crate::ArchiveDirtyBatch::MAX_PAGES,
        )?;
        let mode = if outcome.remaining() == 0 {
            ArchiveMaterializeMode::Consume
        } else {
            ArchiveMaterializeMode::Stage
        };
        cancellation.check()?;
        let report =
            publication::publish(&mut tx, campaign, &receipt, outcome.batch(), mode, &known)?;
        cancellation.check()?;
        tx.commit()
            .map_err(|error| database("commit ordered Archive producer transaction", &error))?;
        let disposition = match (mode, report.disposition()) {
            (_, ArchiveMaterializeDisposition::AlreadyConsumed) => {
                ArchiveReceiptDisposition::AlreadyConsumed
            }
            (ArchiveMaterializeMode::Stage, _) => ArchiveReceiptDisposition::Paged,
            (ArchiveMaterializeMode::Consume, _) => ArchiveReceiptDisposition::Applied,
        };
        dispositions.push((receipt.resolve_tick(), disposition));
        // Never evaluate a later quiet receipt against an incomplete earlier head.
        if mode == ArchiveMaterializeMode::Stage {
            break;
        }
    }
    read_progress(client, campaign, dispositions)
}

fn read_progress(
    client: &mut Client,
    campaign: CampaignId,
    dispositions: Vec<(u64, ArchiveReceiptDisposition)>,
) -> Result<ArchiveWorkerSweepReport, SemanticArchiveError> {
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
        return Err(SemanticArchiveError::StoredPageMismatch);
    }
    let report = ArchiveWorkerSweepReport::new(dispositions, durable, processed, pending);
    tx.commit()
        .map_err(|error| database("finish coherent Archive progress", &error))?;
    Ok(report)
}

#[cfg(test)]
mod live_tests;
