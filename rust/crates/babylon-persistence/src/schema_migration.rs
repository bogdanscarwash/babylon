//! Nominal, bounded contracts for one canonical Rust schema migration.

use babylon_kernel::sha256_of;

/// SHA-256 byte width stored by `babylon_state.schema_migration`.
pub const MIGRATION_CHECKSUM_BYTES: usize = 32;
/// Hard ceiling on the exact SQL bytes in one compiled migration.
pub const MAX_SCHEMA_MIGRATION_SQL_BYTES: usize = 1_048_576;

/// Invalid schema-migration input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaMigrationError {
    /// A migration version was zero or negative.
    NonPositiveVersion { value: i64 },
    /// Advancing the largest representable migration version would wrap.
    VersionOverflow { current: i64 },
    /// The exact SQL source was empty.
    EmptySql,
    /// The exact SQL source exceeded its fixed byte ceiling.
    SqlTooLong { actual: usize, max: usize },
    /// The exact SQL source did not end in one newline byte.
    MissingFinalNewline,
    /// The exact SQL source contained a NUL byte.
    EmbeddedNul { byte_index: usize },
    /// A database checksum did not have the exact SHA-256 width.
    InvalidChecksumLength { actual: usize, expected: usize },
}

impl std::fmt::Display for SchemaMigrationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid schema migration: {self:?}")
    }
}

impl std::error::Error for SchemaMigrationError {}

/// Positive ordinal stored in the migration ledger's `BIGINT` key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct MigrationVersion(i64);

impl MigrationVersion {
    /// Return the checked signed value used at the `PostgreSQL` boundary.
    #[must_use]
    pub fn as_i64(self) -> i64 {
        self.0
    }

    /// Advance by one without wrapping the signed database representation.
    ///
    /// # Errors
    /// Returns [`SchemaMigrationError::VersionOverflow`] at `i64::MAX`.
    pub fn checked_next(self) -> Result<Self, SchemaMigrationError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(SchemaMigrationError::VersionOverflow { current: self.0 })
    }
}

impl TryFrom<i64> for MigrationVersion {
    type Error = SchemaMigrationError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value <= 0 {
            return Err(SchemaMigrationError::NonPositiveVersion { value });
        }
        Ok(Self(value))
    }
}

/// SHA-256 over the exact bytes of one checked-in migration SQL file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct MigrationChecksum([u8; MIGRATION_CHECKSUM_BYTES]);

impl MigrationChecksum {
    /// Decode a checksum returned from the ledger's bounded `BYTEA` column.
    ///
    /// # Errors
    /// Returns [`SchemaMigrationError::InvalidChecksumLength`] unless `bytes`
    /// contains exactly one SHA-256 value.
    pub fn from_database_bytes(bytes: &[u8]) -> Result<Self, SchemaMigrationError> {
        let value = <[u8; MIGRATION_CHECKSUM_BYTES]>::try_from(bytes).map_err(|_| {
            SchemaMigrationError::InvalidChecksumLength {
                actual: bytes.len(),
                expected: MIGRATION_CHECKSUM_BYTES,
            }
        })?;
        Ok(Self(value))
    }

    /// Return the canonical SHA-256 bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; MIGRATION_CHECKSUM_BYTES] {
        &self.0
    }

    fn for_sql(sql: &str) -> Self {
        Self(sha256_of(sql.as_bytes()))
    }
}

/// One compiled migration with its checksum derived from the exact SQL bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaMigration {
    version: MigrationVersion,
    sql: &'static str,
    checksum: MigrationChecksum,
}

impl SchemaMigration {
    /// Validate and bind one compiled migration.
    ///
    /// # Errors
    /// Returns [`SchemaMigrationError`] when the SQL is empty, oversized,
    /// unterminated, or contains a NUL byte.
    pub fn new(version: MigrationVersion, sql: &'static str) -> Result<Self, SchemaMigrationError> {
        validate_sql(sql)?;
        Ok(Self {
            version,
            sql,
            checksum: MigrationChecksum::for_sql(sql),
        })
    }

    /// Return the positive ledger version.
    #[must_use]
    pub fn version(&self) -> MigrationVersion {
        self.version
    }

    /// Return the exact checked-in SQL source.
    #[must_use]
    pub fn sql(&self) -> &'static str {
        self.sql
    }

    /// Return the internally computed checksum.
    #[must_use]
    pub fn checksum(&self) -> MigrationChecksum {
        self.checksum
    }
}

fn validate_sql(sql: &str) -> Result<(), SchemaMigrationError> {
    if sql.is_empty() {
        return Err(SchemaMigrationError::EmptySql);
    }
    if sql.len() > MAX_SCHEMA_MIGRATION_SQL_BYTES {
        return Err(SchemaMigrationError::SqlTooLong {
            actual: sql.len(),
            max: MAX_SCHEMA_MIGRATION_SQL_BYTES,
        });
    }
    if let Some(byte_index) = sql
        .bytes()
        .take(MAX_SCHEMA_MIGRATION_SQL_BYTES + 1)
        .position(|byte| byte == 0)
    {
        return Err(SchemaMigrationError::EmbeddedNul { byte_index });
    }
    if !sql.ends_with('\n') {
        return Err(SchemaMigrationError::MissingFinalNewline);
    }
    Ok(())
}
