//! Fixed PER-309 delivery/stock comparison using the native material session.
//! This example publishes ephemeral observations, never `PostgreSQL` commits.

#[path = "michigan_experiment/artifacts.rs"]
mod artifacts;
#[path = "michigan_experiment/observe.rs"]
mod observe;
#[path = "michigan_experiment/run.rs"]
mod run;
#[cfg(test)]
#[path = "michigan_experiment/tests.rs"]
mod tests;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use artifacts::{contract, Output, Progress, Result, Watchdog};
use serde_json::json;

fn output_argument(args: impl IntoIterator<Item = std::ffi::OsString>) -> Result<PathBuf> {
    let args: Vec<_> = args.into_iter().collect();
    if args.len() != 2 || args[0] != "--output" {
        return Err(contract(
            "usage: michigan_experiment --output /absolute/new-directory",
        ));
    }
    let path = PathBuf::from(&args[1]);
    if !path.is_absolute() {
        return Err(contract("output directory must be absolute"));
    }
    Ok(path)
}

fn execute(output: &mut Output, progress: &Mutex<Progress>) -> Result<()> {
    let provenance = artifacts::provenance()?;
    let cases = run::cases()?;
    let experiment_identity = run::experiment_identity(&cases)?;
    let experiment_digest = run::digest_json(&experiment_identity)?;
    // Retain exact preparation even when a later transition fails.
    output.write_json("inputs.json", &experiment_identity)?;
    let mut results = Vec::new();
    for case in cases {
        *progress
            .lock()
            .map_err(|_| contract("progress mutex poisoned"))? = Progress {
            case: Some(case.spec.id.to_owned()),
            last_completed_period: 0,
        };
        let result = run::run_case(&case, |row| {
            output.append_period(row)?;
            progress
                .lock()
                .map_err(|_| contract("progress mutex poisoned"))?
                .last_completed_period = row.period;
            Ok(())
        })?;
        results.push(result);
    }
    let summary = run::summarize(&results)?;
    output.write_json("summary.json", &summary)?;
    let checksums = output.checksums()?;
    output.write_json("manifest.json", &json!({
        "schema": "MichiganDeliveryStockExperimentManifestV1",
        "status": "complete",
        "experiment_digest": experiment_digest,
        "inputs": experiment_identity,
        "case_foundations": results.iter().map(|result| json!({"case": result.case, "identity": result.foundation})).collect::<Vec<_>>(),
        "provenance": provenance,
        "persistence": false,
        "postgresql_parity_qualified": false,
        "seed": 319,
        "seed_override_supported": false,
        "governed_realization_consumers": 0,
        "observed_choice_receipts": 0,
        "bounds": {"cases": 4, "periods_per_case": 16, "period_records": 64,
            "wall_seconds_after_compilation": 60, "artifact_bytes": 4 * 1024 * 1024,
            "concurrent_sessions": 1, "full_graph_dumps": false},
        "files_sha256": checksums,
        "evidence": {"parameters": "Designed", "outcomes": "Derived",
            "constraint_bounds": "Derived diagnostics for one process per Michigan site; ties do not prove independent causal bottlenecks"}
    }))
}

fn main() -> std::process::ExitCode {
    let result = (|| {
        let directory = output_argument(std::env::args_os().skip(1))?;
        let mut output = Output::create(&directory)?;
        let progress = Arc::new(Mutex::new(Progress::default()));
        let watchdog = Watchdog::start(&directory, Arc::clone(&progress));
        let result = execute(&mut output, &progress);
        watchdog.finish()?;
        if let Err(error) = &result {
            let state = progress
                .lock()
                .map_err(|_| contract("progress mutex poisoned"))?;
            artifacts::write_failure(&directory, &state, &error.to_string())?;
        }
        result?;
        println!("Four cases / 64 periods complete: {}", directory.display());
        Ok::<_, artifacts::Error>(())
    })();
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Michigan experiment refused: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
