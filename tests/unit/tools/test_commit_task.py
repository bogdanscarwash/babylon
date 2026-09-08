"""The commit task preserves merge ancestry without bypassing Git hooks."""

from __future__ import annotations

import os
import subprocess
import tomllib
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
COMMIT_TASK = tomllib.loads((ROOT / ".mise.toml").read_text())["tasks"]["commit"]["run"]


def git(repo: Path, *arguments: str) -> str:
    return subprocess.check_output(
        ["git", *arguments], cwd=repo, text=True, stderr=subprocess.PIPE, timeout=15
    ).strip()


@pytest.fixture
def repo(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    checkout = tmp_path / "repo"
    checkout.mkdir()
    git(checkout, "init", "-q", "-b", "lane")
    git(checkout, "config", "user.name", "Commit task contract")
    git(checkout, "config", "user.email", "commit@example.invalid")
    git(checkout, "config", "commit.gpgsign", "false")
    (checkout / "payload").write_text("base\n")
    git(checkout, "add", "payload")
    git(checkout, "commit", "-qm", "test: base")

    # Keep the package runner bounded while real Git invokes its actual hook.
    # The outer pre-run and Git's commit hook leave independent evidence.
    evidence = tmp_path / "hook-evidence"
    monkeypatch.setenv("COMMIT_TEST_EVIDENCE", str(evidence))
    binary = tmp_path / "bin"
    binary.mkdir()
    uv = binary / "uv"
    uv.write_text(
        "#!/bin/sh\n"
        'test "$*" = "run pre-commit run" || exit 2\n'
        'printf "pre-run\\n" >> "$COMMIT_TEST_EVIDENCE"\n'
    )
    uv.chmod(0o755)
    monkeypatch.setenv("PATH", str(binary) + os.pathsep + os.environ["PATH"])
    hook = checkout / ".git/hooks/pre-commit"
    hook.write_text(
        "#!/bin/sh\n"
        'printf "git-hook\\n" >> "$COMMIT_TEST_EVIDENCE"\n'
        'exit "${COMMIT_TEST_HOOK_FAIL:-0}"\n'
    )
    hook.chmod(0o755)
    return checkout


def equivalent_tree_merge(repo: Path) -> tuple[str, str]:
    """Represent independent commits carrying the same refreshed source tree."""
    git(repo, "checkout", "-qb", "upstream")
    (repo / "payload").write_text("updated\n")
    git(repo, "add", "payload")
    git(repo, "commit", "-qm", "test: upstream update")
    git(repo, "checkout", "-q", "lane")
    (repo / "payload").write_text("updated\n")
    git(repo, "add", "payload")
    git(repo, "commit", "-qm", "test: equivalent local update")
    git(repo, "merge", "--no-ff", "--no-commit", "upstream")
    assert git(repo, "diff", "--cached") == ""
    (repo.parent / "hook-evidence").unlink()
    return git(repo, "rev-parse", "HEAD"), git(repo, "rev-parse", "MERGE_HEAD")


def run_commit(repo: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["bash", "-c", COMMIT_TASK],
        cwd=repo,
        env={**os.environ, "usage_message": "test: finish pending merge"},
        capture_output=True,
        text=True,
        check=False,
        timeout=30,
    )


def test_equivalent_tree_merge_records_both_parents_and_runs_hooks(repo: Path) -> None:
    parents = equivalent_tree_merge(repo)
    tree = git(repo, "rev-parse", "HEAD^{tree}")

    result = run_commit(repo)

    assert result.returncode == 0, result.stdout + result.stderr
    assert git(repo, "show", "-s", "--format=%P", "HEAD").split() == list(parents)
    assert git(repo, "rev-parse", "HEAD^{tree}") == tree
    evidence = (repo.parent / "hook-evidence").read_text().splitlines()
    assert "pre-run" in evidence
    assert "git-hook" in evidence


def test_ordinary_empty_commit_is_refused(repo: Path) -> None:
    head = git(repo, "rev-parse", "HEAD")

    result = run_commit(repo)

    assert result.returncode != 0
    assert "nothing staged" in result.stdout
    assert git(repo, "rev-parse", "HEAD") == head
    assert not (repo.parent / "hook-evidence").exists()


def test_equivalent_tree_merge_cannot_bypass_a_failing_git_hook(
    repo: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    head, merge_head = equivalent_tree_merge(repo)
    monkeypatch.setenv("COMMIT_TEST_HOOK_FAIL", "1")

    result = run_commit(repo)

    assert result.returncode != 0
    assert git(repo, "rev-parse", "HEAD") == head
    assert git(repo, "rev-parse", "MERGE_HEAD") == merge_head
    assert "git-hook" in (repo.parent / "hook-evidence").read_text().splitlines()
