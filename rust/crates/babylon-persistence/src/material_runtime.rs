//! Explicit V3 durable material campaign, marker-last and checkpoint-complete.

mod component_identity;
pub(crate) use component_identity::MaterialComponentIdentity;

use crate::stored_tick::{StoredEvent, StoredTickReadSource, StoredTickRelation};

use crate::CommittedTickReceipt;
use crate::{
    checkpoint::{CommittedFullCheckpoint, CommittedResolveTick},
    committed_tick_envelope::CommittedTickRowFamilies,
    foundation::{CampaignFoundation, FoundationContentBundle},
    identity::CampaignId,
    material_envelope::CommittedMaterialTickEnvelope,
    runtime::{
        insert_campaign_foundation_rows, insert_typed_tick_pre_marker_rows, prepare_committed_tick,
        reconstruct_graph_foundation_session, verify_runtime_schema_client,
        RustPersistenceRuntimeError,
    },
    semantic_batches::{
        compose_graph_rows_with_encoder, compose_material_state_rows, StableGraphRowRef,
    },
    stored_tick,
};
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::stable_state::StableGraphState;
use babylon_kernel::content_digest::sha256_of;
use babylon_material_circuit::MaterialCircuitState;
use babylon_practice_contract::OrderedPracticeActionBatch;
use babylon_tick::choice_receipt::ChoiceReceipt;
use babylon_tick::{
    material_replay::{
        IdentifiedMaterialTick, MaterialCommitError, MaterialReplayError, MaterialReplaySession,
    },
    material_world::{MaterialWorldError, MaterialWorldRegister},
    replay_session::{ReplayCommitDisposition, ReplayTickSession},
};
use postgres::{Config, GenericClient, NoTls};
use std::time::{Duration, Instant};

const FOUNDATION_DOMAIN: &[u8] = b"babylon.material-campaign-foundation.v2\0";

const WRITER_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const WRITER_TCP_USER_TIMEOUT: Duration = Duration::from_secs(30);
// Tick computation is detached before these transactions begin. Individual
// writer statements get a larger budget than the observer's read queries.
const WRITER_STARTUP_OPTIONS: &str = "-c search_path=pg_catalog -c quote_all_identifiers=off \
    -c statement_timeout=120000ms -c lock_timeout=5000ms \
    -c idle_in_transaction_session_timeout=120000ms";

pub(crate) fn bounded_material_writer_config(
    config: &Config,
) -> Result<Config, MaterialRuntimeError> {
    // Validate the caller before introducing trusted startup settings. Never
    // accept caller options merely because they resemble our timeout values.
    crate::postgres_catalog::validate_connection_target(config).map_err(|error| {
        RustPersistenceRuntimeError::CurrentSchema(crate::CurrentSchemaError::ConnectionTarget(
            error,
        ))
    })?;
    let mut bounded = config.clone();
    bounded
        .connect_timeout(WRITER_CONNECT_TIMEOUT)
        .tcp_user_timeout(WRITER_TCP_USER_TIMEOUT)
        .options(WRITER_STARTUP_OPTIONS);
    Ok(bounded)
}

/// Pinned authored identity; quantities remain in the complete material register.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialFoundationSpec {
    pub preset_id: String,
    pub horizon_ticks: u64,
    pub content_digest: [u8; 32],
}
/// Fresh tick-zero owners and their exact combined foundation.
pub struct MaterialRuntimeFoundation {
    graph: ReplayTickSession<HypergraphStore>,
    graph_foundation: CampaignFoundation,
    register: MaterialWorldRegister,
    spec: MaterialFoundationSpec,
    bytes: Vec<u8>,
    digest: [u8; 32],
    labor: babylon_tick::material_staffing::StaffingComposition,
}
/// Precise successor refusal classes. No fallback to a graph-only campaign.
#[derive(Debug)]
pub enum MaterialRuntimeError {
    Graph(RustPersistenceRuntimeError),
    Replay(MaterialReplayError),
    Register(MaterialWorldError),
    Database(postgres::Error),
    DatabaseLockRefused(postgres::Error),
    DatabaseStatementCanceled(postgres::Error),
    SchemaDrift,
    FoundationMismatch,
    #[cfg(test)]
    InjectedCommitLoss,
    LegacyCampaign,
    MissingCampaign,
    AlreadyExists,
    TailConflict,
    InvalidCheckpoint,
    Bounds,
    MichiganEconomy(crate::michigan_economy::MichiganEconomyError),
    MichiganMaterial(crate::michigan_material::MichiganMaterialError),
}
impl std::fmt::Display for MaterialRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "material runtime refused: {self:?}")
    }
}
impl std::error::Error for MaterialRuntimeError {}
impl From<RustPersistenceRuntimeError> for MaterialRuntimeError {
    fn from(error: RustPersistenceRuntimeError) -> Self {
        Self::Graph(error)
    }
}
impl From<MaterialReplayError> for MaterialRuntimeError {
    fn from(error: MaterialReplayError) -> Self {
        Self::Replay(error)
    }
}
impl From<MaterialWorldError> for MaterialRuntimeError {
    fn from(error: MaterialWorldError) -> Self {
        Self::Register(error)
    }
}
impl From<crate::semantic_batches::SemanticBatchError> for MaterialRuntimeError {
    fn from(error: crate::semantic_batches::SemanticBatchError) -> Self {
        Self::Graph(error.into())
    }
}
impl From<postgres::Error> for MaterialRuntimeError {
    fn from(error: postgres::Error) -> Self {
        if error.code() == Some(&postgres::error::SqlState::LOCK_NOT_AVAILABLE) {
            Self::DatabaseLockRefused(error)
        } else if error.code() == Some(&postgres::error::SqlState::QUERY_CANCELED) {
            Self::DatabaseStatementCanceled(error)
        } else {
            Self::Database(error)
        }
    }
}

fn validate_foundation_spec(spec: &MaterialFoundationSpec) -> Result<(), MaterialRuntimeError> {
    if spec.preset_id.is_empty()
        || spec.preset_id.len() > 128
        || !spec
            .preset_id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        || spec.horizon_ticks == 0
        || spec.horizon_ticks > i64::MAX as u64
    {
        return Err(MaterialRuntimeError::Bounds);
    }
    Ok(())
}

impl MaterialRuntimeFoundation {
    /// Capture a foundation whose content explicitly uses the V2 source encoding.
    /// # Errors
    /// Refuses invalid graph, material register, spec or aggregate bounds.
    pub fn capture(
        graph: ReplayTickSession<HypergraphStore>,
        bundle: FoundationContentBundle,
        state: MaterialCircuitState,
        spec: MaterialFoundationSpec,
    ) -> Result<Self, MaterialRuntimeError> {
        validate_foundation_spec(&spec)?;
        let graph_foundation = CampaignFoundation::capture(&graph, bundle)?;
        let register = MaterialWorldRegister::try_new(0, state)?;
        Self::from_parts(graph, graph_foundation, register, spec)
    }

    // Both initial capture and stored reconstruction use this exact encoder.
    // Callers first prove that graph_foundation describes the prepared graph.
    fn from_parts(
        graph: ReplayTickSession<HypergraphStore>,
        graph_foundation: CampaignFoundation,
        register: MaterialWorldRegister,
        spec: MaterialFoundationSpec,
    ) -> Result<Self, MaterialRuntimeError> {
        validate_foundation_spec(&spec)?;
        if graph.completed_tick() != 0 || register.completed_tick() != 0 {
            return Err(MaterialRuntimeError::FoundationMismatch);
        }
        let labor = crate::sector_bundle::foundation::validate_stored_material_authority(
            &graph_foundation,
            &register,
            &spec,
        )
        .map_err(|_| MaterialRuntimeError::FoundationMismatch)?;
        let length = FOUNDATION_DOMAIN
            .len()
            .checked_add(4 + 8 + 32 + 3 * 8)
            .and_then(|n| n.checked_add(spec.preset_id.len()))
            .and_then(|n| n.checked_add(graph_foundation.canonical_bytes().len()))
            .and_then(|n| n.checked_add(register.canonical_bytes().len()))
            .ok_or(MaterialRuntimeError::Bounds)?;
        if length > 67_108_864 {
            return Err(MaterialRuntimeError::Bounds);
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| MaterialRuntimeError::Bounds)?;
        bytes.extend_from_slice(FOUNDATION_DOMAIN);
        bytes.extend_from_slice(&2_u32.to_be_bytes());
        bytes.extend_from_slice(&spec.horizon_ticks.to_be_bytes());
        bytes.extend_from_slice(&spec.content_digest);
        for part in [
            spec.preset_id.as_bytes(),
            graph_foundation.canonical_bytes(),
            register.canonical_bytes(),
        ] {
            bytes.extend_from_slice(
                &u64::try_from(part.len())
                    .map_err(|_| MaterialRuntimeError::Bounds)?
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(part);
        }
        if bytes.len() != length {
            return Err(MaterialRuntimeError::Bounds);
        }
        let digest = sha256_of(&bytes);
        Ok(Self {
            graph,
            graph_foundation,
            register,
            spec,
            bytes,
            digest,
            labor,
        })
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    #[must_use]
    pub const fn initial_register(&self) -> &MaterialWorldRegister {
        &self.register
    }
    #[must_use]
    pub const fn spec(&self) -> &MaterialFoundationSpec {
        &self.spec
    }
    #[must_use]
    pub const fn graph_foundation(&self) -> &CampaignFoundation {
        &self.graph_foundation
    }
    /// Exact labor authority decoded from the admitted stored definitions.
    #[must_use]
    pub const fn labor(&self) -> &babylon_tick::material_staffing::StaffingComposition {
        &self.labor
    }
    /// Consume the exact foundation into its admitted graph and labor authority.
    /// # Errors
    /// Refuses invalid initial clocks or material replay bounds.
    pub fn into_session(
        self,
    ) -> Result<MaterialReplaySession<HypergraphStore>, MaterialRuntimeError> {
        Ok(MaterialReplaySession::new(
            self.graph,
            self.register,
            self.digest,
            self.spec.horizon_ticks,
            self.labor,
        )?)
    }
}

/// Canonical writer for explicitly founded circuit campaigns.
pub struct DurableMaterialRuntime {
    config: Config,
    campaign: CampaignId,
    session: MaterialReplaySession<HypergraphStore>,
    tail: Option<IdentifiedMaterialTick>,
    last_receipt: Option<crate::CommittedTickReceipt>,
    last_choice_receipts: Vec<babylon_tick::choice_receipt::ChoiceReceipt>,
}
impl DurableMaterialRuntime {
    /// Install both foundation components in one transaction, without an implicit tick.
    /// # Errors
    /// Refuses absent authority, existing graph-only campaigns, mismatched foundation or DB failure.
    pub fn create(
        config: &Config,
        campaign: CampaignId,
        foundation: MaterialRuntimeFoundation,
    ) -> Result<Self, MaterialRuntimeError> {
        Self::create_with_admission(config, campaign, foundation, false)
    }
    /// Lifecycle New requires absence under the same founding lock/transaction.
    pub(crate) fn create_new(
        config: &Config,
        campaign: CampaignId,
        foundation: MaterialRuntimeFoundation,
    ) -> Result<Self, MaterialRuntimeError> {
        Self::create_with_admission(config, campaign, foundation, true)
    }
    fn create_with_admission(
        config: &Config,
        campaign: CampaignId,
        foundation: MaterialRuntimeFoundation,
        require_absent: bool,
    ) -> Result<Self, MaterialRuntimeError> {
        let bounded = bounded_material_writer_config(config)?;
        let mut client = bounded.connect(NoTls)?;
        verify_runtime_schema_client(&mut client)?;
        let mut tx = client.transaction()?;
        tx.batch_execute(
            "SET LOCAL search_path TO pg_catalog; SET LOCAL synchronous_commit TO on",
        )?;
        tx.query_one(
            "SELECT pg_catalog.pg_advisory_xact_lock($1)",
            &[&crate::SCHEMA_ADVISORY_LOCK_KEY],
        )?;
        let existed=tx.query_opt("SELECT campaign_id FROM babylon_state.campaign WHERE campaign_id=$1::uuid FOR UPDATE",&[campaign.as_uuid()])?.is_some();
        if existed && require_absent {
            return Err(MaterialRuntimeError::AlreadyExists);
        }
        if existed {
            let stored = hydrate_material_foundation(&mut tx, campaign, foundation.digest())?;
            if stored.canonical_bytes() != foundation.canonical_bytes() {
                return Err(MaterialRuntimeError::FoundationMismatch);
            }
            if read_tail_tick(&mut tx, campaign)? != 0 {
                return Err(MaterialRuntimeError::TailConflict);
            }
        } else {
            insert_campaign_foundation_rows(&mut tx, campaign, &foundation.graph_foundation)?;
            let horizon = i64::try_from(foundation.spec.horizon_ticks)
                .map_err(|_| MaterialRuntimeError::Bounds)?;
            tx.execute("INSERT INTO babylon_state.material_campaign_foundation_v2 (campaign_id,preset_id,horizon_ticks,content_sha256,initial_register_bytes,foundation_bytes,foundation_sha256) VALUES ($1::uuid,$2,$3,$4,$5,$6,$7)",&[campaign.as_uuid(),&foundation.spec.preset_id,&horizon,&&foundation.spec.content_digest[..],&foundation.register.canonical_bytes(),&foundation.canonical_bytes(),&&foundation.digest[..]])?;
        }
        // Staffed rule ownership is fallible: refuse before any founding rows
        // become durable, so dropping this transaction also removes enrollment.
        let session = foundation.into_session()?;
        tx.commit()?;
        Ok(Self {
            config: bounded,
            campaign,
            session,
            tail: None,
            last_receipt: None,
            last_choice_receipts: Vec::new(),
        })
    }
    /// Reconstruct the stored foundation and hydrate the latest complete V3 checkpoint.
    /// The expected digest is supplied by the caller's content-admission policy;
    /// reopening never substitutes a newly constructed scenario or material state.
    /// # Errors
    /// Refuses old campaigns, gaps, altered component rows, identity mismatch or missing checkpoints.
    pub fn open(
        config: &Config,
        campaign: CampaignId,
        expected_foundation_digest: [u8; 32],
    ) -> Result<Self, MaterialRuntimeError> {
        let bounded = bounded_material_writer_config(config)?;
        let mut client = bounded.connect(NoTls)?;
        verify_runtime_schema_client(&mut client)?;
        let mut tx = client
            .build_transaction()
            .isolation_level(postgres::IsolationLevel::RepeatableRead)
            .read_only(true)
            .start()?;
        let foundation =
            hydrate_material_foundation(&mut tx, campaign, expected_foundation_digest)?;
        let tick = read_tail_tick(&mut tx, campaign)?;
        let mut session = foundation.into_session()?;
        let tail = if tick == 0 {
            None
        } else {
            let stored = read_stored_material_tick(&mut tx, campaign, tick, &session)?;
            session.restore_full_checkpoint(
                &stored.graph,
                &stored.material,
                &stored.sections[1],
                stored.register.canonical_bytes(),
            )?;
            if session.current_world_hash()? != stored.identity.result_world_hash() {
                return Err(MaterialRuntimeError::InvalidCheckpoint);
            }
            Some(stored.identity)
        };
        tx.commit()?;
        Ok(Self {
            config: bounded,
            campaign,
            session,
            tail,
            last_receipt: None,
            last_choice_receipts: Vec::new(),
        })
    }
    #[must_use]
    pub const fn session(&self) -> &MaterialReplaySession<HypergraphStore> {
        &self.session
    }
    #[must_use]
    pub const fn campaign_id(&self) -> CampaignId {
        self.campaign
    }
    #[must_use]
    pub const fn tail(&self) -> Option<&IdentifiedMaterialTick> {
        self.tail.as_ref()
    }
    /// Process-local diagnostics for the last acknowledged tick.
    #[must_use]
    pub const fn diagnostic_receipt(&self) -> Option<&crate::CommittedTickReceipt> {
        self.last_receipt.as_ref()
    }

    /// Recompose the exact graph state bound to the current acknowledged receipt.
    ///
    /// This observation stays separate from the bounded receipt so persistence
    /// acknowledgements remain identity- and value-free. The caller must present
    /// the current runtime tail, and recomposition must reproduce its sealed
    /// post-tick graph digest before any state is returned.
    ///
    /// # Errors
    /// Refuses a receipt for any other tick, graph-state recomposition failure,
    /// or a recomposed digest that differs from the acknowledged graph digest.
    pub fn observe_committed_graph_state(
        &self,
        receipt: &CommittedTickReceipt,
    ) -> Result<StableGraphState, RustPersistenceRuntimeError> {
        if self.tail.as_ref().map(IdentifiedMaterialTick::resolve_tick)
            != Some(receipt.resolve_tick().get())
        {
            return Err(
                RustPersistenceRuntimeError::ObservationNotCurrentCommittedTail {
                    receipt_tick: receipt.resolve_tick().get(),
                    current_tail: self.tail.as_ref().map(IdentifiedMaterialTick::resolve_tick),
                },
            );
        }
        let observed = self.observe_current_stable_graph_state()?;
        if observed.digest().as_bytes() != &receipt.result_stable_graph_digest() {
            return Err(RustPersistenceRuntimeError::ObservationGraphDigestMismatch);
        }
        Ok(observed)
    }

    /// Borrow detailed choice evidence for the just-acknowledged process-local tick.
    ///
    /// This is an optional post-commit operational observation seam. It never
    /// participates in adjudication, hashing, persistence, retry, or restart.
    ///
    /// # Errors
    /// Refuses a receipt other than the current tail or detail not retained by
    /// this process (for example immediately after reopening a campaign).
    pub fn observe_committed_choice_receipts(
        &self,
        receipt: &CommittedTickReceipt,
    ) -> Result<&[ChoiceReceipt], RustPersistenceRuntimeError> {
        if self.tail.as_ref().map(IdentifiedMaterialTick::resolve_tick)
            != Some(receipt.resolve_tick().get())
        {
            return Err(
                RustPersistenceRuntimeError::ObservationNotCurrentCommittedTail {
                    receipt_tick: receipt.resolve_tick().get(),
                    current_tail: self.tail.as_ref().map(IdentifiedMaterialTick::resolve_tick),
                },
            );
        }
        if self.last_receipt.as_ref() != Some(receipt)
            || self.last_choice_receipts.len() != receipt.choice_receipt_count()
        {
            return Err(RustPersistenceRuntimeError::ObservationChoiceReceiptUnavailable);
        }
        Ok(&self.last_choice_receipts)
    }

    /// Recompose the runtime's current stable graph without mutating it.
    ///
    /// This unbound observation supports capturing a pre-tick state. A caller
    /// must bind its digest to the next acknowledged receipt before exposing
    /// any derived values.
    ///
    /// # Errors
    /// Refuses any stable-identity, topology, numeric, bound, or allocation
    /// failure while recomposing the current graph.
    pub fn observe_current_stable_graph_state(
        &self,
    ) -> Result<StableGraphState, RustPersistenceRuntimeError> {
        self.session
            .graph_session()
            .stable_graph_state()
            .map_err(|_| RustPersistenceRuntimeError::ReplayTick)
    }

    /// Prepare both owners, durably commit all eight families, then publish both.
    /// # Errors
    /// Refuses stale state, adjudication, row bounds or database failures without live publication.
    pub fn advance_and_commit(
        &mut self,
        sink: &mut CollectingSink,
        actions: &OrderedPracticeActionBatch,
    ) -> Result<IdentifiedMaterialTick, MaterialRuntimeError> {
        let started = Instant::now();
        let candidate = self.session.prepare_advance(actions)?;
        let adjudicated = Instant::now();
        let identity = *candidate.identity();
        let mut diagnostic = crate::CommittedTickReceipt::from_material_candidate(&candidate)?;
        let choices = candidate.graph_report().report().choice_receipts.clone();
        let tick = CommittedResolveTick::try_from(identity.resolve_tick())
            .map_err(|_| MaterialRuntimeError::Bounds)?;
        let checkpoint =
            CommittedFullCheckpoint::capture(self.campaign, tick, candidate.graph_report())?;
        let families = prepare_committed_tick(candidate.graph_report())?
            .into_material_families(identity.tick_content_hash())?;
        let envelope = CommittedMaterialTickEnvelope::compose(
            self.campaign,
            &identity,
            families,
            candidate.material().register().canonical_bytes(),
            candidate.material().receipt_bytes(),
        )?;
        let prepared = Instant::now();
        let timing = [started, adjudicated, prepared];
        let mut client = self.config.connect(NoTls)?;
        let mut tx = client.transaction()?;
        tx.batch_execute(
            "SET LOCAL search_path TO pg_catalog; SET LOCAL synchronous_commit TO on",
        )?;
        let locked=tx.query_opt("SELECT campaign_id FROM babylon_state.material_campaign_foundation_v2 WHERE campaign_id=$1::uuid FOR UPDATE",&[self.campaign.as_uuid()])?;
        if locked.is_none() {
            return Err(MaterialRuntimeError::MissingCampaign);
        }
        verify_runtime_schema_client(&mut tx)?;
        let durable = read_tail_tick(&mut tx, self.campaign)?;
        if durable == identity.resolve_tick() {
            let stored = read_stored_material_tick(&mut tx, self.campaign, durable, &self.session)?;
            if stored.envelope.canonical_bytes() != envelope.canonical_bytes() {
                return Err(MaterialRuntimeError::TailConflict);
            }
            tx.rollback()?;
            let (ack, _) = self
                .session
                .commit_prepared_and_publish(sink, candidate, |_| {
                    Ok::<_, MaterialRuntimeError>(
                        ReplayCommitDisposition::ReconciledAfterAmbiguousCommit,
                    )
                })
                .map_err(commit_error)?;
            self.tail = Some(ack);
            diagnostic.acknowledge(ReplayCommitDisposition::ReconciledAfterAmbiguousCommit);
            self.last_receipt = Some(diagnostic);
            self.last_choice_receipts = choices;
            record_advance_timing(ack.resolve_tick(), timing);
            return Ok(ack);
        }
        if durable != self.session.completed_tick() {
            return Err(MaterialRuntimeError::TailConflict);
        }
        let tick_sql =
            i64::try_from(identity.resolve_tick()).map_err(|_| MaterialRuntimeError::Bounds)?;
        insert_typed_tick_pre_marker_rows(
            &mut tx,
            self.campaign,
            tick_sql,
            candidate.graph_report(),
            &checkpoint,
            identity.tick_content_hash(),
        )?;
        tx.execute("INSERT INTO babylon_state.material_tick_v3 (campaign_id,resolve_tick,identity_bytes,register_bytes,receipt_bytes) VALUES ($1::uuid,$2,$3,$4,$5)",&[self.campaign.as_uuid(),&tick_sql,&identity.canonical_bytes(),&candidate.material().register().canonical_bytes(),&candidate.material().receipt_bytes()])?;
        crate::metadata::advance_campaign_catalog_tick(
            &mut tx,
            self.campaign,
            tick_sql - 1,
            tick_sql,
        )?;
        let scope = candidate
            .graph_report()
            .result_stable_graph()
            .scenario_scope()
            .to_owned();
        let components = MaterialComponentIdentity::from_session(self.session.graph_session());
        let (ack, disposition) = self
            .session
            .commit_prepared_and_publish(sink, candidate, |_| {
                commit_material_envelope(
                    tx,
                    &self.config,
                    self.campaign,
                    &identity,
                    &envelope,
                    &scope,
                    &components,
                )
            })
            .map_err(commit_error)?;
        self.tail = Some(ack);
        diagnostic.acknowledge(disposition);
        self.last_receipt = Some(diagnostic);
        self.last_choice_receipts = choices;
        record_advance_timing(ack.resolve_tick(), timing);
        Ok(ack)
    }
}

// Commit the marker last, then reconcile an ambiguous acknowledgement against
// the complete authenticated candidate before permitting live publication.
fn commit_material_envelope(
    mut tx: postgres::Transaction<'_>,
    config: &Config,
    campaign: CampaignId,
    identity: &IdentifiedMaterialTick,
    envelope: &CommittedMaterialTickEnvelope,
    scope: &str,
    components: &MaterialComponentIdentity,
) -> Result<ReplayCommitDisposition, MaterialRuntimeError> {
    let tick = i64::try_from(identity.resolve_tick()).map_err(|_| MaterialRuntimeError::Bounds)?;
    #[cfg(test)]
    inject_marker_trigger_failure(&mut tx)?;
    tx.execute(
        "INSERT INTO babylon_state.tick_commit \
         (campaign_id,resolve_tick,envelope_layout_version,tick_content_hash,envelope_digest) \
         VALUES ($1::uuid,$2,3,$3,$4)",
        &[
            campaign.as_uuid(),
            &tick,
            &&identity.tick_content_hash().as_bytes()[..],
            &&envelope.digest()[..],
        ],
    )?;
    match commit_material_transaction(tx) {
        Ok(()) => Ok(ReplayCommitDisposition::Committed),
        Err(error) => {
            let mut retry_client = config.connect(NoTls)?;
            let mut retry = retry_client.transaction()?;
            verify_runtime_schema_client(&mut retry)?;
            if !marker_matches(
                &mut retry,
                StoredTickReadSource::Runtime,
                campaign,
                identity,
                envelope,
            )? {
                return Err(error);
            }
            let stored = read_authenticated_material_tick(
                &mut retry,
                StoredTickReadSource::Runtime,
                campaign,
                identity.resolve_tick(),
                scope,
                identity.foundation_digest(),
                components,
            )?;
            if stored.identity != *identity
                || stored.envelope.canonical_bytes() != envelope.canonical_bytes()
            {
                return Err(MaterialRuntimeError::TailConflict);
            }
            Ok(ReplayCommitDisposition::ReconciledAfterAmbiguousCommit)
        }
    }
}

// Inject only after exact schema admission and all candidate rows are prepared.
// The replacement and its marker-trigger failure share the candidate transaction,
// so PostgreSQL rollback restores the original function without external repair.
#[cfg(test)]
fn inject_marker_trigger_failure(
    tx: &mut postgres::Transaction<'_>,
) -> Result<(), MaterialRuntimeError> {
    let armed = COMMIT_FAULT.with(|slot| {
        if slot.get() == Some(CommitFault::MarkerTrigger) {
            slot.set(None);
            true
        } else {
            false
        }
    });
    if armed {
        tx.batch_execute(
            "CREATE OR REPLACE FUNCTION babylon_meta.archive_wakeup_v1() RETURNS trigger \
             LANGUAGE plpgsql SET search_path = pg_catalog AS $fault$ BEGIN \
             RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='test-owned Archive wakeup failure'; END $fault$",
        )?;
    }
    Ok(())
}

// Keep commit acknowledgement loss inside the same reconciliation boundary as
// transport failure: only the persisted complete candidate can authorize publication.
fn commit_material_transaction(tx: postgres::Transaction<'_>) -> Result<(), MaterialRuntimeError> {
    #[cfg(test)]
    let fault = COMMIT_FAULT.with(|slot| slot.replace(None));
    #[cfg(test)]
    if fault == Some(CommitFault::BeforeCommit) {
        tx.rollback()?;
        return Err(MaterialRuntimeError::InjectedCommitLoss);
    }
    tx.commit()?;
    #[cfg(test)]
    if fault == Some(CommitFault::AfterCommit) {
        return Err(MaterialRuntimeError::InjectedCommitLoss);
    }
    Ok(())
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommitFault {
    BeforeCommit,
    AfterCommit,
    MarkerTrigger,
}
#[cfg(test)]
thread_local! {
    pub(crate) static COMMIT_FAULT: std::cell::Cell<Option<CommitFault>> = const { std::cell::Cell::new(None) };
}

// Operator diagnostics only: clocks never enter state, hashes, receipts or the
// protocol. Emit one bounded record after successful durable publication.
fn record_advance_timing(period: u64, [started, adjudicated, prepared]: [Instant; 3]) {
    if std::env::var("BABYLON_TIMINGS").as_deref() == Ok("1") {
        eprintln!(
            "babylon-timing period={period} simulation_us={} preparation_us={} durable_write_publish_us={} total_us={}",
            adjudicated.duration_since(started).as_micros(),
            prepared.duration_since(adjudicated).as_micros(),
            prepared.elapsed().as_micros(),
            started.elapsed().as_micros(),
        );
    }
}

fn commit_error(error: MaterialCommitError<MaterialRuntimeError>) -> MaterialRuntimeError {
    match error {
        MaterialCommitError::Preflight(error) => error.into(),
        MaterialCommitError::Commit(error) => error,
    }
}

struct StoredMaterialFoundation {
    spec: MaterialFoundationSpec,
    initial_register_bytes: Vec<u8>,
    foundation_bytes: Vec<u8>,
    foundation_digest: [u8; 32],
    graph_foundation_digest: [u8; 32],
}

impl StoredMaterialFoundation {
    fn from_row(row: &postgres::Row) -> Result<Self, MaterialRuntimeError> {
        let digest = |column: usize| -> Result<[u8; 32], MaterialRuntimeError> {
            row.try_get::<_, Vec<u8>>(column)?
                .try_into()
                .map_err(|_| MaterialRuntimeError::FoundationMismatch)
        };
        Ok(Self {
            spec: MaterialFoundationSpec {
                preset_id: row.try_get(0)?,
                horizon_ticks: u64::try_from(row.try_get::<_, i64>(1)?)
                    .map_err(|_| MaterialRuntimeError::FoundationMismatch)?,
                content_digest: digest(2)?,
            },
            initial_register_bytes: row.try_get(3)?,
            foundation_bytes: row.try_get(4)?,
            foundation_digest: digest(5)?,
            graph_foundation_digest: digest(6)?,
        })
    }
}

fn hydrate_material_foundation(
    client: &mut impl GenericClient,
    campaign: CampaignId,
    expected_foundation_digest: [u8; 32],
) -> Result<MaterialRuntimeFoundation, MaterialRuntimeError> {
    let row=client.query_opt("SELECT f.preset_id,f.horizon_ticks,f.content_sha256,f.initial_register_bytes,f.foundation_bytes,f.foundation_sha256,g.foundation_sha256 FROM babylon_state.material_campaign_foundation_v2 f JOIN babylon_state.campaign_foundation g USING(campaign_id) WHERE campaign_id=$1::uuid",&[campaign.as_uuid()])?;
    let Some(row) = row else {
        let exists = client
            .query_opt(
                "SELECT campaign_id FROM babylon_state.campaign WHERE campaign_id=$1::uuid",
                &[campaign.as_uuid()],
            )?
            .is_some();
        return Err(if exists {
            MaterialRuntimeError::LegacyCampaign
        } else {
            MaterialRuntimeError::MissingCampaign
        });
    };
    let stored = StoredMaterialFoundation::from_row(&row)?;
    if stored.foundation_digest != expected_foundation_digest {
        return Err(MaterialRuntimeError::FoundationMismatch);
    }
    let graph = crate::runtime::hydrate_campaign_foundation_client(client, campaign)?;
    reconstruct_material_foundation(stored, graph, expected_foundation_digest)
}

fn reconstruct_material_foundation(
    stored: StoredMaterialFoundation,
    graph_foundation: CampaignFoundation,
    expected_foundation_digest: [u8; 32],
) -> Result<MaterialRuntimeFoundation, MaterialRuntimeError> {
    if stored.foundation_digest != expected_foundation_digest
        || sha256_of(&stored.foundation_bytes) != expected_foundation_digest
        || sha256_of(graph_foundation.canonical_bytes()) != stored.graph_foundation_digest
    {
        return Err(MaterialRuntimeError::FoundationMismatch);
    }
    let register = MaterialWorldRegister::decode(&stored.initial_register_bytes)?;
    if register.completed_tick() != 0 {
        return Err(MaterialRuntimeError::FoundationMismatch);
    }
    let graph = reconstruct_graph_foundation_session(&graph_foundation)?;
    let reconstructed =
        MaterialRuntimeFoundation::from_parts(graph, graph_foundation, register, stored.spec)?;
    if reconstructed.canonical_bytes() != stored.foundation_bytes
        || reconstructed.digest() != expected_foundation_digest
    {
        return Err(MaterialRuntimeError::FoundationMismatch);
    }
    Ok(reconstructed)
}
fn read_tail_tick(
    client: &mut impl GenericClient,
    campaign: CampaignId,
) -> Result<u64, MaterialRuntimeError> {
    let row=client.query_one("SELECT count(*),coalesce(max(resolve_tick),0),coalesce(bool_and(envelope_layout_version=3),true) FROM babylon_state.tick_commit WHERE campaign_id=$1::uuid",&[campaign.as_uuid()])?;
    let count: i64 = row.try_get(0)?;
    let tick: i64 = row.try_get(1)?;
    let version: bool = row.try_get(2)?;
    if count != tick || !version {
        return Err(MaterialRuntimeError::TailConflict);
    }
    u64::try_from(tick).map_err(|_| MaterialRuntimeError::TailConflict)
}
fn marker_matches(
    client: &mut impl GenericClient,
    source: StoredTickReadSource,
    campaign: CampaignId,
    identity: &IdentifiedMaterialTick,
    envelope: &CommittedMaterialTickEnvelope,
) -> Result<bool, MaterialRuntimeError> {
    let tick = i64::try_from(identity.resolve_tick()).map_err(|_| MaterialRuntimeError::Bounds)?;
    let Some(row)=client.query_opt(&format!("SELECT envelope_layout_version,tick_content_hash,envelope_digest FROM {} WHERE campaign_id=$1::uuid AND resolve_tick=$2", source.relation(StoredTickRelation::TickCommit)),&[campaign.as_uuid(),&tick])? else{return Ok(false);};
    if row.try_get::<_, i16>(0)? != 3
        || row.try_get::<_, Vec<u8>>(1)? != identity.tick_content_hash().as_bytes()
        || row.try_get::<_, Vec<u8>>(2)? != envelope.digest()
    {
        return Err(MaterialRuntimeError::TailConflict);
    }
    Ok(true)
}

/// A fully authenticated committed material tick, never a partial checkpoint read.
pub(crate) struct StoredMaterialTick {
    pub(crate) identity: IdentifiedMaterialTick,
    envelope: CommittedMaterialTickEnvelope,
    pub(crate) graph: babylon_graph::stable_state::StableGraphState,
    material: babylon_tick::material_state::MaterialStateRows,
    sections: Vec<Vec<u8>>,
    pub(crate) register: MaterialWorldRegister,
    pub(crate) events: Vec<StoredEvent>,
}

/// Uses the same complete decoder and envelope proof as durable reconciliation.
/// The caller retains its role confinement and repeatable-read transaction.
pub(crate) fn read_observer_material_tick(
    client: &mut impl GenericClient,
    campaign: CampaignId,
    tick: u64,
    scope: &str,
    foundation_digest: [u8; 32],
    components: &MaterialComponentIdentity,
) -> Result<StoredMaterialTick, MaterialRuntimeError> {
    read_authenticated_material_tick(
        client,
        StoredTickReadSource::FullObserver,
        campaign,
        tick,
        scope,
        foundation_digest,
        components,
    )
}

fn read_stored_material_tick(
    client: &mut impl GenericClient,
    campaign: CampaignId,
    tick: u64,
    session: &MaterialReplaySession<HypergraphStore>,
) -> Result<StoredMaterialTick, MaterialRuntimeError> {
    let scope = session
        .graph_session()
        .stable_graph_state()
        .map_err(MaterialReplayError::Graph)?
        .scenario_scope()
        .to_owned();
    let components = MaterialComponentIdentity::from_session(session.graph_session());
    read_authenticated_material_tick(
        client,
        StoredTickReadSource::Runtime,
        campaign,
        tick,
        &scope,
        session.foundation_digest(),
        &components,
    )
}

fn read_authenticated_material_tick(
    client: &mut impl GenericClient,
    source: StoredTickReadSource,
    campaign: CampaignId,
    tick: u64,
    scope: &str,
    foundation_digest: [u8; 32],
    components: &MaterialComponentIdentity,
) -> Result<StoredMaterialTick, MaterialRuntimeError> {
    let stored = read_stored_material_tick_rows(client, source, campaign, tick, scope)?;
    if stored.identity.foundation_digest() != foundation_digest {
        return Err(MaterialRuntimeError::InvalidCheckpoint);
    }
    validate_component_identity(client, source, campaign, tick, components, &stored.sections)?;
    Ok(stored)
}

fn read_stored_material_tick_rows(
    client: &mut impl GenericClient,
    source: StoredTickReadSource,
    campaign: CampaignId,
    tick: u64,
    scope: &str,
) -> Result<StoredMaterialTick, MaterialRuntimeError> {
    let tick_sql = i64::try_from(tick).map_err(|_| MaterialRuntimeError::Bounds)?;
    let row=client.query_opt(&format!("SELECT identity_bytes,register_bytes,receipt_bytes FROM {} WHERE campaign_id=$1::uuid AND resolve_tick=$2", source.relation(StoredTickRelation::MaterialTick)),&[campaign.as_uuid(),&tick_sql])?.ok_or(MaterialRuntimeError::InvalidCheckpoint)?;
    let identity = IdentifiedMaterialTick::decode(&row.try_get::<_, Vec<u8>>(0)?)?;
    let register: Vec<u8> = row.try_get(1)?;
    let receipts: Vec<u8> = row.try_get(2)?;
    let decoded_register = MaterialWorldRegister::decode(&register)?;
    if identity.resolve_tick() != tick || decoded_register.completed_tick() != tick {
        return Err(MaterialRuntimeError::InvalidCheckpoint);
    }
    let graph = stored_tick::read_graph_state(client, source, campaign, tick_sql, scope)?;
    let material = stored_tick::read_material_rows(client, source, campaign, tick_sql)?;
    let (checkpoint, sections) = stored_tick::read_checkpoint_rows(
        client, source, campaign, tick, tick_sql, &graph, &material,
    )?;
    let (graph_rows, _) =
        compose_graph_rows_with_encoder(graph.rows(), &mut |row: StableGraphRowRef<'_>| {
            row.encode()
        })?;
    let events = stored_tick::read_event_rows(client, source, campaign, tick_sql)?;
    let families = CommittedTickRowFamilies {
        graph: graph_rows,
        state: compose_material_state_rows(&material)?,
        event: events.encoded,
        choice_receipt: stored_tick::read_choice_receipt_rows(client, source, campaign, tick_sql)?,
        checkpoint,
        archive_dirty_receipt: stored_tick::read_archive_receipt(
            client, source, campaign, tick_sql,
        )?,
    };
    let envelope = CommittedMaterialTickEnvelope::compose(
        campaign, &identity, families, &register, &receipts,
    )?;
    if !marker_matches(client, source, campaign, &identity, &envelope)? {
        return Err(MaterialRuntimeError::InvalidCheckpoint);
    }
    Ok(StoredMaterialTick {
        identity,
        envelope,
        graph,
        material,
        sections,
        register: decoded_register,
        events: events.decoded,
    })
}
fn validate_component_identity(
    client: &mut impl GenericClient,
    source: StoredTickReadSource,
    campaign: CampaignId,
    tick: u64,
    components: &MaterialComponentIdentity,
    sections: &[Vec<u8>],
) -> Result<(), MaterialRuntimeError> {
    components.validate_sections(sections)?;
    let tick_sql = i64::try_from(tick).map_err(|_| MaterialRuntimeError::Bounds)?;
    let row = client
        .query_opt(
            &format!(
                "SELECT layout_version,action_batch_digest,exact_action_batch_bytes FROM {} \
            WHERE campaign_id=$1::uuid AND resolve_tick=$2",
                source.relation(StoredTickRelation::TickActionBatch)
            ),
            &[campaign.as_uuid(), &tick_sql],
        )?
        .ok_or(MaterialRuntimeError::InvalidCheckpoint)?;
    components.validate_actions(
        tick,
        row.try_get(0)?,
        &row.try_get::<_, Vec<u8>>(1)?,
        &row.try_get::<_, Vec<u8>>(2)?,
    )
}

#[cfg(test)]
mod reconstruction_tests;

#[cfg(test)]
mod writer_bounds_tests {
    use super::*;

    #[test]
    fn validated_caller_gets_bounded_writer_settings_without_mutating_the_input() {
        let mut caller = Config::new();
        caller
            .host("127.0.0.1")
            .user("writer")
            .dbname("campaigns")
            .connect_timeout(Duration::from_secs(3600))
            .tcp_user_timeout(Duration::from_secs(3600));
        let bounded = bounded_material_writer_config(&caller).unwrap();
        assert_eq!(bounded.get_user(), caller.get_user());
        assert_eq!(bounded.get_dbname(), caller.get_dbname());
        assert_eq!(
            bounded.get_connect_timeout().copied(),
            Some(WRITER_CONNECT_TIMEOUT)
        );
        assert_eq!(
            bounded.get_tcp_user_timeout().copied(),
            Some(WRITER_TCP_USER_TIMEOUT)
        );
        assert_eq!(bounded.get_options(), Some(WRITER_STARTUP_OPTIONS));
        assert_eq!(caller.get_options(), None);
        crate::postgres_catalog::validate_connection_target(&caller).unwrap();
        assert!(!WRITER_STARTUP_OPTIONS.contains("default_transaction_read_only=on"));
    }

    #[test]
    fn caller_startup_options_are_refused_even_when_equal_to_trusted_settings() {
        for options in [
            "-c lock_timeout=0 -c statement_timeout=0",
            WRITER_STARTUP_OPTIONS,
            "",
        ] {
            let mut caller = Config::new();
            caller.host("127.0.0.1").options(options);
            assert!(matches!(
                bounded_material_writer_config(&caller),
                Err(MaterialRuntimeError::Graph(
                    RustPersistenceRuntimeError::CurrentSchema(
                        crate::CurrentSchemaError::ConnectionTarget(_)
                    )
                ))
            ));
            assert_eq!(caller.get_options(), Some(options));
        }
    }

    #[test]
    fn writer_settings_do_not_bypass_the_loopback_target_boundary() {
        let mut caller = Config::new();
        caller.host("192.0.2.1");
        assert!(matches!(
            bounded_material_writer_config(&caller),
            Err(MaterialRuntimeError::Graph(
                RustPersistenceRuntimeError::CurrentSchema(
                    crate::CurrentSchemaError::ConnectionTarget(_)
                )
            ))
        ));
    }

    #[test]
    #[ignore = "requires a bootstrapped local BABYLON_RUNTIME_DSN; reads authority and settings only"]
    fn live_bounded_writer_verifies_authority_and_timeouts_in_read_only_transaction() {
        let raw: Config = std::env::var("BABYLON_RUNTIME_DSN")
            .expect("explicit bootstrapped local runtime target")
            .parse()
            .expect("runtime connection configuration");
        let bounded = bounded_material_writer_config(&raw).unwrap();
        let mut client = bounded.connect(NoTls).expect("bounded writer connection");
        let mut transaction = client
            .build_transaction()
            .read_only(true)
            .start()
            .expect("read-only authority probe");
        verify_runtime_schema_client(&mut transaction)
            .expect("trusted timeout options preserve the active authority check");
        let settings = transaction
            .query_one(
                "SELECT current_setting('statement_timeout')::interval = interval '120 seconds', \
                 current_setting('lock_timeout')::interval = interval '5 seconds', \
                 current_setting('idle_in_transaction_session_timeout')::interval = interval '120 seconds', \
                 current_setting('transaction_read_only') = 'on'",
                &[],
            )
            .expect("read bounded connection settings");
        for index in 0..4 {
            assert!(settings.get::<_, bool>(index), "connection setting {index}");
        }
        transaction.rollback().expect("end read-only probe");
        assert_eq!(raw.get_options(), None);
    }
}

#[cfg(test)]
mod diagnostics_tests;
