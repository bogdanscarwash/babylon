//! `PostgreSQL` packaging sentinel for the kernel-owned H3 identity.

use std::path::Path;

const POSTGRES_DOCKERFILE: &str = include_str!("../../../../docker/postgres/Dockerfile");
const POSTGRES_INITDB: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../docker/postgres/initdb"
);
const MAX_INITDB_ENTRIES: usize = 64;

#[test]
fn postgres_test_image_pins_h3_pg_without_activating_it() {
    assert!(POSTGRES_DOCKERFILE.starts_with("# syntax=docker/dockerfile:1.6\n"));
    for required in [
        "postgis/postgis:17-3.5-alpine@sha256:08f4b1e1f4a571008c60272ceb9e0d1f9f8f643792d006b74a35b1bec44c2218",
        "https://github.com/postgis/h3-pg/archive/refs/tags/v4.5.0.tar.gz",
        "sha256:c54c119e1d9a578d5cbcce22f6c66dab2b5a45219fc2b260619807f7f061e53a",
        "https://github.com/uber/h3/archive/refs/tags/v4.5.0.tar.gz",
        "sha256:0da8a392a6ff77e76b60e6a331a49497d0935b6b7b6899da7a3e2786139b0441",
        "-DFETCHCONTENT_SOURCE_DIR_H3=/tmp/h3-core-source",
        "--component h3-pg",
    ] {
        assert!(
            POSTGRES_DOCKERFILE.contains(required),
            "PostgreSQL test image lost pinned H3 build input {required:?}"
        );
    }
    assert!(!POSTGRES_DOCKERFILE
        .to_ascii_uppercase()
        .contains("CREATE EXTENSION H3"));
    assert_initdb_does_not_activate_h3(Path::new(POSTGRES_INITDB));
}

fn assert_initdb_does_not_activate_h3(initdb: &Path) {
    let entries = std::fs::read_dir(initdb)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", initdb.display()))
        .take(MAX_INITDB_ENTRIES + 1)
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|error| panic!("failed to enumerate {}: {error}", initdb.display()));
    assert!(
        entries.len() <= MAX_INITDB_ENTRIES,
        "initdb entry count exceeds the static test bound"
    );
    for entry in entries.iter().take(MAX_INITDB_ENTRIES) {
        assert!(
            entry
                .file_type()
                .expect("initdb file type must resolve")
                .is_file(),
            "initdb contract remains a bounded flat file set"
        );
        let body = std::fs::read_to_string(entry.path())
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", entry.path().display()));
        assert!(
            !body.to_ascii_uppercase().contains("CREATE EXTENSION H3"),
            "initdb file {} must not activate the test-only H3 oracle",
            entry.path().display()
        );
    }
}
