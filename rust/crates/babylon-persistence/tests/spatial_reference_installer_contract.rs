//! Static authority contract for the PER-278 maintenance-only installer.

const SOURCE: &str = include_str!("../src/spatial_reference_installer.rs");

#[test]
fn installer_is_current_schema_locked_transactional_and_reconciled() {
    for required in [
        "pub fn install_michigan_spatial_reference_products(",
        "validate_connection_target(config)",
        "acquire_lock(&mut client)",
        "require_current_schema(client)",
        ".isolation_level(IsolationLevel::Serializable)",
        "SET LOCAL search_path TO pg_catalog",
        "SET LOCAL synchronous_commit TO on",
        "ON CONFLICT DO NOTHING",
        "transaction.commit()",
        "CommitAttempt::Ambiguous",
        "session.reconnect(config)",
        "inspect_presence(transaction, bundle)",
        "release_lock(client)",
    ] {
        assert!(SOURCE.contains(required), "installer omits {required:?}");
    }

    for relation in [
        "babylon_ref.reference_product",
        "babylon_ref.county_identity",
        "babylon_ref.place_identity",
        "babylon_ref.h3_land_fraction",
        "babylon_ref.h3_population_count",
        "babylon_ref.h3_workplace_count",
        "babylon_ref.county_h3_land_area",
        "babylon_ref.county_place_h3_land_area",
    ] {
        assert!(
            SOURCE.matches(relation).count() >= 2,
            "installer must write and read back {relation}"
        );
    }

    assert_eq!(
        SOURCE.matches("membership_origin").count(),
        5,
        "every H3-bearing insert must write the governed direct-origin discriminator"
    );
}

#[test]
fn installer_does_not_open_runtime_or_schema_authority() {
    for prohibited in [
        "RustWriterAuthority",
        "request_rust_writer_authority",
        "install_current_schema(",
        "CREATE TABLE",
        "ALTER TABLE",
        "DROP TABLE",
        "TRUNCATE",
        "DELETE FROM",
        "UPDATE babylon_",
        "public.",
        "std::env",
        "option_env!",
    ] {
        assert!(
            !SOURCE.contains(prohibited),
            "installer exposes prohibited surface {prohibited:?}"
        );
    }
}
