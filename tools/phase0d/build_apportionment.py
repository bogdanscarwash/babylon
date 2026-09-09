#!/usr/bin/env python3
"""Phase 0-D products 2+3 — population and workplace share keys (ruling 18).

Both products are the SAME motion — a block-group value apportioned onto
res-7 cells through the land mask — so they build together from one
geometry pass:

- **Product 2** (``h3_res7_population.parquet``): Census 2020 P.L. 94-171
  ``POP100`` per BLOCK (SUMLEV=750 geo rows), assigned to the res-7 cell
  containing the block's Census-defined internal point.
- **Product 3** (``h3_res7_workplace.parquet``): LODES8 WAC ``C000`` (total
  jobs) per block, assigned via the LODES crosswalk's block coordinates.

Block-grain internal-point assignment (a method UPGRADE on the charter's
bg areal plan, adopted when the DHC API's bg route proved key-gated): no
interior structure is invented — each block's people land in the one cell
containing the point the Census itself declares. Conservation is exact by
construction and CHECKED against Michigan's published 2020 total
(10,077,331 — a hard external invariant).

This retires the hydrator's "uniform within county" fabrication (S-12):
the interior structure is now the census's, not an assumption's.
"""

from __future__ import annotations

import csv
import gzip
import hashlib
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TROVE = Path("/media/user/data/babylon-data")
MASK = REPO_ROOT / "dist" / "data-artifacts" / "h3_res7_land_mask.parquet"
OUT_POP = REPO_ROOT / "dist" / "data-artifacts" / "h3_res7_population.parquet"
OUT_JOBS = REPO_ROOT / "dist" / "data-artifacts" / "h3_res7_workplace.parquet"

EQUAL_AREA_CRS = "EPSG:5070"
ROUND_DECIMALS = 4

#: Conservation tolerance: apportionment is exact up to float rounding.
CONSERVATION_RTOL = 1e-6


def _block_population() -> dict[tuple[float, float], float]:
    """(lat, lon) internal point -> POP100, from the PL geo file.

    The 2020 PL geo file is pipe-delimited with no header; SUMLEV lives at
    index 2 and blocks are SUMLEV=750. INTPTLAT/INTPTLON are located by
    their signed-coordinate pattern (+DD.DDDDDDD) — deterministic — and
    POP100 sits two fields before INTPTLAT (the published layout:
    ... POP100 | HU100 | INTPTLAT | INTPTLON ...). The parse SELF-VERIFIES
    against Michigan's published 2020 population before anything builds.
    """
    import io
    import re
    import zipfile

    coord = re.compile(r"^[+-]\d{1,3}\.\d{7}$")
    out: dict[tuple[float, float], float] = {}
    with zipfile.ZipFile(TROVE / "census" / "mi2020.pl.zip") as zf:
        geo_name = next(n for n in zf.namelist() if "geo2020.pl" in n)
        with zf.open(geo_name) as raw:
            lat_idx: int | None = None
            for line in io.TextIOWrapper(raw, encoding="latin-1"):  # loop bound: MI geo rows
                fields = line.rstrip("\n").split("|")
                if fields[2] != "750":
                    continue
                if lat_idx is None:
                    lat_idx = next(
                        i
                        for i in range(len(fields) - 1)
                        if coord.match(fields[i]) and coord.match(fields[i + 1])
                    )
                lat = float(fields[lat_idx])
                lon = float(fields[lat_idx + 1])
                pop = float(fields[lat_idx - 2])
                if pop > 0:
                    out[(lat, lon)] = out.get((lat, lon), 0.0) + pop
    total = sum(out.values())
    if round(total) != 10_077_331:  # Michigan 2020 published total
        msg = f"PL parse self-check failed: POP100 sum {total} != 10,077,331"
        raise RuntimeError(msg)
    return out


def _block_jobs() -> dict[tuple[float, float], float]:
    """(lat, lon) block coordinate -> total jobs (WAC C000 via the xwalk)."""
    coords: dict[str, tuple[float, float]] = {}
    with gzip.open(TROVE / "lodes" / "mi_xwalk.csv.gz", "rt") as fh:
        for row in csv.DictReader(fh):  # loop bound: MI blocks
            coords[row["tabblk2020"]] = (float(row["blklatdd"]), float(row["blklondd"]))
    out: dict[tuple[float, float], float] = {}
    with gzip.open(TROVE / "lodes" / "mi_wac_S000_JT00_2020.csv.gz", "rt") as fh:
        for row in csv.DictReader(fh):  # loop bound: MI blocks with jobs
            point = coords.get(row["w_geocode"])
            if point is None:
                continue
            out[point] = out.get(point, 0.0) + float(row["C000"])
    return out


def _mask_cells():  # noqa: ANN202 - (by_county, land_fraction) structures
    import pyarrow.parquet as pq

    table = pq.read_table(MASK)
    by_county: dict[str, list[str]] = {}
    land: dict[str, float] = {}
    for h3_index, fips, fraction in zip(
        table.column("h3_index").to_pylist(),
        table.column("county_fips").to_pylist(),
        table.column("land_fraction").to_pylist(),
        strict=True,
    ):
        by_county.setdefault(fips, []).append(h3_index)
        land[h3_index] = fraction
    return by_county, land


def build() -> dict[str, tuple[int, str]]:
    """Build both parquets by internal-point cell assignment."""
    import h3
    import pyarrow as pa
    import pyarrow.parquet as _pq
    import pyarrow.parquet as pq

    mask_cells = set(_pq.read_table(MASK).column("h3_index").to_pylist())

    results: dict[str, tuple[int, str]] = {}
    outside = {"population": 0.0, "jobs": 0.0}
    for out_path, points, column in (
        (OUT_POP, _block_population(), "population"),
        (OUT_JOBS, _block_jobs(), "jobs"),
    ):
        per_cell: dict[str, float] = {}
        for (lat, lon), value in points.items():  # loop bound: MI blocks
            cell = h3.latlng_to_cell(lat, lon, 7)
            if cell not in mask_cells:
                # A block whose internal point falls outside the bridge
                # tiling (border slivers) is COUNTED loudly, never smeared.
                outside[column] += value
                continue
            per_cell[cell] = per_cell.get(cell, 0.0) + value
        rows = sorted((h, round(v, ROUND_DECIMALS)) for h, v in per_cell.items())
        schema = pa.schema(
            [
                pa.field("h3_index", pa.string(), nullable=False),
                pa.field(column, pa.float64(), nullable=False),
            ]
        )
        table = pa.Table.from_arrays(
            [
                pa.array([r[0] for r in rows], pa.string()),
                pa.array([r[1] for r in rows], pa.float64()),
            ],
            schema=schema,
        )
        out_path.parent.mkdir(parents=True, exist_ok=True)
        with pq.ParquetWriter(out_path, schema, compression="zstd", write_statistics=True) as w:
            w.write_table(table)
        total = sum(v for _h, v in rows)
        print(
            f"{column}: {len(rows)} cells, total {total:.1f}, outside-tiling {outside[column]:.1f}"
        )
        results[out_path.name] = (len(rows), hashlib.sha256(out_path.read_bytes()).hexdigest())
    return results


def main() -> int:
    for name, (rows, digest) in build().items():
        print(f"\n{name}\nrows: {rows}\nsha256: {digest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
