"""Reference-DB ↔ in-repo CSV sync guards for the unequal-exchange tables.

The shipped CSVs (``src/babylon/data/reference/``) are the source of truth;
``tools/ingest/hickel_erdi.py`` loads the ERDI series into the reference
SQLite. These tests prove the DB actually carries what the CSV says — catching partial ingests, silent re-ingest drift, and the
schema-only-table failure mode this data sat in until 2026-07-16
(``fact_ricci_unequal_exchange`` existed with 0 rows for months while the
raw CSV waited in the trove).
"""

from __future__ import annotations

import csv
import sqlite3
from pathlib import Path

import pytest

SQLITE_REF = Path("data/sqlite/marxist-data-3NF.sqlite")
_REFERENCE_DIR = Path(__file__).resolve().parents[3] / "src" / "babylon" / "data" / "reference"
HICKEL_CSV = _REFERENCE_DIR / "babylon_hickel_final.csv"

pytestmark = [
    pytest.mark.requires_reference_db,
    pytest.mark.skipif(not SQLITE_REF.exists(), reason=f"reference DB missing at {SQLITE_REF}"),
]


def _connect() -> sqlite3.Connection:
    return sqlite3.connect(f"file:{SQLITE_REF}?mode=ro", uri=True)


def test_hickel_erdi_table_matches_csv() -> None:
    """fact_hickel_erdi_annual carries every CSV row, keyed (year, scale_type)."""
    with HICKEL_CSV.open() as fh:
        expected = {
            (int(r["year"]), r["scale_type"].strip()): (
                float(r["erdi"]),
                float(r["annual_drain_usd_billions"]),
            )
            for r in csv.DictReader(fh)
        }
    with _connect() as conn:
        rows = conn.execute(
            "SELECT dt.year, h.scale_type, h.erdi, h.annual_drain_usd_billions "
            "FROM fact_hickel_erdi_annual h JOIN dim_time dt ON dt.time_id = h.time_id"
        ).fetchall()
    actual = {(int(y), str(s)): (float(e), float(d)) for y, s, e, d in rows}
    assert set(actual) == set(expected), (
        f"key drift: db-only={sorted(set(actual) - set(expected))[:3]} "
        f"csv-only={sorted(set(expected) - set(actual))[:3]}"
    )
    for key, (erdi, drain) in expected.items():
        db_erdi, db_drain = actual[key]
        assert abs(db_erdi - erdi) <= 1e-9 and abs(db_drain - drain) <= 1e-6, (
            f"{key}: DB ({db_erdi}, {db_drain}) != CSV ({erdi}, {drain})"
        )
