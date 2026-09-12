#!/usr/bin/env python3
"""Run Rust tests and emit compact, reproducible agent-oriented receipts."""

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import shlex
import shutil
import subprocess
import sys
import time
import xml.etree.ElementTree as ET
from collections.abc import Sequence
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, Final

REPOSITORY_ROOT: Final = Path(__file__).resolve().parents[1]
RUST_ROOT: Final = REPOSITORY_ROOT / "rust"
DEFAULT_REPORT_ROOT: Final = REPOSITORY_ROOT / "reports" / "test-results" / "rust"
DEFAULT_COVERAGE_ROOT: Final = REPOSITORY_ROOT / "reports" / "test-results" / "rust-coverage"
SCHEMA_VERSION: Final = 1
MAX_DIAGNOSTIC_CHARS: Final = 1_200
MAX_LOG_TAIL_LINES: Final = 80
MAX_SLOW_TESTS: Final = 20
MAX_SUMMARY_FAILURES: Final = 20

# Other workspace packages retain every integration target. These three packages
# dominate integration linking; dev keeps their current cross-crate boundaries
# and inexpensive source contracts. Full local/release runs select everything.
DEV_INTEGRATION_TARGETS: Final = {
    "babylon-tick": (
        "tick_goldens",
        "causal_contract_conformance",
        "replay_session",
        "staffed_material_replay",
        "probability_projection_contract",
    ),
    "babylon-persistence": (
        "persistence_semantic_vectors",
        "material_runtime",
        "foundation_content",
        "runtime_foundation_checkpoint_contract",
        "postgres_catalog_contract",
        "reader_role_contract",
        "archive_semantic_contract",
        "archive_worker_contract",
        "michigan_material",
        "statewide_material",
        "spatial_reference_installer_contract",
    ),
    "babylon-client": ("decision_surface_contract", "dynamic_linking_fence"),
}

EXIT_CLASSES: Final = {
    0: "success",
    4: "no_tests_selected",
    5: "rerun_tests_outstanding",
    70: "double_spawn_error",
    92: "required_version_not_met",
    94: "invalid_filterset",
    95: "experimental_feature_not_enabled",
    96: "setup_error",
    100: "test_failed",
    101: "build_failed",
    102: "cargo_metadata_failed",
    103: "archive_creation_failed",
    104: "test_list_failed",
    105: "setup_script_failed",
    106: "incomplete_run",
    110: "write_output_error",
}
SOURCE_LOCATION = re.compile(r"(?P<path>[^\s:]+\.rs:\d+(?::\d+)?)")


def _utc_now() -> str:
    return datetime.now(UTC).isoformat(timespec="seconds").replace("+00:00", "Z")


def _tag_name(element: ET.Element) -> str:
    return element.tag.rsplit("}", maxsplit=1)[-1]


def _element_text(element: ET.Element | None) -> str:
    if element is None:
        return ""
    return "\n".join(part.strip() for part in element.itertext() if part.strip())


def _bounded(text: str, limit: int = MAX_DIAGNOSTIC_CHARS) -> str:
    normalized = text.strip()
    if len(normalized) <= limit:
        return normalized
    return f"{normalized[: limit - 15].rstrip()}\n...[truncated]"


def _identity(case: ET.Element) -> str:
    name = case.attrib.get("name", "<unnamed>")
    classname = case.attrib.get("classname", "")
    if not classname or name == classname or name.startswith(f"{classname}::"):
        return name
    return f"{classname}::{name}"


def _rerun_command(binary_id: str, test_name: str) -> str:
    expression = shlex.quote(f"binary_id(={binary_id}) & test(={test_name})")
    return f"mise run rust:test:q -- -E {expression}"


def _failure_record(case: ET.Element, junit_name: str) -> dict[str, Any] | None:
    children = list(case)
    exceptional = [
        child
        for child in children
        if _tag_name(child)
        in {"failure", "error", "rerunFailure", "rerunError", "flakyFailure", "flakyError"}
    ]
    if not exceptional:
        return None

    tags = {_tag_name(child) for child in exceptional}
    diagnostic_parts = []
    for child in exceptional:
        message = child.attrib.get("message", "").strip()
        body = _element_text(child)
        diagnostic_parts.append("\n".join(part for part in (message, body) if part))
    diagnostic = "\n\n".join(part for part in diagnostic_parts if part)
    # Nextest marks this execution outcome in JUnit's type attribute. Assertion
    # diagnostics may quote timeout configuration without the test timing out.
    if any(child.attrib.get("type") == "test timeout" for child in exceptional):
        kind = "timeout"
    elif tags & {"flakyFailure", "flakyError"}:
        kind = "flaky"
    elif tags & {"error", "rerunError", "flakyError"}:
        kind = "error"
    else:
        kind = "failure"

    first = exceptional[0]
    cause = first.attrib.get("message", "").strip()
    if not cause:
        cause = next((line.strip() for line in diagnostic.splitlines() if line.strip()), kind)
    source_match = SOURCE_LOCATION.search(diagnostic)
    test_name = _identity(case)
    binary_id = case.attrib.get("classname", "")
    stdout = next(
        (_element_text(child) for child in children if _tag_name(child) == "system-out"), ""
    )
    stderr = next(
        (_element_text(child) for child in children if _tag_name(child) == "system-err"), ""
    )

    return {
        "test": test_name,
        "binary_id": binary_id,
        "nextest_name": case.attrib.get("name", test_name),
        "kind": kind,
        "duration_ms": round(float(case.attrib.get("time", "0") or 0) * 1_000, 3),
        "attempts": 1
        + sum(
            _tag_name(child) in {"rerunFailure", "rerunError", "flakyFailure", "flakyError"}
            for child in exceptional
        ),
        "cause": _bounded(cause, 300),
        "source": source_match.group("path") if source_match else None,
        "diagnostic": _bounded(diagnostic),
        "stdout": _bounded(stdout),
        "stderr": _bounded(stderr),
        "rerun": _rerun_command(binary_id, case.attrib.get("name", test_name)),
        "junit": junit_name,
    }


def parse_junit(path: Path) -> dict[str, Any]:
    """Parse stable nextest JUnit into a compact failure-first structure."""
    root = ET.parse(path).getroot()  # noqa: S314 -- repository-generated local XML
    cases = [element for element in root.iter() if _tag_name(element) == "testcase"]
    failures = [
        record for case in cases if (record := _failure_record(case, path.name)) is not None
    ]
    skipped = sum(any(_tag_name(child) == "skipped" for child in case) for case in cases)
    flaky = sum(record["kind"] == "flaky" for record in failures)
    timed_out = sum(record["kind"] == "timeout" for record in failures)
    failed = len(failures) - flaky
    passed = len(cases) - skipped - failed - flaky
    slowest_cases = sorted(
        (
            (
                round(float(case.attrib.get("time", "0") or 0) * 1_000, 3),
                _identity(case),
            )
            for case in cases
        ),
        key=lambda item: (-item[0], item[1]),
    )[:MAX_SLOW_TESTS]
    slowest = [
        {"test": test_name, "duration_ms": duration_ms} for duration_ms, test_name in slowest_cases
    ]
    return {
        "totals": {
            "passed": passed,
            "failed": failed,
            "flaky": flaky,
            "timed_out": timed_out,
            "ignored": skipped,
            "tests": len(cases),
        },
        "failures": failures,
        "slowest": slowest,
    }


def classify_exit_code(exit_code: int) -> str:
    """Return the documented nextest meaning without erasing unknown failures."""
    return EXIT_CLASSES.get(exit_code, "runner_failed")


def _empty_test_results() -> dict[str, Any]:
    return {
        "totals": {
            "passed": 0,
            "failed": 0,
            "flaky": 0,
            "timed_out": 0,
            "ignored": 0,
            "tests": 0,
        },
        "failures": [],
        "slowest": [],
    }


def _run_capture(command: Sequence[str], cwd: Path) -> str:
    try:
        completed = subprocess.run(
            command,
            cwd=cwd,
            check=False,
            capture_output=True,
            text=True,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired):
        return "unavailable"
    output = (completed.stdout or completed.stderr).strip()
    return output.splitlines()[0] if output else "unavailable"


def collect_metadata(profile: str) -> dict[str, Any]:
    """Capture the minimum exact environment needed to reproduce a run."""
    head_sha = _run_capture(("git", "rev-parse", "HEAD"), REPOSITORY_ROOT)
    try:
        status = subprocess.run(
            ("git", "status", "--porcelain"),
            cwd=REPOSITORY_ROOT,
            check=False,
            capture_output=True,
            text=True,
            timeout=30,
        )
        dirty = status.returncode != 0 or bool(status.stdout.strip())
    except (OSError, subprocess.TimeoutExpired):
        dirty = True
    return {
        "head_sha": head_sha,
        "dirty": dirty,
        "rustc": _run_capture(("rustc", "--version"), RUST_ROOT),
        "cargo": _run_capture(("cargo", "--version"), RUST_ROOT),
        "runner": _run_capture(("cargo", "nextest", "--version"), RUST_ROOT),
        "profile": profile,
        "platform": platform.platform(),
        "python": platform.python_version(),
    }


def _write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _tail_log(path: Path) -> str:
    if not path.is_file():
        return ""
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    return _bounded("\n".join(lines[-MAX_LOG_TAIL_LINES:]))


def _markdown(summary: dict[str, Any]) -> str:
    totals = summary["totals"]
    lines = [
        "# Rust test report",
        "",
        f"- Status: `{summary['status']}` (`{summary['exit_class']}`, exit {summary['exit_code']})",
        f"- Head: `{summary.get('head_sha', 'unknown')}` (dirty: `{str(summary.get('dirty', True)).lower()}`)",
        f"- Duration: `{summary['duration_ms']} ms`",
        (
            "- Tests: "
            f"`{totals['tests']}` total, `{totals['passed']}` passed, "
            f"`{totals['failed']}` failed, `{totals['flaky']}` flaky, "
            f"`{totals['timed_out']}` timed out, `{totals['ignored']}` ignored"
        ),
    ]
    if summary["failures"]:
        lines.extend(("", "## Failures", ""))
        for failure in summary["failures"]:
            lines.append(
                f"- `{failure['test']}`: {failure['kind']} — {failure['cause']} "
                f"([{failure['duration_ms']} ms]({failure['junit']}))"
            )
            lines.append(f"  - Rerun: `{failure['rerun']}`")
        if summary["failures_truncated"]:
            remaining = summary["failure_count"] - len(summary["failures"])
            lines.append(f"- … {remaining} more failure records in `failures.jsonl`")
    elif summary.get("diagnostic"):
        lines.extend(("", "## Runner diagnostic", "", "```text", summary["diagnostic"], "```"))
    return "\n".join(lines) + "\n"


def finalize_report(
    *,
    report_dir: Path,
    junit_source: Path,
    exit_code: int,
    command: Sequence[str],
    started_at: str,
    duration_ms: int,
    metadata: dict[str, Any],
) -> dict[str, Any]:
    """Finalize every run, including failures that occur before JUnit exists."""
    report_dir.mkdir(parents=True, exist_ok=True)
    stored_junit = report_dir / "junit.xml"
    if junit_source.is_file() and junit_source.resolve() != stored_junit.resolve():
        shutil.copy2(junit_source, stored_junit)

    parsed: dict[str, Any]
    junit_error = ""
    if stored_junit.is_file():
        try:
            parsed = parse_junit(stored_junit)
        except (ET.ParseError, OSError, TypeError, ValueError) as error:
            parsed = _empty_test_results()
            junit_error = f"unable to parse junit.xml: {error}"
            if exit_code == 0:
                exit_code = 110
    else:
        parsed = _empty_test_results()

    exit_class = classify_exit_code(exit_code)
    all_failures = parsed["failures"]
    manifest = {
        "schema_version": SCHEMA_VERSION,
        "started_at": started_at,
        "finished_at": _utc_now(),
        "duration_ms": duration_ms,
        "command": list(command),
        "exit_code": exit_code,
        "exit_class": exit_class,
        **metadata,
    }
    summary = {
        **manifest,
        "status": "passed" if exit_code == 0 else "failed",
        **parsed,
        "failures": all_failures[:MAX_SUMMARY_FAILURES],
        "failure_count": len(all_failures),
        "failures_truncated": len(all_failures) > MAX_SUMMARY_FAILURES,
        "diagnostic": (
            ""
            if exit_code == 0 or parsed["failures"]
            else _bounded(
                "\n".join(part for part in (junit_error, _tail_log(report_dir / "run.log")) if part)
            )
        ),
        "artifacts": {
            "manifest": "manifest.json",
            "summary": "summary.json",
            "failures": "failures.jsonl",
            "junit": "junit.xml" if stored_junit.is_file() else None,
            "log": "run.log",
        },
    }
    _write_json(report_dir / "manifest.json", manifest)
    _write_json(report_dir / "summary.json", summary)
    with (report_dir / "failures.jsonl").open("w", encoding="utf-8") as stream:
        for failure in all_failures:
            stream.write(json.dumps(failure, sort_keys=True) + "\n")
    (report_dir / "summary.md").write_text(_markdown(summary), encoding="utf-8")
    return summary


def _report_directory(report_root: Path, metadata: dict[str, Any]) -> tuple[str, Path]:
    stamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")
    sha = str(metadata.get("head_sha", "unknown"))[:12]
    run_id = f"{stamp}-{os.getpid()}"
    return run_id, report_root / sha / run_id


def _tee(command: Sequence[str], cwd: Path, log_path: Path, *, append: bool = False) -> int:
    environment = os.environ.copy()
    environment.update(
        {
            "CARGO_TERM_COLOR": "never",
            "NEXTEST_HIDE_PROGRESS_BAR": "1",
            "NO_COLOR": "1",
            "RUST_BACKTRACE": environment.get("RUST_BACKTRACE", "1"),
        }
    )
    log_path.parent.mkdir(parents=True, exist_ok=True)
    try:
        process = subprocess.Popen(  # noqa: S603 -- fixed Cargo executable plus user test selectors
            list(command),
            cwd=cwd,
            env=environment,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            encoding="utf-8",
            errors="replace",
            bufsize=1,
        )
    except OSError as error:
        message = f"unable to start {shlex.join(command)}: {error}\n"
        log_path.write_text(message, encoding="utf-8")
        print(message, end="", file=sys.stderr)
        return 1

    assert process.stdout is not None
    with log_path.open("a" if append else "w", encoding="utf-8", buffering=1) as log:
        if append:
            log.write(f"$ {shlex.join(command)}\n")
        for line in process.stdout:
            print(line, end="", flush=True)
            log.write(line)
    return process.wait()


def _run_logged(command: Sequence[str], cwd: Path, log_path: Path) -> int:
    """Run a non-streaming helper command and append its bounded output to the log."""
    environment = os.environ.copy()
    environment.update({"CARGO_TERM_COLOR": "never", "NO_COLOR": "1"})
    try:
        completed = subprocess.run(  # noqa: S603 -- fixed Cargo coverage commands
            list(command),
            cwd=cwd,
            env=environment,
            check=False,
            capture_output=True,
            text=True,
        )
    except OSError as error:
        with log_path.open("a", encoding="utf-8") as stream:
            stream.write(f"$ {shlex.join(command)}\n{error}\n")
        print(error, file=sys.stderr)
        return 1
    with log_path.open("a", encoding="utf-8") as stream:
        stream.write(f"$ {shlex.join(command)}\n")
        stream.write(completed.stdout)
        stream.write(completed.stderr)
    if completed.stdout:
        print(completed.stdout, end="")
    if completed.stderr:
        print(completed.stderr, end="", file=sys.stderr)
    return completed.returncode


def _update_latest(
    report_root: Path, run_id: str, report_dir: Path, summary: dict[str, Any]
) -> None:
    relative = report_dir.relative_to(report_root).as_posix()
    _write_json(
        report_root / "latest.json",
        {
            "schema_version": SCHEMA_VERSION,
            "run_id": run_id,
            "report_dir": relative,
            "summary": f"{relative}/summary.json",
            "head_sha": summary.get("head_sha"),
            "exit_code": summary["exit_code"],
        },
    )


def _append_github_summary(summary_path: Path) -> None:
    destination = os.environ.get("GITHUB_STEP_SUMMARY")
    if destination:
        with Path(destination).open("a", encoding="utf-8") as stream:
            stream.write(summary_path.read_text(encoding="utf-8"))


def read_cargo_metadata() -> object:
    """Read the locked workspace target inventory without compiling tests."""
    command = ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"]
    try:
        completed = subprocess.run(
            command, cwd=RUST_ROOT, check=False, capture_output=True, text=True, timeout=30
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise ValueError(f"dev Rust selection: cargo metadata failed: {error}") from error
    if completed.returncode != 0:
        raise ValueError(
            f"dev Rust selection: cargo metadata exited {completed.returncode}: "
            f"{_bounded(completed.stderr)}"
        )
    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise ValueError(f"dev Rust selection: invalid cargo metadata JSON: {error}") from error


def _integration_names(package: dict[str, Any]) -> set[str]:
    targets = package.get("targets")
    if not isinstance(targets, list) or not targets:
        raise ValueError("dev Rust selection: package targets must be a nonempty list")
    names: set[str] = set()
    for target in targets:
        if not isinstance(target, dict):
            raise ValueError("dev Rust selection: invalid target record")
        name, kind = target.get("name"), target.get("kind")
        if (
            not isinstance(name, str)
            or not re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_-]*", name)
            or not isinstance(kind, list)
            or not kind
            or not all(isinstance(value, str) for value in kind)
        ):
            raise ValueError("dev Rust selection: invalid target name or kind")
        if "test" in kind:
            names.add(name)
    return names


def dev_target_arguments(metadata: object) -> list[str]:
    """Select one Cargo union, refusing missing required package/target pairs."""
    if not isinstance(metadata, dict):
        raise ValueError("dev Rust selection: metadata must be an object")
    packages, members = metadata.get("packages"), metadata.get("workspace_members")
    if (
        not isinstance(packages, list)
        or not isinstance(members, list)
        or not members
        or not all(isinstance(member, str) for member in members)
        or len(set(members)) != len(members)
    ):
        raise ValueError("dev Rust selection: invalid workspace packages or members")
    by_name: dict[str, set[str]] = {}
    seen_members: set[str] = set()
    for package in packages:
        if not isinstance(package, dict) or not isinstance(package.get("id"), str):
            raise ValueError("dev Rust selection: invalid package record")
        identity = package["id"]
        if identity not in members:
            continue
        name = package.get("name")
        if not isinstance(name, str) or not name or name in by_name or identity in seen_members:
            raise ValueError("dev Rust selection: invalid or duplicate workspace package")
        by_name[name] = _integration_names(package)
        seen_members.add(identity)
    if seen_members != set(members):
        raise ValueError("dev Rust selection: metadata omits a workspace member")

    selected: set[str] = set()
    for package, names in by_name.items():
        if package not in DEV_INTEGRATION_TARGETS:
            selected.update(names)
    for package, required in DEV_INTEGRATION_TARGETS.items():
        if package not in by_name:
            raise ValueError(f"dev Rust selection: required workspace package {package} is missing")
        missing = set(required) - by_name[package]
        if missing:
            raise ValueError(
                f"dev Rust selection: required targets missing from {package}: "
                f"{', '.join(sorted(missing))}"
            )
        selected.update(required)
    return ["--lib", "--bins", *(part for name in sorted(selected) for part in ("--test", name))]


def run_nextest(
    *,
    profile: str,
    workspace: bool,
    extra_args: Sequence[str],
    dev: bool = False,
    report_root: Path = DEFAULT_REPORT_ROOT,
) -> int:
    """Run nextest once and always finalize a structured receipt."""
    metadata = collect_metadata(profile)
    run_id, report_dir = _report_directory(report_root, metadata)
    report_dir.mkdir(parents=True, exist_ok=True)
    junit_source = RUST_ROOT / "target" / "nextest" / profile / "junit.xml"
    junit_source.unlink(missing_ok=True)
    command = ["cargo", "nextest", "run", "--profile", profile, "--locked"]
    if workspace or dev:
        command.append("--workspace")
    started_at = _utc_now()
    started = time.monotonic()
    try:
        if dev:
            if extra_args:
                raise ValueError(
                    "dev Rust selection: --dev cannot include scoped nextest arguments"
                )
            command.extend(dev_target_arguments(read_cargo_metadata()))
    except ValueError as error:
        diagnostic = f"{error}\n"
        (report_dir / "run.log").write_text(diagnostic, encoding="utf-8")
        print(diagnostic, end="", file=sys.stderr)
        exit_code = 102
    else:
        command.extend(extra_args)
        exit_code = _tee(command, RUST_ROOT, report_dir / "run.log")
    duration_ms = round((time.monotonic() - started) * 1_000)
    summary = finalize_report(
        report_dir=report_dir,
        junit_source=junit_source,
        exit_code=exit_code,
        command=command,
        started_at=started_at,
        duration_ms=duration_ms,
        metadata=metadata,
    )
    _update_latest(report_root, run_id, report_dir, summary)
    _append_github_summary(report_dir / "summary.md")
    print(f"Rust test report: {report_dir.relative_to(REPOSITORY_ROOT)}")
    return int(summary["exit_code"])


def _load_latest(report_root: Path) -> tuple[dict[str, Any], Path]:
    pointer_path = report_root / "latest.json"
    if not pointer_path.is_file():
        raise FileNotFoundError("no Rust report found; run `mise run rust:test` first")
    pointer = json.loads(pointer_path.read_text(encoding="utf-8"))
    report_dir = report_root / pointer["report_dir"]
    summary = json.loads((report_dir / "summary.json").read_text(encoding="utf-8"))
    return summary, report_dir


def summarize_latest(report_root: Path = DEFAULT_REPORT_ROOT) -> int:
    """Print the explicit latest summary rather than trusting file mtimes."""
    try:
        _, report_dir = _load_latest(report_root)
    except (FileNotFoundError, KeyError, json.JSONDecodeError) as error:
        print(error, file=sys.stderr)
        return 1
    print((report_dir / "summary.md").read_text(encoding="utf-8"), end="")
    return 0


def rerun_failed(report_root: Path = DEFAULT_REPORT_ROOT) -> int:
    """Rerun exactly the failure identities from the explicit latest receipt."""
    try:
        summary, report_dir = _load_latest(report_root)
        failures = [
            json.loads(line)
            for line in (report_dir / "failures.jsonl").read_text(encoding="utf-8").splitlines()
            if line.strip()
        ]
    except (FileNotFoundError, KeyError, json.JSONDecodeError) as error:
        print(error, file=sys.stderr)
        return 1
    if not failures:
        if summary.get("status") != "passed":
            print(
                "latest Rust run failed before test identities were recorded; "
                "inspect `mise run rust:test:summary` and the retained run.log",
                file=sys.stderr,
            )
            exit_code = summary.get("exit_code", 1)
            return exit_code if isinstance(exit_code, int) and exit_code != 0 else 1
        print("(latest Rust report has no failed tests — nothing to re-run)")
        return 0
    predicates = [
        f"(binary_id(={failure['binary_id']}) & test(={failure['nextest_name']}))"
        for failure in failures
    ]
    return run_nextest(
        profile="ci",
        workspace=True,
        extra_args=("-E", " | ".join(predicates)),
        report_root=report_root,
    )


def write_inventory(report_root: Path = DEFAULT_REPORT_ROOT) -> int:
    """Write stable nextest list JSON for parity audits without taxing every run."""
    metadata = collect_metadata("inventory")
    run_id, report_dir = _report_directory(report_root, metadata)
    report_dir.mkdir(parents=True, exist_ok=True)
    command = [
        "cargo",
        "nextest",
        "list",
        "--workspace",
        "--locked",
        "--message-format",
        "json",
    ]
    completed = subprocess.run(  # noqa: S603 -- fixed repository tool command
        command,
        cwd=RUST_ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    (report_dir / "inventory.json").write_text(completed.stdout, encoding="utf-8")
    (report_dir / "run.log").write_text(completed.stderr, encoding="utf-8")
    manifest = {
        "schema_version": SCHEMA_VERSION,
        "run_id": run_id,
        "command": command,
        "exit_code": completed.returncode,
        "exit_class": classify_exit_code(completed.returncode),
        **metadata,
    }
    _write_json(report_dir / "manifest.json", manifest)
    _write_json(
        report_root / "latest-inventory.json",
        {
            "schema_version": SCHEMA_VERSION,
            "run_id": run_id,
            "report_dir": report_dir.relative_to(report_root).as_posix(),
        },
    )
    print(f"Rust test inventory: {report_dir.relative_to(REPOSITORY_ROOT)}")
    return completed.returncode


def read_coverage_totals(path: Path) -> dict[str, dict[str, int | float]]:
    """Extract stable line, function, and region totals from llvm-cov JSON."""
    payload = json.loads(path.read_text(encoding="utf-8"))
    totals = payload["data"][0]["totals"]
    result: dict[str, dict[str, int | float]] = {}
    for metric in ("lines", "functions", "regions"):
        values = totals[metric]
        result[metric] = {
            "count": int(values["count"]),
            "covered": int(values["covered"]),
            "percent": round(float(values["percent"]), 3),
        }
    return result


def run_coverage(report_root: Path = DEFAULT_COVERAGE_ROOT) -> int:
    """Run one instrumented nextest pass and derive all stable coverage views."""
    metadata = collect_metadata("coverage")
    metadata["coverage_runner"] = _run_capture(("cargo", "llvm-cov", "--version"), RUST_ROOT)
    run_id, report_dir = _report_directory(report_root, metadata)
    report_dir.mkdir(parents=True, exist_ok=True)
    log_path = report_dir / "run.log"
    clean = ["cargo", "llvm-cov", "clean", "--workspace"]
    test_command = [
        "cargo",
        "llvm-cov",
        "nextest",
        "--workspace",
        "--locked",
        "--profile",
        "ci",
        "--no-report",
    ]
    commands = [clean, test_command]
    started_at = _utc_now()
    started = time.monotonic()
    exit_code = _run_logged(clean, RUST_ROOT, log_path)
    if exit_code == 0:
        exit_code = _tee(test_command, RUST_ROOT, log_path, append=True)

    artifacts: dict[str, str | None] = {
        "coverage_summary": None,
        "coverage_json": None,
        "lcov": None,
        "log": "run.log",
    }
    coverage_totals: dict[str, dict[str, int | float]] | None = None
    if exit_code == 0:
        reports = (
            (
                [
                    "cargo",
                    "llvm-cov",
                    "report",
                    "--json",
                    "--summary-only",
                    "--output-path",
                    str(report_dir / "coverage-summary.json"),
                ],
                "coverage_summary",
                "coverage-summary.json",
            ),
            (
                [
                    "cargo",
                    "llvm-cov",
                    "report",
                    "--json",
                    "--output-path",
                    str(report_dir / "coverage.json"),
                ],
                "coverage_json",
                "coverage.json",
            ),
            (
                [
                    "cargo",
                    "llvm-cov",
                    "report",
                    "--lcov",
                    "--output-path",
                    str(report_dir / "lcov.info"),
                ],
                "lcov",
                "lcov.info",
            ),
        )
        for command, artifact_key, artifact_path in reports:
            commands.append(command)
            report_exit = _run_logged(command, RUST_ROOT, log_path)
            if report_exit != 0:
                exit_code = report_exit
                break
            artifacts[artifact_key] = artifact_path

    if exit_code == 0:
        try:
            coverage_totals = read_coverage_totals(report_dir / "coverage-summary.json")
        except (
            OSError,
            IndexError,
            KeyError,
            TypeError,
            ValueError,
            json.JSONDecodeError,
        ) as error:
            exit_code = 1
            with log_path.open("a", encoding="utf-8") as stream:
                stream.write(f"invalid coverage summary: {error}\n")

    duration_ms = round((time.monotonic() - started) * 1_000)
    manifest = {
        "schema_version": SCHEMA_VERSION,
        "run_id": run_id,
        "started_at": started_at,
        "finished_at": _utc_now(),
        "duration_ms": duration_ms,
        "commands": commands,
        "exit_code": exit_code,
        "exit_class": "success" if exit_code == 0 else "coverage_failed",
        "status": "passed" if exit_code == 0 else "failed",
        "totals": coverage_totals,
        "artifacts": artifacts,
        **metadata,
    }
    _write_json(report_dir / "manifest.json", manifest)
    _write_json(
        report_root / "latest.json",
        {
            "schema_version": SCHEMA_VERSION,
            "run_id": run_id,
            "report_dir": report_dir.relative_to(report_root).as_posix(),
            "head_sha": metadata["head_sha"],
            "exit_code": exit_code,
        },
    )
    summary_lines = [
        "# Rust coverage report",
        "",
        f"- Status: `{manifest['status']}` (exit `{exit_code}`)",
        f"- Head: `{metadata['head_sha']}` (dirty: `{str(metadata['dirty']).lower()}`)",
        f"- Duration: `{duration_ms} ms`",
        "- Gate: advisory; no coverage floor is defined",
        f"- Full manifest: `{report_dir.relative_to(REPOSITORY_ROOT) / 'manifest.json'}`",
    ]
    if coverage_totals is not None:
        for metric in ("lines", "functions", "regions"):
            values = coverage_totals[metric]
            summary_lines.append(
                f"- {metric.title()}: `{values['covered']}/{values['count']}` "
                f"(`{values['percent']:.3f}%`)"
            )
    (report_dir / "summary.md").write_text("\n".join(summary_lines) + "\n", encoding="utf-8")
    _append_github_summary(report_dir / "summary.md")
    print(f"Rust coverage report: {report_dir.relative_to(REPOSITORY_ROOT)}")
    return exit_code


def _extra_arguments(values: Sequence[str]) -> list[str]:
    return list(values[1:] if values and values[0] == "--" else values)


def main(argv: Sequence[str] | None = None) -> int:
    """Dispatch Rust report commands."""
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_parser = subparsers.add_parser("run", help="run nextest and finalize a report")
    run_parser.add_argument("--profile", default="ci")
    run_parser.add_argument("--workspace", action="store_true")
    run_parser.add_argument("--dev", action="store_true", help="select the dev workspace contracts")
    run_parser.add_argument("extra", nargs=argparse.REMAINDER)
    subparsers.add_parser("summarize", help="print the explicit latest report")
    subparsers.add_parser("rerun-failed", help="rerun failure IDs from the latest report")
    subparsers.add_parser("inventory", help="write machine-readable nextest inventory")
    subparsers.add_parser("coverage", help="run one instrumented advisory coverage pass")
    args = parser.parse_args(argv)

    if args.command == "run":
        return run_nextest(
            profile=args.profile,
            workspace=args.workspace,
            dev=args.dev,
            extra_args=_extra_arguments(args.extra),
        )
    if args.command == "summarize":
        return summarize_latest()
    if args.command == "rerun-failed":
        return rerun_failed()
    if args.command == "coverage":
        return run_coverage()
    return write_inventory()


if __name__ == "__main__":
    raise SystemExit(main())
