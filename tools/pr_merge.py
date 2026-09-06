#!/usr/bin/env python3
"""The one sanctioned merge path (ADR181 R10): ``mise run pr:merge -- <pr>``.

Mechanizes the merge-protocol prose rules (CLAUDE.md "Merge protocol"),
each of which has already bitten:

1. Never ``--auto`` (#392: it ignores failing non-required checks) — this
   wrapper simply has no such flag.
2. All checks green AND both PR refs unchanged across the verdict snapshot.
3. Copilot review state and unreplied top-level comments are advisory; every
   review thread, regardless of author, must resolve.
4. Any open code-scanning alert is a STOP. CodeQL also runs on pull requests;
   the alert DB is the durable source of truth across all matrix languages.
5. ``--delete-branch`` is refused for ``dev`` and while another open PR bases
   on the head (#193: deleting it closes-not-merges the child).

Stdlib + ``gh`` only.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from enum import IntEnum
from pathlib import Path
from typing import Final

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools.pr_policy import (  # noqa: E402
    DEV_CHECK_MANIFEST,
    GITHUB_ACTIONS_PRODUCER,
    CheckRequirement,
    manifest_for_base,
)

MAX_GITHUB_ITEMS = 100
GH_TIMEOUT_SECONDS: Final[int] = 30
GH_ERROR_DETAIL_LIMIT: Final[int] = 400


class MergeOutcome(IntEnum):
    """Stable process outcomes consumed by humans and trusted automation."""

    SUCCESS = 0
    HARD_REFUSAL = 1
    MUTATION_INDETERMINATE = 2
    DEPENDABOT_MAJOR_REVIEW = 3
    EXACT_HEAD_EVIDENCE_PENDING = 4


INDETERMINATE_EXIT_CODE: Final[int] = MergeOutcome.MUTATION_INDETERMINATE

COMPLETED_REVIEW_STATES = {"APPROVED", "CHANGES_REQUESTED", "COMMENTED"}
COPILOT_GRAPHQL_LOGIN = "copilot-pull-request-reviewer"
COPILOT_REST_REVIEW_LOGIN = "copilot-pull-request-reviewer[bot]"
COPILOT_REST_COMMENT_LOGIN = "Copilot"
COPILOT_BOT_ID = 175728472
COPILOT_BOT_NODE_ID = "BOT_kgDOCnlnWA"
DEPENDABOT_LOGIN = "dependabot[bot]"
DEPENDABOT_BOT_ID = 49699333
DEPENDABOT_CLASSIFIER_CHECK = "Classify Dependabot update"
DEPENDABOT_ELIGIBILITY_CHECK = "Dependabot Eligibility"
OPTIONAL_SKIPPED_CHECKS: Final[frozenset[str]] = frozenset(
    {DEPENDABOT_CLASSIFIER_CHECK, DEPENDABOT_ELIGIBILITY_CHECK}
)
DEPENDABOT_WORKFLOW_PATH = ".github/workflows/dependabot-automerge.yml"
DEPENDABOT_WORKFLOW_ID = 214604133
CI_WORKFLOW_PATH = ".github/workflows/ci.yml"
CI_WORKFLOW_ID = 176131131
GITHUB_ACTIONS_APP_ID: Final[int] = GITHUB_ACTIONS_PRODUCER.integration_id
GITHUB_ACTIONS_APP_SLUG: Final[str] = GITHUB_ACTIONS_PRODUCER.slug
CHECK_RUNS_ENDPOINT: Final[str] = (
    "repos/{{owner}}/{{repo}}/commits/{head_oid}/check-runs?filter=all&per_page=100"
)

_REVIEW_THREADS_QUERY = """
query($owner: String!, $name: String!, $number: Int!) {
  repository(owner: $owner, name: $name) {
    pullRequest(number: $number) {
      reviewThreads(first: 100) {
        nodes {
          isResolved
          comments(first: 1) {
            nodes { url }
          }
        }
        pageInfo { hasNextPage }
      }
    }
  }
}
"""


class GitHubReadError(RuntimeError):
    """A bounded GitHub CLI read failed before producing trusted evidence."""


class MergeIndeterminateError(RuntimeError):
    """A mutating merge call could not be reconciled to one exact outcome."""


class CopilotEvidenceReadError(RuntimeError):
    """A bounded Copilot-only evidence read failed without blocking merge."""


class VerificationFindings:
    """Separate permanent refusals from exact-head evidence still settling."""

    def __init__(
        self,
        *,
        hard: list[str] | None = None,
        pending: list[str] | None = None,
    ) -> None:
        self.hard = [] if hard is None else hard
        self.pending = [] if pending is None else pending

    def extend(self, other: VerificationFindings) -> None:
        self.hard.extend(other.hard)
        self.pending.extend(other.pending)


class DependabotModeResult:
    """Dependabot provenance verdict plus its authenticated update class."""

    def __init__(
        self,
        problems: tuple[str, ...] = (),
        *,
        semver_major: bool = False,
        semver_major_reason: str = "authenticated Dependabot semver-major",
    ) -> None:
        self.problems = problems
        self.semver_major = semver_major
        self.semver_major_reason = semver_major_reason


def _bounded_error_detail(value: object) -> str:
    """Return one sanitized, statically bounded child-process detail."""
    if not isinstance(value, str):
        return ""
    truncated = len(value) > GH_ERROR_DETAIL_LIMIT
    bounded = value[:GH_ERROR_DETAIL_LIMIT]
    printable = re.sub(r"[\x00-\x1f\x7f-\x9f]", " ", bounded)
    detail = " ".join(printable.split())
    return detail + ("…" if truncated else "")


def _gh_failure_message(
    error: subprocess.TimeoutExpired | subprocess.CalledProcessError | OSError,
) -> str:
    """Normalize one bounded GitHub CLI failure without exposing raw output."""
    if isinstance(error, subprocess.TimeoutExpired):
        return f"gh timed out after {GH_TIMEOUT_SECONDS} seconds"
    if isinstance(error, subprocess.CalledProcessError):
        detail = (
            _bounded_error_detail(error.stderr)
            or _bounded_error_detail(error.stdout)
            or f"exit {error.returncode}"
        )
        return f"gh failed: {detail}"
    detail = _bounded_error_detail(str(error)) or type(error).__name__
    return f"gh could not start: {detail}"


def _run_gh(*args: str) -> subprocess.CompletedProcess[str]:
    """Run one GitHub CLI command with the shared fixed timeout."""
    return subprocess.run(
        ["gh", *args],
        capture_output=True,
        text=True,
        check=True,
        timeout=GH_TIMEOUT_SECONDS,
    )


def _gh(*args: str) -> str:
    """Run one bounded read and normalize command failures."""
    try:
        return _run_gh(*args).stdout
    except (subprocess.TimeoutExpired, subprocess.CalledProcessError, OSError) as error:
        raise GitHubReadError(_gh_failure_message(error)) from error


def _gh_json(*args: str) -> object:
    try:
        return json.loads(_gh(*args))
    except json.JSONDecodeError as error:
        raise GitHubReadError("gh returned invalid JSON") from error


def _json_dict(value: object, context: str) -> dict[str, object]:
    """Require one JSON object from a GitHub response."""
    if not isinstance(value, dict):
        raise RuntimeError(f"{context} must be a JSON object")
    return value


def _json_dicts(
    value: object,
    context: str,
    *,
    refuse_full_page: bool = False,
) -> list[dict[str, object]]:
    """Require one JSON array containing only objects."""
    if not isinstance(value, list):
        raise RuntimeError(f"{context} must be a JSON array")
    if len(value) > MAX_GITHUB_ITEMS or (refuse_full_page and len(value) == MAX_GITHUB_ITEMS):
        raise RuntimeError(f"{context} reached the {MAX_GITHUB_ITEMS}-item safety bound")
    items: list[dict[str, object]] = []
    for index, item in enumerate(value):
        if not isinstance(item, dict):
            raise RuntimeError(f"{context}[{index}] must be a JSON object")
        items.append(item)
    return items


def _entry_sort_key(entry: dict[str, object]) -> tuple[str, int]:
    """Recency key for duplicate check names on one head SHA.

    Re-runs (flake retry, close/reopen refire) leave several rollup entries
    per name; branch protection evaluates the LATEST per context and so must
    we — a superseded failure must not poison the verdict (bit PR #673 on
    2026-08-21). Prefer ``startedAt``; fall back to the run id embedded in
    ``detailsUrl`` (``.../actions/runs/<id>/job/...``).
    """
    started = str(entry.get("startedAt") or "")
    if started:
        return (started, 0)
    url = str(entry.get("detailsUrl") or entry.get("targetUrl") or "")
    run_id = 0
    if "/actions/runs/" in url:
        tail = url.split("/actions/runs/", 1)[1].split("/", 1)[0]
        run_id = int(tail) if tail.isdigit() else 0
    return ("", run_id)


def _rollup_findings(
    rollup: list[dict[str, object]],
    manifest: tuple[CheckRequirement, ...] = DEV_CHECK_MANIFEST,
    *,
    allow_pending: bool = False,
) -> VerificationFindings:
    """Classify disallowed unknown rollup conclusions.

    GitHub's GraphQL union includes source-free ``StatusContext`` values, so
    the REST check-run endpoint alone proves the producer-bound manifest.
    The rollup remains useful for visibility into additional failing checks.
    """
    if len(rollup) >= MAX_GITHUB_ITEMS:
        raise RuntimeError(f"status-check rollup reached the {MAX_GITHUB_ITEMS}-item safety bound")
    if len(manifest) >= MAX_GITHUB_ITEMS:
        raise RuntimeError(f"check manifest reached the {MAX_GITHUB_ITEMS}-item safety bound")
    latest: dict[str, dict[str, object]] = {}
    expected = {requirement.context for requirement in manifest}
    for entry in rollup:
        name = str(entry.get("name") or entry.get("context") or "?")
        if name in expected:
            continue
        if name not in latest or _entry_sort_key(entry) >= _entry_sort_key(latest[name]):
            latest[name] = entry
    findings = VerificationFindings()
    for entry in latest.values():
        name = str(entry.get("name") or entry.get("context") or "?")
        status = str(entry.get("status") or "")
        conclusion = str(entry.get("conclusion") or entry.get("state") or "")
        if status and status != "COMPLETED":
            target = findings.pending if allow_pending else findings.hard
            target.append(f"{name}: still {status}")
        elif not status and conclusion in {"EXPECTED", "PENDING"}:
            target = findings.pending if allow_pending else findings.hard
            target.append(f"{name}: still {conclusion}")
        elif allow_pending and conclusion == "NEUTRAL":
            findings.pending.append(f"{name}: {conclusion}")
        elif conclusion != "SUCCESS" and not (
            conclusion == "SKIPPED" and name in OPTIONAL_SKIPPED_CHECKS
        ):
            findings.hard.append(f"{name}: {conclusion}")
    return findings


def _rollup_failures(
    rollup: list[dict[str, object]],
    manifest: tuple[CheckRequirement, ...] = DEV_CHECK_MANIFEST,
) -> list[str]:
    """Compatibility surface returning every blocking rollup finding."""
    findings = _rollup_findings(rollup, manifest)
    return [*findings.hard, *findings.pending]


def _manifest_check_run_findings(
    head_oid: str,
    manifest: tuple[CheckRequirement, ...],
    *,
    allow_pending: bool = False,
) -> VerificationFindings:
    """Classify canonical exact-head REST check-run evidence."""
    if re.fullmatch(r"[0-9a-f]{40}", head_oid) is None:
        raise RuntimeError("pull-request head is not one canonical 40-hex SHA")
    if len(manifest) >= MAX_GITHUB_ITEMS:
        raise RuntimeError(f"check manifest reached the {MAX_GITHUB_ITEMS}-item safety bound")
    endpoint = CHECK_RUNS_ENDPOINT.format(head_oid=head_oid)
    payload = _json_dict(_gh_json("api", endpoint), "exact-head check runs")
    total_count = payload.get("total_count")
    if type(total_count) is not int or total_count < 0 or total_count >= MAX_GITHUB_ITEMS:
        raise RuntimeError(
            f"exact-head check runs reached the {MAX_GITHUB_ITEMS}-item safety bound"
        )
    runs = _json_dicts(payload.get("check_runs"), "exact-head check runs")
    if total_count != len(runs):
        raise RuntimeError("exact-head check runs did not return one complete page")

    expected = {requirement.context: requirement for requirement in manifest}
    latest: dict[str, dict[str, object]] = {}
    seen_ids: dict[str, set[int]] = {}
    wrong_producer: set[str] = set()
    for run in runs:
        name = run.get("name")
        if not isinstance(name, str) or not name:
            raise RuntimeError("exact-head check run has no name")
        requirement = expected.get(name)
        if requirement is None:
            continue
        run_id = run.get("id")
        if type(run_id) is not int or run_id <= 0:
            raise RuntimeError(f"exact-head check run {name!r} has no positive integer id")
        started_at = run.get("started_at")
        if started_at is not None and not isinstance(started_at, str):
            raise RuntimeError(f"exact-head check run {name!r} has malformed started_at")
        run_head = run.get("head_sha")
        if not isinstance(run_head, str) or run_head != head_oid:
            raise RuntimeError(
                f"exact-head check run {name!r} has wrong head: "
                f"expected {head_oid}, found {run_head}"
            )
        app = _json_dict(run.get("app"), f"exact-head check run {name!r} app")
        app_id = app.get("id")
        app_slug = app.get("slug")
        if type(app_id) is not int or not isinstance(app_slug, str) or not app_slug:
            raise RuntimeError(f"exact-head check run {name!r} has malformed app identity")
        if app_id != requirement.producer.integration_id or app_slug != requirement.producer.slug:
            wrong_producer.add(name)
            continue
        context_ids = seen_ids.setdefault(name, set())
        if run_id in context_ids:
            raise RuntimeError(f"exact-head check run {name!r} repeats id {run_id}")
        context_ids.add(run_id)
        prior = latest.get(name)
        prior_id = prior.get("id") if prior is not None else 0
        if type(prior_id) is not int:
            raise RuntimeError(f"exact-head check run {name!r} has malformed prior id")
        # Check-run IDs are monotonic unique API identities; queued runs may have no started_at.
        if prior is None or run_id > prior_id:
            latest[name] = run

    findings = VerificationFindings()
    for requirement in manifest:
        selected = latest.get(requirement.context)
        if selected is None:
            if requirement.context in wrong_producer:
                findings.hard.append(
                    f"{requirement.context}: missing required producer "
                    f"{requirement.producer.slug} ({requirement.producer.integration_id})"
                )
            else:
                target = findings.pending if allow_pending else findings.hard
                target.append(
                    f"{requirement.context}: missing from complete {requirement.kind} manifest"
                )
            continue
        status = selected.get("status")
        conclusion = selected.get("conclusion")
        if not isinstance(status, str) or not status:
            raise RuntimeError(f"exact-head check run {requirement.context!r} has malformed status")
        if conclusion is not None and not isinstance(conclusion, str):
            raise RuntimeError(
                f"exact-head check run {requirement.context!r} has malformed conclusion"
            )
        normalized_status = status.upper()
        normalized_conclusion = conclusion.upper() if isinstance(conclusion, str) else ""
        if normalized_status != "COMPLETED":
            target = findings.pending if allow_pending else findings.hard
            target.append(f"{requirement.context}: still {normalized_status}")
        elif normalized_conclusion not in requirement.allowed_conclusions:
            findings.hard.append(f"{requirement.context}: {normalized_conclusion}")
    return findings


def _manifest_check_run_failures(
    head_oid: str,
    manifest: tuple[CheckRequirement, ...],
) -> list[str]:
    """Compatibility surface returning every blocking manifest finding."""
    findings = _manifest_check_run_findings(head_oid, manifest)
    return [*findings.hard, *findings.pending]


def _unharvested_copilot_comments(pr: int) -> list[str]:
    """Top-level Copilot inline comments with no reply."""
    comments = _json_dicts(
        _gh_json(
            "api",
            f"repos/{{owner}}/{{repo}}/pulls/{pr}/comments?per_page={MAX_GITHUB_ITEMS}",
        ),
        "pull-request comments",
        refuse_full_page=True,
    )
    replied_to = {c.get("in_reply_to_id") for c in comments}
    return [
        f"unaddressed Copilot comment {_bounded_error_detail(c.get('html_url'))}"
        for c in comments
        if _is_rest_copilot_comment(c.get("user"))
        and c.get("in_reply_to_id") is None
        and c["id"] not in replied_to
    ]


def _is_graphql_copilot_review(author: object) -> bool:
    """Match the exact Copilot identity emitted by ``gh pr view``."""
    if not isinstance(author, dict):
        return False
    return author.get("login") == COPILOT_GRAPHQL_LOGIN


def _is_canonical_rest_copilot(author: object, expected_login: str) -> bool:
    """Match a REST Copilot actor by endpoint login and immutable bot identity."""
    if not isinstance(author, dict):
        return False
    return (
        author.get("login") == expected_login
        and author.get("id") == COPILOT_BOT_ID
        and author.get("node_id") == COPILOT_BOT_NODE_ID
        and author.get("type") == "Bot"
    )


def _is_rest_copilot_review(author: object) -> bool:
    return _is_canonical_rest_copilot(author, COPILOT_REST_REVIEW_LOGIN)


def _is_rest_copilot_comment(author: object) -> bool:
    return _is_canonical_rest_copilot(author, COPILOT_REST_COMMENT_LOGIN)


def _rest_reviews(pr: int) -> list[dict[str, object]]:
    return _json_dicts(
        _gh_json(
            "api",
            f"repos/{{owner}}/{{repo}}/pulls/{pr}/reviews?per_page={MAX_GITHUB_ITEMS}",
        ),
        "REST pull-request reviews",
        refuse_full_page=True,
    )


def _copilot_evidence_error(
    context: str,
    error: RuntimeError | KeyError | TypeError,
) -> CopilotEvidenceReadError:
    """Convert one Copilot-only read failure into bounded typed evidence."""
    detail = _bounded_error_detail(str(error)) or type(error).__name__
    return CopilotEvidenceReadError(f"{context} unavailable: {detail}")


def _copilot_advisories(pr: int, graphql_reviews: object, head_oid: str) -> list[str]:
    """Return non-blocking Copilot review and comment evidence."""
    advisories: list[str] = []
    try:
        rest_reviews = _rest_reviews(pr)
        completed = _has_completed_copilot_review(graphql_reviews, rest_reviews, head_oid)
    except (RuntimeError, KeyError, TypeError) as error:
        advisories.append(str(_copilot_evidence_error("Copilot review evidence", error)))
    else:
        if not completed:
            advisories.append("no completed Copilot review on verified head")
    try:
        advisories.extend(_unharvested_copilot_comments(pr))
    except (RuntimeError, KeyError, TypeError) as error:
        advisories.append(str(_copilot_evidence_error("Copilot comment evidence", error)))
    return advisories


def _has_completed_copilot_review(
    graphql_reviews: object,
    rest_reviews: list[dict[str, object]],
    head_oid: str,
) -> bool:
    """Whether both GitHub views identify Copilot's completed exact-head review."""
    graphql_match = False
    for review in _json_dicts(graphql_reviews, "GraphQL pull-request reviews"):
        commit = review.get("commit")
        commit_oid = commit.get("oid") if isinstance(commit, dict) else None
        if (
            _is_graphql_copilot_review(review.get("author"))
            and review.get("submittedAt")
            and review.get("state") in COMPLETED_REVIEW_STATES
            and commit_oid == head_oid
        ):
            graphql_match = True
    rest_match = any(
        _is_rest_copilot_review(review.get("user"))
        and review.get("submitted_at")
        and review.get("state") in COMPLETED_REVIEW_STATES
        and review.get("commit_id") == head_oid
        for review in rest_reviews
    )
    return graphql_match and rest_match


def _review_thread_problems(pr: int) -> list[str]:
    """Query at most 100 threads and report every unresolved one."""
    response = _json_dict(
        _gh_json(
            "api",
            "graphql",
            "-F",
            "owner={owner}",
            "-F",
            "name={repo}",
            "-F",
            f"number={pr}",
            "-f",
            f"query={_REVIEW_THREADS_QUERY}",
        ),
        "review-thread response",
    )
    data = _json_dict(response.get("data"), "review-thread data")
    repository = _json_dict(data.get("repository"), "review-thread repository")
    pull_request = _json_dict(repository.get("pullRequest"), "review-thread pull request")
    connection = _json_dict(pull_request.get("reviewThreads"), "review-thread connection")
    page_info = _json_dict(connection.get("pageInfo"), "review-thread page info")
    has_next_page = page_info.get("hasNextPage")
    if not isinstance(has_next_page, bool):
        raise RuntimeError("review-thread hasNextPage must be a boolean")
    problems = (
        ["more than 100 review threads — bounded verification refused"] if has_next_page else []
    )
    for node in _json_dicts(connection.get("nodes"), "review-thread nodes"):
        is_resolved = node.get("isResolved")
        if not isinstance(is_resolved, bool):
            raise RuntimeError("review-thread isResolved must be a boolean")
        if not is_resolved:
            problems.append(f"unresolved review thread {_first_thread_url(node)}")
    return problems


def _first_thread_url(thread: dict[str, object]) -> str:
    """Return the first comment URL identifying a review thread."""
    comments = _json_dict(thread.get("comments"), "review-thread comments")
    nodes = _json_dicts(comments.get("nodes"), "review-thread comment nodes")
    if not nodes:
        return "(URL unavailable)"
    return str(nodes[0].get("url") or "(URL unavailable)")


def _open_alert_count() -> int:
    return len(
        _json_dicts(
            _gh_json(
                "api",
                "repos/{owner}/{repo}/code-scanning/alerts"
                f"?state=open&per_page={MAX_GITHUB_ITEMS}",
            ),
            "code-scanning alerts",
            refuse_full_page=True,
        )
    )


def _child_prs(head_ref: str) -> list[int]:
    prs = _json_dicts(
        _gh_json(
            "pr",
            "list",
            "--state",
            "open",
            "--base",
            head_ref,
            "--limit",
            str(MAX_GITHUB_ITEMS),
            "--json",
            "number",
        ),
        "child pull requests",
        refuse_full_page=True,
    )
    numbers: list[int] = []
    for pr in prs:
        number = pr.get("number")
        if not isinstance(number, int):
            raise RuntimeError("child pull-request number must be an integer")
        numbers.append(number)
    return numbers


def _dependabot_pr_problems(
    pr: int,
    head_oid: str,
    source_run_id: int,
    classifier_run_id: int,
) -> DependabotModeResult:
    """Revalidate exact-head Dependabot authority from trusted GitHub records."""
    payload = _json_dict(
        _gh_json("api", f"repos/{{owner}}/{{repo}}/pulls/{pr}"),
        "Dependabot pull request",
    )
    problems: list[str] = []
    user = payload.get("user")
    if not _is_canonical_dependabot(user):
        problems.append("Dependabot mode requires the exact Dependabot bot author")
    head = payload.get("head")
    rest_head = head.get("sha") if isinstance(head, dict) else None
    if rest_head != head_oid:
        problems.append(f"Dependabot head moved: expected {head_oid}, found {rest_head}")
    head_ref = head.get("ref") if isinstance(head, dict) else None
    base = payload.get("base")
    rest_base = base.get("ref") if isinstance(base, dict) else None
    if rest_base != "dev":
        problems.append(f"Dependabot mode requires base dev, found {rest_base}")
    if problems:
        return DependabotModeResult(tuple(problems))
    source_run = _json_dict(
        _gh_json("api", f"repos/{{owner}}/{{repo}}/actions/runs/{source_run_id}"),
        "Dependabot source CI run",
    )
    source_problems = _source_ci_run_problems(source_run, pr, head_oid, source_run_id)
    if source_problems:
        return DependabotModeResult(tuple(source_problems))
    classifier_run = _json_dict(
        _gh_json("api", f"repos/{{owner}}/{{repo}}/actions/runs/{classifier_run_id}"),
        "Dependabot classifier run",
    )
    classifier_problems = _classifier_run_problems(
        classifier_run,
        head_oid,
        source_run_id,
        classifier_run_id,
    )
    if classifier_problems:
        return DependabotModeResult(tuple(classifier_problems))
    job_problems = _native_classifier_job_problems(classifier_run_id, classifier_run)
    if job_problems:
        return DependabotModeResult(tuple(job_problems))
    return _dependabot_update_result(
        pr,
        head_oid,
        head_ref if isinstance(head_ref, str) else "",
    )


def _is_canonical_dependabot(actor: object) -> bool:
    """Match immutable Dependabot identity at REST and Actions boundaries."""
    return isinstance(actor, dict) and (
        actor.get("login") == DEPENDABOT_LOGIN
        and actor.get("id") == DEPENDABOT_BOT_ID
        and actor.get("type") == "Bot"
    )


def _positive_int(value: object, context: str) -> int:
    """Require one positive JSON integer identifier."""
    if type(value) is not int or value <= 0:
        raise RuntimeError(f"{context} must be a positive integer")
    return value


def _source_ci_run_problems(
    run: dict[str, object],
    pr: int,
    head_oid: str,
    run_id: int,
) -> list[str]:
    """Bind authorization to the canonical native exact-head CI run."""
    problems: list[str] = []
    if run.get("id") != run_id or run.get("workflow_id") != CI_WORKFLOW_ID:
        problems.append("Dependabot source: wrong CI workflow identity")
    if run.get("path") != CI_WORKFLOW_PATH or run.get("event") != "pull_request":
        problems.append("Dependabot source: wrong CI workflow path or event")
    if run.get("head_sha") != head_oid:
        problems.append("Dependabot source: CI run is not on exact PR head")
    if run.get("status") != "completed" or run.get("conclusion") != "success":
        problems.append("Dependabot source: CI run did not complete successfully")
    if not _is_canonical_dependabot(run.get("actor")):
        problems.append("Dependabot source: wrong CI workflow actor")
    if not _is_canonical_dependabot(run.get("triggering_actor")):
        problems.append("Dependabot source: wrong CI triggering actor")
    problems.extend(_run_repository_problems(run, "Dependabot source"))
    problems.extend(_run_pr_identity_problems(run, pr, head_oid, "Dependabot source"))
    suite_id = _positive_int(run.get("check_suite_id"), "Dependabot source check_suite_id")
    suite = _json_dict(
        _gh_json("api", f"repos/{{owner}}/{{repo}}/check-suites/{suite_id}"),
        "Dependabot source check suite",
    )
    problems.extend(_source_suite_problems(suite, pr, head_oid, suite_id))
    return problems


def _classifier_run_problems(
    run: dict[str, object],
    head_oid: str,
    source_run_id: int,
    classifier_run_id: int,
) -> list[str]:
    """Bind the verifier to its trusted default-branch workflow run."""
    expected_title = f"Dependabot eligibility for CI run {source_run_id} at {head_oid}"
    problems: list[str] = []
    if run.get("id") != classifier_run_id or run.get("workflow_id") != DEPENDABOT_WORKFLOW_ID:
        problems.append("Dependabot classifier: wrong workflow identity")
    if run.get("path") != DEPENDABOT_WORKFLOW_PATH or run.get("event") != "workflow_run":
        problems.append("Dependabot classifier: wrong workflow path or event")
    if run.get("display_title") != expected_title:
        problems.append("Dependabot classifier: source run/head binding mismatch")
    if run.get("status") != "in_progress" or run.get("conclusion") is not None:
        problems.append("Dependabot classifier: workflow run is not currently executing")
    if not _is_canonical_dependabot(run.get("actor")):
        problems.append("Dependabot classifier: wrong workflow actor")
    if not _is_canonical_dependabot(run.get("triggering_actor")):
        problems.append("Dependabot classifier: wrong triggering actor")
    problems.extend(_run_repository_problems(run, "Dependabot classifier"))
    return problems


def _native_classifier_job_problems(
    classifier_run_id: int,
    classifier_run: dict[str, object],
) -> list[str]:
    """Require the verifier's native job/check and owning check suite."""
    run_attempt = _positive_int(
        classifier_run.get("run_attempt"),
        "Dependabot classifier run_attempt",
    )
    payload = _json_dict(
        _gh_json(
            "api",
            f"repos/{{owner}}/{{repo}}/actions/runs/{classifier_run_id}"
            f"/attempts/{run_attempt}/jobs?filter=latest&per_page={MAX_GITHUB_ITEMS}",
        ),
        "Dependabot classifier jobs",
    )
    jobs = _bounded_counted_items(payload, "Dependabot classifier jobs", "jobs")
    matches = [job for job in jobs if job.get("name") == DEPENDABOT_ELIGIBILITY_CHECK]
    if len(matches) != 1:
        return [f"{DEPENDABOT_ELIGIBILITY_CHECK}: expected one native classifier job"]
    job = matches[0]
    job_id = _positive_int(job.get("id"), "Dependabot classifier job id")
    check_run = _json_dict(
        _gh_json("api", f"repos/{{owner}}/{{repo}}/check-runs/{job_id}"),
        "Dependabot classifier check run",
    )
    suite_id = _positive_int(
        classifier_run.get("check_suite_id"),
        "Dependabot classifier check_suite_id",
    )
    return _classifier_job_and_check_problems(
        job,
        check_run,
        classifier_run,
        classifier_run_id,
        suite_id,
    )


def _classifier_job_and_check_problems(
    job: dict[str, object],
    check_run: dict[str, object],
    run: dict[str, object],
    run_id: int,
    suite_id: int,
) -> list[str]:
    """Validate the native job/check equality and check-suite ownership."""
    job_id = _positive_int(job.get("id"), "Dependabot classifier job id")
    problems: list[str] = []
    if job.get("run_id") != run_id or job.get("head_sha") != run.get("head_sha"):
        problems.append(f"{DEPENDABOT_ELIGIBILITY_CHECK}: native job/run mismatch")
    if job.get("status") != "in_progress" or job.get("conclusion") is not None:
        problems.append(f"{DEPENDABOT_ELIGIBILITY_CHECK}: native job is not executing")
    if check_run.get("id") != job_id or check_run.get("name") != job.get("name"):
        problems.append(f"{DEPENDABOT_ELIGIBILITY_CHECK}: native job/check ID mismatch")
    if check_run.get("head_sha") != job.get("head_sha"):
        problems.append(f"{DEPENDABOT_ELIGIBILITY_CHECK}: native job/check head mismatch")
    if check_run.get("status") != job.get("status") or check_run.get("conclusion") != job.get(
        "conclusion"
    ):
        problems.append(f"{DEPENDABOT_ELIGIBILITY_CHECK}: native job/check result mismatch")
    app = check_run.get("app")
    if not _is_actions_app(app):
        problems.append(f"{DEPENDABOT_ELIGIBILITY_CHECK}: wrong Actions app identity")
    suite_ref = _json_dict(check_run.get("check_suite"), "classifier check suite reference")
    if suite_ref.get("id") != suite_id:
        problems.append(f"{DEPENDABOT_ELIGIBILITY_CHECK}: wrong check suite")
    return problems


def _source_suite_problems(
    suite: dict[str, object],
    pr: int,
    head_oid: str,
    suite_id: int,
) -> list[str]:
    """Validate the exact-head native CI check-suite association."""
    problems: list[str] = []
    if suite.get("id") != suite_id or suite.get("head_sha") != head_oid:
        problems.append("Dependabot source: check suite is not on exact head")
    if not _is_actions_app(suite.get("app")):
        problems.append("Dependabot source: wrong check-suite app identity")
    problems.extend(_run_pr_identity_problems(suite, pr, head_oid, "Dependabot source suite"))
    return problems


def _is_actions_app(app: object) -> bool:
    """Match the canonical GitHub Actions app identity."""
    return isinstance(app, dict) and (
        app.get("id") == GITHUB_ACTIONS_APP_ID and app.get("slug") == GITHUB_ACTIONS_APP_SLUG
    )


def _run_repository_problems(run: dict[str, object], context: str) -> list[str]:
    """Require the source and executing repository identities to agree."""
    head_repository = _json_dict(run.get("head_repository"), f"{context} head repository")
    repository = _json_dict(run.get("repository"), f"{context} repository")
    if head_repository.get("full_name") != repository.get("full_name"):
        return [f"{context}: repository mismatch"]
    return []


def _run_pr_identity_problems(
    payload: dict[str, object],
    pr: int,
    head_oid: str,
    context: str,
) -> list[str]:
    """Require one exact PR/head/base association on a run or suite."""
    pulls = _json_dicts(
        payload.get("pull_requests"),
        f"{context} pull requests",
        refuse_full_page=True,
    )
    matches = [pull for pull in pulls if _pull_matches(pull, pr, head_oid)]
    return [] if len(pulls) == 1 and len(matches) == 1 else [f"{context}: PR identity mismatch"]


def _pull_matches(pull: dict[str, object], pr: int, head_oid: str) -> bool:
    """Match one exact PR number, head SHA, and dev base."""
    head = pull.get("head")
    base = pull.get("base")
    return (
        pull.get("number") == pr
        and isinstance(head, dict)
        and head.get("sha") == head_oid
        and isinstance(base, dict)
        and base.get("ref") == "dev"
    )


def _bounded_counted_items(
    payload: dict[str, object],
    context: str,
    key: str,
) -> list[dict[str, object]]:
    """Validate a bounded GitHub total_count/list response."""
    total_count = payload.get("total_count")
    if type(total_count) is not int:
        raise RuntimeError(f"{context} total_count must be an integer")
    if total_count >= MAX_GITHUB_ITEMS:
        raise RuntimeError(f"{context} reached the {MAX_GITHUB_ITEMS}-item safety bound")
    items = _json_dicts(payload.get(key), context, refuse_full_page=True)
    if total_count != len(items):
        raise RuntimeError(f"{context} response is incomplete")
    return items


_CARGO_DEPENDABOT_REF_PREFIX: Final = "dependabot/cargo/"
# SemVer token: numeric core plus optional -prerelease and +build suffixes so
# suffixed versions (e.g. 0.10.0-alpha.1) cannot escape the 0.x guard.
_CARGO_SEMVER_PATTERN: Final = (
    r"(\d+)\.(\d+)\.(\d+)"
    r"(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
    r"(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
)
_CARGO_VERSION_BUMP_RE: Final = re.compile(
    rf"\bfrom\s+{_CARGO_SEMVER_PATTERN}\s+to\s+{_CARGO_SEMVER_PATTERN}\b"
)


def _cargo_zero_x_minor(subject: str) -> bool:
    """Cargo 0.y → 0.(y+1) is breaking-class even when Dependabot tags it minor.

    Dependabot's update-type trailer compares only the leading version
    component, so 0.10.0 → 0.11.0 arrives as ``semver-minor`` (#874). Group
    PRs carry no per-entry before-version in the commit subject, so this guard
    covers the single-dependency bump subject only.
    """
    match = _CARGO_VERSION_BUMP_RE.search(subject)
    if match is None:
        return False
    before_major, before_minor = int(match.group(1)), int(match.group(2))
    after_major, after_minor = int(match.group(4)), int(match.group(5))
    return before_major == 0 and after_major == 0 and after_minor > before_minor


def _update_types_from_message(message: str) -> list[str]:
    """Parse Dependabot update-type trailers from one commit message."""
    update_types = re.findall(r"(?m)^\s*update-type:\s*([^\s]+)\s*$", message)
    if not update_types or len(update_types) >= MAX_GITHUB_ITEMS:
        raise RuntimeError("Dependabot update metadata is missing or exceeds the safety bound")
    allowed = {
        "version-update:semver-patch",
        "version-update:semver-minor",
        "version-update:semver-major",
    }
    if any(update_type not in allowed for update_type in update_types):
        raise RuntimeError("Dependabot update metadata is not a known semver class")
    return update_types


def _dependabot_update_result(
    pr: int,
    head_oid: str,
    head_ref: str,
) -> DependabotModeResult:
    """Classify the sole exact-head commit after native actor provenance."""
    commits = _json_dicts(
        _gh_json(
            "api",
            f"repos/{{owner}}/{{repo}}/pulls/{pr}/commits?per_page={MAX_GITHUB_ITEMS}",
        ),
        "Dependabot pull-request commits",
        refuse_full_page=True,
    )
    if len(commits) != 1:
        return DependabotModeResult(
            (f"Dependabot mode requires exactly one current commit, found {len(commits)}",)
        )
    commit = commits[0]
    if commit.get("sha") != head_oid:
        return DependabotModeResult(("Dependabot commit does not match the exact PR head",))
    details = _json_dict(commit.get("commit"), "Dependabot commit details")
    message = details.get("message")
    if not isinstance(message, str):
        raise RuntimeError("Dependabot commit message must be a string")
    update_types = _update_types_from_message(message)
    subject = message.split("\n", 1)[0]
    if head_ref.startswith(_CARGO_DEPENDABOT_REF_PREFIX) and _cargo_zero_x_minor(subject):
        return DependabotModeResult(
            semver_major=True,
            semver_major_reason="Cargo 0.x minor (breaking-class) Dependabot",
        )
    return DependabotModeResult(
        semver_major="version-update:semver-major" in update_types,
    )


def _base_policy_problems(
    base_ref: str,
    head_ref: str,
    *,
    director_main: bool,
    delete_branch: bool,
) -> list[str]:
    """Enforce the ordinary dev lane and Director-only main lane."""
    problems: list[str] = []
    if director_main:
        if base_ref != "main":
            problems.append(f"--director-main requires base main, found {base_ref}")
        elif head_ref != "dev" and not head_ref.startswith("fix/"):
            problems.append(f"main requires head dev or fix/*, found {head_ref}")
    elif base_ref != "dev":
        problems.append(f"base branch {base_ref} is not dev")
    if delete_branch and head_ref == "dev":
        problems.append("--delete-branch refused for protected branch dev")
    return problems


def _dependabot_mode_problems(
    pr: int,
    head_oid: str,
    source_run_id: int | None,
    classifier_run_id: int | None,
) -> DependabotModeResult:
    """Validate required run identifiers before Dependabot provenance reads."""
    if source_run_id is None or classifier_run_id is None:
        return DependabotModeResult(
            ("--dependabot requires source and classifier workflow run IDs",)
        )
    _positive_int(source_run_id, "Dependabot source run ID")
    _positive_int(classifier_run_id, "Dependabot classifier run ID")
    return _dependabot_pr_problems(pr, head_oid, source_run_id, classifier_run_id)


def _merge_pr(pr: int, head_oid: str, *, delete_branch: bool) -> str:
    """Merge once, then reconcile a failed command without retrying it."""
    merge_args = [
        "pr",
        "merge",
        str(pr),
        "--merge",
        "--match-head-commit",
        head_oid,
    ]
    if delete_branch:
        merge_args.append("--delete-branch")
    try:
        return _run_gh(*merge_args).stdout
    except (subprocess.TimeoutExpired, subprocess.CalledProcessError, OSError) as command_error:
        command_failure = _gh_failure_message(command_error)
    try:
        reconciliation = _json_dict(
            _gh_json(
                "pr",
                "view",
                str(pr),
                "--json",
                "state,headRefOid,mergeCommit",
            ),
            "merge reconciliation",
        )
    except RuntimeError as reconciliation_error:
        raise MergeIndeterminateError(
            f"{command_failure}; reconciliation failed: {reconciliation_error}"
        ) from None
    merge_commit = reconciliation.get("mergeCommit")
    merge_oid = merge_commit.get("oid") if isinstance(merge_commit, dict) else None
    confirmed = (
        reconciliation.get("state") == "MERGED"
        and reconciliation.get("headRefOid") == head_oid
        and isinstance(merge_oid, str)
        and bool(merge_oid)
    )
    if not confirmed:
        raise MergeIndeterminateError(
            f"{command_failure}; reconciliation did not confirm the exact-head merge"
        ) from None
    if delete_branch:
        print(
            "pr:merge WARNING — merge confirmed after command failure; "
            "branch deletion may be incomplete",
            file=sys.stderr,
        )
    return ""


def _main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pr", type=int)
    parser.add_argument("--delete-branch", action="store_true")
    parser.add_argument("--verify-only", action="store_true")
    parser.add_argument("--expected-head")
    parser.add_argument("--director-main", action="store_true")
    parser.add_argument("--dependabot", action="store_true")
    parser.add_argument("--dependabot-source-run", type=int)
    parser.add_argument("--dependabot-classifier-run", type=int)
    args = parser.parse_args()

    view = _json_dict(
        _gh_json(
            "pr",
            "view",
            str(args.pr),
            "--json",
            "state,isDraft,headRefOid,headRefName,baseRefOid,baseRefName,statusCheckRollup,reviews",
        ),
        "pull-request snapshot",
    )
    head_oid = str(view.get("headRefOid") or "")
    head_ref = str(view.get("headRefName") or "")
    base_oid = str(view.get("baseRefOid") or "")
    base_ref = str(view.get("baseRefName") or "")
    findings = VerificationFindings()
    dependabot_result = DependabotModeResult()
    if view.get("state") != "OPEN" or view.get("isDraft"):
        findings.hard.append(f"PR #{args.pr} is {view.get('state')}, draft={view.get('isDraft')}")
    findings.hard.extend(
        _base_policy_problems(
            base_ref,
            head_ref,
            director_main=args.director_main,
            delete_branch=args.delete_branch,
        )
    )
    if args.expected_head is not None and args.expected_head != head_oid:
        findings.hard.append(f"expected head {args.expected_head}, found {head_oid}")
    rollup = view.get("statusCheckRollup") or []
    manifest = manifest_for_base(base_ref) if base_ref in {"dev", "main"} else ()
    if manifest:
        findings.extend(
            _manifest_check_run_findings(
                head_oid,
                manifest,
                allow_pending=args.dependabot,
            )
        )
    findings.extend(
        _rollup_findings(
            _json_dicts(rollup, "status-check rollup"),
            manifest,
            allow_pending=args.dependabot,
        )
    )
    advisories = _copilot_advisories(args.pr, view.get("reviews"), head_oid)
    findings.hard.extend(_review_thread_problems(args.pr))
    if (alerts := _open_alert_count()) > 0:
        findings.hard.append(f"{alerts} open code-scanning alert(s) — the zero floor is a STOP")
    if args.delete_branch and (children := _child_prs(head_ref)):
        findings.hard.append(
            f"--delete-branch refused: open PR(s) {children} base on this branch (#193 class)"
        )
    if args.dependabot:
        dependabot_result = _dependabot_mode_problems(
            args.pr,
            head_oid,
            args.dependabot_source_run,
            args.dependabot_classifier_run,
        )
        findings.hard.extend(dependabot_result.problems)

    # Race guard: neither side may move after its verdict snapshot.
    refs_now = _json_dict(
        _gh_json("pr", "view", str(args.pr), "--json", "headRefOid,baseRefOid"),
        "pull-request ref re-read",
    )
    if refs_now.get("headRefOid") != head_oid:
        findings.hard.append("head moved while verifying — re-run against the new head")
    if refs_now.get("baseRefOid") != base_oid:
        findings.hard.append("base moved while verifying — re-run against the new base")

    for advisory in advisories:
        print(f"pr:merge ADVISORY — {advisory}", file=sys.stderr)
    if findings.hard:
        for problem in findings.hard:
            print(f"pr:merge REFUSED — {problem}", file=sys.stderr)
        for pending in findings.pending:
            print(f"pr:merge PENDING — {pending}", file=sys.stderr)
        return MergeOutcome.HARD_REFUSAL

    if dependabot_result.semver_major:
        print(
            f"pr:merge MANUAL REVIEW — {dependabot_result.semver_major_reason} "
            f"PR #{args.pr}; unattended merge skipped"
        )
        return MergeOutcome.DEPENDABOT_MAJOR_REVIEW

    if findings.pending:
        for pending in findings.pending:
            print(f"pr:merge PENDING — {pending}", file=sys.stderr)
        return MergeOutcome.EXACT_HEAD_EVIDENCE_PENDING

    if args.verify_only:
        print(f"pr:merge: PR #{args.pr} verified at {head_oid}")
        return MergeOutcome.SUCCESS

    print(_merge_pr(args.pr, head_oid, delete_branch=args.delete_branch), end="")
    print(f"pr:merge: PR #{args.pr} merged at {head_oid}")
    return MergeOutcome.SUCCESS


def main() -> int:
    """Map typed read and mutation outcomes onto the stable CLI surface."""
    try:
        return _main()
    except MergeIndeterminateError as error:
        print(f"pr:merge INDETERMINATE — {error}", file=sys.stderr)
        return MergeOutcome.MUTATION_INDETERMINATE
    except RuntimeError as error:
        print(f"pr:merge REFUSED — {error}", file=sys.stderr)
        return MergeOutcome.HARD_REFUSAL


if __name__ == "__main__":
    raise SystemExit(main())
