//! Behavioral contracts for the Rust gameplay-contract authority boundary.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const CHECK: &str = "rust-contract-authority";
const RETIRED_PATHS: [&str; 8] = [
    "src/babylon/contracts/practice_contract_v1.py",
    "src/babylon/contracts/practice_contract_v1_generated.py",
    "src/babylon/contracts/relational_territory_dossier_v1.py",
    "src/babylon/contracts/rtd_v1_generated.py",
    "tools/generate_practice_contract_types.py",
    "tools/generate_rtd_v1_types.py",
    "tools/build_detroit_rtd_control.py",
    "tools/sfs_contract_vectors.py",
];
static SCRATCH_COUNTER: AtomicU64 = AtomicU64::new(0);

struct ScratchRoot(PathBuf);

impl ScratchRoot {
    fn new(label: &str) -> Self {
        let serial = SCRATCH_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "bsl-rust-authority-{label}-{}-{serial}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("scratch root must be created");
        Self(root)
    }
}

impl Drop for ScratchRoot {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("scratch root must be removed");
    }
}

fn write_file(root: &Path, relative: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("retired path parent"))
        .expect("scratch parent must be created");
    std::fs::write(path, b"# retired authority witness\n")
        .expect("scratch witness must be written");
}

fn run(roots: &[&Path]) -> (i32, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bsl-lint"));
    command.arg(CHECK);
    for root in roots.iter().take(2) {
        command.arg(root);
    }
    let output = command.output().expect("bsl-lint must run");
    let mut report = String::from_utf8_lossy(&output.stdout).into_owned();
    report.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.code().unwrap_or(-1), report)
}

#[test]
fn clean_root_passes() {
    let scratch = ScratchRoot::new("clean");
    let (code, report) = run(&[&scratch.0]);
    assert_eq!(code, 0, "report was:\n{report}");
}

#[test]
fn every_retired_python_authority_path_fails() {
    for relative in RETIRED_PATHS {
        let scratch = ScratchRoot::new("retired");
        write_file(&scratch.0, relative);
        let (code, report) = run(&[&scratch.0]);
        assert_eq!(code, 1, "path {relative}; report was:\n{report}");
        assert!(
            report.contains("E-SENTINEL rust-contract-authority")
                && report.contains(relative)
                && report.contains("retired Python gameplay authority exists")
                && report.contains("executable authority belongs to the Rust contract crates"),
            "path {relative}; report was:\n{report}"
        );
    }
}

#[test]
fn more_than_one_root_is_an_infrastructure_error() {
    let first = ScratchRoot::new("first");
    let second = ScratchRoot::new("second");
    let (code, report) = run(&[&first.0, &second.0]);
    assert_eq!(code, 2, "report was:\n{report}");
    assert!(
        report.contains("accepts zero or one root"),
        "report was:\n{report}"
    );
}

#[test]
fn real_repository_has_no_retired_python_gameplay_authority() {
    let (code, report) = run(&[]);
    assert_eq!(code, 0, "report was:\n{report}");
}
