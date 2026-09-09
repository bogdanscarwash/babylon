//! `bsl-lint` — the repo-relationship sentinel host for BSL content.
//!
//! Scope, Director-ruled 2026-08-18 (the BSL hygiene knock-out train, W1):
//! this binary hosts checks over things babylon-bsl's own reader/loader
//! CANNOT see from inside one content set — git tags, `ai/decisions/`, and
//! cross-crate Rust source. A check computable purely from content
//! declarations belongs in the loader as an `E-LOAD` refusal instead (Task
//! W2), not here.
//!
//! Shape mirrors `tools/sentinel_check.py`: a positional check name selects
//! one registered check (`--list` prints the registry, `all` runs every
//! check), optional trailing `paths` scope it. Exit contract: 0 clean, 1 a
//! `Fail`-severity finding was printed, 2 usage/infrastructure error.

mod citation_drift;
mod finding;
mod namespace_unique;
mod repo;
mod rust_contract_authority;
mod sfs_non_authorability;

use finding::{Finding, Severity};
use repo::Repo;
use std::process::ExitCode;

type CheckFn = fn(&Repo, &[String]) -> Result<Vec<Finding>, String>;

const CHECKS: &[(&str, CheckFn)] = &[
    (citation_drift::CHECK, citation_drift::run),
    (namespace_unique::CHECK, namespace_unique::run),
    (rust_contract_authority::CHECK, rust_contract_authority::run),
    (sfs_non_authorability::CHECK, sfs_non_authorability::run),
];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("--help" | "-h") => {
            print_usage();
            ExitCode::from(2)
        }
        Some("--list") => {
            print_registry();
            ExitCode::from(0)
        }
        Some("all") => run_and_report(&args[1..], CHECKS),
        Some(name) => match CHECKS.iter().find(|(n, _)| *n == name) {
            Some(&entry) => run_and_report(&args[1..], std::slice::from_ref(&entry)),
            None => {
                eprintln!("bsl-lint: unknown check {name:?}");
                print_usage();
                ExitCode::from(2)
            }
        },
    }
}

fn print_usage() {
    eprintln!("usage: bsl-lint <check|all|--list> [paths...]");
}

fn print_registry() {
    for (name, _) in CHECKS {
        println!("{name}");
    }
    println!("all");
}

fn run_and_report(paths: &[String], checks: &[(&'static str, CheckFn)]) -> ExitCode {
    let repo = match Repo::discover() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("bsl-lint: {e}");
            return ExitCode::from(2);
        }
    };

    let mut any_fail = false;
    for (name, run) in checks {
        match run(&repo, paths) {
            Ok(findings) => {
                for finding in findings {
                    if finding.severity == Severity::Fail {
                        any_fail = true;
                    }
                    println!("{finding}");
                }
            }
            Err(e) => {
                eprintln!("bsl-lint: {name}: {e}");
                return ExitCode::from(2);
            }
        }
    }
    ExitCode::from(u8::from(any_fail))
}
