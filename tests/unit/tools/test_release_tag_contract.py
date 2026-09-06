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


def test_bump_commits_matching_project_and_lock_versions_without_upgrading_dependencies(
    tmp_path: Path,
) -> None:
    """Run the real bump shell and uv resolver, substituting only owner commands."""
    uv = shutil.which("uv")
    assert uv is not None, "release tests require the project-pinned uv executable"
    repo = tmp_path / "release"
    repo.mkdir()
    (repo / "pyproject.toml").write_text(
        '[project]\nname = "release-fixture"\nversion = "0.3.0"\n'
        'requires-python = ">=3.12"\ndependencies = ["fixture-dependency==1.0.0"]\n'
        '[tool.uv.sources]\nfixture-dependency = { path = "dependency" }\n',
        encoding="utf-8",
    )
    (repo / "dependency").mkdir()
    (repo / "dependency/pyproject.toml").write_text(
        '[project]\nname = "fixture-dependency"\nversion = "1.0.0"\n', encoding="utf-8"
    )
    (repo / "CHANGELOG.md").write_text("0.3.0\n", encoding="utf-8")
    binaries = repo / "commands"
    binaries.mkdir()
    environment = {
        **os.environ,
        "UV_PYTHON": sys.executable,
        "UV_CACHE_DIR": str(tmp_path / "uv-cache"),
        "RELEASE_COMMIT_MARKER": str(tmp_path / "committed"),
    }
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
        "import os, sys\nfrom pathlib import Path\n"
        "if sys.argv[1:] == ['run', 'cz', 'bump', '--dry-run']:\n"
        "    pass\n"
        "elif sys.argv[1:] == ['run', 'cz', 'bump', '--version-files-only', '--yes']:\n"
        "    p = Path('pyproject.toml')\n"
        "    p.write_text(p.read_text().replace('0.3.0', '0.4.0'))\n"
        "    Path('CHANGELOG.md').write_text('0.4.0\\n')\n"
        "else:\n"
        f"    os.execv({uv!r}, [{uv!r}, *sys.argv[1:]])\n",
    )
    executable(
        binaries / "mise",
        "import os, subprocess, sys, tomllib\nfrom pathlib import Path\n"
        "assert sys.argv[1:3] == ['run', 'commit']\n"
        "def staged(path):\n"
        "    return subprocess.check_output(['git', 'show', ':' + path], text=True)\n"
        "version = tomllib.loads(staged('pyproject.toml'))['project']['version']\n"
        "lock = tomllib.loads(staged('uv.lock'))\n"
        "locked = next(p['version'] for p in lock['package'] if p['name'] == 'release-fixture')\n"
        "assert version == locked, 'release commit would contain a stale lock'\n"
        "names = subprocess.check_output(['git', 'diff', '--cached', '--name-only'], text=True)\n"
        "assert set(names.splitlines()) == {'pyproject.toml', 'uv.lock', 'CHANGELOG.md'}\n"
        "Path(os.environ['RELEASE_COMMIT_MARKER']).write_text(version)\n",
    )
    (repo / "tools").mkdir()
    executable(repo / "tools/check_release_pins.sh", "pass\n")
    _git(repo, "init", "--initial-branch=dev")
    _git(repo, "config", "user.name", "Release Contract")
    _git(repo, "config", "user.email", "release-contract@example.invalid")
    _git(repo, "add", ".")
    _git(repo, "commit", "-m", "test: release bump fixture")
    environment["PATH"] = f"{binaries}{os.pathsep}{environment.get('PATH', '')}"
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

    assert result.returncode == 0, result.stdout + result.stderr
    assert (tmp_path / "committed").read_text(encoding="utf-8") == "0.4.0"
    changed_lock = tomllib.loads((repo / "uv.lock").read_text(encoding="utf-8"))
    assert [p for p in changed_lock["package"] if p["name"] != "release-fixture"] == [
        p for p in original_lock["package"] if p["name"] != "release-fixture"
    ]


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
