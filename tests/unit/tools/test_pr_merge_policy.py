"""Behavior contracts for the sanctioned pull-request merge verifier."""

from __future__ import annotations

import copy
import json
import os
import subprocess
import sys
from pathlib import Path

import pytest
import tools.pr_merge as pr_merge_tool

REPO_ROOT = Path(__file__).resolve().parents[3]
PR_MERGE = REPO_ROOT / "tools" / "pr_merge.py"
HEAD_SHA = "a" * 40
BASE_SHA = "b" * 40
OTHER_SHA = "c" * 40

COPILOT_BOT_ID = 175728472
COPILOT_BOT_NODE_ID = "BOT_kgDOCnlnWA"
GITHUB_ACTIONS_APP_ID = 15368
GITHUB_ACTIONS_APP_SLUG = "github-actions"
DEPENDABOT_BOT_ID = 49699333
DEPENDABOT_ELIGIBILITY_CHECK = "Dependabot Eligibility"
DEPENDABOT_WORKFLOW_PATH = ".github/workflows/dependabot-automerge.yml"
DEPENDABOT_WORKFLOW_ID = 214604133
CI_WORKFLOW_PATH = ".github/workflows/ci.yml"
CI_WORKFLOW_ID = 176131131
ELIGIBILITY_CHECK_ID = 93481313127
SOURCE_RUN_ID = 31396722297
CLASSIFIER_RUN_ID = 31396722301
SOURCE_SUITE_ID = 78264088632
CLASSIFIER_SUITE_ID = 78264088701
MERGE_COMMIT_SHA = "d" * 40

DEV_BLOCKING_CHECKS = ("CI Gate",)

MAIN_BLOCKING_CHECKS = (
    *DEV_BLOCKING_CHECKS,
    "Main Qualification / Event Contract",
    "Main Qualification / Non-Unit Behavioral Contracts",
    "Main Qualification / PostgreSQL Determinism Bundle",
    "Main Qualification / Reference-Data Contracts",
    "Main Qualification / Release Documentation",
    "Main Qualification / Container Image Scan",
    "Main Qualification / Native Download / Linux x86_64",
)


def test_merge_outcomes_are_stable_for_trusted_workflow_consumers() -> None:
    assert {outcome.name: outcome.value for outcome in pr_merge_tool.MergeOutcome} == {
        "SUCCESS": 0,
        "HARD_REFUSAL": 1,
        "MUTATION_INDETERMINATE": 2,
        "DEPENDABOT_MAJOR_REVIEW": 3,
        "EXACT_HEAD_EVIDENCE_PENDING": 4,
    }


def test_gh_timeout_is_bounded_and_normalized(monkeypatch: pytest.MonkeyPatch) -> None:
    observed_timeout: object = None

    def timeout(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        nonlocal observed_timeout
        observed_timeout = kwargs.get("timeout")
        raise subprocess.TimeoutExpired(cmd=args[0], timeout=30)

    monkeypatch.setattr(pr_merge_tool.subprocess, "run", timeout)

    with pytest.raises(pr_merge_tool.GitHubReadError, match="gh timed out after 30 seconds"):
        pr_merge_tool._gh("pr", "view", "742")

    assert observed_timeout == 30


def test_gh_nonzero_exit_is_normalized(monkeypatch: pytest.MonkeyPatch) -> None:
    def nonzero(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        raise subprocess.CalledProcessError(
            returncode=7,
            cmd=args[0],
            output="fallback output",
            stderr="permission denied\n",
        )

    monkeypatch.setattr(pr_merge_tool.subprocess, "run", nonzero)

    with pytest.raises(pr_merge_tool.GitHubReadError, match="gh failed: permission denied"):
        pr_merge_tool._gh("pr", "view", "742")


def test_gh_spawn_failure_is_normalized(monkeypatch: pytest.MonkeyPatch) -> None:
    def fail_spawn(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        raise OSError("executable unavailable")

    monkeypatch.setattr(pr_merge_tool.subprocess, "run", fail_spawn)

    with pytest.raises(
        pr_merge_tool.GitHubReadError,
        match="gh could not start: executable unavailable",
    ):
        pr_merge_tool._gh("pr", "view", "742")


_FAKE_GH = r"""#!/usr/bin/env python3
import json
import os
import sys
from pathlib import Path

args = sys.argv[1:]
scenario = json.loads(Path(os.environ["FAKE_GH_SCENARIO"]).read_text())
with Path(os.environ["FAKE_GH_CALLS"]).open("a") as call_log:
    call_log.write(json.dumps(args) + "\n")

if args[:2] == ["pr", "view"]:
    fields = args[args.index("--json") + 1]
    if fields == "state,headRefOid,mergeCommit":
        if scenario.get("reconciliation_failure"):
            print(scenario["reconciliation_failure"], file=sys.stderr)
            raise SystemExit(8)
        print(json.dumps(scenario["merge_reconciliation"]))
    elif fields == "headRefOid,baseRefOid":
        print(json.dumps(scenario["reread"]))
    elif scenario.get("initial_view_failure"):
        print(scenario["initial_view_failure"], file=sys.stderr)
        raise SystemExit(7)
    else:
        print(json.dumps(scenario["view"]))
elif args[:2] == ["api", "graphql"]:
    print(json.dumps(scenario["threads"]))
elif args[0] == "api" and args[1].startswith(
    "repos/{owner}/{repo}/code-scanning/alerts?state=open"
):
    print(json.dumps(scenario["alerts"]))
elif args[:2] == ["pr", "list"]:
    print(json.dumps(scenario["children"]))
elif args[:2] == ["pr", "merge"]:
    if scenario.get("merge_failure"):
        print(scenario["merge_failure"], file=sys.stderr)
        raise SystemExit(7)
    print("merged")
elif args[0] == "api" and args[1].endswith("/reviews?per_page=100"):
    if scenario.get("rest_reviews_failure"):
        print(scenario["rest_reviews_failure"], file=sys.stderr)
        raise SystemExit(9)
    print(json.dumps(scenario["rest_reviews"]))
elif args[0] == "api" and args[1].endswith("/pulls/742"):
    print(json.dumps(scenario["dependabot_pr"]))
elif args[0] == "api" and "/commits/" in args[1] and "/check-runs?filter=all&per_page=100" in args[1]:
    print(json.dumps(scenario["manifest_check_runs"]))
elif args[0] == "api" and args[1].endswith(f"/actions/runs/{scenario['source_run']['id']}"):
    print(json.dumps(scenario["source_run"]))
elif args[0] == "api" and args[1].endswith(
    f"/actions/runs/{scenario['classifier_run']['id']}"
):
    print(json.dumps(scenario["classifier_run"]))
elif args[0] == "api" and "/attempts/" in args[1] and "/jobs?" in args[1]:
    print(json.dumps(scenario["classifier_jobs"]))
elif args[0] == "api" and args[1].endswith(
    f"/check-runs/{scenario['classifier_check']['id']}"
):
    print(json.dumps(scenario["classifier_check"]))
elif args[0] == "api" and args[1].endswith(
    f"/check-suites/{scenario['source_suite']['id']}"
):
    print(json.dumps(scenario["source_suite"]))
elif args[0] == "api" and args[1].endswith("/pulls/742/commits?per_page=100"):
    print(json.dumps(scenario["dependabot_commits"]))
elif args[0] == "api" and "/comments" in args[1]:
    if scenario.get("comments_failure"):
        print(scenario["comments_failure"], file=sys.stderr)
        raise SystemExit(10)
    print(json.dumps(scenario["comments"]))
else:
    print(f"unexpected fake gh arguments: {args}", file=sys.stderr)
    raise SystemExit(97)
"""


def _check(
    name: str,
    *,
    conclusion: str = "SUCCESS",
    status: str = "COMPLETED",
) -> dict[str, str]:
    return {
        "name": name,
        "status": status,
        "conclusion": conclusion,
        "startedAt": "2026-08-25T12:00:00Z",
    }


def _manifest_check_run(
    name: str,
    *,
    run_id: int,
    head_sha: str = HEAD_SHA,
    conclusion: str = "success",
    status: str = "completed",
    started_at: str = "2026-08-25T12:00:00Z",
    app_id: int = GITHUB_ACTIONS_APP_ID,
    app_slug: str = GITHUB_ACTIONS_APP_SLUG,
) -> dict[str, object]:
    return {
        "id": run_id,
        "name": name,
        "head_sha": head_sha,
        "status": status,
        "conclusion": conclusion,
        "started_at": started_at,
        "app": {"id": app_id, "slug": app_slug},
    }


def _copilot_review(commit_oid: str = HEAD_SHA) -> dict[str, object]:
    return {
        "author": {"login": "copilot-pull-request-reviewer"},
        "commit": {"oid": commit_oid},
        "state": "COMMENTED",
        "submittedAt": "2026-08-25T12:01:00Z",
    }


def _rest_copilot_review(commit_oid: str = HEAD_SHA) -> dict[str, object]:
    return {
        "user": {
            "login": "copilot-pull-request-reviewer[bot]",
            "id": COPILOT_BOT_ID,
            "node_id": COPILOT_BOT_NODE_ID,
            "type": "Bot",
        },
        "commit_id": commit_oid,
        "state": "COMMENTED",
        "submitted_at": "2026-08-25T12:01:00Z",
    }


def _dependabot_commit(
    *,
    head_sha: str = HEAD_SHA,
    update_type: str = "version-update:semver-minor",
    subject: str = "Bump dependencies",
) -> dict[str, object]:
    return {
        "sha": head_sha,
        "author": {"login": "dependabot[bot]", "type": "Bot"},
        "committer": {"login": "web-flow", "id": 19864447, "type": "User"},
        "commit": {
            "message": (
                f"{subject}\n\n---\nupdated-dependencies:\n"
                "- dependency-name: example\n"
                f"  update-type: {update_type}\n..."
            ),
            "verification": {"verified": True, "reason": "valid"},
        },
    }


def _classifier_job(
    *,
    job_id: int = ELIGIBILITY_CHECK_ID,
    run_id: int = CLASSIFIER_RUN_ID,
    head_sha: str = BASE_SHA,
    name: str = DEPENDABOT_ELIGIBILITY_CHECK,
    status: str = "in_progress",
    conclusion: str | None = None,
) -> dict[str, object]:
    return {
        "id": job_id,
        "run_id": run_id,
        "head_sha": head_sha,
        "name": name,
        "status": status,
        "conclusion": conclusion,
        "html_url": (
            f"https://github.com/percy-raskova/babylon/actions/runs/{run_id}/job/{job_id}"
        ),
        "check_run_url": (
            f"https://api.github.com/repos/percy-raskova/babylon/check-runs/{job_id}"
        ),
    }


def _dependabot_actor(
    *,
    login: str = "dependabot[bot]",
    actor_id: int = DEPENDABOT_BOT_ID,
    actor_type: str = "Bot",
) -> dict[str, object]:
    return {"login": login, "id": actor_id, "type": actor_type}


def _source_run(
    *,
    run_id: int = SOURCE_RUN_ID,
    path: str = CI_WORKFLOW_PATH,
    head_sha: str = HEAD_SHA,
    event: str = "pull_request",
    actor: dict[str, object] | None = None,
    triggering_actor: dict[str, object] | None = None,
    suite_id: int = SOURCE_SUITE_ID,
) -> dict[str, object]:
    return {
        "id": run_id,
        "workflow_id": CI_WORKFLOW_ID,
        "path": path,
        "event": event,
        "head_sha": head_sha,
        "status": "completed",
        "conclusion": "success",
        "actor": actor or _dependabot_actor(),
        "triggering_actor": triggering_actor or _dependabot_actor(),
        "check_suite_id": suite_id,
        "run_attempt": 1,
        "head_repository": {"full_name": "percy-raskova/babylon"},
        "repository": {"full_name": "percy-raskova/babylon"},
        "pull_requests": [{"number": 742, "head": {"sha": head_sha}, "base": {"ref": "dev"}}],
    }


def _classifier_run(
    *,
    run_id: int = CLASSIFIER_RUN_ID,
    source_run_id: int = SOURCE_RUN_ID,
    source_head_sha: str = HEAD_SHA,
    path: str = DEPENDABOT_WORKFLOW_PATH,
    actor: dict[str, object] | None = None,
    triggering_actor: dict[str, object] | None = None,
) -> dict[str, object]:
    return {
        "id": run_id,
        "workflow_id": DEPENDABOT_WORKFLOW_ID,
        "path": path,
        "event": "workflow_run",
        "head_sha": BASE_SHA,
        "status": "in_progress",
        "conclusion": None,
        "display_title": (
            f"Dependabot eligibility for CI run {source_run_id} at {source_head_sha}"
        ),
        "actor": actor or _dependabot_actor(),
        "triggering_actor": triggering_actor or _dependabot_actor(),
        "check_suite_id": CLASSIFIER_SUITE_ID,
        "run_attempt": 1,
        "head_repository": {"full_name": "percy-raskova/babylon"},
        "repository": {"full_name": "percy-raskova/babylon"},
    }


def _source_suite(
    *,
    suite_id: int = SOURCE_SUITE_ID,
    head_sha: str = HEAD_SHA,
    app_id: int = GITHUB_ACTIONS_APP_ID,
    app_slug: str = GITHUB_ACTIONS_APP_SLUG,
) -> dict[str, object]:
    return {
        "id": suite_id,
        "head_sha": head_sha,
        "app": {"id": app_id, "slug": app_slug},
        "pull_requests": [{"number": 742, "head": {"sha": head_sha}, "base": {"ref": "dev"}}],
    }


def _classifier_check(
    *,
    check_id: int = ELIGIBILITY_CHECK_ID,
    app_id: int = GITHUB_ACTIONS_APP_ID,
    app_slug: str = GITHUB_ACTIONS_APP_SLUG,
) -> dict[str, object]:
    return {
        "id": check_id,
        "name": DEPENDABOT_ELIGIBILITY_CHECK,
        "head_sha": BASE_SHA,
        "status": "in_progress",
        "conclusion": None,
        "app": {"id": app_id, "slug": app_slug},
        "check_suite": {"id": CLASSIFIER_SUITE_ID},
    }


def _default_scenario() -> dict[str, object]:
    manifest_runs = [
        _manifest_check_run(name, run_id=index)
        for index, name in enumerate(DEV_BLOCKING_CHECKS, start=1)
    ]
    return {
        "view": {
            "state": "OPEN",
            "isDraft": False,
            "headRefOid": HEAD_SHA,
            "headRefName": "codex/PER-264-pr-policy",
            "baseRefOid": BASE_SHA,
            "baseRefName": "dev",
            "statusCheckRollup": [_check(name) for name in DEV_BLOCKING_CHECKS],
            "reviews": [_copilot_review()],
        },
        "reread": {"headRefOid": HEAD_SHA, "baseRefOid": BASE_SHA},
        "rest_reviews": [_rest_copilot_review()],
        "comments": [],
        "alerts": [],
        "children": [],
        "manifest_check_runs": {
            "total_count": len(manifest_runs),
            "check_runs": manifest_runs,
        },
        "initial_view_failure": None,
        "rest_reviews_failure": None,
        "comments_failure": None,
        "merge_failure": None,
        "reconciliation_failure": None,
        "merge_reconciliation": {
            "state": "MERGED",
            "headRefOid": HEAD_SHA,
            "mergeCommit": {"oid": MERGE_COMMIT_SHA},
        },
        "dependabot_pr": {
            "head": {"sha": HEAD_SHA, "ref": "dependabot/uv/example-1.2.3"},
            "base": {"ref": "dev"},
            "user": {
                "login": "dependabot[bot]",
                "id": DEPENDABOT_BOT_ID,
                "type": "Bot",
            },
        },
        "dependabot_commits": [_dependabot_commit()],
        "source_run": _source_run(),
        "source_suite": _source_suite(),
        "classifier_run": _classifier_run(),
        "classifier_jobs": {"total_count": 1, "jobs": [_classifier_job()]},
        "classifier_check": _classifier_check(),
        "threads": {
            "data": {
                "repository": {
                    "pullRequest": {
                        "reviewThreads": {
                            "nodes": [],
                            "pageInfo": {"hasNextPage": False},
                        }
                    }
                }
            }
        },
    }


def _run_pr_merge(
    tmp_path: Path,
    *arguments: str,
    scenario: dict[str, object] | None = None,
) -> tuple[subprocess.CompletedProcess[str], list[list[str]]]:
    fake_bin = tmp_path / "bin"
    fake_bin.mkdir()
    fake_gh = fake_bin / "gh"
    fake_gh.write_text(_FAKE_GH)
    fake_gh.chmod(0o755)
    scenario_path = tmp_path / "scenario.json"
    scenario_path.write_text(json.dumps(scenario or _default_scenario()))
    calls_path = tmp_path / "calls.jsonl"
    env = os.environ.copy()
    env.update(
        {
            "FAKE_GH_CALLS": str(calls_path),
            "FAKE_GH_SCENARIO": str(scenario_path),
            "PATH": f"{fake_bin}:{env['PATH']}",
        }
    )
    command_arguments = list(arguments)
    if "--dependabot" in command_arguments:
        command_arguments.extend(
            [
                "--dependabot-source-run",
                str(SOURCE_RUN_ID),
                "--dependabot-classifier-run",
                str(CLASSIFIER_RUN_ID),
            ]
        )
    result = subprocess.run(  # noqa: S603
        [sys.executable, str(PR_MERGE), "742", *command_arguments],
        cwd=REPO_ROOT,
        env=env,
        capture_output=True,
        text=True,
        check=False,
        timeout=10,
    )
    calls = (
        [json.loads(line) for line in calls_path.read_text().splitlines()]
        if calls_path.exists()
        else []
    )
    return result, calls


def _scoped_ci_scenario() -> dict[str, object]:
    scenario = _default_scenario()
    base_url = f"https://github.com/percy-raskova/babylon/actions/runs/{SOURCE_RUN_ID}/job"
    gate = scenario["manifest_check_runs"]["check_runs"][0]
    gate.update({"details_url": f"{base_url}/1", "check_suite": {"id": SOURCE_SUITE_ID}})
    skipped = _manifest_check_run("Rust Validation", run_id=2, conclusion="skipped")
    skipped.update({"details_url": f"{base_url}/2", "check_suite": {"id": SOURCE_SUITE_ID}})
    scenario["manifest_check_runs"]["check_runs"].append(skipped)
    scenario["manifest_check_runs"]["total_count"] = 2
    entry = _check("Rust Validation", conclusion="SKIPPED")
    entry["detailsUrl"] = f"{base_url}/2"
    scenario["view"]["statusCheckRollup"].append(entry)
    return scenario


def test_verified_ci_scope_can_skip_a_native_job_without_fabricating_success(
    tmp_path: Path,
) -> None:
    result, calls = _run_pr_merge(tmp_path, "--verify-only", scenario=_scoped_ci_scenario())

    assert result.returncode == 0, result.stderr
    assert not any(call[:2] == ["pr", "merge"] for call in calls)


@pytest.mark.parametrize("fault", ["workflow", "head", "suite", "app", "run", "unknown", "gate"])
def test_scoped_skips_require_the_exact_successful_ci_workflow(tmp_path: Path, fault: str) -> None:
    scenario = _scoped_ci_scenario()
    skipped = scenario["manifest_check_runs"]["check_runs"][1]
    if fault == "workflow":
        scenario["source_run"]["workflow_id"] = 9
    elif fault == "head":
        scenario["source_run"]["head_sha"] = OTHER_SHA
    elif fault == "suite":
        skipped["check_suite"]["id"] = 9
    elif fault == "app":
        skipped["app"]["id"] = 9
    elif fault == "run":
        skipped["details_url"] = skipped["details_url"].replace(str(SOURCE_RUN_ID), "9")
    elif fault == "unknown":
        skipped["name"] = "Unregistered Optional Check"
        scenario["view"]["statusCheckRollup"][1]["name"] = skipped["name"]
    else:
        scenario["manifest_check_runs"]["check_runs"][0]["conclusion"] = "failure"

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 1, result.stderr
    assert not any(call[:2] == ["pr", "merge"] for call in calls)


def test_policy_migration_can_require_workflow_identity_without_skipped_jobs(
    tmp_path: Path,
) -> None:
    scenario = _scoped_ci_scenario()
    scenario["view"]["statusCheckRollup"] = scenario["view"]["statusCheckRollup"][:1]
    scenario["source_run"]["workflow_id"] = 9

    result, calls = _run_pr_merge(
        tmp_path, "--verify-only", "--require-ci-workflow", scenario=scenario
    )

    assert result.returncode == 1, result.stderr
    assert not any(call[:2] == ["pr", "merge"] for call in calls)


def test_newer_scoped_failure_invalidates_an_older_skip_receipt(tmp_path: Path) -> None:
    scenario = _scoped_ci_scenario()
    newer = copy.deepcopy(scenario["manifest_check_runs"]["check_runs"][1])
    newer.update({"id": 3, "conclusion": "failure"})
    scenario["manifest_check_runs"]["check_runs"].append(newer)
    scenario["manifest_check_runs"]["total_count"] = 3

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 1, result.stderr
    assert not any(call[:2] == ["pr", "merge"] for call in calls)


def _view(scenario: dict[str, object]) -> dict[str, object]:
    view = scenario["view"]
    assert isinstance(view, dict)
    return view


def _merge_calls(calls: list[list[str]]) -> list[list[str]]:
    return [call for call in calls if call[:2] == ["pr", "merge"]]


def _use_main_manifest(scenario: dict[str, object]) -> None:
    _view(scenario)["statusCheckRollup"] = [_check(name) for name in MAIN_BLOCKING_CHECKS]
    runs = [
        _manifest_check_run(name, run_id=index)
        for index, name in enumerate(MAIN_BLOCKING_CHECKS, start=1)
    ]
    scenario["manifest_check_runs"] = {
        "total_count": len(runs),
        "check_runs": runs,
    }


def _manifest_runs(scenario: dict[str, object]) -> list[dict[str, object]]:
    payload = scenario["manifest_check_runs"]
    assert isinstance(payload, dict)
    runs = payload["check_runs"]
    assert isinstance(runs, list)
    assert all(isinstance(run, dict) for run in runs)
    return runs


def _thread_connection(scenario: dict[str, object]) -> dict[str, object]:
    threads = scenario["threads"]
    assert isinstance(threads, dict)
    data = threads["data"]
    assert isinstance(data, dict)
    repository = data["repository"]
    assert isinstance(repository, dict)
    pull_request = repository["pullRequest"]
    assert isinstance(pull_request, dict)
    connection = pull_request["reviewThreads"]
    assert isinstance(connection, dict)
    return connection


@pytest.mark.parametrize("failure_kind", ["timeout", "nonzero", "spawn"])
def test_mutating_merge_boundary_reconciles_every_command_failure(
    monkeypatch: pytest.MonkeyPatch,
    failure_kind: str,
) -> None:
    calls: list[list[str]] = []

    def run(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
        calls.append(command)
        assert kwargs["timeout"] == 30
        if command[1:3] == ["pr", "merge"]:
            if failure_kind == "timeout":
                raise subprocess.TimeoutExpired(cmd=command, timeout=30)
            if failure_kind == "nonzero":
                raise subprocess.CalledProcessError(7, command, stderr="transport failed")
            raise OSError("executable unavailable")
        return subprocess.CompletedProcess(
            command,
            0,
            stdout=json.dumps(
                {
                    "state": "MERGED",
                    "headRefOid": HEAD_SHA,
                    "mergeCommit": {"oid": MERGE_COMMIT_SHA},
                }
            ),
            stderr="",
        )

    monkeypatch.setattr(pr_merge_tool.subprocess, "run", run)

    assert pr_merge_tool._merge_pr(742, HEAD_SHA, delete_branch=False) == ""
    assert len(calls) == 2
    assert calls[0][1:3] == ["pr", "merge"]
    assert calls[1][1:3] == ["pr", "view"]


def test_cli_read_failure_is_refused_without_traceback_or_unbounded_detail(
    tmp_path: Path,
) -> None:
    scenario = _default_scenario()
    scenario["initial_view_failure"] = "\x1b[31m" + ("authenticated detail\n" * 100)

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 1
    assert result.stderr.startswith("pr:merge REFUSED — gh failed: ")
    assert "Traceback" not in result.stderr
    assert "\x1b" not in result.stderr
    assert result.stderr.count("\n") == 1
    assert len(result.stderr) <= 500
    assert _merge_calls(calls) == []


def test_cli_confirms_an_exact_head_merge_after_command_failure(tmp_path: Path) -> None:
    scenario = _default_scenario()
    scenario["merge_failure"] = "connection lost after request"

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert f"PR #742 merged at {HEAD_SHA}" in result.stdout
    assert "INDETERMINATE" not in result.stderr
    assert len(_merge_calls(calls)) == 1
    assert (
        sum(
            call[:2] == ["pr", "view"]
            and call[call.index("--json") + 1] == "state,headRefOid,mergeCommit"
            for call in calls
        )
        == 1
    )


@pytest.mark.parametrize(
    "reconciliation",
    [
        {"state": "OPEN", "headRefOid": HEAD_SHA, "mergeCommit": {"oid": MERGE_COMMIT_SHA}},
        {
            "state": "MERGED",
            "headRefOid": OTHER_SHA,
            "mergeCommit": {"oid": MERGE_COMMIT_SHA},
        },
        {"state": "MERGED", "headRefOid": HEAD_SHA, "mergeCommit": None},
        {"state": "MERGED", "headRefOid": HEAD_SHA, "mergeCommit": "malformed"},
        {"state": "MERGED", "headRefOid": HEAD_SHA, "mergeCommit": {"oid": ""}},
    ],
    ids=[
        "not-merged",
        "wrong-head",
        "missing-merge-commit",
        "malformed-merge-commit",
        "empty-merge-commit-oid",
    ],
)
def test_cli_reports_indeterminate_when_reconciliation_cannot_confirm_exact_merge(
    tmp_path: Path,
    reconciliation: dict[str, object],
) -> None:
    scenario = _default_scenario()
    scenario["merge_failure"] = "connection lost after request"
    scenario["merge_reconciliation"] = reconciliation

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 2
    assert result.stderr.startswith("pr:merge INDETERMINATE — ")
    assert "REFUSED" not in result.stderr
    assert "Traceback" not in result.stderr
    assert "merged at" not in result.stdout
    assert len(_merge_calls(calls)) == 1


def test_cli_reports_indeterminate_when_merge_reconciliation_read_fails(tmp_path: Path) -> None:
    scenario = _default_scenario()
    scenario["merge_failure"] = "connection lost after request"
    scenario["reconciliation_failure"] = "reconciliation unavailable"

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 2
    assert "pr:merge INDETERMINATE —" in result.stderr
    assert "reconciliation failed" in result.stderr
    assert "Traceback" not in result.stderr
    assert len(_merge_calls(calls)) == 1


def test_confirmed_merge_warns_that_requested_branch_deletion_may_be_incomplete(
    tmp_path: Path,
) -> None:
    scenario = _default_scenario()
    scenario["merge_failure"] = "connection lost after request"

    result, calls = _run_pr_merge(tmp_path, "--delete-branch", scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert "branch deletion may be incomplete" in result.stderr
    assert len(_merge_calls(calls)) == 1


def test_verify_only_runs_full_policy_without_merging(tmp_path: Path) -> None:
    result, calls = _run_pr_merge(tmp_path, "--verify-only")

    assert result.returncode == 0, result.stderr
    assert _merge_calls(calls) == []
    assert "verified at" in result.stdout
    assert any(call[:2] == ["api", "graphql"] for call in calls)


def test_all_rest_list_queries_are_bounded_without_pagination(tmp_path: Path) -> None:
    result, calls = _run_pr_merge(tmp_path, "--verify-only", "--delete-branch")

    assert result.returncode == 0, result.stderr
    assert not any("--paginate" in call for call in calls)
    comments = next(call for call in calls if call[0] == "api" and "/comments" in call[1])
    alerts = next(call for call in calls if call[0] == "api" and "code-scanning/alerts" in call[1])
    check_runs = next(
        call
        for call in calls
        if call[0] == "api" and "/commits/" in call[1] and "/check-runs?" in call[1]
    )
    children = next(call for call in calls if call[:2] == ["pr", "list"])
    assert "per_page=100" in comments[1]
    assert "per_page=100" in alerts[1]
    assert "filter=all&per_page=100" in check_runs[1]
    assert children[children.index("--limit") + 1] == "100"


def test_ninety_nine_code_scanning_alerts_keep_the_zero_floor_refusal(
    tmp_path: Path,
) -> None:
    scenario = _default_scenario()
    scenario["alerts"] = [{"number": number, "state": "open"} for number in range(1, 100)]

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 1
    assert "99 open code-scanning alert(s) — the zero floor is a STOP" in result.stderr
    assert "code-scanning alerts reached the 100-item safety bound" not in result.stderr
    assert _merge_calls(calls) == []


def test_exactly_full_code_scanning_page_refuses_as_potentially_truncated(
    tmp_path: Path,
) -> None:
    scenario = _default_scenario()
    scenario["alerts"] = [{"number": number, "state": "open"} for number in range(1, 101)]

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 1
    assert "code-scanning alerts reached the 100-item safety bound" in result.stderr
    assert "100 open code-scanning alert(s)" not in result.stderr
    assert _merge_calls(calls) == []


def test_full_copilot_comment_page_is_a_bounded_advisory(tmp_path: Path) -> None:
    scenario = _default_scenario()
    scenario["comments"] = [
        {
            "id": index,
            "html_url": f"https://example.test/comment/{index}",
            "in_reply_to_id": None,
            "user": {"login": "human-reviewer"},
        }
        for index in range(100)
    ]

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert result.stderr.startswith("pr:merge ADVISORY — Copilot comment evidence unavailable")
    assert "100-item safety bound" in result.stderr
    assert "Traceback" not in result.stderr
    assert len(_merge_calls(calls)) == 1


def test_one_hundred_check_entries_refuse_a_potentially_truncated_rollup(
    tmp_path: Path,
) -> None:
    scenario = _default_scenario()
    rollup = _view(scenario)["statusCheckRollup"]
    assert isinstance(rollup, list)
    rollup.extend(
        _check(f"Unknown successful extension {index}")
        for index in range(100 - len(DEV_BLOCKING_CHECKS))
    )

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode != 0
    assert "100-item safety bound" in result.stderr
    assert _merge_calls(calls) == []


def test_expected_head_refuses_a_different_snapshot(tmp_path: Path) -> None:
    result, calls = _run_pr_merge(
        tmp_path,
        "--verify-only",
        "--expected-head",
        OTHER_SHA,
    )

    assert result.returncode == 1
    assert "expected head" in result.stderr
    assert _merge_calls(calls) == []


@pytest.mark.parametrize("moved_ref", ["headRefOid", "baseRefOid"])
def test_moved_snapshot_refuses_merge(tmp_path: Path, moved_ref: str) -> None:
    scenario = _default_scenario()
    reread = scenario["reread"]
    assert isinstance(reread, dict)
    reread[moved_ref] = OTHER_SHA

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 1
    assert f"{moved_ref.removesuffix('RefOid')} moved" in result.stderr
    assert _merge_calls(calls) == []


def test_default_policy_requires_dev_base(tmp_path: Path) -> None:
    scenario = _default_scenario()
    _view(scenario)["baseRefName"] = "feature/stack"

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 1
    assert "base branch feature/stack is not dev" in result.stderr
    assert _merge_calls(calls) == []


def test_main_base_requires_director_main_flag(tmp_path: Path) -> None:
    scenario = _default_scenario()
    _view(scenario).update({"baseRefName": "main", "headRefName": "fix/security"})

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 1
    assert "base branch main is not dev" in result.stderr
    assert _merge_calls(calls) == []


@pytest.mark.parametrize("head_name", ["dev", "fix/security"])
def test_director_main_accepts_only_sanctioned_main_sources(tmp_path: Path, head_name: str) -> None:
    scenario = _default_scenario()
    _view(scenario).update({"baseRefName": "main", "headRefName": head_name})
    _use_main_manifest(scenario)

    result, calls = _run_pr_merge(tmp_path, "--director-main", scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert len(_merge_calls(calls)) == 1


def test_director_main_refuses_container_image_scan_failure(tmp_path: Path) -> None:
    scenario = _default_scenario()
    _view(scenario).update({"baseRefName": "main", "headRefName": "dev"})
    _use_main_manifest(scenario)
    scan = next(
        run
        for run in _manifest_runs(scenario)
        if run["name"] == "Main Qualification / Container Image Scan"
    )
    scan["conclusion"] = "failure"

    result, calls = _run_pr_merge(
        tmp_path,
        "--director-main",
        "--verify-only",
        scenario=scenario,
    )

    assert result.returncode == 1
    assert "Main Qualification / Container Image Scan" in result.stderr
    assert _merge_calls(calls) == []


def test_director_main_refuses_feature_source(tmp_path: Path) -> None:
    scenario = _default_scenario()
    _view(scenario).update({"baseRefName": "main", "headRefName": "feature/unsafe"})

    result, calls = _run_pr_merge(tmp_path, "--director-main", scenario=scenario)

    assert result.returncode == 1
    assert "main requires head dev or fix/*" in result.stderr
    assert _merge_calls(calls) == []


def test_director_main_flag_is_invalid_for_dev_base(tmp_path: Path) -> None:
    result, calls = _run_pr_merge(tmp_path, "--director-main")

    assert result.returncode == 1
    assert "--director-main requires base main" in result.stderr
    assert _merge_calls(calls) == []


def test_delete_branch_never_deletes_dev(tmp_path: Path) -> None:
    scenario = _default_scenario()
    _view(scenario).update({"baseRefName": "main", "headRefName": "dev"})
    _use_main_manifest(scenario)

    result, calls = _run_pr_merge(
        tmp_path,
        "--director-main",
        "--delete-branch",
        scenario=scenario,
    )

    assert result.returncode == 1
    assert "refused for protected branch dev" in result.stderr
    assert _merge_calls(calls) == []


def test_merge_is_atomically_matched_to_verified_head(tmp_path: Path) -> None:
    result, calls = _run_pr_merge(tmp_path)

    assert result.returncode == 0, result.stderr
    assert _merge_calls(calls) == [
        ["pr", "merge", "742", "--merge", "--match-head-commit", HEAD_SHA]
    ]


@pytest.mark.parametrize(
    "reviews",
    [
        [],
        [_copilot_review(OTHER_SHA)],
        [
            {
                **_copilot_review(),
                "state": "PENDING",
                "submittedAt": None,
            }
        ],
    ],
    ids=["missing", "stale", "pending"],
)
def test_copilot_review_state_is_advisory_on_verified_head(
    tmp_path: Path, reviews: list[dict[str, object]]
) -> None:
    scenario = _default_scenario()
    _view(scenario)["reviews"] = reviews

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert "pr:merge ADVISORY — no completed Copilot review on verified head" in result.stderr
    assert len(_merge_calls(calls)) == 1


@pytest.mark.parametrize(
    "login",
    ["attacker-copilot-reviewer", "copilot-pull-request-reviewer[bot]"],
)
def test_graphql_copilot_review_identity_mismatch_is_advisory(tmp_path: Path, login: str) -> None:
    scenario = _default_scenario()
    review = _copilot_review()
    review["author"] = {"login": login}
    _view(scenario)["reviews"] = [review]

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert "pr:merge ADVISORY — no completed Copilot review on verified head" in result.stderr
    assert len(_merge_calls(calls)) == 1


@pytest.mark.parametrize(
    ("field", "value"),
    [("id", 1), ("node_id", "BOT_impostor"), ("type", "User")],
)
def test_rest_copilot_review_identity_mismatch_is_advisory(
    tmp_path: Path,
    field: str,
    value: object,
) -> None:
    scenario = _default_scenario()
    review = _rest_copilot_review()
    user = review["user"]
    assert isinstance(user, dict)
    user[field] = value
    scenario["rest_reviews"] = [review]

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert "pr:merge ADVISORY — no completed Copilot review on verified head" in result.stderr
    assert len(_merge_calls(calls)) == 1


def test_top_level_copilot_comment_without_reply_is_advisory(tmp_path: Path) -> None:
    scenario = _default_scenario()
    scenario["comments"] = [
        {
            "id": 91,
            "html_url": "https://example.test/comment/91",
            "in_reply_to_id": None,
            "user": {
                "login": "Copilot",
                "id": COPILOT_BOT_ID,
                "node_id": COPILOT_BOT_NODE_ID,
                "type": "Bot",
            },
        }
    ]

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert (
        "pr:merge ADVISORY — unaddressed Copilot comment "
        "https://example.test/comment/91" in result.stderr
    )
    assert len(_merge_calls(calls)) == 1


@pytest.mark.parametrize(
    "malformation",
    ["missing-id", "unhashable-id", "unhashable-reply-id"],
)
def test_malformed_canonical_copilot_comment_is_a_bounded_advisory(
    tmp_path: Path,
    malformation: str,
) -> None:
    scenario = _default_scenario()
    comment: dict[str, object] = {
        "id": 91,
        "html_url": "https://example.test/comment/91",
        "in_reply_to_id": None,
        "user": {
            "login": "Copilot",
            "id": COPILOT_BOT_ID,
            "node_id": COPILOT_BOT_NODE_ID,
            "type": "Bot",
        },
    }
    if malformation == "missing-id":
        comment.pop("id")
    elif malformation == "unhashable-id":
        comment["id"] = []
    else:
        comment["in_reply_to_id"] = []
    scenario["comments"] = [comment]

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert "pr:merge ADVISORY — Copilot comment evidence unavailable" in result.stderr
    assert "Traceback" not in result.stderr
    assert len(result.stderr) <= 550
    assert len(_merge_calls(calls)) == 1


@pytest.mark.parametrize(
    ("failure_field", "expected_context"),
    [
        ("rest_reviews_failure", "Copilot review evidence unavailable"),
        ("comments_failure", "Copilot comment evidence unavailable"),
    ],
)
def test_copilot_only_api_failure_is_a_bounded_advisory(
    tmp_path: Path,
    failure_field: str,
    expected_context: str,
) -> None:
    scenario = _default_scenario()
    scenario[failure_field] = "private detail\n" * 100

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert f"pr:merge ADVISORY — {expected_context}: gh failed:" in result.stderr
    assert "Traceback" not in result.stderr
    assert "private detail" in result.stderr
    assert len(result.stderr) <= 550
    assert len(_merge_calls(calls)) == 1


@pytest.mark.parametrize(
    ("name", "conclusion", "status"),
    [
        (DEV_BLOCKING_CHECKS[0], "", "IN_PROGRESS"),
        (DEV_BLOCKING_CHECKS[0], "SKIPPED", "COMPLETED"),
        (DEV_BLOCKING_CHECKS[0], "NEUTRAL", "COMPLETED"),
    ],
    ids=["late", "skipped", "neutral"],
)
def test_dev_aggregate_requires_explicit_success(
    tmp_path: Path,
    name: str,
    conclusion: str,
    status: str,
) -> None:
    scenario = _default_scenario()
    run = next(run for run in _manifest_runs(scenario) if run["name"] == name)
    run["conclusion"] = conclusion.lower()
    run["status"] = status.lower()

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 1
    assert name in result.stderr
    assert _merge_calls(calls) == []


def test_missing_expected_dev_check_blocks_registration_race(tmp_path: Path) -> None:
    scenario = _default_scenario()
    _manifest_runs(scenario).pop(0)
    payload = scenario["manifest_check_runs"]
    assert isinstance(payload, dict)
    payload["total_count"] = len(_manifest_runs(scenario))

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 1
    assert DEV_BLOCKING_CHECKS[0] in result.stderr
    assert "missing" in result.stderr
    assert _merge_calls(calls) == []


def test_hard_refusal_outranks_pending_exact_head_evidence(tmp_path: Path) -> None:
    scenario = _default_scenario()
    _manifest_runs(scenario).pop(0)
    payload = scenario["manifest_check_runs"]
    assert isinstance(payload, dict)
    payload["total_count"] = len(_manifest_runs(scenario))
    rollup = _view(scenario)["statusCheckRollup"]
    assert isinstance(rollup, list)
    rollup.append(_check("Unknown hard failure", conclusion="FAILURE"))

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "pr:merge REFUSED — Unknown hard failure: FAILURE" in result.stderr
    assert "pr:merge PENDING" in result.stderr
    assert _merge_calls(calls) == []


def test_dependabot_missing_exact_head_check_is_pending(tmp_path: Path) -> None:
    scenario = _default_scenario()
    _manifest_runs(scenario).pop(0)
    payload = scenario["manifest_check_runs"]
    assert isinstance(payload, dict)
    payload["total_count"] = len(_manifest_runs(scenario))

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 4
    assert "pr:merge PENDING" in result.stderr
    assert DEV_BLOCKING_CHECKS[0] in result.stderr
    assert _merge_calls(calls) == []


def test_status_context_success_cannot_satisfy_a_required_check(tmp_path: Path) -> None:
    scenario = _default_scenario()
    missing_name = DEV_BLOCKING_CHECKS[0]
    runs = _manifest_runs(scenario)
    runs[:] = [run for run in runs if run["name"] != missing_name]
    payload = scenario["manifest_check_runs"]
    assert isinstance(payload, dict)
    payload["total_count"] = len(runs)
    rollup = _view(scenario)["statusCheckRollup"]
    assert isinstance(rollup, list)
    rollup[0] = {
        "context": missing_name,
        "state": "SUCCESS",
        "startedAt": "2026-08-29T00:00:00Z",
    }

    result, calls = _run_pr_merge(tmp_path, "--verify-only", scenario=scenario)

    assert result.returncode == 1
    assert f"{missing_name}: missing" in result.stderr
    assert _merge_calls(calls) == []


@pytest.mark.parametrize(
    ("field", "value", "message"),
    [
        ("id", 1, "required producer"),
        ("slug", "attacker-actions", "required producer"),
        ("id", "15368", "malformed app identity"),
    ],
)
def test_wrong_or_malformed_app_cannot_satisfy_a_required_check(
    tmp_path: Path,
    field: str,
    value: object,
    message: str,
) -> None:
    scenario = _default_scenario()
    app = _manifest_runs(scenario)[0]["app"]
    assert isinstance(app, dict)
    app[field] = value

    result, calls = _run_pr_merge(tmp_path, "--verify-only", scenario=scenario)

    assert result.returncode == 1
    assert message in result.stderr
    assert _merge_calls(calls) == []


def test_later_wrong_app_duplicate_cannot_displace_the_canonical_run(tmp_path: Path) -> None:
    scenario = _default_scenario()
    runs = _manifest_runs(scenario)
    forged = copy.deepcopy(runs[0])
    forged.update(
        {
            "id": 1000,
            "started_at": "2026-08-29T00:00:00Z",
            "conclusion": "failure",
            "app": {"id": 1, "slug": "attacker-actions"},
        }
    )
    runs.append(forged)
    payload = scenario["manifest_check_runs"]
    assert isinstance(payload, dict)
    payload["total_count"] = len(runs)

    result, calls = _run_pr_merge(tmp_path, "--verify-only", scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert _merge_calls(calls) == []


def test_later_canonical_duplicate_controls_the_required_verdict(tmp_path: Path) -> None:
    scenario = _default_scenario()
    runs = _manifest_runs(scenario)
    rerun = copy.deepcopy(runs[0])
    rerun.update(
        {
            "id": 1000,
            "started_at": "2026-08-29T00:00:00Z",
            "conclusion": "failure",
        }
    )
    runs.append(rerun)
    payload = scenario["manifest_check_runs"]
    assert isinstance(payload, dict)
    payload["total_count"] = len(runs)

    result, calls = _run_pr_merge(tmp_path, "--verify-only", scenario=scenario)

    assert result.returncode == 1
    assert DEV_BLOCKING_CHECKS[0] in result.stderr
    assert _merge_calls(calls) == []


def test_newer_queued_canonical_duplicate_with_no_started_at_blocks_merge(
    tmp_path: Path,
) -> None:
    scenario = _default_scenario()
    runs = _manifest_runs(scenario)
    queued = copy.deepcopy(runs[0])
    queued.update(
        {
            "id": 1000,
            "started_at": None,
            "status": "queued",
            "conclusion": None,
        }
    )
    runs.append(queued)
    payload = scenario["manifest_check_runs"]
    assert isinstance(payload, dict)
    payload["total_count"] = len(runs)

    result, calls = _run_pr_merge(tmp_path, "--verify-only", scenario=scenario)

    assert result.returncode == 1
    assert f"{DEV_BLOCKING_CHECKS[0]}: still QUEUED" in result.stderr
    assert _merge_calls(calls) == []


def test_dependabot_queued_exact_head_check_is_pending(tmp_path: Path) -> None:
    scenario = _default_scenario()
    run = _manifest_runs(scenario)[0]
    run.update({"status": "queued", "conclusion": None})

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 4
    assert f"{DEV_BLOCKING_CHECKS[0]}: still QUEUED" in result.stderr
    assert _merge_calls(calls) == []


def test_repeated_lower_canonical_check_run_id_is_rejected(tmp_path: Path) -> None:
    scenario = _default_scenario()
    runs = _manifest_runs(scenario)
    newer = copy.deepcopy(runs[0])
    newer["id"] = 1000
    runs.append(newer)
    duplicate = copy.deepcopy(runs[0])
    duplicate["started_at"] = None
    duplicate["status"] = "queued"
    duplicate["conclusion"] = None
    runs.append(duplicate)
    payload = scenario["manifest_check_runs"]
    assert isinstance(payload, dict)
    payload["total_count"] = len(runs)

    result, calls = _run_pr_merge(tmp_path, "--verify-only", scenario=scenario)

    assert result.returncode == 1
    assert f"repeats id {duplicate['id']}" in result.stderr
    assert _merge_calls(calls) == []


def test_required_check_run_must_repeat_the_exact_head_sha(tmp_path: Path) -> None:
    scenario = _default_scenario()
    _manifest_runs(scenario)[0]["head_sha"] = OTHER_SHA

    result, calls = _run_pr_merge(tmp_path, "--verify-only", scenario=scenario)

    assert result.returncode == 1
    assert "wrong head" in result.stderr
    assert _merge_calls(calls) == []


@pytest.mark.parametrize("total_count", [10, 100])
def test_incomplete_or_full_check_run_page_is_refused(
    tmp_path: Path,
    total_count: int,
) -> None:
    scenario = _default_scenario()
    payload = scenario["manifest_check_runs"]
    assert isinstance(payload, dict)
    payload["total_count"] = total_count

    result, calls = _run_pr_merge(tmp_path, "--verify-only", scenario=scenario)

    assert result.returncode == 1
    assert "check runs" in result.stderr
    assert _merge_calls(calls) == []


def test_graphql_required_check_failure_does_not_override_canonical_rest_evidence(
    tmp_path: Path,
) -> None:
    scenario = _default_scenario()
    rollup = _view(scenario)["statusCheckRollup"]
    assert isinstance(rollup, list)
    rollup[0] = _check(DEV_BLOCKING_CHECKS[0], conclusion="FAILURE")

    result, calls = _run_pr_merge(tmp_path, "--verify-only", scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert _merge_calls(calls) == []


@pytest.mark.parametrize("conclusion", ["NEUTRAL", "SKIPPED"])
def test_unknown_non_successful_check_is_not_globally_advisory(
    tmp_path: Path,
    conclusion: str,
) -> None:
    scenario = _default_scenario()
    rollup = _view(scenario)["statusCheckRollup"]
    assert isinstance(rollup, list)
    rollup.append(_check("Unknown extension", conclusion=conclusion))

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 1
    assert "Unknown extension" in result.stderr
    assert _merge_calls(calls) == []


def test_dependabot_unknown_neutral_rollup_is_pending(tmp_path: Path) -> None:
    scenario = _default_scenario()
    rollup = _view(scenario)["statusCheckRollup"]
    assert isinstance(rollup, list)
    rollup.append(_check("CodeQL aggregate", conclusion="NEUTRAL"))

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 4
    assert "pr:merge PENDING — CodeQL aggregate: NEUTRAL" in result.stderr
    assert _merge_calls(calls) == []


def test_hard_refusal_outranks_dependabot_neutral_pending(tmp_path: Path) -> None:
    scenario = _default_scenario()
    rollup = _view(scenario)["statusCheckRollup"]
    assert isinstance(rollup, list)
    rollup.append(_check("CodeQL aggregate", conclusion="NEUTRAL"))
    scenario["alerts"] = [{"number": 17, "state": "open"}]

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "pr:merge REFUSED — 1 open code-scanning alert" in result.stderr
    assert "pr:merge PENDING — CodeQL aggregate: NEUTRAL" in result.stderr
    assert _merge_calls(calls) == []


def test_main_unknown_neutral_rollup_remains_hard(tmp_path: Path) -> None:
    scenario = _default_scenario()
    _view(scenario).update({"baseRefName": "main", "headRefName": "dev"})
    _use_main_manifest(scenario)
    rollup = _view(scenario)["statusCheckRollup"]
    assert isinstance(rollup, list)
    rollup.append(_check("CodeQL aggregate", conclusion="NEUTRAL"))

    result, calls = _run_pr_merge(
        tmp_path,
        "--director-main",
        "--verify-only",
        scenario=scenario,
    )

    assert result.returncode == 1
    assert "pr:merge REFUSED — CodeQL aggregate: NEUTRAL" in result.stderr
    assert _merge_calls(calls) == []


def test_unknown_success_is_informational_after_complete_manifest(tmp_path: Path) -> None:
    scenario = _default_scenario()
    rollup = _view(scenario)["statusCheckRollup"]
    assert isinstance(rollup, list)
    rollup.append(_check("Unknown successful extension"))

    result, calls = _run_pr_merge(tmp_path, "--verify-only", scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert _merge_calls(calls) == []


def test_director_main_rejects_complete_dev_manifest_as_wrong_qualification(
    tmp_path: Path,
) -> None:
    scenario = _default_scenario()
    _view(scenario).update({"baseRefName": "main", "headRefName": "dev"})

    result, calls = _run_pr_merge(tmp_path, "--director-main", scenario=scenario)

    assert result.returncode == 1
    assert MAIN_BLOCKING_CHECKS[len(DEV_BLOCKING_CHECKS)] in result.stderr
    assert _merge_calls(calls) == []


def test_dependabot_mode_requires_native_ci_and_classifier_provenance(
    tmp_path: Path,
) -> None:
    result, calls = _run_pr_merge(tmp_path, "--dependabot")

    assert result.returncode == 0, result.stderr
    assert len(_merge_calls(calls)) == 1
    assert any(
        call[0] == "api" and call[1].endswith(f"/actions/runs/{SOURCE_RUN_ID}") for call in calls
    )
    assert any(
        call[0] == "api" and call[1].endswith(f"/actions/runs/{CLASSIFIER_RUN_ID}")
        for call in calls
    )
    assert any(
        call[0] == "api" and f"/actions/runs/{CLASSIFIER_RUN_ID}/attempts/1/jobs?" in call[1]
        for call in calls
    )
    assert any(
        call[0] == "api" and call[1].endswith(f"/check-runs/{ELIGIBILITY_CHECK_ID}")
        for call in calls
    )
    assert any(call[0] == "api" and "/pulls/742/commits?per_page=100" in call[1] for call in calls)


def test_reviewed_major_update_uses_normal_path_and_dependabot_requests_review(
    tmp_path: Path,
) -> None:
    scenario = _default_scenario()
    scenario["dependabot_commits"] = [_dependabot_commit(update_type="version-update:semver-major")]

    normal_dir = tmp_path / "normal"
    normal_dir.mkdir()
    normal, normal_calls = _run_pr_merge(normal_dir, scenario=scenario)
    dependabot_dir = tmp_path / "dependabot"
    dependabot_dir.mkdir()
    unattended, unattended_calls = _run_pr_merge(
        dependabot_dir,
        "--dependabot",
        scenario=scenario,
    )

    assert normal.returncode == 0, normal.stderr
    assert len(_merge_calls(normal_calls)) == 1
    assert unattended.returncode == 3
    assert "authenticated Dependabot semver-major" in unattended.stdout
    assert _merge_calls(unattended_calls) == []


def test_semver_major_requires_authenticated_dependabot_provenance(tmp_path: Path) -> None:
    scenario = _default_scenario()
    scenario["dependabot_commits"] = [_dependabot_commit(update_type="version-update:semver-major")]
    dependabot_pr = scenario["dependabot_pr"]
    assert isinstance(dependabot_pr, dict)
    dependabot_pr["user"] = {"login": "attacker", "id": 7, "type": "User"}

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "exact Dependabot bot author" in result.stderr
    assert "MANUAL REVIEW" not in result.stdout
    assert not any(call[0] == "api" and "/pulls/742/commits?" in call[1] for call in calls)
    assert _merge_calls(calls) == []


def test_unknown_dependabot_update_class_is_a_hard_refusal(tmp_path: Path) -> None:
    scenario = _default_scenario()
    scenario["dependabot_commits"] = [_dependabot_commit(update_type="version-update:calendar")]

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "not a known semver class" in result.stderr
    assert _merge_calls(calls) == []


def _cargo_zero_x_scenario(subject: str) -> dict[str, object]:
    scenario = _default_scenario()
    dependabot_pr = scenario["dependabot_pr"]
    assert isinstance(dependabot_pr, dict)
    dependabot_pr["head"] = {"sha": HEAD_SHA, "ref": "dependabot/cargo/rust/h3o-0.11.0"}
    scenario["dependabot_commits"] = [_dependabot_commit(subject=subject)]
    return scenario


def test_cargo_zero_x_minor_requires_manual_review_despite_semver_minor_trailer(
    tmp_path: Path,
) -> None:
    scenario = _cargo_zero_x_scenario("fix(deps): bump h3o from 0.10.0 to 0.11.0 in /rust")

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 3
    assert "Cargo 0.x minor" in result.stdout
    assert _merge_calls(calls) == []


def test_cargo_zero_x_patch_auto_merges(tmp_path: Path) -> None:
    scenario = _cargo_zero_x_scenario("fix(deps): bump h3o from 0.10.0 to 0.10.2 in /rust")

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert len(_merge_calls(calls)) == 1


def test_cargo_zero_x_prerelease_suffix_still_requires_manual_review(
    tmp_path: Path,
) -> None:
    scenario = _cargo_zero_x_scenario("fix(deps): bump h3o from 0.10.0-alpha.1 to 0.11.0 in /rust")

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 3
    assert "Cargo 0.x minor" in result.stdout
    assert _merge_calls(calls) == []


def test_cargo_zero_x_build_suffix_patch_auto_merges(tmp_path: Path) -> None:
    scenario = _cargo_zero_x_scenario("fix(deps): bump h3o from 0.10.0+vendor to 0.10.2 in /rust")

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert len(_merge_calls(calls)) == 1


def test_cargo_stable_minor_auto_merges(tmp_path: Path) -> None:
    scenario = _cargo_zero_x_scenario("fix(deps): bump serde from 1.2.3 to 1.4.0 in /rust")

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert len(_merge_calls(calls)) == 1


def test_non_cargo_zero_x_minor_auto_merges(tmp_path: Path) -> None:
    scenario = _default_scenario()
    scenario["dependabot_commits"] = [
        _dependabot_commit(subject="chore(deps-dev): bump example from 0.10.0 to 0.11.0")
    ]

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert len(_merge_calls(calls)) == 1


def test_dependabot_synchronization_invalidates_old_head_classification(tmp_path: Path) -> None:
    scenario = _default_scenario()
    dependabot_pr = scenario["dependabot_pr"]
    assert isinstance(dependabot_pr, dict)
    dependabot_pr["head"] = {"sha": OTHER_SHA}

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "Dependabot head" in result.stderr
    assert _merge_calls(calls) == []


def test_dependabot_attributed_commit_with_generic_signer_cannot_authorize(
    tmp_path: Path,
) -> None:
    scenario = _default_scenario()
    scenario["dependabot_commits"] = [_dependabot_commit()]
    scenario["source_run"] = _source_run(
        actor=_dependabot_actor(login="attacker", actor_id=7, actor_type="User"),
        triggering_actor=_dependabot_actor(login="attacker", actor_id=7, actor_type="User"),
    )

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "wrong CI workflow actor" in result.stderr
    assert not any(call[0] == "api" and "/pulls/742/commits?" in call[1] for call in calls)
    assert _merge_calls(calls) == []


def test_appended_non_dependabot_commit_cannot_reuse_an_exact_head_check(
    tmp_path: Path,
) -> None:
    scenario = _default_scenario()
    scenario["dependabot_commits"] = [
        _dependabot_commit(head_sha=OTHER_SHA),
        {
            "sha": HEAD_SHA,
            "author": {"login": "attacker"},
            "committer": {"login": "attacker"},
        },
    ]
    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "exactly one current commit" in result.stderr
    assert _merge_calls(calls) == []


def test_forged_dependabot_author_with_different_committer_cannot_authorize(
    tmp_path: Path,
) -> None:
    scenario = _default_scenario()
    scenario["dependabot_commits"] = [_dependabot_commit()]
    scenario["source_run"] = _source_run(
        actor=_dependabot_actor(login="attacker", actor_id=7, actor_type="User"),
        triggering_actor=_dependabot_actor(login="attacker", actor_id=7, actor_type="User"),
    )

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "wrong CI workflow actor" in result.stderr
    assert not any(call[0] == "api" and "/pulls/742/commits?" in call[1] for call in calls)
    assert _merge_calls(calls) == []


def test_same_actions_app_check_from_another_workflow_cannot_authorize(
    tmp_path: Path,
) -> None:
    scenario = _default_scenario()
    scenario["source_run"] = _source_run(path=".github/workflows/attacker.yml")

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "CI workflow path" in result.stderr
    assert _merge_calls(calls) == []


def test_dependabot_source_ci_run_must_match_exact_head(tmp_path: Path) -> None:
    scenario = _default_scenario()
    scenario["source_run"] = _source_run(head_sha=OTHER_SHA)

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "head" in result.stderr
    assert _merge_calls(calls) == []


@pytest.mark.parametrize(
    ("app_id", "app_slug"),
    [
        (1, GITHUB_ACTIONS_APP_SLUG),
        (GITHUB_ACTIONS_APP_ID, "attacker-actions"),
    ],
    ids=["wrong-app-id", "wrong-app-slug"],
)
def test_dependabot_source_suite_requires_canonical_actions_app(
    tmp_path: Path,
    app_id: int,
    app_slug: str,
) -> None:
    scenario = _default_scenario()
    scenario["source_suite"] = _source_suite(app_id=app_id, app_slug=app_slug)

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "check-suite app identity" in result.stderr
    assert _merge_calls(calls) == []


@pytest.mark.parametrize(
    ("status", "conclusion"),
    [("queued", None), ("completed", "failure")],
    ids=["incomplete", "non-success"],
)
def test_dependabot_source_ci_run_must_complete_successfully(
    tmp_path: Path,
    status: str,
    conclusion: str | None,
) -> None:
    scenario = _default_scenario()
    source_run = scenario["source_run"]
    assert isinstance(source_run, dict)
    source_run.update({"status": status, "conclusion": conclusion})

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "did not complete successfully" in result.stderr
    assert _merge_calls(calls) == []


def test_dependabot_classifier_jobs_refuse_a_full_page(tmp_path: Path) -> None:
    scenario = _default_scenario()
    jobs = [_classifier_job(job_id=index + 1) for index in range(100)]
    scenario["classifier_jobs"] = {
        "total_count": 100,
        "jobs": jobs,
    }

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "100-item safety bound" in result.stderr
    assert _merge_calls(calls) == []


def test_classifier_run_must_bind_exact_source_run_and_head(tmp_path: Path) -> None:
    scenario = _default_scenario()
    scenario["classifier_run"] = _classifier_run(source_run_id=SOURCE_RUN_ID + 1)

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "source run/head binding mismatch" in result.stderr
    assert _merge_calls(calls) == []


def test_classifier_native_check_requires_canonical_actions_app(tmp_path: Path) -> None:
    scenario = _default_scenario()
    scenario["classifier_check"] = _classifier_check(app_id=1)

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "wrong Actions app identity" in result.stderr
    assert _merge_calls(calls) == []


def test_missing_native_classifier_job_refuses_dependabot_mode(tmp_path: Path) -> None:
    scenario = _default_scenario()
    scenario["classifier_jobs"] = {"total_count": 0, "jobs": []}

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "expected one native classifier job" in result.stderr
    assert _merge_calls(calls) == []


def test_completed_classifier_run_cannot_be_replayed(tmp_path: Path) -> None:
    scenario = _default_scenario()
    classifier_run = scenario["classifier_run"]
    assert isinstance(classifier_run, dict)
    classifier_run.update({"status": "completed", "conclusion": "failure"})

    result, calls = _run_pr_merge(tmp_path, "--dependabot", scenario=scenario)

    assert result.returncode == 1
    assert "not currently executing" in result.stderr
    assert _merge_calls(calls) == []


def test_unresolved_review_thread_refuses_merge(tmp_path: Path) -> None:
    scenario = _default_scenario()
    _thread_connection(scenario)["nodes"] = [
        {
            "isResolved": False,
            "comments": {"nodes": [{"url": "https://example.test/thread/17"}]},
        }
    ]

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 1
    assert "unresolved review thread https://example.test/thread/17" in result.stderr
    assert _merge_calls(calls) == []


def test_more_than_one_hundred_review_threads_refuses_instead_of_paging(
    tmp_path: Path,
) -> None:
    scenario = _default_scenario()
    connection = _thread_connection(scenario)
    page_info = connection["pageInfo"]
    assert isinstance(page_info, dict)
    page_info["hasNextPage"] = True

    result, calls = _run_pr_merge(tmp_path, scenario=scenario)

    assert result.returncode == 1
    assert "more than 100 review threads" in result.stderr
    graph_calls = [call for call in calls if call[:2] == ["api", "graphql"]]
    assert len(graph_calls) == 1
    assert "--paginate" not in graph_calls[0]
    assert "reviewThreads(first: 100)" in " ".join(graph_calls[0])
    assert _merge_calls(calls) == []


def test_exactly_one_hundred_complete_review_threads_remain_within_bound(
    tmp_path: Path,
) -> None:
    scenario = _default_scenario()
    connection = _thread_connection(scenario)
    connection["nodes"] = [
        {
            "isResolved": True,
            "comments": {"nodes": [{"url": f"https://example.test/thread/{index}"}]},
        }
        for index in range(100)
    ]

    result, calls = _run_pr_merge(tmp_path, "--verify-only", scenario=scenario)

    assert result.returncode == 0, result.stderr
    assert _merge_calls(calls) == []


def test_delete_branch_still_refuses_a_stacked_child(tmp_path: Path) -> None:
    scenario = copy.deepcopy(_default_scenario())
    scenario["children"] = [{"number": 743}]

    result, calls = _run_pr_merge(tmp_path, "--delete-branch", scenario=scenario)

    assert result.returncode == 1
    assert "open PR(s) [743] base on this branch" in result.stderr
    assert _merge_calls(calls) == []


def test_delete_branch_refuses_an_exactly_full_child_page(tmp_path: Path) -> None:
    scenario = copy.deepcopy(_default_scenario())
    scenario["children"] = [{"number": number} for number in range(1, 101)]

    result, calls = _run_pr_merge(tmp_path, "--delete-branch", scenario=scenario)

    assert result.returncode == 1
    assert "child pull requests reached the 100-item safety bound" in result.stderr
    assert _merge_calls(calls) == []
