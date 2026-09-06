//! Sole Rust persistence activation and durable replay composition root.

use babylon_bsl::identity_codec::StableBslValueV1;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::stable_state::StableGraphStateV1;
use babylon_kernel::sha256_of;
use babylon_kernel::tick_content_hash::TickContentHashV1;
use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
use babylon_tick::choice_receipt::ChoiceReceiptV1;
use babylon_tick::material_state::MaterialStateV1;
use babylon_tick::replay_session::{
    IdentifiedTickReportV2, PreparedReplayCommitErrorV1, ReplayCommitAcknowledgementV1,
    ReplayCommitDispositionV1, ReplayTickSession,
};
use postgres::binary_copy::BinaryCopyInWriter;
use postgres::types::{ToSql, Type};
use postgres::{Config, GenericClient, NoTls};

use crate::archive::SemanticArchiveStoreV1;
use crate::bootstrap::{bootstrap_h3_reader_epoch_v1, H3ReaderBootstrapErrorV1};
use crate::checkpoint::{
    compose_archive_dirty_receipt_v1, compose_checkpoint_rows_v1, ArchiveDirtyReceiptV1,
    CheckpointRowsV1, CommittedFullCheckpointV1, CommittedResolveTickErrorV1,
    CommittedResolveTickV1,
};
use crate::committed_tick_envelope::{
    CommittedTickEnvelopeErrorV2, CommittedTickEnvelopeV2, CommittedTickRowFamiliesV2,
    CommittedTickRowV2,
};
use crate::foundation::{
    CampaignFoundationV1, FoundationContentBundle, FoundationContentBundleV1,
    FoundationContentLayout,
};
use crate::identity::CampaignId;
use crate::legacy_adopter::{
    acquire_lock, release_lock, validate_legacy_connection_target, LegacyAdopterError,
};
use crate::metadata::{
    advance_campaign_catalog_tick_v1, ensure_campaign_catalog_row_v1, read_campaign_catalog_row_v1,
};
use crate::michigan_dynamic_hex_foundation::michigan_dynamic_hex_foundation_v1;
use crate::postgres_diagnostic::PostgresDiagnosticV1;
use crate::schema_epoch::compiled_committed_tick_v2_activation_migrations;
use crate::schema_migration::SchemaMigration;
use crate::semantic_batches::{
    compose_graph_event_choice_semantic_batches_v2, compose_material_state_rows_v1,
    GraphEventChoiceSemanticBatchesV2, SemanticBatchErrorV2,
};
use crate::semantic_codec::SemanticCodecErrorV1;
use crate::stored_tick::read_stored_typed_tick_v2;
use crate::tick_commit_claim::TickCommitClaimV1;

const MIGRATION_0008_SQL: &str =
    include_str!("../migrations/0008_rust_persistence_preparation.sql");
const MIGRATION_0009_SQL: &str = include_str!("../migrations/0009_rust_persistence_activation.sql");
const PREDECESSOR_CUTOVER_CONTRACT: &[u8] =
    include_bytes!("../../../../contracts/rust_persistence_cutover_v1.yaml");
const PREDECESSOR_READER_CONTRACT: &[u8] =
    include_bytes!("../../../../contracts/h3_reader_cutover_v1.yaml");
const ACTIVE_V2_CUTOVER_CONTRACT: &[u8] =
    include_bytes!("../../../../contracts/rust_persistence_cutover_v2.yaml");
const PREDECESSOR_AUTHORITY_DOMAIN: &[u8] = b"babylon.persistence-authority-ledger-row.v1\0";
const PREDECESSOR_AUTHORITY_LAYOUT: u32 = 1;
const COMMITTED_TICK_V2_AUTHORITY_DOMAIN: &[u8] =
    b"babylon.committed-tick-v2-authority-ledger-row.v1\0";
const COMMITTED_TICK_V2_AUTHORITY_LAYOUT: u32 = 1;
const REQUIRED_POSTGRESQL_SERVER_MAJOR_V2: u32 = 17;
const REFERENCE_BUNDLE_DOMAIN_V1: &[u8] = b"babylon.h3.reference-bundle-composite.v1\0";
const PRE_ACTIVATION_INCOMPATIBLE_RELATIONS_V2: &[&str] = &[
    "babylon_state.archive_dirty_receipt_v1",
    "babylon_state.campaign",
    "babylon_state.campaign_foundation",
    "babylon_state.checkpoint_manifest",
    "babylon_state.checkpoint_section_v1",
    "babylon_state.graph_edge_f64_v1",
    "babylon_state.graph_edge_v1",
    "babylon_state.graph_hyperedge_f64_v1",
    "babylon_state.graph_hyperedge_member_v1",
    "babylon_state.graph_hyperedge_v1",
    "babylon_state.graph_node_currency_v1",
    "babylon_state.graph_node_f64_v1",
    "babylon_state.graph_node_v1",
    "babylon_state.hex_state_delta_v1",
    "babylon_state.organization_state_field_v1",
    "babylon_state.organization_state_v1",
    "babylon_state.organization_territory_v1",
    "babylon_state.territory_state_field_v1",
    "babylon_state.territory_state_v1",
    "babylon_state.tick_action_batch_v1",
    "babylon_state.tick_archive_dirty_receipt_row",
    "babylon_state.tick_boundary_flow_row",
    "babylon_state.tick_checkpoint_row",
    "babylon_state.tick_commit",
    "babylon_state.tick_conservation_row",
    "babylon_state.tick_event_field_v1",
    "babylon_state.tick_event_row",
    "babylon_state.tick_event_v1",
    "babylon_state.tick_graph_row",
    "babylon_state.tick_state_row",
    "babylon_state.tick_subsystem_row",
    "babylon_state.world_register_v1",
    "public.action_result",
    "public.balkanization_claims_audit",
    "public.balkanization_influences_audit",
    "public.boundary_flow_register",
    "public.class_snapshot",
    "public.community_membership",
    "public.community_snapshot",
    "public.community_state",
    "public.conservation_audit_log",
    "public.contradiction_field",
    "public.dynamic_consciousness_state",
    "public.dynamic_demographics_state",
    "public.dynamic_employment_state",
    "public.dynamic_external_node_state",
    "public.dynamic_hex_state",
    "public.dynamic_relationship_state",
    "public.economic_summary",
    "public.edge_curvature",
    "public.edge_snapshot",
    "public.edge_state",
    "public.game_defines_snapshot",
    "public.game_session",
    "public.game_turn",
    "public.graph_metadata",
    "public.hex_activity",
    "public.hex_cell",
    "public.hex_latest",
    "public.hex_map",
    "public.hex_r8_linear_features_reference",
    "public.hex_r8_reference",
    "public.hex_spatial_map",
    "public.hex_state",
    "public.hex_substrate",
    "public.hex_terrain_state",
    "public.immutable_reference_basket_gamma",
    "public.immutable_reference_bea_io",
    "public.immutable_reference_bea_reis_rent",
    "public.immutable_reference_border_commute_synthesis",
    "public.immutable_reference_erdi",
    "public.immutable_reference_faf_freight",
    "public.immutable_reference_fred_rates",
    "public.immutable_reference_hickel_drain",
    "public.immutable_reference_lodes_od_matrix",
    "public.immutable_reference_melt_tau",
    "public.immutable_reference_qcew_employment",
    "public.immutable_reference_ricci_unequal",
    "public.immutable_reference_tiger_county",
    "public.infrastructure_link_state",
    "public.node_state",
    "public.org_snapshot",
    "public.runtime_administers_edges",
    "public.runtime_claims_edges",
    "public.runtime_influences_edges",
    "public.runtime_political_factions",
    "public.runtime_sovereigns",
    "public.simulation_event",
    "public.territory_snapshot",
    "public.tick_commit",
    "public.tick_event",
    "public.tick_log",
    "public.tick_summary",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct PredecessorAuthorityLedgerRowV2 {
    ordinal: u16,
    state_tag: u8,
    schema_epoch: u16,
    contract_sha256: [u8; 32],
    reader_contract_sha256: [u8; 32],
    predecessor_sha256: Option<[u8; 32]>,
    row_sha256: [u8; 32],
}

impl PredecessorAuthorityLedgerRowV2 {
    fn prepared() -> Result<Self, RustPersistenceActivationErrorV2> {
        Self::compose(
            1,
            1,
            8,
            sha256_of(PREDECESSOR_CUTOVER_CONTRACT),
            sha256_of(PREDECESSOR_READER_CONTRACT),
            None,
        )
    }

    fn active(prepared: &Self) -> Result<Self, RustPersistenceActivationErrorV2> {
        Self::compose(
            2,
            2,
            9,
            prepared.contract_sha256,
            prepared.reader_contract_sha256,
            Some(prepared.row_sha256),
        )
    }

    fn compose(
        ordinal: u16,
        state_tag: u8,
        schema_epoch: u16,
        contract_sha256: [u8; 32],
        reader_contract_sha256: [u8; 32],
        predecessor_sha256: Option<[u8; 32]>,
    ) -> Result<Self, RustPersistenceActivationErrorV2> {
        let capacity = PREDECESSOR_AUTHORITY_DOMAIN
            .len()
            .checked_add(4 + 2 + 1 + 2 + 32 + 32 + 1)
            .and_then(|value| value.checked_add(predecessor_sha256.map_or(0, |_| 32)))
            .ok_or(RustPersistenceActivationErrorV2::LedgerEncoding)?;
        let mut canonical_bytes = Vec::new();
        canonical_bytes
            .try_reserve_exact(capacity)
            .map_err(|_| RustPersistenceActivationErrorV2::LedgerEncoding)?;
        canonical_bytes.extend_from_slice(PREDECESSOR_AUTHORITY_DOMAIN);
        canonical_bytes.extend_from_slice(&PREDECESSOR_AUTHORITY_LAYOUT.to_be_bytes());
        canonical_bytes.extend_from_slice(&ordinal.to_be_bytes());
        canonical_bytes.push(state_tag);
        canonical_bytes.extend_from_slice(&schema_epoch.to_be_bytes());
        canonical_bytes.extend_from_slice(&contract_sha256);
        canonical_bytes.extend_from_slice(&reader_contract_sha256);
        match predecessor_sha256 {
            None => canonical_bytes.push(0),
            Some(predecessor) => {
                canonical_bytes.push(1);
                canonical_bytes.extend_from_slice(&predecessor);
            }
        }
        if canonical_bytes.len() != capacity {
            return Err(RustPersistenceActivationErrorV2::LedgerEncoding);
        }
        Ok(Self {
            ordinal,
            state_tag,
            schema_epoch,
            contract_sha256,
            reader_contract_sha256,
            predecessor_sha256,
            row_sha256: sha256_of(&canonical_bytes),
        })
    }
}

/// Closed one-way persistence authority state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommittedTickAuthorityStateV2 {
    /// The additive V2 schema and incompatible-row inventory exist.
    Prepared,
    /// Activation epoch 11 committed with the V2-active row as its final DML.
    Active,
}

impl CommittedTickAuthorityStateV2 {
    const fn tag(self) -> u8 {
        match self {
            Self::Prepared => 1,
            Self::Active => 2,
        }
    }
}

/// One exact digest-chained row from the persistence authority ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedTickAuthorityLedgerRowV2 {
    ordinal: u16,
    state: CommittedTickAuthorityStateV2,
    activation_epoch: u16,
    contract_sha256: [u8; 32],
    reader_contract_sha256: [u8; 32],
    predecessor_sha256: [u8; 32],
    canonical_bytes: Vec<u8>,
    row_sha256: [u8; 32],
}

impl CommittedTickAuthorityLedgerRowV2 {
    fn compose(
        ordinal: u16,
        state: CommittedTickAuthorityStateV2,
        activation_epoch: u16,
        contract_sha256: [u8; 32],
        reader_contract_sha256: [u8; 32],
        predecessor_sha256: [u8; 32],
    ) -> Result<Self, RustPersistenceActivationErrorV2> {
        let capacity = COMMITTED_TICK_V2_AUTHORITY_DOMAIN
            .len()
            .checked_add(4 + 2 + 1 + 2 + 32 + 32 + 32)
            .ok_or(RustPersistenceActivationErrorV2::LedgerEncoding)?;
        let mut canonical_bytes = Vec::new();
        canonical_bytes
            .try_reserve_exact(capacity)
            .map_err(|_| RustPersistenceActivationErrorV2::LedgerEncoding)?;
        canonical_bytes.extend_from_slice(COMMITTED_TICK_V2_AUTHORITY_DOMAIN);
        canonical_bytes.extend_from_slice(&COMMITTED_TICK_V2_AUTHORITY_LAYOUT.to_be_bytes());
        canonical_bytes.extend_from_slice(&ordinal.to_be_bytes());
        canonical_bytes.push(state.tag());
        canonical_bytes.extend_from_slice(&activation_epoch.to_be_bytes());
        canonical_bytes.extend_from_slice(&contract_sha256);
        canonical_bytes.extend_from_slice(&reader_contract_sha256);
        canonical_bytes.extend_from_slice(&predecessor_sha256);
        if canonical_bytes.len() != capacity {
            return Err(RustPersistenceActivationErrorV2::LedgerEncoding);
        }
        let row_sha256 = sha256_of(&canonical_bytes);
        Ok(Self {
            ordinal,
            state,
            activation_epoch,
            contract_sha256,
            reader_contract_sha256,
            predecessor_sha256,
            canonical_bytes,
            row_sha256,
        })
    }

    /// Return the fixed one-based ledger ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> u16 {
        self.ordinal
    }

    /// Return the closed authority state.
    #[must_use]
    pub const fn state(&self) -> CommittedTickAuthorityStateV2 {
        self.state
    }

    /// Return the dedicated activation epoch.
    #[must_use]
    pub const fn activation_epoch(&self) -> u16 {
        self.activation_epoch
    }

    /// Return the exact cutover-contract digest.
    #[must_use]
    pub const fn contract_sha256(&self) -> [u8; 32] {
        self.contract_sha256
    }

    /// Return the exact joined reader-contract digest.
    #[must_use]
    pub const fn reader_contract_sha256(&self) -> [u8; 32] {
        self.reader_contract_sha256
    }

    /// Return the exact predecessor-row digest when this is the active row.
    #[must_use]
    pub const fn predecessor_sha256(&self) -> [u8; 32] {
        self.predecessor_sha256
    }
}

/// Successful exact two-state activation receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivationReportV2 {
    prepared_row: CommittedTickAuthorityLedgerRowV2,
    active_row: CommittedTickAuthorityLedgerRowV2,
}

impl ActivationReportV2 {
    /// Borrow the exact durable preparation row.
    #[must_use]
    pub const fn prepared_row(&self) -> &CommittedTickAuthorityLedgerRowV2 {
        &self.prepared_row
    }

    /// Borrow the exact terminal V2-authority row.
    #[must_use]
    pub const fn active_row(&self) -> &CommittedTickAuthorityLedgerRowV2 {
        &self.active_row
    }
}

/// One exact nonempty relation observed before activation can mutate the target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreActivationIncompatibleRelationV2 {
    relation_name: &'static str,
    observed_row_count: u64,
}

impl PreActivationIncompatibleRelationV2 {
    /// Return the closed relation name from the activation inventory.
    #[must_use]
    pub const fn relation_name(&self) -> &'static str {
        self.relation_name
    }

    /// Return the exact row count from the read-only pre-activation snapshot.
    #[must_use]
    pub const fn observed_row_count(&self) -> u64 {
        self.observed_row_count
    }
}

/// Closed activation refusal surface without driver or credential leakage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustPersistenceActivationErrorV2 {
    /// The target violated the local maintenance connection contract.
    ConnectionTarget,
    /// The ordinary reader epoch or internal epoch-8/9 predecessor could not be established.
    PredecessorBootstrap(H3ReaderBootstrapErrorV1),
    /// The production activation target is not the exact supported `PostgreSQL` server major.
    PostgreSqlServerMajorMismatch {
        /// Major decoded from the server-owned `server_version_num` setting.
        observed_major: u32,
    },
    /// The dedicated compiled activation registry could not be constructed.
    MigrationRegistry,
    /// A bounded database operation failed.
    Database {
        /// Stable operation name without caller-supplied text.
        operation: &'static str,
        /// Bounded secret-safe driver diagnostic, when the failure came from `PostgreSQL`.
        diagnostic: Option<PostgresDiagnosticV1>,
    },
    /// Advisory-lock acquisition failed with its typed database cause.
    Lock(LegacyAdopterError),
    /// Existing nonempty authority rows were identified before any activation mutation.
    PreActivationIncompatibleInventory {
        /// Exact sorted relation/count targets from one read-only snapshot.
        relations: Vec<PreActivationIncompatibleRelationV2>,
    },
    /// A closed inventory name resolved to something other than a table.
    PreActivationRelationShape { relation_name: &'static str },
    /// The durable ledger did not equal the exact two-row state machine.
    AuthorityLedgerMismatch,
    /// Exact ledger bytes could not be allocated or composed.
    LedgerEncoding,
    /// The advisory-lock cleanup failed after the primary operation.
    Cleanup(LegacyAdopterError),
}

impl std::fmt::Display for RustPersistenceActivationErrorV2 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Rust persistence activation refused: {self:?}")
    }
}

impl std::error::Error for RustPersistenceActivationErrorV2 {}

impl RustPersistenceActivationErrorV2 {
    fn database(operation: &'static str) -> Self {
        Self::Database {
            operation,
            diagnostic: None,
        }
    }

    fn postgres(operation: &'static str, error: &postgres::Error) -> Self {
        Self::Database {
            operation,
            diagnostic: Some(PostgresDiagnosticV1::capture(error)),
        }
    }

    fn at_operation(self, operation: &'static str) -> Self {
        match self {
            Self::Database { diagnostic, .. } => Self::Database {
                operation,
                diagnostic,
            },
            other => other,
        }
    }
}

fn expected_v2_activation_report() -> Result<ActivationReportV2, RustPersistenceActivationErrorV2> {
    let predecessor_prepared = PredecessorAuthorityLedgerRowV2::prepared()?;
    let predecessor_active = PredecessorAuthorityLedgerRowV2::active(&predecessor_prepared)?;
    let migrations = compiled_committed_tick_v2_activation_migrations()
        .map_err(|_| RustPersistenceActivationErrorV2::MigrationRegistry)?;
    let (contract_sha256, reader_contract_sha256) = v2_authority_contract_digests(&migrations);
    let prepared_row = CommittedTickAuthorityLedgerRowV2::compose(
        1,
        CommittedTickAuthorityStateV2::Prepared,
        10,
        contract_sha256,
        reader_contract_sha256,
        predecessor_active.row_sha256,
    )?;
    let active_row = CommittedTickAuthorityLedgerRowV2::compose(
        2,
        CommittedTickAuthorityStateV2::Active,
        11,
        contract_sha256,
        reader_contract_sha256,
        prepared_row.row_sha256,
    )?;
    Ok(ActivationReportV2 {
        prepared_row,
        active_row,
    })
}

fn v2_authority_contract_digests(migrations: &[SchemaMigration; 2]) -> ([u8; 32], [u8; 32]) {
    (
        sha256_of(ACTIVE_V2_CUTOVER_CONTRACT),
        *migrations[1].checksum().as_bytes(),
    )
}

fn preflight_v2_activation_before_mutation(
    config: &Config,
) -> Result<(), RustPersistenceActivationErrorV2> {
    let expected = expected_v2_activation_report()?;
    let expected_predecessor_prepared = PredecessorAuthorityLedgerRowV2::prepared()?;
    let expected_predecessor_active =
        PredecessorAuthorityLedgerRowV2::active(&expected_predecessor_prepared)?;
    let mut client = config.connect(NoTls).map_err(|error| {
        RustPersistenceActivationErrorV2::postgres("connect for pre-activation inventory", &error)
    })?;
    require_postgresql_server_major_v2(&mut client)?;
    let mut transaction = client.transaction().map_err(|error| {
        RustPersistenceActivationErrorV2::postgres("begin pre-activation inventory", &error)
    })?;
    transaction
        .batch_execute(
            "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY; \
             SET LOCAL search_path TO pg_catalog",
        )
        .map_err(|error| {
            RustPersistenceActivationErrorV2::postgres("configure pre-activation inventory", &error)
        })?;

    let observed_v2 = read_v2_authority_ledger(&mut transaction)?;
    let terminal_v2 = observed_v2 == [expected.prepared_row.clone(), expected.active_row.clone()];
    if !terminal_v2
        && !observed_v2.is_empty()
        && observed_v2.as_slice() != [expected.prepared_row.clone()]
    {
        return Err(RustPersistenceActivationErrorV2::AuthorityLedgerMismatch);
    }
    if terminal_v2 {
        let observed_predecessor = read_predecessor_authority_ledger_v2(&mut transaction)?;
        if observed_predecessor != [expected_predecessor_prepared, expected_predecessor_active] {
            return Err(RustPersistenceActivationErrorV2::AuthorityLedgerMismatch);
        }
        transaction.commit().map_err(|error| {
            RustPersistenceActivationErrorV2::postgres(
                "commit terminal pre-activation probe",
                &error,
            )
        })?;
        return Ok(());
    }

    let relations = read_pre_activation_incompatible_inventory_v2(&mut transaction)?;
    transaction.commit().map_err(|error| {
        RustPersistenceActivationErrorV2::postgres("commit pre-activation inventory", &error)
    })?;
    if relations.is_empty() {
        Ok(())
    } else {
        Err(RustPersistenceActivationErrorV2::PreActivationIncompatibleInventory { relations })
    }
}

fn require_postgresql_server_major_v2(
    client: &mut impl GenericClient,
) -> Result<(), RustPersistenceActivationErrorV2> {
    let server_version_num: i32 = client
        .query_one(
            "SELECT pg_catalog.current_setting('server_version_num')::pg_catalog.int4",
            &[],
        )
        .and_then(|row| row.try_get(0))
        .map_err(|error| {
            RustPersistenceActivationErrorV2::postgres("read PostgreSQL server version", &error)
        })?;
    validate_postgresql_server_version_num_v2(server_version_num)
}

fn validate_postgresql_server_version_num_v2(
    server_version_num: i32,
) -> Result<(), RustPersistenceActivationErrorV2> {
    let observed_major = u32::try_from(server_version_num).map_err(|_| {
        RustPersistenceActivationErrorV2::database("decode PostgreSQL server version")
    })? / 10_000;
    if observed_major == REQUIRED_POSTGRESQL_SERVER_MAJOR_V2 {
        Ok(())
    } else {
        Err(RustPersistenceActivationErrorV2::PostgreSqlServerMajorMismatch { observed_major })
    }
}

fn read_pre_activation_incompatible_inventory_v2(
    client: &mut impl GenericClient,
) -> Result<Vec<PreActivationIncompatibleRelationV2>, RustPersistenceActivationErrorV2> {
    let mut relations = Vec::new();
    relations
        .try_reserve_exact(PRE_ACTIVATION_INCOMPATIBLE_RELATIONS_V2.len())
        .map_err(|_| RustPersistenceActivationErrorV2::LedgerEncoding)?;
    for &relation_name in PRE_ACTIVATION_INCOMPATIBLE_RELATIONS_V2 {
        let Some(observed_row_count) = pre_activation_relation_row_count_v2(client, relation_name)?
        else {
            continue;
        };
        if observed_row_count > 0 {
            relations.push(PreActivationIncompatibleRelationV2 {
                relation_name,
                observed_row_count,
            });
        }
    }
    relations.sort_unstable_by_key(PreActivationIncompatibleRelationV2::relation_name);
    Ok(relations)
}

fn pre_activation_relation_row_count_v2(
    client: &mut impl GenericClient,
    relation_name: &'static str,
) -> Result<Option<u64>, RustPersistenceActivationErrorV2> {
    let is_table: Option<bool> = client
        .query_one(
            "SELECT CASE \
               WHEN pg_catalog.to_regclass($1::pg_catalog.text) IS NULL THEN NULL \
               ELSE EXISTS ( \
                 SELECT 1 FROM pg_catalog.pg_class AS relation \
                  WHERE relation.oid = pg_catalog.to_regclass($1::pg_catalog.text) \
                    AND relation.relkind IN ('r', 'p') \
               ) \
             END",
            &[&relation_name],
        )
        .and_then(|row| row.try_get(0))
        .map_err(|error| {
            RustPersistenceActivationErrorV2::postgres(
                "inspect pre-activation relation shape",
                &error,
            )
        })?;
    match is_table {
        None => return Ok(None),
        Some(false) => {
            return Err(
                RustPersistenceActivationErrorV2::PreActivationRelationShape { relation_name },
            );
        }
        Some(true) => {}
    }
    let sql = format!("SELECT pg_catalog.count(*) FROM {relation_name}");
    let row_count: i64 = client
        .query_one(&sql, &[])
        .and_then(|row| row.try_get(0))
        .map_err(|error| {
            RustPersistenceActivationErrorV2::postgres("count pre-activation relation", &error)
        })?;
    u64::try_from(row_count).map(Some).map_err(|_| {
        RustPersistenceActivationErrorV2::database("decode pre-activation relation count")
    })
}

fn establish_predecessor_authority_v2(
    config: &Config,
) -> Result<(), RustPersistenceActivationErrorV2> {
    let expected_prepared = PredecessorAuthorityLedgerRowV2::prepared()?;
    let expected_active = PredecessorAuthorityLedgerRowV2::active(&expected_prepared)?;
    let mut probe = config.connect(NoTls).map_err(|error| {
        RustPersistenceActivationErrorV2::postgres("connect for predecessor probe", &error)
    })?;
    let observed = read_predecessor_authority_ledger_v2(&mut probe)?;
    if observed == [expected_prepared.clone(), expected_active.clone()] {
        return Ok(());
    }
    if !observed.is_empty() && observed.as_slice() != [expected_prepared.clone()] {
        return Err(RustPersistenceActivationErrorV2::AuthorityLedgerMismatch);
    }
    drop(probe);

    if observed.is_empty() {
        if let Err(error) = bootstrap_h3_reader_epoch_v1(config) {
            let mut reconciler = config.connect(NoTls).map_err(|error| {
                RustPersistenceActivationErrorV2::postgres(
                    "connect for predecessor reconciliation",
                    &error,
                )
            })?;
            if read_predecessor_authority_ledger_v2(&mut reconciler)?
                == [expected_prepared.clone(), expected_active.clone()]
            {
                return Ok(());
            }
            return Err(RustPersistenceActivationErrorV2::PredecessorBootstrap(
                error,
            ));
        }
    }

    let mut client = config.connect(NoTls).map_err(|error| {
        RustPersistenceActivationErrorV2::postgres("connect predecessor activation", &error)
    })?;
    acquire_lock(&mut client).map_err(RustPersistenceActivationErrorV2::Lock)?;
    let result =
        establish_predecessor_under_lock_v2(&mut client, &expected_prepared, &expected_active);
    let cleanup = release_lock(&mut client).map_err(RustPersistenceActivationErrorV2::Cleanup);
    match cleanup {
        Ok(()) => result,
        Err(error) => Err(error),
    }
}

fn establish_predecessor_under_lock_v2(
    client: &mut postgres::Client,
    expected_prepared: &PredecessorAuthorityLedgerRowV2,
    expected_active: &PredecessorAuthorityLedgerRowV2,
) -> Result<(), RustPersistenceActivationErrorV2> {
    let mut observed = read_predecessor_authority_ledger_v2(client)?;
    if observed.is_empty() {
        let commit_result = execute_predecessor_migration_v2(
            client,
            MIGRATION_0008_SQL,
            expected_prepared,
            "migration 8",
        );
        observed = read_predecessor_authority_ledger_v2(client)?;
        if observed.as_slice() != [expected_prepared.clone()] {
            if let Err(error) = commit_result {
                return Err(error.at_operation("unresolved predecessor preparation commit"));
            }
        }
    }
    if observed.as_slice() == [expected_prepared.clone()] {
        let commit_result = execute_predecessor_migration_v2(
            client,
            MIGRATION_0009_SQL,
            expected_active,
            "migration 9",
        );
        observed = read_predecessor_authority_ledger_v2(client)?;
        if observed.as_slice() != [expected_prepared.clone(), expected_active.clone()] {
            if let Err(error) = commit_result {
                return Err(error.at_operation("unresolved predecessor activation commit"));
            }
        }
    }
    if observed != [expected_prepared.clone(), expected_active.clone()] {
        return Err(RustPersistenceActivationErrorV2::AuthorityLedgerMismatch);
    }
    Ok(())
}

const SERIALIZABLE_ACTIVATION_SETTINGS_V2: &str = "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE; \
     SET LOCAL search_path TO pg_catalog; SET LOCAL synchronous_commit TO on";
const LOCKED_CENSUS_ACTIVATION_SETTINGS_V2: &str =
    "SET TRANSACTION ISOLATION LEVEL READ COMMITTED; \
     SET LOCAL search_path TO pg_catalog; SET LOCAL synchronous_commit TO on";

/// Epoch 9 discovers legacy relations before locking each one, so a
/// transaction-wide snapshot could predate a writer that commits while an
/// `ACCESS EXCLUSIVE` lock is pending. `READ COMMITTED` gives each post-lock
/// census a fresh snapshot; the retained relation locks and outer activation
/// advisory lock prevent later drift. Other migrations remain serializable.
const fn predecessor_activation_transaction_settings_v2(schema_epoch: u16) -> &'static str {
    if schema_epoch == 9 {
        LOCKED_CENSUS_ACTIVATION_SETTINGS_V2
    } else {
        SERIALIZABLE_ACTIVATION_SETTINGS_V2
    }
}

fn execute_predecessor_migration_v2(
    client: &mut postgres::Client,
    sql: &str,
    authority_row: &PredecessorAuthorityLedgerRowV2,
    operation: &'static str,
) -> Result<(), RustPersistenceActivationErrorV2> {
    let mut transaction = client
        .transaction()
        .map_err(|error| RustPersistenceActivationErrorV2::postgres(operation, &error))?;
    transaction
        .batch_execute(predecessor_activation_transaction_settings_v2(
            authority_row.schema_epoch,
        ))
        .map_err(|error| RustPersistenceActivationErrorV2::postgres(operation, &error))?;
    transaction
        .batch_execute(sql)
        .map_err(|error| RustPersistenceActivationErrorV2::postgres(operation, &error))?;
    insert_predecessor_authority_row_v2(&mut transaction, authority_row)?;
    transaction
        .commit()
        .map_err(|error| RustPersistenceActivationErrorV2::postgres(operation, &error))
}

fn insert_predecessor_authority_row_v2(
    client: &mut impl GenericClient,
    row: &PredecessorAuthorityLedgerRowV2,
) -> Result<(), RustPersistenceActivationErrorV2> {
    let affected = client
        .execute(
            "INSERT INTO babylon_meta.persistence_authority_ledger \
             (ordinal, state_tag, schema_epoch, contract_sha256, reader_contract_sha256, \
              predecessor_sha256, row_sha256) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &i16::try_from(row.ordinal)
                    .map_err(|_| RustPersistenceActivationErrorV2::LedgerEncoding)?,
                &i16::from(row.state_tag),
                &i16::try_from(row.schema_epoch)
                    .map_err(|_| RustPersistenceActivationErrorV2::LedgerEncoding)?,
                &&row.contract_sha256[..],
                &&row.reader_contract_sha256[..],
                &row.predecessor_sha256.as_ref().map(<[u8; 32]>::as_slice),
                &&row.row_sha256[..],
            ],
        )
        .map_err(|error| {
            RustPersistenceActivationErrorV2::postgres("insert predecessor authority row", &error)
        })?;
    if affected != 1 {
        return Err(RustPersistenceActivationErrorV2::AuthorityLedgerMismatch);
    }
    Ok(())
}

fn read_predecessor_authority_ledger_v2(
    client: &mut impl GenericClient,
) -> Result<Vec<PredecessorAuthorityLedgerRowV2>, RustPersistenceActivationErrorV2> {
    let exists: bool = client
        .query_one(
            "SELECT pg_catalog.to_regclass('babylon_meta.persistence_authority_ledger') IS NOT NULL",
            &[],
        )
        .and_then(|row| row.try_get(0))
        .map_err(|error| {
            RustPersistenceActivationErrorV2::postgres(
                "locate predecessor authority ledger",
                &error,
            )
        })?;
    if !exists {
        return Ok(Vec::new());
    }
    let rows = client
        .query(
            "SELECT ordinal, state_tag, schema_epoch, contract_sha256, reader_contract_sha256, \
                    predecessor_sha256, row_sha256 \
             FROM babylon_meta.persistence_authority_ledger ORDER BY ordinal LIMIT 3",
            &[],
        )
        .map_err(|error| {
            RustPersistenceActivationErrorV2::postgres("read predecessor authority ledger", &error)
        })?;
    let mut decoded = Vec::new();
    decoded
        .try_reserve_exact(rows.len())
        .map_err(|_| RustPersistenceActivationErrorV2::LedgerEncoding)?;
    for row in rows {
        let ordinal: i16 = row
            .try_get(0)
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let state_tag: i16 = row
            .try_get(1)
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let schema_epoch: i16 = row
            .try_get(2)
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let contract: Vec<u8> = row
            .try_get(3)
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let reader: Vec<u8> = row
            .try_get(4)
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let predecessor: Option<Vec<u8>> = row
            .try_get(5)
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let stored_sha: Vec<u8> = row
            .try_get(6)
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let decoded_row = PredecessorAuthorityLedgerRowV2::compose(
            u16::try_from(ordinal)
                .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?,
            u8::try_from(state_tag)
                .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?,
            u16::try_from(schema_epoch)
                .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?,
            contract
                .try_into()
                .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?,
            reader
                .try_into()
                .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?,
            predecessor
                .map(|bytes| {
                    bytes
                        .try_into()
                        .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)
                })
                .transpose()?,
        )?;
        if stored_sha.as_slice() != decoded_row.row_sha256 {
            return Err(RustPersistenceActivationErrorV2::AuthorityLedgerMismatch);
        }
        decoded.push(decoded_row);
    }
    Ok(decoded)
}

/// Commit the dedicated preparation and one-way V2 activation migrations.
///
/// # Errors
/// Returns a closed bootstrap, target, database, ledger, or cleanup refusal.
pub fn activate_rust_persistence_v2(
    config: &Config,
) -> Result<ActivationReportV2, RustPersistenceActivationErrorV2> {
    validate_legacy_connection_target(config)
        .map_err(|_| RustPersistenceActivationErrorV2::ConnectionTarget)?;
    preflight_v2_activation_before_mutation(config)?;
    establish_predecessor_authority_v2(config)?;
    let mut client = config
        .connect(NoTls)
        .map_err(|error| RustPersistenceActivationErrorV2::postgres("connect", &error))?;
    acquire_lock(&mut client).map_err(RustPersistenceActivationErrorV2::Lock)?;
    let result = activate_v2_under_lock(&mut client);
    let cleanup = release_lock(&mut client).map_err(RustPersistenceActivationErrorV2::Cleanup);
    match cleanup {
        Ok(()) => result,
        Err(error) => Err(error),
    }
}

fn activate_v2_under_lock(
    client: &mut postgres::Client,
) -> Result<ActivationReportV2, RustPersistenceActivationErrorV2> {
    let predecessor = read_predecessor_active_hash(client)?;
    let migrations = compiled_committed_tick_v2_activation_migrations()
        .map_err(|_| RustPersistenceActivationErrorV2::MigrationRegistry)?;
    let (contract_sha256, reader_contract_sha256) = v2_authority_contract_digests(&migrations);
    let expected_prepared = CommittedTickAuthorityLedgerRowV2::compose(
        1,
        CommittedTickAuthorityStateV2::Prepared,
        10,
        contract_sha256,
        reader_contract_sha256,
        predecessor,
    )?;
    let expected_active = CommittedTickAuthorityLedgerRowV2::compose(
        2,
        CommittedTickAuthorityStateV2::Active,
        11,
        contract_sha256,
        reader_contract_sha256,
        expected_prepared.row_sha256,
    )?;
    let mut observed = read_v2_authority_ledger(client)?;
    if observed.is_empty() {
        let commit_result = execute_v2_activation_migration(
            client,
            migrations[0],
            &expected_prepared,
            "migration 10",
        );
        observed = read_v2_authority_ledger(client)?;
        if observed.as_slice() != [expected_prepared.clone()] {
            if let Err(error) = commit_result {
                return Err(error.at_operation("unresolved preparation commit"));
            }
        }
    }
    if observed.as_slice() == [expected_prepared.clone()] {
        let commit_result = execute_v2_activation_migration(
            client,
            migrations[1],
            &expected_active,
            "migration 11",
        );
        observed = read_v2_authority_ledger(client)?;
        if observed.as_slice() != [expected_prepared.clone(), expected_active.clone()] {
            if let Err(error) = commit_result {
                return Err(error.at_operation("unresolved activation commit"));
            }
        }
    }
    if observed != [expected_prepared.clone(), expected_active.clone()] {
        return Err(RustPersistenceActivationErrorV2::AuthorityLedgerMismatch);
    }
    Ok(ActivationReportV2 {
        prepared_row: expected_prepared,
        active_row: expected_active,
    })
}

fn execute_v2_activation_migration(
    client: &mut postgres::Client,
    migration: SchemaMigration,
    authority_row: &CommittedTickAuthorityLedgerRowV2,
    operation: &'static str,
) -> Result<(), RustPersistenceActivationErrorV2> {
    let mut transaction = client
        .transaction()
        .map_err(|error| RustPersistenceActivationErrorV2::postgres(operation, &error))?;
    transaction
        .batch_execute(SERIALIZABLE_ACTIVATION_SETTINGS_V2)
        .map_err(|error| RustPersistenceActivationErrorV2::postgres(operation, &error))?;
    transaction
        .batch_execute(migration.sql())
        .map_err(|error| RustPersistenceActivationErrorV2::postgres(operation, &error))?;
    insert_v2_authority_row(&mut transaction, authority_row)?;
    transaction
        .commit()
        .map_err(|error| RustPersistenceActivationErrorV2::postgres(operation, &error))
}

fn insert_v2_authority_row(
    client: &mut impl GenericClient,
    row: &CommittedTickAuthorityLedgerRowV2,
) -> Result<(), RustPersistenceActivationErrorV2> {
    let affected = client
        .execute(
            "INSERT INTO babylon_meta.committed_tick_v2_authority_ledger \
             (ordinal, state_tag, activation_epoch, contract_sha256, reader_contract_sha256, \
              predecessor_sha256, row_sha256) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &i16::try_from(row.ordinal)
                    .map_err(|_| RustPersistenceActivationErrorV2::LedgerEncoding)?,
                &i16::from(row.state.tag()),
                &i16::try_from(row.activation_epoch)
                    .map_err(|_| RustPersistenceActivationErrorV2::LedgerEncoding)?,
                &&row.contract_sha256[..],
                &&row.reader_contract_sha256[..],
                &&row.predecessor_sha256[..],
                &&row.row_sha256[..],
            ],
        )
        .map_err(|error| {
            RustPersistenceActivationErrorV2::postgres("insert authority ledger row", &error)
        })?;
    if affected != 1 {
        return Err(RustPersistenceActivationErrorV2::AuthorityLedgerMismatch);
    }
    Ok(())
}

fn read_v2_authority_ledger(
    client: &mut impl GenericClient,
) -> Result<Vec<CommittedTickAuthorityLedgerRowV2>, RustPersistenceActivationErrorV2> {
    let exists: bool = client
        .query_one(
            "SELECT pg_catalog.to_regclass('babylon_meta.committed_tick_v2_authority_ledger') IS NOT NULL",
            &[],
        )
        .and_then(|row| row.try_get(0))
        .map_err(|error| {
            RustPersistenceActivationErrorV2::postgres("locate authority ledger", &error)
        })?;
    if !exists {
        return Ok(Vec::new());
    }
    let rows = client
        .query(
            "SELECT ordinal, state_tag, activation_epoch, contract_sha256, reader_contract_sha256, \
                    predecessor_sha256, row_sha256 \
             FROM babylon_meta.committed_tick_v2_authority_ledger ORDER BY ordinal LIMIT 3",
            &[],
        )
        .map_err(|error| {
            RustPersistenceActivationErrorV2::postgres("read authority ledger", &error)
        })?;
    let mut decoded = Vec::new();
    decoded
        .try_reserve_exact(rows.len())
        .map_err(|_| RustPersistenceActivationErrorV2::LedgerEncoding)?;
    for row in rows {
        let ordinal: i16 = row
            .try_get(0)
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let state_tag: i16 = row
            .try_get(1)
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let activation_epoch: i16 = row
            .try_get(2)
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let contract: Vec<u8> = row
            .try_get(3)
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let reader: Vec<u8> = row
            .try_get(4)
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let predecessor: Vec<u8> = row
            .try_get(5)
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let stored_sha: Vec<u8> = row
            .try_get(6)
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let state = match state_tag {
            1 => CommittedTickAuthorityStateV2::Prepared,
            2 => CommittedTickAuthorityStateV2::Active,
            _ => return Err(RustPersistenceActivationErrorV2::AuthorityLedgerMismatch),
        };
        let contract: [u8; 32] = contract
            .try_into()
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let reader: [u8; 32] = reader
            .try_into()
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let predecessor = predecessor
            .try_into()
            .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?;
        let decoded_row = CommittedTickAuthorityLedgerRowV2::compose(
            u16::try_from(ordinal)
                .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?,
            state,
            u16::try_from(activation_epoch)
                .map_err(|_| RustPersistenceActivationErrorV2::AuthorityLedgerMismatch)?,
            contract,
            reader,
            predecessor,
        )?;
        if stored_sha.as_slice() != decoded_row.row_sha256 {
            return Err(RustPersistenceActivationErrorV2::AuthorityLedgerMismatch);
        }
        decoded.push(decoded_row);
    }
    Ok(decoded)
}

fn read_predecessor_active_hash(
    client: &mut impl GenericClient,
) -> Result<[u8; 32], RustPersistenceActivationErrorV2> {
    let expected_prepared = PredecessorAuthorityLedgerRowV2::prepared()?;
    let expected_active = PredecessorAuthorityLedgerRowV2::active(&expected_prepared)?;
    let observed = read_predecessor_authority_ledger_v2(client)?;
    if observed != [expected_prepared, expected_active.clone()] {
        return Err(RustPersistenceActivationErrorV2::AuthorityLedgerMismatch);
    }
    Ok(expected_active.row_sha256)
}

/// A checked refusal while deriving durable inputs from one identified tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustPersistenceRuntimeErrorV2 {
    /// The terminal Rust-active ledger row is absent or does not match this binary.
    ActivationRequired,
    /// A local-only runtime connection or transaction operation failed.
    Database {
        /// Stable operation name without caller-supplied text.
        operation: &'static str,
        /// Bounded secret-safe driver diagnostic, when the failure came from `PostgreSQL`.
        diagnostic: Option<PostgresDiagnosticV1>,
    },
    /// A requested campaign foundation is absent.
    FoundationAbsent,
    /// Durable campaign bytes differ from the requested exact foundation.
    CampaignConflict,
    /// The content bundle's scenario does not reproduce the session's captured graph.
    FoundationScenarioMismatch,
    /// This binary cannot yet reconstruct the named nonzero durable tick.
    RestartUnavailable {
        /// Exact acknowledged tick which requires reconstruction.
        last_committed_tick: u64,
    },
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
    SemanticEnvelope(CommittedTickEnvelopeErrorV2),
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
    TerritoryCountyMap(crate::territory_county_map::TerritoryCountyMapErrorV1),
    /// The additive semantic Archive schema refused installation.
    SemanticArchive(crate::SemanticArchiveErrorV1),
    /// Foundation knowledge-grant seeding refused fixture drift or a grant conflict.
    FoundationGrants(crate::archive_foundation_grants::FoundationGrantsErrorV1),
}

impl From<SemanticBatchErrorV2> for RustPersistenceRuntimeErrorV2 {
    fn from(value: SemanticBatchErrorV2) -> Self {
        match value {
            SemanticBatchErrorV2::Codec(_) => Self::SemanticCodec,
            SemanticBatchErrorV2::Envelope(error) => Self::SemanticEnvelope(error),
            SemanticBatchErrorV2::CapacityOverflow { field } => Self::CapacityOverflow { field },
            SemanticBatchErrorV2::IntegerConversion { field, value } => {
                Self::IntegerConversion { field, value }
            }
            SemanticBatchErrorV2::Allocation { field, requested } => {
                Self::Allocation { field, requested }
            }
        }
    }
}

impl From<SemanticCodecErrorV1> for RustPersistenceRuntimeErrorV2 {
    fn from(value: SemanticCodecErrorV1) -> Self {
        match value {
            SemanticCodecErrorV1::CapacityOverflow { field } => Self::CapacityOverflow { field },
            SemanticCodecErrorV1::IntegerConversion { field, value } => {
                Self::IntegerConversion { field, value }
            }
            SemanticCodecErrorV1::Allocation { field, requested } => {
                Self::Allocation { field, requested }
            }
            SemanticCodecErrorV1::ByteLimit { .. }
            | SemanticCodecErrorV1::Refusal(_)
            | SemanticCodecErrorV1::Invalid(_) => Self::SemanticCodec,
        }
    }
}

impl std::fmt::Display for RustPersistenceRuntimeErrorV2 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Rust persistence runtime refused: {self:?}")
    }
}

impl std::error::Error for RustPersistenceRuntimeErrorV2 {}

impl RustPersistenceRuntimeErrorV2 {
    pub(crate) fn database(operation: &'static str) -> Self {
        Self::Database {
            operation,
            diagnostic: None,
        }
    }

    pub(crate) fn postgres(operation: &'static str, error: &postgres::Error) -> Self {
        Self::Database {
            operation,
            diagnostic: Some(PostgresDiagnosticV1::capture(error)),
        }
    }
}

/// Bounded durable observation returned only after marker-last acknowledgement.
///
/// This intentionally excludes write identities, values, database coordinates,
/// and other detailed persistence evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedTickReceiptV2 {
    resolve_tick: CommittedResolveTickV1,
    commit_disposition: ReplayCommitDispositionV1,
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
    tick_content_hash: TickContentHashV1,
}

impl CommittedTickReceiptV2 {
    fn from_acknowledged_with_choices(
        resolve_tick: CommittedResolveTickV1,
        commit_disposition: ReplayCommitDispositionV1,
        acknowledged: IdentifiedTickReportV2,
    ) -> (Self, Vec<ChoiceReceiptV1>) {
        let prior_stable_graph_digest = acknowledged.prior_stable_graph_digest().into_bytes();
        let result_stable_graph_digest = acknowledged.result_stable_graph_digest().into_bytes();
        let event_count = acknowledged.successful_event_batch().events().len();
        let event_digest = acknowledged.successful_event_batch().source_digest();
        let choice_receipt_count = acknowledged.report().choice_receipts.len();
        let choice_receipt_digest = acknowledged.choice_receipt_source_digest();
        let material_row_count = acknowledged.material_state_rows().source_count();
        let material_row_digest = acknowledged.material_state_rows().source_digest();
        let tick_content_hash = acknowledged.tick_content_hash();
        let babylon_tick::TickReport {
            before: graph_before,
            after: graph_after,
            world_before,
            world_after,
            considered,
            fired,
            per_rule_considered,
            per_rule_fired,
            audit_receipts,
            choice_receipts,
            committed_events: _,
        } = acknowledged.into_report();
        (
            Self {
                resolve_tick,
                commit_disposition,
                graph_before,
                graph_after,
                prior_stable_graph_digest,
                result_stable_graph_digest,
                world_before,
                world_after,
                considered,
                fired,
                per_rule_considered,
                per_rule_fired,
                event_count,
                event_digest,
                choice_receipt_count,
                choice_receipt_digest,
                audit_receipt_count: audit_receipts.len(),
                material_row_count,
                material_row_digest,
                tick_content_hash,
            },
            choice_receipts,
        )
    }

    /// Return the one-based durable tick.
    #[must_use]
    pub const fn resolve_tick(&self) -> CommittedResolveTickV1 {
        self.resolve_tick
    }

    /// Return how `PostgreSQL` acknowledgement was established.
    #[must_use]
    pub const fn commit_disposition(&self) -> ReplayCommitDispositionV1 {
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
    pub const fn tick_content_hash(&self) -> TickContentHashV1 {
        self.tick_content_hash
    }
}

/// Sole production owner of a replay session and its Rust persistence authority.
pub struct DurableReplayRuntimeV2<G> {
    config: Config,
    campaign_id: CampaignId,
    session: ReplayTickSession<G>,
    foundation: CampaignFoundationV1,
    activation_row: CommittedTickAuthorityLedgerRowV2,
    last_committed_tick: Option<CommittedResolveTickV1>,
    last_choice_receipts: Vec<ChoiceReceiptV1>,
}

impl DurableReplayRuntimeV2<HypergraphStore> {
    /// Capture and durably install a new tick-zero campaign under active Rust authority.
    ///
    /// # Errors
    /// Refuses absent authority, a nonzero session, an exact campaign conflict, or a database
    /// failure before exposing a runtime.
    pub fn create(
        config: &Config,
        campaign_id: CampaignId,
        session: ReplayTickSession<HypergraphStore>,
        content_bundle: FoundationContentBundleV1,
    ) -> Result<Self, RustPersistenceRuntimeErrorV2> {
        let activation_row = require_active_authority(config)?;
        let foundation = CampaignFoundationV1::capture(&session, content_bundle)?;
        persist_campaign_foundation_v1(config, campaign_id, &foundation)?;
        if let Some(last) = read_last_committed_tick_v1(config, campaign_id)? {
            return Err(RustPersistenceRuntimeErrorV2::RestartUnavailable {
                last_committed_tick: last.get(),
            });
        }
        Ok(Self {
            config: config.clone(),
            campaign_id,
            session,
            foundation,
            activation_row,
            last_committed_tick: None,
            last_choice_receipts: Vec::new(),
        })
    }

    /// Reconstruct a durable campaign from its latest exact full checkpoint.
    ///
    /// # Errors
    /// Refuses absent authority or foundation, any exact replay-source mismatch, a database
    /// failure, or a full-checkpoint section that cannot reproduce typed state.
    pub fn open(
        config: &Config,
        campaign_id: CampaignId,
    ) -> Result<Self, RustPersistenceRuntimeErrorV2> {
        let activation_row = require_active_authority(config)?;
        let foundation = hydrate_campaign_foundation_v1(config, campaign_id)?;
        {
            let mut client = config.connect(NoTls).map_err(|error| {
                RustPersistenceRuntimeErrorV2::postgres("connect campaign owner reader", &error)
            })?;
            require_graph_campaign_owner_v2(&mut client, campaign_id)?;
        }
        let session = reconstruct_graph_foundation_session_v1(&foundation)?;
        let bundle = foundation.content_bundle();
        let scenario = std::str::from_utf8(bundle.scenario_source_bytes())
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        let prelude = bundle
            .prelude_source_bytes()
            .map(std::str::from_utf8)
            .transpose()
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        // Upgrade path for campaigns founded before the declared mapping
        // existed: install the additive schema and reconcile the rows
        // idempotently. Divergence between stored and declared rows refuses
        // loudly; durable rows are never overwritten.
        crate::territory_county_map::reconcile_territory_county_map_v1(
            config,
            campaign_id,
            scenario,
            prelude,
        )
        .map_err(RustPersistenceRuntimeErrorV2::TerritoryCountyMap)?;
        let (session, last_committed_tick) = replay_durable_tail_v1(config, campaign_id, session)?;
        validate_campaign_catalog_tail_v1(config, campaign_id, last_committed_tick)?;
        Ok(Self {
            config: config.clone(),
            campaign_id,
            session,
            foundation,
            activation_row,
            last_committed_tick,
            last_choice_receipts: Vec::new(),
        })
    }

    /// Adjudicate one detached candidate, commit typed rows marker-last, then publish it.
    ///
    /// # Errors
    /// Every replay, semantic, database, retry, or acknowledgement refusal leaves the caller
    /// sink and live session tick unchanged.
    pub fn advance_and_commit(
        &mut self,
        sink: &mut CollectingSink,
        actions: &OrderedPracticeActionBatchV1,
    ) -> Result<CommittedTickReceiptV2, RustPersistenceRuntimeErrorV2> {
        let candidate = self
            .session
            .prepare_advance(actions)
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplayTick)?;
        let prepared = prepare_committed_tick_v2(candidate.report())?;
        let checkpoint = CommittedFullCheckpointV1::capture(
            self.campaign_id,
            prepared.resolve_tick(),
            candidate.report(),
        )?;
        let resolve_tick = prepared.resolve_tick();
        let envelope = prepared.into_envelope(self.campaign_id)?;
        let config = &self.config;
        let campaign_id = self.campaign_id;
        let content_layout = self.foundation.content_bundle().layout();
        let (acknowledged, disposition) = self
            .session
            .commit_prepared_and_publish(sink, candidate, |report| {
                commit_typed_tick_v2(
                    config,
                    campaign_id,
                    report,
                    &checkpoint,
                    &envelope,
                    content_layout,
                )
            })
            .map_err(|error| match error {
                PreparedReplayCommitErrorV1::Preflight(_) => {
                    RustPersistenceRuntimeErrorV2::ReplayTick
                }
                PreparedReplayCommitErrorV1::Commit(error) => error,
            })?;
        let (receipt, choice_receipts) = CommittedTickReceiptV2::from_acknowledged_with_choices(
            resolve_tick,
            disposition,
            acknowledged,
        );
        self.last_choice_receipts = choice_receipts;
        self.last_committed_tick = Some(resolve_tick);
        Ok(receipt)
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
    pub fn observe_committed_graph_state_v1(
        &self,
        receipt: &CommittedTickReceiptV2,
    ) -> Result<StableGraphStateV1, RustPersistenceRuntimeErrorV2> {
        if self.last_committed_tick != Some(receipt.resolve_tick()) {
            return Err(
                RustPersistenceRuntimeErrorV2::ObservationNotCurrentCommittedTail {
                    receipt_tick: receipt.resolve_tick().get(),
                    current_tail: self.last_committed_tick.map(CommittedResolveTickV1::get),
                },
            );
        }
        let observed = self.observe_current_stable_graph_state_v1()?;
        if observed.digest().as_bytes() != &receipt.result_stable_graph_digest() {
            return Err(RustPersistenceRuntimeErrorV2::ObservationGraphDigestMismatch);
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
    pub fn observe_committed_choice_receipts_v1(
        &self,
        receipt: &CommittedTickReceiptV2,
    ) -> Result<&[ChoiceReceiptV1], RustPersistenceRuntimeErrorV2> {
        if self.last_committed_tick != Some(receipt.resolve_tick()) {
            return Err(
                RustPersistenceRuntimeErrorV2::ObservationNotCurrentCommittedTail {
                    receipt_tick: receipt.resolve_tick().get(),
                    current_tail: self.last_committed_tick.map(CommittedResolveTickV1::get),
                },
            );
        }
        if self.last_choice_receipts.len() != receipt.choice_receipt_count() {
            return Err(RustPersistenceRuntimeErrorV2::ObservationChoiceReceiptUnavailable);
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
    pub fn observe_current_stable_graph_state_v1(
        &self,
    ) -> Result<StableGraphStateV1, RustPersistenceRuntimeErrorV2> {
        self.session
            .stable_graph_state()
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplayTick)
    }

    /// Borrow the exact durable campaign foundation.
    #[must_use]
    pub const fn foundation(&self) -> &CampaignFoundationV1 {
        &self.foundation
    }

    /// Borrow the terminal authority row used to construct this runtime.
    #[must_use]
    pub const fn activation_row(&self) -> &CommittedTickAuthorityLedgerRowV2 {
        &self.activation_row
    }

    /// Return the last marker acknowledged by this runtime.
    #[must_use]
    pub const fn last_committed_tick(&self) -> Option<CommittedResolveTickV1> {
        self.last_committed_tick
    }
}

/// Hydrate one exact durable foundation under terminal Rust authority.
///
/// # Errors
/// Returns an authority, target, database, absence, digest, or replay-source refusal.
pub fn hydrate_campaign_foundation_v1(
    config: &Config,
    campaign_id: CampaignId,
) -> Result<CampaignFoundationV1, RustPersistenceRuntimeErrorV2> {
    let _active = require_active_authority(config)?;
    validate_legacy_connection_target(config)
        .map_err(|_| RustPersistenceRuntimeErrorV2::database("validate foundation target"))?;
    let mut client = config.connect(NoTls).map_err(|error| {
        RustPersistenceRuntimeErrorV2::postgres("connect foundation reader", &error)
    })?;
    require_active_authority_client(&mut client)?;
    crate::foundation_content_schema::install_foundation_content_schema_v2(&mut client)?;
    hydrate_campaign_foundation_client_v1(&mut client, campaign_id)
}

/// Rebuild the exact stored tick-zero graph and verify all captured components.
/// The immutable H3 reference remains the existing admitted reference; scenario,
/// rules, defines, session identity and seed come from the durable foundation.
pub(crate) fn reconstruct_graph_foundation_session_v1(
    foundation: &CampaignFoundationV1,
) -> Result<ReplayTickSession<HypergraphStore>, RustPersistenceRuntimeErrorV2> {
    let bundle = foundation.content_bundle();
    let scenario = std::str::from_utf8(bundle.scenario_source_bytes())
        .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
    let prelude = bundle
        .prelude_source_bytes()
        .map(std::str::from_utf8)
        .transpose()
        .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
    let rules = std::str::from_utf8(bundle.rule_source_bytes())
        .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
    let material = MaterialStateV1::try_new(
        michigan_dynamic_hex_foundation_v1()
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?,
    )
    .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
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
    .map_err(|_| RustPersistenceRuntimeErrorV2::ReplayTick)?;
    let verification_bundle = FoundationContentBundle::try_new(
        bundle.layout(),
        scenario,
        prelude,
        rules,
        bundle.defines_bytes(),
        bundle.reference_bundle_manifest_bytes(),
    )?;
    let verification = CampaignFoundationV1::capture_content(&session, verification_bundle)?;
    if verification.canonical_bytes() != foundation.canonical_bytes() {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    Ok(session)
}

pub(crate) fn hydrate_campaign_foundation_client_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
) -> Result<CampaignFoundationV1, RustPersistenceRuntimeErrorV2> {
    let row = client
        .query_opt(
            "SELECT stable_graph, world_registers, resolver_manifest, prepared_environment, \
                    replay_session_id, rng_seed, defines_hash, rules_hash, ref_digest, \
                    scenario_source, prelude_source, rule_source, defines_bytes, \
                    reference_manifest_bytes, foundation_sha256, content_layout_version \
             FROM babylon_state.campaign_foundation \
             JOIN babylon_state.campaign_foundation_content_layout_v2 USING (campaign_id) \
             WHERE campaign_id = $1::uuid",
            &[campaign_id.as_uuid()],
        )
        .map_err(|error| {
            RustPersistenceRuntimeErrorV2::postgres("read campaign foundation", &error)
        })?
        .ok_or(RustPersistenceRuntimeErrorV2::FoundationAbsent)?;
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
    CampaignFoundationV1::from_persisted(
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
        FoundationContentLayout::from_persisted(decode_runtime_column(&row, 15)?)?,
    )
}

pub(crate) fn require_active_authority(
    config: &Config,
) -> Result<CommittedTickAuthorityLedgerRowV2, RustPersistenceRuntimeErrorV2> {
    validate_legacy_connection_target(config)
        .map_err(|_| RustPersistenceRuntimeErrorV2::ActivationRequired)?;
    let mut client = config.connect(NoTls).map_err(|error| {
        RustPersistenceRuntimeErrorV2::postgres("connect authority reader", &error)
    })?;
    require_active_authority_client(&mut client)
}

pub(crate) fn require_active_authority_client(
    client: &mut impl GenericClient,
) -> Result<CommittedTickAuthorityLedgerRowV2, RustPersistenceRuntimeErrorV2> {
    let predecessor = read_predecessor_active_hash(client).map_err(runtime_authority_error_v2)?;
    let migrations = compiled_committed_tick_v2_activation_migrations()
        .map_err(|_| RustPersistenceRuntimeErrorV2::ActivationRequired)?;
    let (contract_sha256, reader_contract_sha256) = v2_authority_contract_digests(&migrations);
    let expected_prepared = CommittedTickAuthorityLedgerRowV2::compose(
        1,
        CommittedTickAuthorityStateV2::Prepared,
        10,
        contract_sha256,
        reader_contract_sha256,
        predecessor,
    )
    .map_err(|_| RustPersistenceRuntimeErrorV2::ActivationRequired)?;
    let expected_active = CommittedTickAuthorityLedgerRowV2::compose(
        2,
        CommittedTickAuthorityStateV2::Active,
        11,
        contract_sha256,
        reader_contract_sha256,
        expected_prepared.row_sha256,
    )
    .map_err(|_| RustPersistenceRuntimeErrorV2::ActivationRequired)?;
    let observed = read_v2_authority_ledger(client).map_err(runtime_authority_error_v2)?;
    if observed != [expected_prepared, expected_active.clone()] {
        return Err(RustPersistenceRuntimeErrorV2::ActivationRequired);
    }
    Ok(expected_active)
}

fn runtime_authority_error_v2(
    error: RustPersistenceActivationErrorV2,
) -> RustPersistenceRuntimeErrorV2 {
    match error {
        RustPersistenceActivationErrorV2::Database {
            operation,
            diagnostic,
        } => RustPersistenceRuntimeErrorV2::Database {
            operation,
            diagnostic,
        },
        _ => RustPersistenceRuntimeErrorV2::ActivationRequired,
    }
}

fn persist_campaign_foundation_v1(
    config: &Config,
    campaign_id: CampaignId,
    foundation: &CampaignFoundationV1,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    validate_legacy_connection_target(config).map_err(|_| {
        RustPersistenceRuntimeErrorV2::database("validate foundation writer target")
    })?;
    crate::territory_county_map::install_territory_county_map_schema_v1(config)
        .map_err(RustPersistenceRuntimeErrorV2::TerritoryCountyMap)?;
    SemanticArchiveStoreV1::new(config)
        .install_schema()
        .map_err(RustPersistenceRuntimeErrorV2::SemanticArchive)?;
    let mut client = config.connect(NoTls).map_err(|error| {
        RustPersistenceRuntimeErrorV2::postgres("connect foundation writer", &error)
    })?;
    require_active_authority_client(&mut client)?;
    crate::foundation_content_schema::install_foundation_content_schema_v2(&mut client)?;
    let mut transaction = client.transaction().map_err(|error| {
        RustPersistenceRuntimeErrorV2::postgres("begin foundation writer", &error)
    })?;
    transaction
        .batch_execute("SET LOCAL search_path TO pg_catalog; SET LOCAL synchronous_commit TO on")
        .map_err(|error| {
            RustPersistenceRuntimeErrorV2::postgres("foundation writer settings", &error)
        })?;
    // Material creation uses the same founding lock. Hold it through graph
    // insertion so its absence check cannot adopt a concurrent graph campaign.
    transaction
        .query_one(
            "SELECT pg_catalog.pg_advisory_xact_lock($1)",
            &[&crate::SCHEMA_ADVISORY_LOCK_KEY],
        )
        .map_err(|error| {
            RustPersistenceRuntimeErrorV2::postgres("lock campaign founding", &error)
        })?;
    require_graph_campaign_owner_v2(&mut transaction, campaign_id)?;
    insert_campaign_foundation_rows_v1(&mut transaction, campaign_id, foundation)?;
    transaction.commit().map_err(|error| {
        RustPersistenceRuntimeErrorV2::postgres("commit campaign foundation", &error)
    })?;
    let hydrated = hydrate_campaign_foundation_v1(config, campaign_id)?;
    if hydrated.canonical_bytes() != foundation.canonical_bytes() {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    Ok(())
}

pub(crate) fn insert_campaign_foundation_rows_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    foundation: &CampaignFoundationV1,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let _active = require_active_authority_client(client)?;
    let replay_session = std::str::from_utf8(foundation.replay_session_identity().as_bytes())
        .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
    let bundle = foundation.content_bundle();
    let base_reference_digest = base_reference_digest_v1(
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
            RustPersistenceRuntimeErrorV2::postgres("insert campaign identity", &error)
        })?;
    let scenario = std::str::from_utf8(bundle.scenario_source_bytes())
        .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
    let prelude = bundle
        .prelude_source_bytes()
        .map(std::str::from_utf8)
        .transpose()
        .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
    let rules = std::str::from_utf8(bundle.rule_source_bytes())
        .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
    let territory_county_map =
        crate::territory_county_map::extract_declared_territory_county_map_v1(scenario, prelude)
            .map_err(RustPersistenceRuntimeErrorV2::TerritoryCountyMap)?;
    let foundation_sha256 = sha256_of(foundation.canonical_bytes());
    let inserted = client
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
            RustPersistenceRuntimeErrorV2::postgres("insert campaign foundation", &error)
        })?;
    persist_foundation_content_layout_v2(client, campaign_id, bundle.layout(), inserted == 1)?;
    let stored_sha: Vec<u8> = client
        .query_one(
            "SELECT foundation_sha256 FROM babylon_state.campaign_foundation \
             WHERE campaign_id = $1::uuid",
            &[campaign_id.as_uuid()],
        )
        .and_then(|row| row.try_get(0))
        .map_err(|error| {
            RustPersistenceRuntimeErrorV2::postgres("verify campaign foundation", &error)
        })?;
    if stored_sha.as_slice() != foundation_sha256 {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    ensure_campaign_catalog_row_v1(client, campaign_id, foundation)?;
    crate::territory_county_map::insert_territory_county_map_rows_v1(
        client,
        campaign_id,
        &territory_county_map,
    )
    .map_err(RustPersistenceRuntimeErrorV2::TerritoryCountyMap)?;
    crate::archive_foundation_grants::seed_foundation_grants_v1(client, campaign_id)
        .map_err(RustPersistenceRuntimeErrorV2::FoundationGrants)?;
    crate::archive_revision::schema::enroll_foundation(client, campaign_id, inserted == 1)
        .map_err(RustPersistenceRuntimeErrorV2::SemanticArchive)?;
    Ok(())
}

fn persist_foundation_content_layout_v2(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    layout: FoundationContentLayout,
    foundation_inserted: bool,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    if foundation_inserted {
        client
            .execute(
                "INSERT INTO babylon_state.campaign_foundation_content_layout_v2 \
             (campaign_id, content_layout_version) VALUES ($1::uuid, $2)",
                &[campaign_id.as_uuid(), &layout.version()],
            )
            .map_err(|error| {
                RustPersistenceRuntimeErrorV2::postgres("insert foundation content layout", &error)
            })?;
    }
    let stored_layout: i16 = client.query_one(
        "SELECT content_layout_version FROM babylon_state.campaign_foundation_content_layout_v2 \
         WHERE campaign_id = $1::uuid", &[campaign_id.as_uuid()],
    ).and_then(|row| row.try_get(0))
        .map_err(|error| RustPersistenceRuntimeErrorV2::postgres("verify foundation content layout", &error))?;
    if FoundationContentLayout::from_persisted(stored_layout)? != layout {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    Ok(())
}

fn base_reference_digest_v1(
    reference_manifest: &[u8],
    expected_bundle_digest: babylon_kernel::tick_content_hash::RefDigestV1,
) -> Result<[u8; 32], RustPersistenceRuntimeErrorV2> {
    let expected_len = REFERENCE_BUNDLE_DOMAIN_V1
        .len()
        .checked_add(64)
        .ok_or(RustPersistenceRuntimeErrorV2::ReplaySource)?;
    if reference_manifest.len() != expected_len
        || !reference_manifest.starts_with(REFERENCE_BUNDLE_DOMAIN_V1)
        || sha256_of(reference_manifest) != *expected_bundle_digest.as_bytes()
    {
        return Err(RustPersistenceRuntimeErrorV2::ReplaySource);
    }
    reference_manifest[REFERENCE_BUNDLE_DOMAIN_V1.len()..REFERENCE_BUNDLE_DOMAIN_V1.len() + 32]
        .try_into()
        .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)
}

fn read_last_committed_tick_v1(
    config: &Config,
    campaign_id: CampaignId,
) -> Result<Option<CommittedResolveTickV1>, RustPersistenceRuntimeErrorV2> {
    let mut client = config.connect(NoTls).map_err(|error| {
        RustPersistenceRuntimeErrorV2::postgres("connect marker reader", &error)
    })?;
    let row = client
        .query_opt(
            "SELECT resolve_tick FROM babylon_state.tick_commit \
             WHERE campaign_id = $1::uuid ORDER BY resolve_tick DESC LIMIT 1",
            &[campaign_id.as_uuid()],
        )
        .map_err(|error| RustPersistenceRuntimeErrorV2::postgres("read last marker", &error))?;
    row.map(|row| {
        let raw: i64 = decode_runtime_column(&row, 0)?;
        let raw =
            u64::try_from(raw).map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
        CommittedResolveTickV1::try_from(raw)
            .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)
    })
    .transpose()
}

fn validate_campaign_catalog_tail_v1(
    config: &Config,
    campaign_id: CampaignId,
    last_committed_tick: Option<CommittedResolveTickV1>,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let mut client = config.connect(NoTls).map_err(|error| {
        RustPersistenceRuntimeErrorV2::postgres("connect retained campaign catalog reader", &error)
    })?;
    let catalog = read_campaign_catalog_row_v1(&mut client, campaign_id)?
        .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    let expected = last_committed_tick.map_or(0, CommittedResolveTickV1::get);
    if catalog.last_tick() == expected {
        Ok(())
    } else {
        Err(RustPersistenceRuntimeErrorV2::CampaignConflict)
    }
}

/// Inspect the existing owner relation; never attach or replace an owner.
/// Writers call this while holding the campaign/founding lock. A pre-material
/// schema has no registered material campaigns and remains a valid graph estate.
fn require_graph_campaign_owner_v2(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let installed: bool = client.query_one(
        "SELECT pg_catalog.to_regclass('babylon_state.material_campaign_foundation_v2') IS NOT NULL", &[],
    ).and_then(|row| row.try_get(0))
        .map_err(|error| RustPersistenceRuntimeErrorV2::postgres("inspect material owner relation", &error))?;
    if installed {
        let material: bool = client.query_one(
            "SELECT EXISTS (SELECT 1 FROM babylon_state.material_campaign_foundation_v2 WHERE campaign_id=$1::uuid)",
            &[campaign_id.as_uuid()],
        ).and_then(|row| row.try_get(0))
            .map_err(|error| RustPersistenceRuntimeErrorV2::postgres("read campaign writer owner", &error))?;
        if material {
            return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
        }
    }
    Ok(())
}

fn commit_typed_tick_v2(
    config: &Config,
    campaign_id: CampaignId,
    report: &IdentifiedTickReportV2,
    checkpoint: &CommittedFullCheckpointV1,
    envelope: &CommittedTickEnvelopeV2,
    content_layout: FoundationContentLayout,
) -> Result<ReplayCommitDispositionV1, RustPersistenceRuntimeErrorV2> {
    let claim = envelope.claim();
    if claim.campaign_id() != campaign_id
        || claim.resolve_tick()
            != u64::try_from(report.result_registers().completed_tick())
                .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?
        || claim.tick_content_hash() != report.tick_content_hash()
        || checkpoint.sections().len() != 9
    {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    let resolve_tick = i64::try_from(claim.resolve_tick())
        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    validate_legacy_connection_target(config)
        .map_err(|_| RustPersistenceRuntimeErrorV2::database("validate typed tick target"))?;
    let mut client = config.connect(NoTls).map_err(|error| {
        RustPersistenceRuntimeErrorV2::postgres("connect typed tick writer", &error)
    })?;
    #[cfg(test)]
    let before_lock_barrier = LIVE_BEFORE_CAMPAIGN_LOCK_BARRIER
        .lock()
        .expect("live pre-lock barrier lock")
        .clone();
    #[cfg(test)]
    if let Some(barrier) = before_lock_barrier {
        barrier.wait();
    }
    let mut transaction = client
        .transaction()
        .map_err(|error| RustPersistenceRuntimeErrorV2::postgres("begin typed tick", &error))?;
    transaction
        .batch_execute("SET LOCAL search_path TO pg_catalog; SET LOCAL synchronous_commit TO on")
        .map_err(|error| RustPersistenceRuntimeErrorV2::postgres("typed tick settings", &error))?;
    let _active = require_active_authority_client(&mut transaction)?;
    let locked = transaction
        .query_opt(
            "SELECT campaign_id FROM babylon_state.campaign WHERE campaign_id = $1::uuid FOR UPDATE",
            &[campaign_id.as_uuid()],
        )
        .map_err(|error| RustPersistenceRuntimeErrorV2::postgres("lock campaign", &error))?;
    if locked.is_none() {
        return Err(RustPersistenceRuntimeErrorV2::FoundationAbsent);
    }
    require_graph_campaign_owner_v2(&mut transaction, campaign_id)?;
    crate::foundation_content_schema::lock_foundation_content_layout_v2(
        &mut transaction,
        campaign_id,
        content_layout,
    )?;
    // Another identical writer may have committed while this transaction waited for the campaign
    // row. Reconcile the exact marker and envelope under the acquired lock before interpreting the
    // advanced tail as a conflict.
    if marker_matches_envelope_v2(&mut transaction, report, envelope)? {
        return Ok(ReplayCommitDispositionV1::ReconciledAfterAmbiguousCommit);
    }
    let last_tick: Option<i64> = transaction
        .query_one(
            "SELECT pg_catalog.max(resolve_tick) FROM babylon_state.tick_commit WHERE campaign_id = $1::uuid",
            &[campaign_id.as_uuid()],
        )
        .and_then(|row| row.try_get(0))
        .map_err(|error| {
            RustPersistenceRuntimeErrorV2::postgres("read campaign marker tail", &error)
        })?;
    let expected_predecessor = resolve_tick
        .checked_sub(1)
        .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    if last_tick.unwrap_or(0) != expected_predecessor {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }

    insert_typed_tick_pre_marker_rows_v2(
        &mut transaction,
        campaign_id,
        resolve_tick,
        report,
        checkpoint,
        envelope.claim().tick_content_hash(),
    )?;
    commit_marker_last_v2(
        config,
        transaction,
        campaign_id,
        resolve_tick,
        report,
        envelope,
        content_layout,
    )
}

fn commit_marker_last_v2(
    config: &Config,
    mut transaction: postgres::Transaction<'_>,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReportV2,
    envelope: &CommittedTickEnvelopeV2,
    content_layout: FoundationContentLayout,
) -> Result<ReplayCommitDispositionV1, RustPersistenceRuntimeErrorV2> {
    #[cfg(test)]
    if LIVE_FAIL_BEFORE_MARKER.swap(false, std::sync::atomic::Ordering::SeqCst) {
        transaction.batch_execute("SELECT 1 / 0").map_err(|error| {
            RustPersistenceRuntimeErrorV2::postgres("injected pre-marker refusal", &error)
        })?;
    }

    let predecessor = resolve_tick
        .checked_sub(1)
        .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    advance_campaign_catalog_tick_v1(&mut transaction, campaign_id, predecessor, resolve_tick)?;

    // Constitutional visibility point: no durable row may be inserted after this marker.
    require_single_insert_v1(
        transaction.execute(
            "INSERT INTO babylon_state.tick_commit \
             (campaign_id, resolve_tick, envelope_layout_version, tick_content_hash, envelope_digest) \
             VALUES ($1::uuid, $2, 2, $3, $4)",
            &[
                campaign_id.as_uuid(),
                &resolve_tick,
                &&envelope.claim().tick_content_hash().as_bytes()[..],
                &&envelope.digest().as_bytes()[..],
            ],
        ),
        "insert marker last",
    )?;
    match commit_transaction_v1(transaction) {
        TickCommitAttemptV1::Acknowledged => Ok(ReplayCommitDispositionV1::Committed),
        TickCommitAttemptV1::Ambiguous { diagnostic } => {
            let mut client = config.connect(NoTls).map_err(|error| {
                RustPersistenceRuntimeErrorV2::postgres("reconnect ambiguous commit", &error)
            })?;
            let mut reconciliation = client.transaction().map_err(|error| {
                RustPersistenceRuntimeErrorV2::postgres("begin ambiguous reconciliation", &error)
            })?;
            let locked = reconciliation.query_opt(
                "SELECT campaign_id FROM babylon_state.campaign WHERE campaign_id=$1::uuid FOR UPDATE",
                &[campaign_id.as_uuid()],
            ).map_err(|error| RustPersistenceRuntimeErrorV2::postgres("lock reconciliation campaign", &error))?;
            if locked.is_none() {
                return Err(RustPersistenceRuntimeErrorV2::FoundationAbsent);
            }
            require_graph_campaign_owner_v2(&mut reconciliation, campaign_id)?;
            crate::foundation_content_schema::lock_foundation_content_layout_v2(
                &mut reconciliation,
                campaign_id,
                content_layout,
            )?;
            if marker_matches_envelope_v2(&mut reconciliation, report, envelope)? {
                #[cfg(test)]
                LIVE_RECONCILIATIONS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(ReplayCommitDispositionV1::ReconciledAfterAmbiguousCommit)
            } else {
                Err(RustPersistenceRuntimeErrorV2::Database {
                    operation: "unresolved ambiguous commit",
                    diagnostic,
                })
            }
        }
    }
}

pub(crate) fn insert_typed_tick_pre_marker_rows_v2(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReportV2,
    checkpoint: &CommittedFullCheckpointV1,
    tick_content_hash: TickContentHashV1,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let action_layout = i16::try_from(report.action_batch_layout_version())
        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    require_single_insert_v1(
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
    insert_typed_graph_rows_v1(client, campaign_id, resolve_tick, report)?;
    insert_typed_material_rows_v1(client, campaign_id, resolve_tick, report)?;
    insert_choice_receipt_rows_v1(client, campaign_id, resolve_tick, report)?;
    insert_typed_event_rows_v2(client, campaign_id, resolve_tick, report)?;
    insert_full_checkpoint_v1(client, campaign_id, resolve_tick, checkpoint)?;
    require_single_insert_v1(
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum TickCommitAttemptV1 {
    Acknowledged,
    Ambiguous {
        diagnostic: Option<PostgresDiagnosticV1>,
    },
}

fn commit_transaction_v1(transaction: postgres::Transaction<'_>) -> TickCommitAttemptV1 {
    if let Err(error) = transaction.commit() {
        return TickCommitAttemptV1::Ambiguous {
            diagnostic: Some(PostgresDiagnosticV1::capture(&error)),
        };
    }
    #[cfg(test)]
    if LIVE_COMMIT_AS_AMBIGUOUS.swap(false, std::sync::atomic::Ordering::SeqCst) {
        return TickCommitAttemptV1::Ambiguous { diagnostic: None };
    }
    TickCommitAttemptV1::Acknowledged
}

#[cfg(test)]
static LIVE_FAIL_BEFORE_MARKER: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
#[cfg(test)]
static LIVE_COMMIT_AS_AMBIGUOUS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
#[cfg(test)]
static LIVE_RECONCILIATIONS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
#[cfg(test)]
static LIVE_BEFORE_CAMPAIGN_LOCK_BARRIER: std::sync::Mutex<
    Option<std::sync::Arc<std::sync::Barrier>>,
> = std::sync::Mutex::new(None);

fn marker_matches_envelope_v2(
    client: &mut impl GenericClient,
    report: &IdentifiedTickReportV2,
    envelope: &CommittedTickEnvelopeV2,
) -> Result<bool, RustPersistenceRuntimeErrorV2> {
    let claim = envelope.claim();
    let Some(stored) = read_stored_typed_tick_v2(
        client,
        claim.campaign_id(),
        claim.resolve_tick(),
        report.result_stable_graph().scenario_scope(),
    )?
    else {
        return Ok(false);
    };
    let catalog = read_campaign_catalog_row_v1(client, claim.campaign_id())?
        .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    if catalog.last_tick() != claim.resolve_tick() {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    let action_layout = i16::try_from(report.action_batch_layout_version())
        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    if stored.action_layout() != action_layout
        || stored.action_digest().as_slice() != report.action_batch_digest().as_bytes()
        || stored.action_bytes() != report.action_batch_bytes()
    {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    envelope
        .classify_retry_against(stored.envelope())
        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    Ok(true)
}

fn replay_durable_tail_v1(
    config: &Config,
    campaign_id: CampaignId,
    mut session: ReplayTickSession<HypergraphStore>,
) -> Result<
    (
        ReplayTickSession<HypergraphStore>,
        Option<CommittedResolveTickV1>,
    ),
    RustPersistenceRuntimeErrorV2,
> {
    let mut client = config.connect(NoTls).map_err(|error| {
        RustPersistenceRuntimeErrorV2::postgres("connect restart reader", &error)
    })?;
    let Some(root) = select_durable_restart_root_v1(&mut client, campaign_id)? else {
        return Ok((session, None));
    };
    let scenario_scope =
        restore_durable_restart_root_v1(&mut client, campaign_id, &mut session, &root)?;
    let last = replay_ticks_after_restart_root_v1(
        &mut client,
        campaign_id,
        &mut session,
        &scenario_scope,
        &root,
    )?;
    Ok((session, Some(last)))
}

struct DurableRestartRootV1 {
    resolve_tick_sql: i64,
    resolve_tick: u64,
    marker_count: i64,
}

fn select_durable_restart_root_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
) -> Result<Option<DurableRestartRootV1>, RustPersistenceRuntimeErrorV2> {
    let root_tick: Option<i64> = client
        .query_one(
            "SELECT pg_catalog.max(marker.resolve_tick) \
             FROM babylon_state.tick_commit AS marker \
             JOIN babylon_state.checkpoint_manifest AS checkpoint \
               ON checkpoint.campaign_id = marker.campaign_id \
              AND checkpoint.resolve_tick = marker.resolve_tick \
              AND checkpoint.completeness_tag = 1 \
             WHERE marker.campaign_id = $1::uuid",
            &[campaign_id.as_uuid()],
        )
        .and_then(|row| row.try_get(0))
        .map_err(|error| {
            RustPersistenceRuntimeErrorV2::postgres("select latest full checkpoint", &error)
        })?;
    let marker_count: i64 = client
        .query_one(
            "SELECT pg_catalog.count(*) FROM babylon_state.tick_commit \
             WHERE campaign_id = $1::uuid",
            &[campaign_id.as_uuid()],
        )
        .and_then(|row| row.try_get(0))
        .map_err(|error| {
            RustPersistenceRuntimeErrorV2::postgres("count restart markers", &error)
        })?;
    let Some(root_tick_sql) = root_tick else {
        if marker_count == 0 {
            return Ok(None);
        }
        return Err(RustPersistenceRuntimeErrorV2::DeltaCheckpointNotRestartRoot);
    };
    let root_tick = u64::try_from(root_tick_sql)
        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    let prefix_count: i64 = client
        .query_one(
            "SELECT pg_catalog.count(*) FROM babylon_state.tick_commit \
             WHERE campaign_id = $1::uuid AND resolve_tick <= $2",
            &[campaign_id.as_uuid(), &root_tick_sql],
        )
        .and_then(|row| row.try_get(0))
        .map_err(|error| {
            RustPersistenceRuntimeErrorV2::postgres("count checkpoint prefix markers", &error)
        })?;
    if u64::try_from(prefix_count).ok() != Some(root_tick) {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    Ok(Some(DurableRestartRootV1 {
        resolve_tick_sql: root_tick_sql,
        resolve_tick: root_tick,
        marker_count,
    }))
}

fn restore_durable_restart_root_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    session: &mut ReplayTickSession<HypergraphStore>,
    root: &DurableRestartRootV1,
) -> Result<String, RustPersistenceRuntimeErrorV2> {
    let scenario_scope = session
        .stable_graph_state()
        .map_err(|_| RustPersistenceRuntimeErrorV2::ReplayTick)?
        .scenario_scope()
        .to_owned();
    let stored =
        read_stored_typed_tick_v2(client, campaign_id, root.resolve_tick, &scenario_scope)?
            .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    validate_stored_empty_actions_v1(&stored, session, root.resolve_tick)?;
    validate_checkpoint_identity_sections_v1(&stored, session)?;
    session
        .restore_full_checkpoint(
            root.resolve_tick_sql,
            stored.graph_state(),
            stored.material_rows(),
            stored
                .checkpoint_section(2)
                .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?,
        )
        .map_err(|_| RustPersistenceRuntimeErrorV2::ReplayTick)?;
    Ok(scenario_scope)
}

fn replay_ticks_after_restart_root_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    session: &mut ReplayTickSession<HypergraphStore>,
    scenario_scope: &str,
    root: &DurableRestartRootV1,
) -> Result<CommittedResolveTickV1, RustPersistenceRuntimeErrorV2> {
    let tail_ticks = client
        .query(
            "SELECT resolve_tick FROM babylon_state.tick_commit \
             WHERE campaign_id = $1::uuid AND resolve_tick > $2 ORDER BY resolve_tick",
            &[campaign_id.as_uuid(), &root.resolve_tick_sql],
        )
        .map_err(|error| RustPersistenceRuntimeErrorV2::postgres("read restart tail", &error))?;
    let mut expected_tick = root
        .resolve_tick
        .checked_add(1)
        .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    let mut sink = CollectingSink::default();
    let mut last = CommittedResolveTickV1::try_from(root.resolve_tick)
        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    for row in tail_ticks {
        let stored_tick: i64 = decode_runtime_column(&row, 0)?;
        let stored_tick = u64::try_from(stored_tick)
            .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
        if stored_tick != expected_tick {
            return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
        }
        let stored = read_stored_typed_tick_v2(client, campaign_id, expected_tick, scenario_scope)?
            .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?;
        validate_stored_empty_actions_v1(&stored, session, expected_tick)?;
        let actions =
            OrderedPracticeActionBatchV1::empty(session.session_identity().clone(), expected_tick)
                .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        let candidate = session
            .prepare_advance(&actions)
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplayTick)?;
        let prepared = prepare_committed_tick_v2(candidate.report())?;
        let resolve_tick = prepared.resolve_tick();
        if resolve_tick.get() != expected_tick {
            return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
        }
        let checkpoint =
            CommittedFullCheckpointV1::capture(campaign_id, resolve_tick, candidate.report())?;
        let tick_content_hash = prepared.tick_content_hash();
        let envelope = prepared.into_envelope(campaign_id)?;
        envelope
            .classify_retry_against(stored.envelope())
            .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
        validate_stored_checkpoint_sections_v1(&stored, &checkpoint)?;
        let acknowledgement = ReplayCommitAcknowledgementV1::new(
            ReplayCommitDispositionV1::ReconciledAfterAmbiguousCommit,
            expected_tick,
            tick_content_hash,
        );
        session
            .acknowledge_prepared(&mut sink, candidate, acknowledgement)
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplayTick)?;
        last = resolve_tick;
        expected_tick = expected_tick
            .checked_add(1)
            .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    }
    if u64::try_from(root.marker_count).ok() != Some(last.get()) {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    Ok(last)
}

fn validate_stored_empty_actions_v1(
    stored: &crate::stored_tick::StoredTypedTickV2,
    session: &ReplayTickSession<HypergraphStore>,
    resolve_tick: u64,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let actions =
        OrderedPracticeActionBatchV1::empty(session.session_identity().clone(), resolve_tick)
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
    if stored.action_layout() != 1
        || stored.action_digest().as_slice() != actions.digest().as_bytes()
        || stored.action_bytes() != actions.canonical_bytes()
    {
        return Err(RustPersistenceRuntimeErrorV2::ReplaySource);
    }
    Ok(())
}

fn validate_checkpoint_identity_sections_v1(
    stored: &crate::stored_tick::StoredTypedTickV2,
    session: &ReplayTickSession<HypergraphStore>,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let mut content_digest = Vec::new();
    content_digest.try_reserve_exact(64).map_err(|_| {
        RustPersistenceRuntimeErrorV2::Allocation {
            field: "restart content digest",
            requested: 64,
        }
    })?;
    content_digest.extend_from_slice(&session.content_digest().defines_hash);
    content_digest.extend_from_slice(&session.content_digest().rules_hash);
    let seed = session.rng_seed().to_be_bytes();
    let reference = session.reference_digest();
    let expected = [
        (3_u8, session.resolver_manifest_bytes()),
        (4, session.prepared_environment_bytes()),
        (5, session.session_identity().as_bytes()),
        (6, seed.as_slice()),
        (7, content_digest.as_slice()),
        (8, reference.as_bytes().as_slice()),
    ];
    if expected.into_iter().any(|(tag, bytes)| {
        stored
            .checkpoint_section(tag)
            .is_none_or(|stored| stored != bytes)
    }) {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    Ok(())
}

fn validate_stored_checkpoint_sections_v1(
    stored: &crate::stored_tick::StoredTypedTickV2,
    checkpoint: &CommittedFullCheckpointV1,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    if checkpoint.exact_section_bytes().len() != 9
        || checkpoint
            .exact_section_bytes()
            .iter()
            .enumerate()
            .any(|(index, expected)| {
                let tag = u8::try_from(index + 1).ok();
                tag.and_then(|tag| stored.checkpoint_section(tag)) != Some(expected.as_slice())
            })
    {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    Ok(())
}

fn insert_typed_graph_rows_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReportV2,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let rows = report.result_stable_graph().rows();
    for (local_name, node_type) in rows.nodes() {
        require_single_insert_v1(
            client.execute(
                "INSERT INTO babylon_state.graph_node_v1 \
                 (campaign_id, resolve_tick, local_name, node_type) VALUES ($1::uuid, $2, $3, $4)",
                &[campaign_id.as_uuid(), &resolve_tick, local_name, node_type],
            ),
            "insert graph node",
        )?;
    }
    for (local_name, qname, bits) in rows.node_f64() {
        let value_bits = bit_pattern_i64_v1(*bits);
        require_single_insert_v1(
            client.execute(
                "INSERT INTO babylon_state.graph_node_f64_v1 \
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
            "insert graph node f64",
        )?;
    }
    for (edge_type, source, target, strength_bits) in rows.edges() {
        let strength_bits = bit_pattern_i64_v1(*strength_bits);
        require_single_insert_v1(
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
    for (local_name, hyperedge_type, members) in rows.hyperedges() {
        require_single_insert_v1(
            client.execute(
                "INSERT INTO babylon_state.graph_hyperedge_v1 \
                 (campaign_id, resolve_tick, local_name, hyperedge_type) VALUES ($1::uuid, $2, $3, $4)",
                &[campaign_id.as_uuid(), &resolve_tick, local_name, hyperedge_type],
            ),
            "insert graph hyperedge",
        )?;
        for (position, member) in members.iter().enumerate() {
            let position = checked_position_v1(position)?;
            require_single_insert_v1(
                client.execute(
                    "INSERT INTO babylon_state.graph_hyperedge_member_v1 \
                     (campaign_id, resolve_tick, local_name, position, member) \
                     VALUES ($1::uuid, $2, $3, $4, $5)",
                    &[
                        campaign_id.as_uuid(),
                        &resolve_tick,
                        local_name,
                        &position,
                        member,
                    ],
                ),
                "insert graph hyperedge member",
            )?;
        }
    }
    insert_graph_value_rows_v1(client, campaign_id, resolve_tick, report)
}

fn insert_graph_value_rows_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReportV2,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let rows = report.result_stable_graph().rows();
    for (edge_type, source, target, qname, bits) in rows.edge_f64() {
        let value_bits = bit_pattern_i64_v1(*bits);
        require_single_insert_v1(
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
        require_single_insert_v1(
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
        let value_bits = bit_pattern_i64_v1(*bits);
        require_single_insert_v1(
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

fn insert_typed_material_rows_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReportV2,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let rows = report.material_state_rows();
    for row in rows.world_registers().rows() {
        let prefix: [&(dyn ToSql + Sync); 3] = [campaign_id.as_uuid(), &resolve_tick, &row.qname()];
        insert_bsl_value_row_v1(
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
        let territory_id = stable_key_bytes_v1(row.territory_id())?;
        require_single_insert_v1(
            client.execute(
                "INSERT INTO babylon_state.territory_state_v1 \
                 (campaign_id, resolve_tick, territory_id) VALUES ($1::uuid, $2, $3)",
                &[campaign_id.as_uuid(), &resolve_tick, &territory_id],
            ),
            "insert territory state",
        )?;
        for (position, (field_name, value)) in row.ordered_fields().iter().enumerate() {
            let position = checked_position_v1(position)?;
            let prefix: [&(dyn ToSql + Sync); 5] = [
                campaign_id.as_uuid(),
                &resolve_tick,
                &territory_id,
                &position,
                field_name,
            ];
            insert_bsl_value_row_v1(
                client,
                "INSERT INTO babylon_state.territory_state_field_v1 \
                 (campaign_id, resolve_tick, territory_id, position, field_name, value_tag, int_value, \
                  currency_value, real_bits, ratio_bits, ratio_min_bits, ratio_max_bits, bool_value, \
                  enum_type, enum_member, stable_key) \
                 VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8::text::numeric, $9, $10, $11, $12, $13, $14, $15, $16)",
                &prefix,
                value,
                "insert territory field",
            )?;
        }
    }
    insert_dynamic_hex_rows_v1(client, campaign_id, resolve_tick, report)?;
    insert_organization_state_rows_v1(client, campaign_id, resolve_tick, report)
}

fn insert_dynamic_hex_rows_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReportV2,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let rows = report.material_state_rows().dynamic_hexes().rows();
    let expected =
        u64::try_from(rows.len()).map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    let sink = client
        .copy_in(
            "COPY babylon_state.hex_state_delta_v1 \
             (campaign_id, resolve_tick, cell_id, c_bits, v_bits, s_bits, k_bits, \
              biocapacity_stock_bits, energy_stock_bits, raw_material_stock_bits, \
              internet_access_pct_bits, surveillance_coupling_bits) FROM STDIN BINARY",
        )
        .map_err(|error| {
            RustPersistenceRuntimeErrorV2::postgres("begin dynamic hex state copy", &error)
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
            .map_err(|_| RustPersistenceRuntimeErrorV2::SemanticCodec)?;
        let values = row.value_bits().map(bit_pattern_i64_v1);
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
                RustPersistenceRuntimeErrorV2::postgres("write dynamic hex state copy", &error)
            })?;
    }
    let inserted = writer.finish().map_err(|error| {
        RustPersistenceRuntimeErrorV2::postgres("finish dynamic hex state copy", &error)
    })?;
    if inserted != expected {
        return Err(RustPersistenceRuntimeErrorV2::database(
            "count dynamic hex state copy",
        ));
    }
    Ok(())
}

fn insert_organization_state_rows_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReportV2,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    for row in report.material_state_rows().organizations().rows() {
        let organization_id = stable_key_bytes_v1(row.organization_id())?;
        let prefix: [&(dyn ToSql + Sync); 3] =
            [campaign_id.as_uuid(), &resolve_tick, &organization_id];
        insert_bsl_value_row_v1(
            client,
            "INSERT INTO babylon_state.organization_state_v1 \
             (campaign_id, resolve_tick, organization_id, organization_kind_tag, \
              organization_kind_int, organization_kind_currency, organization_kind_real_bits, \
              organization_kind_ratio_bits, organization_kind_ratio_min_bits, \
              organization_kind_ratio_max_bits, organization_kind_bool, \
              organization_kind_enum_type, organization_kind_enum_member, organization_kind_stable_key) \
             VALUES ($1::uuid, $2, $3, $4, $5, $6::text::numeric, $7, $8, $9, $10, $11, $12, $13, $14)",
            &prefix,
            row.organization_kind(),
            "insert organization state",
        )?;
        for (position, territory_id) in row.ordered_territory_ids().iter().enumerate() {
            let position = checked_position_v1(position)?;
            let territory_id = stable_key_bytes_v1(territory_id)?;
            require_single_insert_v1(
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
        for (position, (field_name, value)) in row.ordered_fields().iter().enumerate() {
            let position = checked_position_v1(position)?;
            let prefix: [&(dyn ToSql + Sync); 5] = [
                campaign_id.as_uuid(),
                &resolve_tick,
                &organization_id,
                &position,
                field_name,
            ];
            insert_bsl_value_row_v1(
                client,
                "INSERT INTO babylon_state.organization_state_field_v1 \
                 (campaign_id, resolve_tick, organization_id, position, field_name, value_tag, int_value, \
                  currency_value, real_bits, ratio_bits, ratio_min_bits, ratio_max_bits, bool_value, \
                  enum_type, enum_member, stable_key) \
                 VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8::text::numeric, $9, $10, $11, $12, $13, $14, $15, $16)",
                &prefix,
                value,
                "insert organization field",
            )?;
        }
    }
    Ok(())
}

fn insert_choice_receipt_rows_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReportV2,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    for receipt in &report.report().choice_receipts {
        let encounter_ordinal = i64::from(receipt.encounter_ordinal());
        let slot = i64::from(receipt.slot());
        let stable_carrier = stable_key_bytes_v1(receipt.stable_carrier())?;
        let draw_ticket = receipt.draw_ticket().to_string();
        require_single_insert_v1(
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
            let position = i64::from(checked_u32_position_v1(position)?);
            let mass_nanounits = branch.mass.nanounits().to_string();
            let ticket_start = branch.tickets.start.to_string();
            let ticket_end_exclusive = branch.tickets.end.to_string();
            let ticket_count = branch.tickets.count.to_string();
            require_single_insert_v1(
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
            let position = i64::from(checked_u32_position_v1(position)?);
            let stable_element = stable_key_bytes_v1(element)?;
            require_single_insert_v1(
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

fn insert_typed_event_rows_v2(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    report: &IdentifiedTickReportV2,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    for (ordinal, event) in report.successful_event_batch().events().iter().enumerate() {
        let ordinal = i64::try_from(ordinal).map_err(|_| {
            RustPersistenceRuntimeErrorV2::IntegerConversion {
                field: "successful event ordinal",
                value: ordinal,
            }
        })?;
        let choice_receipt_ordinal = event
            .choice_receipt()
            .map(|reference| i64::from(reference.encounter_ordinal()));
        require_single_insert_v1(
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
            let position = i64::from(checked_u32_position_v1(position)?);
            let prefix: [&(dyn ToSql + Sync); 5] = [
                campaign_id.as_uuid(),
                &resolve_tick,
                &ordinal,
                &position,
                field_name,
            ];
            insert_bsl_value_row_v1(
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

fn insert_full_checkpoint_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    resolve_tick: i64,
    checkpoint: &CommittedFullCheckpointV1,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let completeness_tag = 1_i16;
    require_single_insert_v1(
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
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    for (section, exact_bytes) in checkpoint
        .sections()
        .iter()
        .zip(checkpoint.exact_section_bytes())
    {
        if sha256_of(exact_bytes) != section.sha256() {
            return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
        }
        let section_tag = i16::from(section.tag().tag());
        let ordinal = 0_i64;
        require_single_insert_v1(
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

struct BslSqlValueV1 {
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

impl BslSqlValueV1 {
    fn from_stable(value: &StableBslValueV1) -> Result<Self, RustPersistenceRuntimeErrorV2> {
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
            StableBslValueV1::Int(value) => {
                row.tag = 1;
                row.int_value = Some(*value);
            }
            StableBslValueV1::CurrencyMicroUnits(value) => {
                row.tag = 2;
                row.currency_value = Some(value.to_string());
            }
            StableBslValueV1::RealBits(bits) => {
                row.tag = 3;
                row.real_bits = Some(bit_pattern_i64_v1(*bits));
            }
            StableBslValueV1::RatioBits { value, floor, cap } => {
                row.tag = 4;
                row.ratio_bits = Some(bit_pattern_i64_v1(*value));
                row.ratio_min_bits = floor.map(bit_pattern_i64_v1);
                row.ratio_max_bits = cap.map(bit_pattern_i64_v1);
            }
            StableBslValueV1::Bool(value) => {
                row.tag = 5;
                row.bool_value = Some(*value);
            }
            StableBslValueV1::Enum { enum_type, member } => {
                row.tag = 6;
                row.enum_type = Some(enum_type.clone());
                row.enum_member = Some(member.clone());
            }
            StableBslValueV1::Node(key) => {
                row.tag = 7;
                row.stable_key = Some(stable_key_bytes_v1(key)?);
            }
            StableBslValueV1::Hyperedge(key) => {
                row.tag = 8;
                row.stable_key = Some(stable_key_bytes_v1(key)?);
            }
            StableBslValueV1::Edge(key) => {
                row.tag = 9;
                row.stable_key = Some(stable_key_bytes_v1(key)?);
            }
        }
        Ok(row)
    }
}

fn insert_bsl_value_row_v1(
    client: &mut impl GenericClient,
    sql: &str,
    prefix: &[&(dyn ToSql + Sync)],
    value: &StableBslValueV1,
    operation: &'static str,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let value = BslSqlValueV1::from_stable(value)?;
    let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();
    params.try_reserve_exact(prefix.len() + 11).map_err(|_| {
        RustPersistenceRuntimeErrorV2::Allocation {
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
    require_single_insert_v1(client.execute(sql, &params), operation)
}

fn stable_key_bytes_v1(
    key: &babylon_graph::stable_element::StableElementKeyV1,
) -> Result<Vec<u8>, RustPersistenceRuntimeErrorV2> {
    key.canonical_bytes()
        .map_err(|_| RustPersistenceRuntimeErrorV2::SemanticCodec)
}

fn bit_pattern_i64_v1(bits: u64) -> i64 {
    i64::from_be_bytes(bits.to_be_bytes())
}

fn checked_position_v1(position: usize) -> Result<i32, RustPersistenceRuntimeErrorV2> {
    i32::try_from(position).map_err(|_| RustPersistenceRuntimeErrorV2::IntegerConversion {
        field: "typed child position",
        value: position,
    })
}

fn checked_u32_position_v1(position: usize) -> Result<u32, RustPersistenceRuntimeErrorV2> {
    u32::try_from(position).map_err(|_| RustPersistenceRuntimeErrorV2::IntegerConversion {
        field: "V2 ordered position",
        value: position,
    })
}

fn require_single_insert_v1(
    result: Result<u64, postgres::Error>,
    operation: &'static str,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let affected =
        result.map_err(|error| RustPersistenceRuntimeErrorV2::postgres(operation, &error))?;
    if affected == 1 {
        Ok(())
    } else {
        Err(RustPersistenceRuntimeErrorV2::database(operation))
    }
}

fn decode_runtime_column<T: postgres::types::FromSqlOwned>(
    row: &postgres::Row,
    index: usize,
) -> Result<T, RustPersistenceRuntimeErrorV2> {
    row.try_get(index)
        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)
}

fn decode_digest_column(
    row: &postgres::Row,
    index: usize,
) -> Result<[u8; 32], RustPersistenceRuntimeErrorV2> {
    let bytes: Vec<u8> = decode_runtime_column(row, index)?;
    bytes
        .try_into()
        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)
}

/// Exact report-derived inputs held before a durable commit attempt.
///
/// This type owns no replay engine and cannot adjudicate or recompute a tick.
/// It closes every live V2 row family and converts to an envelope only after
/// all semantic batches and checkpoint sections have composed successfully.
#[derive(Debug, PartialEq, Eq)]
pub struct PreparedCommittedTickV2 {
    resolve_tick: CommittedResolveTickV1,
    tick_content_hash: TickContentHashV1,
    graph_event_batches: GraphEventChoiceSemanticBatchesV2,
    material_state_rows: Vec<CommittedTickRowV2>,
    checkpoint_rows: CheckpointRowsV1,
    archive_dirty_receipt: ArchiveDirtyReceiptV1,
}

impl PreparedCommittedTickV2 {
    /// Return the exact positive resolve tick carried by the source report.
    #[must_use]
    pub const fn resolve_tick(&self) -> CommittedResolveTickV1 {
        self.resolve_tick
    }

    /// Return the constitutional content identity carried by the source report.
    #[must_use]
    pub const fn tick_content_hash(&self) -> TickContentHashV1 {
        self.tick_content_hash
    }

    /// Borrow the exact report-derived checkpoint producer.
    #[must_use]
    pub const fn checkpoint_rows(&self) -> &CheckpointRowsV1 {
        &self.checkpoint_rows
    }

    /// Borrow the exact singular Archive work receipt.
    #[must_use]
    pub const fn archive_dirty_receipt(&self) -> &ArchiveDirtyReceiptV1 {
        &self.archive_dirty_receipt
    }

    fn into_envelope(
        self,
        campaign_id: CampaignId,
    ) -> Result<CommittedTickEnvelopeV2, RustPersistenceRuntimeErrorV2> {
        let claim = TickCommitClaimV1::compose(
            campaign_id,
            self.resolve_tick.get(),
            self.tick_content_hash,
        );
        let (graph, event, choice_receipt) = self.graph_event_batches.into_rows();
        CommittedTickEnvelopeV2::compose(
            claim,
            CommittedTickRowFamiliesV2 {
                graph,
                state: self.material_state_rows,
                event,
                choice_receipt,
                checkpoint: self.checkpoint_rows.into_rows(),
                archive_dirty_receipt: self.archive_dirty_receipt.into_row(),
            },
        )
        .map_err(RustPersistenceRuntimeErrorV2::SemanticEnvelope)
    }

    pub(crate) fn into_material_families(
        self,
        hash: TickContentHashV1,
    ) -> Result<CommittedTickRowFamiliesV2, RustPersistenceRuntimeErrorV2> {
        let (graph, event, choice_receipt) = self.graph_event_batches.into_rows();
        Ok(CommittedTickRowFamiliesV2 {
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

    #[cfg(test)]
    pub(crate) fn graph_row_count(&self) -> usize {
        self.graph_event_batches.graph_row_count()
    }

    #[cfg(test)]
    pub(crate) fn event_row_count(&self) -> usize {
        self.graph_event_batches.event_row_count()
    }

    #[cfg(test)]
    pub(crate) fn choice_receipt_row_count(&self) -> usize {
        self.graph_event_batches.choice_receipt_row_count()
    }
}

/// Derive one stopped, database-free durable candidate from one identified report.
///
/// # Errors
/// Returns the first resolve-tick, codec, allocation, or aggregate-bound
/// refusal. This function never parses rules, executes a tick, or judges game
/// mechanics; every semantic source comes from `report`.
pub fn prepare_committed_tick_v2(
    report: &IdentifiedTickReportV2,
) -> Result<PreparedCommittedTickV2, RustPersistenceRuntimeErrorV2> {
    let completed_tick = report.result_registers().completed_tick();
    let raw_resolve_tick = u64::try_from(completed_tick).map_err(|_| {
        RustPersistenceRuntimeErrorV2::ResolveTickOutOfRange {
            actual: completed_tick,
        }
    })?;
    let resolve_tick = CommittedResolveTickV1::try_from(raw_resolve_tick).map_err(
        |_: CommittedResolveTickErrorV1| RustPersistenceRuntimeErrorV2::ResolveTickOutOfRange {
            actual: completed_tick,
        },
    )?;
    let graph_event_batches = compose_graph_event_choice_semantic_batches_v2(report)?;
    let material_state_rows = compose_material_state_rows_v1(report.material_state_rows())?;
    let checkpoint_rows = compose_checkpoint_rows_v1(report, resolve_tick)?;
    let archive_dirty_receipt = compose_archive_dirty_receipt_v1(report)?;
    Ok(PreparedCommittedTickV2 {
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
    use std::str::FromStr;
    use std::sync::atomic::Ordering;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::{Duration, Instant};

    use babylon_bsl::rule_pipeline::split_content;
    use babylon_bsl::rules_hash_of;
    use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
    use babylon_kernel::sha256_of;
    use babylon_kernel::tick_content_hash::RefDigestV1;
    use babylon_kernel::ContentDigest;
    use postgres::{Config, NoTls};
    use uuid::Uuid;

    use super::*;

    const DSN_ENV: &str = "BABYLON_LEGACY_ADOPTER_TEST_DSN";
    const ACK_ENV: &str = "BABYLON_LEGACY_ADOPTER_DISPOSABLE_ACK";
    const ACK: &str = "I_UNDERSTAND_PER20_DROPS_SCRATCH_DATABASES_ROLES_AND_CREATED_BABYLON_INTEL";
    const CANARY_ENV: &str = "BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY";
    const TEMPLATE_DB_ENV: &str = "BABYLON_RUNTIME_TEMPLATE_DB";
    const DEFINES: &[u8] = br#"{"alpha":1}"#;
    const REFERENCE_BUNDLE_DOMAIN: &[u8] = b"babylon.h3.reference-bundle-composite.v1\0";
    const SCENARIO: &str =
        include_str!("../../babylon-tick/content/scenarios/struggle-spark-conformance.bscn");
    const RULE: &str = include_str!("../../babylon-tick/content/rules/struggle-spark.bsl");
    const RUNTIME_CHOICE_SEEDS: (i64, i64) = (2, 0);
    const RUNTIME_CHOICE_DRAW_TICKETS: (u64, u64) =
        (1_146_489_467_234_058_882, 17_919_240_830_411_110_681);

    fn wait_for_access_exclusive_lock(config: &Config, relation_name: &str, timeout_message: &str) {
        let mut observer = config.connect(NoTls).expect("lock observer connection");
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let access_exclusive_waiting: bool = observer
                .query_one(
                    "SELECT EXISTS (\
                       SELECT 1 FROM pg_catalog.pg_locks AS requested \
                        WHERE requested.relation = pg_catalog.to_regclass($1) \
                          AND requested.mode = 'AccessExclusiveLock' \
                          AND NOT requested.granted)",
                    &[&relation_name],
                )
                .expect("lock wait query")
                .try_get(0)
                .expect("lock wait decodes");
            if access_exclusive_waiting {
                return;
            }
            assert!(Instant::now() < deadline, "{timeout_message}");
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    #[ignore = "requires the task-owned disposable PG17 runtime"]
    fn live_v2_bootstrap_from_empty_pg17_is_exact_and_idempotent() {
        let base = validated_base_config();
        let database = TestDatabase::create(&base, "vbootstrap");
        let config = database.config(&base);

        let first = activate_rust_persistence_v2(&config)
            .expect("empty PG17 database reaches the live V2 authority");
        let second = activate_rust_persistence_v2(&config)
            .expect("terminal V2 bootstrap is byte-exact and idempotent");
        assert_eq!(first, second);

        let mut client = config.connect(NoTls).expect("bootstrap proof connection");
        let server_version: i32 = client
            .query_one(
                "SELECT pg_catalog.current_setting('server_version_num')::pg_catalog.int4",
                &[],
            )
            .expect("server version query")
            .try_get(0)
            .expect("server version decodes");
        assert!((170_000..180_000).contains(&server_version));
        let ordinary_versions: Vec<i64> = client
            .query(
                "SELECT version FROM babylon_state.schema_migration ORDER BY version",
                &[],
            )
            .expect("ordinary epoch query")
            .into_iter()
            .map(|row| row.try_get(0).expect("ordinary epoch decodes"))
            .collect();
        assert_eq!(ordinary_versions, (1_i64..=7).collect::<Vec<_>>());
        let authority_row = client
            .query_one(
                "SELECT \
                   (SELECT pg_catalog.count(*) FROM babylon_meta.persistence_authority_ledger), \
                   (SELECT pg_catalog.max(schema_epoch) FROM babylon_meta.persistence_authority_ledger), \
                   (SELECT pg_catalog.count(*) FROM babylon_meta.committed_tick_v2_authority_ledger), \
                   (SELECT pg_catalog.max(activation_epoch) FROM babylon_meta.committed_tick_v2_authority_ledger), \
                   pg_catalog.to_regclass('babylon_state.tick_event_v1') IS NULL, \
                   pg_catalog.to_regclass('babylon_state.tick_event_field_v1') IS NULL, \
                   pg_catalog.to_regclass('babylon_state.tick_choice_receipt_v1') IS NOT NULL, \
                   pg_catalog.to_regclass('babylon_state.tick_event_v2') IS NOT NULL",
                &[],
            )
            .expect("V2 authority shape query");
        let authority_shape = (
            authority_row
                .try_get::<_, i64>(0)
                .expect("predecessor count"),
            authority_row
                .try_get::<_, i16>(1)
                .expect("predecessor epoch"),
            authority_row.try_get::<_, i64>(2).expect("V2 count"),
            authority_row.try_get::<_, i16>(3).expect("V2 epoch"),
            authority_row
                .try_get::<_, bool>(4)
                .expect("V1 event absent"),
            authority_row
                .try_get::<_, bool>(5)
                .expect("V1 field absent"),
            authority_row
                .try_get::<_, bool>(6)
                .expect("receipt present"),
            authority_row
                .try_get::<_, bool>(7)
                .expect("V2 event present"),
        );
        assert_eq!(authority_shape, (2, 9, 2, 11, true, true, true, true));
        database.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PG17 runtime"]
    fn live_v2_pre_activation_inventory_is_exact_and_read_only() {
        let base = validated_base_config();
        let database = TestDatabase::create(&base, "vpreflight");
        let config = database.config(&base);
        config
            .connect(NoTls)
            .expect("pre-activation fixture connection")
            .batch_execute(
                "CREATE TABLE public.game_session (id BIGINT PRIMARY KEY); \
                 INSERT INTO public.game_session (id) VALUES (1), (2); \
                 CREATE TABLE public.tick_event (id BIGINT PRIMARY KEY); \
                 INSERT INTO public.tick_event (id) VALUES (1)",
            )
            .expect("nonempty incompatible fixtures exist before activation");

        assert_eq!(
            activate_rust_persistence_v2(&config),
            Err(
                RustPersistenceActivationErrorV2::PreActivationIncompatibleInventory {
                    relations: vec![
                        PreActivationIncompatibleRelationV2 {
                            relation_name: "public.game_session",
                            observed_row_count: 2,
                        },
                        PreActivationIncompatibleRelationV2 {
                            relation_name: "public.tick_event",
                            observed_row_count: 1,
                        },
                    ],
                },
            )
        );
        let proof = config
            .connect(NoTls)
            .expect("read-only refusal proof connection")
            .query_one(
                "SELECT \
                   pg_catalog.to_regclass('babylon_state.schema_migration') IS NULL, \
                   pg_catalog.to_regclass('babylon_meta.persistence_authority_ledger') IS NULL, \
                   pg_catalog.to_regclass('babylon_ref.h3_cell') IS NULL, \
                   (SELECT pg_catalog.count(*) FROM public.game_session), \
                   (SELECT pg_catalog.count(*) FROM public.tick_event)",
                &[],
            )
            .expect("read-only refusal state reads");
        assert!(proof.try_get::<_, bool>(0).expect("ordinary ledger absent"));
        assert!(proof
            .try_get::<_, bool>(1)
            .expect("predecessor ledger absent"));
        assert!(proof
            .try_get::<_, bool>(2)
            .expect("reference installation absent"));
        assert_eq!(proof.try_get::<_, i64>(3).expect("game rows remain"), 2);
        assert_eq!(proof.try_get::<_, i64>(4).expect("event rows remain"), 1);
        database.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PG17 runtime"]
    fn live_epoch_nine_locks_legacy_table_before_empty_inventory_and_drop() {
        let base = validated_base_config();
        let database = TestDatabase::create(&base, "vlegacylock");
        let config = database.config(&base);
        bootstrap_h3_reader_epoch_v1(&config).expect("reader predecessor bootstraps");
        let expected_prepared =
            PredecessorAuthorityLedgerRowV2::prepared().expect("predecessor prepared row composes");
        let expected_active = PredecessorAuthorityLedgerRowV2::active(&expected_prepared)
            .expect("predecessor active row composes");
        let mut preparation = config.connect(NoTls).expect("preparation connection");
        execute_predecessor_migration_v2(
            &mut preparation,
            MIGRATION_0008_SQL,
            &expected_prepared,
            "migration 8 lock fixture",
        )
        .expect("epoch 8 preparation commits");
        preparation
            .batch_execute("CREATE TABLE public.game_session (id BIGINT PRIMARY KEY)")
            .expect("exact legacy table fixture exists");

        let mut holder = config.connect(NoTls).expect("writer lock connection");
        let mut holder_transaction = holder.transaction().expect("writer transaction");
        holder_transaction
            .batch_execute("LOCK TABLE public.game_session IN ROW EXCLUSIVE MODE")
            .expect("legacy writer lock held");

        let worker_config = config.clone();
        let worker = thread::spawn(move || {
            let mut client = worker_config
                .connect(NoTls)
                .expect("epoch 9 worker connection");
            execute_predecessor_migration_v2(
                &mut client,
                MIGRATION_0009_SQL,
                &expected_active,
                "migration 9 lock race",
            )
        });

        wait_for_access_exclusive_lock(
            &config,
            "public.game_session",
            "epoch 9 never requested ACCESS EXCLUSIVE on the exact legacy table",
        );

        holder_transaction
            .execute("INSERT INTO public.game_session (id) VALUES (1)", &[])
            .expect("concurrent legacy writer inserts while retaining its lock");
        holder_transaction
            .commit()
            .expect("concurrent legacy writer commits");
        let Err(RustPersistenceActivationErrorV2::Database {
            operation: "migration 9 lock race",
            diagnostic: Some(_),
        }) = worker.join().expect("epoch 9 worker joins")
        else {
            panic!("epoch 9 lock race must retain its server diagnostic");
        };

        let proof = config
            .connect(NoTls)
            .expect("lock-race proof connection")
            .query_one(
                "SELECT pg_catalog.to_regclass('public.game_session') IS NOT NULL, \
                        (SELECT pg_catalog.count(*) FROM public.game_session), \
                        (SELECT pg_catalog.count(*) \
                           FROM babylon_meta.persistence_authority_ledger \
                          WHERE state_tag = 2), \
                        (SELECT pg_catalog.count(*) \
                           FROM babylon_meta.python_relation_disposition_v1)",
                &[],
            )
            .expect("lock-race refusal proof");
        assert!(proof.try_get::<_, bool>(0).expect("legacy table remains"));
        assert_eq!(proof.try_get::<_, i64>(1).expect("legacy row remains"), 1);
        assert_eq!(proof.try_get::<_, i64>(2).expect("active row absent"), 0);
        assert_eq!(
            proof
                .try_get::<_, i64>(3)
                .expect("disposition rows roll back"),
            0
        );
        database.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PG17 runtime"]
    fn live_epoch_nine_locks_opaque_rows_before_empty_inventory_and_drop() {
        let base = validated_base_config();
        let database = TestDatabase::create(&base, "vopaquelock");
        let config = database.config(&base);
        bootstrap_h3_reader_epoch_v1(&config).expect("reader predecessor bootstraps");
        let prepared = PredecessorAuthorityLedgerRowV2::prepared().expect("prepared row composes");
        let active =
            PredecessorAuthorityLedgerRowV2::active(&prepared).expect("active row composes");
        let mut preparation = config.connect(NoTls).expect("preparation connection");
        execute_predecessor_migration_v2(
            &mut preparation,
            MIGRATION_0008_SQL,
            &prepared,
            "migration 8 opaque-lock fixture",
        )
        .expect("epoch 8 preparation commits");
        preparation
            .batch_execute(
                "INSERT INTO babylon_state.campaign (\
                     campaign_id, replay_layout_version, rng_layout_version, replay_session_id, \
                     rng_seed, defines_hash, rules_hash, ref_digest\
                 ) SELECT \
                     '00000000-0000-0000-0000-000000000009'::pg_catalog.uuid, \
                     1, 2, 'epoch-nine-opaque-race', 0, \
                     pg_catalog.decode(pg_catalog.repeat('00', 32), 'hex'), \
                     pg_catalog.decode(pg_catalog.repeat('00', 32), 'hex'), \
                     ref_digest \
                 FROM babylon_ref.h3_reference_cohort \
                 ORDER BY ref_digest \
                 LIMIT 1; \
                 INSERT INTO babylon_state.tick_commit (\
                     campaign_id, resolve_tick, envelope_layout_version, \
                     tick_content_hash, envelope_digest\
                 ) VALUES (\
                     '00000000-0000-0000-0000-000000000009'::pg_catalog.uuid, \
                     0, 1, \
                     pg_catalog.decode(pg_catalog.repeat('00', 32), 'hex'), \
                     pg_catalog.decode(pg_catalog.repeat('00', 32), 'hex')\
                 )",
            )
            .expect("opaque row parent fixture exists");

        let mut holder = config.connect(NoTls).expect("opaque writer connection");
        let mut holder_transaction = holder.transaction().expect("opaque writer transaction");
        holder_transaction
            .batch_execute("LOCK TABLE babylon_state.tick_graph_row IN ROW EXCLUSIVE MODE")
            .expect("opaque writer lock held");

        let worker_config = config.clone();
        let worker = thread::spawn(move || {
            let mut client = worker_config
                .connect(NoTls)
                .expect("epoch 9 opaque worker connection");
            execute_predecessor_migration_v2(
                &mut client,
                MIGRATION_0009_SQL,
                &active,
                "migration 9 opaque lock race",
            )
        });

        wait_for_access_exclusive_lock(
            &config,
            "babylon_state.tick_graph_row",
            "epoch 9 never requested ACCESS EXCLUSIVE on the opaque predecessor table",
        );

        holder_transaction
            .execute(
                "INSERT INTO babylon_state.tick_graph_row (\
                     campaign_id, resolve_tick, row_ordinal, row_key, row_payload\
                 ) VALUES (\
                     '00000000-0000-0000-0000-000000000009'::pg_catalog.uuid, \
                     0, 0, '\\x01'::pg_catalog.bytea, '\\x'::pg_catalog.bytea\
                 )",
                &[],
            )
            .expect("concurrent opaque writer inserts while retaining its lock");
        holder_transaction
            .commit()
            .expect("concurrent opaque writer commits");
        let Err(RustPersistenceActivationErrorV2::Database {
            operation: "migration 9 opaque lock race",
            diagnostic: Some(_),
        }) = worker.join().expect("epoch 9 opaque worker joins")
        else {
            panic!("epoch 9 opaque lock race must retain its server diagnostic");
        };

        let proof = config
            .connect(NoTls)
            .expect("opaque lock-race proof connection")
            .query_one(
                "SELECT \
                     pg_catalog.to_regclass('babylon_state.tick_graph_row') IS NOT NULL, \
                     (SELECT pg_catalog.count(*) FROM babylon_state.tick_graph_row), \
                     (SELECT pg_catalog.count(*) \
                        FROM babylon_meta.persistence_authority_ledger \
                       WHERE state_tag = 2)",
                &[],
            )
            .expect("opaque lock-race refusal proof");
        assert!(proof.try_get::<_, bool>(0).expect("opaque table remains"));
        assert_eq!(proof.try_get::<_, i64>(1).expect("opaque row remains"), 1);
        assert_eq!(proof.try_get::<_, i64>(2).expect("active row absent"), 0);
        database.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PG17 runtime"]
    fn live_epoch_eleven_locks_before_its_serializable_inventory_snapshot() {
        let base = validated_base_config();
        let database = TestDatabase::create(&base, "velevenlock");
        let config = database.config(&base);
        establish_predecessor_authority_v2(&config).expect("epoch 9 predecessor activates");
        let expected = expected_v2_activation_report().expect("V2 authority rows compose");
        let migrations = compiled_committed_tick_v2_activation_migrations()
            .expect("V2 activation registry composes");
        let mut preparation = config.connect(NoTls).expect("V2 preparation connection");
        execute_v2_activation_migration(
            &mut preparation,
            migrations[0],
            &expected.prepared_row,
            "migration 10 lock fixture",
        )
        .expect("epoch 10 preparation commits");

        let mut holder = config.connect(NoTls).expect("V1 event writer connection");
        let mut holder_transaction = holder.transaction().expect("V1 event writer transaction");
        holder_transaction
            .batch_execute("LOCK TABLE babylon_state.tick_event_v1 IN ROW EXCLUSIVE MODE")
            .expect("V1 event writer lock held");

        let worker_config = config.clone();
        let active_row = expected.active_row.clone();
        let worker = thread::spawn(move || {
            let mut client = worker_config
                .connect(NoTls)
                .expect("epoch 11 worker connection");
            execute_v2_activation_migration(
                &mut client,
                migrations[1],
                &active_row,
                "migration 11 lock race",
            )
        });

        wait_for_access_exclusive_lock(
            &config,
            "babylon_state.tick_event_v1",
            "epoch 11 never requested ACCESS EXCLUSIVE before its inventory snapshot",
        );

        holder_transaction
            .execute(
                "INSERT INTO babylon_state.tick_event_v1 (\
                     campaign_id, resolve_tick, ordinal, event_type\
                 ) VALUES (\
                     '00000000-0000-0000-0000-000000000011'::pg_catalog.uuid, \
                     1, 0, 'EPOCH_ELEVEN_RACE'\
                 )",
                &[],
            )
            .expect("concurrent V1 writer inserts while retaining its lock");
        holder_transaction
            .commit()
            .expect("concurrent V1 writer commits");
        let Err(RustPersistenceActivationErrorV2::Database {
            operation: "migration 11 lock race",
            diagnostic: Some(_),
        }) = worker.join().expect("epoch 11 worker joins")
        else {
            panic!("epoch 11 lock race must retain its server diagnostic");
        };

        let proof = config
            .connect(NoTls)
            .expect("epoch 11 lock-race proof connection")
            .query_one(
                "SELECT \
                     pg_catalog.to_regclass('babylon_state.tick_event_v1') IS NOT NULL, \
                     (SELECT pg_catalog.count(*) FROM babylon_state.tick_event_v1), \
                     (SELECT pg_catalog.count(*) \
                        FROM babylon_meta.committed_tick_v2_authority_ledger \
                       WHERE state_tag = 2)",
                &[],
            )
            .expect("epoch 11 lock-race refusal proof");
        assert!(proof.try_get::<_, bool>(0).expect("V1 event table remains"));
        assert_eq!(proof.try_get::<_, i64>(1).expect("V1 event row remains"), 1);
        assert_eq!(proof.try_get::<_, i64>(2).expect("active row absent"), 0);
        database.cleanup();
    }

    fn assert_no_incident_choice_persists_and_restarts(
        config: &Config,
        seed: i64,
        campaign_id: CampaignId,
    ) {
        let (baseline_session, _) = runtime_fixture_with_seed(seed);
        let actions = OrderedPracticeActionBatchV1::empty(
            ReplaySessionIdV1::try_from("per281/runtime-live").expect("no-op session id"),
            1,
        )
        .expect("no-op action batch");
        let baseline_candidate = baseline_session
            .prepare_advance(&actions)
            .expect("baseline no-op candidate");
        let baseline_report = baseline_candidate.report();
        assert_eq!(baseline_report.report().choice_receipts.len(), 1);
        assert_eq!(
            baseline_report.report().choice_receipts[0].selected_outcome(),
            "NO_INCIDENT"
        );
        assert_eq!(seed, RUNTIME_CHOICE_SEEDS.1);
        assert_eq!(
            baseline_report.report().choice_receipts[0].draw_ticket(),
            RUNTIME_CHOICE_DRAW_TICKETS.1
        );
        assert!(baseline_report.report().committed_events.is_empty());
        assert_eq!(
            baseline_report.report().before,
            baseline_report.report().after
        );
        let baseline_choice = baseline_report.report().choice_receipts[0].clone();
        let baseline_hash = baseline_report.tick_content_hash();
        let baseline_envelope = prepare_committed_tick_v2(baseline_report)
            .expect("baseline no-op payload")
            .into_envelope(campaign_id)
            .expect("baseline no-op envelope");

        let (session, bundle) = runtime_fixture_with_seed(seed);
        let mut runtime = DurableReplayRuntimeV2::create(config, campaign_id, session, bundle)
            .expect("no-op runtime constructs");
        let mut sink = CollectingSink::default();
        let receipt = runtime
            .advance_and_commit(&mut sink, &actions)
            .expect("selected no-op commits with a receipt");
        assert_eq!(receipt.tick_content_hash(), baseline_hash);
        assert_eq!(receipt.considered(), 2);
        assert_eq!(receipt.fired(), 1);
        assert_eq!(receipt.choice_receipt_count(), 1);
        assert_eq!(receipt.event_count(), 0);
        assert_eq!(receipt.graph_before(), receipt.graph_after());
        assert!(sink.events.is_empty());
        assert_eq!(
            runtime
                .observe_committed_choice_receipts_v1(&receipt)
                .expect("no-op choice detail is retained"),
            [baseline_choice]
        );

        let stored = read_stored_typed_tick_v2(
            &mut config.connect(NoTls).expect("no-op stored tick connection"),
            campaign_id,
            receipt.resolve_tick().get(),
            "struggle/spark-conformance",
        )
        .expect("stored no-op tick reads")
        .expect("stored no-op tick exists");
        assert_eq!(
            stored.envelope().canonical_bytes(),
            baseline_envelope.canonical_bytes()
        );
        let counts = config
            .connect(NoTls)
            .expect("no-op receipt proof connection")
            .query_one(
                "SELECT \
                   (SELECT pg_catalog.count(*) FROM babylon_state.tick_choice_receipt_v1 \
                     WHERE campaign_id = $1::uuid AND resolve_tick = 1), \
                   (SELECT pg_catalog.count(*) FROM babylon_state.tick_event_v2 \
                     WHERE campaign_id = $1::uuid AND resolve_tick = 1), \
                   (SELECT selected_outcome FROM babylon_state.tick_choice_receipt_v1 \
                     WHERE campaign_id = $1::uuid AND resolve_tick = 1 AND encounter_ordinal = 0)",
                &[campaign_id.as_uuid()],
            )
            .expect("no-op receipt proof reads");
        assert_eq!(counts.try_get::<_, i64>(0).expect("no-op receipt count"), 1);
        assert_eq!(counts.try_get::<_, i64>(1).expect("no-op event count"), 0);
        assert_eq!(
            counts.try_get::<_, String>(2).expect("no-op outcome"),
            "NO_INCIDENT"
        );
        let reopened = DurableReplayRuntimeV2::open(config, campaign_id)
            .expect("no-op marker-owned checkpoint restarts");
        assert_eq!(reopened.last_committed_tick(), Some(receipt.resolve_tick()));
    }

    #[test]
    #[ignore = "requires the task-owned disposable PG17 runtime"]
    fn live_v2_activation_refuses_nonzero_incompatible_inventory() {
        let base = validated_base_config();
        let database = TestDatabase::create(&base, "vrefusal");
        let config = database.config(&base);
        establish_predecessor_authority_v2(&config)
            .expect("internal predecessor construction reaches epoch 9");
        config
            .connect(NoTls)
            .expect("incompatible-row connection")
            .execute(
                "INSERT INTO babylon_state.tick_event_v1 \
                 (campaign_id, resolve_tick, ordinal, event_type) \
                 VALUES ('31400000-0000-0000-0000-000000000001'::pg_catalog.uuid, 1, 0, 'INCOMPATIBLE')",
                &[],
            )
            .expect("one incompatible V1 event row inserts");

        assert_eq!(
            activate_rust_persistence_v2(&config),
            Err(
                RustPersistenceActivationErrorV2::PreActivationIncompatibleInventory {
                    relations: vec![PreActivationIncompatibleRelationV2 {
                        relation_name: "babylon_state.tick_event_v1",
                        observed_row_count: 1,
                    }],
                },
            )
        );
        let proof = config
            .connect(NoTls)
            .expect("refusal proof connection")
            .query_one(
                "SELECT \
                   pg_catalog.to_regclass('babylon_meta.committed_tick_v2_authority_ledger') IS NULL, \
                   pg_catalog.to_regclass('babylon_meta.committed_tick_v2_incompatible_inventory') IS NULL, \
                   pg_catalog.to_regclass('babylon_state.tick_event_v1') IS NOT NULL, \
                   pg_catalog.to_regclass('babylon_state.tick_event_v2') IS NULL, \
                   (SELECT pg_catalog.count(*) FROM babylon_state.tick_event_v1)",
                &[],
            )
            .expect("refusal state query");
        assert!(proof.try_get::<_, bool>(0).expect("V2 ledger absent"));
        assert!(proof.try_get::<_, bool>(1).expect("V2 inventory absent"));
        assert!(proof.try_get::<_, bool>(2).expect("V1 event remains"));
        assert!(proof
            .try_get::<_, bool>(3)
            .expect("additive V2 event absent"));
        assert_eq!(proof.try_get::<_, i64>(4).expect("V1 row remains"), 1);
        database.cleanup();
    }

    fn assert_material_change_tick_persisted(
        config: &Config,
        campaign_id: CampaignId,
        receipt: &CommittedTickReceiptV2,
        baseline_envelope: &CommittedTickEnvelopeV2,
    ) {
        let stored = read_stored_typed_tick_v2(
            &mut config.connect(NoTls).expect("stored tick connection"),
            campaign_id,
            receipt.resolve_tick().get(),
            "struggle/spark-conformance",
        )
        .expect("stored V2 tick reads")
        .expect("stored V2 tick exists");
        assert_eq!(
            stored.envelope().canonical_bytes(),
            baseline_envelope.canonical_bytes()
        );
        let provenance = config
            .connect(NoTls)
            .expect("provenance connection")
            .query_one(
                "SELECT receipt.selected_outcome, event.choice_receipt_ordinal, \
                        event.emitting_rule, event.event_type \
                   FROM babylon_state.tick_choice_receipt_v1 AS receipt \
                   JOIN babylon_state.tick_event_v2 AS event \
                     ON event.campaign_id = receipt.campaign_id \
                    AND event.resolve_tick = receipt.resolve_tick \
                    AND event.choice_receipt_ordinal = receipt.encounter_ordinal \
                  WHERE receipt.campaign_id = $1::uuid AND receipt.resolve_tick = 1",
                &[campaign_id.as_uuid()],
            )
            .expect("projected event provenance reads");
        assert_eq!(
            provenance
                .try_get::<_, String>(0)
                .expect("selected outcome"),
            "EXCESSIVE_FORCE"
        );
        assert_eq!(provenance.try_get::<_, i64>(1).expect("receipt ordinal"), 0);
        assert_eq!(
            provenance.try_get::<_, String>(2).expect("emitting rule"),
            "struggle/spark-recognizer"
        );
        assert_eq!(
            provenance.try_get::<_, String>(3).expect("event type"),
            "EXCESSIVE_FORCE"
        );

        let reopened = DurableReplayRuntimeV2::open(config, campaign_id)
            .expect("marker-owned checkpoint restarts");
        assert_eq!(reopened.last_committed_tick(), Some(receipt.resolve_tick()));
    }

    fn assert_material_change_receipt(
        receipt: &CommittedTickReceiptV2,
        baseline_hash: TickContentHashV1,
    ) {
        assert_eq!(receipt.resolve_tick().get(), 1);
        assert_eq!(
            receipt.commit_disposition(),
            ReplayCommitDispositionV1::Committed
        );
        assert_eq!(receipt.tick_content_hash(), baseline_hash);
        assert_eq!(receipt.considered(), 2);
        assert_eq!(receipt.fired(), 2);
        assert_eq!(receipt.event_count(), 1);
        assert_eq!(receipt.choice_receipt_count(), 1);
        assert_eq!(receipt.audit_receipt_count(), 4);
        assert!(receipt.material_row_count() > 0);
    }

    #[test]
    #[ignore = "requires the task-owned disposable PER-20 PostgreSQL runtime"]
    fn live_marker_last_commit_and_restart_are_atomic() {
        let base = validated_base_config();
        verify_frozen_python_estate_activation(&base);
        let template = validated_template_name();
        let database = TestDatabase::create_from_template(&base, &template, "runtimeatomic");
        let config = database.config(&base);
        let campaign_id =
            CampaignId::from_uuid(Uuid::from_u128(0x2810_0000_0000_0000_0000_0000_0000_00a1));
        let (material_change_seed, no_incident_seed) = RUNTIME_CHOICE_SEEDS;
        assert_ne!(material_change_seed, no_incident_seed);
        let (baseline_session, _) = runtime_fixture_with_seed(material_change_seed);
        let baseline_actions = OrderedPracticeActionBatchV1::empty(
            ReplaySessionIdV1::try_from("per281/runtime-live").expect("baseline session id"),
            1,
        )
        .expect("baseline action batch");
        let baseline_candidate = baseline_session
            .prepare_advance(&baseline_actions)
            .expect("baseline material-change candidate");
        let baseline_choice = baseline_candidate.report().report().choice_receipts[0].clone();
        assert_eq!(baseline_choice.selected_outcome(), "EXCESSIVE_FORCE");
        assert_eq!(baseline_choice.draw_ticket(), RUNTIME_CHOICE_DRAW_TICKETS.0);
        let baseline_hash = baseline_candidate.report().tick_content_hash();
        let baseline_envelope = prepare_committed_tick_v2(baseline_candidate.report())
            .expect("baseline material-change payload")
            .into_envelope(campaign_id)
            .expect("baseline material-change envelope");

        let (session, bundle) = runtime_fixture_with_seed(material_change_seed);
        let mut runtime = DurableReplayRuntimeV2::create(&config, campaign_id, session, bundle)
            .expect("runtime constructs after activation");
        let metadata = crate::metadata::RetainedMetadataStoreV1::new(&config);
        seed_and_verify_navigation_metadata(&metadata, campaign_id);
        let actions = OrderedPracticeActionBatchV1::empty(
            runtime.foundation().replay_session_identity().clone(),
            1,
        )
        .expect("first action batch");
        let mut sink = CollectingSink::default();
        let before_graph = runtime
            .observe_current_stable_graph_state_v1()
            .expect("pre-selection graph observation");

        LIVE_FAIL_BEFORE_MARKER.store(true, Ordering::SeqCst);
        let Err(RustPersistenceRuntimeErrorV2::Database {
            operation: "injected pre-marker refusal",
            diagnostic: Some(diagnostic),
        }) = runtime.advance_and_commit(&mut sink, &actions)
        else {
            panic!("injected refusal must retain its server diagnostic");
        };
        assert_eq!(diagnostic.sqlstate(), Some("22012"));
        assert!(sink.events.is_empty());
        assert_eq!(runtime.last_committed_tick(), None);
        assert_eq!(runtime.session.completed_tick(), 0);
        assert!(runtime.last_choice_receipts.is_empty());
        assert_eq!(committed_payload_row_count(&config, campaign_id), 0);
        assert_eq!(marker_row_count(&config, campaign_id), 0);
        assert_eq!(
            runtime
                .observe_current_stable_graph_state_v1()
                .expect("failed tick graph remains observable"),
            before_graph
        );

        let receipt = runtime
            .advance_and_commit(&mut sink, &actions)
            .expect("identical retry commits");
        assert_material_change_receipt(&receipt, baseline_hash);
        assert_eq!(runtime.last_committed_tick(), Some(receipt.resolve_tick()));
        assert_eq!(sink.events.len(), 1);
        let observed_choices = runtime
            .observe_committed_choice_receipts_v1(&receipt)
            .expect("post-commit choice detail is process-local");
        assert_eq!(observed_choices, [baseline_choice]);
        assert_eq!(marker_row_count(&config, campaign_id), 1);
        assert_eq!(
            metadata
                .campaign(campaign_id)
                .expect("advanced catalog reads")
                .expect("campaign remains retained")
                .last_tick(),
            1
        );

        assert_material_change_tick_persisted(&config, campaign_id, &receipt, &baseline_envelope);

        assert_no_incident_choice_persists_and_restarts(
            &config,
            no_incident_seed,
            CampaignId::from_uuid(Uuid::from_u128(0x2810_0000_0000_0000_0000_0000_0000_00b1)),
        );
        database.cleanup();
    }

    fn seed_and_verify_navigation_metadata(
        metadata: &crate::metadata::RetainedMetadataStoreV1,
        campaign_id: CampaignId,
    ) {
        let initial_catalog = metadata
            .campaign(campaign_id)
            .expect("catalog reads")
            .expect("runtime creates the retained catalog row");
        assert_eq!(initial_catalog.last_tick(), 0);
        metadata
            .replace_watchlist(
                campaign_id,
                &["social-class/C001".to_owned(), "territory/wayne".to_owned()],
            )
            .expect("watchlist replaces atomically");
        metadata
            .replace_jumplist(
                campaign_id,
                &["territory/wayne".to_owned(), "territory/wayne".to_owned()],
            )
            .expect("jumplist preserves legal duplicates");
        metadata
            .replace_breadcrumbs(
                campaign_id,
                &["world/michigan".to_owned(), "territory/wayne".to_owned()],
            )
            .expect("breadcrumbs replace atomically");
        assert_eq!(
            metadata
                .watchlist(campaign_id)
                .expect("watchlist reads")
                .iter()
                .map(crate::metadata::WatchlistRowV1::entity_id)
                .collect::<Vec<_>>(),
            ["social-class/C001", "territory/wayne"]
        );
        assert_eq!(
            metadata
                .jumplist(campaign_id)
                .expect("jumplist reads")
                .iter()
                .map(crate::metadata::JumplistRowV1::entity_id)
                .collect::<Vec<_>>(),
            ["territory/wayne", "territory/wayne"]
        );
        assert_eq!(
            metadata
                .breadcrumbs(campaign_id)
                .expect("breadcrumbs read")
                .iter()
                .map(crate::metadata::BreadcrumbRowV1::entity_id)
                .collect::<Vec<_>>(),
            ["world/michigan", "territory/wayne"]
        );
    }

    fn verify_frozen_python_estate_activation(base: &Config) {
        let database = TestDatabase::create(base, "runtimelegacy");
        let config = database.config(base);
        crate::schema_epoch::legacy_epoch_fixture::build_frozen_python_estate(&config);
        let first = activate_rust_persistence_v2(&config)
            .expect("the exact empty frozen Python estate activates");
        let second =
            activate_rust_persistence_v2(&config).expect("terminal Rust activation is idempotent");
        assert_eq!(first, second);
        let row = config
            .connect(NoTls)
            .expect("disposition connection")
            .query_one(
                "SELECT pg_catalog.count(*), \
                        pg_catalog.bool_and(observed_row_count = 0 \
                          AND ordered_semantic_sha256 = pg_catalog.sha256(''::pg_catalog.bytea) \
                          AND disposition_tag = 1), \
                        pg_catalog.to_regclass('public.game_session') IS NULL, \
                        pg_catalog.to_regclass('public._babylon_schema_stamp') IS NULL, \
                        pg_catalog.to_regclass('public.document_chunk') IS NOT NULL \
                 FROM babylon_meta.python_relation_disposition_v1",
                &[],
            )
            .expect("disposition proof");
        assert_eq!(row.try_get::<_, i64>(0).expect("disposition count"), 61);
        assert!(row.try_get::<_, bool>(1).expect("zero-row proof"));
        assert!(row.try_get::<_, bool>(2).expect("game authority retired"));
        assert!(row
            .try_get::<_, bool>(3)
            .expect("Python schema stamp retired"));
        assert!(row.try_get::<_, bool>(4).expect("AI periphery retained"));
        database.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PER-20 PostgreSQL runtime"]
    fn live_commit_ambiguity_reconciliation_is_exact() {
        let base = validated_base_config();
        let template = validated_template_name();
        let database = TestDatabase::create_from_template(&base, &template, "runtimeambiguous");
        let config = database.config(&base);
        let campaign_id =
            CampaignId::from_uuid(Uuid::from_u128(0x2810_0000_0000_0000_0000_0000_0000_00a2));
        let (session, bundle) = runtime_fixture_with_seed(RUNTIME_CHOICE_SEEDS.0);
        let mut runtime = DurableReplayRuntimeV2::create(&config, campaign_id, session, bundle)
            .expect("runtime constructs after activation");
        let actions = OrderedPracticeActionBatchV1::empty(
            runtime.foundation().replay_session_identity().clone(),
            1,
        )
        .expect("first action batch");
        let mut sink = CollectingSink::default();

        LIVE_RECONCILIATIONS.store(0, Ordering::SeqCst);
        LIVE_COMMIT_AS_AMBIGUOUS.store(true, Ordering::SeqCst);
        let receipt = runtime
            .advance_and_commit(&mut sink, &actions)
            .expect("committed ambiguity reconciles through the exact marker");
        assert_eq!(receipt.resolve_tick().get(), 1);
        assert_eq!(
            receipt.commit_disposition(),
            ReplayCommitDispositionV1::ReconciledAfterAmbiguousCommit
        );
        assert_eq!(LIVE_RECONCILIATIONS.load(Ordering::SeqCst), 1);
        assert_eq!(sink.events.len(), 1);
        assert_eq!(marker_row_count(&config, campaign_id), 1);

        let reopened = DurableReplayRuntimeV2::open(&config, campaign_id)
            .expect("reconciled marker is the restart root");
        assert_eq!(reopened.last_committed_tick(), Some(receipt.resolve_tick()));
        database.cleanup();
    }

    const COUNTY_MAP_RULE: &str = r#"(rule class-dynamics/territory-county-map-probe
  :role mechanic
  :evidence derived
  :material-basis "PER-22: the declared territory-county mapping persists at campaign foundation"
  :fuel 8
  (bindings (binding fips :field territory/county-fips))
  (when (> fips 99999))
  (effects
    (emit EventType/PER22_PROBE)))"#;

    const COUNTY_MAP_SCENARIO: &str = r"
(scenario per281/territory-county-map
  (defvocabulary NodeType (TERRITORY))
  (deffield territory/county-fips int extensive)
  (node wayne NodeType/TERRITORY (territory/county-fips 26163)))
";

    fn county_map_fixture(
        scenario: &str,
    ) -> (
        ReplayTickSession<HypergraphStore>,
        FoundationContentBundleV1,
    ) {
        let (_, rules) = split_content(COUNTY_MAP_RULE).expect("county map rule parses");
        let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
        let content = ContentDigest {
            defines_hash: sha256_of(DEFINES),
            rules_hash: rules_hash_of(&forms).expect("county map rule hashes"),
        };
        let foundation = michigan_dynamic_hex_foundation_v1().expect("foundation decodes");
        let mut reference_manifest = REFERENCE_BUNDLE_DOMAIN.to_vec();
        reference_manifest.extend_from_slice(&foundation.base_reference_cohort_digest());
        reference_manifest.extend_from_slice(&foundation.r8_section_digest());
        let reference = RefDigestV1::from_bytes(foundation.reference_bundle_digest());
        let session = ReplayTickSession::new(
            scenario,
            None,
            COUNTY_MAP_RULE,
            HypergraphStore::new(),
            ReplaySessionIdV1::try_from("per281/territory-county-map").expect("session id"),
            ReplaySeed::new(281),
            content,
            reference,
            MaterialStateV1::try_new(foundation).expect("material state"),
        )
        .expect("county map session prepares");
        let bundle = FoundationContentBundleV1::try_new(
            scenario,
            None,
            COUNTY_MAP_RULE,
            DEFINES,
            &reference_manifest,
        )
        .expect("county map content bundle");
        (session, bundle)
    }

    #[test]
    #[ignore = "requires the task-owned disposable PG17 runtime"]
    fn live_territory_county_map_persists_at_campaign_foundation() {
        let base = validated_base_config();
        let template = validated_template_name();
        let database = TestDatabase::create_from_template(&base, &template, "countymappersist");
        let config = database.config(&base);
        let campaign_id =
            CampaignId::from_uuid(Uuid::from_u128(0x2810_0000_0000_0000_0000_0000_0000_00c1));
        let (session, bundle) = county_map_fixture(COUNTY_MAP_SCENARIO);
        let runtime = DurableReplayRuntimeV2::create(&config, campaign_id, session, bundle)
            .expect("runtime constructs after activation");
        assert_eq!(runtime.last_committed_tick(), None);
        let rows: Vec<(String, String)> = config
            .connect(NoTls)
            .expect("county map read connection")
            .query(
                "SELECT territory_local_name, county_geoid \
                 FROM babylon_meta.territory_county_map_v1 \
                 WHERE campaign_id = $1::uuid ORDER BY territory_local_name",
                &[campaign_id.as_uuid()],
            )
            .expect("county map rows read")
            .into_iter()
            .map(|row| {
                (
                    row.try_get(0).expect("local name decodes"),
                    row.try_get(1).expect("county geoid decodes"),
                )
            })
            .collect();
        assert_eq!(rows, [("wayne".to_owned(), "26163".to_owned())]);
        database.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PG17 runtime"]
    fn live_territory_county_map_refuses_missing_declared_county_fips() {
        let base = validated_base_config();
        let template = validated_template_name();
        let database = TestDatabase::create_from_template(&base, &template, "countymapmissing");
        let config = database.config(&base);
        let campaign_id =
            CampaignId::from_uuid(Uuid::from_u128(0x2810_0000_0000_0000_0000_0000_0000_00c2));
        let scenario = r"
(scenario per281/territory-county-map-missing
  (defvocabulary NodeType (TERRITORY))
  (deffield territory/county-fips int extensive)
  (node wayne NodeType/TERRITORY))
";
        let (session, bundle) = county_map_fixture(scenario);
        let Err(error) = DurableReplayRuntimeV2::create(&config, campaign_id, session, bundle)
        else {
            panic!("a territory node without the declared county-fips refuses");
        };
        assert!(matches!(
            error,
            RustPersistenceRuntimeErrorV2::TerritoryCountyMap(
                crate::TerritoryCountyMapErrorV1::MissingCountyFips { .. }
            )
        ));
        let campaign_rows: i64 = config
            .connect(NoTls)
            .expect("refusal proof connection")
            .query_one(
                "SELECT pg_catalog.count(*) FROM babylon_meta.campaign \
                 WHERE campaign_id = $1::uuid",
                &[campaign_id.as_uuid()],
            )
            .expect("campaign count")
            .try_get(0)
            .expect("campaign count decodes");
        assert_eq!(campaign_rows, 0);
        database.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PG17 runtime"]
    fn live_territory_county_map_refuses_duplicate_county_geoid() {
        let base = validated_base_config();
        let template = validated_template_name();
        let database = TestDatabase::create_from_template(&base, &template, "countymapdupe");
        let config = database.config(&base);
        let campaign_id =
            CampaignId::from_uuid(Uuid::from_u128(0x2810_0000_0000_0000_0000_0000_0000_00c3));
        let scenario = r"
(scenario per281/territory-county-map-duplicate
  (defvocabulary NodeType (TERRITORY))
  (deffield territory/county-fips int extensive)
  (node wayne NodeType/TERRITORY (territory/county-fips 26163))
  (node clone NodeType/TERRITORY (territory/county-fips 26163)))
";
        let (session, bundle) = county_map_fixture(scenario);
        let Err(error) = DurableReplayRuntimeV2::create(&config, campaign_id, session, bundle)
        else {
            panic!("duplicate declared county geoid refuses");
        };
        assert!(matches!(
            error,
            RustPersistenceRuntimeErrorV2::TerritoryCountyMap(
                crate::TerritoryCountyMapErrorV1::DuplicateCountyGeoid { .. }
            )
        ));
        let campaign_rows: i64 = config
            .connect(NoTls)
            .expect("refusal proof connection")
            .query_one(
                "SELECT pg_catalog.count(*) FROM babylon_meta.campaign \
                 WHERE campaign_id = $1::uuid",
                &[campaign_id.as_uuid()],
            )
            .expect("campaign count")
            .try_get(0)
            .expect("campaign count decodes");
        assert_eq!(campaign_rows, 0);
        database.cleanup();
    }

    fn county_map_rows(config: &Config, campaign_id: CampaignId) -> Vec<(String, String)> {
        config
            .connect(NoTls)
            .expect("county map read connection")
            .query(
                "SELECT territory_local_name, county_geoid \
                 FROM babylon_meta.territory_county_map_v1 \
                 WHERE campaign_id = $1::uuid ORDER BY territory_local_name",
                &[campaign_id.as_uuid()],
            )
            .expect("county map rows read")
            .into_iter()
            .map(|row| {
                (
                    row.try_get(0).expect("local name decodes"),
                    row.try_get(1).expect("county geoid decodes"),
                )
            })
            .collect()
    }

    #[test]
    #[ignore = "requires the task-owned disposable PG17 runtime"]
    fn live_territory_county_map_backfills_when_opening_a_pre_feature_campaign() {
        // An already-founded campaign whose mapping rows are absent (a
        // pre-feature foundation) must gain them idempotently on open,
        // without overwriting any existing row.
        let base = validated_base_config();
        let template = validated_template_name();
        let database = TestDatabase::create_from_template(&base, &template, "countymapbackfill");
        let config = database.config(&base);
        let campaign_id =
            CampaignId::from_uuid(Uuid::from_u128(0x2810_0000_0000_0000_0000_0000_0000_00c4));
        let (session, bundle) = county_map_fixture(COUNTY_MAP_SCENARIO);
        let runtime = DurableReplayRuntimeV2::create(&config, campaign_id, session, bundle)
            .expect("runtime constructs after activation");
        drop(runtime);
        // Simulate the pre-feature state: the campaign exists, the rows do not.
        let deleted = config
            .connect(NoTls)
            .expect("pre-feature simulation connection")
            .execute(
                "DELETE FROM babylon_meta.territory_county_map_v1 WHERE campaign_id = $1::uuid",
                &[campaign_id.as_uuid()],
            )
            .expect("pre-feature rows removed");
        assert_eq!(deleted, 1);
        assert!(county_map_rows(&config, campaign_id).is_empty());

        let reopened = DurableReplayRuntimeV2::open(&config, campaign_id)
            .expect("open backfills the declared mapping");
        assert_eq!(reopened.last_committed_tick(), None);
        assert_eq!(
            county_map_rows(&config, campaign_id),
            [("wayne".to_owned(), "26163".to_owned())]
        );
        // A second open reconciles against identical rows without writing or
        // refusing.
        let reopened_again = DurableReplayRuntimeV2::open(&config, campaign_id)
            .expect("a second open reconciles idempotently");
        assert_eq!(reopened_again.last_committed_tick(), None);
        assert_eq!(
            county_map_rows(&config, campaign_id),
            [("wayne".to_owned(), "26163".to_owned())]
        );
        database.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PG17 runtime"]
    fn live_territory_county_map_open_refuses_divergent_stored_rows() {
        // Stored rows are never overwritten: when they disagree with the
        // scenario-declared mapping, open refuses loudly.
        let base = validated_base_config();
        let template = validated_template_name();
        let database = TestDatabase::create_from_template(&base, &template, "countymapdiverge");
        let config = database.config(&base);
        let campaign_id =
            CampaignId::from_uuid(Uuid::from_u128(0x2810_0000_0000_0000_0000_0000_0000_00c5));
        let (session, bundle) = county_map_fixture(COUNTY_MAP_SCENARIO);
        let runtime = DurableReplayRuntimeV2::create(&config, campaign_id, session, bundle)
            .expect("runtime constructs after activation");
        drop(runtime);
        let updated = config
            .connect(NoTls)
            .expect("divergence simulation connection")
            .execute(
                "UPDATE babylon_meta.territory_county_map_v1 SET county_geoid = '26099' \
                 WHERE campaign_id = $1::uuid",
                &[campaign_id.as_uuid()],
            )
            .expect("divergent row installed");
        assert_eq!(updated, 1);

        let Err(error) = DurableReplayRuntimeV2::open(&config, campaign_id) else {
            panic!("divergent stored mapping rows refuse open");
        };
        assert!(matches!(
            error,
            RustPersistenceRuntimeErrorV2::TerritoryCountyMap(
                crate::TerritoryCountyMapErrorV1::StoredMappingDiverged { .. }
            )
        ));
        // The durable rows were not overwritten by the refused open.
        assert_eq!(
            county_map_rows(&config, campaign_id),
            [("wayne".to_owned(), "26099".to_owned())]
        );
        database.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PG17 runtime"]
    fn live_concurrent_identical_retry_reconciles_after_the_campaign_lock() {
        let base = validated_base_config();
        let template = validated_template_name();
        let database = TestDatabase::create_from_template(&base, &template, "runtimeconcurrent");
        let config = database.config(&base);
        let campaign_id =
            CampaignId::from_uuid(Uuid::from_u128(0x2810_0000_0000_0000_0000_0000_0000_00a4));
        let (session, bundle) = runtime_fixture_with_seed(RUNTIME_CHOICE_SEEDS.0);
        let runtime = DurableReplayRuntimeV2::create(&config, campaign_id, session, bundle)
            .expect("campaign foundation installs before concurrent writers");
        drop(runtime);

        let before_lock_barrier = Arc::new(Barrier::new(3));
        *LIVE_BEFORE_CAMPAIGN_LOCK_BARRIER
            .lock()
            .expect("install pre-lock barrier") = Some(Arc::clone(&before_lock_barrier));
        let mut workers = Vec::new();
        for _ in 0..2 {
            let worker_config = config.clone();
            workers.push(thread::spawn(move || {
                let (session, _) = runtime_fixture_with_seed(RUNTIME_CHOICE_SEEDS.0);
                let actions = OrderedPracticeActionBatchV1::empty(
                    ReplaySessionIdV1::try_from("per281/runtime-live")
                        .expect("concurrent session id"),
                    1,
                )
                .expect("concurrent action batch");
                let candidate = session
                    .prepare_advance(&actions)
                    .expect("concurrent candidate prepares");
                let prepared = prepare_committed_tick_v2(candidate.report())
                    .expect("concurrent payload prepares");
                let checkpoint = CommittedFullCheckpointV1::capture(
                    campaign_id,
                    prepared.resolve_tick(),
                    candidate.report(),
                )
                .expect("concurrent checkpoint composes");
                let envelope = prepared
                    .into_envelope(campaign_id)
                    .expect("concurrent envelope composes");
                commit_typed_tick_v2(
                    &worker_config,
                    campaign_id,
                    candidate.report(),
                    &checkpoint,
                    &envelope,
                    FoundationContentLayout::V1,
                )
            }));
        }

        before_lock_barrier.wait();
        *LIVE_BEFORE_CAMPAIGN_LOCK_BARRIER
            .lock()
            .expect("remove pre-lock barrier") = None;
        let dispositions = workers
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .expect("concurrent writer joins")
                    .expect("identical concurrent writer reconciles")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            dispositions
                .iter()
                .filter(|&&value| value == ReplayCommitDispositionV1::Committed)
                .count(),
            1
        );
        assert_eq!(
            dispositions
                .iter()
                .filter(|&&value| {
                    value == ReplayCommitDispositionV1::ReconciledAfterAmbiguousCommit
                })
                .count(),
            1
        );
        assert_eq!(marker_row_count(&config, campaign_id), 1);
        database.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PER-20 PostgreSQL runtime"]
    fn live_retry_and_restart_refuse_a_mutated_typed_payload() {
        let base = validated_base_config();
        let template = validated_template_name();
        let database = TestDatabase::create_from_template(&base, &template, "runtimemutated");
        let config = database.config(&base);
        let campaign_id =
            CampaignId::from_uuid(Uuid::from_u128(0x2810_0000_0000_0000_0000_0000_0000_00a3));
        let (session, bundle) = runtime_fixture();
        let mut runtime = DurableReplayRuntimeV2::create(&config, campaign_id, session, bundle)
            .expect("runtime constructs after activation");
        let actions = OrderedPracticeActionBatchV1::empty(
            runtime.foundation().replay_session_identity().clone(),
            1,
        )
        .expect("first action batch");
        runtime
            .advance_and_commit(&mut CollectingSink::default(), &actions)
            .expect("first tick commits");
        config
            .connect(NoTls)
            .expect("mutation connection")
            .execute(
                "UPDATE babylon_state.graph_node_f64_v1 SET value_bits = 0 \
                 WHERE campaign_id = $1::uuid AND resolve_tick = 1",
                &[campaign_id.as_uuid()],
            )
            .expect("typed test row mutates");

        assert_eq!(
            DurableReplayRuntimeV2::open(&config, campaign_id).map(|_| ()),
            Err(RustPersistenceRuntimeErrorV2::CampaignConflict)
        );

        let (retry_session, _) = runtime_fixture();
        let candidate = retry_session
            .prepare_advance(&actions)
            .expect("retry candidate");
        let prepared = prepare_committed_tick_v2(candidate.report()).expect("retry envelope input");
        let envelope = prepared.into_envelope(campaign_id).expect("retry envelope");
        assert_eq!(
            marker_matches_envelope_v2(
                &mut config.connect(NoTls).expect("retry connection"),
                candidate.report(),
                &envelope,
            ),
            Err(RustPersistenceRuntimeErrorV2::CampaignConflict)
        );
        database.cleanup();
    }

    #[test]
    #[ignore = "requires the task-owned disposable PER-20 PostgreSQL runtime"]
    fn live_restart_uses_the_latest_full_checkpoint_not_the_foundation_history() {
        let base = validated_base_config();
        let template = validated_template_name();
        let database = TestDatabase::create_from_template(&base, &template, "runtimelatest");
        let config = database.config(&base);
        let campaign_id =
            CampaignId::from_uuid(Uuid::from_u128(0x2810_0000_0000_0000_0000_0000_0000_00a4));
        let (session, bundle) = runtime_fixture();
        let mut runtime = DurableReplayRuntimeV2::create(&config, campaign_id, session, bundle)
            .expect("runtime constructs after activation");
        for resolve_tick in 1..=3 {
            let actions = OrderedPracticeActionBatchV1::empty(
                runtime.foundation().replay_session_identity().clone(),
                resolve_tick,
            )
            .expect("action batch");
            runtime
                .advance_and_commit(&mut CollectingSink::default(), &actions)
                .expect("tick commits");
        }
        let next_actions = OrderedPracticeActionBatchV1::empty(
            runtime.foundation().replay_session_identity().clone(),
            4,
        )
        .expect("next action batch");
        let uninterrupted_hash = runtime
            .session
            .prepare_advance(&next_actions)
            .expect("uninterrupted candidate")
            .report()
            .tick_content_hash();
        config
            .connect(NoTls)
            .expect("historical mutation connection")
            .execute(
                "DELETE FROM babylon_state.tick_action_batch_v1 \
                 WHERE campaign_id = $1::uuid AND resolve_tick = 1",
                &[campaign_id.as_uuid()],
            )
            .expect("old action row deletes");

        let reopened = DurableReplayRuntimeV2::open(&config, campaign_id)
            .expect("latest full checkpoint is a sufficient restart root");
        assert_eq!(
            reopened
                .last_committed_tick()
                .map(CommittedResolveTickV1::get),
            Some(3)
        );
        let restored_hash = reopened
            .session
            .prepare_advance(&next_actions)
            .expect("restored candidate")
            .report()
            .tick_content_hash();
        assert_eq!(restored_hash, uninterrupted_hash);
        database.cleanup();
    }

    fn runtime_fixture() -> (
        ReplayTickSession<HypergraphStore>,
        FoundationContentBundleV1,
    ) {
        runtime_fixture_with_seed(281)
    }

    fn runtime_fixture_with_seed(
        seed: i64,
    ) -> (
        ReplayTickSession<HypergraphStore>,
        FoundationContentBundleV1,
    ) {
        let (_, rules) = split_content(RULE).expect("live rule parses");
        let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
        let content = ContentDigest {
            defines_hash: sha256_of(DEFINES),
            rules_hash: rules_hash_of(&forms).expect("live rule hashes"),
        };
        let foundation = michigan_dynamic_hex_foundation_v1().expect("foundation decodes");
        let mut reference_manifest = REFERENCE_BUNDLE_DOMAIN.to_vec();
        reference_manifest.extend_from_slice(&foundation.base_reference_cohort_digest());
        reference_manifest.extend_from_slice(&foundation.r8_section_digest());
        assert_eq!(
            sha256_of(&reference_manifest),
            foundation.reference_bundle_digest()
        );
        let reference = RefDigestV1::from_bytes(foundation.reference_bundle_digest());
        let session = ReplayTickSession::new(
            SCENARIO,
            None,
            RULE,
            HypergraphStore::new(),
            ReplaySessionIdV1::try_from("per281/runtime-live").expect("session id"),
            ReplaySeed::new(seed),
            content,
            reference,
            MaterialStateV1::try_new(foundation).expect("material state"),
        )
        .expect("tick-zero session prepares");
        let bundle =
            FoundationContentBundleV1::try_new(SCENARIO, None, RULE, DEFINES, &reference_manifest)
                .expect("content bundle");
        (session, bundle)
    }

    fn validated_base_config() -> Config {
        assert_eq!(std::env::var(ACK_ENV).as_deref(), Ok(ACK));
        let canary = std::env::var(CANARY_ENV).expect("runner supplies the disposable canary");
        assert_eq!(canary.len(), 32);
        let dsn = std::env::var(DSN_ENV).expect("runner supplies the disposable DSN");
        let config = Config::from_str(&dsn).expect("runner DSN parses");
        validate_legacy_connection_target(&config).expect("loopback target");
        assert_eq!(config.get_user(), Some("test"));
        assert_eq!(config.get_dbname(), Some("postgres"));
        let actual: Option<String> = config
            .connect(NoTls)
            .expect("canary connection")
            .query_one(
                "SELECT pg_catalog.current_setting('babylon.per20_disposable', true)",
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

    fn marker_row_count(config: &Config, campaign_id: CampaignId) -> i64 {
        config
            .connect(NoTls)
            .expect("marker connection")
            .query_one(
                "SELECT pg_catalog.count(*) FROM babylon_state.tick_commit \
                 WHERE campaign_id = $1::uuid",
                &[campaign_id.as_uuid()],
            )
            .expect("marker count")
            .try_get(0)
            .expect("marker count decodes")
    }

    fn committed_payload_row_count(config: &Config, campaign_id: CampaignId) -> i64 {
        config
            .connect(NoTls)
            .expect("payload connection")
            .query_one(
                "SELECT \
                   (SELECT pg_catalog.count(*) FROM babylon_state.tick_action_batch_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.graph_node_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.graph_node_f64_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.graph_node_currency_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.graph_edge_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.graph_edge_f64_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.graph_hyperedge_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.graph_hyperedge_member_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.graph_hyperedge_f64_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.world_register_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.territory_state_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.territory_state_field_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.hex_state_delta_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.organization_state_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.organization_territory_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.organization_state_field_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.tick_event_v2 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.tick_event_field_v2 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.tick_choice_receipt_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.tick_choice_receipt_branch_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.tick_choice_receipt_carrier_element_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.checkpoint_manifest WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.checkpoint_section_v1 WHERE campaign_id = $1::uuid) + \
                   (SELECT pg_catalog.count(*) FROM babylon_state.archive_dirty_receipt_v1 WHERE campaign_id = $1::uuid)",
                &[campaign_id.as_uuid()],
            )
            .expect("payload count")
            .try_get(0)
            .expect("payload count decodes")
    }

    struct TestDatabase {
        name: String,
        admin: Config,
        active: bool,
    }

    impl TestDatabase {
        fn create(base: &Config, label: &str) -> Self {
            assert!(label.bytes().all(|byte| byte.is_ascii_lowercase()));
            let name = format!("per281_runtime_{label}_{}", std::process::id());
            let mut admin = base.clone();
            admin.dbname("postgres");
            let sql = format!("CREATE DATABASE \"{name}\" OWNER test TEMPLATE template1");
            admin
                .connect(NoTls)
                .expect("admin connection")
                .batch_execute(&sql)
                .expect("scratch database creation");
            Self {
                name,
                admin,
                active: true,
            }
        }

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
            let observation = database
                .config(base)
                .connect(NoTls)
                .expect("runtime clone connection")
                .query_one(
                    "SELECT \
                       (SELECT pg_catalog.string_agg(ordinal::pg_catalog.text || ':' || \
                                state_tag::pg_catalog.text || ':' || schema_epoch::pg_catalog.text, \
                                ',' ORDER BY ordinal) \
                        FROM babylon_meta.persistence_authority_ledger), \
                       (SELECT pg_catalog.count(*) FROM babylon_meta.campaign)",
                    &[],
                )
                .expect("runtime clone observation");
            assert_eq!(
                observation
                    .try_get::<_, String>(0)
                    .expect("authority ledger decodes"),
                "1:1:8,2:2:9"
            );
            assert_eq!(
                observation
                    .try_get::<_, i64>(1)
                    .expect("campaign count decodes"),
                0
            );
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
    use babylon_bsl::rule_pipeline::split_content;
    use babylon_bsl::rules_hash_of;
    use babylon_bsl::structural_verbs::CollectingSink;
    use babylon_graph::hypergraph_store::HypergraphStore;
    use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
    use babylon_kernel::tick_content_hash::RefDigestV1;
    use babylon_kernel::{sha256_of, ContentDigest};
    use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
    use babylon_tick::material_state::MaterialStateV1;
    use babylon_tick::replay_session::{ReplayCommitDispositionV1, ReplayTickSession};

    use crate::michigan_dynamic_hex_foundation_v1;

    use super::{
        prepare_committed_tick_v2, v2_authority_contract_digests,
        validate_postgresql_server_version_num_v2, CampaignFoundationV1, CampaignId,
        CommittedResolveTickV1, CommittedTickAuthorityLedgerRowV2, CommittedTickReceiptV2,
        DurableReplayRuntimeV2, FoundationContentBundleV1, RustPersistenceActivationErrorV2,
        RustPersistenceRuntimeErrorV2, ACTIVE_V2_CUTOVER_CONTRACT, MIGRATION_0009_SQL,
        PRE_ACTIVATION_INCOMPATIBLE_RELATIONS_V2,
    };

    const SCENARIO: &str = r"
(scenario demo/runtime-prepare
  (defvocabulary NodeType (SOCIAL_CLASS))
  (deffield social-class/draw coefficient extensive)
  (node class-a NodeType/SOCIAL_CLASS (social-class/draw 0.0c)))
";
    const RULE: &str = r#"
(rule production/runtime-prepare
  :role mechanic
  :evidence derived
  :material-basis "prepared persistence input law"
  :fuel 32
  (bindings (binding draw :field social-class/draw))
  (when #t)
  (effects
    (update-node self social-class/draw (set 0.25c))
    (emit EventType/RUNTIME_PREPARED (subject self))))
"#;

    #[test]
    fn pre_activation_inventory_is_closed_sorted_and_matches_transactional_targets() {
        assert_eq!(PRE_ACTIVATION_INCOMPATIBLE_RELATIONS_V2.len(), 93);
        assert!(PRE_ACTIVATION_INCOMPATIBLE_RELATIONS_V2
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
        let v2_preparation = include_str!("../migrations/0010_committed_tick_v2_preparation.sql");
        for relation in PRE_ACTIVATION_INCOMPATIBLE_RELATIONS_V2 {
            assert!(
                MIGRATION_0009_SQL.contains(relation) || v2_preparation.contains(relation),
                "pre-activation relation is not a transactional target: {relation}"
            );
        }
    }

    #[test]
    fn production_activation_accepts_only_postgresql_server_major_17() {
        for version_num in [170_000, 170_001, 179_999] {
            assert_eq!(
                validate_postgresql_server_version_num_v2(version_num),
                Ok(())
            );
        }
        for (version_num, observed_major) in [(160_999, 16), (180_000, 18)] {
            assert_eq!(
                validate_postgresql_server_version_num_v2(version_num),
                Err(
                    RustPersistenceActivationErrorV2::PostgreSqlServerMajorMismatch {
                        observed_major,
                    },
                )
            );
        }
        assert_eq!(
            validate_postgresql_server_version_num_v2(-1),
            Err(RustPersistenceActivationErrorV2::database(
                "decode PostgreSQL server version"
            ))
        );
    }

    #[test]
    fn epoch_nine_census_uses_fresh_post_lock_snapshots() {
        let preparation = super::predecessor_activation_transaction_settings_v2(8);
        let destructive_activation = super::predecessor_activation_transaction_settings_v2(9);
        let unrelated_epoch = super::predecessor_activation_transaction_settings_v2(10);

        assert!(preparation.contains("SERIALIZABLE"));
        assert!(!preparation.contains("READ COMMITTED"));
        assert!(destructive_activation.contains("READ COMMITTED"));
        assert!(!destructive_activation.contains("SERIALIZABLE"));
        assert!(unrelated_epoch.contains("SERIALIZABLE"));
        assert!(!unrelated_epoch.contains("READ COMMITTED"));
    }

    #[test]
    fn active_v2_authority_digests_bind_contract_and_reader_migration() {
        let migrations = super::compiled_committed_tick_v2_activation_migrations()
            .expect("V2 activation registry composes");
        let (contract_sha256, reader_contract_sha256) = v2_authority_contract_digests(&migrations);

        assert_eq!(contract_sha256, sha256_of(ACTIVE_V2_CUTOVER_CONTRACT));
        assert_ne!(contract_sha256, *migrations[0].checksum().as_bytes());
        assert_eq!(reader_contract_sha256, *migrations[1].checksum().as_bytes());

        let mut changed_contract = ACTIVE_V2_CUTOVER_CONTRACT.to_vec();
        changed_contract[0] ^= 1;
        assert_ne!(contract_sha256, sha256_of(&changed_contract));
    }

    fn committed_observation_fixture() -> (
        DurableReplayRuntimeV2<HypergraphStore>,
        CommittedTickReceiptV2,
    ) {
        const DEFINES: &[u8] = &[0x53];
        const REFERENCE_BUNDLE_DOMAIN: &[u8] = b"babylon.h3.reference-bundle-composite.v1\0";
        let (_, rules) = split_content(RULE).expect("rule parses");
        let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
        let content = ContentDigest {
            defines_hash: sha256_of(DEFINES),
            rules_hash: rules_hash_of(&forms).expect("rule hashes"),
        };
        let session_id =
            ReplaySessionIdV1::try_from("per304/runtime-observation-api").expect("session id");
        let material_foundation = michigan_dynamic_hex_foundation_v1().expect("foundation decodes");
        let mut reference_manifest = REFERENCE_BUNDLE_DOMAIN.to_vec();
        reference_manifest.extend_from_slice(&material_foundation.base_reference_cohort_digest());
        reference_manifest.extend_from_slice(&material_foundation.r8_section_digest());
        assert_eq!(
            sha256_of(&reference_manifest),
            material_foundation.reference_bundle_digest()
        );
        let reference = RefDigestV1::from_bytes(material_foundation.reference_bundle_digest());
        let mut session = ReplayTickSession::new(
            SCENARIO,
            None,
            RULE,
            HypergraphStore::new(),
            session_id.clone(),
            ReplaySeed::new(304),
            content,
            reference,
            MaterialStateV1::try_new(material_foundation).expect("material state"),
        )
        .expect("session prepares");
        let bundle =
            FoundationContentBundleV1::try_new(SCENARIO, None, RULE, DEFINES, &reference_manifest)
                .expect("content bundle");
        let foundation = CampaignFoundationV1::capture(&session, bundle).expect("foundation");
        let actions = OrderedPracticeActionBatchV1::empty(session_id, 1).expect("actions");
        let report = session
            .advance(&mut CollectingSink::default(), &actions)
            .expect("tick succeeds");
        let (receipt, _) = CommittedTickReceiptV2::from_acknowledged_with_choices(
            CommittedResolveTickV1::try_from(1).expect("positive tick"),
            ReplayCommitDispositionV1::Committed,
            report,
        );
        let prepared = CommittedTickAuthorityLedgerRowV2::compose(
            1,
            super::CommittedTickAuthorityStateV2::Prepared,
            10,
            [0x10; 32],
            [0x11; 32],
            [0x09; 32],
        )
        .expect("prepared row");
        let activation_row = CommittedTickAuthorityLedgerRowV2::compose(
            2,
            super::CommittedTickAuthorityStateV2::Active,
            11,
            [0x10; 32],
            [0x11; 32],
            prepared.row_sha256,
        )
        .expect("active row");
        let runtime = DurableReplayRuntimeV2 {
            config: postgres::Config::new(),
            campaign_id: CampaignId::from_uuid(uuid::Uuid::from_u128(0x304)),
            session,
            foundation,
            activation_row,
            last_committed_tick: Some(receipt.resolve_tick()),
            last_choice_receipts: Vec::new(),
        };
        (runtime, receipt)
    }

    #[test]
    fn prepared_tick_uses_one_identified_report_without_engine_authority() {
        let (_, rules) = split_content(RULE).expect("rule parses");
        let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
        let content = ContentDigest {
            defines_hash: [0x51; 32],
            rules_hash: rules_hash_of(&forms).expect("rule hashes"),
        };
        let session_id = ReplaySessionIdV1::try_from("per281/runtime-prepare").expect("session id");
        let foundation = michigan_dynamic_hex_foundation_v1().expect("foundation decodes");
        let mut session = ReplayTickSession::new(
            SCENARIO,
            None,
            RULE,
            HypergraphStore::new(),
            session_id.clone(),
            ReplaySeed::new(53),
            content,
            RefDigestV1::from_bytes(foundation.reference_bundle_digest()),
            MaterialStateV1::try_new(foundation).expect("material state"),
        )
        .expect("session prepares");
        let actions = OrderedPracticeActionBatchV1::empty(session_id, 1).expect("actions");
        let report = session
            .advance(&mut CollectingSink::default(), &actions)
            .expect("tick succeeds");

        let prepared = prepare_committed_tick_v2(&report).expect("report composes once");
        assert_eq!(prepared.resolve_tick().get(), 1);
        assert_eq!(prepared.tick_content_hash(), report.tick_content_hash());
        assert_eq!(prepared.graph_row_count(), 2);
        assert_eq!(prepared.event_row_count(), 1);
        assert_eq!(prepared.choice_receipt_row_count(), 0);

        let source = include_str!("runtime.rs");
        let production = source
            .split_once("#[cfg(test)]\nmod live_tests")
            .expect("live tests follow the complete production runtime")
            .0;
        assert_eq!(
            production
                .matches("compose_graph_event_choice_semantic_batches_v2(report)")
                .count(),
            1
        );
        let prohibited_prepare = ["prepare", "_rules("].concat();
        let prohibited_run = ["run_prepared", "_replay_tick("].concat();
        assert!(!production.contains(&prohibited_prepare));
        assert!(!production.contains(&prohibited_run));
    }

    #[test]
    fn bounded_receipt_preserves_only_report_aggregates_and_hashes() {
        let (_, rules) = split_content(RULE).expect("rule parses");
        let forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
        let content = ContentDigest {
            defines_hash: [0x52; 32],
            rules_hash: rules_hash_of(&forms).expect("rule hashes"),
        };
        let session_id =
            ReplaySessionIdV1::try_from("per304/runtime-observation").expect("session id");
        let foundation = michigan_dynamic_hex_foundation_v1().expect("foundation decodes");
        let mut session = ReplayTickSession::new(
            SCENARIO,
            None,
            RULE,
            HypergraphStore::new(),
            session_id.clone(),
            ReplaySeed::new(304),
            content,
            RefDigestV1::from_bytes(foundation.reference_bundle_digest()),
            MaterialStateV1::try_new(foundation).expect("material state"),
        )
        .expect("session prepares");
        let actions = OrderedPracticeActionBatchV1::empty(session_id, 1).expect("actions");
        let report = session
            .advance(&mut CollectingSink::default(), &actions)
            .expect("tick succeeds");
        let expected_graph_before = report.report().before;
        let expected_graph_after = report.report().after;
        let expected_prior_stable_graph_digest = report.prior_stable_graph_digest().into_bytes();
        let expected_stable_graph_digest = report.result_stable_graph_digest().into_bytes();
        let expected_world_before = report.report().world_before;
        let expected_world_after = report.report().world_after;
        let expected_event_digest = report.successful_event_batch().source_digest();
        let expected_material_digest = report.material_state_rows().source_digest();
        let expected_tick_content_hash = report.tick_content_hash();

        let (receipt, _) = CommittedTickReceiptV2::from_acknowledged_with_choices(
            CommittedResolveTickV1::try_from(1).expect("positive tick"),
            ReplayCommitDispositionV1::ReconciledAfterAmbiguousCommit,
            report,
        );

        assert_eq!(receipt.resolve_tick().get(), 1);
        assert_eq!(
            receipt.commit_disposition(),
            ReplayCommitDispositionV1::ReconciledAfterAmbiguousCommit
        );
        assert_eq!(receipt.graph_before(), expected_graph_before);
        assert_eq!(receipt.graph_after(), expected_graph_after);
        assert_eq!(
            receipt.prior_stable_graph_digest(),
            expected_prior_stable_graph_digest
        );
        assert_eq!(
            receipt.result_stable_graph_digest(),
            expected_stable_graph_digest
        );
        assert_eq!(receipt.world_before(), expected_world_before);
        assert_eq!(receipt.world_after(), expected_world_after);
        assert_eq!(receipt.considered(), 1);
        assert_eq!(receipt.fired(), 1);
        assert_eq!(
            receipt.per_rule_considered(),
            &[("production/runtime-prepare".to_owned(), 1)]
        );
        assert_eq!(
            receipt.per_rule_fired(),
            &[("production/runtime-prepare".to_owned(), 1)]
        );
        assert_eq!(receipt.event_count(), 1);
        assert_eq!(receipt.event_digest(), expected_event_digest);
        assert_eq!(receipt.audit_receipt_count(), 2);
        assert!(receipt.material_row_count() > 0);
        assert_eq!(receipt.material_row_digest(), expected_material_digest);
        assert_eq!(receipt.tick_content_hash(), expected_tick_content_hash);
    }

    #[test]
    fn committed_observation_is_bound_to_the_current_receipt_tick_and_digest() {
        let (mut runtime, mut receipt) = committed_observation_fixture();

        let current = runtime
            .observe_current_stable_graph_state_v1()
            .expect("current stable graph is observable without mutation");
        let observed = runtime
            .observe_committed_graph_state_v1(&receipt)
            .expect("current acknowledged graph is observable");
        assert_eq!(current, observed);
        assert_eq!(
            observed.digest().into_bytes(),
            receipt.result_stable_graph_digest()
        );

        runtime.last_committed_tick =
            Some(CommittedResolveTickV1::try_from(2).expect("positive committed tail"));
        assert_eq!(
            runtime.observe_committed_graph_state_v1(&receipt),
            Err(
                RustPersistenceRuntimeErrorV2::ObservationNotCurrentCommittedTail {
                    receipt_tick: 1,
                    current_tail: Some(2),
                }
            )
        );

        runtime.last_committed_tick = Some(receipt.resolve_tick());
        receipt.result_stable_graph_digest = [0xff; 32];
        assert_eq!(
            runtime.observe_committed_graph_state_v1(&receipt),
            Err(RustPersistenceRuntimeErrorV2::ObservationGraphDigestMismatch)
        );
    }
}
