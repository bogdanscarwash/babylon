//! Explicit candidate intervention qualification through the current staffed replay session.
//! This writes one bounded operator report, never campaign data or authored source files.

#[path = "statewide_experiment/arguments.rs"]
mod arguments;
#[path = "statewide_experiment/facts.rs"]
mod facts;
#[path = "statewide_experiment/report.rs"]
mod report;
#[path = "statewide_experiment/run.rs"]
mod run;
#[cfg(test)]
#[path = "../tests/fixtures/statewide_synthetic.rs"]
pub mod synthetic;
#[cfg(test)]
#[path = "statewide_experiment/tests.rs"]
mod tests;
#[path = "statewide_experiment/witness.rs"]
mod witness;

use std::{error::Error, process::ExitCode};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn refused(message: impl Into<String>) -> Box<dyn Error> {
    std::io::Error::other(message.into()).into()
}
fn checked_sum(mut values: impl Iterator<Item = u64>) -> Result<u64> {
    values.try_fold(0_u64, |sum, n| {
        sum.checked_add(n)
            .ok_or_else(|| refused("receipt sum overflow"))
    })
}
fn quantity(a: u64, b: u64) -> Result<u64> {
    a.checked_mul(b)
        .ok_or_else(|| refused("native quantity or mass overflow"))
}
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(encoded, "{byte:02x}").expect("String write");
    }
    encoded
}

fn execute() -> Result<()> {
    let Some(args) = arguments::Arguments::parse(std::env::args().skip(1))? else {
        return Ok(());
    };
    let inputs = arguments::read_inputs(&args)?;
    let report = run::experiment(&inputs, &args.candidate)?;
    arguments::write_report(&args.output, &report)?;
    if !report.qualified {
        return Err(refused(format!(
            "candidate lacks required causal witnesses; complete report: {}",
            args.output.display()
        )));
    }
    println!(
        "Four candidates / 64 committed periods qualified: {}",
        args.output.display()
    );
    Ok(())
}
fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Statewide experiment refused: {error}");
            ExitCode::FAILURE
        }
    }
}
