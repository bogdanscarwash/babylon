#!/usr/bin/env python3
"""Phase 0-D build product 1 — the res-7 land/water mask (ruling 18).

Replaces the fabricated ``bridge_county_h3.coverage_pct`` (uniformly 100
across all 45,572 res-7 cells, verified 2026-07-30) with a REAL per-cell
``land_fraction``: each cell's boundary polygon intersected with its
county's TIGER 2023 AREAWATER union, areas measured in EPSG:5070 (CONUS
Albers equal-area — degree areas would be fabrication by distortion).

Inputs are manifest-pinned (``fetch.py``); the output parquet is
byte-deterministic (fixed schema, rows sorted by ``h3_index``, zstd,
values rounded to 6 decimals) and registers in ``data-artifacts.yaml``
with its sha — the ADR098 shape: built locally, consumed everywhere.

Usage::

    uv run python tools/phase0d/build_mask.py          # build + print sha/rows
"""

from __future__ import annotations

import hashlib
import sqlite3
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SQLITE = REPO_ROOT / "data" / "sqlite" / "marxist-data-3NF.sqlite"
TROVE = Path("/media/user/data/babylon-data")
OUT = REPO_ROOT / "dist" / "data-artifacts" / "h3_res7_land_mask.parquet"

#: Equal-area CRS for all area ratios (CONUS Albers).
EQUAL_AREA_CRS = "EPSG:5070"

#: Deterministic rounding for land_fraction (6 decimals ≈ 0.4 m² at res-7).
ROUND_DECIMALS = 6


def _cells_by_county() -> dict[str, list[str]]:
    """res-7 cells grouped by county FIPS, both levels sorted."""
    conn = sqlite3.connect(SQLITE)
    try:
        rows = conn.execute(
            """
            SELECT c.fips, b.h3_index FROM bridge_county_h3 b
            JOIN dim_county c ON c.county_id = b.county_id
            WHERE b.resolution = 7 ORDER BY c.fips, b.h3_index
            """
        ).fetchall()
    finally:
        conn.close()
    grouped: dict[str, list[str]] = {}
    for fips, h3_index in rows:
        grouped.setdefault(str(fips), []).append(str(h3_index))
    return grouped


def _county_water(fips: str):  # noqa: ANN202 - geopandas GeoSeries
    """The county's AREAWATER union in the equal-area CRS (None = no water)."""
    import geopandas as gpd

    zip_path = TROVE / "tiger" / "areawater" / f"tl_2023_{fips}_areawater.zip"
    if not zip_path.exists():
        msg = f"manifest input missing: {zip_path} — run fetch.py --pin first"
        raise FileNotFoundError(msg)
    frame = gpd.read_file(f"zip://{zip_path}")
    if frame.empty:
        return None
    return frame.to_crs(EQUAL_AREA_CRS).union_all()


def _cell_polygon(h3_index: str):  # noqa: ANN202 - shapely Polygon
    import h3
    from shapely.geometry import Polygon

    boundary = h3.cell_to_boundary(h3_index)  # (lat, lng) pairs
    return Polygon([(lng, lat) for lat, lng in boundary])


def build() -> tuple[int, str]:
    """Build the mask parquet. Returns (rows, sha256)."""
    import geopandas as gpd
    import pyarrow as pa
    import pyarrow.parquet as pq

    grouped = _cells_by_county()
    records: list[tuple[str, str, float]] = []
    for fips in sorted(grouped):  # loop bound: 83 bridge counties
        water = _county_water(fips)
        cells = grouped[fips]
        polys = gpd.GeoSeries([_cell_polygon(h) for h in cells], crs="EPSG:4326").to_crs(
            EQUAL_AREA_CRS
        )
        for h3_index, poly in zip(cells, polys, strict=True):  # loop bound: cells/county
            if water is None:
                fraction = 1.0
            else:
                wet = poly.intersection(water).area
                fraction = max(0.0, min(1.0, 1.0 - (wet / poly.area)))
            records.append((h3_index, fips, round(fraction, ROUND_DECIMALS)))
        print(f"  {fips}: {len(cells)} cells")

    records.sort(key=lambda r: r[0])
    schema = pa.schema(
        [
            pa.field("h3_index", pa.string(), nullable=False),
            pa.field("county_fips", pa.string(), nullable=False),
            pa.field("land_fraction", pa.float64(), nullable=False),
        ]
    )
    OUT.parent.mkdir(parents=True, exist_ok=True)
    table = pa.Table.from_arrays(
        [
            pa.array([r[0] for r in records], pa.string()),
            pa.array([r[1] for r in records], pa.string()),
            pa.array([r[2] for r in records], pa.float64()),
        ],
        schema=schema,
    )
    with pq.ParquetWriter(OUT, schema, compression="zstd", write_statistics=True) as writer:
        writer.write_table(table)

    digest = hashlib.sha256(OUT.read_bytes()).hexdigest()
    return len(records), digest


def main() -> int:
    rows, digest = build()
    print(f"\n{OUT}\nrows: {rows}\nsha256: {digest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
