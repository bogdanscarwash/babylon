//! Pure current practice-proposal identity and authority validation.

use babylon_kernel::content_digest::sha256_of;

use crate::actor::ActorOrganizationId;
use crate::{
    resolve_input_authority, CampaignId, InputAuthorityId, PracticeAuthorityError,
    PracticeInputAuthority, PracticeInputAuthorityLedger,
};

const SCHEMA_VERSION: u16 = 2;

/// Canonical domain for current practice intents.
pub const PRACTICE_INTENT_DOMAIN_BYTES: &[u8] = b"babylon.practice-intent.v2";
/// SHA-256 of the exact language-neutral current intent schema bytes.
pub const PRACTICE_INTENT_SOURCE_SHA256: [u8; 32] = [
    0xed, 0xe3, 0xc5, 0x5e, 0x1f, 0x62, 0xbb, 0x7b, 0xec, 0x0c, 0x4d, 0x89, 0xaa, 0xfa, 0x56, 0x44,
    0x44, 0xfb, 0xbc, 0x0b, 0xa8, 0xc2, 0x75, 0x26, 0xad, 0x00, 0xef, 0xe4, 0xbe, 0x5a, 0x1e, 0xc5,
];
/// Designed bound on parameters in one intent. current's semantic allowlists are empty.
pub const MAX_PRACTICE_PARAMETERS: usize = 16;
/// Designed structural bound for one parameter value.
pub const MAX_PRACTICE_PARAMETER_VALUE_BYTES: usize = 256;
/// Designed bound on sorted unique evidence digests in one intent.
pub const MAX_PRACTICE_EVIDENCE_DIGESTS: usize = 64;
/// Exact canonical byte length of a valid current intent with no evidence digests.
pub const MIN_PRACTICE_INTENT_CANONICAL_BYTES: usize =
    PRACTICE_INTENT_DOMAIN_BYTES.len() + 1 + 2 + 8 + 8 + 16 + 8 + 1 + 1 + 32 + 16 + 32 + 32 + 2 + 2;
/// Designed canonical-byte and decode-fuel ceiling for one intent.
pub const MAX_PRACTICE_INTENT_CANONICAL_BYTES: usize = 16_384;

/// Exact current intent-contract refusals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PracticeIntentError {
    IntentDomain = 1,
    IntentSchemaVersion = 2,
    IntentEnumCode = 3,
    IntentTruncated = 4,
    IntentTrailingBytes = 5,
    IntentLength = 6,
    IntentTickOverflow = 7,
    IntentTickMismatch = 8,
    IntentParameterLimit = 9,
    IntentParameterLength = 10,
    IntentParameterUnsupported = 11,
    IntentEvidenceLimit = 12,
    IntentEvidenceOrder = 13,
    IntentEvidenceDuplicate = 14,
    IntentTargetMismatch = 15,
}

/// Unknown current intent-contract error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownPracticeIntentErrorCode(pub u16);

impl TryFrom<u16> for PracticeIntentError {
    type Error = UnknownPracticeIntentErrorCode;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::IntentDomain),
            2 => Ok(Self::IntentSchemaVersion),
            3 => Ok(Self::IntentEnumCode),
            4 => Ok(Self::IntentTruncated),
            5 => Ok(Self::IntentTrailingBytes),
            6 => Ok(Self::IntentLength),
            7 => Ok(Self::IntentTickOverflow),
            8 => Ok(Self::IntentTickMismatch),
            9 => Ok(Self::IntentParameterLimit),
            10 => Ok(Self::IntentParameterLength),
            11 => Ok(Self::IntentParameterUnsupported),
            12 => Ok(Self::IntentEvidenceLimit),
            13 => Ok(Self::IntentEvidenceOrder),
            14 => Ok(Self::IntentEvidenceDuplicate),
            15 => Ok(Self::IntentTargetMismatch),
            _ => Err(UnknownPracticeIntentErrorCode(value)),
        }
    }
}

impl From<PracticeIntentError> for u16 {
    fn from(value: PracticeIntentError) -> Self {
        value as Self
    }
}

/// Closed current practice identity table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PracticeId {
    Organize = 1,
    Agitate = 2,
    MutualAid = 3,
    Strike = 4,
    Blockade = 5,
    Occupation = 6,
    Damage = 7,
    CapitalStrike = 8,
}

impl TryFrom<u8> for PracticeId {
    type Error = PracticeIntentError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Organize),
            2 => Ok(Self::Agitate),
            3 => Ok(Self::MutualAid),
            4 => Ok(Self::Strike),
            5 => Ok(Self::Blockade),
            6 => Ok(Self::Occupation),
            7 => Ok(Self::Damage),
            8 => Ok(Self::CapitalStrike),
            _ => Err(PracticeIntentError::IntentEnumCode),
        }
    }
}

/// Closed current tagged-target table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PracticeTargetTag {
    SocialClass = 1,
    LaborProcess = 2,
    Route = 3,
    ShipmentClass = 4,
    AccessPoint = 5,
    Facility = 6,
    Territory = 7,
    Stock = 8,
    InvestmentCommitment = 9,
    CreditCommitment = 10,
    ProcurementCommitment = 11,
    ProductionCommitment = 12,
}

impl TryFrom<u8> for PracticeTargetTag {
    type Error = PracticeIntentError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::SocialClass),
            2 => Ok(Self::LaborProcess),
            3 => Ok(Self::Route),
            4 => Ok(Self::ShipmentClass),
            5 => Ok(Self::AccessPoint),
            6 => Ok(Self::Facility),
            7 => Ok(Self::Territory),
            8 => Ok(Self::Stock),
            9 => Ok(Self::InvestmentCommitment),
            10 => Ok(Self::CreditCommitment),
            11 => Ok(Self::ProcurementCommitment),
            12 => Ok(Self::ProductionCommitment),
            _ => Err(PracticeIntentError::IntentEnumCode),
        }
    }
}

/// Stable domain-separated target identity bytes, not a runtime graph ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PracticeTargetIdentity([u8; 32]);

impl PracticeTargetIdentity {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(self) -> [u8; 32] {
        self.0
    }
}

/// Opaque proposal nonce. It distinguishes proposals and grants no priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ProposalNonce([u8; 16]);

impl ProposalNonce {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(self) -> [u8; 16] {
        self.0
    }
}

/// One tagged stable target identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaggedPracticeTarget {
    pub tag: PracticeTargetTag,
    pub identity: PracticeTargetIdentity,
}

/// Structurally framed parameter row. current semantic allowlists are empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeParameter {
    pub key_u8: u8,
    pub value_kind_u8: u8,
    pub value_length_u16: u16,
    pub value_bytes: Vec<u8>,
}

/// One canonical, authority-bound next-week proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeIntent {
    pub schema_version: u16,
    pub submit_after_tick: u64,
    pub resolve_tick: u64,
    pub input_authority_id: InputAuthorityId,
    pub actor_org_id: ActorOrganizationId,
    pub practice_id: PracticeId,
    pub target: TaggedPracticeTarget,
    pub proposal_nonce: ProposalNonce,
    pub quoted_content_digest: [u8; 32],
    pub quoted_resource_contract_digest: [u8; 32],
    pub parameters: Vec<PracticeParameter>,
    pub evidence_digests: Vec<[u8; 32]>,
}

/// Fixed field order for language-neutral implementations.
pub const PRACTICE_INTENT_FIELD_ORDER: [&str; 12] = [
    "schema_version",
    "submit_after_tick",
    "resolve_tick",
    "input_authority_id",
    "actor_org_id",
    "practice_id",
    "tagged_target_identity",
    "proposal_nonce",
    "quoted_content_digest",
    "quoted_resource_contract_digest",
    "parameters",
    "evidence_digests",
];

/// Unique proposal key. Its order is canonical serialization order, not priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PracticeProposalKey {
    pub resolve_tick: u64,
    pub input_authority_id: InputAuthorityId,
    pub actor_org_id: ActorOrganizationId,
    pub practice_id: PracticeId,
    pub target: TaggedPracticeTarget,
    pub proposal_nonce: ProposalNonce,
}

/// Combined intent/authority refusal without losing either closed error identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PracticeIntentAuthorityError {
    Intent(PracticeIntentError),
    Authority(PracticeAuthorityError),
}

fn validate_schema(value: u16) -> Result<(), PracticeIntentError> {
    if value == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(PracticeIntentError::IntentSchemaVersion)
    }
}

fn validate_tick_pair(submit: u64, resolve: u64) -> Result<(), PracticeIntentError> {
    let expected = submit
        .checked_add(1)
        .ok_or(PracticeIntentError::IntentTickOverflow)?;
    if resolve == expected {
        Ok(())
    } else {
        Err(PracticeIntentError::IntentTickMismatch)
    }
}

pub(crate) fn target_is_valid(practice: PracticeId, tag: PracticeTargetTag) -> bool {
    match practice {
        PracticeId::Organize | PracticeId::Agitate | PracticeId::MutualAid => {
            tag == PracticeTargetTag::SocialClass
        }
        PracticeId::Strike => tag == PracticeTargetTag::LaborProcess,
        PracticeId::Blockade => matches!(
            tag,
            PracticeTargetTag::Route
                | PracticeTargetTag::ShipmentClass
                | PracticeTargetTag::AccessPoint
        ),
        PracticeId::Occupation => matches!(
            tag,
            PracticeTargetTag::Facility
                | PracticeTargetTag::Territory
                | PracticeTargetTag::AccessPoint
        ),
        PracticeId::Damage => matches!(tag, PracticeTargetTag::Facility | PracticeTargetTag::Stock),
        PracticeId::CapitalStrike => matches!(
            tag,
            PracticeTargetTag::InvestmentCommitment
                | PracticeTargetTag::CreditCommitment
                | PracticeTargetTag::ProcurementCommitment
                | PracticeTargetTag::ProductionCommitment
        ),
    }
}

fn validate_parameters(value: &PracticeIntent) -> Result<(), PracticeIntentError> {
    if value.parameters.len() > MAX_PRACTICE_PARAMETERS {
        return Err(PracticeIntentError::IntentParameterLimit);
    }
    for parameter in value.parameters.iter().take(MAX_PRACTICE_PARAMETERS + 1) {
        if parameter.value_bytes.len() > MAX_PRACTICE_PARAMETER_VALUE_BYTES
            || parameter.value_bytes.len() != usize::from(parameter.value_length_u16)
        {
            return Err(PracticeIntentError::IntentParameterLength);
        }
    }
    if value.parameters.is_empty() {
        Ok(())
    } else {
        Err(PracticeIntentError::IntentParameterUnsupported)
    }
}

fn validate_evidence(value: &PracticeIntent) -> Result<(), PracticeIntentError> {
    if value.evidence_digests.len() > MAX_PRACTICE_EVIDENCE_DIGESTS {
        return Err(PracticeIntentError::IntentEvidenceLimit);
    }
    let mut previous: Option<&[u8; 32]> = None;
    for digest in value
        .evidence_digests
        .iter()
        .take(MAX_PRACTICE_EVIDENCE_DIGESTS + 1)
    {
        if previous == Some(digest) {
            return Err(PracticeIntentError::IntentEvidenceDuplicate);
        }
        if previous.is_some_and(|prior| digest < prior) {
            return Err(PracticeIntentError::IntentEvidenceOrder);
        }
        previous = Some(digest);
    }
    Ok(())
}

/// Validate one detached current intent without graph or gameplay authority.
///
/// # Errors
/// Returns the first exact schema, tick, target, parameter, or evidence refusal.
pub fn validate_practice_intent(value: &PracticeIntent) -> Result<(), PracticeIntentError> {
    validate_schema(value.schema_version)?;
    validate_tick_pair(value.submit_after_tick, value.resolve_tick)?;
    if !target_is_valid(value.practice_id, value.target.tag) {
        return Err(PracticeIntentError::IntentTargetMismatch);
    }
    validate_parameters(value)?;
    validate_evidence(value)
}

fn append_domain(output: &mut Vec<u8>) {
    output.extend_from_slice(PRACTICE_INTENT_DOMAIN_BYTES);
    output.push(0);
}

/// Encode one current intent in fixed big-endian field order.
///
/// # Errors
/// Returns the first exact validation or canonical-size refusal.
pub fn encode_practice_intent(value: &PracticeIntent) -> Result<Vec<u8>, PracticeIntentError> {
    validate_practice_intent(value)?;
    let mut output = Vec::with_capacity(256);
    append_domain(&mut output);
    output.extend_from_slice(&value.schema_version.to_be_bytes());
    output.extend_from_slice(&value.submit_after_tick.to_be_bytes());
    output.extend_from_slice(&value.resolve_tick.to_be_bytes());
    output.extend_from_slice(&value.input_authority_id.as_bytes());
    output.extend_from_slice(&value.actor_org_id.to_bytes());
    output.push(value.practice_id as u8);
    output.push(value.target.tag as u8);
    output.extend_from_slice(&value.target.identity.as_bytes());
    output.extend_from_slice(&value.proposal_nonce.as_bytes());
    output.extend_from_slice(&value.quoted_content_digest);
    output.extend_from_slice(&value.quoted_resource_contract_digest);
    output.extend_from_slice(&0_u16.to_be_bytes());
    let evidence_count = u16::try_from(value.evidence_digests.len())
        .map_err(|_| PracticeIntentError::IntentEvidenceLimit)?;
    output.extend_from_slice(&evidence_count.to_be_bytes());
    for digest in value
        .evidence_digests
        .iter()
        .take(MAX_PRACTICE_EVIDENCE_DIGESTS + 1)
    {
        output.extend_from_slice(digest);
    }
    if output.len() > MAX_PRACTICE_INTENT_CANONICAL_BYTES {
        return Err(PracticeIntentError::IntentLength);
    }
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

    fn take(&mut self, count: usize) -> Result<&'a [u8], PracticeIntentError> {
        let end = self
            .index
            .checked_add(count)
            .ok_or(PracticeIntentError::IntentTruncated)?;
        let value = self
            .payload
            .get(self.index..end)
            .ok_or(PracticeIntentError::IntentTruncated)?;
        self.index = end;
        Ok(value)
    }

    fn domain(&mut self) -> Result<(), PracticeIntentError> {
        if self.take(PRACTICE_INTENT_DOMAIN_BYTES.len())? == PRACTICE_INTENT_DOMAIN_BYTES
            && self.take(1)? == [0]
        {
            Ok(())
        } else {
            Err(PracticeIntentError::IntentDomain)
        }
    }

    fn u8(&mut self) -> Result<u8, PracticeIntentError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, PracticeIntentError> {
        Ok(u16::from_be_bytes(
            self.take(2)?
                .try_into()
                .map_err(|_| PracticeIntentError::IntentTruncated)?,
        ))
    }

    fn u64(&mut self) -> Result<u64, PracticeIntentError> {
        Ok(u64::from_be_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| PracticeIntentError::IntentTruncated)?,
        ))
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], PracticeIntentError> {
        self.take(N)?
            .try_into()
            .map_err(|_| PracticeIntentError::IntentTruncated)
    }

    fn finish(&self) -> Result<(), PracticeIntentError> {
        if self.index == self.payload.len() {
            Ok(())
        } else {
            Err(PracticeIntentError::IntentTrailingBytes)
        }
    }
}

fn decode_parameters(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<PracticeParameter>, PracticeIntentError> {
    let count = usize::from(cursor.u16()?);
    if count > MAX_PRACTICE_PARAMETERS {
        return Err(PracticeIntentError::IntentParameterLimit);
    }
    for index in 0..=MAX_PRACTICE_PARAMETERS {
        if index == count {
            break;
        }
        let _key = cursor.u8()?;
        let _kind = cursor.u8()?;
        let length = usize::from(cursor.u16()?);
        if length > MAX_PRACTICE_PARAMETER_VALUE_BYTES {
            return Err(PracticeIntentError::IntentParameterLength);
        }
        cursor.take(length)?;
    }
    if count == 0 {
        Ok(Vec::new())
    } else {
        Err(PracticeIntentError::IntentParameterUnsupported)
    }
}

fn decode_evidence(cursor: &mut Cursor<'_>) -> Result<Vec<[u8; 32]>, PracticeIntentError> {
    let count = usize::from(cursor.u16()?);
    if count > MAX_PRACTICE_EVIDENCE_DIGESTS {
        return Err(PracticeIntentError::IntentEvidenceLimit);
    }
    let mut output = Vec::with_capacity(count);
    let mut previous: Option<[u8; 32]> = None;
    for index in 0..=MAX_PRACTICE_EVIDENCE_DIGESTS {
        if index == count {
            break;
        }
        let digest = cursor.array()?;
        if previous == Some(digest) {
            return Err(PracticeIntentError::IntentEvidenceDuplicate);
        }
        if previous.is_some_and(|prior| digest < prior) {
            return Err(PracticeIntentError::IntentEvidenceOrder);
        }
        output.push(digest);
        previous = Some(digest);
    }
    Ok(output)
}

/// Decode one complete current intent.
///
/// # Errors
/// Returns the first exact size, domain, field, canonical-order, or trailing refusal.
pub fn decode_practice_intent(payload: &[u8]) -> Result<PracticeIntent, PracticeIntentError> {
    if payload.len() > MAX_PRACTICE_INTENT_CANONICAL_BYTES {
        return Err(PracticeIntentError::IntentLength);
    }
    let mut cursor = Cursor::new(payload);
    cursor.domain()?;
    let schema_version = cursor.u16()?;
    validate_schema(schema_version)?;
    let submit_after_tick = cursor.u64()?;
    let resolve_tick = cursor.u64()?;
    validate_tick_pair(submit_after_tick, resolve_tick)?;
    let input_authority_id = InputAuthorityId::from_bytes(cursor.array()?);
    let actor_org_id = ActorOrganizationId::from_bytes(cursor.array()?);
    let practice_id = PracticeId::try_from(cursor.u8()?)?;
    let target = TaggedPracticeTarget {
        tag: PracticeTargetTag::try_from(cursor.u8()?)?,
        identity: PracticeTargetIdentity::from_bytes(cursor.array()?),
    };
    let proposal_nonce = ProposalNonce::from_bytes(cursor.array()?);
    let quoted_content_digest = cursor.array()?;
    let quoted_resource_contract_digest = cursor.array()?;
    let parameters = decode_parameters(&mut cursor)?;
    let evidence_digests = decode_evidence(&mut cursor)?;
    cursor.finish()?;
    let value = PracticeIntent {
        schema_version,
        submit_after_tick,
        resolve_tick,
        input_authority_id,
        actor_org_id,
        practice_id,
        target,
        proposal_nonce,
        quoted_content_digest,
        quoted_resource_contract_digest,
        parameters,
        evidence_digests,
    };
    validate_practice_intent(&value)?;
    Ok(value)
}

/// Hash one successfully encoded current intent.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn practice_intent_digest(value: &PracticeIntent) -> Result<[u8; 32], PracticeIntentError> {
    Ok(sha256_of(&encode_practice_intent(value)?))
}

/// Return the complete unique proposal key. Its ordering grants no priority.
#[must_use]
pub const fn practice_proposal_key(value: &PracticeIntent) -> PracticeProposalKey {
    PracticeProposalKey {
        resolve_tick: value.resolve_tick,
        input_authority_id: value.input_authority_id,
        actor_org_id: value.actor_org_id,
        practice_id: value.practice_id,
        target: value.target,
        proposal_nonce: value.proposal_nonce,
    }
}

/// Validate one intent against the authoritative campaign ledger.
///
/// # Errors
/// Preserves the exact current intent or authority refusal that failed.
pub fn validate_practice_intent_authority<'a>(
    ledger: &'a PracticeInputAuthorityLedger,
    campaign_id: CampaignId,
    intent: &PracticeIntent,
) -> Result<&'a PracticeInputAuthority, PracticeIntentAuthorityError> {
    validate_practice_intent(intent).map_err(PracticeIntentAuthorityError::Intent)?;
    resolve_input_authority(
        ledger,
        campaign_id,
        intent.input_authority_id,
        intent.actor_org_id,
        intent.resolve_tick,
    )
    .map_err(PracticeIntentAuthorityError::Authority)
}

/// Hash the canonical parameter bytes after checking the current allowlist.
///
/// # Errors
/// Returns the precise current parameter bound or allowlist refusal.
pub fn practice_parameter_bytes_digest(
    value: &PracticeIntent,
) -> Result<[u8; 32], PracticeIntentError> {
    validate_parameters(value)?;
    // Every current practice has an empty semantic parameter allowlist.
    Ok(sha256_of(b"babylon.practice-parameter-bytes.v1\0\0\0"))
}

/// Hash a fixed selection of a stable, tagged material target.
#[must_use]
pub fn fixed_practice_target_digest(
    tag: PracticeTargetTag,
    identity: PracticeTargetIdentity,
) -> [u8; 32] {
    let mut preimage = Vec::with_capacity(68);
    preimage.extend_from_slice(b"babylon.fixed-target-selection.v2\0");
    preimage.push(tag as u8);
    preimage.extend_from_slice(&identity.as_bytes());
    sha256_of(&preimage)
}
