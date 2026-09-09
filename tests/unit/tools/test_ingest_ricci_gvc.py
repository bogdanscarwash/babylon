"""Unit tests for the Ricci GVC unequal-exchange loader (Program 26 U5b) —
``tools/ingest_ricci_gvc.py``.

No drive access (the CI-no-drive rule): the source CSV is already in-repo
and canonical (``src/babylon/data/reference/babylon_ricci_final.csv``), and
every ``main()`` test runs against a from-scratch, in-memory-schema sqlite
file built via ``NormalizedBase.metadata.create_all`` — never the real
reference DB.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest
from sqlalchemy import create_engine, func, select
from sqlalchemy.orm import Session

_TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(_TOOLS_DIR))

import ingest_ricci_gvc as loader  # type: ignore[import-not-found]  # noqa: E402

from babylon.reference.database import NormalizedBase  # noqa: E402
from babylon.reference.schema import FactRicciUnequalExchangeGvc  # noqa: E402

pytestmark = [pytest.mark.unit]

_CSV_PATH = (
    Path(__file__).resolve().parents[3]
    / "src"
    / "babylon"
    / "data"
    / "reference"
    / "babylon_ricci_final.csv"
)


def _build_scratch_db(tmp_path: Path) -> Path:
    db_path = tmp_path / "scratch.sqlite"
    engine = create_engine(f"sqlite:///{db_path}")
    NormalizedBase.metadata.create_all(engine, tables=[FactRicciUnequalExchangeGvc.__table__])
    engine.dispose()
    return db_path


class TestReadRicciGvcRows:
    def test_row_count(self) -> None:
        rows = loader.read_ricci_gvc_rows(_CSV_PATH)
        assert len(rows) == 51

    def test_missing_file_raises(self, tmp_path: Path) -> None:
        with pytest.raises(loader.RicciGvcIngestError):
            loader.read_ricci_gvc_rows(tmp_path / "does-not-exist.csv")

    def test_pinned_first_row(self) -> None:
        rows = loader.read_ricci_gvc_rows(_CSV_PATH)
        first = rows[0]
        assert first["year"] == 1995
        assert first["region_name"] == "North America"
        assert first["region_type"] == "CORE"
        assert first["flow_direction"] == "INFLOW"
        assert first["transfer_type"] == "GVC"
        assert first["value_usd_billions"] == pytest.approx(31.685)
        assert first["value_pct_gdp"] == pytest.approx(0.4)
        assert first["signed_value"] == pytest.approx(31.685)
        assert first["gvc_share_of_total"] == pytest.approx(0.4479901593450874)
        assert first["source_table"] == "Ricci_Table_6.2"
        assert first["source_priority"] == 1
        assert first["region_granularity"] == 1
        assert first["edge_id"] == "CORE_INFLOW_GVC"

    def test_pinned_row_with_null_fields(self) -> None:
        """Row 45 (1-indexed, matching the ricci_gvc_id it gets on insert):
        the 2007 OECD/GVC row has blank value_pct_gdp/gvc_share_of_total in
        the source CSV — these must decode to None, never 0.0 (a real
        reported zero would be indistinguishable from "not reported")."""
        rows = loader.read_ricci_gvc_rows(_CSV_PATH)
        row = rows[44]
        assert row["year"] == 2007
        assert row["region_name"] == "OECD"
        assert row["region_type"] == "CORE"
        assert row["flow_direction"] == "INFLOW"
        assert row["transfer_type"] == "GVC"
        assert row["value_usd_billions"] == pytest.approx(2800.0)
        assert row["value_pct_gdp"] is None
        assert row["signed_value"] == pytest.approx(2800.0)
        assert row["gvc_share_of_total"] is None
        assert row["source_table"] == "Source_4"
        assert row["source_priority"] == 3
        assert row["region_granularity"] == 2
        assert row["edge_id"] == "CORE_INFLOW_GVC"

    def test_pinned_last_row(self) -> None:
        rows = loader.read_ricci_gvc_rows(_CSV_PATH)
        last = rows[50]
        assert last["year"] == 2009
        assert last["region_name"] == "Non-OECD"
        assert last["region_type"] == "PERIPHERY"
        assert last["flow_direction"] == "OUTFLOW"
        assert last["transfer_type"] == "TOTAL"
        assert last["value_usd_billions"] == pytest.approx(6500.0)
        assert last["value_pct_gdp"] == pytest.approx(37.0)
        assert last["signed_value"] == pytest.approx(-6500.0)
        assert last["gvc_share_of_total"] == pytest.approx(0.0)
        assert last["source_table"] == "Source_4"
        assert last["source_priority"] == 3
        assert last["region_granularity"] == 2
        assert last["edge_id"] == "PERIPHERY_OUTFLOW_TOTAL"

    def test_region_type_enumeration(self) -> None:
        rows = loader.read_ricci_gvc_rows(_CSV_PATH)
        assert {row["region_type"] for row in rows} == {"CORE", "SEMI_PERIPHERY", "PERIPHERY"}

    def test_flow_direction_enumeration(self) -> None:
        rows = loader.read_ricci_gvc_rows(_CSV_PATH)
        assert {row["flow_direction"] for row in rows} == {"INFLOW", "OUTFLOW"}

    def test_transfer_type_enumeration(self) -> None:
        rows = loader.read_ricci_gvc_rows(_CSV_PATH)
        assert {row["transfer_type"] for row in rows} == {"GVC", "TOTAL"}

    def test_determinism_double_run(self) -> None:
        """Re-reading the same CSV twice yields byte-for-byte-identical rows
        — the double-generation determinism proof (ADR076 decision 3's
        discipline, applied to this loader)."""
        first = loader.read_ricci_gvc_rows(_CSV_PATH)
        second = loader.read_ricci_gvc_rows(_CSV_PATH)
        assert first == second


class TestMain:
    def test_main_inserts_all_rows(self, tmp_path: Path) -> None:
        db_path = _build_scratch_db(tmp_path)
        exit_code = loader.main(["--db-url", f"sqlite:///{db_path}", "--csv", str(_CSV_PATH)])
        assert exit_code == 0

        engine = create_engine(f"sqlite:///{db_path}")
        with Session(engine) as session:
            count = session.execute(
                select(func.count()).select_from(FactRicciUnequalExchangeGvc)
            ).scalar_one()
            assert count == 51

            first = session.get(FactRicciUnequalExchangeGvc, 1)
            assert first is not None
            assert first.region_name == "North America"
            assert first.value_usd_billions == pytest.approx(31.685)

            last = session.get(FactRicciUnequalExchangeGvc, 51)
            assert last is not None
            assert last.region_name == "Non-OECD"
            assert last.signed_value == pytest.approx(-6500.0)
        engine.dispose()

    def test_main_refuses_nonempty_target_table(self, tmp_path: Path) -> None:
        db_path = _build_scratch_db(tmp_path)
        engine = create_engine(f"sqlite:///{db_path}")
        with Session(engine) as session:
            session.add(
                FactRicciUnequalExchangeGvc(
                    ricci_gvc_id=1,
                    year=1995,
                    region_name="North America",
                    region_type="CORE",
                    flow_direction="INFLOW",
                    transfer_type="GVC",
                    value_usd_billions=31.685,
                    value_pct_gdp=0.4,
                    signed_value=31.685,
                    gvc_share_of_total=0.4479901593450874,
                    source_table="Ricci_Table_6.2",
                    source_priority=1,
                    region_granularity=1,
                    edge_id="CORE_INFLOW_GVC",
                )
            )
            session.commit()
        engine.dispose()

        exit_code = loader.main(["--db-url", f"sqlite:///{db_path}", "--csv", str(_CSV_PATH)])
        assert exit_code == 1

        check_engine = create_engine(f"sqlite:///{db_path}")
        with Session(check_engine) as session:
            count = session.execute(
                select(func.count()).select_from(FactRicciUnequalExchangeGvc)
            ).scalar_one()
            assert count == 1  # unchanged — the refusal did not touch the pre-existing row
        check_engine.dispose()

    def test_main_missing_csv_raises_loud(self, tmp_path: Path) -> None:
        db_path = _build_scratch_db(tmp_path)
        exit_code = loader.main(
            ["--db-url", f"sqlite:///{db_path}", "--csv", str(tmp_path / "missing.csv")]
        )
        assert exit_code == 1
