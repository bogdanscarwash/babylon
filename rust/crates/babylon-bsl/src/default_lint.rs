//! The `:default` migration-corpus allowlist lint (`bsl-language.rst` §3.5
//! item 4): every `:default` declaration must appear in the migration
//! corpus's allowlist — the trap DSL's pinned absent-reads-as-0 sites. A
//! `:default` outside the allowlist is a **lint failure requiring Director
//! sign-off — not a load error**, because the allowlist is program state,
//! not language state.
//!
//! Starts EMPTY — Task 17 (conformance corpus transcription) is the only
//! task expected to populate it, one row per transcribed pinned site. Same
//! governance bar as `EXTENSIVE_INTENSIVE_EXEMPTIONS` (Task 10) and
//! Python's `SentinelExemption`: every row carries its reason and owner.

use crate::bindings::BindingDecl;

/// One Director-approved `:default` site, keyed by the transcribed rule
/// source file and the binding name inside it.
#[derive(Debug, Clone)]
pub struct DefaultAllowlistEntry {
    /// The transcribed rule's source file.
    pub rule_file: &'static str,
    /// The binding carrying the approved `:default`.
    pub binding_name: &'static str,
    /// Why absence-with-default is correct here (the trap-DSL site it
    /// transcribes).
    pub reason: &'static str,
    /// Who approved the row.
    pub owner: &'static str,
    /// When (ISO date).
    pub date: &'static str,
}

/// The allowlist: the trap DSL's pinned absent-reads-as-0 sites, one row
/// per transcribed binding (Task 17). Authority: spec §5 "The migration
/// corpus enumerates the exact rules permitted to carry `:default 0`" +
/// the honest-null reading `test_mechanics.py:67-75` pins ("absent = no
/// accumulated strength, never a fabricated nonzero").
pub const DEFAULT_ALLOWLIST: &[DefaultAllowlistEntry] = &[
    DefaultAllowlistEntry {
        rule_file: "tests/conformance/doctrine_adventurism.bsl",
        binding_name: "mass-link",
        reason: "trap DSL pinned site: an unaccrued doctrine tag reads 0 \
                 (test_mechanics.py:48-52, 71-75)",
        owner: "Director (spec \u{a7}5 migration-corpus enumeration)",
        date: "2026-07-30",
    },
    DefaultAllowlistEntry {
        rule_file: "tests/conformance/doctrine_liquidationism.bsl",
        binding_name: "class-analysis",
        reason: "trap DSL pinned site: an unaccrued doctrine tag reads 0 \
                 (test_mechanics.py:54-64)",
        owner: "Director (spec \u{a7}5 migration-corpus enumeration)",
        date: "2026-07-30",
    },
    DefaultAllowlistEntry {
        rule_file: "tests/conformance/doctrine_liquidationism.bsl",
        binding_name: "militancy",
        reason: "trap DSL pinned site: an unaccrued doctrine tag reads 0 \
                 (test_mechanics.py:54-64, 71-75)",
        owner: "Director (spec \u{a7}5 migration-corpus enumeration)",
        date: "2026-07-30",
    },
    DefaultAllowlistEntry {
        rule_file: "tests/conformance/doctrine_liquidation_absorbing.bsl",
        binding_name: "solidarity-mass",
        reason: "trap DSL pinned site: an unmeasured practice variable \
                 reads 0 (test_mechanics.py:105-106, P25 U11/ADR137)",
        owner: "Director (spec \u{a7}5 migration-corpus enumeration)",
        date: "2026-07-30",
    },
    DefaultAllowlistEntry {
        rule_file: "tests/conformance/doctrine_liquidation_absorbing.bsl",
        binding_name: "co-optive-share",
        reason: "trap DSL pinned site: an unmeasured practice variable \
                 reads 0 (test_mechanics.py:101-103, P25 U11/ADR137)",
        owner: "Director (spec \u{a7}5 migration-corpus enumeration)",
        date: "2026-07-30",
    },
    DefaultAllowlistEntry {
        rule_file: "tests/conformance/doctrine_liquidation_absorbing.bsl",
        binding_name: "petty-bourgeois-drift",
        reason: "trap DSL pinned site: an unmeasured practice variable \
                 reads 0 (test_mechanics.py:114-122, P25 U11/ADR137)",
        owner: "Director (spec \u{a7}5 migration-corpus enumeration)",
        date: "2026-07-30",
    },
];

/// Whether `(rule_file, binding_name)` carries Director sign-off for a
/// `:default`.
#[must_use]
pub fn is_allowed(rule_file: &str, binding_name: &str) -> bool {
    DEFAULT_ALLOWLIST
        .iter()
        .any(|e| e.rule_file == rule_file && e.binding_name == binding_name)
}

/// One lint finding: a `:default` declaration with no allowlist row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultLintFinding {
    /// The rule source file scanned.
    pub rule_file: String,
    /// The binding declaring the unapproved `:default`.
    pub binding_name: String,
}

/// Lint a rule's declared bindings: every `:default` must be allowlisted.
/// Returns findings, not errors — §3.5 item 4 makes this a sign-off gate,
/// not a load rejection.
#[must_use]
pub fn lint_defaults(rule_file: &str, decls: &[BindingDecl]) -> Vec<DefaultLintFinding> {
    decls
        .iter()
        .filter(|decl| decl.default.is_some() && !is_allowed(rule_file, &decl.name))
        .map(|decl| DefaultLintFinding {
            rule_file: rule_file.to_owned(),
            binding_name: decl.name.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bindings::{BindSource, BindingDecl};
    use crate::reader::Atom;

    fn optional_decl(name: &str) -> BindingDecl {
        BindingDecl {
            name: name.to_owned(),
            source: BindSource::Field("social-class/wealth".to_owned()),
            optional: true,
            default: Some(Atom::Int(0)),
        }
    }

    #[test]
    fn an_unapproved_default_is_a_finding_not_a_load_error() {
        let findings = lint_defaults("mods/example.bsl", &[optional_decl("wealth")]);
        assert_eq!(
            findings,
            vec![DefaultLintFinding {
                rule_file: "mods/example.bsl".to_owned(),
                binding_name: "wealth".to_owned(),
            }]
        );
    }

    #[test]
    fn a_binding_without_a_default_lints_clean() {
        let decl = BindingDecl {
            name: "wealth".to_owned(),
            source: BindSource::Field("social-class/wealth".to_owned()),
            optional: false,
            default: None,
        };
        assert!(lint_defaults("mods/example.bsl", &[decl]).is_empty());
    }

    #[test]
    fn every_allowlist_row_carries_full_governance_metadata() {
        for entry in DEFAULT_ALLOWLIST {
            assert!(!entry.rule_file.is_empty());
            assert!(!entry.binding_name.is_empty());
            assert!(
                !entry.reason.is_empty(),
                "a row without a reason is not a row"
            );
            assert!(!entry.owner.is_empty());
            assert!(!entry.date.is_empty());
        }
    }
}
