//! Bounded retained-composition history. Cursors never mix pending snapshots.

use postgres::GenericClient;
use sha2::{Digest as _, Sha256};

use super::read::{Candidate, ReadStatus};
use super::record::RevisionRecord;
use super::storage::{signed, unsigned};
use super::{
    ArchiveChangeCursorV2, ArchiveChangePageV2, ArchiveDossierBoundsV2, ArchivePublicationOriginV2,
    ArchiveReadScopeV2,
};
use crate::archive::{database, decode, decode_digest};
use crate::SemanticArchiveErrorV1;

const SCAN_LIMIT: i64 = 16;

pub(super) fn read(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScopeV2,
    head: &RevisionRecord,
    bounds: &ArchiveDossierBoundsV2,
    status: &ReadStatus,
) -> Result<ArchiveChangePageV2, SemanticArchiveErrorV1> {
    let mut result = ArchiveChangePageV2 {
        coverage_from_tick: status.floor,
        changes: Vec::new(),
        next_cursor: None,
    };
    if status.pending.is_some() {
        if bounds.change_cursor.is_some() {
            return Err(SemanticArchiveErrorV1::ArchiveCursorMismatch);
        }
        return Ok(result);
    }
    let digest = history_identity(scope, head, status.floor)?;
    let (start_tick, start_origin, mut offset) = match &bounds.change_cursor {
        None => (status.floor, 0, 0usize),
        Some(cursor)
            if &cursor.scope == scope
                && cursor.subject == head.subject
                && cursor.history_digest == digest
                && cursor.publication_tick >= status.floor
                && cursor.publication_tick <= scope.tick()
                && matches!(cursor.publication_origin, 0 | 1) =>
        {
            (
                cursor.publication_tick,
                cursor.publication_origin,
                usize::try_from(cursor.change_offset)
                    .map_err(|_| SemanticArchiveErrorV1::ArchiveCursorMismatch)?,
            )
        }
        Some(_) => return Err(SemanticArchiveErrorV1::ArchiveCursorMismatch),
    };
    let campaign = scope.campaign_id();
    let params: &[&(dyn postgres::types::ToSql + Sync)] = &[
        campaign.as_uuid(),
        &head.subject.kind().as_str(),
        &head.subject.id(),
        &signed(start_tick)?,
        &start_origin,
        &signed(scope.tick())?,
    ];
    let rows=client.query("SELECT effective_tick,origin,revision_sha256,has_emission_witness \
        FROM public.v_archive_revision_index_v2 WHERE campaign_id=$1 AND subject_kind=$2 AND subject_id=$3 \
        AND (effective_tick,origin)>=($4,$5) AND effective_tick<=$6 \
        ORDER BY effective_tick,origin LIMIT 17",params)
        .map_err(|error| database("read bounded retained Archive history",&error))?;
    let previous=client.query_opt("SELECT effective_tick,origin,revision_sha256,has_emission_witness \
        FROM public.v_archive_revision_index_v2 WHERE campaign_id=$1 AND subject_kind=$2 AND subject_id=$3 \
        AND (effective_tick,origin)<($4,$5) AND effective_tick>=$6 \
        ORDER BY effective_tick DESC,origin DESC LIMIT 1",
        &[campaign.as_uuid(),&head.subject.kind().as_str(),&head.subject.id(),&signed(start_tick)?,&start_origin,&signed(status.floor)?])
        .map_err(|error| database("read retained Archive history predecessor",&error))?;
    let mut previous = previous
        .map(|row| decode_candidate(&row))
        .transpose()?
        .filter(|candidate| candidate.witness)
        .map(|candidate| super::read::load_candidate(client, scope, &head.subject, &candidate))
        .transpose()?;
    for (index, row) in rows.iter().enumerate() {
        let candidate = decode_candidate(row)?;
        if index
            >= usize::try_from(SCAN_LIMIT).map_err(|_| SemanticArchiveErrorV1::CollectionBound)?
        {
            result.next_cursor = Some(cursor(scope, head, digest, &candidate, 0)?);
            break;
        }
        if !candidate.witness {
            if offset != 0 {
                return Err(SemanticArchiveErrorV1::ArchiveCursorMismatch);
            }
            // Opaque retained evidence supplies no inferred earlier assertions.
            previous = None;
            continue;
        }
        let current = super::read::load_candidate(client, scope, &head.subject, &candidate)?;
        let changes =
            if previous.is_none() && current.origin == ArchivePublicationOriginV2::AdoptedHead {
                Vec::new()
            } else {
                super::changes::between(previous.as_ref(), &current)?
            };
        if offset > changes.len() {
            return Err(SemanticArchiveErrorV1::ArchiveCursorMismatch);
        }
        let available = usize::try_from(bounds.change_limit)
            .map_err(|_| SemanticArchiveErrorV1::CollectionBound)?
            .checked_sub(result.changes.len())
            .ok_or(SemanticArchiveErrorV1::CollectionBound)?;
        let taken = available.min(changes.len() - offset);
        result
            .changes
            .extend(changes.iter().skip(offset).take(taken).cloned());
        if offset + taken < changes.len() {
            result.next_cursor = Some(cursor(scope, head, digest, &candidate, offset + taken)?);
            break;
        }
        offset = 0;
        previous = Some(current);
    }
    Ok(result)
}

fn decode_candidate(row: &postgres::Row) -> Result<Candidate, SemanticArchiveErrorV1> {
    Ok(Candidate {
        tick: unsigned(decode(row, 0)?)?,
        origin: decode(row, 1)?,
        digest: decode_digest(row, 2)?,
        witness: decode(row, 3)?,
    })
}

fn cursor(
    scope: &ArchiveReadScopeV2,
    head: &RevisionRecord,
    digest: [u8; 32],
    candidate: &Candidate,
    offset: usize,
) -> Result<ArchiveChangeCursorV2, SemanticArchiveErrorV1> {
    Ok(ArchiveChangeCursorV2 {
        scope: scope.clone(),
        subject: head.subject.clone(),
        history_digest: digest,
        publication_tick: candidate.tick,
        publication_origin: candidate.origin,
        change_offset: u32::try_from(offset)
            .map_err(|_| SemanticArchiveErrorV1::CollectionBound)?,
    })
}

// Once ordered processing covers T, the native publication prefix <=T is closed.
// The complete head identity binds this cursor; pending prefixes never yield one.
fn history_identity(
    scope: &ArchiveReadScopeV2,
    head: &RevisionRecord,
    floor: u64,
) -> Result<[u8; 32], SemanticArchiveErrorV1> {
    let mut digest = Sha256::new();
    digest.update(b"babylon.archive-retained-history.v2\0");
    digest.update(scope.campaign_id().canonical_bytes());
    digest.update(scope.tick().to_be_bytes());
    digest.update(
        scope
            .tick_content_hash()
            .ok_or(SemanticArchiveErrorV1::InvalidVerifiedTick)?,
    );
    digest.update(floor.to_be_bytes());
    digest.update(head.digest()?);
    Ok(digest.finalize().into())
}
