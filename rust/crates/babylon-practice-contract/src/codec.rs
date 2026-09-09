//! Independent fixed-order codecs for detached T2 practice values.

use babylon_kernel::sha256_of;

use crate::{
    OrganizationBudgetDeltaV1, PracticeAuthorityKindV1, PracticeContractError, PracticeIdV1,
    PracticeInputAuthorityV1, PracticeIntentV1, PracticeRejectionCodeV1,
    PracticeSubmissionRejectionV1, PracticeTargetDomainV1, MAX_EVIDENCE_DIGESTS,
    MAX_INTENT_CANONICAL_BYTES, MAX_PARAMETERS, MAX_PARAMETER_VALUE_BYTES,
    ORGANIZATION_BUDGET_DELTA_V1_DOMAIN_BYTES, PRACTICE_INPUT_AUTHORITY_V1_DOMAIN_BYTES,
    PRACTICE_INTENT_V1_DOMAIN_BYTES, PRACTICE_WIRE_DOMAIN_TERMINATOR_BYTES,
};

const SCHEMA_VERSION: u16 = 1;
const PARAMETER_DIGEST_DOMAIN: &[u8] = b"babylon.practice-parameter-bytes.v1";
const FIXED_TARGET_DOMAIN: &[u8] = b"babylon.fixed-target-selection.v1";

fn append_domain(output: &mut Vec<u8>, domain: &[u8]) {
    output.extend_from_slice(domain);
    output.extend_from_slice(PRACTICE_WIRE_DOMAIN_TERMINATOR_BYTES);
}

fn check_schema_version(value: u16) -> Result<(), PracticeContractError> {
    if value == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(PracticeContractError::PracticeSchemaVersion)
    }
}

fn check_tick_pair(submit_after_tick: u64, resolve_tick: u64) -> Result<(), PracticeContractError> {
    let expected = submit_after_tick
        .checked_add(1)
        .ok_or(PracticeContractError::PracticeTickOverflow)?;
    if resolve_tick == expected {
        Ok(())
    } else {
        Err(PracticeContractError::PracticeTickMismatch)
    }
}

fn check_evidence(value: &PracticeIntentV1) -> Result<(), PracticeContractError> {
    if value.evidence_digests.len() > MAX_EVIDENCE_DIGESTS {
        return Err(PracticeContractError::PracticeEvidenceLimit);
    }
    let mut previous: Option<&[u8; 32]> = None;
    for digest in value.evidence_digests.iter().take(MAX_EVIDENCE_DIGESTS + 1) {
        if previous == Some(digest) {
            return Err(PracticeContractError::PracticeEvidenceDuplicate);
        }
        if previous.is_some_and(|item| digest < item) {
            return Err(PracticeContractError::PracticeEvidenceOrder);
        }
        previous = Some(digest);
    }
    Ok(())
}

/// Encodes one authority in its exact fixed field order.
///
/// # Errors
/// Returns the exact schema-version refusal for an invalid typed value.
pub fn encode_input_authority(
    value: &PracticeInputAuthorityV1,
) -> Result<Vec<u8>, PracticeContractError> {
    check_schema_version(value.schema_version)?;
    let mut output = Vec::new();
    append_domain(&mut output, PRACTICE_INPUT_AUTHORITY_V1_DOMAIN_BYTES);
    output.extend_from_slice(&value.schema_version.to_be_bytes());
    output.push(value.authority_kind as u8);
    output.extend_from_slice(&value.actor_org_id.to_be_bytes());
    output.extend_from_slice(&value.producer_content_digest);
    Ok(output)
}

/// Encodes the exact parameter sequence, whose V1 allowlists are empty.
///
/// # Errors
/// Returns the exact count, structural-length, or unsupported-parameter refusal.
pub fn encode_intent_parameters(
    value: &PracticeIntentV1,
) -> Result<Vec<u8>, PracticeContractError> {
    if value.parameters.len() > MAX_PARAMETERS {
        return Err(PracticeContractError::PracticeParameterLimit);
    }
    let count = u16::try_from(value.parameters.len())
        .map_err(|_| PracticeContractError::PracticeParameterLimit)?;
    let mut saw_parameter = false;
    for parameter in value.parameters.iter().take(MAX_PARAMETERS + 1) {
        saw_parameter = true;
        let actual_length = parameter.value_bytes.len();
        if actual_length > MAX_PARAMETER_VALUE_BYTES
            || actual_length != usize::from(parameter.value_length_u16)
        {
            return Err(PracticeContractError::PracticeParameterLength);
        }
    }
    if saw_parameter {
        return Err(PracticeContractError::PracticeParameter);
    }
    let mut output = Vec::new();
    output.extend_from_slice(&count.to_be_bytes());
    Ok(output)
}

/// Encodes one bounded practice intent in exact big-endian field order.
///
/// # Errors
/// Returns the first exact governed tick, parameter, evidence, or size refusal.
pub fn encode_intent(value: &PracticeIntentV1) -> Result<Vec<u8>, PracticeContractError> {
    check_schema_version(value.schema_version)?;
    check_tick_pair(value.submit_after_tick, value.resolve_tick)?;
    let parameter_bytes = encode_intent_parameters(value)?;
    check_evidence(value)?;
    let evidence_count = u16::try_from(value.evidence_digests.len())
        .map_err(|_| PracticeContractError::PracticeEvidenceLimit)?;
    let mut output = Vec::new();
    append_domain(&mut output, PRACTICE_INTENT_V1_DOMAIN_BYTES);
    output.extend_from_slice(&value.schema_version.to_be_bytes());
    output.extend_from_slice(&value.submit_after_tick.to_be_bytes());
    output.extend_from_slice(&value.resolve_tick.to_be_bytes());
    output.extend_from_slice(&value.actor_org_id.to_be_bytes());
    output.push(value.practice_id as u8);
    output.push(value.target_domain as u8);
    output.extend_from_slice(&value.target_node_id.to_be_bytes());
    output.extend_from_slice(&value.quoted_content_digest);
    output.extend_from_slice(&value.quoted_action_budget_cost.to_be_bytes());
    output.extend_from_slice(&parameter_bytes);
    output.extend_from_slice(&evidence_count.to_be_bytes());
    for digest in value.evidence_digests.iter().take(MAX_EVIDENCE_DIGESTS + 1) {
        output.extend_from_slice(digest);
    }
    if output.len() > MAX_INTENT_CANONICAL_BYTES {
        return Err(PracticeContractError::PracticeLength);
    }
    Ok(output)
}

/// Encodes one fixed organization-budget delta.
///
/// # Errors
/// Returns the exact schema-version refusal for an invalid typed value.
pub fn encode_budget_delta(
    value: &OrganizationBudgetDeltaV1,
) -> Result<Vec<u8>, PracticeContractError> {
    check_schema_version(value.schema_version)?;
    let mut output = Vec::new();
    append_domain(&mut output, ORGANIZATION_BUDGET_DELTA_V1_DOMAIN_BYTES);
    output.extend_from_slice(&value.schema_version.to_be_bytes());
    output.extend_from_slice(&value.tick.to_be_bytes());
    output.extend_from_slice(&value.actor_node_id.to_be_bytes());
    output.extend_from_slice(&value.pre_action_world_hash);
    for field in [
        value.budget_before,
        value.governed_cost,
        value.footprint_count,
        value.raw_credit,
        value.credited_credit,
    ] {
        output.extend_from_slice(&field.to_be_bytes());
    }
    output.push(u8::from(value.ceiling_bound));
    output.extend_from_slice(&value.budget_after.to_be_bytes());
    Ok(output)
}

/// Encodes one context-complete submission rejection.
///
/// # Errors
/// Returns the exact schema-version refusal for an invalid typed value.
pub fn encode_rejection(
    value: &PracticeSubmissionRejectionV1,
) -> Result<Vec<u8>, PracticeContractError> {
    check_schema_version(value.schema_version)?;
    let mut output = Vec::with_capacity(76);
    output.extend_from_slice(&value.schema_version.to_be_bytes());
    output.extend_from_slice(&value.submitted_bytes_digest);
    output.extend_from_slice(&(value.reason_code as u16).to_be_bytes());
    output.extend_from_slice(&value.last_committed_tick.to_be_bytes());
    output.extend_from_slice(&value.content_digest);
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

    fn take(&mut self, count: usize) -> Result<&'a [u8], PracticeContractError> {
        let end = self
            .index
            .checked_add(count)
            .ok_or(PracticeContractError::PracticeTruncated)?;
        let value = self
            .payload
            .get(self.index..end)
            .ok_or(PracticeContractError::PracticeTruncated)?;
        self.index = end;
        Ok(value)
    }

    fn domain(&mut self, expected: &[u8]) -> Result<(), PracticeContractError> {
        if self.take(expected.len())? == expected {
            Ok(())
        } else {
            Err(PracticeContractError::PracticeDomain)
        }
    }

    fn u8(&mut self) -> Result<u8, PracticeContractError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, PracticeContractError> {
        Ok(u16::from_be_bytes(
            self.take(2)?
                .try_into()
                .map_err(|_| PracticeContractError::PracticeTruncated)?,
        ))
    }

    fn u32(&mut self) -> Result<u32, PracticeContractError> {
        Ok(u32::from_be_bytes(
            self.take(4)?
                .try_into()
                .map_err(|_| PracticeContractError::PracticeTruncated)?,
        ))
    }

    fn u64(&mut self) -> Result<u64, PracticeContractError> {
        Ok(u64::from_be_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| PracticeContractError::PracticeTruncated)?,
        ))
    }

    fn digest(&mut self) -> Result<[u8; 32], PracticeContractError> {
        self.take(32)?
            .try_into()
            .map_err(|_| PracticeContractError::PracticeTruncated)
    }

    fn finish(&self) -> Result<(), PracticeContractError> {
        if self.index == self.payload.len() {
            Ok(())
        } else {
            Err(PracticeContractError::PracticeTrailingBytes)
        }
    }
}

fn framed_domain(domain: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    append_domain(&mut output, domain);
    output
}

/// Decodes one authority and refuses any partial or trailing representation.
///
/// # Errors
/// Returns the first exact domain, truncation, schema, enum, or trailing refusal.
pub fn decode_input_authority(
    payload: &[u8],
) -> Result<PracticeInputAuthorityV1, PracticeContractError> {
    let mut cursor = Cursor::new(payload);
    cursor.domain(&framed_domain(PRACTICE_INPUT_AUTHORITY_V1_DOMAIN_BYTES))?;
    let schema_version = cursor.u16()?;
    check_schema_version(schema_version)?;
    let authority_kind = PracticeAuthorityKindV1::try_from(cursor.u8()?)?;
    let actor_org_id = cursor.u64()?;
    let producer_content_digest = cursor.digest()?;
    cursor.finish()?;
    Ok(PracticeInputAuthorityV1 {
        schema_version,
        authority_kind,
        actor_org_id,
        producer_content_digest,
    })
}

fn decode_parameters(cursor: &mut Cursor<'_>) -> Result<(), PracticeContractError> {
    let count = usize::from(cursor.u16()?);
    if count > MAX_PARAMETERS {
        return Err(PracticeContractError::PracticeParameterLimit);
    }
    for index in 0..=MAX_PARAMETERS {
        if index == count {
            break;
        }
        let _key = cursor.u8()?;
        let _kind = cursor.u8()?;
        let length = usize::from(cursor.u16()?);
        if length > MAX_PARAMETER_VALUE_BYTES {
            return Err(PracticeContractError::PracticeParameterLength);
        }
        cursor.take(length)?;
    }
    if count == 0 {
        Ok(())
    } else {
        Err(PracticeContractError::PracticeParameter)
    }
}

fn decode_evidence(cursor: &mut Cursor<'_>) -> Result<Vec<[u8; 32]>, PracticeContractError> {
    let count = usize::from(cursor.u16()?);
    if count > MAX_EVIDENCE_DIGESTS {
        return Err(PracticeContractError::PracticeEvidenceLimit);
    }
    let mut output = Vec::with_capacity(count);
    let mut previous: Option<[u8; 32]> = None;
    for index in 0..=MAX_EVIDENCE_DIGESTS {
        if index == count {
            break;
        }
        let digest = cursor.digest()?;
        if previous == Some(digest) {
            return Err(PracticeContractError::PracticeEvidenceDuplicate);
        }
        if previous.is_some_and(|item| digest < item) {
            return Err(PracticeContractError::PracticeEvidenceOrder);
        }
        output.push(digest);
        previous = Some(digest);
    }
    Ok(output)
}

/// Decodes one bounded intent and requires complete byte consumption.
///
/// # Errors
/// Returns the first exact governed refusal and never a partial value.
pub fn decode_intent(payload: &[u8]) -> Result<PracticeIntentV1, PracticeContractError> {
    if payload.len() > MAX_INTENT_CANONICAL_BYTES {
        return Err(PracticeContractError::PracticeLength);
    }
    let mut cursor = Cursor::new(payload);
    cursor.domain(&framed_domain(PRACTICE_INTENT_V1_DOMAIN_BYTES))?;
    let schema_version = cursor.u16()?;
    check_schema_version(schema_version)?;
    let submit_after_tick = cursor.u64()?;
    let resolve_tick = cursor.u64()?;
    check_tick_pair(submit_after_tick, resolve_tick)?;
    let actor_org_id = cursor.u64()?;
    let practice_id = PracticeIdV1::try_from(cursor.u8()?)?;
    let target_domain = PracticeTargetDomainV1::try_from(cursor.u8()?)?;
    let target_node_id = cursor.u64()?;
    let quoted_content_digest = cursor.digest()?;
    let quoted_action_budget_cost = cursor.u32()?;
    decode_parameters(&mut cursor)?;
    let evidence_digests = decode_evidence(&mut cursor)?;
    cursor.finish()?;
    Ok(PracticeIntentV1 {
        schema_version,
        submit_after_tick,
        resolve_tick,
        actor_org_id,
        practice_id,
        target_domain,
        target_node_id,
        quoted_content_digest,
        quoted_action_budget_cost,
        parameters: Vec::new(),
        evidence_digests,
    })
}

/// Decodes one fixed organization-budget delta.
///
/// # Errors
/// Returns the first exact governed refusal and never a partial value.
pub fn decode_budget_delta(
    payload: &[u8],
) -> Result<OrganizationBudgetDeltaV1, PracticeContractError> {
    let mut cursor = Cursor::new(payload);
    cursor.domain(&framed_domain(ORGANIZATION_BUDGET_DELTA_V1_DOMAIN_BYTES))?;
    let schema_version = cursor.u16()?;
    check_schema_version(schema_version)?;
    let tick = cursor.u64()?;
    let actor_node_id = cursor.u64()?;
    let pre_action_world_hash = cursor.digest()?;
    let budget_before = cursor.u32()?;
    let governed_cost = cursor.u32()?;
    let footprint_count = cursor.u32()?;
    let raw_credit = cursor.u32()?;
    let credited_credit = cursor.u32()?;
    let ceiling_bound = match cursor.u8()? {
        0 => false,
        1 => true,
        _ => return Err(PracticeContractError::PracticeBoolean),
    };
    let budget_after = cursor.u32()?;
    cursor.finish()?;
    Ok(OrganizationBudgetDeltaV1 {
        schema_version,
        tick,
        actor_node_id,
        pre_action_world_hash,
        budget_before,
        governed_cost,
        footprint_count,
        raw_credit,
        credited_credit,
        ceiling_bound,
        budget_after,
    })
}

/// Decodes one context-complete rejection.
///
/// # Errors
/// Returns the first exact governed refusal and never a partial value.
pub fn decode_rejection(
    payload: &[u8],
) -> Result<PracticeSubmissionRejectionV1, PracticeContractError> {
    let mut cursor = Cursor::new(payload);
    let schema_version = cursor.u16()?;
    check_schema_version(schema_version)?;
    let submitted_bytes_digest = cursor.digest()?;
    let reason_code = PracticeRejectionCodeV1::try_from(cursor.u16()?)?;
    let last_committed_tick = cursor.u64()?;
    let content_digest = cursor.digest()?;
    cursor.finish()?;
    Ok(PracticeSubmissionRejectionV1 {
        schema_version,
        submitted_bytes_digest,
        reason_code,
        last_committed_tick,
        content_digest,
    })
}

/// Hashes one successfully encoded authority.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn input_authority_digest(
    value: &PracticeInputAuthorityV1,
) -> Result<[u8; 32], PracticeContractError> {
    Ok(sha256_of(&encode_input_authority(value)?))
}

/// Hashes one successfully encoded intent.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn intent_digest(value: &PracticeIntentV1) -> Result<[u8; 32], PracticeContractError> {
    Ok(sha256_of(&encode_intent(value)?))
}

/// Hashes the exact domain-separated parameter bytes.
///
/// # Errors
/// Returns the exact parameter refusal without publishing a digest.
pub fn parameter_bytes_digest(value: &PracticeIntentV1) -> Result<[u8; 32], PracticeContractError> {
    let bytes = encode_intent_parameters(value)?;
    let mut preimage = Vec::new();
    append_domain(&mut preimage, PARAMETER_DIGEST_DOMAIN);
    preimage.extend_from_slice(&bytes);
    Ok(sha256_of(&preimage))
}

/// Hashes the sole closed fixed-target framing.
#[must_use]
pub fn target_selection_policy_digest(
    target_domain: PracticeTargetDomainV1,
    target_node_id: u64,
) -> [u8; 32] {
    let mut preimage = Vec::new();
    append_domain(&mut preimage, FIXED_TARGET_DOMAIN);
    preimage.push(target_domain as u8);
    preimage.extend_from_slice(&target_node_id.to_be_bytes());
    sha256_of(&preimage)
}

/// Hashes one successfully encoded budget delta.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn budget_delta_digest(
    value: &OrganizationBudgetDeltaV1,
) -> Result<[u8; 32], PracticeContractError> {
    Ok(sha256_of(&encode_budget_delta(value)?))
}

/// Returns the exact metadata alias without constructing a rejection.
#[must_use]
pub const fn submission_rejection_alias(
    error: PracticeContractError,
) -> Option<PracticeRejectionCodeV1> {
    match error {
        PracticeContractError::PracticeTickOverflow
        | PracticeContractError::PracticeTickMismatch => {
            Some(PracticeRejectionCodeV1::PracticeTickMismatch)
        }
        PracticeContractError::PracticeAuthorityUnregistered
        | PracticeContractError::PracticeAuthorityContentMismatch => {
            Some(PracticeRejectionCodeV1::PracticeAuthorityUnregistered)
        }
        PracticeContractError::PracticeActorMismatch => {
            Some(PracticeRejectionCodeV1::PracticeActorMismatch)
        }
        PracticeContractError::PracticeQuoteContentMismatch => {
            Some(PracticeRejectionCodeV1::PracticeStaleContent)
        }
        PracticeContractError::PracticeQuoteCostMismatch => {
            Some(PracticeRejectionCodeV1::PracticeCostMismatch)
        }
        PracticeContractError::PracticeBatchLimit => {
            Some(PracticeRejectionCodeV1::PracticeBatchLimit)
        }
        PracticeContractError::PracticeDuplicateActor => {
            Some(PracticeRejectionCodeV1::PracticeDuplicateActor)
        }
        PracticeContractError::PracticeBudgetInsufficient => {
            Some(PracticeRejectionCodeV1::PracticeBudgetInsufficient)
        }
        _ => None,
    }
}

/// Constructs one context-complete rejection from exact typed values.
#[must_use]
pub const fn rejection_for(
    submitted_bytes_digest: [u8; 32],
    reason_code: PracticeRejectionCodeV1,
    last_committed_tick: u64,
    content_digest: [u8; 32],
) -> PracticeSubmissionRejectionV1 {
    PracticeSubmissionRejectionV1 {
        schema_version: SCHEMA_VERSION,
        submitted_bytes_digest,
        reason_code,
        last_committed_tick,
        content_digest,
    }
}

impl PracticeIntentV1 {
    /// Returns the fixed resolve tick.
    #[must_use]
    pub const fn resolve_tick(&self) -> u64 {
        self.resolve_tick
    }
    /// Returns the closed practice identity.
    #[must_use]
    pub const fn practice_id(&self) -> PracticeIdV1 {
        self.practice_id
    }
    /// Returns the closed target domain.
    #[must_use]
    pub const fn target_domain(&self) -> PracticeTargetDomainV1 {
        self.target_domain
    }
    /// Returns the fixed target node identity.
    #[must_use]
    pub const fn target_node_id(&self) -> u64 {
        self.target_node_id
    }
    /// Returns the quoted governed action-budget cost.
    #[must_use]
    pub const fn quoted_action_budget_cost(&self) -> u32 {
        self.quoted_action_budget_cost
    }
}
