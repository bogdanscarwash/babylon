#!/usr/bin/env python3
"""Derive development versions from Git; validate deliberate release identities."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import tomllib
from dataclasses import asdict, dataclass
from pathlib import Path

SEMVER = re.compile(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)")
SHA = re.compile(r"[0-9a-f]{40}")


class VersionError(ValueError):
    """The checkout cannot provide a trustworthy version identity."""


@dataclass(frozen=True)
class VersionIdentity:
    version: str
    release_version: str
    revision: str
    dirty: bool
    anchor: str
    distance: int


def git(*args: str, allowed: tuple[int, ...] = (0,)) -> subprocess.CompletedProcess[bytes]:
    try:
        result = subprocess.run(("git", *args), capture_output=True, check=False, timeout=30)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise VersionError(f"cannot run git {args[0]}: {error}") from error
    if result.returncode not in allowed:
        detail = result.stderr.decode(errors="replace").strip()
        raise VersionError(f"git {' '.join(args)} failed: {detail}")
    return result


def output(*args: str) -> str:
    return git(*args).stdout.decode().strip()


def numeric_version(value: object) -> tuple[int, int, int]:
    if not isinstance(value, str) or (match := SEMVER.fullmatch(value)) is None:
        raise VersionError(f"expected canonical X.Y.Z version, found {value!r}")
    return int(match[1]), int(match[2]), int(match[3])


def read_config(root: Path) -> tuple[str, str | None, str | None]:
    try:
        config = tomllib.loads((root / "pyproject.toml").read_text(encoding="utf-8"))
        version = config["project"]["version"]
        major, _, patch = numeric_version(version)
        if major == 0 and patch != 0:
            raise VersionError("planned 0.x release version must reset patch to zero")
        bootstrap = config.get("tool", {}).get("babylon", {}).get("versioning", {})
        anchor, baseline = bootstrap.get("bootstrap_commit"), bootstrap.get("bootstrap_version")
        if anchor is not None or baseline is not None:
            if not isinstance(anchor, str) or SHA.fullmatch(anchor) is None:
                raise VersionError("bootstrap_commit must be a full lowercase Git SHA")
            numeric_version(baseline)
        return str(version), anchor, baseline
    except (OSError, KeyError, TypeError, tomllib.TOMLDecodeError) as error:
        raise VersionError(f"cannot read version configuration: {error}") from error


def nearest_release(bootstrap: str | None, baseline: str | None) -> tuple[str, str, int]:
    # Count all newly reachable commits: an integration merge includes the branch's
    # commits and the merge itself. Ties use tag names, never iteration order.
    candidates: list[tuple[int, str, str]] = []
    for tag in output("tag", "--merged", "HEAD", "--list", "v*").splitlines():
        if SEMVER.fullmatch(tag.removeprefix("v")):
            # Returning old main history must not revive a pre-native release.
            if (
                bootstrap is not None
                and git(
                    "merge-base", "--is-ancestor", bootstrap, f"refs/tags/{tag}", allowed=(0, 1)
                ).returncode
            ):
                continue
            distance = int(output("rev-list", "--count", f"{tag}..HEAD"))
            candidates.append((distance, tag, tag[1:]))
    if candidates:
        distance, anchor, version = min(candidates)
        return anchor, version, distance
    if bootstrap is not None and baseline is not None:
        if git("merge-base", "--is-ancestor", bootstrap, "HEAD", allowed=(0, 1)).returncode:
            raise VersionError("configured bootstrap commit is not reachable from HEAD")
        return bootstrap, baseline, int(output("rev-list", "--count", f"{bootstrap}..HEAD"))
    raise VersionError("no reachable canonical release tag or explicit native bootstrap anchor")


def dirty_digest(root: Path) -> str | None:
    status = git("status", "--porcelain=v1", "-z", "--untracked-files=all").stdout
    if not status:
        return None
    digest = hashlib.sha256(status)
    digest.update(git("diff", "--binary", "HEAD", "--").stdout)
    for name in git("ls-files", "--others", "--exclude-standard", "-z").stdout.split(b"\0"):
        if not name:
            continue
        path = root / name.decode(errors="surrogateescape")
        digest.update(name + b"\0")
        try:
            contents = str(path.readlink()).encode() if path.is_symlink() else path.read_bytes()
        except OSError as error:
            raise VersionError(f"cannot fingerprint untracked file {path}: {error}") from error
        digest.update(hashlib.sha256(contents).digest())
    return digest.hexdigest()[:12]


def identity(root: Path, release_tag: str | None = None) -> VersionIdentity:
    if output("rev-parse", "--is-shallow-repository") != "false":
        raise VersionError("shallow history cannot provide a version; fetch full history and tags")
    release, bootstrap, baseline = read_config(root)
    revision = output("rev-parse", "HEAD")
    dirty = dirty_digest(root)
    if release_tag is not None:
        if release_tag != f"v{release}":
            raise VersionError(f"release tag must equal configured version v{release}")
        if output("rev-parse", "--verify", f"refs/tags/{release_tag}^{{commit}}") != revision:
            raise VersionError("release tag does not identify checked-out HEAD")
        if dirty is not None:
            raise VersionError("release checkout is dirty")
        return VersionIdentity(release, release, revision, False, release_tag, 0)
    anchor, version, distance = nearest_release(bootstrap, baseline)
    major, minor, patch = numeric_version(version)
    suffix = f"g{revision[:12]}" + (f".dirty.{dirty}" if dirty else "")
    development = f"{major}.{minor}.{patch + distance}+{suffix}"
    return VersionIdentity(development, release, revision, dirty is not None, anchor, distance)


def check_commits(root: Path, base: str, head: str) -> None:
    """Validate new policy-era commit messages without rewriting old history."""
    if output("rev-parse", "--is-shallow-repository") != "false":
        raise VersionError("shallow history cannot provide a commit validation range")
    for revision in (base, head):
        if SHA.fullmatch(revision) is None:
            raise VersionError("commit range endpoints must be full lowercase Git SHAs")
        if output("rev-parse", "--verify", f"{revision}^{{commit}}") != revision:
            raise VersionError("commit range endpoint does not resolve to its exact commit")
    if git("merge-base", "--is-ancestor", base, head, allowed=(0, 1)).returncode:
        raise VersionError("commit range base is not an ancestor of head")
    _, bootstrap, _ = read_config(root)
    if bootstrap is not None:
        if git("merge-base", "--is-ancestor", bootstrap, head, allowed=(0, 1)).returncode:
            raise VersionError("commit range head predates the native commit policy")
        if not git("merge-base", "--is-ancestor", base, bootstrap, allowed=(0, 1)).returncode:
            base = bootstrap
    revision_range = f"{base}..{head}"
    try:
        result = subprocess.run(
            ("cz", "check", "--rev-range", revision_range), check=False, timeout=60
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise VersionError(f"cannot run pinned Commitizen: {error}") from error
    if result.returncode:
        raise VersionError(f"Commitizen refused commit range {revision_range}")
    print(f"commit message contract: {revision_range}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument(
        "--check-commits",
        nargs=2,
        metavar=("BASE", "HEAD"),
        help="Run pinned Commitizen on the exact new commit range",
    )
    mode.add_argument(
        "--release-tag", help="Validate a tagged release against the canonical version"
    )
    mode.add_argument(
        "--release-version", action="store_true", help="Print the planned release version"
    )
    parser.add_argument("--json", action="store_true", help="Emit version metadata as JSON")
    args = parser.parse_args()
    try:
        root = Path(output("rev-parse", "--show-toplevel"))
        os.chdir(root)
        if args.check_commits:
            check_commits(root, *args.check_commits)
        elif args.release_version:
            release, _, _ = read_config(root)
            print(json.dumps({"release_version": release}) if args.json else release)
        else:
            result = identity(root, args.release_tag)
            print(json.dumps(asdict(result), sort_keys=True) if args.json else result.version)
    except VersionError as error:
        parser.error(str(error))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
