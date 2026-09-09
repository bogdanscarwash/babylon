//! Pure contracts for the bounded Rust schema epoch.

use babylon_persistence::{
    compiled_schema_migrations, validate_migration_prefix, MigrationVersion, PersistedMigration,
    SchemaEpochError, SchemaMigration, MAX_SCHEMA_MIGRATIONS,
};

const SQL_ONE: &str = "SELECT 1;\n";
const SQL_TWO: &str = "SELECT 2;\n";

#[test]
fn compiled_registry_is_seven_contiguous_exact_migrations() {
    let compiled = compiled_schema_migrations().expect("checked-in migration bytes are valid");

    assert_eq!(compiled.len(), 7);
    assert_eq!(compiled[0].version().as_i64(), 1);
    assert_eq!(compiled[1].version().as_i64(), 2);
    assert_eq!(compiled[2].version().as_i64(), 3);
    assert_eq!(compiled[3].version().as_i64(), 4);
    assert_eq!(compiled[4].version().as_i64(), 5);
    assert_eq!(compiled[5].version().as_i64(), 6);
    assert_eq!(compiled[6].version().as_i64(), 7);
    assert_eq!(
        compiled[0].sql(),
        include_str!("../migrations/0001_owned_schema_epoch.sql")
    );
    let migration_two = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/0002_h3_cell.sql"
    ))
    .expect("epoch-2 H3 migration must exist");
    assert_eq!(compiled[1].sql(), migration_two);
    let migration_three = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/0003_h3_reference_cohort.sql"
    ))
    .expect("epoch-3 H3 cohort migration must exist");
    assert_eq!(compiled[2].sql(), migration_three);
    let migration_four = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/0004_committed_tick_storage.sql"
    ))
    .expect("epoch-4 committed-tick storage migration must exist");
    assert_eq!(compiled[3].sql(), migration_four);
    let migration_five = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/0005_spatial_reference_products.sql"
    ))
    .expect("epoch-5 spatial reference product migration must exist");
    assert_eq!(compiled[4].sql(), migration_five);
    let migration_six = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/0006_h3_shadow_keys.sql"
    ))
    .expect("epoch-6 H3 shadow-key migration must exist");
    assert_eq!(compiled[5].sql(), migration_six);
    let migration_seven = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/migrations/0007_h3_canonical_readers.sql"
    ))
    .expect("epoch-7 canonical H3 reader migration must exist");
    assert_eq!(compiled[6].sql(), migration_seven);
    assert_eq!(
        compiled[0].checksum().as_bytes(),
        &hex_checksum("4fc40761ed3b9a2bfab574d14ce65d24e828d1d51ca3f953a515b18f6f2667d4")
    );
    assert_eq!(
        compiled[1].checksum().as_bytes(),
        &hex_checksum("1f749f1519196c81c911fff619c42daccbf36d6566442ca66015dadd281ab4cd")
    );
    assert_eq!(
        compiled[2].checksum().as_bytes(),
        &hex_checksum("b3c97abba94a96750a02dadea6b77bf22af8a6440cb3d727709db8bc7bfb02b5")
    );
    assert_eq!(
        compiled[3].checksum().as_bytes(),
        &hex_checksum("496509cd05a5b911e933139cd28bbe281fe9c131560c3efca6bdf1ba8abb7dcf")
    );
    assert_eq!(
        compiled[4].checksum().as_bytes(),
        &hex_checksum("3d49e29410419b0573fd16ecaac65ba0f57ca0a9174c0bff1a00b633cc40720b")
    );
    assert_eq!(
        compiled[5].checksum().as_bytes(),
        &hex_checksum("5d97424b4036a3648d49f1f388b0b6da387bb1082fd330eda9ae39cd80d1e249")
    );
    assert_eq!(
        compiled[6].checksum().as_bytes(),
        &hex_checksum("e32dd97fbc6ad1e1baeeb5bff7c00bce04fb6b0ee99a01bf7d99f5b4bab41604")
    );
}

#[test]
fn empty_and_exact_ledgers_are_valid_prefixes() {
    let compiled = two_compiled_migrations();
    let first = persisted(&compiled[0]);
    let second = persisted(&compiled[1]);

    assert_eq!(validate_migration_prefix(&compiled, &[]), Ok(0));
    assert_eq!(validate_migration_prefix(&compiled, &[first]), Ok(1));
    assert_eq!(
        validate_migration_prefix(&compiled, &[first, second]),
        Ok(2)
    );
}

#[test]
fn ledger_must_be_the_exact_contiguous_compiled_prefix() {
    let compiled = two_compiled_migrations();
    let first = persisted(&compiled[0]);
    let second = persisted(&compiled[1]);
    let wrong_checksum = PersistedMigration::from_database(
        MigrationVersion::try_from(1).unwrap().as_i64(),
        &[0x5a; 32],
    )
    .unwrap();

    assert_eq!(
        validate_migration_prefix(&compiled, &[second]),
        Err(SchemaEpochError::LedgerVersionMismatch {
            row_index: 0,
            expected: 1,
            actual: 2,
        })
    );
    assert_eq!(
        validate_migration_prefix(&compiled, &[first, first]),
        Err(SchemaEpochError::LedgerVersionMismatch {
            row_index: 1,
            expected: 2,
            actual: 1,
        })
    );
    assert_eq!(
        validate_migration_prefix(&compiled, &[wrong_checksum]),
        Err(SchemaEpochError::LedgerChecksumMismatch { version: 1 })
    );

    let future = PersistedMigration::from_database(3, &[0x33; 32]).unwrap();
    assert_eq!(
        validate_migration_prefix(&compiled, &[first, second, future]),
        Err(SchemaEpochError::UnknownFutureVersion {
            actual: 3,
            latest_compiled: 2,
        })
    );
}

#[test]
fn compiled_registry_and_database_rows_have_fixed_bounds() {
    let migration = SchemaMigration::new(MigrationVersion::try_from(1).unwrap(), SQL_ONE).unwrap();
    let too_many_compiled = vec![migration; MAX_SCHEMA_MIGRATIONS + 1];
    assert_eq!(
        validate_migration_prefix(&too_many_compiled, &[]),
        Err(SchemaEpochError::CompiledMigrationBound {
            actual: MAX_SCHEMA_MIGRATIONS + 1,
            max: MAX_SCHEMA_MIGRATIONS,
        })
    );

    let row = persisted(&migration);
    let too_many_rows = vec![row; MAX_SCHEMA_MIGRATIONS + 1];
    assert_eq!(
        validate_migration_prefix(&[migration], &too_many_rows),
        Err(SchemaEpochError::LedgerRowBound {
            actual: MAX_SCHEMA_MIGRATIONS + 1,
            max: MAX_SCHEMA_MIGRATIONS,
        })
    );
}

#[test]
fn compiled_registry_itself_must_start_at_one_and_be_contiguous() {
    let v1 = SchemaMigration::new(MigrationVersion::try_from(1).unwrap(), SQL_ONE).unwrap();
    let v2 = SchemaMigration::new(MigrationVersion::try_from(2).unwrap(), SQL_TWO).unwrap();
    let v3 = SchemaMigration::new(MigrationVersion::try_from(3).unwrap(), "SELECT 3;\n").unwrap();

    assert_eq!(
        validate_migration_prefix(&[v2], &[]),
        Err(SchemaEpochError::CompiledVersionMismatch {
            position: 0,
            expected: 1,
            actual: 2,
        })
    );
    assert_eq!(
        validate_migration_prefix(&[v1, v3], &[]),
        Err(SchemaEpochError::CompiledVersionMismatch {
            position: 1,
            expected: 2,
            actual: 3,
        })
    );
    assert_eq!(validate_migration_prefix(&[v1, v2], &[]), Ok(0));
}

#[test]
fn migration_executes_exact_ddl_verification_marker_then_commit() {
    let source = include_str!("../src/schema_epoch.rs");
    let attempt = source
        .split_once("fn attempt_migration(")
        .unwrap()
        .1
        .split_once("fn begin_migration_transaction(")
        .unwrap()
        .0;
    let execute = attempt.find("execute_migration_before_marker").unwrap();
    let marker = attempt.find("insert_ledger_marker").unwrap();
    let commit = attempt.find("transaction.commit()").unwrap();

    assert!(execute < marker);
    assert!(marker < commit);
    assert!(source.contains(".isolation_level(IsolationLevel::Serializable)"));
    assert!(source.contains(".read_only(false)"));
    assert!(source.contains("SET LOCAL search_path TO pg_catalog"));
    assert!(source.contains("SET LOCAL synchronous_commit TO on"));
}

#[test]
fn fresh_and_owned_census_fixtures_are_bounded_and_exactly_sorted() {
    let fixtures = [
        include_str!("../src/fixtures/fresh_schema_epoch_census_v2.txt"),
        include_str!("../src/fixtures/fresh_schema_epoch_census_with_intel_v2.txt"),
        include_str!("../src/fixtures/schema_epoch_owned_fresh_census_v1.txt"),
        include_str!("../src/fixtures/schema_epoch_owned_fresh_census_v2.txt"),
        include_str!("../src/fixtures/schema_epoch_owned_fresh_census_v3.txt"),
        include_str!("../src/fixtures/schema_epoch_owned_fresh_census_v4.txt"),
        include_str!("../src/fixtures/schema_epoch_owned_fresh_census_v5.txt"),
        include_str!("../src/fixtures/schema_epoch_owned_fresh_census_v6.txt"),
        include_str!("../src/fixtures/schema_epoch_owned_fresh_census_v7.txt"),
    ];
    let expected_counts = [7, 8, 4, 5, 7, 17, 25, 25, 25];
    for (fixture, expected) in fixtures.iter().zip(expected_counts) {
        let parsed = babylon_persistence::parse_catalog_census(fixture).unwrap();
        assert_eq!(parsed.entries().len(), expected);
        assert!(fixture.len() <= 65_536);
        assert!(fixture.ends_with('\n'));
    }
}

fn two_compiled_migrations() -> [SchemaMigration; 2] {
    [
        SchemaMigration::new(MigrationVersion::try_from(1).unwrap(), SQL_ONE).unwrap(),
        SchemaMigration::new(MigrationVersion::try_from(2).unwrap(), SQL_TWO).unwrap(),
    ]
}

fn persisted(migration: &SchemaMigration) -> PersistedMigration {
    PersistedMigration::from_database(
        migration.version().as_i64(),
        migration.checksum().as_bytes(),
    )
    .unwrap()
}

fn hex_checksum(value: &str) -> [u8; 32] {
    let mut checksum = [0_u8; 32];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate().take(32) {
        checksum[index] = (hex_nibble(chunk[0]) << 4) | hex_nibble(chunk[1]);
    }
    checksum
}

fn hex_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => panic!("test checksum must use lowercase hexadecimal"),
    }
}
