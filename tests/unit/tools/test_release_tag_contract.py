"""Contracts for publishing only a qualified main-reachable release tag."""

from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Any

import pytest
import yaml
from tools.check_release_tag import (
    ReleaseTagError,
    git_output,
    validate_release_identity,
    verify_release_tag,
)

ROOT = Path(__file__).resolve().parents[3]
MISE_PATH = ROOT / ".mise.toml"
VERSIONING_PATH = ROOT / "docs" / "versioning.md"
RELEASE_WORKFLOWS = (ROOT / ".github" / "workflows" / "release.yml",)


def _workflow(path: Path) -> dict[str, Any]:
    payload = yaml.safe_load(path.read_text(encoding="utf-8"))
    assert isinstance(payload, dict)
    return payload


def _git(repo: Path, *args: str) -> str:
    result = subprocess.run(
        ("git", *args),
        cwd=repo,
        check=True,
        capture_output=True,
        text=True,
        timeout=10,
    )
    return result.stdout.strip()


def test_release_identity_accepts_the_tagged_main_commit() -> None:
    sha = "a" * 40

    validate_release_identity(
        tag="v1.2.3",
        head_sha=sha,
        tag_commit_sha=sha,
        is_main_ancestor=True,
    )


def test_release_verifier_accepts_an_annotated_tag_on_remote_main(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _git(tmp_path, "init", "--initial-branch=main")
    _git(tmp_path, "config", "user.name", "Release Contract")
    _git(tmp_path, "config", "user.email", "release-contract@example.invalid")
    (tmp_path / "payload").write_text("release\n", encoding="utf-8")
    _git(tmp_path, "add", "payload")
    _git(tmp_path, "commit", "-m", "test: release contract")
    sha = _git(tmp_path, "rev-parse", "HEAD")
    _git(tmp_path, "update-ref", "refs/remotes/origin/main", sha)
    _git(tmp_path, "tag", "--annotate", "v1.2.3", "--message", "v1.2.3")
    monkeypatch.chdir(tmp_path)

    assert verify_release_tag("v1.2.3") == sha


def test_git_timeout_is_reported_as_a_release_refusal(monkeypatch: pytest.MonkeyPatch) -> None:
    def timeout(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        del args, kwargs
        raise subprocess.TimeoutExpired(cmd=("git", "rev-parse"), timeout=30)

    monkeypatch.setattr("tools.check_release_tag.subprocess.run", timeout)

    with pytest.raises(ReleaseTagError, match="timed out"):
        git_output("rev-parse", "HEAD")


@pytest.mark.parametrize(
    ("tag", "head_sha", "tag_commit_sha", "is_main_ancestor"),
    [
        ("release-1.2.3", "a" * 40, "a" * 40, True),
        ("v1.2.3", "a" * 40, "b" * 40, True),
        ("v1.2.3", "a" * 40, "a" * 40, False),
    ],
)
def test_release_identity_rejects_every_unqualified_shape(
    tag: str,
    head_sha: str,
    tag_commit_sha: str,
    is_main_ancestor: bool,
) -> None:
    with pytest.raises(ReleaseTagError):
        validate_release_identity(
            tag=tag,
            head_sha=head_sha,
            tag_commit_sha=tag_commit_sha,
            is_main_ancestor=is_main_ancestor,
        )


def test_bump_commit_and_main_tag_are_separate_owner_actions() -> None:
    text = MISE_PATH.read_text(encoding="utf-8")
    bump = text.split('[tasks."release:bump"]', maxsplit=1)[1].split("[tasks.", maxsplit=1)[0]
    tag = text.split('[tasks."release:tag"]', maxsplit=1)[1].split("[tasks.", maxsplit=1)[0]

    assert "cz bump --version-files-only" in bump
    assert "git tag" not in bump
    assert 'git branch --show-current)" = "main"' in tag
    assert "refs/remotes/origin/main" in tag
    assert "refs/remotes/origin/dev" in tag
    assert "git merge-base --is-ancestor" in tag
    assert "tools/release_lineage.py verify" in tag
    assert "git tag --annotate" in tag
    assert 'git push origin "refs/tags/$TAG"' in tag


@pytest.mark.parametrize("path", RELEASE_WORKFLOWS)
def test_every_publisher_requires_a_main_reachable_tag(path: Path) -> None:
    workflow = _workflow(path)
    text = path.read_text(encoding="utf-8")

    assert "tools/check_release_tag.py" in text
    assert "tools/release_lineage.py verify" in text
    assert "refs/remotes/origin/main" in text
    assert "refs/remotes/origin/dev" in text
    assert "git merge-base --is-ancestor" in text
    assert "fetch-depth: 0" in text
    assert workflow.get("on", workflow.get(True)) is not None


@pytest.mark.parametrize("path", RELEASE_WORKFLOWS)
def test_manual_tag_input_never_enters_shell_source(path: Path) -> None:
    text = path.read_text(encoding="utf-8")

    assert "INPUT_TAG: ${{ inputs.tag }}" in text
    assert 'TAG="${INPUT_TAG}"' in text
    assert 'TAG="${{ inputs.tag }}"' not in text
    assert 'if [[ ! "$TAG" =~ ^v' in text
    assert "RELEASE_TAG: ${{ steps.tag.outputs.tag }}" in text
    assert '--tag "$RELEASE_TAG"' in text


@pytest.mark.parametrize("path", RELEASE_WORKFLOWS)
@pytest.mark.parametrize(
    ("tag", "accepted"),
    [
        ("v1.2.3", True),
        ("v1.2.3-rc.1", True),
        ("v1.2.3+build.7", True),
        ("v1.2.3-rc.1+build.7", True),
        ("release-1.2.3", False),
    ],
)
def test_publisher_shell_regex_accepts_canonical_semver_tags(
    path: Path, tag: str, accepted: bool
) -> None:
    text = path.read_text(encoding="utf-8")
    condition = next(line.strip() for line in text.splitlines() if '"$TAG" =~' in line)
    pattern = condition.split("=~ ", maxsplit=1)[1].split(" ]];", maxsplit=1)[0]
    result = subprocess.run(
        ("bash", "-c", '[[ "$1" =~ $2 ]]', "bash", tag, pattern),
        check=False,
        capture_output=True,
        text=True,
        timeout=10,
    )

    assert (result.returncode == 0) is accepted


def test_release_ceremony_tags_only_after_the_director_main_merge() -> None:
    text = VERSIONING_PATH.read_text(encoding="utf-8")

    assert "mise run release:tag -- --yes" in text
    assert text.index("mise run pr:merge -- N --director-main") < text.index(
        "mise run release:tag -- --yes"
    )
    assert text.index("mise run release:prepare-dev-sync -- vX.Y.Z N") < text.index(
        "mise run release:tag -- --yes"
    )
