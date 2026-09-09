"""Behavioral contract for the view_rent_crisis repair (ADR075 ruling 1).

The broken definition joined the three housing facts on (county_id,
source_id) only — a cross-time/cross-race Cartesian product. The repair
joins on the full shared key and exposes year + race_code. Synthetic
fixtures only; the real-DB outcome is pinned by the refdb view contracts in
``tests/unit/reference/test_marxian_views.py``.
"""

from __future__ import annotations

import sqlite3
import sys
from pathlib import Path

import pytest

TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

from repair_rent_crisis_view import (  # type: ignore[import-not-found]  # noqa: E402
    RepairError,
    main,
    needs_repair,
)

_BROKEN_VIEW = """CREATE VIEW view_rent_crisis AS
        SELECT c.fips, fr.median_rent_usd, fi.median_income_usd
        FROM fact_census_rent fr
        JOIN fact_census_median_income fi
            ON fr.county_id = fi.county_id AND fr.source_id = fi.source_id
        JOIN dim_county c ON fr.county_id = c.county_id"""


def _fixture_db(path: Path) -> None:
    """One county, two years x two races, one burden bracket — 4 fact rows each."""
    conn = sqlite3.connect(path)
    conn.executescript(
        """
        CREATE TABLE dim_county (county_id INTEGER PRIMARY KEY, fips VARCHAR(5),
            county_name VARCHAR(100), state_id INTEGER);
        INSERT INTO dim_county VALUES (1, '26163', 'Wayne', 26);
        CREATE TABLE dim_state (state_id INTEGER PRIMARY KEY, state_name VARCHAR(100));
        INSERT INTO dim_state VALUES (26, 'Michigan');
        CREATE TABLE dim_time (time_id INTEGER PRIMARY KEY, year INTEGER);
        INSERT INTO dim_time VALUES (2022, 2022), (2023, 2023);
        CREATE TABLE dim_race (race_id INTEGER PRIMARY KEY, race_code VARCHAR(1));
        INSERT INTO dim_race VALUES (1, 'T'), (2, 'B');
        CREATE TABLE dim_rent_burden (burden_id INTEGER PRIMARY KEY,
            is_cost_burdened BOOLEAN, is_severely_burdened BOOLEAN);
        INSERT INTO dim_rent_burden VALUES (1, 1, 0);

        CREATE TABLE fact_census_rent (county_id INTEGER, source_id INTEGER,
            time_id INTEGER, race_id INTEGER, median_rent_usd NUMERIC);
        CREATE TABLE fact_census_median_income (county_id INTEGER, source_id INTEGER,
            time_id INTEGER, race_id INTEGER, median_income_usd NUMERIC);
        CREATE TABLE fact_census_rent_burden (county_id INTEGER, source_id INTEGER,
            burden_id INTEGER, time_id INTEGER, race_id INTEGER, household_count INTEGER);
        INSERT INTO fact_census_rent VALUES
            (1, 1, 2022, 1, 1000), (1, 1, 2022, 2, 1100),
            (1, 1, 2023, 1, 1200), (1, 1, 2023, 2, 1300);
        INSERT INTO fact_census_median_income VALUES
            (1, 1, 2022, 1, 60000), (1, 1, 2022, 2, 40000),
            (1, 1, 2023, 1, 62000), (1, 1, 2023, 2, 41000);
        INSERT INTO fact_census_rent_burden VALUES
            (1, 1, 1, 2022, 1, 500), (1, 1, 1, 2022, 2, 200),
            (1, 1, 1, 2023, 1, 550), (1, 1, 1, 2023, 2, 220);
        """
    )
    conn.execute(_BROKEN_VIEW)
    conn.commit()
    conn.close()


@pytest.fixture()
def db_path(tmp_path: Path) -> Path:
    path = tmp_path / "housing.sqlite"
    _fixture_db(path)
    return path


def test_broken_view_is_detected(db_path: Path) -> None:
    conn = sqlite3.connect(db_path)
    try:
        assert needs_repair(conn) is True
    finally:
        conn.close()


def test_missing_view_is_loud(tmp_path: Path) -> None:
    empty = tmp_path / "empty.sqlite"
    sqlite3.connect(empty).close()
    conn = sqlite3.connect(empty)
    try:
        with pytest.raises(RepairError, match="missing"):
            needs_repair(conn)
    finally:
        conn.close()


def test_dry_run_leaves_broken_view(db_path: Path) -> None:
    assert main(["--db", str(db_path)]) == 0
    conn = sqlite3.connect(db_path)
    try:
        assert needs_repair(conn) is True
    finally:
        conn.close()


def test_execute_installs_full_key_join_with_per_slice_rows(db_path: Path) -> None:
    assert main(["--db", str(db_path), "--execute"]) == 0
    conn = sqlite3.connect(db_path)
    try:
        assert needs_repair(conn) is False
        rows = conn.execute(
            "SELECT year, race_code, median_rent_usd, median_income_usd,"
            " ROUND(annual_rent_to_income_ratio, 4), cost_burdened_households"
            " FROM view_rent_crisis ORDER BY year, race_code"
        ).fetchall()
    finally:
        conn.close()
    # One row per (year, race) slice — no Cartesian multiplication.
    assert rows == [
        (2022, "B", 1100, 40000, 0.33, 200),
        (2022, "T", 1000, 60000, 0.2, 500),
        (2023, "B", 1300, 41000, 0.3805, 220),
        (2023, "T", 1200, 62000, 0.2323, 550),
    ]


def test_second_run_is_a_clean_noop(db_path: Path) -> None:
    assert main(["--db", str(db_path), "--execute"]) == 0
    assert main(["--db", str(db_path), "--execute"]) == 0
