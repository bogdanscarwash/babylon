"""Closes the documented §6.5 local-gate gap (WO-10, git-doctrine item 3).

``tools/check_baseline_ceremony.py``'s module docstring says the
``--commit-msg-file`` leg is best-effort: an ``--amend`` that strips an
already-declared trailer, or a pathspec commit, can slip a baseline change
past the local commit-msg hook. The CI ``--range`` leg (PR base..head) is
the authoritative backstop.

This closes the gap *locally*: ``.pre-commit-config.yaml`` now runs the same
``--range`` checker at the ``pre-push`` stage against the merge-base with
the upstream ref, so a slipped amend is still caught before it leaves the
box — the exact scenario exercised below.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

from check_baseline_ceremony import check_range  # type: ignore[import-not-found]  # noqa: E402

pytestmark = pytest.mark.unit

BLESSED = "test(baselines): recalibrate\n\nBaselines: blessed(recalibration-2026-07-21)"


def _git(repo: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=repo,
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout.strip()


def _init_repo(tmp_path: Path) -> Path:
    repo = tmp_path / "repo"
    repo.mkdir()
    _git(repo, "init", "-b", "main")
    _git(repo, "config", "user.email", "gate-test@example.invalid")
    _git(repo, "config", "user.name", "Gate Test")
    (repo / "README.md").write_text("seed\n")
    _git(repo, "add", "README.md")
    _git(repo, "commit", "-m", "chore: seed")
    return repo


class TestAmendSlipCaughtByRange:
    """A commit that started declared, then had its blessing stripped by a
    ``--amend`` (bypassing/mistiming the commit-msg hook), is still flagged
    by the pre-push range leg — the mirror of CI's authoritative check."""

    def test_amend_that_strips_trailer_is_flagged(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")

        (repo / "tests/baselines").mkdir(parents=True)
        (repo / "tests/baselines/glut.json").write_text("{}\n")
        _git(repo, "add", "tests/baselines/glut.json")
        _git(repo, "commit", "-m", BLESSED)

        # Range leg sees the well-formed commit as clean, same as the
        # commit-msg hook did at commit time.
        assert check_range(f"{base}..HEAD", repo) == []

        # Simulate the documented slip: an --amend (e.g. a squash-fixup, or
        # a hook bypassed with --no-verify) drops the trailer without
        # re-running the commit-msg leg in the same way.
        _git(repo, "commit", "--amend", "--no-verify", "-m", "test(baselines): recalibrate (oops)")

        # The pre-push range leg — run against the same merge-base a real
        # pre-push hook would use — catches what the commit-msg leg missed.
        violations = check_range(f"{base}..HEAD", repo)
        assert len(violations) == 1
        assert "baseline change undeclared" in violations[0]

    def test_pathspec_commit_that_omits_trailer_is_flagged(self, tmp_path: Path) -> None:
        """A ``git commit <pathspec>`` commit (temporary index) that never
        carries a declaration is still caught by the range leg."""
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")

        (repo / "tests/baselines").mkdir(parents=True)
        (repo / "tests/baselines/glut.json").write_text("{}\n")
        (repo / "other.py").write_text("x = 1\n")
        _git(repo, "add", "tests/baselines/glut.json", "other.py")
        # Pathspec commit: only stages/commits other.py's change via the
        # temporary index, but the working tree already has the baseline
        # edit queued — a realistic slip shape.
        _git(repo, "commit", "other.py", "--no-verify", "-m", "chore: unrelated pathspec commit")
        _git(repo, "add", "tests/baselines/glut.json")
        _git(repo, "commit", "--no-verify", "-m", "test(baselines): forgot the trailer")

        violations = check_range(f"{base}..HEAD", repo)
        assert len(violations) == 1
