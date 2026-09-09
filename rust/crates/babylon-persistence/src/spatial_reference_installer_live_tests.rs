use postgres::{error::SqlState, Config, NoTls};

use super::{
    attempt_install_transaction, conflict, install_michigan_spatial_reference_products,
    install_michigan_spatial_reference_products_using, prepare_install_transaction,
    rollback_preserving, CommitAttempt, SpatialReferenceInstallDisposition,
    SpatialReferenceInstallError, SpatialReferenceRelation,
};
use crate::{
    install_michigan_h3_reference_bundle_v1, michigan_dynamic_hex_foundation_v1,
    representative_h3_reference_cohort_v1, H3ReferenceCohort, H3ReferenceInstallDisposition,
};

const BACKEND_TERMINATION_TIMEOUT_MILLIS: i64 = 5_000;
const FOUNDATION_RECEIPT_COUNTS: [i64; 8] = [1, 0, 0, 0, 0, 0, 0, 0];
const EXPECTED_COUNTS: [i64; 8] = [8, 3_285, 745, 45_572, 22_509, 11_833, 31_881, 4_813];

pub(crate) fn verify_commit_protocol(config: &Config, admin: &Config) {
    let cohort = representative_cohort();
    let foundation = michigan_dynamic_hex_foundation_v1()
        .expect("the sole checked Michigan foundation fixture must validate");
    let h3_report = install_michigan_h3_reference_bundle_v1(config, &cohort, foundation)
        .expect("the exact H3 cohort must install before its reference products");
    assert_eq!(
        h3_report.disposition(),
        H3ReferenceInstallDisposition::Installed
    );
    assert_eq!(reference_counts(config), FOUNDATION_RECEIPT_COUNTS);

    let mut forced_failure = |client: &mut postgres::Client, bundle: &_| {
        let transaction = prepare_install_transaction(client, bundle)?;
        rollback_preserving(transaction, conflict(SpatialReferenceRelation::Products))
    };
    assert_eq!(
        install_michigan_spatial_reference_products_using(config, &cohort, &mut forced_failure,),
        Err(SpatialReferenceInstallError::Conflict {
            relation: SpatialReferenceRelation::Products,
        })
    );
    assert_eq!(reference_counts(config), FOUNDATION_RECEIPT_COUNTS);

    let mut first_attempt = true;
    let mut committed_ambiguity = |client: &mut postgres::Client, bundle: &_| {
        let backend_pid = backend_pid(client);
        let outcome = attempt_install_transaction(client, bundle)?;
        if first_attempt {
            first_attempt = false;
            assert_eq!(outcome, CommitAttempt::Committed);
            terminate_backend(admin, backend_pid);
            Ok(CommitAttempt::Ambiguous)
        } else {
            Ok(outcome)
        }
    };
    let report = install_michigan_spatial_reference_products_using(
        config,
        &cohort,
        &mut committed_ambiguity,
    )
    .expect("a committed ambiguous install must reconcile to the exact bundle");
    assert_eq!(
        report.disposition(),
        SpatialReferenceInstallDisposition::ReconciledAfterAmbiguousCommit
    );
    assert_eq!(report.product_count(), 7);
    assert_eq!(report.data_row_count(), 120_638);
    assert_eq!(report.commit_attempts(), 1);
    assert_eq!(reference_counts(config), EXPECTED_COUNTS);

    let retry = install_michigan_spatial_reference_products(config, &cohort)
        .expect("an exact bundle retry must be idempotent");
    assert_eq!(
        retry.disposition(),
        SpatialReferenceInstallDisposition::AlreadyPresent
    );
    assert_eq!(retry.commit_attempts(), 0);
    assert_eq!(reference_counts(config), EXPECTED_COUNTS);

    refuse_derived_ancestor_as_product_cell(config);
    mutate_product_digest(config);
    let mutated_counts = reference_counts(config);
    assert_eq!(
        install_michigan_spatial_reference_products(config, &cohort),
        Err(SpatialReferenceInstallError::Conflict {
            relation: SpatialReferenceRelation::Products,
        })
    );
    assert_eq!(reference_counts(config), mutated_counts);
}

fn refuse_derived_ancestor_as_product_cell(config: &Config) {
    let mut client = config.connect(NoTls).unwrap();
    let membership = client
        .query_one(
            "SELECT ref_digest, cell_id \
             FROM babylon_ref.h3_reference_membership \
             WHERE origin = 2 ORDER BY ref_digest, cell_id LIMIT 1",
            &[],
        )
        .unwrap();
    let ref_digest: Vec<u8> = membership.try_get(0).unwrap();
    let cell_id: i64 = membership.try_get(1).unwrap();
    let mut transaction = client.transaction().unwrap();
    transaction
        .execute(
            "INSERT INTO babylon_ref.h3_population_count \
             (ref_digest, product_code, cell_id, membership_origin, population_count) \
             VALUES ($1, 'h3_res7_population', $2, 1, 1)",
            &[&ref_digest, &cell_id],
        )
        .unwrap();
    let refusal = transaction.commit().unwrap_err();
    assert_eq!(refusal.code(), Some(&SqlState::FOREIGN_KEY_VIOLATION));
    let visible: i64 = client
        .query_one(
            "SELECT pg_catalog.count(*) FROM babylon_ref.h3_population_count \
             WHERE ref_digest = $1 AND cell_id = $2",
            &[&ref_digest, &cell_id],
        )
        .unwrap()
        .try_get(0)
        .unwrap();
    assert_eq!(visible, 0);
}

fn backend_pid(client: &mut postgres::Client) -> i32 {
    client
        .query_one("SELECT pg_catalog.pg_backend_pid()", &[])
        .unwrap()
        .try_get(0)
        .unwrap()
}

fn terminate_backend(admin: &Config, backend_pid: i32) {
    let terminated: bool = admin
        .connect(NoTls)
        .unwrap()
        .query_one(
            "SELECT pg_catalog.pg_terminate_backend($1, $2)",
            &[&backend_pid, &BACKEND_TERMINATION_TIMEOUT_MILLIS],
        )
        .unwrap()
        .try_get(0)
        .unwrap();
    assert!(terminated);
}

fn mutate_product_digest(config: &Config) {
    let changed = config
        .connect(NoTls)
        .unwrap()
        .execute(
            "UPDATE babylon_ref.reference_product \
             SET artifact_sha256 = pg_catalog.decode(pg_catalog.repeat('00', 32), 'hex') \
             WHERE product_code = 'dim_county'",
            &[],
        )
        .unwrap();
    assert_eq!(changed, 1);
}

fn reference_counts(config: &Config) -> [i64; 8] {
    let row = config
        .connect(NoTls)
        .unwrap()
        .query_one(
            "SELECT \
                (SELECT pg_catalog.count(*) FROM babylon_ref.reference_product), \
                (SELECT pg_catalog.count(*) FROM babylon_ref.county_identity), \
                (SELECT pg_catalog.count(*) FROM babylon_ref.place_identity), \
                (SELECT pg_catalog.count(*) FROM babylon_ref.h3_land_fraction), \
                (SELECT pg_catalog.count(*) FROM babylon_ref.h3_population_count), \
                (SELECT pg_catalog.count(*) FROM babylon_ref.h3_workplace_count), \
                (SELECT pg_catalog.count(*) FROM babylon_ref.county_h3_land_area), \
                (SELECT pg_catalog.count(*) FROM babylon_ref.county_place_h3_land_area)",
            &[],
        )
        .unwrap();
    std::array::from_fn(|index| row.get(index))
}

fn representative_cohort() -> H3ReferenceCohort {
    representative_h3_reference_cohort_v1()
        .expect("the sole checked-in source fixture must validate")
        .clone()
}
