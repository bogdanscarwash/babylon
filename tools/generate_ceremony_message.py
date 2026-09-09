#!/usr/bin/env python3
r"""Ceremony-authoring helper — mechanizes §6.5 blessing messages.

Sibling to ``tools/check_baseline_ceremony.py`` (same git-plumbing idiom,
same doctrine). That tool *validates* a ceremony declaration; this one
*generates* one, so committers stop hand-writing the drift table.

Given a working tree with staged changes under ``tests/baselines/**``, this
tool:

1. Finds the staged baseline paths (``git diff --cached --name-only``).
2. For each, diffs the working copy against its pre-image (``git show
   HEAD:<path>``, or "new file" if the path did not exist at ``HEAD``) and
   renders a **drift row**: row-count change, changed-cell count, and (where
   the file parses as CSV with numeric columns) the max absolute delta
   across matched cells. Non-CSV files (e.g. the scenario ``.json`` summaries)
   get a coarse line-level drift count instead of a fabricated numeric delta.
3. Emits the full commit-message skeleton: the ``test(baselines): <summary>``
   subject, a Markdown drift table body, and the
   ``Baselines: blessed(<slug>)`` trailer — the exact anatomy
   ``tools/check_baseline_ceremony.py`` validates and CLAUDE.md's "Baseline
   ceremonies" section documents.

The generated message is not auto-committed: print it, review it, paste it
(or pipe it straight into ``git commit -F -``). This tool never touches
``tests/baselines/**`` itself — read-only over the working tree and git
history.

Optionally, ``--ensemble-report <path.json>`` (P27 spec §8) inserts a second
Markdown table for stochastic families between the drift table and the
trailer — one row per ``{family, n, envelope, observed, pass}`` object in the
report's top-level ``families`` list. A malformed row (missing any of those
keys) fails loudly (exit 2) rather than rendering a partial table. Without
the flag, output is byte-unchanged from before it existed.

Run: ``python3 tools/generate_ceremony_message.py --slug <slug> --summary
"<what and why>"`` with baseline changes staged. Exit codes: 0 = message
printed, 1 = no staged baseline changes to describe, 2 = usage, git, or
ensemble-report failure.
"""

from __future__ import annotations

import argparse
import csv
import io
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

# The four required keys of one ensemble-family row (P27 spec §8). Fail loudly
# (III.11) on anything missing rather than rendering a partial/blank row.
_ENSEMBLE_FAMILY_KEYS = ("family", "n", "envelope", "observed", "pass")

# Reuse the governed prefix + trailer grammar from the validator so the two
# tools can never silently drift apart on what counts as "in scope" or
# "well-formed".
sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_baseline_ceremony import (  # type: ignore[import-not-found]  # noqa: E402
    BASELINE_PREFIXES,
    message_declares_ceremony,
)

_EPSILON = 1e-9


@dataclass(frozen=True)
class DriftRow:
    """One drift-table row for a single changed baseline file."""

    path: str
    status: str
    old_rows: int | None
    new_rows: int | None
    changed_cells: int | None
    max_abs_delta: float | None
    parseable: bool

    def render(self) -> str:
        """Render as one Markdown table cell-row."""
        rows = (
            f"{self.old_rows}→{self.new_rows}"
            if self.old_rows is not None and self.new_rows is not None
            else "n/a"
        )
        cells = str(self.changed_cells) if self.changed_cells is not None else "n/a"
        delta = f"{self.max_abs_delta:.6g}" if self.max_abs_delta is not None else "n/a"
        return f"| {self.path} | {self.status} | {rows} | {cells} | {delta} |"


def _git(repo_root: Path, *args: str) -> str:
    """Run a git plumbing command, raising ``CalledProcessError`` on failure."""
    result = subprocess.run(
        ["git", *args],
        cwd=repo_root,
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout


def _read_pre_image(repo_root: Path, path: str) -> str | None:
    """Return the ``HEAD`` version of ``path``, or ``None`` if it is new."""
    try:
        return _git(repo_root, "show", f"HEAD:{path}")
    except subprocess.CalledProcessError:
        return None


def _read_staged_image(repo_root: Path, path: str) -> str:
    """Return the staged (index) version of ``path``."""
    return _git(repo_root, "show", f":{path}")


def _parse_csv_rows(text: str) -> list[list[str]] | None:
    """Parse ``text`` as CSV; ``None`` if it does not look tabular."""
    try:
        rows = list(csv.reader(io.StringIO(text)))
    except csv.Error:
        return None
    if not rows or len(rows[0]) < 2:
        return None
    width = len(rows[0])
    if any(len(row) != width for row in rows[1:]):
        return None
    return rows


def _numeric(value: str) -> float | None:
    """Best-effort float parse; ``None`` for non-numeric cells."""
    try:
        return float(value)
    except ValueError:
        return None


def _diff_csv(old_rows: list[list[str]], new_rows: list[list[str]]) -> tuple[int, float | None]:
    """Return (changed_cells, max_abs_delta) across the matched row range."""
    changed_cells = 0
    max_abs_delta: float | None = None
    matched = min(len(old_rows), len(new_rows))
    for r in range(matched):
        old_row, new_row = old_rows[r], new_rows[r]
        width = min(len(old_row), len(new_row))
        for c in range(width):
            old_cell, new_cell = old_row[c], new_row[c]
            if old_cell == new_cell:
                continue
            changed_cells += 1
            old_num, new_num = _numeric(old_cell), _numeric(new_cell)
            if old_num is not None and new_num is not None:
                delta = abs(new_num - old_num)
                if max_abs_delta is None or delta > max_abs_delta:
                    max_abs_delta = delta
    return changed_cells, max_abs_delta


def _diff_lines(old_text: str, new_text: str) -> int:
    """Coarse non-CSV drift count: number of differing lines by position."""
    old_lines = old_text.splitlines()
    new_lines = new_text.splitlines()
    matched = min(len(old_lines), len(new_lines))
    changed = sum(1 for i in range(matched) if old_lines[i] != new_lines[i])
    changed += abs(len(old_lines) - len(new_lines))
    return changed


def build_drift_row(repo_root: Path, path: str) -> DriftRow:
    """Compute one drift row for a staged baseline path."""
    old_text = _read_pre_image(repo_root, path)
    new_text = _read_staged_image(repo_root, path)

    if old_text is None:
        new_rows = _parse_csv_rows(new_text)
        row_count = len(new_rows) - 1 if new_rows else None
        return DriftRow(
            path=path,
            status="added",
            old_rows=0 if row_count is not None else None,
            new_rows=row_count,
            changed_cells=None,
            max_abs_delta=None,
            parseable=new_rows is not None,
        )

    old_csv = _parse_csv_rows(old_text)
    new_csv = _parse_csv_rows(new_text)
    if old_csv is not None and new_csv is not None:
        changed_cells, max_abs_delta = _diff_csv(old_csv, new_csv)
        return DriftRow(
            path=path,
            status="modified" if changed_cells or len(old_csv) != len(new_csv) else "unchanged",
            old_rows=len(old_csv) - 1,
            new_rows=len(new_csv) - 1,
            changed_cells=changed_cells,
            max_abs_delta=max_abs_delta,
            parseable=True,
        )

    changed_lines = _diff_lines(old_text, new_text)
    return DriftRow(
        path=path,
        status="modified" if changed_lines else "unchanged",
        old_rows=None,
        new_rows=None,
        changed_cells=changed_lines,
        max_abs_delta=None,
        parseable=False,
    )


def staged_baseline_paths(repo_root: Path) -> list[str]:
    """Every staged path under the governed baseline estate."""
    staged_raw = _git(repo_root, "diff", "--cached", "--name-only")
    return [line for line in staged_raw.splitlines() if line and line.startswith(BASELINE_PREFIXES)]


def build_drift_table(rows: list[DriftRow]) -> str:
    """Render the full Markdown drift table body."""
    header = "| file | status | rows (old→new) | changed cells | max \\|Δ\\| |"
    divider = "| --- | --- | --- | --- | --- |"
    lines = [header, divider, *(row.render() for row in rows)]
    return "\n".join(lines)


def load_ensemble_families(path: Path) -> list[dict[str, object]]:
    """Load + schema-check ``--ensemble-report`` JSON (P27 spec §8).

    Raises ``ValueError`` with a precise message on any malformed row —
    III.11 loud failure, never a silently partial table.
    """
    data = json.loads(path.read_text())
    families = data.get("families")
    if not isinstance(families, list):
        raise ValueError(f"ensemble report {path} has no top-level 'families' list")
    for i, family in enumerate(families):
        if not isinstance(family, dict):
            raise ValueError(f"ensemble report {path} families[{i}] is not an object")
        missing = [key for key in _ENSEMBLE_FAMILY_KEYS if key not in family]
        if missing:
            raise ValueError(
                f"ensemble report {path} families[{i}] missing required key(s): "
                f"{', '.join(missing)}"
            )
    return families


def build_ensemble_table(families: list[dict[str, object]]) -> str:
    """Render the second Markdown table for stochastic families."""
    header = "| family | N | envelope | observed | pass |"
    divider = "| --- | --- | --- | --- | --- |"
    lines = [header, divider]
    for family in families:
        lines.append(
            f"| {family['family']} | {family['n']} | {family['envelope']} | "
            f"{family['observed']} | {family['pass']} |"
        )
    return "\n".join(lines)


def build_ceremony_message(
    slug: str,
    summary: str,
    rows: list[DriftRow],
    ensemble_families: list[dict[str, object]] | None = None,
) -> str:
    """Assemble the full commit-message skeleton (subject/body/trailer).

    When ``ensemble_families`` is given, a second Markdown table is inserted
    between the drift table and the trailer; the drift-table-only path is
    byte-unchanged from before this parameter existed.
    """
    subject = f"test(baselines): {summary}"
    body = build_drift_table(rows)
    trailer = f"Baselines: blessed({slug})"
    if ensemble_families:
        ensemble_table = build_ensemble_table(ensemble_families)
        return f"{subject}\n\n{body}\n\n{ensemble_table}\n\n{trailer}\n"
    return f"{subject}\n\n{body}\n\n{trailer}\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Generate a §6.5 ceremony commit-message skeleton from staged baseline drift"
    )
    parser.add_argument(
        "--slug",
        required=True,
        help="ceremony slug, e.g. post-merge-regen-2026-07-20 (lowercase [a-z0-9-])",
    )
    parser.add_argument(
        "--summary",
        required=True,
        help="one-line subject text describing what moved and why",
    )
    parser.add_argument(
        "--repo",
        type=Path,
        default=Path.cwd(),
        help="repository root (default: current directory)",
    )
    parser.add_argument(
        "--ensemble-report",
        type=Path,
        default=None,
        help=(
            "path to a JSON report with a 'families' list of stochastic-family "
            "rows ({family, n, envelope, observed, pass}); when given, a second "
            "Markdown table is inserted between the drift table and the trailer"
        ),
    )
    args = parser.parse_args(argv)

    ensemble_families: list[dict[str, object]] | None = None
    if args.ensemble_report is not None:
        try:
            ensemble_families = load_ensemble_families(args.ensemble_report)
        except (OSError, ValueError, json.JSONDecodeError) as exc:
            print(f"generate_ceremony_message: {exc}", file=sys.stderr)
            return 2

    try:
        paths = staged_baseline_paths(args.repo)
    except subprocess.CalledProcessError as exc:
        print(f"generate_ceremony_message: git failed: {exc.stderr.strip()}", file=sys.stderr)
        return 2

    if not paths:
        print(
            "generate_ceremony_message: no staged changes under tests/baselines/ — nothing to bless",
            file=sys.stderr,
        )
        return 1

    try:
        rows = [build_drift_row(args.repo, path) for path in paths]
    except subprocess.CalledProcessError as exc:
        print(f"generate_ceremony_message: git failed: {exc.stderr.strip()}", file=sys.stderr)
        return 2

    message = build_ceremony_message(args.slug, args.summary, rows, ensemble_families)
    assert message_declares_ceremony(message), (
        "generated message failed its own trailer grammar — this is a bug in "
        "generate_ceremony_message.py, not a usage error"
    )
    print(message)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
