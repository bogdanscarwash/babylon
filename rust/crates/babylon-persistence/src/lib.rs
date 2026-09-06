//! Rust-owned `PostgreSQL` persistence contracts and adapters.
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

mod archive;
pub mod archive_driver;
mod archive_foundation_grants;
pub mod archive_revision;
mod archive_wakeup;
mod archive_worker;
mod babylon_markdown;
mod bootstrap;
mod checkpoint;
pub mod committed_tick_envelope;
mod county_producer;
mod cutover_vectors;
pub mod error;
mod foundation;
mod foundation_content_schema;
mod glossary_concepts;
pub mod h3_reference_cohort;
mod h3_reference_installer;
mod h3_shadow_backfill;
pub mod hashes;
pub mod identity;
pub mod legacy_adopter;
pub mod material_envelope;
pub mod material_runtime;
mod metadata;
mod michigan_dynamic_hex_foundation;
pub mod michigan_economy;
pub mod migration_manifest;
pub(crate) mod observer_material;
pub mod observer_reader;
mod place_producer;
mod postgres_diagnostic;
pub(crate) mod production_projection;
mod reader;
mod runtime;
pub mod runtime_session;
pub mod schema_epoch;
pub mod schema_migration;
pub mod sector_bundle;
#[allow(
    dead_code,
    reason = "the stopped cutover composer remains private until Rust persistence activation"
)]
mod semantic_batches;
mod semantic_codec;
mod spatial_reference_installer;
pub mod spatial_reference_products;
mod stored_tick;
mod territory_county_map;
pub mod tick_commit_claim;

pub use archive::*;
pub use archive_foundation_grants::{
    foundation_grant_rows_v1, foundation_grants_semantic_sha256_v1, seed_foundation_grants_v1,
    FoundationGrantReportV1, FoundationGrantRowV1, FoundationGrantsErrorV1,
    FOUNDATION_CONCEPT_GRANT_KEYS_V1, FOUNDATION_COUNTY_GRANT_KEYS_V1,
    FOUNDATION_COUNTY_LOCATOR_PREFIX_V1, FOUNDATION_COUNTY_SOURCE_ID_V1,
    FOUNDATION_GRANTS_SEMANTIC_DOMAIN_V1, FOUNDATION_GRANT_TICK_V1,
    FOUNDATION_PLACE_CONTAINMENT_LOCATOR_PREFIX_V1, FOUNDATION_PLACE_CONTAINMENT_SOURCE_ID_V1,
    FOUNDATION_PLACE_GRANT_KEYS_V1, FOUNDATION_PLACE_IDENTITY_LOCATOR_PREFIX_V1,
    FOUNDATION_PLACE_IDENTITY_SOURCE_ID_V1, MICHIGAN_GEOID_PREFIX_V1,
    PINNED_FOUNDATION_GRANTS_SEMANTIC_SHA256_V1, STATEWIDE_RESIDUAL_COUNTY_FIPS_V1,
};
pub use archive_wakeup::{archive_wakeup_migration_sha256_v1, ARCHIVE_WAKEUP_CHANNEL_V1};
pub use archive_worker::*;
pub use babylon_markdown::{
    fog_chip_v1, git_export_markdown_v1, is_citation_line_v1, validate_babylon_markdown_v1,
    BabylonMarkdownErrorV1, BABYLON_MARKDOWN_PROFILE_ID_V1, CITATION_LINE_REGEX_V1,
    FOG_CHIP_SEPARATOR_V1,
};
pub use bootstrap::{
    bootstrap_h3_reader_epoch_v1, H3ReaderBootstrapErrorV1, H3ReaderBootstrapReportV1,
};
pub use checkpoint::{
    ArchiveDirtyReceiptV1, CheckpointCompletenessV1, CheckpointRowsV1,
    CommittedCheckpointSectionV1, CommittedFullCheckpointV1, CommittedResolveTickErrorV1,
    CommittedResolveTickV1, FullCheckpointSectionTagV1,
};
pub use county_producer::{
    county_committed_signals_v1, county_page_input_v1, county_page_semantic_sha256_v1,
    desired_county_projection_v1, filter_granted_county_plans_v1, format_county_statblock_value_v1,
    parse_stored_county_page_v1, select_dirty_county_pages_v1, CommittedTerritoryFieldsV1,
    CountyDossierProducerV1, CountyGrantIndexV1, CountyPagePlanV1, CountyPageProjectionV1,
    CountyPlaceLinkV1, CountySignalProjectionV1, CountySignalV1, ARCHIVE_COUNTY_FIELD_READ_SQL_V1,
    ARCHIVE_COUNTY_GRANTS_SQL_V1, ARCHIVE_COUNTY_MAP_READ_SQL_V1, ARCHIVE_COUNTY_PAGE_READ_SQL_V1,
    COMMITTED_TICK_SOURCE_ID_V1, COUNTY_DECISION_QUESTION_V1, COUNTY_MEDIAN_WAGE_GRANT_KEY_V1,
    COUNTY_MEDIAN_WAGE_LABEL_V1, COUNTY_PHI_HOUR_GRANT_KEY_V1, COUNTY_PHI_HOUR_LABEL_V1,
    PINNED_COUNTY_IDENTITY_ARTIFACT_SHA256_V1,
};
pub use cutover_vectors::{
    verify_rust_persistence_cutover_vector_row_v1, verify_rust_persistence_cutover_vectors_v1,
    RustPersistenceVectorErrorV1, RustPersistenceVectorOutcomeV1, RustPersistenceVectorReportV1,
};
pub use error::{PersistenceError, PersistenceFailureKind};
pub use foundation::{
    CampaignFoundationV1, FoundationContentBundle, FoundationContentBundleV1,
    FoundationContentBundleV2, FoundationContentLayout,
};
pub use glossary_concepts::{
    glossary_concepts_v1, GlossaryConceptV1, GlossaryConceptsErrorV1, GlossaryConceptsV1,
    GLOSSARY_CONCEPTS_FIXTURE_PATH_V1, PINNED_GLOSSARY_CONCEPTS_SHA256_V1,
};
pub use h3_reference_cohort::{
    build_representative_h3_cohort_v1, representative_h3_reference_cohort_v1, H3ReferenceCellRow,
    H3ReferenceCohort, H3ReferenceCohortError, H3ReferenceCohortReceipt, H3ReferenceOrigin,
    MAX_H3_REFERENCE_SOURCE_CELLS,
};
pub use h3_reference_installer::{
    install_michigan_h3_reference_bundle_v1, H3ReferenceInstallBoundedResource,
    H3ReferenceInstallConflict, H3ReferenceInstallDisposition, H3ReferenceInstallError,
    H3ReferenceInstallOperation, H3ReferenceInstallReport, H3ReferenceMembershipReadContext,
};
pub use h3_shadow_backfill::{
    backfill_legacy_h3_shadow_keys, H3ShadowBackfillBoundedResource, H3ShadowBackfillDisposition,
    H3ShadowBackfillError, H3ShadowBackfillIssue, H3ShadowBackfillIssueKind,
    H3ShadowBackfillOperation, H3ShadowBackfillReport, H3ShadowFieldReport, H3ShadowRelation,
    H3ShadowRelationReport, H3_SHADOW_FIELD_COUNT, H3_SHADOW_RELATION_COUNT,
    MAX_H3_SHADOW_BACKFILL_BATCH_ROWS, MAX_H3_SHADOW_BACKFILL_COMMIT_ATTEMPTS,
    MAX_H3_SHADOW_BACKFILL_ISSUES, MAX_H3_SHADOW_DISTINCT_GROUPS, MAX_H3_SHADOW_ROWS_PER_RELATION,
    MAX_H3_SHADOW_TEXT_BYTES,
};
pub use hashes::{GraphStateHash, MigrationSetDigest, ReplayIdentityHash};
pub use identity::CampaignId;
pub use legacy_adopter::{
    adopt_legacy_schema, compare_legacy_census, expected_legacy_census,
    legacy_adopter_sql_statements, parse_legacy_census_fixture, validate_legacy_connection_target,
    validate_legacy_stamps, LegacyAdopterError, LegacyAdopterOperation, LegacyAdopterSqlKind,
    LegacyAdopterSqlStatement, LegacyAdoptionReport, LegacyBoundedResource, LegacyCensus,
    LegacyCensusEntry, LegacyCensusParseError, LegacyCleanupFailureV1,
    LegacyConnectionTargetRejection, LegacyObjectKey, LegacyObjectKind,
    LegacyOwnerAuthorityDisposition, LegacyStampClass, LegacyStampDefinition, LegacyStampMatch,
    LegacyStampProvenance, LegacyStampReport, LEGACY_ADOPTER_CONNECT_TIMEOUT,
    LEGACY_ADOPTER_STARTUP_OPTIONS, LEGACY_ADOPTER_TCP_USER_TIMEOUT, LEGACY_CENSUS_FIXTURE,
    LEGACY_CENSUS_VERSION, LEGACY_STAMP_CATALOG, MAX_LEGACY_CENSUS_FIXTURE_BYTES,
    MAX_LEGACY_CENSUS_FIXTURE_LINES, MAX_LEGACY_CENSUS_ROWS,
    MAX_LEGACY_EXTENSION_DEPENDENCY_ADDRESSES, MAX_LEGACY_EXTENSION_MEMBERS,
    MAX_LEGACY_EXTENSION_ROLE_IDENTITIES, MAX_LEGACY_PARTITIONS_PER_FAMILY,
    MAX_LEGACY_SEQUENCE_OWNERSHIP, MAX_LEGACY_STAMP_ROWS, POSTGRES_IDENTIFIER_MAX_BYTES,
};
pub use metadata::{
    BreadcrumbRowV1, CampaignCatalogRowV1, CampaignCatalogStatusV1, JumplistRowV1,
    RetainedMetadataStoreV1, WatchlistRowV1,
};
pub use michigan_dynamic_hex_foundation::{
    decode_michigan_dynamic_hex_foundation_v1, michigan_dynamic_hex_foundation_fixture_parts_v1,
    michigan_dynamic_hex_foundation_v1, MichiganDynamicHexFoundationDecodeErrorV1,
};
pub use michigan_economy::*;
pub use migration_manifest::{
    ManifestError, MigrationManifest, MAX_MANIFEST_BYTES, MAX_MANIFEST_CHUNKS,
    SCHEMA_ADVISORY_LOCK_KEY,
};
pub use observer_reader::*;
pub use place_producer::{
    desired_place_projection_v1, parse_stored_place_page_v1, place_page_input_v1,
    place_page_semantic_sha256_v1, select_dirty_place_pages_v1, PlaceCountySliceV1,
    PlaceDossierProducerV1, PlaceGrantIndexV1, PlacePagePlanV1, PlacePageProjectionV1,
    PlaceSignalProjectionV1, ARCHIVE_PLACE_GRANTS_SQL_V1, ARCHIVE_PLACE_PAGE_READ_SQL_V1,
    PINNED_COUNTY_PLACE_OVERLAP_ARTIFACT_SHA256_V1, PINNED_PLACE_IDENTITY_ARTIFACT_SHA256_V1,
    PLACE_DECISION_QUESTION_V1, PLACE_IDENTITY_GRANT_KEY_V1, PLACE_IDENTITY_LOCATOR_PREFIX_V1,
    PLACE_IDENTITY_SIGNAL_LABEL_V1, PLACE_IDENTITY_SOURCE_ID_V1,
};
pub use postgres_diagnostic::{
    PostgresDiagnosticV1, PostgresFailureClassV1, MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES,
};
pub use reader::*;
pub use runtime::{
    activate_rust_persistence_v2, hydrate_campaign_foundation_v1, prepare_committed_tick_v2,
    ActivationReportV2, CommittedTickAuthorityLedgerRowV2, CommittedTickAuthorityStateV2,
    CommittedTickReceiptV2, DurableReplayRuntimeV2, PreActivationIncompatibleRelationV2,
    PreparedCommittedTickV2, RustPersistenceActivationErrorV2, RustPersistenceRuntimeErrorV2,
};
pub use runtime_session::*;
pub use schema_epoch::{
    compiled_committed_tick_v2_activation_migrations, compiled_schema_migrations,
    migrate_schema_epoch, preflight_schema_epoch, validate_migration_prefix, PersistedMigration,
    SchemaEpochError, SchemaEpochObservation, SchemaEpochOperation, SchemaEpochOrigin,
    SchemaEpochRelation, SchemaEpochReport, SchemaEpochSchemas, MAX_COMMIT_ATTEMPTS_PER_VERSION,
    MAX_SCHEMA_MIGRATIONS,
};
pub use schema_migration::{
    MigrationChecksum, MigrationVersion, SchemaMigration, SchemaMigrationError,
    MAX_SCHEMA_MIGRATION_SQL_BYTES, MIGRATION_CHECKSUM_BYTES,
};
pub use semantic_batches::{StableGraphRowsEmptyProofV1, SuccessfulEventBatchEmptyProofV2};
pub use spatial_reference_installer::{
    install_michigan_spatial_reference_products, SpatialReferenceInstallDisposition,
    SpatialReferenceInstallError, SpatialReferenceInstallOperation, SpatialReferenceInstallReport,
    SpatialReferenceRelation,
};
pub use spatial_reference_products::{
    michigan_spatial_reference_products_v1, CountyH3LandAreaRow, CountyIdentityRow,
    CountyPlaceH3LandAreaRow, H3CountRow, H3LandFractionRow, PlaceIdentityRow, ReferenceProduct,
    ReferenceProductEvidenceClass, SpatialReferenceProducts, SpatialReferenceProductsError,
};
pub use territory_county_map::{
    extract_declared_territory_county_map_v1, install_territory_county_map_schema_v1,
    TerritoryCountyMapErrorV1, TerritoryCountyMapRowV1, TerritoryCountyMapSchemaDispositionV1,
    TERRITORY_COUNTY_MAP_FIELD_V1, TERRITORY_COUNTY_MAP_SCHEMA_CONTRACT_ID,
    TERRITORY_COUNTY_MAP_SCHEMA_V1_SQL,
};

mod production_evidence;
pub use production_evidence::ProductionEvidenceDigestV3;
pub use production_projection::material_balance::{
    CompletedMaterialBalanceV1, ProductionMaterialBalanceRowV1,
};
pub mod production_observation;
pub use production_observation::*;

pub mod michigan_cohorts;
pub mod michigan_content;
pub mod michigan_material;
pub mod michigan_sectors;
