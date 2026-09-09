"""Behavior contract for the accepted PR and dependency-merge policy."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[3]
ADR_KEY = "ADR230_exact_head_pr_and_dependabot_policy"
ADR_PATH = ROOT / "ai" / "decisions" / f"{ADR_KEY}.yaml"
INDEX_PATH = ROOT / "ai" / "decisions" / "index.yaml"
POLICY_PATH = ROOT / ".github" / "settings" / "pr-policy.json"


def _mapping(path: Path) -> dict[str, Any]:
    payload = yaml.safe_load(path.read_text(encoding="utf-8"))
    assert isinstance(payload, dict)
    return payload


def test_accepted_decision_records_every_pr_policy_boundary() -> None:
    decision = _mapping(ADR_PATH)[ADR_KEY]
    text = str(decision["decision"])
    normalized = " ".join(text.split())

    assert decision["status"] == "accepted"
    assert decision["date"] == "2026-08-25"
    assert "exact head" in text
    assert "base" in text
    assert "complete manifest" in text
    assert "blocking" in text
    assert "advisory" in text
    assert "completed Copilot review" in text
    assert "exact endpoint" in text
    assert "immutable bot" in text
    assert "resolved" in text
    assert "pull_request_target" in text
    assert "workflow_run" in text
    assert "Dependabot Eligibility" in text
    assert "GitHub Actions app" in text
    assert "native exact-head CI" in text
    assert "workflow ID" in text
    assert "event actor" in text
    assert "exactly one current commit" in text
    assert "normal reviewed path" in text
    assert "signed commit metadata" not in text
    assert "label is presentation only" in normalized
    assert "snapshot" in text
    assert "rollback" in text
    assert "attempted writes" in text
    assert "ADR181_ci_synergy_rulings" in decision["related"]


def test_decision_index_resolves_to_the_live_record() -> None:
    index = _mapping(INDEX_PATH)
    entry = index["decisions"][ADR_KEY]

    assert index["meta"]["version"] == "1.87.0"
    assert str(index["meta"]["updated"]) == "2026-08-26"
    assert entry["status"] == "accepted"
    assert entry["date"] == "2026-08-25"
    assert entry["file"] == ADR_PATH.name


def test_checked_in_policy_is_merge_only_strict_and_conversation_safe() -> None:
    policy = _mapping(POLICY_PATH)
    repository = policy["repository"]
    assert repository == {
        "allow_merge_commit": True,
        "allow_squash_merge": False,
        "allow_rebase_merge": False,
        "allow_auto_merge": False,
        "delete_branch_on_merge": False,
    }

    rules = {rule["type"]: rule for rule in policy["dev_ruleset"]["rules"]}
    pull_request = rules["pull_request"]["parameters"]
    checks = rules["required_status_checks"]["parameters"]
    assert pull_request["allowed_merge_methods"] == ["merge"]
    assert pull_request["required_review_thread_resolution"] is True
    assert checks["strict_required_status_checks_policy"] is True
