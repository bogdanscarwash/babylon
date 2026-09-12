//! Transactional installation of the exact PER-278 spatial reference bundle.

use babylon_kernel::tick_content_hash::RefDigest;
use babylon_kernel::H3CellId;
use postgres::{Client, Config, GenericClient, IsolationLevel, NoTls, Row, Transaction};

use crate::current_schema::{bounded_config, require_current_schema, CurrentSchemaError};
use crate::postgres_catalog::{
    acquire_lock, release_lock, validate_connection_target, CatalogError,
};
use crate::postgres_diagnostic::PostgresDiagnostic;
use crate::{
    h3_reference_cohort::H3ReferenceCohort,
    spatial_reference_products::michigan_spatial_reference_products,
    spatial_reference_products::CountyH3LandAreaRow, spatial_reference_products::CountyIdentityRow,
    spatial_reference_products::CountyPlaceH3LandAreaRow, spatial_reference_products::H3CountRow,
    spatial_reference_products::H3LandFractionRow, spatial_reference_products::PlaceIdentityRow,
    spatial_reference_products::ReferenceProduct,
    spatial_reference_products::SpatialReferenceProducts,
    spatial_reference_products::SpatialReferenceProductsError,
};

const INSTALL_BATCH_ROWS: usize = 1_024;
const MAX_COMMIT_ATTEMPTS: usize = 2;
const PRODUCT_COUNT: usize = 7;
const TOTAL_DATA_ROWS: usize = 120_638;
const SESSION_SETTINGS_SQL: &str = "SET statement_timeout TO '120000ms'";
const SESSION_SETTINGS_QUERY: &str = "SELECT \
    pg_catalog.current_setting('transaction_read_only'), \
    pg_catalog.current_setting('search_path'), \
    pg_catalog.current_setting('statement_timeout'), \
    pg_catalog.current_setting('lock_timeout'), \
    pg_catalog.current_setting('idle_in_transaction_session_timeout'), \
    pg_catalog.current_setting('quote_all_identifiers'), \
    pg_catalog.current_setting('jit'), \
    pg_catalog.current_setting('event_triggers')";
const WRITE_LOCAL_SETTINGS_SQL: &str = "SET LOCAL search_path TO pg_catalog; \
    SET LOCAL synchronous_commit TO on";
const WRITE_SETTINGS_SQL: &str = "SELECT \
    pg_catalog.current_setting('transaction_isolation'), \
    pg_catalog.current_setting('transaction_read_only'), \
    pg_catalog.current_setting('search_path'), \
    pg_catalog.current_setting('synchronous_commit'), \
    pg_catalog.current_setting('statement_timeout'), \
    pg_catalog.current_setting('lock_timeout'), \
    pg_catalog.current_setting('idle_in_transaction_session_timeout'), \
    pg_catalog.current_setting('quote_all_identifiers'), \
    pg_catalog.current_setting('jit'), \
    pg_catalog.current_setting('event_triggers')";

const READ_PRODUCTS_SQL: &str = "SELECT product_code, artifact_sha256, semantic_sha256, \
    row_count, evidence_class, measure_unit, denominator \
    FROM babylon_ref.reference_product \
    WHERE ref_digest = $1 AND product_code = ANY($2::text[]) \
    ORDER BY product_code LIMIT $3";
const INSERT_PRODUCT_SQL: &str = "INSERT INTO babylon_ref.reference_product \
    (ref_digest, product_code, artifact_sha256, semantic_sha256, row_count, evidence_class, \
     measure_unit, denominator) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
    ON CONFLICT DO NOTHING";
const INSERT_COUNTIES_SQL: &str = "INSERT INTO babylon_ref.county_identity \
    (ref_digest, product_code, county_id, county_geoid, state_id, county_fips, county_name) \
    SELECT $1, 'dim_county', input.county_id, input.county_geoid, input.state_id, \
           input.county_fips, input.county_name \
    FROM ROWS FROM (pg_catalog.unnest($2::bigint[]), pg_catalog.unnest($3::text[]), \
                    pg_catalog.unnest($4::integer[]), pg_catalog.unnest($5::text[]), \
                    pg_catalog.unnest($6::text[])) \
      AS input(county_id, county_geoid, state_id, county_fips, county_name) \
    ON CONFLICT DO NOTHING";
const INSERT_PLACES_SQL: &str = "INSERT INTO babylon_ref.place_identity \
    (ref_digest, product_code, place_geoid, state_fips, place_fips, place_ns, name, name_lsad, \
     lsad, class_fp, principal_city_indicator, mtfcc, functional_status) \
    SELECT $1, 'census_place_identity_mi_2023', input.place_geoid, input.state_fips, \
           input.place_fips, input.place_ns, input.name, input.name_lsad, input.lsad, \
           input.class_fp, input.principal_city_indicator, input.mtfcc, input.functional_status \
    FROM ROWS FROM (pg_catalog.unnest($2::text[]), pg_catalog.unnest($3::text[]), \
                    pg_catalog.unnest($4::text[]), pg_catalog.unnest($5::text[]), \
                    pg_catalog.unnest($6::text[]), pg_catalog.unnest($7::text[]), \
                    pg_catalog.unnest($8::text[]), pg_catalog.unnest($9::text[]), \
                    pg_catalog.unnest($10::text[]), pg_catalog.unnest($11::text[]), \
                    pg_catalog.unnest($12::text[])) \
      AS input(place_geoid, state_fips, place_fips, place_ns, name, name_lsad, lsad, class_fp, \
               principal_city_indicator, mtfcc, functional_status) ON CONFLICT DO NOTHING";
const INSERT_LAND_FRACTIONS_SQL: &str = "INSERT INTO babylon_ref.h3_land_fraction \
    (ref_digest, product_code, cell_id, membership_origin, source_county_geoid, \
     land_fraction_ppm) \
    SELECT $1, 'h3_res7_land_mask', input.cell_id, 1, input.source_county_geoid, \
           input.land_fraction_ppm \
    FROM ROWS FROM (pg_catalog.unnest($2::bigint[]), pg_catalog.unnest($3::text[]), \
                    pg_catalog.unnest($4::integer[])) \
      AS input(cell_id, source_county_geoid, land_fraction_ppm) ON CONFLICT DO NOTHING";
const INSERT_POPULATION_SQL: &str = "INSERT INTO babylon_ref.h3_population_count \
    (ref_digest, product_code, cell_id, membership_origin, population_count) \
    SELECT $1, 'h3_res7_population', input.cell_id, 1, input.measure_count \
    FROM ROWS FROM (pg_catalog.unnest($2::bigint[]), pg_catalog.unnest($3::bigint[])) \
      AS input(cell_id, measure_count) ON CONFLICT DO NOTHING";
const INSERT_WORKPLACE_SQL: &str = "INSERT INTO babylon_ref.h3_workplace_count \
    (ref_digest, product_code, cell_id, membership_origin, workplace_count) \
    SELECT $1, 'h3_res7_workplace', input.cell_id, 1, input.measure_count \
    FROM ROWS FROM (pg_catalog.unnest($2::bigint[]), pg_catalog.unnest($3::bigint[])) \
      AS input(cell_id, measure_count) ON CONFLICT DO NOTHING";
const INSERT_COUNTY_LAND_SQL: &str = "INSERT INTO babylon_ref.county_h3_land_area \
    (ref_digest, product_code, cell_id, membership_origin, county_geoid, land_area_m2) \
    SELECT $1, 'census_county_h3_land_overlap_mi_2023', input.cell_id, 1, \
           input.county_geoid, input.land_area_m2 \
    FROM ROWS FROM (pg_catalog.unnest($2::bigint[]), pg_catalog.unnest($3::text[]), \
                    pg_catalog.unnest($4::bigint[])) \
      AS input(cell_id, county_geoid, land_area_m2) ON CONFLICT DO NOTHING";
const INSERT_COUNTY_PLACE_LAND_SQL: &str = "INSERT INTO babylon_ref.county_place_h3_land_area \
    (ref_digest, product_code, cell_id, membership_origin, county_geoid, place_geoid, \
     place_land_area_m2, cell_mi_land_area_m2, place_land_area_share_ppb) \
    SELECT $1, 'census_county_place_h3_land_overlap_mi_2023', input.cell_id, 1, \
           input.county_geoid, input.place_geoid, input.place_land_area_m2, \
           input.cell_mi_land_area_m2, input.place_land_area_share_ppb \
    FROM ROWS FROM (pg_catalog.unnest($2::bigint[]), pg_catalog.unnest($3::text[]), \
                    pg_catalog.unnest($4::text[]), pg_catalog.unnest($5::bigint[]), \
                    pg_catalog.unnest($6::bigint[]), pg_catalog.unnest($7::integer[])) \
      AS input(cell_id, county_geoid, place_geoid, place_land_area_m2, \
               cell_mi_land_area_m2, place_land_area_share_ppb) ON CONFLICT DO NOTHING";

const READ_COUNTIES_SQL: &str =
    "SELECT county_id, county_geoid, state_id, county_fips, county_name \
    FROM babylon_ref.county_identity WHERE ref_digest = $1 \
    ORDER BY county_geoid LIMIT $2";
const READ_PLACES_SQL: &str = "SELECT place_geoid, state_fips, place_fips, place_ns, name, \
    name_lsad, lsad, class_fp, principal_city_indicator, mtfcc, functional_status \
    FROM babylon_ref.place_identity WHERE ref_digest = $1 ORDER BY place_geoid LIMIT $2";
const READ_LAND_FRACTIONS_SQL: &str = "SELECT cell_id, source_county_geoid, land_fraction_ppm \
    FROM babylon_ref.h3_land_fraction WHERE ref_digest = $1 ORDER BY cell_id LIMIT $2";
const READ_POPULATION_SQL: &str = "SELECT cell_id, population_count \
    FROM babylon_ref.h3_population_count WHERE ref_digest = $1 ORDER BY cell_id LIMIT $2";
const READ_WORKPLACE_SQL: &str = "SELECT cell_id, workplace_count \
    FROM babylon_ref.h3_workplace_count WHERE ref_digest = $1 ORDER BY cell_id LIMIT $2";
const READ_COUNTY_LAND_SQL: &str = "SELECT cell_id, county_geoid, land_area_m2 \
    FROM babylon_ref.county_h3_land_area WHERE ref_digest = $1 \
    ORDER BY cell_id, county_geoid LIMIT $2";
const READ_COUNTY_PLACE_LAND_SQL: &str = "SELECT cell_id, county_geoid, place_geoid, \
    place_land_area_m2, cell_mi_land_area_m2, place_land_area_share_ppb \
    FROM babylon_ref.county_place_h3_land_area WHERE ref_digest = $1 \
    ORDER BY cell_id, county_geoid, place_geoid LIMIT $2";

/// Durable result of one exact bundle installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialReferenceInstallDisposition {
    Installed,
    AlreadyPresent,
    ReconciledAfterAmbiguousCommit,
}

/// Closed relation labels for bounded operations and conflicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialReferenceRelation {
    Products,
    Counties,
    Places,
    LandFractions,
    PopulationCounts,
    WorkplaceCounts,
    CountyLandAreas,
    CountyPlaceLandAreas,
}

/// Credential-safe installer operation labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialReferenceInstallOperation {
    Connect,
    SetSessionSettings,
    VerifySessionSettings,
    Read(SpatialReferenceRelation),
    BeginTransaction,
    SetTransactionSettings,
    VerifyTransactionSettings,
    Insert(SpatialReferenceRelation),
    CommitTransaction,
    RollbackTransaction,
}

/// Closed PER-278 installer failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpatialReferenceInstallError {
    Bundle(SpatialReferenceProductsError),
    ConnectionTarget(CatalogError),
    Lock(CatalogError),
    CurrentSchema(CurrentSchemaError),
    Database {
        operation: SpatialReferenceInstallOperation,
        diagnostic: Option<PostgresDiagnostic>,
    },
    Decode {
        operation: SpatialReferenceInstallOperation,
    },
    Bounds {
        relation: SpatialReferenceRelation,
        actual: usize,
        max: usize,
    },
    CellIdentity,
    NumericRange {
        relation: SpatialReferenceRelation,
    },
    Conflict {
        relation: SpatialReferenceRelation,
    },
    AmbiguousCommitUnresolved {
        attempts: usize,
    },
    AmbiguousCommitAndReconciliation {
        attempts: usize,
        reconciliation: Box<SpatialReferenceInstallError>,
    },
    Unlock(CatalogError),
    FailureAndCleanup {
        primary: Box<SpatialReferenceInstallError>,
        cleanup: Box<SpatialReferenceInstallError>,
    },
    FailureAndRollback {
        primary: Box<SpatialReferenceInstallError>,
        rollback: Box<SpatialReferenceInstallError>,
    },
}

impl std::fmt::Display for SpatialReferenceInstallError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "spatial reference installation refused: {self:?}"
        )
    }
}

impl std::error::Error for SpatialReferenceInstallError {}

/// Exact durable installation receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpatialReferenceInstallReport {
    disposition: SpatialReferenceInstallDisposition,
    ref_digest: RefDigest,
    product_count: usize,
    data_row_count: usize,
    commit_attempts: usize,
}

impl SpatialReferenceInstallReport {
    #[must_use]
    pub const fn disposition(&self) -> SpatialReferenceInstallDisposition {
        self.disposition
    }

    #[must_use]
    pub const fn ref_digest(&self) -> RefDigest {
        self.ref_digest
    }

    #[must_use]
    pub const fn product_count(&self) -> usize {
        self.product_count
    }

    #[must_use]
    pub const fn data_row_count(&self) -> usize {
        self.data_row_count
    }

    #[must_use]
    pub const fn commit_attempts(&self) -> usize {
        self.commit_attempts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstallPresence {
    Absent,
    Exact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommitAttempt {
    Committed,
    Ambiguous,
}

/// Install the exact checked Michigan bundle into the exact current schema epoch.
///
/// This maintenance-only entry point never requests runtime writer authority and
/// never advances the schema epoch for its caller.
///
/// # Errors
/// Refuses target, authority, epoch, transaction, equivalence, bounds, or
/// ambiguous-commit failures before reporting durable success.
pub fn install_michigan_spatial_reference_products(
    config: &Config,
    cohort: &H3ReferenceCohort,
) -> Result<SpatialReferenceInstallReport, SpatialReferenceInstallError> {
    let mut attempt = attempt_install_transaction;
    install_michigan_spatial_reference_products_using(config, cohort, &mut attempt)
}

fn install_michigan_spatial_reference_products_using<Attempt>(
    config: &Config,
    cohort: &H3ReferenceCohort,
    attempt: &mut Attempt,
) -> Result<SpatialReferenceInstallReport, SpatialReferenceInstallError>
where
    Attempt: FnMut(
        &mut Client,
        &SpatialReferenceProducts,
    ) -> Result<CommitAttempt, SpatialReferenceInstallError>,
{
    let bundle = michigan_spatial_reference_products(cohort)
        .map_err(SpatialReferenceInstallError::Bundle)?;
    validate_connection_target(config).map_err(SpatialReferenceInstallError::ConnectionTarget)?;
    let bounded = bounded_config(config);
    let mut session = LockedInstallSession::connect(&bounded)?;
    let primary = install_under_lock(&bounded, &mut session, &bundle, attempt);
    session.finish(primary)
}

fn install_under_lock<Attempt>(
    config: &Config,
    session: &mut LockedInstallSession,
    bundle: &SpatialReferenceProducts,
    attempt_install: &mut Attempt,
) -> Result<SpatialReferenceInstallReport, SpatialReferenceInstallError>
where
    Attempt: FnMut(
        &mut Client,
        &SpatialReferenceProducts,
    ) -> Result<CommitAttempt, SpatialReferenceInstallError>,
{
    require_schema(session.client())?;
    prepare_session(session.client())?;
    if inspect_presence(session.client(), bundle)? == InstallPresence::Exact {
        return Ok(report(
            bundle,
            SpatialReferenceInstallDisposition::AlreadyPresent,
            0,
        ));
    }
    for attempt in 1..=MAX_COMMIT_ATTEMPTS {
        match attempt_install(session.client(), bundle)? {
            CommitAttempt::Committed => {
                return Ok(report(
                    bundle,
                    SpatialReferenceInstallDisposition::Installed,
                    attempt,
                ));
            }
            CommitAttempt::Ambiguous => {
                let reconciled = reconcile(config, session, bundle).map_err(|reconciliation| {
                    SpatialReferenceInstallError::AmbiguousCommitAndReconciliation {
                        attempts: attempt,
                        reconciliation: Box::new(reconciliation),
                    }
                })?;
                if reconciled == InstallPresence::Exact {
                    return Ok(report(
                        bundle,
                        SpatialReferenceInstallDisposition::ReconciledAfterAmbiguousCommit,
                        attempt,
                    ));
                }
            }
        }
    }
    Err(SpatialReferenceInstallError::AmbiguousCommitUnresolved {
        attempts: MAX_COMMIT_ATTEMPTS,
    })
}

fn report(
    bundle: &SpatialReferenceProducts,
    disposition: SpatialReferenceInstallDisposition,
    commit_attempts: usize,
) -> SpatialReferenceInstallReport {
    SpatialReferenceInstallReport {
        disposition,
        ref_digest: bundle.ref_digest(),
        product_count: PRODUCT_COUNT,
        data_row_count: TOTAL_DATA_ROWS,
        commit_attempts,
    }
}

fn reconcile(
    config: &Config,
    session: &mut LockedInstallSession,
    bundle: &SpatialReferenceProducts,
) -> Result<InstallPresence, SpatialReferenceInstallError> {
    session.reconnect(config)?;
    require_schema(session.client())?;
    prepare_session(session.client())?;
    inspect_presence(session.client(), bundle)
}

struct LockedInstallSession {
    client: Option<Client>,
}

impl LockedInstallSession {
    fn connect(config: &Config) -> Result<Self, SpatialReferenceInstallError> {
        let mut client = config.connect(NoTls).map_err(|error| {
            postgres_database_error(SpatialReferenceInstallOperation::Connect, &error)
        })?;
        acquire_lock(&mut client).map_err(SpatialReferenceInstallError::Lock)?;
        Ok(Self {
            client: Some(client),
        })
    }

    fn client(&mut self) -> &mut Client {
        self.client
            .as_mut()
            .expect("locked installer session always has one client")
    }

    fn reconnect(&mut self, config: &Config) -> Result<(), SpatialReferenceInstallError> {
        self.client.take();
        *self = Self::connect(config)?;
        Ok(())
    }

    fn finish<T>(
        mut self,
        primary: Result<T, SpatialReferenceInstallError>,
    ) -> Result<T, SpatialReferenceInstallError> {
        let cleanup = self.client.as_mut().map_or(Ok(()), |client| {
            release_lock(client)
                .map_err(SpatialReferenceInstallError::Unlock)
                .map_err(Box::new)
        });
        match (primary, cleanup) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), Ok(())) => Err(error),
            (Ok(_), Err(cleanup)) => Err(*cleanup),
            (Err(primary), Err(cleanup)) => Err(SpatialReferenceInstallError::FailureAndCleanup {
                primary: Box::new(primary),
                cleanup,
            }),
        }
    }
}

fn require_schema(client: &mut Client) -> Result<(), SpatialReferenceInstallError> {
    require_current_schema(client)
        .map(|_| ())
        .map_err(SpatialReferenceInstallError::CurrentSchema)
}

fn prepare_session(client: &mut Client) -> Result<(), SpatialReferenceInstallError> {
    client
        .batch_execute(SESSION_SETTINGS_SQL)
        .map_err(|error| {
            postgres_database_error(SpatialReferenceInstallOperation::SetSessionSettings, &error)
        })?;
    let operation = SpatialReferenceInstallOperation::VerifySessionSettings;
    let row = client
        .query_one(SESSION_SETTINGS_QUERY, &[])
        .map_err(|error| postgres_database_error(operation, &error))?;
    let expected = ["on", "pg_catalog", "2min", "5s", "5s", "off", "off", "off"];
    for (index, wanted) in expected.iter().enumerate() {
        let actual: String = decode(&row, index, operation)?;
        if actual != *wanted {
            return Err(database_error(operation));
        }
    }
    Ok(())
}

fn attempt_install_transaction(
    client: &mut Client,
    bundle: &SpatialReferenceProducts,
) -> Result<CommitAttempt, SpatialReferenceInstallError> {
    let transaction = prepare_install_transaction(client, bundle)?;
    match transaction.commit() {
        Ok(()) => Ok(CommitAttempt::Committed),
        Err(error) if error.as_db_error().is_some() => Err(postgres_database_error(
            SpatialReferenceInstallOperation::CommitTransaction,
            &error,
        )),
        Err(_) => Ok(CommitAttempt::Ambiguous),
    }
}

fn prepare_install_transaction<'client>(
    client: &'client mut Client,
    bundle: &SpatialReferenceProducts,
) -> Result<Transaction<'client>, SpatialReferenceInstallError> {
    let mut transaction = client
        .build_transaction()
        .isolation_level(IsolationLevel::Serializable)
        .read_only(false)
        .start()
        .map_err(|error| {
            postgres_database_error(SpatialReferenceInstallOperation::BeginTransaction, &error)
        })?;
    let installed = install_and_verify(&mut transaction, bundle);
    if let Err(primary) = installed {
        return rollback_preserving(transaction, primary);
    }
    Ok(transaction)
}

fn rollback_preserving<T>(
    transaction: Transaction<'_>,
    primary: SpatialReferenceInstallError,
) -> Result<T, SpatialReferenceInstallError> {
    match transaction.rollback() {
        Ok(()) => Err(primary),
        Err(error) => Err(SpatialReferenceInstallError::FailureAndRollback {
            primary: Box::new(primary),
            rollback: Box::new(postgres_database_error(
                SpatialReferenceInstallOperation::RollbackTransaction,
                &error,
            )),
        }),
    }
}

fn install_and_verify(
    transaction: &mut Transaction<'_>,
    bundle: &SpatialReferenceProducts,
) -> Result<(), SpatialReferenceInstallError> {
    prepare_transaction(transaction)?;
    insert_products(transaction, bundle)?;
    insert_counties(transaction, bundle)?;
    insert_places(transaction, bundle)?;
    insert_land_fractions(transaction, bundle)?;
    insert_counts(
        transaction,
        bundle,
        SpatialReferenceRelation::PopulationCounts,
    )?;
    insert_counts(
        transaction,
        bundle,
        SpatialReferenceRelation::WorkplaceCounts,
    )?;
    insert_county_land(transaction, bundle)?;
    insert_county_place_land(transaction, bundle)?;
    match inspect_presence(transaction, bundle)? {
        InstallPresence::Exact => Ok(()),
        InstallPresence::Absent => Err(conflict(SpatialReferenceRelation::Products)),
    }
}

fn prepare_transaction(
    transaction: &mut Transaction<'_>,
) -> Result<(), SpatialReferenceInstallError> {
    transaction
        .batch_execute(WRITE_LOCAL_SETTINGS_SQL)
        .map_err(|error| {
            postgres_database_error(
                SpatialReferenceInstallOperation::SetTransactionSettings,
                &error,
            )
        })?;
    let operation = SpatialReferenceInstallOperation::VerifyTransactionSettings;
    let row = transaction
        .query_one(WRITE_SETTINGS_SQL, &[])
        .map_err(|error| postgres_database_error(operation, &error))?;
    let expected = [
        "serializable",
        "off",
        "pg_catalog",
        "on",
        "2min",
        "5s",
        "5s",
        "off",
        "off",
        "off",
    ];
    for (index, wanted) in expected.iter().enumerate() {
        let actual: String = decode(&row, index, operation)?;
        if actual != *wanted {
            return Err(database_error(operation));
        }
    }
    Ok(())
}

fn insert_products(
    transaction: &mut Transaction<'_>,
    bundle: &SpatialReferenceProducts,
) -> Result<(), SpatialReferenceInstallError> {
    let operation = SpatialReferenceInstallOperation::Insert(SpatialReferenceRelation::Products);
    let ref_digest = bundle.ref_digest();
    for product in bundle.products().iter().take(PRODUCT_COUNT) {
        let artifact = product.artifact_sha256();
        let semantic = product
            .semantic_sha256()
            .map(|digest| digest.as_bytes().to_vec());
        let row_count = i64::try_from(product.row_count())
            .map_err(|_| numeric_range(SpatialReferenceRelation::Products))?;
        transaction
            .execute(
                INSERT_PRODUCT_SQL,
                &[
                    &ref_digest.as_bytes().as_slice(),
                    &product.code(),
                    &artifact.as_bytes().as_slice(),
                    &semantic,
                    &row_count,
                    &product.evidence_class().as_str(),
                    &product.measure_unit(),
                    &product.denominator(),
                ],
            )
            .map_err(|error| postgres_database_error(operation, &error))?;
    }
    Ok(())
}

fn insert_counties(
    transaction: &mut Transaction<'_>,
    bundle: &SpatialReferenceProducts,
) -> Result<(), SpatialReferenceInstallError> {
    let relation = SpatialReferenceRelation::Counties;
    let operation = SpatialReferenceInstallOperation::Insert(relation);
    let ref_digest = bundle.ref_digest();
    for batch in bundle.counties().chunks(INSTALL_BATCH_ROWS) {
        let county_ids = batch
            .iter()
            .map(|row| i64::from(row.county_id()))
            .collect::<Vec<_>>();
        let geoids = batch
            .iter()
            .map(CountyIdentityRow::county_geoid)
            .collect::<Vec<_>>();
        let state_ids = batch
            .iter()
            .map(|row| i32::from(row.state_id()))
            .collect::<Vec<_>>();
        let county_fips = batch
            .iter()
            .map(CountyIdentityRow::county_fips)
            .collect::<Vec<_>>();
        let names = batch
            .iter()
            .map(CountyIdentityRow::county_name)
            .collect::<Vec<_>>();
        transaction
            .execute(
                INSERT_COUNTIES_SQL,
                &[
                    &ref_digest.as_bytes().as_slice(),
                    &county_ids,
                    &geoids,
                    &state_ids,
                    &county_fips,
                    &names,
                ],
            )
            .map_err(|error| postgres_database_error(operation, &error))?;
    }
    Ok(())
}

fn insert_places(
    transaction: &mut Transaction<'_>,
    bundle: &SpatialReferenceProducts,
) -> Result<(), SpatialReferenceInstallError> {
    let operation = SpatialReferenceInstallOperation::Insert(SpatialReferenceRelation::Places);
    let ref_digest = bundle.ref_digest();
    for batch in bundle.places().chunks(INSTALL_BATCH_ROWS) {
        let place_geoids = collect_place_text(batch, PlaceIdentityRow::place_geoid);
        let state_fips = collect_place_text(batch, PlaceIdentityRow::state_fips);
        let place_fips = collect_place_text(batch, PlaceIdentityRow::place_fips);
        let place_ns = collect_place_text(batch, PlaceIdentityRow::place_ns);
        let names = collect_place_text(batch, PlaceIdentityRow::name);
        let names_lsad = collect_place_text(batch, PlaceIdentityRow::name_lsad);
        let lsad = collect_place_text(batch, PlaceIdentityRow::lsad);
        let class_fp = collect_place_text(batch, PlaceIdentityRow::class_fp);
        let principal_city = collect_place_text(batch, PlaceIdentityRow::principal_city_indicator);
        let mtfcc = collect_place_text(batch, PlaceIdentityRow::mtfcc);
        let status = collect_place_text(batch, PlaceIdentityRow::functional_status);
        transaction
            .execute(
                INSERT_PLACES_SQL,
                &[
                    &ref_digest.as_bytes().as_slice(),
                    &place_geoids,
                    &state_fips,
                    &place_fips,
                    &place_ns,
                    &names,
                    &names_lsad,
                    &lsad,
                    &class_fp,
                    &principal_city,
                    &mtfcc,
                    &status,
                ],
            )
            .map_err(|error| postgres_database_error(operation, &error))?;
    }
    Ok(())
}

fn collect_place_text(
    rows: &[PlaceIdentityRow],
    getter: fn(&PlaceIdentityRow) -> &'static str,
) -> Vec<&'static str> {
    rows.iter().map(getter).collect()
}

fn insert_land_fractions(
    transaction: &mut Transaction<'_>,
    bundle: &SpatialReferenceProducts,
) -> Result<(), SpatialReferenceInstallError> {
    let relation = SpatialReferenceRelation::LandFractions;
    let operation = SpatialReferenceInstallOperation::Insert(relation);
    let ref_digest = bundle.ref_digest();
    for batch in bundle.land_fractions().chunks(INSTALL_BATCH_ROWS) {
        let cells = batch_cells(batch.iter().map(H3LandFractionRow::cell_id), relation)?;
        let counties = batch
            .iter()
            .map(H3LandFractionRow::source_county_geoid)
            .collect::<Vec<_>>();
        let fractions = batch
            .iter()
            .map(|row| i32::try_from(row.parts_per_million()).map_err(|_| numeric_range(relation)))
            .collect::<Result<Vec<_>, _>>()?;
        transaction
            .execute(
                INSERT_LAND_FRACTIONS_SQL,
                &[
                    &ref_digest.as_bytes().as_slice(),
                    &cells,
                    &counties,
                    &fractions,
                ],
            )
            .map_err(|error| postgres_database_error(operation, &error))?;
    }
    Ok(())
}

fn insert_counts(
    transaction: &mut Transaction<'_>,
    bundle: &SpatialReferenceProducts,
    relation: SpatialReferenceRelation,
) -> Result<(), SpatialReferenceInstallError> {
    let (rows, sql): (&[H3CountRow], &str) = match relation {
        SpatialReferenceRelation::PopulationCounts => {
            (bundle.population_counts(), INSERT_POPULATION_SQL)
        }
        SpatialReferenceRelation::WorkplaceCounts => {
            (bundle.workplace_counts(), INSERT_WORKPLACE_SQL)
        }
        _ => unreachable!("count insertion accepts only count relations"),
    };
    let operation = SpatialReferenceInstallOperation::Insert(relation);
    let ref_digest = bundle.ref_digest();
    for batch in rows.chunks(INSTALL_BATCH_ROWS) {
        let cells = batch_cells(batch.iter().map(H3CountRow::cell_id), relation)?;
        let counts = batch
            .iter()
            .map(|row| i64::try_from(row.count()).map_err(|_| numeric_range(relation)))
            .collect::<Result<Vec<_>, _>>()?;
        transaction
            .execute(sql, &[&ref_digest.as_bytes().as_slice(), &cells, &counts])
            .map_err(|error| postgres_database_error(operation, &error))?;
    }
    Ok(())
}

fn insert_county_land(
    transaction: &mut Transaction<'_>,
    bundle: &SpatialReferenceProducts,
) -> Result<(), SpatialReferenceInstallError> {
    let relation = SpatialReferenceRelation::CountyLandAreas;
    let operation = SpatialReferenceInstallOperation::Insert(relation);
    let ref_digest = bundle.ref_digest();
    for batch in bundle.county_land_areas().chunks(INSTALL_BATCH_ROWS) {
        let cells = batch_cells(batch.iter().map(CountyH3LandAreaRow::cell_id), relation)?;
        let counties = batch
            .iter()
            .map(CountyH3LandAreaRow::county_geoid)
            .collect::<Vec<_>>();
        let areas = batch
            .iter()
            .map(|row| i64::try_from(row.land_area_m2()).map_err(|_| numeric_range(relation)))
            .collect::<Result<Vec<_>, _>>()?;
        transaction
            .execute(
                INSERT_COUNTY_LAND_SQL,
                &[&ref_digest.as_bytes().as_slice(), &cells, &counties, &areas],
            )
            .map_err(|error| postgres_database_error(operation, &error))?;
    }
    Ok(())
}

fn insert_county_place_land(
    transaction: &mut Transaction<'_>,
    bundle: &SpatialReferenceProducts,
) -> Result<(), SpatialReferenceInstallError> {
    let relation = SpatialReferenceRelation::CountyPlaceLandAreas;
    let operation = SpatialReferenceInstallOperation::Insert(relation);
    let ref_digest = bundle.ref_digest();
    for batch in bundle.county_place_land_areas().chunks(INSTALL_BATCH_ROWS) {
        let cells = batch_cells(
            batch.iter().map(CountyPlaceH3LandAreaRow::cell_id),
            relation,
        )?;
        let counties = batch
            .iter()
            .map(CountyPlaceH3LandAreaRow::county_geoid)
            .collect::<Vec<_>>();
        let places = batch
            .iter()
            .map(CountyPlaceH3LandAreaRow::place_geoid)
            .collect::<Vec<_>>();
        let areas = batch
            .iter()
            .map(|row| i64::try_from(row.place_land_area_m2()).map_err(|_| numeric_range(relation)))
            .collect::<Result<Vec<_>, _>>()?;
        let denominators = batch
            .iter()
            .map(|row| {
                i64::try_from(row.cell_mi_land_area_m2()).map_err(|_| numeric_range(relation))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let shares = batch
            .iter()
            .map(|row| {
                i32::try_from(row.place_land_area_share_ppb()).map_err(|_| numeric_range(relation))
            })
            .collect::<Result<Vec<_>, _>>()?;
        transaction
            .execute(
                INSERT_COUNTY_PLACE_LAND_SQL,
                &[
                    &ref_digest.as_bytes().as_slice(),
                    &cells,
                    &counties,
                    &places,
                    &areas,
                    &denominators,
                    &shares,
                ],
            )
            .map_err(|error| postgres_database_error(operation, &error))?;
    }
    Ok(())
}

fn batch_cells(
    cells: impl Iterator<Item = H3CellId>,
    relation: SpatialReferenceRelation,
) -> Result<Vec<i64>, SpatialReferenceInstallError> {
    cells
        .map(|cell| i64::try_from(cell).map_err(|_| numeric_range(relation)))
        .collect()
}

fn inspect_presence<ClientType: GenericClient>(
    client: &mut ClientType,
    bundle: &SpatialReferenceProducts,
) -> Result<InstallPresence, SpatialReferenceInstallError> {
    let relation = SpatialReferenceRelation::Products;
    let operation = SpatialReferenceInstallOperation::Read(relation);
    let ref_digest = bundle.ref_digest();
    let limit = query_limit(PRODUCT_COUNT, relation)?;
    let product_codes = bundle
        .products()
        .iter()
        .map(ReferenceProduct::code)
        .collect::<Vec<_>>();
    let rows = client
        .query(
            READ_PRODUCTS_SQL,
            &[&ref_digest.as_bytes().as_slice(), &product_codes, &limit],
        )
        .map_err(|error| postgres_database_error(operation, &error))?;
    if rows.is_empty() {
        return Ok(InstallPresence::Absent);
    }
    require_row_count(rows.len(), PRODUCT_COUNT, relation)?;
    for (row, expected) in rows.iter().zip(bundle.products()) {
        verify_product(row, expected)?;
    }
    verify_counties(client, bundle)?;
    verify_places(client, bundle)?;
    verify_land_fractions(client, bundle)?;
    verify_counts(client, bundle, SpatialReferenceRelation::PopulationCounts)?;
    verify_counts(client, bundle, SpatialReferenceRelation::WorkplaceCounts)?;
    verify_county_land(client, bundle)?;
    verify_county_place_land(client, bundle)?;
    Ok(InstallPresence::Exact)
}

fn verify_product(
    row: &Row,
    expected: &ReferenceProduct,
) -> Result<(), SpatialReferenceInstallError> {
    let relation = SpatialReferenceRelation::Products;
    let operation = SpatialReferenceInstallOperation::Read(relation);
    let code: String = decode(row, 0, operation)?;
    let artifact: Vec<u8> = decode(row, 1, operation)?;
    let semantic: Option<Vec<u8>> = decode(row, 2, operation)?;
    let row_count: i64 = decode(row, 3, operation)?;
    let evidence: String = decode(row, 4, operation)?;
    let unit: Option<String> = decode(row, 5, operation)?;
    let denominator: Option<String> = decode(row, 6, operation)?;
    let expected_semantic = expected
        .semantic_sha256()
        .map(|digest| digest.as_bytes().to_vec());
    let exact = code == expected.code()
        && artifact.as_slice() == expected.artifact_sha256().as_bytes()
        && semantic == expected_semantic
        && u64::try_from(row_count).ok() == Some(expected.row_count())
        && evidence == expected.evidence_class().as_str()
        && unit.as_deref() == Some(expected.measure_unit())
        && denominator.as_deref() == expected.denominator();
    if exact {
        Ok(())
    } else {
        Err(conflict(relation))
    }
}

fn verify_counties<ClientType: GenericClient>(
    client: &mut ClientType,
    bundle: &SpatialReferenceProducts,
) -> Result<(), SpatialReferenceInstallError> {
    let relation = SpatialReferenceRelation::Counties;
    let rows = read_rows(
        client,
        READ_COUNTIES_SQL,
        bundle,
        bundle.counties().len(),
        relation,
    )?;
    for (row, expected) in rows.iter().zip(bundle.counties()) {
        let operation = SpatialReferenceInstallOperation::Read(relation);
        let county_id: i64 = decode(row, 0, operation)?;
        let geoid: String = decode(row, 1, operation)?;
        let state_id: i32 = decode(row, 2, operation)?;
        let county_fips: String = decode(row, 3, operation)?;
        let name: String = decode(row, 4, operation)?;
        if u32::try_from(county_id).ok() != Some(expected.county_id())
            || geoid != expected.county_geoid()
            || u16::try_from(state_id).ok() != Some(expected.state_id())
            || county_fips != expected.county_fips()
            || name != expected.county_name()
        {
            return Err(conflict(relation));
        }
    }
    Ok(())
}

fn verify_places<ClientType: GenericClient>(
    client: &mut ClientType,
    bundle: &SpatialReferenceProducts,
) -> Result<(), SpatialReferenceInstallError> {
    let relation = SpatialReferenceRelation::Places;
    let rows = read_rows(
        client,
        READ_PLACES_SQL,
        bundle,
        bundle.places().len(),
        relation,
    )?;
    for (row, expected) in rows.iter().zip(bundle.places()) {
        let operation = SpatialReferenceInstallOperation::Read(relation);
        let actual = (0..11)
            .map(|index| decode::<String>(row, index, operation))
            .collect::<Result<Vec<_>, _>>()?;
        let wanted = [
            expected.place_geoid(),
            expected.state_fips(),
            expected.place_fips(),
            expected.place_ns(),
            expected.name(),
            expected.name_lsad(),
            expected.lsad(),
            expected.class_fp(),
            expected.principal_city_indicator(),
            expected.mtfcc(),
            expected.functional_status(),
        ];
        if actual.iter().map(String::as_str).ne(wanted) {
            return Err(conflict(relation));
        }
    }
    Ok(())
}

fn verify_land_fractions<ClientType: GenericClient>(
    client: &mut ClientType,
    bundle: &SpatialReferenceProducts,
) -> Result<(), SpatialReferenceInstallError> {
    let relation = SpatialReferenceRelation::LandFractions;
    let rows = read_rows(
        client,
        READ_LAND_FRACTIONS_SQL,
        bundle,
        bundle.land_fractions().len(),
        relation,
    )?;
    for (row, expected) in rows.iter().zip(bundle.land_fractions()) {
        let operation = SpatialReferenceInstallOperation::Read(relation);
        let cell: i64 = decode(row, 0, operation)?;
        let county: String = decode(row, 1, operation)?;
        let fraction: i32 = decode(row, 2, operation)?;
        if decode_cell(cell)? != expected.cell_id()
            || county != expected.source_county_geoid()
            || u32::try_from(fraction).ok() != Some(expected.parts_per_million())
        {
            return Err(conflict(relation));
        }
    }
    Ok(())
}

fn verify_counts<ClientType: GenericClient>(
    client: &mut ClientType,
    bundle: &SpatialReferenceProducts,
    relation: SpatialReferenceRelation,
) -> Result<(), SpatialReferenceInstallError> {
    let (sql, expected): (&str, &[H3CountRow]) = match relation {
        SpatialReferenceRelation::PopulationCounts => {
            (READ_POPULATION_SQL, bundle.population_counts())
        }
        SpatialReferenceRelation::WorkplaceCounts => {
            (READ_WORKPLACE_SQL, bundle.workplace_counts())
        }
        _ => unreachable!("count verification accepts only count relations"),
    };
    let rows = read_rows(client, sql, bundle, expected.len(), relation)?;
    for (row, wanted) in rows.iter().zip(expected) {
        let operation = SpatialReferenceInstallOperation::Read(relation);
        let cell: i64 = decode(row, 0, operation)?;
        let count: i64 = decode(row, 1, operation)?;
        if decode_cell(cell)? != wanted.cell_id()
            || u64::try_from(count).ok() != Some(wanted.count())
        {
            return Err(conflict(relation));
        }
    }
    Ok(())
}

fn verify_county_land<ClientType: GenericClient>(
    client: &mut ClientType,
    bundle: &SpatialReferenceProducts,
) -> Result<(), SpatialReferenceInstallError> {
    let relation = SpatialReferenceRelation::CountyLandAreas;
    let rows = read_rows(
        client,
        READ_COUNTY_LAND_SQL,
        bundle,
        bundle.county_land_areas().len(),
        relation,
    )?;
    for (row, expected) in rows.iter().zip(bundle.county_land_areas()) {
        let operation = SpatialReferenceInstallOperation::Read(relation);
        let cell: i64 = decode(row, 0, operation)?;
        let county: String = decode(row, 1, operation)?;
        let area: i64 = decode(row, 2, operation)?;
        if decode_cell(cell)? != expected.cell_id()
            || county != expected.county_geoid()
            || u64::try_from(area).ok() != Some(expected.land_area_m2())
        {
            return Err(conflict(relation));
        }
    }
    Ok(())
}

fn verify_county_place_land<ClientType: GenericClient>(
    client: &mut ClientType,
    bundle: &SpatialReferenceProducts,
) -> Result<(), SpatialReferenceInstallError> {
    let relation = SpatialReferenceRelation::CountyPlaceLandAreas;
    let rows = read_rows(
        client,
        READ_COUNTY_PLACE_LAND_SQL,
        bundle,
        bundle.county_place_land_areas().len(),
        relation,
    )?;
    for (row, expected) in rows.iter().zip(bundle.county_place_land_areas()) {
        let operation = SpatialReferenceInstallOperation::Read(relation);
        let cell: i64 = decode(row, 0, operation)?;
        let county: String = decode(row, 1, operation)?;
        let place: String = decode(row, 2, operation)?;
        let area: i64 = decode(row, 3, operation)?;
        let denominator: i64 = decode(row, 4, operation)?;
        let share: i32 = decode(row, 5, operation)?;
        if decode_cell(cell)? != expected.cell_id()
            || county != expected.county_geoid()
            || place != expected.place_geoid()
            || u64::try_from(area).ok() != Some(expected.place_land_area_m2())
            || u64::try_from(denominator).ok() != Some(expected.cell_mi_land_area_m2())
            || u32::try_from(share).ok() != Some(expected.place_land_area_share_ppb())
        {
            return Err(conflict(relation));
        }
    }
    Ok(())
}

fn read_rows<ClientType: GenericClient>(
    client: &mut ClientType,
    sql: &str,
    bundle: &SpatialReferenceProducts,
    expected: usize,
    relation: SpatialReferenceRelation,
) -> Result<Vec<Row>, SpatialReferenceInstallError> {
    let operation = SpatialReferenceInstallOperation::Read(relation);
    let limit = query_limit(expected, relation)?;
    let ref_digest = bundle.ref_digest();
    let rows = client
        .query(sql, &[&ref_digest.as_bytes().as_slice(), &limit])
        .map_err(|error| postgres_database_error(operation, &error))?;
    require_row_count(rows.len(), expected, relation)?;
    Ok(rows)
}

fn query_limit(
    expected: usize,
    relation: SpatialReferenceRelation,
) -> Result<i64, SpatialReferenceInstallError> {
    expected
        .checked_add(1)
        .and_then(|value| i64::try_from(value).ok())
        .ok_or(SpatialReferenceInstallError::Bounds {
            relation,
            actual: expected,
            max: expected,
        })
}

fn require_row_count(
    actual: usize,
    expected: usize,
    relation: SpatialReferenceRelation,
) -> Result<(), SpatialReferenceInstallError> {
    match actual.cmp(&expected) {
        std::cmp::Ordering::Equal => Ok(()),
        std::cmp::Ordering::Greater => Err(SpatialReferenceInstallError::Bounds {
            relation,
            actual,
            max: expected,
        }),
        std::cmp::Ordering::Less => Err(conflict(relation)),
    }
}

fn decode_cell(raw: i64) -> Result<H3CellId, SpatialReferenceInstallError> {
    H3CellId::try_from(raw).map_err(|_| SpatialReferenceInstallError::CellIdentity)
}

fn decode<T: postgres::types::FromSqlOwned>(
    row: &Row,
    index: usize,
    operation: SpatialReferenceInstallOperation,
) -> Result<T, SpatialReferenceInstallError> {
    row.try_get(index)
        .map_err(|_| SpatialReferenceInstallError::Decode { operation })
}

fn conflict(relation: SpatialReferenceRelation) -> SpatialReferenceInstallError {
    SpatialReferenceInstallError::Conflict { relation }
}

fn numeric_range(relation: SpatialReferenceRelation) -> SpatialReferenceInstallError {
    SpatialReferenceInstallError::NumericRange { relation }
}

fn database_error(operation: SpatialReferenceInstallOperation) -> SpatialReferenceInstallError {
    SpatialReferenceInstallError::Database {
        operation,
        diagnostic: None,
    }
}

fn postgres_database_error(
    operation: SpatialReferenceInstallOperation,
    error: &postgres::Error,
) -> SpatialReferenceInstallError {
    SpatialReferenceInstallError::Database {
        operation,
        diagnostic: Some(PostgresDiagnostic::capture(error)),
    }
}

#[cfg(test)]
#[path = "spatial_reference_installer_live_tests.rs"]
pub(crate) mod live_postgres_tests;
