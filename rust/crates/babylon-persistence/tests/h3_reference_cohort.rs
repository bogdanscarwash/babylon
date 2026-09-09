//! Language-neutral representative H3 reference-cohort behavior.

use babylon_kernel::tick_content_hash::RefDigestV1;
use babylon_kernel::H3CellId;
use babylon_persistence::{
    build_representative_h3_cohort_v1, representative_h3_reference_cohort_v1, H3ReferenceCellRow,
    H3ReferenceCohortError, H3ReferenceOrigin, MAX_H3_REFERENCE_SOURCE_CELLS,
};
use std::str::FromStr;

const SOURCE_COUNT: usize = 48_764;
const CLOSURE_COUNT: usize = 59_849;
const SOURCE_COUNTS: [u32; 16] = [0, 0, 0, 0, 0, 3_192, 0, 45_572, 0, 0, 0, 0, 0, 0, 0, 0];
const CLOSURE_COUNTS: [u32; 16] = [
    11, 36, 134, 674, 2_634, 4_121, 6_667, 45_572, 0, 0, 0, 0, 0, 0, 0, 0,
];

#[test]
fn representative_cohort_is_permutation_stable_and_matches_independent_receipts() {
    let source = source_cells();
    let forward = build_representative_h3_cohort_v1(artifact_digest(), &source)
        .expect("pinned representative source should build");
    let reversed_source = source.iter().copied().rev().collect::<Vec<_>>();
    let reversed = build_representative_h3_cohort_v1(artifact_digest(), &reversed_source)
        .expect("input order must not change the cohort");

    assert_eq!(forward, reversed);
    assert_eq!(forward.rows().len(), CLOSURE_COUNT);
    assert_eq!(forward.receipt().artifact_digest(), artifact_digest());
    assert_eq!(forward.receipt().source_cell_count(), SOURCE_COUNT);
    assert_eq!(
        forward.receipt().source_counts_by_resolution(),
        &SOURCE_COUNTS
    );
    assert_eq!(
        forward.receipt().closure_counts_by_resolution(),
        &CLOSURE_COUNTS
    );
    assert_eq!(
        forward.receipt().source_digest().to_hex(),
        "a4685e6ad882930e7064cb225ee649155fb74e52ef8b7d7550691a70a6087f5a"
    );
    assert_eq!(
        forward.receipt().source_r5_digest().to_hex(),
        "83c093393bdf7a0e30ace8e208f3bcaa366fb7c6350abf7ff55d446322dcca87"
    );
    assert_eq!(
        forward.receipt().source_r7_digest().to_hex(),
        "7f8d126ee81356a60605013b4b1c23942a77a4b2d6f890125d6c938dae70228b"
    );
    assert_eq!(
        forward.receipt().closure_digest().to_hex(),
        "467cb7d1af751fe522cc3de818107068373531e51a4d9a7371a3f5f9becae29b"
    );
    assert_eq!(forward.receipt().direct_cell_count(), 48_764);
    assert_eq!(forward.receipt().derived_ancestor_count(), 11_085);
    assert_eq!(
        forward.receipt().membership_digest().to_hex(),
        "4bbcdbf0c592b2cdc7ad52a8a8a5ef9a7e9989bd1b11b159be6eec5f2150247f"
    );
    assert_eq!(
        forward.receipt().ref_digest().to_hex(),
        "92b21ff325bde67f26565f52882d3664daacd6d51423f2a588344da012fd4161"
    );
}

#[test]
fn representative_rows_are_parent_first_and_preserve_full_hierarchy_semantics() {
    let source = source_cells();
    let cohort = build_representative_h3_cohort_v1(artifact_digest(), &source)
        .expect("pinned representative source should build");
    let rows = cohort.rows();

    assert_eq!(rows.len(), CLOSURE_COUNT);
    assert_root_row(&rows[0]);

    let r5 = rows
        .iter()
        .take(CLOSURE_COUNT)
        .find(|row| row.cell_id().to_string() == "852a1073fffffff")
        .expect("audited r5 row must exist");
    assert_r5_row(r5);

    let last = rows.last().expect("cohort must not be empty");
    assert_last_row(last);
    assert_parent_first_numeric_order(rows);
    assert_positive_sql_identities(rows);
}

fn assert_root_row(row: &H3ReferenceCellRow) {
    assert_eq!(row.cell_id().to_string(), "800dfffffffffff");
    assert_eq!(row.resolution(), 0);
    assert_eq!(row.origin(), H3ReferenceOrigin::DerivedAncestor);
    assert_eq!(row.immediate_parent(), None);
    assert_eq!(row.ancestor_r4(), None);
    assert_eq!(row.ancestor_r5(), None);
    assert_eq!(row.ancestor_r6(), None);
    assert_eq!(row.ancestor_r7(), None);
}

fn assert_r5_row(r5: &H3ReferenceCellRow) {
    assert_eq!(r5.resolution(), 5);
    assert_eq!(r5.origin(), H3ReferenceOrigin::Direct);
    assert_eq!(
        r5.immediate_parent().map(|cell| cell.to_string()),
        Some("842a107ffffffff".to_owned())
    );
    assert_eq!(
        r5.ancestor_r4().map(|cell| cell.to_string()),
        Some("842a107ffffffff".to_owned())
    );
    assert_eq!(r5.ancestor_r5(), Some(r5.cell_id()));
    assert_eq!(r5.ancestor_r6(), None);
    assert_eq!(r5.ancestor_r7(), None);
}

fn assert_last_row(last: &H3ReferenceCellRow) {
    assert_eq!(last.cell_id().to_string(), "872ab6db6ffffff");
    assert_eq!(last.resolution(), 7);
    assert_eq!(last.origin(), H3ReferenceOrigin::Direct);
    assert_eq!(
        last.immediate_parent().map(|cell| cell.to_string()),
        Some("862ab6db7ffffff".to_owned())
    );
    assert_eq!(
        last.ancestor_r4().map(|cell| cell.to_string()),
        Some("842ab6dffffffff".to_owned())
    );
    assert_eq!(
        last.ancestor_r5().map(|cell| cell.to_string()),
        Some("852ab6dbfffffff".to_owned())
    );
    assert_eq!(
        last.ancestor_r6().map(|cell| cell.to_string()),
        Some("862ab6db7ffffff".to_owned())
    );
    assert_eq!(last.ancestor_r7(), Some(last.cell_id()));
}

fn assert_parent_first_numeric_order(rows: &[H3ReferenceCellRow]) {
    for pair in rows.windows(2).take(CLOSURE_COUNT) {
        let left = &pair[0];
        let right = &pair[1];
        assert!(
            (left.resolution(), left.cell_id()) < (right.resolution(), right.cell_id()),
            "canonical rows must stay strictly parent-first"
        );
        assert!(
            left.cell_id() < right.cell_id(),
            "canonical H3 numeric order must match parent-first order"
        );
    }
}

fn assert_positive_sql_identities(rows: &[H3ReferenceCellRow]) {
    for row in rows.iter().take(CLOSURE_COUNT) {
        assert!(i64::try_from(row.cell_id()).expect("validated H3 fits SQL") > 0);
    }
}

#[test]
fn representative_cohort_refuses_every_untrusted_or_drifted_source_shape() {
    let source = source_cells();

    assert_eq!(
        build_representative_h3_cohort_v1(RefDigestV1::from_bytes([0; 32]), &source),
        Err(H3ReferenceCohortError::ArtifactDigestMismatch {
            expected: artifact_digest(),
            actual: RefDigestV1::from_bytes([0; 32]),
        })
    );
    assert_eq!(
        build_representative_h3_cohort_v1(artifact_digest(), &[]),
        Err(H3ReferenceCohortError::EmptySource)
    );

    let mut duplicate = source.clone();
    duplicate.push(source[0]);
    assert_eq!(
        build_representative_h3_cohort_v1(artifact_digest(), &duplicate),
        Err(H3ReferenceCohortError::DuplicateSourceCell { cell: source[0] })
    );

    let missing = &source[..SOURCE_COUNT - 1];
    assert_eq!(
        build_representative_h3_cohort_v1(artifact_digest(), missing),
        Err(H3ReferenceCohortError::UnexpectedSourceCount {
            expected: SOURCE_COUNT,
            actual: SOURCE_COUNT - 1,
        })
    );

    let mut wrong_resolution = source.clone();
    wrong_resolution[0] = H3CellId::from_str("842a107ffffffff").unwrap();
    assert_eq!(
        build_representative_h3_cohort_v1(artifact_digest(), &wrong_resolution),
        Err(H3ReferenceCohortError::UnexpectedSourceResolution {
            cell: H3CellId::from_str("842a107ffffffff").unwrap(),
            actual: 4,
        })
    );

    let impostor = H3CellId::from_str("85283473fffffff").unwrap();
    assert!(!source.contains(&impostor));
    let mut wrong_set = source.clone();
    wrong_set[0] = impostor;
    match build_representative_h3_cohort_v1(artifact_digest(), &wrong_set) {
        Err(H3ReferenceCohortError::SourceDigestMismatch { expected, actual }) => {
            assert_eq!(
                expected,
                digest("a4685e6ad882930e7064cb225ee649155fb74e52ef8b7d7550691a70a6087f5a")
            );
            assert_ne!(actual, expected);
        }
        other => panic!("wrong same-resolution source must report its measured digest: {other:?}"),
    }

    let oversized = vec![source[0]; MAX_H3_REFERENCE_SOURCE_CELLS + 1];
    assert_eq!(
        build_representative_h3_cohort_v1(artifact_digest(), &oversized),
        Err(H3ReferenceCohortError::TooManySourceCells {
            actual: MAX_H3_REFERENCE_SOURCE_CELLS + 1,
            max: MAX_H3_REFERENCE_SOURCE_CELLS,
        })
    );
}

fn source_cells() -> Vec<H3CellId> {
    representative_h3_reference_cohort_v1()
        .expect("the sole checked-in source fixture must validate")
        .rows()
        .iter()
        .filter(|row| row.origin() == H3ReferenceOrigin::Direct)
        .map(H3ReferenceCellRow::cell_id)
        .collect()
}

fn artifact_digest() -> RefDigestV1 {
    digest("e60d93a43d6c66e84f1e53ecaf633af5911bd5b48b0ef0ad6a012f6d9f5b13a9")
}

fn digest(text: &str) -> RefDigestV1 {
    assert_eq!(text.len(), 64);
    let mut bytes = [0_u8; 32];
    for (index, byte) in bytes.iter_mut().enumerate().take(32) {
        let offset = index * 2;
        *byte = u8::from_str_radix(&text[offset..offset + 2], 16).unwrap();
    }
    RefDigestV1::from_bytes(bytes)
}
