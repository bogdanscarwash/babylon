//! Pure V2 practice-proposal identity and authority validation.

use babylon_kernel::sha256_of;

use crate::actor_v2::ActorOrganizationIdV2;
use crate::{
    resolve_input_authority_v2, CampaignIdV2, InputAuthorityIdV2, PracticeAuthorityV2Error,
    PracticeInputAuthorityLedgerV2, PracticeInputAuthorityV2,
};

const SCHEMA_VERSION: u16 = 2;

/// Canonical domain for V2 practice intents.
pub const PRACTICE_INTENT_V2_DOMAIN_BYTES: &[u8] = b"babylon.practice-intent.v2";
/// SHA-256 of the exact language-neutral V2 intent schema bytes.
pub const PRACTICE_INTENT_V2_SOURCE_SHA256: [u8; 32] = [
    0xc9, 0xca, 0xdc, 0x34, 0xc7, 0x05, 0x7a, 0x6d, 0x1e, 0x5d, 0x0b, 0x32, 0x5a, 0xa6, 0x69, 0xf8,
    0xa2, 0x79, 0x3f, 0x69, 0x52, 0xc6, 0x3a, 0xac, 0x3f, 0xea, 0xd6, 0xb6, 0xd2, 0x72, 0x3e, 0x1b,
];
/// Designed bound on parameters in one intent. V2's semantic allowlists are empty.
pub const MAX_PRACTICE_PARAMETERS_V2: usize = 16;
/// Designed structural bound for one parameter value.
pub const MAX_PRACTICE_PARAMETER_VALUE_BYTES_V2: usize = 256;
/// Designed bound on sorted unique evidence digests in one intent.
pub const MAX_PRACTICE_EVIDENCE_DIGESTS_V2: usize = 64;
/// Exact canonical byte length of a valid V2 intent with no evidence digests.
pub const MIN_PRACTICE_INTENT_CANONICAL_BYTES_V2: usize = PRACTICE_INTENT_V2_DOMAIN_BYTES.len()
    + 1
    + 2
    + 8
    + 8
    + 16
    + 8
    + 1
    + 1
    + 32
    + 16
    + 32
    + 32
    + 2
    + 2;
/// Designed canonical-byte and decode-fuel ceiling for one intent.
pub const MAX_PRACTICE_INTENT_CANONICAL_BYTES_V2: usize = 16_384;

/// Exact V2 intent-contract refusals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PracticeIntentV2Error {
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

/// Unknown V2 intent-contract error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownPracticeIntentV2ErrorCode(pub u16);

impl TryFrom<u16> for PracticeIntentV2Error {
    type Error = UnknownPracticeIntentV2ErrorCode;

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
            _ => Err(UnknownPracticeIntentV2ErrorCode(value)),
        }
    }
}

impl From<PracticeIntentV2Error> for u16 {
    fn from(value: PracticeIntentV2Error) -> Self {
        value as Self
    }
}

/// Closed V2 practice identity table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PracticeIdV2 {
    Organize = 1,
    Agitate = 2,
    MutualAid = 3,
    Strike = 4,
    Blockade = 5,
    Occupation = 6,
    Damage = 7,
    CapitalStrike = 8,
}

impl TryFrom<u8> for PracticeIdV2 {
    type Error = PracticeIntentV2Error;

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
            _ => Err(PracticeIntentV2Error::IntentEnumCode),
        }
    }
}

/// Closed V2 tagged-target table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PracticeTargetTagV2 {
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

impl TryFrom<u8> for PracticeTargetTagV2 {
    type Error = PracticeIntentV2Error;

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
            _ => Err(PracticeIntentV2Error::IntentEnumCode),
        }
    }
}

/// Stable domain-separated target identity bytes, not a runtime graph ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PracticeTargetIdentityV2([u8; 32]);

impl PracticeTargetIdentityV2 {
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
pub struct ProposalNonceV2([u8; 16]);

impl ProposalNonceV2 {
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
pub struct TaggedPracticeTargetV2 {
    pub tag: PracticeTargetTagV2,
    pub identity: PracticeTargetIdentityV2,
}

/// Structurally framed parameter row. V2 semantic allowlists are empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeParameterV2 {
    pub key_u8: u8,
    pub value_kind_u8: u8,
    pub value_length_u16: u16,
    pub value_bytes: Vec<u8>,
}

/// One canonical, authority-bound next-week proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeIntentV2 {
    pub schema_version: u16,
    pub submit_after_tick: u64,
    pub resolve_tick: u64,
    pub input_authority_id: InputAuthorityIdV2,
    pub actor_org_id: ActorOrganizationIdV2,
    pub practice_id: PracticeIdV2,
    pub target: TaggedPracticeTargetV2,
    pub proposal_nonce: ProposalNonceV2,
    pub quoted_content_digest: [u8; 32],
    pub quoted_resource_contract_digest: [u8; 32],
    pub parameters: Vec<PracticeParameterV2>,
    pub evidence_digests: Vec<[u8; 32]>,
}

/// Fixed field order for language-neutral implementations.
pub const PRACTICE_INTENT_V2_FIELD_ORDER: [&str; 12] = [
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
pub struct PracticeProposalKeyV2 {
    pub resolve_tick: u64,
    pub input_authority_id: InputAuthorityIdV2,
    pub actor_org_id: ActorOrganizationIdV2,
    pub practice_id: PracticeIdV2,
    pub target: TaggedPracticeTargetV2,
    pub proposal_nonce: ProposalNonceV2,
}

/// Combined intent/authority refusal without losing either closed error identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PracticeIntentAuthorityV2Error {
    Intent(PracticeIntentV2Error),
    Authority(PracticeAuthorityV2Error),
}

fn validate_schema(value: u16) -> Result<(), PracticeIntentV2Error> {
    if value == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(PracticeIntentV2Error::IntentSchemaVersion)
    }
}

fn validate_tick_pair(submit: u64, resolve: u64) -> Result<(), PracticeIntentV2Error> {
    let expected = submit
        .checked_add(1)
        .ok_or(PracticeIntentV2Error::IntentTickOverflow)?;
    if resolve == expected {
        Ok(())
    } else {
        Err(PracticeIntentV2Error::IntentTickMismatch)
    }
}

pub(crate) fn target_is_valid(practice: PracticeIdV2, tag: PracticeTargetTagV2) -> bool {
    match practice {
        PracticeIdV2::Organize | PracticeIdV2::Agitate | PracticeIdV2::MutualAid => {
            tag == PracticeTargetTagV2::SocialClass
        }
        PracticeIdV2::Strike => tag == PracticeTargetTagV2::LaborProcess,
        PracticeIdV2::Blockade => matches!(
            tag,
            PracticeTargetTagV2::Route
                | PracticeTargetTagV2::ShipmentClass
                | PracticeTargetTagV2::AccessPoint
        ),
        PracticeIdV2::Occupation => matches!(
            tag,
            PracticeTargetTagV2::Facility
                | PracticeTargetTagV2::Territory
                | PracticeTargetTagV2::AccessPoint
        ),
        PracticeIdV2::Damage => matches!(
            tag,
            PracticeTargetTagV2::Facility | PracticeTargetTagV2::Stock
        ),
        PracticeIdV2::CapitalStrike => matches!(
            tag,
            PracticeTargetTagV2::InvestmentCommitment
                | PracticeTargetTagV2::CreditCommitment
                | PracticeTargetTagV2::ProcurementCommitment
                | PracticeTargetTagV2::ProductionCommitment
        ),
    }
}

fn validate_parameters(value: &PracticeIntentV2) -> Result<(), PracticeIntentV2Error> {
    if value.parameters.len() > MAX_PRACTICE_PARAMETERS_V2 {
        return Err(PracticeIntentV2Error::IntentParameterLimit);
    }
    for parameter in value.parameters.iter().take(MAX_PRACTICE_PARAMETERS_V2 + 1) {
        if parameter.value_bytes.len() > MAX_PRACTICE_PARAMETER_VALUE_BYTES_V2
            || parameter.value_bytes.len() != usize::from(parameter.value_length_u16)
        {
            return Err(PracticeIntentV2Error::IntentParameterLength);
        }
    }
    if value.parameters.is_empty() {
        Ok(())
    } else {
        Err(PracticeIntentV2Error::IntentParameterUnsupported)
    }
}

fn validate_evidence(value: &PracticeIntentV2) -> Result<(), PracticeIntentV2Error> {
    if value.evidence_digests.len() > MAX_PRACTICE_EVIDENCE_DIGESTS_V2 {
        return Err(PracticeIntentV2Error::IntentEvidenceLimit);
    }
    let mut previous: Option<&[u8; 32]> = None;
    for digest in value
        .evidence_digests
        .iter()
        .take(MAX_PRACTICE_EVIDENCE_DIGESTS_V2 + 1)
    {
        if previous == Some(digest) {
            return Err(PracticeIntentV2Error::IntentEvidenceDuplicate);
        }
        if previous.is_some_and(|prior| digest < prior) {
            return Err(PracticeIntentV2Error::IntentEvidenceOrder);
        }
        previous = Some(digest);
    }
    Ok(())
}

/// Validate one detached V2 intent without graph or gameplay authority.
///
/// # Errors
/// Returns the first exact schema, tick, target, parameter, or evidence refusal.
pub fn validate_practice_intent_v2(value: &PracticeIntentV2) -> Result<(), PracticeIntentV2Error> {
    validate_schema(value.schema_version)?;
    validate_tick_pair(value.submit_after_tick, value.resolve_tick)?;
    if !target_is_valid(value.practice_id, value.target.tag) {
        return Err(PracticeIntentV2Error::IntentTargetMismatch);
    }
    validate_parameters(value)?;
    validate_evidence(value)
}

fn append_domain(output: &mut Vec<u8>) {
    output.extend_from_slice(PRACTICE_INTENT_V2_DOMAIN_BYTES);
    output.push(0);
}

/// Encode one V2 intent in fixed big-endian field order.
///
/// # Errors
/// Returns the first exact validation or canonical-size refusal.
pub fn encode_practice_intent_v2(
    value: &PracticeIntentV2,
) -> Result<Vec<u8>, PracticeIntentV2Error> {
    validate_practice_intent_v2(value)?;
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
        .map_err(|_| PracticeIntentV2Error::IntentEvidenceLimit)?;
    output.extend_from_slice(&evidence_count.to_be_bytes());
    for digest in value
        .evidence_digests
        .iter()
        .take(MAX_PRACTICE_EVIDENCE_DIGESTS_V2 + 1)
    {
        output.extend_from_slice(digest);
    }
    if output.len() > MAX_PRACTICE_INTENT_CANONICAL_BYTES_V2 {
        return Err(PracticeIntentV2Error::IntentLength);
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

    fn take(&mut self, count: usize) -> Result<&'a [u8], PracticeIntentV2Error> {
        let end = self
            .index
            .checked_add(count)
            .ok_or(PracticeIntentV2Error::IntentTruncated)?;
        let value = self
            .payload
            .get(self.index..end)
            .ok_or(PracticeIntentV2Error::IntentTruncated)?;
        self.index = end;
        Ok(value)
    }

    fn domain(&mut self) -> Result<(), PracticeIntentV2Error> {
        if self.take(PRACTICE_INTENT_V2_DOMAIN_BYTES.len())? == PRACTICE_INTENT_V2_DOMAIN_BYTES
            && self.take(1)? == [0]
        {
            Ok(())
        } else {
            Err(PracticeIntentV2Error::IntentDomain)
        }
    }

    fn u8(&mut self) -> Result<u8, PracticeIntentV2Error> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, PracticeIntentV2Error> {
        Ok(u16::from_be_bytes(
            self.take(2)?
                .try_into()
                .map_err(|_| PracticeIntentV2Error::IntentTruncated)?,
        ))
    }

    fn u64(&mut self) -> Result<u64, PracticeIntentV2Error> {
        Ok(u64::from_be_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| PracticeIntentV2Error::IntentTruncated)?,
        ))
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], PracticeIntentV2Error> {
        self.take(N)?
            .try_into()
            .map_err(|_| PracticeIntentV2Error::IntentTruncated)
    }

    fn finish(&self) -> Result<(), PracticeIntentV2Error> {
        if self.index == self.payload.len() {
            Ok(())
        } else {
            Err(PracticeIntentV2Error::IntentTrailingBytes)
        }
    }
}

fn decode_parameters(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<PracticeParameterV2>, PracticeIntentV2Error> {
    let count = usize::from(cursor.u16()?);
    if count > MAX_PRACTICE_PARAMETERS_V2 {
        return Err(PracticeIntentV2Error::IntentParameterLimit);
    }
    for index in 0..=MAX_PRACTICE_PARAMETERS_V2 {
        if index == count {
            break;
        }
        let _key = cursor.u8()?;
        let _kind = cursor.u8()?;
        let length = usize::from(cursor.u16()?);
        if length > MAX_PRACTICE_PARAMETER_VALUE_BYTES_V2 {
            return Err(PracticeIntentV2Error::IntentParameterLength);
        }
        cursor.take(length)?;
    }
    if count == 0 {
        Ok(Vec::new())
    } else {
        Err(PracticeIntentV2Error::IntentParameterUnsupported)
    }
}

fn decode_evidence(cursor: &mut Cursor<'_>) -> Result<Vec<[u8; 32]>, PracticeIntentV2Error> {
    let count = usize::from(cursor.u16()?);
    if count > MAX_PRACTICE_EVIDENCE_DIGESTS_V2 {
        return Err(PracticeIntentV2Error::IntentEvidenceLimit);
    }
    let mut output = Vec::with_capacity(count);
    let mut previous: Option<[u8; 32]> = None;
    for index in 0..=MAX_PRACTICE_EVIDENCE_DIGESTS_V2 {
        if index == count {
            break;
        }
        let digest = cursor.array()?;
        if previous == Some(digest) {
            return Err(PracticeIntentV2Error::IntentEvidenceDuplicate);
        }
        if previous.is_some_and(|prior| digest < prior) {
            return Err(PracticeIntentV2Error::IntentEvidenceOrder);
        }
        output.push(digest);
        previous = Some(digest);
    }
    Ok(output)
}

/// Decode one complete V2 intent.
///
/// # Errors
/// Returns the first exact size, domain, field, canonical-order, or trailing refusal.
pub fn decode_practice_intent_v2(
    payload: &[u8],
) -> Result<PracticeIntentV2, PracticeIntentV2Error> {
    if payload.len() > MAX_PRACTICE_INTENT_CANONICAL_BYTES_V2 {
        return Err(PracticeIntentV2Error::IntentLength);
    }
    let mut cursor = Cursor::new(payload);
    cursor.domain()?;
    let schema_version = cursor.u16()?;
    validate_schema(schema_version)?;
    let submit_after_tick = cursor.u64()?;
    let resolve_tick = cursor.u64()?;
    validate_tick_pair(submit_after_tick, resolve_tick)?;
    let input_authority_id = InputAuthorityIdV2::from_bytes(cursor.array()?);
    let actor_org_id = ActorOrganizationIdV2::from_bytes(cursor.array()?);
    let practice_id = PracticeIdV2::try_from(cursor.u8()?)?;
    let target = TaggedPracticeTargetV2 {
        tag: PracticeTargetTagV2::try_from(cursor.u8()?)?,
        identity: PracticeTargetIdentityV2::from_bytes(cursor.array()?),
    };
    let proposal_nonce = ProposalNonceV2::from_bytes(cursor.array()?);
    let quoted_content_digest = cursor.array()?;
    let quoted_resource_contract_digest = cursor.array()?;
    let parameters = decode_parameters(&mut cursor)?;
    let evidence_digests = decode_evidence(&mut cursor)?;
    cursor.finish()?;
    let value = PracticeIntentV2 {
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
    validate_practice_intent_v2(&value)?;
    Ok(value)
}

/// Hash one successfully encoded V2 intent.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn practice_intent_v2_digest(
    value: &PracticeIntentV2,
) -> Result<[u8; 32], PracticeIntentV2Error> {
    Ok(sha256_of(&encode_practice_intent_v2(value)?))
}

/// Return the complete unique proposal key. Its ordering grants no priority.
#[must_use]
pub const fn practice_proposal_key_v2(value: &PracticeIntentV2) -> PracticeProposalKeyV2 {
    PracticeProposalKeyV2 {
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
/// Preserves the exact V2 intent or authority refusal that failed.
pub fn validate_practice_intent_authority_v2<'a>(
    ledger: &'a PracticeInputAuthorityLedgerV2,
    campaign_id: CampaignIdV2,
    intent: &PracticeIntentV2,
) -> Result<&'a PracticeInputAuthorityV2, PracticeIntentAuthorityV2Error> {
    validate_practice_intent_v2(intent).map_err(PracticeIntentAuthorityV2Error::Intent)?;
    resolve_input_authority_v2(
        ledger,
        campaign_id,
        intent.input_authority_id,
        intent.actor_org_id,
        intent.resolve_tick,
    )
    .map_err(PracticeIntentAuthorityV2Error::Authority)
}
