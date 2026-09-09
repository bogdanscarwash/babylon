"""Unit tests for the india/latin_america bilateral-trade loader
(Program 26, Unit U3) — ``tools/ingest_census_bilateral_trade_blocs.py``.

No drive access (the CI-no-drive rule: tests never touch
``/media/user/data``) — the xlsx-reader tests build a small synthetic
workbook matching ``country.xlsx``'s real column layout, and the ``main()``
tests run against a from-scratch in-memory-schema sqlite file, never the
real reference DB.
"""

from __future__ import annotations

import sys
from pathlib import Path

import openpyxl
import pytest
from sqlalchemy import create_engine, select
from sqlalchemy.orm import Session

_TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(_TOOLS_DIR))

import ingest_census_bilateral_trade_blocs as loader  # type: ignore[import-not-found]  # noqa: E402

from babylon.reference.database import NormalizedBase  # noqa: E402
from babylon.reference.schema import DimCountry, DimTime, FactBilateralTradeAnnual  # noqa: E402

pytestmark = [pytest.mark.unit]

_HEADER = (
    ["year", "CTY_CODE", "CTYNAME"]
    + [
        f"I{m}"
        for m in (
            "JAN",
            "FEB",
            "MAR",
            "APR",
            "MAY",
            "JUN",
            "JUL",
            "AUG",
            "SEP",
            "OCT",
            "NOV",
            "DEC",
        )
    ]
    + ["IYR"]
    + [
        f"E{m}"
        for m in (
            "JAN",
            "FEB",
            "MAR",
            "APR",
            "MAY",
            "JUN",
            "JUL",
            "AUG",
            "SEP",
            "OCT",
            "NOV",
            "DEC",
        )
    ]
    + ["EYR"]
)


def _write_fake_country_xlsx(path: Path, rows: list[tuple[int, str, str, float, float]]) -> None:
    """``rows``: (year, cty_code, ctyname, imports_annual, exports_annual)."""
    wb = openpyxl.Workbook()
    ws = wb.active
    ws.title = "country"
    ws.append(_HEADER)
    for year, cty_code, ctyname, imports_annual, exports_annual in rows:
        line = (
            [year, cty_code, ctyname]
            + [0.0] * 12
            + [imports_annual]
            + [0.0] * 12
            + [exports_annual]
        )
        ws.append(line)
    wb.save(path)


def test_read_bloc_annual_rows_extracts_target_blocs_in_window(tmp_path: Path) -> None:
    xlsx_path = tmp_path / "country.xlsx"
    _write_fake_country_xlsx(
        xlsx_path,
        [
            (2010, "0009", "South and Central America", 131424.78, 138576.81),
            (2015, "5330", "India", 44782.66, 21452.91),
            (2009, "0009", "South and Central America", 1.0, 1.0),  # outside window
            (2010, "1", "European Union", 999.0, 999.0),  # not a target bloc
        ],
    )
    out = loader.read_bloc_annual_rows(xlsx_path)
    assert out[("0009", 2010)] == pytest.approx((131424.78, 138576.81))
    assert out[("5330", 2015)] == pytest.approx((44782.66, 21452.91))
    assert ("0009", 2009) not in out
    assert ("1", 2010) not in out


def test_read_bloc_annual_rows_missing_file_raises(tmp_path: Path) -> None:
    with pytest.raises(loader.BilateralTradeIngestError):
        loader.read_bloc_annual_rows(tmp_path / "does-not-exist.xlsx")


def test_target_map_is_disjoint_and_carries_the_u5c_partner_set() -> None:
    """P26 U5c (ADR165 Q3/Q4): the target map is the DISJOINT partner set —
    no country_id mapped twice, and the ruled additions are all present."""
    values = list(loader._TARGET_CTY_CODES.values())
    assert len(values) == len(set(values)), "a country_id is mapped under two cty_codes"
    # The ruled headline partners (Mexico->latin_america per Q3; the
    # disjoint individual/aggregate rows per Q4).
    assert loader._TARGET_CTY_CODES["2010"] == 21  # Mexico
    assert loader._TARGET_CTY_CODES["1220"] == 19  # Canada
    assert loader._TARGET_CTY_CODES["5700"] == 168  # China
    assert loader._TARGET_CTY_CODES["0019"] == 15  # Sub Saharan Africa
    assert loader._TARGET_CTY_CODES["4621"] == 96  # Russia
    # 2 original U3 targets + 26 U5c partners (Mexico, Canada, China, SSA,
    # Russia + 11 CIS, 10 ASEAN).
    assert len(loader._TARGET_CTY_CODES) == 28


def test_read_bloc_annual_rows_extracts_a_u5c_partner(tmp_path: Path) -> None:
    """The extended map extracts individual-country rows (Canada) the
    original U3 pair did not cover."""
    xlsx_path = tmp_path / "country.xlsx"
    _write_fake_country_xlsx(
        xlsx_path,
        [
            (2012, "1220", "Canada", 324263.85, 292651.05),
            (2012, "0004", "World, Seasonally Adjusted", 1.0, 1.0),  # not a target
        ],
    )
    out = loader.read_bloc_annual_rows(xlsx_path)
    assert out[("1220", 2012)] == pytest.approx((324263.85, 292651.05))
    assert ("0004", 2012) not in out


def _build_scratch_db(tmp_path: Path) -> Path:
    db_path = tmp_path / "scratch.sqlite"
    engine = create_engine(f"sqlite:///{db_path}")
    NormalizedBase.metadata.create_all(
        engine, tables=[DimCountry.__table__, DimTime.__table__, FactBilateralTradeAnnual.__table__]
    )
    with Session(engine) as session:
        session.add_all(
            [
                DimCountry(
                    country_id=6,
                    cty_code="0009",
                    country_name="South and Central America",
                    is_region=False,
                ),
                DimCountry(country_id=149, cty_code="5330", country_name="India", is_region=False),
                DimTime(time_id=14, year=2010, is_annual=True),
                DimTime(time_id=15, year=2011, is_annual=True),
            ]
        )
        session.commit()
    engine.dispose()
    return db_path


def test_main_inserts_rows_into_scratch_db(tmp_path: Path) -> None:
    db_path = _build_scratch_db(tmp_path)
    xlsx_path = tmp_path / "country.xlsx"
    _write_fake_country_xlsx(
        xlsx_path,
        [
            (2010, "0009", "South and Central America", 131424.78, 138576.81),
            (2011, "5330", "India", 36154.50, 21542.18),
        ],
    )
    # Restrict the loader's year window scan to years actually seeded in dim_time
    # by only providing 2010/2011 rows in the fake xlsx (2012-2024 dim_time rows
    # are absent from this scratch db on purpose -- the loader would otherwise
    # hard-fail on a missing dim_time row for e.g. 2012).
    exit_code = loader.main(
        [
            "--db-url",
            f"sqlite:///{db_path}",
            "--country-xlsx",
            str(xlsx_path),
        ]
    )
    # The loader scans the full 2010-2024 window against dim_time and this
    # scratch db only seeded 2010/2011 -- expect a loud failure, not a silent
    # partial insert.
    assert exit_code == 1


def test_main_inserts_rows_and_is_idempotent(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    db_path = tmp_path / "scratch.sqlite"
    engine = create_engine(f"sqlite:///{db_path}")
    NormalizedBase.metadata.create_all(
        engine, tables=[DimCountry.__table__, DimTime.__table__, FactBilateralTradeAnnual.__table__]
    )
    years = list(range(2010, 2025))
    with Session(engine) as session:
        session.add_all(
            [
                DimCountry(
                    country_id=6,
                    cty_code="0009",
                    country_name="South and Central America",
                    is_region=False,
                ),
                DimCountry(country_id=149, cty_code="5330", country_name="India", is_region=False),
            ]
        )
        session.add_all(
            [DimTime(time_id=14 + i, year=y, is_annual=True) for i, y in enumerate(years)]
        )
        session.commit()
    engine.dispose()

    xlsx_path = tmp_path / "country.xlsx"
    rows = [(y, "0009", "South and Central America", 100.0 + y, 200.0 + y) for y in years]
    rows += [(y, "5330", "India", 10.0 + y, 20.0 + y) for y in years]
    _write_fake_country_xlsx(xlsx_path, rows)

    exit_code = loader.main(["--db-url", f"sqlite:///{db_path}", "--country-xlsx", str(xlsx_path)])
    assert exit_code == 0

    check_engine = create_engine(f"sqlite:///{db_path}")
    with Session(check_engine) as session:
        count = session.execute(select(FactBilateralTradeAnnual)).scalars().all()
        assert len(count) == len(years) * 2  # 15 years x 2 blocs = 30

    # Re-running must not duplicate rows (idempotent skip on existing PK).
    exit_code2 = loader.main(["--db-url", f"sqlite:///{db_path}", "--country-xlsx", str(xlsx_path)])
    assert exit_code2 == 0
    with Session(check_engine) as session:
        count2 = session.execute(select(FactBilateralTradeAnnual)).scalars().all()
        assert len(count2) == len(years) * 2
    check_engine.dispose()
