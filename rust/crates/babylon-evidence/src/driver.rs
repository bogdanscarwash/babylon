//! Source-bound synthetic driver predicates; never an authoritative producer.
#![allow(dead_code, reason = "called only by the source-bound driver handle")]

use babylon_practice_contract::{
    intent_digest, parameter_bytes_digest, target_selection_policy_digest, PracticeIntentV1,
};

use crate::{
    canonical_envelope, classify_sfs, record_digest, DifferingLedgerKindV1, Digest32,
    InterventionDeltaV1, PersistenceComparisonV1, PracticeAttemptLedgerV1,
    PracticeCandidateScheduleV1, RunIdentityField, RunIdentityV1, SfsClass, SfsPreregistrationV1,
    SfsTraceV1,
};

/// One fixture-only material sample admitted to the aligned comparator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntheticMaterialSample {
    tick: u64,
    contribution_digest: Digest32,
    aggregate_bits: u64,
}

/// Exact refusals produced by the source-bound synthetic predicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntheticDriverError {
    CandidateProjectionMismatch,
    CandidateScheduleDigestMismatch,
    AttemptLedgerDigestMismatch,
    ExogenousLedgerDigestMismatch,
    CandidateCadenceCountMismatch {
        declared: u16,
        actual: usize,
    },
    CandidateCadenceTickMismatch {
        index: usize,
        expected: u64,
        actual: u64,
    },
    CandidateCadenceOverflow {
        index: usize,
    },
    CandidateIntentCountMismatch {
        expected: usize,
        actual: usize,
    },
    CandidateIntentDigestMismatch {
        index: usize,
    },
    CandidateIntentTickMismatch {
        index: usize,
    },
    CandidatePracticeMismatch {
        index: usize,
    },
    CandidateTargetPolicyMismatch {
        index: usize,
    },
    CandidateGovernedCostMismatch {
        index: usize,
    },
    CandidateParameterBytesMismatch {
        index: usize,
    },
    TwinChangedBothLedgers,
    TwinChangedWrongLedger,
    TwinChangedNonLedgerField {
        field: RunIdentityField,
    },
    ControlTraceRunIdentityMismatch,
    InterventionTraceRunIdentityMismatch,
    ComparisonControlTraceDigestMismatch,
    ComparisonInterventionTraceDigestMismatch,
    ComparisonLedgerKindMismatch,
    ComparisonControlLedgerDigestMismatch,
    ComparisonInterventionLedgerDigestMismatch,
    ComparisonInterventionDeltaDigestMismatch,
    DriverAuthoredShape {
        driver: &'static str,
        class: SfsClass,
    },
    InvalidCumulativeDriverValue {
        driver: &'static str,
        index: usize,
        bits: u64,
    },
    CumulativeDriverDecreased {
        driver: &'static str,
        index: usize,
        previous_bits: u64,
        actual_bits: u64,
    },
    SampleLimit {
        actual: usize,
    },
    SampleCountMismatch {
        control: usize,
        aligned: usize,
    },
    MaterialSampleCountMismatch {
        expected: usize,
        control: usize,
        aligned: usize,
    },
    ArithmeticOverflow {
        field: &'static str,
    },
    TickOffsetMismatch {
        index: usize,
    },
    WrongAlignmentOffset {
        expected: u16,
        actual: u16,
    },
    AlignedMaterialMismatch {
        index: usize,
    },
    InvalidSyntheticAggregate {
        bits: u64,
    },
}

impl SyntheticMaterialSample {
    /// Seals one finite non-negative aggregate, normalizing negative zero.
    ///
    /// # Errors
    /// Returns `InvalidSyntheticAggregate` for non-finite or negative values.
    pub fn new(
        tick: u64,
        contribution_digest: Digest32,
        aggregate: f64,
    ) -> Result<Self, SyntheticDriverError> {
        if !aggregate.is_finite() || aggregate < 0.0 {
            return Err(SyntheticDriverError::InvalidSyntheticAggregate {
                bits: aggregate.to_bits(),
            });
        }
        let normalized = if aggregate == 0.0 { 0.0 } else { aggregate };
        Ok(Self {
            tick,
            contribution_digest,
            aggregate_bits: normalized.to_bits(),
        })
    }
}

pub(crate) fn validate_candidate_projection(
    run_identity: &RunIdentityV1,
    preregistration: &SfsPreregistrationV1,
    schedule: &PracticeCandidateScheduleV1,
    attempts: &PracticeAttemptLedgerV1,
    intents: &[PracticeIntentV1],
    actual_exogenous_ledger_digest: Digest32,
) -> Result<(), SyntheticDriverError> {
    let attempt_digest = digest_record(attempts)?;
    if attempt_digest != run_identity.practice_attempt_ledger_digest() {
        return Err(SyntheticDriverError::AttemptLedgerDigestMismatch);
    }
    if actual_exogenous_ledger_digest != preregistration.expected_exogenous_ledger_digest()
        || actual_exogenous_ledger_digest != run_identity.exogenous_input_ledger_digest()
    {
        return Err(SyntheticDriverError::ExogenousLedgerDigestMismatch);
    }
    if digest_record(schedule)? != preregistration.practice_candidate_schedule_digest() {
        return Err(SyntheticDriverError::CandidateScheduleDigestMismatch);
    }
    let declared = preregistration.attempt_count();
    if schedule.rows().len() != usize::from(declared) {
        return Err(SyntheticDriverError::CandidateCadenceCountMismatch {
            declared,
            actual: schedule.rows().len(),
        });
    }
    if intents.len() != schedule.rows().len() {
        return Err(SyntheticDriverError::CandidateIntentCountMismatch {
            expected: schedule.rows().len(),
            actual: intents.len(),
        });
    }
    validate_candidate_rows(preregistration, schedule, intents)?;
    let projected = attempts
        .project_candidates()
        .map_err(|_| SyntheticDriverError::CandidateProjectionMismatch)?;
    if canonical_envelope(&projected)
        .map_err(|_| SyntheticDriverError::CandidateProjectionMismatch)?
        != canonical_envelope(schedule)
            .map_err(|_| SyntheticDriverError::CandidateProjectionMismatch)?
    {
        return Err(SyntheticDriverError::CandidateProjectionMismatch);
    }
    Ok(())
}

fn validate_candidate_rows(
    preregistration: &SfsPreregistrationV1,
    schedule: &PracticeCandidateScheduleV1,
    intents: &[PracticeIntentV1],
) -> Result<(), SyntheticDriverError> {
    #[allow(clippy::needless_range_loop)]
    for index in 0..65_535 {
        if index >= schedule.rows().len() {
            break;
        }
        let stride = u64::from(preregistration.attempt_stride());
        let offset = u64::try_from(index)
            .ok()
            .and_then(|value| value.checked_mul(stride))
            .ok_or(SyntheticDriverError::CandidateCadenceOverflow { index })?;
        let expected = preregistration
            .first_attempt_tick()
            .checked_add(offset)
            .ok_or(SyntheticDriverError::CandidateCadenceOverflow { index })?;
        let actual = schedule.rows()[index].attempt_tick();
        if actual != expected {
            return Err(SyntheticDriverError::CandidateCadenceTickMismatch {
                index,
                expected,
                actual,
            });
        }
        validate_intent(
            index,
            preregistration,
            &schedule.rows()[index],
            &intents[index],
        )?;
    }
    Ok(())
}

fn validate_intent(
    index: usize,
    preregistration: &SfsPreregistrationV1,
    row: &crate::PracticeCandidateRowV1,
    intent: &PracticeIntentV1,
) -> Result<(), SyntheticDriverError> {
    let parameter_digest = parameter_bytes_digest(intent)
        .map(Digest32::from_bytes)
        .map_err(|_| SyntheticDriverError::CandidateParameterBytesMismatch { index })?;
    let complete_digest = intent_digest(intent)
        .map(Digest32::from_bytes)
        .map_err(|_| SyntheticDriverError::CandidateIntentDigestMismatch { index })?;
    if complete_digest != row.practice_intent_digest() {
        return Err(SyntheticDriverError::CandidateIntentDigestMismatch { index });
    }
    if intent.resolve_tick() != row.attempt_tick() {
        return Err(SyntheticDriverError::CandidateIntentTickMismatch { index });
    }
    if intent.practice_id() != preregistration.practice_code() {
        return Err(SyntheticDriverError::CandidatePracticeMismatch { index });
    }
    let target = Digest32::from_bytes(target_selection_policy_digest(
        intent.target_domain(),
        intent.target_node_id(),
    ));
    if target != preregistration.target_selection_policy_digest() {
        return Err(SyntheticDriverError::CandidateTargetPolicyMismatch { index });
    }
    if intent.quoted_action_budget_cost() != preregistration.governed_cost() {
        return Err(SyntheticDriverError::CandidateGovernedCostMismatch { index });
    }
    if parameter_digest != preregistration.parameter_bytes_digest() {
        return Err(SyntheticDriverError::CandidateParameterBytesMismatch { index });
    }
    Ok(())
}

pub(crate) fn validate_twin_identity_difference(
    control: &RunIdentityV1,
    intervention: &RunIdentityV1,
    selected: DifferingLedgerKindV1,
) -> Result<(), SyntheticDriverError> {
    let fields = control.differing_fields(intervention);
    let exogenous = fields.contains(&RunIdentityField::ExogenousInputLedger);
    let practice = fields.contains(&RunIdentityField::PracticeAttemptLedger);
    if exogenous && practice {
        return Err(SyntheticDriverError::TwinChangedBothLedgers);
    }
    for index in 0..18 {
        if index >= fields.len() {
            break;
        }
        let field = fields[index];
        if !matches!(
            field,
            RunIdentityField::ExogenousInputLedger | RunIdentityField::PracticeAttemptLedger
        ) {
            return Err(SyntheticDriverError::TwinChangedNonLedgerField { field });
        }
    }
    let expected = match selected {
        DifferingLedgerKindV1::ExogenousInput => exogenous && !practice,
        DifferingLedgerKindV1::PracticeAttempt => practice && !exogenous,
    };
    if !expected {
        return Err(SyntheticDriverError::TwinChangedWrongLedger);
    }
    Ok(())
}

pub(crate) fn validate_persistence_comparison_identity(
    control: &RunIdentityV1,
    intervention: &RunIdentityV1,
    control_trace: &SfsTraceV1,
    intervention_trace: &SfsTraceV1,
    comparison: &PersistenceComparisonV1,
    intervention_delta: &InterventionDeltaV1,
) -> Result<(), SyntheticDriverError> {
    if control_trace.run_identity_digest() != digest_record(control)? {
        return Err(SyntheticDriverError::ControlTraceRunIdentityMismatch);
    }
    if intervention_trace.run_identity_digest() != digest_record(intervention)? {
        return Err(SyntheticDriverError::InterventionTraceRunIdentityMismatch);
    }
    if comparison.control_trace_digest() != digest_record(control_trace)? {
        return Err(SyntheticDriverError::ComparisonControlTraceDigestMismatch);
    }
    if comparison.intervention_trace_digest() != digest_record(intervention_trace)? {
        return Err(SyntheticDriverError::ComparisonInterventionTraceDigestMismatch);
    }
    let kind = comparison.differing_ledger_kind();
    validate_twin_identity_difference(control, intervention, kind)?;
    if intervention_delta.ledger_kind() != kind {
        return Err(SyntheticDriverError::ComparisonLedgerKindMismatch);
    }
    let (control_ledger, intervention_ledger) = selected_ledgers(control, intervention, kind);
    if comparison.control_differing_ledger_digest() != control_ledger {
        return Err(SyntheticDriverError::ComparisonControlLedgerDigestMismatch);
    }
    if comparison.intervention_differing_ledger_digest() != intervention_ledger {
        return Err(SyntheticDriverError::ComparisonInterventionLedgerDigestMismatch);
    }
    if comparison.intervention_delta_digest() != digest_record(intervention_delta)? {
        return Err(SyntheticDriverError::ComparisonInterventionDeltaDigestMismatch);
    }
    Ok(())
}

fn selected_ledgers(
    control: &RunIdentityV1,
    intervention: &RunIdentityV1,
    kind: DifferingLedgerKindV1,
) -> (Digest32, Digest32) {
    match kind {
        DifferingLedgerKindV1::ExogenousInput => (
            control.exogenous_input_ledger_digest(),
            intervention.exogenous_input_ledger_digest(),
        ),
        DifferingLedgerKindV1::PracticeAttempt => (
            control.practice_attempt_ledger_digest(),
            intervention.practice_attempt_ledger_digest(),
        ),
    }
}

pub(crate) fn validate_cumulative_driver_shapes(
    window_width: u16,
    attempted_quanta: &[f64],
    governed_costs: &[f64],
) -> Result<(), SyntheticDriverError> {
    validate_cumulative_values("attempted-quanta", attempted_quanta)?;
    validate_cumulative_values("governed-costs", governed_costs)?;
    for (driver, values) in [
        ("attempted-quanta", attempted_quanta),
        ("governed-costs", governed_costs),
    ] {
        let class =
            classify_sfs(window_width, values).map_err(|_| SyntheticDriverError::SampleLimit {
                actual: values.len(),
            })?;
        if matches!(class, SfsClass::Continuing | SfsClass::LatePlateau) {
            return Err(SyntheticDriverError::DriverAuthoredShape { driver, class });
        }
    }
    Ok(())
}

fn validate_cumulative_values(
    driver: &'static str,
    values: &[f64],
) -> Result<(), SyntheticDriverError> {
    if values.len() > 157 {
        return Err(SyntheticDriverError::SampleLimit {
            actual: values.len(),
        });
    }
    for index in 0..157 {
        if index >= values.len() {
            break;
        }
        let value = values[index];
        if !value.is_finite() || value < 0.0 {
            return Err(SyntheticDriverError::InvalidCumulativeDriverValue {
                driver,
                index,
                bits: value.to_bits(),
            });
        }
        if index > 0 && value < values[index - 1] {
            return Err(SyntheticDriverError::CumulativeDriverDecreased {
                driver,
                index,
                previous_bits: values[index - 1].to_bits(),
                actual_bits: value.to_bits(),
            });
        }
    }
    Ok(())
}

pub(crate) fn validate_aligned_material(
    control: &[SyntheticMaterialSample],
    aligned: &[SyntheticMaterialSample],
    window_width: u16,
    tick_offset: u16,
) -> Result<(), SyntheticDriverError> {
    if !(2..=52).contains(&window_width) {
        return Err(SyntheticDriverError::MaterialSampleCountMismatch {
            expected: 0,
            control: control.len(),
            aligned: aligned.len(),
        });
    }
    let expected = usize::from(window_width)
        .checked_mul(3)
        .and_then(|value| value.checked_add(1))
        .ok_or(SyntheticDriverError::ArithmeticOverflow {
            field: "sample_count",
        })?;
    if control.len() > 157 || aligned.len() > 157 {
        return Err(SyntheticDriverError::SampleLimit {
            actual: control.len().max(aligned.len()),
        });
    }
    if control.len() != expected || aligned.len() != expected {
        return Err(SyntheticDriverError::MaterialSampleCountMismatch {
            expected,
            control: control.len(),
            aligned: aligned.len(),
        });
    }
    if tick_offset != window_width {
        return Err(SyntheticDriverError::WrongAlignmentOffset {
            expected: window_width,
            actual: tick_offset,
        });
    }
    compare_material_rows(control, aligned, u64::from(tick_offset))
}

fn compare_material_rows(
    control: &[SyntheticMaterialSample],
    aligned: &[SyntheticMaterialSample],
    tick_offset: u64,
) -> Result<(), SyntheticDriverError> {
    for index in 0..157 {
        if index >= control.len() {
            break;
        }
        let expected_tick = control[index].tick.checked_add(tick_offset).ok_or(
            SyntheticDriverError::ArithmeticOverflow {
                field: "aligned_tick",
            },
        )?;
        if aligned[index].tick != expected_tick {
            return Err(SyntheticDriverError::TickOffsetMismatch { index });
        }
        if aligned[index].contribution_digest != control[index].contribution_digest
            || aligned[index].aggregate_bits != control[index].aggregate_bits
        {
            return Err(SyntheticDriverError::AlignedMaterialMismatch { index });
        }
    }
    Ok(())
}

fn digest_record<T: crate::T3Record>(record: &T) -> Result<Digest32, SyntheticDriverError> {
    record_digest(record)
        .map(|digest| Digest32::from_bytes(*digest.as_bytes()))
        .map_err(|_| SyntheticDriverError::CandidateProjectionMismatch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PracticeAttemptRowV1, PracticeCandidateRowV1, PracticeDispositionV1};

    fn digest(tag: u8) -> Digest32 {
        let mut bytes = [0_u8; 32];
        bytes[0] = tag;
        Digest32::from_bytes(bytes)
    }

    #[test]
    fn both_cumulative_drivers_refuse_each_authored_shape_independently() {
        let continuing = [0.0, 1.0, 2.0, 5.0, 8.0, 10.0, 11.0];
        let late_plateau = [0.0, 1.0, 2.0, 5.0, 8.0, 8.0, 8.0];
        let constant_rate = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let other = [0.0, 3.0, 6.0, 7.0, 8.0, 10.0, 12.0];
        for (authored, class) in [
            (&continuing[..], SfsClass::Continuing),
            (&late_plateau[..], SfsClass::LatePlateau),
        ] {
            assert_eq!(
                validate_cumulative_driver_shapes(2, authored, &other),
                Err(SyntheticDriverError::DriverAuthoredShape {
                    driver: "attempted-quanta",
                    class,
                })
            );
            assert_eq!(
                validate_cumulative_driver_shapes(2, &other, authored),
                Err(SyntheticDriverError::DriverAuthoredShape {
                    driver: "governed-costs",
                    class,
                })
            );
        }
        assert_eq!(
            validate_cumulative_driver_shapes(2, &constant_rate, &other),
            Ok(())
        );
        assert_eq!(
            validate_cumulative_driver_shapes(2, &other, &constant_rate),
            Ok(())
        );
    }

    #[test]
    fn either_cumulative_driver_refuses_nonfinite_negative_and_decreasing_values() {
        let valid = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        for invalid_value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
            let mut invalid = valid;
            invalid[1] = invalid_value;
            for (attempted, governed, driver) in [
                (&invalid[..], &valid[..], "attempted-quanta"),
                (&valid[..], &invalid[..], "governed-costs"),
            ] {
                assert_eq!(
                    validate_cumulative_driver_shapes(2, attempted, governed),
                    Err(SyntheticDriverError::InvalidCumulativeDriverValue {
                        driver,
                        index: 1,
                        bits: invalid_value.to_bits(),
                    })
                );
            }
        }
        let decreasing = [0.0, 2.0, 1.0, 3.0, 4.0, 5.0, 6.0];
        for (attempted, governed, driver) in [
            (&decreasing[..], &valid[..], "attempted-quanta"),
            (&valid[..], &decreasing[..], "governed-costs"),
        ] {
            assert_eq!(
                validate_cumulative_driver_shapes(2, attempted, governed),
                Err(SyntheticDriverError::CumulativeDriverDecreased {
                    driver,
                    index: 2,
                    previous_bits: 2.0_f64.to_bits(),
                    actual_bits: 1.0_f64.to_bits(),
                })
            );
        }
    }

    #[test]
    fn aligned_material_bits_match() {
        let control = (0_u8..7)
            .map(|index| {
                SyntheticMaterialSample::new(u64::from(index), digest(index), f64::from(index))
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let aligned = (0_u8..7)
            .map(|index| {
                SyntheticMaterialSample::new(u64::from(index) + 2, digest(index), f64::from(index))
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(validate_aligned_material(&control, &aligned, 2, 2), Ok(()));
        assert_eq!(
            SyntheticMaterialSample::new(0, digest(0), -0.0)
                .unwrap()
                .aggregate_bits,
            0
        );
    }

    #[test]
    fn aligned_material_length_offset_and_bits_are_exact() {
        let width_one = (0_u8..4)
            .map(|index| {
                SyntheticMaterialSample::new(u64::from(index), digest(1), f64::from(index))
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(matches!(
            validate_aligned_material(&width_one, &width_one, 1, 1),
            Err(SyntheticDriverError::MaterialSampleCountMismatch { expected: 0, .. })
        ));
        assert!(matches!(
            validate_aligned_material(&[], &[], 2, 2),
            Err(SyntheticDriverError::MaterialSampleCountMismatch { expected: 7, .. })
        ));
        let too_short = (0..6)
            .map(|index| SyntheticMaterialSample::new(index, digest(1), 1.0))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            validate_aligned_material(&too_short, &too_short, 2, 2),
            Err(SyntheticDriverError::MaterialSampleCountMismatch {
                expected: 7,
                control: 6,
                aligned: 6,
            })
        );
        let too_long = (0..8)
            .map(|index| SyntheticMaterialSample::new(index, digest(1), 1.0))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            validate_aligned_material(&too_long, &too_long, 2, 2),
            Err(SyntheticDriverError::MaterialSampleCountMismatch {
                expected: 7,
                control: 8,
                aligned: 8,
            })
        );
        let control = (0..7)
            .map(|index| SyntheticMaterialSample::new(index, digest(1), 1.0))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let mut aligned = (0..7)
            .map(|index| SyntheticMaterialSample::new(index + 2, digest(1), 1.0))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            validate_aligned_material(&control, &aligned, 2, 1),
            Err(SyntheticDriverError::WrongAlignmentOffset {
                expected: 2,
                actual: 1
            })
        );
        aligned[3].aggregate_bits = 2.0_f64.to_bits();
        assert_eq!(
            validate_aligned_material(&control, &aligned, 2, 2),
            Err(SyntheticDriverError::AlignedMaterialMismatch { index: 3 })
        );
    }

    #[test]
    fn aggregate_constructor_rejects_all_invalid_values() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
            assert!(matches!(
                SyntheticMaterialSample::new(0, digest(1), value),
                Err(SyntheticDriverError::InvalidSyntheticAggregate { .. })
            ));
        }
    }

    #[test]
    fn input_permutations_preserve_bytes() {
        let first = PracticeCandidateRowV1::new(10, digest(2), digest(3));
        let second = PracticeCandidateRowV1::new(12, digest(4), digest(5));
        let left = PracticeCandidateScheduleV1::new(vec![first.clone(), second.clone()]).unwrap();
        let right = PracticeCandidateScheduleV1::new(vec![second.clone(), first.clone()]).unwrap();
        assert_eq!(
            canonical_envelope(&left).unwrap(),
            canonical_envelope(&right).unwrap()
        );
        let left_attempts = PracticeAttemptLedgerV1::new(
            digest(6),
            vec![
                PracticeAttemptRowV1::new(
                    first.clone(),
                    PracticeDispositionV1::Rejected,
                    digest(7),
                )
                .unwrap(),
                PracticeAttemptRowV1::new(
                    second.clone(),
                    PracticeDispositionV1::Rejected,
                    digest(8),
                )
                .unwrap(),
            ],
        )
        .unwrap();
        let right_attempts = PracticeAttemptLedgerV1::new(
            digest(6),
            vec![
                PracticeAttemptRowV1::new(second, PracticeDispositionV1::Rejected, digest(8))
                    .unwrap(),
                PracticeAttemptRowV1::new(first, PracticeDispositionV1::Rejected, digest(7))
                    .unwrap(),
            ],
        )
        .unwrap();
        assert_eq!(
            canonical_envelope(&left_attempts).unwrap(),
            canonical_envelope(&right_attempts).unwrap()
        );
    }

    #[test]
    fn semantic_row_mutation_moves_bytes_and_digest() {
        let original = PracticeCandidateScheduleV1::new(vec![PracticeCandidateRowV1::new(
            10,
            digest(2),
            digest(3),
        )])
        .unwrap();
        let changed = PracticeCandidateScheduleV1::new(vec![PracticeCandidateRowV1::new(
            11,
            digest(2),
            digest(3),
        )])
        .unwrap();
        assert_ne!(
            canonical_envelope(&original).unwrap(),
            canonical_envelope(&changed).unwrap()
        );
        assert_ne!(
            record_digest(&original).unwrap(),
            record_digest(&changed).unwrap()
        );
    }
}
