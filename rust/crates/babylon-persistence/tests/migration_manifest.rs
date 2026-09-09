//! Language-neutral legacy migration framing contracts.

use babylon_persistence::{
    ManifestError, MigrationManifest, MAX_MANIFEST_BYTES, MAX_MANIFEST_CHUNKS,
    SCHEMA_ADVISORY_LOCK_KEY,
};

#[test]
fn ordered_chunks_hash_with_one_trailing_nul_each() {
    let manifest = MigrationManifest::from_chunks("small", &[b"a".as_slice(), b"bc".as_slice()])
        .expect("two non-empty chunks are valid");
    assert_eq!(manifest.chunk_count(), 2);
    assert_eq!(
        manifest.digest().to_hex(),
        "aa795aa4bbb6117911ef062e271bcb05ccfd58ea439da7d46a44e3a3fcefa790"
    );
}

#[test]
fn framing_is_order_and_boundary_sensitive() {
    let left =
        MigrationManifest::from_chunks("left", &[b"a".as_slice(), b"bc".as_slice()]).unwrap();
    let right =
        MigrationManifest::from_chunks("right", &[b"ab".as_slice(), b"c".as_slice()]).unwrap();
    assert_ne!(left.digest(), right.digest());
}

#[test]
fn nul_framed_bytes_parse_to_the_same_manifest() {
    let parsed = MigrationManifest::from_nul_framed("small", b"a\0bc\0").unwrap();
    let direct =
        MigrationManifest::from_chunks("small", &[b"a".as_slice(), b"bc".as_slice()]).unwrap();
    assert_eq!(parsed, direct);
}

#[test]
fn malformed_or_unbounded_inputs_fail_loudly() {
    assert_eq!(
        MigrationManifest::from_chunks("", &[b"a"]),
        Err(ManifestError::EmptyName)
    );
    assert_eq!(
        MigrationManifest::from_chunks("empty", &[]),
        Err(ManifestError::EmptySet)
    );
    assert_eq!(
        MigrationManifest::from_chunks("empty-chunk", &[b""]),
        Err(ManifestError::EmptyChunk { index: 0 })
    );
    assert_eq!(
        MigrationManifest::from_nul_framed("empty-interior", b"a\0\0"),
        Err(ManifestError::EmptyChunk { index: 1 })
    );
    assert_eq!(
        MigrationManifest::from_nul_framed("unterminated", b"a"),
        Err(ManifestError::MissingTrailingNul)
    );
    let too_many = vec![b"a".as_slice(); MAX_MANIFEST_CHUNKS + 1];
    assert!(matches!(
        MigrationManifest::from_chunks("many", &too_many),
        Err(ManifestError::TooManyChunks { .. })
    ));
    let too_large = vec![b'a'; MAX_MANIFEST_BYTES + 1];
    assert!(matches!(
        MigrationManifest::from_nul_framed("large", &too_large),
        Err(ManifestError::TooManyBytes { .. })
    ));
}

#[test]
fn embedded_nul_chunks_are_rejected() {
    assert_eq!(
        MigrationManifest::from_chunks("embedded", &[b"a\0b"]),
        Err(ManifestError::EmbeddedNul { index: 0 })
    );
}

#[test]
fn direct_chunks_enforce_the_framed_byte_ceiling_before_nul_scans() {
    let accepted = vec![b'x'; MAX_MANIFEST_BYTES - 1];
    assert!(MigrationManifest::from_chunks("exact-limit", &[&accepted]).is_ok());

    let mut too_large = vec![b'x'; MAX_MANIFEST_BYTES];
    too_large[MAX_MANIFEST_BYTES - 1] = 0;
    assert_eq!(
        MigrationManifest::from_chunks("over-limit", &[&too_large]),
        Err(ManifestError::TooManyBytes {
            actual: MAX_MANIFEST_BYTES + 1,
            max: MAX_MANIFEST_BYTES,
        })
    );
}

#[test]
fn the_cross_language_lock_key_is_pinned() {
    assert_eq!(SCHEMA_ADVISORY_LOCK_KEY, 0xBAB1_0537_i64);
    assert_eq!(SCHEMA_ADVISORY_LOCK_KEY, 3_132_163_383_i64);
}
