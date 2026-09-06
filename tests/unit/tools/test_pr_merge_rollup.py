"""Tests for ``tools.pr_merge._rollup_failures`` duplicate-check semantics.

A re-run (flake retry, close/reopen refire) leaves MULTIPLE rollup entries
for the same check name on the same head SHA. GitHub branch protection
evaluates the LATEST run per context; the merge gate must do the same,
or a superseded failure poisons the verdict forever (bit PR #673 on
2026-08-21: three stale Security Audit failures outvoted the fresh pass).
"""

from __future__ import annotations

import importlib.util
from pathlib import Path

import pytest

_SPEC = importlib.util.spec_from_file_location(
    "pr_merge", Path(__file__).resolve().parents[3] / "tools" / "pr_merge.py"
)
if _SPEC is None or _SPEC.loader is None:
    raise RuntimeError("tools/pr_merge.py failed import-spec resolution")
pr_merge = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(pr_merge)

DEV_BLOCKING_CHECKS = (
    "Fast Gate (hygiene, lint, format, imports, types, lock)",
    "Unit Tests (xdist, coverage gate)",
    "Determinism Gate (byte-identical dense goldens)",
    "Secret Scan (gitleaks, full history)",
    "IaC Config Scan (trivy, HIGH+CRITICAL blocking)",
    "Security Audit (pip-audit policy — blocking since item-41)",
    "Rust Gate (fmt, clippy, test, doc — rust/ workspace)",
    "Baseline Ceremony Gate (§6.5 provenance)",
    "Postgres Integration Tier (PG 17, pinned runtime)",
)
OPTIONAL_DEPENDABOT_CHECKS = (
    "Classify Dependabot update",
    "Dependabot Eligibility",
)


def _entry(
    name: str,
    conclusion: str,
    started_at: str,
    status: str = "COMPLETED",
) -> dict[str, str]:
    return {
        "name": name,
        "status": status,
        "conclusion": conclusion,
        "startedAt": started_at,
        "completedAt": started_at,
    }


def _dev_manifest_green() -> list[dict[str, str]]:
    return [_entry(name, "SUCCESS", "2026-08-21T17:00:00Z") for name in DEV_BLOCKING_CHECKS]


class TestRollupFailuresLatestPerCheck:
    def test_stale_failure_does_not_poison_a_fresh_success(self) -> None:
        rollup = [
            _entry("Security Audit", "FAILURE", "2026-08-21T17:30:44Z"),
            _entry("Security Audit", "FAILURE", "2026-08-21T18:11:11Z"),
            _entry("Security Audit", "SUCCESS", "2026-08-21T18:21:15Z"),
        ]
        assert pr_merge._rollup_failures(rollup + _dev_manifest_green()) == []

    def test_latest_failure_still_refuses(self) -> None:
        rollup = [
            _entry("Security Audit", "SUCCESS", "2026-08-21T17:30:44Z"),
            _entry("Security Audit", "FAILURE", "2026-08-21T18:21:15Z"),
        ]
        failures = pr_merge._rollup_failures(rollup)
        assert any("Security Audit" in f and "FAILURE" in f for f in failures)

    def test_latest_in_progress_still_refuses(self) -> None:
        rollup = [
            _entry("Unit Tests", "SUCCESS", "2026-08-21T17:30:44Z"),
            _entry("Unit Tests", "", "2026-08-21T18:21:15Z", status="IN_PROGRESS"),
        ]
        failures = pr_merge._rollup_failures(rollup)
        assert any("Unit Tests" in f and "IN_PROGRESS" in f for f in failures)

    def test_in_progress_unknown_check_is_typed_as_pending(self) -> None:
        findings = pr_merge._rollup_findings(
            [_entry("Extension", "", "2026-08-21T18:21:15Z", status="IN_PROGRESS")],
            allow_pending=True,
        )

        assert findings.hard == []
        assert findings.pending == ["Extension: still IN_PROGRESS"]

    @pytest.mark.parametrize("state", ["EXPECTED", "PENDING"])
    def test_pending_status_context_is_typed_as_pending(self, state: str) -> None:
        findings = pr_merge._rollup_findings(
            [
                {
                    "context": "External status",
                    "state": state,
                    "startedAt": "2026-08-21T18:21:15Z",
                }
            ],
            allow_pending=True,
        )

        assert findings.hard == []
        assert findings.pending == [f"External status: still {state}"]

    @pytest.mark.parametrize("with_status", [False, True], ids=["status-context", "check-run"])
    def test_neutral_unknown_rollup_is_pending_only_for_dependabot(
        self,
        with_status: bool,
    ) -> None:
        entry = _entry("CodeQL aggregate", "NEUTRAL", "2026-08-21T18:21:15Z")
        if not with_status:
            entry.pop("status")
            entry["state"] = entry.pop("conclusion")

        dependabot = pr_merge._rollup_findings([entry], allow_pending=True)
        ordinary = pr_merge._rollup_findings([entry])

        assert dependabot.hard == []
        assert dependabot.pending == ["CodeQL aggregate: NEUTRAL"]
        assert ordinary.hard == ["CodeQL aggregate: NEUTRAL"]
        assert ordinary.pending == []

    def test_completed_unknown_failure_is_typed_as_hard(self) -> None:
        findings = pr_merge._rollup_findings(
            [_entry("Extension", "FAILURE", "2026-08-21T18:21:15Z")]
        )

        assert findings.hard == ["Extension: FAILURE"]
        assert findings.pending == []

    def test_distinct_names_are_not_deduped_across(self) -> None:
        rollup = [
            _entry("Fast Gate", "FAILURE", "2026-08-21T17:30:44Z"),
            _entry("Rust Gate", "SUCCESS", "2026-08-21T18:21:15Z"),
        ]
        failures = pr_merge._rollup_failures(rollup)
        assert any("Fast Gate" in f for f in failures)

    def test_entries_without_started_at_fall_back_to_run_id(self) -> None:
        base = "https://github.com/o/r/actions/runs"
        rollup = [
            {
                "name": "Security Audit",
                "status": "COMPLETED",
                "conclusion": "FAILURE",
                "detailsUrl": f"{base}/111/job/1",
            },
            {
                "name": "Security Audit",
                "status": "COMPLETED",
                "conclusion": "SUCCESS",
                "detailsUrl": f"{base}/222/job/1",
            },
        ]
        assert pr_merge._rollup_failures(rollup + _dev_manifest_green()) == []

    def test_every_blocking_check_still_requires_explicit_pass(
        self,
        monkeypatch: pytest.MonkeyPatch,
    ) -> None:
        """Canonical REST evidence, not the source-free rollup, enforces explicit success."""
        critical = DEV_BLOCKING_CHECKS[0]
        rollup = [
            *[entry for entry in _dev_manifest_green() if entry["name"] != critical],
            _entry(critical, "SUCCESS", "2026-08-21T17:30:44Z"),
            _entry(critical, "SKIPPED", "2026-08-21T18:21:15Z"),
        ]
        assert pr_merge._rollup_failures(rollup) == []

        head_sha = "a" * 40
        check_runs = [
            {
                "id": index,
                "name": requirement.context,
                "head_sha": head_sha,
                "status": "completed",
                "conclusion": "skipped" if requirement.context == critical else "success",
                "started_at": "2026-08-21T18:21:15Z",
                "app": {
                    "id": requirement.producer.integration_id,
                    "slug": requirement.producer.slug,
                },
            }
            for index, requirement in enumerate(pr_merge.DEV_CHECK_MANIFEST, start=1)
        ]
        payload = {"total_count": len(check_runs), "check_runs": check_runs}
        monkeypatch.setattr(pr_merge, "_gh_json", lambda *_args: payload)

        failures = pr_merge._manifest_check_run_failures(head_sha, pr_merge.DEV_CHECK_MANIFEST)
        assert any(critical in failure and "SKIPPED" in failure for failure in failures)

    def test_optional_dependabot_checks_may_skip_on_a_human_pr(self) -> None:
        rollup = [
            *_dev_manifest_green(),
            *[
                _entry(name, "SKIPPED", "2026-08-26T02:48:05Z")
                for name in OPTIONAL_DEPENDABOT_CHECKS
            ],
        ]
        assert pr_merge._rollup_failures(rollup) == []

    def test_optional_dependabot_failure_still_refuses(self) -> None:
        rollup = [
            *_dev_manifest_green(),
            _entry(OPTIONAL_DEPENDABOT_CHECKS[0], "FAILURE", "2026-08-26T02:48:05Z"),
        ]
        failures = pr_merge._rollup_failures(rollup)
        assert any(OPTIONAL_DEPENDABOT_CHECKS[0] in failure for failure in failures)

    def test_unknown_skipped_check_still_refuses(self) -> None:
        rollup = [
            *_dev_manifest_green(),
            _entry("Unregistered Optional Check", "SKIPPED", "2026-08-26T02:48:05Z"),
        ]
        failures = pr_merge._rollup_failures(rollup)
        assert any("Unregistered Optional Check" in failure for failure in failures)
