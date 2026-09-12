//! Verify current Archive wake hints. No notification is evidence.

use postgres::GenericClient;

use crate::archive::{database, decode};
use crate::SemanticArchiveError;

/// The sole Archive transport channel. Its payload is always empty.
pub const ARCHIVE_WAKEUP_CHANNEL: &str = "babylon_archive_wakeup_v1";
const FUNCTION_BODY: &str = "\nBEGIN\n    PERFORM pg_catalog.pg_notify('babylon_archive_wakeup_v1', '');\n    RETURN NULL;\nEND\n";
const TRIGGERS: [(&str, &str, &str, i16); 2] = [
    (
        "archive_wakeup_enrollment_v1",
        "babylon_meta",
        "campaign",
        4,
    ),
    ("archive_wakeup_tick_v1", "babylon_state", "tick_commit", 4),
];

pub(crate) fn validate(client: &mut impl GenericClient) -> Result<(), SemanticArchiveError> {
    validate_function(client)?;
    let rows = client.query("SELECT t.tgname,n.nspname,c.relname,t.tgtype,t.tgenabled::text,t.tgnargs,t.tgqual IS NULL,t.tgoldtable IS NULL AND t.tgnewtable IS NULL,t.tgattr::text='',NOT t.tgisinternal \
        FROM pg_catalog.pg_trigger t JOIN pg_catalog.pg_class c ON c.oid=t.tgrelid \
        JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace \
        WHERE t.tgfoid='babylon_meta.archive_wakeup_v1()'::regprocedure ORDER BY t.tgname", &[])
        .map_err(|e| database("read Archive wakeup triggers", &e))?;
    if rows.len() != TRIGGERS.len() {
        return Err(SemanticArchiveError::SchemaMismatch);
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
            return Err(SemanticArchiveError::SchemaMismatch);
        }
    }
    Ok(())
}

fn validate_function(client: &mut impl GenericClient) -> Result<(), SemanticArchiveError> {
    let rows = client.query("SELECT p.prosrc,p.prosecdef,p.proconfig,p.prorettype='pg_catalog.trigger'::regtype,l.lanname,p.provolatile::text, \
        NOT EXISTS (SELECT 1 FROM pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) privilege \
        WHERE privilege.grantee<>p.proowner) \
        FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_language l ON l.oid=p.prolang \
        WHERE p.oid=pg_catalog.to_regprocedure('babylon_meta.archive_wakeup_v1()')", &[])
        .map_err(|e| database("read Archive wakeup function", &e))?;
    if rows.len() != 1 {
        return Err(SemanticArchiveError::SchemaMismatch);
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
        return Err(SemanticArchiveError::SchemaMismatch);
    }
    Ok(())
}
