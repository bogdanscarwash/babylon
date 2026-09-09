"""Promotion of the exact main-qualified native download without rebuilding."""

from __future__ import annotations

import hashlib
import io
import json
import subprocess
import tarfile
import zipfile
from dataclasses import replace
from pathlib import Path

import pytest
from tools.release_artifact import (
    ArtifactError,
    Selection,
    select_artifact,
    select_run,
    validate_pr,
    verify_download,
    verify_source_tree,
)

SOURCE = "a" * 40
RELEASE = "b" * 40
REPO = "owner/babylon"


def run_row(run_id: int = 10, conclusion: str = "success") -> dict[str, object]:
    return {
        "id": run_id,
        "run_attempt": 1,
        "workflow_id": 3,
        "path": ".github/workflows/main.yml",
        "event": "pull_request",
        "head_sha": SOURCE,
        "head_branch": "dev",
        "status": "completed",
        "conclusion": conclusion,
        "repository": {"full_name": REPO},
        "head_repository": {"full_name": REPO},
    }


def artifact_row() -> dict[str, object]:
    return {
        "id": 15,
        "name": "native-linux-x86_64",
        "expired": False,
        "size_in_bytes": 300,
        "digest": "sha256:" + "c" * 64,
        "workflow_run": {"id": 10, "head_sha": SOURCE},
    }


def selection() -> Selection:
    return Selection(
        REPO, "v0.4.0", "0.4.0", RELEASE, SOURCE, "d" * 40, 2, 10, 1, 15, "sha256:" + "c" * 64
    )


def test_latest_run_failure_cannot_fall_back_to_previous_success() -> None:
    payload = {"total_count": 2, "workflow_runs": [run_row(), run_row(11, "failure")]}
    with pytest.raises(ArtifactError, match="latest qualification"):
        select_run(payload, repository=REPO, source_sha=SOURCE, workflow_id=3, branch="dev")


@pytest.mark.parametrize(
    ("field", "value"),
    [
        ("event", "workflow_dispatch"),
        ("head_sha", RELEASE),
        ("workflow_id", 999),
        ("path", ".github/workflows/fake.yml"),
        ("head_repository", {"full_name": "fork/babylon"}),
        ("head_branch", "other"),
    ],
)
def test_run_identity_refuses_unrelated_evidence(field: str, value: object) -> None:
    row = run_row()
    row[field] = value
    with pytest.raises(ArtifactError):
        select_run(
            {"total_count": 1, "workflow_runs": [row]},
            repository=REPO,
            source_sha=SOURCE,
            workflow_id=3,
            branch="dev",
        )


def test_latest_successful_canonical_run_is_selected() -> None:
    assert select_run(
        {"total_count": 2, "workflow_runs": [run_row(9), run_row()]},
        repository=REPO,
        source_sha=SOURCE,
        workflow_id=3,
        branch="dev",
    ) == (10, 1)


@pytest.mark.parametrize(
    ("field", "value"),
    [
        ("expired", True),
        ("digest", None),
        ("size_in_bytes", 0),
        ("workflow_run", {"id": 20, "head_sha": SOURCE}),
        ("workflow_run", {"id": 10, "head_sha": RELEASE}),
    ],
)
def test_artifact_refuses_stale_or_unbound_metadata(field: str, value: object) -> None:
    row = artifact_row()
    row[field] = value
    with pytest.raises(ArtifactError):
        select_artifact({"total_count": 1, "artifacts": [row]}, run_id=10, source_sha=SOURCE)


def test_artifact_name_must_have_one_exact_match() -> None:
    with pytest.raises(ArtifactError, match="exactly one"):
        select_artifact(
            {"total_count": 2, "artifacts": [artifact_row(), artifact_row()]},
            run_id=10,
            source_sha=SOURCE,
        )


def git(repo: Path, *args: str) -> str:
    return subprocess.check_output(("git", *args), cwd=repo, text=True).strip()


def test_real_merge_source_requires_identical_tree(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    git(tmp_path, "init", "-q", "-b", "main")
    git(tmp_path, "config", "user.name", "Artifact contract")
    git(tmp_path, "config", "user.email", "artifact@example.invalid")
    (tmp_path / "a").write_text("base")
    git(tmp_path, "add", ".")
    git(tmp_path, "commit", "-qm", "test: base")
    base = git(tmp_path, "rev-parse", "HEAD")
    git(tmp_path, "checkout", "-qb", "dev")
    (tmp_path / "a").write_text("native")
    git(tmp_path, "commit", "-qam", "feat: native")
    source = git(tmp_path, "rev-parse", "HEAD")
    git(tmp_path, "checkout", "-q", "main")
    git(tmp_path, "merge", "--no-ff", "-qm", "Merge release", "dev")
    release = git(tmp_path, "rev-parse", "HEAD")
    monkeypatch.chdir(tmp_path)
    assert verify_source_tree(source, release) == git(tmp_path, "rev-parse", f"{source}^{{tree}}")
    git(tmp_path, "checkout", "-qb", "divergent", base)
    (tmp_path / "b").write_text("unqualified extra content")
    git(tmp_path, "add", ".")
    git(tmp_path, "commit", "-qm", "feat: extra")
    git(tmp_path, "merge", "--no-ff", "-qm", "Merge changed release", "dev")
    with pytest.raises(ArtifactError, match="tree"):
        verify_source_tree(source, git(tmp_path, "rev-parse", "HEAD"))
    with pytest.raises(ArtifactError, match="source parent"):
        verify_source_tree(base, release)


def test_lineage_pr_must_be_merged_to_exact_main_commit() -> None:
    pr = {
        "number": 2,
        "merged": True,
        "state": "closed",
        "merge_commit_sha": RELEASE,
        "head": {"sha": SOURCE, "ref": "dev", "repo": {"full_name": REPO}},
        "base": {"ref": "main", "repo": {"full_name": REPO}},
    }
    assert validate_pr(pr, repository=REPO, release_pr=2, release_sha=RELEASE) == (SOURCE, "dev")
    pr["merge_commit_sha"] = SOURCE
    with pytest.raises(ArtifactError):
        validate_pr(pr, repository=REPO, release_pr=2, release_sha=RELEASE)


def make_download(
    tmp_path: Path,
    *,
    source: str = SOURCE,
    version: str = "0.4.0",
    bad_checksum: bool = False,
    extra: bool = False,
) -> tuple[Path, Selection]:
    archive_name = "babylon-0.4.0-linux-x86_64.tar.gz"
    archive = io.BytesIO()
    with tarfile.open(fileobj=archive, mode="w:gz") as tar:
        payload = json.dumps(
            {"schema": 1, "platform": "linux-x86_64", "version": version, "source_sha": source}
        ).encode()
        item = tarfile.TarInfo("babylon-0.4.0-linux-x86_64/release.json")
        item.size = len(payload)
        tar.addfile(item, io.BytesIO(payload))
    contents = archive.getvalue()
    digest = "0" * 64 if bad_checksum else hashlib.sha256(contents).hexdigest()
    path = tmp_path / "artifact.zip"
    with zipfile.ZipFile(path, "w") as zip_file:
        zip_file.writestr(archive_name, contents)
        zip_file.writestr(archive_name + ".sha256", f"{digest}  {archive_name}\n")
        if extra:
            zip_file.writestr("../outside", "refuse")
    return path, replace(
        selection(), artifact_digest="sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()
    )


def test_verified_download_preserves_qualified_archive_bytes(tmp_path: Path) -> None:
    path, selected = make_download(tmp_path)
    destination = tmp_path / "staging"
    destination.mkdir()
    verify_download(path, selected, destination)
    assert (destination / "babylon-0.4.0-linux-x86_64.tar.gz").is_file()


@pytest.mark.parametrize(
    "kwargs",
    [
        {"source": RELEASE},
        {"version": "0.5.0"},
        {"bad_checksum": True},
        {"extra": True},
    ],
)
def test_inner_package_must_match_exact_qualification(
    tmp_path: Path, kwargs: dict[str, object]
) -> None:
    path, selected = make_download(tmp_path, **kwargs)
    destination = tmp_path / "staging"
    destination.mkdir()
    with pytest.raises(ArtifactError):
        verify_download(path, selected, destination)


def test_download_sha_must_match_github_immutable_artifact_digest(tmp_path: Path) -> None:
    path, selected = make_download(tmp_path)
    destination = tmp_path / "staging"
    destination.mkdir()
    with pytest.raises(ArtifactError, match="GitHub artifact digest"):
        verify_download(path, replace(selected, artifact_digest="sha256:" + "0" * 64), destination)


def test_cli_promotes_download_through_real_git_lineage_and_mocked_github(tmp_path: Path) -> None:
    """Exercise the real CLI/ZIP promotion; only the remote API is substituted."""
    import os
    import sys

    repo = tmp_path / "repo"
    repo.mkdir()
    git(repo, "init", "-q", "-b", "main")
    git(repo, "config", "user.name", "Artifact contract")
    git(repo, "config", "user.email", "artifact@example.invalid")
    (repo / "pyproject.toml").write_text('[project]\nname="fixture"\nversion="0.4.0"\n')
    manifest = repo / ".github/settings/release-lineage.json"
    manifest.parent.mkdir(parents=True)
    manifest.write_text('{"schema_version":1,"latest_main_release":null}')
    git(repo, "add", ".")
    git(repo, "commit", "-qm", "test: initial")
    git(repo, "checkout", "-qb", "dev")
    (repo / "payload").write_text("qualified native code")
    git(repo, "add", ".")
    git(repo, "commit", "-qm", "feat: native")
    source = git(repo, "rev-parse", "HEAD")
    git(repo, "checkout", "-q", "main")
    git(repo, "merge", "--no-ff", "-qm", "Merge release", "dev")
    release = git(repo, "rev-parse", "HEAD")
    git(repo, "tag", "v0.4.0")
    git(repo, "update-ref", "refs/remotes/origin/main", release)
    git(repo, "checkout", "-qb", "dev-sync")
    manifest.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "latest_main_release": {
                    "tag": "v0.4.0",
                    "main_sha": release,
                    "release_pr": 2,
                },
            }
        )
    )
    git(repo, "commit", "-qam", "ci: record lineage")
    git(repo, "update-ref", "refs/remotes/origin/dev", "HEAD")
    git(repo, "checkout", "-q", "--detach", release)
    archive, selected = make_download(tmp_path, source=source)
    row = artifact_row()
    row.update(
        digest=selected.artifact_digest,
        size_in_bytes=archive.stat().st_size,
        workflow_run={"id": 10, "head_sha": source},
    )
    run = run_row()
    run["head_sha"] = source
    prefix = f"repos/{REPO}"
    responses = {
        f"{prefix}/pulls/2": {
            "number": 2,
            "merged": True,
            "state": "closed",
            "merge_commit_sha": release,
            "head": {"sha": source, "ref": "dev", "repo": {"full_name": REPO}},
            "base": {"ref": "main", "repo": {"full_name": REPO}},
        },
        f"{prefix}/actions/workflows/main.yml": {
            "id": 3,
            "state": "active",
            "path": ".github/workflows/main.yml",
        },
        f"{prefix}/actions/workflows/3/runs?event=pull_request&head_sha={source}&per_page=100": {
            "total_count": 1,
            "workflow_runs": [run],
        },
        f"{prefix}/actions/runs/10/artifacts?per_page=100": {"total_count": 1, "artifacts": [row]},
    }
    commands = tmp_path / "commands"
    commands.mkdir()
    gh = commands / "gh"
    gh.write_text(
        f"#!{sys.executable}\nimport json, sys\nfrom pathlib import Path\n"
        f"responses = {responses!r}\n"
        "assert sys.argv[1] == 'api'\n"
        f"if sys.argv[2] == '{prefix}/actions/artifacts/15/zip':\n"
        f"    sys.stdout.buffer.write(Path({str(archive)!r}).read_bytes())\n"
        "else:\n    print(json.dumps(responses[sys.argv[2]]))\n"
    )
    gh.chmod(0o755)
    destination = tmp_path / "published"
    script = Path(__file__).resolve().parents[3] / "tools/release_artifact.py"
    result = subprocess.run(
        (
            sys.executable,
            str(script),
            "--tag",
            "v0.4.0",
            "--repository",
            REPO,
            "--output-dir",
            str(destination),
        ),
        cwd=repo,
        env={**os.environ, "PATH": str(commands) + os.pathsep + os.environ["PATH"]},
        capture_output=True,
        text=True,
        check=False,
        timeout=10,
    )
    assert result.returncode == 0, result.stderr
    proof = json.loads((destination / "release-provenance.json").read_text())
    assert proof["source_sha"] == source
    assert proof["release_sha"] == release
    assert proof["artifact_digest"] == selected.artifact_digest
    assert proof["run_id"] == 10
    with zipfile.ZipFile(archive) as zipped:
        for name in zipped.namelist():
            assert (destination / name).read_bytes() == zipped.read(name)
