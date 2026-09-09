"""Independent contract tests for PER-276 Census place authority."""

from __future__ import annotations

import copy
import csv
import gzip
import hashlib
import io
import json
import sys
import zipfile
from pathlib import Path
from typing import Any

import geopandas as gpd
import pandas as pd
import pytest
import yaml
from shapely import to_wkb
from shapely.geometry import Point, Polygon

ROOT = Path(__file__).resolve().parents[3]
TOOLS = ROOT / "tools"
sys.path.insert(0, str(TOOLS))

import make_census_place_artifacts as builder  # type: ignore[import-not-found]  # noqa: E402
from verify_census_place_authority_v1 import (  # type: ignore[import-not-found]  # noqa: E402
    CensusPlaceAuthorityRefusal,
    load_contract,
    verify_artifact_manifest,
    verify_artifacts,
    verify_contract,
    verify_source_manifest,
)

CONTRACT = ROOT / "contracts" / "census_place_authority_v1.yaml"
MANIFEST = ROOT / "data-artifacts.yaml"
FETCH_MANIFEST = ROOT / "tools" / "census_place_authority_v1_fetch_manifest.json"


def _gzip_csv(path: Path, header: list[str], rows: list[list[Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with (
        path.open("wb") as raw,
        gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as binary,
        io.TextIOWrapper(binary, encoding="utf-8", newline="") as text,
    ):
        writer = csv.writer(text, lineterminator="\n")
        writer.writerow(header)
        writer.writerows(rows)


def _geometry_parquet(path: Path, rows: list[list[Any]], *, compression: str = "zstd") -> None:
    import pyarrow as pa
    import pyarrow.parquet as pq

    def wkb_bytes(value: object) -> bytes:
        try:
            return bytes.fromhex(str(value))
        except ValueError:
            return str(value).encode("utf-8")

    table = pa.table(
        {
            "place_geoid": pa.array((row[0] for row in rows), type=pa.string()),
            "geometry_wkb": pa.array((wkb_bytes(row[1]) for row in rows), type=pa.binary()),
            "aland_m2": pa.array((row[2] for row in rows), type=pa.uint64()),
            "awater_m2": pa.array((row[3] for row in rows), type=pa.uint64()),
            "internal_point_lat": pa.array((row[4] for row in rows), type=pa.string()),
            "internal_point_lon": pa.array((row[5] for row in rows), type=pa.string()),
        }
    )
    options: dict[str, Any] = {
        "compression": compression,
        "use_dictionary": False,
        "write_statistics": False,
        "data_page_version": "1.0",
        "version": "2.6",
    }
    if compression == "zstd":
        options["compression_level"] = 22
    pq.write_table(table, path, **options)


def _minimal_contract(tmp_path: Path) -> dict[str, Any]:
    contract = copy.deepcopy(load_contract(CONTRACT))
    identity = tmp_path / "identity.csv.gz"
    geometry = tmp_path / "geometry.parquet"
    polygon = Polygon([(-84.0, 42.0), (-83.0, 42.0), (-83.0, 43.0), (-84.0, 42.0)])
    _gzip_csv(
        identity,
        list(contract["identity_artifact"]["columns"]),
        [
            [
                "2600001",
                "26",
                "00001",
                "01234567",
                "Alpha",
                "Alpha city",
                "25",
                "C1",
                "N",
                "G4110",
                "A",
            ]
        ],
    )
    _geometry_parquet(
        geometry,
        [["2600001", to_wkb(polygon, hex=True, byte_order=1), 1, 0, "42.5", "-83.5"]],
    )
    for key, path in (("identity_artifact", identity), ("geometry_artifact", geometry)):
        spec = contract[key]
        spec["path"] = str(path.relative_to(tmp_path))
        spec["rows"] = 1
        spec["sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
    contract["extent"] = {
        "min_lon": -84.0,
        "min_lat": 42.0,
        "max_lon": -83.0,
        "max_lat": 43.0,
    }
    return contract


def _source_frame() -> gpd.GeoDataFrame:
    polygon = Polygon([(-84.0, 42.0), (-83.0, 42.0), (-83.0, 43.0), (-84.0, 42.0)])
    return gpd.GeoDataFrame(
        [
            {
                "STATEFP": "26",
                "PLACEFP": "00001",
                "PLACENS": "01234567",
                "GEOID": "2600001",
                "GEOIDFQ": "1600000US2600001",
                "NAME": "Alpha",
                "NAMELSAD": "Alpha city",
                "LSAD": "25",
                "CLASSFP": "C1",
                "PCICBSA": "N",
                "MTFCC": "G4110",
                "FUNCSTAT": "A",
                "ALAND": 1,
                "AWATER": 0,
                "INTPTLAT": "+42.5000000",
                "INTPTLON": "-083.5000000",
                "geometry": polygon,
            }
        ],
        geometry="geometry",
        crs="EPSG:4269",
    )


def test_dedicated_manifest_declares_only_the_pinned_michigan_place_archive() -> None:
    entries = builder.load_fetch_manifest(FETCH_MANIFEST)

    assert entries == [
        {
            "url": "https://www2.census.gov/geo/tiger/TIGER2023/PLACE/tl_2023_26_place.zip",
            "dest": "tiger/place/tl_2023_26_place.zip",
            "sha256": entries[0]["sha256"],
        }
    ]
    assert len(entries[0]["sha256"]) == 64


def test_checked_in_contract_and_artifacts_verify() -> None:
    contract = load_contract(CONTRACT)

    assert contract["meta"] == {
        "contract": "CensusPlaceAuthorityV1",
        "version": 1,
        "issue": "PER-276",
        "parent": "PER-21",
    }
    verify_contract(contract)
    verify_source_manifest(contract, FETCH_MANIFEST)
    identities, geometries = verify_artifacts(contract, ROOT)
    verify_artifact_manifest(contract, MANIFEST)
    assert len(identities) == len(geometries) == contract["identity_artifact"]["rows"]


def test_contract_refuses_missing_substantive_value_classification() -> None:
    contract = copy.deepcopy(load_contract(CONTRACT))
    del contract["classifications"]["semantic_digests"]

    with pytest.raises(CensusPlaceAuthorityRefusal, match="contract_shape"):
        verify_contract(contract)


def test_contract_loader_refuses_duplicate_keys(tmp_path: Path) -> None:
    duplicate = tmp_path / "duplicate.yaml"
    duplicate.write_text("meta: first\nmeta: second\n", encoding="utf-8")

    with pytest.raises(CensusPlaceAuthorityRefusal, match="invalid_contract"):
        load_contract(duplicate)


def test_source_bytes_are_verified_before_zip_decode(tmp_path: Path) -> None:
    archive = tmp_path / "source.zip"
    archive.write_bytes(b"not a zip")

    with pytest.raises(builder.PlaceAuthorityBuildError, match="source_sha256"):
        builder.verify_source_archive(archive, "0" * 64)


def test_canonicalizer_refuses_source_schema_drift() -> None:
    frame = _source_frame().drop(columns=["PLACENS"])

    with pytest.raises(builder.PlaceAuthorityBuildError, match="source_schema"):
        builder.canonicalize_place_rows(frame)


@pytest.mark.parametrize(
    ("field", "value", "code"),
    [
        ("STATEFP", "27", "source_geoid"),
        ("GEOID", "2600002", "source_geoid"),
        ("GEOIDFQ", "1600000US2600002", "source_geoid_fq"),
        ("INTPTLAT", "nan", "source_coordinate"),
    ],
)
def test_canonicalizer_refuses_identity_or_nonfinite_source_values(
    field: str, value: object, code: str
) -> None:
    frame = _source_frame()
    frame.loc[0, field] = value

    with pytest.raises(builder.PlaceAuthorityBuildError, match=code):
        builder.canonicalize_place_rows(frame)


def test_canonicalizer_refuses_duplicate_geoids() -> None:
    frame = _source_frame()
    duplicate = gpd.GeoDataFrame(pd.concat([frame, frame], ignore_index=True), crs=frame.crs)

    with pytest.raises(builder.PlaceAuthorityBuildError, match="source_duplicate_geoid"):
        builder.canonicalize_place_rows(duplicate)


@pytest.mark.parametrize(
    ("geometry", "code"),
    [
        (Point(-83.5, 42.5), "source_geometry_type"),
        (
            Polygon([(-84.0, 42.0), (-83.0, 43.0), (-84.0, 43.0), (-83.0, 42.0)]),
            "source_geometry_invalid",
        ),
    ],
)
def test_canonicalizer_refuses_non_polygonal_or_invalid_geometry(
    geometry: object, code: str
) -> None:
    frame = _source_frame()
    frame.at[0, "geometry"] = geometry

    with pytest.raises(builder.PlaceAuthorityBuildError, match=code):
        builder.canonicalize_place_rows(frame)


@pytest.mark.parametrize(
    ("members", "code"),
    [
        (["../tl_2023_26_place.shp"], "zip_member_path"),
        (["unexpected.txt"], "zip_members"),
    ],
)
def test_zip_member_allowlist_refuses_unsafe_or_unexpected_members(
    tmp_path: Path, members: list[str], code: str
) -> None:
    archive = tmp_path / "source.zip"
    with zipfile.ZipFile(archive, "w") as zipped:
        for member in members:
            zipped.writestr(member, b"fixture")

    with pytest.raises(builder.PlaceAuthorityBuildError, match=code):
        builder.verify_zip_members(archive)


def test_zip_member_allowlist_refuses_duplicate_members(tmp_path: Path) -> None:
    archive = tmp_path / "source.zip"
    with zipfile.ZipFile(archive, "w") as zipped:
        for member in sorted(builder.EXPECTED_ZIP_MEMBERS):
            zipped.writestr(member, b"fixture")
        with pytest.warns(UserWarning, match="Duplicate name"):
            zipped.writestr("tl_2023_26_place.cpg", b"duplicate")

    with pytest.raises(builder.PlaceAuthorityBuildError, match="zip_member_duplicate"):
        builder.verify_zip_members(archive)


def test_gzip_writer_is_byte_identical_across_output_paths(tmp_path: Path) -> None:
    rows = [
        ["2600001", "26", "00001", "01234567", "Alpha", "Alpha city", "25", "C1", "N", "G4110", "A"]
    ]
    first = tmp_path / "first.csv.gz"
    second = tmp_path / "second.csv.gz"

    first_stats = builder._write_gzip_csv(first, builder.IDENTITY_COLUMNS, rows)
    second_stats = builder._write_gzip_csv(second, builder.IDENTITY_COLUMNS, rows)

    assert first.read_bytes() == second.read_bytes()
    assert first_stats == second_stats


def test_geometry_parquet_writer_is_byte_identical_across_output_paths(tmp_path: Path) -> None:
    polygon = Polygon([(-84.0, 42.0), (-83.0, 42.0), (-83.0, 43.0), (-84.0, 42.0)])
    rows = [["2600001", to_wkb(polygon, hex=True, byte_order=1), 1, 0, "42.5", "-83.5"]]
    first = tmp_path / "first.parquet"
    second = tmp_path / "second.parquet"

    first_stats = builder._write_geometry_parquet(first, rows)
    second_stats = builder._write_geometry_parquet(second, rows)

    assert first.read_bytes() == second.read_bytes()
    assert first_stats == second_stats


def test_artifact_verifier_refuses_unsorted_identity(tmp_path: Path) -> None:
    contract = _minimal_contract(tmp_path)
    identity_path = tmp_path / str(contract["identity_artifact"]["path"])
    _gzip_csv(
        identity_path,
        list(contract["identity_artifact"]["columns"]),
        [
            [
                "2600002",
                "26",
                "00002",
                "01234568",
                "Beta",
                "Beta city",
                "25",
                "C1",
                "N",
                "G4110",
                "A",
            ],
            [
                "2600001",
                "26",
                "00001",
                "01234567",
                "Alpha",
                "Alpha city",
                "25",
                "C1",
                "N",
                "G4110",
                "A",
            ],
        ],
    )
    contract["identity_artifact"]["rows"] = 2
    contract["identity_artifact"]["sha256"] = hashlib.sha256(identity_path.read_bytes()).hexdigest()

    with pytest.raises(CensusPlaceAuthorityRefusal, match="identity_order"):
        verify_artifacts(contract, tmp_path)


def test_artifact_verifier_refuses_duplicate_identity(tmp_path: Path) -> None:
    contract = _minimal_contract(tmp_path)
    identity_path = tmp_path / str(contract["identity_artifact"]["path"])
    duplicate = [
        "2600001",
        "26",
        "00001",
        "01234567",
        "Alpha",
        "Alpha city",
        "25",
        "C1",
        "N",
        "G4110",
        "A",
    ]
    _gzip_csv(
        identity_path,
        list(contract["identity_artifact"]["columns"]),
        [duplicate, duplicate],
    )
    contract["identity_artifact"]["rows"] = 2
    contract["identity_artifact"]["sha256"] = hashlib.sha256(identity_path.read_bytes()).hexdigest()

    with pytest.raises(CensusPlaceAuthorityRefusal, match="identity_order"):
        verify_artifacts(contract, tmp_path)


def test_artifact_verifier_refuses_identity_geometry_keyset_drift(tmp_path: Path) -> None:
    contract = _minimal_contract(tmp_path)
    geometry_path = tmp_path / str(contract["geometry_artifact"]["path"])
    polygon = Polygon([(-84.0, 42.0), (-83.0, 42.0), (-83.0, 43.0), (-84.0, 42.0)])
    _geometry_parquet(
        geometry_path,
        [["2600002", to_wkb(polygon, hex=True, byte_order=1).lower(), 1, 0, "42.5", "-83.5"]],
    )
    contract["geometry_artifact"]["sha256"] = hashlib.sha256(geometry_path.read_bytes()).hexdigest()

    with pytest.raises(CensusPlaceAuthorityRefusal, match="artifact_keyset"):
        verify_artifacts(contract, tmp_path)


@pytest.mark.parametrize(
    ("wkb_hex", "code"),
    [
        ("not-hex", "geometry_wkb"),
        (to_wkb(Polygon(), hex=True, byte_order=1), "geometry_empty"),
    ],
)
def test_artifact_verifier_refuses_malformed_or_empty_geometry(
    tmp_path: Path, wkb_hex: str, code: str
) -> None:
    contract = _minimal_contract(tmp_path)
    geometry_path = tmp_path / str(contract["geometry_artifact"]["path"])
    _geometry_parquet(
        geometry_path,
        [["2600001", wkb_hex, 1, 0, "42.5", "-83.5"]],
    )
    contract["geometry_artifact"]["sha256"] = hashlib.sha256(geometry_path.read_bytes()).hexdigest()

    with pytest.raises(CensusPlaceAuthorityRefusal, match=code):
        verify_artifacts(contract, tmp_path)


def test_artifact_verifier_refuses_noncanonical_parquet_compression(tmp_path: Path) -> None:
    contract = _minimal_contract(tmp_path)
    geometry_path = tmp_path / str(contract["geometry_artifact"]["path"])
    polygon = Polygon([(-84.0, 42.0), (-83.0, 42.0), (-83.0, 43.0), (-84.0, 42.0)])
    _geometry_parquet(
        geometry_path,
        [["2600001", to_wkb(polygon, hex=True, byte_order=1), 1, 0, "42.5", "-83.5"]],
        compression="snappy",
    )
    contract["geometry_artifact"]["sha256"] = hashlib.sha256(geometry_path.read_bytes()).hexdigest()

    with pytest.raises(CensusPlaceAuthorityRefusal, match="artifact_parquet_compression"):
        verify_artifacts(contract, tmp_path)


def test_artifact_manifest_tripwire_is_not_managed_by_sqlite_generator() -> None:
    manifest = yaml.safe_load(MANIFEST.read_text(encoding="utf-8"))
    by_name = {row["name"]: row for row in manifest["artifacts"]}
    managed = {spec.name for spec in builder.make_data_artifacts_specs()}

    assert set(builder.ARTIFACT_NAMES) <= set(by_name)
    assert not set(builder.ARTIFACT_NAMES) & managed


def test_contract_json_is_finite_and_canonicalizable() -> None:
    contract = load_contract(CONTRACT)

    encoded = json.dumps(contract, sort_keys=True, allow_nan=False).encode("utf-8")
    assert encoded
