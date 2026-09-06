#!/usr/bin/env python3
"""Independently verify the bounded ArchiveFoundationGrantsV1 contract corpus."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any, TypedDict

import yaml

FIXTURE_PARTS = [
    "rust/crates/babylon-persistence/src/fixtures/spatial_reference_products_v1.part-00.bin",
    "rust/crates/babylon-persistence/src/fixtures/spatial_reference_products_v1.part-01.bin",
    "rust/crates/babylon-persistence/src/fixtures/spatial_reference_products_v1.part-02.bin",
]
GLOSSARY_FIXTURE_PATH = "contracts/fixtures/glossary_concepts_v1.jsonl"
MAX_CONTRACT_BYTES = 32_768
MAX_GRANT_ROWS = 4_096
MAX_GRANT_KEY_BYTES = 128
MAX_CITATION_SOURCE_ID_BYTES = 128
MAX_CITATION_LOCATOR_BYTES = 4_096
MAX_SPATIAL_FIXTURE_BYTES = 4_194_304
MAX_GLOSSARY_FIXTURE_BYTES = 65_536
MAX_QCEW_ARTIFACT_BYTES = 1_048_576
MAX_TEXT_BYTES = 255
FIXTURE_MAGIC = b"BABYLONSPATREF1\x00"
FIXTURE_VERSION = 1
EXPECTED_COUNTS = (3_285, 745, 45_572, 22_509, 11_833, 31_881, 4_813)
CONCEPT_ID_PATTERN_TEXT = r"^[a-z0-9][a-z0-9-]{0,127}$"
COMPILED_META = {
    "contract": "ArchiveFoundationGrantsV1",
    "version": 1,
    "issue": "PER-23",
    "digest": "SHA-256 diagnostic; exact bytes govern retry equality",
}


class FoundationGrantConstants(TypedDict):
    """Exact types of the compiled contract's heterogeneous constant fields."""

    grant_tick: int
    county_grant_keys: list[str]
    county_qcew_grant_keys: list[str]
    qcew_source_id: str
    qcew_locator_prefix: str
    qcew_artifact_path: str
    qcew_artifact_sha256: str
    place_grant_keys: list[str]
    concept_grant_keys: list[str]
    county_source_id: str
    county_locator_prefix: str
    place_identity_source_id: str
    place_identity_locator_prefix: str
    place_containment_source_id: str
    place_containment_locator_prefix: str
    concept_source_id: str
    concept_locator_prefix: str
    michigan_geoid_prefix: str
    statewide_residual_county_fips: str
    expected_counties: int
    expected_places: int
    expected_concepts: int
    expected_grant_rows: int
    spatial_fixture_digest: str
    glossary_fixture_path: str
    glossary_fixture_sha256: str
    semantic_domain_ascii_nul: str
    semantic_sha256: str


COMPILED_CONSTANTS: FoundationGrantConstants = {
    "grant_tick": 0,
    "county_grant_keys": ["subject", "identity", "containment"],
    "county_qcew_grant_keys": [
        "qcew-establishments",
        "qcew-employment",
        "qcew-total-annual-wages",
        "qcew-average-weekly-wage",
    ],
    "qcew_source_id": "qcew-county-economics-v1",
    "qcew_locator_prefix": "qcew_county_economics_mi_2024.csv.gz#county_geoid=",
    "qcew_artifact_path": "src/babylon/data/reference/economy/qcew_county_economics_mi_2024.csv.gz",
    "qcew_artifact_sha256": "116affb2998c6c0259d5bf14840f99f835d7e0733aa0b4f4c60a257b2723cd16",
    "place_grant_keys": ["subject", "identity", "containment"],
    "concept_grant_keys": ["subject", "identity"],
    "county_source_id": "h3-estate-contract-v1",
    "county_locator_prefix": "dim_county.parquet#fips=",
    "place_identity_source_id": "census-place-authority-v1",
    "place_identity_locator_prefix": "census_place_identity_mi_2023.csv.gz#place_geoid=",
    "place_containment_source_id": "county-place-h3-overlap-v1",
    "place_containment_locator_prefix": (
        "census_county_place_h3_land_overlap_mi_2023.parquet#place_geoid="
    ),
    "concept_source_id": "glossary-concepts-v1",
    "concept_locator_prefix": "contracts/fixtures/glossary_concepts_v1.jsonl#concept_id=",
    "michigan_geoid_prefix": "26",
    "statewide_residual_county_fips": "999",
    "expected_counties": 83,
    "expected_places": 745,
    "expected_concepts": 8,
    "expected_grant_rows": 2_832,
    "spatial_fixture_digest": ("dea8a368d5b7c5a0f1263af2945ffdd94e3914394ec579ca28508661e4464454"),
    "glossary_fixture_path": GLOSSARY_FIXTURE_PATH,
    "glossary_fixture_sha256": ("f47e289dc4e7a11c595f0e42643e352e255775c77dde3a7ed35a91de8d84d85a"),
    "semantic_domain_ascii_nul": "babylon.archive-foundation-grants.v1",
    "semantic_sha256": ("9bb3cdda37dc3dee59242fcfe08f6c3f5bac01ff433988383d459fa0df9dfc65"),
}
COMPILED_BOUNDS = {
    "contract_bytes": MAX_CONTRACT_BYTES,
    "grant_rows": MAX_GRANT_ROWS,
    "grant_key_bytes": MAX_GRANT_KEY_BYTES,
    "citation_source_id_bytes": MAX_CITATION_SOURCE_ID_BYTES,
    "citation_locator_bytes": MAX_CITATION_LOCATOR_BYTES,
    "spatial_fixture_bytes": MAX_SPATIAL_FIXTURE_BYTES,
    "glossary_fixture_bytes": MAX_GLOSSARY_FIXTURE_BYTES,
    "qcew_artifact_bytes": MAX_QCEW_ARTIFACT_BYTES,
}
COMPILED_LAYOUTS = {
    "grant_row_v1": {
        "fields": [
            "subject_kind",
            "subject_id",
            "grant_key",
            "citation_source_id",
            "citation_locator",
            "grant_tick",
        ],
        "order": "sorted by (subject_kind, subject_id, grant_key)",
        "subject_kind_domain": ["county", "place", "concept"],
    },
    "semantic_encoding_v1": {
        "domain": "semantic_domain_ascii_nul plus one trailing NUL byte",
        "row_prefix": "u64 big-endian grant row count",
        "per_row": "five length-prefixed UTF-8 fields then one u64 big-endian grant tick",
        "digest": "SHA-256 over the concatenation",
    },
    "spatial_fixture_v1": {
        "framing": (
            "16-byte magic, u32 big-endian version 1, 32-byte reference digest, "
            "seven u32 big-endian section counts"
        ),
        "counties_row": (
            "u32 county_id, 5-byte geoid, u16 state_id, 3-byte fips, u8-length-framed name"
        ),
        "places_row": (
            "7-byte geoid, 2-byte state_fips, 5-byte place_fips, 8-byte place_ns, "
            "u8-length-framed name, u8-length-framed name_lsad, 2-byte lsad, "
            "2-byte class_fp, 1-byte principal_city_indicator, 5-byte mtfcc, "
            "1-byte functional_status"
        ),
    },
}


class FoundationGrantsRefusal(ValueError):
    """One typed independent-verifier refusal."""

    def __init__(self, code: str, detail: str) -> None:
        self.code = code
        self.detail = detail
        super().__init__(f"{code}: {detail}")


def _bounded_file_bytes(path: Path, maximum: int, code: str) -> bytes:
    try:
        size = path.stat().st_size
    except OSError as error:
        raise FoundationGrantsRefusal("file_read", str(path)) from error
    if size > maximum:
        raise FoundationGrantsRefusal(code, str(size))
    try:
        return path.read_bytes()
    except OSError as error:
        raise FoundationGrantsRefusal("file_read", str(path)) from error


def load_contract(path: Path) -> dict[str, Any]:
    """Load one bounded YAML mapping."""
    raw = _bounded_file_bytes(path, MAX_CONTRACT_BYTES, "schema_too_large")
    try:
        loaded = yaml.safe_load(raw)
    except yaml.YAMLError as error:
        raise FoundationGrantsRefusal("invalid_schema", str(path)) from error
    if not isinstance(loaded, dict):
        raise FoundationGrantsRefusal("invalid_schema", "root mapping")
    return loaded


class _FixtureCursor:
    """Bounded big-endian cursor over the pinned spatial fixture bytes."""

    def __init__(self, data: bytes) -> None:
        self.data = data
        self.offset = 0

    def take(self, length: int, field: str) -> bytes:
        end = self.offset + length
        if end > len(self.data):
            raise FoundationGrantsRefusal("spatial_fixture_shape", field)
        chunk = self.data[self.offset : end]
        self.offset = end
        return chunk

    def u16(self, field: str) -> int:
        return int.from_bytes(self.take(2, field), "big")

    def u32(self, field: str) -> int:
        return int.from_bytes(self.take(4, field), "big")

    def ascii(self, length: int, field: str) -> str:
        raw = self.take(length, field)
        if not all(chr(byte).isascii() and chr(byte).isalnum() for byte in raw):
            raise FoundationGrantsRefusal("spatial_fixture_shape", field)
        return raw.decode("ascii")

    def framed_text(self, field: str) -> str:
        length = self.take(1, field)[0]
        if length == 0 or length > MAX_TEXT_BYTES:
            raise FoundationGrantsRefusal("spatial_fixture_shape", field)
        raw = self.take(length, field)
        if b"\x00" in raw:
            raise FoundationGrantsRefusal("spatial_fixture_shape", field)
        return raw.decode("utf-8")


def load_spatial_subjects(root: Path) -> tuple[list[tuple[str, str]], list[str]]:
    """Parse county (geoid, fips) and place geoid subjects from the fixture."""
    parts = [
        _bounded_file_bytes(root / part, MAX_SPATIAL_FIXTURE_BYTES, "spatial_fixture_too_large")
        for part in FIXTURE_PARTS
    ]
    data = b"".join(parts)
    digest = hashlib.sha256(data).hexdigest()
    if digest != COMPILED_CONSTANTS["spatial_fixture_digest"]:
        raise FoundationGrantsRefusal("spatial_fixture_digest", "composite")
    qcew = _bounded_file_bytes(
        root / COMPILED_CONSTANTS["qcew_artifact_path"],
        MAX_QCEW_ARTIFACT_BYTES,
        "qcew_artifact_too_large",
    )
    if hashlib.sha256(qcew).hexdigest() != COMPILED_CONSTANTS["qcew_artifact_sha256"]:
        raise FoundationGrantsRefusal("qcew_artifact_digest", "county economics")
    cursor = _FixtureCursor(data)
    if cursor.take(16, "magic") != FIXTURE_MAGIC:
        raise FoundationGrantsRefusal("spatial_fixture_shape", "magic")
    if cursor.u32("version") != FIXTURE_VERSION:
        raise FoundationGrantsRefusal("spatial_fixture_shape", "version")
    cursor.take(32, "reference digest")
    counts = tuple(cursor.u32("section count") for _ in range(7))
    if counts != EXPECTED_COUNTS:
        raise FoundationGrantsRefusal("spatial_fixture_shape", "section counts")
    counties: list[tuple[str, str]] = []
    prior_geoid: str | None = None
    for _ in range(counts[0]):
        county_id = cursor.u32("county_id")
        geoid = cursor.ascii(5, "county_geoid")
        state_id = cursor.u16("state_id")
        fips = cursor.ascii(3, "county_fips")
        cursor.framed_text("county_name")
        if (
            county_id == 0
            or state_id == 0
            or geoid[2:] != fips
            or (prior_geoid is not None and prior_geoid >= geoid)
        ):
            raise FoundationGrantsRefusal("spatial_fixture_shape", "counties")
        prior_geoid = geoid
        counties.append((geoid, fips))
    places: list[str] = []
    prior_place: str | None = None
    for _ in range(counts[1]):
        geoid = cursor.ascii(7, "place_geoid")
        state_fips = cursor.ascii(2, "place_state_fips")
        place_fips = cursor.ascii(5, "place_fips")
        cursor.ascii(8, "place_ns")
        cursor.framed_text("place name")
        cursor.framed_text("place name_lsad")
        cursor.ascii(2, "place lsad")
        cursor.ascii(2, "place class_fp")
        cursor.ascii(1, "place principal_city_indicator")
        cursor.ascii(5, "place mtfcc")
        cursor.ascii(1, "place functional_status")
        if (
            state_fips != "26"
            or geoid[:2] != state_fips
            or geoid[2:] != place_fips
            or (prior_place is not None and prior_place >= geoid)
        ):
            raise FoundationGrantsRefusal("spatial_fixture_shape", "places")
        prior_place = geoid
        places.append(geoid)
    return counties, places


def _unique_json_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise FoundationGrantsRefusal("duplicate_json_key", key)
        result[key] = value
    return result


def load_concept_ids(path: Path) -> list[str]:
    """Load the bounded glossary concept ids without an unbounded read."""
    raw = _bounded_file_bytes(path, MAX_GLOSSARY_FIXTURE_BYTES, "glossary_fixture_too_large")
    if hashlib.sha256(raw).hexdigest() != COMPILED_CONSTANTS["glossary_fixture_sha256"]:
        raise FoundationGrantsRefusal("glossary_fixture_digest", str(path))
    lines = raw.splitlines()
    if not lines or len(lines) > 64:
        raise FoundationGrantsRefusal("glossary_fixture_shape", str(len(lines)))
    ids: list[str] = []
    for index, line in enumerate(lines):
        if not line:
            raise FoundationGrantsRefusal("glossary_fixture_shape", str(index + 1))
        try:
            row = json.loads(line, object_pairs_hook=_unique_json_object)
        except (json.JSONDecodeError, UnicodeDecodeError, FoundationGrantsRefusal) as error:
            if isinstance(error, FoundationGrantsRefusal):
                raise FoundationGrantsRefusal(error.code, f"{index + 1}: {error.detail}") from error
            raise FoundationGrantsRefusal("invalid_json", str(index + 1)) from error
        if not isinstance(row, dict) or not isinstance(row.get("concept_id"), str):
            raise FoundationGrantsRefusal("glossary_fixture_shape", str(index + 1))
        ids.append(row["concept_id"])
    return ids


def build_grant_rows(
    counties: list[tuple[str, str]], places: list[str], concept_ids: list[str]
) -> list[tuple[str, str, str, str, str]]:
    """Assemble the canonical (subject_kind, subject_id, grant_key, source, locator) rows."""
    constants = COMPILED_CONSTANTS
    prefix = constants["michigan_geoid_prefix"]
    residual = constants["statewide_residual_county_fips"]
    rows: list[tuple[str, str, str, str, str]] = []
    for geoid, fips in counties:
        if not geoid.startswith(prefix) or fips == residual:
            continue
        for key in constants["county_grant_keys"]:
            rows.append(
                (
                    "county",
                    geoid,
                    key,
                    constants["county_source_id"],
                    f"{constants['county_locator_prefix']}{geoid}",
                )
            )
        for key in constants["county_qcew_grant_keys"]:
            rows.append(
                (
                    "county",
                    geoid,
                    key,
                    constants["qcew_source_id"],
                    f"{constants['qcew_locator_prefix']}{geoid}&sha256={constants['qcew_artifact_sha256']}",
                )
            )
    for geoid in places:
        for key in constants["place_grant_keys"]:
            if key == "containment":
                source = constants["place_containment_source_id"]
                locator = f"{constants['place_containment_locator_prefix']}{geoid}"
            else:
                source = constants["place_identity_source_id"]
                locator = f"{constants['place_identity_locator_prefix']}{geoid}"
            rows.append(("place", geoid, key, source, locator))
    for concept_id in concept_ids:
        for key in constants["concept_grant_keys"]:
            rows.append(
                (
                    "concept",
                    concept_id,
                    key,
                    constants["concept_source_id"],
                    f"{constants['concept_locator_prefix']}{concept_id}",
                )
            )
    rows.sort()
    return rows


def compute_semantic_sha256(rows: list[tuple[str, str, str, str, str]]) -> str:
    """Recompute the canonical sorted grant-row semantic digest."""
    hasher = hashlib.sha256()
    hasher.update(COMPILED_CONSTANTS["semantic_domain_ascii_nul"].encode("ascii") + b"\x00")
    hasher.update(len(rows).to_bytes(8, "big"))
    for kind, subject_id, grant_key, source_id, locator in rows:
        for field in (kind, subject_id, grant_key, source_id, locator):
            encoded = field.encode("utf-8")
            hasher.update(len(encoded).to_bytes(8, "big"))
            hasher.update(encoded)
        hasher.update(COMPILED_CONSTANTS["grant_tick"].to_bytes(8, "big"))
    return hasher.hexdigest()


def _verify_compiled_contract(contract: dict[str, Any]) -> None:
    if contract.get("meta") != COMPILED_META:
        raise FoundationGrantsRefusal("compiled_contract_drift", "meta")
    if contract.get("constants") != COMPILED_CONSTANTS:
        raise FoundationGrantsRefusal("compiled_contract_drift", "constants")
    if contract.get("bounds") != COMPILED_BOUNDS:
        raise FoundationGrantsRefusal("compiled_contract_drift", "bounds")
    if contract.get("layouts") != COMPILED_LAYOUTS:
        raise FoundationGrantsRefusal("compiled_contract_drift", "layouts")
    if contract.get("production_decoder") != "prohibited":
        raise FoundationGrantsRefusal("compiled_contract_drift", "production_decoder")
    required = contract.get("vector_kinds", {}).get("required")
    if required != []:
        raise FoundationGrantsRefusal("compiled_contract_drift", "vector_kinds")


def verify_all(
    contract: dict[str, Any],
    counties: list[tuple[str, str]],
    places: list[str],
    concept_ids: list[str],
) -> list[str]:
    """Verify the assembled grant-row census and return exact mismatches."""
    _verify_compiled_contract(contract)
    constants = contract["constants"]
    errors: list[str] = []
    rows = build_grant_rows(counties, places, concept_ids)
    if len(rows) > MAX_GRANT_ROWS:
        errors.append(f"grant row count exceeds the bound: {len(rows)}")
    mi_counties = [geoid for geoid, fips in counties if geoid.startswith("26") and fips != "999"]
    if len(mi_counties) != constants["expected_counties"]:
        errors.append(
            f"county census: expected {constants['expected_counties']} Michigan counties, "
            f"parsed {len(mi_counties)}"
        )
    if len(places) != constants["expected_places"]:
        errors.append(
            f"place census: expected {constants['expected_places']} Michigan places, "
            f"parsed {len(places)}"
        )
    if len(concept_ids) != constants["expected_concepts"]:
        errors.append(
            f"concept census: expected {constants['expected_concepts']} concepts, "
            f"parsed {len(concept_ids)}"
        )
    if len(rows) != constants["expected_grant_rows"]:
        errors.append(
            f"grant row census: expected {constants['expected_grant_rows']} rows, "
            f"assembled {len(rows)}"
        )
    if concept_ids != sorted(concept_ids) or len(set(concept_ids)) != len(concept_ids):
        errors.append("glossary fixture order: concept ids must be sorted without duplicates")
    for kind, subject_id, grant_key, source_id, locator in rows:
        if len(grant_key.encode("utf-8")) > MAX_GRANT_KEY_BYTES:
            errors.append(f"{kind}:{subject_id}:{grant_key}: grant key exceeds the byte bound")
        if len(source_id.encode("utf-8")) > MAX_CITATION_SOURCE_ID_BYTES:
            errors.append(f"{kind}:{subject_id}:{grant_key}: citation source exceeds the bound")
        if len(locator.encode("utf-8")) > MAX_CITATION_LOCATOR_BYTES:
            errors.append(f"{kind}:{subject_id}:{grant_key}: citation locator exceeds the bound")
        if kind == "county" and not subject_id.startswith(constants["michigan_geoid_prefix"]):
            errors.append(f"county:{subject_id}: non-Michigan county subject")
        if kind == "place" and not subject_id.startswith(constants["michigan_geoid_prefix"]):
            errors.append(f"place:{subject_id}: non-Michigan place subject")
    if compute_semantic_sha256(rows) != constants["semantic_sha256"]:
        errors.append("semantic digest: recomputed digest diverges from the contract constant")
    return errors


def main() -> int:
    """Verify repository contract paths or explicit alternatives."""
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--schema",
        type=Path,
        default=Path("contracts/archive_foundation_grants_v1.yaml"),
    )
    arguments = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    try:
        contract = load_contract(arguments.schema)
        counties, places = load_spatial_subjects(root)
        concept_ids = load_concept_ids(root / GLOSSARY_FIXTURE_PATH)
        errors = verify_all(contract, counties, places, concept_ids)
    except FoundationGrantsRefusal as refusal:
        print(refusal)
        return 1
    for error in errors:
        print(error)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
