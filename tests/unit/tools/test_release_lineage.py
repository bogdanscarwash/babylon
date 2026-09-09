"""Behavioral contracts for returning each protected main merge to dev."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

import pytest
from tools.release_lineage import (
    MANIFEST_PATH,
    ReleaseLineageError,
    build_lineage_payload,
    prepare_lineage,
    validate_stored_lineage,
    verify_lineage,
    write_lineage_payload,
)

MAIN_SHA = "a" * 40
ROOT = Path(__file__).resolve().parents[3]


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


def test_lineage_payload_records_the_exact_release_identity() -> None:
    payload = build_lineage_payload(
        tag="v1.2.3",
        main_sha=MAIN_SHA,
        release_pr=767,
        project_version="1.2.3",
    )

    assert payload == {
        "schema_version": 1,
        "latest_main_release": {
            "tag": "v1.2.3",
            "main_sha": MAIN_SHA,
            "release_pr": 767,
        },
    }
    assert validate_stored_lineage(payload, expected_tag="v1.2.3", expected_sha=MAIN_SHA) == 767


def test_checked_in_lineage_manifest_bootstraps_the_versioned_schema() -> None:
    payload = json.loads((ROOT / MANIFEST_PATH).read_text(encoding="utf-8"))

    assert payload == {"schema_version": 1, "latest_main_release": None}


def test_lineage_manifest_write_failure_is_a_bounded_refusal(tmp_path: Path) -> None:
    with pytest.raises(ReleaseLineageError, match="cannot write release-lineage manifest"):
        write_lineage_payload(tmp_path, {"schema_version": 1})


def test_prepare_and_verify_lineage_through_real_git_refs(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _git(tmp_path, "init", "--initial-branch=main")
    _git(tmp_path, "config", "user.name", "Lineage Contract")
    _git(tmp_path, "config", "user.email", "lineage-contract@example.invalid")
    (tmp_path / "pyproject.toml").write_text(
        '[project]\nname = "lineage-contract"\nversion = "1.2.3"\n',
        encoding="utf-8",
    )
    (tmp_path / MANIFEST_PATH).parent.mkdir(parents=True)
    (tmp_path / MANIFEST_PATH).write_text(
        '{"schema_version": 1, "latest_main_release": null}\n',
        encoding="utf-8",
    )
    _git(tmp_path, "add", ".")
    _git(tmp_path, "commit", "-m", "test: lineage contract")
    main_sha = _git(tmp_path, "rev-parse", "HEAD")
    _git(tmp_path, "update-ref", "refs/remotes/origin/main", main_sha)
    _git(tmp_path, "switch", "-c", "codex/PER-261-lineage-contract")
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr("tools.release_lineage.ROOT", tmp_path)

    assert prepare_lineage(tag="v1.2.3", release_pr=767) == main_sha
    _git(tmp_path, "add", MANIFEST_PATH.as_posix())
    _git(tmp_path, "commit", "-m", "ci: record release lineage")
    dev_sha = _git(tmp_path, "rev-parse", "HEAD")
    _git(tmp_path, "update-ref", "refs/remotes/origin/dev", dev_sha)

    assert (
        verify_lineage(
            ref="refs/remotes/origin/dev",
            tag="v1.2.3",
            main_sha=main_sha,
        )
        == 767
    )


@pytest.mark.parametrize(
    ("tag", "main_sha", "release_pr", "project_version"),
    [
        ("release-1.2.3", MAIN_SHA, 767, "1.2.3"),
        ("v1.2.3", "short", 767, "1.2.3"),
        ("v1.2.3", MAIN_SHA, 0, "1.2.3"),
        ("v1.2.3", MAIN_SHA, 767, "1.2.4"),
    ],
)
def test_lineage_payload_rejects_an_unbound_release(
    tag: str,
    main_sha: str,
    release_pr: int,
    project_version: str,
) -> None:
    with pytest.raises(ReleaseLineageError):
        build_lineage_payload(
            tag=tag,
            main_sha=main_sha,
            release_pr=release_pr,
            project_version=project_version,
        )


@pytest.mark.parametrize(
    "payload",
    [
        {},
        {"schema_version": 2, "latest_main_release": {}},
        {"schema_version": 1, "latest_main_release": None},
        {
            "schema_version": 1,
            "latest_main_release": {
                "tag": "v1.2.4",
                "main_sha": MAIN_SHA,
                "release_pr": 767,
            },
        },
        {
            "schema_version": 1,
            "latest_main_release": {
                "tag": "v1.2.3",
                "main_sha": "b" * 40,
                "release_pr": 767,
            },
        },
    ],
)
def test_stored_lineage_rejects_stale_or_malformed_evidence(payload: dict[str, Any]) -> None:
    with pytest.raises(ReleaseLineageError):
        validate_stored_lineage(payload, expected_tag="v1.2.3", expected_sha=MAIN_SHA)


def test_stored_lineage_rejects_matching_but_noncanonical_identity() -> None:
    payload: dict[str, Any] = {
        "schema_version": 1,
        "latest_main_release": {
            "tag": "release-1.2.3",
            "main_sha": "short",
            "release_pr": 767,
        },
    }

    with pytest.raises(ReleaseLineageError):
        validate_stored_lineage(
            payload,
            expected_tag="release-1.2.3",
            expected_sha="short",
        )


def test_release_lineage_cli_runs_by_repository_path() -> None:
    result = subprocess.run(
        ("python3", "tools/release_lineage.py", "--help"),
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
        timeout=10,
    )

    assert result.returncode == 0, result.stderr
    assert "prepare" in result.stdout
    assert "verify" in result.stdout


def test_release_lineage_cli_reports_git_failure_without_a_traceback(tmp_path: Path) -> None:
    result = subprocess.run(
        (
            "python3",
            str(ROOT / "tools" / "release_lineage.py"),
            "verify",
            "--ref",
            "refs/remotes/origin/dev",
            "--tag",
            "v1.2.3",
            "--main-sha",
            MAIN_SHA,
        ),
        cwd=tmp_path,
        check=False,
        capture_output=True,
        text=True,
        timeout=10,
    )

    assert result.returncode == 1
    assert "release-lineage: REFUSED" in result.stderr
    assert "Traceback" not in result.stderr
