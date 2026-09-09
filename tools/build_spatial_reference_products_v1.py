#!/usr/bin/env python3
"""Build PER-278's bounded Rust fixture from exact governed P0/P1 artifacts."""

from __future__ import annotations

import argparse
import csv
import gzip
import hashlib
import io
import math
import os
import re
import stat
import struct
from dataclasses import dataclass
from pathlib import Path
from typing import Final

MAGIC: Final = b"BABYLONSPATREF1\0"
FORMAT_VERSION: Final = 1
REF_DIGEST: Final = bytes.fromhex(
    "92b21ff325bde67f26565f52882d3664daacd6d51423f2a588344da012fd4161"
)
MAX_ARTIFACT_BYTES: Final = 1_048_576
MAX_ARTIFACT_ROWS: Final = 65_536
MAX_STRING_BYTES: Final = 255
MAX_FIXTURE_PART_BYTES: Final = 1_000_000
EXPECTED_FIXTURE_BYTES: Final = 2_325_740
EXPECTED_FIXTURE_PARTS: Final = 3
PLACE_COLUMNS: Final = (
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


class BuildRefusal(RuntimeError):
    """Closed failure while verifying or framing one governed input."""


@dataclass(frozen=True)
class ArtifactSpec:
    name: str
    relative_path: str
    size: int
    sha256: str
    rows: int
    columns: tuple[tuple[str, str, bool], ...]


P0_SPECS: Final = (
    ArtifactSpec(
        "dim_county",
        "dim_county.parquet",
        36_199,
        "130b7679d0441d5c3c2183a2bef858073d3011039550bfbf015b380566c72032",
        3_285,
        (
            ("county_id", "int64", True),
            ("fips", "string", True),
            ("state_id", "int64", True),
            ("county_fips", "string", True),
            ("county_name", "string", True),
            ("h3_res4", "string", True),
        ),
    ),
    ArtifactSpec(
        "h3_res7_land_mask",
        "h3_res7_land_mask.parquet",
        295_194,
        "4e6caba297f0111a9ec93d948a83543bb9f7179361fe5dd318bb8a98a5be5194",
        45_572,
        (
            ("h3_index", "string", False),
            ("county_fips", "string", False),
            ("land_fraction", "double", False),
        ),
    ),
    ArtifactSpec(
        "h3_res7_population",
        "h3_res7_population.parquet",
        106_095,
        "b096a5891284f0ca55bedae9d1a9092eb8ea9e9e32d32b6ace430a9833b53afc",
        22_509,
        (("h3_index", "string", False), ("population", "double", False)),
    ),
    ArtifactSpec(
        "h3_res7_workplace",
        "h3_res7_workplace.parquet",
        65_162,
        "ea2ce1508f4fe51f1e879b9f4a1daf579c4b00349388b12a85f884a8f49eabb6",
        11_833,
        (("h3_index", "string", False), ("jobs", "double", False)),
    ),
)

P1_SPECS: Final = (
    ArtifactSpec(
        "census_county_h3_land_overlap_mi_2023",
        "src/babylon/data/reference/spatial/census_county_h3_land_overlap_mi_2023.parquet",
        136_487,
        "7054fe2efa378e4db055a6647b9a3834cc382d822a032652b33894732b55b3c3",
        31_881,
        (
            ("cell_id", "int64", False),
            ("county_fips", "string", False),
            ("land_area_m2", "uint64", False),
        ),
    ),
    ArtifactSpec(
        "census_county_place_h3_land_overlap_mi_2023",
        "src/babylon/data/reference/spatial/census_county_place_h3_land_overlap_mi_2023.parquet",
        58_505,
        "fcb7baaf63a5422accce8709997de8e409936f7131fa0ef6b0a28762fdfee42f",
        4_813,
        (
            ("cell_id", "int64", False),
            ("county_fips", "string", False),
            ("place_geoid", "string", False),
            ("place_land_area_m2", "uint64", False),
            ("cell_mi_land_area_m2", "uint64", False),
            ("place_land_area_share_ppb", "uint32", False),
        ),
    ),
)

PLACE_PATH: Final = "src/babylon/data/reference/spatial/census_place_identity_mi_2023.csv.gz"
PLACE_SIZE: Final = 12_580
PLACE_SHA256: Final = "cb864b4f6f43902bb821e84fe9a4055a9039e0a74d8b8399f209ae6ed26a8be7"
PLACE_ROWS: Final = 745
PREDECESSOR_CONTRACTS: Final = (
    (
        "contracts/h3_estate_contract_v1.yaml",
        28_263,
        "1ea546e572c499424086d52bc62820626328c8336c38d57b6053c337c0b943b5",
    ),
    (
        "contracts/census_place_authority_v1.yaml",
        2_820,
        "0bced499b9144e51d48bc2356260448bc09ab56b64035ab94c98a7287b462102",
    ),
    (
        "contracts/county_place_h3_overlap_v1.yaml",
        6_871,
        "e629d87e89175be4ff5a4d271f403f35736d8a413cf95181e2d00b5997a561fa",
    ),
)


def _verified_bytes(path: Path, *, size: int, sha256: str) -> bytes:
    if size > MAX_ARTIFACT_BYTES:
        raise BuildRefusal(f"artifact bound: {path}")
    flags = os.O_RDONLY | os.O_CLOEXEC
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(path, flags)
    except OSError as error:
        raise BuildRefusal(f"artifact open: {path}") from error
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_size != size:
            raise BuildRefusal(f"artifact size: {path}")
        with os.fdopen(descriptor, "rb", closefd=True) as source:
            descriptor = -1
            payload = source.read(size + 1)
    finally:
        if descriptor >= 0:
            os.close(descriptor)
    if len(payload) != size or hashlib.sha256(payload).hexdigest() != sha256:
        raise BuildRefusal(f"artifact digest: {path}")
    return payload


def _read_parquet(path: Path, spec: ArtifactSpec) -> list[dict[str, object]]:
    import pyarrow as pa
    import pyarrow.parquet as pq

    payload = _verified_bytes(path, size=spec.size, sha256=spec.sha256)
    try:
        parquet = pq.ParquetFile(pa.BufferReader(payload))
        if parquet.num_row_groups != 1 or parquet.metadata.num_rows != spec.rows:
            raise BuildRefusal(f"artifact rows: {spec.name}")
        table = parquet.read()
    except BuildRefusal:
        raise
    except (OSError, ValueError, pa.ArrowException) as error:
        raise BuildRefusal(f"artifact decode: {spec.name}") from error
    actual = tuple((field.name, str(field.type), field.nullable) for field in table.schema)
    if actual != spec.columns or table.num_rows > MAX_ARTIFACT_ROWS:
        raise BuildRefusal(f"artifact schema: {spec.name}: {actual!r}")
    return table.to_pylist()


def _ascii(value: object, pattern: str, field: str) -> str:
    if not isinstance(value, str) or re.fullmatch(pattern, value, flags=re.ASCII) is None:
        raise BuildRefusal(f"invalid {field}: {value!r}")
    return value


def _text(value: object, field: str) -> str:
    if not isinstance(value, str) or not value or "\x00" in value:
        raise BuildRefusal(f"invalid {field}: {value!r}")
    encoded = value.encode("utf-8")
    if len(encoded) > MAX_STRING_BYTES:
        raise BuildRefusal(f"oversized {field}")
    return value


def _unsigned(value: object, maximum: int, field: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or not 0 <= value <= maximum:
        raise BuildRefusal(f"invalid {field}: {value!r}")
    return value


def _count(value: object, field: str) -> int:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise BuildRefusal(f"invalid {field}: {value!r}")
    number = float(value)
    if not math.isfinite(number) or not number.is_integer() or not 0 <= number <= 2**53:
        raise BuildRefusal(f"invalid {field}: {value!r}")
    return int(number)


def _cell_text(value: object) -> int:
    text = _ascii(value, r"[0-9a-f]{15}", "H3 identity")
    raw = int(text, 16)
    if raw <= 0 or format(raw, "x") != text:
        raise BuildRefusal(f"noncanonical H3 identity: {text}")
    return raw


def _cell_int(value: object) -> int:
    raw = _unsigned(value, 2**63 - 1, "H3 identity")
    if raw == 0:
        raise BuildRefusal("zero H3 identity")
    return raw


def _fixed_ascii(output: bytearray, value: str, length: int, field: str) -> None:
    encoded = value.encode("ascii")
    if len(encoded) != length:
        raise BuildRefusal(f"invalid {field} length")
    output.extend(encoded)


def _framed_text(output: bytearray, value: str, field: str) -> None:
    encoded = value.encode("utf-8")
    if not encoded or len(encoded) > MAX_STRING_BYTES:
        raise BuildRefusal(f"invalid {field} length")
    output.append(len(encoded))
    output.extend(encoded)


def _place_rows(root: Path) -> list[dict[str, str]]:
    path = root / PLACE_PATH
    payload = _verified_bytes(path, size=PLACE_SIZE, sha256=PLACE_SHA256)
    try:
        text = gzip.decompress(payload).decode("utf-8")
        reader = csv.DictReader(io.StringIO(text, newline=""))
        rows = list(reader)
    except (OSError, UnicodeDecodeError, csv.Error) as error:
        raise BuildRefusal(f"place identity decode: {path}") from error
    if tuple(reader.fieldnames or ()) != PLACE_COLUMNS or len(rows) != PLACE_ROWS:
        raise BuildRefusal("place identity shape")
    return rows


def build_fixture(root: Path, ci_data_dir: Path) -> bytes:
    for path, size, sha256 in PREDECESSOR_CONTRACTS:
        _verified_bytes(root / path, size=size, sha256=sha256)
    p0 = {spec.name: _read_parquet(ci_data_dir / spec.relative_path, spec) for spec in P0_SPECS}
    p1 = {spec.name: _read_parquet(root / spec.relative_path, spec) for spec in P1_SPECS}
    places = _place_rows(root)

    counties = sorted(p0["dim_county"], key=lambda row: str(row["fips"]))
    land = p0["h3_res7_land_mask"]
    population = p0["h3_res7_population"]
    workplace = p0["h3_res7_workplace"]
    county_land = p1["census_county_h3_land_overlap_mi_2023"]
    place_land = p1["census_county_place_h3_land_overlap_mi_2023"]

    counts = (
        len(counties),
        len(places),
        len(land),
        len(population),
        len(workplace),
        len(county_land),
        len(place_land),
    )
    expected = (3_285, 745, 45_572, 22_509, 11_833, 31_881, 4_813)
    if counts != expected:
        raise BuildRefusal(f"section counts: {counts!r}")

    output = bytearray(MAGIC)
    output.extend(struct.pack(">I", FORMAT_VERSION))
    output.extend(REF_DIGEST)
    output.extend(struct.pack(">7I", *counts))

    county_geoids: set[str] = set()
    for row in counties:
        county_id = _unsigned(row["county_id"], 2**32 - 1, "county_id")
        geoid = _ascii(row["fips"], r"\d{5}", "county geoid")
        state_id = _unsigned(row["state_id"], 2**16 - 1, "state_id")
        county_fips = _ascii(row["county_fips"], r"\d{3}", "county_fips")
        name = _text(row["county_name"], "county_name")
        if geoid in county_geoids or geoid[2:] != county_fips or row["h3_res4"] is not None:
            raise BuildRefusal(f"county identity drift: {geoid}")
        county_geoids.add(geoid)
        output.extend(struct.pack(">I", county_id))
        _fixed_ascii(output, geoid, 5, "county geoid")
        output.extend(struct.pack(">H", state_id))
        _fixed_ascii(output, county_fips, 3, "county_fips")
        _framed_text(output, name, "county_name")

    place_geoids: set[str] = set()
    for row in places:
        geoid = _ascii(row["place_geoid"], r"\d{7}", "place_geoid")
        state_fips = _ascii(row["state_fips"], r"\d{2}", "state_fips")
        place_fips = _ascii(row["place_fips"], r"\d{5}", "place_fips")
        place_ns = _ascii(row["place_ns"], r"\d{8}", "place_ns")
        if geoid in place_geoids or geoid != state_fips + place_fips or state_fips != "26":
            raise BuildRefusal(f"place identity drift: {geoid}")
        place_geoids.add(geoid)
        _fixed_ascii(output, geoid, 7, "place_geoid")
        _fixed_ascii(output, state_fips, 2, "state_fips")
        _fixed_ascii(output, place_fips, 5, "place_fips")
        _fixed_ascii(output, place_ns, 8, "place_ns")
        _framed_text(output, _text(row["name"], "place name"), "place name")
        _framed_text(output, _text(row["name_lsad"], "place name_lsad"), "place name_lsad")
        for field, length in (
            ("lsad", 2),
            ("class_fp", 2),
            ("principal_city_indicator", 1),
            ("mtfcc", 5),
            ("functional_status", 1),
        ):
            _fixed_ascii(output, _ascii(row[field], r"[0-9A-Z]+", field), length, field)

    direct_cells: set[int] = set()
    last_cell = -1
    for row in land:
        cell = _cell_text(row["h3_index"])
        county = _ascii(row["county_fips"], r"\d{5}", "source_county_geoid")
        number = float(row["land_fraction"])
        scaled = round(number * 1_000_000)
        if (
            not math.isfinite(number)
            or not 0.0 <= number <= 1.0
            or not math.isclose(number, scaled / 1_000_000, rel_tol=0.0, abs_tol=1e-12)
            or county not in county_geoids
            or cell <= last_cell
        ):
            raise BuildRefusal(f"land fraction drift: {cell:x}")
        last_cell = cell
        direct_cells.add(cell)
        output.extend(struct.pack(">Q", cell))
        _fixed_ascii(output, county, 5, "source_county_geoid")
        output.extend(struct.pack(">I", scaled))

    def append_counts(rows: list[dict[str, object]], column: str) -> int:
        total = 0
        previous = -1
        for row in rows:
            cell = _cell_text(row["h3_index"])
            count = _count(row[column], column)
            if cell not in direct_cells or cell <= previous or count == 0:
                raise BuildRefusal(f"{column} drift: {cell:x}")
            previous = cell
            total += count
            output.extend(struct.pack(">QQ", cell, count))
        return total

    if append_counts(population, "population") != 10_066_869:
        raise BuildRefusal("population total")
    if append_counts(workplace, "jobs") != 3_931_809:
        raise BuildRefusal("workplace total")

    county_land_keys: set[tuple[int, str]] = set()
    denominator: dict[int, int] = {}
    county_total = 0
    previous_county_key: tuple[int, str] | None = None
    for row in county_land:
        cell = _cell_int(row["cell_id"])
        county = _ascii(row["county_fips"], r"\d{5}", "county_fips")
        area = _unsigned(row["land_area_m2"], 2**64 - 1, "land_area_m2")
        key = (cell, county)
        if (
            cell not in direct_cells
            or county not in county_geoids
            or area == 0
            or (previous_county_key is not None and key <= previous_county_key)
        ):
            raise BuildRefusal(f"county land drift: {key!r}")
        previous_county_key = key
        county_land_keys.add(key)
        denominator[cell] = denominator.get(cell, 0) + area
        county_total += area
        output.extend(struct.pack(">Q", cell))
        _fixed_ascii(output, county, 5, "county_fips")
        output.extend(struct.pack(">Q", area))
    if county_total != 146_426_246_267:
        raise BuildRefusal("county land total")

    place_total = 0
    previous_place_key: tuple[int, str, str] | None = None
    for row in place_land:
        cell = _cell_int(row["cell_id"])
        county = _ascii(row["county_fips"], r"\d{5}", "county_fips")
        place = _ascii(row["place_geoid"], r"\d{7}", "place_geoid")
        area = _unsigned(row["place_land_area_m2"], 2**64 - 1, "place_land_area_m2")
        cell_area = _unsigned(row["cell_mi_land_area_m2"], 2**64 - 1, "cell_mi_land_area_m2")
        share = _unsigned(row["place_land_area_share_ppb"], 1_000_000_000, "share_ppb")
        key = (cell, county, place)
        if (
            (cell, county) not in county_land_keys
            or place not in place_geoids
            or area == 0
            or cell_area != denominator[cell]
            or share != area * 1_000_000_000 // cell_area
            or (previous_place_key is not None and key <= previous_place_key)
        ):
            raise BuildRefusal(f"place land drift: {key!r}")
        previous_place_key = key
        place_total += area
        output.extend(struct.pack(">Q", cell))
        _fixed_ascii(output, county, 5, "county_fips")
        _fixed_ascii(output, place, 7, "place_geoid")
        output.extend(struct.pack(">QQI", area, cell_area, share))
    if place_total != 7_689_548_061:
        raise BuildRefusal("place land total")

    return bytes(output)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--ci-data-dir", type=Path, required=True)
    parser.add_argument(
        "--output-prefix",
        type=Path,
        default=Path("rust/crates/babylon-persistence/src/fixtures/spatial_reference_products_v1"),
    )
    args = parser.parse_args()
    root = args.repo_root.resolve()
    output_prefix = (
        args.output_prefix if args.output_prefix.is_absolute() else root / args.output_prefix
    )
    fixture = build_fixture(root, args.ci_data_dir.resolve())
    if len(fixture) != EXPECTED_FIXTURE_BYTES:
        raise BuildRefusal(f"fixture bytes: {len(fixture)}")
    output_prefix.parent.mkdir(parents=True, exist_ok=True)
    outputs = []
    for index, offset in enumerate(range(0, len(fixture), MAX_FIXTURE_PART_BYTES)):
        part = fixture[offset : offset + MAX_FIXTURE_PART_BYTES]
        output = output_prefix.with_name(f"{output_prefix.name}.part-{index:02}.bin")
        output.write_bytes(part)
        outputs.append(output)
    if len(outputs) != EXPECTED_FIXTURE_PARTS:
        raise BuildRefusal(f"fixture parts: {len(outputs)}")
    print(
        "PER278_SPATIAL_REFERENCE_FIXTURE "
        f"bytes={len(fixture)} sha256={hashlib.sha256(fixture).hexdigest()} "
        f"outputs={','.join(str(output) for output in outputs)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
