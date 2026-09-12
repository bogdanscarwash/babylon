//! Current catalog parser, exact-census and local-target contracts.
use babylon_persistence::{
    postgres_catalog::compare_catalog_census, postgres_catalog::parse_catalog_census,
    postgres_catalog::validate_connection_target, postgres_catalog::CatalogCensus,
    postgres_catalog::CatalogCensusEntry, postgres_catalog::CatalogCensusParseError,
    postgres_catalog::CatalogError, postgres_catalog::CatalogObjectKey,
    postgres_catalog::CatalogObjectKind, postgres_catalog::ConnectionTargetRejection,
    postgres_catalog::MAX_CATALOG_CENSUS_FIXTURE_BYTES, postgres_catalog::MAX_CATALOG_CENSUS_ROWS,
    postgres_catalog::POSTGRES_IDENTIFIER_MAX_BYTES,
};
use postgres::Config;
use std::fmt::Write as _;
use std::net::{IpAddr, Ipv4Addr};
const ZERO_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn expected_census() -> CatalogCensus {
    parse_catalog_census(include_str!("../src/fixtures/fresh_schema_census.txt")).unwrap()
}

#[test]
fn census_fixture_parser_rejects_duplicate_unsorted_and_malformed_records() {
    let duplicate =
        format!("relation|public|alpha|{ZERO_DIGEST}\nrelation|public|alpha|{ZERO_DIGEST}\n");
    assert!(matches!(
        parse_catalog_census(&duplicate),
        Err(CatalogCensusParseError::DuplicateObject { .. })
    ));
    let unsorted =
        format!("relation|public|zeta|{ZERO_DIGEST}\nrelation|public|alpha|{ZERO_DIGEST}\n");
    assert!(matches!(
        parse_catalog_census(&unsorted),
        Err(CatalogCensusParseError::OutOfOrder { .. })
    ));
    assert_eq!(
        parse_catalog_census("relation|public|only_three_fields\n"),
        Err(CatalogCensusParseError::MalformedRecord { line: 1, fields: 3 })
    );
    assert_eq!(
        parse_catalog_census(&format!("relation|public|name|{ZERO_DIGEST}|extra\n")),
        Err(CatalogCensusParseError::MalformedRecord { line: 1, fields: 5 })
    );
    assert_eq!(
        parse_catalog_census(&format!(
            "relation|public|name|{ZERO_DIGEST}|extra|second_extra\n"
        )),
        Err(CatalogCensusParseError::MalformedRecord { line: 1, fields: 6 })
    );
}

#[test]
fn census_fixture_parser_rejects_invalid_kind_identifier_digest_and_bounds() {
    let maximum = "a".repeat(POSTGRES_IDENTIFIER_MAX_BYTES);
    let too_long = "a".repeat(POSTGRES_IDENTIFIER_MAX_BYTES + 1);
    assert!(parse_catalog_census(&format!("relation|public|{maximum}|{ZERO_DIGEST}\n")).is_ok());
    let cases = [
        (
            format!("bogus|public|name|{ZERO_DIGEST}\n"),
            CatalogCensusParseError::InvalidKind { line: 1 },
        ),
        (
            format!("relation|Public|name|{ZERO_DIGEST}\n"),
            CatalogCensusParseError::InvalidIdentifier { line: 1 },
        ),
        (
            format!("relation|public|{too_long}|{ZERO_DIGEST}\n"),
            CatalogCensusParseError::InvalidIdentifier { line: 1 },
        ),
        (
            "relation|public|name|abcdef\n".to_owned(),
            CatalogCensusParseError::InvalidDigest { line: 1 },
        ),
    ];
    for (fixture, expected) in cases {
        assert_eq!(parse_catalog_census(&fixture), Err(expected));
    }

    let fixture = "#".repeat(MAX_CATALOG_CENSUS_FIXTURE_BYTES + 1);
    assert_eq!(
        parse_catalog_census(&fixture),
        Err(CatalogCensusParseError::TooManyBytes {
            actual: MAX_CATALOG_CENSUS_FIXTURE_BYTES + 1,
            max: MAX_CATALOG_CENSUS_FIXTURE_BYTES,
        })
    );
}

#[test]
fn census_parser_and_comparison_enforce_row_bounds_without_unbounded_collect() {
    let mut oversized = String::new();
    for index in 0..=MAX_CATALOG_CENSUS_ROWS {
        writeln!(oversized, "relation|public|name_{index:04}|{ZERO_DIGEST}").unwrap();
    }
    assert!(matches!(
        parse_catalog_census(&oversized),
        Err(CatalogCensusParseError::TooManyRows { .. })
    ));

    let expected = expected_census();
    let oversized_actual = vec![expected.entries()[0].clone(); MAX_CATALOG_CENSUS_ROWS + 1];
    assert!(matches!(
        compare_catalog_census(&expected, &oversized_actual),
        Err(CatalogError::Bounds { .. })
    ));
}

#[test]
fn pure_census_comparison_refuses_all_sorted_extra_keys_and_rejects_duplicates() {
    let expected = expected_census();
    let mut actual = expected.entries().to_vec();
    let beta = entry(CatalogObjectKind::Domain, "public", "extra_beta", '1');
    let alpha = entry(CatalogObjectKind::Relation, "public", "extra_alpha", '2');
    actual.push(beta.clone());
    actual.push(alpha.clone());
    assert_eq!(
        compare_catalog_census(&expected, &actual),
        Err(CatalogError::UnsupportedCatalogExtras {
            objects: vec![beta.key().clone(), alpha.key().clone()],
        })
    );

    let mut duplicate = expected.entries().to_vec();
    duplicate.push(expected.entries()[0].clone());
    assert!(matches!(
        compare_catalog_census(&expected, &duplicate),
        Err(CatalogError::DuplicateCensusObject { .. })
    ));
}

#[test]
fn review_census_comparison_refuses_every_extra_object_kind() {
    let expected = expected_census();
    for kind in [
        CatalogObjectKind::Database,
        CatalogObjectKind::Domain,
        CatalogObjectKind::Extension,
        CatalogObjectKind::ForeignTable,
        CatalogObjectKind::MaterializedView,
        CatalogObjectKind::PartitionedTable,
        CatalogObjectKind::Relation,
        CatalogObjectKind::Role,
        CatalogObjectKind::Routine,
        CatalogObjectKind::Schema,
        CatalogObjectKind::SchemaGrant,
        CatalogObjectKind::Sequence,
        CatalogObjectKind::UnsupportedCatalog,
        CatalogObjectKind::UserType,
        CatalogObjectKind::View,
    ] {
        let mut actual = expected.entries().to_vec();
        let extra = entry(kind, "public", "unsafe_extra", '3');
        actual.push(extra.clone());
        assert_eq!(
            compare_catalog_census(&expected, &actual),
            Err(CatalogError::UnsupportedCatalogExtras {
                objects: vec![extra.key().clone()],
            })
        );
    }
}

#[test]
fn pure_census_comparison_reports_missing_and_changed_expected_objects() {
    let expected = expected_census();
    let mut missing_actual = expected.entries().to_vec();
    let missing_key = missing_actual.remove(0).key().clone();
    assert!(matches!(
        compare_catalog_census(&expected, &missing_actual),
        Err(CatalogError::CensusMismatch {
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
        compare_catalog_census(&expected, &changed_actual),
        Err(CatalogError::CensusMismatch {
            missing_objects,
            changed_objects,
            ..
        }) if missing_objects.is_empty() && changed_objects == vec![changed_key]
    ));
}

#[test]
fn connection_target_requires_one_literal_local_endpoint() {
    let mut loopback = Config::new();
    loopback.host("127.0.0.1").port(5432);
    assert_eq!(validate_connection_target(&loopback), Ok(()));

    for (host, reason) in [
        ("localhost", ConnectionTargetRejection::NonLoopbackTcp),
        ("192.0.2.1", ConnectionTargetRejection::NonLoopbackTcp),
    ] {
        let mut config = Config::new();
        config.host(host);
        assert_eq!(
            validate_connection_target(&config),
            Err(CatalogError::UnsupportedConnectionTarget { reason })
        );
    }

    let missing = Config::new();
    assert_eq!(
        validate_connection_target(&missing),
        Err(CatalogError::UnsupportedConnectionTarget {
            reason: ConnectionTargetRejection::MissingHost,
        })
    );
    let mut multiple = Config::new();
    multiple.host("127.0.0.1").host("::1");
    assert_eq!(
        validate_connection_target(&multiple),
        Err(CatalogError::UnsupportedConnectionTarget {
            reason: ConnectionTargetRejection::MultipleHosts,
        })
    );
    let mut ports = Config::new();
    ports.host("127.0.0.1").port(5432).port(5433);
    assert_eq!(
        validate_connection_target(&ports),
        Err(CatalogError::UnsupportedConnectionTarget {
            reason: ConnectionTargetRejection::MultiplePorts,
        })
    );
    let mut redirected = Config::new();
    redirected
        .host("127.0.0.1")
        .hostaddr(IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_eq!(
        validate_connection_target(&redirected),
        Err(CatalogError::UnsupportedConnectionTarget {
            reason: ConnectionTargetRejection::HostAddressOverride,
        })
    );
    let mut caller_options = Config::new();
    caller_options
        .host("127.0.0.1")
        .port(1)
        .options("-c search_path=redirected,public");
    assert_eq!(
        validate_connection_target(&caller_options),
        Err(CatalogError::UnsupportedConnectionTarget {
            reason: ConnectionTargetRejection::StartupOptionsOverride,
        })
    );
    #[cfg(unix)]
    {
        let mut absolute_socket = Config::new();
        absolute_socket.host_path("/var/run/postgresql");
        assert_eq!(validate_connection_target(&absolute_socket), Ok(()));
        let mut relative_socket = Config::new();
        relative_socket.host_path("relative/socket");
        assert_eq!(
            validate_connection_target(&relative_socket),
            Err(CatalogError::UnsupportedConnectionTarget {
                reason: ConnectionTargetRejection::NonAbsoluteUnixSocket,
            })
        );
    }
}

fn entry(kind: CatalogObjectKind, schema: &str, name: &str, digit: char) -> CatalogCensusEntry {
    CatalogCensusEntry::new(
        CatalogObjectKey::new(kind, schema, name).unwrap(),
        digit.to_string().repeat(64).as_str(),
    )
    .unwrap()
}
