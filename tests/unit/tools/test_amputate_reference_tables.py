"""Behavioral contract for the one-shot reference-DB amputation tool (ADR075).

Owner ruling 1 (2026-07-16, recorded in ``ai/decisions/ADR075_data_constitution.yaml``
``rulings_resolved``) approved 16 amputations from the 23 census proposals. This
tool executes 15 of them — 14 tables plus ``view_labor_type`` (which dies with
its A5 base table ``fact_census_occupation``). The 16th, A1
``fact_qcew_annual__pre_086``, rides the dedicated spec-067 CLI
(``tools/normalize_qcew_rollups.py --drop-backup``) so the staged-swap
mechanism keeps its own audit trail. A21 ``dim_education_level`` is DEFERRED:
it is an FK parent (primary-key member) of ``fact_census_education``, whose
disposition is ``investigate`` — dropping the dim would orphan a living table.

All tests run against tiny synthetic sqlite fixtures in ``tmp_path`` — never
the real 5.7 GB database (test-estate law: PG/refdb-requiring tests are
integration tier; this is pure-unit by construction).
"""

from __future__ import annotations

import sqlite3
import sys
from pathlib import Path

import pytest

# Mirror the import path used by tools/*.py unit tests
# (see test_make_reference_subset.py, test_run_pip_audit.py).
TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

from amputate_reference_tables import (  # type: ignore[import-not-found]  # noqa: E402
    APPROVED_DROPS,
    BATCHES,
    DEMOTION_DROPS,
    AmputationError,
    find_stray_readers,
    main,
    verify_all_present,
)

#: The 14 tables + 1 view the owner approved for this tool (A-numbers from the
#: triage matrix). Pinned here so the tool's module-level list cannot drift
#: from the ruling without this contract going red.
_RULED_DROPS: tuple[tuple[str, str], ...] = (
    ("view_labor_type", "view"),
    ("fact_atus_reproductive_labor", "table"),
    ("fact_bls_productivity", "table"),
    ("fact_census_commute", "table"),
    ("fact_census_gini", "table"),
    ("fact_census_occupation", "table"),
    ("fact_employment_industry_annual", "table"),
    ("fact_fred_industry_unemployment", "table"),
    ("fact_fred_state_unemployment", "table"),
    ("fact_hickel_drain", "table"),
    ("fact_qcew_metro_annual", "table"),
    ("fact_qcew_state_annual", "table"),
    ("dim_atus_activity_category", "table"),
    ("dim_commute_mode", "table"),
    ("dim_sector", "table"),
)


def _build_fixture_db(db_path: Path) -> None:
    """Create every doomed object plus survivors in a throwaway sqlite DB."""
    conn = sqlite3.connect(db_path)
    try:
        for name, kind in _RULED_DROPS:
            if kind == "table":
                conn.execute(f'CREATE TABLE "{name}" (id INTEGER PRIMARY KEY)')
        conn.execute('INSERT INTO "fact_census_gini" (id) VALUES (1), (2)')
        # The doomed view reads a doomed base — mirrors the real DB shape.
        conn.execute("CREATE VIEW view_labor_type AS SELECT id FROM fact_census_occupation")
        # Survivors that must be untouched.
        conn.execute("CREATE TABLE fact_qcew_annual (id INTEGER PRIMARY KEY)")
        conn.execute("INSERT INTO fact_qcew_annual (id) VALUES (7)")
        conn.execute("CREATE VIEW view_survivor AS SELECT id FROM fact_qcew_annual")
        conn.commit()
    finally:
        conn.close()


def _object_names(db_path: Path) -> set[str]:
    conn = sqlite3.connect(db_path)
    try:
        rows = conn.execute(
            "SELECT name FROM sqlite_master WHERE type IN ('table', 'view')"
        ).fetchall()
    finally:
        conn.close()
    return {row[0] for row in rows}


@pytest.fixture()
def fixture_db(tmp_path: Path) -> Path:
    db_path = tmp_path / "fixture.sqlite"
    _build_fixture_db(db_path)
    return db_path


#: The ADR076 demotion batch (rulings R1-R5): DB copies whose truth moved to
#: hash-pinned artifacts in data-artifacts.yaml. Pinned like the ruling-1 list.
_DEMOTION_RULED: tuple[tuple[str, str], ...] = (
    ("view_energy_consumption", "view"),
    ("bridge_county_bea_ea", "table"),
    ("dim_bea_economic_area", "table"),
    ("fact_ricci_unequal_exchange", "table"),
    ("fact_energy_annual", "table"),
    ("dim_energy_series", "table"),
    ("dim_energy_table", "table"),
    ("bridge_lodes_block", "table"),
    ("staging_arcgis_feature", "table"),
)


class TestApprovedList:
    """The module-level lists ARE the owner rulings — byte-pinned."""

    def test_approved_drops_match_the_ruling_exactly(self) -> None:
        assert set(APPROVED_DROPS) == set(_RULED_DROPS)
        assert len(APPROVED_DROPS) == 15

    def test_deferred_and_delegated_objects_are_absent(self) -> None:
        names = {name for name, _ in APPROVED_DROPS}
        assert "dim_education_level" not in names  # deferred: FK parent of a living table
        assert "fact_qcew_annual__pre_086" not in names  # rides --drop-backup CLI

    def test_demotion_drops_match_adr076_exactly(self) -> None:
        assert set(DEMOTION_DROPS) == set(_DEMOTION_RULED)
        assert len(DEMOTION_DROPS) == 9

    def test_batches_expose_both_rulings(self) -> None:
        assert BATCHES == {"ruling1": APPROVED_DROPS, "demotion": DEMOTION_DROPS}


class TestVerification:
    def test_verify_all_present_passes_on_complete_fixture(self, fixture_db: Path) -> None:
        conn = sqlite3.connect(fixture_db)
        try:
            assert verify_all_present(conn) == []
        finally:
            conn.close()

    def test_verify_reports_every_missing_object(self, fixture_db: Path) -> None:
        conn = sqlite3.connect(fixture_db)
        try:
            conn.execute("DROP TABLE fact_hickel_drain")
            conn.execute("DROP TABLE dim_sector")
            missing = verify_all_present(conn)
        finally:
            conn.close()
        assert sorted(missing) == ["dim_sector", "fact_hickel_drain"]

    def test_stray_reader_detection_flags_surviving_view_over_doomed_base(
        self, fixture_db: Path
    ) -> None:
        conn = sqlite3.connect(fixture_db)
        try:
            conn.execute("CREATE VIEW view_stray AS SELECT id FROM fact_census_gini")
            strays = find_stray_readers(conn)
        finally:
            conn.close()
        assert strays == ["view_stray"]

    def test_no_strays_on_clean_fixture(self, fixture_db: Path) -> None:
        conn = sqlite3.connect(fixture_db)
        try:
            assert find_stray_readers(conn) == []
        finally:
            conn.close()


class TestDryRunAndExecute:
    def test_dry_run_is_the_default_and_drops_nothing(
        self, fixture_db: Path, tmp_path: Path
    ) -> None:
        before = _object_names(fixture_db)
        exit_code = main(["--db", str(fixture_db), "--report-dir", str(tmp_path / "reports")])
        assert exit_code == 0
        assert _object_names(fixture_db) == before

    def test_execute_drops_exactly_the_approved_objects(
        self, fixture_db: Path, tmp_path: Path
    ) -> None:
        report_dir = tmp_path / "reports"
        exit_code = main(["--db", str(fixture_db), "--report-dir", str(report_dir), "--execute"])
        assert exit_code == 0
        remaining = _object_names(fixture_db)
        assert {name for name, _ in APPROVED_DROPS}.isdisjoint(remaining)
        assert "fact_qcew_annual" in remaining
        assert "view_survivor" in remaining
        reports = list(report_dir.glob("amputation_*.md"))
        assert len(reports) == 1
        body = reports[0].read_text()
        assert "fact_census_gini" in body
        assert "rows_before=2" in body

    def test_missing_object_aborts_loudly_and_drops_nothing(
        self, fixture_db: Path, tmp_path: Path
    ) -> None:
        conn = sqlite3.connect(fixture_db)
        conn.execute("DROP TABLE fact_bls_productivity")
        conn.commit()
        conn.close()
        before = _object_names(fixture_db)
        with pytest.raises(AmputationError, match="fact_bls_productivity"):
            main(
                [
                    "--db",
                    str(fixture_db),
                    "--report-dir",
                    str(tmp_path / "reports"),
                    "--execute",
                ]
            )
        assert _object_names(fixture_db) == before

    def test_demotion_batch_drops_exactly_the_adr076_objects(self, tmp_path: Path) -> None:
        db_path = tmp_path / "demotion.sqlite"
        conn = sqlite3.connect(db_path)
        for name, kind in _DEMOTION_RULED:
            if kind == "table":
                conn.execute(f'CREATE TABLE "{name}" (id INTEGER PRIMARY KEY)')
        conn.execute("CREATE VIEW view_energy_consumption AS SELECT id FROM fact_energy_annual")
        conn.execute("CREATE TABLE fact_survivor (id INTEGER PRIMARY KEY)")
        conn.commit()
        conn.close()
        exit_code = main(
            [
                "--batch",
                "demotion",
                "--db",
                str(db_path),
                "--report-dir",
                str(tmp_path / "reports"),
                "--execute",
            ]
        )
        assert exit_code == 0
        remaining = _object_names(db_path)
        assert {name for name, _ in _DEMOTION_RULED}.isdisjoint(remaining)
        assert "fact_survivor" in remaining
        reports = list((tmp_path / "reports").glob("amputation_demotion_*.md"))
        assert len(reports) == 1

    def test_stray_reader_aborts_execute_and_drops_nothing(
        self, fixture_db: Path, tmp_path: Path
    ) -> None:
        conn = sqlite3.connect(fixture_db)
        conn.execute("CREATE VIEW view_stray AS SELECT id FROM fact_census_gini")
        conn.commit()
        conn.close()
        before = _object_names(fixture_db)
        with pytest.raises(AmputationError, match="view_stray"):
            main(
                [
                    "--db",
                    str(fixture_db),
                    "--report-dir",
                    str(tmp_path / "reports"),
                    "--execute",
                ]
            )
        assert _object_names(fixture_db) == before
