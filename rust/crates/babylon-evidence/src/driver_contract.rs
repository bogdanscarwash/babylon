//! Exact source-bound contract for the fixture-only synthetic driver.

use babylon_kernel::sha256_of;
use babylon_practice_contract::PracticeIntentV1;

use crate::driver;
use crate::{
    DifferingLedgerKindV1, Digest32, InterventionDeltaV1, PersistenceComparisonV1,
    PracticeAttemptLedgerV1, PracticeCandidateScheduleV1, RunIdentityV1, SfsPreregistrationV1,
    SfsTraceV1, SyntheticDriverError, SyntheticMaterialSample,
};

const DRIVER_SOURCE: &[u8] = include_bytes!("driver.rs");
const SOURCE_DOMAIN: &[u8] = b"babylon.sfs-driver-source.v1";
const CONTRACT_DOMAIN: &[u8] = b"babylon.sfs-synthetic-driver-contract.v1";
const MAX_MANIFEST_BYTES: usize = 4_096;
const ROW_COUNT: usize = 7;

/// Parsed exact seven-row synthetic driver source contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntheticDriverContractV1 {
    canonical_bytes: Vec<u8>,
    manifest_digest: Digest32,
    source_digest: Digest32,
}

/// Opaque proof that a preregistration selected the parsed driver contract.
#[derive(Debug)]
pub struct ValidatedSyntheticDriver<'a> {
    contract: &'a SyntheticDriverContractV1,
}

/// Exact driver-contract framing and identity refusals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntheticDriverContractError {
    ManifestByteLimit { actual: usize },
    ManifestMalformed { row: usize },
    SourceDigestMismatch,
    ContractDigestMismatch,
    PreregistrationDigestMismatch,
}

impl SyntheticDriverContractV1 {
    /// Returns the complete domain-separated manifest identity.
    #[must_use]
    pub const fn manifest_digest(&self) -> Digest32 {
        self.manifest_digest
    }

    /// Returns the internally recomputed source identity.
    #[must_use]
    pub const fn source_digest(&self) -> Digest32 {
        self.source_digest
    }

    /// Returns the exact canonical manifest bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

/// Parses and seals the exact source-bound synthetic driver contract.
///
/// # Errors
/// Returns the first byte, row, source, or contract identity refusal.
pub fn parse_synthetic_driver_contract(
    manifest_bytes: &[u8],
) -> Result<SyntheticDriverContractV1, SyntheticDriverContractError> {
    if manifest_bytes.len() > MAX_MANIFEST_BYTES {
        return Err(SyntheticDriverContractError::ManifestByteLimit {
            actual: manifest_bytes.len(),
        });
    }
    if manifest_bytes.contains(&b'\r')
        || !manifest_bytes.ends_with(b"\n")
        || manifest_bytes.ends_with(b"\n\n")
    {
        return Err(SyntheticDriverContractError::ManifestMalformed { row: 0 });
    }
    let source_digest = domain_digest(SOURCE_DOMAIN, DRIVER_SOURCE);
    let source_row = format!("source|driver.rs|{}", source_digest.to_hex());
    let expected = [
        "schema|1",
        "predicate|candidate-projection|1",
        "predicate|cumulative-driver-shape|1",
        "predicate|persistence-comparison-identity|1",
        "predicate|aligned-material-sequence|1",
        "predicate|twin-identity-difference|1",
        &source_row,
    ];
    let mut rows = manifest_bytes[..manifest_bytes.len() - 1].split(|byte| *byte == b'\n');
    #[allow(clippy::needless_range_loop, reason = "seven-row parser contract")]
    for index in 0..ROW_COUNT {
        let actual = rows
            .next()
            .ok_or(SyntheticDriverContractError::ManifestMalformed { row: index + 1 })?;
        if actual != expected[index].as_bytes() {
            if index == 6 && actual.starts_with(b"source|driver.rs|") {
                return Err(SyntheticDriverContractError::SourceDigestMismatch);
            }
            return Err(SyntheticDriverContractError::ManifestMalformed { row: index + 1 });
        }
    }
    if rows.next().is_some() {
        return Err(SyntheticDriverContractError::ManifestMalformed { row: 8 });
    }
    Ok(SyntheticDriverContractV1 {
        canonical_bytes: manifest_bytes.to_vec(),
        manifest_digest: domain_digest(CONTRACT_DOMAIN, manifest_bytes),
        source_digest,
    })
}

/// Binds the only public predicate handle to a preregistered contract digest.
///
/// # Errors
/// Returns `PreregistrationDigestMismatch` before exposing any predicate.
pub fn bind_synthetic_driver<'a>(
    preregistration: &SfsPreregistrationV1,
    contract: &'a SyntheticDriverContractV1,
) -> Result<ValidatedSyntheticDriver<'a>, SyntheticDriverContractError> {
    if preregistration.driver_contract_digest() != contract.manifest_digest {
        return Err(SyntheticDriverContractError::PreregistrationDigestMismatch);
    }
    Ok(ValidatedSyntheticDriver { contract })
}

impl ValidatedSyntheticDriver<'_> {
    /// Validates one complete flat-cadence candidate realization.
    ///
    /// # Errors
    /// Returns the first exact candidate, ledger, cadence, or intent refusal.
    pub fn validate_candidate_projection(
        &self,
        run_identity: &RunIdentityV1,
        preregistration: &SfsPreregistrationV1,
        schedule: &PracticeCandidateScheduleV1,
        attempts: &PracticeAttemptLedgerV1,
        intents: &[PracticeIntentV1],
        actual_exogenous_ledger_digest: Digest32,
    ) -> Result<(), SyntheticDriverError> {
        let _ = self.contract;
        driver::validate_candidate_projection(
            run_identity,
            preregistration,
            schedule,
            attempts,
            intents,
            actual_exogenous_ledger_digest,
        )
    }

    /// Validates that synthetic twins differ in exactly the selected ledger.
    ///
    /// # Errors
    /// Returns the exact twin-identity refusal.
    pub fn validate_twin_identity_difference(
        &self,
        control: &RunIdentityV1,
        intervention: &RunIdentityV1,
        selected: DifferingLedgerKindV1,
    ) -> Result<(), SyntheticDriverError> {
        driver::validate_twin_identity_difference(control, intervention, selected)
    }

    /// Validates every sealed persistence-comparison identity edge.
    ///
    /// # Errors
    /// Returns the first exact trace, ledger, comparison, or delta refusal.
    pub fn validate_persistence_comparison_identity(
        &self,
        control: &RunIdentityV1,
        intervention: &RunIdentityV1,
        control_trace: &SfsTraceV1,
        intervention_trace: &SfsTraceV1,
        comparison: &PersistenceComparisonV1,
        intervention_delta: &InterventionDeltaV1,
    ) -> Result<(), SyntheticDriverError> {
        driver::validate_persistence_comparison_identity(
            control,
            intervention,
            control_trace,
            intervention_trace,
            comparison,
            intervention_delta,
        )
    }

    /// Refuses cumulative traces that themselves encode the target shape.
    ///
    /// # Errors
    /// Returns the first value, monotonicity, classifier, or shape refusal.
    pub fn validate_driver_shapes(
        &self,
        window_width: u16,
        attempted_quanta: &[f64],
        governed_costs: &[f64],
    ) -> Result<(), SyntheticDriverError> {
        driver::validate_cumulative_driver_shapes(window_width, attempted_quanta, governed_costs)
    }

    /// Compares one exact time-shifted synthetic material sequence.
    ///
    /// # Errors
    /// Returns the first count, offset, tick, contribution, or aggregate refusal.
    pub fn validate_aligned_material_sequence(
        &self,
        control: &[SyntheticMaterialSample],
        aligned: &[SyntheticMaterialSample],
        window_width: u16,
        tick_offset: u16,
    ) -> Result<(), SyntheticDriverError> {
        driver::validate_aligned_material(control, aligned, window_width, tick_offset)
    }
}

fn domain_digest(domain: &[u8], payload: &[u8]) -> Digest32 {
    let mut preimage = Vec::with_capacity(domain.len() + 1 + payload.len());
    preimage.extend_from_slice(domain);
    preimage.push(0);
    preimage.extend_from_slice(payload);
    Digest32::from_bytes(sha256_of(&preimage))
}
