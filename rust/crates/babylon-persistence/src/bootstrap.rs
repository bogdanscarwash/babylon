//! Restart-safe installation of the native schema and immutable H3 reference bundle.

use postgres::Config;

use crate::h3_reference_cohort::{representative_h3_reference_cohort_v1, H3ReferenceCohortError};
use crate::h3_reference_installer::{
    install_michigan_h3_reference_bundle_v1, H3ReferenceInstallError, H3ReferenceInstallReport,
};
use crate::michigan_dynamic_hex_foundation::{
    michigan_dynamic_hex_foundation_v1, MichiganDynamicHexFoundationDecodeErrorV1,
};
use crate::schema_epoch::{
    migrate_schema_epoch, SchemaEpochError, SchemaEpochReport, CURRENT_SCHEMA_EPOCH,
};

/// Receipts from the native schema and immutable reference installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct H3ReaderBootstrapReportV1 {
    /// Exact immutable Michigan H3 reference-bundle installation receipt.
    pub reference_bundle_installation: H3ReferenceInstallReport,
    /// Completed native schema construction.
    pub final_epoch: SchemaEpochReport,
}

/// Closed failure boundary for native H3 bootstrap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum H3ReaderBootstrapErrorV1 {
    /// The embedded H3 source fixture failed before database access.
    ReferenceCohort(H3ReferenceCohortError),
    /// The embedded Michigan foundation fixture failed before database access.
    ReferenceFoundation(MichiganDynamicHexFoundationDecodeErrorV1),
    /// The current schema could not be constructed or verified.
    SchemaEpoch(SchemaEpochError),
    /// The exact immutable reference bundle could not be installed.
    ReferenceInstall(H3ReferenceInstallError),
    /// Schema construction did not complete its compiled registry.
    UnexpectedSchemaEpoch { actual: usize },
}

impl std::fmt::Display for H3ReaderBootstrapErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "H3 reader bootstrap failed: {self:?}")
    }
}

impl std::error::Error for H3ReaderBootstrapErrorV1 {}

/// Validate source bytes, construct the current schema, and install its reference bundle.
///
/// Existing current schemas are verified before idempotent installation. Unrelated database
/// objects cannot enter the exact fresh/current schema census and are never adopted or deleted.
///
/// # Errors
/// Returns [`H3ReaderBootstrapErrorV1`] for invalid source data, a refused database shape,
/// incomplete schema construction, or a failed immutable reference installation.
pub fn bootstrap_h3_reader_epoch_v1(
    config: &Config,
) -> Result<H3ReaderBootstrapReportV1, H3ReaderBootstrapErrorV1> {
    let cohort = representative_h3_reference_cohort_v1()
        .map_err(H3ReaderBootstrapErrorV1::ReferenceCohort)?;
    let foundation = michigan_dynamic_hex_foundation_v1()
        .map_err(H3ReaderBootstrapErrorV1::ReferenceFoundation)?;
    let final_epoch =
        migrate_schema_epoch(config).map_err(H3ReaderBootstrapErrorV1::SchemaEpoch)?;
    if final_epoch.final_applied != CURRENT_SCHEMA_EPOCH {
        return Err(H3ReaderBootstrapErrorV1::UnexpectedSchemaEpoch {
            actual: final_epoch.final_applied,
        });
    }
    let reference_bundle_installation =
        install_michigan_h3_reference_bundle_v1(config, cohort, foundation)
            .map_err(H3ReaderBootstrapErrorV1::ReferenceInstall)?;
    Ok(H3ReaderBootstrapReportV1 {
        reference_bundle_installation,
        final_epoch,
    })
}
