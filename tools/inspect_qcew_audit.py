"""Inspect a spec-067 QCEW normalization audit-report sidecar.

Run after `tools/normalize_qcew_rollups.py --apply` produces an audit
artifact under ``reports/ingest/qcew_normalization_*.json``. Prints a
human-readable summary of the rollup-class accounting, BLS-suppressed
county-years, and per-county delta distribution.

Usage::

    uv run python tools/inspect_qcew_audit.py
    uv run python tools/inspect_qcew_audit.py --path reports/ingest/qcew_normalization_20260516-220000.json
"""

from __future__ import annotations

import argparse
import json
import sys
from collections.abc import Iterable
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parent.parent
_DEFAULT_REPORT_DIR = _REPO_ROOT / "reports" / "ingest"


def _resolve_audit_json(explicit_path: Path | None) -> Path:
    if explicit_path is not None:
        if not explicit_path.exists():
            raise FileNotFoundError(f"audit report not found: {explicit_path}")
        return explicit_path
    candidates = sorted(_DEFAULT_REPORT_DIR.glob("qcew_normalization_*.json"))
    if not candidates:
        raise FileNotFoundError(
            f"no audit report under {_DEFAULT_REPORT_DIR} — run "
            "`uv run python tools/normalize_qcew_rollups.py --apply` first"
        )
    return candidates[-1]


def _print_summary(report: dict[str, object]) -> None:
    meta = report["run_metadata"]
    rc = report["row_counts"]
    excl = rc["rows_excluded"]  # type: ignore[index]

    print(f"# QCEW Normalization Audit — {meta['timestamp_utc']}")  # type: ignore[index]
    print(f"  Migration version: {meta['migration_version']}")  # type: ignore[index]
    print(f"  Database: {meta['database_path']}")  # type: ignore[index]
    print(f"  Duration: {meta['migration_duration_seconds']:.1f} s")  # type: ignore[index]
    print(f"  Git: {meta['git_branch']} @ {meta['git_sha']}")  # type: ignore[index]
    print()
    print("## Row counts")
    print(f"  Pre-migration:       {rc['fact_qcew_annual_pre']:>12,}")  # type: ignore[index]
    print(f"  Post-migration:      {rc['fact_qcew_annual_post']:>12,}")  # type: ignore[index]
    print(f"  Excluded total:      {excl['total']:>12,}")  # type: ignore[index]
    print(f"    naics_only:        {excl['naics_only']:>12,}")  # type: ignore[index]
    print(f"    ownership_only:    {excl['ownership_only']:>12,}")  # type: ignore[index]
    print(f"    both_axes:         {excl['both_axes']:>12,}")  # type: ignore[index]
    print(f"  Integrity passed:    {rc['integrity_check_passed']}")  # type: ignore[index]


def _print_deltas(report: dict[str, object]) -> None:
    deltas = report["per_county_deltas"]
    stats = deltas["summary_stats"]  # type: ignore[index]
    print()
    print("## Per-county delta distribution")
    scope = "Michigan-only" if deltas["michigan_scope_only"] else "All US"  # type: ignore[index]
    print(f"  Scope:               {scope}")
    print(
        f"  Within ±5 % band:    {stats['counties_within_5pct_band']} pairs "  # type: ignore[index]
        f"({stats['counties_within_5pct_band_pct']:.2f} %)"
    )  # type: ignore[index]
    print(f"  |delta| > 10 %:      {stats['counties_with_delta_gt_10pct']} pairs")  # type: ignore[index]
    print(f"  Max |delta|:         {stats['max_abs_delta_pct']:.2f} %")  # type: ignore[index]


def _print_outliers(report: dict[str, object], n: int = 5) -> None:
    deltas = report["per_county_deltas"]
    outliers = deltas["outliers"][:n]  # type: ignore[index]
    if not outliers:
        return
    print()
    print(f"## Top {len(outliers)} outliers (|delta| > 10 %)")
    print(f"  {'fips':>7} {'year':>5} {'pre_sum':>12} {'post_sum':>12} {'delta_pct':>10}")
    for o in outliers:
        print(
            f"  {o['county_fips']:>7} {o['year']:>5} "
            f"{o['pre_sum']:>12,.0f} {o['post_sum']:>12,.0f} "
            f"{o['delta_pct']:>+9.2f}%"
        )


def _print_vintages(report: dict[str, object]) -> None:
    vintages = report["naics_vintages"]
    if not vintages:
        return
    print()
    print("## NAICS vintages")
    by_vintage: dict[str, list[str]] = {}
    for year, vintage in sorted(vintages.items()):  # type: ignore[union-attr]
        by_vintage.setdefault(vintage, []).append(year)
    for vintage, years in sorted(by_vintage.items()):
        print(f"  NAICS {vintage}: {', '.join(years)}")


def _print_suppressed(report: dict[str, object]) -> None:
    suppressed = report["bls_suppressed_county_years"]
    if not suppressed:
        return
    print()
    print(f"## BLS-suppressed county-years ({len(suppressed)} pairs)")
    for entry in suppressed[:10]:  # type: ignore[index]
        print(f"  {entry['county_fips']} {entry['year']}: {entry['reason']}")
    if len(suppressed) > 10:  # type: ignore[arg-type]
        print(f"  ... and {len(suppressed) - 10} more")  # type: ignore[arg-type]


def main(argv: Iterable[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="inspect_qcew_audit",
        description="Print a human-readable summary of a spec-067 audit-report JSON sidecar.",
    )
    parser.add_argument(
        "--path",
        type=Path,
        default=None,
        help="Explicit path to a qcew_normalization_*.json file. Defaults to the most recent.",
    )
    parser.add_argument(
        "--outliers",
        type=int,
        default=5,
        help="Number of top outliers to print (default: 5).",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    try:
        audit_path = _resolve_audit_json(args.path)
    except FileNotFoundError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    report: dict[str, object] = json.loads(audit_path.read_text(encoding="utf-8"))
    print(f"# Source: {audit_path}\n")
    _print_summary(report)
    _print_vintages(report)
    _print_suppressed(report)
    _print_deltas(report)
    _print_outliers(report, args.outliers)
    return 0


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
