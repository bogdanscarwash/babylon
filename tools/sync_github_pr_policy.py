#!/usr/bin/env python3
"""Check, apply, or roll back Babylon's GitHub pull-request policy."""

from __future__ import annotations

import argparse
import base64
import binascii
import json
import os
import re
import subprocess
import sys
from copy import deepcopy
from pathlib import Path
from typing import Final, Protocol, cast
from urllib.parse import quote

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools.pr_policy import (  # noqa: E402
    DEV_BLOCKING_CONTEXTS,
    DEV_CHECK_MANIFEST,
    MAIN_BLOCKING_CONTEXTS,
    MAIN_QUALIFICATION_CHECK_MANIFEST,
    CheckRequirement,
    manifest_for_base,
)

REPOSITORY = "percy-raskova/babylon"
REPOSITORY_ENDPOINT = f"repos/{REPOSITORY}"
ACTIONS_PERMISSIONS_ENDPOINT = f"{REPOSITORY_ENDPOINT}/actions/permissions"
RULESETS_ENDPOINT = f"{REPOSITORY_ENDPOINT}/rulesets?includes_parents=false&per_page=100"
DEV_RULESET_ENDPOINT = f"{REPOSITORY_ENDPOINT}/rulesets/{{ruleset_id}}"
MAIN_RULESET_ENDPOINT = DEV_RULESET_ENDPOINT
DEV_REF_ENDPOINT = f"{REPOSITORY_ENDPOINT}/git/ref/heads/dev"
LABELS_ENDPOINT = f"{REPOSITORY_ENDPOINT}/labels"
LABELS_LIST_ENDPOINT = f"{LABELS_ENDPOINT}?per_page=100"
CHECK_RUNS_ENDPOINT = f"{REPOSITORY_ENDPOINT}/commits/{{sha}}/check-runs?filter=all&per_page=100"
DEFAULT_POLICY_PATH = Path(".github/settings/pr-policy.json")

MAX_RULESETS = 100
MAX_LABELS = 100
MAX_CHECK_RUNS = 100
API_TIMEOUT_SECONDS = 30
SHA_PATTERN = re.compile(r"[0-9a-f]{40}")
CANONICAL_AUTOMERGE_LABEL = "dependencies:automerge"
CODEQL_ZERO_ALERT_FLOOR: Final[dict[str, object]] = {
    "code_scanning_tools": [
        {
            "tool": "CodeQL",
            "alerts_threshold": "all",
            "security_alerts_threshold": "all",
        }
    ]
}
DEV_PUSH_ATTESTATION_MANIFEST: Final[tuple[CheckRequirement, ...]] = tuple(
    CheckRequirement(
        requirement.context,
        requirement.kind,
        frozenset({"success"}),
        requirement.producer,
    )
    for requirement in DEV_CHECK_MANIFEST
    if requirement.kind == "blocking"
)
MAIN_QUALIFICATION_ATTESTATION_MANIFEST: Final[tuple[CheckRequirement, ...]] = tuple(
    CheckRequirement(
        requirement.context,
        requirement.kind,
        frozenset(conclusion.lower() for conclusion in requirement.allowed_conclusions),
        requirement.producer,
    )
    for requirement in MAIN_QUALIFICATION_CHECK_MANIFEST
)
DEV_POLICY_ATTESTATION_MANIFEST: Final[tuple[CheckRequirement, ...]] = (
    *DEV_PUSH_ATTESTATION_MANIFEST,
    *MAIN_QUALIFICATION_ATTESTATION_MANIFEST,
)
_UNKNOWN = object()

RULESET_WRITABLE_FIELDS = (
    "name",
    "target",
    "enforcement",
    "bypass_actors",
    "conditions",
    "rules",
)
REPOSITORY_POLICY_FIELDS = (
    "allow_merge_commit",
    "allow_squash_merge",
    "allow_rebase_merge",
    "allow_auto_merge",
    "delete_branch_on_merge",
)
ACTIONS_PERMISSION_FIELDS = ("enabled", "allowed_actions", "sha_pinning_required")
LABEL_FIELDS = ("name", "color", "description")


class PolicyError(RuntimeError):
    """The requested policy transition is unsafe or unverifiable."""


class GitHubApiError(RuntimeError):
    """A GitHub CLI request failed or returned malformed JSON."""


class Api(Protocol):
    """Minimal GitHub API surface used by the transaction."""

    def get_json(self, endpoint: str) -> object: ...

    def send_json(self, method: str, endpoint: str, payload: dict[str, object]) -> object: ...


class GhApi:
    """GitHub API adapter implemented through the authenticated ``gh`` CLI."""

    @staticmethod
    def _run(args: list[str], input_text: str | None = None) -> str:
        try:
            result = subprocess.run(
                ["gh", "api", *args],
                input=input_text,
                capture_output=True,
                text=True,
                check=True,
                timeout=API_TIMEOUT_SECONDS,
            )
        except subprocess.TimeoutExpired as error:
            raise GitHubApiError(f"gh api timed out after {API_TIMEOUT_SECONDS} seconds") from error
        except subprocess.CalledProcessError as error:
            detail = error.stderr.strip() or error.stdout.strip() or f"exit {error.returncode}"
            raise GitHubApiError(f"gh api failed: {detail}") from error
        except OSError as error:
            raise GitHubApiError(f"gh api could not start: {error}") from error
        return result.stdout

    def get_json(self, endpoint: str) -> object:
        output = self._run([endpoint])
        try:
            return json.loads(output)
        except json.JSONDecodeError as error:
            raise GitHubApiError(f"gh api returned invalid JSON for {endpoint}") from error

    def send_json(self, method: str, endpoint: str, payload: dict[str, object]) -> object:
        args = ["--method", method, endpoint]
        input_text: str | None = None
        if method != "DELETE":
            args.extend(["--input", "-"])
            input_text = json.dumps(payload, sort_keys=True)
        output = self._run(args, input_text)
        if not output.strip():
            return {}
        try:
            return json.loads(output)
        except json.JSONDecodeError as error:
            raise GitHubApiError(f"gh api returned invalid JSON for {method} {endpoint}") from error


def _object(value: object, description: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise PolicyError(f"{description} must be a JSON object")
    return cast(dict[str, object], value)


def _objects(value: object, description: str, maximum: int) -> list[dict[str, object]]:
    if not isinstance(value, list):
        raise PolicyError(f"{description} must be a JSON array")
    if len(value) >= maximum:
        raise PolicyError(f"{description} reached the {maximum}-item safety bound")
    if not all(isinstance(item, dict) for item in value):
        raise PolicyError(f"{description} contains a non-object item")
    return cast(list[dict[str, object]], value)


def _select_fields(payload: dict[str, object], fields: tuple[str, ...]) -> dict[str, object]:
    missing = [field for field in fields if field not in payload]
    if missing:
        raise PolicyError(f"GitHub payload is missing required field(s): {missing}")
    return {field: deepcopy(payload[field]) for field in fields}


def normalize_ruleset(payload: dict[str, object]) -> dict[str, object]:
    """Return only fields accepted by GitHub's ruleset update endpoint."""
    normalized = _select_fields(payload, RULESET_WRITABLE_FIELDS)
    rules = _objects(normalized["rules"], "ruleset rules", MAX_RULESETS)
    rule_types = [rule.get("type") for rule in rules]
    if not all(isinstance(rule_type, str) for rule_type in rule_types):
        raise PolicyError("ruleset rule has no string type")
    for rule in rules:
        if rule.get("type") != "required_status_checks":
            continue
        parameters = _object(rule.get("parameters"), "status-check parameters")
        checks = _objects(
            parameters.get("required_status_checks"),
            "required checks",
            MAX_CHECK_RUNS,
        )
        for check in checks:
            context = check.get("context")
            if not isinstance(context, str) or not context:
                raise PolicyError("required status check has no context")
            integration_id = check.get("integration_id", _UNKNOWN)
            if (
                integration_id is not _UNKNOWN
                and integration_id is not None
                and (type(integration_id) is not int or integration_id <= 0)
            ):
                raise PolicyError(f"required status check {context!r} has malformed integration_id")
    normalized["rules"] = sorted(rules, key=lambda rule: str(rule["type"]))
    return normalized


def normalize_repository(payload: dict[str, object]) -> dict[str, object]:
    """Return only repository merge-policy fields owned by PER-264."""
    return _select_fields(payload, REPOSITORY_POLICY_FIELDS)


def normalize_actions_permissions(payload: dict[str, object]) -> dict[str, object]:
    """Return the repository Actions fields owned by PER-263."""
    return _select_fields(payload, ACTIONS_PERMISSION_FIELDS)


def normalize_label(payload: dict[str, object]) -> dict[str, object]:
    """Return the stable label fields owned by PER-264."""
    return _select_fields(payload, LABEL_FIELDS)


def _canonical_label(payload: dict[str, object], description: str) -> dict[str, object]:
    label = normalize_label(payload)
    if label["name"] != CANONICAL_AUTOMERGE_LABEL:
        raise PolicyError(f"{description} must use the canonical automerge label")
    return label


def label_endpoint(name: str) -> str:
    """Return the endpoint for one exact label name."""
    return f"{REPOSITORY_ENDPOINT}/labels/{quote(name, safe='')}"


def load_policy(path: Path = DEFAULT_POLICY_PATH) -> dict[str, object]:
    """Load and minimally validate the checked-in desired policy."""
    try:
        raw = json.loads(path.read_text())
    except OSError as error:
        raise PolicyError(f"cannot read policy {path}: {error}") from error
    except json.JSONDecodeError as error:
        raise PolicyError(f"policy {path} is not valid JSON") from error
    policy = _object(raw, "policy")
    _object(policy.get("repository"), "policy.repository")
    _object(policy.get("actions_permissions"), "policy.actions_permissions")
    _object(policy.get("dev_ruleset"), "policy.dev_ruleset")
    _object(policy.get("main_ruleset"), "policy.main_ruleset")
    _object(policy.get("automerge_label"), "policy.automerge_label")
    _validate_policy(policy)
    return policy


def _exact_branch_scope(payload: dict[str, object], branch: str) -> bool:
    if payload.get("target") != "branch":
        return False
    conditions = _object(payload.get("conditions"), f"{branch} ruleset conditions")
    ref_name = _object(conditions.get("ref_name"), f"{branch} ruleset ref condition")
    return ref_name.get("include") == [f"refs/heads/{branch}"] and ref_name.get("exclude") == []


def _exact_dev_scope(payload: dict[str, object]) -> bool:
    return _exact_branch_scope(payload, "dev")


def _exact_main_scope(payload: dict[str, object]) -> bool:
    return _exact_branch_scope(payload, "main")


def _protected_rulesets(api: Api) -> dict[str, tuple[int, dict[str, object]]]:
    summaries = _objects(api.get_json(RULESETS_ENDPOINT), "repository rulesets", MAX_RULESETS)
    matches: dict[str, list[tuple[int, dict[str, object]]]] = {"dev": [], "main": []}
    for summary in summaries:
        ruleset_id = summary.get("id")
        if not isinstance(ruleset_id, int):
            raise PolicyError("ruleset summary has no integer id")
        payload = _object(
            api.get_json(DEV_RULESET_ENDPOINT.format(ruleset_id=ruleset_id)),
            f"ruleset {ruleset_id}",
        )
        conditions = _object(payload.get("conditions"), f"ruleset {ruleset_id} conditions")
        ref_name = _object(conditions.get("ref_name"), f"ruleset {ruleset_id} ref condition")
        includes = ref_name.get("include")
        if not isinstance(includes, list):
            continue
        protected = [branch for branch in ("dev", "main") if f"refs/heads/{branch}" in includes]
        if len(protected) > 1:
            raise PolicyError(f"ruleset {ruleset_id} has shared protected-branch scope")
        for branch in ("dev", "main"):
            if f"refs/heads/{branch}" not in includes:
                continue
            if not _exact_branch_scope(payload, branch):
                raise PolicyError(
                    f"ruleset {ruleset_id} does not have exact {branch}-only branch scope"
                )
            matches[branch].append((ruleset_id, payload))
    resolved: dict[str, tuple[int, dict[str, object]]] = {}
    for branch in ("dev", "main"):
        if len(matches[branch]) != 1:
            raise PolicyError(
                f"expected exactly one {branch} ruleset, found {len(matches[branch])}"
            )
        resolved[branch] = matches[branch][0]
    return resolved


def _labels(api: Api) -> list[dict[str, object]]:
    return _objects(api.get_json(LABELS_LIST_ENDPOINT), "repository labels", MAX_LABELS)


def _find_label(labels: list[dict[str, object]], name: str) -> dict[str, object] | None:
    matches = [label for label in labels if label.get("name") == name]
    if len(matches) > 1:
        raise PolicyError(f"label {name!r} appears more than once")
    return matches[0] if matches else None


def _current_state(api: Api) -> dict[str, object]:
    rulesets = _protected_rulesets(api)
    dev_ruleset_id, dev_ruleset = rulesets["dev"]
    main_ruleset_id, main_ruleset = rulesets["main"]
    repository = _object(api.get_json(REPOSITORY_ENDPOINT), "repository")
    actions_permissions = _object(
        api.get_json(ACTIONS_PERMISSIONS_ENDPOINT),
        "GitHub Actions permissions",
    )
    labels = _labels(api)
    return {
        "dev_ruleset_id": dev_ruleset_id,
        "dev_ruleset": normalize_ruleset(dev_ruleset),
        "main_ruleset_id": main_ruleset_id,
        "main_ruleset": normalize_ruleset(main_ruleset),
        "repository": normalize_repository(repository),
        "actions_permissions": normalize_actions_permissions(actions_permissions),
        "automerge_label": _find_label(labels, CANONICAL_AUTOMERGE_LABEL),
    }


def _state_ruleset_identity(state: dict[str, object], branch: str) -> tuple[int, dict[str, object]]:
    ruleset_id = state.get(f"{branch}_ruleset_id")
    if not isinstance(ruleset_id, int):
        raise PolicyError(f"state has no integer {branch}_ruleset_id")
    ruleset = normalize_ruleset(_object(state.get(f"{branch}_ruleset"), f"state {branch} ruleset"))
    if not _exact_branch_scope(ruleset, branch):
        raise PolicyError(f"state ruleset does not have exact {branch}-only branch scope")
    return ruleset_id, ruleset


def _state_identity(state: dict[str, object]) -> dict[str, object]:
    dev_ruleset_id, dev_ruleset = _state_ruleset_identity(state, "dev")
    main_ruleset_id, main_ruleset = _state_ruleset_identity(state, "main")
    label = state.get("automerge_label")
    normalized_label = (
        None
        if label is None
        else _canonical_label(_object(label, "state automerge label"), "state label")
    )
    return {
        "dev_ruleset_id": dev_ruleset_id,
        "dev_ruleset": dev_ruleset,
        "main_ruleset_id": main_ruleset_id,
        "main_ruleset": main_ruleset,
        "repository": normalize_repository(_object(state.get("repository"), "state repository")),
        "actions_permissions": normalize_actions_permissions(
            _object(state.get("actions_permissions"), "state Actions permissions")
        ),
        "automerge_label": normalized_label,
    }


def check_policy(api: Api, policy: dict[str, object]) -> list[str]:
    """Return structural drift without mutating GitHub."""
    _validate_policy(policy)
    current = _current_state(api)
    desired_repository = normalize_repository(_object(policy["repository"], "repository policy"))
    desired_actions_permissions = normalize_actions_permissions(
        _object(policy["actions_permissions"], "Actions permissions policy")
    )
    desired_dev_ruleset = normalize_ruleset(_object(policy["dev_ruleset"], "dev ruleset policy"))
    desired_main_ruleset = normalize_ruleset(_object(policy["main_ruleset"], "main ruleset policy"))
    desired_label = _canonical_label(_object(policy["automerge_label"], "label policy"), "policy")
    drift: list[str] = []
    if current["repository"] != desired_repository:
        drift.append("repository merge settings differ")
    if current["actions_permissions"] != desired_actions_permissions:
        drift.append("GitHub Actions permissions differ")
    if current["dev_ruleset"] != desired_dev_ruleset:
        drift.append("dev ruleset differs")
    if current["main_ruleset"] != desired_main_ruleset:
        drift.append("main ruleset differs")
    existing_label = current["automerge_label"]
    if existing_label is None:
        drift.append("automerge label is absent")
    elif (
        _canonical_label(_object(existing_label, "current automerge label"), "current label")
        != desired_label
    ):
        drift.append("automerge label differs")
    return drift


def _dev_sha(api: Api) -> str:
    ref = _object(api.get_json(DEV_REF_ENDPOINT), "dev ref")
    target = _object(ref.get("object"), "dev ref object")
    sha = target.get("sha")
    if not isinstance(sha, str) or SHA_PATTERN.fullmatch(sha) is None:
        raise PolicyError("dev ref has no canonical 40-hex SHA")
    return sha


def _required_contexts(policy: dict[str, object], branch: str = "dev") -> list[str]:
    ruleset = _object(policy[f"{branch}_ruleset"], f"{branch} ruleset policy")
    rules = _objects(ruleset.get("rules"), f"{branch} rules", MAX_RULESETS)
    status_rules = [rule for rule in rules if rule.get("type") == "required_status_checks"]
    if len(status_rules) != 1:
        raise PolicyError(f"{branch} policy must contain exactly one required-status-checks rule")
    parameters = _object(status_rules[0].get("parameters"), "status-check parameters")
    checks = _objects(parameters.get("required_status_checks"), "required checks", MAX_CHECK_RUNS)
    if not checks:
        raise PolicyError(f"{branch} policy must contain at least one required status check")
    contexts = [check.get("context") for check in checks]
    if not all(isinstance(context, str) and context for context in contexts):
        raise PolicyError("required status check has no context")
    if len(set(contexts)) != len(contexts):
        raise PolicyError("required status check contexts must be unique")
    expected_producers = {
        requirement.context: requirement.producer.integration_id
        for requirement in manifest_for_base(branch)
        if requirement.kind == "blocking"
    }
    for check in checks:
        context = cast(str, check["context"])
        integration_id = check.get("integration_id")
        if type(integration_id) is not int:
            raise PolicyError(
                f"required status check {context!r} must declare an integer integration_id"
            )
        expected_integration_id = expected_producers.get(context)
        if expected_integration_id is not None and integration_id != expected_integration_id:
            raise PolicyError(
                f"required status check {context!r} integration_id must be "
                f"{expected_integration_id}"
            )
    return cast(list[str], contexts)


def _require_codeql_zero_alert_floor(ruleset: dict[str, object], branch: str) -> None:
    rules = _objects(ruleset.get("rules"), f"{branch} rules", MAX_RULESETS)
    code_scanning_rules = [rule for rule in rules if rule.get("type") == "code_scanning"]
    if len(code_scanning_rules) != 1:
        raise PolicyError(f"{branch} policy must contain exactly one CodeQL code-scanning rule")
    parameters = _object(
        code_scanning_rules[0].get("parameters"),
        f"{branch} CodeQL code-scanning parameters",
    )
    if parameters != CODEQL_ZERO_ALERT_FLOOR:
        raise PolicyError(f"{branch} CodeQL code-scanning rule must enforce the zero-alert floor")


def _validate_policy(policy: dict[str, object]) -> None:
    dev_ruleset = normalize_ruleset(_object(policy.get("dev_ruleset"), "dev ruleset policy"))
    if not _exact_dev_scope(dev_ruleset):
        raise PolicyError("desired ruleset must have exact dev-only branch scope")
    if any(
        rule.get("type") == "code_scanning"
        for rule in _objects(dev_ruleset["rules"], "dev rules", MAX_RULESETS)
    ):
        raise PolicyError("dev CodeQL scans must be asynchronous, not a code-scanning merge rule")
    dev_contexts = _required_contexts(policy, "dev")
    if len(dev_contexts) != len(DEV_BLOCKING_CONTEXTS) or set(dev_contexts) != set(
        DEV_BLOCKING_CONTEXTS
    ):
        raise PolicyError("required checks must equal the complete dev check manifest")
    main_ruleset = normalize_ruleset(_object(policy.get("main_ruleset"), "main ruleset policy"))
    if not _exact_main_scope(main_ruleset):
        raise PolicyError("desired ruleset must have exact main-only branch scope")
    _require_codeql_zero_alert_floor(main_ruleset, "main")
    main_contexts = _required_contexts(policy, "main")
    if len(main_contexts) != len(MAIN_BLOCKING_CONTEXTS) or set(main_contexts) != set(
        MAIN_BLOCKING_CONTEXTS
    ):
        raise PolicyError("required checks must equal the complete main check manifest")
    normalize_repository(_object(policy.get("repository"), "repository policy"))
    actions_permissions = normalize_actions_permissions(
        _object(policy.get("actions_permissions"), "Actions permissions policy")
    )
    if actions_permissions != {
        "enabled": True,
        "allowed_actions": "all",
        "sha_pinning_required": True,
    }:
        raise PolicyError(
            "Actions permissions must require immutable SHAs without disabling Actions"
        )
    _canonical_label(_object(policy.get("automerge_label"), "label policy"), "policy")


def _verify_green_dev(api: Api, expected_sha: str) -> None:
    actual_sha = _dev_sha(api)
    if actual_sha != expected_sha:
        raise PolicyError(f"dev moved: expected {expected_sha}, found {actual_sha}")
    payload = _object(
        api.get_json(CHECK_RUNS_ENDPOINT.format(sha=expected_sha)),
        "dev check runs",
    )
    total_count = payload.get("total_count")
    if type(total_count) is not int or total_count < 0 or total_count >= MAX_CHECK_RUNS:
        raise PolicyError(f"dev check runs reached the {MAX_CHECK_RUNS}-item safety bound")
    runs = _objects(payload.get("check_runs"), "dev check runs", MAX_CHECK_RUNS)
    if total_count != len(runs):
        raise PolicyError("dev check runs did not return one complete page")
    expected = {requirement.context: requirement for requirement in DEV_POLICY_ATTESTATION_MANIFEST}
    latest: dict[str, dict[str, object]] = {}
    seen_ids: dict[str, set[int]] = {}
    wrong_producer: set[str] = set()
    for candidate in runs:
        name = candidate.get("name")
        if not isinstance(name, str) or not name:
            raise PolicyError("check run has no name")
        run_id = candidate.get("id")
        if type(run_id) is not int or run_id <= 0:
            raise PolicyError(f"check run {name!r} has no positive integer id")
        started_at = candidate.get("started_at")
        if started_at is not None and not isinstance(started_at, str):
            raise PolicyError(f"check run {name!r} has a non-string started_at")
        requirement = expected.get(name)
        if requirement is None:
            continue
        head_sha = candidate.get("head_sha")
        if not isinstance(head_sha, str) or head_sha != expected_sha:
            raise PolicyError(
                f"check run {name!r} has wrong head: expected {expected_sha}, found {head_sha}"
            )
        app = _object(candidate.get("app"), f"check run {name!r} app")
        app_id = app.get("id")
        app_slug = app.get("slug")
        if type(app_id) is not int or not isinstance(app_slug, str) or not app_slug:
            raise PolicyError(f"check run {name!r} has malformed app identity")
        if app_id != requirement.producer.integration_id or app_slug != requirement.producer.slug:
            wrong_producer.add(name)
            continue
        context_ids = seen_ids.setdefault(name, set())
        if run_id in context_ids:
            raise PolicyError(f"check run {name!r} repeats id {run_id}")
        context_ids.add(run_id)
        prior = latest.get(name)
        prior_id = prior.get("id") if prior is not None else 0
        if type(prior_id) is not int:
            raise PolicyError(f"prior check run {name!r} has no integer id")
        # Check-run IDs are monotonic unique API identities; queued runs may have no started_at.
        if prior is None or run_id > prior_id:
            latest[name] = candidate
    problems: list[str] = []
    for requirement in DEV_POLICY_ATTESTATION_MANIFEST:
        selected = latest.get(requirement.context)
        if selected is None:
            if requirement.context in wrong_producer:
                problems.append(
                    f"{requirement.context}: missing required producer "
                    f"{requirement.producer.slug} ({requirement.producer.integration_id})"
                )
            else:
                problems.append(f"{requirement.context}: missing")
            continue
        status_value = selected.get("status")
        conclusion_value = selected.get("conclusion")
        if not isinstance(status_value, str) or not status_value:
            raise PolicyError(f"check run {requirement.context!r} has malformed status")
        if conclusion_value is not None and not isinstance(conclusion_value, str):
            raise PolicyError(f"check run {requirement.context!r} has malformed conclusion")
        status = status_value
        conclusion = conclusion_value or ""
        if status != "completed" or conclusion not in requirement.allowed_conclusions:
            problems.append(f"{requirement.context}: status={status}, conclusion={conclusion}")
    if problems:
        raise PolicyError("exact dev checks are not green: " + "; ".join(problems))


def _write_snapshot(path: Path, state: dict[str, object], dev_sha: str) -> None:
    snapshot = {"dev_sha": dev_sha, **deepcopy(state)}
    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        with path.open("x", encoding="utf-8") as snapshot_file:
            snapshot_file.write(json.dumps(snapshot, indent=2, sort_keys=True) + "\n")
            snapshot_file.flush()
            os.fsync(snapshot_file.fileno())
    except FileExistsError as error:
        raise PolicyError(f"snapshot path already exists: {path}") from error


def _label_update_payload(label: dict[str, object]) -> dict[str, object]:
    return {
        "new_name": label["name"],
        "color": label["color"],
        "description": label["description"],
    }


def _restore_label(api: Api, prior: object, current: object = _UNKNOWN) -> None:
    if prior is None:
        name = (
            str(
                _canonical_label(_object(current, "current automerge label"), "current label")[
                    "name"
                ]
            )
            if current is not _UNKNOWN and current is not None
            else CANONICAL_AUTOMERGE_LABEL
        )
        api.send_json("DELETE", label_endpoint(name), {"name": name})
        return
    prior_label = _canonical_label(_object(prior, "snapshot automerge label"), "snapshot label")
    if current is None:
        api.send_json("POST", LABELS_ENDPOINT, prior_label)
        return
    current_name = (
        str(_canonical_label(_object(current, "current automerge label"), "current label")["name"])
        if current is not _UNKNOWN
        else str(prior_label["name"])
    )
    api.send_json(
        "PATCH",
        label_endpoint(current_name),
        _label_update_payload(prior_label),
    )


def _component_differences(
    current: dict[str, object],
    desired: dict[str, object],
) -> tuple[bool, bool, bool, bool, bool]:
    current_identity = _state_identity(current)
    desired_identity = _state_identity(desired)
    return (
        current_identity["dev_ruleset"] != desired_identity["dev_ruleset"],
        current_identity["main_ruleset"] != desired_identity["main_ruleset"],
        current_identity["repository"] != desired_identity["repository"],
        current_identity["actions_permissions"] != desired_identity["actions_permissions"],
        current_identity["automerge_label"] != desired_identity["automerge_label"],
    )


def _restore_components(
    api: Api,
    before: dict[str, object],
    current: dict[str, object] | None,
    *,
    restore_dev_ruleset: bool,
    restore_main_ruleset: bool,
    restore_repository: bool,
    restore_actions_permissions: bool,
    restore_label: bool,
) -> list[str]:
    dev_ruleset_id, dev_ruleset = _state_ruleset_identity(before, "dev")
    main_ruleset_id, main_ruleset = _state_ruleset_identity(before, "main")
    repository = normalize_repository(_object(before.get("repository"), "snapshot repository"))
    actions_permissions = normalize_actions_permissions(
        _object(before.get("actions_permissions"), "snapshot Actions permissions")
    )
    current_label = current.get("automerge_label") if current is not None else _UNKNOWN
    errors: list[str] = []
    if restore_dev_ruleset:
        try:
            api.send_json(
                "PUT",
                DEV_RULESET_ENDPOINT.format(ruleset_id=dev_ruleset_id),
                dev_ruleset,
            )
        except (GitHubApiError, PolicyError, OSError) as error:
            errors.append(f"dev ruleset restore failed: {error}")
    if restore_main_ruleset:
        try:
            api.send_json(
                "PUT",
                MAIN_RULESET_ENDPOINT.format(ruleset_id=main_ruleset_id),
                main_ruleset,
            )
        except (GitHubApiError, PolicyError, OSError) as error:
            errors.append(f"main ruleset restore failed: {error}")
    if restore_repository:
        try:
            api.send_json("PATCH", REPOSITORY_ENDPOINT, repository)
        except (GitHubApiError, PolicyError, OSError) as error:
            errors.append(f"repository restore failed: {error}")
    if restore_actions_permissions:
        try:
            api.send_json("PUT", ACTIONS_PERMISSIONS_ENDPOINT, actions_permissions)
        except (GitHubApiError, PolicyError, OSError) as error:
            errors.append(f"GitHub Actions permissions restore failed: {error}")
    if restore_label:
        try:
            _restore_label(api, before.get("automerge_label"), current_label)
        except (GitHubApiError, PolicyError, OSError) as error:
            errors.append(f"label restore failed: {error}")
    return errors


def _restore_snapshot(
    api: Api,
    before: dict[str, object],
    *,
    attempted_dev_ruleset: bool,
    attempted_main_ruleset: bool,
    attempted_repository: bool,
    attempted_actions_permissions: bool,
    attempted_label: bool,
) -> None:
    _state_identity(before)
    current: dict[str, object] | None = None
    restore_flags = (
        attempted_dev_ruleset,
        attempted_main_ruleset,
        attempted_repository,
        attempted_actions_permissions,
        attempted_label,
    )
    try:
        current = _current_state(api)
    except (GitHubApiError, PolicyError, OSError):
        pass
    else:
        if current["dev_ruleset_id"] != before["dev_ruleset_id"]:
            raise PolicyError("dev ruleset ID changed; restore refused")
        if current["main_ruleset_id"] != before["main_ruleset_id"]:
            raise PolicyError("main ruleset ID changed; restore refused")
        restore_flags = _component_differences(current, before)
    errors = _restore_components(
        api,
        before,
        current,
        restore_dev_ruleset=restore_flags[0],
        restore_main_ruleset=restore_flags[1],
        restore_repository=restore_flags[2],
        restore_actions_permissions=restore_flags[3],
        restore_label=restore_flags[4],
    )
    try:
        restored = _state_identity(_current_state(api)) == _state_identity(before)
    except (GitHubApiError, PolicyError, OSError) as error:
        errors.append(f"rollback readback failed: {error}")
        restored = False
    if not restored:
        errors.append("rollback readback does not match snapshot")
    if errors and not restored:
        raise PolicyError("; ".join(errors))


def _apply_label(api: Api, desired: dict[str, object], current: object) -> None:
    if current is None:
        api.send_json("POST", LABELS_ENDPOINT, desired)
        return
    current_label = _canonical_label(_object(current, "current automerge label"), "current label")
    if current_label != desired:
        api.send_json(
            "PATCH",
            label_endpoint(str(current_label["name"])),
            _label_update_payload(desired),
        )


def _verify_migration_pr_checks(pr: int, head_sha: str) -> None:
    """Reuse the sanctioned verifier without attempting a merge."""
    try:
        result = subprocess.run(
            [
                sys.executable,
                str(Path(__file__).with_name("pr_merge.py")),
                str(pr),
                "--verify-only",
                "--expected-head",
                head_sha,
                "--require-ci-workflow",
            ],
            cwd=Path(__file__).resolve().parents[1],
            capture_output=True,
            text=True,
            check=False,
            timeout=180,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise PolicyError(f"migration PR verification could not complete: {error}") from error
    if result.returncode != 0:
        detail = (result.stderr.strip() or result.stdout.strip())[:2000]
        raise PolicyError(f"migration PR verification refused: {detail}")


def _verify_migration_refs(api: Api, pr: int, dev_sha: str, head_sha: str) -> None:
    """Bind the migration to a conflict-free, same-repository, exact-head dev PR."""
    if _dev_sha(api) != dev_sha:
        raise PolicyError("dev moved during policy migration")
    pull = _object(api.get_json(f"{REPOSITORY_ENDPOINT}/pulls/{pr}"), "migration PR")
    if pull.get("number") != pr or pull.get("state") != "open" or pull.get("draft") is not False:
        raise PolicyError("migration PR must be open and ready for review")
    if pull.get("mergeable") is not True:
        raise PolicyError("migration PR must have resolved mergeability without conflicts")
    for side, expected_sha in (("base", dev_sha), ("head", head_sha)):
        ref = _object(pull.get(side), f"migration PR {side}")
        repository = _object(ref.get("repo"), f"migration PR {side} repository")
        if repository.get("full_name") != REPOSITORY:
            raise PolicyError(f"migration PR {side} must belong to {REPOSITORY}")
        if ref.get("sha") != expected_sha:
            raise PolicyError(f"migration PR {side} moved from {expected_sha}")
        if side == "base" and ref.get("ref") != "dev":
            raise PolicyError("migration PR must target dev")


def _verify_migration_contents(
    api: Api, policy: dict[str, object], dev_sha: str, head_sha: str
) -> None:
    comparison = _object(
        api.get_json(f"{REPOSITORY_ENDPOINT}/compare/{dev_sha}...{head_sha}"),
        "migration ancestry",
    )
    merge_base = _object(comparison.get("merge_base_commit"), "migration merge base")
    if merge_base.get("sha") != dev_sha:
        raise PolicyError("migration PR must contain exact current dev ancestry")
    content = _object(
        api.get_json(
            f"{REPOSITORY_ENDPOINT}/contents/.github/settings/pr-policy.json?ref={head_sha}"
        ),
        "migration policy content",
    )
    encoded = content.get("content")
    if content.get("encoding") != "base64" or not isinstance(encoded, str):
        raise PolicyError("migration policy content must be complete base64")
    try:
        committed = json.loads(base64.b64decode("".join(encoded.split()), validate=True))
    except (binascii.Error, ValueError, UnicodeError) as error:
        raise PolicyError(f"migration policy content is invalid: {error}") from error
    if committed != policy:
        raise PolicyError("desired policy differs from the exact reviewed migration PR head")


def _verify_authorized_migration(before: dict[str, object], policy: dict[str, object]) -> None:
    """Allow check renames and the authorized exact dev-CodeQL gate removal only."""
    for component in ("repository", "actions_permissions"):
        if before[component] != policy[component]:
            raise PolicyError(
                "policy migration may change only required status checks "
                "or remove the exact dev CodeQL rule"
            )
    label = before["automerge_label"]
    if (
        label is None
        or normalize_label(_object(label, "current label")) != policy["automerge_label"]
    ):
        raise PolicyError(
            "policy migration may change only required status checks "
            "or remove the exact dev CodeQL rule"
        )
    for branch in ("dev", "main"):
        current = deepcopy(_object(before[f"{branch}_ruleset"], f"current {branch} ruleset"))
        desired = normalize_ruleset(
            _object(policy[f"{branch}_ruleset"], f"desired {branch} ruleset")
        )
        if branch == "dev":
            rules = _objects(current["rules"], "current dev rules", MAX_RULESETS)
            if any(rule.get("type") == "code_scanning" for rule in rules):
                _require_codeql_zero_alert_floor(current, "dev")
                current["rules"] = [rule for rule in rules if rule.get("type") != "code_scanning"]
        for ruleset in (current, desired):
            for rule in _objects(ruleset["rules"], f"{branch} rules", MAX_RULESETS):
                if rule.get("type") == "required_status_checks":
                    parameters = _object(rule.get("parameters"), f"{branch} check parameters")
                    parameters["required_status_checks"] = []
        if current != desired:
            raise PolicyError(
                "policy migration may change only required status checks "
                "or remove the exact dev CodeQL rule"
            )


def apply_policy(
    api: Api,
    policy: dict[str, object],
    expected_dev_sha: str,
    snapshot_path: Path,
    *,
    migration_pr: int | None = None,
    expected_pr_head: str | None = None,
) -> None:
    """Apply one coherent policy transaction and roll it back on any mismatch."""
    if SHA_PATTERN.fullmatch(expected_dev_sha) is None:
        raise PolicyError("--expected-dev-sha must be one canonical 40-hex SHA")
    _validate_policy(policy)
    if migration_pr is None:
        if expected_pr_head is not None:
            raise PolicyError("--expected-pr-head requires --migration-pr")
        _verify_green_dev(api, expected_dev_sha)
    else:
        if type(migration_pr) is not int or migration_pr <= 0:
            raise PolicyError("--migration-pr must be one positive PR number")
        if expected_pr_head is None or SHA_PATTERN.fullmatch(expected_pr_head) is None:
            raise PolicyError("--migration-pr requires one canonical --expected-pr-head")
        _verify_migration_refs(api, migration_pr, expected_dev_sha, expected_pr_head)
        _verify_migration_contents(api, policy, expected_dev_sha, expected_pr_head)
        _verify_migration_pr_checks(migration_pr, expected_pr_head)
    before = _current_state(api)
    if migration_pr is not None:
        _verify_authorized_migration(before, policy)
    _write_snapshot(snapshot_path, before, expected_dev_sha)
    if migration_pr is not None and expected_pr_head is not None:
        _verify_migration_pr_checks(migration_pr, expected_pr_head)
    if _state_identity(_current_state(api)) != _state_identity(before):
        raise PolicyError("GitHub settings changed after snapshot; no mutation was attempted")
    if _dev_sha(api) != expected_dev_sha:
        raise PolicyError("dev moved after green evidence; no mutation was attempted")
    if migration_pr is not None and expected_pr_head is not None:
        _verify_migration_refs(api, migration_pr, expected_dev_sha, expected_pr_head)

    desired_repository = normalize_repository(_object(policy["repository"], "repository policy"))
    desired_actions_permissions = normalize_actions_permissions(
        _object(policy["actions_permissions"], "Actions permissions policy")
    )
    desired_dev_ruleset = normalize_ruleset(_object(policy["dev_ruleset"], "dev ruleset policy"))
    desired_main_ruleset = normalize_ruleset(_object(policy["main_ruleset"], "main ruleset policy"))
    desired_label = _canonical_label(_object(policy["automerge_label"], "label policy"), "policy")
    dev_ruleset_id, _dev_ruleset = _state_ruleset_identity(before, "dev")
    main_ruleset_id, _main_ruleset = _state_ruleset_identity(before, "main")

    current_label = before["automerge_label"]
    label_differs = (
        current_label is None
        or _canonical_label(_object(current_label, "current automerge label"), "current label")
        != desired_label
    )
    repository_differs = before["repository"] != desired_repository
    actions_permissions_differ = before["actions_permissions"] != desired_actions_permissions
    dev_ruleset_differs = before["dev_ruleset"] != desired_dev_ruleset
    main_ruleset_differs = before["main_ruleset"] != desired_main_ruleset
    label_attempted = False
    repository_attempted = False
    actions_permissions_attempted = False
    dev_ruleset_attempted = False
    main_ruleset_attempted = False

    try:
        if label_differs:
            label_attempted = True
            _apply_label(api, desired_label, current_label)
        if repository_differs:
            repository_attempted = True
            api.send_json("PATCH", REPOSITORY_ENDPOINT, desired_repository)
        if dev_ruleset_differs:
            dev_ruleset_attempted = True
            api.send_json(
                "PUT",
                DEV_RULESET_ENDPOINT.format(ruleset_id=dev_ruleset_id),
                desired_dev_ruleset,
            )
        if main_ruleset_differs:
            main_ruleset_attempted = True
            api.send_json(
                "PUT",
                MAIN_RULESET_ENDPOINT.format(ruleset_id=main_ruleset_id),
                desired_main_ruleset,
            )
        if actions_permissions_differ:
            actions_permissions_attempted = True
            api.send_json("PUT", ACTIONS_PERMISSIONS_ENDPOINT, desired_actions_permissions)
        drift = check_policy(api, policy)
        if drift:
            raise PolicyError("GitHub policy readback mismatch: " + "; ".join(drift))
        if _dev_sha(api) != expected_dev_sha:
            raise PolicyError("dev moved during policy apply")
        if migration_pr is not None and expected_pr_head is not None:
            _verify_migration_refs(api, migration_pr, expected_dev_sha, expected_pr_head)
    except (GitHubApiError, PolicyError, OSError) as primary_error:
        try:
            _restore_snapshot(
                api,
                before,
                attempted_dev_ruleset=dev_ruleset_attempted,
                attempted_main_ruleset=main_ruleset_attempted,
                attempted_repository=repository_attempted,
                attempted_actions_permissions=actions_permissions_attempted,
                attempted_label=label_attempted,
            )
        except (GitHubApiError, PolicyError, OSError) as rollback_error:
            raise PolicyError(
                f"policy apply failed ({primary_error}); rollback also failed ({rollback_error})"
            ) from rollback_error
        raise


def load_snapshot(path: Path) -> dict[str, object]:
    """Load a previously retained transaction snapshot."""
    try:
        payload = json.loads(path.read_text())
    except OSError as error:
        raise PolicyError(f"cannot read snapshot {path}: {error}") from error
    except json.JSONDecodeError as error:
        raise PolicyError(f"snapshot {path} is not valid JSON") from error
    return _object(payload, "snapshot")


def rollback_policy(api: Api, snapshot: dict[str, object]) -> None:
    """Restore one snapshot only while its exact dev evidence is still current."""
    owns_actions_permissions = "actions_permissions" in snapshot
    validation_snapshot = dict(snapshot)
    if not owns_actions_permissions:
        validation_snapshot["actions_permissions"] = {
            "enabled": True,
            "allowed_actions": "all",
            "sha_pinning_required": True,
        }
    validated_identity = _state_identity(validation_snapshot)

    expected_dev_sha = snapshot.get("dev_sha")
    if not isinstance(expected_dev_sha, str) or SHA_PATTERN.fullmatch(expected_dev_sha) is None:
        raise PolicyError("snapshot has no canonical dev SHA")
    if _dev_sha(api) != expected_dev_sha:
        raise PolicyError("snapshot dev SHA is stale; rollback refused")

    rollback_snapshot = dict(snapshot)
    if not owns_actions_permissions:
        current = _current_state(api)
        rollback_snapshot["actions_permissions"] = current["actions_permissions"]
        desired = _state_identity(rollback_snapshot)
    else:
        desired = validated_identity

    _restore_snapshot(
        api,
        rollback_snapshot,
        attempted_dev_ruleset=True,
        attempted_main_ruleset=True,
        attempted_repository=True,
        attempted_actions_permissions=owns_actions_permissions,
        attempted_label=True,
    )
    if _state_identity(_current_state(api)) != desired:
        raise PolicyError("manual rollback readback mismatch")
    if _dev_sha(api) != expected_dev_sha:
        raise PolicyError("dev moved during manual rollback")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--apply", action="store_true")
    mode.add_argument("--rollback", type=Path)
    parser.add_argument("--policy", type=Path, default=DEFAULT_POLICY_PATH)
    parser.add_argument("--expected-dev-sha")
    parser.add_argument("--snapshot", type=Path)
    parser.add_argument(
        "--migration-pr",
        type=int,
        help="Reviewed dev PR replacing checks or removing the exact dev CodeQL merge rule",
    )
    parser.add_argument("--expected-pr-head", help="Exact reviewed migration PR head SHA")
    args = parser.parse_args()
    if (args.migration_pr is not None or args.expected_pr_head is not None) and not args.apply:
        parser.error("migration evidence is valid only with --apply")

    api = GhApi()
    try:
        if args.rollback is not None:
            rollback_policy(api, load_snapshot(args.rollback))
            print(f"github-pr-policy: restored {args.rollback}")
            return 0
        policy = load_policy(args.policy)
        if args.check:
            drift = check_policy(api, policy)
            for problem in drift:
                print(f"github-pr-policy: DRIFT — {problem}")
            return 1 if drift else 0
        if args.expected_dev_sha is None or args.snapshot is None:
            parser.error("--apply requires --expected-dev-sha and --snapshot")
        apply_policy(
            api,
            policy,
            args.expected_dev_sha,
            args.snapshot,
            migration_pr=args.migration_pr,
            expected_pr_head=args.expected_pr_head,
        )
        print(f"github-pr-policy: applied at dev {args.expected_dev_sha}")
        print(f"github-pr-policy: rollback snapshot retained at {args.snapshot}")
        return 0
    except (GitHubApiError, PolicyError, OSError) as error:
        print(f"github-pr-policy: REFUSED — {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
