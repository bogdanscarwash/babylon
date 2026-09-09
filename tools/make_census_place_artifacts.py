#!/usr/bin/env python3
"""Build PER-276's deterministic Michigan Census place authority artifacts.

The raw TIGER/Line archive is local acquisition evidence. Its URL and
SHA-256 live in ``tools/census_place_authority_v1_fetch_manifest.json`` and
the digest is verified before the ZIP is opened. The two small, immutable
outputs are checked in so CI and downstream work never infer authority from
the local ``babylon-data`` trove.
"""

from __future__ import annotations

import argparse
import csv
import gzip
import hashlib
import io
import json
import math
import sys
import zipfile
from collections.abc import Iterable, Sequence
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any, Final

REPO_ROOT = Path(__file__).resolve().parents[1]
FETCH_MANIFEST = REPO_ROOT / "tools" / "census_place_authority_v1_fetch_manifest.json"
TROVE = Path("/media/user/data/babylon-data")
SOURCE_DEST: Final = "tiger/place/tl_2023_26_place.zip"
SOURCE_URL: Final = "https://www2.census.gov/geo/tiger/TIGER2023/PLACE/tl_2023_26_place.zip"
IDENTITY_OUT = (
    REPO_ROOT
    / "src"
    / "babylon"
    / "data"
    / "reference"
    / "spatial"
    / "census_place_identity_mi_2023.csv.gz"
)
GEOMETRY_OUT = IDENTITY_OUT.with_name("census_place_geometry_mi_2023.parquet")

IDENTITY_COLUMNS: Final = (
    "place_geoid",
    "state_fips",
    "place_fips",
    "place_ns",
    "name",
    "name_lsad",
    "lsad",
    "class_fp",
    "principal_city_indicator",
    "mtfcc",
    "functional_status",
)
GEOMETRY_COLUMNS: Final = (
    "place_geoid",
    "geometry_wkb",
    "aland_m2",
    "awater_m2",
    "internal_point_lat",
    "internal_point_lon",
)
ARTIFACT_NAMES: Final = (
    "census_place_identity_mi_2023",
    "census_place_geometry_mi_2023",
)
EXPECTED_SOURCE_COLUMNS: Final = (
    "STATEFP",
    "PLACEFP",
    "PLACENS",
    "GEOID",
    "GEOIDFQ",
    "NAME",
    "NAMELSAD",
    "LSAD",
    "CLASSFP",
    "PCICBSA",
    "MTFCC",
    "FUNCSTAT",
    "ALAND",
    "AWATER",
    "INTPTLAT",
    "INTPTLON",
    "geometry",
)
EXPECTED_ZIP_MEMBERS: Final = frozenset(
    {
        "tl_2023_26_place.cpg",
        "tl_2023_26_place.dbf",
        "tl_2023_26_place.prj",
        "tl_2023_26_place.shp",
        "tl_2023_26_place.shp.ea.iso.xml",
        "tl_2023_26_place.shp.iso.xml",
        "tl_2023_26_place.shx",
    }
)

MAX_FETCH_MANIFEST_BYTES: Final = 131_072
MAX_FETCH_ENTRIES: Final = 1
MAX_SOURCE_ARCHIVE_BYTES: Final = 4_194_304
MAX_ZIP_MEMBERS: Final = 8
MAX_ZIP_UNCOMPRESSED_BYTES: Final = 8_388_608
MAX_PLACE_ROWS: Final = 2_048


class PlaceAuthorityBuildError(ValueError):
    """One typed refusal from the PER-276 artifact builder."""

    def __init__(self, code: str, detail: str) -> None:
        self.code = code
        self.detail = detail
        super().__init__(f"{code}: {detail}")


@dataclass(frozen=True)
class ArtifactStats:
    """Deterministic output evidence returned by :func:`build`."""

    rows: int
    sha256: str
    semantic_sha256: str


@dataclass(frozen=True)
class BuildResult:
    """Evidence for both artifacts and the decoded source extent."""

    identity: ArtifactStats
    geometry: ArtifactStats
    extent: tuple[float, float, float, float]


def _unique_json_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise PlaceAuthorityBuildError("fetch_manifest_duplicate_key", key)
        result[key] = value
    return result


def load_fetch_manifest(path: Path = FETCH_MANIFEST) -> list[dict[str, str]]:
    """Load the bounded acquisition manifest with duplicate-key refusal."""
    try:
        raw = path.read_bytes()
    except OSError as error:
        raise PlaceAuthorityBuildError("fetch_manifest_read", str(path)) from error
    if len(raw) > MAX_FETCH_MANIFEST_BYTES:
        raise PlaceAuthorityBuildError("fetch_manifest_size", str(len(raw)))
    try:
        document = json.loads(raw, object_pairs_hook=_unique_json_object)
    except (json.JSONDecodeError, UnicodeDecodeError) as error:
        raise PlaceAuthorityBuildError("fetch_manifest_json", str(path)) from error
    if not isinstance(document, dict) or document != {
        "contract": "CensusPlaceAuthorityV1",
        "version": 1,
        "entries": document.get("entries") if isinstance(document, dict) else None,
    }:
        raise PlaceAuthorityBuildError("fetch_manifest_shape", "root")
    entries = document.get("entries")
    if not isinstance(entries, list) or len(entries) != MAX_FETCH_ENTRIES:
        raise PlaceAuthorityBuildError("fetch_manifest_shape", "entries")
    result: list[dict[str, str]] = []
    destinations: set[str] = set()
    for entry in entries:
        if not isinstance(entry, dict) or set(entry) != {"url", "dest", "sha256"}:
            raise PlaceAuthorityBuildError("fetch_manifest_entry", repr(entry))
        if not all(isinstance(entry[key], str) for key in ("url", "dest", "sha256")):
            raise PlaceAuthorityBuildError("fetch_manifest_entry", repr(entry))
        digest = entry["sha256"]
        if len(digest) != 64 or any(char not in "0123456789abcdef" for char in digest):
            raise PlaceAuthorityBuildError("fetch_manifest_sha256", entry["dest"])
        if entry["dest"] in destinations:
            raise PlaceAuthorityBuildError("fetch_manifest_duplicate_dest", entry["dest"])
        destinations.add(entry["dest"])
        result.append({key: entry[key] for key in ("url", "dest", "sha256")})
    return result


def place_source_pin(path: Path = FETCH_MANIFEST) -> dict[str, str]:
    """Return the sole governed PLACE source row."""
    matches = [entry for entry in load_fetch_manifest(path) if entry["dest"] == SOURCE_DEST]
    if len(matches) != 1 or matches[0]["url"] != SOURCE_URL:
        raise PlaceAuthorityBuildError("place_source_pin", SOURCE_DEST)
    return matches[0]


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    try:
        with path.open("rb") as handle:
            for chunk in iter(lambda: handle.read(1 << 20), b""):
                digest.update(chunk)
    except OSError as error:
        raise PlaceAuthorityBuildError("file_read", str(path)) from error
    return digest.hexdigest()


def verify_zip_members(path: Path) -> None:
    """Refuse unsafe, duplicate, unexpected, or oversized ZIP contents."""
    try:
        with zipfile.ZipFile(path) as archive:
            infos = archive.infolist()
    except (OSError, zipfile.BadZipFile) as error:
        raise PlaceAuthorityBuildError("zip_decode", str(path)) from error
    if len(infos) > MAX_ZIP_MEMBERS:
        raise PlaceAuthorityBuildError("zip_member_count", str(len(infos)))
    names: list[str] = []
    total_size = 0
    for info in infos:
        pure = PurePosixPath(info.filename)
        if (
            info.is_dir()
            or pure.is_absolute()
            or ".." in pure.parts
            or len(pure.parts) != 1
            or pure.name != info.filename
        ):
            raise PlaceAuthorityBuildError("zip_member_path", info.filename)
        names.append(info.filename)
        total_size += info.file_size
    if len(names) != len(set(names)):
        raise PlaceAuthorityBuildError("zip_member_duplicate", repr(names))
    if set(names) != EXPECTED_ZIP_MEMBERS:
        raise PlaceAuthorityBuildError("zip_members", repr(sorted(names)))
    if total_size > MAX_ZIP_UNCOMPRESSED_BYTES:
        raise PlaceAuthorityBuildError("zip_uncompressed_size", str(total_size))


def verify_source_archive(path: Path, expected_sha256: str) -> None:
    """Verify the source digest before any ZIP decode occurs."""
    try:
        size = path.stat().st_size
    except OSError as error:
        raise PlaceAuthorityBuildError("source_read", str(path)) from error
    if size > MAX_SOURCE_ARCHIVE_BYTES:
        raise PlaceAuthorityBuildError("source_size", str(size))
    actual = _sha256(path)
    if actual != expected_sha256:
        raise PlaceAuthorityBuildError("source_sha256", f"expected {expected_sha256}, got {actual}")
    verify_zip_members(path)


def _required_text(value: object, field: str, *, length: int | None = None) -> str:
    if not isinstance(value, str) or not value:
        raise PlaceAuthorityBuildError("source_text", field)
    if length is not None and len(value) != length:
        raise PlaceAuthorityBuildError("source_text_length", field)
    return value


def _required_nonnegative_int(value: object, field: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise PlaceAuthorityBuildError("source_integer", field)
    return value


def _finite_coordinate(value: object, field: str, minimum: float, maximum: float) -> str:
    text = _required_text(value, field)
    try:
        number = float(text)
    except ValueError as error:
        raise PlaceAuthorityBuildError("source_coordinate", field) from error
    if not math.isfinite(number) or not minimum <= number <= maximum:
        raise PlaceAuthorityBuildError("source_coordinate", field)
    return text


def canonicalize_place_rows(
    frame: Any,
) -> tuple[list[list[object]], list[list[object]], tuple[float, float, float, float]]:
    """Validate one GeoDataFrame and return sorted canonical artifact rows."""
    from shapely import get_coordinates, to_wkb

    if tuple(frame.columns) != EXPECTED_SOURCE_COLUMNS:
        raise PlaceAuthorityBuildError("source_schema", repr(tuple(frame.columns)))
    if frame.crs is None or frame.crs.to_epsg() != 4269:
        raise PlaceAuthorityBuildError("source_crs", str(frame.crs))
    if not 0 < len(frame) <= MAX_PLACE_ROWS:
        raise PlaceAuthorityBuildError("source_rows", str(len(frame)))

    identities: list[list[object]] = []
    geometries: list[list[object]] = []
    seen: set[str] = set()
    for source in frame.itertuples(index=False, name=None):
        row = dict(zip(EXPECTED_SOURCE_COLUMNS, source, strict=True))
        state_fips = _required_text(row["STATEFP"], "STATEFP", length=2)
        place_fips = _required_text(row["PLACEFP"], "PLACEFP", length=5)
        place_geoid = _required_text(row["GEOID"], "GEOID", length=7)
        if not (state_fips + place_fips == place_geoid and state_fips == "26"):
            raise PlaceAuthorityBuildError("source_geoid", place_geoid)
        if not place_geoid.isascii() or not place_geoid.isdigit():
            raise PlaceAuthorityBuildError("source_geoid", place_geoid)
        if place_geoid in seen:
            raise PlaceAuthorityBuildError("source_duplicate_geoid", place_geoid)
        seen.add(place_geoid)
        geoid_fq = _required_text(row["GEOIDFQ"], "GEOIDFQ", length=16)
        if geoid_fq != f"1600000US{place_geoid}":
            raise PlaceAuthorityBuildError("source_geoid_fq", geoid_fq)

        geometry = row["geometry"]
        if geometry is None or geometry.is_empty:
            raise PlaceAuthorityBuildError("source_geometry_empty", place_geoid)
        if geometry.geom_type not in {"Polygon", "MultiPolygon"}:
            raise PlaceAuthorityBuildError("source_geometry_type", geometry.geom_type)
        if not geometry.is_valid:
            raise PlaceAuthorityBuildError("source_geometry_invalid", place_geoid)
        coordinates = get_coordinates(geometry, include_z=False)
        if len(coordinates) == 0 or not all(
            math.isfinite(float(value)) for coordinate in coordinates for value in coordinate
        ):
            raise PlaceAuthorityBuildError("source_geometry_coordinate", place_geoid)
        wkb_hex = to_wkb(
            geometry,
            hex=True,
            byte_order=1,
            output_dimension=2,
            include_srid=False,
            flavor="iso",
        )
        if not isinstance(wkb_hex, str):
            raise PlaceAuthorityBuildError("source_geometry_wkb", place_geoid)

        identities.append(
            [
                place_geoid,
                state_fips,
                place_fips,
                _required_text(row["PLACENS"], "PLACENS", length=8),
                _required_text(row["NAME"], "NAME"),
                _required_text(row["NAMELSAD"], "NAMELSAD"),
                _required_text(row["LSAD"], "LSAD", length=2),
                _required_text(row["CLASSFP"], "CLASSFP", length=2),
                _required_text(row["PCICBSA"], "PCICBSA", length=1),
                _required_text(row["MTFCC"], "MTFCC", length=5),
                _required_text(row["FUNCSTAT"], "FUNCSTAT", length=1),
            ]
        )
        geometries.append(
            [
                place_geoid,
                wkb_hex.lower(),
                _required_nonnegative_int(row["ALAND"], "ALAND"),
                _required_nonnegative_int(row["AWATER"], "AWATER"),
                _finite_coordinate(row["INTPTLAT"], "INTPTLAT", -90.0, 90.0),
                _finite_coordinate(row["INTPTLON"], "INTPTLON", -180.0, 180.0),
            ]
        )

    identities.sort(key=lambda row: str(row[0]))
    geometries.sort(key=lambda row: str(row[0]))
    bounds = tuple(float(value) for value in frame.total_bounds)
    if len(bounds) != 4 or not all(math.isfinite(value) for value in bounds):
        raise PlaceAuthorityBuildError("source_extent", repr(bounds))
    return identities, geometries, bounds  # type: ignore[return-value]


def _semantic_sha256(
    domain: bytes, columns: Sequence[str], rows: Iterable[Sequence[object]]
) -> str:
    digest = hashlib.sha256(domain + b"\0")
    digest.update(json.dumps(list(columns), separators=(",", ":")).encode("ascii") + b"\n")
    for row in rows:
        digest.update(
            json.dumps(list(row), ensure_ascii=True, separators=(",", ":"), allow_nan=False).encode(
                "ascii"
            )
            + b"\n"
        )
    return digest.hexdigest()


def _write_gzip_csv(
    path: Path, columns: Sequence[str], rows: Sequence[Sequence[object]]
) -> ArtifactStats:
    path.parent.mkdir(parents=True, exist_ok=True)
    with (
        path.open("wb") as raw,
        gzip.GzipFile(filename="", fileobj=raw, mode="wb", compresslevel=9, mtime=0) as binary,
        io.TextIOWrapper(binary, encoding="utf-8", newline="") as text,
    ):
        writer = csv.writer(text, lineterminator="\n")
        writer.writerow(columns)
        writer.writerows(rows)
    domains = {
        IDENTITY_COLUMNS: b"babylon.census-place-identity.v1",
        GEOMETRY_COLUMNS: b"babylon.census-place-geometry.v1",
    }
    try:
        domain = domains[tuple(columns)]
    except KeyError as error:
        raise PlaceAuthorityBuildError("artifact_columns", repr(tuple(columns))) from error
    return ArtifactStats(
        rows=len(rows),
        sha256=_sha256(path),
        semantic_sha256=_semantic_sha256(domain, columns, rows),
    )


def _write_geometry_parquet(path: Path, rows: Sequence[Sequence[object]]) -> ArtifactStats:
    """Write deterministic raw-WKB Parquet below the ordinary-blob budget."""
    import pyarrow as pa
    import pyarrow.parquet as pq

    schema = pa.schema(
        [
            ("place_geoid", pa.string()),
            ("geometry_wkb", pa.binary()),
            ("aland_m2", pa.uint64()),
            ("awater_m2", pa.uint64()),
            ("internal_point_lat", pa.string()),
            ("internal_point_lon", pa.string()),
        ]
    )
    try:
        columns = list(zip(*rows, strict=True))
        table = pa.Table.from_arrays(
            [
                pa.array(columns[0], type=pa.string()),
                pa.array((bytes.fromhex(str(value)) for value in columns[1]), type=pa.binary()),
                pa.array(columns[2], type=pa.uint64()),
                pa.array(columns[3], type=pa.uint64()),
                pa.array(columns[4], type=pa.string()),
                pa.array(columns[5], type=pa.string()),
            ],
            schema=schema,
        )
    except (pa.ArrowException, IndexError, TypeError, ValueError) as error:
        raise PlaceAuthorityBuildError("geometry_parquet_value", str(path)) from error
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
        )
    except (pa.ArrowException, OSError) as error:
        raise PlaceAuthorityBuildError("geometry_parquet_write", str(path)) from error
    return ArtifactStats(
        rows=len(rows),
        sha256=_sha256(path),
        semantic_sha256=_semantic_sha256(
            b"babylon.census-place-geometry.v1", GEOMETRY_COLUMNS, rows
        ),
    )


def build(
    *,
    source_path: Path | None = None,
    identity_out: Path = IDENTITY_OUT,
    geometry_out: Path = GEOMETRY_OUT,
    fetch_manifest: Path = FETCH_MANIFEST,
) -> BuildResult:
    """Build both checked artifacts from the verified official archive."""
    import geopandas as gpd
    from pyogrio.errors import (
        CRSError,
        DataLayerError,
        DataSourceError,
        FeatureError,
        FieldError,
        GeometryError,
    )

    pin = place_source_pin(fetch_manifest)
    source = TROVE / SOURCE_DEST if source_path is None else source_path
    verify_source_archive(source, pin["sha256"])
    try:
        frame = gpd.read_file(f"zip://{source}", engine="pyogrio")
    except (
        CRSError,
        DataLayerError,
        DataSourceError,
        FeatureError,
        FieldError,
        GeometryError,
        OSError,
        ValueError,
    ) as error:
        raise PlaceAuthorityBuildError("source_decode", str(source)) from error
    identities, geometries, extent = canonicalize_place_rows(frame)
    return BuildResult(
        identity=_write_gzip_csv(identity_out, IDENTITY_COLUMNS, identities),
        geometry=_write_geometry_parquet(geometry_out, geometries),
        extent=extent,
    )


def make_data_artifacts_specs() -> tuple[Any, ...]:
    """Expose the SQLite-backed generator census for the tripwire test."""
    import make_data_artifacts

    return make_data_artifacts.ARTIFACTS


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, help="override the manifest destination for tests")
    args = parser.parse_args(argv)
    result = build(source_path=args.source)
    print(
        json.dumps(
            {
                "identity": result.identity.__dict__,
                "geometry": result.geometry.__dict__,
                "extent": result.extent,
            },
            indent=2,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
