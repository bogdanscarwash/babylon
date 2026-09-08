#!/usr/bin/env python3
"""Promote the immutable main-qualified native download without rebuilding it."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools.check_release_tag import (  # noqa: E402
    ReleaseTagError,
    git_output,
    verify_release_tag,
)
from tools.release_lineage import ReleaseLineageError, verify_lineage  # noqa: E402
from tools.release_version import VersionError, identity  # noqa: E402

MAX_ARCHIVE_BYTES = 1024**3
ARTIFACT_NAME = "native-linux-x86_64"
WORKFLOW_PATH = ".github/workflows/main.yml"


class ArtifactError(ValueError):
    """No exact, intact native qualification artifact can be published."""


@dataclass(frozen=True)
class Selection:
    repository: str
    tag: str
    version: str
    release_sha: str
    source_sha: str
    source_tree: str
    release_pr: int
    run_id: int
    run_attempt: int
    artifact_id: int
    artifact_digest: str


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ArtifactError(message)


def positive(value: object, label: str) -> int:
    if type(value) is not int or value <= 0:
        raise ArtifactError(f"{label} must be a positive integer")
    return value


def obj(value: object, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ArtifactError(f"{label} must be an object")
    return value


def complete_page(payload: dict[str, Any], key: str) -> list[dict[str, Any]]:
    rows = payload.get(key)
    if not isinstance(rows, list):
        raise ArtifactError(f"missing {key} list")
    require(
        payload.get("total_count") == len(rows),
        f"incomplete {key} result; refuse ambiguous selection",
    )
    return [obj(row, key) for row in rows]


def api(endpoint: str) -> dict[str, Any]:
    try:
        result = subprocess.run(
            ("gh", "api", endpoint), check=False, capture_output=True, text=True, timeout=60
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise ArtifactError(f"GitHub lookup failed: {error}") from error
    require(result.returncode == 0, f"GitHub lookup failed: {result.stderr.strip()}")
    try:
        return obj(json.loads(result.stdout), "GitHub response")
    except json.JSONDecodeError as error:
        raise ArtifactError(f"GitHub returned invalid JSON: {error}") from error


def validate_pr(
    pr: dict[str, Any], *, repository: str, release_pr: int, release_sha: str
) -> tuple[str, str]:
    require(
        pr.get("number") == release_pr
        and pr.get("merged") is True
        and pr.get("state") == "closed"
        and pr.get("merge_commit_sha") == release_sha,
        "lineage PR is not merged to the exact release commit",
    )
    head, base = obj(pr.get("head"), "PR head"), obj(pr.get("base"), "PR base")
    require(base.get("ref") == "main", "release PR must target main")
    for side in (head, base):
        require(
            obj(side.get("repo"), "PR repository").get("full_name") == repository,
            "release PR repository does not match the publisher",
        )
    branch = head.get("ref")
    require(
        isinstance(branch, str) and (branch == "dev" or branch.startswith("fix/")),
        "release PR source must be dev or a sanctioned fix branch",
    )
    source = head.get("sha")
    require(
        isinstance(source, str) and re.fullmatch(r"[0-9a-f]{40}", source) is not None,
        "release PR source must be an exact commit SHA",
    )
    return str(source), str(branch)


def verify_source_tree(source_sha: str, release_sha: str) -> str:
    parents = git_output("rev-list", "--parents", "-n", "1", release_sha).split()
    require(
        len(parents) == 3 and parents[2] == source_sha,
        "qualified commit is not the exact main merge source parent",
    )
    git_output("merge-base", "--is-ancestor", source_sha, release_sha)
    source_tree = git_output("rev-parse", f"{source_sha}^{{tree}}")
    release_tree = git_output("rev-parse", f"{release_sha}^{{tree}}")
    require(source_tree == release_tree, "main release tree differs from the qualified source tree")
    return source_tree


def select_run(
    payload: dict[str, Any], *, repository: str, source_sha: str, workflow_id: int, branch: str
) -> tuple[int, int]:
    rows = complete_page(payload, "workflow_runs")
    require(bool(rows), "no main qualification run exists for the exact source commit")
    latest = max(rows, key=lambda row: positive(row.get("id"), "workflow run ID"))
    require(
        latest.get("status") == "completed" and latest.get("conclusion") == "success",
        "latest qualification run did not complete successfully; no fallback to older evidence",
    )
    require(
        latest.get("workflow_id") == workflow_id
        and latest.get("path") == WORKFLOW_PATH
        and latest.get("event") == "pull_request"
        and latest.get("head_sha") == source_sha
        and latest.get("head_branch") == branch,
        "qualification run identity does not match release source",
    )
    for key in ("repository", "head_repository"):
        require(
            obj(latest.get(key), key).get("full_name") == repository,
            "qualification run repository mismatch",
        )
    return positive(latest.get("id"), "workflow run ID"), positive(
        latest.get("run_attempt"), "run attempt"
    )


def select_artifact(payload: dict[str, Any], *, run_id: int, source_sha: str) -> tuple[int, str]:
    matches = [
        row for row in complete_page(payload, "artifacts") if row.get("name") == ARTIFACT_NAME
    ]
    require(len(matches) == 1, "qualification must contain exactly one native artifact")
    artifact = matches[0]
    require(artifact.get("expired") is False, "qualified native artifact has expired")
    size = positive(artifact.get("size_in_bytes"), "artifact size")
    require(size <= MAX_ARCHIVE_BYTES, "qualified artifact exceeds the one-GiB download bound")
    digest = artifact.get("digest")
    require(
        isinstance(digest, str) and re.fullmatch(r"sha256:[0-9a-f]{64}", digest) is not None,
        "qualified artifact has no exact GitHub SHA-256 digest",
    )
    run = obj(artifact.get("workflow_run"), "artifact workflow run")
    require(
        run.get("id") == run_id and run.get("head_sha") == source_sha,
        "artifact does not belong to the exact qualification run and source",
    )
    return positive(artifact.get("id"), "artifact ID"), str(digest)


def resolve(repository: str, tag: str) -> Selection:
    require(
        re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository) is not None,
        "repository must be owner/name",
    )
    root = Path(git_output("rev-parse", "--show-toplevel"))
    release_sha = verify_release_tag(tag)
    version = identity(root, tag).version
    release_pr = verify_lineage(ref="refs/remotes/origin/dev", tag=tag, main_sha=release_sha)
    prefix = f"repos/{repository}"
    source_sha, branch = validate_pr(
        api(f"{prefix}/pulls/{release_pr}"),
        repository=repository,
        release_pr=release_pr,
        release_sha=release_sha,
    )
    tree = verify_source_tree(source_sha, release_sha)
    workflow = api(f"{prefix}/actions/workflows/main.yml")
    require(
        workflow.get("path") == WORKFLOW_PATH and workflow.get("state") == "active",
        "canonical main qualification workflow is not active",
    )
    workflow_id = positive(workflow.get("id"), "workflow ID")
    runs = api(
        f"{prefix}/actions/workflows/{workflow_id}/runs?event=pull_request&head_sha={source_sha}&per_page=100"
    )
    run_id, attempt = select_run(
        runs, repository=repository, source_sha=source_sha, workflow_id=workflow_id, branch=branch
    )
    artifact_id, digest = select_artifact(
        api(f"{prefix}/actions/runs/{run_id}/artifacts?per_page=100"),
        run_id=run_id,
        source_sha=source_sha,
    )
    return Selection(
        repository,
        tag,
        version,
        release_sha,
        source_sha,
        tree,
        release_pr,
        run_id,
        attempt,
        artifact_id,
        digest,
    )


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def verify_download(download: Path, selected: Selection, staging: Path) -> None:
    require(download.stat().st_size <= MAX_ARCHIVE_BYTES, "download exceeds the one-GiB bound")
    require(
        "sha256:" + sha256(download) == selected.artifact_digest,
        "download does not match the GitHub artifact digest",
    )
    archive_name = f"babylon-{selected.version}-linux-x86_64.tar.gz"
    expected = {archive_name, archive_name + ".sha256"}
    with zipfile.ZipFile(download) as archive:
        entries = archive.infolist()
        require(
            len(entries) == 2 and {entry.filename for entry in entries} == expected,
            "qualified ZIP must contain only the expected native archive and checksum",
        )
        for entry in entries:
            require(
                not entry.is_dir() and entry.file_size <= MAX_ARCHIVE_BYTES,
                "artifact entry is not a bounded file",
            )
            with archive.open(entry) as source, (staging / entry.filename).open("xb") as target:
                shutil.copyfileobj(source, target)
    checksum_path = staging / (archive_name + ".sha256")
    require(checksum_path.stat().st_size <= 1024, "package checksum file is oversized")
    checksum = checksum_path.read_text(encoding="ascii").split()
    require(
        checksum == [sha256(staging / archive_name), archive_name],
        "native package checksum mismatch",
    )
    with tarfile.open(staging / archive_name, "r:gz") as package:
        name = f"babylon-{selected.version}-linux-x86_64/release.json"
        manifests = [item for item in package.getmembers() if item.name == name]
        require(
            len(manifests) == 1 and manifests[0].isfile() and manifests[0].size <= 65536,
            "package must contain one bounded release manifest",
        )
        stream = package.extractfile(manifests[0])
        if stream is None:
            raise ArtifactError("cannot read native release manifest")
        with stream:
            manifest = obj(json.load(stream), "native release manifest")
    require(
        type(manifest.get("schema")) is int
        and manifest.get("schema") == 1
        and manifest.get("platform") == "linux-x86_64"
        and manifest.get("version") == selected.version
        and manifest.get("source_sha") == selected.source_sha,
        "package manifest does not identify the exact qualified source and release version",
    )


def promote(selected: Selection, destination: Path) -> None:
    require(not destination.exists(), "release download destination already exists")
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="native-release-", dir=destination.parent) as folder:
        download = Path(folder) / "artifact.zip"
        with download.open("xb") as target:
            try:
                result = subprocess.run(
                    (
                        "gh",
                        "api",
                        f"repos/{selected.repository}/actions/artifacts/{selected.artifact_id}/zip",
                    ),
                    stdout=target,
                    stderr=subprocess.PIPE,
                    check=False,
                    timeout=180,
                )
            except (OSError, subprocess.TimeoutExpired) as error:
                raise ArtifactError(f"artifact download failed: {error}") from error
        require(
            result.returncode == 0,
            f"artifact download failed: {result.stderr.decode(errors='replace').strip()}",
        )
        staging = Path(folder) / "verified"
        staging.mkdir()
        verify_download(download, selected, staging)
        (staging / "release-provenance.json").write_text(
            json.dumps(asdict(selected), indent=2) + "\n"
        )
        staging.rename(destination)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--output-dir", required=True, type=Path)
    args = parser.parse_args()
    try:
        os.chdir(git_output("rev-parse", "--show-toplevel"))
        selected = resolve(args.repository, args.tag)
        promote(selected, args.output_dir.resolve())
    except (
        ArtifactError,
        ReleaseTagError,
        ReleaseLineageError,
        VersionError,
        OSError,
        zipfile.BadZipFile,
        tarfile.TarError,
        UnicodeError,
        json.JSONDecodeError,
    ) as error:
        parser.error(str(error))
    print(json.dumps(asdict(selected), sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
