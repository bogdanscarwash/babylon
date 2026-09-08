"""Contracts for publishing only a qualified main-reachable release tag."""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tomllib
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


@pytest.mark.parametrize("branch", ["feature/release", "dev", "main"])
def test_release_bump_updates_only_version_files_on_an_ordinary_lane(
    tmp_path: Path, branch: str
) -> None:
    """Exercise real Commitizen and uv; a bump neither commits nor publishes."""
    uv = shutil.which("uv")
    assert uv is not None, "release tests require the project-pinned uv executable"
    repo = tmp_path / "release"
    repo.mkdir()
    (repo / "pyproject.toml").write_text(
        '[project]\nname = "release-fixture"\nversion = "0.3.0"\n'
        'requires-python = ">=3.12"\ndependencies = ["fixture-dependency==1.0.0"]\n'
        '[tool.uv.sources]\nfixture-dependency = { path = "dependency" }\n'
        '[tool.commitizen]\nname = "cz_conventional_commits"\nversion_provider = "uv"\n'
        'version_scheme = "semver"\nmajor_version_zero = true\ntag_format = "v$version"\n'
        "update_changelog_on_bump = true\n",
        encoding="utf-8",
    )
    (repo / "dependency").mkdir()
    (repo / "dependency/pyproject.toml").write_text(
        '[project]\nname = "fixture-dependency"\nversion = "1.0.0"\n', encoding="utf-8"
    )
    (repo / "CHANGELOG.md").write_text("## v0.3.0\n", encoding="utf-8")
    binaries = repo / "commands"
    binaries.mkdir()
    environment = {
        **os.environ,
        "UV_PYTHON": sys.executable,
        "UV_CACHE_DIR": str(tmp_path / "uv-cache"),
    }
    environment.pop("UV_FROZEN", None)
    subprocess.run(
        (uv, "lock", "--offline"),
        cwd=repo,
        env=environment,
        check=True,
        capture_output=True,
        text=True,
        timeout=20,
    )
    original_lock = tomllib.loads((repo / "uv.lock").read_text(encoding="utf-8"))

    def executable(path: Path, source: str) -> None:
        path.write_text(f"#!{sys.executable}\n{source}", encoding="utf-8")
        path.chmod(0o755)

    executable(
        binaries / "uv",
        "import os, sys\n"
        "if sys.argv[1:4] == ['run', '--frozen', 'cz']:\n"
        "    os.execv(sys.executable, [sys.executable, '-m', 'commitizen', *sys.argv[4:]])\n"
        "else:\n"
        f"    os.execv({uv!r}, [{uv!r}, *sys.argv[1:]])\n",
    )
    (repo / "tools").mkdir()
    executable(repo / "tools/check_release_pins.sh", "pass\n")
    shutil.copyfile(ROOT / "tools/release_version.py", repo / "tools/release_version.py")
    _git(repo, "init", f"--initial-branch={branch}")
    _git(repo, "config", "user.name", "Release Contract")
    _git(repo, "config", "user.email", "release-contract@example.invalid")
    _git(repo, "add", ".")
    _git(repo, "commit", "-m", "test: release bump fixture")
    _git(repo, "tag", "v0.3.0")
    _git(repo, "commit", "--allow-empty", "-m", "feat: next release")
    original_head = _git(repo, "rev-parse", "HEAD")
    environment["PATH"] = f"{binaries}{os.pathsep}{environment.get('PATH', '')}"
    environment["UV_FROZEN"] = "1"
    script = tomllib.loads(MISE_PATH.read_text(encoding="utf-8"))["tasks"]["release:bump"]["run"]
    result = subprocess.run(
        ("bash", "-s", "--", "--yes"),
        input=script,
        cwd=repo,
        env=environment,
        check=False,
        capture_output=True,
        text=True,
        timeout=30,
    )
    assert _git(repo, "rev-parse", "HEAD") == original_head
    assert _git(repo, "tag", "--list") == "v0.3.0"
    assert _git(repo, "diff", "--cached", "--name-only") == ""
    if branch in {"dev", "main"}:
        assert result.returncode != 0
        assert "ordinary lane" in result.stderr
        assert _git(repo, "status", "--porcelain") == ""
        return
    assert result.returncode == 0, result.stdout + result.stderr
    changed_lock = tomllib.loads((repo / "uv.lock").read_text(encoding="utf-8"))
    assert tomllib.loads((repo / "pyproject.toml").read_text())["project"]["version"] == "0.4.0"
    assert (
        next(p["version"] for p in changed_lock["package"] if p["name"] == "release-fixture")
        == "0.4.0"
    )
    assert [p for p in changed_lock["package"] if p["name"] != "release-fixture"] == [
        p for p in original_lock["package"] if p["name"] != "release-fixture"
    ]
    assert (repo / "CHANGELOG.md").read_text().startswith("## v0.4.0")


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
    assert 'TAG="$INPUT_TAG"' in text
    assert 'TAG="${{ inputs.tag }}"' not in text
    assert '[[ "$TAG" =~ ^v' in text
    assert "RELEASE_TAG: ${{ steps.tag.outputs.tag }}" in text
    assert '--tag "$RELEASE_TAG"' in text


@pytest.mark.parametrize("path", RELEASE_WORKFLOWS)
@pytest.mark.parametrize(
    ("tag", "accepted"),
    [
        ("v1.2.3", True),
        ("v0.4.0", True),
        ("v1.2.3-rc.1", False),
        ("v1.2.3+build.7", False),
        ("v1.2.3-rc.1+build.7", False),
        ("v00.4.0", False),
        ("v0.4.0\nextra", False),
        ("release-1.2.3", False),
    ],
)
def test_publisher_shell_regex_accepts_canonical_semver_tags(
    path: Path, tag: str, accepted: bool, tmp_path: Path
) -> None:
    step = next(
        step
        for step in _workflow(path)["jobs"]["identity"]["steps"]
        if step.get("name") == "Resolve tag"
    )
    output_path = tmp_path / "output"
    result = subprocess.run(
        ("bash", "-e", "-s"),
        input=step["run"],
        env={
            **os.environ,
            "INPUT_TAG": tag,
            "GITHUB_EVENT_NAME": "workflow_dispatch",
            "GITHUB_OUTPUT": str(output_path),
        },
        check=False,
        capture_output=True,
        text=True,
        timeout=10,
    )

    assert (result.returncode == 0) is accepted
    if accepted:
        assert output_path.read_text() == f"tag={tag}\n"
    else:
        assert not output_path.exists()


def test_release_ceremony_tags_only_after_the_director_main_merge() -> None:
    text = VERSIONING_PATH.read_text(encoding="utf-8")

    assert "mise run release:tag -- --yes" in text
    assert text.index("mise run pr:merge -- N --director-main") < text.index(
        "mise run release:tag -- --yes"
    )
    assert text.index("mise run release:prepare-dev-sync -- vX.Y.Z N") < text.index(
        "mise run release:tag -- --yes"
    )
