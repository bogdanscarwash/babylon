#!/usr/bin/env python3
r"""Baseline-ceremony gate — §6.5 Provenance & Ceremony (owner ruling 2026-07-20).

Constitution III.7 says "regenerate the baselines *and say so*"; III.12 makes
the baseline estate a behavioral-contract artifact. This gate makes "say so"
machine-checkable: any commit that touches ``tests/baselines/**`` is a
*ceremony* and must carry a declaration trailer::

    Baselines: blessed(<ceremony-slug>)

where the slug is a lowercase ``[a-z0-9-]`` name for the ceremony (convention:
``<what>-<date>``, e.g. ``post-merge-regen-2026-07-20``). The commit subject
stays the existing ``test(baselines): ...`` convention and the body records
the drift table; the trailer is the greppable provenance record::

    git log -E --grep '^Baselines: blessed\(' --format='%h %s'

Two modes (Constitution III.11 — both fail loudly):

``--commit-msg-file PATH``
    Local commit-msg hook: if staged paths touch the baseline estate, the
    commit message must declare the ceremony. Best-effort — an ``--amend``
    of an already-committed baseline change, or a pathspec commit
    (``git commit <path>``, which uses a temporary index), can slip past
    it; the range leg (below) catches both.

``--range A..B``
    Every commit in the range that touches the baseline estate must
    declare its ceremony. Merge commits are inspected for evil-merge
    content only (paths differing from ALL parents via ``diff-tree --cc``);
    clean merges need no trailer of their own. Run twice: locally at the
    ``pre-push`` git-hook stage (``.pre-commit-config.yaml``'s
    ``baseline-ceremony-range`` hook, against the merge-base with the
    upstream ``dev`` ref — closing the commit-msg leg's best-effort gap
    *before a push leaves the box*) and again in CI on every PR (the
    authoritative leg, diffing the PR's base..head). CI also passes
    ``--respect-introduction`` with exact endpoint SHAs: it requires the
    pinned introduction commit in head ancestry and excludes only its strict
    ancestors. The introduction, subsequent commits and later side-branch
    integrations remain checked. Missing history fails closed.

Authoring a ceremony message by hand is unnecessary: once the baseline
drift is staged, ``tools/generate_ceremony_message.py`` computes the
per-file drift table (row/cell counts, max |Δ| where CSV-parseable) and
emits a full ``subject``/body/trailer skeleton that this gate accepts by
construction.

Exit codes: 0 = clean, 1 = undeclared ceremony, 2 = usage or git failure.
Doctrine home: CLAUDE.md "Baseline ceremonies"; the ruling deliberately kept
this out of the constitution (workflow, not law).
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

# The governed artifact estate. Everything under these prefixes is a pinned
# behavioral-contract artifact (regression baselines, dense goldens, the
# mutation and storage-budget baselines); moving any of it is a ceremony.
BASELINE_PREFIXES: tuple[str, ...] = ("tests/baselines/",)

# This immutable commit introduced the machine-required trailer. Its strict
# ancestors remain historical evidence; the introduction itself is enforced.
CEREMONY_INTRODUCTION = "2655fbfa17e40acdb85fb7170098f4f909b21815"

# Strict trailer grammar: a full line, lowercase slug, no empty blessing.
_TRAILER_RE = re.compile(r"^Baselines: blessed\([a-z0-9][a-z0-9-]*\)$", re.MULTILINE)

_DOCTRINE = """\
tests/baselines/** changed without a ceremony declaration.

  If UNINTENDED: STOP — unexplained baseline drift is a red gate.
  Investigate before committing (Constitution III.7).

  If INTENTIONAL: this commit is a ceremony. Declare it:
    - subject:  test(baselines): <what moved and why>
    - body:     the drift table (per-scenario columns, cell counts, max |d|)
    - trailer:  Baselines: blessed(<ceremony-slug>)
  See CLAUDE.md "Baseline ceremonies".
"""


def _git(repo_root: Path, *args: str) -> str:
    """Run a git plumbing command, raising ``CalledProcessError`` on failure."""
    result = subprocess.run(
        ["git", *args],
        cwd=repo_root,
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout


def message_declares_ceremony(message: str) -> bool:
    """True iff the message carries a well-formed blessing trailer.

    Comment lines (``#``-prefixed, stripped by git's default cleanup) are
    removed first so a declaration cannot live only inside one.
    """
    effective = "\n".join(line for line in message.splitlines() if not line.startswith("#"))
    return _TRAILER_RE.search(effective) is not None


def touches_baselines(paths: list[str]) -> bool:
    """True iff any path lies inside the governed baseline estate."""
    return any(path.startswith(BASELINE_PREFIXES) for path in paths)


def check_commit_msg(msg_file: Path, repo_root: Path) -> list[str]:
    """commit-msg hook leg: staged baseline paths demand a declared message."""
    staged_raw = _git(repo_root, "diff", "--cached", "--name-only")
    staged = [line for line in staged_raw.splitlines() if line]
    if not touches_baselines(staged):
        return []
    if message_declares_ceremony(msg_file.read_text(encoding="utf-8")):
        return []
    hit = ", ".join(p for p in staged if p.startswith(BASELINE_PREFIXES))
    return [f"staged baseline change(s) without declaration: {hit}"]


def _is_merge_commit(sha: str, repo_root: Path) -> bool:
    parent_tokens = _git(repo_root, "rev-list", "--parents", "-n", "1", sha).split()
    return len(parent_tokens) > 2


def check_range(
    range_spec: str, repo_root: Path, *, respect_introduction: bool = False
) -> list[str]:
    """CI leg: every commit in the range must declare its ceremony.

    Non-merge commits are diffed against their parent. Merge commits are
    inspected with ``diff-tree --cc``, which lists only paths whose content
    differs from ALL parents — exactly evil-merge material. A clean merge
    yields no such paths and needs no trailer of its own; an evil merge
    that hand-resolves a baseline file is a ceremony in its own right.
    """
    exclusions: list[str] = []
    if respect_introduction:
        if re.fullmatch(r"[0-9a-f]{40}\.\.[0-9a-f]{40}", range_spec) is None:
            raise ValueError("introduction boundary requires an exact BASE_SHA..HEAD_SHA range")
        head = range_spec.split("..", 1)[1]
        _git(repo_root, "merge-base", "--is-ancestor", CEREMONY_INTRODUCTION, head)
        exclusions = ["--not", f"{CEREMONY_INTRODUCTION}^"]
    shas_raw = _git(repo_root, "rev-list", range_spec, *exclusions)
    violations: list[str] = []
    for sha in [line for line in shas_raw.splitlines() if line]:
        if _is_merge_commit(sha, repo_root):
            changed_raw = _git(repo_root, "diff-tree", "--cc", "--no-commit-id", "--name-only", sha)
        else:
            changed_raw = _git(repo_root, "diff-tree", "--no-commit-id", "--name-only", "-r", sha)
        changed = [line for line in changed_raw.splitlines() if line]
        if not touches_baselines(changed):
            continue
        message = _git(repo_root, "log", "-1", "--format=%B", sha)
        if message_declares_ceremony(message):
            continue
        subject = message.splitlines()[0] if message.splitlines() else ""
        violations.append(f"{sha[:12]} ({subject}): baseline change undeclared")
    return violations


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Baseline-ceremony gate — §6.5 Provenance & Ceremony"
    )
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument(
        "--commit-msg-file",
        type=Path,
        help="commit-msg hook mode: path to the message file under edit",
    )
    mode.add_argument(
        "--range",
        dest="range_spec",
        help="CI mode: git revision range (BASE..HEAD) to inspect",
    )
    parser.add_argument(
        "--respect-introduction",
        action="store_true",
        help="Enforce from the pinned trailer introduction, preserving its strict ancestors",
    )
    parser.add_argument(
        "--repo",
        type=Path,
        default=Path.cwd(),
        help="repository root (default: current directory)",
    )
    args = parser.parse_args(argv)

    if args.respect_introduction and args.range_spec is None:
        parser.error("--respect-introduction requires --range")

    if args.commit_msg_file is None and args.range_spec is None:
        parser.print_usage(sys.stderr)
        print("check_baseline_ceremony: one of --commit-msg-file/--range required", file=sys.stderr)
        return 2

    try:
        if args.commit_msg_file is not None:
            violations = check_commit_msg(args.commit_msg_file, args.repo)
        else:
            violations = check_range(
                args.range_spec, args.repo, respect_introduction=args.respect_introduction
            )
    except subprocess.CalledProcessError as exc:
        print(f"check_baseline_ceremony: git failed: {exc.stderr.strip()}", file=sys.stderr)
        return 2
    except (OSError, ValueError) as exc:
        print(f"check_baseline_ceremony: {exc}", file=sys.stderr)
        return 2

    if violations:
        for violation in violations:
            print(f"CEREMONY VIOLATION: {violation}", file=sys.stderr)
        print(_DOCTRINE, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
