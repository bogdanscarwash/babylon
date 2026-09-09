//! `namespace-unique`: two repo-relationship number-registries — neither is
//! visible to babylon-bsl's reader (one lives in `ai/decisions/`, a YAML
//! estate; the other spans every crate's `src/`, outside any one BSL
//! content set).
//!
//! (a) `ai/decisions/ADR<N>_*.yaml` — every number unique, every filename
//! stem present as an `index.yaml` key, every `index.yaml` key backed by a
//! file (bidirectional sync).
//!
//! (b) BSL spec-code (`E-<FAMILY>-<NNN>`) string literals in every crate's
//! `src/` (never `tests/` — assertions aren't emission sites, and never a
//! `#[cfg(test)] mod tests { … }` block for the same reason) grouped by
//! FILE (the practical proxy for "one error class" this codebase actually
//! uses — see `docs/reference/bsl-language.rst`'s own extensive documented
//! reuse of codes like `E-LOAD-001` "duplicate declaration" across many
//! unrelated declaration kinds). A code owned by more than one file is a
//! finding unless [`ALLOWLIST`] names that exact file set with a citation
//! into the spec — the 2026-08-18 survey run of this check found 11 such
//! groups, all confirmed legitimate against the spec text; each is seeded
//! below so the check starts green on real landed content and only reds on
//! a genuinely NEW, unreviewed sharing.
//!
//! (c) The `TAG_*` section-tag pairwise-distinctness assertion lives beside
//! `state_hash.rs`'s constants in `babylon-graph` (a plain `#[test]`), not
//! here — repo-relationship tooling is not where a five-constant content
//! invariant belongs (Director boundary ruling, this session).

use crate::finding::{Finding, Severity};
use crate::repo::Repo;
use regex::Regex;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;

pub const CHECK: &str = "namespace-unique";

/// `(code, files sharing it, citation)` — every cross-file E-code sharing
/// this check has actually found and had reviewed against
/// `docs/reference/bsl-language.rst`. `files` are `crates/<crate>/src/...rs`
/// paths, sorted, matched as an exact set: a code appearing in a DIFFERENT
/// file set (a new file joins, or the sharing narrows) is not this entry
/// and still fails — the allowlist records what was reviewed, not a
/// blanket exemption for the code string.
const ALLOWLIST: &[(&str, &[&str], &str)] = &[
    (
        "E-LOAD-030",
        &[
            "babylon-bsl/src/scenario.rs",
            "babylon-bsl/src/vocabulary.rs",
        ],
        "the task brief's seeded pair — vocabulary.rs's VocabularyError::UnknownEnumType \
         and scenario.rs's load_defvocabulary both refuse an unregistered enum-kind name; \
         bsl-language.rst:2211 documents E-LOAD-030 for both call sites",
    ),
    (
        "E-LOAD-001",
        &[
            "babylon-bsl/src/declarations.rs",
            "babylon-bsl/src/metrics.rs",
            "babylon-bsl/src/rule_pipeline.rs",
            "babylon-bsl/src/scenario.rs",
            "babylon-tick/src/lib.rs",
        ],
        "bsl-language.rst:659 — \"[duplicates within a] content set are E-LOAD-001\", the \
         generic code for ANY duplicate declaration; reused by design across ~17 documented \
         sites in the spec (deffield, defvocabulary, defrule ids, rung names, …). \
         babylon-tick/src/lib.rs joined the set in the 2026-08-21 worktree sweep: the D32 \
         implicit-<edge-type>/strength collision check (#652 T2) refuses a deffield \
         re-declaring an implicit field — the same generic duplicate-declaration class, \
         returned as PrepareError::Composition with the code carried as data. \
         rule_pipeline.rs joined under ADR222/PER-17: duplicate rule ids now carry the \
         same governed code as typed data across aggregate source boundaries.",
    ),
    (
        "E-LOAD-011",
        &["babylon-bsl/src/bindings.rs", "babylon-bsl/src/metrics.rs"],
        "bsl-language.rst:857,2038 — an unregistered metric read is E-LOAD-011 at both the \
         binding-resolution site and metrics.rs's own registry lookup",
    ),
    (
        "E-LOAD-042",
        &[
            "babylon-bsl/src/bound_checker.rs",
            "babylon-bsl/src/manifest.rs",
        ],
        "bsl-language.rst:1672,2927,3105 — a row-flag / :max-members mismatch against the \
         manifest is E-LOAD-042 at both the manifest load and the bound checker's ceiling read",
    ),
    (
        "E-LOAD-045",
        &[
            "babylon-bsl/src/bound_checker.rs",
            "babylon-bsl/src/manifest.rs",
        ],
        "bsl-language.rst:1681,2901 — a queried type carrying no manifest row is E-LOAD-045 \
         at both the manifest lookup and the bound checker's ceiling(query) computation",
    ),
    (
        "E-LOAD-054",
        &[
            "babylon-bsl/src/declarations.rs",
            "babylon-bsl/src/scenario.rs",
        ],
        "bsl-language.rst:483,2240,3884 — an unknown :enum-type registry name is E-LOAD-054 \
         at both the declaration path and the scenario :enum-type load path",
    ),
    (
        "E-PARSE-013",
        &[
            "babylon-bsl/src/bindings.rs",
            "babylon-bsl/src/grammar.rs",
            "babylon-bsl/src/mod_anchors.rs",
        ],
        "bsl-language.rst:630 — \"the keyword set is closed. An unrecognized keyword is \
         E-PARSE-013\" at every one of the (independent) positions that has a closed \
         keyword set: bindings, grammar/graph-flag placement, and mod-anchors",
    ),
    (
        "E-PARSE-015",
        &[
            "babylon-bsl/src/causal_contract.rs",
            "babylon-bsl/src/grammar.rs",
        ],
        "bsl-language.rst:691,1249 — a symbol outside any parser-owned closed set is \
         E-PARSE-015; grammar.rs owns form/update-op sets, while ADR224's causal_contract.rs \
         owns the closed :role and :evidence values",
    ),
    (
        "E-PARSE-022",
        &["babylon-bsl/src/bindings.rs", "babylon-bsl/src/scope.rs"],
        "bsl-language.rst:779,897,1145 — self/it is a reserved name at both a binding's own \
         name position and the general element-name check",
    ),
    (
        "E-PARSE-030",
        &["babylon-bsl/src/bindings.rs", "babylon-bsl/src/scope.rs"],
        "bsl-language.rst:903,1166,4484 — a name colliding with an existing binding/:as name \
         is E-PARSE-030 at both the binding parser and the general element-name check",
    ),
    (
        "E-TYPE-010",
        &["babylon-bsl/src/domain.rs", "babylon-bsl/src/scope.rs"],
        "bsl-language.rst:733,854,936 — a foreign node type read outside a fold body over \
         that type is E-TYPE-010 at both the domain-resolution site and foreign-field scoping",
    ),
    (
        "E-TYPE-011",
        &[
            "babylon-bsl/src/grammar.rs",
            "babylon-bsl/src/vocabulary.rs",
        ],
        "bsl-language.rst:731,968,982,3852 — an <enum-ref> of the wrong kind is E-TYPE-011 \
         at both the grammar's enum-ref-kind check and vocabulary's own kind guard",
    ),
];

/// Run both (a) and (b). `roots`: `[decisions_dir, src_scan_root]`,
/// positional with repo-relative defaults (`ai/decisions`, `.` under
/// `<root>/rust/crates`) — a test fixture overrides either to point at a
/// tiny stand-in tree without touching the real estate.
///
/// # Errors
/// A string on any filesystem read failure — an infrastructure failure,
/// distinct from a namespace finding.
pub fn run(repo: &Repo, roots: &[String]) -> Result<Vec<Finding>, String> {
    let decisions_dir = roots
        .first()
        .map_or_else(|| repo.root.join("ai/decisions"), |p| repo.root.join(p));
    let src_scan_root = roots
        .get(1)
        .map_or_else(|| repo.root.join("rust/crates"), |p| repo.root.join(p));

    let mut findings = check_adr_registry(repo, &decisions_dir)?;
    findings.extend(check_e_codes(repo, &src_scan_root)?);
    Ok(findings)
}

// ── (a) ai/decisions/ <-> index.yaml ────────────────────────────────────

fn adr_filename_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^ADR(\d+)_").expect("static ADR filename regex must compile"))
}

fn index_key_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^  (ADR\d+_\S+):").expect("static index.yaml key regex must compile")
    })
}

fn check_adr_registry(repo: &Repo, decisions_dir: &Path) -> Result<Vec<Finding>, String> {
    let mut findings = Vec::new();
    let mut stems: Vec<String> = Vec::new();
    let mut numbers: BTreeMap<u64, Vec<String>> = BTreeMap::new();

    let entries = std::fs::read_dir(decisions_dir)
        .map_err(|e| format!("{}: {e}", decisions_dir.display()))?;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "yaml") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if stem == "index" {
            continue;
        }
        if let Some(caps) = adr_filename_re().captures(stem) {
            if let Ok(n) = caps[1].parse::<u64>() {
                numbers.entry(n).or_default().push(stem.to_owned());
            }
        }
        stems.push(stem.to_owned());
    }
    stems.sort();

    for (n, files) in &numbers {
        if files.len() > 1 {
            let mut files = files.clone();
            files.sort();
            findings.push(Finding {
                check: CHECK,
                file: repo.display_path(decisions_dir),
                line: 0,
                what: format!("ADR{n} is not unique"),
                evidence: files.join(", "),
                severity: Severity::Fail,
            });
        }
    }

    let index_path = decisions_dir.join("index.yaml");
    let index_text = std::fs::read_to_string(&index_path)
        .map_err(|e| format!("{}: {e}", index_path.display()))?;
    let mut keys: Vec<String> = index_key_re()
        .captures_iter(&index_text)
        .map(|c| c[1].to_owned())
        .collect();
    keys.sort();
    keys.dedup();

    findings.extend(set_diff_findings(
        repo,
        decisions_dir,
        &stems,
        &keys,
        "has no index.yaml entry",
        "has no backing ADR file",
    ));
    Ok(findings)
}

fn set_diff_findings(
    repo: &Repo,
    decisions_dir: &Path,
    stems: &[String],
    keys: &[String],
    stem_only_msg: &str,
    key_only_msg: &str,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let decisions_display = repo.display_path(decisions_dir);
    for stem in stems {
        if !keys.contains(stem) {
            findings.push(Finding {
                check: CHECK,
                file: decisions_display.clone(),
                line: 0,
                what: format!("{stem} {stem_only_msg}"),
                evidence: format!("{decisions_display}/{stem}.yaml exists"),
                severity: Severity::Fail,
            });
        }
    }
    for key in keys {
        if !stems.contains(key) {
            findings.push(Finding {
                check: CHECK,
                file: repo.display_path(&decisions_dir.join("index.yaml")),
                line: 0,
                what: format!("{key} {key_only_msg}"),
                evidence: "index.yaml key with no matching file stem".to_owned(),
                severity: Severity::Fail,
            });
        }
    }
    findings
}

// ── (b) E-code cross-file duplicate scan ────────────────────────────────

fn e_code_literal_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#""(E-[A-Z]+-[0-9]+)""#).expect("static E-code literal regex must compile")
    })
}

fn check_e_codes(repo: &Repo, src_scan_root: &Path) -> Result<Vec<Finding>, String> {
    let mut occurrences: BTreeMap<String, Vec<(String, usize)>> = BTreeMap::new();
    for file in list_src_rs_files(src_scan_root)? {
        for (line_no, code) in scan_e_codes(&file)? {
            let rel = crate_relative(repo, &file);
            occurrences.entry(code).or_default().push((rel, line_no));
        }
    }

    let mut findings = Vec::new();
    for (code, locations) in occurrences {
        let mut files: Vec<String> = locations.iter().map(|(f, _)| f.clone()).collect();
        files.sort();
        files.dedup();
        if files.len() <= 1 {
            continue; // same error class (this codebase's file-grained proxy) — fine
        }
        if is_allowlisted(&code, &files) {
            continue;
        }
        let sites = locations
            .iter()
            .map(|(f, l)| format!("{f}:{l}"))
            .collect::<Vec<_>>()
            .join(", ");
        let (first_file, first_line) = locations.first().cloned().unwrap_or_default();
        findings.push(Finding {
            check: CHECK,
            file: first_file,
            line: first_line,
            what: format!(
                "{code} is emitted by {} distinct files, not allowlisted",
                files.len()
            ),
            evidence: sites,
            severity: Severity::Fail,
        });
    }
    Ok(findings)
}

fn is_allowlisted(code: &str, files: &[String]) -> bool {
    ALLOWLIST.iter().any(|(c, allowed_files, _citation)| {
        if *c != code {
            return false;
        }
        let mut allowed: Vec<&str> = allowed_files.to_vec();
        allowed.sort_unstable();
        let found: Vec<&str> = files.iter().map(String::as_str).collect();
        allowed == found
    })
}

/// `<crate-name>/src/...` — stable across worktrees/absolute-path
/// differences, and what [`ALLOWLIST`] entries are written against.
fn crate_relative(repo: &Repo, file: &Path) -> String {
    let display = repo.display_path(file);
    display
        .strip_prefix("rust/crates/")
        .map_or(display.clone(), str::to_owned)
}

fn list_src_rs_files(root: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut files = Vec::new();
    // One-level-then-recurse-into-src walk: `root` is `rust/crates`, and
    // only each crate's `src/` (never its `tests/`) is in scope — see the
    // module doc. A crate without a `src/` dir is skipped, not an error.
    // `bsl-lint` itself is excluded: it is the auditor of the BSL spec-code
    // surface, not a member of it — [`ALLOWLIST`]'s own citations quote
    // codes like `"E-LOAD-001"` as data, and without this exclusion the
    // scanner would see its own allowlist as a THIRD (or FOURTH) emitter of
    // every code it just finished exempting, self-defeatingly.
    let entries = std::fs::read_dir(root).map_err(|e| format!("{}: {e}", root.display()))?;
    for crate_dir in entries.filter_map(Result::ok).map(|e| e.path()) {
        if crate_dir.file_name().is_some_and(|n| n == "bsl-lint") {
            continue;
        }
        let src = crate_dir.join("src");
        if src.is_dir() {
            walk_rs_files(&src, &mut files)?;
        }
    }
    // I1: std::fs::read_dir's iteration order is filesystem-dependent, not
    // sorted — mirror citation_drift.rs's list_bsl_files (which already
    // sorts) so a finding's evidence string and (file, line) header are
    // deterministic across machines, not an accident of directory layout.
    files.sort();
    Ok(files)
}

fn walk_rs_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            walk_rs_files(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

/// One file's E-code literal occurrences OUTSIDE comments and
/// `#[cfg(test)] mod … { … }` blocks (assertions aren't emission sites; see
/// module doc).
fn scan_e_codes(file: &Path) -> Result<Vec<(usize, String)>, String> {
    let text = std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
    let mut hits = Vec::new();
    let mut skip_depth: Option<i32> = None;
    let mut cfg_test_pending = false;
    for (i, line) in text.lines().enumerate() {
        let stripped = line.trim();
        if let Some(depth) = skip_depth {
            let new_depth = depth + brace_delta(line);
            skip_depth = if new_depth <= 0 {
                None
            } else {
                Some(new_depth)
            };
            continue;
        }
        if stripped == "#[cfg(test)]" {
            cfg_test_pending = true;
            continue;
        }
        if cfg_test_pending {
            cfg_test_pending = false;
            if stripped.contains("mod ") && stripped.contains('{') {
                let depth = brace_delta(line);
                skip_depth = if depth <= 0 { None } else { Some(depth) };
            }
            continue;
        }
        if stripped.starts_with("//") {
            continue;
        }
        for caps in e_code_literal_re().captures_iter(line) {
            hits.push((i + 1, caps[1].to_owned()));
        }
    }
    Ok(hits)
}

fn brace_delta(line: &str) -> i32 {
    let opens = i32::try_from(line.matches('{').count()).unwrap_or(i32::MAX);
    let closes = i32::try_from(line.matches('}').count()).unwrap_or(i32::MAX);
    opens - closes
}
