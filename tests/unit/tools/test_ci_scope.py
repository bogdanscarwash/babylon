"""Changed input and failed-prerequisite contracts for the CI release split."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest
import yaml
from tools.ci_scope import scope, verify_results
from tools.pr_policy import DEV_CHECK_MANIFEST, MAIN_CHECK_MANIFEST


def test_merge_manifests_require_successful_aggregate_and_release_evidence() -> None:
    assert [(check.context, check.allowed_conclusions) for check in DEV_CHECK_MANIFEST] == [
        ("CI Gate", frozenset({"SUCCESS"}))
    ]
    assert "Main Qualification / PostgreSQL Determinism Bundle" in {
        check.context for check in MAIN_CHECK_MANIFEST
    }


def test_live_workflow_cannot_bypass_the_selected_job_receipt() -> None:
    root = Path(__file__).resolve().parents[3]
    workflow = yaml.safe_load((root / ".github/workflows/ci.yml").read_text())
    gate = workflow["jobs"]["ci-gate"]
    assert gate["if"] == "always()"
    assert set(gate["needs"]) == set(workflow["jobs"]) - {"ci-gate"}
    assert gate["steps"][-1]["run"] == "python3 tools/ci_scope.py --verify"


def test_main_qualification_always_selects_every_lane() -> None:
    assert scope([], full=True) == dict.fromkeys(("full", "rust", "python", "postgres"), True)


@pytest.mark.parametrize("event_name", ["pull_request", "workflow_dispatch"])
def test_release_events_emit_the_complete_database_matrix(tmp_path: Path, event_name: str) -> None:
    event = tmp_path / "event.json"
    event.write_text(json.dumps({"pull_request": {"base": {"ref": "main"}}}))
    output = tmp_path / "output"
    subprocess.run(
        [sys.executable, str(Path(__file__).resolve().parents[3] / "tools/ci_scope.py")],
        env={
            "GITHUB_EVENT_NAME": event_name,
            "GITHUB_EVENT_PATH": str(event),
            "GITHUB_OUTPUT": str(output),
        },
        check=True,
        capture_output=True,
    )
    emitted = dict(line.split("=", 1) for line in output.read_text().splitlines())
    assert all(json.loads(emitted[flag]) for flag in ("full", "rust", "python", "postgres"))
    assert json.loads(emitted["pg-matrix"])["focus"] == [
        "runtime_smoke",
        "reference_integrity",
        "runtime",
        "archive",
        "reader",
        "client",
    ]


@pytest.mark.parametrize("path", ["docs/guide.rst", "README.md", "reports/proof.md"])
def test_prose_does_not_start_database_or_compiler(path: str) -> None:
    assert not any(scope([path], full=False).values())


@pytest.mark.parametrize("path", ["rust/crates/babylon-client/src/main.rs", "rust/Cargo.lock"])
def test_native_changes_compile_and_exercise_a_real_database(path: str) -> None:
    assert scope([path], full=False) == {
        "full": False,
        "rust": True,
        "python": False,
        "postgres": True,
    }


def test_authored_toml_changes_invalidate_runtime_and_database_validation() -> None:
    assert scope(["content/michigan.toml"], full=False) == {
        "full": False,
        "rust": True,
        "python": True,
        "postgres": True,
    }


@pytest.mark.parametrize(
    "path", ["tools/ci_scope.py", ".github/workflows/ci.yml", "unknown.input", "docs/build.sh"]
)
def test_unknown_and_gate_inputs_select_all_lanes(path: str) -> None:
    plan = scope([path], full=False)
    assert all(plan[key] for key in ("rust", "python", "postgres"))


@pytest.mark.parametrize("outcome", ["failure", "cancelled", "skipped", "missing"])
def test_failed_or_missing_selected_job_cannot_pass(outcome: str) -> None:
    plan = scope([], full=True)
    results = dict.fromkeys(
        [
            "scope",
            "fast-gate",
            "ceremony-gate",
            "gitleaks",
            "trivy-config",
            "rust-gate",
            "test-unit",
            "security",
            "pg-integration-shards",
        ],
        "success",
    )
    if outcome == "missing":
        results.pop("rust-gate")
    else:
        results["rust-gate"] = outcome
    with pytest.raises(ValueError, match="rust-gate"):
        verify_results(plan, results)


def test_only_explicitly_unselected_jobs_may_skip() -> None:
    plan = scope(["README.md"], full=False)
    results = dict.fromkeys(
        ["scope", "fast-gate", "ceremony-gate", "gitleaks", "trivy-config"], "success"
    )
    results.update(
        dict.fromkeys(["rust-gate", "test-unit", "security", "pg-integration-shards"], "skipped")
    )
    verify_results(plan, results)
    results["fast-gate"] = "skipped"
    with pytest.raises(ValueError, match="fast-gate"):
        verify_results(plan, results)


@pytest.mark.parametrize("outcome", ["success", "failure", "cancelled", "skipped"])
def test_unrecognized_job_receipt_cannot_bypass_the_gate(outcome: str) -> None:
    plan = scope([], full=True)
    results = dict.fromkeys(
        [
            "scope",
            "fast-gate",
            "ceremony-gate",
            "gitleaks",
            "trivy-config",
            "rust-gate",
            "test-unit",
            "security",
            "pg-integration-shards",
        ],
        "success",
    )
    results["additional-contract"] = outcome
    with pytest.raises(ValueError, match="unexpected job receipts: additional-contract"):
        verify_results(plan, results)
