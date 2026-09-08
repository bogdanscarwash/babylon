#!/usr/bin/env python3
"""Ingest Hickel/Sullivan/Zoomkawala (2021) ERDI annual time series.

Spec 057 / R1 / T024d: reads ``babylon_hickel_final.csv`` from the babylon-data
trove and inserts rows into ``fact_hickel_erdi_annual`` in the reference SQLite.
Idempotent — drops + recreates table on each run.

Usage::

    mise exec -- uv run --frozen python -m tools.ingest.hickel_erdi
    # or with custom paths:
    mise exec -- uv run --frozen python -m tools.ingest.hickel_erdi \\
        --csv /path/to/babylon_hickel_final.csv \\
        --db sqlite:////absolute/path/to/marxist-data-3NF.sqlite
"""

from __future__ import annotations

import argparse
import csv
import sys
from pathlib import Path

from sqlalchemy import create_engine, select
from sqlalchemy.orm import Session

from babylon.reference.schema import DimTime, FactHickelERDIAnnual, NormalizedBase

# Shipped in-repo since 2026-07-16 (owner ruling: reference CSVs this small
# travel with the code); the babylon-data trove copy remains the upstream.
DEFAULT_CSV = (
    Path(__file__).resolve().parents[2]
    / "src"
    / "babylon"
    / "data"
    / "reference"
    / "babylon_hickel_final.csv"
)
DEFAULT_DB_URL = "sqlite:///data/sqlite/marxist-data-3NF.sqlite"


def get_or_create_time(session: Session, year: int) -> DimTime:
    time_entry = session.execute(
        select(DimTime).where(DimTime.year == year, DimTime.is_annual.is_(True))
    ).scalar_one_or_none()
    if not time_entry:
        time_entry = DimTime(year=year, is_annual=True)
        session.add(time_entry)
        session.flush()
    return time_entry


def _parse_bool(s: str) -> bool:
    return s.strip().lower() in ("true", "1", "yes", "t", "y")


def _parse_float_or_none(s: str) -> float | None:
    s = s.strip()
    if not s:
        return None
    return float(s)


def ingest_hickel_erdi(csv_path: Path, db_url: str) -> int:
    """Ingest the Hickel ERDI CSV into ``fact_hickel_erdi_annual``.

    Returns:
        Number of rows inserted.
    """
    if not csv_path.exists():
        msg = f"Hickel CSV not found at {csv_path}"
        raise FileNotFoundError(msg)

    engine = create_engine(db_url)

    # Drop + recreate the target table for idempotency.
    FactHickelERDIAnnual.__table__.drop(engine, checkfirst=True)
    NormalizedBase.metadata.create_all(engine, tables=[FactHickelERDIAnnual.__table__])

    inserted = 0
    with Session(engine) as session, csv_path.open() as fh:
        reader = csv.DictReader(fh)
        for row in reader:
            year = int(row["year"])
            scale_type = row["scale_type"].strip()
            time = get_or_create_time(session, year)
            session.add(
                FactHickelERDIAnnual(
                    time_id=time.time_id,
                    scale_type=scale_type,
                    erdi=float(row["erdi"]),
                    annual_drain_usd_billions=float(row["annual_drain_usd_billions"]),
                    alpha=_parse_float_or_none(row["alpha"]),
                    core_gain_per_capita_usd=_parse_float_or_none(row["core_gain_per_capita_usd"]),
                    is_anchor_year=_parse_bool(row["is_anchor_year"]),
                    china_inflection=_parse_bool(row["china_inflection"]),
                    cumulative_drain=_parse_float_or_none(row["cumulative_drain"]),
                    source=row["source"].strip(),
                )
            )
            inserted += 1
        session.commit()

    return inserted


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--csv", type=Path, default=DEFAULT_CSV)
    parser.add_argument("--db", type=str, default=DEFAULT_DB_URL)
    args = parser.parse_args(argv)

    n = ingest_hickel_erdi(args.csv, args.db)
    print(f"Inserted {n} rows into fact_hickel_erdi_annual from {args.csv}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
