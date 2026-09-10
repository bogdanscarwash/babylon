//! Byte-exact contracts for the PER-23 Slice 1 read-only reader role and
//! fog-safe committed-tick status view (ADR249 R8).

use babylon_persistence::{
    CommittedTickStatusV1, ConnectionTargetRejection, ReaderRoleDispositionV1,
    SemanticArchiveErrorV1, SemanticArchiveReaderErrorV1, SemanticArchiveReaderV1,
    COMMITTED_TICK_STATUS_SQL_V1, READER_DSN_ENV_V1, READER_ROLE_CREATE_SQL_V1,
    READER_ROLE_NAME_V1, READER_VIEW_CANONICAL_DEF_V1,
};

#[test]
fn reader_role_create_sql_pins_exact_locked_attributes() {
    assert_eq!(READER_ROLE_NAME_V1, "babylon_reader");
    assert_eq!(
        READER_ROLE_CREATE_SQL_V1,
        "CREATE ROLE babylon_reader NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS"
    );
    assert!(!READER_ROLE_CREATE_SQL_V1.contains("PASSWORD"));
    assert!(!READER_ROLE_CREATE_SQL_V1.contains(" LOGIN"));
    assert!(READER_ROLE_CREATE_SQL_V1.contains("NOBYPASSRLS"));
    assert_eq!(READER_DSN_ENV_V1, "BABYLON_READER_DSN");
}

#[test]
fn committed_tick_status_read_goes_through_the_view_only() {
    assert!(COMMITTED_TICK_STATUS_SQL_V1.contains("FROM public.v_committed_tick_status_v1"));
    assert!(COMMITTED_TICK_STATUS_SQL_V1.contains("WHERE campaign_id = $1::uuid"));
    assert!(
        !COMMITTED_TICK_STATUS_SQL_V1.contains("babylon_state"),
        "the reader must never touch the base table directly"
    );
    assert!(COMMITTED_TICK_STATUS_SQL_V1.contains("ORDER BY resolve_tick DESC"));
}

#[test]
fn reader_view_canonical_definition_pins_the_exact_view_identity() {
    assert_eq!(
        READER_VIEW_CANONICAL_DEF_V1,
        "SELECT campaign_id, resolve_tick, envelope_layout_version, tick_content_hash, \
         envelope_digest FROM babylon_state.tick_commit"
    );
    assert!(
        !READER_VIEW_CANONICAL_DEF_V1.contains(';'),
        "the canonical definition carries no statement separator"
    );
}

#[test]
fn reader_privilege_census_pins_the_exact_restricted_relation_set() {
    let source = include_str!("../src/reader.rs");
    let census = source
        .split("READER_PRIVILEGE_CENSUS_SQL_V1: &str =")
        .nth(1)
        .expect("census SQL constant exists")
        .split(';')
        .next()
        .expect("census SQL constant terminates");
    for marker in [
        "WITH RECURSIVE role_closure(oid) AS",
        "SELECT 0::pg_catalog.oid",
        "pg_catalog.pg_auth_members",
        "pg_catalog.aclexplode",
        "'babylon_state'",
        "'babylon_meta'",
        "'v_archive_revision_known_v2'",
        "'v_committed_tick_status_v1'",
        ":OWNERSHIP",
        "is_grantable",
        "attacl",
    ] {
        assert!(census.contains(marker), "census SQL must pin {marker}");
    }
    for forbidden in ["GRANT ", "REVOKE ", "has_table_privilege"] {
        assert!(
            !census.contains(forbidden),
            "the census query is read-only and must not contain {forbidden}"
        );
    }
    let authority = source
        .split("READER_SESSION_AUTHORITY_SQL_V1: &str =")
        .nth(1)
        .expect("session authority SQL constant exists")
        .split(';')
        .next()
        .expect("session authority SQL constant terminates");
    assert!(authority.contains("current_user"));
    assert!(authority.contains("rolsuper"));
}

#[test]
fn revision_reader_has_no_current_head_sql_path() {
    let source = include_str!("../src/archive_revision/read.rs");
    for forbidden in ["babylon_meta.", "babylon_state.", "v_archive_subject_atoms"] {
        assert!(
            !source.contains(forbidden),
            "read boundary must not name {forbidden}"
        );
    }
    for required in [
        "v_archive_revision_known_v2",
        "v_archive_revision_scope_v2",
        "v_archive_revision_index_v2",
        "v_archive_tick_knowledge_v2",
    ] {
        assert!(
            source.contains(required),
            "one scoped read requires {required}"
        );
    }
}

#[test]
fn reader_error_taxonomy_is_closed_and_reader_scoped() {
    fn assert_exhaustive(error: &SemanticArchiveReaderErrorV1) -> usize {
        match error {
            SemanticArchiveReaderErrorV1::MissingEnv(_)
            | SemanticArchiveReaderErrorV1::EnvNotUtf8(_)
            | SemanticArchiveReaderErrorV1::InvalidDsn
            | SemanticArchiveReaderErrorV1::ConnectionTarget(_)
            | SemanticArchiveReaderErrorV1::RoleMismatch
            | SemanticArchiveReaderErrorV1::ViewMismatch
            | SemanticArchiveReaderErrorV1::PrivilegeDrift(_)
            | SemanticArchiveReaderErrorV1::WriterAuthorityRefused(_)
            | SemanticArchiveReaderErrorV1::CurrentSchema(_)
            | SemanticArchiveReaderErrorV1::Archive(_)
            | SemanticArchiveReaderErrorV1::LockMismatch
            | SemanticArchiveReaderErrorV1::Database { .. } => 1,
        }
    }
    assert_eq!(
        assert_exhaustive(&SemanticArchiveReaderErrorV1::PrivilegeDrift(Vec::new())),
        1
    );
    assert_eq!(
        assert_exhaustive(&SemanticArchiveReaderErrorV1::WriterAuthorityRefused(vec![
            "babylon_state.tick_commit:OWNERSHIP".to_owned()
        ])),
        1
    );
    assert_eq!(
        assert_exhaustive(&SemanticArchiveReaderErrorV1::Archive(
            SemanticArchiveErrorV1::CollectionBound
        )),
        1
    );
    assert_ne!(
        SemanticArchiveReaderErrorV1::PrivilegeDrift(Vec::new()),
        SemanticArchiveReaderErrorV1::WriterAuthorityRefused(Vec::new()),
        "drift and writer authority are distinct refusals"
    );
}

fn refusal(raw: &str) -> SemanticArchiveReaderErrorV1 {
    SemanticArchiveReaderV1::from_dsn(raw)
        .map(|_| ())
        .unwrap_err()
}

#[test]
fn from_dsn_admits_only_validated_loopback_targets() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SemanticArchiveReaderV1>();
    assert_send_sync::<SemanticArchiveReaderErrorV1>();
    assert_send_sync::<CommittedTickStatusV1>();
    assert_send_sync::<ReaderRoleDispositionV1>();

    assert_eq!(
        refusal("not a postgres DSN"),
        SemanticArchiveReaderErrorV1::InvalidDsn
    );
    assert_eq!(
        refusal("postgresql://reader:secret@203.0.113.10:5432/babylon"),
        SemanticArchiveReaderErrorV1::ConnectionTarget(ConnectionTargetRejection::NonLoopbackTcp)
    );
    assert_eq!(
        refusal("postgresql://reader@127.0.0.1:5432/babylon?options=-c%20search_path%3Dredirected"),
        SemanticArchiveReaderErrorV1::ConnectionTarget(
            ConnectionTargetRejection::StartupOptionsOverride
        )
    );
    assert_eq!(
        refusal("postgresql://reader@127.0.0.1,127.0.0.2/babylon"),
        SemanticArchiveReaderErrorV1::ConnectionTarget(
            // The driver expands one default port per named host, so the
            // redundant port list is reported before the host count.
            ConnectionTargetRejection::MultiplePorts
        )
    );

    SemanticArchiveReaderV1::from_dsn("postgresql://reader:secret@127.0.0.1:5432/babylon")
        .expect("loopback reader DSN admits");
    SemanticArchiveReaderV1::from_dsn("postgresql://reader@/babylon?host=/var/run/postgresql")
        .expect("absolute unix-socket target admits");
}

#[test]
fn from_env_reports_a_missing_reader_dsn() {
    std::env::remove_var(READER_DSN_ENV_V1);
    assert_eq!(
        SemanticArchiveReaderV1::from_env().map(|_| ()).unwrap_err(),
        SemanticArchiveReaderErrorV1::MissingEnv(READER_DSN_ENV_V1)
    );
    std::env::set_var(
        READER_DSN_ENV_V1,
        "postgresql://reader@127.0.0.1:5432/babylon",
    );
    SemanticArchiveReaderV1::from_env().expect("loopback reader DSN admits");
    std::env::remove_var(READER_DSN_ENV_V1);
}
