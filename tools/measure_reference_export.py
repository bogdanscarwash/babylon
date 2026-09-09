#!/usr/bin/env python3
"""Phase-0 measurement for the parquet-canonical pipeline (plan 2026-07-19).

Read-only. Exports every governed table to a temp dir with the pipeline's exact pins,
records per-table rows/bytes plus the container pragmas the builder must pin, and writes
reports/parquet-size-measurement.json. Exits 2 (loud, III.11) if any table exceeds the
release-asset safety threshold — that triggers the plan's sharding contingency (D3).
"""

from __future__ import annotations

import argparse
import json
import sqlite3
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from make_data_artifacts import export_table_parquet, governed_db_tables  # noqa: E402

RELEASE_ASSET_LIMIT_BYTES = 2_000_000_000
SAFETY_FACTOR = 0.9
DEFAULT_DB = "data/sqlite/marxist-data-3NF.sqlite"
REPORT = Path("reports/parquet-size-measurement.json")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--db", default=DEFAULT_DB)
    args = parser.parse_args(argv)

    conn = sqlite3.connect(f"file:{args.db}?mode=ro", uri=True)
    pragmas = {
        name: conn.execute(f"PRAGMA {name}").fetchone()[0]
        for name in ("page_size", "application_id", "user_version", "encoding")
    }
    tables: list[dict[str, object]] = []
    threshold = int(RELEASE_ASSET_LIMIT_BYTES * SAFETY_FACTOR)
    oversized: list[str] = []
    with tempfile.TemporaryDirectory() as tmp:
        for table in governed_db_tables(conn):
            rows, size = export_table_parquet(conn, table, Path(tmp) / f"{table}.parquet")
            tables.append({"table": table, "rows": rows, "parquet_bytes": size})
            if size > threshold:
                oversized.append(table)
            print(f"  {table}: {rows} rows, {size:,} bytes")
    report = {
        "db": args.db,
        "sqlite_runtime": sqlite3.sqlite_version,
        "pragmas": pragmas,
        "threshold_bytes": threshold,
        "total_parquet_bytes": sum(int(t["parquet_bytes"]) for t in tables),  # type: ignore[arg-type]
        "tables": sorted(tables, key=lambda t: -int(t["parquet_bytes"])),  # type: ignore[arg-type]
    }
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text(json.dumps(report, indent=2) + "\n")
    print(f"wrote {REPORT} (total {report['total_parquet_bytes']:,} bytes)")
    if oversized:
        print(
            f"OVERSIZED (> {threshold:,} bytes): {oversized} — sharding contingency D3 "
            "required; STOP and get plan addendum approved.",
            file=sys.stderr,
        )
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
