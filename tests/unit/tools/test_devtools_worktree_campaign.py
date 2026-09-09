"""Contracts for worktree-local Rust campaign identities."""

from __future__ import annotations

import io
import uuid
from pathlib import Path

import pytest
from tools.devtools import worktree_campaign

EXPLICIT_CAMPAIGN_ID = "6c402ec5-5749-4f9b-9664-42f9710adcdc"


def _select(root: Path, purpose: str, *, fresh: bool = False) -> str:
    return worktree_campaign.select_campaign_id(
        worktree_root=root,
        purpose=purpose,
        fresh=fresh,
        environment={},
    )


def test_stable_identity_depends_on_resolved_worktree_and_purpose(tmp_path: Path) -> None:
    first_root = (tmp_path / "first").resolve()
    second_root = (tmp_path / "second").resolve()

    first = _select(first_root, "michigan-e2e")

    assert uuid.UUID(first).version == 5
    assert _select(first_root, "michigan-e2e") == first
    assert _select(second_root, "michigan-e2e") != first
    assert _select(first_root, "another-purpose") != first


def test_fresh_identity_is_uuid4_and_changes_per_selection(tmp_path: Path) -> None:
    root = tmp_path.resolve()
    first = _select(root, "one-shot", fresh=True)
    second = _select(root, "one-shot", fresh=True)

    assert uuid.UUID(first).version == 4
    assert uuid.UUID(second).version == 4
    assert second != first


@pytest.mark.parametrize("fresh", [False, True])
def test_explicit_canonical_environment_identity_wins(tmp_path: Path, fresh: bool) -> None:
    selected = worktree_campaign.select_campaign_id(
        worktree_root=tmp_path.resolve(),
        purpose="michigan-e2e",
        fresh=fresh,
        environment={worktree_campaign.CAMPAIGN_ENV: EXPLICIT_CAMPAIGN_ID},
    )

    assert selected == EXPLICIT_CAMPAIGN_ID


@pytest.mark.parametrize(
    "configured",
    [
        "",
        "not-a-uuid",
        "6C402EC5-5749-4F9B-9664-42F9710ADCDC",
        "{6c402ec5-5749-4f9b-9664-42f9710adcdc}",
        "6c402ec557494f9b966442f9710adcdc",
    ],
)
def test_explicit_environment_identity_must_be_canonical(tmp_path: Path, configured: str) -> None:
    with pytest.raises(
        worktree_campaign.CampaignIdentityError,
        match="BABYLON_CAMPAIGN_ID must be a canonical UUID",
    ):
        worktree_campaign.select_campaign_id(
            worktree_root=tmp_path.resolve(),
            purpose="michigan-e2e",
            fresh=False,
            environment={worktree_campaign.CAMPAIGN_ENV: configured},
        )


@pytest.mark.parametrize("purpose", ["", "UPPERCASE", "has space", "two\nlines"])
def test_purpose_is_bounded_and_shell_safe(tmp_path: Path, purpose: str) -> None:
    with pytest.raises(worktree_campaign.CampaignIdentityError, match="campaign purpose"):
        _select(tmp_path.resolve(), purpose)


def test_cli_resolves_the_git_worktree_root_and_prints_only_the_uuid(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    resolved = tmp_path.resolve()
    monkeypatch.setattr(worktree_campaign, "_resolved_worktree_root", lambda _path: resolved)
    stdout = io.StringIO()
    stderr = io.StringIO()

    status = worktree_campaign.main(
        ["--purpose", "michigan-e2e", "--repository", "nested"],
        environment={},
        stdout=stdout,
        stderr=stderr,
    )

    assert status == 0
    assert stdout.getvalue() == f"{_select(resolved, 'michigan-e2e')}\n"
    assert stderr.getvalue() == ""


def test_cli_reports_a_specific_invalid_environment_error(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    monkeypatch.setattr(
        worktree_campaign, "_resolved_worktree_root", lambda _path: tmp_path.resolve()
    )
    stdout = io.StringIO()
    stderr = io.StringIO()

    status = worktree_campaign.main(
        ["--purpose", "michigan-e2e"],
        environment={worktree_campaign.CAMPAIGN_ENV: "invalid"},
        stdout=stdout,
        stderr=stderr,
    )

    assert status == 2
    assert stdout.getvalue() == ""
    assert stderr.getvalue() == (
        "worktree-campaign: BABYLON_CAMPAIGN_ID must be a canonical UUID\n"
    )


def test_cli_explicit_identity_does_not_probe_git(monkeypatch: pytest.MonkeyPatch) -> None:
    def unexpected_git_probe(_path: Path) -> Path:
        raise AssertionError("explicit campaign identity must bypass Git discovery")

    monkeypatch.setattr(worktree_campaign, "_resolved_worktree_root", unexpected_git_probe)
    stdout = io.StringIO()

    status = worktree_campaign.main(
        ["--purpose", "michigan-e2e"],
        environment={worktree_campaign.CAMPAIGN_ENV: EXPLICIT_CAMPAIGN_ID},
        stdout=stdout,
    )

    assert status == 0
    assert stdout.getvalue() == f"{EXPLICIT_CAMPAIGN_ID}\n"
