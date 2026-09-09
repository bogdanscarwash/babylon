//! `babylon-tick` — the Phase 2 Slice 1 driver.
//!
//! ```text
//! babylon-tick <scenario.bscn> <rule.bsl>
//! ```
//!
//! Loads a world, loads a rule through every gate, runs one tick, and prints
//! both the graph-state and nominal world hashes. Running it twice on the
//! same inputs must print the same hashes; that is Constitution III.7 made
//! observable from a shell rather than only from a test.
//!
//! # Where the content lives
//!
//! `content/` **inside this crate**, not at the repo root. §2.2 of the
//! language spec says a content set is "the union of all files under the
//! declared content roots" — *declared*, and plural, so the durable content
//! root is configuration and choosing one is an architecture decision this
//! slice has no standing to make. Until it is declared, this crate's example
//! content is scoped to this crate, where it is unambiguously the driver's
//! demonstration data rather than a claim about where game content belongs.
//!
//! **This is not `babylon-engine`.** The engine — anchor total-order
//! resolution, the system registry, a tick *sequence* — is Phase 3, and
//! naming this binary as though it were would be claiming ground that has
//! not been taken. It drives ONE rule over ONE scenario for ONE tick. When
//! Phase 3 charters the engine, this either becomes its first command or is
//! absorbed and deleted; either way the slice's evidence survives in
//! `babylon-bsl`'s tests, which do not depend on this binary existing.
//!
//! Exit status is 0 on a completed tick and 1 on any failure, with the
//! reason on stderr — a driver that printed a hash for a failed run would be
//! worse than one that printed nothing.
//!
//! The tick flow itself (scenario load -> rule load -> one tick -> state
//! hash) lives in `babylon_tick::run_once` (Program 28 B0) — this binary is
//! now only argument parsing, calling that seam, and printing. The same
//! `run_once` is what `babylon-client`'s engine link calls in-process.

use babylon_tick::{hex, run_once, TickReport};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let [_, scenario_path, rule_path] = args.as_slice() else {
        eprintln!("usage: babylon-tick <scenario.bscn> <rule.bsl>");
        return ExitCode::FAILURE;
    };

    match run(scenario_path, rule_path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("babylon-tick: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(scenario_path: &str, rule_path: &str) -> Result<(), String> {
    let scenario_src = std::fs::read_to_string(scenario_path)
        .map_err(|e| format!("cannot read scenario {scenario_path}: {e}"))?;
    let rule_src = std::fs::read_to_string(rule_path)
        .map_err(|e| format!("cannot read rule {rule_path}: {e}"))?;

    let report = run_once(&scenario_src, &rule_src)?;

    print!("{}", format_report(&report));
    Ok(())
}

fn format_report(report: &TickReport) -> String {
    let unchanged_note = if report.before == report.after {
        // Graph equality says nothing by itself about rule eligibility,
        // emitted events, or the completed-time component of world identity.
        "note             graph state unchanged; the committed tick may still advance time or emit events\n"
    } else {
        ""
    };
    format!(
        "tick 1           {} subjects fired\n\
         graph-before     {}\n\
         graph-after      {}\n\
         world-before     {}\n\
         world-after      {}\n\
         {unchanged_note}",
        report.fired,
        hex(&report.before),
        hex(&report.after),
        hex(&report.world_before),
        hex(&report.world_after),
    )
}

#[cfg(test)]
mod tests {
    use super::format_report;
    use babylon_tick::TickReport;

    #[test]
    fn cli_labels_hashes_and_describes_equal_graph_hashes_without_inferring_no_guard() {
        let report = TickReport {
            before: [0x11; 32],
            after: [0x11; 32],
            world_before: [0x22; 32],
            world_after: [0x33; 32],
            considered: 3,
            fired: 2,
            per_rule_considered: vec![("vitality/emit-only".to_owned(), 3)],
            per_rule_fired: vec![("vitality/emit-only".to_owned(), 2)],
            audit_receipts: Vec::new(),
            choice_receipts: Vec::new(),
            committed_events: Vec::new(),
        };

        let output = format_report(&report);

        assert!(output.contains(
            "graph-before     1111111111111111111111111111111111111111111111111111111111111111"
        ));
        assert!(output.contains(
            "graph-after      1111111111111111111111111111111111111111111111111111111111111111"
        ));
        assert!(output.contains(
            "world-before     2222222222222222222222222222222222222222222222222222222222222222"
        ));
        assert!(output.contains(
            "world-after      3333333333333333333333333333333333333333333333333333333333333333"
        ));
        assert!(output.contains(
            "graph state unchanged; the committed tick may still advance time or emit events"
        ));
        assert!(!output.contains("no subject passed the guard"));
    }

    #[test]
    fn unchanged_note_compares_the_clearly_named_graph_hashes() {
        let report = TickReport {
            before: [0x11; 32],
            after: [0x12; 32],
            world_before: [0x22; 32],
            world_after: [0x22; 32],
            considered: 1,
            fired: 1,
            per_rule_considered: Vec::new(),
            per_rule_fired: Vec::new(),
            audit_receipts: Vec::new(),
            choice_receipts: Vec::new(),
            committed_events: Vec::new(),
        };

        assert!(!format_report(&report).contains("graph state unchanged"));
    }
}
