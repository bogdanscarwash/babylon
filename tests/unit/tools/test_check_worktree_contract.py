"""Diagnostic contract for the worktree environment checker."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

from check_worktree_contract import (  # type: ignore[import-not-found]  # noqa: E402
    check_interpreter,
    check_lock_unmodified,
)


@pytest.mark.parametrize("version", ["3.12.12", "3.12.140", "3.13.14", "3.12.14"])
def test_interpreter_contract_requires_the_exact_patch(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, version: str
) -> None:
    (tmp_path / ".python-version").write_text("3.12.14\n")
    binary = tmp_path / ".venv/bin/python"
    binary.parent.mkdir(parents=True)
    binary.touch()
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr(
        subprocess,
        "run",
        lambda *_args, **_kwargs: subprocess.CompletedProcess([], 0, f"Python {version}\n", ""),
    )
    assert (check_interpreter() is None) == (version == "3.12.14")


def _run_git(arguments: list[str], *, cwd: Path) -> None:
    subprocess.run(["git", *arguments], cwd=cwd, check=True, capture_output=True, text=True)


def _lock_repo(path: Path) -> None:
    _run_git(["init", "-b", "main"], cwd=path)
    _run_git(["config", "user.email", "test@example.invalid"], cwd=path)
    _run_git(["config", "user.name", "Test User"], cwd=path)
    (path / "uv.lock").write_bytes(b"version = 1\n")
    _run_git(["add", "uv.lock"], cwd=path)
    _run_git(["commit", "-m", "initial lock"], cwd=path)


def _merge_lock_repo(path: Path, *, conflict: bool = False) -> None:
    _lock_repo(path)
    _run_git(["checkout", "-b", "incoming"], cwd=path)
    (path / "uv.lock").write_bytes(b"version = 2\n")
    _run_git(["commit", "-am", "accepted incoming lock"], cwd=path)
    _run_git(["checkout", "main"], cwd=path)
    if conflict:
        (path / "uv.lock").write_bytes(b"version = 3\n")
        _run_git(["commit", "-am", "divergent lock"], cwd=path)
    result = subprocess.run(
        ["git", "merge", "--no-ff", "--no-commit", "incoming"],
        cwd=path,
        check=False,
        capture_output=True,
    )
    assert result.returncode == (1 if conflict else 0)


@pytest.mark.unit
def test_lock_contract_accepts_clean_head_and_exact_staged_merge_parent(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _merge_lock_repo(tmp_path)
    monkeypatch.chdir(tmp_path)
    assert check_lock_unmodified() is None
    _run_git(["merge", "--abort"], cwd=tmp_path)
    assert check_lock_unmodified() is None


@pytest.mark.unit
@pytest.mark.parametrize("altered", ["index", "working", "both", "keep_head"])
def test_merge_lock_contract_rejects_any_other_resolution(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, altered: str
) -> None:
    _merge_lock_repo(tmp_path)
    lock = tmp_path / "uv.lock"
    if altered == "keep_head":
        _run_git(["checkout", "HEAD", "--", "uv.lock"], cwd=tmp_path)
    else:
        lock.write_bytes(b"version = 99\n")
        if altered in {"index", "both"}:
            _run_git(["add", "uv.lock"], cwd=tmp_path)
        if altered == "index":
            lock.write_bytes(b"version = 2\n")
    monkeypatch.chdir(tmp_path)
    monkeypatch.setenv("UV_FROZEN", "1")
    assert check_lock_unmodified() is not None


@pytest.mark.unit
def test_lock_contract_refuses_unmerged_index_even_with_exact_working_bytes(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _merge_lock_repo(tmp_path, conflict=True)
    (tmp_path / "uv.lock").write_bytes(b"version = 2\n")
    monkeypatch.chdir(tmp_path)
    assert check_lock_unmodified() is not None
    _run_git(["add", "uv.lock"], cwd=tmp_path)
    assert check_lock_unmodified() is None


@pytest.mark.unit
@pytest.mark.parametrize("merge_head", ["", "HEAD\n", "f" * 40 + "\n", "multiple_commits"])
def test_lock_contract_refuses_invalid_or_multiple_merge_heads(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, merge_head: str
) -> None:
    _merge_lock_repo(tmp_path)
    if merge_head == "multiple_commits":
        merge_head = (tmp_path / ".git/MERGE_HEAD").read_text() * 2
    (tmp_path / ".git/MERGE_HEAD").write_text(merge_head)
    monkeypatch.chdir(tmp_path)
    assert check_lock_unmodified() is not None


@pytest.mark.unit
def test_clean_working_lock_cannot_hide_a_different_staged_lock(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _lock_repo(tmp_path)
    (tmp_path / "uv.lock").write_bytes(b"version = 99\n")
    _run_git(["add", "uv.lock"], cwd=tmp_path)
    (tmp_path / "uv.lock").write_bytes(b"version = 1\n")
    monkeypatch.chdir(tmp_path)
    assert check_lock_unmodified() is not None


@pytest.mark.unit
def test_dirty_lock_guidance_scopes_frozen_commands_and_unsets_lock_check(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """A dirty lock tells users which commands may use frozen mode."""
    _run_git(["init", "-b", "main"], cwd=tmp_path)
    _run_git(["config", "user.email", "test@example.invalid"], cwd=tmp_path)
    _run_git(["config", "user.name", "Test User"], cwd=tmp_path)
    lock_path = tmp_path / "uv.lock"
    lock_path.write_text("version = 1\n")
    _run_git(["add", "uv.lock"], cwd=tmp_path)
    _run_git(["commit", "-m", "initial lock"], cwd=tmp_path)
    lock_path.write_text("version = 2\n")
    monkeypatch.chdir(tmp_path)

    message = check_lock_unmodified()

    assert message is not None
    assert (
        "use `UV_FROZEN=1` only for `uv sync` or `uv run`; "
        "`uv lock --check` must run with `UV_FROZEN` unset"
    ) in message
