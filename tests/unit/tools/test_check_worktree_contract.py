"""Commit prerequisites preserve the exact staged dependency pair."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path
from typing import Any

import pytest

TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

import check_worktree_contract as contract  # type: ignore[import-not-found]  # noqa: E402


def _run_git(arguments: list[str], *, cwd: Path) -> None:
    subprocess.run(["git", *arguments], cwd=cwd, check=True, capture_output=True)


def _lock_repo(path: Path) -> None:
    _run_git(["init", "-b", "main"], cwd=path)
    _run_git(["config", "user.email", "test@example.invalid"], cwd=path)
    _run_git(["config", "user.name", "Test User"], cwd=path)
    (path / "pyproject.toml").write_text('[project]\nname = "example"\nversion = "1"\n')
    (path / "uv.lock").write_text("version = 1\n")
    _run_git(["add", "pyproject.toml", "uv.lock"], cwd=path)
    _run_git(["commit", "-m", "initial dependencies"], cwd=path)


def _mock_uv(
    monkeypatch: pytest.MonkeyPatch,
    *,
    failure: int | Exception = 0,
) -> list[dict[str, Any]]:
    original = subprocess.run
    calls: list[dict[str, Any]] = []

    def run(command: list[str], **kwargs: Any) -> subprocess.CompletedProcess[Any]:
        if command != ["uv", "lock", "--check"]:
            return original(command, **kwargs)
        calls.append(kwargs)
        if isinstance(failure, Exception):
            raise failure
        return subprocess.CompletedProcess(command, failure, "", "stale lock" if failure else "")

    monkeypatch.setattr(subprocess, "run", run)
    return calls


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
    assert (contract.check_interpreter() is None) == (version == "3.12.14")


def test_failed_interpreter_command_cannot_pass_with_matching_output(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    (tmp_path / ".python-version").write_text("3.12.14\n")
    binary = tmp_path / ".venv/bin/python"
    binary.parent.mkdir(parents=True)
    binary.touch()
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr(
        subprocess,
        "run",
        lambda *_args, **_kwargs: subprocess.CompletedProcess([], 1, "Python 3.12.14\n", ""),
    )
    assert contract.check_interpreter() is not None


@pytest.mark.parametrize("staged_edit", [False, True])
def test_lock_accepts_matching_staged_dependencies_and_checks_without_frozen_mode(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, staged_edit: bool
) -> None:
    _lock_repo(tmp_path)
    if staged_edit:
        for name in ("pyproject.toml", "uv.lock"):
            with (tmp_path / name).open("a") as output:
                output.write("# intentional dependency retirement\n")
        _run_git(["add", "pyproject.toml", "uv.lock"], cwd=tmp_path)
    monkeypatch.chdir(tmp_path)
    monkeypatch.setenv("UV_FROZEN", "1")
    monkeypatch.setenv("UV_OFFLINE", "1")
    calls = _mock_uv(monkeypatch)
    assert contract.check_lock_consistency() is None
    assert len(calls) == 1
    assert "UV_FROZEN" not in calls[0]["env"]
    assert calls[0]["env"]["UV_OFFLINE"] == "1"
    assert calls[0]["check"] is False


@pytest.mark.parametrize("name", ["pyproject.toml", "uv.lock"])
@pytest.mark.parametrize("changed_copy", ["index", "working"])
def test_partial_staging_refuses_before_uv_can_validate_another_manifest(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, name: str, changed_copy: str
) -> None:
    _lock_repo(tmp_path)
    path = tmp_path / name
    original = path.read_bytes()
    path.write_bytes(original + b"# changed\n")
    if changed_copy == "index":
        _run_git(["add", name], cwd=tmp_path)
        path.write_bytes(original)
    monkeypatch.chdir(tmp_path)
    calls = _mock_uv(monkeypatch)
    message = contract.check_lock_consistency()
    assert message is not None and name in message
    assert not calls


@pytest.mark.parametrize("name", ["pyproject.toml", "uv.lock"])
@pytest.mark.parametrize("missing_copy", ["index", "working"])
def test_missing_dependency_file_refuses(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, name: str, missing_copy: str
) -> None:
    _lock_repo(tmp_path)
    if missing_copy == "index":
        _run_git(["rm", "--cached", name], cwd=tmp_path)
    else:
        (tmp_path / name).unlink()
    monkeypatch.chdir(tmp_path)
    calls = _mock_uv(monkeypatch)
    assert contract.check_lock_consistency() is not None
    assert not calls


def test_unmerged_index_refuses_even_when_dependency_files_are_resolved(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _lock_repo(tmp_path)
    conflict = tmp_path / "other.txt"
    conflict.write_text("base\n")
    _run_git(["add", "other.txt"], cwd=tmp_path)
    _run_git(["commit", "-m", "base"], cwd=tmp_path)
    _run_git(["checkout", "-b", "incoming"], cwd=tmp_path)
    conflict.write_text("incoming\n")
    _run_git(["commit", "-am", "incoming"], cwd=tmp_path)
    _run_git(["checkout", "main"], cwd=tmp_path)
    conflict.write_text("main\n")
    _run_git(["commit", "-am", "main"], cwd=tmp_path)
    merge = subprocess.run(
        ["git", "merge", "incoming"], cwd=tmp_path, check=False, capture_output=True
    )
    assert merge.returncode == 1
    monkeypatch.chdir(tmp_path)
    calls = _mock_uv(monkeypatch)
    assert contract.check_lock_consistency() is not None
    assert not calls
    _run_git(["add", "other.txt"], cwd=tmp_path)
    assert contract.check_lock_consistency() is None


@pytest.mark.parametrize(
    "failure", [1, FileNotFoundError("uv absent"), subprocess.TimeoutExpired("uv", 60)]
)
def test_invalid_lock_and_uv_command_errors_refuse(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, failure: int | Exception
) -> None:
    _lock_repo(tmp_path)
    monkeypatch.chdir(tmp_path)
    _mock_uv(monkeypatch, failure=failure)
    message = contract.check_lock_consistency()
    assert message is not None and "UV_FROZEN unset" in message
    if failure == 1:
        assert "stale lock" in message


def test_unavailable_git_evidence_refuses(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(contract, "_git_bytes", lambda _arguments: None)
    assert contract.check_lock_consistency() is not None


def test_generic_gate_needs_no_private_data_dotenv_or_operator_extra(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr(contract, "check_interpreter", lambda: None)
    monkeypatch.setattr(contract, "check_lock_consistency", lambda: None)
    assert contract.main() == 0
    assert not (tmp_path / "data").exists()
    assert not (tmp_path / ".env").exists()
