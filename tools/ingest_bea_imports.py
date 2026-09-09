#!/usr/bin/env python3
"""Ingest BEA 'Use of Imports' matrices.

Extracts the 'Use of Imports' matrix from the BEA MAKE-USE-IMPORTS zip archive
(`ImportMatrices_Before_Redefinitions_Summary.xlsx`), parses the coefficients,
and saves them as `FactBEAIOCoefficient` records with `table_type='IMPORT_USE'`.

Sheet layout (identical across all year sheets, verified 1997-2024):

* Rows 0-3 (0-indexed): title, "(Millions of dollars)", "Bureau of Economic
  Analysis", year — no usable data.
* Row 4: blank.
* Row 5: TARGET industry BEA codes in columns 2+ (column 1 is the
  "Commodity / Industry" label).
* Row 6: column 0 == "IOCode" (the anchor we search for — do not hardcode
  the row number in case a future vintage shifts it), column 1 == "Name",
  columns 2+ repeat the industry names.
* Rows 7+: one row per SOURCE (commodity) industry. Column 0 = source BEA
  code, column 1 = name, columns 2+ = dollar value of that commodity used
  by the column's (target) industry, in millions of current dollars.
  Cells are one of: a number, ``"..."`` (BEA disclosure suppression), or
  blank/NaN. There are no stub/total rows (no ``T005``/``T018`` analog) —
  unlike the Supply-Use and IOUse XLSX, this table carries no in-file gross
  output row, so gross output must come from the DB (see below).

Normalization (the whole point of this loader — "honest Φ"):

    import_coeff[i, j, year] = dollar_value[i, j, year] / gross_output_millions[j, year]

``gross_output_millions`` is read from ``fact_bea_national_industry`` (keyed
by the TARGET industry's ``bea_industry_id`` + the year's ``time_id``), which
is exactly how the existing ``USE`` coefficients are normalized (see
``babylon.reference.bea.ingest.io_matrix_parser``, which divides by the
XLSX's own T018 row — the same denominator concept, just sourced from the
DB here since this XLSX has no T018 analog). This parity is what makes
``m_j = sum(IMPORT_USE) / sum(USE)`` (computed by
``babylon.domain.economics.tensor_hierarchy.production_chain_rent.DBImportShareSource``)
a true import-penetration share in [0, 1] rather than a dimensionless mess.

Only years present in BOTH the XLSX and ``fact_bea_national_industry`` can
produce rows (a cell is skipped if gross output for that target/year is
missing or <= 0) — in practice this is 2010-2024, matching USE's coverage.

Usage::

    uv run python tools/ingest_bea_imports.py \\
        --db-url "sqlite:////home/user/projects/game/babylon/data/sqlite/marxist-data-3NF.sqlite"

.. note::
   Post-cutover (parquet-canonical pipeline plan, Task 11), normal usage of
   this loader goes through ``tools/loader_to_sources.py`` instead of calling
   ``main`` here directly: the wrapper runs this module's ``main`` against a
   SCRATCH COPY of the build product, then re-exports the affected tables
   (``fact_bea_io_coefficient``, ``dim_bea_io_table_type``, ``dim_time``) as
   parquet sources and regenerates the manifest — loaders produce sources,
   only the builder produces the DB. This module's ``main(argv)`` signature
   and direct-DB-write behavior are unchanged; they are exactly what the
   wrapper calls against the scratch copy.
"""

from __future__ import annotations

import argparse
import sys
import zipfile
from io import BytesIO

import pandas as pd
from sqlalchemy import create_engine, select
from sqlalchemy.orm import Session

from babylon.reference.database import NormalizedBase
from babylon.reference.schema import (
    DimBEAIndustry,
    DimBEAIOTableType,
    DimTime,
    FactBEAIOCoefficient,
    FactBEANationalIndustry,
)

BEA_ZIP_PATTERN = "/media/user/data/babylon-data/bea/MAKE-USE-IMPORTS*.zip"
XLSX_FILENAME = "ImportMatrices_Before_Redefinitions_Summary.xlsx"
DB_URL = "sqlite:///marxist-data-3NF.sqlite"

_IOCODE_LABEL = "IOCode"
_FIRST_INDUSTRY_COL = 2  # 0-indexed; columns 0-1 are code/name labels


def get_or_create_time(session: Session, year: int) -> DimTime:
    """Get or create DimTime entry."""
    time_entry = session.execute(
        select(DimTime).where(DimTime.year == year, DimTime.is_annual == True)  # noqa: E712
    ).scalar_one_or_none()

    if not time_entry:
        time_entry = DimTime(year=year, is_annual=True)
        session.add(time_entry)
        session.flush()

    return time_entry


def get_or_create_table_type(session: Session, type_name: str) -> DimBEAIOTableType:
    """Get or create DimBEAIOTableType."""
    tt = session.execute(
        select(DimBEAIOTableType).where(DimBEAIOTableType.table_type == type_name)
    ).scalar_one_or_none()

    if not tt:
        tt = DimBEAIOTableType(table_type=type_name)
        session.add(tt)
        session.flush()

    return tt


def get_industry_cache(session: Session) -> dict[str, int]:
    """Load all BEA industries into a code->id map."""
    rows = session.execute(select(DimBEAIndustry)).scalars().all()
    cache = {row.bea_code: row.bea_industry_id for row in rows}

    # If cache is empty, we must be running before the primary BEA loader.
    if not cache:
        print(
            "Warning: DimBEAIndustry is empty. Please run main BEA ingest first.", file=sys.stderr
        )

    return cache


def get_gross_output_cache(session: Session) -> dict[tuple[int, int], float]:
    """Load ``{(bea_industry_id, year): gross_output_millions}`` for annual rows.

    This is the DB-side denominator for normalizing the raw import-matrix
    dollar values into coefficients (mirrors how the USE table is
    normalized against its own gross-output row). Only positive,
    non-null gross output values are cached — a missing/zero entry means
    the corresponding (industry, year) cell must be skipped.
    """
    rows = session.execute(
        select(
            FactBEANationalIndustry.bea_industry_id,
            DimTime.year,
            FactBEANationalIndustry.gross_output_millions,
        )
        .join(DimTime, DimTime.time_id == FactBEANationalIndustry.time_id)
        .where(DimTime.is_annual == True)  # noqa: E712
    ).all()

    cache: dict[tuple[int, int], float] = {}
    for bea_industry_id, year, gross_output_millions in rows:
        if gross_output_millions is None:
            continue
        go = float(gross_output_millions)
        if go > 0.0:
            cache[(bea_industry_id, year)] = go

    return cache


def _find_iocode_row(df: pd.DataFrame) -> int:
    """Return the 0-indexed row number whose column 0 == ``"IOCode"``.

    Raises:
        ValueError: If no such row exists (unexpected sheet layout).
    """
    col0 = df.iloc[:, 0]
    for idx, val in col0.items():
        if isinstance(val, str) and val.strip() == _IOCODE_LABEL:
            return int(idx)
    msg = "Could not find 'IOCode' header row in sheet"
    raise ValueError(msg)


def parse_and_ingest_sheet(
    session: Session,
    df: pd.DataFrame,
    year: int,
    table_type_id: int,
    ind_cache: dict[str, int],
    go_cache: dict[tuple[int, int], float],
) -> int:
    """Parse one year's Import-Use matrix sheet and insert normalized coefficients.

    Args:
        session: SQLAlchemy session (records are ``session.add()``-ed, not committed).
        df: The full sheet read with ``header=None`` (raw, untouched layout).
        year: Calendar year this sheet represents.
        table_type_id: FK to ``dim_bea_io_table_type`` row for ``'IMPORT_USE'``.
        ind_cache: ``bea_code -> bea_industry_id`` map.
        go_cache: ``(bea_industry_id, year) -> gross_output_millions`` map,
            used as the normalization denominator (see module docstring).

    Returns:
        Count of coefficient rows inserted for this sheet.
    """
    time_entry = get_or_create_time(session, year)
    time_id = time_entry.time_id

    iocode_row_idx = _find_iocode_row(df)
    target_code_row_idx = iocode_row_idx - 1
    data_start_idx = iocode_row_idx + 1

    target_code_row = df.iloc[target_code_row_idx]
    col_to_target_id: dict[int, int] = {}
    for col_idx in range(_FIRST_INDUSTRY_COL, df.shape[1]):
        code = target_code_row.iloc[col_idx]
        if not isinstance(code, str):
            continue
        code = code.strip()
        target_id = ind_cache.get(code)
        if target_id is not None:
            col_to_target_id[col_idx] = target_id

    count = 0
    for row_idx in range(data_start_idx, df.shape[0]):
        source_code = df.iat[row_idx, 0]
        if not isinstance(source_code, str):
            continue
        source_code = source_code.strip()
        source_id = ind_cache.get(source_code)
        if source_id is None:
            continue

        for col_idx, target_id in col_to_target_id.items():
            val = df.iat[row_idx, col_idx]

            # BEA suppression marker ("...") or any other non-numeric text.
            if isinstance(val, str):
                continue
            if val is None or pd.isna(val):
                continue

            dollars = float(val)
            if dollars == 0.0:
                continue

            gross_output = go_cache.get((target_id, year))
            if gross_output is None or gross_output <= 0.0:
                continue

            import_coeff = dollars / gross_output

            fact = FactBEAIOCoefficient(
                time_id=time_id,
                table_type_id=table_type_id,
                source_industry_id=source_id,
                target_industry_id=target_id,
                coefficient=import_coeff,
            )
            session.add(fact)
            count += 1

    return count


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Ingest BEA Use of Imports")
    parser.add_argument("--db-url", default=DB_URL, help="Database URL")
    args = parser.parse_args(argv)

    engine = create_engine(args.db_url)
    NormalizedBase.metadata.create_all(engine)

    import glob

    zip_files = glob.glob(BEA_ZIP_PATTERN)
    if not zip_files:
        print(f"No zip files found matching: {BEA_ZIP_PATTERN}", file=sys.stderr)
        return 1

    target_zip = zip_files[0]
    print(f"Reading from {target_zip}...")

    with Session(engine) as session:
        try:
            table_type = get_or_create_table_type(session, "IMPORT_USE")
            ind_cache = get_industry_cache(session)
            if not ind_cache:
                return 1

            go_cache = get_gross_output_cache(session)
            years_with_go = {year for (_bea_id, year) in go_cache}
            if not years_with_go:
                print(
                    "Error: fact_bea_national_industry has no gross-output rows — "
                    "cannot normalize. Run the US1 BEA national ingest first.",
                    file=sys.stderr,
                )
                return 1

            total_inserted = 0
            with zipfile.ZipFile(target_zip, "r") as z:
                if XLSX_FILENAME not in z.namelist():
                    print(f"Error: {XLSX_FILENAME} not found in zip.", file=sys.stderr)
                    return 1

                with z.open(XLSX_FILENAME) as f:
                    file_bytes = BytesIO(f.read())
                    xl = pd.ExcelFile(file_bytes)

                    for sheet_name in xl.sheet_names:
                        if not sheet_name.isdigit():
                            continue
                        year = int(sheet_name)
                        if year not in years_with_go:
                            continue

                        df = pd.read_excel(xl, sheet_name=sheet_name, header=None)
                        n = parse_and_ingest_sheet(
                            session, df, year, table_type.id, ind_cache, go_cache
                        )
                        print(f"  Ingested {n} import use coefficients for {year}")
                        total_inserted += n

            session.commit()
            print(f"Successfully ingested {total_inserted} total coefficients.")

        except Exception as e:
            session.rollback()
            print(f"Error ingesting data: {e}", file=sys.stderr)
            return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
