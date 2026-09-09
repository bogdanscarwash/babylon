#!/usr/bin/env python3
"""Serial Dependabot merge conveyor: ``mise run deps:conveyor``.

Mechanizes the backlog-clearing loop that was twice rebuilt by hand in
``/tmp`` (2026-09-02 sessions): open Dependabot PRs serialize against each
other because every merge to ``dev`` stales the rest, so a human babysitting
``pr:merge`` one PR at a time wastes an hour per pass.

Each pass:

1. ``CLEAN`` PRs are classified from the single head-commit message
   (``update-type`` trailers plus the 0.x-minor rule shared with
   ``tools/pr_merge.py``); major or breaking updates park for manual review
   for the rest of the session, the rest merge via the one sanctioned path
   (``tools/pr_merge.py``), then the pass restarts — the merge staled every
   sibling, so the queue is re-listed and the remaining count is exact.
2. ``BEHIND`` but ``MERGEABLE`` PRs get one ``@dependabot rebase`` comment
   per head SHA. A raw ``update-branch`` API call would commit under the
   caller and poison the source-run actor the unattended verifier requires;
   the Dependabot-native rebase preserves single-commit provenance.
3. ``pr_merge.py`` outcomes are honored, not flattened: exit 1 (hard
   refusal) and exit 2 (mutation indeterminate) stop the conveyor; exit 3
   (manual review) parks the PR for the rest of the session; exit 4
   (evidence pending) retries on the next pass.

The loop exits 0 when no Dependabot PRs remain open and 1 when ``--max-passes``
is reached with PRs still open. A terminal pr:merge outcome exits 2. It never
force-merges, never resolves conflicts itself, and never retries a terminal
outcome within the session.

Stdlib + ``gh`` only, same contract as ``tools/pr_merge.py``.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Final

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools.pr_merge import (  # noqa: E402
    _CARGO_DEPENDABOT_REF_PREFIX,
    MAX_GITHUB_ITEMS,
    MergeOutcome,
    _cargo_zero_x_minor,
    _update_types_from_message,
)

GH_TIMEOUT_SECONDS: Final[int] = 30
PR_MERGE_TIMEOUT_SECONDS: Final[int] = 300
REBASE_BODY: Final[str] = "@dependabot rebase"
LIST_FIELDS: Final[str] = "number,mergeStateStatus,mergeable,headRefOid,headRefName"
NAME_WITH_OWNER_PATTERN: Final[str] = r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+"


class ConveyorStop(RuntimeError):
    """A terminal pr:merge outcome (exit 1 or 2) ends the conveyor."""


@dataclass(frozen=True)
class OpenPr:
    number: int
    state: str
    mergeable: str
    head_oid: str
    head_ref: str


@dataclass
class ConveyorState:
    """Mutable per-session conveyor state."""

    rebase_requested: dict[int, str] = field(default_factory=dict)
    manual_review: set[int] = field(default_factory=set)


@dataclass(frozen=True)
class UpdateClass:
    manual_review: bool
    reason: str


def _run_gh(args: list[str]) -> subprocess.CompletedProcess[str]:
    """Run one GitHub CLI command with the shared fixed timeout."""
    try:
        return subprocess.run(
            ["gh", *args],
            capture_output=True,
            text=True,
            check=False,
            timeout=GH_TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired as error:
        raise RuntimeError(f"gh {' '.join(args)} timed out after {GH_TIMEOUT_SECONDS}s") from error


def resolve_repo() -> str:
    """Resolve the current repository once; fail loudly when unavailable."""
    proc = _run_gh(["repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"])
    if proc.returncode != 0:
        detail = proc.stderr.strip()[:400] or f"exit {proc.returncode}"
        raise RuntimeError(f"gh repo view failed: {detail}")
    name = proc.stdout.strip()
    if re.fullmatch(NAME_WITH_OWNER_PATTERN, name) is None:
        raise RuntimeError(f"gh repo view returned malformed nameWithOwner: {name!r}")
    return name


def gh_json(args: list[str]) -> list[dict[str, object]]:
    proc = _run_gh(args)
    if proc.returncode != 0:
        raise RuntimeError(f"gh {' '.join(args)} failed: {proc.stderr.strip()[:400]}")
    payload = json.loads(proc.stdout)
    if not isinstance(payload, list):
        raise RuntimeError(f"gh {' '.join(args)} returned non-list payload")
    return payload


def list_open_prs(repo: str) -> list[OpenPr]:
    rows = gh_json(
        [
            "pr",
            "list",
            "--repo",
            repo,
            "--state",
            "open",
            "--author",
            "app/dependabot",
            "--limit",
            "30",
            "--json",
            LIST_FIELDS,
        ]
    )
    return [
        OpenPr(
            number=int(row["number"]),
            state=str(row["mergeStateStatus"]),
            mergeable=str(row["mergeable"]),
            head_oid=str(row["headRefOid"]),
            head_ref=str(row["headRefName"]),
        )
        for row in rows
    ]


def fetch_commit_messages(repo: str, number: int) -> list[str]:
    """Fetch every commit message on one PR for pre-merge classification."""
    rows = gh_json(["api", f"repos/{repo}/pulls/{number}/commits"])
    if len(rows) >= MAX_GITHUB_ITEMS:
        raise RuntimeError(f"pull-request {number} commits reached the safety bound")
    messages: list[str] = []
    for row in rows:
        details = row.get("commit")
        message = details.get("message") if isinstance(details, dict) else None
        if not isinstance(message, str):
            raise RuntimeError(f"pull-request {number} commit message must be a string")
        messages.append(message)
    return messages


def classify_update(commit_message: str, head_ref: str) -> UpdateClass:
    """Classify one Dependabot head commit before any merge is attempted."""
    if head_ref.startswith(_CARGO_DEPENDABOT_REF_PREFIX) and _cargo_zero_x_minor(
        commit_message.split("\n", 1)[0]
    ):
        return UpdateClass(
            manual_review=True,
            reason="Cargo 0.x minor (breaking-class) Dependabot bump",
        )
    update_types = _update_types_from_message(commit_message)
    if "version-update:semver-major" in update_types:
        return UpdateClass(
            manual_review=True,
            reason="authenticated update-type is version-update:semver-major",
        )
    return UpdateClass(manual_review=False, reason="")


def should_request_rebase(state: ConveyorState, number: int, head_oid: str) -> bool:
    """One rebase request per head SHA — skip when the head has not moved."""
    return state.rebase_requested.get(number) != head_oid


def request_rebase(repo: str, number: int) -> bool:
    """Ask Dependabot to rebase, preserving single-commit provenance."""
    proc = _run_gh(["pr", "comment", str(number), "--repo", repo, "--body", REBASE_BODY])
    if proc.returncode != 0:
        print(f"~~~ rebase request {number} failed: {proc.stderr.strip()[:400]}")
        return False
    print(f"~~~ rebase request {number} accepted")
    return True


def try_merge(number: int) -> MergeOutcome:
    """One sanctioned merge attempt; returns the typed pr:merge outcome."""
    try:
        proc = subprocess.run(
            [sys.executable, "tools/pr_merge.py", str(number)],
            capture_output=True,
            text=True,
            check=False,
            timeout=PR_MERGE_TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired as error:
        raise RuntimeError(
            f"pr:merge {number} timed out after {PR_MERGE_TIMEOUT_SECONDS}s"
        ) from error
    output = (proc.stdout + proc.stderr).strip()
    print(f"~~~ pr:merge {number} exit={proc.returncode}")
    for line in output.splitlines():
        print(f"    {line}")
    try:
        return MergeOutcome(proc.returncode)
    except ValueError as error:
        raise RuntimeError(f"pr:merge {number} returned unknown exit {proc.returncode}") from error


def conveyor_pass(repo: str, state: ConveyorState) -> tuple[int, bool]:
    """One sweep. Returns (open count, merged something this pass)."""
    prs = list_open_prs(repo)
    if not prs:
        return 0, False
    summary = ", ".join(f"{p.number}:{p.state}" for p in prs)
    print(f"=== conveyor pass {time.strftime('%Y-%m-%dT%H:%M:%S%z')}: {summary}")
    for pr in prs:
        if pr.number in state.manual_review:
            print(f"~~~ {pr.number} parked for manual review this session — skipping")
            continue
        if pr.state == "CLEAN":
            messages = fetch_commit_messages(repo, pr.number)
            if len(messages) != 1:
                state.manual_review.add(pr.number)
                print(f"~~~ {pr.number} has {len(messages)} commits — parked for manual review")
                continue
            classification = classify_update(messages[0], pr.head_ref)
            if classification.manual_review:
                state.manual_review.add(pr.number)
                print(f"~~~ {pr.number} parked for manual review — {classification.reason}")
                continue
            outcome = try_merge(pr.number)
            if outcome == MergeOutcome.SUCCESS:
                print(f"~~~ MERGED {pr.number}")
                return len(list_open_prs(repo)), True
            if outcome in (MergeOutcome.HARD_REFUSAL, MergeOutcome.MUTATION_INDETERMINATE):
                raise ConveyorStop(f"pr:merge {pr.number} returned terminal exit {outcome.name}")
            if outcome == MergeOutcome.DEPENDABOT_MAJOR_REVIEW:
                state.manual_review.add(pr.number)
                print(f"~~~ {pr.number} requires manual review — parked for this session")
                continue
            print(f"~~~ pr:merge {pr.number} pending exact-head evidence — retry next pass")
        elif pr.state == "BEHIND" and pr.mergeable == "MERGEABLE":
            if should_request_rebase(state, pr.number, pr.head_oid):
                if request_rebase(repo, pr.number):
                    state.rebase_requested[pr.number] = pr.head_oid
        else:
            print(f"~~~ {pr.number} state={pr.state} mergeable={pr.mergeable} — waiting")
    return len(prs), False


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--interval", type=int, default=120, help="seconds between passes")
    parser.add_argument(
        "--max-passes",
        type=int,
        default=0,
        help="stop after N passes even if PRs remain (0 = run until empty)",
    )
    args = parser.parse_args()

    try:
        repo = resolve_repo()
    except RuntimeError as error:
        print(f"=== conveyor: {error}", file=sys.stderr)
        return 2

    state = ConveyorState()
    passes = 0
    while True:
        try:
            remaining, merged = conveyor_pass(repo, state)
        except ConveyorStop as stop:
            print(f"=== conveyor: TERMINAL — {stop}", file=sys.stderr)
            return 2
        passes += 1
        if remaining == 0:
            print(
                f"=== conveyor: no open dependabot PRs — done {time.strftime('%Y-%m-%dT%H:%M:%S%z')}"
            )
            return 0
        if args.max_passes and passes >= args.max_passes:
            print(f"=== conveyor: max-passes {args.max_passes} reached with {remaining} open")
            return 1
        if not merged:
            print(f"~~~ no merge this pass; sleeping {args.interval}s")
        time.sleep(args.interval)


if __name__ == "__main__":
    raise SystemExit(main())
