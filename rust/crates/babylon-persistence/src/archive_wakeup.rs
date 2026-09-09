//! Additive, identity-checked Archive wake hints. No notification is evidence.

use postgres::{Client, GenericClient};
use sha2::{Digest as _, Sha256};

use crate::archive::{database, decode, decode_digest};
use crate::{ArchiveSchemaDispositionV1, SemanticArchiveErrorV1, SCHEMA_ADVISORY_LOCK_KEY};

/// The sole Archive transport channel. Its payload is always empty.
pub const ARCHIVE_WAKEUP_CHANNEL_V1: &str = "babylon_archive_wakeup_v1";
const SQL: &str = include_str!("../migrations/archive_wakeup_v1.sql");
const FUNCTION_BODY: &str = "\nBEGIN\n    PERFORM pg_catalog.pg_notify('babylon_archive_wakeup_v1', '');\n    RETURN NULL;\nEND\n";
const TRIGGERS: [(&str, &str, &str, i16); 2] = [
    (
        "archive_wakeup_enrollment_v1",
        "babylon_meta",
        "archive_retention_v2",
        4,
    ),
    ("archive_wakeup_tick_v1", "babylon_state", "tick_commit", 4),
];

/// Identity of the additive transport schema; original Archive hashes stay intact.
#[must_use]
pub fn archive_wakeup_migration_sha256_v1() -> [u8; 32] {
    Sha256::digest(SQL.as_bytes()).into()
}

pub(crate) fn install(
    client: &mut Client,
) -> Result<ArchiveSchemaDispositionV1, SemanticArchiveErrorV1> {
    let mut tx = client
        .transaction()
        .map_err(|e| database("begin Archive wakeup install", &e))?;
    tx.query_one(
        "SELECT pg_catalog.pg_advisory_xact_lock($1)",
        &[&SCHEMA_ADVISORY_LOCK_KEY],
    )
    .map_err(|e| database("lock Archive wakeup install", &e))?;
    let exists: bool = decode(
        &tx.query_one(
            "SELECT pg_catalog.to_regclass('babylon_meta.archive_wakeup_schema_v1') IS NOT NULL",
            &[],
        )
        .map_err(|e| database("inspect Archive wakeup marker", &e))?,
        0,
    )?;
    let disposition = if exists {
        validate(&mut tx)?;
        ArchiveSchemaDispositionV1::AlreadyCurrent
    } else {
        let partial: bool = decode(&tx.query_one(
            "SELECT pg_catalog.to_regprocedure('babylon_meta.archive_wakeup_v1()') IS NOT NULL OR EXISTS \
            (SELECT 1 FROM pg_catalog.pg_trigger WHERE tgname LIKE 'archive_wakeup_%_v1')", &[])
            .map_err(|e| database("inspect partial Archive wakeup schema", &e))?, 0)?;
        if partial {
            return Err(SemanticArchiveErrorV1::PartialSchema);
        }
        tx.batch_execute(SQL)
            .map_err(|e| database("install Archive wakeup transport", &e))?;
        tx.execute(
            "INSERT INTO babylon_meta.archive_wakeup_schema_v1 VALUES(TRUE,$1)",
            &[&&archive_wakeup_migration_sha256_v1()[..]],
        )
        .map_err(|e| database("mark Archive wakeup installation", &e))?;
        validate(&mut tx)?;
        ArchiveSchemaDispositionV1::Installed
    };
    tx.commit()
        .map_err(|e| database("commit Archive wakeup install", &e))?;
    Ok(disposition)
}

pub(crate) fn validate(client: &mut impl GenericClient) -> Result<(), SemanticArchiveErrorV1> {
    let rows = client
        .query(
            "SELECT migration_sha256 FROM babylon_meta.archive_wakeup_schema_v1 WHERE singleton",
            &[],
        )
        .map_err(|e| database("read Archive wakeup identity", &e))?;
    if rows.len() != 1 || decode_digest(&rows[0], 0)? != archive_wakeup_migration_sha256_v1() {
        return Err(SemanticArchiveErrorV1::SchemaMismatch);
    }
    validate_function(client)?;
    let rows = client.query("SELECT t.tgname,n.nspname,c.relname,t.tgtype,t.tgenabled::text,t.tgnargs,t.tgqual IS NULL,t.tgoldtable IS NULL AND t.tgnewtable IS NULL,t.tgattr::text='',NOT t.tgisinternal \
        FROM pg_catalog.pg_trigger t JOIN pg_catalog.pg_class c ON c.oid=t.tgrelid \
        JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace \
        WHERE t.tgfoid='babylon_meta.archive_wakeup_v1()'::regprocedure ORDER BY t.tgname", &[])
        .map_err(|e| database("read Archive wakeup triggers", &e))?;
    if rows.len() != TRIGGERS.len() {
        return Err(SemanticArchiveErrorV1::SchemaMismatch);
    }
    for (row, (name, schema, table, kind)) in rows.iter().zip(TRIGGERS) {
        if decode::<String>(row, 0)? != name
            || decode::<String>(row, 1)? != schema
            || decode::<String>(row, 2)? != table
            || decode::<i16>(row, 3)? != kind
            || decode::<String>(row, 4)? != "O"
            || decode::<i16>(row, 5)? != 0
            || !decode::<bool>(row, 6)?
            || !decode::<bool>(row, 7)?
            || !decode::<bool>(row, 8)?
            || !decode::<bool>(row, 9)?
        {
            return Err(SemanticArchiveErrorV1::SchemaMismatch);
        }
    }
    Ok(())
}

fn validate_function(client: &mut impl GenericClient) -> Result<(), SemanticArchiveErrorV1> {
    let rows = client.query("SELECT p.prosrc,p.prosecdef,p.proconfig,p.prorettype='pg_catalog.trigger'::regtype,l.lanname,p.provolatile::text, \
        NOT EXISTS (SELECT 1 FROM pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) privilege \
        WHERE privilege.grantee<>p.proowner) \
        FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_language l ON l.oid=p.prolang \
        WHERE p.oid=pg_catalog.to_regprocedure('babylon_meta.archive_wakeup_v1()')", &[])
        .map_err(|e| database("read Archive wakeup function", &e))?;
    if rows.len() != 1 {
        return Err(SemanticArchiveErrorV1::SchemaMismatch);
    }
    let row = &rows[0];
    if decode::<String>(row, 0)? != FUNCTION_BODY
        || decode::<bool>(row, 1)?
        || decode::<Option<Vec<String>>>(row, 2)? != Some(vec!["search_path=pg_catalog".into()])
        || !decode::<bool>(row, 3)?
        || decode::<String>(row, 4)? != "plpgsql"
        || decode::<String>(row, 5)? != "v"
        || !decode::<bool>(row, 6)?
    {
        return Err(SemanticArchiveErrorV1::SchemaMismatch);
    }
    Ok(())
}
