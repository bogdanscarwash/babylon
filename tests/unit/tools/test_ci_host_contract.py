"""Behavior contracts for bounded hosted-runner provisioning."""

from __future__ import annotations

import os
import subprocess
import tomllib
from pathlib import Path

import pytest
import yaml

from tests.unit.test_workflow_hygiene import (
    _automation_paths,
    _automation_step_locations,
    _workflow_paths,
)

REPO_ROOT = Path(__file__).resolve().parents[3]
APT_INSTALLER = REPO_ROOT / "tools" / "install_ci_apt_packages.sh"
POSTGRES_COMPOSE = REPO_ROOT / "tools" / "ci_postgres_compose.sh"
MISE_CONFIG = REPO_ROOT / ".mise.toml"
ANALYSIS_TASKS_CONFIG = REPO_ROOT / ".mise" / "tasks" / "analysis.toml"
WORKFLOWS_DIR = REPO_ROOT / ".github" / "workflows"
ACTIONS_DIR = REPO_ROOT / ".github" / "actions"
HOSTED_RUNTIME_DSN = "dbname=babylon_test host=127.0.0.1 port=5433 user=test password=test"


def _write_executable(path: Path, content: str) -> None:
    """Write one executable test double."""
    path.write_text(content)
    path.chmod(0o755)


def _run_apt_installer(
    tmp_path: Path,
    failures_before_success: int,
    *,
    retry_delay_seconds: str = "0",
    sudo_delay_seconds: str = "0",
    timeout_seconds: str = "5",
) -> subprocess.CompletedProcess[str]:
    """Run the apt helper with a deterministic sudo/apt-get test double."""
    if not APT_INSTALLER.is_file():
        pytest.fail(f"missing bounded apt installer: {APT_INSTALLER}")
    fake_bin = tmp_path / "bin"
    fake_bin.mkdir()
    call_log = tmp_path / "apt-calls.log"
    _write_executable(
        fake_bin / "sudo",
        """#!/usr/bin/env bash
set -euo pipefail
printf '%s\\n' "$*" >> "$FAKE_APT_LOG"
sleep "$FAKE_APT_DELAY_SECONDS"
call_count="$(wc -l < "$FAKE_APT_LOG")"
if (( call_count <= FAKE_APT_FAILURES_BEFORE_SUCCESS )); then
  exit 42
fi
""",
    )
    env = os.environ.copy()
    env.update(
        {
            "BABYLON_CI_APT_RETRY_DELAY_SECONDS": retry_delay_seconds,
            "BABYLON_CI_APT_TIMEOUT_SECONDS": timeout_seconds,
            "FAKE_APT_DELAY_SECONDS": sudo_delay_seconds,
            "FAKE_APT_FAILURES_BEFORE_SUCCESS": str(failures_before_success),
            "FAKE_APT_LOG": str(call_log),
            "PATH": f"{fake_bin}:{env['PATH']}",
        }
    )
    return subprocess.run(  # noqa: S603
        [str(APT_INSTALLER), "binutils", "gdal-bin"],
        cwd=REPO_ROOT,
        env=env,
        capture_output=True,
        text=True,
        check=False,
        timeout=15,
    )


def _apt_calls(tmp_path: Path) -> list[str]:
    """Read calls captured by the apt test double."""
    return (tmp_path / "apt-calls.log").read_text().splitlines()


def test_bounded_apt_succeeds_without_retry(tmp_path: Path) -> None:
    """A healthy mirror performs one update and one install."""
    result = _run_apt_installer(tmp_path, failures_before_success=0)

    assert result.returncode == 0, result.stderr
    assert _apt_calls(tmp_path) == [
        "-n env DEBIAN_FRONTEND=noninteractive apt-get update",
        "-n env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends "
        "binutils gdal-bin",
    ]


def test_bounded_apt_recovers_from_one_transient_failure(tmp_path: Path) -> None:
    """A transient update failure retries the complete apt transaction."""
    result = _run_apt_installer(tmp_path, failures_before_success=1)

    assert result.returncode == 0, result.stderr
    assert len(_apt_calls(tmp_path)) == 3
    assert "attempt 1 of 3 failed" in result.stderr


def test_bounded_apt_stops_after_three_failed_attempts(tmp_path: Path) -> None:
    """A dead mirror cannot consume the whole job timeout."""
    result = _run_apt_installer(tmp_path, failures_before_success=99)

    assert result.returncode != 0
    assert len(_apt_calls(tmp_path)) == 3
    assert "failed after 3 attempts" in result.stderr


def test_bounded_apt_times_out_the_complete_transaction(tmp_path: Path) -> None:
    """Update plus install share one per-attempt deadline."""
    result = _run_apt_installer(
        tmp_path,
        failures_before_success=0,
        sudo_delay_seconds="0.75",
        timeout_seconds="1",
    )

    assert result.returncode != 0
    assert len(_apt_calls(tmp_path)) == 6
    assert "exit 124" in result.stderr


@pytest.mark.parametrize(
    ("timeout_seconds", "retry_delay_seconds", "message"),
    [
        ("301", "0", "APT_TIMEOUT_SECONDS cannot exceed 300"),
        ("5", "31", "APT_RETRY_DELAY_SECONDS cannot exceed 30"),
        ("00", "0", "APT_TIMEOUT_SECONDS must be a canonical positive integer"),
        ("0400", "0", "APT_TIMEOUT_SECONDS must be a canonical positive integer"),
        ("5", "00", "APT_RETRY_DELAY_SECONDS must be a canonical nonnegative integer"),
        ("5", "031", "APT_RETRY_DELAY_SECONDS must be a canonical nonnegative integer"),
        ("18446744073709551615", "0", "APT_TIMEOUT_SECONDS cannot exceed 300"),
        ("5", "18446744073709551615", "APT_RETRY_DELAY_SECONDS cannot exceed 30"),
    ],
)
def test_bounded_apt_rejects_unbounded_configuration(
    tmp_path: Path,
    timeout_seconds: str,
    retry_delay_seconds: str,
    message: str,
) -> None:
    """Environment overrides cannot defeat the fixed wall-time ceiling."""
    result = _run_apt_installer(
        tmp_path,
        failures_before_success=0,
        retry_delay_seconds=retry_delay_seconds,
        timeout_seconds=timeout_seconds,
    )

    assert result.returncode == 2
    assert message in result.stderr
    assert not (tmp_path / "apt-calls.log").exists()


@pytest.mark.parametrize(
    ("command", "expected"),
    [
        (
            "up",
            "babylon-pg-ci|compose -f docker-compose.yml -f docker-compose.ci.yml "
            "up -d --wait babylon-pg",
        ),
        (
            "down",
            "babylon-pg-ci|compose -f docker-compose.yml -f docker-compose.ci.yml down -v",
        ),
    ],
)
def test_ci_postgres_wrapper_uses_one_runner_contract(
    tmp_path: Path, command: str, expected: str
) -> None:
    """Start and cleanup use the same override and isolated named volume."""
    if not POSTGRES_COMPOSE.is_file():
        pytest.fail(f"missing CI Postgres wrapper: {POSTGRES_COMPOSE}")
    fake_bin = tmp_path / "bin"
    fake_bin.mkdir()
    call_log = tmp_path / "docker-calls.log"
    _write_executable(
        fake_bin / "docker",
        """#!/usr/bin/env bash
set -euo pipefail
printf '%s|%s\\n' "$BABYLON_PG_DATA" "$*" > "$FAKE_DOCKER_LOG"
""",
    )
    env = os.environ.copy()
    env.update({"FAKE_DOCKER_LOG": str(call_log), "PATH": f"{fake_bin}:{env['PATH']}"})

    result = subprocess.run(  # noqa: S603
        [str(POSTGRES_COMPOSE), command],
        cwd=REPO_ROOT,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0, result.stderr
    assert call_log.read_text().strip() == expected


def test_automation_routes_apt_through_the_bounded_helper() -> None:
    """No hosted runner can bypass the finite apt retry/timeout policy."""
    violations = [
        str(path)
        for path in _automation_paths()
        for _, step in _automation_step_locations(yaml.safe_load(path.read_text()))
        if "apt-get" in str(step.get("run", ""))
    ]
    assert violations == []


def test_every_postgres_action_caller_uses_checked_cleanup() -> None:
    """Every hosted PostgreSQL job starts and stops the shared CI shape."""
    violations: list[str] = []
    for path in _workflow_paths():
        workflow = yaml.safe_load(path.read_text())
        for job_name, job in (workflow.get("jobs") or {}).items():
            steps = job.get("steps") or []
            uses_postgres = any(
                step.get("uses") == "./.github/actions/postgres-up" for step in steps
            )
            runs_compose_directly = any(
                "docker compose" in str(step.get("run", "")) for step in steps
            )
            cleanup = [
                step for step in steps if step.get("run") == "tools/ci_postgres_compose.sh down"
            ]
            if runs_compose_directly:
                violations.append(f"{path.name}:{job_name}: direct docker compose")
            if uses_postgres and not (
                len(cleanup) == 1 and str(cleanup[0].get("if", "")) == "always()"
            ):
                violations.append(f"{path.name}:{job_name}: missing checked cleanup")
    assert violations == []


def test_postgres_action_builds_the_ci_override_with_buildkit_cache() -> None:
    """The shared image build and runtime consume the same Compose fork."""
    action = (ACTIONS_DIR / "postgres-up" / "action.yml").read_text()
    build = (ACTIONS_DIR / "build-postgres" / "action.yml").read_text()
    ci_compose = yaml.safe_load((REPO_ROOT / "docker-compose.ci.yml").read_text())

    assert "uses: ./.github/actions/build-postgres" in action
    assert "docker-compose.yml" in build
    assert "docker-compose.ci.yml" in build
    assert "cache-from=type=gha,scope=babylon-pg" in build
    assert "cache-to=type=gha,scope=babylon-pg,mode=max" in build
    assert "github.event_name != 'pull_request'" in build
    assert "github.event_name != 'pull_request_target'" in build
    assert "tools/ci_postgres_compose.sh up" in action
    assert ci_compose["services"]["babylon-pg"]["restart"] == "no"


def test_ci_cargo_caches_track_git_sources_and_toolchain() -> None:
    """Pinned Git sources and compiler changes invalidate the right caches."""
    workflow = yaml.safe_load((WORKFLOWS_DIR / "ci.yml").read_text())
    persistence_bootstrap = yaml.safe_load(
        (ACTIONS_DIR / "bootstrap-persistence" / "action.yml").read_text()
    )
    assert any(
        step.get("uses") == "./.github/actions/bootstrap-persistence"
        for job in workflow["jobs"].values()
        for step in job.get("steps", [])
    )
    profile_key = "hashFiles('rust/Cargo.lock', 'rust/rust-toolchain.toml', 'rust/Cargo.toml')"
    for steps in (
        workflow["jobs"]["rust-gate"]["steps"],
        persistence_bootstrap["runs"]["steps"],
    ):
        cache = next(step for step in steps if step.get("id") == "cargo-cache")["with"]
        assert any(
            path == "~/.cargo/git" or path.startswith("~/.cargo/git/")
            for path in cache["path"].splitlines()
        )
        assert profile_key in cache["key"]
        assert all(profile_key in key for key in cache["restore-keys"].splitlines())


def test_ci_debug_profile_reaches_every_persistence_consumer_and_cache(tmp_path: Path) -> None:
    """Hosted builds keep source backtraces without restoring full-debug targets."""
    workflow = yaml.safe_load((WORKFLOWS_DIR / "ci.yml").read_text())
    rust_job = workflow["jobs"]["rust-gate"]
    bootstrap = yaml.safe_load((ACTIONS_DIR / "bootstrap-persistence/action.yml").read_text())
    expected = {
        "CARGO_PROFILE_DEV_DEBUG": "line-tables-only",
        "CARGO_PROFILE_TEST_DEBUG": "line-tables-only",
    }
    assert {key: rust_job.get("env", {}).get(key) for key in expected} == expected

    steps = bootstrap["runs"]["steps"]
    configure = next(step for step in steps if step.get("id") == "cargo-profile")
    restore = next(step for step in steps if step.get("id") == "cargo-cache")
    assert steps.index(configure) < steps.index(restore)
    environment_file = tmp_path / "github-env"
    result = subprocess.run(  # noqa: S603 -- run the repository-owned environment setup
        ["bash", "-euo", "pipefail", "-c", configure["run"]],  # noqa: S607
        env={**os.environ, "GITHUB_ENV": str(environment_file)},
        capture_output=True,
        text=True,
        check=False,
        timeout=5,
    )
    assert result.returncode == 0, result.stderr
    assert (
        dict(line.split("=", 1) for line in environment_file.read_text().splitlines()) == expected
    )

    rust_restore = next(step for step in rust_job["steps"] if step.get("id") == "cargo-cache")
    profile_key = "dev-${{ env.CARGO_PROFILE_DEV_DEBUG }}-test-${{ env.CARGO_PROFILE_TEST_DEBUG }}-"
    for cache in (rust_restore, restore):
        keys = [cache["with"]["key"], *cache["with"]["restore-keys"].splitlines()]
        for key in keys:
            assert profile_key in key
            assert key.index(profile_key) < key.index("hashFiles(")

    # Bootstrap writes the job environment, so each caller's later commands
    # inherit the same profiles unless that workflow explicitly overrides them.
    for path in _workflow_paths():
        current = yaml.safe_load(path.read_text())
        for job in current.get("jobs", {}).values():
            if not any(
                step.get("uses") == "./.github/actions/bootstrap-persistence"
                for step in job.get("steps", [])
            ):
                continue
            for environment in [current.get("env", {}), job.get("env", {})]:
                for key, value in expected.items():
                    assert environment.get(key, value) == value
            for step in job["steps"]:
                for key, value in expected.items():
                    assert step.get("env", {}).get(key, value) == value

    workspace = tomllib.loads((REPO_ROOT / "rust/Cargo.toml").read_text())
    profiles = workspace["profile"]
    local_debug = profiles["dev"].get("debug", 2)
    assert local_debug == 2
    assert profiles.get("test", {}).get("debug", local_debug) == 2


def test_scheduled_failure_artifacts_survive_failure() -> None:
    """Slow evidence survives a scheduled-job failure."""
    weekly_sim = yaml.safe_load((WORKFLOWS_DIR / "weekly-sim-artifacts.yml").read_text())
    upload = next(
        step
        for step in weekly_sim["jobs"]["sim-artifacts"]["steps"]
        if str(step.get("uses", "")).startswith("actions/upload-artifact@")
    )
    assert str(upload.get("if", "")) == "always()"


def test_rust_ci_installs_and_retains_pinned_agent_reports() -> None:
    """The blocking Rust gate must publish exact-head evidence even when red."""
    workflow = yaml.safe_load((WORKFLOWS_DIR / "ci.yml").read_text())
    steps = workflow["jobs"]["rust-gate"]["steps"]
    install = next(step for step in steps if step.get("name") == "Install Rust test reporter")
    upload = next(step for step in steps if step.get("name") == "Upload Rust test reports")

    assert install["uses"] == ("taiki-e/install-action@7b8d4719ee4aaa279bdf55df38dacb9ebfe12a6c")
    assert install["with"] == {
        "tool": "cargo-nextest@0.9.143,cargo-deny@0.20.2",
        "fallback": "none",
    }
    audit = next(step for step in steps if step.get("name", "").startswith("cargo-deny ("))
    assert audit["run"] == (
        "cargo-deny --log-level warn --manifest-path rust/Cargo.toml "
        "--all-features check advisories bans licenses sources"
    )
    assert upload["if"] == "always()"
    assert upload["uses"].startswith("actions/upload-artifact@")
    assert upload["with"] == {
        "name": "rust-test-results-${{ github.sha }}",
        "path": "reports/test-results/rust/",
        "retention-days": 14,
        "if-no-files-found": "error",
    }


def test_weekly_rust_coverage_is_advisory_and_single_run() -> None:
    """Coverage gets retained evidence without taxing or redefining the PR gate."""
    workflow = yaml.safe_load((WORKFLOWS_DIR / "weekly-rust-coverage.yml").read_text())
    triggers = workflow.get("on", workflow.get(True))
    assert triggers["schedule"] == [{"cron": "0 9 * * 4"}]
    assert "workflow_dispatch" in triggers

    job = workflow["jobs"]["rust-coverage"]
    steps = job["steps"]
    checkout = next(
        step for step in steps if str(step.get("uses", "")).startswith("actions/checkout@")
    )
    install = next(step for step in steps if step.get("name") == "Install Rust reporting tools")
    run = next(step for step in steps if step.get("name") == "Generate Rust coverage receipts")
    upload = next(step for step in steps if step.get("name") == "Upload Rust coverage receipts")

    assert checkout["with"]["ref"] == "dev"
    assert checkout["with"]["fetch-depth"] == 0
    assert install["uses"] == ("taiki-e/install-action@7b8d4719ee4aaa279bdf55df38dacb9ebfe12a6c")
    assert install["with"] == {
        "tool": "cargo-nextest@0.9.143,cargo-llvm-cov@0.9.0",
        "fallback": "none",
    }
    assert run["run"] == "mise run rust:coverage"
    assert "fail-under" not in run["run"]
    assert upload["if"] == "always()"
    assert upload["with"]["path"] == "reports/test-results/rust-coverage/"


def test_rust_persistence_workflow_dsns_use_a_literal_loopback() -> None:
    """Rust's local-target guard must accept every hosted workflow DSN.

    Keep the hosted value exact instead of duplicating tokio-postgres parsing in
    Python. A deliberate DSN change must update this contract alongside the
    Rust guard.
    """
    runtime_dsns: list[tuple[Path, str]] = []
    for path in _workflow_paths():
        workflow = yaml.safe_load(path.read_text())
        assert isinstance(workflow, dict), path
        jobs = workflow.get("jobs") or {}
        assert isinstance(jobs, dict), path
        environments = [workflow.get("env") or {}]
        for job in jobs.values():
            assert isinstance(job, dict), path
            steps = job.get("steps") or []
            assert isinstance(steps, list), path
            environments.append(job.get("env") or {})
            for step in steps:
                assert isinstance(step, dict), path
                environments.append(step.get("env") or {})

        for environment in environments:
            assert isinstance(environment, dict), path
            if "BABYLON_RUNTIME_DSN" in environment:
                runtime_dsns.append((path, str(environment["BABYLON_RUNTIME_DSN"])))

    assert runtime_dsns
    for path, dsn in runtime_dsns:
        assert dsn == HOSTED_RUNTIME_DSN, path


def test_weekly_rust_report_is_scoped_to_the_michigan_persistence_slice() -> None:
    """The scheduled artifact diagnoses the committed embedded Rust slice."""
    weekly_sim = yaml.safe_load((WORKFLOWS_DIR / "weekly-sim-artifacts.yml").read_text())
    job = weekly_sim["jobs"]["sim-artifacts"]
    steps = job["steps"]
    report = next(
        step for step in steps if step.get("name") == "Generate Rust Michigan diagnostic report"
    )
    upload = next(
        step for step in steps if str(step.get("uses", "")).startswith("actions/upload-artifact@")
    )

    assert job["timeout-minutes"] == 60
    checkout = next(
        step for step in steps if str(step.get("uses", "")).startswith("actions/checkout@")
    )
    assert checkout["with"]["ref"] == "dev"
    assert report["run"] == "mise run sim:report 130 3000 exclusive"
    assert any(step.get("uses") == "./.github/actions/bootstrap-persistence" for step in steps)
    assert any(step.get("uses") == "./.github/actions/postgres-up" for step in steps)
    assert any(step.get("run") == "mise run db:bootstrap" for step in steps)
    assert str(upload.get("if", "")) == "always()"
    assert str(upload["with"]["name"]).startswith("rust-michigan-simulation-diagnostics-")
    assert upload["with"]["path"] == "reports/sim-runs/"
