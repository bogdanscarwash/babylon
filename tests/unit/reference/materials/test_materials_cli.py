"""Synthetic contract for the revived materials loader CLI (Program 22 Wave 1).

Mirrors the qcew CLI test pattern: ``importorskip`` on the external
``babylon_data`` package (absent in CI where the symlink does not resolve),
tiny synthetic staging directories, and a throwaway sqlite built from the
real ORM metadata — never the real DB or the trove.
"""

from __future__ import annotations

import sqlite3
from pathlib import Path

import pytest
from sqlalchemy import create_engine
from sqlalchemy.orm import Session

materials_main = pytest.importorskip(
    "babylon_data.materials.__main__",
    reason="babylon-data symlink not resolved (CI)",
)

from babylon.reference.database import NormalizedBase  # noqa: E402
from babylon.reference.schema import DimState, DimTime  # noqa: E402

_T1_HEADER = (
    "Source,Year,Mine_Production_Metals_mil_dols,Mine_Production_Industrial_mil_dols,"
    "Mine_Production_Coal_mil_dols,Employment_All_Coal_thsnds,Employment_All_Nonfuel_thsnds,"
    "Employment_Chemicals_thsnds,Employment_Stone_Clay_Glass_thsnds,"
    "Employment_Primary_Metal_thsnds,Avg_Weekly_Earnings_All_Coal_dols,"
    "Avg_Weekly_Earnings_dols,Avg_Weekly_Earnings_Stone_Clay_Glass_dols,"
    "Avg_Weekly_Earnings_Primary_Metal_dols"
)


def _write_staging(root: Path) -> None:
    minerals = root / "minerals"
    minerals.mkdir(parents=True)
    (minerals / "mcs2025-lithium_salient.csv").write_text(
        "DataSource,Commodity,Year,USprod_t,NIR_pct\n"
        "MCS2025,Lithium,2023,5000,>50\n"
        "MCS2025,Lithium,2024,5100,>50\n"
    )
    (minerals / "MCS2025_T1_Mineral_Industry_Trends.csv").write_text(
        _T1_HEADER + "\n"
        "MCS2025,2023,36000,60000,20000,38,140,540,300,270,1600,1100,1000,1050\n"
        "MCS2025,2024_estimated,37000,61000,19000,37,141,542,301,271,1650,1120,1020,1070\n"
    )
    (minerals / "MCS2025_T3_State_Value_Rank.csv").write_text(
        "Source,Year,State,State_Notes,Value _millions_prelim_2024,"
        "State_Rank_prelim_2024,State_percent_total_prelim,Principal_commodities\n"
        'MCS2025,2024_estimated,Alabama,,2210,16,2.1,"Cement, lime."\n'
        "MCS2025,2024_estimated,Undistributed,,999,,0.9,\n"
    )
    (minerals / "MCS2025_T4_Critical_Minerals_End_Use.csv").write_text(
        "Source,Critical Mineral,Primary Applications,Category_Note\n"
        "MCS2025,Lithium,Batteries.,\n"
        "MCS2025,Cesium,Research.,\n"  # known-unrepresented: reported, never an error
    )
    (minerals / "MCS2025_Fig3_Major_Import_Sources.csv").write_text(
        "Source,Country,Commodity_Count,Map_Class\n"
        "MCS2024,Australia,6,4 to 6\n"
        "MCS2024,Chile,3,1 to 3\n"
    )


@pytest.fixture()
def staging_dir(tmp_path: Path) -> Path:
    root = tmp_path / "raw_mats"
    _write_staging(root)
    return root


@pytest.fixture()
def db_path(tmp_path: Path) -> Path:
    path = tmp_path / "mini-3nf.sqlite"
    engine = create_engine(f"sqlite:///{path}")
    NormalizedBase.metadata.create_all(engine)
    with Session(engine) as session:
        session.add(DimTime(year=2023, is_annual=True))
        session.add(DimTime(year=2024, is_annual=True))
        session.add(DimState(state_fips="01", state_name="Alabama", state_abbrev="AL"))
        session.commit()
    engine.dispose()
    return path


def _count(db: Path, sql: str) -> int:
    conn = sqlite3.connect(db)
    try:
        return int(conn.execute(sql).fetchone()[0])
    finally:
        conn.close()


def test_dry_run_is_default_and_rolls_back(staging_dir: Path, db_path: Path) -> None:
    exit_code = materials_main.main(["--all", "--db", str(db_path), "--data-dir", str(staging_dir)])
    assert exit_code == 0
    assert _count(db_path, "SELECT COUNT(*) FROM fact_commodity_observation") == 0
    assert _count(db_path, "SELECT COUNT(*) FROM dim_import_source") == 0


def test_execute_fills_all_surfaces(staging_dir: Path, db_path: Path) -> None:
    exit_code = materials_main.main(
        ["--all", "--execute", "--db", str(db_path), "--data-dir", str(staging_dir)]
    )
    assert exit_code == 0
    # Salient EAV: 2 years x 2 metrics (USprod_t + NIR_pct).
    assert _count(db_path, "SELECT COUNT(*) FROM fact_commodity_observation") == 4
    # T1: 2 years x 3 production types; 2 years x 5 sectors.
    assert _count(db_path, "SELECT COUNT(*) FROM fact_mineral_production") == 6
    assert _count(db_path, "SELECT COUNT(*) FROM fact_mineral_employment") == 10
    # T3: Alabama lands; Undistributed skipped.
    assert _count(db_path, "SELECT COUNT(*) FROM fact_state_minerals") == 1
    assert _count(db_path, "SELECT COUNT(*) FROM dim_import_source") == 2
    # T4: Lithium flagged; Cesium reported-unrepresented, not written.
    assert _count(db_path, "SELECT COUNT(*) FROM dim_commodity WHERE is_critical = 1") == 1
    conn = sqlite3.connect(db_path)
    try:
        applications = conn.execute(
            "SELECT primary_applications FROM dim_commodity WHERE is_critical = 1"
        ).fetchone()[0]
    finally:
        conn.close()
    assert applications == "Batteries."


def test_unmatched_t4_mineral_aborts_loudly(staging_dir: Path, db_path: Path) -> None:
    t4 = staging_dir / "minerals" / "MCS2025_T4_Critical_Minerals_End_Use.csv"
    t4.write_text(
        "Source,Critical Mineral,Primary Applications,Category_Note\n"
        "MCS2025,Unobtainium,Plot devices.,\n"
    )
    exit_code = materials_main.main(
        ["--all", "--execute", "--db", str(db_path), "--data-dir", str(staging_dir)]
    )
    assert exit_code == 2
    # Loud abort must leave nothing behind (single rolled-back transaction).
    assert _count(db_path, "SELECT COUNT(*) FROM fact_mineral_production") == 0


def test_missing_data_dir_is_preflight_failure(db_path: Path, tmp_path: Path) -> None:
    exit_code = materials_main.main(
        ["--all", "--db", str(db_path), "--data-dir", str(tmp_path / "nope")]
    )
    assert exit_code == 2
