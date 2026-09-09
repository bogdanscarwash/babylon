#!/usr/bin/env python3
"""Build PER-277's deterministic Michigan county/place/H3 land overlaps.

All raw archives remain local acquisition evidence. The builder verifies every
input digest before decode, treats the PER-275 land-mask artifact as an H3
identity cohort only, subtracts governed TIGER 2023 AREAWATER geometry, and
emits only positive whole-square-metre land intersections.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import importlib.metadata
import io
import json
import math
import re
import sys
import zipfile
from collections import Counter, defaultdict
from collections.abc import Iterable, Mapping, Sequence
from dataclasses import asdict, dataclass
from numbers import Integral
from pathlib import Path, PurePosixPath
from typing import Any, Final

REPO_ROOT = Path(__file__).resolve().parents[1]
TROVE = Path("/media/user/data/babylon-data")
FETCH_MANIFEST = REPO_ROOT / "tools" / "county_place_h3_overlap_v1_fetch_manifest.json"
PHASE0D_MANIFEST = REPO_ROOT / "tools" / "phase0d" / "fetch_manifest.json"
H3_CONTRACT = REPO_ROOT / "contracts" / "h3_estate_contract_v1.yaml"
H3_VECTORS = (
    REPO_ROOT
    / "rust"
    / "crates"
    / "babylon-kernel"
    / "tests"
    / "fixtures"
    / "h3_cell_id_vectors_v1.txt"
)
PLACE_CONTRACT = REPO_ROOT / "contracts" / "census_place_authority_v1.yaml"
PLACE_MANIFEST = REPO_ROOT / "tools" / "census_place_authority_v1_fetch_manifest.json"
PLACE_IDENTITY = (
    REPO_ROOT
    / "src"
    / "babylon"
    / "data"
    / "reference"
    / "spatial"
    / "census_place_identity_mi_2023.csv.gz"
)
PLACE_GEOMETRY = PLACE_IDENTITY.with_name("census_place_geometry_mi_2023.parquet")
COUNTY_OUT = PLACE_IDENTITY.with_name("census_county_h3_land_overlap_mi_2023.parquet")
PLACE_OUT = PLACE_IDENTITY.with_name("census_county_place_h3_land_overlap_mi_2023.parquet")
COUNTY_IDENTITY_OUT = REPO_ROOT / "contracts" / "fixtures" / "census_county_geoids_mi_2023.txt"
H3_IDENTITY_OUT = REPO_ROOT / "contracts" / "fixtures" / "h3_res7_land_mask_ids_v1.txt.gz"

SOURCE_DEST: Final = "tiger/county/tl_2023_us_county.zip"
SOURCE_URL: Final = "https://www2.census.gov/geo/tiger/TIGER2023/COUNTY/tl_2023_us_county.zip"
H3_SOURCE = TROVE / "github" / "ci-data-v8" / "h3_res7_land_mask.parquet"
H3_SOURCE_URL: Final = (
    "https://github.com/percy-raskova/babylon/releases/download/"
    "ci-data-v8/h3_res7_land_mask.parquet"
)

ARTIFACT_NAMES: Final = (
    "census_county_h3_land_overlap_mi_2023",
    "census_county_place_h3_land_overlap_mi_2023",
)
COUNTY_COLUMNS: Final = ("cell_id", "county_fips", "land_area_m2")
PLACE_COLUMNS: Final = (
    "cell_id",
    "county_fips",
    "place_geoid",
    "place_land_area_m2",
    "cell_mi_land_area_m2",
    "place_land_area_share_ppb",
)
COUNTY_SOURCE_COLUMNS: Final = (
    "STATEFP",
    "COUNTYFP",
    "COUNTYNS",
    "GEOID",
    "GEOIDFQ",
    "NAME",
    "NAMELSAD",
    "LSAD",
    "CLASSFP",
    "MTFCC",
    "CSAFP",
    "CBSAFP",
    "METDIVFP",
    "FUNCSTAT",
    "ALAND",
    "AWATER",
    "INTPTLAT",
    "INTPTLON",
    "geometry",
)
AREAWATER_SOURCE_COLUMNS: Final = (
    "ANSICODE",
    "HYDROID",
    "FULLNAME",
    "MTFCC",
    "ALAND",
    "AWATER",
    "INTPTLAT",
    "INTPTLON",
    "geometry",
)

STATE_FIPS: Final = "26"
WORKING_CRS: Final = "EPSG:5070"
SOURCE_CRS_EPSG: Final = 4269
SHARE_SCALE: Final = 1_000_000_000
MAX_FETCH_MANIFEST_BYTES: Final = 131_072
MAX_SOURCE_ARCHIVE_BYTES: Final = 104_857_600
MAX_ZIP_UNCOMPRESSED_BYTES: Final = 268_435_456
MAX_ZIP_MEMBERS: Final = 8
MAX_COUNTY_SOURCE_ROWS: Final = 4_096
MAX_COUNTIES: Final = 128
MAX_COHORT_ROWS: Final = 65_536
MAX_PLACE_ROWS: Final = 2_048
MAX_COUNTY_ROWS: Final = 65_536
MAX_OVERLAP_ROWS: Final = 65_536

PHASE0D_MANIFEST_SHA256: Final = "49261772713642f7848d8ce98a0240f6aef5c2c8bd7340367585b2976b6bdb22"
H3_CONTRACT_SHA256: Final = "1ea546e572c499424086d52bc62820626328c8336c38d57b6053c337c0b943b5"
H3_VECTORS_SHA256: Final = "079fd93db1cfb9b2aef79b7b32476f737689a91c194de912864bb3849e578487"
H3_SOURCE_SHA256: Final = "4e6caba297f0111a9ec93d948a83543bb9f7179361fe5dd318bb8a98a5be5194"
H3_SOURCE_BYTES: Final = 295_194
H3_SOURCE_ROWS: Final = 45_572
PLACE_CONTRACT_SHA256: Final = "0bced499b9144e51d48bc2356260448bc09ab56b64035ab94c98a7287b462102"
PLACE_MANIFEST_SHA256: Final = "223c1419dd9e7bd855efd8fb07b87199ce4ec88bd549e369890edbb1dea71456"
PLACE_IDENTITY_SHA256: Final = "cb864b4f6f43902bb821e84fe9a4055a9039e0a74d8b8399f209ae6ed26a8be7"
PLACE_GEOMETRY_SHA256: Final = "cea5b0ada40b75ae2f6996bef7aa4aeb8d13b36ce5bc41d4334da1e8bf17b737"

EXPECTED_PACKAGE_VERSIONS: Final = {
    "geopandas": "1.1.4",
    "h3": "4.5.0",
    "pyarrow": "25.0.0",
    "pyogrio": "0.13.0",
    "pyproj": "3.7.2",
    "shapely": "2.1.2",
}
EXPECTED_GEOS_VERSION: Final = "3.13.1"
EXPECTED_PROJ_VERSION: Final = "9.5.1"
SHA256 = re.compile(r"^[0-9a-f]{64}$")
COUNTY_FIPS = re.compile(r"^26[0-9]{3}$")
PLACE_GEOID = re.compile(r"^26[0-9]{5}$")


class OverlapBuildError(ValueError):
    """One typed refusal from the PER-277 builder."""

    def __init__(self, code: str, detail: str) -> None:
        self.code = code
        self.detail = detail
        super().__init__(f"{code}: {detail}")


@dataclass(frozen=True, order=True)
class CountyLandRow:
    """One positive canonical cell-by-county land intersection."""

    cell_id: int
    county_fips: str
    land_area_m2: int


@dataclass(frozen=True, order=True)
class PlaceLandRow:
    """One positive canonical cell-by-county-by-place land intersection."""

    cell_id: int
    county_fips: str
    place_geoid: str
    place_land_area_m2: int
    cell_mi_land_area_m2: int
    place_land_area_share_ppb: int


@dataclass(frozen=True)
class ArtifactStats:
    """Deterministic artifact evidence."""

    rows: int
    bytes: int
    sha256: str
    semantic_sha256: str


@dataclass(frozen=True)
class CountySource:
    """Validated Michigan rows from the national county archive."""

    all_rows: int
    geometries: dict[str, Any]
    extent: tuple[float, float, float, float]


@dataclass(frozen=True)
class BuildResult:
    """Final artifacts and bounded cohort evidence."""

    county_identity_fixture: ArtifactStats
    h3_identity_fixture: ArtifactStats
    county_artifact: ArtifactStats
    place_artifact: ArtifactStats
    county_source_rows: int
    county_rows: int
    place_rows: int
    positive_land_cells: int
    cohort_absent_cells: int
    cross_county_cells: int
    places_crossing_counties: int
    land_cells_without_place: int
    total_mi_land_area_m2: int
    total_place_land_area_m2: int


def _unique_json_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise OverlapBuildError("json_duplicate_key", key)
        result[key] = value
    return result


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    try:
        with path.open("rb") as handle:
            for chunk in iter(lambda: handle.read(1 << 20), b""):
                digest.update(chunk)
    except OSError as error:
        raise OverlapBuildError("file_read", str(path)) from error
    return digest.hexdigest()


def _verify_file(path: Path, expected_sha256: str, *, expected_bytes: int | None = None) -> None:
    try:
        size = path.stat().st_size
    except OSError as error:
        raise OverlapBuildError("file_read", str(path)) from error
    if expected_bytes is not None and size != expected_bytes:
        raise OverlapBuildError("file_size", f"{path}: {size} != {expected_bytes}")
    actual = _sha256(path)
    if actual != expected_sha256:
        raise OverlapBuildError("file_sha256", f"{path}: {actual} != {expected_sha256}")


def load_fetch_manifest(path: Path = FETCH_MANIFEST) -> list[dict[str, str]]:
    """Load the dedicated one-source manifest with duplicate-key refusal."""
    try:
        raw = path.read_bytes()
    except OSError as error:
        raise OverlapBuildError("fetch_manifest_read", str(path)) from error
    if len(raw) > MAX_FETCH_MANIFEST_BYTES:
        raise OverlapBuildError("fetch_manifest_size", str(len(raw)))
    try:
        document = json.loads(raw, object_pairs_hook=_unique_json_object)
    except (json.JSONDecodeError, UnicodeDecodeError) as error:
        raise OverlapBuildError("fetch_manifest_json", str(path)) from error
    expected_root = {
        "contract": "CountyPlaceH3OverlapV1",
        "version": 1,
        "entries": document.get("entries") if isinstance(document, dict) else None,
    }
    if not isinstance(document, dict) or document != expected_root:
        raise OverlapBuildError("fetch_manifest_shape", "root")
    entries = document.get("entries")
    if not isinstance(entries, list) or len(entries) != 1:
        raise OverlapBuildError("fetch_manifest_shape", "entries")
    entry = entries[0]
    if not isinstance(entry, dict) or set(entry) != {"url", "dest", "sha256"}:
        raise OverlapBuildError("fetch_manifest_entry", repr(entry))
    if not all(isinstance(entry[key], str) for key in ("url", "dest", "sha256")):
        raise OverlapBuildError("fetch_manifest_entry", repr(entry))
    if not SHA256.fullmatch(entry["sha256"]):
        raise OverlapBuildError("fetch_manifest_sha256", entry["dest"])
    return [{key: entry[key] for key in ("url", "dest", "sha256")}]


def county_source_pin(path: Path = FETCH_MANIFEST) -> dict[str, str]:
    """Return the sole official county source row."""
    entry = load_fetch_manifest(path)[0]
    if entry["url"] != SOURCE_URL or entry["dest"] != SOURCE_DEST:
        raise OverlapBuildError("county_source_pin", repr(entry))
    return entry


def expected_county_members() -> frozenset[str]:
    """Return the closed national COUNTY ZIP member set."""
    stem = "tl_2023_us_county"
    return frozenset(
        {
            f"{stem}.cpg",
            f"{stem}.dbf",
            f"{stem}.prj",
            f"{stem}.shp",
            f"{stem}.shp.ea.iso.xml",
            f"{stem}.shp.iso.xml",
            f"{stem}.shx",
        }
    )


def expected_water_members(county_fips: str) -> frozenset[str]:
    """Return the closed AREAWATER ZIP member set for one county."""
    if not COUNTY_FIPS.fullmatch(county_fips):
        raise OverlapBuildError("county_fips", county_fips)
    stem = f"tl_2023_{county_fips}_areawater"
    return frozenset(
        {
            f"{stem}.cpg",
            f"{stem}.dbf",
            f"{stem}.prj",
            f"{stem}.shp",
            f"{stem}.shp.ea.iso.xml",
            f"{stem}.shp.iso.xml",
            f"{stem}.shx",
        }
    )


def verify_zip_members(path: Path, expected_members: frozenset[str]) -> None:
    """Refuse unsafe, duplicate, unexpected, or oversized ZIP contents."""
    try:
        with zipfile.ZipFile(path) as archive:
            infos = archive.infolist()
    except (OSError, zipfile.BadZipFile) as error:
        raise OverlapBuildError("zip_decode", str(path)) from error
    if len(infos) > MAX_ZIP_MEMBERS:
        raise OverlapBuildError("zip_member_count", str(len(infos)))
    names: list[str] = []
    total = 0
    for info in infos:
        pure = PurePosixPath(info.filename)
        if (
            info.is_dir()
            or pure.is_absolute()
            or ".." in pure.parts
            or len(pure.parts) != 1
            or pure.name != info.filename
        ):
            raise OverlapBuildError("zip_member_path", info.filename)
        names.append(info.filename)
        total += info.file_size
    if len(names) != len(set(names)):
        raise OverlapBuildError("zip_member_duplicate", repr(names))
    if set(names) != expected_members:
        raise OverlapBuildError("zip_members", repr(sorted(names)))
    if total > MAX_ZIP_UNCOMPRESSED_BYTES:
        raise OverlapBuildError("zip_uncompressed_size", str(total))


def _verify_archive(
    path: Path,
    expected_sha256: str,
    expected_members: frozenset[str],
    *,
    maximum_bytes: int,
) -> None:
    try:
        size = path.stat().st_size
    except OSError as error:
        raise OverlapBuildError("source_read", str(path)) from error
    if size > maximum_bytes:
        raise OverlapBuildError("source_size", str(size))
    actual = _sha256(path)
    if actual != expected_sha256:
        raise OverlapBuildError("source_sha256", f"expected {expected_sha256}, got {actual}")
    verify_zip_members(path, expected_members)


def verify_source_archive(path: Path, expected_sha256: str) -> None:
    """Verify national COUNTY bytes before any ZIP decode."""
    _verify_archive(
        path,
        expected_sha256,
        expected_county_members(),
        maximum_bytes=MAX_SOURCE_ARCHIVE_BYTES,
    )


def _valid_polygonal(geometry: Any, code: str, detail: str) -> None:
    from shapely import get_coordinates

    if geometry is None or geometry.is_empty:
        raise OverlapBuildError(f"{code}_empty", detail)
    if geometry.geom_type not in {"Polygon", "MultiPolygon"}:
        raise OverlapBuildError(f"{code}_type", geometry.geom_type)
    if not geometry.is_valid:
        raise OverlapBuildError(f"{code}_invalid", detail)
    coordinates = get_coordinates(geometry, include_z=False)
    if len(coordinates) == 0 or not all(
        math.isfinite(float(value)) for coordinate in coordinates for value in coordinate
    ):
        raise OverlapBuildError(f"{code}_coordinate", detail)


def canonicalize_county_source(frame: Any) -> CountySource:
    """Validate and select the exact Michigan county rows."""
    if tuple(frame.columns) != COUNTY_SOURCE_COLUMNS:
        raise OverlapBuildError("county_source_schema", repr(tuple(frame.columns)))
    if frame.crs is None or frame.crs.to_epsg() != SOURCE_CRS_EPSG:
        raise OverlapBuildError("county_source_crs", str(frame.crs))
    if not 0 < len(frame) <= MAX_COUNTY_SOURCE_ROWS:
        raise OverlapBuildError("county_source_rows", str(len(frame)))
    selected = frame.loc[frame["STATEFP"] == STATE_FIPS].sort_values("GEOID")
    if not 0 < len(selected) <= MAX_COUNTIES:
        raise OverlapBuildError("county_state_rows", str(len(selected)))
    geometries: dict[str, Any] = {}
    for row in selected.itertuples(index=False):
        county_fips = str(row.GEOID)
        if (
            not COUNTY_FIPS.fullmatch(county_fips)
            or str(row.STATEFP) + str(row.COUNTYFP) != county_fips
            or str(row.GEOIDFQ) != f"0500000US{county_fips}"
        ):
            raise OverlapBuildError("county_source_geoid", county_fips)
        if county_fips in geometries:
            raise OverlapBuildError("county_source_duplicate", county_fips)
        if any(
            isinstance(value, bool) or not isinstance(value, Integral) or value < 0
            for value in (row.ALAND, row.AWATER)
        ):
            raise OverlapBuildError("county_source_area", county_fips)
        _valid_polygonal(row.geometry, "county_source_geometry", county_fips)
        geometries[county_fips] = row.geometry
    bounds = tuple(float(value) for value in selected.total_bounds)
    if len(bounds) != 4 or not all(math.isfinite(value) for value in bounds):
        raise OverlapBuildError("county_source_extent", repr(bounds))
    return CountySource(len(frame), geometries, bounds)  # type: ignore[arg-type]


def load_county_source(path: Path) -> CountySource:
    """Decode the already verified county archive."""
    import geopandas as gpd

    try:
        frame = gpd.read_file(f"zip://{path}", engine="pyogrio")
    except (OSError, ValueError) as error:
        raise OverlapBuildError("county_source_decode", str(path)) from error
    return canonicalize_county_source(frame)


def _load_json(path: Path, maximum: int) -> dict[str, Any]:
    try:
        raw = path.read_bytes()
    except OSError as error:
        raise OverlapBuildError("json_read", str(path)) from error
    if len(raw) > maximum:
        raise OverlapBuildError("json_size", str(path))
    try:
        value = json.loads(raw, object_pairs_hook=_unique_json_object)
    except (json.JSONDecodeError, UnicodeDecodeError) as error:
        raise OverlapBuildError("json_decode", str(path)) from error
    if not isinstance(value, dict):
        raise OverlapBuildError("json_shape", str(path))
    return value


def load_areawater_pins(
    county_fips: set[str], path: Path = PHASE0D_MANIFEST
) -> dict[str, dict[str, str]]:
    """Verify the frozen Phase 0-D manifest and return all 83 water pins."""
    _verify_file(path, PHASE0D_MANIFEST_SHA256)
    document = _load_json(path, MAX_FETCH_MANIFEST_BYTES)
    entries = document.get("entries")
    if not isinstance(entries, list) or len(entries) > 100:
        raise OverlapBuildError("areawater_manifest_shape", "entries")
    result: dict[str, dict[str, str]] = {}
    for raw in entries:
        if not isinstance(raw, dict) or set(raw) != {"url", "dest", "sha256"}:
            raise OverlapBuildError("areawater_manifest_entry", repr(raw))
        if not all(isinstance(raw[key], str) for key in ("url", "dest", "sha256")):
            raise OverlapBuildError("areawater_manifest_entry", repr(raw))
        destination = raw["dest"]
        match = re.fullmatch(r"tiger/areawater/tl_2023_(26[0-9]{3})_areawater\.zip", destination)
        if match is None:
            continue
        fips = match.group(1)
        expected_url = (
            f"https://www2.census.gov/geo/tiger/TIGER2023/AREAWATER/tl_2023_{fips}_areawater.zip"
        )
        if raw["url"] != expected_url or not SHA256.fullmatch(raw["sha256"]):
            raise OverlapBuildError("areawater_manifest_entry", destination)
        if fips in result:
            raise OverlapBuildError("areawater_manifest_duplicate", fips)
        result[fips] = {key: raw[key] for key in ("url", "dest", "sha256")}
    if set(result) != county_fips:
        raise OverlapBuildError(
            "areawater_manifest_counties",
            f"missing={sorted(county_fips - set(result))}; extra={sorted(set(result) - county_fips)}",
        )
    return result


def polygonal_components(geometry: Any) -> Any:
    """Keep only areal components; boundary-only touches mean zero area."""
    from shapely import get_parts, union_all
    from shapely.geometry import GeometryCollection

    pending = [geometry]
    polygons: list[Any] = []
    while pending:
        current = pending.pop()
        if current is None or current.is_empty:
            continue
        if current.geom_type == "Polygon":
            polygons.append(current)
        elif current.geom_type in {"MultiPolygon", "GeometryCollection"}:
            pending.extend(get_parts(current).tolist())
    if not polygons:
        return GeometryCollection()
    return union_all(polygons)


def quantize_area_m2(value: float) -> int:
    """Conservatively floor one finite nonnegative binary64 area to m²."""
    if not math.isfinite(value) or value < 0:
        raise OverlapBuildError("area_value", repr(value))
    return math.floor(value)


def verify_toolchain() -> None:
    """Refuse a geometry runtime outside the pinned build contract."""
    import pyproj
    import shapely

    actual = {name: importlib.metadata.version(name) for name in EXPECTED_PACKAGE_VERSIONS}
    if actual != EXPECTED_PACKAGE_VERSIONS:
        raise OverlapBuildError("toolchain_packages", repr(actual))
    if shapely.geos_version_string != EXPECTED_GEOS_VERSION:
        raise OverlapBuildError("toolchain_geos", shapely.geos_version_string)
    if pyproj.proj_version_str != EXPECTED_PROJ_VERSION:
        raise OverlapBuildError("toolchain_proj", pyproj.proj_version_str)


def verify_predecessor_files() -> None:
    """Verify every checked predecessor byte set before derivation."""
    for path, digest in (
        (H3_CONTRACT, H3_CONTRACT_SHA256),
        (H3_VECTORS, H3_VECTORS_SHA256),
        (PLACE_CONTRACT, PLACE_CONTRACT_SHA256),
        (PLACE_MANIFEST, PLACE_MANIFEST_SHA256),
        (PLACE_IDENTITY, PLACE_IDENTITY_SHA256),
        (PLACE_GEOMETRY, PLACE_GEOMETRY_SHA256),
    ):
        _verify_file(path, digest)


def load_h3_cohort(path: Path = H3_SOURCE) -> list[str]:
    """Load only canonical identities from the exact PER-275 release artifact."""
    import h3
    import pyarrow as pa
    import pyarrow.parquet as pq

    _verify_file(path, H3_SOURCE_SHA256, expected_bytes=H3_SOURCE_BYTES)
    try:
        table = pq.read_table(path)
    except (OSError, pa.ArrowException) as error:
        raise OverlapBuildError("h3_cohort_decode", str(path)) from error
    expected_schema = pa.schema(
        [
            pa.field("h3_index", pa.string(), nullable=False),
            pa.field("county_fips", pa.string(), nullable=False),
            pa.field("land_fraction", pa.float64(), nullable=False),
        ]
    )
    if table.schema != expected_schema or table.num_rows != H3_SOURCE_ROWS:
        raise OverlapBuildError("h3_cohort_shape", str(table.schema))
    cells = [str(value) for value in table["h3_index"].to_pylist()]
    if len(cells) > MAX_COHORT_ROWS or cells != sorted(cells, key=lambda value: int(value, 16)):
        raise OverlapBuildError("h3_cohort_order", str(len(cells)))
    if len(cells) != len(set(cells)):
        raise OverlapBuildError("h3_cohort_duplicate", str(len(cells)))
    for cell in cells:
        if (
            not h3.is_valid_cell(cell)
            or h3.get_resolution(cell) != 7
            or h3.int_to_str(h3.str_to_int(cell)) != cell
            or not 0 < h3.str_to_int(cell) < 1 << 63
        ):
            raise OverlapBuildError("h3_cohort_identity", cell)
    return cells


def project_h3_cells(cells: Sequence[str]) -> dict[int, Any]:
    """Construct canonical H3 polygons and project them to EPSG:5070."""
    import geopandas as gpd
    import h3
    from shapely.geometry import Polygon

    source = gpd.GeoSeries(
        [
            Polygon([(longitude, latitude) for latitude, longitude in h3.cell_to_boundary(cell)])
            for cell in cells
        ],
        crs="EPSG:4326",
    )
    projected = source.to_crs(WORKING_CRS)
    result: dict[int, Any] = {}
    for cell, geometry in zip(cells, projected.array, strict=True):
        cell_id = h3.str_to_int(cell)
        _valid_polygonal(geometry, "h3_geometry", cell)
        result[cell_id] = geometry
    return result


def load_place_geometries() -> dict[str, Any]:
    """Verify PER-276 and load its exact place WKB artifact into EPSG:5070."""
    import geopandas as gpd
    import pyarrow as pa
    import pyarrow.parquet as pq
    from shapely import from_wkb
    from verify_census_place_authority_v1 import (
        load_contract,
        verify_artifacts,
        verify_contract,
        verify_source_manifest,
    )

    contract = load_contract(PLACE_CONTRACT)
    verify_contract(contract)
    verify_source_manifest(contract, PLACE_MANIFEST)
    verify_artifacts(contract, REPO_ROOT)
    try:
        table = pq.read_table(PLACE_GEOMETRY)
    except (OSError, pa.ArrowException) as error:
        raise OverlapBuildError("place_artifact_decode", str(PLACE_GEOMETRY)) from error
    if table.num_rows > MAX_PLACE_ROWS:
        raise OverlapBuildError("place_rows", str(table.num_rows))
    place_ids = [str(value) for value in table["place_geoid"].to_pylist()]
    geometries = [from_wkb(value) for value in table["geometry_wkb"].to_pylist()]
    projected = gpd.GeoSeries(geometries, crs="EPSG:4269").to_crs(WORKING_CRS)
    result: dict[str, Any] = {}
    for place_geoid, geometry in zip(place_ids, projected.array, strict=True):
        if not PLACE_GEOID.fullmatch(place_geoid) or place_geoid in result:
            raise OverlapBuildError("place_identity", place_geoid)
        _valid_polygonal(geometry, "place_geometry", place_geoid)
        result[place_geoid] = geometry
    return result


def load_county_land_geometries(
    source: CountySource, pins: Mapping[str, Mapping[str, str]], *, progress: bool = False
) -> dict[str, Any]:
    """Subtract every verified county AREAWATER union in EPSG:5070."""
    import geopandas as gpd
    from shapely import difference

    county_fips = sorted(source.geometries)
    projected = gpd.GeoSeries(
        [source.geometries[fips] for fips in county_fips], crs="EPSG:4269"
    ).to_crs(WORKING_CRS)
    projected_counties = dict(zip(county_fips, projected.array, strict=True))
    result: dict[str, Any] = {}
    for ordinal, fips in enumerate(county_fips, start=1):
        pin = pins[fips]
        archive = TROVE / pin["dest"]
        _verify_archive(
            archive,
            pin["sha256"],
            expected_water_members(fips),
            maximum_bytes=MAX_SOURCE_ARCHIVE_BYTES,
        )
        try:
            frame = gpd.read_file(f"zip://{archive}", engine="pyogrio")
        except (OSError, ValueError) as error:
            raise OverlapBuildError("areawater_decode", fips) from error
        if tuple(frame.columns) != AREAWATER_SOURCE_COLUMNS:
            raise OverlapBuildError("areawater_schema", fips)
        if frame.crs is None or frame.crs.to_epsg() != SOURCE_CRS_EPSG:
            raise OverlapBuildError("areawater_crs", fips)
        if frame.empty:
            raise OverlapBuildError("areawater_rows", fips)
        for geometry in frame.geometry:
            _valid_polygonal(geometry, "areawater_geometry", fips)
        water = polygonal_components(frame.to_crs(WORKING_CRS).union_all())
        _valid_polygonal(water, "areawater_union", fips)
        land = polygonal_components(difference(projected_counties[fips], water))
        _valid_polygonal(land, "county_land", fips)
        result[fips] = land
        if progress and (ordinal % 10 == 0 or ordinal == len(county_fips)):
            print(f"verified county land {ordinal}/{len(county_fips)}: {fips}", flush=True)
    return result


def derive_overlap_rows(
    cell_geometries: Mapping[int, Any],
    county_land_geometries: Mapping[str, Any],
    place_geometries: Mapping[str, Any],
    *,
    progress: bool = False,
) -> tuple[list[CountyLandRow], list[PlaceLandRow]]:
    """Derive exact many-to-many positive land rows without owner collapse."""
    from shapely import STRtree, area, intersection

    if not 0 < len(cell_geometries) <= MAX_COHORT_ROWS:
        raise OverlapBuildError("cell_bound", str(len(cell_geometries)))
    if not 0 < len(county_land_geometries) <= MAX_COUNTIES:
        raise OverlapBuildError("county_bound", str(len(county_land_geometries)))
    if not 0 < len(place_geometries) <= MAX_PLACE_ROWS:
        raise OverlapBuildError("place_bound", str(len(place_geometries)))

    cell_ids = sorted(cell_geometries)
    cells = [cell_geometries[cell_id] for cell_id in cell_ids]
    place_ids = sorted(place_geometries)
    places = [place_geometries[place_id] for place_id in place_ids]
    for cell_id, geometry in zip(cell_ids, cells, strict=True):
        if not 0 < cell_id < 1 << 63:
            raise OverlapBuildError("cell_id", str(cell_id))
        _valid_polygonal(geometry, "cell_geometry", str(cell_id))
    for fips, geometry in county_land_geometries.items():
        if not COUNTY_FIPS.fullmatch(fips):
            raise OverlapBuildError("county_fips", fips)
        _valid_polygonal(geometry, "county_land", fips)
    for geoid, geometry in place_geometries.items():
        if not PLACE_GEOID.fullmatch(geoid):
            raise OverlapBuildError("place_geoid", geoid)
        _valid_polygonal(geometry, "place_geometry", geoid)

    cell_tree = STRtree(cells)
    place_tree = STRtree(places)
    county_rows: list[CountyLandRow] = []
    raw_places: list[tuple[int, str, str, int]] = []
    county_total = len(county_land_geometries)
    for ordinal, fips in enumerate(sorted(county_land_geometries), start=1):
        county_land = county_land_geometries[fips]
        candidate_place_indexes = sorted(
            int(index) for index in place_tree.query(county_land, predicate="intersects")
        )
        clipped_ids: list[str] = []
        clipped_places: list[Any] = []
        for place_index in candidate_place_indexes:
            clipped = polygonal_components(intersection(places[place_index], county_land))
            if clipped.is_empty or quantize_area_m2(float(area(clipped))) == 0:
                continue
            _valid_polygonal(clipped, "clipped_place", f"{fips}/{place_ids[place_index]}")
            clipped_ids.append(place_ids[place_index])
            clipped_places.append(clipped)
        clipped_tree = STRtree(clipped_places) if clipped_places else None

        candidate_cell_indexes = sorted(
            (int(index) for index in cell_tree.query(county_land, predicate="intersects")),
            key=lambda index: cell_ids[index],
        )
        for cell_index in candidate_cell_indexes:
            cell_land = polygonal_components(intersection(cells[cell_index], county_land))
            if cell_land.is_empty:
                continue
            land_area_m2 = quantize_area_m2(float(area(cell_land)))
            if land_area_m2 == 0:
                continue
            cell_id = cell_ids[cell_index]
            county_rows.append(CountyLandRow(cell_id, fips, land_area_m2))
            if clipped_tree is None:
                continue
            candidate_clipped_indexes = sorted(
                (int(index) for index in clipped_tree.query(cell_land, predicate="intersects")),
                key=lambda index: clipped_ids[index],
            )
            for clipped_index in candidate_clipped_indexes:
                placed = polygonal_components(
                    intersection(cell_land, clipped_places[clipped_index])
                )
                if placed.is_empty:
                    continue
                placed_area_m2 = quantize_area_m2(float(area(placed)))
                if placed_area_m2 == 0:
                    continue
                raw_places.append((cell_id, fips, clipped_ids[clipped_index], placed_area_m2))
        if len(county_rows) > MAX_COUNTY_ROWS or len(raw_places) > MAX_OVERLAP_ROWS:
            raise OverlapBuildError("artifact_row_bound", f"{len(county_rows)}/{len(raw_places)}")
        if progress and (ordinal % 10 == 0 or ordinal == county_total):
            print(
                f"derived overlap {ordinal}/{county_total}: {fips}; "
                f"county_rows={len(county_rows)} place_rows={len(raw_places)}",
                flush=True,
            )

    county_rows.sort()
    raw_places.sort()
    if len(county_rows) != len({(row.cell_id, row.county_fips) for row in county_rows}):
        raise OverlapBuildError("county_duplicate", str(len(county_rows)))
    if len(raw_places) != len({row[:3] for row in raw_places}):
        raise OverlapBuildError("place_duplicate", str(len(raw_places)))

    denominator: defaultdict[int, int] = defaultdict(int)
    county_limits: dict[tuple[int, str], int] = {}
    for row in county_rows:
        denominator[row.cell_id] += row.land_area_m2
        county_limits[(row.cell_id, row.county_fips)] = row.land_area_m2
    county_used: defaultdict[tuple[int, str], int] = defaultdict(int)
    cell_used: defaultdict[int, int] = defaultdict(int)
    place_rows: list[PlaceLandRow] = []
    for cell_id, fips, place_geoid, numerator in raw_places:
        cell_denominator = denominator[cell_id]
        county_used[(cell_id, fips)] += numerator
        cell_used[cell_id] += numerator
        if county_used[(cell_id, fips)] > county_limits[(cell_id, fips)]:
            raise OverlapBuildError("county_conservation", f"{cell_id}/{fips}")
        if cell_used[cell_id] > cell_denominator:
            raise OverlapBuildError("cell_conservation", str(cell_id))
        share = numerator * SHARE_SCALE // cell_denominator
        if not 0 < share <= SHARE_SCALE:
            raise OverlapBuildError("place_share", f"{cell_id}/{fips}/{place_geoid}")
        place_rows.append(
            PlaceLandRow(cell_id, fips, place_geoid, numerator, cell_denominator, share)
        )
    return county_rows, place_rows


def _semantic_sha256(
    domain: bytes, columns: Sequence[str], rows: Iterable[Sequence[object]]
) -> str:
    digest = hashlib.sha256(domain + b"\0")
    digest.update(json.dumps(list(columns), separators=(",", ":")).encode("ascii") + b"\n")
    for row in rows:
        digest.update(
            json.dumps(list(row), separators=(",", ":"), allow_nan=False).encode("ascii") + b"\n"
        )
    return digest.hexdigest()


def _identity_semantic_sha256(domain: bytes, raw: bytes) -> str:
    return hashlib.sha256(domain + b"\0" + raw).hexdigest()


def write_identity_fixture(
    path: Path,
    values: Sequence[str],
    *,
    domain: bytes,
    compressed: bool,
) -> ArtifactStats:
    """Write one deterministic newline-delimited identity allowlist."""
    if not values or len(values) > MAX_COHORT_ROWS or len(values) != len(set(values)):
        raise OverlapBuildError("identity_fixture_rows", str(len(values)))
    if any(not value or "\n" in value or "\r" in value for value in values):
        raise OverlapBuildError("identity_fixture_value", repr(values[:1]))
    try:
        raw = "".join(f"{value}\n" for value in values).encode("ascii")
    except UnicodeEncodeError as error:
        raise OverlapBuildError("identity_fixture_value", "non-ascii") from error
    if len(raw) > MAX_ZIP_UNCOMPRESSED_BYTES:
        raise OverlapBuildError("identity_fixture_size", str(len(raw)))
    if compressed:
        buffer = io.BytesIO()
        with gzip.GzipFile(
            filename="", mode="wb", fileobj=buffer, compresslevel=9, mtime=0
        ) as archive:
            archive.write(raw)
        payload = buffer.getvalue()
    else:
        payload = raw
    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        path.write_bytes(payload)
    except OSError as error:
        raise OverlapBuildError("identity_fixture_write", str(path)) from error
    return ArtifactStats(
        rows=len(values),
        bytes=len(payload),
        sha256=hashlib.sha256(payload).hexdigest(),
        semantic_sha256=_identity_semantic_sha256(domain, raw),
    )


def _write_parquet(path: Path, table: Any) -> None:
    import pyarrow as pa
    import pyarrow.parquet as pq

    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        pq.write_table(
            table,
            path,
            compression="zstd",
            compression_level=22,
            use_dictionary=False,
            write_statistics=False,
            data_page_version="1.0",
            version="2.6",
            row_group_size=max(table.num_rows, 1),
        )
    except (OSError, pa.ArrowException) as error:
        raise OverlapBuildError("artifact_write", str(path)) from error


def write_county_parquet(path: Path, rows: Sequence[CountyLandRow]) -> ArtifactStats:
    """Write deterministic cell-by-county land rows."""
    import pyarrow as pa

    schema = pa.schema(
        [
            pa.field("cell_id", pa.int64(), nullable=False),
            pa.field("county_fips", pa.string(), nullable=False),
            pa.field("land_area_m2", pa.uint64(), nullable=False),
        ]
    )
    table = pa.Table.from_arrays(
        [
            pa.array((row.cell_id for row in rows), type=pa.int64()),
            pa.array((row.county_fips for row in rows), type=pa.string()),
            pa.array((row.land_area_m2 for row in rows), type=pa.uint64()),
        ],
        schema=schema,
    )
    _write_parquet(path, table)
    values = [(row.cell_id, row.county_fips, row.land_area_m2) for row in rows]
    return ArtifactStats(
        len(rows),
        path.stat().st_size,
        _sha256(path),
        _semantic_sha256(b"babylon.census-county-h3-land-overlap.v1", COUNTY_COLUMNS, values),
    )


def write_place_parquet(path: Path, rows: Sequence[PlaceLandRow]) -> ArtifactStats:
    """Write deterministic cell-by-county-by-place land rows."""
    import pyarrow as pa

    schema = pa.schema(
        [
            pa.field("cell_id", pa.int64(), nullable=False),
            pa.field("county_fips", pa.string(), nullable=False),
            pa.field("place_geoid", pa.string(), nullable=False),
            pa.field("place_land_area_m2", pa.uint64(), nullable=False),
            pa.field("cell_mi_land_area_m2", pa.uint64(), nullable=False),
            pa.field("place_land_area_share_ppb", pa.uint32(), nullable=False),
        ]
    )
    table = pa.Table.from_arrays(
        [
            pa.array((row.cell_id for row in rows), type=pa.int64()),
            pa.array((row.county_fips for row in rows), type=pa.string()),
            pa.array((row.place_geoid for row in rows), type=pa.string()),
            pa.array((row.place_land_area_m2 for row in rows), type=pa.uint64()),
            pa.array((row.cell_mi_land_area_m2 for row in rows), type=pa.uint64()),
            pa.array((row.place_land_area_share_ppb for row in rows), type=pa.uint32()),
        ],
        schema=schema,
    )
    _write_parquet(path, table)
    values = [
        (
            row.cell_id,
            row.county_fips,
            row.place_geoid,
            row.place_land_area_m2,
            row.cell_mi_land_area_m2,
            row.place_land_area_share_ppb,
        )
        for row in rows
    ]
    return ArtifactStats(
        len(rows),
        path.stat().st_size,
        _sha256(path),
        _semantic_sha256(b"babylon.census-county-place-h3-land-overlap.v1", PLACE_COLUMNS, values),
    )


def build(
    *,
    source_path: Path | None = None,
    h3_source: Path = H3_SOURCE,
    county_identity_out: Path = COUNTY_IDENTITY_OUT,
    h3_identity_out: Path = H3_IDENTITY_OUT,
    county_out: Path = COUNTY_OUT,
    place_out: Path = PLACE_OUT,
    progress: bool = False,
) -> BuildResult:
    """Build both governed overlap artifacts from verified source bytes."""
    verify_toolchain()
    verify_predecessor_files()
    pin = county_source_pin()
    county_archive = TROVE / SOURCE_DEST if source_path is None else source_path
    verify_source_archive(county_archive, pin["sha256"])
    county_source = load_county_source(county_archive)
    if county_source.all_rows != 3_235 or len(county_source.geometries) != 83:
        raise OverlapBuildError(
            "county_source_cardinality",
            f"{county_source.all_rows}/{len(county_source.geometries)}",
        )
    pins = load_areawater_pins(set(county_source.geometries))
    county_land = load_county_land_geometries(county_source, pins, progress=progress)
    h3_cells = load_h3_cohort(h3_source)
    county_identity_stats = write_identity_fixture(
        county_identity_out,
        sorted(county_source.geometries),
        domain=b"babylon.census-county-geoids-mi-2023.v1",
        compressed=False,
    )
    h3_identity_stats = write_identity_fixture(
        h3_identity_out,
        h3_cells,
        domain=b"babylon.h3-res7-land-mask-identities.v1",
        compressed=True,
    )
    cells = project_h3_cells(h3_cells)
    places = load_place_geometries()
    county_rows, place_rows = derive_overlap_rows(cells, county_land, places, progress=progress)
    county_stats = write_county_parquet(county_out, county_rows)
    place_stats = write_place_parquet(place_out, place_rows)

    denominator = {row.cell_id for row in county_rows}
    place_cells = {row.cell_id for row in place_rows}
    counties_by_cell = Counter(row.cell_id for row in county_rows)
    counties_by_place: defaultdict[str, set[str]] = defaultdict(set)
    for row in place_rows:
        counties_by_place[row.place_geoid].add(row.county_fips)
    return BuildResult(
        county_identity_fixture=county_identity_stats,
        h3_identity_fixture=h3_identity_stats,
        county_artifact=county_stats,
        place_artifact=place_stats,
        county_source_rows=county_source.all_rows,
        county_rows=len(county_rows),
        place_rows=len(place_rows),
        positive_land_cells=len(denominator),
        cohort_absent_cells=len(h3_cells) - len(denominator),
        cross_county_cells=sum(count > 1 for count in counties_by_cell.values()),
        places_crossing_counties=sum(len(counties) > 1 for counties in counties_by_place.values()),
        land_cells_without_place=len(denominator - place_cells),
        total_mi_land_area_m2=sum(row.land_area_m2 for row in county_rows),
        total_place_land_area_m2=sum(row.place_land_area_m2 for row in place_rows),
    )


def make_data_artifacts_specs() -> tuple[Any, ...]:
    """Expose the SQLite-backed generator census for the registry tripwire."""
    import make_data_artifacts

    return make_data_artifacts.ARTIFACTS


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, help="override the county archive")
    parser.add_argument("--h3-source", type=Path, default=H3_SOURCE)
    parser.add_argument("--progress", action="store_true")
    args = parser.parse_args(argv)
    result = build(source_path=args.source, h3_source=args.h3_source, progress=args.progress)
    print(json.dumps(asdict(result), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
