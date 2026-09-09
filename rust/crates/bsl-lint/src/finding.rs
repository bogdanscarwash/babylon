//! The one shared finding shape both checks emit, and the output contract
//! (task brief W1.2): one line per finding, gate on `Fail` only.

use std::fmt;

/// Whether a finding gates the check (`Fail`, exit 1) or is informational
/// (`Warn`, printed but never flips the exit code) — the brief's tier
/// split for `citation-drift` (tier 1 fails, tier 2/900-byte warn).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Fail,
    Warn,
}

/// One line of output: `E-SENTINEL <check>: file:line — <what> — <nearest evidence>`
/// (or `W-SENTINEL` for a `Warn`-severity finding — same shape, so the
/// gating/informational split is visible without re-parsing the message).
#[derive(Debug, Clone)]
pub struct Finding {
    pub check: &'static str,
    pub file: String,
    pub line: usize,
    pub what: String,
    pub evidence: String,
    pub severity: Severity,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tag = match self.severity {
            Severity::Fail => "E-SENTINEL",
            Severity::Warn => "W-SENTINEL",
        };
        write!(
            f,
            "{tag} {}: {}:{} — {} — {}",
            self.check, self.file, self.line, self.what, self.evidence
        )
    }
}
