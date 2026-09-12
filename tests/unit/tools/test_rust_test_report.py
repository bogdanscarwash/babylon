"""Agent-oriented Rust test report contracts for PER-310."""

from __future__ import annotations

import json
import subprocess
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

import pytest
from tools import rust_test_report

JUNIT = """\
<?xml version="1.0" encoding="UTF-8"?>
<testsuites tests="4" failures="1" errors="0" time="3.6">
  <testsuite name="babylon_kernel::contract" tests="4" failures="1" time="3.6">
    <testcase name="passes" classname="babylon_kernel::contract" time="0.1" />
    <testcase name="fails" classname="babylon_kernel::contract" time="0.2">
      <failure message="assertion failed: left == right">thread panicked at crates/kernel/tests/contract.rs:42:9
full failure body</failure>
      <system-out>diagnostic stdout</system-out>
      <system-err>diagnostic stderr</system-err>
    </testcase>
    <testcase name="ignored_case" classname="babylon_kernel::contract" time="0">
      <skipped message="ignored" />
    </testcase>
    <testcase name="flaky_case" classname="babylon_kernel::contract" time="3.3">
      <flakyFailure message="failed on attempt 1">first attempt failed</flakyFailure>
    </testcase>
  </testsuite>
</testsuites>
"""


def test_parse_junit_is_failure_first_and_preserves_drilldown_pointer(tmp_path: Path) -> None:
    """The compact view names every exceptional test without embedding passing output."""
    junit = tmp_path / "junit.xml"
    junit.write_text(JUNIT, encoding="utf-8")

    result = rust_test_report.parse_junit(junit)

    assert result["totals"] == {
        "passed": 1,
        "failed": 1,
        "flaky": 1,
        "timed_out": 0,
        "ignored": 1,
        "tests": 4,
    }
    assert [failure["test"] for failure in result["failures"]] == [
        "babylon_kernel::contract::fails",
        "babylon_kernel::contract::flaky_case",
    ]
    failure = result["failures"][0]
    assert failure["kind"] == "failure"
    assert failure["binary_id"] == "babylon_kernel::contract"
    assert failure["nextest_name"] == "fails"
    assert failure["cause"] == "assertion failed: left == right"
    assert failure["source"] == "crates/kernel/tests/contract.rs:42:9"
    assert failure["stdout"] == "diagnostic stdout"
    assert failure["stderr"] == "diagnostic stderr"
    assert failure["junit"] == "junit.xml"
    assert failure["rerun"] == (
        "mise run rust:test:q -- -E 'binary_id(=babylon_kernel::contract) & test(=fails)'"
    )
    assert result["slowest"][0]["test"].endswith("flaky_case")


@pytest.mark.parametrize(
    ("failure_type", "diagnostic", "expected_kind"),
    [
        (
            "test failure with exit code 101",
            'assertion failed: pr_job.contains("timeout-minutes: 69")',
            "failure",
        ),
        (
            "test failure with exit code 101",
            'assertion failed: runner.contains("timeout --signal=TERM")',
            "failure",
        ),
        ("", "assertion failed: error message was 'timed out'", "failure"),
        ("test timeout", "test exceeded its configured execution limit", "timeout"),
    ],
)
def test_junit_timeout_requires_the_nextest_outcome_type(
    tmp_path: Path, failure_type: str, diagnostic: str, expected_kind: str
) -> None:
    """Assertions about timeout policy are ordinary failures, not timed-out tests."""
    suite = ET.Element("testsuite")
    case = ET.SubElement(
        suite, "testcase", name="policy_contract", classname="contracts", time="0.2"
    )
    failure = ET.SubElement(case, "failure", type=failure_type)
    failure.text = diagnostic
    junit = tmp_path / "junit.xml"
    ET.ElementTree(suite).write(junit, encoding="unicode")

    result = rust_test_report.parse_junit(junit)

    assert result["totals"]["failed"] == 1
    assert result["totals"]["timed_out"] == int(expected_kind == "timeout")
    assert result["failures"][0]["kind"] == expected_kind
    assert result["failures"][0]["diagnostic"] == diagnostic


def test_finalize_records_build_failure_even_without_junit(tmp_path: Path) -> None:
    """A missing JUnit file must be a classified runner failure, never zero tests."""
    log = tmp_path / "run.log"
    log.write_text("error: could not compile `babylon-kernel`\n", encoding="utf-8")

    summary = rust_test_report.finalize_report(
        report_dir=tmp_path,
        junit_source=tmp_path / "missing.xml",
        exit_code=101,
        command=["cargo", "nextest", "run"],
        started_at="2026-09-01T00:00:00Z",
        duration_ms=1200,
        metadata={"head_sha": "abc123", "dirty": False},
    )

    assert summary["status"] == "failed"
    assert summary["exit_class"] == "build_failed"
    assert summary["totals"]["tests"] == 0
    assert "could not compile" in summary["diagnostic"]
    assert json.loads((tmp_path / "summary.json").read_text()) == summary
    assert (tmp_path / "summary.md").is_file()
    assert (tmp_path / "manifest.json").is_file()


def test_finalize_classifies_a_truncated_junit_receipt(tmp_path: Path) -> None:
    """An interrupted XML write must still leave a useful failed summary."""
    (tmp_path / "junit.xml").write_text("<testsuites><testcase>", encoding="utf-8")
    (tmp_path / "run.log").write_text("runner stopped while writing output\n", encoding="utf-8")

    summary = rust_test_report.finalize_report(
        report_dir=tmp_path,
        junit_source=tmp_path / "junit.xml",
        exit_code=106,
        command=["cargo", "nextest", "run"],
        started_at="2026-09-01T00:00:00Z",
        duration_ms=900,
        metadata={"head_sha": "abc123", "dirty": False},
    )

    assert summary["status"] == "failed"
    assert summary["exit_class"] == "incomplete_run"
    assert summary["totals"]["tests"] == 0
    assert "unable to parse junit.xml" in summary["diagnostic"]
    assert "runner stopped while writing output" in summary["diagnostic"]


def test_exit_classes_preserve_nextest_discovery_and_selection_failures() -> None:
    """Agents must distinguish failed code from a runner that never ran tests."""
    assert rust_test_report.classify_exit_code(0) == "success"
    assert rust_test_report.classify_exit_code(4) == "no_tests_selected"
    assert rust_test_report.classify_exit_code(100) == "test_failed"
    assert rust_test_report.classify_exit_code(101) == "build_failed"
    assert rust_test_report.classify_exit_code(102) == "cargo_metadata_failed"
    assert rust_test_report.classify_exit_code(104) == "test_list_failed"
    assert rust_test_report.classify_exit_code(1) == "runner_failed"


def test_coverage_totals_are_compact_and_stable(tmp_path: Path) -> None:
    """The agent summary exposes stable metrics without embedding the full export."""
    summary_path = tmp_path / "coverage-summary.json"
    summary_path.write_text(
        json.dumps(
            {
                "type": "llvm.coverage.json.export",
                "version": "3.0.1",
                "data": [
                    {
                        "totals": {
                            "functions": {"count": 20, "covered": 15, "percent": 75.0},
                            "lines": {"count": 100, "covered": 85, "percent": 85.0},
                            "regions": {"count": 150, "covered": 120, "percent": 80.0},
                        }
                    }
                ],
            }
        ),
        encoding="utf-8",
    )

    assert rust_test_report.read_coverage_totals(summary_path) == {
        "functions": {"count": 20, "covered": 15, "percent": 75.0},
        "lines": {"count": 100, "covered": 85, "percent": 85.0},
        "regions": {"count": 150, "covered": 120, "percent": 80.0},
    }


def test_summary_bounds_failures_but_jsonl_preserves_every_record(tmp_path: Path) -> None:
    """The first-read receipt stays bounded while drill-down remains exhaustive."""
    cases = "".join(
        f'<testcase name="case_{index}" classname="suite" time="0.01">'
        f'<failure message="failure {index}">detail {index}</failure></testcase>'
        for index in range(rust_test_report.MAX_SUMMARY_FAILURES + 3)
    )
    junit = tmp_path / "junit.xml"
    junit.write_text(f"<testsuites>{cases}</testsuites>", encoding="utf-8")
    (tmp_path / "run.log").write_text("test failures\n", encoding="utf-8")

    summary = rust_test_report.finalize_report(
        report_dir=tmp_path,
        junit_source=junit,
        exit_code=100,
        command=["cargo", "nextest", "run"],
        started_at="2026-09-01T00:00:00Z",
        duration_ms=100,
        metadata={"head_sha": "abc123", "dirty": False},
    )

    assert summary["failure_count"] == rust_test_report.MAX_SUMMARY_FAILURES + 3
    assert summary["failures_truncated"] is True
    assert len(summary["failures"]) == rust_test_report.MAX_SUMMARY_FAILURES
    assert len((tmp_path / "failures.jsonl").read_text().splitlines()) == (
        rust_test_report.MAX_SUMMARY_FAILURES + 3
    )
    assert "3 more failure records" in (tmp_path / "summary.md").read_text()


def test_run_nextest_returns_finalized_report_failure(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """A malformed receipt must fail the gate even when nextest itself passed."""
    monkeypatch.setattr(
        rust_test_report,
        "collect_metadata",
        lambda _profile: {"head_sha": "abc123", "dirty": False},
    )
    monkeypatch.setattr(
        rust_test_report,
        "_report_directory",
        lambda _root, _metadata: ("run-id", tmp_path / "report"),
    )
    monkeypatch.setattr(rust_test_report, "REPOSITORY_ROOT", tmp_path)
    monkeypatch.setattr(rust_test_report, "_tee", lambda *_args, **_kwargs: 0)
    monkeypatch.setattr(
        rust_test_report,
        "finalize_report",
        lambda **_kwargs: {"exit_code": 110},
    )
    monkeypatch.setattr(rust_test_report, "_update_latest", lambda *_args: None)
    monkeypatch.setattr(rust_test_report, "_append_github_summary", lambda *_args: None)

    assert (
        rust_test_report.run_nextest(
            profile="ci", workspace=True, extra_args=(), report_root=tmp_path
        )
        == 110
    )


def test_rerun_failed_preserves_red_report_without_test_identities(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """Build and discovery failures cannot become a successful rerun no-op."""
    report_dir = tmp_path / "abc123" / "run-id"
    report_dir.mkdir(parents=True)
    (tmp_path / "latest.json").write_text(
        json.dumps({"report_dir": "abc123/run-id"}), encoding="utf-8"
    )
    (report_dir / "summary.json").write_text(
        json.dumps({"status": "failed", "exit_code": 101}), encoding="utf-8"
    )
    (report_dir / "failures.jsonl").write_text("", encoding="utf-8")

    assert rust_test_report.rerun_failed(tmp_path) == 101
    assert "failed before test identities were recorded" in capsys.readouterr().err


def test_tee_replaces_non_utf8_test_output(tmp_path: Path) -> None:
    """Arbitrary test bytes must still produce a finalized UTF-8 run log."""
    log = tmp_path / "run.log"

    exit_code = rust_test_report._tee(
        [
            sys.executable,
            "-c",
            "import sys; sys.stdout.buffer.write(b'bad: \\xff\\n')",
        ],
        tmp_path,
        log,
    )

    assert exit_code == 0
    assert log.read_text(encoding="utf-8") == "bad: �\n"


def _dev_metadata() -> dict[str, object]:
    packages = [
        {
            "id": name,
            "name": name,
            "targets": [{"kind": ["test"], "name": target} for target in targets],
        }
        for name, targets in rust_test_report.DEV_INTEGRATION_TARGETS.items()
    ]
    packages.extend(
        [
            {
                "id": "light",
                "name": "babylon-rtd",
                "targets": [
                    {"kind": ["lib"], "name": "babylon_rtd"},
                    {"kind": ["test"], "name": "canonical_vectors"},
                    {"kind": ["test"], "name": "new_contract"},
                ],
            },
            {
                "id": "dependency",
                "name": "external-dependency",
                "targets": [{"kind": ["test"], "name": "external_test"}],
            },
        ]
    )
    return {
        "packages": packages,
        "workspace_members": [package["id"] for package in packages[:-1]],
    }


def test_dev_selection_keeps_light_contracts_and_current_heavy_seams() -> None:
    """Adding a light contract includes it without expanding the heavy test estate."""
    metadata = _dev_metadata()
    metadata["packages"][0]["targets"].append({"kind": ["test"], "name": "heavy_matrix"})

    arguments = rust_test_report.dev_target_arguments(metadata)

    assert arguments[:2] == ["--lib", "--bins"]
    assert set(arguments[2::2]) == {"--test"}
    names = arguments[3::2]
    assert names == sorted(set(names))
    assert {
        "canonical_vectors",
        "new_contract",
        "material_runtime",
        "michigan_material",
        "statewide_material",
        "staffed_material_replay",
        "decision_surface_contract",
        "dynamic_linking_fence",
        "postgres_catalog_contract",
        "spatial_reference_installer_contract",
    } <= set(names)
    assert {"heavy_matrix", "external_test", "babylon_rtd"}.isdisjoint(names)


@pytest.mark.parametrize(
    "fault", ["required_package", "required_target", "member", "kind", "wildcard"]
)
def test_dev_selection_refuses_incomplete_metadata(fault: str) -> None:
    """A missing gate contract must fail selection rather than silently reduce coverage."""
    metadata = _dev_metadata()
    if fault == "required_package":
        metadata["workspace_members"].remove("babylon-client")
    elif fault == "required_target":
        package = next(p for p in metadata["packages"] if p["name"] == "babylon-client")
        package["targets"] = [t for t in package["targets"] if t["name"] != "dynamic_linking_fence"]
        metadata["packages"][-2]["targets"].append(
            {"kind": ["test"], "name": "dynamic_linking_fence"}
        )
    elif fault == "member":
        metadata["workspace_members"].append("missing-member")
    elif fault == "kind":
        metadata["packages"][0]["targets"][0]["kind"] = "test"
    else:
        metadata["packages"][-2]["targets"].append({"kind": ["test"], "name": "*"})

    with pytest.raises(ValueError, match="dev Rust selection"):
        rust_test_report.dev_target_arguments(metadata)


def test_report_cli_dev_is_explicit_and_full_remains_default(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """The same entrypoint selects dev only when requested and forwards one reported run."""
    calls = []
    monkeypatch.setattr(rust_test_report, "run_nextest", lambda **kwargs: calls.append(kwargs) or 0)

    assert rust_test_report.main(["run", "--workspace", "--dev"]) == 0
    assert rust_test_report.main(["run", "--workspace"]) == 0
    assert [call["dev"] for call in calls] == [True, False]
    assert all(call["workspace"] for call in calls)


@pytest.mark.parametrize("dev", [False, True])
def test_report_runs_one_nextest_command_with_the_selected_cargo_targets(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, dev: bool
) -> None:
    """Dev target selection is a single Cargo union, so library tests cannot repeat."""
    monkeypatch.setattr(rust_test_report, "collect_metadata", lambda _: {"head_sha": "abc"})
    monkeypatch.setattr(
        rust_test_report, "_report_directory", lambda *_: ("run", tmp_path / "report")
    )
    monkeypatch.setattr(rust_test_report, "REPOSITORY_ROOT", tmp_path)
    monkeypatch.setattr(rust_test_report, "RUST_ROOT", tmp_path)
    monkeypatch.setattr(rust_test_report, "_update_latest", lambda *_: None)
    monkeypatch.setattr(rust_test_report, "_append_github_summary", lambda *_: None)
    metadata_calls = []
    commands = []
    monkeypatch.setattr(
        rust_test_report,
        "read_cargo_metadata",
        lambda: metadata_calls.append(True) or _dev_metadata(),
    )
    monkeypatch.setattr(rust_test_report, "_tee", lambda command, *_: commands.append(command) or 0)
    monkeypatch.setattr(rust_test_report, "finalize_report", lambda **_: {"exit_code": 0})

    assert (
        rust_test_report.run_nextest(
            profile="ci", workspace=True, extra_args=(), dev=dev, report_root=tmp_path
        )
        == 0
    )
    assert len(commands) == 1
    command = commands[0]
    assert command[:7] == ["cargo", "nextest", "run", "--profile", "ci", "--locked", "--workspace"]
    assert command.count("--workspace") == 1
    assert command.count("--lib") == int(dev)
    assert command.count("--bins") == int(dev)
    assert "--tests" not in command
    assert len(metadata_calls) == int(dev)
    if dev:
        assert command[7:] == rust_test_report.dev_target_arguments(_dev_metadata())
    else:
        assert len(command) == 7


def test_metadata_selection_failure_leaves_a_failed_report_without_running_nextest(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """A broken metadata probe must neither launch Cargo tests nor reuse a green receipt."""
    monkeypatch.setattr(rust_test_report, "collect_metadata", lambda _: {"head_sha": "abc"})
    monkeypatch.setattr(
        rust_test_report, "_report_directory", lambda *_: ("run", tmp_path / "report")
    )
    monkeypatch.setattr(rust_test_report, "REPOSITORY_ROOT", tmp_path)
    monkeypatch.setattr(rust_test_report, "RUST_ROOT", tmp_path)
    monkeypatch.setattr(rust_test_report, "read_cargo_metadata", lambda: {})
    monkeypatch.setattr(rust_test_report, "_tee", lambda *_: pytest.fail("nextest must not start"))

    assert (
        rust_test_report.run_nextest(
            profile="ci", workspace=True, extra_args=(), dev=True, report_root=tmp_path
        )
        == 102
    )
    summary = json.loads((tmp_path / "report/summary.json").read_text())
    assert summary["status"] == "failed"
    assert summary["exit_class"] == "cargo_metadata_failed"
    assert "dev Rust selection" in summary["diagnostic"]


@pytest.mark.parametrize("failure", ["exit", "json", "timeout"])
def test_metadata_probe_is_locked_bounded_and_reports_its_failure(
    monkeypatch: pytest.MonkeyPatch, failure: str
) -> None:
    """Selection cannot use an unlocked manifest or wait indefinitely for metadata."""

    def failed(command, **kwargs):
        assert command == ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"]
        assert kwargs["timeout"] == 30
        if failure == "timeout":
            raise subprocess.TimeoutExpired(command, 30)
        return subprocess.CompletedProcess(
            command, 1 if failure == "exit" else 0, "{", "metadata failed"
        )

    monkeypatch.setattr(rust_test_report.subprocess, "run", failed)
    with pytest.raises(ValueError, match="dev Rust selection"):
        rust_test_report.read_cargo_metadata()
