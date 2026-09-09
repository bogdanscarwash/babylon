#!/usr/bin/env python3
"""Reject release tags that do not identify qualified main history."""

from __future__ import annotations

import argparse
import re
import subprocess
from typing import Final

TAG_PATTERN: Final[re.Pattern[str]] = re.compile(
    r"v(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)"
    r"(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?"
)
SHA_PATTERN: Final[re.Pattern[str]] = re.compile(r"[0-9a-f]{40}")
MAIN_REF: Final[str] = "refs/remotes/origin/main"
GIT_TIMEOUT_SECONDS: Final[int] = 30


class ReleaseTagError(RuntimeError):
    """The release tag is not safe to publish."""


def validate_release_identity(
    *,
    tag: str,
    head_sha: str,
    tag_commit_sha: str,
    is_main_ancestor: bool,
) -> None:
    """Validate the language-neutral release identity facts."""
    if TAG_PATTERN.fullmatch(tag) is None:
        raise ReleaseTagError(f"release tag must be canonical semver, found {tag!r}")
    if SHA_PATTERN.fullmatch(head_sha) is None:
        raise ReleaseTagError("HEAD is not a full lowercase Git SHA")
    if SHA_PATTERN.fullmatch(tag_commit_sha) is None:
        raise ReleaseTagError("tag target is not a full lowercase Git SHA")
    if head_sha != tag_commit_sha:
        raise ReleaseTagError("checked-out HEAD does not equal the release tag target")
    if not is_main_ancestor:
        raise ReleaseTagError("release tag target is not reachable from origin/main")


def _run_git(*args: str) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(
            ("git", *args),
            check=False,
            capture_output=True,
            text=True,
            timeout=GIT_TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired as error:
        raise ReleaseTagError(
            f"git {' '.join(args)} timed out after {GIT_TIMEOUT_SECONDS}s"
        ) from error
    except OSError as error:
        raise ReleaseTagError(f"cannot run git {' '.join(args)}: {error}") from error


def git_output(*args: str) -> str:
    result = _run_git(*args)
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "no diagnostic"
        raise ReleaseTagError(f"git {' '.join(args)} failed: {detail}")
    return result.stdout.strip()


def _head_is_on_main(head_sha: str) -> bool:
    result = _run_git("merge-base", "--is-ancestor", head_sha, MAIN_REF)
    if result.returncode == 0:
        return True
    if result.returncode == 1:
        return False
    detail = result.stderr.strip() or result.stdout.strip() or "no diagnostic"
    raise ReleaseTagError(f"git merge-base failed: {detail}")


def verify_release_tag(tag: str) -> str:
    """Verify one checked-out tag and return its commit SHA."""
    head_sha = git_output("rev-parse", "HEAD")
    tag_commit_sha = git_output("rev-parse", "--verify", f"refs/tags/{tag}^{{commit}}")
    validate_release_identity(
        tag=tag,
        head_sha=head_sha,
        tag_commit_sha=tag_commit_sha,
        is_main_ancestor=_head_is_on_main(head_sha),
    )
    return tag_commit_sha


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True, help="Canonical vX.Y.Z release tag")
    args = parser.parse_args()
    try:
        sha = verify_release_tag(args.tag)
    except ReleaseTagError as error:
        parser.error(str(error))
    print(f"release tag contract: {args.tag} -> {sha} is reachable from origin/main")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
