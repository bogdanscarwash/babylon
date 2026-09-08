//! Fresh revision schema installation and strict enrollment of current foundations.

use postgres::{Client, GenericClient, IsolationLevel};
use sha2::{Digest as _, Sha256};

use super::enrollment::{self, AdoptionSeed};
use super::record::RevisionRecord;
use super::storage::signed;
use super::ArchiveReadScopeV2;
use crate::archive::{database, decode, decode_digest};
use crate::{
    ArchiveSchemaDispositionV1, CampaignId, SemanticArchiveErrorV1, SCHEMA_ADVISORY_LOCK_KEY,
};

pub(super) const SQL: &str = include_str!("../../migrations/archive_revision_v2.sql");

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
        .map_err(|error| database("begin fresh Archive revision installation", &error))?;
    // Exclude concurrent foundation writes before checking the empty initial state.
    tx.batch_execute(
        "LOCK TABLE babylon_meta.campaign, babylon_state.tick_commit, \
        babylon_meta.archive_receipt_consumption_v1, babylon_meta.archive_page_v1, \
        babylon_meta.archive_page_atom_v1, babylon_meta.archive_atom_v1, \
        babylon_meta.archive_knowledge_grant_v1 IN SHARE ROW EXCLUSIVE MODE",
    )
    .map_err(|error| database("lock initial Archive schema relations", &error))?;
    refuse_partial(&mut tx)?;
    require_empty_campaigns(&mut tx)?;
    tx.batch_execute(SQL)
        .map_err(|error| database("install immutable Archive relations", &error))?;

    tx.execute("INSERT INTO babylon_meta.archive_revision_schema_v2(singleton,migration_sha256) VALUES(TRUE,$1)",
        &[&&migration_digest()[..]])
        .map_err(|error| database("publish Archive revision schema marker", &error))?;
    tx.commit()
        .map_err(|error| database("commit fresh Archive revision installation", &error))?;
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

pub(crate) fn require_empty_campaigns(
    client: &mut impl GenericClient,
) -> Result<(), SemanticArchiveErrorV1> {
    let row = client
        .query_one("SELECT EXISTS(SELECT 1 FROM babylon_meta.campaign)", &[])
        .map_err(|error| database("inspect initial Archive campaign state", &error))?;
    if decode::<bool>(&row, 0)? {
        return Err(SemanticArchiveErrorV1::RevisionSchemaAbsentForExistingCampaigns);
    }
    Ok(())
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
    if !installed(client)? {
        return Err(SemanticArchiveErrorV1::PartialSchema);
    }
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
    Ok(())
}
