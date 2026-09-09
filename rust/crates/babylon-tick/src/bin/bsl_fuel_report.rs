//! `bsl-fuel-report` — Task W3 (BSL Hygiene Knock-out train): the fuel-bound
//! report mode. Prints, for every rule in every landed content set,
//! `rule-id declared=<n> computed=<m> headroom=<n-m>` — the same numbers
//! `bound_checker::check_rule` already computes on a successful load
//! (`bound_checker.rs:757-769`, `LoadedRule::{declared_fuel, static_bound}`),
//! surfaced without the "red-run ritual" (temporarily lowering a rule's
//! `:fuel` until `E-LOAD-040` fires, just to read its error message's
//! `computed_bound`/`declared_budget` fields).
//!
//! This is a REPORT, not a gate: it runs no tick, calls no second
//! reader/parser (`babylon_tick::fuel_bound_report` is a thin wrapper over
//! `prepare_rules`, the identical seam `run_once` uses), and changes no
//! load behavior. Exit is non-zero ONLY when a rule's declared budget is
//! under its computed bound — a condition `E-LOAD-040` already refuses at
//! load, so in practice this branch is unreachable for any content set
//! that made it into a printed row at all (a `check_rule` failure there
//! surfaces as this binary's `Err` branch instead, before any row for that
//! content set exists) — see [`babylon_tick::any_over_budget`]'s own doc
//! for the same documented redundancy on the library side.
//!
//! **Content-set enumeration.** No `content-sets.toml` exists on this
//! branch (it lands with the #652 train — the controller addenda for this
//! task names it explicitly) — this binary hardcodes the same 13-pack,
//! one-scenario-per-pack SOLO pairing every dedicated
//! `tests/*_conformance.rs` file (and `tick_goldens.rs`) already
//! establishes as each pack's own canonical load, matching
//! `w2-preaudit-table.md` §1's identical "13 packs, solo everywhere except
//! two co-loads" finding for the same-tick-ordering checker.
//!
//! **Fix round 1 correction (`task-w3-review.md`, Medium finding #1):** the
//! original text here claimed the two committed co-loads
//! (`vitality+lifecycle`, `decomposition+control-ratio`) could be skipped
//! because they "would only ever repeat the same rule ids… without adding
//! new information." That is FALSE in general and was demonstrated false
//! for `control-ratio/c02-publish-census`: its three `(fold sum (nodes
//! NodeType/SOCIAL_CLASS) …)` bindings scale the computed bound 1:1 with
//! the declared `SOCIAL_CLASS` ceiling (`fold_cost`, `bound_checker.rs:
//! 361`), and the solo scenario (`control-ratio-conformance.bscn`, 6
//! `SOCIAL_CLASS` nodes) differs from the real committed co-load scenario
//! (`carceral-arc-conformance.bscn`, 5) — so the solo-only report printed
//! a bound that never actually ships. **Bounds are per-SCENARIO, not
//! per-rule**: `SOLO_PACKS` and `CO_LOADS` below are both included, and a
//! rule that loads in more than one committed content set (every rule in
//! `decomposition`/`control-ratio`; every rule in `vitality`/`lifecycle`)
//! prints ONCE PER SET it actually loads in, each with that set's own real
//! ceilings. Two rows sharing a rule-id with DIFFERING computed/headroom
//! is the correct, honest output, not a duplicate to collapse —
//! `vitality`+`lifecycle` happen to always agree between their solo and
//! co-load rows (vacuously: neither file's rules contain a
//! `nodes`/`neighbors`/`fold` form at all, confirmed by grep, so no
//! ceiling is ever consulted), but that agreement is a fact about those
//! two files specifically, not a property this report can assume holds
//! for any other pair. The GREEN commit's own message asserted the false
//! "no new information" claim as a blanket property; per the standing
//! no-amend convention that message is not rewritten — this doc comment
//! and the fix-round commit are the correction of record.
//!
//! **Enumeration sentinel (Medium finding #2):** `SOLO_PACKS` is still a
//! hand-maintained table with no runtime link to `content/rules/*.bsl` — a
//! 14th landed pack this table forgets to add would previously make
//! `bsl-fuel-report` silently under-report it (still exit 0, no warning).
//! `tests::solo_packs_names_match_the_content_rules_directory_exactly`
//! (bottom of this file) closes that gap: it reads the real directory
//! listing at test time and fails loudly, by name, the moment the two
//! diverge — `mise run bsl:fuel-check` staying green is no longer
//! sufficient evidence that coverage is complete; `cargo test -p
//! babylon-tick --locked --bin bsl-fuel-report` is.

use babylon_bsl::compose_declaration_preludes;
use babylon_tick::{any_over_budget, fuel_bound_report};
use std::process::ExitCode;

const ORGANIZATION_PRACTICE_PRELUDE: &str =
    include_str!("../../content/declarations/organization-practice.bscn");
const WORLDVIEW_PRELUDE: &str = include_str!("../../content/declarations/worldview.bscn");
const NO_PRELUDES: &[&str] = &[];
const ORGANIZATION_PRELUDES: &[&str] = &[ORGANIZATION_PRACTICE_PRELUDE];
const ORGANIZATION_WORLDVIEW_PRELUDES: &[&str] =
    &[ORGANIZATION_PRACTICE_PRELUDE, WORLDVIEW_PRELUDE];

/// (pack name, scenario source, ordered declaration preludes, rule source)
/// — the SAME scenario each pack's own dedicated conformance test loads it
/// against (see this file's module doc for the citations backing each
/// pairing). `name` doubles as the expected `content/rules/<name>.bsl`
/// file stem the enumeration sentinel test cross-checks — keep it exactly
/// equal to the file's own stem, or that test fails.
const SOLO_PACKS: &[(&str, &str, &[&str], &str)] = &[
    (
        "fundamental-theorem",
        include_str!("../../content/scenarios/two-classes.bscn"),
        NO_PRELUDES,
        include_str!("../../content/rules/fundamental-theorem.bsl"),
    ),
    (
        "vitality",
        include_str!("../../content/scenarios/vitality-conformance.bscn"),
        NO_PRELUDES,
        include_str!("../../content/rules/vitality.bsl"),
    ),
    (
        "lifecycle",
        include_str!("../../content/scenarios/lifecycle-conformance.bscn"),
        NO_PRELUDES,
        include_str!("../../content/rules/lifecycle.bsl"),
    ),
    (
        "organization",
        include_str!("../../content/scenarios/organization-foundation.bscn"),
        ORGANIZATION_PRELUDES,
        include_str!("../../content/rules/organization.bsl"),
    ),
    (
        "territory",
        include_str!("../../content/scenarios/territory-conformance.bscn"),
        NO_PRELUDES,
        include_str!("../../content/rules/territory.bsl"),
    ),
    (
        "production",
        include_str!("../../content/scenarios/production-conformance.bscn"),
        NO_PRELUDES,
        include_str!("../../content/rules/production.bsl"),
    ),
    (
        "worldview",
        include_str!("../../content/scenarios/worldview-foundation.bscn"),
        NO_PRELUDES,
        include_str!("../../content/rules/worldview.bsl"),
    ),
    (
        "consciousness",
        include_str!("../../content/scenarios/consciousness-ternary-conformance.bscn"),
        ORGANIZATION_WORLDVIEW_PRELUDES,
        include_str!("../../content/rules/consciousness.bsl"),
    ),
    (
        "solidarity",
        include_str!("../../content/scenarios/solidarity-conformance.bscn"),
        NO_PRELUDES,
        include_str!("../../content/rules/solidarity.bsl"),
    ),
    (
        "struggle-spark",
        include_str!("../../content/scenarios/struggle-spark-conformance.bscn"),
        NO_PRELUDES,
        include_str!("../../content/rules/struggle-spark.bsl"),
    ),
    (
        "decomposition",
        include_str!("../../content/scenarios/decomposition-conformance.bscn"),
        NO_PRELUDES,
        include_str!("../../content/rules/decomposition.bsl"),
    ),
    (
        "control-ratio",
        include_str!("../../content/scenarios/control-ratio-conformance.bscn"),
        NO_PRELUDES,
        include_str!("../../content/rules/control-ratio.bsl"),
    ),
    (
        "dispossession",
        include_str!("../../content/scenarios/dispossession-conformance.bscn"),
        NO_PRELUDES,
        include_str!("../../content/rules/dispossession.bsl"),
    ),
    (
        "metabolism",
        include_str!("../../content/scenarios/metabolism-conformance.bscn"),
        NO_PRELUDES,
        include_str!("../../content/rules/metabolism.bsl"),
    ),
    (
        // ImperialRent BSL port train (Task 1, plan
        // `docs/superpowers/plans/2026-08-18-imperialrent-port.md`) — the
        // pack's own conformance test loads it solo against this scenario
        // (`imperial_rent_conformance.rs`'s SCENARIO/RULE pair).
        "imperial-rent",
        include_str!("../../content/scenarios/imperial-rent-conformance.bscn"),
        NO_PRELUDES,
        include_str!("../../content/rules/imperial-rent.bsl"),
    ),
    (
        // #491 T5+T6 (the rung-ladder train): the pack's conformance test
        // loads it solo, no prelude (`vitality_attrition_conformance.rs`'s
        // `run_once(SCENARIO, RULE)`).
        "vitality-attrition",
        include_str!("../../content/scenarios/vitality-attrition-conformance.bscn"),
        NO_PRELUDES,
        include_str!("../../content/rules/vitality-attrition.bsl"),
    ),
    (
        // Community port train (issue #667), Tasks 8-9: the pack over
        // conformance world 1. The pack's per-world bounds (worlds 2/3 —
        // D-NF+22) are measured by the declare-low/read-E-LOAD-040 cycle in
        // tests/community_conformance.rs, which loads all three worlds at
        // the declared fuel (a world whose bound exceeds it reds at load).
        "community",
        include_str!("../../content/scenarios/community-conformance.bscn"),
        ORGANIZATION_PRELUDES,
        include_str!("../../content/rules/community.bsl"),
    ),
    (
        // TickDynamics class-dynamics port train (issue #669, Feature-016
        // @4.0). Task 2 lands the header-only pack; later tasks populate
        // it with rules. The fuel report row is added now so the
        // enumeration sentinel stays green as the file lands.
        "class-dynamics",
        include_str!("../../content/scenarios/class-dynamics-conformance.bscn"),
        NO_PRELUDES,
        include_str!("../../content/rules/class-dynamics.bsl"),
    ),
];

/// The two committed multi-file co-loads — see the module doc's fix-round
/// correction for why these are here and why their computed bounds can
/// legitimately differ from `SOLO_PACKS`' rows for the same rule-ids.
/// `w2-preaudit-table.md` §1 is the same-tick-ordering checker's
/// independent confirmation that these are the ONLY two cross-pack
/// combinations any committed test loads together — no third co-load
/// exists to add. Each `name` is a pack PAIR, not a file stem, so these
/// entries are deliberately excluded from the enumeration sentinel below.
const CO_LOADS: &[(&str, &str, &[&str], &str)] = &[
    (
        "vitality+lifecycle",
        include_str!("../../content/scenarios/us-counties-lifecycle-demo.bscn"),
        NO_PRELUDES,
        concat!(
            include_str!("../../content/rules/vitality.bsl"),
            "\n",
            include_str!("../../content/rules/lifecycle.bsl"),
        ),
    ),
    (
        "decomposition+control-ratio",
        include_str!("../../content/scenarios/carceral-arc-conformance.bscn"),
        NO_PRELUDES,
        concat!(
            include_str!("../../content/rules/decomposition.bsl"),
            "\n",
            include_str!("../../content/rules/control-ratio.bsl"),
        ),
    ),
];

fn main() -> ExitCode {
    let mut rows = Vec::new();
    for (pack, scenario, preludes, rule) in SOLO_PACKS.iter().chain(CO_LOADS) {
        let composed = if preludes.is_empty() {
            None
        } else {
            match compose_declaration_preludes(preludes) {
                Ok(source) => Some(source),
                Err(error) => {
                    eprintln!("bsl-fuel-report: {pack}: {error}");
                    return ExitCode::from(2);
                }
            }
        };
        match fuel_bound_report(scenario, composed.as_deref(), rule) {
            Ok(pack_rows) => rows.extend(pack_rows),
            Err(e) => {
                eprintln!("bsl-fuel-report: {pack}: {e}");
                return ExitCode::from(2);
            }
        }
    }
    // Global presentation sort by rule-ID bytes. Each pack's rows arrive in
    // executable phase order from `prepare_rules`; this report deliberately
    // re-sorts the WHOLE inventory so its output stays independent of the
    // table's declared order, which
    // (like any content ordering, §2.2) carries no semantics. `sort_by` is
    // STABLE, so when a rule-id repeats across a solo row and a co-load
    // row (see the module doc), the two keep their relative iteration
    // order (`SOLO_PACKS` then `CO_LOADS`) rather than swapping
    // unpredictably run to run.
    rows.sort_by(|a, b| a.rule_id.as_bytes().cmp(b.rule_id.as_bytes()));

    for row in &rows {
        println!("{row}");
    }

    if any_over_budget(&rows) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use super::SOLO_PACKS;

    /// Fix round 1 (`task-w3-review.md`, Medium finding #2): `SOLO_PACKS`
    /// is hand-maintained with no prior link to the real filesystem, so a
    /// 14th landed `content/rules/*.bsl` pack this table forgot to add
    /// would silently under-report — `bsl-fuel-report` keeps exiting 0,
    /// printing only the packs it already knows about. This reads the
    /// real directory listing (sorted, byte order — same convention
    /// `bsl-lint`'s own directory walks use) and fails loudly, naming
    /// exactly what drifted, the moment the table and the filesystem
    /// disagree.
    #[test]
    fn solo_packs_names_match_the_content_rules_directory_exactly() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/content/rules");
        let mut on_disk: Vec<String> = std::fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("cannot read {dir}: {e}"))
            .map(|entry| {
                entry
                    .expect("directory entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .filter(|name| name.ends_with(".bsl"))
            .collect();
        on_disk.sort();

        let mut in_table: Vec<String> = SOLO_PACKS
            .iter()
            .map(|(name, ..)| format!("{name}.bsl"))
            .collect();
        in_table.sort();

        assert_eq!(
            in_table, on_disk,
            "SOLO_PACKS (bsl_fuel_report.rs) has drifted from content/rules/*.bsl — add \
             (or remove) a row so bsl-fuel-report covers every landed pack, then re-run \
             this test (task-w3-review.md fix round 1, Medium finding #2)"
        );
    }
}
