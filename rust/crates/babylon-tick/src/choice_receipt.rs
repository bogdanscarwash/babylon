//! Transaction-owned evidence for one realized finite material choice.
//!
//! `AuditReceipt` deliberately stays identity-free. A `ChoiceReceiptV1`
//! instead retains the exact replay-keyed realization needed to prove which
//! bounded material alternative was selected. The tick transaction assigns
//! the encounter ordinal and publishes this value only with the rest of a
//! successful tick.

use babylon_bsl::probability::{
    validate_kernel_realization, KernelInstanceIdentityV1, KernelRealizationV1, ProbabilityError,
    RealizedBranchV1,
};
use babylon_graph::stable_element::StableElementKeyV1;

/// Reference from one committed event to the finite choice it projects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChoiceReceiptRefV1 {
    pub(crate) encounter_ordinal: u32,
}

impl ChoiceReceiptRefV1 {
    /// Construct a reference to one tick-local receipt ordinal.
    #[must_use]
    pub const fn new(encounter_ordinal: u32) -> Self {
        Self { encounter_ordinal }
    }

    /// Return the tick-wide encounter ordinal of the referenced receipt.
    #[must_use]
    pub const fn encounter_ordinal(self) -> u32 {
        self.encounter_ordinal
    }
}

/// Exact evidence for one successful finite-kernel realization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceReceiptV1 {
    encounter_ordinal: u32,
    realization: KernelRealizationV1,
}

impl ChoiceReceiptV1 {
    /// Validate and own one successful realization at its tick-wide ordinal.
    ///
    /// # Errors
    /// Returns the realization validator's first exact allocation, selection,
    /// instance-identity, or digest mismatch.
    pub fn try_new(
        encounter_ordinal: u32,
        identity: &KernelInstanceIdentityV1,
        realization: KernelRealizationV1,
    ) -> Result<Self, ProbabilityError> {
        validate_kernel_realization(&realization, identity)?;
        Ok(Self {
            encounter_ordinal,
            realization,
        })
    }

    /// Return the tick-wide continuous encounter ordinal.
    #[must_use]
    pub const fn encounter_ordinal(&self) -> u32 {
        self.encounter_ordinal
    }

    /// Return the stable reference used by a finite-projection event.
    #[must_use]
    pub const fn reference(&self) -> ChoiceReceiptRefV1 {
        ChoiceReceiptRefV1::new(self.encounter_ordinal)
    }

    /// Borrow the firing rule identity.
    #[must_use]
    pub fn rule_id(&self) -> &str {
        &self.realization.rule_id
    }

    /// Borrow the content-set-unique sample identity.
    #[must_use]
    pub fn sample(&self) -> &str {
        &self.realization.sample
    }

    /// Return the append-only authored slot.
    #[must_use]
    pub const fn slot(&self) -> u32 {
        self.realization.slot
    }

    /// Borrow the common branch enum type.
    #[must_use]
    pub fn outcome_enum(&self) -> &str {
        &self.realization.enum_type
    }

    /// Borrow the stable carrier subject.
    #[must_use]
    pub const fn stable_carrier(&self) -> &StableElementKeyV1 {
        &self.realization.subject
    }

    /// Borrow ordered active-element stable identities.
    #[must_use]
    pub fn active_elements(&self) -> &[StableElementKeyV1] {
        &self.realization.active_elements
    }

    /// Borrow enum-ordered exact masses and ticket intervals.
    #[must_use]
    pub fn branches(&self) -> &[RealizedBranchV1] {
        &self.realization.branches
    }

    /// Return the one private draw ticket.
    #[must_use]
    pub const fn draw_ticket(&self) -> u64 {
        self.realization.draw
    }

    /// Borrow the selected outcome member.
    #[must_use]
    pub fn selected_outcome(&self) -> &str {
        &self.realization.selected_outcome
    }

    /// Return the exact allocation digest.
    #[must_use]
    pub const fn allocation_digest(&self) -> [u8; 32] {
        self.realization.allocation_digest
    }

    /// Return the replay-keyed instance digest.
    #[must_use]
    pub const fn instance_digest(&self) -> [u8; 32] {
        self.realization.instance_digest
    }

    /// Borrow the validated engine-neutral realization.
    #[must_use]
    pub const fn realization(&self) -> &KernelRealizationV1 {
        &self.realization
    }
}

/// Validate that receipt ordinals are exactly `0..N` in publication order.
///
/// # Errors
/// Returns the first received ordinal that differs from its zero-based
/// position. This check is shared by tick-payload encoding and persistence
/// reconstruction so neither silently accepts a sparse or reordered ledger.
pub fn validate_choice_receipt_order(receipts: &[ChoiceReceiptV1]) -> Result<(), u32> {
    for (expected, receipt) in receipts.iter().enumerate() {
        let expected = u32::try_from(expected).map_err(|_| u32::MAX)?;
        if receipt.encounter_ordinal != expected {
            return Err(receipt.encounter_ordinal);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use babylon_bsl::probability::{realize_kernel, FiniteKernelV1, KernelBranchV1, Mass};
    use babylon_bsl::reader::{Atom, SExpr};
    use babylon_bsl::types::EnumTypeId;

    fn stable_node(local_name: &str) -> StableElementKeyV1 {
        StableElementKeyV1::Node {
            scenario: "pilot/struggle".to_owned(),
            local_name: local_name.to_owned(),
        }
    }

    fn identity() -> KernelInstanceIdentityV1 {
        KernelInstanceIdentityV1 {
            replay_session: b"campaign/session".to_vec(),
            replay_seed: 17_i64.to_be_bytes(),
            tick: 3,
            rule_id: "struggle/spark-mechanic".to_owned(),
            subject: stable_node("worker"),
            active_elements: vec![stable_node("detroit")],
        }
    }

    fn kernel() -> FiniteKernelV1 {
        FiniteKernelV1 {
            sample: "struggle/spark".to_owned(),
            sample_path: vec![0, 1, 2],
            slot: 0,
            slot_path: vec![0, 1, 4],
            enum_type: EnumTypeId(0),
            enum_type_name: "StruggleSparkOutcome".to_owned(),
            branches: ["EXCESSIVE_FORCE", "NO_INCIDENT"]
                .into_iter()
                .enumerate()
                .map(|(ordinal, member)| KernelBranchV1 {
                    enum_type: "StruggleSparkOutcome".to_owned(),
                    member: member.to_owned(),
                    ordinal: u32::try_from(ordinal).expect("two branches"),
                    mass: SExpr::Atom(Atom::Mass(Mass::from_nanounits(1))),
                    effects: Vec::new(),
                    form_path: vec![0, 1, u32::try_from(ordinal).expect("two branches")],
                    head_path: vec![0, 1, u32::try_from(ordinal).expect("two branches"), 0],
                    mass_path: vec![0, 1, u32::try_from(ordinal).expect("two branches"), 3],
                    mass_literals: Vec::new(),
                    quantize_mass_paths: Vec::new(),
                    static_mass: Some(Mass::from_nanounits(1)),
                })
                .collect(),
            form_path: vec![0, 1],
            head_path: vec![0, 1, 0],
        }
    }

    fn receipt(ordinal: u32) -> ChoiceReceiptV1 {
        let identity = identity();
        let realization = realize_kernel(
            &identity,
            &kernel(),
            &[Mass::from_nanounits(1), Mass::from_nanounits(1)],
            0,
        )
        .expect("valid realization");
        ChoiceReceiptV1::try_new(ordinal, &identity, realization).expect("valid receipt")
    }

    #[test]
    fn receipt_retains_every_exact_realization_fact() {
        let receipt = receipt(7);
        assert_eq!(receipt.encounter_ordinal(), 7);
        assert_eq!(receipt.reference().encounter_ordinal(), 7);
        assert_eq!(receipt.rule_id(), "struggle/spark-mechanic");
        assert_eq!(receipt.sample(), "struggle/spark");
        assert_eq!(receipt.slot(), 0);
        assert_eq!(receipt.outcome_enum(), "StruggleSparkOutcome");
        assert_eq!(receipt.stable_carrier(), &stable_node("worker"));
        assert_eq!(receipt.active_elements(), [stable_node("detroit")]);
        assert_eq!(receipt.branches().len(), 2);
        assert_eq!(receipt.draw_ticket(), 0);
        assert_eq!(receipt.selected_outcome(), "EXCESSIVE_FORCE");
    }

    #[test]
    fn receipt_refuses_a_mutated_realization_digest() {
        let identity = identity();
        let mut realization = realize_kernel(
            &identity,
            &kernel(),
            &[Mass::from_nanounits(1), Mass::from_nanounits(1)],
            0,
        )
        .expect("valid realization");
        realization.instance_digest[0] ^= 0xff;
        assert!(ChoiceReceiptV1::try_new(0, &identity, realization).is_err());
    }

    #[test]
    fn receipt_order_is_zero_based_continuous_and_stable() {
        assert_eq!(
            validate_choice_receipt_order(&[receipt(0), receipt(1)]),
            Ok(())
        );
        assert_eq!(
            validate_choice_receipt_order(&[receipt(0), receipt(2)]),
            Err(2)
        );
    }
}
