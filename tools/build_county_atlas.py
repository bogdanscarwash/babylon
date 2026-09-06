"""Build the county atlas the Bevy client renders (Program 28 B1, Phase A).

Turns the sha-pinned ``dim_county_geometry`` parquet (TIGER/Line 2024 county
boundaries, EPSG:4269) plus the committed ``county_adjacency.json`` into ONE
content-hashed binary at
``assets/map/county_atlas.bin``.

Why a build-time artifact at all: Amendment AF (i)/(iv) ships the game as a
pure Rust binary, so the deleted Ratatui client's "ask Python for WKT over
FFI" geometry seam no longer exists. Geometry has to arrive as a committed
asset. The repository carries the atlas; CI never builds it, exactly as
``src/babylon/data/game/county_adjacency.json`` already establishes.

Determinism: counties sort by FIPS ascending, rings run exterior-first then
holes in input order, no Python ``set`` is ever iterated, and the tool builds
the whole artifact twice and asserts byte-identity before writing (disable
with ``--skip-determinism-check`` only when iterating locally).

Nothing transcendental crosses the language boundary at runtime: the
cartographic projection runs HERE, at build time, and bakes into u16 grid
units the Rust reader turns back into metres with one multiply and one add.

Usage::

    mise run data:county-atlas
    uv run python tools/build_county_atlas.py --sources <dir> --out <path>

Sources resolve to ``dist/data-artifacts/`` when it exists (rebuild it with
``mise run data:artifacts``), otherwise the pinned drive snapshot at
``/media/user/data/babylon-data/backups/data-artifacts-v7/``. The tool prints
the sha256 of every input it read -- put that in the commit body.

Binary format, version 1. All integers little-endian. The Rust reader checks
every offset and count against the file length BEFORE any loop uses it
(Power-of-10 rule 2: no loop may take its bound from an unchecked number read
out of a file)::

    header (fixed 128 bytes)
      magic        [u8; 8]   b"BABCTY\\0\\x01"
      version      u32       = 1
      flags        u32       = 0
      content_hash [u8; 32]  sha256 of every byte AFTER this field
      origin_x     f64       Albers metres of quantization grid origin
      origin_y     f64
      scale        f64       metres per quantization unit
      county_count u32
      ring_count   u32
      vertex_count u32
      csr_nnz      u32       directed adjacency entries (= 2 x pair count)
      source_hash  [u8; 32]  county_adjacency.json's content_hash, for lineage
      reserved     [u8; 8]   zero-filled to 128

    county table    (county_count x 28 bytes)
      fips        [u8; 5]    ASCII
      pad         [u8; 1]
      ring_start  u32        index into the ring table
      ring_count  u16
      flags       u16        bit 0 = has at least one adjacency neighbour
      bbox        [u16; 4]   min_x, min_y, max_x, max_y in grid units
      centroid    [u16; 2]   grid units
      pad         [u8; 2]    reconciles the field list to the 28-byte stride
    ring table      (ring_count x 12 bytes)
      vertex_start u32,  vertex_count u32,  is_hole u8,  pad [u8; 3]
    vertex array    (vertex_count x 4 bytes)   x u16, y u16
    csr_offsets     ((county_count + 1) x u32)
    csr_neighbors   (csr_nnz x u32)            county indices, ascending per row
    name blob       u32 length, then UTF-8 "<county_name>, <state_abbrev>\\n"
                    per county in order

Rings are stored WITHOUT the WKT closing duplicate vertex, so a ring of ``n``
stored vertices tessellates to ``n - 2`` triangles and the whole atlas to
``vertex_count - 2 * ring_count``. A county's rings run in polygon order:
every ``is_hole == 0`` ring opens a new polygon and the ``is_hole == 1`` rings
that follow belong to it, which is exactly the grouping ``earcut`` needs.

``flags`` bit 0 records "this county has at least one neighbour in
``county_adjacency.json``". The committed adjacency artifact records PAIRS
only, and its county universe is the same ``dim_county_geometry`` universe
this tool reads, so an empty CSR row is a real answer (island counties exist)
rather than a coverage hole. The report lists every empty row by FIPS so the
answer stays visible instead of silent.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import struct
import sys
from array import array
from dataclasses import dataclass, field
from pathlib import Path

import pyarrow.parquet as pq  # type: ignore[import-untyped]
from pyproj import Transformer
from shapely import wkt as shapely_wkt  # type: ignore[import-untyped]
from shapely.geometry.base import BaseGeometry  # type: ignore[import-untyped]

from babylon.domain.geography.adjacency import ARTIFACT_PATH, load_adjacency_pairs

_REPO_ROOT = Path(__file__).resolve().parent.parent

DEFAULT_SOURCES = _REPO_ROOT / "dist" / "data-artifacts"
FALLBACK_SOURCES = Path("/media/user/data/babylon-data/backups/data-artifacts-v7")
DEFAULT_OUT = _REPO_ROOT / "assets" / "map" / "county_atlas.bin"

MAGIC = b"BABCTY\0\x01"
FORMAT_VERSION = 1
HEADER_BYTES = 128
COUNTY_RECORD_BYTES = 28

#: Source CRS of the TIGER/Line WKT: NAD83 geographic.
SOURCE_EPSG = 4269

#: Douglas-Peucker tolerance in DEGREES, applied before projection. 0.001 deg
#: is about 111 m -- the measured knee of the vertex/fidelity curve (363,513
#: vertices against 572,318 at 0.0005 deg, for a tolerance still far below the
#: smallest county's scale). Do not widen it quietly: it is a recorded number
#: that later drift measures against.
SIMPLIFY_TOLERANCE_DEG = 0.001

#: Quantization must never cost more than the simplification it rides on.
MAX_QUANTIZATION_ERROR_M = 111.0

#: Refuse to commit an artifact big enough to want Git LFS.
MAX_ARTIFACT_BYTES = 3 * 1024 * 1024

#: Grid units are u16, so the composite bounding box maps onto [0, 65535].
GRID_MAX = 65535

#: Relative area gap at which a county gets named in the report.
AREA_OUTLIER_FRACTION = 0.02

#: CONUS projection: NAD83 / Conus Albers, equal-area, metres.
CONUS_EPSG = 5070

#: Gap between CONUS and the inset row, and between insets, as a fraction of
#: the CONUS bounding-box width.
INSET_GAP_FRACTION = 0.02


@dataclass(frozen=True)
class InsetPlacement:
    """One non-CONUS region's projection and its slot in the inset row.

    :param name: human name, for the report.
    :param epsg: the region's own projected CRS (metres).
    :param scale: size the region is drawn at, relative to true scale.
    :param slot: left-to-right position in the inset row, 0-based.
    """

    name: str
    epsg: int
    scale: float
    slot: int


#: An inset is a DECLARED LIE ABOUT POSITION that every US map tells: Alaska,
#: Hawaii and Puerto Rico get drawn beside the lower 48 at a chosen scale,
#: nowhere near where they are. These constants carry CARTOGRAPHIC PLACEMENT,
#: not measurement -- no distance, area or adjacency is ever read back out of
#: them. Each region projects through its own CRS, then takes an affine
#: (scale, dx, dy) seating it in a row below and left of CONUS. The realised
#: triples land in the report; dx and dy derive from the CONUS bounding box
#: computed in the same run, so the placement cannot silently drift into the
#: map when the geometry moves, and an overlap fails loudly.
#: Keyed by the 2-digit state FIPS prefix.
INSETS: dict[str, InsetPlacement] = {
    "02": InsetPlacement(name="Alaska", epsg=3338, scale=0.35, slot=0),
    "15": InsetPlacement(name="Hawaii", epsg=2782, scale=1.0, slot=1),
    "72": InsetPlacement(name="Puerto Rico", epsg=32161, scale=1.0, slot=2),
}

FloatRing = tuple[list[tuple[float, float]], bool]
GridRing = tuple[list[tuple[int, int]], bool]


@dataclass(frozen=True)
class CountyRow:
    """One county as read from the reference artifacts."""

    fips: str
    name: str
    state_abbrev: str
    area_sq_km: float
    geometry_wkt: str


@dataclass
class ProjectedCounty:
    """One county through the pipeline: projected, placed, then quantized."""

    fips: str
    name: str
    state_abbrev: str
    area_sq_km: float
    region: str
    rings: list[FloatRing] = field(default_factory=list)
    centroid: tuple[float, float] = (0.0, 0.0)
    grid_rings: list[GridRing] = field(default_factory=list)
    grid_centroid: tuple[int, int] = (0, 0)


@dataclass
class Report:
    """Everything the Task 1 Step 6 report prints, for the commit body."""

    inputs: list[tuple[str, str]] = field(default_factory=list)
    county_count: int = 0
    ring_count: int = 0
    vertex_count: int = 0
    dropped_rings: list[str] = field(default_factory=list)
    byte_size: int = 0
    worst_quantization_error_m: float = 0.0
    affines: list[str] = field(default_factory=list)
    dropped_pairs: list[tuple[str, str]] = field(default_factory=list)
    csr_nnz: int = 0
    isolated: list[str] = field(default_factory=list)
    area_outliers: list[tuple[str, float, float]] = field(default_factory=list)


def check_array_widths() -> None:
    """Refuse to write if this platform's ``array`` widths miss the format.

    ``array("H")`` and ``array("I")`` take their size from the C ``unsigned
    short`` and ``unsigned int``, which the language does not pin. The atlas
    format DOES pin them -- u16 vertices, u32 CSR entries -- so on a platform
    where they disagree the tool must fail loudly rather than emit a binary
    the Rust reader would decode into nonsense (Constitution III.11; the
    byte-level contract discipline in CLAUDE.md).

    :raises ValueError: if either width misses what the format declares.
    """
    for typecode, expected in (("H", 2), ("I", 4)):
        actual = array(typecode).itemsize
        if actual != expected:
            raise ValueError(
                f"array('{typecode}') is {actual} bytes on this platform, but the atlas format "
                f"declares {expected}; refusing to write an undecodable artifact"
            )


def sha256_file(path: Path) -> str:
    """SHA-256 of a file's bytes, for provenance stamping.

    :param path: the file to hash.
    :returns: lowercase hex digest.
    """
    return hashlib.sha256(path.read_bytes()).hexdigest()


def resolve_sources(explicit: Path | None) -> Path:
    """Find the parquet source directory, loudly (Constitution III.11).

    :param explicit: a ``--sources`` override, or ``None``.
    :returns: the directory holding ``dim_county_geometry.parquet``.
    :raises FileNotFoundError: if no candidate directory carries the inputs.
    """
    candidates = [explicit] if explicit is not None else [DEFAULT_SOURCES, FALLBACK_SOURCES]
    for candidate in candidates:
        if candidate is not None and (candidate / "dim_county_geometry.parquet").is_file():
            return candidate
    tried = ", ".join(str(c) for c in candidates if c is not None)
    raise FileNotFoundError(
        f"no county geometry source found; tried {tried} -- rebuild with "
        "'mise run data:artifacts' or mount the babylon-data drive"
    )


def read_counties(sources: Path, report: Report) -> list[CountyRow]:
    """Join geometry, county and state dimensions into FIPS-sorted rows.

    :param sources: directory holding the parquet artifacts.
    :param report: mutated with the input provenance stamps.
    :returns: one row per county carrying geometry, sorted by FIPS ascending.
    :raises ValueError: if a geometry row has no county dimension row, no WKT,
        an unknown state, or a duplicate FIPS (a non-unique key silently
        collapses distinct counties).
    """
    names = ("dim_county_geometry", "dim_county", "dim_state")
    paths = {name: sources / f"{name}.parquet" for name in names}
    for path in paths.values():
        report.inputs.append((str(path), sha256_file(path)))

    geometry = pq.read_table(paths["dim_county_geometry"]).to_pydict()
    county = pq.read_table(paths["dim_county"]).to_pydict()
    state = pq.read_table(paths["dim_state"]).to_pydict()

    abbrev_by_state = dict(zip(state["state_id"], state["state_abbrev"], strict=True))
    dim_by_id = {
        cid: (fips, name, sid)
        for cid, fips, name, sid in zip(
            county["county_id"],
            county["fips"],
            county["county_name"],
            county["state_id"],
            strict=True,
        )
    }

    rows: list[CountyRow] = []
    for cid, area, wkt_text in zip(
        geometry["county_id"], geometry["area_sq_km"], geometry["geometry_wkt"], strict=True
    ):
        if cid not in dim_by_id:
            raise ValueError(f"county_id {cid} has geometry but no dim_county row")
        if not wkt_text:
            raise ValueError(f"county_id {cid} has a null geometry_wkt")
        fips, name, sid = dim_by_id[cid]
        if sid not in abbrev_by_state:
            raise ValueError(f"county {fips} names state_id {sid}, absent from dim_state")
        rows.append(CountyRow(fips, name, abbrev_by_state[sid], float(area), wkt_text))

    rows.sort(key=lambda row: row.fips)
    seen = [row.fips for row in rows]
    if len(dict.fromkeys(seen)) != len(seen):
        raise ValueError("duplicate county FIPS in the geometry source; refusing to build")
    return rows


def _rings_of(geom: BaseGeometry) -> list[FloatRing]:
    """Flatten a (Multi)Polygon into rings, each exterior followed by its holes.

    The WKT closing duplicate vertex is dropped: a stored ring of ``n``
    vertices tessellates to ``n - 2`` triangles.

    :param geom: a shapely Polygon or MultiPolygon.
    :returns: ``(vertices, is_hole)`` pairs in deterministic input order.
    :raises TypeError: on any other geometry type -- the source is county
        boundaries, and anything else means the input drifted.
    """
    if geom.geom_type not in ("Polygon", "MultiPolygon"):
        raise TypeError(f"county geometry is {geom.geom_type}, expected Polygon/MultiPolygon")
    polygons = list(geom.geoms) if geom.geom_type == "MultiPolygon" else [geom]
    rings: list[FloatRing] = []
    for polygon in polygons:
        if polygon.is_empty:
            continue
        rings.append(([(x, y) for x, y in polygon.exterior.coords[:-1]], False))
        for hole in polygon.interiors:
            rings.append(([(x, y) for x, y in hole.coords[:-1]], True))
    return rings


def project_counties(rows: list[CountyRow]) -> list[ProjectedCounty]:
    """Simplify in degrees, then project each county through its region CRS.

    Simplification runs BEFORE projection because the measured tolerance table
    reads in degrees. ``preserve_topology=True`` keeps each county's OWN rings
    valid; it is NOT coverage-aware, so independently simplified neighbours can
    open slivers or overlaps up to the tolerance (~1.6 grid units, ~1-2 px at
    typical zoom — measured, e.g. 01021/01047). If Phase B rendering shows
    visible seams, the coverage-safe operation is ``shapely.coverage_simplify``
    (GEOS CoverageSimplify) — a declared artifact rebuild, not a default.

    :param rows: FIPS-sorted county rows.
    :returns: one entry per county, rings in its region's projected metres.
    :raises ValueError: if simplification empties a county outright, or hands
        back a ring of fewer than three vertices -- shapely cannot produce one
        from a valid polygon, so seeing one means the input drifted.
    """
    transformers = {
        "conus": Transformer.from_crs(SOURCE_EPSG, CONUS_EPSG, always_xy=True),
        **{
            prefix: Transformer.from_crs(SOURCE_EPSG, inset.epsg, always_xy=True)
            for prefix, inset in INSETS.items()
        },
    }
    projected: list[ProjectedCounty] = []
    for row in rows:
        region = row.fips[:2] if row.fips[:2] in INSETS else "conus"
        geom = shapely_wkt.loads(row.geometry_wkt)
        simplified = geom.simplify(SIMPLIFY_TOLERANCE_DEG, preserve_topology=True)
        if simplified.is_empty:
            raise ValueError(f"county {row.fips} simplified to nothing")
        transformer = transformers[region]
        entry = ProjectedCounty(row.fips, row.name, row.state_abbrev, row.area_sq_km, region)
        for vertices, is_hole in _rings_of(simplified):
            if len(vertices) < 3:
                raise ValueError(f"county {row.fips} carries a ring of {len(vertices)} vertices")
            xs, ys = transformer.transform(
                [v[0] for v in vertices], [v[1] for v in vertices], errcheck=True
            )
            entry.rings.append((list(zip(xs, ys, strict=True)), is_hole))
        if not entry.rings:
            raise ValueError(f"county {row.fips} kept no ring after simplification")
        centre = simplified.centroid
        cx, cy = transformer.transform(centre.x, centre.y, errcheck=True)
        entry.centroid = (float(cx), float(cy))
        projected.append(entry)
    return projected


def _bounds_of(counties: list[ProjectedCounty]) -> tuple[float, float, float, float]:
    """Bounding box over every ring vertex of a county subset.

    :param counties: the subset to bound; never empty.
    :returns: ``(min_x, min_y, max_x, max_y)``.
    :raises ValueError: if the subset carries no vertex at all.
    """
    # One pass, four running extremes: materializing xs/ys lists here would
    # hold millions of vertices twice over for no gain.
    min_x = min_y = float("inf")
    max_x = max_y = float("-inf")
    for county in counties:
        for ring, _ in county.rings:
            for x, y in ring:
                min_x = min(min_x, x)
                min_y = min(min_y, y)
                max_x = max(max_x, x)
                max_y = max(max_y, y)
    if min_x > max_x:
        raise ValueError("cannot bound an empty county subset")
    return min_x, min_y, max_x, max_y


def _overlaps(a: tuple[float, float, float, float], b: tuple[float, float, float, float]) -> bool:
    """Whether two axis-aligned boxes share any area.

    :param a: first box as ``(min_x, min_y, max_x, max_y)``.
    :param b: second box, same shape.
    :returns: ``True`` if the boxes intersect.
    """
    return a[0] < b[2] and b[0] < a[2] and a[1] < b[3] and b[1] < a[3]


def place_insets(counties: list[ProjectedCounty], report: Report) -> None:
    """Seat Alaska, Hawaii and Puerto Rico in a row below and left of CONUS.

    Mutates each inset county's coordinates in place. The realised affine
    triples go into the report: a positional lie the code does not say out
    loud is the one that later gets mistaken for a measurement.

    :param counties: every projected county, CONUS and insets alike.
    :param report: mutated with one line per realised affine triple.
    :raises ValueError: if an inset has no counties, or if a placed inset
        overlaps CONUS or another inset -- an overlap would put two different
        places on the same pixels.
    """
    conus_bounds = _bounds_of([c for c in counties if c.region == "conus"])
    gap = (conus_bounds[2] - conus_bounds[0]) * INSET_GAP_FRACTION
    report.affines.append(f"conus EPSG:{CONUS_EPSG} scale=1.0 dx=0.0 dy=0.0 (identity)")

    cursor_x = conus_bounds[0]
    placed: list[tuple[float, float, float, float]] = []
    for prefix, inset in sorted(INSETS.items(), key=lambda item: item[1].slot):
        members = [c for c in counties if c.region == prefix]
        if not members:
            raise ValueError(f"inset {inset.name} ({prefix}) has no counties")
        min_x, min_y, max_x, max_y = _bounds_of(members)
        dx = cursor_x - min_x * inset.scale
        dy = conus_bounds[1] - gap - max_y * inset.scale
        for county in members:
            county.rings = [
                ([(x * inset.scale + dx, y * inset.scale + dy) for x, y in ring], is_hole)
                for ring, is_hole in county.rings
            ]
            county.centroid = (
                county.centroid[0] * inset.scale + dx,
                county.centroid[1] * inset.scale + dy,
            )
        box = (
            min_x * inset.scale + dx,
            min_y * inset.scale + dy,
            max_x * inset.scale + dx,
            max_y * inset.scale + dy,
        )
        if any(_overlaps(box, other) for other in [*placed, conus_bounds]):
            raise ValueError(f"inset {inset.name} overlaps an already-placed region")
        placed.append(box)
        cursor_x = box[2] + gap
        report.affines.append(
            f"{inset.name} EPSG:{inset.epsg} scale={inset.scale} dx={dx:.1f} dy={dy:.1f}"
        )


def _quantize_ring(
    ring: list[tuple[float, float]], origin: tuple[float, float], scale: float
) -> tuple[list[tuple[int, int]], float]:
    """Snap one ring onto the grid, dropping runs of coincident vertices.

    :param ring: the ring's projected metres, without a closing duplicate.
    :param origin: the grid origin as ``(min_x, min_y)``.
    :param scale: metres per grid unit.
    :returns: ``(grid vertices, worst per-axis round-trip error in metres)``.
    """
    grid: list[tuple[int, int]] = []
    worst = 0.0
    for x, y in ring:
        gx = min(GRID_MAX, max(0, round((x - origin[0]) / scale)))
        gy = min(GRID_MAX, max(0, round((y - origin[1]) / scale)))
        worst = max(worst, abs(gx * scale + origin[0] - x), abs(gy * scale + origin[1] - y))
        if not grid or grid[-1] != (gx, gy):
            grid.append((gx, gy))
    while len(grid) > 1 and grid[0] == grid[-1]:
        grid.pop()
    return grid, worst


def quantize(counties: list[ProjectedCounty], report: Report) -> tuple[float, float, float]:
    """Snap every vertex onto the u16 grid and prove the error stays honest.

    A ring that collapses is a sub-pixel exclave or enclave -- at this grid
    the whole shape is smaller than one unit, so it could not have been drawn.
    Dropping it is honest; keeping a degenerate ring would hand ``earcut`` a
    zero-area polygon. When an EXTERIOR collapses its holes go with it: a hole
    with no exterior to belong to would silently re-group under the previous
    polygon and punch a void through a neighbouring shape.

    :param counties: placed counties; ``grid_rings`` is filled in place.
    :param report: mutated with the worst round-trip error and dropped rings.
    :returns: ``(origin_x, origin_y, scale)`` -- the reader's inverse transform.
    :raises ValueError: if quantization costs more than the simplification it
        rides on, or if a county loses every ring to the grid.
    """
    min_x, min_y, max_x, max_y = _bounds_of(counties)
    scale = max(max_x - min_x, max_y - min_y) / GRID_MAX
    worst = 0.0
    for county in counties:
        kept: list[GridRing] = []
        polygon_open = False
        for ring, is_hole in county.rings:
            grid, ring_worst = _quantize_ring(ring, (min_x, min_y), scale)
            worst = max(worst, ring_worst)
            survived = len(grid) >= 3
            if not is_hole:
                polygon_open = survived
                if not survived:
                    report.dropped_rings.append(f"{county.fips}:exterior-collapsed")
            elif not polygon_open:
                report.dropped_rings.append(f"{county.fips}:hole-orphaned")
                continue
            elif not survived:
                report.dropped_rings.append(f"{county.fips}:hole-collapsed")
            if survived:
                kept.append((grid, is_hole))
        if not kept:
            raise ValueError(f"county {county.fips} lost every ring to quantization")
        county.grid_rings = kept
        county.grid_centroid = (
            min(GRID_MAX, max(0, round((county.centroid[0] - min_x) / scale))),
            min(GRID_MAX, max(0, round((county.centroid[1] - min_y) / scale))),
        )
    report.worst_quantization_error_m = worst
    if worst >= MAX_QUANTIZATION_ERROR_M:
        raise ValueError(
            f"quantization error {worst:.1f} m >= the {MAX_QUANTIZATION_ERROR_M} m simplification "
            "tolerance it rides on -- fix the grid, never widen the tolerance"
        )
    return min_x, min_y, scale


def build_csr(counties: list[ProjectedCounty], report: Report) -> tuple[array[int], array[int]]:
    """Turn the committed adjacency pairs into a CSR row per county.

    :param counties: FIPS-sorted counties, which fixes the CSR row order.
    :param report: mutated with dropped pairs and isolated counties.
    :returns: ``(offsets, neighbors)`` with each row ascending.
    """
    index_of = {county.fips: i for i, county in enumerate(counties)}
    rows: list[list[int]] = [[] for _ in counties]
    for fips_a, fips_b in load_adjacency_pairs():
        if fips_a not in index_of or fips_b not in index_of:
            report.dropped_pairs.append((fips_a, fips_b))
            continue
        rows[index_of[fips_a]].append(index_of[fips_b])
        rows[index_of[fips_b]].append(index_of[fips_a])

    offsets = array("I", [0])
    neighbors: array[int] = array("I")
    for i, row in enumerate(rows):
        neighbors.extend(sorted(row))
        offsets.append(len(neighbors))
        if not row:
            report.isolated.append(counties[i].fips)
    report.csr_nnz = len(neighbors)
    return offsets, neighbors


def _grid_area(rings: list[GridRing]) -> float:
    """Shoelace area of a county's quantized rings, holes subtracted.

    :param rings: ``(vertices, is_hole)`` in grid units.
    :returns: the net area in squared grid units.
    """
    total = 0.0
    for ring, is_hole in rings:
        acc = 0.0
        for i, (x, y) in enumerate(ring):
            next_x, next_y = ring[(i + 1) % len(ring)]
            acc += x * next_y - next_x * y
        total += -abs(acc) / 2.0 if is_hole else abs(acc) / 2.0
    return total


def check_areas(counties: list[ProjectedCounty], scale: float, report: Report) -> None:
    """Name counties whose tessellated area disagrees with ``area_sq_km``.

    A simplification sanity check, not a gate: TIGER's ``area_sq_km`` and the
    projected polygon area are two different measurements, so the smallest
    counties naturally show the largest relative gap.

    :param counties: quantized counties.
    :param scale: metres per grid unit.
    :param report: mutated with every county past the outlier fraction.
    """
    for county in counties:
        inset = INSETS.get(county.region)
        drawn_scale = inset.scale if inset is not None else 1.0
        area_km2 = _grid_area(county.grid_rings) * (scale / drawn_scale) ** 2 / 1e6
        if county.area_sq_km <= 0.0:
            continue
        deviation = abs(area_km2 - county.area_sq_km) / county.area_sq_km
        if deviation > AREA_OUTLIER_FRACTION:
            report.area_outliers.append((county.fips, county.area_sq_km, area_km2))


def _pack_tables(
    counties: list[ProjectedCounty], offsets: array[int]
) -> tuple[bytes, bytes, array[int], bytes]:
    """Pack the county, ring, vertex and name tables.

    :param counties: quantized, FIPS-sorted counties.
    :param offsets: the CSR offset array, read for the has-neighbour flag.
    :returns: ``(county table, ring table, vertex array, name blob)``.
    :raises ValueError: on a name carrying the blob's own newline delimiter.
    """
    county_bytes = bytearray()
    ring_bytes = bytearray()
    vertices: array[int] = array("H")
    names = bytearray()
    ring_total = 0
    for index, county in enumerate(counties):
        ring_start = ring_total
        for ring, is_hole in county.grid_rings:
            ring_bytes += struct.pack("<IIB3x", len(vertices) // 2, len(ring), int(is_hole))
            for x, y in ring:
                vertices.append(x)
                vertices.append(y)
            ring_total += 1
        xs = [x for ring, _ in county.grid_rings for x, _ in ring]
        ys = [y for ring, _ in county.grid_rings for _, y in ring]
        county_bytes += struct.pack(
            "<5sxIHH4H2H2x",
            county.fips.encode("ascii"),
            ring_start,
            len(county.grid_rings),
            int(offsets[index + 1] > offsets[index]),
            min(xs),
            min(ys),
            max(xs),
            max(ys),
            county.grid_centroid[0],
            county.grid_centroid[1],
        )
        label = f"{county.name}, {county.state_abbrev}"
        if "\n" in label:
            raise ValueError(f"county {county.fips} name carries the blob delimiter")
        names += label.encode("utf-8") + b"\n"
    return bytes(county_bytes), bytes(ring_bytes), vertices, bytes(names)


def encode(
    counties: list[ProjectedCounty],
    origin: tuple[float, float, float],
    csr: tuple[array[int], array[int]],
    report: Report,
) -> bytes:
    """Serialize the atlas, header last so the content hash can cover it.

    :param counties: quantized, FIPS-sorted counties.
    :param origin: ``(origin_x, origin_y, scale)``.
    :param csr: ``(offsets, neighbors)``.
    :param report: mutated with the realised counts.
    :returns: the complete artifact bytes.
    :raises ValueError: if this platform's ``array`` widths miss the format, or
        if the packed header misses its fixed size.
    """
    check_array_widths()
    offsets, neighbors = csr
    county_bytes, ring_bytes, vertices, names = _pack_tables(counties, offsets)

    if sys.byteorder != "little":
        vertices.byteswap()
        offsets.byteswap()
        neighbors.byteswap()

    report.county_count = len(counties)
    report.ring_count = len(ring_bytes) // 12
    report.vertex_count = len(vertices) // 2

    body = (
        county_bytes
        + ring_bytes
        + vertices.tobytes()
        + offsets.tobytes()
        + neighbors.tobytes()
        + struct.pack("<I", len(names))
        + names
    )
    tail = struct.pack(
        "<dddIIII32s8x",
        origin[0],
        origin[1],
        origin[2],
        report.county_count,
        report.ring_count,
        report.vertex_count,
        report.csr_nnz,
        bytes.fromhex(adjacency_content_hash()),
    )
    content_hash = hashlib.sha256(tail + body).digest()
    header = struct.pack("<8sII32s", MAGIC, FORMAT_VERSION, 0, content_hash) + tail
    if len(header) != HEADER_BYTES:
        raise ValueError(f"header is {len(header)} bytes, expected {HEADER_BYTES}")
    return header + body


def adjacency_content_hash() -> str:
    """The committed adjacency artifact's stamped ``content_hash``.

    ``load_adjacency_pairs`` has already recomputed and verified this stamp,
    so the read cannot hand back a stale one.

    :returns: the lowercase hex digest, for the lineage tripwire.
    """
    load_adjacency_pairs()
    return str(json.loads(ARTIFACT_PATH.read_text())["content_hash"])


def build(sources: Path) -> tuple[bytes, Report]:
    """Run the whole pipeline once.

    :param sources: directory holding the parquet artifacts.
    :returns: ``(artifact bytes, report)``.
    """
    report = Report()
    rows = read_counties(sources, report)
    report.inputs.append((str(ARTIFACT_PATH), sha256_file(ARTIFACT_PATH)))
    counties = project_counties(rows)
    place_insets(counties, report)
    origin_x, origin_y, scale = quantize(counties, report)
    csr = build_csr(counties, report)
    check_areas(counties, scale, report)
    payload = encode(counties, (origin_x, origin_y, scale), csr, report)
    report.byte_size = len(payload)
    return payload, report


def print_report(report: Report) -> None:
    """Print the Task 1 Step 6 report that goes into the commit body.

    :param report: the populated report.
    """
    print("county atlas build report")
    for path, digest in report.inputs:
        print(f"  input       {path}  sha256={digest}")
    for line in report.affines:
        print(f"  affine      {line}")
    print(f"  counties    {report.county_count}")
    print(f"  rings       {report.ring_count}")
    print(f"  vertices    {report.vertex_count}")
    print(f"  triangles   {report.vertex_count - 2 * report.ring_count} (earcut expectation)")
    print(f"  csr_nnz     {report.csr_nnz}")
    print(f"  bytes       {report.byte_size} ({report.byte_size / 1024 / 1024:.2f} MiB)")
    print(f"  worst quantization error  {report.worst_quantization_error_m:.2f} m")
    print(f"  dropped rings   {len(report.dropped_rings)}: {report.dropped_rings[:20]}")
    print(f"  dropped pairs   {len(report.dropped_pairs)}: {report.dropped_pairs[:20]}")
    print(f"  isolated counties  {len(report.isolated)}: {report.isolated}")
    print(f"  area outliers (>{AREA_OUTLIER_FRACTION:.0%})  {len(report.area_outliers)}")
    for fips, declared, drawn in report.area_outliers[:20]:
        print(f"    {fips}  declared={declared:.2f} km2  drawn={drawn:.2f} km2")


def main(argv: list[str] | None = None) -> int:
    """Build the atlas, prove it deterministic, and write it.

    :param argv: command line arguments, or ``None`` for ``sys.argv``.
    :returns: process exit status.
    :raises ValueError: on non-determinism or a past-budget artifact.
    """
    parser = argparse.ArgumentParser(description="Build the county atlas the Bevy client renders")
    parser.add_argument("--sources", type=Path, default=None, help="parquet artifact directory")
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT, help="artifact path to write")
    parser.add_argument(
        "--skip-determinism-check",
        action="store_true",
        help="skip the second full build that proves byte-identity",
    )
    args = parser.parse_args(argv)

    sources = resolve_sources(args.sources)
    print(f"sources: {sources}")
    payload, report = build(sources)

    if not args.skip_determinism_check:
        again, _ = build(sources)
        if again != payload:
            raise ValueError(
                "two builds of the same sources disagree; the tool is not deterministic"
            )
        print("determinism: two full builds byte-identical")

    if report.byte_size > MAX_ARTIFACT_BYTES:
        raise ValueError(
            f"artifact is {report.byte_size} bytes, past the {MAX_ARTIFACT_BYTES}-byte budget -- "
            "STOP and report; never quietly coarsen the tolerance"
        )

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_bytes(payload)
    print_report(report)
    print(f"  wrote       {args.out}")
    print(f"  sha256      {hashlib.sha256(payload).hexdigest()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
