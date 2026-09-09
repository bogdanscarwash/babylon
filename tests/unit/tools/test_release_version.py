"""Git-derived development identity and deliberate native release versions."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

SCRIPT = Path(__file__).resolve().parents[3] / "tools/release_version.py"


def git(repo: Path, *args: str) -> str:
    return subprocess.check_output(("git", *args), cwd=repo, text=True).strip()


def commit(repo: Path, text: str = "change") -> str:
    (repo / "payload").write_text(text, encoding="utf-8")
    git(repo, "add", ".")
    git(repo, "commit", "-qm", f"test: {text}")
    return git(repo, "rev-parse", "HEAD")


@pytest.fixture
def repo(tmp_path: Path) -> Path:
    git(tmp_path, "init", "-q", "-b", "dev")
    git(tmp_path, "config", "user.name", "Release version contract")
    git(tmp_path, "config", "user.email", "version@example.invalid")
    (tmp_path / "pyproject.toml").write_text(
        '[project]\nname = "fixture"\nversion = "0.4.0"\n', encoding="utf-8"
    )
    commit(tmp_path, "initial")
    git(tmp_path, "tag", "v0.3.0")
    return tmp_path


def run(repo: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        (sys.executable, str(SCRIPT), *args),
        cwd=repo,
        text=True,
        capture_output=True,
        env={
            **os.environ,
            "PATH": str(Path(sys.executable).parent) + os.pathsep + os.environ["PATH"],
        },
        check=False,
        timeout=10,
    )


def identity(repo: Path) -> dict[str, object]:
    result = run(repo, "--json")
    assert result.returncode == 0, result.stderr
    return json.loads(result.stdout)


def test_every_new_commit_advances_patch_without_bump_commits(repo: Path) -> None:
    first = commit(repo, "first")
    assert identity(repo)["version"] == f"0.3.1+g{first[:12]}"
    second = commit(repo, "second")
    assert identity(repo)["version"] == f"0.3.2+g{second[:12]}"
    assert identity(repo)["release_version"] == "0.4.0"


def test_merge_counts_all_newly_reachable_commits_and_merge_itself(repo: Path) -> None:
    git(repo, "checkout", "-qb", "feature")
    commit(repo, "feature one")
    commit(repo, "feature two")
    git(repo, "checkout", "-q", "dev")
    git(repo, "merge", "--no-ff", "-qm", "test: merge feature", "feature")
    assert identity(repo)["distance"] == 3
    assert str(identity(repo)["version"]).startswith("0.3.3+")


def test_release_resets_patch_and_next_commit_is_one(repo: Path) -> None:
    commit(repo, "ready")
    git(repo, "tag", "-a", "v0.4.0", "-m", "v0.4.0")
    result = run(repo, "--release-tag", "v0.4.0")
    assert result.returncode == 0, result.stderr
    assert result.stdout.strip() == "0.4.0"
    commit(repo, "next")
    assert identity(repo)["anchor"] == "v0.4.0"
    assert str(identity(repo)["version"]).startswith("0.4.1+")


def test_branches_with_same_patch_have_distinct_commit_metadata(repo: Path) -> None:
    git(repo, "checkout", "-qb", "first")
    commit(repo, "one")
    first = identity(repo)["version"]
    git(repo, "checkout", "-qb", "second", "v0.3.0")
    commit(repo, "two")
    second = identity(repo)["version"]
    assert first != second
    assert str(first).split("+")[0] == str(second).split("+")[0] == "0.3.1"


def test_dirty_identity_changes_with_tracked_and_untracked_bytes(repo: Path) -> None:
    clean = identity(repo)
    (repo / "payload").write_text("edited", encoding="utf-8")
    first = identity(repo)
    assert first["dirty"] is True
    assert ".dirty." in str(first["version"])
    assert first["version"] != clean["version"]
    (repo / "untracked").write_text("one", encoding="utf-8")
    second = identity(repo)
    (repo / "untracked").write_text("two", encoding="utf-8")
    third = identity(repo)
    assert len({first["version"], second["version"], third["version"]}) == 3


def test_refuses_shallow_history_even_when_anchor_tag_is_present(
    repo: Path, tmp_path: Path
) -> None:
    clone = tmp_path / "shallow"
    git(repo, "clone", "-q", "--depth=1", repo.as_uri(), str(clone))
    result = run(clone, "--json")
    assert result.returncode != 0
    assert "shallow" in result.stderr


def test_refuses_missing_anchor(repo: Path) -> None:
    git(repo, "tag", "-d", "v0.3.0")
    result = run(repo)
    assert result.returncode != 0
    assert "no reachable canonical release" in result.stderr


def test_explicit_bootstrap_uses_recorded_version_until_first_release(repo: Path) -> None:
    git(repo, "tag", "-d", "v0.3.0")
    anchor = git(repo, "rev-parse", "HEAD")
    with (repo / "pyproject.toml").open("a", encoding="utf-8") as stream:
        stream.write(
            f'\n[tool.babylon.versioning]\nbootstrap_commit = "{anchor}"\n'
            'bootstrap_version = "0.3.0"\n'
        )
    commit(repo, "declare bootstrap")
    assert str(identity(repo)["version"]).startswith("0.3.1+")
    assert identity(repo)["anchor"] == anchor
    git(repo, "tag", "v0.4.0")
    assert identity(repo)["anchor"] == "v0.4.0"


def test_release_rejects_different_tag_version_and_dirty_checkout(repo: Path) -> None:
    assert run(repo, "--release-tag", "v0.3.0").returncode != 0
    git(repo, "tag", "v0.4.0")
    (repo / "payload").write_text("dirty", encoding="utf-8")
    result = run(repo, "--release-tag", "v0.4.0")
    assert result.returncode != 0
    assert "dirty" in result.stderr


def test_release_refuses_tag_at_different_commit(repo: Path) -> None:
    git(repo, "tag", "v0.4.0")
    commit(repo, "after release")
    result = run(repo, "--release-tag", "v0.4.0")
    assert result.returncode != 0
    assert "HEAD" in result.stderr


def test_release_candidate_version_is_canonical_single_source(repo: Path) -> None:
    result = run(repo, "--release-version")
    assert result.returncode == 0, result.stderr
    assert result.stdout.strip() == "0.4.0"


def test_commitizen_rejects_invalid_new_message_and_accepts_conventional(repo: Path) -> None:
    base = git(repo, "rev-parse", "HEAD")
    head = commit(repo, "valid")
    result = run(repo, "--check-commits", base, head)
    assert result.returncode == 0, result.stderr
    git(repo, "commit", "--allow-empty", "-qm", "invalid message")
    result = run(repo, "--check-commits", base, git(repo, "rev-parse", "HEAD"))
    assert result.returncode != 0
    assert "Commitizen refused" in result.stderr


def test_commitizen_clamps_only_history_before_explicit_policy_anchor(repo: Path) -> None:
    base = git(repo, "rev-parse", "HEAD")
    git(repo, "commit", "--allow-empty", "-qm", "historical nonconventional message")
    bootstrap = git(repo, "rev-parse", "HEAD")
    with (repo / "pyproject.toml").open("a", encoding="utf-8") as stream:
        stream.write(
            f'\n[tool.babylon.versioning]\nbootstrap_commit = "{bootstrap}"\n'
            'bootstrap_version = "0.3.0"\n'
        )
    head = commit(repo, "policy era")
    result = run(repo, "--check-commits", base, head)
    assert result.returncode == 0, result.stderr
    assert f"{bootstrap}..{head}" in result.stdout


def test_commit_range_refuses_nonancestor_and_short_sha(repo: Path) -> None:
    base = commit(repo, "first")
    git(repo, "checkout", "-qb", "unrelated", "v0.3.0")
    head = commit(repo, "unrelated")
    result = run(repo, "--check-commits", base, head)
    assert result.returncode != 0
    assert "not an ancestor" in result.stderr
    result = run(repo, "--check-commits", base[:12], head)
    assert result.returncode != 0
    assert "full lowercase" in result.stderr


def test_historical_tags_do_not_replace_the_native_bootstrap(repo: Path) -> None:
    git(repo, "tag", "v1.0.0")
    bootstrap = commit(repo, "native cutover")
    with (repo / "pyproject.toml").open("a", encoding="utf-8") as stream:
        stream.write(
            f'\n[tool.babylon.versioning]\nbootstrap_commit = "{bootstrap}"\n'
            'bootstrap_version = "0.3.0"\n'
        )
    commit(repo, "version policy")
    assert identity(repo)["anchor"] == bootstrap
    assert str(identity(repo)["version"]).startswith("0.3.1+")
    git(repo, "tag", "v0.4.0")
    assert identity(repo)["anchor"] == "v0.4.0"


def test_commit_range_after_bootstrap_does_not_recheck_excluded_messages(repo: Path) -> None:
    bootstrap = git(repo, "rev-parse", "HEAD")
    with (repo / "pyproject.toml").open("a", encoding="utf-8") as stream:
        stream.write(
            f'\n[tool.babylon.versioning]\nbootstrap_commit = "{bootstrap}"\n'
            'bootstrap_version = "0.3.0"\n'
        )
    base = commit(repo, "first policy commit")
    head = commit(repo, "second policy commit")
    result = run(repo, "--check-commits", base, head)
    assert result.returncode == 0, result.stderr
    assert f"{base}..{head}" in result.stdout
