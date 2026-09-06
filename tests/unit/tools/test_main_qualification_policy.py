"""Contracts for the Director-controlled dev-to-main qualification path."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import pytest
import yaml
from tools.check_main_qualification_event import (
    QualificationEventError,
    validate_event,
)
from tools.pr_policy import (
    DEV_CHECK_MANIFEST,
    GITHUB_ACTIONS_PRODUCER,
    MAIN_CHECK_MANIFEST,
    MAIN_QUALIFICATION_CHECK_MANIFEST,
)

ROOT = Path(__file__).resolve().parents[3]
WORKFLOW_PATH = ROOT / ".github" / "workflows" / "main.yml"
CI_WORKFLOW_PATH = ROOT / ".github" / "workflows" / "ci.yml"
POLICY_PATH = ROOT / ".github" / "settings" / "pr-policy.json"
PROMOTE_PATH = ROOT / "tools" / "promote.sh"
MISE_PATH = ROOT / ".mise.toml"


def _workflow() -> dict[str, Any]:
    payload = yaml.safe_load(WORKFLOW_PATH.read_text(encoding="utf-8"))
    assert isinstance(payload, dict)
    return payload


def _triggers(workflow: dict[str, Any]) -> dict[str, Any]:
    payload = workflow.get("on", workflow.get(True))
    assert isinstance(payload, dict)
    return payload


@pytest.mark.parametrize(
    ("event_name", "ref", "base_ref"),
    [
        ("pull_request", "refs/pull/1/merge", "main"),
        ("workflow_dispatch", "refs/heads/dev", ""),
    ],
)
def test_only_main_pr_and_dev_dispatch_are_accepted(
    event_name: str,
    ref: str,
    base_ref: str,
) -> None:
    validate_event(event_name=event_name, ref=ref, base_ref=base_ref)


@pytest.mark.parametrize(
    ("event_name", "ref", "base_ref"),
    [
        ("pull_request", "refs/pull/2/merge", "dev"),
        ("workflow_dispatch", "refs/heads/main", ""),
        ("workflow_dispatch", "refs/heads/feature/test", ""),
        ("push", "refs/heads/main", ""),
        ("schedule", "refs/heads/dev", ""),
    ],
)
def test_every_other_event_branch_combination_is_rejected(
    event_name: str,
    ref: str,
    base_ref: str,
) -> None:
    with pytest.raises(QualificationEventError):
        validate_event(event_name=event_name, ref=ref, base_ref=base_ref)


def test_workflow_runs_before_main_merge_and_can_be_proved_on_dev() -> None:
    workflow = _workflow()
    triggers = _triggers(workflow)

    assert set(triggers) == {"pull_request", "workflow_dispatch"}
    assert triggers["pull_request"] == {"branches": ["main"]}
    assert "push" not in triggers

    jobs = workflow["jobs"]
    event_job = jobs["event-contract"]
    event_step = next(
        step
        for step in event_job["steps"]
        if step.get("name") == "Accept only a main PR or exact dev proof"
    )
    assert "tools/check_main_qualification_event.py" in event_step["run"]


def test_main_manifest_is_ci_plus_unique_qualification_extension() -> None:
    dev_contexts = {requirement.context for requirement in DEV_CHECK_MANIFEST}
    extension_contexts = {requirement.context for requirement in MAIN_QUALIFICATION_CHECK_MANIFEST}
    main_contexts = {requirement.context for requirement in MAIN_CHECK_MANIFEST}

    assert dev_contexts.isdisjoint(extension_contexts)
    assert main_contexts == dev_contexts | extension_contexts
    assert all(context.startswith("Main Qualification / ") for context in extension_contexts)


def test_only_explicit_advisories_accept_a_failure_conclusion() -> None:
    for requirement in MAIN_CHECK_MANIFEST:
        if requirement.kind == "advisory":
            assert "FAILURE" in requirement.allowed_conclusions
        else:
            assert requirement.allowed_conclusions == frozenset({"SUCCESS"})


def test_every_manifest_entry_declares_the_canonical_actions_producer() -> None:
    assert GITHUB_ACTIONS_PRODUCER.integration_id == 15368
    assert GITHUB_ACTIONS_PRODUCER.slug == "github-actions"
    assert all(
        requirement.producer == GITHUB_ACTIONS_PRODUCER for requirement in MAIN_CHECK_MANIFEST
    )


def test_container_image_scan_is_blocking_and_fail_closed() -> None:
    workflow = _workflow()
    job = workflow["jobs"]["trivy-image"]
    requirement = next(
        requirement
        for requirement in MAIN_QUALIFICATION_CHECK_MANIFEST
        if requirement.context == "Main Qualification / Container Image Scan"
    )
    scan = next(step for step in job["steps"] if step.get("name") == "Scan the built image")

    assert job["name"] == requirement.context
    assert "continue-on-error" not in job
    assert requirement.kind == "blocking"
    assert requirement.allowed_conclusions == frozenset({"SUCCESS"})
    assert scan["with"] == {
        "scan-type": "image",
        "image-ref": "babylon-pg:ci",
        "severity": "HIGH,CRITICAL",
        "exit-code": "1",
        "cache": "true",
    }


def test_retired_ai_suite_has_no_qualification_job() -> None:
    assert not (ROOT / "tests/ai").exists()
    assert "ai-tests" not in _workflow()["jobs"]
    assert "not requires_ollama" in MISE_PATH.read_text(encoding="utf-8")


def test_workflow_emits_every_qualification_context_once() -> None:
    jobs = _workflow()["jobs"]
    names = [job["name"] for job in jobs.values()]
    expected = [requirement.context for requirement in MAIN_QUALIFICATION_CHECK_MANIFEST]

    assert len(names) == len(set(names))
    assert set(names) == set(expected)
    for job_id, job in jobs.items():
        if job_id != "event-contract":
            assert job["needs"] == ["event-contract"]


def test_main_ruleset_requires_the_complete_blocking_manifest() -> None:
    policy = json.loads(POLICY_PATH.read_text(encoding="utf-8"))
    ruleset = policy["main_ruleset"]
    rules = {rule["type"]: rule for rule in ruleset["rules"]}
    checks = rules["required_status_checks"]["parameters"]
    pull_request = rules["pull_request"]["parameters"]
    code_scanning = rules["code_scanning"]["parameters"]
    configured = {
        (entry["context"], entry["integration_id"]) for entry in checks["required_status_checks"]
    }
    expected = {
        (requirement.context, requirement.producer.integration_id)
        for requirement in MAIN_CHECK_MANIFEST
        if requirement.kind == "blocking"
    }

    assert ruleset["conditions"] == {"ref_name": {"exclude": [], "include": ["refs/heads/main"]}}
    assert checks["strict_required_status_checks_policy"] is True
    assert configured == expected
    assert pull_request["allowed_merge_methods"] == ["merge"]
    assert pull_request["required_review_thread_resolution"] is True
    assert code_scanning == {
        "code_scanning_tools": [
            {
                "tool": "CodeQL",
                "alerts_threshold": "all",
                "security_alerts_threshold": "all",
            }
        ]
    }


def test_direct_push_promotion_script_is_retired() -> None:
    assert not PROMOTE_PATH.exists()


def test_shared_pr_blocking_pg_tier_is_rust_only_after_cutover() -> None:
    """The PR gate cannot retain the deleted Python writer's test subset."""
    assert '[tasks."test:integration-pg"]' not in MISE_PATH.read_text(encoding="utf-8")

    workflow = yaml.safe_load(CI_WORKFLOW_PATH.read_text(encoding="utf-8"))
    shard = workflow["jobs"]["pg-integration-shards"]
    assert shard["strategy"] == {
        "fail-fast": False,
        "max-parallel": 4,
        "matrix": "${{ fromJSON(needs.scope.outputs.pg-matrix) }}",
    }
    steps = shard["steps"]
    assert [step.get("run") for step in steps if step.get("run")] == ["tools/run_rust_postgres.sh"]
    assert not any(step.get("uses") == "./.github/actions/bootstrap-python" for step in steps)
    assert not any(step.get("uses") == "./.github/actions/fetch-reference-db" for step in steps)
    aggregator = workflow["jobs"]["ci-gate"]
    assert "pg-integration-shards" in aggregator["needs"]
    assert aggregator["if"] == "always()"
    assert aggregator["steps"][-1]["env"]["CI_NEEDS"] == "${{ toJSON(needs) }}"
    assert aggregator["steps"][-1]["run"] == "python3 tools/ci_scope.py --verify"
