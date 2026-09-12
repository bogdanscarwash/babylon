//! Transactional installation of the exact Michigan H3 reference bundle.

#[cfg(test)]
use std::io::Write as _;
#[cfg(test)]
use std::time::Duration;

use babylon_kernel::tick_content_hash::RefDigest;
use babylon_kernel::{content_digest::sha256_of, H3CellId, H3CellIdError};
use babylon_tick::h3_runtime::{MichiganDynamicHexFoundation, MichiganH3R8ChildParent};
use postgres::{Client, Config, GenericClient, IsolationLevel, NoTls, Row, Transaction};

use crate::current_schema::{bounded_config, require_current_schema, CurrentSchemaError};
use crate::h3_reference_cohort::MAX_H3_REFERENCE_CLOSURE_ROWS;
use crate::postgres_catalog::{
    acquire_lock, release_lock, validate_connection_target, CatalogError,
};
use crate::postgres_diagnostic::PostgresDiagnostic;
use crate::{
    h3_reference_cohort::build_representative_h3_cohort, h3_reference_cohort::H3ReferenceCellRow,
    h3_reference_cohort::H3ReferenceCohort, h3_reference_cohort::H3ReferenceCohortError,
    h3_reference_cohort::H3ReferenceCohortReceipt, h3_reference_cohort::H3ReferenceOrigin,
};

const H3_REFERENCE_COHORT_FORMAT_VERSION: i16 = 1;
const H3_REFERENCE_ARTIFACT_NAME: &str = "bridge_county_h3.parquet";
const H3_REFERENCE_ARTIFACT_MANIFEST_VERSION: &str = "2.0.0";
const H3_REFERENCE_SESSION_SETTINGS_SQL: &str = "SET statement_timeout TO '30000ms'";
const H3_REFERENCE_SESSION_SETTINGS_QUERY: &str = "SELECT \
    pg_catalog.current_setting('transaction_read_only'), \
    pg_catalog.current_setting('search_path'), \
    pg_catalog.current_setting('statement_timeout'), \
    pg_catalog.current_setting('lock_timeout'), \
    pg_catalog.current_setting('idle_in_transaction_session_timeout'), \
    pg_catalog.current_setting('quote_all_identifiers'), \
    pg_catalog.current_setting('jit'), \
    pg_catalog.current_setting('event_triggers')";
const H3_REFERENCE_INSTALL_BATCH_ROWS: usize = 1_024;
const MAX_H3_REFERENCE_INSTALL_BATCHES: usize =
    MAX_H3_REFERENCE_CLOSURE_ROWS.div_ceil(H3_REFERENCE_INSTALL_BATCH_ROWS);
const MICHIGAN_R8_CHILD_ROWS: usize = 319_004;
const MAX_MICHIGAN_R8_INSTALL_BATCHES: usize =
    MICHIGAN_R8_CHILD_ROWS.div_ceil(H3_REFERENCE_INSTALL_BATCH_ROWS);
const MICHIGAN_R8_PRODUCT_CODE: &str = "h3_res8_identity";
const MICHIGAN_R8_SECTION_DOMAIN: &[u8] = b"babylon.h3.reference-r8-child-parent.v1\0";
const MICHIGAN_REFERENCE_BUNDLE_DOMAIN: &[u8] = b"babylon.h3.reference-bundle-composite.v1\0";
const MAX_H3_REFERENCE_CARDINALITY_QUERY_ROWS: usize = MAX_H3_REFERENCE_CLOSURE_ROWS + 1;
const MAX_H3_REFERENCE_HEADER_ROWS: usize = 2;
const H3_REFERENCE_HEADER_QUERY_LIMIT: i64 = 3;
const H3_REFERENCE_PRODUCT_QUERY_LIMIT: i64 = 2;
const MAX_H3_REFERENCE_INSTALL_COMMIT_ATTEMPTS: usize = 2;

const READ_HEADER_SQL: &str = "SELECT ref_digest, format_version, artifact_name, \
    artifact_manifest_version, artifact_digest, source_digest, source_r5_digest, \
    source_r7_digest, closure_digest, membership_digest, direct_cell_count, \
    derived_ancestor_count, closure_cell_count \
    FROM babylon_ref.h3_reference_cohort \
    WHERE ref_digest = $1 OR (format_version = $2 AND artifact_digest = $3) \
    ORDER BY ref_digest LIMIT $4";
const READ_MEMBERSHIP_SQL: &str = "SELECT membership.cell_id, membership.origin \
    FROM babylon_ref.h3_reference_membership AS membership \
    WHERE membership.ref_digest = $1 \
    ORDER BY membership.cell_id LIMIT $2";
const READ_CELLS_SQL: &str = "SELECT cell.cell_id, cell.resolution, \
    cell.immediate_parent, cell.ancestor_r4, cell.ancestor_r5, cell.ancestor_r6, \
    cell.ancestor_r7 FROM babylon_ref.h3_cell AS cell \
    WHERE cell.cell_id = ANY($1::bigint[]) \
    ORDER BY cell.cell_id LIMIT $2";
const READ_R8_PRODUCT_SQL: &str = "SELECT product_code, artifact_sha256, semantic_sha256, \
    row_count, evidence_class, measure_unit, denominator \
    FROM babylon_ref.reference_product \
    WHERE ref_digest = $1 AND product_code = $2 \
    ORDER BY product_code LIMIT $3";
const READ_MEMBERSHIP_CARDINALITY_SQL: &str = "SELECT pg_catalog.count(*) \
    FROM (SELECT 1 \
    FROM babylon_ref.h3_reference_membership AS membership \
    WHERE membership.ref_digest = $1 \
    LIMIT $2) AS bounded_membership";
const INSERT_CELLS_SQL: &str = "INSERT INTO babylon_ref.h3_cell \
    (cell_id, resolution, immediate_parent, ancestor_r4, ancestor_r5, ancestor_r6, ancestor_r7) \
    SELECT input.cell_id, input.resolution, input.immediate_parent, input.ancestor_r4, \
           input.ancestor_r5, input.ancestor_r6, input.ancestor_r7 \
    FROM ROWS FROM (pg_catalog.unnest($1::bigint[]), pg_catalog.unnest($2::smallint[]), \
                    pg_catalog.unnest($3::bigint[]), pg_catalog.unnest($4::bigint[]), \
                    pg_catalog.unnest($5::bigint[]), pg_catalog.unnest($6::bigint[]), \
                    pg_catalog.unnest($7::bigint[])) \
      AS input(cell_id, resolution, immediate_parent, ancestor_r4, ancestor_r5, \
               ancestor_r6, ancestor_r7) ON CONFLICT DO NOTHING";
const INSERT_HEADER_SQL: &str = "INSERT INTO babylon_ref.h3_reference_cohort \
    (ref_digest, format_version, artifact_name, artifact_manifest_version, artifact_digest, \
     source_digest, source_r5_digest, source_r7_digest, closure_digest, membership_digest, \
     direct_cell_count, derived_ancestor_count, closure_cell_count) \
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
    ON CONFLICT DO NOTHING";
const INSERT_MEMBERSHIP_SQL: &str = "INSERT INTO babylon_ref.h3_reference_membership \
    (ref_digest, cell_id, origin) \
    SELECT $1, input.cell_id, input.origin \
    FROM ROWS FROM (pg_catalog.unnest($2::bigint[]), pg_catalog.unnest($3::smallint[])) \
      AS input(cell_id, origin) \
    ON CONFLICT DO NOTHING";
const INSERT_R8_PRODUCT_SQL: &str = "INSERT INTO babylon_ref.reference_product \
    (ref_digest, product_code, artifact_sha256, semantic_sha256, row_count, evidence_class, \
     measure_unit, denominator) \
    VALUES ($1, $2, $3, $3, $4, 'Derived', 'identity', NULL) \
    ON CONFLICT DO NOTHING";
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

/// Successful installation disposition at the durable `PostgreSQL` boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H3ReferenceInstallDisposition {
    /// This call committed the exact cohort.
    Installed,
    /// The exact cohort already existed and required no write transaction.
    AlreadyPresent,
    /// A transport-ambiguous commit was proven durable after reconnecting.
    ReconciledAfterAmbiguousCommit,
}

/// Closed database operations used in installer failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H3ReferenceInstallOperation {
    /// Open one fresh bounded local connection.
    Connect,
    /// Read the bounded cohort header identity.
    ReadHeader,
    /// Read the bounded membership cardinality sentinel.
    ReadMembershipCardinality {
        /// Lifecycle path that read the membership cardinality.
        context: H3ReferenceMembershipReadContext,
    },
    /// Read and verify the bounded primary-key-ordered membership rows.
    ReadMembershipRows {
        /// Lifecycle path that read the membership rows.
        context: H3ReferenceMembershipReadContext,
    },
    /// Read and verify the bounded primary-key-ordered H3 cell rows.
    ReadCellRows {
        /// Lifecycle path that read the H3 cell rows.
        context: H3ReferenceMembershipReadContext,
    },
    /// Read and verify the exact foundation-owned R8 child rows.
    ReadR8CellRows {
        /// Lifecycle path that read the R8 rows.
        context: H3ReferenceMembershipReadContext,
    },
    /// Read and verify the exact foundation-owned R8 product receipt.
    ReadR8Product {
        /// Lifecycle path that read the product receipt.
        context: H3ReferenceMembershipReadContext,
    },
    /// Begin the serializable write transaction.
    BeginTransaction,
    /// Apply exact local transaction settings.
    SetTransactionSettings,
    /// Verify exact local transaction settings.
    VerifyTransactionSettings,
    /// Apply the installer-only statement bound after epoch inspection.
    SetSessionSettings,
    /// Verify the narrowed installer session profile.
    VerifySessionSettings,
    /// Insert canonical H3 cell rows.
    InsertCells,
    /// Insert the exact cohort header.
    InsertHeader,
    /// Insert direct and derived membership rows.
    InsertMembership,
    /// Insert the complete foundation-owned R8 child set.
    InsertR8Cells,
    /// Insert the exact foundation-owned R8 product receipt.
    InsertR8Product,
    /// Commit the verified transaction.
    CommitTransaction,
    /// Roll back a failed transaction explicitly.
    RollbackTransaction,
}

/// Lifecycle path that read a representative H3 membership result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H3ReferenceMembershipReadContext {
    /// The installer checked for an existing cohort before writing.
    InitialInspection,
    /// The installer verified the write transaction for one commit attempt.
    CommitAttempt {
        /// One-based commit attempt number.
        attempt: usize,
    },
    /// The installer reconnected after an ambiguous commit attempt.
    AmbiguousCommitReconciliation {
        /// One-based commit attempt number that became ambiguous.
        attempt: usize,
    },
}

/// Fixed installer resources with explicit row ceilings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H3ReferenceInstallBoundedResource {
    /// Header candidates selected by identity.
    HeaderRows,
    /// Canonical membership rows read back from `PostgreSQL`.
    MembershipRows,
    /// Batched canonical row insertions.
    InsertBatches,
    /// Complete foundation-owned R8 child rows.
    R8Rows,
    /// Exact foundation product receipt rows.
    ProductRows,
}

/// Exact equivalence surface which failed after an idempotent insert attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H3ReferenceInstallConflict {
    /// The pinned artifact identity already names another cohort digest.
    ArtifactIdentity,
    /// The expected cohort digest has different provenance or counts.
    CohortHeader,
    /// Stored membership or canonical H3 fields differ from the supplied cohort.
    Membership,
    /// Direct stored membership does not rebuild the supplied immutable cohort.
    RebuiltCohort,
    /// The two typed static authorities do not form the governed Michigan bundle.
    ReferenceBundleAuthority,
    /// Stored foundation-owned R8 cells differ from the governed section.
    R8Cells,
    /// Stored foundation product provenance differs from the governed receipt.
    R8ProductReceipt,
}

/// Closed installer refusal and failure states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum H3ReferenceInstallError {
    /// The supplied maintenance target violated the local-only connection contract.
    ConnectionTarget(CatalogError),
    /// The exact schema advisory lock could not be acquired.
    Lock(CatalogError),
    /// The existing schema epoch or owner contract failed inspection.
    CurrentSchema(CurrentSchemaError),
    /// Installation requires one of the exact verified cutover epochs.
    /// A database operation failed with a credential-safe server diagnostic boundary.
    Database {
        operation: H3ReferenceInstallOperation,
        diagnostic: Option<PostgresDiagnostic>,
    },
    /// A database value could not be decoded into the governed type.
    Decode {
        operation: H3ReferenceInstallOperation,
    },
    /// `PostgreSQL` returned a malformed H3 identity despite the schema contract.
    CellIdentity {
        operation: H3ReferenceInstallOperation,
        source: H3CellIdError,
    },
    /// A fixed row bound was exceeded.
    Bounds {
        resource: H3ReferenceInstallBoundedResource,
        actual: usize,
        max: usize,
    },
    /// A bounded installer buffer could not be allocated.
    Allocation {
        resource: H3ReferenceInstallBoundedResource,
    },
    /// Existing durable rows differ from the exact supplied cohort.
    Conflict {
        component: H3ReferenceInstallConflict,
    },
    /// Direct stored rows could not rebuild through the governed cohort constructor.
    Rebuild(H3ReferenceCohortError),
    /// Two transport-ambiguous attempts remained absent after exact reconciliation.
    AmbiguousCommitUnresolved { attempts: usize },
    /// Commit ambiguity remained visible when reconciliation itself refused.
    AmbiguousCommitAndReconciliation {
        attempts: usize,
        reconciliation: Box<H3ReferenceInstallError>,
    },
    /// Explicit schema-lock release failed.
    Unlock(CatalogError),
    /// A primary failure and explicit unlock failure both occurred.
    FailureAndCleanup {
        primary: Box<H3ReferenceInstallError>,
        cleanup: Box<H3ReferenceInstallError>,
    },
    /// A transaction failure and explicit rollback failure both occurred.
    FailureAndRollback {
        primary: Box<H3ReferenceInstallError>,
        rollback: Box<H3ReferenceInstallError>,
    },
}

impl std::fmt::Display for H3ReferenceInstallError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "H3 reference installation refused: {self:?}")
    }
}

impl std::error::Error for H3ReferenceInstallError {}

/// Exact installation receipt for the durable Michigan reference bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct H3ReferenceInstallReport {
    disposition: H3ReferenceInstallDisposition,
    receipt: H3ReferenceCohortReceipt,
    closure_cell_count: usize,
    r8_child_count: usize,
    r8_child_parent_digest: RefDigest,
    reference_bundle_digest: RefDigest,
    commit_attempts: usize,
}

impl H3ReferenceInstallReport {
    /// Durable installation outcome.
    #[must_use]
    pub fn disposition(&self) -> H3ReferenceInstallDisposition {
        self.disposition
    }

    /// Identity of the exact installed reference cohort.
    #[must_use]
    pub fn ref_digest(&self) -> RefDigest {
        self.receipt.ref_digest()
    }

    /// SHA-256 identity of the pinned source artifact bytes.
    #[must_use]
    pub fn artifact_digest(&self) -> RefDigest {
        self.receipt.artifact_digest()
    }

    /// Governed cohort framing version.
    #[must_use]
    pub fn format_version(&self) -> i16 {
        H3_REFERENCE_COHORT_FORMAT_VERSION
    }

    /// Pinned source artifact filename.
    #[must_use]
    pub fn artifact_name(&self) -> &str {
        H3_REFERENCE_ARTIFACT_NAME
    }

    /// Pinned artifact manifest version.
    #[must_use]
    pub fn artifact_manifest_version(&self) -> &str {
        H3_REFERENCE_ARTIFACT_MANIFEST_VERSION
    }

    /// Direct source identities committed to membership.
    #[must_use]
    pub fn direct_cell_count(&self) -> usize {
        self.receipt.direct_cell_count()
    }

    /// Strict parents added to close the H3 hierarchy.
    #[must_use]
    pub fn derived_ancestor_count(&self) -> usize {
        self.receipt.derived_ancestor_count()
    }

    /// Total canonical membership rows committed for this cohort.
    #[must_use]
    pub fn closure_cell_count(&self) -> usize {
        self.closure_cell_count
    }

    /// Complete foundation-owned resolution-eight child count.
    #[must_use]
    pub fn r8_child_count(&self) -> usize {
        self.r8_child_count
    }

    /// Digest of the exact self-framed R8 child-to-R7-parent section.
    #[must_use]
    pub fn r8_child_parent_digest(&self) -> RefDigest {
        self.r8_child_parent_digest
    }

    /// Digest joining the unchanged base cohort and the governed R8 section.
    #[must_use]
    pub fn reference_bundle_digest(&self) -> RefDigest {
        self.reference_bundle_digest
    }

    /// Number of write-commit attempts made by this invocation.
    #[must_use]
    pub fn commit_attempts(&self) -> usize {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InstallResolution {
    disposition: H3ReferenceInstallDisposition,
    commit_attempts: usize,
}

trait InstallDriver {
    fn attempt_commit(&mut self, attempt: usize) -> Result<CommitAttempt, H3ReferenceInstallError>;
    fn reconcile(
        &mut self,
        after_attempt: usize,
    ) -> Result<InstallPresence, H3ReferenceInstallError>;
}

#[derive(Debug, Clone, Copy)]
struct MichiganH3ReferenceBundle<'a> {
    cohort: &'a H3ReferenceCohort,
    foundation: &'a MichiganDynamicHexFoundation,
    r8_child_parent_digest: RefDigest,
    reference_bundle_digest: RefDigest,
}

impl<'a> MichiganH3ReferenceBundle<'a> {
    fn validate(
        cohort: &'a H3ReferenceCohort,
        foundation: &'a MichiganDynamicHexFoundation,
    ) -> Result<Self, H3ReferenceInstallError> {
        let receipt = cohort.receipt();
        if foundation.base_reference_cohort_digest() != *receipt.ref_digest().as_bytes()
            || foundation.source_r7_digest() != *receipt.source_r7_digest().as_bytes()
        {
            return Err(conflict(
                H3ReferenceInstallConflict::ReferenceBundleAuthority,
            ));
        }
        let rows = foundation.r8_child_parent_rows();
        if rows.len() != MICHIGAN_R8_CHILD_ROWS {
            return Err(H3ReferenceInstallError::Bounds {
                resource: H3ReferenceInstallBoundedResource::R8Rows,
                actual: rows.len(),
                max: MICHIGAN_R8_CHILD_ROWS,
            });
        }
        validate_r8_child_parent_rows(cohort, rows)?;
        let r8_child_parent_digest = digest_r8_child_parent_rows(rows)?;
        if foundation.r8_section_digest() != *r8_child_parent_digest.as_bytes() {
            return Err(conflict(
                H3ReferenceInstallConflict::ReferenceBundleAuthority,
            ));
        }
        let reference_bundle_digest =
            digest_reference_bundle(receipt.ref_digest(), r8_child_parent_digest);
        if foundation.reference_bundle_digest() != *reference_bundle_digest.as_bytes() {
            return Err(conflict(
                H3ReferenceInstallConflict::ReferenceBundleAuthority,
            ));
        }
        Ok(Self {
            cohort,
            foundation,
            r8_child_parent_digest,
            reference_bundle_digest,
        })
    }

    fn r8_rows(&self) -> &[MichiganH3R8ChildParent] {
        self.foundation.r8_child_parent_rows()
    }
}

fn validate_r8_child_parent_rows(
    cohort: &H3ReferenceCohort,
    rows: &[MichiganH3R8ChildParent],
) -> Result<(), H3ReferenceInstallError> {
    let mut previous = None;
    for row in rows.iter().take(MICHIGAN_R8_CHILD_ROWS) {
        let child = row.child_cell_id();
        let parent = row.parent_r7_cell_id();
        if child.resolution() != 8
            || parent.resolution() != 7
            || child.immediate_parent() != Some(parent)
            || previous.is_some_and(|prior| prior >= child)
            || cohort
                .rows()
                .binary_search_by_key(&child.as_u64(), |candidate| candidate.cell_id().as_u64())
                .is_ok()
        {
            return Err(conflict(
                H3ReferenceInstallConflict::ReferenceBundleAuthority,
            ));
        }
        let parent_index = cohort
            .rows()
            .binary_search_by_key(&parent.as_u64(), |candidate| candidate.cell_id().as_u64())
            .map_err(|_| conflict(H3ReferenceInstallConflict::ReferenceBundleAuthority))?;
        let parent_row = &cohort.rows()[parent_index];
        if parent_row.resolution() != 7 || parent_row.origin() != H3ReferenceOrigin::Direct {
            return Err(conflict(
                H3ReferenceInstallConflict::ReferenceBundleAuthority,
            ));
        }
        previous = Some(child);
    }
    Ok(())
}

fn digest_r8_child_parent_rows(
    rows: &[MichiganH3R8ChildParent],
) -> Result<RefDigest, H3ReferenceInstallError> {
    let capacity = rows
        .len()
        .checked_mul(16)
        .and_then(|payload| {
            MICHIGAN_R8_SECTION_DOMAIN
                .len()
                .checked_add(8)?
                .checked_add(payload)
        })
        .ok_or(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::R8Rows,
            actual: usize::MAX,
            max: MICHIGAN_R8_CHILD_ROWS,
        })?;
    let mut framed = Vec::new();
    framed
        .try_reserve_exact(capacity)
        .map_err(|_| H3ReferenceInstallError::Allocation {
            resource: H3ReferenceInstallBoundedResource::R8Rows,
        })?;
    framed.extend_from_slice(MICHIGAN_R8_SECTION_DOMAIN);
    framed.extend_from_slice(
        &u64::try_from(rows.len())
            .map_err(|_| H3ReferenceInstallError::Bounds {
                resource: H3ReferenceInstallBoundedResource::R8Rows,
                actual: rows.len(),
                max: MICHIGAN_R8_CHILD_ROWS,
            })?
            .to_be_bytes(),
    );
    for row in rows.iter().take(MICHIGAN_R8_CHILD_ROWS) {
        framed.extend_from_slice(&row.child_cell_id().to_be_bytes());
        framed.extend_from_slice(&row.parent_r7_cell_id().to_be_bytes());
    }
    Ok(RefDigest::from_bytes(sha256_of(&framed)))
}

fn digest_reference_bundle(
    base_ref_digest: RefDigest,
    r8_child_parent_digest: RefDigest,
) -> RefDigest {
    let mut framed = Vec::with_capacity(MICHIGAN_REFERENCE_BUNDLE_DOMAIN.len() + 64);
    framed.extend_from_slice(MICHIGAN_REFERENCE_BUNDLE_DOMAIN);
    framed.extend_from_slice(base_ref_digest.as_bytes());
    framed.extend_from_slice(r8_child_parent_digest.as_bytes());
    RefDigest::from_bytes(sha256_of(&framed))
}

/// Install the exact Michigan H3 reference bundle into the verified cutover epoch.
///
/// This immutable reference-data maintenance entry point does not request runtime writer
/// authority and never advances the schema epoch for the caller. It admits only the exact
/// additive H3 prefix (v6) or the terminal canonical-reader prefix (v7). The base cohort,
/// complete R8 child section, and R8 product receipt commit through one transaction.
///
/// # Errors
/// Returns [`H3ReferenceInstallError`] before publication for any target, lock, owner, epoch,
/// transaction, equivalence, reconciliation, or cleanup failure.
pub fn install_michigan_h3_reference_bundle(
    config: &Config,
    cohort: &H3ReferenceCohort,
    foundation: &MichiganDynamicHexFoundation,
) -> Result<H3ReferenceInstallReport, H3ReferenceInstallError> {
    let bundle = MichiganH3ReferenceBundle::validate(cohort, foundation)?;
    let mut attempt = attempt_install_transaction;
    install_michigan_h3_reference_bundle_using(config, &bundle, &mut attempt)
}

fn install_michigan_h3_reference_bundle_using<Attempt>(
    config: &Config,
    bundle: &MichiganH3ReferenceBundle<'_>,
    attempt: &mut Attempt,
) -> Result<H3ReferenceInstallReport, H3ReferenceInstallError>
where
    Attempt: FnMut(
        &mut Client,
        &MichiganH3ReferenceBundle<'_>,
        usize,
    ) -> Result<CommitAttempt, H3ReferenceInstallError>,
{
    validate_connection_target(config).map_err(H3ReferenceInstallError::ConnectionTarget)?;
    let bounded = installer_config(config);
    let mut session = LockedInstallSession::connect(&bounded)?;
    let primary = install_under_lock(&bounded, &mut session, bundle, attempt);
    session.finish(primary)
}

fn installer_config(config: &Config) -> Config {
    bounded_config(config)
}

fn install_under_lock<Attempt>(
    config: &Config,
    session: &mut LockedInstallSession,
    bundle: &MichiganH3ReferenceBundle<'_>,
    attempt: &mut Attempt,
) -> Result<H3ReferenceInstallReport, H3ReferenceInstallError>
where
    Attempt: FnMut(
        &mut Client,
        &MichiganH3ReferenceBundle<'_>,
        usize,
    ) -> Result<CommitAttempt, H3ReferenceInstallError>,
{
    require_schema(session.client())?;
    prepare_installer_session(session.client())?;
    let initial = inspect_presence(
        session.client(),
        bundle,
        H3ReferenceMembershipReadContext::InitialInspection,
    )?;
    let resolution = {
        let mut driver = DatabaseInstallDriver {
            config,
            session,
            bundle,
            attempt,
        };
        drive_install(initial, &mut driver)?
    };
    build_report(bundle, resolution)
}

fn drive_install<Driver: InstallDriver>(
    initial: InstallPresence,
    driver: &mut Driver,
) -> Result<InstallResolution, H3ReferenceInstallError> {
    if initial == InstallPresence::Exact {
        return Ok(InstallResolution {
            disposition: H3ReferenceInstallDisposition::AlreadyPresent,
            commit_attempts: 0,
        });
    }
    for attempt_index in 0..MAX_H3_REFERENCE_INSTALL_COMMIT_ATTEMPTS {
        let commit_attempts = attempt_index + 1;
        match driver.attempt_commit(commit_attempts)? {
            CommitAttempt::Committed => {
                return Ok(InstallResolution {
                    disposition: H3ReferenceInstallDisposition::Installed,
                    commit_attempts,
                });
            }
            CommitAttempt::Ambiguous => {
                let reconciled = driver
                    .reconcile(commit_attempts)
                    .map_err(|reconciliation| {
                        H3ReferenceInstallError::AmbiguousCommitAndReconciliation {
                            attempts: commit_attempts,
                            reconciliation: Box::new(reconciliation),
                        }
                    })?;
                if reconciled == InstallPresence::Exact {
                    return Ok(InstallResolution {
                        disposition: H3ReferenceInstallDisposition::ReconciledAfterAmbiguousCommit,
                        commit_attempts,
                    });
                }
            }
        }
    }
    Err(H3ReferenceInstallError::AmbiguousCommitUnresolved {
        attempts: MAX_H3_REFERENCE_INSTALL_COMMIT_ATTEMPTS,
    })
}

fn build_report(
    bundle: &MichiganH3ReferenceBundle<'_>,
    resolution: InstallResolution,
) -> Result<H3ReferenceInstallReport, H3ReferenceInstallError> {
    let receipt = bundle.cohort.receipt().clone();
    let closure_cell_count = receipt
        .direct_cell_count()
        .checked_add(receipt.derived_ancestor_count())
        .ok_or(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::MembershipRows,
            actual: usize::MAX,
            max: MAX_H3_REFERENCE_CLOSURE_ROWS,
        })?;
    Ok(H3ReferenceInstallReport {
        disposition: resolution.disposition,
        receipt,
        closure_cell_count,
        r8_child_count: bundle.r8_rows().len(),
        r8_child_parent_digest: bundle.r8_child_parent_digest,
        reference_bundle_digest: bundle.reference_bundle_digest,
        commit_attempts: resolution.commit_attempts,
    })
}

struct LockedInstallSession {
    client: Option<Client>,
}

impl LockedInstallSession {
    fn connect(config: &Config) -> Result<Self, H3ReferenceInstallError> {
        let mut client = config.connect(NoTls).map_err(|error| {
            postgres_database_error(H3ReferenceInstallOperation::Connect, &error)
        })?;
        acquire_lock(&mut client).map_err(H3ReferenceInstallError::Lock)?;
        Ok(Self {
            client: Some(client),
        })
    }

    fn client(&mut self) -> &mut Client {
        self.client
            .as_mut()
            .expect("locked installer session always contains one client")
    }

    fn reconnect(&mut self, config: &Config) -> Result<(), H3ReferenceInstallError> {
        self.client.take();
        *self = Self::connect(config)?;
        Ok(())
    }

    fn finish<T>(
        mut self,
        primary: Result<T, H3ReferenceInstallError>,
    ) -> Result<T, H3ReferenceInstallError> {
        let cleanup = self.client.as_mut().map_or(Ok(()), |client| {
            release_lock(client).map_err(H3ReferenceInstallError::Unlock)
        });
        match (primary, cleanup) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), Ok(())) => Err(error),
            (Ok(_), Err(cleanup)) => Err(cleanup),
            (Err(primary), Err(cleanup)) => Err(H3ReferenceInstallError::FailureAndCleanup {
                primary: Box::new(primary),
                cleanup: Box::new(cleanup),
            }),
        }
    }
}

struct DatabaseInstallDriver<'a, Attempt> {
    config: &'a Config,
    session: &'a mut LockedInstallSession,
    bundle: &'a MichiganH3ReferenceBundle<'a>,
    attempt: &'a mut Attempt,
}

impl<Attempt> InstallDriver for DatabaseInstallDriver<'_, Attempt>
where
    Attempt: FnMut(
        &mut Client,
        &MichiganH3ReferenceBundle<'_>,
        usize,
    ) -> Result<CommitAttempt, H3ReferenceInstallError>,
{
    fn attempt_commit(&mut self, attempt: usize) -> Result<CommitAttempt, H3ReferenceInstallError> {
        (self.attempt)(self.session.client(), self.bundle, attempt)
    }

    fn reconcile(
        &mut self,
        after_attempt: usize,
    ) -> Result<InstallPresence, H3ReferenceInstallError> {
        self.session.reconnect(self.config)?;
        require_schema(self.session.client())?;
        prepare_installer_session(self.session.client())?;
        inspect_presence(
            self.session.client(),
            self.bundle,
            H3ReferenceMembershipReadContext::AmbiguousCommitReconciliation {
                attempt: after_attempt,
            },
        )
    }
}

fn require_schema(client: &mut Client) -> Result<(), H3ReferenceInstallError> {
    require_current_schema(client)
        .map(|_| ())
        .map_err(H3ReferenceInstallError::CurrentSchema)
}

fn prepare_installer_session(client: &mut Client) -> Result<(), H3ReferenceInstallError> {
    client
        .batch_execute(H3_REFERENCE_SESSION_SETTINGS_SQL)
        .map_err(|error| {
            postgres_database_error(H3ReferenceInstallOperation::SetSessionSettings, &error)
        })?;
    let operation = H3ReferenceInstallOperation::VerifySessionSettings;
    let row = client
        .query_one(H3_REFERENCE_SESSION_SETTINGS_QUERY, &[])
        .map_err(|error| postgres_database_error(operation, &error))?;
    let expected = ["on", "pg_catalog", "30s", "5s", "5s", "off", "off", "off"];
    for (index, wanted) in expected.iter().enumerate().take(8) {
        let actual: String = decode_value(&row, index, operation)?;
        if actual != *wanted {
            return Err(database_error(operation));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CohortHeader {
    ref_digest: RefDigest,
    format_version: i16,
    artifact_name: String,
    artifact_manifest_version: String,
    artifact_digest: RefDigest,
    source_digest: RefDigest,
    source_r5_digest: RefDigest,
    source_r7_digest: RefDigest,
    closure_digest: RefDigest,
    membership_digest: RefDigest,
    direct_cell_count: usize,
    derived_ancestor_count: usize,
    closure_cell_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StoredReferenceRow {
    cell_id: H3CellId,
    resolution: u8,
    immediate_parent: Option<H3CellId>,
    ancestor_r4: Option<H3CellId>,
    ancestor_r5: Option<H3CellId>,
    ancestor_r6: Option<H3CellId>,
    ancestor_r7: Option<H3CellId>,
    origin: H3ReferenceOrigin,
}

fn inspect_presence<ClientType: GenericClient>(
    client: &mut ClientType,
    bundle: &MichiganH3ReferenceBundle<'_>,
    context: H3ReferenceMembershipReadContext,
) -> Result<InstallPresence, H3ReferenceInstallError> {
    let cohort = bundle.cohort;
    let receipt = cohort.receipt();
    let ref_digest = receipt.ref_digest();
    let artifact_digest = receipt.artifact_digest();
    let rows = client
        .query(
            READ_HEADER_SQL,
            &[
                &ref_digest.as_bytes().as_slice(),
                &H3_REFERENCE_COHORT_FORMAT_VERSION,
                &artifact_digest.as_bytes().as_slice(),
                &H3_REFERENCE_HEADER_QUERY_LIMIT,
            ],
        )
        .map_err(|error| {
            postgres_database_error(H3ReferenceInstallOperation::ReadHeader, &error)
        })?;
    if rows.len() > MAX_H3_REFERENCE_HEADER_ROWS {
        return Err(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::HeaderRows,
            actual: rows.len(),
            max: MAX_H3_REFERENCE_HEADER_ROWS,
        });
    }
    let Some(row) = rows.first() else {
        return Ok(InstallPresence::Absent);
    };
    if rows.len() != 1 {
        return Err(conflict(H3ReferenceInstallConflict::ArtifactIdentity));
    }
    let header = decode_header(row)?;
    validate_header(&header, receipt)?;
    verify_membership(client, cohort, context)?;
    if read_r8_product(client, bundle, context)?.is_some() {
        verify_r8_cells(client, bundle, context)?;
        Ok(InstallPresence::Exact)
    } else {
        Err(conflict(H3ReferenceInstallConflict::R8ProductReceipt))
    }
}

fn decode_header(row: &Row) -> Result<CohortHeader, H3ReferenceInstallError> {
    let operation = H3ReferenceInstallOperation::ReadHeader;
    Ok(CohortHeader {
        ref_digest: decode_digest(row, 0, operation)?,
        format_version: decode_value(row, 1, operation)?,
        artifact_name: decode_value(row, 2, operation)?,
        artifact_manifest_version: decode_value(row, 3, operation)?,
        artifact_digest: decode_digest(row, 4, operation)?,
        source_digest: decode_digest(row, 5, operation)?,
        source_r5_digest: decode_digest(row, 6, operation)?,
        source_r7_digest: decode_digest(row, 7, operation)?,
        closure_digest: decode_digest(row, 8, operation)?,
        membership_digest: decode_digest(row, 9, operation)?,
        direct_cell_count: decode_count(row, 10, operation)?,
        derived_ancestor_count: decode_count(row, 11, operation)?,
        closure_cell_count: decode_count(row, 12, operation)?,
    })
}

fn expected_header(
    receipt: &H3ReferenceCohortReceipt,
) -> Result<CohortHeader, H3ReferenceInstallError> {
    let closure_cell_count = receipt
        .direct_cell_count()
        .checked_add(receipt.derived_ancestor_count())
        .ok_or(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::MembershipRows,
            actual: usize::MAX,
            max: MAX_H3_REFERENCE_CLOSURE_ROWS,
        })?;
    Ok(CohortHeader {
        ref_digest: receipt.ref_digest(),
        format_version: H3_REFERENCE_COHORT_FORMAT_VERSION,
        artifact_name: H3_REFERENCE_ARTIFACT_NAME.to_owned(),
        artifact_manifest_version: H3_REFERENCE_ARTIFACT_MANIFEST_VERSION.to_owned(),
        artifact_digest: receipt.artifact_digest(),
        source_digest: receipt.source_digest(),
        source_r5_digest: receipt.source_r5_digest(),
        source_r7_digest: receipt.source_r7_digest(),
        closure_digest: receipt.closure_digest(),
        membership_digest: receipt.membership_digest(),
        direct_cell_count: receipt.direct_cell_count(),
        derived_ancestor_count: receipt.derived_ancestor_count(),
        closure_cell_count,
    })
}

fn validate_header(
    actual: &CohortHeader,
    receipt: &H3ReferenceCohortReceipt,
) -> Result<(), H3ReferenceInstallError> {
    if actual.ref_digest != receipt.ref_digest() {
        return Err(conflict(H3ReferenceInstallConflict::ArtifactIdentity));
    }
    if actual != &expected_header(receipt)? {
        return Err(conflict(H3ReferenceInstallConflict::CohortHeader));
    }
    Ok(())
}

fn read_r8_product<ClientType: GenericClient>(
    client: &mut ClientType,
    bundle: &MichiganH3ReferenceBundle<'_>,
    context: H3ReferenceMembershipReadContext,
) -> Result<Option<()>, H3ReferenceInstallError> {
    let operation = H3ReferenceInstallOperation::ReadR8Product { context };
    let ref_digest = bundle.cohort.receipt().ref_digest();
    let rows = client
        .query(
            READ_R8_PRODUCT_SQL,
            &[
                &ref_digest.as_bytes().as_slice(),
                &MICHIGAN_R8_PRODUCT_CODE,
                &H3_REFERENCE_PRODUCT_QUERY_LIMIT,
            ],
        )
        .map_err(|error| postgres_database_error(operation, &error))?;
    if rows.is_empty() {
        return Ok(None);
    }
    if rows.len() != 1 {
        return Err(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::ProductRows,
            actual: rows.len(),
            max: 1,
        });
    }
    let row = &rows[0];
    let product_code: String = decode_value(row, 0, operation)?;
    let artifact_digest = decode_digest(row, 1, operation)?;
    let semantic_digest = decode_optional_digest(row, 2, operation)?;
    let row_count = decode_count(row, 3, operation)?;
    let evidence_class: String = decode_value(row, 4, operation)?;
    let measure_unit: Option<String> = decode_value(row, 5, operation)?;
    let denominator: Option<String> = decode_value(row, 6, operation)?;
    let exact = product_code == MICHIGAN_R8_PRODUCT_CODE
        && artifact_digest == bundle.r8_child_parent_digest
        && semantic_digest == Some(bundle.r8_child_parent_digest)
        && row_count == bundle.r8_rows().len()
        && evidence_class == "Derived"
        && measure_unit.as_deref() == Some("identity")
        && denominator.is_none();
    if exact {
        Ok(Some(()))
    } else {
        Err(conflict(H3ReferenceInstallConflict::R8ProductReceipt))
    }
}

fn verify_r8_cells<ClientType: GenericClient>(
    client: &mut ClientType,
    bundle: &MichiganH3ReferenceBundle<'_>,
    context: H3ReferenceMembershipReadContext,
) -> Result<(), H3ReferenceInstallError> {
    let operation = H3ReferenceInstallOperation::ReadR8CellRows { context };
    for batch in bundle.r8_rows().chunks(H3_REFERENCE_INSTALL_BATCH_ROWS) {
        let stored = read_r8_cell_batch(client, batch, operation)?;
        if stored.len() != batch.len() {
            return Err(conflict(H3ReferenceInstallConflict::R8Cells));
        }
        for (row, expected) in stored.iter().zip(batch) {
            verify_r8_cell_row(row, expected, operation)?;
        }
    }
    Ok(())
}

fn read_r8_cell_batch<ClientType: GenericClient>(
    client: &mut ClientType,
    batch: &[MichiganH3R8ChildParent],
    operation: H3ReferenceInstallOperation,
) -> Result<Vec<Row>, H3ReferenceInstallError> {
    let cell_ids = batch
        .iter()
        .map(|row| cell_to_sql(row.child_cell_id(), operation))
        .collect::<Result<Vec<_>, _>>()?;
    let bounded_limit = batch
        .len()
        .checked_add(1)
        .filter(|limit| *limit <= H3_REFERENCE_INSTALL_BATCH_ROWS + 1)
        .ok_or(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::R8Rows,
            actual: batch.len(),
            max: H3_REFERENCE_INSTALL_BATCH_ROWS,
        })?;
    let query_limit =
        i64::try_from(bounded_limit).map_err(|_| H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::R8Rows,
            actual: batch.len(),
            max: H3_REFERENCE_INSTALL_BATCH_ROWS,
        })?;
    let rows = client
        .query(READ_CELLS_SQL, &[&cell_ids, &query_limit])
        .map_err(|error| postgres_database_error(operation, &error))?;
    if rows.len() > batch.len() {
        return Err(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::R8Rows,
            actual: rows.len(),
            max: batch.len(),
        });
    }
    Ok(rows)
}

fn verify_r8_cell_row(
    row: &Row,
    expected: &MichiganH3R8ChildParent,
    operation: H3ReferenceInstallOperation,
) -> Result<(), H3ReferenceInstallError> {
    let child = expected.child_cell_id();
    let parent = expected.parent_r7_cell_id();
    let stored_cell = decode_cell(row, 0, operation)?;
    let stored_resolution: i16 = decode_value(row, 1, operation)?;
    let stored_immediate_parent = decode_optional_cell(row, 2, operation)?;
    let stored_ancestor_r4 = decode_optional_cell(row, 3, operation)?;
    let stored_ancestor_r5 = decode_optional_cell(row, 4, operation)?;
    let stored_ancestor_r6 = decode_optional_cell(row, 5, operation)?;
    let stored_ancestor_r7 = decode_optional_cell(row, 6, operation)?;
    let exact = stored_cell == child
        && stored_resolution == 8
        && stored_immediate_parent == Some(parent)
        && stored_ancestor_r4 == Some(ancestor_at(child, 4, operation)?)
        && stored_ancestor_r5 == Some(ancestor_at(child, 5, operation)?)
        && stored_ancestor_r6 == Some(ancestor_at(child, 6, operation)?)
        && stored_ancestor_r7 == Some(parent);
    if exact {
        Ok(())
    } else {
        Err(conflict(H3ReferenceInstallConflict::R8Cells))
    }
}

#[cfg(test)]
fn membership_receipt_identity(
    operation: H3ReferenceInstallOperation,
) -> (&'static str, &'static str, Option<usize>) {
    let (query, context) = match operation {
        H3ReferenceInstallOperation::ReadMembershipCardinality { context } => {
            ("cardinality", context)
        }
        H3ReferenceInstallOperation::ReadMembershipRows { context } => ("rows", context),
        H3ReferenceInstallOperation::ReadCellRows { context } => ("cells", context),
        _ => unreachable!("reference receipts accept only reference-read operations"),
    };
    match context {
        H3ReferenceMembershipReadContext::InitialInspection => (query, "initial", None),
        H3ReferenceMembershipReadContext::CommitAttempt { attempt } => {
            (query, "commit", Some(attempt))
        }
        H3ReferenceMembershipReadContext::AmbiguousCommitReconciliation { attempt } => {
            (query, "reconcile", Some(attempt))
        }
    }
}

#[cfg(test)]
fn emit_membership_read_receipt(
    operation: H3ReferenceInstallOperation,
    elapsed: Duration,
    rows: Option<usize>,
    error: Option<&postgres::Error>,
) {
    let (query, context, attempt) = membership_receipt_identity(operation);
    let attempt = attempt.map_or_else(|| "none".to_owned(), |value| value.to_string());
    let rows = rows.map_or_else(|| "none".to_owned(), |value| value.to_string());
    let sqlstate = error
        .and_then(postgres::Error::as_db_error)
        .map_or("none", |server| server.code().code());
    if error.is_some() {
        eprintln!(
            "PER267_MEMBERSHIP_READ query={query} context={context} attempt={attempt} completion=error elapsed_ms={} rows={rows} sqlstate={sqlstate}",
            elapsed.as_millis()
        );
    } else {
        eprintln!(
            "PER267_MEMBERSHIP_READ query={query} context={context} attempt={attempt} completion=ok elapsed_ms={} rows={rows} sqlstate={sqlstate}",
            elapsed.as_millis()
        );
    }
    std::io::stderr()
        .flush()
        .expect("PER-267 diagnostic receipt must flush");
}

fn verify_membership<ClientType: GenericClient>(
    client: &mut ClientType,
    cohort: &H3ReferenceCohort,
    context: H3ReferenceMembershipReadContext,
) -> Result<(), H3ReferenceInstallError> {
    verify_membership_cardinality(client, cohort, context)?;
    let membership_operation = H3ReferenceInstallOperation::ReadMembershipRows { context };
    let membership_rows = read_membership_rows(client, cohort, membership_operation)?;
    let cell_operation = H3ReferenceInstallOperation::ReadCellRows { context };
    let cell_rows = read_cell_rows(client, cohort, cell_operation)?;
    if membership_rows.len() != cohort.rows().len() || cell_rows.len() != cohort.rows().len() {
        return Err(conflict(H3ReferenceInstallConflict::Membership));
    }
    let mut expected_index = 0_usize;
    let mut direct_cells = Vec::with_capacity(cohort.receipt().direct_cell_count());
    for (membership_row, cell_row) in membership_rows
        .iter()
        .zip(cell_rows.iter())
        .take(MAX_H3_REFERENCE_CLOSURE_ROWS)
    {
        let stored = decode_stored_row(
            membership_row,
            cell_row,
            membership_operation,
            cell_operation,
        )?;
        let expected = cohort
            .rows()
            .get(expected_index)
            .ok_or_else(|| conflict(H3ReferenceInstallConflict::Membership))?;
        compare_stored_row(&stored, expected)?;
        if stored.origin == H3ReferenceOrigin::Direct {
            direct_cells.push(stored.cell_id);
        }
        expected_index = expected_index
            .checked_add(1)
            .ok_or(H3ReferenceInstallError::Bounds {
                resource: H3ReferenceInstallBoundedResource::MembershipRows,
                actual: usize::MAX,
                max: MAX_H3_REFERENCE_CLOSURE_ROWS,
            })?;
    }
    finish_membership_verification(expected_index, &direct_cells, cohort)
}

fn verify_membership_cardinality<ClientType: GenericClient>(
    client: &mut ClientType,
    cohort: &H3ReferenceCohort,
    context: H3ReferenceMembershipReadContext,
) -> Result<(), H3ReferenceInstallError> {
    let operation = H3ReferenceInstallOperation::ReadMembershipCardinality { context };
    let ref_digest = cohort.receipt().ref_digest();
    let expected = cohort.rows().len();
    let query_limit = expected
        .checked_add(1)
        .filter(|limit| *limit <= MAX_H3_REFERENCE_CARDINALITY_QUERY_ROWS)
        .ok_or(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::MembershipRows,
            actual: expected,
            max: MAX_H3_REFERENCE_CLOSURE_ROWS,
        })?;
    let query_limit = i64::try_from(query_limit).map_err(|_| H3ReferenceInstallError::Bounds {
        resource: H3ReferenceInstallBoundedResource::MembershipRows,
        actual: expected,
        max: MAX_H3_REFERENCE_CLOSURE_ROWS,
    })?;
    #[cfg(test)]
    let query_started = std::time::Instant::now();
    let query = client.query_one(
        READ_MEMBERSHIP_CARDINALITY_SQL,
        &[&ref_digest.as_bytes().as_slice(), &query_limit],
    );
    #[cfg(test)]
    emit_membership_read_receipt(
        operation,
        query_started.elapsed(),
        query.as_ref().ok().map(|_| 1),
        query.as_ref().err(),
    );
    let row = query.map_err(|error| postgres_database_error(operation, &error))?;
    let actual: i64 = decode_value(&row, 0, operation)?;
    let actual =
        usize::try_from(actual).map_err(|_| H3ReferenceInstallError::Decode { operation })?;
    if actual < expected {
        return Err(conflict(H3ReferenceInstallConflict::Membership));
    }
    if actual > expected {
        return Err(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::MembershipRows,
            actual,
            max: expected,
        });
    }
    Ok(())
}

fn read_membership_rows<ClientType: GenericClient>(
    client: &mut ClientType,
    cohort: &H3ReferenceCohort,
    operation: H3ReferenceInstallOperation,
) -> Result<Vec<Row>, H3ReferenceInstallError> {
    let expected = cohort.rows().len();
    let query_limit = expected
        .checked_add(1)
        .filter(|limit| *limit <= MAX_H3_REFERENCE_CARDINALITY_QUERY_ROWS)
        .ok_or(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::MembershipRows,
            actual: expected,
            max: MAX_H3_REFERENCE_CLOSURE_ROWS,
        })?;
    let query_limit = i64::try_from(query_limit).map_err(|_| H3ReferenceInstallError::Bounds {
        resource: H3ReferenceInstallBoundedResource::MembershipRows,
        actual: expected,
        max: MAX_H3_REFERENCE_CLOSURE_ROWS,
    })?;
    let ref_digest = cohort.receipt().ref_digest();
    #[cfg(test)]
    let query_started = std::time::Instant::now();
    let query = client.query(
        READ_MEMBERSHIP_SQL,
        &[&ref_digest.as_bytes().as_slice(), &query_limit],
    );
    #[cfg(test)]
    emit_membership_read_receipt(
        operation,
        query_started.elapsed(),
        query.as_ref().ok().map(Vec::len),
        query.as_ref().err(),
    );
    let rows = query.map_err(|error| postgres_database_error(operation, &error))?;
    if rows.len() > expected {
        return Err(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::MembershipRows,
            actual: rows.len(),
            max: expected,
        });
    }
    Ok(rows)
}

fn read_cell_rows<ClientType: GenericClient>(
    client: &mut ClientType,
    cohort: &H3ReferenceCohort,
    operation: H3ReferenceInstallOperation,
) -> Result<Vec<Row>, H3ReferenceInstallError> {
    let expected = cohort.rows().len();
    let query_limit = expected
        .checked_add(1)
        .filter(|limit| *limit <= MAX_H3_REFERENCE_CARDINALITY_QUERY_ROWS)
        .ok_or(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::MembershipRows,
            actual: expected,
            max: MAX_H3_REFERENCE_CLOSURE_ROWS,
        })?;
    let query_limit = i64::try_from(query_limit).map_err(|_| H3ReferenceInstallError::Bounds {
        resource: H3ReferenceInstallBoundedResource::MembershipRows,
        actual: expected,
        max: MAX_H3_REFERENCE_CLOSURE_ROWS,
    })?;
    let mut cell_ids = Vec::with_capacity(expected);
    for row in cohort.rows().iter().take(MAX_H3_REFERENCE_CLOSURE_ROWS) {
        cell_ids.push(cell_to_sql(row.cell_id(), operation)?);
    }
    #[cfg(test)]
    let query_started = std::time::Instant::now();
    let query = client.query(READ_CELLS_SQL, &[&cell_ids, &query_limit]);
    #[cfg(test)]
    emit_membership_read_receipt(
        operation,
        query_started.elapsed(),
        query.as_ref().ok().map(Vec::len),
        query.as_ref().err(),
    );
    let rows = query.map_err(|error| postgres_database_error(operation, &error))?;
    if rows.len() > expected {
        return Err(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::MembershipRows,
            actual: rows.len(),
            max: expected,
        });
    }
    Ok(rows)
}

fn finish_membership_verification(
    expected_index: usize,
    direct_cells: &[H3CellId],
    cohort: &H3ReferenceCohort,
) -> Result<(), H3ReferenceInstallError> {
    if expected_index != cohort.rows().len()
        || direct_cells.len() != cohort.receipt().direct_cell_count()
    {
        return Err(conflict(H3ReferenceInstallConflict::Membership));
    }
    let rebuilt = build_representative_h3_cohort(cohort.receipt().artifact_digest(), direct_cells)
        .map_err(H3ReferenceInstallError::Rebuild)?;
    if rebuilt != *cohort {
        return Err(conflict(H3ReferenceInstallConflict::RebuiltCohort));
    }
    Ok(())
}

fn decode_stored_row(
    membership_row: &Row,
    cell_row: &Row,
    membership_operation: H3ReferenceInstallOperation,
    cell_operation: H3ReferenceInstallOperation,
) -> Result<StoredReferenceRow, H3ReferenceInstallError> {
    let membership_identity: i64 = decode_value(membership_row, 0, membership_operation)?;
    let stored_identity: i64 = decode_value(cell_row, 0, cell_operation)?;
    if membership_identity != stored_identity {
        return Err(conflict(H3ReferenceInstallConflict::Membership));
    }
    let cell_id = H3CellId::try_from(stored_identity).map_err(|source| {
        H3ReferenceInstallError::CellIdentity {
            operation: cell_operation,
            source,
        }
    })?;
    let resolution: i16 = decode_value(cell_row, 1, cell_operation)?;
    let resolution = u8::try_from(resolution).map_err(|_| H3ReferenceInstallError::Decode {
        operation: cell_operation,
    })?;
    Ok(StoredReferenceRow {
        cell_id,
        resolution,
        immediate_parent: decode_optional_cell(cell_row, 2, cell_operation)?,
        ancestor_r4: decode_optional_cell(cell_row, 3, cell_operation)?,
        ancestor_r5: decode_optional_cell(cell_row, 4, cell_operation)?,
        ancestor_r6: decode_optional_cell(cell_row, 5, cell_operation)?,
        ancestor_r7: decode_optional_cell(cell_row, 6, cell_operation)?,
        origin: decode_origin(membership_row, 1, membership_operation)?,
    })
}

fn compare_stored_row(
    actual: &StoredReferenceRow,
    expected: &H3ReferenceCellRow,
) -> Result<(), H3ReferenceInstallError> {
    let exact = actual.cell_id == expected.cell_id()
        && actual.resolution == expected.resolution()
        && actual.immediate_parent == expected.immediate_parent()
        && actual.ancestor_r4 == expected.ancestor_r4()
        && actual.ancestor_r5 == expected.ancestor_r5()
        && actual.ancestor_r6 == expected.ancestor_r6()
        && actual.ancestor_r7 == expected.ancestor_r7()
        && actual.origin == expected.origin();
    if exact {
        Ok(())
    } else {
        Err(conflict(H3ReferenceInstallConflict::Membership))
    }
}

fn attempt_install_transaction(
    client: &mut Client,
    bundle: &MichiganH3ReferenceBundle<'_>,
    attempt: usize,
) -> Result<CommitAttempt, H3ReferenceInstallError> {
    debug_assert!(attempt > 0, "install attempts must be one-based");
    let transaction = prepare_install_transaction(
        client,
        bundle,
        H3ReferenceMembershipReadContext::CommitAttempt { attempt },
    )?;
    match transaction.commit() {
        Ok(()) => Ok(CommitAttempt::Committed),
        Err(error) if error.as_db_error().is_some() => Err(postgres_database_error(
            H3ReferenceInstallOperation::CommitTransaction,
            &error,
        )),
        Err(_) => Ok(CommitAttempt::Ambiguous),
    }
}

fn prepare_install_transaction<'client>(
    client: &'client mut Client,
    bundle: &MichiganH3ReferenceBundle<'_>,
    context: H3ReferenceMembershipReadContext,
) -> Result<Transaction<'client>, H3ReferenceInstallError> {
    let mut transaction = client
        .build_transaction()
        .isolation_level(IsolationLevel::Serializable)
        .read_only(false)
        .start()
        .map_err(|error| {
            postgres_database_error(H3ReferenceInstallOperation::BeginTransaction, &error)
        })?;
    let verification = install_and_verify(&mut transaction, bundle, context);
    if let Err(primary) = verification {
        return rollback_preserving(transaction, primary);
    }
    Ok(transaction)
}

fn install_and_verify(
    transaction: &mut Transaction<'_>,
    bundle: &MichiganH3ReferenceBundle<'_>,
    context: H3ReferenceMembershipReadContext,
) -> Result<(), H3ReferenceInstallError> {
    prepare_transaction(transaction)?;
    insert_cells(transaction, bundle.cohort.rows())?;
    insert_r8_cells(transaction, bundle.r8_rows())?;
    insert_header(transaction, bundle.cohort.receipt())?;
    insert_membership(transaction, bundle.cohort)?;
    insert_r8_product(transaction, bundle)?;
    match inspect_presence(transaction, bundle, context)? {
        InstallPresence::Exact => Ok(()),
        InstallPresence::Absent => Err(conflict(H3ReferenceInstallConflict::R8ProductReceipt)),
    }
}

fn prepare_transaction(transaction: &mut Transaction<'_>) -> Result<(), H3ReferenceInstallError> {
    transaction
        .batch_execute(WRITE_LOCAL_SETTINGS_SQL)
        .map_err(|error| {
            postgres_database_error(H3ReferenceInstallOperation::SetTransactionSettings, &error)
        })?;
    let operation = H3ReferenceInstallOperation::VerifyTransactionSettings;
    let row = transaction
        .query_one(WRITE_SETTINGS_SQL, &[])
        .map_err(|error| postgres_database_error(operation, &error))?;
    let expected = [
        "serializable",
        "off",
        "pg_catalog",
        "on",
        "30s",
        "5s",
        "5s",
        "off",
        "off",
        "off",
    ];
    for (index, wanted) in expected.iter().enumerate() {
        let actual: String = decode_value(&row, index, operation)?;
        if actual != *wanted {
            return Err(database_error(operation));
        }
    }
    Ok(())
}

fn insert_cells(
    transaction: &mut Transaction<'_>,
    rows: &[H3ReferenceCellRow],
) -> Result<(), H3ReferenceInstallError> {
    let batch_count = rows.len().div_ceil(H3_REFERENCE_INSTALL_BATCH_ROWS);
    if batch_count > MAX_H3_REFERENCE_INSTALL_BATCHES {
        return Err(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::InsertBatches,
            actual: batch_count,
            max: MAX_H3_REFERENCE_INSTALL_BATCHES,
        });
    }
    for batch in rows
        .chunks(H3_REFERENCE_INSTALL_BATCH_ROWS)
        .take(MAX_H3_REFERENCE_INSTALL_BATCHES)
    {
        insert_cell_batch(transaction, batch)?;
    }
    Ok(())
}

fn insert_cell_batch(
    transaction: &mut Transaction<'_>,
    rows: &[H3ReferenceCellRow],
) -> Result<(), H3ReferenceInstallError> {
    let mut cell_ids = Vec::with_capacity(rows.len());
    let mut resolutions = Vec::with_capacity(rows.len());
    let mut immediate_parents = Vec::with_capacity(rows.len());
    let mut ancestors_r4 = Vec::with_capacity(rows.len());
    let mut ancestors_r5 = Vec::with_capacity(rows.len());
    let mut ancestors_r6 = Vec::with_capacity(rows.len());
    let mut ancestors_r7 = Vec::with_capacity(rows.len());
    for row in rows.iter().take(H3_REFERENCE_INSTALL_BATCH_ROWS) {
        cell_ids.push(cell_to_sql(
            row.cell_id(),
            H3ReferenceInstallOperation::InsertCells,
        )?);
        resolutions.push(i16::from(row.resolution()));
        immediate_parents.push(optional_cell_to_sql(
            row.immediate_parent(),
            H3ReferenceInstallOperation::InsertCells,
        )?);
        ancestors_r4.push(optional_cell_to_sql(
            row.ancestor_r4(),
            H3ReferenceInstallOperation::InsertCells,
        )?);
        ancestors_r5.push(optional_cell_to_sql(
            row.ancestor_r5(),
            H3ReferenceInstallOperation::InsertCells,
        )?);
        ancestors_r6.push(optional_cell_to_sql(
            row.ancestor_r6(),
            H3ReferenceInstallOperation::InsertCells,
        )?);
        ancestors_r7.push(optional_cell_to_sql(
            row.ancestor_r7(),
            H3ReferenceInstallOperation::InsertCells,
        )?);
    }
    transaction
        .execute(
            INSERT_CELLS_SQL,
            &[
                &cell_ids,
                &resolutions,
                &immediate_parents,
                &ancestors_r4,
                &ancestors_r5,
                &ancestors_r6,
                &ancestors_r7,
            ],
        )
        .map_err(|error| {
            postgres_database_error(H3ReferenceInstallOperation::InsertCells, &error)
        })?;
    Ok(())
}

fn insert_r8_cells(
    transaction: &mut Transaction<'_>,
    rows: &[MichiganH3R8ChildParent],
) -> Result<(), H3ReferenceInstallError> {
    let batch_count = rows.len().div_ceil(H3_REFERENCE_INSTALL_BATCH_ROWS);
    if rows.len() != MICHIGAN_R8_CHILD_ROWS || batch_count > MAX_MICHIGAN_R8_INSTALL_BATCHES {
        return Err(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::R8Rows,
            actual: rows.len(),
            max: MICHIGAN_R8_CHILD_ROWS,
        });
    }
    for batch in rows
        .chunks(H3_REFERENCE_INSTALL_BATCH_ROWS)
        .take(MAX_MICHIGAN_R8_INSTALL_BATCHES)
    {
        insert_r8_cell_batch(transaction, batch)?;
    }
    Ok(())
}

fn insert_r8_cell_batch(
    transaction: &mut Transaction<'_>,
    rows: &[MichiganH3R8ChildParent],
) -> Result<(), H3ReferenceInstallError> {
    let operation = H3ReferenceInstallOperation::InsertR8Cells;
    let mut cell_ids = Vec::with_capacity(rows.len());
    let mut resolutions = Vec::with_capacity(rows.len());
    let mut immediate_parents = Vec::with_capacity(rows.len());
    let mut ancestors_r4 = Vec::with_capacity(rows.len());
    let mut ancestors_r5 = Vec::with_capacity(rows.len());
    let mut ancestors_r6 = Vec::with_capacity(rows.len());
    let mut ancestors_r7 = Vec::with_capacity(rows.len());
    for row in rows.iter().take(H3_REFERENCE_INSTALL_BATCH_ROWS) {
        let child = row.child_cell_id();
        let parent = row.parent_r7_cell_id();
        cell_ids.push(cell_to_sql(child, operation)?);
        resolutions.push(8_i16);
        immediate_parents.push(Some(cell_to_sql(parent, operation)?));
        ancestors_r4.push(Some(cell_to_sql(
            ancestor_at(child, 4, operation)?,
            operation,
        )?));
        ancestors_r5.push(Some(cell_to_sql(
            ancestor_at(child, 5, operation)?,
            operation,
        )?));
        ancestors_r6.push(Some(cell_to_sql(
            ancestor_at(child, 6, operation)?,
            operation,
        )?));
        ancestors_r7.push(Some(cell_to_sql(parent, operation)?));
    }
    transaction
        .execute(
            INSERT_CELLS_SQL,
            &[
                &cell_ids,
                &resolutions,
                &immediate_parents,
                &ancestors_r4,
                &ancestors_r5,
                &ancestors_r6,
                &ancestors_r7,
            ],
        )
        .map_err(|error| postgres_database_error(operation, &error))?;
    Ok(())
}

fn insert_header(
    transaction: &mut Transaction<'_>,
    receipt: &H3ReferenceCohortReceipt,
) -> Result<(), H3ReferenceInstallError> {
    let header = expected_header(receipt)?;
    let direct_count = count_to_sql(header.direct_cell_count)?;
    let derived_count = count_to_sql(header.derived_ancestor_count)?;
    let closure_count = count_to_sql(header.closure_cell_count)?;
    transaction
        .execute(
            INSERT_HEADER_SQL,
            &[
                &header.ref_digest.as_bytes().as_slice(),
                &header.format_version,
                &header.artifact_name,
                &header.artifact_manifest_version,
                &header.artifact_digest.as_bytes().as_slice(),
                &header.source_digest.as_bytes().as_slice(),
                &header.source_r5_digest.as_bytes().as_slice(),
                &header.source_r7_digest.as_bytes().as_slice(),
                &header.closure_digest.as_bytes().as_slice(),
                &header.membership_digest.as_bytes().as_slice(),
                &direct_count,
                &derived_count,
                &closure_count,
            ],
        )
        .map_err(|error| {
            postgres_database_error(H3ReferenceInstallOperation::InsertHeader, &error)
        })?;
    Ok(())
}

fn insert_membership(
    transaction: &mut Transaction<'_>,
    cohort: &H3ReferenceCohort,
) -> Result<(), H3ReferenceInstallError> {
    let rows = cohort.rows();
    let batch_count = rows.len().div_ceil(H3_REFERENCE_INSTALL_BATCH_ROWS);
    if batch_count > MAX_H3_REFERENCE_INSTALL_BATCHES {
        return Err(H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::InsertBatches,
            actual: batch_count,
            max: MAX_H3_REFERENCE_INSTALL_BATCHES,
        });
    }
    for batch in rows
        .chunks(H3_REFERENCE_INSTALL_BATCH_ROWS)
        .take(MAX_H3_REFERENCE_INSTALL_BATCHES)
    {
        insert_membership_batch(transaction, cohort.receipt().ref_digest(), batch)?;
    }
    Ok(())
}

fn insert_membership_batch(
    transaction: &mut Transaction<'_>,
    ref_digest: RefDigest,
    rows: &[H3ReferenceCellRow],
) -> Result<(), H3ReferenceInstallError> {
    let mut cell_ids = Vec::with_capacity(rows.len());
    let mut origins = Vec::with_capacity(rows.len());
    for row in rows.iter().take(H3_REFERENCE_INSTALL_BATCH_ROWS) {
        cell_ids.push(cell_to_sql(
            row.cell_id(),
            H3ReferenceInstallOperation::InsertMembership,
        )?);
        origins.push(i16::from(row.origin().code()));
    }
    transaction
        .execute(
            INSERT_MEMBERSHIP_SQL,
            &[&ref_digest.as_bytes().as_slice(), &cell_ids, &origins],
        )
        .map_err(|error| {
            postgres_database_error(H3ReferenceInstallOperation::InsertMembership, &error)
        })?;
    Ok(())
}

fn insert_r8_product(
    transaction: &mut Transaction<'_>,
    bundle: &MichiganH3ReferenceBundle<'_>,
) -> Result<(), H3ReferenceInstallError> {
    let operation = H3ReferenceInstallOperation::InsertR8Product;
    let ref_digest = bundle.cohort.receipt().ref_digest();
    let row_count =
        i64::try_from(bundle.r8_rows().len()).map_err(|_| H3ReferenceInstallError::Bounds {
            resource: H3ReferenceInstallBoundedResource::R8Rows,
            actual: bundle.r8_rows().len(),
            max: MICHIGAN_R8_CHILD_ROWS,
        })?;
    transaction
        .execute(
            INSERT_R8_PRODUCT_SQL,
            &[
                &ref_digest.as_bytes().as_slice(),
                &MICHIGAN_R8_PRODUCT_CODE,
                &bundle.r8_child_parent_digest.as_bytes().as_slice(),
                &row_count,
            ],
        )
        .map_err(|error| postgres_database_error(operation, &error))?;
    Ok(())
}

fn rollback_preserving<T>(
    transaction: Transaction<'_>,
    primary: H3ReferenceInstallError,
) -> Result<T, H3ReferenceInstallError> {
    let rollback = transaction.rollback().map_err(|error| {
        postgres_database_error(H3ReferenceInstallOperation::RollbackTransaction, &error)
    });
    preserve_rollback_result(primary, rollback)
}

fn preserve_rollback_result<T>(
    primary: H3ReferenceInstallError,
    rollback: Result<(), H3ReferenceInstallError>,
) -> Result<T, H3ReferenceInstallError> {
    match rollback {
        Ok(()) => Err(primary),
        Err(rollback) => Err(H3ReferenceInstallError::FailureAndRollback {
            primary: Box::new(primary),
            rollback: Box::new(rollback),
        }),
    }
}

fn decode_value<T>(
    row: &Row,
    index: usize,
    operation: H3ReferenceInstallOperation,
) -> Result<T, H3ReferenceInstallError>
where
    T: postgres::types::FromSqlOwned,
{
    row.try_get(index)
        .map_err(|_| H3ReferenceInstallError::Decode { operation })
}

fn decode_digest(
    row: &Row,
    index: usize,
    operation: H3ReferenceInstallOperation,
) -> Result<RefDigest, H3ReferenceInstallError> {
    let raw: Vec<u8> = decode_value(row, index, operation)?;
    let bytes = <[u8; 32]>::try_from(raw.as_slice())
        .map_err(|_| H3ReferenceInstallError::Decode { operation })?;
    Ok(RefDigest::from_bytes(bytes))
}

fn decode_optional_digest(
    row: &Row,
    index: usize,
    operation: H3ReferenceInstallOperation,
) -> Result<Option<RefDigest>, H3ReferenceInstallError> {
    let raw: Option<Vec<u8>> = decode_value(row, index, operation)?;
    raw.map(|value| {
        let bytes = <[u8; 32]>::try_from(value.as_slice())
            .map_err(|_| H3ReferenceInstallError::Decode { operation })?;
        Ok(RefDigest::from_bytes(bytes))
    })
    .transpose()
}

fn decode_count(
    row: &Row,
    index: usize,
    operation: H3ReferenceInstallOperation,
) -> Result<usize, H3ReferenceInstallError> {
    let raw: i64 = decode_value(row, index, operation)?;
    usize::try_from(raw).map_err(|_| H3ReferenceInstallError::Decode { operation })
}

fn decode_optional_cell(
    row: &Row,
    index: usize,
    operation: H3ReferenceInstallOperation,
) -> Result<Option<H3CellId>, H3ReferenceInstallError> {
    let raw: Option<i64> = decode_value(row, index, operation)?;
    raw.map(H3CellId::try_from)
        .transpose()
        .map_err(|source| H3ReferenceInstallError::CellIdentity { operation, source })
}

fn decode_cell(
    row: &Row,
    index: usize,
    operation: H3ReferenceInstallOperation,
) -> Result<H3CellId, H3ReferenceInstallError> {
    let raw: i64 = decode_value(row, index, operation)?;
    H3CellId::try_from(raw)
        .map_err(|source| H3ReferenceInstallError::CellIdentity { operation, source })
}

fn ancestor_at(
    cell: H3CellId,
    resolution: u8,
    operation: H3ReferenceInstallOperation,
) -> Result<H3CellId, H3ReferenceInstallError> {
    cell.ancestor_at(resolution)
        .map_err(|source| H3ReferenceInstallError::CellIdentity { operation, source })
}

fn decode_origin(
    row: &Row,
    index: usize,
    operation: H3ReferenceInstallOperation,
) -> Result<H3ReferenceOrigin, H3ReferenceInstallError> {
    let raw: i16 = decode_value(row, index, operation)?;
    match raw {
        1 => Ok(H3ReferenceOrigin::Direct),
        2 => Ok(H3ReferenceOrigin::DerivedAncestor),
        _ => Err(H3ReferenceInstallError::Decode { operation }),
    }
}

fn cell_to_sql(
    cell: H3CellId,
    operation: H3ReferenceInstallOperation,
) -> Result<i64, H3ReferenceInstallError> {
    i64::try_from(cell)
        .map_err(|source| H3ReferenceInstallError::CellIdentity { operation, source })
}

fn optional_cell_to_sql(
    cell: Option<H3CellId>,
    operation: H3ReferenceInstallOperation,
) -> Result<Option<i64>, H3ReferenceInstallError> {
    cell.map(|value| cell_to_sql(value, operation)).transpose()
}

fn count_to_sql(count: usize) -> Result<i64, H3ReferenceInstallError> {
    i64::try_from(count).map_err(|_| H3ReferenceInstallError::Bounds {
        resource: H3ReferenceInstallBoundedResource::MembershipRows,
        actual: count,
        max: MAX_H3_REFERENCE_CLOSURE_ROWS,
    })
}

fn database_error(operation: H3ReferenceInstallOperation) -> H3ReferenceInstallError {
    H3ReferenceInstallError::Database {
        operation,
        diagnostic: None,
    }
}

fn postgres_database_error(
    operation: H3ReferenceInstallOperation,
    error: &postgres::Error,
) -> H3ReferenceInstallError {
    H3ReferenceInstallError::Database {
        operation,
        diagnostic: Some(PostgresDiagnostic::capture(error)),
    }
}

fn conflict(component: H3ReferenceInstallConflict) -> H3ReferenceInstallError {
    H3ReferenceInstallError::Conflict { component }
}

#[cfg(test)]
pub(crate) mod live_postgres_tests {
    use std::io::Write as _;
    use std::time::Instant;

    use babylon_kernel::tick_content_hash::RefDigest;
    use postgres::{error::SqlState, Config, NoTls};

    use super::{
        attempt_install_transaction, conflict, install_michigan_h3_reference_bundle,
        install_michigan_h3_reference_bundle_using, prepare_install_transaction, CommitAttempt,
        H3ReferenceInstallBoundedResource, H3ReferenceInstallConflict,
        H3ReferenceInstallDisposition, H3ReferenceInstallError, H3ReferenceMembershipReadContext,
        MichiganH3ReferenceBundle,
    };
    use crate::{
        h3_reference_cohort::representative_h3_reference_cohort,
        h3_reference_cohort::H3ReferenceCohort, michigan_dynamic_hex_foundation,
    };

    const BACKEND_TERMINATION_TIMEOUT_MILLIS: i64 = 5_000;
    const CLOSURE_COUNT: usize = 59_849;
    const EXCESS_MEMBERSHIP_COUNT: usize = 2;
    const RESOLUTION_ZERO_CANDIDATES: [i64; 12] = [
        0x0800_9fff_ffff_ffff,
        0x0801_dfff_ffff_ffff,
        0x0803_1fff_ffff_ffff,
        0x0804_dfff_ffff_ffff,
        0x0806_3fff_ffff_ffff,
        0x0807_5fff_ffff_ffff,
        0x0807_ffff_ffff_ffff,
        0x0809_1fff_ffff_ffff,
        0x080a_7fff_ffff_ffff,
        0x080c_3fff_ffff_ffff,
        0x080d_7fff_ffff_ffff,
        0x080e_bfff_ffff_ffff,
    ];
    #[derive(Debug, Clone, Copy)]
    enum H3AtomicityPhase {
        ForcedRollback,
        KilledRetry,
        CommittedReconciliation,
    }

    impl H3AtomicityPhase {
        const fn label(self) -> &'static str {
            match self {
                Self::ForcedRollback => "forced_rollback",
                Self::KilledRetry => "killed_retry",
                Self::CommittedReconciliation => "committed_reconciliation",
            }
        }
    }

    fn start_phase(phase: H3AtomicityPhase, suite_started: Instant) -> Instant {
        eprintln!(
            "PER267_H3_PHASE phase={} completion=started phase_ms=0 cumulative_ms={}",
            phase.label(),
            suite_started.elapsed().as_millis()
        );
        std::io::stderr()
            .flush()
            .expect("PER-267 phase receipt must flush");
        Instant::now()
    }

    fn finish_phase(phase: H3AtomicityPhase, phase_started: Instant, suite_started: Instant) {
        eprintln!(
            "PER267_H3_PHASE phase={} completion=complete phase_ms={} cumulative_ms={}",
            phase.label(),
            phase_started.elapsed().as_millis(),
            suite_started.elapsed().as_millis()
        );
        std::io::stderr()
            .flush()
            .expect("PER-267 phase receipt must flush");
    }

    pub(crate) fn verify_rollback_and_killed_retry(
        config: &Config,
        admin: &Config,
        suite_started: Instant,
    ) {
        let cohort = representative_cohort();
        let bundle = representative_bundle(&cohort);
        assert_eq!(reference_counts(config), (0, 0, 0, 0));
        verify_membership_read_is_join_plan_independent(config, &cohort);
        assert_eq!(reference_counts(config), (0, 0, 0, 0));
        assert_eq!(reference_counts(config), (0, 0, 0, 0));
        let forced_rollback_started = start_phase(H3AtomicityPhase::ForcedRollback, suite_started);
        let mut forced_failure = |client: &mut postgres::Client,
                                  bundle: &MichiganH3ReferenceBundle<'_>,
                                  attempt: usize| {
            let transaction = prepare_install_transaction(
                client,
                bundle,
                H3ReferenceMembershipReadContext::CommitAttempt { attempt },
            )?;
            super::rollback_preserving(
                transaction,
                conflict(H3ReferenceInstallConflict::Membership),
            )
        };
        assert!(matches!(
            install_michigan_h3_reference_bundle_using(config, &bundle, &mut forced_failure),
            Err(super::H3ReferenceInstallError::Conflict {
                component: H3ReferenceInstallConflict::Membership
            })
        ));
        assert_eq!(reference_counts(config), (0, 0, 0, 0));
        finish_phase(
            H3AtomicityPhase::ForcedRollback,
            forced_rollback_started,
            suite_started,
        );

        let killed_retry_started = start_phase(H3AtomicityPhase::KilledRetry, suite_started);
        let mut first_attempt = true;
        let mut killed_attempt = |client: &mut postgres::Client,
                                  bundle: &MichiganH3ReferenceBundle<'_>,
                                  attempt: usize| {
            if first_attempt {
                first_attempt = false;
                killed_before_commit(client, bundle, admin, attempt)
            } else {
                attempt_install_transaction(client, bundle, attempt)
            }
        };
        let report =
            install_michigan_h3_reference_bundle_using(config, &bundle, &mut killed_attempt)
                .unwrap();
        assert_eq!(
            report.disposition(),
            H3ReferenceInstallDisposition::Installed
        );
        assert_eq!(report.commit_attempts(), 2);
        assert_eq!(reference_counts(config), (378_853, 1, 59_849, 1));
        finish_phase(
            H3AtomicityPhase::KilledRetry,
            killed_retry_started,
            suite_started,
        );
        verify_cell_lock_timeout_preserves_server_diagnostic(config, &cohort);
    }

    fn verify_membership_read_is_join_plan_independent(
        config: &Config,
        cohort: &H3ReferenceCohort,
    ) {
        let bounded = super::installer_config(config);
        let mut session = super::LockedInstallSession::connect(&bounded).unwrap();
        super::require_schema(session.client()).unwrap();
        super::prepare_installer_session(session.client()).unwrap();
        let mut transaction = session
            .client()
            .build_transaction()
            .isolation_level(postgres::IsolationLevel::Serializable)
            .read_only(false)
            .start()
            .unwrap();
        super::prepare_transaction(&mut transaction).unwrap();
        transaction
            .batch_execute(
                "SET LOCAL statement_timeout TO '5000ms'; \
                 SET LOCAL enable_indexscan TO off; \
                 SET LOCAL enable_indexonlyscan TO off; \
                 SET LOCAL enable_bitmapscan TO off; \
                 SET LOCAL enable_hashjoin TO off; \
                 SET LOCAL enable_mergejoin TO off",
            )
            .unwrap();
        super::insert_cells(&mut transaction, cohort.rows()).unwrap();
        super::insert_header(&mut transaction, cohort.receipt()).unwrap();
        super::insert_membership(&mut transaction, cohort).unwrap();
        super::verify_membership(
            &mut transaction,
            cohort,
            H3ReferenceMembershipReadContext::CommitAttempt { attempt: 1 },
        )
        .unwrap();
        transaction.rollback().unwrap();
        session.finish(Ok(())).unwrap();
    }

    pub(crate) fn verify_committed_reconciliation(
        config: &Config,
        admin: &Config,
        suite_started: Instant,
    ) {
        let phase_started = start_phase(H3AtomicityPhase::CommittedReconciliation, suite_started);
        let cohort = representative_cohort();
        let bundle = representative_bundle(&cohort);
        let mut first_attempt = true;
        let mut ambiguous_attempt = |client: &mut postgres::Client,
                                     bundle: &MichiganH3ReferenceBundle<'_>,
                                     attempt: usize| {
            let backend_pid = backend_pid(client);
            let outcome = attempt_install_transaction(client, bundle, attempt)?;
            if first_attempt {
                first_attempt = false;
                assert_eq!(outcome, CommitAttempt::Committed);
                terminate_backend(admin, backend_pid);
                Ok(CommitAttempt::Ambiguous)
            } else {
                Ok(outcome)
            }
        };
        let report =
            install_michigan_h3_reference_bundle_using(config, &bundle, &mut ambiguous_attempt)
                .unwrap();
        assert_eq!(
            report.disposition(),
            H3ReferenceInstallDisposition::ReconciledAfterAmbiguousCommit
        );
        assert_eq!(report.commit_attempts(), 1);
        assert_eq!(reference_counts(config), (378_853, 1, 59_849, 1));
        finish_phase(
            H3AtomicityPhase::CommittedReconciliation,
            phase_started,
            suite_started,
        );
    }

    fn verify_cell_lock_timeout_preserves_server_diagnostic(
        config: &Config,
        cohort: &H3ReferenceCohort,
    ) {
        let before = reference_counts(config);
        let bounded = super::installer_config(config);
        let mut session = super::LockedInstallSession::connect(&bounded).unwrap();
        super::require_schema(session.client()).unwrap();
        super::prepare_installer_session(session.client()).unwrap();
        let context = H3ReferenceMembershipReadContext::InitialInspection;
        super::verify_membership_cardinality(session.client(), cohort, context).unwrap();

        let mut blocker = config.connect(NoTls).unwrap();
        let mut blocker_transaction = blocker.transaction().unwrap();
        blocker_transaction
            .batch_execute("LOCK TABLE babylon_ref.h3_cell IN ACCESS EXCLUSIVE MODE")
            .unwrap();
        let operation = super::H3ReferenceInstallOperation::ReadCellRows { context };
        let result = super::read_cell_rows(session.client(), cohort, operation);
        blocker_transaction.rollback().unwrap();
        session.finish(Ok(())).unwrap();

        let error = result.expect_err("the cell read must honor the bounded lock timeout");
        match error {
            super::H3ReferenceInstallError::Database {
                operation:
                    super::H3ReferenceInstallOperation::ReadCellRows {
                        context: H3ReferenceMembershipReadContext::InitialInspection,
                    },
                diagnostic,
            } => {
                let server = diagnostic
                    .as_ref()
                    .expect("the cell query must retain its server diagnostic");
                assert_eq!(server.sqlstate(), Some(SqlState::LOCK_NOT_AVAILABLE.code()));
                assert_eq!(
                    server.classification(),
                    crate::PostgresFailureClass::Timeout
                );
                assert!(server.message().is_some_and(|message| !message.is_empty()));
            }
            other => panic!("unexpected cell lock-timeout error: {other:?}"),
        }
        assert_eq!(reference_counts(config), before);

        let reacquired = super::LockedInstallSession::connect(&bounded).unwrap();
        reacquired.finish(Ok(())).unwrap();
    }

    pub(crate) fn verify_membership_cardinality_bound(config: &Config) {
        let cohort = representative_cohort();
        let foundation = michigan_dynamic_hex_foundation().unwrap();
        let report = install_michigan_h3_reference_bundle(config, &cohort, foundation).unwrap();
        assert_eq!(
            report.disposition(),
            H3ReferenceInstallDisposition::Installed
        );
        insert_excess_membership(config, cohort.receipt().ref_digest());
        let excess_count = i64::try_from(CLOSURE_COUNT + EXCESS_MEMBERSHIP_COUNT).unwrap();
        assert_eq!(
            reference_counts(config),
            (
                378_853 + i64::try_from(EXCESS_MEMBERSHIP_COUNT).unwrap(),
                1,
                excess_count,
                1
            )
        );
        assert_eq!(
            install_michigan_h3_reference_bundle(config, &cohort, foundation),
            Err(H3ReferenceInstallError::Bounds {
                resource: H3ReferenceInstallBoundedResource::MembershipRows,
                actual: CLOSURE_COUNT + 1,
                max: CLOSURE_COUNT,
            })
        );
    }

    fn insert_excess_membership(config: &Config, ref_digest: RefDigest) {
        let mut client = config.connect(NoTls).unwrap();
        let mut transaction = client.transaction().unwrap();
        let rows = transaction
            .query(
                "INSERT INTO babylon_ref.h3_cell \
                 (cell_id, resolution, immediate_parent, ancestor_r4, ancestor_r5, \
                  ancestor_r6, ancestor_r7) \
                 SELECT candidate.cell_id, 0, NULL, NULL, NULL, NULL, NULL \
                 FROM pg_catalog.unnest($1::bigint[]) AS candidate(cell_id) \
                 WHERE NOT EXISTS (SELECT 1 FROM babylon_ref.h3_cell AS existing \
                                   WHERE existing.cell_id = candidate.cell_id) \
                 ORDER BY candidate.cell_id LIMIT $2 RETURNING cell_id",
                &[
                    &RESOLUTION_ZERO_CANDIDATES.as_slice(),
                    &i64::try_from(EXCESS_MEMBERSHIP_COUNT).unwrap(),
                ],
            )
            .unwrap();
        assert_eq!(rows.len(), EXCESS_MEMBERSHIP_COUNT);
        let inserted = rows
            .iter()
            .take(EXCESS_MEMBERSHIP_COUNT)
            .map(|row| row.get::<_, i64>(0))
            .collect::<Vec<_>>();
        let memberships = transaction
            .execute(
                "INSERT INTO babylon_ref.h3_reference_membership \
                 (ref_digest, cell_id, origin) \
                 SELECT $1, input.cell_id, 2 \
                 FROM pg_catalog.unnest($2::bigint[]) AS input(cell_id)",
                &[&ref_digest.as_bytes().as_slice(), &inserted],
            )
            .unwrap();
        assert_eq!(
            usize::try_from(memberships).unwrap(),
            EXCESS_MEMBERSHIP_COUNT
        );
        transaction.commit().unwrap();
    }

    fn killed_before_commit(
        client: &mut postgres::Client,
        bundle: &MichiganH3ReferenceBundle<'_>,
        admin: &Config,
        attempt: usize,
    ) -> Result<CommitAttempt, super::H3ReferenceInstallError> {
        let backend_pid = backend_pid(client);
        let transaction = prepare_install_transaction(
            client,
            bundle,
            H3ReferenceMembershipReadContext::CommitAttempt { attempt },
        )?;
        terminate_backend(admin, backend_pid);
        assert!(transaction.commit().is_err());
        Ok(CommitAttempt::Ambiguous)
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

    fn reference_counts(config: &Config) -> (i64, i64, i64, i64) {
        let mut client = config.connect(NoTls).unwrap();
        let row = client
            .query_one(
                "SELECT (SELECT pg_catalog.count(*) FROM babylon_ref.h3_cell), \
                        (SELECT pg_catalog.count(*) FROM babylon_ref.h3_reference_cohort), \
                        (SELECT pg_catalog.count(*) \
                         FROM babylon_ref.h3_reference_membership), \
                        (SELECT pg_catalog.count(*) FROM babylon_ref.reference_product)",
                &[],
            )
            .unwrap();
        (row.get(0), row.get(1), row.get(2), row.get(3))
    }

    fn representative_cohort() -> H3ReferenceCohort {
        representative_h3_reference_cohort()
            .expect("the sole checked-in source fixture must validate")
            .clone()
    }

    fn representative_bundle(cohort: &H3ReferenceCohort) -> MichiganH3ReferenceBundle<'_> {
        MichiganH3ReferenceBundle::validate(cohort, michigan_dynamic_hex_foundation().unwrap())
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        database_error, drive_install, installer_config, preserve_rollback_result, CommitAttempt,
        InstallDriver, InstallPresence, H3_REFERENCE_SESSION_SETTINGS_SQL,
        MAX_H3_REFERENCE_INSTALL_COMMIT_ATTEMPTS,
    };
    use crate::{
        postgres_catalog::CATALOG_CONNECT_TIMEOUT, postgres_catalog::CATALOG_STARTUP_OPTIONS,
        postgres_catalog::CATALOG_TCP_USER_TIMEOUT, H3ReferenceInstallConflict,
        H3ReferenceInstallDisposition, H3ReferenceInstallError, H3ReferenceInstallOperation,
    };
    use postgres::Config;
    use std::time::Duration;

    struct ScriptedDriver {
        attempts: [Option<CommitAttempt>; MAX_H3_REFERENCE_INSTALL_COMMIT_ATTEMPTS],
        reconciliations: [Option<InstallPresence>; MAX_H3_REFERENCE_INSTALL_COMMIT_ATTEMPTS],
        reconciliation_failure: Option<H3ReferenceInstallError>,
        attempt_calls: usize,
        reconciliation_calls: usize,
        received_attempts: [usize; MAX_H3_REFERENCE_INSTALL_COMMIT_ATTEMPTS],
        received_reconciliations: [usize; MAX_H3_REFERENCE_INSTALL_COMMIT_ATTEMPTS],
    }

    impl ScriptedDriver {
        fn new(
            attempts: [Option<CommitAttempt>; MAX_H3_REFERENCE_INSTALL_COMMIT_ATTEMPTS],
            reconciliations: [Option<InstallPresence>; MAX_H3_REFERENCE_INSTALL_COMMIT_ATTEMPTS],
        ) -> Self {
            Self {
                attempts,
                reconciliations,
                reconciliation_failure: None,
                attempt_calls: 0,
                reconciliation_calls: 0,
                received_attempts: [0; MAX_H3_REFERENCE_INSTALL_COMMIT_ATTEMPTS],
                received_reconciliations: [0; MAX_H3_REFERENCE_INSTALL_COMMIT_ATTEMPTS],
            }
        }

        fn with_reconciliation_failure(mut self, failure: H3ReferenceInstallError) -> Self {
            self.reconciliation_failure = Some(failure);
            self
        }
    }

    impl InstallDriver for ScriptedDriver {
        fn attempt_commit(
            &mut self,
            attempt: usize,
        ) -> Result<CommitAttempt, H3ReferenceInstallError> {
            let index = self.attempt_calls;
            self.received_attempts[index] = attempt;
            self.attempt_calls = self
                .attempt_calls
                .checked_add(1)
                .expect("bounded call count");
            Ok(self.attempts[index].expect("unexpected commit attempt"))
        }

        fn reconcile(
            &mut self,
            after_attempt: usize,
        ) -> Result<InstallPresence, H3ReferenceInstallError> {
            let index = self.reconciliation_calls;
            self.received_reconciliations[index] = after_attempt;
            self.reconciliation_calls = self
                .reconciliation_calls
                .checked_add(1)
                .expect("bounded call count");
            if let Some(failure) = self.reconciliation_failure.take() {
                return Err(failure);
            }
            Ok(self.reconciliations[index].expect("unexpected reconciliation"))
        }
    }

    #[test]
    fn installer_preserves_epoch_bounds_before_its_closed_session_widening() {
        let mut caller = Config::new();
        caller
            .host("127.0.0.1")
            .connect_timeout(Duration::from_millis(1))
            .tcp_user_timeout(Duration::from_millis(1))
            .options("-c statement_timeout=1ms -c search_path=public");

        let bounded = installer_config(&caller);

        assert_eq!(bounded.get_options(), Some(CATALOG_STARTUP_OPTIONS));
        assert_eq!(
            bounded.get_connect_timeout().copied(),
            Some(CATALOG_CONNECT_TIMEOUT)
        );
        assert_eq!(
            bounded.get_tcp_user_timeout().copied(),
            Some(CATALOG_TCP_USER_TIMEOUT)
        );
        assert_eq!(
            H3_REFERENCE_SESSION_SETTINGS_SQL,
            "SET statement_timeout TO '30000ms'"
        );
    }

    #[test]
    fn committed_install_reports_installed_after_one_attempt() {
        let mut driver = ScriptedDriver::new([Some(CommitAttempt::Committed), None], [None, None]);
        let resolution = drive_install(InstallPresence::Absent, &mut driver).unwrap();

        assert_eq!(
            resolution.disposition,
            H3ReferenceInstallDisposition::Installed
        );
        assert_eq!(resolution.commit_attempts, 1);
        assert_eq!((driver.attempt_calls, driver.reconciliation_calls), (1, 0));
        assert_eq!(&driver.received_attempts[..driver.attempt_calls], &[1]);
        assert_eq!(
            &driver.received_reconciliations[..driver.reconciliation_calls],
            &[] as &[usize]
        );
    }

    #[test]
    fn exact_preexisting_reference_bundle_skips_commit() {
        let mut driver = ScriptedDriver::new([None, None], [None, None]);
        let resolution = drive_install(InstallPresence::Exact, &mut driver).unwrap();

        assert_eq!(
            resolution.disposition,
            H3ReferenceInstallDisposition::AlreadyPresent
        );
        assert_eq!(resolution.commit_attempts, 0);
        assert_eq!((driver.attempt_calls, driver.reconciliation_calls), (0, 0));
    }

    #[test]
    fn ambiguous_commit_reconciles_exact_reference_bundle_without_retry() {
        let mut driver = ScriptedDriver::new(
            [Some(CommitAttempt::Ambiguous), None],
            [Some(InstallPresence::Exact), None],
        );
        let resolution = drive_install(InstallPresence::Absent, &mut driver).unwrap();

        assert_eq!(
            resolution.disposition,
            H3ReferenceInstallDisposition::ReconciledAfterAmbiguousCommit
        );
        assert_eq!(resolution.commit_attempts, 1);
        assert_eq!((driver.attempt_calls, driver.reconciliation_calls), (1, 1));
        assert_eq!(&driver.received_attempts[..driver.attempt_calls], &[1]);
        assert_eq!(
            &driver.received_reconciliations[..driver.reconciliation_calls],
            &[1]
        );
    }

    #[test]
    fn ambiguous_absence_retries_once_then_installs() {
        let mut driver = ScriptedDriver::new(
            [
                Some(CommitAttempt::Ambiguous),
                Some(CommitAttempt::Committed),
            ],
            [Some(InstallPresence::Absent), None],
        );
        let resolution = drive_install(InstallPresence::Absent, &mut driver).unwrap();

        assert_eq!(
            resolution.disposition,
            H3ReferenceInstallDisposition::Installed
        );
        assert_eq!(resolution.commit_attempts, 2);
        assert_eq!((driver.attempt_calls, driver.reconciliation_calls), (2, 1));
        assert_eq!(&driver.received_attempts[..driver.attempt_calls], &[1, 2]);
        assert_eq!(
            &driver.received_reconciliations[..driver.reconciliation_calls],
            &[1]
        );
    }

    #[test]
    fn two_ambiguous_absences_refuse_without_third_attempt() {
        let mut driver = ScriptedDriver::new(
            [
                Some(CommitAttempt::Ambiguous),
                Some(CommitAttempt::Ambiguous),
            ],
            [Some(InstallPresence::Absent), Some(InstallPresence::Absent)],
        );
        let error = drive_install(InstallPresence::Absent, &mut driver).unwrap_err();

        assert_eq!(
            error,
            H3ReferenceInstallError::AmbiguousCommitUnresolved { attempts: 2 }
        );
        assert_eq!((driver.attempt_calls, driver.reconciliation_calls), (2, 2));
        assert_eq!(&driver.received_attempts[..driver.attempt_calls], &[1, 2]);
        assert_eq!(
            &driver.received_reconciliations[..driver.reconciliation_calls],
            &[1, 2]
        );
    }

    #[test]
    fn ambiguous_reconciliation_failure_preserves_commit_uncertainty() {
        let reconciliation = database_error(H3ReferenceInstallOperation::ReadHeader);
        let mut driver = ScriptedDriver::new([Some(CommitAttempt::Ambiguous), None], [None, None])
            .with_reconciliation_failure(reconciliation.clone());

        assert_eq!(
            drive_install(InstallPresence::Absent, &mut driver),
            Err(H3ReferenceInstallError::AmbiguousCommitAndReconciliation {
                attempts: 1,
                reconciliation: Box::new(reconciliation),
            })
        );
        assert_eq!((driver.attempt_calls, driver.reconciliation_calls), (1, 1));
        assert_eq!(&driver.received_attempts[..driver.attempt_calls], &[1]);
        assert_eq!(
            &driver.received_reconciliations[..driver.reconciliation_calls],
            &[1]
        );
    }

    #[test]
    fn rollback_failure_preserves_the_primary_install_failure() {
        let primary = H3ReferenceInstallError::Conflict {
            component: H3ReferenceInstallConflict::Membership,
        };
        let rollback = database_error(H3ReferenceInstallOperation::RollbackTransaction);

        assert_eq!(
            preserve_rollback_result::<()>(primary.clone(), Ok(())),
            Err(primary.clone())
        );
        assert_eq!(
            preserve_rollback_result::<()>(primary.clone(), Err(rollback.clone())),
            Err(H3ReferenceInstallError::FailureAndRollback {
                primary: Box::new(primary),
                rollback: Box::new(rollback),
            })
        );
    }
}
