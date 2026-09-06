//! Explicit V3 successor owner for atomic graph and routed-material replay.
//!
//! Graph adjudication is an unchanged component. The active world's register,
//! nominal identity and tick content identity are the versioned combined values.

use crate::{
    material_state::MaterialStateRowsV1,
    material_world::{
        nominal_material_world_hash_v2, MaterialWorldErrorV2, MaterialWorldRegisterV2,
        PreparedMaterialWorldV3,
    },
    replay_session::{
        IdentifiedTickReportV2, PreparedReplayCommitErrorV1, PreparedReplayTickV1,
        ReplayCommitDispositionV1, ReplayTickError, ReplayTickSession,
    },
};
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::{
    allocator_state::AllocatorState, stable_state::StableGraphStateV1, state_hash::CanonicalState,
    substrate::GraphSubstrate, working_copy::DetachedCopy,
};
use babylon_kernel::{sha256_of, tick_content_hash::TickContentHashV1};
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;

const TICK_DOMAIN: &[u8] = b"babylon.material-tick-content.v3\0";
const TICK_IDENTITY_BYTES_V3: usize = TICK_DOMAIN.len() + 12 + 7 * 32;

/// Closed errors at the material session boundary.
#[derive(Debug)]
pub enum MaterialReplayErrorV3 {
    Graph(ReplayTickError),
    Material(MaterialWorldErrorV2),
    FoundationTick,
    Horizon,
    StaleCandidate,
    Identity,
}
impl std::fmt::Display for MaterialReplayErrorV3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "material replay refused: {self:?}")
    }
}
impl std::error::Error for MaterialReplayErrorV3 {}
impl From<ReplayTickError> for MaterialReplayErrorV3 {
    fn from(value: ReplayTickError) -> Self {
        Self::Graph(value)
    }
}
impl From<MaterialWorldErrorV2> for MaterialReplayErrorV3 {
    fn from(value: MaterialWorldErrorV2) -> Self {
        Self::Material(value)
    }
}

/// Identity emitted only by successful detached adjudication of both components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentifiedMaterialTickV3 {
    resolve_tick: u64,
    foundation_digest: [u8; 32],
    graph_tick_content_hash: TickContentHashV1,
    graph_world_before: [u8; 32],
    graph_world_after: [u8; 32],
    prior_world_hash: [u8; 32],
    result_world_hash: [u8; 32],
    receipt_digest: [u8; 32],
    canonical_bytes: [u8; TICK_IDENTITY_BYTES_V3],
    tick_content_hash: TickContentHashV1,
}
impl IdentifiedMaterialTickV3 {
    #[must_use]
    pub const fn resolve_tick(&self) -> u64 {
        self.resolve_tick
    }
    #[must_use]
    pub const fn foundation_digest(&self) -> [u8; 32] {
        self.foundation_digest
    }
    #[must_use]
    pub const fn graph_tick_content_hash(&self) -> TickContentHashV1 {
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
    pub const fn tick_content_hash(&self) -> TickContentHashV1 {
        self.tick_content_hash
    }
    /// Decode the closed fixed-width V3 tick identity.
    /// # Errors
    /// Refuses wrong domain/version/length, zero tick or trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, MaterialReplayErrorV3> {
        let head = TICK_DOMAIN.len();
        if bytes.len() != head + 12 + 7 * 32
            || !bytes.starts_with(TICK_DOMAIN)
            || bytes[head..head + 4] != 3_u32.to_be_bytes()
        {
            return Err(MaterialReplayErrorV3::Identity);
        }
        let resolve_tick = u64::from_be_bytes(
            bytes[head + 4..head + 12]
                .try_into()
                .map_err(|_| MaterialReplayErrorV3::Identity)?,
        );
        if resolve_tick == 0 {
            return Err(MaterialReplayErrorV3::Identity);
        }
        let mut digests = [[0_u8; 32]; 7];
        for (index, digest) in digests.iter_mut().enumerate() {
            digest.copy_from_slice(&bytes[head + 12 + index * 32..head + 12 + (index + 1) * 32]);
        }
        Ok(Self {
            resolve_tick,
            foundation_digest: digests[0],
            graph_tick_content_hash: TickContentHashV1::from_bytes(digests[1]),
            graph_world_before: digests[2],
            graph_world_after: digests[3],
            prior_world_hash: digests[4],
            result_world_hash: digests[5],
            receipt_digest: digests[6],
            canonical_bytes: bytes
                .try_into()
                .map_err(|_| MaterialReplayErrorV3::Identity)?,
            tick_content_hash: TickContentHashV1::from_bytes(sha256_of(bytes)),
        })
    }
    fn compose(
        foundation: [u8; 32],
        graph: &IdentifiedTickReportV2,
        prior: &MaterialWorldRegisterV2,
        material: &PreparedMaterialWorldV3,
    ) -> Result<Self, MaterialReplayErrorV3> {
        let resolve_tick = material.register().completed_tick();
        if u64::try_from(graph.result_registers().completed_tick()).ok() != Some(resolve_tick) {
            return Err(MaterialReplayErrorV3::Identity);
        }
        let prior_world_hash = nominal_material_world_hash_v2(graph.report().world_before, prior);
        let result_world_hash =
            nominal_material_world_hash_v2(graph.report().world_after, material.register());
        let receipt_digest = sha256_of(material.receipt_bytes());
        let mut canonical_bytes = [0_u8; TICK_IDENTITY_BYTES_V3];
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
        let tick_content_hash = TickContentHashV1::from_bytes(sha256_of(&canonical_bytes));
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
pub struct MaterialReplaySessionV3<G> {
    graph: ReplayTickSession<G>,
    material: MaterialWorldRegisterV2,
    foundation_digest: [u8; 32],
    horizon: u64,
}
/// Fully prepared candidate; dropping it publishes nothing.
pub struct PreparedMaterialTickV3<G> {
    graph: PreparedReplayTickV1<G>,
    material: PreparedMaterialWorldV3,
    identity: IdentifiedMaterialTickV3,
}
impl<G> PreparedMaterialTickV3<G> {
    #[must_use]
    pub const fn graph_report(&self) -> &IdentifiedTickReportV2 {
        self.graph.report()
    }
    #[must_use]
    pub const fn material(&self) -> &PreparedMaterialWorldV3 {
        &self.material
    }
    #[must_use]
    pub const fn identity(&self) -> &IdentifiedMaterialTickV3 {
        &self.identity
    }
}
/// Fallible preflight or durable operation; neither publishes any candidate state.
#[derive(Debug)]
pub enum MaterialCommitErrorV3<E> {
    Preflight(MaterialReplayErrorV3),
    Commit(E),
}

impl<G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy>
    MaterialReplaySessionV3<G>
{
    /// Bind a new foundation at tick zero. Existing graph sessions cannot acquire mechanics.
    /// # Errors
    /// Refuses nonzero component clocks, an empty horizon or invalid state.
    pub fn new(
        graph: ReplayTickSession<G>,
        material: MaterialWorldRegisterV2,
        foundation_digest: [u8; 32],
        horizon: u64,
    ) -> Result<Self, MaterialReplayErrorV3> {
        if graph.completed_tick() != 0 || material.completed_tick() != 0 {
            return Err(MaterialReplayErrorV3::FoundationTick);
        }
        if horizon == 0 || horizon > i64::MAX as u64 {
            return Err(MaterialReplayErrorV3::Horizon);
        }
        Ok(Self {
            graph,
            material,
            foundation_digest,
            horizon,
        })
    }
    #[must_use]
    pub const fn graph_session(&self) -> &ReplayTickSession<G> {
        &self.graph
    }
    #[must_use]
    pub const fn material(&self) -> &MaterialWorldRegisterV2 {
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
    pub fn current_world_hash(&self) -> Result<[u8; 32], MaterialReplayErrorV3> {
        let graph = self.graph.graph();
        let hash = graph
            .state_hash()
            .map_err(|_| MaterialReplayErrorV3::Identity)?;
        let nominal = crate::world_hash::nominal_world_hash(
            hash,
            self.graph.completed_tick(),
            graph.allocator_cursors(),
            crate::phase_order::schedule_digest().map_err(|_| MaterialReplayErrorV3::Identity)?,
        )
        .map_err(|_| MaterialReplayErrorV3::Identity)?;
        Ok(nominal_material_world_hash_v2(nominal, &self.material))
    }
    /// Prepare one exact interval, with prior commitments and routed freight governed by V2.
    /// # Errors
    /// Either component failure leaves both live owners and all sinks unchanged.
    pub fn prepare_advance(
        &self,
        actions: &OrderedPracticeActionBatchV1,
    ) -> Result<PreparedMaterialTickV3<G>, MaterialReplayErrorV3> {
        if self.completed_tick() >= self.horizon {
            return Err(MaterialReplayErrorV3::Horizon);
        }
        let graph = self.graph.prepare_advance(actions)?;
        let material = self.material.prepare_next()?;
        let identity = IdentifiedMaterialTickV3::compose(
            self.foundation_digest,
            graph.report(),
            &self.material,
            &material,
        )?;
        Ok(PreparedMaterialTickV3 {
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
        prepared: PreparedMaterialTickV3<G>,
        commit: F,
    ) -> Result<(IdentifiedMaterialTickV3, ReplayCommitDispositionV1), MaterialCommitErrorV3<E>>
    where
        F: FnOnce(&IdentifiedMaterialTickV3) -> Result<ReplayCommitDispositionV1, E>,
    {
        if prepared.material.prior_digest() != self.material.digest()
            || prepared.identity.foundation_digest != self.foundation_digest
            || prepared.identity.resolve_tick != self.completed_tick().saturating_add(1)
        {
            return Err(MaterialCommitErrorV3::Preflight(
                MaterialReplayErrorV3::StaleCandidate,
            ));
        }
        let (_, disposition) = self
            .graph
            .commit_prepared_and_publish(sink, prepared.graph, |_| commit(&prepared.identity))
            .map_err(|error| match error {
                PreparedReplayCommitErrorV1::Preflight(error) => {
                    MaterialCommitErrorV3::Preflight(MaterialReplayErrorV3::Graph(error))
                }
                PreparedReplayCommitErrorV1::Commit(error) => MaterialCommitErrorV3::Commit(error),
            })?;
        self.material = prepared.material.into_register();
        Ok((prepared.identity, disposition))
    }
}

impl MaterialReplaySessionV3<babylon_graph::hypergraph_store::HypergraphStore> {
    /// Restore checked component checkpoint sections under the exact pinned foundation.
    /// # Errors
    /// Every decode, tick or graph restore refusal leaves both live owners unchanged.
    pub fn restore_full_checkpoint(
        &mut self,
        graph_state: &StableGraphStateV1,
        graph_material: &MaterialStateRowsV1,
        graph_registers: &[u8],
        material_bytes: &[u8],
    ) -> Result<(), MaterialReplayErrorV3> {
        let material = MaterialWorldRegisterV2::decode(material_bytes)?;
        let tick = material.completed_tick();
        if tick == 0 || tick > self.horizon {
            return Err(MaterialReplayErrorV3::Horizon);
        }
        let tick = i64::try_from(tick).map_err(|_| MaterialReplayErrorV3::Identity)?;
        self.graph
            .restore_full_checkpoint(tick, graph_state, graph_material, graph_registers)?;
        self.material = material;
        Ok(())
    }
}
