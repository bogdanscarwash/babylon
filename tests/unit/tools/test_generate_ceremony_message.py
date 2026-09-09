"""Behavioral contract for the ceremony-authoring helper (§6.5 mechanization,
git-doctrine adoption item 3 — WO-10).

``tools/generate_ceremony_message.py`` is the write-side sibling of
``tools/check_baseline_ceremony.py``: given staged ``tests/baselines/**``
drift, it computes a per-file drift table and emits the full commit-message
skeleton (subject/body/trailer). The load-bearing contract is that the
generated message always passes the *validator's own* trailer grammar
(``message_declares_ceremony``) — the two tools must never silently drift
apart on what counts as "well-formed".

Every test builds a synthetic git repo under ``tmp_path`` and never touches
the real ``tests/baselines/**`` estate.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[3]
TOOLS_DIR = REPO / "tools"
TOOL_SCRIPT = TOOLS_DIR / "generate_ceremony_message.py"
sys.path.insert(0, str(TOOLS_DIR))

from check_baseline_ceremony import (  # type: ignore[import-not-found]  # noqa: E402
    message_declares_ceremony,
)
from generate_ceremony_message import (  # type: ignore[import-not-found]  # noqa: E402
    build_ceremony_message,
    build_drift_row,
    main,
    staged_baseline_paths,
)

pytestmark = pytest.mark.unit


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


_CSV_HEADER = "tick,C001_wealth,C001_agitation\n"


def _seed_csv(repo: Path, rel_path: str, rows: list[str]) -> None:
    """Commit an initial CSV baseline at HEAD."""
    target = repo / rel_path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(_CSV_HEADER + "\n".join(rows) + "\n")
    _git(repo, "add", rel_path)
    _git(repo, "commit", "-m", "test(baselines): seed", "-m", "Baselines: blessed(seed-ceremony)")


class TestStagedBaselinePaths:
    def test_only_staged_baseline_paths_returned(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        (repo / "tests/baselines").mkdir(parents=True)
        (repo / "tests/baselines/glut.csv").write_text(_CSV_HEADER)
        (repo / "notes.md").write_text("x\n")
        _git(repo, "add", "tests/baselines/glut.csv", "notes.md")
        assert staged_baseline_paths(repo) == ["tests/baselines/glut.csv"]

    def test_no_staged_baselines_is_empty(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        (repo / "notes.md").write_text("x\n")
        _git(repo, "add", "notes.md")
        assert staged_baseline_paths(repo) == []


class TestDriftRowCsv:
    def test_added_file_reports_new_rows_no_delta(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        (repo / "tests/baselines").mkdir(parents=True)
        (repo / "tests/baselines/glut.csv").write_text(_CSV_HEADER + "0,1.0,0.0\n1,1.5,0.1\n")
        _git(repo, "add", "tests/baselines/glut.csv")
        row = build_drift_row(repo, "tests/baselines/glut.csv")
        assert row.status == "added"
        assert row.new_rows == 2
        assert row.max_abs_delta is None

    def test_modified_file_reports_changed_cells_and_max_delta(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        _seed_csv(repo, "tests/baselines/dense/glut.csv", ["0,1.0,0.0", "1,1.5,0.1"])
        target = repo / "tests/baselines/dense/glut.csv"
        target.write_text(_CSV_HEADER + "0,1.0,0.0\n1,2.5,0.1\n")
        _git(repo, "add", "tests/baselines/dense/glut.csv")
        row = build_drift_row(repo, "tests/baselines/dense/glut.csv")
        assert row.status == "modified"
        assert row.changed_cells == 1
        assert row.max_abs_delta == pytest.approx(1.0)
        assert row.old_rows == 2
        assert row.new_rows == 2

    def test_unchanged_staged_file_reports_zero_drift(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        _seed_csv(repo, "tests/baselines/dense/glut.csv", ["0,1.0,0.0"])
        target = repo / "tests/baselines/dense/glut.csv"
        # Re-stage identical content (e.g. a no-op regen run).
        _git(repo, "add", "tests/baselines/dense/glut.csv")
        row = build_drift_row(repo, "tests/baselines/dense/glut.csv")
        assert row.status == "unchanged"
        assert row.changed_cells == 0
        assert row.max_abs_delta is None
        assert target.exists()


class TestDriftRowNonCsv:
    def test_non_csv_file_reports_line_drift_not_fabricated_numeric_delta(
        self, tmp_path: Path
    ) -> None:
        repo = _init_repo(tmp_path)
        (repo / "tests/baselines").mkdir(parents=True)
        (repo / "tests/baselines/glut.json").write_text('{"a": 1}\n')
        _git(repo, "add", "tests/baselines/glut.json")
        _git(
            repo,
            "commit",
            "-m",
            "test(baselines): seed json",
            "-m",
            "Baselines: blessed(seed-ceremony)",
        )
        (repo / "tests/baselines/glut.json").write_text('{"a": 2}\n')
        _git(repo, "add", "tests/baselines/glut.json")
        row = build_drift_row(repo, "tests/baselines/glut.json")
        assert not row.parseable
        assert row.max_abs_delta is None
        assert row.changed_cells == 1


class TestGeneratedMessageValidatesClean:
    """The load-bearing contract: generated output always satisfies the
    validator's own trailer grammar."""

    def test_generated_message_passes_validator(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        _seed_csv(repo, "tests/baselines/dense/glut.csv", ["0,1.0,0.0"])
        (repo / "tests/baselines/dense/glut.csv").write_text(_CSV_HEADER + "0,1.0,0.0\n1,2.0,0.2\n")
        _git(repo, "add", "tests/baselines/dense/glut.csv")
        rows = [build_drift_row(repo, p) for p in staged_baseline_paths(repo)]
        message = build_ceremony_message("glut-recalibration-2026-07-21", "recalibrate glut", rows)
        assert message_declares_ceremony(message)
        assert message.startswith("test(baselines): recalibrate glut")
        assert "Baselines: blessed(glut-recalibration-2026-07-21)" in message

    def test_drift_table_lists_every_changed_file(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        (repo / "tests/baselines").mkdir(parents=True)
        (repo / "tests/baselines/a.csv").write_text(_CSV_HEADER)
        (repo / "tests/baselines/b.csv").write_text(_CSV_HEADER)
        _git(repo, "add", "tests/baselines/a.csv", "tests/baselines/b.csv")
        rows = [build_drift_row(repo, p) for p in staged_baseline_paths(repo)]
        message = build_ceremony_message("two-file-ceremony", "add two baselines", rows)
        assert "tests/baselines/a.csv" in message
        assert "tests/baselines/b.csv" in message


class TestMainCli:
    def test_no_staged_baselines_exits_1(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        assert main(["--slug", "x", "--summary", "y", "--repo", str(repo)]) == 1

    def test_happy_path_exits_0_and_prints_message(
        self, tmp_path: Path, capsys: pytest.CaptureFixture[str]
    ) -> None:
        repo = _init_repo(tmp_path)
        (repo / "tests/baselines").mkdir(parents=True)
        (repo / "tests/baselines/glut.csv").write_text(_CSV_HEADER + "0,1.0,0.0\n")
        _git(repo, "add", "tests/baselines/glut.csv")
        exit_code = main(
            [
                "--slug",
                "glut-seed-2026-07-21",
                "--summary",
                "seed glut baseline",
                "--repo",
                str(repo),
            ]
        )
        assert exit_code == 0
        out = capsys.readouterr().out
        assert message_declares_ceremony(out)
        assert "glut-seed-2026-07-21" in out

    def test_bad_repo_is_git_error(self, tmp_path: Path) -> None:
        not_a_repo = tmp_path / "plain"
        not_a_repo.mkdir()
        assert main(["--slug", "x", "--summary", "y", "--repo", str(not_a_repo)]) == 2


class TestEnsembleReportTable:
    """P27 §8 extension: an optional second Markdown table for stochastic
    families, rendered from ``--ensemble-report <path.json>``. The
    deterministic drift-table output must stay byte-unchanged when the flag
    is absent."""

    def test_ensemble_report_renders_second_table(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        (repo / "tests/baselines").mkdir(parents=True)
        (repo / "tests/baselines/glut.csv").write_text(_CSV_HEADER + "0,1.0,0.0\n")
        _git(repo, "add", "tests/baselines/glut.csv")

        report = tmp_path / "ensemble.json"
        report.write_text(
            json.dumps(
                {
                    "families": [
                        {
                            "family": "electoral.turnout",
                            "n": 32,
                            "envelope": "outcome counts within +/-2 of reference",
                            "observed": "REVOLUTIONARY_VICTORY 3/32 (ref 4/32)",
                            "pass": True,
                        }
                    ]
                }
            )
        )
        out = subprocess.run(
            [
                sys.executable,
                str(TOOL_SCRIPT),
                "--slug",
                "test-slug",
                "--summary",
                "test",
                "--repo",
                str(repo),
                "--ensemble-report",
                str(report),
            ],
            capture_output=True,
            text=True,
        )
        assert out.returncode == 0, out.stderr
        assert "electoral.turnout" in out.stdout
        assert "Baselines: blessed(test-slug)" in out.stdout
        assert message_declares_ceremony(out.stdout)

    def test_without_flag_output_shape_unchanged(self, tmp_path: Path) -> None:
        repo = _init_repo(tmp_path)
        (repo / "tests/baselines").mkdir(parents=True)
        (repo / "tests/baselines/glut.csv").write_text(_CSV_HEADER + "0,1.0,0.0\n")
        _git(repo, "add", "tests/baselines/glut.csv")

        out = subprocess.run(
            [
                sys.executable,
                str(TOOL_SCRIPT),
                "--slug",
                "test-slug",
                "--summary",
                "test",
                "--repo",
                str(repo),
            ],
            capture_output=True,
            text=True,
        )
        assert out.returncode == 0, out.stderr
        assert "ensemble" not in out.stdout.lower()

    def test_malformed_family_row_fails_loudly(self, tmp_path: Path) -> None:
        """III.11: a family row missing a required key must error, not
        silently render a partial/blank row."""
        repo = _init_repo(tmp_path)
        (repo / "tests/baselines").mkdir(parents=True)
        (repo / "tests/baselines/glut.csv").write_text(_CSV_HEADER + "0,1.0,0.0\n")
        _git(repo, "add", "tests/baselines/glut.csv")

        report = tmp_path / "ensemble.json"
        report.write_text(json.dumps({"families": [{"family": "electoral.turnout", "n": 32}]}))
        out = subprocess.run(
            [
                sys.executable,
                str(TOOL_SCRIPT),
                "--slug",
                "test-slug",
                "--summary",
                "test",
                "--repo",
                str(repo),
                "--ensemble-report",
                str(report),
            ],
            capture_output=True,
            text=True,
        )
        assert out.returncode == 2
        assert "envelope" in out.stderr or "pass" in out.stderr
