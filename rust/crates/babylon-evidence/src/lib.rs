//! Post-commit evidence contracts that never feed engine judgment.
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

mod classifier;
mod digest;
mod driver;
mod driver_contract;
mod profile;
mod records;
mod validation;
mod wire;

pub use classifier::{
    classify_persistence, classify_sfs, PersistenceClass, PersistenceClassError, SfsClass,
    SfsClassError,
};
pub use digest::{record_digest, Digest32, RecordDigest};
pub use driver::{SyntheticDriverError, SyntheticMaterialSample};
pub use driver_contract::{
    bind_synthetic_driver, parse_synthetic_driver_contract, SyntheticDriverContract,
    SyntheticDriverContractError, ValidatedSyntheticDriver,
};
pub use profile::{
    CanonicalProfileSet, CausalCone, ComponentKind, DifferingLedgerKind, InterventionDelta,
    InterventionDeltaRow, InterventionOperation, PersistenceComparison, SfsComponentProofProfile,
    SfsProfileRecordError, SfsProofProfile,
};
pub use records::{
    practice_attempt_row_id, PracticeAttemptLedger, PracticeAttemptRow, PracticeCandidateRow,
    PracticeCandidateSchedule, PracticeDisposition, RunIdentity, RunIdentityField,
    SfsPreregistration, SfsRecordError, SfsSample, SfsTrace,
};
pub use validation::{
    component_profile_from_bsl, parse_synthetic_governed_manifest, validate_synthetic_cone,
    validate_synthetic_mutation_manifest, validate_synthetic_profile_identity,
    ProducerConsumerEdge, SfsValidationError, SyntheticChannelKind, SyntheticGovernedComponent,
    SyntheticGovernedManifest,
};
pub use wire::{
    canonical_envelope, decode_envelope, PayloadCursor, PayloadEncoder, SfsWireError, T3Record,
};

#[cfg(test)]
mod tests {
    #[test]
    fn shared_normalizer_exports_the_pinned_unicode_data_version() {
        assert_eq!(unicode_normalization::UNICODE_VERSION, (17, 0, 0));
    }
}
