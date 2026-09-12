//! Atomic installation and exact admission of the one supported database schema.

use babylon_kernel::content_digest::sha256_of;
use postgres::{Client, Config, GenericClient, IsolationLevel, NoTls, Transaction};

use crate::postgres_catalog::{
    acquire_lock, compare_catalog_census, parse_catalog_census, read_census_rows, release_lock,
    validate_connection_target, CatalogCensusEntry, CatalogCensusParseError, CatalogError,
    CATALOG_CONNECT_TIMEOUT, CATALOG_STARTUP_OPTIONS, CATALOG_TCP_USER_TIMEOUT,
};
use crate::PostgresDiagnostic;

/// One lock shared by schema construction, reference installation and role provisioning.
pub const SCHEMA_ADVISORY_LOCK_KEY: i64 = 0xBAB1_0537;
/// Exact current native schema source; historical SQL is not a runtime input.
pub const CURRENT_SCHEMA_SQL: &str = include_str!("../migrations/current_schema.sql");
const CURRENT_ARCHIVE_SQL: &str = include_str!("../migrations/current_archive.sql");
const CURRENT_VIEWS_SQL: &str = include_str!("../migrations/current_views.sql");
const FRESH_CENSUS: &str = include_str!("fixtures/fresh_schema_census.txt");
const FRESH_CENSUS_WITH_INTEL: &str = include_str!("fixtures/fresh_schema_census_with_intel.txt");
const CURRENT_CENSUSES: &[&str] = &[
    include_str!("fixtures/current_schema_census.txt"),
    include_str!("fixtures/current_schema_reader_census.txt"),
    include_str!("fixtures/current_schema_observer_census.txt"),
    include_str!("fixtures/current_schema_reader_observer_census.txt"),
];
const OWNER_SQL: &str = "SELECT database_row.datdba = role_row.oid \
    FROM pg_catalog.pg_database AS database_row \
    JOIN pg_catalog.pg_roles AS role_row ON role_row.rolname = CURRENT_USER \
    WHERE database_row.datname = pg_catalog.current_database()";
const MARKERS_SQL: &str = "SELECT \
    pg_catalog.to_regnamespace('babylon_ref') IS NOT NULL, \
    pg_catalog.to_regnamespace('babylon_state') IS NOT NULL, \
    pg_catalog.to_regnamespace('babylon_meta') IS NOT NULL";
const SENTINELS_SQL: &str = "SELECT \
    NOT EXISTS (SELECT 1 FROM pg_catalog.pg_default_acl LIMIT 1), \
    NOT EXISTS (SELECT 1 FROM pg_catalog.pg_seclabel LIMIT 1), \
    NOT EXISTS (SELECT 1 FROM pg_catalog.pg_shseclabel AS label \
      JOIN pg_catalog.pg_database AS database_row \
        ON label.classoid = 'pg_catalog.pg_database'::pg_catalog.regclass \
       AND label.objoid = database_row.oid \
      WHERE database_row.datname = pg_catalog.current_database() LIMIT 1)";
const WRITE_SETTINGS_SQL: &str = "SELECT \
    pg_catalog.current_setting('transaction_isolation'), \
    pg_catalog.current_setting('transaction_read_only'), \
    pg_catalog.current_setting('search_path'), \
    pg_catalog.current_setting('synchronous_commit'), \
    pg_catalog.current_setting('statement_timeout'), \
    pg_catalog.current_setting('lock_timeout'), \
    pg_catalog.current_setting('idle_in_transaction_session_timeout')";

/// Exact source identity admitted only after complete schema verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentSchemaIdentity([u8; 32]);

impl CurrentSchemaIdentity {
    /// Borrow the SHA-256 of the domain-framed exact current SQL sources.
    #[must_use]
    pub const fn schema_sha256(&self) -> &[u8; 32] {
        &self.0
    }
}

/// What this invocation established about the atomic schema commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentSchemaDisposition {
    Installed,
    AlreadyCurrent,
    ReconciledCommit,
}

/// Receipt for a supported complete schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentSchemaReport {
    pub disposition: CurrentSchemaDisposition,
    pub identity: CurrentSchemaIdentity,
}

/// Closed operation labels for secret-safe diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentSchemaOperation {
    Connect,
    VerifyOwner,
    VerifyServer,
    Classify,
    VerifySentinels,
    ReadIdentity,
    BeginInstall,
    ConfigureInstall,
    VerifySettings,
    ExecuteSchema,
    InsertIdentity,
    CommitInstall,
}

/// Typed refusal of an unsupported, incomplete, altered or unowned database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrentSchemaError {
    ConnectionTarget(CatalogError),
    Lock(CatalogError),
    Census(CatalogError),
    CensusFixture(CatalogCensusParseError),
    Unlock(CatalogError),
    Database {
        operation: CurrentSchemaOperation,
        diagnostic: Option<PostgresDiagnostic>,
    },
    CurrentUserIsNotDatabaseOwner,
    UnsupportedServerMajor {
        actual: u32,
    },
    UnsupportedSchema,
    NonCanonicalSession,
    IdentityMismatch,
    AuthoritySentinelResidue,
    FreshCensusMismatch {
        without_intel: Box<CatalogError>,
        with_intel: Box<CatalogError>,
    },
    CurrentCensusMismatch,
    AmbiguousCommitUnresolved,
    AmbiguousCommitAndReconciliation(Box<CurrentSchemaError>),
    FailureAndCleanup {
        primary: Box<CurrentSchemaError>,
        cleanup: Box<CurrentSchemaError>,
    },
}

impl std::fmt::Display for CurrentSchemaError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "current schema refused: {self:?}")
    }
}
impl std::error::Error for CurrentSchemaError {}

fn database(operation: CurrentSchemaOperation, error: &postgres::Error) -> CurrentSchemaError {
    CurrentSchemaError::Database {
        operation,
        diagnostic: Some(PostgresDiagnostic::capture(error)),
    }
}
fn decode(operation: CurrentSchemaOperation) -> CurrentSchemaError {
    CurrentSchemaError::Database {
        operation,
        diagnostic: None,
    }
}

/// SHA-256 of a domain tag followed by little-endian `u64` byte lengths and SQL bytes.
///
/// # Panics
/// Panics if an embedded SQL source exceeds the `u64` length range.
#[must_use]
pub fn current_schema_sha256() -> [u8; 32] {
    let mut bytes = b"babylon.current-schema.v1\0".to_vec();
    for sql in [CURRENT_SCHEMA_SQL, CURRENT_ARCHIVE_SQL, CURRENT_VIEWS_SQL] {
        bytes.extend_from_slice(
            &u64::try_from(sql.len())
                .expect("embedded SQL length fits u64")
                .to_le_bytes(),
        );
        bytes.extend_from_slice(sql.as_bytes());
    }
    sha256_of(&bytes)
}

/// Read-only preflight accepts only the exact fresh template or this current schema.
///
/// # Errors
/// Refuses a nonlocal, nonowned, incompatible or altered database before any writes.
pub fn preflight_current_schema(config: &Config) -> Result<(), CurrentSchemaError> {
    validate_connection_target(config).map_err(CurrentSchemaError::ConnectionTarget)?;
    let mut session = LockedSession::connect(&bounded_config(config))?;
    let result = inspect(session.client()).map(|_| ());
    session.finish(result)
}

/// Construct the complete current schema in one transaction, or verify it exactly.
///
/// A transport error at `COMMIT` is reconciled on a new connection under the same lock.
/// A proven fresh rollback permits one retry. Old or partial databases are never adopted.
///
/// # Errors
/// Refuses every unsupported database without mutating it and preserves cleanup failures.
pub fn install_current_schema(config: &Config) -> Result<CurrentSchemaReport, CurrentSchemaError> {
    validate_connection_target(config).map_err(CurrentSchemaError::ConnectionTarget)?;
    let bounded = bounded_config(config);
    let mut session = LockedSession::connect(&bounded)?;
    let result = install_locked(&bounded, &mut session);
    session.finish(result)
}

pub(crate) fn bounded_config(config: &Config) -> Config {
    let mut bounded = config.clone();
    bounded
        .connect_timeout(CATALOG_CONNECT_TIMEOUT)
        .tcp_user_timeout(CATALOG_TCP_USER_TIMEOUT)
        .options(CATALOG_STARTUP_OPTIONS);
    bounded
}

struct LockedSession {
    client: Option<Client>,
}
impl LockedSession {
    fn connect(config: &Config) -> Result<Self, CurrentSchemaError> {
        let mut client = config
            .connect(NoTls)
            .map_err(|error| database(CurrentSchemaOperation::Connect, &error))?;
        acquire_lock(&mut client).map_err(CurrentSchemaError::Lock)?;
        Ok(Self {
            client: Some(client),
        })
    }
    fn client(&mut self) -> &mut Client {
        self.client
            .as_mut()
            .expect("locked session contains a client")
    }
    fn reconnect(&mut self, config: &Config) -> Result<(), CurrentSchemaError> {
        self.client.take();
        *self = Self::connect(config)?;
        Ok(())
    }
    fn finish<T>(
        mut self,
        primary: Result<T, CurrentSchemaError>,
    ) -> Result<T, CurrentSchemaError> {
        let cleanup = self.client.as_mut().map_or(Ok(()), |client| {
            release_lock(client).map_err(CurrentSchemaError::Unlock)
        });
        match (primary, cleanup) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
            (Err(primary), Err(cleanup)) => Err(CurrentSchemaError::FailureAndCleanup {
                primary: Box::new(primary),
                cleanup: Box::new(cleanup),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Presence {
    Fresh,
    Current(CurrentSchemaIdentity),
}

fn install_locked(
    config: &Config,
    session: &mut LockedSession,
) -> Result<CurrentSchemaReport, CurrentSchemaError> {
    if let Presence::Current(identity) = inspect(session.client())? {
        return Ok(CurrentSchemaReport {
            disposition: CurrentSchemaDisposition::AlreadyCurrent,
            identity,
        });
    }
    for attempt in 0..2 {
        let mut tx = begin_install(session.client())?;
        execute_before_identity(&mut tx)?;
        let identity = current_schema_sha256();
        let affected = tx.execute("INSERT INTO babylon_meta.current_schema (singleton, schema_sha256) VALUES (true, $1)", &[&identity.as_slice()])
            .map_err(|error| database(CurrentSchemaOperation::InsertIdentity, &error))?;
        if affected != 1 {
            return Err(decode(CurrentSchemaOperation::InsertIdentity));
        }
        require_current_schema(&mut tx)?;
        match commit_install(tx)? {
            CommitOutcome::Committed => {
                return Ok(CurrentSchemaReport {
                    disposition: CurrentSchemaDisposition::Installed,
                    identity: CurrentSchemaIdentity(identity),
                })
            }
            CommitOutcome::Ambiguous => {
                let reconciled = session
                    .reconnect(config)
                    .and_then(|()| inspect(session.client()))
                    .map_err(|error| {
                        CurrentSchemaError::AmbiguousCommitAndReconciliation(Box::new(error))
                    })?;
                match reconciled {
                    Presence::Current(identity) => {
                        return Ok(CurrentSchemaReport {
                            disposition: CurrentSchemaDisposition::ReconciledCommit,
                            identity,
                        })
                    }
                    Presence::Fresh if attempt == 0 => {}
                    Presence::Fresh => return Err(CurrentSchemaError::AmbiguousCommitUnresolved),
                }
            }
        }
    }
    Err(CurrentSchemaError::AmbiguousCommitUnresolved)
}

enum CommitOutcome {
    Committed,
    Ambiguous,
}
fn commit_install(tx: Transaction<'_>) -> Result<CommitOutcome, CurrentSchemaError> {
    #[cfg(test)]
    match COMMIT_FAULT.with(|state| state.replace(0)) {
        1 => {
            tx.rollback()
                .map_err(|error| database(CurrentSchemaOperation::CommitInstall, &error))?;
            return Ok(CommitOutcome::Ambiguous);
        }
        2 => {
            tx.commit()
                .map_err(|error| database(CurrentSchemaOperation::CommitInstall, &error))?;
            return Ok(CommitOutcome::Ambiguous);
        }
        _ => {}
    }
    match tx.commit() {
        Ok(()) => Ok(CommitOutcome::Committed),
        Err(error) if error.as_db_error().is_some() => {
            Err(database(CurrentSchemaOperation::CommitInstall, &error))
        }
        Err(_) => Ok(CommitOutcome::Ambiguous),
    }
}
fn begin_install(client: &mut Client) -> Result<Transaction<'_>, CurrentSchemaError> {
    let mut tx = client
        .build_transaction()
        .isolation_level(IsolationLevel::Serializable)
        .read_only(false)
        .start()
        .map_err(|error| database(CurrentSchemaOperation::BeginInstall, &error))?;
    tx.batch_execute("SET LOCAL search_path TO pg_catalog; SET LOCAL synchronous_commit TO on")
        .map_err(|error| database(CurrentSchemaOperation::ConfigureInstall, &error))?;
    let row = tx
        .query_one(WRITE_SETTINGS_SQL, &[])
        .map_err(|error| database(CurrentSchemaOperation::VerifySettings, &error))?;
    for (index, expected) in ["serializable", "off", "pg_catalog", "on", "5s", "5s", "5s"]
        .iter()
        .enumerate()
    {
        let actual: String = row
            .try_get(index)
            .map_err(|error| database(CurrentSchemaOperation::VerifySettings, &error))?;
        if actual != *expected {
            return Err(decode(CurrentSchemaOperation::VerifySettings));
        }
    }
    Ok(tx)
}
fn execute_before_identity(tx: &mut Transaction<'_>) -> Result<(), CurrentSchemaError> {
    for sql in [CURRENT_SCHEMA_SQL, CURRENT_ARCHIVE_SQL, CURRENT_VIEWS_SQL] {
        tx.batch_execute(sql)
            .map_err(|error| database(CurrentSchemaOperation::ExecuteSchema, &error))?;
    }
    verify_current_census(tx)
}

fn inspect(client: &mut impl GenericClient) -> Result<Presence, CurrentSchemaError> {
    verify_owner_and_server(client)?;
    verify_sentinels(client)?;
    let row = client
        .query_one(MARKERS_SQL, &[])
        .map_err(|error| database(CurrentSchemaOperation::Classify, &error))?;
    let mut markers = [false; 3];
    for (index, marker) in markers.iter_mut().enumerate() {
        *marker = row
            .try_get(index)
            .map_err(|error| database(CurrentSchemaOperation::Classify, &error))?;
    }
    if markers == [false; 3] {
        let actual = read_census_rows(client).map_err(CurrentSchemaError::Census)?;
        compare_fresh_census(&actual)?;
        Ok(Presence::Fresh)
    } else if markers == [true; 3] {
        require_current_schema(client).map(Presence::Current)
    } else {
        Err(CurrentSchemaError::UnsupportedSchema)
    }
}

/// Verify identity, exact catalog shape, ownership and bounded authority sentinels.
/// The caller uses canonical startup settings (`search_path=pg_catalog`, `quote_all_identifiers=off`)
/// and supplies its transaction when this admission protects a subsequent write.
pub(crate) fn require_current_schema(
    client: &mut impl GenericClient,
) -> Result<CurrentSchemaIdentity, CurrentSchemaError> {
    let settings = client.query_one("SELECT pg_catalog.current_setting('search_path'), pg_catalog.current_setting('quote_all_identifiers')", &[])
        .map_err(|error| database(CurrentSchemaOperation::VerifySettings, &error))?;
    if settings
        .try_get::<_, String>(0)
        .map_err(|error| database(CurrentSchemaOperation::VerifySettings, &error))?
        != "pg_catalog"
        || settings
            .try_get::<_, String>(1)
            .map_err(|error| database(CurrentSchemaOperation::VerifySettings, &error))?
            != "off"
    {
        return Err(CurrentSchemaError::NonCanonicalSession);
    }
    verify_owner_and_server(client)?;
    verify_sentinels(client)?;
    verify_current_census(client)?;
    let rows = client
        .query(
            "SELECT singleton, schema_sha256 FROM babylon_meta.current_schema LIMIT 2",
            &[],
        )
        .map_err(|error| database(CurrentSchemaOperation::ReadIdentity, &error))?;
    let [row] = rows.as_slice() else {
        return Err(CurrentSchemaError::IdentityMismatch);
    };
    let singleton: bool = row
        .try_get(0)
        .map_err(|error| database(CurrentSchemaOperation::ReadIdentity, &error))?;
    let digest: Vec<u8> = row
        .try_get(1)
        .map_err(|error| database(CurrentSchemaOperation::ReadIdentity, &error))?;
    let expected = current_schema_sha256();
    if !singleton || digest != expected {
        return Err(CurrentSchemaError::IdentityMismatch);
    }
    Ok(CurrentSchemaIdentity(expected))
}

fn verify_owner_and_server(client: &mut impl GenericClient) -> Result<(), CurrentSchemaError> {
    let row = client
        .query_one(OWNER_SQL, &[])
        .map_err(|error| database(CurrentSchemaOperation::VerifyOwner, &error))?;
    let owner: bool = row
        .try_get(0)
        .map_err(|error| database(CurrentSchemaOperation::VerifyOwner, &error))?;
    if !owner {
        return Err(CurrentSchemaError::CurrentUserIsNotDatabaseOwner);
    }
    let row = client
        .query_one(
            "SELECT pg_catalog.current_setting('server_version_num')",
            &[],
        )
        .map_err(|error| database(CurrentSchemaOperation::VerifyServer, &error))?;
    let version: String = row
        .try_get(0)
        .map_err(|error| database(CurrentSchemaOperation::VerifyServer, &error))?;
    let major = version
        .parse::<u32>()
        .map_err(|_| decode(CurrentSchemaOperation::VerifyServer))?
        / 10_000;
    if major != 17 {
        return Err(CurrentSchemaError::UnsupportedServerMajor { actual: major });
    }
    Ok(())
}
fn verify_sentinels(client: &mut impl GenericClient) -> Result<(), CurrentSchemaError> {
    let row = client
        .query_one(SENTINELS_SQL, &[])
        .map_err(|error| database(CurrentSchemaOperation::VerifySentinels, &error))?;
    for index in 0..3 {
        let clean: bool = row
            .try_get(index)
            .map_err(|error| database(CurrentSchemaOperation::VerifySentinels, &error))?;
        if !clean {
            return Err(CurrentSchemaError::AuthoritySentinelResidue);
        }
    }
    Ok(())
}
fn compare_fresh_census(actual: &[CatalogCensusEntry]) -> Result<(), CurrentSchemaError> {
    let without = parse_catalog_census(FRESH_CENSUS).map_err(CurrentSchemaError::CensusFixture)?;
    let Err(without_intel) = compare_catalog_census(&without, actual) else {
        return Ok(());
    };
    let with =
        parse_catalog_census(FRESH_CENSUS_WITH_INTEL).map_err(CurrentSchemaError::CensusFixture)?;
    compare_catalog_census(&with, actual).map_err(|with_intel| {
        CurrentSchemaError::FreshCensusMismatch {
            without_intel: Box::new(without_intel),
            with_intel: Box::new(with_intel),
        }
    })
}
fn verify_current_census(client: &mut impl GenericClient) -> Result<(), CurrentSchemaError> {
    let mut actual = read_census_rows(client).map_err(CurrentSchemaError::Census)?;
    // The sole admitted optional cluster object has an independently pinned complete signature.
    // Reader and observer role provisioning is pinned by the four complete ACL configurations.
    let with_intel =
        parse_catalog_census(FRESH_CENSUS_WITH_INTEL).map_err(CurrentSchemaError::CensusFixture)?;
    let intel = with_intel
        .entries()
        .iter()
        .find(|entry| entry.key().schema() == "pg_roles" && entry.key().name() == "babylon_intel")
        .ok_or(CurrentSchemaError::CurrentCensusMismatch)?;
    actual.retain(|entry| entry != intel);
    for fixture in CURRENT_CENSUSES {
        let expected = parse_catalog_census(fixture).map_err(CurrentSchemaError::CensusFixture)?;
        if compare_catalog_census(&expected, &actual).is_ok() {
            return Ok(());
        }
    }
    Err(CurrentSchemaError::CurrentCensusMismatch)
}

#[cfg(test)]
thread_local! {
    static COMMIT_FAULT: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_census_configurations_pin_every_object_and_only_role_grants_differ() {
        let censuses = CURRENT_CENSUSES
            .iter()
            .map(|source| parse_catalog_census(source).unwrap())
            .collect::<Vec<_>>();
        let keys = censuses[0]
            .entries()
            .iter()
            .map(CatalogCensusEntry::key)
            .collect::<Vec<_>>();
        for census in &censuses {
            assert_eq!(
                census
                    .entries()
                    .iter()
                    .map(CatalogCensusEntry::key)
                    .collect::<Vec<_>>(),
                keys
            );
            assert!(keys
                .iter()
                .any(|key| key.schema() == "babylon_meta" && key.name() == "current_schema"));
            assert!(!keys.iter().any(|key| key.name().contains("retired")
                || key.name() == "schema_migration"
                || key.name().contains("authority_ledger")));
        }
        for census in &censuses[1..] {
            let differences = census
                .entries()
                .iter()
                .zip(censuses[0].entries())
                .filter(|(left, right)| left != right)
                .collect::<Vec<_>>();
            assert!(!differences.is_empty());
            assert!(differences
                .iter()
                .all(|(entry, _)| entry.key().schema() == "public"
                    && entry.key().kind() == crate::postgres_catalog::CatalogObjectKind::View));
        }
    }

    #[test]
    fn current_census_provenance_matches_exact_constructed_sources() {
        use std::fmt::Write as _;
        for (name, source) in [
            ("current_schema.sql", CURRENT_SCHEMA_SQL),
            ("current_archive.sql", CURRENT_ARCHIVE_SQL),
            ("current_views.sql", CURRENT_VIEWS_SQL),
            ("postgres_catalog.sql", include_str!("postgres_catalog.sql")),
        ] {
            let mut digest = String::with_capacity(64);
            for byte in sha256_of(source.as_bytes()) {
                write!(&mut digest, "{byte:02x}").unwrap();
            }
            let expected = format!("# source|{name}|{digest}");
            for fixture in CURRENT_CENSUSES {
                assert!(
                    fixture.lines().any(|line| line == expected),
                    "census source differs: {name}"
                );
            }
        }
    }

    #[test]
    fn schema_identity_binds_all_current_source_fragments() {
        let parts = [CURRENT_SCHEMA_SQL, CURRENT_ARCHIVE_SQL, CURRENT_VIEWS_SQL];
        let digest = current_schema_sha256();
        for changed in 0..parts.len() {
            let mut bytes = b"babylon.current-schema.v1\0".to_vec();
            for (index, source) in parts.iter().enumerate() {
                let mut part = source.as_bytes().to_vec();
                if index == changed {
                    part[0] ^= 1;
                }
                bytes.extend_from_slice(&u64::try_from(part.len()).unwrap().to_le_bytes());
                bytes.extend_from_slice(&part);
            }
            assert_ne!(sha256_of(&bytes), digest);
        }
    }
}

#[cfg(test)]
mod live_tests {
    use super::*;
    use std::str::FromStr;

    const ACK: &str = "I_UNDERSTAND_THIS_DISPOSABLE_RUNTIME_DROPS_ITS_SCRATCH_DATABASES_AND_ROLES";

    #[test]
    #[ignore = "requires the task-owned disposable PostgreSQL runtime"]
    fn live_fresh_install_is_atomic_idempotent_and_all_current_role_configs_are_admitted() {
        let base = validated_base_config();
        let db = TestDatabase::create(&base, "roles");
        let config = db.config(&base);
        preflight_current_schema(&config).unwrap();
        let installed = install_current_schema(&config).unwrap();
        assert_eq!(installed.disposition, CurrentSchemaDisposition::Installed);
        assert_eq!(
            install_current_schema(&config).unwrap().disposition,
            CurrentSchemaDisposition::AlreadyCurrent
        );
        assert_eq!(installed.identity.schema_sha256(), &current_schema_sha256());
        crate::install_reader_role(&config).unwrap();
        assert_eq!(
            install_current_schema(&config).unwrap().identity,
            installed.identity
        );
        crate::observer_reader::provision_observer_role(&config).unwrap();
        assert_eq!(
            install_current_schema(&config).unwrap().identity,
            installed.identity
        );
        let observer_db = TestDatabase::create(&base, "observer");
        let observer_config = observer_db.config(&base);
        install_current_schema(&observer_config).unwrap();
        crate::observer_reader::provision_observer_role(&observer_config).unwrap();
        assert_eq!(
            install_current_schema(&observer_config).unwrap().identity,
            installed.identity
        );
        observer_db.cleanup();
        db.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PostgreSQL runtime"]
    fn live_old_partial_and_unknown_databases_are_refused_without_mutation() {
        let base = validated_base_config();
        let db = TestDatabase::create(&base, "legacy");
        let config = db.config(&base);
        let mut client = config.connect(NoTls).unwrap();
        client.batch_execute("SET search_path TO pg_catalog; SET quote_all_identifiers TO off; SET jit TO off; SET event_triggers TO off").unwrap();
        client.batch_execute("CREATE SCHEMA babylon_state; CREATE TABLE babylon_state.schema_migration (version bigint PRIMARY KEY, checksum bytea NOT NULL); INSERT INTO babylon_state.schema_migration VALUES (1, decode(repeat('01',32),'hex'))").unwrap();
        let before = read_census_rows(&mut client).unwrap();
        assert!(matches!(
            install_current_schema(&config),
            Err(CurrentSchemaError::UnsupportedSchema)
        ));
        assert_eq!(read_census_rows(&mut client).unwrap(), before);
        assert_eq!(
            client
                .query_one("SELECT version FROM babylon_state.schema_migration", &[])
                .unwrap()
                .get::<_, i64>(0),
            1
        );
        drop(client);
        db.cleanup();

        let db = TestDatabase::create(&base, "foreign");
        let config = db.config(&base);
        let mut client = config.connect(NoTls).unwrap();
        client.batch_execute("SET search_path TO pg_catalog; SET quote_all_identifiers TO off; SET jit TO off; SET event_triggers TO off").unwrap();
        client.batch_execute("CREATE TABLE public.unique_user_data (value text); INSERT INTO public.unique_user_data VALUES ('preserve me')").unwrap();
        let before = read_census_rows(&mut client).unwrap();
        assert!(matches!(
            install_current_schema(&config),
            Err(CurrentSchemaError::FreshCensusMismatch { .. })
        ));
        assert_eq!(read_census_rows(&mut client).unwrap(), before);
        assert_eq!(
            client
                .query_one("SELECT value FROM public.unique_user_data", &[])
                .unwrap()
                .get::<_, String>(0),
            "preserve me"
        );
        drop(client);
        db.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PostgreSQL runtime"]
    fn live_current_identity_shape_owner_and_privilege_drift_refuse_before_writes() {
        let base = validated_base_config();
        let db = TestDatabase::create(&base, "drift");
        let config = db.config(&base);
        install_current_schema(&config).unwrap();
        let mut client = config.connect(NoTls).unwrap();
        client.batch_execute("SET search_path TO pg_catalog; SET quote_all_identifiers TO off; SET jit TO off; SET event_triggers TO off").unwrap();
        for mutation in [
            "UPDATE babylon_meta.current_schema SET schema_sha256 = decode(repeat('00',32),'hex')",
            "DELETE FROM babylon_meta.current_schema",
            "ALTER TABLE babylon_meta.current_schema ADD COLUMN extra text",
            "ALTER TABLE babylon_state.tick_commit DROP CONSTRAINT tick_commit_envelope_layout_v3",
            "GRANT SELECT ON babylon_state.tick_commit TO PUBLIC",
            "ALTER DEFAULT PRIVILEGES GRANT SELECT ON TABLES TO PUBLIC",
            "CREATE VIEW public.unregistered_view AS SELECT 1",
        ] {
            let mut tx = client.transaction().unwrap();
            tx.batch_execute(mutation).unwrap();
            assert!(
                require_current_schema(&mut tx).is_err(),
                "admitted alteration: {mutation}"
            );
            tx.rollback().unwrap();
            assert_eq!(
                require_current_schema(&mut client).unwrap().schema_sha256(),
                &current_schema_sha256()
            );
        }
        let mut tx = client.transaction().unwrap();
        let role = format!("per330_schema_nonowner_{}", std::process::id());
        tx.batch_execute(&format!(
            "CREATE ROLE {role} NOLOGIN; SET LOCAL ROLE {role}"
        ))
        .unwrap();
        assert_eq!(
            require_current_schema(&mut tx),
            Err(CurrentSchemaError::CurrentUserIsNotDatabaseOwner)
        );
        tx.rollback().unwrap();
        drop(client);
        db.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PostgreSQL runtime"]
    fn live_interruption_rolls_back_and_concurrent_connection_cannot_publish_partial_schema() {
        let base = validated_base_config();
        let db = TestDatabase::create(&base, "interrupt");
        let config = db.config(&base);
        let mut session = LockedSession::connect(&bounded_config(&config)).unwrap();
        assert_eq!(inspect(session.client()).unwrap(), Presence::Fresh);
        let mut tx = begin_install(session.client()).unwrap();
        execute_before_identity(&mut tx).unwrap();
        assert!(matches!(
            install_current_schema(&config),
            Err(CurrentSchemaError::Lock(CatalogError::LockUnavailable))
        ));
        let mut outsider = config.connect(NoTls).unwrap();
        let visible: bool = outsider
            .query_one(
                "SELECT pg_catalog.to_regnamespace('babylon_meta') IS NOT NULL",
                &[],
            )
            .unwrap()
            .get(0);
        assert!(
            !visible,
            "uncommitted schema and marker must remain invisible"
        );
        drop(outsider);
        tx.rollback().unwrap();
        assert_eq!(inspect(session.client()).unwrap(), Presence::Fresh);
        session.finish(Ok(())).unwrap();
        assert_eq!(
            install_current_schema(&config).unwrap().disposition,
            CurrentSchemaDisposition::Installed
        );
        db.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PostgreSQL runtime"]
    fn live_ambiguous_commit_reconciles_both_rollback_and_lost_acknowledgement() {
        let base = validated_base_config();
        for (label, fault, expected) in [
            ("rollback", 1, CurrentSchemaDisposition::Installed),
            ("lostack", 2, CurrentSchemaDisposition::ReconciledCommit),
        ] {
            let db = TestDatabase::create(&base, label);
            let config = db.config(&base);
            COMMIT_FAULT.with(|state| state.set(fault));
            let installed = install_current_schema(&config).unwrap();
            assert_eq!(installed.disposition, expected);
            assert_eq!(COMMIT_FAULT.with(std::cell::Cell::get), 0);
            let second = install_current_schema(&config).unwrap();
            assert_eq!(second.disposition, CurrentSchemaDisposition::AlreadyCurrent);
            assert_eq!(installed.identity, second.identity);
            db.cleanup();
        }
    }

    #[test]
    #[ignore = "requires the task-owned disposable PostgreSQL runtime"]
    fn live_h3_reference_rollback_and_killed_retry_preserve_atomicity() {
        let base = validated_base_config();
        let db = TestDatabase::create(&base, "hexrollback");
        let config = db.config(&base);
        install_current_schema(&config).unwrap();
        crate::h3_reference_installer::live_postgres_tests::verify_rollback_and_killed_retry(
            &config,
            &base,
            std::time::Instant::now(),
        );
        db.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PostgreSQL runtime"]
    fn live_h3_reference_lost_acknowledgement_reconciles_committed_rows() {
        let base = validated_base_config();
        let db = TestDatabase::create(&base, "hexlostack");
        let config = db.config(&base);
        install_current_schema(&config).unwrap();
        crate::h3_reference_installer::live_postgres_tests::verify_committed_reconciliation(
            &config,
            &base,
            std::time::Instant::now(),
        );
        db.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PostgreSQL runtime"]
    fn live_h3_reference_membership_cardinality_refuses_excess_rows() {
        let base = validated_base_config();
        let db = TestDatabase::create(&base, "hexcardinality");
        let config = db.config(&base);
        install_current_schema(&config).unwrap();
        crate::h3_reference_installer::live_postgres_tests::verify_membership_cardinality_bound(
            &config,
        );
        db.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PostgreSQL runtime"]
    fn live_spatial_reference_commit_protocol_refuses_product_drift() {
        let base = validated_base_config();
        let db = TestDatabase::create(&base, "spatial");
        let config = db.config(&base);
        install_current_schema(&config).unwrap();
        crate::spatial_reference_installer::live_postgres_tests::verify_commit_protocol(
            &config, &base,
        );
        db.cleanup();
    }

    fn validated_base_config() -> Config {
        assert_eq!(
            std::env::var("BABYLON_POSTGRES_DISPOSABLE_ACK").as_deref(),
            Ok(ACK)
        );
        let canary = std::env::var("BABYLON_POSTGRES_DISPOSABLE_CANARY")
            .expect("runner supplies the disposable canary");
        assert_eq!(canary.len(), 32);
        let dsn =
            std::env::var("BABYLON_POSTGRES_TEST_DSN").expect("runner supplies the disposable DSN");
        let config = Config::from_str(&dsn).expect("runner DSN parses");
        validate_connection_target(&config).unwrap();
        assert_eq!(config.get_user(), Some("test"));
        assert_eq!(config.get_dbname(), Some("postgres"));
        let actual: Option<String> = config
            .connect(NoTls)
            .unwrap()
            .query_one(
                "SELECT pg_catalog.current_setting('babylon.disposable_runtime', true)",
                &[],
            )
            .unwrap()
            .get(0);
        assert_eq!(actual.as_deref(), Some(canary.as_str()));
        config
    }

    struct TestDatabase {
        name: String,
        admin: Config,
        active: bool,
    }
    impl TestDatabase {
        fn create(base: &Config, label: &str) -> Self {
            assert!(label.bytes().all(|byte| byte.is_ascii_lowercase()));
            let name = format!("current_schema_{label}_{}", std::process::id());
            let mut admin = base.clone();
            admin.dbname("postgres");
            admin
                .connect(NoTls)
                .unwrap()
                .batch_execute(&format!(
                    "CREATE DATABASE \"{name}\" OWNER test TEMPLATE template1"
                ))
                .unwrap();
            Self {
                name,
                admin,
                active: true,
            }
        }
        fn config(&self, base: &Config) -> Config {
            let mut config = base.clone();
            config.dbname(&self.name);
            config
        }
        fn cleanup(mut self) {
            self.try_drop_database()
                .expect("current schema scratch cleanup");
            self.active = false;
        }
        fn try_drop_database(&self) -> Result<(), postgres::Error> {
            self.admin.connect(NoTls)?.batch_execute(&format!(
                "DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)",
                self.name
            ))
        }
    }
    impl Drop for TestDatabase {
        fn drop(&mut self) {
            if self.active {
                if std::thread::panicking() {
                    let _cleanup = self.try_drop_database();
                } else {
                    self.try_drop_database()
                        .expect("current schema scratch cleanup");
                }
            }
        }
    }
}
