"""Contracts for PER-277's bounded Michigan county/place/H3 overlap authority."""

from __future__ import annotations

import copy
import hashlib
import json
import math
import sys
import zipfile
from pathlib import Path

import geopandas as gpd
import h3
import pyarrow.parquet as pq
import pytest
import yaml
from shapely.geometry import GeometryCollection, LineString, Point, Polygon, box

ROOT = Path(__file__).resolve().parents[3]
TOOLS = ROOT / "tools"
sys.path.insert(0, str(TOOLS))

import make_county_place_h3_overlap_artifacts as builder  # type: ignore[import-not-found]  # noqa: E402
import verify_county_place_h3_overlap_v1 as verifier  # type: ignore[import-not-found]  # noqa: E402
from verify_county_place_h3_overlap_v1 import (  # type: ignore[import-not-found]  # noqa: E402
    CountyPlaceH3OverlapRefusal,
    load_contract,
    verify_artifact_manifest,
    verify_artifacts,
    verify_contract,
    verify_county_records,
    verify_place_records,
    verify_predecessors,
    verify_source_manifest,
)

CONTRACT = ROOT / "contracts" / "county_place_h3_overlap_v1.yaml"
FETCH_MANIFEST = TOOLS / "county_place_h3_overlap_v1_fetch_manifest.json"
ARTIFACT_MANIFEST = ROOT / "data-artifacts.yaml"
CELL_ID = int("872748190ffffff", 16)


def _county_source_frame() -> gpd.GeoDataFrame:
    polygon = Polygon([(-84.0, 42.0), (-83.0, 42.0), (-83.0, 43.0), (-84.0, 42.0)])
    return gpd.GeoDataFrame(
        [
            {
                "STATEFP": "26",
                "COUNTYFP": "001",
                "COUNTYNS": "01622943",
                "GEOID": "26001",
                "GEOIDFQ": "0500000US26001",
                "NAME": "Alcona",
                "NAMELSAD": "Alcona County",
                "LSAD": "06",
                "CLASSFP": "H1",
                "MTFCC": "G4020",
                "CSAFP": None,
                "CBSAFP": None,
                "METDIVFP": None,
                "FUNCSTAT": "A",
                "ALAND": 1,
                "AWATER": 0,
                "INTPTLAT": "+44.5000000",
                "INTPTLON": "-083.5000000",
                "geometry": polygon,
            }
        ],
        geometry="geometry",
        crs="EPSG:4269",
    )


def test_dedicated_manifest_declares_only_the_pinned_county_archive() -> None:
    entries = builder.load_fetch_manifest(FETCH_MANIFEST)

    assert entries == [
        {
            "url": "https://www2.census.gov/geo/tiger/TIGER2023/COUNTY/tl_2023_us_county.zip",
            "dest": "tiger/county/tl_2023_us_county.zip",
            "sha256": entries[0]["sha256"],
        }
    ]
    assert len(entries[0]["sha256"]) == 64


def test_checked_in_contract_and_artifacts_verify() -> None:
    contract = load_contract(CONTRACT)

    assert contract["meta"] == {
        "contract": "CountyPlaceH3OverlapV1",
        "version": 1,
        "issue": "PER-277",
        "parent": "PER-21",
    }
    verify_contract(contract)
    verify_source_manifest(contract, FETCH_MANIFEST)
    verify_predecessors(contract, ROOT)
    county_rows, place_rows = verify_artifacts(contract, ROOT)
    verify_artifact_manifest(contract, ARTIFACT_MANIFEST)
    assert len(county_rows) == contract["county_cell_artifact"]["rows"]
    assert len(place_rows) == contract["county_place_cell_artifact"]["rows"]


def test_contract_refuses_missing_substantive_value_classification() -> None:
    contract = copy.deepcopy(load_contract(CONTRACT))
    del contract["classifications"]["land_area_measures"]

    with pytest.raises(CountyPlaceH3OverlapRefusal, match="contract_shape"):
        verify_contract(contract)


def test_contract_loader_refuses_duplicate_keys(tmp_path: Path) -> None:
    duplicate = tmp_path / "duplicate.yaml"
    duplicate.write_text("meta: first\nmeta: second\n", encoding="utf-8")

    with pytest.raises(CountyPlaceH3OverlapRefusal, match="invalid_contract"):
        load_contract(duplicate)


def test_source_bytes_are_verified_before_zip_decode(tmp_path: Path) -> None:
    archive = tmp_path / "source.zip"
    archive.write_bytes(b"not a zip")

    with pytest.raises(builder.OverlapBuildError, match="source_sha256"):
        builder.verify_source_archive(archive, "0" * 64)


def test_county_canonicalizer_refuses_schema_and_crs_drift() -> None:
    with pytest.raises(builder.OverlapBuildError, match="county_source_schema"):
        builder.canonicalize_county_source(_county_source_frame().drop(columns=["COUNTYNS"]))

    frame = _county_source_frame().to_crs("EPSG:5070")
    with pytest.raises(builder.OverlapBuildError, match="county_source_crs"):
        builder.canonicalize_county_source(frame)


@pytest.mark.parametrize(
    ("members", "code"),
    [
        (["../tl_2023_us_county.shp"], "zip_member_path"),
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

    with pytest.raises(builder.OverlapBuildError, match=code):
        builder.verify_zip_members(archive, builder.expected_county_members())


def test_zip_member_allowlist_refuses_duplicate_members(tmp_path: Path) -> None:
    archive = tmp_path / "source.zip"
    with zipfile.ZipFile(archive, "w") as zipped:
        for member in sorted(builder.expected_county_members()):
            zipped.writestr(member, b"fixture")
        with pytest.warns(UserWarning, match="Duplicate name"):
            zipped.writestr("tl_2023_us_county.cpg", b"duplicate")

    with pytest.raises(builder.OverlapBuildError, match="zip_member_duplicate"):
        builder.verify_zip_members(archive, builder.expected_county_members())


@pytest.mark.parametrize("value", [-1.0, math.inf, -math.inf, math.nan])
def test_area_quantization_refuses_negative_or_nonfinite_values(value: float) -> None:
    with pytest.raises(builder.OverlapBuildError, match="area_value"):
        builder.quantize_area_m2(value)


def test_area_quantization_is_conservative_whole_square_metres() -> None:
    assert builder.quantize_area_m2(0.999999) == 0
    assert builder.quantize_area_m2(1.0) == 1
    assert builder.quantize_area_m2(19.999999) == 19


def test_polygonal_extraction_discards_boundary_only_components() -> None:
    polygon = box(0.0, 0.0, 10.0, 10.0)
    mixed = GeometryCollection([polygon, LineString([(0.0, 0.0), (10.0, 0.0)]), Point(0, 0)])

    extracted = builder.polygonal_components(mixed)

    assert extracted.equals(polygon)
    assert builder.polygonal_components(LineString([(0.0, 0.0), (1.0, 0.0)])).is_empty


def test_derivation_preserves_true_cross_county_place_rows() -> None:
    cells = {CELL_ID: box(0.0, 0.0, 20.0, 10.0)}
    counties = {
        "26001": box(0.0, 0.0, 10.0, 10.0),
        "26003": box(10.0, 0.0, 20.0, 10.0),
    }
    places = {"2600001": box(5.0, 0.0, 15.0, 10.0)}

    county_rows, place_rows = builder.derive_overlap_rows(cells, counties, places)

    assert county_rows == [
        builder.CountyLandRow(CELL_ID, "26001", 100),
        builder.CountyLandRow(CELL_ID, "26003", 100),
    ]
    assert place_rows == [
        builder.PlaceLandRow(CELL_ID, "26001", "2600001", 50, 200, 250_000_000),
        builder.PlaceLandRow(CELL_ID, "26003", "2600001", 50, 200, 250_000_000),
    ]


def test_derivation_omits_zero_area_boundary_touches() -> None:
    cells = {CELL_ID: box(0.0, 0.0, 20.0, 10.0)}
    counties = {"26001": box(0.0, 0.0, 20.0, 10.0)}
    places = {"2600001": box(20.0, 0.0, 30.0, 10.0)}

    county_rows, place_rows = builder.derive_overlap_rows(cells, counties, places)

    assert county_rows == [builder.CountyLandRow(CELL_ID, "26001", 200)]
    assert place_rows == []


def test_parquet_writers_are_byte_identical_across_paths(tmp_path: Path) -> None:
    county_rows = [builder.CountyLandRow(CELL_ID, "26001", 200)]
    place_rows = [builder.PlaceLandRow(CELL_ID, "26001", "2600001", 50, 200, 250_000_000)]
    first_county = tmp_path / "first-county.parquet"
    second_county = tmp_path / "second-county.parquet"
    first_place = tmp_path / "first-place.parquet"
    second_place = tmp_path / "second-place.parquet"

    assert builder.write_county_parquet(first_county, county_rows) == builder.write_county_parquet(
        second_county, county_rows
    )
    assert builder.write_place_parquet(first_place, place_rows) == builder.write_place_parquet(
        second_place, place_rows
    )
    assert first_county.read_bytes() == second_county.read_bytes()
    assert first_place.read_bytes() == second_place.read_bytes()


def test_record_verifier_refuses_unknown_h3_and_conservation_drift() -> None:
    county = [(CELL_ID, "26001", 200)]
    denominator = verify_county_records(county, {CELL_ID}, {"26001"})
    assert denominator == {CELL_ID: 200}

    with pytest.raises(CountyPlaceH3OverlapRefusal, match="county_cell_unknown"):
        verify_county_records([(CELL_ID + 1, "26001", 200)], {CELL_ID}, {"26001"})

    with pytest.raises(CountyPlaceH3OverlapRefusal, match="place_conservation"):
        verify_place_records(
            [(CELL_ID, "26001", "2600001", 201, 200, 1_000_000_000)],
            county,
            {CELL_ID},
            {"26001"},
            {"2600001"},
        )


def test_record_verifier_refuses_syntactically_valid_unknown_county() -> None:
    with pytest.raises(CountyPlaceH3OverlapRefusal, match="county_fips_unknown"):
        verify_county_records([(CELL_ID, "26999", 200)], {CELL_ID}, {"26001"})


def test_artifact_verifier_uses_the_pinned_h3_identity_fixture(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    contract = load_contract(CONTRACT)
    cohort_ids = verifier.load_h3_cohort_ids(contract, ROOT)
    outside_cell_id = h3.str_to_int(h3.latlng_to_cell(0.0, 0.0, 7))
    assert outside_cell_id not in cohort_ids
    original_read = verifier._read_parquet

    def read_with_outside_cell(
        path: Path, spec: dict[str, object], columns: tuple[str, ...]
    ) -> list[tuple[object, ...]]:
        rows = original_read(path, spec, columns)
        if columns == verifier.EXPECTED_COUNTY_COLUMNS:
            first = rows[0]
            return [(outside_cell_id, first[1], first[2]), *rows[1:]]
        return rows

    monkeypatch.setattr(verifier, "_read_parquet", read_with_outside_cell)

    with pytest.raises(CountyPlaceH3OverlapRefusal, match="county_cell_unknown"):
        verify_artifacts(contract, ROOT)


@pytest.mark.parametrize(
    ("record", "code"),
    [
        ((CELL_ID, "26001", "2600001", 0, 200, 0), "place_measure"),
        ((CELL_ID, "26001", "2600001", 50, 199, 251_256_281), "place_denominator"),
        ((CELL_ID, "26001", "2600001", 50, 200, 1), "place_share"),
        ((CELL_ID, "27001", "2600001", 50, 200, 250_000_000), "place_county"),
        ((CELL_ID, "26001", "2700001", 50, 200, 250_000_000), "place_geoid"),
    ],
)
def test_place_record_verifier_refuses_invalid_rows(record: tuple[object, ...], code: str) -> None:
    county = [(CELL_ID, "26001", 200)]

    with pytest.raises(CountyPlaceH3OverlapRefusal, match=code):
        verify_place_records([record], county, {CELL_ID}, {"26001"}, {"2600001"})


def test_artifact_verifier_refuses_noncanonical_parquet_compression(tmp_path: Path) -> None:
    contract = copy.deepcopy(load_contract(CONTRACT))
    county_spec = contract["county_cell_artifact"]
    source = ROOT / str(county_spec["path"])
    table = pq.read_table(source)
    destination = tmp_path / str(county_spec["path"])
    destination.parent.mkdir(parents=True)
    pq.write_table(table, destination, compression="snappy")
    county_spec["sha256"] = hashlib.sha256(destination.read_bytes()).hexdigest()
    contract["county_place_cell_artifact"]["path"] = str(
        ROOT / str(contract["county_place_cell_artifact"]["path"])
    )

    with pytest.raises(CountyPlaceH3OverlapRefusal, match="artifact_compression"):
        verify_artifacts(contract, tmp_path)


def test_artifact_manifest_tripwire_is_not_managed_by_sqlite_generator() -> None:
    manifest = yaml.safe_load(ARTIFACT_MANIFEST.read_text(encoding="utf-8"))
    by_name = {row["name"]: row for row in manifest["artifacts"]}
    managed = {spec.name for spec in builder.make_data_artifacts_specs()}

    assert set(builder.ARTIFACT_NAMES) <= set(by_name)
    assert not set(builder.ARTIFACT_NAMES) & managed


def test_contract_json_is_finite_and_canonicalizable() -> None:
    contract = load_contract(CONTRACT)

    assert json.dumps(contract, sort_keys=True, allow_nan=False).encode("utf-8")
