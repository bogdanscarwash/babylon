#!/usr/bin/env python3
"""Prepare and verify the PR-safe main-to-dev release-lineage record."""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
from pathlib import Path
from typing import Final

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools.check_release_tag import (  # noqa: E402
    SHA_PATTERN,
    TAG_PATTERN,
    ReleaseTagError,
    git_output,
)

ROOT: Final[Path] = Path(__file__).resolve().parents[1]
MANIFEST_PATH: Final[Path] = Path(".github/settings/release-lineage.json")
ALLOWED_BRANCH_PREFIXES: Final[tuple[str, ...]] = (
    "feature/",
    "fix/",
    "docs/",
    "refactor/",
    "test/",
    "codex/",
)


class ReleaseLineageError(RuntimeError):
    """The release ancestry record cannot prove a safe main-to-dev sync."""


def _lineage_git_output(*args: str) -> str:
    try:
        return git_output(*args)
    except ReleaseTagError as error:
        raise ReleaseLineageError(str(error)) from error


def build_lineage_payload(
    *, tag: str, main_sha: str, release_pr: int, project_version: str
) -> dict[str, object]:
    """Build one exact, language-neutral release-lineage record."""
    if TAG_PATTERN.fullmatch(tag) is None:
        raise ReleaseLineageError(f"release tag must be canonical semver, found {tag!r}")
    if SHA_PATTERN.fullmatch(main_sha) is None:
        raise ReleaseLineageError("main SHA must be a full lowercase Git SHA")
    if type(release_pr) is not int or release_pr <= 0:
        raise ReleaseLineageError("release PR must be a positive integer")
    if tag != f"v{project_version}":
        raise ReleaseLineageError(
            f"release tag {tag!r} does not match project version {project_version!r}"
        )
    return {
        "schema_version": 1,
        "latest_main_release": {
            "tag": tag,
            "main_sha": main_sha,
            "release_pr": release_pr,
        },
    }


def validate_stored_lineage(
    payload: dict[str, object], *, expected_tag: str, expected_sha: str
) -> int:
    """Validate an exact stored record and return its release PR number."""
    if TAG_PATTERN.fullmatch(expected_tag) is None:
        raise ReleaseLineageError("expected release tag must be canonical semver")
    if SHA_PATTERN.fullmatch(expected_sha) is None:
        raise ReleaseLineageError("expected main SHA must be a full lowercase Git SHA")
    if set(payload) != {"schema_version", "latest_main_release"}:
        raise ReleaseLineageError("release-lineage manifest has unexpected top-level fields")
    if payload.get("schema_version") != 1:
        raise ReleaseLineageError("release-lineage manifest schema must be 1")
    latest = payload.get("latest_main_release")
    if not isinstance(latest, dict):
        raise ReleaseLineageError("release-lineage manifest has no completed main release")
    if set(latest) != {"tag", "main_sha", "release_pr"}:
        raise ReleaseLineageError("latest main release has unexpected fields")
    if latest.get("tag") != expected_tag or latest.get("main_sha") != expected_sha:
        raise ReleaseLineageError("dev does not record the exact pending main release")
    release_pr = latest.get("release_pr")
    if type(release_pr) is not int or release_pr <= 0:
        raise ReleaseLineageError("stored release PR must be a positive integer")
    return release_pr


def _project_version() -> str:
    try:
        payload = tomllib.loads((ROOT / "pyproject.toml").read_text(encoding="utf-8"))
        version = payload["project"]["version"]
    except (OSError, tomllib.TOMLDecodeError, KeyError, TypeError) as error:
        raise ReleaseLineageError(f"cannot read project version: {error}") from error
    if not isinstance(version, str) or not version:
        raise ReleaseLineageError("project version must be a non-empty string")
    return version


def write_lineage_payload(destination: Path, payload: dict[str, object]) -> None:
    """Write one lineage payload or report a bounded release refusal."""
    try:
        destination.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    except OSError as error:
        raise ReleaseLineageError(f"cannot write release-lineage manifest: {error}") from error


def prepare_lineage(*, tag: str, release_pr: int) -> str:
    """Write a real-diff record from a clean branch at exact origin/main."""
    if _lineage_git_output("status", "--porcelain"):
        raise ReleaseLineageError("lineage lane must be clean before writing")
    branch = _lineage_git_output("branch", "--show-current")
    if not branch.startswith(ALLOWED_BRANCH_PREFIXES):
        raise ReleaseLineageError("prepare on a sanctioned lane branched from origin/main")
    head_sha = _lineage_git_output("rev-parse", "HEAD")
    main_sha = _lineage_git_output("rev-parse", "refs/remotes/origin/main")
    if head_sha != main_sha:
        raise ReleaseLineageError("lineage lane HEAD must equal exact origin/main before writing")
    payload = build_lineage_payload(
        tag=tag,
        main_sha=main_sha,
        release_pr=release_pr,
        project_version=_project_version(),
    )
    destination = ROOT / MANIFEST_PATH
    write_lineage_payload(destination, payload)
    return main_sha


def verify_lineage(*, ref: str, tag: str, main_sha: str) -> int:
    """Verify that a Git ref contains the exact pending release record."""
    raw = _lineage_git_output("show", f"{ref}:{MANIFEST_PATH.as_posix()}")
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as error:
        raise ReleaseLineageError(f"stored release-lineage JSON is invalid: {error}") from error
    if not isinstance(payload, dict):
        raise ReleaseLineageError("stored release-lineage document must be an object")
    return validate_stored_lineage(payload, expected_tag=tag, expected_sha=main_sha)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    prepare = subparsers.add_parser("prepare")
    prepare.add_argument("--tag", required=True)
    prepare.add_argument("--release-pr", required=True, type=int)
    verify = subparsers.add_parser("verify")
    verify.add_argument("--ref", required=True)
    verify.add_argument("--tag", required=True)
    verify.add_argument("--main-sha", required=True)
    args = parser.parse_args()
    try:
        if args.command == "prepare":
            sha = prepare_lineage(tag=args.tag, release_pr=args.release_pr)
            print(f"release lineage prepared: {args.tag} -> {sha}")
        else:
            release_pr = verify_lineage(ref=args.ref, tag=args.tag, main_sha=args.main_sha)
            print(f"release lineage verified through PR #{release_pr}")
    except ReleaseLineageError as error:
        print(f"release-lineage: REFUSED — {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
