//! Database-free execution of fully identified replay ticks.

use std::collections::TryReserveError;

use babylon_bsl::identity_codec::{project_stable_value_v1, IdentityCodecError, StableBslValueV1};
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::allocator_state::AllocatorState;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::stable_element::{StableElementResolverV1, StableIdentityError};
use babylon_graph::stable_state::{
    encode_stable_graph_state_v1, StableGraphStateHashV1, StableGraphStateV1,
};
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::GraphSubstrate;
use babylon_graph::working_copy::DetachedCopy;
use babylon_kernel::replay::{ReplayIdentityError, ReplaySeed, ReplaySessionIdV1};
use babylon_kernel::tick_content_hash::{
    OrderedPracticeActionBatchDigestV1, PreparedEnvironmentDigestV1, RefDigestV1,
    TickContentHashError, TickContentHashV1, TickContentPartsV1, TickContentPreimageV1,
};
use babylon_kernel::ContentDigest;
use babylon_practice_contract::ordered_action_v1::{
    OrderedPracticeActionBatchV1, ORDERED_PRACTICE_ACTION_BATCH_V1_LAYOUT_VERSION,
};

use crate::choice_receipt::ChoiceReceiptRefV1;
use crate::committed_event::CommittedEventV2;
use crate::material_state::{
    MaterialAllocationGate, MaterialProjectionContextV1, MaterialStateErrorV1, MaterialStateRowsV1,
    MaterialStateV1, ProductionMaterialAllocationGate,
};
use crate::replay_identity::{
    encode_prepared_environment_v1, encode_stable_world_v1, encode_tick_payload_for_prepared_v2,
    encode_world_register_set_v1, world_register_manifest_v1, PreparedEnvironmentV1,
    ReplayTickIdentityError, StableWorldV1, TickPayloadV2, WorldRegisterManifestV1,
    WorldRegisterSetV1,
};
use crate::{prepare_rules, EventRecord, PreparedRules, TickReport};

/// A checked replay-session construction or tick refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayTickError {
    /// Scenario, declaration, rule, or schedule preparation failed.
    Preparation {
        /// Human-readable preparation refusal.
        message: String,
    },
    /// One detached adjudication or publication stage failed.
    Execution {
        /// Human-readable execution refusal.
        message: String,
    },
    /// The next completed tick could not be represented.
    TickCounterOverflow,
    /// A runtime action batch contained actions, which this gate forbids.
    NonEmptyActionBatch {
        /// Refused number of accepted actions.
        count: usize,
    },
    /// The action batch belonged to another replay session.
    ActionSessionMismatch,
    /// The action batch named a different resolve tick.
    ActionTickMismatch {
        /// Required next resolve tick.
        expected: u64,
        /// Tick carried by the supplied batch.
        actual: u64,
    },
    /// A tick-owned canonical identity refused its semantic input.
    Identity(ReplayTickIdentityError),
    /// Copying the exact replay-session namespace failed its own typed boundary.
    ReplaySessionIdentity(ReplayIdentityError),
    /// A successful event field could not be projected to stable identity.
    EventIdentity(IdentityCodecError),
    /// One successfully emitted event repeated a retained field name.
    DuplicateSuccessfulEventField {
        /// Exact emitted event type.
        event_type: String,
        /// Repeated field name.
        field: String,
    },
    /// Tick-owned material source identity or projection refused publication.
    MaterialState(MaterialStateErrorV1),
    /// Coupled physical closing or graph-owned staffing refused before publication.
    MaterialBase(crate::material_replay::MaterialBaseErrorV1),
    /// Stable graph or resolver identity refused the graph.
    Stable(StableIdentityError),
    /// The fixed outer preimage could not be composed.
    Outer(TickContentHashError),
    /// A bounded report buffer could not reserve its exact capacity.
    Allocation {
        /// Stable buffer name.
        field: &'static str,
        /// Exact requested capacity.
        requested: usize,
    },
    /// A deterministic test composer refused before publication.
    Composer {
        /// Stable injected refusal detail.
        message: String,
    },
    /// A prepared tick belongs to another replay session.
    PreparedSessionMismatch,
    /// Another prepared tick was published after this candidate was detached.
    StalePreparedTick {
        /// Completed tick observed when the candidate was prepared.
        prepared_after: i64,
        /// Current completed tick owned by the live session.
        live_completed: i64,
    },
    /// The durable acknowledgement named another resolve tick.
    CommitAcknowledgementTickMismatch {
        /// Resolve tick carried by the prepared report.
        expected: u64,
        /// Resolve tick carried by the acknowledgement.
        actual: u64,
    },
    /// The durable acknowledgement named another tick-content identity.
    CommitAcknowledgementHashMismatch {
        /// Tick-content hash carried by the prepared report.
        expected: [u8; 32],
        /// Tick-content hash carried by the acknowledgement.
        actual: [u8; 32],
    },
    /// A typed restart source did not reproduce one checkpoint section.
    CheckpointMismatch {
        /// Stable section name.
        section: &'static str,
    },
}

impl From<ReplayTickIdentityError> for ReplayTickError {
    fn from(value: ReplayTickIdentityError) -> Self {
        Self::Identity(value)
    }
}

impl From<IdentityCodecError> for ReplayTickError {
    fn from(value: IdentityCodecError) -> Self {
        Self::EventIdentity(value)
    }
}

impl From<StableIdentityError> for ReplayTickError {
    fn from(value: StableIdentityError) -> Self {
        Self::Stable(value)
    }
}

impl From<TickContentHashError> for ReplayTickError {
    fn from(value: TickContentHashError) -> Self {
        Self::Outer(value)
    }
}

impl std::fmt::Display for ReplayTickError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preparation { message } => {
                write!(formatter, "replay preparation failed: {message}")
            }
            Self::Execution { message } => write!(formatter, "replay execution failed: {message}"),
            Self::TickCounterOverflow => formatter.write_str("replay tick counter overflowed"),
            Self::NonEmptyActionBatch { count } => {
                write!(
                    formatter,
                    "replay action batch must be empty, got {count} actions"
                )
            }
            Self::ActionSessionMismatch => {
                formatter.write_str("replay action batch session does not match")
            }
            Self::ActionTickMismatch { expected, actual } => write!(
                formatter,
                "replay action batch tick mismatch: expected {expected}, got {actual}"
            ),
            Self::Identity(error) => write!(formatter, "replay identity refused: {error:?}"),
            Self::ReplaySessionIdentity(error) => {
                write!(formatter, "replay session identity refused: {error:?}")
            }
            Self::EventIdentity(error) => {
                write!(formatter, "replay event identity refused: {error:?}")
            }
            Self::DuplicateSuccessfulEventField { event_type, field } => write!(
                formatter,
                "replay event {event_type} repeated retained field {field}"
            ),
            Self::MaterialState(error) => write!(formatter, "material state refused: {error:?}"),
            Self::MaterialBase(error) => write!(formatter, "{error}"),
            Self::Stable(error) => write!(formatter, "stable identity refused: {error:?}"),
            Self::Outer(error) => write!(formatter, "tick-content preimage refused: {error:?}"),
            Self::Allocation { field, requested } => {
                write!(formatter, "{field} allocation of {requested} bytes failed")
            }
            Self::Composer { message } => write!(formatter, "replay composer refused: {message}"),
            Self::PreparedSessionMismatch => {
                formatter.write_str("prepared replay tick belongs to another session")
            }
            Self::StalePreparedTick {
                prepared_after,
                live_completed,
            } => write!(
                formatter,
                "prepared replay tick followed {prepared_after}, but the live session completed {live_completed}"
            ),
            Self::CommitAcknowledgementTickMismatch { expected, actual } => write!(
                formatter,
                "commit acknowledgement tick mismatch: expected {expected}, got {actual}"
            ),
            Self::CommitAcknowledgementHashMismatch { .. } => {
                formatter.write_str("commit acknowledgement tick-content hash mismatch")
            }
            Self::CheckpointMismatch { section } => {
                write!(formatter, "checkpoint {section} bytes do not match")
            }
        }
    }
}

impl std::error::Error for ReplayTickError {}

/// Closed durable dispositions that may publish one prepared replay tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayCommitDispositionV1 {
    /// The commit operation returned success directly.
    Committed,
    /// An ambiguous commit was reconnected and proven byte-exact.
    ReconciledAfterAmbiguousCommit,
}

/// A prepared replay tick refused before durability or its commit operation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparedReplayCommitErrorV1<E> {
    /// Candidate validation or event-publication capacity refused before the commit operation ran.
    Preflight(ReplayTickError),
    /// The caller-owned durable commit operation failed before publication.
    Commit(E),
}

/// Exact durable identity supplied before a prepared replay tick may publish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayCommitAcknowledgementV1 {
    disposition: ReplayCommitDispositionV1,
    resolve_tick: u64,
    tick_content_hash: TickContentHashV1,
}

impl ReplayCommitAcknowledgementV1 {
    /// Construct one typed acknowledgement from a durable commit outcome.
    #[must_use]
    pub const fn new(
        disposition: ReplayCommitDispositionV1,
        resolve_tick: u64,
        tick_content_hash: TickContentHashV1,
    ) -> Self {
        Self {
            disposition,
            resolve_tick,
            tick_content_hash,
        }
    }

    /// Return the exact durable disposition.
    #[must_use]
    pub const fn disposition(self) -> ReplayCommitDispositionV1 {
        self.disposition
    }

    /// Return the acknowledged resolve tick.
    #[must_use]
    pub const fn resolve_tick(self) -> u64 {
        self.resolve_tick
    }

    /// Return the acknowledged tick-content identity.
    #[must_use]
    pub const fn tick_content_hash(self) -> TickContentHashV1 {
        self.tick_content_hash
    }
}

/// One typed event retained from a successfully published replay tick.
#[derive(Debug, Clone, PartialEq)]
pub struct SuccessfulEventV2 {
    emitting_rule: String,
    choice_receipt: Option<ChoiceReceiptRefV1>,
    event_type: String,
    fields: Vec<(String, StableBslValueV1)>,
}

impl SuccessfulEventV2 {
    /// Borrow the rule that emitted this observation.
    #[must_use]
    pub fn emitting_rule(&self) -> &str {
        &self.emitting_rule
    }

    /// Return the adjacent realized-choice reference, when present.
    #[must_use]
    pub const fn choice_receipt(&self) -> Option<ChoiceReceiptRefV1> {
        self.choice_receipt
    }

    /// Borrow the declared event type.
    #[must_use]
    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    /// Borrow retained fields in strict UTF-8 field-name byte order.
    ///
    /// The live event sink preserves emitted order. Only this detached typed
    /// persistence projection is canonicalized, and duplicate names refuse the
    /// replay tick before publication.
    #[must_use]
    pub fn fields(&self) -> &[(String, StableBslValueV1)] {
        &self.fields
    }
}

/// Exact typed events from one successfully published replay tick.
#[derive(Debug, Clone, PartialEq)]
pub struct SuccessfulEventBatchV2 {
    events: Vec<SuccessfulEventV2>,
    source_digest: [u8; 32],
}

trait SuccessfulEventRetention {
    fn copy_string(&self, field: &'static str, source: &str) -> Result<String, ReplayTickError>;

    fn project_value(
        &self,
        value: &babylon_bsl::evaluator::Value,
        resolver: &StableElementResolverV1,
    ) -> Result<StableBslValueV1, ReplayTickError>;
}

struct ProductionSuccessfulEventRetention;

impl SuccessfulEventRetention for ProductionSuccessfulEventRetention {
    fn copy_string(&self, field: &'static str, source: &str) -> Result<String, ReplayTickError> {
        let mut output = String::new();
        output
            .try_reserve_exact(source.len())
            .map_err(|_: TryReserveError| ReplayTickError::Allocation {
                field,
                requested: source.len(),
            })?;
        output.push_str(source);
        Ok(output)
    }

    fn project_value(
        &self,
        value: &babylon_bsl::evaluator::Value,
        resolver: &StableElementResolverV1,
    ) -> Result<StableBslValueV1, ReplayTickError> {
        Ok(project_stable_value_v1(value, resolver)?)
    }
}

impl SuccessfulEventBatchV2 {
    /// Borrow retained events in executable order.
    #[must_use]
    pub fn events(&self) -> &[SuccessfulEventV2] {
        &self.events
    }

    /// Return SHA-256 of the exact V2 tick event-section bytes.
    #[must_use]
    pub const fn source_digest(&self) -> [u8; 32] {
        self.source_digest
    }

    fn try_from_committed_events<R: SuccessfulEventRetention>(
        source: &[CommittedEventV2],
        resolver: &StableElementResolverV1,
        retention: &R,
        source_digest: [u8; 32],
    ) -> Result<Self, ReplayTickError> {
        let mut events = Vec::new();
        events
            .try_reserve_exact(source.len())
            .map_err(|_: TryReserveError| ReplayTickError::Allocation {
                field: "successful event batch",
                requested: source.len(),
            })?;
        for event in source {
            let emitting_rule =
                retention.copy_string("successful event emitting rule", event.emitting_rule())?;
            let retained_event_type =
                retention.copy_string("successful event type", event.event_type())?;
            let mut retained_fields = Vec::new();
            retained_fields
                .try_reserve_exact(event.payload().len())
                .map_err(|_: TryReserveError| ReplayTickError::Allocation {
                    field: "successful event fields",
                    requested: event.payload().len(),
                })?;
            for (name, value) in event.payload() {
                retained_fields.push((
                    retention.copy_string("successful event field name", name)?,
                    retention.project_value(value, resolver)?,
                ));
            }
            retained_fields
                .sort_unstable_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
            if let Some(duplicate_index) = retained_fields
                .windows(2)
                .position(|adjacent| adjacent[0].0 == adjacent[1].0)
            {
                let (field, _) = retained_fields.remove(duplicate_index + 1);
                return Err(ReplayTickError::DuplicateSuccessfulEventField {
                    event_type: retained_event_type,
                    field,
                });
            }
            events.push(SuccessfulEventV2 {
                emitting_rule,
                choice_receipt: event.choice_receipt(),
                event_type: retained_event_type,
                fields: retained_fields,
            });
        }
        Ok(Self {
            events,
            source_digest,
        })
    }
}

/// Non-durable evidence returned by one successful identified replay tick.
#[derive(Debug)]
pub struct IdentifiedTickReportV2 {
    report: TickReport,
    action_batch_bytes: Vec<u8>,
    action_batch_layout_version: u32,
    action_batch_digest: OrderedPracticeActionBatchDigestV1,
    prior_registers: WorldRegisterSetV1,
    prior_world: StableWorldV1,
    result_registers: WorldRegisterSetV1,
    result_world: StableWorldV1,
    result_stable_graph: StableGraphStateV1,
    successful_event_batch: SuccessfulEventBatchV2,
    material_state_rows: MaterialStateRowsV1,
    resolver_manifest_bytes: Vec<u8>,
    prepared_environment_bytes: Vec<u8>,
    replay_session_identity: ReplaySessionIdV1,
    rng_seed: ReplaySeed,
    content_digest: ContentDigest,
    reference_digest: RefDigestV1,
    payload: TickPayloadV2,
    outer_preimage: TickContentPreimageV1,
    resolver_manifest_digest: [u8; 32],
    prepared_environment_digest: PreparedEnvironmentDigestV1,
    prior_stable_graph_digest: StableGraphStateHashV1,
    result_stable_graph_digest: StableGraphStateHashV1,
    tick_content_hash: TickContentHashV1,
}

impl IdentifiedTickReportV2 {
    /// Borrow the existing administrative tick evidence.
    #[must_use]
    pub const fn report(&self) -> &TickReport {
        &self.report
    }

    /// Consume the identified evidence and return its administrative tick report.
    ///
    /// The durable runtime uses this after acknowledgement when a bounded
    /// observation replaces the detailed persistence evidence.
    #[must_use]
    pub fn into_report(self) -> TickReport {
        self.report
    }

    /// Borrow the exact accepted-action batch bytes.
    #[must_use]
    pub fn action_batch_bytes(&self) -> &[u8] {
        &self.action_batch_bytes
    }

    /// Return the accepted-action batch layout version.
    #[must_use]
    pub const fn action_batch_layout_version(&self) -> u32 {
        self.action_batch_layout_version
    }

    /// Return the accepted-action batch digest.
    #[must_use]
    pub const fn action_batch_digest(&self) -> OrderedPracticeActionBatchDigestV1 {
        self.action_batch_digest
    }

    /// Borrow the exact prior register set.
    #[must_use]
    pub const fn prior_registers(&self) -> &WorldRegisterSetV1 {
        &self.prior_registers
    }

    /// Borrow the exact prior stable world.
    #[must_use]
    pub const fn prior_world(&self) -> &StableWorldV1 {
        &self.prior_world
    }

    /// Borrow the exact result register set.
    #[must_use]
    pub const fn result_registers(&self) -> &WorldRegisterSetV1 {
        &self.result_registers
    }

    /// Borrow the exact result stable world.
    #[must_use]
    pub const fn result_world(&self) -> &StableWorldV1 {
        &self.result_world
    }

    /// Borrow the exact typed result graph and its canonical identity bytes.
    #[must_use]
    pub const fn result_stable_graph(&self) -> &StableGraphStateV1 {
        &self.result_stable_graph
    }

    /// Borrow the exact typed events from the successful tick.
    #[must_use]
    pub const fn successful_event_batch(&self) -> &SuccessfulEventBatchV2 {
        &self.successful_event_batch
    }

    /// Return SHA-256 of the exact ordered choice-receipt payload section.
    #[must_use]
    pub const fn choice_receipt_source_digest(&self) -> [u8; 32] {
        self.payload.choice_receipt_section_digest()
    }

    /// Borrow the detached typed material projection for this completed tick.
    #[must_use]
    pub const fn material_state_rows(&self) -> &MaterialStateRowsV1 {
        &self.material_state_rows
    }

    /// Borrow the exact resolver manifest used to adjudicate this tick.
    #[must_use]
    pub fn resolver_manifest_bytes(&self) -> &[u8] {
        &self.resolver_manifest_bytes
    }

    /// Borrow the exact prepared environment used to adjudicate this tick.
    #[must_use]
    pub fn prepared_environment_bytes(&self) -> &[u8] {
        &self.prepared_environment_bytes
    }

    /// Borrow the exact replay-session namespace used by this tick.
    #[must_use]
    pub const fn replay_session_identity(&self) -> &ReplaySessionIdV1 {
        &self.replay_session_identity
    }

    /// Return the exact replay seed used by this tick.
    #[must_use]
    pub const fn rng_seed(&self) -> ReplaySeed {
        self.rng_seed
    }

    /// Borrow the immutable mechanics-content digest used by this tick.
    #[must_use]
    pub const fn content_digest(&self) -> &ContentDigest {
        &self.content_digest
    }

    /// Return the immutable reference-data digest used by this tick.
    #[must_use]
    pub const fn reference_digest(&self) -> RefDigestV1 {
        self.reference_digest
    }

    /// Borrow the exact governed tick payload.
    #[must_use]
    pub const fn payload(&self) -> &TickPayloadV2 {
        &self.payload
    }

    /// Borrow the exact fixed outer preimage.
    #[must_use]
    pub const fn outer_preimage(&self) -> &TickContentPreimageV1 {
        &self.outer_preimage
    }

    /// Return the sealed resolver-manifest digest.
    #[must_use]
    pub const fn resolver_manifest_digest(&self) -> [u8; 32] {
        self.resolver_manifest_digest
    }

    /// Return the prepared-environment digest.
    #[must_use]
    pub const fn prepared_environment_digest(&self) -> PreparedEnvironmentDigestV1 {
        self.prepared_environment_digest
    }

    /// Return the prior stable-graph digest.
    #[must_use]
    pub const fn prior_stable_graph_digest(&self) -> StableGraphStateHashV1 {
        self.prior_stable_graph_digest
    }

    /// Return the result stable-graph digest.
    #[must_use]
    pub const fn result_stable_graph_digest(&self) -> StableGraphStateHashV1 {
        self.result_stable_graph_digest
    }

    /// Return the authoritative tick-content identity.
    #[must_use]
    pub const fn tick_content_hash(&self) -> TickContentHashV1 {
        self.tick_content_hash
    }
}

/// One loaded replay environment advanced through the sealed V2 identity path.
pub struct ReplayTickSession<G> {
    graph: G,
    prepared: PreparedRules,
    completed_tick: i64,
    session: ReplaySessionIdV1,
    seed: ReplaySeed,
    content: ContentDigest,
    reference: RefDigestV1,
    resolver: StableElementResolverV1,
    register_manifest: WorldRegisterManifestV1,
    prepared_environment: PreparedEnvironmentV1,
    material_state: MaterialStateV1,
}

/// One fully adjudicated replay tick held outside every live session owner.
///
/// Dropping this value abandons the candidate. Only [`ReplayTickSession::acknowledge_prepared`]
/// or [`ReplayTickSession::commit_prepared_and_publish`] may move its graph, material state, event
/// batch, and tick counter into the live session.
pub struct PreparedReplayTickV1<G> {
    source_session: ReplaySessionIdV1,
    prepared_after: i64,
    resolve_tick: i64,
    graph: G,
    material_state: MaterialStateV1,
    events: Vec<EventRecord>,
    report: IdentifiedTickReportV2,
}

trait ReplayPublicationAllocationGate {
    fn reserve(&self, sink: &mut CollectingSink, additional: usize) -> Result<(), ReplayTickError>;
}

struct ProductionReplayPublicationAllocationGate;

impl ReplayPublicationAllocationGate for ProductionReplayPublicationAllocationGate {
    fn reserve(&self, sink: &mut CollectingSink, additional: usize) -> Result<(), ReplayTickError> {
        sink.events
            .try_reserve_exact(additional)
            .map_err(|_: TryReserveError| ReplayTickError::Allocation {
                field: "acknowledged replay event publication",
                requested: additional,
            })
    }
}

impl<G> PreparedReplayTickV1<G> {
    /// Borrow the sole identified report from which persistence rows derive.
    #[must_use]
    pub const fn report(&self) -> &IdentifiedTickReportV2 {
        &self.report
    }
}

impl<G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy> ReplayTickSession<G> {
    /// Load, verify, and seal one replay environment without executing a tick.
    ///
    /// # Errors
    /// Returns the first preparation, stable-identity, rules-hash, or bounded
    /// canonical-composition refusal.
    #[allow(
        clippy::too_many_arguments,
        reason = "the constructor makes every authoritative replay identity explicit"
    )]
    pub fn new(
        scenario_src: &str,
        prelude_src: Option<&str>,
        rule_src: &str,
        mut graph: G,
        session: ReplaySessionIdV1,
        seed: ReplaySeed,
        content: ContentDigest,
        reference: RefDigestV1,
        material_state: MaterialStateV1,
    ) -> Result<Self, ReplayTickError> {
        let expected_reference = material_state.reference_bundle_digest();
        let actual_reference = *reference.as_bytes();
        if actual_reference != expected_reference {
            return Err(ReplayTickError::MaterialState(
                MaterialStateErrorV1::ReferenceBundleMismatch {
                    expected: expected_reference,
                    actual: actual_reference,
                },
            ));
        }
        let prepared =
            prepare_rules(scenario_src, prelude_src, rule_src, &mut graph).map_err(|error| {
                ReplayTickError::Preparation {
                    message: error.to_string(),
                }
            })?;
        let resolver = StableElementResolverV1::seal(
            &graph,
            &prepared.scenario_scope,
            &prepared.node_content_ids,
            &prepared.hyperedge_content_ids,
        )?;
        let register_manifest = world_register_manifest_v1()?;
        let prepared_environment =
            encode_prepared_environment_v1(&content, &prepared, &resolver, &register_manifest)?;
        Ok(Self {
            graph,
            prepared,
            completed_tick: 0,
            session,
            seed,
            content,
            reference,
            resolver,
            register_manifest,
            prepared_environment,
            material_state,
        })
    }

    /// Admit the immutable prepared rules for native staffing ownership once.
    /// The normal effect scanner includes every possible guarded/loop write;
    /// scheduled labor does not call this additional composition check.
    pub(crate) fn validate_staffing_ownership(&self) -> Result<(), ReplayTickError> {
        use crate::material_replay::MaterialBaseErrorV1;
        use crate::material_staffing::STAFFING_FIELDS_V1;
        use babylon_bsl::causal_contract::{effect_footprint, EffectSignature};

        for (rule_id, loaded) in &self.prepared.rules {
            let effects = effect_footprint(&loaded.rule).map_err(|error| {
                ReplayTickError::MaterialBase(MaterialBaseErrorV1::StaffingEffectAnalysis {
                    rule_id: rule_id.clone(),
                    error,
                })
            })?;
            for effect in effects {
                if let EffectSignature::NodeField(field) = effect {
                    if STAFFING_FIELDS_V1.contains(&field.as_str()) {
                        return Err(ReplayTickError::MaterialBase(
                            MaterialBaseErrorV1::StaffingFieldOwner {
                                rule_id: rule_id.clone(),
                                field,
                            },
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Execute and atomically publish the next fully identified tick.
    ///
    /// # Errors
    /// Returns the first action, tick, execution, identity, stable-state,
    /// outer-hash, or event-reservation refusal without advancing the session.
    pub fn advance(
        &mut self,
        sink: &mut CollectingSink,
        actions: &OrderedPracticeActionBatchV1,
    ) -> Result<IdentifiedTickReportV2, ReplayTickError> {
        self.advance_with_boundaries(
            sink,
            actions,
            &ProductionReplayIdentityComposer,
            &ProductionMaterialAllocationGate,
        )
    }

    /// Adjudicate the next tick into detached owners without publishing it.
    ///
    /// # Errors
    /// Returns the same checked action, execution, identity, material, and
    /// allocation refusals as [`Self::advance`]. The live session and every
    /// caller-owned event sink remain untouched on both success and failure.
    pub fn prepare_advance(
        &self,
        actions: &OrderedPracticeActionBatchV1,
    ) -> Result<PreparedReplayTickV1<G>, ReplayTickError> {
        self.prepare_with_material(actions, None)
            .map(|(graph, _)| graph)
    }

    pub(crate) fn prepare_material_advance(
        &self,
        actions: &OrderedPracticeActionBatchV1,
        material: crate::material_replay::MaterialBaseInputs<'_>,
    ) -> Result<
        (
            PreparedReplayTickV1<G>,
            crate::material_world::PreparedMaterialWorldV3,
        ),
        ReplayTickError,
    > {
        let (graph, material) = self.prepare_with_material(actions, Some(material))?;
        let material = material.ok_or(ReplayTickError::MaterialBase(
            crate::material_replay::MaterialBaseErrorV1::MissingCandidate,
        ))?;
        Ok((graph, material))
    }

    fn prepare_with_material(
        &self,
        actions: &OrderedPracticeActionBatchV1,
        material: Option<crate::material_replay::MaterialBaseInputs<'_>>,
    ) -> Result<
        (
            PreparedReplayTickV1<G>,
            Option<crate::material_world::PreparedMaterialWorldV3>,
        ),
        ReplayTickError,
    > {
        let next_tick = self
            .completed_tick
            .checked_add(1)
            .ok_or(ReplayTickError::TickCounterOverflow)?;
        validate_replay_action_batch(&self.session, actions, next_tick)?;
        let candidate_material = self
            .material_state
            .try_detached(&ProductionMaterialAllocationGate)
            .map_err(replay_material_error)?;
        let mut candidate_graph = self.graph.detached_copy();
        let mut candidate_sink = CollectingSink::default();
        let execution = ReplayExecutionInputs {
            session: &self.session,
            seed: self.seed,
            content: &self.content,
            reference: self.reference,
            resolver: &self.resolver,
            register_manifest: &self.register_manifest,
            prepared_environment: &self.prepared_environment,
            actions,
            composer: &ProductionReplayIdentityComposer,
            material_state: &candidate_material,
            material_allocation: &ProductionMaterialAllocationGate,
        };
        let (report, material) = crate::run_prepared_replay_tick(
            &self.prepared,
            &mut candidate_graph,
            &mut candidate_sink,
            next_tick,
            execution,
            material,
        )?;
        Ok((
            PreparedReplayTickV1 {
                source_session: self.session.clone(),
                prepared_after: self.completed_tick,
                resolve_tick: next_tick,
                graph: candidate_graph,
                material_state: candidate_material,
                events: candidate_sink.events,
                report,
            },
            material,
        ))
    }

    /// Publish one detached candidate after exact durable acknowledgement.
    ///
    /// # Errors
    /// Refuses a foreign or stale candidate, a mismatched acknowledgement, or
    /// event capacity that cannot be reserved. Every refusal leaves the live
    /// graph, material state, tick counter, allocator cursors, and sink intact.
    pub fn acknowledge_prepared(
        &mut self,
        sink: &mut CollectingSink,
        prepared: PreparedReplayTickV1<G>,
        acknowledgement: ReplayCommitAcknowledgementV1,
    ) -> Result<IdentifiedTickReportV2, ReplayTickError> {
        let resolve_tick = self.validate_prepared_publication(&prepared)?;
        if acknowledgement.resolve_tick != resolve_tick {
            return Err(ReplayTickError::CommitAcknowledgementTickMismatch {
                expected: resolve_tick,
                actual: acknowledgement.resolve_tick,
            });
        }
        let expected_hash = *prepared.report.tick_content_hash().as_bytes();
        let actual_hash = *acknowledgement.tick_content_hash.as_bytes();
        if actual_hash != expected_hash {
            return Err(ReplayTickError::CommitAcknowledgementHashMismatch {
                expected: expected_hash,
                actual: actual_hash,
            });
        }
        ProductionReplayPublicationAllocationGate.reserve(sink, prepared.events.len())?;
        Ok(self.publish_prepared_infallibly(sink, prepared))
    }

    /// Preflight one detached candidate, run its durable commit operation, then publish it.
    ///
    /// Candidate identity, session freshness, tick conversion, and exact event-buffer capacity are
    /// checked before `commit` runs. A successful commit closure is followed only by moves into the
    /// already-reserved live owners, so durable success has no fallible acknowledgement tail.
    ///
    /// # Errors
    /// Returns [`PreparedReplayCommitErrorV1::Preflight`] without invoking `commit` when candidate
    /// validation or event reservation refuses. Returns [`PreparedReplayCommitErrorV1::Commit`]
    /// without publishing when the caller-owned durable operation refuses.
    pub fn commit_prepared_and_publish<E, F>(
        &mut self,
        sink: &mut CollectingSink,
        prepared: PreparedReplayTickV1<G>,
        commit: F,
    ) -> Result<(IdentifiedTickReportV2, ReplayCommitDispositionV1), PreparedReplayCommitErrorV1<E>>
    where
        F: FnOnce(&IdentifiedTickReportV2) -> Result<ReplayCommitDispositionV1, E>,
    {
        self.commit_prepared_and_publish_with_allocation(
            sink,
            prepared,
            commit,
            &ProductionReplayPublicationAllocationGate,
        )
    }

    fn commit_prepared_and_publish_with_allocation<E, F, A>(
        &mut self,
        sink: &mut CollectingSink,
        prepared: PreparedReplayTickV1<G>,
        commit: F,
        allocation: &A,
    ) -> Result<(IdentifiedTickReportV2, ReplayCommitDispositionV1), PreparedReplayCommitErrorV1<E>>
    where
        F: FnOnce(&IdentifiedTickReportV2) -> Result<ReplayCommitDispositionV1, E>,
        A: ReplayPublicationAllocationGate,
    {
        self.validate_prepared_publication(&prepared)
            .map_err(PreparedReplayCommitErrorV1::Preflight)?;
        allocation
            .reserve(sink, prepared.events.len())
            .map_err(PreparedReplayCommitErrorV1::Preflight)?;
        let disposition = commit(&prepared.report).map_err(PreparedReplayCommitErrorV1::Commit)?;
        let report = self.publish_prepared_infallibly(sink, prepared);
        Ok((report, disposition))
    }

    fn validate_prepared_publication(
        &self,
        prepared: &PreparedReplayTickV1<G>,
    ) -> Result<u64, ReplayTickError> {
        if self.session != prepared.source_session {
            return Err(ReplayTickError::PreparedSessionMismatch);
        }
        if self.completed_tick != prepared.prepared_after {
            return Err(ReplayTickError::StalePreparedTick {
                prepared_after: prepared.prepared_after,
                live_completed: self.completed_tick,
            });
        }
        u64::try_from(prepared.resolve_tick).map_err(|_| ReplayTickError::TickCounterOverflow)
    }

    fn publish_prepared_infallibly(
        &mut self,
        sink: &mut CollectingSink,
        prepared: PreparedReplayTickV1<G>,
    ) -> IdentifiedTickReportV2 {
        self.graph = prepared.graph;
        self.material_state = prepared.material_state;
        self.completed_tick = prepared.resolve_tick;
        sink.events.extend(prepared.events);
        prepared.report
    }

    #[cfg(test)]
    fn advance_with_composer<C: ReplayIdentityComposer>(
        &mut self,
        sink: &mut CollectingSink,
        actions: &OrderedPracticeActionBatchV1,
        composer: &C,
    ) -> Result<IdentifiedTickReportV2, ReplayTickError> {
        self.advance_with_boundaries(sink, actions, composer, &ProductionMaterialAllocationGate)
    }

    #[cfg(test)]
    fn advance_with_material_allocation(
        &mut self,
        sink: &mut CollectingSink,
        actions: &OrderedPracticeActionBatchV1,
        material_allocation: &dyn MaterialAllocationGate,
    ) -> Result<IdentifiedTickReportV2, ReplayTickError> {
        self.advance_with_boundaries(
            sink,
            actions,
            &ProductionReplayIdentityComposer,
            material_allocation,
        )
    }

    fn advance_with_boundaries<C: ReplayIdentityComposer>(
        &mut self,
        sink: &mut CollectingSink,
        actions: &OrderedPracticeActionBatchV1,
        composer: &C,
        material_allocation: &dyn MaterialAllocationGate,
    ) -> Result<IdentifiedTickReportV2, ReplayTickError> {
        let next_tick = self
            .completed_tick
            .checked_add(1)
            .ok_or(ReplayTickError::TickCounterOverflow)?;
        validate_replay_action_batch(&self.session, actions, next_tick)?;
        let candidate_material = self
            .material_state
            .try_detached(material_allocation)
            .map_err(replay_material_error)?;
        let execution = ReplayExecutionInputs {
            session: &self.session,
            seed: self.seed,
            content: &self.content,
            reference: self.reference,
            resolver: &self.resolver,
            register_manifest: &self.register_manifest,
            prepared_environment: &self.prepared_environment,
            actions,
            composer,
            material_state: &candidate_material,
            material_allocation,
        };
        let (identified, _) = crate::run_prepared_replay_tick(
            &self.prepared,
            &mut self.graph,
            sink,
            next_tick,
            execution,
            None,
        )?;
        self.material_state = candidate_material;
        self.completed_tick = next_tick;
        Ok(identified)
    }

    /// Return the number of fully published ticks.
    #[must_use]
    pub const fn completed_tick(&self) -> i64 {
        self.completed_tick
    }

    /// Borrow the held graph.
    #[must_use]
    pub const fn graph(&self) -> &G {
        &self.graph
    }

    /// Borrow the session-owned material sources.
    #[must_use]
    pub const fn material_state(&self) -> &MaterialStateV1 {
        &self.material_state
    }

    /// Recompose the current graph through the session's sealed resolver.
    ///
    /// # Errors
    /// Returns the first stable identity, topology, bound, or allocation
    /// refusal without mutating the replay session.
    pub fn stable_graph_state(&self) -> Result<StableGraphStateV1, ReplayTickError> {
        encode_stable_graph_state_v1(&self.graph, &self.resolver).map_err(Into::into)
    }

    /// Recompose the current completed-tick world-register set.
    ///
    /// # Errors
    /// Returns the first completed-tick, bound, or allocation refusal without
    /// mutating the replay session.
    pub fn world_registers(&self) -> Result<WorldRegisterSetV1, ReplayTickError> {
        encode_world_register_set_v1(&self.register_manifest, self.completed_tick)
            .map_err(Into::into)
    }

    /// Borrow the exact replay-session namespace.
    #[must_use]
    pub const fn session_identity(&self) -> &ReplaySessionIdV1 {
        &self.session
    }

    /// Return the exact replay seed.
    #[must_use]
    pub const fn rng_seed(&self) -> ReplaySeed {
        self.seed
    }

    /// Borrow the immutable mechanics-content digest.
    #[must_use]
    pub const fn content_digest(&self) -> &ContentDigest {
        &self.content
    }

    /// Return the immutable reference-data digest.
    #[must_use]
    pub const fn reference_digest(&self) -> RefDigestV1 {
        self.reference
    }

    /// Borrow the session-owned resolver-manifest bytes.
    #[must_use]
    pub fn resolver_manifest_bytes(&self) -> &[u8] {
        self.resolver.manifest().canonical_bytes()
    }

    /// Borrow the session-owned register-manifest bytes.
    #[must_use]
    pub fn register_manifest_bytes(&self) -> &[u8] {
        self.register_manifest.canonical_bytes()
    }

    /// Borrow the session-owned prepared-environment bytes.
    #[must_use]
    pub fn prepared_environment_bytes(&self) -> &[u8] {
        self.prepared_environment.canonical_bytes()
    }
}

impl ReplayTickSession<HypergraphStore> {
    /// Restore one already-prepared session from a complete typed checkpoint.
    ///
    /// Scenario and rule preparation still constructs the sealed topology and
    /// execution environment. This method replaces only checkpoint-owned graph
    /// values, material values, and the completed tick after reproducing the
    /// exact graph, register, and aggregate material section bytes.
    ///
    /// # Errors
    /// Returns the first tick, topology, material, or exact-section mismatch
    /// without changing the live session.
    pub fn restore_full_checkpoint(
        &mut self,
        completed_tick: i64,
        graph_state: &StableGraphStateV1,
        material_rows: &MaterialStateRowsV1,
        expected_world_registers: &[u8],
    ) -> Result<(), ReplayTickError> {
        if completed_tick <= 0 {
            return Err(ReplayTickError::TickCounterOverflow);
        }
        let mut graph = self.graph.detached_copy();
        graph.restore_stable_state_v1(&self.resolver, graph_state)?;
        let restored_graph = encode_stable_graph_state_v1(&graph, &self.resolver)?;
        if restored_graph.canonical_bytes() != graph_state.canonical_bytes() {
            return Err(ReplayTickError::CheckpointMismatch {
                section: "stable graph",
            });
        }
        let registers = encode_world_register_set_v1(&self.register_manifest, completed_tick)?;
        if registers.canonical_bytes() != expected_world_registers {
            return Err(ReplayTickError::CheckpointMismatch {
                section: "world registers",
            });
        }
        let material_state = self
            .material_state
            .try_restore_from_rows(material_rows)
            .map_err(ReplayTickError::MaterialState)?;
        let projected_material = material_state
            .project_rows(
                completed_tick,
                &MaterialProjectionContextV1::new(
                    &restored_graph,
                    &self.prepared.scenario_scope,
                    &self.prepared.types,
                    &self.prepared.enums,
                    &self.resolver,
                    &ProductionMaterialAllocationGate,
                ),
            )
            .map_err(ReplayTickError::MaterialState)?;
        if projected_material.canonical_bytes() != material_rows.canonical_bytes() {
            return Err(ReplayTickError::CheckpointMismatch {
                section: "semantic state",
            });
        }
        self.graph = graph;
        self.material_state = material_state;
        self.completed_tick = completed_tick;
        Ok(())
    }
}

pub(crate) struct ReplayPriorIdentityV1 {
    registers: WorldRegisterSetV1,
    world: StableWorldV1,
    stable_graph_digest: StableGraphStateHashV1,
}

pub(crate) struct ReplayExecutionInputs<'a, C> {
    pub(crate) session: &'a ReplaySessionIdV1,
    pub(crate) seed: ReplaySeed,
    pub(crate) content: &'a ContentDigest,
    pub(crate) reference: RefDigestV1,
    pub(crate) resolver: &'a StableElementResolverV1,
    pub(crate) register_manifest: &'a WorldRegisterManifestV1,
    pub(crate) prepared_environment: &'a PreparedEnvironmentV1,
    pub(crate) actions: &'a OrderedPracticeActionBatchV1,
    pub(crate) composer: &'a C,
    pub(crate) material_state: &'a MaterialStateV1,
    pub(crate) material_allocation: &'a dyn MaterialAllocationGate,
}

pub(crate) struct ReplayIdentityInputs<'a, G, C> {
    pub(crate) execution: &'a ReplayExecutionInputs<'a, C>,
    pub(crate) prepared: &'a PreparedRules,
    pub(crate) prior: ReplayPriorIdentityV1,
    pub(crate) result_graph: &'a G,
    pub(crate) report: &'a TickReport,
    pub(crate) events: &'a [EventRecord],
    pub(crate) resolve_tick: i64,
}

pub(crate) trait ReplayIdentityComposer {
    fn compose<G: CanonicalState>(
        &self,
        inputs: ReplayIdentityInputs<'_, G, Self>,
    ) -> Result<ReplayIdentityArtifactsV2, ReplayTickError>
    where
        Self: Sized;
}

pub(crate) struct ProductionReplayIdentityComposer;

impl ReplayIdentityComposer for ProductionReplayIdentityComposer {
    fn compose<G: CanonicalState>(
        &self,
        inputs: ReplayIdentityInputs<'_, G, Self>,
    ) -> Result<ReplayIdentityArtifactsV2, ReplayTickError> {
        compose_replay_identity_with_retention(inputs, &ProductionSuccessfulEventRetention)
    }
}

pub(crate) struct ReplayIdentityArtifactsV2 {
    action_batch_bytes: Vec<u8>,
    action_batch_layout_version: u32,
    action_batch_digest: OrderedPracticeActionBatchDigestV1,
    prior_registers: WorldRegisterSetV1,
    prior_world: StableWorldV1,
    result_registers: WorldRegisterSetV1,
    result_world: StableWorldV1,
    result_stable_graph: StableGraphStateV1,
    successful_event_batch: SuccessfulEventBatchV2,
    material_state_rows: MaterialStateRowsV1,
    resolver_manifest_bytes: Vec<u8>,
    prepared_environment_bytes: Vec<u8>,
    replay_session_identity: ReplaySessionIdV1,
    rng_seed: ReplaySeed,
    content_digest: ContentDigest,
    reference_digest: RefDigestV1,
    payload: TickPayloadV2,
    outer_preimage: TickContentPreimageV1,
    resolver_manifest_digest: [u8; 32],
    prepared_environment_digest: PreparedEnvironmentDigestV1,
    prior_stable_graph_digest: StableGraphStateHashV1,
    result_stable_graph_digest: StableGraphStateHashV1,
    tick_content_hash: TickContentHashV1,
}

pub(crate) fn validate_replay_actions<C>(
    execution: &ReplayExecutionInputs<'_, C>,
    resolve_tick: i64,
) -> Result<u64, ReplayTickError> {
    validate_replay_action_batch(execution.session, execution.actions, resolve_tick)
}

fn validate_replay_action_batch(
    session: &ReplaySessionIdV1,
    actions: &OrderedPracticeActionBatchV1,
    resolve_tick: i64,
) -> Result<u64, ReplayTickError> {
    let resolve_tick =
        u64::try_from(resolve_tick).map_err(|_| ReplayTickError::TickCounterOverflow)?;
    if !actions.is_empty() {
        return Err(ReplayTickError::NonEmptyActionBatch {
            count: actions.items().len(),
        });
    }
    if actions.session() != session {
        return Err(ReplayTickError::ActionSessionMismatch);
    }
    if actions.resolve_tick() != resolve_tick {
        return Err(ReplayTickError::ActionTickMismatch {
            expected: resolve_tick,
            actual: actions.resolve_tick(),
        });
    }
    Ok(resolve_tick)
}

pub(crate) fn compose_replay_prior<G: CanonicalState, C>(
    graph: &G,
    execution: &ReplayExecutionInputs<'_, C>,
    completed_tick: i64,
) -> Result<ReplayPriorIdentityV1, ReplayTickError> {
    let stable_graph = encode_stable_graph_state_v1(graph, execution.resolver)?;
    let stable_graph_digest = stable_graph.digest();
    let registers = encode_world_register_set_v1(execution.register_manifest, completed_tick)?;
    let world = encode_stable_world_v1(&stable_graph, &registers)?;
    drop(stable_graph);
    Ok(ReplayPriorIdentityV1 {
        registers,
        world,
        stable_graph_digest,
    })
}

fn compose_replay_identity_with_retention<G: CanonicalState, C, R: SuccessfulEventRetention>(
    inputs: ReplayIdentityInputs<'_, G, C>,
    retention: &R,
) -> Result<ReplayIdentityArtifactsV2, ReplayTickError> {
    let result_graph =
        encode_stable_graph_state_v1(inputs.result_graph, inputs.execution.resolver)?;
    let result_stable_graph_digest = result_graph.digest();
    let result_registers =
        encode_world_register_set_v1(inputs.execution.register_manifest, inputs.resolve_tick)?;
    let result_world = encode_stable_world_v1(&result_graph, &result_registers)?;
    validate_committed_event_sink(inputs.report, inputs.events)?;
    let payload = encode_tick_payload_for_prepared_v2(
        inputs.prepared,
        &inputs.report.per_rule_fired,
        inputs.report.fired,
        &inputs.report.committed_events,
        &inputs.report.choice_receipts,
        &inputs.report.audit_receipts,
        inputs.execution.resolver,
    )?;
    let successful_event_batch = SuccessfulEventBatchV2::try_from_committed_events(
        &inputs.report.committed_events,
        inputs.execution.resolver,
        retention,
        payload.event_section_digest(),
    )?;
    let material_state_rows = inputs
        .execution
        .material_state
        .project_rows(
            inputs.resolve_tick,
            &MaterialProjectionContextV1::new(
                &result_graph,
                &inputs.prepared.scenario_scope,
                &inputs.prepared.types,
                &inputs.prepared.enums,
                inputs.execution.resolver,
                inputs.execution.material_allocation,
            ),
        )
        .map_err(replay_material_error)?;
    let outer_preimage = compose_outer_preimage(&inputs, &result_world, &payload)?;
    let tick_content_hash = outer_preimage.digest();
    let action_batch_bytes = copy_action_bytes(inputs.execution.actions.canonical_bytes())?;
    let resolver_manifest_bytes = copy_report_bytes(
        inputs.execution.resolver.manifest().canonical_bytes(),
        "identified resolver manifest bytes",
    )?;
    let prepared_environment_bytes = copy_report_bytes(
        inputs.execution.prepared_environment.canonical_bytes(),
        "identified prepared environment bytes",
    )?;
    let replay_session_identity = ReplaySessionIdV1::try_from(inputs.execution.session.as_bytes())
        .map_err(ReplayTickError::ReplaySessionIdentity)?;
    Ok(ReplayIdentityArtifactsV2 {
        action_batch_bytes,
        action_batch_layout_version: ORDERED_PRACTICE_ACTION_BATCH_V1_LAYOUT_VERSION,
        action_batch_digest: inputs.execution.actions.digest(),
        prior_registers: inputs.prior.registers,
        prior_world: inputs.prior.world,
        result_registers,
        result_world,
        result_stable_graph: result_graph,
        successful_event_batch,
        material_state_rows,
        resolver_manifest_bytes,
        prepared_environment_bytes,
        replay_session_identity,
        rng_seed: inputs.execution.seed,
        content_digest: inputs.execution.content.clone(),
        reference_digest: inputs.execution.reference,
        payload,
        outer_preimage,
        resolver_manifest_digest: inputs.execution.resolver.manifest().digest(),
        prepared_environment_digest: inputs.execution.prepared_environment.digest(),
        prior_stable_graph_digest: inputs.prior.stable_graph_digest,
        result_stable_graph_digest,
        tick_content_hash,
    })
}

fn validate_committed_event_sink(
    report: &TickReport,
    sink_events: &[EventRecord],
) -> Result<(), ReplayTickError> {
    if report.committed_events.len() != sink_events.len() {
        return Err(ReplayTickIdentityError::CommittedEventSinkMismatch {
            ordinal: report.committed_events.len().min(sink_events.len()),
        }
        .into());
    }
    for (ordinal, (committed, sink)) in report.committed_events.iter().zip(sink_events).enumerate()
    {
        if committed.event_type() != sink.0 || committed.payload() != sink.1 {
            return Err(ReplayTickIdentityError::CommittedEventSinkMismatch { ordinal }.into());
        }
    }
    Ok(())
}

fn compose_outer_preimage<G, C>(
    inputs: &ReplayIdentityInputs<'_, G, C>,
    result_world: &StableWorldV1,
    payload: &TickPayloadV2,
) -> Result<TickContentPreimageV1, ReplayTickError> {
    let resolve_tick =
        u64::try_from(inputs.resolve_tick).map_err(|_| ReplayTickError::TickCounterOverflow)?;
    Ok(TickContentPreimageV1::compose(&TickContentPartsV1 {
        session: inputs.execution.session,
        resolve_tick,
        seed: inputs.execution.seed,
        content: inputs.execution.content,
        reference: inputs.execution.reference,
        prepared: inputs.execution.prepared_environment.digest(),
        prior_world: inputs.prior.world.digest(),
        actions: inputs.execution.actions.digest(),
        result_world: result_world.digest(),
        payload: payload.digest(),
    })?)
}

fn copy_action_bytes(source: &[u8]) -> Result<Vec<u8>, ReplayTickError> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(source.len())
        .map_err(|_: TryReserveError| ReplayTickError::Allocation {
            field: "identified action batch bytes",
            requested: source.len(),
        })?;
    bytes.extend_from_slice(source);
    Ok(bytes)
}

fn copy_report_bytes(source: &[u8], field: &'static str) -> Result<Vec<u8>, ReplayTickError> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(source.len())
        .map_err(|_: TryReserveError| ReplayTickError::Allocation {
            field,
            requested: source.len(),
        })?;
    bytes.extend_from_slice(source);
    Ok(bytes)
}

pub(crate) fn identified_report(
    report: TickReport,
    artifacts: ReplayIdentityArtifactsV2,
) -> IdentifiedTickReportV2 {
    IdentifiedTickReportV2 {
        report,
        action_batch_bytes: artifacts.action_batch_bytes,
        action_batch_layout_version: artifacts.action_batch_layout_version,
        action_batch_digest: artifacts.action_batch_digest,
        prior_registers: artifacts.prior_registers,
        prior_world: artifacts.prior_world,
        result_registers: artifacts.result_registers,
        result_world: artifacts.result_world,
        result_stable_graph: artifacts.result_stable_graph,
        successful_event_batch: artifacts.successful_event_batch,
        material_state_rows: artifacts.material_state_rows,
        resolver_manifest_bytes: artifacts.resolver_manifest_bytes,
        prepared_environment_bytes: artifacts.prepared_environment_bytes,
        replay_session_identity: artifacts.replay_session_identity,
        rng_seed: artifacts.rng_seed,
        content_digest: artifacts.content_digest,
        reference_digest: artifacts.reference_digest,
        payload: artifacts.payload,
        outer_preimage: artifacts.outer_preimage,
        resolver_manifest_digest: artifacts.resolver_manifest_digest,
        prepared_environment_digest: artifacts.prepared_environment_digest,
        prior_stable_graph_digest: artifacts.prior_stable_graph_digest,
        result_stable_graph_digest: artifacts.result_stable_graph_digest,
        tick_content_hash: artifacts.tick_content_hash,
    }
}

fn replay_material_error(error: MaterialStateErrorV1) -> ReplayTickError {
    match error {
        MaterialStateErrorV1::Allocation { field, requested } => {
            ReplayTickError::Allocation { field, requested }
        }
        other => ReplayTickError::MaterialState(other),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use babylon_bsl::rule_pipeline::split_content;
    use babylon_bsl::rules_hash_of;
    use babylon_graph::allocator_state::AllocatorState;
    use babylon_graph::memory::MemoryGraph;
    use babylon_graph::state_hash::CanonicalState;
    use babylon_graph::substrate::GraphSubstrate;
    use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
    use babylon_kernel::tick_content_hash::RefDigestV1;
    use babylon_kernel::{ContentDigest, H3CellId};
    use babylon_practice_contract::actor_v2::ActorOrganizationIdV2;
    use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
    use babylon_practice_contract::{
        input_authority_ledger_v2_digest, CampaignIdV2, InputAuthorityIdV2,
        PracticeAuthorityKindV2, PracticeIdV2, PracticeInputAuthorityLedgerV2,
        PracticeInputAuthorityV2, PracticeIntentV2, PracticeTargetIdentityV2, PracticeTargetTagV2,
        ProposalNonceV2, ResolvedPracticeBatchItemV2, ResolvedPracticeBatchV2,
        TaggedPracticeTargetV2,
    };

    use super::{
        PreparedReplayCommitErrorV1, ProductionSuccessfulEventRetention, ReplayCommitDispositionV1,
        ReplayExecutionInputs, ReplayIdentityArtifactsV2, ReplayIdentityComposer,
        ReplayIdentityInputs, ReplayPublicationAllocationGate, ReplayTickError, ReplayTickSession,
        SuccessfulEventRetention,
    };
    use crate::h3_runtime::{
        MichiganDynamicHexValueBitsV1, MICHIGAN_DYNAMIC_HEX_REFERENCE_BUNDLE_DIGEST_V1,
    };
    use crate::material_state::{
        MaterialAllocationGate, MaterialStateErrorV1, MaterialStateV1,
        ProductionMaterialAllocationGate,
    };
    use crate::{EventRecord, PreparedEventBatchSink};

    const SCENARIO: &str = include_str!("../content/scenarios/two-classes.bscn");
    const RULE: &str = include_str!("../content/rules/fundamental-theorem.bsl");
    const EVENT_RULE: &str = r#"
(rule economics/retention-allocation
  :role mechanic
  :evidence derived
  :material-basis "retention allocation atomicity fixture"
  :fuel 16
  (bindings
    (binding wages :field social-class/wages))
  (when #t)
  (effects
    (update-node self social-class/wages (add 1))
    (emit EventType/RETENTION_ALLOCATION (subject self))))
"#;
    const MATERIAL_ATOMIC_SCENARIO: &str = r"
(scenario test/material-atomicity
  (defenum OrgKind (POLITICAL_FACTION))
  (deffield social-class/wages int extensive)
  (deffield territory/population int extensive)
  (deffield organization/kind enum OrgKind)
  (deffield organization/members int extensive)
  (node workers NodeType/SOCIAL_CLASS
    (social-class/wages 10))
  (node territory-a NodeType/TERRITORY
    (territory/population 11))
  (node org-a NodeType/ORGANIZATION
    (organization/kind OrgKind/POLITICAL_FACTION)
    (organization/members 31))
  (edge EdgeType/PRESENCE org-a territory-a 1))
";

    fn session() -> ReplayTickSession<MemoryGraph> {
        session_for_rule(RULE)
    }

    fn session_for_rule(rule: &str) -> ReplayTickSession<MemoryGraph> {
        let replay = ReplaySessionIdV1::try_from("per60/atomic").unwrap();
        let (_, rules) = split_content(rule).unwrap();
        let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
        ReplayTickSession::new(
            SCENARIO,
            None,
            rule,
            MemoryGraph::new(),
            replay,
            ReplaySeed::new(17),
            ContentDigest {
                defines_hash: [0x31; 32],
                rules_hash: rules_hash_of(&forms).unwrap(),
            },
            RefDigestV1::from_bytes(MICHIGAN_DYNAMIC_HEX_REFERENCE_BUNDLE_DIGEST_V1),
            dynamic_fixture_material_state(),
        )
        .unwrap()
    }

    fn dynamic_fixture_material_state() -> MaterialStateV1 {
        dynamic_allocation_material_state()
    }

    fn material_state() -> MaterialStateV1 {
        dynamic_allocation_material_state()
    }

    fn dynamic_allocation_material_state() -> MaterialStateV1 {
        MaterialStateV1::try_dynamic_runtime_fixture_for_test(vec![(
            H3CellId::try_from(0x0872_6648_00ff_ffff_u64).unwrap(),
            MichiganDynamicHexValueBitsV1 {
                c: 1.0_f64.to_bits(),
                v: 2.0_f64.to_bits(),
                s: 3.0_f64.to_bits(),
                k: 4.0_f64.to_bits(),
                biocapacity_stock: 5.0_f64.to_bits(),
                energy_stock: 6.0_f64.to_bits(),
                raw_material_stock: 7.0_f64.to_bits(),
                internet_access_pct: 0.5_f64.to_bits(),
                surveillance_coupling: 0.25_f64.to_bits(),
            },
        )])
        .unwrap()
    }

    fn dynamic_allocation_session() -> ReplayTickSession<MemoryGraph> {
        let replay = ReplaySessionIdV1::try_from("per281/dynamic-allocation").unwrap();
        let (_, rules) = split_content(EVENT_RULE).unwrap();
        let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
        ReplayTickSession::new(
            MATERIAL_ATOMIC_SCENARIO,
            None,
            EVENT_RULE,
            MemoryGraph::new(),
            replay,
            ReplaySeed::new(19),
            ContentDigest {
                defines_hash: [0x51; 32],
                rules_hash: rules_hash_of(&forms).unwrap(),
            },
            RefDigestV1::from_bytes(MICHIGAN_DYNAMIC_HEX_REFERENCE_BUNDLE_DIGEST_V1),
            dynamic_allocation_material_state(),
        )
        .unwrap()
    }

    fn material_session() -> ReplayTickSession<MemoryGraph> {
        material_session_for_scenario(MATERIAL_ATOMIC_SCENARIO)
    }

    fn material_session_for_scenario(scenario: &str) -> ReplayTickSession<MemoryGraph> {
        let replay = ReplaySessionIdV1::try_from("per281/material-allocation").unwrap();
        let (_, rules) = split_content(EVENT_RULE).unwrap();
        let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
        ReplayTickSession::new(
            scenario,
            None,
            EVENT_RULE,
            MemoryGraph::new(),
            replay,
            ReplaySeed::new(19),
            ContentDigest {
                defines_hash: [0x51; 32],
                rules_hash: rules_hash_of(&forms).unwrap(),
            },
            RefDigestV1::from_bytes(MICHIGAN_DYNAMIC_HEX_REFERENCE_BUNDLE_DIGEST_V1),
            material_state(),
        )
        .unwrap()
    }

    struct RefusingComposer;

    struct RefusingPublicationAllocation;

    impl ReplayPublicationAllocationGate for RefusingPublicationAllocation {
        fn reserve(
            &self,
            _sink: &mut babylon_bsl::structural_verbs::CollectingSink,
            additional: usize,
        ) -> Result<(), ReplayTickError> {
            Err(ReplayTickError::Allocation {
                field: "injected acknowledged replay event publication",
                requested: additional,
            })
        }
    }

    impl ReplayIdentityComposer for RefusingComposer {
        fn compose<G: CanonicalState>(
            &self,
            _inputs: ReplayIdentityInputs<'_, G, Self>,
        ) -> Result<ReplayIdentityArtifactsV2, ReplayTickError> {
            Err(ReplayTickError::Composer {
                message: "injected identity reservation refusal".to_owned(),
            })
        }
    }

    struct RefusingEventRetention;

    impl SuccessfulEventRetention for RefusingEventRetention {
        fn copy_string(
            &self,
            field: &'static str,
            source: &str,
        ) -> Result<String, ReplayTickError> {
            ProductionSuccessfulEventRetention.copy_string(field, source)
        }

        fn project_value(
            &self,
            value: &babylon_bsl::evaluator::Value,
            resolver: &babylon_graph::stable_element::StableElementResolverV1,
        ) -> Result<babylon_bsl::identity_codec::StableBslValueV1, ReplayTickError> {
            if matches!(value, babylon_bsl::evaluator::Value::NodeRef(_)) {
                return Err(ReplayTickError::Allocation {
                    field: "injected stable event value retention",
                    requested: 1,
                });
            }
            ProductionSuccessfulEventRetention.project_value(value, resolver)
        }
    }

    struct RetentionRefusingComposer;

    impl ReplayIdentityComposer for RetentionRefusingComposer {
        fn compose<G: CanonicalState>(
            &self,
            inputs: ReplayIdentityInputs<'_, G, Self>,
        ) -> Result<ReplayIdentityArtifactsV2, ReplayTickError> {
            super::compose_replay_identity_with_retention(inputs, &RefusingEventRetention)
        }
    }

    struct RefusingMaterialAllocationGate {
        field: &'static str,
        refuse_on_occurrence: usize,
        observed: Cell<usize>,
    }

    impl MaterialAllocationGate for RefusingMaterialAllocationGate {
        fn before_reserve(
            &self,
            field: &'static str,
            requested: usize,
        ) -> Result<(), MaterialStateErrorV1> {
            if field == self.field {
                let observed = self.observed.get() + 1;
                self.observed.set(observed);
                if observed == self.refuse_on_occurrence {
                    return Err(MaterialStateErrorV1::Allocation { field, requested });
                }
            }
            Ok(())
        }
    }

    fn refusing_material_gate(
        field: &'static str,
        refuse_on_occurrence: usize,
    ) -> RefusingMaterialAllocationGate {
        RefusingMaterialAllocationGate {
            field,
            refuse_on_occurrence,
            observed: Cell::new(0),
        }
    }

    fn nonempty_action_batch(session: ReplaySessionIdV1) -> OrderedPracticeActionBatchV1 {
        let authority = PracticeInputAuthorityV2 {
            schema_version: 2,
            campaign_id: CampaignIdV2::from_bytes([0x10; 16]),
            authority_kind: PracticeAuthorityKindV2::PlayerSeat,
            input_authority_id: InputAuthorityIdV2::from_bytes([0x20; 16]),
            actor_org_id: ActorOrganizationIdV2::from_bytes(7_u64.to_be_bytes()),
            effective_from_tick: 10,
            effective_through_tick_exclusive: 20,
            decision_content_digest: [0x30; 32],
        };
        let ledger = PracticeInputAuthorityLedgerV2 {
            schema_version: 2,
            rows: vec![authority.clone()],
        };
        let intent = PracticeIntentV2 {
            schema_version: 2,
            submit_after_tick: 10,
            resolve_tick: 11,
            input_authority_id: InputAuthorityIdV2::from_bytes([0x20; 16]),
            actor_org_id: ActorOrganizationIdV2::from_bytes(7_u64.to_be_bytes()),
            practice_id: PracticeIdV2::Strike,
            target: TaggedPracticeTargetV2 {
                tag: PracticeTargetTagV2::LaborProcess,
                identity: PracticeTargetIdentityV2::from_bytes([0x50; 32]),
            },
            proposal_nonce: ProposalNonceV2::from_bytes([0x60; 16]),
            quoted_content_digest: [0x30; 32],
            quoted_resource_contract_digest: [0x40; 32],
            parameters: Vec::new(),
            evidence_digests: vec![[0x70; 32]],
        };
        let source = ResolvedPracticeBatchV2 {
            schema_version: 2,
            campaign_id: CampaignIdV2::from_bytes([0x10; 16]),
            resolve_tick: 11,
            authority_ledger_digest: input_authority_ledger_v2_digest(&ledger).unwrap(),
            resource_allocation_contract_digest: [0x40; 32],
            content_digest: [0x30; 32],
            items: vec![ResolvedPracticeBatchItemV2 { authority, intent }],
        };
        OrderedPracticeActionBatchV1::project(session, &source, &ledger).unwrap()
    }

    #[derive(Default)]
    struct RefusingSink {
        prepare_attempts: usize,
        commit_attempts: usize,
    }

    impl PreparedEventBatchSink for RefusingSink {
        fn try_prepare(&mut self, _additional: usize) -> Result<(), String> {
            self.prepare_attempts += 1;
            Err("injected replay event reservation refusal".to_owned())
        }

        fn commit_prepared(&mut self, _events: Vec<EventRecord>) {
            self.commit_attempts += 1;
        }
    }

    #[test]
    fn topology_change_refuses_without_advancing_or_reverting_the_external_change() {
        let mut session = session();
        session.graph.add_node("DYNAMIC").unwrap();
        let before = session.graph.encode_state().unwrap().as_bytes().to_vec();
        let actions = OrderedPracticeActionBatchV1::empty(session.session.clone(), 1).unwrap();
        let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();

        assert!(matches!(
            session.advance(&mut sink, &actions),
            Err(ReplayTickError::Stable(_))
        ));
        assert_eq!(session.completed_tick, 0);
        assert!(sink.events.is_empty());
        assert_eq!(session.graph.encode_state().unwrap().as_bytes(), before);
    }

    #[test]
    fn composer_refusal_leaves_graph_events_and_counter_unpublished() {
        let mut session = session();
        let before = session.graph.encode_state().unwrap().as_bytes().to_vec();
        let actions = OrderedPracticeActionBatchV1::empty(session.session.clone(), 1).unwrap();
        let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
        let execution = ReplayExecutionInputs {
            session: &session.session,
            seed: session.seed,
            content: &session.content,
            reference: session.reference,
            resolver: &session.resolver,
            register_manifest: &session.register_manifest,
            prepared_environment: &session.prepared_environment,
            actions: &actions,
            composer: &RefusingComposer,
            material_state: &session.material_state,
            material_allocation: &ProductionMaterialAllocationGate,
        };

        let error = crate::run_prepared_replay_tick(
            &session.prepared,
            &mut session.graph,
            &mut sink,
            1,
            execution,
            None,
        )
        .unwrap_err();
        assert!(matches!(error, ReplayTickError::Composer { .. }));
        assert_eq!(session.completed_tick, 0);
        assert!(sink.events.is_empty());
        assert_eq!(session.graph.encode_state().unwrap().as_bytes(), before);
    }

    #[test]
    fn retention_allocation_refusal_leaves_graph_events_and_counter_unpublished() {
        let mut session = session_for_rule(EVENT_RULE);
        let before = session.graph.encode_state().unwrap().as_bytes().to_vec();
        let before_cursors = session.graph.allocator_cursors();
        let before_completed_tick = session.completed_tick();
        let actions = OrderedPracticeActionBatchV1::empty(session.session.clone(), 1).unwrap();
        let mut sink = babylon_bsl::structural_verbs::CollectingSink {
            events: vec![("EventType/PRIOR".to_owned(), Vec::new())],
        };
        let before_events = sink.events.clone();

        let error = session
            .advance_with_composer(&mut sink, &actions, &RetentionRefusingComposer)
            .unwrap_err();
        assert!(matches!(
            error,
            ReplayTickError::Allocation {
                field: "injected stable event value retention",
                ..
            }
        ));
        assert_eq!(session.completed_tick(), before_completed_tick);
        assert_eq!(sink.events, before_events);
        assert_eq!(session.graph.encode_state().unwrap().as_bytes(), before);
        assert_eq!(session.graph.allocator_cursors(), before_cursors);
    }

    #[test]
    fn durable_commit_never_runs_when_publication_allocation_preflight_refuses() {
        let mut session = dynamic_allocation_session();
        let actions = OrderedPracticeActionBatchV1::empty(session.session.clone(), 1).unwrap();
        let prepared = session.prepare_advance(&actions).unwrap();
        assert!(!prepared.events.is_empty());
        let before_graph = session.graph.encode_state().unwrap().as_bytes().to_vec();
        let before_material = dynamic_allocation_material_state();
        let mut sink = babylon_bsl::structural_verbs::CollectingSink {
            events: vec![("EventType/PRIOR".to_owned(), Vec::new())],
        };
        let before_events = sink.events.clone();
        let commit_called = Cell::new(false);

        let error = session
            .commit_prepared_and_publish_with_allocation(
                &mut sink,
                prepared,
                |_report| {
                    commit_called.set(true);
                    Ok::<ReplayCommitDispositionV1, ()>(ReplayCommitDispositionV1::Committed)
                },
                &RefusingPublicationAllocation,
            )
            .unwrap_err();

        assert_eq!(
            error,
            PreparedReplayCommitErrorV1::Preflight(ReplayTickError::Allocation {
                field: "injected acknowledged replay event publication",
                requested: 1,
            })
        );
        assert!(!commit_called.get());
        assert_eq!(session.completed_tick, 0);
        assert_eq!(
            session.graph.encode_state().unwrap().as_bytes(),
            before_graph
        );
        assert_eq!(session.material_state, before_material);
        assert_eq!(sink.events, before_events);
    }

    #[test]
    fn material_detachment_allocation_refusals_leave_every_session_owner_unpublished() {
        for (actions, expected) in [
            (
                nonempty_action_batch(
                    ReplaySessionIdV1::try_from("per281/material-allocation").unwrap(),
                ),
                ReplayTickError::NonEmptyActionBatch { count: 1 },
            ),
            (
                OrderedPracticeActionBatchV1::empty(
                    ReplaySessionIdV1::try_from("per281/material-allocation-other").unwrap(),
                    1,
                )
                .unwrap(),
                ReplayTickError::ActionSessionMismatch,
            ),
            (
                OrderedPracticeActionBatchV1::empty(
                    ReplaySessionIdV1::try_from("per281/material-allocation").unwrap(),
                    2,
                )
                .unwrap(),
                ReplayTickError::ActionTickMismatch {
                    expected: 1,
                    actual: 2,
                },
            ),
        ] {
            let mut session = material_session();
            let before_graph = session.graph.encode_state().unwrap().as_bytes().to_vec();
            let before_cursors = session.graph.allocator_cursors();
            let before_material = material_state();
            let before_tick = session.completed_tick();
            let mut sink = babylon_bsl::structural_verbs::CollectingSink {
                events: vec![("EventType/PRIOR".to_owned(), Vec::new())],
            };
            let before_events = sink.events.clone();
            let gate = refusing_material_gate("material family rows", 1);

            assert_eq!(
                session
                    .advance_with_material_allocation(&mut sink, &actions, &gate)
                    .unwrap_err(),
                expected
            );
            assert_eq!(gate.observed.get(), 0);
            assert_eq!(
                session.graph.encode_state().unwrap().as_bytes(),
                before_graph
            );
            assert_eq!(session.graph.allocator_cursors(), before_cursors);
            assert_eq!(session.material_state(), &before_material);
            assert_eq!(session.completed_tick(), before_tick);
            assert_eq!(sink.events, before_events);
        }

        let baseline = {
            let mut session = material_session();
            let actions = OrderedPracticeActionBatchV1::empty(session.session.clone(), 1).unwrap();
            let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
            let report = session.advance(&mut sink, &actions).unwrap();
            (
                report.result_stable_graph().canonical_bytes().to_vec(),
                report.result_registers().canonical_bytes().to_vec(),
                report.payload().canonical_bytes().to_vec(),
                report.tick_content_hash(),
                report.material_state_rows().canonical_bytes().to_vec(),
                sink.events,
            )
        };

        for (field, occurrence) in [
            ("material territory key", 1),
            ("material territory field name", 1),
            ("material territory field value", 1),
            ("material territory row", 1),
            ("material organization presence topology", 1),
            ("material organization rows", 1),
            ("material organization identity", 1),
            ("material organization fields", 1),
            ("material organization field value", 2),
            ("material organization field name", 1),
            ("material organization territory keys", 1),
            ("material organization territory identity", 1),
            ("material organization territory key", 1),
            ("material organization territory ids", 1),
            ("material organization key", 1),
            ("material organization row", 1),
        ] {
            let mut session = material_session();
            let before_graph = session.graph.encode_state().unwrap().as_bytes().to_vec();
            let before_cursors = session.graph.allocator_cursors();
            let before_material = material_state();
            let before_tick = session.completed_tick();
            let actions = OrderedPracticeActionBatchV1::empty(session.session.clone(), 1).unwrap();
            let mut sink = babylon_bsl::structural_verbs::CollectingSink {
                events: vec![("EventType/PRIOR".to_owned(), Vec::new())],
            };
            let before_events = sink.events.clone();
            let gate = refusing_material_gate(field, occurrence);

            assert!(matches!(
                session.advance_with_material_allocation(&mut sink, &actions, &gate),
                Err(ReplayTickError::Allocation {
                    field: refused,
                    ..
                }) if refused == field
            ));
            assert_eq!(gate.observed.get(), occurrence);
            assert_eq!(
                session.graph.encode_state().unwrap().as_bytes(),
                before_graph
            );
            assert_eq!(session.graph.allocator_cursors(), before_cursors);
            assert_eq!(session.material_state(), &before_material);
            assert_eq!(session.completed_tick(), before_tick);
            assert_eq!(sink.events, before_events);

            let recovered = session.advance(&mut sink, &actions).unwrap();
            assert_eq!(
                (
                    recovered.result_stable_graph().canonical_bytes(),
                    recovered.result_registers().canonical_bytes(),
                    recovered.payload().canonical_bytes(),
                    recovered.tick_content_hash(),
                    recovered.material_state_rows().canonical_bytes(),
                ),
                (
                    baseline.0.as_slice(),
                    baseline.1.as_slice(),
                    baseline.2.as_slice(),
                    baseline.3,
                    baseline.4.as_slice(),
                )
            );
            assert_eq!(sink.events.first(), before_events.first());
            assert_eq!(&sink.events[1..], baseline.5.as_slice());
        }
    }

    #[test]
    fn dynamic_runtime_detachment_and_projection_refusals_are_atomic_and_retry_identical() {
        let baseline = {
            let mut session = dynamic_allocation_session();
            let actions = OrderedPracticeActionBatchV1::empty(session.session.clone(), 1).unwrap();
            let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();
            let report = session.advance(&mut sink, &actions).unwrap();
            (
                report
                    .material_state_rows()
                    .dynamic_hexes()
                    .canonical_bytes()
                    .to_vec(),
                sink.events,
            )
        };

        for (field, occurrence) in [
            ("material dynamic rows", 1),
            ("material dynamic rows", 2),
            ("material dynamic row", 1),
        ] {
            let mut session = dynamic_allocation_session();
            let before_graph = session.graph.encode_state().unwrap().as_bytes().to_vec();
            let before_cursors = session.graph.allocator_cursors();
            let before_material = dynamic_allocation_material_state();
            let before_tick = session.completed_tick();
            let actions = OrderedPracticeActionBatchV1::empty(session.session.clone(), 1).unwrap();
            let mut sink = babylon_bsl::structural_verbs::CollectingSink {
                events: vec![("EventType/PRIOR".to_owned(), Vec::new())],
            };
            let before_events = sink.events.clone();
            let gate = refusing_material_gate(field, occurrence);

            assert!(matches!(
                session.advance_with_material_allocation(&mut sink, &actions, &gate),
                Err(ReplayTickError::Allocation { field: refused, .. }) if refused == field
            ));
            assert_eq!(gate.observed.get(), occurrence);
            assert_eq!(
                session.graph.encode_state().unwrap().as_bytes(),
                before_graph
            );
            assert_eq!(session.graph.allocator_cursors(), before_cursors);
            assert_eq!(session.material_state(), &before_material);
            assert_eq!(session.completed_tick(), before_tick);
            assert_eq!(sink.events, before_events);

            let recovered = session.advance(&mut sink, &actions).unwrap();
            assert_eq!(
                recovered
                    .material_state_rows()
                    .dynamic_hexes()
                    .canonical_bytes(),
                baseline.0
            );
            assert_eq!(sink.events.first(), before_events.first());
            assert_eq!(&sink.events[1..], baseline.1.as_slice());
        }
    }

    #[test]
    fn derived_territory_owner_and_declaration_refusals_roll_back_every_owner() {
        for (qname, expected) in [
            (
                "social-class/wages",
                MaterialStateErrorV1::TerritoryFieldOwner,
            ),
            (
                "territory/undeclared",
                MaterialStateErrorV1::TerritoryFieldUndeclared,
            ),
        ] {
            let mut session = material_session();
            let territory = session.graph.nodes("TERRITORY")[0];
            session.graph.update_node(territory, qname, 7.0).unwrap();
            let before_graph = session.graph.encode_state().unwrap().as_bytes().to_vec();
            let before_cursors = session.graph.allocator_cursors();
            let before_material = material_state();
            let before_tick = session.completed_tick();
            let actions = OrderedPracticeActionBatchV1::empty(session.session.clone(), 1).unwrap();
            let mut sink = babylon_bsl::structural_verbs::CollectingSink {
                events: vec![("EventType/PRIOR".to_owned(), Vec::new())],
            };
            let before_events = sink.events.clone();

            assert_eq!(
                session.advance(&mut sink, &actions).unwrap_err(),
                ReplayTickError::MaterialState(expected)
            );
            assert_eq!(
                session.graph.encode_state().unwrap().as_bytes(),
                before_graph
            );
            assert_eq!(session.graph.allocator_cursors(), before_cursors);
            assert_eq!(session.material_state(), &before_material);
            assert_eq!(session.completed_tick(), before_tick);
            assert_eq!(sink.events, before_events);
        }
    }

    #[test]
    fn derived_organization_fields_and_presence_refuse_without_publication() {
        for (qname, expected) in [
            (
                "social-class/wages",
                MaterialStateErrorV1::OrganizationFieldOwner,
            ),
            (
                "organization/undeclared",
                MaterialStateErrorV1::OrganizationFieldUndeclared,
            ),
        ] {
            let mut session = material_session();
            let organization = session.graph.nodes("ORGANIZATION")[0];
            session.graph.update_node(organization, qname, 7.0).unwrap();
            assert_material_refusal_rolls_back(&mut session, expected);
        }

        for scenario in [
            MATERIAL_ATOMIC_SCENARIO.replace(
                "(edge EdgeType/PRESENCE org-a territory-a 1)",
                "(edge EdgeType/PRESENCE territory-a org-a 1)",
            ),
            MATERIAL_ATOMIC_SCENARIO.replace(
                "(edge EdgeType/PRESENCE org-a territory-a 1)",
                "(edge EdgeType/PRESENCE org-a workers 1)",
            ),
        ] {
            let mut session = material_session_for_scenario(&scenario);
            assert_material_refusal_rolls_back(
                &mut session,
                MaterialStateErrorV1::OrganizationTerritoryPresence,
            );
        }
    }

    fn assert_material_refusal_rolls_back(
        session: &mut ReplayTickSession<MemoryGraph>,
        expected: MaterialStateErrorV1,
    ) {
        let before_graph = session.graph.encode_state().unwrap().as_bytes().to_vec();
        let before_cursors = session.graph.allocator_cursors();
        let before_material = material_state();
        let before_tick = session.completed_tick();
        let actions = OrderedPracticeActionBatchV1::empty(session.session.clone(), 1).unwrap();
        let mut sink = babylon_bsl::structural_verbs::CollectingSink {
            events: vec![("EventType/PRIOR".to_owned(), Vec::new())],
        };
        let before_events = sink.events.clone();

        assert_eq!(
            session.advance(&mut sink, &actions).unwrap_err(),
            ReplayTickError::MaterialState(expected)
        );
        assert_eq!(
            session.graph.encode_state().unwrap().as_bytes(),
            before_graph
        );
        assert_eq!(session.graph.allocator_cursors(), before_cursors);
        assert_eq!(session.material_state(), &before_material);
        assert_eq!(session.completed_tick(), before_tick);
        assert_eq!(sink.events, before_events);
    }

    #[test]
    fn event_reservation_refuses_after_identity_and_before_graph_publication() {
        let mut session = session();
        let before = session.graph.encode_state().unwrap().as_bytes().to_vec();
        let actions = OrderedPracticeActionBatchV1::empty(session.session.clone(), 1).unwrap();
        let composer = super::ProductionReplayIdentityComposer;
        let execution = ReplayExecutionInputs {
            session: &session.session,
            seed: session.seed,
            content: &session.content,
            reference: session.reference,
            resolver: &session.resolver,
            register_manifest: &session.register_manifest,
            prepared_environment: &session.prepared_environment,
            actions: &actions,
            composer: &composer,
            material_state: &session.material_state,
            material_allocation: &ProductionMaterialAllocationGate,
        };
        let mut sink = RefusingSink::default();
        let result = crate::run_prepared_tick_transaction(
            &session.prepared,
            &mut session.graph,
            &mut sink,
            &crate::ExecutionIdentity::Replay(execution),
            1,
            |_boundary, graph: &MemoryGraph| graph.state_hash(),
            None,
        );

        assert!(result.is_err());
        assert_eq!(sink.prepare_attempts, 1);
        assert_eq!(sink.commit_attempts, 0);
        assert_eq!(session.completed_tick, 0);
        assert_eq!(session.graph.encode_state().unwrap().as_bytes(), before);
    }

    #[test]
    fn completed_tick_overflow_refuses_before_action_or_graph_work() {
        let mut session = session();
        session.completed_tick = i64::MAX;
        session
            .graph
            .update_node(
                babylon_graph::substrate::NodeId(0),
                "social-class/wages",
                f64::NAN,
            )
            .unwrap();
        let actions = OrderedPracticeActionBatchV1::empty(session.session.clone(), 1).unwrap();
        let mut sink = babylon_bsl::structural_verbs::CollectingSink::default();

        assert!(matches!(
            session.advance(&mut sink, &actions),
            Err(ReplayTickError::TickCounterOverflow)
        ));
        assert_eq!(session.completed_tick, i64::MAX);
        assert!(sink.events.is_empty());
    }
}
