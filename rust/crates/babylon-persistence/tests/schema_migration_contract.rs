//! Pure contracts for one canonical Rust schema migration.

#[path = "../src/schema_migration.rs"]
mod schema_migration;

use std::any::TypeId;

use babylon_persistence::MigrationSetDigest;
use schema_migration::{
    MigrationChecksum, MigrationVersion, SchemaMigration, SchemaMigrationError,
    MAX_SCHEMA_MIGRATION_SQL_BYTES, MIGRATION_CHECKSUM_BYTES,
};

#[test]
fn migration_version_is_positive_and_advances_without_wrapping() {
    assert_eq!(
        MigrationVersion::try_from(0),
        Err(SchemaMigrationError::NonPositiveVersion { value: 0 })
    );
    assert_eq!(
        MigrationVersion::try_from(-1),
        Err(SchemaMigrationError::NonPositiveVersion { value: -1 })
    );

    let first = MigrationVersion::try_from(1).expect("one is the first valid version");
    assert_eq!(first.as_i64(), 1);
    assert_eq!(first.checked_next().unwrap().as_i64(), 2);

    let maximum = MigrationVersion::try_from(i64::MAX).expect("i64::MAX remains positive");
    assert_eq!(
        maximum.checked_next(),
        Err(SchemaMigrationError::VersionOverflow { current: i64::MAX })
    );
}

#[test]
fn database_checksum_bytes_require_the_exact_sha256_width() {
    let mut database_bytes = [0x5a; MIGRATION_CHECKSUM_BYTES];
    let checksum = MigrationChecksum::from_database_bytes(&database_bytes)
        .expect("32 database bytes are one SHA-256 value");
    database_bytes[0] = 0;

    assert_eq!(database_bytes[0], 0);
    assert_eq!(checksum.as_bytes(), &[0x5a; MIGRATION_CHECKSUM_BYTES]);
    assert_eq!(
        MigrationChecksum::from_database_bytes(&[0; MIGRATION_CHECKSUM_BYTES - 1]),
        Err(SchemaMigrationError::InvalidChecksumLength {
            actual: MIGRATION_CHECKSUM_BYTES - 1,
            expected: MIGRATION_CHECKSUM_BYTES,
        })
    );
    assert_eq!(
        MigrationChecksum::from_database_bytes(&[0; MIGRATION_CHECKSUM_BYTES + 1]),
        Err(SchemaMigrationError::InvalidChecksumLength {
            actual: MIGRATION_CHECKSUM_BYTES + 1,
            expected: MIGRATION_CHECKSUM_BYTES,
        })
    );
    assert_ne!(
        TypeId::of::<MigrationChecksum>(),
        TypeId::of::<MigrationSetDigest>()
    );
}

#[test]
fn schema_migration_binds_version_exact_sql_and_its_internal_checksum() {
    const SQL: &str = "SELECT 1;\n";
    const EXPECTED_SHA256: [u8; MIGRATION_CHECKSUM_BYTES] = [
        0xb4, 0xe0, 0x49, 0x78, 0x04, 0xe4, 0x6e, 0x0a, 0x0b, 0x0b, 0x8c, 0x31, 0x97, 0x5b, 0x06,
        0x21, 0x52, 0xd5, 0x51, 0xba, 0xc4, 0x9c, 0x3c, 0x2e, 0x80, 0x93, 0x25, 0x67, 0xb4, 0x08,
        0x5d, 0xcd,
    ];
    let version = MigrationVersion::try_from(7).unwrap();
    let migration = SchemaMigration::new(version, SQL).expect("canonical SQL is valid");

    assert_eq!(migration.version(), version);
    assert_eq!(migration.sql(), SQL);
    assert_eq!(migration.checksum().as_bytes(), &EXPECTED_SHA256);
}

#[test]
fn schema_migration_rejects_each_malformed_sql_shape() {
    let version = MigrationVersion::try_from(1).unwrap();
    assert_eq!(
        SchemaMigration::new(version, ""),
        Err(SchemaMigrationError::EmptySql)
    );
    assert_eq!(
        SchemaMigration::new(version, "SELECT 1;"),
        Err(SchemaMigrationError::MissingFinalNewline)
    );
    assert_eq!(
        SchemaMigration::new(version, "SELECT\0 1;\n"),
        Err(SchemaMigrationError::EmbeddedNul { byte_index: 6 })
    );
}

#[test]
fn schema_migration_sql_scan_is_bounded_before_validation_or_hashing() {
    let version = MigrationVersion::try_from(1).unwrap();
    let exact_limit = leak_sql_with_final_newline(MAX_SCHEMA_MIGRATION_SQL_BYTES);
    assert!(SchemaMigration::new(version, exact_limit).is_ok());

    let too_long = leak_sql_with_final_newline(MAX_SCHEMA_MIGRATION_SQL_BYTES + 1);
    assert_eq!(
        SchemaMigration::new(version, too_long),
        Err(SchemaMigrationError::SqlTooLong {
            actual: MAX_SCHEMA_MIGRATION_SQL_BYTES + 1,
            max: MAX_SCHEMA_MIGRATION_SQL_BYTES,
        })
    );
}

fn leak_sql_with_final_newline(byte_length: usize) -> &'static str {
    let body_length = byte_length
        .checked_sub(1)
        .expect("the migration SQL test lengths include a final newline");
    let mut sql = "x".repeat(body_length);
    sql.push('\n');
    Box::leak(sql.into_boxed_str())
}
