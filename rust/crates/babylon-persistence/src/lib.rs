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
mod current_schema;
mod foundation;
mod glossary_concepts;
pub mod h3_reference_cohort;
mod h3_reference_installer;
pub mod identity;
pub mod material_envelope;
pub mod material_runtime;
mod metadata;
mod michigan_dynamic_hex_foundation;
pub mod michigan_economy;
pub(crate) mod observer_material;
pub mod observer_reader;
mod observer_tick_components;
mod place_producer;
pub mod postgres_catalog;
mod postgres_diagnostic;
pub(crate) mod production_projection;
mod reader;
mod runtime;
pub mod runtime_session;
pub mod sector_bundle;
mod semantic_batches;
mod semantic_codec;
mod semantic_vectors;
mod spatial_reference_installer;
pub mod spatial_reference_products;
mod stored_tick;
mod territory_county_map;
pub mod tick_commit_claim;

pub use archive::*;
pub use archive_foundation_grants::{
    foundation_grant_rows, foundation_grants_semantic_sha256, seed_foundation_grants,
    FoundationGrantReport, FoundationGrantRow, FoundationGrantsError,
    FOUNDATION_CONCEPT_GRANT_KEYS, FOUNDATION_COUNTY_GRANT_KEYS, FOUNDATION_COUNTY_LOCATOR_PREFIX,
    FOUNDATION_COUNTY_SOURCE_ID, FOUNDATION_GRANTS_SEMANTIC_DOMAIN, FOUNDATION_GRANT_TICK,
    FOUNDATION_PLACE_CONTAINMENT_LOCATOR_PREFIX, FOUNDATION_PLACE_CONTAINMENT_SOURCE_ID,
    FOUNDATION_PLACE_GRANT_KEYS, FOUNDATION_PLACE_IDENTITY_LOCATOR_PREFIX,
    FOUNDATION_PLACE_IDENTITY_SOURCE_ID, MICHIGAN_GEOID_PREFIX,
    PINNED_FOUNDATION_GRANTS_SEMANTIC_SHA256, STATEWIDE_RESIDUAL_COUNTY_FIPS,
};
pub use archive_wakeup::ARCHIVE_WAKEUP_CHANNEL;
pub use archive_worker::*;
pub use babylon_markdown::{
    fog_chip, git_export_markdown, is_citation_line, validate_babylon_markdown,
    BabylonMarkdownError, BABYLON_MARKDOWN_PROFILE_ID, CITATION_LINE_REGEX, FOG_CHIP_SEPARATOR,
};
pub use bootstrap::{
    bootstrap_current_runtime, CurrentRuntimeBootstrapError, CurrentRuntimeBootstrapReport,
};
pub use checkpoint::{
    ArchiveDirtyReceipt, CheckpointCompleteness, CheckpointRows, CommittedCheckpointSection,
    CommittedFullCheckpoint, CommittedResolveTick, CommittedResolveTickError,
    FullCheckpointSectionTag,
};
pub use county_producer::{
    county_committed_signals, county_page_input, county_page_semantic_sha256,
    desired_county_projection, filter_granted_county_plans, format_county_statblock_value,
    parse_stored_county_page, select_dirty_county_pages, CommittedTerritoryFields,
    CountyDossierProducer, CountyGrantIndex, CountyPagePlan, CountyPageProjection, CountyPlaceLink,
    CountySignal, CountySignalProjection, ARCHIVE_COUNTY_FIELD_READ_SQL, ARCHIVE_COUNTY_GRANTS_SQL,
    ARCHIVE_COUNTY_MAP_READ_SQL, ARCHIVE_COUNTY_PAGE_READ_SQL, COMMITTED_TICK_SOURCE_ID,
    COUNTY_DECISION_QUESTION, COUNTY_MEDIAN_WAGE_GRANT_KEY, COUNTY_MEDIAN_WAGE_LABEL,
    COUNTY_PHI_HOUR_GRANT_KEY, COUNTY_PHI_HOUR_LABEL, PINNED_COUNTY_IDENTITY_ARTIFACT_SHA256,
};

pub use foundation::{CampaignFoundation, FoundationContentBundle};
pub use glossary_concepts::{
    glossary_concepts, GlossaryConcept, GlossaryConcepts, GlossaryConceptsError,
    GLOSSARY_CONCEPTS_FIXTURE_PATH, PINNED_GLOSSARY_CONCEPTS_SHA256,
};

pub use h3_reference_installer::{
    install_michigan_h3_reference_bundle, H3ReferenceInstallBoundedResource,
    H3ReferenceInstallConflict, H3ReferenceInstallDisposition, H3ReferenceInstallError,
    H3ReferenceInstallOperation, H3ReferenceInstallReport, H3ReferenceMembershipReadContext,
};
pub use semantic_vectors::{
    verify_persistence_semantic_vector_row, verify_persistence_semantic_vectors,
    RustPersistenceVectorError, RustPersistenceVectorOutcome, RustPersistenceVectorReport,
};

pub use metadata::{
    BreadcrumbRow, CampaignCatalogRow, CampaignCatalogStatus, JumplistRow, RetainedMetadataStore,
    WatchlistRow,
};
pub use michigan_dynamic_hex_foundation::{
    decode_michigan_dynamic_hex_foundation, michigan_dynamic_hex_foundation,
    michigan_dynamic_hex_foundation_fixture_parts, MichiganDynamicHexFoundationDecodeError,
};

pub use current_schema::{
    current_schema_sha256, install_current_schema, preflight_current_schema,
    CurrentSchemaDisposition, CurrentSchemaError, CurrentSchemaIdentity, CurrentSchemaOperation,
    CurrentSchemaReport, CURRENT_SCHEMA_SQL, SCHEMA_ADVISORY_LOCK_KEY,
};

pub use place_producer::{
    desired_place_projection, parse_stored_place_page, place_page_input,
    place_page_semantic_sha256, select_dirty_place_pages, PlaceCountySlice, PlaceDossierProducer,
    PlaceGrantIndex, PlacePagePlan, PlacePageProjection, PlaceSignalProjection,
    ARCHIVE_PLACE_GRANTS_SQL, ARCHIVE_PLACE_PAGE_READ_SQL,
    PINNED_COUNTY_PLACE_OVERLAP_ARTIFACT_SHA256, PINNED_PLACE_IDENTITY_ARTIFACT_SHA256,
    PLACE_DECISION_QUESTION, PLACE_IDENTITY_GRANT_KEY, PLACE_IDENTITY_LOCATOR_PREFIX,
    PLACE_IDENTITY_SIGNAL_LABEL, PLACE_IDENTITY_SOURCE_ID,
};

pub use postgres_diagnostic::{
    PostgresDiagnostic, PostgresFailureClass, MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES,
};
pub use reader::*;
pub use runtime::{
    hydrate_campaign_foundation, prepare_committed_tick, CommittedTickReceipt,
    PreparedCommittedTick, RustPersistenceRuntimeError,
};

pub use semantic_batches::{StableGraphRowsEmptyProof, SuccessfulEventBatchEmptyProof};
pub use spatial_reference_installer::{
    install_michigan_spatial_reference_products, SpatialReferenceInstallDisposition,
    SpatialReferenceInstallError, SpatialReferenceInstallOperation, SpatialReferenceInstallReport,
    SpatialReferenceRelation,
};

pub use territory_county_map::{
    extract_declared_territory_county_map, TerritoryCountyMapError, TerritoryCountyMapRow,
    TERRITORY_COUNTY_MAP_FIELD,
};

mod production_evidence;
pub use production_evidence::ProductionEvidenceDigest;
pub use production_projection::material_balance::{
    CompletedMaterialBalance, ProductionMaterialBalanceRow,
};
pub mod production_observation;

pub mod michigan_cohorts;
pub mod michigan_content;
mod michigan_defines;
pub mod michigan_material;
pub use michigan_defines::MichiganDefinesError;
pub mod michigan_sectors;

#[cfg(test)]
mod test_support;
