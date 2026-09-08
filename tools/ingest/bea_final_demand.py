#!/usr/bin/env python3
"""Ingest BEA Use Table "Total Final Uses (GDP)" column.

Spec 057 / R2 / T035: reads the BEA Use Table XLSX from the babylon-data
trove and inserts the per-industry "Total Final Uses (GDP)" column for each
year sheet into ``fact_bea_final_demand_annual``. Idempotent — drops +
recreates table on each run.

Usage::

    mise exec -- uv run --frozen python -m tools.ingest.bea_final_demand
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import pandas as pd
from sqlalchemy import create_engine, select
from sqlalchemy.orm import Session

from babylon.reference.schema import (
    DimBEAIndustry,
    DimTime,
    FactBEAFinalDemandAnnual,
    NormalizedBase,
)

DEFAULT_XLSX = Path(
    "/media/user/data/babylon-data/input-output/make-use/IOUse_Before_Redefinitions_PRO_Summary.xlsx"
)
DEFAULT_DB_URL = "sqlite:///data/sqlite/marxist-data-3NF.sqlite"

# Layout constants for the BEA Use Table (Summary level), verified by inspection:
HEADER_ROW_NAMES = 6  # Row index of column header industry names
DATA_START_ROW = 7  # Row index of the first industry data row
INDUSTRY_CODE_COL = 0  # Column index of the IO Code (industry id)
TOTAL_FINAL_USES_COL_NAME = "Total Final Uses (GDP)"


def get_or_create_time(session: Session, year: int) -> DimTime:
    time_entry = session.execute(
        select(DimTime).where(DimTime.year == year, DimTime.is_annual.is_(True))
    ).scalar_one_or_none()
    if not time_entry:
        time_entry = DimTime(year=year, is_annual=True)
        session.add(time_entry)
        session.flush()
    return time_entry


def get_or_create_bea_industry(
    session: Session, bea_code: str, industry_name: str
) -> DimBEAIndustry:
    """Look up DimBEAIndustry by code; create if missing (Summary level)."""
    entry = session.execute(
        select(DimBEAIndustry).where(DimBEAIndustry.bea_code == bea_code)
    ).scalar_one_or_none()
    if entry:
        return entry
    entry = DimBEAIndustry(
        bea_code=bea_code,
        industry_name=industry_name,
        bea_level=3,  # Summary level
    )
    session.add(entry)
    session.flush()
    return entry


def ingest_year_sheet(session: Session, df: pd.DataFrame, year: int) -> int:
    """Ingest a single year's Use Table sheet. Returns rows inserted."""
    header_names = df.iloc[HEADER_ROW_NAMES].tolist()
    try:
        gdp_col_idx = header_names.index(TOTAL_FINAL_USES_COL_NAME)
    except ValueError:
        gdp_col_idx = df.shape[1] - 2  # Fallback: layout invariant

    time = get_or_create_time(session, year)

    inserted = 0
    for row_idx in range(DATA_START_ROW, df.shape[0]):
        bea_code_raw = df.iloc[row_idx, INDUSTRY_CODE_COL]
        if pd.isna(bea_code_raw):
            continue
        bea_code = str(bea_code_raw).strip()
        if not bea_code:
            continue
        # Skip total / aggregate / value-added rows.
        if bea_code.upper().startswith(("T0", "TOTAL", "VABAS", "GROSS", "GDP")):
            continue
        industry_name = str(df.iloc[row_idx, INDUSTRY_CODE_COL + 1]).strip()
        gdp_value = df.iloc[row_idx, gdp_col_idx]
        if pd.isna(gdp_value) or gdp_value == "...":
            continue
        try:
            gdp_float = float(gdp_value)
        except (TypeError, ValueError):
            continue

        industry = get_or_create_bea_industry(session, bea_code, industry_name)
        session.add(
            FactBEAFinalDemandAnnual(
                time_id=time.time_id,
                bea_industry_id=industry.bea_industry_id,
                total_final_uses_millions=gdp_float,
            )
        )
        inserted += 1
    return inserted


def ingest_bea_final_demand(xlsx_path: Path, db_url: str) -> int:
    if not xlsx_path.exists():
        raise FileNotFoundError(f"BEA Use Table XLSX not found at {xlsx_path}")

    engine = create_engine(db_url)
    FactBEAFinalDemandAnnual.__table__.drop(engine, checkfirst=True)
    NormalizedBase.metadata.create_all(engine, tables=[FactBEAFinalDemandAnnual.__table__])

    xl = pd.ExcelFile(xlsx_path)
    year_sheets = [s for s in xl.sheet_names if s.isdigit()]

    total_inserted = 0
    with Session(engine) as session:
        for sheet_name in year_sheets:
            year = int(sheet_name)
            df = pd.read_excel(xl, sheet_name=sheet_name, header=None)
            n = ingest_year_sheet(session, df, year)
            total_inserted += n
            print(f"  {year}: {n} rows", flush=True)
        session.commit()
    return total_inserted


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--xlsx", type=Path, default=DEFAULT_XLSX)
    parser.add_argument("--db", type=str, default=DEFAULT_DB_URL)
    args = parser.parse_args(argv)

    n = ingest_bea_final_demand(args.xlsx, args.db)
    print(f"Total inserted: {n} rows into fact_bea_final_demand_annual")
    return 0


if __name__ == "__main__":
    sys.exit(main())
