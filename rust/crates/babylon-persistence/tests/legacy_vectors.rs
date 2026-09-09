//! Frozen Python-to-Rust migration adoption vectors at `dev@ae5b2615`.

use babylon_persistence::{MigrationManifest, SCHEMA_ADVISORY_LOCK_KEY};

#[test]
fn legacy_schema_ddl_vector_is_exact() {
    let bytes = include_bytes!("fixtures/legacy_schema_ddl_v1.bin");
    let manifest = MigrationManifest::from_nul_framed("POSTGRES_SCHEMA_DDL", bytes).unwrap();
    assert_eq!(manifest.chunk_count(), 112);
    assert_eq!(
        manifest.digest().to_hex(),
        "0902471053ab7a22cdaf0340978712772990e87a63aaaa1636608894fa52590b"
    );
}

#[test]
fn legacy_numbered_migration_vector_is_exact() {
    let bytes = include_bytes!("fixtures/legacy_migrations_0010_0044_v1.bin");
    let manifest = MigrationManifest::from_nul_framed("migrations-0010-0044", bytes).unwrap();
    assert_eq!(manifest.chunk_count(), 35);
    assert_eq!(
        manifest.digest().to_hex(),
        "4abe69ddc25569d5dff1941b4fbe2973df5cbd70a9bca4c92b9fe26f51dd45db"
    );
    assert_eq!(SCHEMA_ADVISORY_LOCK_KEY, 0xBAB1_0537_i64);
}
