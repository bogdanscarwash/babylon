//! Bounded catalog verification and connection guards for the native runtime.

use std::collections::{BTreeMap, BTreeSet};
use std::net::IpAddr;
use std::time::Duration;

use postgres::config::Host;
use postgres::{Config, Row};

use crate::{PostgresDiagnostic, SCHEMA_ADVISORY_LOCK_KEY};

/// Version of the canonical catalog census contract.
pub const CATALOG_CENSUS_VERSION: u16 = 2;
/// Maximum expected catalog objects accepted from one census.
pub const MAX_CATALOG_CENSUS_ROWS: usize = 512;
/// Maximum bytes accepted by the fixture parser.
pub const MAX_CATALOG_CENSUS_FIXTURE_BYTES: usize = 65_536;
/// Maximum fixture lines scanned by the bounded parser.
pub const MAX_CATALOG_CENSUS_FIXTURE_LINES: usize = MAX_CATALOG_CENSUS_ROWS + 32;
/// `PostgreSQL`'s `NAMEDATALEN - 1` identifier byte ceiling.
pub const POSTGRES_IDENTIFIER_MAX_BYTES: usize = 63;
/// Maximum child partitions checked for each governed parent.
pub const MAX_CATALOG_PARTITIONS_PER_FAMILY: usize = 4_096;
/// Maximum extension-owned catalog members checked across each installed extension.
pub const MAX_CATALOG_EXTENSION_MEMBERS: usize = 8_192;
/// Maximum canonical extension member and dependency addresses checked together.
pub const MAX_CATALOG_EXTENSION_DEPENDENCY_ADDRESSES: usize = 16_384;
/// Maximum distinct non-`PUBLIC` role identities referenced by installed extensions.
pub const MAX_CATALOG_EXTENSION_ROLE_IDENTITIES: usize = 8_192;
/// Maximum `OWNED BY` dependency accepted for one sequence.
pub const MAX_CATALOG_SEQUENCE_OWNERSHIP: usize = 1;
/// Bounded connection timeout for local game-managed `PostgreSQL`.
pub const CATALOG_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
/// Bounded TCP acknowledgement timeout for local game-managed `PostgreSQL`.
pub const CATALOG_TCP_USER_TIMEOUT: Duration = Duration::from_secs(5);
/// Hardened startup settings which replace any caller-supplied options.
pub const CATALOG_STARTUP_OPTIONS: &str =
    "-c default_transaction_read_only=on -c statement_timeout=5000ms \
     -c lock_timeout=5000ms -c idle_in_transaction_session_timeout=5000ms \
     -c quote_all_identifiers=off -c search_path=pg_catalog -c jit=off \
     -c event_triggers=off";

const MAX_ERROR_SOURCE_DEPTH: usize = 8;

const CATALOG_CENSUS_SQL: &str = include_str!("postgres_catalog.sql");
const LOCK_SQL: &str = "SELECT pg_catalog.pg_try_advisory_lock($1)";
const UNLOCK_SQL: &str = "SELECT pg_catalog.pg_advisory_unlock($1)";
/// Census object class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CatalogObjectKind {
    /// Versioned database and server environment contract.
    Database,
    /// `PostgreSQL` domain.
    Domain,
    /// Installed extension identity and version.
    Extension,
    /// Foreign table.
    ForeignTable,
    /// Materialized view.
    MaterializedView,
    /// Partitioned table parent.
    PartitionedTable,
    /// Ordinary table or relation.
    Relation,
    /// Normalized role contract.
    Role,
    /// User-defined routine family.
    Routine,
    /// Extra non-system schema identity.
    Schema,
    /// Governed schema grant.
    SchemaGrant,
    /// Sequence.
    Sequence,
    /// Fail-closed sentinel for an unsupported user-created catalog family.
    UnsupportedCatalog,
    /// Standalone user-defined non-domain type family.
    UserType,
    /// View.
    View,
}

/// Stable catalog object key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CatalogObjectKey {
    kind: CatalogObjectKind,
    schema: Box<str>,
    name: Box<str>,
}

/// One object signature row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogCensusEntry {
    key: CatalogObjectKey,
    digest_hex: Box<str>,
}

/// Parsed bounded census.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogCensus {
    entries: Vec<CatalogCensusEntry>,
}

/// Strict fixture parser failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogCensusParseError {
    /// Fixture bytes exceeded the public ceiling.
    TooManyBytes { actual: usize, max: usize },
    /// The parser hit the fixed line bound.
    TooManyLines { max: usize },
    /// More object rows were present than the fixed bound permits.
    TooManyRows { actual: usize, max: usize },
    /// A row did not have the exact four fields.
    MalformedRecord { line: usize, fields: usize },
    /// A kind field was not in the closed vocabulary.
    InvalidKind { line: usize },
    /// Schema or object identifier was outside the strict vocabulary or byte ceiling.
    InvalidIdentifier { line: usize },
    /// Digest was not 64 lowercase hexadecimal bytes.
    InvalidDigest { line: usize },
    /// The same object key appeared twice.
    DuplicateObject { line: usize },
    /// Fixture entries were not strictly sorted by key.
    OutOfOrder { line: usize },
    /// No object rows were present.
    Empty,
}

/// Bounded database operation names safe for display and logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogOperation {
    /// Acquire the schema advisory lock.
    Lock,
    /// Read the catalog census.
    Census,
    /// Release the schema advisory lock.
    Unlock,
}

/// Safe reason that a caller's connection target is outside the local-only contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionTargetRejection {
    /// Caller-supplied startup options could redirect unqualified database operations.
    StartupOptionsOverride,
    /// No explicit target was supplied.
    MissingHost,
    /// More than one host was supplied.
    MultipleHosts,
    /// More than one port was supplied.
    MultiplePorts,
    /// A separate host-address override could redirect the named target.
    HostAddressOverride,
    /// A TCP target was not a literal loopback address.
    NonLoopbackTcp,
    /// A Unix-domain socket path was not absolute.
    NonAbsoluteUnixSocket,
}

/// Fixed resources protected by public bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogBoundedResource {
    /// Census rows.
    CensusRows,
    /// Ordinary bounded catalog candidate or subordinate rows.
    CatalogRows,
    /// Direct child partitions within one governed family.
    PartitionRows,
    /// Direct members recorded for one extension.
    ExtensionMembers,
    /// Canonical direct-member and dependency addresses across installed extensions.
    ExtensionDependencyAddresses,
    /// Distinct non-`PUBLIC` role identities referenced by installed extensions.
    ExtensionRoleIdentities,
    /// `OWNED BY` dependencies attached to one sequence.
    SequenceOwnership,
    /// One database-origin identifier.
    IdentifierBytes,
}

/// Typed catalog refusal and failure states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogError {
    /// The supplied target was not one exact local socket endpoint.
    UnsupportedConnectionTarget {
        /// Bounded rejection reason which never contains the target text.
        reason: ConnectionTargetRejection,
    },
    /// A bounded database operation timed out.
    Timeout {
        operation: CatalogOperation,
        diagnostic: PostgresDiagnostic,
    },
    /// A read query failed.
    Query {
        operation: CatalogOperation,
        diagnostic: PostgresDiagnostic,
    },
    /// The exact schema lock was already held.
    LockUnavailable,
    /// The same actual census key appeared more than once.
    DuplicateCensusObject { key: CatalogObjectKey },
    /// Expected object signatures did not match.
    CensusMismatch {
        /// Expected objects absent from the live census.
        missing_objects: Vec<CatalogObjectKey>,
        /// Expected objects present with changed signatures.
        changed_objects: Vec<CatalogObjectKey>,
        /// Bounded sorted whole extra objects observed concurrently.
        extra_objects: Vec<CatalogObjectKey>,
    },
    /// One or more whole extra objects were observed.
    UnsupportedCatalogExtras { objects: Vec<CatalogObjectKey> },
    /// Database values could not be decoded into the contract shape.
    Decode { operation: CatalogOperation },
    /// A fixed row bound was exceeded.
    Bounds {
        resource: CatalogBoundedResource,
        actual: usize,
        max: usize,
    },
    /// Releasing the schema advisory lock failed.
    Cleanup {
        operation: CatalogOperation,
        diagnostic: Option<PostgresDiagnostic>,
    },
}

impl CatalogObjectKey {
    /// Create one validated object key.
    ///
    /// # Errors
    /// Returns [`CatalogCensusParseError::InvalidIdentifier`] for non-canonical names.
    pub fn new(
        kind: CatalogObjectKind,
        schema: &str,
        name: &str,
    ) -> Result<Self, CatalogCensusParseError> {
        if !valid_identifier(schema) || !valid_object_name(name) {
            return Err(CatalogCensusParseError::InvalidIdentifier { line: 0 });
        }
        Ok(Self {
            kind,
            schema: schema.into(),
            name: name.into(),
        })
    }

    /// Decode one key from `PostgreSQL` catalog identifiers.
    ///
    /// Database identifiers may be quoted, mixed-case, or Unicode. This path therefore enforces
    /// only `PostgreSQL`'s nonempty 63-byte identifier ceiling; fixture parsing remains canonical.
    ///
    /// # Errors
    /// Returns [`CatalogError`] when either database value is empty or byte-unbounded.
    pub fn from_database(
        kind: CatalogObjectKind,
        schema: &str,
        name: &str,
    ) -> Result<Self, CatalogError> {
        validate_database_identifier(schema)?;
        validate_database_identifier(name)?;
        Ok(Self {
            kind,
            schema: schema.into(),
            name: name.into(),
        })
    }

    /// Return the object kind.
    #[must_use]
    pub fn kind(&self) -> CatalogObjectKind {
        self.kind
    }

    /// Return the schema/category namespace.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Return the object name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl CatalogCensusEntry {
    /// Create one validated census entry.
    ///
    /// # Errors
    /// Returns [`CatalogCensusParseError`] when the digest is malformed.
    pub fn new(key: CatalogObjectKey, digest_hex: &str) -> Result<Self, CatalogCensusParseError> {
        if !valid_digest(digest_hex) {
            return Err(CatalogCensusParseError::InvalidDigest { line: 0 });
        }
        Ok(Self {
            key,
            digest_hex: digest_hex.into(),
        })
    }

    /// Return the object key.
    #[must_use]
    pub fn key(&self) -> &CatalogObjectKey {
        &self.key
    }
}

impl CatalogCensus {
    /// Return entries in canonical sorted order.
    #[must_use]
    pub fn entries(&self) -> &[CatalogCensusEntry] {
        &self.entries
    }
}

/// Parse one bounded, strictly sorted census fixture.
///
/// # Errors
/// Returns [`CatalogCensusParseError`] for malformed or unbounded input.
pub fn parse_catalog_census(text: &str) -> Result<CatalogCensus, CatalogCensusParseError> {
    if text.len() > MAX_CATALOG_CENSUS_FIXTURE_BYTES {
        return Err(CatalogCensusParseError::TooManyBytes {
            actual: text.len(),
            max: MAX_CATALOG_CENSUS_FIXTURE_BYTES,
        });
    }
    let mut entries = Vec::new();
    let mut keys = BTreeSet::new();
    let mut previous_key: Option<CatalogObjectKey> = None;
    for (line_index, line) in text
        .lines()
        .enumerate()
        .take(MAX_CATALOG_CENSUS_FIXTURE_LINES + 1)
    {
        check_fixture_line_bound(line_index)?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line_number = line_index + 1;
        let entry = parse_census_line(line, line_number)?;
        check_fixture_order(&entry, previous_key.as_ref(), line_number)?;
        if !keys.insert(entry.key.clone()) {
            return Err(CatalogCensusParseError::DuplicateObject { line: line_number });
        }
        previous_key = Some(entry.key.clone());
        entries.push(entry);
        check_fixture_row_bound(entries.len())?;
    }
    if entries.is_empty() {
        return Err(CatalogCensusParseError::Empty);
    }
    Ok(CatalogCensus { entries })
}

/// Compare the expected and actual census after validating actual row bounds and uniqueness.
///
/// # Errors
/// Returns [`CatalogError`] for unbounded, duplicate, missing, or changed actual objects.
pub fn compare_catalog_census(
    expected: &CatalogCensus,
    actual: &[CatalogCensusEntry],
) -> Result<(), CatalogError> {
    let actual_map = checked_census_map(actual)?;
    let expected_map = checked_census_map(expected.entries())?;
    let mut missing_objects = Vec::new();
    let mut changed_objects = Vec::new();
    for (key, expected_digest) in expected_map.iter().take(MAX_CATALOG_CENSUS_ROWS) {
        match actual_map.get(key) {
            Some(actual_digest) if actual_digest == expected_digest => {}
            Some(_) => changed_objects.push(key.clone()),
            None => missing_objects.push(key.clone()),
        }
    }
    let extra_objects = actual_map
        .keys()
        .filter(|key| !expected_map.contains_key(*key))
        .take(MAX_CATALOG_CENSUS_ROWS)
        .cloned()
        .collect::<Vec<_>>();
    if !missing_objects.is_empty() || !changed_objects.is_empty() {
        return Err(CatalogError::CensusMismatch {
            missing_objects,
            changed_objects,
            extra_objects,
        });
    }
    if !extra_objects.is_empty() {
        return Err(CatalogError::UnsupportedCatalogExtras {
            objects: extra_objects,
        });
    }
    Ok(())
}

/// Require one explicit local-only connection target before any socket is opened.
///
/// # Errors
/// Returns [`CatalogError::UnsupportedConnectionTarget`] for caller startup options or a
/// missing, remote, DNS, multi-host, or multi-port target.
pub fn validate_connection_target(config: &Config) -> Result<(), CatalogError> {
    if config.get_options().is_some() {
        return Err(connection_target_error(
            ConnectionTargetRejection::StartupOptionsOverride,
        ));
    }
    if !config.get_hostaddrs().is_empty() {
        return Err(connection_target_error(
            ConnectionTargetRejection::HostAddressOverride,
        ));
    }
    if config.get_ports().len() > 1 {
        return Err(connection_target_error(
            ConnectionTargetRejection::MultiplePorts,
        ));
    }
    let [host] = config.get_hosts() else {
        let reason = if config.get_hosts().is_empty() {
            ConnectionTargetRejection::MissingHost
        } else {
            ConnectionTargetRejection::MultipleHosts
        };
        return Err(connection_target_error(reason));
    };
    match host {
        Host::Tcp(address) => address
            .parse::<IpAddr>()
            .ok()
            .filter(IpAddr::is_loopback)
            .map_or_else(
                || {
                    Err(connection_target_error(
                        ConnectionTargetRejection::NonLoopbackTcp,
                    ))
                },
                |_| Ok(()),
            ),
        Host::Unix(path) if path.is_absolute() => Ok(()),
        Host::Unix(_) => Err(connection_target_error(
            ConnectionTargetRejection::NonAbsoluteUnixSocket,
        )),
    }
}

pub(crate) fn read_census_rows(
    transaction: &mut impl postgres::GenericClient,
) -> Result<Vec<CatalogCensusEntry>, CatalogError> {
    let operation = CatalogOperation::Census;
    let row_limit = bounded_limit(MAX_CATALOG_CENSUS_ROWS, CatalogBoundedResource::CensusRows)?;
    let partition_limit = bounded_limit(
        MAX_CATALOG_PARTITIONS_PER_FAMILY,
        CatalogBoundedResource::PartitionRows,
    )?;
    let extension_member_limit = bounded_limit(
        MAX_CATALOG_EXTENSION_MEMBERS,
        CatalogBoundedResource::ExtensionMembers,
    )?;
    let extension_dependency_address_limit = bounded_limit(
        MAX_CATALOG_EXTENSION_DEPENDENCY_ADDRESSES,
        CatalogBoundedResource::ExtensionDependencyAddresses,
    )?;
    let extension_role_identity_limit = bounded_limit(
        MAX_CATALOG_EXTENSION_ROLE_IDENTITIES,
        CatalogBoundedResource::ExtensionRoleIdentities,
    )?;
    let sequence_ownership_limit = bounded_limit(
        MAX_CATALOG_SEQUENCE_OWNERSHIP,
        CatalogBoundedResource::SequenceOwnership,
    )?;
    let rows = transaction
        .query(
            CATALOG_CENSUS_SQL,
            &[
                &row_limit,
                &partition_limit,
                &extension_member_limit,
                &extension_dependency_address_limit,
                &sequence_ownership_limit,
                &extension_role_identity_limit,
            ],
        )
        .map_err(|error| query_error(&error, operation))?;
    decode_census_rows(rows.as_slice())
}

fn decode_census_rows(rows: &[Row]) -> Result<Vec<CatalogCensusEntry>, CatalogError> {
    let operation = CatalogOperation::Census;
    let first = rows.first().ok_or(CatalogError::Decode { operation })?;
    decode_catalog_overflow(first)?;
    check_row_bound(
        CatalogBoundedResource::CensusRows,
        rows.len(),
        MAX_CATALOG_CENSUS_ROWS,
    )?;
    let mut entries = Vec::with_capacity(rows.len());
    let mut keys = BTreeSet::new();
    for row in rows.iter().take(MAX_CATALOG_CENSUS_ROWS) {
        let kind = parse_kind(try_text(row, 0, operation)?.as_str())
            .ok_or(CatalogError::Decode { operation })?;
        let schema = try_text(row, 1, operation)?;
        let name = try_text(row, 2, operation)?;
        let digest = try_text(row, 3, operation)?;
        let key = CatalogObjectKey::from_database(kind, &schema, &name)?;
        if !keys.insert(key.clone()) {
            return Err(CatalogError::DuplicateCensusObject { key });
        }
        entries.push(
            CatalogCensusEntry::new(key, &digest)
                .map_err(|_| CatalogError::Decode { operation })?,
        );
    }
    Ok(entries)
}

fn decode_catalog_overflow(row: &Row) -> Result<(), CatalogError> {
    let operation = CatalogOperation::Census;
    let raw_resource: Option<String> = row
        .try_get(4)
        .map_err(|_| CatalogError::Decode { operation })?;
    let actual: Option<i64> = row
        .try_get(5)
        .map_err(|_| CatalogError::Decode { operation })?;
    let max: Option<i64> = row
        .try_get(6)
        .map_err(|_| CatalogError::Decode { operation })?;
    let (raw_resource, actual, max) = match (raw_resource, actual, max) {
        (None, None, None) => return Ok(()),
        (Some(resource), Some(actual), Some(max)) => (resource, actual, max),
        _ => return Err(CatalogError::Decode { operation }),
    };
    let resource =
        parse_bounded_resource(&raw_resource).ok_or(CatalogError::Decode { operation })?;
    let actual = usize::try_from(actual).map_err(|_| CatalogError::Decode { operation })?;
    let max = usize::try_from(max).map_err(|_| CatalogError::Decode { operation })?;
    Err(CatalogError::Bounds {
        resource,
        actual,
        max,
    })
}

pub(crate) fn acquire_lock(client: &mut postgres::Client) -> Result<(), CatalogError> {
    let operation = CatalogOperation::Lock;
    let locked: bool = client
        .query_one(LOCK_SQL, &[&SCHEMA_ADVISORY_LOCK_KEY])
        .map_err(|error| query_error(&error, operation))?
        .try_get(0)
        .map_err(|_| CatalogError::Decode { operation })?;
    if locked {
        Ok(())
    } else {
        Err(CatalogError::LockUnavailable)
    }
}

pub(crate) fn release_lock(client: &mut postgres::Client) -> Result<(), CatalogError> {
    let operation = CatalogOperation::Unlock;
    let row = client
        .query_one(UNLOCK_SQL, &[&SCHEMA_ADVISORY_LOCK_KEY])
        .map_err(|error| CatalogError::Cleanup {
            operation,
            diagnostic: Some(PostgresDiagnostic::capture(&error)),
        })?;
    let unlocked: bool = row.try_get(0).map_err(|error| CatalogError::Cleanup {
        operation,
        diagnostic: Some(PostgresDiagnostic::capture(&error)),
    })?;
    if unlocked {
        Ok(())
    } else {
        Err(CatalogError::Cleanup {
            operation,
            diagnostic: None,
        })
    }
}

fn checked_census_map(
    entries: &[CatalogCensusEntry],
) -> Result<BTreeMap<CatalogObjectKey, Box<str>>, CatalogError> {
    check_row_bound(
        CatalogBoundedResource::CensusRows,
        entries.len(),
        MAX_CATALOG_CENSUS_ROWS,
    )?;
    let mut map = BTreeMap::new();
    for entry in entries.iter().take(MAX_CATALOG_CENSUS_ROWS) {
        if map
            .insert(entry.key.clone(), entry.digest_hex.clone())
            .is_some()
        {
            return Err(CatalogError::DuplicateCensusObject {
                key: entry.key.clone(),
            });
        }
    }
    Ok(map)
}

fn parse_census_line(
    line: &str,
    line_number: usize,
) -> Result<CatalogCensusEntry, CatalogCensusParseError> {
    let field_count = census_field_count(line);
    let mut fields = line.split('|');
    let kind = fields.next();
    let schema = fields.next();
    let name = fields.next();
    let digest = fields.next();
    let fifth = fields.next();
    let (Some(kind), Some(schema), Some(name), Some(digest), None) =
        (kind, schema, name, digest, fifth)
    else {
        return Err(CatalogCensusParseError::MalformedRecord {
            line: line_number,
            fields: field_count,
        });
    };
    let kind =
        parse_kind(kind).ok_or(CatalogCensusParseError::InvalidKind { line: line_number })?;
    let key = CatalogObjectKey::new(kind, schema, name)
        .map_err(|_| CatalogCensusParseError::InvalidIdentifier { line: line_number })?;
    CatalogCensusEntry::new(key, digest)
        .map_err(|_| CatalogCensusParseError::InvalidDigest { line: line_number })
}

fn census_field_count(line: &str) -> usize {
    line.as_bytes()
        .iter()
        .take(MAX_CATALOG_CENSUS_FIXTURE_BYTES + 1)
        .filter(|byte| **byte == b'|')
        .count()
        .checked_add(1)
        .expect("bounded census field count must fit usize")
}

fn parse_kind(raw: &str) -> Option<CatalogObjectKind> {
    match raw {
        "database" => Some(CatalogObjectKind::Database),
        "domain" => Some(CatalogObjectKind::Domain),
        "extension" => Some(CatalogObjectKind::Extension),
        "foreign_table" => Some(CatalogObjectKind::ForeignTable),
        "materialized_view" => Some(CatalogObjectKind::MaterializedView),
        "partitioned_table" => Some(CatalogObjectKind::PartitionedTable),
        "relation" => Some(CatalogObjectKind::Relation),
        "role" => Some(CatalogObjectKind::Role),
        "routine" => Some(CatalogObjectKind::Routine),
        "schema" => Some(CatalogObjectKind::Schema),
        "schema_grant" => Some(CatalogObjectKind::SchemaGrant),
        "sequence" => Some(CatalogObjectKind::Sequence),
        "unsupported_catalog" => Some(CatalogObjectKind::UnsupportedCatalog),
        "user_type" => Some(CatalogObjectKind::UserType),
        "view" => Some(CatalogObjectKind::View),
        _ => None,
    }
}

fn parse_bounded_resource(raw: &str) -> Option<CatalogBoundedResource> {
    match raw {
        "census_rows" => Some(CatalogBoundedResource::CensusRows),
        "catalog_rows" => Some(CatalogBoundedResource::CatalogRows),
        "partition_rows" => Some(CatalogBoundedResource::PartitionRows),
        "extension_members" => Some(CatalogBoundedResource::ExtensionMembers),
        "extension_dependency_addresses" => {
            Some(CatalogBoundedResource::ExtensionDependencyAddresses)
        }
        "extension_role_identities" => Some(CatalogBoundedResource::ExtensionRoleIdentities),
        "sequence_ownership" => Some(CatalogBoundedResource::SequenceOwnership),
        _ => None,
    }
}

fn check_fixture_line_bound(line_index: usize) -> Result<(), CatalogCensusParseError> {
    if line_index < MAX_CATALOG_CENSUS_FIXTURE_LINES {
        Ok(())
    } else {
        Err(CatalogCensusParseError::TooManyLines {
            max: MAX_CATALOG_CENSUS_FIXTURE_LINES,
        })
    }
}

fn check_fixture_row_bound(actual: usize) -> Result<(), CatalogCensusParseError> {
    if actual <= MAX_CATALOG_CENSUS_ROWS {
        Ok(())
    } else {
        Err(CatalogCensusParseError::TooManyRows {
            actual,
            max: MAX_CATALOG_CENSUS_ROWS,
        })
    }
}

fn check_fixture_order(
    entry: &CatalogCensusEntry,
    previous: Option<&CatalogObjectKey>,
    line: usize,
) -> Result<(), CatalogCensusParseError> {
    if previous.is_some_and(|key| key > entry.key()) {
        Err(CatalogCensusParseError::OutOfOrder { line })
    } else {
        Ok(())
    }
}

fn check_row_bound(
    resource: CatalogBoundedResource,
    actual: usize,
    max: usize,
) -> Result<(), CatalogError> {
    if actual <= max {
        Ok(())
    } else {
        Err(CatalogError::Bounds {
            resource,
            actual,
            max,
        })
    }
}

fn bounded_limit(max: usize, resource: CatalogBoundedResource) -> Result<i64, CatalogError> {
    let actual = max.checked_add(1).ok_or(CatalogError::Bounds {
        resource,
        actual: max,
        max,
    })?;
    i64::try_from(actual).map_err(|_| CatalogError::Bounds {
        resource,
        actual,
        max,
    })
}

fn try_text(row: &Row, index: usize, operation: CatalogOperation) -> Result<String, CatalogError> {
    row.try_get(index)
        .map_err(|_| CatalogError::Decode { operation })
}

fn query_error(error: &postgres::Error, operation: CatalogOperation) -> CatalogError {
    let diagnostic = PostgresDiagnostic::capture(error);
    if is_timeout(error) {
        CatalogError::Timeout {
            operation,
            diagnostic,
        }
    } else {
        CatalogError::Query {
            operation,
            diagnostic,
        }
    }
}

fn is_timeout(error: &postgres::Error) -> bool {
    error.code().is_some_and(|code| {
        code == &postgres::error::SqlState::QUERY_CANCELED
            || code == &postgres::error::SqlState::LOCK_NOT_AVAILABLE
    }) || error_chain_has_timeout(error)
}

fn error_chain_has_timeout(error: &(dyn std::error::Error + 'static)) -> bool {
    let mut current = Some(error);
    for _depth in 0..MAX_ERROR_SOURCE_DEPTH {
        let Some(source) = current else {
            return false;
        };
        if source
            .downcast_ref::<std::io::Error>()
            .is_some_and(|io_error| io_error.kind() == std::io::ErrorKind::TimedOut)
        {
            return true;
        }
        current = source.source();
    }
    false
}

fn valid_digest(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .take(POSTGRES_IDENTIFIER_MAX_BYTES + 1)
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn valid_identifier(text: &str) -> bool {
    valid_identifier_length(text) && valid_identifier_bytes(text, false)
}

fn validate_database_identifier(text: &str) -> Result<(), CatalogError> {
    validate_database_identifier_for(text, CatalogOperation::Census)
}

fn validate_database_identifier_for(
    text: &str,
    operation: CatalogOperation,
) -> Result<(), CatalogError> {
    if text.is_empty() || text.contains('\0') {
        return Err(CatalogError::Decode { operation });
    }
    if text.len() > POSTGRES_IDENTIFIER_MAX_BYTES {
        return Err(CatalogError::Bounds {
            resource: CatalogBoundedResource::IdentifierBytes,
            actual: text.len(),
            max: POSTGRES_IDENTIFIER_MAX_BYTES,
        });
    }
    Ok(())
}

fn connection_target_error(reason: ConnectionTargetRejection) -> CatalogError {
    CatalogError::UnsupportedConnectionTarget { reason }
}

fn valid_object_name(text: &str) -> bool {
    valid_identifier_length(text) && valid_identifier_bytes(text, true)
}

fn valid_identifier_length(text: &str) -> bool {
    !text.is_empty() && text.len() <= POSTGRES_IDENTIFIER_MAX_BYTES
}

fn valid_identifier_bytes(text: &str, allow_hyphen: bool) -> bool {
    let mut bytes = text.bytes().take(POSTGRES_IDENTIFIER_MAX_BYTES + 1);
    let Some(first) = bytes.next() else {
        return false;
    };
    matches!(first, b'a'..=b'z' | b'_')
        && bytes.all(|byte| {
            matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'_') || (allow_hyphen && byte == b'-')
        })
}

impl std::fmt::Display for CatalogCensusParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid catalog census fixture: {self:?}")
    }
}

impl std::error::Error for CatalogCensusParseError {}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "catalog verification failure: {self:?}")
    }
}

impl std::error::Error for CatalogError {}

#[cfg(test)]
mod tests {
    use super::{
        check_row_bound, error_chain_has_timeout, CatalogBoundedResource, CatalogError,
        MAX_CATALOG_EXTENSION_DEPENDENCY_ADDRESSES,
    };

    #[test]
    fn extension_dependency_address_bound_refuses_first_excess() {
        let actual = 16_385_usize;
        assert_eq!(actual, MAX_CATALOG_EXTENSION_DEPENDENCY_ADDRESSES + 1);
        assert_eq!(
            check_row_bound(
                CatalogBoundedResource::ExtensionDependencyAddresses,
                actual,
                MAX_CATALOG_EXTENSION_DEPENDENCY_ADDRESSES,
            ),
            Err(CatalogError::Bounds {
                resource: CatalogBoundedResource::ExtensionDependencyAddresses,
                actual,
                max: MAX_CATALOG_EXTENSION_DEPENDENCY_ADDRESSES,
            })
        );
    }

    #[test]
    fn io_timeout_is_classified_without_exposing_error_text() {
        let timeout = std::io::Error::new(std::io::ErrorKind::TimedOut, "secret-bearing context");
        let refused = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "ordinary error");
        assert!(error_chain_has_timeout(&timeout));
        assert!(!error_chain_has_timeout(&refused));
    }
}
