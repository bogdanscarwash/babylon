"""One-shot repair of ``view_rent_crisis`` (ADR075 ruling 1 — housing pair fill).

The census (data-catalog.yaml, disposition ``investigate``) found the view
BROKEN: it joined ``fact_census_rent`` × ``fact_census_median_income`` ×
``fact_census_rent_burden`` on ``(county_id, source_id)`` only, while all
three facts carry 14 years × 10 races per county — a cross-time/cross-race
Cartesian explosion (a bare ``COUNT(*)`` did not return in two minutes).
The view predates the multi-year multi-race loads.

The repair joins on the full shared key ``(county_id, source_id, time_id,
race_id)`` and exposes year and race, so downstream consumers (the wealth
axis Phase-2 per-county derived sections, specs/114) can select the slice
they mean instead of inheriting an accidental blowup.

Usage::

    uv run python tools/repair_rent_crisis_view.py            # dry run
    uv run python tools/repair_rent_crisis_view.py --execute
"""

from __future__ import annotations

import argparse
import os
import sqlite3
import sys
from pathlib import Path

REPAIRED_SQL = """CREATE VIEW view_rent_crisis AS
        SELECT
            c.fips,
            c.county_name,
            s.state_name,
            t.year,
            r.race_code,
            fr.median_rent_usd,
            fi.median_income_usd,
            CASE
                WHEN fi.median_income_usd > 0
                THEN (fr.median_rent_usd * 12.0) / fi.median_income_usd
                ELSE NULL
            END AS annual_rent_to_income_ratio,
            SUM(CASE WHEN rb.is_cost_burdened = 1 THEN fb.household_count ELSE 0 END)
                AS cost_burdened_households,
            SUM(CASE WHEN rb.is_severely_burdened = 1 THEN fb.household_count ELSE 0 END)
                AS severely_burdened_households,
            SUM(fb.household_count) AS total_renter_households
        FROM fact_census_rent fr
        JOIN fact_census_median_income fi
            ON fr.county_id = fi.county_id AND fr.source_id = fi.source_id
            AND fr.time_id = fi.time_id AND fr.race_id = fi.race_id
        JOIN fact_census_rent_burden fb
            ON fr.county_id = fb.county_id AND fr.source_id = fb.source_id
            AND fr.time_id = fb.time_id AND fr.race_id = fb.race_id
        JOIN dim_rent_burden rb ON fb.burden_id = rb.burden_id
        JOIN dim_county c ON fr.county_id = c.county_id
        JOIN dim_state s ON c.state_id = s.state_id
        JOIN dim_time t ON fr.time_id = t.time_id
        JOIN dim_race r ON fr.race_id = r.race_id
        GROUP BY c.fips, c.county_name, s.state_name, t.year, r.race_code,
                 fr.median_rent_usd, fi.median_income_usd"""

_DEFAULT_DB = Path("data/sqlite/marxist-data-3NF.sqlite")
_ENV_OVERRIDE = "BABYLON_NORMALIZED_DB_PATH"


class RepairError(Exception):
    """A pre-flight check failed; the database was not modified."""


def needs_repair(conn: sqlite3.Connection) -> bool:
    """Return whether the installed view still lacks the time/race join keys.

    :param conn: Open connection to the reference DB.
    :returns: ``True`` when the broken (county+source-only) definition is
        installed; ``False`` when the repaired SQL is already in place.
    :raises RepairError: When the view is missing entirely.
    """
    row = conn.execute(
        "SELECT sql FROM sqlite_master WHERE type = 'view' AND name = 'view_rent_crisis'"
    ).fetchone()
    if row is None:
        msg = "view_rent_crisis missing from the database — nothing to repair"
        raise RepairError(msg)
    installed = str(row[0])
    # Two defects, both required for health: the full-key join AND real
    # (non-integer) division — INTEGER-affinity rent values truncate
    # (rent * 12) / income to 0 under SQLite integer division.
    return "fr.time_id = fi.time_id" not in installed or "* 12.0" not in installed


def main(argv: list[str] | None = None) -> int:
    """Run the repair (dry run unless ``--execute``).

    :param argv: CLI arguments (defaults to ``sys.argv[1:]``).
    :returns: ``0`` on success.
    :raises RepairError: When a pre-flight check fails; nothing is changed.
    """
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--db",
        type=Path,
        default=Path(os.environ.get(_ENV_OVERRIDE, str(_DEFAULT_DB))),
        help="Path to the reference DB (default honors BABYLON_NORMALIZED_DB_PATH).",
    )
    parser.add_argument(
        "--execute", action="store_true", help="Actually replace the view (default: dry run)."
    )
    args = parser.parse_args(argv)

    if not args.db.exists():
        msg = f"database not found: {args.db}"
        raise RepairError(msg)
    conn = sqlite3.connect(args.db)
    try:
        if not needs_repair(conn):
            print("[rent-crisis] view already carries the full-key join — nothing to do")
            return 0
        if not args.execute:
            print(
                "[rent-crisis] dry run — would DROP + recreate view_rent_crisis "
                "with the (county, source, time, race) join (pass --execute)"
            )
            return 0
        conn.execute("DROP VIEW view_rent_crisis")
        conn.execute(REPAIRED_SQL)
        conn.commit()
        smoke = conn.execute(
            "SELECT COUNT(*) FROM view_rent_crisis WHERE fips = '26163'"
        ).fetchone()[0]
        print(f"[rent-crisis] view repaired; Wayne County (26163) rows: {smoke}")
    finally:
        conn.close()
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except RepairError as error:
        print(f"[rent-crisis] ABORT: {error}", file=sys.stderr)
        sys.exit(2)
