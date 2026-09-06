//! Transactional exact-head adoption and strict enrollment of future foundations.

use postgres::{Client, GenericClient, IsolationLevel, Row};
use sha2::{Digest as _, Sha256};

use super::enrollment::{self, AdoptionSeed};
use super::record::RevisionRecord;
use super::storage::{signed, unsigned};
use super::{knowledge, storage, ArchivePublicationOriginV2, ArchiveReadScopeV2};
use crate::archive::{database, decode, decode_digest, decode_stored_atom, decode_subject_kind};
use crate::{
    ArchivePageRefV1, ArchiveSchemaDispositionV1, CampaignId, SemanticArchiveErrorV1,
    SCHEMA_ADVISORY_LOCK_KEY,
};

pub(super) const SQL: &str = include_str!("../../migrations/archive_revision_v2.sql");
const PAGE_BATCH: i64 = 16;

pub(super) fn migration_digest() -> [u8; 32] {
    Sha256::digest(SQL.as_bytes()).into()
}

pub(crate) fn installed(client: &mut impl GenericClient) -> Result<bool, SemanticArchiveErrorV1> {
    let row = client
        .query_one(
            "SELECT pg_catalog.to_regclass('babylon_meta.archive_revision_schema_v2') IS NOT NULL",
            &[],
        )
        .map_err(|error| database("inspect Archive revision schema", &error))?;
    if !decode::<bool>(&row, 0)? {
        return Ok(false);
    }
    let rows = client
        .query(
            "SELECT migration_sha256 FROM babylon_meta.archive_revision_schema_v2 WHERE singleton",
            &[],
        )
        .map_err(|error| database("read Archive revision schema identity", &error))?;
    if rows.len() != 1 || decode_digest(&rows[0], 0)? != migration_digest() {
        return Err(SemanticArchiveErrorV1::SchemaMismatch);
    }
    Ok(true)
}

pub(crate) fn install(
    client: &mut Client,
) -> Result<ArchiveSchemaDispositionV1, SemanticArchiveErrorV1> {
    client
        .query_one(
            "SELECT pg_catalog.pg_advisory_lock($1)",
            &[&SCHEMA_ADVISORY_LOCK_KEY],
        )
        .map_err(|error| database("lock Archive revision schema", &error))?;
    let result = install_locked(client);
    let unlock = client
        .query_one(
            "SELECT pg_catalog.pg_advisory_unlock($1)",
            &[&SCHEMA_ADVISORY_LOCK_KEY],
        )
        .map_err(|error| database("unlock Archive revision schema", &error))
        .and_then(|row| decode::<bool>(&row, 0));
    match (result, unlock) {
        (Err(error), _) | (Ok(_), Err(error)) => Err(error),
        (Ok(disposition), Ok(true)) => Ok(disposition),
        (Ok(_), Ok(false)) => Err(SemanticArchiveErrorV1::SchemaMismatch),
    }
}

fn install_locked(
    client: &mut Client,
) -> Result<ArchiveSchemaDispositionV1, SemanticArchiveErrorV1> {
    if installed(client)? {
        let mut tx = client
            .build_transaction()
            .isolation_level(IsolationLevel::RepeatableRead)
            .start()
            .map_err(|error| database("begin Archive enrollment verification", &error))?;
        enrollment::validate_all(&mut tx)?;
        tx.commit()
            .map_err(|error| database("commit Archive enrollment verification", &error))?;
        return Ok(ArchiveSchemaDispositionV1::AlreadyCurrent);
    }
    refuse_partial(client)?;
    let mut tx = client
        .build_transaction()
        .isolation_level(IsolationLevel::Serializable)
        .start()
        .map_err(|error| database("begin Archive revision adoption", &error))?;
    // This must precede the capture transaction's first query. A snapshot taken
    // before waiting for a legacy writer would omit that writer's retained head.
    tx.batch_execute(
        "LOCK TABLE babylon_meta.campaign, babylon_state.tick_commit, \
        babylon_meta.archive_receipt_consumption_v1, babylon_meta.archive_page_v1, \
        babylon_meta.archive_page_atom_v1, babylon_meta.archive_atom_v1, \
        babylon_meta.archive_knowledge_grant_v1 IN SHARE ROW EXCLUSIVE MODE",
    )
    .map_err(|error| database("exclude obsolete Archive writers during adoption", &error))?;
    refuse_partial(&mut tx)?;
    tx.batch_execute(SQL)
        .map_err(|error| database("install immutable Archive relations", &error))?;
    adopt_campaigns(&mut tx)?;
    tx.execute("INSERT INTO babylon_meta.archive_revision_schema_v2(singleton,migration_sha256) VALUES(TRUE,$1)",
        &[&&migration_digest()[..]])
        .map_err(|error| database("publish Archive revision schema marker", &error))?;
    tx.commit()
        .map_err(|error| database("commit exact Archive head adoption", &error))?;
    Ok(ArchiveSchemaDispositionV1::Installed)
}

fn refuse_partial(client: &mut impl GenericClient) -> Result<(), SemanticArchiveErrorV1> {
    let row = client
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_class relation \
        JOIN pg_catalog.pg_namespace namespace ON namespace.oid=relation.relnamespace \
        WHERE namespace.nspname='babylon_meta' AND relation.relname IN \
        ('archive_retention_v2','archive_page_revision_v2','archive_revision_atom_v2', \
        'archive_revision_grant_v2','archive_retention_seal_v2','archive_page_retired_v1', \
        'archive_page_atom_retired_v1','archive_tick_knowledge_v2','archive_tick_knowledge_member_v2'))",
            &[],
        )
        .map_err(|error| database("census partial Archive revision installation", &error))?;
    if decode::<bool>(&row, 0)? {
        return Err(SemanticArchiveErrorV1::PartialSchema);
    }
    Ok(())
}

fn adopt_campaigns(client: &mut impl GenericClient) -> Result<(), SemanticArchiveErrorV1> {
    let mut after: Option<uuid::Uuid> = None;
    loop {
        let rows = client
            .query(
                "SELECT campaign_id FROM babylon_meta.campaign \
            WHERE ($1::uuid IS NULL OR campaign_id>$1) ORDER BY campaign_id LIMIT 256",
                &[&after],
            )
            .map_err(|error| database("enumerate campaigns for Archive adoption", &error))?;
        if rows.is_empty() {
            return Ok(());
        }
        for row in rows {
            let id = decode(&row, 0)?;
            adopt_campaign(client, CampaignId::from_uuid(id))?;
            after = Some(id);
        }
    }
}

fn adopt_campaign(
    client: &mut impl GenericClient,
    campaign: CampaignId,
) -> Result<(), SemanticArchiveErrorV1> {
    let marker = client
        .query_opt(
            "SELECT resolve_tick,tick_content_hash FROM babylon_state.tick_commit \
        WHERE campaign_id=$1 ORDER BY resolve_tick DESC LIMIT 1",
            &[campaign.as_uuid()],
        )
        .map_err(|error| database("capture Archive adoption durable tail", &error))?;
    let (floor, hash) = marker
        .as_ref()
        .map(|row| Ok((unsigned(decode(row, 0)?)?, Some(decode_digest(row, 1)?))))
        .transpose()?
        .unwrap_or((0, None));
    let progress = client.query_one("SELECT COALESCE((SELECT processed_tick FROM public.v_archive_verification_v1 WHERE campaign_id=$1),0)", &[campaign.as_uuid()])
        .map_err(|error| database("capture Archive adoption processed prefix", &error))?;
    let processed = unsigned(decode(&progress, 0)?)?;
    if processed > floor {
        return Err(SemanticArchiveErrorV1::StoredPageMismatch);
    }
    let (count, heads_digest) = adopt_heads(client, campaign, floor)?;
    let floor = if let Some(hash) = hash {
        ArchiveReadScopeV2::committed(campaign, floor, hash)?
    } else {
        ArchiveReadScopeV2::foundation(campaign)
    };
    enrollment::insert(
        client,
        &AdoptionSeed {
            floor,
            processed,
            count,
            heads_digest,
        },
    )
}

fn adopt_heads(
    client: &mut impl GenericClient,
    campaign: CampaignId,
    floor: u64,
) -> Result<(u64, [u8; 32]), SemanticArchiveErrorV1> {
    let mut after_kind = String::new();
    let mut after_id = String::new();
    let mut count = 0_u64;
    let mut heads = Sha256::new();
    loop {
        let rows = client.query("SELECT subject_kind,subject_id,title,verified_tick,source_resolve_tick, \
            source_tick_content_hash,template_sha256,content_sha256,markdown,search_text,provenance_json \
            FROM babylon_meta.archive_page_retired_v1 WHERE campaign_id=$1 \
            AND (subject_kind,subject_id)>($2,$3) ORDER BY subject_kind,subject_id LIMIT $4",
            &[campaign.as_uuid(), &after_kind, &after_id, &PAGE_BATCH])
            .map_err(|error| database("read exact legacy Archive heads", &error))?;
        if rows.is_empty() {
            return Ok((count, heads.finalize().into()));
        }
        for row in &rows {
            let mut record = decode_legacy_head(row, campaign, floor)?;
            record.atoms = legacy_membership(client, &record)?;
            record.grants = knowledge::capture(client, &record)?;
            verify_source_marker(client, &record)?;
            record.emission = super::recovery::recover(&record)?;
            heads.update(record.digest()?);
            storage::insert(client, &record)?;
            count = count
                .checked_add(1)
                .ok_or(SemanticArchiveErrorV1::CollectionBound)?;
            record.subject.kind().as_str().clone_into(&mut after_kind);
            record.subject.id().clone_into(&mut after_id);
        }
    }
}

fn decode_legacy_head(
    row: &Row,
    campaign: CampaignId,
    floor: u64,
) -> Result<RevisionRecord, SemanticArchiveErrorV1> {
    let tick = unsigned(decode(row, 4)?)?;
    if unsigned(decode(row, 3)?)? != tick || tick > floor {
        return Err(SemanticArchiveErrorV1::StoredPageMismatch);
    }
    Ok(RevisionRecord {
        source: ArchiveReadScopeV2::committed(campaign, tick, decode_digest(row, 5)?)?,
        subject: ArchivePageRefV1::try_new(
            decode_subject_kind(&decode::<String>(row, 0)?)?,
            decode(row, 1)?,
        )?,
        effective_tick: floor,
        origin: ArchivePublicationOriginV2::AdoptedHead,
        title: decode(row, 2)?,
        template_sha256: decode_digest(row, 6)?,
        content_sha256: decode_digest(row, 7)?,
        markdown: decode(row, 8)?,
        search_text: decode(row, 9)?,
        provenance_json: decode(row, 10)?,
        atoms: Vec::new(),
        grants: Vec::new(),
        emission: None,
    })
}

fn legacy_membership(
    client: &mut impl GenericClient,
    record: &RevisionRecord,
) -> Result<Vec<crate::ArchiveAtomV1>, SemanticArchiveErrorV1> {
    let campaign = record.source.campaign_id();
    let rows = client.query("SELECT atom.campaign_id,atom.subject_kind,atom.subject_id,atom.signal_key, \
        atom.grant_key,atom.evidence_class,atom.value_kind,atom.value_text,atom.value_f64,atom.value_u64, \
        atom.value_bool,atom.provenance_source_id,atom.provenance_locator,atom.valid_tick,atom.atom_id, \
        membership.position,membership.source_resolve_tick FROM babylon_meta.archive_page_atom_retired_v1 membership \
        JOIN babylon_meta.archive_atom_v1 atom USING(atom_id) WHERE membership.campaign_id=$1 \
        AND membership.subject_kind=$2 AND membership.subject_id=$3 ORDER BY membership.position LIMIT 514",
        &[campaign.as_uuid(), &record.subject.kind().as_str(), &record.subject.id()])
        .map_err(|error| database("capture legacy ordered Archive membership", &error))?;
    rows.iter()
        .enumerate()
        .map(|(position, row)| {
            if decode::<i32>(row, 15)?
                != i32::try_from(position).map_err(|_| SemanticArchiveErrorV1::CollectionBound)?
                || unsigned(decode(row, 16)?)? != record.source.tick()
            {
                return Err(SemanticArchiveErrorV1::StoredPageMismatch);
            }
            decode_stored_atom(row)
        })
        .collect()
}

pub(super) fn verify_source_marker(
    client: &mut impl GenericClient,
    record: &RevisionRecord,
) -> Result<(), SemanticArchiveErrorV1> {
    let campaign = record.source.campaign_id();
    let tick = signed(record.source.tick())?;
    let row = client.query_opt("SELECT tick_content_hash FROM babylon_state.tick_commit WHERE campaign_id=$1 AND resolve_tick=$2 FOR SHARE",
        &[campaign.as_uuid(), &tick]).map_err(|error| database("validate retained Archive source marker", &error))?
        .ok_or(SemanticArchiveErrorV1::MissingCommittedReceipt)?;
    if Some(decode_digest(&row, 0)?) != record.source.tick_content_hash() {
        return Err(SemanticArchiveErrorV1::ReceiptMismatch);
    }
    Ok(())
}

/// Called only inside the existing foundation creation transaction.
pub(crate) fn enroll_foundation(
    client: &mut impl GenericClient,
    campaign: CampaignId,
    newly_inserted: bool,
) -> Result<(), SemanticArchiveErrorV1> {
    if installed(client)? {
        if newly_inserted {
            enrollment::insert(
                client,
                &AdoptionSeed {
                    floor: ArchiveReadScopeV2::foundation(campaign),
                    processed: 0,
                    count: 0,
                    heads_digest: Sha256::digest([]).into(),
                },
            )?;
        } else {
            enrollment::validate(client, campaign)?;
        }
    }
    Ok(())
}
