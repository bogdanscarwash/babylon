//! §0.2's five founding-constraint guards (task-7-brief.md 7.1-7.5), each
//! mechanically checked here and each proven red under its own hand
//! mutation (the PR body carries the mutation table; the drill is not
//! re-run automatically — a hand mutation followed by a revert, same as
//! this train's other TDD red phases, just not committed).
//!
//! **The one brain, restated** (plan §0.2): `babylon-bsl`
//! (+ `babylon-tick`'s registry assembly) decides what is wrong and what
//! the offending thing is called; `babylon-ls` decides where to draw the
//! squiggle, and nothing else. These five sentinels hold that law
//! mechanically, since Rust has no `lint:imports`-equivalent for most of
//! it.
//!
//! ## Scope: production code only — a decision this file had to make, not
//! one the brief spelled out
//!
//! 7.1 and 7.2 ban patterns "under `src/**`" with no stated carve-out for
//! test code. Taken completely literally, that wording is already violated
//! by Task 6's own merged, reviewed code: `src/diagnostics.rs`'s
//! `#[cfg(test)] mod tests` block asserts against literal codes like
//! `"E-LEX-003"` and calls `d.message.contains("orphan.bsl")`
//! (`src/pass.rs:217`, also inside its file's test module), and three
//! `src/diagnostics.rs` doc comments cite `E-LOAD-001`/`E-LOAD-002` in
//! prose explaining the classifier they sit above. None of that is the
//! violation §0.2 names — the founding law is about what the SERVER'S OWN
//! RUNTIME LOGIC does (never mint a code, never scan a message's prose to
//! find a location) — a test asserting on an expected output string, or a
//! doc comment citing an example, does neither. So every scan below counts
//! a line as "production" only when it is neither a `//`/`///`/`//!`
//! comment nor part of a file's trailing `#[cfg(test)]` module (see
//! [`production_lines`]'s own doc for exactly how that second exclusion is
//! bounded and guarded against silently under-scanning a future file that
//! stops following this crate's one-test-module-per-file convention).
//!
//! This same reasoning is why 7.2's "no exceptions" (plan §6.2) is read as
//! "no allowlisted PRODUCTION module" (the thing §6.2 discusses removing
//! was a production feature, `data.d_records`), not as "no test may ever
//! call `.contains` on a string."
//!
//! ## Why 7.2 and 7.4 key off dependency declarations, not bare substrings
//!
//! A textual ban on `.find(` or `rand` as bare substrings would fail
//! against this crate's own innocent code: `src/lifecycle.rs:98` calls
//! `Iterator::find` on a list of manifest-path candidates (nothing to do
//! with an error message), and the word "operand" — `by_operand_index`,
//! `ErrorIdentity::Operand`, `operand_position`, … — appears dozens of
//! times across `src/locator.rs` alone, every one a false positive for a
//! bare `"rand"` substring scan. So:
//! - 7.2's `.contains(`/`.split(`/`.find(` ban is scoped to receivers that
//!   look like an error's rendered text (`message.`, `msg.`,
//!   `to_string().` — see [`MESSAGE_LIKE_RECEIVERS`]), matching the
//!   brief's own mutation example (`msg.contains("node")`) exactly.
//! - 7.2's `regex` ban and 7.4's `rand` ban are read from `Cargo.toml`
//!   dependency declarations ([`cargo_toml_declares_dependency`]), not
//!   from source text — precise by construction, since Rust cannot call
//!   into either crate without the dependency existing first.

use std::fs;
use std::path::{Path, PathBuf};

/// Static bound on how many `.rs` files this scanner will walk under
/// `src/` (Power-of-10 rule 2). The crate has 10 today; this leaves
/// generous headroom without being unbounded.
const MAX_FILES_UNDER_SRC: usize = 64;

/// Static bound on how many lines of any one file this scanner will walk
/// (Power-of-10 rule 2). The crate's largest file is `diagnostics.rs` at
/// ~730 lines today.
const MAX_LINES_PER_FILE: usize = 5000;

/// Static bound on how many sibling crate directories sentinel 7.5 will
/// walk (Power-of-10 rule 2). The workspace has 7 members today.
const MAX_SIBLING_CRATES: usize = 32;

/// Every `.rs` file directly under this crate's `src/` — flat today, no
/// subdirectories (`find rust/crates/babylon-ls/src -type d` returns only
/// `src` itself). A subdirectory appearing later is a real structural
/// change this scanner does not silently absorb: it panics loudly rather
/// than under-scanning.
fn rs_files_under_src() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let entries = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("sentinels: cannot read {}: {e}", dir.display()));
    let mut files = Vec::new();
    for (count, entry) in entries.enumerate() {
        assert!(
            count < MAX_FILES_UNDER_SRC,
            "sentinels: src/ has grown past the scanner's static {MAX_FILES_UNDER_SRC}-file \
             bound — raise the constant deliberately, don't silently truncate the scan"
        );
        let path = entry
            .unwrap_or_else(|e| {
                panic!(
                    "sentinels: dir entry read failed under {}: {e}",
                    dir.display()
                )
            })
            .path();
        assert!(
            !path.is_dir(),
            "sentinels: src/ gained a subdirectory ({}) — this scanner is flat-only by design; \
             teach it to recurse before trusting any sentinel's verdict",
            path.display()
        );
        if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    files.sort();
    files
}

/// One line of PRODUCTION code: `(path, 1-based line number, text)`. See
/// the module doc's "Scope" section for what "production" excludes and
/// why. The `#[cfg(test)]`-to-EOF exclusion is guarded, not assumed
/// blindly: a SECOND `#[cfg(test)]` marker in the same file would mean
/// this crate's one-trailing-test-module convention broke, and this
/// function panics rather than silently scanning less than it claims to.
fn production_lines() -> Vec<(PathBuf, usize, String)> {
    let mut out = Vec::new();
    for path in rs_files_under_src() {
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("sentinels: cannot read {}: {e}", path.display()));
        let mut in_test_module = false;
        for (idx, line) in text.lines().enumerate() {
            assert!(
                idx < MAX_LINES_PER_FILE,
                "sentinels: {} exceeds the scanner's static {MAX_LINES_PER_FILE}-line bound — \
                 raise the constant deliberately, don't silently truncate the scan",
                path.display()
            );
            let line_no = idx + 1;
            let trimmed = line.trim_start();
            if trimmed == "#[cfg(test)]" {
                assert!(
                    !in_test_module,
                    "sentinels: {}:{line_no} is a SECOND `#[cfg(test)]` marker in one file — \
                     this scanner assumes exactly one per file, running to EOF; teach it to \
                     track brace nesting before trusting this file's verdict",
                    path.display()
                );
                in_test_module = true;
            }
            if in_test_module || trimmed.starts_with("//") {
                continue;
            }
            out.push((path.clone(), line_no, line.to_owned()));
        }
    }
    out
}

/// The first full `E-<FAMILY>-<DIGITS>` literal in `line`, if any. A bare
/// family prefix (`E-LOAD`, no trailing `-<digits>`) never matches — §0.2
/// row 7.1 explicitly allows those. Bounded by `line`'s character count
/// (Power-of-10 rule 2); operates on `char_indices` rather than raw bytes
/// so the returned slice is always sliced at a valid UTF-8 boundary.
fn find_e_code_literal(line: &str) -> Option<&str> {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let n = chars.len();
    let mut i = 0usize;
    while i + 1 < n {
        if chars[i].1 == 'E' && chars[i + 1].1 == '-' {
            let mut j = i + 2;
            while j < n && chars[j].1.is_ascii_uppercase() {
                j += 1;
            }
            if j > i + 2 && j < n && chars[j].1 == '-' {
                let mut k = j + 1;
                while k < n && chars[k].1.is_ascii_digit() {
                    k += 1;
                }
                if k > j + 1 {
                    let start_byte = chars[i].0;
                    let end_byte = chars.get(k).map_or(line.len(), |c| c.0);
                    return Some(&line[start_byte..end_byte]);
                }
            }
        }
        i += 1;
    }
    None
}

/// Receivers whose trailing method call plausibly reads an ERROR's
/// rendered text — the shape sentinel 7.2 bans `.contains(`/`.split(`/
/// `.find(` on. Matches the brief's own mutation example (`msg.contains(
/// "node")`) and this crate's real field name (`message`) and the
/// generic `Display`-then-scan route (`to_string().`).
const MESSAGE_LIKE_RECEIVERS: &[&str] = &["message.", "msg.", "to_string()."];

/// The three prose-scanning methods §0.2 row 7.2 names.
const BANNED_STRING_SCAN_METHODS: &[&str] = &["contains(", "split(", "find("];

/// The first `<message-like receiver>.<banned method>(` needle found in
/// `line`, if any.
fn find_prose_parse(line: &str) -> Option<String> {
    for receiver in MESSAGE_LIKE_RECEIVERS {
        for method in BANNED_STRING_SCAN_METHODS {
            let needle = format!("{receiver}{method}");
            if line.contains(&needle) {
                return Some(needle);
            }
        }
    }
    None
}

/// True when `text` (a `Cargo.toml`'s contents) declares a dependency
/// named exactly `crate_name`, in either `key = value` form
/// (`regex = "1"`) or dotted-table form (`[dependencies.regex]`).
/// Deliberately NOT a bare substring search — `"regex-lite"` or a crate
/// whose name merely contains `crate_name` must not match.
fn cargo_toml_declares_dependency(text: &str, crate_name: &str) -> bool {
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some((key, _value)) = trimmed.split_once('=') {
            if key.trim() == crate_name {
                return true;
            }
        }
        if let Some(inner) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            if inner.rsplit('.').next() == Some(crate_name) {
                return true;
            }
        }
    }
    false
}

/// (path substring, reason) pairs exempted from sentinels 7.3/7.4. Empty
/// today: Task 5.5's size-rotated log sink (plan §5.5) — the ONE module
/// this train ever names as a carve-out for either sentinel — has not
/// landed in this worktree yet (`rg -l 'log_sink|LogSink' src` finds
/// nothing; `Cargo.toml` has no `log` dependency). When it lands, add
/// exactly one entry here naming its path and stating why a log
/// timestamp sits outside the diagnostics determinism contract (plan
/// §5.5/§0.2) — do not loosen either scan itself.
const READ_ONLY_AND_DETERMINISM_ALLOWLIST: &[(&str, &str)] = &[];

fn is_allowlisted(path: &Path, allowlist: &[(&str, &str)]) -> bool {
    let path_text = path.to_string_lossy();
    allowlist
        .iter()
        .any(|(substr, _reason)| path_text.contains(substr))
}

// ---------------------------------------------------------------------
// 7.1 — E-code literal
// ---------------------------------------------------------------------

#[test]
fn sentinel_7_1_no_full_e_code_literal_in_production_src() {
    let mut violations = Vec::new();
    for (path, line_no, line) in production_lines() {
        if let Some(literal) = find_e_code_literal(&line) {
            violations.push(format!("{}:{line_no}: `{literal}`", path.display()));
        }
    }
    assert!(
        violations.is_empty(),
        "sentinel 7.1 (E-code literal, task-7-brief.md/plan §0.2 row 7.1): production code \
         under src/** must never hardcode a full `E-<FAM>-NNN` literal — codes come verbatim \
         from `spec_code()`/`LexCode`/etc, minted nowhere in this crate. Bare family prefixes \
         (`E-LOAD`) are fine. Violations:\n{}",
        violations.join("\n")
    );
}

// ---------------------------------------------------------------------
// 7.2 — no prose-parsing
// ---------------------------------------------------------------------

#[test]
fn sentinel_7_2_no_regex_dependency() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("sentinels: cannot read {}: {e}", manifest_path.display()));
    assert!(
        !cargo_toml_declares_dependency(&text, "regex"),
        "sentinel 7.2 (no prose-parsing, task-7-brief.md/plan §0.2 row 7.2): {} must never \
         depend on `regex` — no exceptions (plan §6.2 removed the one production feature that \
         wanted one, d_records; it did not weaken this guard)",
        manifest_path.display()
    );
}

#[test]
fn sentinel_7_2_no_prose_parsing_of_error_messages() {
    let mut violations = Vec::new();
    for (path, line_no, line) in production_lines() {
        if let Some(needle) = find_prose_parse(&line) {
            violations.push(format!("{}:{line_no}: `{needle}`", path.display()));
        }
    }
    assert!(
        violations.is_empty(),
        "sentinel 7.2 (no prose-parsing, task-7-brief.md/plan §0.2 row 7.2): production code \
         under src/** must never call .contains(/.split(/.find( on an error's rendered text \
         (`message`/`msg`/`.to_string()`) — no exceptions. The server locates via \
         `ErrorIdentity` + `SpanTable`, never by scanning prose. Violations:\n{}",
        violations.join("\n")
    );
}

// ---------------------------------------------------------------------
// 7.3 — read-only
// ---------------------------------------------------------------------

/// The four filesystem-mutation shapes §0.2 row 7.3 names.
const FS_WRITE_PATTERNS: &[&str] = &["fs::write", "File::create", "OpenOptions", "remove_file"];

#[test]
fn sentinel_7_3_read_only_no_filesystem_writes_outside_the_allowlist() {
    let mut violations = Vec::new();
    for (path, line_no, line) in production_lines() {
        if is_allowlisted(&path, READ_ONLY_AND_DETERMINISM_ALLOWLIST) {
            continue;
        }
        for pattern in FS_WRITE_PATTERNS {
            if line.contains(pattern) {
                violations.push(format!("{}:{line_no}: `{pattern}`", path.display()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "sentinel 7.3 (read-only, task-7-brief.md/plan §0.2 row 7.3): {} holds no state the \
         loader does not and never writes content (plan §1 constraint 1) — the load path stays \
         the only door. Allowlist this file's path in READ_ONLY_AND_DETERMINISM_ALLOWLIST with \
         a stated reason ONLY for the Task-5.5 log-sink module. Violations:\n{}",
        env!("CARGO_PKG_NAME"),
        violations.join("\n")
    );
}

// ---------------------------------------------------------------------
// 7.4 — determinism
// ---------------------------------------------------------------------

/// The two wall-clock APIs §0.2 row 7.4 names by pattern (the third,
/// `rand`, is checked via the dependency declaration instead — see the
/// module doc's "Why 7.2 and 7.4 key off dependency declarations" section
/// for why a bare `"rand"` substring scan would misfire on `operand`).
const WALL_CLOCK_PATTERNS: &[&str] = &["SystemTime::now", "Instant::now"];

#[test]
fn sentinel_7_4_no_wall_clock_outside_the_allowlist() {
    let mut violations = Vec::new();
    for (path, line_no, line) in production_lines() {
        if is_allowlisted(&path, READ_ONLY_AND_DETERMINISM_ALLOWLIST) {
            continue;
        }
        for pattern in WALL_CLOCK_PATTERNS {
            if line.contains(pattern) {
                violations.push(format!("{}:{line_no}: `{pattern}`", path.display()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "sentinel 7.4 (determinism, task-7-brief.md/plan §0.2 row 7.4): same (workspace text, \
         manifest) must produce the same diagnostics byte for byte (plan §1 constraint 2) — no \
         wall-clock. Allowlist this file's path in READ_ONLY_AND_DETERMINISM_ALLOWLIST with a \
         stated reason ONLY for the Task-5.5 log-sink module. Violations:\n{}",
        violations.join("\n")
    );
}

#[test]
fn sentinel_7_4_no_rand_dependency() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("sentinels: cannot read {}: {e}", manifest_path.display()));
    assert!(
        !cargo_toml_declares_dependency(&text, "rand"),
        "sentinel 7.4 (determinism, task-7-brief.md/plan §0.2 row 7.4): {} must never depend \
         on `rand` outside the Task-5.5 log-sink module's own needs (it has none — logging a \
         line needs no randomness)",
        manifest_path.display()
    );
}

// ---------------------------------------------------------------------
// 7.5 — layering
// ---------------------------------------------------------------------

/// Every sibling crate's `Cargo.toml` under `rust/crates/`, excluding
/// `babylon-ls`'s own (sentinel 7.5 bans OTHER crates depending on it;
/// its own manifest naturally names itself via `[package] name =`).
fn other_crate_manifests() -> Vec<PathBuf> {
    let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("sentinels: babylon-ls's CARGO_MANIFEST_DIR has no parent")
        .to_path_buf();
    let entries = fs::read_dir(&crates_dir)
        .unwrap_or_else(|e| panic!("sentinels: cannot read {}: {e}", crates_dir.display()));
    let mut manifests = Vec::new();
    for (count, entry) in entries.enumerate() {
        assert!(
            count < MAX_SIBLING_CRATES,
            "sentinels: {} has grown past the scanner's static {MAX_SIBLING_CRATES}-crate \
             bound — raise the constant deliberately, don't silently truncate the scan",
            crates_dir.display()
        );
        let path = entry
            .unwrap_or_else(|e| {
                panic!(
                    "sentinels: dir entry read failed under {}: {e}",
                    crates_dir.display()
                )
            })
            .path();
        if path.file_name().and_then(|n| n.to_str()) == Some("babylon-ls") {
            continue;
        }
        let manifest = path.join("Cargo.toml");
        if manifest.is_file() {
            manifests.push(manifest);
        }
    }
    manifests.sort();
    manifests
}

#[test]
fn sentinel_7_5_no_other_crate_depends_on_babylon_ls() {
    let mut violations = Vec::new();
    for manifest in other_crate_manifests() {
        let text = fs::read_to_string(&manifest)
            .unwrap_or_else(|e| panic!("sentinels: cannot read {}: {e}", manifest.display()));
        if cargo_toml_declares_dependency(&text, "babylon-ls") {
            violations.push(manifest.display().to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "sentinel 7.5 (layering, task-7-brief.md/plan §0.2 row 7.5): babylon-ls is a leaf ABOVE \
         the whole engine stack (kernel < models/formulas < topology < domain < persistence < \
         engine; intelligence observes separately, `src/lib.rs`'s own module doc) — nothing may \
         depend on it. Violating manifests:\n{}",
        violations.join("\n")
    );
}
