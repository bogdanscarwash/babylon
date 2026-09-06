//! Explicit V3 durable material campaign, marker-last and checkpoint-complete.

use crate::{
    checkpoint::{CommittedFullCheckpointV1, CommittedResolveTickV1},
    committed_tick_envelope::CommittedTickRowFamiliesV2,
    foundation::{CampaignFoundationV1, FoundationContentBundleV1, FoundationContentBundleV2},
    identity::CampaignId,
    material_envelope::CommittedMaterialTickEnvelopeV3,
    runtime::{
        insert_campaign_foundation_rows_v1, insert_typed_tick_pre_marker_rows_v2,
        prepare_committed_tick_v2, reconstruct_graph_foundation_session_v1,
        require_active_authority_client, RustPersistenceRuntimeErrorV2,
    },
    semantic_batches::{
        compose_graph_rows_with_encoder_v1, compose_material_state_rows_v1, StableGraphRowRefV1,
    },
    stored_tick,
};
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_kernel::sha256_of;
use babylon_material_circuit::MaterialCircuitStateV2;
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use babylon_tick::{
    material_replay::{
        IdentifiedMaterialTickV3, MaterialCommitErrorV3, MaterialReplayErrorV3,
        MaterialReplaySessionV3,
    },
    material_world::{MaterialWorldErrorV2, MaterialWorldRegisterV2},
    replay_session::{ReplayCommitDispositionV1, ReplayTickSession},
};
use postgres::{Config, GenericClient, NoTls};
use std::time::Duration;

const SCHEMA: &str = include_str!("../migrations/material_runtime_v3.sql");
const FOUNDATION_DOMAIN: &[u8] = b"babylon.material-campaign-foundation.v2\0";

const WRITER_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const WRITER_TCP_USER_TIMEOUT: Duration = Duration::from_secs(30);
// Tick computation is detached before these transactions begin. Individual
// writer statements get a larger budget than the observer's read queries.
const WRITER_STARTUP_OPTIONS: &str = "-c statement_timeout=120000ms -c lock_timeout=5000ms \
    -c idle_in_transaction_session_timeout=120000ms";

pub(crate) fn bounded_material_writer_config_v3(
    config: &Config,
) -> Result<Config, MaterialRuntimeErrorV3> {
    // Validate the caller before introducing trusted startup settings. Never
    // accept caller options merely because they resemble our timeout values.
    crate::validate_legacy_connection_target(config)
        .map_err(|_| RustPersistenceRuntimeErrorV2::ActivationRequired)?;
    let mut bounded = config.clone();
    bounded
        .connect_timeout(WRITER_CONNECT_TIMEOUT)
        .tcp_user_timeout(WRITER_TCP_USER_TIMEOUT)
        .options(WRITER_STARTUP_OPTIONS);
    Ok(bounded)
}

/// Pinned authored identity; quantities remain in the complete material register.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialFoundationSpecV2 {
    pub preset_id: String,
    pub horizon_ticks: u64,
    pub content_digest: [u8; 32],
}
/// Fresh tick-zero owners and their exact combined foundation.
pub struct MaterialRuntimeFoundationV2 {
    graph: ReplayTickSession<HypergraphStore>,
    graph_foundation: CampaignFoundationV1,
    register: MaterialWorldRegisterV2,
    spec: MaterialFoundationSpecV2,
    bytes: Vec<u8>,
    digest: [u8; 32],
}
/// Precise successor refusal classes. No fallback to a graph-only campaign.
#[derive(Debug)]
pub enum MaterialRuntimeErrorV3 {
    Graph(RustPersistenceRuntimeErrorV2),
    Replay(MaterialReplayErrorV3),
    Register(MaterialWorldErrorV2),
    Database(postgres::Error),
    DatabaseLockRefused(postgres::Error),
    DatabaseStatementCanceled(postgres::Error),
    SchemaDrift,
    FoundationMismatch,
    LegacyCampaign,
    MissingCampaign,
    TailConflict,
    InvalidCheckpoint,
    Bounds,
    MichiganEconomy(crate::michigan_economy::MichiganEconomyErrorV1),
    MichiganMaterial(crate::michigan_material::MichiganMaterialErrorV1),
}
impl std::fmt::Display for MaterialRuntimeErrorV3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "material runtime refused: {self:?}")
    }
}
impl std::error::Error for MaterialRuntimeErrorV3 {}
impl From<RustPersistenceRuntimeErrorV2> for MaterialRuntimeErrorV3 {
    fn from(error: RustPersistenceRuntimeErrorV2) -> Self {
        Self::Graph(error)
    }
}
impl From<MaterialReplayErrorV3> for MaterialRuntimeErrorV3 {
    fn from(error: MaterialReplayErrorV3) -> Self {
        Self::Replay(error)
    }
}
impl From<MaterialWorldErrorV2> for MaterialRuntimeErrorV3 {
    fn from(error: MaterialWorldErrorV2) -> Self {
        Self::Register(error)
    }
}
impl From<crate::semantic_batches::SemanticBatchErrorV2> for MaterialRuntimeErrorV3 {
    fn from(error: crate::semantic_batches::SemanticBatchErrorV2) -> Self {
        Self::Graph(error.into())
    }
}
impl From<postgres::Error> for MaterialRuntimeErrorV3 {
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

fn validate_foundation_spec(spec: &MaterialFoundationSpecV2) -> Result<(), MaterialRuntimeErrorV3> {
    if spec.preset_id.is_empty()
        || spec.preset_id.len() > 128
        || !spec
            .preset_id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        || spec.horizon_ticks == 0
        || spec.horizon_ticks > i64::MAX as u64
    {
        return Err(MaterialRuntimeErrorV3::Bounds);
    }
    Ok(())
}

impl MaterialRuntimeFoundationV2 {
    /// Capture a complete new foundation. Substantive source metadata is pinned in `spec`.
    /// # Errors
    /// Refuses any nonzero session, invalid circuit, label, horizon or allocation bound.
    pub fn capture(
        graph: ReplayTickSession<HypergraphStore>,
        bundle: FoundationContentBundleV1,
        state: MaterialCircuitStateV2,
        spec: MaterialFoundationSpecV2,
    ) -> Result<Self, MaterialRuntimeErrorV3> {
        validate_foundation_spec(&spec)?;
        let graph_foundation = CampaignFoundationV1::capture(&graph, bundle)?;
        let register = MaterialWorldRegisterV2::try_new(0, state)?;
        Self::from_parts(graph, graph_foundation, register, spec)
    }

    /// Capture a foundation whose content explicitly uses the V2 source encoding.
    /// # Errors
    /// Refuses invalid graph, material register, spec or aggregate bounds.
    pub fn capture_v2(
        graph: ReplayTickSession<HypergraphStore>,
        bundle: FoundationContentBundleV2,
        state: MaterialCircuitStateV2,
        spec: MaterialFoundationSpecV2,
    ) -> Result<Self, MaterialRuntimeErrorV3> {
        validate_foundation_spec(&spec)?;
        let graph_foundation = CampaignFoundationV1::capture_v2(&graph, bundle)?;
        let register = MaterialWorldRegisterV2::try_new(0, state)?;
        Self::from_parts(graph, graph_foundation, register, spec)
    }

    // Both initial capture and stored reconstruction use this exact encoder.
    // Callers first prove that graph_foundation describes the prepared graph.
    fn from_parts(
        graph: ReplayTickSession<HypergraphStore>,
        graph_foundation: CampaignFoundationV1,
        register: MaterialWorldRegisterV2,
        spec: MaterialFoundationSpecV2,
    ) -> Result<Self, MaterialRuntimeErrorV3> {
        validate_foundation_spec(&spec)?;
        if graph.completed_tick() != 0 || register.completed_tick() != 0 {
            return Err(MaterialRuntimeErrorV3::FoundationMismatch);
        }
        let length = FOUNDATION_DOMAIN
            .len()
            .checked_add(4 + 8 + 32 + 3 * 8)
            .and_then(|n| n.checked_add(spec.preset_id.len()))
            .and_then(|n| n.checked_add(graph_foundation.canonical_bytes().len()))
            .and_then(|n| n.checked_add(register.canonical_bytes().len()))
            .ok_or(MaterialRuntimeErrorV3::Bounds)?;
        if length > 67_108_864 {
            return Err(MaterialRuntimeErrorV3::Bounds);
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| MaterialRuntimeErrorV3::Bounds)?;
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
                    .map_err(|_| MaterialRuntimeErrorV3::Bounds)?
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(part);
        }
        if bytes.len() != length {
            return Err(MaterialRuntimeErrorV3::Bounds);
        }
        let digest = sha256_of(&bytes);
        Ok(Self {
            graph,
            graph_foundation,
            register,
            spec,
            bytes,
            digest,
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
    pub const fn initial_register(&self) -> &MaterialWorldRegisterV2 {
        &self.register
    }
    #[must_use]
    pub const fn spec(&self) -> &MaterialFoundationSpecV2 {
        &self.spec
    }
    #[must_use]
    pub const fn graph_foundation(&self) -> &CampaignFoundationV1 {
        &self.graph_foundation
    }
    fn into_session(
        self,
    ) -> Result<MaterialReplaySessionV3<HypergraphStore>, MaterialRuntimeErrorV3> {
        Ok(MaterialReplaySessionV3::new(
            self.graph,
            self.register,
            self.digest,
            self.spec.horizon_ticks,
        )?)
    }
}

/// Canonical writer for explicitly founded circuit campaigns.
pub struct DurableMaterialRuntimeV3 {
    config: Config,
    campaign: CampaignId,
    session: MaterialReplaySessionV3<HypergraphStore>,
    tail: Option<IdentifiedMaterialTickV3>,
    content_layout: crate::FoundationContentLayout,
}
impl DurableMaterialRuntimeV3 {
    /// Install both foundation components in one transaction, without an implicit tick.
    /// # Errors
    /// Refuses absent authority, existing graph-only campaigns, mismatched foundation or DB failure.
    pub fn create(
        config: &Config,
        campaign: CampaignId,
        foundation: MaterialRuntimeFoundationV2,
    ) -> Result<Self, MaterialRuntimeErrorV3> {
        let bounded = bounded_material_writer_config_v3(config)?;
        install_material_runtime_schema_v3(config)?;
        crate::install_territory_county_map_schema_v1(config)
            .map_err(|_| MaterialRuntimeErrorV3::SchemaDrift)?;
        crate::archive::SemanticArchiveStoreV1::new(config)
            .install_schema()
            .map_err(|_| MaterialRuntimeErrorV3::SchemaDrift)?;
        let mut client = bounded.connect(NoTls)?;
        let mut tx = client.transaction()?;
        tx.batch_execute(
            "SET LOCAL search_path TO pg_catalog; SET LOCAL synchronous_commit TO on",
        )?;
        tx.query_one(
            "SELECT pg_catalog.pg_advisory_xact_lock($1)",
            &[&crate::SCHEMA_ADVISORY_LOCK_KEY],
        )?;
        let existed=tx.query_opt("SELECT campaign_id FROM babylon_state.campaign WHERE campaign_id=$1::uuid FOR UPDATE",&[campaign.as_uuid()])?.is_some();
        if existed {
            let stored = hydrate_material_foundation_v2(&mut tx, campaign, foundation.digest())?;
            if stored.canonical_bytes() != foundation.canonical_bytes() {
                return Err(MaterialRuntimeErrorV3::FoundationMismatch);
            }
            if read_tail_tick(&mut tx, campaign)? != 0 {
                return Err(MaterialRuntimeErrorV3::TailConflict);
            }
        } else {
            insert_campaign_foundation_rows_v1(&mut tx, campaign, &foundation.graph_foundation)?;
            let horizon = i64::try_from(foundation.spec.horizon_ticks)
                .map_err(|_| MaterialRuntimeErrorV3::Bounds)?;
            tx.execute("INSERT INTO babylon_state.material_campaign_foundation_v2 (campaign_id,preset_id,horizon_ticks,content_sha256,initial_register_bytes,foundation_bytes,foundation_sha256) VALUES ($1::uuid,$2,$3,$4,$5,$6,$7)",&[campaign.as_uuid(),&foundation.spec.preset_id,&horizon,&&foundation.spec.content_digest[..],&foundation.register.canonical_bytes(),&foundation.canonical_bytes(),&&foundation.digest[..]])?;
        }
        tx.commit()?;
        let content_layout = foundation.graph_foundation.content_bundle().layout();
        Ok(Self {
            config: bounded,
            campaign,
            session: foundation.into_session()?,
            tail: None,
            content_layout,
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
    ) -> Result<Self, MaterialRuntimeErrorV3> {
        let bounded = bounded_material_writer_config_v3(config)?;
        install_material_runtime_schema_v3(config)?;
        let mut client = bounded.connect(NoTls)?;
        let mut tx = client
            .build_transaction()
            .isolation_level(postgres::IsolationLevel::RepeatableRead)
            .read_only(true)
            .start()?;
        let foundation =
            hydrate_material_foundation_v2(&mut tx, campaign, expected_foundation_digest)?;
        let tick = read_tail_tick(&mut tx, campaign)?;
        let content_layout = foundation.graph_foundation.content_bundle().layout();
        let mut session = foundation.into_session()?;
        let tail = if tick == 0 {
            None
        } else {
            let stored = read_stored_material_tick(&mut tx, campaign, tick, &session)?;
            session.restore_full_checkpoint(
                &stored.graph,
                &stored.material,
                &stored.sections[1],
                &stored.register,
            )?;
            if session.current_world_hash()? != stored.identity.result_world_hash() {
                return Err(MaterialRuntimeErrorV3::InvalidCheckpoint);
            }
            Some(stored.identity)
        };
        tx.commit()?;
        Ok(Self {
            config: bounded,
            campaign,
            session,
            tail,
            content_layout,
        })
    }
    #[must_use]
    pub const fn session(&self) -> &MaterialReplaySessionV3<HypergraphStore> {
        &self.session
    }
    #[must_use]
    pub const fn campaign_id(&self) -> CampaignId {
        self.campaign
    }
    #[must_use]
    pub const fn tail(&self) -> Option<&IdentifiedMaterialTickV3> {
        self.tail.as_ref()
    }
    /// Prepare both owners, durably commit all eight families, then publish both.
    /// # Errors
    /// Refuses stale state, adjudication, row bounds or database failures without live publication.
    pub fn advance_and_commit(
        &mut self,
        sink: &mut CollectingSink,
        actions: &OrderedPracticeActionBatchV1,
    ) -> Result<IdentifiedMaterialTickV3, MaterialRuntimeErrorV3> {
        let candidate = self.session.prepare_advance(actions)?;
        let identity = *candidate.identity();
        let tick = CommittedResolveTickV1::try_from(identity.resolve_tick())
            .map_err(|_| MaterialRuntimeErrorV3::Bounds)?;
        let checkpoint =
            CommittedFullCheckpointV1::capture(self.campaign, tick, candidate.graph_report())?;
        let families = prepare_committed_tick_v2(candidate.graph_report())?
            .into_material_families(identity.tick_content_hash())?;
        let envelope = CommittedMaterialTickEnvelopeV3::compose(
            self.campaign,
            &identity,
            families,
            candidate.material().register().canonical_bytes(),
            candidate.material().receipt_bytes(),
        )?;
        let mut client = self.config.connect(NoTls)?;
        let mut tx = client.transaction()?;
        tx.batch_execute(
            "SET LOCAL search_path TO pg_catalog; SET LOCAL synchronous_commit TO on",
        )?;
        let locked=tx.query_opt("SELECT campaign_id FROM babylon_state.material_campaign_foundation_v2 WHERE campaign_id=$1::uuid FOR UPDATE",&[self.campaign.as_uuid()])?;
        if locked.is_none() {
            return Err(MaterialRuntimeErrorV3::MissingCampaign);
        }
        crate::foundation_content_schema::lock_foundation_content_layout_v2(
            &mut tx,
            self.campaign,
            self.content_layout,
        )?;
        let durable = read_tail_tick(&mut tx, self.campaign)?;
        if durable == identity.resolve_tick() {
            let stored = read_stored_material_tick(&mut tx, self.campaign, durable, &self.session)?;
            if stored.envelope.canonical_bytes() != envelope.canonical_bytes() {
                return Err(MaterialRuntimeErrorV3::TailConflict);
            }
            tx.rollback()?;
            let (ack, _) = self
                .session
                .commit_prepared_and_publish(sink, candidate, |_| {
                    Ok::<_, MaterialRuntimeErrorV3>(
                        ReplayCommitDispositionV1::ReconciledAfterAmbiguousCommit,
                    )
                })
                .map_err(commit_error)?;
            self.tail = Some(ack);
            return Ok(ack);
        }
        if durable != self.session.completed_tick() {
            return Err(MaterialRuntimeErrorV3::TailConflict);
        }
        let tick_sql =
            i64::try_from(identity.resolve_tick()).map_err(|_| MaterialRuntimeErrorV3::Bounds)?;
        insert_typed_tick_pre_marker_rows_v2(
            &mut tx,
            self.campaign,
            tick_sql,
            candidate.graph_report(),
            &checkpoint,
            identity.tick_content_hash(),
        )?;
        tx.execute("INSERT INTO babylon_state.material_tick_v3 (campaign_id,resolve_tick,identity_bytes,register_bytes,receipt_bytes) VALUES ($1::uuid,$2,$3,$4,$5)",&[self.campaign.as_uuid(),&tick_sql,&identity.canonical_bytes(),&candidate.material().register().canonical_bytes(),&candidate.material().receipt_bytes()])?;
        crate::metadata::advance_campaign_catalog_tick_v1(
            &mut tx,
            self.campaign,
            tick_sql - 1,
            tick_sql,
        )?;
        let config = &self.config;
        let campaign = self.campaign;
        let content_layout = self.content_layout;
        let scope = candidate
            .graph_report()
            .result_stable_graph()
            .scenario_scope()
            .to_owned();
        let (ack,_)=self.session.commit_prepared_and_publish(sink,candidate,|_|{
            // The marker is the final durable statement. Publication capacity is already reserved.
            tx.execute("INSERT INTO babylon_state.tick_commit (campaign_id,resolve_tick,envelope_layout_version,tick_content_hash,envelope_digest) VALUES ($1::uuid,$2,3,$3,$4)",&[campaign.as_uuid(),&tick_sql,&&identity.tick_content_hash().as_bytes()[..],&&envelope.digest()[..]])?;
            match tx.commit() {Ok(())=>Ok(ReplayCommitDispositionV1::Committed),Err(error)=>{
                let mut retry_client=config.connect(NoTls)?;
                let mut retry=retry_client.transaction()?;
                crate::foundation_content_schema::lock_foundation_content_layout_v2(&mut retry,campaign,content_layout)?;
                if !marker_matches(&mut retry,campaign,&identity,&envelope)? {return Err(error.into());}
                let stored=read_stored_material_tick_rows(&mut retry,campaign,identity.resolve_tick(),&scope)?;
                if stored.identity!=identity || stored.envelope.canonical_bytes()!=envelope.canonical_bytes(){return Err(MaterialRuntimeErrorV3::TailConflict);}
                Ok(ReplayCommitDispositionV1::ReconciledAfterAmbiguousCommit)
            }}
        }).map_err(commit_error)?;
        self.tail = Some(ack);
        Ok(ack)
    }
}
fn commit_error(error: MaterialCommitErrorV3<MaterialRuntimeErrorV3>) -> MaterialRuntimeErrorV3 {
    match error {
        MaterialCommitErrorV3::Preflight(error) => error.into(),
        MaterialCommitErrorV3::Commit(error) => error,
    }
}

/// Install an exact successor schema; the old codec cannot mark a material campaign.
/// # Errors
/// Refuses absent authority, partial schema or changed installer contract.
pub fn install_material_runtime_schema_v3(config: &Config) -> Result<(), MaterialRuntimeErrorV3> {
    let bounded = bounded_material_writer_config_v3(config)?;
    let mut client = bounded.connect(NoTls)?;
    // The connection already passed target validation before trusted options
    // were added. Verify authority on this same bounded socket.
    require_active_authority_client(&mut client)?;
    crate::foundation_content_schema::install_foundation_content_schema_v2(&mut client)?;
    let mut tx = client.transaction()?;
    tx.query_one(
        "SELECT pg_catalog.pg_advisory_xact_lock($1)",
        &[&crate::SCHEMA_ADVISORY_LOCK_KEY],
    )?;
    let installed: bool = tx
        .query_one(
            "SELECT pg_catalog.to_regclass('babylon_meta.material_runtime_schema_v3') IS NOT NULL",
            &[],
        )?
        .get(0);
    let digest = sha256_of(SCHEMA.as_bytes());
    if installed {
        let stored:Vec<u8>=tx.query_one("SELECT migration_sha256 FROM babylon_meta.material_runtime_schema_v3 WHERE singleton",&[])?.get(0);
        if stored != digest {
            return Err(MaterialRuntimeErrorV3::SchemaDrift);
        }
    } else {
        let partial:bool=tx.query_one("SELECT pg_catalog.to_regclass('babylon_state.material_campaign_foundation_v2') IS NOT NULL OR pg_catalog.to_regclass('babylon_state.material_tick_v3') IS NOT NULL",&[])?.get(0);
        if partial {
            return Err(MaterialRuntimeErrorV3::SchemaDrift);
        }
        tx.batch_execute(SCHEMA)?;
        tx.batch_execute("CREATE TABLE babylon_meta.material_runtime_schema_v3 (singleton boolean PRIMARY KEY CHECK(singleton),migration_sha256 bytea NOT NULL CHECK(octet_length(migration_sha256)=32)); REVOKE ALL ON babylon_meta.material_runtime_schema_v3 FROM PUBLIC")?;
        tx.execute(
            "INSERT INTO babylon_meta.material_runtime_schema_v3 VALUES (true,$1)",
            &[&&digest[..]],
        )?;
    }
    tx.commit()?;
    Ok(())
}
struct StoredMaterialFoundationV2 {
    spec: MaterialFoundationSpecV2,
    initial_register_bytes: Vec<u8>,
    foundation_bytes: Vec<u8>,
    foundation_digest: [u8; 32],
    graph_foundation_digest: [u8; 32],
}

impl StoredMaterialFoundationV2 {
    fn from_row(row: &postgres::Row) -> Result<Self, MaterialRuntimeErrorV3> {
        let digest = |column: usize| -> Result<[u8; 32], MaterialRuntimeErrorV3> {
            row.try_get::<_, Vec<u8>>(column)?
                .try_into()
                .map_err(|_| MaterialRuntimeErrorV3::FoundationMismatch)
        };
        Ok(Self {
            spec: MaterialFoundationSpecV2 {
                preset_id: row.try_get(0)?,
                horizon_ticks: u64::try_from(row.try_get::<_, i64>(1)?)
                    .map_err(|_| MaterialRuntimeErrorV3::FoundationMismatch)?,
                content_digest: digest(2)?,
            },
            initial_register_bytes: row.try_get(3)?,
            foundation_bytes: row.try_get(4)?,
            foundation_digest: digest(5)?,
            graph_foundation_digest: digest(6)?,
        })
    }
}

fn hydrate_material_foundation_v2(
    client: &mut impl GenericClient,
    campaign: CampaignId,
    expected_foundation_digest: [u8; 32],
) -> Result<MaterialRuntimeFoundationV2, MaterialRuntimeErrorV3> {
    let row=client.query_opt("SELECT f.preset_id,f.horizon_ticks,f.content_sha256,f.initial_register_bytes,f.foundation_bytes,f.foundation_sha256,g.foundation_sha256 FROM babylon_state.material_campaign_foundation_v2 f JOIN babylon_state.campaign_foundation g USING(campaign_id) WHERE campaign_id=$1::uuid",&[campaign.as_uuid()])?;
    let Some(row) = row else {
        let exists = client
            .query_opt(
                "SELECT campaign_id FROM babylon_state.campaign WHERE campaign_id=$1::uuid",
                &[campaign.as_uuid()],
            )?
            .is_some();
        return Err(if exists {
            MaterialRuntimeErrorV3::LegacyCampaign
        } else {
            MaterialRuntimeErrorV3::MissingCampaign
        });
    };
    let stored = StoredMaterialFoundationV2::from_row(&row)?;
    if stored.foundation_digest != expected_foundation_digest {
        return Err(MaterialRuntimeErrorV3::FoundationMismatch);
    }
    let graph = crate::runtime::hydrate_campaign_foundation_client_v1(client, campaign)?;
    reconstruct_material_foundation_v2(stored, graph, expected_foundation_digest)
}

fn reconstruct_material_foundation_v2(
    stored: StoredMaterialFoundationV2,
    graph_foundation: CampaignFoundationV1,
    expected_foundation_digest: [u8; 32],
) -> Result<MaterialRuntimeFoundationV2, MaterialRuntimeErrorV3> {
    if stored.foundation_digest != expected_foundation_digest
        || sha256_of(&stored.foundation_bytes) != expected_foundation_digest
        || sha256_of(graph_foundation.canonical_bytes()) != stored.graph_foundation_digest
    {
        return Err(MaterialRuntimeErrorV3::FoundationMismatch);
    }
    let register = MaterialWorldRegisterV2::decode(&stored.initial_register_bytes)?;
    if register.completed_tick() != 0 {
        return Err(MaterialRuntimeErrorV3::FoundationMismatch);
    }
    let graph = reconstruct_graph_foundation_session_v1(&graph_foundation)?;
    let reconstructed =
        MaterialRuntimeFoundationV2::from_parts(graph, graph_foundation, register, stored.spec)?;
    if reconstructed.canonical_bytes() != stored.foundation_bytes
        || reconstructed.digest() != expected_foundation_digest
    {
        return Err(MaterialRuntimeErrorV3::FoundationMismatch);
    }
    Ok(reconstructed)
}
fn read_tail_tick(
    client: &mut impl GenericClient,
    campaign: CampaignId,
) -> Result<u64, MaterialRuntimeErrorV3> {
    let row=client.query_one("SELECT count(*),coalesce(max(resolve_tick),0),coalesce(bool_and(envelope_layout_version=3),true) FROM babylon_state.tick_commit WHERE campaign_id=$1::uuid",&[campaign.as_uuid()])?;
    let count: i64 = row.try_get(0)?;
    let tick: i64 = row.try_get(1)?;
    let version: bool = row.try_get(2)?;
    if count != tick || !version {
        return Err(MaterialRuntimeErrorV3::TailConflict);
    }
    u64::try_from(tick).map_err(|_| MaterialRuntimeErrorV3::TailConflict)
}
fn marker_matches(
    client: &mut impl GenericClient,
    campaign: CampaignId,
    identity: &IdentifiedMaterialTickV3,
    envelope: &CommittedMaterialTickEnvelopeV3,
) -> Result<bool, MaterialRuntimeErrorV3> {
    let tick =
        i64::try_from(identity.resolve_tick()).map_err(|_| MaterialRuntimeErrorV3::Bounds)?;
    let Some(row)=client.query_opt("SELECT envelope_layout_version,tick_content_hash,envelope_digest FROM babylon_state.tick_commit WHERE campaign_id=$1::uuid AND resolve_tick=$2",&[campaign.as_uuid(),&tick])? else{return Ok(false);};
    if row.try_get::<_, i16>(0)? != 3
        || row.try_get::<_, Vec<u8>>(1)? != identity.tick_content_hash().as_bytes()
        || row.try_get::<_, Vec<u8>>(2)? != envelope.digest()
    {
        return Err(MaterialRuntimeErrorV3::TailConflict);
    }
    Ok(true)
}

struct StoredMaterialTickV3 {
    identity: IdentifiedMaterialTickV3,
    envelope: CommittedMaterialTickEnvelopeV3,
    graph: babylon_graph::stable_state::StableGraphStateV1,
    material: babylon_tick::material_state::MaterialStateRowsV1,
    sections: Vec<Vec<u8>>,
    register: Vec<u8>,
}
fn read_stored_material_tick(
    client: &mut impl GenericClient,
    campaign: CampaignId,
    tick: u64,
    session: &MaterialReplaySessionV3<HypergraphStore>,
) -> Result<StoredMaterialTickV3, MaterialRuntimeErrorV3> {
    let scope = session
        .graph_session()
        .stable_graph_state()
        .map_err(MaterialReplayErrorV3::Graph)?
        .scenario_scope()
        .to_owned();
    let stored = read_stored_material_tick_rows(client, campaign, tick, &scope)?;
    if stored.identity.foundation_digest() != session.foundation_digest() {
        return Err(MaterialRuntimeErrorV3::InvalidCheckpoint);
    }
    validate_component_identity(client, campaign, tick, session, &stored.sections)?;
    Ok(stored)
}
fn read_stored_material_tick_rows(
    client: &mut impl GenericClient,
    campaign: CampaignId,
    tick: u64,
    scope: &str,
) -> Result<StoredMaterialTickV3, MaterialRuntimeErrorV3> {
    let tick_sql = i64::try_from(tick).map_err(|_| MaterialRuntimeErrorV3::Bounds)?;
    let row=client.query_opt("SELECT identity_bytes,register_bytes,receipt_bytes FROM babylon_state.material_tick_v3 WHERE campaign_id=$1::uuid AND resolve_tick=$2",&[campaign.as_uuid(),&tick_sql])?.ok_or(MaterialRuntimeErrorV3::InvalidCheckpoint)?;
    let identity = IdentifiedMaterialTickV3::decode(&row.try_get::<_, Vec<u8>>(0)?)?;
    let register: Vec<u8> = row.try_get(1)?;
    let receipts: Vec<u8> = row.try_get(2)?;
    if identity.resolve_tick() != tick
        || MaterialWorldRegisterV2::decode(&register)?.completed_tick() != tick
    {
        return Err(MaterialRuntimeErrorV3::InvalidCheckpoint);
    }
    let graph = stored_tick::read_graph_state(client, campaign, tick_sql, scope)?;
    let material = stored_tick::read_material_rows(client, campaign, tick_sql)?;
    let (checkpoint, sections) =
        stored_tick::read_checkpoint_rows(client, campaign, tick, tick_sql, &graph, &material)?;
    let (graph_rows, _) =
        compose_graph_rows_with_encoder_v1(graph.rows(), &mut |row: StableGraphRowRefV1<'_>| {
            row.encode()
        })?;
    let families = CommittedTickRowFamiliesV2 {
        graph: graph_rows,
        state: compose_material_state_rows_v1(&material)?,
        event: stored_tick::read_event_rows(client, campaign, tick_sql)?,
        choice_receipt: stored_tick::read_choice_receipt_rows(client, campaign, tick_sql)?,
        checkpoint,
        archive_dirty_receipt: stored_tick::read_archive_receipt(client, campaign, tick_sql)?,
    };
    let envelope = CommittedMaterialTickEnvelopeV3::compose(
        campaign, &identity, families, &register, &receipts,
    )?;
    if !marker_matches(client, campaign, &identity, &envelope)? {
        return Err(MaterialRuntimeErrorV3::InvalidCheckpoint);
    }
    Ok(StoredMaterialTickV3 {
        identity,
        envelope,
        graph,
        material,
        sections,
        register,
    })
}
fn validate_component_identity(
    client: &mut impl GenericClient,
    campaign: CampaignId,
    tick: u64,
    session: &MaterialReplaySessionV3<HypergraphStore>,
    sections: &[Vec<u8>],
) -> Result<(), MaterialRuntimeErrorV3> {
    let graph = session.graph_session();
    let seed = graph.rng_seed().to_be_bytes();
    let reference = graph.reference_digest();
    let mut content = [0_u8; 64];
    content[..32].copy_from_slice(&graph.content_digest().defines_hash);
    content[32..].copy_from_slice(&graph.content_digest().rules_hash);
    let expected = [
        graph.resolver_manifest_bytes(),
        graph.prepared_environment_bytes(),
        graph.session_identity().as_bytes(),
        seed.as_slice(),
        content.as_slice(),
        reference.as_bytes().as_slice(),
    ];
    if sections.len() != 9
        || expected
            .iter()
            .enumerate()
            .any(|(index, bytes)| sections[index + 2].as_slice() != *bytes)
    {
        return Err(MaterialRuntimeErrorV3::InvalidCheckpoint);
    }
    let actions = OrderedPracticeActionBatchV1::empty(graph.session_identity().clone(), tick)
        .map_err(|_| MaterialRuntimeErrorV3::InvalidCheckpoint)?;
    let tick_sql = i64::try_from(tick).map_err(|_| MaterialRuntimeErrorV3::Bounds)?;
    let row=client.query_one("SELECT layout_version,action_batch_digest,exact_action_batch_bytes FROM babylon_state.tick_action_batch_v1 WHERE campaign_id=$1::uuid AND resolve_tick=$2",&[campaign.as_uuid(),&tick_sql])?;
    if row.try_get::<_, i16>(0)? != 1
        || row.try_get::<_, Vec<u8>>(1)? != actions.digest().as_bytes()
        || row.try_get::<_, Vec<u8>>(2)? != actions.canonical_bytes()
    {
        return Err(MaterialRuntimeErrorV3::InvalidCheckpoint);
    }
    Ok(())
}

/// Exact shared Michigan graph and material foundation, with no implicit advance.
/// # Errors
/// Refuses source digest/shape failures, invalid designed content or aggregate bounds.
pub fn michigan_material_runtime_foundation_v2(
    preset: crate::michigan_material::MichiganDeliveryPresetV1,
) -> Result<MaterialRuntimeFoundationV2, MaterialRuntimeErrorV3> {
    use crate::michigan_material::{
        michigan_material_foundation_v1, MICHIGAN_INDUSTRY_BASELINE_SHA256_V1,
        MICHIGAN_MATERIAL_SCENARIO_SHA256_V1,
    };
    let (graph, bundle) = crate::michigan_economy::michigan_observer_foundation_v1()
        .map_err(MaterialRuntimeErrorV3::MichiganEconomy)?;
    let state = michigan_material_foundation_v1(preset)
        .map_err(MaterialRuntimeErrorV3::MichiganMaterial)?;
    let mut bytes = Vec::from(&b"babylon.michigan-material-content.v1\0"[..]);
    bytes.extend_from_slice(MICHIGAN_MATERIAL_SCENARIO_SHA256_V1.as_bytes());
    bytes.extend_from_slice(MICHIGAN_INDUSTRY_BASELINE_SHA256_V1.as_bytes());
    MaterialRuntimeFoundationV2::capture(
        graph,
        bundle,
        state,
        MaterialFoundationSpecV2 {
            preset_id: preset.id().to_owned(),
            horizon_ticks: preset.horizon_ticks(),
            content_digest: sha256_of(&bytes),
        },
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
        let bounded = bounded_material_writer_config_v3(&caller).unwrap();
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
        crate::validate_legacy_connection_target(&caller).unwrap();
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
                bounded_material_writer_config_v3(&caller),
                Err(MaterialRuntimeErrorV3::Graph(
                    RustPersistenceRuntimeErrorV2::ActivationRequired
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
            bounded_material_writer_config_v3(&caller),
            Err(MaterialRuntimeErrorV3::Graph(
                RustPersistenceRuntimeErrorV2::ActivationRequired
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
        let bounded = bounded_material_writer_config_v3(&raw).unwrap();
        let mut client = bounded.connect(NoTls).expect("bounded writer connection");
        let mut transaction = client
            .build_transaction()
            .read_only(true)
            .start()
            .expect("read-only authority probe");
        require_active_authority_client(&mut transaction)
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
