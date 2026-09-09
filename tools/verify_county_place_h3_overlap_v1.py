#!/usr/bin/env python3
"""Independently verify PER-277's Michigan county/place/H3 land authority."""

from __future__ import annotations

import argparse
import csv
import gzip
import hashlib
import io
import json
import math
import re
import sys
from collections import Counter, defaultdict
from collections.abc import Iterable, Sequence
from pathlib import Path, PurePosixPath
from typing import Any, Final, cast

import yaml
from yaml.constructor import ConstructorError
from yaml.nodes import MappingNode

EXPECTED_META: Final = {
    "contract": "CountyPlaceH3OverlapV1",
    "version": 1,
    "issue": "PER-277",
    "parent": "PER-21",
}
EXPECTED_CLASSIFICATIONS: Final = {
    "source_url_bytes_vintage_schema_crs_and_members": "Observed",
    "predecessor_artifact_bytes_and_identity": "Observed",
    "land_area_measures": "Derived",
    "semantic_digests_and_fixed_point_shares": "Derived",
    "projection_overlay_ordering_quantization_and_absences": "Designed",
}
EXPECTED_BOUNDS: Final = {
    "contract_bytes": 131_072,
    "source_archive_bytes": 104_857_600,
    "source_uncompressed_bytes": 268_435_456,
    "artifact_bytes": 1_048_576,
    "artifact_uncompressed_bytes": 16_777_216,
    "cohort_rows": 65_536,
    "county_rows": 65_536,
    "county_place_rows": 65_536,
}
EXPECTED_SOURCE_MANIFEST: Final = "tools/county_place_h3_overlap_v1_fetch_manifest.json"
EXPECTED_SOURCE_URL: Final = (
    "https://www2.census.gov/geo/tiger/TIGER2023/COUNTY/tl_2023_us_county.zip"
)
EXPECTED_SOURCE_DEST: Final = "tiger/county/tl_2023_us_county.zip"
EXPECTED_SOURCE_SHA256: Final = "692e12c30c83adcaabdbac0d3954fafa55e1c89a24b36d95e72e02dff938652e"
EXPECTED_COUNTY_IDENTITY_FIXTURE: Final = {
    "path": "contracts/fixtures/census_county_geoids_mi_2023.txt",
    "format": "newline-delimited-ascii",
    "rows": 83,
    "bytes": 498,
    "sha256": "216b281d01b7d457c6d510af24c6026d6d28d918cdca8f6b2ab39975b264faa5",
    "semantic_sha256": "b0357679c075cc3e56a0709872baa92e67bf42b668871e544282cbab7509bdb6",
}
EXPECTED_H3_IDENTITY_FIXTURE: Final = {
    "path": "contracts/fixtures/h3_res7_land_mask_ids_v1.txt.gz",
    "format": "newline-delimited-lowercase-h3-gzip",
    "rows": 45_572,
    "bytes": 111_386,
    "uncompressed_bytes": 729_152,
    "sha256": "de0ae67607aeb2ba80f5b64092d6c3c98b9020f9554685af135d52e33ea6696f",
    "semantic_sha256": "3b6eda7d8b29c6208157ff531f03750c44889535d37e0767c5bed17209a30f5d",
}
EXPECTED_SOURCE_COLUMNS: Final = (
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
EXPECTED_SOURCE_MEMBERS: Final = (
    "tl_2023_us_county.cpg",
    "tl_2023_us_county.dbf",
    "tl_2023_us_county.prj",
    "tl_2023_us_county.shp",
    "tl_2023_us_county.shp.ea.iso.xml",
    "tl_2023_us_county.shp.iso.xml",
    "tl_2023_us_county.shx",
)
EXPECTED_AREAWATER_COLUMNS: Final = (
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
EXPECTED_PREDECESSOR_FILES: Final = {
    "h3_contract": (
        "contracts/h3_estate_contract_v1.yaml",
        "1ea546e572c499424086d52bc62820626328c8336c38d57b6053c337c0b943b5",
    ),
    "h3_vectors": (
        "rust/crates/babylon-kernel/tests/fixtures/h3_cell_id_vectors_v1.txt",
        "079fd93db1cfb9b2aef79b7b32476f737689a91c194de912864bb3849e578487",
    ),
    "place_contract": (
        "contracts/census_place_authority_v1.yaml",
        "0bced499b9144e51d48bc2356260448bc09ab56b64035ab94c98a7287b462102",
    ),
    "place_manifest": (
        "tools/census_place_authority_v1_fetch_manifest.json",
        "223c1419dd9e7bd855efd8fb07b87199ce4ec88bd549e369890edbb1dea71456",
    ),
    "place_identity": (
        "src/babylon/data/reference/spatial/census_place_identity_mi_2023.csv.gz",
        "cb864b4f6f43902bb821e84fe9a4055a9039e0a74d8b8399f209ae6ed26a8be7",
    ),
    "place_geometry": (
        "src/babylon/data/reference/spatial/census_place_geometry_mi_2023.parquet",
        "cea5b0ada40b75ae2f6996bef7aa4aeb8d13b36ce5bc41d4334da1e8bf17b737",
    ),
}
EXPECTED_COUNTY_COLUMNS: Final = (
    "cell_id",
    "county_fips",
    "land_area_m2",
)
EXPECTED_PLACE_COLUMNS: Final = (
    "cell_id",
    "county_fips",
    "place_geoid",
    "place_land_area_m2",
    "cell_mi_land_area_m2",
    "place_land_area_share_ppb",
)
EXPECTED_ARTIFACTS: Final = {
    "county_cell_artifact": (
        "census_county_h3_land_overlap_mi_2023",
        "src/babylon/data/reference/spatial/census_county_h3_land_overlap_mi_2023.parquet",
        EXPECTED_COUNTY_COLUMNS,
    ),
    "county_place_cell_artifact": (
        "census_county_place_h3_land_overlap_mi_2023",
        "src/babylon/data/reference/spatial/census_county_place_h3_land_overlap_mi_2023.parquet",
        EXPECTED_PLACE_COLUMNS,
    ),
}
EXPECTED_ABSENCES: Final = {
    "dominant_county_owner",
    "synthetic_place",
    "normalized_place_share",
    "postgresql_schema",
    "runtime_reader",
    "runtime_writer",
    "writer_cutover",
}
EXPECTED_LINEAGE: Final = {
    "resolves_historical_gap": "county_place_h3_overlap",
    "completes_data_substrate_for": "PER-21",
    "database_dependency_owner": "PER-20",
    "authority_boundary": "checked-artifacts-only-no-postgresql-or-runtime-authority",
}
EXPECTED_EVIDENCE_KEYS: Final = {
    "positive_land_cells",
    "cohort_absent_cells",
    "cross_county_cells",
    "maximum_counties_per_cell",
    "places_crossing_counties",
    "maximum_counties_per_place",
    "land_cells_without_place",
    "total_mi_land_area_m2",
    "total_place_land_area_m2",
    "maximum_share_sum_ppb",
    "maximum_place_share_ppb",
    "minimum_place_share_ppb",
}
SHA256 = re.compile(r"^[0-9a-f]{64}$")
COUNTY_FIPS = re.compile(r"^26[0-9]{3}$")
PLACE_GEOID = re.compile(r"^26[0-9]{5}$")
SHARE_SCALE: Final = 1_000_000_000


class CountyPlaceH3OverlapRefusal(ValueError):
    """One typed refusal from the independent PER-277 verifier."""

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
        raise CountyPlaceH3OverlapRefusal("file_read", str(path)) from error
    if size > maximum:
        raise CountyPlaceH3OverlapRefusal(code, f"{path}: {size}")
    try:
        return path.read_bytes()
    except OSError as error:
        raise CountyPlaceH3OverlapRefusal("file_read", str(path)) from error


def _sha256_bytes(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def _require_sha256(value: object, detail: str) -> str:
    if not isinstance(value, str) or SHA256.fullmatch(value) is None:
        raise CountyPlaceH3OverlapRefusal("contract_sha256", detail)
    return value


def _verify_file(path: Path, digest: object, detail: str) -> None:
    expected = _require_sha256(digest, detail)
    raw = _bounded_bytes(path, EXPECTED_BOUNDS["artifact_uncompressed_bytes"], "file_size")
    if _sha256_bytes(raw) != expected:
        raise CountyPlaceH3OverlapRefusal("predecessor_sha256", detail)


def load_contract(path: Path) -> dict[str, Any]:
    """Load one bounded YAML contract mapping with duplicate-key refusal."""
    raw = _bounded_bytes(path, EXPECTED_BOUNDS["contract_bytes"], "contract_size")
    try:
        document = _safe_load_unique(raw)
    except yaml.YAMLError as error:
        raise CountyPlaceH3OverlapRefusal("invalid_contract", str(path)) from error
    if not isinstance(document, dict):
        raise CountyPlaceH3OverlapRefusal("invalid_contract", "root mapping")
    return document


def _artifact_spec(contract: dict[str, Any], key: str) -> dict[str, Any]:
    value = contract.get(key)
    if not isinstance(value, dict):
        raise CountyPlaceH3OverlapRefusal("contract_shape", key)
    return value


def _mapping(value: object, detail: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise CountyPlaceH3OverlapRefusal("contract_shape", detail)
    return value


def verify_contract(contract: dict[str, Any]) -> None:
    """Verify the closed contract shape and its non-runtime authority boundary."""
    if set(contract) != {
        "meta",
        "classifications",
        "bounds",
        "source",
        "areawater",
        "predecessors",
        "toolchain",
        "geometry_method",
        "county_cell_artifact",
        "county_place_cell_artifact",
        "evidence",
        "declared_absences",
        "lineage",
    }:
        raise CountyPlaceH3OverlapRefusal("contract_shape", "root")
    if (
        contract.get("meta") != EXPECTED_META
        or contract.get("classifications") != EXPECTED_CLASSIFICATIONS
        or contract.get("bounds") != EXPECTED_BOUNDS
    ):
        raise CountyPlaceH3OverlapRefusal("contract_shape", "meta_classifications_or_bounds")

    source = _mapping(contract.get("source"), "source")
    expected_source_keys = {
        "manifest",
        "manifest_sha256",
        "url",
        "dest",
        "sha256",
        "bytes",
        "vintage",
        "state_fips",
        "input_crs",
        "national_rows",
        "michigan_rows",
        "michigan_extent",
        "identity_fixture",
        "columns",
        "zip_members",
    }
    if set(source) != expected_source_keys or (
        source.get("manifest") != EXPECTED_SOURCE_MANIFEST
        or source.get("url") != EXPECTED_SOURCE_URL
        or source.get("dest") != EXPECTED_SOURCE_DEST
        or source.get("sha256") != EXPECTED_SOURCE_SHA256
        or source.get("bytes") != 83_451_409
        or source.get("vintage") != 2023
        or source.get("state_fips") != "26"
        or source.get("input_crs") != "EPSG:4269"
        or source.get("national_rows") != 3_235
        or source.get("michigan_rows") != 83
    ):
        raise CountyPlaceH3OverlapRefusal("contract_shape", "source")
    _require_sha256(source.get("manifest_sha256"), "source.manifest_sha256")
    _require_sha256(source.get("sha256"), "source.sha256")
    if source.get("identity_fixture") != EXPECTED_COUNTY_IDENTITY_FIXTURE:
        raise CountyPlaceH3OverlapRefusal("contract_shape", "source.identity_fixture")
    if (
        isinstance(source.get("bytes"), bool)
        or not isinstance(source.get("bytes"), int)
        or not 0 < source["bytes"] <= EXPECTED_BOUNDS["source_archive_bytes"]
        or not isinstance(source.get("columns"), list)
        or source["columns"] != list(EXPECTED_SOURCE_COLUMNS)
        or not isinstance(source.get("zip_members"), list)
        or source["zip_members"] != list(EXPECTED_SOURCE_MEMBERS)
        or len(source["zip_members"]) != len(set(source["zip_members"]))
        or not isinstance(source.get("michigan_extent"), list)
        or len(source["michigan_extent"]) != 4
        or any(
            isinstance(value, bool)
            or not isinstance(value, (int, float))
            or not math.isfinite(value)
            for value in source["michigan_extent"]
        )
    ):
        raise CountyPlaceH3OverlapRefusal("contract_shape", "source values")

    areawater = _mapping(contract.get("areawater"), "areawater")
    if set(areawater) != {
        "manifest",
        "manifest_sha256",
        "archives",
        "input_crs",
        "url_template",
        "dest_template",
        "columns",
    } or (
        areawater.get("manifest") != "tools/phase0d/fetch_manifest.json"
        or areawater.get("archives") != 83
        or areawater.get("input_crs") != "EPSG:4269"
        or "{county_fips}" not in str(areawater.get("url_template"))
        or "{county_fips}" not in str(areawater.get("dest_template"))
        or not isinstance(areawater.get("columns"), list)
        or areawater.get("columns") != list(EXPECTED_AREAWATER_COLUMNS)
    ):
        raise CountyPlaceH3OverlapRefusal("contract_shape", "areawater")
    _require_sha256(areawater.get("manifest_sha256"), "areawater.manifest_sha256")

    predecessors = _mapping(contract.get("predecessors"), "predecessors")
    if set(predecessors) != {
        "h3_contract",
        "h3_vectors",
        "h3_cohort",
        "place_contract",
        "place_manifest",
        "place_identity",
        "place_geometry",
    }:
        raise CountyPlaceH3OverlapRefusal("contract_shape", "predecessors")
    for key, (expected_path, expected_sha256) in EXPECTED_PREDECESSOR_FILES.items():
        spec = _mapping(predecessors.get(key), f"predecessors.{key}")
        if spec.get("path") != expected_path or spec.get("sha256") != expected_sha256:
            raise CountyPlaceH3OverlapRefusal("contract_shape", f"predecessors.{key}")
        _require_sha256(spec.get("sha256"), f"predecessors.{key}.sha256")
    _require_sha256(
        _mapping(predecessors["h3_contract"], "h3_contract").get("canonical_sha256"),
        "predecessors.h3_contract.canonical_sha256",
    )
    cohort = _mapping(predecessors.get("h3_cohort"), "predecessors.h3_cohort")
    if (
        set(cohort)
        != {
            "url",
            "bytes",
            "sha256",
            "rows",
            "identity_column",
            "ignored_measure_columns",
            "identity_fixture",
        }
        or cohort.get("identity_column") != "h3_index"
        or cohort.get("ignored_measure_columns") != ["county_fips", "land_fraction"]
        or cohort.get("rows") != 45_572
        or cohort.get("bytes") != 295_194
        or cohort.get("sha256")
        != "4e6caba297f0111a9ec93d948a83543bb9f7179361fe5dd318bb8a98a5be5194"
        or cohort.get("identity_fixture") != EXPECTED_H3_IDENTITY_FIXTURE
    ):
        raise CountyPlaceH3OverlapRefusal("contract_shape", "predecessors.h3_cohort")
    _require_sha256(cohort.get("sha256"), "predecessors.h3_cohort.sha256")

    toolchain = _mapping(contract.get("toolchain"), "toolchain")
    if toolchain != {
        "geopandas": "1.1.4",
        "h3": "4.5.0",
        "pyarrow": "25.0.0",
        "pyogrio": "0.13.0",
        "pyproj": "3.7.2",
        "shapely": "2.1.2",
        "geos": "3.13.1",
        "proj": "9.5.1",
    }:
        raise CountyPlaceH3OverlapRefusal("contract_shape", "toolchain")
    method = _mapping(contract.get("geometry_method"), "geometry_method")
    if method != {
        "working_crs": "EPSG:5070",
        "numeric_model": "pinned-binary64",
        "grid_snap": "none",
        "overlay_order": [
            "project_exact_source_geometries",
            "subtract_official_county_areawater",
            "intersect_h3_with_county_land",
            "intersect_place_with_county_land_and_h3",
        ],
        "dimensional_policy": "retain-polygonal-components-only",
        "area_quantization": "floor-positive-whole-square-metres",
        "share_formula": "floor(place_land_area_m2*1000000000/cell_mi_land_area_m2)",
        "denominator": "sum-positive-quantized-cell-county-land-slices",
        "ordering": {
            "county_cell": "cell_id-county_fips-ascending",
            "county_place_cell": "cell_id-county_fips-place_geoid-ascending",
        },
    }:
        raise CountyPlaceH3OverlapRefusal("contract_shape", "geometry_method")

    for key, (name, path, columns) in EXPECTED_ARTIFACTS.items():
        spec = _artifact_spec(contract, key)
        row_bound = (
            EXPECTED_BOUNDS["county_rows"]
            if key == "county_cell_artifact"
            else EXPECTED_BOUNDS["county_place_rows"]
        )
        if set(spec) != {
            "name",
            "path",
            "format",
            "compression",
            "rows",
            "bytes",
            "columns",
            "sha256",
            "semantic_sha256",
        } or (
            spec.get("name") != name
            or spec.get("path") != path
            or spec.get("format") != "parquet"
            or spec.get("compression") != "parquet-zstd-level-22"
            or spec.get("columns") != list(columns)
            or isinstance(spec.get("rows"), bool)
            or not isinstance(spec.get("rows"), int)
            or not 0 < spec["rows"] <= row_bound
            or isinstance(spec.get("bytes"), bool)
            or not isinstance(spec.get("bytes"), int)
            or not 0 < spec["bytes"] <= EXPECTED_BOUNDS["artifact_bytes"]
        ):
            raise CountyPlaceH3OverlapRefusal("contract_shape", key)
        _require_sha256(spec.get("sha256"), f"{key}.sha256")
        _require_sha256(spec.get("semantic_sha256"), f"{key}.semantic_sha256")

    evidence = _mapping(contract.get("evidence"), "evidence")
    if set(evidence) != EXPECTED_EVIDENCE_KEYS or any(
        isinstance(value, bool) or not isinstance(value, int) or value < 0
        for value in evidence.values()
    ):
        raise CountyPlaceH3OverlapRefusal("contract_shape", "evidence")
    absences = contract.get("declared_absences")
    if (
        not isinstance(absences, list)
        or set(absences) != EXPECTED_ABSENCES
        or len(absences) != len(EXPECTED_ABSENCES)
        or contract.get("lineage") != EXPECTED_LINEAGE
    ):
        raise CountyPlaceH3OverlapRefusal("contract_shape", "authority boundary")


def _unique_json_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise CountyPlaceH3OverlapRefusal("source_manifest_duplicate_key", key)
        result[key] = value
    return result


def verify_source_manifest(contract: dict[str, Any], path: Path) -> None:
    """Verify the exact dedicated one-source acquisition manifest."""
    raw = _bounded_bytes(path, EXPECTED_BOUNDS["contract_bytes"], "source_manifest_size")
    source = contract["source"]
    if _sha256_bytes(raw) != source["manifest_sha256"]:
        raise CountyPlaceH3OverlapRefusal("source_manifest_sha256", str(path))
    try:
        document = json.loads(raw, object_pairs_hook=_unique_json_object)
    except (json.JSONDecodeError, UnicodeDecodeError) as error:
        raise CountyPlaceH3OverlapRefusal("source_manifest_json", str(path)) from error
    expected = {
        "contract": "CountyPlaceH3OverlapV1",
        "version": 1,
        "entries": [{"url": source["url"], "dest": source["dest"], "sha256": source["sha256"]}],
    }
    if document != expected:
        raise CountyPlaceH3OverlapRefusal("source_manifest_pin", str(path))


def _repo_path(root: Path, raw: object, detail: str) -> Path:
    if not isinstance(raw, str):
        raise CountyPlaceH3OverlapRefusal("contract_path", detail)
    pure = PurePosixPath(raw)
    if pure.is_absolute() or ".." in pure.parts or not pure.parts:
        raise CountyPlaceH3OverlapRefusal("contract_path", detail)
    return root.joinpath(*pure.parts)


def verify_predecessors(contract: dict[str, Any], root: Path) -> None:
    """Verify checked predecessor bytes and execute both predecessor contracts."""
    predecessors = contract["predecessors"]
    for key in (
        "h3_contract",
        "h3_vectors",
        "place_contract",
        "place_manifest",
        "place_identity",
        "place_geometry",
    ):
        spec = predecessors[key]
        _verify_file(_repo_path(root, spec["path"], key), spec["sha256"], key)
    areawater = contract["areawater"]
    _verify_file(
        _repo_path(root, areawater["manifest"], "areawater.manifest"),
        areawater["manifest_sha256"],
        "areawater.manifest",
    )

    from verify_census_place_authority_v1 import (
        load_contract as load_place_contract,
    )
    from verify_census_place_authority_v1 import (
        verify_artifacts as verify_place_artifacts,
    )
    from verify_census_place_authority_v1 import (
        verify_contract as verify_place_contract,
    )
    from verify_census_place_authority_v1 import (
        verify_source_manifest as verify_place_manifest,
    )
    from verify_h3_estate_contract_v1 import (
        canonical_contract_digest,
    )
    from verify_h3_estate_contract_v1 import (
        load_contract as load_h3_contract,
    )
    from verify_h3_estate_contract_v1 import (
        verify_contract as verify_h3_contract,
    )

    h3_path = _repo_path(root, predecessors["h3_contract"]["path"], "h3_contract")
    h3_contract = load_h3_contract(h3_path)
    verify_h3_contract(h3_contract, root)
    if canonical_contract_digest(h3_contract) != predecessors["h3_contract"]["canonical_sha256"]:
        raise CountyPlaceH3OverlapRefusal("predecessor_semantic_sha256", "h3_contract")
    place_path = _repo_path(root, predecessors["place_contract"]["path"], "place_contract")
    place_contract = load_place_contract(place_path)
    verify_place_contract(place_contract)
    verify_place_manifest(
        place_contract,
        _repo_path(root, predecessors["place_manifest"]["path"], "place_manifest"),
    )
    verify_place_artifacts(place_contract, root)


def _checked_int(value: object, code: str, detail: str, maximum: int) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or not 0 < value <= maximum:
        raise CountyPlaceH3OverlapRefusal(code, detail)
    return value


def verify_county_records(
    records: Iterable[Sequence[object]],
    cohort_ids: set[int],
    county_geoids: set[str],
) -> dict[int, int]:
    """Verify ordered positive cell/county land rows and return denominators."""
    import h3

    rows = list(records)
    if not 0 < len(rows) <= EXPECTED_BOUNDS["county_rows"]:
        raise CountyPlaceH3OverlapRefusal("county_rows", str(len(rows)))
    normalized: list[tuple[int, str, int]] = []
    denominator: defaultdict[int, int] = defaultdict(int)
    for index, row in enumerate(rows):
        if len(row) != 3:
            raise CountyPlaceH3OverlapRefusal("county_row_shape", str(index))
        cell_id = _checked_int(row[0], "county_cell", str(index), (1 << 63) - 1)
        county_fips = row[1]
        land_area = _checked_int(row[2], "county_measure", str(index), (1 << 64) - 1)
        if cell_id not in cohort_ids:
            raise CountyPlaceH3OverlapRefusal("county_cell_unknown", str(cell_id))
        cell = h3.int_to_str(cell_id)
        if not h3.is_valid_cell(cell) or h3.get_resolution(cell) != 7:
            raise CountyPlaceH3OverlapRefusal("county_cell", str(cell_id))
        if not isinstance(county_fips, str) or COUNTY_FIPS.fullmatch(county_fips) is None:
            raise CountyPlaceH3OverlapRefusal("county_fips", repr(county_fips))
        if county_fips not in county_geoids:
            raise CountyPlaceH3OverlapRefusal("county_fips_unknown", county_fips)
        normalized.append((cell_id, county_fips, land_area))
        denominator[cell_id] += land_area
        if denominator[cell_id] > (1 << 64) - 1:
            raise CountyPlaceH3OverlapRefusal("county_measure", str(cell_id))
    keys = [(row[0], row[1]) for row in normalized]
    if keys != sorted(keys) or len(keys) != len(set(keys)):
        raise CountyPlaceH3OverlapRefusal("county_order", repr(keys[:3]))
    return dict(denominator)


def verify_place_records(
    records: Iterable[Sequence[object]],
    county_records: Iterable[Sequence[object]],
    cohort_ids: set[int],
    county_geoids: set[str],
    place_geoids: set[str],
) -> None:
    """Verify exact fixed-point place shares and both conservation bounds."""
    county_rows = list(county_records)
    denominator = verify_county_records(county_rows, cohort_ids, county_geoids)
    county_limit = {
        (cast(int, row[0]), cast(str, row[1])): cast(int, row[2]) for row in county_rows
    }
    rows = list(records)
    if not 0 < len(rows) <= EXPECTED_BOUNDS["county_place_rows"]:
        raise CountyPlaceH3OverlapRefusal("place_rows", str(len(rows)))
    normalized: list[tuple[int, str, str, int, int, int]] = []
    county_used: defaultdict[tuple[int, str], int] = defaultdict(int)
    cell_used: defaultdict[int, int] = defaultdict(int)
    for index, row in enumerate(rows):
        if len(row) != 6:
            raise CountyPlaceH3OverlapRefusal("place_row_shape", str(index))
        cell_id = _checked_int(row[0], "place_cell", str(index), (1 << 63) - 1)
        county_fips = row[1]
        place_geoid = row[2]
        numerator = _checked_int(row[3], "place_measure", str(index), (1 << 64) - 1)
        cell_denominator = _checked_int(row[4], "place_denominator", str(index), (1 << 64) - 1)
        share = _checked_int(row[5], "place_share", str(index), SHARE_SCALE)
        if cell_id not in cohort_ids or cell_id not in denominator:
            raise CountyPlaceH3OverlapRefusal("place_cell_unknown", str(cell_id))
        if (
            not isinstance(county_fips, str)
            or COUNTY_FIPS.fullmatch(county_fips) is None
            or (cell_id, county_fips) not in county_limit
        ):
            raise CountyPlaceH3OverlapRefusal("place_county", repr(county_fips))
        if (
            not isinstance(place_geoid, str)
            or PLACE_GEOID.fullmatch(place_geoid) is None
            or place_geoid not in place_geoids
        ):
            raise CountyPlaceH3OverlapRefusal("place_geoid", repr(place_geoid))
        if cell_denominator != denominator[cell_id]:
            raise CountyPlaceH3OverlapRefusal("place_denominator", str(cell_id))
        county_used[(cell_id, county_fips)] += numerator
        cell_used[cell_id] += numerator
        if (
            county_used[(cell_id, county_fips)] > county_limit[(cell_id, county_fips)]
            or cell_used[cell_id] > cell_denominator
        ):
            raise CountyPlaceH3OverlapRefusal("place_conservation", str(index))
        expected_share = numerator * SHARE_SCALE // cell_denominator
        if share != expected_share:
            raise CountyPlaceH3OverlapRefusal("place_share", str(index))
        normalized.append((cell_id, county_fips, place_geoid, numerator, cell_denominator, share))
    keys = [(row[0], row[1], row[2]) for row in normalized]
    if keys != sorted(keys) or len(keys) != len(set(keys)):
        raise CountyPlaceH3OverlapRefusal("place_order", repr(keys[:3]))


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


def _artifact_path(root: Path, raw: object) -> Path:
    if not isinstance(raw, str):
        raise CountyPlaceH3OverlapRefusal("artifact_path", repr(raw))
    path = Path(raw)
    if path.is_absolute():
        return path
    pure = PurePosixPath(raw)
    if ".." in pure.parts or not pure.parts:
        raise CountyPlaceH3OverlapRefusal("artifact_path", raw)
    return root.joinpath(*pure.parts)


def _read_parquet(
    path: Path, spec: dict[str, Any], columns: Sequence[str]
) -> list[tuple[object, ...]]:
    import pyarrow as pa
    import pyarrow.parquet as pq

    raw = _bounded_bytes(path, EXPECTED_BOUNDS["artifact_bytes"], "artifact_size")
    if _sha256_bytes(raw) != spec["sha256"]:
        raise CountyPlaceH3OverlapRefusal("artifact_sha256", str(path))
    county_schema = pa.schema(
        [
            pa.field("cell_id", pa.int64(), nullable=False),
            pa.field("county_fips", pa.string(), nullable=False),
            pa.field("land_area_m2", pa.uint64(), nullable=False),
        ]
    )
    place_schema = pa.schema(
        [
            pa.field("cell_id", pa.int64(), nullable=False),
            pa.field("county_fips", pa.string(), nullable=False),
            pa.field("place_geoid", pa.string(), nullable=False),
            pa.field("place_land_area_m2", pa.uint64(), nullable=False),
            pa.field("cell_mi_land_area_m2", pa.uint64(), nullable=False),
            pa.field("place_land_area_share_ppb", pa.uint32(), nullable=False),
        ]
    )
    expected_schema = county_schema if tuple(columns) == EXPECTED_COUNTY_COLUMNS else place_schema
    try:
        parquet = pq.ParquetFile(pa.BufferReader(raw))
        if parquet.metadata.num_rows != spec["rows"] or parquet.metadata.num_row_groups != 1:
            raise CountyPlaceH3OverlapRefusal("artifact_rows", str(path))
        row_group = parquet.metadata.row_group(0)
        uncompressed = sum(
            row_group.column(index).total_uncompressed_size
            for index in range(row_group.num_columns)
        )
        if uncompressed > EXPECTED_BOUNDS["artifact_uncompressed_bytes"]:
            raise CountyPlaceH3OverlapRefusal("artifact_uncompressed_size", str(path))
        if any(
            row_group.column(index).compression != "ZSTD" for index in range(row_group.num_columns)
        ):
            raise CountyPlaceH3OverlapRefusal("artifact_compression", str(path))
        if len(raw) != spec["bytes"]:
            raise CountyPlaceH3OverlapRefusal("artifact_bytes", str(path))
        table = parquet.read()
    except CountyPlaceH3OverlapRefusal:
        raise
    except (OSError, ValueError, pa.ArrowException) as error:
        raise CountyPlaceH3OverlapRefusal("artifact_parquet", str(path)) from error
    if table.schema != expected_schema or table.column_names != list(columns):
        raise CountyPlaceH3OverlapRefusal("artifact_schema", str(path))
    values = table.to_pydict()
    return [tuple(values[column][index] for column in columns) for index in range(table.num_rows)]


def _place_geoids(root: Path, contract: dict[str, Any]) -> set[str]:
    spec = contract["predecessors"]["place_identity"]
    path = _repo_path(root, spec["path"], "place_identity")
    raw = _bounded_bytes(path, EXPECTED_BOUNDS["artifact_bytes"], "place_identity_size")
    if _sha256_bytes(raw) != spec["sha256"]:
        raise CountyPlaceH3OverlapRefusal("predecessor_sha256", "place_identity")
    try:
        decoded = gzip.decompress(raw).decode("utf-8")
        rows = list(csv.DictReader(io.StringIO(decoded, newline="")))
    except (OSError, UnicodeDecodeError, csv.Error) as error:
        raise CountyPlaceH3OverlapRefusal("place_identity_decode", str(path)) from error
    geoids = {str(row.get("place_geoid")) for row in rows}
    if len(rows) != spec["rows"] or len(geoids) != len(rows):
        raise CountyPlaceH3OverlapRefusal("place_identity_rows", str(len(rows)))
    return geoids


def _identity_fixture_values(
    root: Path,
    raw_spec: object,
    expected: dict[str, object],
    *,
    domain: bytes,
    compressed: bool,
    detail: str,
) -> list[str]:
    spec = _mapping(raw_spec, detail)
    if spec != expected:
        raise CountyPlaceH3OverlapRefusal("identity_fixture_contract", detail)
    path = _repo_path(root, spec["path"], detail)
    raw = _bounded_bytes(path, EXPECTED_BOUNDS["artifact_bytes"], "identity_fixture_size")
    if len(raw) != spec["bytes"] or _sha256_bytes(raw) != spec["sha256"]:
        raise CountyPlaceH3OverlapRefusal("identity_fixture_sha256", detail)
    if compressed:
        try:
            with gzip.GzipFile(fileobj=io.BytesIO(raw), mode="rb") as archive:
                decoded = archive.read(EXPECTED_BOUNDS["artifact_uncompressed_bytes"] + 1)
        except (EOFError, OSError) as error:
            raise CountyPlaceH3OverlapRefusal("identity_fixture_decode", detail) from error
        expected_uncompressed = spec.get("uncompressed_bytes")
    else:
        decoded = raw
        expected_uncompressed = spec["bytes"]
    if (
        len(decoded) > EXPECTED_BOUNDS["artifact_uncompressed_bytes"]
        or len(decoded) != expected_uncompressed
        or _sha256_bytes(domain + b"\0" + decoded) != spec["semantic_sha256"]
    ):
        raise CountyPlaceH3OverlapRefusal("identity_fixture_semantic_sha256", detail)
    try:
        text = decoded.decode("ascii")
    except UnicodeDecodeError as error:
        raise CountyPlaceH3OverlapRefusal("identity_fixture_decode", detail) from error
    if not text.endswith("\n") or "\r" in text:
        raise CountyPlaceH3OverlapRefusal("identity_fixture_format", detail)
    values = text.splitlines()
    if len(values) != spec["rows"] or len(values) != len(set(values)) or values != sorted(values):
        raise CountyPlaceH3OverlapRefusal("identity_fixture_rows", detail)
    return values


def load_county_geoids(contract: dict[str, Any], root: Path) -> set[str]:
    """Load the independently pinned official Michigan county allowlist."""
    values = _identity_fixture_values(
        root,
        _mapping(contract.get("source"), "source").get("identity_fixture"),
        EXPECTED_COUNTY_IDENTITY_FIXTURE,
        domain=b"babylon.census-county-geoids-mi-2023.v1",
        compressed=False,
        detail="source.identity_fixture",
    )
    if any(COUNTY_FIPS.fullmatch(value) is None for value in values):
        raise CountyPlaceH3OverlapRefusal("identity_fixture_county", repr(values[:1]))
    return set(values)


def load_h3_cohort_ids(contract: dict[str, Any], root: Path) -> set[int]:
    """Load the exact PER-275 H3 cohort from its independent checked fixture."""
    import h3

    predecessors = _mapping(contract.get("predecessors"), "predecessors")
    cohort = _mapping(predecessors.get("h3_cohort"), "predecessors.h3_cohort")
    values = _identity_fixture_values(
        root,
        cohort.get("identity_fixture"),
        EXPECTED_H3_IDENTITY_FIXTURE,
        domain=b"babylon.h3-res7-land-mask-identities.v1",
        compressed=True,
        detail="predecessors.h3_cohort.identity_fixture",
    )
    result: set[int] = set()
    for value in values:
        if (
            not h3.is_valid_cell(value)
            or h3.get_resolution(value) != 7
            or h3.int_to_str(h3.str_to_int(value)) != value
        ):
            raise CountyPlaceH3OverlapRefusal("identity_fixture_h3", value)
        result.add(h3.str_to_int(value))
    if len(result) != len(values):
        raise CountyPlaceH3OverlapRefusal("identity_fixture_h3", "duplicate integer")
    return result


def verify_artifacts(
    contract: dict[str, Any], root: Path
) -> tuple[list[tuple[object, ...]], list[tuple[object, ...]]]:
    """Verify artifact bytes, records, semantic digests, and declared evidence."""
    county_spec = contract["county_cell_artifact"]
    place_spec = contract["county_place_cell_artifact"]
    county_rows = _read_parquet(
        _artifact_path(root, county_spec["path"]), county_spec, EXPECTED_COUNTY_COLUMNS
    )
    place_rows = _read_parquet(
        _artifact_path(root, place_spec["path"]), place_spec, EXPECTED_PLACE_COLUMNS
    )
    typed_county_rows = cast(list[tuple[int, str, int]], county_rows)
    typed_place_rows = cast(list[tuple[int, str, str, int, int, int]], place_rows)
    cohort_ids = load_h3_cohort_ids(contract, root)
    county_geoids = load_county_geoids(contract, root)
    denominator = verify_county_records(typed_county_rows, cohort_ids, county_geoids)
    verify_place_records(
        typed_place_rows,
        typed_county_rows,
        cohort_ids,
        county_geoids,
        _place_geoids(root, contract),
    )
    county_semantic = _semantic_sha256(
        b"babylon.census-county-h3-land-overlap.v1",
        EXPECTED_COUNTY_COLUMNS,
        typed_county_rows,
    )
    place_semantic = _semantic_sha256(
        b"babylon.census-county-place-h3-land-overlap.v1",
        EXPECTED_PLACE_COLUMNS,
        typed_place_rows,
    )
    if county_semantic != county_spec["semantic_sha256"]:
        raise CountyPlaceH3OverlapRefusal("county_semantic_sha256", county_semantic)
    if place_semantic != place_spec["semantic_sha256"]:
        raise CountyPlaceH3OverlapRefusal("place_semantic_sha256", place_semantic)

    counties_by_cell = Counter(row[0] for row in typed_county_rows)
    counties_by_place: defaultdict[str, set[str]] = defaultdict(set)
    cells_with_place: set[int] = set()
    share_sums: defaultdict[int, int] = defaultdict(int)
    for row in typed_place_rows:
        cell_id, county_fips, place_geoid, _, _, share = row
        counties_by_place[place_geoid].add(county_fips)
        cells_with_place.add(cell_id)
        share_sums[cell_id] += share
    evidence = {
        "positive_land_cells": len(denominator),
        "cohort_absent_cells": len(cohort_ids) - len(denominator),
        "cross_county_cells": sum(count > 1 for count in counties_by_cell.values()),
        "maximum_counties_per_cell": max(counties_by_cell.values()),
        "places_crossing_counties": sum(
            len(counties) > 1 for counties in counties_by_place.values()
        ),
        "maximum_counties_per_place": max(map(len, counties_by_place.values())),
        "land_cells_without_place": len(set(denominator) - cells_with_place),
        "total_mi_land_area_m2": sum(row[2] for row in typed_county_rows),
        "total_place_land_area_m2": sum(row[3] for row in typed_place_rows),
        "maximum_share_sum_ppb": max(share_sums.values()),
        "maximum_place_share_ppb": max(row[5] for row in typed_place_rows),
        "minimum_place_share_ppb": min(row[5] for row in typed_place_rows),
    }
    if evidence != contract["evidence"]:
        raise CountyPlaceH3OverlapRefusal("artifact_evidence", repr(evidence))
    return county_rows, place_rows


def verify_artifact_manifest(contract: dict[str, Any], path: Path) -> None:
    """Tripwire the two hand-maintained second-order registry entries."""
    raw = _bounded_bytes(path, 262_144, "artifact_manifest_size")
    try:
        document = _safe_load_unique(raw)
    except yaml.YAMLError as error:
        raise CountyPlaceH3OverlapRefusal("artifact_manifest_yaml", str(path)) from error
    if not isinstance(document, dict) or not isinstance(document.get("artifacts"), list):
        raise CountyPlaceH3OverlapRefusal("artifact_manifest_shape", str(path))
    by_name: dict[str, Any] = {}
    for entry in document["artifacts"]:
        if not isinstance(entry, dict) or not isinstance(entry.get("name"), str):
            raise CountyPlaceH3OverlapRefusal("artifact_manifest_entry", repr(entry))
        if entry["name"] in by_name:
            raise CountyPlaceH3OverlapRefusal("artifact_manifest_duplicate", entry["name"])
        by_name[entry["name"]] = entry
    for key, (name, _, _) in EXPECTED_ARTIFACTS.items():
        spec = contract[key]
        entry = by_name.get(name)
        if not isinstance(entry, dict):
            raise CountyPlaceH3OverlapRefusal("artifact_manifest_pin", name)
        expected = {
            "name": name,
            "format": "parquet",
            "source_table": None,
            "generator": "tools/make_county_place_h3_overlap_artifacts.py",
            "mode": "register",
            "rows": spec["rows"],
            "sha256": spec["sha256"],
            "home": spec["path"],
            "material_relation": entry.get("material_relation"),
        }
        if entry != expected:
            raise CountyPlaceH3OverlapRefusal("artifact_manifest_pin", name)
        relation = entry["material_relation"]
        if (
            not isinstance(relation, str)
            or "PER-277" not in relation
            or "no PostgreSQL" not in relation
        ):
            raise CountyPlaceH3OverlapRefusal("artifact_manifest_relation", name)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--contract", type=Path, default=Path("contracts/county_place_h3_overlap_v1.yaml")
    )
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    args = parser.parse_args(argv)
    contract = load_contract(args.contract)
    verify_contract(contract)
    verify_source_manifest(contract, args.repo_root / EXPECTED_SOURCE_MANIFEST)
    verify_predecessors(contract, args.repo_root)
    county_rows, place_rows = verify_artifacts(contract, args.repo_root)
    verify_artifact_manifest(contract, args.repo_root / "data-artifacts.yaml")
    print(
        "CountyPlaceH3OverlapV1 verified: "
        f"{len(county_rows)} county-cell rows, {len(place_rows)} county-place-cell rows"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
