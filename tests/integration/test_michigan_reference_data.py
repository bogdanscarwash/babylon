"""Integration tests for Michigan statewide reference data coverage (Spec 040).

Validates that the 3NF SQLite reference database contains the data
required for statewide simulation scope expansion:

  - BEA Economic Area dimension and county-to-EA bridge
  - H3 res-7 hex inventory for all 83 Michigan counties
  - County polygon geometries (WKT) for hex generation
  - MSA coverage for the MSA zoom tier

These tests are *read-only* against ``data/sqlite/marxist-data-3NF.sqlite``.
Skipped gracefully when the database file is missing (CI without data).

Markers:
    integration: I/O-bound database tests.
    ledger: Economic/political state validation.
    empirical: Requires the real reference dataset.
"""

from __future__ import annotations

import sqlite3
from pathlib import Path

import pytest

# Needs the reference SQLite DB — excluded on CI until the item-40 subset artifact lands.
pytestmark = pytest.mark.requires_reference_db

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

_DB_CANDIDATES = [
    Path("data/sqlite/marxist-data-3NF.sqlite"),
    Path("data/babylon.db"),
]

MICHIGAN_ABBREV = "MI"
MICHIGAN_STATE_FIPS = "26"
MICHIGAN_COUNTY_COUNT = 83  # Real counties (FIPS 26001–26165)

# BEA Economic Area expectations
BEA_EA_COUNT = 8  # DET, GRR, LAN, KAL, SAG, TVC, MQT, CHI
BEA_EA_CODES = {"DET", "GRR", "LAN", "KAL", "SAG", "TVC", "MQT", "CHI"}

# H3 expectations
H3_RES7_MIN_CELLS = 20_000  # Conservative lower bound (actual ~45k)

# MSA expectations — Michigan has several MSAs
MICHIGAN_MIN_MSAS = 10  # Detroit, GR, Lansing, Kalamazoo, Flint, etc.

# Key counties to spot-check
WAYNE_FIPS = "26163"
OAKLAND_FIPS = "26125"
MACOMB_FIPS = "26099"
MARQUETTE_FIPS = "26103"
KEWEENAW_FIPS = "26083"
BERRIEN_FIPS = "26021"  # Cross-border (Chicago EA)


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


def _find_db() -> Path | None:
    for candidate in _DB_CANDIDATES:
        if candidate.exists():
            return candidate
    return None


@pytest.fixture(scope="module")
def db() -> sqlite3.Connection:
    """Read-only connection to the 3NF reference database."""
    db_path = _find_db()
    if db_path is None:
        pytest.skip(f"Reference database not found; searched {[str(p) for p in _DB_CANDIDATES]}")

    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    conn.row_factory = sqlite3.Row
    yield conn  # type: ignore[misc]
    conn.close()


@pytest.fixture(scope="module")
def michigan_state_id(db: sqlite3.Connection) -> int:
    """Resolve Michigan's surrogate state_id once per module."""
    row = db.execute(
        "SELECT state_id FROM dim_state WHERE state_abbrev = ?",
        (MICHIGAN_ABBREV,),
    ).fetchone()
    assert row is not None, "dim_state missing Michigan row"
    return row["state_id"]


# ===========================================================================
# 2. H3 Res-7 Hex Inventory (Spec 040 — Hex Zoom Tier)
# ===========================================================================


@pytest.mark.integration
@pytest.mark.ledger
@pytest.mark.empirical
class TestH3Res7Coverage:
    """Validate H3 res-7 hex cells exist for all Michigan counties."""

    def test_michigan_has_minimum_h3_res7_cells(
        self, db: sqlite3.Connection, michigan_state_id: int
    ) -> None:
        """Michigan should have at least 20,000 H3 res-7 cells."""
        row = db.execute(
            """
            SELECT count(*) AS n
            FROM bridge_county_h3 bch
            JOIN dim_county c ON bch.county_id = c.county_id
            WHERE c.state_id = ?
              AND bch.resolution = 7
            """,
            (michigan_state_id,),
        ).fetchone()
        assert row is not None
        assert row["n"] >= H3_RES7_MIN_CELLS, (
            f"Expected >= {H3_RES7_MIN_CELLS} res-7 cells, got {row['n']}"
        )

    def test_every_michigan_county_has_h3_cells(
        self, db: sqlite3.Connection, michigan_state_id: int
    ) -> None:
        """Every Michigan county should have at least 1 H3 res-7 cell."""
        rows = db.execute(
            """
            SELECT c.fips, c.county_name, count(bch.h3_index) AS hex_count
            FROM dim_county c
            LEFT JOIN bridge_county_h3 bch
                ON c.county_id = bch.county_id AND bch.resolution = 7
            WHERE c.state_id = ?
              AND c.fips != '26999'
            GROUP BY c.county_id
            HAVING hex_count = 0
            """,
            (michigan_state_id,),
        ).fetchall()
        assert len(rows) == 0, (
            f"Counties with 0 H3 cells: {[(r['fips'], r['county_name']) for r in rows]}"
        )

    def test_h3_index_format_valid(self, db: sqlite3.Connection) -> None:
        """H3 indices should be 15-character hex strings."""
        rows = db.execute(
            """
            SELECT h3_index FROM bridge_county_h3
            WHERE resolution = 7
            LIMIT 100
            """
        ).fetchall()
        assert len(rows) > 0
        for row in rows:
            h3_idx = row["h3_index"]
            assert len(h3_idx) == 15, f"H3 index '{h3_idx}' is not 15 chars"

    def test_keweenaw_has_many_cells(self, db: sqlite3.Connection) -> None:
        """Keweenaw County (Lake Superior peninsula) should have many hex cells."""
        row = db.execute(
            """
            SELECT count(*) AS n
            FROM bridge_county_h3 bch
            JOIN dim_county c ON bch.county_id = c.county_id
            WHERE c.fips = ?
              AND bch.resolution = 7
            """,
            (KEWEENAW_FIPS,),
        ).fetchone()
        assert row is not None
        # Keweenaw is mostly water but has extensive coastline → many hexes
        assert row["n"] >= 500, f"Keweenaw expected >= 500 cells, got {row['n']}"


# ===========================================================================
# 3. County Geometry WKT (Prerequisite for H3 generation)
# ===========================================================================


@pytest.mark.integration
@pytest.mark.ledger
@pytest.mark.empirical
class TestCountyGeometry:
    """Validate county polygon geometries are loaded from TIGER/Line."""

    def test_michigan_counties_have_wkt(
        self, db: sqlite3.Connection, michigan_state_id: int
    ) -> None:
        """All 83 Michigan counties should have geometry_wkt populated."""
        row = db.execute(
            """
            SELECT count(*) AS n
            FROM dim_county_geometry cg
            JOIN dim_county c ON cg.county_id = c.county_id
            WHERE c.state_id = ?
              AND c.fips != '26999'
              AND cg.geometry_wkt IS NOT NULL
              AND LENGTH(cg.geometry_wkt) > 10
            """,
            (michigan_state_id,),
        ).fetchone()
        assert row is not None
        assert row["n"] == MICHIGAN_COUNTY_COUNT, (
            f"Expected {MICHIGAN_COUNTY_COUNT} counties with WKT, got {row['n']}"
        )

    def test_us_wide_geometry_coverage(self, db: sqlite3.Connection) -> None:
        """At least 3,000 US counties should have WKT geometry."""
        row = db.execute(
            """
            SELECT count(*) AS n
            FROM dim_county_geometry
            WHERE geometry_wkt IS NOT NULL
              AND LENGTH(geometry_wkt) > 10
            """
        ).fetchone()
        assert row is not None
        assert row["n"] >= 3000, f"Expected >= 3000 counties with WKT, got {row['n']}"

    def test_wkt_starts_with_polygon_or_multipolygon(self, db: sqlite3.Connection) -> None:
        """WKT geometry should start with POLYGON or MULTIPOLYGON."""
        rows = db.execute(
            """
            SELECT geometry_wkt FROM dim_county_geometry
            WHERE geometry_wkt IS NOT NULL
            LIMIT 50
            """
        ).fetchall()
        assert len(rows) > 0
        for row in rows:
            wkt = row["geometry_wkt"]
            assert wkt.startswith("POLYGON") or wkt.startswith("MULTIPOLYGON"), (
                f"WKT should start with POLYGON/MULTIPOLYGON, got: {wkt[:30]}..."
            )


# ===========================================================================
# 4. MSA Coverage (Spec 040 — MSA Zoom Tier)
# ===========================================================================


@pytest.mark.integration
@pytest.mark.ledger
@pytest.mark.empirical
class TestMSACoverage:
    """Validate MSA data for the MSA zoom tier."""

    def test_michigan_has_sufficient_msas(
        self, db: sqlite3.Connection, michigan_state_id: int
    ) -> None:
        """Michigan should have at least 10 MSAs/Micropolitans."""
        row = db.execute(
            """
            SELECT count(DISTINCT ma.metro_area_id) AS n
            FROM bridge_county_metro bcm
            JOIN dim_county c ON bcm.county_id = c.county_id
            JOIN dim_metro_area ma ON bcm.metro_area_id = ma.metro_area_id
            WHERE c.state_id = ?
            """,
            (michigan_state_id,),
        ).fetchone()
        assert row is not None
        assert row["n"] >= MICHIGAN_MIN_MSAS, (
            f"Expected >= {MICHIGAN_MIN_MSAS} MSAs, got {row['n']}"
        )

    def test_detroit_msa_exists(self, db: sqlite3.Connection) -> None:
        """Detroit MSA should exist and contain Wayne County."""
        row = db.execute(
            """
            SELECT ma.metro_name
            FROM bridge_county_metro bcm
            JOIN dim_county c ON bcm.county_id = c.county_id
            JOIN dim_metro_area ma ON bcm.metro_area_id = ma.metro_area_id
            WHERE c.fips = ?
            """,
            (WAYNE_FIPS,),
        ).fetchone()
        assert row is not None
        assert "Detroit" in row["metro_name"] or "detroit" in row["metro_name"].lower(), (
            f"Wayne County MSA should be Detroit, got: {row['metro_name']}"
        )


# ===========================================================================
# 5. Zoom Hierarchy Completeness
# ===========================================================================


@pytest.mark.integration
@pytest.mark.ledger
@pytest.mark.empirical
class TestZoomHierarchyCompleteness:
    """Validate the 5-tier zoom hierarchy has data at each level."""

    def test_state_tier_exists(self, db: sqlite3.Connection) -> None:
        """State tier: Michigan exists in dim_state."""
        row = db.execute(
            "SELECT count(*) AS n FROM dim_state WHERE state_fips = ?",
            (MICHIGAN_STATE_FIPS,),
        ).fetchone()
        assert row is not None
        assert row["n"] == 1

    def test_bea_tier_exists(self) -> None:
        """BEA tier: at least 1 BEA EA exists — read from the successor CSV.

        ``dim_bea_economic_area`` was amputated from the reference DB by
        ADR076 (36f4cb98) with a committed CSV successor registered in
        ``data-artifacts.yaml``. That commit migrated the whole
        ``TestBEAEconomicAreas`` class but missed this straggler, which kept
        querying the dropped table (nightly Reference-Data red since
        2026-07-17). The zoom-hierarchy claim now reads the successor the
        same way ``tests/unit/reference/test_data_artifacts.py`` does; that
        file additionally pins the full 8-row content.
        """
        import csv

        csv_path = (
            Path(__file__).resolve().parents[2]
            / "src"
            / "babylon"
            / "data"
            / "reference"
            / "dim_bea_economic_area.csv"
        )
        with csv_path.open(encoding="utf-8") as handle:
            rows = list(csv.DictReader(handle))
        assert len(rows) >= 1

    def test_msa_tier_exists(self, db: sqlite3.Connection) -> None:
        """MSA tier: at least 1 metro area exists."""
        row = db.execute("SELECT count(*) AS n FROM dim_metro_area").fetchone()
        assert row is not None
        assert row["n"] >= 1

    def test_county_tier_data(self, db: sqlite3.Connection, michigan_state_id: int) -> None:
        """County tier: 83 Michigan counties exist."""
        row = db.execute(
            "SELECT count(*) AS n FROM dim_county WHERE state_id = ? AND fips != '26999'",
            (michigan_state_id,),
        ).fetchone()
        assert row is not None
        assert row["n"] == MICHIGAN_COUNTY_COUNT

    def test_hex_tier_data(self, db: sqlite3.Connection, michigan_state_id: int) -> None:
        """Hex tier: H3 res-7 cells exist for Michigan."""
        row = db.execute(
            """
            SELECT count(*) AS n FROM bridge_county_h3 bch
            JOIN dim_county c ON bch.county_id = c.county_id
            WHERE c.state_id = ? AND bch.resolution = 7
            """,
            (michigan_state_id,),
        ).fetchone()
        assert row is not None
        assert row["n"] > 0, "No H3 res-7 cells found for Michigan"
