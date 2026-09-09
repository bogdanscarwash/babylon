#!/usr/bin/env python3
"""Independently verify the bounded PER-276 Census place authority contract."""

from __future__ import annotations

import argparse
import csv
import gzip
import hashlib
import io
import json
import math
import re
import struct
import sys
from collections.abc import Iterable, Sequence
from pathlib import Path, PurePosixPath
from typing import Any, Final

import yaml
from shapely import from_wkb, get_coordinates, to_wkb
from shapely.errors import GEOSException
from yaml.constructor import ConstructorError
from yaml.nodes import MappingNode

EXPECTED_META: Final = {
    "contract": "CensusPlaceAuthorityV1",
    "version": 1,
    "issue": "PER-276",
    "parent": "PER-21",
}
EXPECTED_CLASSIFICATIONS: Final = {
    "source_url_bytes_vintage_schema_crs_and_members": "Observed",
    "artifact_bytes_rows_columns_and_extent": "Observed",
    "semantic_digests": "Derived",
    "canonical_formats_ordering_and_geometry_encoding": "Designed",
    "bounds_refusals_absences_and_lineage": "Designed",
}
EXPECTED_SOURCE_URL: Final = (
    "https://www2.census.gov/geo/tiger/TIGER2023/PLACE/tl_2023_26_place.zip"
)
EXPECTED_SOURCE_DEST: Final = "tiger/place/tl_2023_26_place.zip"
EXPECTED_SOURCE_MANIFEST: Final = "tools/census_place_authority_v1_fetch_manifest.json"
EXPECTED_ZIP_MEMBERS: Final = {
    "tl_2023_26_place.cpg",
    "tl_2023_26_place.dbf",
    "tl_2023_26_place.prj",
    "tl_2023_26_place.shp",
    "tl_2023_26_place.shp.ea.iso.xml",
    "tl_2023_26_place.shp.iso.xml",
    "tl_2023_26_place.shx",
}
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
EXPECTED_ARTIFACTS: Final = {
    "identity_artifact": (
        "census_place_identity_mi_2023",
        "src/babylon/data/reference/spatial/census_place_identity_mi_2023.csv.gz",
        IDENTITY_COLUMNS,
    ),
    "geometry_artifact": (
        "census_place_geometry_mi_2023",
        "src/babylon/data/reference/spatial/census_place_geometry_mi_2023.parquet",
        GEOMETRY_COLUMNS,
    ),
}
EXPECTED_ABSENCES: Final = {
    "allocation_weights",
    "county_membership",
    "county_place_h3_overlap",
    "place_land_area_share",
    "postgresql_schema",
    "runtime_importer",
    "cutover_authority",
}
EXPECTED_BOUNDS: Final = {
    "contract_bytes": 65_536,
    "artifact_bytes": 1_048_576,
    "artifact_uncompressed_bytes": 33_554_432,
    "artifact_rows": 2_048,
    "csv_field_bytes": 4_194_304,
}
SHA256 = re.compile(r"^[0-9a-f]{64}$")
ASCII_DIGITS = re.compile(r"^[0-9]+$")


class CensusPlaceAuthorityRefusal(ValueError):
    """One typed refusal from the independent PER-276 verifier."""

    def __init__(self, code: str, detail: str) -> None:
        self.code = code
        self.detail = detail
        super().__init__(f"{code}: {detail}")


class _UniqueKeyLoader(yaml.SafeLoader):
    """Safe YAML loader that rejects duplicate mapping keys."""


def _construct_unique_mapping(
    loader: _UniqueKeyLoader, node: MappingNode, deep: bool = False
) -> dict[Any, Any]:
    mapping: dict[Any, Any] = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node, deep=deep)
        if key in mapping:
            raise ConstructorError(
                "while constructing a mapping",
                node.start_mark,
                f"found duplicate key {key!r}",
                key_node.start_mark,
            )
        mapping[key] = loader.construct_object(value_node, deep=deep)
    return mapping


_UniqueKeyLoader.add_constructor(
    yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, _construct_unique_mapping
)


def _safe_load_unique(raw: bytes) -> Any:
    loader = _UniqueKeyLoader(raw)
    try:
        return loader.get_single_data()
    finally:
        loader.dispose()


def _bounded_bytes(path: Path, maximum: int, code: str) -> bytes:
    try:
        size = path.stat().st_size
    except OSError as error:
        raise CensusPlaceAuthorityRefusal("file_read", str(path)) from error
    if size > maximum:
        raise CensusPlaceAuthorityRefusal(code, str(size))
    try:
        return path.read_bytes()
    except OSError as error:
        raise CensusPlaceAuthorityRefusal("file_read", str(path)) from error


def load_contract(path: Path) -> dict[str, Any]:
    """Load one bounded YAML contract mapping with duplicate-key refusal."""
    raw = _bounded_bytes(path, EXPECTED_BOUNDS["contract_bytes"], "contract_size")
    try:
        document = _safe_load_unique(raw)
    except yaml.YAMLError as error:
        raise CensusPlaceAuthorityRefusal("invalid_contract", str(path)) from error
    if not isinstance(document, dict):
        raise CensusPlaceAuthorityRefusal("invalid_contract", "root mapping")
    return document


def _require_sha256(value: object, detail: str) -> str:
    if not isinstance(value, str) or SHA256.fullmatch(value) is None:
        raise CensusPlaceAuthorityRefusal("contract_sha256", detail)
    return value


def _artifact_spec(contract: dict[str, Any], key: str) -> dict[str, Any]:
    value = contract.get(key)
    if not isinstance(value, dict):
        raise CensusPlaceAuthorityRefusal("contract_shape", key)
    return value


def verify_contract(contract: dict[str, Any]) -> None:
    """Verify the closed contract shape and declared authority boundary."""
    if set(contract) != {
        "meta",
        "classifications",
        "bounds",
        "source",
        "identity_artifact",
        "geometry_artifact",
        "extent",
        "declared_absences",
        "lineage",
    }:
        raise CensusPlaceAuthorityRefusal("contract_shape", "root")
    if (
        contract.get("meta") != EXPECTED_META
        or contract.get("classifications") != EXPECTED_CLASSIFICATIONS
        or contract.get("bounds") != EXPECTED_BOUNDS
    ):
        raise CensusPlaceAuthorityRefusal("contract_shape", "meta_classifications_or_bounds")
    source = contract.get("source")
    if not isinstance(source, dict) or set(source) != {
        "manifest",
        "manifest_sha256",
        "url",
        "dest",
        "sha256",
        "vintage",
        "state_fips",
        "input_crs",
        "zip_members",
    }:
        raise CensusPlaceAuthorityRefusal("contract_shape", "source")
    if (
        source.get("manifest") != EXPECTED_SOURCE_MANIFEST
        or source.get("url") != EXPECTED_SOURCE_URL
        or source.get("dest") != EXPECTED_SOURCE_DEST
        or source.get("vintage") != 2023
        or source.get("state_fips") != "26"
        or source.get("input_crs") != "EPSG:4269"
        or set(source.get("zip_members", [])) != EXPECTED_ZIP_MEMBERS
        or len(source.get("zip_members", [])) != len(EXPECTED_ZIP_MEMBERS)
    ):
        raise CensusPlaceAuthorityRefusal("contract_shape", "source authority")
    _require_sha256(source.get("manifest_sha256"), "source_manifest")
    _require_sha256(source.get("sha256"), "source")

    common_keys = {
        "name",
        "path",
        "format",
        "compression",
        "rows",
        "columns",
        "ordering",
        "sha256",
        "semantic_sha256",
    }
    for key, (name, path, columns) in EXPECTED_ARTIFACTS.items():
        spec = _artifact_spec(contract, key)
        expected_keys = common_keys | (
            {"geometry_encoding"} if key == "geometry_artifact" else set()
        )
        if set(spec) != expected_keys:
            raise CensusPlaceAuthorityRefusal("contract_shape", key)
        expected_format = "parquet" if key == "geometry_artifact" else "csv.gz"
        expected_compression = (
            "parquet-zstd-level-22" if key == "geometry_artifact" else "gzip-mtime-0"
        )
        if (
            spec.get("name") != name
            or spec.get("path") != path
            or spec.get("format") != expected_format
            or spec.get("compression") != expected_compression
            or spec.get("columns") != list(columns)
            or spec.get("ordering") != "place_geoid-ascending"
            or isinstance(spec.get("rows"), bool)
            or not isinstance(spec.get("rows"), int)
            or not 0 < spec["rows"] <= EXPECTED_BOUNDS["artifact_rows"]
        ):
            raise CensusPlaceAuthorityRefusal("contract_shape", key)
        if key == "geometry_artifact" and spec.get("geometry_encoding") != (
            "ogc-iso-wkb-little-endian-2d-binary"
        ):
            raise CensusPlaceAuthorityRefusal("contract_shape", "geometry_encoding")
        _require_sha256(spec.get("sha256"), f"{key}.sha256")
        _require_sha256(spec.get("semantic_sha256"), f"{key}.semantic_sha256")

    extent = contract.get("extent")
    if not isinstance(extent, dict) or set(extent) != {
        "min_lon",
        "min_lat",
        "max_lon",
        "max_lat",
    }:
        raise CensusPlaceAuthorityRefusal("contract_shape", "extent")
    values = [extent[key] for key in ("min_lon", "min_lat", "max_lon", "max_lat")]
    if any(
        isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value)
        for value in values
    ) or not (values[0] <= values[2] and values[1] <= values[3]):
        raise CensusPlaceAuthorityRefusal("contract_shape", "extent values")

    absences = contract.get("declared_absences")
    if (
        not isinstance(absences, list)
        or set(absences) != EXPECTED_ABSENCES
        or len(absences) != len(EXPECTED_ABSENCES)
    ):
        raise CensusPlaceAuthorityRefusal("contract_shape", "declared_absences")
    lineage = contract.get("lineage")
    if not isinstance(lineage, dict) or lineage != {
        "historical_contract": "contracts/h3_estate_contract_v1.yaml",
        "resolves_historical_gaps": ["census_place_identity", "census_place_geometry"],
        "remaining_blocker": "county_place_h3_overlap",
        "database_dependency_owner": "PER-20",
    }:
        raise CensusPlaceAuthorityRefusal("contract_shape", "lineage")


def _unique_json_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise CensusPlaceAuthorityRefusal("source_manifest_duplicate_key", key)
        result[key] = value
    return result


def verify_source_manifest(contract: dict[str, Any], path: Path) -> None:
    """Verify that the acquisition manifest carries the exact contract pin."""
    raw = _bounded_bytes(path, 131_072, "source_manifest_size")
    actual_sha256 = hashlib.sha256(raw).hexdigest()
    if actual_sha256 != contract["source"]["manifest_sha256"]:
        raise CensusPlaceAuthorityRefusal("source_manifest_sha256", actual_sha256)
    try:
        document = json.loads(raw, object_pairs_hook=_unique_json_object)
    except (json.JSONDecodeError, UnicodeDecodeError) as error:
        raise CensusPlaceAuthorityRefusal("source_manifest_json", str(path)) from error
    if not isinstance(document, dict) or document != {
        "contract": "CensusPlaceAuthorityV1",
        "version": 1,
        "entries": document.get("entries") if isinstance(document, dict) else None,
    }:
        raise CensusPlaceAuthorityRefusal("source_manifest_shape", "root")
    entries = document.get("entries")
    if not isinstance(entries, list) or len(entries) != 1:
        raise CensusPlaceAuthorityRefusal("source_manifest_shape", "entries")
    matches = [
        entry
        for entry in entries
        if isinstance(entry, dict) and entry.get("dest") == EXPECTED_SOURCE_DEST
    ]
    expected = contract["source"]
    if matches != [
        {"url": expected["url"], "dest": expected["dest"], "sha256": expected["sha256"]}
    ]:
        raise CensusPlaceAuthorityRefusal("source_manifest_pin", EXPECTED_SOURCE_DEST)


def _artifact_path(root: Path, raw: object) -> Path:
    if not isinstance(raw, str):
        raise CensusPlaceAuthorityRefusal("artifact_path", repr(raw))
    pure = PurePosixPath(raw)
    if pure.is_absolute() or ".." in pure.parts or not pure.parts:
        raise CensusPlaceAuthorityRefusal("artifact_path", raw)
    return root.joinpath(*pure.parts)


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


def _read_gzip_rows(path: Path, spec: dict[str, Any], columns: Sequence[str]) -> list[list[str]]:
    raw = _bounded_bytes(path, EXPECTED_BOUNDS["artifact_bytes"], "artifact_size")
    actual_sha = hashlib.sha256(raw).hexdigest()
    if actual_sha != spec["sha256"]:
        raise CensusPlaceAuthorityRefusal("artifact_sha256", str(path))
    if len(raw) < 10 or raw[:2] != b"\x1f\x8b" or struct.unpack("<I", raw[4:8])[0] != 0:
        raise CensusPlaceAuthorityRefusal("artifact_gzip_header", str(path))
    try:
        with gzip.GzipFile(fileobj=io.BytesIO(raw), mode="rb") as compressed:
            decoded = compressed.read(EXPECTED_BOUNDS["artifact_uncompressed_bytes"] + 1)
    except (OSError, EOFError) as error:
        raise CensusPlaceAuthorityRefusal("artifact_gzip", str(path)) from error
    if len(decoded) > EXPECTED_BOUNDS["artifact_uncompressed_bytes"]:
        raise CensusPlaceAuthorityRefusal("artifact_uncompressed_size", str(path))
    try:
        text = decoded.decode("utf-8")
    except UnicodeDecodeError as error:
        raise CensusPlaceAuthorityRefusal("artifact_utf8", str(path)) from error
    csv.field_size_limit(EXPECTED_BOUNDS["csv_field_bytes"])
    try:
        reader = csv.reader(io.StringIO(text, newline=""))
        header = next(reader)
        rows = list(reader)
    except (csv.Error, StopIteration) as error:
        raise CensusPlaceAuthorityRefusal("artifact_csv", str(path)) from error
    if header != list(columns):
        raise CensusPlaceAuthorityRefusal("artifact_columns", str(path))
    if len(rows) != spec["rows"] or len(rows) > EXPECTED_BOUNDS["artifact_rows"]:
        raise CensusPlaceAuthorityRefusal("artifact_rows", str(path))
    if any(len(row) != len(columns) for row in rows):
        raise CensusPlaceAuthorityRefusal("artifact_row_shape", str(path))
    if any("\x00" in value or "\n" in value or "\r" in value for row in rows for value in row):
        raise CensusPlaceAuthorityRefusal("artifact_field", str(path))
    return rows


def _read_geometry_parquet(path: Path, spec: dict[str, Any]) -> list[list[str]]:
    """Hash-prove and decode one bounded deterministic raw-WKB Parquet file."""
    import pyarrow as pa
    import pyarrow.parquet as pq

    raw = _bounded_bytes(path, EXPECTED_BOUNDS["artifact_bytes"], "artifact_size")
    actual_sha = hashlib.sha256(raw).hexdigest()
    if actual_sha != spec["sha256"]:
        raise CensusPlaceAuthorityRefusal("artifact_sha256", str(path))
    expected_schema = pa.schema(
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
        parquet = pq.ParquetFile(pa.BufferReader(raw))
        if (
            parquet.metadata.num_rows != spec["rows"]
            or parquet.metadata.num_rows > EXPECTED_BOUNDS["artifact_rows"]
            or parquet.metadata.num_row_groups != 1
        ):
            raise CensusPlaceAuthorityRefusal("artifact_rows", str(path))
        row_group = parquet.metadata.row_group(0)
        uncompressed_bytes = sum(
            row_group.column(index).total_uncompressed_size
            for index in range(row_group.num_columns)
        )
        if uncompressed_bytes > EXPECTED_BOUNDS["artifact_uncompressed_bytes"]:
            raise CensusPlaceAuthorityRefusal("artifact_uncompressed_size", str(path))
        if any(
            row_group.column(index).compression != "ZSTD" for index in range(row_group.num_columns)
        ):
            raise CensusPlaceAuthorityRefusal("artifact_parquet_compression", str(path))
        table = parquet.read()
    except CensusPlaceAuthorityRefusal:
        raise
    except (pa.ArrowException, OSError, ValueError) as error:
        raise CensusPlaceAuthorityRefusal("artifact_parquet", str(path)) from error
    if table.schema != expected_schema:
        raise CensusPlaceAuthorityRefusal("artifact_columns", str(path))
    values = table.to_pydict()
    rows: list[list[str]] = []
    for index in range(table.num_rows):
        geoid = values["place_geoid"][index]
        wkb = values["geometry_wkb"][index]
        aland = values["aland_m2"][index]
        awater = values["awater_m2"][index]
        latitude = values["internal_point_lat"][index]
        longitude = values["internal_point_lon"][index]
        if (
            not isinstance(geoid, str)
            or not isinstance(wkb, bytes)
            or isinstance(aland, bool)
            or not isinstance(aland, int)
            or isinstance(awater, bool)
            or not isinstance(awater, int)
            or not isinstance(latitude, str)
            or not isinstance(longitude, str)
        ):
            raise CensusPlaceAuthorityRefusal("artifact_parquet_value", f"row {index}")
        rows.append(
            [
                geoid,
                wkb.hex(),
                str(aland),
                str(awater),
                latitude,
                longitude,
            ]
        )
    return rows


def _verify_identity_rows(rows: list[list[str]]) -> None:
    keys = [row[0] for row in rows]
    if keys != sorted(keys) or len(keys) != len(set(keys)):
        raise CensusPlaceAuthorityRefusal("identity_order", repr(keys[:3]))
    for row in rows:
        (
            geoid,
            state_fips,
            place_fips,
            place_ns,
            name,
            name_lsad,
            lsad,
            class_fp,
            pcicbsa,
            mtfcc,
            funcstat,
        ) = row
        if (
            ASCII_DIGITS.fullmatch(geoid) is None
            or len(geoid) != 7
            or state_fips != "26"
            or len(place_fips) != 5
            or geoid != state_fips + place_fips
            or ASCII_DIGITS.fullmatch(place_ns) is None
            or len(place_ns) != 8
            or not name
            or not name_lsad
            or len(lsad) != 2
            or len(class_fp) != 2
            or len(pcicbsa) != 1
            or len(mtfcc) != 5
            or len(funcstat) != 1
        ):
            raise CensusPlaceAuthorityRefusal("identity_value", geoid)


def _parse_nonnegative_int(text: str, field: str, geoid: str) -> int:
    if ASCII_DIGITS.fullmatch(text) is None:
        raise CensusPlaceAuthorityRefusal("geometry_integer", f"{geoid}.{field}")
    value = int(text)
    if value > (1 << 63) - 1:
        raise CensusPlaceAuthorityRefusal("geometry_integer", f"{geoid}.{field}")
    return value


def _parse_finite_coordinate(
    text: str, field: str, geoid: str, minimum: float, maximum: float
) -> float:
    try:
        value = float(text)
    except ValueError as error:
        raise CensusPlaceAuthorityRefusal("geometry_coordinate", f"{geoid}.{field}") from error
    if not math.isfinite(value) or not minimum <= value <= maximum:
        raise CensusPlaceAuthorityRefusal("geometry_coordinate", f"{geoid}.{field}")
    return value


def _verify_geometry_rows(rows: list[list[str]], expected_extent: dict[str, float]) -> None:
    keys = [row[0] for row in rows]
    if keys != sorted(keys) or len(keys) != len(set(keys)):
        raise CensusPlaceAuthorityRefusal("geometry_order", repr(keys[:3]))
    min_lon = math.inf
    min_lat = math.inf
    max_lon = -math.inf
    max_lat = -math.inf
    for row in rows:
        geoid, wkb_hex, aland, awater, latitude, longitude = row
        if len(geoid) != 7 or not wkb_hex or wkb_hex != wkb_hex.lower():
            raise CensusPlaceAuthorityRefusal("geometry_wkb", geoid)
        try:
            wkb_bytes = bytes.fromhex(wkb_hex)
            geometry = from_wkb(wkb_bytes)
        except (GEOSException, ValueError, TypeError) as error:
            raise CensusPlaceAuthorityRefusal("geometry_wkb", geoid) from error
        if (
            len(wkb_bytes) < 5
            or wkb_bytes[0] != 1
            or struct.unpack("<I", wkb_bytes[1:5])[0] not in {3, 6}
        ):
            raise CensusPlaceAuthorityRefusal("geometry_wkb_encoding", geoid)
        if geometry is None:
            raise CensusPlaceAuthorityRefusal("geometry_wkb", geoid)
        if geometry.is_empty:
            raise CensusPlaceAuthorityRefusal("geometry_empty", geoid)
        if geometry.geom_type not in {"Polygon", "MultiPolygon"}:
            raise CensusPlaceAuthorityRefusal("geometry_type", geoid)
        if not geometry.is_valid:
            raise CensusPlaceAuthorityRefusal("geometry_invalid", geoid)
        canonical_wkb = to_wkb(
            geometry,
            hex=True,
            byte_order=1,
            output_dimension=2,
            include_srid=False,
            flavor="iso",
        )
        if not isinstance(canonical_wkb, str) or canonical_wkb.lower() != wkb_hex:
            raise CensusPlaceAuthorityRefusal("geometry_wkb_encoding", geoid)
        coordinates = get_coordinates(geometry, include_z=False)
        if len(coordinates) == 0 or not all(
            math.isfinite(float(value)) for coordinate in coordinates for value in coordinate
        ):
            raise CensusPlaceAuthorityRefusal("geometry_coordinate", geoid)
        _parse_nonnegative_int(aland, "aland_m2", geoid)
        _parse_nonnegative_int(awater, "awater_m2", geoid)
        _parse_finite_coordinate(latitude, "internal_point_lat", geoid, -90.0, 90.0)
        _parse_finite_coordinate(longitude, "internal_point_lon", geoid, -180.0, 180.0)
        bounds = geometry.bounds
        min_lon = min(min_lon, bounds[0])
        min_lat = min(min_lat, bounds[1])
        max_lon = max(max_lon, bounds[2])
        max_lat = max(max_lat, bounds[3])
    actual = (min_lon, min_lat, max_lon, max_lat)
    expected = tuple(
        float(expected_extent[key]) for key in ("min_lon", "min_lat", "max_lon", "max_lat")
    )
    if any(
        not math.isclose(left, right, rel_tol=0.0, abs_tol=1e-12)
        for left, right in zip(actual, expected, strict=True)
    ):
        raise CensusPlaceAuthorityRefusal("geometry_extent", repr(actual))


def verify_artifacts(
    contract: dict[str, Any], root: Path
) -> tuple[list[list[str]], list[list[str]]]:
    """Verify both checked artifacts, their semantics, and their one-to-one keyset."""
    identity_spec = _artifact_spec(contract, "identity_artifact")
    geometry_spec = _artifact_spec(contract, "geometry_artifact")
    identity_path = _artifact_path(root, identity_spec["path"])
    geometry_path = _artifact_path(root, geometry_spec["path"])
    identities = _read_gzip_rows(identity_path, identity_spec, IDENTITY_COLUMNS)
    geometries = _read_geometry_parquet(geometry_path, geometry_spec)
    _verify_identity_rows(identities)
    _verify_geometry_rows(geometries, contract["extent"])
    if [row[0] for row in identities] != [row[0] for row in geometries]:
        raise CensusPlaceAuthorityRefusal("artifact_keyset", "identity != geometry")
    identity_semantic = _semantic_sha256(
        b"babylon.census-place-identity.v1", IDENTITY_COLUMNS, identities
    )
    geometry_semantic = _semantic_sha256(
        b"babylon.census-place-geometry.v1",
        GEOMETRY_COLUMNS,
        [[row[0], row[1], int(row[2]), int(row[3]), row[4], row[5]] for row in geometries],
    )
    if identity_semantic != identity_spec["semantic_sha256"]:
        raise CensusPlaceAuthorityRefusal("identity_semantic_sha256", identity_semantic)
    if geometry_semantic != geometry_spec["semantic_sha256"]:
        raise CensusPlaceAuthorityRefusal("geometry_semantic_sha256", geometry_semantic)
    return identities, geometries


def verify_artifact_manifest(contract: dict[str, Any], path: Path) -> None:
    """Tripwire the hand-maintained second-order registry entries."""
    raw = _bounded_bytes(path, 262_144, "artifact_manifest_size")
    try:
        document = _safe_load_unique(raw)
    except yaml.YAMLError as error:
        raise CensusPlaceAuthorityRefusal("artifact_manifest_yaml", str(path)) from error
    if not isinstance(document, dict) or not isinstance(document.get("artifacts"), list):
        raise CensusPlaceAuthorityRefusal("artifact_manifest_shape", str(path))
    by_name: dict[str, Any] = {}
    for entry in document["artifacts"]:
        if not isinstance(entry, dict) or not isinstance(entry.get("name"), str):
            raise CensusPlaceAuthorityRefusal("artifact_manifest_entry", repr(entry))
        if entry["name"] in by_name:
            raise CensusPlaceAuthorityRefusal("artifact_manifest_duplicate", entry["name"])
        by_name[entry["name"]] = entry
    for key, (name, _, _) in EXPECTED_ARTIFACTS.items():
        spec = contract[key]
        entry = by_name.get(name)
        if not isinstance(entry, dict) or entry != {
            "name": name,
            "format": spec["format"],
            "source_table": None,
            "generator": "tools/make_census_place_artifacts.py",
            "mode": "register",
            "rows": spec["rows"],
            "sha256": spec["sha256"],
            "home": spec["path"],
            "material_relation": entry.get("material_relation")
            if isinstance(entry, dict)
            else None,
        }:
            raise CensusPlaceAuthorityRefusal("artifact_manifest_pin", name)
        material_relation = entry["material_relation"]
        if not isinstance(material_relation, str) or "PER-276" not in material_relation:
            raise CensusPlaceAuthorityRefusal("artifact_manifest_relation", name)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--contract",
        type=Path,
        default=Path("contracts/census_place_authority_v1.yaml"),
    )
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    args = parser.parse_args(argv)
    contract = load_contract(args.contract)
    verify_contract(contract)
    verify_source_manifest(contract, args.repo_root / EXPECTED_SOURCE_MANIFEST)
    identities, geometries = verify_artifacts(contract, args.repo_root)
    verify_artifact_manifest(contract, args.repo_root / "data-artifacts.yaml")
    print(
        "CensusPlaceAuthorityV1 verified: "
        f"{len(identities)} identities, {len(geometries)} geometries"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
