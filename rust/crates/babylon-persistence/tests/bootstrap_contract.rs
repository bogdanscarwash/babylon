//! Static, DB-free contract for the sole H3 reader bootstrap composition.

use babylon_persistence::{
    bootstrap_h3_reader_epoch_v1, representative_h3_reference_cohort_v1, H3ReaderBootstrapErrorV1,
    H3ReaderBootstrapReportV1,
};

#[test]
fn production_cohort_accessor_has_the_exact_governed_receipt() {
    let cohort = representative_h3_reference_cohort_v1()
        .expect("the checked-in production cohort must validate");
    let receipt = cohort.receipt();

    assert_eq!(cohort.rows().len(), 59_849);
    assert_eq!(receipt.source_cell_count(), 48_764);
    assert_eq!(
        receipt.source_digest().to_hex(),
        "a4685e6ad882930e7064cb225ee649155fb74e52ef8b7d7550691a70a6087f5a"
    );
    assert_eq!(
        receipt.ref_digest().to_hex(),
        "92b21ff325bde67f26565f52882d3664daacd6d51423f2a588344da012fd4161"
    );
}

#[test]
fn runtime_cli_names_the_one_public_activation_composition() {
    let cli = include_str!("../src/bin/babylon-runtime.rs");
    let bootstrap = include_str!("../src/bootstrap.rs");

    assert!(cli.contains("activate_rust_persistence_v2(config)"));
    assert!(bootstrap.contains("pub fn bootstrap_h3_reader_epoch_v1"));
    let _: fn(&postgres::Config) -> Result<H3ReaderBootstrapReportV1, H3ReaderBootstrapErrorV1> =
        bootstrap_h3_reader_epoch_v1;
}
