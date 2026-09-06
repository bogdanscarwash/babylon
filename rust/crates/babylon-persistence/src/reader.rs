//! Fog-safe read-only Archive reader handle and the additive reader-role installer (ADR249 R8).
//!
//! `SemanticArchiveReaderV1` is the split read-only counterpart of
//! [`SemanticArchiveStoreV1`](crate::SemanticArchiveStoreV1): it exposes
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
//! [`READER_DSN_ENV_V1`] at that credential. Because the bounded startup
//! options pin `event_triggers=off`, that login also needs `GRANT SET ON
//! PARAMETER event_triggers` (the parameter is grant-only under the runtime
//! hardening). The handle refuses to operate on connect unless the session's
//! effective privilege census over the restricted relations is exactly the
//! installed reader footprint, including the immutable revision views when
//! present. An owner or superuser DSN refuses before any dossier read.

use std::str::FromStr;

use postgres::{Config, NoTls};

use crate::archive::{database, decode, decode_digest, SemanticArchiveErrorV1};
use crate::identity::CampaignId;
use crate::legacy_adopter::{
    validate_legacy_connection_target, LegacyAdopterError, LegacyConnectionTargetRejection,
    LEGACY_ADOPTER_CONNECT_TIMEOUT, LEGACY_ADOPTER_STARTUP_OPTIONS,
    LEGACY_ADOPTER_TCP_USER_TIMEOUT,
};
use crate::migration_manifest::SCHEMA_ADVISORY_LOCK_KEY;
use crate::postgres_diagnostic::PostgresDiagnosticV1;

/// Environment variable admitting the read-only reader DSN.
pub const READER_DSN_ENV_V1: &str = "BABYLON_READER_DSN";
/// Exact dedicated read-only role identity.
pub const READER_ROLE_NAME_V1: &str = "babylon_reader";
/// Exact fog-safe acknowledged-commit tick-status relation.
pub const COMMITTED_TICK_STATUS_VIEW_V1: &str = "public.v_committed_tick_status_v1";
/// Exact role DDL. `CREATE ROLE` is transactional in `PostgreSQL`, so the
/// installer executes this statement inside the same Serializable transaction
/// as the view and grants; a failed install leaves no cluster-wide partial
/// state.
pub const READER_ROLE_CREATE_SQL_V1: &str =
    "CREATE ROLE babylon_reader NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE";
/// Transactional additive schema: the tick-status view, its exact SELECT
/// grant, and the guarded archive-table revokes.
pub const READER_ROLE_SCHEMA_V1_SQL: &str = include_str!("../migrations/reader_role_v1.sql");
/// Canonical whitespace-normalized view definition the installed relation
/// must store. `pg_get_viewdef` reconstructs the pinned `CREATE VIEW` body;
/// both sides are canonicalized (whitespace collapsed, trailing statement
/// separator trimmed) before comparison.
pub const READER_VIEW_CANONICAL_DEF_V1: &str = "SELECT campaign_id, resolve_tick, \
    envelope_layout_version, tick_content_hash, envelope_digest \
    FROM babylon_state.tick_commit";

const READER_ROLE_MARKERS_SQL_V1: &str = "SELECT \
    EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'babylon_reader'), \
    pg_catalog.to_regclass('public.v_committed_tick_status_v1') IS NOT NULL";
const READER_ROLE_ATTRIBUTES_SQL_V1: &str = "SELECT rolsuper, rolcreatedb, rolcreaterole, \
    rolcanlogin FROM pg_catalog.pg_roles WHERE rolname = 'babylon_reader'";
/// Effective-privilege census over the restricted relations: relation-level
/// and column-level ACL entries (`aclexplode`), ownership, and everything
/// inherited through `pg_auth_members` role-membership recursion, including
/// grants to `PUBLIC` (role oid `0`). Entries read `schema.relation:privilege`
/// with a ` (grantable)` suffix when the grant carries `WITH GRANT OPTION`.
pub(crate) const READER_PRIVILEGE_CENSUS_SQL_V1: &str = "WITH RECURSIVE role_closure(oid) AS (\
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
    OR (namespace.nspname = 'babylon_meta' AND relation.relname IN ('archive_page_v1', \
    'archive_knowledge_grant_v1', 'archive_receipt_consumption_v1', 'archive_atom_v1', \
    'archive_page_atom_v1','archive_page_retired_v1','archive_page_atom_retired_v1', \
    'archive_revision_schema_v2','archive_retention_v2','archive_page_revision_v2', \
    'archive_revision_atom_v2','archive_revision_grant_v2','archive_retention_seal_v2', \
    'archive_tick_knowledge_v2','archive_tick_knowledge_member_v2','archive_wakeup_schema_v1')) \
    OR (namespace.nspname = 'public' AND relation.relname IN ('v_committed_tick_status_v1', \
    'v_archive_page_known_v1', 'v_archive_atom_visible', 'v_county_card_atoms', \
    'v_archive_subject_atoms', 'v_archive_verification_v1', \
    'v_observer_economy_foundation_v1', 'v_known_county_economy_v1', \
    'v_observer_county_economy_v1', 'v_material_campaign_identity_v1', \
    'v_observer_material_state_v1','v_archive_revision_known_v2','v_archive_revision_index_v2', \
    'v_archive_revision_atom_v2','v_archive_revision_grant_v2','v_archive_retention_v2', \
    'v_archive_subject_grant_v2','v_archive_tick_knowledge_v2','v_archive_revision_scope_v2'))), \
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
const READER_VIEW_IDENTITY_SQL_V1: &str = "SELECT relation.relkind::pg_catalog.text, \
    pg_catalog.pg_get_viewdef(relation.oid) \
    FROM pg_catalog.pg_class relation \
    JOIN pg_catalog.pg_namespace namespace ON namespace.oid = relation.relnamespace \
    WHERE namespace.nspname = 'public' AND relation.relname = 'v_committed_tick_status_v1'";
const READER_SESSION_AUTHORITY_SQL_V1: &str = "SELECT current_user::pg_catalog.text, \
    (SELECT pg_roles.rolsuper FROM pg_catalog.pg_roles WHERE pg_roles.rolname = current_user)";
/// The pre-atom reader footprint: the one privilege entry constituting the
/// exact reader footprint before `migrations/archive_atom_v1.sql` installs.
/// Every census entry outside this set is drift or writer authority.
const READER_FOOTPRINT_V1: [&str; 1] = ["public.v_committed_tick_status_v1:SELECT"];
/// The post-atom reader footprint: exactly `SELECT` on the five fog-safe
/// views, sorted because the census emits entries `ORDER BY entry`. Entries
/// beyond this set — any base-table grant, ownership, or a grantable entry —
/// are drift or writer authority.
const READER_FOOTPRINT_WITH_ATOMS_V1: [&str; 5] = [
    "public.v_archive_atom_visible:SELECT",
    "public.v_archive_page_known_v1:SELECT",
    "public.v_archive_subject_atoms:SELECT",
    "public.v_committed_tick_status_v1:SELECT",
    "public.v_county_card_atoms:SELECT",
];
/// Additive-schema marker probe: each installed capability contributes only
/// its exact fog-safe grants to the expected reader footprint. The probe reads `pg_catalog` directly instead
/// of resolving the schema-qualified name: `pg_catalog.to_regclass` needs
/// `USAGE` on `babylon_meta`, which the confined reader role must never hold,
/// and every role may read the catalog.
const READER_SCHEMA_MARKERS_SQL_V1: &str = "SELECT \
    EXISTS (SELECT 1 FROM pg_catalog.pg_class AS relation \
        JOIN pg_catalog.pg_namespace AS namespace \
          ON namespace.oid = relation.relnamespace \
        WHERE namespace.nspname = 'babylon_meta' \
          AND relation.relname = 'archive_atom_schema_v1' \
          AND relation.relkind = 'r'), \
    EXISTS (SELECT 1 FROM pg_catalog.pg_class AS relation \
        JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace \
        WHERE namespace.nspname = 'public' AND relation.relname = 'v_archive_verification_v1' \
          AND relation.relkind = 'v'), \
    EXISTS (SELECT 1 FROM pg_catalog.pg_class AS relation \
        JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace \
        WHERE namespace.nspname = 'public' AND relation.relname = 'v_known_county_economy_v1' \
          AND relation.relkind = 'v'), \
    EXISTS (SELECT 1 FROM pg_catalog.pg_class AS relation \
        JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace \
        WHERE namespace.nspname = 'public' AND relation.relname = 'v_material_campaign_identity_v1' \
          AND relation.relkind = 'v'), \
    EXISTS (SELECT 1 FROM pg_catalog.pg_class relation \
        JOIN pg_catalog.pg_namespace namespace ON namespace.oid=relation.relnamespace \
        WHERE namespace.nspname='babylon_meta' AND relation.relname='archive_revision_schema_v2' \
        AND relation.relkind='r')";
/// Known-acknowledged-commit tick status read. The read goes through the view
/// only; `babylon_state.tick_commit` stays revoked from the reader role.
pub const COMMITTED_TICK_STATUS_SQL_V1: &str = "SELECT campaign_id, resolve_tick, \
    envelope_layout_version, tick_content_hash, envelope_digest \
    FROM public.v_committed_tick_status_v1 \
    WHERE campaign_id = $1::uuid \
    ORDER BY resolve_tick DESC LIMIT 1";

/// Fog-safe receipt-processing status without page or raw-ledger access.
pub const ARCHIVE_VERIFICATION_STATUS_SQL_V1: &str =
    "SELECT durable_tick, processed_tick FROM public.v_archive_verification_v1 \
     WHERE campaign_id = $1::uuid";

/// Campaign-wide processing progress, separate from a page's content source tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArchiveVerificationStatusV1 {
    durable_tick: u64,
    processed_tick: u64,
}

impl ArchiveVerificationStatusV1 {
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
pub struct CommittedTickStatusV1 {
    campaign_id: CampaignId,
    resolve_tick: u64,
    envelope_layout_version: i16,
    tick_content_hash: [u8; 32],
    envelope_digest: [u8; 32],
}

impl CommittedTickStatusV1 {
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
pub enum ReaderRoleDispositionV1 {
    /// The role, view, or grants committed now.
    Installed,
    /// The exact role attributes, view, and view grant already existed.
    AlreadyCurrent,
}

/// Stable closed refusal taxonomy for reader construction and installation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticArchiveReaderErrorV1 {
    /// The reader DSN environment variable is unset.
    MissingEnv(&'static str),
    /// The reader DSN environment variable is not valid UTF-8.
    EnvNotUtf8(&'static str),
    /// The reader DSN did not parse as one `PostgreSQL` configuration.
    InvalidDsn,
    /// The parsed target violated the local-only connection contract.
    ConnectionTarget(LegacyConnectionTargetRejection),
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
    Archive(SemanticArchiveErrorV1),
    /// The advisory lock did not release from this session.
    LockMismatch,
    /// One database operation failed with a bounded secret-safe driver diagnostic.
    Database {
        /// Stable operation identity.
        operation: &'static str,
        /// Secret-safe `PostgreSQL` classification, SQLSTATE, and message.
        diagnostic: PostgresDiagnosticV1,
    },
}

impl std::fmt::Display for SemanticArchiveReaderErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "semantic Archive reader refusal: {self:?}")
    }
}

impl std::error::Error for SemanticArchiveReaderErrorV1 {}

fn database_error(
    operation: &'static str,
    error: &postgres::Error,
) -> SemanticArchiveReaderErrorV1 {
    SemanticArchiveReaderErrorV1::Database {
        operation,
        diagnostic: PostgresDiagnosticV1::capture(error),
    }
}

fn archive_boundary(error: SemanticArchiveErrorV1) -> SemanticArchiveReaderErrorV1 {
    SemanticArchiveReaderErrorV1::Archive(error)
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
fn census_role_privileges(
    client: &mut postgres::Client,
    role: &str,
    operation: &'static str,
) -> Result<Vec<String>, SemanticArchiveReaderErrorV1> {
    let rows = client
        .query(READER_PRIVILEGE_CENSUS_SQL_V1, &[&role])
        .map_err(|error| database_error(operation, &error))?;
    rows.iter()
        .map(|row| row.try_get::<_, String>(0))
        .collect::<Result<_, _>>()
        .map_err(|error| database_error(operation, &error))
}

fn exact_reader_footprint(held: &[String], markers: (bool, bool, bool, bool, bool)) -> bool {
    let (atoms, verification, economy, material, revisions) = markers;
    let mut expected = if revisions {
        vec![
            "public.v_committed_tick_status_v1:SELECT",
            "public.v_archive_revision_known_v2:SELECT",
            "public.v_archive_revision_index_v2:SELECT",
            "public.v_archive_revision_atom_v2:SELECT",
            "public.v_archive_revision_grant_v2:SELECT",
            "public.v_archive_retention_v2:SELECT",
            "public.v_archive_subject_grant_v2:SELECT",
            "public.v_archive_tick_knowledge_v2:SELECT",
            "public.v_archive_revision_scope_v2:SELECT",
        ]
    } else if atoms {
        READER_FOOTPRINT_WITH_ATOMS_V1.to_vec()
    } else {
        READER_FOOTPRINT_V1.to_vec()
    };
    if verification {
        expected.push("public.v_archive_verification_v1:SELECT");
    }
    if economy {
        expected.push("public.v_known_county_economy_v1:SELECT");
        expected.push("public.v_observer_economy_foundation_v1:SELECT");
    }
    if material {
        expected.push("public.v_material_campaign_identity_v1:SELECT");
    }
    expected.sort_unstable();
    held == expected
}

/// Probe installed atom, verification, and economy capabilities without
/// granting access to their backing schemas.
fn reader_schema_markers(
    client: &mut postgres::Client,
) -> Result<(bool, bool, bool, bool, bool), SemanticArchiveReaderErrorV1> {
    let row = client
        .query_one(READER_SCHEMA_MARKERS_SQL_V1, &[])
        .map_err(|error| database_error("census reader schema markers", &error))?;
    let decode_marker = |index| {
        row.try_get(index)
            .map_err(|error| database_error("decode reader schema marker", &error))
    };
    Ok((
        decode_marker(0)?,
        decode_marker(1)?,
        decode_marker(2)?,
        decode_marker(3)?,
        decode_marker(4)?,
    ))
}

fn connection_target_error(error: &LegacyAdopterError) -> SemanticArchiveReaderErrorV1 {
    // The validator is a pure target check: its only failure construction is
    // one bounded target rejection, so any other variant is an internal fault.
    let LegacyAdopterError::UnsupportedConnectionTarget { reason } = error else {
        unreachable!("connection target validation only reports target rejections")
    };
    SemanticArchiveReaderErrorV1::ConnectionTarget(*reason)
}

/// Split read-only `PostgreSQL` handle for fog-safe Archive reads.
///
/// Writer operations (`install_schema`, `grant_knowledge`,
/// `materialize_receipt`, worker sweeps) are unrepresentable on this type;
/// [`SemanticArchiveStoreV1`](crate::SemanticArchiveStoreV1) and the runtime
/// binary remain the sole writers.
#[derive(Clone)]
pub struct SemanticArchiveReaderV1 {
    config: Config,
}

impl SemanticArchiveReaderV1 {
    /// Parse and validate one explicit local-only reader DSN.
    ///
    /// The raw parsed [`Config`] is validated before any bounded startup
    /// options are added, so caller-supplied `options`, host-address
    /// overrides, multi-host targets, and non-loopback hosts refuse before a
    /// socket opens.
    ///
    /// # Errors
    /// Returns [`SemanticArchiveReaderErrorV1`] for a malformed DSN or an
    /// out-of-contract connection target.
    pub fn from_dsn(raw: &str) -> Result<Self, SemanticArchiveReaderErrorV1> {
        let config = Config::from_str(raw).map_err(|_| SemanticArchiveReaderErrorV1::InvalidDsn)?;
        Self::new(&config)
    }

    /// Admit the reader DSN from [`READER_DSN_ENV_V1`].
    ///
    /// # Errors
    /// Returns [`SemanticArchiveReaderErrorV1`] for a missing or non-UTF-8
    /// environment value, a malformed DSN, or an out-of-contract target.
    pub fn from_env() -> Result<Self, SemanticArchiveReaderErrorV1> {
        let raw = std::env::var_os(READER_DSN_ENV_V1)
            .ok_or(SemanticArchiveReaderErrorV1::MissingEnv(READER_DSN_ENV_V1))?;
        let dsn = raw
            .into_string()
            .map_err(|_| SemanticArchiveReaderErrorV1::EnvNotUtf8(READER_DSN_ENV_V1))?;
        Self::from_dsn(&dsn)
    }

    /// Validate one explicit local-only connection target and bind the reader.
    ///
    /// # Errors
    /// Returns [`SemanticArchiveReaderErrorV1::ConnectionTarget`] for
    /// caller-supplied startup options, host-address overrides, multi-host or
    /// multi-port targets, a missing host, or a non-loopback TCP target.
    pub fn new(config: &Config) -> Result<Self, SemanticArchiveReaderErrorV1> {
        validate_legacy_connection_target(config)
            .map_err(|error| connection_target_error(&error))?;
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
    ) -> Result<Option<CommittedTickStatusV1>, SemanticArchiveReaderErrorV1> {
        let mut client = self.connect("connect committed tick status reader")?;
        client
            .query_opt(COMMITTED_TICK_STATUS_SQL_V1, &[campaign_id.as_uuid()])
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
    ) -> Result<Option<ArchiveVerificationStatusV1>, SemanticArchiveReaderErrorV1> {
        let mut client = self.connect("connect Archive verification reader")?;
        let row = client
            .query_opt(ARCHIVE_VERIFICATION_STATUS_SQL_V1, &[campaign_id.as_uuid()])
            .map_err(|error| database_error("read Archive verification status", &error))?;
        row.map(|row| {
            let durable_tick = u64::try_from(decode::<i64>(&row, 0)?)
                .map_err(|_| SemanticArchiveErrorV1::StoredPageMismatch)?;
            let processed_tick = u64::try_from(decode::<i64>(&row, 1)?)
                .map_err(|_| SemanticArchiveErrorV1::StoredPageMismatch)?;
            if durable_tick == 0 || processed_tick > durable_tick {
                return Err(SemanticArchiveErrorV1::StoredPageMismatch);
            }
            Ok(ArchiveVerificationStatusV1 {
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
    ) -> Result<postgres::Client, SemanticArchiveReaderErrorV1> {
        // The stored config stays raw: validation must observe the caller's
        // exact target, not the bounded startup options added here.
        let mut bounded = self.config.clone();
        bounded
            .connect_timeout(LEGACY_ADOPTER_CONNECT_TIMEOUT)
            .tcp_user_timeout(LEGACY_ADOPTER_TCP_USER_TIMEOUT)
            .options(LEGACY_ADOPTER_STARTUP_OPTIONS);
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
) -> Result<(), SemanticArchiveReaderErrorV1> {
    let row = client
        .query_one(READER_SESSION_AUTHORITY_SQL_V1, &[])
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
    if exact_reader_footprint(&held, reader_schema_markers(client)?) {
        Ok(())
    } else {
        Err(SemanticArchiveReaderErrorV1::WriterAuthorityRefused(held))
    }
}

/// Install the additive reader role and fog-safe tick-status view idempotently.
///
/// The installer mirrors the additive Archive pattern: one advisory lock and
/// one Serializable transaction for the role DDL, the view, its exact grant,
/// and the guarded archive-table revokes. `CREATE ROLE` is transactional in
/// `PostgreSQL`, so a failed install leaves no cluster-wide partial state.
/// The base `babylon_state` tables are never granted. This maintenance entry
/// point does not advance the schema epoch.
///
/// The existing-state path is a census, not a courtesy: the view must be a
/// plain view storing the pinned canonical definition, and the role's
/// effective-privilege census over the restricted relations must be exactly
/// the existence-dependent reader footprint (the tick-status view alone
/// before the atom schema, the five fog-safe views after it). Anything else
/// refuses with
/// [`SemanticArchiveReaderErrorV1::PrivilegeDrift`] (or
/// [`SemanticArchiveReaderErrorV1::ViewMismatch`]); drift is never silently
/// re-granted away.
///
/// # Errors
/// Refuses an out-of-contract target, an existing role with wrong attributes,
/// a view without the exact pinned identity, privilege drift, or database
/// failure.
pub fn install_reader_role_v1(
    config: &Config,
) -> Result<ReaderRoleDispositionV1, SemanticArchiveReaderErrorV1> {
    validate_legacy_connection_target(config).map_err(|error| connection_target_error(&error))?;
    let mut client = config
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
        (Ok(_), Ok(false)) => Err(SemanticArchiveReaderErrorV1::LockMismatch),
    }
}

fn install_reader_role_locked(
    client: &mut postgres::Client,
) -> Result<ReaderRoleDispositionV1, SemanticArchiveReaderErrorV1> {
    let row = client
        .query_one(READER_ROLE_MARKERS_SQL_V1, &[])
        .map_err(|error| database_error("inspect reader role markers", &error))?;
    let role_exists: bool = row
        .try_get(0)
        .map_err(|error| database_error("decode reader role marker", &error))?;
    let view_exists: bool = row
        .try_get(1)
        .map_err(|error| database_error("decode reader view marker", &error))?;
    if role_exists {
        verify_reader_role_attributes(client)?;
    }
    if view_exists {
        verify_reader_view_identity(client)?;
        let held =
            census_role_privileges(client, READER_ROLE_NAME_V1, "census reader role privileges")?;
        if !exact_reader_footprint(&held, reader_schema_markers(client)?) {
            return Err(SemanticArchiveReaderErrorV1::PrivilegeDrift(held));
        }
    }
    if !role_exists || !view_exists {
        let mut transaction = client
            .build_transaction()
            .isolation_level(postgres::IsolationLevel::Serializable)
            .start()
            .map_err(|error| database_error("begin reader schema install", &error))?;
        transaction
            .batch_execute(
                "SET LOCAL search_path TO pg_catalog; SET LOCAL synchronous_commit TO on",
            )
            .map_err(|error| database_error("set reader schema install settings", &error))?;
        if !role_exists {
            transaction
                .batch_execute(READER_ROLE_CREATE_SQL_V1)
                .map_err(|error| database_error("create reader role", &error))?;
        }
        if !view_exists {
            transaction
                .batch_execute(READER_ROLE_SCHEMA_V1_SQL)
                .map_err(|error| database_error("install reader schema", &error))?;
        }
        transaction
            .commit()
            .map_err(|error| database_error("commit reader schema install", &error))?;
        return Ok(ReaderRoleDispositionV1::Installed);
    }
    Ok(ReaderRoleDispositionV1::AlreadyCurrent)
}

fn verify_reader_role_attributes(
    client: &mut postgres::Client,
) -> Result<(), SemanticArchiveReaderErrorV1> {
    let row = client
        .query_one(READER_ROLE_ATTRIBUTES_SQL_V1, &[])
        .map_err(|error| database_error("inspect reader role attributes", &error))?;
    let locked = [
        row.try_get::<_, bool>(0)
            .map_err(|error| database_error("decode reader role superuser attribute", &error))?,
        row.try_get::<_, bool>(1)
            .map_err(|error| database_error("decode reader role createdb attribute", &error))?,
        row.try_get::<_, bool>(2)
            .map_err(|error| database_error("decode reader role createrole attribute", &error))?,
        row.try_get::<_, bool>(3)
            .map_err(|error| database_error("decode reader role login attribute", &error))?,
    ];
    if locked == [false; 4] {
        Ok(())
    } else {
        Err(SemanticArchiveReaderErrorV1::RoleMismatch)
    }
}

fn verify_reader_view_identity(
    client: &mut postgres::Client,
) -> Result<(), SemanticArchiveReaderErrorV1> {
    let row = client
        .query_opt(READER_VIEW_IDENTITY_SQL_V1, &[])
        .map_err(|error| database_error("inspect reader view identity", &error))?
        .ok_or(SemanticArchiveReaderErrorV1::ViewMismatch)?;
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
            == Some(READER_VIEW_CANONICAL_DEF_V1)
    {
        Ok(())
    } else {
        Err(SemanticArchiveReaderErrorV1::ViewMismatch)
    }
}

fn decode_committed_tick_status(
    campaign_id: CampaignId,
    row: &postgres::Row,
) -> Result<CommittedTickStatusV1, SemanticArchiveErrorV1> {
    let stored_campaign: uuid::Uuid = decode(row, 0)?;
    if stored_campaign != *campaign_id.as_uuid() {
        return Err(SemanticArchiveErrorV1::StoredPageMismatch);
    }
    let resolve_tick = u64::try_from(decode::<i64>(row, 1)?)
        .ok()
        .filter(|tick| *tick > 0)
        .ok_or(SemanticArchiveErrorV1::StoredPageMismatch)?;
    let envelope_layout_version: i16 = decode(row, 2)?;
    Ok(CommittedTickStatusV1 {
        campaign_id,
        resolve_tick,
        envelope_layout_version,
        tick_content_hash: decode_digest(row, 3)?,
        envelope_digest: decode_digest(row, 4)?,
    })
}
