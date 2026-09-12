//! Current schema admission and durable replay composition.

use babylon_bsl::identity_codec::StableBslValue;
#[cfg(test)]
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_kernel::content_digest::sha256_of;
use babylon_kernel::tick_content_hash::TickContentHash;
#[cfg(test)]
use babylon_practice_contract::OrderedPracticeActionBatch;
use babylon_tick::material_state::MaterialState;
use babylon_tick::replay_session::{
    IdentifiedTickReport, ReplayCommitDisposition, ReplayTickSession,
};
use postgres::binary_copy::BinaryCopyInWriter;
use postgres::types::{ToSql, Type};
use postgres::{Config, GenericClient, NoTls};

use crate::checkpoint::{
    compose_archive_dirty_receipt, compose_checkpoint_rows, ArchiveDirtyReceipt, CheckpointRows,
    CommittedFullCheckpoint, CommittedResolveTick, CommittedResolveTickError,
};
use crate::committed_tick_envelope::{
    CommittedTickEnvelopeError, CommittedTickRow, CommittedTickRowFamilies,
};
use crate::foundation::{CampaignFoundation, FoundationContentBundle};
use crate::identity::CampaignId;
use crate::metadata::ensure_campaign_catalog_row;
use crate::michigan_dynamic_hex_foundation::michigan_dynamic_hex_foundation;
use crate::postgres_catalog::validate_connection_target;
use crate::postgres_diagnostic::PostgresDiagnostic;
use crate::semantic_batches::{
    compose_graph_event_choice_semantic_batches, compose_material_state_rows,
    GraphEventChoiceSemanticBatches, SemanticBatchError,
};
use crate::semantic_codec::SemanticCodecError;

const REFERENCE_BUNDLE_DOMAIN: &[u8] = b"babylon.h3.reference-bundle-composite.v1\0";

/// A checked refusal while deriving durable inputs from one identified tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustPersistenceRuntimeError {
    CurrentSchema(crate::CurrentSchemaError),
    /// A local-only runtime connection or transaction operation failed.
    Database {
        /// Stable operation name without caller-supplied text.
        operation: &'static str,
        /// Bounded secret-safe driver diagnostic, when the failure came from `PostgreSQL`.
        diagnostic: Option<PostgresDiagnostic>,
    },
    /// A requested campaign foundation is absent.
    FoundationAbsent,
    /// Durable campaign bytes differ from the requested exact foundation.
    CampaignConflict,
    /// The content bundle's scenario does not reproduce the session's captured graph.
    FoundationScenarioMismatch,
    /// The replay session refused preparation or acknowledgement.
    ReplayTick,
    /// A post-ack observation receipt does not name this runtime's committed tail.
    ObservationNotCurrentCommittedTail {
        /// Exact tick named by the refused receipt.
        receipt_tick: u64,
        /// Current runtime tail, or `None` before the first acknowledgement.
        current_tail: Option<u64>,
    },
    /// Recomposition did not reproduce the receipt-bound post-tick graph digest.
    ObservationGraphDigestMismatch,
    /// Detailed choice evidence is unavailable for the current process-local acknowledgement.
    ObservationChoiceReceiptUnavailable,
    /// The report's completed tick cannot be a durable `PostgreSQL` tick.
    ResolveTickOutOfRange {
        /// Exact refused completed tick.
        actual: i64,
    },
    /// Campaign foundation capture was attempted after a real tick executed.
    FoundationAfterTickZero {
        /// Exact completed tick owned by the refused session.
        actual: i64,
    },
    /// A tick-owned exact source could not be recomposed or copied.
    ReplaySource,
    /// A delta checkpoint cannot be selected as a restart root.
    DeltaCheckpointNotRestartRoot,
    /// A governed semantic row codec refused its report-owned input.
    SemanticCodec,
    /// The aggregate committed-tick bounds refused the composed rows.
    SemanticEnvelope(CommittedTickEnvelopeError),
    /// Checked semantic-batch arithmetic overflowed.
    CapacityOverflow {
        /// Stable refused buffer or count name.
        field: &'static str,
    },
    /// A semantic producer count cannot fit its governed integer width.
    IntegerConversion {
        /// Stable refused count name.
        field: &'static str,
        /// Exact refused source value.
        value: usize,
    },
    /// A report-derived semantic buffer could not reserve exact capacity.
    Allocation {
        /// Stable refused buffer name.
        field: &'static str,
        /// Exact requested capacity.
        requested: usize,
    },
    /// The declared territory-county mapping refused extraction or persistence.
    TerritoryCountyMap(crate::territory_county_map::TerritoryCountyMapError),
    /// The additive semantic Archive schema refused installation.
    SemanticArchive(crate::SemanticArchiveError),
    /// Foundation knowledge-grant seeding refused fixture drift or a grant conflict.
    FoundationGrants(crate::archive_foundation_grants::FoundationGrantsError),
}

impl From<SemanticBatchError> for RustPersistenceRuntimeError {
    fn from(value: SemanticBatchError) -> Self {
        match value {
            SemanticBatchError::Codec(_) => Self::SemanticCodec,
            SemanticBatchError::Envelope(error) => Self::SemanticEnvelope(error),
            SemanticBatchError::CapacityOverflow { field } => Self::CapacityOverflow { field },
            SemanticBatchError::IntegerConversion { field, value } => {
                Self::IntegerConversion { field, value }
            }
            SemanticBatchError::Allocation { field, requested } => {
                Self::Allocation { field, requested }
            }
        }
    }
}

impl From<SemanticCodecError> for RustPersistenceRuntimeError {
    fn from(value: SemanticCodecError) -> Self {
        match value {
            SemanticCodecError::CapacityOverflow { field } => Self::CapacityOverflow { field },
            SemanticCodecError::IntegerConversion { field, value } => {
                Self::IntegerConversion { field, value }
            }
            SemanticCodecError::Allocation { field, requested } => {
                Self::Allocation { field, requested }
            }
            SemanticCodecError::ByteLimit { .. }
            | SemanticCodecError::Refusal(_)
            | SemanticCodecError::Invalid(_) => Self::SemanticCodec,
        }
    }
}

impl std::fmt::Display for RustPersistenceRuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Rust persistence runtime refused: {self:?}")
    }
}

impl std::error::Error for RustPersistenceRuntimeError {}

impl RustPersistenceRuntimeError {
    pub(crate) fn database(operation: &'static str) -> Self {
        Self::Database {
            operation,
            diagnostic: None,
        }
    }

    pub(crate) fn postgres(operation: &'static str, error: &postgres::Error) -> Self {
        Self::Database {
            operation,
            diagnostic: Some(PostgresDiagnostic::capture(error)),
        }
    }
}

/// Bounded durable observation returned only after marker-last acknowledgement.
///
/// This intentionally excludes write identities, values, database coordinates,
/// and other detailed persistence evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedTickReceipt {
    resolve_tick: CommittedResolveTick,
    commit_disposition: ReplayCommitDisposition,
    graph_before: [u8; 32],
    graph_after: [u8; 32],
    prior_stable_graph_digest: [u8; 32],
    result_stable_graph_digest: [u8; 32],
    world_before: [u8; 32],
    world_after: [u8; 32],
    considered: usize,
    fired: usize,
    per_rule_considered: Vec<(String, usize)>,
    per_rule_fired: Vec<(String, usize)>,
    event_count: usize,
    event_digest: [u8; 32],
    choice_receipt_count: usize,
    choice_receipt_digest: [u8; 32],
    audit_receipt_count: usize,
    material_row_count: usize,
    material_row_digest: [u8; 32],
    tick_content_hash: TickContentHash,
}

impl CommittedTickReceipt {
    pub(crate) fn from_material_candidate(
        candidate: &babylon_tick::material_replay::PreparedMaterialTick<HypergraphStore>,
    ) -> Result<Self, RustPersistenceRuntimeError> {
        let acknowledged = candidate.graph_report();
        let report = acknowledged.report();
        let identity = candidate.identity();
        Ok(Self {
            resolve_tick: CommittedResolveTick::try_from(identity.resolve_tick())
                .map_err(|_| RustPersistenceRuntimeError::ReplayTick)?,
            commit_disposition: ReplayCommitDisposition::Committed,
            graph_before: report.before,
            graph_after: report.after,
            prior_stable_graph_digest: acknowledged.prior_stable_graph_digest().into_bytes(),
            result_stable_graph_digest: acknowledged.result_stable_graph_digest().into_bytes(),
            world_before: identity.prior_world_hash(),
            world_after: identity.result_world_hash(),
            considered: report.considered,
            fired: report.fired,
            per_rule_considered: report.per_rule_considered.clone(),
            per_rule_fired: report.per_rule_fired.clone(),
            event_count: acknowledged.successful_event_batch().events().len(),
            event_digest: acknowledged.successful_event_batch().source_digest(),
            choice_receipt_count: report.choice_receipts.len(),
            choice_receipt_digest: acknowledged.choice_receipt_source_digest(),
            audit_receipt_count: report.audit_receipts.len(),
            material_row_count: acknowledged.material_state_rows().source_count(),
            material_row_digest: acknowledged.material_state_rows().source_digest(),
            tick_content_hash: identity.tick_content_hash(),
        })
    }

    pub(crate) fn acknowledge(&mut self, disposition: ReplayCommitDisposition) {
        self.commit_disposition = disposition;
    }

    /// Return the one-based durable tick.
    #[must_use]
    pub const fn resolve_tick(&self) -> CommittedResolveTick {
        self.resolve_tick
    }

    /// Return how `PostgreSQL` acknowledgement was established.
    #[must_use]
    pub const fn commit_disposition(&self) -> ReplayCommitDisposition {
        self.commit_disposition
    }

    /// Return the administrative `GraphStateHash` before adjudication.
    #[must_use]
    pub const fn graph_before(&self) -> [u8; 32] {
        self.graph_before
    }

    /// Return the administrative `GraphStateHash` after adjudication.
    #[must_use]
    pub const fn graph_after(&self) -> [u8; 32] {
        self.graph_after
    }

    /// Return the stable graph-state digest bound to the acknowledged prior.
    #[must_use]
    pub const fn prior_stable_graph_digest(&self) -> [u8; 32] {
        self.prior_stable_graph_digest
    }

    /// Return the stable graph-state digest bound to the acknowledged result.
    #[must_use]
    pub const fn result_stable_graph_digest(&self) -> [u8; 32] {
        self.result_stable_graph_digest
    }

    /// Return the nominal-world hash before adjudication.
    #[must_use]
    pub const fn world_before(&self) -> [u8; 32] {
        self.world_before
    }

    /// Return the nominal-world hash after adjudication.
    #[must_use]
    pub const fn world_after(&self) -> [u8; 32] {
        self.world_after
    }

    /// Return the total number of guard evaluations.
    #[must_use]
    pub const fn considered(&self) -> usize {
        self.considered
    }

    /// Return the total number of subjects that fired.
    #[must_use]
    pub const fn fired(&self) -> usize {
        self.fired
    }

    /// Borrow per-rule guard counts in governed causal order.
    #[must_use]
    pub fn per_rule_considered(&self) -> &[(String, usize)] {
        &self.per_rule_considered
    }

    /// Borrow per-rule firing counts in governed causal order.
    #[must_use]
    pub fn per_rule_fired(&self) -> &[(String, usize)] {
        &self.per_rule_fired
    }

    /// Return the number of retained successful events.
    #[must_use]
    pub const fn event_count(&self) -> usize {
        self.event_count
    }

    /// Return the digest of the exact tick event section.
    #[must_use]
    pub const fn event_digest(&self) -> [u8; 32] {
        self.event_digest
    }

    /// Return the number of exact realized-choice receipts.
    #[must_use]
    pub const fn choice_receipt_count(&self) -> usize {
        self.choice_receipt_count
    }

    /// Return the digest of the exact ordered choice-receipt section.
    #[must_use]
    pub const fn choice_receipt_digest(&self) -> [u8; 32] {
        self.choice_receipt_digest
    }

    /// Return the number of identity-free causal audit receipts.
    #[must_use]
    pub const fn audit_receipt_count(&self) -> usize {
        self.audit_receipt_count
    }

    /// Return the number of canonical material rows.
    #[must_use]
    pub const fn material_row_count(&self) -> usize {
        self.material_row_count
    }

    /// Return the digest of the canonical material-row aggregate.
    #[must_use]
    pub const fn material_row_digest(&self) -> [u8; 32] {
        self.material_row_digest
    }

    /// Return the constitutional content identity acknowledged by `PostgreSQL`.
    #[must_use]
    pub const fn tick_content_hash(&self) -> TickContentHash {
        self.tick_content_hash
    }
}

/// Read and authenticate the current campaign foundation.
///
/// # Errors
/// Refuses incompatible schemas, database failures, or inconsistent stored foundation rows.
pub fn hydrate_campaign_foundation(
    config: &Config,
    campaign_id: CampaignId,
) -> Result<CampaignFoundation, RustPersistenceRuntimeError> {
    let _active = verify_runtime_schema(config)?;
    validate_connection_target(config)
        .map_err(|_| RustPersistenceRuntimeError::database("validate foundation target"))?;
    let mut client = crate::current_schema::bounded_config(config)
        .connect(NoTls)
        .map_err(|error| {
            RustPersistenceRuntimeError::postgres("connect foundation reader", &error)
        })?;
    verify_runtime_schema_client(&mut client)?;
    hydrate_campaign_foundation_client(&mut client, campaign_id)
}

/// Rebuild the exact stored tick-zero graph and verify all captured components.
/// The immutable H3 reference remains the existing admitted reference; scenario,
/// rules, defines, session identity and seed come from the durable foundation.
pub(crate) fn reconstruct_graph_foundation_session(
    foundation: &CampaignFoundation,
) -> Result<ReplayTickSession<HypergraphStore>, RustPersistenceRuntimeError> {
    let bundle = foundation.content_bundle();
    let scenario = std::str::from_utf8(bundle.scenario_source_bytes())
        .map_err(|_| RustPersistenceRuntimeError::ReplaySource)?;
    let prelude = bundle
        .prelude_source_bytes()
        .map(std::str::from_utf8)
        .transpose()
        .map_err(|_| RustPersistenceRuntimeError::ReplaySource)?;
    let rules = std::str::from_utf8(bundle.rule_source_bytes())
        .map_err(|_| RustPersistenceRuntimeError::ReplaySource)?;
    let material = MaterialState::try_new(
        michigan_dynamic_hex_foundation().map_err(|_| RustPersistenceRuntimeError::ReplaySource)?,
    )
    .map_err(|_| RustPersistenceRuntimeError::ReplaySource)?;
    let session = ReplayTickSession::new(
        scenario,
        prelude,
        rules,
        HypergraphStore::new(),
        foundation.replay_session_identity().clone(),
        foundation.rng_seed(),
        foundation.content_digest().clone(),
        foundation.reference_digest(),
        material,
    )
    .map_err(|_| RustPersistenceRuntimeError::ReplayTick)?;
    let verification_bundle = FoundationContentBundle::try_new(
        scenario,
        prelude,
        rules,
        bundle.defines_bytes(),
        bundle.reference_bundle_manifest_bytes(),
    )?;
    let verification = CampaignFoundation::capture(&session, verification_bundle)?;
    if verification.canonical_bytes() != foundation.canonical_bytes() {
        return Err(RustPersistenceRuntimeError::CampaignConflict);
    }
    Ok(session)
}

pub(crate) fn hydrate_campaign_foundation_client(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
) -> Result<CampaignFoundation, RustPersistenceRuntimeError> {
    let row = client
        .query_opt(
            "SELECT stable_graph, world_registers, resolver_manifest, prepared_environment, \
                    replay_session_id, rng_seed, defines_hash, rules_hash, ref_digest, \
                    scenario_source, prelude_source, rule_source, defines_bytes, \
                    reference_manifest_bytes, foundation_sha256 \
             FROM babylon_state.campaign_foundation \
             WHERE campaign_id = $1::uuid",
            &[campaign_id.as_uuid()],
        )
        .map_err(|error| RustPersistenceRuntimeError::postgres("read campaign foundation", &error))?
        .ok_or(RustPersistenceRuntimeError::FoundationAbsent)?;
    let stable_graph: Vec<u8> = decode_runtime_column(&row, 0)?;
    let world_registers: Vec<u8> = decode_runtime_column(&row, 1)?;
    let resolver_manifest: Vec<u8> = decode_runtime_column(&row, 2)?;
    let prepared_environment: Vec<u8> = decode_runtime_column(&row, 3)?;
    let replay_session_id: String = decode_runtime_column(&row, 4)?;
    let rng_seed: i64 = decode_runtime_column(&row, 5)?;
    let defines_hash = decode_digest_column(&row, 6)?;
    let rules_hash = decode_digest_column(&row, 7)?;
    let reference_digest = decode_digest_column(&row, 8)?;
    let scenario_source: String = decode_runtime_column(&row, 9)?;
    let prelude_source: Option<String> = decode_runtime_column(&row, 10)?;
    let rule_source: String = decode_runtime_column(&row, 11)?;
    let defines_bytes: Vec<u8> = decode_runtime_column(&row, 12)?;
    let reference_manifest: Vec<u8> = decode_runtime_column(&row, 13)?;
    let foundation_sha256 = decode_digest_column(&row, 14)?;
    crate::territory_county_map::verify_territory_county_map(
        client,
        campaign_id,
        &scenario_source,
        prelude_source.as_deref(),
    )
    .map_err(RustPersistenceRuntimeError::TerritoryCountyMap)?;
    CampaignFoundation::from_persisted(
        stable_graph,
        world_registers,
        resolver_manifest,
        prepared_environment,
        &replay_session_id,
        rng_seed,
        defines_hash,
        rules_hash,
        reference_digest,
        &scenario_source,
        prelude_source.as_deref(),
        &rule_source,
        &defines_bytes,
        &reference_manifest,
        foundation_sha256,
    )
}

pub(crate) fn verify_runtime_schema(
    config: &Config,
) -> Result<crate::CurrentSchemaIdentity, RustPersistenceRuntimeError> {
    validate_connection_target(config).map_err(|error| {
        RustPersistenceRuntimeError::CurrentSchema(crate::CurrentSchemaError::ConnectionTarget(
            error,
        ))
    })?;
    let mut client = crate::current_schema::bounded_config(config)
        .connect(NoTls)
        .map_err(|error| {
            RustPersistenceRuntimeError::postgres("connect current schema reader", &error)
        })?;
    verify_runtime_schema_client(&mut client)
}

pub(crate) fn verify_runtime_schema_client(
    client: &mut impl GenericClient,
) -> Result<crate::CurrentSchemaIdentity, RustPersistenceRuntimeError> {
    crate::current_schema::require_current_schema(client)
        .map_err(RustPersistenceRuntimeError::CurrentSchema)
}

pub(crate) fn insert_campaign_foundation_rows(
    client: &mut postgres::Transaction<'_>,
    campaign_id: CampaignId,
    foundation: &CampaignFoundation,
) -> Result<(), RustPersistenceRuntimeError> {
    let _active = verify_runtime_schema_client(client)?;
    let replay_session = std::str::from_utf8(foundation.replay_session_identity().as_bytes())
        .map_err(|_| RustPersistenceRuntimeError::ReplaySource)?;
    let bundle = foundation.content_bundle();
    let base_reference_digest = base_reference_digest(
        bundle.reference_bundle_manifest_bytes(),
        foundation.reference_digest(),
    )?;
    client
        .execute(
            "INSERT INTO babylon_state.campaign \
             (campaign_id, replay_layout_version, rng_layout_version, replay_session_id, rng_seed, \
              defines_hash, rules_hash, ref_digest) \
             VALUES ($1, 1, 2, $2, $3, $4, $5, $6) ON CONFLICT (campaign_id) DO NOTHING",
            &[
                campaign_id.as_uuid(),
                &replay_session,
                &i64::from_be_bytes(foundation.rng_seed().to_be_bytes()),
                &&foundation.content_digest().defines_hash[..],
                &&foundation.content_digest().rules_hash[..],
                &&base_reference_digest[..],
            ],
        )
        .map_err(|error| {
            RustPersistenceRuntimeError::postgres("insert campaign identity", &error)
        })?;
    let scenario = std::str::from_utf8(bundle.scenario_source_bytes())
        .map_err(|_| RustPersistenceRuntimeError::ReplaySource)?;
    let prelude = bundle
        .prelude_source_bytes()
        .map(std::str::from_utf8)
        .transpose()
        .map_err(|_| RustPersistenceRuntimeError::ReplaySource)?;
    let rules = std::str::from_utf8(bundle.rule_source_bytes())
        .map_err(|_| RustPersistenceRuntimeError::ReplaySource)?;
    let territory_county_map =
        crate::territory_county_map::extract_declared_territory_county_map(scenario, prelude)
            .map_err(RustPersistenceRuntimeError::TerritoryCountyMap)?;
    let foundation_sha256 = sha256_of(foundation.canonical_bytes());
    client
        .execute(
            "INSERT INTO babylon_state.campaign_foundation \
             (campaign_id, stable_graph, world_registers, resolver_manifest, prepared_environment, \
              replay_session_id, rng_seed, defines_hash, rules_hash, ref_digest, scenario_source, \
              prelude_source, rule_source, defines_bytes, reference_manifest_bytes, foundation_sha256) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16) \
             ON CONFLICT (campaign_id) DO NOTHING",
            &[
                campaign_id.as_uuid(),
                &foundation.stable_graph_bytes(),
                &foundation.world_register_bytes(),
                &foundation.resolver_manifest_bytes(),
                &foundation.prepared_environment_bytes(),
                &replay_session,
                &i64::from_be_bytes(foundation.rng_seed().to_be_bytes()),
                &&foundation.content_digest().defines_hash[..],
                &&foundation.content_digest().rules_hash[..],
                &foundation.reference_digest().as_bytes().as_slice(),
                &scenario,
                &prelude,
                &rules,
                &bundle.defines_bytes(),
                &bundle.reference_bundle_manifest_bytes(),
                &&foundation_sha256[..],
            ],
        )
        .map_err(|error| {
            RustPersistenceRuntimeError::postgres("insert campaign foundation", &error)
        })?;
    let stored_sha: Vec<u8> = client
        .query_one(
            "SELECT foundation_sha256 FROM babylon_state.campaign_foundation \
             WHERE campaign_id = $1::uuid",
            &[campaign_id.as_uuid()],
        )
        .and_then(|row| row.try_get(0))
        .map_err(|error| {
            RustPersistenceRuntimeError::postgres("verify campaign foundation", &error)
        })?;
    if stored_sha.as_slice() != foundation_sha256 {
        return Err(RustPersistenceRuntimeError::CampaignConflict);
    }
    ensure_campaign_catalog_row(client, campaign_id, foundation)?;
    crate::territory_county_map::insert_territory_county_map_rows(
        client,
        campaign_id,
        &territory_county_map,
    )
    .map_err(RustPersistenceRuntimeError::TerritoryCountyMap)?;
    crate::archive_foundation_grants::seed_foundation_grants(client, campaign_id)
        .map_err(RustPersistenceRuntimeError::FoundationGrants)?;
    Ok(())
}

fn base_reference_digest(
    reference_manifest: &[u8],
    expected_bundle_digest: babylon_kernel::tick_content_hash::RefDigest,
) -> Result<[u8; 32], RustPersistenceRuntimeError> {
    let expected_len = REFERENCE_BUNDLE_DOMAIN
        .len()
        .checked_add(64)
        .ok_or(RustPersistenceRuntimeError::ReplaySource)?;
    if reference_manifest.len() != expected_len
        || !reference_manifest.starts_with(REFERENCE_BUNDLE_DOMAIN)
        || sha256_of(reference_manifest) != *expected_bundle_digest.as_bytes()
    {
        return Err(RustPersistenceRuntimeError::ReplaySource);
    }
    reference_manifest[REFERENCE_BUNDLE_DOMAIN.len()..REFERENCE_BUNDLE_DOMAIN.len() + 32]
        .try_into()
        .map_err(|_| RustPersistenceRuntimeError::ReplaySource)
}

pub(crate) fn insert_typed_tick_pre_marker_rows(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReport,
    checkpoint: &CommittedFullCheckpoint,
    tick_content_hash: TickContentHash,
) -> Result<(), RustPersistenceRuntimeError> {
    let action_layout = i16::try_from(report.action_batch_layout_version())
        .map_err(|_| RustPersistenceRuntimeError::CampaignConflict)?;
    require_single_insert(
        client.execute(
            "INSERT INTO babylon_state.tick_action_batch_v1 \
             (campaign_id, resolve_tick, layout_version, action_batch_digest, exact_action_batch_bytes) \
             VALUES ($1::uuid, $2, $3, $4, $5)",
            &[
                campaign_id.as_uuid(),
                &resolve_tick,
                &action_layout,
                &&report.action_batch_digest().as_bytes()[..],
                &report.action_batch_bytes(),
            ],
        ),
        "insert action batch",
    )?;
    insert_typed_graph_rows(client, campaign_id, resolve_tick, report)?;
    insert_typed_material_rows(client, campaign_id, resolve_tick, report)?;
    insert_choice_receipt_rows(client, campaign_id, resolve_tick, report)?;
    insert_typed_event_rows(client, campaign_id, resolve_tick, report)?;
    insert_full_checkpoint(client, campaign_id, resolve_tick, checkpoint)?;
    require_single_insert(
        client.execute(
            "INSERT INTO babylon_state.archive_dirty_receipt_v1 \
             (campaign_id, resolve_tick, tick_content_hash) VALUES ($1::uuid, $2, $3)",
            &[
                campaign_id.as_uuid(),
                &resolve_tick,
                &&tick_content_hash.as_bytes()[..],
            ],
        ),
        "insert archive dirty receipt",
    )
}

fn insert_typed_graph_rows(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReport,
) -> Result<(), RustPersistenceRuntimeError> {
    let rows = report.result_stable_graph().rows();
    let sink = client.copy_in(
        "COPY babylon_state.graph_node_v1 (campaign_id, resolve_tick, local_name, node_type) FROM STDIN BINARY",
    ).map_err(|error| RustPersistenceRuntimeError::postgres("begin graph node copy", &error))?;
    let mut writer =
        BinaryCopyInWriter::new(sink, &[Type::UUID, Type::INT8, Type::TEXT, Type::TEXT]);
    for (local_name, node_type) in rows.nodes() {
        writer
            .write(&[campaign_id.as_uuid(), &resolve_tick, local_name, node_type])
            .map_err(|error| {
                RustPersistenceRuntimeError::postgres("write graph node copy", &error)
            })?;
    }
    finish_binary_copy(writer, rows.nodes().len(), "finish graph node copy")?;
    let sink = client.copy_in(
        "COPY babylon_state.graph_node_f64_v1 (campaign_id, resolve_tick, local_name, qname, value_bits) FROM STDIN BINARY",
    ).map_err(|error| RustPersistenceRuntimeError::postgres("begin graph node f64 copy", &error))?;
    let mut writer = BinaryCopyInWriter::new(
        sink,
        &[Type::UUID, Type::INT8, Type::TEXT, Type::TEXT, Type::INT8],
    );
    for (local_name, qname, bits) in rows.node_f64() {
        writer
            .write(&[
                campaign_id.as_uuid(),
                &resolve_tick,
                local_name,
                qname,
                &bit_pattern_i64(*bits),
            ])
            .map_err(|error| {
                RustPersistenceRuntimeError::postgres("write graph node f64 copy", &error)
            })?;
    }
    finish_binary_copy(writer, rows.node_f64().len(), "finish graph node f64 copy")?;
    for (edge_type, source, target, strength_bits) in rows.edges() {
        let strength_bits = bit_pattern_i64(*strength_bits);
        require_single_insert(
            client.execute(
                "INSERT INTO babylon_state.graph_edge_v1 \
                 (campaign_id, resolve_tick, edge_type, source_local_name, target_local_name, strength_bits) \
                 VALUES ($1::uuid, $2, $3, $4, $5, $6)",
                &[
                    campaign_id.as_uuid(),
                    &resolve_tick,
                    edge_type,
                    source,
                    target,
                    &strength_bits,
                ],
            ),
            "insert graph edge",
        )?;
    }
    for (local_name, hyperedge_type, _) in rows.hyperedges() {
        require_single_insert(
            client.execute(
                "INSERT INTO babylon_state.graph_hyperedge_v1 \
                 (campaign_id, resolve_tick, local_name, hyperedge_type) VALUES ($1::uuid, $2, $3, $4)",
                &[campaign_id.as_uuid(), &resolve_tick, local_name, hyperedge_type],
            ),
            "insert graph hyperedge",
        )?;
    }
    let sink = client
        .copy_in(
            "COPY babylon_state.graph_hyperedge_member_v1 \
         (campaign_id, resolve_tick, local_name, position, member) FROM STDIN BINARY",
        )
        .map_err(|error| {
            RustPersistenceRuntimeError::postgres("begin graph hyperedge member copy", &error)
        })?;
    let mut writer = BinaryCopyInWriter::new(
        sink,
        &[Type::UUID, Type::INT8, Type::TEXT, Type::INT4, Type::TEXT],
    );
    let mut expected = 0_usize;
    for (local_name, _, members) in rows.hyperedges() {
        for (position, member) in members.iter().enumerate() {
            writer
                .write(&[
                    campaign_id.as_uuid(),
                    &resolve_tick,
                    local_name,
                    &checked_position(position)?,
                    member,
                ])
                .map_err(|error| {
                    RustPersistenceRuntimeError::postgres(
                        "write graph hyperedge member copy",
                        &error,
                    )
                })?;
            expected = expected
                .checked_add(1)
                .ok_or(RustPersistenceRuntimeError::CampaignConflict)?;
        }
    }
    finish_binary_copy(writer, expected, "finish graph hyperedge member copy")?;
    insert_graph_value_rows(client, campaign_id, resolve_tick, report)
}

fn insert_graph_value_rows(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReport,
) -> Result<(), RustPersistenceRuntimeError> {
    let rows = report.result_stable_graph().rows();
    for (edge_type, source, target, qname, bits) in rows.edge_f64() {
        let value_bits = bit_pattern_i64(*bits);
        require_single_insert(
            client.execute(
                "INSERT INTO babylon_state.graph_edge_f64_v1 \
                 (campaign_id, resolve_tick, edge_type, source_local_name, target_local_name, qname, value_bits) \
                 VALUES ($1::uuid, $2, $3, $4, $5, $6, $7)",
                &[
                    campaign_id.as_uuid(),
                    &resolve_tick,
                    edge_type,
                    source,
                    target,
                    qname,
                    &value_bits,
                ],
            ),
            "insert graph edge f64",
        )?;
    }
    for (local_name, qname, micro_units) in rows.node_currency() {
        let decimal = micro_units.to_string();
        require_single_insert(
            client.execute(
                "INSERT INTO babylon_state.graph_node_currency_v1 \
                 (campaign_id, resolve_tick, local_name, qname, micro_units) \
                 VALUES ($1::uuid, $2, $3, $4, $5::text::numeric)",
                &[
                    campaign_id.as_uuid(),
                    &resolve_tick,
                    local_name,
                    qname,
                    &decimal,
                ],
            ),
            "insert graph node currency",
        )?;
    }
    for (local_name, qname, bits) in rows.hyperedge_f64() {
        let value_bits = bit_pattern_i64(*bits);
        require_single_insert(
            client.execute(
                "INSERT INTO babylon_state.graph_hyperedge_f64_v1 \
                 (campaign_id, resolve_tick, local_name, qname, value_bits) \
                 VALUES ($1::uuid, $2, $3, $4, $5)",
                &[
                    campaign_id.as_uuid(),
                    &resolve_tick,
                    local_name,
                    qname,
                    &value_bits,
                ],
            ),
            "insert graph hyperedge f64",
        )?;
    }
    Ok(())
}

fn insert_typed_material_rows(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReport,
) -> Result<(), RustPersistenceRuntimeError> {
    let rows = report.material_state_rows();
    for row in rows.world_registers().rows() {
        let prefix: [&(dyn ToSql + Sync); 3] = [campaign_id.as_uuid(), &resolve_tick, &row.qname()];
        insert_bsl_value_row(
            client,
            "INSERT INTO babylon_state.world_register_v1 \
             (campaign_id, resolve_tick, register_name, value_tag, int_value, currency_value, \
              real_bits, ratio_bits, ratio_min_bits, ratio_max_bits, bool_value, enum_type, enum_member, stable_key) \
             VALUES ($1::uuid, $2, $3, $4, $5, $6::text::numeric, $7, $8, $9, $10, $11, $12, $13, $14)",
            &prefix,
            row.value(),
            "insert world register",
        )?;
    }
    for row in rows.territories().rows() {
        let territory_id = stable_key_bytes(row.territory_id())?;
        require_single_insert(
            client.execute(
                "INSERT INTO babylon_state.territory_state_v1 \
                 (campaign_id, resolve_tick, territory_id) VALUES ($1::uuid, $2, $3)",
                &[campaign_id.as_uuid(), &resolve_tick, &territory_id],
            ),
            "insert territory state",
        )?;
    }
    let campaign = campaign_id.as_uuid().to_string();
    let tick = resolve_tick.to_string();
    let mut writer = client
        .copy_in(
            "COPY babylon_state.territory_state_field_v1 \
         (campaign_id, resolve_tick, territory_id, position, field_name, value_tag, int_value, \
          currency_value, real_bits, ratio_bits, ratio_min_bits, ratio_max_bits, bool_value, \
          enum_type, enum_member, stable_key) FROM STDIN WITH (FORMAT csv)",
        )
        .map_err(|error| {
            RustPersistenceRuntimeError::postgres("begin territory field copy", &error)
        })?;
    let mut expected = 0_usize;
    for row in rows.territories().rows() {
        let territory = bytea_copy_text(&stable_key_bytes(row.territory_id())?);
        for (position, (field_name, value)) in row.ordered_fields().iter().enumerate() {
            let position = checked_position(position)?.to_string();
            write_bsl_csv_row(
                &mut writer,
                &[&campaign, &tick, &territory, &position, field_name],
                value,
                "write territory field copy",
            )?;
            expected = expected
                .checked_add(1)
                .ok_or(RustPersistenceRuntimeError::CampaignConflict)?;
        }
    }
    finish_csv_copy(writer, expected, "finish territory field copy")?;
    insert_dynamic_hex_rows(client, campaign_id, resolve_tick, report)?;
    insert_organization_state_rows(client, campaign_id, resolve_tick, report)
}

fn insert_dynamic_hex_rows(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReport,
) -> Result<(), RustPersistenceRuntimeError> {
    let rows = report.material_state_rows().dynamic_hexes().rows();
    let expected =
        u64::try_from(rows.len()).map_err(|_| RustPersistenceRuntimeError::CampaignConflict)?;
    let sink = client
        .copy_in(
            "COPY babylon_state.hex_state_delta_v1 \
             (campaign_id, resolve_tick, cell_id, c_bits, v_bits, s_bits, k_bits, \
              biocapacity_stock_bits, energy_stock_bits, raw_material_stock_bits, \
              internet_access_pct_bits, surveillance_coupling_bits) FROM STDIN BINARY",
        )
        .map_err(|error| {
            RustPersistenceRuntimeError::postgres("begin dynamic hex state copy", &error)
        })?;
    let mut writer = BinaryCopyInWriter::new(
        sink,
        &[
            Type::UUID,
            Type::INT8,
            Type::INT8,
            Type::INT8,
            Type::INT8,
            Type::INT8,
            Type::INT8,
            Type::INT8,
            Type::INT8,
            Type::INT8,
            Type::INT8,
            Type::INT8,
        ],
    );
    for row in rows {
        let cell_id = i64::try_from(row.cell_id().as_u64())
            .map_err(|_| RustPersistenceRuntimeError::SemanticCodec)?;
        let values = row.value_bits().map(bit_pattern_i64);
        writer
            .write(&[
                campaign_id.as_uuid(),
                &resolve_tick,
                &cell_id,
                &values[0],
                &values[1],
                &values[2],
                &values[3],
                &values[4],
                &values[5],
                &values[6],
                &values[7],
                &values[8],
            ])
            .map_err(|error| {
                RustPersistenceRuntimeError::postgres("write dynamic hex state copy", &error)
            })?;
    }
    let inserted = writer.finish().map_err(|error| {
        RustPersistenceRuntimeError::postgres("finish dynamic hex state copy", &error)
    })?;
    if inserted != expected {
        return Err(RustPersistenceRuntimeError::database(
            "count dynamic hex state copy",
        ));
    }
    Ok(())
}

fn insert_organization_state_rows(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReport,
) -> Result<(), RustPersistenceRuntimeError> {
    let rows = report.material_state_rows().organizations().rows();
    let campaign = campaign_id.as_uuid().to_string();
    let tick = resolve_tick.to_string();
    let mut writer = client.copy_in(
        "COPY babylon_state.organization_state_v1 \
         (campaign_id, resolve_tick, organization_id, organization_kind_tag, \
          organization_kind_int, organization_kind_currency, organization_kind_real_bits, \
          organization_kind_ratio_bits, organization_kind_ratio_min_bits, \
          organization_kind_ratio_max_bits, organization_kind_bool, \
          organization_kind_enum_type, organization_kind_enum_member, organization_kind_stable_key) \
         FROM STDIN WITH (FORMAT csv)",
    ).map_err(|error| RustPersistenceRuntimeError::postgres("begin organization state copy", &error))?;
    for row in rows {
        let organization = bytea_copy_text(&stable_key_bytes(row.organization_id())?);
        write_bsl_csv_row(
            &mut writer,
            &[&campaign, &tick, &organization],
            row.organization_kind(),
            "write organization state copy",
        )?;
    }
    finish_csv_copy(writer, rows.len(), "finish organization state copy")?;
    // Copy each parent family before its children. All ordering is retained in
    // the explicit position columns; the marker remains the transaction's last row.
    for row in rows {
        let organization_id = stable_key_bytes(row.organization_id())?;
        for (position, territory_id) in row.ordered_territory_ids().iter().enumerate() {
            let position = checked_position(position)?;
            let territory_id = stable_key_bytes(territory_id)?;
            require_single_insert(
                client.execute(
                    "INSERT INTO babylon_state.organization_territory_v1 \
                     (campaign_id, resolve_tick, organization_id, position, territory_id) \
                     VALUES ($1::uuid, $2, $3, $4, $5)",
                    &[
                        campaign_id.as_uuid(),
                        &resolve_tick,
                        &organization_id,
                        &position,
                        &territory_id,
                    ],
                ),
                "insert organization territory",
            )?;
        }
    }
    let mut writer = client
        .copy_in(
            "COPY babylon_state.organization_state_field_v1 \
         (campaign_id, resolve_tick, organization_id, position, field_name, value_tag, int_value, \
          currency_value, real_bits, ratio_bits, ratio_min_bits, ratio_max_bits, bool_value, \
          enum_type, enum_member, stable_key) FROM STDIN WITH (FORMAT csv)",
        )
        .map_err(|error| {
            RustPersistenceRuntimeError::postgres("begin organization field copy", &error)
        })?;
    let mut expected = 0_usize;
    for row in rows {
        let organization = bytea_copy_text(&stable_key_bytes(row.organization_id())?);
        for (position, (field_name, value)) in row.ordered_fields().iter().enumerate() {
            let position = checked_position(position)?.to_string();
            write_bsl_csv_row(
                &mut writer,
                &[&campaign, &tick, &organization, &position, field_name],
                value,
                "write organization field copy",
            )?;
            expected = expected
                .checked_add(1)
                .ok_or(RustPersistenceRuntimeError::CampaignConflict)?;
        }
    }
    finish_csv_copy(writer, expected, "finish organization field copy")
}

fn finish_binary_copy(
    writer: BinaryCopyInWriter<'_>,
    expected: usize,
    operation: &'static str,
) -> Result<(), RustPersistenceRuntimeError> {
    let inserted = writer
        .finish()
        .map_err(|error| RustPersistenceRuntimeError::postgres(operation, &error))?;
    require_copy_count(inserted, expected, operation)
}

fn finish_csv_copy(
    writer: postgres::CopyInWriter<'_>,
    expected: usize,
    operation: &'static str,
) -> Result<(), RustPersistenceRuntimeError> {
    let inserted = writer
        .finish()
        .map_err(|error| RustPersistenceRuntimeError::postgres(operation, &error))?;
    require_copy_count(inserted, expected, operation)
}

fn require_copy_count(
    inserted: u64,
    expected: usize,
    operation: &'static str,
) -> Result<(), RustPersistenceRuntimeError> {
    let expected =
        u64::try_from(expected).map_err(|_| RustPersistenceRuntimeError::CampaignConflict)?;
    if inserted != expected {
        return Err(RustPersistenceRuntimeError::database(operation));
    }
    Ok(())
}

fn bytea_copy_text(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::from("\\x");
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 15)]));
    }
    encoded
}

fn write_bsl_csv_row(
    writer: &mut impl std::io::Write,
    prefix: &[&str],
    value: &StableBslValue,
    operation: &'static str,
) -> Result<(), RustPersistenceRuntimeError> {
    let value = BslSqlValue::from_stable(value)?;
    let fields = [
        Some(value.tag.to_string()),
        value.int_value.map(|value| value.to_string()),
        value.currency_value,
        value.real_bits.map(|value| value.to_string()),
        value.ratio_bits.map(|value| value.to_string()),
        value.ratio_min_bits.map(|value| value.to_string()),
        value.ratio_max_bits.map(|value| value.to_string()),
        value.bool_value.map(|value| value.to_string()),
        value.enum_type,
        value.enum_member,
        value.stable_key.as_deref().map(bytea_copy_text),
    ];
    let fields = prefix
        .iter()
        .map(|value| Some(*value))
        .chain(fields.iter().map(Option::as_deref));
    write_csv_row(writer, fields).map_err(|error| {
        if let Some(postgres) = error
            .get_ref()
            .and_then(|error| error.downcast_ref::<postgres::Error>())
        {
            RustPersistenceRuntimeError::postgres(operation, postgres)
        } else {
            RustPersistenceRuntimeError::database(operation)
        }
    })
}

// PostgreSQL CSV distinguishes NULL (unquoted empty) from an empty string
// (quoted empty). Quote every present field, doubling only embedded quotes;
// unlike text COPY, CSV leaves bytea's hexadecimal backslash untouched.
fn write_csv_row<'a>(
    writer: &mut impl std::io::Write,
    fields: impl Iterator<Item = Option<&'a str>>,
) -> std::io::Result<()> {
    for (position, field) in fields.enumerate() {
        if position != 0 {
            writer.write_all(b",")?;
        }
        if let Some(field) = field {
            writer.write_all(b"\"")?;
            for (part, fragment) in field.split('"').enumerate() {
                if part != 0 {
                    writer.write_all(b"\"\"")?;
                }
                writer.write_all(fragment.as_bytes())?;
            }
            writer.write_all(b"\"")?;
        }
    }
    writer.write_all(b"\n")
}

fn insert_choice_receipt_rows(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReport,
) -> Result<(), RustPersistenceRuntimeError> {
    for receipt in &report.report().choice_receipts {
        let encounter_ordinal = i64::from(receipt.encounter_ordinal());
        let slot = i64::from(receipt.slot());
        let stable_carrier = stable_key_bytes(receipt.stable_carrier())?;
        let draw_ticket = receipt.draw_ticket().to_string();
        require_single_insert(
            client.execute(
                "INSERT INTO babylon_state.tick_choice_receipt_v1 \
                 (campaign_id, resolve_tick, encounter_ordinal, rule_id, sample, slot, \
                  outcome_enum, stable_carrier, draw_ticket, selected_outcome, \
                  allocation_digest, instance_digest) \
                 VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8, $9::text::numeric, $10, $11, $12)",
                &[
                    campaign_id.as_uuid(),
                    &resolve_tick,
                    &encounter_ordinal,
                    &receipt.rule_id(),
                    &receipt.sample(),
                    &slot,
                    &receipt.outcome_enum(),
                    &stable_carrier,
                    &draw_ticket,
                    &receipt.selected_outcome(),
                    &&receipt.allocation_digest()[..],
                    &&receipt.instance_digest()[..],
                ],
            ),
            "insert choice receipt",
        )?;
        for (position, branch) in receipt.branches().iter().enumerate() {
            let position = i64::from(checked_u32_position(position)?);
            let mass_nanounits = branch.mass.nanounits().to_string();
            let ticket_start = branch.tickets.start.to_string();
            let ticket_end_exclusive = branch.tickets.end.to_string();
            let ticket_count = branch.tickets.count.to_string();
            require_single_insert(
                client.execute(
                    "INSERT INTO babylon_state.tick_choice_receipt_branch_v1 \
                     (campaign_id, resolve_tick, encounter_ordinal, position, outcome_member, \
                      mass_nanounits, ticket_start, ticket_end_exclusive, ticket_count) \
                     VALUES ($1::uuid, $2, $3, $4, $5, $6::text::numeric, \
                             $7::text::numeric, $8::text::numeric, $9::text::numeric)",
                    &[
                        campaign_id.as_uuid(),
                        &resolve_tick,
                        &encounter_ordinal,
                        &position,
                        &branch.member,
                        &mass_nanounits,
                        &ticket_start,
                        &ticket_end_exclusive,
                        &ticket_count,
                    ],
                ),
                "insert choice receipt branch",
            )?;
        }
        for (position, element) in receipt.active_elements().iter().enumerate() {
            let position = i64::from(checked_u32_position(position)?);
            let stable_element = stable_key_bytes(element)?;
            require_single_insert(
                client.execute(
                    "INSERT INTO babylon_state.tick_choice_receipt_carrier_element_v1 \
                     (campaign_id, resolve_tick, encounter_ordinal, position, stable_element) \
                     VALUES ($1::uuid, $2, $3, $4, $5)",
                    &[
                        campaign_id.as_uuid(),
                        &resolve_tick,
                        &encounter_ordinal,
                        &position,
                        &stable_element,
                    ],
                ),
                "insert choice receipt carrier element",
            )?;
        }
    }
    Ok(())
}

fn insert_typed_event_rows(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReport,
) -> Result<(), RustPersistenceRuntimeError> {
    for (ordinal, event) in report.successful_event_batch().events().iter().enumerate() {
        let ordinal =
            i64::try_from(ordinal).map_err(|_| RustPersistenceRuntimeError::IntegerConversion {
                field: "successful event ordinal",
                value: ordinal,
            })?;
        let choice_receipt_ordinal = event
            .choice_receipt()
            .map(|reference| i64::from(reference.encounter_ordinal()));
        require_single_insert(
            client.execute(
                "INSERT INTO babylon_state.tick_event_v2 \
                 (campaign_id, resolve_tick, ordinal, event_type, emitting_rule, \
                  choice_receipt_ordinal) VALUES ($1::uuid, $2, $3, $4, $5, $6)",
                &[
                    campaign_id.as_uuid(),
                    &resolve_tick,
                    &ordinal,
                    &event.event_type(),
                    &event.emitting_rule(),
                    &choice_receipt_ordinal,
                ],
            ),
            "insert tick event",
        )?;
        for (position, (field_name, value)) in event.fields().iter().enumerate() {
            let position = i64::from(checked_u32_position(position)?);
            let prefix: [&(dyn ToSql + Sync); 5] = [
                campaign_id.as_uuid(),
                &resolve_tick,
                &ordinal,
                &position,
                field_name,
            ];
            insert_bsl_value_row(
                client,
                "INSERT INTO babylon_state.tick_event_field_v2 \
                 (campaign_id, resolve_tick, ordinal, position, field_name, value_tag, int_value, \
                  currency_value, real_bits, ratio_bits, ratio_min_bits, ratio_max_bits, bool_value, \
                  enum_type, enum_member, stable_key) \
                 VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8::text::numeric, $9, $10, $11, $12, $13, $14, $15, $16)",
                &prefix,
                value,
                "insert tick event field",
            )?;
        }
    }
    Ok(())
}

fn insert_full_checkpoint(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    checkpoint: &CommittedFullCheckpoint,
) -> Result<(), RustPersistenceRuntimeError> {
    let completeness_tag = 1_i16;
    require_single_insert(
        client.execute(
            "INSERT INTO babylon_state.checkpoint_manifest \
             (campaign_id, resolve_tick, completeness_tag, manifest_bytes, manifest_sha256) \
             VALUES ($1::uuid, $2, $3, $4, $5)",
            &[
                campaign_id.as_uuid(),
                &resolve_tick,
                &completeness_tag,
                &checkpoint.manifest_bytes(),
                &&checkpoint.manifest_sha256()[..],
            ],
        ),
        "insert checkpoint manifest",
    )?;
    if checkpoint.sections().len() != checkpoint.exact_section_bytes().len() {
        return Err(RustPersistenceRuntimeError::CampaignConflict);
    }
    for (section, exact_bytes) in checkpoint
        .sections()
        .iter()
        .zip(checkpoint.exact_section_bytes())
    {
        if sha256_of(exact_bytes) != section.sha256() {
            return Err(RustPersistenceRuntimeError::CampaignConflict);
        }
        let section_tag = i16::from(section.tag().tag());
        let ordinal = 0_i64;
        require_single_insert(
            client.execute(
                "INSERT INTO babylon_state.checkpoint_section_v1 \
                 (campaign_id, resolve_tick, section_tag, ordinal, exact_section_bytes) \
                 VALUES ($1::uuid, $2, $3, $4, $5)",
                &[
                    campaign_id.as_uuid(),
                    &resolve_tick,
                    &section_tag,
                    &ordinal,
                    exact_bytes,
                ],
            ),
            "insert checkpoint section",
        )?;
    }
    Ok(())
}

struct BslSqlValue {
    tag: i16,
    int_value: Option<i64>,
    currency_value: Option<String>,
    real_bits: Option<i64>,
    ratio_bits: Option<i64>,
    ratio_min_bits: Option<i64>,
    ratio_max_bits: Option<i64>,
    bool_value: Option<bool>,
    enum_type: Option<String>,
    enum_member: Option<String>,
    stable_key: Option<Vec<u8>>,
}

impl BslSqlValue {
    fn from_stable(value: &StableBslValue) -> Result<Self, RustPersistenceRuntimeError> {
        let mut row = Self {
            tag: 0,
            int_value: None,
            currency_value: None,
            real_bits: None,
            ratio_bits: None,
            ratio_min_bits: None,
            ratio_max_bits: None,
            bool_value: None,
            enum_type: None,
            enum_member: None,
            stable_key: None,
        };
        match value {
            StableBslValue::Int(value) => {
                row.tag = 1;
                row.int_value = Some(*value);
            }
            StableBslValue::CurrencyMicroUnits(value) => {
                row.tag = 2;
                row.currency_value = Some(value.to_string());
            }
            StableBslValue::RealBits(bits) => {
                row.tag = 3;
                row.real_bits = Some(bit_pattern_i64(*bits));
            }
            StableBslValue::RatioBits { value, floor, cap } => {
                row.tag = 4;
                row.ratio_bits = Some(bit_pattern_i64(*value));
                row.ratio_min_bits = floor.map(bit_pattern_i64);
                row.ratio_max_bits = cap.map(bit_pattern_i64);
            }
            StableBslValue::Bool(value) => {
                row.tag = 5;
                row.bool_value = Some(*value);
            }
            StableBslValue::Enum { enum_type, member } => {
                row.tag = 6;
                row.enum_type = Some(enum_type.clone());
                row.enum_member = Some(member.clone());
            }
            StableBslValue::Node(key) => {
                row.tag = 7;
                row.stable_key = Some(stable_key_bytes(key)?);
            }
            StableBslValue::Hyperedge(key) => {
                row.tag = 8;
                row.stable_key = Some(stable_key_bytes(key)?);
            }
            StableBslValue::Edge(key) => {
                row.tag = 9;
                row.stable_key = Some(stable_key_bytes(key)?);
            }
        }
        Ok(row)
    }
}

fn insert_bsl_value_row(
    client: &mut impl GenericClient,
    sql: &str,
    prefix: &[&(dyn ToSql + Sync)],
    value: &StableBslValue,
    operation: &'static str,
) -> Result<(), RustPersistenceRuntimeError> {
    let value = BslSqlValue::from_stable(value)?;
    let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();
    params.try_reserve_exact(prefix.len() + 11).map_err(|_| {
        RustPersistenceRuntimeError::Allocation {
            field: "typed BSL SQL parameters",
            requested: prefix.len() + 11,
        }
    })?;
    params.extend_from_slice(prefix);
    params.extend_from_slice(&[
        &value.tag,
        &value.int_value,
        &value.currency_value,
        &value.real_bits,
        &value.ratio_bits,
        &value.ratio_min_bits,
        &value.ratio_max_bits,
        &value.bool_value,
        &value.enum_type,
        &value.enum_member,
        &value.stable_key,
    ]);
    require_single_insert(client.execute(sql, &params), operation)
}

fn stable_key_bytes(
    key: &babylon_graph::stable_element::StableElementKey,
) -> Result<Vec<u8>, RustPersistenceRuntimeError> {
    key.canonical_bytes()
        .map_err(|_| RustPersistenceRuntimeError::SemanticCodec)
}

fn bit_pattern_i64(bits: u64) -> i64 {
    i64::from_be_bytes(bits.to_be_bytes())
}

fn checked_position(position: usize) -> Result<i32, RustPersistenceRuntimeError> {
    i32::try_from(position).map_err(|_| RustPersistenceRuntimeError::IntegerConversion {
        field: "typed child position",
        value: position,
    })
}

fn checked_u32_position(position: usize) -> Result<u32, RustPersistenceRuntimeError> {
    u32::try_from(position).map_err(|_| RustPersistenceRuntimeError::IntegerConversion {
        field: "V2 ordered position",
        value: position,
    })
}

fn require_single_insert(
    result: Result<u64, postgres::Error>,
    operation: &'static str,
) -> Result<(), RustPersistenceRuntimeError> {
    let affected =
        result.map_err(|error| RustPersistenceRuntimeError::postgres(operation, &error))?;
    if affected == 1 {
        Ok(())
    } else {
        Err(RustPersistenceRuntimeError::database(operation))
    }
}

fn decode_runtime_column<T: postgres::types::FromSqlOwned>(
    row: &postgres::Row,
    index: usize,
) -> Result<T, RustPersistenceRuntimeError> {
    row.try_get(index)
        .map_err(|_| RustPersistenceRuntimeError::CampaignConflict)
}

fn decode_digest_column(
    row: &postgres::Row,
    index: usize,
) -> Result<[u8; 32], RustPersistenceRuntimeError> {
    let bytes: Vec<u8> = decode_runtime_column(row, index)?;
    bytes
        .try_into()
        .map_err(|_| RustPersistenceRuntimeError::CampaignConflict)
}

/// Exact report-derived inputs held before a durable commit attempt.
///
/// This type owns no replay engine and cannot adjudicate or recompute a tick.
/// It closes every live V2 row family and converts to an envelope only after
/// all semantic batches and checkpoint sections have composed successfully.
#[derive(Debug, PartialEq, Eq)]
pub struct PreparedCommittedTick {
    resolve_tick: CommittedResolveTick,
    tick_content_hash: TickContentHash,
    graph_event_batches: GraphEventChoiceSemanticBatches,
    material_state_rows: Vec<CommittedTickRow>,
    checkpoint_rows: CheckpointRows,
    archive_dirty_receipt: ArchiveDirtyReceipt,
}

impl PreparedCommittedTick {
    /// Return the exact positive resolve tick carried by the source report.
    #[must_use]
    pub const fn resolve_tick(&self) -> CommittedResolveTick {
        self.resolve_tick
    }

    /// Return the constitutional content identity carried by the source report.
    #[must_use]
    pub const fn tick_content_hash(&self) -> TickContentHash {
        self.tick_content_hash
    }

    /// Borrow the exact report-derived checkpoint producer.
    #[must_use]
    pub const fn checkpoint_rows(&self) -> &CheckpointRows {
        &self.checkpoint_rows
    }

    /// Borrow the exact singular Archive work receipt.
    #[must_use]
    pub const fn archive_dirty_receipt(&self) -> &ArchiveDirtyReceipt {
        &self.archive_dirty_receipt
    }

    pub(crate) fn into_material_families(
        self,
        hash: TickContentHash,
    ) -> Result<CommittedTickRowFamilies, RustPersistenceRuntimeError> {
        let (graph, event, choice_receipt) = self.graph_event_batches.into_rows();
        Ok(CommittedTickRowFamilies {
            graph,
            state: self.material_state_rows,
            event,
            choice_receipt,
            checkpoint: self.checkpoint_rows.into_rows(),
            archive_dirty_receipt: crate::semantic_codec::encode_archive_dirty_receipt(
                hash.as_bytes(),
            )?,
        })
    }
}

/// Derive one stopped, database-free durable candidate from one identified report.
///
/// # Errors
/// Returns the first resolve-tick, codec, allocation, or aggregate-bound
/// refusal. This function never parses rules, executes a tick, or judges game
/// mechanics; every semantic source comes from `report`.
pub fn prepare_committed_tick(
    report: &IdentifiedTickReport,
) -> Result<PreparedCommittedTick, RustPersistenceRuntimeError> {
    let completed_tick = report.result_registers().completed_tick();
    let raw_resolve_tick = u64::try_from(completed_tick).map_err(|_| {
        RustPersistenceRuntimeError::ResolveTickOutOfRange {
            actual: completed_tick,
        }
    })?;
    let resolve_tick = CommittedResolveTick::try_from(raw_resolve_tick).map_err(
        |_: CommittedResolveTickError| RustPersistenceRuntimeError::ResolveTickOutOfRange {
            actual: completed_tick,
        },
    )?;
    let graph_event_batches = compose_graph_event_choice_semantic_batches(report)?;
    let material_state_rows = compose_material_state_rows(report.material_state_rows())?;
    let checkpoint_rows = compose_checkpoint_rows(report, resolve_tick)?;
    let archive_dirty_receipt = compose_archive_dirty_receipt(report)?;
    Ok(PreparedCommittedTick {
        resolve_tick,
        tick_content_hash: report.tick_content_hash(),
        graph_event_batches,
        material_state_rows,
        checkpoint_rows,
        archive_dirty_receipt,
    })
}

#[cfg(test)]
mod live_tests {
    use super::*;
    use postgres::{Config, NoTls};
    use std::str::FromStr;
    use uuid::Uuid;
    const DSN_ENV: &str = "BABYLON_POSTGRES_TEST_DSN";
    const ACK_ENV: &str = "BABYLON_POSTGRES_DISPOSABLE_ACK";
    const ACK: &str = "I_UNDERSTAND_THIS_DISPOSABLE_RUNTIME_DROPS_ITS_SCRATCH_DATABASES_AND_ROLES";
    const CANARY_ENV: &str = "BABYLON_POSTGRES_DISPOSABLE_CANARY";
    const TEMPLATE_DB_ENV: &str = "BABYLON_RUNTIME_TEMPLATE_DB";
    #[test]
    #[ignore = "requires the task-owned disposable PostgreSQL runtime"]
    fn live_material_commit_loss_reconciles_only_the_complete_persisted_candidate() {
        use crate::material_runtime::{CommitFault, DurableMaterialRuntime, COMMIT_FAULT};
        let base = validated_base_config();
        let database = TestDatabase::create_from_template(
            &base,
            &validated_template_name(),
            "materialcommitloss",
        );
        let config = database.config(&base);
        let catalog = crate::michigan_material::MichiganMaterialCatalog::from_defines_toml(
            include_str!("../../../../content/scenarios/michigan/defines.toml"),
        )
        .unwrap();
        let foundation = crate::michigan_content::MichiganContentPreset::new_campaign(
            crate::michigan_material::MichiganDeliveryPreset::Standard,
        )
        .create_foundation(&catalog)
        .unwrap();
        let digest = foundation.digest();
        let campaign = CampaignId::from_uuid(Uuid::from_u128(0x0033_0c01_1055));
        let mut runtime = DurableMaterialRuntime::create(&config, campaign, foundation).unwrap();
        let before = runtime.session().material().canonical_bytes().to_vec();
        let mut sink = CollectingSink::default();
        let actions = OrderedPracticeActionBatch::empty(
            runtime.session().graph_session().session_identity().clone(),
            1,
        )
        .unwrap();
        COMMIT_FAULT.with(|slot| slot.set(Some(CommitFault::BeforeCommit)));
        assert!(runtime.advance_and_commit(&mut sink, &actions).is_err());
        assert_eq!(runtime.session().completed_tick(), 0);
        assert_eq!(runtime.session().material().canonical_bytes(), before);
        assert!(runtime.diagnostic_receipt().is_none());
        assert!(sink.events.is_empty());
        let mut owner = config.connect(NoTls).unwrap();
        let count: i64 = owner
            .query_one(
                "SELECT count(*) FROM babylon_state.tick_commit WHERE campaign_id=$1::uuid",
                &[campaign.as_uuid()],
            )
            .unwrap()
            .get(0);
        assert_eq!(count, 0);
        // Retry the exact action after rollback, then lose the next COMMIT acknowledgement.
        runtime.advance_and_commit(&mut sink, &actions).unwrap();
        let mut stale = DurableMaterialRuntime::open(&config, campaign, digest).unwrap();
        let actions = OrderedPracticeActionBatch::empty(
            runtime.session().graph_session().session_identity().clone(),
            2,
        )
        .unwrap();
        COMMIT_FAULT.with(|slot| slot.set(Some(CommitFault::AfterCommit)));
        let identity = runtime.advance_and_commit(&mut sink, &actions).unwrap();
        assert_eq!(
            runtime.diagnostic_receipt().unwrap().commit_disposition(),
            ReplayCommitDisposition::ReconciledAfterAmbiguousCommit
        );
        let reopened = DurableMaterialRuntime::open(&config, campaign, digest).unwrap();
        assert_eq!(reopened.tail(), Some(&identity));
        assert_eq!(
            reopened.session().material().canonical_bytes(),
            runtime.session().material().canonical_bytes()
        );
        let count: i64 = owner
            .query_one(
                "SELECT count(*) FROM babylon_state.tick_commit WHERE campaign_id=$1::uuid",
                &[campaign.as_uuid()],
            )
            .unwrap()
            .get(0);
        assert_eq!(count, 2);
        let original: Vec<u8> = owner.query_one("SELECT receipt_bytes FROM babylon_state.material_tick_v3 WHERE campaign_id=$1::uuid AND resolve_tick=2", &[campaign.as_uuid()]).unwrap().get(0);
        let mut corrupt = original.clone();
        let last = corrupt.len() - 1;
        corrupt[last] ^= 1;
        owner.execute("UPDATE babylon_state.material_tick_v3 SET receipt_bytes=$2 WHERE campaign_id=$1::uuid AND resolve_tick=2", &[campaign.as_uuid(), &corrupt]).unwrap();
        let mut refused_sink = CollectingSink::default();
        assert!(stale
            .advance_and_commit(&mut refused_sink, &actions)
            .is_err());
        assert_eq!(stale.session().completed_tick(), 1);
        assert!(refused_sink.events.is_empty());
        owner.execute("UPDATE babylon_state.material_tick_v3 SET receipt_bytes=$2 WHERE campaign_id=$1::uuid AND resolve_tick=2", &[campaign.as_uuid(), &original]).unwrap();
        assert_eq!(
            stale
                .advance_and_commit(&mut refused_sink, &actions)
                .unwrap(),
            identity
        );
        drop(owner);
        drop(reopened);
        drop(stale);
        drop(runtime);
        database.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PostgreSQL runtime"]
    fn live_bsl_csv_copy_preserves_numeric_extremes_nulls_and_stable_keys() {
        use babylon_graph::stable_element::StableElementKey;
        let base = validated_base_config();
        let database =
            TestDatabase::create_from_template(&base, &validated_template_name(), "copyvalues");
        let config = database.config(&base);
        let node = StableElementKey::Node {
            scenario: "production/copy".to_owned(),
            local_name: "source".to_owned(),
        };
        let values = [
            StableBslValue::Int(i64::MIN),
            StableBslValue::Int(i64::MAX),
            StableBslValue::CurrencyMicroUnits(i128::MIN),
            StableBslValue::CurrencyMicroUnits(i128::MAX),
            StableBslValue::RealBits((-0.25_f64).to_bits()),
            StableBslValue::RatioBits {
                value: 0.5_f64.to_bits(),
                floor: None,
                cap: Some(1.0_f64.to_bits()),
            },
            StableBslValue::RatioBits {
                value: 0.5_f64.to_bits(),
                floor: Some(0.0_f64.to_bits()),
                cap: None,
            },
            StableBslValue::Bool(false),
            StableBslValue::Bool(true),
            StableBslValue::Enum {
                enum_type: "OrganizationKind".to_owned(),
                member: "COLLECTIVE".to_owned(),
            },
            StableBslValue::Node(node),
            StableBslValue::Hyperedge(StableElementKey::Hyperedge {
                scenario: "production/copy".to_owned(),
                local_name: "assembly".to_owned(),
            }),
            StableBslValue::Edge(StableElementKey::Edge {
                scenario: "production/copy".to_owned(),
                edge_type: "production/links".to_owned(),
                source_local_name: "source".to_owned(),
                target_local_name: "target".to_owned(),
            }),
        ];
        let mut client = config.connect(NoTls).unwrap();
        let mut tx = client.transaction().unwrap();
        let mut writer = tx.copy_in(
            "COPY babylon_state.organization_state_v1 \
             (campaign_id,resolve_tick,organization_id,organization_kind_tag,organization_kind_int, \
              organization_kind_currency,organization_kind_real_bits,organization_kind_ratio_bits, \
              organization_kind_ratio_min_bits,organization_kind_ratio_max_bits,organization_kind_bool, \
              organization_kind_enum_type,organization_kind_enum_member,organization_kind_stable_key) \
             FROM STDIN WITH (FORMAT csv)",
        ).unwrap();
        for (index, value) in values.iter().enumerate() {
            let id = bytea_copy_text(&index.to_be_bytes());
            write_bsl_csv_row(
                &mut writer,
                &["00000000-0000-0000-0000-000000000001", "1", &id],
                value,
                "copy value proof",
            )
            .unwrap();
        }
        finish_csv_copy(writer, values.len(), "finish value proof").unwrap();
        for (index, value) in values.iter().enumerate() {
            let row = tx.query_one(
                "SELECT organization_kind_tag,organization_kind_int,organization_kind_currency::text, \
                 organization_kind_real_bits,organization_kind_ratio_bits,organization_kind_ratio_min_bits, \
                 organization_kind_ratio_max_bits,organization_kind_bool,organization_kind_enum_type, \
                 organization_kind_enum_member,organization_kind_stable_key \
                 FROM babylon_state.organization_state_v1 WHERE organization_id=$1", &[&&index.to_be_bytes()[..]],
            ).unwrap();
            let expected = BslSqlValue::from_stable(value).unwrap();
            assert_eq!(row.get::<_, i16>(0), expected.tag);
            assert_eq!(row.get::<_, Option<i64>>(1), expected.int_value);
            assert_eq!(row.get::<_, Option<String>>(2), expected.currency_value);
            assert_eq!(row.get::<_, Option<i64>>(3), expected.real_bits);
            assert_eq!(row.get::<_, Option<i64>>(4), expected.ratio_bits);
            assert_eq!(row.get::<_, Option<i64>>(5), expected.ratio_min_bits);
            assert_eq!(row.get::<_, Option<i64>>(6), expected.ratio_max_bits);
            assert_eq!(row.get::<_, Option<bool>>(7), expected.bool_value);
            assert_eq!(row.get::<_, Option<String>>(8), expected.enum_type);
            assert_eq!(row.get::<_, Option<String>>(9), expected.enum_member);
            assert_eq!(row.get::<_, Option<Vec<u8>>>(10), expected.stable_key);
        }
        tx.rollback().unwrap();
        let count: i64 = client
            .query_one(
                "SELECT count(*) FROM babylon_state.organization_state_v1",
                &[],
            )
            .unwrap()
            .get(0);
        assert_eq!(
            count, 0,
            "COPY rows retain the enclosing transaction's rollback boundary"
        );
        database.cleanup();
    }
    fn validated_base_config() -> Config {
        assert_eq!(std::env::var(ACK_ENV).as_deref(), Ok(ACK));
        let canary = std::env::var(CANARY_ENV).expect("runner supplies the disposable canary");
        assert_eq!(canary.len(), 32);
        let dsn = std::env::var(DSN_ENV).expect("runner supplies the disposable DSN");
        let config = Config::from_str(&dsn).expect("runner DSN parses");
        validate_connection_target(&config).expect("loopback target");
        assert_eq!(config.get_user(), Some("test"));
        assert_eq!(config.get_dbname(), Some("postgres"));
        let actual: Option<String> = config
            .connect(NoTls)
            .expect("canary connection")
            .query_one(
                "SELECT pg_catalog.current_setting('babylon.disposable_runtime', true)",
                &[],
            )
            .expect("canary query")
            .try_get(0)
            .expect("canary decode");
        assert_eq!(actual.as_deref(), Some(canary.as_str()));
        config
    }
    fn validated_template_name() -> String {
        let template = std::env::var(TEMPLATE_DB_ENV)
            .expect("runner supplies the validated Rust-active template database");
        let suffix = template
            .strip_prefix("per281_runtime_template_")
            .expect("runtime template uses the task-owned prefix");
        assert_eq!(suffix.len(), 12);
        assert!(suffix
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
        assert!(template
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'));
        template
    }
    struct TestDatabase {
        name: String,
        admin: Config,
        active: bool,
    }

    impl TestDatabase {
        fn config(&self, base: &Config) -> Config {
            let mut config = base.clone();
            config.dbname(&self.name);
            config
        }

        fn create_from_template(base: &Config, template: &str, label: &str) -> Self {
            assert!(label.bytes().all(|byte| byte.is_ascii_lowercase()));
            assert!(template
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'));
            let name = format!("per281_runtime_{label}_{}", std::process::id());
            let mut admin = base.clone();
            admin.dbname("postgres");
            let sql = format!("CREATE DATABASE \"{name}\" OWNER test TEMPLATE \"{template}\"");
            admin
                .connect(NoTls)
                .expect("admin connection")
                .batch_execute(&sql)
                .expect("runtime clone creation");
            let database = Self {
                name,
                admin,
                active: true,
            };
            verify_runtime_schema(&database.config(base)).expect("current schema authority");
            database
        }

        fn cleanup(mut self) {
            self.try_drop_database()
                .expect("runtime test database cleanup");
            self.active = false;
        }

        fn try_drop_database(&self) -> Result<(), ()> {
            let sql = format!("DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)", self.name);
            self.admin
                .connect(NoTls)
                .map_err(|_| ())?
                .batch_execute(&sql)
                .map_err(|_| ())
        }
    }

    impl Drop for TestDatabase {
        fn drop(&mut self) {
            if !self.active {
                return;
            }
            if std::thread::panicking() {
                let _cleanup = self.try_drop_database();
                return;
            }
            self.try_drop_database()
                .expect("runtime test database cleanup");
            self.active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn csv_copy_keeps_null_empty_quotes_line_breaks_and_bytea_distinct() {
        let mut actual = Vec::new();
        super::write_csv_row(
            &mut actual,
            [None, Some(""), Some("a,\"b\"\n\\N"), Some("\\x0001ff")].into_iter(),
        )
        .unwrap();
        assert_eq!(actual, b",\"\",\"a,\"\"b\"\"\n\\N\",\"\\x0001ff\"\n");
        assert_eq!(super::bytea_copy_text(&[0, 1, 255]), "\\x0001ff");
    }
}
