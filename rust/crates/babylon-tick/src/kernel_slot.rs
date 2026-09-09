//! Runtime-owned permanent finite-kernel sample/slot reservations.
//!
//! Packaging layers may deserialize these rows from their own manifests, but
//! the executable preparation boundary owns their canonical shape and the
//! exact match between a scheduled `FiniteKernelV1` and its permanent row.

use std::collections::BTreeMap;

use babylon_bsl::probability::FiniteKernelV1;
use babylon_bsl::{read, Atom, SExpr};

/// One borrowed permanent kernel-slot reservation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelSlotReservationV1<'a> {
    /// Continuous global append-only position.
    pub ordinal: u32,
    /// Governed mechanic rule `QName`.
    pub rule: &'a str,
    /// Stable sample `QName`.
    pub sample: &'a str,
    /// Rule-local append-only slot.
    pub slot: u32,
}

/// The built-in ledger compiled into the canonical runtime preparation path.
/// Rows are never removed or reordered; genuinely new kernels append here and
/// in `content/content-sets.toml` together.
pub const BUNDLED_KERNEL_SLOT_RESERVATIONS_V1: &[KernelSlotReservationV1<'static>] =
    &[KernelSlotReservationV1 {
        ordinal: 0,
        rule: "struggle/spark-mechanic",
        sample: "struggle/spark",
        slot: 0,
    }];

/// Structural or live-kernel refusal from the permanent ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelSlotLedgerErrorV1 {
    /// A document position could not be represented by the governed ordinal.
    OrdinalCapacity { position: usize },
    /// A row ordinal did not equal its zero-based document position.
    Ordinal {
        position: usize,
        expected: u32,
        actual: u32,
    },
    /// The exact same reservation appeared twice.
    Collision {
        rule: String,
        slot: u32,
        sample: String,
        first_ordinal: u32,
        duplicate_ordinal: u32,
    },
    /// A `(rule, slot)` key was rebound to another sample.
    Rebind {
        rule: String,
        slot: u32,
        existing_sample: String,
        replacement_sample: String,
        first_ordinal: u32,
        replacement_ordinal: u32,
    },
    /// A stable sample moved to another rule or slot.
    SampleCollision {
        sample: String,
        existing_rule: String,
        existing_slot: u32,
        first_ordinal: u32,
        replacement_rule: String,
        replacement_slot: u32,
        replacement_ordinal: u32,
    },
    /// One rule's reservations did not form `0..N` in ledger order.
    RuleSlotSequence {
        rule: String,
        expected: u64,
        actual: u32,
        ordinal: u32,
    },
    /// A rule or sample was not one canonical `QName` token.
    InvalidQName {
        ordinal: u32,
        field: &'static str,
        value: String,
    },
    /// A live kernel had no permanent reservation.
    MissingLiveReservation {
        rule: String,
        sample: String,
        slot: u32,
    },
    /// A live kernel changed the sample bound to its permanent rule/slot.
    LiveSampleMismatch {
        rule: String,
        slot: u32,
        expected_sample: String,
        actual_sample: String,
        ordinal: u32,
    },
    /// A live kernel changed the slot bound to its permanent rule/sample.
    LiveSlotMismatch {
        rule: String,
        sample: String,
        expected_slot: u32,
        actual_slot: u32,
        ordinal: u32,
    },
    /// A live kernel attempted to use a sample permanently owned elsewhere.
    LiveSampleMoved {
        sample: String,
        expected_rule: String,
        expected_slot: u32,
        actual_rule: String,
        actual_slot: u32,
        ordinal: u32,
    },
}

impl std::fmt::Display for KernelSlotLedgerErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OrdinalCapacity { position } => write!(
                formatter,
                "kernel-slot row at position {position} exceeds the u32 ordinal space"
            ),
            Self::Ordinal {
                position,
                expected,
                actual,
            } => write!(
                formatter,
                "kernel-slot row at position {position} has ordinal {actual}; expected {expected}"
            ),
            Self::Collision {
                rule,
                slot,
                sample,
                first_ordinal,
                duplicate_ordinal,
            } => write!(
                formatter,
                "kernel-slot `{rule}` slot {slot} sample `{sample}` repeats ordinal {first_ordinal} at ordinal {duplicate_ordinal}"
            ),
            Self::Rebind {
                rule,
                slot,
                existing_sample,
                replacement_sample,
                first_ordinal,
                replacement_ordinal,
            } => write!(
                formatter,
                "kernel-slot `{rule}` slot {slot} at ordinal {first_ordinal} binds `{existing_sample}` and cannot be rebound to `{replacement_sample}` at ordinal {replacement_ordinal}"
            ),
            Self::SampleCollision {
                sample,
                existing_rule,
                existing_slot,
                first_ordinal,
                replacement_rule,
                replacement_slot,
                replacement_ordinal,
            } => write!(
                formatter,
                "kernel-slot sample `{sample}` belongs to `{existing_rule}` slot {existing_slot} at ordinal {first_ordinal} and cannot move to `{replacement_rule}` slot {replacement_slot} at ordinal {replacement_ordinal}"
            ),
            Self::RuleSlotSequence {
                rule,
                expected,
                actual,
                ordinal,
            } => write!(
                formatter,
                "kernel-slot reservation ordinal {ordinal} gives `{rule}` slot {actual}; expected append-only slot {expected}"
            ),
            Self::InvalidQName {
                ordinal,
                field,
                value,
            } => write!(
                formatter,
                "kernel-slot reservation ordinal {ordinal} has non-QName {field} `{value}`"
            ),
            Self::MissingLiveReservation { rule, sample, slot } => write!(
                formatter,
                "finite kernel `{rule}` sample `{sample}` slot {slot} has no permanent reservation"
            ),
            Self::LiveSampleMismatch {
                rule,
                slot,
                expected_sample,
                actual_sample,
                ordinal,
            } => write!(
                formatter,
                "finite kernel `{rule}` slot {slot} must retain sample `{expected_sample}` from reservation ordinal {ordinal}, not `{actual_sample}`"
            ),
            Self::LiveSlotMismatch {
                rule,
                sample,
                expected_slot,
                actual_slot,
                ordinal,
            } => write!(
                formatter,
                "finite kernel `{rule}` sample `{sample}` must retain slot {expected_slot} from reservation ordinal {ordinal}, not {actual_slot}"
            ),
            Self::LiveSampleMoved {
                sample,
                expected_rule,
                expected_slot,
                actual_rule,
                actual_slot,
                ordinal,
            } => write!(
                formatter,
                "finite kernel sample `{sample}` belongs to `{expected_rule}` slot {expected_slot} at reservation ordinal {ordinal} and cannot move to `{actual_rule}` slot {actual_slot}"
            ),
        }
    }
}

impl std::error::Error for KernelSlotLedgerErrorV1 {}

/// Exact relationship between one live kernel and a validated ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelSlotReservationMatchV1<'a> {
    Exact,
    Missing,
    SampleMismatch {
        reservation: KernelSlotReservationV1<'a>,
    },
    SlotMismatch {
        reservation: KernelSlotReservationV1<'a>,
    },
    SampleMoved {
        reservation: KernelSlotReservationV1<'a>,
    },
}

/// Validate one complete append-only reservation ledger.
pub fn validate_kernel_slot_ledger_v1(
    reservations: &[KernelSlotReservationV1<'_>],
) -> Result<(), KernelSlotLedgerErrorV1> {
    let mut bindings: BTreeMap<(String, u32), (String, u32)> = BTreeMap::new();
    let mut samples: BTreeMap<String, (String, u32, u32)> = BTreeMap::new();
    let mut next_slot_by_rule: BTreeMap<String, u64> = BTreeMap::new();
    for (position, reservation) in reservations.iter().enumerate() {
        let expected = u32::try_from(position)
            .map_err(|_| KernelSlotLedgerErrorV1::OrdinalCapacity { position })?;
        if reservation.ordinal != expected {
            return Err(KernelSlotLedgerErrorV1::Ordinal {
                position,
                expected,
                actual: reservation.ordinal,
            });
        }
        for (field, value) in [("rule", reservation.rule), ("sample", reservation.sample)] {
            if !is_canonical_qname(value) {
                return Err(KernelSlotLedgerErrorV1::InvalidQName {
                    ordinal: reservation.ordinal,
                    field,
                    value: value.to_owned(),
                });
            }
        }
        let key = (reservation.rule.to_owned(), reservation.slot);
        if let Some((existing_sample, first_ordinal)) = bindings.get(&key) {
            if existing_sample == reservation.sample {
                return Err(KernelSlotLedgerErrorV1::Collision {
                    rule: reservation.rule.to_owned(),
                    slot: reservation.slot,
                    sample: reservation.sample.to_owned(),
                    first_ordinal: *first_ordinal,
                    duplicate_ordinal: reservation.ordinal,
                });
            }
            return Err(KernelSlotLedgerErrorV1::Rebind {
                rule: reservation.rule.to_owned(),
                slot: reservation.slot,
                existing_sample: existing_sample.clone(),
                replacement_sample: reservation.sample.to_owned(),
                first_ordinal: *first_ordinal,
                replacement_ordinal: reservation.ordinal,
            });
        }
        if let Some((existing_rule, existing_slot, first_ordinal)) = samples.get(reservation.sample)
        {
            return Err(KernelSlotLedgerErrorV1::SampleCollision {
                sample: reservation.sample.to_owned(),
                existing_rule: existing_rule.clone(),
                existing_slot: *existing_slot,
                first_ordinal: *first_ordinal,
                replacement_rule: reservation.rule.to_owned(),
                replacement_slot: reservation.slot,
                replacement_ordinal: reservation.ordinal,
            });
        }
        let expected_slot = next_slot_by_rule
            .get(reservation.rule)
            .copied()
            .unwrap_or(0);
        if u64::from(reservation.slot) != expected_slot {
            return Err(KernelSlotLedgerErrorV1::RuleSlotSequence {
                rule: reservation.rule.to_owned(),
                expected: expected_slot,
                actual: reservation.slot,
                ordinal: reservation.ordinal,
            });
        }
        bindings.insert(key, (reservation.sample.to_owned(), reservation.ordinal));
        samples.insert(
            reservation.sample.to_owned(),
            (
                reservation.rule.to_owned(),
                reservation.slot,
                reservation.ordinal,
            ),
        );
        next_slot_by_rule.insert(reservation.rule.to_owned(), expected_slot + 1);
    }
    Ok(())
}

/// Match one typed kernel against a structurally validated ledger.
#[must_use]
pub fn match_kernel_slot_reservation_v1<'a>(
    reservations: &[KernelSlotReservationV1<'a>],
    rule: &str,
    sample: &str,
    slot: u32,
) -> KernelSlotReservationMatchV1<'a> {
    if let Some(reservation) = reservations
        .iter()
        .find(|reservation| reservation.rule == rule && reservation.slot == slot)
    {
        return if reservation.sample == sample {
            KernelSlotReservationMatchV1::Exact
        } else {
            KernelSlotReservationMatchV1::SampleMismatch {
                reservation: *reservation,
            }
        };
    }
    if let Some(reservation) = reservations
        .iter()
        .find(|reservation| reservation.rule == rule && reservation.sample == sample)
    {
        return KernelSlotReservationMatchV1::SlotMismatch {
            reservation: *reservation,
        };
    }
    reservations
        .iter()
        .find(|reservation| reservation.sample == sample)
        .map_or(KernelSlotReservationMatchV1::Missing, |reservation| {
            KernelSlotReservationMatchV1::SampleMoved {
                reservation: *reservation,
            }
        })
}

/// Validate every live typed kernel against the permanent ledger. Historical
/// reservations without a live kernel remain legal tombstones.
pub fn validate_live_kernel_slots_v1(
    reservations: &[KernelSlotReservationV1<'_>],
    kernels: &[(&str, &FiniteKernelV1)],
) -> Result<(), KernelSlotLedgerErrorV1> {
    validate_kernel_slot_ledger_v1(reservations)?;
    for (rule, kernel) in kernels {
        match match_kernel_slot_reservation_v1(reservations, rule, &kernel.sample, kernel.slot) {
            KernelSlotReservationMatchV1::Exact => {}
            KernelSlotReservationMatchV1::Missing => {
                return Err(KernelSlotLedgerErrorV1::MissingLiveReservation {
                    rule: (*rule).to_owned(),
                    sample: kernel.sample.clone(),
                    slot: kernel.slot,
                });
            }
            KernelSlotReservationMatchV1::SampleMismatch { reservation } => {
                return Err(KernelSlotLedgerErrorV1::LiveSampleMismatch {
                    rule: (*rule).to_owned(),
                    slot: kernel.slot,
                    expected_sample: reservation.sample.to_owned(),
                    actual_sample: kernel.sample.clone(),
                    ordinal: reservation.ordinal,
                });
            }
            KernelSlotReservationMatchV1::SlotMismatch { reservation } => {
                return Err(KernelSlotLedgerErrorV1::LiveSlotMismatch {
                    rule: (*rule).to_owned(),
                    sample: kernel.sample.clone(),
                    expected_slot: reservation.slot,
                    actual_slot: kernel.slot,
                    ordinal: reservation.ordinal,
                });
            }
            KernelSlotReservationMatchV1::SampleMoved { reservation } => {
                return Err(KernelSlotLedgerErrorV1::LiveSampleMoved {
                    sample: kernel.sample.clone(),
                    expected_rule: reservation.rule.to_owned(),
                    expected_slot: reservation.slot,
                    actual_rule: (*rule).to_owned(),
                    actual_slot: kernel.slot,
                    ordinal: reservation.ordinal,
                });
            }
        }
    }
    Ok(())
}

fn is_canonical_qname(value: &str) -> bool {
    matches!(
        read(value),
        Ok((SExpr::Atom(Atom::QName(parsed)), consumed))
            if consumed == value.len() && parsed == value
    )
}

#[cfg(test)]
mod tests {
    use super::{
        match_kernel_slot_reservation_v1, validate_kernel_slot_ledger_v1, KernelSlotLedgerErrorV1,
        KernelSlotReservationMatchV1, KernelSlotReservationV1,
    };

    const FIRST: KernelSlotReservationV1<'static> = KernelSlotReservationV1 {
        ordinal: 0,
        rule: "struggle/spark-mechanic",
        sample: "struggle/spark",
        slot: 0,
    };

    #[test]
    fn retained_historical_rows_and_exact_live_matches_are_legal() {
        let rows = [FIRST];
        validate_kernel_slot_ledger_v1(&rows).unwrap();
        assert_eq!(
            match_kernel_slot_reservation_v1(&rows, "struggle/spark-mechanic", "struggle/spark", 0),
            KernelSlotReservationMatchV1::Exact
        );
    }

    #[test]
    fn deletion_reorder_gap_rebind_and_sample_move_are_typed() {
        let ordinal_gap = [KernelSlotReservationV1 {
            ordinal: 1,
            ..FIRST
        }];
        assert!(matches!(
            validate_kernel_slot_ledger_v1(&ordinal_gap),
            Err(KernelSlotLedgerErrorV1::Ordinal { .. })
        ));

        let slot_gap = [KernelSlotReservationV1 { slot: 1, ..FIRST }];
        assert!(matches!(
            validate_kernel_slot_ledger_v1(&slot_gap),
            Err(KernelSlotLedgerErrorV1::RuleSlotSequence { .. })
        ));

        let rebind = [
            FIRST,
            KernelSlotReservationV1 {
                ordinal: 1,
                sample: "struggle/rebound",
                ..FIRST
            },
        ];
        assert!(matches!(
            validate_kernel_slot_ledger_v1(&rebind),
            Err(KernelSlotLedgerErrorV1::Rebind { .. })
        ));

        let moved = [
            FIRST,
            KernelSlotReservationV1 {
                ordinal: 1,
                rule: "struggle/other",
                sample: FIRST.sample,
                ..FIRST
            },
        ];
        assert!(matches!(
            validate_kernel_slot_ledger_v1(&moved),
            Err(KernelSlotLedgerErrorV1::SampleCollision { .. })
        ));
    }
}
