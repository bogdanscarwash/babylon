//! One role-confined snapshot for exact dossier and search observations.

use postgres::{GenericClient, IsolationLevel};

use super::record::RevisionRecord;
use super::storage::{self, signed, unsigned, ReadAuthority};
use super::{
    ArchiveDossierBounds, ArchiveDossierLink, ArchiveDossierPage, ArchiveDossierPending,
    ArchiveDossierRead, ArchiveDossierState, ArchiveDossierUnavailable, ArchiveLinkedPageState,
    ArchiveReadScope, ArchiveSearchHit, ArchiveSearchRead, ArchiveSearchState,
};
use crate::archive::{database, decode, decode_digest, decode_subject_kind, validate_text};
use crate::{
    ArchivePageRef, SemanticArchiveError, SemanticArchiveReader, SemanticArchiveReaderError,
};

#[derive(Clone, Debug)]
pub(super) struct ReadStatus {
    pub durable: u64,
    pub processed: u64,
    pub pending: Option<ArchiveDossierPending>,
}

impl SemanticArchiveReader {
    /// Read one exact retained dossier through the sole confined revision path.
    ///
    /// # Errors
    /// Refuses mismatched markers, corrupt retained bytes or membership, invalid
    /// cursors, reader privilege drift, and database failures.
    pub fn dossier_as_of(
        &self,
        scope: &ArchiveReadScope,
        subject: &ArchivePageRef,
        bounds: &ArchiveDossierBounds,
    ) -> Result<ArchiveDossierRead, SemanticArchiveReaderError> {
        let mut client = self.connect("connect scoped Archive dossier")?;
        let mut tx = client
            .build_transaction()
            .isolation_level(IsolationLevel::RepeatableRead)
            .read_only(true)
            .start()
            .map_err(|error| boundary(database("begin scoped Archive dossier", &error)))?;
        let status = read_status(&mut tx, scope).map_err(boundary)?;
        let state = dossier_state(&mut tx, scope, subject, bounds, &status).map_err(boundary)?;
        tx.commit()
            .map_err(|error| boundary(database("commit scoped Archive dossier read", &error)))?;
        Ok(ArchiveDossierRead {
            scope: scope.clone(),
            subject: subject.clone(),
            durable_tick: status.durable,
            processed_tick: status.processed,
            state,
        })
    }

    /// Search only the latest retained composition eligible at the exact scope.
    ///
    /// # Errors
    /// Refuses invalid bounds/text, marker or payload drift, writer credentials,
    /// and database failures. Pending results never claim complete coverage.
    pub fn search_as_of(
        &self,
        scope: &ArchiveReadScope,
        query: &str,
        limit: u32,
    ) -> Result<ArchiveSearchRead, SemanticArchiveReaderError> {
        if !(1..=100).contains(&limit) {
            return Err(boundary(SemanticArchiveError::CollectionBound));
        }
        if query.trim().is_empty() {
            return Err(boundary(SemanticArchiveError::InvalidText));
        }
        validate_text(query).map_err(boundary)?;
        let mut client = self.connect("connect scoped Archive search")?;
        let mut tx = client
            .build_transaction()
            .isolation_level(IsolationLevel::RepeatableRead)
            .read_only(true)
            .start()
            .map_err(|error| boundary(database("begin scoped Archive search", &error)))?;
        let status = read_status(&mut tx, scope).map_err(boundary)?;
        let mut result = ArchiveSearchRead {
            scope: scope.clone(),
            durable_tick: status.durable,
            processed_tick: status.processed,
            state: base_search_state(scope, &status),
            hits: Vec::new(),
            truncated: false,
        };
        if !matches!(result.state, ArchiveSearchState::Unavailable(_)) {
            search_hits(&mut tx, scope, query, limit, &mut result).map_err(boundary)?;
        }
        tx.commit()
            .map_err(|error| boundary(database("commit scoped Archive search read", &error)))?;
        Ok(result)
    }
}

fn boundary(error: SemanticArchiveError) -> SemanticArchiveReaderError {
    SemanticArchiveReaderError::Archive(error)
}

pub(super) fn read_status(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScope,
) -> Result<ReadStatus, SemanticArchiveError> {
    let campaign = scope.campaign_id();
    verify_marker(client, scope)?;
    let row = client
        .query_opt(
            "SELECT durable_tick,processed_tick \
        FROM public.v_archive_verification_v1 WHERE campaign_id=$1",
            &[campaign.as_uuid()],
        )
        .map_err(|error| database("read scoped Archive progress", &error))?
        .ok_or(SemanticArchiveError::StoredPageMismatch)?;
    let durable = unsigned(decode(&row, 0)?)?;
    let processed = unsigned(decode(&row, 1)?)?;
    if processed > durable || scope.tick() > durable {
        return Err(SemanticArchiveError::StoredPageMismatch);
    }
    let pending = (processed < scope.tick()).then_some(ArchiveDossierPending::ReceiptProcessing);
    let mut status = ReadStatus {
        durable,
        processed,
        pending,
    };
    if scope.tick() > 0 {
        let pin = client
            .query_opt(
                "SELECT tick_content_hash, valid, late_grants, worker_contract_sha256 \
            FROM public.v_archive_tick_knowledge_v2 WHERE campaign_id=$1 AND resolve_tick=$2",
                &[campaign.as_uuid(), &signed(scope.tick())?],
            )
            .map_err(|error| database("read scoped pinned Archive knowledge", &error))?;
        if let Some(pin) = pin {
            if Some(decode_digest(&pin, 0)?) != scope.tick_content_hash()
                || !decode::<bool>(&pin, 1)?
                || decode_digest(&pin, 3)? != crate::archive_worker_contract_sha256()
            {
                return Err(SemanticArchiveError::StoredPageMismatch);
            }
            if status.pending.is_none() && scope.tick() == durable && decode::<bool>(&pin, 2)? {
                status.pending = Some(ArchiveDossierPending::KnowledgeRefresh);
            }
        } else if status.pending.is_none() {
            return Err(SemanticArchiveError::StoredPageMismatch);
        }
    }
    Ok(status)
}

fn verify_marker(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScope,
) -> Result<(), SemanticArchiveError> {
    if scope.tick() == 0 {
        return Ok(());
    }
    let campaign = scope.campaign_id();
    let row = client
        .query_opt(
            "SELECT tick_content_hash FROM public.v_committed_tick_status_v1 \
        WHERE campaign_id=$1 AND resolve_tick=$2",
            &[campaign.as_uuid(), &signed(scope.tick())?],
        )
        .map_err(|error| database("verify exact Archive read marker", &error))?
        .ok_or(SemanticArchiveError::MissingCommittedReceipt)?;
    if Some(decode_digest(&row, 0)?) != scope.tick_content_hash() {
        return Err(SemanticArchiveError::ReceiptMismatch);
    }
    Ok(())
}

fn unavailable(scope: &ArchiveReadScope) -> Option<ArchiveDossierUnavailable> {
    if scope.tick() == 0 {
        Some(ArchiveDossierUnavailable::FoundationHasNoPage)
    } else {
        None
    }
}

fn base_search_state(scope: &ArchiveReadScope, status: &ReadStatus) -> ArchiveSearchState {
    if let Some(reason) = unavailable(scope) {
        ArchiveSearchState::Unavailable(reason)
    } else if let Some(reason) = status.pending {
        ArchiveSearchState::Pending(reason)
    } else {
        ArchiveSearchState::Ready
    }
}

fn dossier_state(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScope,
    subject: &ArchivePageRef,
    bounds: &ArchiveDossierBounds,
    status: &ReadStatus,
) -> Result<ArchiveDossierState, SemanticArchiveError> {
    if let Some(reason) = unavailable(scope) {
        return Ok(ArchiveDossierState::Unavailable(reason));
    }
    if !subject_granted(client, scope, subject)? {
        return Ok(ArchiveDossierState::Unavailable(
            ArchiveDossierUnavailable::SubjectNotDisclosed,
        ));
    }
    let Some(candidate) = candidate(client, scope, subject)? else {
        return Ok(status.pending.map_or(
            ArchiveDossierState::Unavailable(ArchiveDossierUnavailable::PageNotMaterialized),
            |reason| ArchiveDossierState::Pending { page: None, reason },
        ));
    };
    let record = load_candidate(client, scope, subject, &candidate)?;
    let page = page(client, scope, &record, bounds, status)?;
    Ok(if let Some(reason) = status.pending {
        ArchiveDossierState::Pending {
            page: Some(page),
            reason,
        }
    } else {
        ArchiveDossierState::Ready {
            page,
            verified_through_tick: scope.tick(),
        }
    })
}

pub(super) fn subject_granted(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScope,
    subject: &ArchivePageRef,
) -> Result<bool, SemanticArchiveError> {
    let campaign = scope.campaign_id();
    let row = client
        .query_one(
            "SELECT CASE WHEN EXISTS(SELECT 1 FROM public.v_archive_tick_knowledge_v2 WHERE campaign_id=$1 AND resolve_tick=$4) \
        THEN EXISTS(SELECT 1 FROM public.v_archive_subject_grant_v2 WHERE campaign_id=$1 \
            AND subject_kind=$2 AND subject_id=$3 AND resolve_tick=$4) \
        ELSE EXISTS(SELECT 1 FROM public.v_archive_revision_index_v2 WHERE campaign_id=$1 \
            AND subject_kind=$2 AND subject_id=$3 AND effective_tick<=$4) \
        OR EXISTS(SELECT 1 FROM public.v_archive_revision_grant_v2 WHERE campaign_id=$1 \
            AND grant_subject_kind=$2 AND grant_subject_id=$3 AND grant_key='subject' AND effective_tick<=$4) END",
            &[
                campaign.as_uuid(),
                &subject.kind().as_str(),
                &subject.id(),
                &signed(scope.tick())?,
            ],
        )
        .map_err(|error| database("read scoped Archive subject disclosure", &error))?;
    decode(&row, 0)
}

pub(super) struct Candidate {
    pub tick: u64,
    pub digest: [u8; 32],
}

pub(super) fn candidate(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScope,
    subject: &ArchivePageRef,
) -> Result<Option<Candidate>, SemanticArchiveError> {
    let campaign = scope.campaign_id();
    client
        .query_opt(
            "SELECT effective_tick,revision_sha256 \
        FROM public.v_archive_revision_index_v2 WHERE campaign_id=$1 AND subject_kind=$2 \
        AND subject_id=$3 AND effective_tick<=$4 ORDER BY effective_tick DESC LIMIT 1",
            &[
                campaign.as_uuid(),
                &subject.kind().as_str(),
                &subject.id(),
                &signed(scope.tick())?,
            ],
        )
        .map_err(|error| database("select exact retained Archive candidate", &error))?
        .map(|row| {
            Ok(Candidate {
                tick: unsigned(decode(&row, 0)?)?,
                digest: decode_digest(&row, 1)?,
            })
        })
        .transpose()
}

pub(super) fn load_candidate(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScope,
    subject: &ArchivePageRef,
    candidate: &Candidate,
) -> Result<RevisionRecord, SemanticArchiveError> {
    let campaign = scope.campaign_id();
    let row=client.query_opt(&format!("SELECT {} FROM public.v_archive_revision_known_v2 \
        WHERE campaign_id=$1 AND subject_kind=$2 AND subject_id=$3 AND effective_tick=$4 AND EXISTS(SELECT 1 FROM public.v_archive_revision_scope_v2 admitted \
            WHERE admitted.campaign_id=$1 AND admitted.subject_kind=$2 AND admitted.subject_id=$3 \
            AND admitted.effective_tick=$4 AND admitted.observation_tick=$5)",storage::COLUMNS),
        &[campaign.as_uuid(),&subject.kind().as_str(),&subject.id(),&signed(candidate.tick)?,&signed(scope.tick())?])
        .map_err(|error| database("read exact known Archive publication",&error))?
        .ok_or(SemanticArchiveError::StoredPageMismatch)?;
    let record = storage::decode_record(client, &row, ReadAuthority::Confined)?;
    if record.digest()? != candidate.digest
        || record.source.campaign_id() != campaign
        || &record.subject != subject
        || record.effective_tick > scope.tick()
    {
        return Err(SemanticArchiveError::StoredPageMismatch);
    }
    Ok(record)
}

fn page(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScope,
    record: &RevisionRecord,
    bounds: &ArchiveDossierBounds,
    status: &ReadStatus,
) -> Result<ArchiveDossierPage, SemanticArchiveError> {
    let manifest = &record.emission;
    let links = manifest
        .links()
        .iter()
        .map(|link| {
            let target_state = link_state(client, scope, link.target(), status)?;
            Ok(ArchiveDossierLink {
                target: link.target().clone(),
                retained_label: link.known_label().map(str::to_owned),
                target_state,
            })
        })
        .collect::<Result<Vec<_>, SemanticArchiveError>>()?;
    let changes = super::read_history::read(client, scope, record, bounds, status)?;
    Ok(ArchiveDossierPage {
        revision_id: record.digest()?,
        effective_tick: record.effective_tick,
        content_source: record.source.clone(),
        title: record.title.clone(),
        question: manifest.question().to_owned(),
        signals: manifest.signals().to_vec(),
        markdown: record.markdown.clone(),
        content_sha256: record.content_sha256,
        citations: manifest.citations(),
        atoms: record.atoms.clone(),
        links,
        changes,
    })
}

fn link_state(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScope,
    subject: &ArchivePageRef,
    status: &ReadStatus,
) -> Result<ArchiveLinkedPageState, SemanticArchiveError> {
    if !subject_granted(client, scope, subject)? {
        return Ok(ArchiveLinkedPageState::Unknown);
    }
    let Some(candidate) = candidate(client, scope, subject)? else {
        return Ok(if status.pending.is_some() {
            ArchiveLinkedPageState::KnownPending
        } else {
            ArchiveLinkedPageState::KnownUnavailable
        });
    };
    load_candidate(client, scope, subject, &candidate)?;
    Ok(if status.pending.is_some() {
        ArchiveLinkedPageState::KnownPending
    } else {
        ArchiveLinkedPageState::KnownReady
    })
}

fn search_hits(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScope,
    query: &str,
    limit: u32,
    result: &mut ArchiveSearchRead,
) -> Result<(), SemanticArchiveError> {
    let campaign = scope.campaign_id();
    // Latest identity is chosen before payload eligibility or text matching.
    let latest="SELECT DISTINCT ON(subject_kind,subject_id) * FROM public.v_archive_revision_index_v2 \
        WHERE campaign_id=$1 AND effective_tick<=$2 AND ( \
        EXISTS(SELECT 1 FROM public.v_archive_subject_grant_v2 known WHERE known.campaign_id=$1 \
            AND known.resolve_tick=$2 AND known.subject_kind=v_archive_revision_index_v2.subject_kind \
            AND known.subject_id=v_archive_revision_index_v2.subject_id) \
        OR (NOT EXISTS(SELECT 1 FROM public.v_archive_tick_knowledge_v2 \
            WHERE campaign_id=$1 AND resolve_tick=$2))) ORDER BY subject_kind,subject_id,effective_tick DESC";
    // The security-barrier view validates complete grant and atom membership.
    // Evaluate that scoped set once, even before a fresh campaign has planner
    // statistics; a nested loop must not repeat every revision's validation
    // for each latest subject. The existing view remains the authority.
    let scoped = format!(
        "WITH latest AS ({latest}), eligible AS MATERIALIZED (\
         SELECT campaign_id,subject_kind,subject_id,effective_tick,revision_sha256,search_text \
         FROM public.v_archive_revision_known_v2 WHERE campaign_id=$1 AND effective_tick<=$2)"
    );
    let integrity = client
        .query_one(
            &format!(
                "{scoped} SELECT \
        COALESCE(bool_or(page.campaign_id IS NULL),FALSE) \
        FROM latest LEFT JOIN eligible page \
        USING(campaign_id,subject_kind,subject_id,effective_tick)"
            ),
            &[campaign.as_uuid(), &signed(scope.tick())?],
        )
        .map_err(|error| database("validate scoped Archive search eligibility", &error))?;
    if decode::<bool>(&integrity, 0)? {
        return Err(SemanticArchiveError::StoredPageMismatch);
    }
    let rows = client
        .query(
            &format!(
                "{scoped} SELECT page.subject_kind,page.subject_id, \
        page.effective_tick,page.revision_sha256 FROM latest JOIN eligible page \
        USING(campaign_id,subject_kind,subject_id,effective_tick) \
        WHERE pg_catalog.strpos(pg_catalog.lower(page.search_text),pg_catalog.lower($3))>0 \
        ORDER BY page.subject_kind,page.subject_id LIMIT $4"
            ),
            &[
                campaign.as_uuid(),
                &signed(scope.tick())?,
                &query,
                &(i64::from(limit) + 1),
            ],
        )
        .map_err(|error| database("search scoped retained Archive text", &error))?;
    result.truncated =
        rows.len() > usize::try_from(limit).map_err(|_| SemanticArchiveError::CollectionBound)?;
    for row in rows
        .into_iter()
        .take(usize::try_from(limit).map_err(|_| SemanticArchiveError::CollectionBound)?)
    {
        let subject = ArchivePageRef::try_new(
            decode_subject_kind(&decode::<String>(&row, 0)?)?,
            decode(&row, 1)?,
        )?;
        let candidate = Candidate {
            tick: unsigned(decode(&row, 2)?)?,
            digest: decode_digest(&row, 3)?,
        };
        let record = load_candidate(client, scope, &subject, &candidate)?;
        result.hits.push(ArchiveSearchHit {
            subject,
            revision_id: candidate.digest,
            title: record.title,
            content_source: record.source,
        });
    }
    Ok(())
}
