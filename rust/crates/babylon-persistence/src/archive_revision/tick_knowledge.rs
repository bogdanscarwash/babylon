//! Receipt-pinned disclosure membership, shared by every staged batch and consumption at the same tick.

use super::storage::{signed, unsigned};
use super::ArchiveReadScope;
use crate::archive::{database, decode, decode_digest, decode_subject_kind, read_knowledge};
use crate::{
    ArchiveCitation, ArchiveKnowledge, ArchiveKnowledgeGrant, ArchivePageRef, SemanticArchiveError,
};
use postgres::GenericClient;

pub(super) fn pin(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScope,
) -> Result<ArchiveKnowledge, SemanticArchiveError> {
    let campaign = scope.campaign_id();
    let tick = signed(scope.tick())?;
    if client.query_opt("SELECT 1 FROM babylon_meta.archive_tick_knowledge_v2 WHERE campaign_id=$1 AND resolve_tick=$2",
        &[campaign.as_uuid(),&tick]).map_err(|error|database("inspect pinned Archive knowledge",&error))?.is_none() {
        let knowledge=read_knowledge(client,campaign,tick)?;
        let count=i32::try_from(knowledge.rows().count()).map_err(|_|SemanticArchiveError::CollectionBound)?;
        let hash=scope.tick_content_hash().ok_or(SemanticArchiveError::InvalidVerifiedTick)?;
        client.execute("INSERT INTO babylon_meta.archive_tick_knowledge_v2 \
            (campaign_id,resolve_tick,tick_content_hash,worker_contract_sha256,knowledge_sha256,grant_count) VALUES($1,$2,$3,$4,$5,$6)",
            &[campaign.as_uuid(),&tick,&&hash[..],&&crate::archive_worker_contract_sha256()[..],&&knowledge.sha256()[..],&count])
            .map_err(|error|database("pin exact Archive knowledge identity",&error))?;
        // Preserve the captured membership without a database round trip per grant.
        let mut subject_kinds = Vec::new();
        let mut subject_ids = Vec::new();
        let mut grant_keys = Vec::new();
        for grant in knowledge.rows() {
            subject_kinds.push(grant.page_ref.kind().as_str());
            subject_ids.push(grant.page_ref.id());
            grant_keys.push(grant.grant_key.as_str());
        }
        let inserted = client.execute("INSERT INTO babylon_meta.archive_tick_knowledge_member_v2 \
            (campaign_id,resolve_tick,subject_kind,subject_id,grant_key) \
            SELECT $1,$2,membership.subject_kind,membership.subject_id,membership.grant_key \
            FROM UNNEST($3::text[],$4::text[],$5::text[]) \
            AS membership(subject_kind,subject_id,grant_key)",
            &[campaign.as_uuid(),&tick,&subject_kinds,&subject_ids,&grant_keys])
            .map_err(|error|database("pin exact Archive knowledge membership",&error))?;
        if inserted != u64::try_from(count).map_err(|_|SemanticArchiveError::CollectionBound)? {
            return Err(SemanticArchiveError::StoredPageMismatch);
        }
        // Readers authenticate the complete grant cohort immediately after this
        // bulk load. Supply current row counts before their first query instead
        // of waiting for autovacuum: an empty-table estimate turns the late-grant
        // anti join into a quadratic scan of a newly populated cohort.
        client.batch_execute("ANALYZE babylon_meta.archive_knowledge_grant_v1; \
            ANALYZE babylon_meta.archive_tick_knowledge_member_v2")
            .map_err(|error|database("refresh pinned Archive knowledge statistics",&error))?;
    }
    load(client, scope)
}

pub(super) fn load(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScope,
) -> Result<ArchiveKnowledge, SemanticArchiveError> {
    let campaign = scope.campaign_id();
    let tick = signed(scope.tick())?;
    let header = client
        .query_opt(
            "SELECT tick_content_hash,worker_contract_sha256,knowledge_sha256,grant_count \
        FROM babylon_meta.archive_tick_knowledge_v2 WHERE campaign_id=$1 AND resolve_tick=$2",
            &[campaign.as_uuid(), &tick],
        )
        .map_err(|error| database("read pinned Archive knowledge identity", &error))?
        .ok_or(SemanticArchiveError::StoredPageMismatch)?;
    if Some(decode_digest(&header, 0)?) != scope.tick_content_hash()
        || decode_digest(&header, 1)? != crate::archive_worker_contract_sha256()
    {
        return Err(SemanticArchiveError::ReceiptConflict);
    }
    let rows=client.query("SELECT member.subject_kind,member.subject_id,member.grant_key, \
        grant_row.granted_tick,grant_row.provenance_source_id,grant_row.provenance_locator \
        FROM babylon_meta.archive_tick_knowledge_member_v2 member JOIN babylon_meta.archive_knowledge_grant_v1 grant_row \
        USING(campaign_id,subject_kind,subject_id,grant_key) WHERE member.campaign_id=$1 AND member.resolve_tick=$2 \
        ORDER BY member.subject_kind,member.subject_id,member.grant_key LIMIT 65536", &[campaign.as_uuid(),&tick])
        .map_err(|error|database("read pinned Archive knowledge membership",&error))?;
    let count = usize::try_from(decode::<i32>(&header, 3)?)
        .map_err(|_| SemanticArchiveError::StoredPageMismatch)?;
    if count > 65535 || count != rows.len() {
        return Err(SemanticArchiveError::StoredPageMismatch);
    }
    let grants = rows
        .iter()
        .map(|row| {
            let granted_tick = unsigned(decode(row, 3)?)?;
            if granted_tick > scope.tick() {
                return Err(SemanticArchiveError::StoredPageMismatch);
            }
            ArchiveKnowledgeGrant::try_new(
                ArchivePageRef::try_new(
                    decode_subject_kind(&decode::<String>(row, 0)?)?,
                    decode(row, 1)?,
                )?,
                decode(row, 2)?,
                granted_tick,
                ArchiveCitation::try_new(decode(row, 4)?, decode(row, 5)?)?,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let knowledge = ArchiveKnowledge::try_new(grants)?;
    if knowledge.sha256() != decode_digest(&header, 2)? {
        return Err(SemanticArchiveError::StoredPageMismatch);
    }
    Ok(knowledge)
}
