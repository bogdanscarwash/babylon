//! Read-only legacy `PostgreSQL` adopter contract tests.

use babylon_persistence::{
    compare_legacy_census, expected_legacy_census, legacy_adopter_sql_statements,
    parse_legacy_census_fixture, validate_legacy_connection_target, validate_legacy_stamps,
    LegacyAdopterError, LegacyAdopterOperation, LegacyAdopterSqlKind, LegacyBoundedResource,
    LegacyCensusEntry, LegacyCensusParseError, LegacyCleanupFailureV1,
    LegacyConnectionTargetRejection, LegacyObjectKey, LegacyObjectKind, LegacyStampClass,
    LegacyStampProvenance, MigrationManifest, LEGACY_ADOPTER_STARTUP_OPTIONS,
    LEGACY_CENSUS_FIXTURE, LEGACY_CENSUS_VERSION, LEGACY_STAMP_CATALOG,
    MAX_LEGACY_CENSUS_FIXTURE_BYTES, MAX_LEGACY_CENSUS_ROWS, MAX_LEGACY_STAMP_ROWS,
    POSTGRES_IDENTIFIER_MAX_BYTES, SCHEMA_ADVISORY_LOCK_KEY,
};
use postgres::Config;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::process::Command;

const ZERO_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const LEGACY_CENSUS_V1_ARCHIVE: &str = include_str!("../src/fixtures/legacy_adopter_census_v1.txt");
const MAX_WORKFLOW_JOB_BOUNDARY_CANDIDATES: usize = 128;
const MAX_SQL_LITERAL_SEGMENTS: usize = 8_192;
const MAX_SQL_STATEMENT_BYTES: usize = 262_144;

#[test]
fn cleanup_failure_evidence_is_a_public_structured_contract() {
    let failure = LegacyCleanupFailureV1 {
        operation: LegacyAdopterOperation::Rollback,
        diagnostic: None,
    };

    assert_eq!(failure.operation, LegacyAdopterOperation::Rollback);
    assert_eq!(failure.diagnostic, None);
}

#[test]
fn checked_in_census_fixture_is_bounded_sorted_and_versioned() {
    let census = expected_legacy_census().expect("checked-in census fixture must parse");
    assert_eq!(LEGACY_CENSUS_VERSION, 2);
    assert!(census.entries().len() <= MAX_LEGACY_CENSUS_ROWS);
    assert!(LEGACY_CENSUS_FIXTURE.len() <= MAX_LEGACY_CENSUS_FIXTURE_BYTES);
    assert!(census
        .entries()
        .windows(2)
        .take(MAX_LEGACY_CENSUS_ROWS)
        .all(|pair| pair[0].key() < pair[1].key()));
    assert_eq!(count_kind(&census, LegacyObjectKind::PartitionedTable), 9);
    assert_eq!(count_kind(&census, LegacyObjectKind::Role), 1);
    assert_eq!(count_kind(&census, LegacyObjectKind::Database), 1);
    assert_eq!(census.entries().len(), 102);
}

#[test]
fn fixture_preserves_zero_schema_and_two_canonical_schema_grants() {
    let census = expected_legacy_census().expect("checked-in census fixture must parse");
    assert_eq!(count_kind(&census, LegacyObjectKind::Schema), 0);
    let schema_grants = census
        .entries()
        .iter()
        .take(MAX_LEGACY_CENSUS_ROWS)
        .filter(|entry| entry.key().kind() == LegacyObjectKind::SchemaGrant)
        .map(|entry| entry.key().clone())
        .collect::<Vec<_>>();
    assert_eq!(
        schema_grants,
        vec![
            LegacyObjectKey::new(
                LegacyObjectKind::SchemaGrant,
                "pg_namespace",
                "babylon_meta"
            )
            .unwrap(),
            LegacyObjectKey::new(LegacyObjectKind::SchemaGrant, "pg_namespace", "public").unwrap(),
        ]
    );
}

#[test]
fn v1_archive_provenance_preserves_the_observed_legacy_runtime() {
    let fixture = LEGACY_CENSUS_V1_ARCHIVE;
    let pinned = "sha256:77e89c11c4779c394ebeeaac1099dafb77b728abc8cd45dcaf6c4695503a0c37";
    assert!(fixture.lines().take(32).any(|line| {
        line == format!("# provenance|postgres_image|postgis/postgis:17-3.5@{pinned}")
    }));
    assert!(fixture
        .lines()
        .take(32)
        .any(|line| line == "# provenance|postgres_version|17.5 (Debian 17.5-1.pgdg110+1)"));
    for fixture_line in [
        "# provenance|postgres_package|postgresql-17=17.5-1.pgdg110+1",
        "# provenance|postgis_package|postgresql-17-postgis-3=3.5.2+dfsg-1.pgdg110+1",
        "# provenance|pgvector_package|postgresql-17-pgvector=0.8.5-1.pgdg11+1",
        "# provenance|pgvector_deb_sha256|ff0e10806fd87268e2dfac6b2d0aaa5fc2c24341188e7c24f3db7fd112c90f87",
    ] {
        assert!(fixture
            .lines()
            .take(32)
            .any(|line| line == fixture_line));
    }
    assert!(fixture.lines().take(32).any(|line| {
        line == "# provenance|source_git_commit|a95ddd1b24b1157eef95b9b1885219c67486a160"
    }));
    assert!(fixture.lines().take(32).any(|line| {
        line == "# provenance|startup_options|default_transaction_read_only=on|\
                 statement_timeout=5000ms|lock_timeout=5000ms|\
                 idle_in_transaction_session_timeout=5000ms|\
                 quote_all_identifiers=off|search_path=pg_catalog|jit=off|event_triggers=off"
    }));
    for definition in LEGACY_STAMP_CATALOG.iter().take(LEGACY_STAMP_CATALOG.len()) {
        assert!(fixture.lines().take(32).any(|line| {
            line == format!(
                "# provenance|{}|chunks={}|digest={}",
                definition.name, definition.chunk_count, definition.digest_hex
            )
        }));
    }
}

#[test]
fn v2_fixture_provenance_is_exact_across_the_container_boundary() {
    let header = LEGACY_CENSUS_FIXTURE.lines().take(24).collect::<Vec<_>>();
    for expected in [
        "# Babylon PER-272 current legacy adopter census fixture v2",
        "# provenance|source|legacy_adopter_census.sql",
        "# provenance|census_sql_sha256|\
         f6b876b0e379081adf25a89165eac7a57f1e596fc369abbbc34807d6268a6c22",
        "# provenance|postgres_image|postgis/postgis:17-3.5-alpine@sha256:\
         08f4b1e1f4a571008c60272ceb9e0d1f9f8f643792d006b74a35b1bec44c2218",
        "# provenance|postgres_server_version_num|170011",
        "# provenance|locale_provider|builtin",
        "# provenance|locale|C.UTF-8",
        "# provenance|encoding|UTF8",
        "# provenance|postgis_version|3.5.7",
        "# provenance|pgvector_version|0.8.5",
        "# provenance|h3_version|4.5.0",
        "# provenance|artifact_contract|digest-pinned-base|checksum-pinned-sources|\
         exact-final-runtime-packages|behaviorally-verified|not-byte-reproducible",
        "# provenance|optional_cluster_role|babylon_intel",
        "# provenance|authority_schemas|babylon_ref,babylon_state",
        "# provenance|POSTGRES_SCHEMA_DDL|chunks=112|digest=\
         0902471053ab7a22cdaf0340978712772990e87a63aaaa1636608894fa52590b",
        "# provenance|migrations-0010-0044|chunks=35|digest=\
         4abe69ddc25569d5dff1941b4fbe2973df5cbd70a9bca4c92b9fe26f51dd45db",
    ] {
        assert!(
            header.contains(&expected),
            "missing v2 provenance: {expected}"
        );
    }

    let dockerfile = include_str!("../../../../docker/postgres/Dockerfile");
    for exact_input in [
        "Alpine Linux v3.24.1",
        "sha256:35d90a44f150ef26c1c734e86e3f888753ca7fd6e1321d85e73f66a7dbffb531",
        "ed9a09c834787abb5952318bccef1c4fee119f1d",
        "159b79aaad5983fb7459c1e3df2897fbb2d11788",
        "sha256:c54c119e1d9a578d5cbcce22f6c66dab2b5a45219fc2b260619807f7f061e53a",
        "sha256:0da8a392a6ff77e76b60e6a331a49497d0935b6b7b6899da7a3e2786139b0441",
        "sha256:6f88a5cbdde31666f4b6c1a6b75c51dcbeffe58f9a7d2b26e502d5a6e5e14d44",
        "su-exec=0.3-r0",
        "giflib=5.2.2-r2",
        "libcrypto3=3.5.8-r0",
        "libssl3=3.5.8-r0",
        "test \"$(cat /etc/alpine-release)\" = '3.24.1'",
        "does not claim byte-identical",
        "image rebuilds",
    ] {
        assert!(
            dockerfile.contains(exact_input),
            "missing exact container input: {exact_input}"
        );
    }

    let marker = "babylon-postgres-lineage-v1|postgres=17|locale-provider=builtin|\
                  locale=C.UTF-8|encoding=UTF8|postgis=3.5.7|h3=4.5.0|\
                  h3_postgis=4.5.0|vector=0.8.5";
    let compose = include_str!("../../../../docker-compose.yml");
    let entrypoint_patch = include_str!("../../../../docker/postgres/patch-entrypoint.awk");
    assert!(compose.contains(marker));
    assert!(entrypoint_patch.contains(marker));

    for historical_stamp in [
        "# provenance|BABYLON_META_DDL|",
        "# provenance|migrations-0010-0043|",
        "# provenance|trace-view-v2-migrations-0020-0023|",
    ] {
        assert!(!LEGACY_CENSUS_FIXTURE.contains(historical_stamp));
    }
}

#[test]
fn every_v2_header_pins_the_exact_executable_census_sql() {
    const EXPECTED_CENSUS_SQL_SHA256: &str =
        "f6b876b0e379081adf25a89165eac7a57f1e596fc369abbbc34807d6268a6c22";
    let census_sql = include_bytes!("../src/legacy_adopter_census.sql");
    assert_eq!(sha256_hex(census_sql), EXPECTED_CENSUS_SQL_SHA256);

    let expected_header = format!("# provenance|census_sql_sha256|{EXPECTED_CENSUS_SQL_SHA256}");
    for fixture in [
        LEGACY_CENSUS_FIXTURE,
        include_str!("../src/fixtures/fresh_schema_epoch_census_v2.txt"),
        include_str!("../src/fixtures/fresh_schema_epoch_census_with_intel_v2.txt"),
    ] {
        assert_eq!(
            fixture
                .lines()
                .take(24)
                .filter(|line| *line == expected_header)
                .count(),
            1
        );
    }
}

#[test]
fn v2_is_the_only_current_census_path_while_v1_bytes_remain_archival() {
    for (fixture, expected_sha256) in [
        (
            include_bytes!("../src/fixtures/legacy_adopter_census_v1.txt").as_slice(),
            "f80017b2e5bc2ae2577b4ff610af2f9ff369da1c78232edee4dfc67f6a01fd49",
        ),
        (
            include_bytes!("../src/fixtures/fresh_schema_epoch_census_v1.txt").as_slice(),
            "6a2a7f3fbbfac61bee42fee1f920cf4785f3163372f03a6918d8eb029865dc20",
        ),
        (
            include_bytes!("../src/fixtures/fresh_schema_epoch_census_with_intel_v1.txt")
                .as_slice(),
            "023850010f306495080586c4699fd4c99bd8a527951720566c5a2b585094e720",
        ),
        (
            include_bytes!("../src/fixtures/legacy_adopter_census_v2.txt").as_slice(),
            "e2593b124bf813a02aa993bfea1bad91b54a7b37d3589947429a4cbe3b3d218d",
        ),
        (
            include_bytes!("../src/fixtures/fresh_schema_epoch_census_v2.txt").as_slice(),
            "42a03597faa64d388398395f12fb1eaa7890fb1487837c29a4a12985720ea294",
        ),
        (
            include_bytes!("../src/fixtures/fresh_schema_epoch_census_with_intel_v2.txt")
                .as_slice(),
            "51ffa07e0adfa3125f6054169dd475baf3dbce03ecca5d9213548b963d35acfb",
        ),
    ] {
        assert_eq!(sha256_hex(fixture), expected_sha256);
    }

    let adopter = include_str!("../src/legacy_adopter.rs");
    assert!(adopter.contains(
        "pub const LEGACY_CENSUS_FIXTURE: &str = \
         include_str!(\"fixtures/legacy_adopter_census_v2.txt\");"
    ));
    assert!(!adopter.contains("fixtures/legacy_adopter_census_v1.txt"));

    let epoch = include_str!("../src/schema_epoch.rs");
    let current_start = epoch.find("const FRESH_CENSUS:").unwrap();
    let current_end = epoch.find("const EPOCH_OWNED_CENSUS_V1:").unwrap();
    let current = &epoch[current_start..current_end];
    assert!(current.contains("fresh_schema_epoch_census_v2.txt"));
    assert!(current.contains("fresh_schema_epoch_census_with_intel_v2.txt"));
    assert!(!current.contains("_v1.txt"));
}

#[test]
fn conservation_drift_is_only_the_locale_order_of_two_exact_constraints() {
    const V1_PAYLOAD_SHA256: &str =
        "30ef0e3b7795606b15a35a2f91bcc40dc60be80f0a000d362bc17c5737ff00e2";
    const V2_PAYLOAD_SHA256: &str =
        "5eff9766641285da9cf078e6633f603a3492f2735b6e5dd6f1c9b1fd07b84b50";
    let archived_bytes = canonical_json_receipt(include_bytes!(
        "fixtures/conservation_audit_log_payload_v1.json"
    ));
    let current_bytes = canonical_json_receipt(include_bytes!(
        "fixtures/conservation_audit_log_payload_v2.json"
    ));
    assert_eq!(sha256_hex(archived_bytes), V1_PAYLOAD_SHA256);
    assert_eq!(sha256_hex(current_bytes), V2_PAYLOAD_SHA256);
    assert_eq!(
        census_fixture_digest(
            LEGACY_CENSUS_V1_ARCHIVE,
            "partitioned_table|public|conservation_audit_log|"
        ),
        V1_PAYLOAD_SHA256
    );
    assert_eq!(
        census_fixture_digest(
            LEGACY_CENSUS_FIXTURE,
            "partitioned_table|public|conservation_audit_log|"
        ),
        V2_PAYLOAD_SHA256
    );

    let mut archived: serde_json::Value = serde_json::from_slice(archived_bytes).unwrap();
    let current: serde_json::Value = serde_json::from_slice(current_bytes).unwrap();
    let pointer = "/default_partition_template/shape/partition_constraints";
    let archived_constraints = archived.pointer(pointer).unwrap().as_array().unwrap();
    let current_constraints = current.pointer(pointer).unwrap().as_array().unwrap();
    assert_eq!(archived_constraints.len(), 5);
    assert_eq!(current_constraints.len(), 5);
    assert_eq!(archived_constraints[0], current_constraints[1]);
    assert_eq!(archived_constraints[1], current_constraints[0]);
    assert_eq!(&archived_constraints[2..], &current_constraints[2..]);
    assert_eq!(
        archived_constraints[0]["definition"],
        "CHECK (length(hex_frame_hash) = 64)"
    );
    assert_eq!(
        archived_constraints[1]["definition"],
        "CHECK ((scale = ANY (ARRAY['hex'::text, 'county'::text, 'state'::text, \
         'national'::text, 'global_phi'::text, 'per_stage'::text])) OR scale ~~ \
         'external:%'::text)"
    );

    archived
        .pointer_mut(pointer)
        .unwrap()
        .as_array_mut()
        .unwrap()
        .swap(0, 1);
    assert_eq!(archived, current);

    let sql = census_sql();
    assert!(sql.contains(
        "'partition_constraints', coalesce((\n                SELECT \
         pg_catalog.jsonb_agg"
    ));
    assert!(sql.contains(
        ") ORDER BY\n                    con.contype,\n                    \
         pg_catalog.pg_get_constraintdef(con.oid, true),\n                    \
         con.condeferrable,\n                    con.condeferred,\n                    \
         con.convalidated)"
    ));
}

#[test]
fn stamp_catalog_classifies_current_meta_and_named_history() {
    assert_eq!(LEGACY_STAMP_CATALOG.len(), 5);
    assert_stamp(
        "POSTGRES_SCHEMA_DDL",
        112,
        "0902471053ab7a22cdaf0340978712772990e87a63aaaa1636608894fa52590b",
        LegacyStampClass::RequiredCurrent,
    );
    assert_stamp(
        "migrations-0010-0044",
        35,
        "4abe69ddc25569d5dff1941b4fbe2973df5cbd70a9bca4c92b9fe26f51dd45db",
        LegacyStampClass::RequiredCurrent,
    );
    assert_stamp(
        "BABYLON_META_DDL",
        6,
        "edb77a84d35f30eab061ead7620ea2907554ee9231c92b6acfe11fc530f669d8",
        LegacyStampClass::AllowedMeta,
    );
    assert_stamp(
        "migrations-0010-0043",
        34,
        "7c77114a3b7053bed1dafae8aea77a894ef2034504306d23745d217252bd6711",
        LegacyStampClass::HistoricalFullMigration,
    );
    assert_stamp(
        "trace-view-v2-migrations-0020-0023",
        4,
        "1d0a4f5cbd8f5cba3b59d48a25950fc4874923ed962400d8374d34edb31fd7b2",
        LegacyStampClass::HistoricalTestSubset,
    );
}

#[test]
fn stamp_catalog_is_proved_from_the_checked_in_frozen_vectors() {
    let schema = split_nul_framed(include_bytes!("fixtures/legacy_schema_ddl_v1.bin"), 112);
    let migrations = split_nul_framed(
        include_bytes!("fixtures/legacy_migrations_0010_0044_v1.bin"),
        35,
    );
    assert_manifest(
        &schema,
        0,
        112,
        "POSTGRES_SCHEMA_DDL",
        112,
        0,
        LEGACY_STAMP_CATALOG[0].digest_hex,
    );
    assert_manifest(
        &migrations,
        0,
        35,
        "migrations-0010-0044",
        35,
        0,
        LEGACY_STAMP_CATALOG[1].digest_hex,
    );
    assert_manifest(
        &schema,
        106,
        112,
        "BABYLON_META_DDL",
        6,
        1_552,
        LEGACY_STAMP_CATALOG[2].digest_hex,
    );
    assert_manifest(
        &migrations,
        0,
        34,
        "migrations-0010-0043",
        34,
        98_830,
        LEGACY_STAMP_CATALOG[3].digest_hex,
    );
    assert_manifest(
        &migrations,
        10,
        14,
        "trace-view-v2-migrations-0020-0023",
        4,
        9_440,
        LEGACY_STAMP_CATALOG[4].digest_hex,
    );
}

#[test]
fn historical_stamp_provenance_is_exact_and_loud() {
    // Python stored only digest/applied_at. These labels are descriptive identities reconstructed
    // from exact source history; neither historical class satisfies RequiredCurrent.
    assert_eq!(
        LEGACY_STAMP_CATALOG[3].provenance,
        LegacyStampProvenance::HistoricalFullMigration {
            framed_bytes: 98_830,
            source_commit: "2de19c1cc8e2dd1d19f6e95d232d1dc71e7caa96",
            first_dev_merge: "1d5efaf7da9e75d531426224015e0e616719bdde",
            superseded_by: "migrations-0010-0044",
        }
    );
    assert_eq!(
        LEGACY_STAMP_CATALOG[4].provenance,
        LegacyStampProvenance::HistoricalTestSubset {
            framed_bytes: 9_440,
            source_introduction_commit: "312065d7859743a81ac2e17899cb5b0c2971ac53",
            fixture_commit: "79d4390b869cda61a7af550626fe02278ab218aa",
            producer_commit: "7b22fdb9d3abc5348aa0710a3e4fdb995848ccfd",
            producer_dev_merge: "073e3b1ee0640e0b00935cf78755f49747e1fd37",
            removed_commit: "894b473625737276eed9550b8e62a7079bedec08",
            removed_dev_merge: "09cc23da6c5c5770df96ccc16143de298a0c7a50",
        }
    );
}

#[test]
fn stamp_validation_requires_current_and_reports_each_history_class() {
    let mut exact = required_stamp_digests();
    let exact_report = validate_legacy_stamps(&exact).unwrap();
    assert_eq!(
        exact_report.matched_count(LegacyStampClass::RequiredCurrent),
        2
    );
    assert_eq!(exact_report.matches().len(), 2);

    for definition in LEGACY_STAMP_CATALOG
        .iter()
        .filter(|item| item.class != LegacyStampClass::RequiredCurrent)
        .take(3)
    {
        exact.push(definition.digest_hex.to_owned());
    }
    let report = validate_legacy_stamps(&exact).unwrap();
    assert_eq!(report.matched_count(LegacyStampClass::AllowedMeta), 1);
    assert_eq!(
        report.matched_count(LegacyStampClass::HistoricalFullMigration),
        1
    );
    assert_eq!(
        report.matched_count(LegacyStampClass::HistoricalTestSubset),
        1
    );
}

#[test]
fn stamp_validation_rejects_missing_unknown_duplicate_and_over_limit_rows() {
    let missing = vec![LEGACY_STAMP_CATALOG[0].digest_hex.to_owned()];
    assert!(matches!(
        validate_legacy_stamps(&missing),
        Err(LegacyAdopterError::RequiredStampMissing { .. })
    ));

    let mut unknown = required_stamp_digests();
    unknown.push("3333333333333333333333333333333333333333333333333333333333333333".into());
    assert!(matches!(
        validate_legacy_stamps(&unknown),
        Err(LegacyAdopterError::UnknownStamp { .. })
    ));

    let mut duplicate = required_stamp_digests();
    duplicate.push(LEGACY_STAMP_CATALOG[0].digest_hex.into());
    assert!(matches!(
        validate_legacy_stamps(&duplicate),
        Err(LegacyAdopterError::DuplicateStamp { .. })
    ));

    let oversized = vec![ZERO_DIGEST.to_owned(); MAX_LEGACY_STAMP_ROWS + 1];
    assert!(matches!(
        validate_legacy_stamps(&oversized),
        Err(LegacyAdopterError::Bounds { .. })
    ));
}

#[test]
fn census_fixture_parser_rejects_duplicate_unsorted_and_malformed_records() {
    let duplicate =
        format!("relation|public|alpha|{ZERO_DIGEST}\nrelation|public|alpha|{ZERO_DIGEST}\n");
    assert!(matches!(
        parse_legacy_census_fixture(&duplicate),
        Err(LegacyCensusParseError::DuplicateObject { .. })
    ));
    let unsorted =
        format!("relation|public|zeta|{ZERO_DIGEST}\nrelation|public|alpha|{ZERO_DIGEST}\n");
    assert!(matches!(
        parse_legacy_census_fixture(&unsorted),
        Err(LegacyCensusParseError::OutOfOrder { .. })
    ));
    assert_eq!(
        parse_legacy_census_fixture("relation|public|only_three_fields\n"),
        Err(LegacyCensusParseError::MalformedRecord { line: 1, fields: 3 })
    );
    assert_eq!(
        parse_legacy_census_fixture(&format!("relation|public|name|{ZERO_DIGEST}|extra\n")),
        Err(LegacyCensusParseError::MalformedRecord { line: 1, fields: 5 })
    );
    assert_eq!(
        parse_legacy_census_fixture(&format!(
            "relation|public|name|{ZERO_DIGEST}|extra|second_extra\n"
        )),
        Err(LegacyCensusParseError::MalformedRecord { line: 1, fields: 6 })
    );
}

#[test]
fn census_fixture_parser_rejects_invalid_kind_identifier_digest_and_bounds() {
    let maximum = "a".repeat(POSTGRES_IDENTIFIER_MAX_BYTES);
    let too_long = "a".repeat(POSTGRES_IDENTIFIER_MAX_BYTES + 1);
    assert!(
        parse_legacy_census_fixture(&format!("relation|public|{maximum}|{ZERO_DIGEST}\n")).is_ok()
    );
    let cases = [
        (
            format!("bogus|public|name|{ZERO_DIGEST}\n"),
            LegacyCensusParseError::InvalidKind { line: 1 },
        ),
        (
            format!("relation|Public|name|{ZERO_DIGEST}\n"),
            LegacyCensusParseError::InvalidIdentifier { line: 1 },
        ),
        (
            format!("relation|public|{too_long}|{ZERO_DIGEST}\n"),
            LegacyCensusParseError::InvalidIdentifier { line: 1 },
        ),
        (
            "relation|public|name|abcdef\n".to_owned(),
            LegacyCensusParseError::InvalidDigest { line: 1 },
        ),
    ];
    for (fixture, expected) in cases {
        assert_eq!(parse_legacy_census_fixture(&fixture), Err(expected));
    }

    let fixture = "#".repeat(MAX_LEGACY_CENSUS_FIXTURE_BYTES + 1);
    assert_eq!(
        parse_legacy_census_fixture(&fixture),
        Err(LegacyCensusParseError::TooManyBytes {
            actual: MAX_LEGACY_CENSUS_FIXTURE_BYTES + 1,
            max: MAX_LEGACY_CENSUS_FIXTURE_BYTES,
        })
    );
}

#[test]
fn census_parser_and_comparison_enforce_row_bounds_without_unbounded_collect() {
    let mut oversized = String::new();
    for index in 0..=MAX_LEGACY_CENSUS_ROWS {
        writeln!(oversized, "relation|public|name_{index:04}|{ZERO_DIGEST}").unwrap();
    }
    assert!(matches!(
        parse_legacy_census_fixture(&oversized),
        Err(LegacyCensusParseError::TooManyRows { .. })
    ));

    let expected = expected_legacy_census().unwrap();
    let oversized_actual = vec![expected.entries()[0].clone(); MAX_LEGACY_CENSUS_ROWS + 1];
    assert!(matches!(
        compare_legacy_census(&expected, &oversized_actual),
        Err(LegacyAdopterError::Bounds { .. })
    ));
}

#[test]
fn pure_census_comparison_refuses_all_sorted_extra_keys_and_rejects_duplicates() {
    let expected = expected_legacy_census().unwrap();
    let mut actual = expected.entries().to_vec();
    let beta = entry(LegacyObjectKind::Domain, "public", "extra_beta", '1');
    let alpha = entry(LegacyObjectKind::Relation, "public", "extra_alpha", '2');
    actual.push(beta.clone());
    actual.push(alpha.clone());
    assert_eq!(
        compare_legacy_census(&expected, &actual),
        Err(LegacyAdopterError::UnsupportedLegacyExtras {
            objects: vec![beta.key().clone(), alpha.key().clone()],
        })
    );

    let mut duplicate = expected.entries().to_vec();
    duplicate.push(expected.entries()[0].clone());
    assert!(matches!(
        compare_legacy_census(&expected, &duplicate),
        Err(LegacyAdopterError::DuplicateCensusObject { .. })
    ));
}

#[test]
fn review_census_comparison_refuses_every_extra_object_kind() {
    let expected = expected_legacy_census().unwrap();
    for kind in [
        LegacyObjectKind::Database,
        LegacyObjectKind::Domain,
        LegacyObjectKind::Extension,
        LegacyObjectKind::ForeignTable,
        LegacyObjectKind::MaterializedView,
        LegacyObjectKind::PartitionedTable,
        LegacyObjectKind::Relation,
        LegacyObjectKind::Role,
        LegacyObjectKind::Routine,
        LegacyObjectKind::Schema,
        LegacyObjectKind::SchemaGrant,
        LegacyObjectKind::Sequence,
        LegacyObjectKind::UnsupportedCatalog,
        LegacyObjectKind::UserType,
        LegacyObjectKind::View,
    ] {
        let mut actual = expected.entries().to_vec();
        let extra = entry(kind, "public", "unsafe_extra", '3');
        actual.push(extra.clone());
        assert_eq!(
            compare_legacy_census(&expected, &actual),
            Err(LegacyAdopterError::UnsupportedLegacyExtras {
                objects: vec![extra.key().clone()],
            })
        );
    }
}

#[test]
fn pure_census_comparison_reports_missing_and_changed_expected_objects() {
    let expected = expected_legacy_census().unwrap();
    let mut missing_actual = expected.entries().to_vec();
    let missing_key = missing_actual.remove(0).key().clone();
    assert!(matches!(
        compare_legacy_census(&expected, &missing_actual),
        Err(LegacyAdopterError::CensusMismatch {
            missing_objects,
            changed_objects,
            ..
        }) if missing_objects == vec![missing_key] && changed_objects.is_empty()
    ));

    let mut changed_actual = expected.entries().to_vec();
    let changed_key = changed_actual[0].key().clone();
    changed_actual[0] = entry(
        changed_key.kind(),
        changed_key.schema(),
        changed_key.name(),
        '2',
    );
    assert!(matches!(
        compare_legacy_census(&expected, &changed_actual),
        Err(LegacyAdopterError::CensusMismatch {
            missing_objects,
            changed_objects,
            ..
        }) if missing_objects.is_empty() && changed_objects == vec![changed_key]
    ));
}

#[test]
fn stamp_table_shape_refusal_precedes_full_census_comparison() {
    let source = include_str!("../src/legacy_adopter.rs");
    let transaction = cte_slice(
        source,
        "fn verify_transaction(",
        "fn verify_transaction_settings(",
    );
    let read_census = transaction.find("read_census_rows(transaction)").unwrap();
    let verify_stamp = transaction
        .find("verify_stamp_table_shape(&expected, actual.as_slice())")
        .unwrap();
    let compare = transaction
        .find("compare_legacy_census(&expected, actual.as_slice())")
        .unwrap();
    assert!(read_census < verify_stamp);
    assert!(verify_stamp < compare);
}

#[test]
fn adopter_sql_is_single_statement_read_only_and_uses_exact_advisory_functions() {
    let statements = legacy_adopter_sql_statements();
    assert_eq!(statements.len(), 7);
    for statement in statements.iter().take(7) {
        let sql = statement.sql().trim();
        assert!(!sql.contains(';'));
        assert!(sql.starts_with("SELECT") || sql.starts_with("WITH"));
        let code = sql_without_string_literals(sql);
        let tokens = code
            .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
            .filter(|token| !token.is_empty())
            .map(str::to_ascii_uppercase)
            .take(16_384);
        for token in tokens {
            assert!(![
                "ALTER", "CALL", "COMMENT", "COPY", "CREATE", "DELETE", "DO", "DROP", "EXECUTE",
                "GRANT", "INSERT", "MERGE", "REFRESH", "REINDEX", "REVOKE", "TRUNCATE", "UPDATE",
                "VACUUM",
            ]
            .contains(&token.as_str()));
        }
    }
    let all_sql = statements
        .iter()
        .take(7)
        .map(babylon_persistence::LegacyAdopterSqlStatement::sql)
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        all_sql.matches("pg_catalog.pg_try_advisory_lock").count(),
        1
    );
    assert_eq!(all_sql.matches("pg_catalog.pg_advisory_unlock").count(), 1);
    assert_eq!(all_sql.matches("pg_advisory_lock").count(), 0);
    assert_eq!(SCHEMA_ADVISORY_LOCK_KEY, 0xBAB1_0537_i64);
}

#[test]
fn review_relation_shape_binds_inbound_inheritance_and_subpartitioning() {
    let sql = census_sql();
    let shapes = cte_slice(sql, "relation_shapes AS", "index_shapes AS");
    for field in [
        "'kind', c.relkind",
        "'is_partition', c.relispartition",
        "'has_subclass', c.relhassubclass",
        "WHEN c.relkind = 'p' THEN pg_catalog.pg_get_partkeydef(c.oid)",
    ] {
        assert!(
            shapes.contains(field),
            "missing relation shape field: {field}"
        );
    }
    let relations = cte_slice(sql, "relation_payloads AS", "rel_objects AS");
    assert!(relations.contains("'has_subclass', c.relhassubclass"));
    assert!(relations.contains("'partition_key', shape.child_shape -> 'partition_key'"));
}

#[test]
fn relation_and_composite_signatures_preserve_attribute_history_and_state() {
    let sql = census_sql();
    let relation_candidates = cte_slice(
        sql,
        "candidate_attributes AS MATERIALIZED",
        "candidate_attribute_options AS MATERIALIZED",
    );
    assert!(relation_candidates.contains("attribute.attnum > 0"));
    assert!(!relation_candidates.contains("NOT attribute.attisdropped"));
    let relation_shape = cte_slice(sql, "relation_shapes AS", "index_shapes AS");
    for field in [
        "'relnatts', c.relnatts",
        "'dropped', a.attisdropped",
        "'has_missing', a.atthasmissing",
        "pg_catalog.to_jsonb(a.attmissingval)",
        "'is_local', a.attislocal",
        "'inheritance_count', a.attinhcount",
    ] {
        assert!(
            relation_shape.contains(field),
            "missing relation attribute history field: {field}"
        );
    }
    let relation_payload = cte_slice(sql, "relation_payloads AS", "rel_objects AS");
    assert!(relation_payload.contains("'relnatts', shape.child_shape -> 'relnatts'"));
    let composite = cte_slice(
        sql,
        "candidate_composite_attributes AS MATERIALIZED",
        "user_type_objects AS",
    );
    assert!(!composite.contains("NOT attribute.attisdropped"));
    for field in [
        "'relnatts', coalesce(type_relation.relnatts, 0)",
        "'dropped', attribute.attisdropped",
        "'has_missing', attribute.atthasmissing",
        "pg_catalog.to_jsonb(attribute.attmissingval)",
        "'is_local', attribute.attislocal",
        "'inheritance_count', attribute.attinhcount",
    ] {
        assert!(
            composite.contains(field),
            "missing composite attribute history field: {field}"
        );
    }
}

#[test]
fn parent_relation_indexes_embed_the_complete_canonical_index_shape() {
    let sql = census_sql();
    let relations = cte_slice(sql, "relation_payloads AS", "rel_objects AS");
    let indexes = cte_slice(relations, "'indexes', coalesce((", "'view_definition'");
    for contract in [
        "'schema', index_ns.nspname",
        "'name', index_class.relname",
        "'shape', index_shape.index_shape",
        "JOIN index_shapes AS index_shape",
    ] {
        assert!(
            indexes.contains(contract),
            "missing rich parent index: {contract}"
        );
    }
    for reduced_field in [
        "index_row.indisunique",
        "index_row.indisprimary",
        "index_row.indisexclusion",
        "index_row.indisvalid",
        "pg_catalog.pg_get_indexdef",
    ] {
        assert!(
            !indexes.contains(reduced_field),
            "parent index rebuilt reduced field: {reduced_field}"
        );
    }
}

#[test]
fn review_dead_unsafe_authority_api_is_retired_in_favor_of_exact_census() {
    let source = include_str!("../src/legacy_adopter.rs");
    let exports = include_str!("../src/lib.rs");
    assert!(!source.contains("UnsafeAuthority"));
    assert!(!source.contains("unsafe_authority"));
    assert!(!exports.contains("LegacyUnsafeAuthority"));
    assert!(!PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/legacy_adopter_unsafe_authority.sql")
        .exists());
    assert_eq!(legacy_adopter_sql_statements().len(), 7);
    let verification = cte_slice(
        source,
        "fn verify_transaction(",
        "fn verify_transaction_settings(",
    );
    assert!(!verification.contains("refuse_unsafe_authority"));
}

#[test]
fn unsupported_catalog_sentinel_covers_the_exact_bounded_fail_closed_inventory() {
    let sql = census_sql();
    for family in [
        "pg_catalog.pg_am",
        "pg_catalog.pg_cast",
        "pg_catalog.pg_collation",
        "pg_catalog.pg_conversion",
        "pg_catalog.pg_event_trigger",
        "pg_catalog.pg_foreign_data_wrapper",
        "pg_catalog.pg_foreign_server",
        "pg_catalog.pg_language",
        "pg_catalog.pg_largeobject_metadata",
        "pg_catalog.pg_operator",
        "pg_catalog.pg_opclass",
        "pg_catalog.pg_opfamily",
        "pg_catalog.pg_publication",
        "pg_catalog.pg_subscription",
        "pg_catalog.pg_statistic_ext",
        "pg_catalog.pg_transform",
        "pg_catalog.pg_ts_config",
        "pg_catalog.pg_ts_dict",
        "pg_catalog.pg_ts_parser",
        "pg_catalog.pg_ts_template",
        "pg_catalog.pg_user_mapping",
    ] {
        assert!(sql.contains(family), "missing unsupported family: {family}");
    }
    assert!(sql.contains("candidate_unsupported_catalog AS MATERIALIZED"));
    assert!(sql.contains("unsupported_catalog_objects AS"));
    assert!(sql.contains("'unsupported_catalog' AS kind"));
    assert!(sql.contains("SELECT 1 FROM candidate_unsupported_catalog LIMIT $1"));
    assert!(sql.contains("FROM candidate_unsupported_catalog"));
    assert!(sql.contains("extension_dependency.deptype = 'e'"));
    assert!(sql.contains("access_method.oid >= 16384"));
    assert!(sql.contains("language.lanispl"));
    assert!(!sql.contains("subconninfo"));
}

#[test]
fn user_mapping_inventory_uses_the_complete_nonsecret_catalog_view() {
    let sql = census_sql();
    let unsupported = cte_slice(
        sql,
        "candidate_unsupported_catalog AS MATERIALIZED",
        "candidate_extensions AS MATERIALIZED",
    );
    assert!(unsupported.contains("FROM pg_catalog.pg_user_mappings AS mapping"));
    assert!(unsupported.contains("mapping.umid"));
    assert!(!unsupported.contains("FROM pg_catalog.pg_user_mapping AS mapping"));
    assert!(!unsupported.contains("umoptions"));
}

#[test]
fn unsupported_catalog_catches_user_casts_and_persistent_system_namespace_objects() {
    let sql = census_sql();
    let protected = cte_slice(
        sql,
        "protected_system_namespaces AS MATERIALIZED",
        "candidate_unsupported_catalog AS MATERIALIZED",
    );
    assert!(protected.contains("nspname IN ('pg_catalog', 'information_schema')"));
    let unsupported = cte_slice(
        sql,
        "candidate_unsupported_catalog AS MATERIALIZED",
        "candidate_schema_acls AS MATERIALIZED",
    );
    assert!(unsupported.contains("cast_row.oid >= 16384"));
    for family in ["pg_class", "pg_proc", "pg_type"] {
        assert!(
            unsupported.contains(&format!("SELECT '{family}'")),
            "missing system-namespace lane: {family}"
        );
    }
    for contract in [
        "protected_system_namespaces",
        "object_namespace.oid IS NOT NULL",
        "dependency.deptype IN ('e', 'i')",
        "relation.relpersistence <> 't'",
    ] {
        assert!(
            unsupported.contains(contract),
            "missing system lane: {contract}"
        );
    }
    for catalog in [
        "pg_collation",
        "pg_conversion",
        "pg_operator",
        "pg_opclass",
        "pg_opfamily",
        "pg_statistic_ext",
        "pg_ts_config",
        "pg_ts_dict",
        "pg_ts_parser",
        "pg_ts_template",
    ] {
        let family = format!("SELECT '{catalog}'");
        let start = unsupported.find(&family).unwrap();
        let tail = &unsupported[start..];
        let end = tail.find("UNION ALL").unwrap_or(tail.len());
        assert!(
            tail[..end].contains("object_namespace.oid IS NOT NULL"),
            "{catalog} lacks system-namespace user-OID lane"
        );
    }
    let live = include_str!("legacy_adopter_postgres.rs");
    assert!(live.contains("CREATE CAST (uuid AS integer) WITH INOUT"));
    assert!(!live.contains("per20_uuid_to_integer"));
    for repro in [
        "CREATE TABLE pg_catalog.per20_system_relation",
        "CREATE FUNCTION pg_catalog.per20_system_routine",
        "CREATE TYPE pg_catalog.per20_system_type",
    ] {
        assert!(live.contains(repro), "missing compiled live repro: {repro}");
    }
}

#[test]
fn catalog_overflow_protocol_reports_closed_resource_and_measured_limits() {
    let sql = census_sql();
    let overflow = cte_slice(
        sql,
        "overflow_candidates AS MATERIALIZED",
        "catalog_status AS",
    );
    for contract in [
        "'catalog_rows'",
        "'partition_rows'",
        "'extension_members'",
        "'extension_role_identities'",
        "'sequence_ownership'",
        "$1 - 1",
        "$2 - 1",
        "$3 - 1",
        "$4 - 1",
        "$5 - 1",
        "$6 - 1",
        "ORDER BY candidate.priority, candidate.actual DESC",
    ] {
        assert!(
            overflow.contains(contract),
            "missing overflow proof: {contract}"
        );
    }
    let final_select = cte_slice(sql, "catalog_output AS", "SELECT\n    output.kind");
    for column in [
        "status.overflow_resource",
        "status.overflow_actual",
        "status.overflow_max",
    ] {
        assert!(
            final_select.contains(column),
            "missing overflow column: {column}"
        );
    }
    let source = include_str!("../src/legacy_adopter.rs");
    for variant in [
        LegacyBoundedResource::CatalogRows,
        LegacyBoundedResource::PartitionRows,
        LegacyBoundedResource::ExtensionMembers,
        LegacyBoundedResource::ExtensionDependencyAddresses,
        LegacyBoundedResource::ExtensionRoleIdentities,
        LegacyBoundedResource::SequenceOwnership,
    ] {
        assert_ne!(variant, LegacyBoundedResource::CensusRows);
    }
    assert!(source.contains("fn decode_catalog_overflow("));
    assert!(source.contains("parse_bounded_resource"));
}

#[test]
fn non_catalog_bounds_use_owned_parameters_and_one_null_sentinel_row() {
    let sql = census_sql();
    let sequence_dependencies = cte_slice(
        sql,
        "candidate_sequence_dependencies AS MATERIALIZED",
        "relation_shapes AS",
    );
    assert!(sequence_dependencies.contains("LIMIT $5"));
    let overflow = cte_slice(
        sql,
        "overflow_candidates AS MATERIALIZED",
        "catalog_status AS",
    );
    assert!(overflow.contains("'sequence_ownership'"));
    assert!(overflow.contains("$5 - 1"));
    assert!(overflow.contains("'extension_role_identities'"));
    assert!(overflow.contains("$6 - 1"));
    let output = cte_slice(sql, "catalog_output AS", "SELECT\n    output.kind");
    assert!(output.contains("WHERE status.overflow_resource IS NULL"));
    assert!(output.contains("NULL::pg_catalog.text AS kind"));
    assert!(output.contains("WHERE status.overflow_resource IS NOT NULL"));

    let source = include_str!("../src/legacy_adopter.rs");
    assert!(source.contains("pub const MAX_LEGACY_SEQUENCE_OWNERSHIP: usize = 1;"));
    let reader = cte_slice(source, "fn read_census_rows(", "fn decode_census_rows(");
    assert!(reader.contains("LegacyBoundedResource::SequenceOwnership"));
    assert!(reader.contains("&sequence_ownership_limit"));
    assert!(reader.contains("LegacyBoundedResource::ExtensionRoleIdentities"));
    assert!(reader.contains("&extension_role_identity_limit"));

    let live = include_str!("legacy_adopter_postgres.rs");
    let probe = cte_slice(
        live,
        "fn verify_raw_non_catalog_bounds(",
        "fn verify_canonical_state_after_global_cleanup(",
    );
    for resource in [
        "sequence_ownership",
        "extension_members",
        "extension_dependency_addresses",
        "extension_role_identities",
        "partition_rows",
    ] {
        assert!(probe.contains(resource));
    }
    assert!(probe.contains("assert_eq!(rows.len(), 1)"));
    assert_eq!(probe.matches("Option<String>").count(), 4);
    assert!(probe.contains("assert_eq!(objects, (None, None, None, None))"));
}

#[test]
fn role_digest_covers_bounded_normalized_parameter_privileges() {
    let sql = census_sql();
    let parameter_acls = cte_slice(
        sql,
        "candidate_parameter_acl_entries AS MATERIALIZED",
        "candidate_role_memberships AS MATERIALIZED",
    );
    for contract in [
        "pg_catalog.pg_parameter_acl",
        "pg_catalog.aclexplode(parameter_acl.paracl)",
        "acl.grantee = 0",
        "role_row.rolname = 'babylon_intel'",
        "WHEN grantor_role.rolsuper THEN '$superuser'",
        "WHEN acl.grantor = role_row.oid THEN 'babylon_intel'",
        "acl.privilege_type",
        "acl.is_grantable",
        "LIMIT $1",
    ] {
        assert!(
            parameter_acls.contains(contract),
            "missing parameter ACL: {contract}"
        );
    }
    let roles = cte_slice(sql, "role_objects AS", "unsupported_catalog_objects AS");
    assert!(roles.contains("'parameter_privileges'"));
    assert!(roles.contains("'privilege', parameter_acl.privilege_type"));
    let bounds = cte_slice(sql, "overflow_candidates AS", "catalog_output AS");
    assert!(bounds.contains("candidate_parameter_acl_entries"));
    let live = include_str!("legacy_adopter_postgres.rs");
    let grants = cte_slice(
        live,
        "fn verify_global_parameter_privileges(",
        "fn verify_global_role_config(",
    );
    assert!(grants.contains("TO babylon_intel WITH GRANT OPTION"));
    assert!(grants.contains("ALTER SYSTEM ON PARAMETER work_mem TO PUBLIC"));
    assert_eq!(grants.matches("guard.cleanup()").count(), 1);
}

#[test]
fn default_acl_census_includes_database_owner_rows_with_canonical_identities() {
    let sql = census_sql();
    let defaults = cte_slice(
        sql,
        "candidate_default_acl_entries AS MATERIALIZED",
        "role_objects AS",
    );
    for contract in [
        "defaults.defaclrole IN (own.owner_oid, intel_role.oid)",
        "WHEN defaults.defaclrole = own.owner_oid THEN '$database_owner'",
        "WHEN defaults.defaclrole = intel_role.oid THEN 'babylon_intel'",
        "WHEN acl.grantee = 0 THEN 'PUBLIC'",
        "ELSE '$other_owner:' || grantor_role.rolname",
        "ELSE '$other_owner:' || grantee_role.rolname",
        "LIMIT $1",
    ] {
        assert!(
            defaults.contains(contract),
            "missing default ACL contract: {contract}"
        );
    }
    assert!(!defaults.contains("acl.grantee = role_row.oid"));
    let roles = cte_slice(sql, "role_objects AS", "unsupported_catalog_objects AS");
    for normalized in [
        "defaults.owner_identity",
        "defaults.grantor_identity",
        "defaults.grantee_identity",
    ] {
        assert!(
            roles.contains(normalized),
            "missing normalized default ACL: {normalized}"
        );
    }
    let live = include_str!("legacy_adopter_postgres.rs");
    assert!(live.contains("GRANT SELECT ON TABLES TO PUBLIC"));
}

#[test]
fn sequence_payload_binds_one_bounded_owned_by_dependency() {
    let sql = census_sql();
    let dependencies = cte_slice(
        sql,
        "candidate_sequence_dependencies AS MATERIALIZED",
        "relation_shapes AS",
    );
    for contract in [
        "pg_catalog.pg_depend",
        "'pg_catalog.pg_class'::pg_catalog.regclass",
        "dependency.deptype IN ('a', 'i')",
        "dependency.refobjsubid",
        "LIMIT $5",
    ] {
        assert!(
            dependencies.contains(contract),
            "missing sequence dependency: {contract}"
        );
    }
    let shapes = cte_slice(sql, "relation_payloads AS", "rel_objects AS");
    for contract in [
        "'owned_by'",
        "owned_namespace.nspname",
        "owned_relation.relname",
        "owned_column.attname",
        "dependency.deptype",
    ] {
        assert!(
            shapes.contains(contract),
            "missing sequence payload: {contract}"
        );
    }
    let bounds = cte_slice(sql, "overflow_candidates AS", "catalog_output AS");
    assert!(bounds.contains("candidate_sequence_dependencies"));
    assert!(bounds.contains("'sequence_ownership'"));
    assert!(bounds.contains("$5 - 1 AS max_value"));
}

#[test]
fn event_trigger_startup_permission_has_a_dedicated_typed_refusal() {
    let source = include_str!("../src/legacy_adopter.rs");
    let connection = cte_slice(source, "fn connection_error(", "fn query_error(");
    assert!(connection.contains("SqlState::INSUFFICIENT_PRIVILEGE"));
    assert!(connection.contains("EventTriggerSuppressionUnavailable"));
    assert!(connection.contains("diagnostic"));
}

#[test]
fn subscription_fixture_disables_remote_slot_cleanup() {
    let source = include_str!("legacy_adopter_postgres.rs");
    let fixture = cte_slice(
        source,
        "fn verify_event_publication_and_subscription_refusals(",
        "fn verify_unsafe_routine_refusals(",
    );
    assert!(fixture.contains(
        "WITH (connect = false, enabled = false, create_slot = false, slot_name = NONE)"
    ));
    assert!(fixture.contains("DROP SUBSCRIPTION per20_disabled_subscription"));
}

#[test]
fn review_extension_identity_covers_owner_config_and_bounded_members() {
    let sql = census_sql();
    for field in [
        "candidate_extension_members AS MATERIALIZED",
        "pg_catalog.pg_identify_object",
        "extension_row.extowner",
        "extension_row.extrelocatable",
        "extension_row.extconfig",
        "extension_row.extcondition",
        "dependency.classid",
        "dependency.deptype = 'e'",
    ] {
        assert!(
            sql.contains(field),
            "missing extension identity field: {field}"
        );
    }
    let bounds = cte_slice(sql, "overflow_candidates AS", "catalog_output AS");
    assert!(bounds.contains("extension_member_budget"));
    let roles = cte_slice(
        sql,
        "extension_role_identity_profile AS MATERIALIZED",
        "extension_role_identity_counts AS",
    );
    let intel = roles.find("THEN 'babylon_intel'").unwrap();
    let superuser = roles.find("THEN '$superuser'").unwrap();
    let database_owner = roles.find("THEN '$database_owner'").unwrap();
    assert!(intel < superuser && superuser < database_owner);
}

#[test]
fn extension_authority_profile_is_path_aware_and_cross_owner_portable() {
    let sql = census_sql();
    let profile = cte_slice(
        sql,
        "extension_role_identity_profile AS MATERIALIZED",
        "extension_role_identity_counts AS",
    );
    let intel = profile.find("THEN 'babylon_intel'").unwrap();
    let superuser = profile.find("THEN '$superuser'").unwrap();
    let database_owner = profile.find("THEN '$database_owner'").unwrap();
    assert!(intel < superuser && superuser < database_owner);
    assert!(profile.contains("ELSE '$other_owner:' || role_row.rolname"));

    let extension_payloads = cte_slice(
        sql,
        "candidate_extension_initial_privileges AS MATERIALIZED",
        "candidate_role_configs AS MATERIALIZED",
    );
    assert!(
        extension_payloads
            .matches("JOIN extension_role_identity_profile AS owner_identity")
            .count()
            >= 7
    );
    for identity in ["grantor_identity.identity", "grantee_identity.identity"] {
        assert!(
            extension_payloads.contains(identity),
            "missing path-aware ACL identity: {identity}"
        );
    }
    for (payload, next_payload) in [
        (
            "extension_relation_payloads AS",
            "candidate_extension_language_acls AS MATERIALIZED",
        ),
        (
            "extension_routine_payloads AS",
            "extension_type_payloads AS",
        ),
        (
            "extension_type_payloads AS",
            "candidate_extension_amops AS MATERIALIZED",
        ),
        (
            "extension_language_payloads AS",
            "extension_operator_payloads AS",
        ),
        (
            "extension_operator_payloads AS",
            "extension_operator_class_payloads AS",
        ),
        (
            "extension_operator_class_payloads AS",
            "extension_operator_family_payloads AS",
        ),
        (
            "extension_operator_family_payloads AS",
            "extension_member_payload_components AS MATERIALIZED",
        ),
    ] {
        let payload_slice = cte_slice(extension_payloads, payload, next_payload);
        assert!(
            payload_slice.contains("owner_identity.identity"),
            "extension owner is not profile-backed: {payload}"
        );
    }
    let extensions = cte_slice(
        sql,
        "extension_objects AS",
        "candidate_role_configs AS MATERIALIZED",
    );
    assert!(extensions.contains("owner_identity.identity"));
}

fn assert_extension_member_owner_contract(sql: &str) {
    let member_owners = cte_slice(
        sql,
        "candidate_extension_member_owners AS MATERIALIZED",
        "candidate_extension_role_reference_pairs AS MATERIALIZED",
    );
    for owner in [
        "relation.relowner",
        "routine.proowner",
        "type_row.typowner",
        "language.lanowner",
        "operator_row.oprowner",
        "operator_class.opcowner",
        "operator_family.opfowner",
    ] {
        assert!(
            member_owners.contains(owner),
            "missing extension member owner: {owner}"
        );
    }
}

fn assert_extension_role_reference_contract(sql: &str) {
    let reference_pairs = cte_slice(
        sql,
        "candidate_extension_role_reference_pairs AS MATERIALIZED",
        "candidate_extension_role_references AS MATERIALIZED",
    );
    for contract in [
        "extension_oid",
        "extension_row.extowner",
        "member_owner.role_oid",
        "acl.grantor",
        "acl.grantee <> 0",
    ] {
        assert!(
            reference_pairs.contains(contract),
            "missing extension role reference pair: {contract}"
        );
    }
    let references = cte_slice(
        sql,
        "candidate_extension_role_references AS MATERIALIZED",
        "extension_role_identity_profile AS MATERIALIZED",
    );
    for contract in [
        "SELECT DISTINCT pair.role_oid",
        "ORDER BY pair.role_oid",
        "LIMIT $6",
    ] {
        assert!(
            references.contains(contract),
            "missing bounded role set: {contract}"
        );
    }
}

fn assert_extension_role_profile_contract(sql: &str) {
    let profile = cte_slice(
        sql,
        "extension_role_identity_profile AS MATERIALIZED",
        "extension_role_identity_counts AS",
    );
    assert!(profile.contains("JOIN candidate_extension_role_references AS referenced"));
    assert!(!profile.contains("ORDER BY role_row.rolname"));
    let counts = cte_slice(
        sql,
        "extension_role_identity_counts AS",
        "candidate_extension_dependency_edges AS MATERIALIZED",
    );
    assert!(counts.contains("extension_row.oid AS extension_oid"));
    assert!(counts.contains("referenced_role_oid_count"));
    assert!(counts.contains("resolved_role_oid_count"));
    assert!(counts.contains("canonical_identity_count"));
    assert!(counts.contains("pg_catalog.count(DISTINCT profile.identity)"));
    assert!(counts.contains("GROUP BY extension_row.oid"));
}

fn assert_extension_role_bound_contract(sql: &str) {
    let extensions = cte_slice(sql, "extension_objects AS", "candidate_role_configs AS");
    assert!(extensions.contains("'role_identity_count', role_counts.canonical_identity_count"));
    assert!(extensions.contains("'role_identities_complete', true"));
    assert!(!extensions.contains("'referenced_role_count'"));
    assert!(!extensions.contains("'resolved_role_count'"));
    assert!(extensions
        .contains("role_counts.referenced_role_oid_count = role_counts.resolved_role_oid_count"));
    assert!(!extensions.contains("CROSS JOIN extension_role_identity_counts"));
    let overflow = cte_slice(sql, "overflow_candidates AS", "catalog_status AS");
    assert!(overflow.contains("'extension_role_identities'"));
    assert!(overflow.contains("$6 - 1"));
    let source = include_str!("../src/legacy_adopter.rs");
    assert!(source.contains("MAX_LEGACY_EXTENSION_ROLE_IDENTITIES: usize = 8_192"));
    assert!(source.contains("ExtensionRoleIdentities"));
    let live = include_str!("legacy_adopter_postgres.rs");
    assert!(live.contains("zzzz_per20_extension_grantor"));
    assert!(live.contains("yyyy_per20_extension_role_"));
    assert!(live.contains("FOR role_index IN 0..512 LOOP"));
}

#[test]
fn extension_role_identities_are_exact_bounded_and_complete() {
    let sql = census_sql();
    assert_extension_member_owner_contract(sql);
    assert_extension_role_reference_contract(sql);
    assert_extension_role_profile_contract(sql);
    assert_extension_role_bound_contract(sql);
}

#[test]
fn extension_direct_members_require_allowed_catalogs_and_canonical_addresses() {
    let sql = census_sql();
    let extensions = cte_slice(
        sql,
        "candidate_extension_members AS MATERIALIZED",
        "candidate_role_configs AS MATERIALIZED",
    );
    for catalog in [
        "pg_catalog.pg_am",
        "pg_catalog.pg_cast",
        "pg_catalog.pg_class",
        "pg_catalog.pg_language",
        "pg_catalog.pg_opclass",
        "pg_catalog.pg_operator",
        "pg_catalog.pg_opfamily",
        "pg_catalog.pg_proc",
        "pg_catalog.pg_type",
    ] {
        assert!(
            extensions.contains(catalog),
            "missing member catalog: {catalog}"
        );
    }
    for contract in [
        "member.objsubid = 0",
        "pg_catalog.pg_identify_object_as_address",
        "'type'",
        "'names'",
        "'args'",
        "'raw_member_count'",
        "'safe_member_count'",
        "'payload_member_count'",
        "'unsupported_member_count'",
    ] {
        assert!(
            extensions.contains(contract),
            "missing canonical member contract: {contract}"
        );
    }
    assert!(!extensions.contains("pg_catalog.pg_identify_object("));
}

#[test]
fn extension_member_payloads_cover_definition_acl_init_and_dependency_edges() {
    let sql = census_sql();
    let extensions = cte_slice(
        sql,
        "candidate_extension_members AS MATERIALIZED",
        "candidate_role_configs AS MATERIALIZED",
    );
    for contract in [
        "pg_catalog.pg_aggregate",
        "pg_catalog.pg_amop",
        "pg_catalog.pg_amproc",
        "pg_catalog.pg_init_privs",
        "pg_catalog.pg_shdepend",
        "'variadic'",
        "'transform'",
        "'support'",
        "'aggregate'",
        "'current_acl'",
        "'initial_acl'",
        "'outbound_dependencies'",
        "'inbound_dependencies'",
    ] {
        assert!(
            extensions.contains(contract),
            "missing extension member payload contract: {contract}"
        );
    }
    assert!(extensions.contains("dependency.refobjid = member.extension_oid"));
    assert!(extensions.contains("SELECT DISTINCT"));
}

#[test]
fn extension_dependency_addresses_have_a_separate_closed_typed_bound() {
    let source = include_str!("../src/legacy_adopter.rs");
    assert!(source.contains("MAX_LEGACY_EXTENSION_DEPENDENCY_ADDRESSES: usize = 16_384"));
    assert!(source.contains("ExtensionDependencyAddresses"));
    assert!(source.contains("extension_dependency_addresses"));
    assert!(source.contains("fn extension_dependency_address_bound_refuses_first_excess()"));
    assert!(source.contains("MAX_LEGACY_EXTENSION_DEPENDENCY_ADDRESSES + 1"));
    let sql = census_sql();
    let overflow = cte_slice(
        sql,
        "overflow_candidates AS MATERIALIZED",
        "catalog_status AS",
    );
    assert!(overflow.contains("'extension_dependency_addresses'"));
    assert!(overflow.contains("$4 - 1"));
    let live = include_str!("legacy_adopter_postgres.rs");
    assert!(live.contains("extension_dependency_address_limit"));
    assert!(live.contains("Some(\"extension_dependency_bound\")"));
    assert!(live.contains("verify_unknown_extension_classification(base, template)"));
}

#[test]
fn extension_members_are_globally_gated_before_payload_expansion() {
    let sql = census_sql();
    let gate = cte_slice(
        sql,
        "candidate_extension_members AS MATERIALIZED",
        "extension_relation_members AS MATERIALIZED",
    );
    for contract in [
        "pg_catalog.pg_identify_object_as_address",
        "ORDER BY\n        extension_row.extname",
        "LIMIT $3",
        "extension_member_budget AS MATERIALIZED",
        "bounded_extension_members AS MATERIALIZED",
        "extension_member_count < $3",
        "FROM bounded_extension_members AS member",
    ] {
        assert!(
            gate.contains(contract),
            "missing global member gate: {contract}"
        );
    }
    let gate_position = sql
        .find("bounded_extension_members AS MATERIALIZED")
        .unwrap();
    for target in [
        "safe_extension_members AS MATERIALIZED",
        "shape_targets AS MATERIALIZED",
        "routine_targets AS MATERIALIZED",
        "domain_targets AS MATERIALIZED",
        "user_type_targets AS MATERIALIZED",
        "candidate_extension_member_addresses AS MATERIALIZED",
    ] {
        let target_position = sql.find(target).unwrap();
        assert!(
            gate_position < target_position,
            "gate must precede {target}"
        );
    }
    let overflow = cte_slice(
        sql,
        "overflow_candidates AS MATERIALIZED",
        "catalog_status AS",
    );
    assert!(overflow.contains("FROM extension_member_budget AS budget"));
    assert!(overflow.contains("budget.extension_member_count, $3 - 1"));
}

#[test]
fn extension_dependency_addresses_replace_internal_toast_oids_with_parent_addresses() {
    let sql = census_sql();
    let toast = cte_slice(
        sql,
        "candidate_internal_toast_dependencies AS MATERIALIZED",
        "extension_member_dependency_payloads AS MATERIALIZED",
    );
    for contract in [
        "owner_relation.reltoastrelid",
        "pg_catalog.pg_index",
        "toast_relation.relkind = 't'",
        "toast_index.relkind = 'i'",
        "pg_catalog.pg_identify_object_as_address",
        "owner_count = 1",
        "'internal_toast_table'",
        "'internal_toast_index'",
    ] {
        assert!(
            toast.contains(contract),
            "missing stable TOAST contract: {contract}"
        );
    }
    assert!(!sql.contains("pg_toast_"));
}

#[test]
fn extension_dependency_role_addresses_use_the_bounded_canonical_profile() {
    let sql = census_sql();
    let reference_pairs = cte_slice(
        sql,
        "candidate_extension_role_reference_pairs AS MATERIALIZED",
        "candidate_extension_role_references AS MATERIALIZED",
    );
    for contract in [
        "pg_catalog.pg_shdepend",
        "dependency.refclassid = 'pg_catalog.pg_authid'::pg_catalog.regclass",
        "dependency.classid = 'pg_catalog.pg_authid'::pg_catalog.regclass",
        "dependency.refobjid",
        "dependency.objid",
    ] {
        assert!(
            reference_pairs.contains(contract),
            "missing dependency role reference: {contract}"
        );
    }
    let dependency_edges = cte_slice(
        sql,
        "candidate_extension_dependency_edges AS MATERIALIZED",
        "candidate_internal_toast_dependencies AS MATERIALIZED",
    );
    for contract in [
        "SELECT DISTINCT edge.*",
        "FROM candidate_extension_member_owners AS member_owner",
        "'outbound'",
        "'pg_shdepend'",
        "'pg_catalog.pg_authid'::pg_catalog.regclass",
        "member_owner.role_oid",
        "'o'",
    ] {
        assert!(
            dependency_edges.contains(contract),
            "missing canonical member-owner dependency: {contract}"
        );
    }
    assert!(dependency_edges.contains("dependency.refobjid"));
    let addresses = cte_slice(
        sql,
        "extension_dependency_addresses AS",
        "extension_member_dependency_payloads AS MATERIALIZED",
    );
    for contract in [
        "JOIN extension_role_identity_profile AS dependency_role_identity",
        "edge.other_classid = 'pg_catalog.pg_authid'::pg_catalog.regclass",
        "'type', 'role'",
        "pg_catalog.jsonb_build_array(dependency_role_identity.identity)",
        "'args', '[]'::pg_catalog.jsonb",
        "dependency_role_identity.role_oid IS NOT NULL",
    ] {
        assert!(
            addresses.contains(contract),
            "missing canonical dependency role address: {contract}"
        );
    }
    let live = include_str!("legacy_adopter_postgres.rs");
    assert!(live.contains("fn verify_extension_superuser_owner_portability("));
    assert!(live.contains("per20_extension_super_owner_a NOLOGIN SUPERUSER"));
    assert!(live.contains("per20_extension_super_owner_b NOLOGIN SUPERUSER"));
    assert!(live.contains("raw_extension_member_owner_dependency_count(&first_config)"));
    assert!(live.contains("assert_eq!(first_raw_owner_dependencies, 0)"));
    assert!(live.contains("assert_eq!(first_raw_owner_dependencies_after, 1)"));
    assert!(live.contains("assert_eq!(baseline_digest, first_digest)"));
    assert!(live.contains("first_outcome.is_ok(),"));
    assert!(live.contains("second_outcome.is_ok(),"));
    assert!(live.contains("assert_eq!(first_outcome, second_outcome)"));
    assert!(live.contains("owners.cleanup()"));
}

#[test]
fn extension_window_routines_bind_canonical_implementation_identity() {
    let sql = census_sql();
    let routines = cte_slice(sql, "routine_overloads AS", "routine_objects AS");
    for contract in [
        "routine.prokind IN ('f', 'p', 'w')",
        "'source', routine.prosrc",
        "'binary', coalesce(routine.probin, '')",
        "WHEN routine.prosqlbody IS NULL THEN ''",
        "ELSE pg_catalog.pg_get_functiondef(routine.oid)",
    ] {
        assert!(
            routines.contains(contract),
            "missing routine implementation identity: {contract}"
        );
    }
    let live = cte_slice(
        include_str!("legacy_adopter_postgres.rs"),
        "fn verify_extension_window_routine_body_refusal(",
        "fn verify_extension_acl_role_completeness(",
    );
    for contract in [
        "pg_catalog.pg_get_functiondef(target.oid)",
        "target.proname = 'st_clusterdbscan'",
        "target.prokind = 'w'",
        "replacement.proname = 'st_clusterwithinwin'",
        "replacement.prokind = 'w'",
        "assert_eq!(binary, replacement_binary)",
        "definition.starts_with(\"CREATE OR REPLACE FUNCTION public.st_clusterdbscan(\")",
        ".replacen(&source_body, &replacement_body, 1)",
        "mutate(&config, &replacement_definition)",
    ] {
        assert!(
            live.contains(contract),
            "missing live window-routine DDL contract: {contract}"
        );
    }
    assert!(!live.contains("SET allow_system_table_mods = on"));
    assert!(!live.contains("UPDATE pg_catalog.pg_proc AS routine"));
}

#[test]
fn synthetic_authority_role_namespace_is_globally_reserved() {
    let sql = census_sql();
    let unsupported = cte_slice(
        sql,
        "candidate_unsupported_catalog AS MATERIALIZED",
        "candidate_extensions AS MATERIALIZED",
    );
    for token in ["$database_owner", "$superuser", "ALL", "PUBLIC"] {
        assert!(
            unsupported.contains(&format!("role_row.rolname = '{token}'")),
            "missing reserved authority token: {token}"
        );
    }
    assert!(unsupported.contains("pg_catalog.starts_with(role_row.rolname, '$other_owner:')"));
    assert!(unsupported.contains("SELECT 'pg_roles'"));
    let live = include_str!("legacy_adopter_postgres.rs");
    for role_name in [
        "\"$database_owner\"",
        "\"$superuser\"",
        "\"ALL\"",
        "\"PUBLIC\"",
        "\"$other_owner:collision\"",
    ] {
        assert!(
            live.contains(role_name),
            "missing live role collision: {role_name}"
        );
    }
    assert!(live.contains("fn verify_reserved_role_name_refusals("));
    assert!(live.contains("assert_unsupported_catalog_and_lock(&config, \"pg_roles\")"));
}

#[test]
fn live_extension_member_mutations_expect_only_the_extension_key() {
    let live = include_str!("legacy_adopter_postgres.rs");
    for mutation in [
        "CREATE OR REPLACE FUNCTION public.postgis_version()",
        "ALTER FUNCTION public.postgis_version() SECURITY DEFINER",
        "REVOKE EXECUTE ON FUNCTION public.postgis_version() FROM PUBLIC",
        "ALTER TABLE public.spatial_ref_sys SET (fillfactor = 70)",
    ] {
        assert!(live.contains(mutation), "missing live mutation: {mutation}");
    }
    let extension_cases = cte_slice(
        live,
        "fn verify_extension_identity_refusals(",
        "fn verify_census_column_bound(",
    );
    assert!(extension_cases.contains("assert_census_change"));
    assert!(!extension_cases.contains("assert_changed_contains"));
}

#[test]
fn live_mutation_helpers_require_exact_error_vectors() {
    let live = include_str!("legacy_adopter_postgres.rs");
    let census = cte_slice(
        live,
        "fn assert_census_mismatch(",
        "fn assert_unsupported_catalog_and_lock(",
    );
    for proof in [
        "missing_objects: expected_missing",
        "changed_objects: expected_changed",
        "extra_objects: expected_extra",
    ] {
        assert!(
            census.contains(proof),
            "census helper is not exact: {proof}"
        );
    }
    assert!(!live.contains("fn assert_changed_contains("));
    assert!(!live.contains("LegacyAdopterError::CensusMismatch { .. }"));
    let unsupported = cte_slice(
        live,
        "fn assert_unsupported_catalog_and_lock(",
        "fn assert_unsupported_extras(",
    );
    assert!(unsupported.contains("objects: vec![expected]"));
    let bounds = cte_slice(
        live,
        "fn assert_bounds_and_lock(",
        "fn assert_outcome_and_lock(",
    );
    for field in ["resource", "actual", "max"] {
        assert!(bounds.contains(field), "bounds helper omits {field}");
    }
}

#[test]
fn review_adoption_report_defers_database_owner_authority_proof() {
    let source = include_str!("../src/legacy_adopter.rs");
    assert!(source.contains("pub enum LegacyOwnerAuthorityDisposition"));
    assert!(source.contains("DeferredToRustMigratorPreflight"));
    assert!(source.contains("pub owner_authority: LegacyOwnerAuthorityDisposition"));
    assert!(!source.contains("owner_authority_verified: true"));
}

#[test]
fn review_postgres_image_builds_checksum_pinned_sources_on_one_abi() {
    let dockerfile = include_str!("../../../../docker/postgres/Dockerfile");
    let action = include_str!("../../../../.github/actions/postgres-up/action.yml");
    let runner = include_str!("../../../../tools/run_rust_legacy_adopter_pg.sh");
    assert!(dockerfile.starts_with("# syntax=docker/dockerfile:1.6\n"));
    for pinned in [
        "postgis/postgis:17-3.5-alpine@sha256:08f4b1e1f4a571008c60272ceb9e0d1f9f8f643792d006b74a35b1bec44c2218",
        "sha256:c54c119e1d9a578d5cbcce22f6c66dab2b5a45219fc2b260619807f7f061e53a",
        "sha256:0da8a392a6ff77e76b60e6a331a49497d0935b6b7b6899da7a3e2786139b0441",
        "sha256:6f88a5cbdde31666f4b6c1a6b75c51dcbeffe58f9a7d2b26e502d5a6e5e14d44",
        "OPTFLAGS=\"\"",
    ] {
        assert!(dockerfile.contains(pinned), "missing image input {pinned}");
    }
    for floating in ["apt-get", "apt-archive", ".deb", "gcc:12.2.0-bullseye"] {
        assert!(
            !dockerfile.contains(floating),
            "retired Debian build input: {floating}"
        );
    }
    assert!(runner.contains(
        "env DOCKER_BUILDKIT=1 \\\n  timeout --signal=TERM --kill-after=10s 180s \\\n  docker build"
    ));
    assert!(action.contains("reproduces the pinned base and"));
    assert!(action.contains("checksum-pinned source runtime contract"));
    assert!(!action.contains("cold build produces the identical image"));
}

#[test]
fn census_acl_order_uses_normalized_identities() {
    let sql = census_sql();
    assert!(!sql.contains("ORDER BY acl.grantee"));
    assert!(sql.matches("WHEN acl.grantee = 0 THEN 'PUBLIC'").count() >= 6);
}

#[test]
fn database_identifiers_accept_postgresql_names_but_fixture_names_remain_strict() {
    let quoted = LegacyObjectKey::from_database(
        LegacyObjectKind::Relation,
        "Quoted Schema",
        "Míxed-Case Table",
    )
    .unwrap();
    assert_eq!(quoted.schema(), "Quoted Schema");
    assert_eq!(quoted.name(), "Míxed-Case Table");
    assert!(LegacyObjectKey::from_database(LegacyObjectKind::Relation, "", "table").is_err());
    assert!(
        LegacyObjectKey::from_database(LegacyObjectKind::Relation, "schema", &"é".repeat(32))
            .is_err()
    );
    assert!(LegacyObjectKey::new(LegacyObjectKind::Relation, "Quoted Schema", "table").is_err());
}

#[test]
fn census_covers_relation_options_and_sequence_acl_defaults() {
    let sql = census_sql();
    assert!(sql.contains("candidate_relation_options AS MATERIALIZED"));
    assert!(sql.contains("relation.reloptions"));
    assert!(sql.contains("WHEN c.relkind = 'S' THEN 's'"));
    assert!(!sql.contains("WHEN c.relkind = 'S' THEN 'S'"));
}

#[test]
fn census_bounds_candidates_and_covers_effective_write_authority() {
    let sql = census_sql();
    assert_census_candidate_order(sql);
    assert_census_subordinate_bounds(sql);
    assert!(sql.contains("database_objects AS"));
    assert!(sql.contains("pg_database_collation_actual_version"));
    assert!(sql.contains("owner_is_babylon_intel"));
    for field in [
        "server_version_num",
        "database_row.encoding",
        "database_row.datlocprovider",
        "database_row.datcollate",
        "database_row.datctype",
        "database_row.datlocale",
        "database_row.daticurules",
        "database_row.datcollversion",
        "database_row.dattablespace",
        "database_row.datacl",
    ] {
        assert!(sql.contains(field));
    }
    assert!(sql.contains("column_acl"));
    assert!(sql.contains("role_row.rolconfig"));
    assert!(sql.contains("pg_catalog.pg_rewrite"));
    assert!(sql.contains("pg_catalog.pg_policy"));
    assert!(sql.contains("pg_catalog.pg_trigger"));
    assert!(sql.contains("routine_objects AS"));
    assert!(sql.contains("user_type_objects AS"));
    assert!(sql.contains("dependency.classid"));
    assert!(sql.contains("status.overflow_resource"));
}

fn assert_census_candidate_order(sql: &str) {
    let namespaces = sql.find("candidate_namespaces AS MATERIALIZED").unwrap();
    let relations = sql.find("candidate_relations AS MATERIALIZED").unwrap();
    let attributes = sql.find("candidate_attributes AS MATERIALIZED").unwrap();
    let candidate_indexes = sql.find("candidate_indexes AS MATERIALIZED").unwrap();
    let shapes = sql.find("relation_shapes AS").unwrap();
    let indexes = sql.find("index_shapes AS").unwrap();
    assert!(namespaces < relations && relations < attributes);
    assert!(attributes < candidate_indexes && candidate_indexes < shapes);
    assert!(candidate_indexes < indexes);
}

fn assert_census_subordinate_bounds(sql: &str) {
    let bounded_ctes = [
        "candidate_relation_acls AS MATERIALIZED",
        "candidate_column_acls AS MATERIALIZED",
        "candidate_constraints AS MATERIALIZED",
        "candidate_policies AS MATERIALIZED",
        "candidate_rules AS MATERIALIZED",
        "candidate_triggers AS MATERIALIZED",
        "candidate_role_memberships AS MATERIALIZED",
        "candidate_default_acl_entries AS MATERIALIZED",
        "candidate_database_setting_configs AS MATERIALIZED",
        "candidate_database_acls AS MATERIALIZED",
        "candidate_schema_acls AS MATERIALIZED",
        "candidate_domain_acls AS MATERIALIZED",
        "candidate_domain_constraints AS MATERIALIZED",
        "candidate_routine_configs AS MATERIALIZED",
        "candidate_routine_acls AS MATERIALIZED",
        "candidate_user_type_acls AS MATERIALIZED",
        "candidate_enum_labels AS MATERIALIZED",
        "candidate_composite_attributes AS MATERIALIZED",
        "candidate_attribute_options AS MATERIALIZED",
        "candidate_attribute_fdw_options AS MATERIALIZED",
        "candidate_relation_options AS MATERIALIZED",
        "candidate_policy_roles AS MATERIALIZED",
        "candidate_index_options AS MATERIALIZED",
        "candidate_relation_parents AS MATERIALIZED",
    ];
    for bounded_cte in bounded_ctes {
        assert!(sql.contains(bounded_cte), "missing {bounded_cte}");
    }
    let catalog_bounds = cte_slice(sql, "overflow_candidates AS", "catalog_output AS");
    for bounded_name in [
        "candidate_attributes",
        "candidate_relation_acls",
        "candidate_column_acls",
        "candidate_constraints",
        "candidate_policies",
        "candidate_rules",
        "candidate_triggers",
        "candidate_role_memberships",
        "candidate_default_acl_entries",
        "candidate_database_setting_configs",
        "candidate_database_acls",
        "candidate_schema_acls",
        "candidate_domain_acls",
        "candidate_domain_constraints",
        "candidate_routine_configs",
        "candidate_routine_acls",
        "candidate_user_type_acls",
        "candidate_enum_labels",
        "candidate_composite_attributes",
        "candidate_attribute_options",
        "candidate_attribute_fdw_options",
        "candidate_relation_options",
        "candidate_policy_roles",
        "candidate_index_options",
        "candidate_relation_parents",
    ] {
        assert!(
            catalog_bounds.contains(bounded_name),
            "unbounded {bounded_name}"
        );
    }
}

#[test]
fn extension_owned_namespaces_do_not_hide_nonextension_objects() {
    let sql = census_sql();
    let namespaces = cte_slice(
        sql,
        "candidate_namespaces AS MATERIALIZED",
        "candidate_unsupported_catalog AS MATERIALIZED",
    );
    assert!(!namespaces.contains("dependency.classid"));
    let schemas = cte_slice(sql, "schema_objects AS", "schema_grants AS");
    assert!(schemas.contains("'pg_catalog.pg_namespace'::pg_catalog.regclass"));
}

#[test]
fn stamp_query_reads_only_a_bounded_prefix_and_length() {
    let sql = legacy_adopter_sql_statements()
        .iter()
        .take(8)
        .find(|statement| statement.kind() == babylon_persistence::LegacyAdopterSqlKind::ReadStamps)
        .unwrap()
        .sql();
    assert!(sql.contains("pg_catalog.left"));
    assert!(sql.contains("pg_catalog.octet_length"));
    assert!(sql.contains("LIMIT $1"));
}

#[test]
fn connection_startup_options_pin_read_only_timeouts_and_search_path() {
    assert_eq!(
        LEGACY_ADOPTER_STARTUP_OPTIONS,
        "-c default_transaction_read_only=on -c statement_timeout=5000ms \
         -c lock_timeout=5000ms -c idle_in_transaction_session_timeout=5000ms \
         -c quote_all_identifiers=off -c search_path=pg_catalog -c jit=off \
         -c event_triggers=off"
    );
    let settings_sql = legacy_adopter_sql_statements()
        .iter()
        .find(|statement| statement.kind() == LegacyAdopterSqlKind::TransactionSettings)
        .unwrap()
        .sql();
    assert!(settings_sql.contains("pg_catalog.current_setting('jit')"));
    assert!(settings_sql.contains("pg_catalog.current_setting('event_triggers')"));
}

#[test]
fn transaction_settings_verify_every_owned_startup_guard() {
    let source = include_str!("../src/legacy_adopter.rs");
    let verification = cte_slice(
        source,
        "fn verify_transaction_settings(",
        "fn refuse_authority_schemas(",
    );
    for proof in [
        "statement_timeout == \"5s\"",
        "lock_timeout == \"5s\"",
        "idle_timeout == \"5s\"",
        "quote_all_identifiers == \"off\"",
    ] {
        assert!(
            verification.contains(proof),
            "missing setting proof: {proof}"
        );
    }
    let live = include_str!("legacy_adopter_postgres.rs");
    let hostile = cte_slice(
        live,
        "fn verify_hostile_caller_and_connection_redaction(",
        "fn verify_lock_outcome(",
    );
    for hostile_value in [
        "statement_timeout=0",
        "lock_timeout=0",
        "idle_in_transaction_session_timeout=0",
        "quote_all_identifiers=on",
    ] {
        assert!(
            hostile.contains(hostile_value),
            "missing hostile setting: {hostile_value}"
        );
    }
}

#[test]
fn login_event_trigger_harness_proves_preconnect_suppression() {
    let source = include_str!("legacy_adopter_postgres.rs");
    let hostile = cte_slice(
        source,
        "fn verify_hostile_caller_and_connection_redaction(",
        "fn verify_lock_outcome(",
    );
    assert!(hostile.contains("-c event_triggers=on"));

    let login = cte_slice(
        source,
        "fn verify_login_event_trigger_suppression(",
        "fn verify_event_publication_and_subscription_refusals(",
    );
    for proof in [
        "ON login",
        "SECURITY DEFINER",
        "event_triggers_disabled_config",
        "LegacyObjectKind::Database",
        "LegacyAdopterError::EventTriggerSuppressionUnavailable",
        "assert_login_canary_unchanged",
        "assert_lock_released",
    ] {
        assert!(
            login.contains(proof),
            "missing login-trigger proof: {proof}"
        );
    }

    let scratch_role = cte_slice(source, "impl ScratchRole", "impl Drop for ScratchRole");
    assert_eq!(
        scratch_role
            .matches("GRANT SET ON PARAMETER event_triggers")
            .count(),
        1
    );
    assert!(scratch_role.contains("REVOKE SET ON PARAMETER event_triggers"));
    assert!(scratch_role.contains("fn create_without_event_trigger_set"));
}

#[test]
fn identifier_byte_validation_loops_are_statically_bounded() {
    let source = include_str!("../src/legacy_adopter.rs");
    assert_eq!(
        source
            .matches("take(POSTGRES_IDENTIFIER_MAX_BYTES + 1)")
            .count(),
        2
    );
}

#[test]
fn connection_target_requires_one_literal_local_endpoint() {
    let mut loopback = Config::new();
    loopback.host("127.0.0.1").port(5432);
    assert_eq!(validate_legacy_connection_target(&loopback), Ok(()));

    for (host, reason) in [
        ("localhost", LegacyConnectionTargetRejection::NonLoopbackTcp),
        ("192.0.2.1", LegacyConnectionTargetRejection::NonLoopbackTcp),
    ] {
        let mut config = Config::new();
        config.host(host);
        assert_eq!(
            validate_legacy_connection_target(&config),
            Err(LegacyAdopterError::UnsupportedConnectionTarget { reason })
        );
    }

    let missing = Config::new();
    assert_eq!(
        validate_legacy_connection_target(&missing),
        Err(LegacyAdopterError::UnsupportedConnectionTarget {
            reason: LegacyConnectionTargetRejection::MissingHost,
        })
    );
    let mut multiple = Config::new();
    multiple.host("127.0.0.1").host("::1");
    assert_eq!(
        validate_legacy_connection_target(&multiple),
        Err(LegacyAdopterError::UnsupportedConnectionTarget {
            reason: LegacyConnectionTargetRejection::MultipleHosts,
        })
    );
    let mut ports = Config::new();
    ports.host("127.0.0.1").port(5432).port(5433);
    assert_eq!(
        validate_legacy_connection_target(&ports),
        Err(LegacyAdopterError::UnsupportedConnectionTarget {
            reason: LegacyConnectionTargetRejection::MultiplePorts,
        })
    );
    let mut redirected = Config::new();
    redirected
        .host("127.0.0.1")
        .hostaddr(IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_eq!(
        validate_legacy_connection_target(&redirected),
        Err(LegacyAdopterError::UnsupportedConnectionTarget {
            reason: LegacyConnectionTargetRejection::HostAddressOverride,
        })
    );
    let mut caller_options = Config::new();
    caller_options
        .host("127.0.0.1")
        .port(1)
        .options("-c search_path=redirected,public");
    assert_eq!(
        validate_legacy_connection_target(&caller_options),
        Err(LegacyAdopterError::UnsupportedConnectionTarget {
            reason: LegacyConnectionTargetRejection::StartupOptionsOverride,
        })
    );
    #[cfg(unix)]
    {
        let mut absolute_socket = Config::new();
        absolute_socket.host_path("/var/run/postgresql");
        assert_eq!(validate_legacy_connection_target(&absolute_socket), Ok(()));
        let mut relative_socket = Config::new();
        relative_socket.host_path("relative/socket");
        assert_eq!(
            validate_legacy_connection_target(&relative_socket),
            Err(LegacyAdopterError::UnsupportedConnectionTarget {
                reason: LegacyConnectionTargetRejection::NonAbsoluteUnixSocket,
            })
        );
    }
}

#[test]
fn engine_manifests_remain_postgres_free() {
    let manifests = [
        include_str!("../../babylon-kernel/Cargo.toml"),
        include_str!("../../babylon-graph/Cargo.toml"),
        include_str!("../../babylon-tick/Cargo.toml"),
    ];
    for manifest in manifests {
        assert!(!manifest.to_ascii_lowercase().contains("postgres"));
    }
}

#[test]
fn postgres_ci_and_sanctioned_merger_share_the_pinned_runtime_name() {
    let workflow = include_str!("../../../../.github/workflows/ci.yml");
    let merge_policy = include_str!("../../../../tools/pr_policy.py");
    let sanctioned_merger = include_str!("../../../../tools/pr_merge.py");
    let critical_check = "Postgres Integration Tier (PG 17, pinned runtime)";
    assert!(workflow.contains(&format!("    name: {critical_check}")));
    assert!(merge_policy.contains(&format!("\"{critical_check}\",")));
    assert!(sanctioned_merger.contains("from tools.pr_policy import"));
    assert!(sanctioned_merger.contains("manifest = manifest_for_base(base_ref)"));
    assert!(!sanctioned_merger.contains(&format!("\"{critical_check}\",")));
    assert!(workflow.contains("- uses: ./.github/actions/bootstrap-persistence"));
    assert!(workflow.contains("run: tools/run_rust_legacy_adopter_pg.sh"));
    assert!(!workflow.contains("uses: ./.github/actions/fetch-reference-db"));
    assert!(!workflow.contains("digest-pinned base"));
}

#[test]
fn live_adopter_tests_are_split_between_pr_and_weekly_cadences() {
    let mise = include_str!("../../../../.mise.toml");
    let pr_workflow = include_str!("../../../../.github/workflows/ci.yml");
    let weekly_workflow = include_str!("../../../../.github/workflows/weekly-pg-integration.yml");
    let runner = include_str!("../../../../tools/run_rust_legacy_adopter_pg.sh");
    let task = toml_section(mise, "[tasks.\"test:rust-legacy-adopter-pg\"]");
    assert!(task.contains("run = \"tools/run_rust_legacy_adopter_pg.sh\""));
    assert!(runner.contains("cd \"$REPO_ROOT/rust\""));
    assert!(runner.contains("timeout --signal=TERM --kill-after=10s 900s"));
    assert!(runner
        .contains("cargo test -p babylon-persistence --test legacy_adopter_postgres --locked --"));
    assert!(runner.contains("--ignored --test-threads=1"));
    assert!(!runner.contains("--manifest-path"));

    assert_pr_live_cadence(pr_workflow);
    assert_weekly_live_cadence(weekly_workflow);

    let synthetic = "jobs:\n  pg-integration:\n    run: true\n  unrelated:\n    run: tools/run_rust_legacy_adopter_pg.sh\n  security:\n    run: true\n";
    assert!(!yaml_job(synthetic, "  pg-integration:").contains("run_rust_legacy_adopter_pg"));
}

fn assert_pr_live_cadence(pr_workflow: &str) {
    let pr_shards = yaml_job(pr_workflow, "  pg-integration-shards:");
    assert_eq!(
        pr_shards
            .matches("run: tools/run_rust_legacy_adopter_pg.sh")
            .count(),
        1
    );
    for focus in [
        "clean_bootstrap",
        "h3_atomicity",
        "rust_persistence_runtime",
        "installed_mutation",
        "archive_worker",
        "reader_role",
    ] {
        assert!(
            pr_shards.contains(&format!("          - {focus}\n")),
            "the PR matrix pins the {focus} focus"
        );
    }
    assert!(pr_shards.contains(
        "- name: Rust PostgreSQL contract\n        timeout-minutes: 69\n        env:\n          BABYLON_LEGACY_ADOPTER_LIVE_FOCUS: ${{ matrix.focus }}\n        run: tools/run_rust_legacy_adopter_pg.sh"
    ));
    assert!(!pr_shards.contains("BABYLON_LEGACY_ADOPTER_TEST_DSN"));
    assert!(!pr_shards.contains("cargo doc"));
    let pr_aggregator = yaml_job(pr_workflow, "  pg-integration:");
    assert!(pr_aggregator.contains("needs: pg-integration-shards"));
    assert!(pr_aggregator.contains("if: always()"));
    assert!(
        pr_aggregator.contains("run: test '${{ needs.pg-integration-shards.result }}' = success")
    );
    assert!(!pr_aggregator.contains("run_rust_legacy_adopter_pg"));
}

fn assert_weekly_live_cadence(weekly_workflow: &str) {
    let weekly_job = yaml_job(weekly_workflow, "  exhaustive-legacy-adopter:");
    assert!(weekly_job.contains("name: Exhaustive Rust legacy adopter matrix (dev HEAD)"));
    assert!(weekly_job.contains("timeout-minutes: 40"));
    assert!(weekly_job.contains("ref: dev"));
    assert!(weekly_job.contains("- uses: ./.github/actions/bootstrap-persistence"));
    assert!(!weekly_job.contains("- uses: ./.github/actions/bootstrap-python"));
    assert!(!weekly_job.contains("uses: jdx/mise-action"));
    assert!(weekly_job.contains(
        "- name: Exhaustive adopter matrix and rollback proof\n        timeout-minutes: 31\n        run: tools/run_rust_legacy_adopter_pg.sh"
    ));
    assert!(!weekly_job.contains("BABYLON_LEGACY_ADOPTER_LIVE_FOCUS"));
    assert!(!weekly_job.contains("CI_REFDB_READY"));
}

#[test]
fn pr_focus_reuses_the_postgres_atomicity_and_installed_mutation_contracts() {
    let runner = include_str!("../../../../tools/run_rust_legacy_adopter_pg.sh");
    let live = include_str!("legacy_adopter_postgres.rs");
    let installer = include_str!("support/h3_reference_installer_postgres.rs");
    let writer = include_str!("../src/runtime.rs");
    let focused =
        "schema_epoch::live_rollback_tests::h3_installer_rollback_and_ambiguous_commit_reconciliation_are_atomic";
    let full =
        "schema_epoch::live_rollback_tests::rollback_and_ambiguous_commit_reconciliation_are_atomic";

    assert!(runner.contains("BABYLON_LEGACY_ADOPTER_LIVE_FOCUS:-}"));
    assert!(runner.contains(
        "\"\" | clean_bootstrap | h3_atomicity | installed_mutation | schema_epoch_fresh | schema_epoch_matrix | \\\n    schema_epoch_rollback | schema_epoch_v5_census | schema_epoch_v6_census | schema_epoch_v7_census | \\\n    h3_pg_oracle | h3_reference_installer | h3_shadow_backfill | \\\n    rust_persistence_runtime | archive_worker | reader_role | client_dossier | runtime_census_v2 | pr)"
    ));
    assert!(runner.contains("[ \"$LIVE_FOCUS\" = \"h3_pg_oracle\" ]"));
    assert!(runner.contains("[ \"$LIVE_FOCUS\" = \"h3_reference_installer\" ]"));
    assert!(runner.contains("[ \"$LIVE_FOCUS\" = \"h3_shadow_backfill\" ]"));
    assert!(runner.contains("[ \"$LIVE_FOCUS\" = \"rust_persistence_runtime\" ]"));
    assert_eq!(runner.matches("runtime::live_tests::live_").count(), 1);
    for live_test in [
        "fn live_marker_last_commit_and_restart_are_atomic()",
        "fn live_commit_ambiguity_reconciliation_is_exact()",
    ] {
        assert_eq!(writer.matches(live_test).count(), 1);
    }
    assert!(live.contains("Some(\"h3_pg_oracle\")"));
    assert!(live.contains("Some(\"h3_reference_installer\")"));
    assert_eq!(runner.matches(focused).count(), 1);
    assert_eq!(runner.matches(full).count(), 1);
    assert!(runner.contains("--ignored --exact --test-threads=1"));
    assert!(!runner.contains("cargo test -p babylon-persistence --lib \"$"));
    assert!(live.contains("Some(\"installed_mutation\")"));
    assert!(live.contains(
        "h3_reference_installer_postgres::verify_h3_reference_installed_mutations(base)"
    ));
    let installed_focus = cte_slice(
        live,
        "Some(\"installed_mutation\")",
        "Some(\"h3_reference_release\")",
    );
    assert!(installed_focus.contains("verify_h3_installed_mutations(phases, base)"));
    let installed_helper = cte_slice(
        live,
        "fn verify_h3_installed_mutations(",
        "fn verify_h3_pg_oracle_in_scratch(",
    );
    assert!(installed_helper.contains("phases.run(\"h3_installed_mutations\""));
    assert!(installer.contains("pub(super) fn verify_h3_reference_installed_mutations("));
    assert!(installer.contains("verify_installed_state_conflicts(base, &cohort)"));
    assert_eq!(installer.matches("mutate_orphan_membership,").count(), 1);

    let broad = runner
        .find("--test legacy_adopter_postgres")
        .expect("default broad contract must remain");
    let full_position = runner
        .find(full)
        .expect("default full contract must remain");
    assert!(broad < full_position);
}

fn assert_epoch_seven_source_contract(epoch: &str, migration_seven: &str, shared_fixture: &str) {
    assert_eq!(
        epoch
            .matches("fn migration_seven_rollback_and_ambiguous_commit_reconciliation_are_atomic()")
            .count(),
        1
    );
    for proof in [
        "verify_v7_pre_marker_rollback",
        "verify_v7_marker_rollback",
        "verify_v7_killed_commit_retry",
        "verify_v7_committed_reconciliation",
    ] {
        assert!(epoch.contains(proof));
    }
    assert!(epoch.contains("legacy_epoch_fixture::build_frozen_python_estate"));
    assert!(migration_seven.contains("spec-065 trace emission contract"));
    assert!(migration_seven.contains("canonical Rust epoch-7 reader"));
    assert!(migration_seven.contains("definition (spec-088 FR-003)"));
    assert!(!migration_seven.contains("0030_views_current.sql"));
    assert!(shared_fixture.contains("pub(crate) fn build_frozen_python_estate"));
    assert!(shared_fixture.contains("pub(crate) fn execute_h3_reader_parity_v1"));
    assert!(shared_fixture
        .contains("execute_h3_reader_parity_v1(contract, vectors, repository_root, backend)"));
    assert!(shared_fixture.contains("default_transaction_read_only=on"));
    assert!(shared_fixture.contains("export_epoch7_reader_fixture"));
    assert!(shared_fixture.contains("query_epoch7_reader_fixture"));
    for operation in [
        "sparse_fill_forward",
        "value_aggregates",
        "runtime_trace",
        "session_partition_isolation",
        "r8_parent_reference_identity",
        "nullable_locations",
        "tagged_destinations",
        "stable_pagination",
        "archive_round_trip",
    ] {
        assert!(shared_fixture.contains(operation));
    }
}

fn assert_epoch_seven_live_harness(runner: &str, live: &str, shadow: &str) {
    assert!(runner.contains(
        "for migration in migration_four migration_five migration_six migration_seven; do"
    ));
    assert!(live.contains("mod legacy_epoch_fixture;"));
    let exporter = cte_slice(
        live,
        "fn export_v7_epoch_censuses(",
        "fn verify_h3_installed_mutations(",
    );
    assert!(exporter.contains("IsolationLevel::Serializable"));
    assert!(exporter.contains("compiled[6].sql()"));
    assert!(exporter.contains("transaction.rollback()"));
    assert!(!exporter.contains("migrate_schema_epoch(&fresh_config)"));
    assert!(!exporter.contains("INSERT INTO babylon_state.schema_migration"));

    for proof in [
        "verify_epoch_seven_view_definition_mutation_refuses",
        "verify_epoch_seven_cell_id_mutation_refuses",
        "verify_epoch_seven_acl_mutation_refuses",
        "verify_epoch_seven_reader_parity_v1",
    ] {
        assert!(shadow.contains(proof));
    }
    assert!(live.contains("Some(\"h3_shadow_backfill\" | \"schema_epoch_rollback\")"));
    assert!(runner.contains("[ \"$LIVE_FOCUS\" = \"schema_epoch_rollback\" ] ||"));
}

#[test]
fn epoch_seven_fault_census_and_mutation_proofs_reuse_the_closed_live_harness() {
    let runner = include_str!("../../../../tools/run_rust_legacy_adopter_pg.sh");
    let epoch = include_str!("../src/schema_epoch.rs");
    let migration_seven = include_str!("../migrations/0007_h3_canonical_readers.sql");
    let live = include_str!("legacy_adopter_postgres.rs");
    let shadow = include_str!("support/h3_shadow_backfill_postgres.rs");
    let shared_fixture = include_str!("support/legacy_epoch_fixture.rs");

    assert_epoch_seven_source_contract(epoch, migration_seven, shared_fixture);
    assert_epoch_seven_live_harness(runner, live, shadow);
}

#[test]
fn unknown_live_focus_fails_before_any_docker_side_effect() {
    let runner = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../tools/run_rust_legacy_adopter_pg.sh");
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tool_directory = std::env::temp_dir().join(format!("babylon-per267-{unique}"));
    std::fs::create_dir(&tool_directory).unwrap();
    for tool in ["dirname", "od", "tr"] {
        std::os::unix::fs::symlink(format!("/usr/bin/{tool}"), tool_directory.join(tool)).unwrap();
    }
    let output = Command::new("/usr/bin/bash")
        .arg(runner)
        .env("BABYLON_LEGACY_ADOPTER_LIVE_FOCUS", "unknown")
        .env("PATH", &tool_directory)
        .output()
        .unwrap();
    std::fs::remove_dir_all(tool_directory).unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "run_rust_legacy_adopter_pg: unsupported live focus: unknown\n"
    );
    assert!(output.stdout.is_empty());
}

#[test]
fn h3_atomicity_receipts_are_fixed_flushed_and_test_only() {
    let installer = include_str!("../src/h3_reference_installer.rs");
    let schema_epoch = include_str!("../src/schema_epoch.rs");

    assert!(installer.contains("PER267_MEMBERSHIP_READ query="));
    assert!(installer.contains("completion=ok"));
    assert!(installer.contains("completion=error"));
    assert!(installer.contains("std::io::stderr()"));
    assert!(installer.contains(".flush()"));
    for phase in [
        "forced_rollback",
        "killed_retry",
        "committed_reconciliation",
    ] {
        assert!(
            installer.contains(phase),
            "missing fixed phase receipt: {phase}"
        );
    }
    assert!(!installer.contains("static mut"));

    let protocol = cte_slice(
        schema_epoch,
        "fn verify_h3_installer_commit_protocol(base: &Config)",
        "fn verify_post_ddl_rollback(base: &Config)",
    );
    assert_eq!(protocol.matches("Instant::now()").count(), 1);
    assert_eq!(protocol.matches("suite_started").count(), 3);
    for phase in [
        cte_slice(
            installer,
            "pub(crate) fn verify_rollback_and_killed_retry(",
            "pub(crate) fn verify_committed_reconciliation(",
        ),
        cte_slice(
            installer,
            "pub(crate) fn verify_committed_reconciliation(",
            "fn verify_cell_lock_timeout_preserves_server_diagnostic(",
        ),
    ] {
        assert!(phase.contains("suite_started: Instant"));
        assert!(!phase.contains("Instant::now()"));
    }
}

#[test]
fn ci_step_exceeds_the_focused_runner_envelope() {
    const CONTROL_PLANE_ENVELOPE_SECONDS: u64 = 5 * (10 + 2);
    const BUILD_ENVELOPE_SECONDS: u64 = 180 + 10;
    const START_ENVELOPE_SECONDS: u64 = 30 + 5;
    const READINESS_ENVELOPE_SECONDS: u64 = 90 + 2;
    const BOOTSTRAP_ENVELOPE_SECONDS: u64 = 600 + 30;
    const MICHIGAN_ENVELOPE_SECONDS: u64 = 1800 + 30;
    const FOCUSED_CARGO_ENVELOPE_SECONDS: u64 = 2 * (300 + 10) + (420 + 10);
    const CLEANUP_ENVELOPE_SECONDS: u64 = 35 + 12 + 12 + 35;
    const FOCUSED_RUNNER_ENVELOPE_SECONDS: u64 = CONTROL_PLANE_ENVELOPE_SECONDS
        + BUILD_ENVELOPE_SECONDS
        + START_ENVELOPE_SECONDS
        + READINESS_ENVELOPE_SECONDS
        + BOOTSTRAP_ENVELOPE_SECONDS
        + MICHIGAN_ENVELOPE_SECONDS
        + FOCUSED_CARGO_ENVELOPE_SECONDS
        + CLEANUP_ENVELOPE_SECONDS;

    let pr_workflow = include_str!("../../../../.github/workflows/ci.yml");
    let pr_job = yaml_job(pr_workflow, "  pg-integration-shards:");
    assert!(pr_job.contains("- name: Rust PostgreSQL contract\n        timeout-minutes: 69"));
    let pr_job_seconds = pr_job
        .lines()
        .take(MAX_WORKFLOW_JOB_BOUNDARY_CANDIDATES)
        .find_map(|line| line.trim().strip_prefix("timeout-minutes: "))
        .unwrap()
        .parse::<u64>()
        .unwrap()
        * 60;
    let pr_adopter_step = pr_job
        .split_once("      - name: Rust PostgreSQL contract")
        .unwrap()
        .1;
    let pr_step_seconds = pr_adopter_step
        .lines()
        .take(MAX_WORKFLOW_JOB_BOUNDARY_CANDIDATES)
        .find_map(|line| line.trim().strip_prefix("timeout-minutes: "))
        .unwrap()
        .parse::<u64>()
        .unwrap()
        * 60;
    assert_eq!(pr_job_seconds, 99 * 60);
    assert_eq!(pr_step_seconds, 69 * 60);
    assert!(pr_step_seconds >= FOCUSED_RUNNER_ENVELOPE_SECONDS + 120);
    assert!(pr_job_seconds >= pr_step_seconds + 30 * 60);
}

#[test]
fn weekly_step_exceeds_the_exhaustive_runner_envelope() {
    const CONTROL_PLANE_ENVELOPE_SECONDS: u64 = 5 * (10 + 2);
    const BUILD_ENVELOPE_SECONDS: u64 = 180 + 10;
    const START_ENVELOPE_SECONDS: u64 = 30 + 5;
    const READINESS_ENVELOPE_SECONDS: u64 = 90 + 2;
    const EXHAUSTIVE_CARGO_ENVELOPE_SECONDS: u64 = 900 + 10;
    const ROLLBACK_CARGO_ENVELOPE_SECONDS: u64 = 300 + 10;
    const CLEANUP_ENVELOPE_SECONDS: u64 = 35 + 12 + 12 + 35;
    const EXHAUSTIVE_RUNNER_ENVELOPE_SECONDS: u64 = CONTROL_PLANE_ENVELOPE_SECONDS
        + BUILD_ENVELOPE_SECONDS
        + START_ENVELOPE_SECONDS
        + READINESS_ENVELOPE_SECONDS
        + EXHAUSTIVE_CARGO_ENVELOPE_SECONDS
        + ROLLBACK_CARGO_ENVELOPE_SECONDS
        + CLEANUP_ENVELOPE_SECONDS;

    let weekly_workflow = include_str!("../../../../.github/workflows/weekly-pg-integration.yml");
    let weekly_job = yaml_job(weekly_workflow, "  exhaustive-legacy-adopter:");
    let weekly_step = weekly_job
        .split_once("      - name: Exhaustive adopter matrix and rollback proof")
        .unwrap()
        .1;
    let weekly_step_seconds = weekly_step
        .lines()
        .take(MAX_WORKFLOW_JOB_BOUNDARY_CANDIDATES)
        .find_map(|line| line.trim().strip_prefix("timeout-minutes: "))
        .unwrap()
        .parse::<u64>()
        .unwrap()
        * 60;
    assert_eq!(weekly_step_seconds, 31 * 60);
    assert!(weekly_step_seconds >= EXHAUSTIVE_RUNNER_ENVELOPE_SECONDS + 120);

    let runner = include_str!("../../../../tools/run_rust_legacy_adopter_pg.sh");
    assert!(runner.contains("timeout --signal=TERM --kill-after=10s 900s \\"));
    let rollback_phase = runner
        .rsplit_once("if [ \"$status\" -eq 0 ] && [ -z \"$LIVE_FOCUS\" ]; then")
        .unwrap()
        .1;
    let rollback_timeout = rollback_phase
        .find("timeout --signal=TERM --kill-after=10s 300s")
        .unwrap();
    let rollback_cargo = rollback_phase
        .find("cargo test -p babylon-persistence --lib")
        .unwrap();
    assert!(rollback_timeout < rollback_cargo);
}

#[test]
fn live_runner_bounds_every_docker_control_plane_call() {
    let runner = include_str!("../../../../tools/run_rust_legacy_adopter_pg.sh");
    assert!(runner.contains("export CARGO_BUILD_JOBS=4"));
    assert!(runner.contains("readonly CARGO_BUILD_JOBS"));
    for bounded_phase in [
        "timeout --signal=TERM --kill-after=10s 180s \\",
        "timeout --signal=TERM --kill-after=5s 30s docker run --detach \\",
        "local deadline=$((SECONDS + 90))",
        "for _attempt in {1..90}; do",
        "timeout --signal=TERM --kill-after=1s \"${remaining}s\" \\",
    ] {
        assert!(runner.contains(bounded_phase));
    }
    assert_eq!(
        runner
            .matches("timeout --signal=TERM --kill-after=5s 30s \\\n    docker rm --force --volumes \"$CONTAINER\"")
            .count(),
        2
    );
    for bounded_inspect in [
        "timeout --signal=TERM --kill-after=2s 10s \\\n      docker container inspect \"$CONTAINER\"",
        "timeout --signal=TERM --kill-after=2s 10s \\\n      docker volume inspect \"$VOLUME\"",
    ] {
        assert!(runner.contains(bounded_inspect));
    }

    assert_eq!(bounded_docker_call_count(runner), Ok(12));
}

fn bounded_docker_call_count(runner: &str) -> Result<usize, &str> {
    // Scan the complete immutable source: unrelated shell edits must neither
    // invalidate the census nor leave later control-plane calls unchecked.
    let mut docker_calls = 0_usize;
    let mut previous = "";
    for line in runner.lines() {
        if line.contains("docker ") {
            if !line.contains("timeout --signal=TERM")
                && !previous.contains("timeout --signal=TERM")
            {
                return Err(line);
            }
            docker_calls += 1;
        }
        previous = line;
    }
    Ok(docker_calls)
}

#[test]
fn live_runner_docker_census_checks_calls_after_unrelated_source_growth() {
    let runner = include_str!("../../../../tools/run_rust_legacy_adopter_pg.sh");
    let extended = format!("{runner}\n# Additional non-Docker phase.\n\n");
    assert_eq!(bounded_docker_call_count(&extended), Ok(12));
    let unbounded = format!("{extended}docker inspect future-container\n");
    assert_eq!(
        bounded_docker_call_count(&unbounded),
        Err("docker inspect future-container")
    );
}

#[test]
fn failed_pg_contract_emits_bounded_server_log_before_checked_cleanup() {
    let runner = include_str!("../../../../tools/run_rust_legacy_adopter_pg.sh");
    let logs = runner
        .find("docker logs --timestamps --tail 200 \"$CONTAINER\"")
        .expect("failure path must retain bounded server evidence");
    let cleanup = runner
        .rfind("cleanup_checked")
        .expect("cleanup must remain checked");

    assert!(runner.contains("if [ \"$status\" -ne 0 ]; then"));
    assert!(runner.contains("docker logs --timestamps --tail 200 \"$CONTAINER\" >&2 || true"));
    assert!(logs < cleanup);
}

#[test]
fn runner_never_removes_a_startup_collision_before_canary_claim() {
    let runner = include_str!("../../../../tools/run_rust_legacy_adopter_pg.sh");
    assert!(runner.contains("OWNED=0"));
    let best_effort = cte_slice(runner, "cleanup_best_effort()", "cleanup_checked()");
    let checked = cte_slice(runner, "cleanup_checked()", "wait_for_runtime()");
    for cleanup in [best_effort, checked] {
        let ownership_guard = cleanup
            .find("[ \"$OWNED\" -eq 1 ] || return 0")
            .expect("cleanup must be inert before ownership is proved");
        let removal = cleanup.find("docker rm --force --volumes").unwrap();
        assert!(ownership_guard < removal);
    }

    let claim = cte_slice(runner, "claim_task_container()", "cleanup_best_effort()");
    for proof in [
        "babylon.per20_disposable",
        "$CANARY",
        "expected_container_id",
        "OWNED=1",
    ] {
        assert!(claim.contains(proof), "missing ownership proof: {proof}");
    }
    assert!(runner.contains("--label \"babylon.per20_disposable=$CANARY\""));
    assert!(runner.contains("claim_task_container \"$created_container_id\""));

    let signal = cte_slice(runner, "on_signal()", "trap cleanup_best_effort EXIT");
    assert!(signal.contains("local -r status=\"$1\""));
    assert!(!signal.contains("\n  readonly status="));
    let recovery = signal
        .find("claim_task_container \"\" || true")
        .expect("a signal during container creation must recover only the canary-labelled runtime");
    let cleanup = signal.find("cleanup_best_effort").unwrap();
    assert!(recovery < cleanup);
}

#[test]
fn runner_proves_the_authenticated_host_mapping_before_runtime_ready() {
    let runner = include_str!("../../../../tools/run_rust_legacy_adopter_pg.sh");
    let startup = cte_slice(
        runner,
        "[ \"${#CANARY}\" -eq 32 ]",
        "printf 'PER-20 runtime ready:",
    );
    let client_check = startup.find("command -v psql").unwrap();
    let docker_build = startup.find("docker build").unwrap();
    assert!(client_check < docker_build);
    let published_port = startup.find("readonly PORT=").unwrap();
    let readiness = startup.find("wait_for_runtime ||").unwrap();
    assert!(published_port < readiness);

    let wait = cte_slice(runner, "wait_for_runtime()", "on_signal()");
    for proof in [
        "env -u PGHOSTADDR -u PGOPTIONS -u PGSERVICE -u PGSERVICEFILE",
        "PGPASSWORD=test",
        "PGCONNECT_TIMEOUT=1",
        "PGSSLMODE=disable",
        "timeout --signal=TERM --kill-after=1s 1s",
        "psql -X -w -qAt",
        "-h 127.0.0.1",
        "-p \"$PORT\"",
        "-U test",
        "-d postgres",
        "-v ON_ERROR_STOP=1",
        "SELECT 1, pg_catalog.current_setting('babylon.per20_disposable', true)",
        "1|$CANARY",
    ] {
        assert!(
            wait.contains(proof),
            "missing host readiness proof: {proof}"
        );
    }
    assert!(wait.find("docker exec").unwrap() < wait.find("psql -X").unwrap());
}

#[test]
fn destructive_live_harness_preflights_an_ephemeral_loopback_container() {
    let source = include_str!("legacy_adopter_postgres.rs");
    let entry = cte_slice(
        source,
        "fn live_adopter_contract_against_independent_builds_and_disposable_mutations()",
        "fn verify_pinned_postgres_runtime",
    );
    let preflight = entry.find("preflight_disposable_harness(&base)").unwrap();
    let first_connection = entry
        .find("BabylonIntelRolePresenceGuard::capture(&base)")
        .unwrap();
    assert!(preflight < first_connection);

    let preflight_source = cte_slice(
        source,
        "fn validate_disposable_harness_target(",
        "fn config_from_env(",
    );
    for proof in [
        "validate_legacy_connection_target(config)",
        "Host::Tcp",
        "config.get_user() != Some(\"test\")",
        "config.get_dbname() != Some(\"postgres\")",
        "server_version_num",
        "170011",
        "3.5.7",
        "0.8.5",
        "babylon.per20_disposable",
        "DISPOSABLE_CANARY_ENV",
    ] {
        assert!(
            preflight_source.contains(proof),
            "missing destructive preflight proof: {proof}"
        );
    }
    assert!(source.contains(
        "const DISPOSABLE_CANARY_ENV: &str = \"BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY\";"
    ));

    let mise = include_str!("../../../../.mise.toml");
    let task = toml_section(mise, "[tasks.\"test:rust-legacy-adopter-pg\"]");
    let runner = include_str!("../../../../tools/run_rust_legacy_adopter_pg.sh");
    assert!(task.contains("tools/run_rust_legacy_adopter_pg.sh"));
    for proof in [
        "docker run",
        "127.0.0.1::5432",
        "type=volume,target=/var/lib/postgresql/data",
        "BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY",
        "babylon.per20_disposable",
        "docker rm --force --volumes",
    ] {
        assert!(
            runner.contains(proof),
            "missing ephemeral runner proof: {proof}"
        );
    }
    assert!(!task.contains(": \"${BABYLON_LEGACY_ADOPTER_TEST_DSN:?"));
}

fn assert_live_runner_timeout_contract(runner: &str) {
    let h3_reference_focus = cte_slice(
        runner,
        "if [ \"$status\" -eq 0 ] &&\n    [ \"$LIVE_FOCUS\" = \
         \"h3_reference_installer\" ]; then",
        "if [ \"$status\" -eq 0 ] &&\n    { [ \"$LIVE_FOCUS\" = \
         \"schema_epoch_fresh\" ] ||",
    );
    assert!(h3_reference_focus.contains("timeout --signal=TERM --kill-after=10s 900s"));
    assert!(!h3_reference_focus.contains("timeout --signal=TERM --kill-after=10s 300s"));
    let other_single_focus = cte_slice(
        runner,
        "if [ \"$status\" -eq 0 ] &&\n    { [ \"$LIVE_FOCUS\" = \
         \"schema_epoch_fresh\" ] ||",
        "if [ \"$status\" -eq 0 ] && [ -z \"$LIVE_FOCUS\" ]; then",
    );
    assert!(other_single_focus.contains("timeout --signal=TERM --kill-after=10s 300s"));
    assert!(!other_single_focus.contains("timeout --signal=TERM --kill-after=10s 900s"));
    assert!(!other_single_focus.contains("h3_reference_installer"));
    let exhaustive = runner
        .split_once("if [ \"$status\" -eq 0 ] && [ -z \"$LIVE_FOCUS\" ]; then")
        .unwrap()
        .1
        .split_once("if [ \"$status\" -eq 0 ] && [ -z \"$LIVE_FOCUS\" ]; then")
        .unwrap()
        .0;
    assert!(exhaustive.contains("timeout --signal=TERM --kill-after=10s 900s"));
    assert!(!exhaustive.contains("timeout --signal=TERM --kill-after=10s 300s"));
    assert!(runner.contains(
        "schema_epoch::live_rollback_tests::rollback_and_ambiguous_commit_reconciliation_are_atomic"
    ));
    let cargo = runner
        .split_once("cargo test -p babylon-persistence --test legacy_adopter_postgres")
        .unwrap()
        .1
        .split_once("|| status=$?")
        .unwrap()
        .0;
    assert!(cargo.find("--nocapture").unwrap() < cargo.find("--ignored").unwrap());
    assert!(cargo.contains("--test-threads=1"));
}

#[test]
fn live_matrix_receipts_survive_one_fixed_diagnostic_ceiling() {
    let source = include_str!("legacy_adopter_postgres.rs");
    let entry = cte_slice(
        source,
        "fn live_adopter_contract_against_independent_builds_and_disposable_mutations()",
        "fn verify_pinned_postgres_runtime",
    );
    let phases = [
        "config_from_env",
        "preflight",
        "capture_babylon_intel",
        "initial_residue",
        "create_owner_role",
        "create_first_database",
        "create_second_database",
        "runtime_census_v2",
        "repair_first",
        "repair_second",
        "canonical_fixture_bytes",
        "pinned_runtime",
        "raw_non_catalog_bounds",
        "independent_builds",
        "hostile_caller",
        "advisory_lock",
        "blocking_table_lock",
        "no_mutation_quoted_extras",
        "stamp_refusals",
        "authority_epoch",
        "structural_role",
        "effective_authority",
        "write_semantics",
        "inheritance_subpartition",
        "partition_children",
        "extra_surfaces",
        "strict_census",
        "cast_system_namespace",
        "reserved_roles",
        "extension_identity",
        "column_bound",
        "unsupported_catalog_bounds",
        "sequence_owned_by",
        "sequence_acl_default",
        "canonical_after_cleanup",
        "schema_epoch_matrix",
        "h3_pg_oracle",
        "h3_reference_installer",
        "cleanup_first_database",
        "cleanup_second_database",
        "cleanup_owner_role",
        "cleanup_babylon_intel",
        "final_residue",
    ];
    for phase in phases {
        assert!(entry.contains(phase), "missing live phase receipt: {phase}");
    }
    assert_eq!(entry.matches("live_phase!(").count(), phases.len());
    assert!(source.contains("self.emit(name, \"started\", Duration::ZERO)"));
    assert!(source.contains("self.emit(name, \"complete\", phase_start.elapsed())"));
    assert!(source.contains("completion={completion} phase_ms="));
    assert!(source.contains("cumulative_ms="));
    assert_live_runner_timeout_contract(include_str!(
        "../../../../tools/run_rust_legacy_adopter_pg.sh"
    ));
}

#[test]
fn canonical_fixture_export_uses_the_adopters_hardened_session() {
    let live = include_str!("legacy_adopter_postgres.rs");
    let exporter = cte_slice(
        live,
        "fn verify_canonical_fixture_bytes(",
        "fn canonical_fixture_bytes(",
    );
    assert_eq!(
        exporter
            .matches("options(LEGACY_ADOPTER_STARTUP_OPTIONS)")
            .count(),
        2
    );
}

#[test]
fn live_harness_cleanup_is_checked_and_residue_proved() {
    let source = include_str!("legacy_adopter_postgres.rs");
    let entry = cte_slice(
        source,
        "fn live_adopter_contract_against_independent_builds_and_disposable_mutations()",
        "fn verify_independent_builds",
    );
    for proof in [
        "BabylonIntelRolePresenceGuard::capture(&base)",
        "verify_canonical_state_after_global_cleanup(first_config)",
        "second.cleanup()",
        "first.cleanup()",
        "owner.cleanup()",
        "babylon_intel.cleanup()",
        "assert_no_scratch_residue(&base)",
    ] {
        assert!(entry.contains(proof), "missing cleanup proof: {proof}");
    }
    let presence = entry
        .find("BabylonIntelRolePresenceGuard::capture(&base)")
        .unwrap();
    let scratch = entry.find("ScratchRole::create(&base)").unwrap();
    let verification = entry
        .find("run_second_live_phases(&phases, &base, &template, &first_config);")
        .unwrap();
    let first = entry.find("first.cleanup()").unwrap();
    let second = entry.find("second.cleanup()").unwrap();
    let owner = entry.find("owner.cleanup()").unwrap();
    let intel = entry.find("babylon_intel.cleanup()").unwrap();
    let residue = entry.rfind("assert_no_scratch_residue(&base)").unwrap();
    assert!(
        presence < scratch
            && verification < first
            && first < second
            && second < owner
            && owner < intel
            && intel < residue
    );

    let presence_guard = cte_slice(
        source,
        "struct BabylonIntelRolePresenceGuard",
        "impl GlobalMutationGuard",
    );
    assert!(presence_guard.contains("admin: Config"));
    assert!(presence_guard.contains("initially_present: bool"));
    assert!(presence_guard.contains("fn cleanup(mut self)"));
    assert!(presence_guard.contains("std::thread::panicking()"));
    assert!(presence_guard.contains("DROP ROLE babylon_intel"));
    let global_guard = cte_slice(source, "impl GlobalMutationGuard", "impl ScratchDatabase");
    assert!(global_guard.contains("fn cleanup(mut self)"));
    assert!(global_guard.contains("std::thread::panicking()"));
    assert!(global_guard.contains("let connection = base.clone()"));
    assert!(!global_guard.contains("admin_config(base)"));
    assert!(!global_guard.contains("let _cleanup_result"));
    assert_eq!(source.matches("fn cleanup(mut self)").count(), 4);
    assert_eq!(source.matches("std::thread::panicking()").count(), 4);
    assert!(!source.contains("let _result = client.batch_execute"));
    assert!(source.contains(
        "\"I_UNDERSTAND_PER20_DROPS_SCRATCH_DATABASES_ROLES_AND_CREATED_BABYLON_INTEL\";"
    ));
    for cleanup in [
        "guard.cleanup()",
        "fn assert_no_scratch_residue",
        "BabylonIntelRolePresenceGuard::capture",
        "DISPOSABLE_ACK_ENV",
        "DISPOSABLE_ACK_VALUE",
    ] {
        assert!(
            source.contains(cleanup),
            "missing checked cleanup seam: {cleanup}"
        );
    }
}

#[test]
fn untrusted_routine_owner_cleanup_drops_database_before_role() {
    let source = include_str!("legacy_adopter_postgres.rs");
    let refusal = cte_slice(
        source,
        "fn verify_unsafe_routine_refusals(",
        "fn verify_extra_mutation_grant_refusals(",
    );
    let database_cleanup = refusal
        .find("untrusted.cleanup();")
        .expect("owned scratch database needs explicit checked cleanup");
    let role_cleanup = refusal.find("untrusted_owner.cleanup();").unwrap();
    assert!(database_cleanup < role_cleanup);

    let role_guard = cte_slice(source, "impl ScratchRole", "impl Drop for ScratchRole");
    assert!(role_guard.contains("scratch role cleanup must succeed for"));
    assert!(role_guard.contains(": {error}"));
}

fn assert_manifest(
    chunks: &[&[u8]],
    start: usize,
    end: usize,
    name: &'static str,
    expected_chunks: usize,
    expected_bytes: usize,
    expected_digest: &str,
) {
    let selected = &chunks[start..end];
    let manifest = MigrationManifest::from_chunks(name, selected).unwrap();
    let framed_bytes = selected.iter().map(|chunk| chunk.len() + 1).sum::<usize>();
    assert_eq!(manifest.chunk_count(), expected_chunks);
    if expected_bytes != 0 {
        assert_eq!(framed_bytes, expected_bytes);
    }
    assert_eq!(manifest.digest().to_hex(), expected_digest);
}

fn split_nul_framed(bytes: &[u8], expected_chunks: usize) -> Vec<&[u8]> {
    assert_eq!(bytes.last(), Some(&0));
    let chunks = bytes[..bytes.len() - 1]
        .split(|byte| *byte == 0)
        .take(expected_chunks + 1)
        .collect::<Vec<_>>();
    assert_eq!(chunks.len(), expected_chunks);
    chunks
}

fn toml_section<'a>(text: &'a str, header: &str) -> &'a str {
    let start = text.find(header).unwrap();
    let tail = &text[start + header.len()..];
    let end = tail.find("\n[tasks.").unwrap_or(tail.len());
    &tail[..end]
}

fn yaml_job<'a>(text: &'a str, header: &str) -> &'a str {
    let start = text.find(header).unwrap();
    let tail = &text[start + header.len()..];
    let end = tail
        .match_indices("\n  ")
        .enumerate()
        .take(MAX_WORKFLOW_JOB_BOUNDARY_CANDIDATES + 1)
        .find_map(|(candidate_index, (byte_index, _))| {
            assert!(candidate_index < MAX_WORKFLOW_JOB_BOUNDARY_CANDIDATES);
            let line = tail[byte_index + 1..].lines().next().unwrap();
            line.strip_prefix("  ")?.strip_suffix(':').filter(|name| {
                !name.is_empty()
                    && name.len() <= 128
                    && name
                        .bytes()
                        .take(129)
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            })?;
            Some(byte_index)
        })
        .unwrap_or(tail.len());
    &tail[..end]
}

fn cte_slice<'a>(sql: &'a str, start_marker: &str, end_marker: &str) -> &'a str {
    let start = sql.find(start_marker).unwrap();
    let tail = &sql[start..];
    let end = tail.find(end_marker).unwrap();
    &tail[..end]
}

fn sql_without_string_literals(sql: &str) -> String {
    assert!(sql.len() <= MAX_SQL_STATEMENT_BYTES);
    let mut code = String::with_capacity(sql.len());
    for (index, segment) in sql
        .split('\'')
        .enumerate()
        .take(MAX_SQL_LITERAL_SEGMENTS + 1)
    {
        assert!(index < MAX_SQL_LITERAL_SEGMENTS);
        if index % 2 == 0 {
            code.push_str(segment);
        }
    }
    code
}

fn assert_stamp(name: &str, chunks: usize, digest: &str, class: LegacyStampClass) {
    let definition = LEGACY_STAMP_CATALOG
        .iter()
        .find(|item| item.name == name)
        .expect("named stamp definition");
    assert_eq!(definition.chunk_count, chunks);
    assert_eq!(definition.digest_hex, digest);
    assert_eq!(definition.class, class);
}

fn required_stamp_digests() -> Vec<String> {
    LEGACY_STAMP_CATALOG
        .iter()
        .filter(|item| item.class == LegacyStampClass::RequiredCurrent)
        .take(2)
        .map(|item| item.digest_hex.to_owned())
        .collect()
}

fn entry(kind: LegacyObjectKind, schema: &str, name: &str, digit: char) -> LegacyCensusEntry {
    LegacyCensusEntry::new(
        LegacyObjectKey::new(kind, schema, name).unwrap(),
        digit.to_string().repeat(64).as_str(),
    )
    .unwrap()
}

fn count_kind(census: &babylon_persistence::LegacyCensus, kind: LegacyObjectKind) -> usize {
    census
        .entries()
        .iter()
        .take(MAX_LEGACY_CENSUS_ROWS)
        .filter(|entry| entry.key().kind() == kind)
        .count()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest.iter().take(32) {
        write!(&mut hex, "{byte:02x}").unwrap();
    }
    hex
}

fn canonical_json_receipt(receipt: &'static [u8]) -> &'static [u8] {
    let payload = receipt
        .strip_suffix(b"\n")
        .expect("checked-in JSON receipt must have one trailing newline");
    assert!(!payload.ends_with(b"\n"));
    payload
}

fn census_fixture_digest<'a>(fixture: &'a str, row_prefix: &str) -> &'a str {
    fixture
        .lines()
        .take(MAX_LEGACY_CENSUS_ROWS + 32)
        .find_map(|line| line.strip_prefix(row_prefix))
        .expect("named census row must exist")
}

fn census_sql() -> &'static str {
    legacy_adopter_sql_statements()
        .iter()
        .take(8)
        .map(babylon_persistence::LegacyAdopterSqlStatement::sql)
        .find(|sql| sql.contains("relation_shapes"))
        .expect("the central SQL list must contain the census statement")
}
