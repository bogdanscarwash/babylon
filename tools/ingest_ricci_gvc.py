#!/usr/bin/env python3
"""Ingest the Ricci GVC unequal-exchange CSV into the reference DB (Program 26 U5b).

Director ruling (ADR165 D2/D3, ``ai/decisions/
ADR165_p26_director_rulings_trade_slate.yaml``): the checked-in
``src/babylon/data/reference/babylon_ricci_final.csv`` (51 rows; region x
flow-direction x transfer-type, years 1995/2000/2007/2009) becomes a genuine
reference-DB table — ``fact_ricci_unequal_exchange_gvc``
(``babylon.reference.schema.FactRicciUnequalExchangeGvc``) — to ground the
sigma-gradient pipeline's ``world_stats``. This is a DECLARED partial undo of
the 2026-07-17 ADR076 R2 amputation: the CSV was, and remains, the canonical
artifact (``data-artifacts.yaml``'s ``babylon_ricci_final`` register-mode
entry, unchanged by this loader); this table is a build-product MIRROR
populated from it.

**Naming**: deliberately NOT ``fact_ricci_unequal_exchange`` (the amputated
table's retired name) — see ``FactRicciUnequalExchangeGvc``'s docstring for
why (collision risk with the Postgres runtime table
``immutable_reference_ricci_unequal``, which holds UNRELATED Census data
under a legacy label despite the similar name).

Source: the checked-in CSV itself (no ``/media/user/data`` drive dependency
— unlike most ``tools/ingest_*`` loaders, this one's source is already
in-repo and canonical). Row order is preserved verbatim from the CSV file;
``ricci_gvc_id`` (the surrogate PK) is assigned 1..51 in that order — no
combination of the CSV's own columns is a natural key for every row (see the
ORM docstring). Type coercion is explicit: ``year``/``source_priority``/
``region_granularity`` are ``int``; ``value_usd_billions``/``signed_value``
are non-null ``float``; ``value_pct_gdp``/``gvc_share_of_total`` are
nullable ``float`` (an empty CSV field, e.g. the 2007 OECD/Non-OECD ``GVC``
rows, becomes ``None`` — never ``0.0``, which would silently misrepresent
"not reported" as "reported zero").

One-shot, not idempotent (matching the ``ingest_bea_imports``-era loaders'
precedent, ``docs/how-to/reference-data-pipeline.rst`` gotchas): the target
table must be EMPTY when this loader runs — re-running against a non-empty
table aborts loudly rather than silently duplicating or skipping rows (the
surrogate PK has no natural-key grounding to dedupe against).

Usage (build-time only)::

    uv run python tools/ingest_ricci_gvc.py \\
        --db-url "sqlite:////path/to/scratch/marxist-data-3NF.sqlite"

.. note::
   Post-cutover (ADR098), normal usage goes through
   ``tools/loader_to_sources.py --loader ingest_ricci_gvc
   --tables fact_ricci_unequal_exchange_gvc`` instead of calling ``main``
   here directly — the wrapper runs this module's ``main`` against a SCRATCH
   COPY of the build product, re-exports the affected table as a parquet
   source, and regenerates the manifest. Because this is a BRAND NEW table
   (not yet in ``schema.sql``), the scratch copy handed to the wrapper must
   already have ``fact_ricci_unequal_exchange_gvc`` created (DDL added via a
   scratch build + ``tools/extract_reference_schema.py`` re-extraction, per
   ``docs/how-to/reference-data-pipeline.rst``'s "Add a new table" recipe)
   BEFORE the wrapper's own scratch-copy step runs.
"""

from __future__ import annotations

import argparse
import csv
import sys
from pathlib import Path
from typing import TypedDict

from sqlalchemy import create_engine, func, select
from sqlalchemy.orm import Session

from babylon.reference.schema import FactRicciUnequalExchangeGvc

DEFAULT_CSV = Path("src/babylon/data/reference/babylon_ricci_final.csv")
DB_URL = "sqlite:///marxist-data-3NF.sqlite"

#: The CSV's own column order — also the DB column order and the load order
#: (deterministic: file order, never re-sorted).
_EXPECTED_HEADER = (
    "year",
    "region_name",
    "region_type",
    "flow_direction",
    "transfer_type",
    "value_usd_billions",
    "value_pct_gdp",
    "signed_value",
    "gvc_share_of_total",
    "source_table",
    "source_priority",
    "region_granularity",
    "edge_id",
)

#: Valid enumerations — a row outside these is a source-data defect, caught
#: loud at read time rather than surfacing as a CheckConstraint failure deep
#: inside the INSERT (Constitution III.11).
_REGION_TYPES = frozenset({"CORE", "SEMI_PERIPHERY", "PERIPHERY"})
_FLOW_DIRECTIONS = frozenset({"INFLOW", "OUTFLOW"})
_TRANSFER_TYPES = frozenset({"GVC", "TOTAL"})


class RicciGvcRow(TypedDict):
    """One typed, coerced CSV row — the shape :func:`read_ricci_gvc_rows`
    returns and :func:`main` inserts verbatim (plus the surrogate PK)."""

    year: int
    region_name: str
    region_type: str
    flow_direction: str
    transfer_type: str
    value_usd_billions: float
    value_pct_gdp: float | None
    signed_value: float
    gvc_share_of_total: float | None
    source_table: str
    source_priority: int
    region_granularity: int
    edge_id: str


class RicciGvcIngestError(Exception):
    """A loader step failed loudly — malformed source, missing rows, or a
    non-empty target table."""


def _coerce_optional_float(value: str) -> float | None:
    """Empty CSV field -> ``None`` (not reported); otherwise ``float(value)``."""
    return None if value == "" else float(value)


def read_ricci_gvc_rows(csv_path: Path) -> list[RicciGvcRow]:
    """Read every row of ``csv_path`` in file order, explicitly typed.

    :param csv_path: Path to ``babylon_ricci_final.csv`` (or an
        identically-shaped file, e.g. in tests).
    :raises RicciGvcIngestError: If the file is missing, has an unexpected
        header, or a row's ``region_type``/``flow_direction``/
        ``transfer_type`` falls outside the declared enumeration.
    :returns: One :class:`RicciGvcRow` per data row, in file order —
        deterministic: re-reading the same file yields an identical list.
    """
    if not csv_path.is_file():
        msg = f"babylon_ricci_final.csv not found at {csv_path}"
        raise RicciGvcIngestError(msg)

    with csv_path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        if reader.fieldnames is None or tuple(reader.fieldnames) != _EXPECTED_HEADER:
            msg = f"unexpected header in {csv_path}: {reader.fieldnames!r}"
            raise RicciGvcIngestError(msg)

        rows: list[RicciGvcRow] = []
        for line_number, raw in enumerate(reader, start=2):  # header is line 1
            region_type = raw["region_type"]
            flow_direction = raw["flow_direction"]
            transfer_type = raw["transfer_type"]
            if region_type not in _REGION_TYPES:
                msg = f"{csv_path}:{line_number}: unrecognized region_type {region_type!r}"
                raise RicciGvcIngestError(msg)
            if flow_direction not in _FLOW_DIRECTIONS:
                msg = f"{csv_path}:{line_number}: unrecognized flow_direction {flow_direction!r}"
                raise RicciGvcIngestError(msg)
            if transfer_type not in _TRANSFER_TYPES:
                msg = f"{csv_path}:{line_number}: unrecognized transfer_type {transfer_type!r}"
                raise RicciGvcIngestError(msg)
            rows.append(
                RicciGvcRow(
                    year=int(raw["year"]),
                    region_name=raw["region_name"],
                    region_type=region_type,
                    flow_direction=flow_direction,
                    transfer_type=transfer_type,
                    value_usd_billions=float(raw["value_usd_billions"]),
                    value_pct_gdp=_coerce_optional_float(raw["value_pct_gdp"]),
                    signed_value=float(raw["signed_value"]),
                    gvc_share_of_total=_coerce_optional_float(raw["gvc_share_of_total"]),
                    source_table=raw["source_table"],
                    source_priority=int(raw["source_priority"]),
                    region_granularity=int(raw["region_granularity"]),
                    edge_id=raw["edge_id"],
                )
            )
    return rows


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--db-url", default=DB_URL, help="Database URL")
    parser.add_argument("--csv", type=Path, default=DEFAULT_CSV)
    args = parser.parse_args(argv)

    try:
        rows = read_ricci_gvc_rows(args.csv)
    except RicciGvcIngestError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1

    if not rows:
        print(f"Error: no rows found in {args.csv}", file=sys.stderr)
        return 1

    engine = create_engine(args.db_url)
    with Session(engine) as session:
        try:
            existing_count = session.execute(
                select(func.count()).select_from(FactRicciUnequalExchangeGvc)
            ).scalar_one()
            if existing_count:
                msg = (
                    f"fact_ricci_unequal_exchange_gvc already has {existing_count} row(s) — "
                    "this loader is one-shot, not idempotent; refusing to duplicate/skip"
                )
                raise RicciGvcIngestError(msg)

            for ricci_gvc_id, row in enumerate(rows, start=1):
                session.add(FactRicciUnequalExchangeGvc(ricci_gvc_id=ricci_gvc_id, **row))
            session.commit()
            print(f"Inserted {len(rows)} fact_ricci_unequal_exchange_gvc rows.")
        except RicciGvcIngestError as error:
            session.rollback()
            print(f"Error: {error}", file=sys.stderr)
            return 1
        except Exception as error:  # noqa: BLE001 - loader boundary: report and roll back loudly
            session.rollback()
            print(f"Error ingesting Ricci GVC rows: {error}", file=sys.stderr)
            return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
