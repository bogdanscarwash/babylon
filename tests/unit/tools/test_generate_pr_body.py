"""Behavioral contract for the PR-body generator (git doctrine §5.9 item 1).

Covers both halves of the deliverable:

a. **Trailer grammar** (:mod:`tools.trailer_schema`) — the §5.3 coordination-
   database schema (``Task``, ``Train``, ``Lane``, ``Safety``, ``Pinned``,
   ``Baselines``, ``Session``, ``Co-Authored-By``), parsed via git's own
   trailer machinery rather than a hand-rolled heuristic.
b. **PR-body generation** (:mod:`tools.generate_pr_body`) — fed a synthetic
   git history (a real ``git init`` under ``tmp_path``, mirroring
   ``tests/unit/tools/test_baseline_ceremony.py``'s fixture idiom), asserting
   the generated body's structure: grouped by type/scope, trailers surfaced,
   footer present.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

from generate_pr_body import (  # type: ignore[import-not-found]  # noqa: E402
    STANDARD_FOOTER,
    collect_commits,
    main,
    parse_conventional_subject,
    render_pr_body,
)
from trailer_schema import (  # type: ignore[import-not-found]  # noqa: E402
    TRAILER_SCHEMA,
    git_log_trailers,
    parse_trailer_block,
    validate_trailer_value,
    validate_trailers,
)

pytestmark = pytest.mark.unit


def _git(repo: Path, *args: str) -> str:
    result = subprocess.run(["git", *args], cwd=repo, capture_output=True, text=True, check=True)
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


def _commit(repo: Path, rel_path: str, content: str, message: str) -> str:
    target = repo / rel_path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content)
    _git(repo, "add", rel_path)
    _git(repo, "commit", "--no-verify", "-m", message)
    return _git(repo, "rev-parse", "HEAD")


TASK_TRAILER = "Task: 2026-07-18-track1-organizers-map#10"
TRAIN_TRAILER = "Train: train/archive-keel"
PINNED_TRAILER = "Pinned: dev@72def41c"
BASELINES_UNTOUCHED = "Baselines: untouched"


def _full_trailers(extra: str = "") -> str:
    body = "\n".join(
        [
            TASK_TRAILER,
            TRAIN_TRAILER,
            "Lane: pages",
            "Safety: P",
            PINNED_TRAILER,
            BASELINES_UNTOUCHED,
        ]
    )
    if extra:
        body += f"\n{extra}"
    return body


# ---------------------------------------------------------------------------
# Trailer grammar
# ---------------------------------------------------------------------------


class TestTrailerGrammar:
    def test_valid_task_accepted(self) -> None:
        assert validate_trailer_value("Task", "2026-07-18-track1-organizers-map#10")

    def test_task_without_hash_number_rejected(self) -> None:
        assert not validate_trailer_value("Task", "2026-07-18-track1-organizers-map")

    def test_valid_train_accepted(self) -> None:
        assert validate_trailer_value("Train", "train/archive-keel")

    def test_train_missing_prefix_rejected(self) -> None:
        assert not validate_trailer_value("Train", "archive-keel")

    def test_valid_safety_values(self) -> None:
        assert validate_trailer_value("Safety", "P")
        assert validate_trailer_value("Safety", "S")

    def test_invalid_safety_value_rejected(self) -> None:
        assert not validate_trailer_value("Safety", "parallel")

    def test_valid_pinned_accepted(self) -> None:
        assert validate_trailer_value("Pinned", "dev@72def41c")

    def test_pinned_without_sha_rejected(self) -> None:
        assert not validate_trailer_value("Pinned", "dev")

    def test_baselines_untouched_accepted(self) -> None:
        assert validate_trailer_value("Baselines", "untouched")

    def test_baselines_blessed_accepted(self) -> None:
        assert validate_trailer_value("Baselines", "blessed(unit-proof-ceremony)")

    def test_baselines_uppercase_slug_rejected(self) -> None:
        assert not validate_trailer_value("Baselines", "blessed(LOUD)")

    def test_unknown_key_always_valid(self) -> None:
        assert validate_trailer_value("Signed-off-by", "whoever <x@example.com>")

    def test_valid_co_authored_by_accepted(self) -> None:
        assert validate_trailer_value("Co-Authored-By", "Claude Fable 5 <noreply@anthropic.com>")

    def test_malformed_co_authored_by_rejected(self) -> None:
        assert not validate_trailer_value("Co-Authored-By", "no angle brackets here")

    def test_every_schema_example_validates_against_its_own_pattern(self) -> None:
        # Each TrailerSpec's own worked example must satisfy its own pattern —
        # a schema entry whose example fails its own grammar is a bug in the
        # doc, not in a caller.
        for key, spec in TRAILER_SCHEMA.items():
            assert validate_trailer_value(key, spec.example), key


class TestValidateTrailers:
    def test_fully_conformant_set_has_no_violations(self) -> None:
        trailers = {
            "Task": "2026-07-18-track1-organizers-map#10",
            "Train": "train/archive-keel",
            "Lane": "pages",
            "Safety": "P",
            "Pinned": "dev@72def41c",
            "Baselines": "untouched",
            "Session": "abc-123",
        }
        assert validate_trailers(trailers) == []

    def test_missing_required_trailer_reported(self) -> None:
        violations = validate_trailers({"Task": "2026-07-18-track1-organizers-map#10"})
        assert any("Train" in v for v in violations)

    def test_malformed_value_reported(self) -> None:
        violations = validate_trailers(
            {
                "Task": "2026-07-18-track1-organizers-map#10",
                "Train": "train/archive-keel",
                "Lane": "pages",
                "Safety": "sideways",
                "Pinned": "dev@72def41c",
                "Baselines": "untouched",
                "Session": "abc-123",
            }
        )
        assert any("Safety" in v for v in violations)


class TestParseTrailerBlock:
    def test_parses_multiple_trailers(self) -> None:
        raw = "Task: 2026-07-18-track1-organizers-map#10\x02Train: train/archive-keel"
        parsed = parse_trailer_block(raw)
        assert parsed == {
            "Task": "2026-07-18-track1-organizers-map#10",
            "Train": "train/archive-keel",
        }

    def test_empty_block_parses_empty(self) -> None:
        assert parse_trailer_block("") == {}


class TestGitLogTrailers:
    def test_reads_trailers_from_real_commit(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        message = f"feat(projection): add county view\n\n{_full_trailers()}"
        sha = _commit(repo, "src/module.py", "x = 1\n", message)
        entries = git_log_trailers(f"{base}..HEAD", repo)
        assert len(entries) == 1
        got_sha, subject, trailers = entries[0]
        assert got_sha == sha
        assert subject == "feat(projection): add county view"
        assert trailers["Task"] == "2026-07-18-track1-organizers-map#10"
        assert trailers["Baselines"] == "untouched"

    def test_merge_commits_excluded(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        _git(repo, "switch", "-c", "side")
        _commit(repo, "a.py", "a = 1\n", "feat(a): add a")
        _git(repo, "switch", "main")
        _commit(repo, "b.py", "b = 1\n", "feat(b): add b")
        _git(repo, "merge", "--no-ff", "-m", "merge side into main", "side")
        entries = git_log_trailers(f"{base}..HEAD", repo)
        subjects = [subject for _, subject, _ in entries]
        assert "feat(a): add a" in subjects
        assert "feat(b): add b" in subjects
        assert not any("merge" in subject.lower() for subject in subjects)


# ---------------------------------------------------------------------------
# Conventional-subject parsing
# ---------------------------------------------------------------------------


class TestParseConventionalSubject:
    def test_type_scope_description(self) -> None:
        assert parse_conventional_subject("feat(projection): add county view") == (
            "feat",
            "projection",
            "add county view",
        )

    def test_type_only(self) -> None:
        assert parse_conventional_subject("chore: bump lockfile") == (
            "chore",
            None,
            "bump lockfile",
        )

    def test_breaking_change_marker_ignored_in_split(self) -> None:
        commit_type, scope, description = parse_conventional_subject("feat(api)!: remove v1")
        assert commit_type == "feat"
        assert scope == "api"
        assert description == "remove v1"

    def test_non_conventional_falls_back_to_other(self) -> None:
        assert parse_conventional_subject("WIP quick fix") == ("other", None, "WIP quick fix")

    def test_unknown_type_falls_back_to_other(self) -> None:
        assert parse_conventional_subject("bogus: not a real type") == (
            "other",
            None,
            "bogus: not a real type",
        )


# ---------------------------------------------------------------------------
# Full PR-body generation against a synthetic repo
# ---------------------------------------------------------------------------


class TestCollectAndRenderPrBody:
    def test_grouped_by_type_and_scope(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        _commit(
            repo, "src/a.py", "a = 1\n", f"feat(projection): add county view\n\n{_full_trailers()}"
        )
        _commit(
            repo, "src/b.py", "b = 1\n", f"fix(tui): correct wikilink render\n\n{_full_trailers()}"
        )
        _commit(repo, "docs/c.rst", "c\n", f"docs(reference): trailer schema\n\n{_full_trailers()}")
        commits = collect_commits(base, "HEAD", repo)
        body = render_pr_body(base, "HEAD", commits)

        assert "## Commits" in body
        assert "### feat" in body
        assert "### fix" in body
        assert "### docs" in body
        assert body.index("### feat") < body.index("### fix") < body.index("### docs")
        assert "**(projection)** add county view" in body
        assert "**(tui)** correct wikilink render" in body

    def test_oldest_first_within_a_type(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        _commit(repo, "src/a.py", "a = 1\n", f"feat(x): first\n\n{_full_trailers()}")
        _commit(repo, "src/b.py", "b = 1\n", f"feat(x): second\n\n{_full_trailers()}")
        commits = collect_commits(base, "HEAD", repo)
        body = render_pr_body(base, "HEAD", commits)
        assert body.index("first") < body.index("second")

    def test_provenance_section_surfaces_trailers(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        _commit(repo, "src/a.py", "a = 1\n", f"feat(x): a change\n\n{_full_trailers()}")
        commits = collect_commits(base, "HEAD", repo)
        body = render_pr_body(base, "HEAD", commits)
        assert "## Provenance" in body
        assert "Task" in body
        assert "2026-07-18-track1-organizers-map#10" in body
        assert "train/archive-keel" in body

    def test_ends_with_standard_footer(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        _commit(repo, "src/a.py", "a = 1\n", f"feat(x): a change\n\n{_full_trailers()}")
        commits = collect_commits(base, "HEAD", repo)
        body = render_pr_body(base, "HEAD", commits)
        assert body.rstrip("\n").endswith(STANDARD_FOOTER.rstrip("\n"))

    def test_empty_range_still_renders_footer(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        commits = collect_commits(base, "HEAD", repo)
        assert commits == []
        body = render_pr_body(base, "HEAD", commits)
        assert "_No commits in range._" in body
        assert body.rstrip("\n").endswith(STANDARD_FOOTER.rstrip("\n"))

    def test_malformed_trailer_surfaced_as_warning(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        bad_trailers = _full_trailers().replace("Safety: P", "Safety: sideways")
        _commit(repo, "src/a.py", "a = 1\n", f"feat(x): a change\n\n{bad_trailers}")
        commits = collect_commits(base, "HEAD", repo)
        body = render_pr_body(base, "HEAD", commits)
        assert "## Trailer warnings" in body
        assert "Safety" in body

    def test_deterministic_across_two_runs(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        _commit(repo, "src/a.py", "a = 1\n", f"feat(x): a change\n\n{_full_trailers()}")
        commits_1 = collect_commits(base, "HEAD", repo)
        commits_2 = collect_commits(base, "HEAD", repo)
        assert render_pr_body(base, "HEAD", commits_1) == render_pr_body(base, "HEAD", commits_2)


class TestMainCli:
    def test_writes_to_stdout_by_default(
        self, tmp_path: Path, capsys: pytest.CaptureFixture[str]
    ) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        _commit(repo, "src/a.py", "a = 1\n", f"feat(x): a change\n\n{_full_trailers()}")
        assert main(["--base", base, "--head", "HEAD", "--repo", str(repo)]) == 0
        captured = capsys.readouterr()
        assert "## Commits" in captured.out
        assert STANDARD_FOOTER.splitlines()[0] in captured.out

    def test_writes_to_output_file(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        base = _git(repo, "rev-parse", "HEAD")
        _commit(repo, "src/a.py", "a = 1\n", f"feat(x): a change\n\n{_full_trailers()}")
        out = tmp_path / "body.md"
        assert (
            main(["--base", base, "--head", "HEAD", "--repo", str(repo), "--output", str(out)]) == 0
        )
        assert "## Commits" in out.read_text()

    def test_bad_range_is_git_error(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        assert main(["--base", "nonexistent", "--head", "HEAD", "--repo", str(repo)]) == 2
