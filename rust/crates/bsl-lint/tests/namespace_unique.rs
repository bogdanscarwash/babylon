//! Integration tests for `bsl-lint namespace-unique` (task brief W1.1/W1.4).
//! Two independent fixture roots: `tests/fixtures/decisions/` (the ADR
//! number/index-sync RED cases, (a)) and `tests/fixtures/src_scan/` (the
//! cross-file E-code duplicate RED case, (b)). Also runs against the REAL
//! `ai/decisions/` + `rust/crates` to prove the check starts green on
//! landed content (the allowlist seeded in `namespace_unique.rs` must
//! cover every real cross-file sharing, or this goes red for everyone).

use std::path::PathBuf;
use std::process::Command;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn run(decisions: &str, src_scan: &str) -> (i32, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_bsl-lint"))
        .arg("namespace-unique")
        .arg(decisions)
        .arg(src_scan)
        .output()
        .expect("bsl-lint must run");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    (output.status.code().unwrap_or(-1), stdout)
}

fn fixture_run() -> (i32, String) {
    let decisions = fixtures_dir().join("decisions");
    let src_scan = fixtures_dir().join("src_scan");
    run(
        decisions.to_str().expect("utf8 path"),
        src_scan.to_str().expect("utf8 path"),
    )
}

#[test]
fn a_duplicate_adr_number_fails() {
    let (code, stdout) = fixture_run();
    assert_eq!(code, 1, "stdout was:\n{stdout}");
    assert!(
        stdout.contains("E-SENTINEL namespace-unique") && stdout.contains("ADR900 is not unique"),
        "expected an ADR900 duplicate FAIL, got:\n{stdout}"
    );
}

#[test]
fn a_file_with_no_index_entry_fails() {
    let (_, stdout) = fixture_run();
    assert!(
        stdout.contains("ADR901_gamma") && stdout.contains("no index.yaml entry"),
        "expected an ADR901_gamma sync FAIL, got:\n{stdout}"
    );
}

#[test]
fn an_index_key_with_no_file_fails() {
    let (_, stdout) = fixture_run();
    assert!(
        stdout.contains("ADR902_orphan") && stdout.contains("no backing ADR file"),
        "expected an ADR902_orphan sync FAIL, got:\n{stdout}"
    );
}

#[test]
fn an_unallowlisted_cross_file_e_code_fails() {
    let (code, stdout) = fixture_run();
    assert_eq!(code, 1, "stdout was:\n{stdout}");
    assert!(
        stdout.contains("E-FAKE-777") && stdout.contains("2 distinct files"),
        "expected an E-FAKE-777 cross-file FAIL, got:\n{stdout}"
    );
}

#[test]
fn a_test_only_occurrence_does_not_count_toward_the_duplicate() {
    let (_, stdout) = fixture_run();
    assert!(
        !stdout.contains("E-FAKE-778"),
        "crate-a's E-FAKE-778 is test-only (must be excluded) and crate-b's \
         is a single real site — this must stay unflagged:\n{stdout}"
    );
}

#[test]
fn cross_file_evidence_lists_files_in_sorted_order_not_directory_order() {
    // I1: crate-zulu/src/lib.rs was created on disk before crate-alpha/src/lib.rs
    // (see both fixtures' doc comments), so an unsorted directory walk lists
    // crate-zulu first even though crate-alpha sorts first alphabetically.
    // list_src_rs_files must sort — the finding's own (file, line) header
    // AND the evidence "sites:" string must both cite crate-alpha first.
    let (code, stdout) = fixture_run();
    assert_eq!(code, 1, "stdout was:\n{stdout}");
    let line = stdout
        .lines()
        .find(|l| l.contains("E-FAKE-555"))
        .unwrap_or_else(|| panic!("expected an E-FAKE-555 cross-file FAIL, got:\n{stdout}"));
    assert!(
        line.starts_with(
            "E-SENTINEL namespace-unique: bsl-lint/tests/fixtures/src_scan/crate-alpha/src/lib.rs:"
        ),
        "finding header must cite the alphabetically-first file (crate-alpha), got:\n{line}"
    );
    let alpha_pos = line
        .find("crate-alpha/src/lib.rs")
        .expect("crate-alpha must appear in the evidence");
    let zulu_pos = line
        .find("crate-zulu/src/lib.rs")
        .expect("crate-zulu must appear in the evidence");
    assert!(
        alpha_pos < zulu_pos,
        "evidence must list crate-alpha before crate-zulu (sorted order), got:\n{line}"
    );
}

#[test]
fn the_real_estate_is_clean_under_the_seeded_allowlist() {
    let output = Command::new(env!("CARGO_BIN_EXE_bsl-lint"))
        .arg("namespace-unique")
        .output()
        .expect("bsl-lint must run against the real repo estate");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(0),
        "real ai/decisions/ + rust/crates must be clean under the seeded \
         allowlist; findings:\n{stdout}"
    );
}
