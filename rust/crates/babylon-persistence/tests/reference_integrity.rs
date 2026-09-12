//! Current H3 and reference-bundle integrity against an explicitly owned disposable runtime.

use babylon_persistence::{
    postgres_catalog::validate_connection_target, postgres_catalog::CATALOG_CONNECT_TIMEOUT,
    postgres_catalog::CATALOG_STARTUP_OPTIONS, postgres_catalog::CATALOG_TCP_USER_TIMEOUT,
    SCHEMA_ADVISORY_LOCK_KEY,
};
use postgres::config::Host;
use postgres::{Config, NoTls};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "../../babylon-kernel/tests/support/h3_cell_vectors.rs"]
mod h3_cell_vectors;
#[path = "support/h3_pg_oracle.rs"]
mod h3_pg_oracle;
#[path = "support/h3_reference_installer_postgres.rs"]
mod h3_reference_installer_postgres;

const DSN_ENV: &str = "BABYLON_POSTGRES_TEST_DSN";
const DISPOSABLE_ACK_ENV: &str = "BABYLON_POSTGRES_DISPOSABLE_ACK";
const DISPOSABLE_ACK_VALUE: &str =
    "I_UNDERSTAND_THIS_DISPOSABLE_RUNTIME_DROPS_ITS_SCRATCH_DATABASES_AND_ROLES";
const DISPOSABLE_CANARY_ENV: &str = "BABYLON_POSTGRES_DISPOSABLE_CANARY";
const VALID_DISPOSABLE_CANARY: &str = "0123456789abcdef0123456789abcdef";

#[test]
#[ignore = "requires an owned disposable runtime in BABYLON_POSTGRES_TEST_DSN"]
fn live_scratch_login_requires_scram() {
    let base = config_from_env();
    preflight_disposable_harness(&base);
    let admin = admin_config(&base);
    let mut client = admin.connect(NoTls).unwrap();
    client
        .batch_execute("SET password_encryption = 'md5'")
        .unwrap();
    let name = scratch_name("scram");
    let password = create_scratch_login(&mut client, &name);
    let role = ScratchRole {
        name,
        password,
        admin,
        active: true,
    };
    let encoded: bool = client.query_one(
        "SELECT rolpassword LIKE 'SCRAM-SHA-256$%' FROM pg_catalog.pg_authid WHERE rolname = $1",
        &[&role.name],
    ).unwrap().get(0);
    let mut login = base.clone();
    login.user(role.name()).password(role.password());
    let accepts_generated = login.connect(NoTls).is_ok();
    login.password(format!("{}x", role.password()));
    let rejects_wrong = login.connect(NoTls).is_err();
    role.cleanup();
    assert!(
        encoded,
        "scratch login must store a SCRAM verifier even with an MD5 session default"
    );
    assert!(
        accepts_generated,
        "generated scratch credential must authenticate"
    );
    assert!(
        rejects_wrong,
        "scratch login must reject an incorrect credential"
    );
}

#[test]
#[ignore = "requires an owned disposable runtime in BABYLON_POSTGRES_TEST_DSN"]
fn live_reference_bundle_integrity() {
    let base = config_from_env();
    preflight_disposable_harness(&base);
    let owner = ScratchRole::create(&base);
    h3_reference_installer_postgres::verify_h3_reference_installer(
        &base,
        owner.name(),
        owner.password(),
    );
    owner.cleanup();
}

#[test]
#[ignore = "requires an owned disposable runtime in BABYLON_POSTGRES_TEST_DSN"]
fn live_h3_pg_semantic_oracle() {
    let base = config_from_env();
    preflight_disposable_harness(&base);
    let database = ScratchDatabase::empty(&base, "h3_oracle", database_user(&base));
    let config = database.config(&base);
    h3_pg_oracle::verify_h3_pg_oracle(&config, &config);
    database.cleanup();
}

#[test]
fn disposable_harness_accepts_only_the_owned_loopback_shape() {
    let config =
        Config::from_str("host=127.0.0.1 port=55433 user=test password=test dbname=postgres")
            .unwrap();
    assert_eq!(
        validate_disposable_harness_target(&config, Some(VALID_DISPOSABLE_CANARY)),
        Ok(())
    );
}

#[test]
fn disposable_harness_rejects_nonowned_targets_before_connect() {
    let cases = [
        "host=localhost port=55433 user=test dbname=postgres",
        "host=192.0.2.1 port=55433 user=test dbname=postgres",
        "host=/tmp port=55433 user=test dbname=postgres",
        "host=127.0.0.1,127.0.0.1 port=55433 user=test dbname=postgres",
        "host=127.0.0.1 port=55433,55434 user=test dbname=postgres",
        "host=127.0.0.1 hostaddr=127.0.0.1 port=55433 user=test dbname=postgres",
        "host=127.0.0.1 port=55433 user=other dbname=postgres",
        "host=127.0.0.1 port=55433 user=test dbname=babylon_test",
        "host=127.0.0.1 user=test dbname=postgres",
    ];
    for dsn in cases.iter().take(9) {
        let config = Config::from_str(dsn).unwrap();
        assert!(
            validate_disposable_harness_target(&config, Some(VALID_DISPOSABLE_CANARY)).is_err()
        );
    }
}

#[test]
fn disposable_harness_rejects_missing_or_invalid_canary() {
    let config =
        Config::from_str("host=127.0.0.1 port=55433 user=test password=test dbname=postgres")
            .unwrap();
    for canary in [
        None,
        Some(""),
        Some("0123456789abcdef0123456789abcde"),
        Some("0123456789abcdef0123456789abcdef0"),
        Some("0123456789abcdef0123456789abcdeG"),
    ]
    .iter()
    .take(5)
    {
        assert_eq!(
            validate_disposable_harness_target(&config, *canary),
            Err(DisposableHarnessRejection::Canary)
        );
    }
}

fn assert_lock_released(config: &Config) {
    let mut client = config.connect(NoTls).unwrap();
    let locked: bool = client
        .query_one(
            "SELECT pg_catalog.pg_try_advisory_lock($1)",
            &[&SCHEMA_ADVISORY_LOCK_KEY],
        )
        .unwrap()
        .try_get(0)
        .unwrap();
    assert!(locked);
    let unlocked: bool = client
        .query_one(
            "SELECT pg_catalog.pg_advisory_unlock($1)",
            &[&SCHEMA_ADVISORY_LOCK_KEY],
        )
        .unwrap()
        .try_get(0)
        .unwrap();
    assert!(unlocked);
}

struct ScratchDatabase {
    name: String,
    admin: Config,
    active: bool,
}

impl ScratchDatabase {
    fn empty(base: &Config, label: &str, owner: &str) -> Self {
        let name = scratch_name(label);
        let admin = admin_config(base);
        let mut client = admin.connect(NoTls).unwrap();
        client
            .batch_execute(
                format!(
                    "CREATE DATABASE {} OWNER {} TEMPLATE template1",
                    quote_identifier(&name),
                    quote_identifier(owner)
                )
                .as_str(),
            )
            .unwrap();
        Self {
            name,
            admin,
            active: true,
        }
    }

    fn from_template(base: &Config, template: &str, label: &str) -> Self {
        let name = scratch_name(label);
        let admin = admin_config(base);
        let mut client = admin.connect(NoTls).unwrap();
        client
            .batch_execute(
                format!(
                    "CREATE DATABASE {} WITH TEMPLATE {}",
                    quote_identifier(&name),
                    quote_identifier(template)
                )
                .as_str(),
            )
            .unwrap();
        Self {
            name,
            admin,
            active: true,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn config(&self, base: &Config) -> Config {
        let mut config = base.clone();
        config.dbname(&self.name);
        config
    }

    fn config_as(&self, base: &Config, user: &str, password: &str) -> Config {
        let mut config = self.config(base);
        config.user(user).password(password);
        config
    }

    fn cleanup(mut self) {
        match self.try_cleanup() {
            Ok(()) => self.active = false,
            Err(()) => panic!("scratch database cleanup must succeed"),
        }
    }

    fn try_cleanup(&self) -> Result<(), ()> {
        let mut client = self.admin.connect(NoTls).map_err(|_| ())?;
        let sql = format!(
            "DROP DATABASE IF EXISTS {} WITH (FORCE)",
            quote_identifier(&self.name)
        );
        client.batch_execute(&sql).map_err(|_| ())
    }
}

impl Drop for ScratchDatabase {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        if std::thread::panicking() {
            let _unwind_cleanup = self.try_cleanup();
            return;
        }
        match self.try_cleanup() {
            Ok(()) => self.active = false,
            Err(()) => panic!("scratch database cleanup failed"),
        }
    }
}

struct ScratchRole {
    name: String,
    password: String,
    admin: Config,
    active: bool,
}

impl ScratchRole {
    fn create(base: &Config) -> Self {
        let role = Self::create_without_event_trigger_set(base);
        let mut client = role.admin.connect(NoTls).unwrap();
        client
            .batch_execute(
                format!(
                    "GRANT SET ON PARAMETER event_triggers TO {}",
                    quote_identifier(&role.name)
                )
                .as_str(),
            )
            .unwrap();
        drop(client);
        role
    }

    fn create_without_event_trigger_set(base: &Config) -> Self {
        let name = scratch_name("owner");
        let admin = admin_config(base);
        let mut client = admin.connect(NoTls).unwrap();
        let password = create_scratch_login(&mut client, &name);
        Self {
            name,
            password,
            admin,
            active: true,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn password(&self) -> &str {
        &self.password
    }

    fn cleanup(mut self) {
        match self.try_cleanup() {
            Ok(()) => self.active = false,
            Err(error) => panic!(
                "scratch role cleanup must succeed for {}: {error}",
                self.name
            ),
        }
    }

    fn try_cleanup(&self) -> Result<(), postgres::Error> {
        let mut client = self.admin.connect(NoTls)?;
        let role = quote_identifier(&self.name);
        let sql = format!(
            "REVOKE SET ON PARAMETER event_triggers FROM {role}; DROP ROLE IF EXISTS {role}"
        );
        client.batch_execute(&sql)
    }
}

fn create_scratch_login(client: &mut postgres::Client, name: &str) -> String {
    // Generate and register the credential server-side, so client SQL never
    // contains its cleartext value. The catalog receives a SCRAM verifier.
    client
        .batch_execute(
            "CREATE FUNCTION pg_temp.create_scratch_login(role_name text) RETURNS text
         LANGUAGE plpgsql SET password_encryption = 'scram-sha-256' AS $$
         DECLARE generated_password text := pg_catalog.gen_random_uuid()::text;
         BEGIN
             EXECUTE pg_catalog.format(
                 'CREATE ROLE %I LOGIN PASSWORD %L NOSUPERUSER NOCREATEDB NOCREATEROLE',
                 role_name, generated_password);
             RETURN generated_password;
         END $$",
        )
        .unwrap();
    client
        .query_one("SELECT pg_temp.create_scratch_login($1)", &[&name])
        .unwrap()
        .get(0)
}

impl Drop for ScratchRole {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        if std::thread::panicking() {
            let _unwind_cleanup = self.try_cleanup();
            return;
        }
        match self.try_cleanup() {
            Ok(()) => self.active = false,
            Err(error) => panic!("scratch role cleanup failed for {}: {error}", self.name),
        }
    }
}

fn admin_config(base: &Config) -> Config {
    let mut admin = base.clone();
    admin
        .dbname("postgres")
        .connect_timeout(CATALOG_CONNECT_TIMEOUT)
        .tcp_user_timeout(CATALOG_TCP_USER_TIMEOUT)
        .options("-c statement_timeout=120000ms -c lock_timeout=5000ms");
    admin
}

fn database_user(config: &Config) -> &str {
    config
        .get_user()
        .expect("DSN must name an administrative user")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisposableHarnessRejection {
    Target,
    User,
    Database,
    Canary,
    Connection,
    Runtime,
}

fn validate_disposable_harness_target(
    config: &Config,
    canary: Option<&str>,
) -> Result<(), DisposableHarnessRejection> {
    validate_connection_target(config).map_err(|_| DisposableHarnessRejection::Target)?;
    let target_is_exact = matches!(
        config.get_hosts(),
        [Host::Tcp(host)] if host == "127.0.0.1"
    ) && config.get_hostaddrs().is_empty()
        && config.get_ports().len() == 1;
    if !target_is_exact {
        return Err(DisposableHarnessRejection::Target);
    }
    if config.get_user() != Some("test") {
        return Err(DisposableHarnessRejection::User);
    }
    if config.get_dbname() != Some("postgres") {
        return Err(DisposableHarnessRejection::Database);
    }
    let canary = canary.ok_or(DisposableHarnessRejection::Canary)?;
    let valid_canary = canary.len() == 32
        && canary
            .bytes()
            .take(33)
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'));
    valid_canary
        .then_some(())
        .ok_or(DisposableHarnessRejection::Canary)
}

fn preflight_disposable_harness(config: &Config) {
    let canary = std::env::var(DISPOSABLE_CANARY_ENV)
        .map_err(|_| DisposableHarnessRejection::Canary)
        .and_then(|value| {
            validate_disposable_harness_target(config, Some(&value))?;
            Ok(value)
        })
        .unwrap_or_else(|_| panic!("disposable harness target/canary preflight failed"));
    let mut bounded = config.clone();
    bounded
        .connect_timeout(CATALOG_CONNECT_TIMEOUT)
        .tcp_user_timeout(CATALOG_TCP_USER_TIMEOUT)
        .options(CATALOG_STARTUP_OPTIONS);
    let mut client = bounded
        .connect(NoTls)
        .map_err(|_| DisposableHarnessRejection::Connection)
        .unwrap_or_else(|_| panic!("disposable harness connection preflight failed"));
    let row = client
        .query_one(
            "SELECT pg_catalog.current_setting('server_version_num'), \
                    pg_catalog.current_setting('babylon.disposable_runtime', true), \
                    current_user::pg_catalog.text, pg_catalog.current_database(), \
                    role_row.rolsuper, \
                    (SELECT available.default_version FROM pg_catalog.pg_available_extensions \
                     AS available WHERE available.name = 'postgis'), \
                    (SELECT available.default_version FROM pg_catalog.pg_available_extensions \
                     AS available WHERE available.name = 'vector'), \
                    (SELECT available.default_version FROM pg_catalog.pg_available_extensions \
                     AS available WHERE available.name = 'h3'), \
                    pg_catalog.current_setting('transaction_read_only') \
             FROM pg_catalog.pg_roles AS role_row WHERE role_row.rolname = current_user",
            &[],
        )
        .map_err(|_| DisposableHarnessRejection::Runtime)
        .unwrap_or_else(|_| panic!("disposable harness runtime query failed"));
    let runtime = (
        row.try_get::<_, String>(0)
            .map_err(|_| DisposableHarnessRejection::Runtime),
        row.try_get::<_, Option<String>>(1)
            .map_err(|_| DisposableHarnessRejection::Runtime),
        row.try_get::<_, String>(2)
            .map_err(|_| DisposableHarnessRejection::Runtime),
        row.try_get::<_, String>(3)
            .map_err(|_| DisposableHarnessRejection::Runtime),
        row.try_get::<_, bool>(4)
            .map_err(|_| DisposableHarnessRejection::Runtime),
        row.try_get::<_, Option<String>>(5)
            .map_err(|_| DisposableHarnessRejection::Runtime),
        row.try_get::<_, Option<String>>(6)
            .map_err(|_| DisposableHarnessRejection::Runtime),
        row.try_get::<_, Option<String>>(7)
            .map_err(|_| DisposableHarnessRejection::Runtime),
        row.try_get::<_, String>(8)
            .map_err(|_| DisposableHarnessRejection::Runtime),
    );
    assert_eq!(
        runtime,
        (
            Ok("170011".into()),
            Ok(Some(canary)),
            Ok("test".into()),
            Ok("postgres".into()),
            Ok(true),
            Ok(Some("3.5.7".into())),
            Ok(Some("0.8.5".into())),
            Ok(Some("4.5.0".into())),
            Ok("on".into()),
        ),
        "disposable harness runtime profile must match the pinned oracle"
    );
}

fn config_from_env() -> Config {
    let acknowledgement =
        std::env::var(DISPOSABLE_ACK_ENV).expect("BABYLON_POSTGRES_DISPOSABLE_ACK must be set");
    assert_eq!(
        acknowledgement, DISPOSABLE_ACK_VALUE,
        "BABYLON_POSTGRES_DISPOSABLE_ACK must exactly acknowledge destructive cleanup"
    );
    let dsn = std::env::var(DSN_ENV).expect("BABYLON_POSTGRES_TEST_DSN must be set");
    let canary = std::env::var(DISPOSABLE_CANARY_ENV)
        .expect("BABYLON_POSTGRES_DISPOSABLE_CANARY must be set");
    let config = Config::from_str(&dsn).expect("BABYLON_POSTGRES_TEST_DSN must parse");
    validate_disposable_harness_target(&config, Some(&canary))
        .unwrap_or_else(|_| panic!("disposable harness target/canary validation failed"));
    config
}

fn scratch_name(label: &str) -> String {
    format!("native_ref_{label}_{}", unique_suffix())
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

fn quote_identifier(identifier: &str) -> String {
    assert!(identifier.len() <= 63);
    assert!(identifier
        .bytes()
        .all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'_')));
    format!("\"{identifier}\"")
}
