//! Integration tests for `bsl-lint citation-drift` (task brief W1.1/W1.3).
//! Spawns the compiled binary against `tests/fixtures/rules/stale_citation.bsl`
//! — every citation in that fixture targets the REAL frozen
//! `src/babylon/engine/systems/solidarity.py` at the `p27-python-freeze`
//! tag (202 lines), so these tests exercise the real git-tag resolution
//! path, not a stand-in.

use std::path::PathBuf;
use std::process::Command;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rules/stale_citation.bsl")
}

fn run_citation_drift() -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_bsl-lint"))
        .arg("citation-drift")
        .arg(fixture_path())
        .output()
        .expect("bsl-lint must run");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (output.status.code().unwrap_or(-1), stdout, stderr)
}

#[test]
fn a_span_beyond_the_frozen_files_line_count_fails() {
    let (code, stdout, stderr) = run_citation_drift();
    assert_eq!(code, 1, "stdout was:\n{stdout}\nstderr was:\n{stderr}");
    assert!(
        stdout.contains("E-SENTINEL citation-drift")
            && stdout.contains("out of bounds")
            && stdout.contains("solidarity.py:97-999"),
        "expected an out-of-bounds FAIL line, got:\n{stdout}\nstderr was:\n{stderr}"
    );
}

#[test]
fn an_in_bounds_span_with_no_nearby_keyword_warns_not_fails() {
    let (_, stdout, stderr) = run_citation_drift();
    let warn_line = stdout
        .lines()
        .find(|l| l.contains("solidarity.py:1-3"))
        .unwrap_or_else(|| {
            panic!("expected a solidarity.py:1-3 line, got:\n{stdout}\nstderr was:\n{stderr}")
        });
    assert!(
        warn_line.starts_with("W-SENTINEL"),
        "keyword-miss must WARN, not FAIL: {warn_line}"
    );
    assert!(warn_line.contains("no keyword nearby"), "{warn_line}");
}

#[test]
fn a_grounded_in_bounds_citation_with_a_nearby_keyword_is_clean() {
    let (_, stdout, stderr) = run_citation_drift();
    assert!(
        !stdout.contains("solidarity.py:1-14"),
        "the clean citation must produce no finding at all:\n{stdout}\nstderr was:\n{stderr}"
    );
}

#[test]
fn list_includes_citation_drift() {
    let output = Command::new(env!("CARGO_BIN_EXE_bsl-lint"))
        .arg("--list")
        .output()
        .expect("bsl-lint --list must run");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("citation-drift"));
    assert!(stdout.contains("namespace-unique"));
}

/// M5: a repo whose checkout lacks the `p27-python-freeze` tag (the exact
/// shape task-w1-review.md's Critical C1 named — a shallow/no-tags CI
/// checkout) must make citation-drift fail LOUD — exit 2, a distinct
/// infra-failure outcome — never exit 0/1 (which would misreport a missing
/// tag as "no drift" or "drift found", both lies about what happened).
/// Builds a real throwaway git repo with no tags at all, rather than mocking
/// `git`, so this exercises the actual `git ls-tree`/`git show` failure path
/// `repo.rs::tag_tree`/`show_tag_file` hit in a real shallow CI checkout.
fn scratch_repo_dir(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock must be after the epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("bsl-lint-{label}-{}-{nanos}", std::process::id()))
}

#[test]
fn a_missing_freeze_tag_exits_2_not_0_or_1() {
    let repo_dir = scratch_repo_dir("no-freeze-tag");
    std::fs::create_dir_all(&repo_dir).expect("create scratch repo dir");

    // Scrub hook-exported repo overrides: under a pre-push hook, an inherited
    // GIT_DIR makes this `git init` reinitialize the REAL repo instead of
    // creating scratch/.git — and the tag this test requires absent exists.
    let mut init_cmd = Command::new("git");
    for var in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_COMMON_DIR",
    ] {
        init_cmd.env_remove(var);
    }
    let init = init_cmd
        .args(["init", "--quiet"])
        .current_dir(&repo_dir)
        .status()
        .expect("git init must run");
    assert!(init.success(), "git init failed in {repo_dir:?}");

    let rules_dir = repo_dir.join("rules");
    std::fs::create_dir_all(&rules_dir).expect("create fixture rules dir");
    // A single rule citing a frozen (.py) path — enough to reach the
    // is_frozen branch of check_citation, which is what calls
    // repo.tag_tree(FREEZE_TAG) and hits the missing-tag failure.
    std::fs::write(
        rules_dir.join("no_tag.bsl"),
        "(rule fixture/needs-frozen-tag :role mechanic :evidence derived :material-basis \"cites a frozen file (widget.py:1-3).\" :fuel 64\n  \
         (bindings)\n  \
         (effects (update-node self social-class/agitation (add 0.01i))))\n",
    )
    .expect("write fixture .bsl");

    let output = Command::new(env!("CARGO_BIN_EXE_bsl-lint"))
        .current_dir(&repo_dir)
        .arg("citation-drift")
        .arg("rules/no_tag.bsl")
        .output()
        .expect("bsl-lint must run");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let _ = std::fs::remove_dir_all(&repo_dir);

    assert_eq!(
        output.status.code(),
        Some(2),
        "a repo with no p27-python-freeze tag must exit 2 (infra failure), \
         not 0 (clean) or 1 (a drift finding) — stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.is_empty(),
        "an infra failure must not ALSO print a drift finding line — that \
         would misreport a missing tag as a citation-drift verdict; stdout:\n{stdout}"
    );
    assert!(
        stderr.contains("bsl-lint: citation-drift:"),
        "the exit-2 path must name the failing check on stderr, got:\n{stderr}"
    );
}
