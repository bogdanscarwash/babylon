//! One role-confined snapshot for exact dossier and search observations.

use postgres::{GenericClient, IsolationLevel};

use super::enrollment::AdoptionSeed;
use super::record::RevisionRecord;
use super::storage::{self, signed, unsigned, ReadAuthority};
use super::{
    ArchiveDossierBoundsV2, ArchiveDossierLinkV2, ArchiveDossierPageV2, ArchiveDossierPendingV2,
    ArchiveDossierReadV2, ArchiveDossierStateV2, ArchiveDossierUnavailableV2,
    ArchiveLinkedPageStateV2, ArchiveReadScopeV2, ArchiveSearchHitV2, ArchiveSearchReadV2,
    ArchiveSearchStateV2,
};
use crate::archive::{database, decode, decode_digest, decode_subject_kind, validate_text};
use crate::{
    ArchivePageRefV1, SemanticArchiveErrorV1, SemanticArchiveReaderErrorV1, SemanticArchiveReaderV1,
};

#[derive(Clone, Debug)]
pub(super) struct ReadStatus {
    pub durable: u64,
    pub processed: u64,
    pub floor: u64,
    pub pending: Option<ArchiveDossierPendingV2>,
}

impl SemanticArchiveReaderV1 {
    /// Read one exact retained dossier through the sole confined revision path.
    ///
    /// # Errors
    /// Refuses mismatched markers, corrupt retained bytes or membership, invalid
    /// cursors, reader privilege drift, and database failures.
    pub fn dossier_as_of(
        &self,
        scope: &ArchiveReadScopeV2,
        subject: &ArchivePageRefV1,
        bounds: &ArchiveDossierBoundsV2,
    ) -> Result<ArchiveDossierReadV2, SemanticArchiveReaderErrorV1> {
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
        Ok(ArchiveDossierReadV2 {
            scope: scope.clone(),
            subject: subject.clone(),
            durable_tick: status.durable,
            processed_tick: status.processed,
            history_floor_tick: status.floor,
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
        scope: &ArchiveReadScopeV2,
        query: &str,
        limit: u32,
    ) -> Result<ArchiveSearchReadV2, SemanticArchiveReaderErrorV1> {
        if !(1..=100).contains(&limit) {
            return Err(boundary(SemanticArchiveErrorV1::CollectionBound));
        }
        if query.trim().is_empty() {
            return Err(boundary(SemanticArchiveErrorV1::InvalidText));
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
        let mut result = ArchiveSearchReadV2 {
            scope: scope.clone(),
            durable_tick: status.durable,
            processed_tick: status.processed,
            history_floor_tick: status.floor,
            state: base_search_state(scope, &status),
            hits: Vec::new(),
            truncated: false,
        };
        if !matches!(result.state, ArchiveSearchStateV2::Unavailable(_)) {
            search_hits(&mut tx, scope, query, limit, &mut result).map_err(boundary)?;
        }
        tx.commit()
            .map_err(|error| boundary(database("commit scoped Archive search read", &error)))?;
        Ok(result)
    }
}

fn boundary(error: SemanticArchiveErrorV1) -> SemanticArchiveReaderErrorV1 {
    SemanticArchiveReaderErrorV1::Archive(error)
}

pub(super) fn read_status(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScopeV2,
) -> Result<ReadStatus, SemanticArchiveErrorV1> {
    let campaign = scope.campaign_id();
    verify_marker(client, scope)?;
    let row = client.query_opt("SELECT floor_tick,floor_content_hash,processed_at_adoption, \
        adopted_page_count,adopted_heads_sha256,adoption_sha256,sealed, \
        COALESCE((SELECT durable_tick FROM public.v_archive_verification_v1 WHERE campaign_id=$1),0), \
        COALESCE((SELECT processed_tick FROM public.v_archive_verification_v1 WHERE campaign_id=$1),0), \
        seal_present,seal_valid,seal_worker_contract_sha256 FROM public.v_archive_retention_v2 WHERE campaign_id=$1", &[campaign.as_uuid()])
        .map_err(|error| database("read scoped Archive retention", &error))?
        .ok_or(SemanticArchiveErrorV1::StoredPageMismatch)?;
    super::publication::validate_seal(&row, 9)?;
    let floor_tick = unsigned(decode(&row, 0)?)?;
    let floor = match (floor_tick, decode::<Option<Vec<u8>>>(&row, 1)?) {
        (0, None) => ArchiveReadScopeV2::foundation(campaign),
        (0, Some(_)) | (_, None) => return Err(SemanticArchiveErrorV1::StoredPageMismatch),
        (tick, Some(hash)) => ArchiveReadScopeV2::committed(
            campaign,
            tick,
            hash.try_into()
                .map_err(|_| SemanticArchiveErrorV1::StoredPageMismatch)?,
        )?,
    };
    verify_marker(client, &floor)?;
    let seed = AdoptionSeed {
        floor,
        processed: unsigned(decode(&row, 2)?)?,
        count: unsigned(decode(&row, 3)?)?,
        heads_digest: decode_digest(&row, 4)?,
    };
    if seed.digest()? != decode_digest(&row, 5)? {
        return Err(SemanticArchiveErrorV1::StoredPageMismatch);
    }
    let durable = unsigned(decode(&row, 7)?)?;
    let processed = unsigned(decode(&row, 8)?)?;
    if processed > durable || scope.tick() > durable || floor_tick > durable {
        return Err(SemanticArchiveErrorV1::StoredPageMismatch);
    }
    let pending = if !decode::<bool>(&row, 6)? {
        Some(ArchiveDossierPendingV2::CutoverValidation)
    } else if processed < scope.tick() {
        Some(ArchiveDossierPendingV2::ReceiptProcessing)
    } else {
        None
    };
    let mut status = ReadStatus {
        durable,
        processed,
        floor: floor_tick,
        pending,
    };
    if scope.tick() > 0 && scope.tick() >= floor_tick {
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
                || decode_digest(&pin, 3)? != super::publication::worker_contract()
            {
                return Err(SemanticArchiveErrorV1::StoredPageMismatch);
            }
            if status.pending.is_none() && scope.tick() == durable && decode::<bool>(&pin, 2)? {
                status.pending = Some(ArchiveDossierPendingV2::KnowledgeRefresh);
            }
        } else if status.pending.is_none() {
            return Err(SemanticArchiveErrorV1::StoredPageMismatch);
        }
    }
    Ok(status)
}

fn verify_marker(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScopeV2,
) -> Result<(), SemanticArchiveErrorV1> {
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
        .ok_or(SemanticArchiveErrorV1::MissingCommittedReceipt)?;
    if Some(decode_digest(&row, 0)?) != scope.tick_content_hash() {
        return Err(SemanticArchiveErrorV1::ReceiptMismatch);
    }
    Ok(())
}

fn unavailable(
    scope: &ArchiveReadScopeV2,
    status: &ReadStatus,
) -> Option<ArchiveDossierUnavailableV2> {
    if scope.tick() == 0 {
        Some(ArchiveDossierUnavailableV2::FoundationHasNoPage)
    } else if scope.tick() < status.floor {
        Some(ArchiveDossierUnavailableV2::HistoryNotRetained)
    } else {
        None
    }
}

fn base_search_state(scope: &ArchiveReadScopeV2, status: &ReadStatus) -> ArchiveSearchStateV2 {
    if let Some(reason) = unavailable(scope, status) {
        ArchiveSearchStateV2::Unavailable(reason)
    } else if let Some(reason) = status.pending {
        ArchiveSearchStateV2::Pending(reason)
    } else {
        ArchiveSearchStateV2::Ready
    }
}

fn dossier_state(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScopeV2,
    subject: &ArchivePageRefV1,
    bounds: &ArchiveDossierBoundsV2,
    status: &ReadStatus,
) -> Result<ArchiveDossierStateV2, SemanticArchiveErrorV1> {
    if let Some(reason) = unavailable(scope, status) {
        return Ok(ArchiveDossierStateV2::Unavailable(reason));
    }
    if !subject_granted(client, scope, subject)? {
        return Ok(ArchiveDossierStateV2::Unavailable(
            ArchiveDossierUnavailableV2::SubjectNotDisclosed,
        ));
    }
    let Some(candidate) = candidate(client, scope, subject)? else {
        return Ok(status.pending.map_or(
            ArchiveDossierStateV2::Unavailable(ArchiveDossierUnavailableV2::PageNotMaterialized),
            |reason| ArchiveDossierStateV2::Pending { page: None, reason },
        ));
    };
    if !candidate.witness {
        return Ok(ArchiveDossierStateV2::Pending {
            page: None,
            reason: ArchiveDossierPendingV2::EmissionWitnessRequired,
        });
    }
    let record = load_candidate(client, scope, subject, &candidate)?;
    let page = page(client, scope, &record, bounds, status)?;
    Ok(if let Some(reason) = status.pending {
        ArchiveDossierStateV2::Pending {
            page: Some(page),
            reason,
        }
    } else {
        ArchiveDossierStateV2::Ready {
            page,
            verified_through_tick: scope.tick(),
        }
    })
}

pub(super) fn subject_granted(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScopeV2,
    subject: &ArchivePageRefV1,
) -> Result<bool, SemanticArchiveErrorV1> {
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
    pub origin: i16,
    pub digest: [u8; 32],
    pub witness: bool,
}

pub(super) fn candidate(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScopeV2,
    subject: &ArchivePageRefV1,
) -> Result<Option<Candidate>, SemanticArchiveErrorV1> {
    let campaign = scope.campaign_id();
    client
        .query_opt(
            "SELECT effective_tick,origin,revision_sha256,has_emission_witness \
        FROM public.v_archive_revision_index_v2 WHERE campaign_id=$1 AND subject_kind=$2 \
        AND subject_id=$3 AND effective_tick<=$4 ORDER BY effective_tick DESC,origin DESC LIMIT 1",
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
                origin: decode(&row, 1)?,
                digest: decode_digest(&row, 2)?,
                witness: decode(&row, 3)?,
            })
        })
        .transpose()
}

pub(super) fn load_candidate(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScopeV2,
    subject: &ArchivePageRefV1,
    candidate: &Candidate,
) -> Result<RevisionRecord, SemanticArchiveErrorV1> {
    let campaign = scope.campaign_id();
    let row=client.query_opt(&format!("SELECT {} FROM public.v_archive_revision_known_v2 \
        WHERE campaign_id=$1 AND subject_kind=$2 AND subject_id=$3 AND effective_tick=$4 AND origin=$5 AND EXISTS(SELECT 1 FROM public.v_archive_revision_scope_v2 admitted \
            WHERE admitted.campaign_id=$1 AND admitted.subject_kind=$2 AND admitted.subject_id=$3 \
            AND admitted.effective_tick=$4 AND admitted.origin=$5 AND admitted.observation_tick=$6)",storage::COLUMNS),
        &[campaign.as_uuid(),&subject.kind().as_str(),&subject.id(),&signed(candidate.tick)?,&candidate.origin,&signed(scope.tick())?])
        .map_err(|error| database("read exact known Archive publication",&error))?
        .ok_or(SemanticArchiveErrorV1::StoredPageMismatch)?;
    let record = storage::decode_record(client, &row, ReadAuthority::Confined)?;
    if record.digest()? != candidate.digest
        || record.source.campaign_id() != campaign
        || &record.subject != subject
        || record.effective_tick > scope.tick()
    {
        return Err(SemanticArchiveErrorV1::StoredPageMismatch);
    }
    Ok(record)
}

fn page(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScopeV2,
    record: &RevisionRecord,
    bounds: &ArchiveDossierBoundsV2,
    status: &ReadStatus,
) -> Result<ArchiveDossierPageV2, SemanticArchiveErrorV1> {
    let manifest = record
        .emission
        .as_ref()
        .ok_or(SemanticArchiveErrorV1::StoredPageMismatch)?;
    let links = manifest
        .links()
        .iter()
        .map(|link| {
            let target_state = link_state(client, scope, link.target(), status)?;
            Ok(ArchiveDossierLinkV2 {
                target: link.target().clone(),
                retained_label: link.known_label().map(str::to_owned),
                target_state,
            })
        })
        .collect::<Result<Vec<_>, SemanticArchiveErrorV1>>()?;
    let changes = super::read_history::read(client, scope, record, bounds, status)?;
    Ok(ArchiveDossierPageV2 {
        revision_id: record.digest()?,
        effective_tick: record.effective_tick,
        origin: record.origin,
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
    scope: &ArchiveReadScopeV2,
    subject: &ArchivePageRefV1,
    status: &ReadStatus,
) -> Result<ArchiveLinkedPageStateV2, SemanticArchiveErrorV1> {
    if !subject_granted(client, scope, subject)? {
        return Ok(ArchiveLinkedPageStateV2::Unknown);
    }
    let Some(candidate) = candidate(client, scope, subject)? else {
        return Ok(if status.pending.is_some() {
            ArchiveLinkedPageStateV2::KnownPending
        } else {
            ArchiveLinkedPageStateV2::KnownUnavailable
        });
    };
    if !candidate.witness {
        return Ok(ArchiveLinkedPageStateV2::KnownPending);
    }
    load_candidate(client, scope, subject, &candidate)?;
    Ok(if status.pending.is_some() {
        ArchiveLinkedPageStateV2::KnownPending
    } else {
        ArchiveLinkedPageStateV2::KnownReady
    })
}

fn search_hits(
    client: &mut impl GenericClient,
    scope: &ArchiveReadScopeV2,
    query: &str,
    limit: u32,
    result: &mut ArchiveSearchReadV2,
) -> Result<(), SemanticArchiveErrorV1> {
    let campaign = scope.campaign_id();
    // Latest identity is chosen before payload eligibility or text matching.
    let latest="SELECT DISTINCT ON(subject_kind,subject_id) * FROM public.v_archive_revision_index_v2 \
        WHERE campaign_id=$1 AND effective_tick<=$2 AND ( \
        EXISTS(SELECT 1 FROM public.v_archive_subject_grant_v2 known WHERE known.campaign_id=$1 \
            AND known.resolve_tick=$2 AND known.subject_kind=v_archive_revision_index_v2.subject_kind \
            AND known.subject_id=v_archive_revision_index_v2.subject_id) \
        OR (NOT EXISTS(SELECT 1 FROM public.v_archive_tick_knowledge_v2 \
            WHERE campaign_id=$1 AND resolve_tick=$2))) ORDER BY subject_kind,subject_id,effective_tick DESC,origin DESC";
    // The security-barrier view validates complete grant and atom membership.
    // Evaluate that scoped set once, even before a fresh campaign has planner
    // statistics; a nested loop must not repeat every revision's validation
    // for each latest subject. The existing view remains the authority.
    let scoped = format!(
        "WITH latest AS ({latest}), eligible AS MATERIALIZED (\
         SELECT campaign_id,subject_kind,subject_id,effective_tick,origin,revision_sha256,search_text \
         FROM public.v_archive_revision_known_v2 WHERE campaign_id=$1 AND effective_tick<=$2)"
    );
    let integrity = client
        .query_one(
            &format!(
                "{scoped} SELECT \
        COALESCE(bool_or(NOT latest.has_emission_witness),FALSE), \
        COALESCE(bool_or(latest.has_emission_witness AND page.campaign_id IS NULL),FALSE) \
        FROM latest LEFT JOIN eligible page \
        USING(campaign_id,subject_kind,subject_id,effective_tick,origin)"
            ),
            &[campaign.as_uuid(), &signed(scope.tick())?],
        )
        .map_err(|error| database("validate scoped Archive search eligibility", &error))?;
    if decode::<bool>(&integrity, 1)? {
        return Err(SemanticArchiveErrorV1::StoredPageMismatch);
    }
    if decode::<bool>(&integrity, 0)? {
        result.state =
            ArchiveSearchStateV2::Pending(ArchiveDossierPendingV2::EmissionWitnessRequired);
    }
    let rows = client
        .query(
            &format!(
                "{scoped} SELECT page.subject_kind,page.subject_id, \
        page.effective_tick,page.origin,page.revision_sha256 FROM latest JOIN eligible page \
        USING(campaign_id,subject_kind,subject_id,effective_tick,origin) \
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
        rows.len() > usize::try_from(limit).map_err(|_| SemanticArchiveErrorV1::CollectionBound)?;
    for row in rows
        .into_iter()
        .take(usize::try_from(limit).map_err(|_| SemanticArchiveErrorV1::CollectionBound)?)
    {
        let subject = ArchivePageRefV1::try_new(
            decode_subject_kind(&decode::<String>(&row, 0)?)?,
            decode(&row, 1)?,
        )?;
        let candidate = Candidate {
            tick: unsigned(decode(&row, 2)?)?,
            origin: decode(&row, 3)?,
            digest: decode_digest(&row, 4)?,
            witness: true,
        };
        let record = load_candidate(client, scope, &subject, &candidate)?;
        result.hits.push(ArchiveSearchHitV2 {
            subject,
            revision_id: candidate.digest,
            title: record.title,
            content_source: record.source,
        });
    }
    Ok(())
}
