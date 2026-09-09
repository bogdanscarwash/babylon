//! Bounded, exact-prefix contracts for the Rust-owned schema epoch.

use postgres::{Client, Config, IsolationLevel, NoTls, Row, Transaction};

use crate::postgres_catalog::{
    acquire_lock, catalog_census_under_lock, compare_catalog_census, parse_catalog_census,
    read_census_rows, release_lock, validate_connection_target, CatalogCensusEntry,
    CatalogCensusParseError, CatalogError, CatalogObjectKind, CATALOG_CONNECT_TIMEOUT,
    CATALOG_STARTUP_OPTIONS, CATALOG_TCP_USER_TIMEOUT, MAX_CATALOG_CENSUS_ROWS,
};
use crate::postgres_diagnostic::PostgresDiagnosticV1;
use crate::schema_migration::{
    MigrationChecksum, MigrationVersion, SchemaMigration, SchemaMigrationError,
};

/// Maximum number of compiled migrations or persisted ledger rows.
pub const MAX_SCHEMA_MIGRATIONS: usize = 256;
/// Maximum commit/reconciliation attempts for one version.
pub const MAX_COMMIT_ATTEMPTS_PER_VERSION: usize = 2;
pub(crate) const CURRENT_SCHEMA_EPOCH: usize = 7;

const MIGRATION_0001_SQL: &str = include_str!("../migrations/0001_owned_schema_epoch.sql");
const MIGRATION_0002_SQL: &str = include_str!("../migrations/0002_h3_cell.sql");
const MIGRATION_0003_SQL: &str = include_str!("../migrations/0003_h3_reference_cohort.sql");
const MIGRATION_0004_SQL: &str = include_str!("../migrations/0004_committed_tick_storage.sql");
const MIGRATION_0005_SQL: &str = include_str!("../migrations/0005_spatial_reference_products.sql");
const MIGRATION_0006_SQL: &str = include_str!("../migrations/0006_h3_shadow_keys.sql");
const MIGRATION_0007_SQL: &str = include_str!("../migrations/0007_h3_canonical_readers.sql");
const MIGRATION_0010_SQL: &str =
    include_str!("../migrations/0010_committed_tick_v2_preparation.sql");
const MIGRATION_0011_SQL: &str =
    include_str!("../migrations/0011_committed_tick_v2_activation.sql");
const FRESH_CENSUS: &str = include_str!("fixtures/fresh_schema_epoch_census_v2.txt");
const FRESH_CENSUS_WITH_INTEL: &str =
    include_str!("fixtures/fresh_schema_epoch_census_with_intel_v2.txt");
const EPOCH_OWNED_FRESH_CENSUS_V1: &str =
    include_str!("fixtures/schema_epoch_owned_fresh_census_v1.txt");
const EPOCH_OWNED_FRESH_CENSUS_V2: &str =
    include_str!("fixtures/schema_epoch_owned_fresh_census_v2.txt");
const EPOCH_OWNED_FRESH_CENSUS_V3: &str =
    include_str!("fixtures/schema_epoch_owned_fresh_census_v3.txt");
const EPOCH_OWNED_FRESH_CENSUS_V4: &str =
    include_str!("fixtures/schema_epoch_owned_fresh_census_v4.txt");
const EPOCH_OWNED_FRESH_CENSUS_V5: &str =
    include_str!("fixtures/schema_epoch_owned_fresh_census_v5.txt");
const EPOCH_OWNED_FRESH_CENSUS_V6: &str =
    include_str!("fixtures/schema_epoch_owned_fresh_census_v6.txt");
const EPOCH_OWNED_FRESH_CENSUS_V7: &str =
    include_str!("fixtures/schema_epoch_owned_fresh_census_v7.txt");
const OWNER_SQL: &str = "SELECT database_row.datdba = role_row.oid \
    FROM pg_catalog.pg_database AS database_row \
    JOIN pg_catalog.pg_roles AS role_row ON role_row.rolname = CURRENT_USER \
    WHERE database_row.datname = pg_catalog.current_database()";
const MARKERS_SQL: &str = "SELECT \
    pg_catalog.to_regnamespace('babylon_ref') IS NOT NULL, \
    pg_catalog.to_regnamespace('babylon_state') IS NOT NULL, \
    pg_catalog.to_regnamespace('babylon_meta') IS NOT NULL, \
    ledger.oid IS NOT NULL, \
    coalesce(ledger.relkind = 'r' AND ledger.relpersistence = 'p', false) \
    FROM (SELECT 1) AS singleton \
    LEFT JOIN pg_catalog.pg_class AS ledger \
      ON ledger.oid = pg_catalog.to_regclass('babylon_state.schema_migration')";
const FRESH_SENTINELS_SQL: &str = "SELECT \
    NOT EXISTS (SELECT 1 FROM pg_catalog.pg_default_acl LIMIT 1), \
    NOT EXISTS (SELECT 1 FROM pg_catalog.pg_seclabel LIMIT 1), \
    NOT EXISTS (SELECT 1 FROM pg_catalog.pg_shseclabel AS label \
      JOIN pg_catalog.pg_database AS database_row \
        ON label.classoid = 'pg_catalog.pg_database'::pg_catalog.regclass \
       AND label.objoid = database_row.oid \
      WHERE database_row.datname = pg_catalog.current_database() LIMIT 1)";
const LEDGER_SQL: &str = "SELECT version, checksum \
    FROM babylon_state.schema_migration ORDER BY version LIMIT $1";
const INSERT_LEDGER_SQL: &str = "INSERT INTO babylon_state.schema_migration \
    (version, checksum) VALUES ($1, $2)";
const WRITE_SETTINGS_SQL: &str = "SELECT \
    pg_catalog.current_setting('transaction_isolation'), \
    pg_catalog.current_setting('transaction_read_only'), \
    pg_catalog.current_setting('search_path'), \
    pg_catalog.current_setting('synchronous_commit'), \
    pg_catalog.current_setting('statement_timeout'), \
    pg_catalog.current_setting('lock_timeout'), \
    pg_catalog.current_setting('idle_in_transaction_session_timeout')";
const WRITE_LOCAL_SETTINGS_SQL: &str = "SET LOCAL search_path TO pg_catalog; \
    SET LOCAL synchronous_commit TO on";
const EPOCH_V1_SHAPE_SQL: &str = include_str!("schema_epoch_shape.sql");
const EPOCH_V2_SHAPE_SQL: &str = include_str!("schema_epoch_v2_shape.sql");
const EPOCH_V3_SHAPE_SQL: &str = include_str!("schema_epoch_v3_shape.sql");
const EPOCH_V4_SHAPE_SQL: &str = include_str!("schema_epoch_v4_shape.sql");
const EPOCH_V5_SHAPE_SQL: &str = include_str!("schema_epoch_v5_shape.sql");
const EPOCH_V6_SHAPE_SQL: &str = include_str!("schema_epoch_v6_shape.sql");
const EPOCH_V7_SHAPE_SQL: &str = include_str!("schema_epoch_v7_shape.sql");

/// Database lane selected under the schema advisory lock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaEpochOrigin {
    /// Pinned extension template with no Babylon objects.
    Fresh,
    /// Existing exact Rust migration prefix.
    ExistingRustPrefix,
}

/// Successful bounded schema migration receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaEpochReport {
    /// Lane observed before any new migration.
    pub origin: SchemaEpochOrigin,
    /// Exact applied prefix before this invocation.
    pub prior_applied: usize,
    /// Exact applied prefix after this invocation.
    pub final_applied: usize,
    /// Versions committed by this invocation.
    pub applied_versions: Vec<MigrationVersion>,
    /// Versions whose ambiguous commit was reconciled as committed.
    pub reconciled_versions: Vec<MigrationVersion>,
}

/// Closed database operations used in safe failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaEpochOperation {
    Connect,
    VerifyOwner,
    Classify,
    FreshSentinels,
    ReadLedger,
    BeginMigration,
    SetMigrationSettings,
    VerifyMigrationSettings,
    ExecuteMigration,
    VerifyEpochShape,
    InsertLedger,
    CommitMigration,
    ReconcileCommit,
    Unlock,
}

/// Bounded marker state for a partial or mixed authority epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaEpochObservation {
    pub schemas: SchemaEpochSchemas,
    pub ledger: SchemaEpochRelation,
}

/// Presence of the three owned schema markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaEpochSchemas {
    pub babylon_ref: bool,
    pub babylon_state: bool,
    pub babylon_meta: bool,
}

/// Closed relation-marker classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaEpochRelation {
    Absent,
    ExactTable,
    WrongShape,
}

/// One decoded row from `babylon_state.schema_migration`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistedMigration {
    version: MigrationVersion,
    checksum: MigrationChecksum,
}

impl PersistedMigration {
    /// Decode the signed version and bounded checksum returned by `PostgreSQL`.
    ///
    /// # Errors
    /// Returns [`SchemaMigrationError`] for a non-positive version or a
    /// checksum that is not exactly one SHA-256 value.
    pub fn from_database(version: i64, checksum: &[u8]) -> Result<Self, SchemaMigrationError> {
        Ok(Self {
            version: MigrationVersion::try_from(version)?,
            checksum: MigrationChecksum::from_database_bytes(checksum)?,
        })
    }
}

/// Exact-prefix refusal raised before any pending DDL executes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaEpochError {
    /// The compiled registry exceeded its fixed ceiling.
    CompiledMigrationBound { actual: usize, max: usize },
    /// The persisted ledger exceeded its fixed ceiling.
    LedgerRowBound { actual: usize, max: usize },
    /// A compiled version did not equal its one-based position.
    CompiledVersionMismatch {
        position: usize,
        expected: i64,
        actual: i64,
    },
    /// A ledger version did not equal its one-based row position.
    LedgerVersionMismatch {
        row_index: usize,
        expected: i64,
        actual: i64,
    },
    /// The database contains a version unknown to this binary.
    UnknownFutureVersion { actual: i64, latest_compiled: i64 },
    /// A persisted checksum differs from the exact compiled SQL checksum.
    LedgerChecksumMismatch { version: i64 },
    /// The built-in migration registry is malformed.
    CompiledMigration(SchemaMigrationError),
    /// A frozen census fixture was malformed in this binary.
    CensusFixture(CatalogCensusParseError),
    /// The supplied maintenance target violated the local-only connection contract.
    ConnectionTarget(CatalogError),
    /// Schema-lock acquisition failed before epoch inspection.
    Lock(CatalogError),
    /// A fresh or recorded-prefix census operation failed.
    Census(CatalogError),
    /// Explicit schema-lock release failed.
    Unlock(CatalogError),
    /// A database operation failed with an optional secret-safe driver diagnostic.
    Database {
        operation: SchemaEpochOperation,
        diagnostic: Option<PostgresDiagnosticV1>,
    },
    /// The connected principal is not exactly the current database owner.
    CurrentUserIsNotDatabaseOwner,
    /// Marker presence selected no complete supported epoch.
    PartialAuthorityEpoch { observation: SchemaEpochObservation },
    /// Rust-owned objects exist without even the first durable ledger marker.
    UnrecordedRustEpoch,
    /// Neither exact pinned fresh census variant matched.
    FreshCensusMismatch {
        without_intel: Box<CatalogError>,
        with_intel: Box<CatalogError>,
    },
    /// Database-local default privileges or security labels were present.
    AuthoritySentinelResidue,
    /// The three owned schemas or migration ledger have the wrong shape or authority.
    EpochShapeMismatch,
    /// The bounded post-epoch catalog census did not match its frozen origin.
    EpochCensusMismatch,
    /// A ledger insert affected a count other than one.
    AffectedRowCount {
        operation: SchemaEpochOperation,
        expected: u64,
        actual: u64,
    },
    /// A commit failed and its durable outcome could not be resolved safely.
    AmbiguousCommitUnresolved { version: i64, attempts: usize },
    /// Commit ambiguity remained visible when reconciliation itself refused.
    AmbiguousCommitAndReconciliation {
        version: i64,
        reconciliation: Box<SchemaEpochError>,
    },
    /// A primary failure and explicit unlock failure both occurred.
    FailureAndCleanup {
        primary: Box<SchemaEpochError>,
        cleanup: Box<SchemaEpochError>,
    },
}

impl std::fmt::Display for SchemaEpochError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "schema epoch refused: {self:?}")
    }
}

impl std::error::Error for SchemaEpochError {}

impl From<SchemaMigrationError> for SchemaEpochError {
    fn from(error: SchemaMigrationError) -> Self {
        Self::CompiledMigration(error)
    }
}

impl From<CatalogCensusParseError> for SchemaEpochError {
    fn from(error: CatalogCensusParseError) -> Self {
        Self::CensusFixture(error)
    }
}

impl PersistedMigration {
    /// Return the positive ledger version.
    #[must_use]
    pub fn version(self) -> MigrationVersion {
        self.version
    }

    /// Return the exact persisted checksum.
    #[must_use]
    pub fn checksum(self) -> MigrationChecksum {
        self.checksum
    }
}

/// Validate the local target and require the connected principal to own the database.
///
/// This bounded preflight returns before schema classification, migration, or any write.
///
/// # Errors
/// Returns [`SchemaEpochError`] when the target is not admitted, the bounded connection
/// fails, or the connected principal is not exactly the current database owner.
pub fn preflight_schema_epoch(config: &Config) -> Result<(), SchemaEpochError> {
    validate_connection_target(config).map_err(SchemaEpochError::ConnectionTarget)?;
    let mut client = bounded_config(config)
        .connect(NoTls)
        .map_err(|error| postgres_database_error(SchemaEpochOperation::Connect, &error))?;
    verify_database_owner(&mut client)
}

/// Validate, classify, and advance the closed Rust schema migration epoch.
///
/// This maintenance entry point cannot grant runtime writer authority.
///
/// # Errors
/// Returns [`SchemaEpochError`] for any target, authority, census, prefix,
/// transaction, commit-reconciliation, or cleanup failure.
pub fn migrate_schema_epoch(config: &Config) -> Result<SchemaEpochReport, SchemaEpochError> {
    validate_connection_target(config).map_err(SchemaEpochError::ConnectionTarget)?;
    let bounded = bounded_config(config);
    let mut session = LockedSession::connect(&bounded)?;
    let result = migrate_locked(&bounded, &mut session);
    session.finish(result)
}

pub(crate) fn bounded_config(config: &Config) -> Config {
    bounded_config_with_options(config, CATALOG_STARTUP_OPTIONS)
}

fn bounded_config_with_options(config: &Config, startup_options: &str) -> Config {
    let mut bounded = config.clone();
    bounded
        .connect_timeout(CATALOG_CONNECT_TIMEOUT)
        .tcp_user_timeout(CATALOG_TCP_USER_TIMEOUT)
        .options(startup_options);
    bounded
}

struct LockedSession {
    client: Option<Client>,
}

impl LockedSession {
    fn connect(config: &Config) -> Result<Self, SchemaEpochError> {
        let mut client = config
            .connect(NoTls)
            .map_err(|error| postgres_database_error(SchemaEpochOperation::Connect, &error))?;
        acquire_lock(&mut client).map_err(SchemaEpochError::Lock)?;
        Ok(Self {
            client: Some(client),
        })
    }

    fn client(&mut self) -> &mut Client {
        self.client
            .as_mut()
            .expect("locked session always contains one client")
    }

    fn reconnect(&mut self, config: &Config) -> Result<(), SchemaEpochError> {
        self.client.take();
        *self = Self::connect(config)?;
        Ok(())
    }

    fn finish<T>(mut self, primary: Result<T, SchemaEpochError>) -> Result<T, SchemaEpochError> {
        let cleanup = self.client.as_mut().map_or(Ok(()), |client| {
            release_lock(client)
                .map_err(SchemaEpochError::Unlock)
                .map_err(Box::new)
        });
        match (primary, cleanup) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), Ok(())) => Err(error),
            (Ok(_), Err(cleanup)) => Err(*cleanup),
            (Err(primary), Err(cleanup)) => Err(SchemaEpochError::FailureAndCleanup {
                primary: Box::new(primary),
                cleanup,
            }),
        }
    }
}

type ClientPrefixVerifier = fn(&mut Client) -> Result<(), SchemaEpochError>;
type TransactionPrefixVerifier =
    for<'transaction> fn(&mut Transaction<'transaction>) -> Result<(), SchemaEpochError>;

#[derive(Clone, Copy)]
struct SchemaPrefixContract {
    verify_client: ClientPrefixVerifier,
    verify_transaction: TransactionPrefixVerifier,
}

#[derive(Clone, Copy)]
struct SchemaEpochMigration {
    migration: SchemaMigration,
    prefix_contract: SchemaPrefixContract,
}

impl SchemaEpochMigration {
    fn new(migration: SchemaMigration, prefix_contract: SchemaPrefixContract) -> Self {
        Self {
            migration,
            prefix_contract,
        }
    }
}

trait MigrationRegistryEntry {
    fn migration(&self) -> SchemaMigration;
}

impl MigrationRegistryEntry for SchemaMigration {
    fn migration(&self) -> SchemaMigration {
        *self
    }
}

impl MigrationRegistryEntry for SchemaEpochMigration {
    fn migration(&self) -> SchemaMigration {
        self.migration
    }
}

const PREFIX_V1: SchemaPrefixContract = SchemaPrefixContract {
    verify_client: verify_v1_prefix_client,
    verify_transaction: verify_v1_prefix_transaction,
};
const PREFIX_V2: SchemaPrefixContract = SchemaPrefixContract {
    verify_client: verify_v2_prefix_client,
    verify_transaction: verify_v2_prefix_transaction,
};
const PREFIX_V3: SchemaPrefixContract = SchemaPrefixContract {
    verify_client: verify_v3_prefix_client,
    verify_transaction: verify_v3_prefix_transaction,
};
const PREFIX_V4: SchemaPrefixContract = SchemaPrefixContract {
    verify_client: verify_v4_prefix_client,
    verify_transaction: verify_v4_prefix_transaction,
};
const PREFIX_V5: SchemaPrefixContract = SchemaPrefixContract {
    verify_client: verify_v5_prefix_client,
    verify_transaction: verify_v5_prefix_transaction,
};
const PREFIX_V6: SchemaPrefixContract = SchemaPrefixContract {
    verify_client: verify_v6_prefix_client,
    verify_transaction: verify_v6_prefix_transaction,
};
const PREFIX_V7: SchemaPrefixContract = SchemaPrefixContract {
    verify_client: verify_v7_prefix_client,
    verify_transaction: verify_v7_prefix_transaction,
};

fn compiled_schema_epoch_migrations(
) -> Result<[SchemaEpochMigration; CURRENT_SCHEMA_EPOCH], SchemaMigrationError> {
    let migration_v1 = SchemaMigration::new(MigrationVersion::try_from(1)?, MIGRATION_0001_SQL)?;
    let migration_v2 = SchemaMigration::new(MigrationVersion::try_from(2)?, MIGRATION_0002_SQL)?;
    let migration_v3 = SchemaMigration::new(MigrationVersion::try_from(3)?, MIGRATION_0003_SQL)?;
    let migration_v4 = SchemaMigration::new(MigrationVersion::try_from(4)?, MIGRATION_0004_SQL)?;
    let migration_v5 = SchemaMigration::new(MigrationVersion::try_from(5)?, MIGRATION_0005_SQL)?;
    let migration_v6 = SchemaMigration::new(MigrationVersion::try_from(6)?, MIGRATION_0006_SQL)?;
    let migration_v7 = SchemaMigration::new(MigrationVersion::try_from(7)?, MIGRATION_0007_SQL)?;
    Ok([
        SchemaEpochMigration::new(migration_v1, PREFIX_V1),
        SchemaEpochMigration::new(migration_v2, PREFIX_V2),
        SchemaEpochMigration::new(migration_v3, PREFIX_V3),
        SchemaEpochMigration::new(migration_v4, PREFIX_V4),
        SchemaEpochMigration::new(migration_v5, PREFIX_V5),
        SchemaEpochMigration::new(migration_v6, PREFIX_V6),
        SchemaEpochMigration::new(migration_v7, PREFIX_V7),
    ])
}

/// Build the checked-in migration registry from exact SQL bytes.
///
/// # Errors
/// Returns [`SchemaMigrationError`] if a checked-in migration violates its
/// bounded byte contract.
pub fn compiled_schema_migrations(
) -> Result<[SchemaMigration; CURRENT_SCHEMA_EPOCH], SchemaMigrationError> {
    let compiled = compiled_schema_epoch_migrations()?;
    Ok([
        compiled[0].migration,
        compiled[1].migration,
        compiled[2].migration,
        compiled[3].migration,
        compiled[4].migration,
        compiled[5].migration,
        compiled[6].migration,
    ])
}

/// Build the dedicated one-way committed-tick V2 activation pair.
///
/// These migrations deliberately do not extend `CURRENT_SCHEMA_EPOCH`. Epochs 8 and 9 are the
/// immutable historical Rust-persistence cutover, whose authority rows were written by the
/// activation composition root rather than the ordinary schema migrator. The replacement V2
/// activator executes this exact 10/11 pair and writes its corresponding authority row after each
/// migration SQL body.
///
/// # Errors
/// Returns [`SchemaMigrationError`] if either checked-in SQL file violates the bounded migration
/// byte contract.
pub fn compiled_committed_tick_v2_activation_migrations(
) -> Result<[SchemaMigration; 2], SchemaMigrationError> {
    let preparation = SchemaMigration::new(MigrationVersion::try_from(10)?, MIGRATION_0010_SQL)?;
    let activation = SchemaMigration::new(MigrationVersion::try_from(11)?, MIGRATION_0011_SQL)?;
    Ok([preparation, activation])
}

/// Verify that persisted rows are an exact prefix of contiguous migrations.
///
/// The returned count is the first pending migration index.
///
/// # Errors
/// Returns [`SchemaEpochError`] for any bound, order, future-version, or
/// checksum conflict.
pub fn validate_migration_prefix(
    compiled: &[SchemaMigration],
    persisted: &[PersistedMigration],
) -> Result<usize, SchemaEpochError> {
    validate_registry_prefix(compiled, persisted)
}

fn validate_registry_prefix<RegistryEntry: MigrationRegistryEntry>(
    compiled: &[RegistryEntry],
    persisted: &[PersistedMigration],
) -> Result<usize, SchemaEpochError> {
    check_bounds(compiled.len(), persisted.len())?;
    validate_compiled_versions(compiled)?;
    validate_persisted_rows(compiled, persisted)?;
    Ok(persisted.len())
}

fn check_bounds(compiled: usize, persisted: usize) -> Result<(), SchemaEpochError> {
    if compiled > MAX_SCHEMA_MIGRATIONS {
        return Err(SchemaEpochError::CompiledMigrationBound {
            actual: compiled,
            max: MAX_SCHEMA_MIGRATIONS,
        });
    }
    if persisted > MAX_SCHEMA_MIGRATIONS {
        return Err(SchemaEpochError::LedgerRowBound {
            actual: persisted,
            max: MAX_SCHEMA_MIGRATIONS,
        });
    }
    Ok(())
}

fn validate_compiled_versions<RegistryEntry: MigrationRegistryEntry>(
    compiled: &[RegistryEntry],
) -> Result<(), SchemaEpochError> {
    for (position, migration) in compiled.iter().enumerate().take(MAX_SCHEMA_MIGRATIONS) {
        let expected = one_based_version(position);
        let actual = migration.migration().version().as_i64();
        if actual != expected {
            return Err(SchemaEpochError::CompiledVersionMismatch {
                position,
                expected,
                actual,
            });
        }
    }
    Ok(())
}

fn validate_persisted_rows<RegistryEntry: MigrationRegistryEntry>(
    compiled: &[RegistryEntry],
    persisted: &[PersistedMigration],
) -> Result<(), SchemaEpochError> {
    let latest_compiled = i64::try_from(compiled.len()).expect("migration bound fits i64");
    for (row_index, row) in persisted.iter().enumerate().take(MAX_SCHEMA_MIGRATIONS) {
        let expected = one_based_version(row_index);
        let actual = row.version.as_i64();
        if actual != expected {
            return Err(SchemaEpochError::LedgerVersionMismatch {
                row_index,
                expected,
                actual,
            });
        }
        if actual > latest_compiled {
            return Err(SchemaEpochError::UnknownFutureVersion {
                actual,
                latest_compiled,
            });
        }
        if row.checksum != compiled[row_index].migration().checksum() {
            return Err(SchemaEpochError::LedgerChecksumMismatch { version: actual });
        }
    }
    Ok(())
}

fn one_based_version(zero_based: usize) -> i64 {
    let one_based = zero_based
        .checked_add(1)
        .expect("migration bound cannot wrap");
    i64::try_from(one_based).expect("migration bound fits i64")
}

struct InspectedEpoch {
    origin: SchemaEpochOrigin,
    persisted: Vec<PersistedMigration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MigrationAttempt {
    Committed,
    Ambiguous,
}

fn migrate_locked(
    config: &Config,
    session: &mut LockedSession,
) -> Result<SchemaEpochReport, SchemaEpochError> {
    let compiled = compiled_schema_epoch_migrations()?;
    migrate_locked_with_registry(config, session, &compiled)
}

fn migrate_locked_with_registry(
    config: &Config,
    session: &mut LockedSession,
    compiled: &[SchemaEpochMigration],
) -> Result<SchemaEpochReport, SchemaEpochError> {
    migrate_locked_with_registry_using(config, session, compiled, &mut attempt_migration)
}

fn migrate_locked_with_registry_using<Attempt>(
    config: &Config,
    session: &mut LockedSession,
    compiled: &[SchemaEpochMigration],
    attempt_migration_fn: &mut Attempt,
) -> Result<SchemaEpochReport, SchemaEpochError>
where
    Attempt: FnMut(&mut Client, SchemaEpochMigration) -> Result<MigrationAttempt, SchemaEpochError>,
{
    validate_registry_prefix(compiled, &[])?;
    let initial = inspect_epoch(session.client(), compiled)?;
    let prior_applied = validate_registry_prefix(compiled, &initial.persisted)?;
    let mut report = SchemaEpochReport {
        origin: initial.origin,
        prior_applied,
        final_applied: prior_applied,
        applied_versions: Vec::with_capacity(compiled.len()),
        reconciled_versions: Vec::with_capacity(compiled.len()),
    };
    let target_applied = compiled.len();
    let mut next_pending = prior_applied;
    for _attempt in 0..MAX_SCHEMA_MIGRATIONS {
        if next_pending == target_applied {
            break;
        }
        let Some(migration) = compiled.get(next_pending).copied() else {
            break;
        };
        next_pending = apply_with_reconciliation_using(
            config,
            session,
            compiled,
            migration,
            &mut report,
            attempt_migration_fn,
        )?;
    }
    if next_pending < target_applied {
        return Err(SchemaEpochError::CompiledMigrationBound {
            actual: compiled.len(),
            max: MAX_SCHEMA_MIGRATIONS,
        });
    }
    let final_state = inspect_epoch(session.client(), compiled)?;
    let final_applied = validate_registry_prefix(compiled, &final_state.persisted)?;
    if final_state.origin != SchemaEpochOrigin::ExistingRustPrefix
        || final_applied != target_applied
    {
        return Err(SchemaEpochError::EpochShapeMismatch);
    }
    report.final_applied = final_applied;
    Ok(report)
}

fn apply_with_reconciliation_using<Attempt>(
    config: &Config,
    session: &mut LockedSession,
    compiled: &[SchemaEpochMigration],
    migration: SchemaEpochMigration,
    report: &mut SchemaEpochReport,
    attempt_migration_fn: &mut Attempt,
) -> Result<usize, SchemaEpochError>
where
    Attempt: FnMut(&mut Client, SchemaEpochMigration) -> Result<MigrationAttempt, SchemaEpochError>,
{
    let target_index = usize::try_from(migration.migration.version().as_i64() - 1)
        .expect("positive bounded migration version fits usize");
    for attempt in 0..MAX_COMMIT_ATTEMPTS_PER_VERSION {
        match attempt_migration_fn(session.client(), migration)? {
            MigrationAttempt::Committed => {
                report.applied_versions.push(migration.migration.version());
                return Ok(target_index + 1);
            }
            MigrationAttempt::Ambiguous => {
                let applied =
                    reconcile_ambiguous(config, session, compiled).map_err(|reconciliation| {
                        SchemaEpochError::AmbiguousCommitAndReconciliation {
                            version: migration.migration.version().as_i64(),
                            reconciliation: Box::new(reconciliation),
                        }
                    })?;
                if applied > target_index {
                    report
                        .reconciled_versions
                        .push(migration.migration.version());
                    return Ok(applied);
                }
                if applied != target_index || attempt + 1 == MAX_COMMIT_ATTEMPTS_PER_VERSION {
                    return Err(SchemaEpochError::AmbiguousCommitUnresolved {
                        version: migration.migration.version().as_i64(),
                        attempts: attempt + 1,
                    });
                }
            }
        }
    }
    Err(SchemaEpochError::AmbiguousCommitUnresolved {
        version: migration.migration.version().as_i64(),
        attempts: MAX_COMMIT_ATTEMPTS_PER_VERSION,
    })
}

fn reconcile_ambiguous(
    config: &Config,
    session: &mut LockedSession,
    compiled: &[SchemaEpochMigration],
) -> Result<usize, SchemaEpochError> {
    session.reconnect(config)?;
    let reconciled = inspect_epoch(session.client(), compiled)?;
    validate_registry_prefix(compiled, &reconciled.persisted)
}

fn inspect_epoch(
    client: &mut Client,
    compiled: &[SchemaEpochMigration],
) -> Result<InspectedEpoch, SchemaEpochError> {
    verify_database_owner(client)?;
    let observation = read_observation(client)?;
    match classify_observation(observation)? {
        SchemaEpochOrigin::Fresh => {
            verify_fresh_epoch(client)?;
            Ok(InspectedEpoch {
                origin: SchemaEpochOrigin::Fresh,
                persisted: Vec::new(),
            })
        }
        SchemaEpochOrigin::ExistingRustPrefix => {
            let persisted = read_ledger(client, compiled.len())?;
            require_recorded_rust_prefix(&persisted)?;
            let applied = validate_registry_prefix(compiled, &persisted)?;
            verify_recorded_prefix_client(client, compiled, applied)?;
            Ok(InspectedEpoch {
                origin: SchemaEpochOrigin::ExistingRustPrefix,
                persisted,
            })
        }
    }
}

pub(crate) fn inspect_schema_epoch_under_lock(
    client: &mut Client,
) -> Result<(SchemaEpochOrigin, usize), SchemaEpochError> {
    let compiled = compiled_schema_epoch_migrations()?;
    validate_registry_prefix(&compiled, &[])?;
    let inspected = inspect_epoch(client, &compiled)?;
    let applied = validate_registry_prefix(&compiled, &inspected.persisted)?;
    Ok((inspected.origin, applied))
}

fn require_recorded_rust_prefix(persisted: &[PersistedMigration]) -> Result<(), SchemaEpochError> {
    if persisted.is_empty() {
        Err(SchemaEpochError::UnrecordedRustEpoch)
    } else {
        Ok(())
    }
}

fn verify_recorded_prefix_client(
    client: &mut Client,
    compiled: &[SchemaEpochMigration],
    applied: usize,
) -> Result<(), SchemaEpochError> {
    let prefix_index = applied
        .checked_sub(1)
        .ok_or(SchemaEpochError::UnrecordedRustEpoch)?;
    let contract = compiled
        .get(prefix_index)
        .ok_or(SchemaEpochError::EpochShapeMismatch)?
        .prefix_contract;
    (contract.verify_client)(client)
}

fn verify_database_owner(client: &mut Client) -> Result<(), SchemaEpochError> {
    let row = client
        .query_opt(OWNER_SQL, &[])
        .map_err(|error| postgres_database_error(SchemaEpochOperation::VerifyOwner, &error))?
        .ok_or_else(|| database_error(SchemaEpochOperation::VerifyOwner))?;
    let is_owner = row
        .try_get::<_, bool>(0)
        .map_err(|error| postgres_database_error(SchemaEpochOperation::VerifyOwner, &error))?;
    if is_owner {
        Ok(())
    } else {
        Err(SchemaEpochError::CurrentUserIsNotDatabaseOwner)
    }
}

fn read_observation(client: &mut Client) -> Result<SchemaEpochObservation, SchemaEpochError> {
    let row = client
        .query_one(MARKERS_SQL, &[])
        .map_err(|error| postgres_database_error(SchemaEpochOperation::Classify, &error))?;
    Ok(SchemaEpochObservation {
        schemas: SchemaEpochSchemas {
            babylon_ref: decode_bool(&row, 0, SchemaEpochOperation::Classify)?,
            babylon_state: decode_bool(&row, 1, SchemaEpochOperation::Classify)?,
            babylon_meta: decode_bool(&row, 2, SchemaEpochOperation::Classify)?,
        },
        ledger: decode_relation_marker(&row, 3, 4)?,
    })
}

fn classify_observation(
    observation: SchemaEpochObservation,
) -> Result<SchemaEpochOrigin, SchemaEpochError> {
    let no_authority = !observation.schemas.babylon_ref && !observation.schemas.babylon_state;
    let fresh = no_authority
        && !observation.schemas.babylon_meta
        && observation.ledger == SchemaEpochRelation::Absent;
    if fresh {
        return Ok(SchemaEpochOrigin::Fresh);
    }
    let rust_prefix = observation.schemas.babylon_ref
        && observation.schemas.babylon_state
        && observation.schemas.babylon_meta
        && observation.ledger == SchemaEpochRelation::ExactTable;
    if rust_prefix {
        return Ok(SchemaEpochOrigin::ExistingRustPrefix);
    }
    Err(SchemaEpochError::PartialAuthorityEpoch { observation })
}

fn decode_relation_marker(
    row: &Row,
    exists_index: usize,
    exact_index: usize,
) -> Result<SchemaEpochRelation, SchemaEpochError> {
    let exists = decode_bool(row, exists_index, SchemaEpochOperation::Classify)?;
    let exact = decode_bool(row, exact_index, SchemaEpochOperation::Classify)?;
    Ok(match (exists, exact) {
        (false, false) => SchemaEpochRelation::Absent,
        (true, true) => SchemaEpochRelation::ExactTable,
        _ => SchemaEpochRelation::WrongShape,
    })
}

fn verify_fresh_epoch(client: &mut Client) -> Result<(), SchemaEpochError> {
    verify_authority_sentinels_client(client)?;
    let actual = catalog_census_under_lock(client, true).map_err(SchemaEpochError::Census)?;
    compare_fresh_census(actual.as_slice())
}

fn compare_fresh_census(actual: &[CatalogCensusEntry]) -> Result<(), SchemaEpochError> {
    let without_intel = parse_catalog_census(FRESH_CENSUS)?;
    let Err(without_intel) = compare_catalog_census(&without_intel, actual) else {
        return Ok(());
    };
    let with_intel = parse_catalog_census(FRESH_CENSUS_WITH_INTEL)?;
    match compare_catalog_census(&with_intel, actual) {
        Ok(()) => Ok(()),
        Err(with_intel) => Err(SchemaEpochError::FreshCensusMismatch {
            without_intel: Box::new(without_intel),
            with_intel: Box::new(with_intel),
        }),
    }
}

fn verify_v1_prefix_client(client: &mut Client) -> Result<(), SchemaEpochError> {
    verify_epoch_shape_client(client, EPOCH_V1_SHAPE_SQL)?;
    verify_post_epoch_census_client(client, SchemaEpochPrefix::V1)
}

fn verify_v1_prefix_transaction(transaction: &mut Transaction<'_>) -> Result<(), SchemaEpochError> {
    verify_epoch_shape_transaction(transaction, EPOCH_V1_SHAPE_SQL)?;
    verify_post_epoch_census_transaction(transaction, SchemaEpochPrefix::V1)
}

fn verify_v2_prefix_client(client: &mut Client) -> Result<(), SchemaEpochError> {
    verify_epoch_shape_client(client, EPOCH_V2_SHAPE_SQL)?;
    verify_post_epoch_census_client(client, SchemaEpochPrefix::V2)
}

fn verify_v2_prefix_transaction(transaction: &mut Transaction<'_>) -> Result<(), SchemaEpochError> {
    verify_epoch_shape_transaction(transaction, EPOCH_V2_SHAPE_SQL)?;
    verify_post_epoch_census_transaction(transaction, SchemaEpochPrefix::V2)
}

fn verify_v3_prefix_client(client: &mut Client) -> Result<(), SchemaEpochError> {
    verify_epoch_shape_client(client, EPOCH_V3_SHAPE_SQL)?;
    verify_post_epoch_census_client(client, SchemaEpochPrefix::V3)
}

fn verify_v3_prefix_transaction(transaction: &mut Transaction<'_>) -> Result<(), SchemaEpochError> {
    verify_epoch_shape_transaction(transaction, EPOCH_V3_SHAPE_SQL)?;
    verify_post_epoch_census_transaction(transaction, SchemaEpochPrefix::V3)
}

fn verify_v4_prefix_client(client: &mut Client) -> Result<(), SchemaEpochError> {
    verify_epoch_shape_client(client, EPOCH_V4_SHAPE_SQL)?;
    verify_post_epoch_census_client(client, SchemaEpochPrefix::V4)
}

fn verify_v4_prefix_transaction(transaction: &mut Transaction<'_>) -> Result<(), SchemaEpochError> {
    verify_epoch_shape_transaction(transaction, EPOCH_V4_SHAPE_SQL)?;
    verify_post_epoch_census_transaction(transaction, SchemaEpochPrefix::V4)
}

fn verify_v5_prefix_client(client: &mut Client) -> Result<(), SchemaEpochError> {
    verify_epoch_shape_client(client, EPOCH_V5_SHAPE_SQL)?;
    verify_post_epoch_census_client(client, SchemaEpochPrefix::V5)
}

fn verify_v5_prefix_transaction(transaction: &mut Transaction<'_>) -> Result<(), SchemaEpochError> {
    verify_epoch_shape_transaction(transaction, EPOCH_V5_SHAPE_SQL)?;
    verify_post_epoch_census_transaction(transaction, SchemaEpochPrefix::V5)
}

fn verify_v6_prefix_client(client: &mut Client) -> Result<(), SchemaEpochError> {
    verify_epoch_shape_client(client, EPOCH_V6_SHAPE_SQL)?;
    verify_post_epoch_census_client(client, SchemaEpochPrefix::V6)
}

fn verify_v6_prefix_transaction(transaction: &mut Transaction<'_>) -> Result<(), SchemaEpochError> {
    verify_epoch_shape_transaction(transaction, EPOCH_V6_SHAPE_SQL)?;
    verify_post_epoch_census_transaction(transaction, SchemaEpochPrefix::V6)
}

fn verify_v7_prefix_client(client: &mut Client) -> Result<(), SchemaEpochError> {
    verify_epoch_shape_client(client, EPOCH_V6_SHAPE_SQL)?;
    verify_epoch_shape_client(client, EPOCH_V7_SHAPE_SQL)?;
    verify_post_epoch_census_client(client, SchemaEpochPrefix::V7)
}

fn verify_v7_prefix_transaction(transaction: &mut Transaction<'_>) -> Result<(), SchemaEpochError> {
    verify_epoch_shape_transaction(transaction, EPOCH_V6_SHAPE_SQL)?;
    verify_epoch_shape_transaction(transaction, EPOCH_V7_SHAPE_SQL)?;
    verify_post_epoch_census_transaction(transaction, SchemaEpochPrefix::V7)
}

fn verify_post_epoch_census_client(
    client: &mut Client,
    prefix: SchemaEpochPrefix,
) -> Result<(), SchemaEpochError> {
    verify_authority_sentinels_client(client)?;
    let actual = catalog_census_under_lock(client, false).map_err(SchemaEpochError::Census)?;
    verify_post_epoch_census(actual.as_slice(), prefix)
}

fn verify_post_epoch_census_transaction(
    transaction: &mut Transaction<'_>,
    prefix: SchemaEpochPrefix,
) -> Result<(), SchemaEpochError> {
    verify_authority_sentinels_transaction(transaction)?;
    let actual = read_census_rows(transaction).map_err(SchemaEpochError::Census)?;
    verify_post_epoch_census(actual.as_slice(), prefix)
}

fn verify_post_epoch_census(
    actual: &[CatalogCensusEntry],
    prefix: SchemaEpochPrefix,
) -> Result<(), SchemaEpochError> {
    let mut baseline = Vec::with_capacity(actual.len());
    let mut epoch = Vec::with_capacity(25);
    for entry in actual.iter().take(MAX_CATALOG_CENSUS_ROWS) {
        if is_epoch_entry(entry, prefix) {
            epoch.push(entry.clone());
        } else {
            baseline.push(entry.clone());
        }
    }
    let epoch_fixture = match prefix {
        SchemaEpochPrefix::V1 => EPOCH_OWNED_FRESH_CENSUS_V1,
        SchemaEpochPrefix::V2 => EPOCH_OWNED_FRESH_CENSUS_V2,
        SchemaEpochPrefix::V3 => EPOCH_OWNED_FRESH_CENSUS_V3,
        SchemaEpochPrefix::V4 => EPOCH_OWNED_FRESH_CENSUS_V4,
        SchemaEpochPrefix::V5 => EPOCH_OWNED_FRESH_CENSUS_V5,
        SchemaEpochPrefix::V6 => EPOCH_OWNED_FRESH_CENSUS_V6,
        SchemaEpochPrefix::V7 => EPOCH_OWNED_FRESH_CENSUS_V7,
    };
    let expected_epoch = parse_catalog_census(epoch_fixture)?;
    compare_catalog_census(&expected_epoch, epoch.as_slice())
        .map_err(|_| SchemaEpochError::EpochCensusMismatch)?;
    compare_fresh_census(baseline.as_slice())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SchemaEpochPrefix {
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
}

fn is_epoch_entry(entry: &CatalogCensusEntry, prefix: SchemaEpochPrefix) -> bool {
    let key = entry.key();
    let migration_relation = key.kind() == CatalogObjectKind::Relation
        && key.schema() == "babylon_state"
        && key.name() == "schema_migration";
    let h3_relation = matches!(
        prefix,
        SchemaEpochPrefix::V2
            | SchemaEpochPrefix::V3
            | SchemaEpochPrefix::V4
            | SchemaEpochPrefix::V5
            | SchemaEpochPrefix::V6
            | SchemaEpochPrefix::V7
    ) && key.kind() == CatalogObjectKind::Relation
        && key.schema() == "babylon_ref"
        && key.name() == "h3_cell";
    let h3_cohort_relation = matches!(
        prefix,
        SchemaEpochPrefix::V3
            | SchemaEpochPrefix::V4
            | SchemaEpochPrefix::V5
            | SchemaEpochPrefix::V6
            | SchemaEpochPrefix::V7
    ) && key.kind() == CatalogObjectKind::Relation
        && key.schema() == "babylon_ref"
        && matches!(
            key.name(),
            "h3_reference_cohort" | "h3_reference_membership"
        );
    let committed_tick_relation = matches!(
        prefix,
        SchemaEpochPrefix::V4
            | SchemaEpochPrefix::V5
            | SchemaEpochPrefix::V6
            | SchemaEpochPrefix::V7
    ) && key.kind() == CatalogObjectKind::Relation
        && key.schema() == "babylon_state"
        && matches!(
            key.name(),
            "campaign"
                | "tick_commit"
                | "tick_graph_row"
                | "tick_state_row"
                | "tick_event_row"
                | "tick_subsystem_row"
                | "tick_conservation_row"
                | "tick_boundary_flow_row"
                | "tick_checkpoint_row"
                | "tick_archive_dirty_receipt_row"
        );
    let owned_schema = key.kind() == CatalogObjectKind::Schema
        && key.schema() == "pg_namespace"
        && matches!(key.name(), "babylon_ref" | "babylon_state");
    let fresh_meta = key.kind() == CatalogObjectKind::SchemaGrant
        && key.schema() == "pg_namespace"
        && key.name() == "babylon_meta";
    let spatial_reference_relation = matches!(
        prefix,
        SchemaEpochPrefix::V5 | SchemaEpochPrefix::V6 | SchemaEpochPrefix::V7
    ) && key.kind() == CatalogObjectKind::Relation
        && key.schema() == "babylon_ref"
        && matches!(
            key.name(),
            "reference_product"
                | "county_identity"
                | "place_identity"
                | "h3_land_fraction"
                | "h3_population_count"
                | "h3_workplace_count"
                | "county_h3_land_area"
                | "county_place_h3_land_area"
        );
    migration_relation
        || h3_relation
        || h3_cohort_relation
        || committed_tick_relation
        || spatial_reference_relation
        || owned_schema
        || fresh_meta
}

fn verify_authority_sentinels_client(client: &mut Client) -> Result<(), SchemaEpochError> {
    let row = client
        .query_one(FRESH_SENTINELS_SQL, &[])
        .map_err(|error| postgres_database_error(SchemaEpochOperation::FreshSentinels, &error))?;
    require_authority_sentinels(&row)
}

fn verify_authority_sentinels_transaction(
    transaction: &mut Transaction<'_>,
) -> Result<(), SchemaEpochError> {
    let row = transaction
        .query_one(FRESH_SENTINELS_SQL, &[])
        .map_err(|error| postgres_database_error(SchemaEpochOperation::FreshSentinels, &error))?;
    require_authority_sentinels(&row)
}

fn require_authority_sentinels(row: &Row) -> Result<(), SchemaEpochError> {
    let mut clean = true;
    for index in 0..3 {
        clean &= decode_bool(row, index, SchemaEpochOperation::FreshSentinels)?;
    }
    if clean {
        Ok(())
    } else {
        Err(SchemaEpochError::AuthoritySentinelResidue)
    }
}

fn read_ledger(
    client: &mut Client,
    compiled_count: usize,
) -> Result<Vec<PersistedMigration>, SchemaEpochError> {
    let bounded_count = compiled_count.min(MAX_SCHEMA_MIGRATIONS);
    let query_count = bounded_count
        .checked_add(1)
        .expect("migration bound cannot wrap");
    let limit = i64::try_from(query_count).expect("ledger bound fits i64");
    let rows = client
        .query(LEDGER_SQL, &[&limit])
        .map_err(|error| postgres_database_error(SchemaEpochOperation::ReadLedger, &error))?;
    decode_ledger_rows(rows.as_slice(), query_count)
}

fn decode_ledger_rows(
    rows: &[Row],
    query_count: usize,
) -> Result<Vec<PersistedMigration>, SchemaEpochError> {
    let mut persisted = Vec::with_capacity(rows.len().min(query_count));
    for row in rows.iter().take(query_count) {
        let version = row
            .try_get::<_, i64>(0)
            .map_err(|error| postgres_database_error(SchemaEpochOperation::ReadLedger, &error))?;
        let checksum = row
            .try_get::<_, Vec<u8>>(1)
            .map_err(|error| postgres_database_error(SchemaEpochOperation::ReadLedger, &error))?;
        let decoded = PersistedMigration::from_database(version, checksum.as_slice())
            .map_err(|_| database_error(SchemaEpochOperation::ReadLedger))?;
        persisted.push(decoded);
    }
    Ok(persisted)
}

fn attempt_migration(
    client: &mut Client,
    migration: SchemaEpochMigration,
) -> Result<MigrationAttempt, SchemaEpochError> {
    let mut transaction = begin_migration_transaction(client)?;
    execute_migration_before_marker(&mut transaction, migration)?;
    insert_ledger_marker(&mut transaction, migration.migration)?;
    commit_migration(transaction)
}

fn commit_migration(transaction: Transaction<'_>) -> Result<MigrationAttempt, SchemaEpochError> {
    match transaction.commit() {
        Ok(()) => Ok(MigrationAttempt::Committed),
        Err(error) if error.as_db_error().is_some() => Err(postgres_database_error(
            SchemaEpochOperation::CommitMigration,
            &error,
        )),
        Err(_) => Ok(MigrationAttempt::Ambiguous),
    }
}

fn begin_migration_transaction(client: &mut Client) -> Result<Transaction<'_>, SchemaEpochError> {
    client
        .build_transaction()
        .isolation_level(IsolationLevel::Serializable)
        .read_only(false)
        .start()
        .map_err(|error| postgres_database_error(SchemaEpochOperation::BeginMigration, &error))
}

fn execute_migration_before_marker(
    transaction: &mut Transaction<'_>,
    migration: SchemaEpochMigration,
) -> Result<(), SchemaEpochError> {
    prepare_migration_transaction(transaction)?;
    transaction
        .batch_execute(migration.migration.sql())
        .map_err(|error| postgres_database_error(SchemaEpochOperation::ExecuteMigration, &error))?;
    (migration.prefix_contract.verify_transaction)(transaction)
}

fn prepare_migration_transaction(
    transaction: &mut Transaction<'_>,
) -> Result<(), SchemaEpochError> {
    transaction
        .batch_execute(WRITE_LOCAL_SETTINGS_SQL)
        .map_err(|error| {
            postgres_database_error(SchemaEpochOperation::SetMigrationSettings, &error)
        })?;
    let row = transaction
        .query_one(WRITE_SETTINGS_SQL, &[])
        .map_err(|error| {
            postgres_database_error(SchemaEpochOperation::VerifyMigrationSettings, &error)
        })?;
    let expected = ["serializable", "off", "pg_catalog", "on", "5s", "5s", "5s"];
    for (index, value) in expected.iter().enumerate().take(7) {
        let actual = row.try_get::<_, String>(index).map_err(|error| {
            postgres_database_error(SchemaEpochOperation::VerifyMigrationSettings, &error)
        })?;
        if actual != *value {
            return Err(database_error(
                SchemaEpochOperation::VerifyMigrationSettings,
            ));
        }
    }
    Ok(())
}

fn insert_ledger_marker(
    transaction: &mut Transaction<'_>,
    migration: SchemaMigration,
) -> Result<(), SchemaEpochError> {
    let version = migration.version().as_i64();
    let migration_checksum = migration.checksum();
    let checksum: &[u8] = migration_checksum.as_bytes();
    let affected = transaction
        .execute(INSERT_LEDGER_SQL, &[&version, &checksum])
        .map_err(|error| postgres_database_error(SchemaEpochOperation::InsertLedger, &error))?;
    if affected == 1 {
        Ok(())
    } else {
        Err(SchemaEpochError::AffectedRowCount {
            operation: SchemaEpochOperation::InsertLedger,
            expected: 1,
            actual: affected,
        })
    }
}

fn verify_epoch_shape_client(client: &mut Client, shape_sql: &str) -> Result<(), SchemaEpochError> {
    let row = client
        .query_one(shape_sql, &[])
        .map_err(|error| postgres_database_error(SchemaEpochOperation::VerifyEpochShape, &error))?;
    require_epoch_shape(&row)
}

fn verify_epoch_shape_transaction(
    transaction: &mut Transaction<'_>,
    shape_sql: &str,
) -> Result<(), SchemaEpochError> {
    let row = transaction
        .query_one(shape_sql, &[])
        .map_err(|error| postgres_database_error(SchemaEpochOperation::VerifyEpochShape, &error))?;
    require_epoch_shape(&row)
}

fn require_epoch_shape(row: &Row) -> Result<(), SchemaEpochError> {
    if decode_bool(row, 0, SchemaEpochOperation::VerifyEpochShape)? {
        Ok(())
    } else {
        Err(SchemaEpochError::EpochShapeMismatch)
    }
}

fn decode_bool(
    row: &Row,
    index: usize,
    operation: SchemaEpochOperation,
) -> Result<bool, SchemaEpochError> {
    row.try_get(index)
        .map_err(|error| postgres_database_error(operation, &error))
}

fn database_error(operation: SchemaEpochOperation) -> SchemaEpochError {
    SchemaEpochError::Database {
        operation,
        diagnostic: None,
    }
}

fn postgres_database_error(
    operation: SchemaEpochOperation,
    error: &postgres::Error,
) -> SchemaEpochError {
    SchemaEpochError::Database {
        operation,
        diagnostic: Some(PostgresDiagnosticV1::capture(error)),
    }
}

#[cfg(test)]
mod pure_tests {
    use super::*;
    use crate::postgres_catalog::CatalogObjectKey;

    #[test]
    fn empty_and_current_authority_are_the_only_bootstrap_shapes() {
        let fresh = SchemaEpochObservation {
            schemas: SchemaEpochSchemas {
                babylon_ref: false,
                babylon_state: false,
                babylon_meta: false,
            },
            ledger: SchemaEpochRelation::Absent,
        };
        assert_eq!(classify_observation(fresh), Ok(SchemaEpochOrigin::Fresh));
        let partial = SchemaEpochObservation {
            schemas: SchemaEpochSchemas {
                babylon_ref: false,
                babylon_state: false,
                babylon_meta: true,
            },
            ledger: SchemaEpochRelation::Absent,
        };
        assert_eq!(
            classify_observation(partial),
            Err(SchemaEpochError::PartialAuthorityEpoch {
                observation: partial
            })
        );
        let current = SchemaEpochObservation {
            schemas: SchemaEpochSchemas {
                babylon_ref: true,
                babylon_state: true,
                babylon_meta: true,
            },
            ledger: SchemaEpochRelation::ExactTable,
        };
        assert_eq!(
            classify_observation(current),
            Ok(SchemaEpochOrigin::ExistingRustPrefix)
        );
        assert_eq!(
            require_recorded_rust_prefix(&[]),
            Err(SchemaEpochError::UnrecordedRustEpoch)
        );
    }

    #[test]
    fn unexpected_public_objects_cannot_enter_the_fresh_bootstrap() {
        let mut entries = parse_catalog_census(FRESH_CENSUS)
            .unwrap()
            .entries()
            .to_vec();
        let extra = CatalogObjectKey::new(
            CatalogObjectKind::Relation,
            "public",
            "unexpected_game_state",
        )
        .unwrap();
        entries.push(CatalogCensusEntry::new(extra, &"0".repeat(64)).unwrap());
        assert!(matches!(
            compare_fresh_census(&entries),
            Err(SchemaEpochError::FreshCensusMismatch { .. })
        ));
    }
}

#[cfg(test)]
mod live_rollback_tests {
    use std::str::FromStr;
    use std::time::Instant;

    use postgres::error::SqlState;

    use super::*;

    const DSN_ENV: &str = "BABYLON_POSTGRES_TEST_DSN";
    const ACK_ENV: &str = "BABYLON_POSTGRES_DISPOSABLE_ACK";
    const ACK: &str = "I_UNDERSTAND_THIS_DISPOSABLE_RUNTIME_DROPS_ITS_SCRATCH_DATABASES_AND_ROLES";
    const CANARY_ENV: &str = "BABYLON_POSTGRES_DISPOSABLE_CANARY";
    const BACKEND_TERMINATION_TIMEOUT_MILLIS: i64 = 5_000;

    #[test]
    #[ignore = "requires the task-owned disposable PostgreSQL runtime"]
    fn rollback_and_ambiguous_commit_reconciliation_are_atomic() {
        let base = validated_base_config();
        verify_unsupported_startup_setting_diagnostic(&base);
        verify_authentication_diagnostic(&base);
        verify_post_ddl_rollback(&base);
        verify_definite_commit_failure(&base);
        verify_killed_commit_retry(&base);
        verify_committed_reconciliation(&base);
        let spatial = TestDatabase::create(&base, "spatialproducts");
        let config = spatial.config(&base);
        migrate_schema_epoch(&config).unwrap();
        crate::spatial_reference_installer::live_postgres_tests::verify_commit_protocol(
            &config, &base,
        );
        spatial.cleanup();
    }

    fn verify_unsupported_startup_setting_diagnostic(base: &Config) {
        const SETTING_CANARY: &str = "per288_unsupported_setting_canary";
        const VALUE_CANARY: &str = "PER288_OPTION_VALUE_CANARY";
        let options = format!("-c {SETTING_CANARY}={VALUE_CANARY}");
        let config = bounded_config_with_options(base, &options);
        let Err(error) = LockedSession::connect(&config) else {
            panic!("the unsupported startup setting must be rejected");
        };
        let rendered = error.to_string();
        let SchemaEpochError::Database {
            operation: SchemaEpochOperation::Connect,
            diagnostic: Some(diagnostic),
        } = error
        else {
            panic!("startup refusal must retain its connect diagnostic");
        };

        assert_eq!(
            diagnostic.classification(),
            crate::PostgresFailureClassV1::UnsupportedStartupSetting
        );
        assert!(diagnostic.sqlstate().is_some_and(|code| code.len() == 5));
        assert_eq!(
            diagnostic.message(),
            Some("unrecognized configuration parameter <redacted>")
        );
        assert!(rendered.contains("UnsupportedStartupSetting"));
        assert!(!rendered.contains(SETTING_CANARY));
        assert!(!rendered.contains(VALUE_CANARY));
    }

    fn verify_authentication_diagnostic(base: &Config) {
        const PASSWORD_CANARY: &str = "PER288_WRONG_PASSWORD_CANARY";
        let mut config = base.clone();
        config.password(PASSWORD_CANARY);
        let Err(error) = LockedSession::connect(&config) else {
            panic!("the wrong password must be rejected");
        };
        let rendered = error.to_string();
        let SchemaEpochError::Database {
            operation: SchemaEpochOperation::Connect,
            diagnostic: Some(diagnostic),
        } = error
        else {
            panic!("authentication refusal must retain its connect diagnostic");
        };

        assert_eq!(
            diagnostic.classification(),
            crate::PostgresFailureClassV1::Authentication
        );
        assert_eq!(
            diagnostic.sqlstate(),
            Some(SqlState::INVALID_PASSWORD.code())
        );
        assert_eq!(diagnostic.message(), Some("authentication rejected"));
        assert!(rendered.contains("Authentication"));
        assert!(!rendered.contains(PASSWORD_CANARY));
    }

    #[test]
    #[ignore = "requires the task-owned disposable PostgreSQL runtime"]
    fn h3_installer_rollback_and_ambiguous_commit_reconciliation_are_atomic() {
        let base = validated_base_config();
        verify_h3_installer_commit_protocol(&base);
    }

    #[test]
    #[ignore = "requires the task-owned disposable PostgreSQL runtime"]
    fn h3_installer_membership_cardinality_is_bounded() {
        let base = validated_base_config();
        let database = TestDatabase::create(&base, "hcardinality");
        let config = database.config(&base);
        assert_eq!(
            migrate_schema_epoch(&config).unwrap().final_applied,
            CURRENT_SCHEMA_EPOCH
        );
        crate::h3_reference_installer::live_postgres_tests::verify_membership_cardinality_bound(
            &config,
        );
        database.cleanup();
    }

    fn verify_h3_installer_commit_protocol(base: &Config) {
        let suite_started = Instant::now();
        let rollback_retry = TestDatabase::create(base, "hrollback");
        let rollback_config = rollback_retry.config(base);
        assert_eq!(
            migrate_schema_epoch(&rollback_config)
                .unwrap()
                .final_applied,
            CURRENT_SCHEMA_EPOCH
        );
        crate::h3_reference_installer::live_postgres_tests::verify_rollback_and_killed_retry(
            &rollback_config,
            base,
            suite_started,
        );
        rollback_retry.cleanup();

        let reconciliation = TestDatabase::create(base, "hreconcile");
        let reconciliation_config = reconciliation.config(base);
        assert_eq!(
            migrate_schema_epoch(&reconciliation_config)
                .unwrap()
                .final_applied,
            CURRENT_SCHEMA_EPOCH
        );
        crate::h3_reference_installer::live_postgres_tests::verify_committed_reconciliation(
            &reconciliation_config,
            base,
            suite_started,
        );
        reconciliation.cleanup();
    }

    fn verify_post_ddl_rollback(base: &Config) {
        let database = TestDatabase::create(base, "rollback");
        let config = database.config(base);
        let compiled = compiled_schema_epoch_migrations().unwrap();
        let bounded = bounded_config(&config);
        let mut session = LockedSession::connect(&bounded).unwrap();
        let initial = inspect_epoch(session.client(), &compiled).unwrap();
        assert_eq!(initial.origin, SchemaEpochOrigin::Fresh);

        let mut transaction = begin_migration_transaction(session.client()).unwrap();
        execute_migration_before_marker(&mut transaction, compiled[0]).unwrap();
        let failure = transaction.batch_execute("SELECT 1 / 0").unwrap_err();
        assert_eq!(failure.code(), Some(&SqlState::DIVISION_BY_ZERO));
        transaction.rollback().unwrap();
        session.finish(Ok(())).unwrap();

        let mut client = config.connect(NoTls).unwrap();
        assert_eq!(read_observation(&mut client).unwrap(), empty_observation());
        drop(client);
        let retry = migrate_schema_epoch(&config).unwrap();
        assert_eq!(retry.origin, SchemaEpochOrigin::Fresh);
        assert_eq!(retry.final_applied, CURRENT_SCHEMA_EPOCH);
        assert_eq!(retry.applied_versions.len(), CURRENT_SCHEMA_EPOCH);
        database.cleanup();
    }

    fn verify_definite_commit_failure(base: &Config) {
        let database = TestDatabase::create(base, "commiterror");
        let config = database.config(base);
        let compiled = compiled_schema_epoch_migrations().unwrap();
        let bounded = bounded_config(&config);
        let mut session = LockedSession::connect(&bounded).unwrap();
        let mut transaction = begin_migration_transaction(session.client()).unwrap();
        execute_migration_before_marker(&mut transaction, compiled[0]).unwrap();
        transaction
            .batch_execute(
                "CREATE TEMP TABLE deferred_collision (\
                    value INTEGER, UNIQUE (value) DEFERRABLE INITIALLY DEFERRED\
                 ); \
                 INSERT INTO deferred_collision (value) VALUES (1), (1)",
            )
            .unwrap();
        insert_ledger_marker(&mut transaction, compiled[0].migration).unwrap();
        let Err(SchemaEpochError::Database {
            operation: SchemaEpochOperation::CommitMigration,
            diagnostic: Some(diagnostic),
        }) = commit_migration(transaction)
        else {
            panic!("definite commit failure must retain its server diagnostic");
        };
        assert_eq!(diagnostic.sqlstate(), Some("23505"));
        assert_eq!(
            diagnostic.classification(),
            crate::PostgresFailureClassV1::ServerRejected
        );
        session.finish(Ok(())).unwrap();

        let mut client = config.connect(NoTls).unwrap();
        assert_eq!(read_observation(&mut client).unwrap(), empty_observation());
        database.cleanup();
    }

    fn verify_killed_commit_retry(base: &Config) {
        let database = TestDatabase::create(base, "killed");
        let config = database.config(base);
        let bounded = bounded_config(&config);
        let compiled = compiled_schema_epoch_migrations().unwrap();
        let mut session = LockedSession::connect(&bounded).unwrap();
        let mut report = empty_report();
        let mut first_attempt = true;
        let mut attempt = |client: &mut Client, migration: SchemaEpochMigration| {
            if first_attempt {
                first_attempt = false;
                killed_before_commit_attempt(client, migration, base)
            } else {
                attempt_migration(client, migration)
            }
        };
        apply_with_reconciliation_using(
            &bounded,
            &mut session,
            &compiled,
            compiled[0],
            &mut report,
            &mut attempt,
        )
        .unwrap();
        session.finish(Ok(())).unwrap();
        assert_eq!(
            report.applied_versions,
            vec![compiled[0].migration.version()]
        );
        assert!(report.reconciled_versions.is_empty());
        let completed = migrate_schema_epoch(&config).unwrap();
        assert_eq!(completed.prior_applied, 1);
        assert_eq!(completed.final_applied, CURRENT_SCHEMA_EPOCH);
        database.cleanup();
    }

    fn verify_committed_reconciliation(base: &Config) {
        let database = TestDatabase::create(base, "reconciled");
        let config = database.config(base);
        let bounded = bounded_config(&config);
        let compiled = compiled_schema_epoch_migrations().unwrap();
        let mut session = LockedSession::connect(&bounded).unwrap();
        let mut report = empty_report();
        let mut first_attempt = true;
        let mut attempt = |client: &mut Client, migration| {
            let outcome = attempt_migration(client, migration)?;
            if first_attempt {
                first_attempt = false;
                assert_eq!(outcome, MigrationAttempt::Committed);
                Ok(MigrationAttempt::Ambiguous)
            } else {
                Ok(outcome)
            }
        };
        apply_with_reconciliation_using(
            &bounded,
            &mut session,
            &compiled,
            compiled[0],
            &mut report,
            &mut attempt,
        )
        .unwrap();
        session.finish(Ok(())).unwrap();
        assert!(report.applied_versions.is_empty());
        assert_eq!(
            report.reconciled_versions,
            vec![compiled[0].migration.version()]
        );
        let completed = migrate_schema_epoch(&config).unwrap();
        assert_eq!(completed.prior_applied, 1);
        assert_eq!(completed.final_applied, CURRENT_SCHEMA_EPOCH);
        database.cleanup();
    }

    fn killed_before_commit_attempt(
        client: &mut Client,
        migration: SchemaEpochMigration,
        admin: &Config,
    ) -> Result<MigrationAttempt, SchemaEpochError> {
        let backend_pid: i32 = client
            .query_one("SELECT pg_catalog.pg_backend_pid()", &[])
            .unwrap()
            .try_get(0)
            .unwrap();
        let mut transaction = begin_migration_transaction(client)?;
        execute_migration_before_marker(&mut transaction, migration)?;
        insert_ledger_marker(&mut transaction, migration.migration)?;
        let terminated: bool = admin
            .connect(NoTls)
            .unwrap()
            .query_one(
                "SELECT pg_catalog.pg_terminate_backend($1, $2)",
                &[&backend_pid, &BACKEND_TERMINATION_TIMEOUT_MILLIS],
            )
            .unwrap()
            .try_get(0)
            .unwrap();
        assert!(terminated);
        assert!(transaction.commit().is_err());
        Ok(MigrationAttempt::Ambiguous)
    }

    fn empty_report() -> SchemaEpochReport {
        SchemaEpochReport {
            origin: SchemaEpochOrigin::Fresh,
            prior_applied: 0,
            final_applied: 0,
            applied_versions: Vec::new(),
            reconciled_versions: Vec::new(),
        }
    }

    fn validated_base_config() -> Config {
        assert_eq!(std::env::var(ACK_ENV).as_deref(), Ok(ACK));
        let canary = std::env::var(CANARY_ENV).expect("runner supplies the disposable canary");
        assert_eq!(canary.len(), 32);
        let dsn = std::env::var(DSN_ENV).expect("runner supplies the disposable DSN");
        let config = Config::from_str(&dsn).expect("runner DSN parses");
        validate_connection_target(&config).unwrap();
        assert_eq!(config.get_user(), Some("test"));
        assert_eq!(config.get_dbname(), Some("postgres"));
        let mut client = config.connect(NoTls).unwrap();
        let actual: Option<String> = client
            .query_one(
                "SELECT pg_catalog.current_setting('babylon.disposable_runtime', true)",
                &[],
            )
            .unwrap()
            .try_get(0)
            .unwrap();
        assert_eq!(actual.as_deref(), Some(canary.as_str()));
        config
    }

    fn empty_observation() -> SchemaEpochObservation {
        SchemaEpochObservation {
            schemas: SchemaEpochSchemas {
                babylon_ref: false,
                babylon_state: false,
                babylon_meta: false,
            },
            ledger: SchemaEpochRelation::Absent,
        }
    }

    struct TestDatabase {
        name: String,
        admin: Config,
        active: bool,
    }

    impl TestDatabase {
        fn create(base: &Config, label: &str) -> Self {
            assert!(label.bytes().all(|byte| byte.is_ascii_lowercase()));
            let name = format!("native_epoch_{label}_{}", std::process::id());
            let mut admin = base.clone();
            admin.dbname("postgres");
            let sql = format!("CREATE DATABASE \"{name}\" OWNER test TEMPLATE template1");
            admin.connect(NoTls).unwrap().batch_execute(&sql).unwrap();
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
            match self.try_drop_database() {
                Ok(()) => self.active = false,
                Err(()) => panic!("schema-epoch test database cleanup must succeed"),
            }
        }

        fn try_drop_database(&self) -> Result<(), ()> {
            let sql = format!("DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)", self.name);
            self.admin
                .connect(NoTls)
                .map_err(|_| ())?
                .batch_execute(&sql)
                .map_err(|_| ())
        }
    }

    impl Drop for TestDatabase {
        fn drop(&mut self) {
            if !self.active {
                return;
            }
            if std::thread::panicking() {
                let _unwind_cleanup = self.try_drop_database();
                return;
            }
            match self.try_drop_database() {
                Ok(()) => self.active = false,
                Err(()) => panic!("schema-epoch test database cleanup failed"),
            }
        }
    }
}
