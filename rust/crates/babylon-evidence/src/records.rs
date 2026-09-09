//! Frozen run, trace, preregistration, schedule, and attempt record contracts.

use std::cmp::Ordering;

use babylon_kernel::{sha256_of, SessionId};
use babylon_practice_contract::PracticeIdV1;

use crate::classifier::{classify_sfs, SfsClass, SfsClassError};
use crate::digest::Digest32;
use crate::wire::{PayloadCursor, PayloadEncoder, SfsWireError, T3Record};

const MAX_ROWS: usize = 65_535;
const SAMPLE_INTERVAL: u16 = 1;

/// Exact construction and decode refusals for the frozen T3 run records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfsRecordError {
    /// A uniform-envelope or primitive wire rule failed.
    Wire(SfsWireError),
    /// The discrete SFS classifier refused the sample aggregates.
    Classifier(SfsClassError),
    /// Session UTF-8 bytes lie outside 1 through 256.
    InvalidSessionLength { actual: usize },
    /// V1 admits only one committed tick between samples.
    InvalidSampleInterval { found: u16 },
    /// V1 window width lies outside 2 through 52.
    InvalidWindowWidth { found: u16 },
    /// A trace does not contain exactly `3w + 1` samples.
    WrongSampleCount { expected: usize, actual: usize },
    /// A stored tick differs from the next exact committed tick.
    TickDiscontinuity { expected: u64, actual: u64 },
    /// A stored candidate identity differs from its exact preimage hash.
    StableRowDigestMismatch,
    /// Attempt-row projection differs from its declared candidate schedule.
    CandidateProjectionMismatch,
    /// A valid stored class differs from the recomputed class.
    ClassificationMismatch { stored: u8, computed: u8 },
    /// V1 cadence is not exact flat cadence with a positive stride.
    InvalidCadence,
    /// V1 exogenous policy is not exact empty policy.
    InvalidExogenousPolicy,
    /// A practice code is outside the shared practice-contract registry.
    InvalidPracticeCode { value: u8 },
    /// An attempt disposition code is outside the closed V1 mapping.
    InvalidDisposition { value: u8 },
    /// An attempt disposition digest uses the reserved zero value.
    ZeroDispositionDigest,
    /// A checked record index or length calculation overflowed.
    ArithmeticOverflow { field: &'static str },
}

impl From<SfsWireError> for SfsRecordError {
    fn from(value: SfsWireError) -> Self {
        Self::Wire(value)
    }
}

impl From<SfsClassError> for SfsRecordError {
    fn from(value: SfsClassError) -> Self {
        Self::Classifier(value)
    }
}

/// One field of the exact eighteen-field run identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunIdentityField {
    /// Exact session UTF-8 bytes.
    Session,
    /// Scenario content digest.
    Scenario,
    /// Composed prelude/declarations digest.
    PreludeDeclarations,
    /// Vocabulary digest.
    Vocabulary,
    /// Rule-AST digest.
    RuleAst,
    /// Host component manifest digest.
    HostComponentManifest,
    /// Defines digest.
    Defines,
    /// Intrinsic cost-cap digest.
    IntrinsicCostCap,
    /// Reference manifest digest.
    ReferenceManifest,
    /// Governed footprint manifest digest.
    GovernedFootprintManifest,
    /// SFS proof-profile digest.
    SfsProofProfile,
    /// SFS preregistration digest.
    SfsPreregistration,
    /// Initial committed-envelope digest.
    InitialCommittedEnvelope,
    /// Initial nominal-world hash.
    InitialNominalWorld,
    /// Exogenous-input ledger digest.
    ExogenousInputLedger,
    /// Practice-attempt ledger digest.
    PracticeAttemptLedger,
    /// RNG algorithm identifier.
    RngAlgorithmId,
    /// Graph-contract identifier.
    GraphContractId,
}

/// Complete synthetic run identity without a live-envelope authority claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunIdentityV1 {
    session: SessionId,
    scenario_digest: Digest32,
    prelude_declarations_digest: Digest32,
    vocabulary_digest: Digest32,
    rule_ast_digest: Digest32,
    host_component_manifest_digest: Digest32,
    defines_digest: Digest32,
    intrinsic_cost_cap_digest: Digest32,
    reference_manifest_digest: Digest32,
    governed_footprint_manifest_digest: Digest32,
    sfs_proof_profile_digest: Digest32,
    sfs_preregistration_digest: Digest32,
    initial_committed_envelope_digest: Digest32,
    initial_nominal_world_hash: Digest32,
    exogenous_input_ledger_digest: Digest32,
    practice_attempt_ledger_digest: Digest32,
    rng_algorithm_id: String,
    graph_contract_id: String,
}

impl RunIdentityV1 {
    /// Constructs the exact sealed eighteen-field run identity.
    ///
    /// # Errors
    /// Returns the first exact session, NFC, ASCII, or length refusal.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session: SessionId,
        scenario_digest: Digest32,
        prelude_declarations_digest: Digest32,
        vocabulary_digest: Digest32,
        rule_ast_digest: Digest32,
        host_component_manifest_digest: Digest32,
        defines_digest: Digest32,
        intrinsic_cost_cap_digest: Digest32,
        reference_manifest_digest: Digest32,
        governed_footprint_manifest_digest: Digest32,
        sfs_proof_profile_digest: Digest32,
        sfs_preregistration_digest: Digest32,
        initial_committed_envelope_digest: Digest32,
        initial_nominal_world_hash: Digest32,
        exogenous_input_ledger_digest: Digest32,
        practice_attempt_ledger_digest: Digest32,
        rng_algorithm_id: &str,
        graph_contract_id: &str,
    ) -> Result<Self, SfsRecordError> {
        validate_session(&session)?;
        validate_ascii_id("rng_algorithm_id", rng_algorithm_id)?;
        validate_ascii_id("graph_contract_id", graph_contract_id)?;
        Ok(Self {
            session,
            scenario_digest,
            prelude_declarations_digest,
            vocabulary_digest,
            rule_ast_digest,
            host_component_manifest_digest,
            defines_digest,
            intrinsic_cost_cap_digest,
            reference_manifest_digest,
            governed_footprint_manifest_digest,
            sfs_proof_profile_digest,
            sfs_preregistration_digest,
            initial_committed_envelope_digest,
            initial_nominal_world_hash,
            exogenous_input_ledger_digest,
            practice_attempt_ledger_digest,
            rng_algorithm_id: rng_algorithm_id.to_owned(),
            graph_contract_id: graph_contract_id.to_owned(),
        })
    }

    /// Returns every differing field in exact wire-field order.
    #[must_use]
    pub fn differing_fields(&self, other: &Self) -> Vec<RunIdentityField> {
        let mut fields = Vec::with_capacity(18);
        for index in 0..18 {
            let (field, differs) = self.field_difference(index, other);
            if differs {
                fields.push(field);
            }
        }
        fields
    }

    fn field_difference(&self, index: usize, other: &Self) -> (RunIdentityField, bool) {
        match index {
            0 => (RunIdentityField::Session, self.session != other.session),
            1 => (
                RunIdentityField::Scenario,
                self.scenario_digest != other.scenario_digest,
            ),
            2 => (
                RunIdentityField::PreludeDeclarations,
                self.prelude_declarations_digest != other.prelude_declarations_digest,
            ),
            3 => (
                RunIdentityField::Vocabulary,
                self.vocabulary_digest != other.vocabulary_digest,
            ),
            4 => (
                RunIdentityField::RuleAst,
                self.rule_ast_digest != other.rule_ast_digest,
            ),
            5 => (
                RunIdentityField::HostComponentManifest,
                self.host_component_manifest_digest != other.host_component_manifest_digest,
            ),
            6 => (
                RunIdentityField::Defines,
                self.defines_digest != other.defines_digest,
            ),
            7 => (
                RunIdentityField::IntrinsicCostCap,
                self.intrinsic_cost_cap_digest != other.intrinsic_cost_cap_digest,
            ),
            8 => (
                RunIdentityField::ReferenceManifest,
                self.reference_manifest_digest != other.reference_manifest_digest,
            ),
            9 => (
                RunIdentityField::GovernedFootprintManifest,
                self.governed_footprint_manifest_digest != other.governed_footprint_manifest_digest,
            ),
            10 => (
                RunIdentityField::SfsProofProfile,
                self.sfs_proof_profile_digest != other.sfs_proof_profile_digest,
            ),
            11 => (
                RunIdentityField::SfsPreregistration,
                self.sfs_preregistration_digest != other.sfs_preregistration_digest,
            ),
            12 => (
                RunIdentityField::InitialCommittedEnvelope,
                self.initial_committed_envelope_digest != other.initial_committed_envelope_digest,
            ),
            13 => (
                RunIdentityField::InitialNominalWorld,
                self.initial_nominal_world_hash != other.initial_nominal_world_hash,
            ),
            14 => (
                RunIdentityField::ExogenousInputLedger,
                self.exogenous_input_ledger_digest != other.exogenous_input_ledger_digest,
            ),
            15 => (
                RunIdentityField::PracticeAttemptLedger,
                self.practice_attempt_ledger_digest != other.practice_attempt_ledger_digest,
            ),
            16 => (
                RunIdentityField::RngAlgorithmId,
                self.rng_algorithm_id != other.rng_algorithm_id,
            ),
            17 => (
                RunIdentityField::GraphContractId,
                self.graph_contract_id != other.graph_contract_id,
            ),
            _ => unreachable!("fixed run-identity field dispatch is 0 through 17"),
        }
    }

    /// Returns the sealed host-component manifest digest.
    #[must_use]
    pub const fn host_component_manifest_digest(&self) -> Digest32 {
        self.host_component_manifest_digest
    }
    /// Returns the governed footprint manifest digest.
    #[must_use]
    pub const fn governed_footprint_manifest_digest(&self) -> Digest32 {
        self.governed_footprint_manifest_digest
    }
    /// Returns the SFS proof-profile digest.
    #[must_use]
    pub const fn sfs_proof_profile_digest(&self) -> Digest32 {
        self.sfs_proof_profile_digest
    }
    /// Returns the SFS preregistration digest.
    #[must_use]
    pub const fn sfs_preregistration_digest(&self) -> Digest32 {
        self.sfs_preregistration_digest
    }
    /// Returns the exogenous-input ledger digest.
    #[must_use]
    pub const fn exogenous_input_ledger_digest(&self) -> Digest32 {
        self.exogenous_input_ledger_digest
    }
    /// Returns the practice-attempt ledger digest.
    #[must_use]
    pub const fn practice_attempt_ledger_digest(&self) -> Digest32 {
        self.practice_attempt_ledger_digest
    }
}

impl T3Record for RunIdentityV1 {
    const DOMAIN: &'static [u8] = b"babylon.run-identity.v1";
    const MAX_PAYLOAD_BYTES: usize = 870;
    type Error = SfsRecordError;

    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_nfc_utf8("session", self.session.as_str(), 256)?;
        out.push_digest(self.scenario_digest)?;
        out.push_digest(self.prelude_declarations_digest)?;
        out.push_digest(self.vocabulary_digest)?;
        out.push_digest(self.rule_ast_digest)?;
        out.push_digest(self.host_component_manifest_digest)?;
        out.push_digest(self.defines_digest)?;
        out.push_digest(self.intrinsic_cost_cap_digest)?;
        out.push_digest(self.reference_manifest_digest)?;
        out.push_digest(self.governed_footprint_manifest_digest)?;
        out.push_digest(self.sfs_proof_profile_digest)?;
        out.push_digest(self.sfs_preregistration_digest)?;
        out.push_digest(self.initial_committed_envelope_digest)?;
        out.push_digest(self.initial_nominal_world_hash)?;
        out.push_digest(self.exogenous_input_ledger_digest)?;
        out.push_digest(self.practice_attempt_ledger_digest)?;
        out.push_ascii("rng_algorithm_id", &self.rng_algorithm_id, 64)?;
        out.push_ascii("graph_contract_id", &self.graph_contract_id, 64)
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, Self::Error> {
        let session = read_session(cursor)?;
        Self::new(
            session,
            cursor.read_digest()?,
            cursor.read_digest()?,
            cursor.read_digest()?,
            cursor.read_digest()?,
            cursor.read_digest()?,
            cursor.read_digest()?,
            cursor.read_digest()?,
            cursor.read_digest()?,
            cursor.read_digest()?,
            cursor.read_digest()?,
            cursor.read_digest()?,
            cursor.read_digest()?,
            cursor.read_digest()?,
            cursor.read_digest()?,
            cursor.read_digest()?,
            &cursor.read_ascii("rng_algorithm_id", 64)?,
            &cursor.read_ascii("graph_contract_id", 64)?,
        )
    }
}

/// One exact post-commit membership aggregate sample.
#[derive(Debug, Clone, PartialEq)]
pub struct SfsSampleV1 {
    tick: u64,
    nominal_world_hash: Digest32,
    committed_envelope_digest: Digest32,
    sorted_contribution_digest: Digest32,
    aggregate: f64,
}

impl SfsSampleV1 {
    /// Constructs one finite non-negative sample and normalizes either zero sign.
    ///
    /// # Errors
    /// Returns the exact non-finite or negative aggregate refusal.
    pub fn new(
        tick: u64,
        nominal_world_hash: Digest32,
        committed_envelope_digest: Digest32,
        sorted_contribution_digest: Digest32,
        aggregate: f64,
    ) -> Result<Self, SfsRecordError> {
        let aggregate = normalize_nonnegative("aggregate", aggregate)?;
        Ok(Self {
            tick,
            nominal_world_hash,
            committed_envelope_digest,
            sorted_contribution_digest,
            aggregate,
        })
    }
}

impl T3Record for SfsSampleV1 {
    const DOMAIN: &'static [u8] = b"babylon.sfs-sample.v1";
    const MAX_PAYLOAD_BYTES: usize = 112;
    type Error = SfsRecordError;

    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_u64(self.tick)?;
        out.push_digest(self.nominal_world_hash)?;
        out.push_digest(self.committed_envelope_digest)?;
        out.push_digest(self.sorted_contribution_digest)?;
        out.push_finite_non_negative_f64("aggregate", self.aggregate)
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, Self::Error> {
        Self::new(
            cursor.read_u64()?,
            cursor.read_digest()?,
            cursor.read_digest()?,
            cursor.read_digest()?,
            cursor.read_finite_non_negative_f64("aggregate")?,
        )
    }
}

/// One exact completed SFS trace whose class is always recomputed.
#[derive(Debug, Clone, PartialEq)]
pub struct SfsTraceV1 {
    run_identity_digest: Digest32,
    relational_scope_digest: Digest32,
    organization_node_id: u64,
    start_tick: u64,
    sample_interval: u16,
    window_width: u16,
    samples: Vec<SfsSampleV1>,
    classification: SfsClass,
}

impl SfsTraceV1 {
    /// Constructs a consecutive interval-one trace and computes its class.
    ///
    /// # Errors
    /// Returns the first width, count, tick, arithmetic, or classifier refusal.
    #[allow(clippy::needless_range_loop)] // Literal ceiling is part of the trace contract.
    pub fn new(
        run_identity_digest: Digest32,
        relational_scope_digest: Digest32,
        organization_node_id: u64,
        start_tick: u64,
        window_width: u16,
        samples: Vec<SfsSampleV1>,
    ) -> Result<Self, SfsRecordError> {
        let expected = expected_sample_count(window_width)?;
        if samples.len() != expected {
            return Err(SfsRecordError::WrongSampleCount {
                expected,
                actual: samples.len(),
            });
        }
        let mut masses = Vec::with_capacity(expected);
        for index in 0..157 {
            if index >= expected {
                break;
            }
            let offset = u64::try_from(index).map_err(|_| SfsRecordError::ArithmeticOverflow {
                field: "sample_tick",
            })?;
            let expected_tick =
                start_tick
                    .checked_add(offset)
                    .ok_or(SfsRecordError::ArithmeticOverflow {
                        field: "sample_tick",
                    })?;
            if samples[index].tick != expected_tick {
                return Err(SfsRecordError::TickDiscontinuity {
                    expected: expected_tick,
                    actual: samples[index].tick,
                });
            }
            masses.push(samples[index].aggregate);
        }
        let classification = classify_sfs(window_width, &masses)?;
        Ok(Self {
            run_identity_digest,
            relational_scope_digest,
            organization_node_id,
            start_tick,
            sample_interval: SAMPLE_INTERVAL,
            window_width,
            samples,
            classification,
        })
    }

    /// Returns the complete run-identity digest bound by this trace.
    #[must_use]
    pub const fn run_identity_digest(&self) -> Digest32 {
        self.run_identity_digest
    }
}

impl T3Record for SfsTraceV1 {
    const DOMAIN: &'static [u8] = b"babylon.sfs-trace.v1";
    const MAX_PAYLOAD_BYTES: usize = 22_067;
    type Error = SfsRecordError;

    #[allow(clippy::needless_range_loop)] // Literal ceiling is part of the trace contract.
    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_digest(self.run_identity_digest)?;
        out.push_digest(self.relational_scope_digest)?;
        out.push_u64(self.organization_node_id)?;
        out.push_u64(self.start_tick)?;
        out.push_u16(self.sample_interval)?;
        out.push_u16(self.window_width)?;
        out.push_u16(u16::try_from(self.samples.len()).map_err(|_| {
            SfsWireError::ArithmeticOverflow {
                field: "sample_count",
            }
        })?)?;
        for index in 0..157 {
            if index >= self.samples.len() {
                break;
            }
            out.push_complete_envelope(&self.samples[index])?;
        }
        out.push_u8(self.classification.code())
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, Self::Error> {
        let run_identity_digest = cursor.read_digest()?;
        let relational_scope_digest = cursor.read_digest()?;
        let organization_node_id = cursor.read_u64()?;
        let start_tick = cursor.read_u64()?;
        let sample_interval = cursor.read_u16()?;
        if sample_interval != SAMPLE_INTERVAL {
            return Err(SfsRecordError::InvalidSampleInterval {
                found: sample_interval,
            });
        }
        let window_width = cursor.read_u16()?;
        let expected = expected_sample_count(window_width)?;
        let count = usize::from(cursor.read_u16()?);
        if count != expected {
            return Err(SfsRecordError::WrongSampleCount {
                expected,
                actual: count,
            });
        }
        let samples = read_samples(cursor, count)?;
        let stored_code = cursor.read_u8()?;
        let stored = SfsClass::from_code(stored_code).ok_or(SfsWireError::InvalidCode {
            field: "classification",
            value: stored_code,
        })?;
        let value = Self::new(
            run_identity_digest,
            relational_scope_digest,
            organization_node_id,
            start_tick,
            window_width,
            samples,
        )?;
        if value.classification != stored {
            return Err(SfsRecordError::ClassificationMismatch {
                stored: stored.code(),
                computed: value.classification.code(),
            });
        }
        Ok(value)
    }
}

/// One fixed candidate row with its exact stable identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeCandidateRowV1 {
    stable_row_id_digest: Digest32,
    attempt_tick: u64,
    practice_input_authority_digest: Digest32,
    practice_intent_digest: Digest32,
}

impl PracticeCandidateRowV1 {
    /// Constructs one row and derives its stable identity from the exact preimage.
    #[must_use]
    pub fn new(
        attempt_tick: u64,
        practice_input_authority_digest: Digest32,
        practice_intent_digest: Digest32,
    ) -> Self {
        Self {
            stable_row_id_digest: practice_attempt_row_id(
                attempt_tick,
                practice_input_authority_digest,
                practice_intent_digest,
            ),
            attempt_tick,
            practice_input_authority_digest,
            practice_intent_digest,
        }
    }

    /// Returns the canonical attempt tick.
    #[must_use]
    pub const fn attempt_tick(&self) -> u64 {
        self.attempt_tick
    }
    /// Returns the exact practice-intent digest.
    #[must_use]
    pub const fn practice_intent_digest(&self) -> Digest32 {
        self.practice_intent_digest
    }
}

/// Derives the exact stable candidate/attempt row identity.
#[must_use]
pub fn practice_attempt_row_id(
    attempt_tick: u64,
    authority_digest: Digest32,
    intent_digest: Digest32,
) -> Digest32 {
    let mut preimage = Vec::with_capacity(104);
    preimage.extend_from_slice(b"babylon.practice-attempt-row.v1");
    preimage.push(0);
    preimage.extend_from_slice(&attempt_tick.to_be_bytes());
    preimage.extend_from_slice(authority_digest.as_bytes());
    preimage.extend_from_slice(intent_digest.as_bytes());
    Digest32::from_bytes(sha256_of(&preimage))
}

/// Predeclared canonical practice candidates without dispositions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeCandidateScheduleV1 {
    rows: Vec<PracticeCandidateRowV1>,
}

impl PracticeCandidateScheduleV1 {
    /// Preflights, sorts, and rejects duplicate canonical candidate keys.
    ///
    /// # Errors
    /// Returns exact count or duplicate-entry refusals before publishing rows.
    pub fn new(mut rows: Vec<PracticeCandidateRowV1>) -> Result<Self, SfsRecordError> {
        validate_count("candidate_rows", rows.len())?;
        rows.sort_by(candidate_order);
        reject_candidate_duplicates(&rows, "candidate_rows")?;
        Ok(Self { rows })
    }

    /// Returns the immutable canonical rows.
    #[must_use]
    pub fn rows(&self) -> &[PracticeCandidateRowV1] {
        &self.rows
    }
}

impl T3Record for PracticeCandidateScheduleV1 {
    const DOMAIN: &'static [u8] = b"babylon.practice-candidate-schedule.v1";
    const MAX_PAYLOAD_BYTES: usize = 6_815_644;
    type Error = SfsRecordError;

    #[allow(clippy::needless_range_loop)] // Literal ceiling is part of the row contract.
    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_u32(u32::try_from(self.rows.len()).map_err(|_| {
            SfsWireError::ArithmeticOverflow {
                field: "candidate_rows",
            }
        })?)?;
        for index in 0..65_535 {
            if index >= self.rows.len() {
                break;
            }
            encode_candidate(out, &self.rows[index])?;
        }
        Ok(())
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, Self::Error> {
        let count = decode_count(cursor, "candidate_rows")?;
        let rows = read_candidates(cursor, count, "candidate_rows")?;
        Self::new(rows)
    }
}

/// Exact preregistration for the flat synthetic practice schedule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfsPreregistrationV1 {
    preregistered_at_tick: u64,
    start_tick: u64,
    relational_scope_digest: Digest32,
    practice_candidate_schedule_digest: Digest32,
    sfs_proof_profile_digest: Digest32,
    driver_contract_digest: Digest32,
    mutation_manifest_digest: Digest32,
    expected_exogenous_ledger_digest: Digest32,
    exogenous_policy: u8,
    cadence_kind: u8,
    first_attempt_tick: u64,
    attempt_stride: u16,
    attempt_count: u16,
    practice_code: PracticeIdV1,
    target_selection_policy_digest: Digest32,
    governed_cost: u32,
    parameter_bytes_digest: Digest32,
}

impl SfsPreregistrationV1 {
    /// Constructs exact empty-policy, flat-cadence preregistration bytes.
    ///
    /// # Errors
    /// Returns start-tick overflow or a non-positive cadence refusal.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        preregistered_at_tick: u64,
        relational_scope_digest: Digest32,
        practice_candidate_schedule_digest: Digest32,
        sfs_proof_profile_digest: Digest32,
        driver_contract_digest: Digest32,
        mutation_manifest_digest: Digest32,
        expected_exogenous_ledger_digest: Digest32,
        first_attempt_tick: u64,
        attempt_stride: u16,
        attempt_count: u16,
        practice_code: PracticeIdV1,
        target_selection_policy_digest: Digest32,
        governed_cost: u32,
        parameter_bytes_digest: Digest32,
    ) -> Result<Self, SfsRecordError> {
        let start_tick =
            preregistered_at_tick
                .checked_add(1)
                .ok_or(SfsRecordError::ArithmeticOverflow {
                    field: "start_tick",
                })?;
        if attempt_stride == 0 {
            return Err(SfsRecordError::InvalidCadence);
        }
        Ok(Self {
            preregistered_at_tick,
            start_tick,
            relational_scope_digest,
            practice_candidate_schedule_digest,
            sfs_proof_profile_digest,
            driver_contract_digest,
            mutation_manifest_digest,
            expected_exogenous_ledger_digest,
            exogenous_policy: 0,
            cadence_kind: 0,
            first_attempt_tick,
            attempt_stride,
            attempt_count,
            practice_code,
            target_selection_policy_digest,
            governed_cost,
            parameter_bytes_digest,
        })
    }

    /// Returns the candidate-schedule digest.
    #[must_use]
    pub const fn practice_candidate_schedule_digest(&self) -> Digest32 {
        self.practice_candidate_schedule_digest
    }
    /// Returns the SFS proof-profile digest.
    #[must_use]
    pub const fn sfs_proof_profile_digest(&self) -> Digest32 {
        self.sfs_proof_profile_digest
    }
    /// Returns the sealed synthetic driver-contract digest.
    #[must_use]
    pub const fn driver_contract_digest(&self) -> Digest32 {
        self.driver_contract_digest
    }
    /// Returns the mutation-manifest digest.
    #[must_use]
    pub const fn mutation_manifest_digest(&self) -> Digest32 {
        self.mutation_manifest_digest
    }
    /// Returns the expected synthetic exogenous-ledger digest.
    #[must_use]
    pub const fn expected_exogenous_ledger_digest(&self) -> Digest32 {
        self.expected_exogenous_ledger_digest
    }
    /// Returns the first predeclared attempt tick.
    #[must_use]
    pub const fn first_attempt_tick(&self) -> u64 {
        self.first_attempt_tick
    }
    /// Returns the positive flat-cadence stride.
    #[must_use]
    pub const fn attempt_stride(&self) -> u16 {
        self.attempt_stride
    }
    /// Returns the declared attempt count.
    #[must_use]
    pub const fn attempt_count(&self) -> u16 {
        self.attempt_count
    }
    /// Returns the shared governed practice code.
    #[must_use]
    pub const fn practice_code(&self) -> PracticeIdV1 {
        self.practice_code
    }
    /// Returns the target-selection policy digest.
    #[must_use]
    pub const fn target_selection_policy_digest(&self) -> Digest32 {
        self.target_selection_policy_digest
    }
    /// Returns the constant governed cost.
    #[must_use]
    pub const fn governed_cost(&self) -> u32 {
        self.governed_cost
    }
    /// Returns the constant parameter-bytes digest.
    #[must_use]
    pub const fn parameter_bytes_digest(&self) -> Digest32 {
        self.parameter_bytes_digest
    }
}

impl T3Record for SfsPreregistrationV1 {
    const DOMAIN: &'static [u8] = b"babylon.sfs-preregistration.v1";
    const MAX_PAYLOAD_BYTES: usize = 291;
    type Error = SfsRecordError;

    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_u64(self.preregistered_at_tick)?;
        out.push_u64(self.start_tick)?;
        out.push_digest(self.relational_scope_digest)?;
        out.push_digest(self.practice_candidate_schedule_digest)?;
        out.push_digest(self.sfs_proof_profile_digest)?;
        out.push_digest(self.driver_contract_digest)?;
        out.push_digest(self.mutation_manifest_digest)?;
        out.push_digest(self.expected_exogenous_ledger_digest)?;
        out.push_u8(self.exogenous_policy)?;
        out.push_u8(self.cadence_kind)?;
        out.push_u64(self.first_attempt_tick)?;
        out.push_u16(self.attempt_stride)?;
        out.push_u16(self.attempt_count)?;
        out.push_u8(self.practice_code as u8)?;
        out.push_digest(self.target_selection_policy_digest)?;
        out.push_u32(self.governed_cost)?;
        out.push_digest(self.parameter_bytes_digest)
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, Self::Error> {
        let preregistered_at_tick = cursor.read_u64()?;
        let stored_start = cursor.read_u64()?;
        let relational_scope_digest = cursor.read_digest()?;
        let schedule_digest = cursor.read_digest()?;
        let proof_digest = cursor.read_digest()?;
        let driver_digest = cursor.read_digest()?;
        let mutation_digest = cursor.read_digest()?;
        let exogenous_digest = cursor.read_digest()?;
        if cursor.read_u8()? != 0 {
            return Err(SfsRecordError::InvalidExogenousPolicy);
        }
        if cursor.read_u8()? != 0 {
            return Err(SfsRecordError::InvalidCadence);
        }
        let first_attempt_tick = cursor.read_u64()?;
        let stride = cursor.read_u16()?;
        let count = cursor.read_u16()?;
        let practice_value = cursor.read_u8()?;
        let practice = PracticeIdV1::try_from(practice_value).map_err(|_| {
            SfsRecordError::InvalidPracticeCode {
                value: practice_value,
            }
        })?;
        let target_digest = cursor.read_digest()?;
        let governed_cost = cursor.read_u32()?;
        let parameter_digest = cursor.read_digest()?;
        let value = Self::new(
            preregistered_at_tick,
            relational_scope_digest,
            schedule_digest,
            proof_digest,
            driver_digest,
            mutation_digest,
            exogenous_digest,
            first_attempt_tick,
            stride,
            count,
            practice,
            target_digest,
            governed_cost,
            parameter_digest,
        )?;
        if stored_start != value.start_tick {
            return Err(SfsRecordError::TickDiscontinuity {
                expected: value.start_tick,
                actual: stored_start,
            });
        }
        Ok(value)
    }
}

/// Closed attempt disposition codes.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PracticeDispositionV1 {
    /// Candidate admitted to the durable pending ledger.
    Accepted = 0,
    /// Candidate received an exact stable rejection.
    Rejected = 1,
}

impl PracticeDispositionV1 {
    /// Returns the exact V1 code.
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }
    /// Maps one closed V1 code without fallback.
    #[must_use]
    pub const fn from_code(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Accepted),
            1 => Some(Self::Rejected),
            _ => None,
        }
    }
}

/// One candidate plus its admission disposition and exact disposition digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeAttemptRowV1 {
    candidate: PracticeCandidateRowV1,
    disposition: PracticeDispositionV1,
    disposition_digest: Digest32,
}

impl PracticeAttemptRowV1 {
    /// Constructs one attempt row and refuses the reserved zero digest.
    ///
    /// # Errors
    /// Returns `ZeroDispositionDigest` for the reserved zero value.
    pub fn new(
        candidate: PracticeCandidateRowV1,
        disposition: PracticeDispositionV1,
        disposition_digest: Digest32,
    ) -> Result<Self, SfsRecordError> {
        if disposition_digest.is_zero() {
            return Err(SfsRecordError::ZeroDispositionDigest);
        }
        Ok(Self {
            candidate,
            disposition,
            disposition_digest,
        })
    }
}

/// Frozen attempt ledger including both accepted and rejected candidates.
///
/// This crate cannot verify `accepted_intent_ledger_digest` against an
/// authoritative accepted-intent ledger until Gate 5 supplies that contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeAttemptLedgerV1 {
    accepted_intent_ledger_digest: Digest32,
    rows: Vec<PracticeAttemptRowV1>,
}

impl PracticeAttemptLedgerV1 {
    /// Preflights, sorts, and rejects duplicate canonical attempt keys.
    ///
    /// # Errors
    /// Returns exact count or duplicate-entry refusals before publication.
    pub fn new(
        accepted_intent_ledger_digest: Digest32,
        mut rows: Vec<PracticeAttemptRowV1>,
    ) -> Result<Self, SfsRecordError> {
        validate_count("attempt_rows", rows.len())?;
        rows.sort_by(|left, right| candidate_order(&left.candidate, &right.candidate));
        reject_attempt_duplicates(&rows)?;
        Ok(Self {
            accepted_intent_ledger_digest,
            rows,
        })
    }

    /// Projects every attempt, including rejections, to exact candidate bytes.
    ///
    /// # Errors
    /// Returns an exact count or duplicate refusal if internal invariants fail.
    pub fn project_candidates(&self) -> Result<PracticeCandidateScheduleV1, SfsRecordError> {
        let mut rows = Vec::with_capacity(self.rows.len());
        for index in 0..65_535 {
            if index >= self.rows.len() {
                break;
            }
            rows.push(self.rows[index].candidate.clone());
        }
        PracticeCandidateScheduleV1::new(rows)
    }
}

impl T3Record for PracticeAttemptLedgerV1 {
    const DOMAIN: &'static [u8] = b"babylon.practice-attempt-ledger.v1";
    const MAX_PAYLOAD_BYTES: usize = 8_978_331;
    type Error = SfsRecordError;

    #[allow(clippy::needless_range_loop)] // Literal ceiling is part of the row contract.
    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_digest(self.accepted_intent_ledger_digest)?;
        out.push_u32(u32::try_from(self.rows.len()).map_err(|_| {
            SfsWireError::ArithmeticOverflow {
                field: "attempt_rows",
            }
        })?)?;
        for index in 0..65_535 {
            if index >= self.rows.len() {
                break;
            }
            encode_attempt(out, &self.rows[index])?;
        }
        Ok(())
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, Self::Error> {
        let header = cursor.read_digest()?;
        let count = decode_count(cursor, "attempt_rows")?;
        let rows = read_attempts(cursor, count)?;
        Self::new(header, rows)
    }
}

fn validate_session(session: &SessionId) -> Result<(), SfsRecordError> {
    let actual = session.as_bytes().len();
    if !(1..=256).contains(&actual) {
        return Err(SfsRecordError::InvalidSessionLength { actual });
    }
    let mut validator = PayloadEncoder::new(258);
    validator.push_nfc_utf8("session", session.as_str(), 256)?;
    Ok(())
}

fn validate_ascii_id(field: &'static str, value: &str) -> Result<(), SfsRecordError> {
    let mut validator = PayloadEncoder::new(66);
    validator.push_ascii(field, value, 64)?;
    Ok(())
}

fn read_session(cursor: &mut PayloadCursor<'_>) -> Result<SessionId, SfsRecordError> {
    let value = match cursor.read_nfc_utf8("session", 256) {
        Ok(value) => value,
        Err(SfsWireError::StringEmpty { .. }) => {
            return Err(SfsRecordError::InvalidSessionLength { actual: 0 })
        }
        Err(SfsWireError::StringTooLong { actual, .. }) => {
            return Err(SfsRecordError::InvalidSessionLength { actual })
        }
        Err(error) => return Err(error.into()),
    };
    SessionId::new(value).map_err(|_| SfsRecordError::InvalidSessionLength { actual: 0 })
}

fn normalize_nonnegative(field: &'static str, value: f64) -> Result<f64, SfsRecordError> {
    if !value.is_finite() {
        return Err(SfsWireError::NonFinite { field }.into());
    }
    if value < 0.0 {
        return Err(SfsWireError::Negative { field }.into());
    }
    if value == 0.0 {
        Ok(0.0)
    } else {
        Ok(value)
    }
}

fn expected_sample_count(window_width: u16) -> Result<usize, SfsRecordError> {
    if !(2..=52).contains(&window_width) {
        return Err(SfsRecordError::InvalidWindowWidth {
            found: window_width,
        });
    }
    usize::from(window_width)
        .checked_mul(3)
        .and_then(|value| value.checked_add(1))
        .ok_or(SfsRecordError::ArithmeticOverflow {
            field: "sample_count",
        })
}

fn read_samples(
    cursor: &mut PayloadCursor<'_>,
    count: usize,
) -> Result<Vec<SfsSampleV1>, SfsRecordError> {
    let mut samples = Vec::with_capacity(count);
    for index in 0..157 {
        if index >= count {
            break;
        }
        samples.push(cursor.read_complete_envelope::<SfsSampleV1>()?);
    }
    Ok(samples)
}

fn candidate_order(left: &PracticeCandidateRowV1, right: &PracticeCandidateRowV1) -> Ordering {
    left.attempt_tick
        .cmp(&right.attempt_tick)
        .then_with(|| left.stable_row_id_digest.cmp(&right.stable_row_id_digest))
}

fn validate_count(field: &'static str, actual: usize) -> Result<(), SfsRecordError> {
    if actual > MAX_ROWS {
        Err(SfsWireError::CountTooLarge {
            field,
            limit: MAX_ROWS,
            actual,
        }
        .into())
    } else {
        Ok(())
    }
}

fn decode_count(
    cursor: &mut PayloadCursor<'_>,
    field: &'static str,
) -> Result<usize, SfsRecordError> {
    let count = usize::try_from(cursor.read_u32()?)
        .map_err(|_| SfsRecordError::ArithmeticOverflow { field })?;
    validate_count(field, count)?;
    Ok(count)
}

fn encode_candidate(
    out: &mut PayloadEncoder,
    row: &PracticeCandidateRowV1,
) -> Result<(), SfsWireError> {
    out.push_digest(row.stable_row_id_digest)?;
    out.push_u64(row.attempt_tick)?;
    out.push_digest(row.practice_input_authority_digest)?;
    out.push_digest(row.practice_intent_digest)
}

fn decode_candidate(
    cursor: &mut PayloadCursor<'_>,
) -> Result<PracticeCandidateRowV1, SfsRecordError> {
    let stored = cursor.read_digest()?;
    let tick = cursor.read_u64()?;
    let authority = cursor.read_digest()?;
    let intent = cursor.read_digest()?;
    let row = PracticeCandidateRowV1::new(tick, authority, intent);
    if stored != row.stable_row_id_digest {
        return Err(SfsRecordError::StableRowDigestMismatch);
    }
    Ok(row)
}

fn reject_candidate_duplicates(
    rows: &[PracticeCandidateRowV1],
    field: &'static str,
) -> Result<(), SfsRecordError> {
    for index in 1..65_535 {
        if index >= rows.len() {
            break;
        }
        if candidate_order(&rows[index - 1], &rows[index]) == Ordering::Equal {
            return Err(SfsWireError::DuplicateEntry { field }.into());
        }
    }
    Ok(())
}

fn read_candidates(
    cursor: &mut PayloadCursor<'_>,
    count: usize,
    field: &'static str,
) -> Result<Vec<PracticeCandidateRowV1>, SfsRecordError> {
    let mut rows = Vec::with_capacity(count);
    for index in 0..65_535 {
        if index >= count {
            break;
        }
        let row = decode_candidate(cursor)?;
        validate_next_candidate(rows.last(), &row, field)?;
        rows.push(row);
    }
    Ok(rows)
}

fn validate_next_candidate(
    previous: Option<&PracticeCandidateRowV1>,
    row: &PracticeCandidateRowV1,
    field: &'static str,
) -> Result<(), SfsRecordError> {
    let Some(previous) = previous else {
        return Ok(());
    };
    match candidate_order(previous, row) {
        Ordering::Less => Ok(()),
        Ordering::Equal => Err(SfsWireError::DuplicateEntry { field }.into()),
        Ordering::Greater => Err(SfsWireError::OutOfOrder { field }.into()),
    }
}

fn encode_attempt(
    out: &mut PayloadEncoder,
    row: &PracticeAttemptRowV1,
) -> Result<(), SfsWireError> {
    encode_candidate(out, &row.candidate)?;
    out.push_u8(row.disposition.code())?;
    out.push_digest(row.disposition_digest)
}

fn decode_attempt(cursor: &mut PayloadCursor<'_>) -> Result<PracticeAttemptRowV1, SfsRecordError> {
    let candidate = decode_candidate(cursor)?;
    let code = cursor.read_u8()?;
    let disposition = PracticeDispositionV1::from_code(code)
        .ok_or(SfsRecordError::InvalidDisposition { value: code })?;
    let digest = cursor.read_digest()?;
    PracticeAttemptRowV1::new(candidate, disposition, digest)
}

fn reject_attempt_duplicates(rows: &[PracticeAttemptRowV1]) -> Result<(), SfsRecordError> {
    for index in 1..65_535 {
        if index >= rows.len() {
            break;
        }
        if candidate_order(&rows[index - 1].candidate, &rows[index].candidate) == Ordering::Equal {
            return Err(SfsWireError::DuplicateEntry {
                field: "attempt_rows",
            }
            .into());
        }
    }
    Ok(())
}

fn read_attempts(
    cursor: &mut PayloadCursor<'_>,
    count: usize,
) -> Result<Vec<PracticeAttemptRowV1>, SfsRecordError> {
    let mut rows = Vec::with_capacity(count);
    for index in 0..65_535 {
        if index >= count {
            break;
        }
        let row = decode_attempt(cursor)?;
        let previous = rows
            .last()
            .map(|item: &PracticeAttemptRowV1| &item.candidate);
        validate_next_candidate(previous, &row.candidate, "attempt_rows")?;
        rows.push(row);
    }
    Ok(rows)
}
