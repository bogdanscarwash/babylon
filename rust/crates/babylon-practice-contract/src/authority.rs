//! Pure current campaign habitation and practice-input authority contracts.

use babylon_kernel::content_digest::sha256_of;

use crate::actor::ActorOrganizationId;

const SCHEMA_VERSION: u16 = 2;

/// Canonical row domain for authoritative current input authority.
pub const PRACTICE_INPUT_AUTHORITY_DOMAIN_BYTES: &[u8] = b"babylon.practice-input-authority.v2";
/// Canonical ledger domain for authoritative current input authority rows.
pub const PRACTICE_INPUT_AUTHORITY_LEDGER_DOMAIN_BYTES: &[u8] =
    b"babylon.practice-input-authority-ledger.v2";
/// SHA-256 of the exact language-neutral current authority schema bytes.
pub const PRACTICE_INPUT_AUTHORITY_SOURCE_SHA256: [u8; 32] = [
    0x2e, 0x62, 0xa9, 0xe7, 0xf4, 0xc7, 0xd5, 0x08, 0xbf, 0xc1, 0x01, 0x68, 0x0e, 0x73, 0x76, 0x6f,
    0xf9, 0x47, 0x5e, 0x98, 0x9f, 0xb0, 0x24, 0xba, 0x8e, 0x98, 0xa5, 0x8f, 0x5c, 0x12, 0x9e, 0xe8,
];
/// Designed serialization and validation-fuel ceiling, not an organization quota.
pub const MAX_PRACTICE_INPUT_AUTHORITY_ROWS: usize = 16_384;

/// Exact canonical byte length of one frozen current input-authority row.
pub const PRACTICE_INPUT_AUTHORITY_CANONICAL_BYTES: usize =
    PRACTICE_INPUT_AUTHORITY_DOMAIN_BYTES.len() + 1 + 2 + 16 + 1 + 16 + 8 + 8 + 8 + 32;

/// Exact current authority-contract refusals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PracticeAuthorityError {
    AuthorityDomain = 1,
    AuthoritySchemaVersion = 2,
    AuthorityEnumCode = 3,
    AuthorityTruncated = 4,
    AuthorityTrailingBytes = 5,
    AuthorityEmptyInterval = 6,
    AuthorityLedgerLimit = 7,
    AuthorityLedgerOrder = 8,
    AuthorityLedgerDuplicate = 9,
    AuthorityIntervalOverlap = 10,
    AuthorityPlayerSeatOverlap = 11,
    AuthorityNotFound = 12,
    AuthorityInactive = 13,
    AuthorityActorMismatch = 14,
    AuthorityPlayerSeatMissing = 15,
    AuthorityPlayerSeatReassignment = 16,
}

/// Unknown current authority-contract error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownPracticeAuthorityErrorCode(pub u16);

impl TryFrom<u16> for PracticeAuthorityError {
    type Error = UnknownPracticeAuthorityErrorCode;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::AuthorityDomain),
            2 => Ok(Self::AuthoritySchemaVersion),
            3 => Ok(Self::AuthorityEnumCode),
            4 => Ok(Self::AuthorityTruncated),
            5 => Ok(Self::AuthorityTrailingBytes),
            6 => Ok(Self::AuthorityEmptyInterval),
            7 => Ok(Self::AuthorityLedgerLimit),
            8 => Ok(Self::AuthorityLedgerOrder),
            9 => Ok(Self::AuthorityLedgerDuplicate),
            10 => Ok(Self::AuthorityIntervalOverlap),
            11 => Ok(Self::AuthorityPlayerSeatOverlap),
            12 => Ok(Self::AuthorityNotFound),
            13 => Ok(Self::AuthorityInactive),
            14 => Ok(Self::AuthorityActorMismatch),
            15 => Ok(Self::AuthorityPlayerSeatMissing),
            16 => Ok(Self::AuthorityPlayerSeatReassignment),
            _ => Err(UnknownPracticeAuthorityErrorCode(value)),
        }
    }
}

impl From<PracticeAuthorityError> for u16 {
    fn from(value: PracticeAuthorityError) -> Self {
        value as Self
    }
}

/// Opaque canonical campaign UUID bytes in RFC 4122 network order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CampaignId([u8; 16]);

impl CampaignId {
    /// Construct from canonical UUID bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Return the canonical UUID bytes.
    #[must_use]
    pub const fn as_bytes(self) -> [u8; 16] {
        self.0
    }
}

/// Opaque canonical authority UUID bytes in RFC 4122 network order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct InputAuthorityId([u8; 16]);

impl InputAuthorityId {
    /// Construct from canonical UUID bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Return the canonical UUID bytes.
    #[must_use]
    pub const fn as_bytes(self) -> [u8; 16] {
        self.0
    }
}

/// Closed current authority-kind table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PracticeAuthorityKind {
    PlayerSeat = 1,
    DeterministicPolicy = 2,
}

impl TryFrom<u8> for PracticeAuthorityKind {
    type Error = PracticeAuthorityError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::PlayerSeat),
            2 => Ok(Self::DeterministicPolicy),
            _ => Err(PracticeAuthorityError::AuthorityEnumCode),
        }
    }
}

/// One authoritative campaign habitation or deterministic-policy row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeInputAuthority {
    pub schema_version: u16,
    pub campaign_id: CampaignId,
    pub authority_kind: PracticeAuthorityKind,
    pub input_authority_id: InputAuthorityId,
    pub actor_org_id: ActorOrganizationId,
    pub effective_from_tick: u64,
    pub effective_through_tick_exclusive: u64,
    pub decision_content_digest: [u8; 32],
}

/// Fixed row field order for language-neutral implementations.
pub const PRACTICE_INPUT_AUTHORITY_FIELD_ORDER: [&str; 8] = [
    "schema_version",
    "campaign_id",
    "authority_kind",
    "input_authority_id",
    "actor_org_id",
    "effective_from_tick",
    "effective_through_tick_exclusive",
    "decision_content_digest",
];

/// Sorted authoritative current input-authority ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeInputAuthorityLedger {
    pub schema_version: u16,
    pub rows: Vec<PracticeInputAuthority>,
}

fn validate_schema(value: u16) -> Result<(), PracticeAuthorityError> {
    if value == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(PracticeAuthorityError::AuthoritySchemaVersion)
    }
}

pub(crate) fn validate_input_authority_row(
    value: &PracticeInputAuthority,
) -> Result<(), PracticeAuthorityError> {
    validate_schema(value.schema_version)?;
    if value.effective_from_tick >= value.effective_through_tick_exclusive {
        return Err(PracticeAuthorityError::AuthorityEmptyInterval);
    }
    Ok(())
}

fn row_key(value: &PracticeInputAuthority) -> (CampaignId, InputAuthorityId, u64) {
    (
        value.campaign_id,
        value.input_authority_id,
        value.effective_from_tick,
    )
}

fn validate_player_intervals(
    rows: &[PracticeInputAuthority],
) -> Result<(), PracticeAuthorityError> {
    let mut intervals: Vec<(CampaignId, u64, u64, InputAuthorityId, ActorOrganizationId)> = rows
        .iter()
        .take(MAX_PRACTICE_INPUT_AUTHORITY_ROWS + 1)
        .filter(|row| row.authority_kind == PracticeAuthorityKind::PlayerSeat)
        .map(|row| {
            (
                row.campaign_id,
                row.effective_from_tick,
                row.effective_through_tick_exclusive,
                row.input_authority_id,
                row.actor_org_id,
            )
        })
        .collect();
    intervals.sort_unstable();
    for pair in intervals.windows(2).take(MAX_PRACTICE_INPUT_AUTHORITY_ROWS) {
        let prior = pair[0];
        let current = pair[1];
        if prior.0 == current.0 && current.1 < prior.2 {
            return Err(PracticeAuthorityError::AuthorityPlayerSeatOverlap);
        }
        if prior.0 == current.0 && (prior.3 != current.3 || prior.4 != current.4) {
            return Err(PracticeAuthorityError::AuthorityPlayerSeatReassignment);
        }
    }
    Ok(())
}

/// Validate bounded canonical order and effective-interval laws.
///
/// # Errors
/// Returns the first exact schema, row, order, duplicate, or overlap refusal.
pub fn validate_input_authority_ledger(
    ledger: &PracticeInputAuthorityLedger,
) -> Result<(), PracticeAuthorityError> {
    validate_schema(ledger.schema_version)?;
    if ledger.rows.len() > MAX_PRACTICE_INPUT_AUTHORITY_ROWS {
        return Err(PracticeAuthorityError::AuthorityLedgerLimit);
    }
    let mut previous: Option<&PracticeInputAuthority> = None;
    for row in ledger
        .rows
        .iter()
        .take(MAX_PRACTICE_INPUT_AUTHORITY_ROWS + 1)
    {
        validate_input_authority_row(row)?;
        if let Some(prior) = previous {
            if row_key(prior) == row_key(row) {
                return Err(PracticeAuthorityError::AuthorityLedgerDuplicate);
            }
            if row_key(row) < row_key(prior) {
                return Err(PracticeAuthorityError::AuthorityLedgerOrder);
            }
            if prior.campaign_id == row.campaign_id
                && prior.input_authority_id == row.input_authority_id
                && row.effective_from_tick < prior.effective_through_tick_exclusive
            {
                return Err(PracticeAuthorityError::AuthorityIntervalOverlap);
            }
        }
        previous = Some(row);
    }
    validate_player_intervals(&ledger.rows)
}

fn append_domain(output: &mut Vec<u8>, domain: &[u8]) {
    output.extend_from_slice(domain);
    output.push(0);
}

/// Encode one current authority row in fixed big-endian order.
///
/// # Errors
/// Returns the exact schema or interval refusal.
pub fn encode_input_authority(
    value: &PracticeInputAuthority,
) -> Result<Vec<u8>, PracticeAuthorityError> {
    validate_input_authority_row(value)?;
    let mut output = Vec::with_capacity(PRACTICE_INPUT_AUTHORITY_CANONICAL_BYTES);
    append_domain(&mut output, PRACTICE_INPUT_AUTHORITY_DOMAIN_BYTES);
    output.extend_from_slice(&value.schema_version.to_be_bytes());
    output.extend_from_slice(&value.campaign_id.as_bytes());
    output.push(value.authority_kind as u8);
    output.extend_from_slice(&value.input_authority_id.as_bytes());
    output.extend_from_slice(&value.actor_org_id.to_bytes());
    output.extend_from_slice(&value.effective_from_tick.to_be_bytes());
    output.extend_from_slice(&value.effective_through_tick_exclusive.to_be_bytes());
    output.extend_from_slice(&value.decision_content_digest);
    Ok(output)
}

struct Cursor<'a> {
    payload: &'a [u8],
    index: usize,
}

impl<'a> Cursor<'a> {
    const fn new(payload: &'a [u8]) -> Self {
        Self { payload, index: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], PracticeAuthorityError> {
        let end = self
            .index
            .checked_add(count)
            .ok_or(PracticeAuthorityError::AuthorityTruncated)?;
        let value = self
            .payload
            .get(self.index..end)
            .ok_or(PracticeAuthorityError::AuthorityTruncated)?;
        self.index = end;
        Ok(value)
    }

    fn domain(&mut self, expected: &[u8]) -> Result<(), PracticeAuthorityError> {
        if self.take(expected.len())? == expected && self.take(1)? == [0] {
            Ok(())
        } else {
            Err(PracticeAuthorityError::AuthorityDomain)
        }
    }

    fn u8(&mut self) -> Result<u8, PracticeAuthorityError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, PracticeAuthorityError> {
        Ok(u16::from_be_bytes(
            self.take(2)?
                .try_into()
                .map_err(|_| PracticeAuthorityError::AuthorityTruncated)?,
        ))
    }

    fn u32(&mut self) -> Result<u32, PracticeAuthorityError> {
        Ok(u32::from_be_bytes(
            self.take(4)?
                .try_into()
                .map_err(|_| PracticeAuthorityError::AuthorityTruncated)?,
        ))
    }

    fn u64(&mut self) -> Result<u64, PracticeAuthorityError> {
        Ok(u64::from_be_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| PracticeAuthorityError::AuthorityTruncated)?,
        ))
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], PracticeAuthorityError> {
        self.take(N)?
            .try_into()
            .map_err(|_| PracticeAuthorityError::AuthorityTruncated)
    }

    fn finish(&self) -> Result<(), PracticeAuthorityError> {
        if self.index == self.payload.len() {
            Ok(())
        } else {
            Err(PracticeAuthorityError::AuthorityTrailingBytes)
        }
    }
}

/// Decode one complete current authority row.
///
/// # Errors
/// Returns the first exact domain, schema, enum, interval, truncation, or trailing refusal.
pub fn decode_input_authority(
    payload: &[u8],
) -> Result<PracticeInputAuthority, PracticeAuthorityError> {
    let mut cursor = Cursor::new(payload);
    cursor.domain(PRACTICE_INPUT_AUTHORITY_DOMAIN_BYTES)?;
    let schema_version = cursor.u16()?;
    validate_schema(schema_version)?;
    let campaign_id = CampaignId::from_bytes(cursor.array()?);
    let authority_kind = PracticeAuthorityKind::try_from(cursor.u8()?)?;
    let input_authority_id = InputAuthorityId::from_bytes(cursor.array()?);
    let actor_org_id = ActorOrganizationId::from_bytes(cursor.array()?);
    let effective_from_tick = cursor.u64()?;
    let effective_through_tick_exclusive = cursor.u64()?;
    let decision_content_digest = cursor.array()?;
    cursor.finish()?;
    let value = PracticeInputAuthority {
        schema_version,
        campaign_id,
        authority_kind,
        input_authority_id,
        actor_org_id,
        effective_from_tick,
        effective_through_tick_exclusive,
        decision_content_digest,
    };
    validate_input_authority_row(&value)?;
    Ok(value)
}

/// Encode one complete sorted current authority ledger.
///
/// # Errors
/// Returns the first exact ledger or row refusal.
pub fn encode_input_authority_ledger(
    ledger: &PracticeInputAuthorityLedger,
) -> Result<Vec<u8>, PracticeAuthorityError> {
    validate_input_authority_ledger(ledger)?;
    let mut output = Vec::new();
    append_domain(&mut output, PRACTICE_INPUT_AUTHORITY_LEDGER_DOMAIN_BYTES);
    output.extend_from_slice(&ledger.schema_version.to_be_bytes());
    let count = u32::try_from(ledger.rows.len())
        .map_err(|_| PracticeAuthorityError::AuthorityLedgerLimit)?;
    output.extend_from_slice(&count.to_be_bytes());
    for row in ledger
        .rows
        .iter()
        .take(MAX_PRACTICE_INPUT_AUTHORITY_ROWS + 1)
    {
        output.extend_from_slice(&encode_input_authority(row)?);
    }
    Ok(output)
}

/// Hash one successfully encoded current authority row.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn input_authority_digest(
    value: &PracticeInputAuthority,
) -> Result<[u8; 32], PracticeAuthorityError> {
    Ok(sha256_of(&encode_input_authority(value)?))
}

/// Hash one successfully encoded current authority ledger.
///
/// # Errors
/// Returns the exact ledger refusal without publishing a digest.
pub fn input_authority_ledger_digest(
    ledger: &PracticeInputAuthorityLedger,
) -> Result<[u8; 32], PracticeAuthorityError> {
    Ok(sha256_of(&encode_input_authority_ledger(ledger)?))
}

/// Decode one complete current authority ledger.
///
/// # Errors
/// Returns the first exact domain, row-count, row, order, or trailing refusal.
pub fn decode_input_authority_ledger(
    payload: &[u8],
) -> Result<PracticeInputAuthorityLedger, PracticeAuthorityError> {
    let mut cursor = Cursor::new(payload);
    cursor.domain(PRACTICE_INPUT_AUTHORITY_LEDGER_DOMAIN_BYTES)?;
    let schema_version = cursor.u16()?;
    validate_schema(schema_version)?;
    let count =
        usize::try_from(cursor.u32()?).map_err(|_| PracticeAuthorityError::AuthorityLedgerLimit)?;
    if count > MAX_PRACTICE_INPUT_AUTHORITY_ROWS {
        return Err(PracticeAuthorityError::AuthorityLedgerLimit);
    }
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_PRACTICE_INPUT_AUTHORITY_ROWS {
        if index == count {
            break;
        }
        rows.push(decode_input_authority(
            cursor.take(PRACTICE_INPUT_AUTHORITY_CANONICAL_BYTES)?,
        )?);
    }
    cursor.finish()?;
    let ledger = PracticeInputAuthorityLedger {
        schema_version,
        rows,
    };
    validate_input_authority_ledger(&ledger)?;
    Ok(ledger)
}

/// Resolve the one active authority row matching an intent identity.
///
/// # Errors
/// Returns an exact malformed-ledger, missing, inactive, or actor mismatch refusal.
pub fn resolve_input_authority(
    ledger: &PracticeInputAuthorityLedger,
    campaign_id: CampaignId,
    input_authority_id: InputAuthorityId,
    actor_org_id: ActorOrganizationId,
    resolve_tick: u64,
) -> Result<&PracticeInputAuthority, PracticeAuthorityError> {
    validate_input_authority_ledger(ledger)?;
    let mut found_identity = false;
    for row in ledger
        .rows
        .iter()
        .take(MAX_PRACTICE_INPUT_AUTHORITY_ROWS + 1)
    {
        if row.campaign_id != campaign_id || row.input_authority_id != input_authority_id {
            continue;
        }
        found_identity = true;
        if resolve_tick < row.effective_from_tick
            || resolve_tick >= row.effective_through_tick_exclusive
        {
            continue;
        }
        if row.actor_org_id != actor_org_id {
            return Err(PracticeAuthorityError::AuthorityActorMismatch);
        }
        return Ok(row);
    }
    if found_identity {
        Err(PracticeAuthorityError::AuthorityInactive)
    } else {
        Err(PracticeAuthorityError::AuthorityNotFound)
    }
}

/// Resolve the sole active player habitation row for one campaign tick.
///
/// # Errors
/// Returns an exact malformed-ledger, missing-seat, or overlapping-seat refusal.
pub fn active_player_authority(
    ledger: &PracticeInputAuthorityLedger,
    campaign_id: CampaignId,
    tick: u64,
) -> Result<&PracticeInputAuthority, PracticeAuthorityError> {
    validate_input_authority_ledger(ledger)?;
    let mut found: Option<&PracticeInputAuthority> = None;
    for row in ledger
        .rows
        .iter()
        .take(MAX_PRACTICE_INPUT_AUTHORITY_ROWS + 1)
    {
        if row.campaign_id == campaign_id
            && row.authority_kind == PracticeAuthorityKind::PlayerSeat
            && tick >= row.effective_from_tick
            && tick < row.effective_through_tick_exclusive
        {
            if found.is_some() {
                return Err(PracticeAuthorityError::AuthorityPlayerSeatOverlap);
            }
            found = Some(row);
        }
    }
    found.ok_or(PracticeAuthorityError::AuthorityPlayerSeatMissing)
}
