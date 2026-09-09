"""Contracts for the bounded, read-only developer doctor."""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

import pytest
from tools.devtools import doctor

pytestmark = pytest.mark.unit


def _write_executable(path: Path, body: str) -> Path:
    path.write_text("#!/usr/bin/env python3\n" + body, encoding="utf-8")
    path.chmod(0o755)
    return path


def _fake_git(path: Path, repo: Path, *, detached: bool = False) -> Path:
    return _write_executable(
        path,
        f"""\
import sys

args = sys.argv[1:]
command = args[2:]
if command == ["rev-parse", "--show-toplevel"]:
    print({str(repo)!r})
elif command == ["rev-parse", "HEAD"]:
    print("0123456789abcdef0123456789abcdef01234567")
elif command == ["symbolic-ref", "--quiet", "--short", "HEAD"]:
    if {detached!r}:
        raise SystemExit(1)
    print("codex/PER-304-dev-tooling-observability")
elif command == ["status", "--porcelain=v1", "-z"]:
    sys.stdout.write(" M tracked.txt\\0?? new.txt\\0R  renamed.txt\\0old.txt\\0")
elif command == ["rev-list", "--left-right", "--count", "HEAD...origin/dev"]:
    print("3  5")
else:
    print(f"unexpected git argv: {{args!r}}", file=sys.stderr)
    raise SystemExit(91)
""",
    )


def _fake_mise(path: Path, repo: Path) -> Path:
    tasks = [
        {
            "name": "check",
            "source": str(repo / ".mise.toml"),
            "config_sources": [str(repo / ".mise.toml"), str(repo / "mise" / "checks.toml")],
        },
        {
            "name": "sim:report",
            "source": str(repo / "mise" / "dev.toml"),
            "config_sources": [str(repo / ".mise.toml"), str(repo / "mise" / "dev.toml")],
        },
    ]
    return _write_executable(
        path,
        f"""\
import json
import sys

if sys.argv[1:] == ["--version"]:
    print("2026.8.14 linux-x64 (test build)")
elif sys.argv[1:] == ["tasks", "--json"]:
    print(json.dumps({tasks!r}))
else:
    raise SystemExit(92)
""",
    )


def _write_host_policy(repo: Path) -> None:
    host = repo / ".codex" / "host"
    (host / "systemd").mkdir(parents=True)
    (host / "policy.sh").write_text(
        """\
CODEX_RUST_HOST_POLICY_VERSION=11
CODEX_RUST_SCCACHE_POLICY_KEY=0.17.0-p2
CODEX_RUST_SCCACHE_VERSION=0.17.0
CODEX_RUST_MAX_JOBS=4
CODEX_RUST_PARENT_SLICE=codex-rust.slice
CODEX_RUST_BABYLON_SLICE=codex-rust-babylon.slice
CODEX_RUST_BABYLON_TARGET_SUBDIR=rust/target
ignored_shell_function() { echo must-not-run; }
""",
        encoding="utf-8",
    )
    (host / "systemd" / "codex-rust.slice").write_text(
        """\
[Slice]
CPUQuota=800%
MemoryHigh=20G
MemoryMax=24G
MemorySwapMax=4G
IOWeight=500
TasksMax=1024
""",
        encoding="utf-8",
    )
    (host / "systemd" / "codex-rust-babylon.slice").write_text(
        """\
[Slice]
CPUQuota=400%
MemoryHigh=10G
MemoryMax=12G
MemorySwapMax=2G
IOWeight=500
TasksMax=512
""",
        encoding="utf-8",
    )


def test_collect_report_covers_git_mise_and_static_host_policy(tmp_path: Path) -> None:
    repo = tmp_path / "linked-worktree"
    repo.mkdir()
    _write_host_policy(repo)
    git = _fake_git(tmp_path / "git", repo)
    mise = _fake_mise(tmp_path / "mise", repo)

    report = doctor.collect_report(
        repo_hint=repo / "nested",
        git_executable=str(git),
        mise_executable=str(mise),
    )

    assert report == {
        "git": {
            "ahead_of_origin_dev": 3,
            "behind_origin_dev": 5,
            "branch": "codex/PER-304-dev-tooling-observability",
            "detached": False,
            "dirty_count": 3,
            "head": "0123456789abcdef0123456789abcdef01234567",
        },
        "mise": {
            "config_sources": [".mise.toml", "mise/checks.toml", "mise/dev.toml"],
            "task_count": 2,
            "version": "2026.8.14 linux-x64 (test build)",
        },
        "repo_root": str(repo),
        "rust_host_policy": {
            "available": True,
            "max_jobs": 4,
            "policy_version": 11,
            "repository_slice": {
                "limits": {
                    "CPUQuota": "400%",
                    "IOWeight": "500",
                    "MemoryHigh": "10G",
                    "MemoryMax": "12G",
                    "MemorySwapMax": "2G",
                    "TasksMax": "512",
                },
                "name": "codex-rust-babylon.slice",
            },
            "parent_slice": {
                "limits": {
                    "CPUQuota": "800%",
                    "IOWeight": "500",
                    "MemoryHigh": "20G",
                    "MemoryMax": "24G",
                    "MemorySwapMax": "4G",
                    "TasksMax": "1024",
                },
                "name": "codex-rust.slice",
            },
            "sccache_policy_key": "0.17.0-p2",
            "sccache_version": "0.17.0",
            "target_subdir": "rust/target",
        },
        "schema": "babylon.devtools.doctor.v1",
    }


def test_detached_worktree_is_reported_without_guessing_a_branch(tmp_path: Path) -> None:
    repo = tmp_path / "detached-worktree"
    repo.mkdir()
    _write_host_policy(repo)

    report = doctor.collect_report(
        repo_hint=repo,
        git_executable=str(_fake_git(tmp_path / "git", repo, detached=True)),
        mise_executable=str(_fake_mise(tmp_path / "mise", repo)),
    )

    assert report["git"]["detached"] is True
    assert report["git"]["branch"] is None


def test_renderers_are_bounded_and_do_not_emit_environment_values(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    repo = tmp_path / "repo"
    repo.mkdir()
    _write_host_policy(repo)
    monkeypatch.setenv("DATABASE_URL", "postgresql://do-not-print.invalid/secret")
    report = doctor.collect_report(
        repo_hint=repo,
        git_executable=str(_fake_git(tmp_path / "git", repo)),
        mise_executable=str(_fake_mise(tmp_path / "mise", repo)),
    )

    text = doctor.render_text(report)
    encoded = doctor.render_json(report)

    assert "branch codex/PER-304-dev-tooling-observability" in text
    assert "ahead 3, behind 5" in text
    assert "2 tasks" in text
    assert "rust policy (repository): v11" in text
    assert "do-not-print" not in text
    assert "do-not-print" not in encoded
    assert len(text.encode()) <= doctor.MAX_REPORT_BYTES
    assert len(encoded.encode()) <= doctor.MAX_REPORT_BYTES
    assert json.loads(encoded) == report


def test_missing_host_policy_is_an_explicit_available_false_fact(tmp_path: Path) -> None:
    repo = tmp_path / "repo"
    repo.mkdir()

    report = doctor.collect_report(
        repo_hint=repo,
        git_executable=str(_fake_git(tmp_path / "git", repo)),
        mise_executable=str(_fake_mise(tmp_path / "mise", repo)),
    )

    assert report["rust_host_policy"] == {
        "available": False,
        "reason": "missing .codex/host/policy.sh",
    }


def test_mise_failure_has_a_specific_secret_safe_error(tmp_path: Path) -> None:
    repo = tmp_path / "repo"
    repo.mkdir()
    _write_host_policy(repo)
    mise = _write_executable(
        tmp_path / "mise",
        """\
import sys
print("postgresql://must-not-escape.invalid/secret", file=sys.stderr)
raise SystemExit(23)
""",
    )

    with pytest.raises(doctor.DoctorError) as caught:
        doctor.collect_report(
            repo_hint=repo,
            git_executable=str(_fake_git(tmp_path / "git", repo)),
            mise_executable=str(mise),
        )

    assert str(caught.value) == "mise --version exited with status 23"
    assert "must-not-escape" not in str(caught.value)


def test_run_command_kills_child_as_soon_as_output_exceeds_bound(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    pid_file = tmp_path / "child.pid"
    completed_marker = tmp_path / "child-completed"
    command = tmp_path / "excess_output.py"
    command.write_text(
        """\
import os
import pathlib
import sys
import time

pathlib.Path(sys.argv[1]).write_text(str(os.getpid()), encoding="utf-8")
sys.stdout.buffer.write(b"x" * 16_384)
sys.stdout.buffer.flush()
time.sleep(2)
pathlib.Path(sys.argv[2]).write_text("not killed", encoding="utf-8")
""",
        encoding="utf-8",
    )
    monkeypatch.setattr(doctor, "MAX_COMMAND_BYTES", 4096)

    with pytest.raises(doctor.DoctorError) as caught:
        doctor._run_command(
            [sys.executable, str(command), str(pid_file), str(completed_marker)],
            label="oversized child",
        )

    assert str(caught.value) == "oversized child output exceeds the 4096-byte bound"
    child_pid = int(pid_file.read_text(encoding="utf-8"))
    with pytest.raises(ProcessLookupError):
        os.kill(child_pid, 0)
    assert not completed_marker.exists()
