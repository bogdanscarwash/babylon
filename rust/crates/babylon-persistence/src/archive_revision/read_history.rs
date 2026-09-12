//! Bounded retained-composition history. Cursors never mix pending snapshots.

use postgres::GenericClient;
use sha2::{Digest as _, Sha256};

use super::read::{Candidate, ReadStatus};
use super::record::RevisionRecord;
use super::storage::{signed, unsigned};
use super::{ArchiveChangeCursor, ArchiveChangePage, ArchiveDossierBounds, ArchiveReadScope};
use crate::archive::{database, decode, decode_digest};
use crate::SemanticArchiveError;

const SCAN_LIMIT: i64 = 16;

pub(super) fn read(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScope,
    head: &RevisionRecord,
    bounds: &ArchiveDossierBounds,
    status: &ReadStatus,
) -> Result<ArchiveChangePage, SemanticArchiveError> {
    let mut result = ArchiveChangePage {
        coverage_from_tick: 0,
        changes: Vec::new(),
        next_cursor: None,
    };
    if status.pending.is_some() {
        if bounds.change_cursor.is_some() {
            return Err(SemanticArchiveError::ArchiveCursorMismatch);
        }
        return Ok(result);
    }
    let digest = history_identity(scope, head)?;
    let (start_tick, mut offset) = match &bounds.change_cursor {
        None => (0, 0usize),
        Some(cursor)
            if &cursor.scope == scope
                && cursor.subject == head.subject
                && cursor.history_digest == digest
                && cursor.publication_tick <= scope.tick() =>
        {
            (
                cursor.publication_tick,
                usize::try_from(cursor.change_offset)
                    .map_err(|_| SemanticArchiveError::ArchiveCursorMismatch)?,
            )
        }
        Some(_) => return Err(SemanticArchiveError::ArchiveCursorMismatch),
    };
    let campaign = scope.campaign_id();
    let params: &[&(dyn postgres::types::ToSql + Sync)] = &[
        campaign.as_uuid(),
        &head.subject.kind().as_str(),
        &head.subject.id(),
        &signed(start_tick)?,
        &signed(scope.tick())?,
    ];
    let rows=client.query("SELECT effective_tick,revision_sha256 \
        FROM public.v_archive_revision_index_v2 WHERE campaign_id=$1 AND subject_kind=$2 AND subject_id=$3 \
        AND effective_tick>=$4 AND effective_tick<=$5 \
        ORDER BY effective_tick LIMIT 17",params)
        .map_err(|error| database("read bounded retained Archive history",&error))?;
    let previous=client.query_opt("SELECT effective_tick,revision_sha256 \
        FROM public.v_archive_revision_index_v2 WHERE campaign_id=$1 AND subject_kind=$2 AND subject_id=$3 \
        AND effective_tick<$4 \
        ORDER BY effective_tick DESC LIMIT 1",
        &[campaign.as_uuid(),&head.subject.kind().as_str(),&head.subject.id(),&signed(start_tick)?])
        .map_err(|error| database("read retained Archive history predecessor",&error))?;
    let mut previous = previous
        .map(|row| decode_candidate(&row))
        .transpose()?
        .map(|candidate| super::read::load_candidate(client, scope, &head.subject, &candidate))
        .transpose()?;
    for (index, row) in rows.iter().enumerate() {
        let candidate = decode_candidate(row)?;
        if index
            >= usize::try_from(SCAN_LIMIT).map_err(|_| SemanticArchiveError::CollectionBound)?
        {
            result.next_cursor = Some(cursor(scope, head, digest, &candidate, 0)?);
            break;
        }
        let current = super::read::load_candidate(client, scope, &head.subject, &candidate)?;
        let changes = super::changes::between(previous.as_ref(), &current)?;
        if offset > changes.len() {
            return Err(SemanticArchiveError::ArchiveCursorMismatch);
        }
        let available = usize::try_from(bounds.change_limit)
            .map_err(|_| SemanticArchiveError::CollectionBound)?
            .checked_sub(result.changes.len())
            .ok_or(SemanticArchiveError::CollectionBound)?;
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

fn decode_candidate(row: &postgres::Row) -> Result<Candidate, SemanticArchiveError> {
    Ok(Candidate {
        tick: unsigned(decode(row, 0)?)?,
        digest: decode_digest(row, 1)?,
    })
}

fn cursor(
    scope: &ArchiveReadScope,
    head: &RevisionRecord,
    digest: [u8; 32],
    candidate: &Candidate,
    offset: usize,
) -> Result<ArchiveChangeCursor, SemanticArchiveError> {
    Ok(ArchiveChangeCursor {
        scope: scope.clone(),
        subject: head.subject.clone(),
        history_digest: digest,
        publication_tick: candidate.tick,
        change_offset: u32::try_from(offset).map_err(|_| SemanticArchiveError::CollectionBound)?,
    })
}

// Once ordered processing covers T, the native publication prefix <=T is closed.
// The complete head identity binds this cursor; pending prefixes never yield one.
fn history_identity(
    scope: &ArchiveReadScope,
    head: &RevisionRecord,
) -> Result<[u8; 32], SemanticArchiveError> {
    let mut digest = Sha256::new();
    digest.update(b"babylon.archive-retained-history.v2\0");
    digest.update(scope.campaign_id().canonical_bytes());
    digest.update(scope.tick().to_be_bytes());
    digest.update(
        scope
            .tick_content_hash()
            .ok_or(SemanticArchiveError::InvalidVerifiedTick)?,
    );
    digest.update(head.digest()?);
    Ok(digest.finalize().into())
}
