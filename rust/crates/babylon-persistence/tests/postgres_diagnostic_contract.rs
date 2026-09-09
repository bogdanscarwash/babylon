//! Secret-safe `PostgreSQL` diagnostic contracts for PER-288.

use std::time::Duration;

use babylon_persistence::{
    PostgresDiagnosticV1, PostgresFailureClassV1, MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES,
};
use postgres::{Config, NoTls};

const RUNTIME_SOURCE: &str = include_str!("../src/bin/babylon-runtime.rs");

#[test]
fn unreachable_target_is_classified_without_connection_material() {
    let password_canary = "PER288_PASSWORD_CANARY";
    let user_canary = "per288_user_canary";
    let mut config = Config::new();
    config
        .host("127.0.0.1")
        .port(1)
        .user(user_canary)
        .password(password_canary)
        .connect_timeout(Duration::from_millis(100));

    let Err(error) = config.connect(NoTls) else {
        panic!("the reserved local endpoint must refuse the test connection");
    };
    let diagnostic = PostgresDiagnosticV1::capture(&error);
    let rendered = format!("{diagnostic:?}");

    assert_eq!(
        diagnostic.classification(),
        PostgresFailureClassV1::Reachability
    );
    assert_eq!(diagnostic.sqlstate(), None);
    assert!(diagnostic
        .message()
        .is_some_and(|message| message.len() <= MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES));
    assert!(!rendered.contains(password_canary));
    assert!(!rendered.contains(user_canary));
    assert!(!rendered.contains("127.0.0.1"));
}

#[test]
fn maintenance_probe_surfaces_the_bounded_diagnostic_instead_of_erasing_it() {
    assert!(RUNTIME_SOURCE.contains("PostgresDiagnosticV1::capture(error)"));
    assert!(RUNTIME_SOURCE.contains("database probe connection"));
    assert!(RUNTIME_SOURCE.contains("Archive probe connection"));
    assert!(!RUNTIME_SOURCE.contains("map_err(|_| \"database probe connection failed\""));
    assert!(!RUNTIME_SOURCE.contains("map_err(|_| \"Archive probe connection failed\""));
}
