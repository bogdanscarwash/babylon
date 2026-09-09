#!/usr/bin/env python3
"""Emit a bounded, read-only snapshot of Babylon developer-tooling state."""

from __future__ import annotations

import argparse
import json
import os
import re
import selectors
import subprocess
import sys
import time
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import IO, Final, cast

REPORT_SCHEMA: Final = "babylon.devtools.doctor.v1"
COMMAND_TIMEOUT_SECONDS: Final = 15
MAX_COMMAND_BYTES: Final = 8 * 1024 * 1024
COMMAND_READ_CHUNK_BYTES: Final = 64 * 1024
COMMAND_REAP_TIMEOUT_SECONDS: Final = 5
MAX_POLICY_BYTES: Final = 256 * 1024
MAX_REPORT_BYTES: Final = 64 * 1024
MAX_TASKS: Final = 10_000

_POLICY_KEYS: Final = (
    "CODEX_RUST_HOST_POLICY_VERSION",
    "CODEX_RUST_SCCACHE_POLICY_KEY",
    "CODEX_RUST_SCCACHE_VERSION",
    "CODEX_RUST_MAX_JOBS",
    "CODEX_RUST_PARENT_SLICE",
    "CODEX_RUST_BABYLON_SLICE",
    "CODEX_RUST_BABYLON_TARGET_SUBDIR",
)
_SLICE_LIMIT_KEYS: Final = (
    "CPUQuota",
    "IOWeight",
    "MemoryHigh",
    "MemoryMax",
    "MemorySwapMax",
    "TasksMax",
)
_SHA_PATTERN: Final = re.compile(r"[0-9a-f]{40,64}")
_POLICY_ASSIGNMENT: Final = re.compile(r"^([A-Z][A-Z0-9_]*)=([A-Za-z0-9._/+%-]+)$")
_SLICE_NAME: Final = re.compile(r"[a-z0-9.-]+\.slice")


class DoctorError(RuntimeError):
    """Raised when a bounded diagnostic fact cannot be collected safely."""


@dataclass(frozen=True)
class CommandOutput:
    """Bounded bytes returned by one explicit read-only command."""

    stdout: bytes
    stderr: bytes
    returncode: int


def _close_pipe(pipe: IO[bytes] | None) -> None:
    if pipe is None:
        return
    try:
        pipe.close()
    except OSError:
        pass


def _kill_and_reap(process: subprocess.Popen[bytes]) -> None:
    if process.poll() is None:
        try:
            process.kill()
        except OSError:
            pass
    try:
        process.wait(timeout=COMMAND_REAP_TIMEOUT_SECONDS)
    except (OSError, subprocess.TimeoutExpired):
        pass


def _read_command_output(
    process: subprocess.Popen[bytes],
    *,
    label: str,
) -> CommandOutput:
    stdout = process.stdout
    stderr = process.stderr
    if stdout is None or stderr is None:
        _kill_and_reap(process)
        raise DoctorError(f"{label} could not capture output")

    selector = selectors.DefaultSelector()
    buffers = {"stdout": bytearray(), "stderr": bytearray()}
    deadline = time.monotonic() + COMMAND_TIMEOUT_SECONDS
    failure: DoctorError | None = None
    returncode: int | None = None
    try:
        for name, pipe in (("stdout", stdout), ("stderr", stderr)):
            os.set_blocking(pipe.fileno(), False)
            selector.register(pipe, selectors.EVENT_READ, data=name)

        while selector.get_map():
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                failure = DoctorError(f"{label} timed out after {COMMAND_TIMEOUT_SECONDS} seconds")
                break
            events = selector.select(timeout=remaining)
            if not events:
                continue
            for key, _ in events:
                name = cast("str", key.data)
                pipe = cast("IO[bytes]", key.fileobj)
                capacity = MAX_COMMAND_BYTES - len(buffers[name])
                read_size = min(COMMAND_READ_CHUNK_BYTES, capacity + 1)
                try:
                    chunk = os.read(pipe.fileno(), read_size)
                except BlockingIOError:
                    continue
                if not chunk:
                    selector.unregister(pipe)
                    _close_pipe(pipe)
                    continue
                if len(chunk) > capacity:
                    failure = DoctorError(
                        f"{label} output exceeds the {MAX_COMMAND_BYTES}-byte bound"
                    )
                    break
                buffers[name].extend(chunk)
            if failure is not None:
                break

        if failure is None:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                failure = DoctorError(f"{label} timed out after {COMMAND_TIMEOUT_SECONDS} seconds")
            else:
                try:
                    returncode = process.wait(timeout=remaining)
                except subprocess.TimeoutExpired:
                    failure = DoctorError(
                        f"{label} timed out after {COMMAND_TIMEOUT_SECONDS} seconds"
                    )
    except OSError as error:
        failure = DoctorError(
            f"{label} output could not be read: {error.strerror or type(error).__name__}"
        )
    finally:
        selector.close()
        if failure is not None:
            _kill_and_reap(process)
        _close_pipe(stdout)
        _close_pipe(stderr)

    if failure is not None:
        raise failure
    if returncode is None:
        raise DoctorError(f"{label} did not report an exit status")
    return CommandOutput(bytes(buffers["stdout"]), bytes(buffers["stderr"]), returncode)


def _run_command(
    argv: Sequence[str],
    *,
    label: str,
    cwd: Path | None = None,
    allowed_statuses: Sequence[int] = (0,),
) -> CommandOutput:
    if not argv:
        raise DoctorError(f"{label} has no executable")
    try:
        process = subprocess.Popen(
            list(argv),
            cwd=cwd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except FileNotFoundError as error:
        raise DoctorError(f"{label} executable not found") from error
    except OSError as error:
        raise DoctorError(
            f"{label} could not start: {error.strerror or type(error).__name__}"
        ) from error

    completed = _read_command_output(process, label=label)
    if completed.returncode not in allowed_statuses:
        raise DoctorError(f"{label} exited with status {completed.returncode}")
    return completed


def _decode_output(output: bytes, *, label: str) -> str:
    try:
        return output.decode("utf-8")
    except UnicodeDecodeError as error:
        raise DoctorError(f"{label} emitted non-UTF-8 output") from error


def _single_line(output: bytes, *, label: str) -> str:
    text = _decode_output(output, label=label).strip()
    if not text:
        raise DoctorError(f"{label} emitted no value")
    if "\n" in text or "\r" in text:
        raise DoctorError(f"{label} emitted more than one line")
    if len(text.encode()) > 4096:
        raise DoctorError(f"{label} value exceeds the 4096-byte bound")
    return text


def _git(
    git_executable: str,
    repo: Path,
    arguments: Sequence[str],
    *,
    label: str,
    allowed_statuses: Sequence[int] = (0,),
) -> CommandOutput:
    return _run_command(
        [git_executable, "-C", str(repo), *arguments],
        label=label,
        allowed_statuses=allowed_statuses,
    )


def _repo_root(repo_hint: Path, git_executable: str) -> Path:
    output = _git(
        git_executable,
        repo_hint,
        ["rev-parse", "--show-toplevel"],
        label="git repository root",
    )
    root_text = _single_line(output.stdout, label="git repository root")
    root = Path(root_text)
    if not root.is_absolute():
        raise DoctorError("git repository root is not absolute")
    return root


def _dirty_count(output: bytes) -> int:
    if not output:
        return 0
    records = output.split(b"\0")
    if records[-1] != b"":
        raise DoctorError("git status did not terminate its porcelain record")
    records.pop()
    count = 0
    index = 0
    while index < len(records):
        record = records[index]
        if len(record) < 4 or record[2:3] != b" ":
            raise DoctorError("git status emitted an invalid porcelain record")
        status = record[:2]
        count += 1
        index += 1
        if status[0:1] in {b"R", b"C"} or status[1:2] in {b"R", b"C"}:
            if index >= len(records):
                raise DoctorError("git status omitted a rename source path")
            index += 1
    return count


def _collect_git(repo: Path, git_executable: str) -> dict[str, object]:
    head = _single_line(
        _git(git_executable, repo, ["rev-parse", "HEAD"], label="git HEAD").stdout,
        label="git HEAD",
    )
    if _SHA_PATTERN.fullmatch(head) is None:
        raise DoctorError("git HEAD is not a full hexadecimal object ID")

    branch_output = _git(
        git_executable,
        repo,
        ["symbolic-ref", "--quiet", "--short", "HEAD"],
        label="git branch",
        allowed_statuses=(0, 1),
    )
    if branch_output.returncode == 0:
        branch: str | None = _single_line(branch_output.stdout, label="git branch")
        detached = False
    else:
        branch = None
        detached = True

    status = _git(
        git_executable,
        repo,
        ["status", "--porcelain=v1", "-z"],
        label="git status",
    )
    comparison_text = _single_line(
        _git(
            git_executable,
            repo,
            ["rev-list", "--left-right", "--count", "HEAD...origin/dev"],
            label="git origin/dev comparison",
        ).stdout,
        label="git origin/dev comparison",
    )
    comparison = comparison_text.split()
    if len(comparison) != 2 or not all(value.isdecimal() for value in comparison):
        raise DoctorError("git origin/dev comparison emitted an invalid count pair")

    return {
        "head": head,
        "branch": branch,
        "detached": detached,
        "dirty_count": _dirty_count(status.stdout),
        "ahead_of_origin_dev": int(comparison[0]),
        "behind_origin_dev": int(comparison[1]),
    }


def _normalize_config_source(value: str, repo: Path) -> str:
    if len(value.encode()) > 4096:
        raise DoctorError("mise task config source exceeds the 4096-byte bound")
    source = Path(value)
    if source.is_absolute():
        try:
            return str(source.relative_to(repo))
        except ValueError:
            return str(source)
    return str(source)


def _collect_mise(repo: Path, mise_executable: str) -> dict[str, object]:
    version = _single_line(
        _run_command(
            [mise_executable, "--version"],
            cwd=repo,
            label="mise --version",
        ).stdout,
        label="mise --version",
    )
    tasks_output = _run_command(
        [mise_executable, "tasks", "--json"],
        cwd=repo,
        label="mise tasks --json",
    )
    try:
        parsed = json.loads(_decode_output(tasks_output.stdout, label="mise tasks --json"))
    except json.JSONDecodeError as error:
        raise DoctorError(f"mise tasks --json emitted invalid JSON: {error.msg}") from error
    if not isinstance(parsed, list):
        raise DoctorError("mise tasks --json did not emit a task list")
    if len(parsed) > MAX_TASKS:
        raise DoctorError(f"mise tasks --json exceeds the {MAX_TASKS}-task bound")

    sources: set[str] = set()
    for task in parsed:
        if not isinstance(task, dict):
            raise DoctorError("mise tasks --json contains a non-object task")
        source = task.get("source")
        if source is not None:
            if not isinstance(source, str):
                raise DoctorError("mise task source is not text")
            sources.add(_normalize_config_source(source, repo))
        config_sources = task.get("config_sources", [])
        if not isinstance(config_sources, list) or not all(
            isinstance(item, str) for item in config_sources
        ):
            raise DoctorError("mise task config_sources is not a text list")
        for config_source in config_sources:
            sources.add(_normalize_config_source(config_source, repo))

    return {
        "version": version,
        "task_count": len(parsed),
        "config_sources": sorted(sources),
    }


def _read_bounded(path: Path, *, label: str) -> str:
    try:
        size = path.stat().st_size
        if size > MAX_POLICY_BYTES:
            raise DoctorError(f"{label} exceeds the {MAX_POLICY_BYTES}-byte bound")
        return path.read_text(encoding="utf-8")
    except DoctorError:
        raise
    except UnicodeDecodeError as error:
        raise DoctorError(f"{label} is not UTF-8") from error
    except OSError as error:
        raise DoctorError(f"cannot read {label}") from error


def _policy_assignments(path: Path) -> dict[str, str]:
    assignments: dict[str, str] = {}
    for line in _read_bounded(path, label=".codex/host/policy.sh").splitlines():
        match = _POLICY_ASSIGNMENT.fullmatch(line.strip())
        if match is not None and match.group(1) in _POLICY_KEYS:
            assignments[match.group(1)] = match.group(2)
    missing = [key for key in _POLICY_KEYS if key not in assignments]
    if missing:
        raise DoctorError(f".codex/host/policy.sh is missing {missing[0]}")
    return assignments


def _positive_policy_integer(assignments: Mapping[str, str], key: str) -> int:
    value = assignments[key]
    if not value.isdecimal() or int(value) < 1:
        raise DoctorError(f".codex/host/policy.sh has invalid {key}")
    return int(value)


def _slice_limits(host_dir: Path, name: str) -> dict[str, str]:
    if _SLICE_NAME.fullmatch(name) is None:
        raise DoctorError(f".codex/host/policy.sh has invalid slice name {name!r}")
    path = host_dir / "systemd" / name
    limits: dict[str, str] = {}
    for line in _read_bounded(path, label=f".codex/host/systemd/{name}").splitlines():
        key, separator, value = line.strip().partition("=")
        if separator and key in _SLICE_LIMIT_KEYS:
            if not value or len(value.encode()) > 128:
                raise DoctorError(f".codex/host/systemd/{name} has invalid {key}")
            limits[key] = value
    return {key: limits[key] for key in _SLICE_LIMIT_KEYS if key in limits}


def _collect_host_policy(repo: Path) -> dict[str, object]:
    host_dir = repo / ".codex" / "host"
    policy_path = host_dir / "policy.sh"
    if not policy_path.is_file():
        return {
            "available": False,
            "reason": "missing .codex/host/policy.sh",
        }
    assignments = _policy_assignments(policy_path)
    parent_slice = assignments["CODEX_RUST_PARENT_SLICE"]
    repository_slice = assignments["CODEX_RUST_BABYLON_SLICE"]
    return {
        "available": True,
        "policy_version": _positive_policy_integer(assignments, "CODEX_RUST_HOST_POLICY_VERSION"),
        "max_jobs": _positive_policy_integer(assignments, "CODEX_RUST_MAX_JOBS"),
        "sccache_policy_key": assignments["CODEX_RUST_SCCACHE_POLICY_KEY"],
        "sccache_version": assignments["CODEX_RUST_SCCACHE_VERSION"],
        "target_subdir": assignments["CODEX_RUST_BABYLON_TARGET_SUBDIR"],
        "parent_slice": {
            "name": parent_slice,
            "limits": _slice_limits(host_dir, parent_slice),
        },
        "repository_slice": {
            "name": repository_slice,
            "limits": _slice_limits(host_dir, repository_slice),
        },
    }


def collect_report(
    *,
    repo_hint: Path,
    git_executable: str = "git",
    mise_executable: str = "mise",
) -> dict[str, object]:
    """Collect read-only facts without inspecting or serializing the environment."""
    repo = _repo_root(repo_hint, git_executable)
    return {
        "schema": REPORT_SCHEMA,
        "repo_root": str(repo),
        "git": _collect_git(repo, git_executable),
        "mise": _collect_mise(repo, mise_executable),
        "rust_host_policy": _collect_host_policy(repo),
    }


def _bounded_report(text: str) -> str:
    if len(text.encode()) > MAX_REPORT_BYTES:
        raise DoctorError(f"doctor report exceeds the {MAX_REPORT_BYTES}-byte bound")
    return text


def render_json(report: Mapping[str, object]) -> str:
    return _bounded_report(json.dumps(report, indent=2, sort_keys=True, ensure_ascii=False) + "\n")


def render_text(report: Mapping[str, object]) -> str:
    git = cast("Mapping[str, object]", report["git"])
    mise = cast("Mapping[str, object]", report["mise"])
    policy = cast("Mapping[str, object]", report["rust_host_policy"])
    checkout = "detached" if git["detached"] else f"branch {git['branch']}"
    sources = cast("Sequence[str]", mise["config_sources"])
    lines = [
        "Babylon developer doctor",
        f"repo: {report['repo_root']}",
        f"git: {checkout} @ {git['head']}",
        f"dirty paths: {git['dirty_count']}",
        (f"origin/dev: ahead {git['ahead_of_origin_dev']}, behind {git['behind_origin_dev']}"),
        f"mise: {mise['version']} ({mise['task_count']} tasks)",
        f"mise config: {', '.join(sources) if sources else '(none reported)'}",
    ]
    if not policy["available"]:
        lines.append(f"rust host: unavailable ({policy['reason']})")
    else:
        repository_slice = cast("Mapping[str, object]", policy["repository_slice"])
        limits = cast("Mapping[str, str]", repository_slice["limits"])
        rendered_limits = ", ".join(f"{key}={value}" for key, value in limits.items())
        lines.extend(
            [
                (
                    f"rust policy (repository): v{policy['policy_version']}, "
                    f"max jobs {policy['max_jobs']}, target {policy['target_subdir']}, "
                    f"sccache {policy['sccache_version']} ({policy['sccache_policy_key']})"
                ),
                f"rust slice: {repository_slice['name']} ({rendered_limits})",
            ]
        )
    return _bounded_report("\n".join(lines) + "\n")


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Report read-only Git, Mise, and Rust host-policy facts."
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path.cwd(),
        help="repository or path inside it (defaults to the current directory)",
    )
    parser.add_argument(
        "--format",
        choices=("text", "json"),
        default="text",
        help="report format",
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    arguments = _parser().parse_args(argv)
    try:
        report = collect_report(repo_hint=arguments.repo_root)
        rendered = render_json(report) if arguments.format == "json" else render_text(report)
    except DoctorError as error:
        print(f"doctor: {error}", file=sys.stderr)
        return 2
    print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
