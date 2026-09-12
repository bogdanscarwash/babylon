//! Live `PostgreSQL` contracts for the Michigan H3 reference-bundle installer.

use super::{assert_lock_released, database_user, ScratchDatabase};
use babylon_kernel::tick_content_hash::RefDigest;
use babylon_persistence::{
    h3_reference_cohort::representative_h3_reference_cohort,
    h3_reference_cohort::H3ReferenceCohort, install_current_schema,
    install_michigan_h3_reference_bundle, michigan_dynamic_hex_foundation,
    postgres_catalog::CatalogError, CurrentSchemaDisposition, CurrentSchemaError,
    H3ReferenceInstallConflict, H3ReferenceInstallDisposition, H3ReferenceInstallError,
    H3ReferenceInstallOperation, H3ReferenceInstallReport, SCHEMA_ADVISORY_LOCK_KEY,
};
use postgres::{Config, NoTls};

const CLOSURE_COUNT: usize = 59_849;
const SOURCE_COUNT: usize = 48_764;
const R8_CHILD_COUNT: usize = 319_004;
const MAX_COHORT_SNAPSHOT_ROWS: usize = 2;
const MAX_EPOCH_CATALOG_ROWS: usize = 16;
const ARTIFACT_DIGEST_HEX: &str =
    "e60d93a43d6c66e84f1e53ecaf633af5911bd5b48b0ef0ad6a012f6d9f5b13a9";
const REF_DIGEST_HEX: &str = "92b21ff325bde67f26565f52882d3664daacd6d51423f2a588344da012fd4161";
const R8_SECTION_DIGEST_HEX: &str =
    "b5ebf405140f6f79ddbc44fa1005b195bed0bc28e0eacf2d8e1697cd9c839491";
const REFERENCE_BUNDLE_DIGEST_HEX: &str =
    "84bbffa9b2388aa168c065e710a61313fbd46522d2022b628f0919ecffec9831";

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReferenceSnapshot {
    h3_cell_count: i64,
    cohort_count: i64,
    membership_count: i64,
    direct_membership_count: i64,
    derived_membership_count: i64,
    product_count: i64,
    cohorts: Vec<CohortSnapshot>,
    products: Vec<ProductSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CohortSnapshot {
    ref_digest: String,
    format_version: i16,
    artifact_name: String,
    artifact_manifest_version: String,
    artifact_digest: String,
    source_digest: String,
    source_r5_digest: String,
    source_r7_digest: String,
    closure_digest: String,
    membership_digest: String,
    direct_cell_count: i64,
    derived_ancestor_count: i64,
    closure_cell_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProductSnapshot {
    ref_digest: String,
    product_code: String,
    artifact_sha256: String,
    semantic_sha256: Option<String>,
    row_count: i64,
    evidence_class: String,
    measure_unit: Option<String>,
    denominator: Option<String>,
}

pub(super) fn verify_h3_reference_installer(base: &Config, owner: &str, owner_password: &str) {
    let cohort = representative_cohort();
    verify_connection_failure_redacts_credentials(&cohort);
    verify_exact_epoch_install_and_retry(base, &cohort);
    verify_fresh_refusal(base, &cohort);
    verify_lock_refusal(base, &cohort);
    verify_non_owner_refusal(base, owner, owner_password, &cohort);
    verify_installed_state_conflicts(base, &cohort);
    verify_preflight_artifact_identity_conflict(base, &cohort);
}

fn verify_exact_epoch_install_and_retry(base: &Config, cohort: &H3ReferenceCohort) {
    let (database, config) = exact_epoch_database(base, "h3_installer_exact");

    let installed = install_reference_bundle(&config, cohort)
        .expect("the exact current epoch must install the Michigan H3 reference bundle");
    assert_exact_report(&installed, H3ReferenceInstallDisposition::Installed, 1);
    let before_retry = reference_snapshot(&config);
    assert_eq!(before_retry, expected_installed_snapshot());

    let retry = install_reference_bundle(&config, cohort)
        .expect("an exact installed reference bundle must be idempotent");
    assert_exact_report(&retry, H3ReferenceInstallDisposition::AlreadyPresent, 0);
    assert_eq!(reference_snapshot(&config), before_retry);
    assert_lock_released(&config);
    database.cleanup();
}

fn verify_connection_failure_redacts_credentials(cohort: &H3ReferenceCohort) {
    const PASSWORD: &str = "do-not-leak-h3-password";

    let mut unavailable = Config::new();
    unavailable.host("127.0.0.1").port(1).password(PASSWORD);
    let error = install_reference_bundle(&unavailable, cohort)
        .expect_err("the unreachable loopback port must refuse the installer connection");

    match &error {
        H3ReferenceInstallError::Database {
            operation: H3ReferenceInstallOperation::Connect,
            diagnostic: Some(diagnostic),
        } => assert_eq!(
            diagnostic.classification(),
            babylon_persistence::PostgresFailureClass::Reachability
        ),
        _ => panic!("connection refusal must remain a redacted typed database error"),
    }
    assert!(!format!("{error:?}").contains(PASSWORD));
    assert!(!format!("{error}").contains(PASSWORD));
}

fn verify_fresh_refusal(base: &Config, cohort: &H3ReferenceCohort) {
    let database = ScratchDatabase::empty(base, "h3_installer_fresh", database_user(base));
    let config = database.config(base);
    let before = babylon_catalog_snapshot(&config);
    assert!(matches!(
        install_reference_bundle(&config, cohort),
        Err(H3ReferenceInstallError::CurrentSchema(_))
    ));
    assert_eq!(babylon_catalog_snapshot(&config), before);
    assert_lock_released(&config);
    database.cleanup();
}

fn verify_lock_refusal(base: &Config, cohort: &H3ReferenceCohort) {
    let (database, config) = exact_epoch_database(base, "h3_installer_lock");
    let mut blocker = config.connect(NoTls).unwrap();
    let locked: bool = blocker
        .query_one(
            "SELECT pg_catalog.pg_try_advisory_lock($1)",
            &[&SCHEMA_ADVISORY_LOCK_KEY],
        )
        .unwrap()
        .get(0);
    assert!(locked);
    let before = reference_snapshot(&config);
    assert!(matches!(
        install_reference_bundle(&config, cohort),
        Err(H3ReferenceInstallError::Lock(CatalogError::LockUnavailable))
    ));
    assert_eq!(reference_snapshot(&config), before);
    let unlocked: bool = blocker
        .query_one(
            "SELECT pg_catalog.pg_advisory_unlock($1)",
            &[&SCHEMA_ADVISORY_LOCK_KEY],
        )
        .unwrap()
        .get(0);
    assert!(unlocked);
    drop(blocker);
    assert_lock_released(&config);
    database.cleanup();
}

fn verify_non_owner_refusal(
    base: &Config,
    owner: &str,
    owner_password: &str,
    cohort: &H3ReferenceCohort,
) {
    let database = ScratchDatabase::empty(base, "h3_installer_non_owner", owner);
    let owner_config = database.config_as(base, owner, owner_password);
    let report =
        install_current_schema(&owner_config).expect("database owner must establish current epoch");
    assert_eq!(report.disposition, CurrentSchemaDisposition::Installed);

    let admin_config = database.config(base);
    let before = reference_snapshot(&admin_config);
    assert_eq!(
        install_reference_bundle(&admin_config, cohort),
        Err(H3ReferenceInstallError::CurrentSchema(
            CurrentSchemaError::CurrentUserIsNotDatabaseOwner,
        )),
        "non-owner installer call must refuse through the exact owner check"
    );
    assert_eq!(reference_snapshot(&admin_config), before);
    assert_lock_released(&admin_config);
    database.cleanup();
}

fn verify_preflight_artifact_identity_conflict(base: &Config, cohort: &H3ReferenceCohort) {
    let (database, config) = exact_epoch_database(base, "h3_installer_preflight_conflict");
    seed_conflicting_artifact_identity(&config);
    let before = reference_snapshot(&config);
    assert_eq!((before.h3_cell_count, before.cohort_count), (0, 1));
    assert_eq!(
        install_reference_bundle(&config, cohort),
        Err(H3ReferenceInstallError::Conflict {
            component: H3ReferenceInstallConflict::ArtifactIdentity,
        })
    );
    assert_eq!(reference_snapshot(&config), before);
    assert_lock_released(&config);
    database.cleanup();
}

fn verify_installed_state_conflicts(base: &Config, cohort: &H3ReferenceCohort) {
    let (template, config) = exact_epoch_database(base, "h3_installer_mutation_template");
    let installed = install_reference_bundle(&config, cohort)
        .expect("mutation template must contain the exact installed reference bundle");
    assert_exact_report(&installed, H3ReferenceInstallDisposition::Installed, 1);
    assert_eq!(reference_snapshot(&config), expected_installed_snapshot());

    verify_installed_mutation_refusal(
        base,
        template.name(),
        cohort,
        "h3_installer_header_metadata",
        mutate_header_metadata,
        H3ReferenceInstallConflict::CohortHeader,
    );
    verify_installed_mutation_refusal(
        base,
        template.name(),
        cohort,
        "h3_installer_header_count",
        mutate_header_count,
        H3ReferenceInstallConflict::CohortHeader,
    );
    verify_installed_mutation_refusal(
        base,
        template.name(),
        cohort,
        "h3_installer_missing_membership",
        mutate_missing_membership,
        H3ReferenceInstallConflict::Membership,
    );
    verify_installed_mutation_refusal(
        base,
        template.name(),
        cohort,
        "h3_installer_r8_cell",
        mutate_r8_cell,
        H3ReferenceInstallConflict::R8Cells,
    );
    verify_installed_mutation_refusal(
        base,
        template.name(),
        cohort,
        "h3_installer_r8_product",
        mutate_r8_product,
        H3ReferenceInstallConflict::R8ProductReceipt,
    );
    verify_installed_mutation_refusal(
        base,
        template.name(),
        cohort,
        "h3_installer_missing_r8_product",
        mutate_missing_r8_product,
        H3ReferenceInstallConflict::R8ProductReceipt,
    );
    verify_installed_mutation_refusal(
        base,
        template.name(),
        cohort,
        "h3_installer_changed_origin",
        mutate_membership_origin,
        H3ReferenceInstallConflict::Membership,
    );
    verify_installed_mutation_refusal(
        base,
        template.name(),
        cohort,
        "h3_installer_orphan_membership",
        mutate_orphan_membership,
        H3ReferenceInstallConflict::Membership,
    );
    assert_lock_released(&config);
    template.cleanup();
}

fn verify_installed_mutation_refusal(
    base: &Config,
    template: &str,
    cohort: &H3ReferenceCohort,
    label: &str,
    mutate: fn(&Config),
    expected_component: H3ReferenceInstallConflict,
) {
    let database = ScratchDatabase::from_template(base, template, label);
    let config = database.config(base);
    let exact = reference_snapshot(&config);
    assert_eq!(exact, expected_installed_snapshot());
    mutate(&config);
    let conflicted = reference_snapshot(&config);
    assert_ne!(conflicted, exact);
    assert_eq!(
        install_reference_bundle(&config, cohort),
        Err(H3ReferenceInstallError::Conflict {
            component: expected_component,
        }),
        "installed mutation must produce its typed conflict: {label}"
    );
    assert_eq!(
        reference_snapshot(&config),
        conflicted,
        "conflict refusal must not write: {label}"
    );
    assert_lock_released(&config);
    database.cleanup();
}

fn mutate_header_metadata(config: &Config) {
    let ref_digest = digest(REF_DIGEST_HEX);
    let mut client = config.connect(NoTls).unwrap();
    let changed = client
        .execute(
            "UPDATE babylon_ref.h3_reference_cohort \
             SET artifact_name = 'changed_h3.parquet' WHERE ref_digest = $1",
            &[&ref_digest.as_bytes().as_slice()],
        )
        .unwrap();
    assert_eq!(changed, 1);
}

fn mutate_header_count(config: &Config) {
    let ref_digest = digest(REF_DIGEST_HEX);
    let mut client = config.connect(NoTls).unwrap();
    let changed = client
        .execute(
            "UPDATE babylon_ref.h3_reference_cohort \
             SET direct_cell_count = direct_cell_count + 1, \
                 derived_ancestor_count = derived_ancestor_count - 1 \
             WHERE ref_digest = $1",
            &[&ref_digest.as_bytes().as_slice()],
        )
        .unwrap();
    assert_eq!(changed, 1);
}

fn mutate_missing_membership(config: &Config) {
    let ref_digest = digest(REF_DIGEST_HEX);
    let mut client = config.connect(NoTls).unwrap();
    let changed = client
        .execute(
            "DELETE FROM babylon_ref.h3_reference_membership \
             WHERE ref_digest = $1 AND cell_id = ( \
                 SELECT cell_id FROM babylon_ref.h3_reference_membership \
                 WHERE ref_digest = $1 AND origin = 1 ORDER BY cell_id LIMIT 1 \
             )",
            &[&ref_digest.as_bytes().as_slice()],
        )
        .unwrap();
    assert_eq!(changed, 1);
}

fn mutate_membership_origin(config: &Config) {
    let ref_digest = digest(REF_DIGEST_HEX);
    let mut client = config.connect(NoTls).unwrap();
    let changed = client
        .execute(
            "UPDATE babylon_ref.h3_reference_membership SET origin = 2 \
             WHERE ref_digest = $1 AND cell_id = ( \
                 SELECT cell_id FROM babylon_ref.h3_reference_membership \
                 WHERE ref_digest = $1 AND origin = 1 ORDER BY cell_id LIMIT 1 \
             )",
            &[&ref_digest.as_bytes().as_slice()],
        )
        .unwrap();
    assert_eq!(changed, 1);
}

fn mutate_orphan_membership(config: &Config) {
    let ref_digest = digest(REF_DIGEST_HEX);
    let orphan_cell_id = 1_i64;
    let mut client = config.connect(NoTls).unwrap();
    let mut transaction = client.transaction().unwrap();
    let exists: bool = transaction
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM babylon_ref.h3_cell WHERE cell_id = $1)",
            &[&orphan_cell_id],
        )
        .unwrap()
        .get(0);
    assert!(!exists);
    transaction
        .batch_execute("SET LOCAL session_replication_role = replica")
        .unwrap();
    let deleted = transaction
        .execute(
            "DELETE FROM babylon_ref.h3_reference_membership \
             WHERE ref_digest = $1 AND cell_id = ( \
                 SELECT cell_id FROM babylon_ref.h3_reference_membership \
                 WHERE ref_digest = $1 AND origin = 1 ORDER BY cell_id LIMIT 1 \
             )",
            &[&ref_digest.as_bytes().as_slice()],
        )
        .unwrap();
    assert_eq!(deleted, 1);
    let inserted = transaction
        .execute(
            "INSERT INTO babylon_ref.h3_reference_membership (ref_digest, cell_id, origin) \
             VALUES ($1, $2, 2)",
            &[&ref_digest.as_bytes().as_slice(), &orphan_cell_id],
        )
        .unwrap();
    assert_eq!(inserted, 1);
    transaction.commit().unwrap();
}

fn mutate_r8_cell(config: &Config) {
    let mut client = config.connect(NoTls).unwrap();
    let changed = client
        .execute(
            "DELETE FROM babylon_ref.h3_cell \
             WHERE cell_id = (SELECT cell_id FROM babylon_ref.h3_cell \
             WHERE resolution = 8 ORDER BY cell_id LIMIT 1)",
            &[],
        )
        .unwrap();
    assert_eq!(changed, 1);
}

fn mutate_r8_product(config: &Config) {
    let mut client = config.connect(NoTls).unwrap();
    let changed = client
        .execute(
            "UPDATE babylon_ref.reference_product \
             SET semantic_sha256 = pg_catalog.decode(pg_catalog.repeat('00', 32), 'hex') \
             WHERE product_code = 'h3_res8_identity'",
            &[],
        )
        .unwrap();
    assert_eq!(changed, 1);
}

fn mutate_missing_r8_product(config: &Config) {
    let mut client = config.connect(NoTls).unwrap();
    let changed = client
        .execute(
            "DELETE FROM babylon_ref.reference_product \
             WHERE product_code = 'h3_res8_identity'",
            &[],
        )
        .unwrap();
    assert_eq!(changed, 1);
}

fn assert_exact_report(
    report: &H3ReferenceInstallReport,
    disposition: H3ReferenceInstallDisposition,
    commit_attempts: usize,
) {
    assert_eq!(report.disposition(), disposition);
    assert_eq!(report.ref_digest(), digest(REF_DIGEST_HEX));
    assert_eq!(report.artifact_digest(), digest(ARTIFACT_DIGEST_HEX));
    assert_eq!(report.format_version(), 1);
    assert_eq!(report.artifact_name(), "bridge_county_h3.parquet");
    assert_eq!(report.artifact_manifest_version(), "2.0.0");
    assert_eq!(report.direct_cell_count(), SOURCE_COUNT);
    assert_eq!(report.derived_ancestor_count(), 11_085);
    assert_eq!(report.closure_cell_count(), CLOSURE_COUNT);
    assert_eq!(report.r8_child_count(), R8_CHILD_COUNT);
    assert_eq!(
        report.r8_child_parent_digest(),
        digest(R8_SECTION_DIGEST_HEX)
    );
    assert_eq!(
        report.reference_bundle_digest(),
        digest(REFERENCE_BUNDLE_DIGEST_HEX)
    );
    assert_eq!(report.commit_attempts(), commit_attempts);
}

fn exact_epoch_database(base: &Config, label: &str) -> (ScratchDatabase, Config) {
    let database = ScratchDatabase::empty(base, label, database_user(base));
    let config = database.config(base);
    let report =
        install_current_schema(&config).expect("fresh database must reach the exact current epoch");
    assert_eq!(report.disposition, CurrentSchemaDisposition::Installed);
    (database, config)
}

fn seed_conflicting_artifact_identity(config: &Config) {
    let mut client = config.connect(NoTls).unwrap();
    client
        .batch_execute(
            "INSERT INTO babylon_ref.h3_reference_cohort ( \
                 ref_digest, format_version, artifact_name, artifact_manifest_version, \
                 artifact_digest, source_digest, source_r5_digest, source_r7_digest, \
                 closure_digest, membership_digest, direct_cell_count, \
                 derived_ancestor_count, closure_cell_count \
             ) VALUES ( \
                 decode(repeat('11', 32), 'hex'), 1, 'bridge_county_h3.parquet', '2.0.0', \
                 decode('e60d93a43d6c66e84f1e53ecaf633af5911bd5b48b0ef0ad6a012f6d9f5b13a9', 'hex'), \
                 decode(repeat('22', 32), 'hex'), decode(repeat('33', 32), 'hex'), \
                 decode(repeat('44', 32), 'hex'), decode(repeat('55', 32), 'hex'), \
                 decode(repeat('66', 32), 'hex'), 1, 0, 1 \
             )",
        )
        .unwrap();
}

fn reference_snapshot(config: &Config) -> ReferenceSnapshot {
    let mut client = config.connect(NoTls).unwrap();
    let counts = client
        .query_one(
            "SELECT (SELECT pg_catalog.count(*) FROM babylon_ref.h3_cell), \
                    (SELECT pg_catalog.count(*) FROM babylon_ref.h3_reference_cohort), \
                    (SELECT pg_catalog.count(*) FROM babylon_ref.h3_reference_membership), \
                    (SELECT pg_catalog.count(*) FILTER (WHERE origin = 1) \
                     FROM babylon_ref.h3_reference_membership), \
                    (SELECT pg_catalog.count(*) FILTER (WHERE origin = 2) \
                     FROM babylon_ref.h3_reference_membership), \
                    (SELECT pg_catalog.count(*) FROM babylon_ref.reference_product)",
            &[],
        )
        .unwrap();
    let cohort_count = counts.get(1);
    assert!(cohort_count <= i64::try_from(MAX_COHORT_SNAPSHOT_ROWS).unwrap());
    let rows = client
        .query(
            "SELECT pg_catalog.encode(ref_digest, 'hex'), format_version, artifact_name, \
                    artifact_manifest_version, pg_catalog.encode(artifact_digest, 'hex'), \
                    pg_catalog.encode(source_digest, 'hex'), \
                    pg_catalog.encode(source_r5_digest, 'hex'), \
                    pg_catalog.encode(source_r7_digest, 'hex'), \
                    pg_catalog.encode(closure_digest, 'hex'), \
                    pg_catalog.encode(membership_digest, 'hex'), direct_cell_count, \
                    derived_ancestor_count, closure_cell_count \
             FROM babylon_ref.h3_reference_cohort ORDER BY ref_digest LIMIT 2",
            &[],
        )
        .unwrap();
    assert_eq!(i64::try_from(rows.len()).unwrap(), cohort_count);
    let product_count: i64 = counts.get(5);
    assert!(product_count <= 1);
    let products = client
        .query(
            "SELECT pg_catalog.encode(ref_digest, 'hex'), product_code, \
                    pg_catalog.encode(artifact_sha256, 'hex'), \
                    pg_catalog.encode(semantic_sha256, 'hex'), row_count, evidence_class, \
                    measure_unit, denominator \
             FROM babylon_ref.reference_product ORDER BY product_code LIMIT 2",
            &[],
        )
        .unwrap();
    assert_eq!(i64::try_from(products.len()).unwrap(), product_count);
    ReferenceSnapshot {
        h3_cell_count: counts.get(0),
        cohort_count,
        membership_count: counts.get(2),
        direct_membership_count: counts.get(3),
        derived_membership_count: counts.get(4),
        product_count,
        cohorts: rows
            .iter()
            .take(MAX_COHORT_SNAPSHOT_ROWS)
            .map(cohort_snapshot)
            .collect(),
        products: products.iter().take(1).map(product_snapshot).collect(),
    }
}

fn cohort_snapshot(row: &postgres::Row) -> CohortSnapshot {
    CohortSnapshot {
        ref_digest: row.get(0),
        format_version: row.get(1),
        artifact_name: row.get(2),
        artifact_manifest_version: row.get(3),
        artifact_digest: row.get(4),
        source_digest: row.get(5),
        source_r5_digest: row.get(6),
        source_r7_digest: row.get(7),
        closure_digest: row.get(8),
        membership_digest: row.get(9),
        direct_cell_count: row.get(10),
        derived_ancestor_count: row.get(11),
        closure_cell_count: row.get(12),
    }
}

fn product_snapshot(row: &postgres::Row) -> ProductSnapshot {
    ProductSnapshot {
        ref_digest: row.get(0),
        product_code: row.get(1),
        artifact_sha256: row.get(2),
        semantic_sha256: row.get(3),
        row_count: row.get(4),
        evidence_class: row.get(5),
        measure_unit: row.get(6),
        denominator: row.get(7),
    }
}

fn expected_installed_snapshot() -> ReferenceSnapshot {
    ReferenceSnapshot {
        h3_cell_count: 378_853,
        cohort_count: 1,
        membership_count: 59_849,
        direct_membership_count: 48_764,
        derived_membership_count: 11_085,
        product_count: 1,
        cohorts: vec![CohortSnapshot {
            ref_digest: REF_DIGEST_HEX.into(),
            format_version: 1,
            artifact_name: "bridge_county_h3.parquet".into(),
            artifact_manifest_version: "2.0.0".into(),
            artifact_digest: ARTIFACT_DIGEST_HEX.into(),
            source_digest: "a4685e6ad882930e7064cb225ee649155fb74e52ef8b7d7550691a70a6087f5a"
                .into(),
            source_r5_digest: "83c093393bdf7a0e30ace8e208f3bcaa366fb7c6350abf7ff55d446322dcca87"
                .into(),
            source_r7_digest: "7f8d126ee81356a60605013b4b1c23942a77a4b2d6f890125d6c938dae70228b"
                .into(),
            closure_digest: "467cb7d1af751fe522cc3de818107068373531e51a4d9a7371a3f5f9becae29b"
                .into(),
            membership_digest: "4bbcdbf0c592b2cdc7ad52a8a8a5ef9a7e9989bd1b11b159be6eec5f2150247f"
                .into(),
            direct_cell_count: 48_764,
            derived_ancestor_count: 11_085,
            closure_cell_count: 59_849,
        }],
        products: vec![ProductSnapshot {
            ref_digest: REF_DIGEST_HEX.into(),
            product_code: "h3_res8_identity".into(),
            artifact_sha256: R8_SECTION_DIGEST_HEX.into(),
            semantic_sha256: Some(R8_SECTION_DIGEST_HEX.into()),
            row_count: 319_004,
            evidence_class: "Derived".into(),
            measure_unit: Some("identity".into()),
            denominator: None,
        }],
    }
}

fn babylon_catalog_snapshot(config: &Config) -> Vec<(String, String)> {
    let mut client = config.connect(NoTls).unwrap();
    let limit = i64::try_from(MAX_EPOCH_CATALOG_ROWS + 1).unwrap();
    let rows = client
        .query(
            "SELECT object_kind, object_name FROM ( \
                 SELECT 'schema'::pg_catalog.text AS object_kind, \
                        namespace.nspname::pg_catalog.text AS object_name \
                 FROM pg_catalog.pg_namespace AS namespace \
                 WHERE namespace.nspname LIKE 'babylon\\_%' ESCAPE '\\' \
                 UNION ALL \
                 SELECT relation.relkind::pg_catalog.text, \
                        namespace.nspname || '.' || relation.relname \
                 FROM pg_catalog.pg_class AS relation \
                 JOIN pg_catalog.pg_namespace AS namespace \
                   ON namespace.oid = relation.relnamespace \
                 WHERE namespace.nspname LIKE 'babylon\\_%' ESCAPE '\\' \
             ) AS objects ORDER BY object_kind, object_name LIMIT $1",
            &[&limit],
        )
        .unwrap();
    assert!(rows.len() <= MAX_EPOCH_CATALOG_ROWS);
    rows.iter()
        .take(MAX_EPOCH_CATALOG_ROWS)
        .map(|row| (row.get(0), row.get(1)))
        .collect()
}

pub(super) fn representative_cohort() -> H3ReferenceCohort {
    representative_h3_reference_cohort()
        .expect("the sole checked-in source fixture must validate")
        .clone()
}

pub(super) fn install_reference_bundle(
    config: &Config,
    cohort: &H3ReferenceCohort,
) -> Result<H3ReferenceInstallReport, H3ReferenceInstallError> {
    let foundation = michigan_dynamic_hex_foundation()
        .expect("the sole checked Michigan foundation fixture must validate");
    install_michigan_h3_reference_bundle(config, cohort, foundation)
}

fn digest(text: &str) -> RefDigest {
    assert_eq!(text.len(), 64);
    let mut bytes = [0_u8; 32];
    for (index, byte) in bytes.iter_mut().enumerate().take(32) {
        let offset = index * 2;
        *byte = u8::from_str_radix(&text[offset..offset + 2], 16).unwrap();
    }
    RefDigest::from_bytes(bytes)
}
