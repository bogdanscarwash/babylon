"""Transactional contracts for the GitHub pull-request policy applicator."""

from __future__ import annotations

import base64
import importlib.util
import json
import re
import subprocess
from copy import deepcopy
from pathlib import Path
from typing import Any

import pytest

_SPEC = importlib.util.spec_from_file_location(
    "sync_github_pr_policy",
    Path(__file__).resolve().parents[3] / "tools" / "sync_github_pr_policy.py",
)
if _SPEC is None or _SPEC.loader is None:
    raise RuntimeError("tools/sync_github_pr_policy.py failed import-spec resolution")
policy_tool = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(policy_tool)

POLICY_PATH = Path(__file__).resolve().parents[3] / ".github" / "settings" / "pr-policy.json"

DEV_BLOCKING_CHECKS = ("CI Gate",)
GITHUB_ACTIONS_APP_ID = 15368
GITHUB_ACTIONS_APP_SLUG = "github-actions"
CODEQL_PROTECTION = {
    "code_scanning_tools": [
        {
            "tool": "CodeQL",
            "alerts_threshold": "all",
            "security_alerts_threshold": "all",
        }
    ]
}

MAIN_QUALIFICATION_CHECKS = (
    "Main Qualification / Event Contract",
    "Main Qualification / Non-Unit Behavioral Contracts",
    "Main Qualification / PostgreSQL Determinism Bundle",
    "Main Qualification / Reference-Data Contracts",
    "Main Qualification / Release Documentation",
    "Main Qualification / Container Image Scan",
    "Main Qualification / Native Download / Linux x86_64",
)


def _ruleset(
    *,
    strict: bool,
    threads: bool,
    branch: str = "dev",
    ruleset_id: int = 18807584,
) -> dict[str, Any]:
    contexts = (
        DEV_BLOCKING_CHECKS
        if branch == "dev"
        else (*DEV_BLOCKING_CHECKS, *MAIN_QUALIFICATION_CHECKS)
    )
    return {
        "id": ruleset_id,
        "name": f"{branch} protection",
        "target": "branch",
        "source_type": "Repository",
        "source": "percy-raskova/babylon",
        "enforcement": "active",
        "conditions": {"ref_name": {"exclude": [], "include": [f"refs/heads/{branch}"]}},
        "bypass_actors": [],
        "rules": [
            {"type": "deletion"},
            {"type": "non_fast_forward"},
            {
                "type": "pull_request",
                "parameters": {
                    "allowed_merge_methods": ["merge"],
                    "dismiss_stale_reviews_on_push": False,
                    "required_reviewers": [],
                    "require_code_owner_review": False,
                    "require_last_push_approval": False,
                    "required_approving_review_count": 0,
                    "required_review_thread_resolution": threads,
                    "require_extra_approval_for_unattributed_changes": False,
                },
            },
            {
                "type": "required_status_checks",
                "parameters": {
                    "strict_required_status_checks_policy": strict,
                    "do_not_enforce_on_create": False,
                    "required_status_checks": [
                        {
                            "context": context,
                            "integration_id": GITHUB_ACTIONS_APP_ID,
                        }
                        for context in contexts
                    ],
                },
            },
            {
                "type": "code_scanning",
                "parameters": deepcopy(CODEQL_PROTECTION),
            },
        ],
        "node_id": "read-only",
        "created_at": "yesterday",
        "updated_at": "today",
    }


def _repository(*, merge_only: bool) -> dict[str, Any]:
    return {
        "allow_merge_commit": True,
        "allow_squash_merge": not merge_only,
        "allow_rebase_merge": not merge_only,
        "allow_auto_merge": not merge_only,
        "delete_branch_on_merge": False,
        "id": 123,
        "full_name": "percy-raskova/babylon",
    }


def _actions_permissions(*, sha_pinning_required: bool) -> dict[str, Any]:
    return {
        "enabled": True,
        "allowed_actions": "all",
        "sha_pinning_required": sha_pinning_required,
        "selected_actions_url": "read-only",
    }


def _policy() -> dict[str, Any]:
    policy = {
        "repository": policy_tool.normalize_repository(_repository(merge_only=True)),
        "actions_permissions": policy_tool.normalize_actions_permissions(
            _actions_permissions(sha_pinning_required=True)
        ),
        "dev_ruleset": policy_tool.normalize_ruleset(_ruleset(strict=True, threads=True)),
        "main_ruleset": policy_tool.normalize_ruleset(
            _ruleset(
                strict=True,
                threads=True,
                branch="main",
                ruleset_id=18807583,
            )
        ),
        "automerge_label": {
            "name": "dependencies:automerge",
            "color": "1f883d",
            "description": "Patch/minor Dependabot update eligible for exact-head merge",
        },
    }

    policy["dev_ruleset"]["rules"] = [
        rule for rule in policy["dev_ruleset"]["rules"] if rule["type"] != "code_scanning"
    ]
    return policy


class FakeApi:
    """Small bounded API double with explicit state and injected failures."""

    def __init__(self) -> None:
        self.dev_sha = "a" * 40
        self.ruleset = _ruleset(strict=False, threads=False)
        self.main_ruleset = _ruleset(
            strict=False,
            threads=False,
            branch="main",
            ruleset_id=18807583,
        )
        self.repository = _repository(merge_only=False)
        self.actions_permissions = _actions_permissions(sha_pinning_required=False)
        self.labels: list[dict[str, Any]] = []
        self.check_runs = [
            {
                "id": index,
                "name": context,
                "head_sha": self.dev_sha,
                "status": "completed",
                "conclusion": "success",
                "started_at": "2026-08-25T00:00:00Z",
                "app": {
                    "id": GITHUB_ACTIONS_APP_ID,
                    "slug": GITHUB_ACTIONS_APP_SLUG,
                },
            }
            for index, context in enumerate(DEV_BLOCKING_CHECKS, start=1)
        ]
        self.check_runs.extend(
            {
                "id": index,
                "name": context,
                "head_sha": self.dev_sha,
                "status": "completed",
                "conclusion": "success",
                "started_at": "2026-08-26T00:00:00Z",
                "app": {
                    "id": GITHUB_ACTIONS_APP_ID,
                    "slug": GITHUB_ACTIONS_APP_SLUG,
                },
            }
            for index, context in enumerate(MAIN_QUALIFICATION_CHECKS, start=50)
        )
        self.calls: list[tuple[str, str, dict[str, Any] | None]] = []
        self.fail_method_endpoint: tuple[str, str] | None = None
        self.fail_oserror_method_endpoint: tuple[str, str] | None = None
        self.mutate_then_fail_method_endpoint: tuple[str, str] | None = None
        self.fail_ruleset_restore = False
        self.mismatch_ruleset_readback = False
        self.mismatch_ruleset_producer_readback = False
        self.concurrent_ruleset_drift = False
        self.policy_error_ruleset_read: int | None = None
        self._ruleset_reads = 0
        self.move_dev_on_read: int | None = None
        self._dev_reads = 0
        self.check_runs_total_count: int | None = None

    def get_json(self, endpoint: str) -> object:
        self.calls.append(("GET", endpoint, None))
        if endpoint == policy_tool.DEV_REF_ENDPOINT:
            self._dev_reads += 1
            if self._dev_reads == self.move_dev_on_read:
                self.dev_sha = "b" * 40
            return {"object": {"sha": self.dev_sha}}
        if endpoint == policy_tool.RULESETS_ENDPOINT:
            return [{"id": 18807584}, {"id": 18807583}]
        if endpoint == policy_tool.DEV_RULESET_ENDPOINT.format(ruleset_id=18807584):
            self._ruleset_reads += 1
            if self._ruleset_reads == self.policy_error_ruleset_read:
                raise policy_tool.PolicyError("injected malformed ruleset readback")
            value = deepcopy(self.ruleset)
            if self.concurrent_ruleset_drift and self._ruleset_reads == 2:
                value["name"] = "concurrent change"
            if self.mismatch_ruleset_readback and self._ruleset_reads == 3:
                value["enforcement"] = "disabled"
            if self.mismatch_ruleset_producer_readback and self._ruleset_reads == 3:
                status_rule = next(
                    rule for rule in value["rules"] if rule["type"] == "required_status_checks"
                )
                del status_rule["parameters"]["required_status_checks"][0]["integration_id"]
            return value
        if endpoint == policy_tool.MAIN_RULESET_ENDPOINT.format(ruleset_id=18807583):
            return deepcopy(self.main_ruleset)
        if endpoint == policy_tool.REPOSITORY_ENDPOINT:
            return deepcopy(self.repository)
        if endpoint == policy_tool.ACTIONS_PERMISSIONS_ENDPOINT:
            return deepcopy(self.actions_permissions)
        if endpoint == policy_tool.LABELS_LIST_ENDPOINT:
            return deepcopy(self.labels)
        if endpoint == policy_tool.CHECK_RUNS_ENDPOINT.format(sha=self.dev_sha):
            return {
                "total_count": (
                    len(self.check_runs)
                    if self.check_runs_total_count is None
                    else self.check_runs_total_count
                ),
                "check_runs": deepcopy(self.check_runs),
            }
        raise AssertionError(f"unexpected GET {endpoint}")

    def send_json(self, method: str, endpoint: str, payload: dict[str, Any]) -> object:
        self.calls.append((method, endpoint, deepcopy(payload)))
        if self.fail_method_endpoint == (method, endpoint):
            self.fail_method_endpoint = None
            raise policy_tool.GitHubApiError(f"injected {method} failure")
        if self.fail_oserror_method_endpoint == (method, endpoint):
            self.fail_oserror_method_endpoint = None
            raise OSError(f"injected {method} spawn failure")
        if (
            self.fail_ruleset_restore
            and method == "PUT"
            and endpoint == policy_tool.DEV_RULESET_ENDPOINT.format(ruleset_id=18807584)
            and payload["rules"][-1]["parameters"]["strict_required_status_checks_policy"] is False
        ):
            raise policy_tool.GitHubApiError("injected ruleset restore failure")
        if method == "POST" and endpoint == policy_tool.LABELS_ENDPOINT:
            label = {"id": 99, **payload}
            self.labels.append(label)
            return self._write_result(method, endpoint, label)
        if method == "DELETE" and endpoint == policy_tool.label_endpoint("dependencies:automerge"):
            self.labels = [label for label in self.labels if label["name"] != payload["name"]]
            return self._write_result(method, endpoint, {})
        if method == "PATCH" and endpoint == policy_tool.REPOSITORY_ENDPOINT:
            self.repository.update(payload)
            return self._write_result(method, endpoint, self.repository)
        if method == "PUT" and endpoint == policy_tool.ACTIONS_PERMISSIONS_ENDPOINT:
            self.actions_permissions.update(payload)
            return self._write_result(method, endpoint, self.actions_permissions)
        if method == "PUT" and endpoint == policy_tool.DEV_RULESET_ENDPOINT.format(
            ruleset_id=18807584
        ):
            current_id = self.ruleset["id"]
            self.ruleset = {"id": current_id, **deepcopy(payload)}
            return self._write_result(method, endpoint, self.ruleset)
        if method == "PUT" and endpoint == policy_tool.MAIN_RULESET_ENDPOINT.format(
            ruleset_id=18807583
        ):
            current_id = self.main_ruleset["id"]
            self.main_ruleset = {"id": current_id, **deepcopy(payload)}
            return self._write_result(method, endpoint, self.main_ruleset)
        raise AssertionError(f"unexpected {method} {endpoint}")

    def _write_result(
        self,
        method: str,
        endpoint: str,
        result: dict[str, Any],
    ) -> dict[str, Any]:
        if self.mutate_then_fail_method_endpoint == (method, endpoint):
            self.mutate_then_fail_method_endpoint = None
            raise policy_tool.GitHubApiError(f"injected ambiguous {method} failure")
        return deepcopy(result)


class MigrationApi(FakeApi):
    """A protected old policy and one reviewed replacement-workflow PR."""

    def __init__(self) -> None:
        super().__init__()
        desired = _policy()
        self.ruleset = {"id": 18807584, **deepcopy(desired["dev_ruleset"])}
        self.ruleset["rules"].insert(
            0, {"type": "code_scanning", "parameters": deepcopy(CODEQL_PROTECTION)}
        )
        self.main_ruleset = {"id": 18807583, **deepcopy(desired["main_ruleset"])}
        self.repository = deepcopy(desired["repository"])
        self.actions_permissions = deepcopy(desired["actions_permissions"])
        self.labels = [deepcopy(desired["automerge_label"])]
        for ruleset in (self.ruleset, self.main_ruleset):
            status = next(r for r in ruleset["rules"] if r["type"] == "required_status_checks")
            status["parameters"]["required_status_checks"][0]["context"] = "Retiring Gate"
        self.head_sha = "c" * 40
        self.pr = {
            "number": 912,
            "state": "open",
            "draft": False,
            "mergeable": True,
            "head": {"sha": self.head_sha, "repo": {"full_name": policy_tool.REPOSITORY}},
            "base": {
                "sha": self.dev_sha,
                "ref": "dev",
                "repo": {"full_name": policy_tool.REPOSITORY},
            },
        }
        self.committed_policy = deepcopy(desired)
        self.merge_base = self.dev_sha
        self.move_pr_on_read: int | None = None
        self.pr_reads = 0
        self.check_runs = []  # Old dev cannot attest a workflow it has not merged.

    def get_json(self, endpoint: str) -> object:
        if endpoint.endswith("/pulls/912"):
            self.calls.append(("GET", endpoint, None))
            self.pr_reads += 1
            if self.pr_reads == self.move_pr_on_read:
                self.pr["head"]["sha"] = "d" * 40
            return deepcopy(self.pr)
        if "/compare/" in endpoint:
            self.calls.append(("GET", endpoint, None))
            return {"merge_base_commit": {"sha": self.merge_base}}
        if "/contents/.github/settings/pr-policy.json?ref=" in endpoint:
            self.calls.append(("GET", endpoint, None))
            return {
                "encoding": "base64",
                "content": base64.b64encode(json.dumps(self.committed_policy).encode()).decode(),
            }
        return super().get_json(endpoint)


def _migration_apply(api: MigrationApi, tmp_path: Path) -> None:
    policy_tool.apply_policy(
        api,
        _policy(),
        "a" * 40,
        tmp_path / "before.json",
        migration_pr=912,
        expected_pr_head=api.head_sha,
    )


def test_policy_migration_uses_reviewed_pr_without_unmerged_dev_attestations(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    api = MigrationApi()
    api.labels[0]["id"] = 99
    verified: list[tuple[int, str]] = []
    monkeypatch.setattr(
        policy_tool, "_verify_migration_pr_checks", lambda pr, sha: verified.append((pr, sha))
    )

    _migration_apply(api, tmp_path)

    assert verified == [(912, api.head_sha), (912, api.head_sha)]
    assert policy_tool.check_policy(api, _policy()) == []
    assert {method for method, _, _ in api.calls if method != "GET"} == {"PUT"}
    assert json.loads((tmp_path / "before.json").read_text())["dev_sha"] == "a" * 40


@pytest.mark.parametrize("failure_at", [1, 2])
def test_policy_migration_cannot_write_after_pr_verification_refuses(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, failure_at: int
) -> None:
    api = MigrationApi()
    calls = 0

    def refuse(_pr: int, _sha: str) -> None:
        nonlocal calls
        calls += 1
        if calls == failure_at:
            raise policy_tool.PolicyError("CI failed or review unresolved")

    monkeypatch.setattr(policy_tool, "_verify_migration_pr_checks", refuse)
    with pytest.raises(policy_tool.PolicyError, match="CI failed or review unresolved"):
        _migration_apply(api, tmp_path)
    assert all(method == "GET" for method, _, _ in api.calls)


def test_policy_migration_checks_concurrent_settings_after_slow_pr_verification(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    api = MigrationApi()
    calls = 0

    def concurrent_change(_pr: int, _sha: str) -> None:
        nonlocal calls
        calls += 1
        if calls == 2:
            api.ruleset["name"] = "another operator changed settings"

    monkeypatch.setattr(policy_tool, "_verify_migration_pr_checks", concurrent_change)
    with pytest.raises(policy_tool.PolicyError, match="settings changed after snapshot"):
        _migration_apply(api, tmp_path)
    assert all(method == "GET" for method, _, _ in api.calls)


@pytest.mark.parametrize(
    "fault",
    ["fork", "closed", "draft", "conflict", "wrong-base", "wrong-head", "ancestry", "policy"],
)
def test_policy_migration_refuses_unreviewed_identity_before_mutation(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, fault: str
) -> None:
    api = MigrationApi()
    monkeypatch.setattr(policy_tool, "_verify_migration_pr_checks", lambda *_: None)
    if fault == "fork":
        api.pr["head"]["repo"]["full_name"] = "elsewhere/babylon"
    elif fault == "closed":
        api.pr["state"] = "closed"
    elif fault == "draft":
        api.pr["draft"] = True
    elif fault == "conflict":
        api.pr["mergeable"] = False
    elif fault == "wrong-base":
        api.pr["base"]["sha"] = "d" * 40
    elif fault == "wrong-head":
        api.pr["head"]["sha"] = "d" * 40
    elif fault == "ancestry":
        api.merge_base = "d" * 40
    else:
        api.committed_policy["repository"]["allow_auto_merge"] = True

    with pytest.raises(policy_tool.PolicyError):
        _migration_apply(api, tmp_path)

    assert all(method == "GET" for method, _, _ in api.calls)


@pytest.mark.parametrize("component", ["ruleset", "repository", "actions_permissions", "label"])
def test_policy_migration_cannot_change_other_protections(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, component: str
) -> None:
    api = MigrationApi()
    monkeypatch.setattr(policy_tool, "_verify_migration_pr_checks", lambda *_: None)
    if component == "ruleset":
        api.ruleset["bypass_actors"] = [
            {"actor_id": 1, "actor_type": "Team", "bypass_mode": "always"}
        ]
    elif component == "repository":
        api.repository["allow_squash_merge"] = True
    elif component == "actions_permissions":
        api.actions_permissions["sha_pinning_required"] = False
    else:
        api.labels = []

    with pytest.raises(policy_tool.PolicyError, match="only required status checks"):
        _migration_apply(api, tmp_path)

    assert all(method == "GET" for method, _, _ in api.calls)


@pytest.mark.parametrize("read", [2, 3])
def test_policy_migration_rechecks_head_and_rolls_back_if_it_moves(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, read: int
) -> None:
    api = MigrationApi()
    before = deepcopy(api.ruleset), deepcopy(api.main_ruleset)
    api.move_pr_on_read = read
    monkeypatch.setattr(policy_tool, "_verify_migration_pr_checks", lambda *_: None)

    with pytest.raises(policy_tool.PolicyError, match="head"):
        _migration_apply(api, tmp_path)

    assert (api.ruleset, api.main_ruleset) == before
    if read == 2:
        assert all(method == "GET" for method, _, _ in api.calls)


def test_policy_migration_reuses_non_mutating_exact_head_pr_verifier(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls: list[tuple[object, dict[str, Any]]] = []

    def failed(args: object, **kwargs: Any) -> subprocess.CompletedProcess[str]:
        calls.append((args, kwargs))
        return subprocess.CompletedProcess(args, 1, "", "unresolved review thread")

    monkeypatch.setattr(policy_tool.subprocess, "run", failed)
    with pytest.raises(policy_tool.PolicyError, match="unresolved review thread"):
        policy_tool._verify_migration_pr_checks(912, "c" * 40)
    command, options = calls[0]
    assert command[-5:] == [
        "912",
        "--verify-only",
        "--expected-head",
        "c" * 40,
        "--require-ci-workflow",
    ]
    assert options["timeout"] > 0


def test_normalizers_strip_read_only_api_fields() -> None:
    normalized_ruleset = policy_tool.normalize_ruleset(_ruleset(strict=False, threads=False))
    normalized_repository = policy_tool.normalize_repository(_repository(merge_only=False))
    normalized_actions = policy_tool.normalize_actions_permissions(
        _actions_permissions(sha_pinning_required=False)
    )

    assert set(normalized_ruleset) == {
        "name",
        "target",
        "enforcement",
        "bypass_actors",
        "conditions",
        "rules",
    }
    assert set(normalized_repository) == {
        "allow_merge_commit",
        "allow_squash_merge",
        "allow_rebase_merge",
        "allow_auto_merge",
        "delete_branch_on_merge",
    }
    assert normalized_actions == {
        "enabled": True,
        "allowed_actions": "all",
        "sha_pinning_required": False,
    }


def test_ruleset_normalizer_ignores_github_rule_order() -> None:
    desired = _ruleset(strict=True, threads=True)
    github_readback = deepcopy(desired)
    github_readback["rules"][-2:] = reversed(github_readback["rules"][-2:])

    assert policy_tool.normalize_ruleset(github_readback) == policy_tool.normalize_ruleset(desired)


def test_check_reports_drift_without_mutation() -> None:
    api = FakeApi()

    drift = policy_tool.check_policy(api, _policy())

    assert drift == [
        "repository merge settings differ",
        "GitHub Actions permissions differ",
        "dev ruleset differs",
        "main ruleset differs",
        "automerge label is absent",
    ]
    assert all(method == "GET" for method, _endpoint, _payload in api.calls)


def test_dev_ruleset_selection_refuses_a_shared_or_non_branch_scope() -> None:
    shared = FakeApi()
    shared.ruleset["conditions"]["ref_name"]["include"].append("refs/heads/main")
    with pytest.raises(policy_tool.PolicyError, match="shared protected-branch scope"):
        policy_tool.check_policy(shared, _policy())

    tagged = FakeApi()
    tagged.ruleset["target"] = "tag"
    with pytest.raises(policy_tool.PolicyError, match="exact dev-only branch scope"):
        policy_tool.check_policy(tagged, _policy())


def test_main_ruleset_selection_refuses_a_shared_scope() -> None:
    shared = FakeApi()
    shared.main_ruleset["conditions"]["ref_name"]["include"].append("refs/heads/dev")

    with pytest.raises(policy_tool.PolicyError, match="shared protected-branch scope"):
        policy_tool.check_policy(shared, _policy())


def test_ruleset_enumeration_requests_one_bounded_complete_page() -> None:
    assert "per_page=100" in policy_tool.RULESETS_ENDPOINT


def test_required_status_contexts_cannot_be_empty() -> None:
    policy = _policy()
    status_rule = next(
        rule for rule in policy["dev_ruleset"]["rules"] if rule["type"] == "required_status_checks"
    )
    status_rule["parameters"]["required_status_checks"] = []

    with pytest.raises(policy_tool.PolicyError, match="at least one required status check"):
        policy_tool._required_contexts(policy)


@pytest.mark.parametrize("producer", [None, 1, "15368", True])
def test_desired_required_checks_require_the_exact_actions_integration(
    producer: object,
) -> None:
    policy = _policy()
    status_rule = next(
        rule for rule in policy["dev_ruleset"]["rules"] if rule["type"] == "required_status_checks"
    )
    check = status_rule["parameters"]["required_status_checks"][0]
    if producer is None:
        del check["integration_id"]
    else:
        check["integration_id"] = producer

    with pytest.raises(policy_tool.PolicyError, match="integration_id"):
        policy_tool._validate_policy(policy)


def test_ruleset_readback_compares_the_required_check_producer() -> None:
    api = FakeApi()
    status_rule = next(
        rule for rule in api.ruleset["rules"] if rule["type"] == "required_status_checks"
    )
    del status_rule["parameters"]["required_status_checks"][0]["integration_id"]

    assert "dev ruleset differs" in policy_tool.check_policy(api, _policy())


@pytest.mark.parametrize(
    ("replacement", "message"),
    [
        (None, "at least one required status check"),
        ("Unregistered aggregate", "complete dev check manifest"),
    ],
    ids=["missing", "wrong-context"],
)
def test_settings_policy_requires_the_exact_dev_aggregate(
    replacement: str | None, message: str
) -> None:
    policy = _policy()
    status_rule = next(
        rule for rule in policy["dev_ruleset"]["rules"] if rule["type"] == "required_status_checks"
    )
    checks = status_rule["parameters"]["required_status_checks"]
    if replacement is None:
        checks.clear()
    else:
        checks[0]["context"] = replacement

    with pytest.raises(policy_tool.PolicyError, match=message):
        policy_tool._validate_policy(policy)


def test_settings_policy_must_match_the_complete_typed_main_manifest() -> None:
    policy = _policy()
    status_rule = next(
        rule for rule in policy["main_ruleset"]["rules"] if rule["type"] == "required_status_checks"
    )
    status_rule["parameters"]["required_status_checks"].pop()

    with pytest.raises(policy_tool.PolicyError, match="complete main check manifest"):
        policy_tool._validate_policy(policy)


def test_settings_policy_context_order_is_not_authoritative() -> None:
    policy = _policy()
    status_rule = next(
        rule for rule in policy["main_ruleset"]["rules"] if rule["type"] == "required_status_checks"
    )
    status_rule["parameters"]["required_status_checks"].reverse()

    policy_tool._validate_policy(policy)


def test_checked_in_settings_policy_satisfies_all_owned_contracts() -> None:
    policy = json.loads(POLICY_PATH.read_text(encoding="utf-8"))

    policy_tool._validate_policy(policy)


def test_main_settings_policy_requires_codeql_at_the_zero_alert_floor() -> None:
    policy = _policy()
    rules = policy["main_ruleset"]["rules"]
    code_scanning = next(rule for rule in rules if rule["type"] == "code_scanning")

    assert code_scanning["parameters"] == CODEQL_PROTECTION

    rules.remove(code_scanning)
    with pytest.raises(policy_tool.PolicyError, match="CodeQL code-scanning rule"):
        policy_tool._validate_policy(policy)


@pytest.mark.parametrize("threshold", ["none", "errors", "errors_and_warnings"])
def test_settings_policy_cannot_weaken_codeql_alert_protection(threshold: str) -> None:
    policy = _policy()
    rule = next(rule for rule in policy["main_ruleset"]["rules"] if rule["type"] == "code_scanning")
    rule["parameters"]["code_scanning_tools"][0]["alerts_threshold"] = threshold

    with pytest.raises(policy_tool.PolicyError, match="zero-alert floor"):
        policy_tool._validate_policy(policy)


def test_malformed_check_run_id_is_rejected_at_the_json_boundary(tmp_path: Path) -> None:
    api = FakeApi()
    api.check_runs[0]["id"] = "not-an-integer"

    with pytest.raises(policy_tool.PolicyError, match="integer id"):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert all(method == "GET" for method, _endpoint, _payload in api.calls)


@pytest.mark.parametrize(
    ("field", "value", "message"),
    [
        ("app", None, "app"),
        ("app_id", 1, "producer"),
        ("app_slug", "attacker-actions", "producer"),
        ("head_sha", "b" * 40, "head"),
    ],
)
def test_preflight_refuses_untrusted_required_check_evidence_before_mutation(
    tmp_path: Path,
    field: str,
    value: object,
    message: str,
) -> None:
    api = FakeApi()
    run = api.check_runs[0]
    if field == "app":
        run["app"] = value
    elif field == "head_sha":
        run["head_sha"] = value
    else:
        app = run["app"]
        assert isinstance(app, dict)
        app[field.removeprefix("app_")] = value

    with pytest.raises(policy_tool.PolicyError, match=message):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert all(method == "GET" for method, _endpoint, _payload in api.calls)


def test_preflight_refuses_an_incomplete_check_run_page_before_mutation(tmp_path: Path) -> None:
    api = FakeApi()
    api.check_runs_total_count = len(api.check_runs) + 1

    with pytest.raises(policy_tool.PolicyError, match="complete page"):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert all(method == "GET" for method, _endpoint, _payload in api.calls)


def test_later_wrong_app_duplicate_cannot_displace_the_canonical_run(tmp_path: Path) -> None:
    api = FakeApi()
    canonical = api.check_runs[0]
    forged = deepcopy(canonical)
    forged.update(
        {
            "id": 1000,
            "started_at": "2026-08-29T00:00:00Z",
            "conclusion": "failure",
            "app": {"id": 1, "slug": "attacker-actions"},
        }
    )
    api.check_runs.append(forged)

    policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert policy_tool.check_policy(api, _policy()) == []


def test_newer_queued_canonical_duplicate_with_no_started_at_blocks_preflight(
    tmp_path: Path,
) -> None:
    api = FakeApi()
    queued = deepcopy(api.check_runs[0])
    queued.update(
        {
            "id": 1000,
            "started_at": None,
            "status": "queued",
            "conclusion": None,
        }
    )
    api.check_runs.append(queued)

    with pytest.raises(policy_tool.PolicyError, match="status=queued, conclusion="):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert all(method == "GET" for method, _endpoint, _payload in api.calls)


def test_repeated_lower_canonical_check_run_id_is_rejected_before_mutation(
    tmp_path: Path,
) -> None:
    api = FakeApi()
    newer = deepcopy(api.check_runs[0])
    newer["id"] = 1000
    api.check_runs.append(newer)
    duplicate = deepcopy(api.check_runs[0])
    duplicate["started_at"] = None
    duplicate["status"] = "queued"
    duplicate["conclusion"] = None
    api.check_runs.append(duplicate)

    with pytest.raises(policy_tool.PolicyError, match=f"repeats id {duplicate['id']}"):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert all(method == "GET" for method, _endpoint, _payload in api.calls)


def test_gh_api_has_a_fixed_timeout(monkeypatch: pytest.MonkeyPatch) -> None:
    def timeout(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        assert kwargs["timeout"] == policy_tool.API_TIMEOUT_SECONDS
        raise subprocess.TimeoutExpired(cmd=args[0], timeout=kwargs["timeout"])

    monkeypatch.setattr(policy_tool.subprocess, "run", timeout)

    with pytest.raises(policy_tool.GitHubApiError, match="timed out"):
        policy_tool.GhApi._run(["repos/percy-raskova/babylon"])


def test_gh_api_normalizes_an_os_spawn_failure(monkeypatch: pytest.MonkeyPatch) -> None:
    def fail_spawn(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        raise OSError("gh executable unavailable")

    monkeypatch.setattr(policy_tool.subprocess, "run", fail_spawn)

    with pytest.raises(policy_tool.GitHubApiError, match="could not start"):
        policy_tool.GhApi._run(["repos/percy-raskova/babylon"])


def test_alternate_policy_label_is_refused_before_any_api_call(tmp_path: Path) -> None:
    api = FakeApi()
    policy = _policy()
    policy["automerge_label"]["name"] = "dependencies:alternate"

    with pytest.raises(policy_tool.PolicyError, match="canonical automerge label"):
        policy_tool.apply_policy(api, policy, api.dev_sha, tmp_path / "before.json")

    assert api.calls == []


def test_apply_snapshots_before_mutation_and_verifies_readback(tmp_path: Path) -> None:
    api = FakeApi()
    snapshot_path = tmp_path / "before.json"

    policy_tool.apply_policy(api, _policy(), api.dev_sha, snapshot_path)

    snapshot = json.loads(snapshot_path.read_text())
    assert snapshot["dev_sha"] == api.dev_sha
    assert snapshot["dev_ruleset_id"] == 18807584
    assert snapshot["main_ruleset_id"] == 18807583
    assert snapshot["actions_permissions"]["sha_pinning_required"] is False
    assert snapshot["automerge_label"] is None
    first_write = next(index for index, call in enumerate(api.calls) if call[0] != "GET")
    assert snapshot_path.is_file()
    assert first_write > 0
    assert policy_tool.check_policy(api, _policy()) == []


def test_apply_refuses_moved_dev_or_non_green_checks_before_mutation(tmp_path: Path) -> None:
    moved = FakeApi()
    with pytest.raises(policy_tool.PolicyError, match="dev moved"):
        policy_tool.apply_policy(moved, _policy(), "b" * 40, tmp_path / "moved.json")
    assert all(method == "GET" for method, _endpoint, _payload in moved.calls)

    red = FakeApi()
    red.check_runs[0]["conclusion"] = "failure"
    with pytest.raises(policy_tool.PolicyError, match=re.escape(DEV_BLOCKING_CHECKS[0])):
        policy_tool.apply_policy(red, _policy(), red.dev_sha, tmp_path / "red.json")
    assert all(method == "GET" for method, _endpoint, _payload in red.calls)


@pytest.mark.parametrize("conclusion", [None, "failure"])
def test_apply_requires_green_container_scan_before_ruleset_transition(
    tmp_path: Path,
    conclusion: str | None,
) -> None:
    api = FakeApi()
    scan_name = "Main Qualification / Container Image Scan"
    if conclusion is None:
        api.check_runs = [run for run in api.check_runs if run["name"] != scan_name]
    else:
        scan = next(run for run in api.check_runs if run["name"] == scan_name)
        scan["conclusion"] = conclusion

    with pytest.raises(policy_tool.PolicyError, match=re.escape(scan_name)):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert all(method == "GET" for method, _endpoint, _payload in api.calls)


def test_apply_refuses_skipped_dev_aggregate(tmp_path: Path) -> None:
    api = FakeApi()
    api.check_runs[0]["conclusion"] = "skipped"

    with pytest.raises(policy_tool.PolicyError, match=re.escape(DEV_BLOCKING_CHECKS[0])):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert all(method == "GET" for method, _endpoint, _payload in api.calls)


def test_apply_refuses_missing_main_qualification_evidence(tmp_path: Path) -> None:
    api = FakeApi()
    api.check_runs = [
        run
        for run in api.check_runs
        if run["name"] != "Main Qualification / Reference-Data Contracts"
    ]

    with pytest.raises(policy_tool.PolicyError, match="Reference-Data Contracts"):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert all(method == "GET" for method, _endpoint, _payload in api.calls)


def test_existing_snapshot_is_preserved_before_any_remote_mutation(tmp_path: Path) -> None:
    api = FakeApi()
    snapshot_path = tmp_path / "before.json"
    sentinel = b"original rollback bytes\x00\xff"
    snapshot_path.write_bytes(sentinel)

    with pytest.raises(policy_tool.PolicyError, match="already exists"):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, snapshot_path)

    assert snapshot_path.read_bytes() == sentinel
    assert all(method == "GET" for method, _endpoint, _payload in api.calls)


def test_concurrent_drift_aborts_before_first_mutation(tmp_path: Path) -> None:
    api = FakeApi()
    api.concurrent_ruleset_drift = True

    with pytest.raises(policy_tool.PolicyError, match="changed after snapshot"):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert all(method == "GET" for method, _endpoint, _payload in api.calls)


def test_moved_dev_after_green_evidence_aborts_before_first_mutation(tmp_path: Path) -> None:
    api = FakeApi()
    expected_sha = api.dev_sha
    api.move_dev_on_read = 2

    with pytest.raises(policy_tool.PolicyError, match="dev moved after green evidence"):
        policy_tool.apply_policy(api, _policy(), expected_sha, tmp_path / "before.json")

    assert all(method == "GET" for method, _endpoint, _payload in api.calls)


def test_moved_dev_during_apply_rolls_back_every_mutation(tmp_path: Path) -> None:
    api = FakeApi()
    expected_sha = api.dev_sha
    api.move_dev_on_read = 3
    original_ruleset = deepcopy(api.ruleset)
    original_main_ruleset = deepcopy(api.main_ruleset)
    original_repository = deepcopy(api.repository)
    original_actions_permissions = deepcopy(api.actions_permissions)

    with pytest.raises(policy_tool.PolicyError, match="dev moved during policy apply"):
        policy_tool.apply_policy(api, _policy(), expected_sha, tmp_path / "before.json")

    assert policy_tool.normalize_ruleset(api.ruleset) == policy_tool.normalize_ruleset(
        original_ruleset
    )
    assert policy_tool.normalize_ruleset(api.main_ruleset) == policy_tool.normalize_ruleset(
        original_main_ruleset
    )
    assert api.repository == original_repository
    assert api.actions_permissions == original_actions_permissions
    assert api.labels == []


def test_failed_ruleset_update_rolls_back_repo_and_new_label(tmp_path: Path) -> None:
    api = FakeApi()
    api.fail_method_endpoint = (
        "PUT",
        policy_tool.DEV_RULESET_ENDPOINT.format(ruleset_id=18807584),
    )
    original_repository = deepcopy(api.repository)

    with pytest.raises(policy_tool.GitHubApiError, match="injected PUT failure"):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert api.repository == original_repository
    assert api.labels == []
    ruleset_endpoint = policy_tool.DEV_RULESET_ENDPOINT.format(ruleset_id=18807584)
    assert sum(call[:2] == ("PUT", ruleset_endpoint) for call in api.calls) == 1


@pytest.mark.parametrize(
    ("method", "endpoint"),
    [
        ("POST", policy_tool.LABELS_ENDPOINT),
        ("PATCH", policy_tool.REPOSITORY_ENDPOINT),
        ("PUT", policy_tool.ACTIONS_PERMISSIONS_ENDPOINT),
        ("PUT", policy_tool.DEV_RULESET_ENDPOINT.format(ruleset_id=18807584)),
        ("PUT", policy_tool.MAIN_RULESET_ENDPOINT.format(ruleset_id=18807583)),
    ],
)
def test_ambiguous_committed_write_is_discovered_and_rolled_back(
    tmp_path: Path,
    method: str,
    endpoint: str,
) -> None:
    api = FakeApi()
    api.mutate_then_fail_method_endpoint = (method, endpoint)
    original_ruleset = deepcopy(api.ruleset)
    original_repository = deepcopy(api.repository)

    with pytest.raises(policy_tool.GitHubApiError, match=f"ambiguous {method} failure"):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert policy_tool.normalize_ruleset(api.ruleset) == policy_tool.normalize_ruleset(
        original_ruleset
    )
    assert api.repository == original_repository
    assert api.labels == []


def test_failed_label_create_does_not_delete_an_absent_label(tmp_path: Path) -> None:
    api = FakeApi()
    api.fail_method_endpoint = ("POST", policy_tool.LABELS_ENDPOINT)

    with pytest.raises(policy_tool.GitHubApiError, match="injected POST failure"):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert not any(call[0] == "DELETE" for call in api.calls)
    assert api.labels == []


def test_failed_repository_update_rolls_back_only_the_created_label(tmp_path: Path) -> None:
    api = FakeApi()
    api.fail_method_endpoint = ("PATCH", policy_tool.REPOSITORY_ENDPOINT)
    original_ruleset = deepcopy(api.ruleset)
    original_repository = deepcopy(api.repository)

    with pytest.raises(policy_tool.GitHubApiError, match="injected PATCH failure"):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert api.repository == original_repository
    assert api.ruleset == original_ruleset
    assert api.labels == []
    assert not any(call[0] == "PUT" for call in api.calls)


def test_oserror_after_label_creation_still_rolls_back_the_label(tmp_path: Path) -> None:
    api = FakeApi()
    api.fail_oserror_method_endpoint = ("PATCH", policy_tool.REPOSITORY_ENDPOINT)

    with pytest.raises(OSError, match="spawn failure"):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert api.labels == []


def test_readback_mismatch_rolls_back_every_mutation(tmp_path: Path) -> None:
    api = FakeApi()
    api.mismatch_ruleset_readback = True
    original_ruleset = deepcopy(api.ruleset)
    original_repository = deepcopy(api.repository)
    original_actions_permissions = deepcopy(api.actions_permissions)

    with pytest.raises(policy_tool.PolicyError, match="readback mismatch"):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert policy_tool.normalize_ruleset(api.ruleset) == policy_tool.normalize_ruleset(
        original_ruleset
    )
    assert api.repository == original_repository
    assert api.actions_permissions == original_actions_permissions
    assert api.labels == []


def test_required_check_producer_readback_mismatch_rolls_back(tmp_path: Path) -> None:
    api = FakeApi()
    api.mismatch_ruleset_producer_readback = True
    original_ruleset = deepcopy(api.ruleset)

    with pytest.raises(policy_tool.PolicyError, match="readback mismatch"):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert policy_tool.normalize_ruleset(api.ruleset) == policy_tool.normalize_ruleset(
        original_ruleset
    )
    assert api.labels == []


def test_recoverable_policy_error_during_rollback_read_uses_attempted_writes(
    tmp_path: Path,
) -> None:
    api = FakeApi()
    api.move_dev_on_read = 3
    api.policy_error_ruleset_read = 4
    original_ruleset = deepcopy(api.ruleset)
    original_repository = deepcopy(api.repository)

    with pytest.raises(policy_tool.PolicyError, match="dev moved during policy apply"):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert policy_tool.normalize_ruleset(api.ruleset) == policy_tool.normalize_ruleset(
        original_ruleset
    )
    assert api.repository == original_repository
    assert api.labels == []


def test_rollback_attempts_every_component_after_one_restore_fails(tmp_path: Path) -> None:
    api = FakeApi()
    api.mismatch_ruleset_readback = True
    api.fail_ruleset_restore = True
    original_repository = deepcopy(api.repository)

    with pytest.raises(policy_tool.PolicyError, match="rollback also failed"):
        policy_tool.apply_policy(api, _policy(), api.dev_sha, tmp_path / "before.json")

    assert api.repository == original_repository
    assert api.labels == []
    assert any(call[0] == "DELETE" for call in api.calls)


def test_manual_rollback_restores_snapshot_and_verifies_readback(tmp_path: Path) -> None:
    api = FakeApi()
    original_ruleset = deepcopy(api.ruleset)
    original_main_ruleset = deepcopy(api.main_ruleset)
    original_repository = deepcopy(api.repository)
    original_actions_permissions = deepcopy(api.actions_permissions)
    snapshot_path = tmp_path / "before.json"
    policy_tool.apply_policy(api, _policy(), api.dev_sha, snapshot_path)

    policy_tool.rollback_policy(api, policy_tool.load_snapshot(snapshot_path))

    assert policy_tool.normalize_ruleset(api.ruleset) == policy_tool.normalize_ruleset(
        original_ruleset
    )
    assert policy_tool.normalize_ruleset(api.main_ruleset) == policy_tool.normalize_ruleset(
        original_main_ruleset
    )
    assert api.repository == original_repository
    assert api.actions_permissions == original_actions_permissions
    assert api.labels == []


def test_manual_rollback_preserves_actions_permissions_for_legacy_snapshot(
    tmp_path: Path,
) -> None:
    api = FakeApi()
    original_ruleset = deepcopy(api.ruleset)
    original_main_ruleset = deepcopy(api.main_ruleset)
    original_repository = deepcopy(api.repository)
    snapshot_path = tmp_path / "before.json"
    policy_tool.apply_policy(api, _policy(), api.dev_sha, snapshot_path)
    actions_permissions_after_apply = deepcopy(api.actions_permissions)
    legacy_snapshot = policy_tool.load_snapshot(snapshot_path)
    del legacy_snapshot["actions_permissions"]
    api.calls.clear()

    policy_tool.rollback_policy(api, legacy_snapshot)

    assert policy_tool.normalize_ruleset(api.ruleset) == policy_tool.normalize_ruleset(
        original_ruleset
    )
    assert policy_tool.normalize_ruleset(api.main_ruleset) == policy_tool.normalize_ruleset(
        original_main_ruleset
    )
    assert api.repository == original_repository
    assert api.actions_permissions == actions_permissions_after_apply
    assert api.labels == []
    assert not any(
        method == "PUT" and endpoint == policy_tool.ACTIONS_PERMISSIONS_ENDPOINT
        for method, endpoint, _payload in api.calls
    )


def test_manual_rollback_preserves_historical_unbound_required_checks(
    tmp_path: Path,
) -> None:
    api = FakeApi()
    snapshot_path = tmp_path / "before.json"
    policy_tool.apply_policy(api, _policy(), api.dev_sha, snapshot_path)
    historical_snapshot = policy_tool.load_snapshot(snapshot_path)
    for branch in ("dev", "main"):
        status_rule = next(
            rule
            for rule in historical_snapshot[f"{branch}_ruleset"]["rules"]
            if rule["type"] == "required_status_checks"
        )
        for check in status_rule["parameters"]["required_status_checks"]:
            del check["integration_id"]

    policy_tool.rollback_policy(api, historical_snapshot)

    for ruleset in (api.ruleset, api.main_ruleset):
        status_rule = next(
            rule for rule in ruleset["rules"] if rule["type"] == "required_status_checks"
        )
        assert all(
            "integration_id" not in check
            for check in status_rule["parameters"]["required_status_checks"]
        )


def test_manual_rollback_refuses_a_stale_dev_snapshot_without_mutation(tmp_path: Path) -> None:
    api = FakeApi()
    snapshot_path = tmp_path / "before.json"
    policy_tool.apply_policy(api, _policy(), api.dev_sha, snapshot_path)
    snapshot = policy_tool.load_snapshot(snapshot_path)
    api.calls.clear()
    api.dev_sha = "b" * 40

    with pytest.raises(policy_tool.PolicyError, match="snapshot dev"):
        policy_tool.rollback_policy(api, snapshot)

    assert all(method == "GET" for method, _endpoint, _payload in api.calls)


def test_manual_rollback_refuses_tampered_scope_before_mutation(tmp_path: Path) -> None:
    api = FakeApi()
    snapshot_path = tmp_path / "before.json"
    policy_tool.apply_policy(api, _policy(), api.dev_sha, snapshot_path)
    snapshot = policy_tool.load_snapshot(snapshot_path)
    snapshot["dev_ruleset"]["conditions"]["ref_name"]["include"].append("refs/heads/main")
    api.calls.clear()

    with pytest.raises(policy_tool.PolicyError, match="exact dev-only branch scope"):
        policy_tool.rollback_policy(api, snapshot)

    assert api.calls == []


def test_manual_rollback_refuses_tampered_main_scope_before_mutation(tmp_path: Path) -> None:
    api = FakeApi()
    snapshot_path = tmp_path / "before.json"
    policy_tool.apply_policy(api, _policy(), api.dev_sha, snapshot_path)
    snapshot = policy_tool.load_snapshot(snapshot_path)
    snapshot["main_ruleset"]["conditions"]["ref_name"]["include"].append("refs/heads/dev")
    api.calls.clear()

    with pytest.raises(policy_tool.PolicyError, match="exact main-only branch scope"):
        policy_tool.rollback_policy(api, snapshot)

    assert api.calls == []


def test_manual_rollback_refuses_a_malformed_saved_producer_before_mutation(
    tmp_path: Path,
) -> None:
    api = FakeApi()
    snapshot_path = tmp_path / "before.json"
    policy_tool.apply_policy(api, _policy(), api.dev_sha, snapshot_path)
    snapshot = policy_tool.load_snapshot(snapshot_path)
    status_rule = next(
        rule
        for rule in snapshot["dev_ruleset"]["rules"]
        if rule["type"] == "required_status_checks"
    )
    status_rule["parameters"]["required_status_checks"][0]["integration_id"] = "15368"
    api.calls.clear()

    with pytest.raises(policy_tool.PolicyError, match="malformed integration_id"):
        policy_tool.rollback_policy(api, snapshot)

    assert api.calls == []


def test_manual_rollback_refuses_a_different_ruleset_id_before_mutation(tmp_path: Path) -> None:
    api = FakeApi()
    snapshot_path = tmp_path / "before.json"
    policy_tool.apply_policy(api, _policy(), api.dev_sha, snapshot_path)
    snapshot = policy_tool.load_snapshot(snapshot_path)
    snapshot["dev_ruleset_id"] = 999
    api.calls.clear()

    with pytest.raises(policy_tool.PolicyError, match="ruleset ID changed"):
        policy_tool.rollback_policy(api, snapshot)

    assert all(method == "GET" for method, _endpoint, _payload in api.calls)


def test_dev_settings_policy_keeps_codeql_asynchronous() -> None:
    policy = _policy()
    assert all(rule["type"] != "code_scanning" for rule in policy["dev_ruleset"]["rules"])
    policy["dev_ruleset"]["rules"].append(
        {"type": "code_scanning", "parameters": deepcopy(CODEQL_PROTECTION)}
    )
    with pytest.raises(policy_tool.PolicyError, match="dev CodeQL scans must be asynchronous"):
        policy_tool._validate_policy(policy)


def test_codeql_only_migration_preserves_all_other_settings_and_can_restore_gate(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    api = MigrationApi()
    desired = _policy()
    for branch, ruleset in (("dev", api.ruleset), ("main", api.main_ruleset)):
        current = next(r for r in ruleset["rules"] if r["type"] == "required_status_checks")
        expected = next(
            r
            for r in desired[f"{branch}_ruleset"]["rules"]
            if r["type"] == "required_status_checks"
        )
        current["parameters"] = deepcopy(expected["parameters"])
    before = {"dev_sha": api.dev_sha, **policy_tool._current_state(api)}
    monkeypatch.setattr(policy_tool, "_verify_migration_pr_checks", lambda *_: None)

    _migration_apply(api, tmp_path)

    assert all(r["type"] != "code_scanning" for r in api.ruleset["rules"])
    saved = policy_tool.load_snapshot(tmp_path / "before.json")
    assert saved == before
    writes = [(method, endpoint) for method, endpoint, _ in api.calls if method != "GET"]
    assert writes == [("PUT", policy_tool.DEV_RULESET_ENDPOINT.format(ruleset_id=18807584))]
    policy_tool.rollback_policy(api, saved)
    assert {"dev_sha": api.dev_sha, **policy_tool._current_state(api)} == before


@pytest.mark.parametrize("fault", ["threshold", "security-threshold", "extra-tool", "main-rule"])
def test_codeql_migration_refuses_any_other_scanning_transition(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, fault: str
) -> None:
    api = MigrationApi()
    rules = api.main_ruleset["rules"] if fault == "main-rule" else api.ruleset["rules"]
    rule = next(r for r in rules if r["type"] == "code_scanning")
    if fault == "main-rule":
        rules.remove(rule)
    elif fault == "extra-tool":
        rule["parameters"]["code_scanning_tools"].append({"tool": "Other"})
    else:
        field = "alerts_threshold" if fault == "threshold" else "security_alerts_threshold"
        rule["parameters"]["code_scanning_tools"][0][field] = "none"
    monkeypatch.setattr(policy_tool, "_verify_migration_pr_checks", lambda *_: None)
    with pytest.raises(policy_tool.PolicyError):
        _migration_apply(api, tmp_path)
    assert all(method == "GET" for method, _, _ in api.calls)
