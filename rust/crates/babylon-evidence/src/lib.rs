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
    bind_synthetic_driver, parse_synthetic_driver_contract, SyntheticDriverContractError,
    SyntheticDriverContractV1, ValidatedSyntheticDriver,
};
pub use profile::{
    CanonicalProfileSet, CausalConeV1, ComponentKindV1, DifferingLedgerKindV1,
    InterventionDeltaRowV1, InterventionDeltaV1, InterventionOperationV1, PersistenceComparisonV1,
    SfsComponentProofProfileV1, SfsProfileRecordError, SfsProofProfileV1,
};
pub use records::{
    practice_attempt_row_id, PracticeAttemptLedgerV1, PracticeAttemptRowV1, PracticeCandidateRowV1,
    PracticeCandidateScheduleV1, PracticeDispositionV1, RunIdentityField, RunIdentityV1,
    SfsPreregistrationV1, SfsRecordError, SfsSampleV1, SfsTraceV1,
};
pub use validation::{
    component_profile_from_bsl, parse_synthetic_governed_manifest, validate_synthetic_cone,
    validate_synthetic_mutation_manifest, validate_synthetic_profile_identity,
    ProducerConsumerEdgeV1, SfsValidationError, SyntheticChannelKindV1,
    SyntheticGovernedComponentV1, SyntheticGovernedManifestV1,
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
