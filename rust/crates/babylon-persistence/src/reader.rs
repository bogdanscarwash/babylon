//! Fog-safe read-only Archive reader and explicit reader-role provisioning.
//!
//! `SemanticArchiveReaderV1` is the split read-only counterpart of
//! [`SemanticArchiveStore`](crate::SemanticArchiveStoreV1): it exposes
//! search and projection reads and is structurally incapable of schema
//! installation, knowledge grants, or receipt materialization. Client
//! credentials ship with the client, so fog is enforced by the `PostgreSQL`
//! privilege layer (`babylon_reader` holds `SELECT` on the fog-safe views
//! only) and by the validated local-only connection target, not by client
//! courtesy.
//!
//! The reader role is `NOLOGIN` by design. A deployment provisions one
//! confined `LOGIN` role as a member of `babylon_reader`
//! (`NOSUPERUSER NOCREATEDB NOCREATEROLE`) and points
//! [`READER_DSN_ENV`] at that credential. Because the bounded startup
//! options pin `event_triggers=off`, that login also needs `GRANT SET ON
//! PARAMETER event_triggers` (the parameter is grant-only under the runtime
//! hardening). The handle refuses to operate on connect unless the session's
//! effective privilege census over the restricted relations is exactly the
//! installed reader footprint, including all current immutable revision views. An owner or superuser DSN refuses before any dossier read.

use std::str::FromStr;

use postgres::{Config, NoTls};

use crate::archive::{database, decode, decode_digest, SemanticArchiveError};
use crate::identity::CampaignId;
use crate::postgres_catalog::{
    validate_connection_target, CatalogError, ConnectionTargetRejection, CATALOG_CONNECT_TIMEOUT,
    CATALOG_STARTUP_OPTIONS, CATALOG_TCP_USER_TIMEOUT,
};
use crate::postgres_diagnostic::PostgresDiagnostic;
use crate::SCHEMA_ADVISORY_LOCK_KEY;

/// Environment variable admitting the read-only reader DSN.
pub const READER_DSN_ENV: &str = "BABYLON_READER_DSN";
/// Exact dedicated read-only role identity.
pub const READER_ROLE_NAME: &str = "babylon_reader";
/// Exact fog-safe acknowledged-commit tick-status relation.
pub const COMMITTED_TICK_STATUS_VIEW: &str = "public.v_committed_tick_status_v1";
/// Exact role DDL. `CREATE ROLE` is transactional in `PostgreSQL`, so the
/// installer executes this statement inside the same Serializable transaction
/// as the grants; a failed install leaves no cluster-wide partial
/// state.
pub const READER_ROLE_CREATE_SQL: &str =
    "CREATE ROLE babylon_reader NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS";
/// Canonical whitespace-normalized view definition the installed relation
/// must store. `pg_get_viewdef` reconstructs the pinned `CREATE VIEW` body;
/// both sides are canonicalized (whitespace collapsed, trailing statement
/// separator trimmed) before comparison.
pub const READER_VIEW_CANONICAL_DEF: &str = "SELECT campaign_id, resolve_tick, \
    envelope_layout_version, tick_content_hash, envelope_digest \
    FROM babylon_state.tick_commit";

const READER_ROLE_MARKERS_SQL: &str = "SELECT \
    EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'babylon_reader'), \
    pg_catalog.to_regclass('public.v_committed_tick_status_v1') IS NOT NULL";
const READER_ROLE_ATTRIBUTES_SQL: &str = "SELECT rolsuper, rolcreatedb, rolcreaterole, \
    rolcanlogin, rolreplication, rolbypassrls FROM pg_catalog.pg_roles WHERE rolname = 'babylon_reader'";
/// Effective-privilege census over the restricted relations: relation-level
/// and column-level ACL entries (`aclexplode`), ownership, and everything
/// inherited through `pg_auth_members` role-membership recursion, including
/// grants to `PUBLIC` (role oid `0`). Entries read `schema.relation:privilege`
/// with a ` (grantable)` suffix when the grant carries `WITH GRANT OPTION`.
pub(crate) const READER_PRIVILEGE_CENSUS_SQL: &str = "WITH RECURSIVE role_closure(oid) AS (\
    SELECT 0::pg_catalog.oid \
    UNION \
    SELECT pg_roles.oid FROM pg_catalog.pg_roles WHERE pg_roles.rolname = $1 \
    UNION \
    SELECT membership.roleid FROM pg_catalog.pg_auth_members membership \
    JOIN role_closure ON role_closure.oid = membership.member), \
    restricted AS (\
    SELECT relation.oid, namespace.nspname, relation.relname, relation.relacl, relation.relowner \
    FROM pg_catalog.pg_class relation \
    JOIN pg_catalog.pg_namespace namespace ON namespace.oid = relation.relnamespace \
    WHERE (namespace.nspname = 'babylon_state' AND relation.relkind IN ('r', 'p', 'v', 'm', 'f')) \
    OR (namespace.nspname = 'babylon_meta' AND relation.relkind IN ('r', 'p', 'v', 'm', 'f')) \
    OR (namespace.nspname = 'public' AND relation.relname IN ('v_committed_tick_status_v1', \
    'v_archive_verification_v1', \
    'v_observer_economy_foundation_v1', 'v_known_county_economy_v1', \
    'v_observer_county_economy_v1', 'v_material_campaign_identity_v1', \
    'v_observer_material_state_v1','v_archive_revision_known_v2','v_archive_revision_index_v2', \
    'v_archive_revision_atom_v2','v_archive_revision_grant_v2',\
    'v_archive_subject_grant_v2','v_archive_tick_knowledge_v2','v_archive_revision_scope_v2', \
    'v_observer_graph_node_v1', 'v_observer_graph_node_f64_v1', \
    'v_observer_graph_edge_v1', 'v_observer_graph_hyperedge_v1', \
    'v_observer_graph_hyperedge_member_v1', 'v_observer_graph_edge_f64_v1', \
    'v_observer_graph_node_currency_v1', 'v_observer_graph_hyperedge_f64_v1', \
    'v_observer_world_register_v1', 'v_observer_hex_state_delta_v1', \
    'v_observer_territory_state_v1', 'v_observer_territory_state_field_v1', \
    'v_observer_organization_state_v1', 'v_observer_organization_state_field_v1', \
    'v_observer_organization_territory_v1', 'v_observer_tick_event_v2', \
    'v_observer_tick_event_field_v2', 'v_observer_tick_choice_receipt_v1', \
    'v_observer_tick_choice_receipt_branch_v1', 'v_observer_tick_choice_receipt_carrier_element_v1', \
    'v_observer_checkpoint_manifest', 'v_observer_checkpoint_section_v1', \
    'v_observer_archive_dirty_receipt_v1', 'v_observer_tick_action_batch_v1'))), \
    held AS (\
    SELECT restricted.nspname || '.' || restricted.relname || ':' || acl.privilege_type || \
    CASE WHEN acl.is_grantable THEN ' (grantable)' ELSE '' END AS entry \
    FROM restricted \
    CROSS JOIN LATERAL pg_catalog.aclexplode(restricted.relacl) acl \
    JOIN role_closure ON role_closure.oid = acl.grantee \
    UNION \
    SELECT restricted.nspname || '.' || restricted.relname || ':OWNERSHIP' \
    FROM restricted \
    JOIN pg_catalog.pg_roles owner_role ON owner_role.oid = restricted.relowner \
    WHERE owner_role.rolname = $1 \
    UNION \
    SELECT restricted.nspname || '.' || restricted.relname || ':' || acl.privilege_type || \
    CASE WHEN acl.is_grantable THEN ' (grantable)' ELSE '' END \
    FROM restricted \
    CROSS JOIN LATERAL (\
    SELECT attribute.attacl FROM pg_catalog.pg_attribute attribute \
    WHERE attribute.attrelid = restricted.oid AND attribute.attnum > 0 \
    AND NOT attribute.attisdropped \
    ) attributes \
    CROSS JOIN LATERAL pg_catalog.aclexplode(attributes.attacl) acl \
    JOIN role_closure ON role_closure.oid = acl.grantee) \
    SELECT entry FROM held ORDER BY entry";
const READER_VIEW_IDENTITY_SQL: &str = "SELECT relation.relkind::pg_catalog.text, \
    pg_catalog.pg_get_viewdef(relation.oid) \
    FROM pg_catalog.pg_class relation \
    JOIN pg_catalog.pg_namespace namespace ON namespace.oid = relation.relnamespace \
    WHERE namespace.nspname = 'public' AND relation.relname = 'v_committed_tick_status_v1'";
const READER_SESSION_AUTHORITY_SQL: &str = "SELECT current_user::pg_catalog.text, \
    (SELECT pg_roles.rolsuper FROM pg_catalog.pg_roles WHERE pg_roles.rolname = current_user)";
/// Exact fog-safe projection grants on the one current schema.
pub(crate) const READER_VIEWS: [&str; 12] = [
    "public.v_archive_revision_atom_v2",
    "public.v_archive_revision_grant_v2",
    "public.v_archive_revision_index_v2",
    "public.v_archive_revision_known_v2",
    "public.v_archive_revision_scope_v2",
    "public.v_archive_subject_grant_v2",
    "public.v_archive_tick_knowledge_v2",
    "public.v_archive_verification_v1",
    "public.v_committed_tick_status_v1",
    "public.v_known_county_economy_v1",
    "public.v_material_campaign_identity_v1",
    "public.v_observer_economy_foundation_v1",
];
/// Known-acknowledged-commit tick status read. The read goes through the view
/// only; `babylon_state.tick_commit` stays revoked from the reader role.
pub const COMMITTED_TICK_STATUS_SQL: &str = "SELECT campaign_id, resolve_tick, \
    envelope_layout_version, tick_content_hash, envelope_digest \
    FROM public.v_committed_tick_status_v1 \
    WHERE campaign_id = $1::uuid \
    ORDER BY resolve_tick DESC LIMIT 1";

/// Fog-safe receipt-processing status without page or raw-ledger access.
pub const ARCHIVE_VERIFICATION_STATUS_SQL: &str =
    "SELECT durable_tick, processed_tick FROM public.v_archive_verification_v1 \
     WHERE campaign_id = $1::uuid";

/// Campaign-wide processing progress, separate from a page's content source tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArchiveVerificationStatus {
    durable_tick: u64,
    processed_tick: u64,
}

impl ArchiveVerificationStatus {
    /// Highest acknowledged commit.
    #[must_use]
    pub const fn durable_tick(&self) -> u64 {
        self.durable_tick
    }

    /// Contiguous prefix whose Archive receipts have all settled.
    #[must_use]
    pub const fn processed_tick(&self) -> u64 {
        self.processed_tick
    }
}

/// One acknowledged-commit tail row observed through the fog-safe view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommittedTickStatus {
    campaign_id: CampaignId,
    resolve_tick: u64,
    envelope_layout_version: i16,
    tick_content_hash: [u8; 32],
    envelope_digest: [u8; 32],
}

impl CommittedTickStatus {
    /// Borrow the committed campaign identity.
    #[must_use]
    pub const fn campaign_id(&self) -> &CampaignId {
        &self.campaign_id
    }

    /// Return the acknowledged durable resolve tick.
    #[must_use]
    pub const fn resolve_tick(&self) -> u64 {
        self.resolve_tick
    }

    /// Return the committed envelope layout version.
    #[must_use]
    pub const fn envelope_layout_version(&self) -> i16 {
        self.envelope_layout_version
    }

    /// Return the exact committed tick content hash.
    #[must_use]
    pub const fn tick_content_hash(&self) -> &[u8; 32] {
        &self.tick_content_hash
    }

    /// Return the exact committed envelope digest.
    #[must_use]
    pub const fn envelope_digest(&self) -> &[u8; 32] {
        &self.envelope_digest
    }
}

/// Idempotent reader-role install result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReaderRoleDisposition {
    /// The role, view, or grants committed now.
    Installed,
    /// The exact role attributes, view, and view grant already existed.
    AlreadyCurrent,
}

/// Stable closed refusal taxonomy for reader construction and installation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticArchiveReaderError {
    CurrentSchema(crate::CurrentSchemaError),
    /// The reader DSN environment variable is unset.
    MissingEnv(&'static str),
    /// The reader DSN environment variable is not valid UTF-8.
    EnvNotUtf8(&'static str),
    /// The reader DSN did not parse as one `PostgreSQL` configuration.
    InvalidDsn,
    /// The parsed target violated the local-only connection contract.
    ConnectionTarget(ConnectionTargetRejection),
    /// An existing `babylon_reader` role does not have the exact locked attributes.
    RoleMismatch,
    /// The view exists without the exact pinned identity (plain-view relkind
    /// and canonical definition) or is absent when required.
    ViewMismatch,
    /// The effective-privilege census over the restricted relations diverges
    /// from the exact reader footprint; the entries carry the observed drift.
    PrivilegeDrift(Vec<String>),
    /// The connected session carries authority beyond the reader footprint
    /// (superuser, ownership, or extra effective privileges); the entries
    /// carry the observed census.
    WriterAuthorityRefused(Vec<String>),
    /// One read crossed the store boundary and failed there.
    Archive(SemanticArchiveError),
    /// The advisory lock did not release from this session.
    LockMismatch,
    /// One database operation failed with a bounded secret-safe driver diagnostic.
    Database {
        /// Stable operation identity.
        operation: &'static str,
        /// Secret-safe `PostgreSQL` classification, SQLSTATE, and message.
        diagnostic: PostgresDiagnostic,
    },
}

impl std::fmt::Display for SemanticArchiveReaderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "semantic Archive reader refusal: {self:?}")
    }
}

impl std::error::Error for SemanticArchiveReaderError {}

fn database_error(operation: &'static str, error: &postgres::Error) -> SemanticArchiveReaderError {
    SemanticArchiveReaderError::Database {
        operation,
        diagnostic: PostgresDiagnostic::capture(error),
    }
}

fn archive_boundary(error: SemanticArchiveError) -> SemanticArchiveReaderError {
    SemanticArchiveReaderError::Archive(error)
}

/// Collapse whitespace and trim the trailing statement separator so a
/// `pg_get_viewdef` reconstruction compares against the pinned canonical text.
fn canonicalize_view_definition(definition: &str) -> String {
    definition
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_end_matches(';')
        .trim()
        .to_owned()
}

/// Census one role's effective privileges (including inherited, column-level,
/// `PUBLIC`, and ownership entries) over the restricted relations.
pub(crate) fn census_role_privileges(
    client: &mut impl postgres::GenericClient,
    role: &str,
    operation: &'static str,
) -> Result<Vec<String>, SemanticArchiveReaderError> {
    let rows = client
        .query(READER_PRIVILEGE_CENSUS_SQL, &[&role])
        .map_err(|error| database_error(operation, &error))?;
    rows.iter()
        .map(|row| row.try_get::<_, String>(0))
        .collect::<Result<_, _>>()
        .map_err(|error| database_error(operation, &error))
}

fn exact_reader_footprint(held: &[String]) -> bool {
    let expected = READER_VIEWS
        .iter()
        .map(|view| format!("{view}:SELECT"))
        .collect::<Vec<_>>();
    held == expected
}

fn connection_target_error(error: &CatalogError) -> SemanticArchiveReaderError {
    // The validator is a pure target check: its only failure construction is
    // one bounded target rejection, so any other variant is an internal fault.
    let CatalogError::UnsupportedConnectionTarget { reason } = error else {
        unreachable!("connection target validation only reports target rejections")
    };
    SemanticArchiveReaderError::ConnectionTarget(*reason)
}

/// Split read-only `PostgreSQL` handle for fog-safe Archive reads.
///
/// Writer operations (`install_schema`, `grant_knowledge`,
/// `materialize_receipt`, worker sweeps) are unrepresentable on this type;
/// [`SemanticArchiveStore`](crate::SemanticArchiveStoreV1) and the runtime
/// binary remain the sole writers.
#[derive(Clone)]
pub struct SemanticArchiveReader {
    config: Config,
}

impl SemanticArchiveReader {
    /// Parse and validate one explicit local-only reader DSN.
    ///
    /// The raw parsed [`Config`] is validated before any bounded startup
    /// options are added, so caller-supplied `options`, host-address
    /// overrides, multi-host targets, and non-loopback hosts refuse before a
    /// socket opens.
    ///
    /// # Errors
    /// Returns [`SemanticArchiveReaderError`] for a malformed DSN or an
    /// out-of-contract connection target.
    pub fn from_dsn(raw: &str) -> Result<Self, SemanticArchiveReaderError> {
        let config = Config::from_str(raw).map_err(|_| SemanticArchiveReaderError::InvalidDsn)?;
        Self::new(&config)
    }

    /// Admit the reader DSN from [`READER_DSN_ENV`].
    ///
    /// # Errors
    /// Returns [`SemanticArchiveReaderError`] for a missing or non-UTF-8
    /// environment value, a malformed DSN, or an out-of-contract target.
    pub fn from_env() -> Result<Self, SemanticArchiveReaderError> {
        let raw = std::env::var_os(READER_DSN_ENV)
            .ok_or(SemanticArchiveReaderError::MissingEnv(READER_DSN_ENV))?;
        let dsn = raw
            .into_string()
            .map_err(|_| SemanticArchiveReaderError::EnvNotUtf8(READER_DSN_ENV))?;
        Self::from_dsn(&dsn)
    }

    /// Validate one explicit local-only connection target and bind the reader.
    ///
    /// # Errors
    /// Returns [`SemanticArchiveReaderError::ConnectionTarget`] for
    /// caller-supplied startup options, host-address overrides, multi-host or
    /// multi-port targets, a missing host, or a non-loopback TCP target.
    pub fn new(config: &Config) -> Result<Self, SemanticArchiveReaderError> {
        validate_connection_target(config).map_err(|error| connection_target_error(&error))?;
        Ok(Self {
            config: config.clone(),
        })
    }

    /// Read the acknowledged-commit tick status through the fog-safe view.
    ///
    /// The view projects `babylon_state.tick_commit`, which stays revoked
    /// from the reader role; `tick_commit`, not `MAX(tick)`, marks durability.
    ///
    /// # Errors
    /// Refuses a malformed stored row, writer authority on the session, or a
    /// database failure.
    pub fn committed_tick_status(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Option<CommittedTickStatus>, SemanticArchiveReaderError> {
        let mut client = self.connect("connect committed tick status reader")?;
        client
            .query_opt(COMMITTED_TICK_STATUS_SQL, &[campaign_id.as_uuid()])
            .map_err(|error| archive_boundary(database("read committed tick status view", &error)))?
            .map(|row| decode_committed_tick_status(campaign_id, &row))
            .transpose()
            .map_err(archive_boundary)
    }

    /// Read durable and contiguously processed ticks in one database snapshot.
    /// A quiet settled tick advances this status without changing any page,
    /// atom, content hash, or content-source tick.
    ///
    /// # Errors
    /// Refuses writer authority, invalid stored horizons, or a database failure.
    pub fn archive_verification_status(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Option<ArchiveVerificationStatus>, SemanticArchiveReaderError> {
        let mut client = self.connect("connect Archive verification reader")?;
        let row = client
            .query_opt(ARCHIVE_VERIFICATION_STATUS_SQL, &[campaign_id.as_uuid()])
            .map_err(|error| database_error("read Archive verification status", &error))?;
        row.map(|row| {
            let durable_tick = u64::try_from(decode::<i64>(&row, 0)?)
                .map_err(|_| SemanticArchiveError::StoredPageMismatch)?;
            let processed_tick = u64::try_from(decode::<i64>(&row, 1)?)
                .map_err(|_| SemanticArchiveError::StoredPageMismatch)?;
            if processed_tick > durable_tick {
                return Err(SemanticArchiveError::StoredPageMismatch);
            }
            Ok(ArchiveVerificationStatus {
                durable_tick,
                processed_tick,
            })
        })
        .transpose()
        .map_err(archive_boundary)
    }

    pub(crate) fn connect(
        &self,
        operation: &'static str,
    ) -> Result<postgres::Client, SemanticArchiveReaderError> {
        // The stored config stays raw: validation must observe the caller's
        // exact target, not the bounded startup options added here.
        let mut bounded = self.config.clone();
        bounded
            .connect_timeout(CATALOG_CONNECT_TIMEOUT)
            .tcp_user_timeout(CATALOG_TCP_USER_TIMEOUT)
            .options(CATALOG_STARTUP_OPTIONS);
        let mut client = bounded
            .connect(NoTls)
            .map_err(|error| database_error(operation, &error))?;
        confine_reader_authority(&mut client)?;
        Ok(client)
    }
}

/// Refuse the connection unless the session carries exactly the reader
/// footprint. `default_transaction_read_only` is user-changeable, so privilege
/// confinement is re-censused here on every connect: a superuser session, an
/// owner credential, or any inherited extra privilege is a loud refusal.
fn confine_reader_authority(
    client: &mut postgres::Client,
) -> Result<(), SemanticArchiveReaderError> {
    let row = client
        .query_one(READER_SESSION_AUTHORITY_SQL, &[])
        .map_err(|error| database_error("census reader session authority", &error))?;
    let session_role: String = row
        .try_get(0)
        .map_err(|error| database_error("decode reader session role", &error))?;
    let is_superuser: bool = row
        .try_get(1)
        .map_err(|error| database_error("decode reader session superuser attribute", &error))?;
    let mut held = Vec::new();
    if is_superuser {
        held.push(format!("{session_role}:SUPERUSER"));
    }
    held.extend(census_role_privileges(
        client,
        &session_role,
        "census reader session privileges",
    )?);
    if exact_reader_footprint(&held) {
        Ok(())
    } else {
        Err(SemanticArchiveReaderError::WriterAuthorityRefused(held))
    }
}

/// Provision exact current fog-safe grants for the confined reader group.
///
/// # Errors
/// Refuses incompatible schemas, role attributes and any partial or extra privileges.
pub fn install_reader_role(
    config: &Config,
) -> Result<ReaderRoleDisposition, SemanticArchiveReaderError> {
    validate_connection_target(config).map_err(|error| connection_target_error(&error))?;
    let mut client = crate::current_schema::bounded_config(config)
        .connect(NoTls)
        .map_err(|error| database_error("connect reader role installer", &error))?;
    client
        .query_one(
            "SELECT pg_catalog.pg_advisory_lock($1)",
            &[&SCHEMA_ADVISORY_LOCK_KEY],
        )
        .map_err(|error| database_error("lock reader role installer", &error))?;
    let result = install_reader_role_locked(&mut client);
    let unlock = client
        .query_one(
            "SELECT pg_catalog.pg_advisory_unlock($1)",
            &[&SCHEMA_ADVISORY_LOCK_KEY],
        )
        .and_then(|row| row.try_get::<_, bool>(0))
        .map_err(|error| database_error("unlock reader role installer", &error));
    match (result, unlock) {
        (Err(error), _) | (Ok(_), Err(error)) => Err(error),
        (Ok(disposition), Ok(true)) => Ok(disposition),
        (Ok(_), Ok(false)) => Err(SemanticArchiveReaderError::LockMismatch),
    }
}

fn install_reader_role_locked(
    client: &mut postgres::Client,
) -> Result<ReaderRoleDisposition, SemanticArchiveReaderError> {
    let mut tx = client
        .build_transaction()
        .isolation_level(postgres::IsolationLevel::Serializable)
        .read_only(false)
        .start()
        .map_err(|error| database_error("begin reader role provisioning", &error))?;
    crate::current_schema::require_current_schema(&mut tx)
        .map_err(SemanticArchiveReaderError::CurrentSchema)?;
    let row = tx
        .query_one(READER_ROLE_MARKERS_SQL, &[])
        .map_err(|error| database_error("inspect reader role", &error))?;
    let role_exists: bool = row
        .try_get(0)
        .map_err(|error| database_error("decode reader role", &error))?;
    if role_exists {
        verify_reader_role_attributes(&mut tx)?;
    }
    verify_reader_view_identity(&mut tx)?;
    let held = census_role_privileges(&mut tx, READER_ROLE_NAME, "census reader role privileges")?;
    let disposition = if exact_reader_footprint(&held) {
        ReaderRoleDisposition::AlreadyCurrent
    } else if held.is_empty() {
        if !role_exists {
            tx.batch_execute(READER_ROLE_CREATE_SQL)
                .map_err(|error| database_error("create reader role", &error))?;
        }
        let grants = format!(
            "GRANT SELECT ON {} TO babylon_reader",
            READER_VIEWS.join(", ")
        );
        tx.batch_execute(&grants)
            .map_err(|error| database_error("grant reader projections", &error))?;
        ReaderRoleDisposition::Installed
    } else {
        return Err(SemanticArchiveReaderError::PrivilegeDrift(held));
    };
    crate::current_schema::require_current_schema(&mut tx)
        .map_err(SemanticArchiveReaderError::CurrentSchema)?;
    let held = census_role_privileges(&mut tx, READER_ROLE_NAME, "verify reader role privileges")?;
    if !exact_reader_footprint(&held) {
        return Err(SemanticArchiveReaderError::PrivilegeDrift(held));
    }
    tx.commit()
        .map_err(|error| database_error("commit reader provisioning", &error))?;
    Ok(disposition)
}

fn verify_reader_role_attributes(
    client: &mut impl postgres::GenericClient,
) -> Result<(), SemanticArchiveReaderError> {
    let row = client
        .query_one(READER_ROLE_ATTRIBUTES_SQL, &[])
        .map_err(|error| database_error("inspect reader role attributes", &error))?;
    for index in 0..6 {
        if row
            .try_get::<_, bool>(index)
            .map_err(|error| database_error("decode reader role attributes", &error))?
        {
            return Err(SemanticArchiveReaderError::RoleMismatch);
        }
    }
    Ok(())
}

fn verify_reader_view_identity(
    client: &mut impl postgres::GenericClient,
) -> Result<(), SemanticArchiveReaderError> {
    let row = client
        .query_opt(READER_VIEW_IDENTITY_SQL, &[])
        .map_err(|error| database_error("inspect reader view identity", &error))?
        .ok_or(SemanticArchiveReaderError::ViewMismatch)?;
    let relkind: String = row
        .try_get(0)
        .map_err(|error| database_error("decode reader view relkind", &error))?;
    let definition: Option<String> = row
        .try_get(1)
        .map_err(|error| database_error("decode reader view definition", &error))?;
    if relkind == "v"
        && definition
            .as_deref()
            .map(canonicalize_view_definition)
            .as_deref()
            == Some(READER_VIEW_CANONICAL_DEF)
    {
        Ok(())
    } else {
        Err(SemanticArchiveReaderError::ViewMismatch)
    }
}

fn decode_committed_tick_status(
    campaign_id: CampaignId,
    row: &postgres::Row,
) -> Result<CommittedTickStatus, SemanticArchiveError> {
    let stored_campaign: uuid::Uuid = decode(row, 0)?;
    if stored_campaign != *campaign_id.as_uuid() {
        return Err(SemanticArchiveError::StoredPageMismatch);
    }
    let resolve_tick = u64::try_from(decode::<i64>(row, 1)?)
        .ok()
        .filter(|tick| *tick > 0)
        .ok_or(SemanticArchiveError::StoredPageMismatch)?;
    let envelope_layout_version: i16 = decode(row, 2)?;
    Ok(CommittedTickStatus {
        campaign_id,
        resolve_tick,
        envelope_layout_version,
        tick_content_hash: decode_digest(row, 3)?,
        envelope_digest: decode_digest(row, 4)?,
    })
}
