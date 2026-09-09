"""Populate county geometry WKT from TIGER/Line shapefile & generate H3 res-7 cells.

Two-phase script:
  Phase 1: Load WKT polygon boundaries from TIGER/Line 2024 into dim_county_geometry
           for ALL US counties (~3,235 rows).
  Phase 2: Generate H3 res-7 cells for Michigan counties (~28,000 cells) and insert
           into bridge_county_h3.

Data Source:
    /media/user/data/babylon-data/tiger/county/tl_2024_us_county.shp
    TIGER/Line 2024 US county boundaries (Census Bureau)
    CRS: EPSG:4269 (NAD83) — compatible with H3 (WGS84 ~= NAD83 for CONUS)

Usage::

    uv run python tools/ingest_tiger_geometry.py [--h3-state 26]
    # or, via mise:
    mise run data:tiger-sqlite [-- --h3-state 26]

Flags:
    --h3-state FIPS   Generate H3 res-7 cells for this state (default: 26 = Michigan)
    --h3-all          Generate H3 res-7 cells for ALL states (slow, ~3M cells)
    --skip-wkt        Skip WKT loading, only do H3 expansion
"""

from __future__ import annotations

import argparse
import logging
import sys
import time
from decimal import Decimal
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(_REPO_ROOT / "src"))

from sqlalchemy import func, update  # noqa: E402

from babylon.config.logging_config import setup_logging  # noqa: E402
from babylon.reference.database import get_normalized_session, init_normalized_db  # noqa: E402
from babylon.reference.schema import (  # noqa: E402
    BridgeCountyH3,
    DimCounty,
    DimCountyGeometry,
    DimState,
)

setup_logging(default_level="INFO")
logger = logging.getLogger(__name__)

TIGER_SHP = Path("/media/user/data/babylon-data/tiger/county/tl_2024_us_county.shp")
H3_RESOLUTION = 7


def load_wkt_from_tiger() -> None:
    """Phase 1: Load WKT geometry from TIGER/Line shapefile into dim_county_geometry."""
    import geopandas as gpd  # type: ignore[import-untyped]

    if not TIGER_SHP.exists():
        logger.error("TIGER shapefile not found: %s", TIGER_SHP)
        raise SystemExit(1)

    logger.info("Reading TIGER/Line shapefile: %s", TIGER_SHP)
    gdf = gpd.read_file(TIGER_SHP)
    logger.info("  Loaded %d county geometries", len(gdf))

    # Build FIPS → WKT mapping
    fips_to_wkt: dict[str, str] = {}
    for _, row in gdf.iterrows():
        geoid = row["GEOID"]  # 5-digit FIPS
        geom = row["geometry"]
        if geom is not None and not geom.is_empty:
            fips_to_wkt[geoid] = geom.wkt
    logger.info("  Valid geometries: %d", len(fips_to_wkt))

    with get_normalized_session() as session:
        # Get all counties with their current geometry status
        counties = session.query(DimCounty.county_id, DimCounty.fips).all()
        fips_to_county_id = {fips: cid for cid, fips in counties}

        updated = 0
        missing = 0
        for fips, wkt in fips_to_wkt.items():
            county_id = fips_to_county_id.get(fips)
            if county_id is None:
                missing += 1
                continue

            # Check if geometry row exists
            geom_row = (
                session.query(DimCountyGeometry)
                .filter(DimCountyGeometry.county_id == county_id)
                .first()
            )
            if geom_row is None:
                # No geometry row exists — skip (should have been created by prior ingest)
                missing += 1
                continue

            # Update WKT
            session.execute(
                update(DimCountyGeometry)
                .where(DimCountyGeometry.county_id == county_id)
                .values(geometry_wkt=wkt)
            )
            updated += 1

        logger.info("  Updated %d county geometries, %d unmatched", updated, missing)


def generate_h3_cells(state_fips: str | None = "26") -> None:
    """Phase 2: Generate H3 res-7 cells for counties in the given state."""
    from babylon.data.h3_utils import generate_h3_cells as h3_gen
    from babylon.data.h3_utils import wkt_to_geometry

    with get_normalized_session() as session:
        # Build query for counties with geometry
        query = (
            session.query(
                DimCounty.county_id,
                DimCounty.fips,
                DimCounty.county_name,
                DimCountyGeometry.geometry_wkt,
            )
            .join(DimCountyGeometry, DimCounty.county_id == DimCountyGeometry.county_id)
            .filter(DimCountyGeometry.geometry_wkt.isnot(None))
        )

        if state_fips:
            mi_state = session.query(DimState).filter(DimState.state_fips == state_fips).first()
            if not mi_state:
                logger.error("State FIPS %s not found", state_fips)
                raise SystemExit(1)
            state_id = mi_state.state_id
            query = query.filter(DimCounty.state_id == state_id)
            # Exclude state-level aggregate rows
            query = query.filter(DimCounty.fips != f"{state_fips}999")
            label = f"state {state_fips}"
        else:
            state_id = None
            label = "ALL states"

        county_geoms = query.all()
        logger.info(
            "Generating H3 res-%d cells for %d counties (%s)",
            H3_RESOLUTION,
            len(county_geoms),
            label,
        )

        # Check existing res-7 count
        existing_count = session.query(func.count(BridgeCountyH3.h3_index)).filter(
            BridgeCountyH3.resolution == H3_RESOLUTION
        )
        if state_id is not None:
            existing_count = existing_count.join(
                DimCounty, BridgeCountyH3.county_id == DimCounty.county_id
            ).filter(DimCounty.state_id == state_id)
        existing = existing_count.scalar() or 0

        if existing > 1000:
            logger.info("  Already have %d res-%d cells. Skipping.", existing, H3_RESOLUTION)
            return

        total_cells = 0
        t0 = time.perf_counter()

        for county_id, fips, name, wkt in county_geoms:
            geom = wkt_to_geometry(wkt)
            if geom is None:
                logger.warning("  %s (%s): invalid geometry, skipping", name, fips)
                continue

            cells = h3_gen(geom, H3_RESOLUTION)
            if not cells:
                logger.warning("  %s (%s): generated 0 cells", name, fips)
                continue

            for h3_index in cells:
                bridge = BridgeCountyH3(
                    h3_index=h3_index,
                    county_id=county_id,
                    resolution=H3_RESOLUTION,
                    coverage_pct=Decimal("100.00"),
                )
                session.merge(bridge)  # merge handles duplicates for border cells

            total_cells += len(cells)

            # Flush periodically
            if total_cells % 5000 == 0:
                session.flush()
                elapsed = time.perf_counter() - t0
                logger.info("  Progress: %d cells (%.1fs)", total_cells, elapsed)

        session.flush()
        elapsed = time.perf_counter() - t0
        logger.info("Generated %d H3 res-%d cells in %.1fs", total_cells, H3_RESOLUTION, elapsed)


def main() -> None:
    """Run geometry loading and H3 expansion."""
    parser = argparse.ArgumentParser(description="Load county geometry WKT and generate H3 cells")
    parser.add_argument(
        "--h3-state", default="26", help="State FIPS for H3 generation (default: 26=Michigan)"
    )
    parser.add_argument("--h3-all", action="store_true", help="Generate H3 for ALL states")
    parser.add_argument("--skip-wkt", action="store_true", help="Skip WKT loading phase")
    args = parser.parse_args()

    init_normalized_db()

    if not args.skip_wkt:
        logger.info("=" * 60)
        logger.info("Phase 1: Loading WKT from TIGER/Line shapefile")
        logger.info("=" * 60)
        load_wkt_from_tiger()

    logger.info("=" * 60)
    logger.info("Phase 2: Generating H3 res-%d cells", H3_RESOLUTION)
    logger.info("=" * 60)
    state_fips = None if args.h3_all else args.h3_state
    generate_h3_cells(state_fips)

    logger.info("Done.")


if __name__ == "__main__":
    main()
