//! One ordered atomic publication path; no mutable-head write remains.

use super::{knowledge, record::RevisionRecord, storage, tick_knowledge, ArchiveReadScopeV2};
use crate::archive::{database, decode, decode_digest, mint_page_atoms, persist_atom_rows};
use crate::{
    ArchiveDirtyBatchV1, ArchiveKnowledgeV1, ArchiveMaterializeDispositionV1,
    ArchiveMaterializeModeV1, ArchiveMaterializeReportV1, ArchivePageInputV1, ArchivePageRefV1,
    CampaignId, FogSafeArchiveRendererV1, MaterializedArchivePageV1, PendingArchiveReceiptV1,
    SemanticArchiveErrorV1, SemanticArchiveStoreV1,
};
use postgres::{Client, GenericClient, IsolationLevel};
use sha2::{Digest as _, Sha256};

pub(super) fn with_campaign_lock<T>(
    client: &mut Client,
    campaign: CampaignId,
    operation: impl FnOnce(&mut Client) -> Result<T, SemanticArchiveErrorV1>,
) -> Result<T, SemanticArchiveErrorV1> {
    let mut hash = Sha256::new();
    hash.update(b"babylon.archive-campaign-lock.v2\0");
    hash.update(campaign.canonical_bytes());
    let bytes: [u8; 32] = hash.finalize().into();
    let key = i64::from_be_bytes(
        bytes[..8]
            .try_into()
            .map_err(|_| SemanticArchiveErrorV1::InvalidIdentity)?,
    );
    client
        .query_one("SELECT pg_catalog.pg_advisory_lock($1)", &[&key])
        .map_err(|error| database("lock ordered Archive campaign publication", &error))?;
    let result = operation(client);
    let unlocked = client
        .query_one("SELECT pg_catalog.pg_advisory_unlock($1)", &[&key])
        .map_err(|error| database("unlock ordered Archive campaign publication", &error))
        .and_then(|row| decode::<bool>(&row, 0));
    match (result, unlocked) {
        (Err(error), _) | (Ok(_), Err(error)) => Err(error),
        (Ok(value), Ok(true)) => Ok(value),
        (Ok(_), Ok(false)) => Err(SemanticArchiveErrorV1::SchemaMismatch),
    }
}

pub(super) fn next_receipt(
    client: &mut impl GenericClient,
    campaign: CampaignId,
) -> Result<Option<PendingArchiveReceiptV1>, SemanticArchiveErrorV1> {
    let pending=client.query_opt("SELECT marker.resolve_tick,marker.tick_content_hash,dirty.tick_content_hash \
        FROM babylon_state.tick_commit marker LEFT JOIN babylon_state.archive_dirty_receipt_v1 dirty \
        USING(campaign_id,resolve_tick) LEFT JOIN babylon_meta.archive_receipt_consumption_v1 consumed \
        ON consumed.campaign_id=marker.campaign_id AND consumed.resolve_tick=marker.resolve_tick \
        AND consumed.tick_content_hash=marker.tick_content_hash \
        WHERE marker.campaign_id=$1 AND consumed.campaign_id IS NULL ORDER BY marker.resolve_tick LIMIT 1", &[campaign.as_uuid()])
        .map_err(|error|database("read earliest unsettled Archive marker",&error))?;
    let pending = pending
        .map(|row| {
            let hash = decode_digest(&row, 1)?;
            if decode_digest(&row, 2)? != hash {
                return Err(SemanticArchiveErrorV1::ReceiptMismatch);
            }
            PendingArchiveReceiptV1::try_new(storage::unsigned(decode(&row, 0)?)?, hash)
        })
        .transpose()?;
    Ok(pending)
}

pub(crate) fn materialize(
    store: &SemanticArchiveStoreV1,
    campaign: CampaignId,
    batch: &ArchiveDirtyBatchV1,
    mode: ArchiveMaterializeModeV1,
) -> Result<ArchiveMaterializeReportV1, SemanticArchiveErrorV1> {
    let mut client = store.connect("connect immutable Archive materializer")?;
    with_campaign_lock(&mut client, campaign, |client| {
        let mut tx = client
            .build_transaction()
            .isolation_level(IsolationLevel::Serializable)
            .start()
            .map_err(|error| database("begin immutable Archive batch", &error))?;
        let scope = ArchiveReadScopeV2::committed(
            campaign,
            batch.resolve_tick(),
            *batch.tick_content_hash(),
        )?;
        validate_receipt(&mut tx, &scope)?;
        let known = tick_knowledge::pin(&mut tx, &scope)?;
        let receipt =
            PendingArchiveReceiptV1::try_new(batch.resolve_tick(), *batch.tick_content_hash())?;
        let report = publish(&mut tx, campaign, &receipt, batch, mode, &known)?;
        tx.commit()
            .map_err(|error| database("commit immutable Archive batch", &error))?;
        Ok(report)
    })
}

pub(super) fn publish(
    client: &mut impl GenericClient,
    campaign: CampaignId,
    receipt: &PendingArchiveReceiptV1,
    batch: &ArchiveDirtyBatchV1,
    mode: ArchiveMaterializeModeV1,
    known: &ArchiveKnowledgeV1,
) -> Result<ArchiveMaterializeReportV1, SemanticArchiveErrorV1> {
    crate::archive_batch_matches_receipt_v1(batch, receipt)?;
    let scope = ArchiveReadScopeV2::committed(
        campaign,
        receipt.resolve_tick(),
        *receipt.tick_content_hash(),
    )?;
    validate_receipt(client, &scope)?;
    // Keep campaign deletion ordered after this publication and its final claim.
    client
        .query_one(
            "SELECT campaign_id FROM babylon_meta.campaign WHERE campaign_id=$1 FOR UPDATE",
            &[campaign.as_uuid()],
        )
        .map_err(|error| database("hold Archive campaign during publication", &error))?;
    if reconcile(client, campaign, batch, known)? {
        return Ok(ArchiveMaterializeReportV1 {
            disposition: ArchiveMaterializeDispositionV1::AlreadyConsumed,
            pages: Vec::new(),
        });
    }
    if next_receipt(client, campaign)?.as_ref() != Some(receipt) {
        return Err(SemanticArchiveErrorV1::ArchiveOrderViolation);
    }
    if tick_knowledge::load(client, &scope)? != *known {
        return Err(SemanticArchiveErrorV1::ReceiptConflict);
    }
    let renderer = FogSafeArchiveRendererV1::new()?;
    let pages = batch
        .pages()
        .iter()
        .map(|input| publish_page(client, &renderer, &scope, input, known))
        .collect::<Result<Vec<_>, _>>()?;
    if mode == ArchiveMaterializeModeV1::Consume {
        claim(client, campaign, batch, known)?;
    }
    Ok(ArchiveMaterializeReportV1 {
        disposition: ArchiveMaterializeDispositionV1::Applied,
        pages,
    })
}

fn validate_receipt(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScopeV2,
) -> Result<(), SemanticArchiveErrorV1> {
    let campaign = scope.campaign_id();
    let row = client
        .query_opt(
            "SELECT dirty.tick_content_hash,marker.tick_content_hash \
        FROM babylon_state.archive_dirty_receipt_v1 dirty JOIN babylon_state.tick_commit marker \
        USING(campaign_id,resolve_tick) WHERE dirty.campaign_id=$1 AND dirty.resolve_tick=$2 \
        FOR SHARE OF dirty,marker",
            &[campaign.as_uuid(), &storage::signed(scope.tick())?],
        )
        .map_err(|error| database("validate ordered Archive source receipt", &error))?
        .ok_or(SemanticArchiveErrorV1::MissingCommittedReceipt)?;
    if Some(decode_digest(&row, 0)?) != scope.tick_content_hash()
        || Some(decode_digest(&row, 1)?) != scope.tick_content_hash()
    {
        return Err(SemanticArchiveErrorV1::ReceiptMismatch);
    }
    Ok(())
}

fn publish_page(
    client: &mut impl GenericClient,
    renderer: &FogSafeArchiveRendererV1,
    scope: &ArchiveReadScopeV2,
    input: &ArchivePageInputV1,
    known: &ArchiveKnowledgeV1,
) -> Result<MaterializedArchivePageV1, SemanticArchiveErrorV1> {
    let (page, emission) = renderer.render_with_emission(input, known)?;
    let atoms = mint_page_atoms(scope.campaign_id(), scope.tick(), input, known)?;
    let mut record = RevisionRecord {
        source: scope.clone(),
        subject: input.subject().page_ref().clone(),
        effective_tick: scope.tick(),
        title: input.subject().title().to_owned(),
        template_sha256: crate::ARCHIVE_PAGE_TEMPLATE_SHA256_V1,
        content_sha256: page.sha256(),
        markdown: page.markdown().to_owned(),
        search_text: page.search_text().to_owned(),
        provenance_json: serde_json::to_string(page.citations())
            .map_err(|_| SemanticArchiveErrorV1::InvalidText)?,
        atoms,
        grants: Vec::new(),
        emission,
    };
    record.grants = knowledge::capture(client, &record)?;
    let minted = persist_atom_rows(client, scope.campaign_id(), &record.atoms)?;
    let persisted = storage::insert(client, &record)?;
    Ok(MaterializedArchivePageV1 {
        page_ref: record.subject,
        page,
        persisted,
        atoms: minted,
    })
}

fn reconcile(
    client: &mut impl GenericClient,
    campaign: CampaignId,
    batch: &ArchiveDirtyBatchV1,
    known: &ArchiveKnowledgeV1,
) -> Result<bool, SemanticArchiveErrorV1> {
    let row = client
        .query_opt(
            "SELECT tick_content_hash,batch_sha256,worker_contract_sha256,knowledge_sha256 \
        FROM babylon_meta.archive_receipt_consumption_v1 WHERE campaign_id=$1 AND resolve_tick=$2",
            &[campaign.as_uuid(), &storage::signed(batch.resolve_tick())?],
        )
        .map_err(|error| database("reconcile immutable Archive receipt", &error))?;
    let Some(row) = row else {
        return Ok(false);
    };
    if decode_digest(&row, 0)? != *batch.tick_content_hash()
        || decode_digest(&row, 1)? != batch.sha256()
        || decode_digest(&row, 2)? != crate::archive_worker_contract_sha256_v1()
        || decode_digest(&row, 3)? != known.sha256()
    {
        return Err(SemanticArchiveErrorV1::ReceiptConflict);
    }
    Ok(true)
}

fn claim(
    client: &mut impl GenericClient,
    campaign: CampaignId,
    batch: &ArchiveDirtyBatchV1,
    known: &ArchiveKnowledgeV1,
) -> Result<(), SemanticArchiveErrorV1> {
    client.execute("INSERT INTO babylon_meta.archive_receipt_consumption_v1 \
        (campaign_id,resolve_tick,tick_content_hash,batch_sha256,worker_contract_sha256,knowledge_sha256) \
        VALUES($1,$2,$3,$4,$5,$6)", &[campaign.as_uuid(),&storage::signed(batch.resolve_tick())?,
        &&batch.tick_content_hash()[..],&&batch.sha256()[..],&&crate::archive_worker_contract_sha256_v1()[..],&&known.sha256()[..]])
        .map_err(|error|database("claim ordered immutable Archive receipt",&error))?;
    Ok(())
}

/// Compare the full emitted profile at the original source K. This preserves a
/// quiet page's K and citations while detecting changes in values, labels,
/// question, unknown links, and the pinned knowledge projection at current T.
pub(crate) fn select_dirty_pages<T>(
    config: &postgres::Config,
    campaign: CampaignId,
    receipt: &PendingArchiveReceiptV1,
    known: &ArchiveKnowledgeV1,
    plans: &[T],
    budget: usize,
    make: impl Fn(&T, u64, [u8; 32]) -> Result<ArchivePageInputV1, SemanticArchiveErrorV1>,
) -> Result<crate::ArchiveProducerOutcomeV1, SemanticArchiveErrorV1> {
    let mut client =
        SemanticArchiveStoreV1::new(config).connect("connect retained producer comparison")?;
    let mut tx = client
        .build_transaction()
        .isolation_level(IsolationLevel::RepeatableRead)
        .read_only(true)
        .start()
        .map_err(|error| database("begin retained producer comparison", &error))?;
    let renderer = FogSafeArchiveRendererV1::new()?;
    let mut pages = Vec::new();
    let mut remaining = 0usize;
    for plan in plans {
        let input = make(plan, receipt.resolve_tick(), *receipt.tick_content_hash())?;
        if !known.knows_subject(input.subject().page_ref()) {
            continue;
        }
        let subject = input.subject().page_ref();
        let row = tx
            .query_opt(
                &format!(
                    "SELECT {} FROM babylon_meta.archive_page_revision_v2 \
            WHERE campaign_id=$1 AND subject_kind=$2 AND subject_id=$3 AND effective_tick<=$4 \
            ORDER BY effective_tick DESC LIMIT 1",
                    storage::COLUMNS
                ),
                &[
                    campaign.as_uuid(),
                    &subject.kind().as_str(),
                    &subject.id(),
                    &storage::signed(receipt.resolve_tick())?,
                ],
            )
            .map_err(|error| database("read exact producer comparison revision", &error))?;
        let stored = row
            .map(|row| storage::decode_record(&mut tx, &row, storage::ReadAuthority::Writer))
            .transpose()?;
        let quiet = if let Some(record) = stored {
            let old = make(
                plan,
                record.source.tick(),
                record
                    .source
                    .tick_content_hash()
                    .ok_or(SemanticArchiveErrorV1::StoredPageMismatch)?,
            )?;
            let (expected, witness) = renderer.render_with_emission(&old, known)?;
            record.title == old.subject().title()
                && record.markdown == expected.markdown()
                && record.search_text == expected.search_text()
                && record.emission == witness
                && record.provenance_json
                    == serde_json::to_string(expected.citations())
                        .map_err(|_| SemanticArchiveErrorV1::InvalidText)?
        } else {
            false
        };
        if !quiet {
            if pages.len() < budget.min(ArchiveDirtyBatchV1::MAX_PAGES) {
                pages.push(input);
            } else {
                remaining = remaining
                    .checked_add(1)
                    .ok_or(SemanticArchiveErrorV1::CollectionBound)?;
            }
        }
    }
    tx.commit()
        .map_err(|error| database("commit retained producer comparison read", &error))?;
    Ok(crate::ArchiveProducerOutcomeV1::new(
        ArchiveDirtyBatchV1::try_new(receipt.resolve_tick(), *receipt.tick_content_hash(), pages)?,
        remaining,
    ))
}
