"""Python lockfile contracts for a clean Babylon archive."""

from __future__ import annotations

import io
import os
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

import pytest

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
_DIAGNOSTIC_OUTPUT_LIMIT = 1_000


def _archive_member(member: tarfile.TarInfo, destination: str) -> tarfile.TarInfo | None:
    """Exclude the repository's external data symlink from a portable archive."""
    if member.issym() and Path(member.linkname).is_absolute():
        return None
    return tarfile.data_filter(member, destination)


def _extract_archive(ref: str, destination: Path) -> None:
    """Extract one Git revision without copying untracked sibling directories."""
    archive = subprocess.run(  # noqa: S603
        ["git", "archive", "--format=tar", ref],  # noqa: S607
        cwd=REPOSITORY_ROOT,
        capture_output=True,
        check=True,
    )
    with tarfile.open(fileobj=io.BytesIO(archive.stdout)) as tar:
        tar.extractall(destination, filter=_archive_member)  # noqa: S202 -- filtered archive


def _subprocess_diagnostics(result: subprocess.CompletedProcess[str]) -> str:
    """Describe a subprocess result with bounded stdout and stderr."""
    stdout = result.stdout or ""
    stderr = result.stderr or ""
    if len(stdout) > _DIAGNOSTIC_OUTPUT_LIMIT:
        stdout = f"{stdout[:_DIAGNOSTIC_OUTPUT_LIMIT]}\n[truncated]"
    if len(stderr) > _DIAGNOSTIC_OUTPUT_LIMIT:
        stderr = f"{stderr[:_DIAGNOSTIC_OUTPUT_LIMIT]}\n[truncated]"
    return f"stdout:\n{stdout}\nstderr:\n{stderr}"


def _pinned_uv() -> str:
    """Return the repository-managed uv binary before entering the archive."""
    try:
        result = subprocess.run(  # noqa: S603
            ["mise", "which", "uv"],  # noqa: S607
            cwd=REPOSITORY_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
    except FileNotFoundError:
        pytest.fail(
            "repository-managed uv requires `mise`; install mise or expose it on PATH before "
            "running the lock contract"
        )
    if result.returncode != 0:
        raise AssertionError(
            f"mise which uv failed (exit {result.returncode})\n{_subprocess_diagnostics(result)}"
        )
    uv_path = result.stdout.strip()
    if not uv_path:
        raise AssertionError(
            f"mise which uv returned no executable path\n{_subprocess_diagnostics(result)}"
        )
    return uv_path


def test_pinned_uv_fails_actionably_when_mise_is_missing(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """The required pinned-toolchain lookup must not degrade into a skip."""

    def missing_mise(*_args: object, **_kwargs: object) -> None:
        raise FileNotFoundError("mise is absent")

    monkeypatch.setattr(subprocess, "run", missing_mise)

    with pytest.raises(pytest.fail.Exception, match="repository-managed uv"):
        _pinned_uv()


def test_pinned_uv_failure_shows_bounded_combined_diagnostics(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """A failed lookup remains bounded under pytest assertion rewriting."""
    result = subprocess.CompletedProcess(
        ["mise", "which", "uv"],
        returncode=17,
        stdout="mise stdout: " + "x" * 4096,
        stderr="mise stderr: " + "y" * 4096,
    )
    monkeypatch.setattr(subprocess, "run", lambda *_args, **_kwargs: result)

    with pytest.raises(AssertionError) as failure:
        _pinned_uv()

    message = str(failure.value)
    assert "mise which uv failed (exit 17)" in message
    assert "stdout:" in message and "mise stdout:" in message
    assert "stderr:" in message and "mise stderr:" in message
    assert "[truncated]" in message
    assert len(message) < 3_000


def test_pinned_uv_rejects_empty_mise_output(monkeypatch: pytest.MonkeyPatch) -> None:
    """A successful lookup still needs to return an executable path."""
    result = subprocess.CompletedProcess(
        ["mise", "which", "uv"],
        returncode=0,
        stdout=" \n",
        stderr="mise stderr",
    )
    monkeypatch.setattr(subprocess, "run", lambda *_args, **_kwargs: result)

    with pytest.raises(AssertionError, match="returned no executable path"):
        _pinned_uv()


def _run_archive_lock_check(ref: str) -> subprocess.CompletedProcess[str]:
    """Run the pinned lock check against an isolated archive of ``ref``."""
    with tempfile.TemporaryDirectory() as temporary_directory:
        archive_root = Path(temporary_directory) / "babylon"
        archive_root.mkdir()
        _extract_archive(ref, archive_root)
        assert not (archive_root.parent / "hypergraph-rs").exists()
        lock_environment = os.environ.copy()
        lock_environment.pop("UV_FROZEN", None)
        result = subprocess.run(  # noqa: S603
            [_pinned_uv(), "lock", "--check", "--python", sys.executable],  # noqa: S607
            cwd=archive_root,
            capture_output=True,
            text=True,
            check=False,
            env=lock_environment,
        )
    return result


def test_pinned_uv_validates_an_isolated_archive_without_a_hypergraph_sibling() -> None:
    """A clean archive resolves its committed lock without a local sibling checkout."""
    ref = os.environ.get("BABYLON_LOCK_CONTRACT_REF", "HEAD")
    result = _run_archive_lock_check(ref)

    if result.returncode != 0:
        raise AssertionError(
            f"uv lock --check failed (exit {result.returncode})\n{_subprocess_diagnostics(result)}"
        )


def test_archive_lock_check_unsets_uv_frozen(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """The archive check must not inherit the caller's incompatible lock mode."""
    captured_environment = tmp_path / "uv-frozen.txt"
    probe_uv = tmp_path / "uv-probe"
    probe_uv.write_text('#!/bin/sh\nprintf \'%s\' "${UV_FROZEN-absent}" > "$UV_FROZEN_CAPTURE"\n')
    probe_uv.chmod(0o755)
    monkeypatch.setenv("UV_FROZEN", "1")
    monkeypatch.setenv("UV_FROZEN_CAPTURE", str(captured_environment))
    monkeypatch.setattr(__name__ + "._pinned_uv", lambda: str(probe_uv))

    result = _run_archive_lock_check("HEAD")

    if result.returncode != 0:
        raise AssertionError(
            f"uv lock --check failed (exit {result.returncode})\n{_subprocess_diagnostics(result)}"
        )
    assert captured_environment.read_text() == "absent"


def test_archive_lock_check_uses_the_executing_python(monkeypatch: pytest.MonkeyPatch) -> None:
    """The forward-compatibility lane must not require the production interpreter."""
    commands: list[list[str]] = []

    def capture(command: list[str], **_kwargs: object) -> subprocess.CompletedProcess[str]:
        commands.append(command)
        return subprocess.CompletedProcess(command, 0, "", "")

    monkeypatch.setattr(__name__ + "._extract_archive", lambda *_args: None)
    monkeypatch.setattr(__name__ + "._pinned_uv", lambda: "/pinned/uv")
    monkeypatch.setattr(subprocess, "run", capture)
    _run_archive_lock_check("HEAD")
    assert commands == [["/pinned/uv", "lock", "--check", "--python", sys.executable]]
