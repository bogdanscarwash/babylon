//! Fixed-order codec for detached organization-budget receipts.
use crate::{
    OrganizationBudgetDelta, PracticeContractError, ORGANIZATION_BUDGET_DELTA_DOMAIN_BYTES,
    PRACTICE_WIRE_DOMAIN_TERMINATOR_BYTES,
};
use babylon_kernel::content_digest::sha256_of;
const SCHEMA_VERSION: u16 = 1;
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

/// Encodes one fixed organization-budget delta.
///
/// # Errors
/// Returns the exact schema-version refusal for an invalid typed value.
pub fn encode_budget_delta(
    value: &OrganizationBudgetDelta,
) -> Result<Vec<u8>, PracticeContractError> {
    check_schema_version(value.schema_version)?;
    let mut output = Vec::new();
    append_domain(&mut output, ORGANIZATION_BUDGET_DELTA_DOMAIN_BYTES);
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

pub fn decode_budget_delta(
    payload: &[u8],
) -> Result<OrganizationBudgetDelta, PracticeContractError> {
    let mut cursor = Cursor::new(payload);
    cursor.domain(&framed_domain(ORGANIZATION_BUDGET_DELTA_DOMAIN_BYTES))?;
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
    Ok(OrganizationBudgetDelta {
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

/// Hashes one successfully encoded budget delta.
///
/// # Errors
/// Returns the exact encoding refusal without publishing a digest.
pub fn budget_delta_digest(
    value: &OrganizationBudgetDelta,
) -> Result<[u8; 32], PracticeContractError> {
    Ok(sha256_of(&encode_budget_delta(value)?))
}
