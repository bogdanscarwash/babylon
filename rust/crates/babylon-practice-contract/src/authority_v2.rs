//! Pure V2 campaign habitation and practice-input authority contracts.

use babylon_kernel::sha256_of;

use crate::actor_v2::ActorOrganizationIdV2;

const SCHEMA_VERSION: u16 = 2;

/// Canonical row domain for authoritative V2 input authority.
pub const PRACTICE_INPUT_AUTHORITY_V2_DOMAIN_BYTES: &[u8] = b"babylon.practice-input-authority.v2";
/// Canonical ledger domain for authoritative V2 input authority rows.
pub const PRACTICE_INPUT_AUTHORITY_LEDGER_V2_DOMAIN_BYTES: &[u8] =
    b"babylon.practice-input-authority-ledger.v2";
/// SHA-256 of the exact language-neutral V2 authority schema bytes.
pub const PRACTICE_INPUT_AUTHORITY_V2_SOURCE_SHA256: [u8; 32] = [
    0xdb, 0x43, 0x8c, 0xc8, 0x2f, 0xd3, 0x70, 0x62, 0x49, 0x6c, 0xcc, 0x48, 0x87, 0x5e, 0x9a, 0xdc,
    0xeb, 0x5f, 0x57, 0x96, 0xea, 0xbe, 0xff, 0x4d, 0x41, 0xa6, 0xfd, 0xc5, 0xd0, 0x8d, 0x57, 0x5e,
];
/// Designed serialization and validation-fuel ceiling, not an organization quota.
pub const MAX_PRACTICE_INPUT_AUTHORITY_ROWS_V2: usize = 16_384;

/// Exact canonical byte length of one frozen V2 input-authority row.
pub const PRACTICE_INPUT_AUTHORITY_V2_CANONICAL_BYTES: usize =
    PRACTICE_INPUT_AUTHORITY_V2_DOMAIN_BYTES.len() + 1 + 2 + 16 + 1 + 16 + 8 + 8 + 8 + 32;

/// Exact V2 authority-contract refusals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PracticeAuthorityV2Error {
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

/// Unknown V2 authority-contract error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownPracticeAuthorityV2ErrorCode(pub u16);

impl TryFrom<u16> for PracticeAuthorityV2Error {
    type Error = UnknownPracticeAuthorityV2ErrorCode;

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
            _ => Err(UnknownPracticeAuthorityV2ErrorCode(value)),
        }
    }
}

impl From<PracticeAuthorityV2Error> for u16 {
    fn from(value: PracticeAuthorityV2Error) -> Self {
        value as Self
    }
}

/// Opaque canonical campaign UUID bytes in RFC 4122 network order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CampaignIdV2([u8; 16]);

impl CampaignIdV2 {
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
pub struct InputAuthorityIdV2([u8; 16]);

impl InputAuthorityIdV2 {
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

/// Closed V2 authority-kind table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PracticeAuthorityKindV2 {
    PlayerSeat = 1,
    DeterministicPolicy = 2,
}

impl TryFrom<u8> for PracticeAuthorityKindV2 {
    type Error = PracticeAuthorityV2Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::PlayerSeat),
            2 => Ok(Self::DeterministicPolicy),
            _ => Err(PracticeAuthorityV2Error::AuthorityEnumCode),
        }
    }
}

/// One authoritative campaign habitation or deterministic-policy row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeInputAuthorityV2 {
    pub schema_version: u16,
    pub campaign_id: CampaignIdV2,
    pub authority_kind: PracticeAuthorityKindV2,
    pub input_authority_id: InputAuthorityIdV2,
    pub actor_org_id: ActorOrganizationIdV2,
    pub effective_from_tick: u64,
    pub effective_through_tick_exclusive: u64,
    pub decision_content_digest: [u8; 32],
}

/// Fixed row field order for language-neutral implementations.
pub const PRACTICE_INPUT_AUTHORITY_V2_FIELD_ORDER: [&str; 8] = [
    "schema_version",
    "campaign_id",
    "authority_kind",
    "input_authority_id",
    "actor_org_id",
    "effective_from_tick",
    "effective_through_tick_exclusive",
    "decision_content_digest",
];

/// Sorted authoritative V2 input-authority ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeInputAuthorityLedgerV2 {
    pub schema_version: u16,
    pub rows: Vec<PracticeInputAuthorityV2>,
}

fn validate_schema(value: u16) -> Result<(), PracticeAuthorityV2Error> {
    if value == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(PracticeAuthorityV2Error::AuthoritySchemaVersion)
    }
}

pub(crate) fn validate_input_authority_row_v2(
    value: &PracticeInputAuthorityV2,
) -> Result<(), PracticeAuthorityV2Error> {
    validate_schema(value.schema_version)?;
    if value.effective_from_tick >= value.effective_through_tick_exclusive {
        return Err(PracticeAuthorityV2Error::AuthorityEmptyInterval);
    }
    Ok(())
}

fn row_key(value: &PracticeInputAuthorityV2) -> (CampaignIdV2, InputAuthorityIdV2, u64) {
    (
        value.campaign_id,
        value.input_authority_id,
        value.effective_from_tick,
    )
}

fn validate_player_intervals(
    rows: &[PracticeInputAuthorityV2],
) -> Result<(), PracticeAuthorityV2Error> {
    let mut intervals: Vec<(
        CampaignIdV2,
        u64,
        u64,
        InputAuthorityIdV2,
        ActorOrganizationIdV2,
    )> = rows
        .iter()
        .take(MAX_PRACTICE_INPUT_AUTHORITY_ROWS_V2 + 1)
        .filter(|row| row.authority_kind == PracticeAuthorityKindV2::PlayerSeat)
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
    for pair in intervals
        .windows(2)
        .take(MAX_PRACTICE_INPUT_AUTHORITY_ROWS_V2)
    {
        let prior = pair[0];
        let current = pair[1];
        if prior.0 == current.0 && current.1 < prior.2 {
            return Err(PracticeAuthorityV2Error::AuthorityPlayerSeatOverlap);
        }
        if prior.0 == current.0 && (prior.3 != current.3 || prior.4 != current.4) {
            return Err(PracticeAuthorityV2Error::AuthorityPlayerSeatReassignment);
        }
    }
    Ok(())
}

/// Validate bounded canonical order and effective-interval laws.
///
/// # Errors
/// Returns the first exact schema, row, order, duplicate, or overlap refusal.
pub fn validate_input_authority_ledger_v2(
    ledger: &PracticeInputAuthorityLedgerV2,
) -> Result<(), PracticeAuthorityV2Error> {
    validate_schema(ledger.schema_version)?;
    if ledger.rows.len() > MAX_PRACTICE_INPUT_AUTHORITY_ROWS_V2 {
        return Err(PracticeAuthorityV2Error::AuthorityLedgerLimit);
    }
    let mut previous: Option<&PracticeInputAuthorityV2> = None;
    for row in ledger
        .rows
        .iter()
        .take(MAX_PRACTICE_INPUT_AUTHORITY_ROWS_V2 + 1)
    {
        validate_input_authority_row_v2(row)?;
        if let Some(prior) = previous {
            if row_key(prior) == row_key(row) {
                return Err(PracticeAuthorityV2Error::AuthorityLedgerDuplicate);
            }
            if row_key(row) < row_key(prior) {
                return Err(PracticeAuthorityV2Error::AuthorityLedgerOrder);
            }
            if prior.campaign_id == row.campaign_id
                && prior.input_authority_id == row.input_authority_id
                && row.effective_from_tick < prior.effective_through_tick_exclusive
            {
                return Err(PracticeAuthorityV2Error::AuthorityIntervalOverlap);
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

/// Encode one V2 authority row in fixed big-endian order.
///
/// # Errors
/// Returns the exact schema or interval refusal.
pub fn encode_input_authority_v2(
    value: &PracticeInputAuthorityV2,
) -> Result<Vec<u8>, PracticeAuthorityV2Error> {
    validate_input_authority_row_v2(value)?;
    let mut output = Vec::with_capacity(PRACTICE_INPUT_AUTHORITY_V2_CANONICAL_BYTES);
    append_domain(&mut output, PRACTICE_INPUT_AUTHORITY_V2_DOMAIN_BYTES);
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

    fn take(&mut self, count: usize) -> Result<&'a [u8], PracticeAuthorityV2Error> {
        let end = self
            .index
            .checked_add(count)
            .ok_or(PracticeAuthorityV2Error::AuthorityTruncated)?;
        let value = self
            .payload
            .get(self.index..end)
            .ok_or(PracticeAuthorityV2Error::AuthorityTruncated)?;
        self.index = end;
        Ok(value)
    }

    fn domain(&mut self, expected: &[u8]) -> Result<(), PracticeAuthorityV2Error> {
        if self.take(expected.len())? == expected && self.take(1)? == [0] {
            Ok(())
        } else {
            Err(PracticeAuthorityV2Error::AuthorityDomain)
        }
    }

    fn u8(&mut self) -> Result<u8, PracticeAuthorityV2Error> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, PracticeAuthorityV2Error> {
        Ok(u16::from_be_bytes(self.take(2)?.try_into().map_err(
            |_| PracticeAuthorityV2Error::AuthorityTruncated,
        )?))
    }

    fn u32(&mut self) -> Result<u32, PracticeAuthorityV2Error> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().map_err(
            |_| PracticeAuthorityV2Error::AuthorityTruncated,
        )?))
    }

    fn u64(&mut self) -> Result<u64, PracticeAuthorityV2Error> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().map_err(
            |_| PracticeAuthorityV2Error::AuthorityTruncated,
        )?))
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], PracticeAuthorityV2Error> {
        self.take(N)?
            .try_into()
            .map_err(|_| PracticeAuthorityV2Error::AuthorityTruncated)
    }

    fn finish(&self) -> Result<(), PracticeAuthorityV2Error> {
        if self.index == self.payload.len() {
            Ok(())
        } else {
            Err(PracticeAuthorityV2Error::AuthorityTrailingBytes)
        }
    }
}

/// Decode one complete V2 authority row.
///
/// # Errors
/// Returns the first exact domain, schema, enum, interval, truncation, or trailing refusal.
pub fn decode_input_authority_v2(
    payload: &[u8],
) -> Result<PracticeInputAuthorityV2, PracticeAuthorityV2Error> {
    let mut cursor = Cursor::new(payload);
    cursor.domain(PRACTICE_INPUT_AUTHORITY_V2_DOMAIN_BYTES)?;
    let schema_version = cursor.u16()?;
    validate_schema(schema_version)?;
    let campaign_id = CampaignIdV2::from_bytes(cursor.array()?);
    let authority_kind = PracticeAuthorityKindV2::try_from(cursor.u8()?)?;
    let input_authority_id = InputAuthorityIdV2::from_bytes(cursor.array()?);
    let actor_org_id = ActorOrganizationIdV2::from_bytes(cursor.array()?);
    let effective_from_tick = cursor.u64()?;
    let effective_through_tick_exclusive = cursor.u64()?;
    let decision_content_digest = cursor.array()?;
    cursor.finish()?;
    let value = PracticeInputAuthorityV2 {
        schema_version,
        campaign_id,
        authority_kind,
        input_authority_id,
        actor_org_id,
        effective_from_tick,
        effective_through_tick_exclusive,
        decision_content_digest,
    };
    validate_input_authority_row_v2(&value)?;
    Ok(value)
}

/// Encode one complete sorted V2 authority ledger.
///
/// # Errors
/// Returns the first exact ledger or row refusal.
pub fn encode_input_authority_ledger_v2(
    ledger: &PracticeInputAuthorityLedgerV2,
) -> Result<Vec<u8>, PracticeAuthorityV2Error> {
    validate_input_authority_ledger_v2(ledger)?;
    let mut output = Vec::new();
    append_domain(&mut output, PRACTICE_INPUT_AUTHORITY_LEDGER_V2_DOMAIN_BYTES);
    output.extend_from_slice(&ledger.schema_version.to_be_bytes());
    let count = u32::try_from(ledger.rows.len())
        .map_err(|_| PracticeAuthorityV2Error::AuthorityLedgerLimit)?;
    output.extend_from_slice(&count.to_be_bytes());
    for row in ledger
        .rows
        .iter()
        .take(MAX_PRACTICE_INPUT_AUTHORITY_ROWS_V2 + 1)
    {
        output.extend_from_slice(&encode_input_authority_v2(row)?);
    }
    Ok(output)
}

/// Hash one successfully encoded V2 authority row.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn input_authority_v2_digest(
    value: &PracticeInputAuthorityV2,
) -> Result<[u8; 32], PracticeAuthorityV2Error> {
    Ok(sha256_of(&encode_input_authority_v2(value)?))
}

/// Hash one successfully encoded V2 authority ledger.
///
/// # Errors
/// Returns the exact ledger refusal without publishing a digest.
pub fn input_authority_ledger_v2_digest(
    ledger: &PracticeInputAuthorityLedgerV2,
) -> Result<[u8; 32], PracticeAuthorityV2Error> {
    Ok(sha256_of(&encode_input_authority_ledger_v2(ledger)?))
}

/// Decode one complete V2 authority ledger.
///
/// # Errors
/// Returns the first exact domain, row-count, row, order, or trailing refusal.
pub fn decode_input_authority_ledger_v2(
    payload: &[u8],
) -> Result<PracticeInputAuthorityLedgerV2, PracticeAuthorityV2Error> {
    let mut cursor = Cursor::new(payload);
    cursor.domain(PRACTICE_INPUT_AUTHORITY_LEDGER_V2_DOMAIN_BYTES)?;
    let schema_version = cursor.u16()?;
    validate_schema(schema_version)?;
    let count = usize::try_from(cursor.u32()?)
        .map_err(|_| PracticeAuthorityV2Error::AuthorityLedgerLimit)?;
    if count > MAX_PRACTICE_INPUT_AUTHORITY_ROWS_V2 {
        return Err(PracticeAuthorityV2Error::AuthorityLedgerLimit);
    }
    let mut rows = Vec::with_capacity(count);
    for index in 0..=MAX_PRACTICE_INPUT_AUTHORITY_ROWS_V2 {
        if index == count {
            break;
        }
        rows.push(decode_input_authority_v2(
            cursor.take(PRACTICE_INPUT_AUTHORITY_V2_CANONICAL_BYTES)?,
        )?);
    }
    cursor.finish()?;
    let ledger = PracticeInputAuthorityLedgerV2 {
        schema_version,
        rows,
    };
    validate_input_authority_ledger_v2(&ledger)?;
    Ok(ledger)
}

/// Resolve the one active authority row matching an intent identity.
///
/// # Errors
/// Returns an exact malformed-ledger, missing, inactive, or actor mismatch refusal.
pub fn resolve_input_authority_v2(
    ledger: &PracticeInputAuthorityLedgerV2,
    campaign_id: CampaignIdV2,
    input_authority_id: InputAuthorityIdV2,
    actor_org_id: ActorOrganizationIdV2,
    resolve_tick: u64,
) -> Result<&PracticeInputAuthorityV2, PracticeAuthorityV2Error> {
    validate_input_authority_ledger_v2(ledger)?;
    let mut found_identity = false;
    for row in ledger
        .rows
        .iter()
        .take(MAX_PRACTICE_INPUT_AUTHORITY_ROWS_V2 + 1)
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
            return Err(PracticeAuthorityV2Error::AuthorityActorMismatch);
        }
        return Ok(row);
    }
    if found_identity {
        Err(PracticeAuthorityV2Error::AuthorityInactive)
    } else {
        Err(PracticeAuthorityV2Error::AuthorityNotFound)
    }
}

/// Resolve the sole active player habitation row for one campaign tick.
///
/// # Errors
/// Returns an exact malformed-ledger, missing-seat, or overlapping-seat refusal.
pub fn active_player_authority_v2(
    ledger: &PracticeInputAuthorityLedgerV2,
    campaign_id: CampaignIdV2,
    tick: u64,
) -> Result<&PracticeInputAuthorityV2, PracticeAuthorityV2Error> {
    validate_input_authority_ledger_v2(ledger)?;
    let mut found: Option<&PracticeInputAuthorityV2> = None;
    for row in ledger
        .rows
        .iter()
        .take(MAX_PRACTICE_INPUT_AUTHORITY_ROWS_V2 + 1)
    {
        if row.campaign_id == campaign_id
            && row.authority_kind == PracticeAuthorityKindV2::PlayerSeat
            && tick >= row.effective_from_tick
            && tick < row.effective_through_tick_exclusive
        {
            if found.is_some() {
                return Err(PracticeAuthorityV2Error::AuthorityPlayerSeatOverlap);
            }
            found = Some(row);
        }
    }
    found.ok_or(PracticeAuthorityV2Error::AuthorityPlayerSeatMissing)
}
