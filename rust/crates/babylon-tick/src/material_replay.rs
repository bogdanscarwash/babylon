//! Explicit V3 successor owner for atomic graph and routed-material replay.
//!
//! Physical closing and admitted staffing join the detached graph transaction
//! before identity finalization. One acknowledgement publishes both owners.

use crate::{
    material_staffing::{
        apply_material_staffing, MaterialStaffingError, StaffingComposition, StaffingEffectContext,
        StaffingEffects,
    },
    material_state::MaterialStateRows,
    material_world::{
        nominal_material_world_hash, MaterialWorldError, MaterialWorldRegister,
        PreparedMaterialWorld,
    },
    replay_session::{
        IdentifiedTickReport, PreparedReplayCommitError, PreparedReplayTick,
        ReplayCommitDisposition, ReplayTickError, ReplayTickSession,
    },
};
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::{
    allocator_state::AllocatorState, stable_state::StableGraphState, state_hash::CanonicalState,
    substrate::GraphSubstrate, working_copy::DetachedCopy,
};
use babylon_kernel::{content_digest::sha256_of, tick_content_hash::TickContentHash};
use babylon_material_circuit::{close_material_period, MaterialCircuitError};
use babylon_practice_contract::OrderedPracticeActionBatch;

const TICK_DOMAIN: &[u8] = b"babylon.material-tick-content.v3\0";
const TICK_IDENTITY_BYTES: usize = TICK_DOMAIN.len() + 12 + 7 * 32;

/// Typed refusal inside the detached material-base composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialBaseError {
    World(MaterialWorldError),
    Staffing(MaterialStaffingError),
    /// A BSL rule could overwrite a graph field owned by native staffing.
    StaffingFieldOwner {
        rule_id: String,
        field: String,
    },
    /// The existing bounded effect analyzer refused the retained rule AST.
    StaffingEffectAnalysis {
        rule_id: String,
        error: babylon_bsl::causal_contract::ContractError,
    },
    Period,
    MissingCandidate,
    MissingResolver,
}
impl std::fmt::Display for MaterialBaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "material-base composition refused: {self:?}")
    }
}
impl std::error::Error for MaterialBaseError {}
impl From<MaterialWorldError> for MaterialBaseError {
    fn from(error: MaterialWorldError) -> Self {
        Self::World(error)
    }
}
impl From<MaterialCircuitError> for MaterialBaseError {
    fn from(error: MaterialCircuitError) -> Self {
        Self::World(error.into())
    }
}
impl From<MaterialStaffingError> for MaterialBaseError {
    fn from(error: MaterialStaffingError) -> Self {
        Self::Staffing(error)
    }
}

pub(crate) struct MaterialBaseInputs<'a> {
    pub(crate) opening: &'a MaterialWorldRegister,
    pub(crate) labor: &'a StaffingComposition,
}
impl MaterialBaseInputs<'_> {
    pub(crate) fn prepare(
        self,
        graph: &mut impl GraphSubstrate,
        context: StaffingEffectContext<'_>,
        tick: i64,
    ) -> Result<(PreparedMaterialWorld, Option<StaffingEffects>), MaterialBaseError> {
        if u64::try_from(tick).ok() != self.opening.completed_tick().checked_add(1) {
            return Err(MaterialBaseError::Period);
        }
        let composition = self.labor;
        {
            let closed = close_material_period(self.opening.state())?;
            let bindings = composition
                .bindings()
                .iter()
                .map(|row| row.pool().clone())
                .collect::<Vec<_>>();
            let requests = closed.staffing_requests(&bindings)?;
            let effects = apply_material_staffing(
                graph,
                context,
                composition,
                closed.closing_period(),
                &requests,
            )?;
            let transition = closed.finish_with_labor(effects.next_labor().to_vec())?;
            Ok((self.opening.prepare_transition(transition)?, Some(effects)))
        }
    }
}

/// Closed errors at the material session boundary.
#[derive(Debug)]
pub enum MaterialReplayError {
    Graph(ReplayTickError),
    Material(MaterialWorldError),
    FoundationTick,
    Horizon,
    StaleCandidate,
    Identity,
}
impl std::fmt::Display for MaterialReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "material replay refused: {self:?}")
    }
}
impl std::error::Error for MaterialReplayError {}
impl From<ReplayTickError> for MaterialReplayError {
    fn from(value: ReplayTickError) -> Self {
        Self::Graph(value)
    }
}
impl From<MaterialWorldError> for MaterialReplayError {
    fn from(value: MaterialWorldError) -> Self {
        Self::Material(value)
    }
}

/// Identity emitted only by successful detached adjudication of both components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentifiedMaterialTick {
    resolve_tick: u64,
    foundation_digest: [u8; 32],
    graph_tick_content_hash: TickContentHash,
    graph_world_before: [u8; 32],
    graph_world_after: [u8; 32],
    prior_world_hash: [u8; 32],
    result_world_hash: [u8; 32],
    receipt_digest: [u8; 32],
    canonical_bytes: [u8; TICK_IDENTITY_BYTES],
    tick_content_hash: TickContentHash,
}
impl IdentifiedMaterialTick {
    #[must_use]
    pub const fn resolve_tick(&self) -> u64 {
        self.resolve_tick
    }
    #[must_use]
    pub const fn foundation_digest(&self) -> [u8; 32] {
        self.foundation_digest
    }
    #[must_use]
    pub const fn graph_tick_content_hash(&self) -> TickContentHash {
        self.graph_tick_content_hash
    }
    #[must_use]
    pub const fn graph_world_before(&self) -> [u8; 32] {
        self.graph_world_before
    }
    #[must_use]
    pub const fn graph_world_after(&self) -> [u8; 32] {
        self.graph_world_after
    }
    #[must_use]
    pub const fn prior_world_hash(&self) -> [u8; 32] {
        self.prior_world_hash
    }
    #[must_use]
    pub const fn result_world_hash(&self) -> [u8; 32] {
        self.result_world_hash
    }
    #[must_use]
    pub const fn receipt_digest(&self) -> [u8; 32] {
        self.receipt_digest
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    #[must_use]
    pub const fn tick_content_hash(&self) -> TickContentHash {
        self.tick_content_hash
    }
    /// Decode the closed fixed-width V3 tick identity.
    /// # Errors
    /// Refuses wrong domain/version/length, zero tick or trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, MaterialReplayError> {
        let head = TICK_DOMAIN.len();
        if bytes.len() != head + 12 + 7 * 32
            || !bytes.starts_with(TICK_DOMAIN)
            || bytes[head..head + 4] != 3_u32.to_be_bytes()
        {
            return Err(MaterialReplayError::Identity);
        }
        let resolve_tick = u64::from_be_bytes(
            bytes[head + 4..head + 12]
                .try_into()
                .map_err(|_| MaterialReplayError::Identity)?,
        );
        if resolve_tick == 0 {
            return Err(MaterialReplayError::Identity);
        }
        let mut digests = [[0_u8; 32]; 7];
        for (index, digest) in digests.iter_mut().enumerate() {
            digest.copy_from_slice(&bytes[head + 12 + index * 32..head + 12 + (index + 1) * 32]);
        }
        Ok(Self {
            resolve_tick,
            foundation_digest: digests[0],
            graph_tick_content_hash: TickContentHash::from_bytes(digests[1]),
            graph_world_before: digests[2],
            graph_world_after: digests[3],
            prior_world_hash: digests[4],
            result_world_hash: digests[5],
            receipt_digest: digests[6],
            canonical_bytes: bytes
                .try_into()
                .map_err(|_| MaterialReplayError::Identity)?,
            tick_content_hash: TickContentHash::from_bytes(sha256_of(bytes)),
        })
    }
    fn compose(
        foundation: [u8; 32],
        graph: &IdentifiedTickReport,
        prior: &MaterialWorldRegister,
        material: &PreparedMaterialWorld,
    ) -> Result<Self, MaterialReplayError> {
        let resolve_tick = material.register().completed_tick();
        if u64::try_from(graph.result_registers().completed_tick()).ok() != Some(resolve_tick) {
            return Err(MaterialReplayError::Identity);
        }
        let prior_world_hash = nominal_material_world_hash(graph.report().world_before, prior);
        let result_world_hash =
            nominal_material_world_hash(graph.report().world_after, material.register());
        let receipt_digest = sha256_of(material.receipt_bytes());
        let mut canonical_bytes = [0_u8; TICK_IDENTITY_BYTES];
        let head = TICK_DOMAIN.len();
        canonical_bytes[..head].copy_from_slice(TICK_DOMAIN);
        canonical_bytes[head..head + 4].copy_from_slice(&3_u32.to_be_bytes());
        canonical_bytes[head + 4..head + 12].copy_from_slice(&resolve_tick.to_be_bytes());
        for (slot, digest) in canonical_bytes[head + 12..].chunks_exact_mut(32).zip([
            foundation,
            *graph.tick_content_hash().as_bytes(),
            graph.report().world_before,
            graph.report().world_after,
            prior_world_hash,
            result_world_hash,
            receipt_digest,
        ]) {
            slot.copy_from_slice(&digest);
        }
        let tick_content_hash = TickContentHash::from_bytes(sha256_of(&canonical_bytes));
        Ok(Self {
            resolve_tick,
            foundation_digest: foundation,
            graph_tick_content_hash: graph.tick_content_hash(),
            graph_world_before: graph.report().world_before,
            graph_world_after: graph.report().world_after,
            prior_world_hash,
            result_world_hash,
            receipt_digest,
            canonical_bytes,
            tick_content_hash,
        })
    }
}

/// Sole active owner; no mutable access to either graph or circuit component.
pub struct MaterialReplaySession<G> {
    graph: ReplayTickSession<G>,
    material: MaterialWorldRegister,
    foundation_digest: [u8; 32],
    horizon: u64,
    labor: StaffingComposition,
}
/// Fully prepared candidate; dropping it publishes nothing.
pub struct PreparedMaterialTick<G> {
    graph: PreparedReplayTick<G>,
    material: PreparedMaterialWorld,
    identity: IdentifiedMaterialTick,
}
impl<G> PreparedMaterialTick<G> {
    #[must_use]
    pub const fn graph_report(&self) -> &IdentifiedTickReport {
        self.graph.report()
    }
    #[must_use]
    pub const fn material(&self) -> &PreparedMaterialWorld {
        &self.material
    }
    #[must_use]
    pub const fn identity(&self) -> &IdentifiedMaterialTick {
        &self.identity
    }
}
/// Fallible preflight or durable operation; neither publishes any candidate state.
#[derive(Debug)]
pub enum MaterialCommitError<E> {
    Preflight(MaterialReplayError),
    Commit(E),
}

impl<G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy> MaterialReplaySession<G> {
    /// Bind a new foundation at tick zero. Existing graph sessions cannot acquire mechanics.
    /// # Errors
    /// Refuses nonzero component clocks, an empty horizon, invalid state, or
    /// BSL writes to staffing-owned fields when staffed labor is selected.
    pub fn new(
        graph: ReplayTickSession<G>,
        material: MaterialWorldRegister,
        foundation_digest: [u8; 32],
        horizon: u64,
        labor: StaffingComposition,
    ) -> Result<Self, MaterialReplayError> {
        if graph.completed_tick() != 0 || material.completed_tick() != 0 {
            return Err(MaterialReplayError::FoundationTick);
        }
        if horizon == 0 || horizon > i64::MAX as u64 {
            return Err(MaterialReplayError::Horizon);
        }
        graph.validate_staffing_ownership()?;
        Ok(Self {
            graph,
            material,
            foundation_digest,
            horizon,
            labor,
        })
    }
    #[must_use]
    pub const fn graph_session(&self) -> &ReplayTickSession<G> {
        &self.graph
    }
    #[must_use]
    pub const fn material(&self) -> &MaterialWorldRegister {
        &self.material
    }
    #[must_use]
    pub const fn completed_tick(&self) -> u64 {
        self.material.completed_tick()
    }
    #[must_use]
    pub const fn foundation_digest(&self) -> [u8; 32] {
        self.foundation_digest
    }
    #[must_use]
    pub const fn horizon(&self) -> u64 {
        self.horizon
    }

    /// Hash the currently held graph and material world under the successor domain.
    /// # Errors
    /// Refuses invalid graph values or nominal component encoding.
    pub fn current_world_hash(&self) -> Result<[u8; 32], MaterialReplayError> {
        let graph = self.graph.graph();
        let hash = graph
            .state_hash()
            .map_err(|_| MaterialReplayError::Identity)?;
        let nominal = crate::world_hash::nominal_world_hash(
            hash,
            self.graph.completed_tick(),
            graph.allocator_cursors(),
            crate::phase_order::schedule_digest().map_err(|_| MaterialReplayError::Identity)?,
        )
        .map_err(|_| MaterialReplayError::Identity)?;
        Ok(nominal_material_world_hash(nominal, &self.material))
    }
    /// Prepare one exact interval, with prior commitments and routed freight governed by V2.
    /// # Errors
    /// Either component failure leaves both live owners and all sinks unchanged.
    pub fn prepare_advance(
        &self,
        actions: &OrderedPracticeActionBatch,
    ) -> Result<PreparedMaterialTick<G>, MaterialReplayError> {
        if self.completed_tick() >= self.horizon {
            return Err(MaterialReplayError::Horizon);
        }
        let (graph, material) = self.graph.prepare_material_advance(
            actions,
            MaterialBaseInputs {
                opening: &self.material,
                labor: &self.labor,
            },
        )?;
        let identity = IdentifiedMaterialTick::compose(
            self.foundation_digest,
            graph.report(),
            &self.material,
            &material,
        )?;
        Ok(PreparedMaterialTick {
            graph,
            material,
            identity,
        })
    }
    /// Preflight all owners, commit once, then publish both components using only infallible moves.
    /// # Errors
    /// Returns a stale/allocation preflight or the durable operation's precise error.
    pub fn commit_prepared_and_publish<E, F>(
        &mut self,
        sink: &mut CollectingSink,
        prepared: PreparedMaterialTick<G>,
        commit: F,
    ) -> Result<(IdentifiedMaterialTick, ReplayCommitDisposition), MaterialCommitError<E>>
    where
        F: FnOnce(&IdentifiedMaterialTick) -> Result<ReplayCommitDisposition, E>,
    {
        if prepared.material.prior_digest() != self.material.digest()
            || prepared.identity.foundation_digest != self.foundation_digest
            || prepared.identity.resolve_tick != self.completed_tick().saturating_add(1)
        {
            return Err(MaterialCommitError::Preflight(
                MaterialReplayError::StaleCandidate,
            ));
        }
        let (_, disposition) = self
            .graph
            .commit_prepared_and_publish(sink, prepared.graph, |_| commit(&prepared.identity))
            .map_err(|error| match error {
                PreparedReplayCommitError::Preflight(error) => {
                    MaterialCommitError::Preflight(MaterialReplayError::Graph(error))
                }
                PreparedReplayCommitError::Commit(error) => MaterialCommitError::Commit(error),
            })?;
        self.material = prepared.material.into_register();
        Ok((prepared.identity, disposition))
    }
}

impl MaterialReplaySession<babylon_graph::hypergraph_store::HypergraphStore> {
    /// Restore checked component checkpoint sections under the exact pinned foundation.
    /// # Errors
    /// Every decode, tick or graph restore refusal leaves both live owners unchanged.
    pub fn restore_full_checkpoint(
        &mut self,
        graph_state: &StableGraphState,
        graph_material: &MaterialStateRows,
        graph_registers: &[u8],
        material_bytes: &[u8],
    ) -> Result<(), MaterialReplayError> {
        let material = MaterialWorldRegister::decode(material_bytes)?;
        let tick = material.completed_tick();
        if tick == 0 || tick > self.horizon {
            return Err(MaterialReplayError::Horizon);
        }
        let tick = i64::try_from(tick).map_err(|_| MaterialReplayError::Identity)?;
        self.graph
            .restore_full_checkpoint(tick, graph_state, graph_material, graph_registers)?;
        self.material = material;
        Ok(())
    }
}
