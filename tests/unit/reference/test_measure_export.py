"""Red-phase / contract tests for the ``export_table_parquet`` writer and
``governed_db_tables`` helper (Task 1 of the parquet-canonical reference
pipeline plan, 2026-07-19).

These pin the two production interfaces later plan tasks depend on by exact
name: a single reusable per-table parquet writer, and the full-DB table
enumeration the Phase-0 measurement CLI sweeps.
"""

from __future__ import annotations

import hashlib
import sqlite3
import sys
from pathlib import Path

import pyarrow.parquet as pq  # type: ignore[import-untyped]
import pytest

_REPO_ROOT = Path(__file__).resolve().parents[3]
TOOLS_DIR = _REPO_ROOT / "tools"
sys.path.insert(0, str(TOOLS_DIR))

import make_data_artifacts  # type: ignore[import-not-found]  # noqa: E402
from make_data_artifacts import (  # type: ignore[import-not-found]  # noqa: E402
    export_table_parquet,
    governed_db_tables,
)


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def test_export_table_parquet_writes_deterministic_file(tmp_path: Path) -> None:
    conn = sqlite3.connect(tmp_path / "mini.sqlite")
    conn.execute("CREATE TABLE dim_x (id INTEGER PRIMARY KEY, name TEXT, v REAL)")
    conn.executemany("INSERT INTO dim_x VALUES (?,?,?)", [(2, "b", 1.5), (1, "a", 0.5)])
    conn.commit()
    rows, size = export_table_parquet(conn, "dim_x", tmp_path / "a.parquet")
    rows2, _ = export_table_parquet(conn, "dim_x", tmp_path / "b.parquet")
    assert rows == rows2 == 2
    assert size == (tmp_path / "a.parquet").stat().st_size
    assert _sha256(tmp_path / "a.parquet") == _sha256(tmp_path / "b.parquet")


def test_export_table_without_pk_sorts_by_all_columns(tmp_path: Path) -> None:
    conn = sqlite3.connect(tmp_path / "mini.sqlite")
    conn.execute("CREATE TABLE staging_y (a INTEGER, b TEXT)")  # no PK
    conn.executemany("INSERT INTO staging_y VALUES (?,?)", [(2, "z"), (1, "y")])
    conn.commit()
    rows, _ = export_table_parquet(conn, "staging_y", tmp_path / "y.parquet")
    assert rows == 2
    assert pq.read_table(tmp_path / "y.parquet").column("a").to_pylist() == [1, 2]


def test_export_table_parquet_coerces_int_valued_boolean_columns(tmp_path: Path) -> None:
    # Regression: sqlite has no native boolean storage class — a BOOLEAN
    # column round-trips through the stdlib driver as plain int (0/1), and
    # pyarrow's pa.array(ints, type=pa.bool_()) refuses to coerce
    # (ArrowInvalid). Discovered running the Phase-0 full-DB sweep against
    # bridge_county_metro.is_principal_city.
    conn = sqlite3.connect(tmp_path / "mini.sqlite")
    conn.execute("CREATE TABLE dim_flag (id INTEGER PRIMARY KEY, is_active BOOLEAN)")
    conn.executemany("INSERT INTO dim_flag VALUES (?,?)", [(1, 1), (2, 0), (3, None)])
    conn.commit()
    rows, _ = export_table_parquet(conn, "dim_flag", tmp_path / "flag.parquet")
    assert rows == 3
    assert pq.read_table(tmp_path / "flag.parquet").column("is_active").to_pylist() == [
        True,
        False,
        None,
    ]


def test_governed_db_tables_excludes_internal(tmp_path: Path) -> None:
    conn = sqlite3.connect(tmp_path / "m.sqlite")
    conn.execute("CREATE TABLE fact_a (id INTEGER PRIMARY KEY AUTOINCREMENT, x TEXT)")
    conn.execute("INSERT INTO fact_a (x) VALUES ('q')")  # materializes sqlite_sequence
    conn.commit()
    assert governed_db_tables(conn) == ["fact_a"]


def test_export_table_parquet_handles_empty_table(tmp_path: Path) -> None:
    conn = sqlite3.connect(tmp_path / "mini.sqlite")
    conn.execute("CREATE TABLE dim_empty (id INTEGER PRIMARY KEY, name TEXT)")
    conn.commit()
    rows, _size = export_table_parquet(conn, "dim_empty", tmp_path / "a.parquet")
    rows2, _size2 = export_table_parquet(conn, "dim_empty", tmp_path / "b.parquet")
    assert rows == rows2 == 0
    table = pq.read_table(tmp_path / "a.parquet")
    assert table.num_rows == 0
    assert _sha256(tmp_path / "a.parquet") == _sha256(tmp_path / "b.parquet")


def test_export_table_parquet_streams_multiple_row_groups(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Forces the fetchmany-batch loop in export_table_parquet to emit more
    # than one row group, covering the streaming path the curated-table
    # generate() calls never exercise (ROW_GROUP_SIZE=4Mi exceeds every
    # curated table).
    monkeypatch.setattr(make_data_artifacts, "ROW_GROUP_SIZE", 2)
    conn = sqlite3.connect(tmp_path / "mini.sqlite")
    conn.execute("CREATE TABLE dim_multi (id INTEGER PRIMARY KEY, name TEXT)")
    conn.executemany(
        "INSERT INTO dim_multi VALUES (?,?)",
        [(5, "e"), (4, "d"), (3, "c"), (2, "b"), (1, "a")],
    )
    conn.commit()
    rows, _size = export_table_parquet(conn, "dim_multi", tmp_path / "a.parquet")
    rows2, _size2 = export_table_parquet(conn, "dim_multi", tmp_path / "b.parquet")
    assert rows == rows2 == 5
    assert _sha256(tmp_path / "a.parquet") == _sha256(tmp_path / "b.parquet")

    table = pq.read_table(tmp_path / "a.parquet")
    assert table.column("id").to_pylist() == [1, 2, 3, 4, 5]
    assert pq.ParquetFile(tmp_path / "a.parquet").num_row_groups == 3
