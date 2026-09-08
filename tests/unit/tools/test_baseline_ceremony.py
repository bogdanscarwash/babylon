"""Behavioral contract for the baseline-ceremony gate (§6.5 provenance & ceremony).

Owner ruling 2026-07-20: the ceremony doctrine's home is CLAUDE.md + CI teeth,
not a constitutional amendment. The gate (``tools/check_baseline_ceremony.py``)
mechanizes Constitution III.7's "regenerate the baselines *and say so*": any
commit that touches ``tests/baselines/**`` must machine-checkably declare its
ceremony via a ``Baselines: blessed(<ceremony-slug>)`` trailer.

Two enforcement modes, both covered here against synthetic git repos:

a. ``--commit-msg-file`` — the local commit-msg hook (staged paths vs message);
b. ``--range A..B``      — the CI leg (every commit in a PR range; merges
   inspected for evil-merge content via ``diff-tree --cc``).

The synthetic-violation tests are the mutation proof that each mode detects
its violation class (Constitution III.11 — Loud Failure).
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

# Mirror the import path used by tools/*.py and its existing unit tests
# (see tests/unit/tools/test_repo_hygiene.py).
TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

from check_baseline_ceremony import (  # type: ignore[import-not-found]  # noqa: E402
    check_commit_msg,
    check_range,
    main,
    message_declares_ceremony,
    touches_baselines,
)

pytestmark = pytest.mark.unit


def _git(repo: Path, *args: str) -> str:
    """Run git in ``repo``, failing the test loudly on nonzero exit."""
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


def _commit_file(repo: Path, rel_path: str, content: str, message: str) -> str:
    target = repo / rel_path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content)
    _git(repo, "add", rel_path)
    _git(repo, "commit", "-m", message)
    return _git(repo, "rev-parse", "HEAD")


BLESSED = "test(baselines): ceremony\n\nBaselines: blessed(unit-proof-ceremony)"


class TestTrailerGrammar:
    """The declaration is a strict trailer line, not a vibe."""

    def test_valid_trailer_accepted(self) -> None:
        assert message_declares_ceremony(BLESSED)

    def test_missing_trailer_rejected(self) -> None:
        assert not message_declares_ceremony("test(baselines): regen\n\nno declaration")

    def test_empty_slug_rejected(self) -> None:
        assert not message_declares_ceremony("msg\n\nBaselines: blessed()")

    def test_uppercase_slug_rejected(self) -> None:
        assert not message_declares_ceremony("msg\n\nBaselines: blessed(LOUD)")

    def test_untouched_is_not_a_blessing(self) -> None:
        assert not message_declares_ceremony("msg\n\nBaselines: untouched")

    def test_mid_line_mention_rejected(self) -> None:
        assert not message_declares_ceremony("msg\n\nsee Baselines: blessed(x) above")

    def test_comment_line_rejected(self) -> None:
        # git strips '#' comment lines at cleanup; the gate must not accept a
        # declaration that only exists inside one.
        assert not message_declares_ceremony("msg\n\n# Baselines: blessed(sneaky)")


class TestPathScope:
    def test_baseline_path_in_scope(self) -> None:
        assert touches_baselines(["tests/baselines/glut.json"])

    def test_dense_subdir_in_scope(self) -> None:
        assert touches_baselines(["tests/baselines/dense/glut/C001_wealth.csv"])

    def test_non_baseline_paths_out_of_scope(self) -> None:
        assert not touches_baselines(["src/babylon/engine/simulation_engine.py"])

    def test_lookalike_prefix_out_of_scope(self) -> None:
        assert not touches_baselines(["tests/baselines_archive/old.json"])


class TestCommitMsgMode:
    """Local hook: staged baseline paths demand a declared message."""

    def test_staged_baseline_without_trailer_is_violation(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        (repo / "tests/baselines").mkdir(parents=True)
        (repo / "tests/baselines/glut.json").write_text("{}\n")
        _git(repo, "add", "tests/baselines/glut.json")
        msg = tmp_path / "msg.txt"
        msg.write_text("test(baselines): regen without declaration\n")
        violations = check_commit_msg(msg, repo)
        assert violations, "undeclared staged baseline change must be flagged"

    def test_staged_baseline_with_trailer_is_clean(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        (repo / "tests/baselines").mkdir(parents=True)
        (repo / "tests/baselines/glut.json").write_text("{}\n")
        _git(repo, "add", "tests/baselines/glut.json")
        msg = tmp_path / "msg.txt"
        msg.write_text(BLESSED + "\n")
        assert check_commit_msg(msg, repo) == []

    def test_non_baseline_staging_needs_no_trailer(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        (repo / "notes.md").write_text("plain\n")
        _git(repo, "add", "notes.md")
        msg = tmp_path / "msg.txt"
        msg.write_text("docs: plain change\n")
        assert check_commit_msg(msg, repo) == []


class TestRangeMode:
    """CI leg: every commit in the range is inspected (merges via --cc)."""

    def test_undeclared_commit_flagged_by_sha(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        sha = _commit_file(
            repo, "tests/baselines/glut.json", "{}\n", "test(baselines): silent drift"
        )
        violations = check_range(f"{base}..HEAD", repo)
        assert len(violations) == 1
        assert sha[:12] in violations[0] or sha in violations[0]

    def test_declared_commit_is_clean(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        _commit_file(repo, "tests/baselines/glut.json", "{}\n", BLESSED)
        assert check_range(f"{base}..HEAD", repo) == []

    def test_non_baseline_commits_ignored(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        _commit_file(repo, "src/module.py", "x = 1\n", "feat: unrelated")
        assert check_range(f"{base}..HEAD", repo) == []

    def test_clean_merge_commit_not_flagged(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        _git(repo, "switch", "-c", "side")
        _commit_file(repo, "tests/baselines/glut.json", "{}\n", BLESSED)
        _git(repo, "switch", "main")
        _commit_file(repo, "other.md", "y\n", "docs: mainline drift")
        _git(repo, "merge", "--no-ff", "-m", "merge side into main", "side")
        # A clean merge introduces no content beyond its parents: the merge
        # commit needs no trailer of its own (the side commit is blessed).
        assert check_range(f"{base}..HEAD", repo) == []

    @pytest.mark.parametrize("respect_introduction", [False, True])
    def test_evil_merge_baseline_change_flagged(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, respect_introduction: bool
    ) -> None:
        """A merge whose conflict resolution differs from BOTH parents is a
        ceremony in its own right — the Opus-review evil-merge hole."""
        repo = _init_repo(tmp_path)
        _commit_file(repo, "tests/baselines/glut.json", "{}\n", BLESSED)
        base = _git(repo, "rev-parse", "HEAD")
        monkeypatch.setattr("check_baseline_ceremony.CEREMONY_INTRODUCTION", base)
        _git(repo, "switch", "-c", "side")
        _commit_file(repo, "tests/baselines/glut.json", '{"v": 1}\n', BLESSED)
        _git(repo, "switch", "main")
        _commit_file(repo, "tests/baselines/glut.json", '{"v": 2}\n', BLESSED)
        # Conflicting merge; resolve to content present in NEITHER parent.
        merge = subprocess.run(["git", "merge", "side"], cwd=repo, capture_output=True, text=True)
        assert merge.returncode != 0, "merge must conflict for the evil resolution"
        (repo / "tests/baselines/glut.json").write_text('{"v": 999, "evil": true}\n')
        _git(repo, "add", "tests/baselines/glut.json")
        _git(repo, "commit", "--no-verify", "-m", "merge side into main")
        merge_sha = _git(repo, "rev-parse", "HEAD")
        violations = check_range(
            f"{base}..{_git(repo, 'rev-parse', 'HEAD')}",
            repo,
            respect_introduction=respect_introduction,
        )
        assert len(violations) == 1
        assert merge_sha[:12] in violations[0]

    @pytest.mark.parametrize("respect_introduction", [False, True])
    def test_evil_merge_with_trailer_is_clean(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, respect_introduction: bool
    ) -> None:
        repo = _init_repo(tmp_path)
        _commit_file(repo, "tests/baselines/glut.json", "{}\n", BLESSED)
        base = _git(repo, "rev-parse", "HEAD")
        monkeypatch.setattr("check_baseline_ceremony.CEREMONY_INTRODUCTION", base)
        _git(repo, "switch", "-c", "side")
        _commit_file(repo, "tests/baselines/glut.json", '{"v": 1}\n', BLESSED)
        _git(repo, "switch", "main")
        _commit_file(repo, "tests/baselines/glut.json", '{"v": 2}\n', BLESSED)
        merge = subprocess.run(["git", "merge", "side"], cwd=repo, capture_output=True, text=True)
        assert merge.returncode != 0
        (repo / "tests/baselines/glut.json").write_text('{"v": 999}\n')
        _git(repo, "add", "tests/baselines/glut.json")
        _git(
            repo,
            "commit",
            "--no-verify",
            "-m",
            "merge side into main\n\nBaselines: blessed(evil-merge-resolution)",
        )
        assert (
            check_range(
                f"{base}..{_git(repo, 'rev-parse', 'HEAD')}",
                repo,
                respect_introduction=respect_introduction,
            )
            == []
        )

    def test_multiple_undeclared_commits_all_reported(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        _commit_file(repo, "tests/baselines/a.json", "{}\n", "test(baselines): one")
        _commit_file(repo, "tests/baselines/b.json", "{}\n", "test(baselines): two")
        assert len(check_range(f"{base}..HEAD", repo)) == 2


class TestMainCli:
    def test_range_mode_exit_codes(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        _commit_file(repo, "tests/baselines/glut.json", "{}\n", "test(baselines): silent")
        assert main(["--range", f"{base}..HEAD", "--repo", str(repo)]) == 1
        _git(repo, "commit", "--amend", "-m", BLESSED)
        assert main(["--range", f"{base}..HEAD", "--repo", str(repo)]) == 0

    def test_commit_msg_mode_exit_codes(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        (repo / "tests/baselines").mkdir(parents=True)
        (repo / "tests/baselines/glut.json").write_text("{}\n")
        _git(repo, "add", "tests/baselines/glut.json")
        msg = tmp_path / "msg.txt"
        msg.write_text("test(baselines): silent\n")
        assert main(["--commit-msg-file", str(msg), "--repo", str(repo)]) == 1
        msg.write_text(BLESSED + "\n")
        assert main(["--commit-msg-file", str(msg), "--repo", str(repo)]) == 0

    def test_no_mode_is_usage_error(self) -> None:
        assert main([]) == 2

    def test_bad_range_is_git_error(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        assert main(["--range", "nonexistent..HEAD", "--repo", str(repo)]) == 2


class TestIntroductionBoundary:
    """Release ranges preserve old history without grandfathering new drift."""

    def test_old_history_excluded_but_introduction_and_later_drift_block(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        import check_baseline_ceremony as gate

        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        old = _commit_file(repo, "tests/baselines/old.json", "old\n", "historic ceremony")
        introduction = _commit_file(
            repo, "tests/baselines/start.json", "start\n", "introduce the gate with drift"
        )
        monkeypatch.setattr(gate, "CEREMONY_INTRODUCTION", introduction)
        later = _commit_file(repo, "tests/baselines/new.json", "new\n", "silent new drift")
        violations = gate.check_range(f"{base}..{later}", repo, respect_introduction=True)
        assert len(violations) == 2
        assert any(introduction[:12] in item for item in violations)
        assert any(later[:12] in item for item in violations)
        assert all(old[:12] not in item for item in violations)
        assert len(gate.check_range(f"{base}..{later}", repo)) == 3

    def test_late_side_branch_from_old_history_is_still_checked(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        import check_baseline_ceremony as gate

        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        old = _commit_file(repo, "tests/baselines/old.json", "old\n", "historic ceremony")
        introduction = _commit_file(repo, "gate.txt", "enforced\n", "introduce gate")
        monkeypatch.setattr(gate, "CEREMONY_INTRODUCTION", introduction)
        _git(repo, "switch", "-c", "late-side", old)
        late = _commit_file(repo, "tests/baselines/late.json", "late\n", "undeclared side drift")
        _git(repo, "switch", "main")
        _git(repo, "merge", "--no-ff", "-m", "integrate side", "late-side")
        head = _git(repo, "rev-parse", "HEAD")
        violations = gate.check_range(f"{base}..{head}", repo, respect_introduction=True)
        assert len(violations) == 1 and late[:12] in violations[0]

    def test_missing_or_unreachable_introduction_refuses(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        import check_baseline_ceremony as gate

        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        introduction = _commit_file(repo, "gate.txt", "enforced\n", "introduce gate")
        monkeypatch.setattr(gate, "CEREMONY_INTRODUCTION", introduction)
        _git(repo, "switch", "-c", "unrelated-side", base)
        head = _commit_file(repo, "notes.txt", "notes\n", "side commit")
        assert (
            gate.main(["--repo", str(repo), "--range", f"{base}..{head}", "--respect-introduction"])
            == 2
        )
        monkeypatch.setattr(gate, "CEREMONY_INTRODUCTION", "0" * 40)
        assert (
            gate.main(["--repo", str(repo), "--range", f"{base}..{head}", "--respect-introduction"])
            == 2
        )
