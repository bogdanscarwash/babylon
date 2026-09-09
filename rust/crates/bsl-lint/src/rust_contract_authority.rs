//! Closed repository boundary for Rust-owned gameplay contracts.

use crate::finding::{Finding, Severity};
use crate::repo::Repo;
use std::path::PathBuf;

pub const CHECK: &str = "rust-contract-authority";

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

/// Refuse the exact Python gameplay-authority paths retired by ADR229.
///
/// # Errors
///
/// Returns an infrastructure error when the caller supplies more than one root
/// or the selected root is not a directory.
pub fn run(repo: &Repo, roots: &[String]) -> Result<Vec<Finding>, String> {
    if roots.len() > 1 {
        return Err("rust-contract-authority accepts zero or one root".to_owned());
    }
    let requested = roots
        .first()
        .map_or_else(|| repo.root.clone(), PathBuf::from);
    let root = if requested.is_absolute() {
        requested
    } else {
        repo.root.join(requested)
    };
    if !root.is_dir() {
        return Err(format!(
            "{}: authority root is not a directory",
            root.display()
        ));
    }

    let mut findings = Vec::with_capacity(RETIRED_PATHS.len());
    for relative in RETIRED_PATHS {
        let path = root.join(relative);
        if path.is_file() {
            findings.push(Finding {
                check: CHECK,
                file: repo.display_path(&path),
                line: 1,
                what: "retired Python gameplay authority exists".to_owned(),
                evidence: "executable authority belongs to the Rust contract crates".to_owned(),
                severity: Severity::Fail,
            });
        }
    }
    Ok(findings)
}
